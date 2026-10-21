# Phase 2 progress

Current milestone: **cleanup approval and verification**, selected by the user on 2026-10-21. Version 0.2.0 implements this milestone; it does not claim completion of the full Phase 2 specification.

## Milestone 2.1 — individual cache-file review

- Eligible candidates come from an actual retained scan index, then receive fresh filesystem checks. The supported action is moving individually selected regular files beneath the current user's `Library/Caches` to native macOS Trash. Folders, links, hard links, cloud placeholders, excluded locations, files owned by another user and project-managed trees are omitted. No default selection.
- Rust owns a persisted, ten-minute review plan with exact paths, current logical/allocated sizes, modification dates and consequences. Approvals bind to that plan and its creation revision; both application-closed and consequences acknowledgements are required. Declined, expired or already-used plans cannot execute.
- Execution is separate from analysis. Scans, cleanup and diagnostic-data purge share an operation guard. Each target is checked against device/inode, size, allocation, modification/change timestamps and parent identity. Linked ancestors and project markers are checked again. The target is revalidated immediately before calling the native API.
- The native adapter uses [Apple's FileManager Trash API](https://developer.apple.com/documentation/foundation/filemanager/trashitem%28at%3Aresultingitemurl%3A%29?language=objc). A returned move is verified against the source's absence and the same file's identity at the returned Trash path. A partial or unverifiable operation is recorded honestly.
- The journal records approval before mutation and intent before every native move. Cancellation stops remaining files; completed work remains recorded. Channel loss cancels remaining work. Interrupted intent becomes unverified on reopening and is never retried automatically.
- Optimization includes cache-file pagination/selection, exact-path review, Keep files, streamed results, cancellation, per-file details and paginated action history. Available Trash paths can be revealed in Finder. Finder may offer Put Back while an item remains available; Coqui does not promise reversal, overwrite a replacement file, permanently delete anything or empty Trash.
- SQLite migrates from schema 1 to 2 without discarding settings or scan history. Action records share configured retention and the explicit local-data purge. Storage snapshots remain historical after cleanup; rescan to refresh them.

The filesystem checks are point-in-time checks around a pathname-based native API. They do not provide an atomic lock against another process replacing a pathname during the native operation. Uncertain results require manual inspection; there is no automatic retry.

## Validation

The standard check suite passes 20 frontend tests and 26 Rust tests. A separate native integration test also passed: a new disposable fixture was previewed, approved, moved by FileManager, verified, journaled, and returned to its fixture location. No existing user cache files were used for mutation testing.

Tests cover no-change/decline, missing acknowledgements, expired/replayed approval, index membership, out-of-scope paths, directories, links, hard links, projects, new exclusions, changed file/parent, audit persistence failure, native permission/tool failures, partial results, cancellation, channel loss, interruption recovery, schema migration/reopen and purge. UI tests exercise selection, exact-path confirmation, result/recovery presentation, errors and retained records.

Packaged native validation includes schema migration with existing history, a real scan of named disposable fixtures, eligible-file listing, preview, disabled approval until acknowledgement, Keep files and retained declined history. The actual native mutation path was tested from Rust, rather than by approving existing user files through the GUI. The original macOS 14/device/VoiceOver/Gatekeeper validation limits remain applicable.

## Remaining Phase 2 milestones

The full requirements in `phase-2.md` remain authoritative. The following are not implemented by 2.1 and have no inactive navigation:

- Ask Coqui, evidence-bound diagnostic tasks, conversation history and optional explicitly opted-in external models.
- Exact duplicates with content verification and copy preservation; similar-photo and modified-copy distinctions.
- Git/worktree validation and technology-specific developer-artifact analysis/cleanup, Docker/application-native resource management and application leftovers.
- Supported startup/power adjustments and before/after workload baselines.
- Sourced workload suitability/upgrade advice, forecasts, compatibility and cost ranges.
- Maintenance scheduling, reminders and separately authorized rules.
- Additional action types and programmatic recovery only after each has its own eligibility, approval, verification and recovery design.

## Conservative ancestor rule

Supported project markers are checked through the canonical home directory, including `Library` and home itself. A marker there can exclude every cache descendant. This Mac has a home-level `package.json`, so the final packaged build correctly omits the validation cache files instead of offering an override. Resolving that case needs project-aware eligibility in a subsequent milestone; do not move or ignore a user's manifest merely to enable cleanup.

The finish reviewer scored the sole material fix, ancestor-project exclusion, resolved and returned **ship**. The final native capture demonstrates the conservative no-eligible-files case. Other captures establish the selection/review/history composition from before this policy correction; no UI layout changed in the correction. The documentation handoff confirmed that existing design tokens and components were reused and preserved DESIGN.md and its sidecar.
