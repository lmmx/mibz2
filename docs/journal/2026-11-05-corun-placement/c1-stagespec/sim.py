"""Cost model for stage-specialised SMT pairs (construction 1). One physical core, two
siblings. Stages per block, alone c/B (phases 'alone' rows): S scanner, M MTF+RLE2,
B build, W walk incl. assembly, F RLE1+CRC. A stage X progresses at rate 1/f(X|Y), Y the
sibling's current stage (None = idle, f = 1).

today: both siblings loop S M B W F on their own blocks (the 4-copy phases run; the
decoder's 4 workers + floating scanner are the same work on the same 2 cores).
staged: one walker, one compute thread; only the walker walks; S M B F are shared,
with per-core buffer caps (TT T-vectors, LL MTF outputs, OUT walked-unfinished).

Pair factors f(X|Y) = 1 + a[X] * g(X,Y); g is the outcome's pattern, a[X] is fitted so
the 'today' simulation reproduces the observed mixed 4-copy factors (calibration)."""
import random, sys

MODELS = {
    # alone c/B and mixed 4-copy factor per stage (falsify-walk-smt/factors.py output)
    "7763":  dict(alone=dict(S=1.23, M=1.98, B=2.16, W=4.33, F=0.54),
                  mixed=dict(S=1.43, M=1.33, B=1.43, W=2.12, F=1.48)),
    "9V74":  dict(alone=dict(S=1.17, M=2.00, B=1.80, W=4.22, F=0.45),
                  mixed=dict(S=1.32, M=1.23, B=1.76, W=2.22, F=1.56)),
    "8370C": dict(alone=dict(S=1.49, M=2.22, B=2.01, W=7.27, F=0.47),
                  mixed=dict(S=1.56, M=1.54, B=2.25, W=1.46, F=1.94)),
}
NONW = "SMBF"


def g_pattern(outcome, npat):
    """g(X, Y) for busy Y. outcome: walk's pattern; npat: non-walk stages' pattern."""
    def g(x, y):
        if y is None:
            return 0.0
        if x == "W":
            if outcome == "C":
                return 1.0
            if outcome == "M":      # only memory-heavy siblings (walk, build) cost
                return 1.0 if y in "WB" else 0.1
            if outcome == "M2":     # only the walk itself is 'memory-heavy'
                return 1.0 if y == "W" else 0.1
            if outcome == "P":      # walk beside compute half the excess of walk beside walk
                return 1.0 if y == "W" else 0.5
            if outcome == "P25":
                return 1.0 if y == "W" else 0.25
            raise ValueError(outcome)
        # non-walk stage x
        if npat == "flat":
            return 1.0
        if npat == "prefers_walk":   # compute contends with compute (issue), less with walk
            return 0.6 if y == "W" else 1.0
        if npat == "dislikes_walk":
            return 1.0 if y == "W" else 0.6
        raise ValueError(npat)
    return g


class Factors:
    def __init__(self, a, g):
        self.a, self.g = a, g

    def f(self, x, y):
        return 1.0 + self.a[x] * self.g(x, y)


def jit(rng, w, j=0.2):
    return w * rng.uniform(1 - j, 1 + j)


def run_today(m, fac, blocks=4000, seed=1):
    rng = random.Random(seed)
    alone = m["alone"]
    # per thread: list of (stage, remaining work)
    def new_block():
        return [(s, jit(rng, alone[s])) for s in "SMBWF"]
    th = [new_block(), new_block()]
    # random phase offset for thread 1
    off = rng.uniform(0, sum(alone.values()))
    while off > 0 and th[1]:
        s, r = th[1][0]
        if r <= off:
            off -= r; th[1].pop(0)
        else:
            th[1][0] = (s, r - off); off = 0
    if not th[1]:
        th[1] = new_block()
    t = 0.0
    wall = {s: 0.0 for s in "SMBWF"}
    work = {s: 0.0 for s in "SMBWF"}
    done = 0
    warm = blocks // 10
    tw0 = None
    while done < blocks:
        st = [th[i][0][0] for i in (0, 1)]
        rates = [1.0 / fac.f(st[0], st[1]), 1.0 / fac.f(st[1], st[0])]
        dts = [th[i][0][1] / rates[i] for i in (0, 1)]
        dt = min(dts)
        for i in (0, 1):
            s, r = th[i][0]
            if done >= warm:
                wall[s] += dt
                work[s] += dt * rates[i]
            r -= dt * rates[i]
            th[i][0] = (s, r)
        t += dt
        for i in (0, 1):
            if th[i][0][1] <= 1e-12:
                th[i].pop(0)
                if not th[i]:
                    done += 1
                    if done == warm:
                        tw0 = t
                    th[i] = new_block()
    per_block = (t - tw0) / (blocks - warm)  # core wall per block (2 threads)
    mixed = {s: wall[s] / work[s] for s in "SMBWF"}
    return per_block, mixed


