"""Per-phase payload and memory-operand counts. Q8 sums ceil(size/8) per addressed span."""
import argparse
import csv
from pathlib import Path

from baseline import emit, traffic
from shortcut import read_records, reconstruct, runs
from whole import reconstruct as whole


class Ledger:
    def __init__(self):
        self.stages = [[0, 0, 0, 0], [0, 0, 0, 0]]
        self.current = self.stages[0]
    def at(self, addr, size, write=False, group=None):
        self.current[int(write)] += size
        self.current[2+int(write)] += (size+7)//8
    def next_phase(self):
        self.current = self.stages[1]


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("columns", type=Path)
    ap.add_argument("records", type=Path)
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    with args.output.open("w") as out:
        w = csv.writer(out, lineterminator="\n")
        w.writerow(("block", "arm", "phase", "read_bytes", "write_bytes", "q8_reads", "q8_writes"))
        for path in sorted(args.records.glob("*.irec*")):
            rec = traffic.irec.load(str(path))
            records = read_records(args.columns / f"{rec.block:04}.runs")
            assert list(runs(records)) == list(traffic.runs(rec.a["l_byte"], rec.a["l_lam"]))
            for mode in ("eager", "lazy", "shortcut", "whole"):
                m = Ledger()
                if mode in ("eager", "lazy"):
                    emit(rec, mode == "lazy", m)
                    names = "setup_induce", "writer"
                else:
                    run = whole if mode == "whole" else reconstruct
                    run(records, rec.n, rec.p, at=m.at, begin_walk=m.next_phase)
                    names = "setup", "output_and_learning"
                for phase, values in zip(names, m.stages):
                    w.writerow((rec.block, mode, phase, *values))
                out.flush()
