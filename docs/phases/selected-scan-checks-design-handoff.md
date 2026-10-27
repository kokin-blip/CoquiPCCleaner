# Selected scan checks — design handoff

Recorded 2026-10-26 for the uninstalled 0.12.0 source preview. This is an ordinary extension of Coqui's **Operate / Native macOS clarity** system. [DESIGN.md](../../DESIGN.md) remains the visual authority; [the surface contract](../../.impeccable/surfaces/src-scan-modules.md) records the direction. [The functional phase](selected-scan-checks.md) owns collection, exclusions, retention and helper boundaries. This handoff covers the explicit scan choices, matching dated results and reachable scan-review actions.

## Sources and system comparison

Checked [PRODUCT.md](../../PRODUCT.md), [DESIGN.md](../../DESIGN.md), [.impeccable/design.json](../../.impeccable/design.json), the surface contract and functional phase, [tokens.css](../../src/tokens.css), [styles.css](../../src/styles.css), [App.svelte](../../src/App.svelte), [Dialog.svelte](../../src/components/Dialog.svelte), [ScanObservationView.svelte](../../src/components/ScanObservationView.svelte), and the Overview, History-record and Performance integrations. The architecture, roadmap and status retain the source/native acceptance boundaries. This fresh delegated documentation pass used the Impeccable documenter reference and its generic-agent fallback, with the Operate and craft references; it did not rerun context, tests or the GUI.

| Incumbent commitment | Observed extension |
| --- | --- |
| Charcoal/light surfaces and restrained mint | Checkboxes, primary action, separators and result text reuse semantic theme variables. The extension defines no new palette or theme. |
| Compact macOS system type | The dialog and result inherit the established system stack, 13px body, 17px section headings and 14px subheadings. Per-check paragraphs cap at 75ch. Dates, counts and qualifications remain readable text. |
| Flat workspace | Three metadata choices share a fieldset; per-check results use separated rows. Existing modal depth provides protected focus for scope approval. No new shadow or ornamental container system was added. |
| Resizable desktop composition | The existing dialog gains an optional footer: its body scrolls while its heading and Cancel/Scan controls remain outside that scroll area. Other dialogs retain their existing structure. The native capture shows the supported 1100×720 logical window. |
| Semantic Color and Evidence Before Decoration | Collected, Partial, Not selected and Cancelled remain explicit text. Missing counts and completed dates are disclosed; the System check says readings recorded rather than inventing an entry count. |
| Flat Workspace and shared interaction finish | Existing button shapes, checkbox accent, visible focus and reduced-motion rules apply. The captured unchecked System checkbox has a visible focus outline. No separate interaction or motion vocabulary was introduced. |

The approved frog source remains `public/brand/coqui-file-emblem-v1.png`, with its existing favicon/app-icon derivatives. No shipping raster was generated, replaced or recolored. The five images below are review evidence, not runtime artwork or a promise of current measurements.

## Implemented flow

**Scan Mac** opens scope/check review before collection. Whole internal disk retains mounted-volume selection; standard/selected-folder scanning reviews its configured locations. The additional System details, Startup configurations and Application inventory choices start selected for each new review. Their disclosure states that they read standard locations separately from selected storage roots. Duplicate/photo content checks and online updates remain separate; system maintenance stays manual. Cancel closes review without starting a scan.

The fixed footer keeps **Cancel** and **Scan selected volumes** or **Start scan** reachable when the mounted-volume body is long. The native screenshot shows only the beginning of the volume table; it demonstrates scrolling containment and action visibility, not simultaneous display of every volume.

Overview keeps the four established review destinations. **What was checked** contains the matching scan's metadata report with independent state, date, nullable count, omissions and qualification for each check. Older startup/application evidence rows are suppressed when that report exists. History reuses the same result component. Links into system/startup/application detail appear only when the report matches the selected scan, so an older History report cannot silently navigate into a different scan's details.

Startup uses that scan's retained check and rows, including skipped/cancelled states. Its configured-on-disk/runtime-unknown wording and manual System Settings handoff remain visible. Saved browsing performs no new collection. Review navigation, observed footprints and metadata recognition create no removal approval.

## Retained visual evidence

All five final captures were opened during this pass. The native image is 2200×1440 backing pixels for a 1100×720 logical preview window. The other images are 1280×720 development-harness views.

