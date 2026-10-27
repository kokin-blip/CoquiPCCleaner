use coqui_engine::{
    app_updates::{self, FetchFailure, FetchResponse},
    database,
    model::*,
    records,
};
use rusqlite::Connection;
use serde_json::json;
use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

struct Fixture {
    _temp: tempfile::TempDir,
    c: Connection,
    root: PathBuf,
    cancel: AtomicBool,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let c = database::open(&root.join("fixture.db")).unwrap();
        Self {
            _temp: temp,
            c,
            root,
            cancel: AtomicBool::new(false),
        }
    }
    fn app(&self, name: &str, version: &str, build: &str, feed: &str, store: bool) -> String {
        let app = self.root.join(format!("{name}.app"));
        fs::create_dir_all(app.join("Contents")).unwrap();
        self.metadata(&app, version, build, feed);
        if store {
            fs::create_dir(app.join("Contents/_MASReceipt")).unwrap();
            fs::write(app.join("Contents/_MASReceipt/receipt"), []).unwrap();
        }
        app.to_string_lossy().into_owned()
    }
    fn metadata(&self, path: &std::path::Path, version: &str, build: &str, feed: &str) {
        let mut info = plist::Dictionary::new();
        for (key, value) in [
            ("CFBundleIdentifier", "com.coqui.disposable-update-fixture"),
            ("CFBundleDisplayName", "Disposable metadata fixture"),
            ("CFBundleShortVersionString", version),
            ("CFBundleVersion", build),
            ("SUFeedURL", feed),
        ] {
            info.insert(key.into(), plist::Value::String(value.into()));
        }
        plist::Value::Dictionary(info)
            .to_file_xml(path.join("Contents/Info.plist"))
            .unwrap();
    }
    fn prepare(&self, paths: Vec<String>) -> AppUpdatePlan {
        app_updates::prepare(&self.c, paths, &self.cancel).unwrap()
    }
    fn check(&self, plan: &AppUpdatePlan, status: u16, body: Vec<u8>) -> AppUpdateReport {
        app_updates::check_with_fetcher(
            &self.c,
            &plan.id,
            true,
            &self.cancel,
            |_| true,
            |_, _| {
                Ok(FetchResponse {
                    status,
                    body: body.clone(),
                })
            },
        )
        .unwrap()
    }
}
fn feed(items: &str) -> Vec<u8> {
    format!("<rss xmlns:s='http://www.andymatuschak.org/xml-namespaces/sparkle'><channel>{items}</channel></rss>").into_bytes()
}
fn release(build: &str, version: &str) -> String {
    format!("<item><enclosure s:version='{build}' s:shortVersionString='{version}' url='https://downloads.vendor.org/ignored-package.zip'/><s:minimumSystemVersion>14.0</s:minimumSystemVersion><pubDate>Sat, 24 Oct 2026 00:00:00 +0000</pubDate></item>")
}
fn apple(bundle: &str, kind: &str, version: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"resultCount":1,"results":[{"bundleId":bundle,"kind":kind,"version":version,"minimumOsVersion":"14.0","currentVersionReleaseDate":"2026-10-24T00:00:00Z","trackViewUrl":"https://ignored.vendor.org/private-download"}]})).unwrap()
}

#[test]
fn prepare_is_local_bounded_and_private_sources_are_not_disclosed() {
    let f = Fixture::new();
    for source in [
        "http://updates.vendor.org/feed.xml",
        "https://user:secret@updates.vendor.org/feed.xml",
        "https://updates.vendor.org/feed.xml?token=secret",
        "https://updates.vendor.org/feed.xml#secret",
        "https://127.0.0.1/feed.xml",
        "https://updates.vendor.org:444/feed.xml",
    ] {
        let path = f.app("Unsupported", "1.0", "1", source, false);
        let plan = f.prepare(vec![path]);
        assert_eq!(plan.candidates[0].source, AppUpdateSource::Unavailable);
        assert_eq!(plan.candidates[0].source_url, None);
        assert!(!serde_json::to_string(&plan).unwrap().contains("secret"));
        fs::remove_dir_all(f.root.join("Unsupported.app")).unwrap();
    }
    let path = f.app(
        "Public",
        "1.0",
        "1",
        "https://updates.vendor.org/appcast.xml",
        false,
    );
    let plan = f.prepare(vec![path.clone()]);
    assert_eq!(plan.candidates[0].source, AppUpdateSource::Sparkle);
    assert_eq!(plan.candidates[0].source_review_date, "2026-10-24");
    assert!(records::list(&f.c, None, 0).unwrap().is_empty());
    assert!(records::list(&f.c, Some("appUpdatePlan"), 0)
        .unwrap()
        .is_empty());
    assert!(app_updates::prepare(&f.c, vec![path.clone(), path.clone()], &f.cancel).is_err());
    assert!(app_updates::prepare(&f.c, vec![path; 11], &f.cancel).is_err());
}

