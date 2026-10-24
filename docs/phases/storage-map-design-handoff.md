# Folder-first Storage design handoff — 0.9.0

Recorded 2026-10-23 for the ordinary Storage extension. The user confirmed a **folder treemap beside ranked folders, with Large files separate**. The [direction and interaction contract](storage-map.md) owns the feature boundary; the [Disk Inventory X investigation](../research/disk-inventory-x.md) distinguishes observed reference behavior from uncertain internals. The fresh independent finish reviewer returned **ship**, with no material fixes in its five-section review of the supplied eight illustrative web captures and sampled source. That disposition does not establish native execution or full release acceptance.

## Direction and system fit

This is a code-led extension of **Operate / Native macOS clarity**. It preserves the grouped sidebar, toolbar, selected unboxed Coqui logo, macOS system typography, compact controls, flat panels, separators, mint actions, light/dark/system appearances and supported 1100 × 720 desktop minimum. No new visual world, generated comp, replacement token set, brand asset or shipping raster asset was introduced.

Disk Inventory X informed cumulative folder area, size-ranked navigation, synchronized context and logical/allocated choices. Coqui uses its own palette, layout, metadata scanner and persistence. The reference app's gradients, artwork, binary and implementation were not copied. Its displayed totals are not an accounting benchmark for Coqui's differently scoped and timed scan.

The durable surface strategy remains in [storage-map.md](storage-map.md): inspect aggregate footprint, understand purpose and consequences, then enter an independently authorized review if appropriate. Folder recognition and a concentration of small files grant no removal authority. The map never presents indexed bytes as guaranteed recoverable APFS storage.

## Composition and components

The first viewport retains the Storage heading, Analyze storage action, scan allocation/count summary, coverage link and five existing destinations: Explorer, Large files, Cache candidates, Coverage and Removal records. Explorer places navigation/search and a wrapping action/filter row above a shared panel. Ranked contents occupy the left half; the folder map and contextual explanation occupy the right half. At the supported minimum they remain adjacent. The page scrolls to lower content; the ranked table also has bounded vertical scrolling and sticky headers.

| Component | Finished behavior and incumbent fit |
| --- | --- |
| Ranked contents | Compact table with page selection, folder/file action, contextual purpose, optional file/small-file evidence, selected metric and exact-path inspection. Numeric sizes use the existing tabular figures. Long row names truncate while full paths remain in tooltips and inspection; the current folder path wraps below the heading. |
| Folder map | Proportional, clipped rectangles use the existing seven category colors with white labels. Path-derived colors connect the same indexed object to its ranked-row swatch. They identify objects rather than health, safety or disposability. Interior labels are omitted as space decreases; the table remains the textual alternative. |
| More entries | A separate measured remainder participates in map area and has a visible text action. It represents children beyond the bounded map query, rather than uninspected capacity or a fabricated junk category. The table is independently paginated. |
| Folder context | Hover or keyboard focus updates the named object, recognized purpose, reason, available file count, largest allocated file, small-file concentration and consequences. Unknown legacy statistics stay unavailable. |
| Metric control | Allocated/Logical selects map area and ranking explicitly. Small-file allocation remains labeled allocated even when the map uses logical size. Supporting copy distinguishes file length from allocated blocks and qualifies APFS sharing, snapshots, compression and omissions. |
| Existing review/inspection | Exact-path dialogs, Finder reveal, selected-item removal review and retained removal records reuse the established components and authority boundaries. Opening a map tile inspects a file or drills into a folder; it does not select it for removal. |

The folder workspace uses two equal flexible columns with minimum tracks of (350px/330px), tightening to (320px/300px) below (1190px). Below (960px) it stacks, although that width is outside the supported native minimum. The map is (320px) high, increasing to (360px) from (1380px). Map interiors use (18px) padding and a (6px) map-container radius; rectangles use a (3px) corner and a thin surface-colored separation. These are local chart geometry, not new global shape tokens.

The reported detector findings were advisory only: the (2px) row-swatch corner, (3px) map-tile corner and (10px) secondary row metadata were accepted in the incumbent dense geometry context. No primary detector finding or material finish fix was reported. The documenter did not rerun the detector or turn these local values into a new system scale.

