# Original plan completion work

The user requested the onboarding repair first, followed by the missing features identified in [the original-plan audit](../original-plan-audit.md). This tracks implementation after that audit. The original brief and platform/phase specifications remain intact.

## Implemented in 0.10.0

- **First-run scope review:** Onboarding requires the shell's scan callback. Whole-disk setup reviews mounted internal volumes before collection; cancel does not scan. Standard scope retains its direct path. App-level regression tests reproduce the old failure and verify the corrected paths.
- **Dashboard troubleshooting:** a composer and suggested problems open Ask Coqui and submit the draft once. Questions never silently start collection. Workload review, prioritized next steps and retained activity supplement existing measurements/findings.
- **Shared recommendation plan:** Rust reads the selected saved scan, preference/workloads and an explicitly selected completed baseline. Typed recommendations include relevance, benefit, confidence, risk, reversibility, permission, restart and verification. Review controls open the relevant tab. Presentation is read-only; only Save plan to History creates a retained review record. A plan grants no execution authority.
- **Workload and bottleneck review:** matching baseline workloads keep their original timestamps. CPU at or above 90% in at least half the samples is a transparent review threshold. Swap growth is a memory investigation lead, not pressure. Low capacity is not a performance cause; low average CPU is not a suitability verdict. Preference changes review order without changing observations. Missing or unrelated baselines require measurement.
- **Fresh Ask Coqui tasks:** explicit typed selections capture a 6–600-second baseline, recheck indexed developer/Git evidence or review indexed resources. Real stages stream into the conversation. Existing guards, exclusions, bounds, cancellation and interruption handling apply. A captured baseline links to its investigation; unrelated latest baselines are no longer silently adopted. Starting a baseline replaces the stale missing-baseline review task; a later review needs a complete interval with at least three samples before it can say complete. Text cannot select arbitrary commands or perform changes.
- **Similar-photo inspection:** explicit PNG/JPEG preview accepts only a path in the selected duplicate report. Rust repeats no-follow/ownership/link/cloud/exclusion checks, bounds input to 32 MiB and decoding to 12,000px/96 MiB, checks source stability and returns a maximum 512px PNG. Pixels are not retained. UI object URLs are revoked on close/unmount. Only the image CSP gains `blob:`; script/network/execution policy is unchanged. Preview never renews equality or cleanup authority.
- **Legacy state wording:** terminal scans lacking a finish time say Finish time not recorded. No historical timestamp is invented.

## Persistence and compatibility

SQLite stays at schema 7. Recommendation plans use typed `records` and normal retention/purge; no migration or read-time housekeeping is added. `Investigation.baselineId` is additive, nullable and defaulted for old records. Generated TypeScript matches Rust. The helper protocol, allowed removal namespaces, cache/project/keeper checks and administrator-session authorization are unchanged. Rebuilt app/helper binaries still require exact installed-build pairing and must be installed together.

## Validation

The first complete check passed with **77 frontend +153 Rust =230 tests**, zero Svelte/TypeScript errors/warnings, production frontend build, formatting and locked native check. Four native mutation tests stayed ignored. Tests cover onboarding scope/cancel/standard choice, plan read purity/save/reopen, unrelated baseline rejection, preference ordering without execution, legacy data/state, one-time draft submission, real streamed baseline capture, channel loss/cancel/error retention and preview scope/exclusion/link/source/record invariants.

Disk pressure required compact task-owned test output and the existing native check cache. A temporary cargo dispatch wrapper ran unmodified `scripts/check.sh`, removing the task's compact target before the final native check. A generated native incremental compiler cache was removed for headroom. No source, release archive, user document or application-cleanup target was removed. Disk exhaustion truncated one task-created type-export executable; that binary/fingerprint was removed and export succeeded on retry.

After native review found the stale baseline task, the refreshed check script passed the same **230 tests**, formatting, frontend build and native check. Its Rust test invocation used the cached **release** profile to fit the available disk; this is disclosed rather than presented as a second default-profile run. A separate final eight-test recommendation/investigation run also passed after the progress label was clarified to **Expected result**. The failed compact refresh target, executed test-only binaries and newly generated native incremental cache were removed; release/helper libraries were preserved.

Browser captures use an ignored, labeled illustrative harness without filesystem/IPC access. Dashboard, plan and investigation screens were inspected in light/dark and at 1100×720. These do not establish installed collection/helper authentication, full VoiceOver or disk completion. Final artifact/native results are in [status](../status.md).

The first 0.10 build passed matching app/helper authentication, native plan/save/reopen, actual three-sample capture, cancellation, disposable photo previews and restart. Native review found a stale investigation task, which was corrected and regression-tested. The final rebuilt app/helper are now installed with verified exact pairing and successful native authentication; a fresh three-sample capture over 6.3 seconds replaced the stale task and preserved the corrected context when reopened. Administrator features remain off. See status for first/final build pins and each check's scope. Full-scale disk and broader hardware/accessibility acceptance remain open.

## Remaining work in order

| Work | Requirement or dependency |
| --- | --- |
| Full-scale whole-disk finalization/import | Still partial until a fresh installed run succeeds. Low headroom is material to a multi-million-row index. Do not remove real user data for testing or infer completion from fixtures. |
| Deeper workload/upgrade advice | Installed 0.11 adds user-operated task intervals, repeated-trial comparisons and explicit task evidence; native completion/cancellation/save/reopen passed within [task-trial scope](task-trials.md). Automated application benchmarks, independent compatibility/pricing, hardware life and future OS support remain unfinished. Measured intervals do not establish those claims. |
| Startup/power execution | Still manual. A supported executor needs exact model/OS applicability, separately reviewed changes, actual prior-state verification and supported recovery. No undocumented writes or generic login-item manager. |
| Developer-native resources | Docker, emulator/device contents, dependencies/environments, Xcode associations and repository/worktree removal need manager-specific contracts. No folder-name or clean-Git shortcut. |
| Broader photos/duplicates | PNG/JPEG previews are present; HEIC/RAW/video, crop/rotation matching, exhaustive libraries and larger scans need bounded decoders/indexing and validation. Similarity remains advisory. |
| Usage/organization | Suggestions remain qualified rules/evidence. Richer contextual onboarding, representative application requirements and usage-based organization remain work. No continuous tracking or automatic document moves. |
| Scheduling | App-open read-only maintenance exists. Closed-app scheduling and scheduled deletion are not added. Any extension needs per-rule consent/lifecycle checks; firmware reminders must never collect or contact Apple. |
| Native design/distribution acceptance | Figma handoffs, full VoiceOver/contrast/state coverage, physical macOS14/additional Macs, APFS/external layouts, Gatekeeper and independent helper security review. |
| Windows/Intel | Retained future adapters and hardware/release validation. No PC BIOS/TPM/XMP or unsupported architecture claims. |
| Firmware mutation | Unsupported by policy; Apple's tools own install/reboot/Recovery/security changes. This work does not authorize a firmware writer. |

This release completes the first implementation batch, not the entire original product or whole-disk acceptance.
