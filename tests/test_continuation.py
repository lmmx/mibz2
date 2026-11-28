# ruff: noqa: D101, D102, D103
"""Frozen fixtures for decentralised dependency continuations (claim continuation-runbook).

Frozen before the implementation in `frontier/tools/continuation.py` existed. Each class maps
to one numbered clause of the claim's stop_test. Everything here is hermetic: an in-memory
compare-and-swap store, fake clocks and scripted probes, plus one temporary bare Git remote
for the real `refs/heads/wait/*` store. Nothing touches the real origin.
"""

from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import threading
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = REPO_ROOT / "frontier" / "tools" / "continuation.py"

_spec = importlib.util.spec_from_file_location("continuation", SRC)
cont = importlib.util.module_from_spec(_spec)
sys.modules["continuation"] = cont
_spec.loader.exec_module(cont)

T0 = 1_800_000_000  # a fixed epoch second; every clock below starts here
SHA_A = "a" * 40
SHA_B = "b" * 40
SHA_C = "c" * 40


class FakeClock:
    """Virtual time: `sleep` advances instantly; each watcher may own its own clock."""

    def __init__(self, t=T0):
        self.t = t
        self.sleeps = []

    def now(self):
        return self.t

    def sleep(self, s):
        self.sleeps.append(s)
        self.t += s


class Script:
    """A probe whose answer depends on its clock: [(from_t, Observation)], last match wins."""

    def __init__(self, clock, steps):
        self.clock = clock
        self.steps = steps
        self.calls = 0

    def __call__(self, dep):
        self.calls += 1
        out = cont.Observation("waiting")
        for t, obs in self.steps:
            if self.clock.now() >= t:
                out = obs
        return out


def obs(state, at=None, detail="", value=None):
    return cont.Observation(state, authoritative_at=at, detail=detail, value=value)


class CountingStore(cont.MemoryStore):
    """MemoryStore that counts writes, to bound per-poll churn."""

    def __init__(self):
        super().__init__()
        self.writes = 0

    def cas(self, key, expected, record):
        ok, tok = super().cas(key, expected, record)
        if ok:
            self.writes += 1
        return ok, tok


# (1) two or more independent chains wait and resume concurrently, no global watcher
class TestIndependentChains(unittest.TestCase):
    def test_two_chains_wait_and_resume_concurrently(self):
        store = cont.MemoryStore()
        cont.declare(
            store,
            "release-0-5-18",
            "await-preflight-merge",
            f"merged:1256@{SHA_A}",
            next_text="queue preflight",
            critical=True,
            priority=1,
            by="a",
            clock=FakeClock(),
        )
        cont.declare(
            store,
            "bench-x",
            "await-ci",
            f"pr:1400@{SHA_B}",
            next_text="merge",
            priority=3,
            by="b",
            clock=FakeClock(),
        )
        results = {}

        def run(key, session, ready_at):
            clock = FakeClock()
            probe = Script(clock, [(T0 + ready_at, obs("ready", at=T0 + ready_at))])
            results[key] = cont.watch(
                store, key, session=session, holder=session, probe=probe, clock=clock, timeout=3600
            )

        th = [
            threading.Thread(target=run, args=("release-0-5-18--await-preflight-merge", "s1", 120)),
            threading.Thread(target=run, args=("bench-x--await-ci", "s2", 600)),
        ]
        for t in th:
            t.start()
        for t in th:
            t.join()
        self.assertEqual(results["release-0-5-18--await-preflight-merge"].outcome, "ready")
        self.assertEqual(results["bench-x--await-ci"].outcome, "ready")
        for key in results:
            self.assertEqual(store.get(key)[1]["state"], "ready")

    def test_no_global_lease_or_singleton(self):
        text = SRC.read_text()
        code = "\n".join(l for l in text.splitlines() if not l.lstrip().startswith(("#", '"')))
        for word in ("global_lease", "watcher_lease", "elect_", "leader_", "dispatch("):
            self.assertNotIn(word, code)
        # every lock is per continuation key: the store has no key that is not a continuation
        store = cont.MemoryStore()
        cont.declare(store, "c1", "a1", "manual:x", by="a", clock=FakeClock())
        self.assertEqual(sorted(store.list()), ["c1--a1"])