## Interaction and evidence states

1. **Explore a folder.** A folder name or sufficiently large map button opens the same indexed path. Back returns to the previous visited folder; ancestor breadcrumbs and Scan roots provide hierarchy return paths. Navigation clears transient context/search/category state, resets pagination and moves keyboard focus to Ranked contents. The current path is available as the heading's description.
2. **Change size or ordering.** The metric choice requests a fresh map, sorts the table by that metric descending and returns to the first page. Name and metric headers expose sort direction. Map requests are bounded to 60 immediate children; table requests use 100-row pages. Complete-child totals and explicit remainder prevent a top-N set from being described as the whole folder. The layout helper computes squarified areas and returns rectangles in source order; it does not create measurements.
3. **Read context without relying on color.** Row hover, row-action focus and map-button focus share the active path. Names, sizes, icons, counts and purpose text carry meaning alongside color. Map buttons have action-oriented accessible names containing the object name, size/metric and available file count. Very small visual rectangles are hidden from assistive technology and replaced functionally by the ranked table. The remainder also has a text action that restores the chosen-metric ordering and focuses/scrolls to Ranked contents; it does not claim to jump directly to the first omitted child.
4. **Handle incomplete or unavailable evidence.** A non-running unfinished scan names its terminal state, links to coverage and says folder/candidate analysis may be unfinished. Missing legacy counts are not rendered as zero. A zero-byte map distinguishes no indexed file bytes from unavailable retained folder totals. Loading clears the previous map, marks the region busy and displays Loading folder totals. A map failure keeps ranked files usable, shows the concrete error and Retry folder map, and points to Large files. Table errors have their own alert/retry. Response guards discard superseded folder/metric results.
5. **Inspect before any removal.** Large files remains a separate table destination; category/search views also use the file-result presentation. Cache candidates retain their reason and consequences and hand off to the dedicated Optimization flow. Coverage and removal records remain separate destinations. Review removal requires explicit selection or a separate inspector action, then the existing expiring exact-path review, fresh validation, native Trash and per-item verification. Restore retains its separate approval and no-overwrite rules. The new map changes none of the cache/project/Git/duplicate, shared-container or bundle-identity protections.

Small means a logical size of **64 KiB or less**. The interface states that convention alongside the concentration evidence; it is not a junk or eligibility threshold. Directory totals include descendants, so adding every displayed directory row is not a disk-total calculation. The scanner remains metadata-only and does not hydrate cloud files or run package/build scripts.

Accessibility follows the incumbent button, select, table and dialog patterns: roving tab selection, explicit control labels, heading focus after navigation, `aria-sort`, busy/status/alert semantics, full-path inspection and reduced-motion CSS. Map focus has a visible white outline inside colored rectangles; row and heading focus retain the shared focus treatment. The keyboard/logical capture shows the focused Ranked contents heading after drill-down. This is source/capture evidence, not a complete VoiceOver or native accessibility walkthrough.

## Checked evidence and review scope

This documentation pass read PRODUCT.md, DESIGN.md, `.impeccable/design.json`, architecture/status/roadmap, the Storage contract and REA record, whole-disk/helper boundaries, the Impeccable documenter reference and the sources below. It opened all eight supplied captures and checked their image dimensions. It did not run the app, capture new UI, control Figma, rerun context/detector, build, test or mutate inspected files.

Every capture carries **Design review · illustrative data · no filesystem access**. Byte totals, counts, names, timestamps, partial/error messages and recognized technology examples are fixture data, not device observations.

