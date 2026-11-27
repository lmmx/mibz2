"""bench/run_requests.py: declarative run requests on lightweight refs, reconciled into the
existing ready tags (claim declarative-ready-signal). Hermetic: a scratch bare origin for the
refs and tags, and a fake GitHub API for pull requests, reviews and checks. Nothing is
dispatched, nothing runs and no network is used.

    python3 bench/test_run_requests.py

Frozen before the implementation (stop_test of frontier/claims/declarative-ready-signal.toml).
"""

import datetime as dt
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run_requests as rr  # noqa: E402

# the module's own git calls (publish, withdraw) need an identity on a bare CI runner
for _k, _v in (
    ("GIT_AUTHOR_NAME", "t"),
    ("GIT_AUTHOR_EMAIL", "t@t"),
    ("GIT_COMMITTER_NAME", "t"),
    ("GIT_COMMITTER_EMAIL", "t@t"),
):
    os.environ.setdefault(_k, _v)

NOW = dt.datetime(2026, 11, 27, 9, 0, tzinfo=dt.timezone.utc)
RECIPE = "bench-demo"
BRANCH = "bench/demo"
CLAIM = "demo-queue"
SESSION = "abcdef123456"


def git(cwd, *args, input=None):
    env = dict(
        os.environ,
        GIT_AUTHOR_NAME="t",
        GIT_AUTHOR_EMAIL="t@t",
        GIT_COMMITTER_NAME="t",
        GIT_COMMITTER_EMAIL="t@t",
        GIT_CONFIG_GLOBAL="/dev/null",
        GIT_CONFIG_SYSTEM="/dev/null",
    )
    return subprocess.run(
        ["git", *args], cwd=cwd, env=env, check=True, capture_output=True, text=True, input=input
    ).stdout.strip()


class FakeAPI:
    """The GitHub API the reconciler reads, as plain data; a missing entry is an API error."""

    def __init__(self):
        self.prs, self.reviews, self.checks, self.statuses, self.suites = {}, {}, {}, {}, {}
        self.down = False

    def get(self, path):
        if self.down:
            raise rr.Unavailable(f"api down: {path}")
        parts = path.split("?")[0].strip("/").split("/")
        # repos/o/r/pulls/N[/reviews], repos/o/r/commits/S/{check-runs,check-suites,status}
        if parts[3] == "pulls" and len(parts) == 5:
            table, key = self.prs, int(parts[4])
        elif parts[3] == "pulls" and parts[5] == "reviews":
            table, key = self.reviews, int(parts[4])
        elif parts[5] == "check-runs":
            table, key = self.checks, parts[4]
        elif parts[5] == "check-suites":
            table, key = self.suites, parts[4]
        elif parts[5] == "status":
            table, key = self.statuses, parts[4]
        else:
            raise AssertionError(path)
        if key not in table:
            raise rr.Unavailable(f"404 {path}")
        return table[key]


def green():
    """A head that is green as CLAUDE.md defines it: one finished Actions suite whose runs
    passed or were skipped, the required checks among them, and pre-commit.ci's status."""
    suite = {"id": 1}
    runs = [
        ("rust", "success"),
        ("bench", "skipped"),
        ("check-commit", "success"),
        ("no-ai-attribution", "success"),
    ]
    return (
        {
            "total_count": len(runs),
            "check_runs": [
                {"name": n, "status": "completed", "conclusion": c, "check_suite": suite}
                for n, c in runs
            ],
        },
        {
            "total_count": 1,
            "check_suites": [
                {
                    "id": 1,
                    "status": "completed",
                    "conclusion": "success",
                    "app": {"slug": "github-actions"},
                }
            ],
        },
        {
            "state": "success",
            "total_count": 1,
            "statuses": [{"context": "pre-commit.ci - pr", "state": "success"}],
        },
    )


