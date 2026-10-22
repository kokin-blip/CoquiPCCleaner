# CoquiPCCleaner

<!-- impeccable:product-schema 1 -->

## Platform
macOS desktop first; Windows deferred until the first completed macOS build.

## Stack
Tauri 2, Svelte/TypeScript, Vite; Rust-owned SQLite. User selected Tauri + Svelte + Vite during implementation.

## Users
People investigating slow computers, limited storage, and workload constraints, including developers with large tool caches. Both ordinary and technical users need understandable evidence and deeper inspection.

## Product Purpose
Understand the machine, diagnose observed problems, explain options, obtain consent, and verify outcomes. A useful result can be that no changes are warranted.

## Positioning
Workload-aware analysis and safe developer-storage maintenance grounded in real measurements rather than manufactured health scores.

## Operating Context
Current development is Phase 2, milestone 2.4 (0.5.0): local investigations, exact duplicates with keeper-preserving approval, measured comparisons, manual settings plans, workload scenarios, maintenance and broader resource review, building on verified cache/project-output cleanup on Apple Silicon macOS 14+. Standard storage analysis covers the accessible home folder; optional deep analysis expands to accessible internal locations. Monitoring runs only while the app is open. Local direct-download builds require no Developer ID account or notarization.

## Capabilities and Constraints
Phase 1 is read-only: inventory, live CPU/memory monitoring, storage exploration, cache candidates, startup detection, available disk health and platform/security information. Protected and unavailable data must be disclosed. Phase 2.1 adds separately approved native Trash moves for individual eligible user-cache files, persistent action records, verification and manual recovery guidance. Phase 2.2 adds developer-output review with Git/worktree evidence. Phase 2.3 adds separate exact-folder approval, fresh descendant/Git checks and verified native Trash for the supported outputs in docs/phases/project-cleanup.md; dependencies, environments and unassociated Xcode data remain analysis only. Phase 2.4 adds local guided Ask Coqui, bounded exact-copy/PNG-JPEG similarity analysis, separately approved personal-copy Trash moves with fresh keeper verification, baselines/comparisons, recorded manual startup/power plans, assumption-based workload advice, app-open scan/reminder maintenance, qualified leftovers/clone relationships and retained activity. See docs/phases/phase-2-workflows.md for actual limits. Optional external AI, native-manager resource execution, automatic settings changes and closed-app scheduling remain explicit extensions. No cloud AI, telemetry, login, or synchronization in Phase 1.

## Brand Commitments
Native macOS composition using the supplied Apple Music macOS Figma kit and its other application families, adapted to Coqui semantics. Preserve original kit content and add Coqui pages.

## Evidence on Hand
Original brief: docs/original-brief.md. Figma file: Gz7WEiN8Y3Vvivvae0ZPlP. Empty repository at implementation start. Runtime values must come from the engine; design specimens must be labeled illustrative.

## Product Principles
- Observations and hypotheses are different.
- Missing information is never a healthy reading.
- Workload relevance outranks headline specifications.
- Consent, scope, and recoverability must be explicit.
- Local data stays under the user's control.

## Accessibility & Inclusion
Keyboard navigation, visible focus, VoiceOver labels, readable contrast, reduced motion, accessible alternatives to visual charts, and resizable desktop layouts.
