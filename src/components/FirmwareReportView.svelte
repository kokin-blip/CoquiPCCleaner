<script lang="ts">
  import type { FirmwareReport } from '../bindings/FirmwareReport';
  import Evidence from './Evidence.svelte';
  import Status from './Status.svelte';
  import { date, label } from '../format';
  let { report }: { report: FirmwareReport } = $props();
  const updateLabels: Record<string, string> = {
    pending: 'Update check queued after platform identification',
    withheld: 'Update check withheld: platform eligibility unconfirmed',
    notRequested: 'Update check not requested',
    checking: 'Checking with macOS',
    noneOffered: 'No updates offered at this check',
    available: 'Software updates offered',
    partial: 'Partial update listing',
    unknown: 'Update availability unknown',
    unavailable: 'Update check unavailable',
    requiresPermission: 'Update check requires permission',
    cancelled: 'Update check cancelled',
  };
</script>

<section class="panel firmware-report" aria-label="Firmware audit result">
  <div class="section-heading">
    <h2>{report.platform.model} · {report.platform.chip}</h2>
    <Status value={report.status} />
  </div>
  <p>
    {report.platform.manufacturer} · {report.platform.architecture} · macOS {report
      .platform.osVersion} · Build {report.platform.osBuild}
  </p>
  <p class="muted small">
    Saved audit {date(report.createdAt)} · Finished {date(report.finishedAt)}
  </p>
  {#if report.status === 'running'}<p role="status">{report.stage}</p>{/if}
  <p class="muted">{report.platform.qualification}</p>
  {#if report.errors.length}<details open>
      <summary>Coverage gaps ({report.errors.length})</summary>
      <ul>
        {#each report.errors as error}<li>{error}</li>{/each}
      </ul>
    </details>{/if}
  {#if report.evidence.length}<Evidence items={report.evidence} />{:else}<p
      class="empty-inline muted"
    >
      No optional property readings recorded in this audit.
    </p>{/if}
  {#each report.findings as finding}<article class="firmware-finding">
      <h3>{finding.title}</h3>
      <p>{finding.explanation}</p>
      <p class="muted small">{finding.confidence}</p>
    </article>{/each}
  <h3>{updateLabels[report.updateState] ?? label(report.updateState)}</h3>
  <p class="muted small">
    {report.updateCheckedAt
      ? `macOS response recorded ${date(report.updateCheckedAt)}`
      : 'No successful macOS listing response recorded.'} Saved listings can become
    stale immediately; recheck before making decisions.
  </p>
  {#if report.updates.length}<div class="table-scroll">
      <table>
        <thead
          ><tr><th>Offered software</th><th>Version</th><th>Restart</th></tr
          ></thead
        ><tbody
          >{#each report.updates as update}<tr
              ><td
                ><strong>{update.title}</strong>
                <p class="muted small">{update.qualification}</p></td
              ><td>{update.version}</td><td
                >{update.restart
                  ? 'Listed as required'
                  : 'Confirm in Software Update'}</td
              ></tr
            >{/each}</tbody
        >
      </table>
    </div>{/if}
  <p class="muted small">{report.qualification}</p>
  <details>
    <summary>Capability and model coverage</summary
    >{#each report.capabilities as capability}<div class="capability-row">
        <div>
          <strong>{capability.name}</strong>
          <p class="muted">{capability.explanation}</p>
        </div>
        <Status value={capability.state} />
      </div>{/each}
  </details>
</section>
