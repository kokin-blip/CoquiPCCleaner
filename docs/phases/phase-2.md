# Phase 2 — Optimization Engine

## Product outcome
Turn observed findings into workload-specific, reviewable plans. Support a legitimate no-change recommendation. No generic Optimize Everything action.

## Features retained
- Ask Coqui: history, conversation, context inspector; clarification → typed diagnostic tasks → observations/possible or confirmed causes → proposed actions → approval → verification. An optional external model requires explicit opt-in and clear data disclosure. Never invent readings or executed actions.
- Cleanup: scan → select → exact-path preview → confirm → execute → verify/report. Distinguish Trash, regenerable caches, permanent deletion and application-native resource removal. Preserve failures, per-item permission guidance, retry/skip and supported recovery only.
- Exact duplicate detection with content verification; similar-photo candidates and modified copies remain distinct. Previews, intentional-copy warnings and preservation of at least one confirmed copy.
- Developer storage: Git repositories/worktrees/clones, Xcode DerivedData, Dart/Flutter, Rust target, node dependencies, Python environments, Docker resources, emulators, generated artifacts and logs. Technology-specific consequences. Inspect repository relationships, dirty/untracked state and worktree registration before any removal. Never decide disposability from age or size alone.
- Application leftovers and organization advice with evidence. Never move user documents automatically.
- Supported startup adjustments, power preference plans (Balanced/Performance/Efficiency), baselines and before/after comparisons. No unrelated security, services or registry changes.
- Workload suitability and upgrade advice: distinguish wear, workload sufficiency, OS support and upgradeability. Scenario/range forecasts with assumptions, compatibility confidence and sourced costs where available. No upgrade can be the correct answer.
- Maintenance: scheduled scans, alerts, developer-artifact reminders and exclusions. Initially scans/reminders only; manual cleanup approval. Scheduled deletion requires separate per-rule scope authorization in any later extension.
- History/recovery: previous state, requested/actual change, timestamp, verification, logs and real recovery availability. Never imply Trash or every operation guarantees rollback.

## Design and interfaces
Implement the original flows B–G and recommendation/action/approval/execution/recovery models. Reuse Messages, Notes, Numbers, Reminders, Calendar, Mail, App Store and iMovie kit patterns. Keep explanation and command authorization separate. Every action must revalidate eligibility and exact targets immediately before execution.

## Acceptance
Test stale approvals/changed targets, dirty repositories, permission denial, partial failure, cancellation, native-tool failures, unavailable reversal, absent baseline and declined actions. No external AI/network access without explicit consent.
