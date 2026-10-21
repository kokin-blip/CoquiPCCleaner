import { get, writable } from 'svelte/store';
import { backend as nativeBackend, type Backend } from './backend';
import type { AppSettings } from './bindings/AppSettings';
import type { DiagnosticSession } from './bindings/DiagnosticSession';
import type { LiveMetrics } from './bindings/LiveMetrics';

export type Page =
  | 'Dashboard'
  | 'System Analysis'
  | 'Hardware'
  | 'Storage'
  | 'Performance'
  | 'Firmware & Security'
  | 'History'
  | 'Settings';
export interface AppState {
  loading: boolean;
  settings: AppSettings | null;
  session: DiagnosticSession | null;
  history: DiagnosticSession[];
  samples: LiveMetrics[];
  monitoringStatus: 'off' | 'starting' | 'live' | 'failed';
  monitoringError: string | null;
  scanning: boolean;
  cancelling: boolean;
  error: string | null;
  notice: string | null;
  page: Page;
}
export function createAppStore(service: Backend = nativeBackend) {
  const store = writable<AppState>({
    loading: true,
    settings: null,
    session: null,
    history: [],
    samples: [],
    monitoringStatus: 'off',
    monitoringError: null,
    scanning: false,
    cancelling: false,
    error: null,
    notice: null,
    page: 'Dashboard',
  });
  let monitoring: Promise<void> | null = null;
  let disposed = false;
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

  async function scan() {
    const s = get(store);
    if (!s.settings || s.scanning) return;
    store.update((s) => ({
      ...s,
      scanning: true,
      cancelling: false,
      error: null,
      notice: null,
    }));
    try {
      await service.scan(
        {
          mode: s.settings.scanMode,
          roots: s.settings.selectedRoots,
          excludedPaths: s.settings.excludedPaths,
        },
        (e) => store.update((s) => ({ ...s, session: e.session })),
      );
      await refresh();
    } catch (e) {
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
      samples: [],
      notice:
        'Local diagnostic history and file indexes removed. Your files and preferences were preserved.',
    }));
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
    save,
    scan,
    cancel,
    purge,
    run,
    service,
    refresh,
    retryMonitoring: startMonitoring,
    navigate: (page: Page) => store.update((s) => ({ ...s, page })),
    selectSession: (session: DiagnosticSession) =>
      store.update((s) => ({ ...s, session, page: 'System Analysis' })),
    dismiss: () => store.update((s) => ({ ...s, error: null, notice: null })),
    dispose: async () => {
      disposed = true;
      store.update((s) => ({
        ...s,
        samples: [],
        monitoringStatus: 'off',
        monitoringError: null,
      }));
      await service.stopMonitoring();
      if (monitoring) await monitoring;
    },
  };
}
export const app = createAppStore();
