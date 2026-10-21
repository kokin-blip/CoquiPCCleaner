<script lang="ts">
  import { Search, ChevronRight, History } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let loading = $state(false);
  let allLoaded = $state(false);
  async function more() {
    loading = true;
    await controller.run(async () => {
      const batch = await controller.service.history(
        $controller.history.length,
        100,
      );
      controller.update((s) => ({ ...s, history: [...s.history, ...batch] }));
      allLoaded = batch.length < 100;
    });
    loading = false;
  }
</script>

<div class="page-heading">
  <div>
    <h1>History</h1>
    <p class="muted">
      A record of what was inspected, what was observed, and what remained
      unavailable.
    </p>
  </div>
  <span class="subtle-tag"
    >{$controller.settings?.retentionDays} day retention</span
  >
</div>
<div class="callout">
  <History size={20} />
  <div>
    <strong>Cleanup actions have their own record.</strong>
    <p>
      Analysis history stays here. Inspect approved changes, verification
      results and available Trash locations in Optimization.
    </p>
    <button
      class="text-button"
      onclick={() => controller.navigate('Optimization')}
      >Open Optimization<ChevronRight size={14} /></button
    >
  </div>
</div>
<section class="panel history-panel">
  {#each $controller.history as session}<button
      class="history-row"
      onclick={() => controller.selectSession(session)}
      ><span class="history-icon"><Search size={18} /></span><span
        class="history-description"
        ><strong>System analysis</strong><span class="muted small"
          >{date(session.startedAt)} · {session.fileCount.toLocaleString()} files
          · {bytes(session.allocatedBytes)} allocated</span
        ><span class="path muted small">{session.scope}</span></span
      ><Status value={session.status} /><ChevronRight size={15} /></button
    >{:else}<div class="empty-state">
      <History size={35} />
      <h2>Your next scan starts the record.</h2>
      <p>
        Completed, partial, cancelled and interrupted scans are retained
        locally.
      </p>
    </div>{/each}{#if $controller.history.length >= 100 && !allLoaded}<div
      class="pagination"
    >
      <button disabled={loading} onclick={more}
        >{loading ? 'Loading…' : 'Load older scans'}</button
      >
    </div>{/if}
</section>
