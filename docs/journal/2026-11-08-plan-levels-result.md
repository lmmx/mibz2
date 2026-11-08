# Levels 1–3: 0.2.6 against 0.2.4, and 0.2.6 forced to 4 × 2

8 November 2026. This is registration 5 of `2026-11-07-general-plan-policy.md` (§1.6). It was
frozen at 9f13b4e8 and run at a5dcaebd (#254). It is a diagnostic (RULES C1).

## The run

- **Bundle:** `plan-levels/20261108T094232Z.tar.gz` on `evidence` at 71e8dc17.
  - Its SHA-256 is 94802eb3383f1c7bed581bd62a1f75e63d3c59e68d4a4c916eed986d62a8c457, which
    matches the notice on #145.
  - It was run with `just submit`, from the workstation queue.
- **The builds:** the head a5dcaebd and the base 1e644f9a (0.2.4). The binaries' SHA-256 are in
  the notice.
- **VALID:** every check passed.

## The comparisons

- **S** is 0.2.6's default against 0.2.4's.
- **K** is 0.2.6 forced to 4 × 2 against 0.2.6's default.
- **R** is 0.2.6 forced to 4 × 2 against 0.2.4's default: the same plan, so everything else that
  changed between the releases.

Speed is the reference's wall over the arm's.

| level | threads | S | K | R |
|--:|--:|---|---|---|
| 1 | 10 | 0.763 [0.752, 0.775] SLOWER | 1.320 [1.301, 1.338] FASTER | 1.008 [0.997, 1.018] |
| 1 | 20 | 0.765 [0.758, 0.773] SLOWER | 1.317 [1.302, 1.332] FASTER | 1.008 [0.995, 1.020] |
| 2 | 10 | 0.776 [0.759, 0.794] SLOWER | 1.297 [1.266, 1.329] FASTER | 1.007 [0.990, 1.025] |
| 2 | 20 | 0.772 [0.751, 0.794] SLOWER | 1.307 [1.276, 1.338] FASTER | 1.009 [0.986, 1.033] |
| 3 | 10 | 1.047 [1.015, 1.081] FASTER | 0.962 [0.933, 0.992] SLOWER | 1.007 [0.985, 1.030] |
| 3 | 20 | 1.067 [1.030, 1.106] FASTER | 0.933 [0.900, 0.967] SLOWER | 0.996 [0.979, 1.013] |

Levels 1 and 2 form a family of 12 (Bonferroni over 24 tails). Level 3 is its own family of 6,
in no reading.

## The registered readings

**At levels 1 and 2: SLOWDOWN SHOWN, from the decoder count.** The fix to register is the
scheduling one.
- 0.2.6's default is 22–24% slower than 0.2.4 at 10 and 20 threads, at the 0.76–0.83 the design
  journal inferred (§1.6).
- All of the loss is the plan. Forced to 4 × 2, 0.2.6 recovers it fully (K = 1.30–1.32), and on
  the same plan the releases are indistinguishable (R = 1.007–1.009, each unresolved).

**At level 3 (information only),** 0.2.6's plan is 5–7% faster than 0.2.4's, and 4 × 2 is slower.
The crossover between the two plans lies between levels 2 and 3.

## What it means for users

- **0.2.6 is about 23% slower than 0.2.4 on files compressed at bzip2 levels 1–2.** This holds
  on the ten-core topology where its pipelined default applies, at 10 threads or more.
- **Elsewhere, 0.2.6 behaves exactly as 0.2.4:** at levels 3–9 it is faster or unchanged, and on
  any other topology the plan is unchanged.
- **Wikidata's dumps are level 9, so they are unaffected.** Level 1–2 files exist, from
  `bzip2 -1` or `--fast`.

## What follows

1. **The fix, as registered:** a scheduling change. A team whose block is below `MIN_COOP` lets
   its helpers take small blocks of their own.
   - The decision is made per block, so a mixed-level file needs no header read and no gate,
     unlike #115.
   - Its cost is about 15 MB more peak memory at 3 × 3, from a `Front` and a `Solo` per helper, and
     a window that scales with decoders.
   - It needs its own prototype, registration and qualifying run.
2. **The stopgap:** a level-aware plan choice (4 × 2 at levels 1–2) would recover the loss
   outright.
   - The evidence for it here is a diagnostic's, so it can't qualify a change (C1).
   - It is a header gate, of the kind that failed its gates in #115 on mixed-level files.
   - It needs a qualifying run, and the qualifying A/A method waits on Louis (C6).
3. **The README and the CHANGELOG** should state the level 1–2 exposure for 0.2.6 now, as a
   measured fact. That is a docs change.
