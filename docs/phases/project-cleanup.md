# Phase 2.3 — project output approval and verified Trash

Version 0.4.0 extends Developer artifacts with a separate, exact-folder review. Analysis itself grants no mutation authority. One supported folder may be reviewed per plan; nothing is preselected. The existing individual user-cache action remains available with its original eligibility policy.

## Supported rules

Every rule below also requires a complete source scan, the exact directory in its retained file index and developer report, fresh readable ownership/canonical-path checks, a clean confirmed Git checkout with a commit, zero tracked output entries/gitlinks, a Git-ignored output directory and no non-ignored untracked descendants.

| Technology | Folder | Required project evidence | Consequence |
| --- | --- | --- | --- |
| Rust | `target` immediately beneath its project marker | Tracked, user-owned, non-linked, bounded nonempty UTF-8 `Cargo.toml` | Rebuild/build-script cost; generated outputs can include useful local work |
| Node.js | `dist`, `build` | Valid object `package.json` with a nonempty `scripts.build`, plus a tracked text `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock` or `bun.lock` | Rebuild may need packages, services, credentials or original inputs; deployed assets may be valuable |
| Node.js | `.next`, `.nuxt`, `.svelte-kit` | Same manifest/lock policy, with `next`, `nuxt` or `@sveltejs/kit` declared in dependencies/devDependencies | Framework rebuild; nested manifests or links still prevent cleanup |
| Dart / Flutter | `build`, `.dart_tool` immediately beneath its marker | Tracked, bounded nonempty UTF-8 `pubspec.yaml` | Package resolution, regeneration and rebuild; offline artifacts may matter |
| Python | `__pycache__` | Tracked `pyproject.toml`, `requirements.txt` or `setup.py` in the recognized ancestry; only regular `.pyc` children, no nested directories | Bytecode regeneration; preserve the source and intended interpreter |

Tracked manifests and locks must be regular user-owned single-link files, at most 4 MiB, with matching metadata before/open/after a no-follow bounded read. Node JSON/declarations are parsed; other formats and lock semantics are not resolved or proven valid. No package/build script is run. These checks establish a narrow candidate policy, **not reproducibility**. The user must inspect every descendant and explicitly confirm that no unique work exists and that the folder can be regenerated.

Dependencies (`node_modules`), Python environments, Xcode DerivedData without confirmed project association, arbitrary/custom outputs, Docker, package-manager-native cleanup and whole projects/repositories/worktrees remain analysis only. A recognized pattern alone is insufficient. For example, generated nested `package.json` files or hard-linked build products can make a normally named output ineligible.

## Fresh inspection and approval binding

The metadata-only, no-follow folder traversal is bounded to 20,000 entries/five seconds per pass. Two matching passes are required. Every descendant must be a readable user-owned regular file or directory on the same device. Links, hard links, special files, cloud placeholders, unusual filenames that cannot cross the UTF-8 IPC boundary, exclusions, mount boundaries and incomplete/racing coverage prevent a plan. Nested repository, project, workspace and environment markers prevent removal. Files are not hashed and ignored file contents are never read or retained.

The existing bounded Git inspection checks root/Git/common directories, current dirty state and worktree registration. Locked, missing, bare, prunable, unborn or enclosed registered worktrees prevent cleanup. Related worktrees are checked for relationship hazards; their dirty state is not recursively inspected. Git metadata itself cannot be enclosed by the output. Content-filter configurations and unknown Git state remain unavailable/protected.

Additional fixed read-only commands use `check-ignore --no-index -- <exact folder>` and `ls-files --others --exclude-standard -z -- <literal output path>`. Git `check-ignore` accepts filenames and rejects Git's literal-pathspec magic, so only that command omits `--literal-pathspecs`; other inspection commands retain it. Fixed executable/environment, disabled hooks/fsmonitor/optional locks/lazy fetch, timeout and output limits remain in force. No shell, network, clean/reset/prune/worktree mutation or tool-generated cleanup command is exposed. See [Git check-ignore](https://git-scm.com/docs/git-check-ignore) and [Git ls-files](https://git-scm.com/docs/git-ls-files).

