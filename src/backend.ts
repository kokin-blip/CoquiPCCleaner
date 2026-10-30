import type { CacheQuery } from './bindings/CacheQuery';
import type { CacheGroupPage } from './bindings/CacheGroupPage';
import type { CacheSelectionRequest } from './bindings/CacheSelectionRequest';
import type { CacheBulkSummary } from './bindings/CacheBulkSummary';
import type { CacheBulkApproval } from './bindings/CacheBulkApproval';
import type { CacheBulkPageQuery } from './bindings/CacheBulkPageQuery';
import type { CacheBulkPage } from './bindings/CacheBulkPage';
import type { HelperStatus } from './bindings/HelperStatus';
import type { DiskScanPlan } from './bindings/DiskScanPlan';
import type { CoverageGap } from './bindings/CoverageGap';
import type { ApplicationPage } from './bindings/ApplicationPage';
import type { ApplicationFootprint } from './bindings/ApplicationFootprint';
import type { ApplicationPreferences } from './bindings/ApplicationPreferences';
import type { RemovalRequest } from './bindings/RemovalRequest';
import type { RemovalApproval } from './bindings/RemovalApproval';
import type { RemovalPlan } from './bindings/RemovalPlan';
import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { AppSettings } from './bindings/AppSettings';
import type { DiagnosticSession } from './bindings/DiagnosticSession';
import type { FileEntry } from './bindings/FileEntry';
import type { LiveMetrics } from './bindings/LiveMetrics';
import type { ScanEvent } from './bindings/ScanEvent';
import type { ScanRequest } from './bindings/ScanRequest';
import type { StorageQuery } from './bindings/StorageQuery';
import type { StorageMapQuery } from './bindings/StorageMapQuery';
import type { StorageMap } from './bindings/StorageMap';
import type { CleanupPlan } from './bindings/CleanupPlan';
import type { CleanupRequest } from './bindings/CleanupRequest';
import type { CleanupApproval } from './bindings/CleanupApproval';
import type { CleanupFilePage } from './bindings/CleanupFilePage';
import type { CleanupCategoryReport } from './bindings/CleanupCategoryReport';
import type { CleanupCategoryQuery } from './bindings/CleanupCategoryQuery';
import type { CleanupCategoryPage } from './bindings/CleanupCategoryPage';
import type { ProjectCleanupPage } from './bindings/ProjectCleanupPage';

