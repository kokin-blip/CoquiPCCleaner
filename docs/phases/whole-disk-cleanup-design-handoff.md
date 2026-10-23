# Whole-disk and removal design handoff

Recorded 2026-10-23 for the **0.8.0 portable candidate**. This is an ordinary extension of the existing **Operate / Native macOS clarity** workspace. The [whole-disk direction contract and implementation boundary](whole-disk-cleanup.md) remain the authority for this surface. The complete 0.8.0 plan remains unfinished: native administrator-helper installation, authenticated connection and removal, elevated operations and the final optional app-plus-helper installer are still gated.

## Incumbent-system comparison

The build preserves the existing grouped sidebar, 52px toolbar, independently scrolling content, macOS system typography, neutral dark/light surfaces, thin separators, compact controls, mint actions and visible focus treatment. Applications follows Storage. The approved unboxed Coqui file emblem and wordmark remain in the shell. This extension introduces no replacement visual world, new global palette or raster UI assets. The system font is the explicit native product direction; editable Figma handoffs retain the established Inter fallback.

The new views reuse panels, keyboard tabs, search fields, compact tables, separated history rows and native modal dialogs. Application inspection uses a full-width panel instead of appending evidence below a long inventory or journal. Related-file rows use 12px vertical padding and a separator; reviewed removal items use 16px vertical padding. Full paths wrap and remain selectable. Review contents use the existing horizontal table-scroll pattern, a locally bounded 220px contents region and 100-entry pages. These are task compositions within the incumbent system, not new design tokens.

Five-line system summary:

- **Palette:** inherited charcoal/light neutrals and Coqui mint/deep green; warning and status meanings stay explicit in text.
- **Type:** inherited system stack; 32px onboarding, 28px page heading, 17px section, 14px subheading, 13px body and 12/11px supporting text.
- **Semantic Color Rule:** action, selection and qualified completion use their existing semantic roles; color never supplies the only state label.
- **Evidence Before Decoration Rule:** exact object identity, observation context and access gaps precede removal authority; unavailable evidence stays qualified.
- **Flat Workspace Rule:** tonal panels and thin separators stay flat; the existing modal shadow and dimmed/blurred backdrop establish approval focus.

`DESIGN.md` and `.impeccable/design.json` were read and preserved. Existing token-description drift remains outside this extension: frontmatter primary-button typography inherits body weight 400 while the runtime and sidecar use 600; status component typography inherits caption weight 400 while runtime/sidecar badges use 500. These mismatches are reported, not canonized or repaired. No material decoration defect was identified by the supplied finish review. Historical capture labels such as “Contents of item 1” and lowercase raw states are not rules for future surfaces; current source names the inspected object and humanizes item/history labels.

## Components and interaction

- **Mounted scope:** Whole internal disk opens a dedicated review of already-mounted internal filesystems, with individual selection, exact mount, filesystem and access qualification. It distinguishes a read-only root from roots subject to permissions. External/network volumes require separate selection. The engine owns the expiring plan and identity revalidation; selecting scope grants no removal authority and mounts nothing.
- **Storage and coverage:** Explorer/large-file selection leads to a separate removal preview. Coverage retains reviewed roots, per-root traversal state, visited/file counts, categorized omissions and paginated exact omitted paths. “Traversal complete” describes readable metadata and never asserts comprehensive access. Allocation remains qualified against APFS sharing, clones, snapshots, compression and omitted areas; the empty map reports no recorded allocation rather than inventing a trace or capacity result.
- **Applications:** Installed Apps, Leftovers, Protected Apps and Removal records reuse the existing tab treatment. Search/sort/page queries remain separate from inventory refresh. A chosen, dropped or row-selected bundle replaces the long list with its identity, timestamp, full path, protection/guidance and separately selectable associations. Bundle and related data start unchecked. Exact bundle-ID conventions are qualified associations, not proof that data is disposable; shared/unconfirmed containers retain protection. App-open missing-path notices start disabled and do not establish uninstall success.
- **Inspection focus:** Selecting a bundle from any entry point selects Installed Apps, hides the journal/list, scrolls the inspection heading into view and focuses it after rendering. Back restores the original tab and control; a rebuilt row is located by exact application path, with Choose application as the fallback. The source implements this handoff without changing review or execution authority.
- **Removal review:** A new preview independently refreshes eligible paths, identities and descendants. The dialog shows qualification, full paths, logical/allocated sizes, entry counts, associations, consequences and per-item state. Approval expires after ten minutes. Each contents action resets pagination, scrolls to and focuses the heading naming that object; the table references that heading and exact-path description through accessibility attributes. The separate acknowledgement starts unchecked and enables the explicit Move to Trash action only when review state permits it.
- **Outcome and recovery:** Running state exposes cancellation of remaining moves; terminal records retain distinct per-item outcomes and Trash references. History distinguishes removal from restore and labels already-restored removal items “Restored From Trash.” Restore is offered only for retained verified Trash moves and requires its own fresh preview and acknowledgement. Existing destinations are never overwritten. Restoring a bundle does not promise dependencies, registration, changes elsewhere or a working installation. Clearing diagnostic data removes Coqui recovery references and does not empty Trash.

