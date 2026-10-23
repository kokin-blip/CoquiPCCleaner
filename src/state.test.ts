import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createAppStore } from './state';
import type { Backend } from './backend';
import type { AppSettings } from './bindings/AppSettings';
import type { DiagnosticSession } from './bindings/DiagnosticSession';
import { fakeBackend, session, settings, metric } from './testing/fake-backend';
describe('native scan state', () => {
  it('serializes preference saves without losing rapid changes', async () => {
    const saved: AppSettings[] = [];
    const c = createAppStore(
      fakeBackend({
        save: async (s) => {
          saved.push(s);
        },
      }),
    );
    await c.initialize();
    const a = c.save({ ...get(c).settings!, appearance: 'dark' });
    const b = c.save({ ...get(c).settings!, retentionDays: 90 });
    await Promise.all([a, b]);
    expect(saved.map((s) => s.retentionDays)).toEqual([30, 90]);
    expect(get(c).settings?.appearance).toBe('dark');
    expect(get(c).settings?.retentionDays).toBe(90);
  });
  it('loads retained results and saved preferences on reopening', async () => {
    const c = createAppStore(
      fakeBackend({ history: async () => [{ ...session, status: 'partial' }] }),
    );
    await c.initialize();
    expect(get(c).session?.status).toBe('partial');
    expect(get(c).settings?.selectedRoots).toEqual(['/fixture']);
  });
  it('streams real transitions and preserves partial results after cancellation', async () => {
    let finish!: () => void;
    const service = fakeBackend({
      scan: async (_r, on) => {
        on({ session });
        await new Promise<void>((r) => (finish = r));
        on({ session: { ...session, status: 'cancelled', fileCount: 12 } });
      },
    });
    const c = createAppStore(service);
    await c.initialize();
    const running = c.scan();
    expect(get(c).scanning).toBe(true);
    expect(get(c).session?.stage).toBe('Storage metadata');
    await c.cancel();
    expect(service.cancel).toHaveBeenCalledOnce();
    finish();
    await running;
    expect(get(c).session?.fileCount).toBe(12);
    expect(get(c).session?.status).toBe('cancelled');
    expect(get(c).scanning).toBe(false);
  });
  it('reports permission/transport failures without claiming completion', async () => {
    const c = createAppStore(
      fakeBackend({
        scan: async () => {
          throw new Error('Access denied');
        },
      }),
    );
    await c.initialize();
    await c.scan();
    expect(get(c).error).toBe('Access denied');
    expect(get(c).session).toBeNull();
    expect(get(c).scanning).toBe(false);
  });
  it('does not discard displayed data when clearing the database fails', async () => {
    const c = createAppStore(
      fakeBackend({
        history: async () => [session],
        purge: async () => {
          throw new Error('database busy');
        },
      }),
    );
    await c.initialize();
    await c.run(() => c.purge());
    expect(get(c).history).toHaveLength(1);
    expect(get(c).error).toBe('database busy');
  });
  it('keeps a bounded live sample buffer', async () => {
    const metric = {
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
    const c = createAppStore(
      fakeBackend({
        initialize: async () => ({ ...settings, monitoringEnabled: true }),
        monitor: async (on) => {
          for (let i = 0; i < 470; i++) on(metric);
          await new Promise<void>(() => {});
        },
      }),
    );
    await c.initialize();
    expect(get(c).samples).toHaveLength(450);
  });
  it('clears stale measurements on failure and supports recovery after dismissal', async () => {
    let reject!: (e: Error) => void;
    const monitor = vi.fn(async (on: (m: typeof metric) => void) => {
      on(metric);
      await new Promise<void>((_resolve, r) => {
        reject = r;
      });
    });
    const c = createAppStore(
      fakeBackend({
        initialize: async () => ({ ...settings, monitoringEnabled: true }),
        monitor,
      }),
    );
    await c.initialize();
    expect(get(c).monitoringStatus).toBe('live');
    reject(new Error('Collector unavailable'));
    await vi.waitFor(() => expect(get(c).monitoringStatus).toBe('failed'));
    c.dismiss();
    expect(get(c).samples).toHaveLength(0);
    expect(get(c).monitoringError).toBe('Collector unavailable');
    c.retryMonitoring();
    expect(monitor).toHaveBeenCalledTimes(2);
    expect(get(c).monitoringStatus).toBe('live');
  });
  it('does not present an unexpectedly ended stream as live', async () => {
    const c = createAppStore(
      fakeBackend({
        initialize: async () => ({ ...settings, monitoringEnabled: true }),
        monitor: async (on) => {
          on(metric);
        },
      }),
    );
    await c.initialize();
    await vi.waitFor(() => expect(get(c).monitoringStatus).toBe('failed'));
    expect(get(c).samples).toHaveLength(0);
    expect(get(c).monitoringError).toContain('stream ended');
  });
});

describe('new scan scope preparation', () => {
  it('does not show the previous completed scan as active progress and restores it on preflight failure', async () => {
    let reject!: (e: Error) => void;
    const controller = createAppStore(
      fakeBackend({
        history: async () => [{ ...session, status: 'complete' }],
        scan: async () => new Promise<void>((_resolve, r) => (reject = r)),
      }),
    );
    await controller.initialize();
    const running = controller.scan();
    expect(get(controller).scanning).toBe(true);
    expect(get(controller).session).toBeNull();
    reject(new Error('Volume scope changed; review again'));
    await running;
    expect(get(controller).session?.status).toBe('complete');
    expect(get(controller).error).toContain('Volume scope changed');
    await controller.dispose();
  });
});
