use coqui_engine::{
    cleanup::{self, Policy},
    cleanup_categories::{self, CleanupCategoryKind::*, CleanupCategoryMode, CleanupCategoryQuery},
    database,
    model::*,
    storage,
};
use std::{
    fs,
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
        fs::create_dir(&home).unwrap();
        let c = database::open(&temp.path().join("fixture.sqlite3")).unwrap();
        let session:DiagnosticSession=serde_json::from_value(serde_json::json!({"id":"fixture","startedAt":"2026-10-27T12:00:00Z","finishedAt":"2026-10-27T12:01:00Z","status":"complete","stage":"Complete","scope":home,"fileCount":0,"logicalBytes":0,"allocatedBytes":0,"skippedCount":0,"snapshot":null,"findings":[],"permissions":[],"categories":[],"candidates":[],"errors":[]})).unwrap();
        database::save_session(&c, &session).unwrap();
        Self {
            _temp: temp,
            home,
            c,
        }
    }
    fn policy(&self) -> Policy {
        Policy {
            home: self.home.clone(),
            exclusions: vec![],
        }
    }
    fn row(&self, path: &str, logical: f64, allocated: f64, directory: bool) -> FileEntry {
        let path = self.home.join(path);
        FileEntry {
            parent: path.parent().unwrap().to_string_lossy().into_owned(),
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            path: path.to_string_lossy().into_owned(),
            category: "Other".into(),
            logical_bytes: logical,
            allocated_bytes: allocated,
            is_directory: directory,
            modified_at: "2026-10-27T12:00:00Z".into(),
            qualification: "Illustrative index fixture".into(),
        }
    }
    fn insert(&mut self, rows: &[FileEntry]) {
        database::insert_files(&mut self.c, "fixture", rows).unwrap();
    }
}
fn group(
    report: &cleanup_categories::CleanupCategoryReport,
    kind: cleanup_categories::CleanupCategoryKind,
) -> &cleanup_categories::CleanupCategory {
    report
        .categories
        .iter()
        .find(|row| row.kind == kind)
        .unwrap()
}
fn page(
    f: &Fixture,
    kind: cleanup_categories::CleanupCategoryKind,
    offset: u32,
) -> cleanup_categories::CleanupCategoryPage {
    cleanup_categories::entries(
        &f.c,
        CleanupCategoryQuery {
            session_id: "fixture".into(),
            kind,
            offset,
        },
        &f.policy(),
    )
    .unwrap()
}

#[test]
fn file_only_disjoint_categories_retain_unknown_dates_and_no_cleanup_authority() {
    let mut f = Fixture::new();
    f.insert(&[
        f.row("Library/Caches", 999999., 999999., true),
        f.row("Library/Caches/app/a", 7., 4096., false),
        f.row("Library/Caches/com.apple.dt.Xcode/a", 9., 8192., false),
        f.row("Library/Caches-old/outside", 99., 99999., false),
        f.row("Library/Logs", 0., 0., true),
        f.row(
            "Library/Developer/Xcode/Archives/archive/symbols",
            11.,
            12288.,
            false,
        ),
    ]);
    let report = cleanup_categories::report(&f.c, "fixture", &f.policy()).unwrap();
    assert_eq!(report.categories.len(), 12);
    assert_eq!(group(&report, UserCaches).file_count, Some(1));
    assert_eq!(group(&report, UserCaches).allocated_bytes, Some(4096.));
    assert_eq!(group(&report, XcodeCaches).file_count, Some(1));
    assert_eq!(group(&report, XcodeCaches).allocated_bytes, Some(8192.));
    assert_eq!(group(&report, UserLogs).file_count, Some(0));
    assert_eq!(group(&report, UserLogs).state, "indexed");
    assert_eq!(group(&report, DeviceBackups).file_count, None);
    assert_eq!(group(&report, DeviceBackups).state, "notIndexed");
    assert_eq!(
        group(&report, XcodeArchives).mode,
        CleanupCategoryMode::ManualReview
    );
    assert_eq!(
        group(&report, SystemLogs).mode,
        CleanupCategoryMode::Protected
    );
    assert_eq!(report.observed_at.as_deref(), Some("2026-10-27T12:01:00Z"));
    let mut session = database::session(&f.c, "fixture").unwrap();
    session.status = "partial".into();
    session.finished_at = None;
    database::save_session(&f.c, &session).unwrap();
    let partial = cleanup_categories::report(&f.c, "fixture", &f.policy()).unwrap();
    assert_eq!(group(&partial, UserCaches).state, "partial");
    assert!(partial.observed_at.is_none());
}

