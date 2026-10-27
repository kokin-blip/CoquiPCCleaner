<script lang="ts">
  import type { AppUpdateReport } from '../bindings/AppUpdateReport';
  import type { createAppStore } from '../state';
  import { date } from '../format';
  import Status from './Status.svelte';
  import { ExternalLink } from '@lucide/svelte';
  let {
    report,
    controller,
  }: {
    report: AppUpdateReport;
    controller: ReturnType<typeof createAppStore>;
  } = $props();
  let error = $state('');
  const states: Record<string, string> = {
    available: 'Newer version listed',
    noNewerVersionObserved: 'No newer version observed',
    unavailable: 'Check unavailable',
    uncomparable: 'Version comparison unavailable',
    cancelled: 'Not checked · cancelled',
  };
  async function open(path: string) {
    error = '';
    try {
      await controller.work('Opening the vendor update tool', () =>
        controller.service.openAppUpdate(report.id, path),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section aria-label="App update results">
  <div class="section-heading">
    <h2>Update check</h2>
    <Status value={report.status} />
  </div>
  <p class="muted small">
    Started {date(report.createdAt)}{report.finishedAt
      ? ` · Finished ${date(report.finishedAt)}`
      : ''}
  </p>
  <p class="muted">{report.qualification}</p>
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  {#each report.items as item (item.path)}
    <article class="update-result">
      <div class="section-heading">
        <h3>{item.name}</h3>
        <span class="subtle-tag">{states[item.state] ?? 'Not checked'}</span>
      </div>
      <p class="path small muted selectable">{item.path}</p>
      <p>
        Installed {item.installedVersion ||
          'version unavailable'}{item.installedBuild
          ? ` · build ${item.installedBuild}`
          : ''}{item.offeredVersion ? ` → Listed ${item.offeredVersion}` : ''}
      </p>
      {#if item.offeredBuild}<p class="small muted">
          Listed build {item.offeredBuild}
        </p>{/if}
      <p class="small muted">
        {item.checkedAt
          ? `Checked ${date(item.checkedAt)}`
          : 'No completed check'}{item.publishedAt
          ? ` · Published ${date(item.publishedAt)}`
          : ''}
      </p>
      <p class="small selectable path">
        {item.sourceUrl ?? 'No supported public update source'}
      </p>
      <p class="small muted">Source rules reviewed {item.sourceReviewDate}.</p>
      <p class="small">{item.qualification}</p>
      {#if item.minSystemVersion}<p class="small muted">
          Listed minimum macOS {item.minSystemVersion}. Compatibility is not
          verified.
        </p>{/if}
      {#if item.state === 'available'}<button
          disabled={$controller.workRunning || $controller.scanning}
          onclick={() => open(item.path)}
          ><ExternalLink size={14} />{item.source === 'appStore'
            ? 'Open App Store updates'
            : 'Open app to update'}</button
        >{/if}
    </article>
  {/each}
</section>

<style>
  .update-result {
    padding: 20px 0;
    border-top: 1px solid var(--border);
  }
  .update-result h3 {
    font-size: 14px;
  }
  .update-result p {
    margin: 8px 0;
    max-width: 75ch;
  }
</style>
