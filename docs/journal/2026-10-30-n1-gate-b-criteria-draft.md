# Gate B, revised criteria (draft for the maintainer)

**Status:** draft text only. It would replace "A pre-registered Gate B, sketched" and its
decision rule in `2026-10-30-n1-construction-economics.md` §6. It authorises nothing: Gate B
is not authorised, and this draft doesn't propose running it. H-B′, the hypothesis it would
test, stands as written above it in §6.

**The figures it uses** (mibz2 references are to `origin/master` `ae942bd`):
- **the baselines** are the shipped build and walk stages, measured, each labelled below with
  its CI run;
- **the threshold formula** is model: C′ §5.4's parameters, restated in ns per decoded byte
  (`n1-reconcile/gate_ns.py`);
- **the element counts and the high-r blocks** are measured (census, 233 blocks);
- everything else is the rule itself.

---

**A pre-registered Gate B, sketched** (for the maintainer to decide; not proposed to run)

**What a pass means.** A pass is permission to build and measure the traversal (Gate C).
It is not evidence that N1 beats the shipped decoder. That needs Gate C's measured walk
against the shipped walk, on the same runners, at the decoder-stage boundary.

**What it builds:**
- **A standalone kernel bench** over stored L columns of the 233 full bench blocks, as P74's
  gate does with `Symbols`. The production decoder is untouched.
- **Arm N1, the first build: P^{2|4}.** Level 1 from L's runs, one doubling (1→2) with
  strict-interior popcount counting, then the s = 1 upgrade to length 4. It is the smallest
  kernel that decides H-B′.
- **Arm N2, the stronger candidate: P^{4|8}.** Arm N1's passes, plus 2→4 and the upgrade to
  length 8.
  - It runs only if arm N1 is not killed.
  - It shows whether c_e rises once the arrays leave L2. The median level-4 records are
    1.0 MB at 12 B.
- **P⁸ is not an arm.** Its full 4→8 doubling is modelled never to be the optimum on Wikidata
  (§4.1, §7.1). It goes forward only if Gate C measures c_L > 5·c_e.
- **Control:** the shipped `bwt::build` at the benched mibz2 revision (at `ae942bd`, the
  one-pass build of #114), in the same process, in alternating order, on the same blocks.
  - Its counts are the column's own, as in the decoder. So the one-pass path runs, not the
    two-pass fallback.
  - The bench records which path ran. Its result is void if any block fell back.

**Exactness, on every block, untimed:**
- every record's D and word equal the directly computed Ψ² and Ψ⁴ values, and Ψ⁸ for P^{4|8}'s
  upgraded records;
- the upgrade is also checked against a brute-force Ψ^{2a} over every row of each upgraded
  parent. Its correctness is so far argued, not checked;
- a scalar walker over the records, with caps from the same 512 starts, reproduces
  `bwt::walk`'s output byte for byte;
- **caps:** say whether they are fused into the last pass or applied as a separate post-split
  pass. Check the fused form against the post-split one on every block.

**Runners:**
- **The EPYC 7763 (Zen 3) is the continuity reference.** The decision is taken on it, because
  the stage table's history and the break-even model (§5) are anchored there.
- **The EPYC 9V74 (Zen 4) and 9V45 (Zen 5) are sensitivity points.** Each is judged against its
  own threshold, recomputed from that run's own b_T and w_T. Zen 5 is the tightest: its walk is
  the cheapest.
- **No CI runner stands in for the four-worker laptop target** (`Plan::Workers`, four blocks
  in flight on about 4 cores and 16 GB).
  - Gate B is one thread.
  - Four-copy effects (L3 sharing, bandwidth) are out of its scope. §5.4's four-copy column
    is not a Gate B criterion.
  - A CI run with four concurrent copies may be recorded as context. It doesn't decide.

**What it reports:**

Time is taken the way the bench's stage table takes it: each stage's fastest of 2 passes,
summed over the blocks. The table's c/B are those ns times its reported clock, so ns per
decoded byte is the primary unit, and c/B are given beside it.

- **Primary, whole corpus:** build_N1 = Σ N1 build time / Σ decoded bytes over the corpus, in
  ns/B. It includes N1's L scan. It is reported with the control's build_T and walk_T from the
  same run, on the same boundary.
  - The census's 233 full blocks hold 209,697,662 bytes of RLE1-coded text. The `wikidata`
    corpus decodes to 209,700,196 bytes. So the inverse BWT's output and the decoded bytes
    agree to 0.001% here (measured), and the boundary is the stage table's.
- **Per block:**
  - build ns/B and c/B, in total and per pass: L run scan, level-1 emit, counting, F prefix,
    merge, upgrade, directory, caps. The bitvector marking and counting work is reported
    apart from the record work;
  - the shipped build's ns/B for the same block, paired;
  - u, the upgrade's cycles per upgraded parent over the merge's cycles per element.
- **Distributions over the 233 blocks:** p10, p50, p90 and max of build ns/B and
  build_N1/build_T.
- **The named high-r blocks,** individually: census blocks 145 (r = 179,972), 90 (136,178),
  33 (134,923), 150 (132,011) and 24 (131,363). Give each one's r/n, build ns/B and its
  threshold.
- **Secondary, a diagnostic: c_e,** cycles per record element: (build − L-scan cycles) /
  elements. Elements are counted exactly from each block's census (runs, parents, children,
  upgrades).
  - Report the median block's c_e and the aggregate c_e (Σ cycles / Σ elements).
  - The aggregate element count is 1.16× the median block's, for every plan (measured). For
    P^{2|4} that is 0.321 against 0.277 per byte; for P^{4|8}, 0.538 against 0.462.
  - c_e explains a result; it doesn't decide one.
