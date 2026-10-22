# Phase 2 progress

Current milestone: **approval and verified cleanup for supported project caches/build outputs**, requested by the user on 2026-10-21. Version 0.4.0 adds milestone 2.3; versions 0.3.0 and 0.2.x supplied developer/Git review and individual cache-file approval. This does not claim completion of the full Phase 2 specification.

## Milestone 2.3 — project-folder approval and verified cleanup

Implemented one-folder exact-path review, freshly inspected paginated descendants, per-technology eligibility, stopped-tools/regeneration acknowledgement, ten-minute single-use approvals, fresh Git/worktree/manifest/tree checks before native Trash, descendant verification and retained outcomes/recovery guidance. Dependencies, virtual environments and unassociated Xcode data remain analysis only. See [supported rules and limits](project-cleanup.md).

## Milestone 2.2 — developer-artifact review

Implemented read-only recognition, indexed allocation, Git status/tracked-entry checks, registered-worktree relationships, protected/review-required assessments, filters, per-artifact evidence, cancellation and retained reviews. Project removal remains unavailable. See [the complete scope and command boundary](developer-artifacts.md).

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

The 0.4.0 standard check suite passes 32 frontend tests and 69 Rust tests. The 0.3.0 suite previously passed 26 frontend and 51 Rust tests. The 0.2.1 suite previously passed 20 frontend and 37 Rust tests. A separate native integration test also passed: a new disposable fixture was previewed, approved, moved by FileManager, verified, journaled, and returned to its fixture location. No existing user cache files were used for mutation testing.

Tests cover no-change/decline, missing acknowledgements, expired/replayed approval, index membership, out-of-scope paths, directories, links, hard links, projects, new exclusions, changed file/parent, audit persistence failure, native permission/tool failures, partial results, cancellation, channel loss, interruption recovery, schema migration/reopen and purge. UI tests exercise selection, exact-path confirmation, result/recovery presentation, errors and retained records.

Packaged native validation includes schema migration with existing history, a real scan of named disposable fixtures, eligible-file listing, preview, disabled approval until acknowledgement, Keep files and retained declined history. The actual native mutation path was tested from Rust, rather than by approving existing user files through the GUI. The original macOS 14/device/VoiceOver/Gatekeeper validation limits remain applicable.

## Remaining Phase 2 milestones

The full requirements in `phase-2.md` remain authoritative. The following remain outside milestones 2.1–2.3 and have no inactive navigation:

- Ask Coqui, evidence-bound diagnostic tasks, conversation history and optional explicitly opted-in external models.
- Exact duplicates with content verification and copy preservation; similar-photo and modified-copy distinctions.
- Broader developer-artifact execution (dependencies/environments/Xcode/custom outputs), exhaustive Git/tool configurations, Docker/application-native resource management and application leftovers. Milestone 2.3 implements only the bounded supported folder policies above.
- Supported startup/power adjustments and before/after workload baselines.
- Sourced workload suitability/upgrade advice, forecasts, compatibility and cost ranges.
- Maintenance scheduling, reminders and separately authorized rules.
- Additional action types and programmatic recovery only after each has its own eligibility, approval, verification and recovery design.

## Project-aware user-cache eligibility — 0.2.1

The 0.2.0 ancestor rule excluded every cache descendant when home contained a `package.json`. Version 0.2.1 distinguishes the dedicated `~/Library/Caches` namespace from a standalone home package. It does not enable cleanup inside arbitrary project trees.

- A home-level `package.json` qualifies for the narrow exception only when it is a user-owned, single-link regular file, readable without following a link, not cloud-only, at most 256 KiB, and valid JSON with an object root and **no `workspaces` key**. Any workspace declaration, even an invalid or null declaration, remains excluded. Declared workspaces are conservatively excluded regardless of their patterns; glob resolution is deferred.
- `.git` (directory or worktree-reference file), `.hg`, `.svn` and `.jj` markers anywhere through home continue to exclude descendants. `Cargo.toml`, `pubspec.yaml`, `pyproject.toml`, `pnpm-workspace.yaml` and `lerna.json` anywhere through home also exclude descendants. Any `package.json` below home, including in `Library`, the cache root or a nested cache project, remains a project boundary.
- Unknown, unreadable, linked, hard-linked, malformed or oversized home package configuration stays excluded. Metadata is checked before opening, an `O_NOFOLLOW` bounded read checks the opened file identity, and modification/change metadata is compared after reading. Scripts are never run and raw manifest contents are never persisted. The scanner itself remains metadata-only.
- Candidate listing, preview and execution use the same classifier. Every native move repeats eligibility checks; newly added repository/workspace markers invalidate a pending file. Existing exact-path approval, identity, exclusion, expiry, journal and Trash-verification requirements remain intact.
- The review and saved action details show the classification in the existing `origin` field, including that the home package was parsed without a workspace declaration. There is no schema/IPC migration and old action records remain historical.

This boundary is a product policy informed by [Apple's user Library/cache organization](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html) and [npm's explicit workspace declaration](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/#workspaces). A cache path is a candidate, not proof of junk or confirmed application ownership. Repository dirty-state/worktree resolution and technology-specific cleanup remain separate milestones.

For 0.2.0, the finish reviewer scored the sole material fix, ancestor-project exclusion, resolved and returned **ship**. The final native capture demonstrates the conservative no-eligible-files case. Other captures establish the selection/review/history composition from before this policy correction; no UI layout changed in the correction. The documentation handoff confirmed that existing design tokens and components were reused and preserved DESIGN.md and its sidecar.