#[test]
fn current_exclusions_and_literal_boundaries_apply_to_counts_and_pages() {
    let mut f = Fixture::new();
    f.insert(&[
        f.row("Library/Caches", 0., 0., true),
        f.row("Library/Caches/app/a", 1., 4096., false),
        f.row("Library/Caches/app/b", 2., 8192., false),
        f.row("Library/Developer/Xcode/iOS DeviceSupport", 0., 0., true),
    ]);
    let mut policy = f.policy();
    policy.exclusions.push(f.home.join("Library/Caches/app/a"));
    let report = cleanup_categories::report(&f.c, "fixture", &policy).unwrap();
    assert_eq!(group(&report, UserCaches).file_count, Some(1));
    let rows = cleanup_categories::entries(
        &f.c,
        CleanupCategoryQuery {
            session_id: "fixture".into(),
            kind: UserCaches,
            offset: 0,
        },
        &policy,
    )
    .unwrap();
    assert_eq!(rows.files.len(), 1);
    assert!(rows.files[0].path.ends_with("/b"));
    policy.exclusions = vec![
        f.home.join("Library/Caches"),
        f.home.join("Library/Developer/Xcode/iOS DeviceSupport"),
    ];
    let report = cleanup_categories::report(&f.c, "fixture", &policy).unwrap();
    assert_eq!(group(&report, UserCaches).state, "excluded");
    assert_eq!(group(&report, UserCaches).file_count, None);
    assert_eq!(group(&report, XcodeCaches).state, "excluded");
    assert_eq!(group(&report, XcodeDeviceSupport).file_count, None);
}

#[test]
fn pages_are_bounded_pure_and_cannot_turn_manual_paths_into_cache_approval() {
    let mut f = Fixture::new();
    let rows = (0..101)
        .map(|n| f.row(&format!("Library/Caches/app/{n:03}"), 1., 4096., false))
        .collect::<Vec<_>>();
    f.insert(&rows);
    let writes = f.c.total_changes();
    f.c.execute_batch("PRAGMA query_only=ON").unwrap();
    let first = page(&f, UserCaches, 0);
    let second = page(&f, UserCaches, 100);
    assert_eq!(first.files.len(), 100);
    assert_eq!(first.next_offset, Some(100));
    assert_eq!(second.files.len(), 1);
    assert!(second.next_offset.is_none());
    assert_ne!(first.files[0].path, second.files[0].path);
    cleanup_categories::report(&f.c, "fixture", &f.policy()).unwrap();
    assert_eq!(f.c.total_changes(), writes);
    f.c.execute_batch("PRAGMA query_only=OFF").unwrap();
    let logs = f.home.join("Library/Logs/needed.log");
    fs::create_dir_all(logs.parent().unwrap()).unwrap();
    fs::write(&logs, b"fixture diagnostic").unwrap();
    f.insert(&[f.row("Library/Logs/needed.log", 18., 4096., false)]);
    assert!(cleanup::preview(
        &f.c,
        CleanupRequest {
            session_id: "fixture".into(),
            paths: vec![logs.to_string_lossy().into_owned()]
        },
        &f.policy()
    )
    .is_err());
    assert_eq!(fs::read(logs).unwrap(), b"fixture diagnostic");
    assert!(serde_json::from_value::<CleanupCategoryQuery>(
        serde_json::json!({"sessionId":"fixture","kind":"arbitrary","offset":0})
    )
    .is_err());
    assert!(serde_json::from_value::<CleanupCategoryQuery>(
        serde_json::json!({"sessionId":"fixture","kind":"userCaches","offset":0,"root":"/"})
    )
    .is_err());
}

