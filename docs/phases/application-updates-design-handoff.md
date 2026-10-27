# Opt-in application updates design handoff — 2026-10-24

This is an ordinary, code-led extension of Coqui's existing **Native macOS clarity** system. [DESIGN.md](../../DESIGN.md) remains the design authority; [the surface contract](../../.impeccable/surfaces/src-features-appupdates-svelte.md) records the Operate direction. The user selected **Show updates; install through vendor** and **Keep guided manual tools**. This handoff documents the implemented selection, source review and result presentation. The [functional phase](application-updates.md) retains the protocol, source and transport boundaries.

## System and composition

Checked [PRODUCT.md](../../PRODUCT.md), [DESIGN.md](../../DESIGN.md), the surface contract, [AppUpdates.svelte](../../src/features/AppUpdates.svelte), [AppUpdateReportView.svelte](../../src/components/AppUpdateReportView.svelte), and the Applications, Overview and History integrations. The extension preserves the charcoal/light surfaces, compact macOS system typography, mint actions, line icons, flat separators and existing focus treatment. It adds no palette, font, shadow system or decorative imagery. The approved frog artwork remains `public/brand/coqui-file-emblem-v1.png`; its existing favicon and app-icon derivatives are unchanged. No shipping raster was generated, replaced or recolored. The captures below are review artifacts, not runtime assets or device measurements.

| Surface | Implemented composition and meaning |
| --- | --- |
| Applications → Updates | The existing Applications heading and tab strip lead to a plain recorded-app table. Search, selected count, Reload recorded apps and Choose application precede the continuation. **Review update sources** is above the table and selected-app disclosure, so a full 50-row page does not hide the next step. Apps start unselected; at most ten exact applications can be selected. |
| Source review | Local metadata inspection replaces the picker with each exact path, installed version, endpoint, source qualification and development source-rule review date. The ten-minute expiry remains visible. A separate, initially unchecked acknowledgement explains which bundle IDs/feed requests and network address the disclosed services receive. Unsupported sources cannot enable the check. |
| Active check | A readable status identifies ongoing work and Stop remains beside the section title. Incremental results retain their actual item states. Cancellation leaves unfinished items qualified rather than treating them as completed comparisons. |
| Results | The picker is replaced by the dated report and Check other apps. Each app keeps its exact path, installed/listed version and build when available, source, source-rule review date, check/publication dates, qualification and listed minimum macOS with compatibility unverified. Plain separated rows carry the evidence; status text accompanies color. |
| Vendor handoff | Only Newer version listed exposes **Open App Store updates** or **Open app to update**. The label communicates that installation continues in vendor tooling; it does not claim a completed update. |
| Overview and History | Saved reports are read without a network request or inventory refresh. Overview retains the report date and says **Saved <status> check · not refreshed by this scan**. The inventory-only updates-not-checked wording is used only when there is no saved update report. History uses the same result component. |

The result vocabulary remains distinct: **Newer version listed**, **No newer version observed**, **Check unavailable**, **Version comparison unavailable**, and **Not checked · cancelled**. A listed offer is a dated observation, not an installation, compatibility, safety, purchase-eligibility or universal up-to-date verdict.

Entering source review waits for the DOM update and focuses its heading. Change selection and Check other apps focus the app-search field; the latter also clears the prior report, selection and acknowledgement. A terminal check focuses the result region. Focus assertions supplement the still images; the images alone cannot demonstrate these transitions.

## Retained visual evidence

All six final captures were opened for this documentation pass. The two native captures show the separately named, uninstalled **Coqui Design Preview.app** at the supported (1100×720) logical window, recorded as (2200×1440) backing pixels. The result captures are (1280×720) development-only component-harness views. Their visible footer labels the data illustrative and states no network or vendor launch.

| Capture | Evidence and limit |
| --- | --- |
| [Native selection](../../.impeccable/review/app-updates-selection-native.png) | Retained local multi-row inventory, zero selected apps, disabled continuation visibly above the table, and the established native shell. This is not a new inventory refresh or online check. |
| [Native sources](../../.impeccable/review/app-updates-sources-native.png) | Explicitly labeled disposable metadata-only public-catalog fixture, exact path and endpoint, expiry/source date, unchecked acknowledgement and disabled online action. No GUI online check was submitted. |
| [Results, dark](../../.impeccable/review/app-updates-results-dark.png) | Top viewport of the actual component harness: retained report presentation, Check other apps and illustrative partial/listed result. |
| [Results, light](../../.impeccable/review/app-updates-results-light.png) | The same top-viewport result composition in the existing light appearance. |
| [Details, dark](../../.impeccable/review/app-updates-details-dark.png) | Supplemental scrolled view of the offered-version evidence, explicit vendor handoff, compatibility limit and unavailable item. |
| [Details, light](../../.impeccable/review/app-updates-details-light.png) | The same supplemental evidence in light appearance. This is not a first-viewport capture. |

