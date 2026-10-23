<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import { date, label } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let enabled = $state(false),
    days = $state(7),
    loading = $state(true),
    error = $state(''),
    message = $state('');
  let center = $derived($controller.firmwareCenter);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    void load();
  });
  async function load() {
    loading = true;
    error = '';
    await controller.refreshFirmwareCenter();
    const p = $controller.firmwareCenter?.preferences;
    if (p) {
      enabled = p.reminderEnabled;
      days = p.intervalDays;
    }
    loading = false;
  }
  async function save() {
    error = '';
    message = '';
    try {
      controller.setFirmwareCenter(
        await controller.work('Save firmware review reminder', () =>
          controller.service.saveFirmwareReminder(enabled, days),
        ),
      );
      message = enabled
        ? 'Review reminder saved. It never starts a network check.'
        : 'Review reminder disabled.';
    } catch (e) {
      error = String(e);
    }
  }
  async function dismiss(id: string) {
    error = '';
    try {
      controller.setFirmwareCenter(
        await controller.work('Dismiss local firmware notice', () =>
          controller.service.dismissFirmwareNotice(id),
        ),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel">
  <div class="section-heading">
    <h2>Update notices</h2>
    <button disabled={loading} onclick={load}>Refresh local notices</button>
  </div>
  <p class="muted">
    Notices use saved macOS responses. Refreshing this list reads Coqui’s local
    data; choose Advanced audit for a new explicitly consented network check.
  </p>
  {#if loading}<p role="status">
      Loading notices…
    </p>{/if}{#if error || $controller.firmwareNoticeError}<p
      class="inline-error"
      role="alert"
    >
      {error || $controller.firmwareNoticeError}<button onclick={load}
        >Retry</button
      >
    </p>{/if}
  {#each center?.notices ?? [] as notice}<article class="firmware-finding">
      <h3>{notice.title}</h3>
      <p>{notice.message}</p>
      <p class="muted small">
        {date(notice.observedAt)} · {notice.stale
          ? 'Stale or incomplete evidence'
          : 'Recorded evidence only'}
      </p>
      <div class="button-row">
        {#if notice.reportId}<button
            onclick={() => controller.openFirmwareReport(notice.reportId)}
            >Open source audit</button
          >{:else}<button onclick={() => controller.openFirmwareReport(null)}
            >Choose an audit</button
          >{/if}<button disabled={busy} onclick={() => dismiss(notice.id)}
          >Dismiss notice</button
        >
      </div>
    </article>{:else}{#if !loading}<p class="empty-inline muted">
        No outstanding notices. This does not establish current firmware or
        software status.
      </p>{/if}{/each}
  <p class="muted small">
    Latest requested update attempt: {label(
      center?.latestAttemptState ?? 'unknown',
    )} · {date(center?.latestAttemptAt ?? null)}
  </p>
</section>
<section class="panel feature-form">
  <h2>Optional review reminder</h2>
  <p class="muted">
    Disabled by default. A due reminder appears inside Coqui while the app is
    open, or once when reopened after missed time. It never runs an audit,
    contacts Apple or requests macOS notification permission.
  </p>
  <label class="check-row"
    ><input
      type="checkbox"
      bind:checked={enabled}
      disabled={loading || busy}
    />Remind me to review firmware and update evidence</label
  >
  <label
    >Review interval (days)<input
      aria-label="Firmware review interval in days"
      type="number"
      min="1"
      max="365"
      bind:value={days}
      disabled={loading || busy}
    /></label
  >
  <button
    disabled={loading ||
      busy ||
      !Number.isInteger(days) ||
      days < 1 ||
      days > 365}
    onclick={save}>Save review reminder</button
  >
  {#if center?.preferences.reminderEnabled}<p class="muted small">
      Next due {date(center.preferences.nextDueAt)}. Dismissing a due reminder
      schedules the next interval.
    </p>{/if}{#if message}<p role="status">{message}</p>{/if}
  <p class="muted small">
    {center?.qualification ?? 'No notice state loaded.'}
  </p>
</section>
