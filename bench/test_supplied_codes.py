"""bench/supplied_codes.py (bench-supplied-codes' verdict) on synthetic records: each decision
where the frozen rule puts it (ADVANCE, STOP, INCONCLUSIVE, at the judged count only), the
ratios recovered, and no verdict from incomplete or inconsistent records or another machine.

    python3 bench/test_supplied_codes.py
"""

import io
import math
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import supplied_codes as sc  # noqa: E402

I9 = "Model name:            Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
# (n, m, cycle, two-ended, id): two two-ended blocks (one a short cycle), one forward block, the
# first repeated (a copy)
BLOCKS = [
    (900_000, 100_000, 100_000, 1, "00000000000000aa"),
    (899_990, 120_000, 40_000, 1, "00000000000000bb"),
    (450_000, 30_000, 30_000, 0, "00000000000000cc"),
    (900_000, 100_000, 100_000, 1, "00000000000000aa"),
]
OUT = 1000
WORKERS = (1, 10)


def cells(r, workers):
    rot = (r // 4) % len(workers)
    conds = (0, 1) if (r // 2) % 2 == 0 else (1, 0)
    arms = ("open", "supplied") if r % 2 == 0 else ("supplied", "open")
    return [
        (workers[(i + rot) % len(workers)], c, a)
        for i in range(len(workers))
        for c in conds
        for a in arms
    ]


def tsv(
    wall_ratio,
    rounds=8,
    workers=WORKERS,
    spread=0.01,
    consume_ratio=0.5,
    drop=None,
    out=OUT,
    mislabel=None,
    unsupplied=None,
    mismatch_round=None,
    collisions=0,
    no_meta=False,
    shape_off=None,
):
    """`wall_ratio(w)`: B/A wall at w workers; each round's ratio is that times
    exp(+-spread) alternately"""
    blank = [""] * len(sc.COLS)

    def line(**kw):
        f = list(blank)
        for k, v in kw.items():
            f[sc.COLS.index(k)] = str(v)
        return "\t".join(f)

    lines = ["\t".join(sc.COLS)]
    if not no_meta:
        lines.append(
            line(
                kind="meta",
                workers=10,
                arm="record",
                cond=0,
                n=3,
                m=170_000,
                scratch=680_000,
                wall_s=0.5,
                bytes=out,
                mismatches=collisions,
            )
        )
    for r in range(rounds):
        for o, (w, c, a) in enumerate(cells(r, workers)):
            if drop == (r, w, a, c):
                continue
            k = wall_ratio(w) * math.exp(spread if r % 2 == 0 else -spread)
            wall = 0.4 * (k if a == "supplied" else 1.0)
            for i, (n, m, cyc, bidi, bid) in enumerate(BLOCKS):
                if shape_off == (r, w, a, c) and i == 2:
                    m = m - 1
                consume = int(6.0 * m * (consume_ratio if a == "supplied" else 1.0))
                sup = int(a == "supplied")
                if unsupplied == (r, w, a, c) and i == 0:
                    sup = 0
                label = a
                if mislabel == (r, w, a, c) and i == 0:
                    label = "open" if a == "supplied" else "supplied"
                lines.append(
                    line(
                        kind="blk",
                        round=r,
                        workers=w,
                        arm=label,
                        cond=c,
                        order=o,
                        seq=i,
                        n=n,
                        m=m,
                        cycle=cyc if sup else 0,
                        bidi=bidi,
                        refused=0,
                        supplied=sup,
                        construct_ns=7 * m,
                        deliver_ns=(m // 2 if sup else 20),
                        cond_ns=(m // 10 if c else 20),
                        consume_ns=consume,
                        scratch=40 * 120_000,
                        id=bid,
                    )
                )
            lines.append(
                line(
                    kind="blk",
                    round=r,
                    workers=w,
                    arm=a,
                    cond=c,
                    order=o,
                    seq=9,
                    n=800_000,
                    m=0,
                    cycle=0,
                    bidi=0,
                    refused=1,
                    supplied=0,
                    construct_ns=300_000,
                    deliver_ns=0,
                    cond_ns=0,
                    consume_ns=0,
                    scratch=0,
                    id="0000000000000000",
                )
            )
            lines.append(
                line(
                    kind="run",
                    round=r,
                    workers=w,
                    arm=a,
                    cond=c,
                    order=o,
                    wall_s=f"{wall:.6f}",
                    bytes=out,
                )
            )
        lines.append(
            line(
                kind="check",
                round=r,
                workers=10,
                arm="check",
                cond=0,
                supplied=3,
                wall_s=0.5,
                bytes=out,
                mismatches=int(mismatch_round == r),
            )
        )
    return "\n".join(lines) + "\n"


class Verdict(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.lscpu = os.path.join(self.d, "lscpu.txt")
        with open(self.lscpu, "w") as f:
            f.write(I9)

    def run_on(self, text, lscpu=None, rounds=8):
        p = os.path.join(self.d, "runs.tsv")
        with open(p, "w") as f:
            f.write(text)
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = sc.main(
                [
                    p,
                    "--lscpu",
                    lscpu or self.lscpu,
                    "--workers",
                    "1,10",
                    "--judged",
                    "10",
                    "--rounds",
                    str(rounds),
                    "--out-bytes",
                    str(OUT),
                ]
            )
        return code, buf.getvalue()

    def decision(self, text):
        code, out = self.run_on(text)
        self.assertEqual(code, 0, out)
        return [x for x in out.splitlines() if x.startswith("DECISION:")][0], out

    def test_advance(self):
        d, out = self.decision(tsv(lambda w: 0.80))
        self.assertTrue(d.startswith("DECISION: ADVANCE"), d)
        self.assertIn("0.800 [", d)

    def test_stop(self):
        d, _ = self.decision(tsv(lambda w: 1.00))
        self.assertTrue(d.startswith("DECISION: STOP"), d)

    def test_between_the_lines_is_inconclusive(self):
        # 0.92: the upper bound above 0.90 and the lower below 0.95
        d, _ = self.decision(tsv(lambda w: 0.92))
        self.assertTrue(d.startswith("DECISION: INCONCLUSIVE"), d)

    def test_a_wide_interval_is_inconclusive(self):
        # the mean below 0.90 but the interval's upper bound above it
        d, _ = self.decision(tsv(lambda w: 0.85, spread=0.3))
        self.assertTrue(d.startswith("DECISION: INCONCLUSIVE"), d)

    def test_only_the_judged_count_decides(self):
        d, out = self.decision(tsv(lambda w: 0.70 if w == 1 else 1.0))
        self.assertTrue(d.startswith("DECISION: STOP"), d)
        self.assertIn("ADVANCE (decides nothing at this count)", out)

    def test_the_diagnostic_is_recovered_and_decides_nothing(self):
        d, out = self.decision(tsv(lambda w: 1.0, consume_ratio=0.25))
        self.assertTrue(d.startswith("DECISION: STOP"), d)
        self.assertIn(
            "DIAGNOSTIC: consumer per retained row, B/A (cond 1, the substitution "
            "outside the clock): 0.250 [",
            out,
        )
        # the distinct blocks: three, the copy pooled with its original
        self.assertIn("distinct open blocks 3 (2 two-ended)", out)
        self.assertIn("marked cycles: 1 of 4 open blocks", out)

    def test_void_on_another_machine(self):
        other = os.path.join(self.d, "other.txt")
        with open(other, "w") as f:
            f.write("Model name: AMD EPYC 7B13\n")
        code, out = self.run_on(tsv(lambda w: 0.8), lscpu=other)
        self.assertEqual(code, 1)
        self.assertIn("VOID", out)

    def void(self, text, needle):
        code, out = self.run_on(text)
        self.assertEqual(code, 1, out)
        self.assertTrue(out.startswith("VOID"), out)
        self.assertIn(needle, out)

    def test_void_on_a_missing_decode(self):
        self.void(
            tsv(lambda w: 0.8, drop=(3, 10, "supplied", 0)), "missing decode (3, 10, 'supplied', 0)"
        )

    def test_void_on_wrong_bytes(self):
        self.void(tsv(lambda w: 0.8, out=999), "not 1000")

    def test_void_on_a_mislabelled_block(self):
        self.void(tsv(lambda w: 0.8, mislabel=(2, 10, "open", 1)), "a block under")

    def test_void_on_an_unsupplied_block(self):
        self.void(
            tsv(lambda w: 0.8, unsupplied=(1, 10, "supplied", 0)), "an arm-B block not supplied"
        )

    def test_void_on_a_check_mismatch(self):
        self.void(tsv(lambda w: 0.8, mismatch_round=5), "round 5's check")

    def test_void_on_a_collision(self):
        self.void(tsv(lambda w: 0.8, collisions=1), "collisions")

    def test_void_without_the_store(self):
        self.void(tsv(lambda w: 0.8, no_meta=True), "0 meta lines")

    def test_void_on_different_blocks(self):
        self.void(tsv(lambda w: 0.8, shape_off=(4, 1, "open", 0)), "differ between decodes")

    def test_void_on_a_missing_round(self):
        code, out = self.run_on(tsv(lambda w: 0.8, rounds=7))
        self.assertEqual(code, 1)
        self.assertIn("round 7's check missing", out)


if __name__ == "__main__":
    unittest.main()
