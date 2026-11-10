# D4 rescue: the failed constructor is not an impossibility result

Inbox6759188202; pinned specs a2da49638ac54d7763de1c167bd054f67ffdc763. Source561665b61ea61ba8027e4e15068322d938ea0077; registration e90213e5; result PR332 head ae9a1cc005c52f9180c12497391bdee9da980f64.

Checkpoint: F2 remains FAIL. The phase attribution totals14.21, not registered14.45; retain the .24 difference. Removing run acquisition alone leaves11.49 in the attribution binary, or11.73 as an illustrative subtraction from the registered count. Neither passes. Even30Ir/piece with fixed5.46 gives10.22 on the counted piece densities.

Source exposes a possible within-composition rescue: P2 images are not monotone per single byte, but are monotone within a fixed two-byte word. A cursor per word can replace the repeated level-2 predecessor directory lookups. Initialising those cursors, handling stops without the directory and storing the front-end run list are paid work, not free savings. The alternative is worth a complete prospective count price, not a claim of ≤7.35.

The .86MB walk table excludes2.6–4.4MB retained construction scratch and excess record capacity. A table-only load-law forecast therefore omits the main new risk at7–10 workers. Full report follows; no implementation or timing.


## Evidence scope and corrected arithmetic

Read the registration at e90213e5 (blob ebf8c3974b4132bfbd4719dc20457005feeed6ca), complete module and count harness at561665b61ea61ba8027e4e15068322d938ea0077, and its result journal. The published counts/test results are reported evidence, not independently replayed raw callgrind files or rerun safety tests. Source inspection corroborates the loops and their boundaries. No native build, execution, timing or merge was performed.

The registered14.45 is the verdict. The phase binary's14.21 differs by.24 because of compilation context. Phase differences are an attribution, not exact removable credits in the registered binary. The rounded build+walk figures can sum to22.18 while the journal reports22.17 from underlying counts; do not manufacture precision by recombining rounded cells.

On the counted blocks the inferred output densities are q2=2.96/41.8≈.07081 and q4=5.79/65.9≈.08786 piece-steps per row. These differ from the all-corpus .0881 and .1088. Thus:
- fixed terms5.46 leave1.89 for both compositions;
- at30Ir per piece, those compositions cost4.7602, total10.2202;
- with unchanged fixed terms, weighted average composition cost would need ≤11.91Ir/piece, not30;
- dropping run scan2.72 and directory1.21 as idealised credits leaves2.96+5.79+1+.54=10.29 before replacements;
- if those two terms disappear AND both loops reach30, total6.2902, leaving1.0598 for replacements, or.8198 if the.24 compilation-context difference persists.

These are necessary budgets, not achieved savings. They reject scan-only rescue and simple30-Ir loop tuning. They do not prove every composition schedule impossible. “The rescue must change kind” is stronger than the evidence permits.

## Ranked constructions and their cheapest falsifiers

### 1. Run-fed input plus two-byte cursor composition: retain as a narrowly priced proposal

For a P2 piece, source stores w=c0|(c1<<8). LF is order-preserving within one L-byte class. Therefore LF² is order-preserving within a fixed ordered pair(c0,c1): after the first LF, all rows in that class retain source order; after the second, they remain in the same c1 class. Their image intervals lie consecutively in the F-order two-symbol bucket, ordered by(c1,c0), i.e. ascending numeric w.

This supports one monotone covering-piece cursor per active word, not one per byte. The module's §8.4 objection to per-byte cursors does not exclude this. Replacing each second-composition piece(bits2,rank2,b) lookup with cur[w] avoids a repeated rank/popcount predecessor chain. Parent/covering bounds, min, delta/word packing, record/bitmap writes, singleton branches and all loop exits remain. It is not a30-Ir proof.

A concrete preparation outline, for review rather than implementation:
1. Accumulate each word's total image length from P2 lengths.
2. Prefix those counts in numeric w order; merge their bucket beginnings against the sorted P2 starts to initialise covering cursors.
3. Process P2 in source order, advancing only cur[w] through the covering intervals. Prove the cursor remains at the first covering piece at each parent boundary, including exact-end boundaries.
4. Retain the same final records, stop semantics and validation.

Charge preparation: a dense65536-entry u32 array is262144 bytes; one array can hold counts then cursors, but clearing/scanning it and the P2 length pass are not free. Even one dense clear is.073 stores/row at900k, before addressing/loop work; small blocks amortise it worse. A sparse alternative must pay its active-key directory/sort/tag costs. Need active-pair count A, visits to each cursor, cursor-advance counts, and setup Ir before choosing dense versus sparse. A two-byte cursor can increase cache pressure despite removing a168750-byte level-2 directory.

