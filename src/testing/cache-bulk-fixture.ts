import { vi } from 'vitest';
import { fakeBackend, session, settings } from './fake-backend';
import type { CacheBulkSummary } from '../bindings/CacheBulkSummary';
import type { CacheBulkItem } from '../bindings/CacheBulkItem';
export const cacheFiles: CacheBulkItem[] = [1, 2, 3].map((n) => ({
  path: `/Users/fixture/Library/Caches/example.namespace/file-${n}`,
  group: 'example.namespace',
  allocatedBytes: 4096,
  logicalBytes: 10,
  modifiedAt: '2026-10-27T12:00:00Z',
  selected: false,
  eligible: false,
  status: 'indexed',
  detail: 'Illustrative indexed candidate',
  trashPath: null,
}));
export const cacheSummary: CacheBulkSummary = {
  id: 'bulk-fixture',
  sessionId: 'fixture',
  revision: 1,
  status: 'draft',
  createdAt: '2026-10-27T12:00:00Z',
  reviewedAt: null,
  expiresAt: null,
  selectedCount: 0,
  processedCount: 0,
  eligibleCount: 0,
  refusedCount: 0,
  movedCount: 0,
  skippedCount: 0,
  failedCount: 0,
  unverifiedCount: 0,
  unattemptedCount: 0,
  allocatedBytes: 0,
  reviewedBytes: 0,
  detail: 'Illustrative fixture; no files moved.',
};
export function cacheBackend(
  overrides: Parameters<typeof fakeBackend>[0] = {},
) {
  let selected = new Set<string>();
  let summary = { ...cacheSummary };
  const group = () => ({
    key: 'example.namespace',
    title: 'example.namespace',
    fileCount: 3,
    allocatedBytes: 12288,
    selectedCount: selected.size,
    eligibleCount: summary.eligibleCount,
    refusedCount: summary.refusedCount,
  });
  return fakeBackend({
    initialize: async () => ({ ...settings, monitoringEnabled: false }),
    history: async () => [{ ...session, status: 'partial' }],
    cacheGroups: vi.fn(async () => ({ groups: [group()], nextCursor: null })),
    cacheEntries: vi.fn(async (q) => ({
      items: cacheFiles
        .slice(q.cursor ? 2 : 0, q.cursor ? 3 : 2)
        .map((row) => ({ ...row, selected: selected.has(row.path) })),
      nextCursor: q.cursor ? null : cacheFiles[1].path,
    })),
    cacheSelect: vi.fn(async (request, on) => {
      if (request.clear) selected.clear();
      else
        for (const path of request.paths ?? cacheFiles.map((row) => row.path)) {
          if (request.selected) selected.add(path);
          else selected.delete(path);
        }
      summary = {
        ...cacheSummary,
        revision: summary.revision + 1,
        selectedCount: selected.size,
        allocatedBytes: selected.size * 4096,
        unattemptedCount: selected.size,
      };
      on({ ...summary });
      return { ...summary };
    }),
    cachePrepare: vi.fn(async (_id, _rev, on) => {
      summary = {
        ...summary,
        status: 'review',
        processedCount: selected.size,
        eligibleCount: selected.size,
        reviewedBytes: selected.size * 4096,
        reviewedAt: new Date().toISOString(),
        expiresAt: new Date(Date.now() + 600000).toISOString(),
      };
      on({ ...summary });
      return { ...summary };
    }),
    cacheReviewedGroups: vi.fn(async () => ({
      groups: [group()],
      nextCursor: null,
    })),
    cacheBatchPage: vi.fn(async () => ({
      items: cacheFiles
        .filter((row) => selected.has(row.path))
        .map((row) => ({
          ...row,
          selected: true,
          eligible: true,
          status: summary.status === 'complete' ? 'movedToTrash' : 'eligible',
          detail: 'Illustrative review outcome',
        })),
      nextCursor: null,
    })),
    cacheExecute: vi.fn(async (_a, on) => {
      summary = {
        ...summary,
        status: 'complete',
        movedCount: selected.size,
        unattemptedCount: 0,
      };
      on({ ...summary });
      return { ...summary };
    }),
    cacheDecline: vi.fn(async () => ({ ...summary, status: 'declined' })),
    ...overrides,
  });
}
