<script lang="ts">
  import {
    BrushCleaning,
    Search,
    ExternalLink,
    ChevronRight,
    ChevronLeft,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { CleanupFile } from '../bindings/CleanupFile';
  import type { CleanupPlan } from '../bindings/CleanupPlan';
  import type { ProjectCleanupPage } from '../bindings/ProjectCleanupPage';
  import DeveloperArtifacts from './DeveloperArtifacts.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Status from '../components/Status.svelte';
  import { bytes, date } from '../format';
  import { tabKeys } from '../keyboard';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let tab = $state('Cache files');
  let rows = $state<CleanupFile[]>([]);
  let selected = $state<string[]>([]);
  let offset = $state(0);
  let nextOffset = $state<number | null>(null);
  let unavailable = $state(0);
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
  let lastSession = '';
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
  $effect(() => {
    const view = tab;
    const id = session?.id;
    const status = session?.status;
    const page = offset;
    const refresh = revision;
    const settings = $controller.settings;
    let alive = true;
    void refresh;
    void settings;
    if (view !== 'Cache files') return;
    if (!id || status === 'running') {
      rows = [];
      return;
    }
    if (lastSession !== id) {
      lastSession = id;
      selected = [];
      offset = 0;
    }
    loading = true;
    controller.service
      .cleanupFiles(id, page)
      .then((r) => {
        if (alive) {
          rows = r.files;
          nextOffset = r.nextOffset;
          unavailable = r.unavailableCount;
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
  $effect(() => {
    const page = historyOffset;
    const view = tab;
    const plan = currentPlan;
    const refresh = revision;
    let alive = true;
    if (view !== 'Action history') return;
    void plan;
    void refresh;
    loading = true;
    controller.service
      .cleanupHistory(page, 50)
      .then((r) => {
        if (alive) {
          records = r;
          moreHistory = r.length === 50;
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
  function toggle(path: string) {
    selected = selected.includes(path)
      ? selected.filter((p) => p !== path)
      : [...selected, path];
  }
  async function preview() {
    if (!session) return;
    pending = true;
    error = '';
    try {
      review = await controller.service.previewCleanup({
        sessionId: session.id,
        paths: [...selected],
      });
      closed = false;
      understood = false;
    } catch (e) {
      error = String(e);
      revision++;
    } finally {
      pending = false;
    }
  }
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
    selected = [];
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
    <h1>Optimization</h1>
    <p class="muted">
      Choose what to change. Review each path before anything moves.
    </p>
  </div>
  <button
    onclick={onScan}
    disabled={$controller.scanning ||
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
  {#each ['Cache files', 'Developer artifacts', 'Action history'] as name}<button
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
{#if currentPlan && tab !== 'Action history'}
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
{#if tab === 'Cache files'}
  <section class="cleanup-intro">
    <h2>Review individual cache files</h2>
    <p>
      Files listed here were indexed by your scan and are currently readable in
      your Library/Caches. They are candidates, not confirmed junk. Close the
      related apps before proceeding.
    </p>
    {#if session}<p class="path muted small">
        Scan scope: {session.scope}
      </p>{/if}
    <p class="muted small">
      Folders, links, cloud-only files, excluded locations and detected project
      trees are omitted. A standalone home package.json does not exclude these
      caches; declared workspaces remain protected. No files are selected
      automatically. Keep everything is a valid choice.
    </p>
  </section>
  {#if !session}<div class="empty-state">
      <BrushCleaning size={34} />
      <h2>Start with a storage analysis.</h2>
      <p>
        Scan your home folder or select a folder inside your Library/Caches.
        Then review individual files here.
      </p>
      <button class="primary" onclick={onScan}>Analyze storage</button>
    </div>
  {:else if $controller.scanning}<div class="empty-inline" role="status">
      <h2>Analysis is in progress.</h2>
      <p>Eligible files will be available when the scan finishes.</p>
    </div>
  {:else}
    <section class="panel file-panel">
      <div class="file-toolbar">
        <div>
          <strong>{selected.length} selected</strong><span class="muted small">
            · up to 100 files per review</span
          >
        </div>
        <div class="inline-wrap">
          <button
            disabled={!selected.length ||
              pending ||
              $controller.cleanupRunning ||
              $controller.cleanupPreparing ||
              $controller.developerRunning}
            onclick={() => (selected = [])}>Clear selection</button
          ><button
            class="primary"
            disabled={!selected.length ||
              pending ||
              $controller.cleanupRunning ||
              $controller.cleanupPreparing ||
              loading}
            onclick={preview}
            >{pending
              ? 'Preparing review…'
              : 'Review selected files'}<ChevronRight size={14} /></button
          >
        </div>
      </div>
      {#if loading}<p class="empty-inline" role="status">
          Checking indexed files against the current cleanup rules…
        </p>{/if}
      <div class="table-scroll">
        <table>
          <thead
            ><tr
              ><th><span class="sr-only">Select</span></th><th>Exact path</th
              ><th class="numeric">Allocated</th><th>Modified</th><th
                ><span class="sr-only">Reveal</span></th
              ></tr
            ></thead
          ><tbody>
            {#each rows as file}<tr
                ><td
                  ><input
                    type="checkbox"
                    aria-label={`Select ${file.path}`}
                    checked={selected.includes(file.path)}
                    disabled={$controller.cleanupRunning ||
                      $controller.cleanupPreparing ||
                      (!selected.includes(file.path) && selected.length >= 100)}
                    onchange={() => toggle(file.path)}
                  /></td
                ><td class="cleanup-path path selectable">{file.path}</td><td
                  class="numeric nowrap">{bytes(file.allocatedBytes)}</td
                ><td class="nowrap muted">{date(file.modifiedAt)}</td><td
                  ><button
                    class="icon-button"
                    aria-label={`Reveal ${file.path}`}
                    onclick={() =>
                      controller.run(() =>
                        controller.service.reveal(file.path),
                      )}><ExternalLink size={15} /></button
                  ></td
                ></tr
              >{/each}
          </tbody>
        </table>
      </div>
      {#if !rows.length && !loading}<div class="empty-inline">
          <h3>No eligible cache files on this page.</h3>
          <p>
            The current scan may not include your Library/Caches, its index may
            have been replaced, or the files may have changed. Review scan scope
            or continue to the next page when available.
          </p>
          <button onclick={() => controller.navigate('Settings')}
            >Review scan scope</button
          >
        </div>{/if}
      {#if unavailable}<p class="muted small cleanup-qualification">
          {unavailable} indexed {unavailable === 1
            ? 'entry on this page was'
            : 'entries on this page were'} omitted because they are unavailable or
          outside the supported cleanup rules.
        </p>{/if}
      <div class="pagination">
        <button
          disabled={offset === 0 || loading || $controller.cleanupRunning}
          onclick={() => (offset = Math.max(0, offset - 100))}
          ><ChevronLeft size={14} />Previous</button
        ><span class="muted small"
          >Indexed page {Math.floor(offset / 100) + 1}</span
        ><button
          disabled={nextOffset === null ||
            loading ||
            $controller.cleanupRunning}
          onclick={() => {
            if (nextOffset !== null) offset = nextOffset;
          }}>Next<ChevronRight size={14} /></button
        >
      </div>
    </section>
  {/if}
{:else if tab === 'Developer artifacts'}
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
      >{:else}{#if !loading}<div class="empty-state">
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
        : 'Files are rechecked immediately before moving. A changed or unavailable file will be skipped.'}
    </p>
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
          available in Trash.{:else}I reviewed every path and accept that cached
          data may need to be downloaded or regenerated. Recovery depends on
          these items remaining available in Trash.{/if}</span
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
