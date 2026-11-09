"""The service core's target preflight record (`examples/svc_target.rs`, schema
`mibz2.svc_target/1`), checked as a whole: not only its `ok` bit. Used by the preflight-only
entry (`just bench-svc-target`, docs/journal/2026-11-09-svc-target-registration.md) and by
the timed A/B's judge (bench/service_guarded.py).

The record is read strictly (exact schema, every field present and of its type, booleans
never numbers, the three budgets 9, 10, 11 in order) and the predicate is recomputed from the
record's own facts, independently of the binary's answer:

    target = kernels AVX2 and CLMUL, a topology snapshot read (not null) of 10 physical
             cores, 20 logical CPUs, a 20 MiB L3, GenuineIntel family 6 model 165, and an
             established quota of none or at least 10 CPUs (not unknown)

Verdicts:
- TARGET: the facts are the target's; the record says ok with no reasons; the predicate holds
  at 10 and not at 9 or 11; one decoder built at each; svc2 at 10 (8 decoding threads, window
  8, every hub thread ended, as many blocks finished as prepared, at least one); not the
  service core at 9 or 11; every decode gave back the input; the exit status 0.
- NOT TARGET: the facts are not the target's (an unknown quota or an unread snapshot among
  them: fail-closed), the record says not ok, with reasons, and is otherwise consistent: the
  predicate false at every budget and the service core built nowhere; the exit status 3.
- FAILED: the facts are the target's but a dispatch or decode check failed (the record says
  not ok); the exit status 3.
- INVALID RECORD: anything else: a malformed record, a missing fact, an exit status that
  disagrees, an `ok` that disagrees with the recomputation (ok=true beside an unknown quota or
  an absent snapshot), a predicate answer that disagrees with the facts, the service core
  built where the predicate is false.

    python3 bench/svc_target_check.py PREFLIGHT.json --exit STATUS [--out CHECK.json]
"""

import argparse
import json
import sys

SCHEMA = "mibz2.svc_target/1"
BUDGETS = (9, 10, 11)
TARGET = {"phys": 10, "logical": 20, "l3": 20 << 20}
CPU = ("GenuineIntel", 6, 165)
SVC2 = {"teams": 2, "k": 3, "services": 2}


def is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def is_bool(x):
    return isinstance(x, bool)


def shape_problems(r):
    """What is malformed or missing in the record (empty: well formed)"""
    p = []
    if not isinstance(r, dict):
        return ["not a JSON object"]
    if r.get("schema") != SCHEMA:
        p.append(f"schema {r.get('schema')!r}, not {SCHEMA}")
    if not is_bool(r.get("ok")):
        p.append("ok not a boolean")
    if not (isinstance(r.get("why"), list) and all(isinstance(w, str) for w in r["why"])):
        p.append("why not a list of strings")
    k = r.get("kernels")
    if not (isinstance(k, dict) and is_bool(k.get("avx2")) and is_bool(k.get("clmul"))):
        p.append("kernels missing or malformed")
    m = r.get("machine")
    if not isinstance(m, dict):
        return p + ["machine missing"]
    for f in ("phys", "l3"):
        if f not in m or not (m[f] is None or is_int(m[f])):
            p.append(f"machine.{f} missing or malformed")
    t = m.get("target", "missing")
    if t == "missing":
        p.append("machine.target missing")
    elif t is not None:
        cpu = t.get("cpu") if isinstance(t, dict) else None
        if not (
            isinstance(t, dict)
            and all(is_int(t.get(f)) for f in ("phys", "logical", "l3"))
            and isinstance(cpu, dict)
            and isinstance(cpu.get("vendor"), str)
            and is_int(cpu.get("family"))
            and is_int(cpu.get("model"))
        ):
            p.append("machine.target malformed")
    q = m.get("quota")
    if not (isinstance(q, dict) and q.get("kind") in ("unlimited", "cpus", "unknown")):
        p.append("machine.quota missing or malformed")
    elif q["kind"] == "cpus" and not (
        isinstance(q.get("cpus"), (int, float)) and not is_bool(q.get("cpus")) and q["cpus"] > 0
    ):
        p.append("machine.quota.cpus malformed")
    if r.get("service_plan") != SVC2:
        p.append(f"service_plan {r.get('service_plan')!r}, not svc2's")
    if not (is_int(r.get("input_bytes")) and r["input_bytes"] > 0):
        p.append("input_bytes missing or malformed")
    at = r.get("at")
    if not (isinstance(at, list) and [a.get("threads") if isinstance(a, dict) else None for a in at] == list(BUDGETS)):
        return p + ["at: not the budgets 9, 10, 11 in order"]
    for a in at:
        t_ = a["threads"]
        if not (is_bool(a.get("predicate")) and is_bool(a.get("decoded")) and is_int(a.get("builds"))):
            p.append(f"at {t_}: predicate, decoded or builds malformed")
        plan = a.get("plan")
        if not (isinstance(plan, dict) and plan.get("kind") in ("workers", "teams")):
            p.append(f"at {t_}: plan malformed")
        b = a.get("built", "missing")
        if b == "missing":
            p.append(f"at {t_}: built missing")
        elif b is not None:
            c = b.get("counts", "missing") if isinstance(b, dict) else "missing"
            if not (
                isinstance(b, dict)
                and is_bool(b.get("service"))
                and is_bool(b.get("pipelined"))
                and is_int(b.get("threads"))
                and is_int(b.get("window"))
                and (
                    c is None
                    or (
                        isinstance(c, dict)
                        and all(is_int(c.get(f)) for f in ("prepared", "pooled", "finished"))
                        and is_bool(c.get("ended"))
                    )
                )
            ):
                p.append(f"at {t_}: built malformed")
    return p


