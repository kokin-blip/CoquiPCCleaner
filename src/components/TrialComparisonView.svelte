<script lang="ts">
  import type { TrialComparison } from '../bindings/TrialComparison';
  import { date } from '../format';
  let { comparison }: { comparison: TrialComparison } = $props();
  const number = (value: number) =>
    new Intl.NumberFormat(undefined, { maximumFractionDigits: 3 }).format(
      value,
    );
</script>

<div class="comparison-content">
  <h2>Recorded task comparison</h2>
  <p>{comparison.protocol.task} · {comparison.request.change}</p>
  <p class="muted">
    Prepared {date(comparison.createdAt)}. {comparison.matchingConditions
      ? 'Declared protocol and conditions match.'
      : 'Declared conditions or targets differ.'}
  </p>
  <div class="table-scroll">
    <table>
      <caption
        >Elapsed intervals in seconds; task completion is user-reported.</caption
      >
      <thead
        ><tr
          ><th>Group</th><th>Trials</th><th>Median</th><th>Range</th><th
            >Met own target</th
          ></tr
        ></thead
      >
      <tbody
        >{#each [{ name: 'Before', data: comparison.before }, { name: 'After', data: comparison.after }] as group}<tr
            ><th scope="row">{group.name}</th><td>{group.data.trials.length}</td
            ><td>{number(group.data.medianSeconds)}s</td><td
              >{number(group.data.minimumSeconds)}–{number(
                group.data.maximumSeconds,
              )}s</td
            ><td>{group.data.targetMetCount}/{group.data.trials.length}</td></tr
          >{/each}</tbody
      >
    </table>
  </div>
  <p>
    Median elapsed change: {number(comparison.medianChangePercent)}%. A negative
    change is a shorter recorded interval; it does not establish what caused it.
  </p>
  {#each comparison.limitations as limitation}<p class="callout">
      {limitation}
    </p>{/each}
  <details>
    <summary>Included trials and comparison limits</summary>
    {#each [{ name: 'Before', data: comparison.before }, { name: 'After', data: comparison.after }] as group}<h3
      >
        {group.name}
      </h3>
      {#each group.data.trials as trial}<p class="small">
          {date(trial.createdAt)} · {number(trial.elapsedSeconds)}s · {trial.id}
        </p>{/each}{/each}
    <p class="muted small">{comparison.qualification}</p>
  </details>
</div>

<style>
  .comparison-content {
    min-width: 0;
    overflow-wrap: anywhere;
  }
</style>
