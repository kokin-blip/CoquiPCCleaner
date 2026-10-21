<script lang="ts">
  import { tabKeys } from '../keyboard';
  import {
    ShieldAlert,
    ArrowRight,
    Search,
    Check,
    Minus,
    Info,
    Download,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let session = $derived($controller.session);
  let tab = $state('Overview');
  function exportReport() {
    if (session && controller.service.exportReport)
      void controller.run(() => controller.service.exportReport!(session!.id));
  }
</script>

<div class="page-heading">
  <div>
    <h1>System Analysis</h1>
    <p class="muted">
      Observations, supporting evidence, and the limits of this scan.
    </p>
  </div>
  <button onclick={onScan} disabled={$controller.scanning}
    ><Search size={16} />Analyze again</button
  >
</div>
{#if !session}<div class="empty-state">
    <Search size={36} />
    <h2>No analysis yet.</h2>
    <p>
      Choose a scope and run a read-only scan to collect real system
      information.
    </p>
    <button class="primary" onclick={onScan}>Analyze System</button>
  </div>{:else}
  <div
    class="tabs"
    role="tablist"
    tabindex="-1"
    onkeydown={tabKeys}
    aria-label="Analysis views"
  >
    {#each ['Overview', 'Findings', 'Coverage', 'Report'] as name}<button
        role="tab"
        tabindex={tab === name ? 0 : -1}
        aria-selected={tab === name}
        class:active={tab === name}
        onclick={() => (tab = name)}
        >{name}{name === 'Findings'
          ? ` (${session.findings.length})`
          : ''}</button
      >{/each}
  </div>
  {#if tab === 'Overview'}
    <section class="panel">
      <div class="section-heading">
        <h2>{session.stage}</h2>
        <Status value={session.status} />
      </div>
      <div class="scan-facts">
        <div>
          <span>Files inspected</span><strong
            >{session.fileCount.toLocaleString()}</strong
          >
        </div>
        <div>
          <span>Allocated file space</span><strong
            >{bytes(session.allocatedBytes)}</strong
          >
        </div>
        <div>
          <span>Logical file size</span><strong
            >{bytes(session.logicalBytes)}</strong
          >
        </div>
        <div>
          <span>Skipped entries</span><strong
            >{session.skippedCount.toLocaleString()}</strong
          >
        </div>
      </div>
      <p class="path small">{session.scope}</p>
      <p class="muted small">
        Started {date(session.startedAt)} · {session.finishedAt
          ? `Finished ${date(session.finishedAt)}`
          : 'Analysis in progress'}
      </p>
    </section>
    <div class="callout">
      <Info size={20} />
      <div>
        <strong>Completion and coverage are different.</strong>
        <p>
          A completed scan covers accessible locations. Startup registrations,
          sensors, and security properties may still require manual review.
          Skipped links, excluded folders, cloud placeholders and mount
          boundaries are not counted as inspected files.
        </p>
        <button class="text-button" onclick={() => (tab = 'Coverage')}
          >Inspect coverage<ArrowRight size={14} /></button
        >
      </div>
    </div>
    <section class="panel">
      <h2>Scan stages</h2>
      {#each ['Hardware inventory', 'Accessible storage metadata', 'Startup configurations', 'Available firmware and security properties'] as stage}<div
          class="stage-row"
        >
          {#if session.snapshot}<Check size={16} />{:else}<Minus
              size={16}
            />{/if}<span>{stage}</span><span class="muted small"
            >{stage === 'Accessible storage metadata'
              ? `${session.fileCount.toLocaleString()} files`
              : stage === 'Hardware inventory' && session.snapshot
                ? 'Inventory recorded'
                : 'Availability disclosed in details'}</span
          >
        </div>{/each}
    </section>
  {:else if tab === 'Findings'}
    {#each session.findings as finding}<section class="panel finding-detail">
        <div class="section-heading">
          <h2>{finding.title}</h2>
          <Status value={finding.severity} />
        </div>
        <p>{finding.explanation}</p>
        <div class="inline-wrap">
          <span class="subtle-tag">{finding.confidence}</span><span
            class="subtle-tag">{finding.kind}</span
          >
        </div>
        <Evidence items={finding.evidence} />
      </section>{:else}<div class="empty-state">
        <Check size={32} />
        <h2>No findings recorded.</h2>
        <p>
          This does not imply that every subsystem was inspected. Check coverage
          for unavailable information.
        </p>
      </div>{/each}
  {:else if tab === 'Coverage'}
    <section class="panel">
      <h2>System capabilities</h2>
      {#each session.snapshot?.capabilities ?? [] as capability}<div
          class="capability-row"
        >
          <div>
            <strong>{capability.name}</strong>
            <p class="muted">{capability.explanation}</p>
          </div>
          <Status value={capability.state} />
        </div>{:else}<p class="muted">
          Hardware capability information was not collected.
        </p>{/each}
    </section>
    <section class="panel">
      <div class="section-heading">
        <h2>Permission guidance</h2>
        <button
          onclick={() =>
            controller.run(() => controller.service.settings('permissions'))}
          >Open System Settings<ArrowRight size={14} /></button
        >
      </div>
      {#each session.permissions as permission}<div class="permission-row">
          <ShieldAlert size={19} />
          <div>
            <strong class="path">{permission.path}</strong>
            <p>{permission.reason}</p>
            <p class="muted">{permission.guidance}</p>
          </div>
        </div>{:else}<p class="muted">
          No permission-denied entries were reported in this scope. Other
          excluded or unsupported areas may remain uninspected.
        </p>{/each}{#if session.permissions.length >= 100}<p
          class="muted small"
        >
          The permission detail list is capped at 100 entries. Check the skipped
          count for total omitted entries.
        </p>{/if}
    </section>
    {#if session.errors.length}<section class="panel">
        <h2>Collection issues</h2>
        {#each session.errors as error}<p class="muted">{error}</p>{/each}
      </section>{/if}
  {:else}
    <section class="panel">
      <div class="section-heading">
        <h2>Diagnostic report</h2>
        <button onclick={exportReport}
          ><Download size={15} />Export JSON report</button
        >
      </div>
      <p class="muted">
        Includes device metadata, paths, scope, evidence, and scan findings.
        Review before sharing.
      </p>
      <Evidence
        items={session.snapshot?.evidence ?? []}
      />{#if $controller.settings?.advancedDiagnostics}<details>
          <summary>Raw structured report</summary>
          <pre>{JSON.stringify(session, null, 2)}</pre>
        </details>{/if}
    </section>
  {/if}{/if}
