"""bench/share_nothing.py: the schedule, the preflight, the drive, the corpora and the judge
(docs/journal/2026-11-10-share-nothing-pricing.md), on synthetic records. Fixed before data."""

import json
import math
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE))
import share_nothing as sn  # noqa: E402

# ---- the schedule ------------------------------------------------------------------------


def test_schedule_is_registered():
    assert sn.digest(sn.schedule()) == sn.SCHEDULE_SHA256


def test_schedule_shape():
    s = sn.schedule()
    assert len(s) == (sn.WARMUP_ROUNDS + sn.ROUNDS) * 12
    cells = {(c, w) for c in sn.CORPORA for w in sn.WORKERS}
    orders = set()
    for r in range(sn.WARMUP_ROUNDS + sn.ROUNDS):
        slots = [x for x in s if x["round"] == r]
        assert [x["pos"] for x in slots] == list(range(12))
        for x in (slots[0], slots[-1]):
            assert (x["role"], x["arm"], x["threads"], x["corpus"]) == (
                "sentinel",
                "mibz2",
                10,
                "wikidata",
            )
        mid = slots[1:-1]
        assert all(x["role"] == "cell" and x["arm"] == "mibz2-workers" for x in mid)
        assert {(x["corpus"], x["threads"]) for x in mid} == cells
        orders.add(tuple((x["corpus"], x["threads"]) for x in mid))
    # randomised: the rounds' orders are not one order
    assert len(orders) > (sn.WARMUP_ROUNDS + sn.ROUNDS) // 2


def test_schedule_is_a_function_of_the_seed():
    assert sn._schedule(sn.SCHEDULE_SEED, 5) == sn._schedule(sn.SCHEDULE_SEED, 5)
    assert sn._schedule(sn.SCHEDULE_SEED, 5) != sn._schedule(sn.SCHEDULE_SEED + 1, 5)


def test_dry_schedule_is_the_full_ones_prefix():
    d = sn.schedule(dry=True)
    assert d == sn.schedule()[: len(d)]
    assert {x["round"] for x in d} == set(range(sn.WARMUP_ROUNDS + sn.DRY_ROUNDS))


# ---- the statistics ----------------------------------------------------------------------


def test_boot_indices_are_circular_blocks():
    idx = sn.boot_indices(10, 3, 50, 1)
    assert len(idx) == 50 and all(len(i) == 10 for i in idx)
    for i in idx:
        for start in (0, 3, 6):
            a, b, c = i[start : start + 3]
            assert b == (a + 1) % 10 and c == (a + 2) % 10


def test_geo_constant_and_known():
    g = sn.geo([math.log(1.3)] * 20)
    assert g["ratio"] == pytest.approx(1.3)
    assert g["lo"] == pytest.approx(1.3) and g["hi"] == pytest.approx(1.3)
    g = sn.geo([math.log(2), math.log(8)], block=1)
    assert g["ratio"] == pytest.approx(4.0)
    assert 2 - 1e-9 <= g["lo"] <= g["hi"] <= 8 + 1e-9
    assert sn.geo([])["ratio"] is None
    # a block as long as the rounds gives no interval (it would only rotate them)
    g = sn.geo([0.1, 0.3], block=5)
    assert g["lo"] is None and "no_interval" in g and g["t_lo"] < g["ratio"] < g["t_hi"]
    assert sn.geo([0.1])["lo"] is None


def test_geo_interval_widens_with_dependence():
    # persistent blocks of high and low values: blocks of 10 give a wider interval than 1
    logs = ([0.6] * 10 + [0.1] * 10) * 3
    a, b = sn.geo(logs, block=1), sn.geo(logs, block=10)
    assert (b["hi"] - b["lo"]) > (a["hi"] - a["lo"])
    assert a["lag1_autocorr"] > 0.5


@pytest.mark.parametrize(
    "lo,hi,want",
    [
        (1.15, 1.40, "within the band"),
        (1.10, 1.44, "within the band"),
        (0.95, 1.05, "below the band"),
        (1.50, 1.70, "above the band"),
        (1.00, 1.60, "wider than the band"),
        (1.05, 1.30, "overlaps the band's lower edge"),
        (1.30, 1.60, "overlaps the band's upper edge"),
        (None, None, "no interval"),
    ],
)
def test_band_class(lo, hi, want):
    assert sn.band_class({"lo": lo, "hi": hi}) == want


# ---- the preflight -----------------------------------------------------------------------


