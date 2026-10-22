import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend } from '../testing/fake-backend';
import Firmware from './Firmware.svelte';
import FirmwarePlanView from '../components/FirmwarePlanView.svelte';
import ActivityHistory from './ActivityHistory.svelte';
import type { FirmwareReport } from '../bindings/FirmwareReport';
import type { FirmwarePlan } from '../bindings/FirmwarePlan';
const report: FirmwareReport = {
  id: 'firmware-test',
  createdAt: '2026-10-22T10:00:00Z',
  finishedAt: '2026-10-22T10:00:01Z',
  status: 'partial',
  stage: 'Finished',
  request: {
    inventory: true,
    security: true,
    updates: false,
    networkAcknowledged: false,
  },
  platform: {
    manufacturer: 'Apple',
    model: 'Mac16,8',
    chip: 'Apple M4 Pro',
    architecture: 'aarch64',
    osVersion: '26.6.1',
    osBuild: 'test-build',
    eligible: true,
    qualification: 'Fixture platform, not a device measurement.',
  },
  evidence: [
    {
      label: 'FileVault',
      value: 'Unavailable',
      unit: '',
      source: 'fixture',
      observedAt: '2026-10-22T10:00:01Z',
      state: 'requiresPermission',
      qualification: 'Permission failure fixture; review manually.',
    },
  ],
  capabilities: [
    {
      name: 'Startup security policy',
      state: 'manualOnly',
      explanation: 'Inspect in Recovery; SIP does not establish this policy.',
    },
  ],
  findings: [],
  updates: [],
  updateState: 'notRequested',
  updateCheckedAt: null,
  errors: ['Permission failure fixture'],
  qualification: 'Saved observation, not a security certification.',
};
const plan: FirmwarePlan = {
  id: 'plan',
  createdAt: '2026-10-22T10:00:00Z',
  reportId: report.id,
  kind: 'softwareUpdate',
  status: 'awaitingAcknowledgement',
  title: 'Review an Apple software update',
  before: report.platform,
  previous: report.evidence,
  steps: ['Back up and review compatibility.'],
  cautions: ['Installation may restart your Mac.'],
  sourceUrl: 'https://support.apple.com/en-us/108382',
  sourceReviewedAt: '2026-10-22',
  acknowledgedAt: null,
  outcome: 'No action performed.',
  afterReportId: null,
  recovery: 'No rollback guarantee.',
};
async function setup(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend(overrides);
  const controller = createAppStore(service);
  await controller.initialize();
  return { controller, service };
}
describe('Phase 3 firmware workflows', () => {
  it('starts with every optional capability disabled and requires a fresh network acknowledgement per check', async () => {
    const { controller, service } = await setup({
      auditFirmware: vi.fn(async (r, on) => {
        const result = { ...report, request: r };
        on(result);
        return result;
      }),
    });
    render(Firmware, { controller });
    await fireEvent.click(screen.getByRole('tab', { name: 'Advanced audit' }));
    expect(service.auditFirmware).not.toHaveBeenCalled();
    expect(
      screen.getByRole('button', { name: 'Run selected audit' }),
    ).toBeDisabled();
    await fireEvent.click(screen.getByLabelText(/macOS update availability/));
    expect(
      screen.getByRole('button', { name: 'Run selected audit' }),
    ).toBeDisabled();
    await fireEvent.click(screen.getByLabelText(/Allow this macOS network/));
    await fireEvent.click(
      screen.getByRole('button', { name: 'Run selected audit' }),
    );
    await waitFor(() =>
      expect(service.auditFirmware).toHaveBeenCalledWith(
        {
          inventory: false,
          security: false,
          updates: true,
          networkAcknowledged: true,
        },
        expect.any(Function),
      ),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Run selected audit' }),
      ).toBeDisabled(),
    );
    expect(
      screen.getByLabelText(/macOS update availability/),
    ).not.toBeChecked();
    controller.dispose();
  });
  it('shows real streamed partial results, permission guidance and cancellation', async () => {
    let finish!: (r: FirmwareReport) => void;
    const { controller, service } = await setup({
      auditFirmware: vi.fn(async (_r, on) => {
        on({
          ...report,
          status: 'running',
          finishedAt: null,
          stage: 'Read FileVault',
        });
        return new Promise<FirmwareReport>((resolve) => (finish = resolve));
      }),
    });
    render(Firmware, { controller });
    await fireEvent.click(screen.getByRole('tab', { name: 'Advanced audit' }));
    await fireEvent.click(screen.getByLabelText(/Configuration audit/));
    await fireEvent.click(
      screen.getByRole('button', { name: 'Run selected audit' }),
    );
    await screen.findByText('Read FileVault');
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel audit' }));
    expect(service.cancelWork).toHaveBeenCalled();
    finish({ ...report, status: 'cancelled' });
    await screen.findByText('Cancelled');
    expect(
      screen.getByText('Permission failure fixture; review manually.'),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/not a security certification/),
    ).toBeInTheDocument();
    controller.dispose();
  });
  it('requires separate guidance acknowledgement before a manual update handoff and never claims installation success', async () => {
    const acknowledged = {
      ...plan,
      status: 'awaitingManualAction',
      acknowledgedAt: '2026-10-22T10:01:00Z',
    };
    const { controller, service } = await setup({
      acknowledgeFirmwarePlan: vi.fn(async () => acknowledged),
    });
    const onUpdate = vi.fn();
    const view = render(FirmwarePlanView, { controller, plan, onUpdate });
    expect(
      screen.queryByRole('button', { name: 'Open System Settings' }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Acknowledge manual guidance' }),
    ).toBeDisabled();
    await fireEvent.click(screen.getByLabelText(/I understand the scope/));
    await fireEvent.click(
      screen.getByRole('button', { name: 'Acknowledge manual guidance' }),
    );
    await waitFor(() => expect(onUpdate).toHaveBeenCalledWith(acknowledged));
    await view.rerender({ controller, plan: acknowledged, onUpdate });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Open System Settings' }),
    );
    expect(service.settings).toHaveBeenCalledWith('updates');
    expect(
      screen.getByRole('button', { name: 'Record new local observations' }),
    ).toBeDisabled();
    expect(
      screen.getByText(/Does not certify installation or reboot success/),
    ).toBeInTheDocument();
    controller.dispose();
  });
  it('reopens saved audits through activity history with uncertainty intact', async () => {
    const { controller } = await setup({
      records: async () => [
        {
          id: report.id,
          kind: 'firmware',
          createdAt: report.createdAt,
          status: 'partial',
          title: 'Firmware & security audit',
        },
      ],
      record: async () => ({ kind: 'firmware', data: report }),
    });
    render(ActivityHistory, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: /Firmware & security audit/ }),
    );
    expect(
      await screen.findByText('Fixture platform, not a device measurement.'),
    ).toBeInTheDocument();
    expect(screen.getByText('Update check not requested')).toBeInTheDocument();
    controller.dispose();
  });
  it('restores an audit without running it on page entry and offers retry after a load failure', async () => {
    const records = vi
      .fn()
      .mockRejectedValueOnce(new Error('Database busy'))
      .mockResolvedValue([
        {
          id: report.id,
          kind: 'firmware',
          createdAt: report.createdAt,
          status: 'partial',
          title: 'Firmware & security audit',
        },
      ]);
    const { controller, service } = await setup({
      records,
      record: async () => ({ kind: 'firmware', data: report }),
    });
    render(Firmware, { controller });
    await screen.findByText(/Database busy/);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Retry loading records' }),
    );
    await screen.findByText('Fixture platform, not a device measurement.');
    expect(service.auditFirmware).not.toHaveBeenCalled();
    controller.dispose();
  });
});
