# CleanMyMac-style scan and review — source draft

The user asked for a straightforward interface based on the open CleanMyMac app, then requested REA research and equivalent functions. This document records a first implemented functional batch and the remaining gaps. It does not change the original phase specifications or authorize new classes of system mutations.

## Confirmed direction

- Keep Coqui's mint/frog identity.
- The initial view has one Scan Mac action. Category tiles belong to scanning/results; reopening saved results starts no collection.
- Keep developer Git/agent indicators and measured build/test trials as secondary tools.
- Keep hardware, firmware/security, workload advice and maintenance under More tools.
- Removal selections start empty. Existing exact-path, expiring review, fresh checks, per-item journaling, Trash verification and restore rules remain in force.
- App updates use explicit opt-in discovery and vendor/App Store installation. System maintenance remains guided/manual; no automatic DNS/RAM/startup changes are planned under this decision.

## Implemented first batch

Five main destinations: Overview, Cleanup, Performance, Applications and Storage. A separate start/results mode follows actual collection after scope review; cancelling that review leaves the start screen intact. The Overview owns one scan-progress/Stop control. Saved scans retain coverage/status and remain distinguishable from live monitoring.

The pure Rust `scan_review` query produces generated TypeScript types and five result groups. Cache/development groups reuse retained regular-file aggregates, excluding cumulative folder rows. Large-file totals use the existing size index at >=1,073,741,824 allocated bytes and exclude folders. This is an explicit Coqui criterion, not a CleanMyMac threshold. Startup is a limited retained snapshot count. Applications may use separately dated existing observations, independent of the selected scan; no complete-refresh timestamp is inferred from the latest observation. Missing measurements remain null; measured zeros remain zero.

Overview displays these measurements and focused review links, with dates, sources and qualifications under What was checked. It never labels cache-category bytes as removable or reclaimed. Startup's absence in the historical helper scan remains disclosed. Cleanup starts at Cache files. More tools provides explicit Developer files, Build & test trials and Workload advice shortcuts.

The query performs no collection, network access, filesystem traversal, Git inspection, history write or helper operation. No schema/helper protocol upgrade is required. Read-only/current-schema connection and nullable legacy-statistic rules remain intact. Existing cleanup executors and capability boundaries are unchanged.

## Evidence and acceptance

See ../research/cleanmymac-5.2.4.md and its compact evidence JSON for target hashes, REA Evidence IDs, static/live distinctions and unknowns. The reference was Mac App Store CleanMyMac 5.2.4; findings do not cover every edition or unlock state. No reference cleanup, update, maintenance, Trash emptying or startup mutation was executed.

The source/native acceptance checkpoints and screenshots are recorded in ../status.md. A separately named Coqui Design Preview uses retained local data; it is not a new installed app/helper release. Installed 0.12.0 remains unchanged. No comprehensive access, full VoiceOver or other-Mac acceptance follows from this UI batch.

## Remaining functional parity work

1. Validate the newly implemented foreground system/startup/application orchestration through the installed administrator path. The source now offers explicit per-run module choices, cancellation, dates/omissions and saved check results; legacy history is not rewritten and browsing starts no collector. See [selected scan checks](selected-scan-checks.md) for native isolated-database evidence and remaining limits.
2. Extend the new per-scan check review into an optional combined review queue if needed. Module selection and dated metadata reviews now exist; combined execution is still open. Exact duplicates/similar images remain separate content reads; network checks remain separate opt-ins. A metadata scan alone must not mark those modules checked.
3. Validate the newly implemented Downloads-only exact-copy review and saved path/modification-age filters in the installed release. Source filtering precedes the candidate limit; separate photo consent and nullable legacy scope are retained. Browser-origin/history attribution remains unimplemented; reference history-record deduplication is not a content-duplicate algorithm. See [Downloads review](downloads-duplicates.md).
4. Validate the newly implemented twelve-group Cleanup category view in an installed release and separately qualify any future removal expansion. The source/native preview now maps user/Xcode caches, logs, Xcode subgroups, simulator data, device backups, Trash and shared/system groups with file-only dated measurements and exact-path pages. Only user/Xcode caches enter the existing fresh individual-file review; other groups stay manual/protected. Custom locations, broader app-specific resources and new removal classes remain open. An index label or older date is not cleanup authority. Permanent Trash emptying is outside Coqui's removal flow. See [cleanup categories](cleanup-categories.md).
5. Expand and validate app-update source coverage separately. The first opt-in discovery workflow is now implemented with exact local identity/source/version correlation, dated offers, partial/unavailable/cancelled states and vendor-only handoff. See [the update phase](application-updates.md) for native public-fixture verification, conservative protocol support and remaining handoff/device limits. Reference install authenticity remains unverified; Coqui does not install updates itself.
6. Evaluate photo-quality grouping, modern managed login coverage and action verification separately. No manufactured healthy/optimized status or memory/performance improvement is permitted. Maintenance remains guided/manual by the user's explicit choice.

The remaining gaps stay open. Scan/review navigation, honest summary measurements and the bounded update-discovery workflow do not establish whole-product functional parity.

## Guided cache review continuation — 2026-10-30

The user selected guided cleanup, Select all across pages and one approval for a frozen cache selection. The [guided cleanup implementation](guided-cleanup.md) now supplies grouped namespaces, shared explicit selections, a sticky summary, cancellable fresh checks, per-file refusals, bounded exact-path review and individual Trash outcomes. Cache-only bulk execution uses schema 8 and preserves existing cache safety checks. Combined execution across duplicates, projects, updates or maintenance remains outside this change; their current workflows remain available. This source continuation is separate from publication, installation and matching-helper acceptance.
