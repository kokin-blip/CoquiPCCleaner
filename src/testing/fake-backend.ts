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
