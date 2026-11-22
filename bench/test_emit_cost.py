"""bench/emit_cost.py (bench-emit-cost's report) on synthetic records: each class where the
registration puts it, the pooled regression's intercept and slope recovered, and no class from
incomplete or malformed records or machine.

    python3 bench/test_emit_cost.py
"""

import io
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import emit_cost as ec  # noqa: E402

I9 = "Model name:            Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
BLOCKS = [(900_000, 90_000), (899_990, 120_000), (900_000, 60_000), (450_000, 30_000)]
OUT = 1000


def tsv(b, a_ns=20_000, workers=(1, 6, 10), rounds=4, drop=None, out=OUT, jitter=True):
    """emit_ns = a + b(w) m, b(w) a callable or a number; one refused block a decode"""
    lines = ["\t".join(ec.COLS)]
    for r in range(rounds):
        for o, w in enumerate(workers):
            if drop == (r, w):
                continue
            bw = b(w) if callable(b) else b
            for i, (n, m) in enumerate(BLOCKS):
                e = int(a_ns + bw * m + (((r * 7 + i * 3) % 5) - 2) * 50 * jitter)
                lines.append(f"blk\t{r}\t{w}\t{o}\t{i}\t{n}\t{m}\t{2 * n}\t{e}\t1\t0\t\t")
            lines.append(f"blk\t{r}\t{w}\t{o}\t9\t800000\t0\t300000\t0\t0\t1\t\t")
            lines.append(f"run\t{r}\t{w}\t{o}\t\t\t\t\t\t\t\t0.5\t{out}")
    return "\n".join(lines) + "\n"


class Report(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.lscpu = os.path.join(self.d, "lscpu.txt")
        with open(self.lscpu, "w") as f:
            f.write(I9)

    def run_on(self, text, *extra, lscpu=None):
        p = os.path.join(self.d, "runs.tsv")
        with open(p, "w") as f:
            f.write(text)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = ec.main(
                [
                    p,
                    "--lscpu",
                    lscpu or self.lscpu,
                    "--rounds",
                    "4",
                    "--out-bytes",
                    str(OUT),
                    *extra,
                ]
            )
        return rc, buf.getvalue()

    def test_classes(self):
        for b, cls in ((20.0, "PORT BOTH"), (4.0, "SHELVE BOTH"), (11.0, "PORT THE CHEAPER")):
            rc, out = self.run_on(tsv(b, a_ns=0))
            self.assertEqual(rc, 0, out)
            self.assertIn(f"CLASS: {cls}\n", out)

    def test_judged_count_alone_decides(self):
        rc, out = self.run_on(tsv(lambda w: 20.0 if w == 10 else 4.0, a_ns=0))
        self.assertEqual(rc, 0, out)
        self.assertIn("CLASS: PORT BOTH\n", out)

    def test_straddles(self):
        rc, out = self.run_on(tsv(15.0, a_ns=0))
        self.assertIn("(STRADDLES)", out)

    def test_regression_recovered(self):
        rc, out = self.run_on(tsv(6.0, a_ns=40_000, jitter=False))
        self.assertEqual(rc, 0, out)
        self.assertIn("pooled: a 40.0 us, slope 6.000 ns a row", out)

    def test_void(self):
        cases = {
            "missing": tsv(10.0, drop=(2, 10)),
            "bytes": tsv(10.0, out=OUT + 1),
            "header": tsv(10.0).replace("emit_ns", "emit"),
            "shape": tsv(10.0).replace("\t60000\t", "\t60001\t", 1),
        }
        for name, text in cases.items():
            rc, out = self.run_on(text)
            self.assertEqual(rc, 1, name)
            self.assertTrue(out.startswith("VOID"), name)
        other = os.path.join(self.d, "other.txt")
        with open(other, "w") as f:
            f.write("Model name: AMD EPYC\n")
        rc, out = self.run_on(tsv(10.0), lscpu=other)
        self.assertEqual(rc, 1)
        self.assertIn("not the i9-10900K", out)


if __name__ == "__main__":
    unittest.main()
