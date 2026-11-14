"""Preparation and validation around the existing ab runner; no new timing engine."""

import hashlib
import importlib.util
import json
import math
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
COMMON = "a8a23c11607864cb431f87fff38169e10c41685f"
ORIGINAL = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"
GENERATOR = "57f51bbc0ca62ac3ee1cc148a1010842eea6222a"
WINDOWS = "a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a"
CELLS_HASHES = {
    "original": "42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2",
    "selective": "3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee",
}
PINS = json.loads((HERE / "inputs.json").read_text())
ROUNDS = 12
FLAGS = "--cfg mibz2_cells_bench"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with Path(path).open("rb") as src:
        return hashlib.file_digest(src, "sha256").hexdigest()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + "\n")


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    sys.modules[name] = result
    spec.loader.exec_module(result)
    return result


def call(argv, out, *, cwd=ROOT, env=None, timeout=1800):
    """Keep command and combined output even if a preparation step fails."""
    with Path(out).open("x") as log:
        log.write(json.dumps([str(a) for a in argv]) + "\n")
        log.flush()
        subprocess.run(argv, cwd=cwd, env=env, stdout=log, stderr=log, check=True, timeout=timeout)


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def ensure(pin):
    if subprocess.run(
        ["git", "-C", str(ROOT), "cat-file", "-e", pin + "^{commit}"], capture_output=True
    ).returncode:
        subprocess.run(["git", "-C", str(ROOT), "fetch", "origin", pin], check=True, timeout=300)


def archive(pin, tree, out):
    ensure(pin)
    source = out / (pin + ".tar")
    if not source.exists():
        source.write_bytes(git("archive", pin))
    tree.mkdir()
    with tarfile.open(source) as stream:
        stream.extractall(tree, filter="data")


def tree_hashes(tree):
    return {str(p.relative_to(tree)): digest(p) for p in sorted(tree.rglob("*")) if p.is_file()}


def check_inputs(inputs):
    actual = {}
    for name, (size, decoded, wanted) in PINS.items():
        path = inputs / (name + ".bz2")
        require(path.stat().st_size == size and digest(path) == wanted, f"input mismatch: {name}")
        actual[name] = dict(bytes=size, decoded_bytes=decoded, sha256=wanted)
    return actual


def stage_wikidata(inputs, out):
    canonical = ROOT / "corpora/wikidata.bz2"
    wanted = PINS["wikidata"][2]
    if canonical.is_file() and digest(canonical) == wanted:
        shutil.copy2(canonical, inputs / "wikidata.bz2")
        return "verified canonical corpus"
    spec = json.loads((ROOT / "bench/wikidata.json").read_text())
    require(spec["corpus_sha256"] == wanted, "Wikidata manifest differs")
    ensure(WINDOWS)
    windows = out / "windows"
    windows.mkdir()
    for item in spec["windows"]:
        name = item["name"] + ".bin"
        (windows / name).write_bytes(git("show", f"{WINDOWS}:{spec['windows_dir']}/{name}"))
    corpus = module("cells_i9_corpus", ROOT / "bench/cells-gap/corpus.py")
    (inputs / "wikidata.bz2").write_bytes(corpus.rebuild(spec, windows))
    return "verified reconstruction from pinned windows"


