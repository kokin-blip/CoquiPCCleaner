#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
export MACOSX_DEPLOYMENT_TARGET=14.0
export COPYFILE_DISABLE=1
version=$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')
output="release/$version"
app='src-tauri/target/aarch64-apple-darwin/release/bundle/macos/CoquiPCCleaner.app'
[ -d "$app" ] || { echo 'Run scripts/release.sh first.' >&2; exit 1; }
cargo build --manifest-path engine/Cargo.toml --locked --release --bin coqui_helper
stage=$(mktemp -d /private/tmp/coqui-administrator-package.XXXXXX)
trap 'rm -rf "$stage"' EXIT
payload="$stage/payload"
base="$payload/Library/Application Support/CoquiPCCleaner/Helper"
mkdir -p "$base" "$payload/Library/PrivilegedHelperTools" "$payload/Library/LaunchDaemons" "$payload/Applications" "$stage/scripts" "$output"
/usr/bin/ditto "$app" "$payload/Applications/CoquiPCCleaner.app"
helper="$payload/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper"
cp engine/target/release/coqui_helper "$helper"
/usr/bin/codesign --force --sign - --options runtime --identifier com.coqui.CoquiPCCleaner.Helper "$helper"
helper_hash=$(/usr/bin/codesign -d --verbose=4 "$helper" 2>&1 | sed -n 's/^CDHash=//p')
app_hash=$(/usr/bin/codesign -d --verbose=4 "$app" 2>&1 | sed -n 's/^CDHash=//p')
python3 - "$base/pairing.json" "$helper_hash" "$app_hash" "$payload/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist" <<'PY'
import sys,re,json,plistlib
for value in sys.argv[2:4]:
    if not re.fullmatch('[0-9a-fA-F]{40}',value):raise SystemExit('Missing or malformed build hash')
h='identifier "com.coqui.CoquiPCCleaner.Helper" and cdhash H"'+sys.argv[2]+'"'
a='identifier "com.coqui.CoquiPCCleaner" and cdhash H"'+sys.argv[3]+'"'
with open(sys.argv[1],'w') as f:json.dump({'version':2,'mode':'production','applicationPath':'/Applications/CoquiPCCleaner.app','clientExecutable':'/Applications/CoquiPCCleaner.app/Contents/MacOS/coqui-pc-cleaner','applicationRequirement':a,'helperRequirement':h,'clientRequirement':'('+a+') or ('+h+')'},f)
with open(sys.argv[4],'wb') as f:plistlib.dump({'Label':'com.coqui.CoquiPCCleaner.Helper','ProgramArguments':['/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper','--serve'],'MachServices':{'com.coqui.CoquiPCCleaner.Helper':True},'AssociatedBundleIdentifiers':['com.coqui.CoquiPCCleaner'],'ProcessType':'Adaptive'},f)
PY
/usr/bin/codesign --verify --deep --strict "$payload/Applications/CoquiPCCleaner.app"
/usr/bin/codesign --verify --strict -R "=identifier \"com.coqui.CoquiPCCleaner.Helper\" and cdhash H\"$helper_hash\"" "$helper"
cat > "$stage/scripts/preinstall" <<'PRE'
#!/bin/sh
set -eu
for path in '/Library' '/Library/Application Support' '/Library/PrivilegedHelperTools' '/Library/LaunchDaemons' '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper'; do
  [ ! -L "$path" ] || exit 1
  if [ -e "$path" ]; then
    [ -d "$path" ] && [ "$(/usr/bin/stat -f %u "$path")" = 0 ] || exit 1
    mode=$(/usr/bin/stat -f %Lp "$path"); [ $((0$mode & 0022)) = 0 ] || exit 1
  fi
done
for path in '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper' '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist' '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json' '/Library/Application Support/CoquiPCCleaner/Helper/validation.json'; do
  [ ! -L "$path" ] || exit 1
  if [ -e "$path" ]; then [ "$(/usr/bin/stat -f %u "$path")" = 0 ] || exit 1; fi
done
if [ -e '/Applications/CoquiPCCleaner.app' ]; then
  [ ! -L '/Applications/CoquiPCCleaner.app' ] && [ ! -L '/Applications/CoquiPCCleaner.app/Contents' ] && [ ! -L '/Applications/CoquiPCCleaner.app/Contents/Info.plist' ] || exit 1
  /usr/bin/codesign --verify --strict -R '=identifier "com.coqui.CoquiPCCleaner"' '/Applications/CoquiPCCleaner.app' || exit 1
  [ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' '/Applications/CoquiPCCleaner.app/Contents/Info.plist')" = 'com.coqui.CoquiPCCleaner' ] || exit 1
fi
/bin/launchctl bootout system/com.coqui.CoquiPCCleaner.Helper 2>/dev/null || true
# Tear down only installer-created validation objects, after a successful restore cycle.
marker='/Library/Application Support/CoquiPCCleaner/Helper/validation.json'
if [ -f "$marker" ]; then
  [ "$(/usr/bin/plutil -extract purpose raw -o - "$marker")" = 'coqui-disposable-administrator-validation' ] || exit 1
  fixture_id=$(/usr/bin/plutil -extract id raw -o - "$marker")
  case "$fixture_id" in *[!a-fA-F0-9-]*|'') exit 1;; esac
  [ "${#fixture_id}" = 36 ] || exit 1
  fixture_app="/Applications/Coqui Administrator Fixture $fixture_id.app"
  fixture_cache="/Library/Caches/org.coqui.validation.$fixture_id"
  if [ -e "$fixture_app" ]; then
    [ ! -L "$fixture_app" ] && [ "$(/usr/bin/stat -f %u "$fixture_app")" = 0 ] || exit 1
    [ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$fixture_app/Contents/Info.plist")" = "org.coqui.validation.$fixture_id" ] || exit 1
    /bin/rm -rf -- "$fixture_app"
  fi
  if [ -e "$fixture_cache" ]; then
    [ ! -L "$fixture_cache" ] && [ "$(/usr/bin/stat -f %u "$fixture_cache")" = 0 ] || exit 1
    [ "$(/usr/libexec/PlistBuddy -c 'Print :FixtureID' "$fixture_cache/CoquiFixtureMarker.plist")" = "$fixture_id" ] || exit 1
    /bin/rm -rf -- "$fixture_cache"
  fi
  /bin/rm -f "$marker"
