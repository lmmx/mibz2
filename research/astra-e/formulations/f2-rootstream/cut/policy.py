"""Choose cuts using preexisting selector counts and widths alone."""
import csv
from fractions import Fraction
from pathlib import Path
import sys
from frontier import load

def policy(model,penalty):
    n = model.n
    r = model.rows
    terminal_width = [x["terminal_width"] for x in r]
    mass = terminal_width.copy()
    upper = [s-1 for s in model.sel]
    for v in range(n-1,0,-1):
        p = model.parent[v]
        mass[p] += mass[v]
        upper[p] += upper[v]
    upper = [min(m,u+1) for m,u in zip(mass,upper)]
    # Every child selector opening occupies at least one root run. A local
    # selector boundary also changes the projected terminal label.
    floor = [max(x["S"],x["incoming_pieces"]) for x in r]
    strict = [0]*n
    cap = upper.copy()
    for v in range(1,n):
        p = model.parent[v]
        floor[v] = max(floor[v],floor[p]-(mass[p]-mass[v]))
        assert floor[v] <= model.g[v]
    mindep = [0]*n
    for v in range(n-1,-1,-1):
        if terminal_width[v] == 0:
            mindep[v] = 1+min(mindep[c] for c in model.children[v])
        # Every run routed past the terminal must visit a child and reach a
        # terminal. The aggregate child lower bound is independent of this one.
        if model.children[v]:
            hops = 1+min(mindep[c] for c in model.children[v])
            strict[v] = max(strict[v],max(0,floor[v]-terminal_width[v])*hops)
        if v:
            p = model.parent[v]
            cap[p] += cap[v]
            strict[p] += floor[v]+strict[v]
    # Choosing u saves sum_{proper desc} G - sum_{subtree} R, or more for
    # a root cut. Exclude root since predicting that special option needs no cut.
    score = [0]*n
    cut = [False]*n
    p,q = penalty.numerator,penalty.denominator
    for v in range(n-1,0,-1):
        guaranteed = q*(strict[v]-cap[v])-p*cap[v]
        if guaranteed > score[v]:
            cut[v] = True
            score[v] = guaranteed
        score[model.parent[v]] += score[v]
    stops = set()
    stack = [0]
    while stack:
        v = stack.pop()
        if cut[v]:
            stops.add(v)
        else:
            stack.extend(model.children[v])
    return stops,score[0]/q

if __name__ == "__main__":
    models = load(sys.argv[1])
    w = csv.writer(sys.stdout,lineterminator="\n")
    w.writerow(["block","penalty","cuts","extra_pieces","work","work_saved","certified_penalized_saving"])
    for b,m in models.items():
        for penalty in (Fraction(0),Fraction(1,2),Fraction(1),Fraction(2),Fraction(4)):
            stops,cert = policy(m,penalty)
            parts = m.count(stops)
            work = sum(parts[:3])
            saving = sum(m.g)-work
            assert saving-penalty*parts[3] >= cert
            w.writerow([b,penalty,len(stops),parts[3],work,saving,cert])
