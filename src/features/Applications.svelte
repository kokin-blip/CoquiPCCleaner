<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { Package, Search, FolderOpen, ExternalLink } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { ApplicationFootprint } from '../bindings/ApplicationFootprint';
  import type { ApplicationPreferences } from '../bindings/ApplicationPreferences';
  import type { RemovalRequest } from '../bindings/RemovalRequest';
  import { tabKeys } from '../keyboard';
  import { bytes, date } from '../format';
  import RemovalReview from '../components/RemovalReview.svelte';
  import RemovalHistory from '../components/RemovalHistory.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let tab = $state('Installed Apps'),
    query = $state(''),
    sort = $state('name'),
    offset = $state(0),
    revision = $state(0);
  let rows = $state<ApplicationFootprint[]>([]),
    total = $state(0),
    qualification = $state(''),
    loading = $state(false),
    error = $state('');
  let selected = $state<ApplicationFootprint | null>(null),
    paths = $state<string[]>([]),
    request = $state<RemovalRequest | null>(null);
  let chooseButton: HTMLButtonElement;
  let inspectionHeading = $state<HTMLHeadingElement>();
  let returnTab = 'Installed Apps';
  let returnControl: HTMLElement | null = null;
  let returnPath: string | undefined;
  let removalCompleted = false;
  let administratorInspection = $state(false);
  let leftovers = $state<ApplicationFootprint[]>([]);
  let leftoverOffset = $state(0);
  let prefs = $state<ApplicationPreferences>({
    protectedPaths: [],
    disappearanceNotices: false,
  });
  async function refreshInventory() {
    error = '';
    try {
      await controller.work('Refreshing application inventory', () =>
        controller.service.refreshApplicationInventory(),
      );
      revision++;
    } catch (e) {
      error = String(e);
    }
  }
  onMount(() => {
    void refreshInventory();
    void controller.service
      .applicationPreferences()
      .then((p) => (prefs = p))
      .catch((e) => (error = String(e)));
    let disposed = false;
    let unlisten: (() => void) | undefined;
    if (isTauri())
      void getCurrentWebviewWindow()
        .onDragDropEvent((event) => {
          if (event.payload.type === 'drop') {
            const path = event.payload.paths.find((p) => p.endsWith('.app'));
            if (path) void inspect(path);
            else error = 'Drop a macOS .app bundle for inspection.';
          }
        })
        .then((fn) => {
          if (disposed) fn();
          else unlisten = fn;
        })
        .catch((e) => (error = String(e)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
  $effect(() => {
    const q = query;
    const order = sort;
    const page = offset;
    const protectedOnly = tab === 'Protected Apps';
    void revision;
    let alive = true;
    loading = true;
    error = '';
    controller.service
      .applicationPage(q, order, page, protectedOnly)
      .then((r) => {
        if (alive) {
          rows = r.applications;
          total = r.total;
          qualification = r.qualification;
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
    const current = tab;
    void revision;
    const page = leftoverOffset;
    if (current !== 'Leftovers') return;
    controller.service
      .applicationLeftovers(page)
      .then((r) => (leftovers = r))
      .catch((e) => (error = String(e)));
  });
  let visible = $derived(rows);
  async function inspect(
    path: string,
    origin: HTMLElement = chooseButton,
    administrator = false,
  ) {
    error = '';
    const originTab = selected ? returnTab : tab;
    const originControl = selected ? returnControl : origin;
    const originPath = selected ? returnPath : origin?.dataset.applicationPath;
    try {
      const app = await controller.work('Inspecting application', () =>
        administrator
          ? controller.service.administratorInspectApplication(path)
          : controller.service.inspectApplication(path),
      );
      returnTab = originTab;
      returnControl = originControl;
      returnPath = originPath;
      administratorInspection = administrator;
      tab = 'Installed Apps';
      selected = app;
      paths = [];
      await tick();
      inspectionHeading?.scrollIntoView?.({ block: 'start' });
      inspectionHeading?.focus({ preventScroll: true });
    } catch (e) {
      error = String(e);
    }
  }
  async function choose(origin: HTMLElement = chooseButton) {
    const path = await controller.service.selectApplication();
    if (path) await inspect(path, origin);
  }
  async function leaveInspection() {
    selected = null;
    paths = [];
    tab = returnTab;
    await tick();
    const origin = returnControl?.isConnected
      ? returnControl
      : ([
          ...document.querySelectorAll<HTMLButtonElement>(
            '[data-application-path]',
          ),
        ].find((button) => button.dataset.applicationPath === returnPath) ??
        chooseButton);
    origin?.scrollIntoView?.({ block: 'nearest' });
    origin?.focus({ preventScroll: true });
  }
  async function protect(app: ApplicationFootprint) {
    try {
      const next = {
        ...prefs,
        protectedPaths: prefs.protectedPaths.includes(app.path)
          ? prefs.protectedPaths.filter((p) => p !== app.path)
          : [...prefs.protectedPaths, app.path],
      };
      await controller.service.saveApplicationPreferences(next);
      prefs = next;
      await refreshInventory();
      await leaveInspection();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-heading">
  <div>
    <h1>Applications</h1>
    <p class="muted">
      Inspect a bundle and review its associated data separately.
    </p>
  </div>
  <button
    bind:this={chooseButton}
    onclick={(e) => {
      const origin = e.currentTarget;
      void controller.run(() => choose(origin));
    }}
    disabled={$controller.workRunning || $controller.scanning}
    ><FolderOpen size={16} />Choose application…</button
  >
</div>
<div
  class="tabs"
  role="tablist"
  tabindex="-1"
  aria-label="Application views"
  onkeydown={tabKeys}
>
  {#each ['Installed Apps', 'Leftovers', 'Protected Apps', 'Removal records'] as name}<button
      role="tab"
      aria-selected={tab === name}
      tabindex={tab === name ? 0 : -1}
      class:active={tab === name}
      onclick={() => {
        tab = name;
        selected = null;
        paths = [];
        offset = 0;
      }}>{name}</button
    >{/each}
</div>
{#if error}<div class="inline-error" role="alert">
    {error}<button onclick={() => revision++}>Retry inventory</button>
  </div>{/if}
{#if tab === 'Removal records' && !selected}<RemovalHistory {controller} />
{:else if tab === 'Leftovers' && !selected}<section class="panel">
    <h2>Review associated files</h2>
    <p>
      Choose an application or inspect it from Installed Apps to review exact
      bundle-identity associations. Related data starts unselected. Shared
      containers and unconfirmed leftovers have no automatic removal authority.
    </p>
    <p class="muted">
      Applications that have disappeared cannot be assumed to be uninstalled.
      Disconnected locations and moved bundles also appear missing.
    </p>
    <label class="check-row"
      ><input
        type="checkbox"
        checked={prefs.disappearanceNotices}
        onchange={async (e) => {
          const next = {
            ...prefs,
            disappearanceNotices: e.currentTarget.checked,
          };
          try {
            await controller.service.saveApplicationPreferences(next);
            prefs = next;
          } catch (e) {
            error = String(e);
          }
        }}
      />Offer related-file review when a previously observed app is missing at
      app open</label
    ><button
      onclick={(e) => {
        const origin = e.currentTarget;
        void controller.run(() => choose(origin));
      }}>Inspect an application’s files…</button
    >
    {#each leftovers as app}<div class="related-file">
        <strong>{app.name}</strong>
        <p class="path selectable">{app.path}</p>
        <p class="muted">{app.protection.join('; ')}</p>
        {#each app.related as item}<p class="path small selectable">
            {item.path} · {item.confidence}
          </p>
          <p class="muted small">{item.consequences}</p>
          <button
            onclick={() =>
              controller.run(() => controller.service.reveal(item.path))}
            >Reveal retained association</button
          >{/each}
      </div>{:else}<p>
        No missing previously observed bundles. Refresh Installed Apps to
        establish observations.
      </p>{/each}
    <div class="pagination">
      <button
        disabled={leftoverOffset === 0}
        onclick={() => (leftoverOffset = Math.max(0, leftoverOffset - 50))}
        >Previous missing apps</button
      ><span>Page {Math.floor(leftoverOffset / 50) + 1}</span><button
        disabled={leftovers.length < 50}
        onclick={() => (leftoverOffset += 50)}>Next missing apps</button
      >
    </div>
  </section>
{:else if !selected}<section class="panel file-panel">
    <div class="file-toolbar">
      <label
        ><Search size={14} />
        <input
          type="search"
          aria-label="Search installed applications"
          placeholder="Search name, bundle ID or path…"
          value={query}
          oninput={(e) => {
            query = e.currentTarget.value;
            offset = 0;
          }}
        /></label
      ><label
        >Sort <select bind:value={sort}
          ><option value="name">Name</option><option value="path">Path</option
          ></select
        ></label
      ><button
        disabled={loading || $controller.workRunning}
        onclick={refreshInventory}>Refresh</button
      >
    </div>
    {#if loading}<p role="status">Reading installed bundle metadata…</p>{/if}
    <div class="table-scroll">
      <table>
        <thead
          ><tr
            ><th>Application</th><th>Version</th><th>Protection / guidance</th
            ><th>Review</th></tr
          ></thead
        ><tbody
          >{#each visible as app}<tr
              ><td
                ><strong>{app.name}</strong>
                <p class="muted small path">{app.path}</p>
                <span class="small selectable">{app.bundleId}</span></td
              ><td>{app.version || 'Unavailable'}</td><td class="small"
                >{app.protection.length
                  ? app.protection.join('; ')
                  : 'No detected protection; rechecked before each move'}</td
              ><td
                ><button
                  data-application-path={app.path}
                  disabled={$controller.workRunning || $controller.scanning}
                  onclick={(e) => inspect(app.path, e.currentTarget)}
                  >Inspect</button
                ></td
              ></tr
            >{/each}</tbody
        >
      </table>
    </div>
    {#if !visible.length && !loading}<div class="empty-inline">
        <Package size={24} />
        <h3>No applications in this view.</h3>
        <p>
          Try another search, refresh the inventory, or choose a .app bundle.
        </p>
      </div>{/if}
    <p class="muted small">{qualification}</p>
    <div class="pagination">
      <button
        disabled={offset === 0 || loading}
        onclick={() => (offset = Math.max(0, offset - 50))}>Previous</button
      ><span>{offset + 1}–{Math.min(offset + 50, total)} of {total}</span
      ><button
        disabled={offset + 50 >= total || loading}
        onclick={() => (offset += 50)}>Next</button
      >
    </div>
  </section>{/if}
{#if selected}<section class="panel app-inspection">
    <div class="section-heading">
      <div>
        <h2 bind:this={inspectionHeading} tabindex="-1">{selected.name}</h2>
        <p class="path selectable">{selected.path}</p>
      </div>
      <button onclick={leaveInspection}>Back to applications</button>
    </div>
    <p class="muted small">
      {selected.bundleId} · {selected.version || 'Version unavailable'} · Observed
      {date(selected.observedAt)}
    </p>
    <p>{selected.qualification}</p>
    {#if selected.protection.length}<div class="callout">
        <div>
          <strong>Removal requires attention</strong
          >{#each selected.protection as reason}<p>{reason}</p>{/each}
        </div>
      </div>{/if}
    <button onclick={() => protect(selected!)}
      >{prefs.protectedPaths.includes(selected.path)
        ? 'Remove user protection'
        : 'Protect this application'}</button
    >
    {#if $controller.settings?.administratorEnabled && !administratorInspection}<button
        disabled={$controller.workRunning || $controller.scanning}
        onclick={(e) => inspect(selected!.path, e.currentTarget, true)}
        >Inspect with administrator</button
      >{/if}
    {#if administratorInspection}<p class="muted small">
        Administrator inspection. Selection and removal approval remain
        separate.
      </p>{/if}
    <h3>Selected bundle and related data</h3>
    <label class="check-row"
      ><input
        type="checkbox"
        disabled={selected.protection.length > 0 ||
          prefs.protectedPaths.includes(selected.path)}
        checked={paths.includes(selected.path)}
        onchange={(e) => {
          const p = selected!.path;
          paths = e.currentTarget.checked
            ? [...paths, p]
            : paths.filter((s) => s !== p);
        }}
      />Application bundle · size measured during review</label
    >
    {#each selected.related as item}<div class="related-file">
        <label class="check-row"
          ><input
            type="checkbox"
            disabled={!item.eligible || selected.protection.length > 0}
            checked={paths.includes(item.path)}
            onchange={(e) => {
              paths = e.currentTarget.checked
                ? [...paths, item.path]
                : paths.filter((s) => s !== item.path);
            }}
          /><span class="path selectable">{item.path}</span></label
        >
        <p class="small">{item.confidence} · {item.association}</p>
        <p class="muted small">{item.consequences}</p>
        <button
          class="text-button"
          onclick={() =>
            controller.run(() => controller.service.reveal(item.path))}
          >Reveal in Finder<ExternalLink size={13} /></button
        >
      </div>{:else}<p class="muted">
        No readable exact bundle-ID associations found. This does not establish
        absence of other application data.
      </p>{/each}
    <div class="dialog-actions">
      <button
        class="primary"
        disabled={!paths.length ||
          $controller.workRunning ||
          $controller.scanning}
        onclick={() => {
          removalCompleted = false;
          request = { paths: [...paths], application: selected!.path };
        }}>Review removal ({paths.length})</button
      >
    </div>
  </section>{/if}
{#if request}<RemovalReview
    {controller}
    {request}
    administrator={administratorInspection}
    onclose={() => {
      request = null;
      if (removalCompleted) void leaveInspection();
    }}
    oncomplete={() => {
      paths = [];
      removalCompleted = true;
      void refreshInventory();
    }}
  />{/if}

<style>
  .app-inspection {
    margin-top: 20px;
  }
  .related-file {
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
  }
  td:first-child {
    min-width: 210px;
  }
  td:nth-child(3) {
    max-width: 280px;
  }
  td .path {
    max-width: 340px;
    overflow-wrap: anywhere;
  }
</style>
