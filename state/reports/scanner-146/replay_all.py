#!/usr/bin/env python3
"""Verify pinned inputs and replay static evidence/models; never run project code."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--evidence-root', type=Path)
    group.add_argument('--git-repo', type=Path)
    parser.add_argument('--output-dir', type=Path, required=True)
    args = parser.parse_args()
    package = Path(__file__).resolve().parent
    out = args.output_dir.resolve()
    # Never let regenerated output overwrite the archived review package.
    if out == package or package in out.parents or out in package.parents:
        parser.error('output directory must be separate from the review package')
    out.mkdir(parents=True, exist_ok=True)
    manifest_path = package / 'input-manifest.json'
    manifest = json.loads(manifest_path.read_text())
    root = args.evidence_root.resolve() if args.evidence_root else out / 'evidence'
    if args.evidence_root and (out == root or root in out.parents or out in root.parents):
        parser.error('output directory must be separate from the evidence root')
    for item in manifest['evidence_files']:
        path = root / item['path']
        if args.git_repo:
            data = subprocess.check_output([
                'git', '-C', str(args.git_repo), 'show',
                manifest['checkpoint'] + ':' + item['path']])
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        else:
            data = path.read_bytes()
        assert len(data) == item['bytes'], item['path']
        assert blob(data) == item['git_blob'], item['path']
        assert hashlib.sha256(data).hexdigest() == item['sha256'], item['path']
    if args.git_repo:
        for item in manifest['governance']:
            data = subprocess.check_output(['git', '-C', str(args.git_repo), 'show',
                                           manifest['base'] + ':' + item['path']])
            assert blob(data) == item['git_blob'], item['path']
        for item in manifest['frozen']:
            data = subprocess.check_output(['git', '-C', str(args.git_repo), 'show',
                                           item['commit'] + ':' + item['path']])
            assert blob(data) == item['git_blob'], item['path']
    tasks = [
        ('accounting/reconstruct.py', ['--root', str(root)], 'accounting/reconstruction.json'),
        ('safety/replay_safety.py', [str(root)], 'safety/results.json'),
        ('phase/model.py', ['--repo', str(root)], 'phase/results.json'),
        ('fresh-check/check_totals.py', [str(root)], 'fresh-check/check_totals.json'),
    ]
    results = []
    for script, options, expected in tasks:
        data = subprocess.check_output([sys.executable, str(package / script), *options])
        actual = json.loads(data)
        saved = json.loads((package / expected).read_text())
        assert actual == saved, 'replay mismatch: ' + script
        target = out / expected
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        results.append({'script': script, 'matches_saved_json': True})
    summary = {
        'checkpoint': manifest['checkpoint'],
        'manifest_sha256': hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
        'verified_files': len(manifest['evidence_files']),
        'governance_and_historical_blobs_replayed': bool(args.git_repo),
        'static_replays': results,
        'constructor_execution': False,
        'compiler_reproduction': False,
        'new_measurements': False,
    }
    (out / 'validation.json').write_text(json.dumps(summary, indent=2, sort_keys=True) + '\n')
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
