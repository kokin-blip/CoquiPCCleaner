<script lang="ts">
  import { onMount } from 'svelte';
  import type { createAppStore } from '../state';
  import type { Page } from '../state';
  import type { Investigation } from '../bindings/Investigation';
  import type { ActivityRecord } from '../bindings/ActivityRecord';
  import Evidence from '../components/Evidence.svelte';
  import Status from '../components/Status.svelte';
  import { date } from '../format';
  let { controller }: { controller: ReturnType<typeof createAppStore> } =
    $props();
  let current = $state<Investigation | null>(null),
    history = $state<ActivityRecord[]>([]),
    text = $state(''),
    error = $state(''),
    loading = $state(false),
    contextOpen = $state(true),
    offset = $state(0);
  let busy = $derived(
    $controller.workRunning ||
      $controller.scanning ||
      $controller.cleanupRunning ||
      $controller.cleanupPreparing ||
      $controller.developerRunning,
  );
  onMount(() => {
    void refresh();
  });
  async function refresh() {
    loading = true;
    try {
      history = await controller.service.records('investigation', offset);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  async function select(id: string) {
    error = '';
    try {
      const record = await controller.service.record('investigation', id);
      if (record.kind === 'investigation') current = record.data;
    } catch (e) {
      error = String(e);
    }
  }
  async function ask() {
    const question = text.trim();
    if (!question) return;
    error = '';
    try {
      current = await controller.work('Ask Coqui investigation', () =>
        controller.service.ask(
          current?.id ?? null,
          $controller.session?.id ?? null,
          question,
        ),
      );
      text = '';
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-heading">
  <div>
    <h1>Ask Coqui</h1>
    <p class="muted">
      A local, evidence-based guide to your Mac. No external AI connection.
    </p>
  </div>
  <button
    aria-expanded={contextOpen}
    onclick={() => (contextOpen = !contextOpen)}
    >{contextOpen ? 'Hide context' : 'Show context'}</button
  >
</div>
<div class="investigation-layout" class:context-open={contextOpen}>
  <aside class="investigation-history" aria-label="Investigation history">
    <div class="section-heading">
      <h2>Conversations</h2>
      <button
        disabled={busy}
        onclick={() => {
          current = null;
          text = '';
          error = '';
        }}>New</button
      >
    </div>
    {#if loading}<p role="status">Loading…</p>{/if}{#each history as row}<button
        class:active={current?.id === row.id}
        class="conversation-row"
        onclick={() => select(row.id)}
        ><strong>{row.title}</strong><span class="muted small"
          >{date(row.createdAt)}</span
        ></button
      >{:else}{#if !loading}<p class="muted small">
          Investigations are retained locally using your history retention
          setting.
        </p>{/if}{/each}
    <div class="button-row">
      <button
        disabled={offset === 0 || loading}
        onclick={() => {
          offset = Math.max(0, offset - 50);
          void refresh();
        }}>Previous</button
      ><button
        disabled={history.length < 50 || loading}
        onclick={() => {
          offset += 50;
          void refresh();
        }}>Next</button
      >
    </div>
  </aside>
  <section class="discussion" aria-label="Conversation">
    <div class="messages" aria-live="polite">
      {#if current}{#each current.messages as message}<article
            class="message"
            class:user-message={message.role === 'user'}
          >
            <strong
              >{message.role === 'user' ? 'You' : 'Coqui · local guide'}</strong
            >
            <p>{message.text}</p>
            <span class="muted small">{date(message.createdAt)}</span>
          </article>{/each}{:else}<div class="ask-introduction">
          <h2>What would you like to understand?</h2>
          <p class="muted">
            Describe a repeatable issue or choose a starting point. Coqui cites
            saved observations, asks for missing context, and hands actions to
            their review flow.
          </p>
          {#each ['My Mac is slow during a build', 'I need to review storage and duplicates', 'Startup is taking longer'] as prompt}<button
              disabled={busy}
              onclick={() => (text = prompt)}>{prompt}</button
            >{/each}
        </div>{/if}
    </div>
    <form
      class="ask-composer"
      onsubmit={(e) => {
        e.preventDefault();
        void ask();
      }}
    >
      <label for="coqui-question">Your question</label><textarea
        id="coqui-question"
        required
        maxlength="4000"
        rows="3"
        bind:value={text}
        disabled={busy}
        placeholder="Which app, which task, and what did you observe?"
      ></textarea>
      <div class="section-heading">
        <span class="muted small"
          >Uses retained evidence; no arbitrary commands or automatic actions.</span
        ><button class="primary" disabled={busy || !text.trim()}
          >Investigate</button
        >
      </div>
    </form>
    {#if error}<div class="inline-error" role="alert">
        {error}<button onclick={refresh}>Reload history</button>
      </div>{/if}
  </section>
  {#if contextOpen}<aside
      class="investigation-context"
      aria-label="Diagnostic context"
    >
      <h2>Context and tasks</h2>
      <p class="muted small">
        {current?.qualification ??
          'Choose or run a scan first. Evidence stays tied to its original observation time and scope.'}
      </p>
      <p class="small">
        Scan: {current?.sessionId ?? $controller.session?.id ?? 'None selected'}
      </p>
      {#if current}<Status
          value={current.status}
        />{#each current.tasks as task}<section>
            <h3>{task.name.replace(/([A-Z])/g, ' $1').trim()}</h3>
            <Status value={task.status} />
            <p class="muted small">{task.detail}</p>
            <Evidence items={task.evidence} />
          </section>{/each}{#each current.findings as finding}<section>
            <h3>{finding.title}</h3>
            <p>{finding.explanation}</p>
            <span class="muted small">{finding.confidence}</span><Evidence
              items={finding.evidence}
            />
          </section>{/each}{#if current.actions.length}<h3>
            Review next steps
          </h3>
          {#each current.actions as action}<button
              class="context-action"
              onclick={() => controller.navigate(action as Page)}
              >Open {action}</button
            >{/each}{/if}{/if}
    </aside>{/if}
</div>
