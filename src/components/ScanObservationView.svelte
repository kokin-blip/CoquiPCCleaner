<script lang="ts">
  import type { ScanObservations } from '../bindings/ScanObservations';
  import type { createAppStore } from '../state';
  import { date } from '../format';
  import Status from './Status.svelte';
  let {
    report,
    controller,
  }: {
    report: ScanObservations;
    controller: ReturnType<typeof createAppStore>;
  } = $props();
  const names: Record<string, string> = {
    system: 'System details',
    startup: 'Startup configurations',
    applications: 'Application inventory',
  };
  const states: Record<string, string> = {
    notSelected: 'Not selected',
    pending: 'Not started',
    running: 'Checking',
    complete: 'Collected',
    partial: 'Partial',
    unavailable: 'Unavailable',
    cancelled: 'Cancelled',
    interrupted: 'Interrupted',
  };
</script>

<section aria-label="Scan metadata checks">
  <div class="section-heading">
    <h2>Metadata checks</h2>
    <Status value={report.status} />
  </div>
  <p class="small muted">{report.qualification}</p>
  {#each report.checks as check}
    <div class="check-result">
      <div class="section-heading">
        <h3>{names[check.module] ?? check.module}</h3>
        <span class="subtle-tag">{states[check.status] ?? 'Unavailable'}</span>
      </div>
      <p class="small">
        {check.module === 'system' &&
        ['complete', 'partial'].includes(check.status)
          ? 'System readings recorded'
          : check.count == null
            ? 'No count established'
            : `${check.count.toLocaleString()} recorded entries`}{check.finishedAt
          ? ` · ${date(check.finishedAt)}`
          : ' · No completed check date'}
      </p>
      <p class="small muted">
        {check.qualification}{check.omissions
          ? ` ${check.omissions} recorded omissions.`
          : ''}
      </p>
      {#if $controller.session?.id === report.sessionId}
        {#if check.module === 'system'}<button
            onclick={() => controller.navigate('Hardware')}
            >Review system details</button
          >
        {:else if check.module === 'startup'}<button
            onclick={() =>
              controller.openRecommendation('Performance', 'startup')}
            >Review startup configurations</button
          >
        {:else}<button onclick={() => controller.navigate('Applications')}
            >Review applications</button
          >{/if}
      {/if}
    </div>
  {/each}
</section>

<style>
  .check-result {
    padding: 16px 0;
    border-bottom: 1px solid var(--border);
  }
  .check-result h3 {
    font-size: 14px;
  }
  .check-result p {
    margin: 8px 0;
    max-width: 75ch;
  }
</style>
