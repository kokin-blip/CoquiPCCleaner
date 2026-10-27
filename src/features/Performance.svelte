<script lang="ts">
  import type { ScanObservations } from '../bindings/ScanObservations';
  import Baselines from './Baselines.svelte';
  import TaskTrials from './TaskTrials.svelte';
  import ManualPlans from './ManualPlans.svelte';
  import { tabKeys } from '../keyboard';
  import { Activity, ArrowRight, Info } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Chart from '../components/Chart.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let observations = $state<ScanObservations | null>(null);
  let observationError = $state('');
  $effect(() => {
    const id = $controller.session?.id;
    let current = true;
    observations = null;
    observationError = '';
    if (id)
      controller.service
        .scanObservations(id)
        .then((value) => {
          if (current) observations = value;
        })
        .catch(() => {
          if (current) observationError = 'Saved startup check unavailable.';
        });
    return () => {
      current = false;
    };
  });
  let startupItems = $derived(
    observations
      ? observations.startupItems
      : observationError
        ? []
        : ($controller.session?.snapshot?.startupItems ?? []),
  );
  let tab = $state('Overview');
  $effect(() => {
    const target = $controller.reviewTarget;
    if (
      target &&
      ['power', 'startup', 'baseline', 'cpu', 'memory', 'task'].includes(target)
    ) {
      tab =
        target === 'task'
          ? 'Task trials'
          : target === 'power'
            ? 'Manual plans'
            : target === 'startup'
              ? 'Startup'
              : 'Baselines';
      controller.clearReviewTarget();
    }
  });
  let seconds = $state(120);
  let processFilter = $state('');
  let live = $derived(
    $controller.monitoringStatus === 'live'
      ? $controller.samples.at(-1)
      : undefined,
  );
  let samples = $derived($controller.samples.slice(-Math.floor(seconds / 2)));
</script>

<div class="page-heading">
  <div>
    <h1>Performance</h1>
    <p class="muted">
      Understand activity in context. Utilization is not a health score.
    </p>
  </div>
  <span class="muted small"
    >{live
      ? `Updated ${date(live.observedAt)}`
      : 'No live measurements recorded'}</span
  >