def test_collectors():
    ok = "lscpu: exit 0\nab sample: exit 0\nstatic facts: exit 0\n"
    assert sn.collector_problems(ok) == []
    assert sn.collector_problems(None)
    assert any(
        "timed out" in b
        for b in sn.collector_problems(ok.replace("ab sample: exit 0", "ab sample: exit 124"))
    )
    assert any("twice" in b for b in sn.collector_problems(ok + "lscpu: exit 0\n"))
    assert any("unregistered" in b for b in sn.collector_problems(ok + "perf: exit 0\n"))
    assert any("no exit status" in b for b in sn.collector_problems("lscpu: exit 0\n"))


def i9_static():
    pol = {
        f"policy{c}": {
            "affected_cpus": str(c),
            "scaling_driver": "intel_pstate",
            "scaling_governor": "powersave",
            "energy_performance_preference": "balance_performance",
        }
        for c in range(20)
    }
    return {
        "schema": sn.SCHEMA["static"],
        "model": "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz",
        "online": "0-19",
        "cpus_allowed": "0-19",
        "no_turbo": "0",
        "policies": pol,
        "quota": {"state": "unlimited"},
    }


LSCPU = "Architecture: x86_64\nModel name:            Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"


def samples(**over):
    s = {"governor": "powersave", "epp": "balance_performance", "turbo": True, **over}
    return [{"sample": i, "state": s} for i in range(3)]


def test_host_passes_on_the_registered_i9():
    assert sn.host_problems(LSCPU, i9_static(), samples()) == []


@pytest.mark.parametrize(
    "mutate,needle",
    [
        (lambda s: s.update(model="Intel Xeon"), "cpuinfo's model"),
        (lambda s: s.update(online="0-9"), "online"),
        (lambda s: s.update(cpus_allowed="0-8"), "allowed"),
        (lambda s: s.update(quota={"state": "limited"}), "quota"),
        (lambda s: s.update(quota={"state": "unknown"}), "quota"),
        (lambda s: s.update(no_turbo="1"), "no_turbo"),
        (
            lambda s: s["policies"]["policy3"].update(scaling_governor="performance"),
            "scaling_governor",
        ),
        (
            lambda s: s["policies"]["policy3"].update(energy_performance_preference="performance"),
            "energy_performance",
        ),
        (lambda s: s["policies"].pop("policy19"), "cover"),
        (lambda s: s.update(schema="other"), "registered collector"),
    ],
)
def test_host_refuses(mutate, needle):
    s = i9_static()
    mutate(s)
    bad = sn.host_problems(LSCPU, s, samples())
    assert any(needle in b for b in bad), bad


def test_host_refuses_on_lscpu_and_samples():
    assert any(
        "lscpu" in b for b in sn.host_problems("Model name: AMD EPYC\n", i9_static(), samples())
    )
    assert any("turbo" in b for b in sn.host_problems(LSCPU, i9_static(), samples(turbo="true")))
    assert any(
        "governor" in b for b in sn.host_problems(LSCPU, i9_static(), samples(governor=None))
    )
    assert any("samples" in b for b in sn.host_problems(LSCPU, i9_static(), samples()[:2]))


def test_preflight_reads_its_files(tmp_path):
    d = tmp_path / "preflight"
    d.mkdir()
    (d / "exits.txt").write_text("lscpu: exit 0\nab sample: exit 0\nstatic facts: exit 0\n")
    (d / "lscpu.txt").write_text(LSCPU)
    (d / "sample.jsonl").write_text("".join(json.dumps(x) + "\n" for x in samples()))
    (d / "static.json").write_text(json.dumps(i9_static()))
    assert sn.main(["preflight", str(tmp_path)]) == 0
    assert json.loads((tmp_path / "preflight.json").read_text())["verdict"] == "PASS"
    (d / "exits.txt").write_text("lscpu: exit 0\nab sample: exit 1\nstatic facts: exit 0\n")
    assert sn.main(["preflight", str(tmp_path)]) == 1
    (d / "static.json").write_text("{")
    assert sn.main(["preflight", str(tmp_path)]) == 1


def cgroup_root(tmp_path, path="/user.slice/a.scope", mount="/sys/fs/cgroup", mroot="/"):
    (tmp_path / "proc/self").mkdir(parents=True)
    (tmp_path / "proc/self/cgroup").write_text(f"0::{path}\n")
    (tmp_path / "proc/self/mountinfo").write_text(
        f"25 1 0:22 / /proc rw - proc proc rw\n"
        f"30 25 0:26 {mroot} {mount} rw,nosuid - cgroup2 cgroup2 rw\n"
    )
    base = tmp_path / mount.lstrip("/")
    base.mkdir(parents=True)
    (base / "cgroup.controllers").write_text("cpuset cpu io memory pids\n")
    (base / "cgroup.subtree_control").write_text("cpu memory pids\n")
    return base


