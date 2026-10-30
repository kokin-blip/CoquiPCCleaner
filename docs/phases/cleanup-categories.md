# Broader cleanup categories — 2026-10-27 source preview

The user requested continued CleanMyMac-style feature work, starting with broader cleanup categories and preserving agreed differences. This batch extends Cleanup with a dated, file-only category table and paginated exact-path inspection. It preserves existing removal authority rather than granting deletion from a path classification.

## Scope and workflow

Cleanup starts at Categories. Existing Cache files, Duplicates, Developer artifacts, Recommended plan, Resources and Action history remain available; focused existing cache/developer/duplicate handoffs retain their destinations. Five primary sidebar destinations, mint/frog identity and the existing native composition remain.

Twelve standard-location groups distinguish app caches (excluding the separate Xcode cache group), Xcode caches, user logs, Xcode DerivedData, Xcode archives, Xcode device-support paths, simulator device data, local iPhone/iPad backups, account Trash, shared caches, system logs and system caches. Custom Xcode locations, other volumes' Trash, browser-history origins, every app-managed cache/resource type and comprehensive system-data classification are outside these path rules.

Open a category for its purpose, consequences, recognized roots and up to 100 indexed file rows per page. Unknown/unindexed or fully excluded groups have nullable counts/bytes, not measured zero. An indexed empty location can show zero indexed files, with the same scope qualifications. Partial source scans remain partial. Counts/bytes exclude cumulative folder rows, current exclusions and disjoint subgroup paths; allocation is never advertised as removable or uniquely allocated physical capacity.

Only user/Xcode cache groups expose initially empty file selections for the existing fresh cache review. A selected row is not established eligible: current ownership, canonical/no-follow path, cloud/hardlink/project/exclusion checks can refuse it. The existing exact-path expiring approval, consequences acknowledgement, journaling, native Trash verification and recovery limits still govern. Logs, archives, backups, simulator data, Trash and system categories have no removal checkbox/action. DerivedData hands off to existing Developer review, preserving unassociated-Xcode/project/Git protection. No log deletion, archive/backup erasure, simulator reset/prune, Trash emptying or permission repair is introduced.

## Read boundary and persistence

Rust owns fixed category definitions, generated typed report/query/page models and SQL. The new Tauri queries run off the UI thread. They read the current-schema public index/settings in SQLite read transactions, without filesystem traversal, content reading, Git, network, helper operations, history writes, work-guard acquisition or cancellation resets. No migration or helper protocol change is required.

Queries use exact primary-key path ranges before aggregation/sorting, including only OS-published Data aliases. Current exclusions are translated to the same alias; literal percent/underscore names and prefix siblings are not SQL wildcard scopes. At most 200 preference exclusions are supported by this category view, with an explicit error above that bound; Storage remains available. All-category totals are not summed into a reclaimable-space headline. Measurements describe retained file rows, not a new scan or authenticated ownership census.

Svelte owns navigation, focus, source/category/page selection reset and stale-response guards. Primitive source/status/exclusion dependencies prevent live metric updates from launching repeated category reads. Opening/back focuses the detail heading/category button. Retained refreshes keep the original scan date, and rule review date remains separate. The existing approval dialog remains the mutation boundary; browsing creates no approval record.

## Guidance provenance

Vendor sources were reviewed **2026-10-27**, independently of scan dates. Links open only through the existing exact-URL allowlist, after user selection; browsing categories starts no network request.

- [Apple: project-generated file locations](https://help.apple.com/xcode/mac/current/en.lproj/deva52afe8a4.html) describes default DerivedData/Archives and custom locations. Coqui does not infer reproducibility or removal eligibility from those defaults.
- [Apple: missing debug symbols](https://developer.apple.com/documentation/xcode/locating-a-missing-debug-symbol-file) supports retaining needed archive/debug-symbol evidence. Coqui does not verify a particular archive's recovery value.
- [Apple: locate/manage device backups](https://support.apple.com/en-us/108809) identifies the standard MobileSync backup location and Finder's Manage Backups workflow. Coqui does not inspect backup contents or verify another recovery copy.

Other category purpose/consequence text is conservative path guidance, not a vendor-certified cleanup rule, an installed-tool verdict or safe-deletion claim. The rejected software-requirements comparison is not revived.

## Validation boundary

See [status](../status.md) and the design handoff for completed checks and native evidence. Disposable fixtures cover file-only/disjoint totals, unknown versus zero, current exclusions, bounded pages, legacy missing dates, manual-path refusal by the existing cache executor, no-write reads, known aliases and SQL work bounded against unrelated index rows. A separate disposable native export preserves its fixture files and supports an explicitly labeled component viewer.

The installed 0.12.0 app/helper pair, manifest, administrator-off preference and existing helper FDA grant remain unchanged. The preview is not a new installed paired release. The retained 26 system/service denials and all prior readable-scope/VoiceOver/other-Mac limitations remain qualified. No real user file is moved or removed to test this feature.

## Guided selection continuation — 2026-10-30

The user-approved [guided cleanup flow](guided-cleanup.md) supersedes the cache category's page-local 100-file selection limit. Categories and Cache files now share namespace/group selection across all pages, explicit broader all-groups selection, individual exceptions and a disk-backed fresh review with one final approval. Schema 8 adds bounded item journals; legacy APIs/history remain. This does not expand cache eligibility or manual/protected category authority. See status for actual validation; the installed application and release assets were not changed by this source continuation.
