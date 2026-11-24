"""The service core's target preflight record (`examples/svc_target.rs`, schema
`mibz2.svc_target/2`), checked as a whole: not only its `ok` bit. Used by the preflight-only
entry (`just bench-svc-target`, docs/journal/2026-11-09-svc-target-registration.md) and by
the timed A/B's judge (bench/service_guarded.py).

The record is read strictly (exact schema, every field present and of its type, booleans
never numbers, the three budgets 9, 10, 11 in order) and the predicate is recomputed from the
record's own facts, independently of the binary's answer:

    target = kernels AVX2 and CLMUL, a topology snapshot read (not null) of 10 physical
             cores, 20 logical CPUs, a 20 MiB L3, GenuineIntel family 6 model 165, every
             CPU named "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz" (the measured SKU), and
             an established quota of none (unlimited; any limit, or unknown, is not)

Schema 2 adds the processor's name (`machine.target.cpu.name`) and narrows the quota to
unlimited, for the target-gated registration
(docs/journal/2026-11-10-service-guarded-target-gated-registration.md); a schema 1 record
(the earlier attempt's) is not this predicate's and is refused as malformed.

Verdicts:
- TARGET: the facts are the target's; the record says ok with no reasons; the predicate holds
  at 10 and not at 9 or 11; one decoder built at each; svc2 at 10 (8 decoding threads, window
  8, every hub thread ended, as many blocks finished as prepared, at least one); not the
  service core at 9 or 11; every decode gave back the input; the exit status 0.
- NOT TARGET: the facts are not the target's (an unknown quota or an unread snapshot among
  them: fail-closed), the record says not ok, with reasons, and is otherwise consistent and
  clean: the predicate false at every budget, the service core built nowhere, one decoder
  built at each budget and every decode correct; the exit status 3.
- FAILED: a dispatch or decode check failed (the record says not ok; the exit status 3):
  on the target's facts, svc2 not as expected at 10; anywhere, a budget with other than one
  decoder built, or a decode that did not give back its input. This takes precedence over
  NOT TARGET: a failed fallback decode off the target is FAILED, never hidden under it.
- INVALID RECORD: anything else: a malformed record, a missing fact, an exit status that
  disagrees, an `ok` that disagrees with the recomputation (ok=true beside an unknown quota or
  an absent snapshot), a predicate answer that disagrees with the facts, the service core
  built where the predicate is false.

    python3 bench/svc_target_check.py PREFLIGHT.json --exit STATUS [--out CHECK.json]
"""

import argparse
import json
import math
import sys

SCHEMA = "mibz2.svc_target/2"
BUDGETS = (9, 10, 11)
TARGET = {"phys": 10, "logical": 20, "l3": 20 << 20}
CPU = ("GenuineIntel", 6, 165)
NAME = "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz"
SVC2 = {"teams": 2, "k": 3, "services": 2}


def is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def is_bool(x):
    return isinstance(x, bool)


def pos_int(x):
    return is_int(x) and x > 0


def nonneg_int(x):
    return is_int(x) and x >= 0


