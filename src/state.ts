import type { CacheBulkSummary } from './bindings/CacheBulkSummary';
import type { CacheQuery } from './bindings/CacheQuery';
import type { CacheBulkApproval } from './bindings/CacheBulkApproval';
import type { DiskScanPlan } from './bindings/DiskScanPlan';
import type { ScanModules } from './bindings/ScanModules';
import { get, writable } from 'svelte/store';
import { backend as nativeBackend, type Backend } from './backend';
import type { AppSettings } from './bindings/AppSettings';
import type { DiagnosticSession } from './bindings/DiagnosticSession';
import type { LiveMetrics } from './bindings/LiveMetrics';
import type { DeveloperReport } from './bindings/DeveloperReport';
import type { CleanupPlan } from './bindings/CleanupPlan';
import type { CleanupApproval } from './bindings/CleanupApproval';
import type { CleanupRequest } from './bindings/CleanupRequest';

import type { FirmwareCenter } from './bindings/FirmwareCenter';
import type { WorkloadTrial } from './bindings/WorkloadTrial';
import type { WorkloadTrialRequest } from './bindings/WorkloadTrialRequest';
export type Page =
  | 'Ask Coqui'
  | 'Maintenance'
  | 'Dashboard'
  | 'System Analysis'
  | 'Hardware'
  | 'Storage'
  | 'Applications'
  | 'Optimization'
  | 'Performance'
  | 'Firmware & Security'
  | 'History'
  | 'Settings';
