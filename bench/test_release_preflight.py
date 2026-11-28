#!/usr/bin/env python3
"""Frozen regression fixtures for the pre-release physical i9 regression gate (RELEASING §3.5).

Frozen before implementation per frontier/claims/pre-release-i9-gate.toml stop_test.
Covers:
  1. Exact candidate/base and passing verified receipt (pass flows through with no second approval)
  2. Simulated 0.5.17-style primary regression at 8 or 10 threads even with neutral CI and <3% loss
  3. Failed byte parity
  4. Secondary/memory trade outside PERFORMANCE-POLICY.md §2
  5. Missing, unread, invalid, digest-mismatched, wrong-machine, and self-declared PASS bundles
  6. Wrong base, stale source/build inputs, dirty tree, and master advancing (race) at dispatch
  7. Non-performance metadata-only changes (whole release and post-preflight metadata reuse)
  8. Explicit owner-authorised exception in docs/journal/decisions.md vs unauthorised bypass
  9. Shallow git runner handling (safe without full history)
 10. Required §5c recalibration before final candidate comparison
 11. Corrective candidate beating the bad tag but failing recovery against the last known good tag
 12. Rejection occurring before any version bump, tag, push, or publish in deploy.yml and ship.just
"""

from __future__ import annotations

import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT / "bench") not in sys.path:
    sys.path.insert(0, str(REPO_ROOT / "bench"))

import release_preflight as rp  # noqa: E402

I9_LSCPU = """\
Architecture:        x86_64
Model name:          Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz
CPU(s):              20
Thread(s) per core:  2
Core(s) per socket:  10
Socket(s):           1
CPU max MHz:         5300.0000
"""

CI_EPYC_LSCPU = """\
Architecture:        x86_64
Model name:          AMD EPYC 7763 64-Core Processor
CPU(s):              4
Thread(s) per core:  2
Core(s) per socket:  2
Socket(s):           1
"""

I9_ENV_JSON = {
    "cpu": "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz",
    "cpus": 20,
    "cores": 10,
    "threads_per_core": 2,
}

CI_ENV_JSON = {
    "cpu": "AMD EPYC 7763 64-Core Processor",
    "cpus": 4,
    "cores": 2,
    "threads_per_core": 2,
}

DEFAULT_INPUTS_TXT = """\
profile=release
features=default
rustflags=
disallowed_env=
candidate_toolchain=rustc 1.99.1 (abc 2026-11-01) | cargo 1.99.1
base_toolchain=rustc 1.99.1 (abc 2026-11-01) | cargo 1.99.1
"""

CORPUS_META = {
    "wikidata": {
        "hash": "a8830caee5c511a6",
        "out": 209_700_196,
        "base_wall": 0.400,
        "base_rss_kb": 42_000,
    },
    "text": {
        "hash": "b112233445566778",
        "out": 16_000_000,
        "base_wall": 0.120,
        "base_rss_kb": 24_000,
    },
    "records": {
        "hash": "c223344556677889",
        "out": 12_000_000,
        "base_wall": 0.090,
        "base_rss_kb": 20_000,
    },
    "random": {
        "hash": "d334455667788990",
        "out": 10_000_000,
        "base_wall": 0.110,
        "base_rss_kb": 22_000,
    },
    "pattern": {
        "hash": "e445566778899001",
        "out": 14_000_000,
        "base_wall": 0.080,
        "base_rss_kb": 18_000,
    },
    "onebyte": {
        "hash": "f556677889900112",
        "out": 16_000_000,
        "base_wall": 0.060,
        "base_rss_kb": 16_000,
    },
    "runs": {
        "hash": "0667788990011223",
        "out": 14_000_000,
        "base_wall": 0.070,
        "base_rss_kb": 18_000,
    },
}

TEST_GIT_ENV = {
    "GIT_AUTHOR_NAME": "Louis Maddox",
    "GIT_AUTHOR_EMAIL": "louismmx@gmail.com",
    "GIT_COMMITTER_NAME": "Louis Maddox",
    "GIT_COMMITTER_EMAIL": "louismmx@gmail.com",
}


def git(cwd: Path, *args: str, env: dict[str, str] | None = None, check: bool = True) -> str:
    merged = {**os.environ, **TEST_GIT_ENV, **(env or {})}
    proc = subprocess.run(
        ["git", *args],
        cwd=cwd,
        env=merged,
        check=check,
        capture_output=True,
        text=True,
    )
    return proc.stdout.strip()


def make_paired_rows(
    *,
    corpora: tuple[str, ...] = ("wikidata", "text", "records", "random"),
    threads: tuple[int, ...] = rp.DEFAULT_THREADS,
    rounds: int = 10,
    wall_ratios: dict[tuple[str, int], float] | None = None,
    rss_ratios: dict[tuple[str, int], float] | None = None,
    rss_base_kb_override: dict[tuple[str, int], int] | None = None,
    recovery_wall_ratios: dict[tuple[str, int], float] | None = None,
    include_recovery: bool = False,
    parity_break: tuple[str, int, int, str] | None = None,
    error_cell: tuple[str, int, int] | None = None,
    jitter: float = 0.002,
    base_arm: str = "mibz2@/tmp/bin/ab-base",
    recovery_arm: str = "mibz2@/tmp/bin/ab-recovery",
) -> list[dict]:
    """Generate deterministic paired `ab run` JSONL rows with tight controlled variance."""
    wall_ratios = wall_ratios or {}
    rss_ratios = rss_ratios or {}
    rss_base_kb_override = rss_base_kb_override or {}
    recovery_wall_ratios = recovery_wall_ratios or {}
    # Symmetric small per-round perturbations (mean 0) for realistic tight CIs.
    offsets = [
        -1.0,
        1.0,
        -0.7,
        0.7,
        -0.4,
        0.4,
        -0.2,
        0.2,
        -0.5,
        0.5,
        -0.3,
        0.3,
    ]
    rows: list[dict] = []
    for corpus in corpora:
        meta = CORPUS_META[corpus]
        for t in threads:
            ratio = wall_ratios.get((corpus, t), 1.0)
            rss_r = rss_ratios.get((corpus, t), 1.0)
            base_rss = rss_base_kb_override.get((corpus, t), meta["base_rss_kb"])
            base_wall = meta["base_wall"] / max(1.0, min(t, 8) ** 0.5)
            rec_ratio = recovery_wall_ratios.get((corpus, t), 1.0)
            for r in range(rounds):
                eps = offsets[r % len(offsets)] * jitter
                b_wall = round(base_wall * (1.0 + 0.001 * (r % 3 - 1)), 6)
                c_wall = round(b_wall * ratio * (1.0 + eps), 6)
                c_hash = meta["hash"]
                c_out = meta["out"]
                c_err = ""
                c_term = "success"
                if parity_break == (corpus, t, r, "hash"):
                    c_hash = "deadbeef00000000"
                elif parity_break == (corpus, t, r, "out"):
                    c_out = meta["out"] + 1
                if error_cell == (corpus, t, r):
                    c_err = "crc mismatch"
                    c_term = "failed"
                rows.append(
                    {
                        "round": r,
                        "corpus": corpus,
                        "threads": t,
                        "arm": "mibz2",
                        "wall": c_wall,
                        "cpu": round(c_wall * min(t, 10), 6),
                        "rss_kb": int(round(base_rss * rss_r)),
                        "out": c_out,
                        "hash": c_hash,
                        "termination": c_term,
                        "err": c_err,
                    }
                )
                rows.append(
                    {
                        "round": r,
                        "corpus": corpus,
                        "threads": t,
                        "arm": base_arm,
                        "wall": b_wall,
                        "cpu": round(b_wall * min(t, 10), 6),
                        "rss_kb": base_rss,
                        "out": meta["out"],
                        "hash": meta["hash"],
                        "termination": "success",
                        "err": "",
                    }
                )
                if include_recovery:
                    # candidate / recovery = rec_ratio => recovery = candidate / rec_ratio
                    rec_wall = round(c_wall / (rec_ratio * (1.0 + 0.5 * eps)), 6)
                    rows.append(
                        {
                            "round": r,
                            "corpus": corpus,
                            "threads": t,
                            "arm": recovery_arm,
                            "wall": rec_wall,
                            "cpu": round(rec_wall * min(t, 10), 6),
                            "rss_kb": base_rss,
                            "out": meta["out"],
                            "hash": meta["hash"],
                            "termination": "success",
                            "err": "",
                        }
                    )
    return rows


