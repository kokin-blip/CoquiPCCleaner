//! Explicit public-catalog validation in an isolated disposable database.
#![cfg(target_os = "macos")]

use coqui_engine::{api, model::AppUpdateSource, model::AppUpdateState};

#[test]
#[ignore = "Opt-in native public-network validation; no installed-app inventory or app launch"]
fn public_catalog_fixture_uses_reviewed_transport_without_user_history() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let app = root.join("Coqui public catalog validation.app");
    std::fs::create_dir_all(app.join("Contents/_MASReceipt")).unwrap();
    std::fs::write(app.join("Contents/_MASReceipt/receipt"), []).unwrap();
    let mut info = plist::Dictionary::new();
    for (key, value) in [
        ("CFBundleIdentifier", "com.barebones.bbedit"),
        ("CFBundleName", "Coqui public catalog validation fixture"),
        ("CFBundlePackageType", "APPL"),
        ("CFBundleShortVersionString", "0.0"),
        ("CFBundleVersion", "0"),
    ] {
        info.insert(key.into(), plist::Value::String(value.into()));
    }
    plist::Value::Dictionary(info)
        .to_file_xml(app.join("Contents/Info.plist"))
        .unwrap();
    api::init_engine(root.join("isolated-data").to_string_lossy().into_owned()).unwrap();
    let plan = api::prepare_app_updates(vec![app.to_string_lossy().into_owned()]).unwrap();
    assert_eq!(plan.candidates[0].source, AppUpdateSource::AppStore);
    let report = api::check_app_updates(plan.id.clone(), true, |_| true).unwrap();
    std::fs::write(
        "/private/tmp/coqui-app-updates-native-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert_eq!(report.status, "complete", "{:?}", report.items);
    assert_eq!(report.items.len(), 1);
    let item = &report.items[0];
    assert_eq!(item.state, AppUpdateState::Available);
    assert_eq!(item.bundle_id, "com.barebones.bbedit");
    assert!(item.checked_at.is_some());
    assert!(item.offered_version.is_some());
    assert!(item.name.contains("fixture"));
    assert_eq!(api::latest_app_updates().unwrap().unwrap().id, report.id);
    assert!(api::check_app_updates(plan.id, true, |_| true).is_err());
    assert!(app.join("Contents/Info.plist").is_file());
    assert!(!app.join("Contents/MacOS").exists());
}
