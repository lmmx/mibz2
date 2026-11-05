"""Discrete-event model of a 2-core x 2-SMT box decoding identical full blocks.

Each block: S (scanner), M (MTF), B (build), W (walk incl. assembly), R (RLE1 + CRC).
A stage X progresses at rate 1 / f(X | class of the sibling's current stage), with
f(X | idle) = 1. Classes: L (S, M, R), H (B), W (W), I (idle).

Policies:
  today : each worker runs its block S->M->B->W->R, then takes the next job (FIFO).
  token : workers pinned in pairs; W needs the core's token; a worker holding a built
          block that cannot get the token starts (or continues) another block's front
          half, up to `wcap` blocks held; parks for the token when it can do nothing else.
          On release the token goes straight to a parked sibling.

Global job supply: in order, window `win` (blocks issued - blocks read), reader instant.
A throwaway probe for a design note: no timing of real code.
"""
import sys
import random

CLS = {"S": "L", "M": "L", "B": "H", "W": "W", "R": "L"}
ORDER = ["S", "M", "B", "W", "R"]


def calib(alone, mixed, outcome, delta=0.0, cw=0.0):
    """Factor table f[X][Y] that reproduces the observed 4-copy factors `mixed`, given
    the sibling's class shares p (time shares of the 4-copy profile).
    outcome C: f(X|Y) = mixed[X] for every busy Y.
    outcome P: f(W|W) - f(W|L or H) = delta.
    outcome M: f(W|W) = f(W|H); f(W|W) - f(W|L) = delta.
    cw: compute stages beside a walk cost `cw` more than beside compute (re-solved so the
    mixture still matches)."""
    t4 = {x: alone[x] * mixed[x] for x in alone}
    tot = sum(t4.values())
    p = {"W": t4["W"] / tot, "H": t4["B"] / tot}
    p["L"] = 1 - p["W"] - p["H"]
    f = {}
    for x in alone:
        f[x] = {"I": 1.0}
        if x == "W":
            m = mixed[x]
            if outcome == "C":
                for y in "LHW":
                    f[x][y] = m
            elif outcome == "P":
                # m = pW*(c+delta) + (1-pW)*c
                c = m - p["W"] * delta
                f[x].update(L=c, H=c, W=c + delta)
            elif outcome == "M":
                # m = (pW+pH)*(c+delta) + pL*c
                c = m - (p["W"] + p["H"]) * delta
                f[x].update(L=c, H=c + delta, W=c + delta)
        else:
            m = mixed[x]
            # m = pW*(c+cw) + (1-pW)*c
            c = m - p["W"] * cw
            f[x].update(L=c, H=c, W=c + cw)
    return f, p


