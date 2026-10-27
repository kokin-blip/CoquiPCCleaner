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
  administratorEnabled: false,
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
  coverage: [],
  volumePlan: null,
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
export const firmwareCenter = {
  preferences: {
    reminderEnabled: false,
    intervalDays: 7,
    nextDueAt: null,
    dismissedReportId: null,
  },
  notices: [],
  latestAttemptAt: null,
  latestAttemptState: 'notRequested',
  qualification: 'Local fixture; no network check.',
};
export function fakeBackend(overrides: Partial<Backend> = {}): Backend {
  const missing = async (): Promise<never> => {
    throw new Error('No phase 2 fixture configured');
  };
  return {
    prepareAppUpdates: vi.fn(missing),
    checkAppUpdates: vi.fn(missing),
    latestAppUpdates: vi.fn(async () => null),
    scanObservations: vi.fn(async () => null),
    openAppUpdate: vi.fn(missing),
    scanReview: vi.fn(async (sessionId) => ({
      sessionId,
      scanStatus: 'partial',
      observedAt: null,
      groups: [],
    })),
    administratorLock: vi.fn(async () => {}),
    administratorStatus: vi.fn(async () => ({
      authorizationSeconds: 0,
      installed: false,
      compatible: false,
      version: '',
      protocol: 0,
      qualification: 'Test fixture: helper absent',
    })),
    administratorInspectApplication: vi.fn(),
    administratorPreviewRemoval: vi.fn(),
    administratorLifecycle: vi.fn(async () => {}),
    applicationLeftovers: vi.fn(async () => []),
    applicationNotices: vi.fn(async () => 0),
    diskScanPlan: vi.fn(missing),
    coverage: vi.fn(async () => []),
    refreshApplicationInventory: vi.fn(async () => {}),
    applicationPage: vi.fn(async () => ({
      applications: [],
      offset: 0,
      total: 0,
      qualification: 'Fixture',
    })),
    inspectApplication: vi.fn(missing),
    applicationPreferences: vi.fn(async () => ({
      protectedPaths: [],
      disappearanceNotices: false,
    })),
    saveApplicationPreferences: vi.fn(async () => {}),
    selectApplication: vi.fn(async () => null),
    previewRemoval: vi.fn(missing),
    previewRestore: vi.fn(missing),
    removalHistory: vi.fn(async () => []),
    removalEntries: vi.fn(async () => []),
    executeRemoval: vi.fn(missing),
    auditFirmware: vi.fn(missing),
    firmwarePlan: vi.fn(missing),
    acknowledgeFirmwarePlan: vi.fn(missing),
    observeFirmwarePlan: vi.fn(missing),
    firmwareCenter: vi.fn(async () => structuredClone(firmwareCenter)),
    saveFirmwareReminder: vi.fn(missing),
    dismissFirmwareNotice: vi.fn(missing),
    compareFirmware: vi.fn(missing),
    reference: vi.fn(async () => {}),
    analyzeDuplicates: vi.fn(missing),
    analyzeDuplicateScope: vi.fn(missing),
    previewDuplicates: vi.fn(missing),
    previewDuplicatePhoto: vi.fn(missing),
    captureBaseline: vi.fn(missing),
    captureTrial: vi.fn(missing),
    finishTrial: vi.fn(missing),
    compareTrials: vi.fn(missing),
    saveTrialComparison: vi.fn(missing),
    compareBaselines: vi.fn(missing),
    manualPlan: vi.fn(missing),
    verifyManualPlan: vi.fn(missing),
    ask: vi.fn(missing),
    collectInvestigation: vi.fn(missing),
    assess: vi.fn(missing),
    recommendationPlan: vi.fn(
      async (sessionId, baselineId, trialId = null) => ({
        id: 'fixture-plan',
        createdAt: '',
        sessionId,
        baselineId,
        trialId,
        preference: 'Balanced',
        actions: [],
        workloads: [],
        bottlenecks: [],
        qualification: 'Illustrative test fixture, not machine evidence.',
      }),
    ),
    saveRecommendationPlan: vi.fn(missing),
    resources: vi.fn(missing),
    records: vi.fn(async () => []),
    record: vi.fn(missing),
    maintenanceRule: vi.fn(async () => null),
    saveMaintenance: vi.fn(async () => {}),
    maintenanceTick: vi.fn(async () => null),
    cancelWork: vi.fn(async () => {}),
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
    cleanupCategories: vi.fn(async (sessionId) => ({sessionId,observedAt:null,sourceStatus:'partial',rulesReviewedAt:'2026-10-27',categories:[],qualification:'Illustrative test fixture; no measured category totals.'})),
    cleanupCategoryEntries: vi.fn(async (query) => ({sessionId:query.sessionId,kind:query.kind,files:[],nextOffset:null,qualification:'Illustrative test fixture'})),
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
    storageMap: vi.fn(async () => ({
      nodes: [],
      totalAllocatedBytes: 0,
      totalLogicalBytes: 0,
      fileCount: null,
      remainderAllocatedBytes: 0,
      remainderLogicalBytes: 0,
      remainderFileCount: null,
      qualification: 'No folder aggregate fixture configured.',
    })),
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
