"""Bounded two-budget maximum closure on the frozen suffix requirements."""
import argparse
import csv
import hashlib
import json
import lzma
import math
from collections import Counter
from pathlib import Path

import numpy as np
import scipy
from scipy.optimize import Bounds, LinearConstraint, milp
from scipy.sparse import coo_matrix

from model import application, get_input


def optimize(directory, seconds):
    data = json.loads((directory/"frozen.json").read_text())
    with lzma.open(directory/"frozen-costs.csv.xz", "rt") as f:
        costs = {bytes.fromhex(row["prefix_hex"]):(int(row["intersections"]), int(row["delta_bands"]))
                 for row in csv.DictReader(f)}
    with lzma.open(directory/"frozen-goals.jsonl.xz", "rt") as f:
        groups = [(n, tuple(bytes.fromhex(k) for k in req)) for n, req in map(json.loads, f)]
    useful = {key[:d] for _, req in groups for key in req for d in range(len(key)+1)}
    keys = sorted(useful)
    lookup = {key:i for i,key in enumerate(keys)}
    k = len(keys)
    weights = [0]*k
    tails = []
    for weight, req in groups:
        if len(req) == 1:
            weights[lookup[req[0]]] += weight
        else:
            tails.append(req)
            weights.append(weight)
    cols, rows, vals, upper = [], [], [], []
    def inequality(coeffs, maximum):
        row = len(upper)
        for i, v in coeffs:
            if v:
                rows.append(row)
                cols.append(i)
                vals.append(v)
        upper.append(maximum)
    for i, key in enumerate(keys):
        if key:
            inequality(((i,1), (lookup[key[:-1]],-1)), 0)
    for j, req in enumerate(tails,k):
        for key in req:
            inequality(((j,1), (lookup[key],-1)),0)
    q0 = data["original_Q"]
    fmax = data["cap8"]["F"]
    bmax = data["cap8"]["B"]
    inequality(((i,costs[key][0]) for i,key in enumerate(keys)),fmax)
    inequality(((i,costs[key][1]) for i,key in enumerate(keys)),bmax-1)
    size = len(weights)
    constraints = LinearConstraint(coo_matrix((vals,(rows,cols)),shape=(len(upper),size)).tocsc(),
                                   -np.inf, np.array(upper))
    objective = -np.array(weights, dtype=float)
    kwargs = dict(c=objective, bounds=Bounds(np.zeros(size), np.ones(size)), constraints=constraints)

    relaxed = milp(**kwargs, options={"threads":1})
    assert relaxed.success
    def price(enabled):
        q = q0-sum(n for n,req in groups if all(x in enabled for x in req))
        f = sum(costs[key][0] for key in enabled)
        b = 1+sum(costs[key][1] for key in enabled)
        return q,f,b
    best = {key for key in costs if len(key)<8}
    receipt = []
    def offer(name, policy):
        nonlocal best
        assert all(not key or key[:-1] in policy for key in policy)
        q,f,b = price(policy)
        eligible = f<=fmax and b<=bmax
        receipt.append(dict(method=name,Q=q,F=f,B=b,feasible=eligible))
        if eligible and (q,f,b,len(policy)) < (*price(best),len(best)):
            best = policy
    offer("constant-eight",best)
    def offers_from_values(tag, values):
        # Affine bounds + integral closure: sweep all distinct nonzero threshold
        # sets, select largest feasible set by positive geometry, then exact reprice.
        pairs = sorted(((float(v),key) for key,v in zip(keys,values) if v>1e-7), reverse=True)
        pos = f = 0
        b = 1
        enabled = set()
        while pos<len(pairs):
            end = pos+1
            threshold = pairs[pos][0]
            while end<len(pairs) and pairs[end][0]>threshold-1e-7:
                end+=1
            additions = {key[:i] for _,key in pairs[pos:end] for i in range(len(key)+1)}-enabled
            f += sum(costs[key][0] for key in additions)
            b += sum(costs[key][1] for key in additions)
            if f>fmax or b>bmax:
                break
            enabled.update(additions)
            pos = end
        offer(tag,enabled)
    offers_from_values("LP-threshold",relaxed.x)
    # The auxiliary AND variables can remain continuous with integral actions.
    # HiGHS also receives a proved feasible primal bound in objective units.
    integer = milp(**kwargs, integrality=np.r_[np.ones(k), np.zeros(size-k)],
                   options={"time_limit":seconds, "threads":1, "mip_rel_gap":0.00001,
                            "objective_bound":float(price(best)[0]-q0)})
    if integer.x is not None:
        candidate = {key for key,x in zip(keys,integer.x) if x>.5}
        offer("HiGHS-incumbent",candidate)
    dual = getattr(integer,"mip_dual_bound",None)
    lower = q0+relaxed.fun
    if dual is not None and math.isfinite(dual):
        lower=max(lower,q0+dual)
    info=dict(Q_lower_bound=int(math.ceil(lower-1e-5)), LP_Q=q0+relaxed.fun,
              solver_status=integer.message, solver_seconds_limit=seconds,
              solver_nodes=getattr(integer,"mip_node_count",None), solver_dual=dual, scipy=scipy.__version__,
              action_variables=k, implied_rewards=sum(1 for w in weights[:k] if w),
              shared_reward_variables=size-k, inequality_rows=len(upper), candidates=receipt)
    return best,info


if __name__ == "__main__":
    p=argparse.ArgumentParser()
    p.add_argument("directory",type=Path)
    p.add_argument("frozen_tape",type=Path)
    p.add_argument("--seconds",type=float,default=90)
    args=p.parse_args()
    assert args.frozen_tape.name=="0000.irec.xz"
    best,stats=optimize(args.directory,args.seconds)
    col,oracle,sec,words,_=get_input(args.frozen_tape)
    _,actual=application(col,oracle,sec,words,16,best)
    baseline=json.loads((args.directory/"frozen.json").read_text())["cap8"]
    assert actual["F"]<=baseline["F"] and actual["B"]<=baseline["B"]
    assert any(all(candidate[k]==actual[k] for k in ("Q","F","B")) for candidate in stats["candidates"])
    literal=json.dumps([key.hex() for key in sorted(best)],separators=(",",":"))+"\n"
    (args.directory/"policy.json").write_text(literal)
    stats.update(fitted=actual,depth_histogram=dict(sorted(Counter(map(len,best)).items())),
                 literal_sha256=hashlib.sha256(literal.encode()).hexdigest())
    (args.directory/"fit.json").write_text(json.dumps(stats,indent=2)+"\n")
    print(json.dumps(stats))