- **Memory, reported and not gated:**
  - the staged fallback rule's decision on every block (reconciliation §4.7): r after the L
    scan, m₂ (and m₄ for P^{4|8}) after each count, admitted or not. Also the
    arena's high-water, which must never exceed 4n, the T-vector's own;
  - the builder's measured peak and its retained high-water over the corpus, on one arena
    and on separate buffers, with L kept;
  - set against the shipped path's retained capacity on the same boundary;
  - accounted as `n1-reconcile/2026-10-30-n1-traffic-and-peak-reconciliation.md` §4 does.

**The threshold** (primary; recomputed from the same run's own shipped stages, in ns per
decoded byte):

- **pass:** build_N1 ≤ 0.75 · (b_T + w_T) − λ · (24 / 16.9) · w_T
- **kill (for the arm):** build_N1 ≥ 0.90 · (b_T + w_T) − λ · (16 / 16.9) · w_T

**The terms:**
- **b_T and w_T** are the shipped build and walk stages, whole Wikidata corpus, ns/B, from
  the same run.
- **λ** is the arm's lookups per byte (C′ §2, measured on 15 blocks): 0.296 for P^{2|4} and
  0.150 for P^{4|8}.
- **24/16.9 and 16/16.9** turn the shipped walk's measured time into N1's modelled walk. The
  shipped room walk is 16.9 instructions per byte, from the i9 profile, a pre-#114 figure (#114
  left the walk unchanged). N1 takes 24 instructions per lookup when conservative and 16 when
  optimistic (model, C′ §8).
- **The pass** is C′ §5.4's 0.75× of build + walk with the conservative walk. **The kill** is
  0.9× with the optimistic one. Both now cover the whole N1 build, L scan included, so the
  separate scan term goes.

**The baselines.** Every stage figure is from a CI bench comment: the `wikidata` blocks, one
thread, `bench/stages.py`. ns = c/B ÷ the reported clock.

| Runner | Source (bench comment, CI run) | Includes #114? | b_T, ns/B (c/B) | w_T, ns/B (c/B) | P^{2\|4}: pass ≤ / kill ≥, ns/B | P^{4\|8}: pass ≤ / kill ≥, ns/B |
|---|---|---|--:|--:|--:|--:|
| EPYC 7763, 3239 MHz | PR #114, head `f121097` (merged as `6a341f6`), run 40970005807 | **yes** (it is #114's head) | 0.667 (2.16) | 2.065 (6.69) | 1.18 / 1.88 | 1.61 / 2.17 |
| EPYC 7763, 3256 MHz | PR #116, base `7e7cce4` (master), run 40987964152 | **yes** | 0.651 (2.12) | 1.631 (5.31) | 1.03 / 1.60 | 1.36 / 1.82 |
| EPYC 9V74, 2872 MHz | PR #115, base `1291566` (master), run 40983226093 | **yes** | 0.609 (1.75) | 1.661 (4.77) | 1.00 / 1.58 | 1.35 / 1.81 |
| EPYC 9V45, 4020 MHz | PR #107, base `8ed29b0`, run 40969508814 | **no** (pre-#114) | 0.318 (1.28) | 0.709 (2.85) | 0.47 / 0.73 | 0.62 / 0.82 |

