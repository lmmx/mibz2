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
import importlib.util
import io
import json
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
        news = 'news = "A new induction order rebuilds text with a third fewer steps."\nnews_kind = "discovery"\n'
        (blog / "b-draft.toml").write_text(card("b-draft", "draft", news))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        (blog / "b-other.toml").write_text(card("b-other", "outline", news.replace("discovery", "advance")))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("2 posts in progress", r.stdout)
        (blog / "b-other.toml").write_text(card("b-other", "outline", news + "legacy = true\n"))
        r = self.sb.run_frontier(a, "check")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        (blog / "b-other.toml").write_text(card("b-other", "outline", news.replace("discovery", "negative-result")))
        r = self.sb.run_frontier(a, "check")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("lab-notebook entry", r.stdout)

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


if __name__ == "__main__":
    unittest.main()
