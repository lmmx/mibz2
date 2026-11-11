### (b) Transition cost

Ideas that make each step of the inverse BWT cheaper: lanes, prefetch, the room walk, table
layout, and the compact and succinct tables. 48 IDs: P20 to P34 (there is no P35), P36 to P68.
Read only: nothing was built or run for this section.

**Sources, abbreviated in the table.** Files under `docs/journal/` unless a path is given.

| short | file |
|---|---|
| reg | the 30 October register, `scratchpad/portfolio.md` |
| feas | `2026-10-30-compact-psi-feasibility.md` |
| cpsi | `origin/research/compact-psi:docs/journal/2026-10-29-compact-psi.md`, and its grid `2026-10-29-compact-psi/coop-20261029T215136Z.md` |
| ctv | `origin/research/compact-tvector:docs/journal/2026-10-28-compact-tvector.md` |
| for14 | `origin/research/for14-records:docs/journal/2026-11-01-for14-stage1.md` |
| ladder | `origin/research/walker-ladder:docs/journal/2026-11-01-walker-ladder-protocol.md` |
| blog-st | `docs/site/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md` (§6 to §9) |
| camp | `compact30-channel/CAMPAIGN.md` |
| charter | `2026-11-04-compact30-research-charter.md` (§5.6) |
| c30s | `2026-11-05-compact30-staged.md` |
| sweep8573 | `2026-11-05-compact30-staged/sweep-42036331632/` (Xeon 8573C, walk only) |
| bc, bc/nogo, bc/walk | `2026-11-05-binding-census.md` and its directory |
| wr, sw, so, dfw, tp, ts, cw | `2026-11-05-walk-round.md`, `-staged-walk.md`, `-staging-only.md`, `-deferred-flag-walk.md`, `-tail-passes.md`, `-tile-size.md`, `-coresident-walk.md` |
| pw, sc, corun | `2026-11-05-partitioned-walk.md`, `-segment-cost.md`, `-corun-placement.md` |
| isa | `2026-11-05-isa-probe.md` |
| knee | `2026-11-07-knee-mechanism.md` |
| lfa | `2026-11-07-left-field-archive-check.md` |
| bt, bdc, sn | `2026-11-10-backward-table.md`, `-by-data-class.md`, `-share-nothing.md` |
| bw, tm, ps, btr | `2026-11-10-bandwidth-titration.md`, `-two-modes.md`, `-parallel-scan.md`, `-borrowed-tricks.md` |
| ereg, eres | `scratchpad/reg2/design-e-registration.md`, `design-e-result.md` |
| ws | `2026-10-28-workstation-scaling.md` (§9) |
| lit05, lit09 | `2026-10-29-literature-study/05-partitioned-elias-fano-research.md`, `09-cacheline-ef-research.md` |