# (2) competing peers on one continuation: one active waiter, one executor, expiry recovery
class TestDeduplication(unittest.TestCase):
    def setUp(self):
        self.store = cont.MemoryStore()
        self.clock = FakeClock()
        cont.declare(self.store, "c", "gate", f"pr:9@{SHA_A}", by="a", clock=self.clock)
        self.key = "c--gate"

    def test_second_waiter_does_not_wait(self):
        ok, _ = cont.acquire_waiter(self.store, self.key, "s1", "h1", self.clock)
        self.assertTrue(ok)
        probe = Script(self.clock, [])
        r = cont.watch(
            self.store,
            self.key,
            session="s2",
            holder="h2",
            probe=probe,
            clock=self.clock,
            timeout=600,
        )
        self.assertEqual(r.outcome, "duplicate")
        self.assertEqual(probe.calls, 0)

    def test_expired_waiter_is_recoverable(self):
        cont.acquire_waiter(self.store, self.key, "s1", "h1", self.clock, ttl=60)
        self.clock.t += 61
        probe = Script(self.clock, [(T0, obs("ready", at=T0))])
        r = cont.watch(
            self.store,
            self.key,
            session="s2",
            holder="h2",
            probe=probe,
            clock=self.clock,
            timeout=600,
        )
        self.assertEqual(r.outcome, "ready")

    def test_only_one_peer_acquires_execution(self):
        probe = Script(self.clock, [(T0, obs("ready", at=T0))])
        cont.watch(
            self.store,
            self.key,
            session="s1",
            holder="h1",
            probe=probe,
            clock=self.clock,
            timeout=60,
        )
        wins = []

        def go(s):
            wins.append(
                cont.resume(self.store, self.key, session=s, by=s, probe=probe, clock=self.clock).ok
            )

        th = [threading.Thread(target=go, args=(f"p{i}",)) for i in range(8)]
        for t in th:
            t.start()
        for t in th:
            t.join()
        self.assertEqual(sum(wins), 1)

    def test_abandoned_acquisition_expires_to_ready(self):
        probe = Script(self.clock, [(T0, obs("ready", at=T0))])
        cont.watch(
            self.store,
            self.key,
            session="s1",
            holder="h1",
            probe=probe,
            clock=self.clock,
            timeout=60,
        )
        self.assertTrue(
            cont.resume(
                self.store, self.key, session="p1", by="p1", probe=probe, clock=self.clock, ttl=300
            ).ok
        )
        self.assertFalse(
            cont.resume(
                self.store, self.key, session="p2", by="p2", probe=probe, clock=self.clock
            ).ok
        )
        self.clock.t += 301
        self.assertTrue(
            cont.resume(
                self.store, self.key, session="p2", by="p2", probe=probe, clock=self.clock
            ).ok
        )

    def test_duplicate_declare_refused(self):
        with self.assertRaises(cont.ContinuationError):
            cont.declare(self.store, "c", "gate", f"pr:9@{SHA_A}", by="b", clock=self.clock)


