use coqui_engine::{api, model::*};
#[test]
fn real_inventory_scan_reopen_metrics_cancel_and_purge() {
    let data = tempfile::tempdir().unwrap();
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("keep.txt"), b"preserve these bytes").unwrap();
    let settings = api::init_engine(data.path().to_string_lossy().into()).unwrap();
    assert_eq!(settings.retention_days, 30);
    let request = ScanRequest {
        mode: "selected".into(),
        roots: vec![fixture.path().to_string_lossy().into()],
        excluded_paths: vec![],
    };
    let mut events = vec![];
    api::scan(request.clone(), |e| {
        events.push(e);
        true
    })
    .unwrap();
    let last = &events.last().unwrap().session;
    assert_eq!(last.file_count, 1);
    assert_eq!(last.status, "complete");
    assert!(last.snapshot.as_ref().unwrap().memory_bytes > 0.);
    assert!(!last.snapshot.as_ref().unwrap().cpu.is_empty());
    let json = serde_json::to_string(last).unwrap();
    assert!(json.contains("fileCount"));
    assert!(!json.contains("serial_number"));
    api::init_engine(data.path().to_string_lossy().into()).unwrap();
    assert_eq!(api::scan_history(0, 20).unwrap().len(), 1);
    let rows = api::storage_entries(StorageQuery {
        session_id: last.id.clone(),
        parent: Some("".into()),
        category: None,
        sort: "allocated".into(),
        descending: true,
        offset: 0,
        limit: 100,
        files_only: false,
    })
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].is_directory);
    let mut metrics = 0;
    api::monitor(|m| {
        metrics += 1;
        assert!((0.0..=100.0).contains(&m.cpu_percent));
        api::stop_monitoring().unwrap();
        true
    })
    .unwrap();
    assert_eq!(metrics, 1);
    let mut cancelled = None;
    api::scan(request, |e| {
        if e.session.status == "running" {
            api::cancel_scan().unwrap();
        } else {
            cancelled = Some(e.session);
        }
        true
    })
    .unwrap();
    assert_eq!(cancelled.unwrap().status, "cancelled");
    assert_eq!(
        std::fs::read(fixture.path().join("keep.txt")).unwrap(),
        b"preserve these bytes"
    );
    std::fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname='fixture'\n",
    )
    .unwrap();
    std::fs::create_dir(fixture.path().join("target")).unwrap();
    std::fs::write(
        fixture.path().join("target/build.cache"),
        "keep generated fixture",
    )
    .unwrap();
    let mut indexed = None;
    api::scan(
        ScanRequest {
            mode: "selected".into(),
            roots: vec![fixture.path().to_string_lossy().into()],
            excluded_paths: vec![],
        },
        |e| {
            indexed = Some(e.session);
            true
        },
    )
    .unwrap();
    let id = indexed.unwrap().id;
    let mut developer_events = vec![];
    let report = api::analyze_developer(id.clone(), |r| {
        developer_events.push(r.status);
        true
    })
    .unwrap();
    assert_eq!(report.artifacts.len(), 1);
    assert_eq!(report.artifacts[0].git.status, "notDetected");
    assert!(developer_events.contains(&"running".to_string()));
    api::init_engine(data.path().to_string_lossy().into()).unwrap();
    assert_eq!(
        api::developer_report(id.clone())
            .unwrap()
            .unwrap()
            .artifacts
            .len(),
        1
    );
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("target/build.cache")).unwrap(),
        "keep generated fixture"
    );
    // Exercise the real typed engine boundary for a project review, reopening
    // and declining it without invoking native mutation in the standard suite.
    std::fs::write(fixture.path().join(".gitignore"), "/target/\n").unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["add", "Cargo.toml", "keep.txt", ".gitignore"],
        vec!["commit", "-m", "fixture"],
    ] {
        let status = std::process::Command::new("/usr/bin/git")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
                "-C",
            ])
            .arg(fixture.path())
            .args(args)
            .output()
            .unwrap();
        assert!(status.status.success());
    }
    assert_eq!(
        api::analyze_developer(id.clone(), |_| true)
            .unwrap()
            .artifacts[0]
            .git
            .status,
        "clean"
    );
    let path = fixture
        .path()
        .canonicalize()
        .unwrap()
        .join("target")
        .to_str()
        .unwrap()
        .to_owned();
    let plan = api::preview_project_cleanup(CleanupRequest {
        session_id: id.clone(),
        paths: vec![path],
    })
    .unwrap();
    assert!(plan.project_cleanup);
    api::init_engine(data.path().to_string_lossy().into()).unwrap();
    assert!(
        api::get_cleanup_plan(plan.id.clone())
            .unwrap()
            .project_cleanup
    );
    assert_eq!(
        api::project_cleanup_entries(plan.id.clone(), 0)
            .unwrap()
            .total,
        1
    );
    assert_eq!(
        api::decline_cleanup(plan.id.clone()).unwrap().status,
        "declined"
    );
    api::clear_diagnostic_data().unwrap();
    assert!(api::scan_history(0, 10).unwrap().is_empty());
    assert!(api::get_cleanup_plan(plan.id).is_err());
    assert!(api::developer_report(id).unwrap().is_none());
    assert!(fixture.path().join("keep.txt").exists());
}
