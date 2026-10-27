<script lang="ts">
  import type { createAppStore, Page } from '../state';
  import type { RecommendationPlan } from '../bindings/RecommendationPlan';
  import Evidence from './Evidence.svelte';
  import { date } from '../format';
  let {
    controller,
    plan,
    mode = 'all',
    limit = 100,
  }: {
    controller: ReturnType<typeof createAppStore>;
    plan: RecommendationPlan;
    mode?: 'all' | 'actions' | 'workloads' | 'bottlenecks';
    limit?: number;
  } = $props();
  const destinations: Page[] = [
    'Storage',
    'Optimization',
    'Performance',
    'System Analysis',
    'Maintenance',
    'Settings',
    'Hardware',
  ];
  function review(destination: string, target: string) {
    const page = destinations.find((p) => p === destination);
    if (page) controller.openRecommendation(page, target);
  }
</script>

<p class="muted small">
  {plan.preference} preference · Prepared {date(plan.createdAt)}
</p>
{#if mode === 'all' || mode === 'actions'}
  <h3>Recommended next steps</h3>
  {#each plan.actions.slice(0, limit) as action}
    <article class="review-row">
      <div class="section-heading">
        <h4>{action.title}</h4>
        <span class="subtle-tag">{action.classification}</span>
      </div>
      <p>{action.explanation}</p>
      <p class="small">{action.expectedBenefit}</p>
      <p class="muted small">
        Confidence: {action.confidence} · Relevant to: {action.affectedWorkloads.join(
          ', ',
        ) || 'General review'}
      </p>
      <details>
        <summary>Evidence, risk and verification</summary>
        <dl class="review-facts">
          <dt>Risk</dt>
          <dd>{action.risk}</dd>
          <dt>Reversibility</dt>
          <dd>{action.reversibility}</dd>
          <dt>Permission</dt>
          <dd>{action.permission}</dd>
          <dt>Restart</dt>
          <dd>{action.restart}</dd>
          <dt>Verify</dt>
          <dd>{action.verification}</dd>
        </dl>
        <Evidence items={action.evidence} />
      </details>
      {#if destinations.some((p) => p === action.destination)}<button
          class="text-button"
          onclick={() => review(action.destination, action.id)}
          >Review in {action.destination}</button
        >{/if}
    </article>
  {/each}
{/if}
{#if mode === 'all' || mode === 'workloads'}
  <h3>Workload suitability</h3>
  {#each plan.workloads as workload}
    <article class="review-row">
      <h4>{workload.workload}</h4>
      <strong>{workload.status}</strong>
      <p>{workload.explanation}</p>
      <p class="muted small">Next test: {workload.nextTest}</p>
      <Evidence items={workload.evidence} />
    </article>
  {:else}<p class="muted">
      Choose your workloads in Settings to prepare a suitability review.
    </p>{/each}
{/if}
{#if mode === 'all' || mode === 'bottlenecks'}
  <h3>Bottleneck investigation</h3>
  {#each plan.bottlenecks as finding}
    <article class="review-row">
      <h4>{finding.subsystem} · {finding.workload || 'Selected workload'}</h4>
      <strong>{finding.observation}</strong>
      <p>{finding.possibleImpact}</p>
      <p class="muted small">{finding.confidence}</p>
      <p>Next test: {finding.nextTest}</p>
      <Evidence items={finding.evidence} />
    </article>
  {:else}<p class="muted">
      Select a completed representative baseline to examine CPU and swap
      observations. Missing measurements do not establish that no bottleneck
      exists.
    </p>{/each}
{/if}
<details>
  <summary>Sources and limits of this plan</summary>
  <p class="muted small">{plan.qualification}</p>
</details>

<style>
  .review-row {
    padding-block: 16px;
    border-bottom: 1px solid var(--border);
  }
  .review-row:last-of-type {
    border-bottom: 0;
  }
  h4 {
    font-size: 14px;
    margin-block: 0 8px;
  }
  .review-facts {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 8px 16px;
  }
  dt {
    color: var(--secondary);
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .review-row p {
    max-width: 75ch;
  }
  @media (max-width: 1100px) {
    .review-facts {
      grid-template-columns: 1fr;
      gap: 4px;
    }
    dd {
      margin-bottom: 8px;
    }
  }
</style>
