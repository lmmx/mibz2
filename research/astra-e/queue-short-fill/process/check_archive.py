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
    json.loads((d.parent / 'asm/both0-gate/manifest.json').read_text())['full_worker_sha256'],
    json.loads((d.parent / 'official' / 'hashes.json').read_text())['binary_sha256'],
]
assert len(stored) == 3 and all(v in stored.values() for v in expected)
with tarfile.open(d / 'workers.tar.xz') as archive:
    assert archive.getnames() == names
    for name, digest in zip(names, expected):
        member = archive.getmember(name)
        assert member.isfile() and member.mode & 0o111
        contents = archive.extractfile(member)
        assert hashlib.file_digest(contents, 'sha256').hexdigest() == digest
        print(name, digest)