**What the baselines show:**
- **Post-#114 shipped figures exist for the 7763 and the 9V74.** No post-#114 stage table was
  found for the 9V45 (Zen 5) in the bench comments or the journals. For it, the threshold
  stays the formula above, in that run's own b_T and w_T. The pre-#114 row is a placeholder.
- **The construction note's §5.1 stage table, checked:**
  - its EPYC 7763 row (1.23 / 2.29 / 2.16 / 6.69 / 12.93 c/B) is PR #114's head column, so it
    already includes #114;
  - its Zen 4 (build 1.88, walk 7.01) and Zen 5 (1.52, 4.18) rows match no saved bench
    comment. Every saved 9V74 table gives a build of 1.56–1.75 c/B, and every 9V45 table
    0.83–1.50. Their source is untraced, and they shouldn't be used as baselines.
- **The same runner model varies from run to run.**
  - The 7763's walk was 6.69 c/B in #114's run and 5.31 in #116's. Across the saved 7763 runs
    it spans 3.5–7.6 c/B, at reported clocks of 2.45–3.26 GHz.
  - So the pass threshold for P^{2|4} moved from 1.18 to 1.03 ns/B between two post-#114 runs
    of the same runner model.
  - That is why the threshold must come from the same run, never from a reference table.
- **As a multiple of the shipped build** (post-#114 runs):
  - P^{2|4} may cost 1.58–1.77× b_T to pass;
  - P^{4|8} may cost 2.09–2.41× b_T.

**The secondary diagnostic's reference,** c_e at the pass threshold (median block /
aggregate; L scan 0.6 c/B; upgrades weighted u = 1.5):
- **P^{2|4}:**
  - #114's 7763: 10.2 / 8.8;
  - #116's 7763: 8.7 / 7.5;
  - #115's 9V74: 7.3 / 6.3.
- **P^{4|8}:** 9.1 / 7.8, 7.5 / 6.5 and 6.4 / 5.5, in the same order.

**The decision rule** (primary: the whole-corpus ns/B against the same run's threshold):
- **Pass to Gate C**, which means permission to measure the traversal and nothing more. All
  of these must hold:
  - on the EPYC 7763, the arm's whole-corpus build_N1 is at most that run's pass threshold,
    with its 95% interval below it;
  - the result is reported per block, so the share of blocks over threshold is known. The
    high-r blocks are expected over it, and that sends them to the graded selector (P^{2|4},
    P² or the T-vector), not to a fail.
- **Sensitivity:** the 9V74 (Zen 4) and 9V45 (Zen 5) are judged against their own runs'
  thresholds.
  - If either fails a 7763 pass, the pass is recorded as conditional, and Gate C must include
    that runner.
  - Zen 5 failing alone doesn't block Gate C. It is recorded as the named risk C′ §5.1
    already identifies.
- **Kill:** on the 7763, arm N1's (P^{2|4}) whole-corpus build_N1 is at or above its kill
  threshold, with its 95% interval above it.
  - For N1 at one thread as a whole, the measured aggregate c_e must also exceed the
    best-plan kill. That is P²'s: 0.90 · (b_T + w_T) − 0.500 · (16 / 16.9) · w_T, divided by
    P²'s 0.233 elements per byte (aggregate), on the same run's clock.
  - Then N1 at one thread closes as class II (the budget, measured). Arm N2 isn't run.
  - If a sensitivity runner passes where the 7763 kills, the result is recorded as
    inconclusive instead.
  - The four-copy route stays open only through its own pre-registered four-worker
    measurement on hardware like the laptop target. No CI runner stands in for it.
- **Otherwise inconclusive:** recorded, with no further one-thread N1 work. The plan choice
  follows the envelope (§4.1) with the measured c_e and u.
- **The interval:**
  - three paired rounds, as the bench uses;
  - 95% intervals by resampling rounds, and blocks within rounds;
  - the whole-corpus figure and the median block's, each with its own interval. The median
    block's is reported, not gated.
- **In every case:**
  - P⁸ goes forward only if Gate C measures c_L > 5·c_e;
  - a pass is a reason to build the walk, not a claim of a speed-up or of a memory saving.
    §4.6 of the reconciliation finds no resident-memory saving under mibz2's buffer
    discipline. The only memory-side benefit is the walk's smaller working set, which is
    Gate C's question.
