"""One frozen 23-execution screen. No timing rows and no automatic replacements."""

import filecmp
import gzip
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

from prepare import HERE, WORK, sha


def read_memory(stderr, waited):
    records = []
    for line in stderr.read_text().splitlines():
        if line.startswith('{"memory"') or '"memory":' in line:
            records.append(json.loads(line))
    assert len(records) == 1, "exactly one native memory record required"
    memory = records[0]["memory"]
    assert memory["startup"]["pid"] == memory["end"]["pid"] == waited["pid"] == waited["waited_pid"]
    for sample in memory.values():
        assert 0 < sample["rss_kib"] <= sample["hwm_kib"]
    assert memory["end"]["hwm_kib"] >= memory["startup"]["hwm_kib"]
    assert abs(waited["wait4_hwm_kib"] - memory["end"]["hwm_kib"]) <= 4096, (waited, memory)
    return memory


def launch(out, key, binary, args):
    native = out / (key + ".wait4.json")
    stderr = out / (key + ".stderr")
    with (out / (key + ".stdout")).open("x") as stdout, stderr.open("x") as err:
        result = subprocess.run(
            [str(WORK / "bin/launcher"), str(native), str(binary), *map(str, args)],
            stdout=stdout,
            stderr=err,
            env={**os.environ, "AB_SHA256": "1"},
            timeout=40,
        )
    waited = json.loads(native.read_text())
    # Retain raw output/status before any assertion, including failure paths.
    assert not waited["timeout"] and result.returncode == waited["exit"] == 0, (key, waited)
    memory = read_memory(stderr, waited)
    return {"wait4": waited, "native": memory}


def controls_ok(low, high):
    assert high["native"]["end"]["hwm_kib"] - low["native"]["end"]["hwm_kib"] >= 32768, (
        "positive controls lack 32 MiB separation"
    )


def check_blocks(row, cells):
    assert row["clock_calls"] == 0, "untimed counters only"
    assert all(p["inclusive_ns"] == p["exclusive_ns"] == 0 for p in row["phases"].values())
    assert row["error"] is None
    for block in row["blocks"]:
        c = block["counters"]
        assert c["computed_crc"] == block["stored_crc"]
        if not cells:
            continue
        if c.get("outer_runs"):
            assert c.get("attempted_cells", 0) <= c["budget"]
            if block["outcome"].startswith("Ok("):
                for field in (
                    "period",
                    "small_seeds",
                    "long_seeds",
                    "max_unmarked_factor",
                    "reachable_expansion_depth",
                    "root_depth",
                ):
                    assert field in c, field
            else:
                assert c["fallback_calls"] == 1


def run():
    out = WORK / "screen"
    out.mkdir()  # refuse a second attempt in the same prepared tree
    started = time.monotonic()  # deadline only, never reported as performance
    pins = json.loads((HERE / "inputs.json").read_text())
    summary = {"kind": "untimed", "corpora": {}, "controls": {}, "executions": 0}
    try:
        for name, (size, _, expected) in pins.items():
            path = WORK / "inputs" / (name + ".bz2")
            assert path.stat().st_size == size and sha(path) == expected, name
        build = json.loads((WORK / "build.json").read_text())
        for name, expected in build["binaries"].items():
            assert sha(WORK / "bin" / name) == expected
        for mib in (1, 64):
            summary["executions"] += 1
            summary["controls"][str(mib)] = launch(
                out, f"control-{mib}", WORK / "bin/baseline", ["control", mib]
            )
        controls_ok(summary["controls"]["1"], summary["controls"]["64"])
        for name, (_, decoded_size, _) in pins.items():
            family = {}
            summary["corpora"][name] = family
            for arm in ("baseline", "cells", "oracle"):
                assert time.monotonic() - started < 900, "whole screen deadline"
                key = name + "-" + arm
                output = out / (key + ".decoded")
                summary["executions"] += 1
                memory = launch(
                    out,
                    key,
                    WORK / "bin" / arm,
                    ["check", WORK / "inputs" / (name + ".bz2"), output],
                )
                row = json.loads((out / (key + ".stdout")).read_text())
                assert row["bytes"] == output.stat().st_size == decoded_size
                digest = sha(output)
                assert row["sha256"] == digest
                if arm != "oracle":
                    check_blocks(row, arm == "cells")
                family[arm] = {"sha256": digest, "bytes": decoded_size, "memory": memory}
            oracle = out / (name + "-oracle.decoded")
            for arm in ("baseline", "cells"):
                assert filecmp.cmp(oracle, out / (name + "-" + arm + ".decoded"), shallow=False), (
                    name
                )
                assert family[arm]["sha256"] == family["oracle"]["sha256"]
            # All three complete byte streams survive in a compact, lossless artifact.
            for arm in ("baseline", "cells", "oracle"):
                path = out / (name + "-" + arm + ".decoded")
                with path.open("rb") as source, gzip.open(str(path) + ".gz", "xb") as dest:
                    shutil.copyfileobj(source, dest)
                path.unlink()
        assert summary["executions"] == 23
        (out / "COMPLETE").write_text("23 untimed executions; no timing conclusion\n")
    except BaseException as error:
        (out / "FAILED").write_text(repr(error) + "\n")
        raise
    finally:
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    run()
