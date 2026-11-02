"""Capacity-aware fillable opportunity in the i9-10900K dispatch-causality capture.

    E = (1/T) * integral of min(I(t), R(t), Q(t)) dt

over the frozen analysis's span (T = span1 - span0, W = workers * T). Read-only, descriptive.
It reuses bench/dispatch.py at the pin (48f8cda): load_marks, load_perf, align,
sched_reconstruct, Step, PHASE_OF, role_of, KV, and replicates run()'s breakpoint loop
verbatim (same breakpoints, midpoint sampling, availability gate, +-u clock guard,
unknown-state guard), so the per-run class integrals can be checked field for field against
the archived run analyses before anything new is read off the same segments.

    python3 -I capacity_E.py REPO CAPTURE_DIR OUT.json
"""
import bisect
import collections
import json
import statistics
import sys
from pathlib import Path

repo, cap, outp = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
sys.path.insert(0, str(repo / "bench"))
import dispatch as d  # noqa: E402

CPUS = [1, 2, 3, 4]


def counter(dl):
    dl.sort()
    out, c = [], 0
    for t, x in dl:
        c += x
        out.append((t, c))
    return d.Step(out, 0)


def analyse(f):
    b = f.name[:-len(".tsv.gz")]
    meta, names, tids, marks = d.load_marks(f)
    perf, parse = d.load_perf(cap / f"{b}.perf.txt.gz")
    epoch = meta["epoch_monotonic_ns"]
    al = d.align(marks, names, tids, epoch, perf)
    corr = al.get("correction_ns", 0.0)
    u = al.get("uncertainty_ns", 0.0)
    T = lambda th, ns: epoch + ns + corr  # noqa: E731
    role = {th: d.role_of(n) for th, n in names.items()}
    workers = sorted(th for th in names if role[th] == "worker")
    others = [th for th in names if role[th] != "worker"]
    tid_role = {tids[th]: role[th] for th in names}
    tid_of = {th: tids[th] for th in names}
    sched, occupant, stream = d.sched_reconstruct(perf, set(tid_of.values()), CPUS)
    st_of = {th: d.Step(list(sched.get(tid_of[th], []))) for th in names}
    occ = {c: d.Step(list(v)) for c, v in occupant.items()}

    def occupant_role(pid):
        if pid is None:
            return "unknown"
        if pid == 0:
            return "idle"
        return tid_role.get(pid, "foreign")

    # --- verbatim from dispatch.run(): phases, receiver owner, availability ---
    phase = {}
    owner_iv = []
    recvdone_before_got = {}
    jobgot = {}  # added: place -> JobGot time (for the literal Q variant)
    for th in workers:
        ms = marks[th]
        phase[th] = d.Step([(T(th, m[0]), d.PHASE_OF[m[1]]) for m in ms if m[1] in d.PHASE_OF], "start")
        last_recv = None
        for m in ms:
            if m[1] == "JobRecv":
                last_recv = T(th, m[0])
            elif m[1] == "RecvDone":
                rd = T(th, m[0])
            elif m[1] == "JobGot" and last_recv is not None:
                owner_iv.append((last_recv, T(th, m[0]), th))
                recvdone_before_got[m[2]] = rd
                jobgot[m[2]] = T(th, m[0])
                last_recv = None
    owner_iv.sort()
    scan = [th for th in names if role[th] == "scanner"][0]
    issued = {m[2]: T(scan, m[0]) for m in marks[scan] if m[1] == "Issued"}
    enq = {m[2]: T(scan, m[0]) for m in marks[scan] if m[1] == "Enqueued"}
    avail_d, bracket_d = [], []
    for p, e in enq.items():
        r = recvdone_before_got.get(p)
        if r is None:
            continue
        if r > e:
            avail_d += [(e, 1), (r, -1)]
        bracket_d += [(issued.get(p, e), 1), (min(e, r), -1)]
    avail = counter(avail_d)
    bracket = counter(bracket_d)
    # added: literal Q = issued (or enqueued) and not yet JobGot
    lit_d = []
    for p in set(issued) | set(enq):
        t0 = issued.get(p, enq.get(p))
        t1 = jobgot.get(p)
        if t1 is None or t1 <= t0:
            continue
        lit_d += [(t0, 1), (t1, -1)]
    qlit = counter(lit_d)

    allm = [T(th, m[0]) for th in names for m in marks[th]]
    ends = [T(th, m[0]) for th in names for m in marks[th] if m[1] in ("Taken", "DecodeEnd")]
    span0, span1 = min(allm), max(ends)
    span = span1 - span0
    W = len(workers) * span

    bp = {span0, span1}
    for th in workers:
        bp.update(phase[th].t)
        bp.update(st_of[th].t)
        for m in marks[th]:
            if m[1] in d.PHASE_OF:
                x = T(th, m[0])
                bp.update((x - u, x + u))
    for s in occ.values():
        bp.update(s.t)
    bp.update(avail.t)
    bp.update(bracket.t)
    bp = sorted(x for x in bp if span0 <= x <= span1)
    # added change points (literal Q, the non-workers' states) only subdivide a frozen
    # segment for the quantities that use them; the frozen classification stays evaluated at
    # the frozen segment's midpoint (its clock guard is midpoint-sampled), so it is exact
    extra = sorted(set(qlit.t) | {x for th in others for x in st_of[th].t})
    mark_times = {th: sorted(T(th, m[0]) for m in marks[th] if m[1] in d.PHASE_OF) for th in workers}
    avail_times = sorted(avail.t + bracket.t)

    # --- added: the placement label of each runnable interval (analysis/waker.py's matching) ---
    waking = collections.defaultdict(list)
    wakeup = collections.defaultdict(list)
    for t, cpu, tid, ev, rest in perf:
        if ev == "sched:sched_waking":
            kv = dict(d.KV.findall(rest))
            waking[int(kv["pid"])].append((t, cpu, tid))
        elif ev == "sched:sched_wakeup":
            kv = dict(d.KV.findall(rest))
            wakeup[int(kv["pid"])].append((t, int(kv["target_cpu"])))
    place_of = {}
    for th in workers:
        tid = tid_of[th]
        seq = sched.get(tid, [])
        wk = waking[tid]; wkt = [x[0] for x in wk]; wu = wakeup[tid]; wut = [x[0] for x in wu]
        ch = []
        for k, (t0, s0) in enumerate(seq):
            if s0 != "Q":
                ch.append((t0, None))
                continue
            t1 = seq[k + 1][0] if k + 1 < len(seq) else float("inf")
            i = bisect.bisect_left(wkt, t0)
            j = bisect.bisect_left(wut, t0)
            if i >= len(wk) or wk[i][0] != t0 or j >= len(wu) or wu[j][0] > t1:
                lab = "no_wake"  # preempted, or runnable without a matched waking/wakeup pair
            else:
                lab = "waker_cpu" if wu[j][1] == wk[i][1] else "elsewhere"
            ch.append((t0, lab))
        place_of[th] = d.Step(ch)

    cls = collections.Counter()
    acc = collections.Counter()          # integrals in ns of the new quantities
    bind = collections.Counter()         # which cap binds, time with R>0 and I>0
    hist = collections.Counter()         # (I, R, Q capped at 3) -> ns, while R>0
    place_E = collections.Counter()      # E's mass by placement of the counted workers
    place_E_recv = collections.Counter()
    for a_, b_ in zip(bp, bp[1:]):
        dt = b_ - a_
        if dt <= 0:
            continue
        m = (a_ + b_) / 2
        av = avail.at(m)
        idle_cpus = sum(1 for c in CPUS if occupant_role(occ[c].at(m)) == "idle")
        nondec = [th for th in workers if phase[th].at(m) != "decoding"]
        r_mw, r_rc = [], []
        for th in nondec:
            ph = phase[th].at(m)
            st = st_of[th].at(m)
            if av <= 0:
                key = "unresolved.send_bracket" if bracket.at(m) > 0 else f"no_job.{st or 'unknown'}"
                if key.startswith("unresolved"):
                    cls[key] += dt
                continue
            mt = mark_times[th]
            i = bisect.bisect_left(mt, m)
            near = min([abs(m - mt[j]) for j in (i - 1, i) if 0 <= j < len(mt)] + [float("inf")])
            j = bisect.bisect_left(avail_times, m)
            near_av = min([abs(m - avail_times[k]) for k in (j - 1, j) if 0 <= k < len(avail_times)] + [float("inf")])
            if near < u or near_av < u:
                cls["unresolved.clock"] += dt
                continue
            if st is None:
                cls["unresolved.sched"] += dt
                continue
            cpu = "cpu_idle" if idle_cpus > 0 else "cpus_busy"
            if st == "Q":
                cls[f"runnable.{ph}.{cpu}"] += dt
                if ph == "mutex_wait":
                    r_mw.append(th)
                elif ph == "recv":
                    r_rc.append(th)
                continue
            owner = next((w for s, e, w in owner_iv if s <= m < e and w != th), None) if ph == "mutex_wait" else None
            if ph == "mutex_wait":
                key = ("rx.blocked_by_owner" if st == "S" else "rx.running_owner_held") if owner is not None else ("rx.handoff_asleep" if st == "S" else "rx.acquiring")
                cls[f"{key}.{cpu}"] += dt
            elif ph == "recv":
                cls["recv.asleep" if st == "S" else "recv.running"] += dt
            elif ph in ("pub_lock", "buf_lock"):
                cls["state_lock.blocked" if st == "S" else "state_lock.running"] += dt
            else:
                cls[f"{ph}.{'blocked' if st == 'S' else 'running'}"] += dt

        I = idle_cpus
        Q = av
        unk = sum(1 for c in CPUS if occupant_role(occ[c].at(m)) == "unknown")
        acc["occupant_unknown_cpu"] += unk * dt
        acc["occupant_unknown_with_R"] += unk * (len(r_mw) > 0) * dt
        lo, hi = bisect.bisect_right(extra, a_), bisect.bisect_left(extra, b_)
        pts = [a_] + extra[lo:hi] + [b_]
        sub = [((x + y) / 2, y - x) for x, y in zip(pts, pts[1:]) if y > x]
        # workers that hold or are running towards the receiver claim queued jobs first:
        # running (not runnable) in mutex_wait / recv / release (between RecvDone and JobGot
        # the job has already left Q)
        claim = sum(1 for th in workers if phase[th].at(m) in ("mutex_wait", "recv") and st_of[th].at(m) == "R")
        Qs = max(0, Q - claim)
        for tag, Rset in (("mw", r_mw), ("mwrecv", r_mw + r_rc)):
            R = len(Rset)
            acc[f"{tag}.R"] += R * dt
            acc[f"{tag}.R_Igt0"] += R * (I > 0) * dt
            acc[f"{tag}.minIR"] += min(I, R) * dt
            e = min(I, R, Q)
            acc[f"{tag}.minIRQ"] += e * dt
            acc[f"{tag}.minIRQstrict"] += min(I, R, Qs) * dt
            if R and I:
                for sm, sdt in sub:
                    acc[f"{tag}.minIRQlit"] += min(I, R, qlit.at(sm)) * sdt
                    orun = sum(1 for th in others if st_of[th].at(sm) == "Q")
                    acc[f"{tag}.minIoRQ"] += min(max(0, I - orun), R, Q) * sdt
            acc[f"{tag}.minIRQ_gt0_time"] += (e > 0) * dt
            if R > 0 and I > 0:
                # which cap is the strict minimum (ties: R meets the cap, nothing lost)
                if R <= min(I, Q):
                    bk = "R (no cap binds)"
                elif I <= Q:
                    bk = "I binds" if I < Q else "I and Q tie"
                else:
                    bk = "Q binds"
                bind[(tag, bk)] += dt
            if R > 0:
                hist[(tag, min(I, 4), R, min(Q, 3))] += dt
            if e > 0:
                pe = place_E if tag == "mw" else place_E_recv
                share = e / R * dt
                for th in Rset:
                    pe[place_of[th].at(m)] += share

    arch = json.loads((cap / f"{b}.json").read_text())["classes_s"]
    keys = set(arch) | {k for k in cls}
    diff = max(abs(arch.get(k, 0.0) - cls.get(k, 0) / 1e9) for k in keys)
    return dict(run=b, span_s=span / 1e9, W_s=W / 1e9, workers=len(workers), u_ns=u, alignment_ok=al.get("ok"),
                classes_check_max_abs_diff_s=diff,
                cls_runnable_mw_idle_s=cls.get("runnable.mutex_wait.cpu_idle", 0) / 1e9,
                cls_runnable_recv_idle_s=cls.get("runnable.recv.cpu_idle", 0) / 1e9,
                unresolved_s={k: v / 1e9 for k, v in cls.items() if k.startswith("unresolved")},
                integrals_s={k: v / 1e9 for k, v in sorted(acc.items())},
                binding_s={f"{a} / {b2}": v / 1e9 for (a, b2), v in sorted(bind.items())},
                state_hist_s={f"{t}|I={i}|R={r}|Q={'3+' if q == 3 else q}": v / 1e9 for (t, i, r, q), v in sorted(hist.items())},
                placement_of_E_s={k if k else "not_runnable?": v / 1e9 for k, v in place_E.items()},
                placement_of_E_recv_s={k if k else "not_runnable?": v / 1e9 for k, v in place_E_recv.items()})


