"""Fixed live-reader diagnostic. `plan` is inert; `run` is for later authorised use.

Never retries or extends. Fresh directory per attempt, failures remain on disk.
"""

import argparse
import bz2
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
CORPORA = ("wikidata", "pattern", "onebyte")
DIGESTS = {
    "wikidata": "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
    "pattern": "0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407",
    "onebyte": "22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e",
}
SIZES = {"wikidata": 209700196, "pattern": 19600000, "onebyte": 20000000}
PIN = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"


def digest(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as f:
        while data := f.read(1 << 20):
            h.update(data)
    return h.hexdigest()


def schedule():
    jobs = []
    for corpus in CORPORA:
        for arm in ("baseline", "cells", "oracle"):
            jobs.append(dict(corpus=corpus, arm=arm, trace=arm != "oracle", mode="check"))
    for corpus in CORPORA:
        for round_ in range(10):
            traces = (False, True) if round_ % 2 == 0 else (True, False)
            arms = ("baseline", "cells") if round_ % 4 in (0, 3) else ("cells", "baseline")
            for trace in traces:
                for arm in arms:
                    jobs.append(
                        dict(
                            corpus=corpus,
                            arm=arm,
                            trace=trace,
                            round=round_,
                            mode="measure",
                            settle_s=0.25,
                        )
                    )
    return jobs


def binary_key(job):
    return job["arm"] + ("-on" if job["trace"] else "-off")


def checked_child(command, output, error, env=None, measured=False, job=None):
    with Path(output).open("x") as out, Path(error).open("x") as err:
        resources = None
        if measured:
            record = dict(
                command=command,
                job=job,
                pid=None,
                status="not_started",
                wait_status=None,
                returncode=None,
                resources=None,
                validation="pending",
            )
            # Reserve before launching; never overwrite a prior attempt. Flush the
            # collected tuple before any exit/JSON/decoder rejection can discard it.
            with Path(output).with_suffix(".resources").open("x") as sidecar:

                def persist():
                    sidecar.seek(0)
                    sidecar.write(json.dumps(record) + "\n")
                    sidecar.truncate()
                    sidecar.flush()

                persist()
                try:
                    start = time.perf_counter_ns()
                    record["status"] = "spawn_failed"
                    with subprocess.Popen(command, stdout=out, stderr=err, env=env) as child:
                        record.update(pid=child.pid, status="wait_failed")
                        _, status, usage = os.wait4(child.pid, 0)
                        wall = time.perf_counter_ns() - start
                        child.returncode = os.waitstatus_to_exitcode(status)
                        resources = dict(
                            wall_ns=wall,
                            cpu_s=usage.ru_utime + usage.ru_stime,
                            user_s=usage.ru_utime,
                            system_s=usage.ru_stime,
                            rss_kib=usage.ru_maxrss,
                        )
                        record.update(
                            status="exited",
                            wait_status=status,
                            returncode=child.returncode,
                            resources=resources,
                        )
                        persist()
                        if child.returncode:
                            record["validation"] = "nonzero_exit"
                            raise subprocess.CalledProcessError(child.returncode, command)
                    record["validation"] = "malformed_json"
                    row = json.loads(Path(output).read_text())
                    record["validation"] = "decoder_error"
                    if row.get("error"):
                        raise ValueError(f"decoder error: {row['error']}")
                    record["validation"] = "accepted"
                    persist()
                except BaseException as failure:
                    record["error"] = repr(failure)
                    persist()
                    raise
            return row, resources
        else:
            subprocess.run(command, stdout=out, stderr=err, env=env, check=True)
    row = json.loads(Path(output).read_text())
    if row.get("error"):
        raise ValueError(f"decoder error: {row['error']}")
    return (row, resources) if measured else row


def reconcile(row):
    if not row["trace"]:
        if "phases" in row or "blocks" in row:
            raise ValueError("off build emitted instrumentation")
        return
    phases = row["phases"]
    if row["timed"]:
        total = phases["whole_reader_drain"]["inclusive_ns"]
        if sum(p["exclusive_ns"] for p in phases.values()) != total:
            raise ValueError("exclusive phase reconciliation failed")
        if row["clock_calls"] != 2 * sum(p["calls"] for p in phases.values()):
            raise ValueError("clock count mismatch")
    elif row["clock_calls"] or any(p["inclusive_ns"] or p["exclusive_ns"] for p in phases.values()):
        raise ValueError("untimed run read clocks")
    for b in row["blocks"]:
        c = b["counters"]
        if c.get("computed_crc") != b["stored_crc"]:
            raise ValueError("block CRC mismatch or unfinished block")
        if row["arm"] == "cells" and c.get("outer_runs"):
            if not b["outcome"]:
                raise ValueError("missing route")
            if b["outcome"].startswith("Err(") and c.get("fallback_calls") != 1:
                raise ValueError("refusal failed to reach fallback")
            if c.get("attempted_cells", 0) > c["budget"]:
                raise ValueError("budget exceeded")


def interval(values):
    if len(values) != 10 or any(not math.isfinite(v) or v <= 0 for v in values):
        raise ValueError("need ten finite positive paired ratios")
    logs = [math.log(v) for v in values]
    mean = statistics.mean(logs)
    half = 2.2621571628540993 * statistics.stdev(logs) / math.sqrt(10)
    lo, hi = math.exp(mean - half), math.exp(mean + half)
    return dict(
        ratio=math.exp(mean),
        low=lo,
        high=hi,
        conclusion="faster" if hi < 1 else "slower" if lo > 1 else "unresolved",
    )


def analyse(rows):
    if len(rows) != 120:
        raise ValueError("incomplete fixed programme")
    index = {}
    for r in rows:
        key = (r["corpus"], r["round"], r["arm"], r["trace"])
        if key in index:
            raise ValueError("duplicate decode")
        index[key] = r
    out = {}
    for corpus in CORPORA:
        pairs = {}
        for trace in (False, True):
            pairs[str(trace)] = interval(
                [
                    index[corpus, i, "cells", trace]["wall_ns"]
                    / index[corpus, i, "baseline", trace]["wall_ns"]
                    for i in range(10)
                ]
            )
        inflation = {
            arm: interval(
                [
                    index[corpus, i, arm, True]["wall_ns"] / index[corpus, i, arm, False]["wall_ns"]
                    for i in range(10)
                ]
            )
            for arm in ("baseline", "cells")
        }
        phase_ok = True
        try:
            for r in rows:
                if r["corpus"] == corpus:
                    reconcile(r)
        except (KeyError, ValueError):
            phase_ok = False
        # Arithmetic is clustered by complete decode. Inclusive inversion is a
        # diagnostic view, NEVER another summand beside its exclusive subphases.
        cost = None
        if phase_ok:
            traced = [
                index[corpus, i, arm, True] for i in range(10) for arm in ("baseline", "cells")
            ]
            names = sorted({name for r in traced for name in r["phases"]})

            def exclusive(r, name):
                if name == "process_outside_reader":
                    return r["wall_ns"] - r["phases"]["whole_reader_drain"]["inclusive_ns"]
                return r["phases"].get(name, {}).get("exclusive_ns", 0)

            names.append("process_outside_reader")
            for r in traced:
                if (
                    exclusive(r, "process_outside_reader") < 0
                    or sum(exclusive(r, name) for name in names) != r["wall_ns"]
                ):
                    phase_ok = False
            if phase_ok:
                table = {}
                for name in names:
                    a = [exclusive(index[corpus, i, "baseline", True], name) for i in range(10)]
                    b = [exclusive(index[corpus, i, "cells", True], name) for i in range(10)]
                    table[name] = dict(
                        baseline_mean_ns=statistics.mean(a),
                        cells_mean_ns=statistics.mean(b),
                        paired_delta_ns=[y - x for x, y in zip(a, b)],
                    )
                cost = dict(
                    exclusive=table,
                    boundary="instrumented live path; process remainder includes startup, serialization and logging, not an isolated log estimate",
                )
                if all("inversion" in r["phases"] for r in traced):
                    cost["inclusive_inversion_ratio"] = interval(
                        [
                            index[corpus, i, "cells", True]["phases"]["inversion"]["inclusive_ns"]
                            / index[corpus, i, "baseline", True]["phases"]["inversion"][
                                "inclusive_ns"
                            ]
                            for i in range(10)
                        ]
                    )
        out[corpus] = dict(
            cost_reconciliation=cost,
            cells_over_baseline=pairs,
            inflation=inflation,
            quantitative_attribution_valid=phase_ok
            and pairs["False"]["conclusion"] == pairs["True"]["conclusion"],
        )
    for value in out.values():
        if not value["quantitative_attribution_valid"]:
            value["cost_reconciliation"] = None
            value["next_action"] = (
                "Stop quantitative attribution; retain raw descriptive counts and request Coordinator disposition."
            )
        else:
            value["next_action"] = (
                "Price the largest positive exclusive cost against eligible inversion saving; warm research/port cause remains unmeasured and requires separate admission."
            )
    return out


def oracle(path, output):
    h = hashlib.sha256()
    n = 0
    with bz2.open(path, "rb") as f, Path(output).open("xb") as out:
        while data := f.read(4 << 20):
            h.update(data)
            n += len(data)
            out.write(data)
    return dict(sha256=h.hexdigest(), bytes=n)


def equal_files(a, b):
    with Path(a).open("rb") as x, Path(b).open("rb") as y:
        while True:
            left, right = x.read(4 << 20), y.read(4 << 20)
            if left != right:
                raise ValueError("full byte mismatch")
            if not left:
                return


def run(binaries, corpora, directory, cpu):
    directory.mkdir()  # evidence never overwritten
    rows = []
    try:
        if cpu not in os.sched_getaffinity(0):
            raise ValueError("CPU outside allowed affinity")
        build = json.loads((binaries / "build.json").read_text())
        if build.get("tracked_dirty", True) or len(build.get("diagnostic_head", "")) != 40:
            raise ValueError("measurement needs a recorded clean diagnostic commit")
        if (
            build["pin"] != PIN
            or build["rustc"].splitlines()[0] != "rustc 1.99.1 (58cf8f9c3 2026-10-13)"
        ):
            # Exact full compiler identity is recorded by build.py; see toolchain.txt.
            raise ValueError("source/compiler mismatch")
        expected_keys = {a + "-" + t for a in ("baseline", "cells") for t in ("off", "on")}
        if set(build["binaries"]) != expected_keys:
            raise ValueError("missing/extra build arm")
        for path, identity in build["harness"].items():
            if digest(HERE / path) != identity:
                raise ValueError("harness changed after build: " + path)
        for key, identity in build["binaries"].items():
            if digest(binaries / key) != identity:
                raise ValueError("executable digest mismatch")
        for corpus in CORPORA:
            if digest(corpora / f"{corpus}.bz2") != DIGESTS[corpus]:
                raise ValueError("corpus digest mismatch")
        env = os.environ.copy()
        for key in list(env):
            if key.startswith("AB_"):
                del env[key]
        manifest = dict(
            build=build,
            cpu=cpu,
            affinity=sorted(os.sched_getaffinity(0)),
            platform=platform.platform(),
            lscpu=subprocess.check_output(["lscpu"], text=True),
            schedule=schedule(),
            input_sha256=DIGESTS,
        )
        (directory / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        checks = {}
        for i, job in enumerate(schedule()):
            corpus = job["corpus"]
            source = corpora / f"{corpus}.bz2"
            stem = directory / f"{i:03d}-{corpus}-{job['arm']}"
            if job["arm"] == "oracle":
                reference = oracle(source, str(stem) + ".bytes")
                a, b = checks[corpus, "baseline"], checks[corpus, "cells"]
                if (
                    any(
                        (r["bytes"], r["sha256"]) != (reference["bytes"], reference["sha256"])
                        for r, _ in (a, b)
                    )
                    or reference["bytes"] != SIZES[corpus]
                ):
                    raise ValueError("oracle length/SHA mismatch")
                for _, path in (a, b):
                    equal_files(path, str(stem) + ".bytes")
                (Path(str(stem) + ".json")).write_text(json.dumps(reference) + "\n")
                continue
            command = [
                "taskset",
                "-c",
                str(cpu),
                str((binaries / binary_key(job)).resolve()),
                job["mode"],
                str(source.resolve()),
            ]
            child_env = env.copy()
            if job["mode"] == "check":
                command.append(str(stem) + ".bytes")
                child_env["AB_SHA256"] = "1"
                row = checked_child(command, str(stem) + ".json", str(stem) + ".stderr", child_env)
                if (row["arm"], row["trace"], row["timed"]) != (job["arm"], True, False):
                    raise ValueError("wrong precheck executable/build/mode")
                reconcile(row)
                checks[corpus, job["arm"]] = row, str(stem) + ".bytes"
            else:
                # No in-process timer or rusage contamination from previous children.
                time.sleep(job["settle_s"])
                row, resources = checked_child(
                    command,
                    str(stem) + ".json",
                    str(stem) + ".stderr",
                    child_env,
                    measured=True,
                    job=dict(index=i, **job),
                )
                expected = checks[corpus, job["arm"]][0]
                if (row["bytes"], row["hash"]) != (expected["bytes"], expected["hash"]):
                    raise ValueError("measured output mismatch")
                if (row["arm"], row["trace"], row["timed"]) != (job["arm"], job["trace"], True):
                    raise ValueError("wrong executable/build/mode")
                row.update(job, **resources)
                reconcile(row)
                rows.append(row)
                with (directory / "runs.jsonl").open("a") as f:
                    f.write(json.dumps(row) + "\n")
        (directory / "analysis.json").write_text(json.dumps(analyse(rows), indent=2) + "\n")
        (directory / "COMPLETE").write_text(
            "120 measured decodes +9 checks; no production qualification\n"
        )
    except BaseException as error:
        (directory / "FAILED.json").write_text(
            json.dumps({"error": repr(error), "completed_measured": len(rows)}) + "\n"
        )
        raise


def main():
    parser = argparse.ArgumentParser(__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    sub.add_parser("plan")
    a = sub.add_parser("analyse")
    a.add_argument("records", type=Path)
    p = sub.add_parser("run")
    p.add_argument("binaries", type=Path)
    p.add_argument("corpora", type=Path)
    p.add_argument("directory", type=Path)
    p.add_argument("--cpu", type=int, required=True)
    args = parser.parse_args()
    if args.mode == "plan":
        print(
            json.dumps({"measured_decodes": 120, "untimed_checks": 9, "jobs": schedule()}, indent=2)
        )
    elif args.mode == "analyse":
        print(
            json.dumps(
                analyse([json.loads(line) for line in args.records.read_text().splitlines()]),
                indent=2,
            )
        )
    else:
        run(args.binaries, args.corpora, args.directory, args.cpu)


if __name__ == "__main__":
    main()