The scanner remains metadata-only and read-only. Rust owns traversal, policy, expiry, filesystem validation, journaling and verification; Svelte presents evidence, selection and declarative navigation. The separate 0.8 executor does not relax cache, exact-folder/Git/worktree or duplicate-keeper policies. No permanent deletion, automatic Trash emptying, cloud hydration, firmware/security change or arbitrary privileged command is implied by these controls. Ask Coqui remains local.

## Evidence checked

This documentation pass read `PRODUCT.md`, `DESIGN.md`, `.impeccable/design.json`, architecture/status/roadmap, the direction contract, the documenter specification, relevant Svelte views, shared styles/tokens, Tauri configuration, the Figma ledger/authoring script and the supplied finish report/verdict. It opened all eleven captures below and checked their physical dimensions. No browser/native-app control, fresh scan, mutation, detector, build or test run was performed for this documentation pass.

| Capture in `.impeccable/review/` | Evidence and qualification |
| --- | --- |
| `whole-disk-final-inspection.jpg` | Wide dark inspection immediately below the tabs, full disposable bundle path, identity and initially unchecked bundle/data choices; no journal above the chosen object. |
| `whole-disk-final-inspection-minimum.jpg` | Same active inspection at minimum size, with wrapping disclosure and the review action accessible. |
| `whole-disk-final-removal.jpg` | Wide dark review with two separate selected objects, exact paths, consequences, pending states and ten-minute expiry qualification. |
| `whole-disk-final-removal-controls.jpg` | Scrolled wide dialog showing the second object's named contents heading, visible mint focus outline, exact path and corresponding contents table. This is a scrolled view, not a capture of the final acknowledgement/footer. |
| `whole-disk-final-contents-minimum.jpg` | Scrolled minimum dialog showing the second object's focused heading, full path and matching table. |
| `whole-disk-final-return-focus.jpg` | Minimum-size return to Removal records. Current source and the supplied native accessibility observation support focus returning to Choose application; the still image alone does not prove the focused accessibility element. |
| `whole-disk-final-recovery-history.jpg` | Wide dark records with a completed restore, humanized per-item labels and the original removal labeled Restored From Trash, without a stale restore offer. |
| `whole-disk-scope-light-minimum.jpg` | Minimum light mounted-volume scope review, selected roots, filesystem/access columns and visible close-control focus; more rows/controls require dialog scrolling. |
| `whole-disk-coverage-light-minimum.jpg` | Scrolled minimum light Coverage view, readable metadata qualification and retained reviewed-root state. It is not a complete-disk-access claim. |
| `whole-disk-restore-minimum-top.jpg` | Earlier minimum restore review showing identity, original/Trash paths and limited recovery consequences. Lowercase state text predates humanization. |
| `whole-disk-restore-minimum-controls.jpg` | Earlier minimum restore footer showing contents pagination, separate unchecked acknowledgement, Cancel and disabled restore action. The generic contents heading predates the named-heading/focus correction. |

Wide images measure **2880×1786 physical pixels**, representing **1440×893 logical pixels** under the host-clamped window height. Minimum images measure **2200×1442 physical pixels**, representing **1100×721 logical pixels**. Tauri and the structural Figma reference retain a 1440×960 default and 1100×720 supported minimum. Purple sharing and pointer overlays belong to capture tooling, not product styling. The captures show disposable fixture objects and retained observations at a review point; their small allocation values are not device-health or recoverable-capacity promises.

