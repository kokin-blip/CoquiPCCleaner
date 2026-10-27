use coqui_engine::{database, model::*, scan_review};
use rusqlite::{params, Connection};
use serde_json::json;

fn fixture(status: &str, file_count: u32) -> (tempfile::TempDir, Connection, DiagnosticSession) {
    let temp = tempfile::tempdir().unwrap();
    let c = database::open(&temp.path().join("review.db")).unwrap();
    let s: DiagnosticSession = serde_json::from_value(json!({
        "id":"scan", "startedAt":"2026-10-23T01:00:00+00:00",
        "finishedAt":"2026-10-23T02:00:00+00:00", "status":status, "stage":"Fixture",
        "scope":"/fixture", "fileCount":file_count, "logicalBytes":0, "allocatedBytes":0,
        "skippedCount":0, "snapshot":null, "findings":[], "permissions":[],
        "categories":[], "candidates":[], "errors":[]
    }))
    .unwrap();
    database::save_session(&c, &s).unwrap();
    (temp, c, s)
}

fn group(review: &ScanReview, kind: ScanReviewKind) -> &ScanReviewGroup {
    review.groups.iter().find(|item| item.kind == kind).unwrap()
}

fn file(c: &Connection, session: &str, name: &str, logical: f64, allocated: f64, directory: bool) {
    c.execute(
        "INSERT INTO files VALUES(?1,?2,'/fixture',?3,'Caches',?4,?5,?6,'2026-10-23T01:00:00+00:00','Fixture')",
        params![session, format!("/fixture/{name}"), name, logical, allocated, directory],
    ).unwrap();
}

fn snapshot(s: &mut DiagnosticSession) {
    s.snapshot = Some(
        serde_json::from_value(json!({
            "observedAt":"2026-10-22T09:00:00+00:00", "deviceName":"Fixture",
            "model":"Fixture", "osVersion":"Fixture", "architecture":"arm64", "cpu":"Fixture",
            "coreCount":1, "memoryBytes":1, "gpu":"Unavailable", "disks":[], "evidence":[],
            "capabilities":[], "startupItems":[{"name":"Fixture", "path":"/fixture/agent.plist",
            "mechanism":"LaunchAgent", "state":"configured", "explanation":"Not runtime state"}],
            "applications":[{"name":"Fixture", "path":"/fixture/app.app"}]
        }))
        .unwrap(),
    );
}

#[test]
fn categories_reuse_file_only_aggregates_and_do_not_sum_cumulative_folders() {
    let (_temp, c, mut s) = fixture("partial", 2);
    s.categories = vec![
        StorageCategory {
            name: "Caches".into(),
            logical_bytes: 12.,
            allocated_bytes: 8192.,
            file_count: 2,
        },
        StorageCategory {
            name: "Development".into(),
            logical_bytes: 5.,
            allocated_bytes: 4096.,
            file_count: 1,
        },
    ];
    database::save_session(&c, &s).unwrap();
    file(&c, "scan", "huge-parent", 9e12, 9e12, true);
    let result = scan_review::build(&c, "scan").unwrap();
    let caches = group(&result, ScanReviewKind::Caches);
    assert_eq!(caches.state, ScanReviewState::Partial);
    assert_eq!(caches.count, Some(2));
    assert_eq!(caches.logical_bytes, Some(12.));
    assert_eq!(caches.allocated_bytes, Some(8192.));
    assert_eq!(caches.source, ScanReviewSource::RetainedScanCategories);
    assert!(caches.qualification.contains("not an eligible-removal"));
    let development = group(&result, ScanReviewKind::Development);
    assert_eq!(development.count, Some(1));
    assert!(development.qualification.contains("not only Xcode"));
}

#[test]
fn large_files_use_inclusive_allocated_threshold_and_exclude_folders_and_other_scans() {
    let (_temp, c, mut s) = fixture("complete", 3);
    let threshold = scan_review::LARGE_FILE_THRESHOLD_BYTES;
    file(&c, "scan", "threshold", 2. * threshold, threshold, false);
    file(&c, "scan", "larger", 3. * threshold, 2. * threshold, false);
    file(&c, "scan", "sparse", 9. * threshold, 4096., false);
    file(&c, "scan", "folder", 10. * threshold, 10. * threshold, true);
    s.id = "other".into();
    database::save_session(&c, &s).unwrap();
    file(
        &c,
        "other",
        "unrelated",
        99. * threshold,
        99. * threshold,
        false,
    );
    let result = scan_review::build(&c, "scan").unwrap();
    let large = group(&result, ScanReviewKind::LargeFiles);
    assert_eq!(large.state, ScanReviewState::Ready);
    assert_eq!(large.count, Some(2));
    assert_eq!(large.logical_bytes, Some(5. * threshold));
    assert_eq!(large.allocated_bytes, Some(3. * threshold));
    assert_eq!(large.threshold_bytes, Some(threshold));
}

