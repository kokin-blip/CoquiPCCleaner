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
    api::clear_diagnostic_data().unwrap();
    assert!(api::scan_history(0, 10).unwrap().is_empty());
    assert!(fixture.path().join("keep.txt").exists());
}