| Capture in `.impeccable/review/` | Dimensions | Evidence |
| --- | --- | --- |
| [storage-map-dark-wide.png](../../.impeccable/review/storage-map-dark-wide.png) | 1440 × 960 | Dark Explorer, adjacent ranking/map, small-file evidence and measured-remainder presentation. |
| [storage-map-dark-minimum.png](../../.impeccable/review/storage-map-dark-minimum.png) | 1100 × 720 | Minimum dark layout, tightened headings/filters and scroll access. |
| [storage-map-light-wide.png](../../.impeccable/review/storage-map-light-wide.png) | 1440 × 960 | Light theme using the same categorical map and incumbent controls. |
| [storage-map-light-minimum.png](../../.impeccable/review/storage-map-light-minimum.png) | 1100 × 720 | Minimum light layout and retained adjacent columns. |
| [storage-map-partial-minimum.png](../../.impeccable/review/storage-map-partial-minimum.png) | 1100 × 720 | Explicit partial summary/coverage and unavailable count fixture. |
| [storage-map-error-minimum.png](../../.impeccable/review/storage-map-error-minimum.png) | 1100 × 720 | Map error/retry beside retained ranked results. |
| [storage-map-keyboard-logical.png](../../.impeccable/review/storage-map-keyboard-logical.png) | 1440 × 960 | Logical metric, ancestor path and visible heading focus after navigation. |
| [storage-map-large-files.png](../../.impeccable/review/storage-map-large-files.png) | 1440 × 960 | Separate Large files destination and incumbent table composition. |

These exact image dimensions describe web-preview evidence. They do not supersede earlier host-clamped native captures or prove an exact-size native window was exercised. The preview backend intentionally supplies a small fixed dataset: its Large files capture still contains folder-shaped rows, and drill-down reuses illustrative records. Production source requests `filesOnly` for Large files and queries the selected parent for Explorer. This capture therefore establishes the separate destination/layout only, not production filtering, nested data correctness or pagination.

| Source | Responsibility checked |
| --- | --- |
| `src/features/Storage.svelte` | Tabs, two-column layout, typed map/files queries, metric/sort/pagination, path navigation/focus, response guards, partial/error states and existing review handoffs. |
| `src/components/StorageMap.svelte` | Map/remainder rendering, readable context, small-file/legacy qualification, button labels, focus, tiny-tile alternative and theme-token reuse. |
| `src/treemap.ts` | Independent squarified geometry and stable path colors; no accounting or cleanup authority. |
| `src/tokens.css`, `src/styles.css` | Existing appearances, system font, compact controls, table/panel styling, focus and reduced motion. |
| `.impeccable/review/storage-preview.ts` | Review-only service fixtures and persistent illustrative/no-filesystem label. |

The finish review's ship disposition covers this capture/source packet. Loading, empty, unavailable-total, stale-response and full-path behavior were also inspected in source; not every one has a dedicated capture. No native Tauri bridge, installed helper upgrade, fresh full-scale finalization/import, comprehensive-access result or full VoiceOver conformance is established by this review.

## Preservation and remaining validation

This pass adds only `docs/phases/storage-map-design-handoff.md`. **DESIGN.md and `.impeccable/design.json` are preserved.** The extension uses existing tokens and primitives; folder-specific composition and local geometry are recorded here and in the direction contract instead of replacing the global system. The selected logo, original Figma kit, existing components/variables, prior handoffs and Phase 2/Phase 3/Windows/Intel specifications remain intact.

Previously recorded parity drift remains explicit and unrepaired: primary-button frontmatter inherits body weight 400 while runtime/preview use 600; status frontmatter inherits caption weight 400 while runtime/preview use 500; the context loader does not recognize the existing macOS Platform prose. The incumbent sidecar's Storage Category Tile remains its prior illustrative component example; this handoff records the new folder map's behavior and geometry. No incidental global refresh or detector suppression was made.

The 0.9.0 app/package have been reported built and verified; the native upgrade currently awaits the user's action-time approval. Package checks do not establish installed-build pairing or runtime behavior. Schema 7 is additive and retains nullable legacy statistics, but older 0.8 binaries cannot reopen it. A matching installed app/helper and fresh native fixture/bridge validation remain required. The prior installed whole-disk run remains **partial** until a fresh full-scale run completes finalization/import; tests, previews and fixture scans cannot establish comprehensive access. Current implementation checks and release/native results belong in [status.md](../status.md) and the [Storage acceptance record](storage-map.md#acceptance-and-retained-limits).

No editable Figma 0.9 Storage update has been recorded in this handoff. The code, direction contract and labeled captures are the durable current design artifacts. Native keyboard/VoiceOver, macOS 14/additional hardware, other disk layouts, real legacy/partial indexes and whole-disk completion remain explicit validation work. No existing user file was removed or cleanup approval exercised for this documentation pass.
