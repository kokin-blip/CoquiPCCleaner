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
    api::cleanup_files(session_id,offset).map_err(error)
}
#[tauri::command]
fn preview_cleanup(request: CleanupRequest) -> Response<CleanupPlan> {
    api::preview_cleanup(request).map_err(error)
}
#[tauri::command]
async fn preview_project_cleanup(request: CleanupRequest) -> Response<CleanupPlan> {
    tauri::async_runtime::spawn_blocking(move || api::preview_project_cleanup(request)).await.map_err(error)?.map_err(error)
}
#[tauri::command]
fn project_cleanup_entries(id: String, offset: u32) -> Response<ProjectCleanupPage> {
    api::project_cleanup_entries(id,offset).map_err(error)
}
#[tauri::command]
fn cleanup_history(offset: u32, limit: u32) -> Response<Vec<CleanupPlan>> {
    api::cleanup_history(offset,limit).map_err(error)
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
fn cancel_cleanup() -> Response<()> {api::cancel_cleanup().map_err(error)}
#[tauri::command]
async fn execute_cleanup(approval: CleanupApproval, on_event: Channel<CleanupPlan>) -> Response<CleanupPlan> {
    tauri::async_runtime::spawn_blocking(move || api::execute_cleanup(approval,|e|on_event.send(e).is_ok())).await.map_err(error)?.map_err(error)
}
#[tauri::command]
fn developer_report(session_id: String) -> Response<Option<DeveloperReport>> {
    api::developer_report(session_id).map_err(error)
}
#[tauri::command]
fn cancel_developer() -> Response<()> { api::cancel_developer().map_err(error) }
#[tauri::command]
async fn analyze_developer(session_id: String, on_event: Channel<DeveloperReport>) -> Response<DeveloperReport> {
    tauri::async_runtime::spawn_blocking(move || api::analyze_developer(session_id, |e| on_event.send(e).is_ok())).await.map_err(error)?.map_err(error)
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
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
                let _ = api::cancel_developer();
                let _ = api::cancel_scan();
                let _ = api::cancel_cleanup();
                let _ = api::stop_monitoring();
            }
        });
}
