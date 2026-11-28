#!/usr/bin/env python3
"""Frozen C6 physical i9 verification run for zero-extra-RSS A6b (`overlay_aos`).

Compares candidate HEAD (`BASE` + `src/open/endpoint.rs` + `src/open.rs`) against frozen `BASE`
using isolated `CARGO_TARGET_DIR`s, paired alternating `ab run` fresh processes, exact C libbzip2
SHA-256 parity, `objdump` codegen identity, and the frozen four-rule decision criteria in
`docs/journal/2026-11-28-a6b-zero-rss-endpoint/registration.md`.

Claim: frontier/claims/a6b-zero-rss-endpoint-build.toml
Registration: docs/journal/2026-11-28-a6b-zero-rss-endpoint/registration.md

Usage:
    python3 bench/a6b_zero_rss_i9.py run OUT_DIR [--smoke]
    python3 bench/a6b_zero_rss_i9.py score OUT_DIR
"""

from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
from typing import Any

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

from forward_adopt import (  # noqa: E402
    GUARD_CORPORA,
    InvalidRun,
    bootstrap_median_ci,
    build_arms,
    canonical_arm,
    check_baseline_env,
    check_host_i9,
    check_no_quota,
    output,
    position_stratified_ci,
    prepare_inputs,
    run_ab_group,
    save,
)
from mtf_fold import paired_ratio_ci  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-28-a6b-zero-rss-endpoint/registration.md"

# Master commit the candidate is built on (control), frozen before any i9 data.
BASE = "6679ddeed640f9e9cdd6aa3f71e80a7fb43326b7"
DECODER_PATHS = ("src", "Cargo.toml", "Cargo.lock", "build.rs", "examples", "benches")
CANDIDATE_FILES = ("src/open.rs", "src/open/endpoint.rs")

ELIM_AOS_SYMBOL = "mibz2::open::endpoint::overlay_aos::elim_aos::<131072, false>"
WORKER_THREAD_RE = re.compile(r"\bmibz2-(\d+)=([0-9]+(?:\.[0-9]+)?)")

LAZY_ARM = "mibz2-workers-lazy"
DEFAULT_ARM = "mibz2"

LAZY_WIKIDATA_COUNTS = (1, 6, 8)
DEFAULT_WIKIDATA_COUNTS = (1, 4, 6, 8, 10)
GUARD_COUNTS = (1, 8)

ROUNDS = 12
SETTLE_S = "2"
PRESPIN_MS = 100
MAX_BUSY = "1.5"

# Frozen decision constants (claim `a6b-zero-rss-endpoint-build` stop_test Clause 2).
WIKI1_WALL_MAX = 0.985
MULTI_CPU_MAX = 0.990
MULTI_WALL_MAX = 1.000
RSS_MAX = 1.005
GUARD_WALL_MAX = 1.010

CELLS: tuple[tuple[str, str, int], ...] = (
    tuple((LAZY_ARM, "wikidata", k) for k in LAZY_WIKIDATA_COUNTS)
    + tuple((DEFAULT_ARM, "wikidata", k) for k in DEFAULT_WIKIDATA_COUNTS)
    + tuple((DEFAULT_ARM, name, k) for name in GUARD_CORPORA for k in GUARD_COUNTS)
)

MULTI_GATE_CELLS = frozenset(
    {
        (DEFAULT_ARM, "wikidata", 6),
        (DEFAULT_ARM, "wikidata", 8),
        (LAZY_ARM, "wikidata", 6),
        (LAZY_ARM, "wikidata", 8),
    }
)


def check_baseline_env_strict(env: dict[str, str] | None = None) -> None:
    """Enforce baseline x86-64 codegen with no RUSTFLAGS or CARGO_PROFILE_RELEASE_* overrides."""
    cur = os.environ if env is None else env
    check_baseline_env(cur)
    for var, val in sorted(cur.items()):
        if (var.startswith("CARGO_PROFILE_RELEASE_") or var.endswith("RUSTFLAGS")) and val.strip():
            raise InvalidRun(f"non-empty {var}={val!r}; registered run forbids build overrides")


