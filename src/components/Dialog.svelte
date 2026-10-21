<script lang="ts">
  import { X } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  let {
    title,
    onclose,
    children,
  }: { title: string; onclose: () => void; children: Snippet } = $props();
  let element: HTMLDialogElement;
  $effect(() => {
    element?.showModal();
    return () => element?.close();
  });
</script>

<dialog
  bind:this={element}
  class="modal"
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
  {@render children()}
</dialog>
