# 2026-11-07: `bench-plan-levels`, registered (diagnostic: registration only)

**What this is.** Registration 5 of the general plan policy
([`2026-11-07-general-plan-policy.md`](2026-11-07-general-plan-policy.md), §1.6 and §5),
written down before any run. Its recipe is `just bench-plan-levels`; its report is
`bench/plan_diagnostics.py levels`; it is frozen in `bench/plan-levels.frozen`, which also
pins the base.

**It is diagnostic (RULES C1).** Its objective is information: is there a level-1/2 slowdown
since 0.2.4, and does the decoder count explain it? **No outcome of it can qualify a
production change.**
- **If the slowdown is real,** the fix to register is a scheduling one (the journal's
  registration 5): a team whose block is below `MIN_COOP` lets its helpers take small blocks
  of their own. That fix gets its own registration.

## The question

The journal's §1.6 inference, for levels 1 and 2 only:
- **The decoder count:** at 10 or more threads, 0.2.5 and 0.2.6 decode such files with three
  one-core decoders, where 0.2.4 used four.
- **Why:** every block there is below `MIN_COOP`, so a team's helpers never start.
- **The size:** 0.2.6 may be about 17–24% slower than 0.2.4 there, that is at 0.76–0.83 of
  it.
- **The confidence:** moderate to high. The count is certain from the code, but the scale
  comes from one older build.

## The arms

| arm | what | build |
|---|---|---|
| `mibz2` | 0.2.6's default | the frozen commit; its default path is v0.2.6's, unchanged |
| `mibz2-plan-4x2` | 0.2.6 forced to 4 × 2, plain, in 4 × 2's default window (16 blocks) | the frozen commit |
| `base` (`mibz2@…/ab-base`) | 0.2.4's default (4 × 2 at 10 and 20 threads on the i9) | v0.2.4's release commit, **`1e644f9a327cd7282e79b40d9f870abc440e33e5`**, by its SHA, never the tag's name |

**What the forced arm separates:** the decoder count from everything else that changed
between the releases.

**The head's default path is v0.2.6's.** Against v0.2.6 (`a49038b0`), the head's `src/` only
adds the doc-hidden `parallel::general` module. `plan_with`, `Plan::for_threads` and
`StreamDecoder::defaulted` are untouched, which the equivalence test of #254 checks.

## The machine and the inputs

- **The i9-10900K** only. The report checks `lscpu`: 1 socket, 10 cores, 2 threads a core, a
  20 MiB L3.
- **The corpora:** the bench's Wikidata sample (SHA-256 `82f65ace…c2c29915`), decoded and
  recompressed as one stream by `ab recompress`.
  - **The tool:** libbzip2 1.0.8, the bundled source. It is deterministic, so these are the
    same bytes on any machine. The recipe refuses any other file.
  - **The output:** each level decodes to the same 209,700,196 bytes.

| level | file | bytes | SHA-256 |
|--:|---|--:|---|
| 1 | `wikidata-l1.bz2` | 19,018,084 | `2ae39adffbfbaab688d89b7597e57e79a551e25cba6db1c21f9599093d0d9e78` |
| 2 | `wikidata-l2.bz2` | 15,835,821 | `aa55dcfa36b98b5da4269893e512a9b86e1fba398a851a784490657435371f6a` |
| 3 | `wikidata-l3.bz2` | 14,417,759 | `380cbfda616222f8b26831653c845a0cd605d3962ce74ec527b445cd3ae7c5a5` |

- **Threads:** 10 and 20, unmasked.

## The cells

There are two `ab run`s, one at 10 threads and one at 20. Each covers the three arms and
three levels, 20 rounds, interleaved by round.

## Validity (any failure makes the run INVALID, and nothing is read)

- **As in registration 1:** rounds, provenance, topology, corpora by SHA-256, the same bytes,
  no mask.
- **The base:**
  - the base is v0.2.4's release commit by its SHA (`commits.txt`);
  - one base binary, built from its own checkout, the toolchain the same as the head's.
