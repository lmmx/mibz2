#!/usr/bin/env python3
"""Hand-made invariant mutants of the pipelined team (src/coop.rs, src/parallel.rs).

Usage: mutants.py WORKTREE OUTDIR [ID ...]
Each mutant is applied to WORKTREE's src/ (a throwaway worktree, never the PR branch),
then: the pipelined integration tests, coop's unit tests, (for the relay's mutants) coop's
Miri tests, and the pipeline fuzz for a bounded time. Each with a timeout: a hang is a kill.
src/ is restored after each mutant.
"""
import json, os, subprocess, sys, time

M = {
 "M0": ("none (the unmutated baseline)", "src/parallel.rs", None, None),
 "M1": ("build-to-walk ordering: walk_ready published before the build's end check and walk set-up",
   "src/coop.rs",
   """            unsafe { relay.end_build() };
            relay.set_walk_ready::<PARK>();""",
   """            relay.set_walk_ready::<PARK>();
            unsafe { relay.end_build() };"""),
 "M2": ("walk_ready's release/acquire weakened to relaxed (the stage's publication)",
   "src/coop.rs",
   None, None),  # two edits, below
 "M3": ("relay handoff: the walk's count starts one short, so its last-to-leave is early",
   "src/coop.rs",
   "        walkers: AtomicUsize::new(helpers),\n        failed",
   "        walkers: AtomicUsize::new(helpers - 1),\n        failed"),
 "M4": ("park: a parked waiter returns on any wake-up, without re-checking the flag",
   "src/coop.rs",
   """        lock(&self.sleepers).push(thread::current());
        while !ready() {
            thread::park();
        }""",
   """        lock(&self.sleepers).push(thread::current());
        if !ready() {
            thread::park();
        }"""),
 "M5": ("park/unpark ordering: the setter drains the sleepers before storing the flag (lost wake-up)",
   "src/coop.rs",
   """        self.walk_ready.store(true, Ordering::Release);
        if PARK {
            for t in lock(&self.sleepers).drain(..) {
                t.unpark();
            }
        }""",
   """        let ts: Vec<Thread> = if PARK {
            lock(&self.sleepers).drain(..).collect()
        } else {
            Vec::new()
        };
        std::thread::yield_now();
        self.walk_ready.store(true, Ordering::Release);
        for t in ts {
            t.unpark();
        }"""),
 "M6": ("error placement: a move-to-front error published one place late",
   "src/parallel.rs",
   """        Ok(Err(kind)) => {
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }""",
   """        Ok(Err(kind)) => {
            publish(shared, place + 1, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }"""),
 "M7": ("error propagation: a move-to-front error swallowed, the block decoded anyway",
   "src/parallel.rs",
   """        Ok(Err(kind)) => {
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }""",
   """        Ok(Err(_kind)) => Staging::Small(s),"""),
 "M8": ("never-wait: the leader waits for the next job inside a run (holding an unpublished block)",
   "src/parallel.rs",
   """    let rx = match jobs.try_lock() {
        Ok(rx) => rx,
        Err(TryLockError::Poisoned(p)) => p.into_inner(),
        Err(TryLockError::WouldBlock) => return None,
    };
    rx.try_recv().ok()""",
   """    let _ = TryLockError::<()>::WouldBlock;
    let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
    rx.recv().ok()"""),
 "M9": ("small-block priming boundary: lever B keeps staging after a block for the team, into the same front set",
   "src/parallel.rs",
   """                    Staging::Team(next) => l.next = Some(next),
                    Staging::Small(b) => {""",
   """                    Staging::Team(next) => {
                        l.next = Some(next);
                        if SMALL {
                            next_job = try_job(jobs);
                        }
                    }
                    Staging::Small(b) => {"""),
 "M10": ("buffer reuse: the helpers assemble block i into the buffer block i - 1 is finished from",
   "src/parallel.rs",
   "        let a = inverted.as_ref().map_or(0, |b| 1 - b.buf);",
   "        let a = inverted.as_ref().map_or(0, |b| b.buf);"),
 "M11": ("front-set ownership: the next block labelled with the front set the helpers are reading",
   "src/parallel.rs",
   "        let other = 1 - s.front;",
   "        let other = s.front;"),
 "M12": ("panic completion: a run that panics publishes none of the blocks it held",
   "src/parallel.rs",
   """                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten() {""",
   """                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten().take(0) {"""),
}

