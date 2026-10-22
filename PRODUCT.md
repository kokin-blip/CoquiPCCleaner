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
Current development is Phase 2, milestone 2.2 (0.3.0): developer-artifact analysis and Git/worktree validation, alongside the existing cache approval and verification on Apple Silicon macOS 14+. Standard storage analysis covers the accessible home folder; optional deep analysis expands to accessible internal locations. Monitoring runs only while the app is open. Local direct-download builds require no Developer ID account or notarization.

## Capabilities and Constraints
Phase 1 is read-only: inventory, live CPU/memory monitoring, storage exploration, cache candidates, startup detection, available disk health and platform/security information. Protected and unavailable data must be disclosed. Phase 2.1 adds separately approved native Trash moves for individual eligible user-cache files, persistent action records, verification and manual recovery guidance. Phase 2.2 adds read-only developer-output review with Git/worktree evidence; project-artifact removal is not enabled. Other Phase 2 features remain in the roadmap. No cloud AI, telemetry, login, or synchronization in Phase 1.

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
