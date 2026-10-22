<script lang="ts">
  import { tabKeys } from '../keyboard';
  import {
    Cpu,
    MemoryStick,
    Monitor,
    HardDrive,
    Search,
    Package,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let machine = $derived($controller.session?.snapshot);
  let tab = $state('Components');
  let filter = $state('');
</script>

<div class="page-heading">
  <div>
    <h1>Hardware</h1>
    <p class="muted">
      The configuration your operating system actually reports.
    </p>
  </div>
  {#if machine}<span class="muted small"
      >Snapshot {date(machine.observedAt)}</span
    >{/if}
</div>
{#if !machine}<div class="empty-state">
    <Cpu size={36} />
    <h2>Hardware inventory has not been recorded.</h2>
    <p>
      Run an analysis from the toolbar. Missing readings will remain
      unavailable.
    </p>
  </div>{:else}
  <div
    class="tabs"
    role="tablist"
    tabindex="-1"
    onkeydown={tabKeys}
    aria-label="Hardware views"
  >
    {#each ['Components', 'Disks & volumes', 'Applications'] as name}<button
        role="tab"
        tabindex={tab === name ? 0 : -1}
        aria-selected={tab === name}
        class:active={tab === name}
        onclick={() => (tab = name)}>{name}</button
      >{/each}
  </div>
  {#if tab === 'Components'}<div class="hardware-grid">
      {#each [{ title: 'Processor', icon: Cpu, value: machine.cpu, detail: `${machine.coreCount} logical cores · ${machine.architecture}` }, { title: 'Memory', icon: MemoryStick, value: bytes(machine.memoryBytes), detail: 'Installed physical memory' }, { title: 'Graphics', icon: Monitor, value: machine.gpu, detail: 'Identity from system report; utilization unavailable' }, { title: 'System platform', icon: HardDrive, value: machine.model, detail: machine.osVersion }] as item}<section
          class="panel hardware-item"
        >
          <item.icon size={24} />
          <h2>{item.title}</h2>
          <strong>{item.value}</strong>
          <p class="muted">{item.detail}</p>
        </section>{/each}
    </div>
    <section class="panel">
      <h2>Additional evidence</h2>
      <Evidence items={machine.evidence} />
    </section>
  {:else if tab === 'Disks & volumes'}<section class="panel">
      <h2>Reported filesystems</h2>
      <p class="muted">
        APFS volumes may share the same container capacity. Do not sum these
        capacities to estimate physical storage.
      </p>
      <div class="table-scroll">
        <table>
          <thead
            ><tr
              ><th>Name / mount point</th><th>Filesystem</th><th class="numeric"
                >Capacity</th
              ><th class="numeric">Available</th></tr
            ></thead
          ><tbody
            >{#each machine.disks as disk}<tr
                ><td
                  ><strong>{disk.name}</strong>
                  <p class="path small muted">{disk.mount}</p></td
                ><td>{disk.filesystem}{disk.removable ? ' · Removable' : ''}</td
                ><td class="numeric">{bytes(disk.totalBytes)}</td><td
                  class="numeric">{bytes(disk.availableBytes)}</td
                ></tr
              >{/each}</tbody
          >
        </table>
      </div>
    </section>
  {:else}<section class="panel">
      <div class="section-heading">
        <h2>Application inventory</h2>
        <input
          type="search"
          aria-label="Filter applications"
          placeholder="Filter applications…"
          bind:value={filter}
        />
      </div>
      <p class="muted small">
        Application bundles found in /Applications and ~/Applications. This is
        not a complete package-manager inventory.
      </p>
      {#each machine.applications.filter((a) => a.name
          .toLowerCase()
          .includes(filter.toLowerCase())) as item}<div class="application-row">
          <Package size={20} />
          <div>
            <strong>{item.name}</strong>
            <p class="path small muted">{item.path}</p>
          </div>
          <button
            class="text-button"
            onclick={() =>
              controller.run(() => controller.service.reveal(item.path))}
            >Reveal in Finder</button
          >
        </div>{:else}<p class="empty-inline muted">
          No matching applications in this snapshot.
        </p>{/each}
    </section>{/if}{/if}
