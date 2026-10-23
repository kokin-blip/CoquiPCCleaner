<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Activity,
    MessageCircle,
    CalendarClock,
    BrushCleaning,
    LayoutDashboard,
    Search,
    ScanLine,
    Cpu,
    HardDrive,
    Package,
    ShieldCheck,
    History as HistoryIcon,
    Settings as SettingsIcon,
    ChevronDown,
    Bell,
    ArrowRight,
    Info,
    X,
    LoaderCircle,
    Check,
  } from '@lucide/svelte';
  import { app, type createAppStore, type Page } from './state';
  import { date } from './format';
  import Onboarding from './features/Onboarding.svelte';
  import Dashboard from './features/Dashboard.svelte';
  import Analysis from './features/Analysis.svelte';
  import Storage from './features/Storage.svelte';
  import Applications from './features/Applications.svelte';
  import type { DiskScanPlan } from './bindings/DiskScanPlan';
  import Hardware from './features/Hardware.svelte';
  import Firmware from './features/Firmware.svelte';
  import Performance from './features/Performance.svelte';
  import History from './features/History.svelte';
  import Optimization from './features/Optimization.svelte';
  import AskCoqui from './features/AskCoqui.svelte';
  import Maintenance from './features/Maintenance.svelte';
  import Settings from './features/Settings.svelte';
  import Dialog from './components/Dialog.svelte';
  import Status from './components/Status.svelte';
  let { controller = app }: { controller?: ReturnType<typeof createAppStore> } =
    $props();
  let query = $state('');
  let deepConfirm = $state(false);
  let volumePlan = $state<DiskScanPlan | null>(null),
    volumeError = $state(''),
    volumeLoading = $state(false),
    volumeRoots = $state<string[]>([]);
  let notifications = $state(false);
  const pages: { name: Page; icon: typeof LayoutDashboard; group: string }[] = [
    { name: 'Ask Coqui', icon: MessageCircle, group: 'Main' },
    { name: 'Dashboard', icon: LayoutDashboard, group: 'Main' },
    { name: 'System Analysis', icon: ScanLine, group: 'Analysis' },
    { name: 'Hardware', icon: Cpu, group: 'Analysis' },
    { name: 'Storage', icon: HardDrive, group: 'Analysis' },
    { name: 'Applications', icon: Package, group: 'Analysis' },
    { name: 'Optimization', icon: BrushCleaning, group: 'Analysis' },
    { name: 'Performance', icon: Activity, group: 'Analysis' },
    { name: 'Firmware & Security', icon: ShieldCheck, group: 'Analysis' },
    { name: 'Maintenance', icon: CalendarClock, group: 'Application' },
    { name: 'History', icon: HistoryIcon, group: 'Application' },
    { name: 'Settings', icon: SettingsIcon, group: 'Application' },
  ];
  let results = $derived(
    query.trim()
      ? pages.filter((p) => p.name.toLowerCase().includes(query.toLowerCase()))
      : [],
  );
  let findingResults = $derived(
    query.trim()
      ? ($controller.session?.findings ?? []).filter((f) =>
          f.title.toLowerCase().includes(query.toLowerCase()),
        )
      : [],
  );
  let appResults = $derived(
    query.trim()
      ? ($controller.session?.snapshot?.applications ?? [])
          .filter((a) => a.name.toLowerCase().includes(query.toLowerCase()))
          .slice(0, 8)
      : [],
  );
  let machine = $derived($controller.session?.snapshot);
  onMount(() => {
    void controller.initialize();
    return () => {
      void controller.dispose();
    };
  });
  $effect(() => {
    const appearance = $controller.settings?.appearance ?? 'system';
    document.documentElement.dataset.theme = appearance;
  });
  function navigate(page: Page) {
    controller.navigate(page);
    query = '';
    notifications = false;
    document.querySelector('main.content')?.scrollTo(0, 0);
  }
  function scan() {
    if ($controller.settings?.scanMode === 'deep') {
      deepConfirm = true;
      volumePlan = null;
      volumeError = '';
      volumeLoading = true;
      controller.service
        .diskScanPlan()
        .then((p) => {
          volumePlan = p;
          volumeRoots = p.volumes.filter((v) => v.internal).map((v) => v.mount);
        })
        .catch((e) => (volumeError = String(e)))
        .finally(() => (volumeLoading = false));
    } else void controller.scan();
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      document.querySelector<HTMLInputElement>('#global-search')?.focus();
    }
    if (e.key === 'Escape') {
      query = '';
      notifications = false;
    }
  }}