def build_env(target):
    # Refuse inherited profile/target/compiler wrappers rather than silently compare
    # different builds. CARGO_HOME/RUSTUP_HOME remain usable tool/dependency caches.
    forbidden = [
        k
        for k in os.environ
        if k.startswith(("CARGO_PROFILE_", "CARGO_TARGET_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))
        or k in {"RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"}
    ]
    require(not forbidden, f"unregistered build overrides: {forbidden}")
    return dict(
        os.environ,
        RUSTUP_TOOLCHAIN="1.100.0",
        RUSTFLAGS=FLAGS,
        CARGO_TARGET_DIR=str(target),
        CARGO_BUILD_JOBS="4",
    )


def prepare(out):
    out = out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    require(not (out / "prepared.json").exists(), "refuse preparation reuse")
    (out / "commits.txt").write_text(
        f"head {git('rev-parse', 'HEAD').decode().strip()}\ncommon {COMMON}\noriginal {ORIGINAL}\n"
    )
    work = out.parent / (out.name + "-build")
    work.mkdir()  # Outside bundle; never reuse a partially built source tree.
    env = build_env(work / "generator-target")
    probe = ["rustup", "run", "1.100.0", "rustc", "-Vv"]
    if subprocess.run(probe, capture_output=True).returncode:
        call(
            ["rustup", "toolchain", "install", "1.100.0", "--profile", "minimal"],
            out / "toolchain-install.log",
        )
    compiler = subprocess.check_output(probe, text=True)
    require(
        "release: 1.100.0\n" in compiler and "host: x86_64-unknown-linux-gnu\n" in compiler,
        "wrong compiler or host",
    )
    (out / "rustc.txt").write_text(compiler)
    call(["cc", "--version"], out / "cc.txt")
    call(["ld", "--version"], out / "ld.txt")
    generator = work / "generator"
    archive(GENERATOR, generator, out)
    inputs = out / "inputs"
    call(
        ["cargo", "run", "--release", "--locked", "--example", "ab", "--", "gen", str(inputs), "4"],
        out / "generator.log",
        cwd=generator,
        env=env,
    )
    origin = stage_wikidata(inputs, out)
    identities = check_inputs(inputs)
    save(out / "inputs.json", identities)
    (out / "inputs.txt").write_text(
        "\n".join(f"{k} {v['sha256']}" for k, v in identities.items()) + "\n"
    )
    save(out / "generator-source.json", tree_hashes(generator))
    binary = out / "bin"
    binary.mkdir()
    manifests = {}
    for arm in ("original", "selective"):
        tree = work / arm
        archive(COMMON, tree, out)
        if arm == "original":
            ensure(ORIGINAL)
            (tree / "src/cells.rs").write_bytes(git("show", ORIGINAL + ":src/cells.rs"))
        require(digest(tree / "src/cells.rs") == CELLS_HASHES[arm], "Cells source mismatch")
        manifests[arm] = tree_hashes(tree)
        save(out / (arm + "-source.json"), manifests[arm])
        env = build_env(work / (arm + "-target"))
        call(
            ["cargo", "build", "--release", "--locked", "--example", "ab"],
            out / (arm + "-build.log"),
            cwd=tree,
            env=env,
        )
        shutil.copy2(Path(env["CARGO_TARGET_DIR"]) / "release/examples/ab", binary / arm)
        env["CELLS_PINNED_INPUTS"] = str(inputs)
        call(
            [
                "cargo",
                "test",
                "--release",
                "--locked",
                "--example",
                "ab",
                "cells_pinned_readers_byte_parity",
                "--",
                "--ignored",
                "--nocapture",
            ],
            out / (arm + "-parity.log"),
            cwd=tree,
            env=env,
            timeout=600,
        )
    changed = [
        p
        for p in manifests["original"]
        if manifests["original"][p] != manifests["selective"].get(p)
    ]
    require(
        changed == ["src/cells.rs"]
        and manifests["original"].keys() == manifests["selective"].keys(),
        f"unexpected common-tree differences: {changed}",
    )
    routes = {}
    for arm in ("original", "selective"):
        routes[arm] = [
            line
            for line in (out / (arm + "-parity.log")).read_text().splitlines()
            if line.startswith("CELLS_ROUTE ")
        ]
        require(len(routes[arm]) == 7, "missing full-byte oracle route records")
    require(routes["original"] == routes["selective"], "route differences")
    save(out / "routes.json", routes)
    source = work / "selective/bench/cells-selective"
    for filename, executable in (("launcher.c", "rss-launcher"), ("rss_control.c", "rss-control")):
        call(
            [
                "cc",
                "-O2",
                "-Wall",
                "-Wextra",
                "-Werror",
                str(source / filename),
                "-o",
                str(binary / executable),
            ],
            out / (executable + "-build.log"),
        )
    call([sys.executable, str(source / "rss_check.py")], out / "rss-check.log", cwd=out)
    record = dict(
        common=COMMON,
        original=ORIGINAL,
        generator=GENERATOR,
        window_pin=WINDOWS,
        wikidata_origin=origin,
        compiler=compiler,
        rustflags=FLAGS,
        source_differences=changed,
        rounds=ROUNDS,
        binary_sha256={arm: digest(binary / arm) for arm in ("original", "selective")},
        untimed_only=True,
    )
    save(out / "prepared.json", record)


def cpu_binding(allowed, topology):
    require(bool(allowed), "no allowed CPUs")
    cpu = min(allowed)
    item = topology[str(cpu)]
    siblings = item["siblings"]
    require(cpu in siblings and set(siblings) <= set(allowed), "incomplete sibling facts")
    core = (item["socket"], item["core"])
    physical = {int(c) for c, facts in topology.items() if (facts["socket"], facts["core"]) == core}
    require(physical == set(siblings), "inconsistent topology")
    parent = sorted(set(allowed) - physical)
    require(bool(parent), "no separate parent core")
    return dict(cpu=cpu, siblings=sorted(physical), parent=parent, allowed=sorted(allowed))


def preflight(out):
    helper = module("cells_i9_host", ROOT / "bench/share_nothing.py")
    directory = out / "preflight"
    directory.mkdir()
    ab = out / "bin/original"
    commands = {
        "lscpu": ["lscpu"],
        "sample": [str(ab), "sample", "--count", "3", "--interval", "1"],
        "static": [
            sys.executable,
            str(ROOT / "bench/share_nothing.py"),
            "static",
            "--out",
            str(directory / "static.json"),
        ],
    }
    for name, command in commands.items():
        filename = {"lscpu": "lscpu.txt", "sample": "sample.jsonl", "static": "static.log"}[name]
        with (
            (directory / filename).open("x") as stdout,
            (directory / (name + ".err")).open("x") as stderr,
        ):
            try:
                result = subprocess.run(
                    command, stdout=stdout, stderr=stderr, timeout=30, check=False
                )
                code = result.returncode
            except subprocess.TimeoutExpired:
                code = 124
        with (directory / "exits.txt").open("a") as log:
            label = {"lscpu": "lscpu", "sample": "ab sample", "static": "static facts"}[name]
            log.write(f"{label}: exit {code}\n")
    problems = helper.preflight_problems(str(directory))
    save(directory / "problems.json", problems)
    require(not problems, f"workstation preflight refused: {problems}")
    allowed = sorted(os.sched_getaffinity(0))
    topology = {}
    for cpu in allowed:
        path = Path(f"/sys/devices/system/cpu/cpu{cpu}/topology")
        topology[str(cpu)] = dict(
            socket=int((path / "physical_package_id").read_text()),
            core=int((path / "core_id").read_text()),
            siblings=helper.cpu_list((path / "thread_siblings_list").read_text().strip()),
        )
    binding = cpu_binding(allowed, topology)
    save(directory / "binding.json", binding | dict(topology=topology, host=platform.node()))
    with (directory / "facts.json").open("x") as stream:
        subprocess.run([str(ab), "env", "--facts"], stdout=stream, check=True, timeout=30)
    return binding


def arms(out):
    return [
        f"mibz2@{out}/bin/original",
        f"mibz2-cells@{out}/bin/original",
        f"mibz2-cells@{out}/bin/selective",
    ]


def schedule(out):
    order = arms(out)
    rows = []
    for rnd in range(ROUNDS):
        indices = list(range(3))
        indices = indices[rnd % 3 :] + indices[: rnd % 3]
        if (rnd // 3) % 2:
            indices.reverse()
        for corpus in PINS:
            rows.extend((rnd, corpus, order[i]) for i in indices)
    return rows


def validate(rows, out, cpu):
    require(
        [(r.get("round"), r.get("corpus"), r.get("arm")) for r in rows] == schedule(out),
        "missing/duplicate/reordered/unregistered measured row",
    )
    hashes = {}
    for row in rows:
        name = row["corpus"]
        require(
            row.get("termination") == "success" and not row.get("err") and not row.get("broken"),
            f"failed child: {row}",
        )
        require(
            row.get("threads") == 1 and row.get("cpus") == str(cpu), "wrong thread count or pin"
        )
        require(
            row.get("out") == PINS[name][1] and bool(row.get("hash")), "output length/hash missing"
        )
        hashes.setdefault(name, row["hash"])
        require(hashes[name] == row["hash"], "measured output mismatch")
        require(
            row.get("sampler_s") == 0
            and row.get("sampler_cpus") == "none"
            and row.get("sha256") is None,
            "timed instrumentation was active",
        )
        for phase in ("machine_pre", "machine_post"):
            state = row.get(phase, {})
            require(
                state.get("governor") == "powersave"
                and state.get("epp") == "balance_performance"
                and state.get("turbo") is True,
                f"machine changed: {phase}",
            )
        for key in ("wall", "cpu", "rss_kb"):
            value = row.get(key)
            require(
                isinstance(value, (float, int)) and math.isfinite(value) and value > 0,
                f"invalid {key}",
            )
    return rows


def summarise(rows, out):
    estimator = module("cells_i9_report", ROOT / "bench/report.py")
    result = {}
    names = ("Segmented", "original", "selective")
    arm_names = arms(out)
    for corpus in PINS:
        values = [
            [r for r in rows if r["corpus"] == corpus and r["arm"] == arm] for arm in arm_names
        ]
        absolute = {
            name: {
                key: statistics.median(r[key] for r in group) for key in ("wall", "cpu", "rss_kb")
            }
            for name, group in zip(names, values, strict=True)
        }
        for name, group in zip(names, values, strict=True):
            absolute[name]["max_rss_kb"] = max(r["rss_kb"] for r in group)
        ratios = {
            f"{names[a]}/{names[b]}": {
                key: estimator.ratio_ci(
                    [x[key] / y[key] for x, y in zip(values[a], values[b], strict=True)]
                )
                for key in ("wall", "cpu", "rss_kb")
            }
            for a, b in ((1, 0), (2, 1), (2, 0))
        }
        result[corpus] = dict(absolute=absolute, ratios=ratios)
    save(out / "summary.json", result)
    lines = [
        "# Matched i9 complete readers",
        "",
        "Ratios: geometric mean [95% paired log-t interval].",
        "",
        "| Corpus | B/A wall | C/B wall | C/A wall | C/B CPU | C/B RSS |",
        "|---|---|---|---|---|---|",
    ]
    for corpus, data in result.items():
        fields = [
            data["ratios"][ratio][key]
            for ratio, key in (
                ("original/Segmented", "wall"),
                ("selective/original", "wall"),
                ("selective/Segmented", "wall"),
                ("selective/original", "cpu"),
                ("selective/original", "rss_kb"),
            )
        ]
        lines.append(
            "| "
            + corpus
            + " | "
            + " | ".join(f"{v[0]:.4f} [{v[1]:.4f}, {v[2]:.4f}]" for v in fields)
            + " |"
        )
    lines += [
        "",
        "Absolute wall/CPU seconds, median and maximum native RSS KiB: summary.json.",
        "This is diagnostic evidence, not default-path or integration approval.",
    ]
    (out / "summary.md").write_text("\n".join(lines) + "\n")


def measure(out):
    out = out.resolve()
    prepared = json.loads((out / "prepared.json").read_text())
    require(
        prepared["common"] == COMMON and prepared["original"] == ORIGINAL, "wrong prepared sources"
    )
    check_inputs(out / "inputs")
    for arm, wanted in prepared["binary_sha256"].items():
        require(digest(out / "bin" / arm) == wanted, "binary changed after preparation")
    require(not (out / "runs.jsonl").exists(), "refuse to repeat an attempted measurement")
    binding = preflight(out)
    save(out / "schedule.json", schedule(out))
    command = [
        str(out / "bin/original"),
        "run",
        str(out / "runs.jsonl"),
        "--arms",
        ",".join(arms(out)),
        "--threads",
        "1",
        "--pin",
        str(binding["cpu"]),
        "--rounds",
        str(ROUNDS),
        "--settle",
        "2",
        "--max-busy",
        "1.5",
        "--sysfs-state",
        *[str(out / "inputs" / (name + ".bz2")) for name in PINS],
    ]
    env = {k: v for k, v in os.environ.items() if not k.startswith("AB_")}
    env["AB_SAMPLE_MS"] = "0"
    # Parent/census stay off the entire measured physical core. taskset in the
    # existing harness pins each child back to the verified decoder CPU.
    os.sched_setaffinity(0, binding["parent"])
    call(command, out / "measurement.log", env=env, timeout=7100)
    rows = [json.loads(line) for line in (out / "runs.jsonl").read_text().splitlines()]
    validate(rows, out, binding["cpu"])
    summarise(rows, out)
    (out / "outcome.txt").write_text(
        "COMPLETE: 252 matched decodes; Coordinator must reconcile all gates\n"
    )


if __name__ == "__main__":
    require(
        len(sys.argv) == 3 and sys.argv[1] in {"prepare", "measure"},
        "usage: workstation.py prepare|measure ATTEMPT",
    )
    {"prepare": prepare, "measure": measure}[sys.argv[1]](Path(sys.argv[2]))
