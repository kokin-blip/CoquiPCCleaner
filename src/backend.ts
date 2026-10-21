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

export interface Backend {
  cleanupFiles(sessionId: string, offset: number): Promise<CleanupFilePage>;
  previewCleanup(request: CleanupRequest): Promise<CleanupPlan>;
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
  cleanupFiles: (sessionId, offset) =>
    invoke('cleanup_files', { sessionId, offset }),
  previewCleanup: (request) => invoke('preview_cleanup', { request }),
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
