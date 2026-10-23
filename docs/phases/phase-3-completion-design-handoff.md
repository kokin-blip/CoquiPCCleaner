# Phase 3.2 design handoff

Recorded 2026-10-22 for version 0.7.0. This is an ordinary extension of the existing **Operate / Native macOS clarity** workspace. The direction contract is [phase-3-completion.md](phase-3-completion.md), building on [Phase 3.1](phase-3-implementation.md). The original Phase 3 specification and Flow H remain intact.

## Incumbent-system comparison

The extension preserves the grouped sidebar, independently scrolling content, macOS system typography, neutral light/dark surfaces, thin separators, restrained mint actions, text-qualified statuses and keyboard tabs in `DESIGN.md`. The approved unboxed Coqui file emblem and wordmark remain unchanged. No raster assets, new palette, new global tokens or replacement visual world were introduced. The absence of repeated shield advisories remains intentional.

The two new task tabs reuse the existing panel, field, button, history-row and evidence-table patterns. Comparison selectors and before/after context use two equal columns with a 16px gap. Table cells align at the top, allow long evidence to wrap and sit in the existing horizontal scroll container. Action groups wrap with 8px gaps. Notices use separated text rows; the toolbar reuses the existing 330px-wide, scrollable notification popover, surface fill, border and floating shadow. These are local composition rules rather than additions to the system token vocabulary.

`DESIGN.md` and `.impeccable/design.json` were preserved. Their previously reported drift remains outside this extension. Runtime type remains the macOS system stack; the editable Figma structures retain the established Inter fallback.

## Components and state behavior

- **Firmware navigation:** Overview, Advanced audit and Guidance remain. Compare audits and Updates & reminders add retained-data tasks. Opening an exact-source notice selects its saved audit in Overview, including a repeated request for the same audit from another tab. A reminder without a source opens Advanced audit for the user to choose a scope.
- **Comparison form:** Before and After selectors identify saved records by time, terminal status and shortened ID. Choices include finished complete/partial audits, with page controls and explicit loading, error/retry and empty states. The main action requires two different selections and is disabled during conflicting work. Rust additionally requires strict chronological order and rejects unfinished/cancelled records. Comparing records starts no collection.
- **Comparison evidence:** A shared view shows the qualification, both observation times and model/chip context, followed by Property / comparison, Before and After columns. Each reading retains its value/unit, source, time and qualification; non-supported states keep their status label. Missing scope reads “Not collected in this scope.” Platform mismatch is disclosed. Unknown, unavailable and manual properties stay qualified rather than becoming success indicators. The footer retains source IDs and comparison creation time. Saved comparisons hold selected evidence copies under their own retention date.
- **Comparison meaning:** Changed or unchanged values describe observations. Neither state certifies installation, reboot, recovery, integrity, version ordering, causality or current security. Matching reported model/chip eligibility is not physical-device attestation. A disappearing software offering never establishes installation.
- **Update notices:** The workspace and toolbar show retained evidence, observation time, exact-source navigation and local dismissal. The workspace also names the latest requested attempt and distinguishes recorded evidence from stale/incomplete evidence. Refresh local notices reads local data. A later complete none-offered response suppresses an older offering notice; a later failed/cancelled/unknown attempt preserves the older evidence with uncertainty. Partial, invalid/future-time and 24-hour-old listings are flagged; younger evidence still makes no current-state claim. Dismissal belongs to that exact source, so a later explicitly requested response may produce a new notice. Empty notices do not establish healthy or current status.
- **Optional review reminder:** The checkbox starts unchecked by default, with an integer interval of 1–365 days, a save action, status/error feedback and next-due text when enabled. Due notices appear only in Coqui while open or once after reopening; missed intervals coalesce. Dismissal schedules one future interval. An unchanged saved rule preserves its due time; disabling clears next due. Rust computes the schedule. This is independent from Maintenance and survives diagnostic purge. It requests no macOS notification permission and never starts an audit, utility or Apple request.
- **Manual follow-up:** Guidance can expose “Inspect before and after evidence” for an attached saved comparison. Its existing acknowledgement and user-requested local-observation steps remain separate. The outcome stays unverified; comparison failure or cancelled observation remains disclosed. No writer, installer, reboot, startup-policy change, Revive or Restore control was added.

## Reviewed evidence and disposition

