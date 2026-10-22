<script lang="ts">
  import type { createAppStore } from '../state';
  import type { ManualPlan } from '../bindings/ManualPlan';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let kind = $state('power'),
    requested = $state(
      'Balanced: inspect the normal / Automatic power mode offered by this Mac',
    ),
    plan = $state<ManualPlan | null>(null),
    error = $state(''),
    ack = $state(false);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  async function prepare() {
    error = '';
    ack = false;
    try {
      plan = await controller.work('Record manual settings plan', () =>
        controller.service.manualPlan(kind, requested),
      );
    } catch (e) {
      error = String(e);
    }
  }
  async function verify(performed: boolean) {
    if (!plan) return;
    error = '';
    try {
      plan = await controller.work('Observe manual plan outcome', () =>
        controller.service.verifyManualPlan(plan!.id, performed),
      );
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="panel">
  <h2>Recorded manual settings plan</h2>
  <p class="muted">
    Coqui records your intent and readable observations. You make any change in
    System Settings. Availability depends on the Mac; login-item runtime
    coverage remains incomplete.
  </p>
  <form
    class="feature-form"
    onsubmit={(e) => {
      e.preventDefault();
      void prepare();
    }}
  >
    <label
      >Setting<select bind:value={kind} disabled={busy}
        ><option value="power">Power preference</option><option value="login"
          >Login item</option
        ></select
      ></label
    >{#if kind === 'power'}<label
        >Desired preference<select
          disabled={busy}
          onchange={(e) =>
            (requested = `${e.currentTarget.value}: inspect the corresponding mode offered by this Mac`)}
          ><option>Balanced</option><option>Performance</option><option
            >Efficiency</option
          ></select
        ></label
      >{/if}<label
      >Exact requested change / item<textarea
        required
        maxlength="1000"
        bind:value={requested}
        disabled={busy}></textarea></label
    ><button disabled={busy}>Record plan and previous observations</button>
  </form>
</section>
{#if error}<p class="inline-error" role="alert">
    {error}
  </p>{/if}{#if plan}<section class="panel">
    <div class="section-heading">
      <h2>{plan.requested}</h2>
      <Status value={plan.status} />
    </div>
    <details>
      <summary>Previous observations</summary><Evidence items={plan.previous} />
    </details>
    <ol>
      {#each plan.steps as step}<li>{step}</li>{/each}
    </ol>
    <button
      onclick={() =>
        controller.run(() => controller.service.settings(plan!.kind))}
      >Open System Settings</button
    >{#if plan.kind === 'power'}<p class="muted small">
        On desktop Macs, open Energy if Battery is unavailable. <a
          class="reference-link"
          href="https://support.apple.com/en-us/101613"
          onclick={(e) => {
            e.preventDefault();
            void controller.run(() =>
              controller.service.reference(
                'https://support.apple.com/en-us/101613',
              ),
            );
          }}
          target="_blank"
          rel="noreferrer">Apple power-mode guidance</a
        >
      </p>{/if}{#if plan.status === 'awaitingManualAction'}<label
        class="check-row"
        ><input type="checkbox" bind:checked={ack} />I performed the described
        manual step myself.</label
      >
      <div class="button-row">
        <button disabled={busy} onclick={() => verify(false)}
          >Decline plan</button
        ><button
          class="primary"
          disabled={busy || !ack}
          onclick={() => verify(true)}>Record new observations</button
        >
      </div>{/if}
    <p>{plan.verification}</p>
    {#if plan.observed.length}<Evidence items={plan.observed} />{/if}
    <p class="muted small">{plan.recovery}</p>
  </section>{/if}
