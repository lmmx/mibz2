# The pipelined default: confirming pipe3 with both levers against v0.2.5

Registered before any run, on 2026-11-07. Modelled on the three-team policy's registration
(`2026-11-06-three-team-policy.md`), the precedent, whose rules this copies where they apply.

## The change

`StreamDecoder::new` (`src/parallel.rs`, `defaulted`), and so `BzReader::parallel`,
`parallel_with` and `decode_parallel`, builds the decoder for the policy's plan
(`Plan::for_threads`). This change adds one exception:
- **When:** the plan is exactly three teams of three (`PIPELINED_PLAN`). `plan_with` gives that
  plan at 10 or more threads on 10 physical cores and a 20 MiB L3, and nowhere else.
- **What:** the pipelined teams (`StreamDecoder::pipelined_levers(3, 3, ..)`) with park and
  small (`PIPELINED_LEVERS`), in the same window of 12 blocks.
- **Everything else:** the plain path, unchanged: below 10 threads, every other plan, workers,
  and the one-thread `decode` and `BzReader::new`. `planned(PIPELINED_PLAN, ..)` stays the plain
  three teams of three.

**Why this candidate.** The levers' registered run (`2026-11-07-pipeline-levers-result.md`,
`pipeline-levers/20261107T091926Z`) reported pipe3 with both levers at 3 × 3 on 10 threads at
1.072 [1.053, 1.091] against the plain team3@10, for about 5% more CPU and 8 MB more peak RSS.
That comparison was reported, not registered as a decision. Louis decided to ship it, after
one frozen confirmation run (`decisions.md`, 2026-11-07: "Confirm first, then ship"). This is
that run.

**What it reaches beyond what was measured.**
- **Thread counts:** only 10 was measured. At 11 or more threads the plan is the same 3 × 3, in
  the same window.
- **Machines:** any CPU with exactly 10 physical cores and a 20 MiB L3, as for v0.2.5's plan.
- **Workloads:** only the Wikidata sample at level 9. Other corpora and levels are covered for
  correctness, not speed.

## The evidence, as Louis decided

1. **CI:** the paired bench shows no regression. On its 4-core runners, both builds take the
   plain path, so this is the no-regression check only.
2. **Correctness tests on the changed path, without the hardware** (`tests/pipelined.rs`,
   with the plan forced through `parallel::with_plan`):
   - the default constructors at 3 × 3, byte for byte against libbzip2, the plain 3 × 3 and
     the default decoder, at every level and shape, on several streams;
   - corrupted, truncated and empty input failing as the plain path fails, at the same block,
     with the same bytes before it;
   - every other plan forced through the default constructor on the plain path;
   - cancellation and dropped readers.
3. **This registration:** one workstation A/B, below. Its 10-thread figure counts as the
   confirmation of `pipeline-levers/20261107T091926Z`'s pipe3-both@10 against team3@10.

## The run

`just submit bench-pipelined-default` on the i9, at this branch's frozen commit.
- **The commits:** `bench/pipelined-default.frozen` names the candidate and the base, set once,
  when this registration freezes. The base is v0.2.5's release commit,
  `e46f826379313b7a04d93005c349e2aada203582`, by its SHA, never by the tag's name. The recipe
  refuses to run:
  - if either is unset, or the base is not that commit;
  - if the checkout has uncommitted changes, or untracked files among the build and workload
    inputs;
  - if HEAD's build and workload inputs differ from the candidate's, including a file that
    appears or disappears: `src`, `Cargo.toml`, `Cargo.lock`, `.cargo`, `build.rs`,
    `rust-toolchain`, `rust-toolchain.toml`, `examples`, the corpus's builder and manifest
    (`bench/wikidata.py`, `bench/wikidata.json`), the judge (`bench/pipelined_default.py`),
    the bundler (`bench/bundle.sh`), the recipe and this registration;
  - if any `CARGO_*` or `RUST*` environment variable is set, except those that only locate
    the installation (`CARGO_HOME`, `RUSTUP_HOME`), shape Cargo's network or terminal
    (`CARGO_HTTP_*`, `CARGO_NET_*`, `CARGO_TERM_*`), or only affect panics and logs
    (`RUST_BACKTRACE`, `RUST_LOG`); or if `AB_CENSUS_MS` or `AB_SAMPLE_MS` is set, so the
    census reads every 5 ms and the child's sampler every 10 ms, as registered;
  - if the two checkouts select different toolchains (`rustc -vV` and `cargo -V`, asked in
    each);
  - if a Cargo configuration (`.cargo/config` or `.cargo/config.toml`) sits in any directory
    between the base checkout, which is inside this one, and this checkout's root: Cargo
    would apply it to the base build only;
  - if the base build's worktree is not at the base, or has changes or untracked files;
  - if `CORPUS=` is given without `DRY=1`.

  Each build runs in its own checkout, and Cargo's own build messages name each executable.
  The recipe copies them into the attempt's directory (`bin/`), and those copies are what
  run, so neither build can overwrite the other's. It records HEAD, the candidate and the
  base (`commits.txt`), and the sha256 of both `ab` copies, the corpus builder and the
  corpus, with the toolchain and the sha256 of every Cargo configuration both builds read
  from above this checkout and from Cargo's home (`inputs.txt`). The digests are a record
  of what ran, not a proof of what each was built from; the refusals above are what bind
  the builds to the frozen inputs.
