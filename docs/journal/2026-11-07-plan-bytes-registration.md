# 2026-11-07: `bench-plan-bytes`, registered (diagnostic: registration only)

**What this is.** Registration 2 of the general plan policy
([`2026-11-07-general-plan-policy.md`](2026-11-07-general-plan-policy.md), §4 "What block
size can test" and §5), written down before any run. Its recipe is `just bench-plan-bytes`;
its report is `bench/plan_diagnostics.py bytes`; it is frozen in `bench/plan-bytes.frozen`.

**It is diagnostic (RULES C1).** Its objective is information: does the table's size move
the three-team advantage? **No outcome of it can qualify a production change.**

## The question

Rule A's extrapolation to other caches rests on one premise: what limits the team count is
table bytes against the L3. The i9's L3 can't change, but the table size can, by level.
- **The tables:** a level-6 block has a 2.4 MB table, so four of them hold 9.6 MB, less than
  the 10.8 MB of three full tables.
- **If the premise holds,** plain 3 × 3 against 4 × 2 at 10 threads should fall from about
  1.08 at level 9 to about 1.00 or below at level 6. Level 7, at 11.2 MB for four tables,
  should sit near the boundary.
- **If cores or synchronisation are the limit,** the ratio should stay near level 9's.

**What it cannot show.** This is the closest the i9 can come to the premise. Nothing the i9
can run is evidence for the L3 extrapolation itself.

**Confounds.** Smaller blocks also mean more blocks, more per-block overhead and a different
share below `MIN_COOP`. No block at levels 6 and 7 is below it.

## The machine and the inputs

- **The i9-10900K** only. The report checks `lscpu`: 1 socket, 10 cores, 2 threads a core, a
  20 MiB L3. The recipe refuses another topology, except as a dry run.
- **The corpora:** the bench's Wikidata sample (SHA-256 `82f65ace…c2c29915`), decoded and
  compressed again as one stream at each level.
  - **The tool:** `ab recompress`, which uses libbzip2 1.0.8, the bzip2 crate's bundled
    source pinned by `Cargo.lock`. bzip2 is deterministic, so the files are the same bytes
    on any machine. The recipe refuses any other file.
  - **The output:** each level decodes to the same 209,700,196 bytes.

| level | file | bytes | SHA-256 |
|--:|---|--:|---|
| 6 | `wikidata-l6.bz2` | 12,640,513 | `f3bbccf5d914280a40c27b311a2694893329937251617e98dbba280594158502` |
| 7 | `wikidata-l7.bz2` | 12,345,892 | `58c8897f3da919c95001a8f60f823d6b9652405f1baf8cd7bcb2352ea4dd68a1` |
| 9 | `wikidata-l9.bz2` | 11,863,758 | `817e6d89f4ab7df83d91d6d356ea8aa4b0d0e7af9ea11e21541a08c9d06adc32` |

- **The build:** the frozen commit's `examples/ab`.
- **The arms**, forced, each in the default's window for its plan (4 blocks a team):
  - **plain 3 × 3:** `mibz2-plan-3x3`;
  - **3 × 3 pipelined with park and small:** `mibz2-plan-3x3p`;
  - **4 × 2:** `mibz2-plan-4x2`, the reference.

## The cells

One `ab run`: N = 10 unmasked, the three levels and the three arms, 20 rounds, interleaved
by round.

## Validity (any failure makes the run INVALID, and nothing is read)

The same checks as `bench-plan-boundaries`, with these corpora:
- **The run itself:** the registered 20 rounds, each arm's rounds 0–19 once each on each
  level, and no other arm, thread count, corpus or cell.
- **Provenance:** a clean tree, no `NOT-FROZEN.txt` and no `DRY-RUN.txt`, `commit.txt` the
  head, and the ten-core topology.
- **The corpora:** each by the SHA-256 above.
- **The bytes:** every run gives the same bytes, and none fails.
- **The mask:** none; every run has every CPU.
- **The plans:** every run's observed plan, from `thread_cpu` and `observed_threads`, must be
  the registered one:
  - 3 leaders and 6 plain helpers;
  - 3 leaders and 6 pipelined helpers;
  - 4 leaders and 4 plain helpers.
  - **A differing plan makes the run INVALID.**

## The comparisons and the family

**The speed ratio** is 4 × 2's wall over the arm's, paired by round.

**The family:** 6 comparisons. At each of levels 6, 7 and 9, plain 3 × 3 and 3 × 3 pipelined
are each compared against 4 × 2.
- **Bonferroni:** two-sided 95% overall, so each interval is at 1 − 0.05/12 a tail, t₁₉ =
  **2.9439**.
- **The words**, as in registration 1:
  - FASTER: a point estimate of at least 1.03 and a lower bound above 1;
  - SLOWER: an upper bound below 1;
  - UNRESOLVED: anything else.

**The minimum detectable effect.** At 80% power, about (2.94 + 0.86) × sd / √20 = 2.3–3.7%,
for per-round standard deviations of 0.027–0.043. The 20 rounds are kept.

**Reported, not judged:** CPU and the median frequency of each arm (`freq_mhz`), and each
CPU's `cpuinfo_max_freq` in `env.json`.

## The decision rule (pre-stated in the journal; mechanical: `read_bytes`)

It is read on plain 3 × 3 against 4 × 2. The pipelined arm is reported.
- **Validity of the reading:** the run reads the premise only if level 9 reproduces the
  advantage, with a lower bound above 1. Otherwise it is **INCONCLUSIVE**.
- **The premise is CONTRADICTED** if the level-6 ratio's lower bound stays above 1.03. The
  three-team advantage would then survive tables that fit four to the L3.
- **The premise is SUPPORTED** if both hold:
  - the level-6 ratio's upper bound is below 1.03;
  - its point estimate is below level 9's by more than both intervals' half-widths combined.
    The half-widths are on the log scale: log g₉ − log g₆ > h₆ + h₉, each h being t × se.
- **UNRESOLVED:** anything else.
- **Level 7** is reported as the boundary, and not judged.

**What follows:**
- **A contradiction** would count against rule A's L3 scaling, and against rule B's curve,
  before either reaches another machine.
- **Support** would not validate the extrapolation. It would only fail to refute its premise
  on one machine.

## The run

- **How to run it:** `just submit bench-plan-bytes`, with no arguments.
- **The arguments:** DRY (first, positional) and CORPUS (second, positional, DRY only) are
  for the untimed check of the recipe, which is never evidence.
- **How long:** about 12 minutes:
  - 180 runs (9 a round), at about 2.3 s each;
  - the build;
  - three recompressions of about 30 s each, which are kept between attempts.