Removing bits2/rank2 also removes lf12 for stop-chain discovery. Substitute sorted batched queries or predecessor searches on P1/P2 and count every chain lookup, sort and prefix; leaving the directory just for stops forfeits most of its allocation/build saving. No double credit.

Run-fed input saves rescanning LL only to the extent the front end already exposes the required maximal LL runs. RLE2 run tokens alone are insufficient: adjacent emissions can join into one L run. Retain cross-token merging, source positions, per-byte bucket ranks, exact count validation, allocation, hand-off and record writes. Changed emitter bookkeeping is charged to the whole decoder even if it moves outside count_d4_build. A claimed2.72 saving must be net of that work, not a boundary shift.

**Falsifier:** first an untimed event census and inclusive source/listing price for acquisition, cursor preparation, both loops, stops, ranks and fallback. A future registered count must fit7.35 with all induced front-end work included in a matched whole-decoder ledger. The current candidate does not earn that count or timing automatically. If credible replacement cost exceeds the~.82–1.06 remaining budget under the optimistic30-Ir assumption, do not build this variant merely to rediscover the deficit. Source dependency reduction is real; satisfying F2 is unestablished.

### 2. Direct LF4 refinement: plausible algebra, unpriced machine construction

The discontinuity partition of LF4 refines the union of the L-run boundaries pulled back by LF^j for j=0,1,2,3, plus the required singleton/stop cuts. Correctness follows by restricting an interval until each of its first four images stays in one L run; then each LF is a translation and the four emitted bytes are constant. Validate boundary conventions and finite-domain endpoints; no wraparound interval is silently allowed.

A direct constructor could propagate/intersect these intervals and emit final records without materialising P2. But discovering preimages, ordering/merging endpoints, recovering words and destinations, and resolving stops remain. A naive direct four-step lookup per final piece can cost more than doubling. Radix sorting requires scratch and passes; calling them linear does not price them.

With the old fixed5.46, only21.51Ir per final piece is available. With both run acquisition and intermediate-directory terms genuinely removed, the allowance becomes66.24Ir per final piece, before replacement work and the.24 context gap. This larger budget makes direct construction a reasonable second paper route, not evidence it meets F2. Require a complete event schedule and storage bound before a prototype. Prefer the more concrete two-byte cursor question first.

### 3. Across-block reuse: not justified for ordinary bzip2 blocks

Similar run counts or substrings do not identify equal LF maps, bucket bases, primary indices or segment stops. Reusing a table requires verified exact structural equivalence, or reconstructing all changed fields with a proof. Fingerprints must include collision resolution; caching retains tables and adds lookup/copy costs. No measured reuse rate or net credit is supplied. Reject cross-block amortisation as the next rescue; not a theorem that exact duplicate blocks cannot benefit.

Do not close the underlying small-table advantage merely because these two candidates lack a current proof of7.35. Record the failed implementation, the open mechanisms, and the economic reason to defer any unpriced build.

## Load law: conditional width economics, with construction exposed

Previous report9ff1d0e2 already established that X from the old p grid includes build congestion. It cannot be multiplied wholesale by.75 and called a measured D4 penalty. Dr/Dw are instruction-level references, including stack/spills; they are neither bytes transferred nor independent cache misses. The boundary word and rank share an index and may overlap in execution; the record depends on both. “Three dependent loads” is an overcompressed description of that dependency graph, not three necessarily serial misses.

For transparency, using the old p1=544.135MB/s, interpolated X, s=.857428MB, a=.75 and **g=22.17/15.38≈1.4415 only as an equal-time-per-instruction proxy**, gives:

| W | X(Ws) | optimistic inversion MB/s | maximum g to equal2430MB/s |
|---|---:|---:|---:|
| 7 | .01025 | 2628 | 1.560 |
| 8 | .01390 | 2998 | 1.781 |
| 9 | .04942 | 3312 | 1.978 |
| 10 | .10594 | 3578 | 2.160 |

These are sensitivity calculations, not forecasts, timing evidence or permission. They explain why a1.44 instruction ratio does not disprove benefit at width. They do not show D4 achieves it: target build/walk cycles, overlap and live scratch are unknown; the old corpus/harness and current production baseline differ.

