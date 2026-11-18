"""bench/moonshot/moonshot.py's logic: the ratio statistics, the gate's pass and fail, the
settle rule, the round order, the proxy's reading and its out-of-calibration flag.

    python3 bench/moonshot/test_moonshot.py
"""

import json
import math
import random
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import moonshot as m  # noqa: E402


class Statistics(unittest.TestCase):
    def test_t_quantiles_match_the_tables(self):
        for df, want in [
            (1, 12.706),
            (2, 4.303),
            (4, 2.776),
            (9, 2.262),
            (29, 2.045),
            (120, 1.980),
        ]:
            self.assertAlmostEqual(m.t_quantile(0.975, df), want, places=3)

    def test_t_cdf_is_symmetric(self):
        for df in (1, 3, 10):
            for t in (0.3, 1.0, 2.5):
                self.assertAlmostEqual(m.t_cdf(t, df) + m.t_cdf(-t, df), 1.0, places=12)

    def test_identical_walls_give_one_and_a_zero_width_interval(self):
        r = m.paired_ratio([1.0, 2.0, 3.0], [1.0, 2.0, 3.0])
        self.assertAlmostEqual(r["geomean"], 1.0)
        self.assertAlmostEqual(r["lo"], 1.0)
        self.assertAlmostEqual(r["hi"], 1.0)
        self.assertEqual(m.verdict(r), "INCONCLUSIVE")

    def test_the_geometric_mean_is_of_the_paired_ratios(self):
        r = m.paired_ratio([0.8, 1.6, 0.4], [1.0, 2.0, 0.5])
        self.assertAlmostEqual(r["geomean"], 0.8)
        r = m.paired_ratio([2.0, 0.5], [1.0, 1.0])
        self.assertAlmostEqual(r["geomean"], 1.0)

    def test_the_interval_is_a_t_interval_on_the_logs(self):
        cand, base = [0.9, 0.95, 0.85, 0.92], [1.0, 1.0, 1.0, 1.0]
        logs = [math.log(c) for c in cand]
        mean = sum(logs) / 4
        sd = math.sqrt(sum((x - mean) ** 2 for x in logs) / 3)
        half = 3.182446 * sd / 2
        r = m.paired_ratio(cand, base)
        self.assertAlmostEqual(r["lo"], math.exp(mean - half), places=5)
        self.assertAlmostEqual(r["hi"], math.exp(mean + half), places=5)
        self.assertEqual(m.verdict(r), "FASTER")

    def test_verdicts(self):
        self.assertEqual(m.verdict({"lo": 1.01, "hi": 1.2}), "SLOWER")
        self.assertEqual(m.verdict({"lo": 0.8, "hi": 0.99}), "FASTER")
        self.assertEqual(m.verdict({"lo": 0.9, "hi": 1.1}), "INCONCLUSIVE")
        self.assertEqual(m.verdict(m.paired_ratio([1.0], [2.0])), "NO INTERVAL")

    def test_coverage_is_near_95_percent(self):
        rng = random.Random(7)
        hits, trials = 0, 2000
        for _ in range(trials):
            base = [1.0] * 6
            cand = [math.exp(math.log(0.9) + rng.gauss(0, 0.05)) for _ in range(6)]
            r = m.paired_ratio(cand, base)
            hits += r["lo"] <= 0.9 <= r["hi"]
        self.assertTrue(0.93 <= hits / trials <= 0.97, hits / trials)

    def test_bad_input_is_refused(self):
        with self.assertRaises(ValueError):
            m.paired_ratio([1.0], [1.0, 2.0])
        with self.assertRaises(ValueError):
            m.paired_ratio([], [])
        with self.assertRaises(ValueError):
            m.paired_ratio([0.0], [1.0])

    def test_the_median(self):
        self.assertEqual(m.median([3, 1, 2]), 2)
        self.assertEqual(m.median([4, 1, 2, 3]), 2.5)


