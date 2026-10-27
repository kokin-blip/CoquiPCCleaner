import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import App from '../App.svelte';
import Dashboard from './Dashboard.svelte';
import Performance from './Performance.svelte';
import RecordDetails from '../components/RecordDetails.svelte';
import { createAppStore } from '../state';
import { fakeBackend, settings, session } from '../testing/fake-backend';
import type { ScanObservations } from '../bindings/ScanObservations';

const report: ScanObservations = {
  id: 'scan-observations-fixture',
  sessionId: session.id,
  createdAt: '2026-10-26T12:00:00Z',
  finishedAt: '2026-10-26T12:00:01Z',
  status: 'partial',
  system: null,
  startupItems: [],
  applications: [],
  qualification: 'Saved scan checks; no new collection.',
  checks: [
    {
      module: 'system',
      status: 'notSelected',
      startedAt: null,
      finishedAt: null,
      count: null,
      omissions: 0,
      qualification: 'System check was not selected.',
    },
    {
      module: 'startup',
      status: 'cancelled',
      startedAt: null,
      finishedAt: null,
      count: null,
      omissions: 0,
      qualification: 'Startup check was cancelled before collection.',
    },
    {
      module: 'applications',
      status: 'partial',
      startedAt: '2026-10-26T12:00:00Z',
      finishedAt: '2026-10-26T12:00:01Z',
      count: 2,
      omissions: 1,
      qualification: 'Only two application metadata records were observed.',
    },
  ],
};
describe('selected scan modules and saved checks', () => {
  it('reviews choices before selected-folder collection and passes explicit skipped checks', async () => {
    const service = fakeBackend({
      initialize: async () => ({ ...settings, scanMode: 'selected' }),
    });
    const controller = createAppStore(service);
    render(App, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Scan Mac' }),
    );
    expect(service.scan).not.toHaveBeenCalled();
    for (const name of [
      'System details',
      'Startup configurations',
      'Application inventory',
    ]) {
      const input = screen.getByRole('checkbox', { name });
      expect(input).toBeChecked();
      await fireEvent.click(input);
    }
    await fireEvent.click(screen.getByRole('button', { name: 'Start scan' }));
    await waitFor(() =>
      expect(service.scan).toHaveBeenCalledWith(
        expect.objectContaining({ mode: 'selected' }),
        expect.any(Function),
        null,
        { system: false, startup: false, applications: false },
      ),
    );
    expect(service.checkAppUpdates).not.toHaveBeenCalled();
    expect(service.analyzeDuplicates).not.toHaveBeenCalled();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('cancels selection without work and starts a later review with fresh defaults', async () => {
    const service = fakeBackend();
    const controller = createAppStore(service);
    render(App, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Scan Mac' }),
    );
    await fireEvent.click(
      screen.getByRole('checkbox', { name: 'Application inventory' }),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(service.scan).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole('button', { name: 'Scan Mac' }));
    expect(
      screen.getByRole('checkbox', { name: 'Application inventory' }),
    ).toBeChecked();
    await controller.dispose();
  });
  it('opens saved per-check states and separate tools without collecting or authorizing removal', async () => {
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      scanObservations: vi.fn(async () => report),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openOverviewResults();
    render(Dashboard, { controller, onScan: vi.fn() });
    await waitFor(() =>
      expect(service.scanObservations).toHaveBeenCalledWith(session.id),
    );
    await fireEvent.click(await screen.findByText('What was checked'));
    expect(await screen.findByText('Not selected')).toBeVisible();
    expect(screen.getByText('Cancelled')).toBeVisible();
    expect(screen.queryByText('0 recorded entries')).toBeNull();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review duplicate checks' }),
    );
    expect(service.analyzeDuplicates).not.toHaveBeenCalled();
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.checkAppUpdates).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('does not route an old history report to another scan’s system or startup results', async () => {
    const controller = createAppStore(fakeBackend());
    await controller.initialize();
    render(RecordDetails, {
      controller,
      record: { kind: 'scanObservations', data: report },
    });
    expect(
      screen.getByText('System check was not selected.'),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: 'Review system details' }),
    ).toBeNull();
    await controller.dispose();
  });
  it('uses this scan’s startup result instead of older snapshot rows and only reads retained observations', async () => {
    const service = fakeBackend({
      history: async () => [{ ...session, status: 'partial' }],
      scanObservations: vi.fn(async () => report),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.openRecommendation('Performance', 'startup');
    render(Performance, { controller });
    expect(
      await screen.findByText('Startup check was cancelled before collection.'),
    ).toBeInTheDocument();
    expect(service.scan).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
