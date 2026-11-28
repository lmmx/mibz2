"""Adversarial stress tests for the frontier protocol (claims stress-test-frontier & harden-frontier-tooling).

Exercises all eight scenarios from `frontier/claims/stress-test-frontier.toml`
against isolated temporary bare git remotes so no debris refs are pushed to the
real `origin`. All protocol invariants and regression targets are asserted as
hard passes.
"""

from __future__ import annotations

import concurrent.futures
import contextlib
import datetime as dt
import fcntl
import hashlib
import importlib.util
import io
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import unittest
from unittest import mock

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
FRONTIER_SRC = REPO_ROOT / "frontier" / "tools" / "frontier.py"
RUN_QUEUE_SH = REPO_ROOT / "bench" / "run-queue.sh"

GIT_ENV = {
    "GIT_AUTHOR_NAME": "Frontier Test",
    "GIT_AUTHOR_EMAIL": "frontier-test@example.invalid",
    "GIT_COMMITTER_NAME": "Frontier Test",
    "GIT_COMMITTER_EMAIL": "frontier-test@example.invalid",
}

BASE_CONFIG = """\
status_issue = 943
needs_louis_issue = 944
max_live_holds = 3
max_open_claims = 12
protected = [
  ".github/",
  "AGENTS.md",
  "CLAUDE.md",
  "GEMINI.md",
  "bench/workstation-queue.txt",
  "docs/RELEASING.md",
  "docs/team/RULES.md",
  "frontier/README.md",
  "frontier/config.toml",
  "frontier/tools/",
]
"""

BASE_README = """\
# Frontier test fixture

<!-- brief -->
Minimal brief block for tests.
<!-- /brief -->
"""

BASE_SURFACE = (
    "target\tpart\twhat\troom_1w\troom_8w\tstate\tevidence\tclaims\n"
    "demo\tscan\tdemo row\t10 ms\t2 ms\tmeasured\tabc123\tc1\n"
)

BASE_DEAD_ENDS = (
    "claim\tverdict\twhat it showed\tevidence\nold-idea\tKILL\tslower by 12%\tdelta-001\n"
)

SAMPLE_DELTA = """\
claim: c1
by: agent-a
outcome: done
evidence: commit deadbeef

**What was asked:** test baseline delta.
**What I found:** claimed a 50% speedup (flawed arithmetic).
**What changed:** docs/note.md.
**What's next:** verify independently.
**Caveats:** unverified.
"""

SAMPLE_FLAG = """\
flags: flagged-init
invariant: conservation of evidence
by: agent-b
status: open

**Evidence:** initial committed flag for deletion test.
**Scope:** flagged-init.
**What would resolve it:** independent check.
"""


def _claim_toml(
    cid: str,
    *,
    kind: str = "probe",
    zone: str = "smooth",
    needs: list[str] | None = None,
    priority: int = 1,
    status: str = "open",
    pr: int = 0,
    owned_paths: list[str] | None = None,
    mechanisms: list[str] | None = None,
    needs_louis: str = "",
    premises: list[str] | None = None,
    stop_test: str = "PASS iff exact bytes match.",
    done_when: str = "Delta committed.",
    links: list[str] | None = None,
) -> str:
    """Build a valid claim TOML document for test fixtures."""
    op = owned_paths or []
    mechs = mechanisms or []
    prems = premises or []
    req_needs = needs if needs is not None else ["shell"]
    lks = links or []
    pr_line = f"pr = {pr}\n" if pr else ""
    return (
        f'id = "{cid}"\n'
        f'title = "Test claim {cid}"\n'
        f'kind = "{kind}"\n'
        f'zone = "{zone}"\n'
        f"needs = {req_needs!r}\n".replace("'", '"')
        + f"priority = {priority}\n"
        + f'status = "{status}"\n'
        + pr_line
        + f"owned_paths = {op!r}\n".replace("'", '"')
        + f"mechanisms = {mechs!r}\n".replace("'", '"')
        + f'needs_louis = "{needs_louis}"\n'
        + f'question = "Test question for {cid}"\n'
        + 'why = "Test why"\n'
        + f'stop_test = "{stop_test}"\n'
        + f'done_when = "{done_when}"\n'
        + f"premises = {prems!r}\n".replace("'", '"')
        + f"links = {lks!r}\n".replace("'", '"')
    )


def _git(cwd: pathlib.Path, *args: str, env: dict[str, str] | None = None) -> str:
    """Run a git command in `cwd`, asserting exit status 0."""
    full_env = {**os.environ, **GIT_ENV, **(env or {})}
    res = subprocess.run(
        ["git", "-C", str(cwd), *args],
        capture_output=True,
        text=True,
        env=full_env,
    )
    if res.returncode != 0:
        raise AssertionError(
            f"git {' '.join(args)} failed ({res.returncode}):\n{res.stdout}\n{res.stderr}"
        )
    return res.stdout.strip()


class FrontierSandbox:
    """Hermetic temporary bare remote plus helper to spawn independent agent clones."""

    def __init__(self, tmp: pathlib.Path) -> None:
        """Create a bare remote and push an initial frontier repository to `master`."""
        self.tmp = tmp
        self.remote = tmp / "origin.git"
        _git(tmp, "init", "--bare", "-b", "master", str(self.remote))
        self.seed = tmp / "seed"
        _git(tmp, "clone", str(self.remote), str(self.seed))
        self._populate_seed()
        _git(self.seed, "add", ".")
        _git(self.seed, "commit", "-m", "init frontier fixture")
        _git(self.seed, "push", "-u", "origin", "master")

    def _populate_seed(self) -> None:
        """Write minimal frontier files, claims, deltas, flags and protected files."""
        fr = self.seed / "frontier"
        for sub in ("claims", "blog", "deltas", "flags", "tools"):
            (fr / sub).mkdir(parents=True, exist_ok=True)
        shutil.copy2(FRONTIER_SRC, fr / "tools" / "frontier.py")
        (fr / "config.toml").write_text(BASE_CONFIG)
        (fr / "README.md").write_text(BASE_README)
        (fr / "surface.tsv").write_text(BASE_SURFACE)
        (fr / "dead-ends.tsv").write_text(BASE_DEAD_ENDS)
        (fr / "deltas" / "2026-11-26-initial.md").write_text(SAMPLE_DELTA)
        (fr / "flags" / "2026-11-26-initial.md").write_text(SAMPLE_FLAG)
        (self.seed / "CLAUDE.md").write_text("# Safety rules\nNever relax C2.\n")
        (self.seed / "bench").mkdir(exist_ok=True)
        (self.seed / "bench" / "workstation-queue.txt").write_text("# repo: lmmx/mibz2\n")
        (self.seed / "bench" / "gate.frozen").write_text("threshold = 0.05\n")
        (self.seed / "docs").mkdir(exist_ok=True)
        (self.seed / "docs" / "premise.md").write_text("Original measured premise.\n")

        claims = {
            "c1": _claim_toml("c1", premises=["docs/premise.md"]),
            "c2": _claim_toml("c2", premises=["docs/premise.md"]),
            "c3": _claim_toml("c3"),
            "c4": _claim_toml("c4"),
            "race-a": _claim_toml("race-a"),
            "race-b": _claim_toml("race-b"),
            "race-c": _claim_toml("race-c"),
            "needs-louis": _claim_toml("needs-louis", needs_louis="RULES D"),
            "idea-one": _claim_toml("idea-one", status="idea"),
            "flagged-init": _claim_toml("flagged-init"),
            "run-a": _claim_toml(
                "run-a",
                zone="striated",
                owned_paths=["bench/workstation-queue.txt"],
                mechanisms=["queue"],
            ),
            "run-b": _claim_toml(
                "run-b",
                zone="striated",
                owned_paths=["bench/workstation-queue.txt"],
            ),
            "run-parent": _claim_toml(
                "run-parent",
                zone="striated",
                owned_paths=["bench/"],
            ),
            "run-sibling": _claim_toml(
                "run-sibling",
                zone="striated",
                owned_paths=["bench/workstation-queue-other.txt"],
            ),
            "same-mech": _claim_toml(
                "same-mech",
                zone="smooth",
                mechanisms=["queue"],
            ),
        }
        for cid, body in claims.items():
            (fr / "claims" / f"{cid}.toml").write_text(body)

    def clone(self, name: str) -> pathlib.Path:
        """Create an independent working clone of the bare remote."""
        dst = self.tmp / name
        _git(self.tmp, "clone", str(self.remote), str(dst))
        _git(dst, "config", "--local", "user.name", GIT_ENV["GIT_AUTHOR_NAME"])
        _git(dst, "config", "--local", "user.email", GIT_ENV["GIT_AUTHOR_EMAIL"])
        return dst

    def run_frontier(self, repo: pathlib.Path, *args: str) -> subprocess.CompletedProcess[str]:
        """Run `frontier/tools/frontier.py` inside `repo`."""
        return subprocess.run(
            [sys.executable, str(repo / "frontier" / "tools" / "frontier.py"), *args],
            cwd=str(repo),
            capture_output=True,
            text=True,
            env={**os.environ, **GIT_ENV},
        )


def _extract_session(output: str) -> str:
    """Parse the 12-hex session identifier from `frontier.py claim` output."""
    for line in output.splitlines():
        if "your session is " in line:
            return line.rsplit("your session is ", 1)[1].strip()
    raise AssertionError(f"session id not found in output:\n{output}")


