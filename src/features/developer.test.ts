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
      development: [],
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
        recovery: null,
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

const projectPlan = {
  projectCleanup: true, duplicateCleanup:false,
  id: 'project-review',
  sessionId: 'fixture',
  createdAt: '2026-10-21T10:01:00Z',
  expiresAt: '2026-10-21T10:11:00Z',
  approvedAt: null,
  finishedAt: null,
  status: 'review',
  allocatedBytes: 4096,
  verification: 'Not executed.',
  items: [
    {
      file: {
        path: '/fixture/target',
        allocatedBytes: 4096,
        logicalBytes: 32,
        modifiedAt: '2026-10-21T10:00:00Z',
        origin:
          'Rust · fresh inspected folder · clean confirmed checkout · main HEAD abcdef',
        consequences: 'The entire folder moves. Rebuild with project tooling.',
      },
      status: 'pending',
      detail: 'Awaiting folder approval',
      trashPath: null,
    },
  ],
};
const cleanReport = () => ({
  ...structuredClone(report),
  artifacts: [
    {
      ...structuredClone(report.artifacts[0]),
      assessment: 'Review required',
      trackedEntries: 0,
      reasons: ['Review contents and regeneration.'],
      git: { ...structuredClone(report.artifacts[0].git), status: 'clean' },
    },
  ],
});
const contentPage = {
  total: 1,
  entries: [
    {
      path: '/fixture/target/build.cache',
      directory: false,
      logicalBytes: 32,
      allocatedBytes: 4096,
    },
  ],
};
async function projectSetup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  return setup({
    developerReport: async () => cleanReport(),
    previewProjectCleanup: vi.fn(async () => structuredClone(projectPlan)),
    projectCleanupEntries: vi.fn(async () => structuredClone(contentPage)),
    declineCleanup: vi.fn(async () => ({ ...projectPlan, status: 'declined' })),
    ...overrides,
  });
}
async function openFolderReview() {
  await fireEvent.click(
    await screen.findByRole('button', { name: 'Inspect /fixture/target' }),
  );
  await fireEvent.click(
    screen.getByRole('button', { name: 'Review folder cleanup' }),
  );
  return screen.findByRole('dialog', {
    name: 'Move 1 selected folder to Trash?',
  });
}
describe('project-folder cleanup approval', () => {
  it('reviews exact current contents and declines without mutation', async () => {
    const { service } = await projectSetup();
    await openFolderReview();
    expect(
      await screen.findByText('/fixture/target/build.cache'),
    ).toBeInTheDocument();
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'clean confirmed checkout',
    );
    expect(service.previewProjectCleanup).toHaveBeenCalledWith({
      sessionId: 'fixture',
      paths: ['/fixture/target'],
    });
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', { name: 'Keep folder' }));
    expect(service.declineCleanup).toHaveBeenCalledWith('project-review');
    expect(service.executeCleanup).not.toHaveBeenCalled();
  });
  it('requires stopped tools and regeneration acknowledgement then presents verified folder results', async () => {
    const done = {
      ...projectPlan,
      status: 'complete',
      verification:
        'Original folder absent; same directory and descendants verified in Trash.',
      items: projectPlan.items.map((i) => ({
        ...i,
        status: 'movedToTrash',
        detail: 'All reviewed descendant identities verified.',
        trashPath: '/fixture/.Trash/target',
      })),
    };
    const { service } = await projectSetup({
      executeCleanup: vi.fn(async (_approval, on) => {
        on(done);
        return done;
      }),
    });
    await openFolderReview();
    await screen.findByText('/fixture/target/build.cache');
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I have stopped builds/ }),
    );
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed the entire folder/ }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    );
    await screen.findByText('1 of 1 folders verified in Trash.');
    expect(service.executeCleanup).toHaveBeenCalledWith(
      {
        planId: 'project-review',
        reviewedAt: projectPlan.createdAt,
        applicationsClosed: true,
        consequencesAccepted: true,
      },
      expect.any(Function),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Inspect folder results' }),
    );
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'All reviewed descendant identities verified.',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Show in Trash' }),
    );
    expect(service.reveal).toHaveBeenCalledWith('/fixture/.Trash/target');
  });
  it('keeps failed or missing reviewed contents unapprovable and supports retry', async () => {
    const entries = vi
      .fn()
      .mockRejectedValueOnce(new Error('Inspection record unavailable'))
      .mockResolvedValue(contentPage);
    await projectSetup({ projectCleanupEntries: entries });
    await openFolderReview();
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Inspection record unavailable',
    );
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I have stopped builds/ }),
    );
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /I reviewed the entire folder/ }),
    );
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry contents' }),
    );
    await screen.findByText('/fixture/target/build.cache');
    expect(
      screen.getByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeEnabled();
  });
  it('cancels preparation, serializes scans, and refuses stale Git without offering execution', async () => {
    let reject!: (error: Error) => void;
    const { controller, service } = await projectSetup({
      previewProjectCleanup: vi.fn(
        async () =>
          new Promise<typeof projectPlan>((_resolve, rejectFn) => {
            reject = rejectFn;
          }),
      ),
    });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect /fixture/target' }),
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review folder cleanup' }),
    );
    await waitFor(() => expect(get(controller).cleanupPreparing).toBe(true));
    await controller.scan();
    expect(service.scan).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Cancel review preparation' }),
    );
    expect(service.cancelCleanup).toHaveBeenCalled();
    reject(new Error('Git changed; protected'));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Nothing was moved',
    );
    expect(get(controller).cleanupPreparing).toBe(false);
    expect(
      screen.queryByRole('button', { name: 'Approve and move to Trash' }),
    ).toBeNull();
  });
  it('reopens project action records with manual recovery and their consequences', async () => {
    await projectSetup({
      cleanupHistory: async () => [
        {
          ...projectPlan,
          status: 'interrupted',
          verification: 'Inspect manually before retrying.',
        },
      ],
    });
    await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
    await fireEvent.click(
      await screen.findByRole('button', { name: /Project folder review/ }),
    );
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'The entire folder moves.',
    );
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'Finder may offer Put Back',
    );
  });
});