# (3) exact-pin readiness wakes; stale/VOID/unread never PASS; changed head and independence
class TestPinsAndJudgement(unittest.TestCase):
    def setUp(self):
        self.store = cont.MemoryStore()
        self.clock = FakeClock()

    def test_evidence_ready_is_not_pass(self):
        cont.declare(
            self.store,
            "r",
            "readback",
            f"evidence:bench-release-preflight@{SHA_A}",
            by="a",
            clock=self.clock,
        )
        probe = Script(self.clock, [(T0, obs("ready", at=T0, detail="returned-unread"))])
        r = cont.watch(
            self.store,
            "r--readback",
            session="s",
            holder="h",
            probe=probe,
            clock=self.clock,
            timeout=60,
        )
        self.assertEqual(r.outcome, "ready")
        rec = self.store.get("r--readback")[1]
        self.assertNotIn("disposition", rec)
        self.assertNotEqual(rec["state"], "done")
        with self.assertRaises(cont.ContinuationError):
            cont.done(self.store, "r--readback", session="s", disposition="", clock=self.clock)

    def test_pin_parse_requires_exact_sha(self):
        with self.assertRaises(cont.ContinuationError):
            cont.parse_dep("pr:12@abcdef")
        with self.assertRaises(cont.ContinuationError):
            cont.parse_dep("evidence:bench-x@" + "a" * 39)
        self.assertEqual(cont.parse_dep(f"merged:12@{SHA_A}"), ("merged", {"pr": 12, "sha": SHA_A}))

    def test_evidence_probe_ignores_other_pins(self):
        bundles = [{"pin": SHA_B, "rel": "x/old.tar.gz", "commit": "e" * 40, "at": T0}]
        probe = cont.evidence_probe_from(lambda recipe: bundles)
        self.assertEqual(probe(("evidence", {"recipe": "bench-x", "sha": SHA_A})).state, "waiting")
        bundles.append({"pin": SHA_A, "rel": "x/new.tar.gz", "commit": "f" * 40, "at": T0 + 5})
        o = probe(("evidence", {"recipe": "bench-x", "sha": SHA_A}))
        self.assertEqual((o.state, o.authoritative_at), ("ready", T0 + 5))
        self.assertIn("unread", o.detail)

    def test_evidence_probe_unavailable_is_unknown(self):
        def boom(recipe):
            raise OSError("no evidence branch")

        probe = cont.evidence_probe_from(boom)
        self.assertEqual(probe(("evidence", {"recipe": "bench-x", "sha": SHA_A})).state, "unknown")

    def test_changed_head_supersedes(self):
        pr = {"state": "open", "merged": False, "head": {"sha": SHA_B}}
        probe = cont.pr_probe_from(lambda n: pr, lambda sha: ("success", T0))
        self.assertEqual(probe(("pr", {"pr": 5, "sha": SHA_A})).state, "superseded")
        cont.declare(self.store, "c", "ci", f"pr:5@{SHA_A}", by="a", clock=self.clock)
        r = cont.watch(
            self.store, "c--ci", session="s", holder="h", probe=probe, clock=self.clock, timeout=60
        )
        self.assertEqual(r.outcome, "superseded")
        self.assertFalse(
            cont.resume(self.store, "c--ci", session="p", by="p", probe=probe, clock=self.clock).ok
        )

    def test_pr_green_and_red(self):
        pr = {"state": "open", "merged": False, "head": {"sha": SHA_A}}
        checks = {"v": ("pending", None)}
        probe = cont.pr_probe_from(lambda n: pr, lambda sha: checks["v"])
        dep = ("pr", {"pr": 5, "sha": SHA_A})
        self.assertEqual(probe(dep).state, "waiting")
        checks["v"] = ("failure", T0 + 9)
        self.assertEqual(probe(dep).state, "failed")
        checks["v"] = ("success", T0 + 10)
        o = probe(dep)
        self.assertEqual((o.state, o.authoritative_at), ("ready", T0 + 10))

    def test_merged_probe(self):
        pr = {
            "state": "closed",
            "merged": True,
            "head": {"sha": SHA_A},
            "merged_at": "2026-11-28T10:00:00Z",
        }
        probe = cont.pr_probe_from(lambda n: pr, lambda sha: ("success", T0))
        o = probe(("merged", {"pr": 5, "sha": SHA_A}))
        self.assertEqual(o.state, "ready")
        self.assertIsNotNone(o.authoritative_at)
        pr2 = {"state": "closed", "merged": False, "head": {"sha": SHA_A}}
        probe2 = cont.pr_probe_from(lambda n: pr2, lambda sha: ("success", T0))
        self.assertEqual(probe2(("merged", {"pr": 5, "sha": SHA_A})).state, "superseded")

    def test_independent_review_respected(self):
        cont.declare(
            self.store,
            "c",
            "review",
            f"pr:5@{SHA_A}",
            by="author-x",
            independent=True,
            clock=self.clock,
        )
        probe = Script(self.clock, [(T0, obs("ready", at=T0))])
        cont.watch(
            self.store,
            "c--review",
            session="s",
            holder="h",
            probe=probe,
            clock=self.clock,
            timeout=60,
        )
        self.assertFalse(
            cont.resume(
                self.store, "c--review", session="s1", by="author-x", probe=probe, clock=self.clock
            ).ok
        )
        self.assertTrue(
            cont.resume(
                self.store,
                "c--review",
                session="s2",
                by="reviewer-y",
                probe=probe,
                clock=self.clock,
            ).ok
        )

    def test_resume_revalidates_pin(self):
        cont.declare(self.store, "c", "ci", f"pr:5@{SHA_A}", by="a", clock=self.clock)
        p1 = Script(self.clock, [(T0, obs("ready", at=T0))])
        cont.watch(
            self.store, "c--ci", session="s", holder="h", probe=p1, clock=self.clock, timeout=60
        )
        p2 = Script(self.clock, [(T0, obs("superseded", detail="head moved"))])
        r = cont.resume(self.store, "c--ci", session="p", by="p", probe=p2, clock=self.clock)
        self.assertFalse(r.ok)
        self.assertEqual(self.store.get("c--ci")[1]["state"], "superseded")


