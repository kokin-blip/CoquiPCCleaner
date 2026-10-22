# Phase 2.4 design handoff — 0.5.0

Recorded 2026-10-22. This is a precisely specified extension of the existing native macOS workspace. The direction contract in [phase-2-remainder.md](phase-2-remainder.md) and supported boundaries in [phase-2-workflows.md](phase-2-workflows.md) govern the new flows. Ask Coqui is a local deterministic guide with retained observations and review handoffs; the user explicitly chose no external AI connection.

## Incumbent system preserved

`DESIGN.md` and `.impeccable/design.json` were read and left unchanged. The implementation continues the existing system font stack, warm neutral light/dark appearances, semantic mint actions, grouped sidebar, flat bordered panels, compact controls, evidence rows, tables and protected-focus dialogs. No material new visual-system rule or token requires a system-file update. The new 15px context heading is a local density adjustment, not a new global type scale.

`src/tokens.css` retains the incumbent palette. In `src/styles.css`, textarea controls now inherit the shared font, input surface/border/radius and secondary placeholder ink; manual guidance links use readable accent ink in both normal and visited states. These changes apply existing field/link semantics. Visible focus and reduced-motion rules remain shared.

Ask uses conversation history, discussion/composer and collapsible evidence/context. At the compact desktop breakpoint, context moves below the discussion. Duplicates extend the existing Optimization tabs with explicit keeper selection, full paths, qualified coverage and disabled analysis-only removal choices. Baselines, manual plans and advice use the existing forms/evidence vocabulary; assumptions, unavailable readings and manual outcomes remain labeled. Maintenance uses an opt-in schedule form, disclosed scope and chronological records. Activity reuses retained-history rows and detail dialogs.

## Evidence and review scope

Checked PRODUCT.md, DESIGN.md, the parsed sidecar, architecture/roadmap/status records, the workflow specification, direction contract and `.impeccable/review/phase2-review-packet.md`. Source sampling covered shared styles/tokens and AskCoqui, Duplicates, Baselines, ManualPlans, Advisor, Maintenance, Resources and ActivityHistory.

Native images inspected in `.impeccable/review/`: `phase2-ask-dark.png`, `phase2-ask-1100-light.png`, `phase2-duplicates-dark.png`, `phase2-maintenance-1100-dark.png`, `phase2-manual-plan-dark.png`, `phase2-advice-outcome-dark.png` and `phase2-history-dark.png`. They support the retained shell/material, local-only wording, minimum-width reflow, qualified duplicate evidence, manual outcome and unavailable advice states. The advice and manual-plan images are scrolled outcomes. The refreshed maintenance image confirms the neutral scope border at compact width; its 2200×1772 pixels represent 1100×886 logical pixels at 2×, rather than exact minimum-height evidence. Images are review points, not current device measurements or exhaustive acceptance evidence.

The saved initial findings in `.impeccable/review/phase2-finish-findings.md` requested the textarea and visited-link corrections. The independent reviewer checked corrected captures and recorded **ship for those two fixes** in `.impeccable/review/phase2-finish-verdict.md`, with no observed fix-introduced regressions. This document does not broaden that targeted follow-up into a fresh whole-surface audit. No new detector, browser session, screenshot or runtime test was run for this documentation pass.

## Assets, Figma and remaining qualifications

No raster was authored for this extension. The selected existing `public/brand/coqui-file-emblem-v1.png` and existing icons are reused. Its original generation mode and prompt are recorded in `public/brand/coqui-alternatives-prompts.txt`. The prior provenance scan still flags pre-existing PNG metadata; repository prompt records do not establish that embedded metadata passes. No provenance was invented and no asset was rewritten.

The review packet reports pre-existing DESIGN/sidecar drift (`CONTEXT_STALE`) and detector advisories. They remain disclosed and unrepaired; preserving the incumbent system here does not certify complete synchronization. Original kit content and unrelated logo work remain outside this handoff.

The editable Figma extension is recorded in `docs/figma-phase-2-remainder.json` and `docs/figma.md`: existing shell/button/styles, illustrative records and established Inter fallback. It is a structural handoff; the native app uses system type. Native captures cover the actual host and available window sizes, without a macOS 14, other-Mac, mobile or pixel-parity claim.