The [initial finish review](../../.impeccable/review/whole-disk-finish-review.md) identified two material focus/task-handoff issues. The [finish verdict](../../.impeccable/review/whole-disk-finish-verdict.md) scores both resolved: selected application inspection becomes the active view with return focus, and a second-item contents action navigates/focuses its named evidence at wide and minimum sizes. Its disposition is **ship for those two scored fixes within the portable capture/source scope**, not certification of the whole surface or complete 0.8 release. The reviewer reported no attributable regressions and source-token contrast of 6.27:1 dark / 6.10:1 light for secondary-on-surface text and 7.83:1 dark / 6.47:1 light for primary-action text. These contrast results and native keyboard/accessibility observations are supplied evidence, not independently rerun results.

The supplied [once-only detector result](../../.impeccable/review/whole-disk-detector.json) is `[]`. It was read and not rerun. Reported validation is **53 frontend and 124 Rust tests**, plus native disposable file/folder/application Trash and verified restore. That implementation evidence supports the portable boundary; it does not establish every state, platform or elevated path.

## Structural Figma handoff

The [source ledger](../figma-whole-disk-cleanup.json) records four linked editable structural frames on page `155:91` of file `Gz7WEiN8Y3Vvivvae0ZPlP`: mounted scope/coverage (`183:1633`), Applications/associations (`183:1761`), exact-path approval (`183:1889`) and verified Trash/reviewed restore (`183:2017`). The [authoring source](../../scripts/figma/whole-disk-cleanup.js) clones the prior handoff frame `180:1441`, reuses existing shell/control components and appearance variables, and adds a separate flow. The original kit/source is preserved. The ledger reports Inter and no image nodes in these frames. Their copy is illustrative; these structures make no pixel-parity, measured-device or elevated-feature-completion claim. This pass inspected the local ledger/script and did not independently reopen Figma.

## Artifact and source map

The local packaged app exists at `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app`; its Info.plist identifies 0.8.0 and minimum macOS 14.0. The supplied archive exists at `release/whole-disk/CoquiPCCleaner-0.8.0-portable-candidate-arm64.zip` (8,637,103 bytes; SHA-256 `8bbc0f43dca7a018c3d055d2b37679673e2cb4ffb204f22e2b280959f9ed4666`). These checks connect the handoff to a local candidate; this pass did not launch, rebuild, extract or independently validate its signature/distribution behavior.

| Source | Responsibility |
| --- | --- |
| `src/App.svelte` | Applications after Storage, mounted-volume review entry and qualified missing-application notice. |
| `src/features/Storage.svelte` | Indexed selection, separate review entry, coverage/history destinations and allocation qualification. |
| `src/features/Applications.svelte` | Inventory/association/protection views, unchecked selection and inspection/return-focus handoff. |
| `src/components/RemovalReview.svelte` | Preview, per-item evidence, named contents focus/pagination, separate approval and terminal presentation. |
| `src/components/RemovalHistory.svelte` | Retained outcomes, humanized states and separately reviewed restore entry. |
| `src/components/Coverage.svelte` | Root traversal/access distinctions and exact omission paths. |
| `src/components/Dialog.svelte`, `src/styles.css`, `src/tokens.css` | Native modal semantics, shared focus/depth, compact components and unchanged semantic appearances. |
| `docs/phases/whole-disk-cleanup.md` | Engine authority, validation record, portable limits and mandatory administrator gate. |

## Remaining limits and documentation outcome

The production helper is health-gate-only. Local package/probe build and signature checks do not establish native installation, root launch, authenticated connection or removal. Production GUI pairing, per-operation authorization, privileged sealed previews, user-specific Trash/restore, helper journals, administrator-owned fixtures and security/interruption cases remain behind that mandatory gate. The gate package is not the final optional installer and is not elevated cleanup support. App trust grants no administrator rights, TCC access, SIP exception or notarization.

Physical evidence remains limited to the local Mac16,8 / Apple M4 Pro / macOS 26.6.1. Full VoiceOver conformance, every dialog/error/partial state, additional hardware and physical macOS 14 coverage, other-Mac downloaded Gatekeeper behavior, complete whole-disk traversal and comprehensive access remain unvalidated. Metadata review is not a content backup or a lock against other processes; native pathname races, limited restore and uncertain/interrupted outcomes retain the documented qualifications. Windows, Intel, Phase 2 and Phase 3 specifications remain intact.

Documentation outcome: **created only this handoff**. The incumbent `DESIGN.md`, sidecar and all other files were preserved by this pass. No system expansion was needed to describe the portable extension, and no completion claim was added for the administrator or full 0.8 release.
