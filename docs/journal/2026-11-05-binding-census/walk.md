#### FRONT walk
MACHINE: All CI runners: 4 vCPU = 2 cores x 2 SMT siblings (0-1 | 2-3); no core PMU (PMU probe run 40857395037: core_pmu false on 9V45 and 9V74). Measurements used, by runner (all decoder src identical to d1fd4405; I checked git diff d1fd4405..origin/master over src/ examples/ Cargo.* sweep.yml, and it is empty):
(1) AMD EPYC 7763 (Zen 3), 3.2408 GHz reported. Phases sweep run 42011789918 (head 4bfaf7cb), dispatched by the account just before this front, fastest of 5 passes. Walk incl. assembly: alone 4.33 c/B = 1.34 ns/B; 4 copies mean 9.20 c/B = 2.84 ns/B (x2.12). Plain one-chain walk 40.8 -> 47.9 cycles/step. Total 10.24 -> 17.46 c/B (x1.71). Walk share 42.3% -> 52.7%. Build 2.16 -> 3.08, MTF 1.98 -> 2.63. Decoder (#183 run 1 base, run 41986885355 att 1): 1t 0.692 s, 4t 0.314 s, amplification 1.69x. Occupancy run 41976148952 (7763, window 16): workers decoding 93.0% at 4t; T-vector phase mean 2.85 of 4 blocks; live T-vectors mean 10.3 MB, max 14.4 MB. Caches (published, not probed here): L1D 32 KiB, L2 512 KiB (the journal's 'L2 1 MB' at mlp-scheduling:263 is Zen 4's figure), 24 MAB, L3 46 cycles (AMD 56665, quoted at docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md:261-264).
(2) Intel Xeon 8370C (Ice Lake SP), 3.5806 GHz reported. Phases run 42008210159 (head b7734313). Walk 7.27 -> 10.64 c/B (2.03 -> 2.97 ns/B, x1.46). Plain walk 43.9 -> 68.3 cycles/step. Total 13.45 -> 21.82 (x1.62). Walk share 54.1% -> 48.8%. Build 2.01 -> 4.52 (x2.25: on ICX the build, not the walk

- [OBSERVED] walk step -> one random T-vector line (one per byte; plain chain 40.8 cycles on the 7763 matches 0.14 x 14 + 0.86 x 46, so there is no reuse of lines between steps) -> per-core miss-handling or L2-to-L3 fill capacity shared by both siblings
  CLAIM: On the AMD runners (EPYC 7763 Zen 3, 9V45 Zen 5, 9V74 Zen 4) the 32-lane walk is bound by a per-core memory-side ceiling that one thread already reaches. The SMT sibling adds no walk throughput, so at 4 threads each walking worker gets about half of its core's line rate. The interpretation (per-core ceiling) is INFERRED; the ratios are measured.
  EVID: Per-core walk step rate, 4 copies over 1 copy (2*t1/t4): 0.94 on the 7763 (phases run 42011789918: 4.33 -> 9.20 c/B), 0.90 on the 9V45 (sweep 41989592674: 2.89 -> 6.44 c/B), 0.94 on the 9V74 (sweep 41989596336: 4.27 -> 9.08 c/B). The 7763's other stages gain from SMT: 5.91 -> 8.26 c/B, a yield of 1.43, so issue width is not what the walk lacks. Lanes past the knee add nothing. 9V74 alone: 24/32 lanes give 4.33/4.27 c/B; 48/64 give 4.56/4.84, slower beyond noise. A fit of t(L) = a + lat/min(L,M) (scripts/fit.py) puts the knee M at 22 alone and 16 per thread at 4 copies on the 9V74 (Zen 4's publ
  SHARE: 7763: the walk is 52.7% of the 4-copy stage time (OBSERVED), times 93% of worker time decoding (occupancy 41976148952), so about 49% of 4t wall. 9V45/9V74: about 50-60%, INFERRED as 1t stage share x sweep 4-copy factor / decoder worker CPU (9V45: 0.625 ns/B x 2.23 = 1.39 against 2.34-2.77 ns/B of worker CPU).
  RECOV: Levers that keep about 1 line per output byte (lanes, hints, segments, tile, shape, loop instructions) are bounded by the gap between the 4-copy walk and twice its 1t time. That gap is +6.2% on the 7763 (<= 3% of 4t wall, <= 9 ms of 0.314 s), +11.4% on the 9V45 (<= 6-7%, <= 12 ms of 0.179 s) and +6.3% on the 9V74 (<= 3.5%). Lower bound 0: none of the 59 sweep settings is faster beyond noise, alone or at 4 copies, on either runner (docs/journal/2026-11-05-coresident-walk.md:88-97). Central 0-1%.
  FALSIFIER: Two copies pinned to siblings (CPUs 0,1) reaching >= 1.2x one copy's per-core walk rate; or any walk-loop setting faster beyond noise in a 4-copy sweep on a Zen 3/4/5 runner.

- [INFERRED] walk -> per-lane dependent chain -> prefetch lead = one round -> hidden
  CLAIM: The latency-bound regime does not bind at 32 lanes on any CI model measured. Each lane's prefetch completes about a full round before its demand load, so the misses are borne by prefetches and latency is not exposed per step. Prefetch distance is not a lever: in a pointer chain each lane's lead is fixed at one step, and only the lane count lengthens it.
  EVID: Single-chain latency, from the plain walk (src/bwt.rs:410): 40.8 -> 47.9 cycles/step on the 7763 (+17%) and 43.9 -> 68.3 on the 8370C (+56%), alone vs 4 copies (phases runs 42011789918, 42008210159). The latency-bound prediction, chain latency / 32, gives 1.27 / 1.50 c/B on the 7763 (1t / 4 copies) against 4.33 / 9.20 measured, and 1.37 / 2.13 on the 8370C against 7.27 / 10.64. On the AMD runners it also predicts the wrong 4-copy/1-copy ratio: 1.12-1.18 against 2.12-2.23. One round, 32 x t, is 139 / 294 cycles on the 7763 (1t / 4 copies), 233 / 341 on the 8370C and 92 / 206 on the 9V45, which 
  SHARE: none separable: exposed latency per step is about 0 on average
  RECOV: 0-1%: prefetch placement, lead and hint. L2, NTA and late variants tie or lose (docs/journal/2026-11-05-walk-round.md:82; coresident-walk.md:79-97).
  FALSIFIER: Counters showing more than about 20% of demand loads hitting lines still in flight at 4 copies, or a longer-lead variant faster beyond noise. Neither is measurable on CI today (no PMU).

- [OBSERVED] walk -> line per step -> 12 L1 fill buffers per core shared by siblings
  CLAIM: On the Xeon 8370C (Ice Lake SP) the walk is fill-buffer-bound (the MLP regime) at 4 threads but below its ceiling at 1 thread. Here SMT helps the walk, and the build is the stage that loses from it.
  EVID: Phases run 42008210159. The walk's SMT yield is 1.37 (7.27 alone -> 10.64 c/B at 4 copies); the build's is 0.89 (2.01 -> 4.52). The effective in-flight count (plain/room) is 6.0 per thread alone and 6.4 per thread = 12.8 per core at 4 copies, about Sunny Cove's 12 fill buffers. The MLP prediction for 4 copies, loaded chain 68.3 cycles / (12/2 FB per thread) = 11.4 c/B, is within 7% of the measured 10.64. At 1t the same model predicts 43.9/12 = 3.66 against 7.27 measured, so one thread uses only about half the fill buffers. The cause of that is unknown.
  SHARE: 48.8% of 4-copy stage time, about 45% of 4t wall (0.39 s, run 41986908473)
  RECOV: From the walk loop at 4t: 0 to about 3% of wall (at the fill-buffer ceiling within the model's 7%). From the build: outside this front, but its x2.25 at 4 copies (20.7% of 4-copy stage time) is the larger SMT penalty on this model.
  FALSIFIER: A fill-buffer-light variant (for example an L2-hinted prefetch, which per the Intel manual does not hold a fill buffer, journal mlp-scheduling:270) faster beyond noise at 4 copies on an 8370C.

- [INFERRED] walk -> lines per byte (about 1) x L2 hit rate (footprint per core) x per-core ceiling
  CLAIM: The walk's share of 4t wall is not its recoverable time. Its 4t floor is (L2-miss lines per output byte) x (per-core ceiling, halved per sibling). Below the gap in the first entry, its 4t time can fall only through more L2 hits (a smaller table) or fewer lines per byte. At 4t each core holds two 3.6 MB T-vectors in a 0.5-2 MB L2, and on Zen 3 the walk's lines still come almost entirely from the on-chip L3.
  EVID: T-vector 4n = 3.6 MB per full block. The 233 blocks of at least 64Ki rows hold 99.996% of rows (journal mlp-scheduling:207). Live at 4t: mean 2.85 of 4 T-vector phases, 10.3 MB mean, 14.4 MB max (occupancy 41976148952). Per core at 4t: 7.2 MB against an L2 of 0.5 MB (Zen 3), 1 MB (Zen 4/5, PMU probe 40857395037), 1.25 MB (ICX) or 2 MB (EMR). Uniform-random L2 hit rates, 1t/4t: 0.14/0.07, 0.28/0.14, 0.35/0.17, 0.56/0.28. On the 7763 the plain chain's 40.8 cycles matches 0.14 x 14 + 0.86 x 46 (L3, AMD 56665), and its rise to 47.9 at 4 copies is accounted for by the L2 loss (+2.4 cycles) plus a D
  SHARE: about 45-49% measured (8370C, 7763); about 50-60% inferred (9V45, 9V74); 55-67% bound on the 8573C (its whole-decoder 4t amplification of 1.62 caps the walk's factor at about 2.1-2.3 if the other stages amplify 1.0-1.15x)
  RECOV: Bounded only by changing lines. For example, half the lines per byte (a two-bytes-per-line walk) would halve the 4t walk: up to -22% to -30% of 4t wall, before its added step and build cost.
  FALSIFIER: A table with the same lines per step but a higher L2 hit rate (for example, the walk over an L2-resident 0.75 MB synthetic table in the 4-copy sweep) that is not faster per step at 4 copies.

- [INFERRED] walk -> table footprint per core -> L2 hit rate -> per-core ceiling; plus the step's added instructions and loads
  CLAIM: Compact30 (64-byte records of 30 entries plus a RAW area, about 2.13-2.164 B/entry: 1.92-1.95 MB a block, 3.8-3.9 MB per core at 4t) still costs one line per step. In the regime that fits CI, the per-core ceiling, it changes the 4t walk only through the L2 hit rate. Its added step work (21 vs 16 instructions, 4 vs 3 loads in the prototype; blog 2026-11-02 :1593) is free only to the extent that the ceiling binds. On CI its sign is unresolved.
  EVID: 4t, two tables per core (scripts/model.py), optimistic / pessimistic (L2 share 0.6). Fill bandwidth model, L2-miss lines: Zen 3 -6.6% / -3.8%; Zen 4/5 -14.2% / -8.0%; ICX -18.4% / -10.2%; EMR about -33% / -17%. MLP model, miss occupancy, using the measured loaded chain where available: Zen 3 -4.5% / -2.7%; Zen 4/5 -9.8% / -5.6%; ICX -14.1% / -7.9%; EMR about -26% / -14% (assuming an SPR-like 33 ns L3). Latency-bound model: the same latency change, but the +31% step instructions are exposed. The i9 measured exactly that cost at 1 core, 1.148x (blog 2026-11-02:1624). The i9's 4-worker 0.840x (:1
  SHARE: acts on the walk's about 45-60% of 4t wall
  RECOV: If the step overhead is fully hidden: Zen 3 -1.3% to -3.2% of 4t wall; Zen 4/5 -2.8% to -7%; ICX -3.5% to -9%; EMR -5% to -16% (regime unmeasured there). If the overhead is exposed as on the i9 at 1 core: up to +5% to +8%. The builder's cost comes on top: today's build is 17.6% (7763) and 20.7% (8370C) of 4-copy stage time, so each +10% of build adds about +2% of wall. Net bound across CI models: -16% to +8%, central -1% to -4% on Zen 4/5 and ICX, about 0 on Zen 3.
  FALSIFIER: A compact30 staged-walk row in the 4-copy sweep on a Zen 4/5 runner that is not faster than room/32/L1 beyond noise. That would refute the L2-capacity gain on CI. It is a walk-only test, so it needs no builder.

- [INFERRED] walk at 4t -> sibling-shared per-core resources (main) vs L3/fabric shared across cores (minor)
  CLAIM: Most of the walk's 4t penalty on CI comes from the SMT sibling sharing a core, not from the other core or the shared L3. VM-wide footprint levers, the i9's mechanism, have little to act on here.
  EVID: Bench 2-thread runs (one worker per core, CPUs 0,2) have whole-decoder amplification of 1.04-1.05 on the 8573C (41980640481; artifact 13392722800), 1.04-1.05 on the 8370C (41986908473), 1.07 on the 9V45 (41986885355 attempt 2) and 1.06-1.11 on the 9V74 (41974498254, 41974509600). With the walk at 42% or more of 1t time, its 2-thread factor is at most 1 + (amp2 - 1)/share, about 1.10-1.26, even if no other stage amplifies. The 7763's 1.17-1.20 comes from a noisy run (41986912287). The 4t amplification, by contrast, is 1.55-1.88.
  SHARE: cross-core part of the walk: at most about 5-12% of 4t wall
  RECOV: Cross-core and L3 terms: at most about 10-25% of the walk's 2x penalty, so roughly at most 2-6% of 4t wall from any VM-wide footprint change, before costs.
  FALSIFIER: A walk-only 2-copy run on separate cores (CPUs 0,2) slower than 1.2x alone.

- [OBSERVED] walk -> instructions -> hidden under the per-core ceiling
  CLAIM: Instruction counts do not predict the walk's 4t time on CI. This is consistent with the ceiling: cuts inside the loop buy nothing, and +4% Ir has cost nothing.
  EVID: #168 (-2.6% Ir) and #173 (-2.0% Ir): no gain (docs/journal/2026-11-05-walk-round.md:18-39). #181 (+4% Ir): walk stage -2.6% on the 9V74 and -6.8% on the 9V45, 4t wall -3.2% (-5.6, -0.7) on the 9V45 (docs/journal/2026-11-05-staging-only.md:85-99). #171's staging, which took the 32 store streams' line traffic off the miss buffers, gained most at 4t: -5% to -14% (walk-round.md:23, 63-71).
  SHARE: n/a
  RECOV: 0-1% from walk-loop instruction cuts on the AMD runners. ICX has headroom at 1t only.
  FALSIFIER: A pure instruction cut in room_segments, with the same lines and stores, that is faster beyond noise at 4t on an AMD runner.

- NOGO More lanes (48/64) or a co-resident 16-lane/L2 shape at 4t: of=the 'more MLP from more chains' route (and #183's co-resident dispatch)
  changed: At 4t each core already carries 64 lane-chains against a measured effective capacity of about 10-13 per core (7763: 10.4; 8370C: 12.8) and fitted per-thread knees of 16-22. The ceiling is per core, not per lane.
  now: 0 change, or slower (48/64 lanes flat to +15% alone on the 9V74; 16/L2 +10% to +54% at 4 copies)
  test: Done: sweeps 41989592674 and 41989596336 found no setting faster beyond noise (coresident-walk.md:72-97)
  score: 0

- NOGO A longer prefetch distance, or a latency-scaled prefetch distance (listed as never tried in the 2026-11-01 proposal): of=the H2 prefetch-interaction lever
  changed: In a pointer chain each lane's lead is one step, so the distance is one round: 92-341 cycles, 2.9-6x the loaded chain latency (41-68 cycles) on the CI models measured. Latency is already hidden, and only lanes, which are flat past the knee, lengthen the lead.
  now: 0 at 1t and 4t on the 7763, 8370C and 9V45
  test: Needs a demand-hits-in-flight-line count; CI has no PMU (probe 40857395037). A 'late' variant already lost (walk-round.md:82).
  score: 0

- NOGO Instruction cuts inside the walk loop (deferred flags, tail passes, flush SIMD): of=#168, #173 and walk/flush
  changed: The AMD walk is at a per-core ceiling at both 1t and 4t (SMT yield 0.90-0.94), so instructions are hidden. ICX at 1t is the exception: it is below its ceiling (yield 1.37).
  now: 0 at 4t on any CI model. Possibly a small 1t gain on the Xeon 8370C only.
  test: A 1t A/B that happens to land on an 8370C; runner assignment is random
  score: 0.5

- NOGO A larger tile (512, 1024): of=#177 and #178
  changed: None. The prefetched lines and two siblings' tiles must stay in L1D (32 KiB on Zen 3/4).
  now: worse: 1024 gave +23.5% at 1t on the 9V74
  test: done (tile-size.md:84-104)
  score: 0

- NOGO A two-bytes-per-line walk (psi^2, pair walk): of=#6 and #34 (lost on CI's EPYC at every thread count, with the older flagged walk before staging; bz2ngrs.md:266-267, expert-review.md:302-306)
  changed: The per-core ceiling is now identified on the Zen runners: the walk's 4t time scales with L2-miss lines per output byte, and halving them is the only lever that scales the floor. Its extra step work could hide under the ceiling at 4t, which the old flagged kernel could not show.
  now: Walk at 4t -30% to -45% if the step overhead stays hidden, so -15% to -25% of 4t wall from the walk. But the bigram build (65,536 streams, about 3x) was what sank it. Net is likely still negative unless the build is fixed.
  test: A staged psi^2 room walk as a row in the 4-copy walk sweep, walk only, on a Zen 4/5 runner. Only if that is faster beyond noise should a build budget be priced.
  score: 2

- NOGO Huge pages for the T-vector (pending the maintainer: allocator-adjacent): of=the H6 page-walk lever
  changed: Under the per-core ceiling, STLB hit latency on a prefetch delays its issue but does not hold a miss buffer. 879 pages per table, two tables per core at 4t (1,758 pages), against Zen 4's 3,072-entry L2 DTLB (journal mlp-scheduling:309); Zen 3's is smaller.
  now: Small (<= 5% of the walk) unless page walks occur. Possibly more on Zen 3 at 4t. Unmeasured.
  test: Needs DTLB walk counters (not on CI), or a THP A/B, which needs the user's decision
  score: 1

- NOGO Fused post-walk pass (#183): of=closed #183
  changed: Consistent with this model: the post-walk traffic is sequential and handled by the hardware prefetcher, not the per-core random-line ceiling that binds the walk
  now: 0 at 4t
  test: done (fused-postwalk.md:133-158)
  score: 0

DISPATCHED: None. Two master phases runs, 42008210159 (Xeon 8370C, head b7734313) and 42011789918 (EPYC 7763, head 4bfaf7cb), both with src identical to d1fd4405, were dispatched by the account between 10:08 and 10:34 UTC and supplied the per-stage 4-copy data this front needed. A third would land on a random runner and could not decide the remaining open question (miss buffers vs fill bandwidth) without counters, which CI lacks (PMU probe 40857395037).
UNKNOWNS: (1) What the AMD per-core ceiling actually is: miss-buffer capacity at a loaded latency of about 2.4x the single chain (the effective about 10 per core is well under the published 24 MAB), or L2-to-L3 fill bandwidth at about 2 lines per L2 miss from the up/down prefetcher (this fits the 9V45 at 1t and 4 copies, but is 1.25-1.5x off on Zen 3/4). Both predict the same sign for footprint changes and differ on how much an L2 hit saves. CI has no PMU to separate them. (2) Zen 5's L1 miss-buffer count: no primary source found. (3) The 8573C (Emerald Rapids) at 4 copies: no walk or phases measurement exists, and its L3 latency is unknown (Sapphire Rapids is about 33 ns). (4) Why the 8370C uses only about 6 of 12 fill buffers at 1t with 32 lanes. (5) The share of the VM's visible 32 MB L3 that is actually usable alongside other tenants, and the DRAM share at 4t. That share is bounded at about 2% on the 7763 by the plain chain's +7 cycles; the 8370C's +24 cycles is unattributed (L3/mesh vs DRAM). (6) The sibling-vs-cross-core split of the walk's x2.1-2.2 is bounded only indirectly, through whole-decoder 2t amplification. (7) The 4-copy phases figures are fastest of 5 passes, which biases them low against the sweep's median of rounds, so the 7763 and 8370C factors may be understated. (8) The compact30 staged walk and builder have never run on CI; their cost there is unknown. (9) TLB terms are unmeasured. (10) Zen 3's 512 KiB L2 is from AMD's specification, not probed on the 7763 in these runs.
NEXT: A 2-copy mode in examples/sweep.rs and sweep.yml (one small PR, full tier only, about 5-6 minutes a dispatch). The walk-only factorial and the phases tool would each run with copies on CPUs {0,1} (siblings) and on {0,2} (separate cores), plus two fixed sibling probes on CPU 1 beside a measured walk on CPU 0: (i) a 32-lane chase over a 0.75 MB L2-resident table, which holds miss buffers with short fills and little L3 traffic; (ii) a 64 MB streaming read, which uses L3/DRAM bandwidth with few miss buffers per byte. Rows to add: compact30's staged walk over prebuilt records, and an L2-resident synthetic table of the same step shape. What each outcome decides: if {0,2} gives about 1.0x and {0,1} about 2x, the walk's 4t penalty is per core (this model), and VM-wide L3 levers are out. If probe (i) reproduces the slowdown, the ceiling is miss-buffer capacity; if (ii) does, it is fill bandwidth. The compact30 row settles compact30's sign on CI before any builder work: faster beyond noise at {0,1} on a Zen 4/5 runner, or not. Because runner assignment is random, dispatch until a Zen 4/5 and an Intel model have each been covered.

#### FALSIFIER

- FALLS: AMD (EPYC 7763 / 9V45 / 9V74): the 32-lane walk sits at a per-core memory-side ceiling that one thread already reaches; the SMT sibling adds no walk throughput (2*t1/t4 = 0.94 / 0.90 / 0.94).
  WHY: The ratios and gaps reproduce (+6.2 / +11.4 / +6.3%). The regime is contradicted by C's own runs, on five counts.
(a) KERNEL TEST, OBSERVED (phases 42011789918, 7763). Plain, bitmap, flagged and room walks all read about one random T line per byte. Their 1t line rates are 0.025 / 0.071 / 0.145 / 0.231 lines a cycle, and their 4-copy factors are x1.18 / x2.16 / x2.13 / x2.12. At 4 copies the bitmap walk draws at most 0.065 lines a cycle per core, 28% of what the memory side gave one room thread alone, and still loses x2.16. A per-core line-rate ceiling cannot impose the same x2.1 on kernels 3.3x apart in line rate. The factor follows each kernel's own per-byte work. The sweeps agree: flagged/32/L1 goes 4.37 -> 11.78 (x2.69) on the 9V45 and 6.82 -> 14.57 (x2.14) on the 9V74. That is at most 0.170 and 0.137 lines a cycle per core, against 0.346 and 0.234 delivered to one room thread.
(b) LA
  CORR: Not 0-5%. On AMD the 4-copy walk is about 2.1-2.2x its 1t time across kernels (7763: three kernels at x2.12-2.16). So a cut of x in the 1t walk that keeps lines per byte fixed carries over about 1:1 to the 4t walk, which is about 0.5x of 4t wall. Check: #181's -6.8% on the 1t walk predicts -3.4% of 4t wall; the bench measured -3.2% (-5.6, -0.7) on the 9V45. Bounds: lo 0. Theoretical hi = the walk's SMT excess, 53-55% of the 4-copy walk, which is about 26-33% of 4t wall. Single past changes realized 3% (#181) and 5-14% (#171, 4t wall).

- FALLS: The latency-bound regime does not bind at 32 lanes on any CI model; exposed latency per step is about 0; prefetch distance is not a lever.
  WHY: The mechanism fails; the 4t prediction stands.
- C's own fit is additive, t = a + lat/min(L, M). Its latency term is 47.1/32 = 1.47 c/B of 2.89 (51%) on the 9V45 alone and 52.0/22 = 2.36 of 4.27 (55%) on the 9V74 alone. That is an exposed latency term, not about 0.
- On the 9V45 alone, 24 lanes is +19.0% (+14.5, +23.6) slower than 32. Its round, 24 x 3.45 = 83 cycles, is already 1.8x the fitted 47-cycle latency. And 48 lanes is -3.3% (-10.0, +3.9). So the inference 'round >> chain latency, therefore latency is hidden' fails on Zen 5 at 1t. A falsifier already exists without counters.
- At 4 copies every AMD curve is flat from 24 to 64 lanes, so 0 at 4t survives.
- Missed: without prefetch, the 4-copy walk is flat at 10.5-10.9 c/B (9V45) and 12.1-12.9 (9V74) from 12 to 64 lanes. Prefetch's value grows under SMT: none-vs-L1 goes +48% -> +66% on the 9V45 and +23% -> +40% on the 9V74. That f
  CORR: 4t: 0-1% from lanes, hints, segments and prefetch lead. No 4-copy setting is faster beyond noise (coresident-walk.md:88-97). 1t on Zen 5: 0-3% (48 lanes, -3.3%, n.s.), which is not 4t wall.

- FALLS: Xeon 8370C (Ice Lake SP): the walk is fill-buffer-bound (MLP regime) at 4 threads and below its ceiling at 1 thread; SMT helps the walk, and the build loses.
  WHY: The fill-buffer part does not hold. The build's x2.25 (2.01 -> 4.52 c/B) stands as an observation.
(a) KERNEL TEST, OBSERVED (phases 42008210159). Bitmap (x2.24) and flagged (x1.95) lose more than room (x1.46). Yet by Little's law with plain4, their per-core in-flight at 4 copies is 4.0 and 7.3, far under 12 fill buffers. The walks with the most work per byte lose the most.
(b) Room's per-thread in-flight barely changes, 6.0 -> 6.4. Both walk only about 49% of the time (duty), and 6 + 6 reaches 12 only then, so the fill-buffer cap is never needed.
(c) plain4 (68.3 cycles) is measured while the siblings run plain passes. phases.rs runs each unused walk's passes on their own, after the production passes. So plain4 is not the room walk's loaded latency. A heavier co-load would put in-flight at 12.8 or more a core, over the asserted cap; the 'within 7%' is 7% faster than a hard 12-buffer bou
  CORR: The regime is unresolved: core-side or constant-MLP latency, not shown to be fill-buffer-bound. Walk levers: lo 0, hi about 14% of 4t wall (the room walk's SMT excess, 3.38 of 10.64 c/B, times about 45% walk share). The build's +2.51 c/B at 4 copies (OBSERVED) is the larger SMT term on this model.

- FALLS: The walk's 4t floor is L2-miss lines per byte times the per-core ceiling; below the gap only more L2 hits or fewer lines per byte help; a two-bytes-per-line walk would cut 22-30% of 4t wall.
  WHY: It inherits verdict 1's refuted regime. In C's own fit, the lane-dependent (memory) part of the 4-copy walk is lat/M = 2.03 of 6.44 c/B (32%, 9V45) and 3.43 of 9.08 (38%, 9V74). The remainder is the lane-independent SMT term. Changes that keep the lines fixed have moved 4t: #171 (store pattern, same lines) -5% to -14%, and #181 -3.2%. Halving lines halves steps, but the pair step's work lands on the binding core-side term. #6 and #34 lost on CI EPYC. Vendor nit: the Zen 3 L2 is 'no less than 12 cycles' (56665 section 2.6.3), not 14. That gives 0.14 x 12 + 0.86 x 46 = 41.2 instead of 41.5, which changes nothing.
  CORR: Footprint and line levers act only on the 32-38% memory term, a theoretical hi of about 15-19% of 4t wall. An L2-hit change like compact30's (hit rate 0.14 -> 0.26, average latency about -10%) recovers about 1.5-2% of 4t wall. Pair walk: sign unknown; the -22 to -30% has no support.

- FALLS: Compact30 at 4t changes the walk only through the L2 hit rate; its added step work is free to the extent the ceiling binds; net -16% to +8% of 4t wall, central -1% to -4% on Zen 4/5 and ICX.
  WHY: The binding 4t term on AMD is lane-independent work: a4 = 4.49 and 5.68 c/B, 70% and 63% of the 4-copy walk. That is exactly where +31% step instructions and a 4th load a step land. The phases kernel rows show the 4-copy walk time tracking per-byte work. Ir is not proportional (#181: +4% Ir and faster), so the size of the hit is uncertain, but it is not hidden. The EMR -5% to -16% has no 4-copy measurement under it.
  CORR: Zen 4/5, 4t wall before the builder:
- if a4 scales with step work: walk +15.6% (9V74) / +18.6% (9V45), about +8% to +11% of wall;
- if a4 does not scale at all: walk -3.7% / -3.0%, about -2%.
Range -2% to +11%, central about +3% to +5% (slower). Add about +2% of wall for each +10% of build cost. Zen 3: same sign, with a smaller L2 gain (0.07 -> 0.13). Intel models: no regime basis for any figure.

- SURVIVES: Most of the walk's 4t penalty is from the SMT sibling, not the other core or L3; cross-core and L3 terms are at most about 2-6% of 4t wall.
  WHY: The 2-thread pinning is real: --spread uses one CPU a core (examples/ab.rs:19, 718-720; test spread_list(cores, 2) == "0,2"). The arithmetic checks: 1 + 0.04/0.42 = 1.10 and 1 + 0.11/0.42 = 1.26. Verdict 1 sharpens it: the per-core part is core-side, not memory-side. The 7763's 2t figure comes from a noisy run.
  CORR: As stated: at most about 2-6% of 4t wall from VM-wide footprint changes.

- FALLS: Instruction counts do not predict the walk's 4t time; cuts inside the loop buy nothing, being hidden under the ceiling (#168, #173, #181).
  WHY: Only 'Ir is not the predictor' survives (#181: +4% Ir and faster).
- #168 (-2.6% Ir) and #173 (-2.0%) could move 4t wall by at most about 1.3% even if fully proportional. Their 4t half-widths were +/-1.8-2.1% (pr168, 7763 and 9V74) and +/-1.55% / +/-5.2% (pr173, 7763 and 9V45). 'No gain' is an unresolved result, not a null.
- #181 is a walk-loop change that cut 4t wall by -3.2% (-5.6, -0.7) on the 9V45. That meets C's own falsifier for claim 1 ('any walk-loop setting faster beyond noise... on a Zen runner') through the decoder bench.
- 'Hidden under the ceiling' falls with verdict 1.
- The same refutation applies to C's no-go 'instruction cuts inside the walk loop: 0 at 4t on any CI model'. Reopen it as per-byte core-work changes (stores, loads, branches, replay), judged on 4-copy rows.
  CORR: Walk-loop restructuring: lo 0, realized 3% (#181) to 5-14% (#171) of 4t wall, theoretical hi about 30%. Ir-only cuts of 3% or less: at most 1.5%, below the bench's resolution.

- FALLS: No-gos: more lanes / co-resident 16/L2 (0 or slower); longer prefetch distance (0); larger tile (worse); fused post-walk (0); psi^2 pair walk (premise: 'the per-core ceiling is now identified').
  WHY: Mixed.
- Survive on measurement: lanes / co-resident (sweeps flat at 4 copies; 16/L2 slower on 7 of 8 CPUs), tile (tile-size.md), fused post-walk (fused-postwalk.md).
- Prefetch distance survives for 4t only. Its premise 'latency already hidden' fails on Zen 5 at 1t (verdict 2).
- psi^2: the premise is refuted. The walk's 4t time tracks per-byte core work, so a pair walk's worth hinges on whether a 2-byte step costs less than two single steps in core work, not on lines. The prior CI losses (#6, #34) stand, and its rank (2) is not earned.
  CORR: Lanes, hints, segments, tile, fused post-walk: 0 at 4t. psi^2: unknown sign, likely negative with its bigram build.

MISSED: 1. The decisive contradiction is already in C's own phases runs: the unused-walk rows (bitmap, flagged, plain). On the 7763, kernels 3.3x apart in line rate all lose x2.12-2.16 at 4 copies. On the 8370C the heavier kernels lose more (x2.24 bitmap, x1.95 flagged, x1.46 room). Reproduce with /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/census/FC/scripts/phases_walks.py.
2. C's fit.py output contains the lane-independent penalty, but C never reported it. The intercept a rises x2.7-3.1 while lat is unchanged, and +2.75-3.97 c/B is already present at 8-16 lanes. Reproduce with /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/census/FC/scripts/sw.py.
3. No CI figure is a walk measured against a walking sibling. The sibling walks about 49-53% of the time (phases 4-copy shares; occupancy 41976148952: T-vector phase 2.85 of 4). The sweep's copies run the same rotation unsynchronised (examples/sweep.rs:12-18; sweep.yml:87-88). So 2*t1 is not a floor under any model.
4. Vendor primary sources, per-core miss capacity / L2 / L3 / SMT:
   - Zen 3 (EPYC 7763): MAB 24, L2 512 KB at 12 cycles or more, L3 average 46 (56665 rev 3.00, sections 2.12 and 2.6.3-4).
   - Zen 4 (EPYC 9V74): MAB 24, L2 1 MB at 14 or more, L3 50, retire queue 320 / 160 a thread, store queue 64 (57647, sections 2.12, 2.6.3-4, 2.10.3).
   - Zen 5 (EPYC 9V45): 124 outstanding misses, L2 1 MB at 14 or more with a 64 B path to L1, L3 46, retire queue 448 / 224, store queue 104, load queue 64 uncompleted (58455 rev 1.00). C called this count unknown.
   - All three Zens: memory request buffers, load queue and schedulers are watermarked under SMT; store, retire and op queues are statically partitioned (Tables 3 / 4 / 10, section 2.14).
   - Ice Lake (Xeon 8370C): the ORM (248966-050) gives no fill-buffer count. Golden Cove is '16 FB, 32->48 MLC'. Emerald Rapids (Xeon 8573C) is Golden Cove-class (INFERRED): 16 fill buffers, 48 MLC misses.
   - Ceiling = capacity / latency: 1.70, 2.17, 0.38 and 3.66 c/B on the 7763, 9V74, 9V45 and 8370C, against 4.33, 4.27, 2.89 and 7.27 c/B measured. No model is at its documented miss capacity at 1t.
5. #181 was a lines-neutral change, yet the 1t walk was 6.8% / 2.6% faster (9V45 / 9V74), against 'one thread at the ceiling'.
6. Power: #168 and #173 could not resolve 1.3% or less at 4t.
7. plain4 is measured beside plain passes, not beside a room walk (examples/phases.rs:23-29, main loop). C's '10.4 per core' and its ICX fill-buffer arithmetic both rest on it.
8. On the 8370C, the room and room+assembly rows differ by +/-10% at 4 copies: noise C did not carry into its yields.
9. Small arithmetic: the 8573C share of 55-67% does not follow from C's stated assumption (others x1.0-1.15 gives 64-68%; 55% needs others x1.4). The Zen 3 L2 latency is 12 cycles, not 14. Neither changes a conclusion.
10. What would decide core-side vs memory-side without a PMU: a sibling-probe mode. Run the walk on CPU 0 against CPU 1 running:
    (i) an L1-resident, uop-heavy loop;
    (ii) the plain chain;
    (iii) the room walk.
    Core-side predicts (i) about equal to (iii), both well above (ii). Memory-side predicts (iii) well above (i), with (i) about equal to (ii). Also add bitmap/flagged-style kernels to the 4-copy sweep on Zen 4/5. Not dispatched: sweep.yml can only run 1 copy and then N identical copies, and another phases run would land on a random model and only replicate point 1.