#[test]
fn explicit_ack_seal_expiry_and_single_use_precede_all_network() {
    let f = Fixture::new();
    let path = f.app(
        "Sealed",
        "1.0",
        "1",
        "https://updates.vendor.org/feed.xml",
        false,
    );
    let plan = f.prepare(vec![path.clone()]);
    let mut calls = 0;
    assert!(app_updates::check_with_fetcher(
        &f.c,
        &plan.id,
        false,
        &f.cancel,
        |_| true,
        |_, _| {
            calls += 1;
            Err(FetchFailure::Unavailable)
        }
    )
    .is_err());
    f.metadata(
        &PathBuf::from(&path),
        "2.0",
        "2",
        "https://other.vendor.org/feed.xml",
    );
    assert!(app_updates::check_with_fetcher(
        &f.c,
        &plan.id,
        true,
        &f.cancel,
        |_| true,
        |_, _| {
            calls += 1;
            Err(FetchFailure::Unavailable)
        }
    )
    .is_err());
    assert_eq!(calls, 0);
    let expired = f.prepare(vec![path.clone()]);
    f.c.execute("UPDATE records SET json=json_set(json,'$.plan.expiresAt','2000-01-01T00:00:00Z') WHERE id=?1",[&expired.id]).unwrap();
    assert!(app_updates::check_with_fetcher(
        &f.c,
        &expired.id,
        true,
        &f.cancel,
        |_| true,
        |_, _| {
            calls += 1;
            Err(FetchFailure::Unavailable)
        }
    )
    .is_err());
    assert_eq!(calls, 0);
    let fresh = f.prepare(vec![path]);
    let report = f.check(&fresh, 200, feed(&release("3", "3.0")));
    assert_eq!(report.items[0].state, AppUpdateState::Available);
    assert!(app_updates::check_with_fetcher(
        &f.c,
        &fresh.id,
        true,
        &f.cancel,
        |_| true,
        |_, _| {
            calls += 1;
            Err(FetchFailure::Unavailable)
        }
    )
    .is_err());
    assert_eq!(calls, 0);
}

#[test]
fn sparkle_uses_namespace_builds_and_ignores_delta_and_nondefault_channel_releases() {
    let f = Fixture::new();
    let path = f.app(
        "Sparkle",
        "99.0",
        "4",
        "https://updates.vendor.org/feed.xml",
        false,
    );
    let plan = f.prepare(vec![path]);
    let body=feed(&format!("{}<item><s:channel>beta</s:channel><enclosure url='https://ignored.vendor.org/beta.zip' s:version='900'/></item><item><enclosure url='https://ignored.vendor.org/delta.zip' s:version='901' s:deltaFrom='3'/></item><item><enclosure url='https://ignored.vendor.org/full.zip' s:version='6' s:shortVersionString='1.0'/><s:deltas><enclosure s:version='999' s:deltaFrom='4'/></s:deltas></item>",release("5","1.0")));
    let report = f.check(&plan, 200, body);
    assert_eq!(report.status, "complete");
    assert_eq!(report.items[0].state, AppUpdateState::Available);
    assert_eq!(report.items[0].offered_build.as_deref(), Some("6"));
    assert_eq!(report.items[0].offered_version.as_deref(), Some("1.0"));
    assert!(report.items[0].qualification.contains("unverified"));
    assert!(!serde_json::to_string(&report)
        .unwrap()
        .contains("ignored-package"));
}

