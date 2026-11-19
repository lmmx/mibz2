"""Preregistered hindsight, repeated-oracle and held-out stream policy prices."""
import csv
import lzma
from collections import defaultdict
from fractions import Fraction
from pathlib import Path
from statistics import median
import sys

root = Path(__file__).resolve().parent
src = Path(sys.argv[1]) if len(sys.argv) > 1 else root
dst = Path(sys.argv[2]) if len(sys.argv) > 2 else root
arms = ("writer","popcnt")
path = src / "observations.csv"
raw = defaultdict(list)
positions = defaultdict(list)
paid = {}
geom = {}
with (path.open() if path.exists() else lzma.open(str(path)+".xz","rt")) as f:
    for row in csv.DictReader(f):
        r,b,p = (int(row[k]) for k in ("round","block","position"))
        arm = row["arm"]
        g = "A" if (r-1)%4 < 2 else "B"
        shape = tuple(int(row[k]) for k in ("stream","rows","caps","quotes","output"))
        assert b not in geom or geom[b] == shape
        geom[b] = shape
        assert p == (arms.index(arm)-r-b)%2
        assert int(row["total_ns"]) == sum(int(row[k+"_ns"]) for k in ("front","inverse","rle","crc"))
        assert (r,b,arm) not in paid
        paid[r,b,arm] = int(row["total_ns"])
        raw[g,b,arm].append(paid[r,b,arm])
        positions[g,b,arm].append(p)
blocks = sorted(geom)
assert len(blocks)==228
for key,times in raw.items():
    assert len(times)==12 and sum(positions[key])==6
costs = {}
for b in blocks:
    for arm in arms:
        costs["all",b,arm] = median(raw["A",b,arm]+raw["B",b,arm])
        for group in ("A","B"):
            costs[group,b,arm] = median(raw[group,b,arm])

prices = []
def best_single(group, ids):
    return min(arms,key=lambda arm: sum(costs[group,b,arm] for b in ids))
def winner(group,b):
    return min(arms,key=lambda arm:costs[group,b,arm])
def record(group,label,ids,pick):
    arm_cost={arm:sum(costs[group,b,arm] for b in ids) for arm in arms}
    ms = sum(costs[group,b,pick(b)] for b in ids) / 1e6
    prices.append(dict(group=group,policy=label,blocks=len(ids),popcnt_blocks=sum(pick(b)=="popcnt" for b in ids),
                       worker_ms=round(ms,4),vs_writer=round(ms/(arm_cost["writer"]/1e6),6),
                       vs_popcnt=round(ms/(arm_cost["popcnt"]/1e6),6),vs_best_fixed=round(ms/(min(arm_cost.values())/1e6),6)))
for group in ("all","A","B"):
    for arm in arms:
        record(group,"always_"+arm,blocks,lambda b,arm=arm:arm)
    record(group,"hindsight",blocks,lambda b:winner(group,b))
for form,eval_group in [("A","B"),("B","A")]:
    record(eval_group,"choose_block_on_"+form,blocks,lambda b:winner(form,b))
train=[b for b in blocks if geom[b][0]%3 != 0]
test=[b for b in blocks if geom[b][0]%3 == 0]
density={b:Fraction(geom[b][3],geom[b][1]) for b in blocks}
# Exact rational cuts: lower region includes equality. The initialization enforces the
# tie preference for unsplit; ascending cuts, then writer-low, give the remaining order.
single=best_single("A",train)
chosen=lambda b:single
best=sum(costs["A",b,single] for b in train)
decision="always_"+single
for cutoff in sorted({density[b] for b in train})[:-1]:
    for low,high in (arms,arms[::-1]):
        fn=lambda b,cutoff=cutoff,low=low,high=high: low if density[b] <= cutoff else high
        price=sum(costs["A",b,fn(b)] for b in train)
        if price<best:
            chosen,best=fn,price
            decision=f"quote_count/rows <= {cutoff}: {low}, otherwise {high}"
record("A","training_gate",train,chosen)
for arm in arms:
    record("B","heldout_always_"+arm,test,lambda b,arm=arm:arm)
record("B","heldout_gate",test,chosen)
record("B","heldout_hindsight",test,lambda b:winner("B",b))
record("B","heldout_repeat_oracle_A",test,lambda b:winner("A",b))
for s in [0,3,6,9]:
    record("B","heldout_gate_stream_"+str(s),[b for b in test if geom[b][0]==s],chosen)
with (dst/"prices.csv").open("w") as f:
    w=csv.DictWriter(f,fieldnames=list(prices[0]),lineterminator="\n")
    w.writeheader();w.writerows(prices)
with (dst/"blocks.csv").open("w") as f:
    w=csv.writer(f,lineterminator="\n")
    pairs = [(g,arm) for g in ("all","A","B") for arm in arms]
    w.writerow(["block","stream","rows","caps","quotes","output","holdout","gate",*(g+"_"+arm+"_ns" for g,arm in pairs)])
    for b in blocks:
        w.writerow([b,*geom[b],int(b in test),chosen(b),*(costs[g,b,arm] for g,arm in pairs)])
with (src/"capacity.csv").open() as f:
    caps=[(int(r["writer_vectors"]),int(r["popcnt_vectors"])) for r in csv.DictReader(f)]
assert len(caps)==228
(dst/"selection.txt").write_text(f"train n={len(train)} heldout n={len(test)}; {decision}\n"
                              f"within-block winner agrees between A/B on {sum(winner('A',b)==winner('B',b) for b in blocks)} of 228\n"
                              f"peak observed retained vectors in bytes: writer {max(v[0] for v in caps)} popcnt {max(v[1] for v in caps)} both {max(sum(v) for v in caps)}\n")

# Secondary sensitivity: price the independently chosen label with the full observed
# durations rather than using medians to suppress outlying host delays.
with (dst/"paid-rotations.csv").open("w") as f:
    w=csv.writer(f,lineterminator="\n")
    w.writerow(["evaluation_round","policy","blocks","selected_ms","writer_ms","popcnt_ms","ratio_best_fixed"])
    for r in range(1,25):
        form="B" if (r-1)%4<2 else "A"
        policies=[("repeat_oracle_"+form,blocks,lambda b:winner(form,b))]
        if form=="A": policies.append(("heldout_gate",test,chosen))
        for policy,ids,choose in policies:
            selected=sum(paid[r,b,choose(b)] for b in ids)
            fixed=[sum(paid[r,b,a] for b in ids) for a in arms]
            w.writerow([r,policy,len(ids),*(round(x/1e6,4) for x in (selected,*fixed)),round(selected/min(fixed),6)])
