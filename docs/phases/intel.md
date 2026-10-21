# Intel Mac compatibility follow-up

Phase 1 ships arm64 only. Intel support requires native x86_64 Rust artifacts, Tauri/Svelte universal packaging and an actual Intel-machine validation pass before release claims.

Audit GPU reporting, battery/thermal/SMART availability, firmware/boot security differences, macOS deployment support and platform parsers. Conventional PC BIOS tuning still does not become a generic Mac capability. Keep architecture-specific unavailable states explicit. Compare real monitoring readings and scan overhead on supported Intel models.
