# Delivery and validation

Current build: **0.4.0, Phase 2 milestone 2.3 — supported project-output approval and verified cleanup.** See [project cleanup scope](phases/project-cleanup.md) and [Phase 2 progress](phases/phase-2-progress.md) for supported actions, tests and remaining roadmap. The Phase 1 record below is retained as implementation history.

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


## Project-aware cache eligibility — 2026-10-21

Version 0.2.1 resolves the blanket exclusion caused by a standalone home `package.json`. Cleanup checks the dedicated user-cache namespace, parses the home package without running scripts, and allows the narrow exception only for a readable, user-owned, single-link, bounded JSON object with no workspace declaration. Repository markers, workspace configuration, cache-local projects, unknown configuration and all existing file/exclusion checks remain protective boundaries. The same policy runs during listing, preview and each move. Details and precise limits are in `phases/phase-2-progress.md`. Review and saved action details now show the file's eligibility context.

`./scripts/check.sh` passes with zero Svelte errors/warnings, **20 frontend tests and 37 Rust tests**, Rust formatting and the locked native check. Eleven added regression tests cover standalone home packages, mixed project/cache trees, new project/workspace declarations after review, ambiguous/oversized/hard-linked/unreadable manifests and unchanged manifest contents. The separate native Trash integration test passed with its own disposable home-package fixture; both newly created files were verified in Trash and restored to the temporary fixture. Total: **58 passing tests**, including that explicit integration run.

The 0.2.1 ad hoc signed macOS app and ZIP were rebuilt and launched. With the actual home manifest untouched, both previously indexed named cache fixtures were listed unselected. Native IPC preview displayed the parsed no-workspace qualification and required acknowledgements. Keep files saved a declined action; reopening that record preserved its eligibility context. No existing user cache files were moved. Visual evidence: `.impeccable/review/project-aware-approval.png`. Existing macOS 14/other-device/VoiceOver/Gatekeeper validation limits remain applicable. This change reuses the existing review layout, design tokens and SQLite schema 2; full developer-artifact and workspace cleanup remains deferred. The 0.2.0 ZIP is retained under `release/phase-2/`.


## Developer-artifact analysis and Git/worktree validation — 2026-10-21

Version 0.3.0 adds Optimization → Developer artifacts. It recognizes indexed Rust, Node.js, Dart/Flutter, Python and Xcode-style outputs with qualified project evidence. Exact-path inspection includes indexed logical/allocated sizes, current checkout changes and tracked entries, nested markers, Git/common directories and registered worktrees. Filters, pagination, streaming progress, cancellation, retry and retained reports are implemented. Project removal is unavailable; clean or ignored status does not prove disposability. Existing individual user-cache approvals retain their separate eligibility and execution checks.

Git inspection uses fixed read-only commands, bounded output/time, disabled hooks/fsmonitor and no lazy fetch. Custom content filters, unreadable/cloud metadata, unknown or broken repository/worktree state, exclusions and incomplete evidence produce protected/manual review. SQLite schema 3 retains one latest developer report per scan and includes it in retention and local-data removal. Older schema-2 releases reject schema 3. Implementation limits and reproduction commands are in `phases/developer-artifacts.md`.

`./scripts/check.sh` passes: **26 frontend tests and 51 Rust tests**, zero Svelte/TypeScript errors or warnings, production frontend build, Rust formatting and locked native Tauri check. Real Git fixtures exercise dirty/tracked outputs, linked/detached/locked/prunable worktrees, submodules, filters, cloud guards, cancellation, persistence, migrations and read-only behavior. The native Trash integration test was not rerun for this read-only milestone; earlier explicit runs remain recorded above.

`./scripts/release.sh` rebuilt the ad hoc signed 0.3.0 Apple Silicon app and ZIP. The final packaged app launched, restored the saved report and completed a fresh check through real Tauri IPC. A disposable selected-folder fixture produced three artifacts: a tracked/dirty Node output marked Protected, plus ignored Rust outputs in the main and linked detached worktrees marked Review required. No project outputs or existing user cache files were moved. The added fixture was removed from future scan preferences after validation; its scan/report remain in app history. The 0.2.1 archive is preserved under `release/phase-2/`.

