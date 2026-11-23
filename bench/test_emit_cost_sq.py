"""bench/emit_cost_sq.py (bench-emit-cost-sq's verdict) on synthetic records: each class and
decision where the frozen rule (record 93 §5, its addendum, LOG 7097052169) puts it, E and K
recovered, only the judged count deciding, and no verdict from incomplete or inconsistent
records or another machine.

    python3 bench/test_emit_cost_sq.py
"""

import io
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import emit_cost_sq as ecs  # noqa: E402

I9 = "Model name:            Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
# (n, m, two-ended): three two-ended blocks and one forward one
BLOCKS = [(900_000, 100_000, 1), (899_990, 120_000, 1), (900_000, 80_000, 1), (450_000, 30_000, 0)]
OUT = 1000
BASE = {"emit": 5.8, "construct": 7.0, "square": 0.0}


def tsv(
    arms,
    workers=(1, 6, 10),
    rounds=6,
    drop=None,
    out=OUT,
    jitter=0.05,
    base=BASE,
    mislabel=None,
    bidi_off=None,
):
    """per arm a callable w -> {emit, construct, square} ns a row (two-ended blocks; forward
    blocks keep base's); `jitter` ns a row of deterministic noise a round"""
    lines = ["\t".join(ecs.COLS)]
    cells = [(w, a) for w in workers for a in ["base", *arms]]
    for r in range(rounds):
        for o in range(len(cells)):
            w, a = cells[(o + r) % len(cells)]
            if drop == (r, w, a):
                continue
            noise = (((r * 7 + w + 3 * len(a)) % 5) - 2) * jitter
            for i, (n, m, bidi) in enumerate(BLOCKS):
                p = base if a == "base" or not bidi else arms[a](w)
                if bidi_off == (r, w, a) and i == 0:
                    bidi = 0
                c = int((p["construct"] + noise) * m)
                s = int(p["square"] * m) + 20
                e = int((p["emit"] + noise) * m)
                rounds_ran = int(a != "base" and bidi == 1)
                scratch = 8 * 120_000 if a != "base" else 0
                label = mislabel[3] if mislabel and mislabel[:3] == (r, w, a) and i == 0 else a
                lines.append(
                    f"blk\t{r}\t{w}\t{label}\t{o}\t{i}\t{n}\t{m}\t{c}\t{s}\t{e}\t{bidi}\t0\t"
                    f"{rounds_ran}\t{scratch}\t\t"
                )
            lines.append(f"blk\t{r}\t{w}\t{a}\t{o}\t9\t800000\t0\t300000\t0\t0\t0\t1\t0\t0\t\t")
            lines.append(f"run\t{r}\t{w}\t{a}\t{o}\t" + "\t" * 10 + f"0.5\t{out}")
    return "\n".join(lines) + "\n"


def arm(emit, d_construct=0.2, square=0.0):
    return lambda w: {"emit": emit, "construct": BASE["construct"] + d_construct, "square": square}


