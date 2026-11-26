"""Adversarial stress tests for the frontier protocol (claims stress-test-frontier & harden-frontier-tooling).

Exercises all eight scenarios from `frontier/claims/stress-test-frontier.toml`
against isolated temporary bare git remotes so no debris refs are pushed to the
real `origin`. All protocol invariants and regression targets are asserted as
hard passes.
"""

from __future__ import annotations

import concurrent.futures
import datetime as dt
import fcntl
import importlib.util
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest import mock

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
FRONTIER_SRC = REPO_ROOT / "frontier" / "tools" / "frontier.py"
RUN_QUEUE_SH = REPO_ROOT / "bench" / "run-queue.sh"

GIT_ENV = {
    "GIT_AUTHOR_NAME": "Louis Maddox",
    "GIT_AUTHOR_EMAIL": "louismmx@gmail.com",
    "GIT_COMMITTER_NAME": "Louis Maddox",
    "GIT_COMMITTER_EMAIL": "louismmx@gmail.com",
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
    zone: str = "smooth",
    status: str = "open",
    owned_paths: list[str] | None = None,
    mechanisms: list[str] | None = None,
    needs_louis: str = "",
    premises: list[str] | None = None,
    stop_test: str = "PASS iff exact bytes match.",
) -> str:
    """Build a valid claim TOML document for test fixtures."""
    op = owned_paths or []
    mechs = mechanisms or []
    prems = premises or []
    return (
        f'id = "{cid}"\n'
        f'title = "Test claim {cid}"\n'
        'kind = "probe"\n'
        f'zone = "{zone}"\n'
        'needs = ["shell"]\n'
        "priority = 1\n"
        f'status = "{status}"\n'
        f"owned_paths = {op!r}\n".replace("'", '"')
        + f"mechanisms = {mechs!r}\n".replace("'", '"')
        + f'needs_louis = "{needs_louis}"\n'
        + f'question = "Test question for {cid}"\n'
        + 'why = "Test why"\n'
        + f'stop_test = "{stop_test}"\n'
        + 'done_when = "Delta committed."\n'
        + f"premises = {prems!r}\n".replace("'", '"')
        + "links = []\n"
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


if __name__ == "__main__":
    unittest.main()
