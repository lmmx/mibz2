"""Exhaustive unit, shallow-clone, estimator, and bundle tests for bench/plan_schedule.py."""

from __future__ import annotations

import json
import math
import os
from pathlib import Path
import subprocess
import tarfile
from unittest import mock

import plan_schedule as ps
import pytest

ROOT = Path(__file__).resolve().parents[1]


def _make_plan_schedule_rows(
    *,
    rounds: int = ps.DEFAULT_ROUNDS,
    overrides: dict[tuple[str, str, int], dict[str, float]] | None = None,
    cpr_fn=None,
    wall_jitter_fn=None,
) -> list[dict]:
    """Build synthetic full 22-cell `plan-schedule` rows that PASS all gates by default."""
    overrides = overrides or {}
    # Default base/head walls and RSS matching expected physical i9 behavior.
    defaults: dict[tuple[str, str, int], tuple[float, float, float, float]] = {
        # (base_wall, head_wall, base_rss_kb, head_rss_kb)
        ("wikidata", "pinned", 4): (0.120, 0.119, 40_000, 40_000),
        ("wikidata", "pinned", 5): (0.114, 0.090, 97_500, 40_000),  # 0.789x wall, 0.410x RSS
        ("wikidata", "pinned", 6): (0.081, 0.081, 40_000, 40_000),
        ("wikidata", "pinned", 7): (0.103, 0.070, 129_700, 42_000),  # 0.680x wall < 6T, 0.324x RSS
        ("wikidata", "pinned", 8): (0.068, 0.068, 44_000, 44_000),
        ("wikidata", "pinned", 9): (0.066, 0.066, 51_500, 44_000),  # 1.000x wall, 0.854x RSS
        ("wikidata", "pinned", 10): (0.062, 0.062, 46_000, 46_000),
        ("wikidata", "unpinned", 4): (0.118, 0.118, 42_000, 42_000),
        ("wikidata", "unpinned", 8): (0.066, 0.066, 46_000, 46_000),
        ("wikidata", "unpinned", 10): (0.060, 0.060, 48_000, 48_000),
        ("wikidata", "unpinned", 12): (0.060, 0.060, 48_000, 48_000),
        ("wikidata", "unpinned", 16): (0.061, 0.061, 48_000, 48_000),
        ("wikidata", "unpinned", 20): (0.062, 0.062, 48_000, 48_000),
    }
    for corpus in ps.GUARD_CORPORA:
        for t in ps.PLAN_SCHEDULE_GUARD_COUNTS:
            defaults[(corpus, "pinned", t)] = (0.050, 0.050, 35_000, 35_000)

    out_bytes = {
        "wikidata": 838_800_784,
        "text": 28_000_000,
        "records": 19_200_000,
        "random": 16_000_000,
    }
    rows = []
    cells = ps.expected_cells("plan-schedule")
    sched = ps.build_schedule(cells, rounds=rounds, seed=ps.SEED)
    for step in sched:
        key = (step["corpus"], step["pinning"], step["threads"])
        bw, hw, br, hr = defaults[key]
        ov = overrides.get(key, {})
        bw = ov.get("base_wall", bw)
        hw = ov.get("head_wall", hw)
        br = ov.get("base_rss", br)
        hr = ov.get("head_rss", hr)
        wall = bw if step["arm"] == "base" else hw
        if wall_jitter_fn is not None:
            wall = wall_jitter_fn(step, wall)
        rss = br if step["arm"] == "base" else hr
        cpr = cpr_fn(step) if cpr_fn is not None else 1.28
        rows.append(
            {
                **step,
                "wall": wall,
                "cpu": wall * step["threads"] * 0.8,
                "rss_kb": rss,
                "out": out_bytes[step["corpus"]],
                "hash": f"h-{step['corpus']}",
                "err": "",
                "termination": "success",
                "cycles_per_ref": cpr,
                "runq_s": 0.01,
            }
        )
    return rows


