# Measured task results — design handoff

The 0.11 task-result flow is an ordinary extension of Coqui's established **Native macOS clarity** system. The user selected **Software-development builds/tests** for this original-plan continuation. The [task-trial contract](task-trials.md) supplies the Operate direction: describe a repeatable protocol, wait for Running, perform the task, report completion, inspect the interval, repeat and review two selected groups. No new world, design token or shipping raster was introduced.

## Sources and system comparison

Checked [PRODUCT.md](../../PRODUCT.md), [DESIGN.md](../../DESIGN.md), [.impeccable/design.json](../../.impeccable/design.json), the phase direction contract, [TaskTrials.svelte](../../src/features/TaskTrials.svelte), [WorkloadTrialView.svelte](../../src/components/WorkloadTrialView.svelte), [TrialComparisonView.svelte](../../src/components/TrialComparisonView.svelte), [tokens.css](../../src/tokens.css) and the shared [styles.css](../../src/styles.css). The existing product, design-system and sidecar files are preserved; no system change was approved.

| System commitment | Observed extension |
| --- | --- |
| Warm charcoal and soft neutral appearances; mint for actions and selection | The protocol, result and comparison reuse semantic theme variables through existing panels, fields, buttons, tabs and statuses. No new palette is defined. |
| Compact system typography and text-first evidence | The flow inherits the macOS system stack, 13px body, 17px section titles and shared compact table typography. Trial facts use tabular figures. Units, declared targets, timestamps and qualifications remain text. |
| Flat bordered workspace | Existing surface panels, thin separators and restrained control radii carry the hierarchy. The extension adds no shadows, material effects or decorative illustration. |
| Resizable desktop layout and bounded tables | The two-column protocol follows the shared form grid. Saved trials, activity samples and comparisons use the established horizontally scrolling table container. Long task, protocol and comparison values wrap within the result panels; saved-trial titles wrap without hiding selection controls. |
| Semantic Color, Evidence Before Decoration and Flat Workspace rules | A readable state accompanies color. Explicit completion is separate from retained activity; incomplete states make no completion or target claim. Results retain the user-reported completion and human-latency caveats. |
| Shared interaction finish | Existing visible focus, placeholder ink, selection, disabled controls, scrollbars and reduced-motion behavior apply. The extension introduces no competing motion system. |

The first viewport retains the Performance heading and tabs, labeled protocol fields, ten-minute qualification and visible **Start task timer** action. The result follows the form, then saved trials and the comparison review. The source keeps preparation distinct from Running, exposes explicit finish, allows protocol reuse and makes before/after membership mutually exclusive. Comparison preparation and saving remain separate actions. The shared app operation state and banner carry finish/cancel across navigation; the supplied screenshots do not independently validate that native lifecycle.

## Review evidence and finish scope

The [original finish review](../../.impeccable/review/trials-finish-review.md) found one material containment issue: the three new components used an unstyled table wrapper, and long unbroken user text lacked wrapping. The implementation now reuses `table-scroll` and adds scoped wrapping to trial titles, trial details and comparison content. The [fix verdict](../../.impeccable/review/trials-finish-verdict.md) returns **ship**, scoped to that resolved containment fix, with no remaining findings in that batch.

A fresh delegated reviewer used the degraded finish-reviewer reference because the harness exposes only generic task agents. Its ship decision covers the containment fix and supplied illustrative capture/source scope. This documentation pass likewise used the degraded documenter reference through a generic delegated task agent.

| Capture | Evidence supplied |
| --- | --- |
| [trials-dark.png](../../.impeccable/review/trials-dark.png) | 1280×720 dark first viewport: native shell, protocol, limit and primary action. |
| [trials-1100-light.png](../../.impeccable/review/trials-1100-light.png) | 1100×720 light first viewport: readable two-column form and visible primary action. |
| [trials-result-1100-light.png](../../.impeccable/review/trials-result-1100-light.png) | Scrolled complete-result specimen, own-target qualification, protocol disclosure and saved-trial controls. |
| [trials-comparison-dark.png](../../.impeccable/review/trials-comparison-dark.png) | Scrolled dark comparison: selected counts, separate Compare/Save actions, median/range/own-target table and descriptive-change limit. |
| [trials-long-result-1100-light.png](../../.impeccable/review/trials-long-result-1100-light.png) | Minimum-width long task/project/tool/conditions content wraps inside the expanded result panel. |
| [trials-long-table-1100-light.png](../../.impeccable/review/trials-long-table-1100-light.png) | Long saved-trial titles wrap while State, Before and After controls remain visible. |
| [trials-long-comparison-1100-light.png](../../.impeccable/review/trials-long-comparison-1100-light.png) | Long task/change text wraps; the comparison table and Compare/Save actions remain contained and reachable. |

All seven captures were opened for this documentation pass. Their visible illustrative-data labels are retained. They are review artifacts, not shipping raster assets or machine measurements. The existing approved brand artwork remains unchanged, and this extension requires no new shipping-raster provenance entry.

The captures establish the inspected visual states only. They do not establish native timing, task-output correctness, an automated benchmark, causality, hardware lifespan or upgrade benefit, comprehensive disk access, full VoiceOver review, additional-device acceptance or completion of the original plan. Actual checks, installed-build acceptance and remaining limitations belong in [status](../status.md).

## Preserved drift

DESIGN.md is newer than the sidecar: it includes the 0.10 original-plan extension narrative, while the sidecar's provenance remains centered on the earlier administrator extension and its narrative omits that addition. The sidecar's recorded generation time is `2026-10-23T23:30:48.103676+00:00`. This pre-existing freshness drift is disclosed without regenerating either file.

The sidecar also records existing weight mismatches: primary-button frontmatter inherits body weight 400 while runtime and its preview use 600; status frontmatter inherits caption weight 400 while runtime and its preview use 500. These are not new task-trial tokens or rules. They were not canonized or repaired because this is an ordinary extension and no design-system refresh was approved. No unresolved containment defect from the reviewed fix batch is promoted into the system.