def run_staged(m, fac, blocks=4000, seed=1, TT=3, LL=2, OUT=2, overflow=False, wb_only=False, walker_sm=True):
    """Walker = thread 0, compute = thread 1. Work-conserving except: only thread 0 walks
    (overflow: thread 1 may also walk when it would otherwise idle)."""
    rng = random.Random(seed)
    alone = m["alone"]
    mtfd = 0      # blocks MTF'd, waiting for a build (hold an ll)
    built = 0     # T-vectors built, waiting for the walk
    walked = 0    # assembled, waiting for RLE1+CRC
    in_sm = 0     # blocks in S/M
    in_b = 0
    in_w = 0
    cur = [None, None]   # (stage, remaining)
    t = 0.0
    done = 0
    warm = blocks // 10
    tw0 = None
    busy = {0: {s: 0.0 for s in "SMBWF"}, 1: {s: 0.0 for s in "SMBWF"}}
    idle = [0.0, 0.0]
    pairs = {}

    def pick(i):
        nonlocal mtfd, built, walked, in_sm, in_b, in_w
        tt_used = in_b + built + in_w
        ll_used = in_sm + mtfd
        if i == 0:
            if built:
                built -= 1; in_w += 1; return ["W", jit(rng, alone["W"])]
            if walked:   # keep the walker's output buffers flowing
                walked -= 1; return ["F", jit(rng, alone["F"])]
            if mtfd and tt_used < TT:
                mtfd -= 1; in_b += 1; return ["B", jit(rng, alone["B"])]
            if walker_sm and ll_used < LL:
                in_sm += 1; return ["SM", jit(rng, alone["S"]), jit(rng, alone["M"])]
            return None
        else:
            if (not wb_only) and mtfd and tt_used < TT:
                mtfd -= 1; in_b += 1; return ["B", jit(rng, alone["B"])]
            if ll_used < LL:
                in_sm += 1; return ["SM", jit(rng, alone["S"]), jit(rng, alone["M"])]
            if walked:
                walked -= 1; return ["F", jit(rng, alone["F"])]
            if overflow and built:
                built -= 1; in_w += 1; return ["W", jit(rng, alone["W"])]
            return None

    def stage_of(c):
        if c is None:
            return None
        if c[0] == "SM":
            return "S" if c[1] > 0 else "M"
        return c[0]

    for i in (0, 1):
        cur[i] = pick(i)
    while done < blocks:
        for i in (0, 1):
            if cur[i] is None:
                cur[i] = pick(i)
        st = [stage_of(cur[0]), stage_of(cur[1])]
        if st[0] is None and st[1] is None:
            raise RuntimeError("deadlock")
        rates = [0.0, 0.0]
        dts = []
        for i in (0, 1):
            if st[i] is None:
                dts.append(float("inf")); continue
            rates[i] = 1.0 / fac.f(st[i], st[1 - i])
            rem = cur[i][1] if cur[i][0] != "SM" or cur[i][1] > 0 else cur[i][2]
            dts.append(rem / rates[i])
        dt = min(dts)
        key = tuple(st)
        if done >= warm:
            pairs[key] = pairs.get(key, 0.0) + dt
            for i in (0, 1):
                if st[i] is None:
                    idle[i] += dt
                else:
                    busy[i][st[i]] += dt
        for i in (0, 1):
            if st[i] is None:
                continue
            c = cur[i]
            if c[0] == "SM" and c[1] > 0:
                c[1] -= dt * rates[i]
            elif c[0] == "SM":
                c[2] -= dt * rates[i]
            else:
                c[1] -= dt * rates[i]
        t += dt
        for i in (0, 1):
            c = cur[i]
            if c is None:
                continue
            if c[0] == "SM":
                if c[1] <= 1e-12 and c[1] != 0:
                    c[1] = 0.0
                if c[1] <= 0 and c[2] <= 1e-12:
                    in_sm -= 1; mtfd += 1; cur[i] = None
            elif c[1] <= 1e-12:
                s = c[0]
                cur[i] = None
                if s == "B":
                    in_b -= 1; built += 1
                elif s == "W":
                    in_w -= 1
                    if walked < OUT:
                        walked += 1
                    else:   # no free output buffer: the walker finishes it itself
                        cur[i] = ["F", jit(rng, alone["F"])]
                elif s == "F":
                    done += 1
                    if done == warm:
                        tw0 = t
    per_block = (t - tw0) / (blocks - warm)
    span = t - tw0
    return per_block, dict(idle_walker=idle[0] / span, idle_compute=idle[1] / span,
                           pairs={k: v / span for k, v in pairs.items()})


def calibrate(m, outcome, npat, iters=30):
    a = {s: m["mixed"][s] - 1.0 for s in "SMBWF"}
    g = g_pattern(outcome, npat)
    for _ in range(iters):
        fac = Factors(a, g)
        _, mixed = run_today(m, fac, blocks=1500)
        for s in "SMBWF":
            a[s] *= (m["mixed"][s] - 1.0) / max(mixed[s] - 1.0, 1e-6)
    fac = Factors(a, g)
    _, mixed = run_today(m, fac, blocks=3000, seed=7)
    return fac, mixed


if __name__ == "__main__":
    outcomes = sys.argv[1:] or ["C", "M", "M2", "P", "P25"]
    for name, m in MODELS.items():
        for npat in ["flat", "prefers_walk", "dislikes_walk"]:
            for oc in outcomes:
                fac, mixed = calibrate(m, oc, npat)
                today, _ = run_today(m, fac, blocks=4000, seed=3)
                row = []
                for ov in (False, True):
                    st, info = run_staged(m, fac, blocks=4000, seed=3, overflow=ov)
                    row.append((st / today - 1) * 100)
                    if not ov:
                        inf0 = info
                fw = lambda y: fac.f("W", y)
                print(f"{name:6} N={npat:13} {oc:4} fWW {fw('W'):.2f} fWM {fw('M'):.2f} fWB {fw('B'):.2f} "
                      f"fMW {fac.f('M','W'):.2f} fMM {fac.f('M','M'):.2f} fBW {fac.f('B','W'):.2f} "
                      f"| calib W {mixed['W']:.2f} | staged {row[0]:+6.1f}%  +overflow {row[1]:+6.1f}% "
                      f"| idle wk {inf0['idle_walker']:.2f} cp {inf0['idle_compute']:.2f}")
