# Measured task results — 0.11.0

The user requested continued completion of the original plan and selected **Software-development builds/tests** as the first task focus. This batch extends Performance with a user-operated task timer and repeated-trial comparisons. It closes the missing actual task-interval measurement within the bounds below; automated application benchmarks, independent compatibility/pricing and hardware-life forecasts remain separate work.

## Measurement contract

The user describes a workload, repeatable task, inputs/project/revision, application/tool version, comparison conditions and positive target duration. Preparation finishes before Running appears. The user starts the task themselves and explicitly reports completion. Coqui executes no project command or application task. A Rust monotonic clock measures the interval through receipt of that completion request, including human start/finish latency. Each capture is capped at 600 seconds while the app is open. Target values can describe longer expectations, but this capture cannot complete a task lasting ten minutes or more.

Available two-second CPU/memory activity and a fixed read-only power-profile observation accompany the interval. They do not establish memory pressure, causality, task correctness or an automated benchmark score. Running observations are retained about once per second. Explicit cancellation, channel loss, timeout, collection/storage failure and app-exit interruption never establish completion or a met target. Reopening an interrupted record preserves its last interval without inventing a finish time. One active foreground operation is allowed; finish uses the matching active trial ID and accepts one completion report. The timer and finish control survive navigation through shared app state and the global work banner.

## Repeated trials and retained evidence

Select one to twenty completed trials per group. IDs must be unique across both groups. Workload, task and inputs must match exactly. Application/version, declared conditions and target differences remain visible qualifications. Each group shows count, median, range and count meeting each trial's own target. Fewer than three trials per group prompts repetition. Median change is descriptive and does not prove an improvement caused by the declared change. Conditions are declarations, not verified controls.

Preparing a comparison writes nothing. Saving is a separate action that validates the retained inputs again and stores the complete comparison, including source IDs. Trial and comparison records reopen through History and use existing retention and diagnostic-data removal. Schema 7 is unchanged. Older recommendation records deserialize with a null trial link; older advice requests can omit the additive trial ID, and existing advice reports remain readable. Recommendation and Advisor selectors accept only an explicitly chosen completed trial; advice requires a matching workload and evaluates the trial's recorded target, not a later expectation. Other workloads retain their missing-measurement state.

## Direction contract

THESIS: Extend the established Performance workspace with a repeatable task protocol, a clear Running/finish boundary and cautious before/after review. This is an Operate flow.

OWN-WORLD: Preserve the existing system type, charcoal and warm neutral appearances, mint primary actions, bordered panels, native fields, tables and shared visible-focus rules. No new visual-system tokens or raster assets.

STORY: Describe one build/test, wait for Running, perform it, report completion, inspect the actual interval, repeat, select two groups and save only when wanted. Missing and incomplete measurements stay explicit.

FIRST VIEWPORT: Existing shell, Performance heading and tabs, then a two-column labeled protocol form with the ten-minute cap and Start task timer. Results, saved trials and comparison review follow in reading order. The shared operation banner keeps finish and cancel available on other pages.

FORM: A local extension of the existing Performance composition, scoped by the user's build/test answer; no new world or concept seed applies. Signature interaction is explicit completion followed by reusable protocol and mutually exclusive comparison selection. Existing restrained/reduced-motion behavior remains.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

## Validation

Results and remaining native/distribution limits are recorded in [status](../status.md). Illustrative UI captures do not establish native timing, output correctness, whole-disk acceptance or broad accessibility/device coverage.

Installed 0.11 passed exact app/helper pairing and scoped native checks on 2026-10-23: two actual user-operated test intervals (met/exceeded targets), navigation-safe completion, an incomplete cancellation, pure comparison versus explicit save, selected trial evidence in recommendation/advice, and retained record reopen after quit/relaunch. The comparison used one trial per group and preserved its small-sample and differing-target limitations. These validation intervals include agent/UI latency and establish no performance improvement or automated benchmark. Administrator features stayed off; existing preferences and files were preserved.
