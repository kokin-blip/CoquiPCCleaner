# Whole-disk cleanup and application removal — 0.8.0

Approved implementation scope: explicit mounted-internal-volume scan planning; APFS identity/alias handling; bounded SQLite-backed traversal; categorized coverage; reviewed file/folder Trash and restore; application footprint/protection/leftover review; default-off app-open disappearance notices; optional separately installed administrator helper. Existing cache, project/Git and duplicate checks remain independent. No permanent deletion, automatic Trash emptying, cloud hydration, security/firmware modification or arbitrary privileged command API.

The helper installation/launch/authenticated connection/removal gate comes first. This Mac has no valid signing identity and no cached administrator authorization. Ad hoc builds remain required by the user. A traditional Installer package is planned; merely trusting an app does not grant administrator privileges or satisfy SMAppService daemon requirements. Installation requires the user's native administrator authorization. If the OS refuses the package/launch path, record the exact failure before choosing another mechanism.

UI extends the existing Operate/native macOS system, logo, themes and evidence/review/history patterns. Storage adds scope/coverage and review selection. Applications sits after Storage with installed, leftovers and protected views. Related data starts unselected. Missing ownership/coverage stays qualified. No inactive future feature controls.

Validation must use disposable fixtures only, including an explicitly installed administrator-owned fixture. No existing user application or personal file is removed for testing. Tests cover coverage/aliases/scalability, stale seals, permissions, mixed results, authenticated helper boundaries, restart/migrations, native Trash/restore and UI accessibility/layout. Final release requires native app/ZIP/package validation; other hardware/macOS and Gatekeeper limits must be reported rather than inferred.

Progress: portable implementation and local validation are complete within the boundary below. The mandatory helper installation gate is still pending, so the full 0.8.0 plan is incomplete. Local artifacts are labeled portable candidates.

## Direction contract

THESIS: Inspect exact objects and access gaps before granting removal authority. Preserve the native Coqui workspace.

OWN-WORLD: Existing charcoal/light neutrals, mint actions, system typography, compact controls, tables and protected-focus approval dialogs. No new visual system or raster assets.

STORY: Review mounted scope, traverse readable metadata, inspect object evidence, explicitly select items, approve Trash, then inspect verification or review restore.

FIRST VIEWPORT: Existing 232px sidebar/52px toolbar; Applications follows Storage. Search and inventory are immediately available. Inspection replaces the long table; final approval has a dedicated dialog with full paths and a paginated contents table. Minimum 1100×720.

FORM: Incumbent native Operate extension, code-led; no alternative-world selection or generated comp is needed. The signature interaction is the exact-path selection → independently refreshed review → per-item verified outcome.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.

## Implemented portable boundary

`volumes` enumerates already-mounted filesystems with `getfsstat` and qualifies internal identity using structured `diskutil info`. A persisted, ten-minute volume plan is required for Whole internal disk. Revalidation checks reviewed mount/device/volume identity. The saved session retains its reviewed volume plan. No mounting occurs. External/network locations use separate folder selection.

APFS System/Data on the validation Mac share `st_dev`. Root normalization and directory mount boundaries therefore also use `statfs` mounted identity. macOS's published `/usr/share/firmlinks` maps Data aliases to visible paths. Global `(device,inode)` deduplication prevents repeated hard-link/firmlink object allocation. SQLite owns the traversal frontier, identity set, folder accumulation and sorting; its scratch cache is bounded to 8MiB. File/gap writes use bounded batches, with folder totals emitted deepest first. Cancellation retains discovered context and marks unstarted roots separately. The scanner reads metadata only, prunes symbolic links and cloud-only placeholders, and records omission reasons with paginated exact paths. Directory races remain qualified rather than failing the entire scan.

Schema 6 adds mounted-scope plans, coverage gaps, removal journals, application observations and separate application preferences. Existing settings, history and legacy cleanup records remain readable. Retention keeps the latest complete and latest unfinished file index per stable root scope, while retaining summaries for the configured interval. Diagnostic purge also removes application observations and recovery references; preferences survive.

`removal` uses explicit selected-file/folder, app-bundle, associated-data and restore kinds. It shares the existing operation guard and native Trash adapter while retaining a distinct typed journal; it does not relax old cache/project/duplicate rules. Ordinary portable removal supports individually selected user-owned personal contents and selected-volume/shared-folder contents. Home, volume/container roots, system locations, Library namespaces, project boundaries, cloud/provider data and exclusions remain protected or require their dedicated workflow. Folder reviews are capped at 100,000 entries and 100 selected roots; larger reviews must be split. This is an inspection limit, not a scanner/index limit.

