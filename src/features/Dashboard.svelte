<script lang="ts">
  import {
    ArrowRight,
    Activity,
    HardDrive,
    MemoryStick,
    Cpu,
    ShieldCheck,
    Search,
    Info,
    ChevronRight,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  import Chart from '../components/Chart.svelte';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let machine = $derived($controller.session?.snapshot);
  let live = $derived(
    $controller.monitoringStatus === 'live'
      ? $controller.samples.at(-1)
      : undefined,
  );
  let disk = $derived(
    machine?.disks.find((d) => d.mount === '/System/Volumes/Data') ??
      machine?.disks.find((d) => d.mount === '/'),
  );
  let findings = $derived($controller.session?.findings ?? []);
</script>

<div class="page-heading">
  <div>
    <h1>A clearer view of your Mac.</h1>
    <p class="muted">
      {machine
        ? `${machine.model} · ${machine.osVersion}`
        : 'Start with an analysis of your hardware, storage, and startup configuration.'}
    </p>
  </div>
  <button class="primary" onclick={onScan} disabled={$controller.scanning}
    ><Search size={16} />Analyze System</button
  >
</div>
<section class="device-overview">
  <div class="device-art" aria-hidden="true">
    <svg viewBox="0 0 180 135"
      ><rect x="20" y="10" width="140" height="92" rx="7" /><path
        d="M25 91H155 M76 102L72 121H108L104 102 M63 123H117"
      /><path
        class="screen-pulse"
        d="M45 59H65L74 39L88 77L103 49L112 59H136"
      /></svg
    >
  </div>
  <div class="device-description">
    <h2>{machine?.deviceName ?? 'Your next insight starts here.'}</h2>
    <p>
      {machine
        ? `${machine.cpu} · ${bytes(machine.memoryBytes)} memory`
        : 'An evidence-based system scanner, built around your workload.'}
    </p>
    <div class="inline-wrap">
      <span class="subtle-tag"
        >{$controller.settings?.workload.preference ?? 'Balanced'}</span
      >{#each ($controller.settings?.workload.workloads ?? []).slice(0, 2) as workload}<span
          class="subtle-tag">{workload}</span
        >{/each}
    </div>
  </div>
  <div class="device-scan">
    <ShieldCheck size={22} /><strong>Read-only analysis</strong><span
      class="muted small">Your files and settings stay yours.</span
    ><span class="muted small"
      >{machine
        ? `Last scan ${date($controller.session?.startedAt ?? null)}`
        : 'No scan recorded yet'}</span
    >
  </div>
</section>
<div class="summary-grid">
  <button
    class="summary-item"
    onclick={() => controller.navigate('Performance')}
    ><div>
      <Cpu size={17} /><span>CPU utilization</span><ChevronRight size={15} />
    </div>
    <strong>{live ? `${live.cpuPercent.toFixed(1)}%` : 'Not recorded'}</strong>
    <p>
      {live
        ? 'Live · sampled every 2 seconds'
        : $controller.monitoringStatus === 'failed'
          ? 'Monitoring stopped · open Performance to retry'
          : $controller.monitoringStatus === 'starting'
            ? 'Waiting for a measurement'
            : 'Enable live monitoring in Settings'}
    </p></button
  >
  <button
    class="summary-item"
    onclick={() => controller.navigate('Performance')}
    ><div>
      <MemoryStick size={17} /><span>Memory in use</span><ChevronRight
        size={15}
      />
    </div>
    <strong>{live ? bytes(live.usedMemoryBytes) : 'Not recorded'}</strong>
    <p>
      {live
        ? `of ${bytes(live.totalMemoryBytes)} · ${bytes(live.swapBytes)} swap`
        : 'Memory use alone is not a problem'}
    </p></button
  >
  <button class="summary-item" onclick={() => controller.navigate('Storage')}
    ><div>
      <HardDrive size={17} /><span>Available storage</span><ChevronRight
        size={15}
      />
    </div>
    <strong>{disk ? bytes(disk.availableBytes) : 'Not scanned'}</strong>
    <p>
      {disk
        ? `of ${bytes(disk.totalBytes)} · shared APFS capacity may apply`
        : 'Explore what is taking up space'}
    </p></button
  >
</div>
<div class="dashboard-columns">
  <section class="panel">
    <div class="section-heading">
      <h2>What deserves a closer look</h2>
      <button
        class="text-button"
        onclick={() => controller.navigate('System Analysis')}
        >All findings<ArrowRight size={14} /></button
      >
    </div>
    {#if findings.length}{#each findings.slice(0, 5) as finding}<button
          class="finding-row"
          onclick={() => controller.navigate('System Analysis')}
          ><span
            class:attention-icon={finding.severity === 'attention'}
            class="finding-icon"><Info size={18} /></span
          ><span
            ><strong>{finding.title}</strong><span class="muted small"
              >{finding.confidence}</span
            ></span
          ><ChevronRight size={15} /></button
        >{/each}{:else}<div class="empty-inline">
        <Search size={27} />
        <h3>Let the evidence lead.</h3>
        <p>
          Run an analysis to inspect storage, hardware, and startup
          configuration. Coqui will show what it can verify and where coverage
          is limited.
        </p>
        <button onclick={onScan} disabled={$controller.scanning}
          >Start an analysis</button
        >
      </div>{/if}
  </section>
  <section class="panel">
    <div class="section-heading">
      <h2>Live activity</h2>
      <Activity size={17} />
    </div>
    <div class="inline-between">
      <strong>CPU utilization</strong><span class="muted"
        >{live ? `${live.cpuPercent.toFixed(1)}%` : 'Not recorded'}</span
      >
    </div>
    <Chart samples={$controller.samples} />
    <p class="small muted">
      Recorded in this session only. Monitoring pauses when the app exits.
    </p>
    <button
      class="text-button"
      onclick={() => controller.navigate('Performance')}
      >Explore performance<ArrowRight size={14} /></button
    >
  </section>
</div>
<section class="panel activity-panel">
  <div class="section-heading">
    <h2>Recent activity</h2>
    <button class="text-button" onclick={() => controller.navigate('History')}
      >View history<ArrowRight size={14} /></button
    >
  </div>
  {#each $controller.history.slice(0, 3) as session}<button
      class="activity-row"
      onclick={() => controller.selectSession(session)}
      ><Search size={16} /><span
        ><strong>System analysis</strong><span class="muted small"
          >{date(session.startedAt)} · {session.fileCount.toLocaleString()} files
          inspected</span
        ></span
      ><Status value={session.status} /><ChevronRight size={15} /></button
    >{:else}<p class="muted">
      Your completed, partial, and cancelled analyses will appear here.
    </p>{/each}
</section>