def finite_pos(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def plan_ok(plan):
    """A complete, typed `Plan`: {"kind": "workers", "workers": n} or {"kind": "teams",
    "teams": n, "k": k}, each a positive integer, nothing else"""
    if not isinstance(plan, dict):
        return False
    if plan.get("kind") == "workers":
        return set(plan) == {"kind", "workers"} and pos_int(plan["workers"])
    if plan.get("kind") == "teams":
        return set(plan) == {"kind", "teams", "k"} and pos_int(plan["teams"]) and pos_int(plan["k"])
    return False


def built_ok(b):
    """A decoder built (`parallel::Built`): typed, positive threads and window, its counts (if
    any) nonnegative integers with `ended` a boolean"""
    if not (
        isinstance(b, dict)
        and set(b) == {"service", "pipelined", "threads", "window", "counts"}
        and is_bool(b["service"])
        and is_bool(b["pipelined"])
        and pos_int(b["threads"])
        and pos_int(b["window"])
    ):
        return False
    c = b["counts"]
    return c is None or (
        isinstance(c, dict)
        and set(c) == {"prepared", "pooled", "finished", "ended"}
        and all(nonneg_int(c[f]) for f in ("prepared", "pooled", "finished"))
        and is_bool(c["ended"])
    )


def shape_problems(r):
    """What is malformed or missing in the record (empty: well formed). Strict: every field
    present and of its type, integers never floats or booleans, booleans never numbers,
    numbers finite, sizes positive, counts nonnegative"""
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
    if not (isinstance(k, dict) and set(k) == {"avx2", "clmul"} and is_bool(k["avx2"]) and is_bool(k["clmul"])):
        p.append("kernels missing or malformed")
    m = r.get("machine")
    if not isinstance(m, dict):
        return p + ["machine missing"]
    for f in ("phys", "l3"):
        if f not in m or not (m[f] is None or pos_int(m[f])):
            p.append(f"machine.{f} missing or malformed")
    t = m.get("target", "missing")
    if t == "missing":
        p.append("machine.target missing")
    elif t is not None:
        cpu = t.get("cpu") if isinstance(t, dict) else None
        if not (
            isinstance(t, dict)
            and set(t) == {"phys", "logical", "l3", "cpu"}
            and all(pos_int(t[f]) for f in ("phys", "logical", "l3"))
            and isinstance(cpu, dict)
            and set(cpu) == {"vendor", "family", "model", "name"}
            and isinstance(cpu["vendor"], str)
            and isinstance(cpu["name"], str)
            and nonneg_int(cpu["family"])
            and nonneg_int(cpu["model"])
        ):
            p.append("machine.target malformed")
    q = m.get("quota")
    if not (isinstance(q, dict) and q.get("kind") in ("unlimited", "cpus", "unknown")):
        p.append("machine.quota missing or malformed")
    elif q["kind"] == "cpus":
        if not (set(q) == {"kind", "cpus"} and finite_pos(q["cpus"])):
            p.append("machine.quota.cpus not a finite positive number")
    elif set(q) != {"kind"}:
        p.append("machine.quota has fields its kind doesn't")
    sp = r.get("service_plan")
    if not (isinstance(sp, dict) and set(sp) == set(SVC2) and all(is_int(sp[f]) and sp[f] == v for f, v in SVC2.items())):
        p.append(f"service_plan {sp!r}, not svc2's integers")
    if not pos_int(r.get("input_bytes")):
        p.append("input_bytes missing or malformed")
    at = r.get("at")
    if not (
        isinstance(at, list)
        and len(at) == len(BUDGETS)
        and all(isinstance(a, dict) and is_int(a.get("threads")) and a["threads"] == t for a, t in zip(at, BUDGETS))
    ):
        return p + ["at: not the integer budgets 9, 10, 11 in order"]
    for a in at:
        t_ = a["threads"]
        if set(a) != {"threads", "predicate", "plan", "builds", "built", "decoded"}:
            p.append(f"at {t_}: fields {sorted(a)}")
            continue
        if not (is_bool(a["predicate"]) and is_bool(a["decoded"]) and nonneg_int(a["builds"])):
            p.append(f"at {t_}: predicate, decoded or builds malformed")
        if not plan_ok(a["plan"]):
            p.append(f"at {t_}: plan incomplete or malformed")
        if a["built"] is not None and not built_ok(a["built"]):
            p.append(f"at {t_}: built malformed")
    return p


def facts_are_target(r):
    """The predicate, recomputed from the record's facts (fail-closed: unknown is not target)"""
    k, m = r["kernels"], r["machine"]
    t, q = m["target"], m["quota"]
    quota_ok = q["kind"] == "unlimited"
    return bool(
        k["avx2"]
        and k["clmul"]
        and t is not None
        and all(t[f] == v for f, v in TARGET.items())
        and (t["cpu"]["vendor"], t["cpu"]["family"], t["cpu"]["model"]) == CPU
        and t["cpu"]["name"] == NAME
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
    # (precedence, explicit: a dispatch or decode failure anywhere, at the fallback budgets
    # or off the target too, is FAILED, never hidden under NOT TARGET)
    if dispatch:
        if r["ok"]:
            return "INVALID RECORD", ["ok, but " + "; ".join(dispatch)]
        return "FAILED", dispatch
    if target:
        if not r["ok"]:
            return "INVALID RECORD", ["the facts and dispatch are the target's, but not ok"]
        return "TARGET", []
    if r["ok"]:
        return "INVALID RECORD", ["ok, but the facts are not the target's"]
    return "NOT TARGET", []


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
