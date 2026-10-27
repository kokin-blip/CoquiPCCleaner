<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { RecommendationPlan } from '../bindings/RecommendationPlan';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import RecommendationPlanView from '../components/RecommendationPlanView.svelte';
  let {
    controller,
    mode = 'all',
    compact = false,
  }: {
    controller: ReturnType<typeof createAppStore>;
    mode?: 'all' | 'actions' | 'workloads' | 'bottlenecks';
    compact?: boolean;
  } = $props();
  let plan = $state<RecommendationPlan | null>(null),
    records = $state<ActivityRecord[]>([]),
    trials = $state<ActivityRecord[]>([]),
    baselineId = $state(''),
    trialId = $state(''),
    loading = $state(false),
    error = $state(''),
    saved = $state(false);
  let revision = 0;
  let session = $derived($controller.session);
  let sourceId = $derived(session?.id);
  let sourceStatus = $derived(session?.status);
  let workloadKey = $derived(JSON.stringify($controller.settings?.workload));
  let busy = $derived(
    $controller.scanning ||
      $controller.workRunning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    void controller.service
      .records('baseline', 0)
      .then((r) => (records = r))
      .catch((e) => (error = String(e)));
    void controller.service
      .records('workloadTrial', 0)
      .then((r) => (trials = r.filter((row) => row.kind === 'workloadTrial')))
      .catch((e) => (error = String(e)));
    return () => {
      revision++;
    };
  });
  $effect(() => {
    const id = sourceId,
      status = sourceStatus;
    const baseline = baselineId;
    const trial = trialId;
    void workloadKey;
    if (id && status !== 'running') void load(id, baseline, trial);
    else {
      revision++;
      plan = null;
      loading = false;
    }
  });
  async function load(id: string, baseline: string, trial: string) {
    const current = ++revision;
    loading = true;
    error = '';
    saved = false;
    plan = null;
    try {
      const result = await controller.service.recommendationPlan(
        id,
        baseline || null,
        trial || null,
      );
      if (current === revision) plan = result;
    } catch (e) {
      if (current === revision) error = String(e);
    } finally {
      if (current === revision) loading = false;
    }
  }
  async function save() {
    if (!session) return;
    const id = session.id,
      baseline = baselineId,
      trial = trialId,
      current = revision;
    try {
      const result = await controller.work('Save recommendation plan', () =>
        controller.service.saveRecommendationPlan(
          id,
          baseline || null,
          trial || null,
        ),
      );
      if (current === revision) {
        plan = result;
        saved = true;
      }
    } catch (e) {
      if (current === revision) error = String(e);
    }
  }
</script>

<section class="panel" aria-label="Workload recommendation plan">
  <div class="section-heading">
    <h2>{mode === 'bottlenecks' ? 'Bottlenecks' : 'Workload review plan'}</h2>
    {#if plan && !compact}<button
        disabled={busy || loading || saved}
        onclick={save}
        >{saved ? 'Saved to History' : 'Save plan to History'}</button
      >{/if}
  </div>
  <p class="muted">
    Start with the task you need to complete. Each recommendation opens its own
    review; saving a plan applies no changes.
  </p>
  {#if !compact}<label
      >Representative baseline<select
        bind:value={baselineId}
        disabled={busy || loading}
      >
        <option value="">No baseline selected</option>
        {#each records.filter((r) => r.status === 'complete') as row}<option
            value={row.id}>{row.title} · {row.createdAt}</option
          >{/each}
      </select></label
    ><label
      >Measured task result<select
        bind:value={trialId}
        disabled={busy || loading}
        ><option value="">No task trial selected</option
        >{#each trials.filter((r) => r.status === 'complete') as row}<option
            value={row.id}>{row.title} · {row.createdAt}</option
          >{/each}</select
      ></label
    ><button
      class="text-button"
      disabled={busy}
      onclick={() => controller.openRecommendation('Performance', 'task')}
      >Time or compare a task</button
    >{/if}
  {#if loading}<p role="status">
      Preparing recommendations from saved evidence…
    </p>{/if}
  {#if error}<p class="inline-error" role="alert">
      {error}
      <button
        disabled={busy}
        onclick={() => session && load(session.id, baselineId, trialId)}
        >Retry recommendations</button
      >
    </p>{/if}
  {#if plan}<RecommendationPlanView
      {controller}
      {plan}
      {mode}
      limit={compact ? 5 : 100}
    />
  {:else if !loading && !error}<p class="muted">
      {session?.status === 'running'
        ? 'Wait for the current analysis to finish.'
        : 'Run or select an analysis to prepare a workload review plan.'}
    </p>{/if}
</section>
