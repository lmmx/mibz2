# Small tables: carry the interval; separate cost screens from speed gates

Researcher response to inbox **6758126153**, dispatching 6757811329.
Spec pin a2da49638ac54d7763de1c167bd054f67ffdc763. Research only.
Date: 2026-11-10. No implementation, new timing, merge or adoption approval.

## Disposition

Keep D4-delta's registered counting filters unchanged. **Neither 1.44 nor 2.5 is an established allowable slowdown.** The former compares concurrency; the latter is a conditional throughput model with a different denominator. For future exploration use an explicit break-even surface, then qualify on a matched whole-decoder comparison.

The best adjacent formulation is a **stateful move walk**, carrying the interval containing the current row. It removes the repeated rank query rather than making rank faster. Price its destination search and record inflation before proposing another build. It is not yet better than D4-delta. Wavelet/rank decoding is a useful space reference, but unlikely to clear the present 15-Ir/row screen on a byte alphabet. Interleaving can hide latency; it does not divide instruction counts.

## Evidence actually read

- Inbox6758126153; QUEUED6757811329; Pathfinder6757876056; Optimiser6757974415; Coordinator6758018574.
- Optimiser's report at d4c4b41c89db2561bd3b3470670894a4b6d2e318, state/records/reviews/inbox-6757808815/report.md.
- Pathfinder's docs/journal/2026-11-10-share-nothing.md at 2c70fd13e (blob59adea98dc03b7f82ae5bc2bab8795fcb98054eb).
- Nishimoto and Tabei, *Optimal-Time Queries on BWT-runs Compressed Indexes*, §3, Lemma6 and Theorem7: https://arxiv.org/html/2006.05104v3
- Claude, Navarro and Ordóñez, *The Wavelet Matrix*, author abstract: https://users.dcc.uchile.cl/~gnavarro/abstracts/is14.html

E's 3.665× inversion, 4.342× walk and 48.21Ir/row are reported/replayed by the Optimiser, not independently replayed here. Likewise par-scan's raw archive was not replayed here. These observations motivate hypotheses; this report does not recertify their measurements. The wavelet full PDF and Movi full article fetches failed; no empirical number from either is imported.

## 1. The budget is a surface, not either scalar

Let A be prospective small-table throughput at g=1, f the fraction of *matched one-thread CPU cost* attributable to build+walk, g the candidate's one-thread build+walk CPU ratio, h(g) its additional concurrency inflation relative to the g=1 case, and Q the scanner/sink supply ceiling:

    T_new(g) <= min(Q, A / ((1-f+f*g)*h(g))).

This is a modelling ceiling, not a performance guarantee: imbalance and serial sections may lower it further. Against matched default throughput D with desired gain epsilon, require

    (1-f+f*g)*h(g) < A / (D*(1+epsilon)),
    Q > D*(1+epsilon).

When h=1, g_max = [A/(D*(1+epsilon))-(1-f)]/f.
At A=3.0GB/s, D=1.5GB/s, f=.7, epsilon=0: **g_max=2.42857**, not exactly2.5.
At g=2.5 the model gives1.4634GB/s; at g=1.2,2.6316; at g=3.67,1.0457.
For a prospective10% advantage, the first threshold becomes2.16883.

Sensitivity at D=1.5,f=.7,h=1:
A=2.4 -> g_max1.857; A=2.563 ->2.012; A=2.7 ->2.143; A=3.0 ->2.429; A=3.3 ->2.714.
Using2.563 here is a sensitivity, NOT a claim that level1's observed rate estimates level9's counterfactual A.
Even at A=3.0, h=1.2 lowers break-even to1.9524. A1.2MB final table has no right to inherit a .4MB table's h.

**Denominators matter twice.** 1.44=CPU10/CPU4 on one representation is not candidate/reference kernel cost. And the earlier15-Ir threshold concerns the walk alone, whereas g is build+walk *CPU time*. Neither f=.7 nor g can be derived by substituting fractions of Ir. A general two-phase model uses
1-f_B-f_W+f_B*g_B+f_W*g_W, separately charging new construction.

**Busy cores do not prove unconstrained throughput.** CPU/wall=.84/.082≈10.24 measures occupied CPU equivalents, includes harness activity and separately summarised observations. Spinning workers or memory-stalled executing threads can have high occupancy and poor useful throughput. It cannot prove “nothing structural binds”. High occupancy and flattened CPU growth are useful supporting observations, not that conclusion.

