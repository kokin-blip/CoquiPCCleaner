<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { createAppStore } from '../state';
  let {
    controller,
    reportId,
    path,
  }: {
    controller: ReturnType<typeof createAppStore>;
    reportId: string;
    path: string;
  } = $props();
  let src = $state(''),
    error = $state(''),
    qualification = $state('');
  let revision = 0;
  const busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  function close() {
    revision++;
    if (src) URL.revokeObjectURL(src);
    src = '';
    error = '';
  }
  onDestroy(close);
  async function preview() {
    const current = ++revision;
    error = '';
    try {
      const image = await controller.work('Read selected photo preview', () =>
        controller.service.previewDuplicatePhoto(reportId, path),
      );
      if (current !== revision) return;
      if (src) URL.revokeObjectURL(src);
      src = URL.createObjectURL(
        new Blob([new Uint8Array(image.pngBytes)], { type: 'image/png' }),
      );
      qualification = image.qualification;
    } catch (e) {
      if (current === revision) error = String(e);
    }
  }
</script>

<div class="photo-preview">
  <p class="path small">{path}</p>
  {#if src}<figure>
      <img {src} alt={`Current preview of ${path}`} />
      <figcaption class="muted small">{qualification}</figcaption>
    </figure>
    <button onclick={close}>Close preview</button>
  {:else}<button disabled={busy} onclick={preview}>Preview photo</button>{/if}
  {#if error}<p class="inline-error small" role="alert">{error}</p>{/if}
</div>

<style>
  .photo-preview {
    min-width: 0;
  }
  figure {
    margin: 8px 0;
  }
  img {
    display: block;
    max-width: 100%;
    max-height: 320px;
    object-fit: contain;
  }
  figcaption {
    margin-top: 8px;
    max-width: 65ch;
  }
</style>
