# Phase 3.1 design handoff

Recorded 2026-10-22 for version 0.6.0. This is an ordinary extension of the existing **Operate / Native macOS clarity** workspace. The direction contract is [phase-3-implementation.md](phase-3-implementation.md); the original Phase 3 specification and Flow H remain intact.

## Inherited system

The implementation retains the grouped sidebar, independent content scrolling, compact system typography, neutral surfaces, thin separators, restrained mint actions, text-qualified statuses and keyboard tabs described in `DESIGN.md`. The approved unboxed Coqui file emblem and wordmark remain unchanged. No raster assets or new visual world were introduced.

`src/features/Firmware.svelte`, `src/components/FirmwareReportView.svelte` and `src/components/FirmwarePlanView.svelte` reuse the shared panel, button, evidence, status, table and history patterns. The added shared styles provide separated audit-choice rows, wrapping action groups, bounded reading width and warning-colored cautions. These are task composition rules, not new global tokens. Runtime typography remains the macOS system stack; the editable Figma handoff uses the established Inter fallback.

`DESIGN.md` and `.impeccable/design.json` were preserved. Their previously documented drift is not repaired by this milestone. `PRODUCT.md` describes macOS desktop first in human-readable terms; the actual runtime is a native Tauri macOS application with a Svelte web UI, regardless of whether tooling recognizes that platform string.

## UX additions

- **Overview** retains the latest audit or earlier scan observations, identifies observation time, explains adapter coverage and preserves explicit uncertainty. A missing optional reading never becomes a healthy result.
- **Advanced audit** offers firmware inventory, configuration audit and macOS update availability as separate choices, unchecked on arrival. Update selection reveals a second per-run network acknowledgement. The audit action stays disabled until the selected scope is valid; update selection and its acknowledgement reset after the run. Actual stages and cancellation use the existing operation flow.
- **Evidence and coverage** retain model/build identity, timestamps, collection status, gaps, findings and source-qualified properties. Offered updates have a response time and immediate-staleness warning. Capability/model coverage remains inspectable, without a latest-firmware or security-certification claim.
- **Guidance** separates software-update review, startup-policy inspection and recovery requirements from collection and cleanup. Plans show cautions, steps, bundled-source review date, current Apple links and a separate acknowledgement. The Settings handoff is labeled **Open System Settings** with General → Software Update guidance. Later local observations remain an unverified manual outcome; no installation, restart, recovery or security mutation is executed.
- Saved audits and plans connect to History and existing diagnostic retention/purge. Ask Coqui remains local with no external AI connection.

## Evidence and disposition

The documenter checked PRODUCT/DESIGN context, the design sidecar, architecture/status/roadmap, the Phase 3.1 contract, the three Svelte views, relevant shared styles/tokens, the Figma ledger and the finish report. The [finish review](../../.impeccable/review/phase3-finish-findings.md) gives **ship within the reviewed scope**, with no outstanding material actionable finding.

That reviewer inspected the supplied native captures in `.impeccable/review/`: `phase3-scope-dark.png`, `phase3-overview-dark.png`, `phase3-scope-1100-dark.png`, `phase3-scope-1100-light.png`, `phase3-guidance-1100-light.png` and the final replacement `phase3-plan-dark.png`. Wide evidence is 1440 logical pixels wide with host-clamped height around 893; minimum evidence is 2200 × 1440 physical pixels, representing 1100 × 720 logical pixels. Captures cover dark/light scope, retained results and scrolled manual guidance. Filename suffixes are not proof of image encoding.

An earlier dark recovery capture showed a conflicting pale Settings selection. The final rebuilt/relaunched replacement shows the correct Firmware & Security selection. The cause remains undiagnosed; this records the final displayed evidence, not a code-fix or complete theme-transition claim.

The parent's completed validation reports `scripts/check.sh` passing with **42 frontend + 96 Rust tests (138 total)**, zero Svelte errors/warnings, production build, Rust formatting and locked native check. The once-only detector reported **31 advisories and zero structural findings**; advisories concern pre-existing/shared CSS before the new firmware styles. This documentation pass did not rerun tests, detector or live QA.

Three linked editable structural Figma views are recorded in `docs/figma-phase-3.json`, with authoring source in `scripts/figma/phase-3.js`. They contain no raster UI and make no pixel-parity or measured-device claim.

## Validation limits

Physical validation is limited to Mac16,8 / Apple M4 Pro / macOS 26.6.1. macOS 14, other models, managed-device behavior, full VoiceOver coverage, repeated appearance transitions and another-Mac Gatekeeper/distribution validation remain follow-up work. Version 0.6.0 is local, ad hoc signed and not notarized. No firmware change, OS installation or real-user-file cleanup was performed for this design review.
