# Remaining permission-denial review — 2026-10-24

Source: installed 0.11.3 metadata scan `5eca2c14-51c8-44de-8681-f5dd7287bd53`, after the specifically authorized helper Full Disk Access refresh. Read-only public-index query confirms exactly **26** `permissionDenied` rows, all on the Data volume. Desktop, Documents and Downloads have zero such gaps.

The root helper already had Full Disk Access during this run. All 26 locations belong to macOS system/service namespaces. Read-only `lstat` succeeded for 25: seven have Darwin's `restricted` flag; directory owners are root or system service accounts. One (`/private/var/db/fpsd/dvp`) also refused ordinary-user `lstat` with EACCES. Ordinary-user metadata is supporting context, not a substitute for the retained root-helper observation. The restricted flag primarily describes system protection and is not itself proof of the reason a directory read failed.

No safe, ordinary user-folder permission repair was identified. No ownership, ACL, mode, TCC, SIP, entitlement or service changes were made. No new full-disk scan was run for this inspection, and no denial is claimed resolved. Keep all 26 gaps explicit rather than concealing them as exclusions or declaring comprehensive access. The earlier reduction from 652 to 26 remains the last fresh helper traversal evidence.

The saved reason does not retain the original errno or identify the exact enforcement mechanism. Do not label every path as SIP or assume a chmod can fix it. [Apple's file-system permissions explanation](https://developer.apple.com/forums/thread/678819) describes BSD/ACL, sandbox, mandatory access controls (including internal Data Vaults), and Endpoint Security; mandatory controls apply to root too, and protected locations are not exhaustively documented. Future directory-denial summaries now retain the OS error number; existing historical records stay historical.

## Exact retained paths

- `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_FM_CodeLM`
- `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_FM_GenerativeModels`
- `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_FM_Overrides`
- `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_FM_Visual`
- `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_IF_Planner`
- `/private/var/OOPJit`
- `/private/var/db/AppleIntelligencePlatform`
- `/private/var/db/DumpPanic`
- `/private/var/db/KernelExtensionManagement/Staging`
- `/private/var/db/MobileIdentityService`
- `/private/var/db/SoC`
- `/private/var/db/Spotlight`
- `/private/var/db/Spotlight-V100`
- `/private/var/db/SystemKeys`
- `/private/var/db/appinstalld`
- `/private/var/db/fpsd/dvp`
- `/private/var/db/installcoordinationd`
- `/private/var/db/lockdown`
- `/private/var/db/oah`
- `/private/var/db/searchparty`
- `/private/var/db/sysdiagnose/com.apple.sysdiagnose`
- `/private/var/folders/zz/zyxvpxvq6csfxvn_n00000sm00006d/0`
- `/private/var/folders/zz/zyxvpxvq6csfxvn_n00000sm00006d/C`
- `/private/var/networkd/db`
- `/private/var/protected/trustd/private`
- `/private/var/protected/xprotect/db`

## Product follow-up

Coverage can filter to permission denials with bounded 100-row pages in both ordinary/public and authenticated helper reads. Rust adds location-specific current guidance to coverage responses, without changing the stored gap or database schema. System/service guidance explains the limit; personal-folder guidance identifies the actual scanning process and intended owner/privacy settings. No security-setting mutation or new scan authority is added by the feature. Both the final uninstalled build and subsequently installed 0.12.0 native public-index filter displayed all 26 guidance disclosures with Next disabled. Installed native pairing passed; the separately approved same-helper Full Disk Access entry refresh was completed and verified on, with app access off and administrator mode off. These installation checks do not establish new privileged filter execution, fresh traversal or resolved denials. Evidence: `.impeccable/review/development-0120-installed-denials.png` and `development-0120-fda-refreshed.png`.
