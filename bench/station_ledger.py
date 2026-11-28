#!/usr/bin/env python3
"""Read-only per-station CPU ledger for archived `ab` JSONL files and bundles.

Reads one or more `ab` JSONL files, directories, or `.tar.gz`/`.tar` bundles and
prints, per `(corpus, threads, arm)`:

- median wall time (`wall ms`)
- mean scanner CPU (`S ms`, `mibz2-scan`)
- mean total worker CPU (`W ms`, sum of `mibz2-N` worker threads)
- worker count (`k`)
- mean worker CPU per worker (`W/k ms`)
- station gap (`W/k - S`, positive = worker-bound, negative = scanner-bound)
- residual outside both stations (`wall - max(S, W/k)`)
- binding station label (`workers`, `scanner`, `tie`, `residual`, `single`, `other`)
- a `W_GROWTH` flag when an arm family's `W` grows by more than 10% between its
  smallest and largest `k` (indicating waiting or contention, not work demand)

Rows with missing or empty `thread_cpu` are skipped and counted.

Usage:
    python3 bench/station_ledger.py BUNDLE_JSONL... [--json] [--corpus NAME] [--arm SUBSTR]
"""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
import io
import json
import math
from pathlib import Path
import re
import statistics
import sys
import tarfile
from typing import Any, Iterable

WORKER_RE = re.compile(r"mibz2-\d+")
OCC_SUFFIX_RE = re.compile(r"-\d+-\d+$")
TIE_BAND_MS = 1.0
DEFAULT_GROWTH_THRESHOLD_PCT = 10.0


@dataclass(frozen=True)
class CellLedger:
    corpus: str
    threads: int
    arm: str
    arm_family: str
    n: int
    k: int
    wall_ms: float
    s_ms: float
    w_ms: float
    main_ms: float
    w_per_k_ms: float | None
    delta_ms: float | None
    residual_ms: float | None
    binding: str
    w_growth_flag: bool = False
    w_growth_pct: float | None = None


@dataclass(frozen=True)
class FamilyGrowth:
    corpus: str
    arm_family: str
    k_min: int
    k_max: int
    w_min_ms: float
    w_max_ms: float
    growth_pct: float
    flagged: bool


@dataclass(frozen=True)
class LedgerReport:
    source: str
    rows_total: int
    rows_used: int
    skipped_empty_thread_cpu: int
    skipped_invalid: int
    growth_threshold_pct: float
    cells: list[CellLedger]
    families: list[FamilyGrowth]


def arm_family(arm: str) -> str:
    """Group occupancy/plan variants (e.g. `mibz2-occ-6-24`, `mibz2-occ-10-40`) into one family."""
    base, sep, exe = arm.partition("@")
    base = OCC_SUFFIX_RE.sub("", base)
    return f"{base}{sep}{exe}"


def display_arm(arm: str, max_len: int = 28) -> str:
    """Shorten long `@/abs/path/.../ab` suffixes for table alignment while keeping arm identity."""
    if "@" not in arm or len(arm) <= max_len:
        return arm
    base, exe = arm.split("@", 1)
    parts = [p for p in Path(exe).parts if p not in ("/", "")]
    for part in reversed(parts):
        if part.startswith("build-"):
            return f"{base}@{part}"
    tail = parts[-1] if parts else exe
    if len(parts) >= 2 and tail == "ab":
        tail = f"{parts[-2]}/{tail}"
    return f"{base}@{tail}"


def parse_thread_cpu(thread_cpu: str) -> tuple[float, float, int, float]:
    """Return `(S_ms, W_ms, k, main_ms)` from an `ab` `thread_cpu` string.

    Iterates every whitespace-separated `name=seconds` token without collapsing
    duplicate thread names.
    """
    s_s = 0.0
    w_s = 0.0
    k = 0
    main_s = 0.0
    for token in thread_cpu.split():
        name, sep, raw_val = token.rpartition("=")
        if not sep or not name:
            continue
        try:
            val = float(raw_val)
        except ValueError:
            continue
        if not math.isfinite(val):
            continue
        if name == "mibz2-scan":
            s_s += val
        elif WORKER_RE.fullmatch(name):
            w_s += val
            k += 1
        elif name == "main":
            main_s += val
    return s_s * 1000.0, w_s * 1000.0, k, main_s * 1000.0


