use coqui_engine::{cleanup::Policy, database, duplicates, model::*, records, storage};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

struct Fixture {
    _temp: tempfile::TempDir,
    home: PathBuf,
    c: rusqlite::Connection,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().canonicalize().unwrap().join("home_%");
        fs::create_dir_all(home.join("Downloads")).unwrap();
        let c = database::open(&temp.path().join("fixture.sqlite3")).unwrap();
        Self {
            _temp: temp,
            home,
            c,
        }
    }
    fn file(&self, path: &str, bytes: &[u8]) -> PathBuf {
        let path = self.home.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
    fn index(&mut self) {
        let mut session: DiagnosticSession = serde_json::from_value(serde_json::json!({
            "id":"fixture", "startedAt":now(), "finishedAt":now(), "status":"complete", "stage":"Fixture complete", "scope":self.home,
            "fileCount":0, "logicalBytes":0, "allocatedBytes":0, "skippedCount":0, "snapshot":null,
            "findings":[], "permissions":[], "categories":[], "candidates":[], "errors":[]
        })).unwrap();
        database::save_session(&self.c, &session).unwrap();
        let mut rows = vec![];
        let summary = storage::walk(
            &[self.home.clone()],
            &[],
            &AtomicBool::new(false),
            |row| {
                rows.push(row);
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
        database::insert_files(&mut self.c, &session.id, &rows).unwrap();
        session.file_count = summary.files;
        session.logical_bytes = summary.logical;
        session.allocated_bytes = summary.allocated;
        database::save_session(&self.c, &session).unwrap();
    }
    fn policy(&self) -> Policy {
        Policy {
            home: self.home.clone(),
            exclusions: vec![],
        }
    }
    fn analyze(&self, policy: &Policy, cancel: bool) -> DuplicateReport {
        duplicates::analyze_scoped(
            &self.c,
            DuplicateAnalysisRequest {
                session_id: "fixture".into(),
                location: DuplicateLocation::Downloads,
                similar_photos: false,
            },
            policy,
            &AtomicBool::new(cancel),
            |_| true,
        )
        .unwrap()
    }
}

#[test]
fn downloads_prefix_is_applied_before_largest_candidate_limit_and_does_not_read_siblings() {
    let mut f = Fixture::new();
    let a = f.file("Downloads/first.txt", b"equal bytes");
    let b = f.file("Downloads/subfolder/second.txt", b"equal bytes");
    f.file("Downloads-old/not-in-scope.txt", b"equal bytes");
    for i in 0..2001 {
        f.file(&format!("Documents/larger-{i}"), &[b'x'; 32]);
    }
    f.index();
    let report = f.analyze(&f.policy(), false);
    assert_eq!(report.status, "complete");
    assert_eq!(report.total, 2);
    assert_eq!(report.examined, 2);
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert!(report.groups[0]
        .files
        .iter()
        .all(|file| Path::new(&file.path).starts_with(f.home.join("Downloads"))));
    assert!(report.photo_matches.is_empty());
    assert_eq!(fs::read(&a).unwrap(), b"equal bytes");
    assert_eq!(fs::read(&b).unwrap(), b"equal bytes");
    let retained = duplicates::get(&f.c, &report.id).unwrap();
    assert_eq!(
        retained.scope_root.as_deref(),
        f.home.join("Downloads").to_str()
    );
    assert!(!retained.request.unwrap().similar_photos);
    let writes = f.c.total_changes();
    f.c.execute_batch("PRAGMA query_only=ON").unwrap();
    assert_eq!(duplicates::get(&f.c, &report.id).unwrap().total, 2);
    assert_eq!(f.c.total_changes(), writes);
}

#[test]
fn empty_unindexed_partial_and_cancelled_downloads_remain_qualified() {
    let mut f = Fixture::new();
    f.index();
    let empty = f.analyze(&f.policy(), false);
    assert_eq!(empty.status, "complete");
    assert_eq!(empty.total, 0);
    f.c.execute("DELETE FROM files WHERE session_id='fixture'", [])
        .unwrap();
    let absent = f.analyze(&f.policy(), false);
    assert_eq!(absent.status, "partial");
    assert!(!absent.gaps.is_empty());
    f.file("Downloads/a", b"same");
    f.file("Downloads/b", b"same");
    f.index();
    let mut session = database::session(&f.c, "fixture").unwrap();
    session.status = "partial".into();
    database::save_session(&f.c, &session).unwrap();
    let partial = f.analyze(&f.policy(), false);
    assert_eq!(partial.status, "partial");
    assert_eq!(partial.groups.len(), 1);
    let cancelled = f.analyze(&f.policy(), true);
    assert_eq!(cancelled.status, "cancelled");
    assert_eq!(cancelled.hashed_bytes, 0.);
    assert!(cancelled.groups.is_empty());
}

#[test]
fn exclusions_links_and_invalid_requests_cannot_expand_downloads_scope() {
    let mut f = Fixture::new();
    let a = f.file("Downloads/a", b"same");
    f.file("Downloads/b", b"same");
    f.index();
    let mut policy = f.policy();
    policy.exclusions.push(a);
    let report = f.analyze(&policy, false);
    assert_eq!(report.status, "partial");
    assert!(report.groups.is_empty());
    policy.exclusions = vec![f.home.join("Downloads")];
    let request = || DuplicateAnalysisRequest {
        session_id: "fixture".into(),
        location: DuplicateLocation::Downloads,
        similar_photos: false,
    };
    assert!(
        duplicates::analyze_scoped(&f.c, request(), &policy, &AtomicBool::new(false), |_| true)
            .is_err()
    );
    let originals = f.home.join("Original downloads");
    fs::rename(f.home.join("Downloads"), &originals).unwrap();
    symlink(&originals, f.home.join("Downloads")).unwrap();
    assert!(duplicates::analyze_scoped(
        &f.c,
        request(),
        &f.policy(),
        &AtomicBool::new(false),
        |_| true
    )
    .is_err());
    assert!(serde_json::from_value::<DuplicateAnalysisRequest>(
        serde_json::json!({"sessionId":"fixture","location":"arbitrary","similarPhotos":false})
    )
    .is_err());
    assert!(serde_json::from_value::<DuplicateAnalysisRequest>(serde_json::json!({"sessionId":"fixture","location":"downloads","similarPhotos":false,"root":"/"})).is_err());
}

#[test]
fn legacy_duplicate_records_leave_scope_unrecorded_and_disconnect_stops_reads() {
    let mut f = Fixture::new();
    f.file("Downloads/a", b"same");
    f.file("Downloads/b", b"same");
    f.index();
    let report = duplicates::analyze_scoped(
        &f.c,
        DuplicateAnalysisRequest {
            session_id: "fixture".into(),
            location: DuplicateLocation::Downloads,
            similar_photos: false,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| false,
    )
    .unwrap();
    assert_eq!(report.status, "cancelled");
    assert_eq!(report.examined, 0);
    assert_eq!(report.hashed_bytes, 0.);
    let mut value = serde_json::to_value(&report).unwrap();
    value.as_object_mut().unwrap().remove("request");
    value.as_object_mut().unwrap().remove("scopeRoot");
    let legacy: DuplicateReport = serde_json::from_value(value).unwrap();
    records::save(
        &f.c,
        "duplicates",
        "legacy",
        &now(),
        "cancelled",
        "Fixture legacy",
        &legacy,
    )
    .unwrap();
    let restored = duplicates::get(&f.c, "legacy").unwrap();
    assert!(restored.request.is_none() && restored.scope_root.is_none());
}

#[test]
fn photo_decoding_requires_the_explicit_option_and_candidate_bound_is_partial() {
    let mut f = Fixture::new();
    f.file("Downloads/first.png", b"invalid image");
    f.file("Downloads/second.jpg", b"also an invalid image");
    f.index();
    let exact_only = f.analyze(&f.policy(), false);
    assert_eq!(exact_only.status, "complete");
    assert_eq!(exact_only.hashed_bytes, 0.);
    let photos = duplicates::analyze_scoped(
        &f.c,
        DuplicateAnalysisRequest {
            session_id: "fixture".into(),
            location: DuplicateLocation::Downloads,
            similar_photos: true,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert_eq!(photos.status, "partial");
    assert!(!photos.gaps.is_empty());
    let mut entry: FileEntry = serde_json::from_value(serde_json::json!({"path":"", "parent":"", "name":"", "category":"Other", "logicalBytes":100, "allocatedBytes":0, "isDirectory":false, "modifiedAt":"", "qualification":"Fixture"})).unwrap();
    let entries = (0..2001)
        .map(|i| {
            entry.path = f
                .home
                .join("Downloads")
                .join(format!("unavailable-{i}"))
                .to_string_lossy()
                .into_owned();
            entry.clone()
        })
        .collect::<Vec<_>>();
    database::insert_files(&mut f.c, "fixture", &entries).unwrap();
    let bounded = f.analyze(&f.policy(), false);
    assert_eq!(bounded.total, 2000);
    assert_eq!(bounded.status, "partial");
    assert!(bounded.gaps.iter().any(|gap| gap.contains("2,000")));
}

#[test]
#[ignore = "Explicit disposable Downloads report capture; no user files or user history are read"]
fn native_disposable_downloads_report_preserves_contents_and_exports_retained_evidence() {
    let mut f = Fixture::new();
    let keeper = f.file(
        "Downloads/Project notes.txt",
        b"Coqui disposable exact-copy fixture",
    );
    let copy = f.file(
        "Downloads/Older copy/Project notes.txt",
        b"Coqui disposable exact-copy fixture",
    );
    fs::File::open(&copy)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(
            std::time::SystemTime::now() - std::time::Duration::from_secs(100 * 86_400),
        ))
        .unwrap();
    f.file("Downloads/Unique.txt", b"Different disposable content");
    f.index();
    let report = f.analyze(&f.policy(), false);
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.total, 3);
    assert_eq!(fs::read(keeper).unwrap(), fs::read(copy).unwrap());
    fs::write(
        "/private/tmp/coqui-downloads-native-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    fs::write(
        "/private/tmp/coqui-downloads-native-session.json",
        serde_json::to_vec_pretty(&database::session(&f.c, "fixture").unwrap()).unwrap(),
    )
    .unwrap();
}
