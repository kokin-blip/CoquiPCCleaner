# Advice references and verification limits — 0.11.1

The user explicitly **skipped** comparing macOS/RAM with requirements for Xcode, Visual Studio Code and Docker Desktop. Do not infer authorization for that comparison from the earlier proposed scope. The approved changes are vendor sources with review dates, and clear statements of what Coqui cannot verify. This patch extends the existing workload-advice view and its retained History view; it introduces no software-requirements catalog or checker.

## Reference provenance

Rust stores a publisher, title, source URL, reference-content review date and scope for each existing advisor link. The existing URL list remains compatible with older reports. New fields default to empty when reading legacy records. Legacy reports show that the publisher/review date and structured limits were not recorded; the app does not attach today's date or the report creation time to them.

Reference content was checked during development on **2026-10-23**, America/Phoenix. This is not a live per-assessment lookup, a device-specific compatibility check or a vendor publication date. No automatic network request or source refresh was added. The existing allowlisted browser-link action remains explicit.

| Vendor source | Reference scope |
| --- | --- |
| [Apple: power modes](https://support.apple.com/en-us/101613) | Mode behavior and model-specific availability; the app does not establish that a mode is offered/enabled on this Mac. |
| [Apple: Mac mini memory](https://support.apple.com/en-ie/102328) | Mac mini guidance, not universal Mac upgrade/service eligibility. |
| [Apple: Ventura upgrade guide](https://support.apple.com/macos/upgrade) | The page currently describes Ventura; it does not establish eligibility for a current or future release. |
| [Apple: Mac comparison](https://www.apple.com/mac/compare/) | Vendor comparison; configurations, price, availability and compatibility are not imported into Coqui. |

New reports retain explicit limits for exact software/peripheral requirements, model-specific upgrade feasibility and benefit, current quotes, future support/lifespan and performance/causality. Existing measured evidence and scenarios are unchanged. Sharing one presentation component keeps current and retained reports consistent. Schema 7, helper protocol, cleanup authority, task timing and saved preferences remain unchanged.

## Interface and validation

This is a local hardening change within the existing Operate workspace. System type, shared semantic colors, flat panels, native controls, source-link styling, focus and reduced-motion behavior are inherited. Source text wraps inside the existing panel/dialog. No new tokens, identity, images, tabs or requirements form were added; existing DESIGN/sidecar drift is outside this patch.

See [status](../status.md) for actual checks, artifacts and installed acceptance. Static source review dates must not be presented as current source freshness or as a successful compatibility evaluation.
