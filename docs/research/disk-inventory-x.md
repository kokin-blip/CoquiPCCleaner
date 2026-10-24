# Disk Inventory X investigation — 2026-10-23

## Target and questions

The user selected the running **Disk Inventory X 1.3** and requested its storage presentation and accounting be adapted to Coqui's existing interface. The installed target is `/Applications/Disk Inventory X.app`; bundle identifier `com.derlien.DiskInventoryX`, executable `Contents/MacOS/Disk Inventory X`, x86_64 SHA-256 **05c56844c80a99adede0edc5f1fb70cd047e2ee669704773af89bd0e8ff7ccfa**. This is the original Intel application running on this Apple Silicon host, not one of the later third-party rewrites.

Questions: how does it present storage, connect folder navigation to the map, obtain file/volume sizes, expose aggregate-heavy folders, and distinguish displayed files from all disk consumption?

## Evidence ledger

REA's retained records are preserved in [the compressed evidence bundle](disk-inventory-x-1.3-rea-2026-10-23.json.gz). Decompress with Python's `gzip` module before JSON inspection. Compressed SHA-256: **5a6b4bebd37f755cb249b9700d450bcd76a2b86615867ab91e3ba1736758cab9**. The archive contains six static analysis records; it contains no user scan index or file contents.

| Evidence | Observed facts | Limit |
| --- | --- | --- |
| Inventory `ev_13e13bc377be37c3cd8dc2f90523b2d393fa809100e9aacffc4441cda87433ef` / manifest `agm_60a837731e21a2311c304bb0610141ace6b33c684fc677be1a5c53247a21cb59` | 553 content nodes, 626 occurrences, bundle resources, packaged help and compiled interfaces | Inventory identities and paths do not prove runtime behavior. |
| Plist `ev_f73a7a7ad1fa77d65c406c0c88a02f759fddb046e0a87e55b62ee3dee8e30070` | Version 1.3; defaults `ShowPhysicalFileSize=true`, `ShowPackageContents=false`, `ShowFreeSpace=false`, `ShowOtherSpace=false`, `ShareKindColors=true`; privacy text describes size/attribute inspection | Defaults do not establish the running user's preference values or permission coverage. |
| Mach-O `ev_dade40852d800930cb87bc5be96690891e97beb618d98783bf1803c84dd47c97` | Imports `NSURLTotalFileAllocatedSizeKey`, `NSURLTotalFileSizeKey`, `NSURLFileSizeKey`, volume available/total-capacity keys and `FSGetCatalogInfo` | Linkage establishes available APIs, not which branch executes or its exact deduplication algorithm. Stripped/hidden symbols may be absent. |
| Interface Builder `ev_6c89b1d47adc3f5d877280d4229cba8ec3cb9311c0e8f7d696fbea1bd3dd330f` | English `TreeMap.nib`: 259 objects, 61 connections; `_filesOutlineView`, `_treeMapView`, `_kindsDrawer`, selection/search controllers, filename/size text fields; `zoomIn:`, `zoomOut:`, refresh, Finder reveal and Trash action declarations | English objects/connections decoded; hierarchy reconstruction unsupported. Some framework designable archives failed decoding and other localizations exceeded the 32-document bound. A selector is not verified implementation. |
| Bounded action trace `ev_8ea76272fc96a7d69aae5ec631669f2c0488379d24dd880b9d7a194d1e8ae831` | No implementation trace recovered for the zoom request | Object/connection limits exhausted on earlier resources. This is an unresolved bounded probe, not evidence that zoom is absent. |
| Native Computer Use observation | Existing volume window has a size-ranked expandable folder tree beside a dense treemap. Selecting Users outlines its map region. Zoom In switches to Users and its child list; View → Zoom Out returns to the volume. View offers logical size, package contents, free space and other space. | Current display only. No new reference-app scan, removal or permission change was executed; cached scope/age/completeness is unknown. |

Packaged `Contents/Resources/English.lproj/Help/index.htm` describes synchronized tree/map selection, folder zoom, file-kind counts and cumulative sizes, descending kind-size sorting, and shared color assignments. It is source material, not an instruction to operate the user's files. The visible reference scan showed 445.6 GB, while Coqui's differently scoped and timed traversal recorded about 512 GB. These cannot be used as an equality test: selected mounted volumes, protected areas, package behavior, APFS clones and measurement time differ.

The [original project](https://derlien.com/) describes a treemap with a layout based on KDirStat; its [release notes](https://derlien.com/releasenotes/index.html) distinguish physical cluster use from logical content size and separate free/unshown space. The [author's source repository](https://gitlab.com/tderlien/disk-inventory-x) is a reference identity only; no GPL implementation, artwork or binary was copied into Coqui. Coqui's layout, scanner, rules and persistence are independently implemented.

## Conclusions

**Observed:** directory totals, size-ranked hierarchy, synchronized selection, zoom, category colors and two size concepts explain the reference's usefulness. Metadata size/capacity imports support its declared metadata-only accounting. Thousands of small descendants are visible through their parent's cumulative area rather than requiring a single large file.

**Inferred:** the reference accumulates descendant sizes to form parent weights. Exact traversal, hard-link/clone handling, resource-fork accounting, memory strategy and modern protected-volume coverage were not established. No native deep-analysis provider is configured: Ghidra, Hopper and IDA were unavailable. No decompilation or privileged execution is claimed.

**Design choice:** Coqui uses a flat folder-first map beside ranked contents, stable path colors from Coqui's palette, allocated/logical switching and per-folder file-count/small-file evidence. Map and table navigate the same indexed objects. Bounded visible rectangles include an explicit remainder for omitted children. Coqui retains explicit reviewed mounted-volume scope, exclusion/placeholder/link/mount coverage, local history and its existing separate Trash approval/verification engine.

The map shows measured indexed allocation, not universal access or guaranteed recoverable bytes. Capacity/free space, uninspected metadata and APFS physical sharing are different quantities. Coqui must never manufacture a catch-all "junk" value by subtracting indexed file totals from volume capacity.