/>
{#if $controller.loading}<div class="app-loading">
    <img class="brand-icon" src="/brand/coqui-file-emblem-v1.png" alt="" />
    <h1>Opening Coqui</h1>
    <p>Loading local preferences and diagnostic history…</p>
    <LoaderCircle class="spin" size={20} />
  </div>
{:else if !$controller.settings}<div class="app-loading">
    <ShieldCheck size={36} />
    <h1>Local diagnostics are unavailable.</h1>
    <p role="alert">{$controller.error}</p>
    <button class="primary" onclick={() => controller.initialize()}
      >Try again</button
    >
  </div>
{:else if !$controller.settings.onboardingComplete}<Onboarding
    {controller}
  />{#if $controller.error}<div class="floating-error" role="alert">
      {$controller.error}<button onclick={() => controller.dismiss()}
        >Dismiss</button
      >
    </div>{/if}
{:else}
  <div class="app-shell">
    <aside class="sidebar">
      <div class="sidebar-titlebar" data-tauri-drag-region></div>
      <div class="sidebar-brand">
        <img class="brand-icon" src="/brand/coqui-file-emblem-v1.png" alt="" />
        <div><strong>coqui</strong><span>PC Cleaner</span></div>
      </div>
      <nav aria-label="Main navigation">
        {#each pages as page, i}{#if i === 0 || pages[i - 1].group !== page.group}<p
              class="nav-group"
            >
              {page.group}
            </p>{/if}<button
            class:active={$controller.page === page.name}
            aria-current={$controller.page === page.name ? 'page' : undefined}
            onclick={() => navigate(page.name)}
            ><page.icon size={17} /><span>{page.name}</span
            >{#if page.name === 'System Analysis' && $controller.session?.findings.length}<span
                class="nav-count">{$controller.session.findings.length}</span
              >{/if}</button
          >{/each}
      </nav>
      <div class="sidebar-bottom">
        <div class="sidebar-device">
          <Cpu size={19} />
          <div>
            <strong>{machine?.model ?? 'This Mac'}</strong><span
              >{machine?.architecture ?? 'Apple Silicon build'}</span
            >
          </div>
        </div>
      </div>
    </aside>
    <div class="workspace">
      <header class="toolbar" data-tauri-drag-region>
        <div class="toolbar-device">
          <span>{machine?.deviceName ?? 'This Mac'}</span><ChevronDown
            size={12}
          />
        </div>
        <div class="global-search">
          <Search size={14} /><input
            id="global-search"
            type="search"
            aria-label="Search pages, findings and applications"
            placeholder="Search Coqui"
            bind:value={query}
          /><kbd>⌘ K</kbd>{#if query}<div class="search-results">
              {#each results as p}<button onclick={() => navigate(p.name)}
                  ><p.icon size={15} />{p.name}<span class="muted small"
                    >Page</span
                  ></button
                >{/each}{#each findingResults as f}<button
                  onclick={() => navigate('System Analysis')}
                  ><Info size={15} />{f.title}<span class="muted small"
                    >Finding</span
                  ></button
                >{/each}{#each appResults as a}<button
                  onclick={() => {
                    query = '';
                    void controller.run(() =>
                      controller.service.reveal(a.path),
                    );
                  }}
                  ><Cpu size={15} />{a.name}<span class="muted small"
                    >Reveal app</span
                  ></button
                >{/each}{#if !results.length && !findingResults.length && !appResults.length}<p
                  class="muted small"
                >
                  No matching pages, findings, or recorded applications.
                </p>{/if}
            </div>{/if}
        </div>
        <button
          class="icon-button notification-button"
          aria-label="Show notifications"
          aria-expanded={notifications}
          onclick={() => (notifications = !notifications)}
          ><Bell
            size={17}
          />{#if $controller.session?.findings.some((f) => f.severity === 'attention') || $controller.firmwareCenter?.notices.length || $controller.firmwareNoticeError}<span
            ></span>{/if}</button
        >
        <div class="toolbar-divider"></div>
        <button
          class="toolbar-scan"
          disabled={$controller.scanning ||
            $controller.cleanupRunning ||
            $controller.cleanupPreparing ||
            $controller.developerRunning}
          onclick={scan}
          ><ScanLine size={15} />{$controller.scanning
            ? 'Analyzing…'
            : 'Analyze'}</button
        >{#if notifications}<div class="notification-popover">
            <h3>Notifications</h3>
            {#if $controller.session}<Status
                value={$controller.session.status}
              />
              <p>{$controller.session.stage}</p>
              <p class="muted small">{date($controller.session.startedAt)}</p>
              <button
                class="text-button"
                onclick={() => navigate('System Analysis')}
                >Inspect findings<ArrowRight size={14} /></button
              >{:else}<p class="muted">No scans recorded yet.</p>{/if}
            {#if $controller.firmwareNoticeError}<p
                class="inline-error"
                role="alert"
              >
                {$controller.firmwareNoticeError}<button
                  onclick={() => controller.refreshFirmwareCenter()}
                  >Retry notices</button
                >
              </p>{/if}
            {#each $controller.firmwareCenter?.notices ?? [] as notice}<article
                class="firmware-notice"
              >
                <h4>{notice.title}</h4>
                <p class="muted small">{notice.message}</p>
                <p class="muted small">{date(notice.observedAt)}</p>
                <div class="button-row">
                  <button
                    onclick={() => {
                      controller.openFirmwareReport(notice.reportId);
                      notifications = false;
                      query = '';
                    }}
                    >{notice.reportId
                      ? 'Open source audit'
                      : 'Choose an audit'}</button
                  ><button
                    disabled={$controller.workRunning}
                    onclick={() =>
                      controller.run(async () => {
                        controller.setFirmwareCenter(
                          await controller.work(
                            'Dismiss local firmware notice',
                            () =>
                              controller.service.dismissFirmwareNotice(
                                notice.id,
                              ),
                          ),
                        );
                      })}>Dismiss notice</button
                  >
                </div>
              </article>{/each}
          </div>{/if}
      </header>
      {#if $controller.scanning}<div class="scan-banner" role="status">
          <LoaderCircle class="spin" size={17} />
          <div>
            <strong
              >{$controller.session?.stage ?? 'Preparing analysis…'}</strong
            ><span
              >{$controller.session?.fileCount.toLocaleString() ?? 0} files inspected
              · {$controller.cancelling
                ? 'Cancellation requested; finishing the current collector.'
                : 'Counting accessible metadata'}</span
            >
          </div>
          <button
            disabled={$controller.cancelling}
            onclick={() => controller.cancel()}
            >{$controller.cancelling ? 'Cancelling…' : 'Cancel scan'}</button
          >
        </div>{/if}
      {#if $controller.error || $controller.notice}<div
          class:error={$controller.error}
          class="notice-banner"
          role={$controller.error ? 'alert' : 'status'}
        >
          <Info size={16} /><span
            >{$controller.error ?? $controller.notice}</span
          ><button
            class="icon-button"
            aria-label="Dismiss notification"
            onclick={() => controller.dismiss()}><X size={16} /></button
          >
        </div>{/if}
      {#if $controller.developerRunning}<div class="scan-banner" role="status">
          <LoaderCircle class="spin" size={17} /><span
            >Checking developer artifacts and Git/worktree relationships…</span
          ><button onclick={() => controller.navigate('Optimization')}
            >View optimization</button
          ><button
            onclick={() =>
              controller.run(() => controller.service.cancelDeveloper())}
            >Cancel validation</button
          >
        </div>{/if}
      {#if $controller.missingApplications > 0}<div class="callout">
          <div>
            <strong
              >{$controller.missingApplications} previously observed applications
              are missing at their recorded paths</strong
            >
            <p>
              They may have moved or become disconnected. Review retained
              associations before deciding what to do.
            </p>
            <button onclick={() => controller.navigate('Applications')}
              >Review applications</button
            ><button onclick={() => controller.dismissApplicationNotice()}
              >Dismiss</button
            >
          </div>
        </div>{/if}
      {#if $controller.cleanupRunning || ($controller.cleanupPreparing && $controller.page !== 'Optimization')}<div
          class="scan-banner"
          role="status"
        >
          <BrushCleaning size={18} /><span
            >{$controller.cleanupPreparing
              ? 'Preparing a project-folder review…'
              : 'Moving approved items to Trash…'}</span
          ><button onclick={() => controller.navigate('Optimization')}
            >View progress</button
          ><button
            onclick={() =>
              controller.run(() => controller.service.cancelCleanup())}
            >{$controller.cleanupPreparing
              ? 'Cancel review preparation'
              : 'Stop remaining moves'}</button
          >
        </div>{/if}
      {#if $controller.workRunning}<div class="work-banner" role="status">
          <span
            ><LoaderCircle class="spin" size={15} />
            {$controller.workLabel}…</span
          ><button
            onclick={() =>
              controller.run(() => controller.service.cancelWork())}
            >Cancel operation</button
          >
        </div>{/if}
      <main class="content" id="main-content" tabindex="-1">
        {#if $controller.page === 'Dashboard'}<Dashboard
            {controller}
            onScan={scan}
          />{:else if $controller.page === 'System Analysis'}<Analysis
            {controller}
            onScan={scan}
          />{:else if $controller.page === 'Storage'}<Storage
            {controller}
            onScan={scan}
          />{:else if $controller.page === 'Applications'}<Applications
            {controller}
          />{:else if $controller.page === 'Optimization'}<Optimization
            {controller}
            onScan={scan}
          />{:else if $controller.page === 'Hardware'}<Hardware
            {controller}
          />{:else if $controller.page === 'Performance'}<Performance
            {controller}
          />{:else if $controller.page === 'Firmware & Security'}<Firmware
            {controller}
          />{:else if $controller.page === 'Ask Coqui'}<AskCoqui
            {controller}
          />{:else if $controller.page === 'Maintenance'}<Maintenance
            {controller}
          />{:else if $controller.page === 'History'}<History
            {controller}
          />{:else}<Settings {controller} />{/if}
      </main>
    </div>
  </div>
{/if}
{#if deepConfirm}<Dialog
    title="Review whole internal disk scope"
    onclose={() => (deepConfirm = false)}
  >
    <p>
      Scan readable metadata on the selected mounted internal volumes. Protected
      locations can remain unavailable. External and network volumes require
      separate selection.
    </p>
    {#if volumeLoading}<p role="status">Enumerating mounted volumes…</p>{/if}
    {#if volumeError}<p class="inline-error" role="alert">{volumeError}</p>
      <button onclick={scan}>Retry volume inventory</button>{/if}
    {#if volumePlan}<div class="table-scroll">
        <table>
          <thead
            ><tr
              ><th>Scan</th><th>Mounted volume</th><th>Filesystem</th><th
                >Access</th
              ></tr
            ></thead
          ><tbody
            >{#each volumePlan.volumes as v}<tr
                ><td
                  ><input
                    type="checkbox"
                    aria-label={`Scan ${v.mount}`}
                    disabled={!v.internal}
                    checked={volumeRoots.includes(v.mount)}
                    onchange={(e) =>
                      (volumeRoots = e.currentTarget.checked
                        ? [...volumeRoots, v.mount]
                        : volumeRoots.filter((p) => p !== v.mount))}
                  /></td
                ><td class="path selectable"
                  >{v.mount}
                  <p class="muted small">
                    {v.internal ? 'Internal' : 'Separate selection required'}
                  </p></td
                ><td>{v.filesystem}</td><td
                  >{v.readOnly
                    ? 'Read-only volume'
                    : 'Subject to permissions'}</td
                ></tr
              >{/each}</tbody
          >
        </table>
      </div>
      <p class="muted">{volumePlan.qualification}</p>{/if}
    <div class="dialog-actions">
      <button onclick={() => (deepConfirm = false)}>Cancel</button><button
        class="primary"
        disabled={!volumePlan || !volumeRoots.length || volumeLoading}
        onclick={() => {
          const p = volumePlan!;
          deepConfirm = false;
          void controller.scan(p.id, volumeRoots);
        }}>Scan selected volumes</button
      >
    </div>
  </Dialog>{/if}
