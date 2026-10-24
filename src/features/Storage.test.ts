import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import Storage from './Storage.svelte';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import type { FileEntry } from '../bindings/FileEntry';
import type { StorageMap } from '../bindings/StorageMap';
import type { StorageMapNode } from '../bindings/StorageMapNode';

const debug: FileEntry = {
  path: '/fixture/project/target/debug',
  parent: '',
  name: 'debug',
  category: 'Other',
  logicalBytes: 4_000_000_000,
  allocatedBytes: 3_000_000_000,
  isDirectory: true,
  modifiedAt: '',
  qualification: 'Disposable test data only.',
};
const node: StorageMapNode = {
  entry: debug,
  fileCount: 120_000,
  smallFileCount: 110_000,
  smallAllocatedBytes: 2_200_000_000,
  largestFileBytes: 4_000_000,
  kind: 'buildOutput',
  title: 'Rust build output',
  reason: 'A target/debug folder within a recognized Rust output layout.',
  consequences:
    'Rebuilding may take time. Recognition does not authorize cleanup.',
  qualification: 'Metadata-derived context only.',
};
const map: StorageMap = {
  nodes: [node],
  totalAllocatedBytes: 4_000_000_000,
  totalLogicalBytes: 5_000_000_000,
  fileCount: 160_000,
  remainderAllocatedBytes: 1_000_000_000,
  remainderLogicalBytes: 1_000_000_000,
  remainderFileCount: 40_000,
  qualification:
    'All indexed immediate children contribute to these totals; remainder is included.',
};

async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: vi.fn(async () => [
      {
        ...session,
        status: 'complete',
        fileCount: 160_000,
        allocatedBytes: map.totalAllocatedBytes,
      },
    ]),
    files: vi.fn(async () => [debug]),
    storageMap: vi.fn(async () => map),
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  render(Storage, { controller, onScan: vi.fn() });
  return { controller, service };
}

