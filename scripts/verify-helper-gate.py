#!/usr/bin/env python3
"""Inspect the diagnostic gate payload without installing or starting it."""
import hashlib
import json
from pathlib import Path
import plistlib
import re
import subprocess
import tempfile

project = Path(__file__).resolve().parent.parent
package = project / 'release/helper-gate/Coqui-Helper-Gate.pkg'
probe = project / 'release/helper-gate/coqui-helper-probe'
with tempfile.TemporaryDirectory(prefix='coqui-gate-verification-', dir='/private/tmp') as temporary:
    expanded = Path(temporary) / 'expanded'
    subprocess.run(['/usr/sbin/pkgutil', '--expand-full', str(package), str(expanded)], check=True)
    manifests = list(expanded.rglob('pairing.json'))
    assert len(manifests) == 1, 'Expected exactly one pairing manifest'
    manifest = json.loads(manifests[0].read_text())
    assert manifest['version'] == 1 and manifest['mode'] == 'installationGate'
    helpers = [p for p in expanded.rglob('com.coqui.CoquiPCCleaner.Helper') if p.is_file()]
    assert len(helpers) == 1, 'Expected exactly one helper executable'
    helper = helpers[0]
    assert hashlib.sha256(probe.read_bytes()).digest() == hashlib.sha256(helper.read_bytes()).digest(), 'Probe and payload differ'
    for key in ['clientRequirement', 'helperRequirement']:
        requirement = manifest[key]
        assert re.fullmatch(r'identifier "com\.coqui\.CoquiPCCleaner\.Helper" and cdhash H"[0-9a-fA-F]{40}"', requirement), 'Missing or invalid pinned code hash'
        for binary in [helper, probe]:
            subprocess.run(['/usr/bin/codesign', '--verify', '--strict', '-R', '=' + requirement, str(binary)], check=True)
    rejected = subprocess.run(['/usr/bin/codesign', '--verify', '--strict', '-R', '=cdhash H"' + '0' * 40 + '"', str(helper)], capture_output=True)
    assert rejected.returncode != 0, 'Mismatched build requirement unexpectedly accepted'
    daemon = plistlib.loads(next(expanded.rglob('com.coqui.CoquiPCCleaner.Helper.plist')).read_bytes())
    assert daemon['ProgramArguments'] == ['/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper', '--serve']
    assert daemon['MachServices'] == {'com.coqui.CoquiPCCleaner.Helper': True}
    assert not daemon.get('RunAtLoad') and not daemon.get('KeepAlive')
print('Gate payload/probe hash pins, mismatched-build rejection and on-demand service metadata verified; installation/connection/removal are separate native gates.')
