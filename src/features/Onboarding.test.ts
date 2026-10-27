import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import App from '../App.svelte';
import { createAppStore } from '../state';
import { fakeBackend, settings } from '../testing/fake-backend';
import type { DiskScanPlan } from '../bindings/DiskScanPlan';

const plan: DiskScanPlan = {
  id: 'reviewed-onboarding-scope',
  createdAt: new Date().toISOString(),
  qualification: 'Disposable scope fixture; no collection.',
  volumes: [
    {
      id: 'internal',
      mount: '/fixture/internal',
      device: 'fixture0',
      filesystem: 'apfs',
      internal: true,
      readOnly: false,
      selected: false,
      qualification: '',
    },
    {
      id: 'external',
      mount: '/fixture/external',
      device: 'fixture1',
      filesystem: 'apfs',
      internal: false,
      readOnly: false,
      selected: false,
      qualification: '',
    },
  ],
};
async function privacy(overrides: Parameters<typeof fakeBackend>[0] = {}) {
  const service = fakeBackend({
    initialize: vi.fn(async () => ({
      ...settings,
      onboardingComplete: false,
      scanMode: 'deep',
    })),
    diskScanPlan: vi.fn(async () => plan),
    ...overrides,
  });
  const controller = createAppStore(service);
  render(App, { controller });
  await fireEvent.click(
    await screen.findByRole('button', { name: /Set up my computer/ }),
  );
  await fireEvent.click(
    screen.getByRole('checkbox', { name: 'Software development' }),
  );
  await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
  return { service, controller };
}

describe('first-launch analysis', () => {
  it('reviews internal volumes before scanning and submits only the confirmed scope', async () => {
    const { service, controller } = await privacy();
    await fireEvent.click(
      screen.getByRole('button', { name: /Start read-only analysis/ }),
    );
    expect(
      await screen.findByRole('dialog', {
        name: 'Review whole internal disk scope',
      }),
    ).toBeInTheDocument();
    expect(service.scan).not.toHaveBeenCalled();
    expect(
      await screen.findByRole('checkbox', { name: 'Scan /fixture/internal' }),
    ).toBeChecked();
    expect(
      screen.getByRole('checkbox', { name: 'Scan /fixture/external' }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Scan selected volumes' }),
    );
    await waitFor(() =>
      expect(service.scan).toHaveBeenCalledWith(
        expect.objectContaining({
          mode: 'deep',
          planId: plan.id,
          roots: ['/fixture/internal'],
        }),
        expect.any(Function),
        null,
        { system: true, startup: true, applications: true },
      ),
    );
    await controller.dispose();
  });

  it('cancels first-launch scope review without collecting', async () => {
    const { service, controller } = await privacy();
    await fireEvent.click(
      screen.getByRole('button', { name: /Start read-only analysis/ }),
    );
    await screen.findByRole('dialog', {
      name: 'Review whole internal disk scope',
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(service.scan).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('starts an explicitly selected standard scope without a volume-plan request', async () => {
    const { service, controller } = await privacy();
    await fireEvent.change(
      screen.getByRole('combobox', { name: 'Storage analysis scope' }),
      { target: { value: 'standard' } },
    );
    await fireEvent.click(
      screen.getByRole('button', { name: /Start read-only analysis/ }),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Start scan' }),
    );
    await waitFor(() =>
      expect(service.scan).toHaveBeenCalledWith(
        expect.objectContaining({ mode: 'standard', planId: null }),
        expect.any(Function),
        null,
        { system: true, startup: true, applications: true },
      ),
    );
    expect(service.diskScanPlan).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