- **The arms:** the head build's `mibz2` against the base build's (`mibz2@<base ab>`), the same
  `ab` mechanism as the precedent. Both use the default decoder, unpinned, as a caller gets it.
- **The grid:** 8, 9, 10, 11, 12, 16 and 20 threads, 20 rounds, paired by round, on the bench's
  Wikidata sample.
- **The plans each build must run,** read from every run's threads twice, and both must show
  the plan:
  - from those that ran, as the child reports them (`thread_cpu`, more than 2% of the
    busiest's CPU);
  - from every thread the child ran, busy or not, so a helper that sat idle still counts
    (`observed_threads`, the observed threads). The parent `ab` reads these names from `/proc`
    every 5 ms while each child runs, the same way for both builds, keeping each thread's
    latest name (a new thread carries its parent's name until it names itself); the base
    build's own `ab` records no such list. The census thread runs in the parent, not in the
    decoding process, and alike for both arms. Its cost is not measured separately.

  **Pipelined or plain, from the names.** Thread names alone could not tell a pipelined team
  from a plain one, so this change adds a mark: a pipelined team's helpers are named
  `mibz2-coop-pN` (`coop::Pool::pipelined`), a plain team's stay `mibz2-coop-N`. Both start
  `mibz2-coop`, so every earlier judge counts both as helpers. The judge reads a plan as
  (leaders, plain helpers, pipelined helpers):

  | threads | base | head |
  |---|---|---|
  | 8, 9 | 4 × 2, plain | 4 × 2, plain |
  | 10, 11, 12, 16, 20 | 3 × 3, plain | 3 × 3, pipelined |

- **The window** is not observed: it follows from the plan. Each build's window is
  `floor(slots × teams / threads)` for a team plan (`window` in `src/parallel.rs`, the same at
  both commits), and `ab` always requests 4 slots a thread. So the observed plan fixes it, at
  every registered thread count: **16 blocks for 4 × 2 and 12 for 3 × 3,** in both builds.
  The pipelined team takes the plain team's window unchanged.
- **The output:** one directory per attempt, its console log included, packed into one tarball
  however the attempt ends. Every attempt states one outcome, in `outcome.txt` and on its
  last line:
  - `VALID PASS`, `VALID FAIL` or `INVALID: <the problems>`, read from the report's own
    fields;
  - `REFUSED: <why>`, before anything ran;
  - `REPORT ERROR`, when the judge wrote no report;
  - `INCOMPLETE`, when the attempt stopped before the run finished.

  The recipe exits 0 for a valid verdict, 1 for `INVALID`, and nonzero for anything that
  kept the run from a verdict.

  A broken runs file (truncated JSON, a null row or arm) is reported `INVALID`, never a
  crash.
- **`DRY=1`** is the untimed check of the recipe itself: one round, no settling, and
  optionally `CORPUS=` a .bz2 in place of the built sample. It is never evidence: its one round
  makes every cell incomplete, so the judge never calls it valid.

## The rule

`bench/pipelined_default.py`, fixed with this registration. Speed is the base build's wall over
the head's, the geometric mean of the 20 rounds' ratios.

- **Validity:**
  - only the registered rows: the Wikidata sample, the seven thread counts, the two arms and one
    base binary;
  - every run with a hash, a positive output length, and finite, positive wall, CPU and RSS;
  - 20 rounds a cell, each once, no failed run;
  - one output hash and one length across every run;
  - every run's plan as in the table, plain or pipelined, in both the busy threads and the full
    census.
- **Identity checks, at 8 and 9 threads:** both builds run the same plain plan, so head and base
  should match. Their plans count toward validity. Their speed is reported, with #220's ±2% flag
  (a 95% interval wholly beyond ±2%), and **does not count toward the verdict**.
- **Primary, at 10 threads:** speed at least 1.03, with its two-sided 95% lower bound above
  1.00.
- **No harm, at 11, 12, 16 and 20 threads:** each head-over-base wall ratio's one-sided upper
  bound at most 1.02. The bound is Bonferroni over the four: t at 1 − 0.05/4, 19 degrees of
  freedom. An interval too wide to show it is inconclusive, and fails.
- **CPU and peak RSS:** the head's median over the base's, at every cell, **reported, not
  judged.**
- **A run's verdict** is the `pass` field of its `pipelined-default.json`; it passes only if
  every predicate holds.
- **Admission:** one slot, as decided ("one frozen A/B"): one run of the recipe at the frozen
  commits. No pooling, no switching cells.
- **Replacement:** at most one in the whole experiment, so at most two attempts. Only a run
  that is not valid (`valid` false: a structural defect, such as a failed run, a missing round
  or a wrong plan) may be replaced, and its replacement takes its slot. A valid run that fails
  is never replaced: it fails the registration.
- **Every attempt is kept,** replaced or not, with its tarball, and reported.

## The prediction

- **At 10 threads:** about 1.07–1.08. The levers' run measured pipe3-both@10 at 1.072 against
  team3@10, and its medians against the shipped default (mibz2@10, v0.2.5's plan) were 0.142 s
  against 0.154 s. Its noise control put team3@10 and mibz2@10 at 1.007.
- **At 11 to 20 threads:** about the same, since the plan and window are the same.
- **CPU:** about 1.05 to 1.06 (the paired ratio 0.951 in the levers' run; its medians 1.030 s
  against 0.973 s).
- **Peak memory:** about 1.12 (75 MB against 67 MB).
- **At 8 and 9 threads:** no change; the plain path in both builds.

## What each outcome would mean

- **The run passes:** the change merges, with this PR's CI green, and ships.
- **It fails:** it doesn't merge. The report names the failed predicate, and the change stays
  unmerged until a new registration.

## What a pass would not show

- **Unmeasured thread counts:** 13 to 15, 17 to 19, and above 20. The plan there is the same
  3 × 3, so they are expected to behave like their neighbours, but nothing measures them.
- **Other machines:** only the i9. A CPU with the same core count and L3 can differ in cache
  layout, NUMA, affinity or a CPU quota, and would still get the pipelined teams.
- **Other uses:** a custom window, small inputs, other workload shapes, and a machine busy with
  other work are not measured.
- **The cause:** the run can't separate the overlap's gain from the leader's extra walking, as
  the levers' run couldn't.
- **The identity checks** screen for drift between the builds; they do not show the builds
  equivalent at 8 and 9 threads, and they don't count toward the verdict.
- **The intervals** are conditional on the model: paired rounds that are independent of each
  other, and a normal log-ratio.
- **A measured 1.03** does not show that the true gain is at least 3%; the lower bound above 1.00
  shows only that it is a gain.

## Frozen

In `bench/pipelined-default.frozen`, in a commit of its own that changes nothing else: the
candidate (the commit whose build and workload inputs, the recipe's `measured` list, HEAD must
match) and the base, v0.2.5's release commit `e46f826379313b7a04d93005c349e2aada203582`.
Nothing in this registration changes from there; a change needs a new registration.

**The run:** Louis runs it on the i9 under "The rule" above.
