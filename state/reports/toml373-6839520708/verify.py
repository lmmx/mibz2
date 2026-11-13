"""Check the two live manifest edits without regenerating frozen evidence."""
import hashlib
import json
import subprocess
import tomllib
from pathlib import Path

ROOT = Path('/workspace/mibz2-live')
BASE = 'ed6f424f238851ff8296e7a48c2b96dfb8332e7e'
HISTORICAL = 'a08277e1c36e276cfea7b3116d99ea5fd93f0e84'
PIN = 'd4f0e8fb3ac68fa649e1810dff58dc06e9c545c2'
ALLOWED = {'bench/cells-gap/Cargo.toml', 'bench/cells-gap/live/Cargo.toml'}

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args])

def sha(data):
    return hashlib.sha256(data).hexdigest()

paths = git('ls-tree', '-r', '--name-only', BASE).decode().splitlines()
changed = []
for path in paths:
    if (ROOT / path).read_bytes() != git('show', f'{BASE}:{path}'):
        changed.append(path)
assert set(changed) == ALLOWED, changed
records = {}
for path in sorted(ALLOWED):
    old = git('show', f'{BASE}:{path}')
    new = (ROOT / path).read_bytes()
    assert tomllib.loads(old.decode()) == tomllib.loads(new.decode()), path
    records[path] = {'old_sha256': sha(old), 'new_sha256': sha(new), 'parsed_toml_identical': True}

journal = ROOT / 'docs/journal/2026-11-12-cells-live'
archives = {}
for folder in ['evidence', 'r1']:
    data = json.loads((journal / folder / 'SHA256.json').read_text())
    for name, expected in data.items():
        assert sha((journal / folder / name).read_bytes()) == expected, (folder, name)
    archives[folder] = len(data)
source = json.loads((journal / 'evidence/source-correspondence.json').read_text())
assert source['pin'] == PIN
for path, expected in source['original'].items():
    assert sha(git('show', f'{PIN}:{path}')) == expected, path
plan = 'docs/journal/2026-11-12-cells-live/PLAN.md'
assert (ROOT / plan).read_bytes() == git('show', f'3b43af956d553905ea744f6cb6e1c785a0f8297a:{plan}')

# The old census manifest is a record of its historical source, not a new
# assertion that a formatting-only follow-up has the same live file hash.
historical = ROOT / 'docs/journal/monodromy/2026-11-10-cells-gap/evidence/SOURCE-SHA256SUMS'
historical_count = 0
for line in historical.read_text().splitlines():
    expected, path = line.split(maxsplit=1)
    assert sha(git('show', f'{HISTORICAL}:{path}')) == expected, path
    historical_count += 1

# Check the already-prepared overlay in place; do not regenerate/rebaseline it.
live = ROOT / 'bench/cells-gap/live'
prepared = json.loads((live / '.generated/source.json').read_text())
assert prepared['pin'] == PIN
assert prepared['original'] == source['original']
for path, original in prepared['original'].items():
    expected = prepared['edits'].get(path, {}).get('diagnostic', original)
    assert sha((live / '.source' / path).read_bytes()) == expected, path
assert sha((live / 'prepare.py').read_bytes()) == prepared['generator_sha256']
assert sha((live / 'src/trace.rs').read_bytes()) == prepared['trace_sha256']
assert sha((live / '.source/src/live_diagnostic.rs').read_bytes()) == prepared['trace_sha256']
assert sha((live / '.generated/hash.rs').read_bytes()) == prepared['hash_sha256']
patch = (live / '.generated/instrumentation.patch').read_bytes()
assert sha(patch) == prepared['patch_sha256']
assert not any(line.startswith(b'-') and not line.startswith(b'---') for line in patch.splitlines())

print(json.dumps({'base': BASE, 'changed': records, 'other_tracked_files_identical': len(paths)-2,
                  'archives_verified': archives, 'source_pin': PIN, 'original_hashes': len(source['original']),
                  'historical_source_hashes_verified_at_original_commit': historical_count,
                  'historical_live_manifest_hash_explicitly_superseded_only_for_formatting': True,
                  'plan_frozen_bytes_unchanged': True, 'prepared_overlay_unchanged': True,
                  'R1_driver_tests_schedule_and_estimator_unchanged': True,
                  'measurements': 0}, indent=2))
