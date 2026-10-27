//! Explicit, bounded update discovery. No installer, package download or release HTML.
use crate::{applications, model::*, records};
use anyhow::{Context, Result};
use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::HashSet,
    net::IpAddr,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
};
use url::{Host, Url};

pub const SOURCE_REVIEW_DATE: &str = "2026-10-24";
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const SPARKLE_NS: &str = "http://www.andymatuschak.org/xml-namespaces/sparkle";
const PLAN_KIND: &str = "appUpdatePlan";
pub const REPORT_KIND: &str = "appUpdateReport";

#[derive(Clone, Debug)]
pub struct FetchTarget {
    url: Url,
}
impl FetchTarget {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn host(&self) -> &str {
        self.url.host_str().expect("Validated host")
    }
}
pub struct FetchResponse {
    pub status: u16,
    pub body: Vec<u8>,
}
#[derive(Debug, PartialEq)]
pub enum FetchFailure {
    Cancelled,
    Unavailable,
}
pub type FetchResult = std::result::Result<FetchResponse, FetchFailure>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Seal {
    device: u64,
    inode: u64,
    digest: String,
    candidate: AppUpdateCandidate,
}
#[derive(Serialize, Deserialize)]
struct StoredPlan {
    plan: AppUpdatePlan,
    seals: Vec<Option<Seal>>,
    state: String,
}
pub enum Handoff {
    AppStore,
    VendorApplication(PathBuf),
}

fn safe_text(value: &str, limit: usize) -> Option<String> {
    (!value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control))
        .then(|| value.to_owned())
}
fn identifier(value: &str) -> bool {
    value.len() <= 255
        && value.contains('.')
        && value
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || b".-_".contains(&ch))
}
fn public_host(host: &str) -> bool {
    host.len() <= 253
        && host.contains('.')
        && !host.ends_with('.')
        && ![
            "localhost",
            "local",
            "internal",
            "home",
            "invalid",
            "test",
            "example",
            "arpa",
        ]
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
        && host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == b'-')
        })
}
pub fn validate_feed_url(raw: &str) -> Result<FetchTarget> {
    anyhow::ensure!(
        raw.len() <= 2048 && !raw.chars().any(char::is_control) && !raw.contains('\\'),
        "Unsupported feed URL"
    );
    let url = Url::parse(raw).context("Unsupported feed URL")?;
    anyhow::ensure!(
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url.port_or_known_default() == Some(443),
        "Unsupported feed URL"
    );
    anyhow::ensure!(
        matches!(url.host(), Some(Host::Domain(host)) if public_host(host)),
        "Unsupported feed host"
    );
    Ok(FetchTarget { url })
}
fn store_target(bundle_id: &str) -> Result<FetchTarget> {
    anyhow::ensure!(identifier(bundle_id), "Unsupported bundle identity");
    let mut url = Url::parse("https://itunes.apple.com/lookup")?;
    url.query_pairs_mut()
        .append_pair("bundleId", bundle_id)
        .append_pair("entity", "desktopSoftware")
        .append_pair("country", "us");
    Ok(FetchTarget { url })
}

/// Conservative public-unicast allowlist; special, local and transition ranges fail closed.
pub fn public_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v) => {
            let [a, b, c, _] = v.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192
                    && (b == 168 || (b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(v) => {
            let s = v.segments();
            s[0] & 0xe000 == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && s[1] & 0xf000 == 0)
        }
    }
}

fn receipt(path: &Path) -> Result<bool> {
    let dir = path.join("Contents/_MASReceipt");
    match std::fs::symlink_metadata(&dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => anyhow::bail!("Receipt location unavailable"),
        Ok(meta) => anyhow::ensure!(
            meta.is_dir() && !meta.file_type().is_symlink(),
            "Linked receipt location"
        ),
    }
    #[cfg(target_os = "macos")]
    {
        let directory = crate::safe_fs::directory(&dir)?;
        let name = std::ffi::CString::new("receipt")?;
        return match crate::safe_fs::stat(&directory, &name) {
            Ok(stat) => Ok(stat.st_mode & libc::S_IFMT == libc::S_IFREG),
            Err(_) => Ok(false),
        };
    }
    #[cfg(not(target_os = "macos"))]
    Ok(std::fs::symlink_metadata(dir.join("receipt"))
        .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink()))
}

