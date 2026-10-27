import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import Optimization from './Optimization.svelte';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import type { CleanupPlan } from '../bindings/CleanupPlan';
const file = {
  path: '/Users/fixture/Library/Caches/Example/cache.dat',
  allocatedBytes: 4096,
  logicalBytes: 25,
  modifiedAt: '2026-10-21T08:00:00Z',
  origin:
    'User Library/Caches. Home package.json was parsed with no workspace declaration; it does not establish project ownership of this cache namespace.',
  consequences: 'Cached data may need to be regenerated.',
};
const plan: CleanupPlan = {
  projectCleanup: false, duplicateCleanup:false,
  id: 'review',
  sessionId: 'fixture',
  createdAt: '2026-10-21T08:01:00Z',
  expiresAt: '2026-10-21T08:11:00Z',
  approvedAt: null,
  finishedAt: null,
  status: 'review',
  items: [
    { file, status: 'pending', detail: 'Awaiting approval', trashPath: null },
  ],
  allocatedBytes: 4096,
  verification: 'Not executed.',
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: async () => [{ ...session, status: 'complete' }],
    cleanupFiles: async () => ({
      files: [file],
      nextOffset: null,
      unavailableCount: 0,
    }),
    previewCleanup: vi.fn(async () => structuredClone(plan)),
    declineCleanup: vi.fn(async () => ({ ...plan, status: 'declined' })),
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  render(Optimization, { controller, onScan: () => {} });
  await fireEvent.click(screen.getByRole('tab', { name: 'Cache files' }));
  return { controller, service };
}
async function openReview() {
  await fireEvent.click(
    await screen.findByRole('checkbox', { name: `Select ${file.path}` }),
  );
  await fireEvent.click(
    screen.getByRole('button', { name: 'Review selected files' }),
  );
  return screen.findByRole('dialog', {
    name: 'Move 1 selected file to Trash?',
  });
}
describe('cleanup approval and verification', () => {
  it('never preselects files or executes during preview and decline', async () => {
    const { service } = await setup();
    expect(
      await screen.findByRole('checkbox', { name: `Select ${file.path}` }),
    ).not.toBeChecked();
    expect(
      screen.getByRole('button', { name: 'Review selected files' }),
    ).toBeDisabled();
    await openReview();
    expect(screen.getByRole('dialog')).toHaveTextContent(file.origin);
    expect(service.executeCleanup).not.toHaveBeenCalled();
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', { name: 'Keep files' }));
    await waitFor(() =>
      expect(service.declineCleanup).toHaveBeenCalledWith('review'),
    );
    expect(service.executeCleanup).not.toHaveBeenCalled();
  });
  it('requires both acknowledgements, sends the bound approval and presents verified results', async () => {
    const completed: CleanupPlan = {
      ...plan,
      status: 'complete',
      verification: 'Original path absent; same file verified in Trash.',
      items: [
        {
          file,
          status: 'movedToTrash',
          detail: 'Verified in Trash',
          trashPath: '/Users/fixture/.Trash/cache.dat',
        },
      ],
    };
    const executeCleanup = vi.fn(async (_approval, on) => {
      on(completed);
      return completed;
    });
    const { service } = await setup({ executeCleanup });
    await openReview();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'I have closed the applications that use these files.',
      }),
    );
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed every path/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    );
    await waitFor(() =>
      expect(service.executeCleanup).toHaveBeenCalledWith(
        {
          planId: 'review',
          reviewedAt: plan.createdAt,
          applicationsClosed: true,
          consequencesAccepted: true,
        },
        expect.any(Function),
      ),
    );
    expect(
      await screen.findByText('1 of 1 files verified in Trash.'),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Inspect per-file results' }),
    );
    expect(screen.getByText('Verified in Trash')).toBeInTheDocument();
    expect(screen.getByRole('dialog')).toHaveTextContent(file.origin);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Show in Trash' }),
    );
    expect(service.reveal).toHaveBeenCalledWith(
      '/Users/fixture/.Trash/cache.dat',
    );
  });
  it('clears a transport-failed approval and sends the user to action history', async () => {
    const { controller } = await setup({
      executeCleanup: async () => {
        throw new Error('Channel disconnected');
      },
    });
    await openReview();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'I have closed the applications that use these files.',
      }),
    );
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed every path/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Open Action history',
    );
    expect(get(controller).cleanupRunning).toBe(false);
    expect(get(controller).cleanupPlan).toBeNull();
  });
  it('reopens retained interrupted records with truthful recovery availability', async () => {
    await setup({
      cleanupHistory: async () => [
        {
          ...plan,
          status: 'interrupted',
          verification: 'Inspect unverified files manually.',
          items: [
            {
              file,
              status: 'unverified',
              detail: 'The app closed during this operation.',
              trashPath: null,
            },
          ],
        },
      ],
    });
    await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: /Cache file review/ }),
    );
    expect(
      screen.getByText('The app closed during this operation.'),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Show in Trash' })).toBeNull();
  });
  it('provides an empty state and retries a failed candidate query', async () => {
    const cleanupFiles = vi
      .fn()
      .mockRejectedValueOnce(new Error('Scan index unavailable'))
      .mockResolvedValue({ files: [], nextOffset: null, unavailableCount: 2 });
    await setup({ cleanupFiles });
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Scan index unavailable',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry loading' }),
    );
    expect(
      await screen.findByText('No eligible cache files on this page.'),
    ).toBeInTheDocument();
    expect(screen.getByText(/2 indexed entries/)).toBeInTheDocument();
  });
});
