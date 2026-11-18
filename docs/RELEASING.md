# Releasing mibz2

The release runbook. The coordinator follows it step by step for every release. It is
referenced from `CLAUDE.md`, so every session reads it before releasing.

The maintainer's standing instruction is that the coordinator releases mibz2 itself, without
asking first, whenever merged work is worth releasing. Nobody needs to prompt it.

## 1. Is there something worth releasing?

After every merge to master, not only when asked, look at what has merged since the last tag:

```sh
git log <last tag>..master -- src Cargo.toml
```

Worth releasing:
- **A performance gain** shown under its PR's registered rule: the CI bench, or a registered
  workstation run admitted under RULES C5.
- **A change that is definitely committed to:** a fix users need, or an API or packaging
  change.

Docs, journal, bench-harness or blog changes alone are not.

## 2. When

- **Release unless the maintainer has said they are asleep or otherwise indisposed.** A broken
  release needs someone to hear of it. Being away from the desk doesn't count: they can still
  oversee from their phone. Only their own statement that they are asleep or indisposed holds a
  release back; never infer it from silence, a quiet session or the time of day (Louis, 17
  November 2026).
- **If work becomes releasable while they are indisposed,** release it when they are next
  available, and say so.

## 3. Before

1. **Master is green:** Test, Fingerprint and the site.
2. **One paired bench run of the last tag against master:** a `workflow_dispatch` of the Benchmark
   workflow on master, with `base=<the last tag's commit SHA>`. Give it the SHA, never the
   tag name.
   - It checks that nothing got worse. It is not a gate to re-run until the result looks good.
   - Name its CPU. It is in the run's `env.json` artifact and in its step summary.
3. **Review the research features' retention notes** (the comments above each entry of
   `Cargo.toml`'s `[features]`; #145, LOG 6893084445). A feature stays while it has a specific
   purpose and a removal condition that hasn't been met. Remove any whose reason has lapsed in
   its own PR, never in the release's. The review never delays the release, and a removal is
   never automatic: list what was reviewed, and what is to go, in the release's report.

## 4. The release

1. **Dispatch Deploy Release** with `bump`: patch, or minor for a public API change. It bumps the crate
   and the Python package together, tags `vX.Y.Z` and `py-X.Y.Z`, and starts CI.yml. CI.yml
   publishes the crate (release-plz, Trusted Publishing) and the PyPI wheels.
2. **Verify the publication:**
   - crates.io and PyPI are at the new version, with every wheel and the sdist;
   - docs.rs has built it.
3. **Fix any publishing failure** with a follow-up patch release, as #209 did.

## 5. Release scaling: the new release against the last, on the workstation

The History page's Release scaling view shows each release against the one before it on a
real many-core machine (`bench/release-scaling/`, `docs/generated/release-scaling.md`). It
shows what CI's 4-CPU runners can't, such as the decoder's knee past 8 threads (the
three-team policy came from it). Every release gets one.

1. **As soon as the tag exists, ask Louis to run, on the i9:**

   ```sh
   cd ~/dev/mibz2 && git checkout master && git pull
   just submit bench-release-scaling
   ```

   - With no arguments, the recipe compares the current release (the newest `vX.Y.Z` tag)
     against the last one (the tag before it). Never write version numbers into the command.
   - It is his run, under RULES C5. The agents never time on the workstation. Its figures inform; they don't decide.
