"""Synthetic `mibz2.svc_target/1` records for the tests (bench/test_svc_target*.py,
bench/test_service_guarded.py): the target's, and a 4-core machine's, as examples/svc_target.rs
would print them"""

import copy


def record(target=True):
    """(record, exit status) on the i9's facts (`target`) or on a 4-core Intel machine's"""
    if target:
        machine = {
            "phys": 10,
            "l3": 20 << 20,
            "target": {"phys": 10, "logical": 20, "l3": 20 << 20,
                       "cpu": {"vendor": "GenuineIntel", "family": 6, "model": 165}},
            "quota": {"kind": "unlimited"},
        }
    else:
        machine = {
            "phys": 4,
            "l3": 528482304,
            "target": {"phys": 4, "logical": 4, "l3": 528482304,
                       "cpu": {"vendor": "GenuineIntel", "family": 6, "model": 173}},
            "quota": {"kind": "unlimited"},
        }
    at = []
    for t in (9, 10, 11):
        if target:
            plan = {"kind": "teams", "teams": 4, "k": 2} if t == 9 else {"kind": "teams", "teams": 3, "k": 3}
            if t == 10:
                built = {"service": True, "pipelined": False, "threads": 8, "window": 8,
                         "counts": {"prepared": 5, "pooled": 0, "finished": 5, "ended": True}}
            elif t == 9:
                built = {"service": False, "pipelined": False, "threads": 8, "window": 16, "counts": None}
            else:
                built = {"service": False, "pipelined": True, "threads": 9, "window": 12, "counts": None}
        else:
            plan = {"kind": "workers", "workers": t}
            built = {"service": False, "pipelined": False, "threads": t, "window": 4 * t, "counts": None}
        at.append({"threads": t, "predicate": target and t == 10, "plan": plan, "builds": 1,
                   "built": built, "decoded": True})
    why = [] if target else ["10 threads: the predicate is false",
                             "10 threads: not svc2 at 8 threads, window 8: ..."]
    r = {"schema": "mibz2.svc_target/1", "ok": target, "why": why,
         "kernels": {"avx2": True, "clmul": True}, "machine": machine,
         "service_plan": {"teams": 2, "k": 3, "services": 2}, "input_bytes": 450000, "at": at}
    return r, 0 if target else 3


def edit(r, f):
    r = copy.deepcopy(r)
    f(r)
    return r
