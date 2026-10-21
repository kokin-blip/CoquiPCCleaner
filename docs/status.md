# Delivery and validation

Current build: **0.2.0, Phase 2 milestone 2.1 — cleanup approval and verification.** See [Phase 2 progress](phases/phase-2-progress.md) for the supported action, tests and remaining roadmap. The Phase 1 record below is retained as implementation history.

## Phase 1 baseline

Updated 2026-10-21. The working implementation uses **Tauri 2, Svelte 5, Vite, TypeScript, Rust and SQLite**, following the user's stack change. Flutter and Flutter Rust Bridge are superseded. Full Xcode is not a blocker for this implementation: the release built with Apple's Command Line Tools selected per process.

## Delivered

The app supports onboarding, preferences, a real inventory/storage scan, streamed stages and counts, cancellation with retained partial results, evidence and capability inspection, storage exploration, live monitoring, saved history, report export and reviewed removal of local diagnostic records. The production UI uses native IPC; it does not provide simulated device readings. All Phase 1 destinations are connected. Later action and conversational workflows are documentation only.

The release application is `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app`. Distribution archive: `release/CoquiPCCleaner-macOS-arm64.zip`; checksum: `release/SHA256SUMS`. These are local build artifacts, excluded from Git. The executable is arm64, targets macOS 14+, and has a verified local ad hoc signature. It has no Developer ID signature or notarization. Nothing was uploaded or published. Git remote is configured as `https://github.com/kokin-blip/CoquiPCCleaner.git`.

## Capability coverage

| Area | Implemented behavior and qualification |
| --- | --- |
| Inventory | Model, OS, CPU/core configuration, memory, GPU identity, readable disks/volumes/filesystems, applications and battery information when supplied by macOS. Missing fields stay unavailable. |
| Monitoring | CPU, memory, swap and readable process I/O every two seconds; 450-sample memory buffer. Collection stops with app exit. Collector failure clears stale readings, shows stopped/unavailable and offers retry. No inferred hardware bottleneck or invented sensor readings. |
| Storage | Standard accessible home scope, confirmed expanded deep scope, separately selected additional folders, exclusions, metadata traversal, overlap/hard-link deduplication and mount boundaries. Logical and allocated sizes are distinct from filesystem capacity. |
| Exploration | Category treemap and accessible controls/table, directory drill-down, paginated/sortable file results, large files, exact-path inspector and Finder reveal. Non-Unicode filenames that cannot cross the string IPC boundary are skipped with a coverage error. |
| Review candidates | Explicit cache/temp rules with origin, reason and consequences. Candidate allocation is not guaranteed recoverable space. No deletion control for inspected files. |
| Startup | Readable launch configuration inventory. It does not claim runtime enabled state or complete modern login-item coverage; manual Login Items guidance is available. |
| Disk health/security | Available SMART/firmware properties, readable FileVault/SIP status and qualified platform information. Secure Boot inspection is manual where no supported reading is exposed. GPU utilization, temperatures and wear estimates remain unavailable. |
| Local data | Rust-owned SQLite schema/migrations, settings, summaries/evidence/findings, retained partial sessions, latest complete file index per scope, 30-day configurable retention, local-data removal and JSON report export through a native save dialog. |

## Checks performed

`./scripts/check.sh` passed after the final code changes:

- Svelte/TypeScript analysis: **0 errors, 0 warnings**; production frontend build passed.
- Frontend: **15 tests passed** across state, workflows and treemap geometry. Includes onboarding persistence, routing/tabs, partial/cancel transitions, preference-save ordering, storage drill-down, purge review, and monitoring sample-then-failure/recovery.
- Rust: **12 tests passed**, including real inventory, live metrics, scan/cancel, database reopen/purge and a controlled filesystem fixture. Other tests cover missing/malformed fields, startup qualification, migrations/version protection, pagination, exclusions, links/overlap, permissions, placeholders, disappearing/disconnected roots and read-only traversal.
- Rust formatting and native Tauri `cargo check --locked` passed.

`./scripts/release.sh` passed. The rebuilt `.app` launched successfully and restored retained scan results through the real Tauri command/channel boundary. The ZIP was recreated after the monitoring correction and its signature verified by `codesign --verify --deep --strict`.

Native validation ran on Mac16,8 / Apple M4 Pro / macOS 26.6.1. Inventory showed 24 GiB memory (25.8 GB in decimal UI units); CPU, memory and swap updated while the app was open. Native folder selection and a real three-file fixture scan succeeded, with category allocation, cache candidate, drill-down and saved history visible. App relaunch restored the saved results. A real session was exported through the native save dialog and reopened as valid JSON with matching inventory and scan counts. Capacity comes from macOS filesystem statistics; shared APFS volume warnings are qualified rather than added together.