# (4) stable read-only ordering; conflicts and unknown state do not suppress other work
class TestView(unittest.TestCase):
    def test_order_and_classes(self):
        store = cont.MemoryStore()
        c = FakeClock()
        cont.declare(store, "p3-old", "a", "manual:x", priority=3, by="x", clock=c)
        c.t += 10
        cont.declare(store, "p1-crit", "a", "manual:x", priority=1, critical=True, by="x", clock=c)
        c.t += 10
        cont.declare(store, "p1-plain", "a", "manual:x", priority=1, by="x", clock=c)
        cont.declare(store, "p2-i9", "a", "manual:x", priority=2, by="x", clock=c)
        cont.declare(store, "p2-held", "a", "manual:x", priority=2, by="x", clock=c)
        cont.declare(store, "p2-owner", "a", "manual:x", priority=2, by="x", clock=c)
        cont.declare(store, "p2-unk", "a", "manual:x", priority=2, by="x", clock=c)
        for k in list(store.list()):
            tok, rec = store.get(k)
            if k != "p2-unk--a":
                rec["state"] = "ready"
                rec["ready_observed"] = rec["declared_at"]
            store.cas(k, tok, rec)
        claims = {"p2-i9": {"needs": ["shell", "i9"]}, "p2-owner": {"needs_louis": "decide"}}
        holds = {"p2-held": {"holder": "other", "live": True}}
        rows = cont.view(
            store,
            clock=c,
            claims=claims,
            holds=holds,
            can={"shell", "repo-write"},
            probe=lambda dep: obs("unknown", detail="remote unavailable"),
        )
        order = [r["key"] for r in rows]
        cls = {r["key"]: r["class"] for r in rows}
        self.assertEqual(cls["p2-i9--a"], "capability-blocked")
        self.assertEqual(cls["p2-owner--a"], "owner-blocked")
        self.assertEqual(cls["p2-held--a"], "held")
        self.assertEqual(cls["p2-unk--a"], "unknown")
        self.assertEqual(cls["p1-crit--a"], "ready")
        ready = [r["key"] for r in rows if r["class"] == "ready"]
        self.assertEqual(ready, ["p1-crit--a", "p1-plain--a", "p3-old--a"])
        self.assertEqual(order[0], "p1-crit--a")
        # deterministic
        rows2 = cont.view(
            store,
            clock=c,
            claims=claims,
            holds=holds,
            can={"shell", "repo-write"},
            probe=lambda dep: obs("unknown"),
        )
        self.assertEqual([r["key"] for r in rows2], order)

    def test_view_is_read_only(self):
        store = CountingStore()
        c = FakeClock()
        cont.declare(store, "a", "b", "manual:x", by="x", clock=c)
        w = store.writes
        cont.view(store, clock=c, probe=lambda dep: obs("ready", at=T0))
        self.assertEqual(store.writes, w)

    def test_tie_break_is_stable_key(self):
        store = cont.MemoryStore()
        c = FakeClock()
        for cid in ("zeta", "alpha", "mid"):
            cont.declare(store, cid, "a", "manual:x", priority=2, by="x", clock=c)
            tok, rec = store.get(f"{cid}--a")
            rec["state"] = "ready"
            rec["ready_observed"] = T0
            store.cas(f"{cid}--a", tok, rec)
        rows = cont.view(store, clock=c)
        self.assertEqual([r["key"] for r in rows], ["alpha--a", "mid--a", "zeta--a"])