def test_cgroup_quota_unlimited_and_limited(tmp_path):
    base = cgroup_root(tmp_path)
    (base / "user.slice/a.scope").mkdir(parents=True)
    (base / "user.slice/cpu.max").write_text("max 100000\n")
    (base / "user.slice/cgroup.subtree_control").write_text("memory\n")
    q = sn.cgroup_quota(str(tmp_path))
    assert q["state"] == "unlimited", q
    assert [lv.get("why") for lv in q["levels"]] == [
        "the hierarchy's root",
        None,
        "the cpu controller is not delegated",
    ]
    (base / "user.slice/cpu.max").write_text("200000 100000\n")
    q = sn.cgroup_quota(str(tmp_path))
    assert q["state"] == "limited" and q["limits"] == ["/sys/fs/cgroup/user.slice/cpu.max"]


def test_cgroup_quota_missing_path_is_unknown(tmp_path):
    # the Researcher's counterexample: a path with no hierarchy behind it
    (tmp_path / "proc/self").mkdir(parents=True)
    (tmp_path / "proc/self/cgroup").write_text("0::/missing.scope\n")
    q = sn.cgroup_quota(str(tmp_path))
    assert q["state"] == "unknown", q


def test_cgroup_quota_missing_directory_is_unknown(tmp_path):
    cgroup_root(tmp_path, path="/missing.scope")
    q = sn.cgroup_quota(str(tmp_path))
    assert q["state"] == "unknown" and "does not exist" in q["why"]


def test_cgroup_quota_no_mount_is_unknown(tmp_path):
    cgroup_root(tmp_path)
    (tmp_path / "proc/self/mountinfo").write_text("25 1 0:22 / /proc rw - proc proc rw\n")
    assert sn.cgroup_quota(str(tmp_path))["state"] == "unknown"


def test_cgroup_quota_delegated_without_cpu_max_is_unknown(tmp_path):
    base = cgroup_root(tmp_path, path="/a")
    (base / "a").mkdir()
    q = sn.cgroup_quota(str(tmp_path))
    assert q["state"] == "unknown" and "delegated" in q["why"]


def test_cgroup_quota_unreadable_cpu_max_is_unknown(tmp_path):
    base = cgroup_root(tmp_path, path="/a")
    (base / "a").mkdir()
    (base / "a/cpu.max").mkdir()  # (a directory: reading it fails)
    assert sn.cgroup_quota(str(tmp_path))["state"] == "unknown"


def test_cgroup_quota_v1_cpu_is_unknown(tmp_path):
    cgroup_root(tmp_path, path="/")
    (tmp_path / "proc/self/cgroup").write_text("1:cpu,cpuacct:/\n0::/\n")
    assert "v1" in sn.cgroup_quota(str(tmp_path))["why"]


def test_cgroup_quota_namespaced_mount_root(tmp_path):
    base = cgroup_root(tmp_path, path="/ns/a", mroot="/ns")
    (base / "a").mkdir()
    (base / "cpu.max").write_text("max 100000\n")
    (base / "a/cpu.max").write_text("max 100000\n")
    assert sn.cgroup_quota(str(tmp_path))["state"] == "unlimited"
    (base / "cpu.max").unlink()
    assert sn.cgroup_quota(str(tmp_path))["state"] == "unknown"


def test_count_blocks(tmp_path):
    raw = tmp_path / "x"
    rng = __import__("random").Random(1)
    raw.write_bytes(bytes(rng.randrange(97, 123) for _ in range(250_000)))
    subprocess.run(["bzip2", "-1", "-k", str(raw)], check=True)
    assert sn.count_blocks((tmp_path / "x.bz2").read_bytes()) == 3


# ---- the drive ---------------------------------------------------------------------------

FAKE_AB = """#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
assert a[0] == "run"
out, f = a[1], a[-1]
arm = a[a.index("--arms") + 1]; t = int(a[a.index("--threads") + 1])
assert "--sysfs-state" in a and a[a.index("--rounds") + 1] == "1"
fail = os.environ.get("FAKE_FAIL_AT")
n = sum(1 for _ in open(out)) if os.path.exists(out) else 0
if fail is not None and n == int(fail):
    sys.exit(3)
rec = {"round": 0, "corpus": os.path.basename(f)[:-4], "threads": t, "arm": arm,
       "machine_pre": {"freq_mhz": 4500.0}, "t_spawn_unix_ns": 1,
       "wall": 0.2, "cpu": 1.0, "out": 209700196, "hash": "a8830caee5c511a6", "err": ""}
open(out, "a").write(json.dumps(rec) + "\\n")
"""


