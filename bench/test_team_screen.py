"""The forced-team screen checker's negative controls (bench/team_screen.py, a DRAFT stub):
every admission failure the Researcher's probes found (#145 6759658271) must now fail,
and a complete, correct set must pass. Synthetic records and mocked profiles only: no
decoder, no profile, no timing.

    python3 bench/test_team_screen.py
"""

import copy
import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import team_screen as ts  # noqa: E402

MANIFEST = json.loads((HERE / "team-screen" / "manifest.json").read_text())
HEAD, BASE = "a" * 40, "b" * 40


def manifest():
    """The draft manifest with its FREEZE values filled by test stand-ins"""
    m = copy.deepcopy(MANIFEST)
    for i, f in enumerate(m["fixtures"]):
        if f["kind"] == "valid" and f["id"] != "W":
            f["expect"] = {"out": 1000 + i, "hash": f"{i:016x}"}
        if f["kind"] == "error":
            f["expect"].update(block=10 + i, out=500 + i, hash=f"{0xE0 + i:016x}")
    return m


def delta():
    return {
        "pr": 0,
        "head": HEAD,
        "base": BASE,
        "head_team_env": {"AB_STAGED_TRAVERSE": "1"},
        "walker": {"H-team": "coop::traverse_staged", "B-team": "coop::traverse(?!_staged)"},
        "forbid": {"H-team": ["coop::traverse(?!_staged)"], "B-team": ["coop::traverse_staged"]},
    }


def good_records(m, d):
    out = []
    for a in m["arms"]:
        env = d["head_team_env"] if a["env"] == "delta.head_team_env" else a["env"]
        for f in m["fixtures"]:
            e = f["expect"]
            if f["kind"] == "valid":
                rec = {
                    "out": e["out"],
                    "hash": e["hash"],
                    "err": "",
                    "err_kind": None,
                    "err_block": None,
                }
                status = 0
            else:
                text = (
                    "unexpected end of input" if e["kind"] == "Truncated" else "block CRC mismatch"
                )
                rec = {
                    "out": e["out"],
                    "hash": e["hash"],
                    "err": f"bz2 data error: {text} (block {e['block']})",
                    "err_kind": e["kind"],
                    "err_block": e["block"],
                }
                status = 1
            out.append(
                {
                    "arm": a["id"],
                    "fixture": f["id"],
                    "build": a["build"],
                    "build_sha": HEAD if a["build"] == "head" else BASE,
                    "exe_sha256": "c" * 64,
                    "env": env,
                    "exit_status": status,
                    "signal": None,
                    "record": rec,
                }
            )
    return out


class Outputs(unittest.TestCase):
    def setUp(self):
        self.m, self.d = manifest(), delta()
        self.good = good_records(self.m, self.d)

    def fails(self, records):
        self.assertTrue(ts.outputs_check(self.m, self.d, records))

    def test_complete_and_correct_is_admitted(self):
        self.assertEqual(ts.outputs_check(self.m, self.d, self.good), [])

    def test_no_records(self):
        self.fails([])

    def test_four_empty_objects(self):
        self.fails([{}, {}, {}, {}])

    def test_one_empty_record(self):
        r = copy.deepcopy(self.good)
        r[0]["record"] = {}
        self.fails(r)

    def test_missing_pair(self):
        self.fails(self.good[1:])

    def test_duplicate_pair(self):
        self.fails(self.good + [copy.deepcopy(self.good[0])])

    def test_matching_failures_on_a_valid_input(self):
        r = copy.deepcopy(self.good)
        for w in r:
            if w["fixture"] == "W":
                w["exit_status"], w["record"] = (
                    1,
                    {"out": 0, "hash": "0" * 16, "err": "bz2 data error: x"},
                )
        self.fails(r)

    def test_all_arms_agree_on_a_wrong_output(self):
        r = copy.deepcopy(self.good)
        for w in r:
            if w["fixture"] == "W":
                w["record"]["hash"] = "f" * 16
        self.fails(r)

    def test_error_without_its_block(self):
        r = copy.deepcopy(self.good)
        for w in r:
            if w["fixture"] == "E-crc-team":
                w["record"]["err_block"] = None
        self.fails(r)

    def test_error_at_the_wrong_block(self):
        r = copy.deepcopy(self.good)
        for w in r:
            if w["fixture"] == "E-trunc-solo":
                w["record"]["err_block"] += 1
        self.fails(r)

    def test_wrong_prefix(self):
        r = copy.deepcopy(self.good)
        r[-1]["record"]["out"] += 1
        self.fails(r)

    def test_wrong_build_or_selector(self):
        r = copy.deepcopy(self.good)
        r[0]["build_sha"] = "d" * 40
        self.fails(r)
        r = copy.deepcopy(self.good)
        for w in r:
            if w["arm"] == "H-team":
                w["env"] = {}
        self.fails(r)

    def test_error_kind_differs(self):
        r = copy.deepcopy(self.good)
        for w in r:
            if w["fixture"] == "E-crc-solo" and w["arm"] == "B-plain":
                w["record"]["err_kind"] = "Truncated"
        self.fails(r)

    def test_record_without_structured_error_fields(self):
        r = copy.deepcopy(self.good)
        del r[0]["record"]["err_block"]
        self.fails(r)

    def test_killed(self):
        r = copy.deepcopy(self.good)
        r[0]["signal"] = 9
        self.fails(r)