#[test]
fn known_data_aliases_and_large_unrelated_index_do_not_change_category_scope() {
    let mut f = Fixture::new();
    let mut rows = (0..30_000)
        .map(|n| f.row(&format!("Documents/outside-{n}"), 100., 4096., false))
        .collect::<Vec<_>>();
    rows.push(f.row("Library/Caches/app/one", 1., 4096., false));
    f.insert(&rows);
    // A VM instruction budget detects a full-session sort/walk without relying on timing.
    unsafe extern "C" fn tick(context: *mut libc::c_void) -> libc::c_int {
        let remaining = unsafe { &mut *context.cast::<u32>() };
        *remaining = remaining.saturating_sub(1);
        i32::from(*remaining == 0)
    }
    let mut ticks = Box::new(100u32);
    unsafe {
        rusqlite::ffi::sqlite3_progress_handler(
            f.c.handle(),
            1000,
            Some(tick),
            (&mut *ticks as *mut u32).cast(),
        );
    }
    assert_eq!(page(&f, UserCaches, 0).files.len(), 1);
    assert_eq!(
        group(
            &cleanup_categories::report(&f.c, "fixture", &f.policy()).unwrap(),
            UserCaches
        )
        .file_count,
        Some(1)
    );
    unsafe {
        rusqlite::ffi::sqlite3_progress_handler(f.c.handle(), 0, None, std::ptr::null_mut());
    }
    let home = PathBuf::from("/Users/coqui-category-fixture");
    if coqui_engine::volumes::friendly(Path::new(
        "/System/Volumes/Data/Users/coqui-category-fixture",
    )) == home
    {
        let policy = Policy {
            home,
            exclusions: vec![],
        };
        let mut alias = f.row("unused", 5., 8192., false);
        alias.path =
            "/System/Volumes/Data/Users/coqui-category-fixture/Library/Caches/app/a".into();
        f.insert(&[alias]);
        assert_eq!(
            group(
                &cleanup_categories::report(&f.c, "fixture", &policy).unwrap(),
                UserCaches
            )
            .file_count,
            Some(1)
        );
    }
}

#[test]
#[ignore = "Explicit disposable category evidence export; no user history or existing files are read"]
fn native_disposable_categories_preserve_files_and_export_dated_report() {
    let mut f = Fixture::new();
    for path in [
        "Library/Caches/app/cache.bin",
        "Library/Caches/com.apple.dt.Xcode/cache.bin",
        "Library/Logs/needed.log",
        "Library/Developer/Xcode/DerivedData/project/Build/output",
        "Library/Developer/Xcode/Archives/release/symbols",
        "Library/Application Support/MobileSync/Backup/backup/file",
    ] {
        let path = f.home.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"Disposable category fixture").unwrap();
    }
    let mut rows = vec![];
    let summary = storage::walk(
        &[f.home.clone()],
        &[],
        &AtomicBool::new(false),
        |row| {
            rows.push(row);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    f.insert(&rows);
    let mut session = database::session(&f.c, "fixture").unwrap();
    session.started_at = now();
    session.finished_at = Some(now());
    session.file_count = summary.files;
    session.logical_bytes = summary.logical;
    session.allocated_bytes = summary.allocated;
    database::save_session(&f.c, &session).unwrap();
    let report = cleanup_categories::report(&f.c, "fixture", &f.policy()).unwrap();
    let mut pages = serde_json::Map::new();
    for category in &report.categories {
        pages.insert(
            serde_json::to_value(category.kind)
                .unwrap()
                .as_str()
                .unwrap()
                .into(),
            serde_json::to_value(page(&f, category.kind, 0)).unwrap(),
        );
    }
    fs::write(
        "/private/tmp/coqui-categories-native-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    fs::write(
        "/private/tmp/coqui-categories-native-pages.json",
        serde_json::to_vec_pretty(&pages).unwrap(),
    )
    .unwrap();
    fs::write(
        "/private/tmp/coqui-categories-native-session.json",
        serde_json::to_vec_pretty(&session).unwrap(),
    )
    .unwrap();
    assert_eq!(
        fs::read(f.home.join("Library/Logs/needed.log")).unwrap(),
        b"Disposable category fixture"
    );
}