2. **When the submission lands on #145:**
   - fetch the bundle from the `evidence` branch at the commit the post names, and check its SHA-256;
   - copy its `recorded/` directory to `bench/release-scaling/<date>-<machine>-<release>/`;
   - commit that to master. The site rebuilds the History page from it.
   - **check the live History page shows it,** after the site deploys: its hero's Release
     scaling view must open on the new run, with the run named by its releases ("0.2.9 against
     0.2.8"). Look in a fresh browser profile and in one that has seen the page before, since
     the view remembers a reader's last choice. A run that is recorded but not shown first is
     not done (10 November 2026: the hero stayed on 0.2.4 against 0.2.1 after 0.2.9's run).
3. **If the run can't be had** (the i9 is away, or Louis declines), say so in the report. The release
   doesn't wait for it.

## 5a. The team threshold: when it may have moved

The **team threshold** is the thread count at which the default decoder stops giving each
thread a block of its own (workers, Cells inverting) and has threads share blocks in teams
(`Plan::for_threads` in `src/parallel.rs`: `COOP_MIN_CORES`, `WORKERS_BELOW_THREADS`). It
was set once on 28 October and not checked again until 15 November, by which time Cells had
made workers 4–14% faster than the teams at 6–9 threads (#425).

1. **When:** a release whose changes since the last tag touch either side of the threshold:
   - the workers path: `src/cells.rs`, the T-vector build or walk, MTF, the scanner, or
     anything else a worker runs on its block;
   - the team path: `src/coop.rs`, the cooperative inverse, the pool, or the pipelined teams;
   - the plan policy itself, or the window (`SLOTS_PER_THREAD`).

   Not for docs, bench or packaging releases. When unsure, run it: it takes about 15 minutes.
2. **The run:** queue `bench-crossover` (`bench/workstation-queue.txt`, its ready tag at the
   release's tag commit) beside the release scaling. It runs the shipped decoder against one
   block a thread (`mibz2-workers-w`) and teams of 2 and 3 (`mibz2-team2`, `mibz2-team3`) at
   4 to 10, 12, 16 and 20 threads, every arm with the default window.
3. **The rule** (the one judged on 15 November, #145 LOG 6896455773, kept for every run):
   - **sanity:** where an arm runs the same plan as the shipped decoder, their interval should
     span 1. A near miss is noise to weigh, not a veto: judge the run as a whole, and repeat
     it if an effect needs confirming (Louis, 16 November 2026: use judgement, not rigid
     rules);
   - **move the threshold** to where a forced arm is faster than the shipped decoder (its
     interval wholly below 1) at every count it would change, in a PR measured by the next
     release scaling;
   - **a move that costs more than 10% CPU** at any count it changes goes to Louis, not merged
     by the coordinator;
   - counts above the physical cores (SMT siblings) are reported only.
4. **Record** the bundle and the verdict in the release's CHANGELOG section, moved or not.

## 5b. Attribution: only when a step clears the bar

`docs/journal/release-attribution.md` is a ledger: the merge behind each jump in the release
scaling. Most releases add nothing to it. Louis, 16 November 2026: interpretation only where
there is something worth commenting on, never as a routine part of every release.

1. **The bar.** A release gets a row only if its step against the release before is at least 3%,
   with its 95% interval excluding zero, at some thread count of the release-scaling run. A
   step below the bar gets no row and no comment anywhere, the CHANGELOG included.
2. **The row.** One row per release that clears the bar, in the ledger's table:
   - the thread counts that moved, with their intervals;
   - the merge;
   - the mechanism class from the ledger's fixed list;
   - one line of evidence.

   No prose around it.
3. **Naming a merge.** A merge is named only:
   - when it is the only decoder merge between the two tags;
   - from CI's per-merge paired delta, at a thread count CI runs (1, 2 or 4);
   - or from a run that split the candidates.

   Otherwise the row lists the candidates and the class is "unexplained". A row may name the
   run that would split them; that run is an experiment like any other, with its rule posted
   before its data (RULES C1a).
4. **Hypotheses.** At most one line, labelled as unconfirmed, under the ledger's *Open*, with
   the run that would test it. A hypothesis is never put in a row, the CHANGELOG or a release
   report.
5. **The CHANGELOG** names the row, if there is one, and says nothing more about it.
6. **The reading.** The ledger's *What the ledger says* section changes only when a new row
   changes it, such as a new way the high thread counts moved.

## 5c. Worker counts: recalibrate when the cost model has moved

The trimmed worker counts (`trimmed_workers` in `src/parallel.rs`) were measured against a
particular kernel and its memory traffic. When that changes, the best count can move: with wide
admission (#559), 8 workers beat 7 within the new kernel (0.979), where the shipped trim uses 7.

1. **When:** queue a recalibration with the release scaling if either holds.
   - **The release changes what a worker costs:** the kernel's admission or work per block, its
     working set or scratch layout, or DRAM bytes per output byte. Examples are a new admission
     line, a new induction, or a store or table change.
   - **A measurement shows the knee has moved:** a frozen run reports a different best count
     within the new kernel, or the release scaling or crossover shows the shipped trim losing to
     a neighbouring count.
2. **The run:** a worker-count sweep on the i9 at the release's tag commit, in both placement
   modes. Pinned, one CPU on each of exactly T cores; and unpinned. The counts are those the trims
   cover (4, 6, 8 and 10 threads, and 12–20 unpinned), against the shipped trim at each. Its rule
   is posted before the data, and a change of trim is an adoption registration (C6), with the
   Researcher's review.
3. **Model first:** MONODROMY's contention model (records 45–55) gives each count's predicted
   best from the new kernel's measured traffic. Record the prediction beside the sweep, so the
   model is scored each time.
4. **Record** the verdict in the release's CHANGELOG section, changed or not.

## 6. After

1. **A CHANGELOG PR** with the release's notes. #210 is the model. It gives:
   - what changed;
   - the paired run's table and its CPU;
   - the stages;
   - the cost;
   - the caveats;
   - the release-scaling figures once they are in.
2. **The downstream bump:** mibz2 in lmmx/wikidata-pq's `reshape/Cargo.toml`, in a PR. It merges
   under that repository's bench rules.
3. **Report the release:**
   - to the maintainer, in one short message;
   - in the channel log, as a LOG on #145;
   - to the Observer, as a CHECKPOINT.
