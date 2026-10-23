<script lang="ts">
  import { tick } from 'svelte';
  import type { createAppStore } from '../state';
  import type { RemovalRequest } from '../bindings/RemovalRequest';
  import type { RemovalPlan } from '../bindings/RemovalPlan';
  import type { FileEntry } from '../bindings/FileEntry';
  import Dialog from './Dialog.svelte';
  import { bytes, label } from '../format';
  let {
    controller,
    request = null,
    initial = null,
    onclose,
    oncomplete = () => {},
  }: {
    controller: ReturnType<typeof createAppStore>;
    request?: RemovalRequest | null;
    initial?: RemovalPlan | null;
    onclose: () => void;
    oncomplete?: () => void;
  } = $props();
  let plan = $state<RemovalPlan | null>(null),
    error = $state(''),
    pending = $state(false),
    accepted = $state(false);
  let index = $state(0),
    offset = $state(0),
    entries = $state<FileEntry[]>([]),
    contentsError = $state(''),
    contentsLoading = $state(false);
  let contentsHeading = $state<HTMLHeadingElement>();
  async function inspectContents(item: number) {
    index = item;
    offset = 0;
    await tick();
    contentsHeading?.scrollIntoView?.({ block: 'start' });
    contentsHeading?.focus({ preventScroll: true });
  }
  let active = $derived(plan?.status === 'running');
  let restore = $derived(
    plan?.items.every((i) => i.kind === 'restore') ?? false,
  );
  $effect(() => {
    const r = request;
    const saved = initial;
    let alive = true;
    accepted = false;
    error = '';
    plan = saved;
    if (!r) return;
    pending = true;
    controller
      .work('Preparing removal review', () =>
        controller.service.previewRemoval(r),
      )
      .then((p) => {
        if (alive) plan = p;
      })
      .catch((e) => {
        if (alive) error = String(e);
      })
      .finally(() => {
        if (alive) pending = false;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const p = plan;
    const i = index;
    const page = offset;
    let alive = true;
    entries = [];
    contentsError = '';
    if (!p || p.status !== 'review') return;
    contentsLoading = true;
    controller.service
      .removalEntries(p.id, i, page)
      .then((r) => {
        if (alive) entries = r;
      })
      .catch((e) => {
        if (alive) contentsError = String(e);
      })
      .finally(() => {
        if (alive) contentsLoading = false;
      });
    return () => {
      alive = false;
    };
  });
  async function execute() {
    if (!plan) return;
    const p = plan;
    error = '';
    accepted = false;
    try {
      plan = await controller.work(
        restore ? 'Restoring retained items' : 'Moving reviewed items to Trash',
        () =>
          controller.service.executeRemoval(
            { id: p.id, paths: p.items.map((i) => i.path), acknowledged: true },
            (v) => (plan = v),
          ),
      );
      oncomplete();
    } catch (e) {
      error = String(e);
      plan = null;
    }
  }
</script>

<Dialog title={restore ? 'Review restore' : 'Review removal'} {onclose}>
  {#if pending}<p role="status">Inspecting identities and folder contents…</p>
    <button
      onclick={() => controller.run(() => controller.service.cancelWork())}
      >Cancel review</button
    >{/if}
  {#if error}<div class="inline-error" role="alert">
      {error}
      <p>
        Close this review and select eligible items again. Existing cache and
        project workflows remain in Optimization.
      </p>
    </div>{/if}
  {#if plan}
    <p class="muted">{plan.qualification}</p>
    {#each plan.items as item, i}<section class="removal-item">
        <div class="inline-between">
          <strong class="path selectable">{item.path}</strong><span
            >{label(item.status)}</span
          >
        </div>
        <p class="small">
          {bytes(item.allocatedBytes)} allocated · {bytes(item.logicalBytes)} logical
          · {item.entryCount.toLocaleString()} entries
        </p>
        <p class="muted small">{item.association}</p>
        <p>{item.consequences}</p>
        {#if item.status !== 'pending'}<p class="small">{item.detail}</p>{/if}
        {#if item.trashPath}<p class="path small selectable">
            Trash: {item.trashPath}
          </p>{/if}
        {#if plan.status === 'review'}<button onclick={() => inspectContents(i)}
            >Inspect reviewed contents</button
          >{/if}
      </section>{/each}
    {#if plan.status === 'review'}
      <h3
        id="reviewed-contents-heading"
        bind:this={contentsHeading}
        tabindex="-1"
      >
        Contents of {plan.items[index].path.split('/').pop()}
      </h3>
      <p id="reviewed-contents-path" class="path selectable">
        {plan.items[index].path}
      </p>
      {#if contentsLoading}<p role="status">Checking reviewed contents…</p>{/if}
      {#if contentsError}<p class="inline-error" role="alert">
          {contentsError}
        </p>{/if}
      <div class="review-contents table-scroll">
        <table
          aria-labelledby="reviewed-contents-heading"
          aria-describedby="reviewed-contents-path"
        >
          <thead
            ><tr
              ><th>Exact path</th><th>Type</th><th class="numeric">Logical</th
              ></tr
            ></thead
          ><tbody
            >{#each entries as entry}<tr
                ><td class="path selectable">{entry.path}</td><td
                  >{entry.isDirectory ? 'Folder' : entry.category}</td
                ><td class="numeric">{bytes(entry.logicalBytes)}</td></tr
              >{/each}</tbody
          >
        </table>
      </div>
      <div class="pagination">
        <button
          disabled={offset === 0 || contentsLoading}
          onclick={() => (offset = Math.max(0, offset - 100))}
          >Previous contents</button
        ><span>Page {Math.floor(offset / 100) + 1}</span><button
          disabled={entries.length < 100 || contentsLoading}
          onclick={() => (offset += 100)}>Next contents</button
        >
      </div>
      <label class="check-row"
        ><input type="checkbox" bind:checked={accepted} />I reviewed these exact
        paths, folder contents and consequences. {restore
          ? 'Restore without overwriting.'
          : 'Move the selected items to Trash.'}</label
      >
      <div class="dialog-actions">
        <button onclick={onclose}>Cancel</button><button
          class="primary"
          disabled={!accepted ||
            contentsLoading ||
            !!contentsError ||
            $controller.workRunning}
          onclick={execute}
          >{restore ? 'Restore reviewed items' : 'Move to Trash'}</button
        >
      </div>
    {:else if active}<p role="status">Processing and verifying each move…</p>
      <button
        onclick={() => controller.run(() => controller.service.cancelWork())}
        >Cancel remaining moves</button
      >
    {:else}<p role="status">
        Review status: {plan.status}. Rescan to refresh storage results.
      </p>
      <button onclick={onclose}>Done</button>{/if}
  {/if}
</Dialog>

<style>
  .removal-item {
    padding: 16px 0;
    border-bottom: 1px solid var(--border);
  }
  .review-contents {
    max-height: 220px;
    overflow: auto;
  }
  .review-contents td:first-child {
    max-width: 460px;
    overflow-wrap: anywhere;
  }
</style>
