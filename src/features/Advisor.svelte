<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { AdviceReport } from '../bindings/AdviceReport';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import Evidence from '../components/Evidence.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let workload = $state(''),
    expectation = $state(''),
    baselineId = $state(''),
    demand = $state(60),
    growth = $state(10),
    low = $state<number | undefined>(),
    high = $state<number | undefined>(),
    source = $state(''),
    records = $state<ActivityRecord[]>([]),
    report = $state<AdviceReport | null>(null),
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
    void controller.service
      .records('baseline', 0)
      .then((r) => (records = r))
      .catch((e) => (error = String(e)));
  });
  async function assess() {
    if (!$controller.session) return;
    error = '';
    try {
      report = await controller.work('Workload and upgrade assessment', () =>
        controller.service.assess({
          sessionId: $controller.session!.id,
          baselineId: baselineId || null,
          workload,
          expectation,
          currentDemandPercent: demand,
          annualGrowthPercent: growth,
          quoteLow: low ?? null,
          quoteHigh: high ?? null,
          quoteSource: source,
        }),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel">
  <h2>Workload suitability and upgrade scenarios</h2>
  <p class="muted">
    Start with the result you need. Capacity scenarios use your assumptions;
    they do not predict hardware life or future macOS support.
  </p>
  <form
    class="feature-form"
    onsubmit={(e) => {
      e.preventDefault();
      void assess();
    }}
  >
    <div class="form-grid">
      <label
        >Workload<input
          required
          maxlength="2000"
          bind:value={workload}
        /></label
      ><label
        >Expected task result<input
          required
          maxlength="2000"
          placeholder="Compile in under 2 minutes"
          bind:value={expectation}
        /></label
      >
    </div>
    <label
      >Representative baseline<select bind:value={baselineId}
        ><option value=""
          >No baseline — guidance will request measurements</option
        >{#each records.filter((r) => r.status === 'complete') as r}<option
            value={r.id}>{r.title} · {r.createdAt}</option
          >{/each}</select
      ></label
    >
    <div class="form-grid">
      <label
        >Assumed current demand (% of chosen capacity)<input
          type="number"
          required
          min="1"
          max="100"
          step="1"
          bind:value={demand}
        /></label
      ><label
        >Assumed annual demand growth (%)<input
          type="number"
          required
          min="0"
          max="100"
          step="1"
          bind:value={growth}
        /></label
      >
    </div>
    <details>
      <summary>Optional model-specific cost quote</summary>
      <div class="form-grid">
        <label
          >Quoted low<input
            type="number"
            min="0"
            step="0.01"
            bind:value={low}
          /></label
        ><label
          >Quoted high<input
            type="number"
            min="0"
            step="0.01"
            bind:value={high}
          /></label
        >
      </div>
      <label
        >Quote source, date and currency<input
          maxlength="1000"
          bind:value={source}
          placeholder="Vendor / URL, date, USD"
        /></label
      >
    </details>
    <button class="primary" disabled={busy || !$controller.session}
      >Assess with these assumptions</button
    >
  </form>
</section>
{#if error}<p class="inline-error" role="alert">
    {error}
  </p>{/if}{#if report}<section class="panel">
    <h2>{report.assessment}</h2>
    <p class="muted">{report.qualification}</p>
    {#each report.findings as finding}<h3>{finding.title}</h3>
      <p>{finding.explanation}</p>
      <p class="muted small">{finding.confidence}</p>
      <Evidence items={finding.evidence} />{/each}<Evidence
      items={report.scenarios}
    />
    <h3>Verify model, mode and compatibility</h3>
    <div class="source-links">
      {#each report.sources as href}<a
          {href}
          onclick={(e) => {
            e.preventDefault();
            void controller.run(() => controller.service.reference(href));
          }}
          target="_blank"
          rel="noreferrer">{href}</a
        >{/each}
    </div>
    <p class="muted small">
      Sources are reference links; Coqui does not fetch current prices or
      compatibility automatically. Confirm exact model and software requirements
      before buying.
    </p>
  </section>{/if}
