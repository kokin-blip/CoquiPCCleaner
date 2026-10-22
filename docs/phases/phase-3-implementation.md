# Phase 3.1 — Apple Silicon platform audits

Direction contract: extend the existing Operate workspace and its system typography, neutral surfaces, mint actions and keyboard tabs. Preserve branding and the absence of repeated shield advisories. The firmware destination gains three task views: retained overview; explicitly scoped audit; separately acknowledged Apple guidance. The first viewport explains the saved observation time and gives the user control over each optional capability. The principal interaction is selecting an audit scope, watching real stages, then reviewing evidence and a manual guidance plan independently of cleanup. No new raster assets or visual world.

Success criteria: no optional collection on page entry; inventory/security/update selection is explicit per run; update network acknowledgement is enforced in Rust; cancel preserves partial observations; unavailable/custom/localized readings stay uncertain; compatible-model evidence and unsupported controls are findable; guidance acknowledgements do not execute changes or assert a verified reboot/installation. Results survive restart and participate in diagnostic retention/purge. Minimum 1100 × 720, dark/light, keyboard focus and scrolling use existing components.

Implementation and validation results are recorded below when verified. The original Phase 3 specification remains in phase-3.md.

## Capability and model matrix

| Platform / property | Adapter result | Validation boundary |
| --- | --- | --- |
| macOS 14+ arm64 target, reported Apple M-series chip and model | Eligible for individually requested inventory, security reads and update listing | Model-family/API match; property availability checked at runtime. New models are not automatically physically certified. |
| Mac16,8 / Apple M4 Pro / macOS 26.6.1 (25G76) | Local native validation | Only physical model exercised in this milestone. macOS 14 and other models need hardware coverage. |
| Missing model/chip/version, older OS, Intel or other targets | Advanced checks withheld, uncertainty disclosed | Intel/Windows remain separate adapter and release work. |
| System firmware and OS loader | Selected `SPHardwareDataType` fields | Version only; no attestation or authoritative latest-version map. |
| SIP / authenticated-root / FileVault | Strict recognized public utility statuses, independently qualified | Unknown/custom/localized statuses are not enabled. Permission failures remain requires-permission. Encryption transitions are not completion. |
| Full / Reduced startup security | Manual only in Recovery for intended disk | Never inferred from SIP, authenticated-root or firmware version. |
| Update availability | macOS software offerings at response time, on explicit network opt-in | Not a standalone firmware catalog; managed/deferred offerings, connectivity and future changes can affect results. |
| PC BIOS / XMP / EXPO / firmware password | Unsupported by this Apple Silicon adapter | No pretend tuning controls. |
| Firmware installation, policy change, Revive/Restore | Apple-managed/manual guidance only | No writer, privileged helper, install, reboot, security change or DFU operation in Coqui. |

## Collection and data contract

`FirmwareRequest` selects inventory, security and updates separately; all begin unchecked on page entry. A storage scan is not required. Baseline identification uses literal `system_profiler -json -detailLevel mini SPHardwareDataType`, `sw_vers -productVersion` and `sw_vers -buildVersion`. Inventory retains only selected version fields. Security reads use `csrutil status`, `csrutil authenticated-root status` and `fdesetup status`. Update listing uses only `/usr/sbin/softwareupdate --list` after Rust validates `networkAcknowledged`. There are no caller-supplied executable paths, arguments, labels to install, shell strings or credentials.

Each process uses fixed paths/arguments, C locale, sanitized environment, null stdin, nonblocking pipes, a 1 MiB cap per stdout/stderr and pollable cancellation. Hardware timeout is 35 seconds; version/build/status commands are four seconds each; update check is at most 120 seconds. Whole-run time includes sequential identification and selected collectors. A cancelled client can leave the OS update service completing its own work; macOS owns catalog/log changes. Coqui requests no download/install or reboot. Updates never join maintenance or cleanup. Unrecognized/incomplete listings remain unknown/partial; only an exact recognized “none offered” result is shown as such, with its response time. No notifications/background checks are silently enabled.

`FirmwareReport`, `PlatformCompatibility`, `FirmwareUpdate` and `FirmwarePlan` are Rust-owned generated TS models. Tauri streams real stages and terminal status. Reports retain sources, timestamps, scope, selected evidence, offered software, findings and errors. Raw output, serials and hardware UUIDs are discarded. A capability match describes the adapter, not a positive reading of an unavailable property. Active work shares the existing engine guard, cancellation and cross-navigation banner. SQLite schema 4 records cover audits/plans without changing existing settings or source scans. Restart recovers running audits as interrupted; completed/partial/cancelled audits remain inspectable. All records follow the configured diagnostic retention and purge.

