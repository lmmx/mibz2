"""Replay exact measured per-thread precedence and residual delays; keep actual job assignment."""
from collections import defaultdict
import json
from pathlib import Path
import sys
from schedule import rows, weighted_bound


def main(path, w):
    phases = defaultdict(dict)
    slot, workers, assignment = [], {}, {}
    last = {}
    for r in rows(path):
        who, kind, i, t = r["thread"], r["kind"], int(r["place"]), int(r["ns"])
        if kind == "ThreadStart":
            workers[who] = t
        if kind == "SlotGot":
            slot.append(t)
        if kind == "JobGot":
            last[who] = i
            assignment[i] = who
        if kind == "MtfEnd":
            i = last[who]
        if kind in ["Issued", "JobGot", "DecodeStart", "MtfEnd", "DecodeEnd", "Ready", "Taken"]:
            assert i not in phases[kind]
            phases[kind][i] = t
    n = len(assignment)
    assert all(len(x) == n for x in phases.values())
    assert n == 1724 and len(slot) == n+1
    finished = {}
    eligible = []
    signatures = []
    for i in range(n):
        who = assignment[i]
        previous = finished.get(who)
        avail = phases["Ready"][previous] if previous is not None else workers[who]
        issued, got, d, f, end, ready, taken = (phases[k][i] for k in
            ["Issued", "JobGot", "DecodeStart", "MtfEnd", "DecodeEnd", "Ready", "Taken"])
        assert issued <= got <= d <= f <= end <= ready <= taken
        prior_scan = phases["Issued"][i-1] if i else workers["mibz2-scan"]
        hold = phases["Taken"].get(i-4*w, 0)
        slot_delay = slot[i] - max(prior_scan, hold)
        job_delay = got - max(avail, issued)
        take_delay = taken - max(ready, phases["Taken"].get(i-1, 0))
        assert min(slot_delay, job_delay, take_delay) >= 0
        scan, front = issued-slot[i], f-d
        eligible.append(min(front, max(0,issued-max(avail,slot[i]))))
        signatures.append((who,previous,slot_delay,scan,job_delay,ready-got,min(front,scan),take_delay))
        finished[who] = i

    def replay(early):
        issues, done, taken = [], [], []
        for i, (who,prev,delay,scan,jobdelay,total,discount,takedelay) in enumerate(signatures):
            prior_issue = issues[-1] if i else workers["mibz2-scan"]
            hold = taken[i-4*w] if i>=4*w else 0
            issue = max(prior_issue,hold)+delay+scan
            worker = done[prev] if prev is not None else workers[who]
            ready = max(worker,issue - (discount if early else 0))+jobdelay+total
            take = max(ready,taken[-1] if i else 0)+takedelay
            issues.append(issue);done.append(ready);taken.append(take)
            if not early:
                assert issue == phases["Issued"][i] and ready == phases["Ready"][i] and take == phases["Taken"][i]
        return taken
    actual, ideal = replay(False), replay(True)
    gain = actual[-1]-ideal[-1]
    bound = weighted_bound([x[6] for x in signatures],4*w)
    assert 0 <= gain <= bound
    payloads = [int(r["len"])//2 for r in rows(path) if r["kind"]=="Issued"]
    output = dict(workers=w,slots=4*w,blocks=n,thread_received={k:list(assignment.values()).count(k)
                   for k in sorted(set(assignment.values()))}, mtf_symbols=sum(payloads),
                  first_mark_to_last_taken_ms=actual[-1]/1e6,
                  independent_front_opportunities=sum(v>0 for v in eligible),
                  original_local_opportunity_ms=sum(eligible)/1e6,
                  scan_interval_sum_ms=sum(x[3] for x in signatures)/1e6,
                  front_cap_sum_ms=sum(x[6] for x in signatures)/1e6,
                  ideal_saved_ms=gain/1e6,ideal_fraction=gain/actual[-1],
                  no_feedback_max_ms=max(x[6] for x in signatures)/1e6,
                  packing_max_ms=bound/1e6,
                  cumulative_savings_at_copy_end_ms=[(actual[i]-ideal[i])/1e6 for i in [430,861,1292,1723]])
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main(Path(sys.argv[1]), int(sys.argv[2]))