def check_source(root: Path = ROOT, base: str = BASE) -> dict[str, Any]:
    """VOID unless HEAD is BASE plus exactly CANDIDATE_FILES under every decoder path.

    Shallow-safe: if BASE is absent on a `--depth 1` checkout, fetches BASE alone at depth 1
    and skips ancestry verification while recording `shallow` and `ancestry_checked`.
    """
    if output("git", "status", "--porcelain", "--untracked-files=no", cwd=root):
        raise InvalidRun("recipe checkout has tracked changes")
    head = output("git", "rev-parse", "HEAD", cwd=root)
    shallow = output("git", "rev-parse", "--is-shallow-repository", cwd=root) == "true"
    if subprocess.run(
        ["git", "cat-file", "-e", f"{base}^{{commit}}"], cwd=root, capture_output=True
    ).returncode:
        fetch = [
            "git",
            "fetch",
            "-q",
            "--no-tags",
            *(["--depth=1"] if shallow else []),
            "origin",
            base,
        ]
        if subprocess.run(fetch, cwd=root).returncode:
            raise InvalidRun(f"cannot fetch BASE {base} from origin")
    ancestry: bool | None = None
    if not shallow:
        ancestry = not subprocess.run(
            ["git", "merge-base", "--is-ancestor", base, head], cwd=root
        ).returncode
        if not ancestry:
            raise InvalidRun(f"BASE {base} is not an ancestor of HEAD {head}")
    changed = output("git", "diff", "--name-only", base, head, "--", *DECODER_PATHS, cwd=root)
    changed_set = tuple(sorted(filter(None, changed.splitlines())))
    if changed_set != tuple(sorted(CANDIDATE_FILES)):
        raise InvalidRun(f"decoder paths changed between BASE and HEAD: {changed_set}")
    return {
        "head": head,
        "base": base,
        "decoder_diff": list(changed_set),
        "shallow": shallow,
        "ancestry_checked": ancestry,
    }


def check_identity(out: Path, bins: dict[str, Path]) -> dict[str, Any]:
    """Verify candidate contains zero-bounds-check `overlay_aos::elim_aos` and base does not."""
    info: dict[str, Any] = {}
    for arm, key in (("base", "base_ab"), ("cand", "cand_ab")):
        nm_lines = output("nm", "-C", str(bins[key])).splitlines()
        matches = [line.split(" ", 2)[-1] for line in nm_lines if "overlay_aos::elim_aos" in line]
        entry: dict[str, Any] = {"overlay_elim_symbols": len(matches)}
        if arm == "cand":
            if ELIM_AOS_SYMBOL not in matches:
                raise InvalidRun(f"candidate missing {ELIM_AOS_SYMBOL}; found {matches[:5]}")
            asm = output(
                "objdump",
                "-dC",
                "--no-show-raw-insn",
                f"--disassemble={ELIM_AOS_SYMBOL}",
                str(bins[key]),
            )
            (out / "elim-aos-cand.asm").write_text(asm + "\n", encoding="utf-8")
            mnems = re.findall(r"^\s+[0-9a-f]+:\s+(\S+)", asm, flags=re.M)
            bc_calls = len(re.findall(r"panic_bounds_check|slice_end_index", asm))
            entry["elim_131072_instructions"] = len(mnems)
            entry["elim_131072_bounds_checks"] = bc_calls
            if bc_calls != 0:
                raise InvalidRun(f"candidate elim_aos has {bc_calls} bounds checks (expected 0)")
        elif matches:
            raise InvalidRun(
                f"base binary unexpectedly contains overlay_aos symbols: {matches[:5]}"
            )
        info[arm] = entry
    save(out / "identity.json", info)
    return info


def parse_worker_thread_cpu(row: dict[str, Any]) -> float | None:
    """Sum `mibz2-<n>=<sec>` entries from `thread_cpu` when present and positive."""
    raw = str(row.get("thread_cpu") or "")
    vals = [float(m.group(2)) for m in WORKER_THREAD_RE.finditer(raw)]
    total = sum(vals)
    return total if total > 0.0 else None


