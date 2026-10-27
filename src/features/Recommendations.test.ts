import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import Recommendations from './Recommendations.svelte';
import Dashboard from './Dashboard.svelte';
import Analysis from './Analysis.svelte';
import AskCoqui from './AskCoqui.svelte';
import type { RecommendationPlan } from '../bindings/RecommendationPlan';
import type { Investigation } from '../bindings/Investigation';
const plan: RecommendationPlan = {
  id: 'plan',
  createdAt: '2026-10-23T12:00:00Z',
  sessionId: 'fixture',
  baselineId: null,
  trialId: null,
  preference: 'Balanced',
  actions: [
    {
      id: 'capacity',
      priority: 1,
      title: 'Review capacity',
      explanation: 'Disposable capacity fixture',
      classification: 'Requires careful review',
      affectedWorkloads: ['Coding'],
      expectedBenefit: 'Identify workspace needs',
      confidence: 'Observed capacity',
      risk: 'May contain unique work',
      reversibility: 'Separate Trash review',
      permission: 'Readable scope only',
      restart: 'No restart',
      verification: 'Rescan after approved changes',
      destination: 'Storage',
      evidence: [],
    },
  ],
  workloads: [
    {
      workload: 'Coding',
      status: 'Needs representative measurement',
      explanation: 'No performance conclusion',
      evidence: [],
      nextTest: 'Capture a baseline',
    },
  ],
  bottlenecks: [],
  qualification: 'Saved evidence is not execution authority',
};
const investigation: Investigation = {
  id: 'investigation',
  createdAt: '',
  title: 'Storage review',
  sessionId: 'fixture',
  baselineId: null,
  messages: [],
  tasks: [],
  findings: [],
  actions: [],
  status: 'evidenceReady',
  qualification: 'Fixture',
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    history: async () => [{ ...session, status: 'partial' }],
    recommendationPlan: vi.fn(async () => structuredClone(plan)),
    ...overrides,
  });
  const controller = createAppStore(service);
  await controller.initialize();
  return { service, controller };
}
describe('evidence-based review plans', () => {
  it('exposes risks and navigates to review without creating an approval', async () => {
    const { controller, service } = await setup();
    render(Recommendations, { controller });
    await screen.findByRole('heading', { name: 'Review capacity' });
    await fireEvent.click(screen.getByText('Evidence, risk and verification'));
    expect(screen.getByText('May contain unique work')).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Review in Storage' }),
    );
    expect(get(controller).page).toBe('Storage');
    expect(service.previewRemoval).not.toHaveBeenCalled();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    expect(service.saveRecommendationPlan).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('saves only on request, and preserves the explicitly selected baseline', async () => {
    const { controller, service } = await setup({
      records: vi.fn(async (kind) =>
        kind === 'workloadTrial'
          ? [
              {
                id: 'task-result',
                kind,
                title: 'Build result',
                createdAt: '',
                status: 'complete',
              },
            ]
          : [
              {
                id: 'chosen',
                kind: 'baseline',
                title: 'Coding fixture',
                createdAt: '',
                status: 'complete',
              },
            ],
      ),
      saveRecommendationPlan: vi.fn(async () => plan),
    });
    render(Recommendations, { controller });
    await screen.findByRole('heading', { name: 'Review capacity' });
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Representative baseline' }),
      { target: { value: 'chosen' } },
    );
    await waitFor(() =>
      expect(service.recommendationPlan).toHaveBeenLastCalledWith(
        'fixture',
        'chosen',
        null,
      ),
    );
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Measured task result' }),
      { target: { value: 'task-result' } },
    );
    await waitFor(() =>
      expect(service.recommendationPlan).toHaveBeenLastCalledWith(
        'fixture',
        'chosen',
        'task-result',
      ),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Save plan to History' }),
    );
    await waitFor(() =>
      expect(service.saveRecommendationPlan).toHaveBeenCalledWith(
        'fixture',
        'chosen',
        'task-result',
      ),
    );
    expect(
      await screen.findByRole('button', { name: 'Saved to History' }),
    ).toBeDisabled();
    await controller.dispose();
  });
  it('keeps the plan stable across unrelated live monitoring updates', async () => {
    const { controller, service } = await setup();
    render(Recommendations, { controller });
    await screen.findByRole('heading', { name: 'Review capacity' });
    controller.update((s) => ({
      ...s,
      monitoringStatus: 'starting',
      samples: [],
    }));
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(service.recommendationPlan).toHaveBeenCalledTimes(1);
    await controller.dispose();
  });
  it('does not label a terminal legacy scan as still running', async () => {
    const { controller } = await setup();
    render(Analysis, { controller, onScan: () => {} });
    expect(screen.getByText(/Finish time not recorded/)).toBeInTheDocument();
    expect(screen.queryByText(/Analysis in progress/)).toBeNull();
    await controller.dispose();
  });
  it('submits the dashboard question once to the local guide', async () => {
    const { controller, service } = await setup({
      ask: vi.fn(async () => investigation),
    });
    const dashboard = render(Dashboard, { controller, onScan: () => {} });
    await fireEvent.click(screen.getByText('Ask about a problem'));
    await fireEvent.input(
      screen.getByRole('textbox', {
        name: 'What problem are you experiencing with your computer?',
      }),
      { target: { value: 'Why is startup slow?' } },
    );
    await fireEvent.submit(screen.getByRole('textbox').closest('form')!);
    expect(get(controller).page).toBe('Ask Coqui');
    dashboard.unmount();
    render(AskCoqui, { controller });
    await waitFor(() =>
      expect(service.ask).toHaveBeenCalledWith(
        null,
        'fixture',
        'Why is startup slow?',
      ),
    );
    expect(service.ask).toHaveBeenCalledTimes(1);
    expect(get(controller).askDraft).toBe('');
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.collectInvestigation).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('runs only the explicitly requested read-only task and renders streamed evidence', async () => {
    const complete = {
      ...investigation,
      tasks: [
        {
          name: 'ReviewResources',
          status: 'complete',
          evidence: [],
          detail: 'Actual indexed scope inspected',
        },
      ],
    };
    const { controller, service } = await setup({
      ask: vi.fn(async () => investigation),
      collectInvestigation: vi.fn(async (_id, _task, on) => {
        on(complete);
        return complete;
      }),
    });
    controller.startInvestigation('Review storage');
    render(AskCoqui, { controller });
    await screen.findByRole('heading', { name: 'Gather fresh evidence' });
    expect(service.collectInvestigation).not.toHaveBeenCalled();
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Read-only task' }),
      { target: { value: 'resources' } },
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Run selected read-only task' }),
    );
    await waitFor(() =>
      expect(service.collectInvestigation).toHaveBeenCalledWith(
        'investigation',
        { kind: 'resources' },
        expect.any(Function),
      ),
    );
    expect(
      await screen.findByText('Actual indexed scope inspected'),
    ).toBeInTheDocument();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