export interface AppState {
  overviewMode: 'start' | 'results';
  workloadTrial: WorkloadTrial | null;
  trialFinishing: boolean;
  askDraft: string;
  reviewTarget: string | null;
  missingApplications: number;
  firmwareCenter: FirmwareCenter | null;
  firmwareNoticeError: string | null;
  firmwareReportId: string | null;
  firmwareRequest: number;
  loading: boolean;
  settings: AppSettings | null;
  session: DiagnosticSession | null;
  history: DiagnosticSession[];
  samples: LiveMetrics[];
  monitoringStatus: 'off' | 'starting' | 'live' | 'failed';
  monitoringError: string | null;
  workRunning: boolean;
  workLabel: string;
  scanning: boolean;
  developerRunning: boolean;
  developerReport: DeveloperReport | null;
  cleanupRunning: boolean;
  cleanupPreparing: boolean;
  cleanupPlan: CleanupPlan | null;
  cacheBatch: CacheBulkSummary | null;
  cacheSourceKey: string;
  cancelling: boolean;
  error: string | null;
  notice: string | null;
  page: Page;
}
export function createAppStore(service: Backend = nativeBackend) {
  const store = writable<AppState>({
    overviewMode: 'start',
    workloadTrial: null,
    trialFinishing: false,
    askDraft: '',
    reviewTarget: null,
    missingApplications: 0,
    firmwareCenter: null,
    firmwareNoticeError: null,
    firmwareReportId: null,
    firmwareRequest: 0,
    loading: true,
    settings: null,
    session: null,
    history: [],
    samples: [],
    monitoringStatus: 'off',
    monitoringError: null,
    workRunning: false,
    workLabel: '',
    scanning: false,
    developerRunning: false,
    developerReport: null,
    cleanupRunning: false,
    cleanupPreparing: false,
    cleanupPlan: null,
    cacheBatch: null,
    cacheSourceKey: '',
    cancelling: false,
    error: null,
    notice: null,
    page: 'Dashboard',
  });
  // Selection belongs to its source even while Cleanup is not mounted.
  const stopCacheSource = store.subscribe((s) => {
    const key = `${s.session?.id ?? ''}:${JSON.stringify(s.settings?.excludedPaths ?? [])}`;
    if (s.cacheSourceKey !== key) store.update((v) => ({ ...v, cacheSourceKey: key, cacheBatch: null }));
  });
  let monitoring: Promise<void> | null = null;
  let disposed = false;
  let firmwareRevision = 0;
  let maintenanceTimer: ReturnType<typeof setInterval> | null = null;
  let saves: Promise<void> = Promise.resolve();
  const fail = (e: unknown) =>
    store.update((s) => ({
      ...s,
      error: String(e instanceof Error ? e.message : e),
    }));
  async function refresh() {
    const history = await service.history(0, 100);
    store.update((s) => ({ ...s, history }));
  }
  function startMonitoring() {
    if (monitoring || disposed || !get(store).settings?.monitoringEnabled)
      return;
    store.update((s) => ({
      ...s,
      monitoringStatus: 'starting',
      monitoringError: null,
    }));
    const ended = (error: unknown) => {
      const wanted = !disposed && get(store).settings?.monitoringEnabled;
      store.update((s) => ({
        ...s,
        samples: [],
        monitoringStatus: wanted ? 'failed' : 'off',
        monitoringError: wanted
          ? String(error instanceof Error ? error.message : error)
          : null,
      }));
    };
    monitoring = service
      .monitor((m) =>
        store.update((s) =>
          disposed || !s.settings?.monitoringEnabled
            ? s
            : {
                ...s,
                monitoringStatus: 'live',
                samples: [...s.samples.slice(-449), m],
              },
        ),
      )
      .then(() =>
        ended(
          'The monitoring stream ended. Retry to collect new measurements.',
        ),
      )
      .catch(ended)
      .finally(() => {
        monitoring = null;
      });
  }
  function setFirmwareCenter(center: FirmwareCenter) {
    firmwareRevision++;
    store.update((s) => ({
      ...s,
      firmwareCenter: center,
      firmwareNoticeError: null,
    }));
  }
  async function refreshFirmwareCenter() {
    const revision = ++firmwareRevision;
    try {
      const center = await service.firmwareCenter();
      if (!disposed && revision === firmwareRevision)
        store.update((s) => ({
          ...s,
          firmwareCenter: center,
          firmwareNoticeError: null,
        }));
    } catch (e) {
      if (!disposed && revision === firmwareRevision)
        store.update((s) => ({ ...s, firmwareNoticeError: String(e) }));
    }
  }
  async function initialize() {
    store.update((s) => ({ ...s, loading: true, error: null }));
    try {
      const settings = await service.initialize();
      const history = await service.history(0, 100);
      store.update((s) => ({
        ...s,
        loading: false,
        settings,
        history,
        session: history[0] ?? null,
      }));
      if (!maintenanceTimer) {
        maintenanceTimer = setInterval(() => {
          void checkMaintenance();
          void refreshFirmwareCenter();
        }, 30000);
        void checkMaintenance();
      }
      const missingApplications = await service.applicationNotices();
      store.update((s) => ({ ...s, missingApplications }));
      await refreshFirmwareCenter();
      if (settings.onboardingComplete && settings.monitoringEnabled)
        startMonitoring();
    } catch (e) {
      store.update((s) => ({ ...s, loading: false }));
      fail(e);
    }
  }
  async function save(settings: AppSettings) {
    const previous = get(store).settings;
    store.update((s) => ({ ...s, settings }));
    const job = saves.catch(() => {}).then(() => service.save(settings));
    saves = job;
    try {
      await job;
    } catch (e) {
      store.update((s) =>
        s.settings === settings ? { ...s, settings: previous } : s,
      );
      throw e;
    }
    store.update((s) => ({
      ...s,
      notice: 'Preferences saved. No system settings were changed.',
    }));
    const latest = get(store).settings;
    if (latest?.onboardingComplete && latest.monitoringEnabled)
      startMonitoring();
    else {
      store.update((s) => ({
        ...s,
        samples: [],
        monitoringStatus: 'off',
        monitoringError: null,
      }));
      if (monitoring) {
        await service.stopMonitoring();
        await monitoring;
        if (get(store).settings?.monitoringEnabled) startMonitoring();
      }
    }
  }

  async function scan(
    planId: string | null = null,
    roots?: string[],
    administratorPlan: DiskScanPlan | null = null,
    modules?: ScanModules,
  ) {
    const s = get(store);
    if (!s.settings || s.scanning) return;
    if (
      s.workRunning ||
      s.cleanupRunning ||
      s.cleanupPreparing ||
      s.developerRunning
    ) {
      fail('Wait for the active operation to finish before scanning.');
      return;
    }
    let receivedEvent = false;
    store.update((s) => ({
      ...s,
      scanning: true,
      overviewMode: 'results',
      session: null,
      cancelling: false,
      error: null,
      notice: null,
    }));
    try {
      await service.scan(
        {
          mode: s.settings.scanMode,
          planId,
          roots: roots ?? s.settings.selectedRoots,
          excludedPaths: s.settings.excludedPaths,
        },
        (e) => {
          receivedEvent = true;
          store.update((s) => ({ ...s, session: e.session }));
        },
        administratorPlan,
        modules,
      );
      await refresh();
    } catch (e) {
      if (!receivedEvent) store.update((v) => ({ ...v, session: s.session }));
      fail(e);
    } finally {
      store.update((s) => ({ ...s, scanning: false, cancelling: false }));
    }
  }
  async function cancel() {
    store.update((s) => ({ ...s, cancelling: true }));
    try {
      await service.cancel();
    } catch (e) {
      store.update((s) => ({ ...s, cancelling: false }));
      fail(e);
    }
  }
  async function purge() {
    await service.purge();
    store.update((s) => ({
      ...s,
      session: null,
      history: [],
      cacheBatch: null,
      cleanupPlan: null,
      firmwareCenter: null,
      firmwareReportId: null,
      developerReport: null,
      samples: [],
      notice:
        'Local diagnostic records and evidence notices removed. Files, preferences, maintenance and firmware review schedules were preserved; disable schedules separately.',
    }));
    await refreshFirmwareCenter();
  }
  async function executeCleanup(approval: CleanupApproval) {
    if (
      get(store).workRunning ||
      get(store).cleanupRunning ||
      get(store).cleanupPreparing ||
      get(store).scanning ||
      get(store).developerRunning
    )
      throw new Error('Wait for the active operation to finish.');
    store.update((s) => ({ ...s, cleanupRunning: true, error: null }));
    try {
      const plan = await service.executeCleanup(approval, (cleanupPlan) =>
        store.update((s) => ({ ...s, cleanupPlan })),
      );
      store.update((s) => ({ ...s, cleanupPlan: plan }));
      return plan;
    } catch (e) {
      // The command may have moved a file before a transport failure. Clear
      // any stale approval and direct the user to the persisted action record.
      store.update((s) => ({ ...s, cleanupPlan: null }));
      throw e;
    } finally {
      store.update((s) => ({ ...s, cleanupRunning: false }));
    }
  }
  async function cacheOperation(executing: boolean, action: (on: (value: CacheBulkSummary) => void) => Promise<CacheBulkSummary>) {
    const s = get(store);
    if (s.workRunning || s.scanning || s.cleanupRunning || s.cleanupPreparing || s.developerRunning) throw new Error('Wait for the active operation to finish.');
    const key = s.cacheSourceKey;
    store.update((v) => ({...v,cleanupPreparing:!executing,cleanupRunning:executing,error:null}));
    const on = (cacheBatch: CacheBulkSummary) => store.update((v) => v.cacheSourceKey === key ? {...v,cacheBatch} : v);
    try { const result = await action(on); on(result); return result; }
    catch (e) {
      // An uncertain command can have persisted intent. Never retain an executable
      // approval after transport failure; use the read-only history to inspect it.
      store.update((v) => ({...v,cacheBatch:null})); throw executing ? new Error(`${String(e)} Open Action history and inspect the last cache record before retrying.`) : e;
    } finally { store.update((v) => ({...v,cleanupPreparing:false,cleanupRunning:false})); }
  }
  async function changeCacheSelection(query: CacheQuery, paths: string[] | null, selected: boolean, clear = false) {
    const batch = get(store).cacheBatch;
    const draft = batch?.status === 'draft' ? batch : null;
    return cacheOperation(false, (on) => service.cacheSelect({query:{...query,batchId:draft?.id ?? null},revision:draft?.revision ?? null,paths,selected,clear},on));
  }
  async function prepareCacheBulk() {
    const batch = get(store).cacheBatch;
    if (!batch || batch.status !== 'draft') return;
    return cacheOperation(false,(on) => service.cachePrepare(batch.id,batch.revision,on));
  }
  async function executeCacheBulk(approval: CacheBulkApproval) {
    // Clear the displayed approval immediately; execution events replace it.
    return cacheOperation(true,(on) => service.cacheExecute(approval,on));
  }
  async function keepCacheBulk() {
    const batch = get(store).cacheBatch;
    if (!batch) return;
    const result = await service.cacheDecline(batch.id);
    store.update((v) => ({...v,cacheBatch:result}));
  }
  function newCacheSelection() { store.update((v) => ({...v,cacheBatch:null})); }
  async function previewProjectCleanup(request: CleanupRequest) {
    const s = get(store);
    if (
      s.workRunning ||
      s.scanning ||
      s.cleanupRunning ||
      s.cleanupPreparing ||
      s.developerRunning
    )
      throw new Error('Wait for the active operation to finish.');
    store.update((v) => ({ ...v, cleanupPreparing: true, error: null }));
    try {
      return await service.previewProjectCleanup(request);
    } finally {
      store.update((v) => ({ ...v, cleanupPreparing: false }));
    }
  }
  async function analyzeDeveloper() {
    const s = get(store);
    if (
      !s.session ||
      s.workRunning ||
      s.scanning ||
      s.cleanupRunning ||
      s.cleanupPreparing ||
      s.developerRunning
    )
      return;
    store.update((v) => ({
      ...v,
      developerRunning: true,
      error: null,
      developerReport: null,
    }));
    try {
      const report = await service.analyzeDeveloper(
        s.session.id,
        (developerReport) => store.update((v) => ({ ...v, developerReport })),
      );
      store.update((v) => ({ ...v, developerReport: report }));
    } catch (e) {
      fail(e);
    } finally {
      store.update((v) => ({ ...v, developerRunning: false }));
    }
  }
  async function work<T>(label: string, action: () => Promise<T>): Promise<T> {
    const s = get(store);
    if (
      s.workRunning ||
      s.scanning ||
      s.cleanupRunning ||
      s.cleanupPreparing ||
      s.developerRunning
    )
      throw new Error('Wait for the active operation');
    store.update((s) => ({ ...s, workRunning: true, workLabel: label }));
    try {
      return await action();
    } finally {
      store.update((s) => ({ ...s, workRunning: false, workLabel: '' }));
    }
  }
  async function captureTrial(request: WorkloadTrialRequest) {
    return work('Timed task trial', async () => {
      store.update((s) => ({
        ...s,
        workloadTrial: null,
        trialFinishing: false,
      }));
      try {
        const trial = await service.captureTrial(request, (workloadTrial) =>
          store.update((s) => ({ ...s, workloadTrial })),
        );
        store.update((s) => ({ ...s, workloadTrial: trial }));
        return trial;
      } catch (e) {
        await service.cancelWork().catch(() => {});
        store.update((s) => ({
          ...s,
          workloadTrial:
            s.workloadTrial?.status === 'running'
              ? {
                  ...s.workloadTrial,
                  status: 'unverified',
                  qualification:
                    'The capture connection ended without a verified terminal result. Reload retained history before using this interval.',
                }
              : s.workloadTrial,
        }));
        throw e;
      } finally {
        store.update((s) => ({ ...s, trialFinishing: false }));
      }
    });
  }
  async function finishTrial() {
    const s = get(store);
    if (
      !s.workRunning ||
      s.workloadTrial?.status !== 'running' ||
      s.trialFinishing
    )
      throw new Error('No task timer is waiting for completion');
    store.update((v) => ({ ...v, trialFinishing: true }));
    try {
      await service.finishTrial(s.workloadTrial.id);
    } catch (e) {
      store.update((v) => ({ ...v, trialFinishing: false }));
      throw e;
    }
  }
  async function checkMaintenance() {
    if (
      disposed ||
      get(store).loading ||
      !get(store).settings?.onboardingComplete
    )
      return;
    const s = get(store);
    if (
      s.workRunning ||
      s.scanning ||
      s.cleanupRunning ||
      s.cleanupPreparing ||
      s.developerRunning
    )
      return;
    try {
      const rule = await service.maintenanceRule();
      if (!rule?.enabled || Date.parse(rule.nextDueAt) > Date.now()) return;
      const event = await work('Scheduled maintenance', () =>
        service.maintenanceTick(),
      );
      if (event) {
        store.update((s) => ({ ...s, notice: event.message }));
        await refresh();
      }
    } catch (e) {
      fail(e);
    }
  }
  async function run(action: () => Promise<unknown>) {
    try {
      await action();
    } catch (e) {
      fail(e);
    }
  }
  return {
    ...store,
    initialize,
    setFirmwareCenter,
    refreshFirmwareCenter,
    dismissApplicationNotice: () =>
      store.update((s) => ({ ...s, missingApplications: 0 })),
    selectFirmwareReport: (id: string) =>
      store.update((s) => ({ ...s, firmwareReportId: id })),
    openFirmwareReport: (id: string | null) =>
      store.update((s) => ({
        ...s,
        firmwareReportId: id,
        firmwareRequest: s.firmwareRequest + 1,
        page: 'Firmware & Security',
      })),
    work,
    captureTrial,
    finishTrial,
    executeCleanup,
    changeCacheSelection, prepareCacheBulk, executeCacheBulk, keepCacheBulk, newCacheSelection,
    previewProjectCleanup,
    analyzeDeveloper,
    save,
    scan,
    cancel,
    purge,
    run,
    service,
    refresh,
    retryMonitoring: startMonitoring,
    navigate: (page: Page) => store.update((s) => ({ ...s, page })),
    openOverviewResults: () =>
      store.update((s) => ({
        ...s,
        page: 'Dashboard',
        overviewMode: 'results',
      })),
    showOverviewStart: () =>
      store.update((s) => (s.scanning ? s : { ...s, overviewMode: 'start' })),
    startInvestigation: (question: string) =>
      store.update((s) => ({
        ...s,
        askDraft: question.trim(),
        page: 'Ask Coqui',
      })),
    clearAskDraft: () => store.update((s) => ({ ...s, askDraft: '' })),
    openRecommendation: (page: Page, target: string) =>
      store.update((s) => ({ ...s, reviewTarget: target, page })),
    clearReviewTarget: () =>
      store.update((s) => ({ ...s, reviewTarget: null })),
    selectSession: (session: DiagnosticSession) =>
      store.update((s) => ({ ...s, session, page: 'System Analysis' })),
    dismiss: () => store.update((s) => ({ ...s, error: null, notice: null })),
    dispose: async () => {
      disposed = true;
      stopCacheSource();
      if (maintenanceTimer) clearInterval(maintenanceTimer);
      await service.cancelWork();
      store.update((s) => ({
        ...s,
        samples: [],
        monitoringStatus: 'off',
        monitoringError: null,
      }));
      await service.cancelDeveloper();
      await service.cancelCleanup();
      await service.stopMonitoring();
      if (monitoring) await monitoring;
    },
  };
}
export const app = createAppStore();
