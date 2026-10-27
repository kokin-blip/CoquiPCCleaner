import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import type { WorkloadTrial } from '../bindings/WorkloadTrial';
import type { TrialComparison } from '../bindings/TrialComparison';
import TaskTrials from './TaskTrials.svelte';
import WorkloadTrialView from '../components/WorkloadTrialView.svelte';
import Advisor from './Advisor.svelte';

const trial: WorkloadTrial = {
  id: 'trial-a',
  createdAt: '2026-10-23T12:00:00Z',
  finishedAt: '2026-10-23T12:00:08Z',
  status: 'complete',
  request: {
    workload: 'Software development',
    task: 'Fixture build',
    inputs: 'revision 1',
    application: 'compiler 1',
    conditions: 'fixture only',
    targetSeconds: 10,
  },
  elapsedSeconds: 8,
  completionReported: true,
  samples: [],
  power: [],
  qualification: 'Synthetic test fixture',
};
const group = {
  trials: [
    {
      id: 'trial-a',
      createdAt: trial.createdAt,
      elapsedSeconds: 8,
      targetMet: true,
    },
  ],
  medianSeconds: 8,
  minimumSeconds: 8,
  maximumSeconds: 8,
  targetMetCount: 1,
};
const comparison: TrialComparison = {
  id: 'comparison',
  createdAt: trial.createdAt,
  request: {
    beforeIds: ['trial-a'],
    afterIds: ['trial-b'],
    change: 'Fixture change',
  },
  protocol: trial.request,
  before: group,
  after: group,
  medianChangePercent: 0,
  matchingConditions: true,
  limitations: ['Too few repeated trials'],
  qualification: 'Synthetic comparison',
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: async () => [{ ...session, status: 'complete' }],
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  return { service, controller };
}
async function fill() {
  for (const [name, value] of [
    ['Workload', 'Software development'],
    ['Task', 'Fixture build'],
    ['Inputs / project', 'revision 1'],
    ['Application / tool version', 'compiler 1'],
    ['Comparison conditions', 'fixture only'],
  ]) {
    await fireEvent.input(screen.getByRole('textbox', { name }), {
      target: { value },
    });
  }
  await fireEvent.input(screen.getByRole('spinbutton'), {
    target: { value: '10' },
  });
}
describe('measured task results', () => {
  it('starts only explicitly and keeps completion available after leaving the view', async () => {
    let resolve!: (t: WorkloadTrial) => void;
    const { service, controller } = await setup({
      captureTrial: vi.fn(async (_request, on) => {
        on({
          ...trial,
          status: 'running',
          finishedAt: null,
          completionReported: false,
          elapsedSeconds: 0,
        });
        return new Promise<WorkloadTrial>((r) => {
          resolve = r;
        });
      }),
      finishTrial: vi.fn(async () => {
        resolve(trial);
      }),
    });
    const view = render(TaskTrials, { controller });
    await screen.findByText(
      'No task trials recorded yet. Start a timer above; an interval baseline remains available in Baselines.',
    );
    expect(service.captureTrial).not.toHaveBeenCalled();
    await fill();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Start task timer' }),
    );
    await screen.findByRole('button', { name: 'Task completed — stop timer' });
    expect(service.captureTrial).toHaveBeenCalledWith(
      trial.request,
      expect.any(Function),
    );
    view.unmount();
    controller.navigate('Storage');
    expect(get(controller).workRunning).toBe(true);
    await controller.finishTrial();
    await waitFor(() => expect(get(controller).workRunning).toBe(false));
    expect(service.finishTrial).toHaveBeenCalledWith('trial-a');
    render(TaskTrials, { controller });
    await screen.findByText(/This trial met the declared target/);
    await expect(controller.finishTrial()).rejects.toThrow('No task timer');
    expect(service.finishTrial).toHaveBeenCalledTimes(1);
    await controller.dispose();
  });
  it('preserves form input and marks lost transport as unverified', async () => {
    const { service, controller } = await setup({
      captureTrial: vi.fn(async (_request, on) => {
        on({
          ...trial,
          status: 'running',
          completionReported: false,
          finishedAt: null,
        });
        throw new Error('Connection lost');
      }),
    });
    render(TaskTrials, { controller });
    await fill();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Start task timer' }),
    );
    await screen.findByRole('alert');
    expect(screen.getByRole('textbox', { name: 'Task' })).toHaveValue(
      'Fixture build',
    );
    expect(screen.queryByText(/This trial met the declared target/)).toBeNull();
    expect(get(controller).workloadTrial?.status).toBe('unverified');
    expect(service.cancelWork).toHaveBeenCalledTimes(1);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Reload saved trials' }),
    );
    await waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
    await controller.dispose();
  });
  it('keeps groups exclusive, excludes incomplete trials and saves only by a separate action', async () => {
    const { service, controller } = await setup({
      records: vi.fn(async () =>
        ['a', 'b', 'c'].map((id) => ({
          id: `trial-${id}`,
          kind: 'workloadTrial',
          title: `Build ${id}`,
          createdAt: trial.createdAt,
          status: id === 'c' ? 'cancelled' : 'complete',
        })),
      ),
      compareTrials: vi.fn(async () => comparison),
      saveTrialComparison: vi.fn(async () => comparison),
    });
    render(TaskTrials, { controller });
    const before = await screen.findByRole('checkbox', {
      name: /^Before: Build a/,
    });
    const after = screen.getByRole('checkbox', { name: /^After: Build a/ });
    expect(screen.queryByRole('checkbox', { name: /Build c/ })).toBeNull();
    await fireEvent.click(before);
    await fireEvent.click(after);
    expect(before).not.toBeChecked();
    await fireEvent.click(
      screen.getByRole('checkbox', { name: /^Before: Build b/ }),
    );
    await fireEvent.input(
      screen.getByRole('textbox', { name: 'Comparison / intended change' }),
      { target: { value: 'Fixture change' } },
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Compare selected trials' }),
    );
    await screen.findByRole('heading', { name: 'Recorded task comparison' });
    expect(service.compareTrials).toHaveBeenCalledWith({
      beforeIds: ['trial-b'],
      afterIds: ['trial-a'],
      change: 'Fixture change',
    });
    expect(service.saveTrialComparison).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Save comparison to History' }),
    );
    expect(
      await screen.findByRole('button', {
        name: 'Comparison saved to History',
      }),
    ).toBeDisabled();
    expect(service.saveTrialComparison).toHaveBeenCalledTimes(1);
    await controller.dispose();
  });
  it('does not display a cancelled or missing interval as a met target', () => {
    const view = render(WorkloadTrialView, {
      trial: { ...trial, status: 'cancelled' },
    });
    expect(screen.queryByText(/This trial met the declared target/)).toBeNull();
    view.unmount();
    render(WorkloadTrialView, { trial: { ...trial, elapsedSeconds: null } });
    expect(screen.getByText('Not recorded')).toBeInTheDocument();
    expect(screen.queryByText(/This trial met the declared target/)).toBeNull();
  });
  it('includes only an explicitly selected task result in an advisor request', async () => {
    const { service, controller } = await setup({
      records: vi.fn(async (kind) =>
        kind === 'workloadTrial'
          ? [
              {
                id: trial.id,
                kind,
                createdAt: trial.createdAt,
                title: 'Fixture build',
                status: 'complete',
              },
            ]
          : [],
      ),
      assess: vi.fn(async () => ({
        id: 'advice',
        createdAt: trial.createdAt,
        workload: 'Software development',
        assessment: 'Fixture review',
        findings: [],
        scenarios: [],
        sources: [],
        sourceReviews: [],
        verificationLimits: [],
        qualification: 'Fixture',
      })),
    });
    render(Advisor, { controller });
    const selected = await screen.findByRole('option', {
      name: /Fixture build/,
    });
    expect(selected).toBeInTheDocument();
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Measured task result' }),
      { target: { value: trial.id } },
    );
    await fireEvent.input(screen.getByRole('textbox', { name: 'Workload' }), {
      target: { value: 'Software development' },
    });
    await fireEvent.input(
      screen.getByRole('textbox', { name: 'Expected task result' }),
      { target: { value: 'Build under ten seconds' } },
    );
    await fireEvent.submit(
      screen.getByRole('textbox', { name: 'Workload' }).closest('form')!,
    );
    await waitFor(() =>
      expect(service.assess).toHaveBeenCalledWith(
        expect.objectContaining({
          trialId: trial.id,
          workload: 'Software development',
        }),
      ),
    );
    await controller.dispose();
  });
});