- **The plans:** every run's observed plan, from `thread_cpu` and `observed_threads`, must be
  the registered one.
  - **At levels 1 and 2,** no block reaches `MIN_COOP`, so no team ever starts its helpers.
    The registered plan there is the leaders alone:
    - 0.2.6's default runs 3 decoders;
    - 0.2.6 forced to 4 × 2 runs 4;
    - 0.2.4's default runs 4.
  - **At level 3,** the blocks cooperate:
    - 0.2.6's default runs 3 leaders and 6 pipelined helpers;
    - the 4 × 2 arms run 4 leaders and 4 plain helpers.
  - **A differing plan makes the run INVALID.** A team at level 1 that started helpers would
    be one.

## The comparisons

**The speed ratio** is the reference's wall over the arm's, paired by round. Each is made at
each level and thread count:

| name | reference | arm | what it shows |
|---|---|---|---|
| S | 0.2.4 | 0.2.6's default | the slowdown |
| K | 0.2.6's default | 0.2.6 forced to 4 × 2 | the decoder count |
| R | 0.2.4 | 0.2.6 forced to 4 × 2 | the same plan: everything else |

**The words**, as in registration 1:
- FASTER: a point estimate of at least 1.03 and a lower bound above 1;
- SLOWER: an upper bound below 1;
- UNRESOLVED: anything else.

**The families:**
- **Levels 1 and 2, the question:** 12 comparisons (2 levels × 2 thread counts × S, K, R).
  Bonferroni, two-sided 95% overall: each interval at 1 − 0.05/24 a tail, t₁₉ = **3.2550**.
- **Level 3, a separate question:** the journal's §1.6 inference does not cover levels 3–5,
  whose blocks cooperate. Its 6 comparisons are a family of their own: each interval at
  1 − 0.05/12 a tail, t₁₉ = 2.9439. They are reported, and enter no reading below.

**The minimum detectable effect.**
- **The question's family:** at 80% power, about (3.26 + 0.86) × sd / √20 = 2.5–4.0%, for
  per-round standard deviations of 0.027–0.043.
- **Level 3:** 2.3–3.7%.
- **Against the inference:** the inferred effect, 17–24%, is several times either. The 20
  rounds are kept.

**Reported, not judged:** CPU, the median frequency of each arm (`freq_mhz`), and each CPU's
`cpuinfo_max_freq` in `env.json`.

## The outcome map (mechanical: `read_levels`)

**Where it comes from.** The journal gives this registration's arms and their purpose, but no
decision rule. This one is stated here, before any run, in the journal's terms. It is read
for level 1 and for level 2, each on its own.

- **SLOWDOWN SHOWN:** S is SLOWER at both 10 and 20 threads, so 0.2.6 is slower than 0.2.4.
  Its source is then read from K and R at both thread counts:
  - **THE DECODER COUNT:**
    - K is FASTER at both: forcing four decoders on 0.2.6 is faster than its default;
    - and R is SLOWER at neither.
    - The fix to register is the scheduling one.
  - **OTHER CHANGES:**
    - R is SLOWER at both: the same 4 × 2 plan on 0.2.6 is still slower than 0.2.4;
    - and K is FASTER at neither.
  - **BOTH:** K is FASTER at both and R is SLOWER at both.
  - **NEITHER RESOLVED:** any other combination. The run could not separate the decoder count
    from the other changes.
- **SLOWDOWN NOT SHOWN:** anything else. The S intervals are reported against the inferred
  0.76–0.83. Where both lie wholly above 0.83, the report says so: the inferred size is then
  excluded at both thread counts.

## The run

- **How to run it:** `just submit bench-plan-levels`, with no arguments.
- **The arguments:** DRY (first, positional) and CORPUS (second, positional, DRY only) are
  for the untimed check of the recipe, which is never evidence.
- **How long:** about 20 minutes:
  - 360 runs (18 a round), at about 2.3 s each;
  - two builds (the head's and v0.2.4's, in a worktree);
  - three recompressions of about 30 s each, which are kept between attempts.
