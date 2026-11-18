#!/usr/bin/env python3
"""Only shell changes require a new APK; Rust updates remain compatible."""
import hashlib,pathlib,sys
def shell_hash(kind,root=pathlib.Path('.')):
    if kind!='game': raise ValueError('Invalid kind')
    patterns=['android-shared/src/main/java/**/*.java','android-shell/**/*.kts',
              'android-shell/app/src/main/AndroidManifest.xml','android-shell/app/src/main/res/**/*']
    files=sorted({p for pattern in patterns for p in root.glob(pattern) if p.is_file()})
    if not files:raise ValueError('No shell source files')
    h=hashlib.sha256()
    for p in files:h.update(str(p.relative_to(root)).encode()+b'\0'+p.read_bytes()+b'\0')
    return h.hexdigest()
if __name__=='__main__':print(shell_hash(sys.argv[1]))