def _make_trim_kstar_rows(
    *,
    rounds: int = ps.DEFAULT_ROUNDS,
    overrides: dict[str, dict[str, float]] | None = None,
    wall_fn=None,
) -> list[dict]:
    overrides = overrides or {}
    defaults = {
        "wikidata": (0.065, 0.061, 46_000, 47_500),  # 0.938x wall, 1.033x RSS
        "text": (0.040, 0.040, 36_000, 36_500),
        "records": (0.050, 0.050, 38_000, 38_500),
        "random": (0.055, 0.055, 38_000, 38_500),
    }
    out_bytes = {
        "wikidata": 838_800_784,
        "text": 28_000_000,
        "records": 19_200_000,
        "random": 16_000_000,
    }
    rows = []
    cells = ps.expected_cells("trim-kstar")
    sched = ps.build_schedule(cells, rounds=rounds, seed=ps.SEED)
    for step in sched:
        c = step["corpus"]
        bw, hw, br, hr = defaults[c]
        ov = overrides.get(c, {})
        bw = ov.get("base_wall", bw)
        hw = ov.get("head_wall", hw)
        br = ov.get("base_rss", br)
        hr = ov.get("head_rss", hr)
        wall = bw if step["arm"] == "base" else hw
        if wall_fn is not None:
            wall = wall_fn(step, wall)
        rss = br if step["arm"] == "base" else hr
        rows.append(
            {
                **step,
                "wall": wall,
                "cpu": wall * 8.0,
                "rss_kb": rss,
                "out": out_bytes[c],
                "hash": f"h-{c}",
                "err": "",
                "termination": "success",
                "cycles_per_ref": 1.25,
                "runq_s": 0.02,
            }
        )
    return rows


# ---------------------------------------------------------------------------
# 1. plan-schedule PASS / STOP edge cases
# ---------------------------------------------------------------------------


def test_plan_schedule_pass_all_22_cells():
    rows = _make_plan_schedule_rows()
    res = ps.score(rows, mode="plan-schedule")
    assert res["verdict"] == "PASS"
    assert res["failed_checks"] == []
    assert len(res["cells"]) == 22
    txt = ps.format_summary(res)
    assert "VERDICT: PASS" in txt
    assert "wikidata:pinned:t7" in txt


@pytest.mark.parametrize(
    ("cell", "override", "expected_failed"),
    [
        (("wikidata", "pinned", 5), {"head_wall": 0.114 * 0.86}, "wikidata_pinned_5t_wall"),
        (("wikidata", "pinned", 5), {"head_rss": 97_500 * 0.56}, "wikidata_pinned_5t_rss"),
        (("wikidata", "pinned", 7), {"head_wall": 0.103 * 0.76}, "wikidata_pinned_7t_wall"),
        (("wikidata", "pinned", 7), {"head_rss": 129_700 * 0.46}, "wikidata_pinned_7t_rss"),
        (("wikidata", "pinned", 9), {"head_wall": 0.066 * 1.03}, "wikidata_pinned_9t_wall"),
        (("wikidata", "pinned", 9), {"head_rss": 51_500 * 0.93}, "wikidata_pinned_9t_rss"),
        (("wikidata", "pinned", 8), {"head_wall": 0.068 * 1.02}, "wikidata_pinned_8t_wall"),
        (("wikidata", "unpinned", 12), {"head_wall": 0.060 * 1.02}, "wikidata_unpinned_12t_wall"),
        (("wikidata", "unpinned", 20), {"head_rss": 48_000 * 1.03}, "wikidata_unpinned_20t_rss"),
        (("text", "pinned", 6), {"head_rss": 35_000 * 1.06}, "guard_text_6t_rss"),
    ],
)
def test_plan_schedule_stop_boundaries(cell, override, expected_failed):
    rows = _make_plan_schedule_rows(overrides={cell: override})
    res = ps.score(rows, mode="plan-schedule")
    assert res["verdict"] == "STOP"
    failed_names = {c["name"] for c in res["failed_checks"]}
    assert expected_failed in failed_names


def test_plan_schedule_7t_monotonicity_against_6t():
    # 7T pinned is <= 0.75x vs base (0.075 / 0.103 = 0.728x) AND RSS <= 0.45x,
    # BUT head 6T is faster (0.074s < 0.075s) -> must STOP on monotonicity!
    rows = _make_plan_schedule_rows(
        overrides={
            ("wikidata", "pinned", 6): {"base_wall": 0.074, "head_wall": 0.074},
            ("wikidata", "pinned", 7): {"base_wall": 0.103, "head_wall": 0.075},
        }
    )
    res = ps.score(rows, mode="plan-schedule")
    assert res["verdict"] == "STOP"
    failed_names = {c["name"] for c in res["failed_checks"]}
    assert "wikidata_pinned_7t_faster_than_6t" in failed_names
    assert "wikidata_pinned_7t_wall" not in failed_names