The documenter read the product/design context, direction contract, architecture/status/roadmap, relevant Svelte views/styles, Figma ledger and authoring script, and inspected these six supplied native captures:

| Capture in `.impeccable/review/` | Evidence shown |
| --- | --- |
| `phase3-rest-notices-wide.png` | Dark update notice, saved-response qualification, source/dismiss actions and default-off reminder form. |
| `phase3-rest-comparison-wide.png` | Dark selected audit pair, primary comparison action and start of retained evidence. |
| `phase3-rest-comparison-minimum.png` | Minimum-size dark form, persistent navigation and comparison context below. |
| `phase3-rest-comparison-evidence-minimum.png` | Scrolled minimum-size dark evidence table with unchanged readings, sources, times and qualifications. |
| `phase3-rest-notices-light-minimum.png` | Minimum-size light notice layout, selected navigation/tab and the start of reminder guidance. |
| `phase3-rest-bell-light-minimum.png` | Light toolbar popover with retained notice text and source/dismiss actions. |

Wide captures are 2880 × 1786 physical pixels, representing 1440 × 893 logical pixels; the host clamps the default window height. Minimum captures are 2200 × 1440 physical pixels, representing 1100 × 720 logical pixels. The 1440 × 960 design reference remains in Figma. Purple screen-sharing/cursor treatments are capture tooling, not product styling.

The [fresh finish report](../../.impeccable/review/phase3-rest-finish-findings.md) returned **SHIP within these six captures and the supplied source scope**, with no material actionable findings. Its reported token contrast checks were 6.27:1 dark / 6.10:1 light for secondary panel text, 7.83:1 for the dark primary action and 6.47:1 for light accent text. The supplied once-only detector result was 31 historical shared advisories and zero primary findings, with none attributed to the new CSS. This documentation pass did not rerun context, detector, tests, build or live app interaction. Automated/runtime acceptance belongs to the implementation validation recorded in `docs/status.md`; this handoff does not extend it.

The three linked editable structural Figma views are recorded in `docs/figma-phase-3-completion.json`: saved audit comparison (`180:1441`), local notices/reminders (`180:1544`) and compared manual outcome (`180:1647`) in file `Gz7WEiN8Y3Vvivvae0ZPlP`. They reuse the existing shell, appearance collection and button component, preserve the original source frame, and contain no raster UI. Their content is explicitly illustrative. They are structural handoff evidence, without a native pixel-parity or measured-device claim.

## Source map

| Source | Responsibility |
| --- | --- |
| `src/features/Firmware.svelte` | Task tabs, explicit audit scope and exact-source audit routing. |
| `src/features/FirmwareComparisons.svelte` | Saved-record selectors, pagination, comparison action and saved-comparison list. |
| `src/components/FirmwareComparisonView.svelte` | Reusable qualified before/after evidence table. |
| `src/features/FirmwareUpdates.svelte` | Local notice center, dismissal and reminder form. |
| `src/components/FirmwarePlanView.svelte` | Manual guidance state and attached-comparison inspection. |
| `src/App.svelte`, `src/state.ts` | Toolbar notice entry, source navigation and local refresh lifecycle. |
| `src/styles.css`, `src/tokens.css` | Local composition rules and inherited semantic tokens. |
| `engine/src/firmware_review.rs`, `engine/src/database.rs` | Retained-evidence semantics, chronological validation, schema 5 preferences and Rust-owned dates. |
| `scripts/figma/phase-3-completion.js` | Reproducible editable structural handoff source. |

## Validation limits

The captures show real retained observations at a review point, not a guarantee of present device status. They do not by themselves exercise every stale/failed listing, mismatched platform, missing permission/scope, due reminder, dismissal/reopen, purge/migration or cancelled/manual-outcome state. Those behaviors are specified and implemented separately; source inspection is not runtime proof. The compared-manual-outcome Figma frame is illustrative and has no corresponding new native capture in this six-image set.

Physical validation remains limited to Mac16,8 / Apple M4 Pro / macOS 26.6.1. macOS 14, additional Apple Silicon families, managed/deferred catalogs, full keyboard/VoiceOver workflows, repeated appearance transitions and another-Mac Gatekeeper/distribution behavior require separate validation. No firmware integrity or authoritative latest-version mapping is available. This review required no fresh network listing, installation, reboot, recovery operation or real-user-file cleanup. Ask Coqui remains local with no external AI connection; Windows and Intel specifications remain deferred and intact.
