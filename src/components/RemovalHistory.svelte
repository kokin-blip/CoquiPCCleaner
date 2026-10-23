<script lang="ts">
  import type { createAppStore } from '../state';
  import type { RemovalPlan } from '../bindings/RemovalPlan';
  import RemovalReview from './RemovalReview.svelte';
  import { date, label } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let rows = $state<RemovalPlan[]>([]),
    offset = $state(0),
    error = $state(''),
    loading = $state(false),
    review = $state<RemovalPlan | null>(null),
    revision = $state(0);
  $effect(() => {
    const page = offset;
    void revision;
    let alive = true;
    loading = true;
    error = '';
    controller.service
      .removalHistory(page)
      .then((r) => {
        if (alive) rows = r;
      })
      .catch((e) => {
        if (alive) error = String(e);
      })
      .finally(() => {
        if (alive) loading = false;
      });
    return () => {
      alive = false;
    };
  });
  async function restore(p: RemovalPlan) {
    try {
      review = await controller.work('Preparing restore review', () =>
        controller.service.previewRestore(
          p.id,
          p.items.filter((i) => i.status === 'movedToTrash').map((i) => i.path),
        ),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel">
  <div class="section-heading">
    <h2>Removal and recovery records</h2>
    <button disabled={loading} onclick={() => revision++}
      >Refresh records</button
    >
  </div>
  <p class="muted">
    Only verified, retained Trash moves can be reviewed for restore. Existing
    destinations are never overwritten. Clearing diagnostic data removes Coqui’s
    recovery references.
  </p>
  {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
  {#if loading}<p role="status">Loading records…</p>{/if}
  {#each rows as p}<div class="activity-row">
      <div>
        <strong
          >{p.items.length} reviewed {p.items.every((i) => i.kind === 'restore')
            ? 'restore'
            : 'removal'} items · {label(p.status)}</strong
        >
        <p class="muted small">{date(p.createdAt)}</p>
        {#each p.items as item}<p class="path small selectable">
            {item.path} · {label(item.status)}
          </p>{/each}
      </div>
      <div>
        <button onclick={() => (review = p)}>Inspect record</button
        >{#if p.items.some((i) => i.status === 'movedToTrash')}<button
            disabled={$controller.workRunning || $controller.scanning}
            onclick={() => restore(p)}>Review restore</button
          >{/if}
      </div>
    </div>{:else}{#if !loading}<p>No retained removal records.</p>{/if}{/each}
  <div class="pagination">
    <button
      disabled={offset === 0 || loading}
      onclick={() => (offset = Math.max(0, offset - 50))}
      >Previous records</button
    ><span>Page {Math.floor(offset / 50) + 1}</span><button
      disabled={rows.length < 50 || loading}
      onclick={() => (offset += 50)}>Next records</button
    >
  </div>
</section>
{#if review}<RemovalReview
    {controller}
    initial={review}
    onclose={() => {
      review = null;
      revision++;
    }}
  />{/if}