class Schedule(unittest.TestCase):
    def test_settle_is_zero_or_at_least_two_seconds(self):
        self.assertEqual(m.check_settle(0), 0)
        self.assertEqual(m.check_settle(2.0), 2.0)
        self.assertEqual(m.check_settle(5), 5)
        for bad in (0.25, 1.0, 1.99):
            with self.assertRaises(ValueError):
                m.check_settle(bad)

    def test_each_arm_takes_each_position_equally_often(self):
        for arms in (["lazy", "candidate"], ["lazy", "candidate", "opend"]):
            k = len(arms)
            seen = {a: [0] * k for a in arms}
            before = {(a, b): 0 for a in arms for b in arms if a != b}
            for rnd in range(2 * k):
                o = m.order(arms, rnd)
                self.assertEqual(sorted(o), sorted(arms))
                for i, a in enumerate(o):
                    seen[a][i] += 1
                    for b in o[i + 1 :]:
                        before[(a, b)] += 1
            for a in arms:
                self.assertEqual(seen[a], [2] * k)
            for (a, b), n in before.items():
                self.assertEqual(n, before[(b, a)])

    def test_sample_blocks(self):
        self.assertEqual(m.sample_blocks(431, 4), [53, 161, 269, 377])
        self.assertEqual(m.sample_blocks(3, 10), [0, 1, 2])
        self.assertEqual(len(m.sample_blocks(431, 16)), 16)


class Gate(unittest.TestCase):
    def lines(self, passes, final="PASS"):
        out = [
            json.dumps({"case": f"c{i}", "bytes": 1, "pass": p, "failure": None if p else "x"})
            for i, p in enumerate(passes)
        ]
        if final:
            out.append(
                json.dumps({"arm": "candidate", "verdict": final, "failures": passes.count(False)})
            )
        return out

    def test_a_clean_run_passes(self):
        g = m.gate_verdict(self.lines([True, True, True]), 0)
        self.assertEqual(g["verdict"], "PASS")
        self.assertEqual(g["cases"], 3)

    def test_any_failing_case_fails(self):
        g = m.gate_verdict(self.lines([True, False, True], final="FAIL"), 1)
        self.assertEqual(g["verdict"], "FAIL")
        self.assertEqual(g["failed"], [{"case": "c1", "failure": "x"}])

    def test_a_failing_case_fails_even_with_a_pass_line_and_exit_zero(self):
        self.assertEqual(m.gate_verdict(self.lines([True, False]), 0)["verdict"], "FAIL")

    def test_a_crash_or_missing_verdict_fails(self):
        self.assertEqual(m.gate_verdict(self.lines([True], final=None), 0)["verdict"], "FAIL")
        self.assertEqual(m.gate_verdict(self.lines([True]), 101)["verdict"], "FAIL")
        self.assertEqual(m.gate_verdict([], 0)["verdict"], "FAIL")
        self.assertEqual(m.gate_verdict(self.lines([]), 0)["verdict"], "FAIL")


RAW = {
    "out_bytes": 1000,
    "instructions": 20000,
    "cycles": 8000,
    "loads": 4000,
    "stores": 3000,
    "l1_miss": 400,
    "l2_miss": 100,
    "llc_miss": 10,
    "l2_transfers": 500,
    "l3_transfers": 200,
    "dram_read_lines": 60,
    "dram_write_lines": 20,
    "dram_drain_lines": 20,
    "prefetched_lines": 50,
    "past_l2_loads": 80,
    "dram_loads": 10,
    "dep_past_l2_loads": 20,
    "dep_dram_loads": 2,
    "dep_any_loads": 900,
    "max_chain_past_l2": 7,
    "max_chain_dram": 3,
    "chain_sum_dram": 12,
    "dram_latency_sum": 5000,
    "dram_busy_cycles": 2500,
    "dep_overlap_sum": 9,
    "dep_overlap_n": 2,
    "store_buffer_stall_cycles": 0,
    "config": {},
}