def fake_ab(tmp_path):
    p = tmp_path / "ab"
    p.write_text(FAKE_AB)
    p.chmod(0o755)
    return str(p)


def drive(d, ab, *extra, env=None):
    return subprocess.run(
        [sys.executable, str(HERE / "share_nothing.py"), "drive", str(d), ab, *extra],
        capture_output=True,
        text=True,
        timeout=300,
        env={**os.environ, **(env or {})},
    )


def test_drive_runs_every_slot_once(tmp_path):
    d = tmp_path / "w"
    d.mkdir()
    r = drive(d, fake_ab(tmp_path), "--dry")
    assert r.returncode == 0, r.stdout + r.stderr
    slots = [json.loads(x) for x in (d / "slots.jsonl").read_text().splitlines()]
    runs = [json.loads(x) for x in (d / "runs.jsonl").read_text().splitlines()]
    want = sn.schedule(dry=True)
    assert len(slots) == len(runs) == len(want)
    for i, (s, w, run) in enumerate(zip(slots, want, runs)):
        assert s["rc"] == 0 and s["line"] == i and s["lines_after"] == i + 1
        assert (run["arm"], run["threads"], run["corpus"]) == (w["arm"], w["threads"], w["corpus"])


def test_drive_stops_at_the_first_failure(tmp_path):
    d = tmp_path / "w"
    d.mkdir()
    r = drive(d, fake_ab(tmp_path), "--dry", env={"FAKE_FAIL_AT": "5"})
    assert r.returncode == 1
    slots = [json.loads(x) for x in (d / "slots.jsonl").read_text().splitlines()]
    assert len(slots) == 6 and slots[-1]["rc"] == 3


def test_drive_bounds_a_stalled_slot(tmp_path, monkeypatch):
    # an ab whose busy gate never opens (here: a sleep) is stopped at the slot's deadline,
    # recorded with 124, and nothing after it runs
    p = tmp_path / "ab"
    p.write_text("#!/bin/sh\nexec sleep 600\n")
    p.chmod(0o755)
    d = tmp_path / "w"
    d.mkdir()
    monkeypatch.setattr(sn, "CELL_DEADLINE", (2, 1))
    rc = sn.main(["drive", str(d), str(p), "--dry"])
    assert rc == 124
    slots = [json.loads(x) for x in (d / "slots.jsonl").read_text().splitlines()]
    assert len(slots) == 1 and slots[0]["rc"] == 124 and slots[0]["lines_after"] == 0


def test_drive_refuses_a_relaxed_gate_outside_dry(tmp_path):
    d = tmp_path / "w"
    d.mkdir()
    assert drive(d, fake_ab(tmp_path), "--max-busy", "4").returncode == 2
    assert not (d / "slots.jsonl").exists()


# ---- the judge ---------------------------------------------------------------------------


def cpu_model(corpus, w, slow):
    """A synthetic decode's CPU: -9 grows x3.5 from 4 to 10, -2 x1.25; a slow round doubles
    the per-thread CPU at 10 workers"""
    base = {"wikidata-l2": 0.70, "wikidata-l9": 0.80}[corpus]
    g = {"wikidata-l2": 1.25, "wikidata-l9": 3.5}[corpus]
    c = base * g ** ((w - 4) / 6)
    return c * (2.0 if slow and w == 10 else 1.0)


HEAD = "1" * 40
FROZEN = "2" * 40
AB = "a" * 64
L2, L9 = "c" * 64, "d" * 64


def state(t, **over):
    """A sysfs state as ab's --sysfs-state records it on the i9"""
    s = {
        "freq_mhz": 4600.0,
        "governor": "powersave",
        "epp": "balance_performance",
        "turbo": True,
        "pkg_temp_c": 50.0,
        "t_unix_ns": t,
    }
    s.update(over)
    return s


def write_preflight(d, passing=True):
    pdir = d / "preflight"
    pdir.mkdir()
    (pdir / "exits.txt").write_text("lscpu: exit 0\nab sample: exit 0\nstatic facts: exit 0\n")
    (pdir / "lscpu.txt").write_text(LSCPU if passing else "Model name: Intel Xeon\n")
    (pdir / "sample.jsonl").write_text("".join(json.dumps(x) + "\n" for x in samples()))
    (pdir / "static.json").write_text(json.dumps(i9_static()))


