# Administrator helper and session design handoff

Recorded 2026-10-23 for the 0.8.0 optional helper/session extension. The [direction contract](whole-disk-cleanup.md#direction-contract) preserves **Operate / Native macOS clarity**. The [administrator boundary](administrator-helper.md) owns implementation authority and current native validation. This handoff records the supplied source and captures after the [fresh finish review](../../.impeccable/review/administrator-finish-review.md) returned **ship**, with no material fixes in that scope.

## System and task order

The extension keeps the grouped sidebar, toolbar, selected unboxed Coqui logo, macOS system font, compact controls, flat panels, separators, mint actions, dark/light appearances and visible modal focus. It is code-led; no replacement visual world, generated comp or raster asset was introduced. Settings places the helper between storage and privacy controls. Existing switch rows, wrapping action groups, work banners, evidence tables and protected-focus dialogs carry the new states.

One additive semantic token records the destructive-button contrast correction: runtime `--on-danger` maps to `dark-on-danger` and `light-on-danger` in DESIGN.md and the sidecar. Filled destructive controls pair it with `danger`; the frontmatter owns exact values. The supplied reviewer reports approximately 7.50:1 dark and 6.47:1 light contrast. This documentation pass did not rerun that contrast calculation.

The intended task sequence and exceptional cases are:

1. **Install and check.** Use the matching optional app-plus-helper package and open `/Applications/CoquiPCCleaner.app`. Check installed helper explicitly. Unavailable, mismatched or failed connections retain their qualification/error; the off-to-on switch requires a compatible check. Installation and checking start no scan or cleanup. The portable app remains usable.
2. **Set privacy access and enable.** Full Disk Access for the exact installed helper is separate macOS setup and can require renewal after an ad hoc build replacement. Enabling administrator choices requests a native session lasting up to 15 minutes in the current app process; macOS may expire rights earlier. The waiting banner directs attention to macOS and exposes Cancel operation. Waiting never asserts completed authorization. Installer, privacy and app authorization do not promise one total setup prompt.
3. **Review each operation.** The session makes administrator choices available. Each scan still reviews its mounted/selected scope; each removal independently refreshes eligible objects and requires exact paths, contents and consequences to be acknowledged. Related application data starts unchecked. Review preparation and active work expose cancellation. Cancellation, lost clients and restart never replay approval or resume uncertain moves; a possible in-flight move remains recorded.
4. **Inspect retained outcomes.** Per-item terminal detail, original/Trash paths and verification remain inspectable. Coverage distinguishes readable traversal from comprehensive access, including unavailable evidence. Helper-index results and imported local records retain their provenance and partial qualifications after restart. Restore requires its own review and acknowledgement, retained identities and an absent original destination; it never overwrites or promises complete application recovery.
5. **Lock when finished.** Explicit Lock, switching off and app exit clear process authority. Session text says remaining time is “at the last check”; it is not a live expiry guarantee. No closed-app collection starts.

Helper-record removal and helper removal remain separate confirmations. Record removal names this account's private scans/recovery references and imported/public mirrors, while preserving files, Trash and other accounts. Removing the helper preserves the app and private journal; matching reinstallation is needed for private recovery references. Ordinary local-history removal has its own scope. The captured private-journal dialog was inspected and cancelled; the user's records were not purged.

## Checked evidence

This pass read PRODUCT.md, DESIGN.md, the sidecar, architecture/status/roadmap, the direction contract, administrator specification, documenter playbook, finish report, Figma ledger/script and the source below. It opened all eight captures and checked dimensions. No live app/Figma control, new capture, detector, build, test or mutation ran in this documentation pass.

| Capture in `.impeccable/review/` | Checked state |
| --- | --- |
| `helper-session-light-wide.png` | Matched helper, enabled session, qualified remaining time and explicit Lock in light Settings. |
| `helper-session-light-minimum.png` | Light minimum-size wrapping, paths and helper action group. |
| `helper-session-dark-wide.png` | Dark session and Lock with the incumbent focus treatment. |
| `helper-session-dark-minimum.png` | Dark minimum-size session disclosure and actions. |
| `helper-session-wait-light-wide.png` | Native-authorization waiting banner, disabled conflicting work and cancellation. |
| `helper-session-purge-dark-minimum.png` | Destructive record-removal confirmation, scope disclosure and visible Cancel focus. |
| `helper-session-coverage-minimum.png` | Reviewed VM root, four files/five entries and explicit traversal/access distinction. |
| `helper-session-restored-light-minimum.png` | Restored disposable cache/app, exact paths, verified outcome and limited recovery disclosure in a scrollable dialog. |

Light/wait wide captures are **2880×1786 physical / 1440×893 logical**; dark wide is **2880×1784 / 1440×892**. All minimum captures are **2200×1442 / 1100×721**. Wide height is host-clamped; the 1440×960 design reference and 1100×720 supported minimum remain unchanged. Settings/coverage are intentionally scrolled; the restored record shows the top of a longer dialog. Purple sharing/pointer overlays belong to capture tooling.

| Source | Responsibility checked |
| --- | --- |
| `src/components/HelperSettings.svelte` | Compatible-build gate, default-off session, waiting/Lock, privacy access and distinct lifecycle confirmations. |
| `src/features/Settings.svelte` | Placement, ordinary permissions/history scope and incumbent settings composition. |
| `src/components/RemovalReview.svelte` | Independent administrator preview, contents focus/pagination, acknowledgement, cancellation and terminal evidence. |
| `src/components/Dialog.svelte` | Native modal, accessible title, close and Escape behavior. |
| `src/tokens.css`, `src/styles.css` | Both `on-danger` values, destructive pairing, focus, wrapping, spacing, typography and reduced motion. |

The ship disposition covers these captures and sampled source. The reported clean Svelte/CSS detector result is builder evidence, without a native-code detector verdict. Full VoiceOver, independent security review, macOS 14/additional hardware, every error state and downloaded Gatekeeper behavior remain outside this review.

## Native and Figma handoff scope

Reported implementation validation passed **58 frontend + 135 Rust = 193 standard tests**, plus the fixed root-owned disposable app/cache Trash and separately reviewed restore cycle. Peer/version/missing-authorization/replay rejection, cancelled scanning and stop/on-demand restart retained records passed. The native validator measured **one interactive Authorization Services API request** across that workflow, then verified Lock; this does not count Installer/Full Disk Access setup dialogs. The actual GUI waited for native authorization, reused its session for the reviewed VM metadata scan without another prompt, then locked. Four files/five entries, folder/coverage index and records reopened after restart; headless hardware inventory remained unavailable and the session partial.

The ordinary release package subsequently installed successfully, removed only marked disposable fixtures/marker, preserved app/private journal, and passed matched health and strict signatures with no active session. System appearance, Selected folders scope, administrator off and monitoring on were restored. These are reported native results, not rerun by the documenter. No existing user file was removed, complete disk access or full recovery established, or artifact published. Builds remain ad hoc and unnotarized.

The [Figma ledger](../figma-whole-disk-cleanup.json) and [authoring script](../../scripts/figma/administrator-helper.js) record the editable 1440×960 helper frame `185:1933` on page `155:91` in file `Gz7WEiN8Y3Vvivvae0ZPlP`. It includes session/Lock/separate setup, with action `185:1960` linking to cleanup `183:1889`. Source kit, components, variables and earlier frames remain intact. Inter is the established editable fallback; native runtime uses system type. This is illustrative structural handoff, with no pixel-parity claim; this pass checked local provenance rather than reopening Figma.

Documentation changes are limited to DESIGN.md, `.impeccable/design.json` and this handoff. Existing primary-button frontmatter weight 400 versus runtime/preview 600, status frontmatter weight 400 versus runtime/preview 500, and the context loader's unrecognized macOS Platform prose are reported and preserved. The authorized token addition does not repair those unrelated mismatches. Windows, Intel, Phase 2/3 and existing project/cache/duplicate/shared-container protections remain intact.
