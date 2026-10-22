<script lang="ts">
  import type { createAppStore } from '../state';
  import type { ResourceReview } from '../bindings/ResourceReview';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let report = $state<ResourceReview | null>(null),
    error = $state(''),
    loading = $state(false);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  $effect(() => {
    const id = $controller.session?.id;
    let alive = true;
    report = null;
    if (id) {
      loading = true;
      controller.service
        .records('resources', 0)
        .then(async (rows) => {
          for (const row of rows) {
            const r = await controller.service.record(row.kind, row.id);
            if (r.kind === 'resources' && r.data.sessionId === id) {
              if (alive) report = r.data;
              break;
            }
          }
        })
        .catch((e) => {
          if (alive) error = String(e);
        })
        .finally(() => {
          if (alive) loading = false;
        });
    }
    return () => {
      alive = false;
    };
  });
  async function analyze() {
    if (!$controller.session) return;
    error = '';
    try {
      report = await controller.work('Review developer resources', () =>
        controller.service.resources($controller.session!.id),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="section-heading">
  <div>
    <h2>Developer resources and organization</h2>
    <p class="muted">
      Project outputs, application-native storage and unconfirmed associations.
    </p>
  </div>
  <button disabled={busy || !$controller.session} onclick={analyze}
    >Review indexed resources</button
  >
</div>
{#if error}<p class="inline-error" role="alert">
    {error}<button onclick={analyze}>Retry</button>
  </p>{/if}{#if loading}<p role="status">Loading resource review…</p>{/if}
{#if report}<section class="panel">
    <h2>Coverage and supported next steps</h2>
    <p class="muted small">{date(report.createdAt)} · {report.qualification}</p>
    {#each report.capabilities as capability}<div class="resource-pair">
        <div>
          <strong>{capability.name}</strong>
          <p class="muted">{capability.explanation}</p>
        </div>
        <Status value={capability.state} />
      </div>{/each}
  </section>
  <section class="panel">
    <h2>Indexed folder evidence</h2>
    {#if report.evidence.length}<Evidence items={report.evidence} />{:else}<p
        class="muted"
      >
        No matching folder evidence in this scope. Choose relevant accessible
        developer folders and rescan.
      </p>{/if}
  </section>
  {#each report.suggestions as f}<section class="panel">
      <h2>{f.title}</h2>
      <p>{f.explanation}</p>
      <span class="muted small">{f.confidence}</span>
    </section>{/each}{:else if !loading}<div class="empty-state compact">
    <h2>Inspect storage in its project context.</h2>
    <p>
      Start from the saved index. Developer artifacts performs fresh
      Git/worktree checks; application-native resources include explicit manual
      guidance.
    </p>
  </div>{/if}