def build_tarball_bytes(
    *,
    stamp: str = "20261128T010000Z",
    rows: list[dict] | None = None,
    candidate_sha: str,
    base_sha: str,
    recovery_base_sha: str | None = None,
    lscpu_txt: str = I9_LSCPU,
    env_json: dict | None = None,
    inputs_txt: str = DEFAULT_INPUTS_TXT,
    extra_files: dict[str, bytes] | None = None,
    omit_files: set[str] | None = None,
    raw_runs_bytes: bytes | None = None,
) -> tuple[bytes, str]:
    """Create a `.tar.gz` evidence bundle in memory and return `(tar_bytes, sha256_hex)`."""
    omit = omit_files or set()
    commits_lines = [f"candidate {candidate_sha}", f"base {base_sha}"]
    if recovery_base_sha:
        commits_lines.append(f"recovery_base {recovery_base_sha}")
    files: dict[str, bytes] = {
        "commits.txt": ("\n".join(commits_lines) + "\n").encode(),
        "lscpu.txt": lscpu_txt.encode(),
        "env.json": (json.dumps(env_json if env_json is not None else I9_ENV_JSON) + "\n").encode(),
        "inputs.txt": inputs_txt.encode(),
        "runs.jsonl": (
            raw_runs_bytes
            if raw_runs_bytes is not None
            else ("".join(json.dumps(r) + "\n" for r in (rows or []))).encode()
        ),
    }
    if extra_files:
        files.update(extra_files)
    for k in omit:
        files.pop(k, None)

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tf:
        for name, data in sorted(files.items()):
            info = tarfile.TarInfo(name=f"{stamp}/{name}")
            info.size = len(data)
            tf.addfile(info, io.BytesIO(data))
    blob = buf.getvalue()
    return blob, hashlib.sha256(blob).hexdigest()


