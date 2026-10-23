#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use coqui_engine::{api, model::*};
use tauri::{ipc::Channel, Manager};
type Response<T> = Result<T, String>;
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
fn initialize(app: tauri::AppHandle) -> Response<AppSettings> {
    let dir = app
        .path()
        .home_dir()
        .map_err(error)?
        .join("Library/Application Support/CoquiPCCleaner");
    api::init_engine(dir.to_string_lossy().into_owned()).map_err(error)
}
#[tauri::command]
fn get_settings() -> Response<AppSettings> {
    api::get_settings().map_err(error)
}
#[tauri::command]
fn update_settings(settings: AppSettings) -> Response<()> {
    api::update_settings(settings).map_err(error)
}
#[tauri::command]
fn scan_history(offset: u32, limit: u32) -> Response<Vec<DiagnosticSession>> {
    api::scan_history(offset, limit).map_err(error)
}
#[tauri::command]
fn get_session(id: String) -> Response<DiagnosticSession> {
    api::get_session(id).map_err(error)
}
#[tauri::command]
fn storage_entries(query: StorageQuery) -> Response<Vec<FileEntry>> {
    api::storage_entries(query).map_err(error)
}
#[tauri::command]
fn clear_diagnostic_data() -> Response<()> {
    api::clear_diagnostic_data().map_err(error)
}
#[tauri::command]
fn export_report(id: String, path: String) -> Response<()> {
    let target = std::path::Path::new(&path);
    if !target.is_absolute() || target.extension().is_none_or(|e| e != "json") {
        return Err("Choose an absolute path ending in .json".into());
    }
    let report = api::get_session(id).map_err(error)?;
    let output = serde_json::to_vec_pretty(&report).map_err(error)?;
    std::fs::write(target, output).map_err(error)
}
#[tauri::command]
fn cancel_scan() -> Response<()> {
    api::cancel_scan().map_err(error)
}
#[tauri::command]
fn stop_monitoring() -> Response<()> {
    api::stop_monitoring().map_err(error)
}
#[tauri::command]
async fn scan(request: ScanRequest, on_event: Channel<ScanEvent>) -> Response<()> {
    tauri::async_runtime::spawn_blocking(move || api::scan(request, |e| on_event.send(e).is_ok()))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
async fn monitor(on_event: Channel<LiveMetrics>) -> Response<()> {
    tauri::async_runtime::spawn_blocking(move || api::monitor(|e| on_event.send(e).is_ok()))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
fn reveal_path(path: String) -> Response<()> {
    let p = std::path::Path::new(&path);
    if !p.is_absolute() || !p.exists() {
        return Err("This location is no longer available in Finder.".into());
    }
    let status = std::process::Command::new("/usr/bin/open")
        .args(["-R", "--", &path])
        .status()
        .map_err(error)?;
    if status.success() {
        Ok(())
    } else {
        Err("Finder could not reveal this location.".into())
    }
}
#[tauri::command]
fn open_system_settings(destination: String) -> Response<()> {
    let url = match destination.as_str() {
        "permissions" => "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
        "login" => "x-apple.systempreferences:com.apple.LoginItems-Settings.extension",
        "power" => "x-apple.systempreferences:com.apple.Battery-Settings.extension",
        "storage" => "x-apple.systempreferences:com.apple.settings.Storage",
        "updates" => "x-apple.systempreferences:com.apple.Software-Update-Settings.extension",
        "security" => "x-apple.systempreferences:com.apple.preference.security",
        _ => return Err("Unknown System Settings destination".into()),
    };
    let status = std::process::Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(error)?;
    if status.success() {
        Ok(())
    } else {
        Err("System Settings could not be opened. Open it from the Apple menu.".into())
    }
}
#[tauri::command]
fn cleanup_files(session_id: String, offset: u32) -> Response<CleanupFilePage> {
    api::cleanup_files(session_id, offset).map_err(error)
}
#[tauri::command]
fn preview_cleanup(request: CleanupRequest) -> Response<CleanupPlan> {
    api::preview_cleanup(request).map_err(error)
}
#[tauri::command]
async fn preview_project_cleanup(request: CleanupRequest) -> Response<CleanupPlan> {
    tauri::async_runtime::spawn_blocking(move || api::preview_project_cleanup(request))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
fn project_cleanup_entries(id: String, offset: u32) -> Response<ProjectCleanupPage> {
    api::project_cleanup_entries(id, offset).map_err(error)
}
#[tauri::command]
fn cleanup_history(offset: u32, limit: u32) -> Response<Vec<CleanupPlan>> {
    api::cleanup_history(offset, limit).map_err(error)
}
#[tauri::command]
fn get_cleanup_plan(id: String) -> Response<CleanupPlan> {
    api::get_cleanup_plan(id).map_err(error)
}
#[tauri::command]
fn decline_cleanup(id: String) -> Response<CleanupPlan> {
    api::decline_cleanup(id).map_err(error)
}
#[tauri::command]
fn cancel_cleanup() -> Response<()> {
    api::cancel_cleanup().map_err(error)
}
#[tauri::command]
async fn execute_cleanup(
    approval: CleanupApproval,
    on_event: Channel<CleanupPlan>,
) -> Response<CleanupPlan> {
    tauri::async_runtime::spawn_blocking(move || {
        api::execute_cleanup(approval, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}
#[tauri::command]
fn developer_report(session_id: String) -> Response<Option<DeveloperReport>> {
    api::developer_report(session_id).map_err(error)
}
#[tauri::command]
fn cancel_developer() -> Response<()> {
    api::cancel_developer().map_err(error)
}
#[tauri::command]
async fn analyze_developer(
    session_id: String,
    on_event: Channel<DeveloperReport>,
) -> Response<DeveloperReport> {
    tauri::async_runtime::spawn_blocking(move || {
        api::analyze_developer(session_id, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}

#[tauri::command]
fn cancel_work() -> Response<()> {
    api::cancel_work().map_err(error)
}
#[tauri::command]
async fn analyze_duplicates(
    session_id: String,
    on_event: Channel<DuplicateReport>,
) -> Response<DuplicateReport> {
    tauri::async_runtime::spawn_blocking(move || {
        api::analyze_duplicates(session_id, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}
#[tauri::command]
async fn preview_duplicate_cleanup(request: DuplicateCleanupRequest) -> Response<CleanupPlan> {
    tauri::async_runtime::spawn_blocking(move || api::preview_duplicate_cleanup(request))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
async fn capture_baseline(
    request: BaselineRequest,
    on_event: Channel<Baseline>,
) -> Response<Baseline> {
    tauri::async_runtime::spawn_blocking(move || {
        api::capture_baseline(request, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}
#[tauri::command]
fn compare_baselines(before: String, after: String) -> Response<BaselineComparison> {
    api::compare_baselines(before, after).map_err(error)
}
#[tauri::command]
async fn create_manual_plan(kind: String, requested: String) -> Response<ManualPlan> {
    tauri::async_runtime::spawn_blocking(move || api::create_manual_plan(kind, requested))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
async fn verify_manual_plan(id: String, performed: bool) -> Response<ManualPlan> {
    tauri::async_runtime::spawn_blocking(move || api::verify_manual_plan(id, performed))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
fn ask_coqui(
    id: Option<String>,
    session_id: Option<String>,
    text: String,
) -> Response<Investigation> {
    api::ask_coqui(id, session_id, text).map_err(error)
}
#[tauri::command]
fn assess_workload(request: AdviceRequest) -> Response<AdviceReport> {
    api::assess_workload(request).map_err(error)
}
#[tauri::command]
async fn analyze_resources(session_id: String) -> Response<ResourceReview> {
    tauri::async_runtime::spawn_blocking(move || api::analyze_resources(session_id))
        .await
        .map_err(error)?
        .map_err(error)
}
#[tauri::command]
fn activity_records(kind: Option<String>, offset: u32) -> Response<Vec<ActivityRecord>> {
    api::activity_records(kind, offset).map_err(error)
}
#[tauri::command]
fn get_record(kind: String, id: String) -> Response<RetainedRecord> {
    api::get_record(kind, id).map_err(error)
}
#[tauri::command]
fn maintenance_rule() -> Response<Option<MaintenanceRule>> {
    api::maintenance_rule().map_err(error)
}
#[tauri::command]
fn save_maintenance(rule: MaintenanceRule) -> Response<()> {
    api::save_maintenance(rule).map_err(error)
}
#[tauri::command]
async fn maintenance_tick() -> Response<Option<MaintenanceEvent>> {
    tauri::async_runtime::spawn_blocking(api::maintenance_tick)
        .await
        .map_err(error)?
        .map_err(error)
}

#[tauri::command]
fn open_reference(url: String) -> Response<()> {
    let allowed = [
        "https://support.apple.com/en-us/101613",
        "https://support.apple.com/en-ie/102328",
        "https://support.apple.com/macos/upgrade",
        "https://www.apple.com/mac/compare/",
        coqui_engine::firmware::UPDATE_GUIDE,
        coqui_engine::firmware::POLICY_GUIDE,
        coqui_engine::firmware::RECOVERY_GUIDE,
    ];
    if !allowed.contains(&url.as_str()) {
        return Err("Unknown reference link".into());
    }
    let status = std::process::Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(error)?;
    if status.success() {
        Ok(())
    } else {
        Err("Could not open reference in your browser".into())
    }
}
#[tauri::command]
async fn audit_firmware(
    request: FirmwareRequest,
    on_event: Channel<FirmwareReport>,
) -> Response<FirmwareReport> {
    tauri::async_runtime::spawn_blocking(move || {
        api::audit_firmware(request, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}
#[tauri::command]
fn firmware_plan(report_id: String, kind: FirmwarePlanKind) -> Response<FirmwarePlan> {
    api::firmware_plan(report_id, kind).map_err(error)
}
#[tauri::command]
fn acknowledge_firmware_plan(id: String, accepted: bool) -> Response<FirmwarePlan> {
    api::acknowledge_firmware_plan(id, accepted).map_err(error)
}
#[tauri::command]
async fn observe_firmware_plan(
    id: String,
    performed: bool,
    on_event: Channel<FirmwareReport>,
) -> Response<FirmwarePlan> {
    tauri::async_runtime::spawn_blocking(move || {
        api::observe_firmware_plan(id, performed, |e| on_event.send(e).is_ok())
    })
    .await
    .map_err(error)?
    .map_err(error)
}
#[tauri::command]
fn compare_firmware(before: String, after: String) -> Response<FirmwareComparison> {
    api::compare_firmware(before, after).map_err(error)
}
#[tauri::command]
fn firmware_center() -> Response<FirmwareCenter> {
    api::firmware_center().map_err(error)
}
#[tauri::command]
fn save_firmware_reminder(enabled: bool, days: u32) -> Response<FirmwareCenter> {
    api::save_firmware_reminder(enabled, days).map_err(error)
}
#[tauri::command]
fn dismiss_firmware_notice(id: String) -> Response<FirmwareCenter> {
    api::dismiss_firmware_notice(id).map_err(error)
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_reference,
            compare_firmware,
            firmware_center,
            save_firmware_reminder,
            dismiss_firmware_notice,
            audit_firmware,
            firmware_plan,
            acknowledge_firmware_plan,
            observe_firmware_plan,
            cancel_work,
            analyze_duplicates,
            preview_duplicate_cleanup,
            capture_baseline,
            compare_baselines,
            create_manual_plan,
            verify_manual_plan,
            ask_coqui,
            assess_workload,
            analyze_resources,
            activity_records,
            get_record,
            maintenance_rule,
            save_maintenance,
            maintenance_tick,
            developer_report,
            analyze_developer,
            cancel_developer,
            cleanup_files,
            preview_cleanup,
            preview_project_cleanup,
            project_cleanup_entries,
            cleanup_history,
            get_cleanup_plan,
            decline_cleanup,
            execute_cleanup,
            cancel_cleanup,
            initialize,
            get_settings,
            update_settings,
            scan_history,
            get_session,
            storage_entries,
            clear_diagnostic_data,
            export_report,
            cancel_scan,
            stop_monitoring,
            scan,
            monitor,
            reveal_path,
            open_system_settings
        ])
        .build(tauri::generate_context!())
        .expect("Could not initialize CoquiPCCleaner")
        .run(|_app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                let _ = api::cancel_work();
                let _ = api::cancel_developer();
                let _ = api::cancel_scan();
                let _ = api::cancel_cleanup();
                let _ = api::stop_monitoring();
            }
        });
}
