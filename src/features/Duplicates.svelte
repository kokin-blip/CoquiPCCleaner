<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { DuplicateReport } from '../bindings/DuplicateReport';
  import type { CleanupPlan } from '../bindings/CleanupPlan';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  let {
    controller,
    onReview,
  }: {
    controller: ReturnType<typeof createAppStore>;
    onReview: (plan: CleanupPlan) => void;
  } = $props();
  let report = $state<DuplicateReport | null>(null),
    error = $state(''),
    loading = $state(false);
  let groupPage = $state(0);
  let keep = $state<Record<string, string>>({}),
    selected = $state<string[]>([]);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    void restore();
  });
  async function restore() {
    loading = true;
    try {
      const rows = await controller.service.records('duplicates', 0);
      for (const row of rows) {
        const r = await controller.service.record(row.kind, row.id);
        if (
          r.kind === 'duplicates' &&
          r.data.sessionId === $controller.session?.id
        ) {
          report = r.data;
          break;
        }
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function analyze() {
    if (!$controller.session) return;
    error = '';
    report = null;
    selected = [];
    groupPage = 0;
    keep = {};
    try {
      report = await controller.work('Duplicate content analysis', () =>
        controller.service.analyzeDuplicates(
          $controller.session!.id,
          (r) => (report = r),
        ),
      );
    } catch (e) {
      error = String(e);
    }
  }
  async function preview(groupId: string, paths: string[]) {
    if (!report) return;
    error = '';
    try {
      const plan = await controller.work('Verifying duplicate copies', () =>
        controller.service.previewDuplicates({
          reportId: report!.id,
          groupId,
          keepPath: keep[groupId],
          removePaths: paths,
        }),
      );
      onReview(plan);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="section-heading">
  <div>
    <h2>Exact copies and similar photos</h2>
    <p class="muted">
      Content reads are opt-in. Intentional backups and project copies may be
      valuable.
    </p>
  </div>
  <button
    class="primary"
    disabled={busy || !$controller.session}
    onclick={analyze}>Analyze contents</button
  >
</div>
<p class="muted small">
  Up to 2,000 indexed files, 256 MiB per file and 2 GiB of initial hashing.
  PNG/JPEG similarity is a review hint; only freshly byte-verified exact copies
  support approval. Home personal folders only for duplicate cleanup; project
  and Library contents keep their separate protections.
</p>
{#if error}<div class="inline-error" role="alert">
    {error}<button onclick={restore}>Retry saved results</button>
  </div>{/if}
{#if loading}<p role="status">Loading retained duplicate report…</p>{/if}
{#if report}
  <section class="panel">
    <div class="section-heading">
      <h2>{report.stage}</h2>
      <Status value={report.status} />
    </div>
    <p aria-live="polite">
      {report.examined} / {report.total} indexed files examined · {bytes(
        report.hashedBytes,
      )} hashed · {report.groups.length} exact groups
    </p>
    <p class="muted small">{date(report.createdAt)} · {report.qualification}</p>
    {#if report.gaps.length}<details>
        <summary>{report.gaps.length} coverage notes</summary
        >{#each report.gaps as gap}<p class="path small">{gap}</p>{/each}
      </details>{/if}
  </section>
  {#if report.groups.length > 10}<div class="button-row">
      <button disabled={groupPage === 0} onclick={() => groupPage--}
        >Previous groups</button
      ><span class="muted small"
        >Groups {groupPage * 10 + 1}–{Math.min(
          report.groups.length,
          groupPage * 10 + 10,
        )} of {report.groups.length}</span
      ><button
        disabled={(groupPage + 1) * 10 >= report.groups.length}
        onclick={() => groupPage++}>Next groups</button
      >
    </div>{/if}
  {#each report.groups.slice(groupPage * 10, groupPage * 10 + 10) as group}
    <section class="panel">
      <div class="section-heading">
        <h2>
          {group.files.length} verified copies · {bytes(
            group.files[0].logicalBytes,
          )} each
        </h2>
        <span class="muted small">SHA-256 + byte comparison</span>
      </div>
      <p class="muted">
        Choose a keeper explicitly, then select copies to review. Equal contents
        do not imply equal purpose.
      </p>
      <div class="table-wrap">
        <table>
          <thead
            ><tr
              ><th>Keep</th><th>Review removal</th><th>Exact path / modified</th
              ><th>Allocated</th><th>Inspect</th></tr
            ></thead
          ><tbody
            >{#each group.files.slice(0, 100) as file}<tr
                ><td
                  ><input
                    type="radio"
                    name={`keeper-${group.id}`}
                    aria-label={`Keep ${file.path}`}
                    checked={keep[group.id] === file.path}
                    disabled={busy}
                    onchange={() => {
                      keep = { ...keep, [group.id]: file.path };
                      selected = selected.filter((p) => p !== file.path);
                    }}
                  /></td
                ><td
                  ><input
                    type="checkbox"
                    aria-label={`Review removal of ${file.path}`}
                    disabled={busy ||
                      !file.reviewSupported ||
                      keep[group.id] === file.path}
                    checked={selected.includes(file.path)}
                    onchange={(e) =>
                      (selected = e.currentTarget.checked
                        ? [...selected, file.path]
                        : selected.filter((p) => p !== file.path))}
                  /></td
                ><td
                  ><span class="path">{file.path}</span>
                  <p class="muted small">{date(file.modifiedAt)}</p>
                  <p class="muted small">
                    {file.reviewQualification ||
                      'Eligibility unavailable; run a fresh content analysis.'}
                  </p></td
                ><td>{bytes(file.allocatedBytes)}</td><td
                  ><button
                    onclick={() =>
                      controller.run(() =>
                        controller.service.reveal(file.path),
                      )}>Finder</button
                  ></td
                ></tr
              >{/each}</tbody
          >
        </table>
      </div>
      {#if group.files.length > 100}<p class="muted small">
          First 100 copies shown; narrow the scan scope to inspect further
          copies.
        </p>{/if}
      <button
        disabled={busy ||
          !keep[group.id] ||
          !group.files.some((f) => selected.includes(f.path))}
        onclick={() =>
          preview(
            group.id,
            group.files
              .filter((f) => selected.includes(f.path))
              .map((f) => f.path),
          )}>Review selected copies</button
      >
    </section>
  {/each}
  {#if report.status !== 'running' && !report.groups.length}<div
      class="empty-state compact"
    >
      <h2>No exact groups confirmed.</h2>
      <p>
        This covers only the report's readable candidates and bounds. It does
        not establish that all files are unique.
      </p>
    </div>{/if}
  {#if report.photoMatches.length}<section class="panel">
      <h2>Similar / possible modified photos</h2>
      <p class="muted">
        Perceptual matches can be wrong. Inspect both originals; no removal
        controls apply to these pairs.
      </p>
      {#each report.photoMatches as pair}<div class="resource-pair">
          <div>
            <strong>{pair.classification} · distance {pair.distance}/64</strong>
            <p class="path small">{pair.left}</p>
            <p class="path small">{pair.right}</p>
            <p class="muted small">{pair.qualification}</p>
          </div>
          <div class="button-row">
            <button
              onclick={() =>
                controller.run(() => controller.service.reveal(pair.left))}
              >First in Finder</button
            ><button
              onclick={() =>
                controller.run(() => controller.service.reveal(pair.right))}
              >Second in Finder</button
            >
          </div>
        </div>{/each}
    </section>{/if}
{:else if !loading}<div class="empty-state compact">
    <h2>Compare the actual contents.</h2>
    <p>
      Run a storage scan, then choose Analyze contents. Cancellation keeps a
      qualified partial report.
    </p>
  </div>{/if}
