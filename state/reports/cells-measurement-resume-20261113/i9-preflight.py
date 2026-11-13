"""Read-only machine/corpus facts; no decoder, timing, selection or admission."""
import argparse
import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("corpus_directory", type=Path, help="existing absolute directory")
args = parser.parse_args()
if not args.corpus_directory.is_absolute():
    parser.error("supply the actual absolute corpus directory; no default is inferred")
directory = args.corpus_directory.resolve(strict=True)
expected = {
    "wikidata": "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
    "pattern": "0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407",
    "onebyte": "22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e",
}
records = {}
for name, identity in expected.items():
    path = directory / (name + ".bz2")
    try:
        with path.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        records[name] = dict(path=str(path.resolve()), bytes=path.stat().st_size,
                             sha256=digest, matches_frozen=digest == identity)
    except OSError as error:
        records[name] = dict(path=str(path), error=str(error), matches_frozen=False)
allowed = sorted(os.sched_getaffinity(0))
topology = {}
for cpu in allowed:
    root = Path(f"/sys/devices/system/cpu/cpu{cpu}/topology")
    topology[str(cpu)] = {
        field: (root / field).read_text().strip()
        for field in ("physical_package_id", "core_id", "thread_siblings_list")
        if (root / field).is_file()
    }
print(json.dumps(dict(host=platform.node(), platform=platform.platform(),
                     allowed_cpus=allowed, topology=topology,
                     lscpu=subprocess.check_output(["lscpu", "--json"], text=True),
                     corpus_directory=str(directory), corpora=records,
                     selected_cpu=None, measurement_authorised=False), indent=2))
raise SystemExit(0 if all(row["matches_frozen"] for row in records.values()) else 1)
