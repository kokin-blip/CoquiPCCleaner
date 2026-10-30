<script lang="ts">
  import { tick } from 'svelte';
  import type { createAppStore } from '../state';
  import type { CleanupCategoryKind } from '../bindings/CleanupCategoryKind';
  import type { CacheGroupPage } from '../bindings/CacheGroupPage';
  import type { CacheBulkPage } from '../bindings/CacheBulkPage';
  import type { CacheQuery } from '../bindings/CacheQuery';
  import { bytes, date } from '../format';
  let {
    controller,
    kind = null,
  }: {
    controller: ReturnType<typeof createAppStore>;
    kind?: CleanupCategoryKind | null;
  } = $props();
  let groups = $state<CacheGroupPage | null>(null),
    page = $state<CacheBulkPage | null>(null);
  let group = $state<string | null>(null),
    cursor = $state<string | null>(null),
    groupCursor = $state<string | null>(null);
  let previous = $state<(string | null)[]>([]),
    groupPrevious = $state<(string | null)[]>([]);
  let loading = $state(false),
    error = $state(''),
    refresh = $state(0);
  let heading = $state<HTMLHeadingElement>();
  let groupButtons = $state<Record<string, HTMLButtonElement>>({});
  let returnFocus = $state<string | null>(null);
  $effect(() => {
    if (!groups || returnFocus === null) return;
    const key = returnFocus;
    returnFocus = null;
    void tick().then(() => groupButtons[key]?.focus());
  });
  const sessionId = $derived($controller.session?.id ?? '');
  const sourceStatus = $derived($controller.session?.status ?? '');
  const exclusions = $derived(
    JSON.stringify($controller.settings?.excludedPaths ?? []),
  );
  const batch = $derived($controller.cacheBatch);
  const revision = $derived(batch?.revision ?? 0);
  const draftId = $derived(batch?.status === 'draft' ? batch.id : null);
  const busy = $derived(
    $controller.scanning ||
      $controller.workRunning ||
      $controller.cleanupPreparing ||
      $controller.cleanupRunning ||
      $controller.developerRunning,
  );
  const locked = $derived(busy || batch?.status === 'review');
  const allPage = $derived(
    !!page?.items.length && page.items.every((row) => row.selected),
  );
  const somePage = $derived(!!page?.items.some((row) => row.selected));
  const query = (): CacheQuery => ({
    sessionId,
    batchId: draftId,
    kind,
    group,
    cursor,
  });
  $effect(() => {
    void sessionId;
    void exclusions;
    void kind;
    group = null;
    cursor = null;
    groupCursor = null;
    previous = [];
    groupPrevious = [];
    returnFocus = null;
  });
  $effect(() => {
    const id = sessionId,
      selectedGroup = group,
      start = cursor,
      groupStart = groupCursor,
      batchId = draftId,
      categoryKind = kind;
    void revision;
    void exclusions;
    void refresh;
    let alive = true;
    error = '';
    loading = false;
    page = null;
    groups = null;
    if (!id || sourceStatus === 'running' || busy) return;
    loading = true;
    const q: CacheQuery = {
      sessionId: id,
      batchId,
      kind: categoryKind,
      group: selectedGroup,
      cursor: selectedGroup === null ? groupStart : start,
    };
    const run =
      selectedGroup === null
        ? controller.service.cacheGroups(q).then((value) => {
            if (alive) groups = value;
          })
        : controller.service.cacheEntries(q).then((value) => {
            if (alive) page = value;
          });
    run
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
  async function open(key: string) {
    group = key;
    cursor = null;
    previous = [];
    await tick();
    heading?.focus();
  }
  async function select(paths: string[] | null, selected = true) {
    await controller.run(() =>
      controller.changeCacheSelection(query(), paths, selected),
    );
  }
</script>

<section aria-label="Cache candidates">
  {#if error}<div class="inline-error" role="alert">
      {error}<button onclick={() => refresh++}>Retry cache view</button>
    </div>{/if}
  {#if group !== null}<button
      class="text-button"
      disabled={busy}
      onclick={() => {
        returnFocus = group;
        group = null;
        cursor = null;
        previous = [];
      }}>Back to cache groups</button
    >{/if}
  <div class="section-heading">
    <div>
      <h3 tabindex="-1" bind:this={heading}>
        {group === null
          ? 'Choose cache groups'
          : group || 'Files directly in Caches'}
      </h3>
      <p class="small muted">
        Namespace names identify cache locations. Application ownership is
        unverified. {#if group === null}Select all cache groups includes every
          namespace in this cache category, across all pages.{/if}
      </p>
    </div>
    <button
      disabled={locked ||
        loading ||
        !!error ||
        (!groups?.groups.length && !page?.items.length)}
      onclick={() => select(null)}
      >{group === null
        ? 'Select all cache groups'
        : 'Select all in this group'}</button
    >
  </div>
  {#if loading}<p role="status">Reading indexed cache candidates…</p>{/if}
  {#if groups}
    <div class="table-wrap">
      <table>
        <thead
          ><tr
            ><th>Select group</th><th>Cache namespace</th><th>Indexed files</th
            ><th class="numeric">Allocated</th></tr
          ></thead
        ><tbody>
          {#each groups.groups as row}<tr
              ><td
                ><input
                  type="checkbox"
                  aria-label={`Select all in ${row.title}`}
                  checked={row.selectedCount === row.fileCount &&
                    row.fileCount > 0}
                  indeterminate={row.selectedCount > 0 &&
                    row.selectedCount < row.fileCount}
                  disabled={locked}
                  onchange={(event) =>
                    controller.run(() =>
                      controller.changeCacheSelection(
                        { ...query(), group: row.key },
                        null,
                        event.currentTarget.checked,
                      ),
                    )}
                /></td
              ><td
                ><button
                  class="text-button"
                  bind:this={groupButtons[row.key]}
                  onclick={() => open(row.key)}>{row.title}</button
                >
                <p class="small muted">
                  {row.selectedCount.toLocaleString()} selected across all pages
                </p></td
              ><td>{row.fileCount.toLocaleString()}</td><td
                class="numeric nowrap">{bytes(row.allocatedBytes)}</td
              ></tr
            >{/each}
        </tbody>
      </table>
    </div>
    {#if !groups.groups.length}<p class="empty-inline">
        No indexed cache candidates in this scope. Missing or excluded locations
        remain unknown.
      </p>{/if}
    <div class="pagination">
      <button
        disabled={!groupPrevious.length || loading || busy}
        onclick={() => {
          groupCursor = groupPrevious.at(-1) ?? null;
          groupPrevious = groupPrevious.slice(0, -1);
        }}>Previous cache groups</button
      ><span class="small muted">Group page {groupPrevious.length + 1}</span
      ><button
        disabled={groups.nextCursor === null || loading || busy}
        onclick={() => {
          groupPrevious = [...groupPrevious, groupCursor];
          groupCursor = groups?.nextCursor ?? null;
        }}>Next cache groups</button
      >
    </div>
  {/if}
  {#if page}
    <div class="inline-wrap cache-page-tools">
      <label
        ><input
          type="checkbox"
          checked={allPage}
          indeterminate={somePage && !allPage}
          disabled={locked || !page.items.length}
          onchange={(event) =>
            select(
              page!.items.map((row) => row.path),
              event.currentTarget.checked,
            )}
        /> Select this page</label
      ><span class="small muted"
        >Group selection includes every page; no 100-file review cap.</span
      >
    </div>
    <div class="table-wrap">
      <table>
        <thead
          ><tr
            ><th>Select</th><th>Exact path / modified</th><th class="numeric"
              >Allocated</th
            ><th>Inspect</th></tr
          ></thead
        ><tbody>
          {#each page.items as file}<tr
              ><td
                ><input
                  type="checkbox"
                  aria-label={`Select ${file.path}`}
                  checked={file.selected}
                  disabled={locked}
                  onchange={(event) =>
                    select([file.path], event.currentTarget.checked)}
                /></td
              ><td
                ><span class="path selectable">{file.path}</span>
                <p class="small muted">{date(file.modifiedAt)}</p></td
              ><td class="numeric nowrap">{bytes(file.allocatedBytes)}</td><td
                ><button
                  onclick={() =>
                    controller.run(() => controller.service.reveal(file.path))}
                  >Finder</button
                ></td
              ></tr
            >{/each}
        </tbody>
      </table>
    </div>
    {#if !page.items.length}<p class="empty-inline">
        No indexed files on this page. This does not establish complete access
        or cleanup eligibility.
      </p>{/if}
    <div class="pagination">
      <button
        disabled={!previous.length || loading || busy}
        onclick={() => {
          cursor = previous.at(-1) ?? null;
          previous = previous.slice(0, -1);
        }}>Previous cache files</button
      ><span class="small muted">File page {previous.length + 1}</span><button
        disabled={page.nextCursor === null || loading || busy}
        onclick={() => {
          previous = [...previous, cursor];
          cursor = page?.nextCursor ?? null;
        }}>Next cache files</button
      >
    </div>
  {/if}
</section>

<style>
  .cache-page-tools {
    padding: 12px 0;
  }
  .cache-page-tools label {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  h3:focus {
    outline: none;
  }
</style>
