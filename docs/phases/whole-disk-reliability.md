# Whole-disk scan reliability — 0.11.2/0.11.3

The user asked to focus on scanning the entire storage through Coqui. This work targets the reviewed **Whole internal disk** scope, including every selected currently mounted internal root. External/network volumes remain separately selected; there is no mounting of hidden volumes/snapshots or bypass of TCC/SIP. Scanner metadata collection remains separate from removal authority.

## Evidence and source changes

The latest retained whole-disk attempt, `8b608b89-569c-4a17-bbe2-d8fd597844cc`, stopped after **2,038,000 files** with **SQLite CannotOpen, code 14**. Data traversal was interrupted; six later internal roots were not started. That error does not prove disk exhaustion; see [SQLite result codes](https://www.sqlite.org/rescode.html#cantopen) and [temporary-file behavior](https://www.sqlite.org/tempfiles.html). An earlier administrator traversal reached **3,331,424 files** but interrupted during aggregation/transport, without a complete local import. Neither run establishes full-scale completion.

At this work's start the public index was **2,049,056,768 bytes**, schema 7, with only **145 free 4-KiB database pages**. Filesystem headroom was about **1.4 GiB**. A fresh large index and temporary directory/identity/folder work require additional space; the user was asked to make roughly **8 GiB** available for this retry. That is an operational target for this Mac, not a universal requirement or completion guarantee. No inspected/user files are selected for cleanup by this work.

Changes in 0.11.2:

- Check the actual diagnostic and temporary destination filesystems before collection. Recheck at bounded progress/batch intervals and during administrator result/coverage import. Stop when measured available space falls below **512 MiB**, preserving room to record an incomplete outcome. This threshold is best effort; unrelated processes can consume space and it is not a reservation or an index-size estimate. Unavailable measurements fail with an explicit error.
- Include the collection stage in terminal scan errors. Temporary-frontier/identity/folder operations and file-index persistence retain a static step, SQLite class/extended code and connection-local numeric OS errno. Raw temporary/user paths and command output are not added to these diagnostics. The historical code-14 cause remains unresolved until a fresh run supplies evidence.
- Exclude the live traversal scratch database by filesystem identity. A literal path comparison could miss the same file through `/var`, `/private/var`, a Data-volume alias or another directory alias and count the scanner's growing index as inspected storage. A disposable fixture with an aliased temporary directory covers this case without scanning real user data.

The IPC models, schema 7, retention policy, nullable legacy statistics, complete-child/remainder totals, logical/allocated accounting, helper pairing and separate removal/authorization rules are unchanged. Cancellation still applies to traversal, aggregation, retention and both administrator import loops.

## Verification and remaining work

Focused Rust tests cover the scratch alias, low-space boundary, unavailable capacity measurement and safe SQLite step/errno retention. The complete check script, versioned release/package and native full-scale results are recorded in [status](../status.md) when completed. Fixtures do not close the full-scale checkpoint.

Native acceptance must verify reviewed mounted roots, every root's terminal traversal state, completed aggregation, terminal summary/error semantics, retained folder/file/coverage queries and restart. For an administrator run, validate the complete local import with administrator access off as well as helper queries. Successful traversal with permission or cloud omissions is qualified coverage, not access to every byte of the physical device. The prior broad hardware/macOS14/VoiceOver/Gatekeeper/security limits remain separate.

## Fresh installed checkpoint and folder-query follow-up

The authorized 0.11.2 run finished traversal/aggregation across all eight mounted internal roots and imported the full **2,537,001-file** index, metrics and **161,298** gaps. Public file-only count/logical/allocated sums exactly matched the retained summary after idle app exit. Administrator-off root queries and summary restart passed. The run remains `partial` because of coverage omissions, including **652 permission denials** and protected personal folders. No new FDA grant was made. See status for exact root states, timestamps and retained issues. This closes the prior unproven large-scale traversal/import checkpoint for this readable scope; it does not establish comprehensive access or explain the earlier code-14 failure.

Native exploration also exposed a folder-query performance defect at this scale. 0.11.3 restricts folder ranking to the existing parent index, with an exact parent equality for file pages; global queries retain their prior semantics. No new full-index sort index or housekeeping write is added. Deterministic VM-work-budget fixtures fail before and pass after the change, checking both map totals/remainder and folder/global results. Actual retained-index SQL evidence and the pending release/native verification are recorded in status.

Installed 0.11.3 verification passed exact pairing/health and native administrator-off root map, Data drill-down and logical switching. The user approved FDA plus a repeat whole-internal scan, but refreshing the existing enabled helper entry is pending: automatic review rejected removing it under grant-only consent, and the Add dialog left the selected binary disabled. No privacy entry has changed; see status for the specific follow-up approval and native evidence.

## Final 0.11.3 expanded-access acceptance

The user separately approved refreshing only the helper FDA entry; removal/re-add succeeded. The fresh installed scan completed traversal/aggregation across all eight reviewed internal roots, then complete public metadata/metric/coverage import and retention. **3,224,312 files**, **985,421,204,488 logical bytes** and **481,994,366,976 allocated bytes** match exactly between summary, public file rows and all root metrics. Permission denials fell **652 →26**; Desktop, Documents and Downloads now have nonempty matching subtree/folder counts and no permission-denied events.

Administrator-off restart, root map, personal-folder drill-down, file pagination and retained coverage passed. The real 76-child Documents folder also preserves complete totals with a 60-node map and exact remainder in both metrics. Administrator mode is off; the approved persistent helper FDA grant remains enabled, with other privacy grants untouched. No cleanup or inspected-file mutation occurred. All detailed counts, timings, errors, artifacts and evidence are in status.

The terminal `partial` state remains accurate: duplicate identities, links, cloud placeholders, special objects, mount/exclusion boundaries, filesystem races and 26 denied locations remain qualified. Headless hardware inventory remains unavailable. This closes the earlier unproven large-scale traversal/import checkpoint for this Mac's reviewed readable scope; it does not establish every-byte access, unique/recoverable APFS usage, a frozen filesystem view, another hardware/platform's acceptance or the historical code-14 cause.
