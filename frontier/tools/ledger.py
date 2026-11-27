"""Maintain and validate the shipped-changes ledger (``frontier/ledger.tsv``).

Usage::

    python3 frontier/tools/ledger.py             # list merged software PRs since newest ledger row
    python3 frontier/tools/ledger.py --missing   # exit 0 iff no software PR since last tag is missing
    python3 frontier/tools/ledger.py --check     # validate frontier/ledger.tsv schema and prose rules

Standard library only (Python 3.10+).
"""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parent
DEFAULT_LEDGER = ROOT / "ledger.tsv"

COLUMNS = ("pr", "merged", "effect", "state", "measured", "evidence")
VALID_STATES = ("default", "experimental", "awaiting-review", "prototype", "target")
VALID_STATE_SET = frozenset(VALID_STATES)
SOFTWARE_PATHSPECS = (
    ":(top)src",
    ":(top)python",
    ":(top)Cargo.*",
    ":(top)pyproject.toml",
    ":(top)build.rs",
)

PR_NUM_RE = re.compile(r"\(#(\d+)\)")
PR_REF_RE = re.compile(r"#\d+\b")
MACHINE_RE = re.compile(r"\b(i9|VM|CI|local)\b")


@dataclass(frozen=True)
class LedgerRow:
    """One parsed row of ``frontier/ledger.tsv``."""

    line: int
    pr: int
    merged: str
    effect: str
    state: str
    measured: str
    evidence: str


@dataclass(frozen=True)
class MergedPR:
    """One merged software pull request discovered from git history."""

    pr: int
    sha: str
    merged: str
    subject: str


def _git(repo: pathlib.Path, *args: str) -> tuple[int, str]:
    """Run git in ``repo`` and return ``(returncode, output)``."""
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
    )
    out = proc.stdout.strip()
    if proc.returncode and proc.stderr.strip():
        out = f"{out}\n{proc.stderr.strip()}".strip()
    return proc.returncode, out


def resolve_ref(repo: pathlib.Path, ref: str | None = None) -> str:
    """Resolve the target git ref, defaulting to ``origin/master`` then ``HEAD``."""
    candidates = [ref] if ref else ["origin/master", "HEAD"]
    for cand in candidates:
        rc, _ = _git(repo, "rev-parse", "--verify", f"{cand}^{{commit}}")
        if rc == 0:
            return cand
    raise RuntimeError(f"cannot resolve git ref from {candidates!r}")


def latest_release_tag(repo: pathlib.Path, ref: str) -> str:
    """Return the latest ``v*`` release tag reachable from ``ref``."""
    rc, tag = _git(repo, "describe", "--tags", "--abbrev=0", "--match", "v*", ref)
    if rc or not tag:
        raise RuntimeError(f"cannot find a v* release tag reachable from {ref}: {tag}")
    return tag


def load_ledger(path: pathlib.Path) -> tuple[list[LedgerRow], list[str]]:
    """Parse ``path`` into ledger rows and return ``(rows, errors)``."""
    errors: list[str] = []
    if not path.is_file():
        return [], [f"ledger file not found: {path}"]

    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines:
        return [], [f"ledger file is empty: {path}"]

    header = tuple(lines[0].split("\t"))
    if header != COLUMNS:
        errors.append(f"line 1: expected header {list(COLUMNS)!r}, got {list(header)!r}")

    rows: list[LedgerRow] = []
    for lineno, raw in enumerate(lines[1:], start=2):
        if not raw.strip():
            errors.append(f"line {lineno}: blank line in ledger")
            continue
        parts = raw.split("\t")
        if len(parts) != len(COLUMNS):
            errors.append(
                f"line {lineno}: expected {len(COLUMNS)} tab-separated columns, got {len(parts)}"
            )
            continue
        pr_str, merged, effect, state, measured, evidence = (p.strip() for p in parts)
        if not re.fullmatch(r"\d+", pr_str) or int(pr_str) <= 0:
            errors.append(f"line {lineno}: pr must be a positive integer, got {pr_str!r}")
            continue
        rows.append(
            LedgerRow(
                line=lineno,
                pr=int(pr_str),
                merged=merged,
                effect=effect,
                state=state,
                measured=measured,
                evidence=evidence,
            )
        )
    return rows, errors


def validate_ledger(path: pathlib.Path) -> list[str]:
    """Validate schema, states, plain-language effect rules, and measurement fields."""
    rows, errors = load_ledger(path)
    seen: dict[int, int] = {}

    for row in rows:
        if row.pr in seen:
            errors.append(f"line {row.line}: duplicate PR {row.pr} (first on line {seen[row.pr]})")
        else:
            seen[row.pr] = row.line

        try:
            dt.date.fromisoformat(row.merged)
        except ValueError:
            errors.append(f"line {row.line}: merged must be YYYY-MM-DD, got {row.merged!r}")

        if not row.effect:
            errors.append(f"line {row.line}: effect must not be empty")
        else:
            if "`" in row.effect:
                errors.append(f"line {row.line}: effect must not contain inline code (backticks)")
            if PR_REF_RE.search(row.effect):
                errors.append(f"line {row.line}: effect must not contain #N PR references")

        if row.state not in VALID_STATE_SET:
            errors.append(
                f"line {row.line}: invalid state {row.state!r}; expected one of {VALID_STATES!r}"
            )

        if not row.measured:
            errors.append(f"line {row.line}: measured must not be empty")
        elif row.measured != "not measured" and not MACHINE_RE.search(row.measured):
            errors.append(
                f"line {row.line}: measured must be 'not measured' or name a machine "
                f"(i9, VM, CI, local); got {row.measured!r}"
            )

        if not row.evidence:
            errors.append(f"line {row.line}: evidence must not be empty")

    return errors