def _mode_k(k_vals: list[int]) -> int:
    counts = Counter(k_vals)
    # Deterministic tie-break: highest count, then larger k.
    return max(counts.items(), key=lambda item: (item[1], item[0]))[0]


def _classify_binding(
    k: int,
    s_ms: float,
    w_per_k_ms: float | None,
    delta_ms: float | None,
    residual_ms: float | None,
    main_ms: float,
) -> str:
    if k <= 0 or w_per_k_ms is None or delta_ms is None or residual_ms is None:
        return "single" if main_ms > 0.0 and s_ms == 0.0 else "other"
    dominant = max(s_ms, w_per_k_ms)
    if residual_ms > dominant:
        return "residual"
    if abs(delta_ms) <= TIE_BAND_MS:
        return "tie"
    return "workers" if delta_ms > 0.0 else "scanner"


def analyze_lines(
    lines: Iterable[str],
    *,
    source: str = "<stdin>",
    growth_threshold_pct: float = DEFAULT_GROWTH_THRESHOLD_PCT,
    corpora: set[str] | None = None,
    arm_substr: str | None = None,
) -> LedgerReport:
    """Compute the per-station ledger and arm-family W-growth flags over JSONL lines."""
    rows_total = 0
    rows_used = 0
    skipped_empty = 0
    skipped_invalid = 0
    grouped: dict[tuple[str, int, str], list[tuple[float, float, float, int, float]]] = defaultdict(
        list
    )

    for raw in lines:
        line = raw.strip()
        if not line:
            continue
        rows_total += 1
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            skipped_invalid += 1
            continue
        if not isinstance(row, dict):
            skipped_invalid += 1
            continue

        tc = row.get("thread_cpu")
        if not isinstance(tc, str) or not tc.strip():
            skipped_empty += 1
            continue

        wall = row.get("wall")
        corpus = row.get("corpus")
        threads = row.get("threads")
        arm = row.get("arm")
        if (
            not isinstance(wall, (int, float))
            or not math.isfinite(float(wall))
            or float(wall) <= 0.0
            or not isinstance(corpus, str)
            or not isinstance(threads, int)
            or isinstance(threads, bool)
            or not isinstance(arm, str)
        ):
            skipped_invalid += 1
            continue

        if corpora is not None and corpus not in corpora:
            continue
        if arm_substr is not None and arm_substr not in arm:
            continue

        s_ms, w_ms, k, main_ms = parse_thread_cpu(tc)
        grouped[(corpus, threads, arm)].append((float(wall) * 1000.0, s_ms, w_ms, k, main_ms))
        rows_used += 1

    raw_cells: list[CellLedger] = []
    for (corpus, threads, arm), samples in sorted(grouped.items()):
        walls = [x[0] for x in samples]
        s_vals = [x[1] for x in samples]
        w_vals = [x[2] for x in samples]
        k_vals = [x[3] for x in samples]
        main_vals = [x[4] for x in samples]

        med_wall = statistics.median(walls)
        mean_s = statistics.mean(s_vals)
        mean_w = statistics.mean(w_vals)
        mean_main = statistics.mean(main_vals)
        k = _mode_k(k_vals)
        if k > 0:
            w_per_k = mean_w / k
            delta = w_per_k - mean_s
            residual = med_wall - max(mean_s, w_per_k)
        else:
            w_per_k = None
            delta = None
            residual = None

        binding = _classify_binding(k, mean_s, w_per_k, delta, residual, mean_main)
        raw_cells.append(
            CellLedger(
                corpus=corpus,
                threads=threads,
                arm=arm,
                arm_family=arm_family(arm),
                n=len(samples),
                k=k,
                wall_ms=med_wall,
                s_ms=mean_s,
                w_ms=mean_w,
                main_ms=mean_main,
                w_per_k_ms=w_per_k,
                delta_ms=delta,
                residual_ms=residual,
                binding=binding,
            )
        )

    # Compute W growth between smallest and largest k for each (corpus, arm_family).
    fam_by_k: dict[tuple[str, str], dict[int, list[float]]] = defaultdict(lambda: defaultdict(list))
    for cell in raw_cells:
        if cell.k > 0:
            fam_by_k[(cell.corpus, cell.arm_family)][cell.k].append(cell.w_ms)

    families: list[FamilyGrowth] = []
    fam_map: dict[tuple[str, str], FamilyGrowth] = {}
    for (corpus, fam), by_k in sorted(fam_by_k.items()):
        if len(by_k) < 2:
            continue
        k_min, k_max = min(by_k), max(by_k)
        w_min = statistics.mean(by_k[k_min])
        w_max = statistics.mean(by_k[k_max])
        growth_pct = ((w_max - w_min) / w_min * 100.0) if w_min > 0.0 else 0.0
        flagged = growth_pct > growth_threshold_pct
        fg = FamilyGrowth(
            corpus=corpus,
            arm_family=fam,
            k_min=k_min,
            k_max=k_max,
            w_min_ms=w_min,
            w_max_ms=w_max,
            growth_pct=growth_pct,
            flagged=flagged,
        )
        families.append(fg)
        fam_map[(corpus, fam)] = fg

    cells: list[CellLedger] = []
    for cell in raw_cells:
        fg = fam_map.get((cell.corpus, cell.arm_family))
        if fg is not None and cell.k > 0:
            cells.append(
                CellLedger(
                    **{
                        **asdict(cell),
                        "w_growth_flag": fg.flagged,
                        "w_growth_pct": fg.growth_pct,
                    }
                )
            )
        else:
            cells.append(cell)

    return LedgerReport(
        source=source,
        rows_total=rows_total,
        rows_used=rows_used,
        skipped_empty_thread_cpu=skipped_empty,
        skipped_invalid=skipped_invalid,
        growth_threshold_pct=growth_threshold_pct,
        cells=cells,
        families=families,
    )


