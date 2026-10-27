<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { DuplicateReport } from '../bindings/DuplicateReport';
  import type { CleanupPlan } from '../bindings/CleanupPlan';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  import PhotoPreview from '../components/PhotoPreview.svelte';
  import DuplicateContext from '../components/DuplicateContext.svelte';
  import type { DuplicateLocation } from '../bindings/DuplicateLocation';
  import type { DuplicateFile } from '../bindings/DuplicateFile';
  import type { DuplicateGroup } from '../bindings/DuplicateGroup';
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
  let copyPages = $state<Record<string, number>>({});
  let location = $state<DuplicateLocation>('downloads');
  let similarPhotos = $state(false);
  let search = $state('');
  let age = $state('any');
  let sort = $state('largest');
  let keep = $state<Record<string, string>>({}),
    selected = $state<string[]>([]);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  let groups = $derived.by(() => {
    if (!report) return [];
    const observed = Date.parse(report.createdAt);
    const query = search.trim().toLocaleLowerCase();
    const matches = report.groups.filter((group) => group.files.some((file) => {
      if (query && !file.path.toLocaleLowerCase().includes(query)) return false;
      if (age === 'any') return true;
      const modified = Date.parse(file.modifiedAt);
      if (age === 'unknown') return !Number.isFinite(modified) || !Number.isFinite(observed);
      return Number.isFinite(observed) && Number.isFinite(modified)
        && modified <= observed - Number(age) * 86_400_000;
    }));
    const oldest = (files: DuplicateFile[]) => Math.min(...files.map((file) => Date.parse(file.modifiedAt)).filter(Number.isFinite));
    return matches.sort((a, b) => sort === 'oldest'
      ? oldest(a.files) - oldest(b.files)
      : sort === 'smallest' ? (a.files[0]?.logicalBytes ?? 0) - (b.files[0]?.logicalBytes ?? 0)
      : (b.files[0]?.logicalBytes ?? 0) - (a.files[0]?.logicalBytes ?? 0));
  });
  function clearSelection() { selected = []; keep = {}; groupPage = 0; copyPages = {}; }
  function changeLocation() { report = null; clearSelection(); age = 'any'; search = ''; }
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
          location = r.data.request?.location ?? 'downloads';
          similarPhotos = r.data.request?.similarPhotos ?? false;
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
    copyPages = {};
    keep = {};
    try {
      report = await controller.work('Duplicate content analysis', () =>
        controller.service.analyzeDuplicateScope(
          { sessionId: $controller.session!.id, location, similarPhotos },
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

{#snippet copyNavigation(group: DuplicateGroup)}
  {#if group.files.length > 100}<div class="pagination" aria-label="Copies in this exact group">
    <button disabled={busy || !(copyPages[group.id] ?? 0)} onclick={() => copyPages = {...copyPages, [group.id]: (copyPages[group.id] ?? 0) - 1}}>Previous copies</button>
    <span class="small muted">Copies {(copyPages[group.id] ?? 0) * 100 + 1}–{Math.min(group.files.length, ((copyPages[group.id] ?? 0) + 1) * 100)} of {group.files.length}</span>
    <button disabled={busy || ((copyPages[group.id] ?? 0) + 1) * 100 >= group.files.length} onclick={() => copyPages = {...copyPages, [group.id]: (copyPages[group.id] ?? 0) + 1}}>Next copies</button>
  </div>{/if}
{/snippet}

<div class="section-heading">
  <div>
    <h2>Exact copies</h2>
    <p class="muted">
      Content reads are opt-in. Intentional backups and project copies may be
      valuable.
    </p>
  </div>
  <button
    class="primary"
    disabled={busy || loading || !$controller.session}
    onclick={analyze}>Analyze contents</button
  >
</div>
<div class="duplicate-options">
  <label>Content-check location
    <select bind:value={location} disabled={busy || loading} onchange={changeLocation}>
      <option value="downloads">Downloads only</option>
      <option value="indexed">All indexed locations</option>
    </select>
  </label>
  <label class="duplicate-photo-option"><input type="checkbox" bind:checked={similarPhotos} disabled={busy || loading} />Include similar PNG/JPEG photos in the next check</label>
</div>
<p class="small muted">
  {location === 'downloads'
    ? 'Compare indexed files in your account’s Downloads folder.'
    : 'Compare candidates across the saved scan’s indexed locations.'}
  Browser history and download URLs are not read. Choosing a location starts no
  analysis; Analyze contents starts the content reads.
</p>
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
      )} hashed · {report.groups.length} exact {report.groups.length === 1 ? 'group' : 'groups'}
    </p>
    <p class="muted small">{date(report.createdAt)}</p>
    <DuplicateContext {report} />
    <details><summary>Read limits and verification</summary><p class="muted small">{report.qualification}</p></details>
    {#if report.gaps.length}<details>
        <summary>{report.gaps.length} coverage notes</summary
        >{#each report.gaps as gap}<p class="path small">{gap}</p>{/each}
      </details>{/if}
  </section>
  {#if report.groups.length}
    <div class="duplicate-filters" role="group" aria-label="Filter saved exact groups">
      <label>Find a path<input type="search" bind:value={search} oninput={clearSelection} placeholder="Name or folder" disabled={busy} /></label>
      <label>Last modified<select bind:value={age} onchange={clearSelection} disabled={busy}>
        <option value="any">Any date</option><option value="30">At least 30 days before this report</option><option value="90">At least 90 days before this report</option><option value="365">At least 365 days before this report</option><option value="unknown">Age unavailable</option>
      </select></label>
      <label>Sort groups<select bind:value={sort} onchange={clearSelection} disabled={busy}>
        <option value="largest">Largest copies first</option><option value="smallest">Smallest copies first</option><option value="oldest">Oldest modification first</option>
      </select></label>
    </div>
    <p class="small muted" aria-live="polite">{groups.length} of {report.groups.length} exact groups shown · {selected.length} copies selected.</p>
    <p class="small muted">Filters use this report’s recorded paths and last-modified dates, not download dates or last use. A matching group keeps every copy available for keeper review. Changing filters clears selections and starts no content reads.</p>
  {/if}
  {#if groups.length > 10}<div class="button-row">
      <button disabled={groupPage === 0} onclick={() => groupPage--}
        >Previous groups</button
      ><span class="muted small"
        >Groups {groupPage * 10 + 1}–{Math.min(
          groups.length,
          groupPage * 10 + 10,
        )} of {groups.length}</span
      ><button
        disabled={(groupPage + 1) * 10 >= groups.length}
        onclick={() => groupPage++}>Next groups</button
      >
    </div>{/if}
  {#each groups.slice(groupPage * 10, groupPage * 10 + 10) as group}
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
      {#if group.files.length > 100}
        <p class="small muted">Every retained copy is available across these pages. Page changes keep your explicit choices; changing filters clears them.</p>
        <p class="small">Keeper: <span class="path">{keep[group.id] || 'Not selected'}</span> · {group.files.filter((file) => selected.includes(file.path)).length} copies selected across pages.</p>
      {/if}
      {@render copyNavigation(group)}
      <div class="table-wrap">
        <table>
          <thead
            ><tr
              ><th>Keep</th><th>Review removal</th><th>Exact path / modified</th
              ><th>Allocated</th><th>Inspect</th></tr
            ></thead
          ><tbody
            >{#each group.files.slice((copyPages[group.id] ?? 0) * 100, ((copyPages[group.id] ?? 0) + 1) * 100) as file}<tr
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
      {@render copyNavigation(group)}
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
  {#if report.groups.length && !groups.length}<div class="empty-state compact">
    <h2>No groups match these filters.</h2><p>The report still retains its other groups. Clear filters to review them.</p>
    <button onclick={() => { search = ''; age = 'any'; clearSelection(); }}>Clear filters</button>
  </div>{/if}
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
        controls apply to these pairs. The exact-group filters above do not filter these photo hints.
      </p>
      {#each report.photoMatches as pair}<div class="resource-pair">
          <div>
            <strong>{pair.classification} · distance {pair.distance}/64</strong>
            <div class="photo-pair-previews">
              <PhotoPreview {controller} reportId={report.id} path={pair.left}/>
              <PhotoPreview {controller} reportId={report.id} path={pair.right}/>
            </div>
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

<style>
  .duplicate-options, .duplicate-filters { display:flex; flex-wrap:wrap; align-items:end; gap:16px; margin-block:16px; }
  .duplicate-options label:not(.duplicate-photo-option), .duplicate-filters label { display:grid; gap:6px; }
  .duplicate-photo-option { display:flex; align-items:center; gap:8px; padding-block:6px; }
  .duplicate-filters input, .duplicate-filters select { max-width:100%; }
  .duplicate-filters label { min-width:0; }
  .photo-pair-previews { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:16px; }
  @media(max-width:1100px) { .photo-pair-previews { grid-template-columns:1fr; } }
</style>