Every plan saves metadata identities, parent identities and a length-framed descendant digest. Review expires after ten minutes; execution atomically claims the record, journals intent, repeats policy/identity/application/contents checks and calls native Trash. Bundle symlinks are opaque entries. Verification checks source absence, same returned object and descendant metadata. Lost consumers, cancellation, partial failures and interruption never acquire an automatic retry. Metadata is not a content backup, and a pathname operation cannot freeze other processes; close tools first. No exact capacity recovery is promised.

Restore uses only retained verified Trash references, fresh source/parent/contents checks and original parent identity. Darwin `renamex_np(RENAME_EXCL)` refuses an existing destination at the move itself. Each restore has its own review, acknowledgement, journal and verification. It does not restore dependencies, registrations or changes made elsewhere, and never promises a full application recovery.

`applications` reads bounded, no-follow local bundle metadata and qualifies running state through public NSWorkspace APIs. Inventory refresh is separate from SQLite search/sort/pagination. Exact bundle-ID cache/preferences/saved-state/support conventions plus filesystem ownership are candidates for explicit review, not proof of disposability. Group/sandbox containers remain unconfirmed; another installed bundle declaring the same identity blocks shared-data removal. Related data starts unselected. Running/self/macOS/user-protected bundles, project boundaries, services/extensions and vendor requirements receive protection/guidance. Missing-path notices are default off and checked only at app open; missing does not establish uninstall success.

## Administrator installation gate — pending

`scripts/helper-package.sh` builds an ad hoc signed health-only XPC daemon, a matching probe, `Coqui-Helper-Gate.pkg` and its removal package. The package checks fixed root-owned installation paths for symlinks/unsafe ownership, uses `/Library/LaunchDaemons` with `ProgramArguments`, and registers only its own Mach service. It has no RunAtLoad scan, cleanup or arbitrary command endpoint. A root-owned pairing manifest pins exact code hashes using public NSXPCConnection code-signing requirements. The health request is typed, denies unknown fields/operations/protocols and root/noninteractive callers; client response completion is synchronized. The diagnostic gate pairs its probe, not a production GUI mutation client.

The package/probe build, signature and local gate tests pass. Installation, root launch, authenticated connection and removal have **not** passed native validation. No administrator credentials are cached, and `sudo -n` reports that a password is required. The package was presented for native authorization; no OS rejection of this package mechanism has been observed. Do not infer that local app trust supplies administrator rights, TCC access, SIP exceptions or notarization. Apple describes the traditional Installer daemon route at https://developer.apple.com/forums/thread/771162.

The sandboxed launch of `/System/Library/CoreServices/Installer.app` returned Launch Services `-10827` (`kLSNoExecutableErr`), and native app control timed out before Installer was running. Repeating the explicit package open with sandbox approval succeeded. Installer is now at Standard Install on Macintosh HD (683 KB), stopped before Install and native administrator authorization. No package/daemon rejection has been observed and no different privilege mechanism has been chosen. The user must complete that native step before the authenticated health probe/removal gate can be verified.

Production GUI pairing, per-operation administrator authorization, independently sealed privileged previews, user-specific Trash/restore, bounded metadata jobs and helper-owned journals remain behind this mandatory first gate. The current gate package is **not** the final app+helper installer. It must not be advertised as elevated cleanup support. Installation/connection/removal, administrator-owned fixtures, impersonation/replay/version-mismatch/interruption tests and the final optional installer remain required before calling the complete 0.8.0 plan finished.

## Reproduction and validation

```sh
npm ci
npm run types
./scripts/check.sh
DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo test --manifest-path engine/Cargo.toml --locked --test removal native_file_folder_and_application_trash_and_verified_restore -- --ignored --nocapture
DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo build --manifest-path engine/Cargo.toml --locked --bin native_validation
engine/target/debug/native_validation --volumes
engine/target/debug/native_validation --whole-disk 90
./scripts/release.sh
./scripts/helper-package.sh
python3 scripts/verify-helper-gate.py
# After user-authorized native installation:
./release/helper-gate/coqui-helper-probe --health
```

Native commands require normal macOS access. The validation CLI creates/purges only disposable Coqui diagnostic data; the explicit native removal test creates only uniquely named disposable Downloads fixtures. Neither deletes existing personal data. The whole-disk test is bounded and cancels deliberately; it is not a comprehensive-access claim.

Validated on the local Mac: one native file, folder and `.app` Trash/restore cycle passed with every fixture returned and contents/link identities checked. A corrected 90-second whole-internal scope run inspected 657,688 files: System 271,478 with recorded omissions; Data cancelled after 386,210; six other mounted internal roots remained explicitly not started. It retained 75,645 categorized omissions, including permission failures, cloud placeholders, links, mount boundaries and filesystem races. Restart, retained results/coverage and purge passed in disposable data. A 12,000-file fixture verifies aggregation and high-water RSS growth below 48MiB; other tests cover partial-index retention and separate System/Data roots sharing `st_dev`.

