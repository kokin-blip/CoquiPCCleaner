use serde::{Deserialize, Serialize};

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityState {
    Supported,
    RequiresPermission,
    ManualOnly,
    Unsupported,
    Unknown,
    TemporarilyUnavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapability {
    pub name: String,
    pub state: CapabilityState,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub label: String,
    pub value: String,
    pub unit: String,
    pub source: String,
    pub observed_at: String,
    pub state: CapabilityState,
    pub qualification: String,
}
impl DiagnosticEvidence {
    pub fn observed(label: &str, value: String, unit: &str, source: &str) -> Self {
        Self {
            label: label.into(),
            value,
            unit: unit.into(),
            source: source.into(),
            observed_at: now(),
            state: CapabilityState::Supported,
            qualification: String::new(),
        }
    }
    pub fn unavailable(label: &str, explanation: &str) -> Self {
        Self {
            label: label.into(),
            value: "Unavailable".into(),
            unit: String::new(),
            source: "macOS adapter".into(),
            observed_at: now(),
            state: CapabilityState::Unknown,
            qualification: explanation.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub filesystem: String,
    pub total_bytes: f64,
    pub available_bytes: f64,
    pub removable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub name: String,
    pub path: String,
    pub mechanism: String,
    pub state: String,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApplication {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MachineSnapshot {
    pub observed_at: String,
    pub device_name: String,
    pub model: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu: String,
    pub core_count: u32,
    pub memory_bytes: f64,
    pub gpu: String,
    pub disks: Vec<DiskInfo>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub capabilities: Vec<SystemCapability>,
    pub startup_items: Vec<StartupItem>,
    pub applications: Vec<InstalledApplication>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadProfile {
    pub workloads: Vec<String>,
    pub preference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub onboarding_complete: bool,
    pub workload: WorkloadProfile,
    pub appearance: String,
    pub scan_mode: String,
    pub excluded_paths: Vec<String>,
    pub selected_roots: Vec<String>,
    pub retention_days: u32,
    pub monitoring_enabled: bool,
    pub advanced_diagnostics: bool,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            onboarding_complete: false,
            workload: WorkloadProfile {
                workloads: vec!["General productivity".into()],
                preference: "Balanced".into(),
            },
            appearance: "system".into(),
            scan_mode: "standard".into(),
            excluded_paths: vec![],
            selected_roots: vec![],
            retention_days: 30,
            monitoring_enabled: true,
            advanced_diagnostics: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticFinding {
    pub id: String,
    pub title: String,
    pub explanation: String,
    pub severity: String,
    pub confidence: String,
    pub kind: String,
    pub evidence: Vec<DiagnosticEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequirement {
    pub path: String,
    pub reason: String,
    pub guidance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCandidate {
    pub path: String,
    pub allocated_bytes: f64,
    pub logical_bytes: f64,
    pub modified_at: String,
    pub origin: String,
    pub reason: String,
    pub safety: String,
    pub consequences: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub parent: String,
    pub name: String,
    pub category: String,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub is_directory: bool,
    pub modified_at: String,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageCategory {
    pub name: String,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub file_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticSession {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub stage: String,
    pub scope: String,
    pub file_count: u32,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub skipped_count: u32,
    pub snapshot: Option<MachineSnapshot>,
    pub findings: Vec<DiagnosticFinding>,
    pub permissions: Vec<PermissionRequirement>,
    pub categories: Vec<StorageCategory>,
    pub candidates: Vec<CleanupCandidate>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub mode: String,
    pub roots: Vec<String>,
    pub excluded_paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanEvent {
    pub session: DiagnosticSession,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSample {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveMetrics {
    pub observed_at: String,
    pub cpu_percent: f32,
    pub used_memory_bytes: f64,
    pub total_memory_bytes: f64,
    pub swap_bytes: f64,
    pub disk_read_bytes: f64,
    pub disk_written_bytes: f64,
    pub processes: Vec<ProcessSample>,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageQuery {
    pub session_id: String,
    pub parent: Option<String>,
    pub category: Option<String>,
    pub sort: String,
    pub descending: bool,
    pub offset: u32,
    pub limit: u32,
    pub files_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupFile {
    pub path: String,
    pub allocated_bytes: f64,
    pub logical_bytes: f64,
    pub modified_at: String,
    pub origin: String,
    pub consequences: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItem {
    pub file: CleanupFile,
    pub status: String,
    pub detail: String,
    pub trash_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    #[serde(default)]
    pub duplicate_cleanup: bool,
    #[serde(default)]
    pub project_cleanup: bool,
    pub id: String,
    pub session_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub approved_at: Option<String>,
    pub finished_at: Option<String>,
    pub status: String,
    pub items: Vec<CleanupItem>,
    pub allocated_bytes: f64,
    pub verification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRequest {
    pub session_id: String,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupApproval {
    pub plan_id: String,
    pub reviewed_at: String,
    pub applications_closed: bool,
    pub consequences_accepted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupFilePage {
    pub files: Vec<CleanupFile>,
    pub next_offset: Option<u32>,
    pub unavailable_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCleanupEntry {
    pub path: String,
    pub directory: bool,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCleanupPage {
    pub entries: Vec<ProjectCleanupEntry>,
    pub total: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GitWorktree {
    pub path: String,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
    pub locked: bool,
    pub prunable: bool,
    pub present: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GitChanges {
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
    pub conflicts: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GitValidation {
    pub status: String,
    pub root: Option<String>,
    pub git_directory: Option<String>,
    pub common_directory: Option<String>,
    pub changes: Option<GitChanges>,
    pub worktrees: Vec<GitWorktree>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperArtifact {
    pub path: String,
    pub project_root: String,
    pub technology: String,
    pub kind: String,
    pub marker: String,
    pub indexed_allocated_bytes: f64,
    pub indexed_logical_bytes: f64,
    pub indexed_file_count: u32,
    pub scanned_at: String,
    pub checked_at: String,
    pub git: GitValidation,
    pub tracked_entries: Option<u32>,
    pub gitlinks: Option<u32>,
    pub nested_markers: bool,
    pub assessment: String,
    pub reasons: Vec<String>,
    pub consequences: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperReport {
    pub session_id: String,
    pub checked_at: String,
    pub status: String,
    pub examined: u32,
    pub candidate_count: u32,
    pub artifacts: Vec<DeveloperArtifact>,
    pub errors: Vec<String>,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFile {
    #[serde(default)]
    pub review_supported: bool,
    #[serde(default)]
    pub review_qualification: String,
    pub path: String,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub modified_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub id: String,
    pub digest: String,
    pub files: Vec<DuplicateFile>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMatch {
    pub left: String,
    pub right: String,
    pub distance: u32,
    pub classification: String,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateReport {
    pub id: String,
    pub session_id: String,
    pub created_at: String,
    pub status: String,
    pub stage: String,
    pub examined: u32,
    pub total: u32,
    pub hashed_bytes: f64,
    pub groups: Vec<DuplicateGroup>,
    pub photo_matches: Vec<PhotoMatch>,
    pub gaps: Vec<String>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCleanupRequest {
    pub report_id: String,
    pub group_id: String,
    pub keep_path: String,
    pub remove_paths: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct BaselineRequest {
    pub workload: String,
    pub target: String,
    pub context: String,
    pub duration_seconds: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Baseline {
    pub id: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub request: BaselineRequest,
    pub samples: Vec<LiveMetrics>,
    pub power: Vec<DiagnosticEvidence>,
    pub summary: Vec<DiagnosticEvidence>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct BaselineComparison {
    pub before: Baseline,
    pub after: Baseline,
    pub observations: Vec<String>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ManualPlan {
    pub id: String,
    pub created_at: String,
    pub kind: String,
    pub requested: String,
    pub previous: Vec<DiagnosticEvidence>,
    pub steps: Vec<String>,
    pub status: String,
    pub observed: Vec<DiagnosticEvidence>,
    pub verification: String,
    pub recovery: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceRule {
    pub enabled: bool,
    pub mode: String,
    pub interval_days: u32,
    pub next_due_at: String,
    pub scope: ScanRequest,
    pub capacity_threshold_percent: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceEvent {
    pub id: String,
    pub created_at: String,
    pub status: String,
    pub message: String,
    pub scan_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticTask {
    pub name: String,
    pub status: String,
    pub evidence: Vec<DiagnosticEvidence>,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct InvestigationMessage {
    pub role: String,
    pub text: String,
    pub created_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Investigation {
    pub id: String,
    pub created_at: String,
    pub title: String,
    pub session_id: Option<String>,
    pub messages: Vec<InvestigationMessage>,
    pub tasks: Vec<DiagnosticTask>,
    pub findings: Vec<DiagnosticFinding>,
    pub actions: Vec<String>,
    pub status: String,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AdviceRequest {
    pub session_id: String,
    pub baseline_id: Option<String>,
    pub workload: String,
    pub expectation: String,
    pub annual_growth_percent: f64,
    pub current_demand_percent: f64,
    pub quote_low: Option<f64>,
    pub quote_high: Option<f64>,
    pub quote_source: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AdviceReport {
    pub id: String,
    pub created_at: String,
    pub workload: String,
    pub assessment: String,
    pub findings: Vec<DiagnosticFinding>,
    pub scenarios: Vec<DiagnosticEvidence>,
    pub sources: Vec<String>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ResourceReview {
    pub id: String,
    pub created_at: String,
    pub session_id: String,
    pub capabilities: Vec<SystemCapability>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub suggestions: Vec<DiagnosticFinding>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    pub id: String,
    pub kind: String,
    pub created_at: String,
    pub title: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum RetainedRecord {
    Duplicates(DuplicateReport),
    Baseline(Baseline),
    ManualPlan(ManualPlan),
    Maintenance(MaintenanceEvent),
    Investigation(Investigation),
    Advice(AdviceReport),
    Resources(ResourceReview),
}
