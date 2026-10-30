# mibz2 research: merged hypothesis register

The companion to `2026-10-30-research-portfolio.md`. That document holds this round's
new candidates (N0–N3, D1), their theory, the rankings, the plans and the i9 queue.

**Sources.** This register merges four read-only audits of the record, kept outside the
repository:
- **M27:** the journals dated 2026-10-27 on master;
- **M28:** the journals dated 2026-10-28 to 2026-10-30 on master, and CHANGELOG.md;
- **BR:** the research/*, perf/* and scratch/* branches;
- **IS:** the GitHub issues and PRs #1–#116.

**Marks.**
- **[coord]:** a coordinator correction that overrides the audits where they conflict.
- **[checked]:** a fact checked read-only for this merge:
  - the v0.2.1 tag does not contain #116 (`git merge-base --is-ancestor`);
  - wikidata-pq master (dd9e1031) has `DEFAULT_BZ2_DECODER = "native"` and
    `BZ2_THREADS_CAP = 4` (`src/wikidata/dump.py` L154, L170), and pins mibz2 0.2.1 at
    d09edd91 (#213);
  - the question lists in section 3 were read from `origin/research/psi-census`
    (c32042b) and `origin/master` (ae942bd), and #6's body from the issue dump.
- *(inferred):* anything inferred rather than read.

Nothing was rerun.

## Conventions

- **Boundaries.** The register keeps the audits' terms and never pools figures across them:
  - **kernel**: inverse BWT alone (`kernels`, `bz2k`, scratch examples);
  - **T-vector phase**: build + walk in the coop grid, pinned;
  - **stage**: one stage timed alone (`phases.rs`, `bench/stages.py`);
  - **full decoder**: `.bz2` to output, `ab` paired A/B;
  - **application**: wikidata-pq's split or a release-shaped run;
  - **census/analysis**: untimed counts, proofs or code reading.
- **Machines.**
  - **VM**: the shared 4-vCPU dev VM. Its spec changed between documents (K10).
  - **CI**: GitHub 4-vCPU runners, which are 2 physical cores with SMT: EPYC 7763, 9V74 or
    9V45, Xeon 6973P-C or 8573C/8370C.
  - **i9**: i9-10900K, 10C/20T, 20 MB L3, DDR4-2666.
  - Thread or copy counts are given per figure.
- **Issue numbers.** `wpq#N` is lmmx/wikidata-pq. A bare `#N` is lmmx/mibz2.
- **Versions.**
  - 0.1.2: #12.
  - 0.1.3: #21, #24.
  - 0.1.4: #32.
  - 0.1.5: #37.
  - 0.1.6: instruments.
  - 0.2.0: #101, #103–#106.
  - 0.2.1: #114.
  - #116 is on master after the v0.2.1 tag **[checked]**.
- **Status.** Exactly one of open / conditional / superseded / rejected / validated / shipped.
  Where the audits disagree, it gives the most recent primary record's status and lists the
  disagreement in section 4.
- **Failure classes.**
  - I: mathematical or informational impossibility;
  - II: representation budget;
  - III: experimentally inferior at the deciding boundary;
  - IV: depends on the architecture or concurrency.
  - "(judgement)" marks a class assigned in this merge that no source records.

---

## 1. Alias table

| ID | Idea | Aliases | Sources (notes → files / issues / PRs / branches) |
|---|---|---|---|
| **(a) dependent transitions per byte** ||||
| P01 | Ψ² pair walk, 4 B/row, libsais construction | H1; super-alphabet; LF²/ψ²; E1; "pair walk, two bytes per load"; PSI2; PAIR-WALK; BZN-7; XR3 priority 1; #6's two open questions | M27 H1; M28 E1; BR PSI2; IS #6. literature-reading §2, pair-walk.md, research-synthesis §1, expert-review §7, bz2ngrs §7, external-report-3; #6, #31, #33, #34, #80, #83; perf/pair-walk; docs/e1-e2-results |
| P02 | LF[LF[i]] gather, 8 B/row pair table | #29; LF2; `bwt::Pairs` | M27 H1; BR LF2; IS #6 exp. 2; perf/lf2 |
| P03 | libsais walk parameters in the Ψ² port | E6 | M28 E6; BR PSI2; IS #6 |
| P04 | Precopy: copy repeats instead of walking | H2; #9 item 2; #30; #36; KP 2011 forward chains; KKP s-runs; PRECOPY | M27 H2; BR PRECOPY; IS #9. precopy-forward-chains.md, bloom-inverse-bwt.md, coop §5c; perf/precopy, perf/precopy-chains |
| P05 | Pre-walk scan for repeat candidates (Bloom) | R12 | M27 R12; bloom-inverse-bwt.md |
| P06 | Tunnels as copy opportunities | LT-T; bwt-inversion-literature hypothesis 1; triage queries 3–4 | M27 LT-T |
| P07 | "Wikidata is in Precopy's regime" | LT-W; bwt-inversion-literature hypothesis 3 | M27 LT-W |
| P08 | LZP pre-pass | R13; #9 item 3 | M27 R13; IS #9 |
| P09 | KKP interleaved parts with stored ends | LT-K | M27 LT-K |
| P10 | Auxiliary sampled indices | R10 | M27 R10 |
| P11 | Length-capped move structure / run-length ψ | M2; #42; E8 (its reopening, never run); LIT-1; WS-14; MOVE | M27 M2, LT-C; M28 E8, LIT-1, WS-14; BR MOVE; IS #42/E8. move-structure-walk.md, mlp §3, walk-sweep, literature-study 01; #42, #43, #46; scratch/move-structure |
| P12 | Run-aware LF traversal (GNP) | LT-A; bwt-inversion-literature hypothesis 2; triage query 1 | M27 LT-A |
| P13 | Two-tier translation directory | LIT-18; SR-TWOTIER; phase-2 Finding 1; feasibility "two-level" row | M28 LIT-18, CPF-Q3; BR SR-TWOTIER; succinct §9, §14, §16 |
| P14 | Rank/select and wavelet structures in the walk | SR-RANKSEL; succinct §10 Q3 (alphabet-partitioned select) | BR SR-RANKSEL |
| P15 | Blocked rank directory over L | SR-BLOCKRANK; feasibility "blocked rank" row | BR SR-BLOCKRANK; M28 CPF-Q3 |
| P16 | Compressed Ψ near nH₀ with class 1–2 access | #79 theory; succinct §10 Q1 | IS #79 theory; BR SR-OQ |
| **(b) transition cost** ||||
| P20 | Flagged segment starts, joined-walk bookkeeping | #4; #12; W7; pbz2 START bit 31; WALK-SHIPPED (#12) | M27 W7; BR WALK-SHIPPED; IS #4; perf/walk-flagged-starts |
| P21 | 32 lanes + software prefetch + 512 segments | #5; #21 | M27 W1, W3; IS #5; perf/walk-lanes |
| P22 | How much of the gain is prefetch vs interleaving | W3; #39 Q1; mlp lead 1 | M27 W3; IS #39-Q |
| P23 | Re-tune lane or segment count after the room walk | W1; #41 sweep; WS-6 (i9 bench-variants arms); BZN-H3; BZ-H3; E10 addition (lanes8/lanes16) | M27 W1; M28 WS-6, BZN-H3; BR BZ-H3; IS #5, #39-Q; perf/lane-sweep |
| P24 | CI's 2-core runners selected for a sharp knee | WS-6 hypothesis; #39 "version ladder on the i9" | M28 WS-6; IS #39 |
| P25 | Prefetch hint T1/L2/NTA in the production walk | W2; #39 Q5; mlp lead 5 | M27 W2; IS #39-Q |
| P26 | L2 prefetch in the cooperative walk | PF-L2 | BR PF-L2 |
| P27 | Instruction-window (AMAC/runahead) reading of #37 | W4; #39 Q3; mlp lead 3 | M27 W4; IS #39-Q |
| P28 | Intel AOP/DDP prefetchers and the software prefetch | W10; #39 Q2; mlp lead 2 | M27 W10; IS #39-Q |
| P29 | Software lanes vs misses in flight | #39 Q4; mlp lead 4 (Lemire); LIT-20 | IS #39-Q; M28 LIT-20 |
| P30 | Evenly spaced vs pseudo-random starts | W6; #4 step 5 | M27 W6, C8; IS #4 |
| P31 | Shared-offset room walk | R2; #37 | M27 R2; IS #37; perf/walk-room |
| P32 | Adjacent-line hardware prefetcher doubles traffic | W9; EW-NOTES-PF | M27 W9; M28 EW-NOTES-PF |
| P33 | TLB reach and huge pages | W11; #40; WS-10; LS-H6 (mibz2 half); EW-NOTES-THP | M27 W11; M28 WS-10, LS-H6; IS #40 |
| P34 | Per-block kernel choice (plain walk for few-run blocks) | LT-D; #9 item 1; #27; WALK-CHOICE; SR-CYCLES cycle 7 | M27 LT-D; BR WALK-CHOICE; IS #9; perf/walk-choice |
| P36 | Compact Ψ: push out the throughput–memory frontier | FR-1; #102; LIT-25 ("the fundamental question"); feasibility Q5; frontier page; LT-C | M28 FR-1, CPF-Q5, LIT-25; IS #102; frontier.md, seward-frontier-*, compact-psi-feasibility.md |
| P37 | 24-bit link-only table `u`, byte from a 32-row window | CPSI-A; #112; #79 milestone C; "32-row symbol descriptor" (feasibility Q1–Q2 **[coord]**); LIT-13; `v` control arm | BR CPSI-A; M28 CPF-Q1–Q2, LIT-13; IS #79 milestone; research/compact-psi |
| P38 | Cheaper recovery of F's byte | CPSI-F; feasibility Q1–Q2 variants; `win` u16 + boundary flag; 64/128-row windows; byte in the link's line (`e`, #130) | BR CPSI-F; M28 CPF-Q1–Q2, LIT-13 |
| P39 | 20-bit dense ψ and the libbzip2 ll16+ll4 split | E7; #79; #91 `c` and `b` arms; SR-DENSE; CTV | M28 E7; BR CTV, SR-DENSE, E7; IS #79; research/compact-tvector |
| P40 | Fixed 64-byte line records FOR-14 / EF-12 / exceptions | SR-FIXLINE; SR-EF16; LIT-15; LIT-6; #130 `h` (and `e`) | BR SR-FIXLINE, SR-EF16; M28 LIT-6, LIT-15 |
| P41 | Few-plateau (two-plateau) group descriptor | LIT-9; LIT-10; SR-PLATEAU; #99 | M28 LIT-9, LIT-10; BR SR-PLATEAU; IS #99; research/psi-census |
| P42 | Segment-end markers without per-row bits (B2/B3) | MA-1; MA-2; #100; LIT-5 | M28 MA-1, MA-2, LIT-5; IS #100 |
| P43 | A separate RUN fast path | SR-RUNFAST; agenda III | BR SR-RUNFAST |
| P44 | Alphabet-specialised byte recovery | SR-ALPHA | BR SR-ALPHA |
| P45 | Fixed vs adaptive partition boundaries | LIT-11 | M28 LIT-11 |
| P46 | Per-group encoding chosen by hardware cost | LIT-12; SR-CYCLES cycle 2; conclusions Q5 | M28 LIT-12; BR SR-CYCLES |
| P47 | Select without PDEP; in-line extraction questions | LIT-16; LIT-24 | M28 LIT-16, LIT-24 |
| P48 | Compression cuts lines occupied, not traffic per step | LIT-17; CPSI-READS | M28 LIT-17; BR CPSI-READS |
| P49 | Is the smallest structure the fastest? | LIT-8 | M28 LIT-8 |
| P50 | Per-segment variation bound; hybrid fixed-address groups | LIT-2; LIT-3 | M28 LIT-2, LIT-3 |
| P51 | Cost and timing distribution of exceptional steps | LIT-7; SR-DISC A₂; overview §5 clustering; SR-CYCLES 9.3 | M28 LIT-7; BR SR-DISC, SR-CYCLES |
| P52 | Discontinuity theory (d bounds, K₃₂ from L's runs) | SR-DISC (proofs) | BR SR-DISC |
| P53 | Several ψ cycles in a cyclic BWT | LIT-22 | M28 LIT-22 |
| P54 | Adaptive representation by concurrency | FR-2 | M28 FR-2 |
| P55 | Dependent-successor throughput benchmark, 1–32 lanes | LIT-14 | M28 LIT-14; IS #99 |
| P56 | libsais cyclic inverse as reference R | R1 | M27 R1 |
| P57 | libsais's lead is a memory-pressure effect | E2; KNEE-MATCH | M28 E2; BR PSI2, KNEE-MATCH |
| P58–P64 | libsais mechanism hypotheses | LS-H1…LS-H7 (path H1–H8) | M28 LS-H1–H7 |
| P65 | E3's libsais half (phase attribution) | E3 (libsais part) | M28 E3; IS #76 |
| P66 | Shipped cooperative kernel vs libsais | #107 rematch; REMATCH; CD-6 | BR REMATCH; M28 CD-6; IS #107; research/libsais-rematch |
| P67 | Intel IPP as a timing reference | R9 | M27 R9 |
| P68 | rapidgzip leads (windows, memcpy, allocator, block finder) | R11 | M27 R11 |
| **(c) non-navigation work** ||||
| P70 | Two Huffman symbols a lookup | R4; #24; #8 step 1; HUFF-2SYM; perf/huffman-2sym | M27 R4; BR HUFF-2SYM; IS #8 |
| P71 | Smaller Huffman LUT | R3 | M27 R3 |
| P72 | Parallel scanner from speculative block markers | H5; #10 | M27 H5; IS #10 |
| P73 | MTF head in registers | R6; #25; #8 step 2; perf/mtf-register | M27 R6; BR MTF-REG; IS #8 |
| P74 | MTF store-forwarding chain / front 32 B in a register (§15.4) | COOP-STORE; MTF-REG lead; succinct §10 step 4 | BR COOP-STORE, MTF-REG |
| P75 | Seward's sliding-block MTF | R7 | M27 R7 |
| P76 | MTF count checkpoints for the team build | E5; #78; COOP-RECOUNT; COOP-CP; #101 `mtf_cp2p` | M28 E5; BR COOP-RECOUNT, COOP-CP; IS #78 |
| P77 | Batched team: overlap MTF with another block's inversion | BATCH-TEAM; CPF-6.3a; plan PR 9; "phase-aware teams" (expert review) | BR BATCH-TEAM; M28 CPF-6.3a |
| P78 | Fused front end / in-place T-vector | H4; #8 step 3; WS-13 ("#8's fusion") | M27 H4; M28 WS-13; IS #8 |
| P79 | Run lanes in the build | H3; #7 (b); #32; BUILD-RUNS | M27 H3; BR BUILD-RUNS; IS #7 |
| P80 | Build in four interleaved quarters | R5; #7 (a); #23; BUILD-4LANE | M27 R5; BR BUILD-4LANE; IS #7 |
| P81 | One-core build in one pass | E11 (one-core half); #81; #93; #114; ONEPASS; OPB-1 | M28 E11, OPB-1; BR ONEPASS; IS #81; perf/build-one-pass, perf/one-pass-build |
| P82 | Cooperative fused build | E11 (coop half); #81; #95; #104; COOP-FUSED | M28 E11; BR COOP-FUSED; IS #81 |
| P83 | First-column packing (A′), write-only scatter | BZN-H1; BZ-H1; bz2ngrs h1; #79 A′; #91 `e` arm | M28 BZN-H1; BR BZ-H1; IS #74 |
| P84 | Build compact Ψ directly from L in one pass | LIT-4; SR-DIRECT; phase-2 Q5; overview construction question | M28 LIT-4; BR SR-DIRECT |
| P85 | Remove the segment-assembly copy (RLE1 over slices) | O3; #48 step 2; #49; WS-15 (#48 part) | M27 O3; M28 WS-15; IS #48 |
| P86 | Non-temporal stores on the output path | O2; #48 last step | M27 O2; IS #48 |
| P87 | Walk output in L's freed buffer | E9; BZN-H2; BZ-H2; bz2ngrs hypothesis 2 | M28 E9; BR BZ-H2; IS #79 |
| P88 | Randomised blocks (bzip2 ≤ 0.9.0) | A11 | M27 A11 |
| P89 | ARM / aarch64 kernels and timing | A10; A2 condition 5 | M27 A10 |
| **(d) scheduling, MLP, concurrency** ||||
| P90 | Cooperative inversion: k cores share one block's table | COOP; #73; #65 lead; CD-1; EW-LEAD-1; COOP-DEC-MODEL | BR COOP; M28 CD-1, EW-LEAD-1; IS #65 lead/#73; research/coop-*; perf/coop-* |
| P91 | Team shape: 2 vs 3 cores | CD-5 | M28 CD-5; BR C1 |
| P92 | Policy at 6 threads (3 × 2) | CD-2 | M28 CD-2 |
| P93 | The L3 budget is per L3 domain | COOP-P7 | BR COOP-P7 |
| P94 | No two team threads on SMT siblings | COOP-P6 | BR COOP-P6 |
| P95 | Cooperative kernel at production's per-core speed | E4; #77; COOP-KERNEL; XR3-REG | M28 E4, XR3-REG; BR COOP-KERNEL; IS #77 |
| P96 | Residency vs overlap (why 3x3 wins) | E3 (coop half); #76; COOP-MECH | M28 E3; BR COOP-MECH; IS #76 |
| P97 | Teams are instruction-bound | COOP-IB | BR COOP-IB |
| P98 | Cross-core line handoffs cost | COOP-C2C; #81 handoff motivation | BR COOP-C2C; IS #81 |
| P99 | Pool wake-up latency | COOP-WAKE; CD-4 (wake-ups) | BR COOP-WAKE; M28 CD-4 |
| P100 | L3-capacity cap rule (k ≈ 0.7 × L3) | EW-NOTES-MRC; WS-12 rule; COOP §1 policy; #65 formula | M28 EW-NOTES-MRC, WS-12; BR COOP-§1 |
| P101 | mibz2 worker cap below 6 cores; autodetect | #65 (mibz2 side); WS-12; EW-NOTES-AUTO | IS #65; M28 WS-12, EW-NOTES-AUTO |
| P102 | Level-aware team cap formula | LP-FORMULA | BR LP-FORMULA |
| P103 | Teams on every core at level ≤ 3 | LP-R1; #113; #115; LAP-MIX; CD-3 | BR LP-R1, LAP-MIX; IS #113/#115; M28 CD-3; research/level-policy, perf/level-aware-plan |
| P104 | One core a block at -2/-3 (over the RSS budget) | LP-WORKERS | BR LP-WORKERS |
| P105 | Default at 6 threads runs at 0.76–0.83× its 20-thread speed | LP-6T | BR LP-6T |
| P106 | Phase-aware gating (≤ K workers in `invert`) | WS-9; #39 walk gates; gate2…gate6 | M28 WS-9; IS #39 |
| P107 | Admission window starves workers | O4; #39 head-of-line blocking | M27 O4; IS #39 |
| P108 | Run-queue delay inflates block times | O5 | M27 O5 |
| P109 | CI's four-thread ceiling is SMT | O6; #39; #53–#55 | M27 O6; IS #39 |
| P110 | The full-occupancy remainder | O7 | M27 O7 |
| P111 | Shared L3/bandwidth binds on CI's four copies | O1 | M27 O1, C7 |
| P112 | The many-core knee: L3 capacity, then queueing | WS-2; WS-7; #39/#60/#63/#66 | M28 WS-2, WS-7; IS #39 knee |
| P113 | Table bytes (block size) move the knee | WS-11; #68; KNEE-CAP; #112 hit-share result | M28 WS-11; BR KNEE-CAP; IS #39/#68 |
| P114 | One knee shared across decoders / rate per worker | WS-5 | M28 WS-5 |
| P115 | Past the knee, throughput ≈ 28 GB/s ÷ DRAM bytes/B | KNEE-28GB; COOP-P5; WS-2 sub-question | BR KNEE-28GB, COOP-P5; M28 WS-2 |
| P116 | DDR4 four-activate window as a second knee | EW-LEAD-3 | M28 EW-LEAD-3 |
| P117 | Request cost grows with cores at a high hit share | KNEE-2ND; KNEE-ISO (its test) | BR KNEE-2ND, KNEE-ISO |
| P118 | Write traffic past the knee | WS-8; KNEE-BUILDSTORE | M28 WS-8; BR KNEE-BUILDSTORE |
| P119 | K blocks in lockstep per core | W8 | M27 W8 |
| P120 | AMAC / dynamic lane scheduling, direct table | E10; AMAC (rematch); LIT-21; feasibility Q4 | M28 E10, LIT-21, CPF-Q4; BR AMAC; IS #39/E10 |
| P121 | AMAC-staged payloads for a two-tier format | LIT-19 | M28 LIT-19 |
| P122 | Batched segment dispatch | COOP-BATCH | BR COOP-BATCH |
| P123 | Work-optimal parallel inversion (Edwards & Vishkin) | COOP-EV | BR COOP-EV |
| P124 | GPU rank-and-scatter inversion; scanned iMTF | R8; COOP-GPU | M27 R8; BR COOP-GPU |
| P125 | Early warning: CPU amplification from 1 to 4 threads | EW-1; #69 | M28 EW-1; IS #69/#72 |
| P126 | Contention fingerprint and its confound | EW-6; #71 | M28 EW-6; IS #71 |
| P127–P133 | Early-warning open leads | EW-OPEN-1, -2, -4, -5; EW-LEAD-2; EW-NOTES-USL; EW-NOTES-DIRIGENT | M28 |
| P134 | Kernel+MTF (1,707 MB/s) vs decoder (1,256): unattributed gap | CPF-6.3b; CD-4 (placement) | M28 CPF-6.3b, CD-4 |
| **(e) measurement and harness** ||||
| P140 | Warm-repeat bias | W5; #45; #47 | M27 W5; IS #45/#47 |
| P141 | A/B order bias (head always first) | EW-3; #69; #72 | M28 EW-3; IS #69/#72 |
| P142 | Per-thread CPU accounting bug | EW-2 | M28 EW-2 |
| P143 | Sampler cost | EW-4 | M28 EW-4 |
| P144 | Footprint and queue ledger | EW-5; #70 | M28 EW-5 |
| P145 | Occupancy trace under team plans | CPF-6.3c; OCC-TRACE; #116 | M28 CPF-6.3c; BR OCC-TRACE |
| P146 | PMU availability (CI none; VM none; i9 yes) | R14; #40 (step 1); #87 | M27 R14; IS #87 |
| P147 | Busy-desktop plateau at ~470 MB/s | WS-1 | M28 WS-1 |
| P148 | memcpy counter calibration | WS-3; #57 | M28 WS-3 |
| P149 | Slow runs after an idle gap | WS-4; #39; #58–#60 | M28 WS-4; IS #39 |
| P150 | Native block table size: by count vs by time | KNEE-SIZE; #110; #111 | BR KNEE-SIZE; IS #39/#68 |
| P151 | Raw offcore excess = L2 streamer prefetches | COOP-OFFCORE | BR COOP-OFFCORE |
| P152 | Six pilot-harness negatives | COOP-NEG6 | BR COOP-NEG6 |
| P153 | Code-layout control for ~2% kernel A/Bs | COOP-LAYOUT | BR COOP-LAYOUT |
| P154 | The scatter compiles two ways | COOP-SCATTER | BR COOP-SCATTER |
| P155 | The given-counts research build's own overhead | COOP-GIVEN-OVH | BR COOP-GIVEN-OVH |
| P156 | l3x3 is 7–8% faster than its twin x3x3 | REMATCH-GAP | BR REMATCH-GAP |
| P157 | The -6 control slowdown under #115's build | LAP-6 | BR LAP-6 |
| P158 | Controlled version ladder | #52 | IS #52 |
| P159 | Correction of libsais's decode path (symbol rows) | LS-COR | M28 LS-COR |
| P160 | bz2ngrs's gap is Wikidata's inverse BWT | BZN-4 | M28 BZN-4 |
| P161 | Fair multicore comparison with bz2ngrs; `bench-compete` | BZN-5; BZN-6; BZ-MC | M28 BZN-5, BZN-6; BR BZ-MC |
| P162 | External report 3's ranking | XR3-RANK | M28 XR3-RANK |
| P163 | Literature tasks and citation checks | LT-Q; LT-N; LIT-23 | M27 LT-Q, LT-N; M28 LIT-23 |
| **(f) application (wikidata-pq)** ||||
| P170 | mibz2 (prototype `bz2k`) as wikidata-pq's bz2 decoder | A1 | M27 A1; wpq#194, wpq#197 |
| P171 | Making native the default (seven conditions) | A2 | M27 A2 |
| P172 | wikidata-pq's thread cap of 4 | #65 (application side); wpq#210–#212; EW-OPEN-3; CPF-6.2; COOP-P1 | IS #65; M28 EW-OPEN-3; BR COOP-P1 |
| P173 | 8 threads (4 teams of 2) in the release | wpq PRs #215–#217 **[coord]** | coordinator; wpq journal `docs/journal/2026-10-30-bz2-threads-teams/` |
| P174 | Pin mibz2 0.2.x in wikidata-pq | CPF-6.1; wpq#213 (d09edd91) | M28 CPF-6.1 **[coord]** |
| P175 | No fresh pages per chunk | A3 | M27 A3 |
| P176 | No thread oversubscription in the split | A4 | M27 A4 |
| P177 | Finalise's Parquet decode built for AVX2 | A5 | M27 A5 |
| P178 | Parquet encode with levels from the decoder | A6 | M27 A6 |
| P179 | Schema-compiled claims JSON → Arrow | A7 | M27 A7 |
| P180 | wpq#194 dead ends (asm/AVX-512 walk, zstd, bandwidth, line split) | A8 | M27 A8 |
| P181 | gz input instead of bz2 | A9 | M27 A9 |

---

## 2. The register

Entries give Status, Boundary/machine/figures, Condition, Class and Sources. "Cond." quotes the
recorded reopening or falsification condition.

### (a) Dependent-transition count per byte

**P01 Ψ² pair walk (4 B/row, libsais construction).** Status: **rejected** (for production; #6
closed not_planned; "It stays closed for production").
- kernel, VM, 1 pinned core, 2.8 GHz (#6 feasibility): the pair pass alone cost 5.86 / 5.96 /
  13.40 c/B (wikidata / text / records). The best-case saving was 5.19 / 4.30 / 5.12.
- kernel, VM (H1 cheap test): libsais-style build 18.7 c/B against `build` 6.1. With a sequential
  bigram histogram, 12.46 against 5.68. 971 distinct pairs a block; 0.12 fastbits probes a step.
- stage, VM, 1 thread (#34): build 7.1–7.6 against 2.5 c/B; walk ≈10 against ≈10.4–10.8; total
  17.4–17.6 against 13.1 (libsais 10.9).
- full decoder, CI, 10 paired rounds, #34 against 0.1.4: Wikidata +40.6% at 1 thread, +24.3% at 4
  (CPU +29.2% at 4); text +51.5% / +30.4%; random +48.8% / +23.8%; peak RSS +1–8%.
- full decoder, i9, 10 rounds, E1 (pairwalk a3687d8 against v0.1.4): 0.691 (0.649–0.736) at 1
  thread, 0.816 at 4, 0.991 at 6, **1.107** (1.070–1.145) at 8, **1.117** at 10. Best against
  best **0.856** (0.833–0.881).
- Cond.: "reopens only if E3 (#76) attributes that low-pressure lead to parts a port can carry
  (its build, lanes or prefetch), with a new criterion fixed before the run". Also: "If the walk
  ever becomes latency-bound again … re-run this check", and "Revisit if #8's fused front end"
  makes the pair histogram free. BR keeps it as "ψ² only with an instruction budget" (27.8 against
  libsais's 23.2 instr/B).
- Class: III (each step costs about twice as much and the build 3×) and IV (it pays only past the
  i9's knee, at 8–10 workers). Recorded.
- Sources: see the alias table. E1 met neither its registered pass nor its fail (K53).

**P02 LF[LF[i]] gather, 8 B/row.** Status: **rejected** (#29 closed unmerged).
- full decoder, CI (Xeon 8573C), 10 paired rounds: Wikidata +9.3% (+8.7, +9.9) at 1 thread, +20.7%
  (+18.6, +22.8) at 4; peak RSS 77 → 105 MB; every corpus slower.
- Cond.: none recorded. literature-reading: "#29 tested constructions their papers do not use".
- Class: II (8 B/row; the 7.2 MB table exceeds Skylake's STLB reach, by arithmetic only). Recorded
  in BR. M27 reads it as III/IV. #40 later measured page walks at 1–2.5% on the i9 (P33).

**P03 libsais walk parameters in the Ψ² port (E6).** Status: **rejected** (never run; its
precondition E1 did not pass).
- No figures.
- Cond.: E1's reopening, plus E6's own bar: "≥ libsais's s4x1/p4 on the i9 and ≥ 1.0 × production
  on CI's 2-core runners".
- Class: none applies; it was never run (precondition not met).

**P04 Precopy (copy repeats instead of walking).** Status: **rejected**.
- census, VM (#9 offline): 65.1% of Wikidata bytes copyable in runs ≥ 32 B, mean 200 B. In the
  32-lane walk order, 44.7%, ~1,250 chains a block.
- full decoder, CI, 10 rounds, #30 (backward search): +118.2% / +117.9% (1 / 4 threads); RSS
  77 → 167 MB.
- full decoder, CI, #36 against 0.1.4 (forward chains): Wikidata +18.0% / +6.0% with 31.5%
  copied; text +9.7% / +4.8%; pattern +20.4% / +19.9%; runs +11.1% / +9.5%.
- stage (walk), VM: +30% with 31.5% copied, against +22–27% with 0% copied. Branch-free
  bookkeeping +31–47%.
- Cond.: "Not closed: cheaper walk steps" (precopy-forward-chains). Coop §5c adds:
  "`perf/precopy-chains` (kept) measured at 8–10 workers on the workstation, after the registered
  run and the pair walk's lead". No record of that run.
- Class: III (the steps it removes are the cheap cached ones; every walked step pays the
  neighbour test). Recorded.

**P05 Pre-walk scan for repeats (Bloom).** Status: **superseded** (with P04).
- Nearest evidence: walk stage, VM, #36: +22–27% overhead with 0% copied.
- Cond.: "if the forward-chain experiment shows the per-step compare is the cost". The data met it,
  but Precopy closed for a separate reason.
- Class: III, via P04.

**P06 Tunnels (Baier) as copy opportunities.** Status: **superseded** (same structure as KKP's
s-runs and KP's chains, P04).
- Tunnel planning itself is a format change.
- Cond.: as P04.
- Class: I (judgement) as a format technique; III via P04 as a decoder technique.

**P07 "Wikidata is in Precopy's regime".** Status: **rejected**.
- census, VM: n/r 13.85, below KKP's n/r ≥ 50 and below the paper's smallest dataset (29.2). #36
  copied 31.5% and was still slower.
- Cond.: none recorded.
- Class: III.

**P08 LZP pre-pass.** Status: **rejected**.
- No figures. It is encoder-side and "can't apply to decoding standard bzip2 streams".
- Class: I (judgement): the information is not in a standard stream.

**P09 KKP interleaved parts with stored ends.** Status: **superseded** by flagged pseudo-random
starts (#4/#12), which need no metadata.
- Class: I (judgement): the ends would have to be stored in the format.

**P10 Auxiliary sampled indices.** Status: **rejected**.
- "A bzip2 stream carries only `orig_ptr`, so they can't exist for our input."
- Cond.: none; it needs a format change.
- Class: I (judgement): the format carries no such samples.

**P11 Length-capped move structure (run-length ψ).** Status: **rejected** (#42 closed
not_planned). E8, the four-copy re-run, was **never run [coord]**.
- census, VM, 233 blocks ≥ 65,536 rows: r/n 0.0722 (n/r 13.85).
  - c = 8 / 4 / 2 / 1: mean fast-forwards 0.534 / 0.416 / 0.342 / 0.297; size 0.635 / 0.695 /
    0.821 / 1.067 B/row.
  - Uncapped: 7.475 fast-forwards, max 3,160.
- kernel, VM ("Xeon @ 2.80GHz", 1 MB L2, 33 MB L3), 1 thread, 233 Wikidata blocks, best of 3,
  two runs:
  - move build + room-style walk against shipped build + `walk_segments_room`: **+91.5% /
    +89.7%** (c = 2), +94.2% / +97.8% (c = 4), **+106.9% / +107.6%** (c = 8). That is **+90% to
    +108%** against a criterion of ≤ −10%.
  - one chain against the plain walk: **−60.5% / −59.3%** (c = 2) to −63.2% / −62.7% (c = 8).
  - about 41 instructions a move step against 16 a T-vector step. Interleaving speeds the move
    walk 1.4–1.9×, against 6.7× for the T-vector.
  - "Taking the whole 25% off [today's arms] still leaves (3) at about +50%."
  - An unreplicated 6-block contention check: room walk 19.4 c/B against move walk 16.0–20.3.
- Cond. (E8): "move build + walk at least 10% under build + room walk per copy, with four copies
  at once. Otherwise this closes for good." The workstation adds: "measure it at 8–10 workers, not
  only 1–4". Literature study: "Nothing in papers 1 or 2 reopens #42".
- Class: III (each interleaved step is dearer; "a finding about the schedule, not about
  compressing ψ"), with IV (the result depends on the walk being instruction-bound rather than
  memory-bound). Recorded.
- Note: #42 is the move-structure negative, not a Ψ² result (K47).

**P12 Run-aware LF traversal (GNP).** Status: **superseded**. It is realised as run lanes in the
build (P79, shipped) and as the move structure in the walk (P11, rejected).
- "For inverting a plain BWT they add nothing past the run lemma."
- Class: none applies; it was superseded by its realisations, not failed.

**P13 Two-tier translation directory.** Status: **open** (untimed).
- census v3: **5.70 bits a row** (0.641 MB a block: descriptors 225 KB, payloads 398 KB,
  exceptions 18 KB); per block 3.98–7.89.
  - A second dependent load on 12.0–36.3% of steps (median 22.0%).
  - Random input: 18.14 bits, 100% second load. Periodic input: 2.86 bits.
- Model: chain ≈53 cycles against 43. "Predicted to lose inside the L3, can win only past the
  knee."
- Cond.: n/a. Open question: whether the second dependent load destroys the overlap.

**P14 Rank/select and wavelet structures in the walk.** Status: **rejected** (succinct review's
first verdict, by analysis).
- Each step becomes several dependent operations (access class 4).
- Cond.: none recorded.
- Class: IV (recorded): dependent-load depth in an interleaved walk.

**P15 Blocked rank directory over L.** Status: **rejected** ("ruled out for the ψ walk (the walk
needs select, not rank)").
- Class: I (recorded): the structure answers the wrong direction.

**P16 Compressed Ψ near nH₀ with class 1–2 access.** Status: **conditional** ("recorded, not
scheduled").
- Theory only.
- Cond.: "a representation in classes 1–2 … below 24 bits".
- Class: II (recorded, by argument).

### (b) Transition cost

**P20 Flagged segment starts.** Status: **shipped (0.1.2, #12)**. The flag sits in bit 31 of the
entry; raw per-lane write pointers; the idle-lane test is out of the hot loop.
- full decoder, CI, 10 rounds: Wikidata −10.7% (−11.3, −10.2) at 1 thread, −16.6% (−18.2, −15.0)
  at 4.
- stage, VM: walk 31.8 → 17.2 c/B.
- Pre-#12, wpq#194 local kernel: joined 14.1 c/B against unjoined 4.4. The ~10 c/B bookkeeping
  premise was "inferred …, not attributed by measurement".
- #69's audit: "#12 … stand[s] by a wide margin".
- W7's paged arena "was not faster". #4's step 5 (start spacing) is P30.

**P21 32 lanes + software prefetch + 512 segments.** Status: **shipped (0.1.3, #21)**.
- stage (walk), VM, 2.8 GHz:
  - T0 prefetch at 8 / 12 / 16 / 24 / 32 lanes: 14.63 / 11.97 / 11.17 / 11.22 / 11.51 c/B;
  - no prefetch: 16.42 / 15.41 / 15.31 / 16.24 / 16.39;
  - four copies: 12/256/no prefetch 50.99 against 32/512/prefetch 20.35.
- full decoder, VM: 0.809 (0.696, 0.942) at 1 thread, 0.742 (0.664, 0.829) at 4. CI decided the
  merge (bot comment).
- Class: n/a.

**P22 How much is prefetch, how much interleaving.** Status: **validated**, on one CI runner and
on the i9.
- stage (walk), CI, decode order, 5 rounds (run 40573190665): L1 prefetch saves 4% / 7% / 12% /
  14% / 13% / 13% / 13% at 8–64 lanes with one copy. With four copies: 11–16% at 8 lanes and
  15–22% at 32.
- full decoder, i9: pf-off 0.881 (0.852–0.912) of the build's best; 0.76× at 10 workers.
- Older figures: VM #21, 26–28%; the withdrawn warm sweep, 34%.

**P23 Re-tune lane or segment count.** Status: **rejected** ("Lane and prefetch tuning: closed").
- stage (walk), CI, decode order, 5 rounds:
  - one copy, L1 prefetch, 8 / 12 / 16 / 24 / 32 / 48 / 64 lanes: 8.01 / 6.36 / 5.35 / 4.87 /
    **5.00** / 5.15 / 4.96 c/B;
  - four copies: 11.97 / 10.60 / 10.23 / 10.28 / **10.37** / 10.46 / 10.83;
  - "no setting is faster beyond noise" at one copy (32 lanes NTA −1.0% (−6.0, +4.3));
  - 256, 1,024 and 2,048 segments within noise at one copy; with four copies, 2,048 is 8% slower
    on two of four CPUs.
- full decoder, i9, bench-variants, 6 rounds, best against best: v0.1.5 1.006 (0.972–1.040),
  pf-l2 0.989, v0.1.4 0.976, lanes16 0.955 (0.928–0.983), pf-off 0.881, lanes8 0.853, pf-nta
  0.610. At 10 workers: lanes16 0.86×, lanes8 0.71×.
- Cond. (E10 addition): "If the walk is memory-bound with misses in flight well below the fill
  buffers, the next test is the lane count under pressure (lanes8/lanes16 at 4–10 workers on the
  i9)". That has already been measured as above; the addition doesn't cite it (K27).
- Class: III, with IV (CI: one runner model only).

**P24 CI's runners selected for a sharp knee.** Status: **rejected** (the hypothesis was
falsified).
- full decoder, i9: v0.1.2's 8/4 ratio 0.78 (0.77–0.79), not ≥ 0.85. v0.1.2 → v0.1.3: +21–28% at
  6–10 threads. v0.1.3 → v0.1.5: +14–20% at 2–4 threads and nothing measurable at 6–10.
- "The history steepened the normalised curve by raising the low-concurrency end."
- Class: III.

**P25 Prefetch hint T1/L2/NTA in the production walk.** Status: **conditional**.
- stage (walk), CI, decode order: at four copies, 16 lanes L2 9.70 against the default 10.37 c/B,
  faster on all four CPUs (−3.8%, −6.3%, −8.9%, −6.7%). At one copy the same setting is 23% slower
  (+17%, +29%).
- full decoder, i9: pf-l2 0.989 (0.962–1.018), equivalent; pf-nta 0.610.
- Cond.: "must win on both runner types or be gated by vendor", or a gate by vendor or thread
  count that the decoder A/B confirms at 1 and 4 threads.
- Class: IV (concurrency-specific: the best setting flips with contention).

**P26 L2 prefetch in the cooperative walk.** Status: **open** (not run).
- Teams hit full fill buffers 0.194/cycle (l3x3) against p4's 0.244 (i9 counters).
- Recorded as "the cheapest test the counters point at, about a line in the cooperative walk".

**P27 Instruction-window reading of #37.** Status: **open** ("lost its main evidence. It is not
refuted").
- In the decode-order sweep, both curves flatten near 24 lanes. The counter test needs a PMU.
- Cond.: "a new discriminating test before any experiment that adds instructions to the step".

**P28 Intel AOP/DDP prefetchers.** Status: **open**.
- No figures: CI has no PMU (#87), and no Intel-runner prefetch-on/off comparison exists. AOP's
  stated conditions "don't match the walk".

**P29 Software lanes vs misses in flight.** Status: **open** (arithmetic only).
- Vendor buffers: 24 on Zen 3/4, 16 on Golden Cove. AMAC's Fig. 6 saturates at ~10 in flight.
- For the direct table, lanes8 and lanes16 measured worse (P23).

**P30 Evenly spaced vs pseudo-random starts.** Status: **open** (never swept).
- wpq#194 (VM): evenly spaced starts "clustered in the text and stalled the lanes", no figure.
  pbz2 uses evenly spaced rows and ties mibz2.
- Marker audit, untimed model: JSON 400k evenly spaced makespan 1.30 against random 1.147 (P42).
- Recorded next step: "A sweep of lanes × starts × spacing … would settle it".

**P31 Shared-offset room walk.** Status: **shipped (0.1.5, #37)**.
- full decoder, CI, 10 rounds:
  - EPYC 9V74 (db3593d): −10.3% (−10.6, −10.0) at 1 thread, −9.1% (−12.2, −5.9) at 4;
  - EPYC 9V74 (a09c91b): −10.5% / −2.8%;
  - Xeon 6973P-C (6c645ef): −5.8% / −2.7% (−4.1, −1.3);
  - tag to tag v0.1.4 → v0.1.5 (9V74): −9.5% / −7.9%.
- stage (walk), Xeon: 4.37 → 3.81 c/B (−14.2%).
- full decoder, i9: v0.1.3 → v0.1.5 +14–20% at 2–4 threads, nothing at 6–10.
- #69: the 1-thread result stands; the 4-thread −2.7% is "within reach of the bias".

**P32 The adjacent-line hardware prefetcher doubles traffic.** Status: **rejected** as a library
lever: "turning prefetchers off is machine configuration".
- The MSR 0x1A4 bit-1 A/B (bare metal, root) was proposed (EW-NOTES-PF) and never run.
- Class: IV (judgement): a machine setting, outside the library.

**P33 TLB reach and huge pages.** Status: **rejected**. #40 closed as measured.
- full decoder, i9, `dtlb_load_misses.walk_active`: 1.1%, 1.1%, 1.0% and 2.2% of cycles at 4, 6,
  8 and 10 workers (multiplexed 0.8–2.5%). The criterion was "over about 10% and rising".
- kernel, VM (wpq#194): huge pages "no change".
- Bloom (published, i7-3770): 36.1 → 28.2 c/B with large pages; "doesn't transfer".
- Huge pages also need the user's decision (allocator rule).
- Cond.: "only if a different machine class or block size shows page-walk activity over about
  10%".
- Class: III (no headroom).

**P34 Per-block kernel choice (plain walk for few-run blocks).** Status: **rejected** (#27). The
pattern-corpus weakness stays open ("Choosing the kernel per block is open work").
- full decoder, CI (EPYC 7763), 10 rounds: Wikidata ±0.3%; pattern +21.0% (+20.1, +22.0) at 1
  thread and −18.8% (−21.1, −16.4) at 4.
- kernel, VM, cargo bench: pattern mibz2 93 MB/s, libbzip2 139, plain walk 210.
- Cond.: "a selector that wins on pattern at both thread counts without moving Wikidata" *(M27's
  reading)*, or a run-length ψ per block by r/n (depends on P11).
- Class: III.

**P36 Compact Ψ: push out the throughput–memory frontier.** Status: **conditional** ("Not approved
for implementation").
- The mechanism is validated (P113): the knee is at ~14.4 MB of live tables (0.7 of the i9's L3).
- kernel, i9, 20 rounds (#112): best compact arm u6 1,615 MB/s = **0.687×** l4x2 (2,345 MB/s);
  l5x2 0.746×.
- Modelled bound, i9: a fifth team gives ≤ 1.25× the inversion and ≤ ~1.1–1.15× the decoder
  (assuming inversion is 60–65% of a team's wall, "an estimate … not a measurement").
- Cond.:
  - success means "a measured configuration with more throughput and less measured memory than
    the best 32-bit configuration, on the same machine and workload";
  - pre-register "against `l4x2` and the complete decoder, not `p4`";
  - a machine with less L3 per core; laptop-class measurement.
- Class: IV (recorded): on the i9, four teams' tables already fit the L3. III for #112's arms.

**P37 24-bit link-only `u`, byte from a 32-row window.** Status: **rejected** in this form. The
scientific result (the knee moves) is validated.
- This is also the "32-row symbol descriptor": it is what #112 measured, not a new idea **[coord]**.
- kernel/T-vector phase, i9, one core a block, 20 rounds:
  - u peaks at 6 blocks (1,615 MB/s) against p at 4 (1,461);
  - hit share u6 0.926 against p6 0.793 (Δ 0.133 against a predicted ≥ 0.05);
  - u6/p4 **1.100** (1.093–1.108), meeting the gate of ≥ 1.05;
  - cost: **u1/o1 0.681** (0.678–0.684) against a stage-B gate of ≥ 0.85; v1/o1 0.829;
  - byte recovery costs 28% of the walk at 1 block and 16% at 4 (packing 3%). The always-paid
    part is two dependent L1 loads, a compare and a branch.
  - u6/l4x2 0.687.
- census: straddling groups ≤ 0.91% of groups by bound (≤ 255 boundaries), 0.28% of rows
  measured; 0.15% of rows take the step.
- VM first look: `v` build 1.55× production's (#114 quotes 1.28–1.49×, K62); u1 0.764 of p1.
- Cond.: "the next candidate is a cheaper byte recovery, not a smaller table" (P38).
- Class: II (recorded in BR): byte recovery and packing exceed the residency gain. M28 records III.

**P38 Cheaper recovery of F's byte.** Status: **open** (the registered next candidate).
- Candidates, none built:
  - one load: a u16 `win` entry with byte + boundary flag (56 KB; boundary table for ~1% of
    windows);
  - 64- or 128-row windows (14 / 7 KB);
  - the byte in the link's own line (`e`, #130).
- census at k = 5: 0.29% of rows in mixed windows, 0.15% stepped, 93% of mixed windows have one
  boundary.
- Cond.: stage B runs "only if the lookup gets cheaper". "The floor is set by production": the
  byte comes from the same load as the link.

**P39 20-bit dense ψ and ll16+ll4.** Status: **rejected** ("Ruled out: dense 20/21-bit packing
(measured)"). E7's registered i9 test was never run.
- kernel, VM, 8 rounds (#91):
  - `c` build 0.15× `t` (155 against 1,052 MB/s; elsewhere "85 against 1,052", K22); c walk
    0.74×; c4/t4 0.42;
  - `b` build 0.43×, walk 0.77×, b4/t4 0.67.
  - The VM's 32 MB L3 holds every table, so no residency effect can show.
- Cond.: E7's bar, "best absolute at 6–10 workers on the i9 beats p4's best", never run. Dense
  packing "comes after C, and only if C is encouraging"; C (P37) was not.
- Class: II (recorded): narrow stores and nibble read-modify-writes.

**P40 Fixed 64-byte line records (FOR-14 / EF-12 / exceptions).** Status: **open** (untimed:
#130 `h`; `e` and `h` "already designed").
- census, VM, exact: RUN 77.39%, FOR 20.73%, EF 1.40%, crossing 0.28%, too wide 0.21%, exceptions
  0.49%.
  - **16.16 bits a row** (1.818 MB a block), 16.12 in the feasibility review (K24). Other inputs
    16.0–16.6.
- Model: chain ≈45 cycles against ≈43; ≈25 against ≈17 instructions a step; ~15 cycles of decode
  on the chain (EF).
- Ordering: the release, then the cooperative decoder, then the #79 F-recovery audit, then
  compact designs. The navigation-cost harness "is not authorised yet".
- Pre-register "against `l4x2` and the complete decoder".

**P41 Two-plateau group descriptor (57 bits).** Status: **conditional** (marginal band; not
prototyped).
- census, VM, 233 full blocks (#99): two-plateau groups in one bucket = **4.18% of all rows**
  (18.46% of non-translation rows).
  - Per block 0.91–7.12%, median 4.23%. That is inside the pre-registered marginal band (3–8%);
    ≥ 8% would be credible.
  - 42.8% of non-translation rows are in groups of 9+ plateaus.
- The marker audit says it fits in 58 or 63 bits.
- Cond.: "Do not start a new compact-table prototype until the census and marker audit give us a
  reason to select one". The successor benchmark (P55) comes first if compact descriptors are
  pursued.
- Class: II (recorded): too few rows benefit.

**P42 Segment-end markers without per-row bits.** Status: **validated** (code audit and untimed
model).
- B2 (1 bit a group) passes; B3 (6 bits) passes with one more ALU operation.
- B1 fails balance with several cycles (10.6–16.0× the ideal). C (per-lane count) is impossible:
  lengths are unknowable without walking. D (a side table) adds one load a step.
- Balance at 32 lanes, makespan over ideal: Python 1.155 random / 1.146 B2; JSON 1.147 / 1.227;
  random bytes 1.21 / 1.116.
- Still open:
  - B2's balance on real encoders' orders;
  - B3's extra operations as throughput (MA-2), to be measured by P55.
- Feasibility §4 later says "a bit per row" without citing this (K23).

**P43 A separate RUN fast path.** Status: **rejected** by analysis.
- RUN costs no less space than FOR-14 in fixed records. A format branch mispredicts on ≈1/5 of
  steps.
- Agenda III is answered: one common FOR-14 path.
- Class: IV (recorded, modelled): misprediction flushes other lanes' loads.

**P44 Alphabet-specialised recovery.** Status: **conditional**, deferred "until a general lookup
is measured". Class: none recorded (deferred, not failed).

Compact-Ψ side questions, all **open**, with no timed figures and no recorded condition beyond
"measure":

| ID | Idea | What is recorded |
|---|---|---|
| P45 | Fixed vs adaptive partitions | Uniform chunks ~12% larger (Gov2 5.17 against 4.65 GB, a reading of the paper). "Checkable offline". |
| P46 | Per-group encoding by hardware cost | C_total = construction + navigation + cache pressure, "not a directly measurable additive equation yet". Per-block design choice deferred (needs per-design models). |
| P47 | PDEP-free select; in-line extraction | PDEP is slow on Zen 1/2 and fast on Zen 3+ and Intel. One line a lookup: answered. Compiler output and independent low/high loads: open. A precondition for any EF arm. |
| P48 | Lines occupied, not traffic | `u` makes 26% more reads past L2 a byte than `p` at 4 blocks (1.765 against 1.399). Line crossings explain ≤ ~5 points. Unexplained. |
| P49 | Smallest ≠ fastest? | Related: `u` (2.7 MB) moved the knee but lost per step. |
| P50 | Per-segment variation bound; hybrid fixed-address groups | Theory; "the paper does not supply those assumptions". |
| P51 | Exception cost distribution, lane bunching, A₂ | A₂ = \|S ∩ Ψ⁻¹(S)\| defined, not computed; "kept for after the release". |
| P53 | Several ψ cycles | An open check for construction arguments; handled in the walk (marker audit). |
| P54 | Adaptive representation by concurrency | A "possible consequence", not proposed. |

**P52 Discontinuity theory.** Status: **validated** (proved and checked; census, VM).
- r − σ′ ≤ d ≤ r − 1. d = r − 1 on 229 of 233 full blocks.
- Run-derived K₃₂ equals the census on every block (1,481,882).
- Agenda I is answered: ψ's translation runs are exactly L's runs.

**P55 Dependent-successor throughput benchmark, 1–32 lanes.** Status: **conditional** ("Only if
these results support compact descriptors"; "stays unscheduled until a compact layout is chosen
for its own reasons"). Class: none recorded.

**P56 libsais cyclic inverse as reference R.** Status: **validated** (as a reference).
- stage (build + walk), VM, 1 thread, best of 5, 431 blocks byte-identical:
  - wikidata mibz2 18.21 against libsais **12.10 c/B (−34%)**;
  - text 17.02 against 18.36; records 17.59 against 29.10;
  - after #32: libsais 10.86 against mibz2 12.92.
- It led to #32, #34 and #37. "libsais cannot decode bzip2's blocks as it is" was stale (K1).

**P57 libsais's lead is a memory-pressure effect.** Status: **validated**.
- T-vector phase, i9 (E2): sNx1/pN 1.065, 1.135, 1.264, 1.343 at 4, 6, 8, 10 workers.
- T-vector phase, i9 (knee run): s/p 0.989 at 1 block, 1.062 at 4, 1.251 at 8, 1.320 at 10.
  Falloff at 10: p 0.383, s 0.476. "The knee moves: no", for either.
- counters, i9: libsais 0.86–0.89 reads past L2 a byte against ours 1.40–1.55.

**P58–P64 libsais mechanism hypotheses.** All **open**. None has been run.
- No CI runner has a PMU (P146), so each EPYC test needs another EPYC host.
- Context, kernel, one core: libsais is 1.13× faster on the i9, 1.44× (1.43–1.46) slower on CI's
  EPYC, and ~even on the VM (0.99×, 0.82–1.20).

| ID | Path label | Hypothesis | Named test |
|---|---|---|---|
| P58 | LS-H1 (path H1) | Prefetch reach (24 lanes, one round ahead ≈ 500 instructions) covers an i9 L3 hit, not EPYC DRAM. Its "gap shrinking at 4 workers" remark contradicts E2 (K19). | fill-source counters on both; `UNBWT_CHAINS` = 32, 48, 64 |
| P59 | LS-H2 (path H2) | The instruction budget (~10.5/B) pays only at high clock. | `perf stat` instructions/cycles on both CPUs |
| P60 | LS-H3 (path H5 + H6) | The bigram scatter and two post-walk copies cost bandwidth and TLB reach. | `perf record` shares; page faults |
| P61 | LS-H4 (path H3) | Load-port pressure, with lane state through `slots[]`. | port counters; lanes-in-registers variant |
| P62 | LS-H5 (path H4) | 28 KB of `first_symbols` evicts prefetched P lines from L1D. | IBS / `perf mem`; `UNBWT_FIRST_SYMBOLS` 13/14 |
| P63 | LS-H6 (path H7) | TLB on EPYC/VM (closed for mibz2 on the i9, P33). | `dTLB-load-misses`; THP-backed A |
| P64 | LS-H7 (path H8) | The serial fallback explains the 13 ms s3x3 straggler. | count fallbacks; gdb breakpoint |

**P65 E3's libsais half (phase attribution).** Status: **open**. Whether libsais's `perf record`
phase split was ever done "is not stated". E1's reopening depends on it.

**P66 Shipped cooperative kernel vs libsais (#107).** Status: **validated**, at the inversion
boundary, on the i9.
- T-vector phase, i9, 20 rounds:
  - build + walk: l3x3/s3x3 **1.114 (1.027–1.208)**, median 1.081, at 9 cores; l4x2/s4x2 1.343
    (1.303–1.385) at 8; p4/s4x1 0.930;
  - with MTF timed: l3x3/s3x3 1.022 (0.986–1.059), a tie; l4x2/s4x2 1.218.
- libsais keeps the faster one-core kernel (s4x1/p4 1.075). See K32 for the one-block figure.

**P67 Intel IPP as a reference.** Status: **rejected** (not pursued: binaries only, needs Intel's
runtime). Class: none applies; never tried.

**P68 rapidgzip leads.** Status: **rejected** as transfers.
- The 344 → 388 MB/s gain came from seek windows mibz2 doesn't have.
- memcpy for repeats is P04. rpmalloc is out of scope (allocator rule). The block finder is
  Deflate's.
- Class: IV (judgement): format- or architecture-specific. The allocator part is excluded by rule.

### (c) Non-navigation work

**P70 Two Huffman symbols a lookup.** Status: **shipped (0.1.3, #24; perf/huffman-2sym)
[coord]**.
- full decoder, CI, 10 rounds: Wikidata −2.3% at 1 thread and −4.7% (−8.1, −1.1) at 4; text,
  records and random 3–5% faster.
- #69: the 4-thread margin is "within reach of the bias", but "#24's mechanism stands: the
  scanner's own CPU fell 0.16 → 0.10 s".
- Earlier bound (wpq#194): multi-symbol decoding "≤ 1 c/B of 1.9".

**P71 Smaller Huffman LUT.** Status: **rejected** (not pursued).
- The table is `LUT_BITS = 10`, 4 KB a table, ≤ 24 KB, L1-resident.
- The whole scanner is 5.4% of a decode (VM, stage), so the ceiling is under 5%.
- Cond.: the scanner becoming a larger share (as P72).
- Class: III (Amdahl-bounded, by arithmetic, not measured).

**P72 Parallel scanner from speculative block markers.** Status: **conditional** (#10 is open
and deferred; "only pays on many cores, or on poorly compressible data").
- Scanner share (stage/decoder, VM): 1.20 c/B, 5.4%; 1.6–1.9 of ~24–30 c/B (~7%); about 8% of CPU
  (K7).
- full decoder, CI: 4-thread Wikidata scanner ~0.16 s CPU against 1.5–1.7 s for the workers. Idle
  while the scanner reads: 6% at 2 threads, 11% at 4 (EPYC 7763).
- full decoder, i9, -1: the scanner takes 0.09 s of a 0.143 s wall; workers are 29–46% busy (the
  scanner is the binding stage there by inference).
- Binding point (modelled): ~9–11, ~13 or ~11 workers, depending on the source (K7).
- Cond.: "a workstation curve that bends before its physical core count, with rising idle while
  the scanner reads", confirmed by a paired experiment (`just bench-occupancy`). #116 now makes
  the trace possible under team plans.
- Class: IV (judgement): it binds only at high worker counts or on incompressible data.

**P73 MTF head in registers (#25).** Status: **rejected [coord]**.
- full decoder, CI, 10 rounds, on top of #24: Wikidata −1.6% (−2.3, −1.0) at 1 thread, +1.0%
  (−0.9, +2.8) at 4; random +11.1% / +8.2%; records +9.7% at 4; text +3.0% / +2.0%.
- Cond.: none recorded for x86-64. The aarch64 "portable twin" is unmeasured.
- Class: III (high-index symbols cost more).

**P74 MTF store-forwarding chain; front 32 B in a register (coop §15.4).** Status: **open** ("A
lead, recorded and not started").
- T-vector phase, i9: `mtf_cp2p`/`mtf_cp2` **1.0582 ±0.0093** (19/20). The fence moved a store
  off the loop-carried chain. The mechanism is hypothesised: "no counters were taken".
- MTF is 24% of a 3x3 worker's time and 14% of 4x1's.
- Cond.: queued as its own pre-registered production experiment, CI-measurable. §15.4 doesn't cite
  #25. A new proposal must differ materially from P73 and answer its random-data regression (K39).

**P75 Seward's sliding-block MTF.** Status: **conditional** ("No experiment now").
- census, VM: moves with index ≥ 16 / ≥ 32 / ≥ 64 are 15.2% / 1.52% / 0.23% on Wikidata and
  95.5% / 90.6% / 75.9% on records. "98.5% of moves already take the AVX2 path".
- Cond.: "in case a records-heavy workload appears".
- Class: III (judgement): Amdahl-bounded on Wikidata.

**P76 MTF count checkpoints for the team build.** Status: **shipped (0.2.0, #101)**. Two research
sub-rules failed.
- T-vector phase, i9, 20 rounds:
  - stage 1 (checkpoints free): q3x3/r3x3 **1.083 ±0.016**; build 1.223 (predicted 1.25–1.46);
  - stage 2: `mtf_cp` **0.9370 ±0.0025** of production's MTF, a 6.3% cost against a 2% budget
    (fail); n3x3/r3x3 1.033 ±0.008;
  - spill fix: `mtf_cp2p` **0.9743 ±0.0074** (≥ 0.98 failed); f3x3/r3x3 1.050 ±0.006; f3x3/n3x3
    1.003 (failed);
  - PR 0: `mtf_cp2p` 0.9776 ±0.0027; checkpoints cost 0.9% of the round. 30.2 KiB a block mean,
    55 KiB max.
- full decoder, CI (#101): MTF+RLE2 +0.2% (−0.2, +0.6). The function is uncalled on 2-core
  runners.
- #78's own pass rule ("3x3's build ≥ 1.25 × its current rate, and 3x3 ≥ 1.45 × p4") was never
  reported against (K50).

**P77 Batched team.** Status: **open** ("the next experiment, sized here"; second in the
feasibility review's order). Not built on any branch.
- T-vector phase, i9: helpers are idle 0.19 of a 3x3 team's core-time and 0.11 of a 4x2's; libsais
  s3x3 also 0.19. This is an upper bound, "not a throughput gain". The MTF share of a team's round
  is 22–28% (PR 0).

**P78 Fused front end / in-place T-vector.** Status: **open** (#8 open; step 3 never tried).
- The aim is memory. Expected: "peak RSS down by roughly 2.5 MB per slot and worker … Speed
  roughly neutral" (not measured).
- application-level StreamDecoder RSS, VM: 4 threads / 8 slots / 4 MB chunk 71.2 MB; 4/4/4 MB
  62.9 MB; 1/1/256 kB 23.4 MB; ~7 MB scratch per added worker.
- i9 contention proxy: MTF+RLE2 inflates 1.8× at 10 copies, against the build's 8.95×.
- Cond. (#8): "this build's best against the change's best, paired by round, at 2–10 workers",
  plus CI at 1 and 4 threads. The workstation adds "Measure … at 8–10 workers as well as 1–4". #101
  changed what fusion would save (MTF now emits checkpoints).

**P79 Run lanes in the build.** Status: **shipped (0.1.4, #32)**.
- stage (build), VM: Wikidata 5.70 → 2.55 c/B, with 81.9% of rows via run stores.
- full decoder, CI, 10 rounds: Wikidata −16.4% wall at 1 thread; −15.5% wall and −19.8% CPU at 4;
  pattern −30.5%.
- application: wpq#208 split ETA −7.4%.
- Modelled whole-dump decode at 4 threads: 1h11m → 1h00m.
- H3 had predicted "low-medium for Wikidata" (K4).

**P80 Build in four interleaved quarters.** Status: **rejected** (#23).
- full decoder, CI (EPYC 7763), 10 rounds: Wikidata +12.1% (+9.9, +14.4) at 1 thread, +10.9% at 4;
  pattern +18.7% / +13.7%.
- Cond.: none recorded (8 lanes not tried; never re-tested on top of #32).
- Class: III ("splitting the scatter into four write streams costs more than the forwarding it
  removes").

**P81 One-core build in one pass.** Status: **shipped (0.2.1, #114, 6a341f6)**. #93 was not
merged.
- stage, VM, 6 rounds (#114): build 1.117 (1.051–1.186); walk 1.003; whole 1.029 (0.993–1.066).
- stage, CI, 10 rounds (#114, f121097 against 8ed29b0): build **−6.0% (−8.5, −3.5)**, 2.31 →
  2.16 c/B.
- full decoder, CI (#114): Wikidata +0.8% (−1.2, +2.8) at 1 thread, +0.8% (−0.7, +2.3) at 4; CPU
  +1.1% (+0.2, +2.1) at 4. Bench verdict: "Worse (CPU cost), and no measure better".
- stage/full decoder, CI (#93, fc2e80e against 0e6ab25): build −2.6% (−2.9, −2.2), i.e. 1.027×,
  below the 1.03 floor; Wikidata wall +0.2% / 0.0% / 0.0%. Not merged (K21).
- full decoder, i9, 10 rounds, against v0.2.0: 1.048 (1.026–1.070) at 1 thread (CPU 0.954), 1.035
  (1.010–1.060) at 4 (CPU 0.971); 0.999 / 1.007 / 0.998 at 6 / 8 / 10 (team controls).
- Open sub-question: why CI and the i9 disagree. The hypothesis is that L loads compete with the
  SMT sibling on CI. "Not established: why."

**P82 Cooperative fused build.** Status: **shipped (0.2.0, #104)**.
- T-vector phase, i9, 20 rounds, MTF untimed:
  - x3x3/q3x3 **1.102 ±0.012**, build **1.313 ±0.033** (pass ≥ 1.15);
  - x4x2/q4x2 1.111, build 1.408; x4x1/q4x1 1.110, build 1.437;
  - x3x3 1.643 ±0.014 × p4.
- PR 0, MTF timed: fusion y against f 1.099 ±0.020 (4x2), 1.074 ±0.016 (3x3).
- The ratios are against q (the given-counts build, with its own overhead, P155), not against
  production or PR #73 (K20, K26). Predictions were 1.02–1.06; the results missed high.

**P83 First-column packing (A′).** Status: **superseded**: #114 removed the init pass while keeping
the byte from L.
- kernel, VM, 6 passes: build mibz2 0.888 against bz2ngrs 1.462 ns/B (1.67×, 1.59–1.75). Net of
  count, "about even".
- kernel, CI EPYC: build 0.691 against 1.317 (1.91×); net ~0.96 against 0.69.
- "Hypothesis 1 … loses its premise on both machines."
- Cond.: "only if the workstation's build row agrees with the VM's or favours bz2ngrs". #91's `e`
  arm (A′) was never run.
- Class: III (as bz2ngrs implements it), recorded.

**P84 Build compact Ψ directly from L in one pass.** Status: **open** (untested).
- No compact-format figures.
- Related figures: `u`'s 24-bit build "gave back" `v`'s 1.28–1.49× build gain; the `c` arm ran at
  0.15× the build.
- Baseline to beat: the fused build (x3x3 1.643 × p4).
- The coordinator's note: "writing fewer bytes need not be less work". In teams, records need one
  owner, and per-cursor buffers cost ~32 KB a thread.

**P85 Remove the segment-assembly copy.** Status: **rejected** (#48 stopped at step 1).
- stage, CI, decode order, 5 rounds (#49): EPYC 9V74 **1.4%** (0.14 of 9.94 c/B). EPYC 7763 1.5%
  alone, 1.9–2.0% with four copies. VM 3.9–4.3%. A/A +1.2% (−2.1, +4.7).
- i9 contention proxy: assembly 13.6× at 10 copies but 3% of time per byte. That bounds recovery
  at high concurrency to "a few percent".
- Cond.: "only if a runner or a change puts the assembly over 5% of a decode".
- Class: III (the ceiling is below the gate).

**P86 Non-temporal stores.** Status: **rejected** for the decode path.
- A code trace shows every buffer is re-read almost at once; the one write-once buffer is
  `decode_parallel`'s.
- Cond.: "only if a runner or a change puts the assembly over 5% of a decode", or a genuinely
  streaming boundary.
- Class: I (judgement): there is no write-once buffer on the path for the mechanism to act on.

**P87 Walk output in L's freed buffer.** Status: **open** (not run; "behind the cooperative work").
- No run. bz2ngrs's gather (its form of this): 10–15% faster than our assembly on the VM but
  0.026 ns/B (< 1% of the inverse BWT); 1.03× (1.02–1.04) slower on CI.
- Corrected mechanism: it saves ~0.9 MB of resident memory a block but "does not shrink the walk's
  hot working set" (K46).
- Pass bar: the occupancy curve at 6–10 workers ≥ 1.03 × today's, with DRAM write bytes per
  output byte.

**P88 Randomised blocks.** Status: **open** (completeness, not speed; pbz2 supports them in ~30
lines).

**P89 ARM / aarch64.** Status: **open**.
- Correctness only: qemu-user, 32,585 fuzz cases, 0 failures. No ARM hardware timed; no aarch64
  bench runner.

### (d) Scheduling, MLP, concurrency, teams, the knee, early warning

**P90 Cooperative inversion (teams).** Status: **shipped (0.2.0, #101/#103–#106)**. It is the
default on Linux with ≥ 6 physical cores.
- T-vector phase, VM: "cannot decide the question" (2x2 0.53 of p4; no capacity pressure).
- T-vector phase, i9, pinned:
  - discovery, 19 arms × 38 rounds: 4x2 1.226 (1.214–1.238) × p4; 3x3 1.354; s3x3 1.629; p8
    0.469;
  - confirmation, 20 rounds: 4x2 **1.230** (1.217–1.243); 3x3 **1.323** (1.312–1.335);
  - PR 0 (MTF timed): y4x2 1.439 ±0.020; y3x3 1.375 ±0.022.
- full decoder, i9, unpinned, 20 rounds (34c6ef7): auto at 20 CPUs against 4 workers **1.257
  (1.197–1.320)**; RSS 1.015; CPU 1.214; 4 physical cores 0.997 (0.980–1.014). The release
  candidate (ce7cdc6, 10 rounds) gave 1.223 (1.163–1.286).
- full decoder, CI, tag to tag: no change (2 cores).
- The model predicted ≈1.23× for the decoder; the measurement is consistent.

**P91 Team shape: 2 vs 3.** Status: **shipped (teams of 2, 0.2.0)**.
- PR 0 (research `y` kernel): 4x2 1.439 against 3x3 1.375.
- Rematch (library `l` kernel): l3x3/l4x2 1.158 in pass 1, 1.014 in pass 2.
- full decoder: 3 × 3 at 10 threads tied the policy (1.008, 0.994–1.023). See K34.

**P92 Policy at 6 threads.** Status: **open** ("the policy-tuning question, after the release").
- full decoder, i9: 6 threads 1,071 against 20 threads 1,050 MB/s, "within noise". 6 threads
  1.217 (1.178–1.258) at 0.83× RSS.

**P93 The L3 budget is per L3 domain (multi-CCD AMD).** Status: **open**. Untested; whether
`Plan::for_threads` reads per-domain L3 was not checked.

**P94 No two team threads on SMT siblings.** Status: **shipped (0.2.0, as a design rule)**. Not
separately A/B'd for teams. Basis: siblings cost ~1.43× CPU a block (P109).

**P95 Cooperative kernel at production's per-core speed.** Status: **superseded**. The release was
gated on the full decoder instead. The room walk part shipped in #104 as `coop::walk_room`.
- T-vector phase, i9:
  - 4x1 0.865 × p4; build 0.98 against 0.83 ns/B (+18%); walk 1.71 against 1.48 (+16%);
  - room-walk arms, 20 rounds: r4x1/4x1 **1.060 ±0.011**; r3x3/3x3 **1.027 ±0.009** (predicted
    1.04–1.14, failed);
  - walk instructions 22.2 → 16.9 and 22.4 → 17.1; r4x1 0.923 × p4.
- The gate 4x1 ≥ 0.97 × p4 was not met. The remaining gap "is all the build: 9.28 instructions a
  byte against production's 6.65".
- Cond.: none recorded.
- Class: III at k = 1 (recorded).

**P96 Residency vs overlap.** Status: **validated** (T-vector phase, i9; 588 runs).
- Hit share (raw, prefetch-inclusive): p4 0.945, p8 0.657, 4x2 0.949, 3x3 0.967. These are later
  labelled "unreconciled" (P151).
- Reads in flight: p4 28.1, 4x2 38.8, 3x3 41.9. Wait per read: p4 67, p8 366, 4x2 64 cycles.
- DRAM read B/B: p4 7.7, p8 34.4, 3x3 5.1.
- Little's law matches within 0–11%.
- Reading: 4x2 wins by overlap; 3x3 by both overlap and residency.

**P97 Teams are instruction-bound.** Status: **superseded** for the shipped kernel.
- On the old kernel: fill-buffer-full 0.07–0.08/cycle against p4's 0.24.
- On the shipped kernel: l3x3 0.194; "neither label is established".
- Class: none applies; the kernel changed.

**P98 Cross-core line handoffs.** Status: **rejected** (on the load side).
- T-vector phase, i9: HITM 0.33 (4x2), 0.84 (3x3), 2.98 (s3x3) per KB, against 0.005 (p4); "under
  1%" of cycles.
- Store-side RFOs were not counted.
- Cond.: `perf c2c` if coordination is high.
- Class: III (negligible).

**P99 Pool wake-up latency.** Status: **open** (test deferred: helpers that never park, then
`cpupower idle-set -D`).
- One block alone: helper start 117–337 µs, against 7–11 µs in the grid. A single-block build with
  k = 2–4 runs at 1.57–2.61 ns/B against 0.96.

**P100 L3-capacity cap rule (k ≈ 0.7 × L3).** Status: **shipped (0.2.0, as the team cap)**.
- full decoder, i9: the -9 and -6 knees both sit at an aggregate 14.4 MB.
- The registered constant (6.5 MB) "was wrong by about 2.2×". The rule was fitted on one machine.
- Cond.: "k should be checked on a second machine before it's trusted as a constant".

**P101 mibz2 worker cap below 6 cores; autodetect.** Status: **open** (#65 open; not updated since
v0.2.0).
- Below 6 cores the default plans one worker a thread with no cap. Autodetect was never built.
- CI 9V45: 2 threads on separate cores 826 MB/s, against 808 at 4 on siblings (2.11× CPU a byte).

**P102 Level-aware cap formula.** Status: **rejected**.
- full decoder, i9, 20 rounds: -3 1.092 (1.058–1.128), met; -6 1.006 (0.990–1.021), not met.
- Class: III.

**P103 Teams on every physical core at level ≤ 3.** Status: **rejected** (shipping; #115 closed
unmerged). The research result is validated.
- full decoder, i9, held-out sweep: -1 **1.150 (1.138–1.162)**, -2 **1.216 (1.193–1.241)**, -3
  1.075 (1.043–1.108). -4 1.085 met; -5 1.049 not met.
- #115 rev 1: -3→-9 0.986 (0.974–0.997); -6 0.983 (0.956–1.009), not met.
- #115 rev 2: -3→-9 **0.945 (0.930–0.959)**; -6 pooled 0.984 (0.969–0.999).
- Cond.: "A design that re-plans, or spawns helpers only when they can run, could remove it; it is
  not designed here." Widening to level ≤ 4 needs a held-out test on -7/-8. #115's trace re-run is
  "Not repaired or re-run" (the user's decision).
- Class: IV (recorded): mixed-level concatenations and an unexplained -6 regression.

**P104 One core a block at -2/-3, over the RSS budget.** Status: **conditional**, pending the user
("the trade-off is put to you explicitly").
- full decoder, i9:
  - -3: 6 workers 1.148 at 0.79× CPU and 1.26× RSS (52 against 41 MB);
  - -2: 6 workers 1.355 (1.21× RSS), 8 workers 1.362 (1.40×).
- Class: none recorded (a trade-off, not a failure).

**P105 The default at 6 threads.** Status: **open** ("Recorded for a later look").
- At levels 1–5 it runs at 0.76–0.83× its own 20-thread speed. At -1/-2, 6 of 10 cores sit idle
  below `MIN_COOP`.

**P106 Phase-aware gating.** Status: **rejected** ("dropped as a remedy beyond the worker cap").
- full decoder, i9: gate6 1.13 (1.09–1.17) at 8 workers and 1.21 (1.15–1.27) at 10, against
  ungated. Its best against the build's best: 1.002 (0.960–1.046); best gated 782 MB/s = 0.77× of
  1,019.
- Cond. (#65): kept as a second-ranked candidate "if the plain cap doesn't win end to end".
- Class: III.

**P107 The admission window starves workers.** Status: **rejected** (demoted).
- full decoder, CI 9V74, 4 threads, trace: slot waits are 7–8% of worker time, "an upper bound on
  what a wider window could recover". The 8/16/32-slot counterfactual was never run.
- Class: III (bounded).

**P108 Run-queue delay.** Status: **rejected** ("The cost is on a CPU, not waiting for one").
- CI: run-queue wait ~6% of a block's time; steal 0%.
- Class: III.

**P109 CI's four-thread ceiling is SMT.** Status: **validated** (two EPYC models, plus the i9 at 2
threads).
- full decoder, 2 threads pinned, 12 paired rounds: spread over packed **+40.5% (+37.5, +43.6)**
  on 9V74 and +39.6% (+37.5, +41.6) on 7763; i9 +39.0% (+31.2, +47.2).
- 4 threads against packed 2: 1.62× (1.59, 1.65), not 2×. Split: SMT 0.72 (61% of the shortfall),
  full occupancy 0.81 (39%).
- Falsified: that "SMT sharing alone explains the four-thread ceiling". "No decoder change
  follows."

**P110 The full-occupancy remainder (0.81).** Status: **open**.
- full decoder, CI 7763: CPU per block +14% with both cores busy; idle while the scanner reads 11%
  against 6%.
- Discriminator: idle rising while the scanner reads points to P72; CPU per block rising with
  little idle points to P11's contention test. The 9V45's 1.50× is "Not concluded".

**P111 Shared L3/bandwidth on CI's four copies.** Status: **open** (no counters on CI). The i9
mechanism is P112.
- stage (walk), CI: each copy's walk doubles with four copies (5.0 → 10.4 c/B).
- full decoder, VM bake-off: CPU 1.3–1.6× from 1 to 4 threads for pbz2, mibz2 and lbzip2.
- "Four processes are not four threads."

**P112 The many-core knee: L3 capacity, then queueing past the LLC.** Status: **validated**, on
the i9 only ("The workstation investigation is complete").
- full decoder, i9, quiet:
  - 6 → 8 → 10 threads: 884–907 → 706–708 → 634–653 MB/s;
  - IPC 1.06 → 0.50; DRAM reads flat at 18.6–19.2 GiB/s from 8;
  - read bytes per output byte 0.4 (1 thread) → 31–32 (10);
  - demand L3 hit share 0.76 → 0.38 (4 → 10);
  - occupancy per L3-missing demand read 396 → 858 cycles (81 → 175 ns);
  - page walks 1–2%.
- Default at 20 threads: 43% of the peak.
- Contention proxy (N pinned stage copies, 10 copies): build 8.95×, walk 9.85×, MTF+RLE2 1.80×,
  scanner 1.45×. Build + walk carry 84–93% of the added time.
- L3-miss stalls account for ~30% of the added cycles; "The rest isn't attributed".

**P113 Table bytes (block size) move the knee.** Status: **validated** (i9).
- full decoder, best counts: -9 4 (1,026 MB/s); -6 6 (1,327); -3 6–8 (1,654–1,729); -1 6–10
  (~1,464). -9: 4 beats 6 by 1.15× (1.12–1.18).
- T-vector phase, one core a block: -3 mibz2 peaks at 8 blocks, 3,279 MB/s, 2.2× the -9 peak.
  Hit share follows live table bytes within 0.031 (#112).
- "Table bytes are the dominant control, not the only one." Confounds: block count, tables,
  ratio and runs.

**P114 One knee shared across decoders.** Status: **rejected** (lbzip2 keeps scaling to 8).
- full decoder, i9, 6 rounds: mibz2 705 / 1,013 / 894 / 714 / 645 / 486 MB/s at 2 / 4 / 6 / 8 /
  10 / 20; pbz2 best 668 at 6; lbzip2 416 at 8–10.
- The rate-per-worker alternative stays **open**: "equally consistent with mibz2 touching more
  memory a byte".
- Class: III (falsified).

**P115 Past the knee, throughput ≈ 28 GB/s ÷ DRAM bytes per byte.** Status: **conditional** (a
working model).
- full decoder/kernel, i9: mibz2 and libsais both at 28–29 GB/s read+write from 6 blocks. The
  model predicts p10 572 (measured 567) and s10x1 747 (746).
- Probe: random line reads saturate at 26 GB/s; copy 34.4 GB/s.
- "Correlation, not proof." Also open: whether 26–27 GiB/s is the memory system's limit or a
  queueing limit before it.
- Cond.: a probe of this workload's own mix; an out-of-sample prediction.
- Class: none recorded.

**P116 DDR4 four-activate window as a second knee.** Status: **open** (modelled).
- Single-rank ceiling ≈ 381 M lines/s. The output ceiling of ≈530 MB/s compares with 490
  measured.
- The DIMMs are dual-rank, two per channel (K29). Named test: a random-lanes stressor or MLC
  `--loaded_latency`.

**P117 Request cost grows with cores at a high hit share.** Status: **open** ("not established").
- -3, i9: p10/p8 0.948 (0.937–0.959); wait 82 against 67 cycles at hit shares 0.940 against 0.929.
  l5x2/l4x2 0.746.
- The test, KNEE-ISO (iso-footprint lines), is "proposed, not registered, not run".

**P118 Write traffic past the knee.** Status: **open** (the attribution is "hypothesis, testable").
- full decoder, i9: IMC write bytes per output byte 0.3 (1 worker) → 11.5–12.8 (6–10). At -3, DRAM
  writes/B 1.16 → 2.17, and the build takes most of the loss at 10 blocks.

**P119 K blocks in lockstep per core.** Status: **rejected** ("A dead end").
- kernel, VM (`bz2k walk`): K = 2–8 gives 1.03–1.27×, with or without huge pages.
- Cond.: none recorded. *(inferred)* It would need a much smaller per-block table.
- Class: II (recorded): K × 3.6 MB outgrows the cache as fast as K hides latency.

**P120 AMAC / dynamic lane scheduling on the direct table.** Status: **rejected**. The two-tier
case is P121.
- T-vector phase, i9: 3x3 threads end within ~11 µs; the busiest walk is 1.36× the mean bytes
  (#95 gives 1.60×, K33); coordination is ~4% of a block's walk.
- Fill buffers are already full 0.19/cycle (rematch).
- AMAC's own Table 3 shows staging can regress a cache-resident table.
- Cond.:
  - "only if E3 finds the walk's end spread or start lag above 10% of a block";
  - or `l1d_pend_miss.*` well below the fill buffers, in which case the lever is lane count (see
    P23, already measured worse);
  - or a two-level format.
- Class: III/IV (recorded): bounded at ≤ ~4%, and AMAC's lever (irregular work per step) is
  absent.

**P121 AMAC-staged payloads for a two-tier format.** Status: **conditional** (on a two-tier
format existing and its stall profile being measured). The cited `2026-10-29-amac.md` is on no
ref (K30). Class: none recorded.

**P122 Batched segment dispatch.** Status: **conditional** (not run).
- Coordination ≈4% at 4x2/3x3; helper start 7–11 µs.
- Cond.: a winning arm with a large coordination share; "end spread or start lag above 10% of a
  block".
- Class: III (judgement): bounded by the measured 4%.

**P123 Edwards & Vishkin work-optimal parallel inversion.** Status: **rejected** ("depth is not
the problem here").
- Class: IV (judgement): a PRAM/XMT design, while the CPU decoder is already parallel across
  blocks.

**P124 GPU rank-and-scatter inversion (Weißenberger & Schmidt); scanned iMTF.** Status:
**conditional**. The later record (coop §5c) supersedes M27's "rejected" (K60).
- No runs. The code was read; the paper was not. ≈14 B/row (12.8 MB a block) against our 4.
- Cond.: "pursued only if a cooperative arm's join (`SegScratch::assemble`'s copy) shows up as a
  cost". Scanned iMTF: a time-to-first-byte or few-block workload.
- Class: II (recorded): a 3.5× footprint.

**P125 Early warning: CPU amplification 1 → 4 threads.** Status: **shipped** (instrument, #69,
bench in 0.1.6).
- full decoder, CI, 18 master records: mibz2 1.48–1.69× (7763), 1.71–1.83× (9V74), 1.75–2.31×
  (9V45), 1.48–1.68× (Xeon). Every record's mibz2/lbzip2 interval excludes 1.
- 9V74, 10 rounds: 2 threads spread 1.06× (1.05–1.07); 4 threads on siblings 1.78× (1.76–1.79).
- "Not claimed: … that a 1→4 amplification predicts the 10-core knee's position."

**P126 Contention fingerprint.** Status: **shipped** (instrument, #71). Its confound is open.
- CI 9V74: stream slowdown 1.69×, cache 1.07×, twin 1.10×. Retrospective stream: 1.30× → 1.47×
  from v0.1.1 to head.
- Confound: "More memory-level parallelism" and "less compute per byte" read alike.
- Test: "a victim with a fixed, known amount of compute added per block".

Early-warning leads, all **open** or **conditional**, none run:

| ID | Lead | Status | Recorded condition / test |
|---|---|---|---|
| P127 | Does a rise in CI's 2-spread amplification predict a lower knee on 10 cores? | open | two variants differing only in per-worker footprint, under #69 and `bench-scaling` |
| P128 | Is lbzip2's 1→4 amplification a stable baseline? | open | varies 1.11–1.42× by runner; "a control, not a constant" |
| P129 | Does the 2-core fingerprint order decoders as the knee does? | open | none recorded |
| P130 | A twin that loops inside one process | conditional | "if the decoder gets much faster" (twin CPU 0.93–0.98 against a 0.9 screen) |
| P131 | Cachegrind / drcachesim LL-miss proxy per PR | open | cachegrind serialises threads and ignores prefetch |
| P132 | USL α as a contention descriptor | open | as an extrapolator: three points fix three parameters (class I for extrapolation) |
| P133 | Dirigent-style per-block progress against a solo profile | open | inference, not pursued |

**P134 Kernel+MTF 1,707 MB/s (pinned) against decoder 1,256 MB/s (unpinned).** Status: **open**.
- Candidates: the scanner, RLE1 and CRC on the caller, placement, the reader.
- The precondition (the trace under team plans) is now met by #116 (P145).

### (e) Measurement and harness findings that changed hypotheses

**P140 Warm-repeat bias.** Status: **shipped** (bench 2.0.0, #45/#47).
- VM: the 1st 512-segment call 4.05 ns/B against the 4th 3.00.
- stage, CI 9V74: walk 4.02 → 4.97 c/B in decode order (+23.9%, +15.8 to +32.5); 9V45 +3.6%
  (build +8.7% instead).
- It invalidated the first sweep's NTA/24-lane rankings and the instruction-window evidence
  (P27). Lesson: "a stage table's shares are a runner's, not the decoder's".

**P141 A/B order bias (head always first).** Status: **shipped** (fix, bench 3.0.0, #69).
- CI A/A before the fix: 7 of 42 rows flagged; head 1.8% faster at 1 thread (−3.2, −0.4). After
  the fix: 0 of 42.
- Audit: every rejection stands; #12, #21 and #32 stand. #37's and #24's 4-thread margins are
  within the bias and were not re-measured.

**P142 Per-thread CPU accounting bug.** Status: **shipped** (#69). It lost up to 50 ms a worker:
0.10–0.16 s short on CI 4-thread runs, 0.31–0.35 s at 10 workers on the i9. Exact totals 4 → 10
workers: ×3.50 / ×2.65 / ×1.44 / ×1.10 at -9 / -6 / -3 / -1.

**P143 Sampler cost.** Status: **shipped** (fix). 1-thread runs +1.0% (+0.4, +1.7); 12 ms
single-thread runs +4.7–17%. The run-queue-lock mechanism is "[inference]".

**P144 Footprint and queue ledger (#70).** Status: **shipped (0.1.6, opt-in `trace`)**.
- VM, 4 threads: the hot set ≈ 13% of reserved (9.3 MB mean, 14.4 MB max, against ~111 MB
  reserved and 74 MB peak RSS).
- "The footprint lever is the T-vector per block × blocks in its phase, and nothing else."

**P145 Occupancy trace under team plans.** Status: **shipped** (#116, on master ae942bd, after the
v0.2.1 tag **[checked]**; "complete traces under a team plan; --teams K, checked in CI")
**[coord]**. The notes calling it open are stale (K57).

**P146 PMU availability.** Status: **validated** (negative for CI). The VM has none; CI shows "0
of 14" events (#87: 9V45 ×2, 9V74); the i9 has a PMU. Cond.: re-run #87 on a later runner image.

**P147 Busy-desktop plateau at ~470 MB/s.** Status: **validated**. A browser caused ~16 GiB/s of
idle DRAM traffic; quiet, the decoder ran at 884–907 MB/s against 471. The absolute levels from
busy runs were withdrawn; paired comparisons stand.

**P148 memcpy calibration (#57).** Status: **validated**. Counters calibrated at 1.56–1.58× reads
and 1.11× writes.

**P149 Slow runs after an idle gap.** Status: **open**, deliberately not pursued.
- i9, 6 threads, 12 rounds: plain + 0.15 s gap 5/12 slow; no gap 0/12; sudo perf + gap 0/12 but
  ~7% slower (1.072×). Heat falsified (≤ 53 °C, ≤ 72 W).
- Cond.: "unless it shows up in gap-free runs".

**P150 Native table size by count vs by time (#110/#111).** Status: **rejected**. #110 was
reverted: the mean is 1.946 MB by count, but the **time-weighted** size is 3.600 MB (Σn²/Σn; 54% of
431 blocks are full). Class: none of I–IV: it was a measurement error.

**P151 Raw offcore excess = L2 streamer prefetches (bit 7).** Status: **rejected** (it "fails twice
and is withdrawn").
- i9, 126 runs: without bit 7 the excess is 1.219–1.365 (fail); bit 7 alone 0.004–0.036/B (fail).
- What `offcore_requests.all_data_rd` leaves out stays open, so the raw hit shares in P96 are
  "unreconciled". Cond.: a per-bit split or Comet Lake PMU documentation.
- Class: III (the hypothesis was refuted).

**P152 Six pilot-harness negatives.** Status: **rejected** (each fixed). Results on the VM:
- a thread a block: 1.05×;
- `Vec`-header false sharing: 0.47×;
- 512 segments for k × 32 lanes: 1.08×;
- the output hash inside the timing: p4 647 → 931 MB/s once moved;
- fixed-rotation dealing;
- unpinned spinning pools: 0.65–1.15× → 1.67×.

Class: III/IV.

Open harness items:

| ID | Item | Status | Recorded figures / next step |
|---|---|---|---|
| P153 | Code-layout control for ~2% kernel A/Bs | open | `mtf_cp` 6.3% in one binary and 3.9% in another from the same source (K38); "the same arms in several builds with their code shifted" |
| P154 | The scatter compiles two ways | open | closure 0x6b2 against 0x35f bytes, ~1 instr/B apart; "a question for later" |
| P155 | The given-counts build's overhead | open | k = 1: q4x1 build 54.6 ms against p4 37.2 and x4x1 38.1; candidate `resize(n, 0)`; "Recorded, not started" |
| P156 | l3x3 7–8% faster than x3x3 | open (parked by the user) | g3x3/x3x3 0.999; l3x3/g3x3 1.082 (1.072–1.092): code, not placement. "pursued only if it points at an optimisation that transfers" |
| P157 | The -6 slowdown under #115's build | open | pooled 0.984 (0.969–0.999) against master A/A 1.006 (0.986–1.026); "unexplained" |
| P158 | Controlled version ladder (#52) | open | designed, feasibility shown, not run |
| P161 | Fair multicore comparison with bz2ngrs; `just bench-compete` on the i9 | open | "It is not built"; no workstation result |
| P163 | Literature tasks | open | 7 of the 10 triage queries unread; Reportable Shortcuts unlocated (K2); duplicates (navarro2020/2021, kempa2017 ×2); citation checks (LIT-23: arXiv v4 date, venue, Table 2, Fig. 3, PtrHash table; log₂ C(31,k−1) corrected to 12.13 / 21.33 / 28.16) |

**P159 libsais's real decode path.** Status: **validated** (source reading). For a 900,000-byte
block libsais takes the symbol-rows path; coop §5b.1 had described fastbits + bucket2.

**P160 bz2ngrs's gap is Wikidata's inverse BWT.** Status: **validated**.
- full decoder, CI 7763, 10 rounds: bz2ngrs 1.45× (1.45–1.46) slower on Wikidata.
- The kernels' 3.979 against 2.262 ns/B × 209.7 MB ≈ 0.36 s, matching the whole-decoder gap of
  ≈0.35 s.
- Peak RSS: bz2ngrs 0.45× mibz2 at 1 thread (12 against 26 MB). The 26 MB is not broken down.

**P162 External report 3's ranking.** Status: **superseded** by the E-matrix.
- Priority 1 rejected (P01); 2 superseded (P95); 3 shipped (P76, P81/P82); 4 measured (P96);
  5 rejected (P120); 6 per item (P39, P11, P87).
- Its "libsais … beats our cooperative research implementation by about 23%" holds on the i9 only.
  It was reversed by P66.
- Class: none applies; it was a ranking, not an experiment.

### (f) Application (wikidata-pq)

**P170 mibz2 as wikidata-pq's bz2 decoder.** Status: **shipped**. It is now wikidata-pq's default,
`DEFAULT_BZ2_DECODER = "native"` on master dd9e1031 **[checked]**. The notes' "lbzip2 stays the
default" is stale (K58).
- kernel, VM (prototype bz2k): chains 1 / 2 / 4 / 8 / 16 give 28.5 / 17.6 / 10.9 / 6.8 / 4.4 c/B.
- full decoder, VM: 24.3 c/B against lbzip2's 42.7 (1.76×).
- application, CI 7763, wpq bench 17.14.0, 3 paired rounds: split CPU **−20.6% ±4.7%**, wall
  −16.6% ±1.6%, peak RSS +3.0% ±1.2%; decode alone CPU −31.1% ±5.6%.
- Modelled release saving: −35 min, assuming the split's wall falls by its CPU change.

**P171 The seven conditions for the default.** Status: **superseded**: native became the default
**[checked]**. Which conditions were met is not recorded in the notes.
- The conditions: SMT gain (partly P109); one whole dump against lbzip2; the thread split with the
  writer's pool (P176); peak RSS; ARM (P89); `unsafe` review and coverage fuzz; the scanner at 16
  cores (P72).
- Also pending: pbz2 as a second backend; `decode_blocks` for the sweep reader wpq#185.
- Class: none applies.

**P172 wikidata-pq's thread cap of 4.** Status: **shipped** (wpq#212; `BZ2_THREADS_CAP = 4`
**[checked]**).
- application, CI, 2 threads against auto (4): 9V45 split CPU −11.9% ±1.1%, wall −3.6% ±4.3%;
  7763 CPU −6.9% ±1.5%, wall −0.2% ±2.0%.
- application, release-shaped CI (2 × 7763, 8 pairs): total wall −1.9% ±0.6%, split wall −4.4%
  ±0.5%.
- application, i9, 12 rounds, 72/72 parity: 4 threads against auto (20): split wall −20.3% ±0.9%,
  CPU −60.1%, RSS −5.9%.
- The rule `min(physical cores, 4)` is "still a hypothesis from two machine classes".

**P173 8 threads (4 teams of 2) in the release.** Status: **rejected** ("cap stays 4") **[coord]**.
- application, i9, release-shaped, 8 paired rounds, 8 threads against auto (4) (wpq PRs
  #215–#217):
  - whole-run wall **−2.0% ±0.6%**, CPU **+6.7% ±0.4%**;
  - split wall −4.2% ±0.5%, split CPU +16.9% ±0.3%;
  - parity 16/16.
- The extra CPU is "contention suspected, not demonstrated".
- Cond.: the default is not to be proposed again on this evidence. The maintainer's
  conditional hypothesis is still open: if the extra CPU is avoidable overhead rather than
  inherent latency-hiding cost, the wall gain might be had at less CPU. That is the
  portfolio's D1, an attribution by thread role (`2026-10-30-research-portfolio.md` §3.4).
- Class: IV (judgement): the trade-off depends on concurrency with the split's parse workers.
  The wall gain was weighed against the CPU cost.

**P174 Pin mibz2 0.2.x in wikidata-pq.** Status: **shipped** (mibz2 0.2.1, wpq#213, d09edd91)
**[coord]**. M28's "pinned 0.1.4 … has never run in the release" is stale (K56).

| ID | Item (wpq#194 / wpq#197) | Status | Figures (application unless noted; VM, single runs) | Recorded next step |
|---|---|---|---|---|
| P175 | No fresh pages per chunk | open | 804,748 minor faults, ~8.5 µs each; sys 6.8 of 45.2 CPU-s (15%) | "to be benched on CI"; modelled ≤ −30 min on the VM, −5 to −10 on a laptop |
| P176 | No thread oversubscription in the split | open | `RAYON_NUM_THREADS=1`: split CPU −13%; `=2`: wall −9% | "CI must measure"; with native decode the best split "probably moves" |
| P177 | Finalise's Parquet decode for AVX2 | open | decode −10% (1.47 → 1.29 CPU-s); encode 0%; bz2k 0% | needs run-time dispatch; modelled −8 min |
| P178 | Parquet encode with levels from the decoder | open | bounded ≤ 1.3×, not measured | upstream arrow-rs work |
| P179 | Schema-compiled JSON → Arrow | open (SIMD scan rejected, III: −5.6% instructions, 0% CPU) | callgrind 32.8 instr/B, IPC ≈ 4.3 | bounded ≤ 2.5–4×; "very high" effort and risk |
| P180 | Dead ends: asm/AVX-512 walk, zstd settings, bandwidth, Python line splitting | rejected (III) | zstd 8% of encode; ≤ 0.2 GB/s a core; 0.9 c/B | none |
| P181 | gz input instead of bz2 | conditional | modelled: the bz2 split ~2h34m beats gz at Wikimedia's rate | the user's decision |

---

## 3. Questions inventory

**The "~30 open questions" list does not exist in the repository.** None of the four auditors
found it (master, branches, issues/PRs). succinct-review §0 (research/psi-census, L27–33) says the
research chatbot's original log covered "… bzip2's small mode; 30 open questions", and "only its
conclusions are kept here". It should not be reconstructed.

The lists that do exist, merged below:
- #39 Q1–Q5 (= mlp §5 leads 1–5 = PR #43 §5);
- #6's 2;
- H1–H5 (= PR #31);
- E1–E11;
- literature-triage's 10 queries + 4 synthesis checkboxes (all still unticked);
- succinct §10's 4;
- conclusions-research §8's 8;
- phase-2-findings' 6;
- agenda-phase-2 I–V;
- feasibility Q1–Q5 (4 headings: "Q1–Q2", Q3, Q4, Q5);
- literature-study 00-overview's questions (18 by my count; M28 said "about 16");
- also included: the blog §8's 3 and early-warning's 7 "Open" items.

**Counts.** These lists hold 103 items. 93 remain after dropping the two verbatim duplicate
lists (PR #43 §5 and PR #31). I added one item (#84, the E10 refinement on #39), making 94, then
merged 10 more as duplicates (blog ×3, conclusions Q4, phase-2 Q5, overview ×4, triage synthesis
3, and feasibility Q1/Q2 as one heading). That leaves **84 distinct questions**: 31 answered,
6 conditional and 47 open. The breakdown is in the tally after the table.

Status key: A = answered (positive or negative), C = conditional, O = open.

| # | Question | Also asked as | St. | Entry |
|---|---|---|---|---|
| 1 | After the room walk, how much is interleaving and how much prefetch? | #39 Q1; lead 1 | A | P22 |
| 2 | Do Intel's DDP/AOP prefetchers overlap or interfere with the software prefetch? | #39 Q2; lead 2 | O | P28 |
| 3 | Does the room walk's lower instruction count let the window hold more lanes (AMAC)? | #39 Q3; lead 3 | O | P27 |
| 4 | Are 32 software lanes 32 misses in flight (Lemire)? | #39 Q4; lead 4; LIT-20 | O | P29 |
| 5 | Does the best hint or lane count differ between 1 and 4 threads? | #39 Q5; lead 5 | C | P25 (yes on CI's four-copy walk stage; no on the i9 full decoder) |
| 6 | Pair-table entry layout: 5- or 8-byte entries, with a start flag? | #6 | A | P01, P02 (8 B #29, 4 B #34; both lost) |
| 7 | Does the pair-table build pay on ≤ 900 kB blocks? | #6 | A | P01 (no: 3× the build) |
| 8 | H1: Ψ² built as libsais builds it, in the flagged walk | PR #31 | A | P01 |
| 9 | H2: Precopy with forward chains and per-chain records | PR #31 | A | P04 |
| 10 | H3: run lanes in the build | PR #31 | A | P79 (shipped) |
| 11 | H4: the fused front end / in-place T-vector | PR #31; triage synthesis 3 (memory/time trade-offs against pbz2's RSS) | O | P78 |
| 12 | H5: a parallel scanner from speculative markers | PR #31 | C | P72 |
| 13 | E1: does the pair walk pay on the i9? | — | A | P01 (neither pass nor fail; closed) |
| 14 | E2: is libsais's lead a pressure effect? | bz2ngrs §7 test | A | P57 |
| 15 | E3: why 3x3 wins, and where libsais's time goes | #76 | O | P96 (coop half answered); P65 (libsais half open) |
| 16 | E4: the coop kernel at production's per-core speed | #77 | A | P95 (not met; superseded) |
| 17 | E5: the team build without its count pass | #78 | A | P76 (shipped) |
| 18 | E6: libsais's walk parameters in the Ψ² port | — | C | P03 (on E1) |
| 19 | E7: the 20-bit ψ under pressure, best at 6–10 workers | #79 | O | P39 (registered i9 run never made) |
| 20 | E8: the move structure under four copies | #42 reopening | O | P11 (never run) |
| 21 | E9: the walk's output in L's freed buffer | #79 | O | P87 |
| 22 | E10: dynamic scheduling beyond the segment counter | #39 | A | P120 (no for the direct table); P121 (C for two-tier) |
| 23 | E11: the build in one pass | #81 | A | P81, P82 (shipped) |
| 24 | GNP 2017: can consecutive LF steps be predicted, batched or prefetched? | triage q1 | A | P12 (negative) |
| 25 | Gagie 2026, Reportable Shortcuts | triage q2 | O | P163 (not located; K2) |
| 26 | Baier 2018: can undetected redundancy be exploited when decoding? | triage q3 | A | P06 (a format change) |
| 27 | Baier & Dede 2019: tunnel planning, decoder-side? | triage q4 | A | P06 (format-side) |
| 28 | Petri 2024: scalable succinct indexing | triage q5 | O | P163 (unread) |
| 29 | Alanko 2020: prefix-sortable graphs | triage q6 | O | P163 |
| 30 | Navarro 2020/2021: repetitive collections I/II | triage q7 | O | P163 |
| 31 | KKP 2016: lazy LZ | triage q8 | O | P163 |
| 32 | Kempa 2017 ×2: same work or not? | triage q9 | O | P163 |
| 33 | Bannai 2019: bijective BWT inversion tricks | triage q10 | O | P163 |
| 34 | Compare run-aware, copying and shortcut techniques | triage synthesis 1 | A | literature-reading §3; P12, P04 |
| 35 | Can any method be built in one pass and pay back within the block? | triage synthesis 2 | A | P79 (yes), P01 (no) |
| 36 | Overlooked references on cache-aware inversion, pointer chasing, prefetch, SIMD | triage synthesis 4 | O | partly done in mlp-scheduling (Kohout, Ainsworth & Jones, AMAC, Lemire) |
| 37 | Minimum redundancy above nH₀ for class 1–2 access to ψ | succinct §10 Q1 | O | P16 |
| 38 | Can a run-record format (ψ = base + r) make the common step cheaper while halving the table? | succinct §10 Q2 | O | P40, P43 (fast path rejected by analysis; speed untimed) |
| 39 | How does alphabet-partitioned select map onto the cost classes? | succinct §10 Q3 | O | P14, P44 |
| 40 | Where do the corpus's tiny blocks come from; does any figure average over them? | succinct §10 Q4 | A | P150 (lbzip2-made, bimodal; time-weighted 3.6 MB) |
| 41 | Can a 24-bit direct link raise total throughput despite byte recovery? | conclusions Q1; phase-2 Q6 (in part) | A | P37 (no, as built); P38 open |
| 42 | What fraction of blocks fit FOR-14 / EF-12; how does it vary with alphabet and block size? | conclusions Q2 | A | P40 (corpus measured; variation in #58) |
| 43 | Can a 64-byte fixed record be about as cheap as a `u32` load? | conclusions Q3; blog §8 Q1 | O | P40, P55 (untimed) |
| 44 | Can the next pointer come from register arithmetic, with no second dependent load? | agenda IV; conclusions Q4 | O | P40, P13 (modelled only) |
| 45 | A FOR + EF + exception hybrid, or one scheme? | conclusions Q5 | O | P46 |
| 46 | Can a direct-indexed select exploit F's contiguous intervals? | conclusions Q6 | O | P37 (window design measured), P38 |
| 47 | Does a different physical organisation enable more MLP at more bits? | conclusions Q7 | O | P36, P29 |
| 48 | Where is the space–throughput frontier as residency, lanes and microarchitecture change? | conclusions Q8 | O | P36 |
| 49 | Is translation coverage stable per block? | phase-2 Q1 | A | P41, P52 (per-block 0.91–7.12% two-plateau; K₃₂ per block) |
| 50 | Can mixed-symbol groups be handled cheaply? | phase-2 Q2 | A | P37 (≤ 0.91% of groups, 0.15% of rows stepped; but recovery costs 16–28% of the walk) |
| 51 | How expensive are non-translation payload loads? | phase-2 Q3 | O | P13, P55 (no dependent-load microbenchmark) |
| 52 | What is the true physical footprint? | phase-2 Q4 | A | P13 (0.641 MB), P40 (1.818 MB) |
| 53 | Can Ψ groups be encoded in one streaming (parallel) pass? | phase-2 Q5; overview §4; opportunity 3; blog Q2 | O | P84 |
| 54 | Does any representation improve best-concurrency throughput? | phase-2 Q6; frontier success condition | O | P36 |
| 55 | Structure of the 77.6% translation population | agenda I | A | P52 (ψ's translation runs are L's runs) |
| 56 | The simplest 64-byte Ψ record | agenda II | A | P40 (sized, §14.2) |
| 57 | Can format-dependent branching be removed? | agenda III | A | P43 (one FOR-14 path, by analysis) |
| 58 | Behaviour under other input distributions; worst-case exception rate | agenda V; blog Q3; conclusions Q2 (variation) | O | P40 (7 inputs measured: 16.0–16.6 bits, exceptions ≤ 2.0%; no worst-case bound) |
| 59 | Does a 32-row symbol descriptor remove the hot-loop work? | feasibility Q1–Q2 | A | P37 (it is #112's design **[coord]**) |
| 60 | The formats already studied, sized | feasibility Q3 | A | P36 table |
| 61 | AMAC and the cooperative decoder's fit | feasibility Q4 | A | P120, P121 |
| 62 | Could compact ψ compete with the shipped decoder on Wikidata (i9)? | feasibility Q5 | C | P36 (bounded ≤ 1.25× / ≤ ~1.1–1.15×, modelled) |
| 63 | Does average BWT run length predict group occupancy? | overview §1 | O | P51 |
| 64 | Do runs of different symbols have different ordinal-boundary patterns? | overview §1 | O | P51 |
| 65 | Are breaks concentrated near alignments mod 32? | overview §1 | O | P51 |
| 66 | Does high repetitiveness cluster breaks or just give fewer? | overview §1; opportunity 1 | O | P51 |
| 67 | Can segment boundaries be chosen to make interval navigation cheaper? | overview §3 | O | P50 |
| 68 | Can interval splitting cut occasional long navigation paths? | overview §3; LIT-3 | O | P50 |
| 69 | Does the full-cycle amortised guarantee survive 512 segments over lanes? | overview §3; opportunity 2; LIT-2 | O | P50 |
| 70 | How many loads to extract one value within one line? | overview cacheline | A | P47 (one line a lookup) |
| 71 | Can low and high parts load independently? | overview cacheline | O | P47 |
| 72 | What does the compiler emit for in-register select, extraction and dispatch? | overview cacheline | O | P47 |
| 73 | CPU-specific alternatives to PDEP? | overview cacheline; LIT-16 | O | P47 (PDEP note only) |
| 74 | Do slow records bunch across lanes in one window? | overview §5; SR-CYCLES 9.3 | O | P51 |
| 75 | Which structural properties predict when extra dependent loads get expensive under 32 lanes? | overview opportunity 4 | O | P13, P55 |
| 76 | How much of Ψ can be rebuilt from compact information while traversals run near flat-table speed? | overview "fundamental" | O | P36 |
| 77 | Does a rise in CI's 2-spread amplification predict a lower 10-core knee? | early-warning | O | P127 |
| 78 | Is lbzip2's 1→4 amplification a stable baseline? | early-warning | O | P128 |
| 79 | Does #65's cap help the release, in the split? | early-warning; CPF-6.2 | A | P172, P173 (cap 4 kept; 8 threads rejected) |
| 80 | Does the 2-core fingerprint order decoders as the knee does? | early-warning | O | P129 |
| 81 | The fingerprint's confound: more MLP, or less compute per byte? | early-warning | O | P126 |
| 82 | lbzip2's and pbz2's 2-spread amplification after #69 (the control for mibz2's 1.06–1.14×) | early-warning | O | P125, P128 (not recorded as read) |
| 83 | A twin that loops inside one process | early-warning | C | P130 |
| 84 | E3's fill-buffer headroom for production at 4–10 workers (the E10 addition) | #39 E10 refinement | C | P23, P120 (lanes8/16 measured worse; counter verdict not posted) |

**Tally.**
- 31 answered: #1, 6–10, 13, 14, 16, 17, 22–24, 26, 27, 34, 35, 40–42, 49, 50, 52, 55–57, 59–61,
  70, 79.
- 6 conditional: #5, 12, 18, 62, 83, 84.
- 47 open: the rest.

Where an item is "answered" only in part (#41, 42, 50), I mark it answered because its stated
question has a measured answer. The remainder is carried by another row. Not counted as
questions: bz2-decoder.md's seven "before it could be the default" conditions (P171);
literature-triage's directions A–D and bwt-inversion-literature's three hypotheses (all in the
register as P12, P36, P34, P06, P07).

---

## 4. Contradictions

Merged from M27 C1–C16, M28 1–16, BR C1–C20 and IS 1–16: 68 points.
- Three non-contradictions are dropped: IS 14 (#12's unverified premise), IS 15 (the
  gap-triggered runs, now open item P149) and IS 16 (models against measurements).
- 13 overlaps are merged, leaving 52.
- 11 are added (K53–K63): status disagreements between the notes, and the coordinator's
  corrections.

That makes **63 points**.

Types:
- **real**: the claims conflict on one boundary and machine;
- **boundary**: different boundary, machine, run or kernel;
- **label**: a naming, unit or status error;
- **stale**: a later record supersedes an earlier one.

| K | Claims (sources, boundary) | Type | What would resolve it |
|---|---|---|---|
| K1 | literature-reading: libsais "cannot decode bzip2's blocks as it is". research-synthesis: `libsais_unbwt_cyclic` takes a bzip2 last column and `orig_ptr`, run byte-identical on 431 blocks. | stale | Resolved by the later measurement: literature-reading read only the suffix path. |
| K2 | bwt-inversion-literature lists "Reportable Shortcuts" (arXiv:2609.06596). literature-reading: "did not turn up under that title". | real (unresolved) | Look up the arXiv id. |
| K3 | compute-frontier: huge pages "no change" (wpq#194, VM kernel). mlp: huge pages open for the user and #40. Bloom: 36.1 → 28.2 c/B. | stale | #40 measured page walks at 1–2.5% (i9, full decoder) and closed it. Bloom's figure is another table size. |
| K4 | H3 predicted "low-medium" gains (mean run 1–19 bytes). Measured: 81.9% of rows via run stores; 71.35% of rows in runs ≥ 256. | real (prediction wrong) | None needed. The two measures differ (per-run mean against row-weighted share). |
| K5 | Peak RSS at 4 threads: 71 MB (0.1.3; = #8's application-level StreamDecoder 71.2 MB), 98/99 MB (bake-off), 113 MB (wpq#197), 74 MB (EW-5 ledger). | boundary | Different versions, harnesses and boundaries. State the version and boundary with each. |
| K6 | coop-decoder report RSS 72 / 255 / 61 / 74 "MB" against CHANGELOG 76 / 267 / 64 / 77 MB for the same arms. | label | The ratios ≈1.048 match MiB → MB. Check the report's unit. |
| K7 | Scanner share: 1.20 c/B 5.4%; ~3 c/B (+1.5 Huffman); 1.6–1.9 c/B ~7%; ~8% of CPU. Binding point ~9–11, ~13 or ~11 workers. | boundary | Different versions and instruments, and the binding points are models. Measure it on one instrument at one version, or paired on the i9 (P72). |
| K8 | wpq#194 (VM, a quiet day): "No shared resource … binds at 4 cores". Bake-off (VM, later day): 1.3–1.6× CPU. CI: four copies double the walk; +14% CPU per block with both cores busy. | boundary | Different machines, days and protocols. On the i9 it is P112. On CI it needs counters CI lacks. |
| K9 | wpq#194: evenly spaced starts cluster and stall. pbz2 uses evenly spaced rows and ties mibz2. | real (untested) | The lanes × starts × spacing sweep (P30). |
| K10 | The "local" VM: 2.1 GHz, L2 2 MiB, L3 260 MiB (compute-frontier, coop pilot); 2.8 GHz, L2 1 MiB, L3 33 MB (bake-off, move-structure); "2 MiB" (bz2-decoder). | stale (re-hosted) | Treat VM figures from different documents as different hardware, at different nominal clocks. |
| K11 | #37: −10.3% / −9.1% (9V74, db3593d) against −5.8% / −2.7% (Xeon, 6c645ef). CHANGELOG 0.1.5: −9.5% / −7.9% (tag to tag). | boundary | Heads, bench versions and runners differ. A same-head A/B on both vendors would settle it. |
| K12 | Decode-order walk penalty: +23.9% (9V74), +3.6% (9V45, where the build moved +8.7%), +40–50% (VM). | boundary | "A stage table's shares are a runner's." |
| K13 | bwt-inversion-literature and bz2-decoder say super-alphabet and Precopy are "not yet tried"; literature-reading records #29 and #30 as tried. | stale | Timeline only. |
| K14 | Prefetch saving: 34% (warm CI sweep, withdrawn), 26–28% (#21 VM, before the room walk), 13% alone / 15–22% with four copies (decode-order CI), 12% (i9 full decoder). | boundary | Runner, state and boundary all changed; not attributable to any one. |
| K15 | Local split CPU fell −30 to −37% against wpq#194's model of ~−23%; CI measured −20.5%. | real (unexplained) | A local paired re-run, or accept CI as the authority. |
| K16 | libsais 12.10 c/B before #32 against 10.86 / 10.9 "after #32" (libsais itself didn't change). | boundary (run to run) | Rerun in one session. |
| K17 | literature-triage's 14 checkboxes are unticked, though literature-reading answered several. | stale | Tick or annotate them (questions 24–36). |
| K18 | bz2ngrs and expert-review: libsais is "1.20 × our 3x3" (research kernel); external report 3: "about 23%". #107: the shipped 3 × 3 is 1.114× libsais at 9 cores. | stale | The kernel changed (#77, #78, #81). The journals were never updated. |
| K19 | libsais-unbwt-path H1: "consistent with the i9 gap shrinking at 4 workers". E2: sNx1/pN rises 1.065 → 1.343 from 4 to 10. | real (H1's remark is wrong) | E2 is measured. Drop H1's supporting remark; the mechanism (P58) is untested. |
| K20 | E11's registered pass: "3x3's build ≥ 1.15 × PR #73's". Tested against q3x3 (the given-counts twin): 1.313. coop §16.1 claimed x4x1 against q4x1 checks #93; §16.4 withdrew it. | label (baseline) | Report the x3x3 build against PR #73's. |
| K21 | One-core one-pass build on CI: #93 1.027× (−2.6%, −2.9 to −2.2; 7763) against #114 −6.0% (−8.5 to −3.5). #93 was not merged ("no end-to-end figure moved"); #114 merged with the same end-to-end null and +1.1% CPU at 4 threads. Different pass rules (#81 ≥ 1.10× against #114's "faster, no count slower"). | real (beyond intervals) + label | Different heads and bases. An A/B of #93's head against #114's on one runner. The merge decision rests on the i9 figures (P81). |
| K22 | Compact pilot `c` build: "85 against 1,052 MB/s" (≈0.08×; expert-review, #79) against "0.15×" / "155 against 1,052" (feasibility, CTV). | label or arm (unresolved) | Read #91's results file: probably c1 against c4, or two runs. |
| K23 | Marker audit (#100): segment ends need 1 or 6 bits a group. Feasibility §4 (a day later): "A record format needs a bit per row … the EF line's 32-bit start mask is that bit". | real (the review missed the audit) | The audit's B2/B3 stands unless a record format can't carry them. Feasibility should cite #100. |
| K24 | Census: "77.4% of groups" against "77.4% of rows" against "77.6% of rows in run groups" against "about 77%". Straddling 0.28% (#99) against 0.29% (#99 body, succinct §9 v1). Line records 16.16 against 16.12 bits a row. | label | Groups against rows; census versions v1/v2/v3; 0.29% was "a rounding slip" (0.2815%). Quote the version with each figure. |
| K25 | "#8's fusion" (workstation, early-warning) against CHANGELOG 0.1.3's "#8 … two symbols a table lookup". | label (resolved) | #8 is the umbrella issue: step 1 Huffman (#24), step 2 MTF head (#25), step 3 fusion (P78). |
| K26 | The researcher's "#81 improved cooperative build by 31.3% and build+walk by 10.2%" (00-overview, seward-frontier) without its baseline. Corrected in literature-study and 10-synthesis: against q, not production. | label | seward-frontier-research still repeats the uncorrected framing. |
| K27 | The E10 addition proposes lanes8/lanes16 at 4–10 workers. workstation §6 had already measured lanes16 0.86× and lanes8 0.71× at 10 workers. | stale (not cited) | Cite WS-6. The remaining question is the counter verdict (#84). |
| K28 | E4 / #77 status: "superseded" (M28), "validated" (BR, room walk adopted), "open" (IS). The 4x1 ≥ 0.97 × p4 parity rule was never formally waived; integration proceeded on PR 0's rule. | label | #77 needs a closing verdict: not met (0.923), superseded by the decoder gate. |
| K29 | research-notes' tFAW ceiling assumes single rank ("rank count unknown"); workstation-scaling records dual-rank DIMMs, two per channel. | real (model input) | Redo the 530 / 490 MB/s check with dual rank, or run MLC. |
| K30 | `2026-10-28-cooperative-inversion.md`, `2026-10-29-compact-psi.md` and `2026-10-29-succinct-review.md` are cited on master but exist only on research branches. `2026-10-29-amac.md` is on no ref. `12-amac-research.md` is missing from the literature-study index. | stale (dangling) | Merge the journals to master, or link the branches. Locate or drop the AMAC citation. |
| K31 | "The faster reference is libsais" (bz2ngrs §7 heading) against its own body: i9 only, 1.44× slower on CI EPYC, ~even on the VM. | boundary | Machine-specific. Qualify the heading. |
| K32 | libsais's one-core lead on the i9: 1.13× (coop pilot block mode), against s1x1/p1 0.989 (0.974–1.004) at 1 block (knee grid), against s4x1/p4 1.075 at 4 blocks (rematch). | boundary | Different harness modes and block counts. "libsais keeps the faster one-core kernel" rests on the 4-block figure. |
| K33 | External report 3: "unequal per-thread work … a severe straggler". Expert review: imbalance ≈4% (busiest 1.36× mean). #95: busiest 1.60× (3x3), 1.34× (4x2). | boundary (runs and arms) | Different runs. The straggler is attributed to libsais's fallback by hypothesis (P64). |
| K34 | Team shape: research `y` kernel 4x2 1.439 > 3x3 1.375; library `l` kernel l3x3/l4x2 1.158 (pass 1), 1.014 (pass 2). | boundary (kernel) | Different kernels. The shipped policy uses teams of 2; 3 × 3 tied it at the full decoder (1.008). |
| K35 | Native table size: coop §7.1 "18 MB … 0.9 of the L3"; #39 "~2 MB"; #110 1.95 MB; #111 time-weighted 3.600 MB. | stale (resolved) | Time-weighted 3.6 MB (P150). |
| K36 | Coop §10.4: teams instruction-bound (fill-buffer-full 0.07–0.08). Rematch on the shipped kernel: 0.194, "neither label is established". | stale (kernel) | Holds only for the old two-pass kernel. |
| K37 | Elasticity of throughput to walk instructions: predicted 0.49; measured 0.32 (r4x1) and 0.16 (r3x3). #79's −2% to −4.5% forecast reuses the measured values. | real (prediction wrong) | The forecast is still a forecast. |
| K38 | `mtf_cp`'s cost: 6.3% (§14.4 binary) against 3.9% (§15 binary), same source. CHANGELOG: window checks "about 7–8% of the build" on the VM. | boundary (code layout) | A code-layout control (P153). |
| K39 | Coop §15.4 queues "the list's front 32 bytes in a register" as new; #25 (two AVX2 registers) was rejected on CI; §15.4 doesn't cite #25. Coordinator: register-held MTF (#25) rejected. | label/real | A proposal that states how it differs from #25 (the store-forwarding mechanism; the MTF share in teams) and answers random +11% (P74). |
| K40 | Succinct §16.2: the d ≤ r − 1 bound "is loose by an order of magnitude". §17.1/§18.5: that table measured K₃₂, not d; d = r − 1 on 229/233 blocks. | stale (corrected) | Resolved in §18.5. |
| K41 | The level-policy registration said the formula keeps 4 teams at levels 8–9; at level 8 it gives 4.59 → 5. | label (registration arithmetic) | Re-register if the formula is revived (P102 rejected anyway). |
| K42 | Two-tier sizing: phase-2 ~5.6 bits, second load on 22% (illustrative), against census v3 5.70 bits exact, 12.0–36.3% by block. | boundary (illustrative against exact) | Use census v3. |
| K43 | #115's -6 slowdown (pooled 0.984) against master A/A at -6 1.006 (0.986–1.026). The arms explanation lost by a 0.001 margin; no mechanism found in the code. | real (unexplained) | More rounds, or a build-layout control (P157). |
| K44 | Coop §16's fusion predictions (1.02–1.06) missed high (1.102). §17's y against p4 came in above prediction, with p4 slower in that run (1,206 against 1,282 MB/s). | real (predictions wrong) | Record p4's drift beside the ratios. |
| K45 | Pair walk sign: CI +40.6% / +24.3% (#34) against i9 0.691 at 1 thread, 1.107–1.117 at 8–10, 0.856 best against best. | boundary (machine and pressure) | Consistent with a pressure mechanism (P57). Not contradictory once machine and N are stated. |
| K46 | Output in L's buffer: research/bz2ngrs-compare "up to ~20% fewer live bytes a worker" against master: "does not shrink the walk's hot working set". | stale | The branch holds superseded text (P87). |
| K47 | A note described #42 as a Ψ² negative. #42 is the move-structure negative; it cites Ψ² only as prior evidence. | label **[coord]** | Resolved: Ψ² figures are #6, #29, #34 and E1 (P01, P02). |
| K48 | #39, #60 and #63 write "#10 and #42 stay closed"; #10 has been open (deferred) throughout. | label | Say "deferred" for #10. |
| K49 | #42's criterion: its body says local, one thread, ≥ 10% under build + room walk. #39 calls the four-copy form "its pre-registered test"; E8 restates it with four copies. | label (criterion drift) | The four-copy form is the reopening condition, not #42's original test. E8 was never run **[coord]**. |
| K50 | #78's registered pass ("3x3's build ≥ 1.25 × its current rate, and 3x3 ≥ 1.45 × p4") against what shipped (y3x3 1.375, y4x2 1.439, PR 0's lower-bound ≥ 1.20 rule; #92 failed two of four rules). #78 is still open. | label (gate replaced) | Close #78 against its own rule, or record that the rule was replaced. |
| K51 | L3 hit share at 4 workers: 0.76 (workstation, full decoder, demand loads) against p4 0.945 (E3, T-vector phase, raw prefetch-inclusive offcore). The raw shares were later labelled "unreconciled". | boundary (+ an unreconciled event) | Different boundaries and events. P151's per-bit split would settle the raw figure. |
| K52 | #65's recommendation (cap via wikidata-pq) predates v0.2.0's team plan; how an explicit thread count interacts with `Plan::for_threads` "is not recorded". | stale (partly resolved) | wpq PRs #215–#217 measured 8 threads (teams) against 4 on the i9; the cap stays 4 **[coord]**. The interaction is still not written in #65. |
| K53 | E1's status: "rejected for production" (M28, IS) against "conditional" (BR). | label | One record: closed for production, with a recorded reopening condition (P01). |
| K54 | Precopy: "rejected" (M27, IS) against coop §5c's later conditional reopening at 8–10 workers on the workstation (BR). | stale (a later condition) | Rejected, with that reopening condition (P04). The run is not recorded. |
| K55 | First-column packing: "superseded" (M28) against "conditional, `e` arm first step of the link-only milestone" (IS) against "loses its premise … stays a cheap kernels experiment" (BR). | label | Superseded by #114, with the `e` arm unrun (P83). |
| K56 | M28 (feasibility §6.1, EW-OPEN-3, commit 7e7cce4): wikidata-pq pins 0.1.4; 0.2.x "has never run in the release". wikidata-pq master pins 0.2.1 (d09edd91), and #215–#217 ran 8 threads. | stale **[coord]** | Resolved (P174, P173). |
| K57 | M28 (EW-5, CPF-6.3c): the occupancy trace "collects no complete round under a team plan", an open bug. #116 fixed it (master ae942bd, after v0.2.1). | stale **[coord]** | Resolved (P145). |
| K58 | bz2-decoder.md / M27 A1–A2: "lbzip2 stays the default". wikidata-pq master: `DEFAULT_BZ2_DECODER = "native"`. | stale **[checked]** | Resolved (P170). When it changed and which of the seven conditions were met is not in the notes. |
| K59 | W7 (M27): "raw write-pointer lane state has no recorded result". IS #4: #12 shipped raw per-lane write pointers. | stale or label | Read #12's diff to confirm. |
| K60 | Weißenberger & Schmidt: "rejected" as a transfer (M27, research-synthesis) against a "conditional reference" (BR, coop §5c, later). | stale | Conditional (P124). |
| K61 | E9: "open" (M28, IS) against "conditional" (BR). | label | Open, not run, with a recorded pass bar (P87). |
| K62 | `v`'s link-only build: 1.55× production's (#112 VM first look) against "1.28–1.49×" (as quoted in #114; compact-psi i9, 1 / 4 blocks). | boundary (VM against i9, runs) | State the machine with each. |
| K63 | Checkpoint memory: "30.2 a block (55 max)" (M28) against "30.2 KiB a block mean, 55 KiB max" (IS) against "1.27% extra memory". | label (units) | Use KiB. 1.27% is relative to the block. |

---

## 5. Open items and recorded next steps

Ordered roughly as the records order them. The feasibility review's order is: (1) the wikidata-pq
bump and the trace fix (both now done **[coord]**); (2) the batched team; (3) #130's `e`/`h` arms
pre-registered against `l4x2` and the complete decoder.

| Item | Entry | Recorded next step |
|---|---|---|
| Batched team | P77 | "the next experiment, sized here": overlap one block's MTF with another's inversion |
| `e` / `h` arms (#130) | P38, P40 | pre-register "against `l4x2` and the complete decoder, not against `p4`" |
| Cheaper F-byte recovery | P38 | one-load `win` u16 + flag, or 64/128-row windows; stage B "only if the lookup gets cheaper" |
| Kernel+MTF against decoder gap | P134 | attribute the gap now that #116 makes traces complete under team plans |
| E8: move structure at four copies | P11 | "move build + walk ≥ 10% under build + room walk per copy, four copies; otherwise it closes for good"; "measure it at 8–10 workers" |
| E9: output in L's buffer | P87 | occupancy curve at 6–10 workers ≥ 1.03× today's, with DRAM write bytes per output byte |
| E7: 20-bit ψ on the i9 | P39 | registered bar "best absolute at 6–10 workers > p4's best", never run; superseded in practice by P37 |
| E3's libsais half | P65 | libsais `perf record` phase shares; gates P01's reopening, with a new criterion fixed first |
| libsais mechanisms H1–H7 | P58–P64 | counters on an EPYC with a PMU; `UNBWT_CHAINS` 32/48/64; `UNBWT_FIRST_SYMBOLS` 13/14; fallback counting |
| L2 prefetch in the coop walk | P26 | change the prefetch-level const in the cooperative walk; A/B on the i9 |
| MTF store-forwarding lead | P74 | its own pre-registered production experiment, CI-measurable; must differ from #25 |
| Fused front end (#8 step 3) | P78 | best against best, paired, 2–10 workers on the i9, plus CI at 1 and 4 threads |
| Parallel scanner (#10) | P72 | `just bench-occupancy` on the workstation: rising idle while the scanner reads, then a paired A/B |
| Full-occupancy remainder | P110 | as P72's discriminator; also the 9V45's 1.50× |
| Policy at 6 threads | P92 | "the policy-tuning question, after the release" |
| Per-domain L3 | P93 | check that `Plan::for_threads` reads `shared_cpu_list` |
| One core a block at -2/-3 | P104 | the user's decision on the RSS trade-off |
| Default at 6 threads (-1 to -5) | P105 | "Recorded for a later look" |
| Level-aware plan | P103 | "re-plans, or spawns helpers only when they can run" (not designed); -7/-8 held-out test for level ≤ 4 |
| -6 slowdown | P157 | unexplained; no step recorded |
| Pool wake-up | P99 | helpers that never park, then `cpupower idle-set -D` |
| Request cost at a high hit share | P117 | KNEE-ISO iso-footprint lines (not registered) |
| 28 GB/s model | P115 | a probe of this workload's mix; an out-of-sample prediction |
| Write traffic attribution | P118 | test the attribution (it is marked testable) |
| DDR4 second knee | P116 | random-lanes stressor or Intel MLC `--loaded_latency`, with dual rank |
| One-pass build: CI against i9 | P81 | "Not established: why" (SMT-sibling competition is the hypothesis) |
| Precopy at 8–10 workers | P04 | run `perf/precopy-chains` on the workstation at 8–10 workers |
| Start spacing | P30 | lanes × starts × spacing sweep |
| Prefetch hint gate | P25 | vendor- or thread-count-gated setting, confirmed at 1 and 4 threads |
| Intel AOP/DDP; instruction window; lanes vs misses | P28, P27, P29 | need a PMU host or an Intel runner prefetch-on/off pair; "a new discriminating test before any experiment that adds instructions to the step" |
| CI PMU | P146 | re-run #87 on a later runner image |
| mibz2 worker cap below 6 cores | P101 | #65: "a release-shaped paired comparison on a many-core machine"; autodetect "a candidate beside it" |
| Two-plateau descriptor; successor benchmark | P41, P55 | only if a compact layout is chosen for its own reasons |
| Direct compact construction | P84 | untested; baseline the fused build |
| A₂ and structural statistics | P51, P52 | the coordinator's "narrow structural step" (r, d, K₃₂, coverage, A₂), kept for after the release |
| Two-tier format; AMAC staging | P13, P121 | measure the payload-load cost first (phase-2 Q3) |
| Compact Ψ on other machines | P36 | a machine with less L3 per core; a laptop-class measurement |
| Seward MTF | P75 | only for a records-heavy workload |
| ARM | P89 | time on ARM hardware; an aarch64 bench runner |
| Randomised blocks | P88 | completeness (~30 lines in pbz2) |
| Harness items | P153–P156, P158 | code-layout control; scatter codegen; given-counts overhead (`resize(n, 0)`); REMATCH-GAP parked; version ladder not run |
| Early-warning leads | P126–P133 | fingerprint confound test (a fixed-compute victim); a two-variant footprint test; the rest as listed |
| Gap-triggered slow runs | P149 | only if it appears in gap-free runs |
| bz2ngrs fair multicore; bench-compete | P161 | not built; no workstation result |
| Literature | P163 | locate Reportable Shortcuts; read queries 5–10; resolve duplicates; LIT-23 citation checks; tick triage checkboxes (K17) |
| wikidata-pq items | P175–P179, P171 | A3/A4 "to be benched on CI"; A5 run-time dispatch; pbz2 as a second backend; `decode_blocks` for wpq#185; ARM, whole-dump and fuzz conditions of A2 (outcome not recorded) |
| Closing records | K28, K50, #81, #93 | #77, #78 and #81 are open without closing verdicts; #93 is open, unmerged and superseded |

## Unresolved by this merge

- **K22**: the `c` arm's build, 85 or 155 MB/s. The notes don't say which arm or run each is.
- **K59**: whether #12 included raw per-lane write pointers. Two notes disagree; I didn't read #12's
  diff.
- **P170/P171**: when wikidata-pq made native its default, and which of the seven conditions were
  met. The notes predate it; I checked only the current default.
- **P173**: no recorded reopening condition for the release's thread count. Only the decision is
  in the coordinator's correction; I didn't read the wikidata-pq journal.
- **E4, #78, #81**: each has a pre-registered rule that was never evaluated as written. Their
  statuses here (superseded / shipped) follow what happened, not the rule.