## Separate guidance and outcomes

Flow H is overview → optional readings → findings/evidence → specific Apple manual guidance. Plans require an eligible finished audit under 24 hours old, identify the exact kind (Software Update, startup policy inspection or recovery requirements), and retain previous observations, source/review date, steps, cautions and recovery limits. A plan requires its own acknowledgement within 20 minutes; decline changes only the local record. No guidance text authorizes native mutation. The acknowledged handoff opens System Settings; navigate General → Software Update if the deep link is not honored (the validation host landed on General). Opening Software Update may itself check Apple's network; installation decisions remain entirely in Apple's UI. Merely reading the current linked Apple source is always available.

After a user-reported manual step, a separately requested read collects inventory/security, without another network update check. A fresh report and before/after OS build are retained; final plan status is **unverified**, never “firmware updated”, “reboot successful” or “Full Security enabled”. Cancellation leaves the manual outcome pending with retry guidance. Coqui cannot establish exact security policy, causal changes, a completed restart, integrity, rollback or successful recovery from these readings. It does not initiate a system operation during testing.

## Sources and freshness

Bundled guidance was reviewed **2026-10-22**, independently of each device reading/listing timestamp. It is not fetched or refreshed automatically and can become stale; current allowlisted Apple sources are linked in the app. No latest-macOS number is hardcoded.

- [Apple: Update macOS](https://support.apple.com/en-us/108382) — compatible offerings and the supported installation interface.
- [Apple Platform Security: startup disk policy](https://support.apple.com/guide/security/startup-disk-security-policy-control-sec7d92dc49f/web) — per-disk policy and user authorization.
- [Apple: firmware recovery](https://support.apple.com/en-us/108900) — model-specific requirements; distinguishes Revive and erasing Restore.
- [Apple: firmware passwords](https://support.apple.com/en-us/102384) — Intel applicability; not an Apple Silicon control.
- Local `/usr/sbin/softwareupdate --help`, checked 2026-10-22, confirms the `--list` read operation. No undocumented firmware-setting API is used.

## Deliberate next boundaries

No firmware modification is implemented or offered as an inactive action. Adding one requires a tested vendor/OS-supported API, exact model/OS eligibility, independent high-risk consent, interruption/reboot constraints and supported recovery, separate from any routine batch. Additional physical models/OS versions, managed-device scenarios, full VoiceOver testing and another-Mac distribution validation remain follow-up work. Windows and Intel specifications remain intact.

## Measured acceptance — 2026-10-22

The full check script passed after parser/layout corrections: 42 frontend and 96 Rust tests (138 passed), no Svelte/TypeScript errors or warnings, production build, Rust format and locked native check. Final UI copy/version corrections passed production build, all frontend tests and native check again. Eleven Rust unit tests and five UI tests are new for Phase 3, plus real API-flow coverage. Three existing opt-in native Trash fixture tests remain outside the standard suite; no new mutation test is needed for these read operations.

On Mac16,8/Apple M4 Pro / macOS 26.6.1 build 25G76, actual system firmware/loader 18000.161.9 matched native selected-field inventory. SIP/authenticated-root/FileVault read as enabled. The first audit truthfully retained an unfamiliar punctuation variant as unknown; native comparison and a regression test corrected exact authenticated-root parsing, then a fresh audit completed. Full/Reduced policy stayed manual. One consented native update query produced four software offerings, with no install/download requested. Results/guidance reopened after app restart; acknowledgement and recording no manual action affected only Coqui records. Inspection of the app's retained records found no serial/UUID raw keys and confirmed System appearance restored. The Settings handoff reached General, so the UI names System Settings and supplies the General → Software Update fallback.

Native dark/light captures, reference-width and 1100 × 720 views and the supplied UI source received an independent **ship** disposition at that scope. An earlier conflicting selected navigation capture cleared after final relaunch; its cause is undiagnosed. This is not a full theme-transition, VoiceOver or security audit. The design handoff records the precise capture set and inherited-system comparison. Figma frame IDs are in ../figma-phase-3.json; they are illustrative editable structures, not native pixel captures.

The final local 0.6.0 release archive is 8,339,402 bytes, SHA-256 `30bdc7fdc17253ce9ec8c6c0ac10643b9cd7af2d496773ac27ac76a95fef0ea3`. Re-extraction verified arm64, minimum macOS 14 and strict ad hoc signature; the final app launched and reports 0.6.0. Another-Mac Gatekeeper, macOS 14 and additional models remain unvalidated. Nothing was uploaded or published.
