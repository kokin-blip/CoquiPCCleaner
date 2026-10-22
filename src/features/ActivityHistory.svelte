<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import type { RetainedRecord } from '../bindings/RetainedRecord';
  import RecordDetails from '../components/RecordDetails.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let rows = $state<ActivityRecord[]>([]),
    detail = $state<RetainedRecord | null>(null),
    filter = $state(''),
    offset = $state(0),
    loading = $state(false),
    error = $state('');
  onMount(() => {
    void load();
  });
  async function load() {
    loading = true;
    error = '';
    try {
      rows = await controller.service.records(filter || null, offset);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function inspect(row: ActivityRecord) {
    error = '';
    try {
      detail = await controller.service.record(row.kind, row.id);
    } catch (e) {
      error = String(e);
    }
  }
  const names: Record<string, string> = {
    firmware: 'Firmware & security audits',
    firmwarePlan: 'Apple manual guidance plans',
    duplicates: 'Duplicate reports',
    baseline: 'Workload baselines',
    manualPlan: 'Manual settings plans',
    maintenance: 'Maintenance',
    investigation: 'Ask Coqui',
    advice: 'Workload advice',
    resources: 'Resource reviews',
  };
</script>

<div class="section-heading">
  <label
    >Record type<select
      bind:value={filter}
      onchange={() => {
        offset = 0;
        void load();
      }}
      ><option value="">All activity</option
      >{#each Object.entries(names) as [value, label]}<option {value}
          >{label}</option
        >{/each}</select
    ></label
  ><button disabled={loading} onclick={load}>Refresh</button>
</div>
{#if loading}<p role="status">Loading retained activity…</p>{/if}{#if error}<p
    class="inline-error"
    role="alert"
  >
    {error}<button onclick={load}>Retry</button>
  </p>{/if}
<section class="panel">
  {#each rows as row}<button class="history-row" onclick={() => inspect(row)}
      ><span class="history-description"
        ><strong>{row.title}</strong><span class="muted small"
          >{names[row.kind] ?? row.kind} · {date(row.createdAt)}</span
        ></span
      ><Status value={row.status} /></button
    >{:else}{#if !loading}<div class="empty-state compact">
        <h2>No activity records.</h2>
        <p>
          Investigations, measurements, manual plans and maintenance outcomes
          appear here. Cleanup outcomes are in Optimization → Action history.
        </p>
      </div>{/if}{/each}
</section>
<div class="button-row">
  <button
    disabled={loading || offset === 0}
    onclick={() => {
      offset = Math.max(0, offset - 50);
      void load();
    }}>Previous</button
  ><button
    disabled={loading || rows.length < 50}
    onclick={() => {
      offset += 50;
      void load();
    }}>Next</button
  >
</div>
{#if detail}<Dialog
    title="Retained diagnostic record"
    onclose={() => (detail = null)}
    ><RecordDetails
      {controller}
      record={detail}
      onUpdate={(r) => {
        detail = r;
        void load();
      }}
    />
    <div class="dialog-actions">
      <button onclick={() => (detail = null)}>Close</button>
    </div></Dialog
  >{/if}
