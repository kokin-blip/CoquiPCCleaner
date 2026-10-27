<script lang="ts">
  import type { WorkloadTrial } from '../bindings/WorkloadTrial';
  import Status from './Status.svelte';
  import Evidence from './Evidence.svelte';
  import { date, bytes } from '../format';
  let { trial }: { trial: WorkloadTrial } = $props();
  const seconds = (value: number) =>
    new Intl.NumberFormat(undefined, { maximumFractionDigits: 3 }).format(
      value,
    );
  let complete = $derived(
    trial.status === 'complete' &&
      trial.completionReported &&
      trial.finishedAt !== null &&
      trial.elapsedSeconds !== null &&
      Number.isFinite(trial.elapsedSeconds) &&
      trial.elapsedSeconds > 0 &&
      trial.elapsedSeconds < 600,
  );
</script>

<div class="trial-content">
  <div class="section-heading">
    <h2>{trial.request.task}</h2>
    <Status value={trial.status} />
  </div>
  <p class="muted">
    {trial.request.workload} · Recorded {date(trial.createdAt)}
  </p>
  <dl class="trial-facts">
    <div>
      <dt>{complete ? 'Measured task interval' : 'Last measured elapsed'}</dt>
      <dd>
        {trial.elapsedSeconds === null
          ? 'Not recorded'
          : `${seconds(trial.elapsedSeconds)} seconds`}
      </dd>
    </div>
    <div>
      <dt>Declared target</dt>
      <dd>{seconds(trial.request.targetSeconds)} seconds or less</dd>
    </div>
  </dl>
  {#if complete}<p>
      {trial.elapsedSeconds! <= trial.request.targetSeconds
        ? 'This trial met the declared target.'
        : 'This trial exceeded the declared target.'} Completion was reported by you.
      Repeat under comparable conditions before judging a configuration or upgrade.
    </p>
  {:else}<p class="muted">
      This state does not establish task completion or whether the target was
      met.
    </p>{/if}
  <details>
    <summary>Task protocol and measurements</summary>
    <p><strong>Inputs / project:</strong> {trial.request.inputs}</p>
    <p><strong>Application / version:</strong> {trial.request.application}</p>
    <p><strong>Conditions:</strong> {trial.request.conditions}</p>
    <p>
      {trial.samples.length} recorded two-second activity samples. Utilization does
      not identify the limiting component.
    </p>
    {#if trial.samples.length}<div class="table-scroll">
        <table>
          <thead
            ><tr
              ><th>Observed</th><th>CPU</th><th>Used memory</th><th>Swap</th
              ></tr
            ></thead
          ><tbody>
            {#each trial.samples as sample}<tr
                ><td>{date(sample.observedAt)}</td><td
                  >{sample.cpuPercent.toFixed(1)}%</td
                ><td>{bytes(sample.usedMemoryBytes)}</td><td
                  >{bytes(sample.swapBytes)}</td
                ></tr
              >{/each}
          </tbody>
        </table>
      </div>{/if}
    <Evidence items={trial.power} />
    <p class="muted small">{trial.qualification}</p>
  </details>
</div>

<style>
  .trial-content {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .trial-facts {
    display: flex;
    flex-wrap: wrap;
    gap: 16px 40px;
    margin-block: 16px;
  }
  dt {
    color: var(--secondary);
  }
  dd {
    margin: 4px 0 0;
    font-variant-numeric: tabular-nums;
  }
</style>
