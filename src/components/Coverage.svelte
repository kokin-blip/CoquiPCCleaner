<script lang="ts">
  import type { createAppStore } from '../state';
  import type { CoverageGap } from '../bindings/CoverageGap';
  let {
    controller,
    id,
  }: { controller: ReturnType<typeof createAppStore>; id: string } = $props();
  let offset = $state(0),
    rows = $state<CoverageGap[]>([]),
    error = $state(''),
    loading = $state(false),
    revision = $state(0);
  const labels: Record<string, string> = {
    notStarted: 'Not started',
    running: 'Traversing',
    complete: 'Traversal complete',
    completedWithOmissions: 'Traversal complete with omissions',
    cancelled: 'Cancelled',
    unavailable: 'Unavailable',
    permissionDenied: 'Permission denied',
    cloudPlaceholder: 'Cloud-only placeholder',
    symbolicLink: 'Symbolic link',
    mountBoundary: 'Another mounted filesystem',
    duplicateIdentity: 'Already counted identity',
    filesystemRace: 'Path changed or unavailable',
    changedParent: 'Parent identity changed',
    excluded: 'Excluded',
    specialFile: 'Special filesystem object',
    unrepresentableName: 'Unrepresentable filename',
    disconnected: 'Disconnected',
  };
  const label = (key: string) => labels[key] ?? key;
  $effect(() => {
    const key = id;
    const page = offset;
    void revision;
    let alive = true;
    loading = true;
    error = '';
    controller.service
      .coverage(key, page)
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
</script>

<section class="panel">
  <div class="section-heading">
    <h2>Traversal and access coverage</h2>
    <button disabled={loading} onclick={() => revision++}
      >Refresh coverage</button
    >
  </div>
  <p class="muted">
    Completed traversal describes readable metadata in each root. It does not
    mean comprehensive disk access. Exclusions, links, cloud placeholders and
    mount boundaries are deliberately skipped.
  </p>
  {#if $controller.session?.volumePlan}<h3>Reviewed mounted volumes</h3>
    {#each $controller.session.volumePlan.volumes.filter((v) => v.selected) as volume}<p
        class="path small selectable"
      >
        {volume.mount} · {volume.filesystem} · {volume.readOnly
          ? 'Read-only'
          : 'Subject to permissions'} · {volume.qualification}
      </p>{/each}{/if}
  {#each $controller.session?.coverage ?? [] as root}<div class="activity-row">
      <div>
        <strong class="path">{root.root}</strong>
        <p>
          {label(root.state)} · {root.files.toLocaleString()} files · {root.visited.toLocaleString()}
          entries visited
        </p>
        <p class="muted small">
          {Object.entries(root.skips)
            .map(([reason, count]) => `${label(reason)}: ${count}`)
            .join(' · ') || 'No recorded omissions'}
        </p>
      </div>
    </div>{/each}
  {#if error}<p class="inline-error" role="alert">
      {error}
    </p>{/if}{#if loading}<p role="status">Loading coverage records…</p>{/if}
  <div class="table-scroll">
    <table>
      <thead><tr><th>Root</th><th>Exact path</th><th>Reason</th></tr></thead
      ><tbody
        >{#each rows as row}<tr
            ><td class="path">{row.root}</td><td class="path selectable"
              >{row.path}</td
            ><td>{label(row.reason)}</td></tr
          >{/each}</tbody
      >
    </table>
  </div>
  {#if !rows.length && !loading}<p>
      No additional omission records on this page. Older scans may predate
      detailed coverage recording.
    </p>{/if}
  <div class="pagination">
    <button
      disabled={offset === 0 || loading}
      onclick={() => (offset = Math.max(0, offset - 100))}
      >Previous omissions</button
    ><span>Page {Math.floor(offset / 100) + 1}</span><button
      disabled={rows.length < 100 || loading}
      onclick={() => (offset += 100)}>Next omissions</button
    >
  </div>
  <button
    onclick={() =>
      controller.run(() => controller.service.settings('permissions'))}
    >Open Full Disk Access settings</button
  >
</section>

<style>
  td {
    max-width: 430px;
    overflow-wrap: anywhere;
  }
</style>
