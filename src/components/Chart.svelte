<script lang="ts">
  import type { LiveMetrics } from '../bindings/LiveMetrics';
  let {
    samples,
    kind = 'cpu',
  }: { samples: LiveMetrics[]; kind?: 'cpu' | 'memory' } = $props();
  const percentage = (s: LiveMetrics) =>
    kind === 'cpu'
      ? s.cpuPercent
      : s.totalMemoryBytes
        ? (s.usedMemoryBytes / s.totalMemoryBytes) * 100
        : 0;
  let points = $derived(
    samples
      .map(
        (s, i) =>
          `${samples.length < 2 ? 0 : (i / (samples.length - 1)) * 600},${110 - Math.max(0, Math.min(100, percentage(s)))}`,
      )
      .join(' '),
  );
</script>

<div
  class="chart"
  role="img"
  aria-label={`${kind === 'cpu' ? 'CPU utilization' : 'Memory usage'} over ${samples.length} recorded samples. ${samples.length ? `Latest ${percentage(samples[samples.length - 1]).toFixed(1)} percent.` : 'No measurements recorded.'}`}
>
  <div class="chart-labels">
    <span>100%</span><span>50%</span><span>0%</span>
  </div>
  <svg viewBox="0 0 600 120" preserveAspectRatio="none" aria-hidden="true"
    ><path
      d="M0 10H600 M0 60H600 M0 110H600"
      class="chart-grid"
    />{#if samples.length > 1}<polyline
        {points}
        fill="none"
        class="chart-line"
        vector-effect="non-scaling-stroke"
      />{/if}</svg
  >
  {#if samples.length < 2}<span class="chart-empty"
      >Waiting for two recorded samples</span
    >{/if}
</div>