def _select_archive_jsonl(names: list[str]) -> list[str]:
    """Select timed JSONL members from a tar archive, skipping parity and duplicate recorded/."""
    jsonl = sorted(n for n in names if n.endswith(".jsonl"))
    if not jsonl:
        return []
    non_parity = [n for n in jsonl if Path(n).name != "parity.jsonl"]
    candidates = non_parity or jsonl

    # Skip `.../recorded/<name>.jsonl` when a shallower `<name>.jsonl` sibling exists.
    by_basename: dict[str, list[str]] = defaultdict(list)
    for n in candidates:
        by_basename[Path(n).name].append(n)
    selected: list[str] = []
    for n in candidates:
        parts = Path(n).parts
        if "recorded" in parts:
            siblings = [s for s in by_basename[Path(n).name] if "recorded" not in Path(s).parts]
            if siblings:
                continue
        selected.append(n)
    return selected


def read_input(
    path: Path,
    *,
    growth_threshold_pct: float = DEFAULT_GROWTH_THRESHOLD_PCT,
    corpora: set[str] | None = None,
    arm_substr: str | None = None,
) -> list[LedgerReport]:
    """Read a `.jsonl` file, directory, or `.tar.gz`/`.tar` bundle into LedgerReports."""
    if path.is_dir():
        jsonl_files = sorted(p for p in path.glob("*.jsonl") if p.name != "parity.jsonl")
        if not jsonl_files:
            jsonl_files = sorted(path.glob("*.jsonl"))
        return [
            analyze_lines(
                p.read_text(encoding="utf-8").splitlines(),
                source=str(p),
                growth_threshold_pct=growth_threshold_pct,
                corpora=corpora,
                arm_substr=arm_substr,
            )
            for p in jsonl_files
        ]

    if tarfile.is_tarfile(path):
        reports: list[LedgerReport] = []
        with tarfile.open(path, "r:*") as tf:
            members = {m.name: m for m in tf.getmembers() if m.isfile()}
            for name in _select_archive_jsonl(list(members)):
                extracted = tf.extractfile(members[name])
                if extracted is None:
                    continue
                text = io.TextIOWrapper(extracted, encoding="utf-8")
                reports.append(
                    analyze_lines(
                        text,
                        source=f"{path}::{name}",
                        growth_threshold_pct=growth_threshold_pct,
                        corpora=corpora,
                        arm_substr=arm_substr,
                    )
                )
        return reports

    with path.open("r", encoding="utf-8") as fh:
        return [
            analyze_lines(
                fh,
                source=str(path),
                growth_threshold_pct=growth_threshold_pct,
                corpora=corpora,
                arm_substr=arm_substr,
            )
        ]


def _fmt_opt(val: float | None, signed: bool = False) -> str:
    if val is None:
        return "n/a"
    return f"{val:+.1f}" if signed else f"{val:.1f}"