import type { DeveloperReport } from './bindings/DeveloperReport';
import type { DuplicateReport } from './bindings/DuplicateReport';
import type { DuplicateAnalysisRequest } from './bindings/DuplicateAnalysisRequest';
import type { PhotoPreview } from './bindings/PhotoPreview';
import type { DuplicateCleanupRequest } from './bindings/DuplicateCleanupRequest';
import type { Baseline } from './bindings/Baseline';
import type { BaselineRequest } from './bindings/BaselineRequest';
import type { BaselineComparison } from './bindings/BaselineComparison';
import type { WorkloadTrial } from './bindings/WorkloadTrial';
import type { WorkloadTrialRequest } from './bindings/WorkloadTrialRequest';
import type { TrialComparison } from './bindings/TrialComparison';
import type { TrialComparisonRequest } from './bindings/TrialComparisonRequest';
import type { ManualPlan } from './bindings/ManualPlan';
import type { Investigation } from './bindings/Investigation';
import type { InvestigationCollection } from './bindings/InvestigationCollection';
import type { AdviceReport } from './bindings/AdviceReport';
import type { RecommendationPlan } from './bindings/RecommendationPlan';
import type { AdviceRequest } from './bindings/AdviceRequest';
import type { ResourceReview } from './bindings/ResourceReview';
import type { ActivityRecord } from './bindings/ActivityRecord';
import type { RetainedRecord } from './bindings/RetainedRecord';
import type { MaintenanceRule } from './bindings/MaintenanceRule';
import type { MaintenanceEvent } from './bindings/MaintenanceEvent';
import type { FirmwareRequest } from './bindings/FirmwareRequest';
import type { FirmwareReport } from './bindings/FirmwareReport';
import type { FirmwarePlan } from './bindings/FirmwarePlan';
import type { FirmwarePlanKind } from './bindings/FirmwarePlanKind';
import type { FirmwareCenter } from './bindings/FirmwareCenter';
import type { FirmwareComparison } from './bindings/FirmwareComparison';
import type { ScanReview } from './bindings/ScanReview';
import type { ScanModules } from './bindings/ScanModules';
import type { ScanObservations } from './bindings/ScanObservations';
import type { AppUpdatePlan } from './bindings/AppUpdatePlan';
import type { AppUpdateReport } from './bindings/AppUpdateReport';
export interface Backend {
  prepareAppUpdates(paths: string[]): Promise<AppUpdatePlan>;
  checkAppUpdates(
    planId: string,
    networkAcknowledged: boolean,
    onEvent: (report: AppUpdateReport) => void,
  ): Promise<AppUpdateReport>;
  latestAppUpdates(): Promise<AppUpdateReport | null>;
  openAppUpdate(reportId: string, path: string): Promise<void>;
  scanReview(sessionId: string): Promise<ScanReview>;
  scanObservations(id: string): Promise<ScanObservations | null>;
  administratorStatus(): Promise<HelperStatus>;
  administratorLock(): Promise<void>;
  administratorInspectApplication(path: string): Promise<ApplicationFootprint>;
  administratorPreviewRemoval(request: RemovalRequest): Promise<RemovalPlan>;
  administratorLifecycle(action: 'purge' | 'stop' | 'uninstall'): Promise<void>;
  applicationLeftovers(offset: number): Promise<ApplicationFootprint[]>;
  applicationNotices(): Promise<number>;
  diskScanPlan(): Promise<DiskScanPlan>;
  coverage(id: string, offset: number, reason?: string): Promise<CoverageGap[]>;
  refreshApplicationInventory(): Promise<void>;
  applicationPage(
    query: string,
    sort: string,
    offset: number,
    protectedOnly?: boolean,
  ): Promise<ApplicationPage>;
  inspectApplication(path: string): Promise<ApplicationFootprint>;
  applicationPreferences(): Promise<ApplicationPreferences>;
  saveApplicationPreferences(p: ApplicationPreferences): Promise<void>;
  selectApplication(): Promise<string | null>;
  previewRemoval(r: RemovalRequest): Promise<RemovalPlan>;
  previewRestore(id: string, paths: string[]): Promise<RemovalPlan>;
  removalHistory(offset: number): Promise<RemovalPlan[]>;
  removalEntries(
    id: string,
    index: number,
    offset: number,
  ): Promise<FileEntry[]>;
  executeRemoval(
    a: RemovalApproval,
    event: (p: RemovalPlan) => void,
  ): Promise<RemovalPlan>;
  firmwareCenter(): Promise<FirmwareCenter>;
  saveFirmwareReminder(enabled: boolean, days: number): Promise<FirmwareCenter>;
  dismissFirmwareNotice(id: string): Promise<FirmwareCenter>;
  compareFirmware(before: string, after: string): Promise<FirmwareComparison>;
  auditFirmware(
    request: FirmwareRequest,
    onEvent: (r: FirmwareReport) => void,
  ): Promise<FirmwareReport>;
  firmwarePlan(reportId: string, kind: FirmwarePlanKind): Promise<FirmwarePlan>;
  acknowledgeFirmwarePlan(id: string, accepted: boolean): Promise<FirmwarePlan>;
  observeFirmwarePlan(
    id: string,
    performed: boolean,
    onEvent: (r: FirmwareReport) => void,
  ): Promise<FirmwarePlan>;
  reference(url: string): Promise<void>;
  analyzeDuplicates(
    sessionId: string,
    onEvent: (r: DuplicateReport) => void,
  ): Promise<DuplicateReport>;
  analyzeDuplicateScope(request: DuplicateAnalysisRequest, onEvent: (r: DuplicateReport) => void): Promise<DuplicateReport>;
  previewDuplicates(request: DuplicateCleanupRequest): Promise<CleanupPlan>;
  previewDuplicatePhoto(reportId: string, path: string): Promise<PhotoPreview>;
  captureBaseline(
    request: BaselineRequest,
    onEvent: (r: Baseline) => void,
  ): Promise<Baseline>;
  compareBaselines(before: string, after: string): Promise<BaselineComparison>;
  captureTrial(
    request: WorkloadTrialRequest,
    onEvent: (r: WorkloadTrial) => void,
  ): Promise<WorkloadTrial>;
  finishTrial(id: string): Promise<void>;
  compareTrials(request: TrialComparisonRequest): Promise<TrialComparison>;
  saveTrialComparison(
    request: TrialComparisonRequest,
  ): Promise<TrialComparison>;
  manualPlan(kind: string, requested: string): Promise<ManualPlan>;
  verifyManualPlan(id: string, performed: boolean): Promise<ManualPlan>;
  ask(
    id: string | null,
    sessionId: string | null,
    text: string,
  ): Promise<Investigation>;
  assess(request: AdviceRequest): Promise<AdviceReport>;
  collectInvestigation(
    id: string,
    task: InvestigationCollection,
    onEvent: (r: Investigation) => void,
  ): Promise<Investigation>;
  recommendationPlan(
    sessionId: string,
    baselineId: string | null,
    trialId?: string | null,
  ): Promise<RecommendationPlan>;
  saveRecommendationPlan(
    sessionId: string,
    baselineId: string | null,
    trialId?: string | null,
  ): Promise<RecommendationPlan>;
  resources(sessionId: string): Promise<ResourceReview>;
  records(kind: string | null, offset: number): Promise<ActivityRecord[]>;
  record(kind: string, id: string): Promise<RetainedRecord>;
  maintenanceRule(): Promise<MaintenanceRule | null>;
  saveMaintenance(rule: MaintenanceRule): Promise<void>;
  maintenanceTick(): Promise<MaintenanceEvent | null>;
  cancelWork(): Promise<void>;