</div>
{#if $controller.monitoringStatus === 'failed'}
  <div class="callout" role="status">
    <Activity size={21} />
    <div>
      <strong>Monitoring stopped. Current measurements are unavailable.</strong>
      <p>{$controller.monitoringError}</p>
      <button onclick={() => controller.retryMonitoring()}
        >Retry monitoring</button
      >
    </div>
  </div>
{:else if $controller.monitoringStatus === 'starting'}
  <p class="muted" role="status">
    Waiting for the first monitoring measurement…
  </p>
{/if}
<div
  class="tabs"
  role="tablist"
  tabindex="-1"
  onkeydown={tabKeys}
  aria-label="Performance views"
>
  {#each ['Overview', 'Processes', 'Startup', 'Preferences', 'Baselines', 'Task trials', 'Manual plans'] as name}<button
      role="tab"
      tabindex={tab === name ? 0 : -1}
      aria-selected={tab === name}
      class:active={tab === name}
      onclick={() => (tab = name)}>{name}</button
    >{/each}
</div>
{#if tab === 'Task trials'}<TaskTrials
    {controller}
  />{:else if tab === 'Baselines'}<Baselines
    {controller}
  />{:else if tab === 'Manual plans'}<ManualPlans
    {controller}
  />{:else if tab === 'Overview'}
  {#if !$controller.settings?.monitoringEnabled}<div class="callout">
      <Activity size={21} />
      <div>
        <strong>Live monitoring is off.</strong>
        <p>
          Enable two-second sampling while Coqui is open. It adds no background
          agent and changes no system settings.
        </p>
        <button
          class="text-button"
          onclick={() =>
            controller.run(() =>
              controller.save({
                ...$controller.settings!,
                monitoringEnabled: true,
              }),
            )}>Enable monitoring<ArrowRight size={14} /></button
        >
      </div>
    </div>{/if}
  <div class="section-heading">
    <h2>Current session</h2>
    <label class="inline-label"
      >Chart interval<select bind:value={seconds}
        ><option value={60}>1 minute</option><option value={120}
          >2 minutes</option
        ><option value={300}>5 minutes</option><option value={900}
          >15 minutes</option
        ></select
      ></label
    >
  </div>
  <div class="performance-grid">
    <section class="panel">
      <div class="section-heading">
        <h2>CPU utilization</h2>
        <strong
          >{live ? `${live.cpuPercent.toFixed(1)}%` : 'Not recorded'}</strong
        >
      </div>
      <Chart {samples} />
      <p class="muted small">
        An average across logical cores. Sustained high use can be normal during
        demanding work.
      </p>
    </section>
    <section class="panel">
      <div class="section-heading">
        <h2>Memory in use</h2>
        <strong>{live ? bytes(live.usedMemoryBytes) : 'Not recorded'}</strong>
      </div>
      <Chart {samples} kind="memory" />
      <p class="muted small">
        {live
          ? `${bytes(live.totalMemoryBytes)} installed · ${bytes(live.swapBytes)} swap in use`
          : 'No history is invented for unrecorded intervals.'}
      </p>
    </section>
  </div>
  <section class="panel">
    <h2>Readable process I/O</h2>
    <div class="scan-facts">
      <div>
        <span>Read during last interval</span><strong
          >{live ? bytes(live.diskReadBytes) : 'Not recorded'}</strong
        >
      </div>
      <div>
        <span>Written during last interval</span><strong
          >{live ? bytes(live.diskWrittenBytes) : 'Not recorded'}</strong
        >
      </div>
      <div><span>GPU utilization</span><strong>Unavailable</strong></div>
      <div><span>Temperatures</span><strong>Unavailable</strong></div>
    </div>
    <p class="muted small">
      {live?.qualification ??
        'Supported metrics will appear after monitoring records samples. No sensor estimates are supplied.'}
    </p>
  </section>
{:else if tab === 'Processes'}
  <section class="panel">
    <div class="section-heading">
      <h2>Top readable processes</h2>
      <input
        type="search"
        placeholder="Filter processes…"
        aria-label="Filter processes"
        bind:value={processFilter}
      />
    </div>
    <p class="muted small">
      Up to 40 processes ordered by current CPU usage. Permissions can limit
      visibility. Coqui does not terminate processes.
    </p>
    <div class="table-scroll">
      <table>
        <thead
          ><tr
            ><th>Process</th><th>PID</th><th class="numeric"
              >CPU · normalized</th
            ><th class="numeric">Memory</th></tr
          ></thead
        ><tbody
          >{#each live?.processes.filter((p) => p.name
              .toLowerCase()
              .includes(processFilter.toLowerCase())) ?? [] as process}<tr
              ><td>{process.name}</td><td class="muted">{process.pid}</td><td
                class="numeric">{process.cpuPercent.toFixed(1)}%</td
              ><td class="numeric">{bytes(process.memoryBytes)}</td></tr
            >{/each}</tbody
        >
      </table>
    </div>
    {#if !live}<p class="empty-inline muted">
        {$controller.monitoringStatus === 'failed'
          ? 'Retry monitoring to record new process samples.'
          : 'No process samples recorded.'}
      </p>{/if}
  </section>
{:else if tab === 'Startup'}
  <div class="callout">
    <Info size={20} />
    <div>
      <strong>Configured on disk does not mean enabled at runtime.</strong>
      <p>
        This list covers readable launch configurations. Modern login items and
        management policies may require manual inspection. No boot-time
        improvement is estimated.
      </p>
      <button
        class="text-button"
        onclick={() =>
          controller.run(() => controller.service.settings('login'))}
        >Open Login Items in System Settings<ArrowRight size={14} /></button
      >
    </div>
  </div>
  <section class="panel">
    <h2>Detected startup configurations</h2>
    {#if observations}<p class="small muted">
        {observations.checks.find((check) => check.module === 'startup')
          ?.qualification}
      </p>{/if}
    {#if observationError}<p role="alert">{observationError}</p>{/if}
    {#each startupItems as item}<div class="startup-row">
        <div>
          <strong>{item.name}</strong>
          <p class="path small muted">{item.path}</p>
          <p class="small">{item.state}</p>
        </div>
        <span class="subtle-tag">{item.mechanism}</span>
      </div>{:else}<p class="empty-inline muted">
        No readable startup configurations have been recorded. This does not
        establish that no login items exist.
      </p>{/each}
  </section>
{:else}<section class="panel">
    <h2>{$controller.settings?.workload.preference} preference</h2>
    <p>
      Preferences describe the tradeoffs you want future recommendations to
      consider. This build does not apply power profiles or change startup
      settings.
    </p>
    <div class="inline-wrap">
      {#each ['Balanced', 'Performance', 'Efficiency'] as name}<button
          class:primary={$controller.settings?.workload.preference === name}
          onclick={() =>
            controller.run(() =>
              controller.save({
                ...$controller.settings!,
                workload: {
                  ...$controller.settings!.workload,
                  preference: name,
                },
              }),
            )}>{name}</button
        >{/each}
    </div>
    <p class="muted">
      Balanced: stability and responsiveness. Performance: supported sustained
      performance. Efficiency: battery life and energy savings.
    </p>
    <button
      class="text-button"
      onclick={() => controller.run(() => controller.service.settings('power'))}
      >Review native power settings<ArrowRight size={14} /></button
    >
  </section>{/if}