#[test]
fn no_newer_is_bounded_and_unknown_numeric_versions_are_uncomparable() {
    for (installed, offered, expected) in [
        ("2.0", "2", AppUpdateState::NoNewerVersionObserved),
        ("2", "1.9", AppUpdateState::NoNewerVersionObserved),
        ("1.2b", "2.0", AppUpdateState::Uncomparable),
        ("1", "2-beta", AppUpdateState::Uncomparable),
        ("1", "1..2", AppUpdateState::Uncomparable),
        (
            "1",
            "999999999999999999999999",
            AppUpdateState::Uncomparable,
        ),
    ] {
        let f = Fixture::new();
        let path = f.app(
            "Version",
            "1.0",
            installed,
            "https://updates.vendor.org/feed.xml",
            false,
        );
        let report = f.check(
            &f.prepare(vec![path]),
            200,
            feed(&release(offered, "Display")),
        );
        assert_eq!(report.items[0].state, expected, "{installed} -> {offered}");
        assert!(report.items[0].qualification.contains("not an up-to-date"));
    }
}

#[test]
fn catalog_requires_exact_bundle_and_desktop_kind_and_empty_means_unavailable() {
    for body in [
        b"{\"resultCount\":0,\"results\":[]}".to_vec(),
        apple("com.other.app", "mac-software", "9"),
        apple("com.coqui.disposable-update-fixture", "software", "9"),
    ] {
        let f = Fixture::new();
        let path = f.app("Store", "1.0", "999", "", true);
        let report = f.check(&f.prepare(vec![path]), 200, body);
        assert_eq!(report.status, "partial");
        assert_eq!(report.items[0].state, AppUpdateState::Unavailable);
    }
    let f = Fixture::new();
    let path = f.app("Store", "1.0", "999", "", true);
    let plan = f.prepare(vec![path]);
    let report=app_updates::check_with_fetcher(&f.c,&plan.id,true,&f.cancel,|_|true,|target,_|{
        assert_eq!(target.host(),"itunes.apple.com");
        assert_eq!(target.url(),"https://itunes.apple.com/lookup?bundleId=com.coqui.disposable-update-fixture&entity=desktopSoftware&country=us");
        Ok(FetchResponse{status:200,body:apple("com.coqui.disposable-update-fixture","mac-software","2.0")})
    }).unwrap();
    assert_eq!(report.items[0].state, AppUpdateState::Available);
    assert_eq!(report.items[0].offered_build, None);
    assert_eq!(report.items[0].min_system_version.as_deref(), Some("14.0"));
}

#[test]
fn malformed_spoofed_dtd_entities_redirects_and_oversized_responses_fail_closed() {
    let valid = feed(&release("2", "2.0"));
    let spoof = String::from_utf8(valid.clone())
        .unwrap()
        .replace(
            "http://www.andymatuschak.org/xml-namespaces/sparkle",
            "https://attacker.example.org/fake",
        )
        .into_bytes();
    let mut entity = b"<!DOCTYPE rss [<!ENTITY version '999'>]>".to_vec();
    entity.extend(feed("<item><s:version>&version;</s:version></item>"));
    for (status, body) in [
        (200, spoof),
        (200, entity),
        (200, feed("<item><s:version>2&amp;0</s:version></item>")),
        (200, b"<rss><channel><item>".to_vec()),
        (200, vec![b'x'; app_updates::MAX_RESPONSE_BYTES + 1]),
        (302, valid.clone()),
        (404, valid),
    ] {
        let f = Fixture::new();
        let path = f.app(
            "Malformed",
            "1.0",
            "1",
            "https://updates.vendor.org/feed.xml",
            false,
        );
        let report = f.check(&f.prepare(vec![path]), status, body);
        assert_eq!(report.items[0].state, AppUpdateState::Unavailable);
        assert_eq!(report.status, "partial");
    }
}