  developerReport(sessionId: string): Promise<DeveloperReport | null>;
  analyzeDeveloper(
    sessionId: string,
    onEvent: (report: DeveloperReport) => void,
  ): Promise<DeveloperReport>;
  cancelDeveloper(): Promise<void>;
  cacheGroups(query: CacheQuery): Promise<CacheGroupPage>;
  cacheEntries(query: CacheQuery): Promise<CacheBulkPage>;
  cacheSelect(request: CacheSelectionRequest, onEvent: (value: CacheBulkSummary) => void): Promise<CacheBulkSummary>;
  cachePrepare(id: string, revision: number, onEvent: (value: CacheBulkSummary) => void): Promise<CacheBulkSummary>;
  cacheExecute(approval: CacheBulkApproval, onEvent: (value: CacheBulkSummary) => void): Promise<CacheBulkSummary>;
  cacheBatchPage(query: CacheBulkPageQuery): Promise<CacheBulkPage>;
  cacheBatchSummary(id: string): Promise<CacheBulkSummary>;
  cacheReviewedGroups(id: string, cursor: string | null): Promise<CacheGroupPage>;
  cacheHistory(offset: number): Promise<CacheBulkSummary[]>;
  cacheDecline(id: string): Promise<CacheBulkSummary>;
  cleanupFiles(sessionId: string, offset: number): Promise<CleanupFilePage>;
  cleanupCategories(sessionId: string): Promise<CleanupCategoryReport>;
  cleanupCategoryEntries(query: CleanupCategoryQuery): Promise<CleanupCategoryPage>;
  previewCleanup(request: CleanupRequest): Promise<CleanupPlan>;
  previewProjectCleanup(request: CleanupRequest): Promise<CleanupPlan>;
  projectCleanupEntries(
    id: string,
    offset: number,
  ): Promise<ProjectCleanupPage>;
  executeCleanup(
    approval: CleanupApproval,
    onEvent: (plan: CleanupPlan) => void,
  ): Promise<CleanupPlan>;
  declineCleanup(id: string): Promise<CleanupPlan>;
  cleanupHistory(offset: number, limit: number): Promise<CleanupPlan[]>;
  cancelCleanup(): Promise<void>;
  initialize(): Promise<AppSettings>;
  save(settings: AppSettings): Promise<void>;
  history(offset: number, limit: number): Promise<DiagnosticSession[]>;
  files(query: StorageQuery): Promise<FileEntry[]>;
  storageMap(query: StorageMapQuery): Promise<StorageMap>;
  scan(
    request: ScanRequest,
    onEvent: (event: ScanEvent) => void,
    administratorPlan?: DiskScanPlan | null,
    modules?: ScanModules,
  ): Promise<void>;
  cancel(): Promise<void>;
  monitor(onEvent: (metrics: LiveMetrics) => void): Promise<void>;
  stopMonitoring(): Promise<void>;
  purge(): Promise<void>;
  folders(): Promise<string[]>;
  reveal(path: string): Promise<void>;
  settings(destination: string): Promise<void>;
  exportReport?(id: string): Promise<void>;
}
export const backend: Backend = {
  prepareAppUpdates: (paths) => invoke('prepare_app_updates', { paths }),
  checkAppUpdates: (planId, networkAcknowledged, onEvent) => {
    const channel = new Channel<AppUpdateReport>();
    channel.onmessage = onEvent;
    return invoke('check_app_updates', {
      planId,
      networkAcknowledged,
      onEvent: channel,
    });
  },
  latestAppUpdates: () => invoke('latest_app_updates'),
  openAppUpdate: (reportId, path) =>
    invoke('open_app_update', { reportId, path }),
  scanReview: (sessionId) => invoke('scan_review', { sessionId }),
  scanObservations: (id) => invoke('scan_observations', { id }),
  administratorStatus: () => invoke('administrator_status'),
  administratorLock: () => invoke('administrator_lock'),
  administratorInspectApplication: (path) =>
    invoke('administrator_inspect_application', { path }),
  administratorPreviewRemoval: (request) =>
    invoke('administrator_preview_removal', { request }),
  administratorLifecycle: (action) =>
    invoke('administrator_lifecycle', { action }),
  applicationLeftovers: (offset) => invoke('application_leftovers', { offset }),
  applicationNotices: () => invoke('application_notices'),
  diskScanPlan: () => invoke('disk_scan_plan'),
  coverage: (id, offset, reason) =>
    invoke('scan_coverage', { id, offset, reason }),
  refreshApplicationInventory: () => invoke('refresh_application_inventory'),
  applicationPage: (query, sort, offset, protectedOnly = false) =>
    invoke('application_page', { query, sort, offset, protectedOnly }),
  inspectApplication: (path) => invoke('inspect_application', { path }),
  applicationPreferences: () => invoke('application_preferences'),
  saveApplicationPreferences: (p) =>
    invoke('save_application_preferences', { p }),
  selectApplication: async () => {
    const p = await open({
      multiple: false,
      directory: false,
      title: 'Choose a macOS application',
      filters: [{ name: 'macOS application', extensions: ['app'] }],
    });
    return typeof p === 'string' ? p : null;
  },
  previewRemoval: (r) => invoke('preview_removal', { r }),
  previewRestore: (id, paths) => invoke('preview_restore', { id, paths }),
  removalHistory: (offset) => invoke('removal_history', { offset }),
  removalEntries: (id, index, offset) =>
    invoke('removal_entries', { id, index, offset }),
  executeRemoval: (a, onEvent) => {
    const channel = new Channel<RemovalPlan>();
    channel.onmessage = onEvent;
    return invoke('execute_removal', { a, onEvent: channel });
  },
  firmwareCenter: () => invoke('firmware_center'),
  saveFirmwareReminder: (enabled, days) =>
    invoke('save_firmware_reminder', { enabled, days }),
  dismissFirmwareNotice: (id) => invoke('dismiss_firmware_notice', { id }),
  compareFirmware: (before, after) =>
    invoke('compare_firmware', { before, after }),
  auditFirmware: (request, onEvent) => {
    const channel = new Channel<FirmwareReport>();
    channel.onmessage = onEvent;
    return invoke('audit_firmware', { request, onEvent: channel });
  },
  firmwarePlan: (reportId, kind) => invoke('firmware_plan', { reportId, kind }),
  acknowledgeFirmwarePlan: (id, accepted) =>
    invoke('acknowledge_firmware_plan', { id, accepted }),
  observeFirmwarePlan: (id, performed, onEvent) => {
    const channel = new Channel<FirmwareReport>();
    channel.onmessage = onEvent;
    return invoke('observe_firmware_plan', { id, performed, onEvent: channel });
  },
  reference: (url) => invoke('open_reference', { url }),
  analyzeDuplicates: (sessionId, onEvent) => {
    const channel = new Channel<DuplicateReport>();
    channel.onmessage = onEvent;
    return invoke('analyze_duplicates', { sessionId, onEvent: channel });
  },
  analyzeDuplicateScope: (request, onEvent) => {
    const channel = new Channel<DuplicateReport>();
    channel.onmessage = onEvent;
    return invoke('analyze_duplicate_scope', { request, onEvent: channel });
  },
  previewDuplicates: (request) =>
    invoke('preview_duplicate_cleanup', { request }),
  captureBaseline: (request, onEvent) => {
    const channel = new Channel<Baseline>();
    channel.onmessage = onEvent;
    return invoke('capture_baseline', { request, onEvent: channel });
  },
  compareBaselines: (before, after) =>
    invoke('compare_baselines', { before, after }),
  captureTrial: (request, onEvent) => {
    const channel = new Channel<WorkloadTrial>();
    channel.onmessage = onEvent;
    return invoke('capture_workload_trial', { request, onEvent: channel });
  },
  finishTrial: (id) => invoke('finish_workload_trial', { id }),
  compareTrials: (request) => invoke('compare_workload_trials', { request }),
  saveTrialComparison: (request) =>
    invoke('save_trial_comparison', { request }),
  manualPlan: (kind, requested) =>
    invoke('create_manual_plan', { kind, requested }),
  verifyManualPlan: (id, performed) =>
    invoke('verify_manual_plan', { id, performed }),
  ask: (id, sessionId, text) => invoke('ask_coqui', { id, sessionId, text }),
  assess: (request) => invoke('assess_workload', { request }),
  previewDuplicatePhoto: (reportId, path) =>
    invoke('preview_duplicate_photo', { reportId, path }),
  collectInvestigation: (id, task, onEvent) => {
    const channel = new Channel<Investigation>();
    channel.onmessage = onEvent;
    return invoke('collect_investigation', { id, task, onEvent: channel });
  },
  recommendationPlan: (sessionId, baselineId, trialId = null) =>
    invoke('recommendation_plan', { sessionId, baselineId, trialId }),
  saveRecommendationPlan: (sessionId, baselineId, trialId = null) =>
    invoke('save_recommendation_plan', { sessionId, baselineId, trialId }),
  resources: (sessionId) => invoke('analyze_resources', { sessionId }),
  records: (kind, offset) => invoke('activity_records', { kind, offset }),
  record: (kind, id) => invoke('get_record', { kind, id }),
  maintenanceRule: () => invoke('maintenance_rule'),
  saveMaintenance: (rule) => invoke('save_maintenance', { rule }),
  maintenanceTick: () => invoke('maintenance_tick'),
  cancelWork: () => invoke('cancel_work'),

  developerReport: (sessionId) => invoke('developer_report', { sessionId }),
  analyzeDeveloper: (sessionId, onEvent) => {
    const channel = new Channel<DeveloperReport>();
    channel.onmessage = onEvent;
    return invoke('analyze_developer', { sessionId, onEvent: channel });
  },
  cancelDeveloper: () => invoke('cancel_developer'),
  cacheGroups: (query) => invoke('cache_groups', { query }),
  cacheEntries: (query) => invoke('cache_entries', { query }),
  cacheSelect: (request, onEvent) => { const channel = new Channel<CacheBulkSummary>(); channel.onmessage = onEvent; return invoke('cache_select', { request, onEvent: channel }); },
  cachePrepare: (id, revision, onEvent) => { const channel = new Channel<CacheBulkSummary>(); channel.onmessage = onEvent; return invoke('cache_prepare', { id, revision, onEvent: channel }); },
  cacheExecute: (approval, onEvent) => { const channel = new Channel<CacheBulkSummary>(); channel.onmessage = onEvent; return invoke('cache_execute', { approval, onEvent: channel }); },
  cacheBatchPage: (query) => invoke('cache_batch_page', { query }),
  cacheBatchSummary: (id) => invoke('cache_batch_summary', { id }),
  cacheReviewedGroups: (id, cursor) => invoke('cache_reviewed_groups', { id, cursor }),
  cacheHistory: (offset) => invoke('cache_history', { offset }),
  cacheDecline: (id) => invoke('cache_decline', { id }),
  cleanupFiles: (sessionId, offset) =>
    invoke('cleanup_files', { sessionId, offset }),
  cleanupCategories: (sessionId) => invoke('cleanup_categories', { sessionId }),
  cleanupCategoryEntries: (query) => invoke('cleanup_category_entries', { query }),
  previewCleanup: (request) => invoke('preview_cleanup', { request }),
  previewProjectCleanup: (request) =>
    invoke('preview_project_cleanup', { request }),
  projectCleanupEntries: (id, offset) =>
    invoke('project_cleanup_entries', { id, offset }),
  executeCleanup: (approval, onEvent) => {
    const channel = new Channel<CleanupPlan>();
    channel.onmessage = onEvent;
    return invoke('execute_cleanup', { approval, onEvent: channel });
  },
  declineCleanup: (id) => invoke('decline_cleanup', { id }),
  cleanupHistory: (offset, limit) =>
    invoke('cleanup_history', { offset, limit }),
  cancelCleanup: () => invoke('cancel_cleanup'),
  initialize: async () => {
    if (!isTauri())
      throw new Error(
        'Open the CoquiPCCleaner desktop app to access local diagnostics. Browser previews cannot scan your Mac.',
      );
    return invoke('initialize');
  },
  save: (settings) => invoke('update_settings', { settings }),
  history: (offset, limit) => invoke('scan_history', { offset, limit }),
  files: (query) => invoke('storage_entries', { query }),
  storageMap: (query) => invoke('storage_map', { query }),
  scan: (request, onEvent, administratorPlan, modules) => {
    const channel = new Channel<ScanEvent>();
    channel.onmessage = onEvent;
    return administratorPlan
      ? invoke(
          modules ? 'administrator_scan_with_modules' : 'administrator_scan',
          {
            request,
            plan: administratorPlan,
            events: channel,
            ...(modules ? { modules } : {}),
          },
        )
      : invoke(modules ? 'scan_with_modules' : 'scan', {
          request,
          onEvent: channel,
          ...(modules ? { modules } : {}),
        });
  },
  cancel: () => invoke('cancel_scan'),
  monitor: (onEvent) => {
    const channel = new Channel<LiveMetrics>();
    channel.onmessage = onEvent;
    return invoke('monitor', { onEvent: channel });
  },
  stopMonitoring: () => invoke('stop_monitoring'),
  purge: () => invoke('clear_diagnostic_data'),
  folders: async () => {
    const result = await open({
      directory: true,
      multiple: true,
      title: 'Choose locations for storage analysis',
    });
    return result === null ? [] : Array.isArray(result) ? result : [result];
  },
  reveal: (path) => invoke('reveal_path', { path }),
  settings: (destination) => invoke('open_system_settings', { destination }),
  exportReport: async (id) => {
    const path = await save({
      title: 'Export diagnostic report',
      defaultPath: `Coqui-analysis-${id}.json`,
      filters: [{ name: 'JSON report', extensions: ['json'] }],
    });
    if (path) await invoke('export_report', { id, path });
  },
};
