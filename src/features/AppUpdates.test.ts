import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import AppUpdates from './AppUpdates.svelte';
import Applications from './Applications.svelte';
import RecordDetails from '../components/RecordDetails.svelte';
import { createAppStore } from '../state';
import { fakeBackend } from '../testing/fake-backend';
import type { AppUpdatePlan } from '../bindings/AppUpdatePlan';
import type { AppUpdateReport } from '../bindings/AppUpdateReport';

const candidate = {
  path: '/fixture/Example.app',
  name: 'Example',
  bundleId: 'org.example.Example',
  installedVersion: '1.0',
  installedBuild: '100',
  source: 'sparkle' as const,
  sourceUrl: 'https://updates.example.org/appcast.xml',
  sourceReviewDate: '2026-10-24',
  qualification: 'Fixture source; no network request has been made.',
};
const plan: AppUpdatePlan = {
  id: 'plan',
  createdAt: '2026-10-24T10:00:00Z',
  expiresAt: '2026-10-24T10:10:00Z',
  candidates: [candidate],
  qualification:
    'Fresh metadata will be rechecked before contacting this source.',
};
const report: AppUpdateReport = {
  id: 'report',
  planId: plan.id,
  createdAt: '2026-10-24T10:01:00Z',
  finishedAt: '2026-10-24T10:01:01Z',
  status: 'complete',
  qualification:
    'Offered metadata only; installation and compatibility remain unverified.',
  items: [
    {
      ...candidate,
      state: 'available',
      checkedAt: '2026-10-24T10:01:01Z',
      offeredVersion: '1.1',
      offeredBuild: '110',
      publishedAt: null,
      minSystemVersion: null,
    },
  ],
};
function service(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  return fakeBackend({
    applicationPage: vi.fn(async () => ({
      applications: [
        {
          path: candidate.path,
          name: candidate.name,
          bundleId: candidate.bundleId,
          version: candidate.installedVersion,
          observedAt: '2026-10-23T10:00:00Z',
          protection: [],
          related: [],
          qualification: 'Recorded fixture',
        },
      ],
      total: 1,
      offset: 0,
      qualification: 'Recorded inventory',
    })),
    prepareAppUpdates: vi.fn(async () => structuredClone(plan)),
    checkAppUpdates: vi.fn(async (_id, _ack, on) => {
      on(report);
      return structuredClone(report);
    }),
    openAppUpdate: vi.fn(async () => {}),
    ...overrides,
  });
}
async function sourceReview() {
  await fireEvent.click(
    await screen.findByRole('checkbox', { name: 'Check updates for Example' }),
  );
  await fireEvent.click(
    screen.getByRole('button', { name: 'Review update sources' }),
  );
  await screen.findByRole('heading', { name: 'Review online sources' });
  await waitFor(() =>
    expect(
      screen.getByRole('heading', { name: 'Review online sources' }),
    ).toHaveFocus(),
  );
}

