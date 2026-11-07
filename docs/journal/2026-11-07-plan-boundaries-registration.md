# 2026-11-07: `bench-plan-boundaries`, registered (diagnostic: registration only)

**What this is.** Registration 1 of the general plan policy
([`2026-11-07-general-plan-policy.md`](2026-11-07-general-plan-policy.md), §4 and §5),
written down before any run. Its recipe is `just bench-plan-boundaries`; its report is
`bench/plan_diagnostics.py boundaries`; it is frozen in `bench/plan-boundaries.frozen`.

**It is diagnostic (RULES C1).** Its objective is information: which rule's boundaries hold
on the i9. **No outcome of it can qualify a production change.** A default change needs its
own qualifying registration (the journal's registration 3), which waits on Louis's decision
about the A/A method (C6).

## The question

Rule A and rule B, as the journal writes them, predict different plans on the i9 at 8
threads and under CPU masks. They agree at 9 threads unmasked. The run measures those cells
directly, each plan forced.

## The machine and the inputs

- **The i9-10900K** only. The report checks `lscpu`: 1 socket, 10 cores, 2 threads a core, a
  20 MiB L3. The recipe refuses to run on any other topology, except as a dry run.
- **The bench's Wikidata sample** (`bench/wikidata.json`, built by `bench/wikidata.py`),
  SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.
- **The build:** the frozen commit's `examples/ab`.
- **The arms** are plans forced through `ab`'s `mibz2-plan-TxK` (plain) and `mibz2-plan-TxKp`
  (pipelined, with park and small: `parallel::PIPELINED_LEVERS`). Each runs in the default's
  window for its plan: `SLOTS_PER_THREAD` (4) blocks a team.
  - At 9 threads, 3 × 3 pipelined runs the same 12-block window as the 10-thread default
    (the journal's §1.4).
- **The controls** also run `mibz2`, today's default.

## The cells

Each cell is one `ab run`, 20 rounds, its arms interleaved by round. Runs are unpinned
inside the mask. A mask is set with `taskset -c` on `ab run`, so every run inherits it.

**How the masks are drawn.** They use the first c cores in the kernel's order (`ab env`'s
`core_cpus`).
- **One CPU a core** takes each core's lowest-numbered CPU, which emulates a c-core part
  without SMT.
- **Both CPUs** take every CPU of those cores, which emulates a c-core part with SMT.
- On the i9 (CPUs c and c + 10 are siblings):
  - one CPU a core of 8 cores is CPUs 0–7;
  - both CPUs of 8 cores is CPUs 0–7 and 10–17;
  - one CPU a core of 9 cores is CPUs 0–8;
  - one CPU a core of 6 cores is CPUs 0–5.

| cell | mask | threads | arms (the first is the reference) | role |
|---|---|--:|---|---|
| U8 | none | 8 | 4 × 2, 3 × 2, 2 × 4 p | the family |
| U9 | none | 9 | 4 × 2, 3 × 3 p | the family |
| M8 | 8 cores, one CPU a core | 8 | 4 × 2, 3 × 2 | the family |
| M8S | 8 cores, both CPUs | 16 | 4 × 2, 3 × 2 | the family |
| M9 | 9 cores, one CPU a core | 9 | 4 × 2, 3 × 2, 3 × 3 p | the family |
| C6 | none | 6 | `mibz2` (today's 3 × 2), 3 × 2 | control: reported, not judged |
| C6M | 6 cores, one CPU a core | 6 | `mibz2` (today's 3 × 2), 3 × 2 | control: reported, not judged |

The journal's masked N = 20 cells stay dropped, as it says.
- **The controls' arms:** the journal names the control cells but not their arms. Every rule
  picks 3 × 2 there, so the control is today's default against 3 × 2 forced: the same plan,
  reported with a plain 95% interval outside the family.

## Validity (any failure makes the run INVALID, and nothing is read)

- **The run itself:**
  - the registered 20 rounds in every cell, each arm's rounds 0–19 once each;
  - no other arm, thread count, corpus or cell.
- **Provenance:**
  - a clean tree;
  - no `NOT-FROZEN.txt` and no `DRY-RUN.txt`;
  - `commit.txt` is the checkout's head;
  - the ten-core topology;
  - the corpus by its SHA-256.
- **The bytes:** every run gives the same output hash and length, and none fails.
- **The masks:** each cell's mask (`masks.txt`) must be the registered shape on this
  machine's `core_cpus`, and every run's `allowed` (the CPUs `ab run` was given) must equal
  it. An unmasked cell's runs must have every CPU.
- **The plans:** every run's plan must be the registered one, read both from the threads that
  ran (`thread_cpu`) and from every thread the parent `ab` saw (`observed_threads`).
  - Leaders are `mibz2-N`, a plain team's helpers `mibz2-coop-N`, a pipelined team's
    `mibz2-coop-pN`.
  - So 4 × 2 is 4 leaders and 4 plain helpers, and 2 × 4 p is 2 leaders and 6 pipelined
    helpers.
  - **A cell whose observed plan differs from its registered plan makes the run INVALID.**

## The comparisons and the family

**The speed ratio** is 4 × 2's wall over the arm's, paired by round (above 1: the arm is
faster). Its geometric mean is reported with an interval.

**One multiplicity family for both rules: the 7 comparisons against 4 × 2.**

| # | cell | arm |
|--:|---|---|
| 1 | U8 | 3 × 2 |
| 2 | U8 | 2 × 4 p |
| 3 | U9 | 3 × 3 p |
| 4 | M8 | 3 × 2 |
| 5 | M8S | 3 × 2 |
| 6 | M9 | 3 × 2 |
| 7 | M9 | 3 × 3 p |

- **Bonferroni:** two-sided 95% overall, so each interval is at 1 − 0.05/14 a tail. Student's
  t at 19 degrees of freedom there is **3.0136**.
  - The journal rounds it to "≈ 3.1". The report uses the exact quantile its definition gives.
- **FASTER:** a point estimate of at least 1.03 and a lower bound above 1.
- **SLOWER:** an upper bound below 1.
- **UNRESOLVED:** anything else.

**The minimum detectable effect.**
- **The noise:** earlier runs' per-round log-ratio standard deviations were about
  0.027–0.043.
- **The power:** at 80% power, a true difference of about (3.01 + 0.86) × sd / √20 = 2.3–3.7%
  is detectable.
- **What that means:** with the 1.03 point bar, effects below about 3–4.5% will usually read
  UNRESOLVED.
- **The rounds:** the journal's 20 are kept. The run fits its time without trimming, so
  nothing is traded against the MDE.

**Reported, not judged:**
- each arm's CPU (median over the reference's);
- each arm's median frequency during its runs (`freq_mhz`: `ab run` samples the mean
  `scaling_cur_freq` of the run's CPUs every 100 ms);
- each CPU's `cpuinfo_max_freq` (`env.json`'s `max_freq_khz`). This also settles whether the
  i9's favoured cores report a higher maximum, which the topology reader's fixture assumes
  (#254).
- **Turbo:** both arms of a comparison share the mask, so the paired ratio is fair. Absolute
  rates across masks are not comparable.

## The outcome map (mechanical: `read_boundaries`)

- **The shared 9-thread cell (#3):**
  - FASTER: **supports both rules**;
  - SLOWER: **falsifies both**;
  - anything else: **leaves both open**.
- **Rule A at 8 threads:** **falsified** if any alternative (3 × 2, or 2 × 4 p) is FASTER at
  any of the three 8-thread cells (#1, #2, #4, #5). Otherwise **not falsified**.
- **Rule B at 8 threads:**
  - **supported** if 3 × 2 is FASTER at all three 8-thread cells (#1, #4, #5);
  - **falsified** only if 3 × 2's Bonferroni upper bound is below 1.03 at all three, so that
    its predicted gain is excluded;
  - otherwise **neither falsified nor supported**.
- **Neither resolved:** rule A not falsified, and rule B neither falsified nor supported.
  - **This outcome does not favour rule A.** It records that the run could not separate the
    rules at 8 threads.
  - The default stays 0.2.6's because of the envelope, not because of the run.
  - Any other combination is reported as each rule's own status.
- **The 9-core mask's 3 × 3 p (#7) is information only.**
  - It is in the family for multiplicity, and enters neither rule's reading.
  - **If FASTER,** the core-left-over assumption is unnecessary, and the 9-core shape becomes
    a candidate envelope entry by its own registration.
  - **Otherwise** nothing follows.
- **The 9-core mask's 3 × 2 (#6)** is in the family and reported. It enters none of the
  journal's listed readings, which use only the 8-thread cells for rule B.
- **What it does not show:** anything about the L3 extrapolation. No cell changes the L3.

## The run

- **How to run it:** `just submit bench-plan-boundaries`, with no arguments.
- **The arguments:** DRY (first, positional) and CORPUS (second, positional, DRY only) are
  for the untimed check of the recipe, which is never evidence.
- **How long:** about 15 minutes:
  - 320 runs (16 a round over 7 cells), at about 2.3 s each with the harness's 2 s settle;
  - the build and the sample.
- **What it produces:** one bundle, with:
  - `plan-boundaries.md` and `.json`;
  - `runs-CELL.jsonl`, `cells.txt`, `masks.txt`;
  - `env.json` (with `max_freq_khz`), `lscpu.txt`, `corpus.sha256`;
  - the provenance files.