#[test]
fn unknown_measurements_stay_null_and_completed_empty_measurements_are_zero() {
    let (_temp, c, mut s) = fixture("partial", 12);
    let result = scan_review::build(&c, "scan").unwrap();
    for kind in [
        ScanReviewKind::Caches,
        ScanReviewKind::Development,
        ScanReviewKind::Startup,
        ScanReviewKind::Applications,
    ] {
        let item = group(&result, kind);
        assert_eq!(item.state, ScanReviewState::NotChecked);
        assert_eq!(item.count, None);
        assert_eq!(item.allocated_bytes, None);
    }
    assert_eq!(
        group(&result, ScanReviewKind::LargeFiles).state,
        ScanReviewState::Unavailable
    );
    s.status = "complete".into();
    s.file_count = 0;
    database::save_session(&c, &s).unwrap();
    let empty = scan_review::build(&c, "scan").unwrap();
    for kind in [
        ScanReviewKind::Caches,
        ScanReviewKind::Development,
        ScanReviewKind::LargeFiles,
    ] {
        let item = group(&empty, kind);
        assert_eq!(item.state, ScanReviewState::Ready);
        assert_eq!(item.count, Some(0));
        assert_eq!(item.allocated_bytes, Some(0.));
    }
    s.status = "cancelled".into();
    database::save_session(&c, &s).unwrap();
    assert_eq!(
        group(
            &scan_review::build(&c, "scan").unwrap(),
            ScanReviewKind::Caches
        )
        .count,
        None
    );
}

#[test]
fn retained_inventory_uses_its_own_latest_date_and_never_refreshes_missing_paths() {
    let (_temp, c, mut s) = fixture("complete", 0);
    snapshot(&mut s);
    database::save_session(&c, &s).unwrap();
    for (path, observed, missing) in [
        (
            "/definitely-missing/a.app",
            "2026-10-21T11:00:00+02:00",
            None,
        ),
        (
            "/definitely-missing/b.app",
            "2026-10-21T10:00:00+00:00",
            None,
        ),
        ("/old.app", "2026-10-24T10:00:00+00:00", Some("2026-10-24")),
    ] {
        c.execute(
            "INSERT INTO application_observations VALUES(?1,?2,?3)",
            params![path, json!({"observedAt":observed}).to_string(), missing],
        )
        .unwrap();
    }
    let result = scan_review::build(&c, "scan").unwrap();
    let apps = group(&result, ScanReviewKind::Applications);
    assert_eq!(apps.source, ScanReviewSource::RetainedApplicationInventory);
    assert_eq!(apps.count, Some(2));
    assert_eq!(
        apps.observed_at.as_deref(),
        Some("2026-10-21T10:00:00+00:00")
    );
    assert_eq!(apps.state, ScanReviewState::Partial);
    assert!(apps.qualification.contains("independent of this scan"));
    let remaining: u32 = c
        .query_row(
            "SELECT count(*) FROM application_observations WHERE missing_at IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 2);
}

#[test]
fn undated_inventory_falls_back_to_explicit_snapshot_and_startup_stays_limited() {
    let (_temp, c, mut s) = fixture("complete", 0);
    snapshot(&mut s);
    database::save_session(&c, &s).unwrap();
    c.execute(
        "INSERT INTO application_observations VALUES('/legacy.app','{}',NULL)",
        [],
    )
    .unwrap();
    let result = scan_review::build(&c, "scan").unwrap();
    for kind in [ScanReviewKind::Applications, ScanReviewKind::Startup] {
        let item = group(&result, kind);
        assert_eq!(item.source, ScanReviewSource::RetainedSnapshot);
        assert_eq!(
            item.observed_at.as_deref(),
            Some("2026-10-22T09:00:00+00:00")
        );
        assert_eq!(item.count, Some(1));
        assert_eq!(item.state, ScanReviewState::Partial);
        assert_eq!(item.allocated_bytes, None);
    }
    assert!(group(&result, ScanReviewKind::Startup)
        .qualification
        .contains("not complete modern"));
    assert!(group(&result, ScanReviewKind::Applications)
        .qualification
        .contains("scan-snapshot"));
}

#[test]
fn summary_runs_on_read_only_current_schema_and_does_not_write_or_create_records() {
    let (temp, c, _s) = fixture("complete", 0);
    let before = c.total_changes();
    c.execute_batch("PRAGMA query_only=ON").unwrap();
    let result = scan_review::build(&c, "scan").unwrap();
    assert_eq!(
        result.observed_at.as_deref(),
        Some("2026-10-23T02:00:00+00:00")
    );
    assert_eq!(before, c.total_changes());
    assert_eq!(
        c.query_row("SELECT count(*) FROM records", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        0
    );
    let read = Connection::open_with_flags(
        temp.path().join("review.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(scan_review::build(&read, "scan").unwrap().groups.len(), 5);
    assert!(scan_review::build(&read, "missing").is_err());
}

#[test]
fn large_summary_does_not_walk_the_unrelated_small_file_population() {
    let (_temp, c, _s) = fixture("complete", 30_001);
    c.execute_batch("WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<30000) INSERT INTO files SELECT 'scan','/fixture/small-'||i,'/fixture','small','Other',1,1,0,'fixture','fixture' FROM n").unwrap();
    file(&c, "scan", "large", 2e9, 2e9, false);
    unsafe extern "C" fn stop_after_budget(context: *mut libc::c_void) -> libc::c_int {
        let ticks = unsafe { &mut *context.cast::<u32>() };
        *ticks = ticks.saturating_sub(1);
        i32::from(*ticks == 0)
    }
    let mut ticks = 100u32;
    unsafe {
        rusqlite::ffi::sqlite3_progress_handler(
            c.handle(),
            100,
            Some(stop_after_budget),
            (&mut ticks as *mut u32).cast(),
        )
    };
    let result = scan_review::build(&c, "scan");
    unsafe { rusqlite::ffi::sqlite3_progress_handler(c.handle(), 0, None, std::ptr::null_mut()) };
    assert_eq!(
        group(&result.unwrap(), ScanReviewKind::LargeFiles).count,
        Some(1)
    );
}
