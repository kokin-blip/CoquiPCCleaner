import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { AppSettings } from './bindings/AppSettings';
import type { DiagnosticSession } from './bindings/DiagnosticSession';
import type { FileEntry } from './bindings/FileEntry';
import type { LiveMetrics } from './bindings/LiveMetrics';
import type { ScanEvent } from './bindings/ScanEvent';
import type { ScanRequest } from './bindings/ScanRequest';
import type { StorageQuery } from './bindings/StorageQuery';
import type { CleanupPlan } from './bindings/CleanupPlan';
import type { CleanupRequest } from './bindings/CleanupRequest';
import type { CleanupApproval } from './bindings/CleanupApproval';
import type { CleanupFilePage } from './bindings/CleanupFilePage';
import type { ProjectCleanupPage } from './bindings/ProjectCleanupPage';

import type { DeveloperReport } from './bindings/DeveloperReport';
import type { DuplicateReport } from './bindings/DuplicateReport';
import type { DuplicateCleanupRequest } from './bindings/DuplicateCleanupRequest';
import type { Baseline } from './bindings/Baseline';
import type { BaselineRequest } from './bindings/BaselineRequest';
import type { BaselineComparison } from './bindings/BaselineComparison';
import type { ManualPlan } from './bindings/ManualPlan';
import type { Investigation } from './bindings/Investigation';
import type { AdviceReport } from './bindings/AdviceReport';
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
export interface Backend {
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
  previewDuplicates(request: DuplicateCleanupRequest): Promise<CleanupPlan>;
  captureBaseline(
    request: BaselineRequest,
    onEvent: (r: Baseline) => void,
  ): Promise<Baseline>;
  compareBaselines(before: string, after: string): Promise<BaselineComparison>;
  manualPlan(kind: string, requested: string): Promise<ManualPlan>;
  verifyManualPlan(id: string, performed: boolean): Promise<ManualPlan>;
  ask(
    id: string | null,
    sessionId: string | null,
    text: string,
  ): Promise<Investigation>;
  assess(request: AdviceRequest): Promise<AdviceReport>;
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
  cleanupFiles(sessionId: string, offset: number): Promise<CleanupFilePage>;
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
  scan(
    request: ScanRequest,
    onEvent: (event: ScanEvent) => void,
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
  previewDuplicates: (request) =>
    invoke('preview_duplicate_cleanup', { request }),
  captureBaseline: (request, onEvent) => {
    const channel = new Channel<Baseline>();
    channel.onmessage = onEvent;
    return invoke('capture_baseline', { request, onEvent: channel });
  },
  compareBaselines: (before, after) =>
    invoke('compare_baselines', { before, after }),
  manualPlan: (kind, requested) =>
    invoke('create_manual_plan', { kind, requested }),
  verifyManualPlan: (id, performed) =>
    invoke('verify_manual_plan', { id, performed }),
  ask: (id, sessionId, text) => invoke('ask_coqui', { id, sessionId, text }),
  assess: (request) => invoke('assess_workload', { request }),
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
  cleanupFiles: (sessionId, offset) =>
    invoke('cleanup_files', { sessionId, offset }),
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
  scan: (request, onEvent) => {
    const channel = new Channel<ScanEvent>();
    channel.onmessage = onEvent;
    return invoke('scan', { request, onEvent: channel });
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