class Fixture(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.origin = self.tmp / "origin.git"
        git(self.tmp, "init", "-q", "--bare", "-b", "master", str(self.origin))
        self.work = self.tmp / "work"
        git(self.tmp, "clone", "-q", str(self.origin), str(self.work))
        w = self.work
        (w / "bench").mkdir()
        (w / "frontier" / "claims").mkdir(parents=True)
        (w / "bench" / "workstation-queue.txt").write_text(
            f"# repo: o/r\n# channel: 1\n{RECIPE:<28} {BRANCH:<24} the demo recipe\n"
        )
        self.write_claim()
        git(w, "add", "-A")
        git(w, "commit", "-qm", "master")
        git(w, "push", "-q", "origin", "master")
        git(w, "checkout", "-qb", BRANCH)
        (w / "recipe.txt").write_text("candidate\n")
        git(w, "add", "-A")
        git(w, "commit", "-qm", "candidate")
        self.sha = git(w, "rev-parse", "HEAD")
        git(w, "push", "-q", "origin", BRANCH)
        git(w, "checkout", "-q", "master")
        self.set_lease()
        self.api = FakeAPI()
        self.api.prs[7] = {
            "state": "open",
            "draft": False,
            "merged": False,
            "base": {"ref": "master"},
            "head": {"sha": self.sha, "ref": BRANCH, "repo": {"full_name": "o/r"}},
        }
        self.api.reviews[7] = [
            {
                "id": 99,
                "commit_id": self.sha,
                "state": "COMMENTED",
                "body": "Independent recipe review passed.",
            }
        ]
        self.make_green(self.sha)

    def make_green(self, sha):
        self.api.checks[sha], self.api.suites[sha], self.api.statuses[sha] = green()

    def write_claim(self, **over):
        fields = {
            "id": CLAIM,
            "status": "open",
            "mechanisms": ["workstation-queue"],
            "owned_paths": ["bench/workstation-queue.txt"],
            "stop_test": f"Queue {RECIPE} once, and bench-other too.",
        }
        fields.update(over)
        lines = [f"{k} = {json.dumps(v)}" for k, v in fields.items()]
        (self.work / "frontier" / "claims" / f"{CLAIM}.toml").write_text("\n".join(lines) + "\n")

    def set_lease(
        self, session=SESSION, expires="2026-11-27T12:00:00+00:00", released=False, claim=CLAIM
    ):
        lease = (
            f'id = "{claim}"\nholder = "x"\nsession = "{session}"\n'
            f'claimed_at = "2026-11-27T06:00:00+00:00"\nexpires = "{expires}"\n'
            f'owned_paths = ["bench/workstation-queue.txt"]\nreleased = {str(released).lower()}\n'
        )
        blob = git(self.work, "hash-object", "-w", "--stdin", input=lease)
        tree = git(self.work, "mktree", input=f"100644 blob {blob}\tlease.toml\n")
        commit = git(self.work, "commit-tree", tree, "-m", "lease")
        git(self.work, "push", "-qf", "origin", f"{commit}:refs/heads/hold/{claim}")

    def world(self):
        return rr.World(remote=str(self.origin), repo="o/r", api=self.api, cwd=self.work)

    def publish(self, **over):
        fields = dict(
            recipe=RECIPE,
            sha=self.sha,
            branch=BRANCH,
            claim=CLAIM,
            session=SESSION,
            pr=7,
            review=99,
        )
        fields.update(over)
        return rr.publish(remote=str(self.origin), cwd=self.work, now=NOW, **fields)

    def reconcile(self):
        return rr.reconcile(self.world(), now=NOW, apply=True)

    def tag(self, name):
        out = git(
            self.work, "ls-remote", str(self.origin), f"refs/tags/{name}", f"refs/tags/{name}^{{}}"
        )
        lines = dict(reversed(line.split("\t")) for line in out.splitlines())
        return lines.get(f"refs/tags/{name}^{{}}") or lines.get(f"refs/tags/{name}")


class Activation(Fixture):
    def test_eligible_request_becomes_the_ready_tag_with_a_receipt(self):
        ref = self.publish()
        self.assertTrue(ref.startswith(f"refs/heads/run-request/{RECIPE}/"))
        report = self.reconcile()
        self.assertEqual([d.outcome for d in report], ["activated"], report)
        self.assertEqual(self.tag(f"ready/{RECIPE}"), self.sha)
        self.assertEqual(self.tag(f"run-receipt/{RECIPE}/{self.sha}"), self.sha)

    def test_reconcile_is_idempotent(self):
        self.publish()
        self.reconcile()
        report = self.reconcile()
        self.assertEqual([d.outcome for d in report], ["done"], report)

    def test_dry_run_changes_nothing(self):
        self.publish()
        report = rr.reconcile(self.world(), now=NOW, apply=False)
        self.assertEqual([d.outcome for d in report], ["eligible"])
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_concurrent_publication_of_two_recipes_is_batched(self):
        other = "bench-other"
        q = self.work / "bench" / "workstation-queue.txt"
        q.write_text(q.read_text() + f"{other:<28} {BRANCH:<24} another\n")
        git(self.work, "commit", "-qam", "q")
        git(self.work, "push", "-q", "origin", "master")
        a = self.publish()
        b = self.publish(recipe=other)
        self.assertNotEqual(a, b)
        report = self.reconcile()
        self.assertEqual(sorted(d.outcome for d in report), ["activated", "activated"], report)
        self.assertEqual(self.tag(f"ready/{other}"), self.sha)

    def test_two_requests_for_the_same_run_activate_once(self):
        self.publish()
        self.publish()
        outcomes = sorted(d.outcome for d in self.reconcile())
        self.assertEqual(outcomes, ["activated", "done"], outcomes)


class Refusal(Fixture):
    def refused(self, needle):
        before = self.tag(f"ready/{RECIPE}")
        report = self.reconcile()
        self.assertEqual(len(report), 1, report)
        self.assertEqual(report[0].outcome, "refused", report)
        self.assertIn(needle, report[0].reason)
        self.assertEqual(self.tag(f"ready/{RECIPE}"), before)  # refused: nothing moved

    def test_hold_tag_takes_precedence(self):
        git(self.work, "push", "-q", "origin", f"{self.sha}:refs/tags/hold/{RECIPE}")
        self.publish()
        self.refused("hold")

    def test_no_queue_entry(self):
        self.publish(recipe="bench-unlisted")
        self.refused("queue")

    def test_branch_mismatch_with_queue(self):
        self.publish(branch="bench/elsewhere")
        self.refused("queue")

    def test_commit_not_on_branch(self):
        # a commit on origin but not on the branch (master's commit is the branch's ancestor)
        tree = git(self.work, "rev-parse", "master^{tree}")
        stray = git(self.work, "commit-tree", tree, "-m", "stray")
        git(self.work, "push", "-q", "origin", f"{stray}:refs/heads/elsewhere")
        self.api.prs[7]["head"]["sha"] = stray
        self.publish(sha=stray)
        self.refused("not on")

    def test_moved_pr_head(self):
        self.api.prs[7]["head"]["sha"] = "f" * 40
        self.publish()
        self.refused("head")

    def test_stale_review(self):
        self.api.reviews[7][0]["commit_id"] = "e" * 40
        self.publish()
        self.refused("review")

    def test_changes_requested_at_head(self):
        self.api.reviews[7].append(
            {"id": 100, "commit_id": self.sha, "state": "CHANGES_REQUESTED", "body": "no"}
        )
        self.publish()
        self.refused("changes")

    def test_failed_check(self):
        self.api.checks[self.sha]["check_runs"][0]["conclusion"] = "failure"
        self.publish()
        self.refused("check")

    def test_pending_check(self):
        self.api.checks[self.sha]["check_runs"][0].update(status="in_progress", conclusion=None)
        self.publish()
        self.refused("check")

    def test_no_checks_at_all(self):
        self.api.checks[self.sha] = {"total_count": 0, "check_runs": []}
        self.publish()
        self.refused("check")

    def test_status_error_holds_like_a_red_check(self):
        self.api.statuses[self.sha] = {"state": "error", "total_count": 1}
        self.publish()
        self.refused("status")

    def test_macos_real_failure_holds(self):
        self.api.checks[self.sha]["check_runs"].append(
            {"name": "wheel (macos-14)", "status": "completed", "conclusion": "failure"}
        )
        self.publish()
        self.refused("check")

    def test_macos_waiting_for_a_runner_does_not_hold(self):
        self.api.checks[self.sha]["check_runs"].append(
            {
                "name": "wheel (macos-14)",
                "status": "queued",
                "conclusion": None,
                "check_suite": {"id": 2},
            }
        )
        self.api.suites[self.sha]["check_suites"].append(
            {"id": 2, "status": "in_progress", "app": {"slug": "github-actions"}}
        )
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_macos_started_holds(self):
        self.api.checks[self.sha]["check_runs"].append(
            {"name": "macos (macos-15)", "status": "in_progress", "conclusion": None}
        )
        self.publish()
        self.refused("check")

    def test_macos_timed_out_holds(self):
        self.api.checks[self.sha]["check_runs"].append(
            {"name": "macos (macos-15)", "status": "completed", "conclusion": "timed_out"}
        )
        self.publish()
        self.refused("check")

    def test_cancelled_duplicate_of_a_passed_job_does_not_hold(self):
        self.api.checks[self.sha]["check_runs"].append(
            {"name": "bench", "status": "completed", "conclusion": "cancelled"}
        )
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_cancelled_twin_in_another_workflow_does_not_hold(self):
        # as on #1104: `mirror` cancelled in one workflow run, skipped in another
        self.api.checks[self.sha]["check_runs"].append(
            {
                "name": "bench",
                "status": "completed",
                "conclusion": "cancelled",
                "check_suite": {"id": 9},
            }
        )
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_cancelled_job_without_a_passed_twin_holds(self):
        self.api.checks[self.sha]["check_runs"].append(
            {"name": "miri", "status": "completed", "conclusion": "cancelled"}
        )
        self.publish()
        self.refused("check")

    def test_required_check_absent_holds(self):
        runs = self.api.checks[self.sha]["check_runs"]
        runs[:] = [r for r in runs if r["name"] != "no-ai-attribution"]
        self.api.checks[self.sha]["total_count"] = len(runs)
        self.publish()
        self.refused("required")

    def test_required_status_absent_holds(self):
        self.api.statuses[self.sha] = {"state": "pending", "total_count": 0, "statuses": []}
        self.publish()
        self.refused("pre-commit.ci")

    def test_workflow_not_finished_holds(self):
        self.api.suites[self.sha]["check_suites"].append(
            {"id": 3, "status": "queued", "app": {"slug": "github-actions"}}
        )
        self.publish()
        self.refused("workflow")

    def test_malformed_lease_refuses_without_stopping_the_batch(self):
        blob = git(self.work, "hash-object", "-w", "--stdin", input="not [[ toml")
        tree = git(self.work, "mktree", input=f"100644 blob {blob}\tlease.toml\n")
        commit = git(self.work, "commit-tree", tree, "-m", "bad lease")
        git(self.work, "push", "-qf", "origin", f"{commit}:refs/heads/hold/{CLAIM}")
        self.publish()
        self.refused("lease")

    def test_claim_not_naming_the_recipe(self):
        self.write_claim(stop_test="Queue bench-demo-two only.")
        git(self.work, "commit", "-qam", "c")
        git(self.work, "push", "-q", "origin", "master")
        self.publish()
        self.refused("does not name")

    def test_claim_with_unknown_status(self):
        self.write_claim(status="paused")
        git(self.work, "commit", "-qam", "c")
        git(self.work, "push", "-q", "origin", "master")
        self.publish()
        self.refused("claim")

    def test_fork_pr_with_the_same_branch_name(self):
        self.api.prs[7]["head"]["repo"]["full_name"] = "someone/fork"
        self.publish()
        self.refused("head")

    def test_closed_unmerged_pr(self):
        self.api.prs[7]["state"] = "closed"
        self.publish()
        self.refused("closed")

    def test_boolean_version_is_refused(self):
        with self.assertRaises(rr.Refused):
            rr.parse(
                rr.render(dict(version=True, action="hold", recipe=RECIPE, created=NOW.isoformat()))
            )

    def test_api_unavailable_fails_closed(self):
        self.api.down = True
        self.publish()
        self.refused("unavailable")

    def test_lease_session_mismatch(self):
        self.publish(session="000000000000")
        self.refused("lease")

    def test_lease_expired(self):
        self.set_lease(expires="2026-11-27T08:00:00+00:00")
        self.publish()
        self.refused("lease")

    def test_lease_released(self):
        self.set_lease(released=True)
        self.publish()
        self.refused("lease")

    def test_claim_without_queue_mechanism(self):
        self.write_claim(mechanisms=["other"], owned_paths=["src/"])
        git(self.work, "commit", "-qam", "c")
        git(self.work, "push", "-q", "origin", "master")
        self.publish()
        self.refused("claim")

    def test_claim_needing_louis(self):
        self.write_claim(needs_louis="a decision")
        git(self.work, "commit", "-qam", "c")
        git(self.work, "push", "-q", "origin", "master")
        self.publish()
        self.refused("claim")

    def test_ready_already_elsewhere_without_supersedes(self):
        other = git(self.work, "rev-parse", "master")
        git(self.work, "push", "-q", "origin", f"{other}:refs/tags/ready/{RECIPE}")
        self.publish()
        self.refused("ready/")
        self.assertEqual(self.tag(f"ready/{RECIPE}"), other)

    def test_expired_request(self):
        rr.publish(
            remote=str(self.origin),
            cwd=self.work,
            now=NOW - dt.timedelta(hours=73),
            recipe=RECIPE,
            sha=self.sha,
            branch=BRANCH,
            claim=CLAIM,
            session=SESSION,
            pr=7,
            review=99,
        )
        self.refused("expired")


class Malformed(Fixture):
    def raw_request(self, text, recipe=RECIPE, name="0123456789ab"):
        blob = git(self.work, "hash-object", "-w", "--stdin", input=text)
        tree = git(self.work, "mktree", input=f"100644 blob {blob}\trequest.toml\n")
        commit = git(self.work, "commit-tree", tree, "-m", "request")
        git(self.work, "push", "-q", "origin", f"{commit}:refs/heads/run-request/{recipe}/{name}")

    def test_malformed_requests_are_refused_not_crashing(self):
        good = rr.render(
            dict(
                version=1,
                action="ready",
                recipe=RECIPE,
                sha=self.sha,
                branch=BRANCH,
                claim=CLAIM,
                session=SESSION,
                pr=7,
                review=99,
                created=NOW.isoformat(),
            )
        )
        cases = {
            "aaaaaaaaaaa1": "not toml [[[",
            "aaaaaaaaaaa2": good.replace(self.sha, self.sha[:12]),
            "aaaaaaaaaaa3": good.replace(f'"{BRANCH}"', '"bench/demo; rm -rf /"'),
            "aaaaaaaaaaa4": good.replace('action = "ready"', 'action = "clear"'),
            "aaaaaaaaaaa5": good + 'extra = "field"\n',
            "aaaaaaaaaaa6": "x" * 5000,
        }
        for name, text in cases.items():
            self.raw_request(text, name=name)
        self.raw_request(good, recipe="bench-other", name="aaaaaaaaaaa7")  # ref/file mismatch
        report = self.reconcile()
        self.assertEqual(len(report), 7, report)
        self.assertTrue(all(d.outcome == "refused" for d in report), report)
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_bad_ref_names_are_ignored(self):
        good = rr.render(
            dict(
                version=1,
                action="ready",
                recipe=RECIPE,
                sha=self.sha,
                branch=BRANCH,
                claim=CLAIM,
                session=SESSION,
                pr=7,
                review=99,
                created=NOW.isoformat(),
            )
        )
        self.raw_request(good, recipe="Bad_Recipe", name="0123456789ab")
        self.raw_request(good, name="short")
        self.assertEqual(self.reconcile(), [])


class Replay(Fixture):
    def test_cleared_run_is_not_reactivated_by_the_old_request(self):
        self.publish()
        self.reconcile()
        git(self.work, "push", "-q", "origin", "--delete", f"refs/tags/ready/{RECIPE}")
        report = self.reconcile()
        self.assertEqual([d.outcome for d in report], ["done"], report)
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_new_request_for_a_received_run_is_done_not_rerun(self):
        self.publish()
        self.reconcile()
        git(self.work, "push", "-q", "origin", "--delete", f"refs/tags/ready/{RECIPE}")
        self.publish()
        self.assertEqual(sorted(d.outcome for d in self.reconcile()), ["done", "done"])
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_withdrawn_request_is_not_activated(self):
        ref = self.publish()
        rr.withdraw(remote=str(self.origin), cwd=self.work, ref=ref, now=NOW)
        self.assertEqual([d.outcome for d in self.reconcile()], ["withdrawn"])
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_supersedes_moves_ready_from_a_received_run(self):
        old = git(self.work, "rev-parse", "master")
        git(self.work, "push", "-q", "origin", f"{old}:refs/tags/ready/{RECIPE}")
        self.publish(supersedes=old)
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])
        self.assertEqual(self.tag(f"ready/{RECIPE}"), self.sha)

    def test_an_older_request_does_not_revive_a_cleared_run(self):
        old = self.publish(note="older, for the same recipe")
        git(self.work, "checkout", "-q", BRANCH)
        (self.work / "recipe.txt").write_text("candidate 2\n")
        git(self.work, "commit", "-qam", "candidate 2")
        sha2 = git(self.work, "rev-parse", "HEAD")
        git(self.work, "push", "-q", "origin", BRANCH)
        git(self.work, "checkout", "-q", "master")
        self.make_green(sha2)
        self.api.prs[7]["head"]["sha"] = sha2
        self.api.reviews[7].append(
            {
                "id": 101,
                "commit_id": sha2,
                "state": "COMMENTED",
                "body": "Reviewed at the new head.",
            }
        )
        later = NOW + dt.timedelta(minutes=5)
        rr.publish(
            remote=str(self.origin),
            cwd=self.work,
            now=later,
            recipe=RECIPE,
            sha=sha2,
            branch=BRANCH,
            claim=CLAIM,
            session=SESSION,
            pr=7,
            review=101,
        )
        report = {d.ref: d for d in rr.reconcile(self.world(), now=later, apply=True)}
        self.assertEqual(report[old].outcome, "refused")  # the PR's head had moved on
        self.assertEqual(self.tag(f"ready/{RECIPE}"), sha2)
        git(self.work, "push", "-q", "origin", "--delete", f"refs/tags/ready/{RECIPE}")
        self.api.prs[7]["head"]["sha"] = self.sha  # even were the old head eligible again
        report = {d.ref: d for d in rr.reconcile(self.world(), now=later, apply=True)}
        self.assertEqual(report[old].outcome, "refused", report[old])
        self.assertIn("published before", report[old].reason)
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_supersedes_must_match_the_current_tag(self):
        git(
            self.work,
            "push",
            "-q",
            "origin",
            f"{git(self.work, 'rev-parse', 'master')}:refs/tags/ready/{RECIPE}",
        )
        self.publish(supersedes="d" * 40)
        report = self.reconcile()
        self.assertEqual(report[0].outcome, "refused")
        self.assertIn("supersedes", report[0].reason)


