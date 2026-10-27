<script lang="ts">
  import ScanObservationView from '../components/ScanObservationView.svelte';
  import type { ScanObservations } from '../bindings/ScanObservations';
  import {
    ArrowRight,
    ArrowLeft,
    Activity,
    HardDrive,
    BrushCleaning,
    Package,
    ScanLine,
    LoaderCircle,
    ChevronRight,
    MessageCircle,
  } from '@lucide/svelte';
  import type { createAppStore } from '../state';
  import { bytes, date } from '../format';
  import Status from '../components/Status.svelte';
  import type { ScanReview } from '../bindings/ScanReview';
  import type { ScanReviewDestination } from '../bindings/ScanReviewDestination';
  import type { AppUpdateReport } from '../bindings/AppUpdateReport';
  import Recommendations from './Recommendations.svelte';
  let {
    controller,
    onScan,
  }: { controller: ReturnType<typeof createAppStore>; onScan: () => void } =
    $props();
  let session = $derived($controller.session);
  let machine = $derived(session?.snapshot);
  let live = $derived(
    $controller.monitoringStatus === 'live'
      ? $controller.samples.at(-1)
      : undefined,
  );
  let question = $state('');
  let busy = $derived(
    $controller.scanning ||
      $controller.workRunning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  let scanning = $derived($controller.scanning);
  let showingResults = $derived(
    scanning || $controller.overviewMode === 'results',
  );
  let terminal = $derived(session?.status !== 'running' ? session : null);
  let review = $state<ScanReview | null>(null);
  let reviewLoading = $state(false);
  let reviewError = $state('');
  let reviewRevision = $state(0);
  let updateReport = $state<AppUpdateReport | null>(null);
  let observations = $state<ScanObservations | null>(null);
  let observationError = $state('');
  let updateReadFailed = $state(false);
  let listedUpdates = $derived(
    updateReport?.items.filter((item) => item.state === 'available').length ??
      0,
  );
  let currentReview = $derived(
    !scanning && review?.sessionId === terminal?.id ? review : null,
  );
  let caches = $derived(currentReview?.groups.find((g) => g.kind === 'caches'));
  let startup = $derived(
    currentReview?.groups.find((g) => g.kind === 'startup'),
  );
  let applications = $derived(
    currentReview?.groups.find((g) => g.kind === 'applications'),
  );
  let largeFiles = $derived(
    currentReview?.groups.find((g) => g.kind === 'largeFiles'),
  );
  const groupLabels = {
    caches: 'Caches',
    development: 'Development files',
    startup: 'Startup configurations',
    applications: 'Applications',
    largeFiles: 'Files at least 1 GiB allocated',
  };
  $effect(() => {
    const id = terminal?.id;
    const visible = showingResults && !scanning;
    void reviewRevision;
    let alive = true;
    review = null;
    reviewError = '';
    reviewLoading = false;
    updateReport = null;
    observations = null;
    observationError = '';
    updateReadFailed = false;
    if (!id || !visible) return;
    reviewLoading = true;
    controller.service
      .scanObservations(id)
      .then((report) => {
        if (alive) observations = report;
      })
      .catch(() => {
        if (alive)
          observationError =
            'Saved metadata checks could not be read. Retry the saved findings.';
      });
    controller.service
      .scanReview(id)
      .then((value) => {
        if (alive) review = value;
      })
      .catch((error) => {
        if (alive) reviewError = String(error);
      })
      .finally(() => {
        if (alive) reviewLoading = false;
      });
    controller.service
      .latestAppUpdates()
      .then((value) => {
        if (alive) updateReport = value;
      })
      .catch(() => {
        if (alive) updateReadFailed = true;
      });
    return () => {
      alive = false;
    };
  });
  function openReview(destination: ScanReviewDestination) {
    switch (destination) {
      case 'cacheFiles':
        controller.openRecommendation('Optimization', 'cache');
        break;
      case 'developerArtifacts':
        controller.openRecommendation('Optimization', 'developer');
        break;
      case 'startup':
        controller.openRecommendation('Performance', 'startup');
        break;
      case 'applications':
        controller.navigate('Applications');
        break;
      case 'largeFiles':
        controller.openRecommendation('Storage', 'largeFiles');
        break;
    }
  }
</script>

<div class="overview">
  {#if !showingResults}
    <section class="overview-start" aria-label="Start a scan">
      <img
        src="/brand/coqui-file-emblem-v1.png"
        alt=""
        width="88"
        height="88"
      />
      <h1>Let’s check your Mac.</h1>
      <p class="muted">
        Scan storage and system details, then choose what to review.
      </p>
      <button
        class="primary overview-scan-button"
        onclick={onScan}
        disabled={busy}><ScanLine size={20} />Scan Mac</button
      >
      <span class="muted small"
        >{$controller.settings?.scanMode === 'deep'
          ? 'Whole internal disk · review volumes before scanning'
          : $controller.settings?.scanMode === 'selected'
            ? 'Selected folders · managed in Settings'
            : 'Home folder · managed in Settings'}</span
      >
      {#if terminal}
        <div class="overview-last-scan">
          <span class="muted small">Last scan · {date(terminal.startedAt)}</span
          >
          <button
            class="text-button"
            onclick={() => controller.openOverviewResults()}
            >View saved results<ArrowRight size={14} /></button
          >
        </div>
      {/if}
    </section>
  {:else}
    <div class="overview-heading">
      <h1>{scanning ? 'Scanning your Mac…' : 'Your scan results.'}</h1>
      <p class="muted">
        {machine
          ? `${machine.model} · ${machine.osVersion}`
          : 'Find what’s taking up space and what needs a closer look.'}
      </p>
      {#if !scanning}<button
          class="text-button overview-back"
          onclick={() => controller.showOverviewStart()}
          ><ArrowLeft size={14} />Back to start</button
        >{/if}
    </div>

    {#if scanning}
      <section
        class="overview-progress"
        aria-label="Scan progress"
        aria-live="polite"
      >
        <LoaderCircle class="spin" size={28} />
        <div>
          <h2>{session?.stage ?? 'Preparing your scan…'}</h2>
          <p class="muted">
            {session
              ? `${session.fileCount.toLocaleString()} files inspected`
              : 'Checking the selected scope'}
          </p>
        </div>
        <button
          disabled={$controller.cancelling}
          onclick={() => controller.cancel()}
        >
          {$controller.cancelling ? 'Stopping…' : 'Stop scan'}
        </button>
        {#if $controller.cancelling}<p
            class="overview-progress-note muted small"
          >
            Finishing the current operation and retaining partial results.
          </p>{/if}
      </section>
    {/if}

    <section class="overview-grid" aria-label="Explore your Mac">
      <button class="overview-tile" onclick={() => openReview('cacheFiles')}>
        <span class="overview-tile-heading"
          ><BrushCleaning size={22} /><span>Cleanup</span><ChevronRight
            size={17}
          /></span
        >
        <strong
          >{caches?.allocatedBytes != null
            ? `${bytes(caches.allocatedBytes)} in caches`
            : 'Review your files'}</strong
        >
        <span class="overview-tile-description"
          >{caches?.count != null
            ? `${caches.count.toLocaleString()} indexed cache files`
            : 'Caches, developer builds and duplicates'}</span
        >
        <span class="overview-tile-footnote"
          >{caches?.allocatedBytes != null
            ? 'Recorded footprint · removal eligibility checked separately'
            : 'Choose what to inspect before making changes'}</span
        >
      </button>
      <button class="overview-tile" onclick={() => openReview('startup')}>
        <span class="overview-tile-heading"
          ><Activity size={22} /><span>Performance</span><ChevronRight
            size={17}
          /></span
        >
        <strong
          >{startup?.count != null
            ? `${startup.count.toLocaleString()} startup configurations`
            : live
              ? `${live.cpuPercent.toFixed(1)}% CPU in use`
              : 'Measure your Mac'}</strong
        >
        <span class="overview-tile-description"
          >{startup?.count != null
            ? 'Review login and background configurations'
            : live
              ? `${bytes(live.usedMemoryBytes)} memory in use · live`
              : 'Live activity and build or test timing'}</span
        >
        <span class="overview-tile-footnote"
          >{$controller.monitoringStatus === 'failed'
            ? 'Monitoring unavailable · open to retry'
            : startup?.count != null
              ? 'Enabled state unverified · review before changing'
              : live
                ? 'Live activity only · startup not recorded'
                : 'Startup and performance measurements are unavailable'}</span
        >
      </button>
      <button
        class="overview-tile"
        onclick={() =>
          updateReport
            ? controller.openRecommendation('Applications', 'updates')
            : controller.navigate('Applications')}
      >
        <span class="overview-tile-heading"
          ><Package size={22} /><span>Applications</span><ChevronRight
            size={17}
          /></span
        >
        <strong
          >{updateReport
            ? listedUpdates > 0
              ? `${listedUpdates} newer version${listedUpdates === 1 ? '' : 's'} listed · saved ${date(updateReport.createdAt)}`
              : `Saved ${updateReport.status} update check · open results`
            : updateReadFailed
              ? 'Saved update results unavailable'
              : applications?.count != null
                ? `${applications.count.toLocaleString()} recorded apps`
                : 'Manage applications'}</strong
        >
        <span class="overview-tile-description"
          >Installed apps and their related files</span
        >
        <span class="overview-tile-footnote"
          >{updateReport
            ? `Saved ${updateReport.status} check · not refreshed by this scan`
            : applications?.count != null
              ? 'Saved observations · updates not checked'
              : 'Inventory and update availability not checked here'}</span
        >
      </button>
      <button
        class="overview-tile"
        onclick={() => controller.navigate('Storage')}
      >
        <span class="overview-tile-heading"
          ><HardDrive size={22} /><span>Storage</span><ChevronRight
            size={17}
          /></span
        >
        <strong
          >{terminal
            ? `${bytes(terminal.allocatedBytes)} indexed`
            : 'Explore your storage'}</strong
        >
        <span class="overview-tile-description"
          >{terminal
            ? largeFiles?.count != null
              ? `${largeFiles.count.toLocaleString()} files at least 1 GiB allocated`
              : `${terminal.fileCount.toLocaleString()} files · folders and large files`
            : 'Browse folders and find large files'}</span
        >
        <span class="overview-tile-footnote"
          >{terminal
            ? terminal.status === 'complete'
              ? 'Inspected allocation · open for coverage'
              : `${terminal.status[0].toUpperCase()}${terminal.status.slice(1)} scan · coverage gaps remain`
            : scanning
              ? 'Building your storage map…'
              : 'A scan is needed to build your storage map'}</span
        >
      </button>
    </section>

    {#if !scanning && terminal}
      {#if reviewLoading}<p class="muted small" role="status">
          Reading recorded scan findings…
        </p>{/if}
      {#if reviewError}<div class="inline-error" role="alert">
          <span>Recorded review totals could not be read. {reviewError}</span>
          <button onclick={() => reviewRevision++}>Retry review totals</button>
        </div>{/if}
      {#if currentReview?.groups.length || observations}
        <details class="overview-evidence">
          <summary>What was checked</summary>
          {#if observations}<ScanObservationView
              {controller}
              report={observations}
            />{:else if observationError}<p role="alert">
              {observationError}
            </p>{/if}
          {#each (currentReview?.groups ?? []).filter((group) => !observations || !['startup', 'applications'].includes(group.kind)) as group}
            <div class="overview-evidence-row">
              <div class="section-heading">
                <h3>{groupLabels[group.kind]}</h3>
                <Status value={group.state} />
              </div>
              <p class="muted small">
                {group.count == null
                  ? 'Count unavailable'
                  : `${group.count.toLocaleString()} recorded items`}{group.allocatedBytes !=
                null
                  ? ` · ${bytes(group.allocatedBytes)} allocated`
                  : ''} · {group.observedAt
                  ? date(group.observedAt)
                  : 'Observation date unavailable'}
              </p>
              <p class="small">{group.qualification}</p>
              <button
                class="text-button"
                onclick={() => openReview(group.destination)}
                >{group.kind === 'largeFiles'
                  ? 'Browse files by size'
                  : `Review ${groupLabels[group.kind].toLowerCase()}`}<ArrowRight
                  size={14}
                /></button
              >
            </div>
          {/each}
          <div class="overview-evidence-row">
            <h3>Separate checks</h3>
            <p class="small muted">
              This metadata scan does not read content to find duplicates or
              similar photos, or contact update services.
            </p>
            <div class="button-row">
              <button
                onclick={() =>
                  controller.openRecommendation('Optimization', 'duplicates')}
                >Review duplicate checks</button
              ><button
                onclick={() =>
                  controller.openRecommendation('Applications', 'updates')}
                >Review app update checks</button
              >
            </div>
          </div>
          {#if updateReport}<div class="overview-evidence-row">
              <div class="section-heading">
                <h3>App update check</h3>
                <Status value={updateReport.status} />
              </div>
              <p class="small muted">
                {updateReport.items.length} selected apps · {date(
                  updateReport.createdAt,
                )}
              </p>
              <p class="small">{updateReport.qualification}</p>
              <button
                class="text-button"
                onclick={() =>
                  controller.openRecommendation('Applications', 'updates')}
                >Review app updates<ArrowRight size={14} /></button
              >
            </div>{/if}
        </details>
      {/if}
    {/if}

    {#if !scanning}
      <section class="overview-scan" aria-label="Scan your Mac">
        <div>
          <h2>{session ? 'Ready for a fresh look?' : 'Start with a scan.'}</h2>
          <p class="muted">Scan your Mac, then choose what to review.</p>
        </div>
        <button
          class="primary overview-scan-button"
          onclick={onScan}
          disabled={busy}><ScanLine size={20} />Scan again</button
        >
      </section>
    {/if}

    {#if terminal}
      <div class="overview-saved">
        <span>Saved scan · {date(terminal.startedAt)}</span>
        <Status value={terminal.status} />
        <button
          class="text-button"
          onclick={() => controller.navigate('System Analysis')}
          >View scan results<ArrowRight size={14} /></button
        >
      </div>
    {/if}
  {/if}

  <details class="overview-disclosure">
    <summary
      ><MessageCircle size={17} /><span>Ask about a problem</span><ChevronRight
        size={16}
      /></summary
    >
    <div class="overview-disclosure-content">
      <form
        class="feature-form"
        onsubmit={(e) => {
          e.preventDefault();
          if (question.trim()) controller.startInvestigation(question);
        }}
      >
        <label for="dashboard-question"
          >What problem are you experiencing with your computer?</label
        >
        <input
          id="dashboard-question"
          maxlength="4000"
          required
          bind:value={question}
          placeholder="For example, my Mac is slow after login"
        />
        <div class="button-row">
          <button class="primary" disabled={busy || !question.trim()}
            >Ask Coqui</button
          >
          <button
            type="button"
            disabled={busy}
            onclick={() =>
              controller.startInvestigation('What is consuming my storage?')}
            >Review storage</button
          >
          <button
            type="button"
            disabled={busy}
            onclick={() =>
              controller.startInvestigation(
                'Is my machine still suitable for software development?',
              )}>Assess my workload</button
          >
        </div>
      </form>
      <p class="muted small">
        Local guidance uses saved evidence. Choose a separate task for fresh
        measurements.
      </p>
    </div>
  </details>
  {#if showingResults}<details class="overview-disclosure">
      <summary
        ><ScanLine size={17} /><span>Suggested next steps</span><ChevronRight
          size={16}
        /></summary
      >
      <div class="overview-disclosure-content">
        <Recommendations {controller} compact />
      </div>
    </details>{/if}
</div>

<style>
  .overview-start {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    min-height: min(560px, calc(100vh - 185px));
    gap: 14px;
    padding: 40px 24px;
  }
  .overview-start > img {
    object-fit: contain;
    margin-bottom: 10px;
  }
  .overview-start h1 {
    font-size: 32px;
  }
  .overview-start > p {
    max-width: 48ch;
    line-height: 1.6;
  }
  .overview-start > button {
    margin: 12px 0 2px;
  }
  .overview-last-scan {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    margin-top: 28px;
  }
  .overview-back {
    margin-top: 12px;
    font-size: 12px;
  }
  .overview {
    max-width: 1040px;
    margin: 0 auto;
  }
  .overview-heading {
    margin: 2px 0 28px;
  }
  .overview-heading h1 {
    font-size: 28px;
  }
  .overview-heading p {
    margin-top: 8px;
    font-size: 13px;
  }
  .overview-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
  }
  .overview-tile {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 0;
    min-width: 0;
    min-height: 190px;
    padding: 24px;
    text-align: left;
    border-radius: 12px;
    background: var(--surface);
  }
  .overview-tile:hover {
    background: var(--hover);
  }
  .overview-tile-heading {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    font-size: 15px;
    font-weight: 600;
  }
  .overview-tile-heading :global(svg:first-child) {
    color: var(--accent-text);
  }
  .overview-tile-heading :global(svg:last-child) {
    margin-left: auto;
    color: var(--secondary);
  }
  .overview-tile > strong {
    font-size: 25px;
    font-weight: 600;
    letter-spacing: -0.025em;
    line-height: 1.25;
    margin: 24px 0 6px;
    font-variant-numeric: tabular-nums;
  }
  .overview-tile-description {
    font-size: 13px;
    line-height: 1.5;
  }
  .overview-tile-footnote {
    font-size: 11px;
    color: var(--secondary);
    line-height: 1.5;
    margin-top: 12px;
  }
  .overview-scan {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 24px 0;
  }
  .overview-scan h2 {
    font-size: 17px;
  }
  .overview-scan p {
    font-size: 12px;
    margin-top: 5px;
  }
  .overview-scan-button {
    min-width: 156px;
    min-height: 46px;
    font-size: 14px;
    gap: 10px;
  }
  .overview-saved {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding-bottom: 20px;
    font-size: 11px;
    color: var(--secondary);
  }
  .overview-saved > button {
    margin-left: auto;
    font-size: 12px;
  }
  .overview-progress {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 20px;
    padding: 20px 24px;
    background: var(--accent-wash);
    border-radius: 12px;
  }
  .overview-progress > :global(svg) {
    color: var(--accent-text);
  }
  .overview-progress h2 {
    font-size: 19px;
  }
  .overview-progress p {
    margin-top: 5px;
  }
  .overview-progress > button {
    margin-left: auto;
  }
  .overview-progress-note {
    flex-basis: 100%;
  }
  .overview-disclosure {
    border-top: 1px solid var(--border);
  }
  .overview-disclosure > summary {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 15px 0;
    cursor: pointer;
    list-style: none;
    font-size: 12px;
  }
  .overview-disclosure > summary::-webkit-details-marker {
    display: none;
  }
  .overview-disclosure > summary :global(svg) {
    color: var(--secondary);
  }
  .overview-disclosure > summary :global(svg:last-child) {
    margin-left: auto;
  }
  .overview-disclosure[open] > summary :global(svg:last-child) {
    transform: rotate(90deg);
  }
  .overview-disclosure-content {
    padding: 4px 0 20px;
  }
  .overview-disclosure-content > p {
    margin-top: 12px;
  }
  .overview-evidence {
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    margin-top: 20px;
  }
  .overview-evidence > summary {
    padding: 14px 0;
    cursor: pointer;
    font-size: 12px;
  }
  .overview-evidence-row {
    padding: 16px 0;
    border-top: 1px solid var(--border);
  }
  .overview-evidence-row h3 {
    font-size: 14px;
  }
  .overview-evidence-row p {
    margin: 8px 0;
    max-width: 75ch;
  }
  @media (max-width: 1150px) {
    .overview-heading {
      margin-bottom: 22px;
    }
    .overview-tile {
      min-height: 176px;
      padding: 20px;
    }
    .overview-tile > strong {
      font-size: 23px;
      margin-top: 20px;
    }
    .overview-grid {
      gap: 12px;
    }
  }
</style>