**Production's prices, for comparison** (ereg §4, counted): T-vector build 4.92 instructions a
row and room walk 10.41 on Wikidata, 15.3 together; 11.04 and 10.65 on text.

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price | cheapest falsifier now | bearing on share-nothing | sources | premise the verdict rests on | premise still true? |
|---|---|---|---|---|---|---|---|---|---|---|
| P20 | Flag each segment's start in bit 31 of its entry; raw per-lane write pointers. | Joined-walk bookkeeping falls: full decoder −10.7% at 1 thread and −16.6% at 4 (CI, Wikidata). | shipped | n.a. Later form: #168's once-an-offset flag test bought no time. #181 put the per-lane test back and gained −2.0% (−2.7, −1.2) at 1 thread on EPYC 9V74. | Bitmap walk before #4: 34.16 instructions a row at 12 lanes; room walk today 10.41. Flagged skeleton 5.68 c/B against room 3.87 on EPYC 9V74. | none needed | none. But the flag needs a spare bit a row, which a table under 4 bytes a row lacks (see P42). | reg; so; dfw; ereg §4; eres | "The ~10 c/B bookkeeping premise was inferred, not attributed by measurement." | yes for the T-vector. Moved for small tables: "E's line has no spare bit a row to flag them" (ereg §1). |
| P21 | 32 lanes, a software prefetch one round ahead, 512 segments. | Walk stage 11.51 c/B at 32 lanes with the prefetch against 16.39 without (VM). Full decoder 0.809 at 1 thread. | shipped | n.a. Re-swept on the staged kernel: 59 settings on EPYC 9V45 and 9V74, none faster beyond noise alone or at four copies. The same on Xeon 8573C. | Room walk 10.41 instructions a row (Wikidata), 10.65 (text). A segment costs about 457 instructions, steady. | none needed on CI. An i9 recipe only if a small table ships: no sweep exists on a table under 1.4 MB. | none: each independent worker keeps it. | reg; cw; sweep8573; ereg §4; sc | Tuned on level-9 blocks, 3.6 MB a table (inferred from the corpora used). | moved: the target table is 1.2 to 1.4 MB. Never swept there. |
| P22 | How much of the gain is prefetch and how much interleaving. | L1 prefetch saves 13% at 32 lanes with one copy and 15 to 22% with four (CI). pf-off 0.881 on the i9. | validated | n.a. Confirmed with counters on the i9: no prefetch to prefetch is −31.8% cycles a row at 32 lanes, and demand L1 misses fall from 0.978 to 0.004 a row. | The prefetch is 1 of the staged step's 8 instructions. Dropping it saves 5% of instructions and costs about 6% of wall at 1 thread (VM). | none needed | none | reg; ladder §8.3, §8.4; sweep8573; bc/walk; wr; c30s | "on one CI runner and on the i9" | yes, and wider. Xeon 8573C: no prefetch +9.2% alone, +19.8% to +23.9% at four copies. EPYC 9V45: +48% alone, +66% at four copies. |
| P23 | Re-tune the lane or segment count after the room walk. | Some lane or segment count beats 32 and 512 beyond noise. | rejected | advantage, on CI: none of 59 settings faster beyond noise on EPYC 9V45 or 9V74 (alone or four copies), nor on Xeon 8573C. i9: lanes16 0.955, lanes8 0.853; the ladder reads 6.76 against 6.90 cycles a row at 16 and 32 lanes. | Instructions +0.6% to +5% across 16 to 64 lanes (VM callgrind). | i9 recipe, only with a small table: lanes at ten plain workers on level-1 input. Never run. | none | reg; cw; wr; sweep8573; ladder §8.2 | "If the walk is memory-bound with misses in flight well below the fill buffers" (E10 addition). | moved: the i9 ladder counted `fb_full` at 0.377 events a cycle in the shipped walk and found "no plateau". On CI "no model is at its documented miss capacity at 1t" (bc/walk). |
| P24 | CI's 2-core runners selected the releases for a sharp knee. | v0.1.2's ratio of 8 to 4 threads on the i9 would be at least 0.85. | rejected | advantage: 0.78 (0.77 to 0.79). | n.a.: a claim about history, not a loop. | none needed | none | reg; knee | "The history steepened the normalised curve by raising the low-concurrency end." | yes. The knee is since tied to planned teams: 461 to 466 MB/s a table at 3 teams, about 320 at 4 (knee E1). |
| P25 | Prefetch hint T1, L2 or NTA in the production walk. | 16 lanes with the L2 hint beat the default at four copies by 3.8% to 8.9% (one CI run, the unstaged kernel). | rejected | advantage, on the staged kernel: room/16/L2 slower beyond noise on 7 of 8 CPUs (+10.6% to +53.7%) on EPYC 9V45 and 9V74. On Xeon 8573C at four copies 16/L2 is +33.9% to +38.8% and 32/L2 +6.8% to +10.0%. | derived: a hint change, no added instruction. | none needed on CI. The i9 team case is P26. | none | reg; cw; sweep8573 | "the best setting flips with contention" (Class IV). | no: "The old four-copy interaction was a property of kernel A"; staging removed it (cw). |
| P26 | L2 prefetch in the cooperative walk. | An L2-hinted team walk beats the L1 hint at three teams on the i9, by relieving full fill buffers (0.194 a cycle at l3x3). | open | n.a. Not run: "No i9 team run with L2 prefetch was found in the archive" (lfa). | derived: a hint change, no added instruction. | i9 recipe: one more arm of the pipelined default with `prefetcht1`, at 10 threads. | none: it tunes the cooperative walk that share-nothing would retire. | reg; lfa; bw | "the cheapest test the counters point at" (counters of 29 October, research kernel). | moved: the default is now three pipelined teams. The simulated ledger puts its cold traffic in sequential writes (19.6 M cold lines written against 3.7 M read), not table reads. L2 hints lose on every CI model (P25). |
| P27 | The room walk's gain read as an instruction-window effect (AMAC, runahead). | Added step instructions cost overlap even when the loads are prefetched, so they show as time at one thread. | open | n.a. The ladder's D3 "classifies nothing": the shipped walk issues 0.57 of 4 µops a cycle. | n.a. It prices a step instruction: 22 more cost 1.401 ns a row on the i9 (`h`); 37.8 more cost 13.0 c/B on EPYC 9V74 (E). | i9 recipe with port and occupancy counters, blocked while `perf_event_paranoid` is 4 (bw). The ladder's archived counters are already read. | cuts CPU a byte: it sets what a heavier step costs, which is the gate of every small table. | reg; ladder §8.1, §8.4; for14 §7a; eres; bw | "a new discriminating test before any experiment that adds instructions to the step" | no: `h`, compact30 and E each added instructions without it. The one test run decided nothing. |
| P28 | Intel's AOP and DDP hardware prefetchers against the software prefetch. | On Intel the hardware pointer prefetchers make the software prefetch redundant or harmful. | open | n.a. The toggle itself was never run. | n.a. | read of an archived bundle: the Xeon 8573C sweep already holds prefetch on and off. The prefetcher toggle needs bare metal. | none | reg; sweep8573; bc | "no Intel-runner prefetch-on/off comparison exists" | no: Xeon 8573C, 5 November: no prefetch +9.2% (+7.3, +11.1) alone and +19.8% to +23.9% at four copies. The software prefetch is not redundant there. |
| P29 | Software lanes against misses in flight. | The best lane count follows the core's fill buffers: 24 on Zen 3 and 4, 16 on Golden Cove, about 10 in AMAC's figure. | open | n.a. Fitted knees since: 22 lanes alone and 16 at four copies on EPYC 9V74; 32 and 22 on 9V45. On the i9 the speed is reached by 16 lanes with prefetch, by 8 without. | n.a. | i9 recipe: the occupancy the prefetches hold, which the ladder's D4 did not measure. Blocked while perf is closed. | none | reg; bc/walk; sc; ladder §8.3 | "arithmetic only" | moved: measured since. But "no model is at its documented miss capacity at 1t" (bc/walk), and the ladder calls the fill-buffer match "a coincidence of the model's number". |
| P30 | Evenly spaced against pseudo-random segment starts. | Start placement changes lane balance: evenly spaced JSON starts model a makespan of 1.30 against 1.147 random. | open | n.a. for 512 starts: never swept. For dense row-sorted starts: construction. The lean segment path costs 33 to 64 cycles a segment against a gate of 10. The miss saving stands in simulation: 0.926 to 0.756 T-vector misses a byte (SMT pair, 512 KB L2). | Lean walk: 10.9 instructions a byte plus 108 a segment; 25.37 a byte at one segment per 8 rows against master's 10.74. | counting probe: the trace simulator at 512 evenly spaced against 512 random starts (misses a byte, makespan). | none | reg; pw; sc | "evenly spaced starts clustered in the text and stalled the lanes" (VM, no figure). | moved: not re-measured, and the walk changed. The room walk now hands a free lane the next queued segment, so a stall costs less (inferred). |
| P31 | The shared-offset room walk. | Full decoder −10.3% at 1 thread (EPYC 9V74), −5.8% (Xeon 6973P-C). | shipped | n.a. Extended since: #157 (walk −22% instructions), #171 staging (4 threads −5% to −14%), #181 (1 thread −2.0% on EPYC 9V74). | 10.41 instructions a row (Wikidata), 10.65 (text). i9, 2 November: 17.0 instructions and 6.90 cycles a row. In the default team plan the walk is 17.8 instructions a byte. | none needed | cuts CPU a byte: it is the skeleton a small table must reuse. E in the flagged skeleton paid about 22 instructions a row of bookkeeping. | reg; bc/nogo; sw; so; ereg §4; ladder §8.3; sn §1 | "v0.1.3 → v0.1.5: +14–20% at 2–4 threads and nothing measurable at 6–10" (i9, independent workers). | yes for 3.6 MB tables. With 0.4 MB tables ten plain workers reach 2,563 MB/s on it (sn §3b). |
| P32 | The adjacent-line hardware prefetcher doubles the walk's traffic. | Turning it off (MSR 0x1A4 bit 1) would cut lines fetched a step. | rejected | n.a.: a judgement, never tested. "turning prefetchers off is machine configuration". | n.a. | i9 recipe with root, which is Louis's to run. Not worth asking for. | none | reg; bc/walk | "a machine setting, outside the library" | yes |
| P33 | TLB reach and huge pages for the table. | Page walks take over about 10% of cycles, and rising with workers. | rejected | advantage, on the i9 only: 1.1%, 1.1%, 1.0% and 2.2% of cycles at 4, 6, 8 and 10 workers. The ladder's `walk_active` (0.28 to 0.47) was no different from its control. | derived: an `madvise`, no added instruction. | census: read `/sys/kernel/mm/transparent_hugepage/enabled` in a CI job. A CI A/B after that needs Louis (allocator rule). | none. A 1.2 MB table spans a third of the 879 pages a 3.6 MB one does (inferred). | reg; bc; bc/nogo; bc/walk; ladder §8.1; tm; wr | "only if a different machine class or block size shows page-walk activity over about 10%" | moved: CI's 4-thread row puts two tables a core on a nested-paging guest with no counters ("0 to −3% at 4t … the sign cannot be shown without counters"). tm lists huge pages as one of five candidate causes of the i9's slow mode. |
| P34 | Choose the kernel a block: the plain walk for blocks with few runs. | Pattern blocks decode at the plain walk's 210 MB/s, not 93, without moving Wikidata. | rejected | construction (#27's selector): Wikidata ±0.3%; pattern +21.0% at 1 thread and −18.8% at 4. The advantage on pattern was not disproved. | unknown: the selector's cost is not in the record. | CI A/B: a selector on the run count (known before the build) on pattern and Wikidata at 1 and 4 threads. | shrinks the table: per-block admission is how a small table avoids records and random blocks, where E is 4.82 and 4.55 bytes a row. | reg; ereg §3; bdc §3; bt §4; btr §4; wr; tp | "a selector that wins on pattern at both thread counts without moving Wikidata" | moved: the pattern gap predates staging (pattern wall −25% locally with staging alone, wr). Not re-measured since. |
| P36 | Compact Ψ: push out the frontier of throughput against memory. | "a measured configuration with more throughput and less measured memory than the best 32-bit configuration, on the same machine and workload" | open | n.a. Its bound has moved. feas put the gain at no more than 1.25× the inversion and 1.1 to 1.15× the decoder. sn §3b measured 2,563 MB/s at ten plain workers on 0.4 MB tables against about 1.5 GB/s for the cooperative default. | The gate: a small table whose inversion takes g times production's time models 3.0 GB/s ÷ (0.3 + 0.7 g) at ten: 2.6 at g = 1.2, 1.5 at g = 2.5. Production's inversion is 15.3 instructions a row. | i9 recipe: plain workers on `lbzip2 -2` Wikidata at 2 to 10 (sn §4.2). | shrinks the table | reg; feas §5; knee H3; sn §2, §3b | "on the i9, four teams' tables already fit the L3" (Class IV). | no: "contested by E1" (knee H3). Per-table rate falls from 461–466 MB/s at 3 teams to about 320 at 4. |
| P37 | `u`: a 24-bit link-only table, the byte taken from a 32-row window of F. | The knee moves: u's best at least 1.05× p4's best. | rejected | construction: u1/o1 0.681 (0.678 to 0.684) against a gate of 0.85. The advantage held: u6/p4 1.100; u8 1,213 MB/s against p8 697; u10 805 against p10 565. | Step 31 instructions and 7 loads against production's 16 and 3 (harness copy). Derived: walk about 0.598 of production's rate, build about 1.005 (0.721 × 0.830; 1.279 × 0.786). | read of an archived bundle: `coop-20261029T215136Z.jsonl` holds u1 to u10 a round. Read each worker's rate against live bytes. | shrinks the table, to 2.7 MB only: ten are 27 MB, past the 14.4 MB knee. | reg; cpsi §5; for14 §5; knee E2 | Stage B's gate "u1/o1 ≥ 0.85", read at one block; target "`l4x2` … not `p4`". | moved: the one-thread gate is now g up to about 2.5 (sn §3b). u's g is 1/0.681 = 1.47 (inferred). It clears the step gate and fails on size. |
| P38 | Cheaper recovery of F's byte: one load, a larger window, or the byte in the link's own line. | Byte recovery at one load and a test costs less than `u`'s 16% of the walk at 4 blocks. | superseded | n.a. The byte-in-line form was built twice: `h` carries it from the record's header (4 of its 22 extra instructions); compact30's header holds it (3 loads a step against 2). The u16 window was never built. | `h`: +4 instructions a step for the carried byte. The packed token removes its load and store: `k` 31 and `j` 27 instructions, compile only. | none needed for a backward table: there the byte is the table's own L[i] (bt §1). | shrinks the table: a link-only entry needs it; walking backward removes the need. | reg; feas §2; for14 §5, §8a; c30s; bt §1 | "The floor is set by production": the byte comes from the same load as the link. | yes. E's line keeps that property: it holds the 32 bytes of L. |
| P39 | 20-bit dense ψ, and libbzip2's split ll16 + ll4. | E7's bar: "best absolute at 6–10 workers on the i9 beats p4's best". | rejected | construction, on a VM whose 32 MB L3 holds every table: `c` build 0.15× the control's (155 against 1,052 MB/s), walk 0.74×; `b` 0.43× and 0.77×. The advantage was never tested: "Not run on the i9". | 2.5 bytes a row. Step instructions are not in the record. Whole phase c4/t4 0.42, b4/t4 0.67 (VM). | counting probe: a 20-bit entry with wide stores inside each bucket range (ctv's "missing" arm), build and walk instructions a row. | shrinks the table, to 2.25 MB: six fit under the knee, ten do not. | reg; ctv §1 to §3; feas §3 | "Dense packing comes after C, and only if C is encouraging"; the VM "cannot show a knee". | moved: C's science was encouraging (u6/p4 1.100) and its gate failed. And 2.25 MB is above the 1.2 to 1.4 MB target. |
| P40 | Fixed 64-byte line records: FOR-14, EF-12 and exceptions; built since as `h`, compact30 and the line table E. | About 2 bytes a row at one line a step, with the walk inside its budget (W1 ≥ 0.774 on the i9). | rejected | construction, three times. `h`: W1 0.475 (0.473 to 0.477), STOP. compact30: CI 4-thread wall +4.8% and +7.7%, rule failed. E: one-thread inversion 3.665× (3.591 to 3.740), FALSIFIED. The advantage held each time it was read: `h` removed 69% of the four-block penalty; compact30 0.840 CPU a row at four workers on the i9. | `h` 38 instructions a step. compact30 walk 20,207,700 instructions on block 1 against 15,426,892, build 4,369,204 against 5,429,062; staged step 12 a lane against 8. E 48.21 a row walk, 4.06 build. | i9 recipe: compact30 (`perf/c30-staged`, ba94ff5c) under plain workers at 6, 8 and 10. Or its frozen walk-only protocol at 8 and 10. | shrinks the table, to 1.87 to 1.95 MB: seven fit under 14.4 MB. | reg; for14 §5 to §7a; blog-st §6, §7; camp; charter §5.6; c30s; lfa; ereg; eres | `h`: "four-worker cost at least one-worker", with thresholds from "1.236 CPU-s/GB at one worker and 1.847 at four". compact30: "the per-byte core term binds", at 4 threads on 2 SMT cores. | moved: the i9's comparison point is now ten independent workers, where production's CPU grows from 0.58 to 3.69 CPU-seconds (sn §3b), not 1.236 to 1.847. |
| P41 | A two-plateau group descriptor (57 bits). | Two-plateau groups hold at least 8% of rows. | conditional | advantage, in part: 4.18% of rows (18.46% of non-translation rows), inside the registered marginal band of 3 to 8%. Not prototyped. | unknown | census: the same count on text and records blocks. Wikidata only so far. | shrinks the table, by little: 4.18% of rows. | reg; lfa | "Do not start a new compact-table prototype until the census and marker audit give us a reason to select one". | moved: three prototypes started anyway (`h`, compact30, E). None used this form: "non-RUN groups default to payloads" (lfa). |
| P42 | Segment-end markers without a bit in every row (one bit or six a group). | A group mark keeps the lanes balanced: makespan 1.146 against 1.155 for random starts at 32 lanes (model). | validated | n.a. Built since: `h` keeps a 32-bit start mask in each record's header and folds the test into one sentinel compare. E had no spare bit and kept a bitmap. D "did not say how lanes end". | `h`: 2 instructions a step (the sentinel's compare and branch). E's flagged-shape bookkeeping, its bitmap probe included: about 22 instructions a row. | counting probe: a group-mask start test in the room walk's skeleton (the model for E is 26 + 6). | shrinks the table: every table under 4 bytes a row needs a marker that is not a per-row bit. It is one of D's two open problems. | reg; for14 §2, §5; ereg §1, §4; sn §3a | "B3's extra operations as throughput (MA-2)", to be measured by P55. | yes: still unmeasured as throughput. `h` shows a mask form working, and E shows what the lack of one costs. |
| P43 | A separate RUN fast path, with a format branch on the step. | The verdict's claim: the branch mispredicts on about a fifth of steps and flushes other lanes' loads. | rejected | n.a.: analysis only. "no RUN fast path (a format branch on the common step) was ever timed"; "That branch cost is analysis with assumed latencies, never measured" (lfa). | unknown. Sized exactly: two-tier 5.70 bits a row, 0.641 MB a block; with raw payloads about 1.04 MB on the median block (9.2 bits a row). | counting probe: the RUN and non-RUN steps as a walk-only arm, listing plus callgrind `--branch-sim=yes` (lfa's steps 1 and 2). | shrinks the table: these are the only sized designs at or under 1.2 MB a block with one load on the common step. | reg; feas §3; lfa | "RUN costs no less space than FOR-14 in fixed records"; kill thresholds "bounded by #195's loss at 12 instructions". | no. The space premise holds for 64-byte records only: an 8-byte descriptor serves a RUN group. And 12 a step is CI's 4-thread threshold, not the i9's ten-worker one. |
| P44 | Byte recovery specialised to the block's alphabet. | A lookup shaped to few distinct bytes is cheaper than the general one. | superseded | n.a. Deferred "until a general lookup is measured". It was: the F lookup costs 28% of the walk at one block and 16% at four (`v`, i9); `h`'s carried byte is 4 instructions. | unknown. E is the nearest built form: a slot found by compare and popcount among at most 9 distinct bytes a line, 22 to 26 instructions a step. | none needed while tables walk backward: the byte is stored. | none | reg; cpsi §5.3; for14 §5; ereg §4 | "until a general lookup is measured" | no: measured, so the condition is met. The question lapsed with the LF direction. |
| P45 | Fixed against adaptive partition boundaries. | Uniform chunks are about 12% larger than optimal ones (Gov2 5.17 against 4.65 GB). | open | n.a. Counted since: the RUN share of fixed groups is 81.95% at 16 rows, 77.39% at 32, 72.18% at 64. Exact runs with a directory are 0.83 bytes a row (C); fixed 32-row groups with payloads are 5.70 bits. | C about 45 instructions a row (model, never counted). Two-tier: unknown. | census: the bench corpora's text and records at 16, 32 and 64 rows. | shrinks the table | reg; lit05; lfa; bt §4; sn §3a | "Checkable offline". | yes: checked in part, on Wikidata. |
| P46 | Choose each group's encoding by its hardware cost. | C_total = construction + navigation + cache pressure can be made measurable. | open | n.a. sn §3b gives the measurable form for the i9: GB/s at ten ≈ 3.0 ÷ (0.3 + 0.7 g) for a table small enough, with the knee at 14.4 MB live. | n.a.: it is the pricing rule itself. | i9 recipe: the `lbzip2 -2` pricing run (sn §4.2). | shrinks the table | reg; sn §3b; knee E2 | "not a directly measurable additive equation yet" | no: moved by sn §3b. |
| P47 | Select without PDEP; extraction inside the line. | An in-line field or select costs a few instructions with BMI2. | open | n.a. Counted since: `h`'s 14-bit extraction is about 12 instructions a step; BMI2 removes one (38 to 37); an `imul` form reaches 27, untimed. E's rank by `bzhi` and popcount makes a 22 to 26 instruction step. No EF arm was built: EF-12 groups became RAW in `h`. | as counted, left. | counting probe, per format. | cuts CPU a byte: it is the step cost of any packed table. | reg; for14 §5, §5a, §8a; ereg §4; isa; lit09 | "A precondition for any EF arm." | yes. And "BMI2 has no instruction for the rest" (for14 §5a). A whole-crate x86-64-v3 build left the walk at +2.5% (−1.3, +6.5) (isa). |
| P48 | Compression cuts lines occupied, not traffic a step. | `u` makes 26% more reads past L2 a byte than `p` at 4 blocks (1.765 against 1.399); crossings explain about 5 points. | open | n.a. Still unexplained. | n.a. | counting probe: the trace simulator of pw, which matches callgrind within 0.5%, replaying `u`'s access order. | shrinks the table. bw adds that the table's build is the largest cold writer in the default plan (35.5% of cold lines written), so a smaller table cuts traffic too. | reg; cpsi §5.3; pw; bw §2 | "Line crossings explain ≤ ~5 points." | yes |
| P49 | Is the smallest structure the fastest? | Size and speed do not rank alike. | open | n.a. Cases both ways since. compact30's walk alone is −9.4% against the room walk where 1.95 MB fits a 2 MB L2 (Xeon 8573C), and +21.4% on Zen 5's 1 MB. | By size: T-vector 4 bytes a row, 10.41 instructions a row; compact30 2.16, 22.45 (derived, block 1); E 2.04, 48.21; C 0.83, about 45 (model). | none needed: descriptive. | shrinks the table: the gate is the loop's cost, not the size. | reg; c30s; camp; ereg; sn §3a | "`u` (2.7 MB) moved the knee but lost per step." | yes |
| P50 | A per-segment variation bound; hybrid fixed-address groups. | Most groups fit a base plus small offsets; few need exceptions. | superseded | n.a. Answered by counting, not by a bound: RAW groups are 1.90% of rows in `h` on Wikidata and 3.17% to 4.74% on text; compact30 is 2.164 bytes a row. | n.a. | none needed | shrinks the table | reg; for14 §1; blog-st §6 | "the paper does not supply those assumptions" | yes: the counts stand in for them. |
| P51 | The cost and timing of exceptional steps; lane bunching; A₂. | Exceptions are rare and do not bunch in lanes. | open | n.a. Rates counted since: RAW arrivals 0.00708 a byte on Wikidata, 0.01118 on text, 0.0362 in the worst random block; 6.2% of arrivals fall in the tail. E's two-access steps: 1.82% (Wikidata), 3.52% (text), 72.94% (records). A₂ is still not computed. | `h`: a raw row costs one lane turn and one extra line. | census: A₂ and the per-lane arrival spacing. | shrinks the table: the exception rate sets admission by data class. | reg; c30s; for14 §2; ereg §3 | "kept for after the release" | moved: the release is out and the rates are counted. Their time cost was never isolated. |
| P52 | Discontinuity theory: ψ's translation runs are L's runs. | d = r − 1 on full blocks; K₃₂ from L's runs equals the census. | validated | n.a. Used since: in the LF direction a run of L is a run of consecutive entries, checked on 431 blocks and 15,148,086 runs (0.0722 a row). k-step pieces number 1.22 r, 1.51 r and 1.97 r at 2, 4 and 8 steps. | n.a. | none needed | shrinks the table: it is the count that sizes C and D (0.83 and 1.6 bytes a row on Wikidata). | reg; bt §1, §3, §4a; bdc §1, §4c | Counted on Wikidata's 233 full blocks. | moved by data class: text has 0.300 runs a row against 0.072, so D is 2.5 bytes a row at k = 2 and 4.6 at k = 4 there. |
| P53 | Several ψ cycles in a cyclic BWT. | A construction or walk argument fails when the permutation has more than one cycle. | open | n.a. E handles it: the plain walk finds the cycle's length and walks again; the segmented join "holds on every cycle"; Miri runs columns with several cycles. D has no lane design to check. | n.a. | none needed for a built table. Each new table needs a crafted-permutation test. | none | reg; ereg §1, §2; sn §3a | "handled in the walk (marker audit)" | yes |
| P54 | Choose the representation by concurrency. | A table that loses alone and gains under contention pays when chosen by thread count. | open | n.a. Every compact table measured fits the pattern: `u` 0.681 alone, and 805 against 565 MB/s at ten; compact30 1.148 alone and 0.840 at four workers (CPU a row, i9 walk). | The selector's cost is unknown. | i9 recipe: the pricing run, then the registered sweep (sn §4). | removes cooperation: it is share-nothing's own choice, a small table at ten workers and the T-vector alone. | reg; cpsi; blog-st §7; sn; bdc §7 | "A 'possible consequence', not proposed." | no: sn proposes exactly this. |
| P55 | A dependent-successor throughput benchmark at 1 to 32 lanes. | Successor throughput against lane count separates a format's step cost from its memory cost. | open | n.a. Built since in three forms: the walker ladder (i9, 1, 8, 16, 32 lanes, with counters), `sweep.rs` walk-only rows, and the `phases` research table (bench 3.5.0). None runs a compact layout at 1 to 10 workers on the i9. | n.a.: an instrument. | i9 recipe: sn §4's sweeps are this benchmark. | shrinks the table: it is the gate's instrument. | reg; ladder; c30s; eres; sn §4 | "stays unscheduled until a compact layout is chosen for its own reasons" | moved: a small table is now the objective, though no layout is chosen. |
| P56 | libsais's cyclic inverse as reference R. | A reference kernel exposes gaps: libsais 12.10 c/B against 18.21 on Wikidata (VM, before #32). | validated | n.a. Later rows: libsais over mibz2 1.05× (1.04 to 1.06) in time in CI's kernels table on the master bench run; 1.44× on CI's EPYC earlier. | libsais 23.2 instructions a byte at 3×3 against production's 23.8 (lfa). | none needed | none | reg; lfa | After #32: libsais 10.86 against 12.92 c/B (VM). | moved: the walk has since lost 22% of its instructions (#157) and gained staging. "No libsais run on the i9 postdates 0.2.0." |
| P57 | libsais's lead is a memory-pressure effect. | Its lead grows with live blocks: 0.989, 1.062, 1.251, 1.320 at 1, 4, 8, 10. | validated | n.a. lfa keeps two bytes a step as "a candidate, not a finding", and closes the direction with a condition. | n.a. | none needed: s10x1 against p10 is run (1.320). | cuts CPU a byte: fewer reads past L2 a byte (0.897 against 1.366) is what pays with ten large tables live. D's k-step cells are the same lever. | reg; lfa | "The team policy exists to avoid that regime" (lfa's verdict). | no: sn proposes ten independent tables. lfa's condition, "a target machine and plan that keep 6 or more independent tables live", is met in the letter. |
| P58 | libsais: prefetch reach covers an i9 L3 hit, not EPYC DRAM. | Fill-source counters differ by machine; more chains help on EPYC. | conditional | n.a. Not run. lfa closed the libsais direction: "the shipped decoder already does at least as well as libsais at its operating points". | n.a. | i9 and EPYC counters. None is open: CI has no PMU and the workstation's perf is closed. | none | reg; lfa; bw | libsais 1.13× faster on the i9 and 1.44× slower on CI's EPYC. | moved: a third point: 1.05× slower on the master bench runner (lfa, citing run 41980640481). |
| P59 | libsais: the instruction budget pays only at a high clock. | Instructions a cycle differ by CPU. | conditional | n.a. Not run. | libsais 23.2 instructions a byte at 3×3, production 23.8, the research kernel 35.3 (lfa). | as P58. | none | reg; lfa | "about 10.5/B" (reg). | yes, so far as the record says: not checked since. |
| P60 | libsais: the bigram scatter and two post-walk copies cost bandwidth and TLB reach. | Their share shows in a profile. | conditional | n.a. Not run. Its setup is about 4.2 c/B against our port's 7.1 to 7.6 (lfa). | n.a. | as P58. | none | reg; lfa | none recorded beyond the hypothesis. | yes, so far as the record says: not checked since. |
| P61 | libsais: load-port pressure, with lane state through `slots[]`. | Port counters show it; a lanes-in-registers variant removes it. | conditional | n.a. Not run. | n.a. | as P58. | none | reg | none recorded beyond the hypothesis. | yes, so far as the record says: not checked since. |
| P62 | libsais: 28 KB of `first_symbols` evicts prefetched lines from L1D. | A smaller table (13 or 14 bits) speeds it. | conditional | n.a. Not run. | n.a. | as P58. | none | reg; lfa | none recorded beyond the hypothesis. | yes, so far as the record says: not checked since. |
| P63 | libsais: TLB reach on EPYC and the VM. | dTLB misses differ; huge pages help it there. | conditional | n.a. Not run. Closed for mibz2 on the i9 by P33. | n.a. | as P58. | none | reg | P33's i9 figure. | yes for the i9; CI unread (see P33). |
| P64 | libsais: its serial fallback explains the 13 ms s3x3 straggler. | A count of fallbacks matches the stall. | conditional | n.a. Not run: "the fallback path is a hypothesis for it" (lfa). | n.a. | a fallback count in the harness, on the i9. | none | reg; lfa | one stalled round of s3x3. | yes, so far as the record says: not checked since. |
| P65 | E3's libsais half: the phase attribution. | A `perf record` split says where libsais's time goes. | open | n.a. "libsais's build is inside its walk call, so it has no separate attribution"; the rematch's symbol profiles "were withdrawn as confounded by setup" (lfa). | n.a. | i9 recipe: `perf record` of the timed rounds only. Blocked while perf is closed (bw). | none | reg; lfa; bw | "is not stated" whether it was done. | yes: still not done. |
| P66 | The shipped cooperative kernel against libsais (#107). | l3x3 over s3x3 above 1 at the inversion boundary: 1.114 (1.027 to 1.208). | validated | n.a. Reproduced at 1.096 (1.080 to 1.112), l3x3 at 2,584 MB/s. With MTF timed, a tie: 1.022 (0.986 to 1.059). | The plan it validates costs 38.0 instructions a decoded byte against one thread's 26.7 (sn §1). | none needed | none: it validates the cooperative plan that share-nothing would retire. | reg; lfa; sn §1 | The comparison arm is l3x3, the plan 0.2.5 runs. | moved: the default since 0.2.6 is the pipelined three teams, and the pass with MTF timed predates two MTF speed-ups (#172, #207). |
| P67 | Intel IPP as a timing reference. | A second reference kernel. | rejected | n.a.: not pursued (binaries only, needs Intel's runtime). | n.a. | none needed | none | reg | "binaries only". | yes |
| P68 | rapidgzip's leads: seek windows, memcpy for repeats, an allocator, a block finder. | Each transfers to bzip2. | superseded | n.a. The finder part was reversed: a bit-aligned bzip2 finder was built on 10 November and gave 2,563 against 1,598 MB/s at ten workers on level-1 input (i9). The allocator stays out: "not applicable" (btr §2). | Finder: 9.3 GB/s of compressed input on one thread, 0.19% of a decode (local). | none needed: the registered `bench-par-scan` ran. | removes cooperation: the finder removes the single scanner, the other wall beside the table. | reg; ps; sn §3b; btr | "The block finder is Deflate's." | no: ps §7 reads indexed_bzip2 and rapidgzip's shifted-byte search as prior art for a bzip2 finder. |

## Compact tables against the new objective

Every compact or succinct table in the record, with what the record holds. The objective: a
table of 1.2 to 1.4 MB a block, and an inversion (build plus walk) of up to about 2.5 times
production's at one thread, which still ties the cooperative default at ten threads on the i9
(sn §3b).

| table (ID) | bytes a row; a 900k-row block | step or loop, counted | one-thread cost, measured | build cost | under contention | run at 8 or 10 workers on the i9? |
|---|---|---|---|---|---|---|
| production's T-vector | 4; 3.6 MB | walk 10.41 instructions a row, build 4.92 (ereg) | 1 | 1 | plain workers: 1,145 MB/s at 4, 741 at 8, 631 at 10 (sn §3b) | yes |
| `v`: 32-bit link, byte from F (P37's control) | 4; 3.6 MB | 28 instructions, 7 loads, against 16 and 3 (for14 §5) | i9: v1/o1 0.829 whole, walk 0.721 | 1.279× production's rate (store-only) | v4/o4 0.950; v6 1,204 MB/s against p6 1,102 | no: 1, 4 and 6 only |
| `u`: 24-bit link (P37) | 3; 2.7 MB plus 28 KB | 31 instructions, 7 loads | i9: u1/o1 0.681 whole; walk about 0.598, build about 1.005 (derived) | VM: `v`'s build 1.55× production's; `u` gives it back | u4 1,313, u5 1,479, u6 1,615, u8 1,213, u10 805 MB/s; p8 697, p10 565 (cpsi grid) | **yes: 8 and 10**, inversion harness, one core a block. Never in the decoder, never under teams. |
| `c`: 20-bit dense (P39) | 2.5; 2.25 MB | not in the record | VM only: walk 0.74× the link-only control | 0.15× (155 against 1,052 MB/s) | c4/t4 0.42, on a VM that cannot show a knee | no: never on the i9 |
| `b`: ll16 + ll4 (P39) | 2.5; 2.25 MB | two random accesses a step | VM only: walk 0.77× | 0.43× | b4/t4 0.67 (VM) | no: never on the i9 |
| `h`: FOR-14/raw (P40) | 2.076; 1.868 MB | 38 instructions, 5 loads, 3 stores; `k` 31 and `j` 27, compile only | i9: W1 0.475; 2.676 against 1.275 ns a row | none built: records copied in | W4 0.688; penalty from 1 to 4 blocks +0.223 ns a row against +0.716 | no: 1 and 4 only |
| compact30 (P40) | 2.164 (2.13 in #195); 1.95 MB | unstaged step 21 instructions and 4 loads against 16 and 3; staged 12 and 3 against 8 and 2; walk +31.0% instructions on block 1 | i9 walk 1.148 (1.147 to 1.149) CPU a row; CI decoder +11.4% and +13.7% at 1 thread | Assembly 1: −19.5% instructions on block 1; v2 at 1.086× four-worker build CPU, from a capture that failed its validity gate | i9 four workers 0.840 (0.838 to 0.843) CPU a row, walk only; CI 4 threads +4.8% and +7.7% | no: 1 and 4 only, walk only. "Cooperative teams … stay on the T-vector" (c30s). |
| line table E (new, P40's family) | 2.039 Wikidata, 2.077 text, 4.818 records; about 1.8 MB | walk 48.21 instructions a row (step 22 to 26, bookkeeping about 22); build 4.06 | EPYC 9V74: walk 4.342×, inversion 3.665× | 0.82× the instructions, 1.827× the time | not measured | no: never on the i9, "untimed by design" |
| two-tier: descriptor plus payloads (P43; P13 in section a) | 5.70 bits; 0.641 MB | never built; modelled chain 43 cycles a RUN step, 90 a FOR step | none | "direct construction", untested (feas §4) | none | no |
| two-tier with raw payloads (lfa) | 9.2 bits; about 1.04 MB on the median block, 0.66 to 1.53 by block | never built or counted | none | none | none | no |
| two-plateau descriptor (P41) | a 57-bit group descriptor, for 4.18% of rows | never built | none | none | none | no |
| design C: runs and a rank directory (bt §4; section a's family) | 0.83; 0.75 MB | about 45 instructions a row (model, sn §3a), never counted | none | two sequential streams of 0.83 bytes a row (model) | none | no |
| design D, k = 4 (bt §4a) | 1.6; 1.44 MB | about 12 a row walk plus 6 to 10 build (on paper, sn §3a) | none | 0.197 n cell-steps at about 30 instructions (paper) | none | no |
| cacheline-ef (lit09, literature) | 11.64 bits a value, 44 values a line | not a successor walk | PtrHash query 12.9 ns against 12.5 for a plain u32 vector | n.a. | n.a. | no |

- **Only `u` was ever read at eight or ten workers on the i9,** and only in the inversion harness.
  It is also the largest of them: 3 bytes a row.
- **The three tables near 2 bytes a row (`h`, compact30, E) were each stopped at one or four
  workers,** on gates set before sn §3b: W1 ≥ 0.774 (`h`), a CI 4-thread win (compact30), and a
  one-thread inversion under 2.0× on a CI EPYC (E).
- **No table at or under 1.4 MB a block has been built.** The two-tier forms and C are sized;
  D at k = 4 is sized at 1.44 MB.

## Which verdicts rest on "latency-bound", which on "instruction-bound"

The record has held both, on different machines and for different questions.

| date, machine | the statement | what rests on it |
|---|---|---|
| 27 Oct, CI and VM | "the walk is instruction-bound, not latency-bound" (pair-walk, as quoted by lfa and bc/nogo) | the register's Class II verdicts: P37 ("byte recovery and packing exceed the residency gain"), P39 |
| 30 Oct, i9, analysis | "Extra µops per step use the reorder window … fewer misses then overlap. This is the leading reading, not a finding." (feas §2) | P37's stage-B stop; P38; P27 |
| 1 Nov, i9 | "the shipped walker is partly execution-bound at one worker"; the extra work "was neither overlapped nor hidden" (for14 §0, §7a) | P40's `h`: STOP by rule 1 |
| 2 Nov, i9, counters | "the prefetched walks are not waiting on their demand loads"; D3 "classifies nothing"; "FOR-14's calibration … is not confirmed" (ladder §8.3, §8.4) | no walker rewrite (D1); P23, P29 |
| 5 Nov, VM and CI, no counters | "the walk is bound by the number of misses a core can keep in flight, and by their latency … Fewer instructions don't help a loop that waits on misses." (wr) | #168, #173 closed; #181 merged; P25's origin; the partitioned walk's premise |
| 5 Nov, CI AMD, four copies | "'reversal' … is itself a 'reading that fits', INFERRED with no counters"; the four-copy walk "tracking per-byte work"; the lane-independent term is 63 to 70% (bc/nogo, bc/walk) | compact30's predicted and measured loss at 4 threads (c30s); the partitioned walk rejected (pw) |
| 10 Nov, i9, model | "Where the T-vector's load is an L3 hit anyway (the i9, 256 KiB L2), the lanes hide the second load, and the step is bound by its instructions" (bt §4) | C's 1.6× estimate, later withdrawn |
| 10 Nov, CI EPYC 9V74 | registration: "The room walk is bound by load latency, near 2 instructions a cycle on CI's runners". Result: "The walk is instruction-bound, as the counts said" (of E's walk) (ereg §5, eres) | E FALSIFIED at one thread; the transfer to ten workers "by the knee model" |

- **Both hold, for different changes.** Removing instructions from the shipped step bought
  nothing (#168 −2.6% instructions, #173 −2.0%; #181 added 4% and was faster). Adding
  instructions cost time close to in proportion: `h` 0.475 against 16/38 = 0.421; E 4.34× the
  time for 4.6× the instructions.
- **In this slice, on the instruction side:** P37, P39, P40 (`h`, compact30, E) and P43 (a
  model).
- **On the latency or miss side:** P23's condition, P25's first evidence, P26, P29, P30's
  partitioned construction and P33.
- **On cache capacity:** P36, P37's science and P57.
- **None of the instruction-side verdicts was read at ten workers on the i9,** except `u`'s.

## Mined

Ranked by what each could be worth. Figures marked (inferred) are this section's arithmetic on
recorded figures.

- **P46 and P49, the gate itself, and design C.** sn §3a withdrew C against a gate of "a
  one-thread walk within 1.44× of the plain walk, about 15 instructions a row": "C is
  withdrawn as a walk table." eres set E's reopening at the same 15. Then sn §3b replaced it:
  "The gate, from these figures and no longer from §2's 1.44×", with break-even "near
  g = 2.5", where g is the inversion's time, not the walk's instructions. §3a's table was not
  redone. The i9's own price of an added step instruction is in the record four times: 0.064 ns
  a row (`h`: +22 instructions, +1.401 ns), 0.058 (`u`: +15, 2.151 against 1.285 ns),
  0.041 (`v`), 0.036 (compact30: +5, 1.402 against 1.221 ns) (inferred from for14 §7a, the cpsi
  grid and blog-st §7). One thread's inversion on the i9 is about 1.94 ns a row (0.58 CPU-s ×
  0.7 ÷ 209.7 M rows, sn §3b; inferred). So C's modelled 45 instructions a row with a nearly
  free build, about 30 more than production's 15.3, would be g ≈ 1.6 to 2.0 on the i9
  (inferred), not 4.6. C is 0.75 MB a block: ten are 7.5 MB, the only sized design that is
  plainly in level 1's regime. **Falsifier:** a counting probe, callgrind of C's complete loop
  (fill plus walk in the room skeleton) on three blocks; it has never been counted. Its two
  dependent loads a step may cost more than their instructions, which only timing shows.
  **Worth:** 3.0 ÷ (0.3 + 0.7 g) = 2.1 GB/s at g = 1.6 and 1.8 at g = 2.0, against about 1.5
  (sn §3b's model with the inferred g).
- **P43, the RUN fast path and the two-tier table.** The register rejects it "by analysis":
  a format branch "mispredicts on ≈1/5 of steps". lfa says of the same analysis: "That branch
  cost is analysis with assumed latencies, never measured", and "no RUN fast path … was ever
  timed". lfa's own kill thresholds (RUN step over 10, weighted step over 12) are "bounded by
  #195's loss at 12 instructions", a CI 4-thread loss; sn §3b sets the i9's tie at g near 2.5.
  feas §3 says of the RUN step: "That is one add after the fetch, cheaper than today's shift",
  which sits against sn §3a's "the step itself cannot be brought near the T-vector's three
  instructions". The sizes are exact: 0.641 MB a block, or about 1.04 MB with raw payloads.
  **Falsifier:** lfa's steps 1 and 2 as written, a walk-only arm counted by listing and
  callgrind `--branch-sim=yes`. Its builder is the unpriced part: an F-order constructor is
  what compact30's campaign fought. **Worth:** sn §3b's table by g, if ten tables of 6.4 to
  10.4 MB behave as level 1's: 2.6 GB/s at g = 1.2, 1.8 at 2.0. The g is not in the record.
- **P40, compact30 at seven to ten plain workers on the i9.** c30s closes it "as the
  production table on CI", reopening "only if the sibling-pair matrix shows the walk's excess
  to be memory-side"; the matrix was never run. Its premise is CI's: 4 threads on 2 SMT cores,
  where "the per-byte core term binds". On the i9 the same table measured 0.840 of
  production's CPU a row at four workers (blog-st §7), and "Cooperative teams … stay on the
  T-vector" kept it out of every i9 decode. Its one-thread g is about 1.16 to 1.20 (inferred:
  CI's +11.4% and +13.7% at 1 thread over an inversion share of 0.7), far under 2.5. At
  1.95 MB it is over the size target: seven tables are 13.65 MB, ten are 19.5 MB.
  **Falsifier:** an i9 recipe on `perf/c30-staged` (ba94ff5c), the forced plain-workers arm at
  6 to 10. The code exists, is byte-exact and passed CI. **Worth:** sn §3b's model, with the
  inversion slowed by a factor f read from the cpsi grid at the same live bytes (next finding):
  N × 368 MB/s ÷ (0.36 + 0.84 f). Here 368 is 209.7 MB over 0.57 CPU-seconds, 0.36 is the
  other 30% at level 1's CPU growth of about 1.2, and 0.84 is 0.7 × g at g = 1.2. With f of
  1.25 to 1.45 at seven workers, 1.35 to 1.7 at eight and 2.0 to 2.5 at ten, that is about
  1.6 to 1.8 GB/s at seven, 1.7 to 2.0 at eight and 1.5 to 1.8 at ten (inferred), against
  about 1.5. A modest gain at best. Its value is as the first test of sn's model with a real,
  smaller table.
- **P36, the frontier's bound.** feas §5 bounded a compact table on the i9: "The gain stops at
  five teams", at most "5/4 = 1.25× on the inversion", and its cross-check from block size
  "bound the gain at about 1.38× over today's 1,256", where level 1 "plateaued at about 1,460
  where the scanner binds". knee H3: that review's premise "is contested by E1". sn §3b: with
  the finder, level 1 runs at 2,563 MB/s at ten workers, against about 1,500 for the default:
  1.71× (inferred from those two figures), past both bounds. **Falsifier:** the `lbzip2 -2`
  pricing run (sn §4.2). **Worth:** 3.0 GB/s ÷ (0.3 + 0.7 g): 2.6 at g = 1.2, 2.3 at 1.44.
- **P37, `u`'s grid is a price list already paid for.** The register keeps of that run "u peaks
  at 6 blocks" and "u6/l4x2 0.687". The grid also holds u8 and u10. A worker's rate against
  u1's 371 MB/s is 1.13× slower at four (10.8 MB live), 1.25 at five (13.5), 1.38 at six
  (16.2), 2.45 at eight (21.6) and 4.6 at ten (27.0); production's is 1.49 at four (14.4 MB),
  2.09 at five, 2.96 at six, 6.2 at eight, 9.6 at ten (inferred from the grid's medians). ereg
  assumed "E's own factor at about 1.5 or more" at 18 MB; the grid puts it at about 1.7
  (`u`, interpolated) to 2.1 (production at five).
  The slowdown is not a function of live bytes alone: at 13.5 to 14.4 MB `u` reads 1.25 and
  production 1.49. **Falsifier:** a read of the archived bundle,
  `coop-20261029T215136Z.jsonl`, per round. **Worth:** it prices any table of known size at N
  workers before a run; no throughput of its own.
- **P40, E's verdict on the i9 is a transfer the registration disclaimed.** ereg §8: "The
  verdict holds for that model. The other model is NOT SHOWN, not inferred." eres: "E as built
  is closed, for one thread and, by the knee model, for the ten-worker plans it was proposed
  for. No i9 timing follows." The ratio 3.67× is over a room walk of 3.87 c/B on EPYC 9V74.
  On the i9 the same walk cost 6.90 cycles a row (ladder, 2 November, before #157, #171 and
  #181), so a fixed added cost is a smaller ratio there. The i9's g would be between about
  2.0 (E's 37.8 added instructions a row at the i9's lowest recorded price, 0.036 ns) and 2.6
  (E's +13.0 cycles of walk and its 1.83× build, on 6.90 cycles of walk and about 2.9 of
  build) (inferred): at the 2.5 tie. **Falsifier:** none
  worth running: an i9 recipe would only confirm a tie. **Worth:** nothing for E. It shows
  that a one-thread ratio from a CI EPYC overstates the i9's, for every table.
- **P57, the reopening condition is met in the letter and is worth nothing alone.** lfa: two
  bytes a step "reopens only on … a target machine and plan that keep 6 or more independent
  tables live", and "The team policy exists to avoid that regime". sn proposes ten independent
  workers. **Worth:** bounded below the default. Ten plain workers on level 9 give 619 to
  631 MB/s (sn §3b); libsais's lead at ten is 1.320; 631 × 1.320 = 833 MB/s (inferred),
  against about 1,500. The lever pays only joined to a small table, which is design D.
  **Falsifier:** none needed.
- **P54 and P34, the per-block and per-plan choice has a free input now.** #27 was rejected on
  "Wikidata ±0.3%; pattern +21.0%" at 1 thread, with the condition "a selector that wins on
  pattern at both thread counts without moving Wikidata". Every small table since depends on a
  choice a block: "Admission: by the record count, known before the build" (bt §4); "chosen by
  the block's run count" (ereg §3). The emitter gives that count before the build.
  **Falsifier:** CI A/B of a selector on the run count. **Worth:** cannot be bounded from the
  record.
- **P27, the asymmetry.** The register's condition was "a new discriminating test before any
  experiment that adds instructions to the step". Three tables were built without it. The
  ladder, the one test run, found that the capture "shows neither that extra instructions are
  the walk's main limit nor that they cannot be one". Yet the losses are regular: an added
  step instruction costs 0.036 to 0.064 ns a row on the i9 and about 0.34 cycles on EPYC 9V74
  (inferred from for14 §7a, blog-st §7, the cpsi grid and eres). So a counted loop predicts a
  table's one-thread loss well enough to screen it, though counts do not predict gains from
  removing instructions. **Falsifier:** the next counted table, timed once. **Worth:** a
  method: it turns the gate into a count. No throughput of its own.
- **P25, P28 and P22: one census sentence is out of date.** bc: "No Intel lane or prefetch
  sweep has run." The compact30 dispatch on the same day ran the 59-setting grid on a Xeon
  8573C. Alone, no room setting is faster beyond noise (48 lanes −1.5%, −3.4 to +0.4); no
  prefetch is +9.2% alone and +19.8% to +23.9% at four copies; 32/L2 is +13.9% alone. **Falsifier:** done; a read of `sweep-42036331632/`. **Worth:** nothing to gain. It
  closes P25 on Intel and answers P28's missing comparison.
- **P30, dense sorted starts cut misses in simulation and were closed on CI's prices only.**
  The register: "never swept". pw: master's walk given dense, row-sorted starts "captures 74%
  to over 100% of partitioning's T-vector saving", 0.926 to 0.756 misses a byte. sc closes it:
  the lean path costs 33 to 64 cycles a segment and "the whole miss saving is worth only 6–9
  cycles a segment", at CI's latencies; "Cross-chain lockstep is closed on CI". Past the i9's
  knee a read waits 137 cycles at 6 tables and 361 at 8 (knee E2), a regime sc did not price.
  **Falsifier:** the trace simulator at the i9's cache sizes. **Worth:** cannot be bounded
  from the record, and the regime disappears if tables shrink.
- **P26, L2 prefetch in the team walk is still one line and still unrun.** The register:
  "the cheapest test the counters point at, about a line in the cooperative walk". lfa, eight
  days later: "No i9 team run with L2 prefetch was found in the archive." bw weakens its
  premise: the plan's cold traffic is sequential writes, not table reads. **Falsifier:** one
  arm in an i9 recipe. **Worth:** cannot be bounded from the record; none under share-nothing.

## Changed since 30 October

- **P20:** #168 closed, #181 merged (dfw, so).
- **P21:** re-swept on three CPU models, still the optimum (cw, sweep8573); a segment priced (sc).
- **P22:** i9 counters (ladder); Xeon 8573C and Zen 5 figures (sweep8573, bc/walk).
- **P23:** CI sweeps and the i9 ladder (cw, ladder).
- **P25:** conditional to rejected: not reproduced on the staged kernel (cw); Intel row (sweep8573).
- **P26:** premise weakened (bw); confirmed unrun (lfa).
- **P27:** the ladder decided nothing; `h`, compact30 and E priced an added instruction.
- **P28:** an Intel prefetch on and off comparison exists (sweep8573).
- **P29:** fitted knees and the ladder's occupancy caveats (bc/walk, sc, ladder).
- **P30:** dense sorted starts simulated and closed on CI (pw, sc).
- **P31:** #157, #171, #181; counted at 10.41 instructions a row (ereg).
- **P33:** CI's regime named as unread (bc/nogo); the ladder's dTLB reading; tm's H4.
- **P34:** pattern moved by staging (wr); admission by run count designed (bt, ereg, bdc).
- **P36:** conditional to open: the bound contradicted and the direction adopted (knee H3, sn).
- **P37:** read against the new gate (sn §3b); cited in knee E2 and H3.
- **P38:** open to superseded: built as `h`'s header byte and compact30's; dissolved by the LF
  direction (for14, c30s, bt).
- **P39:** no new measurement; its reopening condition is out of reach of the size target (sn).
- **P40:** open to rejected: `h` on the i9 (for14), compact30 (#155, #195; blog-st, camp, c30s),
  E (eres).
- **P41:** unused by three prototypes (lfa).
- **P42:** a mask form built in `h`; E's bitmap; D's lane ends open (for14, ereg, sn §3a).
- **P43:** found to rest on an unmeasured model; falsifier written (lfa).
- **P44:** conditional to superseded (cpsi, for14, bt).
- **P45, P46, P47, P48, P49, P51, P52, P53:** new counts or cases, statuses unchanged (lfa, sn,
  for14, isa, bw, c30s, ereg, bt, bdc).
- **P50:** open to superseded: answered by the censuses (for14, blog-st).
- **P54:** now the proposed direction (sn).
- **P55:** conditional to open: three instruments built; its condition moved (ladder, c30s, eres, sn).
- **P56, P57, P66:** the left-field check (lfa); P66's plan superseded as the default by 0.2.6.
- **P58 to P64:** open to conditional: the libsais direction closed with named conditions (lfa).
- **P65:** the rematch's profiles withdrawn; perf closed on the workstation (lfa, bw).
- **P68:** rejected to superseded: the finder built and measured (ps, sn §3b).
- **Unchanged:** P24, P32, P67.

## Reopen candidates

Rejected or superseded IDs whose rejection was of a construction only.

- **P43, the RUN path and two-tier table.** Never built. A new construction must count at no
  more than about 38 instructions a row for build plus walk to tie (15.3 × 2.5), and about 22
  for g = 1.44 (15.3 × 1.44), if time follows instructions as E's did on EPYC. lfa's stricter
  rule (RUN step at most 10 a lane, weighted at most 12) is CI's.
- **P40, the 2-byte line records.** compact30 already meets the step gate (g about 1.16 to
  1.20, inferred) and misses the size: 1.95 MB. A new record would need about 1.44 MB a block
  or less to keep ten under 14.4 MB, at a one-thread inversion under 2.5× on the i9. `j`
  (27 instructions) and E in the room skeleton (about 32, model) are the uncounted or untimed
  variants on file; neither changes the size.
- **P37, `u`.** Clears the step gate (g 1.47, inferred) and cannot meet the size at 3 bytes a
  row. Not a candidate at ten; at six it already beat p4 by 1.100.
- **P39, 20-bit packing.** Rejected on narrow stores, on a VM. A wide-store build would have
  to bring the build from 0.15× to near 1×. At 2.25 MB it still misses the size target.
- **P34, the per-block choice.** A selector on the run count, at a cost not in the record, that
  holds Wikidata within noise.
- **P33, huge pages.** Rejected on the i9's counters only. CI's co-run regime is unread, and
  the change is Louis's to allow.
- **Not candidates:** P23 and P25 (advantage disproved on three CPU models); P24 (advantage
  disproved); P32 and P67 (never tried, by judgement); P38, P44 and P50 (superseded by a
  different route, not by a failed build).