| Capture | Evidence and limit |
| --- | --- |
| [Native whole-disk review](../../.impeccable/review/scan-modules-scope-native.png) | Actual packaged **Coqui Design Preview.app** scope dialog, System deliberately unchecked, visible focus, long scroll body and fixed action footer. The review was cancelled; this is no scan or helper-authorization evidence. |
| [Selected-folder review, light](../../.impeccable/review/scan-modules-selected-light.png) | Browser harness rendering of the three selected choices, separate-location disclosure and Start/Cancel actions in light appearance. The modal obscures the harness footer; it displays no invented measurement. It does not establish native bridge interaction. |
| [Metadata results, dark](../../.impeccable/review/scan-modules-results-dark.png) | Scrolled Overview rendered from the isolated native report: System collected with no count, Startup partial with 30 entries/two omissions, Applications partial with 121 entries/ten omissions. Independent dates and the aggregate Partial label remain visible. The footer identifies the isolated record viewer. |
| [Skipped/cancelled states, light](../../.impeccable/review/scan-modules-cancelled-light.png) | Explicitly illustrative specimen: System Not selected, other checks Cancelled, no counts or completed check dates. It establishes state presentation, not a native cancellation run. |
| [Startup detail, dark](../../.impeccable/review/scan-modules-startup-dark.png) | The same isolated native record rendered in Startup. Exact configuration paths and runtime-unknown text remain readable, alongside the manual Settings handoff. No Settings action is evidenced. |

The result captures establish presentation of genuine isolated observations through the harness; they do not establish packaged native result interaction or administrator orchestration. No UI source changed after these captures in the reviewed backend correction batch.

## Finish verdict and verification

The independent finish review retained five sections covering capture validity, material findings, contract/craft, disposition and bounded evidence. It found no additional material visual defect in the supplied states, but identified three integration findings: implicit legacy metadata collection by the helper despite unchecked choices; exclusions inside a bundle failing to guard the actual Info.plist read; and a failed administrator import starting foreground checks.

One backend correction batch resolved all three. The helper worker now enters storage-only traversal without the legacy snapshot; the actual bundle Info.plist target is checked against normalized exclusions; and foreground checks require successful public import with the exact expected file count. Focused disposable regressions cover those decisions. The ordinary legacy scan API remains separate. No visual rebuild or replacement capture was required.

The appended verdict is **ship**, limited to F1–F3 being resolved and the previously valid capture/source scope. It is not installed-helper or whole-product acceptance. Review record: `/private/tmp/coqui-scan-modules-finish-review.md`; focused correction log: `/private/tmp/coqui-scan-modules-review-fixes.log` (**49 unit tests passed**).

The completed `scripts/check.sh` run exited zero: **108 frontend +200 Rust =308 passing standard tests**, six opt-in native tests ignored, zero Svelte errors/warnings, production frontend build, Rust formatting and locked native compilation. The completed log is `/private/tmp/coqui-scan-modules-final-accepted-check.log`. The verdict's earlier log snapshot correctly said the full script was still running; this final checkpoint comes from its completed log and confirmed terminal exit.

The separate opt-in metadata test passed in `/private/tmp/coqui-scan-observations-native.log`. Its saved `/private/tmp/coqui-native-scan-observations.json` matches the displayed 30/two and 121/ten results, with System collected and the report partial. It used an isolated temporary database and one disposable storage file, preserved that file and retained exactly one public storage entry. It added no user-history record and establishes no new full-scale disk acceptance.

The corrected preview build also exited zero; `/private/tmp/coqui-scan-modules-reviewed-preview.log` records the completed ad hoc signed, unnotarized **Coqui Design Preview.app** bundle. The parent confirmed that it reopened on Overview's start screen without submitting scope review, authorizing the helper or starting another scan. This documentation pass inspected that build log and existing captures, without operating the app. Actual release status remains in [status](../status.md).

## Preserved boundaries and drift

The installed 0.12.0 app/helper pair, pairing manifest, administrator-off preference and approved persistent helper FDA grant remain unchanged. The helper-worker source correction requires a future matching app/helper build; this handoff does not certify the installed helper's new behavior. No removal, restore, privacy reset, firmware/security change, maintenance execution or broader cleanup authority follows. Vendor-managed updates, local Ask Coqui and empty removal selections remain in force. The 26 retained system/service permission denials and partial coverage remain qualified.

Native administrator orchestration of the new foreground stage, full VoiceOver, keyboard traversal of every result state, macOS 14/other Macs, modern login coverage and comprehensive disk access remain unverified. No deterministic native detector verdict is claimed, and detector silence is not visual acceptance.

DESIGN.md and `.impeccable/design.json` were preserved. The sidecar's generation time remains `2026-10-23T23:30:48.103676+00:00`; its administrator-centered provenance/narrative predates subsequent extensions. Existing primary-button frontmatter inherits body weight 400 while runtime/preview use 600, and status frontmatter inherits caption weight 400 while runtime/preview use 500. The previously recorded context-loader failure to recognize the macOS Platform prose also remains disclosed; context was not rerun here. These pre-existing discrepancies are neither new selected-check rules nor defects to canonize. They were not repaired because this ordinary extension authorizes no design-system refresh.