def verify_parity(
    out: Path,
    bins: dict[str, Path],
    files: dict[str, Path],
    refs: dict[str, dict[str, Any]],
    cells: tuple[tuple[str, str, int], ...] = CELLS,
) -> None:
    """Untimed AB_SHA256=1 `ab child` on every cell, both arms, verified against C libbzip2."""
    env = {**os.environ, "AB_SHA256": "1"}
    with (out / "parity.jsonl").open("w", encoding="utf-8", buffering=1) as log:
        for arm_mode, corpus, k in cells:
            ref = refs[corpus]
            for side, key in (("base", "base_ab"), ("cand", "cand_ab")):
                done = subprocess.run(
                    [str(bins[key]), "child", arm_mode, str(k), str(files[corpus])],
                    capture_output=True,
                    text=True,
                    timeout=300,
                    env=env,
                )
                if done.returncode != 0:
                    raise InvalidRun(f"parity child failed on {side}/{arm_mode}/{corpus}/{k}")
                row = json.loads(done.stdout.strip())
                if row.get("err") not in (None, ""):
                    raise InvalidRun(f"parity decoder error on {side}/{arm_mode}/{corpus}/{k}")
                if row.get("out") != ref["out"] or row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(
                        f"parity mismatch against libbzip2 on {side}/{arm_mode}/{corpus}/{k}"
                    )
                if not row.get("hash") or ref.setdefault("hash", row["hash"]) != row["hash"]:
                    raise InvalidRun(f"64-bit hash missing or differing on {arm_mode}/{corpus}/{k}")
                log.write(
                    json.dumps(
                        {"side": side, "arm_mode": arm_mode, "corpus": corpus, "k": k, **row}
                    )
                    + "\n"
                )


def _row_arm_mode(row: dict[str, Any]) -> str:
    explicit = row.get("arm_mode")
    if explicit:
        return str(explicit)
    raw = str(row.get("arm", ""))
    return raw.split("@", 1)[0]


def collect(
    rows: list[dict[str, Any]],
    refs: dict[str, dict[str, Any]],
    rounds: int,
    cells: tuple[tuple[str, str, int], ...],
    cand_exe: Path | str | None = None,
) -> tuple[
    dict[tuple[str, str, int, int, str], dict[str, Any]],
    dict[tuple[str, str, int, int], list[str]],
]:
    """Validate every timed row; return {(arm_mode, corpus, k, round, side): row} and order."""
    expected = set(cells)
    by_key: dict[tuple[str, str, int, int, str], dict[str, Any]] = {}
    order: dict[tuple[str, str, int, int], list[str]] = {}
    for row in rows:
        arm_mode = _row_arm_mode(row)
        corpus = str(row.get("corpus") or Path(str(row.get("file", ""))).stem)
        k, r = int(row["threads"]), int(row["round"])
        side = canonical_arm(str(row["arm"]), cand_exe=cand_exe)
        key = (arm_mode, corpus, k, r, side)
        if (arm_mode, corpus, k) not in expected or not 0 <= r < rounds or key in by_key:
            raise InvalidRun(f"unexpected or duplicate sample {key}")
        ref = refs.get(corpus)
        if ref is None or row.get("err") not in (None, ""):
            raise InvalidRun(f"decoder error or missing reference on {key}")
        if row.get("out") != ref["out"] or ("hash" in ref and row.get("hash") != ref["hash"]):
            raise InvalidRun(f"output mismatch on {key}")
        for field in ("wall", "cpu", "rss_kb"):
            v = row.get(field)
            if not (isinstance(v, (int, float)) and math.isfinite(v) and v > 0):
                raise InvalidRun(f"invalid {field} on {key}")
        by_key[key] = row
        order.setdefault((arm_mode, corpus, k, r), []).append(side)
    if len(by_key) != len(cells) * rounds * 2:
        raise InvalidRun(f"incomplete grid: {len(by_key)} of {len(cells) * rounds * 2} samples")
    return by_key, order


