#### FRONT serial-stages
MACHINE: CI runners: 4 vCPU = 2 physical cores x 2 SMT siblings (pairs 0-1 | 2-3), with no core PMU (pmu.yml run 40857395037: core_pmu false, cycles/instructions unsupported). The bench's 1t arm is BzReader::new, the sequential path on one pinned CPU (examples/ab.rs:432), so its wall is the sum of the stages. 2t is 2 workers + scanner + main on CPUs 0,2: one thread a core, no SMT co-run (amplification 1.04x on the 8573C). 4t is 4 workers + scanner + main on all 4 CPUs, SMT siblings shared. Master-decoder figures used, each with its run id and CPU:
- Xeon 8573C, master bench 41980640481 (d1fd4405, 3.0 GHz reported): wall 1t 0.802, 2t 0.426, 4t 0.3426 s. 4t CPU 1.314 s (scanner 0.135, workers 1.065, main 0.03), runq 0.368 s. Stages 1t (ps/B): scan 418, MTF 720, build 582, walk 1774 (assembly 95 inside it), RLE1 92, CRC 46.
- EPYC 7763, run 41986912287 base arm (0769b52): 4t 0.4528 s, CPU 1.733 s (scanner 0.12, workers 1.485). Stages 1t: 385 / 619 / 682 / 1531 (98) / 106 / 87.
- EPYC 9V45, run 41986885355 attempt 2, base arm (0769b52): 4t 0.1789 s, CPU 0.658 s (scanner 0.06, workers 0.495). Stages 1t: 234 / 388 / 326 / 625 (25) / 36 / 56.
- Xeon 8370C, run 41986908473 base arm (5761a32, master plus the unrle Step refactor only): 4t 0.3922 s, CPU 1.521 s (scanner 0.14, workers 1.25).
- 4-copy stage tables (sweep.yml tool=phases: each stage alone on CPU 1, then one copy pinned to every CPU):
  - run 42008210159 (b7734313, Xeon 8370C, 3.5806 GHz reported), co-run/alone factors: scanner 1.56

- [OBSERVED] 4t wall <- total co-run CPU / 4 (saturated, runq > 0) <- sum over stages of (1t stage cost x co-run factor)
  CLAIM: Regime: at 4 threads on CI all 4 vCPUs are saturated and no serial stage binds, so the 4t wall is the total co-run CPU divided by 4 (times 1/utilisation). A non-walk stage's 4t wall cost is its co-run CPU over 4, and its recoverable wall is its co-run CPU saved over 4, wherever the thread runs (the scanner included). Across all runners the non-walk stages (scanner, MTF, build, RLE1, CRC, assembly) come to 45-54% of the 4t wall.
  EVID: Utilisation, CPU/(4 x wall): 0.959 on 8573C (run 41980640481), 0.957 on 7763 (41986912287), 0.920 on 9V45 (41986885355), 0.970 on 8370C (41986908473). runq 0.37-0.40 s a run. 2t is saturated too (0.836 s CPU / 0.852 on 8573C). The scanner is not binding: busy 34-42% of the 4t wall, and in occupancy runs 41976148952 and 42009406264 the scanner waits on full slots and the reader is starved 77-79%. The stage shares are INFERRED: 4-copy factors from runs 42008210159 (8370C) and 42011789918 (7763), applied to each runner's 1t stage table, then split across the measured scanner and worker CPU. Non-w
  SHARE: non-walk 45-54% (assembly counted as post-walk)
  RECOV: framework, not itself recoverable. Ceiling if every non-walk stage cost nothing: 185 ms of 343 on 8573C. The floors (below) cap it at about 28% of the 4t wall.
  FALSIFIER: A change that measurably cuts one stage's co-run CPU, such as the scanner's, where the 4t wall falls by less than half of (CPU saved)/4 across 2-3 runs.