class Verdict(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.lscpu = os.path.join(self.d, "lscpu.txt")
        with open(self.lscpu, "w") as f:
            f.write(I9)

    def run_on(self, text, *extra, lscpu=None, rounds=6):
        p = os.path.join(self.d, "runs.tsv")
        with open(p, "w") as f:
            f.write(text)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = ecs.main(
                [
                    p,
                    "--lscpu",
                    lscpu or self.lscpu,
                    "--rounds",
                    str(rounds),
                    "--out-bytes",
                    str(OUT),
                    *extra,
                ]
            )
        return rc, buf.getvalue()

    def verdict(self, sq, shadow, **kw):
        rc, out = self.run_on(tsv({"sq": sq, "sq-shadow": shadow}, **kw))
        self.assertEqual(rc, 0, out)
        return out

    def test_kill(self):
        # sq's E > 0.80: KILL, whatever sq-shadow does
        out = self.verdict(arm(5.0, square=2.0), arm(3.0, d_construct=0.0, square=0.0))
        self.assertIn("sq at 10 workers: KILL\n", out)
        self.assertIn("sq-shadow at 10 workers: PORT-CANDIDATE (decides nothing: sq is KILL)", out)
        self.assertIn("DECISION: KILL", out)

    def test_shadow_then_end(self):
        # sq: E 0.62, a net loss (square 2.5): SHADOW; sq-shadow: E 0.9: END
        out = self.verdict(arm(3.6, square=2.5), arm(5.2))
        self.assertIn("sq at 10 workers: SHADOW\n", out)
        self.assertIn("sq-shadow at 10 workers: END\n", out)
        self.assertIn("DECISION: END", out)

    def test_shadow_then_port(self):
        # sq-shadow: E 0.7, K about -1.5: PORT-CANDIDATE
        out = self.verdict(arm(3.6, square=2.5), arm(4.1))
        self.assertIn("sq at 10 workers: SHADOW\n", out)
        self.assertIn("sq-shadow at 10 workers: PORT-CANDIDATE\n", out)
        self.assertIn("DECISION: PORT-CANDIDATE: sq-shadow,", out)

    def test_both_port_the_lower_k(self):
        out = self.verdict(arm(3.6, square=0.5), arm(3.9))
        self.assertIn("sq at 10 workers: PORT-CANDIDATE\n", out)
        self.assertIn("sq-shadow at 10 workers: PORT-CANDIDATE\n", out)
        # sq: K = 3.6 + 0.5 + 0.2 - 5.8 = -1.5 a row on the two-ended blocks; sq-shadow -1.7
        self.assertIn("DECISION: PORT-CANDIDATE: sq-shadow (both are; the lower K", out)
        out = self.verdict(arm(3.0, square=0.5), arm(3.9))
        self.assertIn("DECISION: PORT-CANDIDATE: sq (both are; the lower K", out)

    def test_sq_port_shadow_end(self):
        out = self.verdict(arm(3.6, square=0.5), arm(5.6))
        self.assertIn("DECISION: PORT-CANDIDATE: sq,", out)

    def test_k_line_and_interval(self):
        # K past -0.3 but its interval not wholly below 0 (noisy): not PORT-CANDIDATE
        out = self.verdict(arm(3.6, square=1.8, d_construct=0.0), arm(5.6), jitter=0.3)
        self.assertIn("sq at 10 workers: SHADOW\n", out)
        self.assertRegex(out, r"sq against base: E 0\.6\d\d .*; K -0\.46\d \[-\d\.\d+, 0\.\d+\]")
        # E just under 0.80 passes the E line; K about -0.18 a row does not pass the K line
        out = self.verdict(arm(4.5, square=1.1, d_construct=0.0), arm(5.6), jitter=0.0)
        self.assertIn("sq at 10 workers: SHADOW\n", out)

    def test_e_and_k_recovered(self):
        out = self.verdict(arm(2.9, square=1.0, d_construct=0.3), arm(5.6), jitter=0.0)
        # on retained rows over all open blocks: three two-ended blocks of 300,000 rows and a
        # forward one of 30,000 at base's
        m2, m1 = 300_000, 30_000
        e = (2.9 * m2 + 5.8 * m1) / (5.8 * (m2 + m1))
        k = ((2.9 + 1.0 + 0.3 - 5.8) * m2) / (m2 + m1)
        self.assertIn(f"sq against base: E {e:.3f} [{e:.3f}, {e:.3f}]", out)
        self.assertIn(f"K {k:.3f} [{k:.3f}, {k:.3f}]", out)

    def test_judged_count_alone_decides(self):
        sq = lambda w: arm(3.0, square=0.3)(w) if w == 10 else arm(5.7, square=0.3)(w)  # noqa: E731
        out = self.verdict(sq, arm(5.6))
        self.assertIn("sq at 10 workers: PORT-CANDIDATE\n", out)
        self.assertIn("class by the rule KILL (decides nothing at this count)", out)

    def test_machine_check(self):
        out = self.verdict(arm(3.6), arm(5.0))
        self.assertIn("within 10%", out)
        out = self.verdict(arm(3.6), arm(5.0), base={**BASE, "emit": 8.0})
        self.assertIn("NOT within 10%", out)

    def test_void(self):
        both = {"sq": arm(3.6), "sq-shadow": arm(4.0)}
        cases = {
            "missing": tsv(both, drop=(2, 10, "sq-shadow")),
            "bytes": tsv(both, out=OUT + 1),
            "header": tsv(both).replace("emit_ns", "emit"),
            "shape": tsv(both).replace("\t80000\t", "\t80001\t", 1),
            "two-ended": tsv(both, bidi_off=(3, 6, "sq")),
            "arm": tsv(both, mislabel=(1, 1, "sq", "sq-shadow")),
        }
        for name, text in cases.items():
            rc, out = self.run_on(text)
            self.assertEqual(rc, 1, f"{name}: {out}")
            self.assertTrue(out.startswith("VOID"), name)
        # a base block with the arms' scratch
        text = tsv(both).splitlines()
        i = next(i for i, x in enumerate(text) if x.startswith("blk\t0\t1\tbase\t"))
        f = text[i].split("\t")
        f[14] = "4096"
        text[i] = "\t".join(f)
        rc, out = self.run_on("\n".join(text) + "\n")
        self.assertEqual(rc, 1, out)
        other = os.path.join(self.d, "other.txt")
        with open(other, "w") as f:
            f.write("Model name: AMD EPYC\n")
        rc, out = self.run_on(tsv(both), lscpu=other)
        self.assertEqual(rc, 1)
        self.assertIn("not the i9-10900K", out)


if __name__ == "__main__":
    unittest.main()