class Sim:
    def __init__(self, alone, f, policy, nblocks=400, win=16, wcap=2, ccap=99,
                 keep=False, cores=2, jit=0.3, seed=1, buildtok=False):
        self.alone, self.f, self.policy = alone, f, policy
        self.n, self.win, self.wcap, self.ccap, self.keep = nblocks, win, wcap, ccap, keep
        self.cores = cores
        self.buildtok = buildtok
        self.t = 0.0
        self.issued = 0  # blocks issued by the scanner (into the queue)
        self.queue = []  # issued, not yet taken by a worker
        self.done = set()
        self.taken = 0  # read by the reader
        W = 2 * cores
        self.w = [dict(i=i, core=i // 2, cur=None, rem=0.0, held=[], parked=False,
                       jobwait=False) for i in range(W)]
        self.token = [None] * cores
        self.stats = dict(tok_busy=0.0, park=0.0, jobwait=0.0, live=0.0, live_max=0,
                          pair_ww=0.0, walk_t=0.0, walk_beside_idle=0.0)
        self.order_hold = 0.0
        rng = random.Random(seed)
        self.size = [rng.uniform(1 - jit, 1 + jit) for _ in range(nblocks)]
        self.refill()

    # --- supply
    def refill(self):
        while self.issued < self.n and self.issued - self.taken < self.win:
            self.queue.append(self.issued)
            self.issued += 1

    def read(self):
        while self.taken in self.done:
            self.taken += 1
        self.refill()

    def sib(self, w):
        return self.w[w["i"] ^ 1]

    def cls(self, w):
        return CLS[w["cur"][0]] if w["cur"] else "I"

    def start(self, w, stage, blk):
        w["cur"] = (stage, blk)
        w["rem"] = self.alone[stage] * self.size[blk["id"]]
        w["parked"] = False
        w["jobwait"] = False

    # --- policies
    def decide(self, w):
        if w["cur"] is not None:
            return
        if self.policy == "today":
            if w["held"]:
                b = w["held"][0]
                nxt = ORDER[ORDER.index(b["st"]) + 1] if b["st"] else "S"
                self.start(w, nxt, b)
                return
            if self.queue:
                b = dict(id=self.queue.pop(0), st=None)
                w["held"].append(b)
                self.start(w, "S", b)
            else:
                w["jobwait"] = True
            return
        # token policy
        core = w["core"]
        built = [b for b in w["held"] if b["st"] == "B"]
        if built and self.token[core] is None:
            self.token[core] = w["i"]
            self.start(w, "W", min(built, key=lambda b: b["id"]))
            return
        # a walked block's tail (only with keep=True, deferred tails)
        walked = [b for b in w["held"] if b["st"] == "W"]
        if walked:
            self.start(w, "R", min(walked, key=lambda b: b["id"]))
            return
        # continue a front half
        for b in sorted(w["held"], key=lambda b: b["id"]):
            if b["st"] in (None, "S", "M"):
                nxt = "S" if b["st"] is None else ORDER[ORDER.index(b["st"]) + 1]
                if nxt == "B" and self.core_tables(core) >= self.ccap:
                    continue
                if nxt == "B" and self.buildtok:
                    if self.token[core] is not None:
                        continue
                    self.token[core] = w["i"]
                self.start(w, nxt, b)
                return
        if len(w["held"]) < self.wcap and self.queue:
            b = dict(id=self.queue.pop(0), st=None)
            w["held"].append(b)
            self.start(w, "S", b)
            return
        if built:
            w["parked"] = True
        elif not w["held"]:
            w["jobwait"] = True
        else:
            # held blocks all blocked by the core cap (MTF'd, waiting to build): wait
            w["parked"] = True

    def core_tables(self, core):
        """tables built and waiting, or being built, on a core (not the one walking)"""
        n = 0
        for w in self.w[2 * core: 2 * core + 2]:
            for b in w["held"]:
                if b["st"] == "B" and not (w["cur"] and w["cur"][1] is b):
                    n += 1
            if w["cur"] and w["cur"][0] == "B":
                n += 1
        return n

    def finish(self, w):
        stage, b = w["cur"]
        w["cur"] = None
        b["st"] = stage
        if stage == "W" and self.policy == "token":
            core = w["core"]
            s = self.sib(w)
            more = [x for x in w["held"] if x["st"] == "B"]
            if s["parked"] and any(x["st"] == "B" for x in s["held"]):
                self.token[core] = s["i"]
                sb = min([x for x in s["held"] if x["st"] == "B"], key=lambda x: x["id"])
                self.start(s, "W", sb)
            elif self.keep and more:
                # keep the token: walk the next built block now, the tail after
                self.start(w, "W", min(more, key=lambda x: x["id"]))
                return
            else:
                self.token[core] = None
            self.start(w, "R", b)
            return
        if stage == "B" and self.policy == "token" and self.buildtok:
            core = w["core"]
            s_ = self.sib(w)
            if s_["parked"] and any(x["st"] == "B" for x in s_["held"]):
                self.token[core] = s_["i"]
                sb = min([x for x in s_["held"] if x["st"] == "B"], key=lambda x: x["id"])
                self.start(s_, "W", sb)
            else:
                self.token[core] = None
            return
        if stage == "R":
            w["held"].remove(b)
            self.done.add(b["id"])
            self.read()

    def live_tables(self):
        n = 0
        for w in self.w:
            for b in w["held"]:
                if b["st"] == "B":
                    n += 1
            if w["cur"] and w["cur"][0] in ("B",) and w["cur"][1]["st"] != "B":
                n += 1
        return n

    def run(self):
        for w in self.w:
            self.decide(w)
        guard = 0
        while len(self.done) < self.n:
            guard += 1
            if guard > 10_000_000:
                raise RuntimeError("no progress")
            # wake job-waiters / parked workers whose situation changed
            changed = True
            while changed:
                changed = False
                for w in self.w:
                    if w["cur"] is None:
                        before = (w["parked"], w["jobwait"])
                        w["parked"] = w["jobwait"] = False
                        self.decide(w)
                        if w["cur"] is not None:
                            changed = True
                        elif (w["parked"], w["jobwait"]) != before:
                            pass
            running = [w for w in self.w if w["cur"] is not None]
            if not running:
                raise RuntimeError(f"deadlock at t={self.t} done={len(self.done)}")
            rates = {}
            for w in running:
                st = w["cur"][0]
                rates[w["i"]] = 1.0 / self.f[st][self.cls(self.sib(w))]
            dt = min(w["rem"] / rates[w["i"]] for w in running)
            # stats
            for c in range(self.cores):
                if self.token[c] is not None:
                    self.stats["tok_busy"] += dt
            for w in self.w:
                if w["cur"] is None:
                    if w["parked"]:
                        self.stats["park"] += dt
                    else:
                        self.stats["jobwait"] += dt
                elif w["cur"][0] == "W":
                    self.stats["walk_t"] += dt
                    sc = self.cls(self.sib(w))
                    if sc == "W":
                        self.stats["pair_ww"] += dt
                    elif sc == "I":
                        self.stats["walk_beside_idle"] += dt
            lt = self.live_tables()
            self.stats["live"] += lt * dt
            self.stats["live_max"] = max(self.stats["live_max"], lt)
            self.t += dt
            for w in running:
                w["rem"] -= dt * rates[w["i"]]
            for w in running:
                if w["rem"] <= 1e-9:
                    self.finish(w)
                    self.decide(w)
        s = self.stats
        T = self.t
        return dict(T=T, per_block=T / self.n,
                    tok_busy=s["tok_busy"] / (T * self.cores),
                    park=s["park"] / (T * 2 * self.cores),
                    jobwait=s["jobwait"] / (T * 2 * self.cores),
                    live_mean=s["live"] / T, live_max=s["live_max"],
                    ww=s["pair_ww"] / max(s["walk_t"], 1e-12),
                    w_idle=s["walk_beside_idle"] / max(s["walk_t"], 1e-12))