def _load_frontier_module(repo: pathlib.Path):
    """Dynamically load the sandbox copy of `frontier.py` as an isolated module."""
    path = repo / "frontier" / "tools" / "frontier.py"
    spec = importlib.util.spec_from_file_location(f"frontier_mod_{id(repo)}", path)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class TestFrontierStress(unittest.TestCase):
    """Machine-checkable tests for all eight frontier stress-test scenarios."""

    def setUp(self) -> None:
        """Create a fresh isolated git remote and workspace per test."""
        self._td = tempfile.TemporaryDirectory(prefix="frontier-stress-")
        self.sb = FrontierSandbox(pathlib.Path(self._td.name))

    def tearDown(self) -> None:
        """Remove temporary directories after each test."""
        self._td.cleanup()

    # ------------------------------------------------------------------
    # Scenario 1: two agents claim the same id at once
    # ------------------------------------------------------------------

    def test_read_only_check_needs_no_commit_identity(self) -> None:
        """Local validation does not acquire commit identity or require a PR query."""
        a = self.sb.clone("reader")
        _git(a, "config", "--local", "--unset", "user.email")
        mod = _load_frontier_module(a)
        with (
            mock.patch.object(mod, "lease_identity", side_effect=AssertionError("identity read")),
            mock.patch.object(mod, "read_open_prs", side_effect=AssertionError("PR query")),
        ):
            self.assertEqual(mod.check(), 0)

    def test_lease_identity_is_local_and_missing_identity_refuses_write(self) -> None:
        """Commit identity is configured in the checkout, not baked into the client."""
        a = self.sb.clone("author")
        mod = _load_frontier_module(a)
        self.assertEqual(mod.lease_identity(), GIT_ENV)
        _git(a, "config", "--local", "--unset", "user.email")
        self.assertIsNone(mod.lease_identity())
        result = self.sb.run_frontier(a, "claim", "c1", "--by", "author")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("repository-local Git identity is not configured", result.stderr)
        self.assertNotIn("c1", mod.holds())
        _git(a, "config", "--local", "user.email", GIT_ENV["GIT_AUTHOR_EMAIL"])
        result = self.sb.run_frontier(a, "claim", "c1", "--by", "author")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        sha = mod.holds()["c1"][0]
        self.assertEqual(
            _git(a, "log", "-1", "--format=%ae %ce", sha),
            "frontier-test@example.invalid frontier-test@example.invalid",
        )

    def test_s1_concurrent_claim_same_id_exactly_one_wins(self) -> None:
        """Two concurrent claims on the same id yield exactly one acquisition."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")
        for cid in ("race-a", "race-b", "race-c"):
            barrier = threading.Barrier(2)

            def _claim(repo: pathlib.Path, who: str) -> subprocess.CompletedProcess[str]:
                barrier.wait()
                return self.sb.run_frontier(repo, "claim", cid, "--by", who)

            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                f1 = pool.submit(_claim, a, "agent-a")
                f2 = pool.submit(_claim, b, "agent-b")
                r1, r2 = f1.result(), f2.result()

            codes = sorted([r1.returncode, r2.returncode])
            self.assertEqual(
                codes,
                [0, 1],
                f"expected exactly one winner for {cid}: r1={r1.returncode} ({r1.stdout}{r1.stderr}), "
                f"r2={r2.returncode} ({r2.stdout}{r2.stderr})",
            )
            winner_out = r1.stdout if r1.returncode == 0 else r2.stdout
            self.assertIn(f"acquired {cid}", winner_out)
            # Release so the next iteration stays under max_live_holds=3
            winner_repo = a if r1.returncode == 0 else b
            sess = _extract_session(winner_out)
            rel = self.sb.run_frontier(winner_repo, "release", cid, "--session", sess)
            self.assertEqual(rel.returncode, 0, rel.stdout + rel.stderr)

    def test_s1_budget_needs_louis_and_non_open_refused(self) -> None:
        """Claim refuses non-open claims, needs_louis claims, and beyond max_live_holds."""
        a = self.sb.clone("agent-a")
        r_louis = self.sb.run_frontier(a, "claim", "needs-louis", "--by", "agent-a")
        self.assertNotEqual(r_louis.returncode, 0)
        self.assertIn("needs Louis", r_louis.stderr)

        r_idea = self.sb.run_frontier(a, "claim", "idea-one", "--by", "agent-a")
        self.assertNotEqual(r_idea.returncode, 0)
        self.assertIn("not open", r_idea.stderr)

        # Fill max_live_holds = 3
        for cid in ("c1", "c2", "c3"):
            r = self.sb.run_frontier(a, "claim", cid, "--by", "agent-a")
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

        r_fourth = self.sb.run_frontier(a, "claim", "c4", "--by", "agent-a")
        self.assertNotEqual(r_fourth.returncode, 0)
        self.assertIn("budget is full", r_fourth.stderr)

    # ------------------------------------------------------------------
    # Scenario 2: agent dies mid-claim -> lease lapses -> CAS takeover
    # ------------------------------------------------------------------

    def test_s2_lapsed_lease_takeover_and_concurrent_takeover_race(self) -> None:
        """After a lease expires, concurrent challengers race via CAS and exactly one wins."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")
        c = self.sb.clone("agent-c")

        # Negative hours creates an already-lapsed lease on origin
        r_a = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a", "--hours", "-0.01")
        self.assertEqual(r_a.returncode, 0, r_a.stdout + r_a.stderr)
        sha_a = _git(self.sb.remote, "rev-parse", "refs/heads/hold/c1")

        holds_out = self.sb.run_frontier(b, "holds").stdout
        self.assertIn("c1: expired", holds_out)

        barrier = threading.Barrier(2)

        def _take(repo: pathlib.Path, who: str) -> subprocess.CompletedProcess[str]:
            barrier.wait()
            return self.sb.run_frontier(repo, "claim", "c1", "--by", who)

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            f_b = pool.submit(_take, b, "agent-b")
            f_c = pool.submit(_take, c, "agent-c")
            rb, rc = f_b.result(), f_c.result()

        self.assertEqual(sorted([rb.returncode, rc.returncode]), [0, 1])
        sha_new = _git(self.sb.remote, "rev-parse", "refs/heads/hold/c1")
        parent = _git(self.sb.remote, "rev-parse", f"{sha_new}^")
        self.assertEqual(parent, sha_a, "takeover commit must chain from lapsed hold commit")

    # ------------------------------------------------------------------
    # Scenario 3: lapsed holder tries verify, renew, release
    # ------------------------------------------------------------------

    def test_s3_lapsed_holder_refused_verify_renew_and_post_takeover_release(self) -> None:
        """Lapsed holder is refused verify/renew, and refused release once taken over."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")

        r_a = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a", "--hours", "-0.01")
        self.assertEqual(r_a.returncode, 0, r_a.stdout + r_a.stderr)
        sess_a = _extract_session(r_a.stdout)

        r_ver = self.sb.run_frontier(a, "verify", "c1", "--session", sess_a)
        self.assertNotEqual(r_ver.returncode, 0)
        self.assertIn("lapsed", r_ver.stderr)

        r_ren = self.sb.run_frontier(a, "renew", "c1", "--session", sess_a)
        self.assertNotEqual(r_ren.returncode, 0)
        self.assertIn("lapsed", r_ren.stderr)

        # B takes over
        r_b = self.sb.run_frontier(b, "claim", "c1", "--by", "agent-b")
        self.assertEqual(r_b.returncode, 0, r_b.stdout + r_b.stderr)
        sess_b = _extract_session(r_b.stdout)

        # A tries to release after B took over -> refused
        r_rel = self.sb.run_frontier(a, "release", "c1", "--session", sess_a)
        self.assertNotEqual(r_rel.returncode, 0)
        self.assertIn("not held by session", r_rel.stderr)

        # Live holder B can verify, renew, and release cleanly
        self.assertEqual(self.sb.run_frontier(b, "verify", "c1", "--session", sess_b).returncode, 0)
        self.assertEqual(self.sb.run_frontier(b, "renew", "c1", "--session", sess_b).returncode, 0)
        self.assertEqual(
            self.sb.run_frontier(b, "release", "c1", "--session", sess_b).returncode, 0
        )

    def test_s3_cas_failure_on_renew_and_release_must_exit_nonzero(self) -> None:
        """`renew` and `release` exit non-zero when CAS push is rejected."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")

        r_a = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a")
        sess_a = _extract_session(r_a.stdout)
        mod = _load_frontier_module(a)
        stale_holds = mod.holds()
        # Simulate B advancing hold/c1 while A's renew/release is mid-flight
        stale_sha, stale_lease = stale_holds["c1"]
        other_lease = {
            **stale_lease,
            "holder": "agent-b",
            "session": "bbbbbbbbbbbb",
            "expires": (dt.datetime.now(dt.timezone.utc) + dt.timedelta(hours=8))
            .replace(microsecond=0)
            .isoformat(),
        }
        mod_b = _load_frontier_module(b)
        mod_b.holds()
        ok, _ = mod_b.write_lease("c1", other_lease, stale_sha, stale_sha)
        self.assertTrue(ok)

        with mock.patch.object(mod, "holds", return_value=stale_holds):
            rc_renew = mod.main(["frontier.py", "renew", "c1", "--session", sess_a])
            rc_release = mod.main(["frontier.py", "release", "c1", "--session", sess_a])
        self.assertNotEqual(rc_renew, 0)
        self.assertNotEqual(rc_release, 0)

    def test_s3_verify_must_fail_when_origin_unreachable(self) -> None:
        """`verify` fails closed offline instead of trusting stale local hold refs."""
        a = self.sb.clone("agent-a")
        r_a = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a")
        sess_a = _extract_session(r_a.stdout)
        # Prime local remote-tracking ref, then break origin URL
        self.assertEqual(self.sb.run_frontier(a, "verify", "c1", "--session", sess_a).returncode, 0)
        _git(a, "remote", "set-url", "origin", str(self.sb.tmp / "missing-remote.git"))

        r_offline = self.sb.run_frontier(a, "verify", "c1", "--session", sess_a)
        self.assertNotEqual(r_offline.returncode, 0)

    # ------------------------------------------------------------------
    # Scenario 4: invalidating result -> premises catch it & flag holds
    # ------------------------------------------------------------------

    def test_s4_local_flag_holds_named_claims_only(self) -> None:
        """A local open flag blocks `claim` and `verify` for named ids without blocking others."""
        a = self.sb.clone("agent-a")
        r_c1 = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a")
        sess_c1 = _extract_session(r_c1.stdout)

        flag_file = a / "frontier" / "flags" / "2026-11-26-invalidate.md"
        flag_file.write_text(
            "flags: c1 c2\n"
            "invariant: traceability\n"
            "by: agent-b\n"
            "status: open\n\n"
            "**Evidence:** new result invalidates premise.\n"
            "**Scope:** c1 c2.\n"
            "**What would resolve it:** re-check premises.\n"
        )
        # Flag holds c1 (verify) and c2 (claim), while c3 carries on
        self.assertNotEqual(
            self.sb.run_frontier(a, "verify", "c1", "--session", sess_c1).returncode, 0
        )
        self.assertNotEqual(self.sb.run_frontier(a, "claim", "c2", "--by", "agent-a").returncode, 0)
        r_c3 = self.sb.run_frontier(a, "claim", "c3", "--by", "agent-a")
        self.assertEqual(r_c3.returncode, 0, r_c3.stdout + r_c3.stderr)

    def test_s4_remote_flag_on_origin_master_must_block_unrebased_claim_and_verify(self) -> None:
        """Flags merged on `origin/master` block un-rebased `claim` and `verify`."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")

        r_c1 = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a")
        sess_c1 = _extract_session(r_c1.stdout)

        # Agent B merges a flag to origin/master invalidating c1 and c2
        flag_b = b / "frontier" / "flags" / "2026-11-26-remote-invalidate.md"
        flag_b.write_text(
            "flags: c1 c2\n"
            "invariant: traceability\n"
            "by: agent-b\n"
            "status: open\n\n"
            "**Evidence:** new measurement contradicts docs/premise.md.\n"
            "**Scope:** c1 c2.\n"
            "**What would resolve it:** update claims.\n"
        )
        _git(b, "add", str(flag_b))
        _git(b, "commit", "-m", "flag c1 and c2")
        _git(b, "push", "origin", "master")

        # Agent A has NOT pulled master locally, then calls verify(c1) and claim(c2)
        r_ver = self.sb.run_frontier(a, "verify", "c1", "--session", sess_c1)
        r_clm = self.sb.run_frontier(a, "claim", "c2", "--by", "agent-a")
        self.assertNotEqual(r_ver.returncode, 0)
        self.assertNotEqual(r_clm.returncode, 0)

    def test_s4_path_scoped_remote_flag_matches_premises_and_owned_paths(self) -> None:
        """Path-scoped flags on `origin/master` match claim `premises` and `owned_paths`."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")

        r_c1 = self.sb.run_frontier(a, "claim", "c1", "--by", "agent-a")
        sess_c1 = _extract_session(r_c1.stdout)

        # Agent B pushes a path-scoped flag covering docs/premise.md and bench/
        flag_b = b / "frontier" / "flags" / "2026-11-26-path-scope.md"
        flag_b.write_text(
            "flags: docs/premise.md bench/\n"
            "invariant: traceability\n"
            "by: agent-b\n"
            "status: open\n\n"
            "**Evidence:** premise and queue paths flagged.\n"
            "**Scope:** claims resting on docs/premise.md or owning bench/.\n"
            "**What would resolve it:** independent check.\n"
        )
        _git(b, "add", str(flag_b))
        _git(b, "commit", "-m", "flag premise and bench paths")
        _git(b, "push", "origin", "master")

        # c1 has premise docs/premise.md -> verify blocked
        self.assertNotEqual(
            self.sb.run_frontier(a, "verify", "c1", "--session", sess_c1).returncode, 0
        )
        # c2 has premise docs/premise.md -> claim blocked
        self.assertNotEqual(self.sb.run_frontier(a, "claim", "c2", "--by", "agent-a").returncode, 0)
        # run-a owns bench/workstation-queue.txt (under bench/) -> claim blocked
        self.assertNotEqual(
            self.sb.run_frontier(a, "claim", "run-a", "--by", "agent-a").returncode, 0
        )
        # c3 has no matching premise or owned_path -> allowed
        r_c3 = self.sb.run_frontier(a, "claim", "c3", "--by", "agent-a")
        self.assertEqual(r_c3.returncode, 0, r_c3.stdout + r_c3.stderr)

    def test_s4_check_validates_claim_premises_exist(self) -> None:
        """`check` reports nonexistent paths listed in claim `premises`."""
        a = self.sb.clone("agent-a")
        bad_claim = a / "frontier" / "claims" / "bad-prem.toml"
        bad_claim.write_text(_claim_toml("bad-prem", premises=["docs/nonexistent-evidence.md"]))
        r_chk = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r_chk.returncode, 0)
        self.assertIn("nonexistent-evidence.md", r_chk.stdout)

    def test_blog_admission_needs_news_and_one_post_in_progress(self) -> None:
        """A blog card past idea states its news and kind; one post is in progress at a time."""
        a = self.sb.clone("agent-a")

        def card(cid: str, stage: str, extra: str = "") -> str:
            base = _claim_toml(cid).replace('kind = "probe"', 'kind = "blog"')
            return base + f'stage = "{stage}"\npost = ""\ndrafted_by = "agent-a"\nreviewed_by = ""\n' + extra

        blog = a / "frontier" / "blog"
        blog.mkdir(parents=True, exist_ok=True)
        (blog / "b-idea.toml").write_text(card("b-idea", "idea"))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        (blog / "b-draft.toml").write_text(card("b-draft", "draft"))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("needs news", r.stdout)
        self.assertIn("news_kind", r.stdout)
        news = (
            'news = "A new induction order rebuilds text with a third fewer steps."\n'
            'news_kind = "discovery"\n'
            'evidence = ["frontier/surface.tsv"]\n'
        )
        (blog / "b-draft.toml").write_text(card("b-draft", "draft", news))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        (blog / "b-other.toml").write_text(
            card("b-other", "outline", news.replace("discovery", "advance"))
        )
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("2 posts in progress", r.stdout)
        (blog / "b-other.toml").write_text(card("b-other", "outline", news + "legacy = true\n"))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        (blog / "b-other.toml").write_text(
            card("b-other", "outline", news.replace("discovery", "negative-result"))
        )
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("lab-notebook entry", r.stdout)

    def test_blog_advance_cites_ledger_and_master_and_discovery_cites_evidence(self) -> None:
        """Advance cards require ledgered merged PRs; discovery cards require existing evidence."""
        a = self.sb.clone("agent-a")
        blog = a / "frontier" / "blog"
        blog.mkdir(parents=True, exist_ok=True)

        # Merge PR #501 onto origin/master and record PR #501 (merged) and #502 (unmerged) in ledger.
        (a / "src").mkdir(exist_ok=True)
        (a / "src" / "lib.rs").write_text("pub fn v() -> u32 { 1 }\n")
        _git(a, "add", "src/lib.rs")
        _git(a, "commit", "-m", "perf: bound decoder memory (#501)")
        _git(a, "push", "origin", "master")
        _git(a, "fetch", "-q", "origin")

        (a / "frontier" / "ledger.tsv").write_text(
            "pr\tmerged\teffect\tstate\tmeasured\tevidence\n"
            "501\t2026-11-27\tBounds peak memory on zeros.\tdefault\t-50% RSS on zeros, local\tPR #501\n"
            "502\t2026-11-27\tUnmerged candidate.\texperimental\tnot measured\tPR #502\n"
        )

        def adv_card(extra: str) -> str:
            base = _claim_toml("b-adv").replace('kind = "probe"', 'kind = "blog"')
            return (
                base
                + 'stage = "draft"\npost = ""\ndrafted_by = "agent-a"\nreviewed_by = ""\n'
                + 'news = "Decoder memory stays flat on compressible streams."\n'
                + 'news_kind = "advance"\n'
                + extra
            )

        # (1a) Advance card with no changes -> fails check.
        (blog / "b-adv.toml").write_text(adv_card(""))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("changes", r.stdout)

        # (1b) Advance card naming an unledgered PR (#999) -> fails check.
        (blog / "b-adv.toml").write_text(adv_card("changes = [999]\n"))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("no row in frontier/ledger.tsv", r.stdout)

        # (1c) Advance card naming a ledgered but unmerged PR (#502) -> fails check.
        (blog / "b-adv.toml").write_text(adv_card("changes = [502]\n"))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("no merge commit on origin/master", r.stdout)

        # (2) Advance card naming a ledgered PR merged on origin/master (#501) -> passes check.
        (blog / "b-adv.toml").write_text(adv_card("changes = [501]\n"))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

        # (3) Discovery card with missing evidence path -> fails; existing path -> passes.
        (blog / "b-adv.toml").unlink()
        disc_base = (
            _claim_toml("b-disc").replace('kind = "probe"', 'kind = "blog"')
            + 'stage = "outline"\npost = ""\ndrafted_by = "agent-a"\nreviewed_by = ""\n'
            + 'news = "Memory traffic saturates before eight workers."\n'
            + 'news_kind = "discovery"\n'
        )
        (blog / "b-disc.toml").write_text(disc_base + 'evidence = ["docs/does-not-exist.md"]\n')
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("evidence path does not exist", r.stdout)

        (blog / "b-disc.toml").write_text(disc_base + 'evidence = ["frontier/surface.tsv"]\n')
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_frontier_news_lists_unposted_default_and_experimental_changes(self) -> None:
        """`frontier.py news` lists unposted default/experimental ledger rows since last post."""
        a = self.sb.clone("agent-a")
        blog_posts = a / "docs" / "site" / "blog"
        blog_posts.mkdir(parents=True, exist_ok=True)
        (blog_posts / "2026-11-26-previous-post.md").write_text("# Previous\n")

        (a / "frontier" / "ledger.tsv").write_text(
            "pr\tmerged\teffect\tstate\tmeasured\tevidence\n"
            "600\t2026-11-25\tOlder default change.\tdefault\tnot measured\tPR #600\n"
            "601\t2026-11-27\tUnposted default change.\tdefault\t-50% RSS on zeros, local\tPR #601\n"
            "602\t2026-11-27\tAlready covered default change.\tdefault\t-4% wall on text, CI\tPR #602\n"
            "603\t2026-11-27\tUnposted experimental flag.\texperimental\t-7% CPU on wikidata, VM\tPR #603\n"
            "604\t2026-11-27\tResearch prototype only.\tprototype\tnot measured\tPR #604\n"
            "605\t2026-11-27\tThroughput target only.\ttarget\tnot measured\tdocs/model.md\n"
        )

        blog_cards = a / "frontier" / "blog"
        blog_cards.mkdir(parents=True, exist_ok=True)
        covered = (
            _claim_toml("b-covered").replace('kind = "probe"', 'kind = "blog"')
            + 'stage = "published"\npost = "docs/site/blog/2026-11-26-previous-post.md"\n'
            + 'drafted_by = "agent-a"\nreviewed_by = "agent-b"\n'
            + 'news = "Faster text decoding."\nnews_kind = "advance"\nchanges = [602]\n'
        )
        (blog_cards / "b-covered.toml").write_text(covered)

        r = self.sb.run_frontier(a, "news")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("#601", r.stdout)
        self.assertIn("#603", r.stdout)
        self.assertNotIn("#600", r.stdout)
        self.assertNotIn("#602", r.stdout)
        self.assertNotIn("#604", r.stdout)
        self.assertNotIn("#605", r.stdout)

    def test_release_due_is_recognised_and_release_claims_are_p1(self) -> None:
        """Software merged since the last tag with no live release claim is a due release (P1)."""
        a = self.sb.clone("agent-a")
        _git(a, "tag", "v0.0.1")
        _git(a, "push", "origin", "v0.0.1")
        r = self.sb.run_frontier(a, "check")
        self.assertNotIn("release due", r.stdout + r.stderr)
        (a / "src").mkdir(exist_ok=True)
        (a / "src" / "lib.rs").write_text("pub fn f() {}\n")
        _git(a, "add", "src/lib.rs")
        _git(a, "commit", "-m", "perf: a software change")
        _git(a, "push", "origin", "master")
        _git(a, "fetch", "-q", "origin")
        r = self.sb.run_frontier(a, "check")
        self.assertIn("release due: 1 software commit(s) on master since v0.0.1", r.stdout + r.stderr)
        rel = a / "frontier" / "claims" / "release-0-0-2.toml"
        rel.write_text(_claim_toml("release-0-0-2").replace("priority = 1", "priority = 2"))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("a release claim is P1", r.stdout)
        rel.write_text(_claim_toml("release-0-0-2"))
        r = self.sb.run_frontier(a, "check")
        self.assertNotIn("release due", r.stdout + r.stderr)
        self.assertNotIn("a release claim is P1", r.stdout)

    # ------------------------------------------------------------------
    # Scenario 5: persuasive wrong argument -> independent cold review
    # ------------------------------------------------------------------

    def test_s5_cold_review_contradicts_committed_delta_and_surfaces_flag(self) -> None:
        """Independent cold review can append a contradicting delta and open a visible flag."""
        b = self.sb.clone("agent-b")
        contra = b / "frontier" / "deltas" / "2026-11-26-cold-review-c1.md"
        contra.write_text(
            "claim: none\n"
            "by: agent-b\n"
            "outcome: done\n"
            "evidence: docs/premise.md\n\n"
            "**What was asked:** cold review of c1's speedup claim.\n"
            "**What I found:** arithmetic in 2026-11-26-initial.md is wrong by 10x.\n"
            "**What changed:** added flag 2026-11-26-wrong-arg.md.\n"
            "**What's next:** re-measure.\n"
            "**Caveats:** none.\n"
        )
        flag = b / "frontier" / "flags" / "2026-11-26-wrong-arg.md"
        flag.write_text(
            "flags: c1\n"
            "invariant: traceability\n"
            "by: agent-b\n"
            "status: open\n\n"
            "**Evidence:** cold check of primary numbers.\n"
            "**Scope:** c1.\n"
            "**What would resolve it:** corrected measurement.\n"
        )
        r_chk = self.sb.run_frontier(b, "check")
        self.assertEqual(r_chk.returncode, 0, r_chk.stdout + r_chk.stderr)
        r_status = self.sb.run_frontier(b, "status")
        self.assertEqual(r_status.returncode, 0)
        status_md = (b / "frontier" / "STATUS.md").read_text()
        self.assertIn("2026-11-26-wrong-arg.md", status_md)
        r_brief = self.sb.run_frontier(b, "brief")
        self.assertIn("2026-11-26-wrong-arg.md", r_brief.stdout)

    def test_s5_check_rejects_flag_resolved_without_independent_resolver(self) -> None:
        """`check` rejects resolved flags missing independent `resolved_by:` and accepts distinct ones."""
        a = self.sb.clone("agent-a")
        flag = a / "frontier" / "flags" / "2026-11-26-initial.md"
        original = flag.read_text()

        # Missing resolved_by -> rejected
        flag.write_text(original.replace("status: open", "status: resolved"))
        r_missing = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r_missing.returncode, 0)

        # Self-resolution (same as by: agent-b) -> rejected
        flag.write_text(original.replace("status: open", "status: resolved\nresolved_by: agent-b"))
        r_self = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r_self.returncode, 0)

        # Independent resolver (different from by: agent-b) -> accepted
        flag.write_text(original.replace("status: open", "status: resolved\nresolved_by: agent-c"))
        r_ok = self.sb.run_frontier(a, "check")
        self.assertEqual(r_ok.returncode, 0, r_ok.stdout)

    # ------------------------------------------------------------------
    # Scenario 6: gate changed after data -> check & review catch it
    # ------------------------------------------------------------------

    def test_s6_check_catches_modified_delta_and_deleted_flag_locally(self) -> None:
        """Locally with `origin/master`, modifying a delta or deleting a flag fails `check`."""
        a = self.sb.clone("agent-a")
        delta = a / "frontier" / "deltas" / "2026-11-26-initial.md"
        delta.write_text(delta.read_text() + "edited after commit\n")
        r_mod = self.sb.run_frontier(a, "check")
        self.assertEqual(r_mod.returncode, 1)
        self.assertIn("append-only", r_mod.stdout)

        _git(a, "checkout", "--", str(delta))
        (a / "frontier" / "flags" / "2026-11-26-initial.md").unlink()
        r_del = self.sb.run_frontier(a, "check")
        self.assertEqual(r_del.returncode, 1)
        self.assertIn("never deleted", r_del.stdout)

    def test_s6_check_catches_modified_stop_test_and_deleted_dead_end(self) -> None:
        """`check` catches relaxed `stop_test` and deleted or rewritten `dead-ends.tsv` rows."""
        a = self.sb.clone("agent-a")
        c1 = a / "frontier" / "claims" / "c1.toml"
        c1.write_text(c1.read_text().replace("PASS iff exact bytes match.", "PASS always."))
        r_stop = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r_stop.returncode, 0)
        self.assertIn("stop_test", r_stop.stdout)

        _git(a, "checkout", "--", str(c1))
        dead = a / "frontier" / "dead-ends.tsv"
        dead.write_text("claim\tverdict\twhat it showed\tevidence\n")
        r_dead = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r_dead.returncode, 0)
        self.assertIn("dead-ends.tsv", r_dead.stdout)

    def test_s6_check_must_not_silently_skip_diff_without_origin_master(self) -> None:
        """Shallow checkout without `origin/master` still runs check #3 and fails closed offline."""
        shallow = self.sb.tmp / "shallow-ci"
        _git(self.sb.tmp, "clone", "--depth=1", f"file://{self.sb.remote}", str(shallow))
        # Simulate actions/checkout@v6 on refs/pull/N/merge where refs/remotes/origin/master is absent
        _git(shallow, "update-ref", "-d", "refs/remotes/origin/master")
        delta = shallow / "frontier" / "deltas" / "2026-11-26-initial.md"
        delta.write_text(delta.read_text() + "tampered in PR\n")
        r_chk = self.sb.run_frontier(shallow, "check")
        self.assertNotEqual(r_chk.returncode, 0)

        # Even on a clean tree, missing origin/master + unreachable origin fails closed
        _git(shallow, "checkout", "--", str(delta))
        _git(shallow, "update-ref", "-d", "refs/remotes/origin/master")
        _git(shallow, "remote", "set-url", "origin", str(self.sb.tmp / "missing-remote.git"))
        r_offline = self.sb.run_frontier(shallow, "check")
        self.assertNotEqual(r_offline.returncode, 0)
        self.assertIn("origin/master", r_offline.stdout)

    def test_s6_unrebased_parallel_branch_not_false_flagged_by_new_master_delta(self) -> None:
        """Merge-base diff does not false-fail un-rebased branches when master adds a delta."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")
        _git(a, "checkout", "-b", "frontier/c2")

        # B merges a new delta to origin/master
        d_b = b / "frontier" / "deltas" / "2026-11-26-from-b.md"
        d_b.write_text(SAMPLE_DELTA.replace("claim: c1", "claim: c3"))
        _git(b, "add", str(d_b))
        _git(b, "commit", "-m", "add delta from b")
        _git(b, "push", "origin", "master")

        # A fetches origin (as `claim` does!) without rebasing its own branch
        _git(a, "fetch", "origin", "master")
        r_chk = self.sb.run_frontier(a, "check")
        self.assertEqual(r_chk.returncode, 0, r_chk.stdout)

    # ------------------------------------------------------------------
    # Scenario 7: two sessions trigger a workstation run
    # ------------------------------------------------------------------

    def test_s7_overlapping_owned_paths_and_mechanisms_refused_sequentially(self) -> None:
        """Striated owned_paths overlap (exact/parent) and shared mechanisms refuse the second."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")

        r1 = self.sb.run_frontier(a, "claim", "run-a", "--by", "agent-a")
        self.assertEqual(r1.returncode, 0, r1.stdout + r1.stderr)

        # Exact owned_paths clash -> refused
        r2 = self.sb.run_frontier(b, "claim", "run-b", "--by", "agent-b")
        self.assertNotEqual(r2.returncode, 0)
        self.assertIn("would share", r2.stderr)

        # Parent directory clash (`bench/` contains `bench/workstation-queue.txt`) -> refused
        r_par = self.sb.run_frontier(b, "claim", "run-parent", "--by", "agent-b")
        self.assertNotEqual(r_par.returncode, 0)
        self.assertIn("would share", r_par.stderr)

        # Shared mechanism -> refused even in smooth zone
        r_mech = self.sb.run_frontier(b, "claim", "same-mech", "--by", "agent-b")
        self.assertNotEqual(r_mech.returncode, 0)
        self.assertIn("queue", r_mech.stderr)

        # Non-overlapping sibling path -> allowed (segment-aware matching)
        r_sib = self.sb.run_frontier(b, "claim", "run-sibling", "--by", "agent-b")
        self.assertEqual(r_sib.returncode, 0, r_sib.stdout + r_sib.stderr)

    def test_s7_workstation_flock_serializes_concurrent_runners(self) -> None:
        """The workstation runner's flock on `workstation/queue.lock` refuses a second run."""
        state_home = self.sb.tmp / "xdg-state"
        lock_file = state_home / "workstation" / "queue.lock"
        lock_file.parent.mkdir(parents=True, exist_ok=True)
        with open(lock_file, "a+") as held:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            held.write("holder-pid\n")
            held.flush()
            res = subprocess.run(
                ["bash", str(RUN_QUEUE_SH), "--no-suspend", "--repo", str(self.sb.seed)],
                cwd=str(self.sb.seed),
                capture_output=True,
                text=True,
                env={**os.environ, "XDG_STATE_HOME": str(state_home)},
            )
        self.assertNotEqual(res.returncode, 0)
        combined = res.stdout + res.stderr
        self.assertIn("queue.lock", combined)

    def test_s7_verify_detects_cross_id_overlapping_owned_paths_after_race(self) -> None:
        """`verify` re-checks overlapping `owned_paths` across live holds after a race."""
        a = self.sb.clone("agent-a")
        b = self.sb.clone("agent-b")
        mod_a = _load_frontier_module(a)
        mod_b = _load_frontier_module(b)
        now_iso = dt.datetime.now(dt.timezone.utc).replace(microsecond=0)
        exp_iso = (now_iso + dt.timedelta(hours=8)).isoformat()
        # Deterministically create the post-TOCTOU state: two live holds owning the same queue file
        lease_a = {
            "id": "run-a",
            "holder": "agent-a",
            "session": "aaaaaaaaaaaa",
            "claimed_at": now_iso.isoformat(),
            "expires": exp_iso,
            "owned_paths": ["bench/workstation-queue.txt"],
            "mechanisms": [],
            "base": _git(a, "rev-parse", "origin/master"),
            "released": False,
        }
        lease_b = {**lease_a, "id": "run-b", "holder": "agent-b", "session": "bbbbbbbbbbbb"}
        self.assertTrue(mod_a.write_lease("run-a", lease_a, None, None)[0])
        self.assertTrue(mod_b.write_lease("run-b", lease_b, None, None)[0])

        r_va = self.sb.run_frontier(a, "verify", "run-a", "--session", "aaaaaaaaaaaa")
        r_vb = self.sb.run_frontier(b, "verify", "run-b", "--session", "bbbbbbbbbbbb")
        self.assertTrue(r_va.returncode != 0 or r_vb.returncode != 0)

    # ------------------------------------------------------------------
    # Scenario 8: agent loosens a safety rule -> protected paths & merge
    # ------------------------------------------------------------------

    def test_s8_protected_path_edit_emits_warning_locally(self) -> None:
        """Locally with `origin/master`, editing a protected file emits a warning."""
        a = self.sb.clone("agent-a")
        (a / "CLAUDE.md").write_text("# Safety rules\nC2 is optional now.\n")
        r_chk = self.sb.run_frontier(a, "check")
        self.assertEqual(r_chk.returncode, 0)
        self.assertIn("warning: protected files changed (CLAUDE.md)", r_chk.stdout)

    def test_s8_protected_warning_cannot_be_silenced_by_editing_config(self) -> None:
        """Editing `frontier/config.toml` to clear `protected` does not silence `check`."""
        a = self.sb.clone("agent-a")
        (a / "CLAUDE.md").write_text("# Safety rules\nC2 is optional now.\n")
        cfg = a / "frontier" / "config.toml"
        cfg.write_text(
            "status_issue = 943\nneeds_louis_issue = 944\n"
            "max_live_holds = 100\nmax_open_claims = 100\nprotected = []\n"
        )
        r_chk = self.sb.run_frontier(a, "check")
        self.assertIn("protected files changed", r_chk.stdout)

    def test_s8_protected_warning_survives_shallow_ci_checkout(self) -> None:
        """Shallow checkout without `origin/master` still warns on protected edits."""
        shallow = self.sb.tmp / "shallow-ci-s8"
        _git(self.sb.tmp, "clone", "--depth=1", f"file://{self.sb.remote}", str(shallow))
        _git(shallow, "update-ref", "-d", "refs/remotes/origin/master")
        (shallow / "CLAUDE.md").write_text("# Safety rules\nC2 is optional now.\n")
        r_chk = self.sb.run_frontier(shallow, "check")
        self.assertIn("protected files changed", r_chk.stdout)

    def test_s8_workflow_fetches_master_and_stops_needs_louis_at_open_flags(self) -> None:
        """`.github/workflows/frontier.yml` fetches `master` and slices only `## Needs Louis`."""
        wf = (REPO_ROOT / ".github" / "workflows" / "frontier.yml").read_text()
        self.assertIn("fetch-depth: 0", wf)
        self.assertIn("origin/master", wf)
        self.assertIn("/^## Open flags/{f=0}", wf)
        self.assertNotIn("## In progress", wf)

        # Verify the awk filter against rendered STATUS.md stops cleanly before ## Open flags
        a = self.sb.clone("agent-a")
        self.assertEqual(self.sb.run_frontier(a, "status").returncode, 0)
        res = subprocess.run(
            ["awk", "/^## Needs Louis/{f=1} /^## Open flags/{f=0} f", "frontier/STATUS.md"],
            cwd=str(a),
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertIn("## Needs Louis", res.stdout)
        self.assertIn("needs-louis", res.stdout)
        self.assertNotIn("## Open flags", res.stdout)
        self.assertNotIn("## The blog board", res.stdout)


    # ------------------------------------------------------------------
    # Live review discovery: a PR must be visible before its claim edit merges
    # ------------------------------------------------------------------

    def test_review_visible_from_clean_master_at_capacity(self) -> None:
        """The actual open PR is the review event, even at full capacity and before merge."""
        a = self.sb.clone("author")
        for cid in ("c1", "c2", "c3"):
            result = self.sb.run_frontier(a, "claim", cid, "--by", "author")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        _git(a, "switch", "-c", "work-c1")
        card = a / "frontier" / "claims" / "c1.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "review"\npr = 42'))
        _git(a, "add", str(card))
        _git(a, "commit", "-m", "work awaiting review")
        _git(a, "push", "origin", "work-c1")
        head = _git(a, "rev-parse", "HEAD")

        b = self.sb.clone("fresh-reviewer")
        self.assertIn('status = "open"', (b / "frontier/claims/c1.toml").read_text())
        mod = _load_frontier_module(b)
        pr = {
            "number": 42,
            "body": "Claim: c1\n\nReady for independent review.",
            "head": {"sha": head},
            "base": {"ref": "master"},
            "state": "open",
            "draft": False,
        }
        with mock.patch.object(mod, "read_open_prs", return_value=([pr], None)):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                mod.brief({"shell"})
            brief = buf.getvalue()
            status = mod.render()
        review = brief.split("## Awaiting review or merge", 1)[1].split("##", 1)[0]
        self.assertIn(f"c1: PR #42 at {head[:12]}", review)
        self.assertIn(f"c1: PR #42 at {head[:12]}", status)
        self.assertNotIn("**c1**", status.split("## Open claims, by priority", 1)[1].split("##", 1)[0])
        self.assertIn("c1", brief.split("## Held now", 1)[1].split("##", 1)[0])

    def test_review_projection_drafts_duplicates_and_stale_pointers(self) -> None:
        """Neither stale TOML pointers nor malformed, draft or duplicate PRs look like authority."""
        a = self.sb.clone("reader")
        mod = _load_frontier_module(a)
        items = mod.load("claims")
        c2 = next(c for c in items if c["id"] == "c2")
        c2.update(status="review", pr=99)

        def pr(number, body, draft=False):
            return {
                "number": number,
                "body": body,
                "head": {"sha": "a" * 40},
                "base": {"ref": "master"},
                "state": "open",
                "draft": draft,
            }

        rows = [
            pr(12, "Claim: c1"),
            pr(13, "Claim: c1", draft=True),
            pr(14, "Claim: none"),
            pr(15, "legacy PR"),
            pr(16, "Claim: not-in-checkout"),
        ]
        lines, pending = mod.review_projection(items, rows, None)
        out = "\n".join(lines)
        self.assertEqual(pending, {"c1", "not-in-checkout"})
        for part in (
            "PR #12",
            "PR #13",
            "multiple open PRs",
            "draft, not ready",
            "Claim: none): #14",
            "no valid Claim header; may be legacy): #15",
            "claim not in this checkout",
            "PR #99 is not a linked open PR (stale pointer)",
        ):
            self.assertIn(part, out)
        lines, pending = mod.review_projection(items, None, "offline")
        self.assertEqual(pending, set())
        self.assertIn("UNAVAILABLE: offline", lines[0])
        self.assertIn("stored PR #99 (unverified)", "\n".join(lines))
        lines, pending = mod.review_projection(items, [{"number": 1}], None)
        self.assertEqual(pending, set())
        self.assertIn("UNAVAILABLE", lines[0])

    def test_review_projection_excludes_linked_open_claim_without_hold(self) -> None:
        """An expired or absent lease must not make a PR's claim look like new work."""
        a = self.sb.clone("reader")
        mod = _load_frontier_module(a)
        pr = {
            "number": 42,
            "body": "Claim: c1",
            "head": {"sha": "b" * 40},
            "base": {"ref": "master"},
            "state": "open",
            "draft": False,
        }
        with mock.patch.object(mod, "read_open_prs", return_value=([pr], None)):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                mod.brief({"shell"})
        take = buf.getvalue().split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("**c1**", take)
        self.assertIn("**c2**", take)

    def test_review_discovery_paginates_without_gh(self) -> None:
        """The standard-library fallback reads every page, or reports failure."""
        a = self.sb.clone("reader")
        mod = _load_frontier_module(a)
        page1 = [{"number": n} for n in range(1, 101)]
        page2 = [{"number": 101}]
        with (
            mock.patch.object(mod, "github_repository", return_value="test-owner/test-repo"),
            mock.patch.object(mod.shutil, "which", return_value=None),
            mock.patch.object(
                mod.urllib.request,
                "urlopen",
                side_effect=[
                    io.BytesIO(json.dumps(page1).encode()),
                    io.BytesIO(json.dumps(page2).encode()),
                ],
            ) as request,
        ):
            prs, error = mod.read_open_prs()
        self.assertIsNone(error)
        self.assertEqual(prs, page1 + page2)
        self.assertIn("page=2", request.call_args.args[0].full_url)

    def test_review_lineage_gate_checks_frontier_pr_event(self) -> None:
        """CI rejects missing or mismatched existing Claim lineage, not ordinary legacy PRs."""
        a = self.sb.clone("author")
        base = _git(a, "rev-parse", "HEAD")
        mod = _load_frontier_module(a)
        event_path = a / "event.json"
        pr = {"number": 42, "base": {"sha": base}, "body": None}
        event_path.write_text(json.dumps({"pull_request": pr}))
        self.assertIsNone(mod.check_pr_lineage(event_path, mod.load("claims")))

        card = a / "frontier/claims/c1.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "review"\npr = 42'))
        _git(a, "add", str(card))
        _git(a, "commit", "-m", "work awaiting review")
        items = mod.load("claims")
        self.assertIn("Claim:", mod.check_pr_lineage(event_path, items))
        pr["body"] = "Claim: missing"
        event_path.write_text(json.dumps({"pull_request": pr}))
        self.assertIn("no claim", mod.check_pr_lineage(event_path, items))
        pr["body"] = "Claim: c2"
        event_path.write_text(json.dumps({"pull_request": pr}))
        self.assertIn("does not match", mod.check_pr_lineage(event_path, items))
        pr["body"] = "Claim: c1"
        event_path.write_text(json.dumps({"pull_request": pr}))
        self.assertIsNone(mod.check_pr_lineage(event_path, items))


    def _submitted_review(self):
        """Fill the canary and publish an exact submitted claim in the sandbox."""
        author = self.sb.clone("producer")
        sessions = {}
        for cid in ("c1", "c2", "c3"):
            result = self.sb.run_frontier(author, "claim", cid, "--by", "producer")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            sessions[cid] = _extract_session(result.stdout)
        _git(author, "switch", "-c", "submitted")
        card = author / "frontier/claims/c1.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "review"\npr = 42'))
        _git(author, "add", str(card))
        _git(author, "commit", "-m", "submit result for integration")
        _git(author, "push", "origin", "HEAD:refs/pull/42/head")
        head = _git(author, "rev-parse", "HEAD")
        pr = {
            "number": 42,
            "body": "Claim: c1",
            "head": {"sha": head},
            "base": {"ref": "master"},
            "state": "open",
            "draft": False,
            "merged": False,
        }
        return author, sessions["c1"], head, pr

    def test_handoff_at_saturation_and_recover_after_merge(self):
        """The same reservation transfers at cap; completion survives the reviewer."""
        author, old_session, head, pr = self._submitted_review()
        fresh = self.sb.clone("reviewer")
        mod = _load_frontier_module(fresh)
        with mock.patch.object(mod, "read_pr", return_value=pr):
            mod.take_review("c1", 42, head, "reviewer", 1)
            holds = mod.holds()
            lease = holds["c1"][1]
            self.assertEqual(sum(mod.live(v) for _, v in holds.values()), 3)
            self.assertEqual(lease["phase"], "integration")
            self.assertEqual(lease["producer"], "producer")
            with self.assertRaises(SystemExit):
                mod.mine("c1", old_session)
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 42, head, "other-reviewer", 1)
            self.assertEqual(mod.main(["frontier", "verify", "c1", "--session", lease["session"]]), 0)
            with self.assertRaises(SystemExit):
                mod.finish_review("c1", 42, head)

        # Simulate an exact, verified merge to trunk and the reviewer disappearing.
        _git(author, "push", "origin", "HEAD:master")
        merged = {**pr, "state": "closed", "merged": True, "merge_commit_sha": head}
        newcomer = _load_frontier_module(self.sb.clone("newcomer"))
        with mock.patch.object(newcomer, "read_pr", return_value=merged):
            newcomer.finish_review("c1", 42, head)
            newcomer.finish_review("c1", 42, head)  # retry is harmless
        self.assertTrue(newcomer.holds()["c1"][1]["released"])
        self.assertEqual(sum(newcomer.live(v) for _, v in newcomer.holds().values()), 2)
        result = self.sb.run_frontier(fresh, "claim", "c4", "--by", "newcomer")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_handoff_head_change_and_yield(self):
        """A changed PR head invalidates authority; yielding lets a fresh participant take it."""
        author, _, head, pr = self._submitted_review()
        mod = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(mod, "read_pr", return_value=pr):
            mod.take_review("c1", 42, head, "reviewer", 1)
        session = mod.holds()["c1"][1]["session"]
        _git(author, "commit", "--allow-empty", "-m", "change submitted head")
        _git(author, "push", "origin", "HEAD:refs/pull/42/head")
        new_head = _git(author, "rev-parse", "HEAD")
        changed = {**pr, "head": {"sha": new_head}}
        with mock.patch.object(mod, "read_pr", return_value=changed):
            with self.assertRaises(SystemExit):
                mod.main(["frontier", "verify", "c1", "--session", session])
            mod.yield_review("c1", session)
            with self.assertRaises(SystemExit):
                mod.mine("c1", session)
            mod.take_review("c1", 42, new_head, "fresh-reviewer", 1)
        self.assertEqual(mod.holds()["c1"][1]["pr_head"], new_head)

    def test_handoff_rejects_unsubmitted_and_same_producer(self):
        """A draft or a mere PR header is not a surrendered work reservation."""
        author, _, head, pr = self._submitted_review()
        mod = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(mod, "read_pr", return_value={**pr, "draft": True}):
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 42, head, "reviewer", 1)
        with mock.patch.object(mod, "read_pr", return_value=pr):
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 42, head, "producer", 1)
        card = author / "frontier/claims/c1.toml"
        card.write_text(card.read_text().replace('status = "review"', 'status = "open"'))
        _git(author, "add", str(card))
        _git(author, "commit", "-m", "withdraw review declaration")
        _git(author, "push", "origin", "HEAD:refs/pull/42/head")
        new_head = _git(author, "rev-parse", "HEAD")
        with mock.patch.object(mod, "read_pr", return_value={**pr, "head": {"sha": new_head}}):
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 42, new_head, "reviewer", 1)
        self.assertEqual(mod.holds()["c1"][1]["holder"], "producer")

    def test_handoff_cas_race_and_closed_unmerged_cleanup(self):
        """Two independent contenders cannot both take the same reservation."""
        _, _, head, pr = self._submitted_review()
        mods = [_load_frontier_module(self.sb.clone(f"reviewer-{i}")) for i in range(2)]
        barrier = threading.Barrier(2)

        def attempt(i):
            mod = mods[i]
            original = mod.write_lease

            def write(*args):
                barrier.wait(timeout=10)
                return original(*args)

            with (
                mock.patch.object(mod, "read_pr", return_value=pr),
                mock.patch.object(mod, "write_lease", side_effect=write),
            ):
                try:
                    mod.take_review("c1", 42, head, f"reviewer-{i}", 1)
                    return True
                except SystemExit:
                    return False

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(attempt, range(2)))
        self.assertEqual(sum(results), 1)
        mod = mods[0]
        closed = {**pr, "state": "closed"}
        with mock.patch.object(mod, "read_pr", return_value=closed):
            with self.assertRaises(SystemExit):
                mod.finish_review("c1", 42, head)
            mod.finish_review("c1", 42, head, cancelled=True)
            mod.finish_review("c1", 42, head, cancelled=True)
        self.assertTrue(mod.holds()["c1"][1]["released"])


    def test_atomic_admission_distinct_ids_at_last_slot(self):
        """Distinct claims cannot both acquire the last slot."""
        seed = self.sb.clone("capacity-seed")
        for cid in ("c1", "c2"):
            result = self.sb.run_frontier(seed, "claim", cid, "--by", "seed")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        mods = [_load_frontier_module(self.sb.clone(f"contender-{i}")) for i in range(2)]
        barrier = threading.Barrier(2)

        def attempt(i):
            mod = mods[i]
            original = mod.write_lease

            def write(*args):
                barrier.wait(timeout=10)
                return original(*args)

            with mock.patch.object(mod, "write_lease", side_effect=write):
                try:
                    mod.claim(("c3", "c4")[i], f"agent-{i}", 1)
                    return True
                except SystemExit:
                    return False

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(attempt, range(2)))
        self.assertEqual(sum(results), 1)
        self.assertEqual(sum(mods[0].live(v) for _, v in mods[0].holds().values()), 3)

    def test_operational_limit_refresh_without_worktree_merge(self):
        """An existing updated client reads new settings from trunk, not its worktree."""
        seed = self.sb.seed
        fr = seed / "frontier"
        (fr / "config.toml").write_text(
            BASE_CONFIG.replace("max_live_holds = 3", 'limits_file = "limits.toml"\nmax_live_holds = 0')
        )
        (fr / "limits.toml").write_text("max_live_holds = 1\n")
        _git(seed, "add", "frontier")
        _git(seed, "commit", "-m", "separate operational limit")
        _git(seed, "push", "origin", "master")
        author = self.sb.clone("limits-author")
        first = self.sb.run_frontier(author, "claim", "c1", "--by", "author")
        self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
        blocked = self.sb.run_frontier(author, "claim", "c2", "--by", "author")
        self.assertNotEqual(blocked.returncode, 0)
        (fr / "limits.toml").write_text("max_live_holds = 2\n")
        _git(seed, "commit", "-am", "owner raises operational limit")
        _git(seed, "push", "origin", "master")
        second = self.sb.run_frontier(author, "claim", "c2", "--by", "author")
        self.assertEqual(second.returncode, 0, second.stdout + second.stderr)
        self.assertEqual((author / "frontier/limits.toml").read_text(), "max_live_holds = 1\n")

    def test_operational_limit_invalid_and_missing_refuse(self):
        """Never use a permissive default if the authoritative setting is absent or invalid."""
        seed = self.sb.seed
        fr = seed / "frontier"
        (fr / "config.toml").write_text(
            BASE_CONFIG.replace("max_live_holds = 3", 'limits_file = "limits.toml"\nmax_live_holds = 0')
        )
        limit = fr / "limits.toml"
        limit.write_text("max_live_holds = 0\n")
        _git(seed, "add", "frontier")
        _git(seed, "commit", "-m", "invalid limit")
        _git(seed, "push", "origin", "master")
        author = self.sb.clone("invalid-limit")
        for value in ("0", "true", '"10"', "-1"):
            limit.write_text(f"max_live_holds = {value}\n")
            _git(seed, "add", "frontier")
            if subprocess.run(["git", "-C", str(seed), "diff", "--cached", "--quiet"]).returncode:
                _git(seed, "commit", "-m", "invalid limit value")
                _git(seed, "push", "origin", "master")
            result = self.sb.run_frontier(author, "claim", "c1", "--by", "author")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("cannot read reservation limit", result.stderr)
        _git(seed, "rm", "frontier/limits.toml")
        _git(seed, "commit", "-m", "missing limit")
        _git(seed, "push", "origin", "master")
        result = self.sb.run_frontier(author, "claim", "c1", "--by", "author")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing authoritative frontier/limits.toml", result.stderr)

    def test_ten_slot_admission_and_reuse(self):
        """At the proposed ceiling, racing distinct IDs cannot overshoot; release reopens a slot."""
        seed = self.sb.seed
        config_path = seed / "frontier" / "config.toml"
        config_path.write_text(
            config_path.read_text().replace("max_live_holds = 3", "max_live_holds = 10")
        )
        for i in range(12):
            cid = f"cap-ten-{i}"
            (seed / "frontier" / "claims" / f"{cid}.toml").write_text(_claim_toml(cid))
        _git(seed, "add", "frontier")
        _git(seed, "commit", "-m", "ten-slot fixture")
        _git(seed, "push", "origin", "master")
        author = self.sb.clone("ten-author")
        for i in range(9):
            result = self.sb.run_frontier(author, "claim", f"cap-ten-{i}", "--by", "seed")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        mods = [_load_frontier_module(self.sb.clone(f"ten-racer-{i}")) for i in range(2)]
        barrier = threading.Barrier(2)

        def attempt(i):
            mod = mods[i]
            original = mod.write_lease

            def write(*args):
                barrier.wait(timeout=10)
                return original(*args)

            with mock.patch.object(mod, "write_lease", side_effect=write):
                try:
                    mod.claim(f"cap-ten-{9 + i}", f"agent-{i}", 1)
                    return True
                except SystemExit:
                    return False

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(attempt, range(2)))
        self.assertEqual(sum(results), 1)
        self.assertEqual(sum(mods[0].live(v) for _, v in mods[0].holds().values()), 10)
        full = self.sb.run_frontier(author, "claim", "cap-ten-11", "--by", "seed")
        self.assertNotEqual(full.returncode, 0)
        lease = mods[0].holds()["cap-ten-0"][1]
        release = self.sb.run_frontier(
            author, "release", "cap-ten-0", "--session", lease["session"]
        )
        self.assertEqual(release.returncode, 0, release.stdout + release.stderr)
        result = self.sb.run_frontier(author, "claim", "cap-ten-11", "--by", "seed")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(sum(mods[0].live(v) for _, v in mods[0].holds().values()), 10)

    def test_expired_producer_cannot_revive_at_capacity(self):
        _, _, head, pr = self._submitted_review()
        mod = _load_frontier_module(self.sb.clone("reviewer"))
        sha, lease = mod.holds()["c1"]
        expired = {**lease, "expires": "2000-01-01T00:00:00+00:00"}
        self.assertTrue(mod.write_lease("c1", expired, sha, sha)[0])
        mod.claim("c4", "fresh", 1)
        with mock.patch.object(mod, "read_pr", return_value=pr):
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 42, head, "reviewer", 1)
        self.assertEqual(sum(mod.live(v) for _, v in mod.holds().values()), 3)

    def test_next_acquisition_recovers_closed_unmerged_integration(self):
        _, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)
        closed = {**pr, "state": "closed", "merged": False}
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=closed):
            fresh.claim("c4", "fresh", 1)
        lease = fresh.holds()["c1"][1]
        self.assertTrue(lease["released"])
        self.assertEqual(lease["terminal"], "closed-unmerged")

    def test_dropped_claim_releases_rebased_closed_unmerged_parked_hold(self):
        """A parked lease whose PR was rebased after handoff still releases once the claim is
        dropped on master and the PR is closed unmerged; a live (undropped) claim does not."""
        self._rebased_parked_hold_releases_when("dropped")

    def test_blocked_claim_releases_rebased_closed_unmerged_parked_hold(self):
        """The second stranding mode: the claim is terminal as `blocked`, not dropped."""
        self._rebased_parked_hold_releases_when("blocked")

    def _rebased_parked_hold_releases_when(self, status):
        author, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                reviewer.take_review("c1", 42, head, "reviewer", 1)
        reviewer.yield_review("c1", _extract_session(out.getvalue()))
        _git(author, "commit", "--allow-empty", "-m", "rebased")
        moved = _git(author, "rev-parse", "HEAD")
        closed = {**pr, "head": {"sha": moved}, "state": "closed", "merged": False}
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=closed):
            with self.assertRaises(SystemExit):
                fresh.finish_review("c1", 42, moved, cancelled=True)
            fresh.reconcile_integrations(fresh.holds())
        self.assertFalse(fresh.holds()["c1"][1].get("released"))
        card = author / "frontier/claims/c1.toml"
        card.write_text(card.read_text().replace('status = "review"', f'status = "{status}"'))
        _git(author, "commit", "-am", f"mark c1 {status}")
        _git(author, "push", "origin", "HEAD:master")
        _git(fresh.ROOT.parent, "fetch", "-q", "origin", "+refs/heads/master:refs/remotes/origin/master")
        with mock.patch.object(fresh, "read_pr", return_value=closed):
            with self.assertRaises(SystemExit):
                fresh.finish_review("c1", 42, moved)
            fresh.reconcile_integrations(fresh.holds())
        lease = fresh.holds()["c1"][1]
        self.assertTrue(lease["released"])
        self.assertEqual(lease["terminal"], "closed-unmerged")

    def test_next_acquisition_recovers_abandoned_merged_integration(self):
        """A new swarm iteration need not ask the old reviewer to release a merged PR."""
        author, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)
        _git(author, "push", "origin", "HEAD:master")
        merged = {**pr, "state": "closed", "merged": True, "merge_commit_sha": head}
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=merged):
            fresh.claim("c4", "fresh", 1)
        holds = fresh.holds()
        self.assertTrue(holds["c1"][1]["released"])
        self.assertTrue(fresh.live(holds["c4"][1]))
        self.assertEqual(sum(fresh.live(v) for _, v in holds.values()), 3)

    def test_merged_integration_hold_at_descendant_head_releases_via_reconcile(self):
        """Reconciling a merged integration hold whose head moved forward from pr_head releases it."""
        author, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)

        # Advance master independently, then merge master into PR #42 (moving head -> moved)
        # and squash-merge onto master (creating an independent merge_sha on origin/master).
        seed = self.sb.seed
        _git(seed, "commit", "--allow-empty", "-m", "advance master during review")
        _git(seed, "push", "origin", "master")
        _git(author, "fetch", "-q", "origin", "master")
        _git(author, "merge", "-q", "--no-edit", "origin/master")
        moved = _git(author, "rev-parse", "HEAD")
        self.assertNotEqual(moved, head)
        _git(author, "push", "-q", "origin", f"{moved}:refs/pull/42/head")

        self._squash_marking_done(seed)
        merge_sha = _git(seed, "rev-parse", "HEAD")

        merged = {
            **pr,
            "head": {"sha": moved},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }
        # A --no-local clone copies only branches, so the moved head is reachable only through
        # refs/pull/42/head: reconciliation must fetch it to prove ancestry, as in production.
        dst = self.sb.tmp / "fresh"
        _git(self.sb.tmp, "clone", "-q", "--no-local", str(self.sb.remote), str(dst))
        _git(dst, "config", "--local", "user.name", GIT_ENV["GIT_AUTHOR_NAME"])
        _git(dst, "config", "--local", "user.email", GIT_ENV["GIT_AUTHOR_EMAIL"])
        self.assertNotEqual(subprocess.run(
            ["git", "cat-file", "-e", f"{moved}^{{commit}}"], cwd=dst, capture_output=True
        ).returncode, 0)
        fresh = _load_frontier_module(dst)
        with mock.patch.object(fresh, "read_pr", return_value=merged):
            # Passing the stale pre-merge head still fails closed.
            with self.assertRaises(SystemExit):
                fresh.finish_review("c1", 42, head)
            # Reconcile fetches refs/pull/42/head, verifies head -> moved ancestry, and releases.
            fresh.reconcile_integrations(fresh.holds())
            lease = fresh.holds()["c1"][1]
            self.assertTrue(lease["released"])
            self.assertEqual(lease["terminal"], merge_sha)
            self.assertEqual(lease["moved_head"], [head, moved])
            # Direct finish_review on the moved head is idempotent once released.
            fresh.finish_review("c1", 42, moved)

    def test_merged_integration_hold_at_descendant_head_releases_via_finish_review(self):
        """Direct `finish_review` releases a merged moved-head integration reservation."""
        author, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)
        _git(author, "commit", "--allow-empty", "-m", "integrator polish before squash")
        moved = _git(author, "rev-parse", "HEAD")
        _git(author, "push", "-q", "origin", f"{moved}:refs/pull/42/head")
        self._squash_marking_done(self.sb.seed)
        merge_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        merged = {
            **pr,
            "head": {"sha": moved},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=merged):
            fresh.finish_review("c1", 42, moved)
        lease = fresh.holds()["c1"][1]
        self.assertTrue(lease["released"])
        self.assertEqual(lease["terminal"], merge_sha)
        self.assertEqual(lease["moved_head"], [head, moved])

    def test_merged_integration_hold_at_unrelated_head_refuses_release(self):
        """Fail closed when a merged PR head does not descend from the recorded pr_head."""
        _, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)

        # Create an unrelated branch off master (does NOT have `head` as an ancestor).
        seed = self.sb.seed
        _git(seed, "checkout", "-q", "-b", "unrelated-pr")
        _git(seed, "commit", "--allow-empty", "-m", "unrelated replacement head")
        unrelated = _git(seed, "rev-parse", "HEAD")
        _git(seed, "push", "-q", "origin", f"+{unrelated}:refs/pull/42/head")
        _git(seed, "checkout", "-q", "master")
        self._squash_marking_done(seed)
        merge_sha = _git(seed, "rev-parse", "HEAD")

        merged = {
            **pr,
            "head": {"sha": unrelated},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=merged):
            with self.assertRaises(SystemExit):
                fresh.finish_review("c1", 42, unrelated)
            fresh.reconcile_integrations(fresh.holds())
        self.assertFalse(fresh.holds()["c1"][1].get("released"))

    def _squash_marking_done(self, seed):
        """Land a squash of PR #42 on master that marks c1 done, as an integrator's merge does."""
        card = seed / "frontier/claims/c1.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "done"\npr = 42'))
        _git(seed, "commit", "-qam", "squash merge PR #42")
        _git(seed, "push", "origin", "master")

    def test_merged_integration_hold_at_descendant_head_retained_while_claim_live(self):
        """A moved-head merge does not release while the claim is not terminal on master."""
        author, _, head, pr = self._submitted_review()
        reviewer = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(reviewer, "read_pr", return_value=pr):
            reviewer.take_review("c1", 42, head, "reviewer", 1)
        _git(author, "commit", "--allow-empty", "-m", "integrator polish before squash")
        moved = _git(author, "rev-parse", "HEAD")
        _git(author, "push", "-q", "origin", f"{moved}:refs/pull/42/head")
        _git(self.sb.seed, "commit", "--allow-empty", "-m", "squash merge PR #42, card not closed")
        _git(self.sb.seed, "push", "origin", "master")
        merge_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        merged = {
            **pr,
            "head": {"sha": moved},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }
        fresh = _load_frontier_module(self.sb.clone("fresh"))
        with mock.patch.object(fresh, "read_pr", return_value=merged):
            with self.assertRaises(SystemExit):
                fresh.finish_review("c1", 42, moved)
            fresh.reconcile_integrations(fresh.holds())
        self.assertFalse(fresh.holds()["c1"][1].get("released"))

    def _stale_docs_team_pair(self):
        """Set up archive-team-strata holding docs/team/ and migrate-rules needing docs/team/RULES.md."""
        seed = self.sb.seed
        fr = seed / "frontier" / "claims"
        (fr / "archive-team-strata.toml").write_text(
            _claim_toml("archive-team-strata", zone="striated", owned_paths=["docs/team/"])
        )
        (fr / "migrate-rules.toml").write_text(
            _claim_toml("migrate-rules", zone="striated", owned_paths=["docs/team/RULES.md"])
        )
        _git(seed, "add", "frontier/claims")
        _git(seed, "commit", "-m", "add archive-team-strata and migrate-rules fixtures")
        _git(seed, "push", "origin", "master")
        prior_master = _git(seed, "rev-parse", "HEAD")

        # Advance master once so prior_master strictly predates the lease's base.
        _git(seed, "commit", "--allow-empty", "-m", "advance master before claim")
        _git(seed, "push", "origin", "master")

        holder = self.sb.clone("holder")
        res = self.sb.run_frontier(holder, "claim", "archive-team-strata", "--by", "holder")
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        session = _extract_session(res.stdout)
        hold_sha = _git(self.sb.remote, "rev-parse", "refs/heads/hold/archive-team-strata")
        return holder, session, hold_sha, prior_master

    def test_stale_ordinary_hold_reconciles_on_verified_merged_pr_and_unblocks_overlap(self):
        """A done ordinary hold with a verified merged PR on master reconciles and unblocks overlap."""
        holder, old_session, old_hold_sha, _ = self._stale_docs_team_pair()
        fresh_repo = self.sb.clone("fresh")

        # While still open, migrate-rules is refused due to docs/team/ overlap.
        blocked = self.sb.run_frontier(fresh_repo, "claim", "migrate-rules", "--by", "fresh")
        self.assertNotEqual(blocked.returncode, 0)
        self.assertIn("would share", blocked.stderr)
        self.assertIn("archive-team-strata", blocked.stderr)

        # Merge PR #977 to origin/master marking archive-team-strata done with pr = 977; forget release.
        card = holder / "frontier" / "claims" / "archive-team-strata.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "done"\npr = 977'))
        _git(holder, "add", str(card))
        _git(holder, "commit", "-m", "finish archive-team-strata (#977)")
        _git(holder, "push", "origin", "HEAD:master")
        merge_sha = _git(holder, "rev-parse", "HEAD")

        # Without GitHub PR verification (API unavailable), claim fails closed and retains the lease.
        unverified = self.sb.run_frontier(fresh_repo, "claim", "migrate-rules", "--by", "fresh")
        self.assertNotEqual(unverified.returncode, 0)
        self.assertIn("would share", unverified.stderr)
        fresh = _load_frontier_module(fresh_repo)
        self.assertFalse(fresh.holds()["archive-team-strata"][1]["released"])

        # With verified merged PR #977 on origin/master, claim reconciles and acquires migrate-rules.
        merged_pr = {
            "number": 977,
            "body": "Claim: archive-team-strata\n\nArchived coordinator-era machinery.",
            "base": {"ref": "master"},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }
        with mock.patch.object(fresh, "read_pr", return_value=merged_pr):
            fresh.claim("migrate-rules", "fresh", 1)

        all_holds = fresh.holds()
        rel_sha, rel_lease = all_holds["archive-team-strata"]
        self.assertTrue(rel_lease["released"])
        self.assertEqual(rel_lease["terminal"], merge_sha)
        self.assertEqual(
            _git(self.sb.remote, "rev-parse", f"{rel_sha}^"),
            old_hold_sha,
            "reconciliation release commit must chain from the previous hold commit",
        )
        self.assertTrue(fresh.live(all_holds["migrate-rules"][1]))
        # Forgetful holder can no longer verify the reconciled lease.
        old_verify = self.sb.run_frontier(
            holder, "verify", "archive-team-strata", "--session", old_session
        )
        self.assertNotEqual(old_verify.returncode, 0)

    def test_stale_ordinary_hold_dropped_claim_reconciles_at_capacity_with_atomic_cas(self):
        """Dropped ordinary holds with verified merged PRs free capacity without CAS overshoot."""
        seed = self.sb.seed
        (seed / "frontier" / "claims" / "c5.toml").write_text(_claim_toml("c5"))
        _git(seed, "add", "frontier/claims/c5.toml")
        _git(seed, "commit", "-m", "add c5 claim")
        _git(seed, "push", "origin", "master")

        holder = self.sb.clone("holder")
        for cid in ("c1", "c2", "c3"):
            res = self.sb.run_frontier(holder, "claim", cid, "--by", "holder")
            self.assertEqual(res.returncode, 0, res.stdout + res.stderr)

        # Merge PR #55 dropping c1 on origin/master without releasing hold/c1.
        card = holder / "frontier" / "claims" / "c1.toml"
        card.write_text(card.read_text().replace('status = "open"', 'status = "dropped"\npr = 55'))
        _git(holder, "add", str(card))
        _git(holder, "commit", "-m", "drop c1 (#55)")
        _git(holder, "push", "origin", "HEAD:master")
        merge_sha = _git(holder, "rev-parse", "HEAD")

        merged_pr = {
            "number": 55,
            "body": "Claim: c1\n\nDropped after evidence.",
            "base": {"ref": "master"},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }

        # Two contenders race for the single slot freed by reconciling c1.
        mods = [_load_frontier_module(self.sb.clone(f"racer-{i}")) for i in range(2)]
        barrier = threading.Barrier(2)

        def attempt(i: int) -> bool:
            mod = mods[i]
            original = mod.write_lease

            def guarded_write(cid, lease, parent, expected, admission=None):
                if admission is not None:
                    barrier.wait(timeout=10)
                return original(cid, lease, parent, expected, admission)

            with (
                mock.patch.object(mod, "read_pr", return_value=merged_pr),
                mock.patch.object(mod, "write_lease", side_effect=guarded_write),
            ):
                try:
                    mod.claim(("c4", "c5")[i], f"agent-{i}", 1)
                    return True
                except SystemExit:
                    return False

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(attempt, range(2)))

        self.assertEqual(sum(results), 1)
        all_holds = mods[0].holds()
        self.assertTrue(all_holds["c1"][1]["released"])
        self.assertEqual(all_holds["c1"][1]["terminal"], merge_sha)
        self.assertEqual(sum(mods[0].live(v) for _, v in all_holds.values()), 3)

    def test_stale_ordinary_hold_fails_closed_on_uncertainty(self):
        """Never release an ordinary hold when status, pr, API, header, base, or merge is uncertain."""
        holder, _, _, prior_master = self._stale_docs_team_pair()
        fresh_repo = self.sb.clone("fresh")
        fresh = _load_frontier_module(fresh_repo)

        # Create an unmerged side-branch commit (not on origin/master).
        _git(holder, "checkout", "-b", "unmerged-side")
        _git(holder, "commit", "--allow-empty", "-m", "unmerged side commit")
        unmerged_sha = _git(holder, "rev-parse", "HEAD")
        _git(holder, "push", "origin", "HEAD:refs/heads/unmerged-side")
        _git(holder, "checkout", "master")

        def push_card(replacement: str) -> str:
            card = holder / "frontier" / "claims" / "archive-team-strata.toml"
            text = _claim_toml(
                "archive-team-strata", zone="striated", owned_paths=["docs/team/"]
            ).replace('status = "open"', replacement)
            card.write_text(text)
            _git(holder, "add", str(card))
            _git(holder, "commit", "-m", f"update card: {replacement.splitlines()[0]}")
            _git(holder, "push", "origin", "HEAD:master")
            return _git(holder, "rev-parse", "HEAD")

        # 1. Local-only done edit must not override origin/master status = "open".
        local_card = fresh_repo / "frontier" / "claims" / "archive-team-strata.toml"
        local_card.write_text(
            local_card.read_text().replace('status = "open"', 'status = "done"\npr = 977')
        )
        with mock.patch.object(fresh, "read_pr", side_effect=AssertionError("must not query PR")):
            with self.assertRaises(SystemExit):
                fresh.claim("migrate-rules", "fresh", 1)
        _git(fresh_repo, "checkout", "--", str(local_card))

        # 2. Non-terminal status ("review") with pr = 977 must not release or query PR.
        push_card('status = "review"\npr = 977')
        with mock.patch.object(fresh, "read_pr", side_effect=AssertionError("must not query PR")):
            with self.assertRaises(SystemExit):
                fresh.claim("migrate-rules", "fresh", 1)

        # 3. Terminal status ("done") with missing pr or pr = 0 must not release or query PR.
        for bad_pr in ('status = "done"', 'status = "done"\npr = 0', 'status = "done"\npr = true'):
            push_card(bad_pr)
            with mock.patch.object(
                fresh, "read_pr", side_effect=AssertionError("must not query PR")
            ):
                with self.assertRaises(SystemExit):
                    fresh.claim("migrate-rules", "fresh", 1)

        # 4. Terminal status with pr = 977: reject API error and every invalid/unverified PR payload.
        merge_sha = push_card('status = "done"\npr = 977')
        valid_pr = {
            "number": 977,
            "body": "Claim: archive-team-strata",
            "base": {"ref": "master"},
            "state": "closed",
            "merged": True,
            "merge_commit_sha": merge_sha,
        }

        with mock.patch.object(fresh, "read_pr", side_effect=SystemExit("API unavailable")):
            with self.assertRaises(SystemExit):
                fresh.claim("migrate-rules", "fresh", 1)

        invalid_payloads = [
            {**valid_pr, "state": "open", "merged": False},
            {**valid_pr, "state": "closed", "merged": False, "merge_commit_sha": None},
            {**valid_pr, "body": "Claim: migrate-rules"},
            {**valid_pr, "body": "Claim: none"},
            {**valid_pr, "base": {"ref": "other-branch"}},
            {**valid_pr, "number": 978},
            {**valid_pr, "merge_commit_sha": "not-a-sha"},
            {**valid_pr, "merge_commit_sha": unmerged_sha},
            {**valid_pr, "merge_commit_sha": prior_master},
        ]
        for payload in invalid_payloads:
            with mock.patch.object(fresh, "read_pr", return_value=payload):
                with self.assertRaises(SystemExit):
                    fresh.claim("migrate-rules", "fresh", 1)
            self.assertFalse(
                fresh.holds()["archive-team-strata"][1]["released"],
                f"must fail closed for payload: {payload}",
            )

    def _released_pr_handoff(
        self,
        cid: str = "run-a",
        *,
        status: str = "open",
        number: int = 42,
        include_pr: bool = True,
        release: bool = True,
        expire: bool = False,
    ):
        """Claim `cid`, open a PR with the given head card, and release or expire the hold."""
        author = self.sb.clone(f"producer-{cid}-{number}")
        res = self.sb.run_frontier(author, "claim", cid, "--by", "producer")
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        session = _extract_session(res.stdout)
        _git(author, "switch", "-c", f"handed-back-{cid}-{number}")
        card = author / "frontier" / "claims" / f"{cid}.toml"
        replacement = f'status = "{status}"' + (f"\npr = {number}" if include_pr else "")
        card.write_text(card.read_text().replace('status = "open"', replacement, 1))
        _git(author, "add", str(card))
        _git(author, "commit", "--allow-empty", "-m", f"hand back {cid} on PR #{number}")
        _git(author, "push", "origin", f"HEAD:refs/pull/{number}/head")
        head = _git(author, "rev-parse", "HEAD")
        if release:
            rel = self.sb.run_frontier(author, "release", cid, "--session", session)
            self.assertEqual(rel.returncode, 0, rel.stdout + rel.stderr)
        elif expire:
            mod = _load_frontier_module(author)
            sha, lease = mod.holds()[cid]
            expired = {**lease, "expires": "2000-01-01T00:00:00+00:00"}
            self.assertTrue(mod.write_lease(cid, expired, sha, sha)[0])
        pr = {
            "number": number,
            "body": f"Claim: {cid}",
            "head": {"sha": head},
            "base": {"ref": "master"},
            "state": "open",
            "draft": False,
            "merged": False,
        }
        return author, head, pr

    def test_resume_released_pr_claim_open_and_review_head_cards(self) -> None:
        """Released or expired holds with an open/review PR card can be resumed via take-review."""
        author, head, pr = self._released_pr_handoff("run-a", status="open", number=42)
        mod = _load_frontier_module(self.sb.clone("reviewer"))

        # `claim` still refuses because an open PR already exists for the claim.
        with (
            mock.patch.object(mod, "github_repository", return_value="test-owner/test-repo"),
            mock.patch.object(mod, "read_open_prs", return_value=([pr], None)),
        ):
            with self.assertRaises(SystemExit) as cm:
                mod.claim("run-a", "reviewer", 1)
            self.assertIn("already has a pending PR", str(cm.exception))

        # `take_review` acquires a fresh integration lease on the exact PR head.
        with mock.patch.object(mod, "read_pr", return_value=pr):
            mod.take_review("run-a", 42, head, "reviewer", 1)
            lease = mod.holds()["run-a"][1]
            self.assertFalse(lease["released"])
            self.assertTrue(mod.live(lease))
            self.assertEqual(lease["phase"], "integration")
            self.assertEqual(lease["holder"], "reviewer")
            self.assertEqual(lease["producer"], "producer")
            self.assertEqual(lease["pr"], 42)
            self.assertEqual(lease["pr_head"], head)
            self.assertEqual(lease["owned_paths"], ["bench/workstation-queue.txt"])
            self.assertEqual(lease["mechanisms"], ["queue"])
            self.assertEqual(
                mod.main(["frontier", "verify", "run-a", "--session", lease["session"]]),
                0,
            )

            # Yielding and handing off to a second independent reviewer still works on an open card.
            mod.yield_review("run-a", lease["session"])
            mod.take_review("run-a", 42, head, "reviewer-2", 1)
            self.assertEqual(mod.holds()["run-a"][1]["holder"], "reviewer-2")

        # Terminal merge recovery releases the resumed integration lease cleanly.
        _git(author, "push", "origin", "HEAD:master")
        merged = {**pr, "state": "closed", "merged": True, "merge_commit_sha": head}
        with mock.patch.object(mod, "read_pr", return_value=merged):
            mod.finish_review("run-a", 42, head)
        self.assertTrue(mod.holds()["run-a"][1]["released"])

        # Released lease with a `review` head card also resumes cleanly.
        _, head_c1, pr_c1 = self._released_pr_handoff("c1", status="review", number=43)
        with mock.patch.object(mod, "read_pr", return_value=pr_c1):
            mod.take_review("c1", 43, head_c1, "reviewer", 1)
        self.assertEqual(mod.holds()["c1"][1]["phase"], "integration")
        self.assertEqual(mod.holds()["c1"][1]["pr_head"], head_c1)

        # Expired (unreleased) producer lease with an `open` head card also resumes cleanly.
        _, head_c2, pr_c2 = self._released_pr_handoff(
            "c2", status="open", number=44, release=False, expire=True
        )
        with mock.patch.object(mod, "read_pr", return_value=pr_c2):
            mod.take_review("c2", 44, head_c2, "reviewer", 1)
        self.assertEqual(mod.holds()["c2"][1]["phase"], "integration")
        self.assertEqual(mod.holds()["c2"][1]["pr_head"], head_c2)

    def test_resume_released_pr_claim_enforces_head_pr_producer_and_ancestry(self) -> None:
        """Resuming a released PR still enforces producer-differs, exact head, PR match, and base."""
        author, head, pr = self._released_pr_handoff("run-a", status="open", number=42)
        mod = _load_frontier_module(self.sb.clone("reviewer"))

        with mock.patch.object(mod, "read_pr", return_value=pr):
            # Producer cannot self-certify its own released PR.
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("run-a", 42, head, "producer", 1)
            self.assertIn("different from the producer", str(cm.exception))

            # Wrong PR head SHA is refused.
            with self.assertRaises(SystemExit):
                mod.take_review("run-a", 42, "0" * 40, "reviewer", 1)

        # Draft PR is refused.
        with mock.patch.object(mod, "read_pr", return_value={**pr, "draft": True}):
            with self.assertRaises(SystemExit):
                mod.take_review("run-a", 42, head, "reviewer", 1)

        # Wrong PR number (card says pr = 42, caller asks for PR #99) is refused.
        _git(author, "push", "origin", "HEAD:refs/pull/99/head")
        pr99 = {**pr, "number": 99}
        with mock.patch.object(mod, "read_pr", return_value=pr99):
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("run-a", 99, head, "reviewer", 1)
            self.assertIn("not declared this claim ready for handoff", str(cm.exception))

        # Stale PR head after a new commit on the PR is refused.
        _git(author, "commit", "--allow-empty", "-m", "advance PR head")
        _git(author, "push", "origin", "HEAD:refs/pull/42/head")
        new_head = _git(author, "rev-parse", "HEAD")
        with mock.patch.object(mod, "read_pr", return_value={**pr, "head": {"sha": new_head}}):
            with self.assertRaises(SystemExit):
                mod.take_review("run-a", 42, head, "reviewer", 1)

        # Head card without `pr = N` is refused even when the lease is released.
        _, head_nopr, pr_nopr = self._released_pr_handoff(
            "c1", status="open", number=45, include_pr=False
        )
        with mock.patch.object(mod, "read_pr", return_value=pr_nopr):
            with self.assertRaises(SystemExit):
                mod.take_review("c1", 45, head_nopr, "reviewer", 1)

        # A claim that was never reserved (no hold ref) is still refused.
        unclaimed = self.sb.clone("unclaimed-author")
        _git(unclaimed, "switch", "-c", "unclaimed-c4")
        c4_card = unclaimed / "frontier" / "claims" / "c4.toml"
        c4_card.write_text(c4_card.read_text().replace('status = "open"', 'status = "open"\npr = 77', 1))
        _git(unclaimed, "add", str(c4_card))
        _git(unclaimed, "commit", "-m", "unreserved PR for c4")
        _git(unclaimed, "push", "origin", "HEAD:refs/pull/77/head")
        c4_head = _git(unclaimed, "rev-parse", "HEAD")
        c4_pr = {**pr, "number": 77, "body": "Claim: c4", "head": {"sha": c4_head}}
        with mock.patch.object(mod, "read_pr", return_value=c4_pr):
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("c4", 77, c4_head, "reviewer", 1)
            self.assertIn("no existing reservation", str(cm.exception))

        # A PR head that does not descend from the reserved work base is refused.
        _, head_c2, pr_c2 = self._released_pr_handoff("c2", status="open", number=46)
        sha_c2, lease_c2 = mod.holds()["c2"]
        self.assertTrue(mod.write_lease("c2", {**lease_c2, "base": "f" * 40}, sha_c2, sha_c2)[0])
        with mock.patch.object(mod, "read_pr", return_value=pr_c2):
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("c2", 46, head_c2, "reviewer", 1)
            self.assertIn("does not descend from the reserved work base", str(cm.exception))

        # A terminal ordinary lease from an earlier merged PR is refused for a new PR.
        _, head_c3, pr_c3 = self._released_pr_handoff("c3", status="open", number=47)
        sha_c3, lease_c3 = mod.holds()["c3"]
        self.assertTrue(
            mod.write_lease("c3", {**lease_c3, "terminal": "a" * 40}, sha_c3, sha_c3)[0]
        )
        with mock.patch.object(mod, "read_pr", return_value=pr_c3):
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("c3", 47, head_c3, "reviewer", 1)
            self.assertIn("closed for a different PR", str(cm.exception))

        # Lease stayed released throughout all refusals.
        self.assertTrue(mod.holds()["run-a"][1]["released"])

    def test_resume_released_pr_claim_respects_reservation_limit_and_cas_race(self) -> None:
        """Resuming a released PR consumes a slot, refuses at capacity, and serializes via CAS."""
        _, head, pr = self._released_pr_handoff("run-a", status="open", number=42)
        seed = self.sb.clone("filler")
        sessions = {}
        for cid in ("c1", "c2", "c3"):
            res = self.sb.run_frontier(seed, "claim", cid, "--by", "filler")
            self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
            sessions[cid] = _extract_session(res.stdout)

        # At 3/3 live holds, resuming a released lease MUST fail.
        mod = _load_frontier_module(self.sb.clone("reviewer"))
        with mock.patch.object(mod, "read_pr", return_value=pr):
            with self.assertRaises(SystemExit) as cm:
                mod.take_review("run-a", 42, head, "reviewer", 1)
            self.assertIn("capacity", str(cm.exception))
        self.assertTrue(mod.holds()["run-a"][1]["released"])
        self.assertEqual(sum(mod.live(v) for _, v in mod.holds().values()), 3)

        # Free one slot (2/3 live) and race take_review(run-a) against claim(c4) on the last slot.
        rel = self.sb.run_frontier(seed, "release", "c3", "--session", sessions["c3"])
        self.assertEqual(rel.returncode, 0, rel.stdout + rel.stderr)

        rev_mod = _load_frontier_module(self.sb.clone("racer-review"))
        clm_mod = _load_frontier_module(self.sb.clone("racer-claim"))
        barrier = threading.Barrier(2)

        def run_take_review() -> bool:
            original = rev_mod.write_lease

            def write(*args):
                barrier.wait(timeout=10)
                return original(*args)

            with (
                mock.patch.object(rev_mod, "read_pr", return_value=pr),
                mock.patch.object(rev_mod, "write_lease", side_effect=write),
            ):
                try:
                    rev_mod.take_review("run-a", 42, head, "reviewer", 1)
                    return True
                except SystemExit:
                    return False

        def run_claim() -> bool:
            original = clm_mod.write_lease

            def write(*args):
                barrier.wait(timeout=10)
                return original(*args)

            with mock.patch.object(clm_mod, "write_lease", side_effect=write):
                try:
                    clm_mod.claim("c4", "contender", 1)
                    return True
                except SystemExit:
                    return False

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            f1 = pool.submit(run_take_review)
            f2 = pool.submit(run_claim)
            results = [f1.result(), f2.result()]
        self.assertEqual(sum(results), 1)
        self.assertEqual(sum(mod.live(v) for _, v in mod.holds().values()), 3)

    def test_resume_released_pr_claim_refuses_overlapping_live_holds(self) -> None:
        """Resuming a released PR never bypasses an overlapping path or mechanism live hold."""
        _, head, pr = self._released_pr_handoff("run-a", status="open", number=42)
        other = self.sb.clone("other")
        mod = _load_frontier_module(self.sb.clone("reviewer"))

        for blocker in ("run-b", "run-parent", "same-mech"):
            res = self.sb.run_frontier(other, "claim", blocker, "--by", "other")
            self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
            session = _extract_session(res.stdout)
            with mock.patch.object(mod, "read_pr", return_value=pr):
                with self.assertRaises(SystemExit) as cm:
                    mod.take_review("run-a", 42, head, "reviewer", 1)
                self.assertIn("overlaps another live reservation", str(cm.exception))
            self.assertTrue(mod.holds()["run-a"][1]["released"])
            rel = self.sb.run_frontier(other, "release", blocker, "--session", session)
            self.assertEqual(rel.returncode, 0, rel.stdout + rel.stderr)

        # A disjoint sibling path does NOT block resuming the released PR.
        res = self.sb.run_frontier(other, "claim", "run-sibling", "--by", "other")
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        with mock.patch.object(mod, "read_pr", return_value=pr):
            mod.take_review("run-a", 42, head, "reviewer", 1)
        self.assertFalse(mod.holds()["run-a"][1]["released"])
        self.assertEqual(mod.holds()["run-a"][1]["phase"], "integration")