fi
PRE
cat > "$stage/scripts/postinstall" <<'POST'
#!/bin/sh
set -eu
/usr/sbin/chown -Rh root:wheel '/Applications/CoquiPCCleaner.app'
/bin/chmod -RN '/Applications/CoquiPCCleaner.app'
/bin/chmod 755 '/Applications/CoquiPCCleaner.app' '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper'
/usr/sbin/chown root:wheel '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper'
/bin/chmod 644 '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json' '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist'
/bin/chmod 755 '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper'
/bin/launchctl bootstrap system '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist'
POST
chmod 755 "$stage/scripts/preinstall" "$stage/scripts/postinstall"
/usr/bin/pkgbuild --analyze --root "$payload" "$stage/components.plist"
python3 - "$stage/components.plist" <<'PYCOMP'
import plistlib,sys
p=sys.argv[1]
with open(p,'rb') as f:items=plistlib.load(f)
for item in items:
    item.update(BundleIsRelocatable=False,BundleIsVersionChecked=False,BundleHasStrictIdentifier=True,BundleOverwriteAction='upgrade')
with open(p,'wb') as f:plistlib.dump(items,f)
PYCOMP
/usr/bin/pkgbuild --root "$payload" --component-plist "$stage/components.plist" --scripts "$stage/scripts" --identifier com.coqui.CoquiPCCleaner.Administrator --version "$version" --install-location / --ownership recommended "$output/CoquiPCCleaner-$version-administrator-arm64.pkg"
if [ "${1:-}" = '--with-fixtures' ]; then
  python3 - "$payload" "$output/administrator-fixture.json" <<'PY'
from pathlib import Path
import json,plistlib,sys,uuid,os
root=Path(sys.argv[1]);marker=Path(sys.argv[2])
# Retain the reviewed disposable identity across repair builds of one cycle.
previous=json.loads(marker.read_text()) if marker.exists() else {}
id=str(uuid.UUID(previous['id'])) if previous.get('purpose')=='coqui-disposable-administrator-validation' else str(uuid.uuid4())
meta={'id':id,'purpose':'coqui-disposable-administrator-validation'}
(root/'Library/Application Support/CoquiPCCleaner/Helper/validation.json').write_text(json.dumps(meta))
Path(sys.argv[2]).write_text(json.dumps(meta,indent=2)+'\n')
app=root/f'Applications/Coqui Administrator Fixture {id}.app'
(app/'Contents/MacOS').mkdir(parents=True)
(app/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':f'org.coqui.validation.{id}','CFBundleName':'Coqui Administrator Fixture','CFBundleDisplayName':'Coqui Administrator Fixture','CFBundleShortVersionString':'0.0.0','CFBundlePackageType':'APPL','CFBundleExecutable':'fixture'}))
(app/'Contents/MacOS/fixture').write_text('Disposable non-executable metadata fixture. Never launch.\n')
(app/'Contents/Opaque link').symlink_to('MacOS/fixture')
cache=root/f'Library/Caches/org.coqui.validation.{id}'
cache.mkdir(parents=True);cache.chmod(0o700)
(cache/'CoquiFixtureMarker.plist').write_bytes(plistlib.dumps({'FixtureID':id}))
(cache/'Disposable data.txt').write_text('Disposable administrator-owned cache data.\n')
PY
  /usr/bin/pkgbuild --analyze --root "$payload" "$stage/validation-components.plist"
  python3 - "$stage/validation-components.plist" <<'PYCOMP'
import plistlib,sys
p=sys.argv[1]
with open(p,'rb') as f:items=plistlib.load(f)
for item in items:item.update(BundleIsRelocatable=False,BundleIsVersionChecked=False,BundleHasStrictIdentifier=True,BundleOverwriteAction='upgrade')
with open(p,'wb') as f:plistlib.dump(items,f)
PYCOMP
  /usr/bin/pkgbuild --root "$payload" --component-plist "$stage/validation-components.plist" --scripts "$stage/scripts" --identifier com.coqui.CoquiPCCleaner.AdministratorValidation --version "$version" --install-location / --ownership recommended "$output/CoquiPCCleaner-$version-administrator-validation-arm64.pkg"
fi
python3 scripts/verify-administrator-package.py
if [ "${1:-}" = '--with-fixtures' ]; then python3 scripts/verify-administrator-package.py "$output/CoquiPCCleaner-$version-administrator-validation-arm64.pkg"; fi
/usr/bin/shasum -a 256 "$output"/*.pkg "$output"/*.zip > "$output/SHA256SUMS"
