"""Verify archived whole-worker images against both gates and the paid run."""
import hashlib
import json
from pathlib import Path
import tarfile

d = Path(__file__).resolve().parent
manifest = json.loads((d / 'manifest.json').read_text())
stored = manifest['binaries_and_corpus']
names = ['original', 'singleton']
expected = [
    '2fa4fb7fd78367e124d22aa5d07abc4ee32806839ef59801d99cdc6f56ecc1b1',
    json.loads((d.parent / 'official' / 'hashes.json').read_text())['binary_sha256'],
]
assert len(stored) == 3 and all(v in stored.values() for v in expected)
assert expected[0] in (d.parent / 'asm/both0-gate/README.md').read_text()
with tarfile.open(d / 'workers.tar.xz') as archive:
    assert archive.getnames() == names
    for name, digest in zip(names, expected):
        member = archive.getmember(name)
        assert member.isfile() and member.mode & 0o111
        contents = archive.extractfile(member)
        assert hashlib.file_digest(contents, 'sha256').hexdigest() == digest
        print(name, digest)