describe('folder-first storage explorer', () => {
  it('shows aggregate small files beside ranked folders without granting cleanup authority', async () => {
    const { controller, service } = await setup();
    expect(
      await screen.findByRole('table', { name: 'Ranked folder contents' }),
    ).toBeInTheDocument();
    const rankedContents = within(
      screen.getByRole('table', { name: 'Ranked folder contents' }),
    );
    expect(
      await rankedContents.findByText('110,000 small files'),
    ).toBeInTheDocument();
    expect(rankedContents.getByText('2.2 GB allocated')).toBeInTheDocument();
    expect(await screen.findByText(/together occupy/)).toHaveTextContent('2.2 GB');
    expect(
      screen.getByText(/Recognition does not authorize cleanup/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Review removal (0)' }),
    ).toBeDisabled();
    expect(
      screen.getByRole('checkbox', {
        name: `Select ${debug.path} for removal review`,
      }),
    ).not.toBeChecked();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('drills down from the folder map and breadcrumb, using the same full-index scope as the table', async () => {
    const { controller, service } = await setup();
    await fireEvent.click(
      await screen.findByRole('button', { name: /Explore folder debug,/ }),
    );
    await waitFor(() =>
      expect(service.storageMap).toHaveBeenLastCalledWith({
        sessionId: session.id,
        parent: debug.path,
        metric: 'allocated',
        limit: 60,
      }),
    );
    expect(service.files).toHaveBeenLastCalledWith(
      expect.objectContaining({
        parent: debug.path,
        filesOnly: false,
        offset: 0,
      }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to previous folder' }),
    );
    await waitFor(() =>
      expect(service.storageMap).toHaveBeenLastCalledWith(
        expect.objectContaining({ parent: '' }),
      ),
    );
    await fireEvent.click(
      within(
        screen.getByRole('table', { name: 'Ranked folder contents' }),
      ).getByRole('button', { name: 'debug' }),
    );
    await waitFor(() =>
      expect(service.storageMap).toHaveBeenLastCalledWith(
        expect.objectContaining({ parent: debug.path }),
      ),
    );
    await fireEvent.click(
      within(
        screen.getByRole('navigation', { name: 'Storage folder path' }),
      ).getByRole('button', { name: 'project' }),
    );
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({ parent: '/fixture/project' }),
      ),
    );
    await controller.dispose();
  });

  it('switches logical and allocated weights and preserves an explicit remainder independent of table pagination', async () => {
    const entries = Array.from({ length: 100 }, (_, i) => ({
      ...debug,
      path: `/fixture/${i}`,
      name: `folder-${i}`,
    }));
    const { controller, service } = await setup({
      files: vi.fn(async () => entries),
    });
    await screen.findByRole('button', {
      name: /Browse more entries, 1 GB allocated/,
    });
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Storage size metric' }),
      { target: { value: 'logical' } },
    );
    await waitFor(() =>
      expect(service.storageMap).toHaveBeenLastCalledWith(
        expect.objectContaining({ metric: 'logical', limit: 60 }),
      ),
    );
    expect(service.files).toHaveBeenLastCalledWith(
      expect.objectContaining({ sort: 'logical', descending: true }),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: 100 }),
      ),
    );
    expect(service.storageMap).toHaveBeenCalledTimes(2);
    await fireEvent.click(
      screen.getByRole('button', { name: /Browse more entries, 1 GB logical/ }),
    );
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: 0 }),
      ),
    );
    expect(
      screen.getByRole('heading', { name: 'Ranked contents' }),
    ).toHaveFocus();
    await controller.dispose();
  });

  it('keeps Large files separate from folder aggregates and retries a map failure', async () => {
    const storageMap = vi
      .fn()
      .mockRejectedValueOnce(new Error('Saved folder index is unavailable'))
      .mockResolvedValue(map);
    const { controller, service } = await setup({ storageMap });
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Saved folder index is unavailable',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry folder map' }),
    );
    await screen.findByRole('button', { name: /Explore folder debug,/ });
    await fireEvent.click(screen.getByRole('tab', { name: 'Large files' }));
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({ filesOnly: true, parent: null }),
      ),
    );
    expect(screen.queryByRole('heading', { name: 'Folder map' })).toBeNull();
    await controller.dispose();
  });

  it('ignores late folder-map responses after navigation instead of presenting stale totals', async () => {
    let resolveRoot: (value: StorageMap) => void = () => {};
    const oldRoot = new Promise<StorageMap>((resolve) => {
      resolveRoot = resolve;
    });
    const nested = {
      ...map,
      nodes: [{ ...node, entry: { ...debug, name: 'Nested output' } }],
    };
    const { controller, service } = await setup({
      storageMap: vi.fn((q) =>
        q.parent === '' ? oldRoot : Promise.resolve(nested),
      ),
    });
    await fireEvent.click(await screen.findByRole('button', { name: 'debug' }));
    await screen.findByRole('button', {
      name: /Explore folder Nested output,/,
    });
    resolveRoot(map);
    await waitFor(() => expect(service.storageMap).toHaveBeenCalledTimes(2));
    expect(
      screen.queryByRole('button', { name: /Explore folder debug,/ }),
    ).toBeNull();
    expect(
      screen.getByRole('button', { name: /Explore folder Nested output,/ }),
    ).toBeInTheDocument();
    await controller.dispose();
  });

  it('discloses unfinished and legacy aggregates without displaying false zero cache or folder findings', async () => {
    const legacy = {
      ...map,
      nodes: [
        {
          ...node,
          entry: { ...debug, allocatedBytes: 0, logicalBytes: 0 },
          fileCount: null,
          smallFileCount: null,
          smallAllocatedBytes: null,
          largestFileBytes: null,
        },
      ],
      totalAllocatedBytes: 0,
      totalLogicalBytes: 0,
      fileCount: null,
      remainderAllocatedBytes: 0,
      remainderLogicalBytes: 0,
      remainderFileCount: null,
    };
    const { controller } = await setup({
      history: vi.fn(async () => [
        { ...session, status: 'partial', fileCount: 10_000 },
      ]),
      storageMap: vi.fn(async () => legacy),
      files: vi.fn(async () => [legacy.nodes[0].entry]),
    });
    expect(
      await screen.findByText('Folder totals unavailable'),
    ).toBeInTheDocument();
    expect(screen.getByText('Unavailable')).toBeInTheDocument();
    expect(
      within(
        screen.getByRole('table', { name: 'Ranked folder contents' }),
      ).queryByText(/small files/),
    ).toBeNull();
    expect(
      screen.getByText(/File counts and small-file totals are unavailable/),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('tab', { name: 'Cache candidates (0)' }),
    ).toBeNull();
    await fireEvent.click(
      screen.getByRole('tab', { name: 'Cache candidates' }),
    );
    expect(
      screen.getByRole('heading', {
        name: 'Candidate results unavailable.',
      }),
    ).toBeInTheDocument();
    await controller.dispose();
  });
});
