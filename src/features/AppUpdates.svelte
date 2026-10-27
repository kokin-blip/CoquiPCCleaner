<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { Search, RefreshCw, ArrowLeft, FolderOpen } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { AppUpdatePlan } from '../bindings/AppUpdatePlan';
  import type { AppUpdateReport } from '../bindings/AppUpdateReport';
  import type { ApplicationFootprint } from '../bindings/ApplicationFootprint';
  import AppUpdateReportView from '../components/AppUpdateReportView.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let rows = $state<ApplicationFootprint[]>([]);
  let total = $state(0),
    offset = $state(0),
    query = $state(''),
    revision = $state(0);
  let selected = $state<string[]>([]),
    plan = $state<AppUpdatePlan | null>(null),
    report = $state<AppUpdateReport | null>(null);
  let acknowledged = $state(false),
    loading = $state(false),
    checking = $state(false),
    stopping = $state(false),
    error = $state('');
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  let alive = true;
  let reportRevision = 0;
  let resultElement = $state<HTMLDivElement>();
  let sourceHeading = $state<HTMLHeadingElement>();
  let appSearch = $state<HTMLInputElement>();
  onMount(() => {
    const revision = reportRevision;
    controller.service
      .latestAppUpdates()
      .then((value) => {
        if (alive && revision === reportRevision) report = value;
      })
      .catch((e) => {
        if (alive) error = String(e);
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const text = query.trim(),
      page = offset;
    void revision;
    let current = true;
    loading = true;
    controller.service
      .applicationPage(text, 'name', page, false)
      .then((value) => {
        if (current) {
          rows = value.applications;
          total = value.total;
        }
      })
      .catch((e) => {
        if (current) error = String(e);
      })
      .finally(() => {
        if (current) loading = false;
      });
    return () => {
      current = false;
    };
  });
  function toggle(path: string) {
    selected = selected.includes(path)
      ? selected.filter((value) => value !== path)
      : [...selected, path];
  }
  async function choose() {
    error = '';
    try {
      const path = await controller.service.selectApplication();
      if (path && !selected.includes(path) && selected.length < 10)
        selected = [...selected, path];
    } catch (e) {
      error = String(e);
    }
  }
  async function prepare() {
    error = '';
    acknowledged = false;
    try {
      plan = await controller.work('Reading selected update sources', () =>
        controller.service.prepareAppUpdates([...selected]),
      );
      await tick();
      sourceHeading?.focus();
    } catch (e) {
      error = String(e);
    }
  }
  async function check() {
    if (!plan || !acknowledged || busy) return;
    const id = plan.id;
    reportRevision++;
    checking = true;
    stopping = false;
    error = '';
    report = null;
    try {
      const value = await controller.work('Checking selected app updates', () =>
        controller.service.checkAppUpdates(id, acknowledged, (value) => {
          if (alive) report = value;
        }),
      );
      if (alive) {
        report = value;
        await tick();
        resultElement?.focus();
      }
    } catch (e) {
      if (alive) error = String(e);
    } finally {
      if (alive) {
        checking = false;
        stopping = false;
        plan = null;
        acknowledged = false;
        selected = [];
      }
    }
  }
  async function stop() {
    stopping = true;
    try {
      await controller.service.cancelWork();
    } catch (e) {
      stopping = false;
      error = String(e);
    }
  }
  async function showSelection(restart: boolean) {
    plan = null;
    acknowledged = false;
    if (restart) {
      reportRevision++;
      report = null;
      selected = [];
    }
    await tick();
    appSearch?.focus();
  }
</script>

<section aria-label="Check application updates">
  <div class="section-heading">
    <h2>Check app updates</h2>
    {#if checking}<button disabled={stopping} onclick={stop}
        >{stopping ? 'Stopping…' : 'Stop update check'}</button
      >{/if}
  </div>
  <p class="muted">
    Choose up to 10 apps, review their sources, then opt in to an online check.
    Install through the App Store or the app’s own updater.
  </p>
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  {#if checking}<p role="status">
      Reading the selected update sources… Results appear as each check
      finishes.
    </p>
  {:else if plan}
    <div class="update-source-review">
      <h3 bind:this={sourceHeading} tabindex="-1">Review online sources</h3>
      <p class="small muted">
        Expires {date(plan.expiresAt)}. {plan.qualification}
      </p>
      {#each plan.candidates as item (item.path)}<div class="update-source">
          <strong>{item.name}</strong><span class="muted small">
            · Installed {item.installedVersion || 'version unavailable'}</span
          >
          <p class="path small selectable">{item.path}</p>
          <p class="small selectable path">
            {item.sourceUrl ?? 'No supported public source'}
          </p>
          <p class="small muted">{item.qualification}</p>
          <p class="small muted">
            Source rules reviewed {item.sourceReviewDate}.
          </p>
        </div>{/each}
      <label class="check-row"
        ><input type="checkbox" bind:checked={acknowledged} />Contact these
        sources for this check. Apple receives selected App Store bundle IDs;
        vendors receive requests to the displayed feed URLs. These services also
        see my network address.</label
      >
      <div class="button-row">
        <button disabled={busy} onclick={() => showSelection(false)}
          ><ArrowLeft size={14} />Change selection</button
        ><button
          class="primary"
          disabled={busy ||
            !acknowledged ||
            !plan.candidates.some((item) => item.source !== 'unavailable')}
          onclick={check}>Check selected updates</button
        >
      </div>
    </div>
  {:else if report}
    <button disabled={busy} onclick={() => showSelection(true)}
      >Check other apps</button
    >
  {:else}
    <div class="file-toolbar">
      <label
        ><Search size={14} /><input
          type="search"
          aria-label="Search apps for updates"
          placeholder="Search recorded apps…"
          bind:this={appSearch}
          bind:value={query}
          oninput={() => (offset = 0)}
        /></label
      >
      <span class="muted small">{selected.length} of 10 selected</span>
      <button disabled={busy || loading} onclick={() => revision++}
        ><RefreshCw size={14} />Reload recorded apps</button
      >
      <button disabled={busy || selected.length >= 10} onclick={choose}
        ><FolderOpen size={14} />Choose application…</button
      >
    </div>
    <div class="button-row">
      <button
        class="primary"
        disabled={busy || !selected.length}
        onclick={prepare}>Review update sources</button
      >
    </div>
    {#if selected.length}<details>
        <summary>{selected.length} selected apps</summary
        >{#each selected as path}<div class="inline-between">
            <span class="path small selectable">{path}</span><button
              disabled={busy}
              onclick={() => toggle(path)}
              aria-label={`Deselect ${path}`}>Deselect</button
            >
          </div>{/each}
      </details>{/if}
    {#if loading}<p role="status">Reading recorded applications…</p>{/if}
    <div class="table-scroll">
      <table>
        <thead
          ><tr
            ><th scope="col">Select</th><th scope="col">Application</th><th
              scope="col">Recorded version</th
            ></tr
          ></thead
        ><tbody>
          {#each rows as item (item.path)}<tr
              ><td
                ><input
                  type="checkbox"
                  aria-label={`Check updates for ${item.name}`}
                  checked={selected.includes(item.path)}
                  disabled={busy ||
                    (!selected.includes(item.path) && selected.length >= 10)}
                  onchange={() => toggle(item.path)}
                /></td
              ><td
                ><strong>{item.name}</strong><span
                  class="path small muted selectable">{item.path}</span
                ></td
              ><td>{item.version || 'Unavailable'}</td></tr
            >{:else}<tr
              ><td colspan="3"
                >No recorded apps match. Use Installed Apps to refresh the local
                inventory.</td
              ></tr
            >{/each}
        </tbody>
      </table>
    </div>
    <div class="pagination">
      <button
        disabled={loading || offset === 0}
        onclick={() => (offset = Math.max(0, offset - 50))}
        >Previous apps</button
      ><span class="muted small"
        >{total
          ? `${offset + 1}–${Math.min(offset + rows.length, total)} of ${total}`
          : 'No recorded apps'}</span
      ><button
        disabled={loading || offset + rows.length >= total}
        onclick={() => (offset += 50)}>Next apps</button
      >
    </div>
  {/if}
  {#if report}<div
      class="update-report"
      bind:this={resultElement}
      tabindex="-1"
      role="region"
      aria-label="Completed and pending update results"
    >
      <AppUpdateReportView {controller} {report} />
    </div>{/if}
</section>

<style>
  td .path {
    display: block;
    margin-top: 4px;
  }
  .check-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin: 18px 0;
    max-width: 75ch;
  }
  .check-row input {
    flex: 0 0 auto;
    margin-top: 3px;
  }
  .update-source-review {
    margin: 20px 0;
  }
  .update-source-review h3 {
    font-size: 15px;
  }
  .update-source-review > p {
    margin-top: 8px;
  }
  .update-source {
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
  }
  .update-source p {
    margin-top: 6px;
  }
  .update-report {
    border-top: 1px solid var(--border);
    margin-top: 28px;
    padding-top: 24px;
  }
</style>
