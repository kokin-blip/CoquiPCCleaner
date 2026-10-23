//! Independently implemented, conservative bundle identity associations. No fuzzy cleanup authority.
use crate::{model::now, removal::Policy, storage};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationData {
    pub path: String,
    pub association: String,
    pub confidence: String,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub eligible: bool,
    pub consequences: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationFootprint {
    pub path: String,
    pub name: String,
    pub bundle_id: String,
    pub version: String,
    pub observed_at: String,
    pub protection: Vec<String>,
    pub related: Vec<ApplicationData>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationPreferences {
    pub protected_paths: Vec<String>,
    pub disappearance_notices: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationPage {
    pub applications: Vec<ApplicationFootprint>,
    pub offset: u32,
    pub total: u32,
    pub qualification: String,
}
fn info(path: &Path) -> Result<plist::Value> {
    let contents = fs::symlink_metadata(path.join("Contents"))?;
    anyhow::ensure!(
        contents.is_dir() && !contents.file_type().is_symlink(),
        "Linked bundle Contents is not eligible"
    );
    let p = path.join("Contents/Info.plist");
    let before = fs::symlink_metadata(&p)?;
    anyhow::ensure!(
        before.is_file()
            && !before.file_type().is_symlink()
            && before.len() <= 1024 * 1024
            && !storage::placeholder(&p, &before),
        "Bundle metadata unavailable, linked or cloud-only"
    );
    let mut f = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&p)?;
    let m = f.metadata()?;
    anyhow::ensure!(
        m.dev() == before.dev() && m.ino() == before.ino() && !storage::placeholder(&p, &m),
        "Bundle metadata changed"
    );
    let mut bytes = vec![];
    (&mut f).take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let after = f.metadata()?;
    anyhow::ensure!(
        bytes.len() <= 1024 * 1024
            && after.len() == m.len()
            && after.mtime() == m.mtime()
            && after.mtime_nsec() == m.mtime_nsec()
            && after.ctime() == m.ctime()
            && after.ctime_nsec() == m.ctime_nsec(),
        "Bundle metadata changed or exceeds the read limit"
    );
    Ok(plist::Value::from_reader(std::io::Cursor::new(bytes))?)
}
#[cfg(target_os = "macos")]
fn running() -> Result<Vec<String>> {
    unsafe extern "C" {
        fn coqui_running_applications() -> *mut std::ffi::c_char;
    }
    let p = unsafe { coqui_running_applications() };
    anyhow::ensure!(!p.is_null(), "Running applications unavailable");
    let value = unsafe { std::ffi::CStr::from_ptr(p) }
        .to_string_lossy()
        .into_owned();
    unsafe { libc::free(p.cast()) };
    Ok(serde_json::from_str(&value)?)
}
#[cfg(not(target_os = "macos"))]
fn running() -> Result<Vec<String>> {
    anyhow::bail!("Running application protection requires macOS")
}
fn id_safe(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 255
        && s.contains('.')
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_')
}
pub fn inspect(path: &Path, policy: &Policy) -> Result<ApplicationFootprint> {
    let friendly = crate::volumes::friendly(path);
    let path = friendly.as_path();
    anyhow::ensure!(
        path.is_absolute() && path.extension().is_some_and(|e| e == "app"),
        "Select a macOS .app bundle"
    );
    let canonical = path.canonicalize()?;
    anyhow::ensure!(
        canonical == path && !fs::symlink_metadata(path)?.file_type().is_symlink(),
        "Linked application paths are not eligible"
    );
    let m = fs::symlink_metadata(path)?;
    anyhow::ensure!(
        m.is_dir() && !storage::placeholder(path, &m),
        "Application is unavailable or cloud-only"
    );
    let value = info(path)?;
    let d = value.as_dictionary().context("Invalid bundle metadata")?;
    let string = |key: &str| {
        d.get(key)
            .and_then(plist::Value::as_string)
            .unwrap_or("")
            .to_string()
    };
    let bundle_id = string("CFBundleIdentifier");
    anyhow::ensure!(id_safe(&bundle_id), "Bundle identity is missing or invalid");
    let mut protection = vec![];
    if let Some(parent) = path.parent() {
        let home = policy.home.canonicalize()?;
        let project = if parent.starts_with(&home) {
            crate::cleanup::cache_project_context(parent, &home).is_err()
        } else {
            parent.ancestors().any(crate::removal::project_marker)
        };
        if project {
            protection.push("Application is inside a project or repository boundary; review it with project/Git tooling instead".into());
        }
    }

    if bundle_id.starts_with("com.apple.")
        || path.starts_with("/System")
        || path.starts_with("/Library/Apple")
    {
        protection.push("macOS component; manage through Apple-supported interfaces".into());
    }
    if policy.self_app.as_ref().is_some_and(|p| p == path)
        || bundle_id == "com.coqui.CoquiPCCleaner"
    {
        protection.push("Coqui cannot remove itself".into());
    }
    match running() {
        Ok(paths) => {
            if paths.iter().any(|p| Path::new(p) == path) {
                protection.push("Application is running; quit it and inspect again".into())
            }
        }
        Err(_) => {
            protection.push("Running application state is unavailable; retry before removal".into())
        }
    }
    if m.uid() != unsafe { libc::geteuid() } {
        protection.push(
            "Administrator-owned bundle; removal is unavailable in this portable build".into(),
        );
    }
    if policy
        .exclusions
        .iter()
        .any(|p| crate::volumes::same_or_below(path, p))
    {
        protection.push("Application path is excluded".into());
    }
    for relative in [
        "Contents/Library/LoginItems",
        "Contents/Library/LaunchServices",
        "Contents/Library/SystemExtensions",
        "Contents/PlugIns",
    ] {
        let p = path.join(relative);
        if fs::read_dir(p).is_ok_and(|mut d| d.next().is_some()) {
            protection.push(format!(
                "Contains services or extensions ({relative}); use vendor uninstall guidance"
            ));
        }
    }
    let lower = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if lower.contains("uninstall") {
        protection.push("Vendor uninstaller; open its documented workflow instead".into());
    }
    for base in [
        policy.home.join("Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchDaemons"),
    ] {
        if fs::read_dir(base).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(&bundle_id))
        }) {
            protection.push(
                "Related launch configuration detected; vendor uninstall guidance required".into(),
            );
        }
    }
    let mut related = vec![];
    for (relative,confidence,consequences) in [
  (format!("Library/Caches/{bundle_id}"),"Bundle identity","Cached data may need regeneration or downloading."),
  (format!("Library/Preferences/{bundle_id}.plist"),"Bundle identity","Removes preferences; the app may reset its configuration."),
  (format!("Library/Saved Application State/{bundle_id}.savedState"),"Bundle identity","Removes saved windows and application state."),
  (format!("Library/Application Support/{bundle_id}"),"Bundle identity","May contain unique documents, databases, licenses or account data. Review contents independently."),
  (format!("Library/Containers/{bundle_id}"),"Unconfirmed container","Sandbox containers may contain unique or shared data. No automatic removal authority."),
 ] {
  let p=policy.home.join(relative);let Ok(m)=fs::symlink_metadata(&p) else{continue};
  let eligible=confidence=="Bundle identity" && m.uid()==unsafe{libc::geteuid()} && !m.file_type().is_symlink() && !storage::placeholder(&p,&m);
  related.push(ApplicationData{path:p.to_string_lossy().into_owned(),association:format!("Exact path component equals {bundle_id}"),confidence:confidence.into(),logical_bytes:if m.is_file(){m.len() as f64}else{0.},allocated_bytes:if m.is_file(){m.blocks() as f64*512.}else{0.},eligible,consequences:format!("{consequences} Folder totals are measured during removal review.")});
 }
    // Group containers and names alone are deliberately never assigned to an app.
    let name = string("CFBundleDisplayName");
    let name = if name.is_empty() {
        string("CFBundleName")
    } else {
        name
    };
    Ok(ApplicationFootprint{path:path.to_string_lossy().into_owned(),name:if name.is_empty(){path.file_stem().unwrap_or_default().to_string_lossy().into_owned()}else{name},bundle_id,version:string("CFBundleShortVersionString"),observed_at:now(),protection,related,qualification:"Bundle metadata and exact bundle-ID paths only. Shared group containers, fuzzy names and external services are not established as disposable. Removing the bundle is not a complete vendor uninstall.".into()})
}
pub fn preferences(c: &rusqlite::Connection) -> Result<ApplicationPreferences> {
    use rusqlite::OptionalExtension;
    let s: Option<String> = c
        .query_row(
            "SELECT json FROM application_preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(s.map(|s| serde_json::from_str(&s))
        .transpose()?
        .unwrap_or_default())
}
pub fn save_preferences(c: &rusqlite::Connection, p: &ApplicationPreferences) -> Result<()> {
    anyhow::ensure!(
        p.protected_paths.len() <= 1000
            && p.protected_paths.iter().all(|p| Path::new(p).is_absolute()),
        "Invalid protected application paths"
    );
    c.execute("INSERT INTO application_preferences VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(p)?])?;
    Ok(())
}
pub fn page(
    c: &rusqlite::Connection,
    policy: &Policy,
    prefs: &ApplicationPreferences,
    query: &str,
    sort: &str,
    offset: u32,
) -> Result<ApplicationPage> {
    let mut paths = vec![];
    let mut omissions = 0;
    for root in [
        PathBuf::from("/Applications"),
        policy.home.join("Applications"),
        PathBuf::from("/System/Applications"),
    ] {
        let mut it = walkdir::WalkDir::new(root)
            .max_depth(3)
            .follow_links(false)
            .into_iter();
        while let Some(next) = it.next() {
            match next {
                Ok(e) => {
                    if e.path().extension().is_some_and(|e| e == "app") {
                        paths.push(e.path().to_path_buf());
                        it.skip_current_dir();
                    }
                }
                Err(_) => omissions += 1,
            }
        }
    }
    paths.sort();
    paths.dedup();
    let mut apps = vec![];
    for p in paths {
        match inspect(&p, policy) {
            Ok(mut app) => {
                c.execute("INSERT INTO application_observations VALUES(?1,?2,NULL) ON CONFLICT(path) DO UPDATE SET json=excluded.json,missing_at=NULL",rusqlite::params![app.path,serde_json::to_string(&app)?])?;
                if prefs.protected_paths.contains(&app.path) {
                    app.protection.push("User-protected application".into());
                }
                if format!("{} {} {}", app.name, app.bundle_id, app.path)
                    .to_lowercase()
                    .contains(&query.to_lowercase())
                {
                    apps.push(app);
                }
            }
            Err(_) => omissions += 1,
        }
    }
    if sort == "path" {
        apps.sort_by(|a, b| a.path.cmp(&b.path));
    } else {
        apps.sort_by_key(|a| a.name.to_lowercase());
    }
    let total = apps.len() as u32;
    Ok(ApplicationPage{applications:apps.into_iter().skip(offset as usize).take(50).collect(),offset,total,qualification:format!("Installed bundle inventory in system, shared and user Applications folders (up to three levels). {omissions} unavailable entries. Select another .app explicitly to inspect it. Detected configuration does not prove enabled services.")})
}

/// Only exact absence at a previously observed path; permission errors remain unknown.
pub fn refresh_missing(c: &rusqlite::Connection) -> Result<()> {
    let mut q = c.prepare("SELECT path FROM application_observations")?;
    let paths = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for p in paths {
        match fs::symlink_metadata(&p) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                c.execute("UPDATE application_observations SET missing_at=COALESCE(missing_at,?1) WHERE path=?2",rusqlite::params![now(),p])?;
            }
            Ok(_) => {
                c.execute(
                    "UPDATE application_observations SET missing_at=NULL WHERE path=?1",
                    [p],
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}
pub fn leftovers(c: &rusqlite::Connection, offset: u32) -> Result<Vec<ApplicationFootprint>> {
    refresh_missing(c)?;
    let mut q=c.prepare("SELECT json FROM application_observations WHERE missing_at IS NOT NULL ORDER BY missing_at DESC LIMIT 50 OFFSET ?1")?;
    let rows = q.query_map([offset], |r| r.get::<_, String>(0))?;
    rows.map(|r|{let mut app:ApplicationFootprint=serde_json::from_str(&r?)?;app.protection=vec!["Previously observed bundle is missing at this path. It may have moved or its volume may be disconnected. These retained associations have no removal authority.".into()];for item in &mut app.related {item.eligible=false;}Ok(app)}).collect()
}
pub fn notices(c: &rusqlite::Connection) -> Result<u32> {
    if !preferences(c)?.disappearance_notices {
        return Ok(0);
    }
    refresh_missing(c)?;
    Ok(c.query_row(
        "SELECT count(*) FROM application_observations WHERE missing_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?)
}

/// Search/sort/page retained inventory; native collection happens only on explicit refresh.
pub fn indexed_page(
    c: &rusqlite::Connection,
    query: &str,
    sort: &str,
    offset: u32,
    protected_only: bool,
) -> Result<ApplicationPage> {
    let prefs = preferences(c)?;
    let paths = serde_json::to_string(&prefs.protected_paths)?;
    let filter="missing_at IS NULL AND instr(lower(json_extract(json,'$.name')||' '||json_extract(json,'$.bundleId')||' '||path),lower(?1))>0 AND (?2=0 OR json_array_length(json,'$.protection')>0 OR EXISTS(SELECT 1 FROM json_each(?3) p WHERE p.value=application_observations.path))";
    let total: u32 = c.query_row(
        &format!("SELECT count(*) FROM application_observations WHERE {filter}"),
        rusqlite::params![query, protected_only, paths],
        |r| r.get(0),
    )?;
    let order = if sort == "path" {
        "path"
    } else {
        "lower(json_extract(json,'$.name'))"
    };
    let mut q=c.prepare(&format!("SELECT json FROM application_observations WHERE {filter} ORDER BY {order},path LIMIT 50 OFFSET ?4"))?;
    let rows = q.query_map(
        rusqlite::params![query, protected_only, paths, offset],
        |r| r.get::<_, String>(0),
    )?;
    let mut apps = vec![];
    for row in rows {
        let mut app: ApplicationFootprint = serde_json::from_str(&row?)?;
        if prefs.protected_paths.contains(&app.path) {
            app.protection.push("User-protected application".into());
        }
        apps.push(app);
    }
    Ok(ApplicationPage{applications:apps,offset,total,qualification:"Retained native inventory from the latest refresh. Search, sorting and paging do not recollect. Running/protected state is checked again on inspection and before each move. Up to three levels of Applications folders; choose another bundle explicitly. Unreadable metadata is omitted, not proven absent.".into()})
}

/// A second installed claimant makes exact bundle-ID paths shared, never exclusive app data.
pub fn qualify_related(mut app: ApplicationFootprint, policy: &Policy) -> ApplicationFootprint {
    let mut shared = false;
    for root in [
        PathBuf::from("/Applications"),
        policy.home.join("Applications"),
        PathBuf::from("/System/Applications"),
    ] {
        let mut entries = walkdir::WalkDir::new(root)
            .max_depth(3)
            .follow_links(false)
            .into_iter();
        while let Some(next) = entries.next() {
            if let Ok(entry) = next {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "app") {
                    entries.skip_current_dir();
                    if p != Path::new(&app.path)
                        && info(p)
                            .ok()
                            .and_then(|v| {
                                v.as_dictionary()
                                    .and_then(|d| d.get("CFBundleIdentifier"))
                                    .and_then(plist::Value::as_string)
                                    .map(str::to_owned)
                            })
                            .is_some_and(|id| id == app.bundle_id)
                    {
                        shared = true;
                        break;
                    }
                }
            }
        }
        if shared {
            break;
        }
    }
    if shared {
        for item in &mut app.related {
            item.eligible = false;
            item.confidence = "Shared bundle identity".into();
            item.consequences="Another installed bundle declares this identity. Associated data may be shared; Coqui does not authorize its removal.".into();
        }
    }
    app
}
