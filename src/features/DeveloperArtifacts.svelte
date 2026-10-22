<script lang="ts">
  import {
    CodeXml,
    ExternalLink,
    RefreshCw,
    ChevronLeft,
    ChevronRight,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import type { DeveloperArtifact } from '../bindings/DeveloperArtifact';
  import type { DeveloperReport } from '../bindings/DeveloperReport';
  import Dialog from '../components/Dialog.svelte';
  import Status from '../components/Status.svelte';
  import { bytes, date } from '../format';
  let {
    controller,
    onScan,
    onReview,
  }: {
    controller: ReturnType<typeof createAppStore>;
    onScan: () => void;
    onReview: (path: string) => void;
  } = $props();
  let saved = $state<DeveloperReport | null>(null);
  let loading = $state(false);
  let error = $state('');
  let revision = $state(0);
  let detail = $state<DeveloperArtifact | null>(null);
  let query = $state('');
  let technology = $state('All technologies');
  let assessment = $state('All assessments');
  let page = $state(0);
  let sessionId = $derived($controller.session?.id);
  let report = $derived(
    $controller.developerReport?.sessionId === $controller.session?.id
      ? $controller.developerReport
      : saved,
  );
  let busy = $derived(
    $controller.workRunning || $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  function supported(a: DeveloperArtifact) {
    const name = a.path.split('/').at(-1);
    return (
      (a.technology === 'Rust' && name === 'target') ||
      (a.technology === 'Node.js' &&
        ['.next', '.nuxt', '.svelte-kit', 'dist', 'build'].includes(
          name ?? '',
        )) ||
      (a.technology === 'Dart / Flutter' &&
        ['build', '.dart_tool'].includes(name ?? '')) ||
      (a.technology === 'Python' && name === '__pycache__')
    );
  }
  let stacks = $derived(
    [...new Set(report?.artifacts.map((a) => a.technology) ?? [])].sort(),
  );
  let filtered = $derived(
    (report?.artifacts ?? []).filter(
      (a) =>
        (technology === 'All technologies' || a.technology === technology) &&
        (assessment === 'All assessments' || a.assessment === assessment) &&
        a.path.toLowerCase().includes(query.toLowerCase()),
    ),
  );
  $effect(() => {
    void query;
    void technology;
    void assessment;
    void report;
    page = 0;
  });
  $effect(() => {
    const id = sessionId;
    const refresh = revision;
    let alive = true;
    void refresh;
    saved = null;
    detail = null;
    loading = false;
    error = '';
    if (!id) return;
    loading = true;
    error = '';
    controller.service
      .developerReport(id)
      .then((r) => {
        if (alive) saved = r;
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

<section class="cleanup-intro">
  <div class="section-heading">
    <h2>Developer artifacts</h2>
    <button
      class="primary"
      disabled={!$controller.session || busy}
      onclick={() => controller.analyzeDeveloper()}
    >
      <RefreshCw size={15} />{report ? 'Recheck artifacts' : 'Check artifacts'}
    </button>
  </div>
  <p>
    Review project caches, dependencies and build outputs with their Git and
    worktree context.
  </p>
  {#if $controller.session}<p class="path muted small">
      Scan scope: {$controller.session.scope}
    </p>{/if}
  <p class="muted small">
    Sizes describe the storage scan. Git validation is a later observation.
    Supported folders require a separate fresh inspection and your exact-path
    approval.
  </p>
</section>
{#if $controller.cleanupPreparing}<div class="scan-banner" role="status">
    <span
      >Preparing a fresh folder review: inspecting contents, Git state and
      worktree relationships…</span
    >
    <button
      onclick={() => controller.run(() => controller.service.cancelCleanup())}
      >Cancel review preparation</button
    >
  </div>{/if}
{#if error}<div class="inline-error" role="alert">
    <span>{error}</span><button onclick={() => revision++}
      >Retry loading review</button
    >
  </div>{/if}
{#if loading}<p role="status">Loading retained developer review…</p>{/if}
{#if $controller.developerRunning}<div class="scan-banner" role="status">
    <CodeXml size={18} /><span
      >Checking project boundaries and Git state · {report?.examined ?? 0} of {report?.candidateCount ??
        0} indexed output-like directories examined</span
    >
    <button
      onclick={() => controller.run(() => controller.service.cancelDeveloper())}
      >Cancel validation</button
    >
  </div>{/if}
{#if report}
  <div class="inline-between">
    <p class="muted small">
      Git checked {date(report.checkedAt)} · {report.artifacts.length} artifacts ·
      {report.examined} indexed directories examined
    </p>
    <Status
      value={report.status === 'running' && !$controller.developerRunning
        ? 'interrupted'
        : report.status}
    />
  </div>
  <p class="muted small">{report.qualification}</p>
  {#if report.status === 'running' && !$controller.developerRunning}<p
      role="status"
    >
      The previous validation did not finish. Recheck before using these
      observations.
    </p>{/if}
  {#if report.errors.length}<details>
      <summary>Inspection gaps ({report.errors.length})</summary
      >{#each report.errors as message}<p class="path small">
          {message}
        </p>{/each}
    </details>{/if}
  <section class="panel file-panel">
    <div class="file-toolbar inline-wrap">
      <input
        aria-label="Filter artifact paths"
        type="search"
        placeholder="Filter paths"
        bind:value={query}
      />
      <select aria-label="Technology" bind:value={technology}
        ><option>All technologies</option>{#each stacks as stack}<option
            >{stack}</option
          >{/each}</select
      >
      <select aria-label="Assessment" bind:value={assessment}
        ><option>All assessments</option><option>Protected</option><option
          >Review required</option
        ></select
      >
    </div>
    <div class="table-scroll">
      <table>
        <thead
          ><tr
            ><th>Artifact / exact path</th><th>Technology</th><th
              class="numeric">Indexed allocation</th
            ><th>Git</th><th>Assessment</th><th
              ><span class="sr-only">Inspect</span></th
            ></tr
          ></thead
        >
        <tbody
          >{#each filtered.slice(page * 25, (page + 1) * 25) as artifact}<tr>
              <td class="cleanup-path"
                ><strong>{artifact.kind}</strong>
                <div class="path selectable small">{artifact.path}</div></td
              >
              <td>{artifact.technology}</td><td class="numeric nowrap"
                >{bytes(artifact.indexedAllocatedBytes)}</td
              >
              <td>{artifact.git.status}</td><td>{artifact.assessment}</td>
              <td
                ><button
                  aria-label={`Inspect ${artifact.path}`}
                  onclick={() => (detail = artifact)}
                  >Inspect<ChevronRight size={14} /></button
                ></td
              >
            </tr>{/each}</tbody
        >
      </table>
    </div>
    {#if !filtered.length}<div class="empty-inline">
        <h3>
          {report.artifacts.length
            ? 'No matching artifacts.'
            : 'No recognized artifacts in this index.'}
        </h3>
        <p>
          {report.artifacts.length
            ? 'Change the filters to see more results.'
            : 'Scan a project folder containing supported build outputs, then check again. Unrecognized paths are not classified as disposable.'}
        </p>
      </div>{/if}
    <div class="pagination">
      <button disabled={page === 0} onclick={() => page--}
        ><ChevronLeft size={14} />Previous</button
      ><span class="muted small"
        >Page {page + 1} · {filtered.length} matching</span
      ><button
        disabled={(page + 1) * 25 >= filtered.length}
        onclick={() => page++}>Next<ChevronRight size={14} /></button
      >
    </div>
  </section>
{:else if !loading}
  <div class="empty-state">
    <CodeXml size={32} />
    <h2>
      {$controller.session
        ? 'Check the projects in this scan.'
        : 'Start with a project-folder scan.'}
    </h2>
    <p>
      Recognizes Rust, Node.js, Dart/Flutter, Python and Xcode output patterns.
      Checks use installed Git and do not run build scripts.
    </p>
    {#if !$controller.session}<button onclick={onScan}>Analyze storage</button
      >{:else}<button onclick={() => controller.navigate('Settings')}
        >Choose scan scope</button
      >{/if}
  </div>
{/if}
{#if detail}<Dialog
    title="Developer artifact review"
    onclose={() => (detail = null)}
  >
    <p class="path selectable"><strong>{detail.path}</strong></p>
    <p>
      <strong>{detail.assessment}</strong> · {detail.technology} · {detail.kind}
    </p>
    <dl>
      <dt>Project marker</dt>
      <dd>{detail.marker}</dd>
      <dt>Project</dt>
      <dd class="path selectable">{detail.projectRoot}</dd>
      <dt>Indexed allocation / logical size</dt>
      <dd>
        {bytes(detail.indexedAllocatedBytes)} / {bytes(
          detail.indexedLogicalBytes,
        )} · {detail.indexedFileCount} indexed {detail.indexedFileCount === 1
          ? 'file'
          : 'files'}
      </dd>
      <dt>Storage observed</dt>
      <dd>{date(detail.scannedAt)}</dd>
      <dt>Git checked</dt>
      <dd>{date(detail.checkedAt)}</dd>
    </dl>
    <h3>Review findings</h3>
    {#each detail.reasons as reason}<p>{reason}</p>{/each}
    <h3>Technology consequences</h3>
    <p>{detail.consequences}</p>
    <h3>Git and worktrees</h3>
    <p>{detail.git.status} · {detail.git.qualification}</p>
    {#if detail.git.changes}<p>
        {detail.git.changes.staged} staged · {detail.git.changes.modified} modified
        · {detail.git.changes.untracked} untracked entries · {detail.git.changes
          .conflicts} conflicts
      </p>{/if}
    <dl>
      <dt>Repository</dt>
      <dd class="path selectable">{detail.git.root ?? 'Unavailable'}</dd>
      <dt>Git directory</dt>
      <dd class="path selectable">
        {detail.git.gitDirectory ?? 'Unavailable'}
      </dd>
      <dt>Shared Git directory</dt>
      <dd class="path selectable">
        {detail.git.commonDirectory ?? 'Unavailable'}
      </dd>
      <dt>Tracked index entries / gitlinks in output</dt>
      <dd>
        {detail.trackedEntries ?? 'Unknown'} / {detail.gitlinks ?? 'Unknown'}
      </dd>
      <dt>Nested markers in scan index</dt>
      <dd>
        {detail.nestedMarkers
          ? 'Detected — preserve nested project'
          : 'Not detected in indexed coverage'}
      </dd>
    </dl>
    {#each detail.git.worktrees as worktree}<div class="cleanup-review-item">
        <strong class="path selectable">{worktree.path}</strong>
        <p>
          {worktree.branch ??
            (worktree.detached ? 'Detached HEAD' : 'No branch reported')} · {worktree.head ??
            'No confirmed commit'}
        </p>
        <p class="muted small">
          {worktree.bare ? 'Bare repository · ' : ''}{worktree.locked
            ? 'Locked · '
            : ''}{worktree.prunable
            ? 'Prunable registration · '
            : ''}{worktree.present
            ? 'Directory present'
            : 'Directory unavailable'} · Other worktrees’ dirty state is not inspected.
        </p>
      </div>{/each}
    <p class="muted small">{report?.qualification}</p>
    <div class="dialog-actions">
      <button
        onclick={() =>
          controller.run(() => controller.service.reveal(detail!.path))}
        >Reveal output<ExternalLink size={14} /></button
      ><button onclick={() => (detail = null)}>Done</button>
    </div>
    {#if supported(detail) && detail.assessment === 'Review required'}
      <div class="dialog-actions">
        <button
          disabled={busy}
          class="primary"
          onclick={() => {
            const path = detail!.path;
            detail = null;
            onReview(path);
          }}>Review folder cleanup</button
        >
      </div>
      <p class="muted small">
        A fresh inspection may refuse this folder. Only ignored, untracked
        outputs in a clean checkout are eligible. You must confirm that all
        contents can be regenerated.
      </p>
    {:else}<p class="muted small">
        Analysis only for this artifact or current assessment. Preserve it and
        inspect its project with the appropriate tooling.
      </p>{/if}
  </Dialog>{/if}
