# Phase 2.2 — developer-artifact analysis

Version 0.3.0 adds a working Developer artifacts tab in Optimization. It reviews recognized output directories from the selected retained scan and performs fresh, read-only Git checks. It does **not** authorize removal of project directories or artifacts. Existing individual-user-cache approval remains separate and unchanged.

## Recognition and scope

| Output pattern | Required evidence | Review consequence |
| --- | --- | --- |
| Rust `target` | Regular, non-linked parent `Cargo.toml` | Rebuild cost, build-script and generated-file review |
| Node `node_modules`, `.next`, `.nuxt`, `.svelte-kit`, `dist`, `build` | Regular, non-linked parent `package.json` | Package-manager restoration, local edits, offline dependencies and published assets |
| Dart/Flutter `.dart_tool`, `build` | Regular, non-linked parent `pubspec.yaml` | Resolution, rebuild and generated-source review |
| Python `.venv`, `venv`, `__pycache__` | `pyproject.toml`, `requirements.txt` or `setup.py` in at most 12 ancestors | Interpreter/environment reproducibility and local packages |
| Xcode-style `Library/Developer/Xcode/DerivedData` | Indexed path shape | Rebuild/reindex; project association explicitly unconfirmed |

These are named patterns with detected markers, not proof that a manifest is valid, an output is reproducible, or files are disposable. Custom output locations, arbitrary logs, Docker, emulators, archives and package-manager-native cleanup are not inferred. Nested dependency/output patterns are grouped under the first recognized outer artifact; bytes are not added twice. A wrong/missing/linked marker does not become a successful technology detection.

Sizes and file counts are SQL aggregates of non-directory entries under the exact indexed path prefix. They describe the scan's timestamp, not current filesystem allocation or guaranteed recoverable storage. Excluded, disconnected, replaced, non-owned, linked and cloud-only artifact directories are not reviewed. Index loss, partial source scans, query/command limits and inaccessible locations remain visible as coverage gaps. Only UTF-8 paths cross the UI boundary.

Discovery examines at most 2,000 output-like indexed directories; reports contain at most 250 outer artifacts. Before status, a no-follow metadata guard refuses cloud placeholders, unreadable/racing entries or traversal beyond 50,000 entries/three seconds in the checkout (and external common Git directory). These conditions leave Git state unavailable. Git commands have five-second timeouts and a two-MiB output cap; the overall loop has a two-minute budget checked between artifacts. SQL aggregate execution and an in-flight Git operation can extend cancellation latency. Limits produce partial coverage, not a claim of exhaustive developer storage.

## Git validation

The implementation uses `/usr/bin/git` with fixed read-only commands and literal pathspecs:

- `rev-parse` establishes the checkout root, absolute Git directory and shared common directory. A root inconsistent with the detected boundary remains unavailable. Broken `.git` files never become “not a repository.” Non-Git and possible bare repositories require manual inspection.
- `worktree list --porcelain -z` preserves paths with spaces/newlines and reports branch/HEAD or detached state, bare, locked, prunable and directory presence. The current checkout must match a present non-prunable registration. Related worktrees are listed, not inspected for dirty state. Shared common directories establish worktree relationships; separate clones are not inferred from remote URLs.
- `config --null --name-only --list` checks for content filters without retaining values. Repositories defining filters require manual inspection; `status` is not executed there, avoiding configured clean/filter processes.
- `status --porcelain=v1 -z --untracked-files=normal --ignore-submodules=all` records staged, modified, untracked-entry and conflict counts. A directory may be one untracked entry. Rename records and unusual filenames use NUL parsing. Ignored contents and recursive submodule dirty state are outside coverage.
- `ls-files --cached --stage -z -- <literal output path>` reports tracked index entries and gitlinks within the artifact. Any tracked entry remains protected, even if its name resembles a build output.

The child environment is cleared of inherited Git overrides. System/global Git configuration, fsmonitor, hooks, untracked-cache writes, optional index locks, terminal prompts and submodule recursion are disabled; no shell, build/package scripts, fetch, checkout, reset, worktree mutation, prune, cleanup or network command is available. Raw command outputs, file contents, manifest scripts, credentials and remote URLs are not persisted. Configuration names are inspected only for filter presence.

Git state unavailable/dirty, tracked contents, indexed nested repository/project markers, locked/prunable or enclosed worktrees, and unconfirmed commits produce **Protected**. Otherwise the result is **Review required**. Neither label grants cleanup eligibility; a clean status does not inspect ignored/untracked file contents, prove reproducibility, or establish recovery. Checks are point-in-time observations, and project state can change while commands run. Git results are reused within one analysis for outputs with the same detected project root; they are never reused for a new recheck.

Sources: [Git status porcelain](https://git-scm.com/docs/git-status), [worktree porcelain](https://git-scm.com/docs/git-worktree), [repository paths](https://git-scm.com/docs/git-rev-parse), [tracked entries and gitlinks](https://git-scm.com/docs/git-ls-files).

## Review, local persistence and cancellation

Check artifacts streams actual examined counts and partial results. Recheck performs fresh Git inspection against the same historical index; rescan refreshes sizes. Path, technology and assessment filters and 25-row pages support inspection. The detail dialog contains exact paths, timestamps, markers, Git/common directories, counts, worktree registrations, limitations and technology consequences. Finder reveal does not change the inspected data.

SQLite schema 2 migrates transactionally to 3 with `developer_reports`, one latest report per retained scan. Running reports become interrupted on reopening and are never resumed automatically. Source-scan expiry and explicit diagnostic-data removal delete associated developer reviews. Old scan summaries, preferences and cleanup action records are preserved. Version 0.2.x intentionally cannot open a newer schema; retain the new build when using that data directory.

Analysis shares the engine operation guard with scans, preview/cleanup and purge. Cancellation and channel loss stop remaining inspection and retain partial observations. App exit requests cancellation. The UI preserves an open detail view through unrelated live-metric updates. Transport failure never turns a running/incomplete observation into a successful check.

## Validation and remaining work

`./scripts/check.sh` includes real local Git fixtures, parser failures, dirty/staged/untracked/tracked outputs, gitlinks, nested projects, detached/locked/prunable linked worktrees, broken references, unborn commits, content filters, fsmonitor suppression, read-only index/config checks, exclusions, linked/missing paths, cancellation, migration/restart/purge/retention and stale indexes. UI tests cover saved reports, filtering, inspection, unknown state, cancellation, operation serialization and live-update stability. The API integration test scans a disposable project, streams analysis, reopens SQLite and clears app-owned records.

Create an isolated native validation fixture with `./scripts/fixtures/developer.sh`; select the printed folder in Settings, scan it, then open Optimization → Developer artifacts → Check artifacts. It creates only new disposable repositories and an attached worktree under `/private/tmp`; it never runs against existing projects. Native validation does not remove project outputs.

Separately approved project cleanup still needs per-technology manifests, untracked/ignored-content review, nested repository/worktree and dirty-state revalidation immediately before execution, interrupted-operation handling and verified recovery design. This milestone supplies evidence and review rather than bypassing that work. Full real-world repository configurations, huge/remote filesystems, complete VoiceOver and other supported hardware/OS combinations remain validation work.