def make_attempt(tmp_path, dry=False, slow_rounds=(), mutate=None, preflight=True):
    d = tmp_path / ("dry" if dry else "full")
    d.mkdir()
    (d / "corpora").mkdir()
    write_preflight(d, preflight)
    sched = sn.schedule(dry)
    slots, runs = [], []
    t = 1_000.0
    for i, s in enumerate(sched):
        slow = s["round"] in slow_rounds
        if s["role"] == "sentinel":
            cpu, wall = 1.0, (0.30 if slow else 0.15)
        else:
            cpu = cpu_model(s["corpus"], s["threads"], slow)
            wall = cpu / s["threads"]
        slots.append(
            {
                "schema": sn.SCHEMA["slot"],
                "index": i,
                **s,
                "rc": 0,
                "line": i,
                "lines_after": i + 1,
                "t_start_unix": t,
                "t_end_unix": t + 0.7,
            }
        )
        ns = int((t + 0.3) * 1e9)
        runs.append(
            {
                "round": 0,
                "corpus": s["corpus"],
                "threads": s["threads"],
                "arm": s["arm"],
                "waited_s": 0,
                "machine_pre": state(ns - 1000),
                "t_spawn_unix_ns": ns,
                "t_exit_unix_ns": ns + int(wall * 1e9),
                "machine_post": state(ns + int(wall * 1e9) + 1000),
                "wall": wall,
                "cpu": cpu,
                "out": sn.CONTENT_BYTES,
                "hash": sn.WORKLOAD_HASH,
                "err": "",
            }
        )
        t += 0.8
    files = {
        n: {"sha256": h, "decoded_sha256": sn.CONTENT_SHA256, "decoded_bytes": sn.CONTENT_BYTES}
        for n, h in (("wikidata-l2", L2), ("wikidata-l9", L9))
    }
    corpora = {
        "schema": sn.SCHEMA["corpora"],
        "ok": True,
        "problems": [],
        "files": files,
        "content": {"sha256": sn.CONTENT_SHA256, "bytes": sn.CONTENT_BYTES},
        "source": {"sha256": sn.CORPUS_SHA256},
    }
    ident = {
        "schema": sn.SCHEMA["identity"],
        "dry": dry,
        "head": HEAD,
        "frozen": None if dry else FROZEN,
        "inputs_match_frozen": not dry,
        "preflight_exit": 0 if preflight else 1,
        "drive_exit": 0,
        "ab_sha256": AB,
        "ab_sha256_after": AB,
        "tree_clean_after": True,
        "schedule_sha256": sn.digest(sched),
        "sources": {x: {"head": "e" * 64, "frozen": None if dry else "e" * 64} for x in sn.SOURCES},
        "corpora_after": {"wikidata-l2": L2, "wikidata-l9": L9, "wikidata": sn.CORPUS_SHA256},
    }
    parts = {"slots": slots, "runs": runs, "corpora": corpora, "identity": ident}
    if mutate:
        mutate(parts)
    (d / "slots.jsonl").write_text("".join(json.dumps(x) + "\n" for x in parts["slots"]))
    (d / "runs.jsonl").write_text("".join(json.dumps(x) + "\n" for x in parts["runs"]))
    (d / "corpora.json").write_text(json.dumps(parts["corpora"]))
    (d / "identity.json").write_text(json.dumps(parts["identity"]))
    return d


def judge(d, dry=False):
    rc = sn.main(["judge", str(d)] + (["--dry"] if dry else []))
    return rc, json.loads((d / "judgement.json").read_text())


def test_judge_valid_and_its_figures(tmp_path):
    rc, j = judge(make_attempt(tmp_path))
    assert rc == 0 and j["outcome"].startswith("VALID: "), j["outcome"]
    p = j["result"]["primary"]["cpu"]
    assert p["growth"]["wikidata-l2"]["ratio"] == pytest.approx(1.25)
    assert p["growth"]["wikidata-l9"]["ratio"] == pytest.approx(3.5)
    assert p["ratio_of_ratios"]["ratio"] == pytest.approx(1.25 / 3.5)
    m = j["result"]["model_check"]
    assert m["l2_interval_vs_band"] == "within the band" and m["l2_point_in_band"]
    assert m["l9_against_s2"].startswith("compatible") and "not equivalence" in m["l9_against_s2"]
    assert m["ratio_of_ratios"].startswith("below 1: a favourable level x worker interaction")
    assert m["model_gb_s"]["point"] == pytest.approx(3.76 / 1.25)
    assert m["model_gb_s"]["label"].startswith("HISTORICAL MODEL")
    assert "VALID" in j["outcome"] and "ratio of ratios" in j["outcome"]
    r = j["result"]
    assert len(r["chronological"]) == sn.ROUNDS
    assert r["pre_state"]["unread"]["freq_mhz"] == 0
    assert r["waits"]["by_cell"]["wikidata-l2@10"]["runs"] == sn.ROUNDS
    assert j["result"]["rounds"] == sn.ROUNDS
    assert j["result"]["modes"]["counts"]["fast"] == sn.ROUNDS
    # (the wall: here cpu / workers, so its growth is the CPU's x 4/10)
    assert j["result"]["primary"]["wall"]["growth"]["wikidata-l2"]["ratio"] == pytest.approx(
        1.25 * 0.4
    )


