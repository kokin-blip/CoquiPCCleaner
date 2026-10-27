# Opt-in application update discovery — source preview

The user selected **Show updates; install through vendor** and **Keep guided manual tools** on 2026-10-24. These decisions close the corresponding questions in the CleanMyMac continuation plan. Coqui discovers offered versions for explicitly selected applications; App Store or vendor tooling decides and installs. DNS flushing, RAM purging, startup changes and other system maintenance remain guided/manual. No reference-app preselection or factory grants action authority.

## User flow

Applications has an Updates tab. It reads the retained local inventory, starts with zero selected apps and accepts at most ten exact applications, including an explicitly chosen bundle outside the inventory. Application inventory refresh is now explicit through Installed Apps → Refresh; opening Applications or saved update results does not start collection. Search and Reload recorded apps query retained data only.

Review update sources performs bounded local metadata inspection, then displays exact paths, installed versions, disclosed endpoints, the static source-rule review date and a ten-minute expiry. A separate, initially unchecked acknowledgement discloses selected App Store bundle IDs sent to Apple, vendor-feed requests and network-address exposure. Unsupported sources cannot enable a check. Each check consumes its review once and rechecks local metadata before networking. Stop cancels the active operation; unfinished comparisons stay unmeasured.

Results distinguish Newer version listed, No newer version observed, Check unavailable, Version comparison unavailable and cancelled items. They retain source/check/publication dates and explicit limits. Completed results replace the selection table; Check other apps starts a new empty selection and acknowledgement. History and Overview read saved reports without network requests. Offers are not labeled installed, compatible, safe, healthy or universally up to date.

Only a listed newer version offers a vendor handoff. App Store items open the fixed App Store Updates page. Supported vendor-feed items open the exact reviewed application so its updater can be used. Coqui revalidates the local seal before this handoff. It does not open a response-supplied download URL, fetch a package or execute an installer.

## Engine boundary

Rust owns inspection, validation, networking and retained records; Svelte owns presentation, state and navigation. Generated `AppUpdate*` bindings align four IPC commands: `prepare_app_updates`, `check_app_updates` with a typed event channel, pure `latest_app_updates`, and `open_app_update`. Additive record kinds use existing schema 7; no migration or helper protocol change is needed. Private source-review records are hidden from ordinary History. Public reports retain per-item results, cancellation and interrupted-operation recovery.

Exact unlinked `.app` paths use bounded, no-follow Info.plist reads and device/inode plus metadata-digest seals. Receipt **presence** selects Apple's US public catalog; no receipt contents, purchase details or account credentials are read. Catalog responses require one exact bundle-ID/desktop-software match. No match remains unavailable. A declared public HTTPS `SUFeedURL` can select Sparkle: default-channel, full non-delta macOS release metadata with the proper namespace and direct item fields. Malformed, conflicting, unsupported-channel or unsupported-version metadata fails closed. Comparison supports bounded dotted-numeric App Store display versions or Sparkle build versions; unsupported version semantics remain uncomparable. Minimum macOS is displayed without evaluating compatibility.

The macOS adapter uses bounded fixed commands for system proxy inspection, DNS and HTTPS GET. Configured proxy environments, enabled/unknown/scoped system proxy policies, non-public addresses, credentials/query-bearing vendor feeds and unsupported redirects withhold the direct check. Every resolved address is validated, then the complete public address set is pinned for normal IPv4/IPv6 fallback. Curl configuration is disabled and its environment is cleared; TLS validation remains enabled. One fifteen-second request deadline includes proxy/DNS work, with cancellation polling and a two-MiB response cap. No credentials, cookies, local file contents, response HTML, package URLs or updater downloads are retained or forwarded.

## Source basis

Primary documentation reviewed on 2026-10-24:

- [Apple Search API](https://performance-partners.apple.com/search-api): public lookup and desktop software entities. Public catalog results are incomplete and can differ by region/account.
- [Sparkle publishing documentation](https://sparkle-project.org/documentation/publishing/) and [SUAppcastItem reference](https://sparkle-project.org/documentation/api-reference/Classes/SUAppcastItem.html): build/display version distinction, channels, enclosures and minimum-system metadata. This implementation supports a conservative subset, not every Sparkle deployment.
- [Public BBEdit App Store entry](https://apps.apple.com/us/app/bbedit/id404009241?mt=12) and [Bare Bones update guidance](https://web.barebones.com/support/bbedit/updates.html): a public validation example and vendor-managed installation guidance.

REA's observed CleanMyMac candidate/source/response correlation informed the workflow, not copied proprietary implementation or unverified installation behavior. The [research ledger](../research/cleanmymac-5.2.4.md) retains the unresolved Sparkle validator/authenticity boundary.

## Verification and limits

The final unmodified `scripts/check.sh` passed **103 frontend +192 Rust =295 tests**, with four native mutation tests and one opt-in network test ignored by default, **zero Svelte errors and warnings**, production frontend build, formatting and locked native compile. This run includes the final review corrections. Twelve offline update integration tests and two transport unit tests exercise consent, single-use/expiry/identity seals, exact catalog identity, feed parsing, strict comparison, cancellation, read-only saved results, handoff and proxy/public-address boundaries. All 28 focused update/Overview/application/removal tests also passed after the correction batch. See the latest [status checkpoint](../status.md) for logs and final visual-review disposition.

The opt-in native test ran successfully with a labeled metadata-only BBEdit fixture and an isolated temporary database. Apple's public response listed 16.0.3, the exact bundle ID matched, the offered state/check time persisted, reopening read the same report and review replay was refused. No actual installed-app inventory was transmitted, user-history record added, app launched, package downloaded or update installed. The first attempt revealed unreachable IPv6 selection; a public IPv4 control succeeded, and pinning all validated addresses resolved the native failure without bypassing proxy policy. Log: `/private/tmp/coqui-app-updates-native-check.log`.

The packaged preview's Updates route, zero selections, native picker and disabled-before-acknowledgement source review were inspected using retained inventory and an explicitly labeled disposable metadata fixture. No GUI online check was submitted. Result/theme captures use a visibly labeled development-only component harness with illustrative data, no network and disabled vendor launch; they do not establish native vendor-handoff acceptance. Full VoiceOver, macOS 14, another Mac, configured-proxy/device scenarios, broad live vendor-feed coverage and native vendor/App Store handoff remain unverified. Declared feeds and receipt presence are not signature/authorship/purchase proofs. New complete storage access and whole-product CleanMyMac parity are not claimed.

This remains a separately named, uninstalled source preview, versioned 0.12.0. The installed app/helper pair, pairing manifest and privacy grants are unchanged. No existing release ZIP/installer was overwritten.

The [design handoff](application-updates-design-handoff.md) records all six final captures and the bounded finish verdict: the reviewer scored the continuation, focus and saved-caption fixes resolved. Existing tokens, sidecar and shipping raster provenance were preserved.