class Prof:
    def __init__(self, thread, calls):
        self.thread = thread
        self.calls = {("caller", callee): n for callee, n in calls.items()}


SYMS = [
    "mibz2::mtf::mtf_checkpoints_avx2",
    "mibz2::bwt::walk_segments_room",
    "<mibz2::coop::FusedBuild>::share",
    "<mibz2::coop::Relay>::walk_and_leave::<32>",
    "mibz2::coop::traverse::<32>",
    "mibz2::coop::traverse_staged::<32, 0, mibz2::coop::Counter>",
]
STAGED = "coop::traverse_staged"


def team(arm="H-team"):
    """A good one-team run on W: a leader (MTF, 198 solo walks, 2 joins), two helpers"""
    w = MANIFEST["fixtures"][0]["counted"]
    blocks, walked, takes = w["coop_blocks"], w["walked"], w["takes"]
    leader = Prof(
        3, {"mibz2::mtf::mtf_checkpoints_avx2": 431, "mibz2::bwt::walk_segments_room": 198}
    )
    h1 = Prof(
        7,
        {
            "<mibz2::coop::FusedBuild>::share": blocks,
            "<mibz2::coop::Relay>::walk_and_leave::<32>": blocks,
        },
    )
    h2 = Prof(
        8,
        {
            "<mibz2::coop::FusedBuild>::share": blocks,
            "<mibz2::coop::Relay>::walk_and_leave::<32>": blocks,
        },
    )
    key = STAGED if arm == "H-team" else "coop::traverse(?!_staged)"
    counts = {
        3: {key: dict(calls=2, takes=10, claims=12, walked=1000)},
        7: {
            key: dict(
                calls=blocks, takes=takes // 2, claims=takes // 2 + blocks, walked=walked // 2
            )
        },
        8: {
            key: dict(
                calls=blocks,
                takes=takes - takes // 2 - 10,
                claims=takes - takes // 2 - 10 + blocks,
                walked=walked - walked // 2 - 1000,
            )
        },
    }
    return [leader, h1, h2], counts


class Check(unittest.TestCase):
    def run_check(self, profs, counts, arm="H-team", syms=SYMS):
        return ts.check_profiles(
            manifest(), delta(), arm, profs, syms, lambda p: counts.get(p.thread, {})
        )

    def test_good_team_is_admitted(self):
        self.assertEqual(self.run_check(*team()), [])
        self.assertEqual(self.run_check(*team("B-team"), arm="B-team"), [])

    def test_omitted_helper(self):
        profs, counts = team()
        self.assertTrue(self.run_check(profs[:2], counts))

    def test_walker_on_threads_that_are_not_helpers(self):
        profs, counts = team()
        counts[9] = counts.pop(8)  # the walk on an unidentified thread, not helper 8
        self.assertTrue(self.run_check(profs, counts))

    def test_missing_attribution_is_not_zero(self):
        syms = [s for s in SYMS if "traverse::<32>" not in s]  # the forbidden walker inlined
        self.assertTrue(self.run_check(*team(), syms=syms))

    def test_forbidden_walker_executed(self):
        profs, counts = team()
        counts[7]["coop::traverse(?!_staged)"] = dict(calls=1, takes=0, claims=1, walked=0)
        self.assertTrue(self.run_check(profs, counts))

    def test_helper_without_a_claim(self):
        profs, counts = team()
        c7, c8 = counts[7][STAGED], counts[8][STAGED]
        c8["takes"] += c7["takes"]
        c8["claims"] += c7["takes"]
        c7["claims"] -= c7["takes"]
        c7["takes"] = 0
        self.assertTrue(self.run_check(profs, counts))

    def test_conservation(self):
        profs, counts = team()
        counts[7][STAGED]["walked"] -= 1
        self.assertTrue(self.run_check(profs, counts))

    def test_leader_never_joined_or_solo_missing(self):
        profs, counts = team()
        counts.pop(3)
        self.assertTrue(self.run_check(profs, counts))
        profs, counts = team()
        profs[0].calls = {
            k: v for k, v in profs[0].calls.items() if "walk_segments_room" not in k[1]
        }
        self.assertTrue(self.run_check(profs, counts))

    def test_duplicate_thread(self):
        profs, counts = team()
        self.assertTrue(self.run_check(profs + [profs[1]], counts))


if __name__ == "__main__":
    unittest.main()
