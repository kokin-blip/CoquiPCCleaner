# Straightforward scan and review design handoff — 2026-10-24

This is an ordinary extension of Coqui's existing native macOS design. [DESIGN.md](../../DESIGN.md) remains the system authority; [the surface contract](../../.impeccable/surfaces/src-app-svelte.md) records the user-confirmed scan-first direction. The existing charcoal/light palette, system fonts, mint actions, line icons and frog mark are retained. No raster was generated, replaced or recolored. The source mark remains `public/brand/coqui-file-emblem-v1.png`; its existing favicon/app-icon derivatives are unchanged.

## Implemented composition

The sidebar exposes Overview, Cleanup, Performance, Applications and Storage. More tools starts collapsed and contains Developer files, Build & test trials, Workload advice, Scan results, Hardware, Firmware & Security and Maintenance. Ask Coqui, History and Settings remain separate utility destinations. Primary rows use the existing rounded selection treatment at a larger (44px) minimum height; secondary rows are compact, and the main navigation scrolls when expanded.

Overview starts with the supplied frog, a short purpose statement and one Scan Mac action. Whole-disk collection still requires the existing volume review; cancelling it leaves the opening view intact. View saved results opens retained results without collection. During a real scan, Overview owns progress and Stop, and category destinations use available evidence rather than invented completion values.

Results use four flat, bordered tiles in two columns: Cleanup, Performance, Applications and Storage. The content caps at (1040px); below (1150px), tile spacing and typography tighten while the grid remains two columns. What was checked discloses recorded measurements and focused review links. Ask about a problem and Suggested next steps remain progressive disclosures. Cleanup opens Cache files; the developer and trial shortcuts retain their detailed existing workflows.

## Evidence and action contracts

The Rust-owned `ScanReview` read supplies five groups and preserves nullable counts, logical/allocated bytes, observation dates, source and state. It performs no collection, history write, network request, Git inspection, helper operation or removal. The displayed four-tile composition does not imply that five independent modules ran.

| Group | Retained meaning and date | Focused review |
| --- | --- | --- |
| Caches / Development files | Exact file-only category aggregates; cumulative folder rows are excluded. Date is the saved scan finish time when recorded. Missing aggregates stay null; measured absence can be zero. Partial source scans stay partial. | Cache files / Developer artifacts |
| Startup configurations | Limited saved foreground snapshot count and its own date. No snapshot means null, not zero. Readable configurations do not establish complete modern login coverage, enabled state or startup delay. | Performance startup review |
| Applications | Qualified retained observations with their own latest individual observation date, independent of the scan; that date is not a complete-refresh timestamp. Empty or undated inventory falls back to the explicitly dated/undated scan snapshot. Sizes remain null. No versions or update offers are refreshed. | Applications |
| Files at least 1 GiB allocated | Exact retained file-row count and bytes at or above (1,073,741,824 allocated bytes), excluding folder rows, dated to the saved scan finish when present. Missing index data stays unavailable/null; logical size can differ. This is Coqui's criterion, not an inferred reference threshold. | Storage size-ranked file list |

The large-file threshold applies to the summary. Its Browse files by size link opens the existing all-indexed-files list ranked by allocated size; it does not apply a threshold filter to that list.

The Performance tile uses live CPU/memory only while monitoring is live and labels that activity separately from unrecorded startup evidence. Monitoring failure remains unavailable. Rounded screenshot values describe their capture moment; they are not new device measurements or fixed UI fixtures.

Removal selections start empty. Review links select a destination, never items or execution. Cache footprint, file size, development/Git evidence, app inventory and category recognition grant no cleanup authority. Existing exact-path review, fresh eligibility checks, expiring approval, journal, verified Trash and separately reviewed restore boundaries remain in force. No combined execution queue is implemented.

## Native captures and finish verdict

These captures show the separately named local **Coqui Design Preview.app** reading the real retained partial index. Minimum captures are (2200×1440) backing pixels; the expanded view is (3024×1786). They are evidence of the observed composition, not comprehensive runtime acceptance.

| Capture | Evidence |
| --- | --- |
| [Start, dark minimum](../../.impeccable/review/straightforward-start-dark-minimum.png) | One Scan Mac, saved-results entry, five primary pages and collapsed More tools. |
| [Results, dark minimum](../../.impeccable/review/straightforward-results-dark-minimum.png) | Four real category destinations with retained/live/partial qualifications. |
| [Results, light minimum](../../.impeccable/review/straightforward-results-light-minimum.png) | Readable selected Overview and two-column results at minimum size. |
| [More tools, light minimum](../../.impeccable/review/straightforward-tools-light-minimum.png) | Expanded secondary tools with independent navigation scrolling. |
| [Results, light expanded](../../.impeccable/review/straightforward-results-light-expanded.png) | Wider content, secondary routes and saved partial status. |
| [Recorded evidence](../../.impeccable/review/straightforward-recorded-evidence.png) | Expanded What was checked with retained count, allocation, date and Partial status. |

The finish review first found one material P2: selected navigation retained a dark fill with dark ink after switching to light appearance. The scoped fix removes sidebar button transitions and explicitly sets the active background color. All three linked light captures are fresh replacements and supersede the earlier defective captures at those paths. The same-process System/dark → Light check passed without restart. The bounded follow-up verdict is **ship**, prior P2 resolved, with no material finding remaining from that review. This resolves that finding within the supplied source/capture scope; it does not broaden accessibility or functional acceptance. Review records: `/private/tmp/coqui-straightforward-finish-review.md` and `/private/tmp/coqui-straightforward-finish-verdict.md`.

## Validation and remaining limits

The full `scripts/check.sh` checkpoint passed **95 frontend +178 Rust =273 standard tests**, with four explicit native mutation tests ignored, zero Svelte errors/warnings, production frontend build, Rust formatting and locked native check. That checkpoint preceded the navigation CSS fix; the subsequent preview native build and Svelte check passed. Logs are `/private/tmp/coqui-cmm-review-final-check.log` and `/private/tmp/coqui-cmm-review-native-final.log`. The final Browse files by size wording is a later scoped label correction with no geometry, theme or flow change; all nine focused Overview tests, production frontend/Svelte checks and the preview native rebuild passed afterward in `/private/tmp/coqui-cmm-review-label-tests.log` and `/private/tmp/coqui-cmm-review-preview-final.log`. This documentation pass inspected the completed logs and six captures; it did not rerun source tests or operate the GUI.

The installed paired 0.12.0 app/helper is unchanged. Appearance was restored to System and administrator preference remains off. No new helper release, privacy grant, scan, removal or restore is established by this local design preview. Full VoiceOver, another Mac/OS, Gatekeeper/notarization and comprehensive disk access remain outside this acceptance. Existing partial coverage and the 26 retained system/service permission denials remain qualified.

The [functional phase](cleanmymac-workflow.md) and [CleanMyMac 5.2.4 research](../research/cleanmymac-5.2.4.md) retain the observed-reference limits and open parity work: foreground observation orchestration, combined review queues, Downloads-focused analysis, broader cleanup mapping, application updates and maintenance execution. This handoff records the first scan/review batch, not whole-product CleanMyMac parity. Pre-existing design-token/sidecar/platform prose drift was preserved; the exact existing token frontmatter and `.impeccable/design.json` were not regenerated.
