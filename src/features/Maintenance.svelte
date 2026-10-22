<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { MaintenanceRule } from '../bindings/MaintenanceRule';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import type { MaintenanceEvent } from '../bindings/MaintenanceEvent';
  import { date } from '../format';
  import Status from '../components/Status.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let enabled = $state(false),
    mode = $state('reminder'),
    days = $state(7),
    threshold = $state(90),
    saved = $state<MaintenanceRule | null>(null),
    rows = $state<ActivityRecord[]>([]),
    detail = $state<MaintenanceEvent | null>(null),
    error = $state(''),
    loading = $state(true),
    message = $state('');
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    void load();
  });
  async function load() {
    loading = true;
    error = '';
    try {
      saved = await controller.service.maintenanceRule();
      if (saved) {
        enabled = saved.enabled;
        mode = saved.mode;
        days = saved.intervalDays;
        threshold = saved.capacityThresholdPercent;
      }
      rows = await controller.service.records('maintenance', 0);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function save() {
    const settings = $controller.settings;
    if (!settings) return;
    error = '';
    message = '';
    try {
      const rule: MaintenanceRule = {
        enabled,
        mode,
        intervalDays: days,
        capacityThresholdPercent: threshold,
        nextDueAt: new Date(Date.now() + days * 86400000).toISOString(),
        scope: {
          mode: settings.scanMode === 'selected' ? 'selected' : 'standard',
          roots: settings.scanMode === 'selected' ? settings.selectedRoots : [],
          excludedPaths: settings.excludedPaths,
        },
      };
      await controller.work('Save maintenance schedule', () =>
        controller.service.saveMaintenance(rule),
      );
      saved = rule;
      message = 'Schedule saved. Next occurrence starts from this save time.';
    } catch (e) {
      error = String(e);
    }
  }
  async function inspect(id: string) {
    try {
      const r = await controller.service.record('maintenance', id);
      if (r.kind === 'maintenance') detail = r.data;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-heading">
  <div>
    <h1>Maintenance</h1>
    <p class="muted">
      Optional read-only scans and review reminders, with a visible scope.
    </p>
  </div>
  <button onclick={load} disabled={loading}>Refresh</button>
</div>
{#if loading}<p role="status">
    Loading maintenance preferences…
  </p>{/if}{#if error}<p class="inline-error" role="alert">
    {error}<button onclick={load}>Retry</button>
  </p>{/if}{#if message}<p role="status" class="callout">{message}</p>{/if}
<section class="panel">
  <h2>Your schedule</h2>
  <p class="muted">
    Coqui checks every 30 seconds while open. A missed occurrence is handled
    once when the app reopens; nothing runs while it is closed. Busy operations
    defer the check. Cleanup always requires a separate manual review.
  </p>
  <form
    class="feature-form"
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
  >
    <label class="check-row"
      ><input
        type="checkbox"
        bind:checked={enabled}
        disabled={busy || loading}
      />Enable maintenance</label
    >
    <div class="form-grid">
      <label
        >Action<select bind:value={mode} disabled={busy || loading}
          ><option value="reminder"
            >Remind me to review storage and developer artifacts</option
          ><option value="scan">Run a read-only storage and system scan</option
          ></select
        ></label
      ><label
        >Every (days)<input
          type="number"
          min="1"
          max="365"
          required
          bind:value={days}
          disabled={busy || loading}
        /></label
      >
    </div>
    <label
      >Capacity alert threshold (%)<input
        type="number"
        min="50"
        max="99"
        required
        bind:value={threshold}
        disabled={busy || loading}
      /></label
    >
    <p class="small muted">
      Capacity alerts use filesystem-reported utilization of readable internal
      disks; APFS volumes can share capacity. No disk health or exact
      recoverable space is inferred.
    </p>
    <div class="schedule-scope">
      <strong>Scope copied from Settings when saved</strong>
      <p>
        {$controller.settings?.scanMode === 'selected'
          ? 'Selected folders'
          : 'Standard home-folder scan (deep scheduling is unavailable)'}
      </p>
      {#if $controller.settings?.scanMode === 'selected'}{#each $controller.settings.selectedRoots as path}<p
            class="path small"
          >
            {path}
          </p>{/each}{/if}
      <p class="muted small">
        Exclusions: {$controller.settings?.excludedPaths.join('; ') ||
          'None selected'}. New exclusions are also applied at execution.
      </p>
    </div>
    <button class="primary" disabled={busy || loading}
      >Save schedule and scope</button
    >
  </form>
</section>
{#if saved}<section class="panel">
    <div class="section-heading">
      <h2>{saved.enabled ? 'Maintenance enabled' : 'Maintenance paused'}</h2>
      <Status value={saved.enabled ? 'supported' : 'off'} />
    </div>
    <p>
      Next due: {date(saved.nextDueAt)} · {saved.mode} · every {saved.intervalDays}
      days
    </p>
    <p class="small muted">
      Saved scope: {saved.scope.mode} · {saved.scope.roots.join('; ') ||
        'Accessible standard home locations'}
    </p>
    <p class="small muted">
      Saved exclusions: {saved.scope.excludedPaths.join('; ') || 'None'}.
      Current exclusions are added at run time.
    </p>
  </section>{/if}
<section class="panel">
  <h2>Maintenance timeline</h2>
  {#each rows as row}<button class="history-row" onclick={() => inspect(row.id)}
      ><span class="history-description"
        ><strong>{row.title}</strong><span class="muted small"
          >{date(row.createdAt)}</span
        ></span
      ><Status value={row.status} /></button
    >{:else}<p class="muted">
      No due occurrences have been recorded.
    </p>{/each}{#if rows.length === 50}<p class="muted small">
      Latest 50 shown; older records are in History.
    </p>{/if}{#if detail}<h3>{date(detail.createdAt)}</h3>
    <p>{detail.message}</p>
    {#if detail.scanId}<p class="small path">Scan ID: {detail.scanId}</p>
      <button onclick={() => controller.navigate('History')}
        >Open scan history</button
      >{/if}{/if}
</section>
