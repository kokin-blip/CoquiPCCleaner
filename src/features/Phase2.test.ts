import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import AskCoqui from './AskCoqui.svelte';
import Duplicates from './Duplicates.svelte';
import Maintenance from './Maintenance.svelte';
import ManualPlans from './ManualPlans.svelte';
import Baselines from './Baselines.svelte';
import type { Investigation } from '../bindings/Investigation';
import type { DuplicateReport } from '../bindings/DuplicateReport';
import type { ManualPlan } from '../bindings/ManualPlan';
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: async () => [{ ...session, status: 'complete' }],
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  return { controller, service };
}
const investigation: Investigation = {
  baselineId: null,
  id: 'ask',
  createdAt: '2026-10-22T12:00:00Z',
  title: 'Storage',
  sessionId: 'fixture',
  messages: [
    { role: 'user', text: 'Review storage', createdAt: '' },
    {
      role: 'assistant',
      text: 'Saved scoped observations, not a confirmed cause.',
      createdAt: '',
    },
  ],
  tasks: [
    {
      name: 'ReviewStorageEvidence',
      status: 'complete',
      evidence: [],
      detail: 'Saved scan only',
    },
  ],
  findings: [],
  actions: ['Optimization'],
  status: 'evidenceReady',
  qualification: 'Local guide; no external AI',
};
const report: DuplicateReport = {
  request: null,
  scopeRoot: null,
  id: 'duplicates',
  sessionId: 'fixture',
  createdAt: '',
  status: 'complete',
  stage: 'Finished',
  examined: 2,
  total: 2,
  hashedBytes: 20,
  groups: [
    {
      id: 'group',
      digest: 'fixture',
      files: [
        {
          reviewSupported: true,
          reviewQualification: 'Fixture personal folder',
          path: '/Users/fixture/Documents/keep',
          logicalBytes: 10,
          allocatedBytes: 4096,
          modifiedAt: '',
        },
        {
          reviewSupported: true,
          reviewQualification: 'Fixture personal folder',
          path: '/Users/fixture/Documents/copy',
          logicalBytes: 10,
          allocatedBytes: 4096,
          modifiedAt: '',
        },
      ],
    },
  ],
  photoMatches: [],
  gaps: [],
  qualification: 'Fixture content verification',
};
describe('remaining Phase 2 workflows', () => {
  it('saves local investigations, exposes evidence context and hands actions to review pages', async () => {
    const { controller, service } = await setup({
      ask: vi.fn(async () => investigation),
    });
    render(AskCoqui, { controller });
    await fireEvent.input(
      screen.getByRole('textbox', { name: 'Your question' }),
      { target: { value: 'Review storage' } },
    );
    await fireEvent.submit(
      screen.getByRole('textbox', { name: 'Your question' }).closest('form')!,
    );
    expect(
      await screen.findByText(
        'Saved scoped observations, not a confirmed cause.',
      ),
    ).toBeInTheDocument();
    expect(service.ask).toHaveBeenCalledWith(null, 'fixture', 'Review storage');
    expect(screen.getByText('Saved scan only')).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Open Optimization' }),
    );
    expect(get(controller).page).toBe('Optimization');
    await controller.dispose();
  });
  it('requires an explicit keeper and selected removal copies before duplicate review', async () => {
    const { controller, service } = await setup({
      analyzeDuplicateScope: vi.fn(async (_request, on) => {
        on(report);
        return report;
      }),
    });
    render(Duplicates, { controller, onReview: vi.fn() });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Analyze contents' }),
    );
    const action = await screen.findByRole('button', {
      name: 'Review selected copies',
    });
    expect(action).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'Review removal of /Users/fixture/Documents/copy',
      }),
    );
    expect(action).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('radio', { name: 'Keep /Users/fixture/Documents/keep' }),
    );
    expect(action).toBeEnabled();
    expect(
      screen.getByRole('checkbox', {
        name: 'Review removal of /Users/fixture/Documents/keep',
      }),
    ).toBeDisabled();
    await fireEvent.click(action);
    await waitFor(() =>
      expect(service.previewDuplicates).toHaveBeenCalledWith({
        reportId: 'duplicates',
        groupId: 'group',
        keepPath: '/Users/fixture/Documents/keep',
        removePaths: ['/Users/fixture/Documents/copy'],
      }),
    );
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('keeps maintenance disabled by default and saves an explicit read-only scope', async () => {
    const { controller, service } = await setup();
    render(Maintenance, { controller });
    const toggle = await screen.findByRole('checkbox', {
      name: 'Enable maintenance',
    });
    await waitFor(() => expect(toggle).toBeEnabled());
    expect(toggle).not.toBeChecked();
    await fireEvent.click(toggle);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Save schedule and scope' }),
    );
    await waitFor(() =>
      expect(service.saveMaintenance).toHaveBeenCalledWith(
        expect.objectContaining({
          enabled: true,
          mode: 'reminder',
          scope: {
            planId: null,
            mode: 'selected',
            roots: ['/fixture'],
            excludedPaths: [],
          },
        }),
      ),
    );
    expect(
      screen.getByText(/nothing runs while it is closed/),
    ).toBeInTheDocument();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('does not record a manual change until the user explicitly acknowledges it', async () => {
    const plan: ManualPlan = {
      id: 'manual',
      createdAt: '',
      kind: 'power',
      requested: 'Inspect Balanced mode',
      previous: [],
      steps: ['Inspect current setting'],
      status: 'awaitingManualAction',
      observed: [],
      verification: 'Not applied',
      recovery: 'Manual restoration only',
    };
    const { controller, service } = await setup({
      manualPlan: vi.fn(async () => plan),
      verifyManualPlan: vi.fn(async () => ({
        ...plan,
        status: 'manuallyReported',
      })),
    });
    render(ManualPlans, { controller });
    await fireEvent.click(
      screen.getByRole('button', {
        name: 'Record plan and previous observations',
      }),
    );
    const verify = await screen.findByRole('button', {
      name: 'Record new observations',
    });
    expect(verify).toBeDisabled();
    expect(service.verifyManualPlan).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('checkbox', {
        name: 'I performed the described manual step myself.',
      }),
    );
    await fireEvent.click(verify);
    await waitFor(() =>
      expect(service.verifyManualPlan).toHaveBeenCalledWith('manual', true),
    );
    await controller.dispose();
  });
  it('streams measurement samples and clears busy state after cancellation/failure', async () => {
    let release!: (value: never) => void;
    const { controller, service } = await setup({
      captureBaseline: vi.fn(
        async () =>
          new Promise<never>((_resolve, reject) => (release = reject)),
      ),
      cancelWork: vi.fn(async () =>
        release(new Error('Cancelled fixture') as never),
      ),
    });
    render(Baselines, { controller });
    await fireEvent.input(
      screen.getByRole('textbox', { name: 'Expected task / target' }),
      { target: { value: 'Build fixture' } },
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Start measured interval' }),
    );
    await waitFor(() => expect(get(controller).workRunning).toBe(true));
    await service.cancelWork();
    await waitFor(() => expect(get(controller).workRunning).toBe(false));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Cancelled fixture',
    );
    await controller.dispose();
  });
});