fn inspect(path: &str) -> Result<Seal> {
    let path = Path::new(path);
    anyhow::ensure!(
        path.is_absolute()
            && path.extension().is_some_and(|e| e == "app")
            && path.canonicalize()? == path,
        "Select an exact unlinked .app path"
    );
    let before = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        before.is_dir()
            && !before.file_type().is_symlink()
            && !crate::storage::placeholder(path, &before),
        "Application metadata unavailable"
    );
    let bytes = applications::info_bytes(path)?;
    let value = plist::Value::from_reader(std::io::Cursor::new(&bytes))?;
    let map = value
        .as_dictionary()
        .context("Invalid application metadata")?;
    let field = |key: &str, max| {
        map.get(key)
            .and_then(plist::Value::as_string)
            .and_then(|s| safe_text(s, max))
            .unwrap_or_default()
    };
    let bundle_id = field("CFBundleIdentifier", 255);
    anyhow::ensure!(identifier(&bundle_id), "Application identity unavailable");
    let (source, source_url, qualification) = if receipt(path)? {
        (AppUpdateSource::AppStore, Some(store_target(&bundle_id)?.url().to_owned()), "Receipt presence routes to Apple's US public catalog; receipt contents and purchase/account eligibility are not read or verified.")
    } else {
        let feed = field("SUFeedURL", 2048);
        match validate_feed_url(&feed) {
            Ok(target) => (AppUpdateSource::Sparkle, Some(target.url().to_owned()), "Declared public HTTPS Sparkle feed only. Coqui does not run Sparkle, send system-profile/version query parameters or verify vendor signatures, compatibility, rollout or hardware eligibility."),
            Err(_) => (AppUpdateSource::Unavailable, None, "No supported App Store route or public HTTPS feed was established. Unsupported or sensitive source URLs are not retained."),
        }
    };
    let after = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        before.dev() == after.dev() && before.ino() == after.ino(),
        "Application changed during review"
    );
    let display = field("CFBundleDisplayName", 255);
    let name = if !display.is_empty() {
        display
    } else {
        let name = field("CFBundleName", 255);
        if !name.is_empty() {
            name
        } else {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        }
    };
    Ok(Seal {
        device: before.dev(),
        inode: before.ino(),
        digest: format!("{:x}", Sha256::digest(&bytes)),
        candidate: AppUpdateCandidate {
            path: path.to_string_lossy().into_owned(),
            name,
            bundle_id,
            installed_version: field("CFBundleShortVersionString", 128),
            installed_build: field("CFBundleVersion", 128),
            source,
            source_url,
            source_review_date: SOURCE_REVIEW_DATE.into(),
            qualification: qualification.into(),
        },
    })
}
fn unavailable_candidate(path: String) -> AppUpdateCandidate {
    AppUpdateCandidate { path, name:"Application metadata unavailable".into(), bundle_id:String::new(), installed_version:String::new(), installed_build:String::new(), source:AppUpdateSource::Unavailable, source_url:None, source_review_date:SOURCE_REVIEW_DATE.into(), qualification:"Exact bounded application metadata could not be reviewed; no network source was retained.".into() }
}
pub fn prepare(c: &Connection, paths: Vec<String>, cancel: &AtomicBool) -> Result<AppUpdatePlan> {
    anyhow::ensure!(
        (1..=10).contains(&paths.len()),
        "Select one to ten applications"
    );
    let mut unique = HashSet::new();
    anyhow::ensure!(
        paths.iter().all(|p| unique.insert(p.clone())
            && p.len() <= 4096
            && Path::new(p).is_absolute()
            && !p.chars().any(char::is_control)),
        "Select unique exact application paths"
    );
    let mut candidates = Vec::new();
    let mut seals = Vec::new();
    for path in paths {
        anyhow::ensure!(!cancel.load(AtomicOrdering::SeqCst), "Review cancelled");
        let seal = inspect(&path).ok();
        candidates.push(
            seal.as_ref()
                .map(|s| s.candidate.clone())
                .unwrap_or_else(|| unavailable_candidate(path)),
        );
        seals.push(seal);
    }
    let plan = AppUpdatePlan { id:uuid::Uuid::new_v4().to_string(), created_at:now(), expires_at:(chrono::Utc::now()+chrono::Duration::minutes(10)).to_rfc3339(), candidates, qualification:"Review the exact applications and disclosed network destinations. Local identity is an Info.plist/metadata seal, not code-signature authentication. A single check requires separate network acknowledgement within ten minutes. Sources were reviewed during development on 2026-10-24, not refreshed by preparing this plan. No installation or download is authorized.".into() };
    anyhow::ensure!(!cancel.load(AtomicOrdering::SeqCst), "Review cancelled");
    let stored = StoredPlan {
        plan: plan.clone(),
        seals,
        state: "review".into(),
    };
    records::save(
        c,
        PLAN_KIND,
        &plan.id,
        &plan.created_at,
        "review",
        "Application update review",
        &stored,
    )?;
    Ok(plan)
}

