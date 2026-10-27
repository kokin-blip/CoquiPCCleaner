import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import Onboarding from './Onboarding.svelte';
import Storage from './Storage.svelte';
import Settings from './Settings.svelte';
import Status from '../components/Status.svelte';
import Performance from './Performance.svelte';
import Dashboard from './Dashboard.svelte';
import { fakeBackend } from '../testing/fake-backend';

describe('desktop workflows', () => {
  it('keeps stopped monitoring visible after dismissing the global error and offers retry', async () => {
    const c = createAppStore(fakeBackend());
    await c.initialize();
    c.update((s) => ({
      ...s,
      monitoringStatus: 'failed',
      monitoringError: 'Collector unavailable',
    }));
    c.dismiss();
    c.openOverviewResults();
    const dashboard = render(Dashboard, { controller: c, onScan: () => {} });
    expect(
      screen.getByText('Monitoring unavailable · open to retry'),
    ).toBeInTheDocument();
    expect(screen.queryByText('Live · sampled every 2 seconds')).toBeNull();
    dashboard.unmount();
    render(Performance, { controller: c });
    expect(
      screen.getByText(
        'Monitoring stopped. Current measurements are unavailable.',
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Retry monitoring' }),
    ).toBeInTheDocument();
  });
  it('switches tabs using arrow-key navigation', async () => {
    const c = createAppStore(fakeBackend());
    await c.initialize();
    render(Performance, { controller: c });
    await fireEvent.keyDown(screen.getByRole('tab', { name: 'Overview' }), {
      key: 'ArrowRight',
    });
    expect(screen.getByRole('tab', { name: 'Processes' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    expect(
      screen.getByRole('heading', { name: 'Top readable processes' }),
    ).toBeInTheDocument();
  });
  it('requires a workload before advancing and saves onboarding without scanning', async () => {
    const service = fakeBackend();
    const c = createAppStore(service);
    await c.initialize();
    render(Onboarding, { controller: c, onScan: () => {} });
    await fireEvent.click(
      screen.getByRole('button', { name: /Set up my computer/ }),
    );
    expect(screen.getByRole('button', { name: 'Continue' })).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: 'Software development' }),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await fireEvent.click(
      screen.getByRole('button', { name: 'Explore first' }),
    );
    await waitFor(() => expect(service.save).toHaveBeenCalled());
    expect(service.scan).not.toHaveBeenCalled();
  });
  it('drills from an indexed directory to its exact path', async () => {
    const service = fakeBackend({
      files: vi.fn(async () => [
        {
          path: '/fixture/nested',
          parent: '',
          name: 'nested',
          category: 'Other',
          logicalBytes: 4,
          allocatedBytes: 4096,
          isDirectory: true,
          modifiedAt: '2026-10-20T00:00:00Z',
          qualification: '',
        },
      ]),
    });
    const c = createAppStore(service);
    await c.initialize();
    c.update((s) => ({
      ...s,
      session: {
        coverage: [],
        volumePlan: null,
        id: 'scan',
        startedAt: '',
        finishedAt: null,
        status: 'complete',
        stage: 'Complete',
        scope: '/fixture',
        fileCount: 1,
        logicalBytes: 4,
        allocatedBytes: 4096,
        skippedCount: 0,
        snapshot: null,
        findings: [],
        permissions: [],
        categories: [],
        candidates: [],
        errors: [],
      },
    }));
    render(Storage, { controller: c, onScan: () => {} });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'nested' }),
    );
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({ parent: '/fixture/nested' }),
      ),
    );
  });
  it('opens the purge review without deleting data', async () => {
    const service = fakeBackend();
    const c = createAppStore(service);
    await c.initialize();
    render(Settings, { controller: c });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Remove diagnostic history' }),
    );
    expect(
      screen.getByRole('dialog', { name: 'Remove local diagnostic history?' }),
    ).toBeInTheDocument();
    expect(service.purge).not.toHaveBeenCalled();
  });
  it('never presents unsupported capability as supported', () => {
    render(Status, { value: 'unsupported' });
    expect(screen.getByText('Unsupported')).toBeInTheDocument();
    expect(screen.queryByText('Supported')).toBeNull();
  });
});