def cell_stats(
    by_key: dict[tuple[str, str, int, int, str], dict[str, Any]],
    order: dict[tuple[str, str, int, int], list[str]],
    arm_mode: str,
    corpus: str,
    k: int,
    rounds: int,
) -> dict[str, Any]:
    pairs = [
        (by_key[arm_mode, corpus, k, r, "base"], by_key[arm_mode, corpus, k, r, "cand"])
        for r in range(rounds)
    ]
    bw = [float(b["wall"]) for b, _ in pairs]
    cw = [float(c["wall"]) for _, c in pairs]
    bc = [float(b["cpu"]) for b, _ in pairs]
    cc = [float(c["cpu"]) for _, c in pairs]
    br = [float(b["rss_kb"]) for b, _ in pairs]
    cr = [float(c["rss_kb"]) for _, c in pairs]

    w_ratio, w_lo, w_hi = paired_ratio_ci(cw, bw)
    c_ratio, c_lo, c_hi = paired_ratio_ci(cc, bc)
    if not all(math.isfinite(x) and x > 0 for x in (w_ratio, w_lo, w_hi, c_ratio, c_lo, c_hi)):
        raise InvalidRun(f"undefined confidence interval at {arm_mode}/{corpus}/{k}")

    by_pos: dict[int, list[tuple[float, float]]] = {0: [], 1: []}
    for r, (b, c) in enumerate(zip(bw, cw, strict=True)):
        seq = order.get((arm_mode, corpus, k, r), [])
        by_pos[1 if len(seq) == 2 and seq[1] == "cand" else 0].append((b, c))
    w_med, w_blo, w_bhi = bootstrap_median_ci([c / b for b, c in zip(bw, cw, strict=True)])

    b_wt = [parse_worker_thread_cpu(b) for b, _ in pairs]
    c_wt = [parse_worker_thread_cpu(c) for _, c in pairs]
    wt_ratio: float | None = None
    if all(v is not None for v in b_wt) and all(v is not None for v in c_wt):
        wt_ratio = statistics.median([c / b for b, c in zip(b_wt, c_wt, strict=True)])  # type: ignore[operator]

    return {
        "arm_mode": arm_mode,
        "corpus": corpus,
        "threads": k,
        "wall_ratio": w_ratio,
        "wall_ci95": [w_lo, w_hi],
        "wall_base_median": statistics.median(bw),
        "wall_cand_median": statistics.median(cw),
        "wall_paired_median": w_med,
        "wall_boot_ci95": [w_blo, w_bhi],
        "cpu_ratio": c_ratio,
        "cpu_ci95": [c_lo, c_hi],
        "cpu_base_median": statistics.median(bc),
        "cpu_cand_median": statistics.median(cc),
        "worker_thread_cpu_ratio": wt_ratio,
        "position_stratified": position_stratified_ci(by_pos),
        "rss_ratio": statistics.median([c / b for b, c in zip(br, cr, strict=True)]),
        "rss_base_median_kib": statistics.median(br),
        "rss_cand_median_kib": statistics.median(cr),
    }


def decide(cells: dict[str, dict[str, Any]]) -> dict[str, Any]:
    """Frozen four-rule verdict (registration section 4): PASS iff rules 1-4 all hold, else STOP."""
    findings: list[str] = []

    w1_key = f"{LAZY_ARM}/wikidata/1"
    w1 = cells.get(w1_key)
    if w1 is None:
        findings.append(f"rule 1: missing {w1_key}")
    else:
        if w1["wall_ratio"] > WIKI1_WALL_MAX:
            findings.append(f"rule 1: {w1_key} wall {w1['wall_ratio']:.4f} > {WIKI1_WALL_MAX}")
        if w1["wall_ci95"][1] >= 1.0:
            findings.append(f"rule 1: {w1_key} 95% CI upper {w1['wall_ci95'][1]:.4f} >= 1.000")

    for arm_mode, corpus, k in sorted(MULTI_GATE_CELLS):
        key = f"{arm_mode}/{corpus}/{k}"
        c = cells.get(key)
        if c is None:
            continue
        if c["cpu_ratio"] > MULTI_CPU_MAX:
            findings.append(f"rule 2: {key} cpu {c['cpu_ratio']:.4f} > {MULTI_CPU_MAX}")
        if c["wall_ratio"] > MULTI_WALL_MAX:
            findings.append(f"rule 2: {key} wall {c['wall_ratio']:.4f} > {MULTI_WALL_MAX}")

    for key, c in cells.items():
        if c["rss_ratio"] > RSS_MAX:
            findings.append(f"rule 3: {key} median peak RSS ratio {c['rss_ratio']:.4f} > {RSS_MAX}")

    for key, c in cells.items():
        triple = (c["arm_mode"], c["corpus"], int(c["threads"]))
        if triple == (LAZY_ARM, "wikidata", 1) or triple in MULTI_GATE_CELLS:
            continue
        if c["wall_ratio"] > GUARD_WALL_MAX:
            findings.append(f"rule 4: {key} wall {c['wall_ratio']:.4f} > {GUARD_WALL_MAX}")

    return {
        "verdict": "STOP" if findings else "PASS",
        "findings": findings,
    }


