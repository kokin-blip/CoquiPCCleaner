<script lang="ts">
  import type { createAppStore } from '../state';
  import type { RetainedRecord } from '../bindings/RetainedRecord';
  import Evidence from './Evidence.svelte';
  import FirmwareComparisonView from './FirmwareComparisonView.svelte';
  import FirmwareReportView from './FirmwareReportView.svelte';
  import FirmwarePlanView from './FirmwarePlanView.svelte';
  import Status from './Status.svelte';
  import { bytes, date } from '../format';
  let {
    controller,
    record,
    onUpdate = () => {},
  }: {
    controller: ReturnType<typeof createAppStore>;
    record: RetainedRecord;
    onUpdate?: (r: RetainedRecord) => void;
  } = $props();
  let ack = $state(false),
    error = $state('');
  async function verify(performed: boolean) {
    if (record.kind !== 'manualPlan') return;
    const id = record.data.id;
    try {
      const p = await controller.work('Observe manual plan outcome', () =>
        controller.service.verifyManualPlan(id, performed),
      );
      onUpdate({ kind: 'manualPlan', data: p });
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if error}<p class="inline-error" role="alert">{error}</p>{/if}
{#if record.kind === 'firmwareComparison'}<FirmwareComparisonView comparison={record.data}/>{:else if record.kind === 'firmware'}<FirmwareReportView
    report={record.data}
  />{:else if record.kind === 'firmwarePlan'}<FirmwarePlanView
    {controller}
    plan={record.data}
    onUpdate={(p) => onUpdate({ kind: 'firmwarePlan', data: p })}
  />{:else if record.kind === 'baseline'}<h2>{record.data.request.workload}</h2>
  <Status value={record.data.status} />
  <p>{record.data.request.target} · {record.data.request.context}</p>
  <p class="muted">{record.data.qualification}</p>
  <Evidence items={record.data.summary} />
  <details>
    <summary>Power observations</summary><Evidence items={record.data.power} />
  </details>
  <details>
    <summary>{record.data.samples.length} recorded samples</summary>
    <div class="table-wrap">
      <table>
        <thead
          ><tr
            ><th>Observed</th><th>CPU</th><th>Used memory</th><th>Swap</th></tr
          ></thead
        ><tbody
          >{#each record.data.samples as sample}<tr
              ><td>{date(sample.observedAt)}</td><td
                >{sample.cpuPercent.toFixed(1)}%</td
              ><td>{bytes(sample.usedMemoryBytes)}</td><td
                >{bytes(sample.swapBytes)}</td
              ></tr
            >{/each}</tbody
        >
      </table>
    </div>
  </details>
{:else if record.kind === 'manualPlan'}<h2>{record.data.requested}</h2>
  <Status value={record.data.status} />
  <h3>Previous observations</h3>
  <Evidence items={record.data.previous} />
  <ol>
    {#each record.data.steps as step}<li>{step}</li>{/each}
  </ol>
  <p>{record.data.verification}</p>
  {#if record.data.status === 'awaitingManualAction'}<button
      onclick={() =>
        controller.run(() =>
          controller.service.settings(
            record.kind === 'manualPlan' ? record.data.kind : '',
          ),
        )}>Open System Settings</button
    ><label class="check-row"
      ><input type="checkbox" bind:checked={ack} />I performed this exact manual
      step myself.</label
    >
    <div class="button-row">
      <button disabled={$controller.workRunning} onclick={() => verify(false)}
        >Decline plan</button
      ><button
        disabled={!ack || $controller.workRunning}
        onclick={() => verify(true)}>Record new observations</button
      >
    </div>{/if}<Evidence items={record.data.observed} />
  <p class="muted">{record.data.recovery}</p>
{:else if record.kind === 'maintenance'}<h2>Maintenance occurrence</h2>
  <Status value={record.data.status} />
  <p>{record.data.message}</p>
  {#if record.data.scanId}<p class="path small">
      Scan: {record.data.scanId}
    </p>{/if}
{:else if record.kind === 'investigation'}<h2>{record.data.title}</h2>
  <p class="muted">{record.data.qualification}</p>
  {#each record.data.messages as m}<article class="message">
      <strong>{m.role === 'user' ? 'You' : 'Coqui · local guide'}</strong>
      <p>{m.text}</p>
    </article>{/each}{#each record.data.tasks as task}<h3>{task.name}</h3>
    <p>{task.detail}</p>
    <Evidence items={task.evidence} />{/each}
{:else if record.kind === 'advice'}<h2>{record.data.assessment}</h2>
  <p>{record.data.workload}</p>
  <p class="muted">{record.data.qualification}</p>
  {#each record.data.findings as f}<h3>{f.title}</h3>
    <p>{f.explanation}</p>
    <Evidence items={f.evidence} />{/each}<Evidence
    items={record.data.scenarios}
  />
  <div class="source-links">
    {#each record.data.sources as href}<a
        {href}
        onclick={(e) => {
          e.preventDefault();
          void controller.run(() => controller.service.reference(href));
        }}
        target="_blank"
        rel="noreferrer">{href}</a
      >{/each}
  </div>
{:else if record.kind === 'resources'}<h2>
    Developer resources and organization
  </h2>
  <p class="muted">{record.data.qualification}</p>
  {#each record.data.capabilities as c}<h3>{c.name}</h3>
    <Status value={c.state} />
    <p>{c.explanation}</p>{/each}<Evidence
    items={record.data.evidence}
  />{#each record.data.suggestions as f}<h3>{f.title}</h3>
    <p>{f.explanation}</p>{/each}
{:else if record.kind === 'duplicates'}<h2>Duplicate content report</h2>
  <Status value={record.data.status} />
  <p>
    {record.data.examined} of {record.data.total} files examined · {bytes(
      record.data.hashedBytes,
    )} hashed
  </p>
  <p class="muted">{record.data.qualification}</p>
  {#each record.data.groups as group}<h3>
      {group.files.length} exact copies · {bytes(group.files[0].logicalBytes)}
    </h3>
    {#each group.files as file}<p class="path">
        {file.path}
      </p>{/each}{/each}{#each record.data.photoMatches as pair}<h3>
      {pair.classification}
    </h3>
    <p class="path small">{pair.left}<br />{pair.right}</p>
    <p class="muted small">
      {pair.qualification}
    </p>{/each}{#each record.data.gaps as gap}<p class="small path">
      {gap}
    </p>{/each}{/if}
