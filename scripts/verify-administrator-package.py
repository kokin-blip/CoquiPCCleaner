#!/usr/bin/env python3
"""Verify Coqui's local package payload and exact build pairing without installing it."""
from pathlib import Path
import json, plistlib, re, subprocess, sys, tempfile, uuid
project=Path(__file__).resolve().parent.parent
version=json.loads((project/'package.json').read_text())['version']
package=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else project/f'release/{version}/CoquiPCCleaner-{version}-administrator-arm64.pkg'
with tempfile.TemporaryDirectory(prefix='coqui-administrator-verification-',dir='/private/tmp') as temporary:
    expanded=Path(temporary)/'expanded'
    subprocess.run(['/usr/sbin/pkgutil','--expand-full',str(package),str(expanded)],check=True)
    manifests=list(expanded.rglob('pairing.json'))
    assert len(manifests)==1
    manifest=json.loads(manifests[0].read_text())
    assert manifest['version']==2 and manifest['mode']=='production'
    assert manifest['applicationPath']=='/Applications/CoquiPCCleaner.app'
    assert manifest['clientExecutable']=='/Applications/CoquiPCCleaner.app/Contents/MacOS/coqui-pc-cleaner'
    app=next(expanded.rglob('CoquiPCCleaner.app'))
    helper=next(p for p in expanded.rglob('com.coqui.CoquiPCCleaner.Helper') if p.is_file())
    for binary,key,identifier in [(app,'applicationRequirement','com.coqui.CoquiPCCleaner'),(helper,'helperRequirement','com.coqui.CoquiPCCleaner.Helper')]:
        assert re.fullmatch(r'identifier "'+re.escape(identifier)+r'" and cdhash H"[a-fA-F0-9]{40}"',manifest[key])
        subprocess.run(['/usr/bin/codesign','--verify','--strict','-R','='+manifest[key],str(binary)],check=True)
        subprocess.run(['/usr/bin/codesign','--verify','--strict','-R','='+manifest['clientRequirement'],str(binary)],check=True)
        info=subprocess.run(['/usr/bin/codesign','-d','--verbose=4',str(binary)],capture_output=True,text=True,check=True).stderr
        assert 'runtime' in info, 'Missing hardened runtime'
        rejected=subprocess.run(['/usr/bin/codesign','--verify','--strict','-R','=cdhash H"'+'0'*40+'"',str(binary)],capture_output=True)
        assert rejected.returncode!=0
    info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
    assert info['CFBundleShortVersionString']==version and info['LSMinimumSystemVersion']=='14.0'
    architecture=subprocess.check_output(['/usr/bin/file',str(app/'Contents/MacOS/coqui-pc-cleaner')],text=True)
    assert 'arm64' in architecture
    daemon=plistlib.loads(next(expanded.rglob('com.coqui.CoquiPCCleaner.Helper.plist')).read_bytes())
    assert daemon['ProgramArguments']==['/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper','--serve']
    assert daemon['MachServices']=={'com.coqui.CoquiPCCleaner.Helper':True}
    assert not daemon.get('RunAtLoad') and not daemon.get('KeepAlive')
    fixtures=list(expanded.rglob('validation.json'))
    if fixtures:
        assert len(fixtures)==1
        fixture=json.loads(fixtures[0].read_text());uuid.UUID(fixture['id'])
        assert fixture['purpose']=='coqui-disposable-administrator-validation'
        apps=list(expanded.rglob('Coqui Administrator Fixture '+fixture['id']+'.app'));assert len(apps)==1
        assert plistlib.loads((apps[0]/'Contents/Info.plist').read_bytes())['CFBundleIdentifier']=='org.coqui.validation.'+fixture['id']
        cache=next(expanded.rglob('org.coqui.validation.'+fixture['id']))
        assert plistlib.loads((cache/'CoquiFixtureMarker.plist').read_bytes())['FixtureID']==fixture['id']
    print(json.dumps({'package':package.name,'productionPairing':manifest,'architecture':'arm64','minimumOS':'14.0','hardenedRuntime':True,'disposableFixtures':bool(fixtures),'qualification':'Payload and signing checks only; native installation and operations are separate validation.'}))
