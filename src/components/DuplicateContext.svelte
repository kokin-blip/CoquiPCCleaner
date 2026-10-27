<script lang="ts">
  import type { DuplicateReport } from '../bindings/DuplicateReport';
  let { report }: { report: DuplicateReport } = $props();
</script>

{#if report.request}
  <p class="small muted">
    Location: {report.request.location === 'downloads' ? 'Downloads only' : 'All indexed locations'}.
    Similar-photo check: {report.request.similarPhotos ? 'included' : 'not requested'}.
    {#if report.scopeRoot}<span class="path">{report.scopeRoot}</span>{/if}
  </p>
{:else}
  <p class="small muted">Historical analysis choices were not recorded. No Downloads scope or photo-check choice is inferred.</p>
{/if}
