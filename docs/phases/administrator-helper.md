# Optional administrator helper — 0.8.0

The portable app remains useful without this component. Administrator features are off by default and require the matching package-installed app at `/Applications/CoquiPCCleaner.app`. Installing the service, enabling its preference, reviewing a scope, and approving an operation are separate actions. Nothing runs a scan or cleanup at installation or app open.

## Installation gate and pairing

The corrected health-only installation gate passed on 2026-10-23: an authenticated non-root client reached a UID 0 daemon; a different identifier and a mismatched build were rejected. The user approved removal, and the diagnostic daemon, helper and pairing manifest were removed. A later probe failed closed. The first package-installed ad hoc gate is therefore complete. The original malformed-hash defect and repair remain recorded in whole-disk-cleanup.md.

The production Installer package contains the app, hardened-runtime ad hoc helper, a fixed launch-daemon plist and a root-owned pairing manifest. It disables component relocation and same-version skipping. Preinstall guards existing paths and stops only Coqui's service; postinstall establishes root ownership and registers the on-demand Mach service. No RunAtLoad or KeepAlive job collects data. Apple describes the traditional Installer approach at <https://developer.apple.com/forums/thread/771162>. Local ad hoc validation does not establish downloaded Gatekeeper behavior or notarization.

Public NSXPC signing requirements pin both exact build hashes. Production additionally checks the installed executable path, root ownership/non-writable ancestors and strict static signatures; signed copies elsewhere are refused. Updates must refresh app/helper/manifest together. A mismatched portable build cannot use an installed helper. No webview or arbitrary client supplies an effective UID, home, database location, command or script.

## Operation authority and interruption

The strict protocol exposes health, preferences, application inspection, removal/restore preview, approved execution, metadata scanning, paginated retained results, cancellation and service lifecycle. Following the user’s release-UX request on 2026-10-23, enabling administrator choices obtains a native Authorization Services session. Coqui reuses that session for up to 15 minutes in the current process, timed with a continuous monotonic clock that includes sleep. The first later operation can authorize again if the session expired or the app restarted; macOS may expire rights sooner. Settings offers Lock administrator access, and turning the preference off clears the session. App exit loses its in-memory authority. No password or reusable authorization form enters the webview, SQLite or logs. Every elevated request still carries independently validated native authorization plus its own fresh operation-bound challenge. Exact-path removal/restore approvals remain separate and mandatory. The helper independently verifies authorization and consumes a short-lived, operation/UID-bound challenge once. Unknown fields, operations and protocol/build mismatches fail closed.

Authorized filesystem work first returns an inert reservation. Only the original process, identified by PID and kernel start time, may start it. Unclaimed reservations expire; restart marks them interrupted. Active jobs are cancelled when their client exits or stops polling. Destructive work waits for acknowledged progress before the first move and between items, limiting a disconnected consumer to a possible in-flight move. Cancellation never reverses an operation already in flight, retries an uncertain move, or resumes interrupted work.

The helper independently validates the user's exclusions/protected apps and current mounted-volume scope. Prepared work must still match the latest saved policy when it starts; it cannot overwrite a newer exclusion preference. Settings cannot change during active work. All requests retain the original non-root account identity; root is never the Trash owner. Root access does not bypass TCC, SIP, sealed-system restrictions or cloud protection.

## Metadata and removal scope

Elevated scans enumerate only reviewed mounted internal roots, or separately selected volumes; they do not mount snapshots/hidden volumes. Metadata traversal and SQLite-backed aggregation keep the portable coverage semantics. Complete traversal remains distinct from comprehensive access. Paginated queries use the retained helper index when enabled. Bounded local copies preserve partial results when it is disabled; an interrupted import is explicitly qualified.

Elevated removal adds root-owned items only inside independently allowed personal/selected-volume namespaces, eligible application bundles, qualified exact-identity associated data and explicitly selected contents of `/Library/Caches`. System/Apple locations, whole namespace roots, project/developer-managed outputs, cloud/provider data and protected/shared/uncertain associations remain blocked or use their existing dedicated workflow. Size and age never grant authority. Running apps, Coqui, Apple bundles, vendor services/extensions and user protection remain guarded. Related data starts unselected.

Root review uses descriptor-relative `O_NOFOLLOW` traversal, retained parent/object identities and a descendant seal. It caps a review at 20,000 entries, 128 levels and 8 MiB of relative-path data; split larger reviews. The scanner has no corresponding index-size cap. Bundle links are opaque entries; linked parents and metadata are rejected. Immediately before each move the helper repeats policy, identity, flags, permissions, running-app and descendant checks. Metadata seals are not content backups and cannot freeze another process.

