<script lang="ts">
  import {
    BrushCleaning,
    Search,
    ExternalLink,
    ChevronRight,
    ChevronLeft,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import CacheBrowser from './CacheBrowser.svelte';
  import CacheBulkFlow from './CacheBulkFlow.svelte';
  import type { CacheBulkSummary } from '../bindings/CacheBulkSummary';
  import type { CleanupPlan } from '../bindings/CleanupPlan';
  import type { ProjectCleanupPage } from '../bindings/ProjectCleanupPage';
  import Duplicates from './Duplicates.svelte';
  import Resources from './Resources.svelte';
  import DeveloperArtifacts from './DeveloperArtifacts.svelte';
  import Recommendations from './Recommendations.svelte';
  import CleanupCategories from './CleanupCategories.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Status from '../components/Status.svelte';
  import { bytes, date } from '../format';
  import { tabKeys } from '../keyboard';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let tab = $state('Categories');
  $effect(() => {
    const target = $controller.reviewTarget;
    if (
      target === 'cache' ||
      target === 'developer' ||
      target === 'duplicates'
    ) {
      tab =
        target === 'developer'
          ? 'Developer artifacts'
          : target === 'duplicates'
            ? 'Duplicates'
            : 'Cache files';
      controller.clearReviewTarget();
    }
  });
  let loading = $state(false);
  let error = $state('');
  let revision = $state(0);
  let pending = $state(false);
  let review = $state<CleanupPlan | null>(null);
  let detail = $state<CleanupPlan | null>(null);
  let closed = $state(false);
  let understood = $state(false);
  let records = $state<CleanupPlan[]>([]);
  let historyOffset = $state(0);
  let moreHistory = $state(false);
  let contents = $state<ProjectCleanupPage | null>(null);
  let contentsOffset = $state(0);
  let contentsLoading = $state(false);
  let contentsError = $state('');
  let contentsRevision = $state(0);
  let session = $derived($controller.session);
  let currentPlan = $derived($controller.cleanupPlan);
  $effect(() => {
    const plan = review;
    const offset = contentsOffset;
    void contentsRevision;
    let alive = true;
    contents = null;
    contentsError = '';
    contentsLoading = false;
    if (!plan?.projectCleanup) return;
    contentsLoading = true;
    controller.service
      .projectCleanupEntries(plan.id, offset)
      .then((r) => {
        if (alive) contents = r;
      })
      .catch((e) => {
        if (alive) contentsError = String(e);
      })
      .finally(() => {
        if (alive) contentsLoading = false;
      });
    return () => {
      alive = false;
    };
  });
  let cacheRecords = $state<CacheBulkSummary[]>([]);
  let cacheRecord = $state<CacheBulkSummary | null>(null);
  const cacheHistoryVersion = $derived(
    `${$controller.cacheBatch?.id ?? ''}:${$controller.cacheBatch?.status ?? ''}`,
  );
  $effect(() => {
    const page = historyOffset;
    const view = tab;
    const plan = currentPlan;
    const refresh = revision;
    void cacheHistoryVersion;
    let alive = true;
    if (view !== 'Action history') return;
    void plan;
    void refresh;
    loading = true;
    Promise.all([
      controller.service.cleanupHistory(page, 50),
      controller.service.cacheHistory(page),
    ])
      .then(([r, bulk]) => {
        if (alive) {
          records = r;
          cacheRecords = bulk;
          if (cacheRecord)
            cacheRecord =
              bulk.find((row) => row.id === cacheRecord?.id) ?? cacheRecord;
          moreHistory = r.length === 50 || bulk.length === 50;
        }
      })
      .catch((e) => {
        if (alive) error = String(e);
      })
      .finally(() => {
        if (alive) loading = false;
      });
    return () => {
      alive = false;
    };
  });
  const activeCacheHistoryIds = $derived(
    cacheRecords
      .filter((row) => ['preparing', 'running'].includes(row.status))
      .map((row) => row.id)
      .join(','),
  );
  $effect(() => {
    const ids = activeCacheHistoryIds.split(',').filter(Boolean),
      view = tab;
    if (view !== 'Action history' || !ids.length) return;
    let alive = true,
      inFlight = false;
    const timer = setInterval(async () => {
      if (inFlight) return;
      inFlight = true;
      try {
        const updates = await Promise.all(
          ids.map((id) => controller.service.cacheBatchSummary(id)),
        );
        if (alive) {
          cacheRecords = cacheRecords.map(
            (row) => updates.find((value) => value.id === row.id) ?? row,
          );
          if (cacheRecord)
            cacheRecord =
              updates.find((value) => value.id === cacheRecord?.id) ??
              cacheRecord;
        }
      } catch (reason) {
        if (alive)
          error = `Could not refresh active cache records: ${String(reason)}`;
      } finally {
        inFlight = false;
      }
    }, 1000);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  });
  async function decline() {
    const plan = review;
    review = null;
    if (plan)
      await controller.run(async () => {
        await controller.service.declineCleanup(plan.id);
        revision++;
      });
  }
  async function previewProject(path: string) {
    if (!session) return;
    pending = true;
    error = '';
    try {
      review = await controller.previewProjectCleanup({
        sessionId: session.id,
        paths: [path],
      });
      contentsOffset = 0;
      closed = false;
      understood = false;
    } catch (e) {
      error = `${String(e)} Nothing was moved. Recheck artifacts or rescan the selected project after addressing this condition.`;
    } finally {
      pending = false;
    }
  }
  async function execute() {
    if (!review || !closed || !understood) return;
    const plan = review;
    review = null;
    error = '';
    try {
      await controller.executeCleanup({
        planId: plan.id,
        reviewedAt: plan.createdAt,
        applicationsClosed: closed,
        consequencesAccepted: understood,
      });
    } catch (e) {
      error = `${String(e)} Open Action history and inspect the last record before retrying.`;
    } finally {
      revision++;
    }
  }
</script>

<div class="page-heading">
  <div>
    <h1>Cleanup</h1>
    <p class="muted">
      Review caches, build output and duplicates before removing anything.
    </p>
  </div>
  <button
    onclick={onScan}
    disabled={$controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning}><Search size={16} />Analyze storage</button
  >
</div>
<div
  class="tabs"
  role="tablist"
  tabindex="-1"
  onkeydown={tabKeys}
  aria-label="Optimization views"
>
  {#each ['Categories', 'Cache files', 'Duplicates', 'Developer artifacts', 'Recommended plan', 'Resources', 'Action history'] as name}<button
      role="tab"
      tabindex={tab === name ? 0 : -1}
      aria-selected={tab === name}
      class:active={tab === name}
      onclick={() => {
        tab = name;
        error = '';
      }}>{name}</button
    >{/each}
</div>
{#if error}<div class="inline-error" role="alert">
    <span>{error}</span><button
      onclick={() => {
        error = '';
        revision++;
      }}>Retry loading</button
    >
  </div>{/if}
{#if currentPlan && !$controller.cacheBatch && tab !== 'Action history'}
  <section class="panel cleanup-result" aria-live="polite">
    <div class="section-heading">
      <h2>
        {$controller.cleanupRunning
          ? 'Moving approved items…'
          : 'Last cleanup result'}
      </h2>
      <Status value={currentPlan.status} />
    </div>
    <p>
      {currentPlan.items.filter((i) => i.status === 'movedToTrash').length} of {currentPlan
        .items.length}
      {currentPlan.projectCleanup ? 'folders' : 'files'} verified in Trash.
    </p>
    <p class="muted">{currentPlan.verification}</p>
    <div class="inline-wrap">
      <button onclick={() => (detail = currentPlan)}
        >{currentPlan.projectCleanup
          ? 'Inspect folder results'
          : 'Inspect per-file results'}<ChevronRight size={14} /></button
      >{#if $controller.cleanupRunning}<button
          onclick={() =>
            controller.run(() => controller.service.cancelCleanup())}
          >Stop remaining moves</button
        >{:else}<button
          class="text-button"
          onclick={() => (tab = 'Action history')}>Action history</button
        >{/if}
    </div>
  </section>
{/if}
{#if tab !== 'Action history'}<CacheBulkFlow {controller} />{/if}
{#if ['Categories', 'Cache files'].includes(tab) && ['preparing', 'review', 'running'].includes($controller.cacheBatch?.status ?? '')}
  <!-- The shared flow above owns this stage. -->
{:else if tab === 'Categories'}
  <CleanupCategories {controller} {onScan} {pending} />
{:else if tab === 'Cache files'}
  <section class="cleanup-intro">
    <h2>Choose cache files</h2>
    <p>
      Choose a namespace or individual files, then review the selection before
      moving anything.
    </p>
    {#if session}<p class="small muted">
        Scan recorded {date(session.finishedAt)} · <Status
          value={session.status}
        />
      </p>
      <details>
        <summary>Scope and eligibility</summary>
        <p class="path small">{session.scope}</p>
        <p class="small muted">
          Only indexed individual user-cache files can qualify after fresh
          checks. Links, cloud-only files, exclusions and detected project trees
          remain protected. Indexed allocation is not removable space.
        </p>
      </details>{/if}
  </section>
  {#if !session}<div class="empty-state">
      <h2>Start with a storage scan.</h2>
      <button class="primary" onclick={onScan}>Analyze storage</button>
    </div>{:else if session.status === 'running'}<p role="status">
      Wait for the storage scan to finish.
    </p>{:else}<CacheBrowser {controller} />{/if}
{:else if tab === 'Recommended plan'}<Recommendations
    {controller}
  />{:else if tab === 'Duplicates'}<Duplicates
    {controller}
    onReview={(plan) => {
      review = plan;
      closed = false;
      understood = false;
    }}
  />{:else if tab === 'Resources'}<Resources
    {controller}
  />{:else if tab === 'Developer artifacts'}
  <DeveloperArtifacts
    {controller}
    {onScan}
    onReview={(path) => void previewProject(path)}
  />
{:else}
  <p class="muted">
    Approved plans, declined reviews and per-item outcomes are retained locally.
    A completed move does not guarantee that Finder can still restore the item.
  </p>
  <button disabled={loading} onclick={() => revision++}
    >Refresh action records</button
  >
  {#if cacheRecord}<CacheBulkFlow
      {controller}
      record={cacheRecord.id === $controller.cacheBatch?.id
        ? $controller.cacheBatch
        : cacheRecord}
      onClose={() => (cacheRecord = null)}
    />{/if}
  {#each cacheRecords as batch}<button
      class="history-row"
      onclick={() => (cacheRecord = batch)}
      ><strong
        >Cache group review · {batch.selectedCount.toLocaleString()} files</strong
      ><span>{date(batch.createdAt)} · <Status value={batch.status} /></span
      ></button
    >{/each}

  {#if loading}<p role="status">Loading action records…</p>{/if}
  <section class="panel history-panel">
    {#each records as plan}<button
        class="history-row"
        onclick={() => (detail = plan)}
        ><span class="history-icon"><BrushCleaning size={18} /></span><span
          class="history-description"
          ><strong
            >{plan.projectCleanup
              ? 'Project folder review'
              : plan.duplicateCleanup
                ? 'Duplicate copy review'
                : 'Cache file review'} · {plan.items.length}
            {plan.projectCleanup
              ? plan.items.length === 1
                ? 'folder'
                : 'folders'
              : plan.items.length === 1
                ? 'file'
                : 'files'}</strong
          ><span class="muted small"
            >{date(plan.createdAt)} · {bytes(plan.allocatedBytes)} reviewed allocation</span
          ></span
        ><Status value={plan.status} /><ChevronRight size={15} /></button
      >{:else}{#if !loading && !cacheRecords.length}<div class="empty-state">
          <BrushCleaning size={32} />
          <h2>No action records yet.</h2>
          <p>
            Previewing a selection creates a review record. Nothing changes
            until you approve it.
          </p>
        </div>{/if}{/each}
    <div class="pagination">
      <button
        disabled={historyOffset === 0 || loading}
        onclick={() => (historyOffset = Math.max(0, historyOffset - 50))}
        >Previous</button
      ><span class="muted small">Page {Math.floor(historyOffset / 50) + 1}</span
      ><button
        disabled={!moreHistory || loading}
        onclick={() => (historyOffset += 50)}>Next</button
      >
    </div>
  </section>
{/if}
{#if review}<Dialog
    title={`Move ${review.items.length} selected ${review.projectCleanup ? (review.items.length === 1 ? 'folder' : 'folders') : review.items.length === 1 ? 'file' : 'files'} to Trash?`}
    onclose={() => void decline()}
  >
    <p>
      Review expires {date(review.expiresAt)}. {review.projectCleanup
        ? 'The entire folder, manifests, Git state and worktree relationships are rechecked before moving. Changed or unavailable contents prevent the move.'
        : review.duplicateCleanup
          ? 'Each copy and its retained keeper are freshly hashed and byte-compared before the move. Changed or unavailable contents prevent the move.'
          : 'Files are rechecked immediately before moving. A changed or unavailable file will be skipped.'}
    </p>
    {#if review.duplicateCleanup}<p class="callout">
        {review.verification}
      </p>{/if}
    <p>
      <strong
        >{bytes(review.allocatedBytes)} allocated across the selection.</strong
      > This space is not freed while the files remain in Trash.
    </p>
    <div
      class="cleanup-review-list"
      class:project-review-list={review.projectCleanup}
    >
      {#each review.items as item}<div class="cleanup-review-item">
          <strong class="path selectable">{item.file.path}</strong><span
            class="muted small"
            >{bytes(item.file.allocatedBytes)} allocated · modified {date(
              item.file.modifiedAt,
            )}</span
          >
          <p class="muted small" class:path={review.projectCleanup}>
            {item.file.origin}
          </p>
          <p>{item.file.consequences}</p>
        </div>{/each}
    </div>
    {#if review.projectCleanup}
      <h3>Reviewed folder contents</h3>
      <p class="muted small">
        Exact descendant paths from this fresh review. Folder allocation is the
        sum of regular-file allocation; APFS sharing and Trash affect actual
        capacity.
      </p>
      {#if contentsLoading}<p role="status">Loading reviewed contents…</p>{/if}
      {#if contentsError}<div class="inline-error" role="alert">
          <span>{contentsError}</span><button onclick={() => contentsRevision++}
            >Retry contents</button
          >
        </div>{/if}
      {#if contents}<div class="table-scroll">
          <table>
            <thead
              ><tr
                ><th>Exact path</th><th>Kind</th><th class="numeric"
                  >Allocated</th
                ></tr
              ></thead
            ><tbody
              >{#each contents.entries as entry}<tr
                  ><td class="path selectable cleanup-path">{entry.path}</td><td
                    >{entry.directory ? 'Folder' : 'File'}</td
                  ><td class="numeric nowrap"
                    >{entry.directory ? '—' : bytes(entry.allocatedBytes)}</td
                  ></tr
                >{/each}</tbody
            >
          </table>
        </div>
        <div class="pagination">
          <button
            disabled={contentsOffset === 0 || contentsLoading}
            onclick={() => (contentsOffset = Math.max(0, contentsOffset - 50))}
            >Previous contents</button
          ><span class="muted small"
            >Page {Math.floor(contentsOffset / 50) + 1} · {contents.total}
            {contents.total === 1 ? 'descendant' : 'descendants'}</span
          ><button
            disabled={contentsOffset + 50 >= contents.total || contentsLoading}
            onclick={() => (contentsOffset += 50)}>Next contents</button
          >
        </div>
      {/if}
    {/if}
    <label class="cleanup-consent"
      ><input type="checkbox" bind:checked={closed} /><span
        >{review.projectCleanup
          ? 'I have stopped builds, watchers, editors and tools that use this folder.'
          : 'I have closed the applications that use these files.'}</span
      ></label
    >
    <label class="cleanup-consent"
      ><input type="checkbox" bind:checked={understood} /><span
        >{#if review.projectCleanup}I reviewed the entire folder and its
          contents, including ignored files. They contain no unique work and I
          can regenerate them. Recovery depends on the folder remaining
          available in Trash.{:else if review.duplicateCleanup}I reviewed every
          copy and the keeper. These are not intentional backups I need; I
          accept removal of the selected copies. Recovery depends on the copies
          remaining available in Trash.{:else}I reviewed every path and accept
          that cached data may need to be downloaded or regenerated. Recovery
          depends on these items remaining available in Trash.{/if}</span
      ></label
    >
    <div class="dialog-actions">
      <button onclick={() => void decline()}
        >{review.projectCleanup ? 'Keep folder' : 'Keep files'}</button
      ><button
        class="primary"
        disabled={!closed ||
          !understood ||
          $controller.cleanupRunning ||
          $controller.cleanupPreparing ||
          (review.projectCleanup &&
            (!contents || contentsLoading || !!contentsError))}
        onclick={execute}>Approve and move to Trash</button
      >
    </div>
  </Dialog>{/if}
{#if detail}<Dialog
    title="Cleanup action record"
    onclose={() => (detail = null)}
  >
    <div class="inline-between">
      <span>{date(detail.createdAt)}</span><Status value={detail.status} />
    </div>
    <p>{detail.verification}</p>
    {#each detail.items as item}<div class="cleanup-review-item">
        <div class="inline-between">
          <strong class="path selectable">{item.file.path}</strong><Status
            value={detail.status === 'declined' && item.status === 'pending'
              ? 'declined'
              : item.status}
          />
        </div>
        <p class="muted small">{item.file.origin}</p>
        <p>{item.file.consequences}</p>
        <p>
          {detail.status === 'declined' && item.status === 'pending'
            ? 'Not moved; review declined. Create a fresh review for another attempt.'
            : item.detail}
        </p>
        {#if item.trashPath}<p class="path muted small selectable">
            {item.trashPath}
          </p>
          <button
            onclick={() =>
              controller.run(() => controller.service.reveal(item.trashPath!))}
            >Show in Trash<ExternalLink size={14} /></button
          >{/if}
      </div>{/each}
    <p class="muted small">
      Finder may offer Put Back. If the item was emptied or moved, restoration
      may be unavailable. Coqui never overwrites a replacement file or
      automatically retries an interrupted move.
    </p>
  </Dialog>{/if}