**Future evidence needed:** matched workload/source/output; whole-decoder default D in the same conditions; one-thread phase CPU fractions; concurrency inflation and useful-work occupancy; full per-worker live set including build scratch and queued outputs; mode-preserving paired observations. Recompression changes block count, run density, MTF work and per-block overhead as well as cache footprint. The -2 pricing run can constrain the architectural model, not identify a new compact level9 kernel's g or h.

Keep the existing D4 screen15Ir/row,7.35Ir/build row,1.2MB fixed. For a later registration, a candidate missing15 may still deserve a different diagnostic if its measured/predicted cost and small-table benefit justify it prospectively. That is a new experiment, not reclassification of this probe or E.

## 2. Adjacent mechanism: stateful move, not repeated predecessor/rank

The primary move-structure result accepts (row, interval-ID), returns both the mapped row and its next interval-ID. It balances piecewise translations to at most twice the original interval count; a destination scan then advances at most three intervals. Its representation uses three words per balanced interval. This is a theorem about queries and space, not an x86 instruction measurement.

Our derivation: within piece x starting at p[x], LF(i)=q[x]+i-p[x].
Store delta[x]=q[x]-p[x], plus b[x]=the interval containing q[x].
Given the carried x, compute j=i+delta[x], begin y=b[x], and advance y while p[y+1]<=j. Carry y into the next step. No arbitrary-position rank query occurs after initialization.

**Counterexample to a zero-scan assumption:** input intervals [0,1),[1,2),[2,4),[4,8) with q=[0,1,6,2] form a permutation of [0,8). Starting i=7 in the final interval maps to5. q=2 lies in interval2 but5 lies in interval3: carrying only b without correction gives the wrong next interval. Larger constructions can require many advances without balancing.

**Cost hypothesis, expressly not a measured count:** scalar complete one-byte walk18–26 + (3–5)*s Ir/row, where s is the traversal-weighted number of extra intervals advanced. This charges row/interval state, record decoding, destination bound test, output and ordinary loop control; lane/tail extras must be added if absent. Thus even s=0 probably misses15. Falsifier: compiled complete loop on registered blocks; below18 refutes this estimate favorably, above26 at s≈0 rejects the proposed price. Census mean and maximum s, not just number of runs.

**Space prerequisite:** an engineering12-byte record for start20,delta20,destination-ID20,symbol8 and flags can fit96bits; this is a proposed packing, not the paper's word layout. At balanced density rho'=r'/n, final records cost12*rho'*n; under1.2MB at n900000 demands rho'<.111111 before other arrays and capacities. Worst-case doubling needs original rho<.055556. Count actual inflation; do not assume typical genome compression on Wikidata.

### Four-step move: promising amortisation, severe space test

LF^4 is itself a permutation. Partition into pieces with both affine LF^4 and identical emitted four-byte words. Apply the same carried-ID idea to that fixed map. A12-byte record can provision start20+delta20+destination20+word32=92bits, with4bits left for flags; no per-row directory is needed on the main path.

At reported rho4=.1088,12-byte records alone take **1,175,040 bytes**. Only24,960 bytes remain under1.2MB: **2.124% record inflation**, before stops, allocation rounding, initial lookup or any other metadata. A16-byte layout already needs1,566,720. General balancing's <=2× guarantee does not establish fit.

Hypothesis:28–44Ir/full four-byte step plus(3–5)*s, hence7–11+(0.75–1.25)*s Ir/byte before setup/short steps. This is a hand ledger with deliberately explicit uncertainty; it is not “E's cost divided by four”. Falsifier: full counted loop including output/lane logic exceeds15Ir/emitted byte, or capacity exceeds1.2MB, or construction exceeds its prospective budget. The cheapest first action is a **piece census and destination-scan distribution**, alongside the already-running D4 probe, not a second timed implementation.

**Important proof boundary:** changing some records to LF^j for marker stops, j<4, can destroy bijectivity. Therefore do NOT apply the balanced-permutation theorem wholesale to that mixed-step table. Keep a fixed LF^4 main map; separately prove and charge exceptional stop handling or show the needed property anew. LF bijectivity also does not prove that a cyclic BWT has one cycle: periodic inputs and bounded output/termination need their decoder-specific argument. Initial interval lookup and seeds for every lane must be counted.