def merged_software_prs(
    repo: pathlib.Path,
    since_ref: str,
    target_ref: str,
) -> list[MergedPR]:
    """List first-parent merged software PRs in ``since_ref..target_ref`` oldest-first."""
    rc, _ = _git(repo, "merge-base", "--is-ancestor", since_ref, target_ref)
    if rc != 0:
        raise RuntimeError(
            f"{since_ref} is not an ancestor of {target_ref} "
            "(shallow clone? fetch full history back to the release tag)"
        )

    rc, out = _git(
        repo,
        "log",
        "--first-parent",
        "--reverse",
        "--date=short",
        "--format=%H\t%P\t%ad\t%s",
        f"{since_ref}..{target_ref}",
        "--",
        *SOFTWARE_PATHSPECS,
    )
    if rc != 0:
        raise RuntimeError(f"git log failed: {out}")

    prs: list[MergedPR] = []
    seen: set[int] = set()
    for line in out.splitlines():
        if not line.strip():
            continue
        sha, parents, date_str, subject = line.split("\t", 3)
        if not parents.strip():
            # Never treat a shallow-graft root commit as a diff touching software paths.
            continue
        matches = PR_NUM_RE.findall(subject)
        if not matches:
            continue
        pr_num = int(matches[-1])
        if pr_num in seen:
            continue
        seen.add(pr_num)
        prs.append(MergedPR(pr=pr_num, sha=sha, merged=date_str, subject=subject))
    return prs


def missing_prs(
    repo: pathlib.Path,
    ledger_path: pathlib.Path,
    since_ref: str,
    target_ref: str,
) -> list[MergedPR]:
    """Return merged software PRs in ``since_ref..target_ref`` absent from ``ledger_path``."""
    rows, errors = load_ledger(ledger_path)
    if errors:
        raise ValueError("; ".join(errors))
    recorded = {row.pr for row in rows}
    return [pr for pr in merged_software_prs(repo, since_ref, target_ref) if pr.pr not in recorded]


def since_newest_prs(
    repo: pathlib.Path,
    ledger_path: pathlib.Path,
    since_ref: str,
    target_ref: str,
) -> list[MergedPR]:
    """Return unledgered merged software PRs occurring after the newest ledgered PR."""
    rows, errors = load_ledger(ledger_path)
    if errors:
        raise ValueError("; ".join(errors))
    all_prs = merged_software_prs(repo, since_ref, target_ref)
    if not rows:
        return all_prs
    recorded = {row.pr for row in rows}
    last_idx = -1
    for idx, pr in enumerate(all_prs):
        if pr.pr in recorded:
            last_idx = idx
    return [pr for pr in all_prs[last_idx + 1 :] if pr.pr not in recorded]


def build_parser() -> argparse.ArgumentParser:
    """Create the command-line argument parser."""
    parser = argparse.ArgumentParser(
        description="Inspect and validate frontier/ledger.tsv against merged software PRs."
    )
    parser.add_argument(
        "--missing",
        action="store_true",
        help="Exit 0 iff every merged software PR since --since has a row in the ledger.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate ledger schema, states, plain-language effects, and measurements.",
    )
    parser.add_argument(
        "--since",
        default=None,
        help="Base git ref/tag (default: latest reachable v* tag).",
    )
    parser.add_argument(
        "--ref",
        default=None,
        help="Target git ref (default: origin/master, falling back to HEAD).",
    )
    parser.add_argument(
        "--ledger",
        type=pathlib.Path,
        default=DEFAULT_LEDGER,
        help="Path to ledger TSV (default: frontier/ledger.tsv).",
    )
    parser.add_argument(
        "--repo",
        type=pathlib.Path,
        default=REPO,
        help="Path to git repository root.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Run the ledger CLI and return an exit status code."""
    args = build_parser().parse_args(argv)
    exit_code = 0

    if args.check:
        errors = validate_ledger(args.ledger)
        if errors:
            for err in errors:
                print(f"error: {err}", file=sys.stderr)
            exit_code = 1
        else:
            rows, _ = load_ledger(args.ledger)
            print(f"ok: {len(rows)} ledger rows valid in {args.ledger}")

    if args.missing:
        try:
            ref = resolve_ref(args.repo, args.ref)
            since = args.since or latest_release_tag(args.repo, ref)
            missing = missing_prs(args.repo, args.ledger, since, ref)
        except (RuntimeError, ValueError) as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 1
        if missing:
            print(
                f"missing {len(missing)} merged software PR(s) in {since}..{ref}:",
                file=sys.stderr,
            )
            for pr in missing:
                print(f"  #{pr.pr}\t{pr.merged}\t{pr.sha[:8]}\t{pr.subject}", file=sys.stderr)
            exit_code = 1
        else:
            print(f"ok: no missing merged software PRs in {since}..{ref}")

    if not args.check and not args.missing:
        try:
            ref = resolve_ref(args.repo, args.ref)
            since = args.since or latest_release_tag(args.repo, ref)
            pending = since_newest_prs(args.repo, args.ledger, since, ref)
        except (RuntimeError, ValueError) as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 1
        if not pending:
            print(
                f"up to date: no new merged software PRs since newest ledger row ({since}..{ref})"
            )
            return 0
        print("\t".join(COLUMNS))
        for pr in pending:
            print(
                f"{pr.pr}\t{pr.merged}\t<plain-english effect>\t<state>\tnot measured\tPR #{pr.pr}"
            )

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
