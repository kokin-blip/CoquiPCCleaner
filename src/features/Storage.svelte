<script lang="ts">
  import { tick } from 'svelte';
  import { tabKeys } from '../keyboard';
  import {
    ArrowRight,
    ArrowDown,
    ArrowUp,
    ChevronRight,
    Folder,
    File,
    Search,
    HardDrive,
    Info,
    ExternalLink,
    ChevronLeft,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { FileEntry } from '../bindings/FileEntry';
  import { bytes, date } from '../format';
  import Dialog from '../components/Dialog.svelte';
  import { storageColor } from '../treemap';
  import FolderMap from '../components/StorageMap.svelte';
  import type { StorageMap } from '../bindings/StorageMap';
  import RemovalReview from '../components/RemovalReview.svelte';
  import RemovalHistory from '../components/RemovalHistory.svelte';
  import Coverage from '../components/Coverage.svelte';
  import type { RemovalRequest } from '../bindings/RemovalRequest';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let tab = $state('Explorer');
  let metric = $state<'allocated' | 'logical'>('allocated');
  let map = $state<StorageMap | null>(null);
  let mapLoading = $state(false);
  let mapError = $state('');
  let refresh = $state(0);
  let activePath = $state<string | null>(null);
  let tableHeading: HTMLHeadingElement | undefined = $state();
  let backPaths = $state<string[]>([]);
  let category = $state('');
  let parent = $state<string | null>('');
  let sort = $state('allocated');
  let descending = $state(true);
  let offset = $state(0);
  let rows = $state<FileEntry[]>([]);
  let loading = $state(false);
  let error = $state('');
  let selected = $state<FileEntry | null>(null);
  let search = $state('');
  let selection = $state<string[]>([]),
    request = $state<RemovalRequest | null>(null);
  $effect(() => {
    void sessionId;
    selection = [];
    backPaths = [];
  });
  let session = $derived($controller.session);
  let sessionId = $derived(session?.id);
  let revision = $derived(
    $controller.workRunning || $controller.scanning
      ? Math.floor((session?.fileCount ?? 0) / 250)
      : session?.status,
  );
  let filtered = $derived(rows);
  let explorer = $derived(tab === 'Explorer' && !category && !search.trim());
  let stats = $derived(new Map(map?.nodes.map((n) => [n.entry.path, n]) ?? []));
  let incomplete = $derived(session?.status !== 'complete');
  let candidateResultsUnavailable = $derived(
    incomplete && !session?.categories.length && !session?.candidates.length,
  );
  let crumbs = $derived(
    (parent || '')
      .split('/')
      .filter(Boolean)
      .map((name, i, parts) => ({
        name,
        path: '/' + parts.slice(0, i + 1).join('/'),
      })),
  );
  $effect(() => {
    const id = sessionId;
    const rev = revision;
    const path = parent || '';
    const measure = metric;
    const retry = refresh;
    if (!id || !explorer) return;
    let alive = true;
    void rev;
    void retry;
    mapLoading = true;
    mapError = '';
    map = null;
    controller.service
      .storageMap({ sessionId: id, parent: path, metric: measure, limit: 60 })
      .then((result) => {
        if (alive) map = result;
      })
      .catch((e) => {
        if (alive) mapError = String(e);
      })
      .finally(() => {
        if (alive) mapLoading = false;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const id = sessionId;
    const rev = revision;
    void refresh;
    const p = parent;
    const cat = category;
    const order = sort;
    const desc = descending;
    const page = offset;
    const text = search.trim();
    const filesOnly = tab === 'Large files' || !!cat;
    let alive = true;
    if (
      !id ||
      ['Cache candidates', 'Coverage', 'Removal records'].includes(tab)
    )
      return;
    void rev;
    loading = true;
    rows = [];
    error = '';
    controller.service
      .files({
        sessionId: id,
        search: text || null,
        parent: filesOnly || text ? null : p,
        category: cat || null,
        sort: order,
        descending: desc,
        offset: page,
        limit: 100,
        filesOnly,
      })
      .then((result) => {
        if (alive) rows = result;
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
  async function navigate(path: string | null, remember = true) {
    if (remember && path !== parent) backPaths = [...backPaths, parent || ''];
    parent = path;
    activePath = null;
    category = '';
    offset = 0;
    search = '';
    await tick();
    tableHeading?.focus();
  }
  function openEntry(entry: FileEntry) {
    if (entry.isDirectory) navigate(entry.path);
    else selected = entry;
  }
  function totalsUnavailable(entry: FileEntry) {
    const detail = stats.get(entry.path);
    return (
      entry.isDirectory &&
      (detail?.fileCount === null || (!detail && incomplete)) &&
      !entry.allocatedBytes &&
      !entry.logicalBytes
    );
  }
  function goBack() {
    const path = backPaths[backPaths.length - 1];
    backPaths = backPaths.slice(0, -1);
    navigate(path ?? '', false);
  }
  function sortBy(name: string) {
    if (sort === name) descending = !descending;
    else {
      sort = name;
      descending = name !== 'name';
    }
    offset = 0;
  }
  function browseRemainder() {
    offset = 0;
    sort = metric;
    descending = true;
    tableHeading?.focus();
    tableHeading?.scrollIntoView?.({ block: 'nearest' });
  }
  let rootsAvailable = $derived(
    session?.status === 'complete' ||
      session?.status === 'partial' ||
      session?.status === 'cancelled',
  );
</script>

<div class="page-heading">
  <div>
    <h1>Storage</h1>
    <p class="muted">
      See what’s here. Understand it before deciding what to remove.
    </p>
  </div>
  <button
    onclick={onScan}
    disabled={$controller.workRunning || $controller.scanning}
    ><Search size={16} />Analyze storage</button
  >
</div>
{#if !session}<div class="empty-state">
    <HardDrive size={36} />
    <h2>Your storage, in perspective.</h2>
    <p>
      Scan accessible file metadata to explore folders, large files, and cache
      candidates.
    </p>
    <button class="primary" onclick={onScan}>Start storage analysis</button>
  </div>{:else}
  <div class="storage-summary">
    <div>
      <strong>{bytes(session.allocatedBytes)} allocated</strong>
      <span class="muted"
        >{session.fileCount.toLocaleString()} files · {session.skippedCount.toLocaleString()}
        skipped entries</span
      >
    </div>
    <button class="text-button" onclick={() => (tab = 'Coverage')}
      >{session.status === 'running'
        ? 'Analysis in progress'
        : incomplete
          ? `${session.status[0].toUpperCase()}${session.status.slice(1)} results · Review coverage`
          : 'Review scan coverage'}<ChevronRight size={14} /></button
    >
  </div>
  {#if incomplete && session.status !== 'running'}
    <p class="scope-qualification muted small">
      This saved scan is {session.status}. Folder totals and candidate analysis
      may be unfinished; available results do not describe comprehensive disk
      access.
    </p>
  {/if}
  <div
    class="tabs"
    role="tablist"
    tabindex="-1"
    onkeydown={tabKeys}
    aria-label="Storage views"
  >
    {#each ['Explorer', 'Large files', 'Cache candidates', 'Coverage', 'Removal records'] as name}<button
        role="tab"
        tabindex={tab === name ? 0 : -1}
        aria-selected={tab === name}
        class:active={tab === name}
        onclick={() => {
          tab = name;
          offset = 0;
          category = '';
          parent = '';
          backPaths = [];
          activePath = null;
        }}
        >{name}{name === 'Cache candidates'
          ? !candidateResultsUnavailable
            ? ` (${session.candidates.length})`
            : ''
          : ''}</button
      >{/each}
  </div>
  {#if tab === 'Coverage'}<Coverage
      {controller}
      id={session.id}
    />{:else if tab === 'Removal records'}<RemovalHistory
      {controller}
    />{:else if tab === 'Cache candidates'}
    <div class="callout">
      <Info size={20} />
      <div>
        <strong>Candidates for review, not confirmed junk.</strong>
        <p>
          Known cache locations can contain useful data. Review individual
          eligible files in Optimization before proposing any changes.
        </p>
      </div>
    </div>
    {#each session.candidates as item}<section class="panel">
        <div class="section-heading">
          <h2 class="path">{item.path}</h2>
          <strong class="nowrap">{bytes(item.allocatedBytes)}</strong>
        </div>
        <p>{item.reason}</p>
        <p class="muted">{item.consequences}</p>
        <button onclick={() => controller.navigate('Optimization')}
          >Review individual cache files<ArrowRight size={14} /></button
        >
        <div class="inline-between">
          <span class="subtle-tag">{item.safety}</span><button
            onclick={() =>
              controller.run(() => controller.service.reveal(item.path))}
            >Reveal in Finder<ExternalLink size={14} /></button
          >
        </div>
        <p class="muted small">
          {item.origin} · Modified {date(item.modifiedAt)}
        </p>
      </section>{:else}<div class="empty-state">
        <Folder size={32} />
        <h2>
          {candidateResultsUnavailable
            ? 'Candidate results unavailable.'
            : 'No cache candidates recorded.'}
        </h2>
        <p>
          Known cache directories may be outside your scope, excluded,
          inaccessible, or not yet scanned.
        </p>
      </div>{/each}
  {:else}
    <section class="panel file-panel">
      <div class="file-toolbar">
        <nav class="breadcrumbs" aria-label="Storage folder path">
          {#if explorer}<button
              class="icon-button"
              aria-label="Back to previous folder"
              disabled={!backPaths.length}
              onclick={goBack}><ChevronLeft size={16} /></button
            >{/if}
          <button class="text-button" onclick={() => navigate('')}
            >Scan roots</button
          >
          {#if parent === '/'}<ChevronRight size={13} /><button
              class="text-button"
              onclick={() => navigate('/')}>/</button
            >{/if}
          {#each crumbs as crumb}<ChevronRight size={13} /><button
              class="text-button"
              title={crumb.path}
              aria-current={crumb.path === parent ? 'location' : undefined}
              onclick={() => navigate(crumb.path)}>{crumb.name}</button
            >{/each}
          {#if category}<ChevronRight size={13} /><span>{category}</span>{/if}
        </nav>
        <input
          class="compact-search"
          type="search"
          aria-label="Search all indexed paths"
          placeholder="Search all indexed paths…"
          value={search}
          oninput={(e) => {
            search = e.currentTarget.value;
            offset = 0;
          }}
        />
      </div>
      <div class="file-filters">
        <button
          class="primary"
          disabled={!selection.length ||
            $controller.workRunning ||
            $controller.scanning}
          onclick={() =>
            (request = { paths: [...selection], application: null })}
          >Review removal ({selection.length})</button
        >
        {#if selection.length}<button onclick={() => (selection = [])}
            >Clear selection</button
          >{/if}
        <label
          >Category<select
            value={category}
            onchange={(e) => {
              category = e.currentTarget.value;
              parent = category ? null : '';
              offset = 0;
            }}
            ><option value="">All categories</option
            >{#each session.categories as c}<option value={c.name}
                >{c.name}</option
              >{/each}</select
          ></label
        >
        {#if explorer}<label
            >Size<select
              aria-label="Storage size metric"
              value={metric}
              onchange={(e) => {
                metric = e.currentTarget.value as 'allocated' | 'logical';
                sort = metric;
                descending = true;
                offset = 0;
              }}
              ><option value="allocated">Allocated</option><option
                value="logical">Logical</option
              ></select
            ></label
          >{/if}
        <button
          class="text-button"
          disabled={loading || mapLoading}
          onclick={() => refresh++}>Refresh results</button
        >
        <span class="muted small"
          >{loading ? 'Loading…' : `${rows.length} results on this page`}</span
        >
      </div>
      {#if error}<div class="inline-error" role="alert">
          {error}<button
            onclick={() => {
              refresh++;
            }}>Retry results</button
          >
        </div>{/if}
      {#if explorer}
        <div class="folder-workspace">
          <div class="ranked-contents">
            <div class="ranked-heading">
              <h3
                bind:this={tableHeading}
                tabindex="-1"
                aria-describedby={parent ? 'storage-current-folder' : undefined}
              >
                Ranked contents
              </h3>
              <span class="muted small">Folders include their files</span>
            </div>
            {#if parent}<p
                id="storage-current-folder"
                class="folder-location path selectable"
              >
                {parent}
              </p>{/if}
            <div class="table-scroll contents-scroll" aria-busy={loading}>
              <table aria-label="Ranked folder contents">
                <thead
                  ><tr>
                    <th
                      ><input
                        type="checkbox"
                        aria-label="Select this results page"
                        checked={rows.length > 0 &&
                          rows.every((r) => selection.includes(r.path))}
                        onchange={(e) =>
                          (selection = e.currentTarget.checked
                            ? rows.map((r) => r.path)
                            : [])}
                      /></th
                    >
                    <th
                      aria-sort={sort === 'name'
                        ? descending
                          ? 'descending'
                          : 'ascending'
                        : 'none'}
                      ><button onclick={() => sortBy('name')}
                        >Name{#if sort === 'name'}{#if descending}<ArrowDown
                              size={11}
                            />{:else}<ArrowUp size={11} />{/if}{/if}</button
                      ></th
                    >
                    <th
                      class="numeric"
                      aria-sort={sort === metric
                        ? descending
                          ? 'descending'
                          : 'ascending'
                        : 'none'}
                      ><button onclick={() => sortBy(metric)}
                        >{metric === 'allocated'
                          ? 'Allocated'
                          : 'Logical size'}{#if sort === metric}{#if descending}<ArrowDown
                              size={11}
                            />{:else}<ArrowUp size={11} />{/if}{/if}</button
                      ></th
                    >
                    <th><span class="sr-only">Inspect</span></th>
                  </tr></thead
                >
                <tbody
                  >{#each rows as row}
                    {@const detail = stats.get(row.path)}
                    <tr
                      class:active-entry={activePath === row.path}
                      onmouseenter={() => (activePath = row.path)}
                    >
                      <td
                        ><input
                          type="checkbox"
                          aria-label={`Select ${row.path} for removal review`}
                          checked={selection.includes(row.path)}
                          onchange={(e) =>
                            (selection = e.currentTarget.checked
                              ? [...selection, row.path]
                              : selection.filter((p) => p !== row.path))}
                        /></td
                      >
                      <td
                        ><button
                          class="file-name"
                          title={row.path}
                          onfocus={() => (activePath = row.path)}
                          onclick={() => openEntry(row)}
                          ><span
                            class="path-color"
                            style={`background:var(--category-${storageColor(row.path)})`}
                          ></span>{#if row.isDirectory}<Folder
                              size={16}
                            />{:else}<File size={16} />{/if}<span
                            >{row.name}</span
                          >{#if row.isDirectory}<ChevronRight
                              size={13}
                            />{/if}</button
                        >
                        <span class="entry-context muted"
                          >{detail?.title ||
                            (row.isDirectory
                              ? 'Folder'
                              : row.category)}{detail?.fileCount === null ||
                          detail?.fileCount === undefined
                            ? ''
                            : ` · ${detail.fileCount.toLocaleString()} files`}</span
                        >
                        {#if row.isDirectory && detail?.smallFileCount !== null && detail?.smallFileCount !== undefined && detail.smallAllocatedBytes !== null && detail.smallFileCount > 0}
                          <span
                            class="entry-small-files"
                            title="Each small file has a logical size of 64 KiB or less."
                          >
                            <span
                              >{detail.smallFileCount.toLocaleString()} small files</span
                            >
                            <span
                              >{bytes(detail.smallAllocatedBytes)} allocated</span
                            >
                          </span>
                        {/if}
                      </td>
                      <td class="numeric"
                        >{totalsUnavailable(row)
                          ? 'Unavailable'
                          : bytes(
                              metric === 'allocated'
                                ? row.allocatedBytes
                                : row.logicalBytes,
                            )}</td
                      >
                      <td
                        ><button
                          class="icon-button"
                          title="Inspect exact path and details"
                          aria-label={`Inspect ${row.name}`}
                          onclick={() => (selected = row)}
                          ><Info size={15} /></button
                        ></td
                      >
                    </tr>
                  {/each}</tbody
                >
              </table>
            </div>
            {#if !rows.length && !loading}<div class="empty-inline">
                <Folder size={25} />
                <h3>
                  {incomplete
                    ? 'No folder totals retained here.'
                    : 'No indexed entries in this view.'}
                </h3>
                <p>
                  Try Scan roots or Large files. A newer scan may have replaced
                  this scope’s index.
                </p>
              </div>{/if}
            <div class="pagination">
              <button
                disabled={offset === 0 || loading}
                onclick={() => (offset = Math.max(0, offset - 100))}
                ><ChevronLeft size={14} />Previous</button
              ><span class="muted small"
                >Page {Math.floor(offset / 100) + 1}</span
              ><button
                disabled={rows.length < 100 || loading}
                onclick={() => (offset += 100)}
                >Next<ChevronRight size={14} /></button
              >
            </div>
          </div>
          <div class="folder-visualization" aria-busy={mapLoading}>
            {#if mapLoading}<div class="map-loading" role="status">
                <div class="map-placeholder"></div>
                <span>Loading folder totals…</span>
              </div>
            {:else if mapError}<div class="map-loading">
                <h3>Folder map unavailable</h3>
                <p class="inline-error" role="alert">{mapError}</p>
                <button onclick={() => refresh++}>Retry folder map</button>
                <p class="muted small">
                  Retained individual files may still be available in Large
                  files.
                </p>
              </div>
            {:else if map}<FolderMap
                data={map}
                {metric}
                {activePath}
                onactive={(path) => (activePath = path)}
                onopen={openEntry}
                onremaining={browseRemainder}
              />{/if}
          </div>
        </div>
        <p class="storage-accounting muted small">
          {metric === 'allocated'
            ? 'Allocated space measures filesystem blocks; logical size measures file length.'
            : 'Logical size measures file length; sparse virtual disks may occupy much less allocated space.'}
          Folder totals include small files together. APFS clones, snapshots, shared
          volumes, compression and omitted locations can differ from disk capacity.
          Size and folder type do not establish cleanup eligibility.
        </p>
      {:else}
        <div class="table-scroll">
          <table>
            <thead
              ><tr
                ><th
                  ><input
                    type="checkbox"
                    aria-label="Select this results page"
                    checked={rows.length > 0 &&
                      rows.every((r) => selection.includes(r.path))}
                    onchange={(e) =>
                      (selection = e.currentTarget.checked
                        ? rows.map((r) => r.path)
                        : [])}
                  /></th
                ><th
                  ><button onclick={() => sortBy('name')}
                    >Name {sort === 'name'
                      ? descending
                        ? '↓'
                        : '↑'
                      : ''}</button
                  ></th
                ><th>Category</th><th class="numeric"
                  ><button onclick={() => sortBy('allocated')}
                    >Allocated {sort === 'allocated'
                      ? descending
                        ? '↓'
                        : '↑'
                      : ''}</button
                  ></th
                ><th class="numeric"
                  ><button onclick={() => sortBy('logical')}
                    >Logical size</button
                  ></th
                ><th
                  ><button onclick={() => sortBy('modified')}>Modified</button
                  ></th
                ><th><span class="sr-only">Inspect</span></th></tr
              ></thead
            ><tbody
              >{#each filtered as row}<tr
                  ><td
                    ><input
                      type="checkbox"
                      aria-label={`Select ${row.path} for removal review`}
                      checked={selection.includes(row.path)}
                      onchange={(e) =>
                        (selection = e.currentTarget.checked
                          ? [...selection, row.path]
                          : selection.filter((p) => p !== row.path))}
                    /></td
                  ><td
                    ><button
                      class="file-name"
                      title={row.path}
                      onclick={() =>
                        row.isDirectory ? navigate(row.path) : (selected = row)}
                      >{#if row.isDirectory}<Folder size={16} />{:else}<File
                          size={16}
                        />{/if}<span>{row.name}</span></button
                    ></td
                  ><td class="muted"
                    >{row.isDirectory ? 'Folder' : row.category}</td
                  ><td class="numeric">{bytes(row.allocatedBytes)}</td><td
                    class="numeric muted">{bytes(row.logicalBytes)}</td
                  ><td class="muted nowrap">{date(row.modifiedAt)}</td><td
                    ><button
                      class="icon-button"
                      title="Inspect exact path and details"
                      aria-label={`Inspect ${row.name}`}
                      onclick={() => (selected = row)}
                      ><Info size={16} /></button
                    ></td
                  ></tr
                >{/each}</tbody
            >
          </table>
        </div>
        {#if !rows.length && !loading}<div class="empty-inline">
            <Folder size={25} />
            <h3>
              {rootsAvailable
                ? 'No indexed entries in this view.'
                : 'Folder totals are available when traversal finishes.'}
            </h3>
            <p>
              Try Scan roots, another category, or expand the scan scope. An
              older completed scan’s index may have been replaced by a newer
              one.
            </p>
          </div>{:else if rows.length && !filtered.length}<p
            class="empty-inline muted"
          >
            No matches on this results page.
          </p>{/if}
        <div class="pagination">
          <button
            disabled={offset === 0 || loading}
            onclick={() => (offset = Math.max(0, offset - 100))}
            ><ChevronLeft size={14} />Previous</button
          ><span class="muted small">Page {Math.floor(offset / 100) + 1}</span
          ><button
            disabled={rows.length < 100 || loading}
            onclick={() => (offset += 100)}
            >Next<ChevronRight size={14} /></button
          >
        </div>
      {/if}
    </section>
  {/if}
{/if}
{#if selected}<Dialog title={selected.name} onclose={() => (selected = null)}
    ><p class="path selectable">{selected.path}</p>
    <dl>
      <dt>Allocated space</dt>
      <dd>
        {totalsUnavailable(selected)
          ? 'Unavailable'
          : bytes(selected.allocatedBytes)}
      </dd>
      <dt>Logical size</dt>
      <dd>
        {totalsUnavailable(selected)
          ? 'Unavailable'
          : bytes(selected.logicalBytes)}
      </dd>
      <dt>Modified</dt>
      <dd>{date(selected.modifiedAt)}</dd>
      <dt>Type</dt>
      <dd>{selected.isDirectory ? 'Directory' : selected.category}</dd>
    </dl>
    {#if stats.get(selected.path)}
      {@const detail = stats.get(selected.path)!}
      <h3>{detail.title}</h3>
      <p>{detail.reason}</p>
      <p class="muted">{detail.consequences}</p>
      {#if detail.fileCount !== null}<p class="muted small">
          {detail.fileCount.toLocaleString()} files{detail.smallFileCount ===
            null || detail.smallAllocatedBytes === null
            ? ''
            : ` · ${detail.smallFileCount.toLocaleString()} files at 64 KiB or smaller logically, ${bytes(detail.smallAllocatedBytes)} allocated`}
        </p>{/if}
      {#if detail.qualification}<p class="muted small">
          {detail.qualification}
        </p>{/if}
    {/if}
    <p class="muted">
      Size does not establish whether a file is unnecessary. Coqui has not read
      its contents or selected it for deletion.
    </p>
    <button
      disabled={$controller.workRunning || $controller.scanning}
      onclick={() => {
        request = { paths: [selected!.path], application: null };
        selected = null;
      }}>Review removal</button
    >
    <button
      class="primary"
      onclick={() =>
        controller.run(() => controller.service.reveal(selected!.path))}
      >Reveal in Finder<ExternalLink size={15} /></button
    ></Dialog
  >{/if}

{#if request}<RemovalReview
    {controller}
    {request}
    onclose={() => (request = null)}
    oncomplete={() => (selection = [])}
  />{/if}

<style>
  .storage-summary {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 16px;
    padding: 4px 0 18px;
  }
  .storage-summary > div {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 18px;
    font-size: 12px;
  }
  .storage-summary strong {
    font-size: 14px;
  }
  .storage-summary > button {
    flex-shrink: 0;
    font-size: 12px;
  }
  .scope-qualification {
    margin: -4px 0 18px;
    max-width: 90ch;
  }
  .breadcrumbs {
    overflow-x: auto;
    scrollbar-width: thin;
  }
  .breadcrumbs > :global(*) {
    flex-shrink: 0;
    white-space: nowrap;
  }
  .file-filters {
    flex-wrap: wrap;
    gap: 10px 16px;
  }
  .folder-workspace {
    display: grid;
    grid-template-columns: minmax(350px, 1fr) minmax(330px, 1fr);
    align-items: start;
  }
  .ranked-contents {
    min-width: 0;
    border-right: 1px solid var(--border);
  }
  .ranked-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 18px 16px 8px;
  }
  .ranked-heading h3 {
    font-size: 13px;
    margin: 0;
  }
  .ranked-heading > span {
    text-align: right;
  }
  .folder-location {
    margin: 0;
    padding: 0 16px 8px;
    color: var(--secondary);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  .contents-scroll {
    max-height: 620px;
    overflow-y: auto;
    scrollbar-width: thin;
  }
  .contents-scroll table {
    table-layout: fixed;
  }
  .contents-scroll th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--surface);
  }
  .contents-scroll th:first-child {
    width: 33px;
    padding: 10px 0 10px 12px;
  }
  .contents-scroll td:first-child {
    padding: 10px 0 10px 12px;
  }
  .contents-scroll th:nth-child(3) {
    width: 100px;
  }
  .contents-scroll th:last-child {
    width: 40px;
    padding: 10px 10px 10px 0;
  }
  .contents-scroll td:last-child {
    padding: 8px 10px 8px 0;
  }
  .contents-scroll td {
    padding: 9px 10px;
  }
  .contents-scroll .file-name {
    max-width: 100%;
    width: 100%;
    gap: 6px;
  }
  .contents-scroll .file-name > :global(svg) {
    flex-shrink: 0;
  }
  .contents-scroll .file-name > span:last-of-type {
    flex: 1;
  }
  .path-color {
    width: 6px;
    height: 16px;
    flex-shrink: 0;
    border-radius: 2px;
  }
  .entry-context {
    display: block;
    margin: 3px 0 0 28px;
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-small-files {
    display: flex;
    flex-wrap: wrap;
    gap: 0 8px;
    margin: 3px 0 0 28px;
    font-size: 11px;
    line-height: 1.5;
    color: var(--text);
  }
  .entry-small-files > span {
    white-space: nowrap;
  }
  .active-entry {
    background: var(--selected);
  }
  .folder-visualization {
    min-width: 0;
  }
  .map-loading {
    min-height: 420px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 16px;
    padding: 24px;
    text-align: center;
    font-size: 12px;
    color: var(--secondary);
  }
  .map-placeholder {
    width: 100%;
    height: 240px;
    background: var(--tag);
    border-radius: 6px;
  }
  .map-loading p {
    margin: 0;
    max-width: 42ch;
    overflow-wrap: anywhere;
  }
  .storage-accounting {
    padding: 14px 18px;
    margin: 0;
    border-top: 1px solid var(--border);
  }
  @media (max-width: 1190px) {
    .folder-workspace {
      grid-template-columns: minmax(320px, 1fr) minmax(300px, 1fr);
    }
    .ranked-heading {
      flex-direction: column;
      gap: 4px;
    }
    .storage-summary {
      align-items: flex-start;
    }
    .storage-summary > div {
      flex-direction: column;
      gap: 4px;
    }
    .contents-scroll th:nth-child(3) {
      width: 88px;
    }
    .file-toolbar {
      gap: 12px;
    }
    .compact-search {
      width: 170px;
      flex-shrink: 0;
    }
  }
  @media (max-width: 960px) {
    .folder-workspace {
      grid-template-columns: 1fr;
    }
    .ranked-contents {
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }
    .contents-scroll {
      max-height: 400px;
    }
    .storage-summary {
      flex-wrap: wrap;
    }
  }
</style>
