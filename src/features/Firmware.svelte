<script lang="ts">
  import { onMount } from 'svelte';
  import { tabKeys } from '../keyboard';
  import type { createAppStore } from '../state';
  import type { FirmwareReport } from '../bindings/FirmwareReport';
  import type { FirmwarePlan } from '../bindings/FirmwarePlan';
  import type { FirmwarePlanKind } from '../bindings/FirmwarePlanKind';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import FirmwareReportView from '../components/FirmwareReportView.svelte';
  import FirmwarePlanView from '../components/FirmwarePlanView.svelte';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let tab = $state('Overview'),
    inventory = $state(false),
    security = $state(false),
    updates = $state(false),
    networkAck = $state(false);
  let report = $state<FirmwareReport | null>(null),
    plan = $state<FirmwarePlan | null>(null),
    audits = $state<ActivityRecord[]>([]),
    plans = $state<ActivityRecord[]>([]);
  let loading = $state(false),
    error = $state('');
  let machine = $derived($controller.session?.snapshot);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  const planKinds: { kind: FirmwarePlanKind; title: string; detail: string }[] =
    [
      {
        kind: 'softwareUpdate',
        title: 'Review a software update',
        detail:
          'Backup, compatibility, power and restart requirements; installation stays in Apple’s tools.',
      },
      {
        kind: 'startupPolicyReview',
        title: 'Inspect startup security',
        detail:
          'Read the policy for the intended disk in Recovery. No security reduction is proposed.',
      },
      {
        kind: 'recoveryReview',
        title: 'Review recovery requirements',
        detail:
          'Understand model-specific Revive and Restore procedures. Restore erases the affected Mac.',
      },
    ];
  onMount(() => {
    void load();
  });
  async function load(clearError = true) {
    loading = true;
    if (clearError) error = '';
    try {
      audits = await controller.service.records('firmware', 0);
      plans = await controller.service.records('firmwarePlan', 0);
      if (!report && audits[0]) await inspectAudit(audits[0].id);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function inspectAudit(id: string) {
    const r = await controller.service.record('firmware', id);
    if (r.kind === 'firmware') report = r.data;
  }
  async function inspectPlan(id: string) {
    error = '';
    try {
      const r = await controller.service.record('firmwarePlan', id);
      if (r.kind === 'firmwarePlan') {
        plan = r.data;
        tab = 'Guidance';
      }
    } catch (e) {
      error = String(e);
    }
  }
  async function audit() {
    error = '';
    const request = {
      inventory,
      security,
      updates,
      networkAcknowledged: networkAck,
    };
    try {
      report = await controller.work('Firmware & security audit', () =>
        controller.service.auditFirmware(request, (r) => (report = r)),
      );
    } catch (e) {
      error = String(e);
    } finally {
      networkAck = false;
      updates = false;
      void load(false);
    }
  }
  async function createPlan(kind: FirmwarePlanKind) {
    if (!report) return;
    error = '';
    try {
      plan = await controller.work('Prepare separate Apple guidance', () =>
        controller.service.firmwarePlan(report!.id, kind),
      );
      tab = 'Guidance';
      await load();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="firmware-workspace">
  <div class="page-heading">
    <div>
      <h1>Firmware & Security</h1>
      <p class="muted">
        Inspect the platform, understand coverage, then choose an
        Apple-supported next step.
      </p>
    </div>
  </div>
  <div
    class="tabs"
    role="tablist"
    tabindex="-1"
    onkeydown={tabKeys}
    aria-label="Firmware views"
  >
    {#each ['Overview', 'Advanced audit', 'Guidance'] as name}<button
        role="tab"
        tabindex={tab === name ? 0 : -1}
        aria-selected={tab === name}
        class:active={tab === name}
        onclick={() => (tab = name)}>{name}</button
      >{/each}
  </div>
  {#if error}<p class="inline-error" role="alert">
      {error}<button onclick={() => load()}>Retry loading records</button>
    </p>{/if}
  {#if loading}<p role="status">Loading retained platform records…</p>{/if}
  {#if tab === 'Overview'}
    <section class="panel">
      <div class="section-heading">
        <h2>{report?.platform.model ?? machine?.model ?? 'This Mac'}</h2>
        <button class="primary" onclick={() => (tab = 'Advanced audit')}
          >Choose an audit</button
        >
      </div>
      <p>
        {report
          ? `${report.platform.chip} · macOS ${report.platform.osVersion}`
          : machine
            ? `${machine.architecture} · ${machine.osVersion}`
            : 'No platform inventory recorded yet.'}
      </p>
      <p class="muted">
        Apple Silicon uses Mac firmware and per-disk startup security. PC BIOS
        tuning, XMP/EXPO and firmware-password controls are outside this
        adapter.
      </p>
      <p class="muted small">
        Advanced audits run only when requested. Saved readings are observations
        at their timestamp, not a current security certification.
      </p>
    </section>
    {#if report}<FirmwareReportView {report} />{:else if machine}<section
        class="panel"
      >
        <h2>Earlier scan observations</h2>
        <p class="muted small">
          Snapshot {date(machine.observedAt)}. Choose an optional audit for
          fresh platform readings.
        </p>
        <Evidence
          items={machine.evidence.filter((e) =>
            [
              'System firmware',
              'OS loader',
              'System Integrity Protection',
              'FileVault',
            ].includes(e.label),
          )}
        />
      </section>{:else}<div class="empty-state compact">
        <h2>No firmware audit recorded.</h2>
        <p>Choose an optional audit. A storage scan is not required.</p>
      </div>{/if}
  {:else if tab === 'Advanced audit'}
    <section class="panel">
      <h2>Choose the readings for this run</h2>
      <p class="muted">
        Every run identifies model, chip and macOS build first. No optional
        capability is enabled by default.
      </p>
      <div class="firmware-options">
        <label class="check-row"
          ><input
            type="checkbox"
            bind:checked={inventory}
            disabled={busy}
          /><span
            ><strong>Firmware inventory</strong><span class="muted small"
              >Reported system firmware and OS loader versions; no “latest” or
              integrity verdict.</span
            ></span
          ></label
        >
        <label class="check-row"
          ><input
            type="checkbox"
            bind:checked={security}
            disabled={busy}
          /><span
            ><strong>Configuration audit</strong><span class="muted small"
              >Readable SIP, authenticated-root and FileVault status. Startup
              security policy remains manual.</span
            ></span
          ></label
        >
        <label class="check-row"
          ><input
            type="checkbox"
            bind:checked={updates}
            disabled={busy}
            onchange={() => (networkAck = false)}
          /><span
            ><strong>macOS update availability</strong><span class="muted small"
              >Ask macOS for offered software updates. No download, installation
              or restart.</span
            ></span
          ></label
        >
      </div>
      {#if updates}<div class="firmware-network">
          <p>
            macOS may contact Apple and refresh its update catalog and system
            logs. A check can take up to two minutes. Stopping Coqui’s client
            may leave the OS service finishing its check.
          </p>
          <label class="check-row"
            ><input
              type="checkbox"
              bind:checked={networkAck}
              disabled={busy}
            />Allow this macOS network update check for this run.</label
          >
        </div>{/if}
      <div class="button-row">
        <button
          class="primary"
          disabled={busy ||
            !(inventory || security || updates) ||
            (updates && !networkAck)}
          onclick={audit}>Run selected audit</button
        >{#if $controller.workRunning}<button
            onclick={() =>
              controller.run(() => controller.service.cancelWork())}
            >Cancel audit</button
          >{/if}
      </div>
    </section>
    {#if report}<FirmwareReportView {report} />{/if}
  {:else}
    <section class="panel">
      <h2>Separate manual guidance</h2>
      <p class="muted">
        Use a finished audit from the last 24 hours. These plans explain Apple’s
        steps; Coqui has no firmware modification, installation, restart or
        recovery command.
      </p>
      {#each planKinds as choice}<div class="capability-row">
          <div>
            <strong>{choice.title}</strong>
            <p class="muted">{choice.detail}</p>
          </div>
          <button
            disabled={busy ||
              !report?.platform.eligible ||
              !['complete', 'partial'].includes(report?.status ?? '')}
            onclick={() => createPlan(choice.kind)}
            aria-label={`Prepare ${choice.title}`}>Prepare plan</button
          >
        </div>{/each}
      {#if !report?.platform.eligible}<p class="muted">
          Run an eligible Apple Silicon audit before preparing a plan.
        </p>{/if}
    </section>
    {#if plan}<section class="panel">
        <FirmwarePlanView
          {controller}
          {plan}
          onUpdate={(p) => {
            plan = p;
            void load();
          }}
          onReport={(r) => (report = r)}
        />
      </section>{/if}
    <section class="panel">
      <div class="section-heading">
        <h2>Retained guidance plans</h2>
        <button disabled={loading || busy} onclick={() => load()}
          >Refresh</button
        >
      </div>
      {#each plans as item}<button
          class="history-row"
          onclick={() => inspectPlan(item.id)}
          ><span
            >{item.title}<span class="muted small">
              · {date(item.createdAt)}</span
            ></span
          ><Status value={item.status} /></button
        >{:else}<p class="empty-inline muted">
          No guidance plans recorded.
        </p>{/each}
    </section>
  {/if}
  {#if audits.length}<section class="panel">
      <h2>Saved audits</h2>
      <p class="muted small">
        Latest 50 records. Older records and full guidance history are in
        History → Activity. Retention follows Settings.
      </p>
      {#each audits as item}<button
          class="history-row"
          disabled={busy}
          onclick={() =>
            controller.run(async () => {
              await inspectAudit(item.id);
              tab = 'Overview';
            })}
          ><span class="history-description">{date(item.createdAt)}</span
          ><Status value={item.status} /></button
        >{/each}
    </section>{/if}
</div>