The critical omitted costs are observed in the result:
- retained construction scratch2.59–4.40MB per worker, separate from the walk table; at7–10 copies the latter figure alone is30.8–44MB;
- reserved record address space2.05MB on block2, although only written records are touched;
- the full-corpus final-table p90 is1.373MB, max2.018MB; only197/233 full Wikidata blocks pass1.2MB, not all;
- full text/records/random blocks do not pass. Admission r4 is known only after the costly second composition in this implementation.

Retained capacity is not proof of simultaneous cache residency, and not all reserved bytes are touched. But overlapping builders, high-water reuse and fallback tables prevent assuming W×.86 is the decoder's live set. A rejected block can pay D4 construction PLUS the production fallback. Early r-based admission can be conservative and lose admitted blocks; late admission must charge wasted work. A hybrid's relevant mean is byte-weighted and its latency follows the busiest worker, not the84.5% block fraction.

Whole-decoder accounting, in seconds per row:
t_worker = t_front + t_build + t_walk + t_assembly + t_fallback + t_coord.
For balanced, continuously supplied workers, D ≤ W/t_worker; supply and ordered-output service impose additional ceilings. Actual completion uses the slowest worker's sum over assigned blocks. No mean CPU ratio alone establishes that maximum.

An explicitly optimistic illustration: if one-worker baseline were400MB/s, non-inversion fraction.3 inflated1.2×, and inversion fraction.7 using the proxy above, W=7/8/9/10 would give2037/2325/2581/2808MB/s. These assumed fractions, clocks and supply conditions are not measured for D4. They exclude scratch-induced and fallback penalties. They must not become a registration's expected gain without target evidence. Against any matched target D*, allowable complete worker cost is W/D*; that is the real budget to compare with measured per-phase service, not7.35Ir converted into seconds.

**Answer:** yes, locality can compensate for more instructions at7–10 workers. No, this load law cannot currently establish that compensation for this build. A future time-oriented rule must be a separately justified prospective registration for a new construction, not relabelled permission to time the failed candidate.

## Correcting the reusable piece-cost model

My report for6757808815 explicitly labelled .197n×30≈5.91 as a model and exposed only1.44Ir/row for omissions. It was still optimistic: the compiled inclusive pieces here cost41.8 and65.9, factors1.39 and2.20 above30, plus large fixed terms. Withdraw30 as a default price for complete composition. It is at most an aspirational kernel budget.

For similar materialised schemes use:
B/n = acquisition/n + c2 q2 + directory/n + c4 q4 + stops/n + ranks/n + allocation/validation/context,
with41.8/65.9 as empirical reference costs for this source/input, not universal constants. Use corpus-specific densities and a separate split/output count. At all-Wikidata q2=.0881,q4=.1088, those coefficients alone give≈10.85Ir/row before fixed costs. This is extrapolation, not a counted corpus total.

D2 remains unqualified: fewer composition passes do not cure its previously estimated expensive walk. D8 needs extra composition and wider records; no reuse of30 rescues it. Direct run tables, dense fixed-width fills and tunnel copy events have different operations; do not blindly multiply every old model by2.2. Reprice each event from its source, including control, directories, validation and output. Published earlier verdicts remain intact; append this correction rather than overwrite their evidence.

The tunnel figures .0785/.0385 are potential lookup counts under a post-hoc stretch definition, not executable copy coverage. LF4-step heights are not byte heights; singleton cuts, segmented landings, available output and word metadata still need charging. No construction credit is imported from that census.

## Decision and next value

Expected: a cheap composed representation might fit the count filter. Found: walk and registered three-block table fit; the full constructor does not. Zero measured performance gap is closed by this review. The new consequence is a concrete two-symbol monotonicity route and explicit replacement budget, not an instruction-gate exception.

Keep team-path/service/emitter and the scoped geometry work ahead of a new D4 build. For this line, prefer a small event/cost census for pair cursors with run-fed acquisition, then direct refinement if that budget fails; do not schedule timing or cross-block caching. An untimed falsifier can eliminate an optimistic constructor cheaply; a full rescue build without its complete price is economically inferior to the already-defined work.

Fresh migration head/digest matched before receipt6cf81f36. Checkpoint dfe05b92 retained. Routing audit on freshly fetched headers has five historically resolved entries and the existing researcher-inbox LOG6757065549; no new routing error from this brief. Raw profiles and native safety validation were not repeated. No implementation, timing, merge, adoption or authority change.