# (5) bounded task-local wait: fake clock, backoff, timeout handback, no per-poll churn
class TestBoundedWait(unittest.TestCase):
    def test_backoff_bounds_and_no_churn(self):
        store = CountingStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", f"pr:1@{SHA_A}", by="x", clock=clock)
        w0 = store.writes
        probe = Script(clock, [(T0 + 3 * 3600, obs("ready", at=T0 + 3 * 3600))])
        r = cont.watch(
            store,
            "c--a",
            session="s",
            holder="h",
            probe=probe,
            clock=clock,
            timeout=4 * 3600,
            interval=45,
            max_interval=300,
            waiter_ttl=1800,
        )
        self.assertEqual(r.outcome, "ready")
        self.assertGreaterEqual(min(clock.sleeps), 30)
        self.assertLessEqual(clock.sleeps[0], 60)
        self.assertLessEqual(max(clock.sleeps), 300)
        # writes: acquire + one renewal per ttl/2 at most + the transition; never one per poll
        self.assertLess(store.writes - w0, probe.calls)
        self.assertLessEqual(store.writes - w0, 2 + (3 * 3600) // 900 + 1)

    def test_timeout_hands_back_with_predicate(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", f"pr:1@{SHA_A}", next_text="merge it", by="x", clock=clock)
        r = cont.watch(
            store,
            "c--a",
            session="s",
            holder="h",
            probe=Script(clock, []),
            clock=clock,
            timeout=600,
        )
        self.assertEqual(r.outcome, "timeout")
        rec = store.get("c--a")[1]
        self.assertEqual(rec["state"], "waiting")
        self.assertEqual(rec["dep"], f"pr:1@{SHA_A}")
        self.assertEqual(rec["next"], "merge it")
        self.assertFalse(cont.waiter_live(rec, clock.now()))

    def test_unknown_keeps_waiting_and_reports(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", f"pr:1@{SHA_A}", by="x", clock=clock)
        r = cont.watch(
            store,
            "c--a",
            session="s",
            holder="h",
            probe=Script(clock, [(T0, obs("unknown", detail="api down"))]),
            clock=clock,
            timeout=300,
        )
        self.assertEqual(r.outcome, "timeout")
        self.assertIn("unknown", r.detail)

    def test_watch_returns_once_per_change(self):
        """The model sees one return per meaningful change, not one per poll."""
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", f"pr:1@{SHA_A}", by="x", clock=clock)
        probe = Script(clock, [(T0 + 900, obs("ready", at=T0 + 900))])
        r = cont.watch(
            store, "c--a", session="s", holder="h", probe=probe, clock=clock, timeout=7200
        )
        self.assertEqual(r.outcome, "ready")
        self.assertGreater(probe.calls, 3)
        self.assertEqual(r.polls, probe.calls)

    def test_no_path_or_queue_locks(self):
        text = SRC.read_text()
        self.assertNotIn("queue.lock", text)
        self.assertNotIn("refs/heads/hold/", text.replace("reads refs/heads/hold/", ""))
        self.assertNotIn("frontier-admission", text)


# (6) one chain advances several sequential gates while another progresses independently
class TestSequentialGates(unittest.TestCase):
    def test_release_chain_replay_alongside_ci_chain(self):
        """Replay of the v0.5.18 dependency chain beside an unrelated CI chain."""
        store = cont.MemoryStore()
        gates = [
            ("review-preflight", f"pr:1256@{SHA_A}"),
            ("merge-preflight", f"merged:1256@{SHA_A}"),
            ("queue-recalibration", f"ref:refs/tags/ready/recalibrate-0518@{SHA_B}"),
            ("read-recalibration", f"evidence:recalibrate-0518@{SHA_B}"),
            ("publish", f"ref:refs/tags/v0.5.18@{SHA_C}"),
        ]
        log = []
        lock = threading.Lock()

        def release_chain():
            clock = FakeClock()
            cont.declare(
                store,
                "release-0-5-18",
                gates[0][0],
                gates[0][1],
                critical=True,
                priority=1,
                by="rel",
                clock=clock,
            )
            for i, (action, dep) in enumerate(gates):
                key = f"release-0-5-18--{action}"
                probe = Script(clock, [(clock.now() + 200, obs("ready", at=clock.now() + 200))])
                r = cont.watch(
                    store, key, session="rel", holder="rel", probe=probe, clock=clock, timeout=3600
                )
                assert r.outcome == "ready", r
                assert cont.resume(store, key, session="rel", by="rel", probe=probe, clock=clock).ok
                nxt = gates[i + 1] if i + 1 < len(gates) else None
                cont.done(
                    store,
                    key,
                    session="rel",
                    disposition=f"{action} handled",
                    clock=clock,
                    next_action=nxt[0] if nxt else None,
                    next_dep=nxt[1] if nxt else None,
                    critical=True,
                    priority=1,
                )
                with lock:
                    log.append(("rel", action))

        def ci_chain():
            clock = FakeClock()
            cont.declare(
                store, "bench-y", "ci", f"pr:1401@{SHA_C}", priority=3, by="ci", clock=clock
            )
            probe = Script(clock, [(T0 + 50, obs("ready", at=T0 + 50))])
            r = cont.watch(
                store,
                "bench-y--ci",
                session="ci",
                holder="ci",
                probe=probe,
                clock=clock,
                timeout=3600,
            )
            assert r.outcome == "ready"
            assert cont.resume(
                store, "bench-y--ci", session="ci", by="ci", probe=probe, clock=clock
            ).ok
            cont.done(store, "bench-y--ci", session="ci", disposition="merged", clock=clock)
            with lock:
                log.append(("ci", "ci"))

        th = [threading.Thread(target=release_chain), threading.Thread(target=ci_chain)]
        for t in th:
            t.start()
        for t in th:
            t.join(timeout=30)
        self.assertIn(("ci", "ci"), log)
        self.assertEqual([a for who, a in log if who == "rel"], [g[0] for g in gates])
        recs = store.list()
        for action, _ in gates:
            self.assertEqual(recs[f"release-0-5-18--{action}"][1]["state"], "done")
        first = recs["release-0-5-18--review-preflight"][1]
        self.assertEqual(first["next_key"], "release-0-5-18--merge-preflight")
        self.assertIn("next_started_at", first)

    def test_stalled_chain_does_not_block_other(self):
        store = cont.MemoryStore()
        c1, c2 = FakeClock(), FakeClock()
        cont.declare(store, "stuck", "a", "manual:never", by="x", clock=c1)
        cont.declare(store, "moving", "a", f"pr:2@{SHA_A}", by="y", clock=c2)
        out = {}
        t1 = threading.Thread(
            target=lambda: out.setdefault(
                "stuck",
                cont.watch(
                    store,
                    "stuck--a",
                    session="s1",
                    holder="h1",
                    probe=Script(c1, []),
                    clock=c1,
                    timeout=7200,
                ),
            )
        )
        t2 = threading.Thread(
            target=lambda: out.setdefault(
                "moving",
                cont.watch(
                    store,
                    "moving--a",
                    session="s2",
                    holder="h2",
                    probe=Script(c2, [(T0 + 60, obs("ready", at=T0 + 60))]),
                    clock=c2,
                    timeout=7200,
                ),
            )
        )
        t1.start()
        t2.start()
        t1.join()
        t2.join()
        self.assertEqual(out["moving"].outcome, "ready")
        self.assertEqual(out["stuck"].outcome, "timeout")


# (7) timing: authoritative vs observed ready, acquire, completion, next start; restart-safe
class TestTimings(unittest.TestCase):
    def test_authoritative_and_observed(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", f"pr:1@{SHA_A}", by="x", clock=clock)
        probe = Script(clock, [(T0 + 100, obs("ready", at=T0 + 70))])
        cont.watch(store, "c--a", session="s", holder="h", probe=probe, clock=clock, timeout=3600)
        clock.t += 40
        cont.resume(store, "c--a", session="p", by="p", probe=probe, clock=clock)
        clock.t += 500
        cont.done(store, "c--a", session="p", disposition="merged", clock=clock)
        t = cont.timings(store.get("c--a")[1])
        self.assertEqual(t["ready_source"], "authoritative")
        self.assertEqual(t["ready_at"], T0 + 70)
        self.assertGreaterEqual(t["ready_observed"], T0 + 100)
        self.assertEqual(t["ready_to_acquire"], t["acquired_at"] - (T0 + 70))
        self.assertEqual(t["acquire_to_disposition"], 500)
        self.assertIsNone(t["disposition_to_next_start"])

    def test_unknown_is_not_invented(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", "manual:x", by="x", clock=clock)
        t = cont.timings(store.get("c--a")[1])
        self.assertEqual(t["ready_source"], "unknown")
        self.assertIsNone(t["ready_at"])
        self.assertIsNone(t["ready_to_acquire"])
        probe = Script(clock, [(T0, obs("ready"))])  # no authoritative time
        cont.watch(store, "c--a", session="s", holder="h", probe=probe, clock=clock, timeout=60)
        t = cont.timings(store.get("c--a")[1])
        self.assertEqual(t["ready_source"], "observed")

    def test_first_observation_is_kept(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(store, "c", "a", "manual:x", by="x", clock=clock)
        probe = Script(clock, [(T0 + 60, obs("ready"))])
        cont.watch(store, "c--a", session="s", holder="h", probe=probe, clock=clock, timeout=600)
        first = store.get("c--a")[1]["ready_observed"]
        clock.t += 1000
        cont.watch(store, "c--a", session="s2", holder="h2", probe=probe, clock=clock, timeout=60)
        self.assertEqual(store.get("c--a")[1]["ready_observed"], first)


def _git(cwd, *args):
    env = {
        "GIT_AUTHOR_NAME": "T",
        "GIT_AUTHOR_EMAIL": "t@example.invalid",
        "GIT_COMMITTER_NAME": "T",
        "GIT_COMMITTER_EMAIL": "t@example.invalid",
        "PATH": "/usr/bin:/bin:/usr/local/bin",
        "HOME": str(cwd),
    }
    return subprocess.run(
        ["git", "-C", str(cwd), *args], capture_output=True, text=True, env=env, check=True
    ).stdout.strip()


class TestGitRefStore(unittest.TestCase):
    """Real CAS on a temporary bare remote; restart reads the same state; only wait/ refs move."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        self.remote = self.tmp / "remote.git"
        _git(self.tmp, "init", "-q", "--bare", "-b", "master", str(self.remote))
        self.clones = []
        for name in ("a", "b"):
            d = self.tmp / name
            _git(self.tmp, "init", "-q", "-b", "master", str(d))
            _git(d, "remote", "add", "origin", str(self.remote))
            _git(d, "config", "user.name", "T")
            _git(d, "config", "user.email", "t@example.invalid")
            self.clones.append(d)

    def test_cas_restart_and_ref_scope(self):
        a = cont.GitRefStore(self.clones[0])
        clock = FakeClock()
        cont.declare(a, "c", "gate", f"pr:3@{SHA_A}", by="x", clock=clock)
        probe = Script(clock, [(T0 + 60, obs("ready", at=T0 + 30))])
        r = cont.watch(a, "c--gate", session="s", holder="h", probe=probe, clock=clock, timeout=600)
        self.assertEqual(r.outcome, "ready")
        b = cont.GitRefStore(self.clones[1])  # a fresh peer: "restart"
        tok, rec = b.get("c--gate")
        self.assertEqual(rec["state"], "ready")
        self.assertEqual(rec["ready_authoritative"], T0 + 30)
        # stale token loses
        ok, _ = b.cas("c--gate", "0" * 40, rec)
        self.assertFalse(ok)
        ok, _ = b.cas("c--gate", tok, rec)
        self.assertTrue(ok)
        ok, _ = a.cas("c--gate", tok, rec)
        self.assertFalse(ok)
        refs = _git(self.tmp, "--git-dir", str(self.remote), "for-each-ref", "--format=%(refname)")
        self.assertTrue(refs)
        for ref in refs.splitlines():
            self.assertTrue(ref.startswith("refs/heads/wait/"), ref)


# (8) no spend, no dispatch, no merge/publish, no presence requirement
class TestAuthorityBounds(unittest.TestCase):
    def test_source_has_no_consequential_calls(self):
        text = SRC.read_text()
        for needle in (
            "merge_pull_request",
            "/merge",
            "workflow_dispatch",
            "dispatches",
            "create_session",
            "presence_",
            "git push origin master",
            "refs/heads/master:",
        ):
            self.assertNotIn(needle, text)

    def test_resume_prints_ordinary_reservation(self):
        store = cont.MemoryStore()
        clock = FakeClock()
        cont.declare(
            store, "c", "a", f"pr:1@{SHA_A}", next_text="merge under gates", by="x", clock=clock
        )
        probe = Script(clock, [(T0, obs("ready", at=T0))])
        cont.watch(store, "c--a", session="s", holder="h", probe=probe, clock=clock, timeout=60)
        r = cont.resume(store, "c--a", session="p", by="p", probe=probe, clock=clock)
        self.assertTrue(r.ok)
        self.assertIn("frontier.py", r.detail)
        self.assertIn("merge under gates", r.detail)


class TestViewAfterFreeze(unittest.TestCase):
    """Added after the freeze: a held row still reports its pin's probed state."""

    def test_held_row_reports_pin(self):
        store = cont.MemoryStore()
        c = FakeClock()
        cont.declare(store, "h", "a", f"pr:1@{SHA_A}", by="x", clock=c)
        rows = cont.view(
            store,
            clock=c,
            holds={"h": {"holder": "peer", "live": True}},
            probe=lambda dep: obs("waiting", detail="checks pending"),
        )
        self.assertEqual(rows[0]["class"], "held")
        self.assertIn("pin waiting: checks pending", rows[0]["detail"])

    def test_rerun_supersedes_earlier_failed_attempt(self):
        """Live finding on #1356: a re-run's old failed attempt stays listed at the head."""
        runs = [
            {
                "name": "x",
                "id": 2,
                "status": "completed",
                "conclusion": "success",
                "started_at": "2026-11-28T12:02:34Z",
                "completed_at": "2026-11-28T12:02:50Z",
            },
            {
                "name": "x",
                "id": 1,
                "status": "completed",
                "conclusion": "failure",
                "started_at": "2026-11-28T11:41:16Z",
                "completed_at": "2026-11-28T11:41:30Z",
            },
        ]
        st = {"state": "success", "statuses": [{"state": "success"}]}
        self.assertEqual(cont.summarise_checks(runs, st)[0], "success")
        self.assertEqual(cont.summarise_checks(runs[::-1][:1], st)[0], "failure")


if __name__ == "__main__":
    unittest.main()