class Hold(Fixture):
    def test_hold_request_sets_hold_without_review_or_ci(self):
        self.api.down = True  # a hold needs no API: it only ever stops work
        rr.publish(remote=str(self.origin), cwd=self.work, now=NOW, action="hold", recipe=RECIPE)
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])
        self.assertIsNotNone(self.tag(f"hold/{RECIPE}"))

    def test_a_cleared_hold_is_not_reapplied(self):
        rr.publish(remote=str(self.origin), cwd=self.work, now=NOW, action="hold", recipe=RECIPE)
        self.reconcile()
        git(self.work, "push", "-q", "origin", "--delete", f"refs/tags/hold/{RECIPE}")
        self.assertEqual([d.outcome for d in self.reconcile()], ["done"])
        self.assertIsNone(self.tag(f"hold/{RECIPE}"))

    def test_a_hold_in_the_same_pass_as_a_ready_request_wins(self):
        self.publish()  # the ready request, published first
        rr.publish(
            remote=str(self.origin),
            cwd=self.work,
            now=NOW + dt.timedelta(minutes=5),
            action="hold",
            recipe=RECIPE,
        )
        report = rr.reconcile(self.world(), now=NOW + dt.timedelta(minutes=10), apply=True)
        self.assertIsNotNone(self.tag(f"hold/{RECIPE}"))
        self.assertIsNone(self.tag(f"ready/{RECIPE}"))
        self.assertEqual(sorted(d.outcome for d in report), ["activated", "refused"])

    def test_a_ready_receipt_does_not_retire_a_later_pass_hold(self):
        self.publish()
        self.reconcile()  # ready activated, receipt written at NOW
        early = NOW - dt.timedelta(minutes=1)  # a hold published before that receipt
        rr.publish(remote=str(self.origin), cwd=self.work, now=early, action="hold", recipe=RECIPE)
        self.reconcile()
        self.assertIsNotNone(self.tag(f"hold/{RECIPE}"))

    def test_hold_request_for_unqueued_recipe_is_refused(self):
        rr.publish(
            remote=str(self.origin), cwd=self.work, now=NOW, action="hold", recipe="bench-unlisted"
        )
        self.assertEqual([d.outcome for d in self.reconcile()], ["refused"])


