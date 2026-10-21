# Decisions

- Original product brief retained verbatim in original-brief.md.
- Phase 1 is read-only, Apple Silicon macOS 14+, local data, manual scan consent.
- User selected home-folder scans plus an explicit deep-scan mode. External/network roots require separate folder selection.
- User selected new Coqui pages in the existing Figma kit.
- User selected direct distribution without Developer ID signing/notarization; ad hoc signing is allowed.
- During implementation the user explicitly changed the UI stack from Flutter to Tauri. Current stack is Tauri 2 + Svelte/TypeScript + Rust + SQLite. Flutter, Dart, Riverpod, go_router and Flutter Rust Bridge are superseded, although their original requirements remain in the verbatim brief.

- User clarified the frontend as Svelte + Vite; no React implementation is retained.
- Native diagnostics use typed Tauri commands/channels and Rust-generated TypeScript models; bridge regeneration is `npm run types`.
- Command Line Tools are selected per build process, avoiding a global Xcode selection change. Apple Silicon release builds succeeded without full Xcode.
- A failed or unexpectedly ended monitoring stream clears current readings and retains a stopped/retry state; a cached measurement never establishes liveness.
- Figma deliverables are editable structural handoff views and linked prototypes. Native screenshots and real behavior are authoritative for the implemented result; exhaustive visual/state parity remains documented acceptance work.
