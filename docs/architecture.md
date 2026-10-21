# Architecture and data contract

Svelte renders engine data and collects scan/permission choices. A shared Svelte store owns application state and the bounded metrics buffer. Typed Tauri commands/channels handle IPC. Rust owns SQLite and all filesystem/diagnostic work. Tauri dialog APIs handle native folder selection. Narrow Rust commands handle Finder reveal and allowlisted System Settings destinations.

Rust modules: model (bridge-safe records), platform (allowlisted read-only macOS collectors), storage (metadata walker), database (migrations/queries), api (engine lifecycle and scan streams). Process execution uses fixed executable paths, argument arrays, bounded timeouts, and no shell interpolation. No privileges are requested by collectors.

Capabilities: supported, requiresPermission, manualOnly, unsupported, unknown, temporarilyUnavailable. Observations carry source/time/units/qualification. Scan states: running, complete, partial, cancelled, failed. Cancellation preserves partial results, and interrupted persisted runs are marked interrupted on startup. Missing collections do not erase successful collections.

Storage uses lstat, never follows symlinks, skips cloud placeholders and mount boundaries, deduplicates roots and hard links, and distinguishes logical/allocated bytes from filesystem capacity. Deep scan uses the Data volume once rather than double-scanning firmlinked home folders. No content reads, hashing or modification of inspected data. Cache rules explain potential future cleanup; bytes are not promised recoverable.

SQLite is stored in ~/Library/Application Support/CoquiPCCleaner. Schema migrations run in transactions. Preferences, snapshots, evidence/findings and scan summaries persist locally; file inventory is replaced per successfully completed scope. Failed or cancelled scans remain inspectable without replacing a previous complete scope index. History defaults to 30 days. No raw file contents, serial numbers or command dumps in routine logging.

No execution/action API is exposed in Phase 1. Future approval-bearing commands must be designed separately rather than extending conversational text into executable instructions.