def test_plan_schedule_guard_disjoint_regression_vs_noisy_overlap():
    # Disjoint +4% slower on records 8T -> STOP.
    rows_disjoint = _make_plan_schedule_rows(
        overrides={("records", "pinned", 8): {"base_wall": 0.050, "head_wall": 0.052}}
    )
    res_bad = ps.score(rows_disjoint, mode="plan-schedule")
    assert res_bad["verdict"] == "STOP"
    assert any(c["name"] == "guard_records_8t_wall" for c in res_bad["failed_checks"])

    # Disjoint +2% slower (<= 3%) -> PASS.
    rows_small = _make_plan_schedule_rows(
        overrides={("records", "pinned", 8): {"base_wall": 0.050, "head_wall": 0.051}}
    )
    assert ps.score(rows_small, mode="plan-schedule")["verdict"] == "PASS"

    # High-variance overlapping cell where mean is +3.5% but 95% CI spans 1.0 -> NOT disjoint -> PASS.
    def noisy_overlap(step, w):
        if (step["corpus"], step["pinning"], step["threads"]) != ("random", "pinned", 8):
            return w
        if step["arm"] == "base":
            return 0.050 * (1.20 if step["round"] % 2 == 0 else 0.85)
        return 0.050 * (0.90 if step["round"] % 2 == 0 else 1.21)

    rows_noisy = _make_plan_schedule_rows(wall_jitter_fn=noisy_overlap)
    res_noisy = ps.score(rows_noisy, mode="plan-schedule")
    cell = res_noisy["cells"]["random:pinned:t8"]
    assert not cell["disjoint_slower"]
    assert res_noisy["verdict"] == "PASS"


# ---------------------------------------------------------------------------
# 2. trim-kstar PASS / STOP edge cases
# ---------------------------------------------------------------------------


def test_trim_kstar_pass_and_stop_boundaries():
    rows = _make_trim_kstar_rows()
    res = ps.score(rows, mode="trim-kstar")
    assert res["verdict"] == "PASS"
    assert len(res["cells"]) == 4

    # Wikidata noisy CI upper > 0.97 even though point estimate is 0.95 -> STOP.
    def noisy_wiki(step, w):
        if step["corpus"] != "wikidata":
            return w
        if step["arm"] == "base":
            return 0.065
        # Half rounds 0.90x, half rounds 1.00x -> geo mean ~0.949, CI upper > 0.97.
        return 0.065 * (0.90 if step["round"] % 2 == 0 else 1.005)

    res_ci = ps.score(_make_trim_kstar_rows(wall_fn=noisy_wiki), mode="trim-kstar")
    assert res_ci["verdict"] == "STOP"
    assert any("wall_ci_hi" in c["name"] for c in res_ci["failed_checks"])

    # Guard wall > 1.01 -> STOP.
    res_guard = ps.score(
        _make_trim_kstar_rows(overrides={"text": {"base_wall": 0.040, "head_wall": 0.0408}}),
        mode="trim-kstar",
    )
    assert res_guard["verdict"] == "STOP"
    assert any("text:pinned:t10_guard_wall" == c["name"] for c in res_guard["failed_checks"])

    # RSS > 1.05x -> STOP.
    res_rss = ps.score(
        _make_trim_kstar_rows(overrides={"wikidata": {"base_rss": 46_000, "head_rss": 49_000}}),
        mode="trim-kstar",
    )
    assert res_rss["verdict"] == "STOP"
    assert any("wikidata:pinned:t10_rss" == c["name"] for c in res_rss["failed_checks"])


# ---------------------------------------------------------------------------
# 3. VOID fail-closed guards
# ---------------------------------------------------------------------------