class TestPresence(unittest.TestCase):
    """Frozen fixtures for claim agent-presence: observational presence, apart from leases."""

    def setUp(self) -> None:
        """Create a fresh isolated remote per test."""
        self._td = tempfile.TemporaryDirectory(prefix="frontier-presence-")
        self.sb = FrontierSandbox(pathlib.Path(self._td.name))

    def tearDown(self) -> None:
        """Remove temporary directories after each test."""
        self._td.cleanup()

    def _refs(self, prefix: str = "refs/heads/presence/") -> dict[str, str]:
        """{ref: sha} on the bare remote under `prefix`."""
        out = _git(self.sb.remote, "for-each-ref", "--format=%(objectname) %(refname)", prefix)
        return {ref: sha for sha, ref in (line.split() for line in out.splitlines())}

    def _record(self, ref: str) -> dict:
        """The presence record a remote ref holds."""
        import tomllib

        return tomllib.loads(_git(self.sb.remote, "show", f"{ref}:presence.toml"))

    def _arrive(self, repo: pathlib.Path, family: str = "agent", *extra: str):
        """Run `arrive` in `repo`, asserting success."""
        r = self.sb.run_frontier(repo, "arrive", "--as", family, *extra)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        return r

    def _push_raw(self, repo: pathlib.Path, ref: str, body: str) -> str:
        """Publish an arbitrary presence.toml on `ref` (a malformed or forged record)."""
        blob = subprocess.run(
            ["git", "-C", str(repo), "hash-object", "-w", "--stdin"],
            input=body, capture_output=True, text=True, check=True,
        ).stdout.strip()
        tree = subprocess.run(
            ["git", "-C", str(repo), "mktree"],
            input=f"100644 blob {blob}\tpresence.toml\n", capture_output=True, text=True,
            check=True,
        ).stdout.strip()
        sha = _git(repo, "commit-tree", tree, "-m", "raw")
        _git(repo, "push", "-q", "origin", f"{sha}:{ref}")
        return sha

    def test_concurrent_independent_instances_publish_separate_records(self) -> None:
        """Instances arriving at once each get their own ref and are counted as fresh."""
        clones = [self.sb.clone(f"inst-{i}") for i in range(4)]
        barrier = threading.Barrier(len(clones))

        def go(repo):
            barrier.wait()
            return self.sb.run_frontier(repo, "arrive", "--as", "agent", "--can", "shell")

        with concurrent.futures.ThreadPoolExecutor(max_workers=len(clones)) as pool:
            results = list(pool.map(go, clones))
        for r in results:
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        refs = self._refs()
        self.assertEqual(len(refs), 4)
        self.assertEqual(len({ref.rsplit("/", 1)[1] for ref in refs}), 4)
        pop = self.sb.run_frontier(clones[0], "population").stdout
        self.assertIn("4 fresh", pop)
        self.assertIn("live reservations (hold/*, not instances): 0", pop)

    def test_repeat_arrival_and_touch_keep_one_instance_and_throttle(self) -> None:
        """Re-arrival reuses the instance; unchanged touches are throttled; changes are pushed."""
        a = self.sb.clone("a")
        self._arrive(a, "agent", "--can", "shell")
        self._arrive(a, "agent", "--can", "shell,rust")
        refs = self._refs()
        self.assertEqual(len(refs), 1)
        (ref, sha), = refs.items()
        self.assertEqual(self._record(ref)["capabilities"], ["rust", "shell"])
        r = self.sb.run_frontier(a, "touch")
        self.assertEqual(r.returncode, 0)
        self.assertIn("throttled", r.stdout)
        self.assertEqual(self._refs()[ref], sha, "an unchanged touch must not write")
        r = self.sb.run_frontier(a, "touch", "--state", "active")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        new = self._refs()[ref]
        self.assertNotEqual(new, sha)
        self.assertEqual(_git(self.sb.remote, "rev-parse", f"{new}^"), sha, "fast-forward chain")
        self.assertEqual(self._record(ref)["state"], "active")
        mod = _load_frontier_module(a)
        later = mod.now() + mod.PRESENCE_THROTTLE + dt.timedelta(minutes=1)
        with mock.patch.object(mod, "now", return_value=later):
            ok, _ = mod.presence_touch()
        self.assertTrue(ok, "an unchanged record is re-pushed after the throttle interval")

    def test_instance_is_stable_across_claim_transfers_and_distinct_from_session(self) -> None:
        """Claim, release and a different claim keep one instance; lease sessions are not in it."""
        a = self.sb.clone("a")
        self._arrive(a, "agent", "--can", "shell,workflow-dispatch")
        (ref,) = self._refs()
        instance = self._record(ref)["instance"]
        r = self.sb.run_frontier(a, "claim", "c1", "--by", "agent")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        session = _extract_session(r.stdout)
        rec = self._record(ref)
        self.assertEqual((rec["state"], rec["claim"], rec["instance"]), ("active", "c1", instance))
        self.assertNotEqual(session, instance)
        body = _git(self.sb.remote, "show", f"{ref}:presence.toml")
        self.assertNotIn(session, body)
        self.assertEqual(set(rec), {"schema", "participant", "instance", "state", "claim",
                                    "capabilities", "started", "last_seen"})
        r = self.sb.run_frontier(a, "release", "c1", "--session", session)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        rec = self._record(ref)
        self.assertEqual((rec["state"], rec["claim"], rec["instance"]), ("idle", "", instance))
        r = self.sb.run_frontier(a, "claim", "c2", "--by", "agent")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self._record(ref)["claim"], "c2")
        self.assertEqual(list(self._refs()), [ref], "one instance, one ref, across claims")

    def test_optional_capabilities_and_input_bounds(self) -> None:
        """Capabilities may be absent; unsafe family, capability or state values are refused."""
        a = self.sb.clone("a")
        self._arrive(a, "agent")
        (ref,) = self._refs()
        self.assertEqual(self._record(ref)["capabilities"], [])
        for bad in (["--as", "../evil"], ["--as", "Agent"], ["--as", "a" * 40],
                    ["--as", "ok", "--can", "Shell"], ["--as", "ok", "--state", "departed"],
                    ["--as", "ok", "--can", ",".join(f"c{i}" for i in range(17))]):
            r = self.sb.run_frontier(a, "arrive", *bad)
            self.assertNotEqual(r.returncode, 0, bad)
        self.assertEqual(list(self._refs()), [ref])

    def test_population_separates_instances_from_parked_and_released_holds(self) -> None:
        """Reservations are counted apart from instances, parked included, released excluded."""
        a = self.sb.clone("a")
        b = self.sb.clone("b")
        self._arrive(a, "agent")
        r = self.sb.run_frontier(b, "claim", "c1", "--by", "other")  # b never arrives
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        session = _extract_session(r.stdout)
        mod = _load_frontier_module(b)
        sha, lease = mod.holds()["c2"] if "c2" in mod.holds() else (None, None)
        parked = {"id": "c2", "holder": "x", "session": "abcdef012345", "phase": "parked",
                  "pr": 7, "pr_head": "0" * 40, "claimed_at": mod.now().isoformat(),
                  "expires": mod.now().isoformat(), "owned_paths": [], "mechanisms": [],
                  "base": "", "released": False}
        ok, out = mod.write_lease("c2", parked, sha, sha)
        self.assertTrue(ok, out)
        pop = self.sb.run_frontier(a, "population").stdout
        self.assertIn("1 fresh", pop)
        self.assertIn("live reservations (hold/*, not instances): 2 (1 parked, 1 work)", pop)
        self.sb.run_frontier(b, "release", "c1", "--session", session)
        pop = self.sb.run_frontier(a, "population").stdout
        self.assertIn("live reservations (hold/*, not instances): 1 (1 parked)", pop)
        self.assertIn("1 fresh", pop)

    def test_graceful_departure_then_new_instance(self) -> None:
        """Departure publishes a departed record; the next arrival is a new instance."""
        a = self.sb.clone("a")
        self._arrive(a)
        (ref,) = self._refs()
        r = self.sb.run_frontier(a, "depart")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self._record(ref)["state"], "departed")
        self.assertIn("1 departed", self.sb.run_frontier(a, "population").stdout)
        r = self.sb.run_frontier(a, "touch")
        self.assertNotEqual(r.returncode, 0, "a departed checkout has nothing to refresh")
        self._arrive(a)
        self.assertEqual(len(self._refs()), 2)

    def test_stale_records_are_unknown_not_dead_and_old_ones_are_pruned(self) -> None:
        """Age past the TTL reads as stale/unknown; only long-stale or departed records prune."""
        a = self.sb.clone("a")
        b = self.sb.clone("b")
        self._arrive(a)
        (ref_a,) = self._refs()
        mod = _load_frontier_module(b)
        t3 = mod.now() + dt.timedelta(hours=3)
        with mock.patch.object(mod, "now", return_value=t3):
            lines = "\n".join(mod.presence_population())
        self.assertIn("0 fresh", lines)
        self.assertIn("1 stale or unknown", lines)
        self.assertNotIn("dead", lines)
        with mock.patch.object(mod, "now", return_value=t3):
            self.assertEqual(mod.presence_prune(), 0, "3 h stale is not prunable")
        t8 = mod.now() + dt.timedelta(days=8)
        with mock.patch.object(mod, "now", return_value=t8):
            lines = "\n".join(mod.presence_population())
            self.assertIn("1 expired (unseen 7+ days)", lines)
            self.assertIn("0 stale or unknown", lines)
            rec, (ok, _) = mod.presence_arrive("agent", [])
            self.assertTrue(ok)
        refs = self._refs()
        self.assertNotIn(ref_a, refs, "an 8-day-old record is pruned on another's arrival")
        self.assertEqual(len(refs), 1)
        # Bounded: at most PRESENCE_PRUNE_MAX deletions per write.
        for i in range(5):
            self._arrive(self.sb.clone(f"old-{i}"))
        t16 = mod.now() + dt.timedelta(days=16)
        with mock.patch.object(mod, "now", return_value=t16):
            self.assertEqual(mod.presence_prune(), mod.PRESENCE_PRUNE_MAX)

    def test_malformed_records_are_reported_never_trusted_or_deleted(self) -> None:
        """Bad TOML, wrong keys, ref/record mismatch and oversized records count as malformed."""
        a = self.sb.clone("a")
        good = {"schema": 1, "participant": "agent", "instance": "0123456789abcdef",
                "state": "active", "claim": "", "capabilities": [],
                "started": "2026-11-27T00:00:00+00:00", "last_seen": "2026-11-27T00:00:00+00:00"}
        mod = _load_frontier_module(a)
        self._push_raw(a, "refs/heads/presence/agent/zzzz", "not = [toml")
        self._push_raw(a, "refs/heads/presence/agent/0123456789abcdef",
                       mod.presence_body(good) + 'session = "secret"\n')
        self._push_raw(a, "refs/heads/presence/other/0123456789abcdef", mod.presence_body(good))
        self._push_raw(a, "refs/heads/presence/agent/fedcba9876543210",
                       mod.presence_body({**good, "instance": "fedcba9876543210"})
                       + "#" * 4096 + "\n")
        self._push_raw(a, "refs/heads/presence/flat", mod.presence_body(good))
        self._push_raw(a, "refs/heads/presence/agent/1111111111111111",
                       "a = " + "[" * 1020 + "]" * 1020 + "\n")  # RecursionError in tomllib
        self._push_raw(a, "refs/heads/presence/agent/2222222222222222",
                       mod.presence_body({**good, "instance": "2222222222222222"})
                       .replace("schema = 1", "schema = true"))
        self._push_raw(a, "refs/heads/presence/agent/3333333333333333",
                       mod.presence_body({**good, "instance": "3333333333333333"})
                       .replace('claim = ""', "claim = false"))
        # presence.toml as a tree, sorting first: must not desynchronise later records.
        sub = subprocess.run(["git", "-C", str(a), "mktree"], input="", capture_output=True,
                             text=True, check=True).stdout.strip()
        blob = subprocess.run(["git", "-C", str(a), "hash-object", "-w", "--stdin"],
                              input="x" * 300, capture_output=True, text=True,
                              check=True).stdout.strip()
        inner = subprocess.run(["git", "-C", str(a), "mktree"],
                               input=f"100644 blob {blob}\tz\n040000 tree {sub}\ty\n",
                               capture_output=True, text=True, check=True).stdout.strip()
        tree = subprocess.run(["git", "-C", str(a), "mktree"],
                              input=f"040000 tree {inner}\tpresence.toml\n",
                              capture_output=True, text=True, check=True).stdout.strip()
        _git(a, "push", "-q", "origin",
             f"{_git(a, 'commit-tree', tree, '-m', 'tree')}:refs/heads/presence/aaa/0000000000000000")
        ok_ref = "refs/heads/presence/zzz/4444444444444444"
        self._push_raw(a, ok_ref, mod.presence_body(
            {**good, "participant": "zzz", "instance": "4444444444444444",
             "last_seen": mod.now().isoformat()}))
        lines = "\n".join(mod.presence_population())
        self.assertIn("9 malformed", lines)
        self.assertIn("1 fresh", lines, "a record after a hostile one is still read correctly")
        self.assertIn("zzz/44444444", lines)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), mock.patch.object(
            mod, "read_open_prs", return_value=([], None)
        ):
            mod.brief({"shell"})
        self.assertIn("9 malformed", buf.getvalue())
        with mock.patch.object(mod, "presence_records", side_effect=RecursionError):
            self.assertIn("unknown (error", mod.presence_population()[0])
        t = mod.now() + dt.timedelta(days=30)
        with mock.patch.object(mod, "now", return_value=t):
            self.assertEqual(mod.presence_prune(), 1, "only the well-formed record is pruned")
        self.assertEqual(len(self._refs()), 9)

    def test_unavailable_remote_and_refused_presence_never_block_claim_work(self) -> None:
        """Presence failure is reported; claims still succeed and population says unknown."""
        a = self.sb.clone("a")
        self._arrive(a)
        hook = self.sb.remote / "hooks" / "pre-receive"
        hook.write_text(
            "#!/bin/sh\nwhile read old new ref; do case \"$ref\" in refs/heads/presence/*)"
            " echo refused >&2; exit 1;; esac; done\nexit 0\n"
        )
        hook.chmod(0o755)
        r = self.sb.run_frontier(a, "claim", "c1", "--by", "agent")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("acquired c1", r.stdout)
        self.assertIn("presence not refreshed", r.stderr)
        b = self.sb.clone("b")
        _git(b, "remote", "set-url", "origin", str(self.sb.tmp / "missing.git"))
        mod = _load_frontier_module(b)
        lines = mod.presence_population()
        self.assertIn("unknown (origin unavailable)", lines[0])
        with mock.patch.object(mod, "presence_touch", side_effect=RuntimeError("boom")):
            mod.presence_checkpoint("active", "c1")  # never raises

    def test_cleanup_racing_refresh_cannot_delete_a_refreshed_record(self) -> None:
        """Cleanup deletes by exact SHA, so a refresh in between survives; a pruned record returns."""
        a = self.sb.clone("a")
        b = self.sb.clone("b")
        self._arrive(a)
        (ref,) = self._refs()
        pruner = _load_frontier_module(b)
        t8 = pruner.now() + dt.timedelta(days=8)
        with mock.patch.object(pruner, "now", return_value=t8):
            snapshot = pruner.presence_records()
        r = self.sb.run_frontier(a, "touch", "--state", "active")  # refresh lands first
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        refreshed = self._refs()[ref]
        with (
            mock.patch.object(pruner, "now", return_value=t8),
            mock.patch.object(pruner, "presence_records", return_value=snapshot),
        ):
            self.assertEqual(pruner.presence_prune(), 0)
        self.assertEqual(self._refs()[ref], refreshed)
        # Prune wins the other way round: the owner's next refresh recreates its record.
        with mock.patch.object(pruner, "now", return_value=t8 + dt.timedelta(days=8)):
            self.assertEqual(pruner.presence_prune(), 1)
        self.assertNotIn(ref, self._refs())
        r = self.sb.run_frontier(a, "touch", "--state", "idle")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self._record(ref)["state"], "idle")

    def test_presence_leaves_admission_holds_and_scheduling_unchanged(self) -> None:
        """No epoch mutation, no hold refs, no slot use, and an unarrived client writes nothing."""
        a = self.sb.clone("a")
        b = self.sb.clone("b")
        r = self.sb.run_frontier(b, "claim", "c3", "--by", "b")  # creates the admission epoch
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self._refs(), {}, "a client that never arrived publishes nothing")
        before = (self._refs("refs/heads/frontier-admission"), self._refs("refs/heads/hold/"))
        mod_b = _load_frontier_module(b)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), mock.patch.object(
            mod_b, "read_open_prs", return_value=([], None)
        ):
            mod_b.brief({"shell"})
        take_before = buf.getvalue().split("## What you can take", 1)[1].split("##", 1)[0]
        for i in range(6):
            self._arrive(self.sb.clone(f"p{i}"), "agent", "--state", "active")
        self._arrive(a)
        self.sb.run_frontier(a, "touch", "--state", "active", "--claim", "c4")
        self.sb.run_frontier(a, "depart")
        after = (self._refs("refs/heads/frontier-admission"), self._refs("refs/heads/hold/"))
        self.assertEqual(before, after)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), mock.patch.object(
            mod_b, "read_open_prs", return_value=([], None)
        ):
            mod_b.brief({"shell"})
        out = buf.getvalue()
        self.assertEqual(out.split("## What you can take", 1)[1].split("##", 1)[0], take_before)
        self.assertIn("6 fresh", out.split("## Population", 1)[1])
        self.assertIn("1 departed", out.split("## Population", 1)[1])
        # A presence claiming c4 does not reserve it: another agent can still claim c4.
        r = self.sb.run_frontier(b, "claim", "c4", "--by", "b")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