def format_report(report: LedgerReport) -> str:
    """Render one LedgerReport as an aligned plain-text table plus W-growth flags."""
    lines = [
        f"=== {report.source} ===",
        (
            f"rows: {report.rows_used} used / {report.rows_total} total "
            f"(skipped empty thread_cpu: {report.skipped_empty_thread_cpu}"
            + (f", invalid: {report.skipped_invalid}" if report.skipped_invalid else "")
            + ")"
        ),
    ]
    if not report.cells:
        lines.append("(no cells)")
        return "\n".join(lines) + "\n"

    arm_labels = [display_arm(c.arm) for c in report.cells]
    corpus_w = max(len("corpus"), max(len(c.corpus) for c in report.cells))
    arm_w = max(len("arm"), max(len(lbl) for lbl in arm_labels))

    header = (
        f"{'corpus':<{corpus_w}}  {'T':>2}  {'arm':<{arm_w}}  {'n':>2}  {'k':>2}  "
        f"{'wall ms':>7}  {'S ms':>6}  {'W ms':>7}  {'W/k ms':>6}  "
        f"{'W/k-S':>7}  {'resid':>6}  {'binding':<8}  flag"
    )
    lines.append(header)
    lines.append("-" * len(header))

    for cell, arm_lbl in zip(report.cells, arm_labels, strict=True):
        flag = (
            f"W_GROWTH({cell.w_growth_pct:+.1f}%)"
            if cell.w_growth_flag and cell.w_growth_pct is not None
            else ""
        )
        lines.append(
            f"{cell.corpus:<{corpus_w}}  {cell.threads:>2d}  {arm_lbl:<{arm_w}}  "
            f"{cell.n:>2d}  {cell.k:>2d}  "
            f"{cell.wall_ms:>7.1f}  {cell.s_ms:>6.1f}  {cell.w_ms:>7.1f}  "
            f"{_fmt_opt(cell.w_per_k_ms):>6}  {_fmt_opt(cell.delta_ms, signed=True):>7}  "
            f"{_fmt_opt(cell.residual_ms, signed=True):>6}  {cell.binding:<8}  {flag}".rstrip()
        )

    flagged = [f for f in report.families if f.flagged]
    if flagged:
        lines.append("")
        lines.append(
            f"W growth > {report.growth_threshold_pct:.1f}% across k "
            "(waiting or contention, not demand):"
        )
        for fg in flagged:
            lines.append(
                f"  FLAG {fg.corpus} / {display_arm(fg.arm_family)}: "
                f"k={fg.k_min}->{fg.k_max}, "
                f"W={fg.w_min_ms:.1f}->{fg.w_max_ms:.1f} ms ({fg.growth_pct:+.1f}%)"
            )
    return "\n".join(lines) + "\n"


def report_to_dict(report: LedgerReport) -> dict[str, Any]:
    return {
        "source": report.source,
        "rows_total": report.rows_total,
        "rows_used": report.rows_used,
        "skipped_empty_thread_cpu": report.skipped_empty_thread_cpu,
        "skipped_invalid": report.skipped_invalid,
        "growth_threshold_pct": report.growth_threshold_pct,
        "cells": [asdict(c) for c in report.cells],
        "families": [asdict(f) for f in report.families],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "paths",
        nargs="+",
        type=Path,
        metavar="BUNDLE_JSONL",
        help="Archived ab JSONL file(s), bundle directory, or .tar.gz/.tar archive(s)",
    )
    parser.add_argument(
        "--corpus",
        action="append",
        default=None,
        help="Filter to one or more corpus names (repeatable)",
    )
    parser.add_argument(
        "--arm",
        default=None,
        help="Filter to arm names containing this substring",
    )
    parser.add_argument(
        "--growth-threshold",
        type=float,
        default=DEFAULT_GROWTH_THRESHOLD_PCT,
        help="Flag arm-family W growth exceeding this percent (default: 10.0)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit structured JSON instead of plain-text tables",
    )
    args = parser.parse_args(argv)
    corpora = set(args.corpus) if args.corpus else None

    reports: list[LedgerReport] = []
    for path in args.paths:
        if not path.exists():
            print(f"error: path not found: {path}", file=sys.stderr)
            return 2
        reports.extend(
            read_input(
                path,
                growth_threshold_pct=args.growth_threshold,
                corpora=corpora,
                arm_substr=args.arm,
            )
        )

    if args.json:
        print(json.dumps([report_to_dict(r) for r in reports], indent=2))
    else:
        for idx, report in enumerate(reports):
            if idx:
                print()
            print(format_report(report), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
