# Storage map and folder footprints — 0.9

## User direction and product boundary

The user asked for Disk Inventory X's storage system adapted to Coqui, with intelligent identification of folders whose small files add up to substantial space. They confirmed **folder treemap beside ranked folders**, retaining Large files as a separate view. [The REA investigation](../research/disk-inventory-x.md) distinguishes reference observations from inferred internals.

This extends the existing Storage screen, shell, typography, flat surfaces, mint accent, light/dark/system appearance and 1100 × 720 minimum. It is an Operate surface inside the established visual system. No new branding, repeated shield advisories or reference-app gradient artwork is introduced.

## Interaction contract

- Explorer presents cumulative immediate-child folder/file sizes in a ranked list beside a proportional treemap. Selecting/drilling into a folder updates both; breadcrumbs, parent and scan-root actions provide a return path.
- Allocated and logical size are explicit map/ranking choices. A sparse VM can have a much larger logical size than allocated footprint. Neither metric guarantees unique recoverable APFS bytes.
- The map requests a bounded set of up to 60 immediate children, independently of paginated file-table results. Remaining measured children have an explicit remainder region/label; a page or top-N set must not be normalized and described as the whole folder.
- Stable colors, hover/focus context and keyboard buttons connect map and ranked contents. The accessible list/table remains usable when rectangles are too small. Exact paths and qualifications stay inspectable.
- Counts and size evidence explain many-small-file concentration, rather than merely sorting individual files. Unavailable legacy counts remain unavailable, not zero. Partial traversal, interrupted aggregation/import, omitted locations, stale responses, loading, empty and retry states stay explicit.
- Large files, cache review, coverage and removal/restore records remain working destinations. Recognition adds no execution authority. Every removal retains existing independently revalidated preview, exact-path approval and native Trash/verification requirements.

## Accounting and typed boundary

Rust calculates and persists descendant file counts, small-file counts, the allocated size of small files and the largest allocated file in each folder. "Small" means **logical size at most 64 KiB**; a 64 KiB threshold is a transparent grouping convention, not a disposability test. Direct files and child-folder aggregates are accumulated exactly once per retained filesystem identity. Directory rows contain descendant totals; do not sum every directory row to obtain disk usage.

SQLite schema 7 adds folder statistics alongside existing file rows, retaining schema-6 summaries/settings/history. Old scans can show indexed sizes, but missing statistics stay nullable. Typed `StorageMapQuery`, `StorageMapNode` and `StorageMap` models cover bounded queries and explicit remainder accounting through Tauri and the paired helper. `FileEntry` and cleanup authority remain unchanged. New helper operations are read-only and retain exact-build pairing/version checks; the installed 0.8 helper cannot be assumed to implement the new contract.

Folder recognition uses explicit context and measured evidence, and reports its reason and consequences. A familiar folder name is a hypothesis about purpose, never proof of ownership or safe removal. Fresh Git/worktree checks remain the separate developer-cleanup engine's responsibility. Metadata analysis must not execute package scripts, build tools, Git hooks or dependency managers, read arbitrary file contents, or download cloud placeholders.

Finalization must keep status/read/cancel requests responsive while storing aggregates and applying retention. Cancellation must be honored in folder aggregation and every result-import loop. A database or helper error must preserve observed counts, terminal state and concrete safe diagnostics instead of implying successful completion. Large retained indexes need bounded memory and disk-headroom handling as well as responsive UI.

## Similar scenarios and interpretation backlog

The list below is a product analysis map. It is not a claim that every technology has a cleanup executor or a dedicated recognition rule in this release. Implemented rules and checks are recorded in the validation checkpoint.

