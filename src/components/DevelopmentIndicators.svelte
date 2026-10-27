<script lang="ts">
  import { CodeXml, ChevronDown } from '@lucide/svelte';
  import type { DevelopmentIndicator } from '../bindings/DevelopmentIndicator';
  let { indicators = [] }: { indicators?: DevelopmentIndicator[] } = $props();
</script>

{#if indicators.length}
  <details class="development-indicators">
    <summary aria-label="Development and deletion evidence">
      <CodeXml size={13} aria-hidden="true" />
      {#each indicators.slice(0, 2) as indicator}
        <span class="indicator" class:attention={indicator.attention}>{indicator.label}</span>
      {/each}
      {#if indicators.length > 2}<span class="muted small">+{indicators.length - 2} {indicators.length === 3 ? 'check' : 'checks'}</span>{/if}
      <ChevronDown size={12} aria-hidden="true" />
    </summary>
    <ul>
      {#each indicators as indicator}
        <li><strong>{indicator.label}</strong><p>{indicator.detail}</p></li>
      {/each}
    </ul>
  </details>
{/if}

<style>
  .development-indicators { margin-top: 6px; font-size: var(--font-size-label, 12px); }
  summary { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; width: fit-content; min-height: 24px; cursor: pointer; }
  summary::-webkit-details-marker { display: none; }
  summary :global(svg) { color: var(--secondary); flex-shrink: 0; }
  .indicator { border: 1px solid var(--border); border-radius: 5px; padding: 1px 5px; color: var(--secondary); background: var(--tag); }
  .indicator.attention { color: var(--warning); background: var(--warning-wash); border-color: transparent; }
  summary:hover .indicator { border-color: var(--secondary); }
  summary:focus-visible { outline: 3px solid var(--focus); outline-offset: 3px; border-radius: 5px; }
  ul { list-style: none; margin: 10px 0; padding: 0; max-width: 70ch; }
  li + li { margin-top: 10px; }
  p { color: var(--secondary); margin: 3px 0 0; overflow-wrap: anywhere; font-weight: 400; line-height: 1.5; }
</style>
