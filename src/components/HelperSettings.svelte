<script lang="ts">
  import type { createAppStore } from '../state';
  import type { HelperStatus } from '../bindings/HelperStatus';
  import Dialog from './Dialog.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let status = $state<HelperStatus | null>(null);
  let pending = $state(false);
  let error = $state('');
  let confirm = $state<'purge' | 'uninstall' | null>(null);
  async function check() {
    pending = true;
    error = '';
    try {
      status = await controller.service.administratorStatus();
    } catch (e) {
      error = String(e);
    } finally {
      pending = false;
    }
  }
  async function enable(enabled: boolean) {
    await controller.run(() => controller.work(
      enabled ? 'Administrator authorization — respond to macOS if prompted' : 'Locking administrator access',
      () => controller.save({
        ...$controller.settings!,
        administratorEnabled: enabled,
      }),
    ));
    if (status?.compatible) await check();
  }
  async function lockAccess() {
    await controller.run(() => controller.work('Locking administrator access', async () => {
      await controller.service.administratorLock();
      if (status) status = { ...status, authorizationSeconds: 0 };
    }));
  }
  async function lifecycle() {
    const action = confirm!;
    await controller.run(() => controller.work('Administrator operation — complete or cancel the macOS prompt', async () => {
      await controller.service.administratorLifecycle(action);
      confirm = null;
      if (action === 'uninstall') {
        await controller.save({
          ...$controller.settings!,
          administratorEnabled: false,
        });
        status = null;
      }
    }));
  }
</script>

<section class="panel settings-panel">
  <div class="section-heading">
    <h2>Administrator helper</h2>
    <button
      disabled={pending || $controller.workRunning || $controller.scanning}
      onclick={check}>Check installed helper</button
    >
  </div>
  <p>
    Optional and off by default. Install the matching app-and-helper package,
    then open <span class="path">/Applications/CoquiPCCleaner.app</span>.
    Updates refresh the build pairing; mismatches refuse access.
  </p>
  {#if pending}<p role="status">
      Checking the installed build and authenticated connection…
    </p>{/if}
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  {#if status}<p class="muted">{status.qualification}</p>{/if}
  <label class="switch-row"
    ><div>
      <strong>Offer administrator review and scanning</strong>
      <p class="muted small">
        Authenticate once to enable a session lasting up to 15 minutes while
        Coqui is open. Each scan and removal still needs its own review. Turn
        this option off to lock access. No scan or cleanup starts here.
      </p>
    </div>
    <input
      type="checkbox"
      role="switch"
      checked={$controller.settings?.administratorEnabled ?? false}
      disabled={(!$controller.settings?.administratorEnabled &&
        !status?.compatible) ||
        $controller.workRunning ||
        $controller.scanning}
      onchange={(e) => enable(e.currentTarget.checked)}
    /></label
  >
  {#if status && status.authorizationSeconds > 0}<div class="inline-wrap">
      <p class="muted small" role="status">
        At the last check: up to {Math.ceil(status.authorizationSeconds / 60)}
        minutes of authorization remaining. macOS may expire it earlier.
      </p>
      <button
        disabled={$controller.workRunning || $controller.scanning}
        onclick={lockAccess}>Lock administrator access</button
      >
    </div>{/if}
  <p class="muted small">
    The helper keeps a private journal separately from this app’s local history.
    Removing the helper preserves those recovery records; reinstall the matching
    helper to use them.
  </p>
  {#if status?.compatible}<div class="inline-wrap">
      <button
        onclick={() => controller.run(() => controller.service.settings('permissions'))}
        >Review helper privacy access</button
      >
      <button
        disabled={$controller.workRunning || $controller.scanning}
        onclick={() => (confirm = 'purge')}>Remove helper records…</button
      >
      <button
        disabled={$controller.workRunning || $controller.scanning}
        onclick={() => (confirm = 'uninstall')}
        >Remove administrator helper…</button
      >
    </div>{/if}
  {#if status?.compatible}<p class="muted small">
      If macOS refuses protected locations or Trash, review Full Disk Access for
      <span class="path">/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper</span>.
      This expands readable scope; each elevated operation still needs approval.
    </p>{/if}
</section>
{#if confirm}<Dialog
    title={confirm === 'purge'
      ? 'Remove your helper records?'
      : 'Remove administrator helper?'}
    onclose={() => (confirm = null)}
  >
    <p>
      {confirm === 'purge'
        ? 'Remove your retained helper scans, evidence and removal/recovery references, plus their imported local scans and public action mirrors. This does not remove your files or empty Trash. Other accounts’ helper records are preserved.'
        : 'Unregister Coqui’s service and remove its helper binary and pairing manifest. Your app, files and retained helper recovery records are preserved. Administrator features will be disabled.'}
    </p>
    <p class="muted">
      macOS administrator authorization is required for this operation.
    </p>
    {#if $controller.workRunning}<p role="status">{$controller.workLabel}</p>{/if}
    <div class="dialog-actions">
      <button onclick={() => (confirm = null)}>Cancel</button><button
        class="danger"
        disabled={$controller.workRunning}
        onclick={lifecycle}
        >{confirm === 'purge'
          ? 'Remove helper records'
          : 'Remove helper'}</button
      >
    </div>
  </Dialog>{/if}
