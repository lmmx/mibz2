"""Frozen three-symbol A3 scanner registration: plan, score and source checks."""

import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest

from scanner_a3_k3 import (
    CELLS,
    CORPORA,
    GUARDS,
    ROUNDS,
    InvalidRun,
    check_build_env,
    check_distinct_binaries,
    materialize_sources,
    plan,
    score,
    score_dir,
)


def samples(scan=0.94):
    rows = []
    for n, (kind, corpus, k), arm in plan():
        ratio = scan if (kind, corpus) == ("phases", "wikidata") else 1
        wall = (ratio if arm == "head" else 1) * (1 + n / 10000)
        rows.append(
            dict(
                kind=kind,
                corpus=corpus,
                k=k,
                round=n,
                build=arm,
                wall_s=wall,
                scan_s=wall,
                peak_kib=60000,
            )
        )
    return rows


def git(cwd, *args):
    return subprocess.check_output(["git", *args], cwd=cwd, text=True).strip()


class Sources(unittest.TestCase):
    def test_change_and_master_parent_from_shallow_runner(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            origin = root / "origin"
            git(root, "init", "-q", "-b", "master", origin)
            git(origin, "config", "user.email", "fixture@example.invalid")
            git(origin, "config", "user.name", "fixture")

            def commit(text):
                (origin / text).write_text(text)
                git(origin, "add", text)
                git(origin, "commit", "-qm", text)
                return git(origin, "rev-parse", "HEAD")

            commit("root")
            base = commit("master")
            git(origin, "switch", "-qc", "pr")
            head = commit("change")
            git(origin, "merge", "-q", "--no-edit", "--no-ff", "master")
            commit("recipe")
            runner = root / "runner"
            git(root, "clone", "-q", "--depth=1", "-b", "pr", origin.as_uri(), runner)
            materialize_sources(root / "good", runner, head, base)
            self.assertEqual(git(root / "good" / "head", "rev-parse", "HEAD"), head)
            self.assertEqual(git(root / "good" / "base", "rev-parse", "HEAD"), base)
            self.assertTrue((root / "good" / "head" / "change").exists())
            self.assertFalse((root / "good" / "base" / "change").exists())
            with self.assertRaisesRegex(InvalidRun, "only parent"):
                materialize_sources(root / "wrong", runner, head, head)
            # a merge is not a single change on its parent
            git(origin, "switch", "-q", "master")
            commit("later")
            git(origin, "switch", "-q", "pr")
            git(origin, "merge", "-q", "--no-edit", "master")
            merge = git(origin, "rev-parse", "HEAD")
            with self.assertRaisesRegex(InvalidRun, "only parent"):
                materialize_sources(root / "merge", runner, merge, head)


class Verdict(unittest.TestCase):
    def test_balanced_interleaved_grid(self):
        seq = list(plan())
        self.assertEqual(len(CELLS), 17)
        self.assertEqual(len(seq), 17 * ROUNDS * 2)
        for i in range(0, len(seq), 2):
            self.assertEqual(seq[i][:2], seq[i + 1][:2])
            self.assertNotEqual(seq[i][2], seq[i + 1][2])
        for cell in CELLS:
            self.assertEqual(sum(c == cell and a == "head" for _, c, a in seq[::2]), ROUNDS // 2)

    def test_pass_and_scan_bound(self):
        self.assertEqual(score(samples(0.94))["verdict"], "PASS")
        # A 3.5% cut misses the 4.0% bound.
        self.assertEqual(score(samples(0.965))["verdict"], "STOP")

    def test_overlapping_ranges_stop(self):
        rows = samples(0.94)
        for r in rows:
            if (r["kind"], r["corpus"]) == ("phases", "wikidata") and r["round"] % 2:
                r["scan_s"] *= 2
        self.assertEqual(score(rows)["verdict"], "STOP")

    def test_guard_rss_and_wall(self):
        for corpus in GUARDS:
            rows = samples()
            for r in rows:
                if (r["kind"], r["corpus"], r["build"]) == ("phases", corpus, "head"):
                    r["scan_s"] *= 1.021
            result = score(rows)
            self.assertEqual(result["verdict"], "STOP")
            self.assertTrue(any(corpus in m for m in result["findings"]))
        rows = samples()
        for r in rows:
            if (r["kind"], r["corpus"], r["build"]) == ("phases", "records", "head"):
                r["scan_s"] *= 1.019
        self.assertEqual(score(rows)["verdict"], "PASS")
        # A 3% point loss inside noise (a sub-millisecond corpus) does not stop.
        rows = samples()
        for r in rows:
            if (r["kind"], r["corpus"], r["build"]) == ("phases", "onebyte", "head"):
                r["scan_s"] *= 1.35 if r["round"] % 2 else 0.80
        result = score(rows)
        self.assertGreater(result["cells"]["phases/onebyte/1"]["ratio"], 1.02)
        self.assertEqual(result["verdict"], "PASS")
        # Calibrated RSS: 1T >1.005x + >+256 KiB beyond noise stops; <=+256 KiB passes.
        rows = samples()
        for r in rows:
            if (
                r["kind"] == "real"
                and r["build"] == "head"
                and (r["corpus"], r["k"])
                == (
                    "wikidata",
                    1,
                )
            ):
                r["peak_kib"] = 60200
        self.assertEqual(score(rows)["verdict"], "PASS")
        rows = samples()
        for r in rows:
            if (
                r["kind"] == "real"
                and r["build"] == "head"
                and (r["corpus"], r["k"])
                == (
                    "wikidata",
                    1,
                )
            ):
                r["peak_kib"] = 60400
        self.assertEqual(score(rows)["verdict"], "STOP")
        # Calibrated MT primary (>1.030x stops; 1.022x allocator baseline noise passes).
        rows = samples()
        for r in rows:
            if (
                r["kind"] == "real"
                and r["build"] == "head"
                and (r["corpus"], r["k"])
                == (
                    "wikidata",
                    8,
                )
            ):
                r["peak_kib"] = 61320
        self.assertEqual(score(rows)["verdict"], "PASS")
        rows = samples()
        for r in rows:
            if (
                r["kind"] == "real"
                and r["build"] == "head"
                and (r["corpus"], r["k"])
                == (
                    "wikidata",
                    8,
                )
            ):
                r["peak_kib"] = 62000
        self.assertEqual(score(rows)["verdict"], "STOP")
        # Calibrated MT guard (>1.050x stops; 1.034x passes).
        rows = samples()
        for r in rows:
            if r["kind"] == "real" and r["build"] == "head" and r["corpus"] == "runs":
                r["peak_kib"] = 62000
        self.assertEqual(score(rows)["verdict"], "PASS")
        rows = samples()
        for r in rows:
            if r["kind"] == "real" and r["build"] == "head" and r["corpus"] == "runs":
                r["peak_kib"] = 63100
        self.assertEqual(score(rows)["verdict"], "STOP")
        # Real Wikidata wall >1.020x beyond noise stops.
        rows = samples()
        for r in rows:
            if (
                r["kind"] == "real"
                and r["build"] == "head"
                and (r["corpus"], r["k"])
                == (
                    "wikidata",
                    9,
                )
            ):
                r["wall_s"] *= 1.03
        self.assertEqual(score(rows)["verdict"], "STOP")

    def test_build_env_distinct_binaries_and_offline_parity_gate(self):
        check_build_env({"PATH": "/usr/bin"})
        for key in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            with self.assertRaisesRegex(InvalidRun, key):
                check_build_env({key: "-C target-cpu=native"})
        with self.assertRaisesRegex(InvalidRun, "CARGO_PROFILE_RELEASE_LTO"):
            check_build_env({"CARGO_PROFILE_RELEASE_LTO": "off"})

        good_digests = {
            "base-real": "aaa",
            "head-real": "bbb",
            "base-phases": "ccc",
            "head-phases": "ddd",
        }
        check_distinct_binaries(good_digests)
        with self.assertRaisesRegex(InvalidRun, "identical"):
            check_distinct_binaries({**good_digests, "head-real": "aaa"})
        with self.assertRaisesRegex(InvalidRun, "identical"):
            check_distinct_binaries({**good_digests, "head-phases": "ccc"})

        with TemporaryDirectory() as directory:
            out = Path(directory)
            (out / "binaries.json").write_text(json.dumps(good_digests))
            (out / "runs.jsonl").write_text("\n".join(json.dumps(r) for r in samples()) + "\n")
            with self.assertRaisesRegex(InvalidRun, "missing parity.jsonl"):
                score_dir(out)
            expected_parity = (len(CORPORA) + 1) * 2 * 2
            (out / "parity.jsonl").write_text(
                "\n".join(json.dumps(["base", ["real", "wikidata", 1], {}]) for _ in range(2))
                + "\n"
            )
            with self.assertRaisesRegex(InvalidRun, "incomplete parity.jsonl"):
                score_dir(out)
            (out / "parity.jsonl").write_text(
                "\n".join(
                    json.dumps(["base", ["real", "wikidata", 1], {}])
                    for _ in range(expected_parity)
                )
                + "\n"
            )
            self.assertEqual(score_dir(out)["verdict"], "PASS")

    def test_missing_duplicate_and_undefined(self):
        rows = samples()
        for bad in (rows[:-1], rows + [rows[-1]]):
            with self.assertRaises(InvalidRun):
                score(bad)
        rows[0].update(wall_s=0, scan_s=0)
        with self.assertRaises(InvalidRun):
            score(rows)


if __name__ == "__main__":
    unittest.main()
