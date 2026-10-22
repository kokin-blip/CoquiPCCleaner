use coqui_engine::model::*;
use ts_rs::TS;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings");
    AppSettings::export_all_to(&path)?;
    ScanRequest::export_all_to(&path)?;
    ScanEvent::export_all_to(&path)?;
    LiveMetrics::export_all_to(&path)?;
    StorageQuery::export_all_to(&path)?;
    FileEntry::export_all_to(&path)?;
    CleanupRequest::export_all_to(&path)?;
    CleanupApproval::export_all_to(&path)?;
    CleanupPlan::export_all_to(&path)?;
    CleanupFilePage::export_all_to(&path)?;
    ProjectCleanupPage::export_all_to(&path)?;
    DeveloperReport::export_all_to(&path)?;
    Ok(())
}