M2_EDITS = [
    ("        self.walk_ready.store(true, Ordering::Release);",
     "        self.walk_ready.store(true, Ordering::Relaxed);"),
    ("        let ready = || self.walk_ready.load(Ordering::Acquire);",
     "        let ready = || self.walk_ready.load(Ordering::Relaxed);"),
]

RELAY = {"M0", "M1", "M2", "M3", "M4", "M5"}


def run(cmd, cwd, env, timeout, log):
    t0 = time.time()
    with open(log, "w") as f:
        try:
            p = subprocess.run(cmd, cwd=cwd, env=env, stdout=f, stderr=subprocess.STDOUT,
                               timeout=timeout)
            rc = p.returncode
        except subprocess.TimeoutExpired:
            rc = "TIMEOUT"
    return rc, round(time.time() - t0)


def tail(log, n=6):
    try:
        return open(log, errors="replace").read().splitlines()[-n:]
    except OSError:
        return []


def main():
    wt, out = sys.argv[1], sys.argv[2]
    ids = sys.argv[3:] or list(M)
    os.makedirs(out, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(wt, "target"),
               CARGO_PROFILE_FUZZ_LTO="false", CARGO_PROFILE_FUZZ_CODEGEN_UNITS="16")
    fuzz_secs = os.environ.get("FUZZ_SECS", "45")
    results = []
    for mid in ids:
        what, path, old, new = M[mid]
        f = os.path.join(wt, path)
        src = open(f).read()
        if mid == "M0":
            mutated = src
        elif mid == "M2":
            mutated = src
            for o, n in M2_EDITS:
                assert mutated.count(o) == 1, (mid, o)
                mutated = mutated.replace(o, n)
        else:
            assert src.count(old) == 1, (mid, old)
            mutated = src.replace(old, new)
        open(f, "w").write(mutated)
        subprocess.run(["git", "diff"], cwd=wt, stdout=open(f"{out}/{mid}.patch", "w"))
        r = {"id": mid, "invariant": what, "file": path}
        try:
            rc, s = run(["cargo", "build", "--profile", "fuzz", "--example", "fuzz"], wt, env, 900,
                        f"{out}/{mid}-build.log")
            rc2, _ = run(["cargo", "test", "--no-run", "--test", "pipelined", "--lib"], wt, env, 900,
                         f"{out}/{mid}-build-tests.log")
            if rc != 0 or rc2 != 0:
                r["build"] = "FAILED"
                results.append(r)
                print(json.dumps(r), flush=True)
                continue
            r["pipelined"] = run(["cargo", "test", "--test", "pipelined"], wt, env, 300,
                                 f"{out}/{mid}-pipelined.log")
            r["coop_unit"] = run(["cargo", "test", "--lib", "coop::"], wt, env, 300,
                                 f"{out}/{mid}-coop.log")
            if mid in RELAY and os.environ.get("MIRI", "1") == "1":
                menv = dict(env, CARGO_TARGET_DIR=os.path.join(wt, "target-miri"),
                            MIRIFLAGS=os.environ.get("MIRIFLAGS", ""))
                r["miri"] = run(["cargo", "+nightly", "miri", "test", "--lib", "--",
                                 "coop::tests::miri_"], wt, menv, 900, f"{out}/{mid}-miri.log")
            fuzz = os.path.join(wt, "target/fuzz/examples/fuzz")
            r["fuzz"] = run([fuzz, fuzz_secs, "7", f"{out}/{mid}-cases", "pipeline"], wt, env,
                            int(fuzz_secs) + 240, f"{out}/{mid}-fuzz.log")
            r["fuzz_tail"] = tail(f"{out}/{mid}-fuzz.log", 4)
        finally:
            open(f, "w").write(src)
        results.append(r)
        print(json.dumps(r), flush=True)
    json.dump(results, open(f"{out}/results-{'-'.join(ids)}.json", "w"), indent=1)


if __name__ == "__main__":
    main()
