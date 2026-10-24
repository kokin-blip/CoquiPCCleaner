# Decisions

- Original product brief retained verbatim in original-brief.md.
- Phase 1 is read-only, Apple Silicon macOS 14+, local data, manual scan consent.
- User selected home-folder scans plus an explicit deep-scan mode. External/network roots require separate folder selection.
- User selected new Coqui pages in the existing Figma kit.
- User selected direct distribution without Developer ID signing/notarization; ad hoc signing is allowed.
- During implementation the user explicitly changed the UI stack from Flutter to Tauri. Current stack is Tauri 2 + Svelte/TypeScript + Rust + SQLite. Flutter, Dart, Riverpod, go_router and Flutter Rust Bridge are superseded, although their original requirements remain in the verbatim brief.

- User clarified the frontend as Svelte + Vite; no React implementation is retained.
- Native diagnostics use typed Tauri commands/channels and Rust-generated TypeScript models; bridge regeneration is `npm run types`.
- Command Line Tools are selected per build process, avoiding a global Xcode selection change. Apple Silicon release builds succeeded without full Xcode.
- A failed or unexpectedly ended monitoring stream clears current readings and retains a stopped/retry state; a cached measurement never establishes liveness.
- Figma deliverables are editable structural handoff views and linked prototypes. Native screenshots and real behavior are authoritative for the implemented result; exhaustive visual/state parity remains documented acceptance work.
- User requested removing repeated shield advisories from the interface. Sidebar/footer reassurance and the dashboard read-only blurb are removed; contextual permission information remains in onboarding and Settings.

- User selected cleanup approval and verification as the first Phase 2 milestone. Version 0.2.0 is this bounded milestone, not the whole optimization roadmap. Native Trash moves are limited to explicitly selected eligible individual user-cache files. Scanners remain read-only; execution has its own approval, journal and verification boundary.
- Recovery in milestone 2.1 means exact available Trash locations and manual Finder guidance. No blanket rollback, capacity-reclaimed or performance-improved claim is made.

- User requested project-aware cache cleanup so a home `package.json` does not block everything. Version 0.2.1 grants a narrow dedicated-user-cache exception for a parsed standalone home npm package with no workspace declaration. Repositories, workspace declarations, cache-local projects and ambiguous configuration stay excluded. This does not authorize project-artifact removal or bypass exclusions. Eligibility context is retained with each reviewed file.

- User requested developer-artifact analysis and Git/worktree validation. Version 0.3.0 delivers milestone 2.2 as read-only evidence review: detected outputs, indexed sizes, repository state, tracked artifacts and registered worktrees. A report grants no project-cleanup approval. Content-filter repositories require manual Git inspection rather than allowing Git status to execute filter processes.

- 2026-10-21: Phase 2.3 (0.4.0) adds one supported project-output folder per review, requiring explicit content/regenerability acknowledgement, clean confirmed Git and ignored/untracked output with fresh bounded descendant inspection. Reuse native Trash and the existing journal; verify all descendant identities. Preserve dependencies/environments/Xcode as analysis only. Keep SQLite schema 3 with additive backward-defaulted plan fields; details in phases/project-cleanup.md.

- 2026-10-22: User requested the rest of Phase 2 and explicitly selected local guided Ask Coqui with no external AI connection. Version 0.5.0 delivers the bounded supported workflows in phases/phase-2-workflows.md. Expose application-native/manual capability boundaries instead of generic prune or undocumented privileged changes. Exact duplicates use explicit content reads plus a retained keeper and independent approval, never perceptual similarity as deletion authority. Workload growth scenarios remain declared assumptions, and maintenance runs only while open with no deletion. Rust owns schema 4 retention/restart recovery; schedule configuration remains a preference when diagnostic data is removed.

- 2026-10-22, Phase 3.1: user selected an opt-in macOS update check. Use only `softwareupdate --list`, with explicit network acknowledgement each run, no background maintenance integration, download, install or restart. Apple determines offerings; Coqui does not maintain a per-model latest-firmware catalog. All firmware/security operations remain read-only; Apple Software Update/Recovery guidance is separately acknowledged and cannot imply a successful modification. Accept exact SIP/authenticated-root status with or without terminal punctuation; unfamiliar/custom/localized text stays unknown.

- 2026-10-22, Phase3.2: user chose in-app update notices and optional review reminders, without macOS notification permission. Derive evidence notices only from retained explicit listings; preserve older evidence with failed-check/age warnings, never infer installation from missing offerings. Default reminders off; app-open/reopen only, no automatic utility/network check. Compare chronological selected observations without unique-device, version-ordering or firmware-operation-success claims. Schema5 preserves review preferences/schedule through diagnostic purge; local manual outcomes link a comparison and remain unverified. Supported macOS diagnostic/review scope is complete within documented validation limits; firmware mutation remains unsupported.

- 2026-10-23, 0.9: user selected folder treemap beside ranked folders, with Large files separate. Independently implement folder weights and stable path colors from Coqui's existing palette; preserve reference evidence rather than copying its GPL code/artwork. Measure small files (logical size ≤64 KiB) collectively and retain purpose/consequence qualifications. Labels and aggregate size never establish disposability. Schema 7 preserves legacy counts as unavailable. Ordinary read/status connections must not perform housekeeping writes; keep retention bounded and cancellation active through aggregation and all import stages. Preserve the prior partial scan and avoid a new low-space whole-disk run merely to claim completion.
