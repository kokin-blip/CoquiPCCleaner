<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { Baseline } from '../bindings/Baseline';
  import type { BaselineComparison } from '../bindings/BaselineComparison';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let workload = $state(''),
    target = $state(''),
    context = $state('Same app/input, power source and background conditions'),
    duration = $state(60),
    report = $state<Baseline | null>(null),
    records = $state<ActivityRecord[]>([]),
    before = $state(''),
    after = $state(''),
    comparison = $state<BaselineComparison | null>(null),
    error = $state('');
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    workload = $controller.settings?.workload.workloads.join(', ') ?? '';
    void refresh();
  });
  async function refresh() {
    try {
      records = await controller.service.records('baseline', 0);
    } catch (e) {
      error = String(e);
    }
  }
  async function capture() {
    error = '';
    comparison = null;
    try {
      report = await controller.work('Workload baseline', () =>
        controller.service.captureBaseline(
          { workload, target, context, durationSeconds: duration },
          (r) => (report = r),
        ),
      );
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
  async function compare() {
    error = '';
    try {
      comparison = await controller.service.compareBaselines(before, after);
    } catch (e) {
      error = String(e);
    }
  }
  async function inspect(id: string) {
    error = '';
    try {
      const r = await controller.service.record('baseline', id);
      if (r.kind === 'baseline') report = r.data;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel">
  <h2>Measure a representative workload</h2>
  <p class="muted">
    Run the task yourself while Coqui samples every two seconds. Record matching
    conditions before and after a manual change; no benchmark score is inferred.
  </p>
  <form
    onsubmit={(e) => {
      e.preventDefault();
      void capture();
    }}
    class="feature-form"
  >
    <label
      >Workload<input
        required
        maxlength="2000"
        bind:value={workload}
        disabled={busy}
      /></label
    ><label
      >Expected task / target<input
        required
        maxlength="2000"
        placeholder="Build this project; note completion time separately"
        bind:value={target}
        disabled={busy}
      /></label
    ><label
      >Comparison conditions<textarea
        required
        maxlength="2000"
        bind:value={context}
        disabled={busy}></textarea></label
    ><label
      >Duration in seconds<input
        type="number"
        min="6"
        max="600"
        step="2"
        required
        bind:value={duration}
        disabled={busy}
      /></label
    ><button class="primary" disabled={busy}>Start measured interval</button>
  </form>
</section>
{#if error}<p class="inline-error" role="alert">
    {error}<button onclick={refresh}>Reload baselines</button>
  </p>{/if}
{#if report}<section class="panel" aria-live="polite">
    <div class="section-heading">
      <h2>{report.request.workload}</h2>
      <Status value={report.status} />
    </div>
    <p>
      {report.samples.length} samples · {date(report.createdAt)} · {report
        .request.durationSeconds}s requested
    </p>
    <p class="muted">{report.request.target} · {report.request.context}</p>
    <Evidence items={report.summary} />
    <details>
      <summary>Observed power settings</summary><Evidence
        items={report.power}
      />
    </details>
    <p class="muted small">{report.qualification}</p>
  </section>{/if}
<section class="panel">
  <h2>Before / after comparison</h2>
  <div class="form-grid">
    <label
      >Before<select bind:value={before}
        ><option value="">Choose a baseline</option
        >{#each records.filter((r) => r.status === 'complete') as r}<option
            value={r.id}>{date(r.createdAt)} · {r.title}</option
          >{/each}</select
      ></label
    ><label
      >After<select bind:value={after}
        ><option value="">Choose a baseline</option
        >{#each records.filter((r) => r.status === 'complete') as r}<option
            value={r.id}>{date(r.createdAt)} · {r.title}</option
          >{/each}</select
      ></label
    >
  </div>
  <button
    disabled={!before || !after || before === after || busy}
    onclick={compare}>Compare observations</button
  >{#if comparison}{#each comparison.observations as text}<p>{text}</p>{/each}
    <p class="muted small">{comparison.qualification}</p>{/if}
</section>
<section class="panel">
  <div class="section-heading">
    <h2>Saved baselines</h2>
    <button onclick={refresh}>Refresh</button>
  </div>
  {#each records as r}<button class="history-row" onclick={() => inspect(r.id)}
      ><span class="history-description"
        ><strong>{r.title}</strong><span class="muted small"
          >{date(r.createdAt)}</span
        ></span
      ><Status value={r.status} /></button
    >{:else}<p class="muted">
      No recorded baselines yet.
    </p>{/each}{#if records.length === 50}<p class="muted small">
      Latest 50 shown; older records are available in History.
    </p>{/if}
</section>
