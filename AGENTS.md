# CoquiPCCleaner

Read PRODUCT.md, DESIGN.md, docs/architecture.md, docs/status.md and docs/roadmap.md before changes. The original request is preserved verbatim in docs/original-brief.md. Current development is Phase 2 on macOS 14+ Apple Silicon. The first milestone adds explicitly approved, verified Trash moves for individual user-cache files; see docs/phases/phase-2-progress.md.

Never fabricate device measurements or interpret unavailable data as healthy. The scanner remains read-only apart from its own Application Support data. The separate cleanup executor may move only explicitly reviewed and approved eligible cache files to native Trash. No deletion of inspected files, privilege escalation, startup changes, firmware changes, or security changes in Phase 1. Never run cleanup against real user files merely to test the app. Use disposable fixtures. Keep the TypeScript IPC contract aligned with Rust models.

Svelte owns presentation, application state and declarative navigation. Rust owns collection, validation, filesystem traversal, and SQLite. Keep platform code behind the macOS adapter. Run scripts/check.sh and update docs/status.md with actual results and remaining limitations. Keep the Phase 2, Phase 3, Windows and Intel specifications intact.
