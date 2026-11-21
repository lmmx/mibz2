"""Part A: a written attribution for every trigger, from the per-function split (attribution.json)
and the triggers (summary.json); writes summary.json back with each trigger's `class` and
`attribution`. A trigger is attributed by where its difference lies:
- decoder: the mibz2 crate's own functions moved beyond their same-binary spread;
- harness: the `ab` binary's own functions (its sampler thread and /proc parsing; the two builds'
  harnesses differ, each the harness of its own commit), not the decoder;
- runtime: libc, the allocator and the loader (shared code either may call);
- within same-binary spread: the whole difference is no larger than the range of the same binary
  over its own 5 repeats (valgrind serialises the threads, so the time-driven sampler, the pool's
  waits and the order of first touches vary run to run). Checked first.
Each also gives the difference's cost by cachegrind's rough model (10 per L1 miss or mispredict,
100 per LL miss), as a share of the whole run's CEst."""
import json, os

here = os.path.dirname(os.path.abspath(__file__))
s = json.load(open(os.path.join(here, "summary.json")))
a = json.load(open(os.path.join(here, "attribution.json")))
W = {"Ir": 1, "I1mr": 10, "D1mr": 10, "D1mw": 10, "Bcm": 10, "Bim": 10, "ILmr": 100, "DLmr": 100, "DLmw": 100}
classes = {}
for t in s["triggers"]:
    r = a[f"{t['corpus']} {t['threads']}"]
    e = t["event"]
    tot = r[e]
    d = {g: r[g][e]["port"] - r[g][e]["v056"] for g in ("decoder", "harness", "runtime")}
    dec_aa = max(r["decoder"][e]["aa"])
    run_cest = r["decoder"]["CEst"]["v056"] + r["harness"]["CEst"]["v056"] + r["runtime"]["CEst"]["v056"]
    cost = abs(tot["delta"]) * W[e] / run_cest
    dec_cost = abs(d["decoder"]) * W[e] / r["decoder"]["CEst"]["v056"]
    dec_ir = r["decoder"]["Ir"]["port"] / r["decoder"]["Ir"]["v056"]
    aa = max(tot["aa"])
    if abs(tot["delta"]) <= aa:
        cls = "within same-binary spread"
    elif abs(d["decoder"]) > dec_aa and abs(d["decoder"]) >= 0.5 * abs(tot["delta"]):
        cls = "decoder layout"
    elif abs(d["harness"]) >= max(abs(d["runtime"]), abs(d["decoder"])):
        cls = "harness"
    else:
        cls = "runtime/scheduling"
    top = [x for x in tot["top"] if abs(x["delta"]) > 0][:3]
    dec_top = [x for x in tot["top"] if "mibz2::" in x["fn"] and not x["fn"].startswith(("ab::", "<ab::"))][:2]
    txt = (
        f"{e} x{t['ratio']:.4f} ({tot['v056']:,.0f} -> {tot['port']:,.0f}, {tot['delta']:+,.0f}; same-binary ranges "
        f"{tot['aa'][0]:,.0f} port, {tot['aa'][1]:,.0f} v0.5.6). Split: decoder {d['decoder']:+,.0f} (its same-binary range "
        f"{dec_aa:,.0f}), harness {d['harness']:+,.0f}, runtime {d['runtime']:+,.0f}. Largest by function: "
        + "; ".join(f"{x['fn'][:90]} {x['delta']:+,.0f}" for x in top)
        + f". The decoder executes the same instructions (decoder Ir x{dec_ir:.6f}). Class: {cls}. "
        + (
            f"Decoder functions moved: " + "; ".join(f"{x['fn'][:70]} {x['v056']:,.0f} -> {x['port']:,.0f}" for x in dec_top)
            + f"; same instructions, so an address/layout effect; its cost {dec_cost * 100:.4f}% of the decoder's CEst. "
            if cls == "decoder layout" else ""
        )
        + f"Cost of the whole difference by the CEst model: {cost * 100:.4f}% of the run."
    )
    t["class"], t["attribution"] = cls, txt
    t["cost_share"] = cost
    classes.setdefault(cls, []).append(t)
json.dump(s, open(os.path.join(here, "summary.json"), "w"), indent=1)
print(f"{len(s['triggers'])} triggers: " + ", ".join(f"{k} {len(v)}" for k, v in sorted(classes.items())))
for k, v in sorted(classes.items()):
    worst = max(v, key=lambda t: t["cost_share"])
    print(f"  {k}: events {sorted({t['event'] for t in v})}; the largest cost {worst['cost_share'] * 100:.4f}% ({worst['corpus']} {worst['threads']} {worst['event']})")
