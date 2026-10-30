<script lang="ts">
  import { tick } from 'svelte';
  import type { createAppStore } from '../state';
  import type { CacheBulkSummary } from '../bindings/CacheBulkSummary';
  import type { CacheBulkPage } from '../bindings/CacheBulkPage';
  import type { CacheGroupPage } from '../bindings/CacheGroupPage';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  let {
    controller,
    record = null,
    onClose = () => {},
  }: {
    controller: ReturnType<typeof createAppStore>;
    record?: CacheBulkSummary | null;
    onClose?: () => void;
  } = $props();
  const batch = $derived(record ?? $controller.cacheBatch);
  const busy = $derived(
    $controller.cleanupPreparing || $controller.cleanupRunning,
  );
  const reviewing = $derived(batch?.status === 'review' && !record);
  let closed = $state(false),
    understood = $state(false),
    cursor = $state<string | null>(null),
    previous = $state<(string | null)[]>([]),
    filter = $state('all');
  let groupCursor = $state<string | null>(null),
    groupPrevious = $state<(string | null)[]>([]);
  let page = $state<CacheBulkPage | null>(null),
    groups = $state<CacheGroupPage | null>(null),
    loading = $state(false),
    error = $state(''),
    revision = $state(0);
  let heading = $state<HTMLHeadingElement>();
  let clock = $state(Date.now());
  $effect(() => {
    if (!reviewing) return;
    clock = Date.now();
    const timer = setInterval(() => (clock = Date.now()), 1000);
    return () => clearInterval(timer);
  });
  const id = $derived(batch?.id ?? ''),
    status = $derived(batch?.status ?? '');
  const browsable = $derived(
    !!batch && !['draft', 'preparing', 'running'].includes(status),
  );
  const expired = $derived(
    !!batch?.expiresAt && new Date(batch.expiresAt).getTime() <= clock,
  );
  $effect(() => {
    void id;
    void status;
    closed = false;
    understood = false;
    cursor = null;
    previous = [];
    filter = 'all';
    groupCursor = null;
    groupPrevious = [];
    if (status === 'review') void tick().then(() => heading?.focus());
  });
  $effect(() => {
    const batchId = id,
      start = cursor,
      selected = filter,
      groupStart = groupCursor;
    void revision;
    let alive = true;
    page = null;
    groups = null;
    error = '';
    loading = false;
    if (!browsable || !batchId) return;
    loading = true;
    Promise.all([
      controller.service.cacheBatchPage({
        id: batchId,
        cursor: start,
        status: selected === 'all' ? null : selected,
      }),
      controller.service.cacheReviewedGroups(batchId, groupStart),
    ])
      .then(([p, g]) => {
        if (alive) {
          page = p;
          groups = g;
        }
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
  async function execute() {
    if (!batch || !reviewing || !closed || !understood || !batch.reviewedAt)
      return;
    const approval = {
      id: batch.id,
      revision: batch.revision,
      reviewedAt: batch.reviewedAt,
      applicationsClosed: closed,
      consequencesAccepted: understood,
    };
    closed = false;
    understood = false;
    await controller.run(() => controller.executeCacheBulk(approval));
  }
</script>

{#if batch}
  <section
    class="panel cache-bulk-flow"
    class:selection-summary={status === 'draft'}
    aria-label="Cache selection and review"
  >
    {#if status === 'draft'}
      <div class="file-toolbar">
        <div>
          <strong>{batch.selectedCount.toLocaleString()} files selected</strong>
          <p class="small muted">
            {bytes(batch.allocatedBytes)} indexed allocation · across all pages
          </p>
        </div>
        <div class="inline-wrap">
          <button
            disabled={busy || !batch.selectedCount}
            onclick={() =>
              controller.run(() =>
                controller.changeCacheSelection(
                  {
                    sessionId: batch!.sessionId,
                    batchId: batch!.id,
                    kind: null,
                    group: null,
                    cursor: null,
                  },
                  null,
                  false,
                  true,
                ),
              )}>Clear selection</button
          ><button
            class="primary"
            disabled={busy || !batch.selectedCount}
            onclick={() => controller.run(() => controller.prepareCacheBulk())}
            >Review selection</button
          >{#if busy}<button
              onclick={() =>
                controller.run(() => controller.service.cancelCleanup())}
              >Cancel selection</button
            >{/if}
        </div>
      </div>
      {#if !busy && batch.detail && batch.detail !== 'Nothing approved'}<p
          class="small muted"
          role="status"
        >
          {batch.detail}
        </p>{/if}
      {#if busy}<p role="status">
          Materializing your selection… {batch.selectedCount.toLocaleString()} files
          recorded so far.
        </p>{/if}
    {:else}
      <div class="section-heading">
        <h2 tabindex="-1" bind:this={heading}>
          {status === 'preparing'
            ? 'Checking selected files…'
            : status === 'running'
              ? 'Moving approved files…'
              : reviewing
                ? 'Review cache selection'
                : 'Cache cleanup result'}
        </h2>
        <Status value={status} />
      </div>
      {#if status === 'preparing'}<p role="status">
          {batch.processedCount.toLocaleString()} of {batch.selectedCount.toLocaleString()}
          checked · {batch.eligibleCount.toLocaleString()} eligible · {batch.refusedCount.toLocaleString()}
          refused.
        </p>
        <progress
          max={Math.max(1, batch.selectedCount)}
          value={batch.processedCount}
          aria-label="Fresh cache checks"
        ></progress>
      {:else}<p>
          <strong>{batch.eligibleCount.toLocaleString()} eligible files</strong>
          · {bytes(batch.reviewedBytes)} reviewed allocation · {batch.refusedCount.toLocaleString()}
          refused.
        </p>{/if}
      {#if !record && (status === 'running' || status === 'preparing')}<button
          onclick={() =>
            controller.run(() => controller.service.cancelCleanup())}
          >{status === 'running'
            ? 'Stop remaining moves'
            : 'Cancel fresh checking'}</button
        >{/if}
      {#if reviewing}<p class="small muted">
          This frozen selection expires {date(batch.expiresAt)}. Cache data may
          need downloading or rebuilding. Moving files to Trash does not free
          space until Trash is emptied.
        </p>{:else if status !== 'preparing'}<p aria-live="polite">
          {batch.movedCount.toLocaleString()} verified in Trash · {batch.skippedCount.toLocaleString()}
          skipped · {batch.failedCount.toLocaleString()} failed · {batch.unverifiedCount.toLocaleString()}
          unverified · {batch.unattemptedCount.toLocaleString()} unattempted.
        </p>{/if}
      <p class="small muted">{batch.detail}</p>
      {#if error}<div class="inline-error" role="alert">
          {error}<button onclick={() => revision++}>Retry review details</button
          >
        </div>{/if}
      {#if loading}<p role="status">Loading exact review details…</p>{/if}
      {#if groups}
        <div class="table-wrap">
          <table>
            <thead
              ><tr
                ><th>Selected namespace</th><th>Eligible files</th><th
                  >Refused</th
                ><th class="numeric">Reviewed allocation</th></tr
              ></thead
            ><tbody
              >{#each groups.groups as group}<tr
                  ><td>{group.title}</td><td
                    >{group.eligibleCount.toLocaleString()}</td
                  ><td>{group.refusedCount.toLocaleString()}</td><td
                    class="numeric">{bytes(group.allocatedBytes)}</td
                  ></tr
                >{/each}</tbody
            >
          </table>
        </div>
        {#if groups.nextCursor !== null || groupPrevious.length}<div
            class="pagination"
          >
            <button
              disabled={!groupPrevious.length || loading}
              onclick={() => {
                groupCursor = groupPrevious.at(-1) ?? null;
                groupPrevious = groupPrevious.slice(0, -1);
              }}>Previous reviewed groups</button
            ><button
              disabled={groups.nextCursor === null || loading}
              onclick={() => {
                groupPrevious = [...groupPrevious, groupCursor];
                groupCursor = groups?.nextCursor ?? null;
              }}>Next reviewed groups</button
            >
          </div>{/if}
      {/if}
      {#if page}
        <details class="cache-review-files">
          <summary>Exact files and outcomes</summary>
          <label class="result-filter"
            >Show <select
              bind:value={filter}
              onchange={() => {
                cursor = null;
                previous = [];
              }}
              ><option value="all">All selected files</option><option
                value="eligible">Eligible files</option
              ><option value="refused">Refused files and reasons</option><option
                value="movedToTrash">Verified moves</option
              ><option value="skipped">Skipped</option><option value="failed"
                >Failed</option
              ><option value="unverified">Unverified</option><option
                value="unattempted">Unattempted</option
              ></select
            ></label
          >
          <div class="table-wrap">
            <table>
              <thead
                ><tr
                  ><th>Exact path</th><th>Outcome / reason</th><th
                    class="numeric">Allocated</th
                  ></tr
                ></thead
              ><tbody
                >{#each page.items as item}<tr
                    ><td
                      ><span class="path selectable">{item.path}</span
                      >{#if item.trashPath}<p class="small path">
                          Trash: {item.trashPath}
                        </p>{/if}</td
                    ><td
                      ><Status value={item.status} />
                      <p class="small muted">{item.detail}</p></td
                    ><td class="numeric nowrap">{bytes(item.allocatedBytes)}</td
                    ></tr
                  >{/each}</tbody
              >
            </table>
          </div>
          {#if !page.items.length}<p>
              No files match this outcome filter.
            </p>{/if}
          <div class="pagination">
            <button
              disabled={!previous.length || loading}
              onclick={() => {
                cursor = previous.at(-1) ?? null;
                previous = previous.slice(0, -1);
              }}>Previous reviewed files</button
            ><span class="small muted">Page {previous.length + 1}</span><button
              disabled={page.nextCursor === null || loading}
              onclick={() => {
                previous = [...previous, cursor];
                cursor = page?.nextCursor ?? null;
              }}>Next reviewed files</button
            >
          </div>
        </details>
      {/if}
      {#if reviewing}
        <label class="cleanup-consent"
          ><input type="checkbox" bind:checked={closed} /> I have closed the applications
          that use these files.</label
        >
        <label class="cleanup-consent"
          ><input type="checkbox" bind:checked={understood} /> I reviewed the selected
          groups and included file scope. I accept that cached data may need downloading
          or rebuilding, and recovery depends on these files remaining in Trash.</label
        >
        <div class="dialog-actions">
          <button
            disabled={busy}
            onclick={() => controller.run(() => controller.keepCacheBulk())}
            >Keep files</button
          ><button
            class="primary"
            disabled={!closed ||
              !understood ||
              busy ||
              expired ||
              loading ||
              !!error ||
              !page ||
              !groups}
            onclick={execute}>Approve and move to Trash</button
          >
        </div>
        {#if expired}<p role="alert">
            This review expired. Start a new selection for fresh checks.
          </p>{/if}
      {:else if browsable}<div class="inline-wrap">
          {#if record}<button onclick={onClose}>Close cache record</button
            >{:else}<button onclick={() => controller.newCacheSelection()}
              >Start a new selection</button
            >{/if}
          <p class="small muted">
            Recovery: inspect the recorded Trash paths in Finder. Put Back may
            be available while the files remain in Trash. No automatic retry.
          </p>
        </div>{/if}
    {/if}
  </section>
{/if}

<style>
  .cache-bulk-flow.selection-summary {
    position: sticky;
    top: 0;
    z-index: 3;
    background: var(--surface);
  }
  .cache-review-files .table-wrap {
    max-height: 300px;
    overflow: auto;
  }
  .cache-bulk-flow {
    margin: 20px 0;
    position: relative;
  }
  .cache-bulk-flow .file-toolbar {
    padding: 0;
  }
  .result-filter {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 12px 0;
  }
  progress {
    width: 100%;
    accent-color: var(--accent);
  }
  h2:focus {
    outline: none;
  }
  .cleanup-consent {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin: 16px 0;
  }
</style>
