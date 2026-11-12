"""Real executable dry run: tiny fixtures only; no clocks or corpus census."""

import bz2
import json
import os
import subprocess
import sys
from pathlib import Path

from driver import binary_key, checked_child, digest, equal_files, reconcile


def check(binaries, directory):
    directory.mkdir()
    env = os.environ.copy()
    env["AB_SHA256"] = "1"
    fixtures = {
        "empty": b"",
        "runs": b"abacaba" * 1000,
        "column": bytes(range(256)) * 10,
        "repetition": b"a" * 3000,
    }
    n = 0
    for name, raw in fixtures.items():
        source = directory / f"{name}.bz2"
        source.write_bytes(bz2.compress(raw, compresslevel=9))
        expected = directory / f"{name}.oracle"
        expected.write_bytes(bz2.decompress(source.read_bytes()))
        for arm in ("baseline", "cells"):
            for trace in (False, True):
                key = binary_key(dict(arm=arm, trace=trace))
                stem = directory / f"{name}-{key}"
                row = checked_child(
                    [str(binaries / key), "check", str(source), str(stem) + ".bytes"],
                    str(stem) + ".json",
                    str(stem) + ".stderr",
                    env,
                )
                assert (row["arm"], row["trace"], row["timed"]) == (arm, trace, False)
                assert (row["bytes"], row["sha256"]) == (len(raw), digest(expected))
                equal_files(str(stem) + ".bytes", expected)
                reconcile(row)
                n += 1
    # Real children: truncated stream, block CRC corruption, and OS I/O failure.
    good = bz2.compress(b"abacaba" * 1000, compresslevel=9)
    damaged = bytearray(good)
    damaged[10] ^= 1  # first block stored CRC, after BZh9 and block magic
    invalid = {"truncated": good[:-4], "block-crc": bytes(damaged)}
    errors = 0
    for name, data in invalid.items():
        source = directory / f"{name}.bz2"
        source.write_bytes(data)
        identities = []
        for key in ("baseline-off", "baseline-on", "cells-off", "cells-on"):
            stem = directory / f"error-{name}-{key}"
            try:
                checked_child(
                    [str(binaries / key), "check", str(source)],
                    str(stem) + ".json",
                    str(stem) + ".stderr",
                    env,
                )
            except subprocess.CalledProcessError as e:
                assert e.returncode != 0
            else:
                raise AssertionError("bad input accepted")
            row = json.loads(Path(str(stem) + ".json").read_text())
            identities.append((row["bytes"], row["hash"], row["error"]))
            assert row["error"]
            if row["trace"]:
                assert row["clock_calls"] == 0
            errors += 1
        assert all(i == identities[0] for i in identities), identities
    for key in ("baseline-off", "baseline-on", "cells-off", "cells-on"):
        stem = directory / f"io-{key}"
        try:
            checked_child(
                [str(binaries / key), "check", str(directory / "absent")],
                str(stem) + ".json",
                str(stem) + ".stderr",
                env,
            )
        except subprocess.CalledProcessError:
            errors += 1
        else:
            raise AssertionError("I/O failure accepted")
    result = dict(
        valid_child_checks=n,
        failure_child_checks=errors,
        clocks_read=0,
        full_byte_parity=True,
        error_parity=True,
        performance_measurements=0,
    )
    (directory / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    check(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())
