<script lang="ts">
  import type { DiagnosticEvidence } from '../bindings/DiagnosticEvidence';
  import Status from './Status.svelte';
  import { bytes, date } from '../format';
  let { items }: { items: DiagnosticEvidence[] } = $props();
</script>

<div class="evidence-list">
  {#each items as item}
    <div class="evidence-row">
      <div>
        <strong>{item.label}</strong>
        <p class="muted small">{item.source} · {date(item.observedAt)}</p>
        {#if item.qualification}<p class="small muted">
            {item.qualification}
          </p>{/if}
      </div>
      <div class="evidence-value">
        <span
          >{item.unit === 'bytes'
            ? bytes(Number(item.value))
            : item.value}{item.unit && item.unit !== 'bytes'
            ? ` ${item.unit}`
            : ''}</span
        >{#if item.state !== 'supported'}<Status value={item.state} />{/if}
      </div>
    </div>
  {/each}
</div>
