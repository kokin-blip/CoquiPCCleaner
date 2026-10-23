import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend, settings } from '../testing/fake-backend';
import HelperSettings from '../components/HelperSettings.svelte';
import RemovalReview from '../components/RemovalReview.svelte';
import Applications from './Applications.svelte';
import type { RemovalPlan } from '../bindings/RemovalPlan';
const plan: RemovalPlan = {
  administrator: true,
  id: 'root-preview',
  createdAt: '2026-10-23T00:00:00Z',
  expiresAt: '2026-10-23T00:10:00Z',
  status: 'review',
  finishedAt: null,
  qualification: 'Disposable test fixture, not a device result',
  items: [
    {
      path: '/fixture/Root owned folder',
      kind: 'selectedFolder',
      logicalBytes: 12,
      allocatedBytes: 4096,
      entryCount: 2,
      association: 'Explicit fixture selection',
      consequences: 'May contain unique work',
      status: 'pending',
      detail: 'Awaiting approval',
      trashPath: null,
    },
  ],
};
describe('optional administrator workflows', () => {
  it('locks an authorized session locally without starting helper work', async () => {
    const service = fakeBackend({ administratorStatus: vi.fn(async () => ({ authorizationSeconds: 900, installed: true, compatible: true, version: '0.8.0', protocol: 2, qualification: 'Matching fixture' })) });
    const controller = createAppStore(service);
    await controller.initialize();
    render(HelperSettings, { controller });
    await fireEvent.click(screen.getByRole('button', { name: 'Check installed helper' }));
    const lock = await screen.findByRole('button', { name: 'Lock administrator access' });
    await fireEvent.click(lock);
    await waitFor(() => expect(service.administratorLock).toHaveBeenCalledOnce());
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.administratorLifecycle).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: 'Lock administrator access' })).not.toBeInTheDocument();
    await controller.dispose();
  });
  it('starts off without contacting the helper, and checking compatibility does not start collection', async () => {
    const service = fakeBackend({
      administratorStatus: vi.fn(async () => ({
        installed: true,
        compatible: true,
        version: '0.8.0',
        protocol: 2,
        authorizationSeconds: 0,
        qualification: 'Matching test fixture',
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(HelperSettings, { controller });
    const toggle = screen.getByRole('switch', {
      name: /Offer administrator review/,
    });
    expect(toggle).not.toBeChecked();
    expect(toggle).toBeDisabled();
    expect(service.administratorStatus).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check installed helper' }),
    );
    await waitFor(() => expect(toggle).toBeEnabled());
    expect(toggle).not.toBeChecked();
    await fireEvent.click(toggle);
    await waitFor(() =>
      expect(service.save).toHaveBeenCalledWith(
        expect.objectContaining({ administratorEnabled: true }),
      ),
    );
    expect(service.scan).not.toHaveBeenCalled();
    expect(service.administratorPreviewRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('requires an explicit administrator preview and new acknowledgement after portable failure', async () => {
    const service = fakeBackend({
      previewRemoval: vi.fn(async () => {
        throw new Error('Only user-owned objects support portable removal');
      }),
      administratorPreviewRemoval: vi.fn(async () => plan),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.update((s) => ({
      ...s,
      settings: { ...settings, administratorEnabled: true },
    }));
    render(RemovalReview, {
      controller,
      request: { paths: [plan.items[0].path], application: null },
      onclose: () => {},
    });
    await screen.findByRole('alert');
    expect(service.administratorPreviewRemoval).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Prepare administrator review' }),
    );
    await screen.findByText(plan.qualification);
    expect(service.administratorPreviewRemoval).toHaveBeenCalledWith({
      paths: [plan.items[0].path],
      application: null,
    });
    expect(
      screen.getByRole('checkbox', { name: /I reviewed these exact paths/ }),
    ).not.toBeChecked();
    expect(
      screen.getByRole('button', { name: 'Move to Trash' }),
    ).toBeDisabled();
    expect(service.executeRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('keeps helper journal purge separate and waits for its confirmation', async () => {
    const service = fakeBackend({
      administratorStatus: vi.fn(async () => ({
        installed: true,
        compatible: true,
        version: '0.8.0',
        protocol: 2,
        authorizationSeconds: 0,
        qualification: 'Fixture',
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(HelperSettings, { controller });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Check installed helper' }),
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Remove helper records…' }),
    );
    expect(service.administratorLifecycle).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', {
        name: 'Remove helper records',
      }),
    );
    await waitFor(() =>
      expect(service.administratorLifecycle).toHaveBeenCalledWith('purge'),
    );
    expect(service.purge).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('inspects an app with administrator access only when requested, leaving selections unchecked', async () => {
    const app = {
      path: '/fixture/Example.app',
      name: 'Example',
      bundleId: 'org.coqui.fixture',
      version: '1',
      observedAt: '2026-10-23T00:00:00Z',
      protection: ['Administrator-owned bundle'],
      related: [],
      qualification: 'Fixture',
    };
    const service = fakeBackend({
      applicationPage: vi.fn(async () => ({
        applications: [app],
        offset: 0,
        total: 1,
        qualification: 'Fixture',
      })),
      inspectApplication: vi.fn(async () => app),
      administratorInspectApplication: vi.fn(async () => ({
        ...app,
        protection: [],
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    controller.update((s) => ({
      ...s,
      settings: { ...settings, administratorEnabled: true },
    }));
    render(Applications, { controller });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Inspect' }),
    );
    expect(service.administratorInspectApplication).not.toHaveBeenCalled();
    expect(
      screen.getByRole('checkbox', { name: /Application bundle/ }),
    ).toBeDisabled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Inspect with administrator' }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('checkbox', { name: /Application bundle/ }),
      ).toBeEnabled(),
    );
    expect(
      screen.getByRole('checkbox', { name: /Application bundle/ }),
    ).not.toBeChecked();
    expect(service.previewRemoval).not.toHaveBeenCalled();
    expect(service.administratorPreviewRemoval).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