def test_judge_keeps_every_round_and_splits_modes_descriptively(tmp_path):
    slow = set(range(sn.WARMUP_ROUNDS, sn.WARMUP_ROUNDS + 20))
    rc, j = judge(make_attempt(tmp_path, slow_rounds=slow))
    assert rc == 0
    r = j["result"]
    assert r["rounds"] == sn.ROUNDS  # nothing excluded
    assert r["modes"]["counts"]["slow"] == 20 and r["modes"]["counts"]["fast"] == sn.ROUNDS - 20
    assert r["modes"]["label"].startswith("DESCRIPTIVE")
    fast = r["modes"]["classes"]["fast"]["cpu"]["growth"]["wikidata-l2"]
    slowg = r["modes"]["classes"]["slow"]["cpu"]["growth"]["wikidata-l2"]
    assert fast["ratio"] == pytest.approx(1.25) and slowg["ratio"] == pytest.approx(2.5)
    # point estimates only: no class interval (a selection's blocks would compress time)
    assert "lo" not in fast and "lo" not in slowg
    assert [c["class"] for c in r["chronological"]][:20] == ["slow"] * 20
    full = r["primary"]["cpu"]["growth"]["wikidata-l2"]
    assert 1.25 < full["ratio"] < 2.5 and full["hi"] > full["lo"]


def test_judge_warmup_is_not_analysed(tmp_path):
    rc, j = judge(make_attempt(tmp_path, slow_rounds={0}))
    assert rc == 0
    assert j["result"]["primary"]["cpu"]["growth"]["wikidata-l2"]["ratio"] == pytest.approx(1.25)


def test_judge_mixed_round(tmp_path):
    def m(p):
        for s, r in zip(p["slots"], p["runs"]):
            if s["round"] == 3 and s["pos"] == 11:
                r["wall"] = 0.30

    rc, j = judge(make_attempt(tmp_path, mutate=m))
    assert rc == 0 and j["result"]["modes"]["counts"]["mixed"] == 1


def test_judge_dry(tmp_path):
    rc, j = judge(make_attempt(tmp_path, dry=True), dry=True)
    assert rc == 0 and j["outcome"].startswith("DRY (not evidence): "), j["outcome"]
    assert j["result"]["rounds"] == sn.DRY_ROUNDS


def _drop_slot(p):
    p["slots"].pop()
    p["runs"].pop()


def _bad_hash(p):
    p["runs"][7]["hash"] = "0000"


def _bad_bytes(p):
    p["runs"][7]["out"] = 1


def _err(p):
    p["runs"][7]["err"] = "CRC mismatch"


def _swap(p):
    p["runs"][1], p["runs"][2] = p["runs"][2], p["runs"][1]


def _rc(p):
    p["slots"][4]["rc"] = 124


def _two_lines(p):
    p["slots"][4]["lines_after"] = 6


def _no_state(p):
    del p["runs"][9]["machine_pre"]


def _nan(p):
    p["runs"][9]["cpu"] = float("nan")


def _unfrozen(p):
    p["identity"]["inputs_match_frozen"] = False


def _preflight(p):
    p["identity"]["preflight_exit"] = 1


def _drive(p):
    p["identity"]["drive_exit"] = 124


def _ab_changed(p):
    p["identity"]["ab_sha256_after"] = "b"


def _corpus_changed(p):
    p["identity"]["corpora_after"]["wikidata-l2"] = "other"


def _corpora_bad(p):
    p["corpora"]["ok"] = False


def _dirty(p):
    p["identity"]["tree_clean_after"] = False


def _dry_identity(p):
    p["identity"]["dry"] = True