def test_void_conditions_and_guards(tmp_path):
    rows = _make_trim_kstar_rows()

    assert ps.score(rows, mode="trim-kstar", host_ok=False)["verdict"] == "VOID"
    assert ps.score(rows, mode="trim-kstar", distinct_binaries=False)["verdict"] == "VOID"
    assert ps.score(rows[:-1], mode="trim-kstar")["verdict"] == "VOID"
    assert ps.score([*rows, dict(rows[0])], mode="trim-kstar")["verdict"] == "VOID"

    bad_err = [dict(r) for r in rows]
    bad_err[0]["err"] = "crc mismatch"
    assert ps.score(bad_err, mode="trim-kstar")["verdict"] == "VOID"

    bad_hash = [dict(r) for r in rows]
    bad_hash[0]["hash"] = "different-hash"
    assert ps.score(bad_hash, mode="trim-kstar")["verdict"] == "VOID"

    bad_nan = [dict(r) for r in rows]
    bad_nan[0]["wall"] = math.nan
    assert ps.score(bad_nan, mode="trim-kstar")["verdict"] == "VOID"

    unbalanced = [dict(r) for r in rows]
    for r in unbalanced:
        if r["corpus"] == "wikidata":
            r["pair_pos"] = 0 if r["arm"] == "base" else 1
    assert ps.score(unbalanced, mode="trim-kstar")["verdict"] == "VOID"

    # Distinct binary guard on real files.
    f1 = tmp_path / "a.bin"
    f2 = tmp_path / "b.bin"
    f3 = tmp_path / "c.bin"
    f1.write_bytes(b"same-bytes")
    f2.write_bytes(b"same-bytes")
    f3.write_bytes(b"different-bytes")
    with pytest.raises(ps.InvalidRun, match="identical"):
        ps.assert_distinct_binaries(f1, f2)
    h1, h3 = ps.assert_distinct_binaries(f1, f3)
    assert h1 != h3

    # Baseline env & CPU quota checks.
    ps.check_baseline_env({})
    for var in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        with pytest.raises(ps.InvalidRun, match=var):
            ps.check_baseline_env({var: "-C target-cpu=native"})

    ps.check_cpu_quota_unlimited({"cpu_quota": None})
    ps.check_cpu_quota_unlimited({"cpu_quota": {"state": "unlimited", "cpus": None}})
    with pytest.raises(ps.InvalidRun, match="quota"):
        ps.check_cpu_quota_unlimited({"cpu_quota": {"state": "limited", "cpus": 4.0}})


# ---------------------------------------------------------------------------
# 4. Mode-robust position-stratified & within-mode (`cycles_per_ref`) estimators
# ---------------------------------------------------------------------------


def test_position_stratified_and_cycles_per_ref_within_mode_estimators():
    # True head/base speedup = 0.90x in BOTH positions, plus a +12% pair_pos=0 cold penalty,
    # AND 4 rounds hit a slow clock mode (cycles_per_ref=0.90 vs 1.30, +35% wall).
    rows = []
    for r in range(12):
        arm_order = ("base", "head") if r % 2 == 0 else ("head", "base")
        for pos, arm in enumerate(arm_order):
            slow = (arm == "base" and r in (0, 2)) or (arm == "head" and r in (1, 3, 5))
            base_w = 0.100
            if arm == "head":
                base_w *= 0.90
            if pos == 0:
                base_w *= 1.12
            if slow:
                base_w *= 1.35
            cpr = 0.90 if slow else 1.30
            rows.append(
                {
                    "corpus": "wikidata",
                    "pinning": "pinned",
                    "threads": 10,
                    "arm": arm,
                    "round": r,
                    "pair_pos": pos,
                    "wall": base_w,
                    "cpu": base_w * 8.0,
                    "rss_kb": 46_000,
                    "out": 1000,
                    "hash": "h",
                    "err": "",
                    "cycles_per_ref": cpr,
                    "runq_s": 0.01,
                }
            )

    est = ps.analyze_cell(rows, threads=10)
    # Position-stratified ratio cancels the +12% pos=0 bias cleanly.
    assert est["position_stratified"]["combined_ratio"] is not None
    # Within-mode uses hardware cycles_per_ref and recovers exact 0.90x in fast mode!
    wm = est["within_mode"]
    assert wm["basis"] == "cycles_per_ref"
    assert wm["base_mode_counts"] == {"fast": 10, "slow": 2}
    assert wm["head_mode_counts"] == {"fast": 9, "slow": 3}
    fast_ratio = wm["by_mode"]["fast"]["ratio"]
    assert fast_ratio is not None
    assert abs(fast_ratio - 0.90) < 0.04

    # Fallback works cleanly when cycles_per_ref is absent.
    no_cpr = [{**r, "cycles_per_ref": None} for r in rows]
    est_fb = ps.analyze_cell(no_cpr, threads=10)
    assert est_fb["within_mode"]["basis"] == "wall_fallback"


# ---------------------------------------------------------------------------
# 5. Shallow --depth=1 checkout, isolated CARGO_TARGET_DIR, CLI & bundle.sh
# ---------------------------------------------------------------------------


