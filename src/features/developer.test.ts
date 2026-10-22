import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { tick } from 'svelte';
import Optimization from './Optimization.svelte';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import type { DeveloperReport } from '../bindings/DeveloperReport';
const report: DeveloperReport = {
  sessionId: 'fixture',
  checkedAt: '2026-10-21T10:00:00Z',
  status: 'complete',
  examined: 1,
  candidateCount: 1,
  errors: [],
  qualification:
    'Review only. Clean Git status does not establish disposability.',
  artifacts: [
    {
      path: '/fixture/target',
      projectRoot: '/fixture',
      technology: 'Rust',
      kind: 'Build output',
      marker: 'Cargo.toml',
      indexedAllocatedBytes: 4096,
      indexedLogicalBytes: 32,
      indexedFileCount: 1,
      scannedAt: '2026-10-21T09:00:00Z',
      checkedAt: '2026-10-21T10:00:00Z',
      trackedEntries: 1,
      gitlinks: 0,
      nestedMarkers: false,
      assessment: 'Protected',
      reasons: ['Git tracks files in this output. Preserve them.'],
      consequences: 'Rebuilding can be expensive.',
      git: {
        status: 'dirty',
        root: '/fixture',
        gitDirectory: '/fixture/.git',
        commonDirectory: '/fixture/.git',
        changes: { staged: 0, modified: 1, untracked: 2, conflicts: 0 },
        qualification: 'Only the current worktree was checked.',
        worktrees: [
          {
            path: '/fixture',
            head: 'abcd',
            branch: 'refs/heads/main',
            detached: false,
            locked: false,
            prunable: false,
            bare: false,
            present: true,
          },
        ],
      },
    },
  ],
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: async () => [{ ...session, status: 'complete' }],
    developerReport: async () => structuredClone(report),
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  render(Optimization, { controller, onScan: vi.fn() });
  await fireEvent.click(
    screen.getByRole('tab', { name: 'Developer artifacts' }),
  );
  return { controller, service };
}
describe('developer artifact reviews', () => {
  it('reopens retained evidence and never proposes project mutation', async () => {
    const { service } = await setup();
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect /fixture/target' }),
    );
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'Git tracks files in this output. Preserve them.',
    );
    expect(screen.getByRole('dialog')).toHaveTextContent('refs/heads/main');
    expect(screen.getByRole('dialog')).toHaveTextContent('1 modified');
    expect(
      screen.queryByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeNull();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Reveal output' }),
    );
    expect(service.reveal).toHaveBeenCalledWith('/fixture/target');
    expect(service.executeCleanup).not.toHaveBeenCalled();
  });
  it('filters artifact paths and classifications with a truthful empty result', async () => {
    await setup();
    await screen.findByRole('button', { name: 'Inspect /fixture/target' });
    await fireEvent.input(
      screen.getByRole('searchbox', { name: 'Filter artifact paths' }),
      { target: { value: 'missing' } },
    );
    expect(
      await screen.findByText('No matching artifacts.'),
    ).toBeInTheDocument();
    await fireEvent.input(
      screen.getByRole('searchbox', { name: 'Filter artifact paths' }),
      { target: { value: '' } },
    );
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Assessment' }),
      { target: { value: 'Review required' } },
    );
    expect(
      await screen.findByText('No matching artifacts.'),
    ).toBeInTheDocument();
  });
  it('streams real progress, cancels and serializes validation with scans', async () => {
    let finish!: (value: DeveloperReport) => void;
    const analyzeDeveloper = vi.fn(async (_id, on) => {
      on({ ...report, status: 'running', artifacts: [] });
      return new Promise<DeveloperReport>((resolve) => (finish = resolve));
    });
    const { controller, service } = await setup({ analyzeDeveloper });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Recheck artifacts' }),
    );
    expect(get(controller).developerRunning).toBe(true);
    await controller.scan();
    expect(service.scan).not.toHaveBeenCalled();
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Cancel validation' }),
    );
    expect(service.cancelDeveloper).toHaveBeenCalled();
    finish({ ...report, status: 'cancelled' });
    await waitFor(() => expect(get(controller).developerRunning).toBe(false));
    expect(await screen.findByText('Cancelled')).toBeInTheDocument();
  });
  it('retries unavailable retained reports and presents unknown Git without clean claims', async () => {
    const developerReport = vi
      .fn()
      .mockRejectedValueOnce(new Error('Index unavailable'))
      .mockResolvedValue({
        ...report,
        artifacts: [
          {
            ...report.artifacts[0],
            git: {
              ...report.artifacts[0].git,
              status: 'unavailable',
              changes: null,
              worktrees: [],
              qualification: 'Git validation unavailable.',
            },
            trackedEntries: null,
          },
        ],
      });
    await setup({ developerReport });
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Index unavailable',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry loading review' }),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect /fixture/target' }),
    );
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'Git validation unavailable.',
    );
    expect(screen.getByRole('dialog')).toHaveTextContent('Unknown / 0');
  });
  it('keeps inspection gaps and interrupted state visible after restart', async () => {
    await setup({
      developerReport: async () => ({
        ...report,
        status: 'interrupted',
        errors: ['App closed before validation completed.'],
      }),
    });
    expect(await screen.findByText('Interrupted')).toBeInTheDocument();
    await fireEvent.click(screen.getByText('Inspection gaps (1)'));
    expect(
      screen.getByText('App closed before validation completed.'),
    ).toBeInTheDocument();
  });
  it('keeps an open review stable during unrelated live-metric updates', async () => {
    const developerReport = vi.fn(async () => structuredClone(report));
    const { controller } = await setup({ developerReport });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect /fixture/target' }),
    );
    controller.update((s) => ({ ...s, samples: [], monitoringStatus: 'live' }));
    await tick();
    await tick();
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'Git tracks files in this output. Preserve them.',
    );
    expect(developerReport).toHaveBeenCalledTimes(1);
  });
});