class Proxy(unittest.TestCase):
    def test_the_line_is_read_from_valgrind_output(self):
        text = "==1== noise\n==1== MOONSHOT-PROXY " + json.dumps(RAW) + "\n==1== more\n"
        self.assertEqual(m.parse_proxy(text)["cycles"], 8000)
        with self.assertRaises(ValueError):
            m.parse_proxy("==1== nothing here")

    def test_metrics_per_byte(self):
        x = m.proxy_metrics(RAW)
        self.assertAlmostEqual(x["dram_bytes_per_byte"], 64 * 100 / 1000)
        self.assertAlmostEqual(x["l2_bytes_per_byte"], 32.0)
        self.assertAlmostEqual(x["dep_dram_links_per_byte"], 0.002)
        self.assertAlmostEqual(x["dram_mlp"], 2.0)
        self.assertAlmostEqual(x["overlap_per_dep_link"], 4.5)
        self.assertAlmostEqual(x["dram_loads_per_dep_link"], 5.0)
        self.assertAlmostEqual(x["cycles_per_byte"], 8.0)
        with self.assertRaises(ValueError):
            m.proxy_metrics(dict(RAW, out_bytes=0))

    def test_no_dependent_links_leaves_overlap_undefined(self):
        x = m.proxy_metrics(dict(RAW, dep_dram_loads=0, dep_overlap_n=0, dep_overlap_sum=0))
        self.assertIsNone(x["overlap_per_dep_link"])
        self.assertIsNone(x["dram_loads_per_dep_link"])

    def test_out_of_calibration(self):
        refs = [
            m.proxy_metrics(RAW),
            m.proxy_metrics(dict(RAW, dram_write_lines=40, dep_past_l2_loads=40)),
        ]
        self.assertEqual(m.out_of_range(m.proxy_metrics(dict(RAW, dram_write_lines=30)), refs), [])
        flags = m.out_of_range(m.proxy_metrics(dict(RAW, dep_past_l2_loads=400)), refs)
        self.assertEqual(len(flags), 1)
        self.assertTrue(flags[0].startswith("dep_past_l2_links_per_byte"))
        # (within the margin is in range)
        self.assertEqual(m.out_of_range(m.proxy_metrics(dict(RAW, dep_past_l2_loads=42)), refs), [])

    def test_nominal_latencies(self):
        mo = json.loads(m.CALIBRATION.read_text())["model"]
        self.assertEqual(m.nominal_latencies(mo, 4)["llc_kb"], 5120)
        self.assertEqual(m.nominal_latencies(mo, 1, kappa=10)["l3_lat"], mo["l3_latency_floor"])
        self.assertEqual(
            m.nominal_latencies(mo, 3, kappa=10)["l3_lat"], mo["l3_latency_floor"] + 20
        )
        # (lazy's nominal bandwidth rises with the workers, and so its DRAM latency)
        lat = [m.nominal_latencies(mo, w)["dram_lat"] for w in (1, 4, 7, 8, 10)]
        self.assertEqual(lat, sorted(lat))
        self.assertGreaterEqual(lat[0], mo["dram"]["idle"])

    def test_the_dram_queue_fits_bench_imc_ten(self):
        mo = json.loads(m.CALIBRATION.read_text())["model"]
        for gbs, cycles in mo["dram"]["points_gbs_cycles"]:
            self.assertLess(abs(m.dram_latency(gbs, mo) - cycles), 50)
        self.assertAlmostEqual(m.dram_latency(0, mo), mo["dram"]["idle"])
        # (capped short of the ceiling)
        self.assertLess(m.dram_latency(1e9, mo), 1e5)

    def test_pick_spreads_evenly(self):
        self.assertEqual(m.pick(list(range(10)), 2), [2, 7])
        self.assertEqual(m.pick([1, 2], 5), [1, 2])

    def test_the_sample_leaves_out_the_warm_block(self):
        cal = json.loads(m.CALIBRATION.read_text())
        s = m.proxy_sample(cal, "48+16")
        self.assertEqual((len(s["full"]), len(s["tiny"])), (48, 16))
        self.assertNotIn(s["warm"], s["full"] + s["tiny"])
        self.assertEqual((s["n_full"], s["n_tiny"]), (233, 198))
        self.assertTrue(set(s["slope"]) <= set(s["full"]))

    def test_the_corpus_total_weights_each_stratum(self):
        runs = {
            1: dict(RAW, cycles=100, out_bytes=10),
            2: dict(RAW, cycles=300, out_bytes=30),
            9: dict(RAW, cycles=5, out_bytes=1, max_chain_dram=9),
        }
        t = m.corpus_total(runs, [1, 2], [9], 4, 10)
        self.assertAlmostEqual(t["cycles"], 200 * 4 + 5 * 10)
        self.assertAlmostEqual(t["out_bytes"], 20 * 4 + 10)
        self.assertEqual(t["max_chain_dram"], 9)
        self.assertNotIn("config", t)

    def mo(self):
        mo = json.loads(m.CALIBRATION.read_text())["model"]
        return dict(mo, i9_lazy_dram_bpb=8.0, lazy_anchor={"workers": 7, "gbs": 20.0})

    def test_the_fixed_point_anchors_lazy_and_is_neutral_for_lazy_itself(self):
        mo = self.mo()
        lz = {"cpb": 12.0, "lat0": 500, "rel_slope": 0.001, "dram_bpb": 6.0}
        cells = {
            "7": {"lazy": lz, "same": dict(lz)},
            "8": {"lazy": dict(lz, cpb=13.0), "same": dict(lz, cpb=13.0)},
        }
        sol = m.solve(cells, mo)
        self.assertAlmostEqual(sol["7"]["lazy"]["gbs"], 20.0)
        self.assertAlmostEqual(sol["7"]["lazy"]["lat"], m.dram_latency(20.0, mo))
        self.assertAlmostEqual(m.ratio(sol, "same", 7, 7), 1.0)
        self.assertAlmostEqual(sol["7"]["lazy"]["c0"], 2.0)
        # (an arm the same as lazy at 8 workers over lazy at 7: its workers over 7, its cycles)
        self.assertGreater(sol["8"]["lazy"]["gbs"], 20.0)

    def test_more_dram_traffic_raises_an_arms_latency_and_its_cycles(self):
        mo = self.mo()
        # (lazy run at its anchored latency, so that only the arms' own latencies move)
        lz = {"cpb": 12.0, "lat0": m.dram_latency(20.0, mo), "rel_slope": 0.001, "dram_bpb": 6.0}
        heavy = dict(lz, dram_bpb=12.0)
        deaf = dict(heavy, rel_slope=0.0)
        sol = m.solve({"7": {"lazy": lz, "heavy": heavy, "deaf": deaf}}, mo)
        self.assertGreater(sol["7"]["heavy"]["lat"], sol["7"]["lazy"]["lat"])
        self.assertGreater(m.ratio(sol, "heavy", 7, 7), 1.0)
        # (an arm insensitive to DRAM latency pays nothing for it)
        self.assertAlmostEqual(m.ratio(sol, "deaf", 7, 7), 1.0)

    def test_the_fixed_point_needs_lazy_at_the_anchor(self):
        with self.assertRaises(ValueError):
            m.solve(
                {"8": {"lazy": {"cpb": 1, "lat0": 500, "rel_slope": 0, "dram_bpb": 1}}}, self.mo()
            )

    def test_predicted_ratio_scales_by_workers(self):
        self.assertAlmostEqual(m.predicted_ratio(10, 7, 10, 7), 1.0)
        # eight workers at the same cycles a byte as seven: 7/8 of the wall
        self.assertAlmostEqual(m.predicted_ratio(10, 8, 10, 7), 7 / 8)

    def test_calibration_has_one_fitted_cell(self):
        cal = json.loads(m.CALIBRATION.read_text())
        roles = [c["role"] for c in cal["i9"]]
        self.assertEqual(roles.count("fitted"), 1)
        self.assertGreaterEqual(roles.count("held-out"), 4)


