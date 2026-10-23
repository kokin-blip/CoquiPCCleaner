import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import Applications from './Applications.svelte';
import Storage from './Storage.svelte';
import RemovalReview from '../components/RemovalReview.svelte';
import Coverage from '../components/Coverage.svelte';
import type { ApplicationFootprint } from '../bindings/ApplicationFootprint';
import type { RemovalPlan } from '../bindings/RemovalPlan';
const footprint: ApplicationFootprint = {
  path: '/fixture/Example.app',
  name: 'Example',
  bundleId: 'org.example.fixture',
  version: '1.0',
  observedAt: '2026-10-23T00:00:00Z',
  protection: [],
  related: [
    {
      path: '/fixture/Library/Caches/org.example.fixture',
      association: 'Exact bundle ID',
      confidence: 'Bundle identity',
      logicalBytes: 0,
      allocatedBytes: 0,
      eligible: true,
      consequences:
        'Size measured at review; cached data may need regeneration.',
    },
    {
      path: '/fixture/Library/Containers/org.example.fixture',
      association: 'Unconfirmed container',
      confidence: 'Unconfirmed',
      logicalBytes: 0,
      allocatedBytes: 0,
      eligible: false,
      consequences: 'Shared data; no authority.',
    },
  ],
  qualification: 'Test fixture; no device measurements.',
};
const plan: RemovalPlan = {
  administrator: false,
  id: 'removal',
  createdAt: '2026-10-23T00:00:00Z',
  expiresAt: '2026-10-23T00:10:00Z',
  status: 'review',
  finishedAt: null,
  qualification: 'Fixture review',
  items: [
    {
      path: '/fixture/Downloads/unique.txt',
      kind: 'selectedFile',
      logicalBytes: 9,
      allocatedBytes: 4096,
      entryCount: 1,
      association: 'Explicit selection',
      consequences: 'May be unique work.',
      status: 'pending',
      detail: 'Awaiting approval',
      trashPath: null,
    },
  ],
};
describe('whole-disk and removal workflows', () => {
  it('starts related data unselected and gives shared data no selection authority', async () => {
    const service = fakeBackend({
      applicationPage: vi.fn(async () => ({
        applications: [footprint],
        offset: 0,
        total: 1,
        qualification: 'Fixture inventory',
      })),
      inspectApplication: vi.fn(async () => footprint),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(Applications, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect' }),
    );
    const bundle = await screen.findByRole('checkbox', {
      name: /Application bundle/,
    });
    expect(bundle).not.toBeChecked();
    const related = screen.getByRole('checkbox', {
      name: footprint.related[0].path,
    });
    expect(related).not.toBeChecked();
    expect(
      screen.getByRole('checkbox', { name: footprint.related[1].path }),
    ).toBeDisabled();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    await fireEvent.click(related);
    expect(
      screen.getByRole('button', { name: 'Review removal (1)' }),
    ).toBeEnabled();
    expect(screen.getByRole('heading', { name: 'Example' })).toHaveFocus();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to applications' }),
    );
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Inspect' })).toHaveFocus(),
    );
    await controller.dispose();
  });
  it('replaces Removal records with chosen inspection and returns focus to its origin', async () => {
    const service = fakeBackend({
      selectApplication: vi.fn(async () => footprint.path),
      inspectApplication: vi.fn(async () => footprint),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(Applications, { controller });
    await fireEvent.click(screen.getByRole('tab', { name: 'Removal records' }));
    await screen.findByRole('heading', {
      name: 'Removal and recovery records',
    });
    const choose = screen.getByRole('button', { name: 'Choose application…' });
    await waitFor(() => expect(choose).toBeEnabled());
    await fireEvent.click(choose);
    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Example' })).toHaveFocus(),
    );
    expect(
      screen.queryByRole('heading', { name: 'Removal and recovery records' }),
    ).toBeNull();
    expect(screen.getByRole('tab', { name: 'Installed Apps' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Back to applications' }),
    );
    await screen.findByRole('heading', {
      name: 'Removal and recovery records',
    });
    expect(choose).toHaveFocus();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('focuses the named contents of the second reviewed object with its exact path', async () => {
    const second = '/fixture/Downloads/second folder';
    const service = fakeBackend({
      previewRemoval: vi.fn(async () => ({
        ...plan,
        items: [
          ...plan.items,
          { ...plan.items[0], path: second, kind: 'selectedFolder' as const },
        ],
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(RemovalReview, {
      controller,
      request: { paths: [plan.items[0].path, second], application: null },
      onclose: () => {},
    });
    const inspect = await screen.findAllByRole('button', {
      name: 'Inspect reviewed contents',
    });
    await fireEvent.click(inspect[1]);
    const heading = screen.getByRole('heading', {
      name: 'Contents of second folder',
    });
    await waitFor(() => expect(heading).toHaveFocus());
    expect(
      screen.getByRole('table', { name: 'Contents of second folder' }),
    ).toHaveAccessibleDescription(second);
    await waitFor(() =>
      expect(service.removalEntries).toHaveBeenLastCalledWith(plan.id, 1, 0),
    );
    expect(
      screen.getByRole('button', { name: 'Move to Trash' }),
    ).toBeDisabled();
    expect(service.executeRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('requires exact-path acknowledgement before native Trash and retains the returned verification state', async () => {
    const execute = vi.fn(async (a: unknown, on: (p: RemovalPlan) => void) => {
      const done = {
        ...plan,
        status: 'complete',
        items: plan.items.map((i) => ({
          ...i,
          status: 'movedToTrash',
          detail: 'Fixture identity verified',
          trashPath: '/fixture/.Trash/unique.txt',
        })),
      };
      on(done);
      return done;
    });
    const service = fakeBackend({
      previewRemoval: vi.fn(async () => plan),
      executeRemoval: execute,
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(RemovalReview, {
      controller,
      request: { paths: [plan.items[0].path], application: null },
      onclose: () => {},
    });
    const move = await screen.findByRole('button', { name: 'Move to Trash' });
    await waitFor(() =>
      expect(screen.queryByText('Checking reviewed contents…')).toBeNull(),
    );
    expect(move).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed these exact paths/ }),
    );
    await fireEvent.click(move);
    await waitFor(() =>
      expect(execute).toHaveBeenCalledWith(
        { id: 'removal', paths: [plan.items[0].path], acknowledged: true },
        expect.any(Function),
      ),
    );
    expect(
      await screen.findByText(
        'Review status: complete. Rescan to refresh storage results.',
      ),
    ).toBeInTheDocument();
    await controller.dispose();
  });
  it('searches the full paginated index and proposes only explicitly checked Storage paths', async () => {
    const file = {
      path: '/fixture/Downloads/unique.txt',
      parent: '',
      name: 'unique.txt',
      category: 'Documents',
      logicalBytes: 9,
      allocatedBytes: 4096,
      isDirectory: false,
      modifiedAt: '',
      qualification: '',
    };
    const service = fakeBackend({
      history: vi.fn(async () => [{ ...session, status: 'complete' }]),
      files: vi.fn(async () => [file]),
      previewRemoval: vi.fn(async () => plan),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(Storage, { controller, onScan: () => {} });
    await screen.findByRole('button', { name: 'unique.txt' });
    await fireEvent.input(
      screen.getByRole('searchbox', { name: 'Search all indexed paths' }),
      { target: { value: 'Downloads' } },
    );
    await waitFor(() =>
      expect(service.files).toHaveBeenLastCalledWith(
        expect.objectContaining({
          search: 'Downloads',
          parent: null,
          offset: 0,
        }),
      ),
    );
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: `Select ${file.path} for removal review`,
      }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review removal (1)' }),
    );
    await waitFor(() =>
      expect(service.previewRemoval).toHaveBeenCalledWith({
        paths: [file.path],
        application: null,
      }),
    );
    await controller.dispose();
  });
  it('keeps completed traversal distinct from permission and cloud coverage omissions', async () => {
    const service = fakeBackend({
      coverage: vi.fn(async () => [
        {
          root: '/fixture',
          path: '/fixture/private',
          reason: 'permissionDenied',
        },
      ]),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.update((s) => ({
      ...s,
      session: {
        ...session,
        status: 'complete',
        coverage: [
          {
            root: '/fixture',
            state: 'completedWithOmissions',
            files: 3,
            visited: 5,
            skips: { permissionDenied: 1, cloudPlaceholder: 1 },
            examples: [],
          },
        ],
      },
    }));
    render(Coverage, { controller, id: session.id });
    expect(await screen.findByText('Permission denied')).toBeInTheDocument();
    expect(
      screen.getByText(/Traversal complete with omissions/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Open Full Disk Access settings' }),
    ).toBeEnabled();
    await controller.dispose();
  });
});
