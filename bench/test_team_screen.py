"""The forced-team screen checker's negative controls (bench/team_screen.py, a DRAFT stub):
every admission failure the Researcher's probes found (#145 6759658271, and the recheck's
four, 6760210997) must fail, and a complete, correct set must pass. Synthetic records and mocked profiles only: no
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
    """The draft manifest, its frozen values as made (the text corpus per-head)"""
    return copy.deepcopy(MANIFEST)


def artifacts():
    return {
        "head": {"sha": HEAD, "exe_sha256": "c" * 64},
        "base": {"sha": BASE, "exe_sha256": "d" * 64},
        "lbzip2": {"exe_sha256": "e" * 64},
        "per_head_inputs": {
            "C-text": {
                "sha256": "f" * 64,
                "expect": {"out": 12684479, "hash": "bb94ebc48944d6c4", "sha256": "7" * 64},
            }
        },
    }


def delta():
    return {
        "pr": 0,
        "head": HEAD,
        "base": BASE,
        "head_team_env": {"AB_STAGED_TRAVERSE": "1"},
        "walker": {"H-team": "coop::traverse_staged", "B-team": "coop::traverse(?!_staged)"},
        "forbid": {"H-team": ["coop::traverse(?!_staged)"], "B-team": ["coop::traverse_staged"]},
    }


def record(out, err="", kind=None, block=None):
    return dict(
        schema="ab-child/2",
        out=out["out"],
        hash=out["hash"],
        sha256=out["sha256"],
        err=err,
        err_kind=kind,
        err_block=block,
    )


def good_records(m, d, art=None):
    """A complete, correct set of completion records for the manifest's runs"""
    art = art or artifacts()
    fx = {f["id"]: f for f in m["fixtures"]}
    arms = {a["id"]: a for a in m["arms"]}
    out = []
    for run, (kind, arm_id, fx_id, buf) in ts.expected_runs(m).items():
        f = fx[fx_id]
        in_sha, exp = ts._frozen(f, art)
        w = dict(kind=kind, run=run, fixture=fx_id, exit_status=0, signal=None, timed_out=False)
        if kind == "reject":
            w.update(
                command=["lbzip2", "-dc", f["file"]],
                input_sha256=in_sha,
                env={},
                exe_sha256=art["lbzip2"]["exe_sha256"],
                exit_status=1,
                record=None,
            )
        elif kind == "oracle":
            if f["kind"] == "error":
                in_sha = f["prefix_oracle"]["sha256"]
            w.update(
                command=["bin/ab", "child", "lbzip2", "1", ts._file(f, kind)],
                input_sha256=in_sha,
                env=dict(m["common_env"]),
                exe_sha256=art["head"]["exe_sha256"],
                record=record(exp),
            )
        else:
            a = arms[arm_id]
            env = dict(m["common_env"], AB_READ_BUF=str(buf))
            env.update(d["head_team_env"] if a["env"] == "delta.head_team_env" else a["env"])
            if f["kind"] == "valid":
                rec = record(exp)
            else:
                rec = record(
                    exp, f"bz2 data error (block {exp['block']})", exp["kind"], exp["block"]
                )
            w.update(
                arm=arm_id,
                build=a["build"],
                build_sha=d[a["build"]],
                read_buffer=buf,
                command=[f"{a['build']}/ab", "child", a["arm"], str(a["threads"]), f["file"]],
                input_sha256=in_sha,
                env=env,
                exe_sha256=art[a["build"]]["exe_sha256"],
                record=rec,
            )
        out.append(w)
    return out


def where(records, **kw):
    return [w for w in records if all(w.get(k) == v for k, v in kw.items())]


