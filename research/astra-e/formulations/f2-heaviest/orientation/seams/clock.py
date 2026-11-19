"""Fit an implementable common direction clock by length-grouped prefix/suffix charges."""
from collections import defaultdict
from itertools import combinations, product
from pathlib import Path
import argparse
import csv
import json
import numpy as np

from cut import Column, Direct, HERE, heavy_set, price


def prices(words):
    tables = []
    for reverse in (False, True):
        maxima = {}
        for word in words:
            s = word[::-1] if reverse else word
            for i in range(len(s)):
                if len(s)>maxima.get(s[:i],0):
                    maxima[s[:i]] = len(s)
        table = defaultdict(list)
        for key, length in maxima.items():
            table[length].append(len(key))
        tables.append(table)
    return tables


def train(words):
    forward, backward = prices(words)
    parents = [np.zeros(1, dtype=bool)]
    prev = np.zeros(1, dtype=np.int64)
    for n in range(1, max(map(len, words)) + 1):
        left = np.r_[np.int64(2**50), prev]
        right = np.r_[prev, np.int64(2**50)]
        choose_forward = left < right
        f = np.r_[0, np.cumsum(np.bincount(forward[n], minlength=n))]
        b = np.r_[0, np.cumsum(np.bincount(backward[n], minlength=n))]
        prev = f + b[::-1] + np.minimum(left, right)
        parents.append(choose_forward)
    t = int(prev.argmin())
    best = int(prev[t])
    ticks = []
    for choices in reversed(parents[1:]):
        f = int(choices[t])
        ticks.append(f)
        t -= f
    assert t == 0
    ticks.reverse()
    assert best == fixed_price(words, ticks)
    return ticks, best


def fixed_price(words, ticks):
    n = max(map(len, words))
    schedule = list(ticks[:n])
    schedule.extend(1-i%2 for i in range(n-len(schedule)))
    sums = np.r_[0, np.cumsum(schedule)]
    return price(words, [int(sums[len(w)]) for w in words])


def small():
    ws = [bytes(x) for n in (1,2,3) for x in product(range(2),repeat=n)]
    cases = 0
    for n in (1,2,3):
        for words in combinations(ws,n):
            _, observed = train(words)
            assert observed == min(fixed_price(words,clock) for clock in product((0,1),repeat=max(map(len,words))))
            cases += 1
    print(f"{cases} exact clock optimizations matched every feasible direction schedule")


def real(tapes, fit):
    import sys
    sys.path.insert(0, str(HERE.parents[5]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, read_input
    frozen = {} if fit else json.loads((HERE/"clock.json").read_text())
    prior = {(int(r["ordinal"]),r["section"]):int(r["queries"]) for r in csv.DictReader((HERE/"wikidata.csv").open())}
    writer = None
    for tape in paths(tapes):
        idx = int(tape.name.split(".")[0])
        if fit and idx != 0:continue
        col = Column(read_input(tape)[0])
        p = col.interval(b'"')
        for section, bounds in [("quote", [p]), ("frequent", heavy_set(col,p,1)[0])]:
            words = list(Direct(col,bounds).word_counts)
            if fit:
                ticks, optimum = train(words)
                frozen[section] = dict(clock="".join(map(str,ticks)),queries=optimum)
                continue
            clock = list(map(int,frozen[section]["clock"]))
            paid = fixed_price(words,clock)
            assert idx != 0 or paid == frozen[section]["queries"]
            backward = price(words,[0]*len(words))
            half = fixed_price(words,[])
            oracle = prior[idx,section]
            assert oracle <= min(paid, half)
            row = dict(ordinal=idx,section=section,words=len(words),max_length=max(map(len,words)),
                       backward=backward,forward=price(words,list(map(len,words))),alternating=half,
                       frozen_clock=paid,unrestricted=oracle,
                       both_screen=100*paid<=90*backward and 100*paid<=95*half)
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()
    if fit:
        print(json.dumps(frozen,indent=2))


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("tapes",nargs="?",type=Path)
    p.add_argument("--fit",action="store_true")
    args=p.parse_args()
    real(args.tapes,args.fit) if args.tapes else small()