def facts_are_target(r):
    """The predicate, recomputed from the record's facts (fail-closed: unknown is not target)"""
    k, m = r["kernels"], r["machine"]
    t, q = m["target"], m["quota"]
    quota_ok = q["kind"] == "unlimited" or (q["kind"] == "cpus" and q["cpus"] >= 10)
    return bool(
        k["avx2"]
        and k["clmul"]
        and t is not None
        and all(t[f] == v for f, v in TARGET.items())
        and (t["cpu"]["vendor"], t["cpu"]["family"], t["cpu"]["model"]) == CPU
        and quota_ok
    )


def check(r, exit_status):
    """(verdict, problems): see the module's doc"""
    p = shape_problems(r)
    if not is_int(exit_status):
        p.append("no exit status")
    if p:
        return "INVALID RECORD", p
    target = facts_are_target(r)
    at = {a["threads"]: a for a in r["at"]}
    if r["ok"] != (exit_status == 0) or exit_status not in (0, 3):
        p.append(f"exit status {exit_status} disagrees with ok={r['ok']}")
    if r["ok"] and r["why"]:
        p.append("ok with reasons")
    if not r["ok"] and not r["why"]:
        p.append("not ok, with no reason")
    for t in BUDGETS:
        if at[t]["predicate"] != (target and t == 10):
            p.append(f"at {t}: the predicate's answer {at[t]['predicate']} disagrees with the facts")
        b = at[t]["built"]
        if b is not None and b["service"] and not (target and t == 10):
            p.append(f"at {t}: the service core built where the predicate is false")
    dispatch = []
    for t in BUDGETS:
        a, b = at[t], at[t]["built"]
        if a["builds"] != 1 or b is None:
            dispatch.append(f"at {t}: {a['builds']} decoders built")
        if not a["decoded"]:
            dispatch.append(f"at {t}: the decode did not give back the input")
    b = at[10]["built"]
    if target and not (
        b is not None
        and b["service"]
        and not b["pipelined"]
        and b["threads"] == 8
        and b["window"] == 8
        and b["counts"] is not None
        and b["counts"]["ended"]
        and b["counts"]["finished"] == b["counts"]["prepared"] > 0
    ):
        dispatch.append("at 10: not svc2 at 8 threads, window 8, completed")
    if p:
        return "INVALID RECORD", p
    if target and not dispatch:
        if not r["ok"]:
            return "INVALID RECORD", ["the facts and dispatch are the target's, but not ok"]
        return "TARGET", []
    if r["ok"]:
        return "INVALID RECORD", ["ok, but " + ("; ".join(dispatch) if target else "the facts are not the target's")]
    if target:
        return "FAILED", dispatch
    return "NOT TARGET", dispatch


def main(argv=None):
    a = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    a.add_argument("record")
    a.add_argument("--exit", type=int, required=True)
    a.add_argument("--out")
    args = a.parse_args(argv)
    try:
        r = json.load(open(args.record))
    except (OSError, json.JSONDecodeError) as e:
        r = None
        verdict, problems = "INVALID RECORD", [f"unreadable: {e}"]
    else:
        verdict, problems = check(r, args.exit)
    out = {"verdict": verdict, "problems": problems, "exit": args.exit}
    if args.out:
        with open(args.out, "w") as f:
            json.dump(out, f, indent=1, sort_keys=True)
    print(json.dumps(out))
    return {"TARGET": 0, "NOT TARGET": 4, "FAILED": 5}.get(verdict, 1)


if __name__ == "__main__":
    sys.exit(main())