Native captures cover dark/light appearance, the approximately 1100 × 720 minimum layout, a scrollable worktree inspector and saved results after restart. A live-monitoring update initially reloaded the report/closed the inspector; stable session selection fixes it and a regression test covers it. The independent finish reviewer returned **ship** with no material findings. The documenter preserved DESIGN.md and the design sidecar because this view reuses the established visual system. Captures are under `.impeccable/review/developer-*`; linked structural Figma views and source nodes are recorded in `figma-developer.json`.

This completes the requested analysis/review milestone, not the full Phase 2 roadmap. Technology-specific cleanup approval/execution, duplicate analysis and remaining Phase 2 work remain deferred. The macOS 14, additional hardware, complete VoiceOver and other-Mac Gatekeeper validation limits above still apply.


## Supported project-output approval and verification — 2026-10-21

Version 0.4.0 adds one-folder cleanup review from Optimization → Developer artifacts → Inspect → Review folder cleanup. Supported Rust, Node build, Dart/Flutter and Python bytecode output policies require complete indexed source coverage, a fresh bounded descendant inspection, tracked manifests, clean confirmed Git, ignored/untracked output and safe registered-worktree relationships. Dependencies, virtual environments, unassociated Xcode data and custom/native-tool execution remain analysis only. Exact policy and pathname-race/metadata limits are in `phases/project-cleanup.md`.

The persisted ten-minute review includes exact folder/descendant paths, freshly calculated regular-file allocation, technology consequences and inspection seals. Both stopped-tools and content/regenerability acknowledgements are required. Rust repeats eligibility, Git/worktree, manifest and entire-tree checks before native Trash. The journal records approval and move intent; verification checks original-path absence plus the directory and all reviewed descendant identities at the returned Trash path. Cancellation, partial/failed/unverified outcomes, interruption without automatic retry, history and manual recovery guidance use the existing action flow. Individual user-cache eligibility remains separate. SQLite schema 3 retains additive fields with legacy defaults; no raw contents/scripts are persisted.

The final `./scripts/check.sh` passes: zero Svelte/TypeScript errors or warnings, production frontend build, **32 frontend tests + 69 Rust tests (101 standard tests)**, formatting and locked native Tauri check. Real Git fixtures cover supported rules, stale contents/manifests/Git, linked/locked worktrees, nested markers, links/hard links/cloud/special/unreadable files, bounded coverage, cancellation/interruption, uncertainty, pagination, restart/purge, intent and legacy records. Both separately invoked native Trash tests passed using newly created disposable cache files/project folders; moved entries were verified and restored. No existing user files were used for mutation testing.

The ad hoc signed 0.4.0 Apple Silicon release app and ZIP were rebuilt, archive/checksum/signature verified, and the final app launched. Packaged IPC scanned the named disposable fixture scope (67 files), analyzed two outputs, and prepared Rust/Node exact-folder reviews with a paginated descendant list. The GUI checks confirmed disabled approval until both acknowledgements, keyboard focus/toggling, Keep folder, and retained records after restart. Every GUI review was declined; native mutation was exercised by the separate fixture tests. System appearance and the original two selected scan folders were restored after validation. No project outputs or existing user cache files were moved through the GUI. The 0.3.0 ZIP remains under `release/phase-2/`.

Visual captures cover dark desktop, the approximately 1100 × 720 minimum, consent/contents scrolling, keyboard focus and light appearance. Removed a redundant nested scroll from the single-folder approval. The independent finish reviewer identified one material issue: declined item history still appeared pending/awaiting approval. New declines now record terminal item outcomes; the presentation qualifies legacy records without rewriting their evidence. Regression checks pass, and the reviewer scored this fix **resolved**, returning **ship for the scored fix**. The documenter compared final code/captures with the incumbent system and preserved DESIGN.md and its sidecar; no durable design-system change was required. Figma's linked structural inspection → approval → verification/recovery handoff is recorded in `figma-project-cleanup.json`. Native evidence is under `.impeccable/review/project-cleanup-*`.

This completes the requested supported-folder cleanup milestone, not all Phase 2 work. Per-technology policies intentionally refuse ambiguous content/repository configurations. Original macOS 14, additional hardware, large/remote-filesystem, full VoiceOver and other-Mac Gatekeeper validation limits remain applicable.