| Scenario | Useful evidence beyond largest-file sorting | Consequence and review boundary |
| --- | --- | --- |
| Rust `target/debug`, incremental, fingerprints and build outputs | Cargo/project context; descendant count; small-file share; debug/release split | Rebuilding costs time; generated folders may still contain non-generated additions. Exact supported-folder/Git seals remain required. |
| JavaScript dependency trees, `.next`, `.nuxt`, Vite/parcel/turbo caches | Package/workspace context, dependencies versus output/cache location, nested totals | Dependencies and offline stores are different from build output. A home `package.json` does not classify the entire home as one project. |
| Python `.venv`, `__pycache__`, pip/wheel/conda stores | Environment marker/context; compiled bytecode versus installed environments and shared package cache | Environments can contain unique configuration; dependency recreation may need network access. No blanket environment removal. |
| Xcode DerivedData, module/index stores, device support and Archives | Recognized namespace, project association, separate archive/simulator/index totals | Archives/dSYMs may be required for distribution or crash symbolication; simulator/device data may be valuable. |
| Gradle/Maven/NuGet and language package stores | Per-project output versus shared versioned artifact cache; many small files and repeated version directories | Shared consumers and offline builds matter; named directories alone do not authorize removal. |
| Unity/Unreal/Godot generated imports, shaders and intermediates | Engine/project markers; Library/Intermediate/DerivedData versus Assets/content/saves | Reimport/shader compilation can be costly. Do not confuse original assets or saves with rebuildable intermediates. |
| Docker/container stores | Native resource relationships, VM allocation, images/layers/volumes/build cache | VM bytes are not a promise of reclaimable host space. Databases/persistent volumes require manager-specific review; never suggest deleting Docker.raw directly. |
| VM bundles and snapshot disk chains | Whole-bundle footprint, sparse logical/allocated split, related disk segments | Individual segment/snapshot deletion can destroy the VM. Preserve coherent bundles and use vendor tools. |
| AI models and model-download caches | Related shards/encoders/model variants; per-model totals; incomplete-download naming | Model weights may be intentional/offline-critical; equal sizes or similar names are not duplicate evidence. |
| Media render/proxy/waveform caches versus source media | Application/project context and generated namespaces; many small preview files | Re-rendering costs time; source footage, sample libraries and final exports are user data. |
| Games, shader caches, compatibility prefixes and launchers | Shader/generated data versus full installed games, saves and prefixes | A prefix can hold settings or user data; use native managers where required. |
| Databases, mail, Photos, chat history and search indexes | Bundle/store ownership, database plus WAL/journal relationships and aggregate size | These are managed stores, not generic cache files. Do not remove database internals based on size. |
| Git objects/worktrees, language servers and editor histories | Repository ownership, worktree/common-dir relationships, tracked/untracked state | Git storage can be recovery history; worktrees need fresh validation. Editor history may be the only retained copy. |
| Multipart archives, installers and partial downloads | Grouped part names, download origin/age and whole-set size | A part may belong to a necessary archive; age or suffix alone does not prove a failed download. |
| Cloud placeholders, APFS clones and hard links | Availability flags, filesystem identity and logical/allocated distinction | Metadata size is not downloaded data or uniquely recoverable physical space. Do not materialize cloud contents for analysis. |
| Thousands of logs, thumbnails, package metadata or tiny general files | File count, size distribution and measured small-file total even without a named technology | Surface the concentration as an observation; unknown origin stays unknown and ordinary data is not reclassified as junk. |

The first useful prioritization is measured aggregate footprint, then understandable purpose and consequences, then the separately supported cleanup policy. Recency, hardware age, extension, size and a "clean Git" result never establish disposability by themselves.

## Acceptance and retained limits

Use disposable filesystem fixtures with many small files, nested outputs, hard links, exclusions and legacy databases. Verify exact folder totals/counts, metadata-only behavior, bounded top-N/remainder math, schema migration/reopen/purge, cancellation during aggregation/import, status/read concurrency and retention. Exercise the real Tauri bridge and a released native UI with disposable scope; do not test cleanup on existing personal files.

Verify map/list drill-down, metric switching, keyboard/focus, loading/error/retry, unavailable legacy statistics and partial summaries. Capture light/dark at the 1440-wide host reference and 1100 × 720 minimum; keep host-clamped heights and untested VoiceOver/hardware limits explicit. A successful fixture or traversal is not proof of whole-disk finalization or comprehensive access.

Do not overwrite or publish the 0.8 artifacts. Build a versioned local ad hoc app/archive and matching optional administrator package. A rebuilt helper requires updated pairing and can require macOS authorization or a refreshed privacy grant; no promise of one total setup prompt. Record actual checks and unresolved limitations in `../status.md` before delivery.


The installed 0.9.0 checkpoint passed a 515-file disposable real Tauri scan, exact small-file metrics and omissions, native keyboard/drill-down/metric/pagination behavior, public schema-7 migration and restart. This does not complete whole-disk finalization/import or elevated large-index validation. Actual results and verified preference restoration are recorded in [status.md](../status.md#090-installed-native-validation--2026-10-23).

## Fresh full-scale installed checkpoint — 0.11.3, 2026-10-24

The specifically approved helper FDA refresh and all-eight-root scan finished traversal/aggregation and complete public import for **3,224,312 files**. Public file-only logical/allocated sums and all root counts/metrics exactly match the summary. The three previously denied personal folders are indexed with no permission-denied gaps; total denials fell 652 →26. Administrator-off map/drill-down, 100-row file pagination, retained root coverage and restart passed. The real 76-child Documents folder preserves all totals with 60 bounded nodes and an exact 286,720-byte allocated /213,502-byte logical /24-file remainder.

The installed folder-query correction uses the existing parent index before sorting, avoiding unrelated-row walks without schema/index or read-time housekeeping changes. No source changed after the 251-test/package checkpoint. See status and whole-disk-reliability.md for timestamps and evidence. The run remains partial because of remaining protected/unsupported/deliberately skipped or changing locations; readable-scope completion does not establish comprehensive physical-device access, unique APFS allocation or cleanup authority. Administrator mode is off, the specifically approved helper FDA grant remains enabled, and no inspected files were moved/removed.