- [INFERRED] 4t wall <- worker CPU/4 <- MTF co-run cost (1.33-1.54x) <- 0.0722 moves/B x (store-forward-fail commit wait + blend) + run/move mispredicts
  CLAIM: MTF (priced only; a separate workflow owns MTF constructions). Its time is a serial chain through the move-to-front list in memory. Each move's vector store and the following 1-byte store of the new front are read by the next move's two 32-byte loads, of base and base-1. A wide load over a narrower younger store cannot be forwarded, so it waits for the store to commit: about 20-24 cycles a move against a throughput bound of about 0.27 c/B. Floor with the list front held in a register: about 5 cycles a move, so MTF at about 0.8 c/B, about 40% of today.
  EVID: Code: src/mtf.rs:149-156 at d1fd4405 (blend store, then `*base = v`). The comment at src/mtf.rs:224-229 already says the next load 'can't be forwarded from it and waits for it to commit'. Local disassembly of mtf_avx2: `mov BYTE PTR [rsp+0x60],al`, then on the next move `vmovdqa ymm0,[rsp+0x60]` and `vpblendvb ...,[rsp+0x5f]`. Derivation: 15.15M moves; MTF 1t is 2.16 c/B on 8573C at its reported 3.0 GHz, which is 453M cycles. About 95M of that is run arithmetic, mispredicts on the run/move pattern, and memset/memmove calls for runs over 64 B and nn >= 32 (0.5M calls). That leaves about 23.6 cy
  SHARE: 13-17%
  RECOV: MTF's 4t share: 58 ms (17.0%) on 8573C, 60 ms (13.2%) on 7763, 28 ms (15.8%) on 9V45, 57 ms (14.4%) on 8370C. Recoverable: 1t MTF saving of 30-55% times 4t retention of 0.6-1.0 (a shorter chain gets less SMT benefit). 8573C 10.5-32 ms (3.1-9.4%); 7763 2.4-7.3%; 9V45 2.9-8.7%. Point about 5-6%. At 1t: 4.9-12.8%.
  FALSIFIER: A construction with no store-to-wide-load on the list that cuts the 1t MTF stage by less than 15% refutes the commit-wait chain. An MTF stage cut of 30% or more that moves 4t wall by less than 1% refutes the regime rule for MTF.

