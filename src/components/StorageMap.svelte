<script lang="ts">
  import { Folder, File } from '@lucide/svelte';
  import type { FileEntry } from '../bindings/FileEntry';
  import type { StorageMap } from '../bindings/StorageMap';
  import type { StorageMapNode } from '../bindings/StorageMapNode';
  import { bytes } from '../format';
  import { storageColor, treemap } from '../treemap';
  import DevelopmentIndicators from './DevelopmentIndicators.svelte';
  let {
    data,
    metric,
    activePath,
    onactive,
    onopen,
    onremaining,
  }: {
    data: StorageMap;
    metric: 'allocated' | 'logical';
    activePath: string | null;
    onactive: (path: string) => void;
    onopen: (entry: FileEntry) => void;
    onremaining: () => void;
  } = $props();
  const size = (node: StorageMapNode) =>
    metric === 'allocated'
      ? node.entry.allocatedBytes
      : node.entry.logicalBytes;
  let remainder = $derived(
    metric === 'allocated'
      ? data.remainderAllocatedBytes
      : data.remainderLogicalBytes,
  );
  let total = $derived(
    metric === 'allocated' ? data.totalAllocatedBytes : data.totalLogicalBytes,
  );
  let layout = $derived(treemap([...data.nodes.map(size), remainder], 1.25));
  let active = $derived(
    activePath === null
      ? data.nodes[0]
      : data.nodes.find((n) => n.entry.path === activePath),
  );
  const label = (node: StorageMapNode) =>
    `${node.entry.isDirectory ? 'Explore folder' : 'Inspect file'} ${node.entry.name}, ${bytes(size(node))} ${metric}${node.fileCount === null ? '' : `, ${node.fileCount.toLocaleString()} files`}`;
</script>