fn numeric(value: &str) -> Option<Vec<u64>> {
    if value.is_empty() || value.len() > 128 {
        return None;
    }
    let mut parts = value
        .split('.')
        .map(|p| {
            (!p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
                .then(|| p.parse().ok())
                .flatten()
        })
        .collect::<Option<Vec<u64>>>()?;
    if parts.len() > 16 {
        return None;
    }
    while parts.len() > 1 && parts.last() == Some(&0) {
        parts.pop();
    }
    Some(parts)
}
fn compare(offered: &str, installed: &str) -> Option<Ordering> {
    let a = numeric(offered)?;
    let b = numeric(installed)?;
    Some(
        a.iter()
            .copied()
            .chain(std::iter::repeat(0))
            .take(a.len().max(b.len()))
            .cmp(
                b.iter()
                    .copied()
                    .chain(std::iter::repeat(0))
                    .take(a.len().max(b.len())),
            ),
    )
}
#[derive(Default)]
struct Offer {
    version: Option<String>,
    build: Option<String>,
    published: Option<String>,
    minimum: Option<String>,
    channel: Option<String>,
    skip: bool,
    full_enclosure: bool,
}
fn clean(value: &str) -> Option<String> {
    safe_text(value.trim(), 128)
}
fn apple(body: &[u8], bundle_id: &str) -> Result<Offer> {
    let value: serde_json::Value = serde_json::from_slice(body)?;
    let results = value
        .get("results")
        .and_then(|v| v.as_array())
        .context("Catalog response unavailable")?;
    anyhow::ensure!(
        results.len() <= 50,
        "Catalog response exceeded review bound"
    );
    let matches = results
        .iter()
        .filter(|v| {
            v.get("bundleId").and_then(|v| v.as_str()) == Some(bundle_id)
                && v.get("kind").and_then(|v| v.as_str()) == Some("mac-software")
        })
        .collect::<Vec<_>>();
    anyhow::ensure!(
        matches.len() == 1,
        "No unique exact desktop application match in the US catalog"
    );
    let item = matches[0];
    Ok(Offer {
        version: item.get("version").and_then(|v| v.as_str()).and_then(clean),
        minimum: item
            .get("minimumOsVersion")
            .and_then(|v| v.as_str())
            .and_then(clean),
        published: item
            .get("currentVersionReleaseDate")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.to_rfc3339()),
        ..Offer::default()
    })
}
fn set_field(offer: &mut Offer, name: &str, value: String) -> Result<()> {
    let slot = match name {
        "version" => &mut offer.build,
        "shortVersionString" => &mut offer.version,
        "minimumSystemVersion" => &mut offer.minimum,
        "channel" => &mut offer.channel,
        _ => return Ok(()),
    };
    anyhow::ensure!(
        slot.as_ref().is_none_or(|old| old == &value),
        "Conflicting feed fields"
    );
    *slot = Some(value);
    Ok(())
}
fn sparkle(body: &[u8]) -> Result<Offer> {
    let source = std::str::from_utf8(body)?;
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(true);
    reader.resolver_mut().set_max_namespace_bindings(64);
    // Namespace URI, not a preferred prefix, identifies Sparkle metadata.
    // The booleans record unnamespaced RSS versus Sparkle elements.
    let mut stack: Vec<(String, bool, bool)> = Vec::new();
    let mut current: Option<Offer> = None;
    let mut offers = Vec::new();
    let mut text_field: Option<String> = None;
    let mut events = 0;
    let mut root_seen = false;
    loop {
        events += 1;
        anyhow::ensure!(events <= 50_000, "Feed event bound exceeded");
        let event = reader.read_event()?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::DocType(_) | Event::GeneralRef(_) => {
                anyhow::bail!("DTD and entity references are unsupported")
            }
            Event::Start(e) | Event::Empty(e) => {
                anyhow::ensure!(
                    text_field.is_none(),
                    "Nested feed comparison fields are unsupported"
                );
                let (ns, local) = reader.resolver().resolve_element(e.name());
                anyhow::ensure!(
                    !matches!(&ns, ResolveResult::Unknown(_)),
                    "Unbound feed namespace"
                );
                let sparkle = matches!(&ns,ResolveResult::Bound(n) if n.as_ref()==SPARKLE_NS);
                let plain = matches!(&ns, ResolveResult::Unbound);
                let name = local.as_ref().to_owned();
                if stack.is_empty() {
                    anyhow::ensure!(
                        !root_seen && plain && name == "rss",
                        "A single RSS root is required"
                    );
                    root_seen = true;
                }
                let security_field = matches!(
                    name.as_str(),
                    "version"
                        | "shortVersionString"
                        | "minimumSystemVersion"
                        | "channel"
                        | "deltas"
                );
                let rss_channel = plain
                    && name == "channel"
                    && stack.len() == 1
                    && stack[0].0 == "rss"
                    && stack[0].1;
                anyhow::ensure!(
                    !security_field || sparkle || rss_channel,
                    "Unrecognized Sparkle namespace"
                );
                if plain
                    && name == "item"
                    && stack
                        .iter()
                        .map(|(n, p, _)| (n.as_str(), *p))
                        .eq([("rss", true), ("channel", true)])
                {
                    anyhow::ensure!(
                        offers.len() < 500 && current.is_none(),
                        "Feed item bound exceeded"
                    );
                    current = Some(Offer::default());
                }
                if let Some(offer) = current.as_mut() {
                    let in_deltas = stack.iter().any(|(n, _, s)| *s && n == "deltas");
                    let direct_child =
                        stack.len() == 3 && stack.last().is_some_and(|(n, p, _)| n == "item" && *p);
                    if plain && name == "enclosure" && !in_deltas && direct_child {
                        let mut enclosure = Offer::default();
                        let mut has_url = false;
                        for (i, attr) in e.attributes().enumerate() {
                            anyhow::ensure!(i < 128, "Feed attribute bound exceeded");
                            let attr = attr?;
                            let (ans, aname) = reader.resolver().resolve_attribute(attr.key);
                            if matches!(&ans, ResolveResult::Unbound)
                                && aname.as_ref() == "url"
                                && !attr.value.trim().is_empty()
                            {
                                has_url = true;
                            }
                            let relevant = matches!(
                                aname.as_ref(),
                                "version" | "shortVersionString" | "deltaFrom" | "os"
                            );
                            if relevant {
                                anyhow::ensure!(
                                    matches!(ans,ResolveResult::Bound(n) if n.as_ref()==SPARKLE_NS),
                                    "Unrecognized Sparkle attribute namespace"
                                );
                                anyhow::ensure!(
                                    !attr.value.contains('&'),
                                    "Entity references unsupported in feed metadata"
                                );
                                let value = clean(&attr.value).context("Invalid feed metadata")?;
                                if aname.as_ref() == "deltaFrom"
                                    || (aname.as_ref() == "os" && value != "macos")
                                {
                                    enclosure.skip = true;
                                } else {
                                    set_field(&mut enclosure, aname.as_ref(), value)?;
                                }
                            }
                        }
                        if !enclosure.skip && has_url {
                            offer.full_enclosure = true;
                            if let Some(build) = enclosure.build {
                                set_field(offer, "version", build)?;
                            }
                            if let Some(version) = enclosure.version {
                                set_field(offer, "shortVersionString", version)?;
                            }
                        }
                    }
                    if direct_child
                        && !in_deltas
                        && name != "deltas"
                        && ((sparkle && security_field) || (plain && name == "pubDate"))
                    {
                        text_field = Some(name.clone());
                    }
                }
                if !empty {
                    stack.push((name, plain, sparkle));
                    anyhow::ensure!(stack.len() <= 64, "Feed depth exceeded");
                }
            }
            Event::Text(e) => {
                if let (Some(field), Some(offer)) = (&text_field, current.as_mut()) {
                    let value = clean(e.as_ref()).context("Invalid feed field")?;
                    if field == "pubDate" {
                        offer.published = chrono::DateTime::parse_from_rfc2822(&value)
                            .ok()
                            .map(|d| d.to_rfc3339());
                    } else {
                        set_field(offer, field, value)?;
                    }
                }
            }
            Event::CData(_) if text_field.is_some() => {
                anyhow::bail!("CDATA comparison fields are unsupported")
            }
            Event::End(e) => {
                let name = e.local_name();
                if name.as_ref() == "item" && stack.len() == 3 && current.is_some() {
                    offers.push(current.take().unwrap());
                }
                text_field = None;
                stack.pop().context("Unbalanced feed")?;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    anyhow::ensure!(root_seen && stack.is_empty(), "Incomplete feed");
    let mut selected: Option<Offer> = None;
    for offer in offers.into_iter().filter(|o| {
        !o.skip
            && o.full_enclosure
            && o.channel.as_deref().is_none_or(|v| v == "default")
            && o.build.is_some()
    }) {
        let build = offer.build.as_deref().unwrap();
        if numeric(build).is_none() {
            return Ok(offer);
        }
        if selected.as_ref().is_none_or(|old| {
            compare(build, old.build.as_deref().unwrap()) == Some(Ordering::Greater)
        }) {
            selected = Some(offer);
        }
    }
    selected.context("No comparable default-channel full release was observed")
}

fn pending(candidate: &AppUpdateCandidate) -> AppUpdateResult {
    AppUpdateResult {
        path: candidate.path.clone(),
        name: candidate.name.clone(),
        bundle_id: candidate.bundle_id.clone(),
        installed_version: candidate.installed_version.clone(),
        installed_build: candidate.installed_build.clone(),
        source: candidate.source.clone(),
        source_url: candidate.source_url.clone(),
        source_review_date: candidate.source_review_date.clone(),
        state: AppUpdateState::Unavailable,
        checked_at: None,
        offered_version: None,
        offered_build: None,
        published_at: None,
        min_system_version: None,
        qualification: "Not checked; awaiting this explicitly acknowledged operation.".into(),
    }
}
fn save_report(c: &Connection, report: &AppUpdateReport) -> Result<()> {
    records::save(
        c,
        REPORT_KIND,
        &report.id,
        &report.created_at,
        &report.status,
        "Application update discovery",
        report,
    )
}
pub fn check_with_fetcher(
    c: &Connection,
    plan_id: &str,
    acknowledged: bool,
    cancel: &AtomicBool,
    mut send: impl FnMut(AppUpdateReport) -> bool,
    mut fetch: impl FnMut(&FetchTarget, &AtomicBool) -> FetchResult,
) -> Result<AppUpdateReport> {
    anyhow::ensure!(
        acknowledged,
        "Explicit network acknowledgement is required for this check"
    );
    let stored: StoredPlan = records::get(c, PLAN_KIND, plan_id)?;
    anyhow::ensure!(
        stored.state == "review"
            && chrono::DateTime::parse_from_rfc3339(&stored.plan.expires_at)? > chrono::Utc::now(),
        "Review expired or already used; prepare a new review"
    );
    anyhow::ensure!(
        !cancel.load(AtomicOrdering::SeqCst),
        "Check cancelled before network"
    );
    for seal in stored.seals.iter().flatten() {
        anyhow::ensure!(
            inspect(&seal.candidate.path).ok().as_ref() == Some(seal),
            "Application identity, version or source changed; prepare a new review"
        );
    }
    anyhow::ensure!(c.execute("UPDATE records SET status='consumed',json=json_set(json,'$.state','consumed') WHERE id=?1 AND kind=?2 AND status='review'",params![plan_id,PLAN_KIND])?==1,"Review already used");
    let mut report=AppUpdateReport{id:uuid::Uuid::new_v4().to_string(),plan_id:plan_id.into(),created_at:now(),finished_at:None,status:"running".into(),items:stored.plan.candidates.iter().map(pending).collect(),qualification:"Explicit network discovery only. US public App Store catalog and declared default-channel Sparkle metadata can be incomplete, stale or unavailable. Strict dotted-numeric comparisons do not establish compatibility, vendor signature validity, rollout eligibility, purchase rights or installation. No package/download URLs or release HTML are retained; installation stays in the vendor app or App Store.".into()};
    save_report(c, &report)?;
    if !send(report.clone()) {
        cancel.store(true, AtomicOrdering::SeqCst);
    }
    for index in 0..report.items.len() {
        let item = &mut report.items[index];
        if cancel.load(AtomicOrdering::SeqCst) {
            break;
        }
        if item.source == AppUpdateSource::Unavailable {
            item.qualification = "No supported source was reviewed; no request was sent.".into();
            continue;
        }
        let fresh = stored
            .seals
            .iter()
            .flatten()
            .find(|s| s.candidate.path == item.path)
            .context("Missing review seal")?;
        if inspect(&item.path).ok().as_ref() != Some(fresh) {
            item.qualification =
                "Local application metadata changed before the request; no request was sent."
                    .into();
            continue;
        }
        let target = if item.source == AppUpdateSource::AppStore {
            store_target(&item.bundle_id)
        } else {
            validate_feed_url(item.source_url.as_deref().unwrap_or(""))
        }?;
        let response = fetch(&target, cancel);
        if response.is_ok() && !cancel.load(AtomicOrdering::SeqCst) {
            item.checked_at = Some(now());
        }
        let offer = match response {
            Err(FetchFailure::Cancelled) => {
                cancel.store(true, AtomicOrdering::SeqCst);
                None
            }
            Err(_) => None,
            Ok(response) if response.status == 200 && response.body.len() <= MAX_RESPONSE_BYTES => {
                if item.source == AppUpdateSource::AppStore {
                    apple(&response.body, &item.bundle_id).ok()
                } else {
                    sparkle(&response.body).ok()
                }
            }
            Ok(_) => None,
        };
        if let Some(offer) = offer {
            let compared = if item.source == AppUpdateSource::AppStore {
                offer
                    .version
                    .as_deref()
                    .and_then(|v| compare(v, &item.installed_version))
            } else {
                offer
                    .build
                    .as_deref()
                    .and_then(|v| compare(v, &item.installed_build))
            };
            item.state = match compared {
                Some(Ordering::Greater) => AppUpdateState::Available,
                Some(_) => AppUpdateState::NoNewerVersionObserved,
                None => AppUpdateState::Uncomparable,
            };
            item.offered_version = offer.version;
            item.offered_build = offer.build;
            item.published_at = offer.published;
            item.min_system_version = offer.minimum;
            item.qualification="Only the reviewed source response and strict dotted-numeric comparison were observed. No newer version observed is not an up-to-date or healthy verdict. OS/hardware compatibility, rollout and vendor/account eligibility remain unverified. Use the vendor app or App Store to decide and install.".into();
        } else {
            item.qualification="The proxy/network policy, bounded request or exact source response was unavailable, unsupported or cancelled. Use the vendor tool when this checker cannot preserve network policy. An empty catalog, redirect, malformed feed or missing identity match establishes no current-version verdict.".into();
        }
        // Persist each completed result before asking the consumer to continue.
        save_report(c, &report)?;
        if !send(report.clone()) {
            cancel.store(true, AtomicOrdering::SeqCst);
        }
    }
    if cancel.load(AtomicOrdering::SeqCst) {
        report.status = "cancelled".into();
        for item in &mut report.items {
            if item.checked_at.is_none() || item.state == AppUpdateState::Unavailable {
                item.state = AppUpdateState::Cancelled;
                item.checked_at = None;
                item.offered_version = None;
                item.offered_build = None;
                item.published_at = None;
                item.min_system_version = None;
                item.qualification =
                    "Check cancelled; no completed comparison was established for this item."
                        .into();
            }
        }
    } else {
        report.status = if report.items.iter().all(|i| {
            matches!(
                i.state,
                AppUpdateState::Available | AppUpdateState::NoNewerVersionObserved
            )
        }) {
            "complete"
        } else {
            "partial"
        }
        .into();
    }
    report.finished_at = Some(now());
    save_report(c, &report)?;
    if !send(report.clone()) && report.status != "cancelled" {
        report.status = "cancelled".into();
        report
            .qualification
            .push_str(" The result consumer disconnected; this operation is incomplete.");
        save_report(c, &report)?;
    }
    Ok(report)
}
pub fn latest(c: &Connection) -> Result<Option<AppUpdateReport>> {
    records::latest(c, REPORT_KIND)
}
pub fn handoff(c: &Connection, report_id: &str, path: &str) -> Result<Handoff> {
    let report: AppUpdateReport = records::get(c, REPORT_KIND, report_id)?;
    anyhow::ensure!(
        ["complete", "partial"].contains(&report.status.as_str()),
        "Choose a finished update observation"
    );
    let item = report
        .items
        .iter()
        .find(|i| i.path == path && i.state == AppUpdateState::Available)
        .context("No available update observation for that application")?;
    let stored: StoredPlan = records::get(c, PLAN_KIND, &report.plan_id)?;
    let seal = stored
        .seals
        .iter()
        .flatten()
        .find(|s| s.candidate.path == path)
        .context("Review identity unavailable")?;
    anyhow::ensure!(
        inspect(path).ok().as_ref() == Some(seal)
            && item.bundle_id == seal.candidate.bundle_id
            && item.source == seal.candidate.source
            && item.source_url == seal.candidate.source_url,
        "Application identity or source changed; prepare a new review"
    );
    match item.source {
        AppUpdateSource::AppStore => Ok(Handoff::AppStore),
        AppUpdateSource::Sparkle => Ok(Handoff::VendorApplication(path.into())),
        _ => anyhow::bail!("No validated update handoff"),
    }
}
