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
    #[serde(default)]
    pub administrator_enabled: bool,
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
            administrator_enabled: false,
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
    #[serde(default)]
    pub volume_plan: Option<DiskScanPlan>,
    #[serde(default)]
    pub coverage: Vec<RootCoverage>,
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
    #[serde(default)]
    pub plan_id: Option<String>,
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
    #[serde(default)]
    pub search: Option<String>,
    pub session_id: String,
    pub parent: Option<String>,
    pub category: Option<String>,
    pub sort: String,
    pub descending: bool,
    pub offset: u32,
    pub limit: u32,
    pub files_only: bool,
}

/// A bounded map of immediate children. An empty parent selects scan roots.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapQuery {
    pub session_id: String,
    pub parent: String,
    pub metric: String,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapNode {
    pub entry: FileEntry,
    pub file_count: Option<f64>,
    pub small_file_count: Option<f64>,
    pub small_allocated_bytes: Option<f64>,
    pub largest_file_bytes: Option<f64>,
    pub kind: String,
    pub title: String,
    pub reason: String,
    pub consequences: String,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageMap {
    pub nodes: Vec<StorageMapNode>,
    pub total_allocated_bytes: f64,
    pub total_logical_bytes: f64,
    pub file_count: Option<f64>,
    pub remainder_allocated_bytes: f64,
    pub remainder_logical_bytes: f64,
    pub remainder_file_count: Option<f64>,
    pub qualification: String,
}

/// Persisted descendant measurements, never an assertion that an item is disposable.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StorageMetrics {
    pub file_count: u64,
    pub small_file_count: u64,
    pub small_allocated_bytes: f64,
    pub largest_file_bytes: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedStorageEntry {
    pub entry: FileEntry,
    pub metrics: Option<StorageMetrics>,
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
    Firmware(FirmwareReport),
    FirmwarePlan(FirmwarePlan),
    FirmwareComparison(FirmwareComparison),
}

/// Scope is explicit for every run; no capability is enabled by a saved default.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareRequest {
    pub inventory: bool,
    pub security: bool,
    pub updates: bool,
    pub network_acknowledged: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCompatibility {
    #[serde(default)]
    pub model_name: String,
    pub manufacturer: String,
    pub model: String,
    pub chip: String,
    pub architecture: String,
    pub os_version: String,
    pub os_build: String,
    pub eligible: bool,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareUpdate {
    pub label: String,
    pub title: String,
    pub version: String,
    pub restart: bool,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareReport {
    pub id: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub stage: String,
    pub request: FirmwareRequest,
    pub platform: PlatformCompatibility,
    pub evidence: Vec<DiagnosticEvidence>,
    pub capabilities: Vec<SystemCapability>,
    pub findings: Vec<DiagnosticFinding>,
    pub updates: Vec<FirmwareUpdate>,
    pub update_state: String,
    pub update_checked_at: Option<String>,
    pub errors: Vec<String>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FirmwarePlanKind {
    SoftwareUpdate,
    StartupPolicyReview,
    RecoveryReview,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwarePlan {
    pub id: String,
    pub created_at: String,
    pub report_id: String,
    pub kind: FirmwarePlanKind,
    pub status: String,
    pub title: String,
    pub before: PlatformCompatibility,
    pub previous: Vec<DiagnosticEvidence>,
    pub steps: Vec<String>,
    pub cautions: Vec<String>,
    pub source_url: String,
    pub source_reviewed_at: String,
    pub acknowledged_at: Option<String>,
    pub outcome: String,
    pub after_report_id: Option<String>,
    #[serde(default)]
    pub comparison_id: Option<String>,
    pub recovery: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareDelta {
    pub property: String,
    pub before: Option<DiagnosticEvidence>,
    pub after: Option<DiagnosticEvidence>,
    pub state: String,
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareComparison {
    pub id: String,
    pub created_at: String,
    pub status: String,
    pub before_report_id: String,
    pub after_report_id: String,
    pub before_observed_at: String,
    pub after_observed_at: String,
    pub before_platform: PlatformCompatibility,
    pub after_platform: PlatformCompatibility,
    pub comparable_platform: bool,
    pub rows: Vec<FirmwareDelta>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwarePreferences {
    pub reminder_enabled: bool,
    pub interval_days: u32,
    pub next_due_at: Option<String>,
    pub dismissed_report_id: Option<String>,
}
impl Default for FirmwarePreferences {
    fn default() -> Self {
        Self {
            reminder_enabled: false,
            interval_days: 7,
            next_due_at: None,
            dismissed_report_id: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareNotice {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub report_id: Option<String>,
    pub observed_at: String,
    pub stale: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareCenter {
    pub preferences: FirmwarePreferences,
    pub notices: Vec<FirmwareNotice>,
    pub latest_attempt_at: Option<String>,
    pub latest_attempt_state: String,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, Default)]
#[serde(rename_all = "camelCase")]
pub struct RootCoverage {
    pub root: String,
    pub state: String,
    pub files: u32,
    pub visited: u32,
    pub skips: std::collections::BTreeMap<String, u32>,
    pub examples: Vec<CoverageGap>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CoverageGap {
    pub root: String,
    pub path: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiskVolume {
    pub id: String,
    pub mount: String,
    pub device: String,
    pub filesystem: String,
    pub internal: bool,
    pub read_only: bool,
    pub selected: bool,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiskScanPlan {
    pub id: String,
    pub created_at: String,
    pub volumes: Vec<DiskVolume>,
    pub qualification: String,
}