class Outputs(unittest.TestCase):
    def setUp(self):
        self.m, self.d, self.art = manifest(), delta(), artifacts()
        self.good = good_records(self.m, self.d, self.art)

    def check(self, records):
        return ts.outputs_check(self.m, self.d, self.art, records)

    def fails(self, records):
        self.assertTrue(self.check(records))

    def test_complete_and_correct_is_admitted(self):
        self.assertEqual(self.check(self.good), [])

    def test_the_run_count(self):
        # 8 valid fixtures x 4 arms; 4 error fixtures x 6 arms x 2 read buffers; 12 oracles; 4 rejections
        self.assertEqual(len(self.good), 32 + 48 + 12 + 4)

    # the first review's counterexamples (#145 6759658271)
    def test_no_records(self):
        self.fails([])

    def test_four_empty_objects(self):
        self.fails([{}, {}, {}, {}])

    def test_one_empty_record(self):
        r = copy.deepcopy(self.good)
        r[0]["record"] = {}
        self.fails(r)

    def test_missing_run(self):
        self.fails(self.good[1:])

    def test_duplicate_run(self):
        self.fails(self.good + [copy.deepcopy(self.good[0])])

    def test_matching_failures_on_a_valid_input(self):
        r = copy.deepcopy(self.good)
        for w in where(r, fixture="W", kind="arm"):
            w["record"].update(out=0, err="x", err_kind="Truncated", err_block=0)
        self.fails(r)

    def test_all_arms_agree_on_a_wrong_output(self):
        r = copy.deepcopy(self.good)
        for w in where(r, fixture="W", kind="arm"):
            w["record"]["sha256"] = "9" * 64
        self.fails(r)

    def test_error_without_its_block(self):
        r = copy.deepcopy(self.good)
        for w in where(r, fixture="E-crc-team", kind="arm"):
            w["record"]["err_block"] = None
        self.fails(r)

    def test_error_at_the_wrong_block(self):
        r = copy.deepcopy(self.good)
        for w in where(r, fixture="E-trunc-solo", kind="arm"):
            w["record"]["err_block"] += 1
        self.fails(r)

    def test_error_kind_differs_in_one_arm(self):
        r = copy.deepcopy(self.good)
        where(r, fixture="E-crc-solo", arm="B-plain1")[0]["record"]["err_kind"] = "Truncated"
        self.fails(r)

    def test_wrong_prefix(self):
        r = copy.deepcopy(self.good)
        where(r, fixture="E-trunc-team", arm="H-team")[0]["record"]["out"] += 1
        self.fails(r)

    def test_killed(self):
        r = copy.deepcopy(self.good)
        r[0]["signal"] = 9
        self.fails(r)

    # the recheck's counterexamples (#145 6760210997), each passing at 25b778cd
    def test_wrong_build_label(self):
        r = copy.deepcopy(self.good)
        where(r, kind="arm")[0]["build"] = "unrelated"
        self.fails(r)

    def test_a_different_executable_for_every_fixture(self):
        r = copy.deepcopy(self.good)
        for i, w in enumerate(r):
            w["exe_sha256"] = f"{i:064x}"
        self.fails(r)

    def test_success_records_carrying_an_error_identity(self):
        r = copy.deepcopy(self.good)
        for w in where(r, kind="arm"):
            if not w["record"]["err"]:
                w["record"].update(err_kind="BlockCrc", err_block=99)
        self.fails(r)

    def test_malformed_input_children_timing_out(self):
        r = copy.deepcopy(self.good)
        for w in where(r, kind="arm"):
            if w["record"]["err"]:
                w["exit_status"] = 124
        self.fails(r)

    # the recheck's other points
    def test_expected_decoder_error_with_a_normal_exit_is_admitted(self):
        r = copy.deepcopy(self.good)
        self.assertTrue(
            all(w["exit_status"] == 0 for w in where(r, kind="arm", fixture="E-crc-team"))
        )
        self.assertEqual(self.check(r), [])

    def test_timeout_flag(self):
        r = copy.deepcopy(self.good)
        where(r, kind="arm")[0]["timed_out"] = True
        self.fails(r)

    def test_missing_oracle_completion(self):
        self.fails([w for w in self.good if w["run"] != "oracle__C-records"])

    def test_oracle_that_did_not_reproduce(self):
        r = copy.deepcopy(self.good)
        where(r, run="oracle__W")[0]["record"]["sha256"] = "8" * 64
        self.fails(r)

    def test_missing_lbzip2_rejection(self):
        self.fails([w for w in self.good if w["run"] != "reject__E-crc-solo"])

    def test_lbzip2_accepting_a_malformed_fixture(self):
        r = copy.deepcopy(self.good)
        where(r, run="reject__E-trunc-team")[0]["exit_status"] = 0
        self.fails(r)

    def test_wrong_command_input_or_selector(self):
        r = copy.deepcopy(self.good)
        where(r, kind="arm", fixture="W")[0]["command"][-1] = "corpora/text.bz2"
        self.fails(r)
        r = copy.deepcopy(self.good)
        where(r, kind="arm", fixture="W")[0]["input_sha256"] = "0" * 64
        self.fails(r)
        r = copy.deepcopy(self.good)
        for w in where(r, arm="H-team"):
            w["env"].pop("AB_STAGED_TRAVERSE")
        self.fails(r)
        r = copy.deepcopy(self.good)
        where(r, arm="H-plain", fixture="E-crc-team")[0]["env"]["AB_READ_BUF"] = "1"
        self.fails(r)

    # the second recheck's witnesses (#145 6763089509, item 5), each admitted at c077cbdc
    def test_artifacts_head_sha_away_from_the_delta(self):
        self.art["head"]["sha"] = "9" * 40
        self.fails(self.good)

    def test_artifacts_base_sha_away_from_the_delta(self):
        self.art["base"]["sha"] = "8" * 40
        self.fails(self.good)

    def test_lbzip2_rejection_exit_not_its_typed_error_exit(self):
        for status in (None, -9, 124, 2, True, "1"):
            r = copy.deepcopy(self.good)
            w = where(r, run="reject__E-crc-team")[0]
            w.update(exit_status=status, timed_out=False, signal=None)
            self.assertTrue(self.check(r), status)

    def test_ab_exit_must_be_the_integer_zero(self):
        r = copy.deepcopy(self.good)
        where(r, kind="arm", fixture="E-crc-team")[0]["exit_status"] = False
        self.fails(r)

    def test_base_harness_older_than_the_schema(self):
        r = copy.deepcopy(self.good)
        for w in where(r, build="base"):
            w["record"]["schema"] = None
        self.fails(r)

    def test_record_without_sha256(self):
        r = copy.deepcopy(self.good)
        del where(r, kind="arm")[0]["record"]["sha256"]
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