class SecurityHardening(Fixture):
    """Regression tests for the 4 security/protocol defects reproduced against #1104 (01181d78)."""

    def _commit_flag(self, filename, body):
        flags_dir = self.work / "frontier" / "flags"
        flags_dir.mkdir(parents=True, exist_ok=True)
        (flags_dir / filename).write_text(body)
        git(self.work, "add", "frontier/flags")
        git(self.work, "commit", "-qm", f"flag {filename}")
        git(self.work, "push", "-q", "origin", "master")

    def test_directory_request_toml_refused_without_halting_other_requests(self):
        """A directory named request.toml/ must be refused individually, never crashing the batch."""
        good_ref = self.publish()
        blob = git(self.work, "hash-object", "-w", "--stdin", input="x\n")
        sub = git(self.work, "mktree", input=f"100644 blob {blob}\tinner\n")
        root = git(self.work, "mktree", input=f"040000 tree {sub}\trequest.toml\n")
        commit = git(self.work, "commit-tree", root, "-m", "dir request")
        bad_ref = f"refs/heads/run-request/{RECIPE}/deadbeefcafe"
        git(self.work, "push", "-q", "origin", f"{commit}:{bad_ref}")

        report = {d.ref: d for d in self.reconcile()}
        self.assertEqual(report[bad_ref].outcome, "refused", report[bad_ref])
        self.assertEqual(report[good_ref].outcome, "activated", report[good_ref])
        self.assertEqual(self.tag(f"ready/{RECIPE}"), self.sha)

    def test_symlink_request_toml_refused_without_halting_other_requests(self):
        """A symlink (mode 120000) named request.toml must be refused without halting valid work."""
        good_ref = self.publish()
        target = git(self.work, "hash-object", "-w", "--stdin", input="other.toml")
        root = git(self.work, "mktree", input=f"120000 blob {target}\trequest.toml\n")
        commit = git(self.work, "commit-tree", root, "-m", "symlink request")
        bad_ref = f"refs/heads/run-request/{RECIPE}/cafebabedead"
        git(self.work, "push", "-q", "origin", f"{commit}:{bad_ref}")

        report = {d.ref: d for d in self.reconcile()}
        self.assertEqual(report[bad_ref].outcome, "refused", report[bad_ref])
        self.assertEqual(report[good_ref].outcome, "activated", report[good_ref])

    def test_completed_non_passing_check_suites_with_zero_runs_refused(self):
        """Completed Actions suites with startup_failure/failure/cancelled (0 runs) fail closed."""
        for bad_conclusion in (
            "startup_failure",
            "failure",
            "timed_out",
            "action_required",
            "cancelled",
            None,
        ):
            with self.subTest(conclusion=bad_conclusion):
                self.setUp()
                self.api.suites[self.sha]["check_suites"].append(
                    {
                        "id": 99,
                        "status": "completed",
                        "conclusion": bad_conclusion,
                        "app": {"slug": "github-actions"},
                    }
                )
                self.publish()
                report = self.reconcile()
                self.assertEqual(report[0].outcome, "refused", report)
                self.assertIn("completed with", report[0].reason)
                self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_completed_cancelled_suite_with_passed_twins_allowed(self):
        """A cancelled suite whose runs all have passed twins still activates cleanly."""
        self.api.suites[self.sha]["check_suites"].append(
            {
                "id": 9,
                "status": "completed",
                "conclusion": "cancelled",
                "app": {"slug": "github-actions"},
            }
        )
        self.api.checks[self.sha]["check_runs"].append(
            {
                "name": "bench",
                "status": "completed",
                "conclusion": "cancelled",
                "check_suite": {"id": 9},
            }
        )
        self.api.checks[self.sha]["total_count"] += 1
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_required_check_skipped_and_neutral_refused(self):
        """Required checks (check-commit, no-ai-attribution) must succeed, not skip or neutral."""
        for req_name, bad_concl in (
            ("no-ai-attribution", "skipped"),
            ("check-commit", "neutral"),
            ("no-ai-attribution", "neutral"),
        ):
            with self.subTest(check=req_name, conclusion=bad_concl):
                self.setUp()
                for r in self.api.checks[self.sha]["check_runs"]:
                    if r["name"] == req_name:
                        r["conclusion"] = bad_concl
                self.publish()
                report = self.reconcile()
                self.assertEqual(report[0].outcome, "refused", report)
                self.assertIn("required", report[0].reason)
                self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_required_check_cancelled_duplicate_with_success_twin_allowed(self):
        """A superseded cancelled duplicate of a required check is allowed if a success twin exists."""
        self.api.checks[self.sha]["check_runs"].append(
            {
                "name": "no-ai-attribution",
                "status": "completed",
                "conclusion": "cancelled",
                "check_suite": {"id": 8},
            }
        )
        self.api.checks[self.sha]["total_count"] += 1
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_parked_lease_refused_and_integration_lease_allowed(self):
        """Parked leases (yield-review) are refused; active integration leases remain accepted."""
        for phase, expected in (("parked", "refused"), ("integration", "activated")):
            with self.subTest(phase=phase):
                self.setUp()
                lease = (
                    f'id = "{CLAIM}"\nholder = "x"\nsession = "{SESSION}"\n'
                    f'phase = "{phase}"\n'
                    f'claimed_at = "2026-11-27T06:00:00+00:00"\n'
                    f'expires = "2026-11-27T12:00:00+00:00"\n'
                    f'owned_paths = ["bench/workstation-queue.txt"]\nreleased = false\n'
                )
                blob = git(self.work, "hash-object", "-w", "--stdin", input=lease)
                tree = git(self.work, "mktree", input=f"100644 blob {blob}\tlease.toml\n")
                commit = git(self.work, "commit-tree", tree, "-m", f"lease {phase}")
                git(self.work, "push", "-qf", "origin", f"{commit}:refs/heads/hold/{CLAIM}")
                self.publish()
                report = self.reconcile()
                self.assertEqual([d.outcome for d in report], [expected], report)

    def test_open_flags_block_claim_recipe_or_owned_path_but_not_resolved_or_unrelated(self):
        """Open flags matching claim, recipe, or owned path refuse; unrelated or resolved pass."""
        blocking = [
            ("2026-11-27-claim.md", f"flags: {CLAIM}\ninvariant: test\nby: a\nstatus: open\n"),
            ("2026-11-27-recipe.md", f"flags: {RECIPE}\ninvariant: test\nby: a\nstatus: open\n"),
            (
                "2026-11-27-path.md",
                "flags: bench/workstation-queue.txt\ninvariant: test\nby: a\nstatus: open\n",
            ),
            (
                "2026-11-27-bad-status.md",
                f"flags: {CLAIM}\ninvariant: test\nby: a\nstatus: unknown\n",
            ),
        ]
        for fname, body in blocking:
            with self.subTest(blocking=fname):
                self.setUp()
                self._commit_flag(fname, body)
                self.publish()
                report = self.reconcile()
                self.assertEqual(report[0].outcome, "refused", report)
                self.assertIn("open flag", report[0].reason)
                self.assertIsNone(self.tag(f"ready/{RECIPE}"))

        # Resolved flag on claim + unrelated open flag on another claim must NOT block
        self.setUp()
        self._commit_flag(
            "2026-11-27-resolved.md",
            f"flags: {CLAIM}\ninvariant: test\nby: a\nstatus: resolved\nresolved_by: b\n",
        )
        self._commit_flag(
            "2026-11-27-unrelated.md",
            "flags: other-claim-id\ninvariant: test\nby: a\nstatus: open\n",
        )
        self.publish()
        self.assertEqual([d.outcome for d in self.reconcile()], ["activated"])

    def test_draft_pr_and_non_default_base_branch_refused(self):
        """Draft PRs, missing draft field, and PRs targeting non-default bases are refused."""
        cases = [
            ("draft_true", {"draft": True}, "draft"),
            ("draft_missing", {"draft": None}, "draft"),
            ("wrong_base", {"base": {"ref": "feature-branch"}}, "targets"),
            ("missing_base", {"base": {}}, "targets"),
        ]
        for label, patch, needle in cases:
            with self.subTest(case=label):
                self.setUp()
                for k, v in patch.items():
                    if v is None:
                        self.api.prs[7].pop(k, None)
                    else:
                        self.api.prs[7][k] = v
                self.publish()
                report = self.reconcile()
                self.assertEqual(report[0].outcome, "refused", report)
                self.assertIn(needle, report[0].reason)
                self.assertIsNone(self.tag(f"ready/{RECIPE}"))

    def test_identifiers_with_a_trailing_newline_are_refused_at_parse(self):
        """`$` matches before a final newline; every identifier must match whole, not prefix."""
        fields = dict(
            version=1,
            action="ready",
            recipe=RECIPE,
            sha=self.sha,
            branch=BRANCH,
            claim=CLAIM,
            session=SESSION,
            pr=7,
            review=99,
            created=NOW.isoformat(),
        )
        for key in ("recipe", "sha", "branch", "claim", "session"):
            with self.subTest(key=key):
                text = rr.render({**fields, key: fields[key] + "\n"})
                with self.assertRaisesRegex(rr.Refused, f"bad {key}"):
                    rr.parse(text)


if __name__ == "__main__":
    unittest.main()