class TestWorkstationPressure(unittest.TestCase):
    """Frozen fixtures for `workstation-pressure` (eight stop_test scenarios + #1086 handoff)."""

    def setUp(self) -> None:
        """Create an isolated temporary frontier sandbox for each pressure test."""
        self._td = tempfile.TemporaryDirectory(prefix="frontier-wp-")
        self.sb = FrontierSandbox(pathlib.Path(self._td.name))

    def tearDown(self) -> None:
        """Remove the temporary frontier sandbox."""
        self._td.cleanup()

    def _reset_claims(self, claims: dict[str, str], *, queue_text: str = "# repo: lmmx/mibz2\n") -> None:
        """Replace sandbox claims and workstation queue on `origin/master`."""
        seed = self.sb.seed
        cdir = seed / "frontier" / "claims"
        for existing in list(cdir.glob("*.toml")):
            existing.unlink()
        for cid, body in claims.items():
            (cdir / f"{cid}.toml").write_text(body)
        (seed / "bench" / "workstation-queue.txt").write_text(queue_text)
        _git(seed, "add", "-A", "frontier/claims", "bench/workstation-queue.txt")
        _git(seed, "commit", "-m", "configure workstation-pressure fixture")
        _git(seed, "push", "origin", "master")

    def _publish_evidence(
        self,
        bundles: dict[str, str | dict | None],
        *,
        message: str = "evidence: fixture/run-20261127T000000Z",
    ) -> str:
        """Create or advance orphan `origin/evidence` with `.tar.gz` bundles.

        A string value is written as `commit.txt`; a dict as `provenance.json`.
        """
        seed = self.sb.seed
        index = self.sb.tmp / f"ev-index-{secrets_counter[0]}"
        secrets_counter[0] += 1
        env = {**os.environ, **GIT_ENV, "GIT_INDEX_FILE": str(index)}
        for rel_path, commit_sha in bundles.items():
            stamp = pathlib.PurePosixPath(rel_path).name.removesuffix(".tar.gz")
            buf = io.BytesIO()
            with tarfile.open(fileobj=buf, mode="w:gz") as tf:
                if isinstance(commit_sha, dict):
                    data = json.dumps(commit_sha).encode()
                    ti = tarfile.TarInfo(name=f"{stamp}/provenance.json")
                    ti.size = len(data)
                    tf.addfile(ti, io.BytesIO(data))
                elif commit_sha is not None:
                    data = f"{commit_sha}\n".encode()
                    ti = tarfile.TarInfo(name=f"{stamp}/commit.txt")
                    ti.size = len(data)
                    tf.addfile(ti, io.BytesIO(data))
            blob = subprocess.run(
                ["git", "-C", str(seed), "hash-object", "-w", "--stdin"],
                input=buf.getvalue(),
                capture_output=True,
                check=True,
                env=env,
            ).stdout.decode().strip()
            _git(seed, "update-index", "--add", "--cacheinfo", f"100644,{blob},{rel_path}", env=env)
        tree = _git(seed, "write-tree", env=env)
        if index.exists():
            index.unlink()
        parent_res = subprocess.run(
            ["git", "-C", str(seed), "rev-parse", "-q", "--verify", "refs/remotes/origin/evidence^{commit}"],
            capture_output=True,
            text=True,
        )
        parent = parent_res.stdout.strip() if parent_res.returncode == 0 else ""
        args = ["commit-tree", tree, "-m", message]
        if parent:
            args += ["-p", parent]
        ev_sha = _git(seed, *args)
        _git(seed, "push", "origin", f"{ev_sha}:refs/heads/evidence")
        _git(seed, "fetch", "-q", "origin", "+refs/heads/evidence:refs/remotes/origin/evidence")
        return ev_sha

    def _tag_ready(self, recipe: str, sha: str, *, hold: bool = False) -> None:
        ref = f"refs/tags/{'hold' if hold else 'ready'}/{recipe}"
        _git(self.sb.seed, "push", "-f", "origin", f"{sha}:{ref}")

    def _run_brief_and_status(self, repo: pathlib.Path, *, can: set[str] | None = None, prs=None, pr_error=None):
        mod = _load_frontier_module(repo)
        pr_val = ([], None) if (prs is None and pr_error is None) else (prs, pr_error)
        with mock.patch.object(mod, "read_open_prs", return_value=pr_val):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                mod.brief(can or set())
            brief_out = buf.getvalue()
            status_out = mod.render()
        return brief_out, status_out

    # Scenario 1: verified empty/drained queue with eligible candidate
    def test_wp1_verified_empty_and_drained_queue_elevates_shortest_path_without_mutating_priorities(self) -> None:
        """Empty or drained repo queue derives effective P1 for shortest eligible path; TOML unchanged."""
        master_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "ws-build": _claim_toml(
                    "ws-build",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell", "rust"],
                    owned_paths=["src/open/endpoint.rs"],
                    done_when="Prototype PR and i9-10900K workstation registration ready.",
                ),
                "ws-run": _claim_toml(
                    "ws-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    needs=["shell", "rust"],
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Queue frozen workstation recipe and read bundle.",
                ),
                "ws-review": _claim_toml(
                    "ws-review",
                    kind="build",
                    zone="striated",
                    priority=2,
                    needs=["shell", "rust"],
                    owned_paths=["src/huff.rs"],
                    done_when="Frozen workstation registration and i9 evidence bundle.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\nbench-old bench/old already ran\n",
        )
        self._tag_ready("bench-old", master_sha)
        self._publish_evidence(
            {"old/run-20261126T133034Z.tar.gz": master_sha},
            message="evidence: old/run-20261126T133034Z",
        )

        reader = self.sb.clone("reader-wp1")
        before_toml = {
            p.name: p.read_text() for p in sorted((reader / "frontier" / "claims").glob("*.toml"))
        }
        pr_review = {
            "number": 1053,
            "body": "Claim: ws-review\n\nReady for review.",
            "head": {"sha": "c" * 40},
            "base": {"ref": "master"},
            "state": "open",
            "draft": False,
        }
        brief_out, status_out = self._run_brief_and_status(
            reader, can={"shell", "rust"}, prs=[pr_review]
        )

        # Pressure active, physical i9 unknown, review PR surfaced first in shortest path
        self.assertIn("## Workstation replenishment pressure", brief_out)
        self.assertIn("Repository queue: starved", brief_out)
        self.assertIn("Physical i9 state: unknown", brief_out)
        self.assertIn("Replenishment pressure: active", brief_out)
        pressure_sec = brief_out.split("## Workstation replenishment pressure", 1)[1].split("##", 1)[0]
        self.assertLess(pressure_sec.index("ws-review"), pressure_sec.index("ws-run"))
        self.assertLess(pressure_sec.index("ws-run"), pressure_sec.index("ws-build"))

        # `What you can take` elevates ws-run (Tier 1) and ws-build (Tier 2) ahead of other-p2
        take_sec = brief_out.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertIn("ws-run", take_sec)
        self.assertIn("ws-build", take_sec)
        self.assertIn("P1 effective, declared P2", take_sec)
        self.assertIn("P1 effective, declared P3", take_sec)
        self.assertNotIn("**ws-review**", take_sec)  # pending PR, not new claim
        self.assertLess(take_sec.index("**ws-run**"), take_sec.index("**ws-build**"))
        self.assertLess(take_sec.index("**ws-build**"), take_sec.index("**other-p2**"))

        # `status` explains the signal clearly
        self.assertIn("## Workstation replenishment pressure", status_out)
        self.assertIn("Repository queue: starved", status_out)
        self.assertIn("Physical i9 state: unknown", status_out)
        self.assertIn("Replenishment pressure: active", status_out)

        # Declared priorities in claim TOML files are byte-for-byte unchanged; no holds/tags mutated
        after_toml = {
            p.name: p.read_text() for p in sorted((reader / "frontier" / "claims").glob("*.toml"))
        }
        self.assertEqual(before_toml, after_toml)

    # Regression (integration review of #1093): newer recipes record the run in provenance.json
    def test_wp_provenance_json_counts_as_archived_evidence(self) -> None:
        """A bundle's provenance.json `recipe`/`head` marks the tagged run as evidenced; `source` does not."""
        old_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._reset_claims(
            {"other-p2": _claim_toml("other-p2", kind="probe", priority=2)},
            queue_text="# repo: lmmx/mibz2\nbench-a bench/a run a\nbench-b bench/b run b\n",
        )
        new_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._tag_ready("bench-a", old_sha)
        self._tag_ready("bench-b", new_sha)
        self._publish_evidence(
            {
                "a/run-20261127T135104Z.tar.gz": {"recipe": old_sha, "source": new_sha},
                "b/run-20261127T172728Z.tar.gz": {"head": new_sha, "head_src_tree": old_sha},
            },
            message="evidence: provenance bundles",
        )
        reader = self.sb.clone("reader-prov")
        mod = _load_frontier_module(reader)
        state = mod.read_workstation_queue_state(fetch=True)
        self.assertEqual(state["state"], "starved", state)

        # A provenance file naming the tagged commit only as a source/base is not evidence of the run.
        self._publish_evidence(
            {
                "a/run-20261127T135104Z.tar.gz": {"recipe": old_sha, "source": new_sha},
                "b/run-20261127T172728Z.tar.gz": {"head": old_sha, "source": new_sha},
            },
            message="evidence: provenance source only",
        )
        state = mod.read_workstation_queue_state(fetch=True)
        self.assertEqual(state["state"], "replenished", state)
        self.assertEqual([r for r, _ in state["runnable"]], ["bench-b"])

    # Scenario 2: nonempty/running queue -> pressure decays on replenishment & re-activates on drain
    def test_wp2_replenished_queue_decays_pressure_and_redrains_on_evidence(self) -> None:
        """Adding an un-evidenced ready entry or re-tagging decays pressure; evidence re-drains it."""
        old_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "ws-build": _claim_toml(
                    "ws-build",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell"],
                    done_when="Frozen workstation recipe and i9 evidence.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\nbench-old bench/old prior run\nbench-next bench/next new run\n",
        )
        new_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._tag_ready("bench-old", old_sha)
        self._tag_ready("bench-next", new_sha)
        self._publish_evidence(
            {"old/run-20261126T133034Z.tar.gz": old_sha},
            message="evidence: old/run-20261126T133034Z",
        )

        reader = self.sb.clone("reader-wp2")
        brief_out, status_out = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: replenished", brief_out)
        self.assertIn("bench-next", brief_out)
        self.assertIn("Replenishment pressure: inactive", brief_out)
        self.assertIn("Repository queue: replenished", status_out)
        take_sec = brief_out.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("P1 effective", take_sec)
        # Declared priority order governs normally (other-p2 P2 ahead of ws-build P3)
        self.assertLess(take_sec.index("**other-p2**"), take_sec.index("**ws-build**"))

        # Archiving matching evidence for bench-next drains the queue -> pressure activates
        self._publish_evidence(
            {
                "old/run-20261126T133034Z.tar.gz": old_sha,
                "next/run-20261127T010000Z.tar.gz": new_sha,
            },
            message="evidence: next/run-20261127T010000Z",
        )
        brief_drained, _ = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: starved", brief_drained)
        self.assertIn("Replenishment pressure: active", brief_drained)
        take_drained = brief_drained.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertLess(take_drained.index("**ws-build**"), take_drained.index("**other-p2**"))

        # Re-tagging bench-next to a brand-new commit decays pressure again
        _git(self.sb.seed, "commit", "--allow-empty", "-m", "new commit for rerun")
        rerun_sha = _git(self.sb.seed, "rev-parse", "HEAD")
        self._tag_ready("bench-next", rerun_sha)
        brief_rerun, _ = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: replenished", brief_rerun)
        self.assertIn("Replenishment pressure: inactive", brief_rerun)

        # Placing hold/bench-next pauses it -> queue becomes starved again
        self._tag_ready("bench-next", rerun_sha, hold=True)
        brief_held_tag, _ = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: starved", brief_held_tag)
        self.assertIn("Replenishment pressure: active", brief_held_tag)

    # Scenario 3: stale or unavailable data -> fail closed
    def test_wp3_stale_or_unavailable_data_fails_closed(self) -> None:
        """Missing origin/evidence, unreachable origin, missing queue, or failed PR query fail closed."""
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "ws-build": _claim_toml(
                    "ws-build",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell"],
                    done_when="Workstation i9 validation.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        reader = self.sb.clone("reader-wp3")

        # (a) Missing origin/evidence -> fail closed
        brief_no_ev, status_no_ev = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: unknown", brief_no_ev)
        self.assertIn("Replenishment pressure: inactive (fail-closed", brief_no_ev)
        self.assertIn("Repository queue: unknown", status_no_ev)
        take_no_ev = brief_no_ev.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("P1 effective", take_no_ev)
        self.assertLess(take_no_ev.index("**other-p2**"), take_no_ev.index("**ws-build**"))

        # Publish evidence, then break PR API -> fail closed
        self._publish_evidence({})
        brief_bad_pr, _ = self._run_brief_and_status(
            reader, can={"shell"}, prs=None, pr_error="GitHub open-PR query failed or was incomplete"
        )
        self.assertIn("Replenishment pressure: inactive (fail-closed", brief_bad_pr)
        self.assertNotIn("P1 effective", brief_bad_pr)

        # Break origin URL -> fail closed even with stale local refs
        _git(reader, "remote", "set-url", "origin", str(self.sb.tmp / "nonexistent-origin.git"))
        brief_offline, _ = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: unknown", brief_offline)
        self.assertIn("Replenishment pressure: inactive (fail-closed", brief_offline)
        self.assertNotIn("P1 effective", brief_offline)

    # Scenario 4: multiple repositories -> never infer physical i9 idleness
    def test_wp4_multiple_repositories_physical_i9_always_unknown(self) -> None:
        """An empty repository queue reports physical i9 state as unknown, never idle."""
        self._reset_claims(
            {
                "ws-run": _claim_toml(
                    "ws-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    needs=["shell"],
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Queue workstation recipe.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        reader = self.sb.clone("reader-wp4")
        brief_out, status_out = self._run_brief_and_status(reader, can={"shell"})
        for out in (brief_out, status_out):
            self.assertIn("Physical i9 state: unknown", out)
            self.assertIn("other repositories", out.lower())
            self.assertNotIn("Physical i9 state: idle", out)
            self.assertNotIn("i9 is idle", out.lower())

    # Scenario 5: leased dependencies -> live holds and overlapping paths respected
    def test_wp5_leased_and_path_conflicting_dependencies_respected(self) -> None:
        """Live holds and overlapping owned_paths are never bypassed or double-promoted."""
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "src-holder": _claim_toml(
                    "src-holder",
                    kind="probe",
                    zone="striated",
                    priority=2,
                    owned_paths=["src/open.rs"],
                ),
                "ws-run": _claim_toml(
                    "ws-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    needs=["shell"],
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Queue workstation recipe.",
                ),
                "ws-build-conflict": _claim_toml(
                    "ws-build-conflict",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell"],
                    owned_paths=["src/open.rs"],
                    done_when="Workstation i9 validation.",
                ),
                "ws-build-free": _claim_toml(
                    "ws-build-free",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell"],
                    owned_paths=["src/huff.rs"],
                    done_when="Workstation i9 validation.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        holder = self.sb.clone("holder-wp5")
        self.assertEqual(
            self.sb.run_frontier(holder, "claim", "ws-run", "--by", "agent-a").returncode, 0
        )
        self.assertEqual(
            self.sb.run_frontier(holder, "claim", "src-holder", "--by", "agent-b").returncode, 0
        )

        reader = self.sb.clone("reader-wp5")
        brief_out, _ = self._run_brief_and_status(reader, can={"shell"})
        pressure_sec = brief_out.split("## Workstation replenishment pressure", 1)[1].split("##", 1)[0]
        self.assertIn("ws-run", pressure_sec)
        self.assertIn("leased by agent-a", pressure_sec)
        self.assertIn("ws-build-conflict", pressure_sec)
        self.assertIn("blocked by live hold src-holder", pressure_sec)

        take_sec = brief_out.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("**ws-run**", take_sec)
        self.assertIn("**ws-build-free** (P1 effective, declared P3", take_sec)
        self.assertNotIn("**ws-build-conflict** (P1 effective", take_sec)
        self.assertLess(take_sec.index("**ws-build-free**"), take_sec.index("**other-p2**"))
        # Lease enforcement still refuses double-claim
        self.assertNotEqual(
            self.sb.run_frontier(reader, "claim", "ws-run", "--by", "agent-c").returncode, 0
        )
        self.assertNotEqual(
            self.sb.run_frontier(reader, "claim", "ws-build-conflict", "--by", "agent-c").returncode,
            0,
        )

    # Scenario 6: failed, blocked, dropped, done, flagged, or dead-end candidates never promoted
    def test_wp6_failed_obsolete_blocked_flagged_and_dead_end_candidates_excluded(self) -> None:
        """Blocked, dropped, done, flagged, and dead-end candidates are never promoted."""
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "mtf-fold-run": _claim_toml(
                    "mtf-fold-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    status="blocked",
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Superseded workstation run.",
                ),
                "dropped-run": _claim_toml(
                    "dropped-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    status="dropped",
                    done_when="Dropped workstation run.",
                ),
                "done-run": _claim_toml(
                    "done-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    status="done",
                    done_when="Finished workstation run.",
                ),
                "flagged-init": _claim_toml(
                    "flagged-init",
                    kind="run",
                    zone="striated",
                    priority=2,
                    status="open",
                    done_when="Flagged workstation run.",
                ),
                "old-idea": _claim_toml(
                    "old-idea",
                    kind="build",
                    zone="striated",
                    priority=2,
                    status="open",
                    done_when="Dead-ended workstation i9 build.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        reader = self.sb.clone("reader-wp6")
        brief_out, status_out = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: starved", brief_out)
        self.assertIn("Replenishment pressure: inactive", brief_out)
        self.assertIn("Replenishment pressure: inactive", status_out)
        self.assertNotIn("P1 effective", brief_out)

    # Scenario 7: owner-only blockers (`needs_louis`) never promoted
    def test_wp7_owner_only_blockers_never_promoted(self) -> None:
        """Claims with non-empty `needs_louis` are never promoted as runnable pressure targets."""
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "owner-ws-run": _claim_toml(
                    "owner-ws-run",
                    kind="run",
                    zone="striated",
                    priority=2,
                    status="open",
                    needs_louis="RULES D: owner workstation authorization needed",
                    done_when="Queue workstation recipe.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        reader = self.sb.clone("reader-wp7")
        brief_out, status_out = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: starved", brief_out)
        self.assertIn("Replenishment pressure: inactive", brief_out)
        self.assertIn("Replenishment pressure: inactive", status_out)
        self.assertNotIn("P1 effective", brief_out)

    # Scenario 8: no useful candidates -> inactive, no synthetic benchmark work
    def test_wp8_no_useful_candidates_stays_inactive_no_synthetic_work(self) -> None:
        """When only non-workstation or idea claims exist, pressure stays inactive."""
        self._reset_claims(
            {
                "explore": _claim_toml("explore", kind="explore", priority=3),
                "synthesise": _claim_toml("synthesise", kind="synthesise", priority=2),
                "migrate-rules": _claim_toml(
                    "migrate-rules", kind="write", zone="striated", priority=2
                ),
                "ci-only-build": _claim_toml(
                    "ci-only-build",
                    kind="build",
                    zone="striated",
                    priority=2,
                    done_when="Unit tests and CI paired bench pass.",
                ),
                "future-ws-idea": _claim_toml(
                    "future-ws-idea",
                    kind="build",
                    zone="striated",
                    priority=2,
                    status="idea",
                    done_when="Future workstation i9 validation.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        reader = self.sb.clone("reader-wp8")
        brief_out, status_out = self._run_brief_and_status(reader, can={"shell"})
        self.assertIn("Repository queue: starved", brief_out)
        self.assertIn("Replenishment pressure: inactive", brief_out)
        self.assertIn("no eligible unfinished workstation-validation candidates", brief_out)
        self.assertIn("Replenishment pressure: inactive", status_out)
        self.assertNotIn("P1 effective", brief_out)

    # Scenario 9 (#1086): capability-blocked dispatch dependency exposes handoff, never sends incapable workers
    def test_wp9_capability_blocked_dispatch_handoff(self) -> None:
        """Capability-blocked dispatch exposes handoff to capable workers and elevates next eligible work."""
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "ws-queue-dispatch": _claim_toml(
                    "ws-queue-dispatch",
                    kind="run",
                    zone="striated",
                    priority=2,
                    needs=["shell", "workflow-dispatch"],
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Dispatch reviewed workstation recipe via run-signal workflow.",
                ),
                "ws-build-next": _claim_toml(
                    "ws-build-next",
                    kind="build",
                    zone="striated",
                    priority=3,
                    needs=["shell", "rust"],
                    owned_paths=["src/open/endpoint.rs"],
                    done_when="Frozen workstation i9 registration and build ready.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        self._publish_evidence({})
        reader = self.sb.clone("reader-wp9")

        # (a) Incapable worker (lacks workflow-dispatch): sees capability-blocked handoff, gets next eligible build
        brief_incap, status_out = self._run_brief_and_status(reader, can={"shell", "rust"})
        pressure_incap = brief_incap.split("## Workstation replenishment pressure", 1)[1].split(
            "##", 1
        )[0]
        self.assertIn("Repository queue: starved", pressure_incap)
        self.assertIn("Replenishment pressure: active", pressure_incap)
        self.assertIn("Capability-blocked handoff (needs workflow-dispatch", pressure_incap)
        self.assertIn("ws-queue-dispatch", pressure_incap)
        self.assertIn("Shortest path (upstream build): **ws-build-next**", pressure_incap)
        take_incap = brief_incap.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("**ws-queue-dispatch**", take_incap)
        self.assertIn("**ws-build-next** (P1 effective, declared P3", take_incap)
        self.assertLess(take_incap.index("**ws-build-next**"), take_incap.index("**other-p2**"))

        # Global status surfaces runtime capability requirement explicitly
        pressure_status = status_out.split("## Workstation replenishment pressure", 1)[1].split(
            "##", 1
        )[0]
        self.assertIn("ws-queue-dispatch", pressure_status)
        self.assertIn("needs shell, workflow-dispatch", pressure_status)

        # (b) Capable worker (has workflow-dispatch): receives ws-queue-dispatch as derived P1
        brief_cap, _ = self._run_brief_and_status(reader, can={"shell", "workflow-dispatch"})
        pressure_cap = brief_cap.split("## Workstation replenishment pressure", 1)[1].split(
            "##", 1
        )[0]
        self.assertIn("Replenishment pressure: active", pressure_cap)
        self.assertIn("Shortest path (run): **ws-queue-dispatch**", pressure_cap)
        self.assertNotIn("Capability-blocked handoff (needs workflow-dispatch", pressure_cap)
        self.assertIn("Capability-blocked handoff (needs rust", pressure_cap)
        take_cap = brief_cap.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertIn("**ws-queue-dispatch** (P1 effective, declared P2", take_cap)
        self.assertLess(take_cap.index("**ws-queue-dispatch**"), take_cap.index("**other-p2**"))

        brief_full, _ = self._run_brief_and_status(
            reader, can={"shell", "rust", "workflow-dispatch"}
        )
        pressure_full = brief_full.split("## Workstation replenishment pressure", 1)[1].split(
            "##", 1
        )[0]
        self.assertNotIn("Capability-blocked handoff", pressure_full)
        self.assertLess(
            pressure_full.index("ws-queue-dispatch"), pressure_full.index("ws-build-next")
        )

        # (c) Only capability-blocked dispatch exists -> incapable worker continues independent work
        self._reset_claims(
            {
                "other-p2": _claim_toml("other-p2", kind="probe", priority=2),
                "ws-queue-dispatch": _claim_toml(
                    "ws-queue-dispatch",
                    kind="run",
                    zone="striated",
                    priority=2,
                    needs=["shell", "workflow-dispatch"],
                    owned_paths=["bench/workstation-queue.txt"],
                    done_when="Dispatch reviewed workstation recipe via run-signal workflow.",
                ),
            },
            queue_text="# repo: lmmx/mibz2\n",
        )
        reader_only = self.sb.clone("reader-wp9-only")
        brief_only, _ = self._run_brief_and_status(reader_only, can={"shell", "rust"})
        pressure_only = brief_only.split("## Workstation replenishment pressure", 1)[1].split(
            "##", 1
        )[0]
        self.assertIn("Repository queue: starved", pressure_only)
        self.assertIn("Replenishment pressure: inactive for this worker's capabilities", pressure_only)
        self.assertIn("Capability-blocked handoff (needs workflow-dispatch", pressure_only)
        take_only = brief_only.split("## What you can take", 1)[1].split("##", 1)[0]
        self.assertNotIn("**ws-queue-dispatch**", take_only)
        self.assertNotIn("P1 effective", take_only)
        self.assertIn("**other-p2**", take_only)


class TestResultToRelease(unittest.TestCase):
    """Frozen fixtures for all 10 stop_test classes of `result-to-release`."""

    def setUp(self) -> None:
        """Create an isolated temporary frontier sandbox for each result-to-release test."""
        self._td = tempfile.TemporaryDirectory(prefix="frontier-r2r-")
        self.sb = FrontierSandbox(pathlib.Path(self._td.name))

    def tearDown(self) -> None:
        """Remove the temporary frontier sandbox."""
        self._td.cleanup()

    def _reset_state(
        self,
        claims: dict[str, str],
        *,
        queue_text: str = "# repo: lmmx/mibz2\n",
        deltas: dict[str, str] | None = None,
        ledger_rows: list[str] | None = None,
        dead_ends: list[str] | None = None,
        max_open_claims: int | None = None,
        max_live_holds: int | None = None,
    ) -> str:
        seed = self.sb.seed
        cdir = seed / "frontier" / "claims"
        for existing in list(cdir.glob("*.toml")):
            existing.unlink()
        for cid, body in claims.items():
            (cdir / f"{cid}.toml").write_text(body, encoding="utf-8")
        ddir = seed / "frontier" / "deltas"
        for existing in list(ddir.glob("*.md")):
            if existing.name != "README.md":
                existing.unlink()
        for fname, body in (deltas or {}).items():
            (ddir / fname).write_text(body, encoding="utf-8")
        (seed / "bench" / "workstation-queue.txt").write_text(queue_text, encoding="utf-8")
        if ledger_rows is not None:
            header = "pr\tmerged\teffect\tstate\tmeasured\tevidence\n"
            (seed / "frontier" / "ledger.tsv").write_text(
                header + "".join(r + "\n" for r in ledger_rows), encoding="utf-8"
            )
        if dead_ends is not None:
            header = "claim\tverdict\tsummary\tevidence\n"
            (seed / "frontier" / "dead-ends.tsv").write_text(
                header + "".join(r + "\n" for r in dead_ends), encoding="utf-8"
            )
        if max_open_claims is not None:
            cfg = (seed / "frontier" / "config.toml").read_text(encoding="utf-8")
            cfg = re.sub(r"max_open_claims\s*=\s*\d+", f"max_open_claims = {max_open_claims}", cfg)
            (seed / "frontier" / "config.toml").write_text(cfg, encoding="utf-8")
        if max_live_holds is not None:
            cfg_path = seed / "frontier" / "config.toml"
            cfg_txt = cfg_path.read_text(encoding="utf-8")
            if "limits_file" in cfg_txt:
                (seed / "frontier" / "limits.toml").write_text(
                    f"max_live_holds = {max_live_holds}\n", encoding="utf-8"
                )
            else:
                cfg_txt = re.sub(
                    r"max_live_holds\s*=\s*\d+", f"max_live_holds = {max_live_holds}", cfg_txt
                )
                cfg_path.write_text(cfg_txt, encoding="utf-8")
        _git(seed, "add", "-A", "frontier", "bench/workstation-queue.txt")
        _git(seed, "commit", "-m", "configure result-to-release fixture")
        _git(seed, "push", "origin", "master")
        return _git(seed, "rev-parse", "HEAD")

    def _publish_evidence(
        self,
        bundles: dict[str, str | dict | None],
        *,
        message: str = "evidence: fixture/run-20261127T000000Z",
        sidecar_override: dict[str, str] | None = None,
    ) -> str:
        seed = self.sb.seed
        index = self.sb.tmp / f"r2r-ev-index-{secrets_counter[0]}"
        secrets_counter[0] += 1
        env = {**os.environ, **GIT_ENV, "GIT_INDEX_FILE": str(index)}
        # Preserve existing tree entries from origin/evidence when advancing
        parent_res = subprocess.run(
            ["git", "-C", str(seed), "rev-parse", "-q", "--verify", "refs/remotes/origin/evidence^{commit}"],
            capture_output=True,
            text=True,
        )
        parent = parent_res.stdout.strip() if parent_res.returncode == 0 else ""
        if parent:
            _git(seed, "read-tree", parent, env=env)
        for rel_path, payload in bundles.items():
            stamp = pathlib.PurePosixPath(rel_path).name.removesuffix(".tar.gz")
            buf = io.BytesIO()
            with tarfile.open(fileobj=buf, mode="w:gz") as tf:
                if isinstance(payload, dict):
                    data = json.dumps(payload).encode()
                    ti = tarfile.TarInfo(name=f"{stamp}/provenance.json")
                    ti.size = len(data)
                    tf.addfile(ti, io.BytesIO(data))
                elif payload is not None:
                    data = f"{payload}\n".encode()
                    ti = tarfile.TarInfo(name=f"{stamp}/commit.txt")
                    ti.size = len(data)
                    tf.addfile(ti, io.BytesIO(data))
            raw = buf.getvalue()
            blob = subprocess.run(
                ["git", "-C", str(seed), "hash-object", "-w", "--stdin"],
                input=raw,
                capture_output=True,
                check=True,
                env=env,
            ).stdout.decode().strip()
            _git(seed, "update-index", "--add", "--cacheinfo", f"100644,{blob},{rel_path}", env=env)
            digest = (sidecar_override or {}).get(rel_path) or hashlib.sha256(raw).hexdigest()
            sc_bytes = f"{digest}  {pathlib.PurePosixPath(rel_path).name}\n".encode()
            sc_blob = subprocess.run(
                ["git", "-C", str(seed), "hash-object", "-w", "--stdin"],
                input=sc_bytes,
                capture_output=True,
                check=True,
                env=env,
            ).stdout.decode().strip()
            _git(
                seed,
                "update-index",
                "--add",
                "--cacheinfo",
                f"100644,{sc_blob},{rel_path}.sha256",
                env=env,
            )
        tree = _git(seed, "write-tree", env=env)
        if index.exists():
            index.unlink()
        args = ["commit-tree", tree, "-m", message]
        if parent:
            args += ["-p", parent]
        ev_sha = _git(seed, *args)
        _git(seed, "push", "origin", f"{ev_sha}:refs/heads/evidence")
        _git(seed, "fetch", "-q", "origin", "+refs/heads/evidence:refs/remotes/origin/evidence")
        return ev_sha

    def _tag_ready(self, recipe: str, sha: str, *, hold: bool = False) -> None:
        ref = f"refs/tags/{'hold' if hold else 'ready'}/{recipe}"
        _git(self.sb.seed, "push", "-f", "origin", f"{sha}:{ref}")

    def _run_brief_and_status(self, repo: pathlib.Path, *, can: set[str] | None = None, prs=None, pr_error=None):
        mod = _load_frontier_module(repo)
        pr_val = ([], None) if (prs is None and pr_error is None) else (prs, pr_error)
        with mock.patch.object(mod, "read_open_prs", return_value=pr_val):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                mod.brief(can or set())
            brief_out = buf.getvalue()
            status_out = mod.render()
        return brief_out, status_out

    def _section(self, text: str, heading: str) -> str:
        self.assertIn(heading, text)
        return text.split(heading, 1)[1].split("\n## ", 1)[0]

    # Fixture (1): promising probe with/without concrete successor & admission cap reached
    def test_r2r_01_promising_probe_successor_and_admission_cap(self) -> None:
        """Promising probe surfaces missing successor, admission cap, or actionable admit."""
        self._publish_evidence({})
        # (a) Promising probe with NO successor -> planned / needs concrete successor
        self._reset_state(
            {
                "probe-win": _claim_toml("probe-win", kind="probe", status="done", priority=2),
            },
            deltas={
                "2026-11-27-probe-win.md": (
                    "claim: probe-win\nby: antigravity\noutcome: done\n"
                    "evidence: docs/journal/probe-win.md\nverdict: probe-pass\n\n"
                    "**What was asked:** probe.\n"
                ),
            },
        )
        r = self.sb.clone("r2r-01a")
        brief_out, status_out = self._run_brief_and_status(r, can={"shell", "repo-write"})
        sec = self._section(brief_out, "## Result-to-release")
        self.assertIn("probe-win", sec)
        self.assertIn("planned", sec)
        self.assertIn("successor", sec)
        self.assertIn("probe-win", self._section(status_out, "## Result-to-release"))

        # (b) Linked idea successor when admission cap is reached -> explicit cap blocker
        self._reset_state(
            {
                "probe-win": _claim_toml("probe-win", kind="probe", status="done", priority=2),
                "build-win": _claim_toml(
                    "build-win",
                    kind="build",
                    zone="striated",
                    status="idea",
                    priority=2,
                    owned_paths=["src/block.rs"],
                    links=["frontier/claims/probe-win.toml"],
                ),
                "filler-open": _claim_toml("filler-open", kind="probe", status="open", priority=3),
            },
            deltas={
                "2026-11-27-probe-win.md": (
                    "claim: probe-win\nby: antigravity\noutcome: done\n"
                    "evidence: docs/journal/probe-win.md\nverdict: probe-pass\n\n"
                    "**What was asked:** probe.\n"
                ),
            },
            max_open_claims=1,
        )
        r2 = self.sb.clone("r2r-01b")
        brief2, _ = self._run_brief_and_status(r2, can={"shell", "repo-write"})
        sec2 = self._section(brief2, "## Result-to-release")
        self.assertIn("build-win", sec2)
        self.assertIn("admission cap", sec2)

        # (c) With room under max_open_claims -> actionable admit build-win
        self._reset_state(
            {
                "probe-win": _claim_toml("probe-win", kind="probe", status="done", priority=2),
                "build-win": _claim_toml(
                    "build-win",
                    kind="build",
                    zone="striated",
                    status="idea",
                    priority=2,
                    owned_paths=["src/block.rs"],
                    links=["frontier/claims/probe-win.toml"],
                ),
            },
            deltas={
                "2026-11-27-probe-win.md": (
                    "claim: probe-win\nby: antigravity\noutcome: done\n"
                    "evidence: docs/journal/probe-win.md\nverdict: probe-pass\n\n"
                    "**What was asked:** probe.\n"
                ),
            },
            max_open_claims=12,
        )
        r3 = self.sb.clone("r2r-01c")
        brief3, _ = self._run_brief_and_status(r3, can={"shell", "repo-write"})
        sec3 = self._section(brief3, "## Result-to-release")
        self.assertIn("build-win", sec3)
        self.assertIn("admit", sec3)
        self.assertNotIn("admission cap", sec3)

    # Fixture (2): ordinary CI-accepted default change requiring no separate i9 adoption run
    def test_r2r_02_ordinary_ci_default_change_needs_no_i9_run(self) -> None:
        """Ordinary CI-accepted default build requires no separate i9 adoption run."""
        self._publish_evidence({})
        ci_claim = (
            _claim_toml(
                "open-prescan-fix",
                kind="build",
                zone="striated",
                status="open",
                priority=2,
                owned_paths=["src/open.rs"],
                done_when="Merge on paired CI/native bench; no separate workstation run.",
            )
            + 'validation_kind = "ci"\n'
        )
        self._reset_state({"open-prescan-fix": ci_claim})
        r = self.sb.clone("r2r-02")
        brief_out, status_out = self._run_brief_and_status(r, can={"shell", "rust", "repo-write"})
        sec = self._section(brief_out, "## Result-to-release")
        self.assertIn("open-prescan-fix", sec)
        self.assertIn("ordinary-ci", sec)
        self.assertNotIn("missing queue row", sec)
        self.assertNotIn("missing ready/", sec)
        self.assertIn("ordinary-ci", self._section(status_out, "## Result-to-release"))

    # Fixture (3): reviewed recipe with missing queue row or pin
    def test_r2r_03_reviewed_recipe_missing_queue_row_or_pin(self) -> None:
        """Reviewed recipe with missing queue row or ready pin is prepared and decays to queued."""
        self._publish_evidence({})
        run_claim = (
            _claim_toml(
                "seam-i9-run",
                kind="run",
                zone="striated",
                status="open",
                priority=2,
                needs=["shell", "workflow-dispatch", "repo-write"],
                owned_paths=["bench/workstation-queue.txt"],
                done_when="Queue frozen i9 recipe and read bundle.",
            )
            + 'recipe = "bench-seam-test"\n'
        )
        sha = self._reset_state({"seam-i9-run": run_claim}, queue_text="# repo: lmmx/mibz2\n")
        r = self.sb.clone("r2r-03a")
        brief_a, _ = self._run_brief_and_status(r, can={"shell", "workflow-dispatch", "repo-write"})
        sec_a = self._section(brief_a, "## Result-to-release")
        self.assertIn("seam-i9-run", sec_a)
        self.assertIn("prepared", sec_a)
        self.assertIn("missing queue row", sec_a)
        self.assertIn("missing ready/bench-seam-test", sec_a)

        # Add queue row only (pin still missing)
        self._reset_state(
            {"seam-i9-run": run_claim},
            queue_text="# repo: lmmx/mibz2\nbench-seam-test master test\n",
        )
        r_b = self.sb.clone("r2r-03b")
        brief_b, _ = self._run_brief_and_status(r_b, can={"shell", "workflow-dispatch", "repo-write"})
        sec_b = self._section(brief_b, "## Result-to-release")
        self.assertIn("prepared", sec_b)
        self.assertIn("missing ready/bench-seam-test", sec_b)
        self.assertNotIn("missing queue row", sec_b)

        # Pin ready/bench-seam-test -> decays from prepared to queued
        self._tag_ready("bench-seam-test", sha)
        r_c = self.sb.clone("r2r-03c")
        brief_c, _ = self._run_brief_and_status(r_c, can={"shell", "workflow-dispatch", "repo-write"})
        sec_c = self._section(brief_c, "## Result-to-release")
        self.assertIn("seam-i9-run", sec_c)
        self.assertIn("queued", sec_c)
        self.assertNotIn("prepared", sec_c)

    # Fixture (4): queued current pin without evidence
    def test_r2r_04_queued_current_pin_without_evidence(self) -> None:
        """Queued current pin without archived evidence reports queued with no duplicate dispatch."""
        self._publish_evidence({})
        run_claim = (
            _claim_toml(
                "fwd-lanes-run",
                kind="run",
                zone="striated",
                status="blocked",
                priority=1,
                owned_paths=["bench/forward_lanes.py"],
                done_when="Read i9 bundle.",
            )
            + 'recipe = "bench-forward-lanes"\n'
        )
        sha = self._reset_state(
            {"fwd-lanes-run": run_claim},
            queue_text="# repo: lmmx/mibz2\nbench-forward-lanes master kill test\n",
        )
        self._tag_ready("bench-forward-lanes", sha)
        r = self.sb.clone("r2r-04")
        brief_out, status_out = self._run_brief_and_status(r, can={"shell", "repo-write"})
        sec = self._section(brief_out, "## Result-to-release")
        self.assertIn("fwd-lanes-run", sec)
        self.assertIn("queued", sec)
        self.assertNotIn("returned-unread", sec)
        self.assertIn("queued", self._section(status_out, "## Result-to-release"))

    # Fixture (5): matching archived unread evidence while another recipe is runnable, including parked claim
    def test_r2r_05_unread_evidence_while_another_recipe_runnable_including_parked(self) -> None:
        """Archived unread evidence for a parked claim surfaces P1 even when queue is replenished."""
        parked_claim = (
            _claim_toml(
                "adaptive-seam",
                kind="build",
                zone="striated",
                status="blocked",
                priority=2,
                needs=["shell", "repo-write"],
                owned_paths=["src/stream.rs"],
                done_when="Read archived i9 bundle and record verdict.",
            )
            + 'recipe = "bench-adaptive-seam"\n'
        )
        other_claim = (
            _claim_toml(
                "other-run",
                kind="run",
                zone="striated",
                status="blocked",
                priority=2,
                owned_paths=["bench/other.py"],
            )
            + 'recipe = "bench-other"\n'
        )
        sha = self._reset_state(
            {"adaptive-seam": parked_claim, "other-run": other_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-adaptive-seam master adaptive seam\n"
                "bench-other master other runnable recipe\n"
            ),
        )
        self._tag_ready("bench-adaptive-seam", sha)
        self._tag_ready("bench-other", sha)
        ev_sha = self._publish_evidence(
            {"adaptive-seam/run-20261127T192609Z.tar.gz": sha},
            message="evidence: adaptive-seam/run-20261127T192609Z",
        )
        r = self.sb.clone("r2r-05a")
        brief_a, status_a = self._run_brief_and_status(r, can={"shell", "repo-write"})
        # Queue is replenished by bench-other, yet unread evidence for parked adaptive-seam MUST surface at P1!
        self.assertIn("Repository queue: replenished", brief_a)
        sec_a = self._section(brief_a, "## Result-to-release")
        self.assertIn("adaptive-seam", sec_a)
        self.assertIn("returned-unread", sec_a)
        self.assertIn(ev_sha[:8], sec_a)
        self.assertIn("P1 effective", sec_a)
        self.assertIn("returned-unread", self._section(status_a, "## Result-to-release"))
        # Declared priority in TOML must remain untouched (priority = 2)
        self.assertIn("priority = 2", (r / "frontier/claims/adaptive-seam.toml").read_text())

        # Record disposition delta naming ev_sha -> returned-unread pressure decays cleanly!
        self._reset_state(
            {"adaptive-seam": parked_claim, "other-run": other_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-adaptive-seam master adaptive seam\n"
                "bench-other master other runnable recipe\n"
            ),
            deltas={
                "2026-11-27-adaptive-seam-verdict.md": (
                    f"claim: adaptive-seam\nby: antigravity\noutcome: done\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: not-adopted\n\n"
                    "**What was asked:** read bundle.\n"
                ),
            },
        )
        r2 = self.sb.clone("r2r-05b")
        brief_b, _ = self._run_brief_and_status(r2, can={"shell", "repo-write"})
        sec_b = self._section(brief_b, "## Result-to-release")
        self.assertNotIn("returned-unread", sec_b)

    # Fixture (6): diagnostic PASS needing adoption vs adoption PASS ready to integrate; never infer PASS from prose
    def test_r2r_06_diagnostic_pass_vs_adoption_pass_and_no_prose_inference(self) -> None:
        """Distinguish diagnostic-pass from adoption-pass and never infer PASS from prose."""
        diag_claim = (
            _claim_toml(
                "fwd-diag",
                kind="run",
                zone="striated",
                status="open",
                priority=2,
                owned_paths=["bench/fwd.py"],
            )
            + 'recipe = "bench-fwd"\nvalidation_kind = "diagnostic"\n'
        )
        adopt_claim = (
            _claim_toml(
                "a2-adopt",
                kind="build",
                zone="striated",
                status="review",
                pr=1117,
                priority=2,
                owned_paths=["src/block.rs"],
            )
            + 'recipe = "bench-a2"\nvalidation_kind = "adoption"\n'
        )
        prose_only_claim = (
            _claim_toml(
                "prose-only",
                kind="run",
                zone="striated",
                status="blocked",
                priority=2,
                owned_paths=["bench/prose.py"],
            )
            + 'recipe = "bench-prose"\nvalidation_kind = "adoption"\n'
        )
        sha = self._reset_state(
            {"fwd-diag": diag_claim, "a2-adopt": adopt_claim, "prose-only": prose_only_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-fwd master diag\n"
                "bench-a2 master adopt\n"
                "bench-prose master prose\n"
            ),
        )
        for rec in ("bench-fwd", "bench-a2", "bench-prose"):
            self._tag_ready(rec, sha)
        ev_sha = self._publish_evidence(
            {
                "fwd/run-20261127T100000Z.tar.gz": sha,
                "a2/run-20261127T100001Z.tar.gz": sha,
                "prose/run-20261127T100002Z.tar.gz": sha,
            }
        )
        self._reset_state(
            {"fwd-diag": diag_claim, "a2-adopt": adopt_claim, "prose-only": prose_only_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-fwd master diag\n"
                "bench-a2 master adopt\n"
                "bench-prose master prose\n"
            ),
            deltas={
                "2026-11-27-fwd-diag.md": (
                    f"claim: fwd-diag\nby: antigravity\noutcome: done\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: diagnostic-pass\n\n"
                    "**What was asked:** diagnostic.\n"
                ),
                "2026-11-27-a2-adopt.md": (
                    f"claim: a2-adopt\nby: antigravity\noutcome: done\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: adoption-pass\n\n"
                    "**What was asked:** adoption.\n"
                ),
                # Prose says PASS without structured verdict: header -> must NOT infer adoption-passed!
                "2026-11-27-prose-only.md": (
                    "claim: prose-only\nby: antigravity\noutcome: partial\n"
                    "evidence: docs/notes.md\n\n"
                    "**What I found:** Looks like PASS in prose only!\n"
                ),
            },
        )
        prs = [
            {
                "number": 1117,
                "title": "A2 fast",
                "body": "Claim: a2-adopt",
                "head": {"sha": sha},
                "base": {"ref": "master"},
                "state": "open",
                "draft": False,
            }
        ]
        r = self.sb.clone("r2r-06")
        brief_out, _ = self._run_brief_and_status(r, can={"shell", "repo-write"}, prs=prs)
        sec = self._section(brief_out, "## Result-to-release")
        self.assertIn("fwd-diag", sec)
        self.assertIn("diagnostic-pass", sec)
        self.assertIn("adoption", sec.split("fwd-diag", 1)[1].splitlines()[0])
        self.assertIn("a2-adopt", sec)
        self.assertIn("adoption-passed", sec)
        self.assertIn("#1117", sec.split("a2-adopt", 1)[1].splitlines()[0])
        # prose-only still has unread verified bundle and MUST NOT be promoted to adoption-passed
        prose_line = [ln for ln in sec.splitlines() if "prose-only" in ln][0]
        self.assertNotIn("adoption-passed", prose_line)
        self.assertNotIn("diagnostic-pass", prose_line)
        self.assertIn("returned-unread", prose_line)

    # Fixture (7): STOP/KILL/VOID/INCOMPLETE and superseded pins
    def test_r2r_07_stop_kill_void_incomplete_and_superseded_pins(self) -> None:
        """Handle STOP/VOID dispositions without promotion and ignore superseded pins."""
        stop_claim = (
            _claim_toml("stopped-line", kind="build", zone="striated", status="open", priority=2, owned_paths=["src/a.rs"])
            + 'recipe = "bench-stop"\n'
        )
        void_claim = (
            _claim_toml("void-run", kind="run", zone="striated", status="open", priority=2, owned_paths=["bench/v.py"])
            + 'recipe = "bench-void"\n'
        )
        sup_claim = (
            _claim_toml("superseded-run", kind="run", zone="striated", status="blocked", priority=2, owned_paths=["bench/s.py"])
            + 'recipe = "bench-sup"\n'
        )
        old_sha = self._reset_state(
            {"stopped-line": stop_claim, "void-run": void_claim, "superseded-run": sup_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-stop master stop\n"
                "bench-void master void\n"
                "bench-sup master sup\n"
            ),
        )
        ev_sha = self._publish_evidence(
            {
                "stop/run-20261127T010000Z.tar.gz": old_sha,
                "void/run-20261127T010001Z.tar.gz": old_sha,
                "sup/run-20261127T010002Z.tar.gz": old_sha,
            }
        )
        new_sha = self._reset_state(
            {"stopped-line": stop_claim, "void-run": void_claim, "superseded-run": sup_claim},
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-stop master stop\n"
                "bench-void master void\n"
                "bench-sup master sup\n"
            ),
            deltas={
                "2026-11-27-stopped-line.md": (
                    f"claim: stopped-line\nby: antigravity\noutcome: done\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: stop\n\n"
                    "**What was asked:** stop.\n"
                ),
                "2026-11-27-void-run.md": (
                    f"claim: void-run\nby: antigravity\noutcome: partial\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: void\n\n"
                    "**What was asked:** void.\n"
                ),
            },
        )
        self._tag_ready("bench-stop", old_sha)
        self._tag_ready("bench-void", old_sha)
        # Superseded pin: ready/bench-sup now points to new_sha != old_sha!
        self._tag_ready("bench-sup", new_sha)
        r = self.sb.clone("r2r-07")
        brief_out, _ = self._run_brief_and_status(r, can={"shell", "repo-write"})
        sec = self._section(brief_out, "## Result-to-release")
        stop_line = [ln for ln in sec.splitlines() if "stopped-line" in ln][0]
        self.assertIn("stop", stop_line.lower())
        self.assertNotIn("adoption-passed", stop_line)
        self.assertIn("dead-ends", stop_line.lower())
        void_line = [ln for ln in sec.splitlines() if "void-run" in ln][0]
        self.assertIn("void", void_line.lower())
        self.assertNotIn("adoption-passed", void_line)
        sup_line = [ln for ln in sec.splitlines() if "superseded-run" in ln][0]
        # Superseded bundle for old_sha MUST NOT match new_sha!
        self.assertIn("queued", sup_line)
        self.assertIn("superseded", sup_line.lower())
        self.assertNotIn("returned-unread", sup_line)

    # Fixture (8): accepted default merge awaiting tag + recalibration, then tagged release awaiting scaling/recording
    def test_r2r_08_default_merge_recalibration_and_post_tag_scaling(self) -> None:
        """Default merge triggers P1 release + recalibration; tagged release tracks scaling."""
        self._publish_evidence({})
        self._reset_state({}, ledger_rows=[])
        seed = self.sb.seed
        _git(seed, "tag", "v0.5.17")
        _git(seed, "push", "origin", "v0.5.17")
        # Land a default-state software change touching src/open.rs (#1222)
        (seed / "src").mkdir(exist_ok=True)
        (seed / "src" / "open.rs").write_text("// default change\n", encoding="utf-8")
        (seed / "frontier" / "ledger.tsv").write_text(
            "pr\tmerged\teffect\tstate\tmeasured\tevidence\n"
            "1222\t2026-11-27\tRecalibrates open admission.\tdefault\t-2% wall, VM\tPR #1222\n",
            encoding="utf-8",
        )
        _git(seed, "add", "src/open.rs", "frontier/ledger.tsv")
        _git(seed, "commit", "-m", "perf(open): recalibrate admission (#1222)")
        _git(seed, "push", "origin", "master")
        r = self.sb.clone("r2r-08a")
        brief_a, status_a = self._run_brief_and_status(r, can={"shell", "repo-write"})
        sec_a = self._section(brief_a, "## Result-to-release")
        self.assertIn("merged-default", sec_a)
        self.assertIn("v0.5.17", sec_a)
        self.assertIn("recalibration", sec_a.lower())
        self.assertIn("P1", sec_a)
        self.assertIn("merged-default", self._section(status_a, "## Result-to-release"))

        # Now tag v0.5.18 and open release-0-5-18 awaiting scaling/recording
        tag_sha = _git(seed, "rev-parse", "HEAD")
        _git(seed, "tag", "v0.5.18")
        _git(seed, "push", "origin", "v0.5.18")
        self._reset_state(
            {
                "release-0-5-18": _claim_toml(
                    "release-0-5-18",
                    kind="ops",
                    zone="striated",
                    status="open",
                    priority=1,
                    owned_paths=["CHANGELOG.md"],
                ),
            }
        )
        self._tag_ready("bench-release-scaling", tag_sha)
        r_b = self.sb.clone("r2r-08b")
        brief_b, _ = self._run_brief_and_status(r_b, can={"shell", "repo-write"})
        sec_b = self._section(brief_b, "## Result-to-release")
        self.assertIn("released", sec_b)
        self.assertIn("v0.5.18", sec_b)
        self.assertIn("scaling", sec_b.lower())

        # Once release-0-5-18 is marked done -> release scaling pressure decays!
        self._reset_state(
            {
                "release-0-5-18": _claim_toml(
                    "release-0-5-18",
                    kind="ops",
                    zone="striated",
                    status="done",
                    priority=1,
                    owned_paths=["CHANGELOG.md"],
                ),
            }
        )
        r_c = self.sb.clone("r2r-08c")
        brief_c, _ = self._run_brief_and_status(r_c, can={"shell", "repo-write"})
        sec_c = self._section(brief_c, "## Result-to-release")
        self.assertNotIn("merged-default", sec_c)
        self.assertNotIn("awaiting scaling", sec_c)

    def test_r2r_08b_live_untagged_release_is_not_released(self) -> None:
        """A live release claim whose own tag does not exist yet is in progress, not released."""
        self._publish_evidence({})
        self._reset_state({}, ledger_rows=[])
        seed = self.sb.seed
        _git(seed, "tag", "v0.5.17")
        _git(seed, "push", "origin", "v0.5.17")
        self._reset_state(
            {
                "release-0-5-18": _claim_toml(
                    "release-0-5-18",
                    kind="ops",
                    zone="striated",
                    status="blocked",
                    priority=1,
                    owned_paths=["CHANGELOG.md"],
                ),
            }
        )
        r = self.sb.clone("r2r-08d")
        brief_out, status_out = self._run_brief_and_status(r, can={"shell", "repo-write"})
        for text in (brief_out, status_out):
            sec = self._section(text, "## Result-to-release")
            self.assertIn("release-in-progress", sec)
            self.assertIn("v0.5.18 not yet tagged", sec)
            self.assertNotIn("`released`", sec)
            self.assertNotIn("tagged v0.5.17", sec)

    # Fixture (9): experimental-only merge does not by itself imply a production performance release
    def test_r2r_09_experimental_only_merge_does_not_imply_production_release(self) -> None:
        """Experimental-only merge does not trigger a production performance release."""
        self._publish_evidence({})
        self._reset_state({}, ledger_rows=[])
        seed = self.sb.seed
        _git(seed, "tag", "v0.5.17")
        _git(seed, "push", "origin", "v0.5.17")
        (seed / "src").mkdir(exist_ok=True)
        (seed / "src" / "lanes.rs").write_text("// experimental feature only\n", encoding="utf-8")
        (seed / "frontier" / "ledger.tsv").write_text(
            "pr\tmerged\teffect\tstate\tmeasured\tevidence\n"
            "1217\t2026-11-27\tExperimental forward lanes.\texperimental\t-10% VM\tPR #1217\n",
            encoding="utf-8",
        )
        _git(seed, "add", "src/lanes.rs", "frontier/ledger.tsv")
        _git(seed, "commit", "-m", "perf(open/lanes): disjoint slabs (#1217)")
        _git(seed, "push", "origin", "master")
        r = self.sb.clone("r2r-09")
        brief_out, status_out = self._run_brief_and_status(r, can={"shell", "repo-write"})
        # Release due MUST NOT fire for experimental-only merge!
        rel_sec = self._section(brief_out, "## Release due (P1)")
        self.assertIn("- nothing", rel_sec)
        sec = self._section(brief_out, "## Result-to-release")
        self.assertIn("experimental-only", sec)
        self.assertNotIn("merged-default", sec)
        self.assertNotIn("merged-default", self._section(status_out, "## Result-to-release"))

    # Fixture (10): missing/stale/ambiguous evidence, failed checks, owner-only, live hold, capability, cap, conflicting successors
    def test_r2r_10_fail_closed_unknown_blockers_caps_holds_capabilities_conflicts(self) -> None:
        """Fail closed on corrupt/missing data, failed checks, caps, holds, and conflicts."""
        # (a) Corrupt .sha256 sidecar -> fail closed to unknown
        bad_claim = (
            _claim_toml("corrupt-ev", kind="run", zone="striated", status="blocked", priority=2, owned_paths=["bench/c.py"])
            + 'recipe = "bench-corrupt"\n'
        )
        louis_claim = (
            _claim_toml(
                "owner-blocked",
                kind="build",
                zone="striated",
                status="open",
                priority=2,
                needs_louis="Requires owner API decision.",
                owned_paths=["src/api.rs"],
            )
            + 'validation_kind = "ci"\n'
        )
        cap_claim = (
            _claim_toml(
                "need-dispatch",
                kind="run",
                zone="striated",
                status="open",
                priority=2,
                needs=["shell", "workflow-dispatch"],
                owned_paths=["bench/d.py"],
            )
            + 'recipe = "bench-need-dispatch"\n'
        )
        failed_ci_claim = (
            _claim_toml(
                "failed-ci",
                kind="build",
                zone="striated",
                status="review",
                pr=999,
                priority=2,
                owned_paths=["src/f.rs"],
            )
            + 'recipe = "bench-fci"\nvalidation_kind = "adoption"\n'
        )
        # Conflicting successors for the same probe
        probe_conflict = _claim_toml("probe-c", kind="probe", status="done", priority=2)
        succ_a = _claim_toml(
            "succ-a",
            kind="build",
            zone="striated",
            status="idea",
            priority=2,
            owned_paths=["src/s.rs"],
            mechanisms=["same-mech"],
            links=["frontier/claims/probe-c.toml"],
        )
        succ_b = _claim_toml(
            "succ-b",
            kind="build",
            zone="striated",
            status="idea",
            priority=2,
            owned_paths=["src/s.rs"],
            mechanisms=["same-mech"],
            links=["frontier/claims/probe-c.toml"],
        )
        held_claim = (
            _claim_toml("held-ev", kind="run", zone="striated", status="open", priority=2, owned_paths=["bench/h.py"])
            + 'recipe = "bench-held"\n'
        )
        sha = self._reset_state(
            {
                "corrupt-ev": bad_claim,
                "owner-blocked": louis_claim,
                "need-dispatch": cap_claim,
                "failed-ci": failed_ci_claim,
                "probe-c": probe_conflict,
                "succ-a": succ_a,
                "succ-b": succ_b,
                "held-ev": held_claim,
            },
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-corrupt master c\n"
                "bench-fci master f\n"
                "bench-held master h\n"
            ),
            deltas={
                "2026-11-27-probe-c.md": (
                    "claim: probe-c\nby: antigravity\noutcome: done\n"
                    "evidence: docs/p.md\nverdict: probe-pass\n\n**What was asked:** p.\n"
                ),
            },
        )
        for rec in ("bench-corrupt", "bench-fci", "bench-held"):
            self._tag_ready(rec, sha)
        ev_sha = self._publish_evidence(
            {
                "corrupt/run-20261127T020000Z.tar.gz": sha,
                "fci/run-20261127T020001Z.tar.gz": sha,
                "held/run-20261127T020002Z.tar.gz": sha,
            },
            sidecar_override={"corrupt/run-20261127T020000Z.tar.gz": "0" * 64},
        )
        # Record adoption-pass on failed-ci, whose PR has failing checks
        self._reset_state(
            {
                "corrupt-ev": bad_claim,
                "owner-blocked": louis_claim,
                "need-dispatch": cap_claim,
                "failed-ci": failed_ci_claim,
                "probe-c": probe_conflict,
                "succ-a": succ_a,
                "succ-b": succ_b,
                "held-ev": held_claim,
            },
            queue_text=(
                "# repo: lmmx/mibz2\n"
                "bench-corrupt master c\n"
                "bench-fci master f\n"
                "bench-held master h\n"
            ),
            deltas={
                "2026-11-27-probe-c.md": (
                    "claim: probe-c\nby: antigravity\noutcome: done\n"
                    "evidence: docs/p.md\nverdict: probe-pass\n\n**What was asked:** p.\n"
                ),
                "2026-11-27-failed-ci.md": (
                    f"claim: failed-ci\nby: antigravity\noutcome: done\n"
                    f"evidence: origin/evidence {ev_sha[:8]}\nverdict: adoption-pass\n\n"
                    "**What was asked:** fci.\n"
                ),
            },
        )
        # Acquire a live hold on held-ev so we verify non-preemption & deduplication
        holder = self.sb.clone("holder-10")
        res = self.sb.run_frontier(holder, "claim", "held-ev", "--by", "claude-code")
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)

        prs = [
            {
                "number": 999,
                "title": "FCI",
                "body": "Claim: failed-ci",
                "head": {"sha": sha},
                "base": {"ref": "master"},
                "state": "open",
                "draft": False,
                "checks_failed": True,
            }
        ]
        reader = self.sb.clone("r2r-10")
        brief_out, _ = self._run_brief_and_status(reader, can={"shell", "repo-write"}, prs=prs)
        sec = self._section(brief_out, "## Result-to-release")
        # Corrupt sidecar -> fail closed to unknown
        corrupt_line = [ln for ln in sec.splitlines() if "corrupt-ev" in ln][0]
        self.assertIn("unknown", corrupt_line)
        # Owner-only blocker -> explicit needs_louis, no P1 effective
        owner_line = [ln for ln in sec.splitlines() if "owner-blocked" in ln][0]
        self.assertIn("needs_louis", owner_line)
        self.assertNotIn("P1 effective", owner_line)
        # Absent capability -> capability-blocked handoff
        cap_line = [ln for ln in sec.splitlines() if "need-dispatch" in ln][0]
        self.assertIn("workflow-dispatch", cap_line)
        self.assertIn("capability-blocked", cap_line)
        # Failed CI checks -> explicit blocker, never recommends merge
        fci_line = [ln for ln in sec.splitlines() if "failed-ci" in ln][0]
        self.assertIn("failed checks", fci_line)
        self.assertNotIn("P1 effective", fci_line)
        # Conflicting successors -> fail closed to unknown
        pc_line = [ln for ln in sec.splitlines() if "probe-c" in ln][0]
        self.assertIn("unknown", pc_line)
        self.assertIn("conflicting successors", pc_line)
        # Live hold -> held by claude-code, never preempted
        held_line = [ln for ln in sec.splitlines() if "held-ev" in ln][0]
        self.assertIn("held by claude-code", held_line)
        self.assertNotIn("P1 effective", held_line)

        # (b) Unavailable PR discovery -> fail closed to unknown
        brief_err, _ = self._run_brief_and_status(reader, can={"shell", "repo-write"}, prs=None, pr_error="gh offline")
        sec_err = self._section(brief_err, "## Result-to-release")
        self.assertIn("unknown", sec_err)

        # (c) Live-hold cap reached -> explicit live-hold cap blocker (no new acquisition elevated)
        self._reset_state(
            {
                "held-ev": held_claim,
                "ready-ci": _claim_toml("ready-ci", kind="build", zone="striated", status="open", priority=2, owned_paths=["src/r.rs"])
                + 'validation_kind = "ci"\n',
            },
            max_live_holds=1,
        )
        reader_cap = self.sb.clone("r2r-10-cap")
        brief_cap, _ = self._run_brief_and_status(reader_cap, can={"shell", "rust", "repo-write"})
        sec_cap = self._section(brief_cap, "## Result-to-release")
        self.assertIn("live-hold cap", sec_cap)


secrets_counter = [0]


if __name__ == "__main__":
    unittest.main()
