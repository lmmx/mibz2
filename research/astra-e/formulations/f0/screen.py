"""Cyclic linear state required by the marked inverse-LF byte stream."""
import argparse
import csv
import hashlib
import itertools
import sys
from pathlib import Path

import flint
from flint import nmod_poly

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import dense_audit, read_input


def cyclic_degree(period):
    # The first generator is the modulus of the cyclic convolution algebra.
    n = len(period)
    common = nmod_poly([1] + [0] * (n - 1) + [1], 2)
    degrees = []
    for bit in range(8):
        if common.degree() > 0:
            coefficients = list(period.translate(bytes((v >> bit) & 1 for v in range(256))))
            common = common.gcd(nmod_poly(coefficients, 2))
        degrees.append(n - common.degree())
    return degrees


def brute_degree(period):
    n = len(period)
    for k in range(n + 1):
        for taps in itertools.product(range(2), repeat=k):
            if all(period[(i + k) % n] == sum_xor(period[(i+j) % n] for j,t in enumerate(taps) if t)
                   for i in range(n)):
                return k
    raise AssertionError("the d-step identity always works")


def sum_xor(values):
    v = 0
    for x in values:
        v ^= x
    return v


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", nargs="*", type=Path)
    parser.add_argument("--small", action="store_true")
    args = parser.parse_args()
    if args.small:
        cases = 0
        for n in range(1, 8):
            for values in itertools.product(range(3), repeat=n):
                period = bytes(values)
                assert cyclic_degree(period)[-1] == brute_degree(period)
                cases += 1
        print(f"{cases} cyclic byte words, every binary-tap recurrence through the first admissible order verified")
        return
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(["record", "n", "mark", "marked_period", "bit0_degree", "joint_degree", "oracle_output_sha256", "python_flint"])
    for path in args.archives:
        runs, mark = read_input(path)
        _, output, orbit = dense_audit(runs, mark)
        d = orbit.index(mark, 1)
        degrees = cyclic_degree(bytes(output[:d]))
        writer.writerow([path.name, len(output), mark, d, degrees[0], degrees[-1], hashlib.sha256(output).hexdigest(), flint.__version__])
        sys.stdout.flush()


if __name__ == "__main__":
    main()
