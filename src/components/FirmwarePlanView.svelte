<script lang="ts">
  import type { FirmwarePlan } from '../bindings/FirmwarePlan';
  import type { FirmwareReport } from '../bindings/FirmwareReport';
  import type { createAppStore } from '../state';
  import type { FirmwareComparison } from '../bindings/FirmwareComparison';
  import FirmwareComparisonView from './FirmwareComparisonView.svelte';
  import Evidence from './Evidence.svelte';
  import Status from './Status.svelte';
  import { date } from '../format';
  let {
    controller,
    plan,
    onUpdate = () => {},
    onReport = () => {},
  }: {
    controller: ReturnType<typeof createAppStore>;
    plan: FirmwarePlan;
    onUpdate?: (p: FirmwarePlan) => void;
    onReport?: (r: FirmwareReport) => void;
  } = $props();
  let comparison = $state<FirmwareComparison | null>(null);
  let acknowledged = $state(false),
    performed = $state(false),
    error = $state('');
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  $effect(() => {
    plan.id;
    comparison = null;
    acknowledged = false;
    performed = false;
    error = '';
  });
  async function inspectComparison() {
    error = '';
    try {
      const record = await controller.service.record(
        'firmwareComparison',
        plan.comparisonId!,
      );
      if (record.kind === 'firmwareComparison') comparison = record.data;
    } catch (e) {
      error = String(e);
    }
  }
  async function accept(accepted: boolean) {
    error = '';
    try {
      onUpdate(
        await controller.work('Record guidance acknowledgement', () =>
          controller.service.acknowledgeFirmwarePlan(plan.id, accepted),
        ),
      );
    } catch (e) {
      error = String(e);
    }
  }
  async function observe(didPerform: boolean) {
    error = '';
    try {
      onUpdate(
        await controller.work('Observe Apple manual guidance outcome', () =>
          controller.service.observeFirmwarePlan(plan.id, didPerform, onReport),
        ),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="firmware-plan">
  <div class="section-heading">
    <h2>{plan.title}</h2>
    <Status value={plan.status} />
  </div>
  <p class="muted small">
    Created {date(plan.createdAt)} · {plan.before.model} · macOS {plan.before
      .osVersion} ({plan.before.osBuild})
  </p>
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  <ul class="firmware-cautions">
    {#each plan.cautions as caution}<li>{caution}</li>{/each}
  </ul>
  <ol>
    {#each plan.steps as step}<li>{step}</li>{/each}
  </ol>
  <p class="muted small">
    Bundled guidance reviewed {plan.sourceReviewedAt}. Read the current Apple
    source before any manual step.
  </p>
  <a
    class="reference-link"
    href={plan.sourceUrl}
    onclick={(e) => {
      e.preventDefault();
      void controller.run(() => controller.service.reference(plan.sourceUrl));
    }}>Read current Apple guidance</a
  >
  {#if plan.status === 'awaitingAcknowledgement'}
    <label class="check-row"
      ><input type="checkbox" bind:checked={acknowledged} disabled={busy} />I
      understand the scope, interruption and recovery limits. This acknowledges
      guidance only.</label
    >
    <div class="button-row">
      <button disabled={busy} onclick={() => accept(false)}
        >Decline guidance plan</button
      ><button
        class="primary"
        disabled={busy || !acknowledged}
        onclick={() => accept(true)}>Acknowledge manual guidance</button
      >
    </div>
  {:else if plan.status === 'awaitingManualAction'}
    {#if plan.kind === 'softwareUpdate'}<button
        disabled={busy}
        onclick={() =>
          controller.run(() => controller.service.settings('updates'))}
        >Open System Settings</button
      >
      <p class="muted small">
        In General → Software Update, macOS can check Apple's network again.
        Choose any installation in Apple's interface.
      </p>{/if}
    <label class="check-row"
      ><input type="checkbox" bind:checked={performed} disabled={busy} />I
      completed my manual step and returned to this Mac.</label
    >
    <div class="button-row">
      <button disabled={busy} onclick={() => observe(false)}
        >Record no action taken</button
      ><button disabled={busy || !performed} onclick={() => observe(true)}
        >Record new local observations</button
      >
    </div>
    <p class="muted small">
      Collects firmware, SIP, authenticated-root and FileVault status. No
      update-network check. Does not certify installation or reboot success.
    </p>
  {/if}
  <p role="status">{plan.outcome}</p>
  <p class="muted">{plan.recovery}</p>
  {#if plan.afterReportId}<p class="path small">
      New audit: {plan.afterReportId}. Open it in Firmware & Security or History
      → Activity.
    </p>{/if}
  {#if plan.comparisonId}<button onclick={inspectComparison}
      >Inspect before and after evidence</button
    >{/if}
  {#if comparison}<FirmwareComparisonView {comparison} />{/if}
  <details>
    <summary>Previous local observations</summary><Evidence
      items={plan.previous}
    />
  </details>
</div>
