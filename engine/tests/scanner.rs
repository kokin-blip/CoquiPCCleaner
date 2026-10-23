use coqui_engine::{database, model::*, platform, storage};
use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

fn session(id: &str) -> DiagnosticSession {
    DiagnosticSession {
        id: id.into(),
        started_at: now(),
        finished_at: None,
        status: "running".into(),
        stage: "test".into(),
        scope: "fixture".into(),
        file_count: 0,
        logical_bytes: 0.,
        allocated_bytes: 0.,
        skipped_count: 0,
        snapshot: None,
        findings: vec![],
        permissions: vec![],
        categories: vec![],
        candidates: vec![],
        errors: vec![],
    }
}

#[test]
fn traversal_is_read_only_and_deduplicates_links() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("nested/a.txt"), b"keep my data").unwrap();
    fs::hard_link(root.join("nested/a.txt"), root.join("copy.txt")).unwrap();
    std::os::unix::fs::symlink(&root, root.join("cycle")).unwrap();
    let before = fs::metadata(root.join("nested/a.txt"))
        .unwrap()
        .modified()
        .unwrap();
    let roots =
        storage::normalized_roots(vec![root.join("nested"), root.clone(), root.clone()]).unwrap();
    assert_eq!(roots.len(), 1);
    let mut entries = vec![];
    let w = storage::walk(
        &roots,
        &[],
        &AtomicBool::new(false),
        |f| {
            entries.push(f);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(w.files, 1);
    assert_eq!(w.logical, 12.);
    assert!(w.skipped >= 2);
    assert_eq!(
        fs::read(root.join("nested/a.txt")).unwrap(),
        b"keep my data"
    );
    assert_eq!(
        fs::metadata(root.join("nested/a.txt"))
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
    assert_eq!(
        entries
            .iter()
            .find(|f| f.path == root.to_str().unwrap())
            .unwrap()
            .logical_bytes,
        12.
    );
}
#[test]
fn exclusions_placeholders_and_cancellation() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    fs::create_dir(root.join("excluded")).unwrap();
    fs::write(root.join("excluded/secret"), b"secret").unwrap();
    fs::write(root.join(".cloud.icloud"), b"placeholder").unwrap();
    fs::write(root.join("visible"), b"data").unwrap();
    let w = storage::walk(
        std::slice::from_ref(&root),
        &[root.join("excluded")],
        &AtomicBool::new(false),
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(w.files, 1);
    assert_eq!(w.logical, 4.);
    assert_eq!(w.skipped, 2);
    let w = storage::walk(
        &[root],
        &[],
        &AtomicBool::new(true),
        |_| panic!("cancelled traversal emitted a file"),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(w.files, 0);
}
#[test]
fn malformed_and_missing_system_values_stay_missing() {
    let v = serde_json::json!({"SPHardwareDataType":[{"chip_type":"Apple test chip","serial_number":"MUST_NOT_PERSIST"}]});
    assert_eq!(
        platform::json_string(&v, &["chip_type"]).unwrap(),
        "Apple test chip"
    );
    assert!(platform::json_string(&v, &["boot_rom_version"]).is_none());
    assert!(serde_json::from_slice::<serde_json::Value>(b"not JSON").is_err());
    let e = DiagnosticEvidence::unavailable("Temperature", "Not supported");
    assert_ne!(e.state, CapabilityState::Supported);
    assert_eq!(e.value, "Unavailable");
}
#[test]
fn startup_configuration_does_not_claim_runtime_status() {
    let mut d = plist::Dictionary::new();
    d.insert("Label".into(), "org.test.agent".into());
    d.insert("Disabled".into(), false.into());
    let item = platform::startup_from_plist(
        &PathBuf::from("/Library/LaunchAgents/test.plist"),
        &plist::Value::Dictionary(d),
    )
    .unwrap();
    assert!(item.state.contains("unknown"));
    assert!(platform::startup_from_plist(
        &PathBuf::from("/test"),
        &plist::Value::String("bad".into())
    )
    .is_none());
}
#[test]
fn database_roundtrip_pagination_recovery_and_purge() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test.sqlite3");
    let mut c = database::open(&path).unwrap();
    let mut settings = AppSettings::default();
    settings.onboarding_complete = true;
    database::save_settings(&c, &settings).unwrap();
    let s = session("one");
    database::save_session(&c, &s).unwrap();
    let items = (0..4)
        .map(|i| FileEntry {
            path: format!("/fixture/{i}"),
            parent: "/fixture".into(),
            name: i.to_string(),
            category: "Other".into(),
            logical_bytes: i as f64,
            allocated_bytes: i as f64 * 512.,
            is_directory: false,
            modified_at: now(),
            qualification: String::new(),
        })
        .collect::<Vec<_>>();
    database::insert_files(&mut c, &s.id, &items).unwrap();
    drop(c);
    let c = database::open(&path).unwrap();
    assert!(database::settings(&c).unwrap().onboarding_complete);
    let result = database::files(
        &c,
        &StorageQuery {
            session_id: "one".into(),
            parent: Some("/fixture".into()),
            category: None,
            sort: "allocated; DROP TABLE files".into(),
            descending: true,
            offset: 1,
            limit: 2,
            files_only: true,
        },
    )
    .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "2");
    database::recover_interrupted(&c).unwrap();
    assert_eq!(database::session(&c, "one").unwrap().status, "interrupted");
    database::purge(&c).unwrap();
    assert!(database::history(&c, 0, 20).unwrap().is_empty());
    assert!(database::settings(&c).unwrap().onboarding_complete);
}
#[test]
fn missing_paths_are_partial_and_symlink_roots_are_rejected() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().join("gone");
    fs::create_dir(&root).unwrap();
    let roots = storage::normalized_roots(vec![root.clone()]).unwrap();
    fs::remove_dir(&root).unwrap();
    let missing =
        storage::walk(&roots, &[], &AtomicBool::new(false), |_| Ok(()), |_| Ok(())).unwrap();
    assert_eq!(missing.errors.len(), 1);
    std::os::unix::fs::symlink(d.path(), &root).unwrap();
    assert!(storage::normalized_roots(vec![root]).is_err());
}
#[test]
fn invalid_unicode_is_not_rewritten_to_a_different_path() {
    use std::os::unix::ffi::OsStringExt;
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    if let Err(e) = fs::write(root.join(std::ffi::OsString::from_vec(vec![0xff])), b"data") {
        // APFS or the macOS sandbox may reject the name before traversal.
        // Linux filesystems exercise the walker branch below.
        assert!(
            cfg!(target_os = "macos") && matches!(e.raw_os_error(), Some(1 | 22 | 92)),
            "unexpected fixture creation error: {e}"
        );
        return;
    }
    let w = storage::walk(
        &[root],
        &[],
        &AtomicBool::new(false),
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(w.files, 0);
    assert_eq!(w.errors.len(), 1);
}

#[test]
fn disconnected_selected_root_does_not_discard_accessible_roots() {
    let d = tempfile::tempdir().unwrap();
    let request = ScanRequest {
        mode: "selected".into(),
        roots: vec![
            d.path().to_string_lossy().into_owned(),
            d.path().join("disconnected").to_string_lossy().into_owned(),
        ],
        excluded_paths: vec![],
    };
    let (valid, gaps) = storage::roots(&request).unwrap();
    assert_eq!(valid.len(), 1);
    assert_eq!(gaps.len(), 1);
}

#[test]
fn permission_denial_is_reported_while_other_files_are_scanned() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let root = d.path().canonicalize().unwrap();
    let denied = root.join("denied");
    fs::create_dir(&denied).unwrap();
    fs::write(denied.join("private.txt"), b"private").unwrap();
    fs::write(root.join("visible.txt"), b"visible").unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0)).unwrap();
    let result = storage::walk(
        &[root],
        &[],
        &AtomicBool::new(false),
        |_| Ok(()),
        |_| Ok(()),
    );
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).unwrap();
    let summary = result.unwrap();
    assert_eq!(summary.files, 1);
    assert_eq!(summary.permissions.len(), 1);
}

#[test]
fn newer_database_schema_is_not_silently_downgraded() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("newer.db");
    let c = database::open(&path).unwrap();
    c.execute_batch("PRAGMA user_version=6").unwrap();
    drop(c);
    assert!(database::open(&path).is_err());
}
#[test]
fn old_complete_indexes_are_pruned_without_losing_summary() {
    let d = tempfile::tempdir().unwrap();
    let c = database::open(&d.path().join("t.db")).unwrap();
    let mut a = session("old");
    a.started_at = "2026-10-20T01:00:00+00:00".into();
    a.status = "complete".into();
    database::save_session(&c, &a).unwrap();
    let mut b = session("new");
    b.status = "complete".into();
    database::save_session(&c, &b).unwrap();
    database::retain(&c, 3650).unwrap();
    assert_eq!(database::history(&c, 0, 10).unwrap().len(), 2);
}