def score(
    rows: list[dict[str, Any]],
    refs: dict[str, dict[str, Any]],
    rounds: int = ROUNDS,
    cells: tuple[tuple[str, str, int], ...] = CELLS,
    cand_exe: Path | str | None = None,
) -> dict[str, Any]:
    by_key, order = collect(rows, refs, rounds, cells, cand_exe)
    stats = {
        f"{arm_mode}/{corpus}/{k}": cell_stats(by_key, order, arm_mode, corpus, k, rounds)
        for arm_mode, corpus, k in cells
    }
    return {**decide(stats), "cells": stats}


def format_summary(summary: dict[str, Any]) -> str:
    lines = [f"{summary['verdict']}: {'; '.join(summary.get('findings') or ['rules 1-4 hold'])}"]
    for name, c in summary.get("cells", {}).items():
        wlo, whi = c["wall_ci95"]
        lines.append(
            f"{name:<28s} wall {c['wall_ratio']:.4f} [{wlo:.4f}, {whi:.4f}] "
            f"cpu {c['cpu_ratio']:.4f} rss {c['rss_ratio']:.4f}"
        )
    return "\n".join(lines) + "\n"


def run(out: Path, smoke: bool = False) -> dict[str, Any]:
    out.mkdir(parents=True, exist_ok=True)
    check_baseline_env_strict()
    toolchain = output("rustc", "-Vv")
    if not smoke and not toolchain.startswith("rustc 1.99.1 "):
        raise InvalidRun("registered toolchain requires Rust 1.99.1")
    (out / "toolchain.txt").write_text(toolchain + "\n", encoding="utf-8")
    shutil.copyfile(REGISTRATION, out / "registration.md")
    source = check_source()
    host = check_host_i9(out, require_i9=not smoke)

    rounds = 2 if smoke else ROUNDS
    lazy_w_counts = (1, 2) if smoke else LAZY_WIKIDATA_COUNTS
    def_w_counts = (1, 2) if smoke else DEFAULT_WIKIDATA_COUNTS
    g_counts = (1,) if smoke else GUARD_COUNTS
    guards = ("text", "runs") if smoke else GUARD_CORPORA
    cells: tuple[tuple[str, str, int], ...] = (
        tuple((LAZY_ARM, "wikidata", k) for k in lazy_w_counts)
        + tuple((DEFAULT_ARM, "wikidata", k) for k in def_w_counts)
        + tuple((DEFAULT_ARM, n, k) for n in guards for k in g_counts)
    )
    settle, prespin = ("0", 0) if smoke else (SETTLE_S, PRESPIN_MS)
    save(
        out / "provenance.json",
        {**source, "smoke": smoke, "rounds": rounds, "settle_s": settle, "prespin_ms": prespin},
    )

    with tempfile.TemporaryDirectory(prefix="a6b-zero-rss-", dir=out.parent) as td:
        tmp = Path(td)
        base_root = tmp / "base"
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(base_root), BASE],
            cwd=ROOT,
            check=True,
        )
        try:
            clean = {
                k: v
                for k, v in os.environ.items()
                if not k.endswith("RUSTFLAGS") and not k.startswith("CARGO_PROFILE_RELEASE_")
            }
            cand_target = tmp / "target-cand"
            subprocess.run(
                ["cargo", "build", "--release", "--locked", "--example", "ab"],
                cwd=ROOT,
                check=True,
                env={**clean, "CARGO_TARGET_DIR": str(cand_target)},
            )
            cand_ab = tmp / "cand-ab"
            shutil.copy2(cand_target / "release/examples/ab", cand_ab)
            bins, _ = build_arms(
                out,
                tmp,
                root=base_root,
                base_features="",
                cand_features="",
                prebuilt_cand=cand_ab,
            )
            check_identity(out, bins)
            files, refs = prepare_inputs(
                out, tmp, bins, root=ROOT, smoke=smoke, guard_corpora=guards
            )
            verify_parity(out, bins, files, refs, cells=cells)
            facts = output(str(bins["base_ab"]), "env", "--facts")
            (out / "env-facts.json").write_text(facts + "\n", encoding="utf-8")
            check_no_quota(json.loads(facts))

            common = dict(
                rounds=rounds,
                pin_cpu=int(host["pin_cpu"]),
                prespin_ms=prespin,
                settle_s=settle,
                max_busy=MAX_BUSY,
                machine_state=not smoke,
            )
            rows_lazy = run_ab_group(
                bins["base_ab"],
                bins["cand_ab"],
                out / "wikidata-lazy.jsonl",
                arm_name=LAZY_ARM,
                counts=lazy_w_counts,
                corpus_paths=[files["wikidata"]],
                **common,
            )
            for r in rows_lazy:
                r["arm_mode"] = LAZY_ARM

            rows_def = run_ab_group(
                bins["base_ab"],
                bins["cand_ab"],
                out / "wikidata-default.jsonl",
                arm_name=DEFAULT_ARM,
                counts=def_w_counts,
                corpus_paths=[files["wikidata"]],
                **common,
            )
            for r in rows_def:
                r["arm_mode"] = DEFAULT_ARM

            rows_guards = run_ab_group(
                bins["base_ab"],
                bins["cand_ab"],
                out / "guards.jsonl",
                arm_name=DEFAULT_ARM,
                counts=g_counts,
                corpus_paths=[files[n] for n in guards],
                **common,
            )
            for r in rows_guards:
                r["arm_mode"] = DEFAULT_ARM

            rows = rows_lazy + rows_def + rows_guards
            with (out / "runs.jsonl").open("x", encoding="utf-8") as fh:
                for row in rows:
                    fh.write(json.dumps(row, allow_nan=False) + "\n")
            save(out / "refs.json", refs)
            summary = score(rows, refs, rounds=rounds, cells=cells, cand_exe=bins["cand_ab"])
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(base_root)], cwd=ROOT)

    if smoke:
        summary["verdict"] = f"SMOKE ({summary['verdict']}; not the registered run)"
    save(out / "summary.json", summary)
    text = format_summary(summary)
    (out / "summary.txt").write_text(text, encoding="utf-8")
    (out / "outcome.txt").write_text(text.splitlines()[0] + "\n", encoding="utf-8")
    print(text, end="")
    return summary


