# Broader cleanup categories — design handoff

Recorded 2026-10-27 for the **uninstalled 0.12.0 source preview**. This ordinary extension preserves **Operate / Native macOS clarity**. [PRODUCT.md](../../PRODUCT.md), [DESIGN.md](../../DESIGN.md) and [the sidecar](../../.impeccable/design.json) remain authoritative; [the surface contract](../../.impeccable/surfaces/src-cleanup-categories.md) records direction and [the functional phase](cleanup-categories.md) owns detailed scope and protections.

## Incumbent comparison and workflow

The final [CleanupCategories component](../../src/features/CleanupCategories.svelte) and [Optimization host](../../src/features/Optimization.svelte), shared styles/tokens, authorities, phase, status and [finish review](../../.impeccable/review/categories-finish-review.md) were inspected. All nine captures below were opened. This documentation pass inspected completed logs without rerunning tests or collection.

| Incumbent commitment | Finished extension |
| --- | --- |
| Charcoal/light neutrals, restrained mint, system typography | Existing semantic variables, compact headings, metadata and aligned numeric columns; no new palette, tokens or visual world. |
| Flat workspace and native controls | Category table → replacement detail view, shared separators/disclosures, buttons, checkboxes and pagination. |
| Persistent desktop shell | Five primary destinations and existing Cleanup tabs remain; native captures cover the 1100×720 logical minimum. |
| Evidence Before Decoration | Partial/unavailable states, source dates and qualifications remain text. Indexed zero differs from unknown; allocation is not removable space. |
| Explicit review and focus | Opening focuses the detail heading; Back restores its category button. Cache choices start empty and enter an independent fresh review. |

**Cleanup → Categories** now opens twelve fixed groups: app caches, Xcode caches, user logs, DerivedData, archives, device support, simulator device data, local iPhone/iPad backups, account Trash, shared caches, system logs and system caches. Existing focused review handoffs keep their destinations. The approved [frog artwork](../../public/brand/coqui-file-emblem-v1.png) and its icon/favicon derivatives remain unchanged; no shipping raster was generated or replaced.

Counts/allocation describe retained regular-file rows, excluding cumulative folder totals, current exclusions and overlapping Xcode cache paths. Detail shows purpose, consequences, roots and up to 100 exact file rows per page. Unknown/excluded locations remain nullable; empty pages do not establish a clean category or complete access. Custom paths and comprehensive app/resource classification remain outside these rules.

Refresh and pagination are pure retained-index reads. Rust owns fixed definitions and bounded SQLite queries; Svelte owns navigation, focus, empty/reset selections and stale-response guards. No traversal, content/Git/network/helper operation, history write, cancellation reset, schema migration or arbitrary-root query is added. Rule-review dates remain separate from scan dates.

Only user/Xcode cache categories expose initially empty selections, cleared on source/category/page changes. **Check selected cache files** forwards exact paths to the existing fresh cache review; ownership, no-follow/canonical path, link/cloud/project/exclusion checks, separate expiring approval, consequences acknowledgement, journal and native Trash verification still govern. No cache executor/rule is expanded. DerivedData opens the existing protected Developer review. Other groups remain manual or Protected with no removal selection: no log/archive/backup deletion, simulator reset/prune, Trash emptying or permission repair. Path recognition, age, Git history and cached remote matches establish neither safe deletion nor a current backup.