describe('opt-in vendor-managed update review', () => {
  it('opens the Applications update route without starting inventory collection or network work', async () => {
    const backend = service();
    const controller = createAppStore(backend);
    await controller.initialize();
    controller.openRecommendation('Applications', 'updates');
    render(Applications, { controller });
    await screen.findByRole('heading', { name: 'Check app updates' });
    expect(backend.refreshApplicationInventory).not.toHaveBeenCalled();
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('browses recorded apps without selecting items or contacting a network source', async () => {
    const backend = service();
    const controller = createAppStore(backend);
    await controller.initialize();
    render(AppUpdates, { controller });
    expect(
      await screen.findByRole('checkbox', {
        name: 'Check updates for Example',
      }),
    ).not.toBeChecked();
    expect(
      screen.getByRole('button', { name: 'Review update sources' }),
    ).toBeDisabled();
    expect(backend.prepareAppUpdates).not.toHaveBeenCalled();
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    expect(backend.refreshApplicationInventory).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('requires a separate network acknowledgement after source review and opens the vendor only on request', async () => {
    const backend = service();
    const controller = createAppStore(backend);
    await controller.initialize();
    render(AppUpdates, { controller });
    await sourceReview();
    expect(backend.prepareAppUpdates).toHaveBeenCalledWith([candidate.path]);
    expect(screen.getByText(candidate.sourceUrl!)).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Check selected updates' }),
    ).toBeDisabled();
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /^Contact these sources/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check selected updates' }),
    );
    await screen.findByText('Newer version listed');
    expect(
      screen.queryByRole('button', { name: 'Review update sources' }),
    ).toBeNull();
    expect(
      screen.getByRole('region', {
        name: 'Completed and pending update results',
      }),
    ).toHaveFocus();
    expect(backend.checkAppUpdates).toHaveBeenCalledWith(
      'plan',
      true,
      expect.any(Function),
    );
    expect(backend.openAppUpdate).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Open app to update' }),
    );
    await waitFor(() =>
      expect(backend.openAppUpdate).toHaveBeenCalledWith(
        'report',
        candidate.path,
      ),
    );
    expect(backend.previewRemoval).not.toHaveBeenCalled();
    expect(backend.administratorStatus).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check other apps' }),
    );
    expect(
      await screen.findByRole('checkbox', {
        name: 'Check updates for Example',
      }),
    ).not.toBeChecked();
    expect(
      screen.getByRole('searchbox', { name: 'Search apps for updates' }),
    ).toHaveFocus();
    await controller.dispose();
  });

  it('does not offer a check when no supported source was found, even after acknowledgement', async () => {
    const backend = service({
      prepareAppUpdates: vi.fn(async () => ({
        ...plan,
        candidates: [
          { ...candidate, source: 'unavailable' as const, sourceUrl: null },
        ],
      })),
    });
    const controller = createAppStore(backend);
    await controller.initialize();
    render(AppUpdates, { controller });
    await sourceReview();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /^Contact these sources/ }),
    );
    expect(
      screen.getByRole('button', { name: 'Check selected updates' }),
    ).toBeDisabled();
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Change selection' }),
    );
    expect(
      screen.getByRole('searchbox', { name: 'Search apps for updates' }),
    ).toHaveFocus();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review update sources' }),
    );
    await screen.findByRole('heading', { name: 'Review online sources' });
    expect(
      screen.getByRole('checkbox', { name: /^Contact these sources/ }),
    ).not.toBeChecked();
    await controller.dispose();
  });

  it('keeps Stop available during a check and retains cancelled findings as unmeasured', async () => {
    let finish!: (report: AppUpdateReport) => void;
    const pending = new Promise<AppUpdateReport>((resolve) => {
      finish = resolve;
    });
    const backend = service({
      checkAppUpdates: vi.fn(async (_id, _ack, on) => {
        on({ ...report, status: 'running', finishedAt: null, items: [] });
        return pending;
      }),
    });
    const controller = createAppStore(backend);
    await controller.initialize();
    render(AppUpdates, { controller });
    await sourceReview();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /^Contact these sources/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check selected updates' }),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Stop update check' }),
    );
    expect(backend.cancelWork).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: 'Stopping…' })).toBeDisabled();
    finish({
      ...report,
      status: 'cancelled',
      items: [
        {
          ...report.items[0],
          state: 'cancelled',
          checkedAt: null,
          offeredVersion: null,
          offeredBuild: null,
        },
      ],
    });
    await screen.findByText('Not checked · cancelled');
    expect(screen.getByText('No completed check')).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: 'Open app to update' }),
    ).toBeNull();
    await controller.dispose();
  });

  it('does not let a late saved-report read overwrite a new explicit check', async () => {
    let finish!: (report: AppUpdateReport) => void;
    const pending = new Promise<AppUpdateReport>((resolve) => {
      finish = resolve;
    });
    const backend = service({ latestAppUpdates: vi.fn(async () => pending) });
    const controller = createAppStore(backend);
    await controller.initialize();
    render(AppUpdates, { controller });
    await sourceReview();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /^Contact these sources/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check selected updates' }),
    );
    await screen.findByText('Newer version listed');
    finish({
      ...report,
      id: 'old',
      items: [
        {
          ...report.items[0],
          state: 'unavailable',
          qualification: 'Obsolete saved result',
        },
      ],
    });
    await waitFor(() =>
      expect(screen.queryByText('Obsolete saved result')).toBeNull(),
    );
    expect(screen.getByText('Newer version listed')).toBeInTheDocument();
    await controller.dispose();
  });

  it('reopens retained update records without a new network check or installation claim', async () => {
    const backend = service();
    const controller = createAppStore(backend);
    await controller.initialize();
    render(RecordDetails, {
      controller,
      record: {
        kind: 'appUpdateReport',
        data: {
          ...report,
          items: [{ ...report.items[0], state: 'noNewerVersionObserved' }],
        },
      },
    });
    expect(screen.getByText('No newer version observed')).toBeInTheDocument();
    expect(screen.queryByText(/up to date|update installed/i)).toBeNull();
    expect(backend.checkAppUpdates).not.toHaveBeenCalled();
    expect(backend.openAppUpdate).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
