# Windows follow-up

Begin after the first completed macOS build. Implement a Rust Windows adapter and adapt window chrome/platform affordances. Retain shared domain models, Tauri/Svelte components, SQLite migrations and capability states.

Collect Windows inventory, startup mechanisms, supported power controls, CPU/memory/GPU/disk measurements and Windows-specific cleanup candidates. Expand BIOS/UEFI version, motherboard/vendor information, Secure Boot, TPM, virtualization and XMP/EXPO only where supported APIs expose them. Manufacturer-specific audit/update guidance belongs behind explicit capability checks.

Test ordinary user/admin distinctions, permission denial, NTFS links/reparse points, volume accounting, locked files, native resource tools, cancellation, approval/revalidation and recovery. No aggressive registry cleaning, silent security changes or essential-service disabling.