Three exact-allowlisted Apple sources were reviewed **2026-10-27**, independently of scan time: [Xcode locations](https://help.apple.com/xcode/mac/current/en.lproj/deva52afe8a4.html), [debug symbols](https://developer.apple.com/documentation/xcode/locating-a-missing-debug-symbol-file) and [device backups](https://support.apple.com/en-us/108809). They open only on explicit selection; category browsing starts no network request. Default paths do not establish reproducibility, recovery-copy validity or cleanup eligibility.

## Capture evidence

Native images are 2200×1440 backing pixels for **1100×720 logical** size. The seven **1280×720** browser captures render actual components with visible fixture labels. [categories-preview.ts](../../.impeccable/review/categories-preview.ts) is review-only source, uses disposable exports and blocks collection/network/removal. Scrolled captures establish only the named state.

| Capture | Evidence and limit |
| --- | --- |
| [Native list](../../.impeccable/review/coqui-categories-native-list.png) | Actual retained Partial scan, dated rules, recorded footprints and unavailable Xcode caches. |
| [Native User logs](../../.impeccable/review/coqui-categories-native-logs.png) | Heading focus, manual consequence text, exact paths and Page 1. Recorded native walkthrough also reached Page 2; no Page 2 capture is claimed. |
| [Light fixture list](../../.impeccable/review/coqui-categories-light.png) | Labeled disposable metadata, existing light shell and table hierarchy. |
| [Dark fixture list](../../.impeccable/review/coqui-categories-dark.png) | Same fixture and hierarchy in the incumbent dark appearance. |
| [Lower groups](../../.impeccable/review/coqui-categories-dark-lower.png) | Deliberately scrolled archives through system groups, unavailable values and Protected modes. |
| [Cache detail](../../.impeccable/review/coqui-categories-cache-light.png) | Zero selected, unchecked file and disabled fresh-check action; no approval/removal acceptance. |
| [Archive guidance](../../.impeccable/review/coqui-categories-archives-dark.png) | Manual-only archive consequences and separate Apple source-review date. |
| [Protected Trash](../../.impeccable/review/coqui-categories-protected-dark.png) | Inspect-only unavailable/empty state and explicit incomplete-access qualification. |
| [Unknown measurements](../../.impeccable/review/coqui-categories-unknown-light.png) | Explicitly illustrative Not recorded date, Partial status and unavailable measurements. |

## Finish verdict and validation

The independent [finish review](../../.impeccable/review/categories-finish-review.md) returned **ship**, **no material fixes**, within the supplied source/nine-capture scope. It preserves empty selections, independent fresh cache review, Protected categories, qualified measurements and the mint/frog composition. This is not whole-product or accessibility certification.

The completed unmodified `scripts/check.sh` passed **119 frontend +209 Rust =328 standard tests**, with **eight opt-in native tests ignored**, zero Svelte/TypeScript errors/warnings, production frontend build, Rust formatting and locked native check (`/private/tmp/coqui-categories-check.log`). Focused category/Optimization tests passed **10/10**. Coverage includes null/zero states, pure refresh, focus, manual/Protected controls, fresh-review handoff without execution, reset/stale responses, disjoint file-only totals, literal path/alias/exclusion boundaries, 100+1 pagination, strict decoding, no-write reads, disposable-log executor refusal and bounded SQL work.

The separate native test passed, preserving six files in a synthetic home/isolated database (`/private/tmp/coqui-categories-native.log`). Its [report](../../.impeccable/review/categories-native-report.json), [pages](../../.impeccable/review/categories-native-pages.json) and [session](../../.impeccable/review/categories-native-session.json) export all twelve views. Fixture success does not establish comprehensive access or real-file cleanup.

The separately named, ad hoc signed/unnotarized **Coqui Design Preview.app** built successfully with zero Svelte errors/warnings (`/private/tmp/coqui-categories-preview.log`). It opened the real retained Partial scan: app caches **219,016 files /24.1 GB rounded allocation**, user logs **1,647 /556.5 MB**, DerivedData **300 /14.3 MB**, unavailable Xcode/backup groups and protected shared/system groups. Logs Page 1/Page 2 required no collection or approval. These are historical footprints, not new measurements or removable amounts.

## Preserved limits and drift

The installed verified 0.12.0 app/helper pair, manifest, release ZIP/installer, administrator-off preference and approved persistent helper FDA grant remain unchanged. No real user file was selected, moved or removed for validation. No new full-scale scan, privacy/helper authorization, update installation or maintenance action occurred. Schema 7, removal protections, local Ask Coqui, vendor-managed updates and guided manual maintenance remain intact; the rejected software-requirements comparison stays rejected.

The 26 system/service denials and prior deliberate/live-filesystem omissions remain qualified. Installed matching-build/helper acceptance, comprehensive access, macOS 14/other Macs, full VoiceOver, every resize/interaction state, custom/app-specific coverage and full reference-product parity remain unverified. Native AX flattens category cell controls; recorded keyboard focus/Return opened User logs and focused its heading. A clean changed-Svelte hook scan does not establish complete accessibility. Earlier login/photo/update/review-queue gaps remain open.

DESIGN.md and its sidecar were preserved. Pre-existing drift remains: primary-button frontmatter inherits weight 400 while runtime/preview use 600; status frontmatter inherits 400 while runtime/preview use 500. The sidecar's `2026-10-23T23:30:48.103676+00:00` generation time and administrator-centered provenance/narrative predate later extensions. The previously recorded context-loader macOS Platform-prose mismatch remains disclosed. No design-system refresh or drift repair was performed.
