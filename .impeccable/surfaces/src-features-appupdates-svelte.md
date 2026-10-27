---
version: 1
slug: "src-features-appupdates-svelte"
primary_target: "src/features/AppUpdates.svelte"
related_targets: ["src/features/Applications.svelte","src/components/AppUpdateReportView.svelte"]
---

# Application update review

Mode: Operate. Help the Mac owner find offered updates for selected apps while retaining control of networking and vendor-managed installation.

## Direction contract

THESIS: Select apps, inspect sources, explicitly check, then hand off to the vendor. An app-update offer is a dated observation rather than an installation or compatibility verdict.

OWN-WORLD: Extend Coqui's existing system typography, charcoal/light surfaces, mint action and compact native tables. Use line icons; no new raster assets or decorative status scores.

STORY: Apps start unselected. Local source review shows exact apps and endpoints. A per-run acknowledgement enables networking. Results distinguish offered versions, unavailable checks, cancellation and unsupported comparison.

FIRST VIEWPORT: Applications owns the existing header and tab strip, with Updates as a secondary tab. The selected count and Review update sources precede the plain recorded-app table. Source review replaces the table; Stop remains visible while checking. Completed results replace the picker and retain detailed evidence.

FORM: Code-led extension of the pinned incumbent macOS composition. Seed key: not applicable to the established system.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

User confirmed online checks are opt-in, installation stays in the App Store/vendor updater, and system maintenance remains guided/manual. Keep empty removal selections and all existing cleanup boundaries. No app-update network request occurs while browsing or reopening history.

## Final evidence — 2026-10-24

The ordinary extension preserves the incumbent world and introduces no shipping raster. The six final captures are `app-updates-selection-native.png`, `app-updates-sources-native.png`, `app-updates-results-dark.png`, `app-updates-results-light.png`, `app-updates-details-dark.png` and `app-updates-details-light.png` in `.impeccable/review/`. Native selection/source captures use the (1100×720) logical packaged preview, retained inventory and a labeled disposable metadata fixture; result/theme captures use a visibly illustrative (1280×720) component harness with network/vendor launch disabled. Detail captures are supplemental scrolled views.

The bounded finish verdict is **ship** for the three corrected P2 findings: continuation placement above a full inventory, source/back/restart focus orientation and Overview's saved-report caption. This does not establish native result/handoff acceptance, real update availability, comprehensive VoiceOver or whole-app acceptance. [The design handoff](../../docs/phases/application-updates-design-handoff.md) retains the evidence and validation scope; [status](../../docs/status.md) owns actual source/package/native acceptance.
