import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend } from '../testing/fake-backend';
import PhotoPreview from './PhotoPreview.svelte';
const OriginalURL = globalThis.URL;
beforeEach(() =>
  vi.stubGlobal(
    'URL',
    class extends OriginalURL {
      static createObjectURL = vi.fn(() => 'blob:fixture-photo');
      static revokeObjectURL = vi.fn();
    },
  ),
);
afterEach(() => vi.unstubAllGlobals());
describe('explicit photo preview', () => {
  it('reads only on request and releases pixels when closed', async () => {
    const service = fakeBackend({
      previewDuplicatePhoto: vi.fn(async () => ({
        pngBytes: [137, 80, 78, 71],
        width: 1,
        height: 1,
        qualification: 'Fixture pixels; no cleanup approval',
      })),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(PhotoPreview, {
      controller,
      reportId: 'report',
      path: '/fixture/photo.png',
    });
    expect(service.previewDuplicatePhoto).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Preview photo' }),
    );
    expect(
      await screen.findByRole('img', {
        name: 'Current preview of /fixture/photo.png',
      }),
    ).toHaveAttribute('src', 'blob:fixture-photo');
    expect(service.previewDuplicatePhoto).toHaveBeenCalledWith(
      'report',
      '/fixture/photo.png',
    );
    await fireEvent.click(
      screen.getByRole('button', { name: 'Close preview' }),
    );
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:fixture-photo');
    expect(service.previewDuplicates).not.toHaveBeenCalled();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });
  it('keeps failed access explicit without creating an image or removal review', async () => {
    const service = fakeBackend({
      previewDuplicatePhoto: vi.fn(async () => {
        throw new Error('Photo changed during preview read');
      }),
    });
    const controller = createAppStore(service);
    await controller.initialize();
    render(PhotoPreview, {
      controller,
      reportId: 'report',
      path: '/fixture/photo.png',
    });
    await fireEvent.click(
      screen.getByRole('button', { name: 'Preview photo' }),
    );
    await waitFor(() =>
      expect(screen.getByRole('alert')).toHaveTextContent(
        'Photo changed during preview read',
      ),
    );
    expect(URL.createObjectURL).not.toHaveBeenCalled();
    expect(service.previewDuplicates).not.toHaveBeenCalled();
    await controller.dispose();
  });
});
