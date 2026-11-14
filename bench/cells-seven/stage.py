"""Generate the exact six synthetic inputs and verify all seven compressed pins."""

import json
import shutil
import subprocess
import sys
from pathlib import Path

from prepare import HERE, WORK, sha


def stage(wikidata):
    inputs = WORK / "inputs"
    inputs.mkdir()
    with (WORK / "generator.log").open("x") as log:
        subprocess.run(
            [
                "cargo",
                "+1.99.1",
                "run",
                "--release",
                "--locked",
                "--manifest-path",
                str(WORK / "generator/Cargo.toml"),
                "--example",
                "ab",
                "--",
                "gen",
                str(inputs),
                "4",
            ],
            stdout=log,
            stderr=subprocess.STDOUT,
            check=True,
            timeout=1800,
        )
    shutil.copyfile(wikidata, inputs / "wikidata.bz2")
    pins = json.loads((HERE / "inputs.json").read_text())
    records = {}
    for name, (size, decoded_size, expected) in pins.items():
        path = inputs / (name + ".bz2")
        actual = sha(path)
        assert (path.stat().st_size, actual) == (size, expected), (name, actual)
        records[name] = {
            "compressed_bytes": size,
            "decoded_bytes": decoded_size,
            "compressed_sha256": actual,
        }
    (WORK / "inputs.json").write_text(json.dumps(records, indent=2) + "\n")
    print(json.dumps(records))


if __name__ == "__main__":
    stage(Path(sys.argv[1]).resolve(strict=True))
