"""Compare FIFO schedules with 1, 8 or 64 consecutive same-side activations."""
from pathlib import Path
import csv
import sys

from buckets import MassBuckets
from schedule import Column, Direct, HERE, grow, heavy_set

sys.path.insert(0, str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input, paths

reference = {(int(r["ordinal"]),r["section"]):r for r in csv.DictReader((HERE/"wikidata.csv").open())}
out = None
for tape in paths(Path(sys.argv[1])):
    ordinal = int(tape.name.split(".")[0])
    col = Column(read_input(tape)[0])
    parent = col.interval(b'"')
    for name, selection in (("quote",[parent]), ("frequent",heavy_set(col,parent,1)[0])):
        direct=Direct(col,selection)
        counts={w:direct.word_counts[w] for w in sorted(direct.word_counts)}
        ref=reference[ordinal,name]
        for batch in (1,8,64):
            q=MassBuckets(sum(counts.values()),batch)
            steps,_,pops,seams=grow(counts,False,q)
            assert steps>=int(ref["optimal"])
            row=dict(ordinal=ordinal, section=name, batch=batch, states=steps, pops=pops,
                     demotions=q.demotions, side_changes=q.side_changes, interior=seams,
                     optimal=ref["optimal"], exact_priority=ref["width_price"])
            if out is None:
                out=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                out.writeheader()
            out.writerow(row)
            sys.stdout.flush()