- [INFERRED] 4t wall <- scanner-thread CPU/4 (and i:rd idle) <- 0.0646 lookups/B x (store-forward on buf + LUT load + shift)
  CLAIM: Huffman (scanner): the bit buffer is not in a register. In the 4-lookup unrolled loop each lookup stores buf and does a memory read-modify-write on live, and the next lookup reloads buf. So every lookup carries a store-to-load forward of about 4-5 cycles. The chain is then about 13-15 cycles a lookup (forward + shr + LUT load + and + `shl r64,cl`), against about 8-9 with buf in a register and a throughput bound of about 2-3. Probable cause: the cold non-inlined calls inside the loop take &mut Bits (Bits::refill_slow, Huff::decode_entry_slow), which stops LLVM promoting the struct to registers. The journals record no attempt at this.
  EVID: Local disassembly of read_item (rustc 1.97.0, d1fd4405): per step `shl rsi,cl; mov QWORD PTR [r15+0x18],rsi; sub DWORD PTR [r15+0x20],ecx`, then `mov rsi,QWORD PTR [r15+0x18]` at the next step, and the refill does `or rsi,QWORD PTR [r15+0x18]`. Code: src/bits.rs:49-50 (#[cold] refill_slow(&mut self)), src/block.rs:324-357 (step! macro, decode_entry_slow(e, br) at :343, br.refill() at :357). Counts: 0.0646 lookups/B, 83% pairs. Measured scanner 1t: 1.23 c/B (7763, run 42011789918), 1.49 (8370C, 42008210159), 418 ps/B (8573C), 234 ps/B (9V45). That is 19-23 cycles a lookup, of which about 15% is
  SHARE: 7-10%, plus 3-4% worker idle tied to the scanner
  RECOV: Scanner's 4t share: 35 ms (10.3%) on 8573C, 31 ms (6.9%) on 7763, 16 ms (9.1%) on 9V45, 36 ms (9.2%) on 8370C. Recoverable: 1t scanner saving of 15-40% times 4t retention of 0.6-1.0. 8573C 3.2-14.1 ms (0.9-4.1%); 7763 0.6-2.8%; 9V45 0.8-3.6%. Point about 2.5%, plus up to about 1% from the i:rd starvation. At 1t: 1.7-5.6%. The cheapest and most certain mechanism on this front.
  FALSIFIER: read_item with the bit state in locals (cold paths taking and returning it by value): a 1t stage-table scanner cut under 10% on CI refutes the store-forward chain (for example, memory renaming already hides it). A scanner cut of 25% or more where 4t CPU does not fall by about the scanner CPU saved refutes the regime. Precondition: check that CI's rustc 1.99.1 emits the same memory-resident buf.

- [INFERRED] 4t wall <- worker CPU/4 <- build co-run cost (1.43-2.25x) <- 0.109 same-byte rows/B x ftab store-forward + 4 B/row of tt stores with line fills
  CLAIM: T-vector build: not a random scatter. It writes 256 sequential bucket streams, and 81.9% of rows go through the 16-row run path. At 1t its cost is a store-forward chain on ftab[c]: the 60.1% of one-row steps that repeat the previous byte reload a cursor stored one row earlier, about 5-6 cycles each, about 0.6 c/B. On top of that come the store throughput and line fills for 4n bytes of tt (3.6 MB a block, larger than every runner's L2). At 4t on Intel the build is the stage most hurt by a busy sibling (2.25x, worse than taking turns), which reads as store-drain/fill-buffer contention. On AMD 7763 it is SMT-friendly (1.43x). Floor: about 0.6-1.0 c/B of store throughput plus line fills, about 45-55% of today.
  EVID: Code: src/bwt.rs:304-312 (the one-row closure: ftab[c] loaded, s+1 stored) and :319 (the run16 test). Local disassembly: per row `mov edi,[rsp+r11+0x408]` ... `mov [rsp+r10*4+0x408],edi`. Census: same-byte rows 22.77M; run16 windows 10.74M. Co-run factors: 2.25x (8370C, run 42008210159) and 1.43x (7763, 42011789918). Build 1t share: 15-20% (8573C 582 ps/B, 7763 682). #169 cut the build stage 3.1% (Xeon 8573C) with no decoder gain (build-run-vectors.md:60-75). At the build's 4t share that predicts only 0.4-0.6% of wall, below resolution: a power null, not evidence that the build does not bind.
  SHARE: 14-20%
  RECOV: Build's 4t share: 69 ms (20.1%) on 8573C, 71 ms (15.6%) on 7763, 26 ms (14.3%) on 9V45, 75 ms (19.0%) on 8370C. Recoverable from breaking the same-byte chain (taking runs of 2-15 equal bytes with one cursor step, or interleaving 2-4 row ranges with cursors from mtf_checkpoints) plus run16 vector stores: 1t build saving of 10-25% times retention of 0.3-1.0 (low on Intel, where the co-run build is drain-bound). 8573C 2.1-17 ms (0.6-5.0%); 7763 0.5-3.9%; 9V45 0.4-3.6%. Point about 2%. At 1t: 1.5-5.0%.
  FALSIFIER: A chain-breaking build that cuts the 1t build stage by 15% or more but moves 4t wall by under 0.5% on an Intel runner shows that the 4t build is drain-bound and the chain does not matter there. If the 1t build stage falls under 8%, the forward-chain reading is wrong.

- [OBSERVED] 4t wall <- worker CPU/4 <- (n read + n write) + (n read + m write) + m read, all bandwidth-bound and SMT-inflated
  CLAIM: Post-walk passes (segment assembly, RLE1, CRC) are not fused on master: three passes, 3n + 2m bytes. They have no meaningful chain. RLE1 here is a 32-wide compare plus a copy (12,029 runs in 209.7M bytes), and the CRC is 4 clmul fold lanes. All three are bandwidth- or line-fill-bound, and slowed by a busy sibling: RLE1 2.08x/1.53x, assembly 1.77x/2.97x, CRC 1.59x/1.41x (8370C/7763). The fusion was built and measured (#183). Its 4t effect is small but real on one Intel run.
  EVID: Code: src/lib.rs:349-353 (unrle, then crc32 re-reading out); fused-postwalk.md:9-16. #183 runs (fused-postwalk.md:138-143): PR run 1 (EPYC 7763) 4t -4.2% (-10.0,+1.8); PR run 2 (EPYC 9V45) +1.5% (-3.9,+7.2); fusion alone (Xeon 8370C) 4t -1.5% (-2.4,-0.6); 1t -1.1% to -2.5%, beyond noise in all of them.
  SHARE: 6.4-9.3%
  RECOV: Post-walk 4t share: 23 ms (6.6%) on 8573C, 42 ms (9.3%) on 7763 (assembly inflates 2.97x there), 11.5 ms (6.4%) on 9V45. Recoverable: 0 to about 55% of that share, i.e. 0-3.7% on 8573C and 0-5.1% on 7763. Measured pooled point about -1.4% (the three runs: -4.2, +1.5, -1.5).
  FALSIFIER: Pooled over 3 or more fresh 4t runs of the fused head, a point estimate that is not faster.

- [OBSERVED] stage co-run factor <- which per-core resource the stage uses (issue slots vs fill buffers/store drain) x what the sibling uses
  CLAIM: SMT co-residency differs by stage and by microarchitecture, and no compute chain gets a free ride beside a busy sibling. On Intel (8370C) the walk tolerates a sibling best (1.46x), while the build and RLE1 do worse than taking turns (2.25x, 2.08x). On AMD (7763) the walk is the stage most slowed (2.12x) and the build, MTF and scanner tolerate a sibling (1.33-1.43x). AMD Zen 4/5 walk-beside-walk is 2.03x/2.29x. So on AMD the walk is slowed about 2x whatever the sibling runs.
  EVID: Runs 42008210159 (8370C) and 42011789918 (7763), each stage's fastest of 5 passes, alone and as 4 pinned copies. Walk-beside-walk: runs 41989596336 (EPYC 9V74: 4.27 to 8.65 c/B) and 41989592674 (EPYC 9V45: 2.89 to 6.63 c/B), coresident-walk/ files. The plain single-chain walk inflates only 1.5-1.6x (8370C) and 1.17x (7763) in the same 4-copy runs.
  SHARE: n/a
  RECOV: No direct saving. It sets the retention factors used above and the price of pipelining (next entry).
  FALSIFIER: A repeat 4-copy run on the same CPU models with factors moving by more than about 0.2 (for example the build under 1.8x on an 8370C) would show the factors are run-to-run noise or pairing-phase artefacts.

- [INFERRED] 4t wall <- sum over pairs of cost(X | sibling Y) <- pairing policy
  CLAIM: Pipelining stages of different blocks on SMT siblings (the walk on one sibling, MTF/build/Huffman of another block on the other) is priced but unproven, and its sign is unknown. Today pairing is random; the walk is about 50% of worker co-run time, so the two halves would balance. On AMD there is no headroom for the walk, which is slowed about 2x beside any sibling. On Intel the upside is keeping the build away from walking siblings: at most the build's excess over about 1.5x, i.e. 69 ms x (1 - 1.5/2.25) = 23 ms. The cost is walks beside walks, unmeasured on today's kernel on Intel, which could cancel it. It is also a large structural change: stage-split workers pinned to siblings and topology detection.
  EVID: Factors from the entry above; worker mix from run 41980640481's thread split; walk-beside-walk on AMD only (runs 41989592674 and 41989596336).
  SHARE: n/a
  RECOV: AMD: about 0 (-2 to +2%). Intel: -3% to +6.7%. Not rankable until a pair matrix exists.
  FALSIFIER: A pair-matrix measurement in which the build beside an MTF or Huffman sibling costs about as much as beside a walking sibling (within about 0.2x) on an Intel runner removes the whole upside.

- NOGO #169: the build's run-16 entries as SSE2 stores: of=closed as not shown (docs/journal/2026-11-05-build-run-vectors.md:60-75); the journal's reading was that the scattered writes bind the build
  changed: The build is 14-20% of 4t wall, so a 3.1% build cut predicts only 0.4-0.6% of wall, below the bench's 4t resolution of about 2-5%. The null was power, not evidence the build does not bind. The larger build term is the same-byte ftab store-forward chain (60.1% of one-row rows), which #169 did not touch.
  now: Alone: still not shown. Combined with a same-byte-chain break: build stage -15 to -25% at 1t, 4t wall -0.4 to -5% (low end on Intel, where the co-run build is drain-bound).
  test: One PR with both changes. Gate on the 1t stage table first (build -10% or more), then pool 2-3 runs at 4t, naming the CPUs.
  score: 3 (point about 2% of 4t wall, medium confidence)

- NOGO #183: RLE1 straight from the walk's segments, the CRC folded in: of=closed: its registration required a 4t wall gain beyond noise on wikidata in a PR run (docs/journal/2026-11-05-fused-postwalk.md:133-153)
  changed: Post-walk is only 6.4-9.3% of 4t wall, so the expected gain was 1-3%, below the registered power. The fusion-alone dispatch did resolve 4t -1.5% (-2.4,-0.6) on Xeon 8370C, and every run was faster beyond noise at 1t.
  now: 1t -1.5 +/- 1%; 4t about -1.4% pooled (resolvable only by pooling 3 or more runs).
  test: Re-register on fresh heads: 1t wall as primary, 4t pooled over 3 runs, with the no-harm clause given an explicit settle-run rule.
  score: 4 (point about 1.4%, high confidence in sign, small)

- NOGO A bigger or wider Huffman table (more symbols a lookup): of=never tried. research-synthesis.md:104 rejected Huffman work because 'the whole scanner is 5.4%'; external-report-2.md:35-44 warns bigger tables can be slower
  changed: The scanner is 11-14% of 1t and 7-10% of 4t wall today. Its chain is dominated by the in-memory bit buffer (a store-forward each lookup), not by table width: 83% of lookups already decode a pair.
  now: Do the register-held bit state first: scanner -15 to -40%. A wider table afterwards adds at most about 10% more to the scanner (under 1% of 4t wall).
  test: The bit-state PR, read on the 1t stage table and the 4t CPU split. Revisit table width only if the scanner stays above 1.0 c/B.
  score: 2 (via the bit-buffer fix: point about 2.5% + about 1% i:rd, high confidence in mechanism, cheap)

- NOGO Co-residency-keyed dispatch (coresident-walk: 16 lanes / L2 prefetch when workers outnumber cores): of=not reproduced on today's kernel (docs/journal/2026-11-05-coresident-walk.md:72-98)
  changed: Front D adds that the sibling's stage matters per microarchitecture. On Intel it is the build (2.25x), not the walk (1.46x), that suffers. On AMD it is the walk (2.12x), whatever the sibling runs. A dispatch should target the stage pairing, not the walk's shape.
  now: Walk-shape dispatch: still no headroom. Stage-pairing dispatch: AMD about 0, Intel -3% to +6.7%, sign unknown.
  test: A pair-matrix tool: two threads pinned to sibling CPUs, stage X looping over real blocks beside stage Y. Run on one Intel and one AMD runner.
  score: 5 (unknown sign; large build cost)

- NOGO The scanner as a serial bottleneck at 4 threads: of=split-condvars journal (docs/journal/2026-11-05-split-condvars.md:150-167): the scanner idles about 70% of the 4t wall waiting for slots (local VM)
  changed: Not a bottleneck, as found. But on saturated CI vCPUs its CPU (0.12-0.14 s at 4t) still costs wall at 1/4 rate. Workers also idle 3-4% of their time while the scanner reads a block (i:rd, occupancy runs 41976148952 and 42009406264).
  now: Making the scanner faster pays at 1t and 4t in proportion to its CPU, with no serial-stage effect beyond the i:rd share.
  test: The bit-state PR: 4t CPU should fall by about the scanner CPU saved, and i:rd should fall in an occupancy run on its head.
  score: folded into rank 2

DISPATCHED: sweep.yml, tool=phases, rounds=5, on master 4bfaf7cb (decoder identical to d1fd4405): run 42011789918, completed successfully. Stated beforehand, it was to decide whether the 8370C's per-stage co-run factors (build 2.25x, walk 1.46x, MTF 1.54x) hold on a second runner model. It landed on an AMD EPYC 7763 (3.2408 GHz reported). Result: they do not hold in kind. On the 7763 the walk is 2.12x, the build 1.43x, MTF 1.33x, the scanner 1.43x and assembly 2.97x. Non-walk stages remain 45-50% of 4t stage CPU on both models. The build is still the largest non-walk 4t term on both (17.7% against MTF's 15.0% on the 7763), but only narrowly on AMD. MTF's serial chain is slowed by a busy sibling on both models (1.33-1.54x). This front dispatched nothing else. It also read run 42008210159, the earlier phases sweep at b7734313 (Xeon 8370C), which another front dispatched.
UNKNOWNS: - No hardware counters on CI. The store-forward chains (MTF list, Huffman bit buffer, build cursor) and the build's store-drain reading are derived from disassembly plus counts plus measured cycles; they are not counted.
- The disassembly comes from local rustc 1.97.0. CI builds with 1.99.1, and the memory-resident Huffman buf is not yet confirmed on CI's codegen.
- Golden Cove-class cores (Xeon 8573C) may rename memory and hide the bit-buffer forward, so the Huffman gain could be smallest there.
- 4-copy co-run factors exist only for Xeon 8370C and EPYC 7763. Applying them to the 8573C (Intel family) and the 9V45 (AMD family) is a cross-model inference. There is no stage factor for the 9V74.
- The 4-copy harness pairs each stage with a sibling at an uncontrolled phase: copies drift, other-walk passes interleave, and taking each stage's fastest of 5 passes favours gentler pairings. The factors are therefore 'beside a mixed sibling', not the decoder's exact mix.
- The decoder's 4t CPU exceeds the 4-copy stage sum by about 10% (8370C) to about 50% (7763). Part of that is VM-to-VM variance: on the 7763 VMs the 1t walk differed by 14.6% between runs 41986912287 and 42011789918. Part is oversubscription, preemption and the harness's reader and hash. None of it is attributed to stages.
- The stage-pair matrix (X beside Y) is unmeasured, so the sign of pipelining is unknown.
- The MTF construction space belongs to the other workflow; the 30-55% MTF saving range here is a chain-model bound, not a tested construction.
- The census overcounts Huffman lookups by about 2.7% (each window's partial last block).
- i9: at 4 threads there is no SMT co-run, so its stage mix is like CI's 2t. Its many-copy figures are DRAM contention and were not used for CI.
NEXT: 1. Decisive and cheap: a PR that keeps read_item's bit state in locals across the symbol loop. Bits::refill_slow and Huff::decode_entry_slow would take and return the state by value (or the loop writes back and reloads around them). No format change; the existing table_decodes_as_libbzip2 and stream tests cover it. First confirm with objdump that CI's rustc 1.99.1 keeps buf in memory today. Register before timing:
   - scanner stage (1t stage table) -15 to -40%;
   - wikidata 1t wall -1.7 to -4.6%;
   - 4t wall -0.6 to -4.1%, with 4t CPU falling by about the scanner CPU saved (that also tests the regime rule).
   Falsifiers as in the Huffman entry. Name each run's CPU.
2. Fill in the per-stage co-run factors with a sweep.yml tool=phases run that lands on a Xeon 8573C and one on an EPYC 9V45 or 9V74. This replaces the cross-model inference for the build's and MTF's 4t shares.
3. Only if stage pipelining is to be pursued: a pair-matrix example (two threads pinned to sibling CPUs, stage X beside stage Y over the real blocks), added to sweep.yml as a new tool and run on one Intel and one AMD runner.

#### FALSIFIER

- FALLS: Regime: the 4t wall is total co-run CPU / 4. A non-walk stage's recoverable 4t wall is its co-run CPU saved / 4. Non-walk stages are 45-54% of 4t wall on every runner.
  WHY: The aggregate part holds (OBSERVED). Utilisation is 0.919-0.970 (8573C 41980640481 0.959; 8370C 41986908473 0.970; 9V45 41986885355 attempt 2 0.919). Where both resolved, 4t wall moved with 4t CPU: #183 fusion alone on the 8370C, wall -1.5% against CPU -1.9%; #181 run 3 (41979947814) on the 9V45, -3.2% against -3.5%. The occupancy runs also back 'no serial stage binds': the scanner is in a slot wait for 40-47% of the span and the reader is starved for 77-79%.

The per-stage rule fails, for three reasons.

(a) A stage's co-run factor is not fixed when its own code changes. #171 changed only the walk. On the 9V45 it cut 4t CPU by 17.6% (run 41952512753 attempt 1) and 16.3% (41956321455 attempt 2), and amplification fell from 2.23x to 1.84x. At 1t the same runs moved -0.1% and +0.4%, and the walk stage +0.3% and +3.4%. So co-run cost can move with no change in alone time, and '1t saving x f
  CORR: The 4t wall tracks total 4t CPU (OBSERVED). A stage's recoverable 4t wall is its own co-run saving plus an unmeasured change in what its sibling costs; for a footprint change (#171) the second term was the larger. The non-walk share is 42-56% of 4t wall depending on the factor set and method (INFERRED). On a clean 7763 the ms figures are about 30% smaller than the front's (0.314 s vs 0.453 s wall).

- FALLS: MTF (priced only): a store-forward chain of about 20-24 cycles a move. 4t recoverable 2.4-9.4%, with retention 0.6-1.0; 'a latency chain does not get the sibling's idle slots for free'.
  WHY: Nothing measures the 4t retention floor of 0.6. On AMD, MTF has the lowest co-run factor of any stage: 1.33x on the 7763 (42011789918) and 1.23x on the 9V74 (42020095581, a run the front did not have). So on AMD it largely shares the core with its sibling.

Across stages on AMD, latency-bound code has the low factors: the plain walk 1.18x on the 7763 and 1.28x on the 9V74, MTF 1.23-1.33x. Throughput- and memory-bound code has the high ones: the room walk 2.12-2.22x, assembly 2.97-3.53x. A chain cut makes the MTF that remains more throughput-bound, which should push its factor toward the decoder's average of 1.71-1.79x. Derivation (INFERRED): a 30% alone cut with the rest of MTF at 1.71x is 0.70 x 1.71 / 1.33 = 0.90 of today's MTF co-run time, a 10% saving. That is a retention of about 0.34, and it leaves out any added cost to the sibling.

The 95M cycles subtracted as overhead are assert
  CORR: 8573C: 3.1-9.4%, if the 8370C factor applies. AMD (7763): about 1.3-7.3%, from 13.2% x 30% x 0.34 up to 13.2% x 55% x 1.0. Point estimate about 3-4%, not 5-6%.

- FALLS: Huffman bit buffer in memory: 0.6-4.1% of 4t wall, plus part of a 3-4% scanner-starvation idle (i:rd). 'The cheapest and most certain mechanism on this front.'
  WHY: The scanner's co-run cost is the best-pinned 4t term on the front, because it runs on its own thread (OBSERVED). Its 4t CPU against its 1t stage time:
- 9V45: 0.062 s against 0.049 s, 1.27x.
- 7763 (noisy run): 0.120 s against 0.081 s, 1.49x.
- 8573C: 0.135 s against 0.088 s, 1.54x.
- 8370C: 0.143 s against 0.087 s, 1.64x.

The starvation share is overstated. The occupancy JSONs split worker idle two ways: {no job, job queued} and {scanner reading, input, slot, outside}.
- 'i:rd' (4.15% on the 9V74, 3.45% on the 7763, timeline mode) includes time when a job was queued but the worker had not yet taken it (jobs_queued 2.43% and 1.68%).
- In parallel.rs:418-426 a worker holds the jobs Mutex while blocked in recv. So w.lock (3.49% and 3.00%) is mostly idle workers queued behind another idle worker.
- Workers idle with no job while the scanner reads come to 1.4-3.3% across the timeline and fu
  CORR: 0.5-4.1% of 4t wall (low end on the 9V45: 9.1% x 15% x 0.35), plus at most about 0.9% from starvation.

- FALLS: T-vector build: the same-byte cursor chain at 1t. At 4t on Intel the build is 2.25x, 'worse than taking turns' and drain-bound (retention 0.3). Build share 14-20% of 4t wall; recoverable 0.4-5.0%.
  WHY: 'Worse than taking turns' needs the sibling's slowdown during the build, which is unmeasured. The pair's throughput is 0.44 plus the sibling's speed. It is below 1 only if the sibling is also slowed 1.8x or more, which holds only if the 4-copy siblings ran build beside build, and the front itself calls that phase uncontrolled.

The 2.25x comes from one 8370C run, with copies at 2.07-2.33x, applied across models to the 8573C. Within AMD the build's factor moves by 0.33: 1.43x on the 7763 against 1.76x on the 9V74 (42020095581). That is beyond the front's own falsifier threshold of 0.2.

The 8573C share (20.1%) is not pinned down by the totals: 13.4% fits equally well (see the regime verdict). 'Drain-bound' is a reading of one factor, with no counter behind it.

The build scatters into 256 store streams (docs/ledger.md:20). That is the same pattern #171 removed from the walk, the change th
  CORR: Build 4t share: 13-20% on the 8573C (not pinned down), 14-17% on AMD. Recoverable from breaking the chain: 0.2-5%. A footprint (staging) change is unpriced and of unknown sign.

- SURVIVES: Post-walk passes (assembly, RLE1, CRC) unfused on master; #183 4t effect small but real on one Intel run; pooled about -1.4%.
  WHY: OBSERVED, and the runs are cited correctly. On the 8370C the fusion alone transferred at 0.65 of its 1t gain in wall and 0.83 in CPU (1t -2.3%, 4t wall -1.5%, CPU -1.9%). That matches the stage's own share, with no extra relief for the sibling showing. Two corrections: the 7763's 42 ms (9.3%) comes from the noisy VM, and assembly's factor varies from 1.77x to 3.53x by model.
  CORR: Unchanged: about 1.4% pooled, 0-5% bound.

- FALLS: SMT co-residency differs by stage and microarchitecture; no compute chain gets a free ride beside a busy sibling; on Intel the walk tolerates a sibling best (1.46x); on AMD the walk is slowed about 2x whatever the sibling runs (walk-beside-walk 2.03x/2.29x).
  WHY: The factor values themselves are right; I reproduced them from the artifacts. The conclusions drawn from them fail.

(a) The runs labelled 'walk-beside-walk' (41989596336 on the 9V74, 41989592674 on the 9V45) come from examples/sweep.rs (0769b52f, lines 150-160). There every copy runs MTF, the build, the timed walk and RLE1 for each block, so the walk's sibling is a mix of stages at an uncontrolled phase, just as in phases. No run separates walk beside walk from walk beside another stage. 'Whatever the sibling runs' is therefore unsupported.

(b) 'No compute chain gets a free ride': MTF is 1.23x on the 9V74, and the plain single-chain walk is 1.18x on the 7763 and 1.28x on the 9V74.

(c) 'On Intel the walk tolerates a sibling best': the walk timed as one is 1.61x in the same 8370C run (42008210159; copies 1.57-1.65x), against 1.46x timed in parts. That gap comes from taking the fastest o
  CORR: Not applicable. The per-run factors are observed, each 'beside a mixed sibling at uncontrolled phase'. Which sibling causes a slowdown is unmeasured on every model.

- FALLS: Pipelining stages on SMT siblings: AMD about 0 (-2 to +2%), because the walk is slowed about 2x beside any sibling; Intel -3% to +6.7%; the walk is about 50% of worker co-run time, so the halves balance.
  WHY: The AMD bound rests on claim 6's sibling-independence, which nothing measures.

The walk is 55-60% of worker co-run time, not about 50%: 54.6% on the 8370C, 58.5% on the 7763, 59.7% on the 9V74 (from the phases 4-copy tables). So the two halves do not balance.

INFERRED model, 7763 phases VM, 2 files a core: walk alone 4.33 c/B, non-walk 5.91 c/B including the scanner, non-walk beside non-walk 1.38x. An ideal pairing of walk against non-walk gives:
- walk beside non-walk 1.2x, non-walk beside walk 1.4x: core time 13.4 against 17.46 today, -23%;
- 2.0x and 1.4x: 16.9, -3%;
- 2.2x and 1.6x: 19.0, +9%.

The Intel +6.7% uses the cross-model build share. It also assumes the build's excess comes from walking siblings, which is unmeasured. Only the front's conclusion, 'not rankable until a pair matrix exists', survives.
  CORR: Sign and size unknown on both vendors. On AMD the bound is about -23% to +9% (INFERRED), not about 0.

MISSED: 1. A third phases run exists that the front did not have: 42020095581 (EPYC 9V74, 4bfaf7cb, another front's dispatch). Its factors:
   - scanner 1.32x, MTF 1.23x, build 1.76x;
   - walk 2.22x (2.16x without assembly);
   - RLE1 1.86x, CRC 1.37x, assembly 3.53x;
   - total 1.79x.
   AMD factors therefore differ by model by more than 0.2.

2. The largest 4t lever so far was a change in co-run cost, not in alone time. #171 on the 9V45: 4t CPU -17.6% and -16.3%, wall -14.4% and -13.9%, with 1t flat. The 7763 runs gained 10-13% at 4t, alongside 7-13% at 1t; the 8573C 5.2% at 4t and 4.7% at 1t. That is pre-#180, but CPU measures co-run cost directly.
   - The front's whole ranking assumes fixed factors.
   - It never prices cuts to the high factors in non-walk stages: the build's 256-stream scatter (2.25x on the 8370C, 1.76x on the 9V74; the store pattern #171 removed from the walk), RLE1 (1.53-2.08x) and assembly (1.77-3.53x).
   - The journals record no staged-build experiment.

3. The job handoff is serialised. In src/parallel.rs:418-426 (d1fd4405) a worker holds the jobs Mutex<Receiver> while blocked in recv.
   - At 4t: w.lock 3.0-3.5% and 'idle with a job queued' 1.7-2.4% of worker time (occupancy 42009406264 and 41976148952, timeline mode).
   - At 2t: w.lock 0.8-0.9% and job-queued idle 0.06%.
   - split-condvars measured the condvar, not this Mutex, and on a non-SMT local VM.
   - Ceiling for all worker idle (4.1-5.7% of worker time) is about 2-3% of 4t wall (INFERRED). The ratio comes from #180: 18 points more worker-busy bought 8-10% of wall.

4. Each block at 4t is preempted about once: 0.82-0.99 timeslices and 0.27-0.35 ms in the run queue, 12.5-13.4% of decode elapsed (full mode). So 'decoding 92-93%' means about 80-81% of the span actually on CPU decoding. The refill cost after each preemption is spread over the stages without being named.

5. SMT gives only 1.13-1.24x from 2t to 4t: 8573C 1.244, 8370C 1.162, 9V45 1.126, noisy 7763 1.128.
   - The 4t wall is the two-core wall divided by that small gain.
   - A stage's 4t share differs from its 1t share only through its own SMT gain, which the totals cannot identify.

6. A mix-shift term the additive model omits. Shrinking non-walk work makes a walk's sibling more often another walk. If a walk beside a walk is slowed more than a walk beside another stage, that offsets part of every non-walk saving: plausibly 0-30% on AMD, unmeasured (INFERRED).

7. Inputs and method.
   - The 7763 figures come from the noisy run 41986912287; use #183 run 1 (41986885355 attempt 1) instead.
   - Taking each stage's fastest of 5 passes separately biases the per-part 4-copy factors low on Intel, by about 0.15 for the walk.
   - The front's i9 description checks out: plan_with(4, 10 cores) is Workers(4).