#[test]
fn cancellation_keeps_completed_observations_and_marks_pending_items_incomplete() {
    let f = Fixture::new();
    let a = f.app(
        "First",
        "1.0",
        "1",
        "https://updates.vendor.org/a.xml",
        false,
    );
    let b = f.app(
        "Second",
        "1.0",
        "1",
        "https://updates.vendor.org/b.xml",
        false,
    );
    let plan = f.prepare(vec![a, b]);
    let mut calls = 0;
    let report = app_updates::check_with_fetcher(
        &f.c,
        &plan.id,
        true,
        &f.cancel,
        |report| report.items.iter().all(|i| i.checked_at.is_none()),
        |_, _| {
            calls += 1;
            Ok(FetchResponse {
                status: 200,
                body: feed(&release("2", "2.0")),
            })
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(report.status, "cancelled");
    assert_eq!(report.items[0].state, AppUpdateState::Available);
    assert_eq!(report.items[1].state, AppUpdateState::Cancelled);
    assert_eq!(
        app_updates::latest(&f.c).unwrap().unwrap().status,
        "cancelled"
    );
    assert_eq!(records::list(&f.c, None, 0).unwrap().len(), 1);
}

#[test]
fn each_item_rechecks_local_metadata_before_its_request() {
    let f = Fixture::new();
    let a = f.app(
        "First",
        "1.0",
        "1",
        "https://updates.vendor.org/a.xml",
        false,
    );
    let b = f.app(
        "Second",
        "1.0",
        "1",
        "https://updates.vendor.org/b.xml",
        false,
    );
    let plan = f.prepare(vec![a, b.clone()]);
    let mut calls = 0;
    let report = app_updates::check_with_fetcher(
        &f.c,
        &plan.id,
        true,
        &f.cancel,
        |report| {
            if report.items[0].checked_at.is_some() {
                f.metadata(
                    &PathBuf::from(&b),
                    "9.0",
                    "9",
                    "https://changed.vendor.org/feed.xml",
                );
            }
            true
        },
        |_, _| {
            calls += 1;
            Ok(FetchResponse {
                status: 200,
                body: feed(&release("2", "2.0")),
            })
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(report.status, "partial");
    assert!(report.items[1].qualification.contains("changed before"));
}

#[test]
fn report_restart_recovery_and_handoff_revalidate_the_exact_stored_app() {
    let f = Fixture::new();
    let path = f.app(
        "Handoff",
        "1.0",
        "1",
        "https://updates.vendor.org/feed.xml",
        false,
    );
    let report = f.check(
        &f.prepare(vec![path.clone()]),
        200,
        feed(&release("2", "2.0")),
    );
    assert!(matches!(
        app_updates::handoff(&f.c, &report.id, &path).unwrap(),
        app_updates::Handoff::VendorApplication(_)
    ));
    assert!(app_updates::handoff(&f.c, &report.id, "/unselected.app").is_err());
    f.metadata(
        &PathBuf::from(&path),
        "2.0",
        "2",
        "https://updates.vendor.org/feed.xml",
    );
    assert!(app_updates::handoff(&f.c, &report.id, &path).is_err());
    let mut interrupted = report;
    interrupted.status = "running".into();
    interrupted.finished_at = None;
    records::save(
        &f.c,
        app_updates::REPORT_KIND,
        &interrupted.id,
        &interrupted.created_at,
        "running",
        "Fixture interruption",
        &interrupted,
    )
    .unwrap();
    database::recover_interrupted(&f.c).unwrap();
    let recovered = app_updates::latest(&f.c).unwrap().unwrap();
    assert_eq!(recovered.status, "interrupted");
    assert_eq!(recovered.finished_at, None);
    assert!(app_updates::handoff(&f.c, &recovered.id, &path).is_err());
}

#[test]
fn private_reserved_and_transition_addresses_are_rejected_before_http() {
    for address in [
        "0.0.0.0",
        "10.1.2.3",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.169.254",
        "172.16.0.1",
        "192.0.0.1",
        "192.168.1.1",
        "192.0.2.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "255.255.255.255",
        "::1",
        "::ffff:8.8.8.8",
        "fc00::1",
        "fe80::1",
        "2001:db8::1",
        "2002:0808:0808::1",
        "3fff::1",
    ] {
        assert!(
            !app_updates::public_address(address.parse().unwrap()),
            "{address}"
        );
    }
    for address in ["8.8.8.8", "17.253.144.10", "2606:4700:4700::1111"] {
        assert!(
            app_updates::public_address(address.parse().unwrap()),
            "{address}"
        );
    }
    for raw in [
        "https://localhost/feed",
        "https://host.local/feed",
        "https://user:secret@updates.vendor.org/feed",
        "https://[::1]/feed",
        "https://updates.vendor.org:444/feed",
        "https://updates.vendor.org/feed?token=x",
        "https://updates.vendor.org/feed#x",
        "https://updates.vendor.org\\@127.0.0.1/feed",
    ] {
        assert!(app_updates::validate_feed_url(raw).is_err(), "{raw}");
    }
}

#[test]
fn nested_metadata_cannot_shadow_direct_item_fields_and_delta_only_is_not_full_release() {
    let f = Fixture::new();
    let path = f.app(
        "Nested",
        "2.0",
        "2",
        "https://updates.vendor.org/feed.xml",
        false,
    );
    let body=feed("<item><enclosure url='https://ignored.vendor.org/full.zip' s:version='1' s:shortVersionString='1.0'/><unrelated><s:version>999</s:version><s:channel>beta</s:channel><s:minimumSystemVersion>999</s:minimumSystemVersion></unrelated></item>");
    let report = f.check(&f.prepare(vec![path.clone()]), 200, body);
    assert_eq!(
        report.items[0].state,
        AppUpdateState::NoNewerVersionObserved
    );
    assert_eq!(report.items[0].offered_build.as_deref(), Some("1"));
    assert_eq!(report.items[0].min_system_version, None);
    let body=feed(&format!("{}<item><enclosure url='https://ignored.vendor.org/beta.zip' s:version='999'/><s:channel><![CDATA[beta]]></s:channel></item>",release("1","1.0")));
    let report = f.check(&f.prepare(vec![path.clone()]), 200, body);
    assert_eq!(report.items[0].state, AppUpdateState::Unavailable);
    for enclosure in [
        "<enclosure url='https://ignored.vendor.org/delta.zip' s:deltaFrom='1'/>",
        "<enclosure url='https://ignored.vendor.org/windows.zip' s:os='windows'/>",
    ] {
        let body = feed(&format!(
            "<item><s:version>999</s:version>{enclosure}</item>"
        ));
        let report = f.check(&f.prepare(vec![path.clone()]), 200, body);
        assert_eq!(report.items[0].state, AppUpdateState::Unavailable);
    }
}

#[test]
fn unsupported_sources_and_failed_or_cancelled_transport_do_not_invent_check_times() {
    let f = Fixture::new();
    let path = f.app("Unknown", "1", "1", "", false);
    let report = app_updates::check_with_fetcher(
        &f.c,
        &f.prepare(vec![path]).id,
        true,
        &f.cancel,
        |_| true,
        |_, _| panic!("unsupported source must not request"),
    )
    .unwrap();
    assert_eq!(report.items[0].checked_at, None);
    let path = f.app(
        "Blocked",
        "1",
        "1",
        "https://updates.vendor.org/feed.xml",
        false,
    );
    let report = app_updates::check_with_fetcher(
        &f.c,
        &f.prepare(vec![path.clone()]).id,
        true,
        &f.cancel,
        |_| true,
        |_, _| Err(FetchFailure::Unavailable),
    )
    .unwrap();
    assert_eq!(report.items[0].checked_at, None);
    let report = app_updates::check_with_fetcher(
        &f.c,
        &f.prepare(vec![path]).id,
        true,
        &f.cancel,
        |_| true,
        |_, _| Err(FetchFailure::Cancelled),
    )
    .unwrap();
    assert_eq!(report.status, "cancelled");
    assert_eq!(report.items[0].state, AppUpdateState::Cancelled);
    assert_eq!(report.items[0].checked_at, None);
}