def score_dir(out: Path) -> dict[str, Any]:
    prov = json.loads((out / "provenance.json").read_text(encoding="utf-8"))
    refs = json.loads((out / "refs.json").read_text(encoding="utf-8"))
    rows = [json.loads(x) for x in (out / "runs.jsonl").read_text().splitlines() if x.strip()]
    if (
        prov.get("smoke")
        or int(prov.get("rounds", -1)) != ROUNDS
        or prov.get("base") != BASE
        or str(prov.get("settle_s", "")) != SETTLE_S
        or int(prov.get("prespin_ms", -1)) != PRESPIN_MS
    ):
        return {"verdict": "VOID", "findings": ["not the registered run"]}
    return score(rows, refs)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    p_run = sub.add_parser("run")
    p_run.add_argument("out", type=Path)
    p_run.add_argument("--smoke", action="store_true", help="any host, 2 rounds; decides nothing")
    p_score = sub.add_parser("score")
    p_score.add_argument("out", type=Path)
    args = p.parse_args(argv)
    out = args.out.resolve()
    if args.cmd == "score":
        print(format_summary(score_dir(out)), end="")
        return 0
    try:
        run(out, smoke=args.smoke)
    except InvalidRun as err:
        out.mkdir(parents=True, exist_ok=True)
        (out / "outcome.txt").write_text(f"VOID: {err}\n", encoding="utf-8")
        print(f"VOID: {err}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
