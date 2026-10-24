# CoquiPCCleaner

<!-- impeccable:product-schema 1 -->

## Platform
macOS desktop first; Windows deferred until the first completed macOS build.

## Stack
Tauri 2, Svelte/TypeScript, Vite; Rust-owned SQLite. User selected Tauri + Svelte + Vite during implementation.

## Users
People investigating slow computers, limited storage, and workload constraints, including developers with large tool caches. Both ordinary and technical users need understandable evidence and deeper inspection.

## Product Purpose
Understand the machine, diagnose observed problems, explain options, obtain consent, and verify outcomes. A useful result can be that no changes are warranted.

## Positioning
Workload-aware analysis and safe developer-storage maintenance grounded in real measurements rather than manufactured health scores.

## Operating Context
The whole-disk/removal extension is 0.8.0 (see docs/phases/whole-disk-cleanup.md). Mounted-scope analysis, storage/application review, recoverable removal and the default-off administrator service are implemented. Native disposable root-owned Trash/restore, short-session authorization reuse, cancellation, restart, final ordinary installation and fixture teardown passed on the local Mac. Validation does not establish comprehensive disk access or acceptance on every supported Mac; retained limits are in docs/status.md. The prior completed milestone is Phase 3.2 (0.7.0): Apple Silicon firmware/security audits, consented update listings, manual guidance, saved-audit comparisons and local notices/reminders; see docs/phases/phase-3-completion.md and phase-3-implementation.md. Phase 2, milestone 2.4 (0.5.0) added local investigations, exact duplicates with keeper-preserving approval, measured comparisons, manual settings plans, workload scenarios, maintenance and broader resource review, building on verified cache/project-output cleanup on Apple Silicon macOS 14+. Whole internal disk is the default for user-initiated analysis, with a mounted-volume review before collection. Standard home-folder and selected-folder scopes remain choices. Monitoring runs only while the app is open. Local direct-download builds require no Developer ID account or notarization.

## Capabilities and Constraints
Phase 1 is read-only: inventory, live CPU/memory monitoring, storage exploration, cache candidates, startup detection, available disk health and platform/security information. Protected and unavailable data must be disclosed. Phase 2.1 adds separately approved native Trash moves for individual eligible user-cache files, persistent action records, verification and manual recovery guidance. Phase 2.2 adds developer-output review with Git/worktree evidence. Phase 2.3 adds separate exact-folder approval, fresh descendant/Git checks and verified native Trash for the supported outputs in docs/phases/project-cleanup.md; dependencies, environments and unassociated Xcode data remain analysis only. Phase 2.4 adds local guided Ask Coqui, bounded exact-copy/PNG-JPEG similarity analysis, separately approved personal-copy Trash moves with fresh keeper verification, baselines/comparisons, recorded manual startup/power plans, assumption-based workload advice, app-open scan/reminder maintenance, qualified leftovers/clone relationships and retained activity. See docs/phases/phase-2-workflows.md for actual limits. Optional external AI, native-manager resource execution, automatic settings changes and closed-app scheduling remain explicit extensions. Phase 3.1 adds a separate read-only audit module and retained Apple guidance plans. No firmware writer, automatic installation, restart, Recovery operation or security-policy mutation is exposed. macOS update listings contact Apple only with per-run opt-in; saved listings have explicit timestamps and no latest-firmware verdict. Phase 3.2 adds chronological saved-audit comparisons without version ordering or operation-success claims, evidence-backed in-app notices, and a disabled-by-default app-open review schedule. Reminders never collect or contact Apple; each update check still requires per-run consent. No cloud AI, telemetry, login, or synchronization.

## Brand Commitments
Native macOS composition using the supplied Apple Music macOS Figma kit and its other application families, adapted to Coqui semantics. Preserve original kit content and add Coqui pages.

## Evidence on Hand
Original brief: docs/original-brief.md. Figma file: Gz7WEiN8Y3Vvivvae0ZPlP. Empty repository at implementation start. Runtime values must come from the engine; design specimens must be labeled illustrative.

## Product Principles
- Observations and hypotheses are different.
- Missing information is never a healthy reading.
- Workload relevance outranks headline specifications.
- Consent, scope, and recoverability must be explicit.
- Local data stays under the user's control.

## Accessibility & Inclusion
Keyboard navigation, visible focus, VoiceOver labels, readable contrast, reduced motion, accessible alternatives to visual charts, and resizable desktop layouts.

## Whole-disk and application extension
Whole internal disk requires a reviewed mounted-volume scope. Traversal completion and comprehensive access remain distinct. Storage selection and Applications use exact-path, expiring Trash approval and per-item verification, with separately reviewed no-overwrite restore. Shared/unconfirmed data and system/project protections remain qualified. The optional administrator helper is not a route around TCC or SIP and requires a matching installed build plus separate native operation authorization. See docs/phases/administrator-helper.md for independent validation and retained limitations. See docs/reference-feature-map.md for reference evidence and capability gaps.

## Folder-first storage — 0.9.0
The user confirmed a folder treemap beside ranked contents, with Large files separate. Cumulative folder allocation and small-file counts expose substantial footprints even when no single file is large. Allocated/logical switching, bounded maps with explicit remainder, paginated contents and contextual purpose/consequence explanations support inspection. Recognition never grants removal authority. Schema 7 retains old history with unavailable legacy counts. See docs/phases/storage-map.md and docs/research/disk-inventory-x.md. The prior installed 0.8 whole-disk traversal ended partial during finalization; the 0.9 concurrency/cancellation fixes require a new full-scale native run before that limitation can be closed.

The 0.9.0 matching app/helper are installed and authenticated locally. Public schema-7 migration preserved the earlier history/settings and removal records; a 515-file disposable native scan, exact aggregate/omission accounting, keyboard navigation, pagination and restart passed. This does not establish elevated large-index import or a completed whole-disk run; see docs/status.md.
