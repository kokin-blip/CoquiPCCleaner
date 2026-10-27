<script lang="ts">
  import { X } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  let {
    title,
    onclose,
    children,
    footer,
  }: {
    title: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();
  let element: HTMLDialogElement;
  $effect(() => {
    element?.showModal();
    return () => element?.close();
  });
</script>

<dialog
  bind:this={element}
  class="modal"
  class:with-footer={!!footer}
  aria-label={title}
  {onclose}
  onclick={(e) => {
    if (e.target === element) element.close();
  }}
  onkeydown={(e) => {
    if (e.key === 'Escape') element.close();
  }}
>
  <div class="section-heading">
    <h2>{title}</h2>
    <button
      class="icon-button"
      aria-label="Close dialog"
      onclick={() => element.close()}><X size={18} /></button
    >
  </div>
  {#if footer}<div class="dialog-body">{@render children()}</div>
    <div class="dialog-footer">
      {@render footer()}
    </div>{:else}{@render children()}{/if}
</dialog>

<style>
  .with-footer[open] {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .with-footer > .section-heading,
  .dialog-footer {
    flex: 0 0 auto;
  }
  .dialog-body {
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
  }
  .dialog-footer {
    padding-top: 14px;
  }
  .dialog-footer :global(.dialog-actions) {
    margin-top: 0;
  }
</style>
