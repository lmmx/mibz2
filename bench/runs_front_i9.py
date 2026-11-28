#!/usr/bin/env python3
"""Frozen C6 physical i9 adoption run for #1238's `runs::front_avx2` against its master parent.

The candidate is this checkout's HEAD (the ready tag): master at BASE plus #1238's final
`src/runs.rs` and `src/runs/tests.rs` (791ee415: 64-symbol `u64` `vptest` Pass 1, zero-dead-`per`
`[u32; 256]` counts, hoisted 128-bit step table). The control is BASE itself. The decision rule
is PERFORMANCE-POLICY.md section 2, frozen in the registration before any i9 data.

Claim: frontier/claims/runs-front-i9-rescue.toml
Registration: docs/journal/2026-11-28-runs-front-i9/registration.md

Usage:
    python3 bench/runs_front_i9.py run OUT_DIR [--smoke]
    python3 bench/runs_front_i9.py score OUT_DIR
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
REGISTRATION = ROOT / "docs/journal/2026-11-28-runs-front-i9/registration.md"

# The master commit the candidate is built on (its control), frozen before any data.
BASE = "629222aab45f65743ae2d98443f76b1d1c7a1265"
# Every path that can change the decoder or the harness binaries; between BASE and HEAD only
# CANDIDATE_FILES may differ under them.
DECODER_PATHS = ("src", "Cargo.toml", "Cargo.lock", "build.rs", "examples", "benches")
CANDIDATE_FILES = ("src/runs.rs", "src/runs/tests.rs")
FRONT_SYMBOL = re.compile(r"\S*4runs10front_avx2$")

ARM = "mibz2"  # the production default at every thread count
WIKIDATA_COUNTS = (1, 6, 8, 10)
GUARD_COUNTS = (1, 8)
PRIMARY = ("wikidata", "text")
ROUNDS = 16
SETTLE_S = "2"
PRESPIN_MS = 100
MAX_BUSY = "1.5"

# Frozen decision constants (PERFORMANCE-POLICY.md section 2 rules 1-4).
SECONDARY_FREE = 1.05  # up to 5% slower (point): accepted, listed
SECONDARY_TRADE = 1.10  # 5-10%: only against a primary gain at least twice the loss at that k
RSS_MAX = 1.10  # median paired peak-RSS ratio, every cell

CELLS = tuple(("wikidata", k) for k in WIKIDATA_COUNTS) + tuple(
    (name, k) for name in GUARD_CORPORA for k in GUARD_COUNTS
)


def check_source(root: Path = ROOT, base: str = BASE) -> dict[str, Any]:
    """VOID unless HEAD is BASE plus exactly CANDIDATE_FILES under every decoder path."""
    if output("git", "status", "--porcelain", "--untracked-files=no", cwd=root):
        raise InvalidRun("recipe checkout has tracked changes")
    head = output("git", "rev-parse", "HEAD", cwd=root)
    if subprocess.run(["git", "cat-file", "-e", f"{base}^{{commit}}"], cwd=root).returncode:
        subprocess.run(["git", "fetch", "-q", "--no-tags", "origin", base], cwd=root, check=True)
    if subprocess.run(["git", "merge-base", "--is-ancestor", base, head], cwd=root).returncode:
        raise InvalidRun(f"BASE {base} is not an ancestor of HEAD {head}")
    changed = output("git", "diff", "--name-only", base, head, "--", *DECODER_PATHS, cwd=root)
    changed_set = tuple(sorted(filter(None, changed.splitlines())))
    if changed_set != tuple(sorted(CANDIDATE_FILES)):
        raise InvalidRun(f"decoder paths changed between BASE and HEAD: {changed_set}")
    return {"head": head, "base": base, "decoder_diff": list(changed_set)}


def front_asm(exe: Path) -> str:
    syms = [line.split()[-1] for line in output("nm", str(exe)).splitlines()]
    match = [s for s in syms if FRONT_SYMBOL.match(s)]
    if len(match) != 1:
        raise InvalidRun(f"cannot identify runs::front_avx2 in {exe.name}: {match}")
    return output("objdump", "-d", "--no-show-raw-insn", f"--disassemble={match[0]}", str(exe))


def check_identity(out: Path, bins: dict[str, Path]) -> dict[str, Any]:
    """The candidate's front_avx2 carries the 64-symbol vptest Pass 1; the base's does not."""
    info: dict[str, Any] = {}
    for arm, key in (("base", "base_ab"), ("cand", "cand_ab")):
        asm = front_asm(bins[key])
        (out / f"front-avx2-{arm}.asm").write_text(asm + "\n", encoding="utf-8")
        mnems = re.findall(r"^\s+[0-9a-f]+:\s+(\S+)", asm, flags=re.M)
        info[arm] = {
            "instructions": len(mnems),
            **{m: mnems.count(m) for m in ("vptest", "vpermq", "vpshufb", "vzeroupper")},
        }
    if info["cand"]["vptest"] < 1 or info["base"]["vptest"] != 0:
        raise InvalidRun(f"front_avx2 identity not observed (vptest counts {info})")
    save(out / "identity.json", info)
    return info


def verify_parity(out, bins, files, refs, cells=CELLS):
    """Untimed AB_SHA256=1 `ab child mibz2` on every cell, both arms, against C libbzip2."""
    env = {**os.environ, "AB_SHA256": "1"}
    with (out / "parity.jsonl").open("w", encoding="utf-8", buffering=1) as log:
        for corpus, k in cells:
            ref = refs[corpus]
            for arm, key in (("base", "base_ab"), ("cand", "cand_ab")):
                done = subprocess.run(
                    [str(bins[key]), "child", ARM, str(k), str(files[corpus])],
                    capture_output=True,
                    text=True,
                    timeout=300,
                    env=env,
                )
                if done.returncode != 0:
                    raise InvalidRun(f"parity child failed on {arm}/{corpus}/{k}")
                row = json.loads(done.stdout.strip())
                if row.get("err") not in (None, ""):
                    raise InvalidRun(f"parity decoder error on {arm}/{corpus}/{k}")
                if row.get("out") != ref["out"] or row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(f"parity mismatch against libbzip2 on {arm}/{corpus}/{k}")
                if not row.get("hash") or ref.setdefault("hash", row["hash"]) != row["hash"]:
                    raise InvalidRun(f"64-bit hash missing or differing on {corpus}/{k}")
                log.write(json.dumps({"arm": arm, "corpus": corpus, "k": k, **row}) + "\n")


def collect(rows, refs, rounds, cells, cand_exe=None):
    """Validate every timed row; return {(corpus, k, round, arm): row} and arm order per pair."""
    expected = set(cells)
    by_key: dict[tuple[str, int, int, str], dict[str, Any]] = {}
    order: dict[tuple[str, int, int], list[str]] = {}
    for row in rows:
        corpus = str(row.get("corpus") or Path(str(row.get("file", ""))).stem)
        k, r = int(row["threads"]), int(row["round"])
        arm = canonical_arm(str(row["arm"]), cand_exe=cand_exe)
        key = (corpus, k, r, arm)
        if (corpus, k) not in expected or not 0 <= r < rounds or key in by_key:
            raise InvalidRun(f"unexpected or duplicate sample {key}")
        ref = refs.get(corpus)
        if ref is None or row.get("err") not in (None, ""):
            raise InvalidRun(f"decoder error or missing reference on {key}")
        if row.get("out") != ref["out"] or ("hash" in ref and row.get("hash") != ref["hash"]):
            raise InvalidRun(f"output mismatch on {key}")
        for field in ("wall", "rss_kb"):
            v = row.get(field)
            if not (isinstance(v, (int, float)) and math.isfinite(v) and v > 0):
                raise InvalidRun(f"invalid {field} on {key}")
        by_key[key] = row
        order.setdefault((corpus, k, r), []).append(arm)
    if len(by_key) != len(cells) * rounds * 2:
        raise InvalidRun(f"incomplete grid: {len(by_key)} of {len(cells) * rounds * 2} samples")
    return by_key, order


def cell_stats(by_key, order, corpus, k, rounds):
    pairs = [(by_key[corpus, k, r, "base"], by_key[corpus, k, r, "cand"]) for r in range(rounds)]
    bw = [float(b["wall"]) for b, _ in pairs]
    cw = [float(c["wall"]) for _, c in pairs]
    br = [float(b["rss_kb"]) for b, _ in pairs]
    cr = [float(c["rss_kb"]) for _, c in pairs]
    ratio, lo, hi = paired_ratio_ci(cw, bw)
    if not all(math.isfinite(x) and x > 0 for x in (ratio, lo, hi)):
        raise InvalidRun(f"undefined confidence interval at {corpus}/{k}")
    by_pos: dict[int, list[tuple[float, float]]] = {0: [], 1: []}
    for r, (b, c) in enumerate(zip(bw, cw, strict=True)):
        seq = order.get((corpus, k, r), [])
        by_pos[1 if len(seq) == 2 and seq[1] == "cand" else 0].append((b, c))
    med, blo, bhi = bootstrap_median_ci([c / b for b, c in zip(bw, cw, strict=True)])
    return {
        "corpus": corpus,
        "threads": k,
        "class": "primary" if corpus in PRIMARY else "secondary",
        "wall_ratio": ratio,
        "wall_ci95": [lo, hi],
        "wall_base_median": statistics.median(bw),
        "wall_cand_median": statistics.median(cw),
        "wall_paired_median": med,
        "wall_boot_ci95": [blo, bhi],
        "position_stratified": position_stratified_ci(by_pos),
        "rss_ratio": statistics.median([c / b for b, c in zip(br, cr, strict=True)]),
        "rss_base_median_kib": statistics.median(br),
    }


def decide(cells: dict[str, dict[str, Any]]) -> dict[str, Any]:
    """The frozen rule (registration section 4): PASS iff rules 1-4 all hold, else STOP."""
    findings: list[str] = []
    prim = [c for c in cells.values() if c["class"] == "primary"]
    gains = [c for c in prim if c["wall_ci95"][1] < 1.0]
    if not gains:
        findings.append("rule 1: no primary cell faster beyond noise")
    for c in prim:
        if c["wall_ci95"][0] > 1.0:
            findings.append(f"rule 2: {c['corpus']}/{c['threads']} slower beyond noise")
    listed: list[str] = []
    for c in cells.values():
        if c["class"] != "secondary" or c["wall_ratio"] <= SECONDARY_FREE:
            continue
        name, loss = f"{c['corpus']}/{c['threads']}", c["wall_ratio"] - 1.0
        if c["wall_ratio"] > SECONDARY_TRADE:
            findings.append(f"rule 3: {name} {c['wall_ratio']:.4f} > {SECONDARY_TRADE}")
            continue
        paid = [
            g for g in gains if g["threads"] == c["threads"] and 1.0 - g["wall_ratio"] >= 2 * loss
        ]
        if not paid:
            findings.append(f"rule 3: {name} {c['wall_ratio']:.4f} without a 2x primary gain")
        else:
            listed.append(name)
    for name, c in cells.items():
        if c["rss_ratio"] > RSS_MAX:
            findings.append(
                f"rule 4: {name} median peak RSS ratio {c['rss_ratio']:.4f} > {RSS_MAX}"
            )
    secondary_listed = sorted(
        f"{n} {c['wall_ratio']:.4f}"
        for n, c in cells.items()
        if c["class"] == "secondary" and c["wall_ratio"] > 1.0
    )
    return {
        "verdict": "STOP" if findings else "PASS",
        "findings": findings,
        "primary_gains": sorted(f"{g['corpus']}/{g['threads']}" for g in gains),
        "secondary_traded": listed,
        "secondary_slower_points": secondary_listed,
    }


def score(rows, refs, rounds=ROUNDS, cells=CELLS, cand_exe=None) -> dict[str, Any]:
    by_key, order = collect(rows, refs, rounds, cells, cand_exe)
    stats = {f"{c}/{k}": cell_stats(by_key, order, c, k, rounds) for c, k in cells}
    return {**decide(stats), "cells": stats}


def format_summary(summary: dict[str, Any]) -> str:
    lines = [f"{summary['verdict']}: {'; '.join(summary.get('findings') or ['rules 1-4 hold'])}"]
    for name, c in summary.get("cells", {}).items():
        lo, hi = c["wall_ci95"]
        lines.append(
            f"{name:<12s} {c['class']:<9s} base {c['wall_base_median']:.4f}s "
            f"ratio {c['wall_ratio']:.4f} [{lo:.4f}, {hi:.4f}] rss {c['rss_ratio']:.4f}"
        )
    return "\n".join(lines) + "\n"


def run(out: Path, smoke: bool = False) -> dict[str, Any]:
    out.mkdir(parents=True, exist_ok=True)
    check_baseline_env()
    toolchain = output("rustc", "-Vv")
    if not smoke and not toolchain.startswith("rustc 1.99.1 "):
        raise InvalidRun("registered toolchain requires Rust 1.99.1")
    (out / "toolchain.txt").write_text(toolchain + "\n", encoding="utf-8")
    shutil.copyfile(REGISTRATION, out / "registration.md")
    source = check_source()
    host = check_host_i9(out, require_i9=not smoke)
    rounds = 2 if smoke else ROUNDS
    w_counts, g_counts = ((1, 2), (1, 2)) if smoke else (WIKIDATA_COUNTS, GUARD_COUNTS)
    guards = ("text", "runs") if smoke else GUARD_CORPORA
    cells = tuple(("wikidata", k) for k in w_counts) + tuple(
        (n, k) for n in guards for k in g_counts
    )
    settle, prespin = ("0", 0) if smoke else (SETTLE_S, PRESPIN_MS)
    save(
        out / "provenance.json",
        {**source, "smoke": smoke, "rounds": rounds, "settle_s": settle, "prespin_ms": prespin},
    )
    with tempfile.TemporaryDirectory(prefix="runs-front-i9-", dir=out.parent) as td:
        tmp = Path(td)
        base_root = tmp / "base"
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(base_root), BASE], cwd=ROOT, check=True
        )
        try:
            clean = {k: v for k, v in os.environ.items() if not k.endswith("RUSTFLAGS")}
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
                out, tmp, root=base_root, base_features="", cand_features="", prebuilt_cand=cand_ab
            )
            check_identity(out, bins)
            files, refs = prepare_inputs(
                out, tmp, bins, root=ROOT, smoke=smoke, guard_corpora=guards
            )
            verify_parity(out, bins, files, refs, cells=cells)
            facts = output(str(bins["base_ab"]), "env", "--facts")
            (out / "env-facts.json").write_text(facts + "\n", encoding="utf-8")
            if not smoke:
                check_no_quota(json.loads(facts))
            common = dict(
                arm_name=ARM,
                rounds=rounds,
                pin_cpu=int(host["pin_cpu"]),
                prespin_ms=prespin,
                settle_s=settle,
                max_busy=MAX_BUSY,
                machine_state=not smoke,
            )
            rows = run_ab_group(
                bins["base_ab"],
                bins["cand_ab"],
                out / "wikidata.jsonl",
                counts=w_counts,
                corpus_paths=[files["wikidata"]],
                **common,
            )
            rows += run_ab_group(
                bins["base_ab"],
                bins["cand_ab"],
                out / "guards.jsonl",
                counts=g_counts,
                corpus_paths=[files[n] for n in guards],
                **common,
            )
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
    if prov.get("smoke") or int(prov["rounds"]) != ROUNDS or prov.get("base") != BASE:
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
