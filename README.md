# CoquiPCCleaner

A macOS system analyzer with separately approved cache-file cleanup. **Tauri 2 + Svelte 5 + Vite + Rust + SQLite.** Targets Apple Silicon and macOS 14+.

## Run

Install Node 24 LTS, Rust 1.97.1, and the macOS Command Line Tools. Full Xcode is not required for the current Tauri implementation. The scripts select Command Line Tools per process without changing your system developer directory.

```sh
npm ci
./scripts/dev.sh
```

`npm run dev` opens only the frontend development server; native diagnostics require the Tauri desktop app. A browser preview deliberately does not fabricate system data.

## Check and package

```sh
./scripts/check.sh
./scripts/release.sh
```

The release script builds `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app`, verifies its ad hoc signature, and creates `release/CoquiPCCleaner-macOS-arm64.zip` with a SHA-256 checksum. No Developer ID identity, notarization, account or release upload is required. macOS may require explicit user approval when opening a downloaded, unnotarized build.

Regenerate TypeScript domain models after Rust model changes:

```sh
npm run types
```

## What works

- Onboarding, workload/preferences, light/dark/system appearance, local settings.
- Real hardware, disks/volumes, application and readable startup configuration inventory.
- Two-second CPU, memory, swap and readable process I/O monitoring while open.
- Standard home-folder scans, confirmed deep scans, explicit additional folders and exclusions.
- Metadata-only storage traversal, category treemap, large files, folder drill-down, cache candidates and Finder reveal.
- Capability disclosures, scan/cancel/history, permission guidance and user-chosen report export.
- Exact-path cache-file review, expiring approval, native Trash moves, cancellation, verification and action history.
- Rust-owned SQLite history and indexes with configurable retention and reviewed diagnostic-data removal.

The scanner remains read-only. Optimization can move individually approved eligible files in your Library/Caches to macOS Trash. It does not permanently delete files, empty Trash, change startup/power/security/firmware settings, or estimate unavailable sensors. Modern login items can be incompletely observable. SMART/firmware fields depend on the model. Used memory alone is not a bottleneck, allocated file size is not guaranteed recoverable space, and APFS capacity must not be summed across shared volumes.

## Project context

Read PRODUCT.md, DESIGN.md and AGENTS.md. `docs/original-brief.md` preserves the request verbatim. `docs/roadmap.md` and `docs/phases/` retain later optimization, firmware, Windows and Intel work. `docs/phases/phase-2-progress.md` records the completed 2.1 milestone and remaining Phase 2 work. `docs/status.md` records actual validation and remaining limits; `docs/figma.md` links the prototype and kit mapping; `docs/figma-*.json` records created design objects.