# Observed alone c/B and 4-copy factors (phases runs; census/falsify-walk-smt/factors.py)
RUNNERS = {
    # 7763: phases 42011789918 (sweep-4bfa)
    "7763": (dict(S=1.23, M=1.98, B=2.16, W=4.33, R=0.54),
             dict(S=1.43, M=1.33, B=1.43, W=2.12, R=(0.43 + 0.37) / 0.54)),
    # 9V74: phases 42020095581
    "9V74": (dict(S=1.17, M=2.00, B=1.80, W=4.22, R=0.45),
             dict(S=1.32, M=1.23, B=1.76, W=2.22, R=(0.35 + 0.35) / 0.45)),
    # 8370C: phases 42008210159 (sweep-b773)
    "8370C": (dict(S=1.49, M=2.22, B=2.01, W=7.27, R=0.47),
              dict(S=1.56, M=1.54, B=2.25, W=1.46, R=(0.65 + 0.26) / 0.47)),
}


def observed_T4(alone, mixed):
    return sum(alone[x] * mixed[x] for x in alone)


if __name__ == "__main__":
    N = int(sys.argv[1]) if len(sys.argv) > 1 else 600
    seeds = (1, 2, 3)
    for name, (alone, mixed) in RUNNERS.items():
        obs = observed_T4(alone, mixed)
        f0, p = calib(alone, mixed, "C")
        print(f"== {name}: observed 4-copy {obs:.2f} c/B a worker; shares pW {p['W']:.3f} "
              f"pH {p['H']:.3f} pL {p['L']:.3f}")
        if name != "8370C":
            cases = [("C", 0.0)] + [("P", d) for d in (0.3, 0.6, 1.0, 1.4)] + \
                [("M", d) for d in (0.3, 0.6, 1.0)]
        else:
            cases = [("C", 0.0)] + [("P", d) for d in (0.2, 0.4, 0.6)] + \
                [("M", d) for d in (0.2, 0.4)]
        for oc, d in cases:
            for cw in (0.0, 0.2):
                f, _ = calib(alone, mixed, oc, d, cw)
                res = {}
                for key, kw in (("today", dict(policy="today")),
                                ("tok", dict(policy="token", wcap=2, keep=True)),
                                ("tok0", dict(policy="token", wcap=2, keep=False)),
                                ("tokc1", dict(policy="token", wcap=2, ccap=1, keep=True))):
                    pol = kw.pop("policy")
                    rs = [Sim(alone, f, pol, N, seed=sd, **kw).run() for sd in seeds]
                    res[key] = {k: sum(r[k] for r in rs) / len(rs) for k in rs[0]}
                b = res["today"]
                g = {k: (res[k]["T"] / b["T"] - 1) * 100 for k in ("tok", "tok0", "tokc1")}
                t = res["tok"]
                print(f"  {oc} d={d:.1f} cw={cw:.1f} f(W|C)={f['W']['L']:.2f}/{f['W']['H']:.2f} "
                      f"f(W|W)={f['W']['W']:.2f} | today {4*b['per_block']:.2f} ww {b['ww']:.2f} "
                      f"live {b['live_mean']:.2f}/{b['live_max']} | token {g['tok']:+5.1f}% "
                      f"(keep0 {g['tok0']:+5.1f}%, ccap1 {g['tokc1']:+5.1f}%) tok_busy "
                      f"{t['tok_busy']:.2f} park {t['park']:.3f} w|idle {t['w_idle']:.2f} "
                      f"live {t['live_mean']:.2f}/{t['live_max']:.0f} c1-live "
                      f"{res['tokc1']['live_mean']:.2f}/{res['tokc1']['live_max']:.0f}")
