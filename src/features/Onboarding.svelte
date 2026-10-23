<script lang="ts">
  import {
    ArrowRight,
    Check,
    ShieldCheck,
    Activity,
    Monitor,
    Leaf,
    Zap,
  } from '@lucide/svelte';
  import { workloads } from '../format';
  import type { createAppStore } from '../state';
  let {
    controller,
    onScan = () => void controller.scan(),
  }: { controller: ReturnType<typeof createAppStore>; onScan?: () => void } =
    $props();
  let step = $state(0);
  let selected = $state<string[]>([]);
  let custom = $state('');
  let preference = $state('Balanced');
  let mode = $state('standard');
  let monitor = $state(true);
  let saving = $state(false);
  const steps = [
    'Welcome',
    'Your workload',
    'Your preference',
    'Privacy & scope',
  ];
  async function finish(scan: boolean) {
    if (!$controller.settings) return;
    saving = true;
    await controller.run(async () => {
      await controller.save({
        ...$controller.settings!,
        onboardingComplete: true,
        workload: {
          workloads: [...selected, ...(custom.trim() ? [custom.trim()] : [])],
          preference,
        },
        scanMode: mode,
        monitoringEnabled: monitor,
      });
      if (scan) onScan();
    });
    saving = false;
  }
</script>

<main class="onboarding">
  <div class="onboard-chrome" data-tauri-drag-region></div>
  <div class="onboard-card">
    <div class="wordmark">
      <img
        class="brand-icon"
        src="/brand/coqui-file-emblem-v1.png"
        alt=""
      />coqui<span class="wordmark-detail">PC Cleaner</span>
    </div>
    <div class="onboard-steps" aria-label="Setup progress">
      {#each steps as name, i}<button
          class:current={step === i}
          disabled={i > step}
          onclick={() => (step = i)}
          aria-current={step === i ? 'step' : undefined}
          ><span>{i < step ? '✓' : i + 1}</span>{name}</button
        >{/each}
    </div>
    {#if step === 0}
      <div class="welcome-symbol">
        <Monitor size={62} strokeWidth={1.15} /><span><Check size={20} /></span>
      </div>
      <h1>Get to know your Mac.</h1>
      <p class="onboard-intro">
        Understand what’s using its resources, where your storage goes, and what
        deserves a closer look.
      </p>
      <div class="welcome-points">
        <div>
          <Activity size={20} /><strong>Real measurements</strong>
          <p>No invented health scores or unsupported promises.</p>
        </div>
        <div>
          <ShieldCheck size={20} /><strong>Look first. Decide later.</strong>
          <p>
            Analysis does not change your files or settings. Any cleanup
            requires a separate exact-path review and approval.
          </p>
        </div>
      </div>
    {:else if step === 1}
      <h1>What do you use your Mac for?</h1>
      <p class="muted">Choose what fits your day. You can edit this later.</p>
      <div class="workload-grid">
        {#each workloads as item}<label
            class:selected={selected.includes(item)}
            class="choice"
            ><input type="checkbox" value={item} bind:group={selected} /><span
              >{item}</span
            ></label
          >{/each}
      </div>
      {#if selected.includes('Other / custom')}<label class="field-label"
          >Describe your workload<input
            bind:value={custom}
            placeholder="For example, 3D animation and local AI models"
          /></label
        >{/if}
    {:else if step === 2}
      <h1>Choose your preference.</h1>
      <p class="muted">
        This helps organize future recommendations. It changes no system
        settings.
      </p>
      <div class="profile-options">
        {#each [{ name: 'Balanced', icon: Monitor, copy: 'Stability, responsiveness, and reasonable energy use.' }, { name: 'Performance', icon: Zap, copy: 'Sustained performance through supported system controls.' }, { name: 'Efficiency', icon: Leaf, copy: 'Battery life, lower energy use, and less unnecessary activity.' }] as p}<label
            class:selected={preference === p.name}
            class="profile-choice"
            ><input
              type="radio"
              name="preference"
              value={p.name}
              bind:group={preference}
            /><p.icon size={23} />
            <div>
              <strong>{p.name}</strong>
              <p>{p.copy}</p>
            </div></label
          >{/each}
      </div>
    {:else}
      <h1>Your Mac. Your boundaries.</h1>
      <p class="muted">
        Diagnostics and file metadata stay in local SQLite storage. No file
        contents, telemetry, or cloud AI are sent anywhere.
      </p>
      <label class="field-label"
        >Storage analysis scope<select bind:value={mode}
          ><option value="standard">Standard — accessible home folder</option
          ><option value="deep">Whole internal disk</option></select
        ></label
      >
      <div class="callout">
        <ShieldCheck size={20} />
        <div>
          <strong
            >{mode === 'deep'
              ? 'Whole internal disk analysis reviews mounted internal volumes.'
              : 'Protected folders may be skipped.'}</strong
          >
          <p>
            External and network volumes require separate folder selection.
            macOS protections stay in place. Cloud-only files and linked
            locations are skipped.
          </p>
          <button
            class="text-button"
            onclick={() =>
              controller.run(() => controller.service.settings('permissions'))}
            >Review Full Disk Access in System Settings <ArrowRight
              size={13}
            /></button
          >
        </div>
      </div>
      <label class="switch-row"
        ><div>
          <strong>Live monitoring while Coqui is open</strong>
          <p class="muted">
            Record CPU, memory, swap, and readable process activity every two
            seconds. No background agent.
          </p>
        </div>
        <input type="checkbox" role="switch" bind:checked={monitor} /></label
      >
      <p class="muted small">
        Scan history is kept for 30 days by default. You can change retention,
        exclude folders, or remove diagnostic data in Settings.
      </p>
    {/if}
    <footer class="onboard-footer">
      {#if step > 0}<button onclick={() => step--}>Back</button>{:else}<button
          class="text-button"
          onclick={() => finish(false)}
          disabled={saving}>Explore without scanning</button
        >{/if}<span></span>{#if step < 3}<button
          class="primary"
          onclick={() => step++}
          disabled={step === 1 && selected.length === 0}
        >
          {step === 0 ? 'Set up my computer' : 'Continue'}
          <ArrowRight size={16} /></button
        >{:else}<button onclick={() => finish(false)} disabled={saving}
          >Explore first</button
        ><button class="primary" onclick={() => finish(true)} disabled={saving}
          >{saving ? 'Preparing…' : 'Start read-only analysis'}<ArrowRight
            size={16}
          /></button
        >{/if}
    </footer>
  </div>
</main>