# the Researcher's counterexamples (#145 6758300190), each now refused
def _l2_digest_gone_both_sides(p):
    p["corpora"]["files"]["wikidata-l2"].pop("sha256")
    p["identity"]["corpora_after"].pop("wikidata-l2")


def _empty_telemetry(p):
    for r in p["runs"]:
        r["machine_pre"] = {}
        r["t_spawn_unix_ns"] = None


def _no_head(p):
    p["identity"].pop("head")


def _no_frozen(p):
    p["identity"]["frozen"] = None


def _no_sources(p):
    p["identity"].pop("sources")


def _source_not_frozen(p):
    p["identity"]["sources"]["bench/share_nothing.py"]["frozen"] = "f" * 64


def _null_without_reason(p):
    p["runs"][5]["machine_pre"]["governor"] = None


def _no_post(p):
    del p["runs"][5]["machine_post"]


def _spawn_after_exit(p):
    r = p["runs"][5]
    r["t_spawn_unix_ns"] = r["t_exit_unix_ns"] + 1


def _runs_out_of_order(p):
    p["runs"][6]["t_spawn_unix_ns"] = p["runs"][4]["t_spawn_unix_ns"]


def _slot_time_missing(p):
    p["slots"][3].pop("t_end_unix")


def _no_wait(p):
    del p["runs"][3]["waited_s"]


def _ab_digest_gone(p):
    p["identity"]["ab_sha256"] = p["identity"]["ab_sha256_after"] = None


def _content_not_shown(p):
    p["corpora"].pop("content")


def _decode_not_shown(p):
    p["corpora"]["files"]["wikidata-l9"]["decoded_sha256"] = None


@pytest.mark.parametrize(
    "mutate",
    [
        _drop_slot,
        _bad_hash,
        _bad_bytes,
        _err,
        _swap,
        _rc,
        _two_lines,
        _no_state,
        _nan,
        _unfrozen,
        _preflight,
        _drive,
        _ab_changed,
        _corpus_changed,
        _corpora_bad,
        _dirty,
        _dry_identity,
        _l2_digest_gone_both_sides,
        _empty_telemetry,
        _no_head,
        _no_frozen,
        _no_sources,
        _source_not_frozen,
        _null_without_reason,
        _no_post,
        _spawn_after_exit,
        _runs_out_of_order,
        _slot_time_missing,
        _no_wait,
        _ab_digest_gone,
        _content_not_shown,
        _decode_not_shown,
    ],
)
def test_judge_invalid(tmp_path, mutate):
    rc, j = judge(make_attempt(tmp_path, mutate=mutate))
    assert rc == 2 and j["outcome"].startswith("INVALID: "), j["outcome"]
    assert j["result"] is None


def test_judge_dry_admits_an_unfrozen_failed_preflight(tmp_path):
    rc, j = judge(make_attempt(tmp_path, dry=True, preflight=False), dry=True)
    assert rc == 0 and j["preflight_replay"]["verdict"] == "REFUSED"


def test_judge_replays_the_archived_preflight(tmp_path):
    # a recorded exit 0 is not a pass: the archived files are judged again
    rc, j = judge(
        make_attempt(
            tmp_path, preflight=False, mutate=lambda p: p["identity"].update(preflight_exit=0)
        )
    )
    assert rc == 2 and "replays REFUSED" in j["outcome"]


def test_judge_needs_the_archived_preflight(tmp_path):
    d = make_attempt(tmp_path)
    shutil.rmtree(d / "preflight")
    rc, j = judge(d)
    assert rc == 2 and "no archived preflight" in j["outcome"]


def test_judge_dry_refuses_a_disagreeing_preflight_record(tmp_path):
    rc, j = judge(
        make_attempt(
            tmp_path,
            dry=True,
            preflight=False,
            mutate=lambda p: p["identity"].update(preflight_exit=0),
        ),
        dry=True,
    )
    assert rc == 2


def test_judge_keeps_explicitly_unread_telemetry(tmp_path):
    # unread readings with their reasons are explicit missingness: admitted, counted apart
    def m(p):
        for r in p["runs"]:
            for st_ in (r["machine_pre"], r["machine_post"]):
                for k in ("freq_mhz", "governor", "epp", "turbo", "pkg_temp_c"):
                    st_[k] = None
                    st_[k + "_why"] = "no cpufreq driver"

    rc, j = judge(make_attempt(tmp_path, mutate=m))
    assert rc == 0
    ps = j["result"]["pre_state"]
    assert ps["unread"]["governor"] == ps["runs"]
    assert ps["read_not_as_declared"]["governor"] == 0