runs = []
for f in sorted(cap.glob("w4-r??.tsv.gz")):
    r = analyse(f)
    runs.append(r)
    print(r["run"], "classes diff", r["classes_check_max_abs_diff_s"], "E_mw", r["integrals_s"]["mw.minIRQ"] / r["span_s"], file=sys.stderr)


def stats(v):
    return dict(median=statistics.median(v), min=min(v), max=max(v), mean=statistics.fmean(v))


summ = {}
for tag in ("mw", "mwrecv"):
    for q in ("R", "R_Igt0", "minIR", "minIRQ", "minIRQlit", "minIRQstrict", "minIoRQ"):
        k = f"{tag}.{q}"
        cores = [r["integrals_s"][k] / r["span_s"] for r in runs]
        pct4 = [r["integrals_s"][k] / r["W_s"] * 100 for r in runs]
        pooled_cores = sum(r["integrals_s"][k] for r in runs) / sum(r["span_s"] for r in runs)
        pooled_pct = sum(r["integrals_s"][k] for r in runs) / sum(r["W_s"] for r in runs) * 100
        summ[k] = dict(per_run_cores=cores, per_run_pct_of_4=pct4, cores=stats(cores), pct_of_4=stats(pct4),
                       pooled_cores=pooled_cores, pooled_pct_of_4=pooled_pct)
pool = collections.Counter()
for r in runs:
    for k, v in r["binding_s"].items():
        pool[k] += v
place = collections.Counter(); place_r = collections.Counter()
for r in runs:
    place.update(r["placement_of_E_s"]); place_r.update(r["placement_of_E_recv_s"])
hist = collections.Counter()
for r in runs:
    hist.update(r["state_hist_s"])
out = dict(note="descriptive, post hoc; recorded schedule's directly fillable capacity, not a simulation",
           pooled_W_s=sum(r["W_s"] for r in runs), pooled_span_s=sum(r["span_s"] for r in runs),
           classes_check_max_abs_diff_s=max(r["classes_check_max_abs_diff_s"] for r in runs),
           summary=summ, binding_pooled_s=dict(sorted(pool.items())),
           placement_of_E_pooled_s=dict(place), placement_of_E_recv_pooled_s=dict(place_r),
           state_hist_pooled_s=dict(sorted(hist.items(), key=lambda x: -x[1])), runs=runs)
outp.write_text(json.dumps(out, indent=1) + "\n")
