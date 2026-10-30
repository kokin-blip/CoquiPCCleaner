<script lang="ts">
  import { tick } from 'svelte';
  import type { createAppStore } from '../state';
  import type { CleanupCategoryReport } from '../bindings/CleanupCategoryReport';
  import type { CleanupCategoryKind } from '../bindings/CleanupCategoryKind';
  import type { CleanupCategoryMode } from '../bindings/CleanupCategoryMode';
  import type { CleanupCategoryPage } from '../bindings/CleanupCategoryPage';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  import CacheBrowser from './CacheBrowser.svelte';
  let {
    controller,
    onScan,
    pending = false,
  }: {
    controller: ReturnType<typeof createAppStore>;
    onScan: () => void;
    pending?: boolean;
  } = $props();
  let report = $state<CleanupCategoryReport | null>(null);
  let kind = $state<CleanupCategoryKind | null>(null);
  let page = $state<CleanupCategoryPage | null>(null);
  let offset = $state(0),
    revision = $state(0);
  let loading = $state(false),
    pageLoading = $state(false),
    error = $state('');
  let heading = $state<HTMLHeadingElement>();
  let categoryButtons = $state<
    Partial<Record<CleanupCategoryKind, HTMLButtonElement>>
  >({});
  let previousSession = '';
  const sessionId = $derived($controller.session?.id ?? '');
  const sourceStatus = $derived($controller.session?.status ?? '');
  const exclusions = $derived(
    JSON.stringify($controller.settings?.excludedPaths ?? []),
  );
  const category = $derived(
    report?.categories.find((row) => row.kind === kind),
  );
  const busy = $derived(
    pending ||
      $controller.scanning ||
      $controller.workRunning ||
      $controller.cleanupPreparing ||
      $controller.cleanupRunning ||
      $controller.developerRunning,
  );
  const modeLabel = (mode: CleanupCategoryMode) =>
    ({
      cacheReview: 'Check individual files',
      developerReview: 'Developer review',
      manualReview: 'Manual review',
      protected: 'Protected · inspect only',
    })[mode];
  $effect(() => {
    const id = sessionId,
      status = sourceStatus;
    void exclusions;
    void revision;
    let alive = true;
    report = null;
    error = '';
    if (id !== previousSession) {
      previousSession = id;
      kind = null;
      offset = 0;
    }
    if (!id || status === 'running') {
      loading = false;
      return;
    }
    loading = true;
    controller.service
      .cleanupCategories(id)
      .then((value) => {
        if (alive) report = value;
      })
      .catch((reason) => {
        if (alive) error = String(reason);
      })
      .finally(() => {
        if (alive) loading = false;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const id = sessionId,
      selectedKind = kind,
      start = offset,
      status = sourceStatus;
    void exclusions;
    void revision;
    let alive = true;
    page = null;
    pageLoading = false;
    if (
      !id ||
      !selectedKind ||
      status === 'running' ||
      ['userCaches', 'xcodeCaches'].includes(selectedKind)
    )
      return;
    pageLoading = true;
    controller.service
      .cleanupCategoryEntries({
        sessionId: id,
        kind: selectedKind,
        offset: start,
      })
      .then((value) => {
        if (alive) page = value;
      })
      .catch((reason) => {
        if (alive) error = String(reason);
      })
      .finally(() => {
        if (alive) pageLoading = false;
      });
    return () => {
      alive = false;
    };
  });
  async function open(value: CleanupCategoryKind) {
    kind = value;
    offset = 0;
    error = '';
    await tick();
    heading?.focus();
  }
  async function back() {
    const previous = kind;
    kind = null;
    offset = 0;
    error = '';
    await tick();
    if (previous) categoryButtons[previous]?.focus();
  }
</script>

{#if !sessionId || sourceStatus === 'running'}
  <div class="empty-state compact">
    <h2>
      {sourceStatus === 'running'
        ? 'Wait for the storage scan.'
        : 'Start with a storage scan.'}
    </h2>
    <p>
      Category measurements come from the saved index. Missing locations remain
      unknown.
    </p>
    <button disabled={busy} onclick={onScan}>Review scan scope</button>
  </div>
{:else}
  {#if error}<div class="inline-error" role="alert">
      <span>{error}</span><button disabled={busy} onclick={() => revision++}
        >Retry category view</button
      >
    </div>{/if}
  {#if !kind}
    <div class="section-heading">
      <div>
        <h2>Cleanup categories</h2>
        <p class="muted">
          Inspect recorded footprints and choose the appropriate review.
        </p>
      </div>
      <button disabled={loading || busy} onclick={() => revision++}
        >Refresh saved view</button
      >
    </div>
    {#if loading}<p role="status">Reading saved category measurements…</p>{/if}
    {#if report}
      <p class="small muted">
        Scan recorded {date(report.observedAt)} · <Status
          value={report.sourceStatus}
        /> · path rules reviewed {report.rulesReviewedAt}.
      </p>
      <p class="small muted">
        File allocation is not removable space. Manual and protected categories
        have no removal selection.
      </p>
      <div class="table-wrap">
        <table>
          <thead
            ><tr
              ><th>Category</th><th>Indexed files</th><th class="numeric"
                >Allocated</th
              ><th>Review</th></tr
            ></thead
          ><tbody>
            {#each report.categories as row}<tr
                ><td
                  ><button
                    class="text-button"
                    bind:this={categoryButtons[row.kind]}
                    disabled={busy}
                    onclick={() => open(row.kind)}>{row.title}</button
                  >
                  <p class="small muted">
                    {row.state === 'notIndexed'
                      ? 'Not indexed'
                      : row.state === 'excluded'
                        ? 'Excluded by current preferences'
                        : row.state === 'partial'
                          ? 'Partial source scan'
                          : 'Recorded in index'}
                  </p></td
                >
                <td
                  >{row.fileCount === null
                    ? 'Unavailable'
                    : row.fileCount.toLocaleString()}</td
                ><td class="numeric nowrap"
                  >{row.allocatedBytes === null
                    ? 'Unavailable'
                    : bytes(row.allocatedBytes)}</td
                ><td class="small muted">{modeLabel(row.mode)}</td></tr
              >{/each}
          </tbody>
        </table>
      </div>
      <details>
        <summary>Scope and measurement limits</summary>
        <p class="small muted">{report.qualification}</p>
      </details>
    {/if}
  {:else}
    <button class="text-button" disabled={busy} onclick={back}
      >Back to categories</button
    >
    <h2 tabindex="-1" bind:this={heading}>
      {category?.title ?? 'Category details'}
    </h2>
    {#if category}
      <p class="muted">{category.description}</p>
      <details>
        <summary>Consequences and scope</summary>
        <p>{category.consequences}</p>
      </details>
      <p class="small muted">
        {modeLabel(category.mode)} · recorded {date(
          report?.observedAt ?? null,
        )}. {category.allocatedBytes === null
          ? 'Allocation unavailable'
          : `${bytes(category.allocatedBytes)} indexed allocation`}.
      </p>
      <details>
        <summary>Locations and limits</summary>{#each category.roots as root}<p
            class="path small"
          >
            {root}
          </p>{/each}
        <p class="small muted">{category.qualification}</p>
      </details>
      <div class="button-row">
        {#if category.mode === 'developerReview'}<button
            disabled={busy}
            onclick={() =>
              controller.openRecommendation('Optimization', 'developer')}
            >Open developer review</button
          >{/if}
        {#if category.referenceUrl}<button
            onclick={() =>
              controller.run(() =>
                controller.service.reference(category!.referenceUrl!),
              )}>Apple guidance</button
          ><span class="small muted"
            >Source reviewed {report?.rulesReviewedAt}</span
          >{/if}
      </div>
      {#if category.mode === 'cacheReview'}
        <CacheBrowser {controller} kind={category.kind} />
      {/if}
    {/if}
    {#if pageLoading}<p role="status">Loading indexed category files…</p>{/if}
    {#if page && category}
      <div class="pagination">
        <button
          disabled={!offset || pageLoading || busy}
          onclick={() => (offset = Math.max(0, offset - 100))}
          >Previous category files</button
        ><span class="small muted"
          >Page {Math.floor(offset / 100) + 1} · indexed file rows</span
        ><button
          disabled={page.nextOffset === null || pageLoading || busy}
          onclick={() => {
            if (page?.nextOffset !== null && page?.nextOffset !== undefined)
              offset = page.nextOffset;
          }}>Next category files</button
        >
      </div>
      <div class="table-wrap">
        <table>
          <thead
            ><tr
              ><th>Exact path / modified</th><th class="numeric">Allocated</th
              ><th>Inspect</th></tr
            ></thead
          ><tbody
            >{#each page.files as file}<tr>
                <td
                  ><span class="path selectable">{file.path}</span>
                  <p class="small muted">{date(file.modifiedAt)}</p></td
                ><td class="numeric nowrap">{bytes(file.allocatedBytes)}</td><td
                  ><button
                    disabled={busy}
                    onclick={() =>
                      controller.run(() =>
                        controller.service.reveal(file.path),
                      )}>Finder</button
                  ></td
                >
              </tr>{/each}</tbody
          >
        </table>
      </div>
      {#if !page.files.length}<p class="empty-inline">
          No indexed file rows on this page. This can reflect an empty,
          excluded, unretained or unscanned location; it does not establish
          complete access or a clean category.
        </p>{/if}
      <p class="small muted">{page.qualification}</p>
    {/if}
  {/if}
{/if}