def test_shallow_depth1_checkout_from_local_repo(tmp_path):
    origin = tmp_path / "origin"
    origin.mkdir()
    env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "Test",
        "GIT_AUTHOR_EMAIL": "t@example.com",
        "GIT_COMMITTER_NAME": "Test",
        "GIT_COMMITTER_EMAIL": "t@example.com",
    }
    subprocess.run(["git", "init", "-q", "-b", "master"], cwd=origin, check=True)
    (origin / "f.txt").write_text("v1\n", encoding="utf-8")
    subprocess.run(["git", "add", "f.txt"], cwd=origin, check=True)
    subprocess.run(["git", "commit", "-q", "-m", "c1"], cwd=origin, env=env, check=True)
    sha1 = ps.output("git", "rev-parse", "HEAD", cwd=origin)
    (origin / "f.txt").write_text("v2\n", encoding="utf-8")
    subprocess.run(["git", "commit", "-q", "-am", "c2"], cwd=origin, env=env, check=True)

    # Create a --depth=1 shallow clone that only has c2 (NOT c1!).
    shallow = tmp_path / "shallow"
    subprocess.run(
        ["git", "clone", "-q", "--depth=1", f"file://{origin}", str(shallow)],
        check=True,
    )
    # Confirm c1 is absent in shallow clone, then fetch c1 via checkout_ref_shallow!
    has_c1 = subprocess.run(["git", "cat-file", "-e", f"{sha1}^{{commit}}"], cwd=shallow).returncode
    assert has_c1 != 0

    dest = tmp_path / "materialized"
    got_sha = ps.checkout_ref_shallow(shallow, dest, sha1)
    assert got_sha == sha1
    assert (dest / "f.txt").read_text(encoding="utf-8") == "v1\n"