it('presents legacy declined project items as terminal while preserving their evidence', async () => {
  await projectSetup({
    cleanupHistory: async () => [
      {
        ...projectPlan,
        status: 'declined',
        verification: 'Declined. No files were changed.',
      },
    ],
  });
  await fireEvent.click(screen.getByRole('tab', { name: 'Action history' }));
  await fireEvent.click(
    await screen.findByRole('button', { name: /Project folder review/ }),
  );
  const dialog = screen.getByRole('dialog');
  expect(dialog).toHaveTextContent(
    'Not moved; review declined. Create a fresh review for another attempt.',
  );
  expect(dialog).not.toHaveTextContent('Pending');
  expect(dialog).not.toHaveTextContent('Awaiting folder approval');
  expect(dialog).toHaveTextContent('main HEAD abcdef');
  expect(
    screen.queryByRole('button', { name: 'Approve and move to Trash' }),
  ).toBeNull();
});

describe('development safety indicators', () => {
  it('expands evidence without turning a cached remote match into deletion approval', async () => {
    const current = structuredClone(report);
    current.artifacts[0].development = [
      { label: 'Build / tool output', attention: false, detail: 'Recognized pattern; regeneration is unverified.' },
      { label: 'Review before deleting', attention: true, detail: 'Review unique contents before deleting.' },
      { label: 'Matches cached remote', attention: false, detail: 'GitHub/remote was not contacted. Ignored files are not backed up.' },
    ];
    const { service, controller } = await setup({ developerReport: async () => current });
    const summary = await screen.findByLabelText('Development and deletion evidence');
    const disclosure = summary.closest('details')!;
    await fireEvent.click(disclosure.querySelector('summary')!);
    expect(disclosure).toHaveTextContent('Ignored files are not backed up.');
    expect(disclosure).toHaveTextContent('Review before deleting');
    expect(screen.queryByRole('button', { name: /safe to delete/i })).toBeNull();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
