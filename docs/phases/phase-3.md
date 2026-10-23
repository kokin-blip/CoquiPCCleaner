# Phase 3 — Advanced Firmware Diagnostics

Expand manufacturer/model identification, platform firmware inventory, supported configuration auditing and firmware update guidance. Apple Silicon surfaces Mac firmware and supported system security rather than PC BIOS, XMP/EXPO or TPM tuning controls.

Each advanced capability is individually enabled. Default stays read-only. Provide model/API compatibility evidence and label restricted or unavailable properties. Prefer manufacturer/OS-supported update paths and guided manual configuration.

Any supported firmware modification is a separate operation with additional explicit authorization, compatibility/eligibility checks, interruption/reboot warnings and supported recovery guidance. It never joins routine cleanup or power-profile batches. Do not invent vendor APIs or promise modification support before a tested implementation exists.

Acceptance: supported/unsupported model matrix, permission failures, update-source freshness, manual-only actions, security status uncertainty and truthful reboot/recovery states. Preserve original Flow H.

Milestone 3.1 implementation and measured acceptance results: [phase-3-implementation.md](phase-3-implementation.md). The specification above remains the full Phase 3 contract; no firmware-modification support is claimed.

Milestone 3.2 (0.7.0) adds the remaining supported macOS review workflows: [phase-3-completion.md](phase-3-completion.md). The original specification above remains intact. Supported diagnostic/review scope is implemented; firmware writers, automated installation/reboot/Recovery and untested vendor-specific operations remain unsupported rather than inactive controls.