## 3. Other routes, and falsifiable price predictions

All prices here are hypotheses for complete traversal excluding construction, in Ir per emitted BWT row, not elapsed time or final RLE-expanded bytes.

| Route | Mechanism and space | Count prediction / cheap falsifier |
|---|---|---|
| Binary wavelet matrix, joint access+rank | Eight serial bit decisions for full byte alphabet. Plain bitmaps total n bytes; rank metadata extra. Avoid a separate second descent for rank. | 8*(6–12)+6–12 =54–108Ir/row, before exceptional lanes. Count an eight-level complete step; expectation is far above15. Huffman shaping replaces8 by measured average depth, not by entropy without code overhead. |
| Blocked rank, direct symbol bitmap | One bitvector per symbol permits one rank after finding the byte, but256*n/8=32n bytes uncompressed. Compressed/local-alphabet variants restore decoding cost. | For <=4 effective symbols, a two-level path predicts18–36Ir/row. Admit only after actual alphabet/density census; byte-alphabet figures do not transfer. |
| Run-aware one-step move | Carries interval ID, replacing global rank with destination scan; space governed by balanced run count. |18–26+(3–5)*sIr/row; likely misses15, but not logically ruled out by E's22–26 step. |
| Four-step move | Carries LF^4 piece ID and emits packed word; record density near capacity boundary. |7–11+(.75–1.25)*s before short-step overhead. Census first; complete count and footprint decide. |
| Interleaved independent chains | Overlaps dependency latency across seeds; table unchanged. | Approximately underlying walk Ir plus0–4Ir/row schedule overhead if no spills; **not** divided by number of lanes. Count two/four/eight lanes; reject instruction claims if bookkeeping grows, and any speed mechanism requires latency evidence separately. |

Independent byte-alphabet space calculation: eight levels with one32-bit prefix per64-bit word cost900000 + 8*ceil(900000/64)*4 = **1,350,016 bytes** before anything else. To reach1.2MB, rank support needs less metadata, larger sampled spans (extra popcounts), or compression. “Fits in one cache line” does not remove the eight-level address dependency. The wavelet-matrix authors establish a better space/time tradeoff than wavelet-tree variants for their comparisons; they do not establish <=15Ir or a win over a plain decoder T-vector.

Interleaving is orthogonal to representation. With K chains, latency-bound throughput may improve up to K-fold in an ideal isolated chain model, but instruction work stays; cache pressure, spills and outstanding-load capacity limit it. Within-block seeds/segment order and setup may cost enough to erase it; interleaving whole blocks adds K working sets and undermines the footprint objective. First reuse established lanes and count their full loop.

## 4. What literature numbers are credible here

Trust the move theorem's stated interface and asymptotic/balancing bounds under its hypotheses. Trust published memory/time results only for their reported machine, alphabet, dataset and operation. Do not transplant FM backward-search throughput (two interval ranks and different locality), pangenome matching throughput, or a rank-only microbenchmark into full inversion Ir. For any claimed price demand exact implementation/compiler/ISA, retired instruction scope, emitted-byte denominator, runs/row, balanced pieces/row, scan distribution, lane count, setup/tails, allocation capacity and peak live bytes. No literature source inspected provides a transferable x86 Ir/row figure for this decoder; every number in §3 is our falsifiable model.

## 5. Check performed and next consequential step

proof.py exhaustively checks all ternary last columns of lengths1–8, k in{1,2,4}: **29,520 sequence/k cases,221,436 row queries**. It compares explicit LF^k with affine-piece translation and carried destination search; it also requires LF^k to remain a permutation. It checks the algebra, not balancing, compiled costs, BWT acceptance, marker termination or unsafe code. No production code was written or run.

Expected E to buy locality cheaply; observed large extra traversal work and oversized full-block table. No production performance gap closed. Learned: repeated arbitrary-row lookup is avoidable if interval state travels with the walk; the alternative pays in successor search and metadata. D4-delta remains first compact-table probe. Adjacent next information is a census of carried-state LF^4 destination scans and inflation, not a claim that only D can work. Service/adoption work stays ahead of an unpriced full compact implementation.

Migration head and manifest matched; routing audit clean. Research only on team-state/researcher. No queued #326 review was undertaken: it is not yet an inbox assignment.