<div class="map-content">
  <div class="map-heading">
    <h3>Folder map</h3>
    {#if total > 0}<span class="muted small">{bytes(total)} {metric}</span>{/if}
  </div>
  {#if total > 0}
    <div
      class="folder-map"
      aria-label={`Folder map by ${metric} size. Equivalent entries are in the ranked contents table.`}
    >
      {#each data.nodes as node, i (node.entry.path)}
        {@const tile = layout[i]}
        {#if tile.width > 0 && tile.height > 0}
          {#if tile.width >= 3 && tile.height >= 4}
            <button
              class="folder-tile"
              class:active={activePath === node.entry.path}
              class:compact={tile.width < 17 || tile.height < 16}
              style={`left:${tile.left}%;top:${tile.top}%;width:${tile.width}%;height:${tile.height}%;--tile:var(--category-${storageColor(node.entry.path)})`}
              aria-label={label(node)}
              title={`${node.entry.path}\n${bytes(size(node))} ${metric}`}
              onmouseenter={() => onactive(node.entry.path)}
              onfocus={() => onactive(node.entry.path)}
              onclick={() => onopen(node.entry)}
            >
              {#if tile.width >= 17 && tile.height >= 16}
                <strong>{node.entry.name}</strong><span
                  >{bytes(size(node))}</span
                >
                {#if tile.width >= 27 && tile.height >= 26 && node.fileCount !== null}<span
                    >{node.fileCount.toLocaleString()} files</span
                  >{/if}
              {/if}
            </button>
          {:else}
            <div
              class="folder-tile compact"
              aria-hidden="true"
              style={`left:${tile.left}%;top:${tile.top}%;width:${tile.width}%;height:${tile.height}%;--tile:var(--category-${storageColor(node.entry.path)})`}
            ></div>
          {/if}
        {/if}
      {/each}
      {#if remainder > 0}
        {@const tile = layout[data.nodes.length]}
        <button
          class="folder-tile remainder"
          style={`left:${tile.left}%;top:${tile.top}%;width:${tile.width}%;height:${tile.height}%`}
          aria-label={`Browse more entries, ${bytes(remainder)} ${metric}`}
          title={`More entries: ${bytes(remainder)} ${metric}`}
          onclick={onremaining}
        >
          {#if tile.width >= 18 && tile.height >= 16}<strong
              >More entries</strong
            ><span>{bytes(remainder)}</span>{/if}
        </button>
      {/if}
    </div>
    <p class="map-caption muted small">
      Area represents {metric} size. Open a folder to see its contents.
    </p>
    {#if remainder > 0}<button
        class="text-button remainder-link"
        onclick={onremaining}
        >Browse more entries · {bytes(remainder)}{data.remainderFileCount ===
        null
          ? ''
          : ` · ${data.remainderFileCount.toLocaleString()} files`}</button
      >{/if}
  {:else}
    <div class="map-unavailable">
      <Folder size={28} />
      <strong
        >{data.fileCount === 0
          ? 'No indexed file bytes here'
          : 'Folder totals unavailable'}</strong
      >
      <p>
        {data.fileCount === 0
          ? 'This view contains no recorded file allocation. Open another folder or review the scan coverage.'
          : 'Completed folder totals are not retained for this view. Individual files may still be available in Large files.'}
      </p>
    </div>
  {/if}
  {#if active}
    <div class="folder-context" aria-label="Selected storage context">
      <div class="context-heading">
        {#if active.entry.isDirectory}<Folder size={16} />{:else}<File
            size={16}
          />{/if}
        <strong title={active.entry.path}>{active.entry.name}</strong>
      </div>
      <p class="context-kind">{active.title}</p>
      <DevelopmentIndicators indicators={active.development} />
      <p class="muted small">{active.reason}</p>
      {#if active.entry.isDirectory && active.fileCount !== null}
        <p class="muted small">
          {active.fileCount.toLocaleString()} files{active.largestFileBytes ===
          null
            ? ''
            : ` · Largest file ${bytes(active.largestFileBytes)} allocated`}
        </p>
      {/if}
      {#if active.smallFileCount !== null && active.smallAllocatedBytes !== null && active.smallFileCount > 1}
        <p class="small-file-insight">
          <strong>{active.smallFileCount.toLocaleString()} small files</strong>
          together occupy <strong>{bytes(active.smallAllocatedBytes)}</strong> allocated.
          Each is 64 KiB or smaller logically.
        </p>
      {:else if active.entry.isDirectory && active.fileCount === null}
        <p class="muted small">
          File counts and small-file totals are unavailable in this saved index.
        </p>
      {/if}
      <p class="muted small">{active.consequences}</p>
    </div>
  {/if}
  <p class="map-qualification muted small">{data.qualification}</p>
</div>

<style>
  .map-content {
    min-width: 0;
    padding: 18px;
  }
  .map-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 12px;
  }
  h3 {
    font-size: 13px;
    margin: 0;
  }
  .folder-map {
    position: relative;
    height: 320px;
    background: var(--tag);
    overflow: hidden;
    border-radius: 6px;
  }
  .folder-tile {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-end;
    min-width: 0;
    min-height: 0;
    padding: 10px;
    gap: 3px;
    border: 1px solid var(--surface);
    border-radius: 3px;
    color: #fff;
    background: var(--tile);
    overflow: hidden;
    text-align: left;
  }
  .folder-tile strong {
    max-width: 100%;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .folder-tile > span {
    font-size: 11px;
    color: #fff;
  }
  button.folder-tile:hover {
    background: var(--tile);
    filter: brightness(1.12);
  }
  .folder-tile.active {
    outline: 2px solid #fff;
    outline-offset: -3px;
  }
  .folder-tile:focus-visible {
    outline: 3px solid #fff;
    outline-offset: -4px;
    z-index: 1;
  }
  .compact {
    padding: 0;
  }
  .remainder {
    --tile: var(--category-6);
  }
  .map-caption {
    margin: 10px 0 0;
  }
  .remainder-link {
    font-size: 11px;
    margin-top: 6px;
    text-align: left;
  }
  .map-unavailable {
    display: flex;
    min-height: 230px;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 12px;
    color: var(--secondary);
    background: var(--tag);
    border-radius: 6px;
    padding: 20px;
  }
  .map-unavailable p {
    font-size: 12px;
    max-width: 38ch;
    margin: 0;
  }
  .folder-context {
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
  .context-heading {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .context-heading strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .context-heading :global(svg) {
    flex-shrink: 0;
    color: var(--secondary);
  }
  .folder-context p {
    margin: 7px 0 0;
  }
  .context-kind {
    font-size: 12px;
  }
  .small-file-insight {
    font-size: 12px;
    line-height: 1.6;
  }
  .map-qualification {
    margin: 16px 0 0;
  }
  @media (min-width: 1380px) {
    .folder-map {
      height: 360px;
    }
  }
</style>
