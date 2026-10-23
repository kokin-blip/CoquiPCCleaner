<script lang="ts">
  import type { FirmwareComparison } from '../bindings/FirmwareComparison';
  import { date, label } from '../format';
  import Status from './Status.svelte';
  let { comparison }: { comparison: FirmwareComparison } = $props();
</script>

<section
  class="panel firmware-comparison"
  aria-label="Saved firmware comparison"
>
  <h2>Recorded changes between audits</h2>
  <p class="muted">{comparison.qualification}</p>
  <div class="firmware-comparison-context">
    <div>
      <strong>Before · {date(comparison.beforeObservedAt)}</strong>
      <p>
        {comparison.beforePlatform.modelName || comparison.beforePlatform.model} ·
        {comparison.beforePlatform.chip}
      </p>
    </div>
    <div>
      <strong>After · {date(comparison.afterObservedAt)}</strong>
      <p>
        {comparison.afterPlatform.modelName || comparison.afterPlatform.model} · {comparison
          .afterPlatform.chip}
      </p>
    </div>
  </div>
  {#if !comparison.comparablePlatform}<p class="inline-error">
      Reported platform eligibility differs or is unconfirmed. These audits do
      not support a configuration-change conclusion.
    </p>{/if}
  <div class="table-scroll">
    <table>
      <thead
        ><tr><th>Property / comparison</th><th>Before</th><th>After</th></tr
        ></thead
      ><tbody
        >{#each comparison.rows as row}<tr
            ><td
              ><strong>{row.property}</strong>
              <p>{label(row.state)}</p>
              <p class="muted small">{row.explanation}</p></td
            >{#each [row.before, row.after] as reading}<td
                >{#if reading}<strong>{reading.value} {reading.unit}</strong
                  >{#if reading.state !== 'supported'}<Status
                      value={reading.state}
                    />{/if}
                  <p class="muted small">
                    {reading.source} · {date(reading.observedAt)}
                  </p>
                  <p class="muted small">{reading.qualification}</p>{:else}<span
                    class="muted">Not collected in this scope</span
                  >{/if}</td
              >{/each}</tr
          >{/each}</tbody
      >
    </table>
  </div>
  <p class="muted small">
    Comparison saved {date(comparison.createdAt)}. Source audits:
    <span class="path"
      >{comparison.beforeReportId} / {comparison.afterReportId}</span
    >
  </p>
</section>