The packaged captures establish native selection/source rendering for the observed states. The labeled harness captures establish result/theme rendering with illustrative data; they do not establish native result-state or vendor-handoff acceptance, live update availability or backend/network safety.

## Finish review and validation

The first bounded finish review found three P2 issues: the continuation followed all 50 inventory rows, stage changes removed focused controls without an orientation destination, and Overview could pair a saved update offer with updates-not-checked wording. The correction batch moved the continuation above the table, added source/back/restart focus transitions while retaining terminal-result focus, and conditioned Overview's caption on the saved report. The replacement captures supersede the earlier captures at those six paths.

The follow-up verdict is **ship**, scoped to all three prior findings being resolved, with no regression attributable to that correction batch visible in the supplied captures and reviewed changes. It is a scoring pass for those fixes, not whole-app acceptance or a new Overview design approval. Review records are `/private/tmp/coqui-app-updates-finish-review.md` and `/private/tmp/coqui-app-updates-finish-verdict.md`. This handoff used the Impeccable documenter reference through the available delegated task-agent workflow.

The final `scripts/check.sh` log records **103 frontend +192 Rust =295 passing standard tests**, with four native mutation tests and one opt-in network test ignored by default, zero Svelte errors/warnings, production frontend build, Rust formatting and locked native check. The focused correction-batch log records **28 passing tests across four files**, including source-entry, Back, restart, terminal-result focus and the Overview saved-caption assertions. The subsequent packaged preview build completed with zero Svelte errors/warnings. Logs inspected here: `/private/tmp/coqui-app-updates-accepted-check.log`, `/private/tmp/coqui-app-updates-review-tests.log` and `/private/tmp/coqui-app-updates-review-native.log`. This documentation pass opened existing evidence and logs; it did not rerun tests, operate the GUI, submit an online request or launch a vendor tool.

The separate opt-in native public-catalog test passed with a labeled metadata-only BBEdit fixture and isolated temporary database; its log is `/private/tmp/coqui-app-updates-native-check.log`. The functional phase records the initial IPv6 failure and correction to pin the complete validated public address set. No actual installed-app inventory was transmitted, user-history record added, application launched, installer fetched or update installed by that validation. This test is transport/persistence evidence, not native GUI or broad vendor coverage.

## Preserved boundaries and remaining limits

This remains a separately named, uninstalled 0.12.0 source preview. The installed app/helper pair, pairing manifests and privacy grants remain unchanged. No schema migration or helper-protocol change was introduced. Opening the update route, reloading recorded apps and reopening saved reports do not collect a fresh inventory or contact a source. Inventory refresh is an explicit Installed Apps action. System maintenance remains guided/manual and Ask Coqui remains local.

Removal selections remain empty by default. Update offers, application inventory and source recognition grant no cleanup authority. Existing exact-path review, fresh eligibility checks, expiring approval, journal, verified Trash and separately reviewed restore boundaries remain in force. This work establishes no new disk access, resolved permission denial or whole-product reference parity. The 26 retained system/service denials and existing partial coverage remain qualified.

Full VoiceOver review, macOS 14, another Mac, configured-proxy/device scenarios, broad live vendor-feed coverage and native vendor/App Store handoff remain unverified. Receipt presence and declared feeds do not prove purchase, authorship or installation safety. Actual source/package/native acceptance and remaining limitations belong in [status](../status.md).

The existing DESIGN.md token frontmatter and all prior sections were preserved; only the focused application-updates component guidance was inserted. `.impeccable/design.json` was not regenerated. Its recorded generation time remains `2026-10-23T23:30:48.103676+00:00`, and its earlier narrative freshness drift remains disclosed by the prior handoffs. Existing weight mismatches also remain: primary-button frontmatter inherits body weight 400 while runtime/sidecar preview use 600; status frontmatter inherits caption weight 400 while runtime/sidecar preview use 500. These are not new app-update tokens or rules and were not repaired as a side effect.