Current physical tests do not establish every Mac/macOS14 configuration, full VoiceOver conformance, downloaded Gatekeeper behavior on another Mac, administrator cleanup, complete whole-disk traversal or comprehensive access. See `reference-feature-map.md` for the complete observed catalogue and explicit capability gaps. No uploads or publishing.

## Portable validation checkpoint — 2026-10-23

`scripts/check.sh` passes: 53 frontend tests, 124 Rust tests, Svelte/TypeScript 0 errors/0 warnings, production frontend build, Rust formatting and locked native check. Four explicit native mutation tests stay outside the standard suite. The new native file/folder/application fixture test passed separately; the older native cleanup suites were not rerun for this checkpoint. One earlier fixture restore attempt correctly refused a changed Trash item/parent; the disposable test now retains an identity-checked recovery guard and obtains a new review for fixture-only metadata changes, rather than replaying an approval.

The real packaged IPC inspected a uniquely named disposable application and exact-ID cache, required separate selection and acknowledgement, verified native Trash moves, separately reviewed/restored both objects, and reopened the records after restart. Original records now say Restored From Trash and do not offer stale restores. No existing personal data or installed application was removed. Scope review was also exercised with only the mounted VM volume selected; coverage and the reviewed plan persisted. User appearance and scan-mode preferences were restored to their initial values.

Native wide captures are 1440×893 logical because the host clamps the requested 1440×960 height. Minimum captures are 1100×721 logical. Dark/light, wrapping, scroll access, review/restore, replacement inspection and keyboard contents navigation were checked within that scope. The independent finish review found two focus/task-handoff issues; both were fixed with regression tests and native recaptures, and the same reviewer scored both resolved. Its verdict covers those fixes and the supplied portable evidence, not full 0.8 release acceptance. No full VoiceOver walkthrough or exact 1440×960 physical-device run is claimed. Figma has four linked editable structural handoffs with source/component ledger `docs/figma-whole-disk-cleanup.json`; these preserve the kit and are not pixel-parity evidence.

`scripts/release.sh` rebuilt the ad hoc arm64 `.app` and ZIP. Re-extraction verifies version 0.8.0, minimum macOS 14.0, arm64 executable and strict signature. `release/whole-disk/CoquiPCCleaner-0.8.0-portable-candidate-arm64.zip` is **8,637,103 bytes**, SHA-256 **8bbc0f43dca7a018c3d055d2b37679673e2cb4ffb204f22e2b280959f9ed4666**. Gate packages and their checksums are under `release/helper-gate/`; they are unsigned Installer packages containing an ad hoc signed health-only binary. They are not the final app-plus-helper installer. Nothing was published.

## Installed gate verification and repair — 2026-10-23

The user installed the initial diagnostic gate and requested continuation. Native inspection confirms root-owned helper, pairing manifest and launch-daemon plist, with the Mach service registered on demand. The initial manifest contained `cdhash H""`: `codesign -dvv` did not emit the hash that the packaging script expected. The resulting invalid signing requirement threw an Objective-C exception and aborted the probe before any authenticated connection. Launchd reported zero runs. This is a Coqui packaging defect, not an observed macOS rejection of package-installed ad hoc daemons. The connection/removal gate has not passed and production privileged operations remain gated.

The repair extracts the hash using explicit verbosity 4, rejects anything other than a 40-digit hexadecimal hash, and verifies the requirement against the staged binary before packaging. Native pairing validation parses both signing requirements before XPC; invalid configuration returns an error. Objective-C exceptions are contained on the native side rather than unwinding into Rust. A regression test rejects the empty/invalid hashes and accepts a well-formed pin.

`scripts/verify-helper-gate.py` expands the package into disposable temporary storage, checks both manifest pins against the actual payload and identical probe, verifies a mismatched code hash is rejected, and confirms the daemon has only fixed `--serve` arguments/Mach service without RunAtLoad or KeepAlive. This verifier passes and now runs after package creation. The corrected probe returns a qualified error against the old invalid installed manifest instead of aborting. `scripts/check.sh` passes with **53 frontend +125 Rust tests**, zero Svelte errors/warnings, build/format/native checks. No user-file mutation or production-helper operation was performed.

Corrected gate package SHA-256: **55a060abccf59e9c311972ad9afccb252e3b15bc6eed1ecd7a9207a328881934**; removal package: **ff54e25aa0600e85fe6b2c46cd0a03e03243879ea9eaddfd584d7e641af86090**. The repaired payload/probe pin is `d7bcf340bc80dc707071cfa59b9a7ba65292a8c2`. Installer is ready for the repaired 700 KB gate. Automatic approval review rejected initiating this root-helper installation without action-time confirmation; user approval/native administrator authorization is pending. After repair installation, verify authenticated root health, mismatched peers/version behavior and native removal before implementing the production helper. The previous portable candidate is unchanged.
