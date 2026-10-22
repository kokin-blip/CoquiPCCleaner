# CoquiPCCleaner

A local macOS investigation and maintenance app with separately approved, verified cleanup. **Tauri 2 + Svelte 5 + Vite + Rust + SQLite.** Targets Apple Silicon and macOS 14+.

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
- Local guided Ask Coqui, saved conversations, typed evidence/context and action handoffs.
- Explicit exact-duplicate and PNG/JPEG similarity review; separate personal-copy approval with retained keeper verification.
- Workload baselines, before/after comparisons, recorded manual startup/power plans and assumption-based upgrade guidance.
- Opt-in app-open read-only scan/reminder maintenance, qualified developer-resource/leftover/clone review and activity history.
- Rust-owned SQLite history and indexes with configurable retention and reviewed diagnostic-data removal.

The scanner remains read-only. Optimization can move individually approved eligible files in your Library/Caches, and separately reviewed supported project-output folders after fresh Git/worktree and descendant checks, and separately confirmed personal-folder duplicate copies with a freshly verified keeper, to macOS Trash. It does not permanently delete files, empty Trash, change startup/power/security/firmware settings, or estimate unavailable sensors. Modern login items can be incompletely observable. SMART/firmware fields depend on the model. Used memory alone is not a bottleneck, allocated file size is not guaranteed recoverable space, and APFS capacity must not be summed across shared volumes.

## Project context

Read PRODUCT.md, DESIGN.md and AGENTS.md. `docs/original-brief.md` preserves the request verbatim. `docs/roadmap.md` and `docs/phases/` retain later optimization, firmware, Windows and Intel work. `docs/phases/phase-2-progress.md` records milestones 2.1–2.4 and explicit Phase 2 extensions. `docs/status.md` records actual validation and remaining limits; `docs/figma.md` links the prototype and kit mapping; `docs/figma-*.json` records created design objects.

Project-aware eligibility in 0.2.1 distinguishes a parsed standalone home `package.json` from actual repository/workspace boundaries. Declared workspaces, cache-local projects and ambiguous configuration remain excluded; see [the eligibility rules](docs/phases/phase-2-progress.md#project-aware-user-cache-eligibility--021).

Version 0.3.0 adds Optimization → Developer artifacts: recognized outputs, indexed allocation, fresh Git status/tracked-file checks, worktree relationships, filters and saved evidence. It grants no project-removal approval. See [developer-artifact review](docs/phases/developer-artifacts.md). Native fixture: `./scripts/fixtures/developer.sh`.

Supported project-folder approval, verification, recovery and disposable fixture commands: [Phase 2.3](docs/phases/project-cleanup.md).

Version 0.6.0 remaining supported workflows, operation bounds, manual capabilities and persistence: [Phase 2.4](docs/phases/phase-2-workflows.md). Native disposable duplicate/keeper Trash verification (explicit, then restores its fixture):

```sh
cargo test --manifest-path engine/Cargo.toml --locked --test phase2 native_duplicate_trash -- --ignored
```

Phase 3.1 adds **Firmware & Security → Advanced audit**: individually select firmware inventory, readable security status and an optional macOS update check. Update checks require network consent each run and never download or install. The Guidance tab prepares separate Apple manual checklists with interruption/recovery disclosures; observations after a manual step remain qualified. Audits and plans are retained in History → Activity and follow Settings retention/purge. See [capabilities, compatibility and validation](docs/phases/phase-3-implementation.md).