Visual checks captured the packaged app in dark and light appearance, a roughly 1440-wide window and the approximately 1100 × 720 minimum. Keyboard focus, scrolling, tab navigation and exact-path handling were checked in source/tests and native captures. The independent finish reviewer found one material issue, stale readings labeled live after monitoring failure; its final verdict was **ship**, scoring that fix resolved. The exceptional-state image uses the actual UI/controller with a review-only failing collector fixture; it is not a production test backdoor. Normal captures use the packaged native app and real readings. Design handoff is recorded in `figma.md` and the Figma ledgers.

## Validation still requiring follow-up

- macOS 14 itself and additional Apple Silicon models have not been exercised on physical hardware. Intel and Windows remain separate deferred work.
- Full-home/deep scans at real-user scale, protected locations with Full Disk Access, disconnected physical external/network volumes, battery-equipped models and additional disk-health implementations need device coverage. Tests cover fixture equivalents; this is not a claim of physical-device validation.
- Engine cancellation and UI cancellation transitions pass automated tests. The manual fixture scan finished before a native cancel click could interrupt it; a long-running native cancellation should be checked on a larger selected fixture.
- Accessibility semantics, keyboard operation and reduced-motion styles are implemented. A complete manual VoiceOver walkthrough, all contrast combinations and every dialog/partial/error state have not been independently audited.
- Gatekeeper behavior on another Mac after download is unverified; this distribution is deliberately unnotarized.
- Figma frames are editable structural handoff views with illustrative labels, not pixel-exact captures of every implemented screen or exhaustive variants of every state.

The principal Phase 1 workflow is implemented and locally validated. The items above remain explicit acceptance work before claiming validation across the entire supported hardware/OS range. Later phases retain their original approval, verification and recovery requirements in `phases/`.

## UI refinement — repeated advisories

Removed the persistent shield/read-only notices from the sidebar, content footer and dashboard device summary at the user's request. Contextual onboarding and permission guidance stays in place. Removed unused styles. `scripts/check.sh` passed again (15 frontend tests, 12 Rust tests, zero Svelte errors/warnings); the release app/archive were rebuilt, signature verified, and the relaunched native dashboard visually confirmed without the repeated notices. Capture: `.impeccable/review/native-dashboard-clean.png`.

## Phase 2.1 — cleanup approval and verification

User selected this milestone on 2026-10-21. Version 0.2.0 adds eligible individual user-cache files, exact-path review, ten-minute approvals, native macOS Trash, per-file verification, cancellation and retained action history. Scans remain read-only. The complete Phase 2 vision remains in `phases/phase-2.md`; current scope and exclusions are detailed in `phases/phase-2-progress.md`.

`./scripts/check.sh` passes: zero Svelte errors/warnings, 20 frontend tests, 26 Rust tests, formatting and locked native Tauri check. The explicit native cleanup integration test also passes: newly created disposable fixtures were reviewed, approved, moved by FileManager, verified, journaled and restored to their fixture directory. No existing user files were used for mutation testing. The earlier lockfile mismatch noted during concurrent logo work is resolved by the Phase 2 dependency updates.

The ad hoc signed 0.2.0 release app/archive were rebuilt. Packaged-app validation preserved old settings/history through schema 2 migration, scanned named disposable cache fixtures through real IPC, listed eligible files, displayed a disabled-until-acknowledged exact-path approval dialog, and saved Keep files as a declined action record. Visual captures cover cache files, approval, history and minimum window size. Native GUI validation declined the reviews; approved native execution was exercised by the separate Rust integration test. Full-device, macOS 14, VoiceOver and other-Mac Gatekeeper checks remain outstanding as above.

Figma has a linked structural selection → approval → verification/recovery handoff in the existing Optimization Center page; see `figma-cleanup.json`. Phase 1's ZIP is retained locally under `release/phase-1/`. Nothing has been published or uploaded.

Final review found and resolved a project-ancestry eligibility gap. Checks now include project markers through canonical home, with additional candidate/preview/execution fixtures. The reviewer scored that fix resolved and returned **ship**; the documenter preserved the existing visual system. The final packaged app was relaunched after the fix. This Mac's home-level `package.json` means current cache descendants are conservatively omitted; project-aware support for that case remains a subsequent milestone. No manifest was moved or bypassed. The final native fixture integration test passed again, for **20 frontend + 26 standard Rust + 1 explicit native integration test**.


## Approved logo integration — 2026-10-21

Applied the user-selected `public/brand/coqui-file-emblem-v1.png` to the sidebar, onboarding, opening state and browser favicon. Generated the macOS ICNS and PNG sizes from that source with the pinned Tauri CLI (`npm run tauri -- icon public/brand/coqui-file-emblem-v1.png --output /private/tmp/coqui-logo-icons`), then copied the ICNS and macOS PNG sizes to `src-tauri/icons/`. The transparent artwork uses the existing wordmark without a colored container. Other logo concepts remain untouched.

`./scripts/check.sh` passes: zero Svelte errors/warnings, 20 frontend tests, 26 standard Rust tests, formatting and locked native check. Rebuilt the ad hoc signed release app and ZIP, verified the archive/checksum and packaged ICNS, relaunched the native app and visually confirmed the new sidebar mark. No scanner or cleanup behavior changed.
