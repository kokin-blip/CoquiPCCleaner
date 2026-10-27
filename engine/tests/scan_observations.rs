use coqui_engine::{api, database, model::*, scan_observations::*, scan_review};
use std::sync::atomic::AtomicBool;

fn request(path: &std::path::Path) -> ScanRequest {
    ScanRequest {
        plan_id: None,
        mode: "selected".into(),
        roots: vec![path.to_string_lossy().into_owned()],
        excluded_paths: vec![],
    }
}
#[test]
fn selected_storage_scan_records_skipped_checks_and_cancelled_scan_without_legacy_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let files = root.join("disposable");
    std::fs::create_dir(&files).unwrap();
    std::fs::write(files.join("fixture.txt"), b"fixture").unwrap();
    api::init_engine(root.join("data").to_string_lossy().into_owned()).unwrap();
    let modules = ScanModules {
        system: false,
        startup: false,
        applications: false,
    };
    let mut terminal = None;
    api::scan_with_modules(request(&files), modules.clone(), |event| {
        terminal = Some(event.session);
        true
    })
    .unwrap();
    let scan = terminal.unwrap();
    assert_eq!(scan.file_count, 1);
    assert!(scan.snapshot.is_none());
    let report = api::scan_observations(scan.id.clone()).unwrap().unwrap();
    assert_eq!(report.status, "complete");
    assert!(report
        .checks
        .iter()
        .all(|check| check.status == "notSelected"
            && check.count.is_none()
            && check.finished_at.is_none()));
    assert_eq!(
        std::fs::read(files.join("fixture.txt")).unwrap(),
        b"fixture"
    );
    let mut cancelled = None;
    api::scan_with_modules(request(&files), modules, |event| {
        cancelled = Some(event.session);
        false
    })
    .unwrap();
    assert_eq!(cancelled.unwrap().status, "cancelled");
    assert_eq!(
        api::scan_observations(scan.id).unwrap().unwrap().status,
        "complete"
    );
}
#[test]
fn disconnect_before_collection_cancels_unstarted_checks_without_completed_dates() {
    let mut report = begin(
        "fixture",
        &ScanModules {
            system: true,
            startup: true,
            applications: true,
        },
    );
    collect(&mut report, &AtomicBool::new(false), |_| false).unwrap();
    assert_eq!(report.status, "cancelled");
    assert!(report.system.is_none());
    assert!(report.startup_items.is_empty());
    assert!(report.applications.is_empty());
    assert!(report
        .checks
        .iter()
        .all(|check| check.count.is_none() && check.status == "cancelled"));
    assert!(report.checks[1].started_at.is_none());
    assert!(report.checks[1].finished_at.is_none());
}
#[test]
fn per_scan_not_selected_overrides_other_inventory_and_saved_reads_never_write() {
    let temp = tempfile::tempdir().unwrap();
    let c = database::open(&temp.path().join("data.sqlite3")).unwrap();
    let session: DiagnosticSession = serde_json::from_value(serde_json::json!({"id":"fixture","startedAt":"2026-10-26T12:00:00Z","finishedAt":"2026-10-26T12:01:00Z","status":"partial","stage":"fixture","scope":"fixture","fileCount":1,"logicalBytes":0,"allocatedBytes":0,"skippedCount":0,"snapshot":null,"findings":[],"permissions":[],"categories":[],"candidates":[],"errors":[]})).unwrap();
    database::save_session(&c, &session).unwrap();
    let mut report = begin(
        &session.id,
        &ScanModules {
            system: false,
            startup: false,
            applications: false,
        },
    );
    finish(&mut report, false);
    save(&c, &report).unwrap();
    c.execute(
        "INSERT INTO application_observations VALUES('/fixture.app',?1,NULL)",
        [r#"{"observedAt":"2026-10-26T12:00:00Z"}"#],
    )
    .unwrap();
    let changes = c.total_changes();
    c.execute_batch("PRAGMA query_only=ON").unwrap();
    let review = scan_review::build(&c, &session.id).unwrap();
    for kind in [ScanReviewKind::Startup, ScanReviewKind::Applications] {
        let group = review
            .groups
            .iter()
            .find(|group| group.kind == kind)
            .unwrap();
        assert_eq!(group.state, ScanReviewState::NotChecked);
        assert!(group.count.is_none());
        assert!(group.observed_at.is_none());
        assert_eq!(group.source, ScanReviewSource::RetainedScanObservations);
    }
    assert!(read(&c, "legacy").unwrap().is_none());
    assert_eq!(read(&c, "fixture").unwrap().unwrap().id, report.id);
    assert_eq!(c.total_changes(), changes);
}
#[test]
fn interrupted_pending_checks_stay_incomplete_on_reopen_without_rewriting_history() {
    let temp = tempfile::tempdir().unwrap();
    let c = database::open(&temp.path().join("data.sqlite3")).unwrap();
    let report = begin(
        "fixture",
        &ScanModules {
            system: true,
            startup: false,
            applications: true,
        },
    );
    save(&c, &report).unwrap();
    database::recover_interrupted(&c).unwrap();
    let changes = c.total_changes();
    let retained = read(&c, "fixture").unwrap().unwrap();
    assert_eq!(retained.status, "interrupted");
    assert_eq!(retained.checks[0].status, "interrupted");
    assert_eq!(retained.checks[1].status, "notSelected");
    assert!(retained.checks[0].finished_at.is_none());
    assert_eq!(c.total_changes(), changes);
}
#[test]
#[ignore = "Explicit native foreground metadata validation with isolated database; reads real OS metadata only"]
fn native_metadata_checks_use_isolated_history_and_keep_fixture_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let files = root.join("Coqui scan module validation");
    std::fs::create_dir(&files).unwrap();
    std::fs::write(files.join("fixture.txt"), b"fixture").unwrap();
    api::init_engine(root.join("data").to_string_lossy().into_owned()).unwrap();
    let mut terminal = None;
    api::scan_with_modules(
        request(&files),
        ScanModules {
            system: true,
            startup: true,
            applications: true,
        },
        |event| {
            terminal = Some(event.session);
            true
        },
    )
    .unwrap();
    let scan = terminal.unwrap();
    let report = api::scan_observations(scan.id.clone()).unwrap().unwrap();
    assert_eq!(scan.file_count, 1);
    assert!(!["running", "failed", "cancelled"].contains(&scan.status.as_str()));
    assert!(report
        .checks
        .iter()
        .all(|check| check.started_at.is_some() && check.finished_at.is_some()));
    assert!(report.system.is_some());
    assert!(report
        .checks
        .iter()
        .find(|check| check.module == "applications")
        .unwrap()
        .count
        .is_some_and(|count| count > 0));
    std::fs::write(
        "/private/tmp/coqui-native-scan-observations.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    std::fs::write(
        "/private/tmp/coqui-native-scan-review.json",
        serde_json::to_vec_pretty(&api::scan_review(scan.id.clone()).unwrap()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        "/private/tmp/coqui-native-scan-session.json",
        serde_json::to_vec_pretty(&scan).unwrap(),
    )
    .unwrap();
    assert_eq!(
        std::fs::read(files.join("fixture.txt")).unwrap(),
        b"fixture"
    );
}
