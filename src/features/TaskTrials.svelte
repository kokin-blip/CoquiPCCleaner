<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { WorkloadTrial } from '../bindings/WorkloadTrial';
  import type { TrialComparison } from '../bindings/TrialComparison';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import WorkloadTrialView from '../components/WorkloadTrialView.svelte';
  import TrialComparisonView from '../components/TrialComparisonView.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let workload = $state(''),
    task = $state(''),
    inputs = $state(''),
    application = $state(''),
    conditions = $state(''),
    targetSeconds = $state(120);
  let records = $state<ActivityRecord[]>([]),
    inspected = $state<WorkloadTrial | null>(null),
    error = $state(''),
    loading = $state(false),
    offset = $state(0);
  let beforeIds = $state<string[]>([]),
    afterIds = $state<string[]>([]),
    change = $state(''),
    comparison = $state<TrialComparison | null>(null),
    saved = $state(false),
    comparing = $state(false);
  let busy = $derived(
    comparing ||
      $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  let running = $derived(
    $controller.workRunning && $controller.workloadTrial?.status === 'running',
  );
  let current = $derived(
    running
      ? $controller.workloadTrial
      : (inspected ?? $controller.workloadTrial),
  );
  let refreshRevision = 0,
    inspectRevision = 0;
  onMount(() => {
    workload = $controller.settings?.workload.workloads[0] ?? '';
    void refresh();
  });
  async function refresh() {
    const revision = ++refreshRevision;
    loading = true;
    error = '';
    try {
      const result = await controller.service.records('workloadTrial', offset);
      if (revision === refreshRevision) records = result;
    } catch (e) {
      if (revision === refreshRevision) error = String(e);
    } finally {
      if (revision === refreshRevision) loading = false;
    }
  }
  async function capture() {
    ++inspectRevision;
    error = '';
    inspected = null;
    try {
      inspected = await controller.captureTrial({
        workload,
        task,
        inputs,
        application,
        conditions,
        targetSeconds,
      });
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
  async function finish() {
    try {
      await controller.finishTrial();
    } catch (e) {
      error = String(e);
    }
  }
  async function inspect(id: string) {
    const revision = ++inspectRevision;
    try {
      const record = await controller.service.record('workloadTrial', id);
      if (revision === inspectRevision && record.kind === 'workloadTrial')
        inspected = record.data;
    } catch (e) {
      if (revision === inspectRevision) error = String(e);
    }
  }
  function reuse() {
    if (!current) return;
    ({ workload, task, inputs, application, conditions, targetSeconds } =
      current.request);
  }
  function choose(id: string, group: 'before' | 'after', checked: boolean) {
    beforeIds = beforeIds.filter((value) => value !== id);
    afterIds = afterIds.filter((value) => value !== id);
    if (checked) {
      if (group === 'before') beforeIds = [...beforeIds, id];
      else afterIds = [...afterIds, id];
    }
    comparison = null;
    saved = false;
  }
  async function compare(save: boolean) {
    if (comparing) return;
    comparing = true;
    error = '';
    try {
      const request = {
        beforeIds: [...beforeIds],
        afterIds: [...afterIds],
        change,
      };
      comparison = save
        ? await controller.work('Save task comparison', () =>
            controller.service.saveTrialComparison(request),
          )
        : await controller.service.compareTrials(request);
      saved = save;
    } catch (e) {
      error = String(e);
    } finally {
      comparing = false;
    }
  }
</script>

<section class="panel" aria-label="Time a build or test">
  <h2>Time a build or test</h2>
  <p class="muted">
    Choose a repeatable task and target. When the timer says Running, perform
    the task yourself, then report completion. Coqui records the interval and
    available activity samples; it does not execute your build or verify its
    output.
  </p>
  <form
    class="feature-form"
    onsubmit={(e) => {
      e.preventDefault();
      void capture();
    }}
  >
    <div class="form-grid">
      <label
        >Workload<input
          required
          maxlength="2000"
          bind:value={workload}
          disabled={busy}
        /></label
      >
      <label
        >Task<input
          required
          maxlength="2000"
          placeholder="Clean build of MyApp"
          bind:value={task}
          disabled={busy}
        /></label
      >
      <label
        >Inputs / project<input
          required
          maxlength="2000"
          placeholder="Project, revision, build profile and cache state"
          bind:value={inputs}
          disabled={busy}
        /></label
      >
      <label
        >Application / tool version<input
          required
          maxlength="2000"
          placeholder="IDE or compiler version"
          bind:value={application}
          disabled={busy}
        /></label
      >
      <label
        >Comparison conditions<input
          required
          maxlength="2000"
          placeholder="Power source, background apps and warm/cold run"
          bind:value={conditions}
          disabled={busy}
        /></label
      >
      <label
        >Target duration (seconds)<input
          type="number"
          required
          min="0.001"
          max="86400"
          step="any"
          bind:value={targetSeconds}
          disabled={busy}
        /></label
      >
    </div>
    <p class="muted small">
      Each capture stops after ten minutes. Longer tasks remain uncompleted in
      this flow. Human start/finish latency is part of the recorded interval.
    </p>
    <button class="primary" disabled={busy}>Start task timer</button>
  </form>
</section>
{#if error}<p class="inline-error" role="alert">
    {error}<button onclick={refresh}>Reload saved trials</button>
  </p>{/if}
{#if $controller.workRunning && $controller.workLabel === 'Timed task trial' && !$controller.workloadTrial}<p
    role="status"
  >
    Preparing the timer. Wait for Running before starting your task.
  </p>{/if}
{#if current}<section class="panel" aria-label="Current task trial">
    <WorkloadTrialView trial={current} />
    {#if running}<p role="status">
        Running. Perform your task now; report completion when it finishes.
        Cancel operation keeps an incomplete result.
      </p>
      <button
        class="primary"
        disabled={$controller.trialFinishing}
        onclick={finish}
        >{$controller.trialFinishing
          ? 'Recording completion…'
          : 'Task completed — stop timer'}</button
      >
    {:else}<button disabled={busy} onclick={reuse}
        >Reuse this task protocol</button
      >{/if}
  </section>{/if}
<section class="panel" aria-label="Saved task trials">
  <div class="section-heading">
    <h2>Saved task trials</h2>
    <button disabled={loading} onclick={refresh}>Refresh trials</button>
  </div>
  <p class="muted">
    Repeat the same task and inputs. Add completed trials to the two comparison
    groups; each group accepts up to twenty trials.
  </p>
  {#if loading}<p role="status">Loading task trials…</p>{/if}
  {#if records.length}<div class="table-scroll">
      <table>
        <thead
          ><tr
            ><th>Task / recorded</th><th>State</th><th>Before</th><th>After</th
            ></tr
          ></thead
        ><tbody>
          {#each records as row}<tr
              ><td
                ><button
                  disabled={running}
                  class="text-button trial-title"
                  onclick={() => inspect(row.id)}>{row.title}</button
                >
                <p class="muted small">{date(row.createdAt)}</p></td
              ><td><Status value={row.status} /></td>
              {#each ['before', 'after'] as group}<td
                  >{#if row.status === 'complete'}<input
                      type="checkbox"
                      aria-label={`${group === 'before' ? 'Before' : 'After'}: ${row.title} · ${date(row.createdAt)}`}
                      checked={(group === 'before'
                        ? beforeIds
                        : afterIds
                      ).includes(row.id)}
                      disabled={busy ||
                        ((group === 'before' ? beforeIds : afterIds).length >=
                          20 &&
                          !(group === 'before' ? beforeIds : afterIds).includes(
                            row.id,
                          ))}
                      onchange={(e) =>
                        choose(
                          row.id,
                          group as 'before' | 'after',
                          e.currentTarget.checked,
                        )}
                    />{:else}<span class="muted">Unavailable</span>{/if}</td
                >{/each}
            </tr>{/each}
        </tbody>
      </table>
    </div>{:else if !loading}<p class="muted">
      No task trials recorded yet. Start a timer above; an interval baseline
      remains available in Baselines.
    </p>{/if}
  <div class="button-row">
    <button
      disabled={loading || offset === 0}
      onclick={() => {
        offset = Math.max(0, offset - 50);
        void refresh();
      }}>Previous trials</button
    ><button
      disabled={loading || records.length < 50}
      onclick={() => {
        offset += 50;
        void refresh();
      }}>Next trials</button
    >
  </div>
</section>
<section class="panel" aria-label="Compare task trials">
  <h2>Compare repeated task results</h2>
  <p>
    {beforeIds.length} before · {afterIds.length} after selected. Three or more trials
    per group help reveal variation; matching declared conditions are still not verified
    controls.
  </p>
  <div class="feature-form">
    <label
      >Comparison / intended change<input
        required
        maxlength="2000"
        placeholder="For example, before and after a manually changed build setting"
        bind:value={change}
        disabled={busy}
        oninput={() => {
          comparison = null;
          saved = false;
        }}
      /></label
    >
  </div>
  {#if comparing}<p role="status">Preparing task comparison…</p>{/if}
  <div class="button-row">
    <button
      disabled={busy || !beforeIds.length || !afterIds.length || !change.trim()}
      onclick={() => compare(false)}>Compare selected trials</button
    >
    {#if comparison}<button
        disabled={busy || saved}
        onclick={() => compare(true)}
        >{saved
          ? 'Comparison saved to History'
          : 'Save comparison to History'}</button
      >{/if}
  </div>
</section>
{#if comparison}<section class="panel">
    <TrialComparisonView {comparison} />
  </section>{/if}

<style>
  .trial-title {
    overflow-wrap: anywhere;
    white-space: normal;
    text-align: left;
  }
</style>
