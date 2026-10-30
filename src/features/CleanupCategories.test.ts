import {
  render,
  screen,
  fireEvent,
  waitFor,
  within,
} from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend, session, settings } from '../testing/fake-backend';
import Optimization from './Optimization.svelte';
import { cacheBackend } from '../testing/cache-bulk-fixture';
import type { CleanupCategoryReport } from '../bindings/CleanupCategoryReport';
import type { CleanupCategoryPage } from '../bindings/CleanupCategoryPage';
import type { FileEntry } from '../bindings/FileEntry';
import type { CleanupPlan } from '../bindings/CleanupPlan';

const categories: CleanupCategoryReport = {
  sessionId: 'fixture',
  observedAt: '2026-10-27T12:00:00Z',
  sourceStatus: 'partial',
  rulesReviewedAt: '2026-10-27',
  qualification: 'Illustrative retained category fixture',
  categories: [
    {
      kind: 'userCaches',
      title: 'App caches',
      roots: ['/Users/fixture/Library/Caches'],
      mode: 'cacheReview',
      state: 'partial',
      fileCount: 101,
      logicalBytes: 101,
      allocatedBytes: 413696,
      description: 'Recorded cache paths',
      consequences: 'Fresh checks required; caches may be useful.',
      qualification: 'Illustrative',
      referenceUrl: null,
    },
    {
      kind: 'userLogs',
      title: 'User logs',
      roots: ['/Users/fixture/Library/Logs'],
      mode: 'manualReview',
      state: 'partial',
      fileCount: 1,
      logicalBytes: 10,
      allocatedBytes: 4096,
      description: 'Recorded logs',
      consequences:
        'Logs can be needed for troubleshooting. No removal action.',
      qualification: 'Illustrative',
      referenceUrl: null,
    },
    {
      kind: 'deviceBackups',
      title: 'iPhone and iPad backups',
      roots: ['/Users/fixture/Library/Application Support/MobileSync/Backup'],
      mode: 'manualReview',
      state: 'notIndexed',
      fileCount: null,
      logicalBytes: null,
      allocatedBytes: null,
      description: 'Unverified backups',
      consequences: 'Keep needed recovery copies.',
      qualification: 'Illustrative',
      referenceUrl: 'https://support.apple.com/en-us/108809',
    },
    {
      kind: 'trash',
      title: 'Trash',
      roots: ['/Users/fixture/.Trash'],
      mode: 'protected',
      state: 'indexed',
      fileCount: 0,
      logicalBytes: 0,
      allocatedBytes: 0,
      description: 'Indexed Trash',
      consequences: 'Never emptied by Coqui.',
      qualification: 'Illustrative',
      referenceUrl: null,
    },
  ],
};
const entry: FileEntry = {
  path: '/Users/fixture/Library/Caches/app/a',
  parent: '/Users/fixture/Library/Caches/app',
  name: 'a',
  category: 'Caches',
  logicalBytes: 1,
  allocatedBytes: 4096,
  isDirectory: false,
  modifiedAt: '2026-10-27T12:00:00Z',
  qualification: 'Illustrative',
};
const plan: CleanupPlan = {
  id: 'category-review',
  sessionId: 'fixture',
  projectCleanup: false,
  duplicateCleanup: false,
  createdAt: '2026-10-27T12:00:00Z',
  expiresAt: '2026-10-27T12:10:00Z',
  approvedAt: null,
  finishedAt: null,
  status: 'review',
  allocatedBytes: 4096,
  items: [
    {
      file: {
        path: entry.path,
        logicalBytes: 1,
        allocatedBytes: 4096,
        modifiedAt: entry.modifiedAt,
        origin: 'Fixture cache inspection',
        consequences: 'May need regeneration',
      },
      status: 'pending',
      detail: 'Awaiting approval',
      trashPath: null,
    },
  ],
  verification: 'Not executed.',
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = cacheBackend({
    initialize: async () => ({ ...settings, monitoringEnabled: false }),
    history: async () => [{ ...session, status: 'partial' }],
    cleanupCategories: vi.fn(async () => structuredClone(categories)),
    cleanupCategoryEntries: vi.fn(async (query) => ({
      sessionId: query.sessionId,
      kind: query.kind,
      files: [
        query.kind === 'userLogs'
          ? {
              ...entry,
              path: '/Users/fixture/Library/Logs/needed.log',
              category: 'Other',
            }
          : entry,
      ],
      nextOffset: query.offset === 0 ? 100 : null,
      qualification: 'Saved rows only',
    })),
    previewCleanup: vi.fn(async () => structuredClone(plan)),
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  render(Optimization, { controller, onScan: vi.fn() });
  return { controller, service };
}
describe('broader cleanup categories', () => {
  it('opens Categories by default with unknown and measured zero distinct and no fresh collection', async () => {
    const { controller, service } = await setup();
    expect(screen.getByRole('tab', { name: 'Categories' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const backups = (
      await screen.findByRole('button', { name: 'iPhone and iPad backups' })
    ).closest('tr')!;
    expect(within(backups).getAllByText('Unavailable')).toHaveLength(2);
    const trash = screen.getByRole('button', { name: 'Trash' }).closest('tr')!;
    expect(within(trash).getByText('0', { exact: true })).toBeInTheDocument();
    await controller.refresh();
    expect(service.cleanupCategories).toHaveBeenCalledTimes(1);
    expect(service.cleanupFiles).not.toHaveBeenCalled();
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.previewCleanup).not.toHaveBeenCalled();
    expect(service.analyzeDeveloper).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('manual/protected views expose inspection without selection or removal controls', async () => {
    const { controller, service } = await setup();
    await fireEvent.click(
      await screen.findByRole('button', { name: 'User logs' }),
    );
    const heading = await screen.findByRole('heading', { name: 'User logs' });
    await waitFor(() => expect(heading).toHaveFocus());
    await screen.findByText('/Users/fixture/Library/Logs/needed.log');
    expect(screen.queryAllByRole('checkbox')).toHaveLength(0);
    expect(
      screen.queryByRole('button', { name: 'Check selected cache files' }),
    ).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Finder' }));
    expect(service.reveal).toHaveBeenCalledWith(
      '/Users/fixture/Library/Logs/needed.log',
    );
    expect(service.previewCleanup).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to categories' }),
    );
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'User logs' })).toHaveFocus(),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Trash' }));
    await screen.findByRole('heading', { name: 'Trash' });
    expect(screen.queryAllByRole('checkbox')).toHaveLength(0);
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('cache categories use shared, initially empty namespace selection and one fresh review', async () => {
    const { controller, service } = await setup();
    await fireEvent.click(
      await screen.findByRole('button', { name: 'App caches' }),
    );
    const checkbox = await screen.findByRole('checkbox', {
      name: 'Select all in example.namespace',
    });
    expect(checkbox).not.toBeChecked();
    await fireEvent.click(checkbox);
    await screen.findByText('3 files selected');
    expect(service.cacheSelect).toHaveBeenCalledWith(
      expect.objectContaining({
        query: expect.objectContaining({
          kind: 'userCaches',
          group: 'example.namespace',
        }),
        paths: null,
      }),
      expect.any(Function),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review selection' }),
    );
    await screen.findByRole('heading', { name: 'Review cache selection' });
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    expect(service.cacheExecute).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('ignores a late file page after the user changes category', async () => {
    let finish: (page: CleanupCategoryPage) => void = () => {};
    const { controller } = await setup({
      cleanupCategoryEntries: vi.fn((query) =>
        query.kind === 'userLogs'
          ? new Promise<CleanupCategoryPage>((resolve) => {
              finish = resolve;
            })
          : Promise.resolve({
              sessionId: 'fixture',
              kind: query.kind,
              files: [],
              nextOffset: null,
              qualification: 'Fixture',
            }),
      ),
    });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'User logs' }),
    );
    await screen.findByText('Loading indexed category files…');
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to categories' }),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Trash' }));
    finish({
      sessionId: 'fixture',
      kind: 'userCaches',
      files: [entry],
      nextOffset: null,
      qualification: 'Late cache page',
    });
    await waitFor(() =>
      expect(screen.queryByText(entry.path)).not.toBeInTheDocument(),
    );
    expect(screen.queryAllByRole('checkbox')).toHaveLength(0);
    await controller.dispose();
  });
});