The helper spawns only its fixed installed worker with the requesting UID/GID before Foundation resolves that user's native Trash directory. That separate worker clears all supplementary groups and drops real/effective UID/GID before Foundation. The root daemon never changes its own credentials. It verifies a private, user-owned Trash destination on the same device and uses descriptor-relative Darwin exclusive rename. Elevated moves do not create Finder Put Back metadata: use Coqui's retained, separately reviewed restore. No permanent deletion, Trash emptying, cross-device copying or full application-recovery promise is exposed. Restore verifies retained identities/contents and original parent and refuses any existing destination at the rename itself.

## Local data and lifecycle

Private per-UID schema-6 databases live below `/Library/Application Support/CoquiPCCleaner/Helper/State`, with root-only directories and single-link database guards. Intent and terminal records use synchronous durable writes. Private operation challenges/jobs/policy tables are additive. The GUI retains public action mirrors and imported metadata in its own schema-6 database. Legacy settings/history/removal records remain readable with default-false administrator authority.

Retention follows the user's configured interval. Settings offers a separately confirmed and authorized removal of that account's helper records, imported helper scans and action mirrors. It never empties Trash or removes another account's records. Local diagnostic-data removal does not silently purge the private helper journal. Helper removal unregisters/removes only Coqui's fixed daemon/helper/pairing files, preserving app and journals; reinstall a matching helper to use retained recovery references. The service may remain idle after a request, but has no scheduled/background collection.

## Release setup and prompt expectations

1. Install the optional matching app-plus-helper package. Installer controls its administrator authentication.
2. If protected locations or user Trash are denied, approve the exact installed helper in System Settings → Privacy & Security → Full Disk Access. That is a separate macOS permission and may require authentication. Ad hoc build replacements can revoke a previous grant; check the final installed build after updates.
3. Open the installed app, explicitly check the helper, and enable administrator choices. Authenticate for the short session. Subsequent scan/inspection/review/Trash/restore operations reuse that session while still requiring their individual scopes and approvals.
4. Lock the session, turn the option off or quit Coqui when finished. No closed-app collection or cleanup is started.

Coqui cannot guarantee one total prompt across Installer, privacy settings and app authorization. The intended ordinary workflow uses one interactive Authorization Services request for its short app session. The fixed native fixture validator measures these API requests; it does not inspect passwords or operate SecurityAgent.

## Reproduction and current validation

```sh
./scripts/check.sh
./scripts/release.sh
./scripts/administrator-package.sh --with-fixtures
python3 scripts/verify-administrator-package.py
# After explicit native installation/authorization of the validation package:
/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper --validate-fixtures
```

The validation build adds uniquely named, root-owned disposable application/cache fixtures and a root-owned marker. Its diagnostic command accepts no caller paths/general RPC. It tests paired health, impostors/uninstalled copies/version mismatch/missing authorization, separately approved fixture Trash and restore, replay refusal, cancellable whole-internal metadata collection and stop/on-demand restart. The ordinary package removes only the matching marked fixtures after recovery. Never run mutation tests on existing user files.

Current source checks pass: **58 frontend + 135 Rust = 193 standard tests**, zero Svelte/TypeScript warnings/errors, frontend production build, formatting and locked native check. Four explicit native mutation tests are intentionally separate. Release and both package payload verifiers pass (arm64, minimum macOS 14, hardened runtime, literal verified pins, fixed daemon paths). The previous production build passed native installed authentication, rejected impostors/version mismatch/missing authorization/replay, verified root-owned fixture Trash and separately reviewed restore, and retained records after stop/on-demand restart. Its exact helper uninstall also passed. A native VM-volume GUI scan imported two file rows with folder drill-down and coverage; headless hardware inventory remained unavailable. The new short-session build awaits installation and measured prompt-count validation. Those earlier results do not certify the changed session flow. Additional macOS 14 hardware, other APFS/external layouts, full VoiceOver, downloaded Gatekeeper and an independent security review remain validation limits.


Production service removal passed with the user’s specific action-time approval: launchd no longer registered it and the daemon plist, binary and pairing manifest were absent. The app, restored root-owned fixtures and private State directory remained. A final matching install and native short-session workflow validation are in progress. The previous native VM-volume GUI scan imported two file rows and exposed folder drill-down and retained coverage; the headless inventory was unavailable and remained qualified. That run did not traverse the entire disk.
