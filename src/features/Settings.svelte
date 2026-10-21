<script lang="ts">
  import {
    FolderPlus,
    X,
    ArrowRight,
    ShieldCheck,
    Trash2,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { workloads } from '../format';
  import Dialog from '../components/Dialog.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let settings = $derived($controller.settings!);
  let confirmPurge = $state(false);
  let custom = $state('');
  async function patch(values: Partial<typeof settings>) {
    await controller.run(() => controller.save({ ...settings, ...values }));
  }
  async function addFolders(excluded: boolean) {
    const paths = await controller.service.folders();
    if (paths.length)
      await patch(
        excluded
          ? {
              excludedPaths: [
                ...new Set([...settings.excludedPaths, ...paths]),
              ],
            }
          : {
              selectedRoots: [
                ...new Set([...settings.selectedRoots, ...paths]),
              ],
              scanMode: 'selected',
            },
      );
  }
</script>

<div class="page-heading">
  <div>
    <h1>Settings</h1>
    <p class="muted">
      Your workload, your scan boundaries, and your local data.
    </p>
  </div>
  <span class="subtle-tag">Local-first</span>
</div>
<section class="panel settings-panel">
  <h2>General & appearance</h2>
  <label class="settings-row"
    ><span
      ><strong>Appearance</strong><span class="muted small"
        >Follow your Mac or choose a fixed appearance.</span
      ></span
    ><select
      value={settings.appearance}
      onchange={(e) => patch({ appearance: e.currentTarget.value })}
      ><option value="system">System</option><option value="light">Light</option
      ><option value="dark">Dark</option></select
    ></label
  ><label class="switch-row"
    ><div>
      <strong>Live monitoring while Coqui is open</strong>
      <p class="muted small">
        CPU, memory, swap, and readable process counters every two seconds.
      </p>
    </div>
    <input
      type="checkbox"
      role="switch"
      checked={settings.monitoringEnabled}
      onchange={(e) => patch({ monitoringEnabled: e.currentTarget.checked })}
    /></label
  >
</section>
<section class="panel settings-panel">
  <h2>Workload profile</h2>
  <p class="muted">
    Context for future recommendations, not a performance promise.
  </p>
  <div class="workload-grid compact">
    {#each workloads as workload}<label
        class:selected={settings.workload.workloads.includes(workload)}
        class="choice"
        ><input
          type="checkbox"
          checked={settings.workload.workloads.includes(workload)}
          onchange={(e) =>
            patch({
              workload: {
                ...settings.workload,
                workloads: e.currentTarget.checked
                  ? [...settings.workload.workloads, workload]
                  : settings.workload.workloads.filter((w) => w !== workload),
              },
            })}
        /><span>{workload}</span></label
      >{/each}
  </div>
  <div class="inline-wrap">
    <input
      aria-label="Add a custom workload"
      placeholder="Custom workload…"
      bind:value={custom}
    /><button
      disabled={!custom.trim()}
      onclick={() => {
        void patch({
          workload: {
            ...settings.workload,
            workloads: [
              ...new Set([...settings.workload.workloads, custom.trim()]),
            ],
          },
        });
        custom = '';
      }}>Add workload</button
    >
  </div>
  {#each settings.workload.workloads.filter((w) => !workloads.includes(w)) as name}<button
      class="subtle-tag"
      onclick={() =>
        patch({
          workload: {
            ...settings.workload,
            workloads: settings.workload.workloads.filter((w) => w !== name),
          },
        })}
      aria-label={`Remove workload ${name}`}>{name}<X size={12} /></button
    >{/each}<label class="settings-row"
    ><span
      ><strong>Optimization preference</strong><span class="muted small"
        >Saved only. No power settings are applied.</span
      ></span
    ><select
      value={settings.workload.preference}
      onchange={(e) =>
        patch({
          workload: { ...settings.workload, preference: e.currentTarget.value },
        })}
      ><option>Balanced</option><option>Performance</option><option
        >Efficiency</option
      ></select
    ></label
  >
</section>
<section class="panel settings-panel">
  <h2>Storage scanning</h2>
  <label class="settings-row"
    ><span
      ><strong>Default scope</strong><span class="muted small"
        >Deep scans are confirmed before each run.</span
      ></span
    ><select
      value={settings.scanMode}
      onchange={(e) => patch({ scanMode: e.currentTarget.value })}
      ><option value="standard">Standard · home folder</option><option
        value="deep">Deep · accessible internal volumes</option
      ><option value="selected">Selected folders only</option></select
    ></label
  >
  <p class="muted small">
    External and network locations are included only when explicitly selected.
    Links, mount boundaries, and cloud-only placeholders are skipped.
  </p>
  <div class="section-heading">
    <h3>Selected folders</h3>
    <button onclick={() => controller.run(() => addFolders(false))}
      ><FolderPlus size={15} />Choose folders</button
    >
  </div>
  {#each settings.selectedRoots as path}<div class="path-row">
      <span class="path">{path}</span><button
        class="icon-button"
        aria-label={`Remove scan folder ${path}`}
        onclick={() =>
          patch({
            selectedRoots: settings.selectedRoots.filter((p) => p !== path),
          })}><X size={15} /></button
      >
    </div>{:else}<p class="muted small">
      No additional folders selected.
    </p>{/each}
  <div class="section-heading">
    <h3>Excluded directories</h3>
    <button onclick={() => controller.run(() => addFolders(true))}
      ><FolderPlus size={15} />Add exclusion</button
    >
  </div>
  {#each settings.excludedPaths as path}<div class="path-row">
      <span class="path">{path}</span><button
        class="icon-button"
        aria-label={`Remove exclusion ${path}`}
        onclick={() =>
          patch({
            excludedPaths: settings.excludedPaths.filter((p) => p !== path),
          })}><X size={15} /></button
      >
    </div>{:else}<p class="muted small">
      No user-defined exclusions. Coqui’s own diagnostic database is always
      excluded.
    </p>{/each}
</section>
<section class="panel settings-panel">
  <div class="section-heading">
    <h2>Privacy & permissions</h2>
    <ShieldCheck size={20} />
  </div>
  <p>
    Scanning respects macOS protections. Coqui does not request administrator
    privileges or bypass restricted folders. Grant Full Disk Access only if you
    want protected locations included.
  </p>
  <button
    onclick={() =>
      controller.run(() => controller.service.settings('permissions'))}
    >Review Full Disk Access<ArrowRight size={14} /></button
  ><label class="switch-row"
    ><div>
      <strong>Show advanced diagnostic metadata</strong>
      <p class="muted small">
        Expose structured scan reports for technical inspection. Grants no
        additional permissions.
      </p>
    </div>
    <input
      type="checkbox"
      role="switch"
      checked={settings.advancedDiagnostics}
      onchange={(e) => patch({ advancedDiagnostics: e.currentTarget.checked })}
    /></label
  >
</section>
<section class="panel settings-panel">
  <h2>Data retention</h2>
  <label class="settings-row"
    ><span
      ><strong>Keep diagnostic history</strong><span class="muted small"
        >Latest completed file index per scope; older summaries expire
        automatically.</span
      ></span
    ><select
      value={settings.retentionDays}
      onchange={(e) => patch({ retentionDays: Number(e.currentTarget.value) })}
      ><option value={7}>7 days</option><option value={30}>30 days</option
      ><option value={90}>90 days</option><option value={365}>1 year</option
      ></select
    ></label
  >
  <p class="path muted small">
    ~/Library/Application Support/CoquiPCCleaner/coqui.sqlite3
  </p>
  <p class="muted small">
    File contents and routine command dumps are not retained. Reports contain
    file paths and device metadata; inspect before sharing.
  </p>
  <button
    class="danger-outline"
    disabled={$controller.scanning || $controller.cleanupRunning}
    onclick={() => (confirmPurge = true)}
    ><Trash2 size={15} />Remove diagnostic history</button
  >{#if $controller.scanning || $controller.cleanupRunning}<p
      class="muted small"
    >
      Finish or cancel the active operation before clearing diagnostic data.
    </p>{/if}
</section>
<section class="panel">
  <h2>About CoquiPCCleaner</h2>
  <p>Version 0.2.0 · Apple Silicon · macOS 14+</p>
  <p class="muted">
    Tauri · Svelte · Vite · Rust · SQLite. Analysis remains read-only.
    Optimization can move individually approved cache files to macOS Trash.
    Conversational troubleshooting, developer-artifact cleanup and maintenance
    remain planned work.
  </p>
</section>
{#if confirmPurge}<Dialog
    title="Remove local diagnostic history?"
    onclose={() => (confirmPurge = false)}
    ><p>
      This removes stored scans, findings, file indexes, and cleanup action
      records from Coqui’s local database. Your files, preferences, and system
      settings are preserved.
    </p>
    <div class="dialog-actions">
      <button onclick={() => (confirmPurge = false)}>Keep history</button
      ><button
        class="danger"
        onclick={() =>
          controller.run(async () => {
            await controller.purge();
            confirmPurge = false;
          })}>Remove diagnostic history</button
      >
    </div></Dialog
  >{/if}
