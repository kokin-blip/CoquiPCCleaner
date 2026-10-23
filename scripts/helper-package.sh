#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export DEVELOPER_DIR="${DEVELOPER_DIR:-/Library/Developer/CommandLineTools}"
export MACOSX_DEPLOYMENT_TARGET=14.0
export COPYFILE_DISABLE=1
cargo build --manifest-path engine/Cargo.toml --locked --release --bin coqui_helper
stage=$(mktemp -d /private/tmp/coqui-helper-package.XXXXXX)
trap 'rm -rf "$stage"' EXIT
helper_root="$stage/payload/Library/PrivilegedHelperTools"
pair_root="$stage/payload/Library/Application Support/CoquiPCCleaner/Helper"
daemon_root="$stage/payload/Library/LaunchDaemons"
mkdir -p "$helper_root" "$pair_root" "$daemon_root" "$stage/scripts" release/helper-gate
cp engine/target/release/coqui_helper "$helper_root/com.coqui.CoquiPCCleaner.Helper"
/usr/bin/codesign --force --sign - --identifier com.coqui.CoquiPCCleaner.Helper "$helper_root/com.coqui.CoquiPCCleaner.Helper"
helper_hash=$(/usr/bin/codesign -dvv "$helper_root/com.coqui.CoquiPCCleaner.Helper" 2>&1 | sed -n 's/^CDHash=//p')
python3 - "$pair_root/pairing.json" "$helper_hash" "$daemon_root/com.coqui.CoquiPCCleaner.Helper.plist" <<'PY'
import json,plistlib,sys
req='identifier "com.coqui.CoquiPCCleaner.Helper" and cdhash H"'+sys.argv[2]+'"'
with open(sys.argv[1],'w') as f:json.dump({'version':1,'mode':'installationGate','clientRequirement':req,'helperRequirement':req},f)
with open(sys.argv[3],'wb') as f:plistlib.dump({'Label':'com.coqui.CoquiPCCleaner.Helper','ProgramArguments':['/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper','--serve'],'MachServices':{'com.coqui.CoquiPCCleaner.Helper':True},'AssociatedBundleIdentifiers':['com.coqui.CoquiPCCleaner'],'ProcessType':'Adaptive'},f)
PY
cat > "$stage/scripts/preinstall" <<'PRE'
#!/bin/sh
set -eu
for path in '/Library' '/Library/Application Support' '/Library/PrivilegedHelperTools' '/Library/LaunchDaemons' '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper'; do
  if [ -L "$path" ]; then echo "Refusing a linked installation directory: $path" >&2; exit 1; fi
  if [ -e "$path" ]; then
    [ -d "$path" ] && [ "$(/usr/bin/stat -f %u "$path")" = 0 ] || exit 1
    mode=$(/usr/bin/stat -f %Lp "$path")
    [ $((0$mode & 0022)) = 0 ] || { echo "Installation directory is writable by another user: $path" >&2; exit 1; }
  fi
done
for path in '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper' '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist' '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json'; do
  [ ! -L "$path" ] || exit 1
  if [ -e "$path" ]; then [ -f "$path" ] && [ "$(/usr/bin/stat -f %u "$path")" = 0 ] || exit 1; fi
done
PRE
chmod 755 "$stage/scripts/preinstall"
cat > "$stage/scripts/postinstall" <<'POST'
#!/bin/sh
set -eu
/bin/chmod 755 '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper'
/usr/sbin/chown root:wheel '/Library/Application Support/CoquiPCCleaner' '/Library/Application Support/CoquiPCCleaner/Helper' '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json' '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist' '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper'
/bin/chmod 644 '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json' '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist'
/bin/chmod 755 '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper'
/bin/launchctl bootout system/com.coqui.CoquiPCCleaner.Helper 2>/dev/null || true
/bin/launchctl bootstrap system '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist'
POST
chmod 755 "$stage/scripts/postinstall"
/usr/bin/pkgbuild --root "$stage/payload" --scripts "$stage/scripts" --identifier com.coqui.CoquiPCCleaner.HelperGate --version 0.8.0 --install-location / --ownership recommended release/helper-gate/Coqui-Helper-Gate.pkg
cp "$helper_root/com.coqui.CoquiPCCleaner.Helper" release/helper-gate/coqui-helper-probe
mkdir -p "$stage/remove"
cat > "$stage/remove/postinstall" <<'POST'
#!/bin/sh
set -eu
/bin/launchctl bootout system/com.coqui.CoquiPCCleaner.Helper 2>/dev/null || true
/bin/rm -f '/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist' '/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper' '/Library/Application Support/CoquiPCCleaner/Helper/pairing.json'
/bin/rmdir '/Library/Application Support/CoquiPCCleaner/Helper' 2>/dev/null || true
POST
chmod 755 "$stage/remove/postinstall"
/usr/bin/pkgbuild --nopayload --scripts "$stage/remove" --identifier com.coqui.CoquiPCCleaner.HelperGateRemoval --version 0.8.0 release/helper-gate/Remove-Coqui-Helper-Gate.pkg
/usr/bin/shasum -a 256 release/helper-gate/*.pkg > release/helper-gate/SHA256SUMS
