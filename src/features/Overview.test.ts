import {
  render,
  screen,
  fireEvent,
  within,
  waitFor,
} from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import App from '../App.svelte';
import Dashboard from './Dashboard.svelte';
import { createAppStore } from '../state';
import type { ScanReview } from '../bindings/ScanReview';
import {
  fakeBackend,
  session,
  metric,
  settings,
} from '../testing/fake-backend';

describe('straightforward overview', () => {
  const retainedReview: ScanReview = {
    sessionId: 'fixture',
    scanStatus: 'partial',
    observedAt: '2026-10-23T10:00:00Z',
    groups: [
      {
        kind: 'caches',
        state: 'partial',
        source: 'retainedScanCategories',
        observedAt: '2026-10-23T10:00:00Z',
        count: 14,
        logicalBytes: 200,
        allocatedBytes: 1000000000,
        thresholdBytes: null,
        destination: 'cacheFiles',
        qualification:
          'Indexed cache footprint; eligibility has not been reviewed.',
      },
      {
        kind: 'startup',
        state: 'partial',
        source: 'retainedSnapshot',
        observedAt: '2026-10-23T09:00:00Z',
        count: 3,
        logicalBytes: null,
        allocatedBytes: null,
        thresholdBytes: null,
        destination: 'startup',
        qualification:
          'Readable launch configurations; enabled state is unknown.',
      },
      {
        kind: 'applications',
        state: 'notChecked',
        source: 'retainedSnapshot',
        observedAt: null,
        count: null,
        logicalBytes: null,
        allocatedBytes: null,
        thresholdBytes: null,
        destination: 'applications',
        qualification: 'No application inventory observation was retained.',
      },
    ],
  };

  it('shows retained file totals and unknown observations without treating candidates as reclaimable', async () => {
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      scanReview: vi.fn(async () => structuredClone(retainedReview)),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openOverviewResults();
    render(Dashboard, { controller, onScan: vi.fn() });
    await screen.findByText('1 GB in caches');
    expect(screen.getByText('14 indexed cache files')).toBeInTheDocument();
    expect(screen.getByText('3 startup configurations')).toBeInTheDocument();
    expect(
      screen.getByText('Inventory and update availability not checked here'),
    ).toBeInTheDocument();
    expect(screen.queryByText('0 recorded apps')).toBeNull();
    expect(screen.queryByText(/GB (reclaimable|of junk)/)).toBeNull();
    await fireEvent.click(screen.getByText('What was checked'));
    expect(
      screen.getByText(retainedReview.groups[0].qualification),
    ).toBeVisible();
    await fireEvent.click(screen.getByRole('button', { name: /^Cleanup / }));
    expect(get(controller).reviewTarget).toBe('cache');
    expect(service.scanReview).toHaveBeenCalledWith('fixture');
    expect(service.refreshApplicationInventory).not.toHaveBeenCalled();
    expect(service.previewCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('retries a failed summary read without starting collection', async () => {
    const scanReview = vi
      .fn()
      .mockRejectedValueOnce(new Error('Read temporarily unavailable'))
      .mockResolvedValue(retainedReview);
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      scanReview,
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openOverviewResults();
    render(Dashboard, { controller, onScan: vi.fn() });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Retry review totals' }),
    );
    await screen.findByText('1 GB in caches');
    expect(scanReview).toHaveBeenCalledTimes(2);
    expect(service.scan).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('links a saved partial update check without starting a network request or claiming apps are current', async () => {
    const backend = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      latestAppUpdates: vi.fn(async () => ({
        id: 'update-report',
        planId: 'plan',
        createdAt: '2026-10-23T10:00:00Z',
        finishedAt: '2026-10-23T10:00:01Z',
        status: 'partial',
        items: [],
        qualification: 'Source was unavailable; no version verdict.',
      })),
    });
    const controller = createAppStore(backend);
    await controller.initialize();
    controller.openOverviewResults();
    render(Dashboard, { controller, onScan: vi.fn() });
    await screen.findByText('Saved partial update check · open results');
    expect(
      screen.getByText('Saved partial check · not refreshed by this scan'),
    ).toBeInTheDocument();
    expect(screen.queryByText(/updates not checked/)).toBeNull();
    await fireEvent.click(
      screen.getByRole('button', { name: /^Applications / }),
    );
    expect(get(controller).reviewTarget).toBe('updates');
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    expect(screen.queryByText(/up to date|0 newer versions/i)).toBeNull();
    await controller.dispose();
  });

  it('discards an older summary response when the selected scan changes', async () => {
    let resolveOld!: (value: ScanReview) => void;
    const old = new Promise<ScanReview>((resolve) => {
      resolveOld = resolve;
    });
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      scanReview: vi.fn(async (id) =>
        id === 'fixture'
          ? old
          : { ...retainedReview, sessionId: id, groups: [] },
      ),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openOverviewResults();
    render(Dashboard, { controller, onScan: vi.fn() });
    await waitFor(() =>
      expect(service.scanReview).toHaveBeenCalledWith('fixture'),
    );
    controller.update((value) => ({
      ...value,
      session: { ...session, id: 'new-scan', status: 'partial' },
    }));
    await waitFor(() =>
      expect(service.scanReview).toHaveBeenCalledWith('new-scan'),
    );
    resolveOld(retainedReview);
    await waitFor(() =>
      expect(screen.queryByText('Reading recorded scan findings…')).toBeNull(),
    );
    expect(screen.queryByText('1 GB in caches')).toBeNull();
    await controller.dispose();
  });
  it('starts with one scan action and opens saved results without collecting again', async () => {
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(Dashboard, { controller, onScan: vi.fn() });
    expect(screen.getAllByRole('button', { name: 'Scan Mac' })).toHaveLength(1);
    expect(
      screen.queryByRole('region', { name: 'Explore your Mac' }),
    ).toBeNull();
    await fireEvent.click(
      screen.getByRole('button', { name: 'View saved results' }),
    );
    expect(
      screen.getByRole('region', { name: 'Explore your Mac' }),
    ).toBeInTheDocument();
    expect(service.scan).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to start' }),
    );
    expect(
      screen.queryByRole('region', { name: 'Explore your Mac' }),
    ).toBeNull();
    await controller.dispose();
  });

  it('keeps the volume review before collection and changes the view only when scanning starts', async () => {
    const service = fakeBackend({
      initialize: async () => ({ ...settings, scanMode: 'deep' }),
      diskScanPlan: async () => ({
        id: 'reviewed-scope',
        createdAt: '',
        qualification: 'Disposable scope fixture',
        volumes: [
          {
            id: 'root',
            mount: '/',
            device: 'fixture',
            filesystem: 'apfs',
            internal: true,
            selected: true,
            readOnly: true,
            qualification: 'Fixture',
          },
        ],
      }),
    });
    const controller = createAppStore(service);
    render(App, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Scan Mac' }),
    );
    await screen.findByRole('dialog', {
      name: 'Review whole internal disk scope',
    });
    expect(service.scan).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(get(controller).overviewMode).toBe('start');
    await fireEvent.click(screen.getByRole('button', { name: 'Scan Mac' }));
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Scan selected volumes' }),
      ).toBeEnabled(),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Scan selected volumes' }),
    );
    await waitFor(() => expect(get(controller).overviewMode).toBe('results'));
    expect(service.scan).toHaveBeenCalledTimes(1);
    expect(service.scan).toHaveBeenCalledWith(
      expect.objectContaining({
        mode: 'deep',
        planId: 'reviewed-scope',
        roots: ['/'],
      }),
      expect.any(Function),
      null,
      { system: true, startup: true, applications: true },
    );
    await controller.dispose();
  });

  it('opens existing review workflows without scanning or granting cleanup authority', async () => {
    const service = fakeBackend();
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openOverviewResults();
    const scan = vi.fn();
    render(Dashboard, { controller, onScan: scan });
    const destinations = within(
      screen.getByRole('region', { name: 'Explore your Mac' }),
    );
    for (const [name, page] of [
      ['Cleanup', 'Optimization'],
      ['Performance', 'Performance'],
      ['Applications', 'Applications'],
      ['Storage', 'Storage'],
    ]) {
      await fireEvent.click(
        destinations.getByRole('button', { name: new RegExp(`^${name}`) }),
      );
      expect(get(controller).page).toBe(page);
    }
    expect(scan).not.toHaveBeenCalled();
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.previewCleanup).not.toHaveBeenCalled();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    expect(screen.queryByText(/healthy|safe to delete|GB of junk/i)).toBeNull();
    await controller.dispose();
  });

  it('preserves partial-scan qualifications and never presents stale metrics as live', async () => {
    const controller = createAppStore(
      fakeBackend({
        history: async () => [
          {
            ...session,
            status: 'partial',
            fileCount: 1500,
            allocatedBytes: 2000000000,
          },
        ],
      }),
    );
    await controller.initialize();
    controller.openOverviewResults();
    controller.update((s) => ({
      ...s,
      monitoringStatus: 'failed',
      samples: [metric],
    }));
    render(Dashboard, { controller, onScan: vi.fn() });
    expect(screen.getByText('2 GB indexed')).toBeInTheDocument();
    expect(
      screen.getByText('Partial scan · coverage gaps remain'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('Monitoring unavailable · open to retry'),
    ).toBeInTheDocument();
    expect(screen.queryByText('12.0% CPU in use')).toBeNull();
    await controller.dispose();
  });

  it('keeps one Stop control active through scan preparation and cancellation', async () => {
    const service = fakeBackend();
    const controller = createAppStore(service);
    await controller.initialize();
    controller.update((s) => ({ ...s, scanning: true, session: null }));
    render(Dashboard, { controller, onScan: vi.fn() });
    expect(screen.getByText('Preparing your scan…')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Scan Mac' })).toBeNull();
    await fireEvent.click(screen.getByRole('button', { name: 'Stop scan' }));
    expect(service.cancel).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: 'Stopping…' })).toBeDisabled();
    expect(screen.getByText(/retaining partial results/)).toBeInTheDocument();
    await controller.dispose();
  });

  it('keeps advanced routes findable and automatically opens their navigation group', async () => {
    const controller = createAppStore(fakeBackend());
    render(App, { controller });
    await screen.findByRole('heading', { name: 'Let’s check your Mac.' });
    document.querySelector<HTMLElement>('main.content')!.scrollTo = vi.fn();
    const navigation = within(
      screen.getByRole('navigation', { name: 'Main navigation' }),
    );
    expect(
      navigation.getByRole('button', { name: 'Overview' }),
    ).toHaveAttribute('aria-current', 'page');
    expect(
      navigation.getByRole('button', { name: 'Hardware' }),
    ).not.toBeVisible();
    await fireEvent.click(navigation.getByText('More tools'));
    expect(navigation.getByRole('button', { name: 'Hardware' })).toBeVisible();
    await fireEvent.click(navigation.getByRole('button', { name: 'Hardware' }));
    expect(get(controller).page).toBe('Hardware');
    await fireEvent.click(navigation.getByText('More tools'));
    controller.navigate('Dashboard');
    await waitFor(() =>
      expect(
        navigation.getByRole('button', { name: 'Overview' }),
      ).toHaveAttribute('aria-current', 'page'),
    );
    controller.navigate('System Analysis');
    await waitFor(() =>
      expect(
        navigation.getByRole('button', { name: 'Scan results' }),
      ).toHaveAttribute('aria-current', 'page'),
    );
    const utility = within(
      screen.getByRole('navigation', { name: 'Application navigation' }),
    );
    for (const name of ['Ask Coqui', 'History', 'Settings'])
      expect(utility.getByRole('button', { name })).toBeInTheDocument();
    await controller.dispose();
  });
});