def test_build_arm_binaries_enforces_isolated_target_dir_and_locked(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    seen_envs: list[dict[str, str]] = []
    seen_cmds: list[list[str]] = []

    def fake_run(cmd, *, cwd, env, check):
        seen_cmds.append(list(cmd))
        seen_envs.append(dict(env))
        ex_dir = Path(env["CARGO_TARGET_DIR"]) / "release/examples"
        ex_dir.mkdir(parents=True, exist_ok=True)
        for name in ("ab", "windows"):
            if name in cmd:
                (ex_dir / name).write_bytes(f"bin-{name}-{env['CARGO_TARGET_DIR']}".encode())
        return subprocess.CompletedProcess(cmd, 0)

    clean_env = {
        k: v
        for k, v in os.environ.items()
        if k not in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
    }
    with mock.patch("subprocess.run", side_effect=fake_run):
        b_base = ps.build_arm_binaries(
            src,
            tmp_path / "target-base",
            tmp_path / "bin",
            "base",
            include_windows=True,
            env=clean_env,
        )
        b_head = ps.build_arm_binaries(
            src,
            tmp_path / "target-head",
            tmp_path / "bin",
            "head",
            include_windows=False,
            env=clean_env,
        )

    assert seen_envs[0]["CARGO_TARGET_DIR"] != seen_envs[1]["CARGO_TARGET_DIR"]
    assert "--locked" in seen_cmds[0] and "--locked" in seen_cmds[1]
    assert b_base["ab"].exists() and b_base["windows"].exists() and b_head["ab"].exists()
    ps.assert_distinct_binaries(b_base["ab"], b_head["ab"])


def test_score_cli_and_bundle_sh_smoke(tmp_path):
    rows = _make_trim_kstar_rows()
    run_dir = tmp_path / "run-smoke"
    run_dir.mkdir()
    jsonl = run_dir / "runs.jsonl"
    jsonl.write_text("".join(json.dumps(r) + "\n" for r in rows), encoding="utf-8")

    rc = ps.main(["score", str(jsonl), "--mode", "trim-kstar", "--out", str(run_dir)])
    assert rc == 0
    assert (run_dir / "outcome.txt").read_text(encoding="utf-8").startswith("PASS:")
    assert json.loads((run_dir / "summary.json").read_text(encoding="utf-8"))["verdict"] == "PASS"

    manifest = tmp_path / "bundles.txt"
    env = {**os.environ, "SUBMIT_BUNDLES": str(manifest)}
    subprocess.run(["bash", str(ROOT / "bench/bundle.sh"), str(run_dir)], env=env, check=True)
    bundle_paths = [
        ln.strip() for ln in manifest.read_text(encoding="utf-8").splitlines() if ln.strip()
    ]
    assert len(bundle_paths) == 1
    tar_path = Path(bundle_paths[0])
    assert tar_path.exists()
    with tarfile.open(tar_path, "r:gz") as tf:
        names = tf.getnames()
    assert any(n.endswith("/outcome.txt") for n in names)
    assert any(n.endswith("/summary.json") for n in names)
    assert any(n.endswith("/runs.jsonl") for n in names)


def test_prebuilt_binaries_forbidden_on_registered_i9_run(tmp_path):
    out = tmp_path / "run-i9"
    fake_a = tmp_path / "a"
    fake_b = tmp_path / "b"
    fake_a.write_bytes(b"a")
    fake_b.write_bytes(b"b")
    with (
        mock.patch.object(ps, "check_machine", return_value=[[0, 1]] * 10),
        mock.patch.object(ps, "check_baseline_env", return_value=None),
        mock.patch.object(ps, "checkout_ref_shallow", return_value="deadbeef" * 5),
        mock.patch.object(
            ps, "output", side_effect=lambda *a, **k: "" if "status" in a else "0" * 40
        ),
    ):
        rc = ps.main(
            [
                "run",
                str(out),
                "--mode",
                "trim-kstar",
                "--base-ab",
                str(fake_a),
                "--head-ab",
                str(fake_b),
            ]
        )
    assert rc == 2
    summary = json.loads((out / "summary.json").read_text(encoding="utf-8"))
    assert summary["verdict"] == "VOID"
    assert "forbidden on registered i9 runs" in summary["reason"]


def test_strict_i9_refuses_parameter_overrides(tmp_path):
    for extra in (
        ["--rounds", "4"],
        ["--settle", "0.2"],
        ["--prespin-ms", "0"],
        ["--corpus-scale", "2"],
        ["--base-dir", str(tmp_path)],
    ):
        out = tmp_path / f"override-{extra[0].lstrip('-')}"
        rc = ps.main(["run", str(out), "--mode", "plan-schedule", *extra])
        assert rc == 2
        summary = json.loads((out / "summary.json").read_text(encoding="utf-8"))
        assert summary["verdict"] == "VOID"
        assert "frozen registered parameters overridden" in summary["reason"]


def test_cmd_score_voids_non_registered_provenance(tmp_path):
    run_dir = tmp_path / "non-strict"
    run_dir.mkdir()
    rows = _make_plan_schedule_rows(rounds=4)
    runs_path = run_dir / "runs.jsonl"
    runs_path.write_text("\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8")
    (run_dir / "provenance.json").write_text(
        json.dumps(
            {
                "mode": "plan-schedule",
                "strict_i9": False,
                "smoke": True,
                "rounds": 4,
                "corpus_scale": 1,
                "prespin_ms": ps.PRESPIN_MS,
                "settle_s": ps.SETTLE_S,
                "max_busy": ps.MAX_BUSY,
            }
        ),
        encoding="utf-8",
    )
    out_dir = tmp_path / "rescore"
    rc = ps.main(["score", str(runs_path), "--mode", "plan-schedule", "--out", str(out_dir)])
    assert rc == 2
    summary = json.loads((out_dir / "summary.json").read_text(encoding="utf-8"))
    assert summary["verdict"] == "VOID"
    assert "not the registered i9 run" in summary["reason"]


def test_checkout_ref_shallow_strips_origin_prefix_on_remote_fallback(tmp_path):
    calls: list[list[str]] = []

    def fake_run(cmd, cwd=None, stdout=None, stderr=None, text=None, check=False):
        calls.append(list(cmd))
        if cmd[:2] == ["git", "fetch"] and cmd[-1] == "origin/master":
            return subprocess.CompletedProcess(cmd, 128, "", "couldn't find remote ref")
        return subprocess.CompletedProcess(cmd, 0, "", "")

    with (
        mock.patch("subprocess.run", side_effect=fake_run),
        mock.patch.object(
            ps,
            "output",
            side_effect=lambda *a, **k: (
                "https://example.invalid/repo.git" if "get-url" in a else "a" * 40
            ),
        ),
    ):
        sha = ps.checkout_ref_shallow(tmp_path / "repo", tmp_path / "dest", "origin/master")
    assert sha == "a" * 40
    fetch_refs = [c[-1] for c in calls if c[:2] == ["git", "fetch"]]
    assert fetch_refs == ["origin/master", "master"]