class ScratchRepo:
    """Isolated bare origin + working clone supporting tags, origin/evidence, and shallow clones."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.origin = root / "origin.git"
        self.work = root / "work"
        git(root, "init", "-q", "--bare", "-b", "master", str(self.origin))
        git(root, "clone", "-q", str(self.origin), str(self.work))
        self.write("README.md", "# mibz2 test\n")
        self.write("Cargo.toml", '[package]\nname = "mibz2"\nversion = "0.5.16"\n')
        self.write("pyproject.toml", '[project]\nname = "mibz2"\nversion = "0.5.16"\n')
        self.write("src/open/dense.rs", "// v0.5.16 dense\n")
        self.write("src/parallel.rs", "// v0.5.16 trims\n")
        self.write("docs/journal/decisions.md", "# Decisions\n")
        self.write("frontier/deltas/README.md", "# Deltas\n")
        self.write(
            "frontier/claims/release-0-5-18.toml",
            'id = "release-0-5-18"\nstatus = "open"\nneeds_louis = ""\n',
        )
        self.v0516_sha = self.commit("initial v0.5.16")
        self.tag("v0.5.16", self.v0516_sha)
        self.write("Cargo.toml", '[package]\nname = "mibz2"\nversion = "0.5.17"\n')
        self.write("pyproject.toml", '[project]\nname = "mibz2"\nversion = "0.5.17"\n')
        self.write("src/open/dense.rs", "// v0.5.17 prescan\n")
        self.v0517_sha = self.commit("release v0.5.17")
        self.tag("v0.5.17", self.v0517_sha)

    def write(self, rel: str, content: str | bytes) -> Path:
        path = self.work / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            path.write_bytes(content)
        else:
            path.write_text(content, encoding="utf-8")
        return path

    def commit(self, msg: str, *, push: bool = True) -> str:
        git(self.work, "add", "-A")
        git(self.work, "commit", "-q", "--allow-empty", "-m", msg)
        sha = git(self.work, "rev-parse", "HEAD")
        if push:
            git(self.work, "push", "-q", "origin", "HEAD:master")
        return sha

    def tag(self, name: str, sha: str) -> None:
        git(self.work, "tag", "-a", name, sha, "-m", f"Release {name}")
        git(self.work, "push", "-q", "origin", f"refs/tags/{name}")

    def publish_evidence(
        self,
        rel_path: str,
        tar_bytes: bytes,
        *,
        sidecar_sha: str | None = None,
        omit_sidecar: bool = False,
    ) -> tuple[str, str]:
        """Archive a bundle on origin/evidence (orphan branch, same plumbing as bench/submit.sh)."""
        actual_sha = hashlib.sha256(tar_bytes).hexdigest()
        sidecar_digest = sidecar_sha if sidecar_sha is not None else actual_sha
        stamp_file = Path(rel_path).name
        blob_proc = subprocess.run(
            ["git", "hash-object", "-w", "--stdin"],
            cwd=self.origin,
            input=tar_bytes,
            check=True,
            capture_output=True,
        )
        blob = blob_proc.stdout.decode().strip()

        env = {**os.environ, **TEST_GIT_ENV}
        parent_proc = subprocess.run(
            ["git", "rev-parse", "-q", "--verify", "refs/heads/evidence^{commit}"],
            cwd=self.origin,
            capture_output=True,
            text=True,
        )
        parent = parent_proc.stdout.strip() if parent_proc.returncode == 0 else ""

        idx = self.root / f"idx-{hashlib.sha256(rel_path.encode()).hexdigest()[:8]}"
        env["GIT_INDEX_FILE"] = str(idx)
        if idx.exists():
            idx.unlink()
        if parent:
            subprocess.run(["git", "read-tree", parent], cwd=self.origin, env=env, check=True)
        subprocess.run(
            ["git", "update-index", "--add", "--cacheinfo", f"100644,{blob},{rel_path}"],
            cwd=self.origin,
            env=env,
            check=True,
        )
        if not omit_sidecar:
            sum_bytes = f"{sidecar_digest}  {stamp_file}\n".encode()
            sum_blob = (
                subprocess.run(
                    ["git", "hash-object", "-w", "--stdin"],
                    cwd=self.origin,
                    input=sum_bytes,
                    check=True,
                    capture_output=True,
                )
                .stdout.decode()
                .strip()
            )
            subprocess.run(
                [
                    "git",
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    f"100644,{sum_blob},{rel_path}.sha256",
                ],
                cwd=self.origin,
                env=env,
                check=True,
            )
        tree = subprocess.run(
            ["git", "write-tree"],
            cwd=self.origin,
            env=env,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        idx.unlink(missing_ok=True)
        env.pop("GIT_INDEX_FILE", None)
        cmd = [
            "git",
            "commit-tree",
            tree,
            "-m",
            f"evidence: {rel_path}",
            "-m",
            f"sha256 {actual_sha}",
        ]
        if parent:
            cmd[2:2] = ["-p", parent]
        ev_commit = subprocess.run(
            cmd, cwd=self.origin, env=env, check=True, capture_output=True, text=True
        ).stdout.strip()
        subprocess.run(
            ["git", "update-ref", "refs/heads/evidence", ev_commit],
            cwd=self.origin,
            env=env,
            check=True,
        )
        return ev_commit, actual_sha

    def write_preflight_delta(
        self,
        filename: str,
        *,
        candidate_sha: str,
        base_sha: str,
        ev_commit: str,
        bundle_path: str,
        sha256: str,
        recalibration: str = "completed",
        recalibration_commit: str | None = None,
        recalibration_evidence: str = "origin/evidence recal-1",
        recovery_base: str | None = None,
        recovery_cells: str | None = None,
        claim: str = "release-0-5-18",
        extra_lines: list[str] | None = None,
    ) -> str:
        lines = [
            f"claim: {claim}",
            "by: antigravity",
            "outcome: partial",
            f"evidence: origin/evidence {ev_commit[:8]} ({bundle_path}, sha256 {sha256[:12]})",
            "",
            "- **Preflight receipt:**",
            f"  - `preflight_candidate`: `{candidate_sha}`",
            f"  - `preflight_base`: `{base_sha}`",
            f"  - `preflight_evidence_commit`: `{ev_commit}`",
            f"  - `preflight_bundle`: `{bundle_path}`",
            f"  - `preflight_sha256`: `{sha256}`",
            f"  - `preflight_recalibration`: `{recalibration}`",
        ]
        if recalibration_commit is not None:
            lines.append(f"  - `preflight_recalibration_commit`: `{recalibration_commit}`")
        if recalibration_evidence:
            lines.append(f"  - `preflight_recalibration_evidence`: `{recalibration_evidence}`")
        if recovery_base is not None:
            lines.append(f"  - `preflight_recovery_base`: `{recovery_base}`")
        if recovery_cells is not None:
            lines.append(f"  - `preflight_recovery_cells`: `{recovery_cells}`")
        if extra_lines:
            lines.extend(extra_lines)
        self.write(f"frontier/deltas/{filename}", "\n".join(lines) + "\n")
        return self.commit(f"delta: record preflight receipt {filename}")

    def record_0517_known_regression(self) -> str:
        """Record a 0.5.17 release-scaling folder showing the 8T (+2.6%) and 10T (+1.5%) losses vs 0.5.16."""
        folder = "bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.17"
        rows = make_paired_rows(
            corpora=("wikidata",),
            threads=rp.DEFAULT_THREADS,
            rounds=10,
            wall_ratios={("wikidata", 8): 1.026, ("wikidata", 10): 1.015},
            base_arm="mibz2@/tmp/build-v0.5.16/ab",
        )
        meta = {
            "date": "2026-11-27",
            "cpu": "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz",
            "cpus": 20,
            "cores": 10,
            "ref": "mibz2",
            "arms": {
                "mibz2": "0.5.17",
                "mibz2@/tmp/build-v0.5.16/ab": "0.5.16",
            },
            "rounds": 10,
            "threads": list(rp.DEFAULT_THREADS),
        }
        self.write(f"{folder}/meta.json", json.dumps(meta, indent=2) + "\n")
        self.write(f"{folder}/runs.jsonl", "".join(json.dumps(r) + "\n" for r in rows))
        return self.commit("bench: record 0.5.17 release scaling regression")


class ReleasePreflightFrozenFixtures(unittest.TestCase):
    """Frozen fixture matrix implementing every case of pre-release-i9-gate's stop_test."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="mibz2-preflight-test-"))
        self.repo = ScratchRepo(self.tmp)

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    # ------------------------------------------------------------------
    # 1. Exact candidate/base + passing verified receipt (no second approval)
    # ------------------------------------------------------------------
    def test_01_passing_verified_receipt_allows_release_without_second_approval(self) -> None:
        self.repo.write("src/open/dense.rs", "// fixed open dense\n")
        cand = self.repo.commit("perf(open): remove dense prescan")
        rows = make_paired_rows(wall_ratios={("wikidata", 8): 0.975, ("wikidata", 10): 0.985})
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010000Z.tar.gz", tar_bytes
        )
        head = self.repo.write_preflight_delta(
            "2026-11-28-release-0-5-18-preflight.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T010000Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
        )
        res = rp.verify_release(self.repo.work, expected_head=head)
        self.assertTrue(res["allowed"], res)
        self.assertEqual(res["mode"], "verified-i9-pass")
        self.assertEqual(res["base_tag"], "v0.5.17")
        self.assertEqual(res["base_sha"], self.repo.v0517_sha)
        self.assertEqual(res["candidate_sha"], cand)

    def test_01b_harness_and_queue_churn_after_receipt_keeps_it_fresh(self) -> None:
        # Added at integration review (not one of the frozen fixtures): queueing the preflight
        # itself edits bench/workstation-queue.txt after the candidate is pinned, so harness,
        # and test paths must not stale a receipt; a build input (the release recipes included)
        # still must.
        self.repo.write("src/open/dense.rs", "// prescan removed\n")
        cand = self.repo.commit("perf(open): remove dense prescan")
        rows = make_paired_rows(wall_ratios={})
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010001Z.tar.gz", tar_bytes
        )
        for rel in (
            "bench/workstation-queue.txt",
            "tests/extra.rs",
            "examples/ab.rs",
        ):
            self.repo.write(rel, "# churn\n")
        self.repo.commit("bench: queue the preflight; harness and test churn")
        head = self.repo.write_preflight_delta(
            "2026-11-28-release-0-5-18-preflight-churn.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T010001Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
        )
        res = rp.verify_release(self.repo.work, expected_head=head)
        self.assertTrue(res["allowed"], res)
        self.assertEqual(res["mode"], "verified-i9-pass")

        # Each path must itself be named as the staling input (the commits accumulate, so a bare
        # "stale" would hold vacuously after the first).
        for rel in (
            ".cargo/config.toml",
            "uv.lock",
            ".github/workflows/CI.yml",
            ".just/ship.just",
            "Justfile",
        ):
            with self.subTest(rel=rel):
                self.repo.write(rel, f"# build input change {rel}\n")
                head = self.repo.commit(f"build: change {rel}")
                res = rp.verify_release(self.repo.work, expected_head=head)
                self.assertFalse(res["allowed"], res)
                self.assertTrue(
                    any("stale" in p and rel in p for p in res["problems"]), res["problems"]
                )

    # ------------------------------------------------------------------
    # 2. Simulated 0.5.17-style primary regression at 8T / 10T (<3%, neutral CI)
    # ------------------------------------------------------------------
    def test_02_simulated_0517_primary_regression_8t_under_3pct_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// regressed at 8T\n")
        cand = self.repo.commit("perf(open): extra prescan")
        # 1T/2T/4T completely neutral (simulating neutral 4-CPU CI), 8T +2.6% (<3% attribution bar)
        rows = make_paired_rows(wall_ratios={("wikidata", 8): 1.026})
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010100Z.tar.gz", tar_bytes
        )
        self.repo.write_preflight_delta(
            "2026-11-28-preflight-8t-reg.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T010100Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
        )
        res = rp.verify_release(self.repo.work)
        self.assertFalse(res["allowed"])
        self.assertEqual(res["mode"], "rejected")
        self.assertTrue(any("wikidata" in p and "8" in p for p in res["problems"]), res["problems"])

    def test_02b_simulated_0517_primary_regression_10t_under_3pct_rejected(self) -> None:
        rows = make_paired_rows(wall_ratios={("wikidata", 10): 1.015})
        verdict = rp.judge_runs(rows)
        self.assertTrue(verdict["valid"])
        self.assertFalse(verdict["pass"])
        self.assertTrue(any("wikidata" in m and "10" in m for m in verdict["regressions"]), verdict)

    def test_02c_primary_text_regression_rejected(self) -> None:
        rows = make_paired_rows(wall_ratios={("text", 8): 1.018})
        verdict = rp.judge_runs(rows)
        self.assertTrue(verdict["valid"])
        self.assertFalse(verdict["pass"])
        self.assertTrue(any("text" in m and "8" in m for m in verdict["regressions"]), verdict)

    # ------------------------------------------------------------------
    # 3. Failed byte parity
    # ------------------------------------------------------------------
    def test_03_failed_byte_parity_hash_and_length_rejected(self) -> None:
        rows_hash = make_paired_rows(parity_break=("wikidata", 8, 3, "hash"))
        v_hash = rp.judge_runs(rows_hash)
        self.assertFalse(v_hash["pass"])
        self.assertFalse(v_hash["parity_ok"])

        rows_out = make_paired_rows(parity_break=("text", 4, 0, "out"))
        v_out = rp.judge_runs(rows_out)
        self.assertFalse(v_out["pass"])
        self.assertFalse(v_out["parity_ok"])

        rows_err = make_paired_rows(error_cell=("wikidata", 1, 0))
        v_err = rp.judge_runs(rows_err)
        self.assertFalse(v_err["pass"])
        self.assertFalse(v_err["valid"])

    # ------------------------------------------------------------------
    # 4. Secondary wall-time and peak RSS memory policy (PERFORMANCE-POLICY §2)
    # ------------------------------------------------------------------
    def test_04_secondary_wall_policy_bounds(self) -> None:
        # <= 5% secondary wall loss is allowed
        ok_small = rp.judge_runs(make_paired_rows(wall_ratios={("random", 8): 1.045}))
        self.assertTrue(ok_small["pass"], ok_small)

        # > 5%..10% secondary wall loss WITHOUT 2x primary gain at that thread count -> REJECT
        bad_mid = rp.judge_runs(
            make_paired_rows(wall_ratios={("records", 8): 1.07, ("wikidata", 8): 0.98})
        )
        self.assertFalse(bad_mid["pass"])
        self.assertTrue(bad_mid["secondary_failures"], bad_mid)

        # > 5%..10% secondary wall loss WITH >= 2x primary gain beyond noise at same thread count -> ALLOW
        ok_trade = rp.judge_runs(
            make_paired_rows(wall_ratios={("records", 8): 1.06, ("wikidata", 8): 0.87})
        )
        self.assertTrue(ok_trade["pass"], ok_trade)

        # > 10% secondary wall loss -> REJECT even with big primary gain
        bad_large = rp.judge_runs(
            make_paired_rows(wall_ratios={("random", 8): 1.11, ("wikidata", 8): 0.70})
        )
        self.assertFalse(bad_large["pass"])
        self.assertTrue(bad_large["secondary_failures"], bad_large)

    def test_04b_memory_rss_policy_bounds(self) -> None:
        # RSS > 10% without qualifying speed gain -> REJECT
        bad_no_gain = rp.judge_runs(make_paired_rows(rss_ratios={("wikidata", 8): 1.15}))
        self.assertFalse(bad_no_gain["pass"])
        self.assertTrue(bad_no_gain["memory_failures"], bad_no_gain)

        # Valid WideLazyCells-style trade: -16% wall, +14% RSS (<= 2*16%), +25 MB (<= 64 MB), single corpus
        ok_trade = rp.judge_runs(
            make_paired_rows(
                wall_ratios={("wikidata", 8): 0.84},
                rss_ratios={("wikidata", 8): 1.14},
                rss_base_kb_override={("wikidata", 8): 180_000},  # +14% = +24.6 MB <= 64 MB
            )
        )
        self.assertTrue(ok_trade["pass"], ok_trade)

        # RSS > 10% with >=5% gain and <=2x ratio, BUT absolute delta > 64 MB -> REJECT
        bad_abs_mb = rp.judge_runs(
            make_paired_rows(
                wall_ratios={("wikidata", 8): 0.80},
                rss_ratios={("wikidata", 8): 1.20},
                rss_base_kb_override={("wikidata", 8): 400_000},  # +20% = +78.1 MB > 64 MB
            )
        )
        self.assertFalse(bad_abs_mb["pass"])
        self.assertTrue(bad_abs_mb["memory_failures"], bad_abs_mb)

        # RSS > 10% on two corpora simultaneously violates "no other corpus grows > 10%" -> REJECT
        bad_two_corpora = rp.judge_runs(
            make_paired_rows(
                wall_ratios={("wikidata", 8): 0.84, ("text", 8): 0.84},
                rss_ratios={("wikidata", 8): 1.14, ("text", 8): 1.14},
            )
        )
        self.assertFalse(bad_two_corpora["pass"])
        self.assertTrue(bad_two_corpora["memory_failures"], bad_two_corpora)

        # RSS > 10% on a secondary corpus -> REJECT
        bad_secondary_rss = rp.judge_runs(make_paired_rows(rss_ratios={("random", 8): 1.12}))
        self.assertFalse(bad_secondary_rss["pass"])
        self.assertTrue(bad_secondary_rss["memory_failures"], bad_secondary_rss)

    # ------------------------------------------------------------------
    # 5. Missing / unread / invalid / digest-mismatched / wrong-machine / self-declared PASS
    # ------------------------------------------------------------------
    def test_05a_missing_and_unread_bundle_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// change without receipt\n")
        cand = self.repo.commit("perf(open): change")

        # Missing bundle & receipt
        res_missing = rp.verify_release(self.repo.work)
        self.assertFalse(res_missing["allowed"])

        # Passing bundle archived on origin/evidence, BUT unread (no delta receipt recorded!)
        rows = make_paired_rows()
        tar_bytes, _ = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        self.repo.publish_evidence("release-preflight/20261128T010500Z.tar.gz", tar_bytes)
        res_unread = rp.verify_release(self.repo.work)
        self.assertFalse(res_unread["allowed"])
        self.assertTrue(
            any("receipt" in p.lower() or "unread" in p.lower() for p in res_unread["problems"]),
            res_unread,
        )

    def test_05b_digest_mismatch_and_corrupt_bundle_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// digest mismatch test\n")
        cand = self.repo.commit("perf(open): digest test")
        rows = make_paired_rows()
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )

        # Sidecar digest mismatch on origin/evidence
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010600Z.tar.gz",
            tar_bytes,
            sidecar_sha="0" * 64,
        )
        self.repo.write_preflight_delta(
            "2026-11-28-digest-bad.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T010600Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
        )
        res = rp.verify_release(self.repo.work)
        self.assertFalse(res["allowed"])
        self.assertTrue(
            any("sha256" in p.lower() or "digest" in p.lower() for p in res["problems"]), res
        )

        # Corrupt / non-gzip bundle
        corrupt_path = self.tmp / "corrupt.tar.gz"
        corrupt_path.write_bytes(b"not a gzip tarball")
        v_corrupt = rp.judge_bundle(corrupt_path)
        self.assertFalse(v_corrupt["pass"])
        self.assertFalse(v_corrupt["valid"])

    def test_05c_wrong_machine_and_invalid_config_and_fake_pass_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// wrong machine test\n")
        cand = self.repo.commit("perf(open): wrong machine")
        rows = make_paired_rows()
        tar_bytes, digest = build_tarball_bytes(
            rows=rows,
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            lscpu_txt=CI_EPYC_LSCPU,
            env_json=CI_ENV_JSON,
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010700Z.tar.gz", tar_bytes
        )
        # Even if delta prose claims "PASS", gate MUST reject wrong machine!
        self.repo.write_preflight_delta(
            "2026-11-28-wrong-machine.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T010700Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
            extra_lines=["- Verdict: PASS (4-CPU CI runner)"],
        )
        res = rp.verify_release(self.repo.work)
        self.assertFalse(res["allowed"])
        self.assertTrue(
            any("i9-10900k" in p.lower() or "machine" in p.lower() for p in res["problems"]), res
        )

        # Disallowed RUSTFLAGS override in inputs.txt -> REJECT
        bad_inputs_tar, bad_inputs_sha = build_tarball_bytes(
            rows=rows,
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            inputs_txt=DEFAULT_INPUTS_TXT.replace(
                "rustflags=\n", "rustflags=-C target-cpu=native\n"
            ),
        )
        p = self.tmp / "bad-inputs.tar.gz"
        p.write_bytes(bad_inputs_tar)
        v_inputs = rp.judge_bundle(
            p,
            expected_candidate=cand,
            expected_base=self.repo.v0517_sha,
            expected_sha256=bad_inputs_sha,
        )
        self.assertFalse(v_inputs["pass"])
        self.assertTrue(
            any("rustflags" in m.lower() or "build" in m.lower() for m in v_inputs["problems"]),
            v_inputs,
        )

        # Missing primary corpus (`text`) or missing 8T/10T thread counts -> REJECT
        rows_no_text = make_paired_rows(corpora=("wikidata",))
        self.assertFalse(rp.judge_runs(rows_no_text)["pass"])
        rows_ci_threads = make_paired_rows(threads=(1, 2, 4))
        self.assertFalse(rp.judge_runs(rows_ci_threads)["pass"])

    # ------------------------------------------------------------------
    # 6. Wrong base, stale source/build inputs, dirty tree, master race at dispatch
    # ------------------------------------------------------------------
    def test_06_wrong_base_stale_inputs_and_dispatch_race_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// change 1\n")
        cand1 = self.repo.commit("perf(open): change 1")
        rows = make_paired_rows()

        # 6a. Wrong base (v0.5.16 instead of previous published tag v0.5.17)
        tar_wrong_base, sha_wb = build_tarball_bytes(
            rows=rows, candidate_sha=cand1, base_sha=self.repo.v0516_sha
        )
        ev_wb, _ = self.repo.publish_evidence(
            "release-preflight/20261128T010800Z.tar.gz", tar_wrong_base
        )
        self.repo.write_preflight_delta(
            "2026-11-28-wrong-base.md",
            candidate_sha=cand1,
            base_sha=self.repo.v0516_sha,
            ev_commit=ev_wb,
            bundle_path="release-preflight/20261128T010800Z.tar.gz",
            sha256=sha_wb,
            recalibration="completed",
            recalibration_commit=cand1,
        )
        res_wb = rp.verify_release(self.repo.work)
        self.assertFalse(res_wb["allowed"], res_wb)

        # 6b. Passing receipt for cand1 vs v0.5.17, then stale source input changes on HEAD!
        tar_ok, sha_ok = build_tarball_bytes(
            rows=rows, candidate_sha=cand1, base_sha=self.repo.v0517_sha
        )
        ev_ok, _ = self.repo.publish_evidence("release-preflight/20261128T010900Z.tar.gz", tar_ok)
        good_head = self.repo.write_preflight_delta(
            "2026-11-28-good-for-cand1.md",
            candidate_sha=cand1,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_ok,
            bundle_path="release-preflight/20261128T010900Z.tar.gz",
            sha256=sha_ok,
            recalibration="completed",
            recalibration_commit=cand1,
        )
        self.assertTrue(rp.verify_release(self.repo.work, expected_head=good_head)["allowed"])

        # Now land a new performance-relevant commit on src/
        self.repo.write("src/open/dense.rs", "// extra unmeasured edit\n")
        self.repo.commit("perf(open): late edit after preflight")
        res_stale = rp.verify_release(self.repo.work)
        self.assertFalse(res_stale["allowed"])
        self.assertTrue(
            any("stale" in p.lower() or "changed" in p.lower() for p in res_stale["problems"]),
            res_stale,
        )

        # 6c. Race at dispatch: origin/master advances past local HEAD (or expected_head)
        # Rewind local worktree 1 commit behind origin/master
        git(self.repo.work, "reset", "--hard", good_head)
        res_race = rp.verify_release(self.repo.work, expected_head=good_head)
        self.assertFalse(res_race["allowed"])
        self.assertTrue(
            any(
                "master" in p.lower() or "race" in p.lower() or "advanced" in p.lower()
                for p in res_race["problems"]
            ),
            res_race,
        )

    # ------------------------------------------------------------------
    # 7. Non-performance metadata-only changes (whole release & post-preflight reuse)
    # ------------------------------------------------------------------
    def test_07_non_performance_metadata_only_allowed_and_mixed_rejected(self) -> None:
        # 7a. Whole release since v0.5.17 touches ONLY docs/ and CHANGELOG.md -> ALLOW
        self.repo.write("docs/notes.md", "documentation update\n")
        self.repo.write("CHANGELOG.md", "## Unreleased\n- docs\n")
        head_docs = self.repo.commit("docs: update notes and changelog")
        res_docs = rp.verify_release(self.repo.work, expected_head=head_docs)
        self.assertTrue(res_docs["allowed"], res_docs)
        self.assertEqual(res_docs["mode"], "non-performance-metadata")

        # 7b. Mixed change (Cargo.toml + docs/) -> REJECT without i9 evidence
        self.repo.write(
            "Cargo.toml", '[package]\nname = "mibz2"\nversion = "0.5.17"\n# build flag\n'
        )
        self.repo.commit("chore: touch Cargo.toml")
        res_mixed = rp.verify_release(self.repo.work)
        self.assertFalse(res_mixed["allowed"])

    # ------------------------------------------------------------------
    # 8. Explicit owner-authorised exception vs unauthorised bypass
    # ------------------------------------------------------------------
    def test_08_owner_exception_allowed_and_unauthorised_bypass_rejected(self) -> None:
        self.repo.write("src/open/dense.rs", "// urgent fix\n")
        cand = self.repo.commit("fix(open): urgent hotfix")

        # 8a. Unauthorised env switches MUST NOT bypass the gate
        for env_key in (
            "SKIP_RELEASE_PREFLIGHT",
            "BYPASS_PREFLIGHT",
            "RELEASE_PREFLIGHT_PASS",
            "ALLOW_UNVERIFIED_RELEASE",
        ):
            old = os.environ.get(env_key)
            os.environ[env_key] = "PASS"
            try:
                res = rp.verify_release(self.repo.work)
                self.assertFalse(res["allowed"], f"{env_key} bypassed gate!")
            finally:
                if old is None:
                    os.environ.pop(env_key, None)
                else:
                    os.environ[env_key] = old

        # 8b. Uncommitted local edit to docs/journal/decisions.md MUST NOT bypass
        waiver_entry = (
            "\n## 2026-11-28: emergency release preflight waiver\n\n"
            "- **Source:** Louis (repository owner), directly in conversation.\n"
            "- **Observed:** 2026-11-28T01:30:00Z.\n"
            "- **Applies:** RELEASING §3.5, pre-release-i9-gate.\n"
            f"- **Waiver:** `release-preflight-waiver: {cand} {self.repo.v0517_sha}`\n\n"
            "> Ship this hotfix now without waiting for the i9 preflight.\n"
        )
        self.repo.write("docs/journal/decisions.md", "# Decisions\n" + waiver_entry)
        res_uncommitted = rp.verify_release(self.repo.work)
        self.assertFalse(res_uncommitted["allowed"], "uncommitted decisions.md bypassed gate!")

        # 8c. Committed entry authored by an agent (no owner source / no verbatim quote) -> REJECT
        self.repo.write(
            "docs/journal/decisions.md",
            "# Decisions\n\n## 2026-11-28: agent self-waiver\n\n"
            "- **Source:** agent session.\n"
            "- **Observed:** 2026-11-28T01:30:00Z.\n"
            "- **Applies:** RELEASING §3.5.\n"
            f"- **Waiver:** `release-preflight-waiver: {cand} {self.repo.v0517_sha}`\n",
        )
        self.repo.commit("docs: invalid agent waiver")
        self.assertFalse(rp.verify_release(self.repo.work)["allowed"])

        # 8d. Committed entry with owner provenance bound to WRONG candidate SHA -> REJECT
        wrong_sha = "a" * 40
        self.repo.write(
            "docs/journal/decisions.md",
            "# Decisions\n\n## 2026-11-28: wrong sha waiver\n\n"
            "- **Source:** Louis (repository owner).\n"
            "- **Observed:** 2026-11-28T01:30:00Z.\n"
            "- **Applies:** RELEASING §3.5.\n"
            f"- **Waiver:** `release-preflight-waiver: {wrong_sha} {self.repo.v0517_sha}`\n\n"
            "> Waive preflight for that earlier commit.\n",
        )
        self.repo.commit("docs: wrong sha waiver")
        self.assertFalse(rp.verify_release(self.repo.work)["allowed"])

        # 8e. Properly committed owner-authorised exception bound to exact HEAD + base SHAs -> ALLOW
        # Note: committing decisions.md creates a new HEAD, so bind to candidate_sha (performance commit)
        # OR exact HEAD. We bind to the exact performance tree/commit OR exact HEAD: here we verify
        # binding to the exact candidate commit SHA `cand` when HEAD only adds docs/journal/decisions.md!
        self.repo.write("docs/journal/decisions.md", "# Decisions\n" + waiver_entry)
        head_waived = self.repo.commit("docs: record owner release-preflight waiver")
        res_owner = rp.verify_release(self.repo.work, expected_head=head_waived)
        self.assertTrue(res_owner["allowed"], res_owner)
        self.assertEqual(res_owner["mode"], "owner-exception")

        # 8f. Live release claim with unresolved needs_louis blocks release even with waiver
        self.repo.write(
            "frontier/claims/release-0-5-18.toml",
            'id = "release-0-5-18"\nstatus = "open"\nneeds_louis = "confirm emergency hotfix"\n',
        )
        self.repo.commit("frontier: block release on needs_louis")
        res_needs = rp.verify_release(self.repo.work)
        self.assertFalse(res_needs["allowed"])
        self.assertTrue(any("needs_louis" in p for p in res_needs["problems"]), res_needs)

    # ------------------------------------------------------------------
    # 9. Shallow git runner handling (safe without full history)
    # ------------------------------------------------------------------
    def test_09_shallow_git_runner_handles_verify_and_materialize(self) -> None:
        self.repo.write("src/open/dense.rs", "// shallow test change\n")
        cand = self.repo.commit("perf(open): shallow test")
        rows = make_paired_rows()
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T011000Z.tar.gz", tar_bytes
        )
        head = self.repo.write_preflight_delta(
            "2026-11-28-shallow-pass.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T011000Z.tar.gz",
            sha256=digest,
            recalibration="completed",
            recalibration_commit=cand,
        )

        shallow = self.tmp / "shallow-clone"
        git(self.tmp, "clone", "-q", "--depth=1", f"file://{self.repo.origin}", str(shallow))
        self.assertEqual(git(shallow, "rev-parse", "--is-shallow-repository"), "true")

        # Verify succeeds cleanly on the shallow clone
        res = rp.verify_release(shallow, expected_head=head)
        self.assertTrue(res["allowed"], res)
        self.assertEqual(res["mode"], "verified-i9-pass")

        # Shallow-safe source materialisation fetches exact candidate & base SHAs into isolated dirs
        mat_tmp = self.tmp / "mat"
        mat_tmp.mkdir()
        dirs = rp.materialize_sources(
            mat_tmp, shallow, {"candidate": cand, "base": self.repo.v0517_sha}
        )
        self.assertEqual(git(dirs["candidate"], "rev-parse", "HEAD"), cand)
        self.assertEqual(git(dirs["base"], "rev-parse", "HEAD"), self.repo.v0517_sha)
        self.assertIn("shallow test change", (dirs["candidate"] / "src/open/dense.rs").read_text())
        self.assertIn("v0.5.17 prescan", (dirs["base"] / "src/open/dense.rs").read_text())

    # ------------------------------------------------------------------
    # 10. Required §5c recalibration before final candidate comparison
    # ------------------------------------------------------------------
    def test_10_recalibration_5c_required_and_ordered_before_final_comparison(self) -> None:
        self.repo.write("src/open/dense.rs", "// worker cost model moved\n")
        cand = self.repo.commit("perf(open): change worker cost")
        rows = make_paired_rows()
        tar_bytes, digest = build_tarball_bytes(
            rows=rows, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_commit, _ = self.repo.publish_evidence(
            "release-preflight/20261128T011100Z.tar.gz", tar_bytes
        )

        # 10a. Unrecorded / pending recalibration -> REJECT
        self.repo.write_preflight_delta(
            "2026-11-28-recal-pending.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T011100Z.tar.gz",
            sha256=digest,
            recalibration="pending",
        )
        res_pending = rp.verify_release(self.repo.work)
        self.assertFalse(res_pending["allowed"])
        self.assertTrue(
            any("5c" in p.lower() or "recalibration" in p.lower() for p in res_pending["problems"]),
            res_pending,
        )

        # 10b. Waiving §5c citing unchanged route census or prior crossover -> REJECT
        self.repo.write_preflight_delta(
            "2026-11-28-recal-waived-census.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_commit,
            bundle_path="release-preflight/20261128T011100Z.tar.gz",
            sha256=digest,
            recalibration="not-applicable (unchanged route census; 0.5.16 crossover held)",
        )
        res_waived = rp.verify_release(self.repo.work)
        self.assertFalse(res_waived["allowed"], res_waived)

        # 10c. Stale recalibration: recalibrated at cand, then worker-cost kernel changed again!
        self.repo.write("src/open/dense.rs", "// second worker cost edit after recalibration\n")
        cand2 = self.repo.commit("perf(open): second worker cost edit")
        tar2, sha2 = build_tarball_bytes(
            rows=rows, candidate_sha=cand2, base_sha=self.repo.v0517_sha
        )
        ev2, _ = self.repo.publish_evidence("release-preflight/20261128T011200Z.tar.gz", tar2)
        self.repo.write_preflight_delta(
            "2026-11-28-recal-stale.md",
            candidate_sha=cand2,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev2,
            bundle_path="release-preflight/20261128T011200Z.tar.gz",
            sha256=sha2,
            recalibration="completed",
            recalibration_commit=cand,  # stale: src/open/dense.rs changed between cand and cand2!
        )
        res_stale_recal = rp.verify_release(self.repo.work)
        self.assertFalse(res_stale_recal["allowed"], res_stale_recal)

        # 10d. Valid recalibration at cand2, followed by trim retune in src/parallel.rs (cand3),
        # then final preflight measured on cand3 -> ALLOW!
        self.repo.write("src/parallel.rs", "// retuned trims from 5c sweep\n")
        cand3 = self.repo.commit("perf(parallel): retune trims after 5c sweep")
        tar3, sha3 = build_tarball_bytes(
            rows=rows, candidate_sha=cand3, base_sha=self.repo.v0517_sha
        )
        ev3, _ = self.repo.publish_evidence("release-preflight/20261128T011300Z.tar.gz", tar3)
        head3 = self.repo.write_preflight_delta(
            "2026-11-28-recal-valid.md",
            candidate_sha=cand3,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev3,
            bundle_path="release-preflight/20261128T011300Z.tar.gz",
            sha256=sha3,
            recalibration="completed",
            recalibration_commit=cand2,
        )
        res_valid = rp.verify_release(self.repo.work, expected_head=head3)
        self.assertTrue(res_valid["allowed"], res_valid)

    # ------------------------------------------------------------------
    # 11. Corrective candidate beats bad tag (v0.5.17) but fails recovery vs last known good (v0.5.16)
    # ------------------------------------------------------------------
    def test_11_corrective_candidate_must_recover_against_last_known_good_tag(self) -> None:
        self.repo.record_0517_known_regression()
        detected = rp.detect_known_regression(self.repo.work, "v0.5.17", self.repo.v0517_sha)
        self.assertIsNotNone(detected)
        self.assertEqual(detected["recovery_tag"], "v0.5.16")
        self.assertEqual(detected["recovery_base"], self.repo.v0516_sha)
        self.assertIn(("wikidata", 8), detected["affected_cells"])
        self.assertIn(("wikidata", 10), detected["affected_cells"])

        self.repo.write("src/open/dense.rs", "// partial fix for 0.5.17\n")
        cand = self.repo.commit("fix(open): partial prescan fix")

        # 11a. Candidate omits v0.5.16 recovery arm -> REJECT
        rows_no_rec = make_paired_rows(
            wall_ratios={("wikidata", 8): 0.985, ("wikidata", 10): 0.990}
        )
        tar_no_rec, sha_no_rec = build_tarball_bytes(
            rows=rows_no_rec, candidate_sha=cand, base_sha=self.repo.v0517_sha
        )
        ev_no_rec, _ = self.repo.publish_evidence(
            "release-preflight/20261128T011400Z.tar.gz", tar_no_rec
        )
        self.repo.write_preflight_delta(
            "2026-11-28-corrective-no-rec.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_no_rec,
            bundle_path="release-preflight/20261128T011400Z.tar.gz",
            sha256=sha_no_rec,
            recalibration="completed",
            recalibration_commit=cand,
        )
        res_no_rec = rp.verify_release(self.repo.work)
        self.assertFalse(res_no_rec["allowed"])
        self.assertTrue(any("recovery" in p.lower() for p in res_no_rec["problems"]), res_no_rec)

        # 11b. Candidate beats v0.5.17 (0.985 at 8T, 0.990 at 10T) BUT STILL loses to v0.5.16 (1.012 at 8T) -> REJECT
        rows_partial = make_paired_rows(
            wall_ratios={("wikidata", 8): 0.985, ("wikidata", 10): 0.990},
            include_recovery=True,
            recovery_wall_ratios={("wikidata", 8): 1.012, ("wikidata", 10): 1.000},
        )
        tar_partial, sha_partial = build_tarball_bytes(
            rows=rows_partial,
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            recovery_base_sha=self.repo.v0516_sha,
        )
        ev_partial, _ = self.repo.publish_evidence(
            "release-preflight/20261128T011500Z.tar.gz", tar_partial
        )
        self.repo.write_preflight_delta(
            "2026-11-28-corrective-partial.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_partial,
            bundle_path="release-preflight/20261128T011500Z.tar.gz",
            sha256=sha_partial,
            recalibration="completed",
            recalibration_commit=cand,
            recovery_base=self.repo.v0516_sha,
            recovery_cells="wikidata:8,wikidata:10",
        )
        res_partial = rp.verify_release(self.repo.work)
        self.assertFalse(res_partial["allowed"])
        self.assertTrue(
            any("recovery" in p.lower() and "8" in p for p in res_partial["problems"]), res_partial
        )

        # 11c. Full recovery against v0.5.16 AND beats v0.5.17 -> ALLOW
        rows_recovered = make_paired_rows(
            wall_ratios={("wikidata", 8): 0.974, ("wikidata", 10): 0.985},
            include_recovery=True,
            recovery_wall_ratios={("wikidata", 8): 1.000, ("wikidata", 10): 0.999},
        )
        tar_rec, sha_rec = build_tarball_bytes(
            rows=rows_recovered,
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            recovery_base_sha=self.repo.v0516_sha,
        )
        ev_rec, _ = self.repo.publish_evidence("release-preflight/20261128T011600Z.tar.gz", tar_rec)
        head_rec = self.repo.write_preflight_delta(
            "2026-11-28-corrective-recovered.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev_rec,
            bundle_path="release-preflight/20261128T011600Z.tar.gz",
            sha256=sha_rec,
            recalibration="completed",
            recalibration_commit=cand,
            recovery_base=self.repo.v0516_sha,
            recovery_cells="wikidata:8,wikidata:10",
        )
        res_rec = rp.verify_release(self.repo.work, expected_head=head_rec)
        self.assertTrue(res_rec["allowed"], res_rec)
        self.assertEqual(res_rec["mode"], "verified-i9-pass")

    def test_11d_malformed_recovery_cells_fail_closed(self) -> None:
        # A recovery-cell spec in the wrong form (e.g. "wikidata@8") must never be
        # silently dropped: that would skip the recovery check for those cells.
        self.assertEqual(
            rp._parse_recovery_cells(" wikidata:8, wikidata:10 ,"),
            [("wikidata", 8), ("wikidata", 10)],
        )
        for bad in ("wikidata@8", "wikidata:8,text@10", ":8", "wikidata:", "wikidata:0"):
            with self.assertRaises(rp.PreflightError, msg=bad):
                rp._parse_recovery_cells(bad)

        # judge CLI refuses a malformed spec rather than judging without recovery.
        with tempfile.TemporaryDirectory() as td:
            bundle = Path(td) / "missing.tar.gz"
            rc = rp.main(["judge", "--recovery-cells", "wikidata@8", str(bundle)])
        self.assertEqual(rc, 2)

        # verify rejects a receipt whose preflight_recovery_cells is malformed,
        # even when the bundle itself would pass.
        self.repo.write("src/open/dense.rs", "// candidate\n")
        cand = self.repo.commit("fix(open): candidate")
        rows = make_paired_rows(
            wall_ratios={("wikidata", 8): 0.974, ("wikidata", 10): 0.985},
            include_recovery=True,
            recovery_wall_ratios={("wikidata", 8): 1.000, ("wikidata", 10): 0.999},
        )
        tar, sha = build_tarball_bytes(
            rows=rows,
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            recovery_base_sha=self.repo.v0516_sha,
        )
        ev, _ = self.repo.publish_evidence("release-preflight/20261128T011700Z.tar.gz", tar)
        head = self.repo.write_preflight_delta(
            "2026-11-28-malformed-cells.md",
            candidate_sha=cand,
            base_sha=self.repo.v0517_sha,
            ev_commit=ev,
            bundle_path="release-preflight/20261128T011700Z.tar.gz",
            sha256=sha,
            recalibration="completed",
            recalibration_commit=cand,
            recovery_base=self.repo.v0516_sha,
            recovery_cells="wikidata@8,wikidata@10",
        )
        res = rp.verify_release(self.repo.work, expected_head=head)
        self.assertFalse(res["allowed"], res)
        self.assertTrue(any("malformed recovery cell" in p for p in res["problems"]), res)

    # ------------------------------------------------------------------
    # 12. Rejection precedes version bump, tag, push, or publish in deploy.yml & ship.just
    # ------------------------------------------------------------------
    def test_12_deploy_workflow_and_ship_just_enforce_gate_before_bump_tag_push(self) -> None:
        deploy_text = (REPO_ROOT / ".github/workflows/deploy.yml").read_text(encoding="utf-8")
        gate_idx = deploy_text.find("bench/release_preflight.py verify")
        release_idx = deploy_text.find("just release")
        ci_idx = deploy_text.find("gh workflow run CI.yml")
        self.assertGreater(gate_idx, 0, "deploy.yml must invoke bench/release_preflight.py verify")
        self.assertLess(gate_idx, release_idx, "preflight verify must run before just release")
        self.assertLess(release_idx, ci_idx, "just release must run before CI.yml publish dispatch")
        self.assertIn("--expected-head", deploy_text, "deploy.yml must bind expected head SHA")
        # Must NOT expose any arbitrary skip/bypass/PASS input
        self.assertNotRegex(
            deploy_text,
            re.compile(r"\b(skip_preflight|bypass_preflight|preflight_pass|force_release)\b", re.I),
        )

        ship_text = (REPO_ROOT / ".just/ship.just").read_text(encoding="utf-8")
        # Check every release/ship recipe runs release_preflight.py verify before mutating versions or tags
        for recipe_name, mutator in (
            ("release bump_level", "uv version --frozen --bump"),
            ("ship-rust-and-python", 'uv version --frozen "$version"'),
            ("ship-rust version", "release-plz set-version"),
            ("ship-rust-manual", "cargo set-version --bump"),
        ):
            r_start = ship_text.index(recipe_name)
            m_pos = ship_text.index(mutator, r_start)
            g_pos = ship_text.find("bench/release_preflight.py verify", r_start, m_pos)
            self.assertNotEqual(
                g_pos,
                -1,
                f".just/ship.just recipe {recipe_name!r} must run release_preflight.py verify before {mutator!r}",
            )

        # Justfile must import .just/release-preflight.just
        justfile_text = (REPO_ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn('import ".just/release-preflight.just"', justfile_text)
        self.assertTrue((REPO_ROOT / ".just/release-preflight.just").is_file())

    def test_12b_end_to_end_ship_guard_aborts_before_any_mutation_on_reject(self) -> None:
        """Execute a release guard script mirroring .just/ship.just with fake tools."""
        self.repo.write("src/open/dense.rs", "// unverified change\n")
        self.repo.commit("perf(open): unverified")
        (self.repo.work / "bench").mkdir(parents=True, exist_ok=True)
        shutil.copyfile(
            REPO_ROOT / "bench/release_preflight.py", self.repo.work / "bench/release_preflight.py"
        )
        # Keep bench/release_preflight.py tracked & clean
        self.repo.commit("ops: add preflight verifier")
        before_cargo = (self.repo.work / "Cargo.toml").read_text()
        before_pyproj = (self.repo.work / "pyproject.toml").read_text()
        before_tags = git(self.repo.work, "tag", "-l")
        before_head = git(self.repo.work, "rev-parse", "HEAD")

        call_log = self.tmp / "release-calls.log"
        script = f"""\
set -euo pipefail
python3 bench/release_preflight.py verify --expected-head "{before_head}"
echo "BUMP_CALLED" >> "{call_log}"
"""
        proc = subprocess.run(
            ["bash", "-c", script],
            cwd=self.repo.work,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(proc.returncode, 0, "unverified release must abort")
        self.assertFalse(call_log.exists(), "bump/tag/push must never execute after gate rejection")
        self.assertEqual((self.repo.work / "Cargo.toml").read_text(), before_cargo)
        self.assertEqual((self.repo.work / "pyproject.toml").read_text(), before_pyproj)
        self.assertEqual(git(self.repo.work, "tag", "-l"), before_tags)
        self.assertEqual(git(self.repo.work, "rev-parse", "HEAD"), before_head)


if __name__ == "__main__":
    unittest.main()
