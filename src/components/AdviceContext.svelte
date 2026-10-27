<script lang="ts">
  import type { AdviceReport } from '../bindings/AdviceReport';
  import type { createAppStore } from '../state';
  let {
    report,
    controller,
  }: { report: AdviceReport; controller: ReturnType<typeof createAppStore> } =
    $props();
</script>

<section
  class="advice-context"
  aria-label="Advice sources and verification limits"
>
  <h3>What this assessment does not verify</h3>
  {#if report.verificationLimits?.length}<ul>
      {#each report.verificationLimits as limit}<li>{limit}</li>{/each}
    </ul>{:else}<p class="muted">
      Structured verification limits were not retained in this older report. Its
      links are reference material, not proof of current compatibility, prices
      or device support.
    </p>{/if}

  <h3>Vendor sources and review dates</h3>
  <p class="muted small">
    Review dates refer to reference content checked during development. Coqui
    does not fetch these pages or recheck compatibility when you create an
    assessment. A report’s creation time is not a source review date.
  </p>
  <div class="source-links">
    {#each report.sources as href}
      {@const source = report.sourceReviews?.find((item) => item.url === href)}
      <div>
        <a
          {href}
          onclick={(e) => {
            e.preventDefault();
            void controller.run(() => controller.service.reference(href));
          }}
          target="_blank"
          rel="noreferrer">{source?.title || href}</a
        >
        <p class="muted small">
          {source?.publisher || 'Publisher not recorded'} · {source?.reviewedOn
            ? `Reviewed for this build: ${source.reviewedOn}`
            : 'Review date not recorded'}
        </p>
        {#if source?.scope}<p class="small">{source.scope}</p>{/if}
      </div>
    {:else}<p class="muted">
        No vendor sources were retained in this report.
      </p>{/each}
  </div>
</section>

<style>
  .advice-context {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  li + li {
    margin-top: 8px;
  }
  .source-links p {
    margin: 4px 0 0;
  }
</style>
