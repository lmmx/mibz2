# The three-team policy: admitting 3 × 3 on the ten-core topology

Registered before any run, on 2026-11-06.

## The change

`Plan::for_threads` (`src/parallel.rs`, `plan_with`) chooses the cooperative decoder's plan.
Until now it used teams of 2, as many as keep the live tables near 0.7 of the L3, which is
four teams on the i9-10900K. This change adds one exception:
- **When:** 10 physical cores, a 20 MiB L3 (as `l3_bytes` reads it), and 10 or more threads
  asked for.
- **What:** three teams of three, 9 decoding threads, in a default window of 12 blocks.
- **Everything else:** the old rule, unchanged, including 8 and 9 threads on the i9, which
  stay at four teams of two.

**Why 3 × 3.** #220 qualified 3 × 3 at 10 threads and 3 × 4 at 12 against four teams of two
at 8 (`2026-11-06-three-tables.md`, two runs each). It ranked neither. The Optimiser's design
picked 3 × 3 (#145, its REVIEW 6693638662): about the same speed as 3 × 4, with about 15% less
CPU.

**What it reaches beyond what was measured.**
- **Thread counts:** only 10 was measured. At 11 or more threads the policy also picks 3 × 3,
  which leaves threads unused that the old plan didn't use either.
- **Machines:** any CPU with exactly 10 physical cores and a 20 MiB L3 gets the change, not
  only the i9-10900K. The planner has no CPU model to check.
- **Workloads:** only the Wikidata sample at level 9. Other corpora and levels are covered for
  correctness, not speed.

## The evidence, as Louis decided (`decisions.md`, 2026-11-06)

1. **CI:** the paired bench shows no regression. On its 4-core runners, both builds take the
   old path.
2. **Correctness tests on the changed plan, without the hardware:**
   - the plan table with the topology injected: every row, both boundaries, the budget;
   - three teams of three, through the planned decoder, byte for byte against libbzip2 at
     every level and shape, on several streams;
   - corrupted, truncated and empty input failing as four teams of two fail;
   - windows too small for a block a team.
3. **This registration:** one workstation A/B and its confirmation, below.

## The run

`just bench-three-team-policy` on the i9, at this branch's frozen commit.
- **The commits:** `bench/three-team-policy.frozen` names the candidate and the base, set
  once, when this registration freezes. The recipe refuses to run:
  - if either is unset;
  - if the checkout has uncommitted changes, or untracked files among the build and workload
    inputs;
  - if HEAD's build and workload inputs differ from the candidate's, including a file that
    appears or disappears: `src`, `Cargo.toml`, `Cargo.lock`, `.cargo`, `build.rs`,
    `rust-toolchain`, `rust-toolchain.toml`, `examples`, the corpus's builder and manifest
    (`bench/wikidata.py`, `bench/wikidata.json`), the judge and the recipe;
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
  - if the base build's worktree is not at the base, or has changes or untracked files.

  Each build runs in its own checkout, and Cargo's own build messages name each executable.
  The recipe copies them into the attempt's directory (`bin/`), and those copies are what
  run, so neither build can overwrite the other's. It records HEAD, the candidate and the
  base (`commits.txt`), and the sha256 of both `ab` copies, the corpus builder and the
  corpus, with the toolchain and the sha256 of every Cargo configuration both builds read
  from above this checkout and from Cargo's home (`inputs.txt`). The digests are a record
  of what ran, not a proof of what each was built from; the refusals above are what bind
  the builds to the frozen inputs.
- **The arms:** the head build's `mibz2` against the base build's (`mibz2@<base ab>`). Both use
  the default plan, unpinned, as a caller gets it.
- **The grid:** 8, 9, 10, 11, 12, 16 and 20 threads, 20 rounds, paired by round, on the bench's
  Wikidata sample.
- **The plans each build must run,** read from every run's threads twice, and both must show
  the plan:
  - from those that ran, as the child reports them (`thread_cpu`, more than 2% of the
    busiest's CPU);
  - from every thread the child ran, busy or not, so a helper that sat idle still counts
    (`observed_threads`, the observed threads). The parent `ab` reads these names from `/proc`
    every 5 ms while each
    child runs, the same way for both builds, keeping each thread's latest name (a new
    thread carries its parent's name until it names itself); the base build's own `ab`
    records no such list.
    The census thread runs in the parent, not in the decoding process, and alike for both
    arms. Its cost is not measured separately.

  | threads | base | head |
  |---|---|---|
  | 8, 9 | 4 × 2 | 4 × 2 |
  | 10, 11, 12, 16, 20 | 4 × 2 | 3 × 3 |

- **The window** is not observed: it follows from the plan. Each build's window is
  `floor(slots × teams / threads)` for a team plan (`window` in `src/parallel.rs`, at its
  frozen commit), and `ab` always requests 4 slots a thread. So the observed plan fixes it,
  at every registered thread count: **16 blocks for 4 × 2 and 12 for 3 × 3.** The windows
  differ by design: they are part of the change being tested. `the_ten_core_exception`
  tests the 3 × 3 values.
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

## The rule

`bench/policy_ab.py`, fixed with this registration. Speed is the base build's wall over the
head's, the geometric mean of the 20 rounds' ratios.

- **Validity:**
  - only the registered rows: the Wikidata sample, the seven thread counts, the two arms and one
    base binary;
  - every run with a hash, a positive output length, and finite, positive wall, CPU and RSS;
  - 20 rounds a cell, each once, no failed run;
  - one output hash and one length across every run;
  - every run's plan as in the table, in both the busy threads and the full census.
- **Controls, at 8 and 9 threads:** as #220's control, flagged when a speed ratio's 95% interval
  lies wholly beyond ±2%. A flagged run fails.
- **Primary, at 10 threads:** speed at least 1.03, with its two-sided 95% lower bound above
  1.00. The head's median CPU-seconds at most 1.5 times the base's, and its median peak RSS at
  most 1.15 times.
- **No harm, at 11, 12, 16 and 20 threads:** each head-over-base wall ratio's one-sided upper
  bound at most 1.02. The bound is Bonferroni over the four: t at 1 − 0.05/4, 19 degrees of
  freedom. An interval too wide to show it is inconclusive, and fails.
- **A run's verdict** is the `pass` field of its `policy-ab.json`; it passes only if every
  predicate holds.
- **Admission:** two slots, the first run and its confirmation, each a run of the same recipe
  at the frozen commits. The policy is admitted only if both pass. No pooling, no switching
  cells.
- **Replacement:** at most one in the whole experiment, so at most three attempts. Only a run
  that is not valid (`valid` false: a structural defect, such as a failed run, a missing round
  or a wrong plan) may be replaced, and its replacement takes its slot. A valid run that fails
  is never replaced: it fails the registration.
- **Every attempt is kept,** replaced or not, with its tarball, and reported.

## The prediction

- **At 10 threads:** about 1.08. #220 measured 1.076 and 1.087 for 3 × 3 at 10 against 4 × 2 at
  8. The old default at 10 runs 4 × 2 too, and #220 measured `mibz2@10` at 0.999 against
  `mibz2@8`.
- **At 11 to 20 threads:** about the same, since the plan and window are the same.
- **CPU:** about 0.9.
- **Peak memory:** about 0.82.
- **At 8 and 9 threads:** no change.

## What each outcome would mean

- **Both runs pass:** the policy merges, with this PR's CI green.
- **Either fails:** it doesn't merge. The report names the failed predicate, and the change
  stays unmerged until a new registration.

## What a pass would not show

- **Unmeasured thread counts:** 13 to 15, 17 to 19, and above 20. The plan there is the same
  3 × 3, so they are expected to behave like their neighbours, but nothing measures them.
- **Other machines:** only the i9. A CPU with the same core count and L3 can differ in cache
  layout, NUMA, affinity or a CPU quota, and the exception would still pick 3 × 3 there.
- **Other uses:** a custom window, small inputs, other workload shapes, and a machine busy with
  other work are not measured.
- **The controls** screen for drift between the builds; they do not show the builds equivalent
  at 8 and 9 threads.
- **The intervals** are conditional on the model: paired rounds that are independent of each
  other, and a normal log-ratio.
- **A measured 1.03** does not show that the true gain is at least 3%; the lower bound above 1.00
  shows only that it is a gain.

## Frozen

On 2026-11-06, in `bench/three-team-policy.frozen`:
- **candidate** `3b780fb3d70efc8573bed1485254a5579e3a82c7`: its build and workload inputs are what the recipe accepts;
- **base** `8b111867ff1caaca727a0d35e160786c499c4849`, the candidate's merge base with master (#237's merge). Its source is
  identical to `818d521c`, the base the reviews inspected; master has moved since only in
  files outside the build and workload inputs.

**Reviewed:**
- the Guardian: D040 (the recipe cleared at `ce3fa6d2`) and D041 (held at `87ddcf26`);
- the Researcher: REVIEW 6695772196 (cleared at `87ddcf26`).

`3b780fb3` differs from `87ddcf26` only in the configuration inventory's loop, which keeps a
path with a space whole: the nonblocking point both raised.

**The run:** Louis runs it on the i9, and its confirmation after it, under "The rule" above.
Nothing in this registration changes from here; a change needs a new registration.

## Result

Not yet run.
