<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { FirmwareComparison } from '../bindings/FirmwareComparison';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import FirmwareComparisonView from '../components/FirmwareComparisonView.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let rows = $state<ActivityRecord[]>([]),
    comparisons = $state<ActivityRecord[]>([]),
    result = $state<FirmwareComparison | null>(null);
  let before = $state(''),
    after = $state(''),
    beforeLabel = $state(''),
    afterLabel = $state(''),
    offset = $state(0),
    rawCount = $state(0),
    loading = $state(false),
    error = $state('');
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  const title = (r: ActivityRecord) =>
    `${date(r.createdAt)} · ${r.status} · ${r.id.slice(0, 8)}`;
  onMount(() => {
    void load();
  });
  async function load() {
    loading = true;
    error = '';
    try {
      const raw = await controller.service.records('firmware', offset);
      rawCount = raw.length;
      rows = raw.filter((r) => ['complete', 'partial'].includes(r.status));
      comparisons = await controller.service.records('firmwareComparison', 0);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function compare() {
    error = '';
    try {
      result = await controller.work('Compare saved firmware audits', () =>
        controller.service.compareFirmware(before, after),
      );
      comparisons = await controller.service.records('firmwareComparison', 0);
    } catch (e) {
      error = String(e);
    }
  }
  async function inspect(id: string) {
    error = '';
    try {
      const r = await controller.service.record('firmwareComparison', id);
      if (r.kind === 'firmwareComparison') result = r.data;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel feature-form">
  <h2>Compare saved audits</h2>
  <p class="muted">
    Choose an earlier Before and later After audit. Different values are
    observations, not proof that an installation or reboot succeeded. Partial
    scopes stay qualified.
  </p>
  {#if loading}<p role="status">Loading audit choices…</p>{/if}{#if error}<p
      role="alert"
      class="inline-error"
    >
      {error}<button onclick={load}>Retry loading</button>
    </p>{/if}
  <div class="firmware-comparison-context">
    {#each ['Before', 'After'] as side}<label
        >{side}<select
          aria-label={`${side} firmware audit`}
          value={side === 'Before' ? before : after}
          disabled={loading || busy}
          onchange={(e) => {
            const id = e.currentTarget.value;
            const selected = rows.find((r) => r.id === id);
            if (side === 'Before') {
              before = id;
              beforeLabel = selected ? title(selected) : '';
            } else {
              after = id;
              afterLabel = selected ? title(selected) : '';
            }
          }}
          ><option value="">Choose a saved audit</option
          >{#if (side === 'Before' ? before : after) && !rows.some((r) => r.id === (side === 'Before' ? before : after))}<option
              value={side === 'Before' ? before : after}
              >{side === 'Before' ? beforeLabel : afterLabel}</option
            >{/if}{#each rows as r}<option value={r.id}>{title(r)}</option
            >{/each}</select
        ></label
      >{/each}
  </div>
  <div class="button-row">
    <button
      disabled={loading || !offset}
      onclick={() => {
        offset = Math.max(0, offset - 50);
        void load();
      }}>Previous audits</button
    ><button
      disabled={loading || rawCount < 50}
      onclick={() => {
        offset += 50;
        void load();
      }}>Next audits</button
    ><button
      class="primary"
      disabled={busy || loading || !before || !after || before === after}
      onclick={compare}>Compare observations</button
    >
  </div>
  {#if !loading && !rows.length}<p class="empty-inline muted">
      No finished complete/partial audits on this page. Run an optional audit or
      choose another page.
    </p>{/if}
</section>
{#if result}<FirmwareComparisonView comparison={result} />{/if}
<section class="panel">
  <h2>Saved comparisons</h2>
  <p class="muted small">
    Latest 50. Older comparisons are in History → Activity.
  </p>
  {#each comparisons as r}<button
      class="history-row"
      onclick={() => inspect(r.id)}
      ><span class="history-description">{date(r.createdAt)}</span><span
        >{r.status}</span
      ></button
    >{:else}<p class="empty-inline muted">
      No audit comparisons saved yet.
    </p>{/each}
</section>
