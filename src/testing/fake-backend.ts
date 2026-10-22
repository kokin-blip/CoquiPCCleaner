import { vi } from 'vitest';
import type { Backend } from '../backend';
import type { AppSettings } from '../bindings/AppSettings';
import type { DiagnosticSession } from '../bindings/DiagnosticSession';
import type { LiveMetrics } from '../bindings/LiveMetrics';
export const metric: LiveMetrics = {
  observedAt: new Date().toISOString(),
  cpuPercent: 12,
  usedMemoryBytes: 100,
  totalMemoryBytes: 200,
  swapBytes: 0,
  diskReadBytes: 0,
  diskWrittenBytes: 0,
  processes: [],
  qualification: 'fixture',
};
export const settings: AppSettings = {
  onboardingComplete: true,
  workload: { workloads: ['Software development'], preference: 'Balanced' },
  appearance: 'system',
  scanMode: 'selected',
  excludedPaths: [],
  selectedRoots: ['/fixture'],
  retentionDays: 30,
  monitoringEnabled: false,
  advancedDiagnostics: false,
};
export const session: DiagnosticSession = {
  id: 'fixture',
  startedAt: new Date().toISOString(),
  finishedAt: null,
  status: 'running',
  stage: 'Storage metadata',
  scope: '/fixture',
  fileCount: 0,
  logicalBytes: 0,
  allocatedBytes: 0,
  skippedCount: 0,
  snapshot: null,
  findings: [],
  permissions: [],
  categories: [],
  candidates: [],
  errors: [],
};
export function fakeBackend(overrides: Partial<Backend> = {}): Backend {
  return {
    developerReport: async () => null,
    analyzeDeveloper: vi.fn(async (sessionId, on) => {
      const report = {
        sessionId,
        checkedAt: '',
        status: 'complete',
        examined: 0,
        candidateCount: 0,
        artifacts: [],
        errors: [],
        qualification: 'Fixture',
      };
      on(report);
      return report;
    }),
    cancelDeveloper: vi.fn(async () => {}),
    cleanupFiles: vi.fn(async () => ({
      files: [],
      nextOffset: null,
      unavailableCount: 0,
    })),
    previewCleanup: vi.fn(async () => {
      throw new Error('No fixture plan configured');
    }),
    previewProjectCleanup: vi.fn(async () => {
      throw new Error('No project fixture plan configured');
    }),
    projectCleanupEntries: vi.fn(async () => ({ entries: [], total: 0 })),
    executeCleanup: vi.fn(async () => {
      throw new Error('No fixture execution configured');
    }),
    declineCleanup: vi.fn(async () => {
      throw new Error('No fixture plan configured');
    }),
    cleanupHistory: vi.fn(async () => []),
    cancelCleanup: vi.fn(async () => {}),
    initialize: vi.fn(async () => structuredClone(settings)),
    save: vi.fn(async () => {}),
    history: vi.fn(async () => []),
    files: vi.fn(async () => []),
    scan: vi.fn(async (_r, on) =>
      on({
        session: {
          ...session,
          status: 'complete',
          finishedAt: new Date().toISOString(),
        },
      }),
    ),
    cancel: vi.fn(async () => {}),
    monitor: vi.fn(async () => {}),
    stopMonitoring: vi.fn(async () => {}),
    purge: vi.fn(async () => {}),
    folders: vi.fn(async () => []),
    reveal: vi.fn(async () => {}),
    settings: vi.fn(async () => {}),
    ...overrides,
  };
}