A persisted ten-minute plan binds folder/parent identity, the complete descendant metadata manifest, project manifests/locks and the Git/worktree observation. The approval dialog displays exact folder scope, freshly calculated regular-file logical/allocated totals, consequences and a paginated exact descendant list (50 per page). Approval remains disabled until contents load and the user acknowledges stopped editors/builds/watchers/tools and review/regenerability. Keep folder saves a declined plan and terminal declined/not-attempted items; no mutation occurs. The history UI qualifies legacy declined items that were recorded as pending without rewriting their historical evidence. An old history review cannot silently resume execution; create a fresh review.

## Execution, verification and recovery

Scans, developer checks, preview, execution, settings changes and local-data purge serialize through the engine guard. Preparation can be cancelled; a cancelled inspection does not create a review. Approval revision, expiry, acknowledgements and single use are enforced in Rust. Before the native call, the entire project eligibility and metadata seal are checked again; a changed folder, manifest, Git state, registration or exclusion prevents the move. Approval is journaled before any mutation and intent before native Trash. Cancellation/channel loss stops pending work. Once a native operation starts, its result still receives verification.

Foundation FileManager moves the whole approved directory to macOS Trash. Verification requires the source path to be absent, the original directory device/inode/size at the returned path, and the same ordered descendant identities/metadata in Trash. Only the root directory's change timestamp is allowed to differ after rename. Any discrepancy produces **unverified**, never a success claim. Native errors are retained; partial/unverified/interrupted operations are never retried automatically. On restart, an in-flight intent becomes unverified and pending work becomes cancelled.

These are point-in-time checks around a pathname-based native API. They cannot atomically lock other applications out or prevent a replacement during the native call. Verification observes metadata/identity, not byte hashes. Stop related tools first. Finder may offer Put Back while the item remains in Trash; Coqui does not overwrite replacements, offer guaranteed rollback, permanently delete anything or empty Trash. Moved allocation is not reclaimed capacity or a measured performance improvement; rescan for current storage observations.

## Local data and validation

Existing SQLite schema 3 stores the additive project flag and private inspection seals in the cleanup-plan JSON. Legacy file plans deserialize with project cleanup false and no project seal. The journal retains exact paths and metadata, not file contents, scripts or raw Git output. Existing configured retention and explicit diagnostic-data removal cover these records. The history dialog presents requested scope, consequences, outcome and available Trash reveal; interruption remains explicit.

`./scripts/check.sh` covers real disposable Git repositories, complete source/index membership, fresh descendant pagination, no-change/decline, stale file/folder/manifest/Git state, tracked/nonignored/unborn outputs, new nested markers/exclusions, linked/locked worktrees, broken/filter configurations, links/hard links/special/cloud/unreadable entries, supported technology rules, entry limits, cancellation/channel loss, intent/interruption, unverified native outcomes, restart/purge and legacy records. UI tests cover exact-folder scope, contents failure/retry, stopped-tool/regeneration acknowledgement, preparation cancellation, serialized scans, verified results and retained recovery details. The real engine API integration previews, reopens and declines a Git project plan.

Run `./scripts/fixtures/project-cleanup.sh` to create only new disposable Rust/Node repositories under `/private/tmp`. Select its printed scope for a native scan and review. Native mutation tests use only new fixture files/folders and restore them:

```sh
cargo test --manifest-path engine/Cargo.toml --locked --test cleanup --test project_cleanup -- --ignored
```

Large/remote filesystems, broader repository/tool configurations, macOS 14 and additional physical hardware, a complete VoiceOver walkthrough and Gatekeeper on another Mac still require validation. This milestone does not complete the full Phase 2 roadmap.
