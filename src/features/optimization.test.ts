import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import Optimization from './Optimization.svelte';
import { createAppStore } from '../state';
import {
  cacheBackend,
  cacheFiles,
  cacheSummary,
} from '../testing/cache-bulk-fixture';
import type { CleanupPlan } from '../bindings/CleanupPlan';
async function setup(overrides: Parameters<typeof cacheBackend>[0] = {}) {
  const service = cacheBackend(overrides);
  const controller = createAppStore(service);
  await controller.initialize();
  render(Optimization, { controller, onScan: vi.fn() });
  await fireEvent.click(screen.getByRole('tab', { name: 'Cache files' }));
  return { controller, service };
}
async function selectAll() {
  await fireEvent.click(
    await screen.findByRole('checkbox', {
      name: 'Select all in example.namespace',
    }),
  );
  await screen.findByText('3 files selected');
}
async function review() {
  await selectAll();
  await fireEvent.click(
    screen.getByRole('button', { name: 'Review selection' }),
  );
  await screen.findByRole('heading', { name: 'Review cache selection' });
  await screen.findAllByText('Illustrative review outcome');
}
describe('guided cache selection', () => {
  it('starts empty, selects a whole group across pages and allows individual exceptions', async () => {
    const { service } = await setup();
    expect(
      await screen.findByRole('checkbox', {
        name: 'Select all in example.namespace',
      }),
    ).not.toBeChecked();
    expect(service.cacheSelect).not.toHaveBeenCalled();
    await selectAll();
    await fireEvent.click(
      await screen.findByRole('button', { name: 'example.namespace' }),
    );
    expect(
      await screen.findByRole('checkbox', {
        name: `Select ${cacheFiles[0].path}`,
      }),
    ).toBeChecked();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Next cache files' }),
    );
    expect(
      await screen.findByRole('checkbox', {
        name: `Select ${cacheFiles[2].path}`,
      }),
    ).toBeChecked();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: `Select ${cacheFiles[2].path}` }),
    );
    await screen.findByText('2 files selected');
    expect(service.cacheSelect).toHaveBeenLastCalledWith(
      expect.objectContaining({
        paths: [cacheFiles[2].path],
        selected: false,
        revision: expect.any(Number),
      }),
      expect.any(Function),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to cache groups' }),
    );
    const box = await screen.findByRole('checkbox', {
      name: 'Select all in example.namespace',
    });
    expect(box).not.toBeChecked();
    expect((box as HTMLInputElement).indeterminate).toBe(true);
    expect(service.cacheExecute).not.toHaveBeenCalled();
  });
  it('selects the displayed page only, clears selection, and preserves it between both cache views', async () => {
    const { controller, service } = await setup({
      cleanupCategories: async () => ({
        sessionId: 'fixture',
        observedAt: null,
        sourceStatus: 'partial',
        rulesReviewedAt: '2026-10-27',
        qualification: 'Fixture',
        categories: [
          {
            kind: 'userCaches',
            title: 'App caches',
            roots: [],
            mode: 'cacheReview',
            state: 'partial',
            fileCount: 3,
            logicalBytes: 30,
            allocatedBytes: 12288,
            description: 'Fixture',
            consequences: 'Regeneration',
            qualification: 'Fixture',
            referenceUrl: null,
          },
        ],
      }),
    });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'example.namespace' }),
    );
    await fireEvent.click(
      await screen.findByRole('checkbox', { name: 'Select this page' }),
    );
    await screen.findByText('2 files selected');
    expect(service.cacheSelect).toHaveBeenLastCalledWith(
      expect.objectContaining({
        paths: cacheFiles.slice(0, 2).map((row) => row.path),
      }),
      expect.any(Function),
    );
    await fireEvent.click(screen.getByRole('tab', { name: 'Categories' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: 'App caches' }),
    );
    const box = await screen.findByRole('checkbox', {
      name: 'Select all in example.namespace',
    });
    expect((box as HTMLInputElement).indeterminate).toBe(true);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Clear selection' }),
    );
    await screen.findByText('0 files selected');
    expect(get(controller).cacheBatch?.selectedCount).toBe(0);
  });
  it('requires both acknowledgements and sends one revision-bound approval', async () => {
    const { service } = await setup();
    await review();
    expect(service.cacheExecute).not.toHaveBeenCalled();
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'I have closed the applications that use these files.',
      }),
    );
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed the selected groups/ }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Approve and move to Trash' }),
      ).toBeEnabled(),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    );
    await waitFor(() => expect(service.cacheExecute).toHaveBeenCalledTimes(1));
    expect(service.cacheExecute).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 'bulk-fixture',
        revision: expect.any(Number),
        reviewedAt: expect.any(String),
        applicationsClosed: true,
        consequencesAccepted: true,
      }),
      expect.any(Function),
    );
    expect(await screen.findByText(/3 verified in Trash/)).toBeInTheDocument();
    expect(service.executeCleanup).not.toHaveBeenCalled();
  });
  it('declines without execution and clears selection when source or exclusions change', async () => {
    const { controller, service } = await setup();
    await review();
    await fireEvent.click(screen.getByRole('button', { name: 'Keep files' }));
    await waitFor(() =>
      expect(service.cacheDecline).toHaveBeenCalledWith('bulk-fixture'),
    );
    expect(service.cacheExecute).not.toHaveBeenCalled();
    controller.update((s) => ({
      ...s,
      settings: {
        ...s.settings!,
        excludedPaths: ['/Users/fixture/Library/Caches/example.namespace'],
      },
    }));
    await waitFor(() => expect(get(controller).cacheBatch).toBeNull());
    controller.update((s) => ({
      ...s,
      cacheBatch: { ...cacheSummary, selectedCount: 3 },
      session: { ...s.session!, id: 'another-scan' },
    }));
    await waitFor(() => expect(get(controller).cacheBatch).toBeNull());
  });
  it('keeps cancellation available while preparing and clears an uncertain approval', async () => {
    let finish: (value: typeof cacheSummary) => void = () => {};
    const { controller, service } = await setup({
      cachePrepare: vi.fn(async (_id, _rev, on) => {
        on({
          ...cacheSummary,
          status: 'preparing',
          selectedCount: 3,
          processedCount: 1,
        });
        return new Promise<typeof cacheSummary>((resolve) => {
          finish = resolve;
        });
      }),
    });
    await selectAll();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review selection' }),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Cancel fresh checking' }),
    );
    expect(service.cancelCleanup).toHaveBeenCalled();
    finish({ ...cacheSummary, status: 'cancelled', selectedCount: 3 });
    await waitFor(() => expect(get(controller).cleanupPreparing).toBe(false));
    expect(
      screen.queryByRole('button', { name: 'Approve and move to Trash' }),
    ).not.toBeInTheDocument();
  });
  it('rejects late query responses, retries errors, and never selects stale rows', async () => {
    const cacheGroups = vi
      .fn()
      .mockRejectedValueOnce(new Error('Index unavailable'))
      .mockResolvedValue({ groups: [], nextCursor: null });
    const { service } = await setup({ cacheGroups });
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Index unavailable',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry cache view' }),
    );
    expect(
      await screen.findByText(/No indexed cache candidates/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Select all cache groups' }),
    ).toBeDisabled();
    expect(service.cacheSelect).not.toHaveBeenCalled();
  });
  it('retains legacy interrupted cache records and bulk history without executable approvals', async () => {
    const legacy: CleanupPlan = {
      id: 'legacy',
      sessionId: 'fixture',
      projectCleanup: false,
      duplicateCleanup: false,
      createdAt: '2026-10-27T12:00:00Z',
      expiresAt: '2026-10-27T12:10:00Z',
      approvedAt: null,
      finishedAt: null,
      status: 'interrupted',
      allocatedBytes: 4096,
      items: [
        {
          file: {
            path: cacheFiles[0].path,
            allocatedBytes: 4096,
            logicalBytes: 10,
            modifiedAt: '',
            origin: 'Legacy fixture',
            consequences: 'Regeneration',
          },
          status: 'unverified',
          detail: 'Inspect legacy source and Trash manually',
          trashPath: null,
        },
      ],
      verification: 'Interrupted',
    };
    await setup({
      cleanupHistory: async () => [legacy],
      cacheHistory: async () => [
        { ...cacheSummary, status: 'interrupted', selectedCount: 3 },
      ],
    });
    await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: /Cache file review/ }),
    );
    expect(
      screen.getByText('Inspect legacy source and Trash manually'),
    ).toBeInTheDocument();
  });
  it('ignores an older cache response after the source scan changes', async () => {
    let finish: (value: {
      groups: Array<{
        key: string;
        title: string;
        fileCount: number;
        allocatedBytes: number;
        selectedCount: number;
        eligibleCount: number;
        refusedCount: number;
      }>;
      nextCursor: null;
    }) => void = () => {};
    const cacheGroups = vi
      .fn()
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            finish = resolve;
          }),
      )
      .mockResolvedValue({ groups: [], nextCursor: null });
    const { controller, service } = await setup({ cacheGroups });
    controller.update((s) => ({
      ...s,
      session: { ...s.session!, id: 'new-source' },
    }));
    await screen.findByText(/No indexed cache candidates/);
    finish({
      groups: [
        {
          key: 'late.namespace',
          title: 'Late namespace',
          fileCount: 1,
          allocatedBytes: 4096,
          selectedCount: 0,
          eligibleCount: 0,
          refusedCount: 0,
        },
      ],
      nextCursor: null,
    });
    await waitFor(() =>
      expect(screen.queryByText('Late namespace')).not.toBeInTheDocument(),
    );
    expect(service.cacheSelect).not.toHaveBeenCalled();
  });
  it('clears an uncertain execution approval and directs recovery to history', async () => {
    const { controller } = await setup({
      cacheExecute: async () => {
        throw new Error('Channel disconnected');
      },
    });
    await review();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'I have closed the applications that use these files.',
      }),
    );
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed the selected groups/ }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Approve and move to Trash' }),
      ).toBeEnabled(),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    );
    await waitFor(() =>
      expect(get(controller).error).toContain('Open Action history'),
    );
    expect(get(controller).cacheBatch).toBeNull();
    expect(get(controller).cleanupRunning).toBe(false);
  });
  it('shows refused-file reasons and queries the frozen outcome filter', async () => {
    const service = cacheBackend({
      cachePrepare: async (_id, _revision, on) => {
        const s = {
          ...cacheSummary,
          status: 'review',
          revision: 2,
          selectedCount: 3,
          processedCount: 3,
          eligibleCount: 2,
          refusedCount: 1,
          reviewedBytes: 8192,
          reviewedAt: new Date().toISOString(),
          expiresAt: new Date(Date.now() + 600000).toISOString(),
        };
        on(s);
        return s;
      },
      cacheBatchPage: vi.fn(async (q) => ({
        items:
          q.status === 'refused'
            ? [
                {
                  ...cacheFiles[2],
                  selected: true,
                  status: 'refused',
                  detail: 'Changed indexed file; not eligible',
                },
              ]
            : [
                {
                  ...cacheFiles[0],
                  selected: true,
                  eligible: true,
                  status: 'eligible',
                },
              ],
        nextCursor: null,
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(Optimization, { controller, onScan: vi.fn() });
    await fireEvent.click(screen.getByRole('tab', { name: 'Cache files' }));
    await selectAll();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review selection' }),
    );
    await screen.findByRole('heading', { name: 'Review cache selection' });
    await fireEvent.click(screen.getByText('Exact files and outcomes'));
    await fireEvent.change(screen.getByRole('combobox', { name: 'Show' }), {
      target: { value: 'refused' },
    });
    expect(
      await screen.findByText('Changed indexed file; not eligible'),
    ).toBeInTheDocument();
    expect(service.cacheBatchPage).toHaveBeenLastCalledWith({
      id: 'bulk-fixture',
      cursor: null,
      status: 'refused',
    });
    expect(service.cacheExecute).not.toHaveBeenCalled();
  });
  it('refreshes an open active history record when bulk execution completes', async () => {
    let retained = {
      ...cacheSummary,
      status: 'running',
      selectedCount: 3,
      eligibleCount: 3,
      unattemptedCount: 3,
    };
    const cacheHistory = vi.fn(async () => [{ ...retained }]);
    const { controller, service } = await setup({
      cacheHistory,
      cacheBatchSummary: async () => ({ ...retained }),
    });
    controller.update((s) => ({ ...s, cacheBatch: { ...retained } }));
    await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: /Cache group review/ }),
    );
    await screen.findByRole('heading', { name: 'Moving approved files…' });
    const calls = cacheHistory.mock.calls.length;
    retained = {
      ...retained,
      status: 'complete',
      movedCount: 3,
      unattemptedCount: 0,
    };
    controller.update((s) => ({ ...s, cacheBatch: { ...retained } }));
    expect(await screen.findByText(/3 verified in Trash/)).toBeInTheDocument();
    await waitFor(() =>
      expect(cacheHistory.mock.calls.length).toBeGreaterThan(calls),
    );
    expect(
      screen.queryByRole('heading', { name: 'No action records yet.' }),
    ).not.toBeInTheDocument();
    expect(service.cacheExecute).not.toHaveBeenCalled();
  });
  it('polls an active retained record after transport loss without retrying execution', async () => {
    const running = {
      ...cacheSummary,
      status: 'running',
      selectedCount: 3,
      eligibleCount: 3,
      unattemptedCount: 3,
    };
    const cacheBatchSummary = vi.fn(async () => ({
      ...running,
      status: 'partial',
      movedCount: 2,
      unverifiedCount: 1,
      unattemptedCount: 0,
    }));
    const { service } = await setup({
      cacheHistory: async () => [running],
      cacheBatchSummary,
    });
    await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: /Cache group review/ }),
    );
    expect(
      await screen.findByText(/2 verified in Trash/, {}, { timeout: 3000 }),
    ).toBeInTheDocument();
    expect(cacheBatchSummary).toHaveBeenCalledWith('bulk-fixture');
    expect(service.cacheExecute).not.toHaveBeenCalled();
  });
  it('restores namespace focus and does not reread candidates for unrelated store changes', async () => {
    const { controller, service } = await setup();
    await screen.findByRole('button', { name: 'example.namespace' });
    const calls = vi.mocked(service.cacheGroups).mock.calls.length;
    controller.update((s) => ({ ...s, samples: [...s.samples] }));
    await waitFor(() =>
      expect(vi.mocked(service.cacheGroups).mock.calls.length).toBe(calls),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'example.namespace' }),
    );
    await screen.findByRole('checkbox', { name: 'Select this page' });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to cache groups' }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'example.namespace' }),
      ).toHaveFocus(),
    );
  });
  it('discloses a stopped partial selection before review', async () => {
    await setup({
      cacheSelect: async (_request, on) => {
        const s = {
          ...cacheSummary,
          selectedCount: 1,
          allocatedBytes: 4096,
          detail:
            'Selection stopped; the displayed count includes only materialized files. Nothing approved.',
        };
        on(s);
        return s;
      },
    });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Select all cache groups' }),
    );
    expect(
      await screen.findByText(/Selection stopped; the displayed count/),
    ).toBeInTheDocument();
    expect(screen.getByText('1 files selected')).toBeInTheDocument();
  });
  it('clears a hidden Cleanup selection when source or exclusions change', async () => {
    const controller = createAppStore(cacheBackend());
    await controller.initialize();
    controller.update((s) => ({
      ...s,
      cacheBatch: { ...cacheSummary, selectedCount: 3 },
      page: 'Settings',
    }));
    expect(get(controller).cacheBatch?.selectedCount).toBe(3);
    controller.update((s) => ({
      ...s,
      settings: { ...s.settings!, excludedPaths: ['/new-exclusion'] },
    }));
    expect(get(controller).cacheBatch).toBeNull();
    controller.update((s) => ({
      ...s,
      cacheBatch: { ...cacheSummary, selectedCount: 3 },
    }));
    controller.update((s) => ({
      ...s,
      session: { ...s.session!, id: 'another-source' },
    }));
    expect(get(controller).cacheBatch).toBeNull();
    await controller.dispose();
  });
});