class Timing(unittest.TestCase):
    def test_summary_pairs_by_round(self):
        runs = []
        for rnd in range(4):
            runs.append(
                {"arm": "lazy", "workers": 2, "round": rnd, "wall_s": 1.0 + rnd, "peak_rss_kb": 100}
            )
            runs.append(
                {
                    "arm": "candidate",
                    "workers": 2,
                    "round": rnd,
                    "wall_s": 0.8 * (1.0 + rnd),
                    "peak_rss_kb": 90,
                }
            )
        # an unpaired run is left out
        runs.append({"arm": "lazy", "workers": 2, "round": 9, "wall_s": 50.0, "peak_rss_kb": 100})
        s = m.summarise_timing(runs, ["lazy", "candidate"], [2])
        c = s["2"]["candidate"]
        self.assertEqual(s["2"]["rounds"], 4)
        self.assertAlmostEqual(c["geomean"], 0.8)
        self.assertAlmostEqual(c["speedup"], 1.25)
        self.assertAlmostEqual(c["peak_rss_ratio"], 0.9)


class Perf(unittest.TestCase):
    def test_perf_stat_csv_is_read(self):
        text = "# started on x\n\n1234,,cycles,100.00,,\n<not counted>,,l1d_pend_miss.pending,0,,\n99,,instructions,100.00,1.0,insn per cycle\n"
        self.assertEqual(m.parse_perf(text), {"cycles": 1234.0, "instructions": 99.0})

    def test_counters_per_byte_in_the_summary(self):
        runs = []
        for rnd in range(2):
            for arm, c in (("lazy", 100.0), ("open", 50.0)):
                runs.append(
                    {
                        "arm": arm,
                        "workers": 7,
                        "round": rnd,
                        "wall_s": 1.0,
                        "peak_rss_kb": 1,
                        "out_bytes": 10,
                        "counters": {e: c for e in m.PERF_EVENTS},
                    }
                )
        s = m.summarise_timing(runs, ["lazy", "open"], [7])
        self.assertEqual(s["7"]["counters_per_byte"]["open"]["cycles"], 5.0)
        self.assertEqual(s["7"]["counters_per_byte"]["lazy"]["cycle_activity.stalls_l3_miss"], 10.0)


if __name__ == "__main__":
    unittest.main()