def test_judge_counts_read_deviations_without_excluding(tmp_path):
    def m(p):
        p["runs"][20]["machine_pre"]["governor"] = "performance"

    rc, j = judge(make_attempt(tmp_path, mutate=m))
    assert rc == 0 and j["result"]["pre_state"]["read_not_as_declared"]["governor"] == 1
    assert j["result"]["rounds"] == sn.ROUNDS


def test_judge_r_including_1_is_inconclusive(tmp_path):
    # the Researcher's series: R at 0.8 in 30 rounds and 1.2 in 30, its interval spanning 1
    def m(p):
        k = 0
        for s, r in zip(p["slots"], p["runs"]):
            if (
                s["role"] == "cell"
                and s["corpus"] == "wikidata-l2"
                and s["threads"] == 10
                and s["round"] >= 1
            ):
                f = 0.8 if (s["round"] - 1) < 30 else 1.2
                r["cpu"] = cpu_model("wikidata-l2", 4, False) * 3.5 * f
                k += 1
        assert k == sn.ROUNDS

    rc, j = judge(make_attempt(tmp_path, mutate=m))
    assert rc == 0
    rr = j["result"]["primary"]["cpu"]["ratio_of_ratios"]
    assert rr["lo"] < 1 < rr["hi"]
    assert j["result"]["model_check"]["ratio_of_ratios"].startswith("includes 1: INCONCLUSIVE")
    assert "INCONCLUSIVE" in j["outcome"]


def test_judge_r_above_1_closes_nothing(tmp_path):
    def m(p):
        for s, r in zip(p["slots"], p["runs"]):
            if s["role"] == "cell" and s["corpus"] == "wikidata-l2" and s["threads"] == 10:
                r["cpu"] = cpu_model("wikidata-l2", 4, False) * 4.0

    rc, j = judge(make_attempt(tmp_path, mutate=m))
    assert rc == 0 and "closes nothing" in j["result"]["model_check"]["ratio_of_ratios"]


def test_judge_unreadable_records_are_invalid(tmp_path):
    d = make_attempt(tmp_path)
    (d / "runs.jsonl").write_text("{not json\n")
    rc, j = judge(d)
    assert rc == 2 and "does not load" in j["outcome"]
    (d / "identity.json").unlink()
    rc, j = judge(d)
    assert rc == 2


def test_judge_own_failure_is_invalid(tmp_path, monkeypatch):
    def boom(*a):
        raise RuntimeError("synthetic")

    monkeypatch.setattr(sn, "analyse", boom)
    rc, j = judge(make_attempt(tmp_path))
    assert rc == 2 and "the judge failed" in j["outcome"]


# ---- the corpora -------------------------------------------------------------------------


def test_corpora_refuses_another_source(tmp_path):
    src = tmp_path / "x.bz2"
    src.write_bytes(b"BZh9")
    rc = sn.main(["corpora", str(src), str(tmp_path / "out"), "--record", str(tmp_path / "c.json")])
    assert rc == 1 and not json.loads((tmp_path / "c.json").read_text())["ok"]


@pytest.mark.skipif(
    not (shutil.which("lbzip2") and shutil.which("bzip2")), reason="needs lbzip2 and bzip2"
)
def test_corpora_builds_and_checks(tmp_path, monkeypatch):
    content = (b'{"id": "Q%d", "labels": {}}\n' * 3) * 20000
    raw = tmp_path / "content"
    raw.write_bytes(content)
    subprocess.run(["bzip2", "-9", "-k", str(raw)], check=True)
    src = tmp_path / "content.bz2"
    import hashlib

    monkeypatch.setattr(sn, "CORPUS_SHA256", sn.sha256_file(src))
    monkeypatch.setattr(sn, "CONTENT_SHA256", hashlib.sha256(content).hexdigest())
    monkeypatch.setattr(sn, "CONTENT_BYTES", len(content))
    out = tmp_path / "out"
    rc = sn.main(["corpora", str(src), str(out), "--record", str(tmp_path / "c.json")])
    rec = json.loads((tmp_path / "c.json").read_text())
    assert rc == 0 and rec["ok"], rec
    for n in sn.CORPORA:
        f = rec["files"][n]
        assert f["blocks_by_magic"] >= 1
        assert f["decoded_sha256"] == hashlib.sha256(content).hexdigest()
        assert f["thread_invariant"] is True
        assert f["sha256"] == sn.sha256_file(out / f"{n}.bz2")
        assert "-n 4" in f["command"]
    assert not (out / "content").exists()
    assert rec["lbzip2_version"]
