# Downloads exact-copy review — 2026-10-27 source preview

This continuation implements the Downloads location and modification-age filters in the CleanMyMac feature ledger. It extends the existing explicit content-analysis and keeper-preserving review workflow. It does not broaden removal authority or implement browser-history inspection, download-source attribution or photo-quality classification.

## Flow

Cleanup → Duplicates starts with Downloads only and a separate unchecked similar-PNG/JPEG option. All indexed locations remains available. Choosing a location starts no collection; Analyze contents is the explicit content-read action. Changing location clears the displayed result and all keeper/removal selections; changing the optional photo choice applies to the next analysis. Report context separately retains the choices used for the displayed result.

Search, last-modified age (any, at least 30/90/365 days before the report, unavailable) and largest/smallest/oldest sorting operate on saved exact groups. A group matches when one copy satisfies the path/age filters, but every member remains available for keeper choice, with 100-copy pages for larger groups. Page changes preserve explicit keeper/copy choices; the group summary names the keeper and counts selections across pages. Filters reset both page levels and all selections, preventing hidden selected copies. Unknown dates do not become old dates; the age anchor is the report timestamp, so reopening does not silently change its classification. Last modification is not download time, last use, abandonment, backup evidence or safe-deletion authority. Similar-photo hints are separately labeled and outside these exact-group filters.

## Engine and scope

The generated `DuplicateAnalysisRequest` selects only Indexed or Downloads and the photo option; unknown locations/extra caller-selected root fields are rejected. Rust derives Downloads from the account home, refuses linked/cloud/unavailable/excluded roots, and includes a Data alias only when macOS's published firmlink map confirms it. Candidate selection applies component-boundary path ranges before the existing largest-2,000 limit, using the existing `(session_id,path)` index rather than SQL LIKE wildcards. Literal percent/underscore home names and Downloads-like siblings are fixture-covered. Known aliases normalize before deduplication and content reads. Exclusion checks compare the same normalized aliases.

New `analyze_duplicate_scope` IPC uses the existing work guard, fixed user process and cancellation channel. The older analysis API remains available with its existing all-indexed/photo behavior. No helper, network, Git, browser history/URLs, privacy mutation or schema upgrade is added. Reads and cleanup retain user ownership, no-follow/cloud/hardlink/project protections and independent fresh hash/byte comparison. A disconnected initial consumer cancels before content reads.

The existing content limits remain: at most 2,000 nonempty indexed candidates, 256 MiB each, 2 GiB initial hashing plus the separate verification-read bound, cooperative 120-second outer bound, and optional bounded PNG/JPEG similarity. Source scan omissions, an absent requested index, candidate/time/byte bounds and unreadable/changed copies keep the report partial. An indexed empty folder is distinct from an unindexed requested location. A bounded no-group result never establishes global uniqueness or exhaustive Downloads access.

## Persistence and removal

The existing schema-7 duplicate record adds defaulted nullable request/resolved-root fields. Legacy records leave those choices unrecorded; History and restored reports do not infer a Downloads scope. Retained reads, filtering and sorting write nothing and start no analysis. Report creation is the separately requested diagnostic write; inspected file contents and image pixels are not retained.

Keepers and removal selections start empty. Only exact confirmed copies with the existing personal-folder eligibility may proceed to the separate fresh review. Existing project/Library/system, shared/hardlinked/cloud/exclusion protections, ten-minute single-use approval, per-item journal, keeper revalidation, Trash verification and restore limits remain. No automatic preselection, removal, permanent deletion or Trash emptying is added.

## Evidence and remaining limits

See [status](../status.md) for completed checks and [the design handoff](downloads-duplicates-design-handoff.md) for final capture/reviewer evidence when completed. The native disposable fixture uses an isolated database and synthetic home, creates three files, intentionally dates one copy 100 days earlier, confirms one exact group and leaves both copies unchanged. Exported report/session files support a visibly labeled review viewer. They are fixture evidence, not measurements of the user's Downloads folder or native bridge/cleanup acceptance.

The actual packaged preview can review location/photo choices without submitting content analysis. The installed verified 0.12.0 pair, FDA grant and administrator-off preference remain unchanged. No user-file cleanup, fresh whole-disk scan, browser-source catalog, modern login coverage, full VoiceOver or other-Mac acceptance is implied. All earlier agreed Coqui differences remain in force.
