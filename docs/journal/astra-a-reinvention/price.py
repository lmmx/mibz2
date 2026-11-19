"""Replay a fixed geometry sample at private capacities and with actual interleaved blocks."""
import argparse
import csv
import subprocess
from pathlib import Path


SAMPLE = tuple(range(0, 233, 24))
# The eight jointly admitted sampled blocks in ascending order. Two further independent
# blocks, the next two large blocks after frozen, complete a ten-worker cohort.
COHORT = (0, 48, 72, 96, 120, 168, 192, 216, 1, 2)
ARMS = ("eager", "lazy", "shortcut", "whole")


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("binary", type=Path)
    ap.add_argument("traces", type=Path)
    ap.add_argument("outdir", type=Path)
    args = ap.parse_args()
    binary = args.binary.resolve()
    def trace(i, arm):
        return args.traces / f"{i:04}-{arm}.bin"
    with (args.outdir/"cache.csv").open("w") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(("block","arm","lines","reads","writes","read_fills","store_fills","writebacks",
                    "lookup_reads","lookup_read_fills"))
        for i in SAMPLE:
            for arm in ARMS:
                command = [binary, trace(i,arm), "512", "4096", "32768", "46811"]
                for row in subprocess.check_output(command,text=True).splitlines():
                    fields = row.split(",")
                    if arm in ("eager","lazy"):
                        fields[-2:] = ["",""]
                    w.writerow((i,arm,*fields))
    with (args.outdir/"teams.csv").open("w") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(("arm","workers","llc_lines","l2_fills","l2_writebacks","llc_write_only_misses",
                    "memory_reads","memory_writebacks"))
        for workers in (7,8,9,10):
            for arm in ARMS:
                command = [binary,"--team","327680",*(trace(i,arm) for i in COHORT[:workers])]
                row = subprocess.check_output(command,text=True).strip().split(",")
                w.writerow((arm,*row))
                f.flush()

    with (args.outdir/"warm.csv").open("w") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(("arm","workers","llc_lines","wave","memory_reads","memory_writebacks"))
        for workers in (7,10):
            for capacity in (163840,327680):
                for arm in ARMS:
                    command = [binary,"--warm-team",str(capacity),*(trace(i,arm) for i in COHORT[:workers])]
                    for row in subprocess.check_output(command,text=True).strip().splitlines():
                        w.writerow((arm,*row.split(",")))
                    f.flush()
