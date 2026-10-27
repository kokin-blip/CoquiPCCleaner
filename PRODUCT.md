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

## Original-plan completion batch — 0.10.0

The installed release fixes onboarding's whole-disk review handoff and adds shared workload/recommendation review, Dashboard question submission, explicit fresh local Ask tasks and bounded PNG/JPEG previews. Saved preferences order reviews, not system mutations. Missing measurements, unrelated baselines and unverified causes remain explicit. The final matching app/helper authentication and focused context-refresh capture/reopen passed locally; earlier scoped native checks cover saving, cancellation, previews and restart. See docs/phases/original-plan-completion.md for actual scope and remaining work. Full-scale scanning and broader hardware/accessibility acceptance remain separate from these checks.

The 0.11 continuation adds user-operated build/test timing and repeated-trial comparisons. The user performs the task and reports completion; measured intervals include human timing latency. Available activity is evidence of utilization, not a confirmed bottleneck. Explicitly selected results can inform the matching workload review; comparison medians/ranges do not prove causality or an upgrade benefit. See docs/phases/task-trials.md and docs/status.md for capture bounds and actual release/native acceptance.

## Whole-disk reliability checkpoint — 0.11.2/0.11.3

The installed 0.11.3 helper's FDA entry was refreshed with specific user consent. A fresh metadata-only scan completed all eight mounted internal roots, aggregation and full public import for **3,224,312 files**. Public count/byte/root totals matched exactly; administrator-off restart, folder exploration, pagination and coverage passed. Desktop, Documents and Downloads now have indexed descendants and no permission-denied gaps.

Permission denials fell from **652 to 26**; remaining denials, deliberate link/identity/cloud/mount exclusions and live-filesystem races keep the result partial. This is completed readable-scope acceptance on this Mac, not every-byte access or guaranteed recoverable APFS space. Folder queries use the existing parent index and remain responsive at full scale. Administrator mode is off; the approved persistent helper FDA grant remains enabled. No inspected files were changed or removed. See docs/status.md for exact source/native checks and retained hardware/accessibility/security limitations.


## Development-storage evidence — 0.12.0

Compact, expandable Storage and Developer artifacts indicators distinguish recognized debug/build patterns, dependencies, Git history, indexed projects and common agent worktree namespaces. Dated local Git checks show unique changes, HEAD commit age and cached upstream differences. Cached matches do not verify current GitHub uploads or whole-folder backups; older commits do not establish abandonment. Repository/worktree reviews stay Protected, and generated paths retain Review before deleting. Removal still requires its existing separate fresh inspection and approval. Coverage can isolate permission denials with location guidance. All 26 remaining denials in the reviewed 0.11.3 scan are system/service locations; no safe ordinary permission repair was found. See docs/phases/development-indicators.md and docs/status.md for evidence and actual acceptance.

## Simpler scan and review — source preview

The user approved a CleanMyMac-style scan-first structure while retaining Coqui's mint/frog identity. Five primary destinations lead to focused reviews; developer files and build/test trials remain secondary tools, with hardware, firmware/security and workload advice under More tools. Starting a scan and reopening saved results are distinct actions. Recorded category totals retain their dates, partial coverage and unavailable measurements, and never represent cleanup approval or guaranteed reclaimed space. Removal selections remain empty. This first functional batch is a local source preview; installed 0.12.0 is unchanged, and full feature parity is not claimed. See docs/phases/cleanmymac-workflow.md for the implementation boundary and remaining work.

The user also selected opt-in app-update discovery with installation through the App Store/vendor and guided manual system maintenance. Applications → Updates reviews at most ten explicit apps and their disclosed sources before a per-run network acknowledgement. Dated saved results retain unavailable/uncomparable/partial/cancelled states; local browsing/history starts no update request or application-inventory refresh. A listed offer opens the reviewed vendor tool; Coqui downloads and installs no updates. Ask Coqui remains local with no external AI connection. See docs/phases/application-updates.md for protocol, source, consent and validation limits.

The next source preview adds per-run system/startup/application metadata choices alongside the existing storage scope review. Each check has independent dates, omissions and nullable results; saved views never fill a skipped check from older inventory. Foreground observations run as the user after traversal/import, and content/network tools remain separate. The selected check list appears in Overview and History; Startup reads the matching report. Native isolated-database validation is separate from installed administrator acceptance. See docs/phases/selected-scan-checks.md.

## Downloads exact-copy review — source preview

Cleanup → Duplicates offers Downloads only or all indexed locations before bounded content reads, with similar-photo analysis separately unchecked. Saved exact groups can be searched, sorted and filtered by last-modified age while keeping all copies available for explicit keeper review. Location/age never establish browser origin, disuse, backup or safe deletion; all removal selections start empty. Legacy scope remains unrecorded and installed-release acceptance is separate. See docs/phases/downloads-duplicates.md and docs/status.md.

## Broader cleanup categories — source preview

Cleanup now opens a dated category view for app/Xcode caches, user logs, DerivedData, archives, device support, simulator device data, local device backups, Trash and shared/system locations. File-only indexed footprints remain distinct from removable space; unknown/excluded locations stay unavailable. Exact paths are paginated. User/Xcode caches can select individual files for the existing independent fresh review, starting empty. DerivedData opens existing protected Developer review; other categories offer manual guidance or protected inspection without removal actions. No Trash emptying or automatic permission repair is added. See docs/phases/cleanup-categories.md and docs/status.md for scope, provenance and uninstalled-preview acceptance.

## 0.13.0 macOS release checkpoint

The user authorized committing/pushing the accumulated work and publishing it as the latest macOS release. Version 0.13.0 packages the continuation above for Apple Silicon/macOS 14+, with a portable application ZIP and a separately optional matching app/helper installer. Both are ad hoc signed and unnotarized; package/signature checks and ordinary bundle startup are separate from installed privileged-operation acceptance. The development Mac's installed 0.12.0 pair and privacy grants remain unchanged. Source tests and release evidence are in docs/status.md; installation notes and retained limits are in docs/releases/0.13.0.md. No Windows/Intel binary or claim of full reference-product parity is added.
