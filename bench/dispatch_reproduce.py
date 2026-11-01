"""Reproduce, from the attribution archive's ten four-worker full traces (read only), the
scheduler accounting of external report 106 and the conditional claim-delay table of
report 105 (docs/journal/2026-11-01-dispatch-causality-protocol.md §1):

    dispatch_reproduce.py MARKS_DIR      # .../occupancy/wikidata-20261026/marks (r??-t4-full.tsv[.gz])

- the run-queue wait (schedstat `wait_ns`) over every inter-decode gap (DecodeEnd to the
  worker's next DecodeStart), pooled;
- the zero-backlog observations: a job enqueued while three workers decode and no older
  job is unclaimed, by the fourth worker's last mark (JobLock or JobRecv); who claims it;
- the distinct inter-decode gaps of the fourth worker that contain a zero-backlog JobLock
  observation, and their pooled run-queue wait;
- the median claim delay (Enqueued to JobGot) by the number of workers decoding at enqueue.
Encompassing-gap figures are not fractions of individual claim delays.
"""
import collections
import glob
import gzip
import json
import statistics
import sys


def rows(path):
    op = gzip.open if path.endswith(".gz") else open
    with op(path, "rt") as f:
        next(f)
        for l in f:
            r = l.rstrip("\n").split("\t")
            yield (int(r[2]), r[1], r[3], None if r[4] == "-" else int(r[4]), None if r[7] == "-" else int(r[7]))


def main(d):
    files = sorted(glob.glob(f"{d}/r??-t4-full.tsv*"))
    gaps_el = gaps_wait = 0
    obs = collections.Counter()
    claimed = collections.Counter()
    gaps = {}
    delay = collections.defaultdict(list)
    for f in files:
        ev = sorted(rows(f))
        workers = sorted({e[1] for e in ev if e[1].startswith("mibz2-") and e[1][6:].isdigit()})
        gl = {}
        for w in workers:
            ws = [e for e in ev if e[1] == w and e[2] in ("DecodeStart", "DecodeEnd")]
            gl[w] = [(a[0], b[0], b[0] - a[0], b[4] - a[4]) for a, b in zip(ws, ws[1:]) if a[2] == "DecodeEnd" and b[2] == "DecodeStart"]
            gaps_el += sum(g[2] for g in gl[w])
            gaps_wait += sum(g[3] for g in gl[w])
        got = {e[3]: (e[0], e[1]) for e in ev if e[2] == "JobGot"}
        active, last, enq = set(), {}, {}
        for ns, name, kind, place, _ in ev:
            if name in workers:
                if kind == "DecodeStart":
                    active.add(name)
                elif kind == "DecodeEnd":
                    active.discard(name)
                last[name] = kind
            if kind != "Enqueued":
                continue
            if place in got:
                delay[len(active)].append((got[place][0] - ns) / 1e3)
            older = [p for p in enq if p < place and (p not in got or got[p][0] > ns)]
            enq[place] = ns
            if len(active) != 3 or older:
                continue
            idle = next(w for w in workers if w not in active)
            st = last.get(idle)
            obs[st] += 1
            if got.get(place, (0, None))[1] == idle:
                claimed[st] += 1
            if st == "JobLock":
                g = next((x for x in gl[idle] if x[0] <= ns <= x[1]), None)
                if g:
                    gaps[(f, idle, g[0])] = g
    n = sum(len(v) for v in delay.values())
    out = dict(
        files=len(files),
        inter_decode=dict(elapsed_s=gaps_el / 1e9, runq_wait_s=gaps_wait / 1e9, share=gaps_wait / gaps_el),
        zero_backlog_observations=dict(obs), claimed_by_the_nondecoding_worker=dict(claimed),
        zero_backlog_joblock_gaps=dict(n=len(gaps), elapsed_s=sum(g[2] for g in gaps.values()) / 1e9,
                                       runq_wait_s=sum(g[3] for g in gaps.values()) / 1e9,
                                       share=sum(g[3] for g in gaps.values()) / sum(g[2] for g in gaps.values())),
        claim_delay_by_decoding_at_enqueue={k: dict(share=len(v) / n, median_us=statistics.median(v)) for k, v in sorted(delay.items())})
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main(sys.argv[1])
