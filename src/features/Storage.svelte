<script lang="ts">
  import { tabKeys } from '../keyboard';
  import {
    ArrowUp,
    ArrowRight,
    ChevronRight,
    Folder,
    File,
    Search,
    HardDrive,
    Info,
    X,
    ExternalLink,
    ChevronLeft,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { FileEntry } from '../bindings/FileEntry';
  import { bytes, date } from '../format';
  import Dialog from '../components/Dialog.svelte';
  import { treemap } from '../treemap';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let tab = $state('Explorer');
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
  let session = $derived($controller.session);
  let sessionId = $derived(session?.id);
  let revision = $derived(
    $controller.scanning
      ? Math.floor((session?.fileCount ?? 0) / 250)
      : session?.status,
  );
  let filtered = $derived(
    rows.filter((r) => r.name.toLowerCase().includes(search.toLowerCase())),
  );
  $effect(() => {
    const id = sessionId;
    const rev = revision;
    const p = parent;
    const cat = category;
    const order = sort;
    const desc = descending;
    const page = offset;
    const filesOnly = tab === 'Large files' || !!cat;
    let alive = true;
    if (!id || tab === 'Cache candidates') return;
    void rev;
    loading = true;
    error = '';
    controller.service
      .files({
        sessionId: id,
        parent: filesOnly ? null : p,
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
  function navigate(path: string | null) {
    parent = path;
    category = '';
    offset = 0;
    search = '';
  }
  function pickCategory(name: string) {
    category = name;
    parent = null;
    tab = 'Explorer';
    offset = 0;
    search = '';
  }
  function sortBy(name: string) {
    if (sort === name) descending = !descending;
    else {
      sort = name;
      descending = name !== 'name';
    }
    offset = 0;
  }
  function tiles() {
    const cats = session?.categories ?? [];
    const layout = treemap(cats.map((c) => c.allocatedBytes));
    return cats.map((c, i) => ({ ...c, ...layout[i], color: i % 7 }));
  }
  let segments = $derived(tiles());
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
  <button onclick={onScan} disabled={$controller.scanning}
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
  <section class="panel storage-overview">
    <div class="section-heading">
      <div>
        <h2>Accessible file allocation</h2>
        <p class="muted small">
          {bytes(session.allocatedBytes)} allocated · {session.fileCount.toLocaleString()}
          files · {session.skippedCount.toLocaleString()} skipped entries
        </p>
      </div>
      <span class="subtle-tag"
        >{session.status === 'running'
          ? 'Analysis in progress'
          : 'Read-only results'}</span
      >
    </div>
    {#if segments.length}<div
        class="storage-map"
        aria-label="Allocated storage by category"
      >
        {#each segments as tile}<button
            class="storage-tile"
            class:tile-small={tile.width < 8 || tile.height < 20}
            style={`left:${tile.left}%;top:${tile.top}%;width:${tile.width}%;height:${tile.height}%;--tile:var(--category-${tile.color})`}
            aria-label={`${tile.name}, ${bytes(tile.allocatedBytes)}. Explore files.`}
            title={`${tile.name}: ${bytes(tile.allocatedBytes)}`}
            onclick={() => pickCategory(tile.name)}
            >{#if tile.width > 7 && tile.height > 20}<strong>{tile.name}</strong
              ><span>{bytes(tile.allocatedBytes)}</span>{/if}</button
          >{/each}
      </div>
      <div class="storage-legend">
        {#each segments as tile}<button
            class:active={category === tile.name}
            onclick={() => pickCategory(tile.name)}
            ><span style={`background:var(--category-${tile.color})`}
            ></span>{tile.name}<strong>{bytes(tile.allocatedBytes)}</strong
            ></button
          >{/each}
      </div>{:else}<div class="storage-map empty-map">
        {session.status === 'running'
          ? 'Counting accessible files…'
          : 'No file allocation recorded for this scope.'}
      </div>{/if}
    <p class="muted small">
      This is allocated space for inspected files, not a promise of recoverable
      storage. APFS clones, snapshots, shared volumes, compression and excluded
      areas can make this differ from disk capacity. Hard-linked files are
      counted once.
    </p>
  </section>
  <div
    class="tabs"
    role="tablist"
    tabindex="-1"
    onkeydown={tabKeys}
    aria-label="Storage views"
  >
    {#each ['Explorer', 'Large files', 'Cache candidates'] as name}<button
        role="tab"
        tabindex={tab === name ? 0 : -1}
        aria-selected={tab === name}
        class:active={tab === name}
        onclick={() => {
          tab = name;
          offset = 0;
          category = '';
          parent = '';
        }}
        >{name}{name === 'Cache candidates'
          ? ` (${session.candidates.length})`
          : ''}</button
      >{/each}
  </div>
  {#if tab === 'Cache candidates'}
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
        <h2>No cache candidates recorded.</h2>
        <p>
          Known cache directories may be outside your scope, excluded,
          inaccessible, or not yet scanned.
        </p>
      </div>{/each}
  {:else}
    <section class="panel file-panel">
      <div class="file-toolbar">
        <div class="breadcrumbs">
          <button class="text-button" onclick={() => navigate('')}
            >Scan roots</button
          >{#if parent}<ChevronRight size={13} /><button
              class="text-button path"
              onclick={() =>
                navigate(parent?.substring(0, parent.lastIndexOf('/')) || '')}
              >{parent}</button
            >{/if}{#if category}<ChevronRight size={13} /><span>{category}</span
            >{/if}
        </div>
        <input
          class="compact-search"
          type="search"
          aria-label="Filter the current results page"
          placeholder="Filter this page…"
          bind:value={search}
        />
      </div>
      <div class="file-filters">
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
        >{#if parent}<button
            class="text-button"
            onclick={() =>
              navigate(parent?.substring(0, parent.lastIndexOf('/')) || '')}
            ><ArrowUp size={14} />Up one level</button
          >{/if}<span class="muted small"
          >{loading ? 'Loading…' : `${rows.length} results on this page`}</span
        >
      </div>
      {#if error}<div class="inline-error" role="alert">
          {error}<button
            onclick={() => {
              rows = [];
              offset = 0;
              parent = '';
              category = '';
            }}>Return to scan roots</button
          >
        </div>{/if}
      <div class="table-scroll">
        <table>
          <thead
            ><tr
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
                ><button onclick={() => sortBy('logical')}>Logical size</button
                ></th
              ><th
                ><button onclick={() => sortBy('modified')}>Modified</button
                ></th
              ><th><span class="sr-only">Inspect</span></th></tr
            ></thead
          ><tbody
            >{#each filtered as row}<tr
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
                    onclick={() => (selected = row)}><Info size={16} /></button
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
            Try Scan roots, another category, or expand the scan scope. An older
            completed scan’s index may have been replaced by a newer one.
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
          onclick={() => (offset += 100)}>Next<ChevronRight size={14} /></button
        >
      </div>
    </section>
  {/if}
{/if}
{#if selected}<Dialog title={selected.name} onclose={() => (selected = null)}
    ><p class="path selectable">{selected.path}</p>
    <dl>
      <dt>Allocated space</dt>
      <dd>{bytes(selected.allocatedBytes)}</dd>
      <dt>Logical size</dt>
      <dd>{bytes(selected.logicalBytes)}</dd>
      <dt>Modified</dt>
      <dd>{date(selected.modifiedAt)}</dd>
      <dt>Type</dt>
      <dd>{selected.isDirectory ? 'Directory' : selected.category}</dd>
    </dl>
    <p class="muted">
      Size does not establish whether a file is unnecessary. Coqui has not read
      its contents or selected it for deletion.
    </p>
    <button
      class="primary"
      onclick={() =>
        controller.run(() => controller.service.reveal(selected!.path))}
      >Reveal in Finder<ExternalLink size={15} /></button
    ></Dialog
  >{/if}
