#### FRONT accounting
MACHINE: Every run is a GitHub-hosted Azure VM: 4 vCPUs on 2 physical cores, SMT siblings 0-1 | 2-3 (env.json core_cpus), no CPU quota. The bench pins 1t to CPU 1, runs 2t on CPUs 0,2 (one a core) and runs 4t on all four CPUs, sharing siblings.

| run | CPU | clock | what was used |
|---|---|---|---|
| master bench 41980640481 | INTEL XEON PLATINUM 8573C | 3001 MHz reported | arm mibz2 = d1fd4405 |
| 41986885355 (#183 PR) | AMD EPYC 9V45 | 4531.6 MHz | the downloadable artifact is attempt 2's; attempt 1 was the 7763 the journal lists; base arm 0769b52f |
| 41988588303 (#183 PR, docs-only push 9bee6b40) | AMD EPYC 7763 | 3230.8 MHz | base arm 0769b52f |
| 41986912287 | EPYC 7763 | | base arm; noisy, excluded |
| 41979947814 | EPYC 9V45 | 4027.3 MHz | head arm e3ea69f, the same src as d1fd4405 |
| 41986908473 | Xeon 8370C | 3491 MHz | refactor arm 5761a32; master plus a Step refactor of RLE1 |
| occupancy 41976148952 | EPYC 7763 | | c90b3625: #180's window, before #181's walk change |
| occupancy 41986885416 | 8370C | | fused-postwalk branch |
| occupancy 41988588220 | 7763 | | fused-postwalk branch |
| walk sweep 41989592674 | EPYC 9V45 | 4.19 GHz | 0769b52f |
| walk sweep 41989596336 | EPYC 9V74 | 3.69 GHz | 0769b52f |
| my dispatch 42008210159 | Xeon 8370C | 3.58 GHz | master b7734313 |

Caveats on the measurements:
- No PMU on any runner, so mechanisms are inferred throughout.
- pmu.yml has never run on master; its one run is 40857395037, PR ci/pmu-probe.
- The bench's 1t arm is the

- [OBSERVED] examples/ab.rs runs.jsonl cpu/wall; bench/report.py:788-850 model(): overlap 1.25-1.46
  CLAIM: At 2 and 4 threads the decoder is limited by CPU capacity, not by a serial path. The 4t wall is about the CPU of all threads at co-resident rates divided by (4 x u), and on these runners 4 vCPUs are only about 2.3-2.5 one-thread equivalents. This is the exchange rate every other term uses. On the 8573C, 1 ms of one-thread stage time removed is worth about 0.43 ms of 4t wall with uniform inflation.
  EVID: Utilisation at 4t: 0.961 (8573C, 41980640481), 0.973 (8370C, 41986908473), 0.953 (7763, 41988588303), 0.910 (9V45, 41986885355) and 0.924 (9V45, 41979947814). At 2t: 0.977-0.996. Mean runnable-not-running threads at 4t is 1.07 (runq 0.368 s / wall 0.343 s). Paired evidence: #181's -3.2% 4t wall came with -3.5% 4t CPU (41979947814). #180's -8 to -10% wall came with +4 to +6% CPU because it removed idle (docs/journal/2026-11-05-window.md:100-104).
  SHARE: workers 81-83%, scanner 9.7-10.7%, reader 3%, idle 2.7-9%
  RECOV: frame only: 0 by itself
  FALSIFIER: A change that cuts 4t CPU by X% with idle unchanged, and whose 4t wall does not fall by about X% over two bench runs. Or an occupancy run on master showing vCPUs idle above 10% mid-stream.

- [INFERRED] bench comment amplification table; four-thread-ceiling.md:45-90 (packed against spread, +40%)
  CLAIM: SMT co-residency is what loses 4t scaling: 76-89% of the efficiency loss on a log scale. Idle is 5-14% and threading or cross-core overhead (A2) is 6-14%. The serial start and tail are about 1.5%, an Amdahl ceiling of 3.8x against the observed 2.0-2.34x. The floor is a property of the core. The co-run inflation of compute-bound stages (1.54-1.59x on the 8370C) is per-core SMT yield and cannot be recovered by decoder changes.
  EVID: From OBSERVED CPU and wall per thread count, efficiency = u4/A4. 8573C: 0.961/1.634, of which SMT (A4/A2) is 1.572. 8370C: 0.973/1.757, SMT 1.691. 7763: 0.953/1.746, SMT 1.60. 9V45: 0.910/1.791, SMT 1.672. The 2t->4t throughput gain is 1.24x, 1.16x, 1.20x and 1.13x. Compute-stage inflation in four copies (run 42008210159): scan 1.56x, MTF 1.54x, CRC 1.59x. The start, s.input, is a fixed 3.3-4.4 ms at 1, 2 and 4 threads (41976148952: 0.006x0.729, 0.0098x0.340, 0.0113x0.300 s).
  SHARE: the SMT excess is about 35-40% of 4t wall against a no-SMT-cost ideal
  RECOV: 0 (floor)
  FALSIFIER: A 2-copy, cross-core phases run whose total inflation is near the 4-copy's 1.62x. That would put the loss beyond the core (L3 or memory) rather than in SMT.

- [OBSERVED] bwt::build -> build_one_pass scatter; examples/phases.rs 'T-vector build' row
  CLAIM: The T-vector build is the stage most sensitive to SMT co-residency on the Xeon 8370C: 2.25x its alone time in four copies, against about 1.55x for compute-bound stages and 1.46-1.61x for the staged walk. Its 4t share rises from about 15% to 19-23%. The excess above the compute baseline, about 0.70 x 2.01 c/B out of 21.8 c/B a copy, is the recoverable bound. The pattern fits many concurrent store streams being what the sibling penalises: the build has up to 256 bucket write streams into a 3.6 MB T-vector and as many read streams over ll. The unstaged flagged walk (1.95x) and bitmap walk (2.24x) show the same pattern; the staged room walk does not. #171's staging of the walk's 32 store streams gained 5-14% at 4t.
  EVID: Run 42008210159 (dispatched; master b7734313, Xeon 8370C @ 3.58 GHz). Build alone 2.01 c/B; four copies 4.17, 4.61, 4.62 and 4.68. Walk alone 7.27; four copies 10.50-10.82 split, 11.23-11.75 as one. MTF 2.22 against 3.33-3.50. Scan 1.49 against 2.22-2.44. Total 13.45 against 21.33-22.26. Build code at src/bwt.rs:288-336 at d1fd4405 (tt[ftab[c]] = ll[s] | i<<8, so one write cursor and one read cursor per byte value). The mechanism is INFERRED; there is no PMU.
  SHARE: 19.2% (8370C, from co-run factors); 14.7-20% (8573C, inferred)
  RECOV: 0-6% on the 8370C (about 0-23 ms of 392); unknown on the 8573C and AMD
  FALSIFIER: (a) A phases four-copy run on an AMD runner or the 8573C showing build inflation within 15% of MTF's: then the term is 8370C-specific. (b) A write-combining (staged bucket) build arm whose four-copy build inflation stays at or above 2.0x, or whose bench 4t wall shows no gain in 2 runs.

- [OBSERVED] parallel.rs scanner (lines 346-415) and worker recv (418-430); read.rs feeder CHUNK 4 MiB
  CLAIM: vCPU idle at 4t is 2.7-9% depending on the CPU. About 13-16 ms of wall-equivalent is roughly constant in absolute terms across runners, so it is a larger share on the faster 9V45. Of it: about 20% is the fixed stream start (the scanner waiting for the feeder's first 4 MiB chunk, src/read.rs:13,112-133), about 8% is the tail, and about 70% falls mid-stream. Mid-stream, workers sit idle while the scanner reads the block they need (i:rd 2-4% of worker time), and in the handoff (i:q 1-2%). With 6-7 threads on 4 vCPUs the scanner is preempted: it reads 57% of the span, at 0.39 ms elapsed a block against 0.2 ms of one-thread CPU. The window does not bind.
  EVID: Utilisation from runs.jsonl (see the frame claim). Occupancy 41976148952 (7763, c90b3625, window 16): w.idle 4.9%, w.idle.reading 3.4%, w.idle.jobs_queued 1.7%, s.input 1.1% of span, s.reading 0.566, w.idle_during_slot_wait_s 0.003. The fused branches show the same split: 41986885416 (8370C) i:rd 2%, i:q 1%; 41988588220 (7763) i:rd 4%, i:q 2%. There is no occupancy run of d1fd4405 itself. Splitting the idle into start, middle and tail is INFERRED.
  SHARE: 3.9% (8573C), 2.7% (8370C), 4.7% (7763), 7.6-9.0% (9V45)
  RECOV: lo 0.8% (start only, by a small first chunk); hi 4% (8573C, 7763), 7% (9V45), 1.7% (8370C)
  FALSIFIER: An occupancy dispatch on master showing s.input not fixed across thread counts (so not the start), or mid-stream i:rd+i:q under 1%. An arm that lowers the workers' priority (nice +1..+3) or shrinks the first chunk without moving 4t wall in 2 bench runs.

- [INFERRED] decode_block under the parallel decoder against examples/phases.rs pass()
  CLAIM: There is time nothing accounts for. At 1t, 2-4% of wall is covered neither by the stage table nor by the harness's copy and hash. At 4t, the decoder's CPU on the 8370C exceeds four co-resident sequential decodes by about 8%, of which 3-6% is plausibly phases' fastest-of-5 optimism under co-run, leaving 0-5% unattributed. The decoder's per-block decode CPU at 4t/1t is 1.61-1.75x against the four-copy worker-stage 1.63x: preemption about once a block, cross-core symbol and output handoffs, or a 16-slot footprint are the candidates.
  EVID: 1t residual +40.6 ms (8573C), +16.5 (9V45), +35.4 (7763), +59.5 (8370C). main=0.03-0.04 CPU-s at 4t bounds copy plus hash at about 2.5-3% at the one-thread rate. On the 8370C, decoder 4t CPU 1.521 (41986908473) against a four-copy stage sum of 1.278 s a file (42008210159) x 1.076 (that instance's 1t residual factor) x 1.02 (instance scale) = 1.405. Co-run measurement noise is shown by the walk timed split (10.65 c/B) against as one (11.50 c/B) in the same run. Per-block CPU: occupancy b.cpu_ms 2.258/1.403 (41976148952), 2.27/1.38 (41988588220), 2.94/1.68 (41986885416).
  SHARE: 0-5% of 4t CPU; 2-4% of 1t wall
  RECOV: 0-4%
  FALSIFIER: A same-instance comparison, the occupancy 'full' per-block CPU at 4t against a phases four-copy on one runner in one job, showing them equal within 2%.

- [OBSERVED] bwt::walk_segments_room (staged, 32 lanes, TILE 256, PF_L1)
  CLAIM: The walk is the largest term, about 40-49% of 4t wall, but almost none of it is recoverable within today's representation (32-bit entries, a 3.6 MB T-vector a worker). The 59-setting grid has its optimum at the default, alone and with four copies, on the 9V45 and 9V74. Tile 512 and 1024 failed. The deferred flags cost time. libsais's whole inverse BWT is 1.05x mibz2's. On the 8370C its co-run inflation (1.46-1.61x) is at the compute baseline, so there is no SMT excess to recover on Intel. Its floor is set by dependent loads in the table: 5.3-7.3 cycles a row. Only a smaller table moves it, and that is unmeasured on CI.
  EVID: Sweeps 41989592674 (9V45) and 41989596336 (9V74): no setting faster beyond noise, closest -0.5% to +1.4% (coresident-walk.md:253-262). Tile arms #177/#178 (tile-size.md:183-197). #181 result (staging-only.md:80-99). Kernels table in 41980640481: libsais/mibz2 1.05x (1.04-1.06). Run 42008210159: walk 7.27 against 10.50-10.82 c/B.
  SHARE: 45.3% (8370C); 39.7-44.7% (8573C, inferred)
  RECOV: 0-1.5% (tuning); a representation change is unknown
  FALSIFIER: A phases four-copy on AMD showing the walk at about 2x or more in mixed co-run (the walk-only four copies showed 2.18-2.29x on the 9V45 and 2.02-2.26x on the 9V74). Then an AMD-specific walk co-run excess of up to about 15% of 4t worker CPU exists, with no known lever.

- [OBSERVED] mtf::mtf; stream::Scanner::next; SegScratch::assemble, rle::unrle, crc::crc32
  CLAIM: MTF+RLE2 is a serial chain per block. It inflates at the compute baseline under co-run, so only its one-thread cost is recoverable, and instruction cuts on it have bought time. The scanner (Huffman) takes about 10% of 4t capacity and is not on the critical path, so only its CPU share is recoverable. The post-walk passes (assembly, RLE1, CRC) are about 7% of 4t on the 8370C and are co-run-sensitive there (RLE1 2.08x, assembly 1.77x). That fits #183's fusion showing at 4t only on the 8370C.
  EVID: Run 42008210159: MTF 1.54x, scan 1.56x, RLE1 2.08x, assembly 1.77x. #170 -1.9% at 4t and #172 4t CPU -2.2% (walk-round.md:18-25). The scanner binds only from about 7.9-8.8 worker-equivalents (result.json models, all runs); it is slot-blocked 30% of the span (41976148952). #183 fusion alone: -1.5% (-2.4, -0.6) at 4t on the 8370C (41986908473); not shown on the 7763 or 9V45 (fused-postwalk.md:138-161).
  SHARE: MTF 14.5-18%; scanner 9.7-10.7%; post-walk about 7%
  RECOV: MTF 0-3%; scanner 0-3%; post-walk 0-2% (Intel), about 0 (AMD)
  FALSIFIER: For MTF and the scanner, an arm that cuts the stage's one-thread time by Y% while 4t wall moves by less than Y x share / 2. For the post-walk passes, an AMD phases four-copy with RLE1 under 1.6x would confirm about 0 there.

- [OBSERVED] StreamDecoder::read; scanner slot wait
  CLAIM: The ordered-output reader and the window do not bind. Idle during slot waits at window 16 is 3 worker-ms, so a wider window recovers at most 0.3%. Wasted wakes are about one a block, 0.2-0.4% of CPU. The reader's copy into the caller's buffer, plus the bench's hash, is about 3% of 4t capacity. Only the copy is decoder cost, and only a zero-copy API removes it.
  EVID: Occupancy 41976148952: w.idle_during_slot_wait_s 0.003; reader waiting 91% of span. split-condvars.md:308-325 (local VM count). main=0.03 sampled at 4t (41980640481). parallel.rs:231-280 copies with copy_from_slice.
  SHARE: reader about 3%
  RECOV: window 0-0.3%; condvars 0-0.4%; Read copy 0-1.5%
  FALSIFIER: An occupancy run on master with slot-wait idle above 1%, or a bench run of a zero-copy reader arm that moves 4t wall by more than 2%.

- NOGO #183 fused post-walk pass (RLE1 over segments, CRC folded in): of=closed: not shown at 4t (fused-postwalk.md:133-161)
  changed: The four-copy phases run shows the post-walk passes co-run-sensitive on Intel (RLE1 2.08x, assembly 1.77x on the 8370C, run 42008210159). Its one resolved 4t gain was also on an 8370C (-1.5%, 41986908473).
  now: 4t -1 to -2% on Intel (8370C, 8573C), about 0 on AMD (7763, 9V45). The PR runs landed on AMD.
  test: A phases four-copy on an AMD runner (RLE1 under 1.6x confirms about 0 there), then a third bench run of #183's head that lands on an Intel runner.
  score: medium-low (0-2%)

- NOGO #169 build writes runs as vector stores: of=closed: build stage -3.1%, no decoder gain
  changed: The build's 4t cost is dominated by co-run inflation (2.25x on the 8370C), not instructions.
  now: Still no gain. Cutting instructions does not address the store streams. The new candidate is a write-combining or staged-bucket build, not #169.
  test: None for #169. For the new build arm: phases four-copy first (build inflation must fall below about 1.8x), then a bench A/B.
  score: #169 itself: low

- NOGO Co-resident walk shape (16 lanes, prefetch to L2, when workers outnumber cores): of=closed: not reproduced (coresident-walk.md:237-272)
  changed: On the 8370C the staged walk has the lowest co-run inflation of any stage (1.46-1.61x).
  now: No gain on Intel. On AMD it is open only if a mixed co-run walk inflation of about 2x appears.
  test: Phases four-copy on an AMD runner: walk inflation of 1.8x or more reopens only the question of co-scheduling a walk against a non-walk, not the shape.
  score: low

- NOGO Wider window, byte-budgeted admission, self-scanning workers (Astra ranks 4 and 5), split condition variables: of=window: merged at 4x; condvars: rejected; ranks 4 and 5: bounded
  changed: Nothing changed. Mid-stream idle is in scanner reads and handoffs under oversubscription, not in slot waits.
  now: Window at most 0.3%; condvars 0.2-0.4%. Self-scanning or admission changes are bounded by mid-stream idle: 2-3% of 4t on the 8573C and 7763, up to about 6% on the 9V45.
  test: An occupancy.yml dispatch on master: the i:rd + i:q share at 4t bounds ranks 4 and 5.
  score: low (window, condvars); medium-low (ranks 4 and 5 on the 9V45)

DISPATCHED: One diagnostic workflow, sweep.yml with tool=phases and rounds=5, on master b7734313. Its src, examples, Cargo.* and sweep.yml are identical to d1fd4405, checked with git diff.

- Run 42008210159, dispatched 2026-11-05T10:08:31Z and completed 10:11:27Z, success. Artifact 13406366127.
- Runner: Intel Xeon Platinum 8370C @ 3.58 GHz reported.
- What I said it would decide, before dispatching: the per-stage SMT co-residency inflation (phases alone on CPU 1 against four copies, one per vCPU). That sets how the 4t workers' CPU divides across scan, MTF, build, walk, RLE1 and CRC.
- The rule: if every stage is within ±15% of the total's inflation, the 1t shares stand for the 4t ranking. If a stage differs by 1.3x or more, its 4t share is re-weighted.
- Outcome: total 1.62x. The build is 2.25x, 1.45x the compute-bound stages (scan 1.56x, MTF 1.54x), so the build is re-weighted. Its four-copy share rises from 14.9% to 20.7%. The walk is 1.46x timed split and 1.61x timed as one; its share falls from 54.0% to 48.8%.
- Nothing commented on any PR, and no bench.yml was dispatched.
UNKNOWNS: 1. Per-stage co-run inflation has been measured on one CPU only: Xeon 8370C, one instance, run 42008210159. On the 8573C (the master bench's CPU) and on AMD (7763, 9V45, 9V74) it is unmeasured. On AMD the walk-only four-copy inflation is 2.0-2.3x (sweeps 41989592674 and 41989596336). If that holds in a mixed co-run, the walk carries an AMD-specific excess of up to about 15% of 4t worker CPU, with no known lever, and the AMD ranking changes.

2. Co-run figures carry about ±8% measurement noise: the walk timed split reads 10.65 c/B and timed as one 11.50 c/B in the same run. They are also biased low by fastest-of-5.

3. Idle on today's master has not been split into start, mid-stream and tail. The occupancy runs are c90b3625 (before #181) and the fused branches. The cause of the 9V45's larger idle (7.6-9%) is unknown.

4. 2-4% of 1t wall is unaccounted. At 4t, 0-5% of CPU beyond co-resident stage costs is unattributed (cross-instance comparison).

5. Mechanisms are unidentified: no PMU on any runner. The store-stream reading of the build's 2.25x is a hypothesis.

6. Whether the build's co-run excess can be recovered (write-combining) is untested.

7. On the 9V45, the alternative placement (2 workers, with the scanner and reader on the siblings) is unmeasured. It might come close to 4 workers' wall at about 60% of the CPU (INFERRED from the 2t split).

8. i9-10900K: no run of today's master.
NEXT: Three measurements, in this order:

1. Re-dispatch sweep.yml with tool=phases (rounds 5) on master until it lands on an AMD runner (7763, 9V45 or 9V74); the runner model cannot be chosen. It decides two things:
   - whether the build's 2.25x co-run inflation is Intel-specific;
   - whether the walk's mixed co-run inflation on AMD is near the walk-only 2.0-2.3x.
   Together these set the AMD 4t stage table and whether the build or the walk ranks first there.
2. Dispatch occupancy.yml on master (rounds 6, threads 1,2,4). It splits today's 4t idle into start (s.input), mid-stream (i:rd, i:q) and tail, and gives per-block CPU at 4t/1t to set against the phases four-copy on the same runner model. That bounds the unattributed parallel-structure excess.
3. Then build a write-combining (staged bucket) T-vector build. Gate it on its phases four-copy build inflation falling below about 1.8x, before any bench A/B.

#### FALSIFIER

- SURVIVES: FRAME: at 2t/4t the decoder is CPU-capacity-bound. The 4t wall ≈ thread CPU/(4u). On the 8573C, 1 CPU-s is worth 0.26 s of wall and 1 ms of 1t stage time is worth 0.43 ms of 4t wall.
  WHY: Re-derived from runs.jsonl:
- 8573C (41980640481): u4 0.961, runq/wall 1.079.
- 8370C (41986908473, base arm): 0.973.
- 7763 (41988588303): 0.953.
- 9V45 (41986885355): 0.910.
#181's paired -3.5% CPU with -3.2% wall (41979947814) checks out.

Caveat: the 0.43 ms figure holds only under uniform inflation. The measured per-stage factors span 1.23-2.22x (9V74) and 1.46-2.25x (8370C), so the rate is 0.32-0.58 ms per stage-ms depending on stage and CPU. For idle the exchange rate is not 1:1 (see the idle verdict).
  CORR: frame only

- FALLS: SMT co-residency is 76-89% of the 4t efficiency loss. Its floor is a property of the core, and the 1.54-1.59x compute-stage inflation is unrecoverable SMT yield. Recoverable: 0.
  WHY: The log decomposition's arithmetic is right. The floor is not shown:
1. On the AMD four-copies, the compute passes are 1.23-1.43x but the walk is 2.12x (7763, 42011789918) and 2.22x (9V74, 42020095581). The walk's excess above the compute passes is 37% (7763) and 44% (9V74) of the four-copy's log inflation. It scales with its lines-in-flight demand: the 1-line plain walk is only 1.18x and 1.28x.
2. On the 8370C (42008210159), the plain walk (IPC about 0.1, 1 line in flight) inflates 1.55x, as much as scan and MTF. So '1.54-1.59x = issue-slot SMT yield' is not established; it matches a memory-latency inflation.
3. On the 7763, the project measured CPU per block 14% higher with both cores busy at the same two workers a core (docs/journal/2026-10-27-four-thread-ceiling.md:84-86). That is a beyond-core component inside A4/A2. A4/A2 is SMT plus both cores busy plus 7 threads on 4 vCPUs.
  CORR: The compute-pass part (about 1.23-1.43x on AMD, about 1.55x on the 8370C) is plausibly a floor. The walk's AMD excess above it is a demand-dependent term: lo 0 (no lever measured), hi about 16% of 4t wall on the 7763 (52 ms of 324) and about 18% on the 9V74 (from four-copy shares; no 9V74 decoder bench).

- FALLS: The T-vector build is the stage most sensitive to SMT (2.25x on the 8370C). The mechanism is many concurrent store streams. Recoverable 0-6%, 8370C only; the lever is a write-combining build.
  WHY: 2.25x on the 8370C is OBSERVED, in one run. Front A's own falsifier (a) has now run twice:
- 7763: build 1.43x against MTF 1.33x and scan 1.43x. That is within 15% of MTF, so there is no excess.
- 9V74: build 1.76x against MTF 1.23x. The excess exists, about 4.8% of four-copy CPU.
So the build is neither 'the most sensitive stage' (on AMD the walk and assembly exceed it) nor an '8370C only' term.

The store-stream mechanism is contradicted. RLE1, a single sequential store stream, tracks the build on all three CPUs: 2.08 vs 2.25 (8370C), 1.86 vs 1.76 (9V74), 1.53 vs 1.43 (7763). And the identical 256-stream build is at baseline on the 7763.

What fits is alone IPC (INFERRED, from sonly2-b1.out Ir and the runs' alone c/B): the build and RLE1 run at IPC 2.8-4.9, the highest of the non-walk passes on each CPU, and on each CPU the inflation rises with IPC. Under that reading a write-combining
  CORR: 8370C 0-6% (about 22 ms of 392); 9V74 0-4% (four-copy share); 7763 about 0 (0.3%); 8573C and 9V45 unmeasured. The proposed lever is disfavoured; an instruction-cut lever is untested.

- FALLS: vCPU idle at 4t is 2.7-9%: about 20% start, 8% tail, 70% mid-stream (i:rd, i:q). The window does not bind. Recoverable lo 0.8%, hi 4% (8573C, 7763), 7% (9V45), 1.7% (8370C).
  WHY: The split survives on today's master. Occupancy 42009406264 (4bfaf7cb, EPYC 9V74) at 4t:
- worker idle 4.07%: i:in 0.76%, i:rd 2.65%, i:q 1.21%;
- slot-wait 4.4 worker-ms;
- s.input 3.5 ms at 1t but 1.9 ms at 2t and at 4t, so it is fixed only from 2t up.

The bound does not survive. It treats idle as recoverable 1:1, but Front A's own frame evidence (#180, docs/journal/2026-11-05-window.md:102-104: u up about 15-16% bought -8.1..-10.3% wall with CPU +4.3..+6.1%) gives an idle-to-wall exchange of about 0.62-0.77. Filling an idle sibling slows the busy one.

The 8573C's hi of 4% also exceeds its total idle less the start (3.9% − about 0.8%).

The start term is a per-run fixed cost (about 1.9 ms on the 9V74). It is real in the 0.18-0.39 s bench and about 0 in a release-length decode.
  CORR: lo 0.5-0.8% (the start, about 1:1, bench-only). hi about 3% (8573C), 3.5% (7763), 6% (9V45), 2% (8370C), each including the start.

- FALLS: Time nothing accounts for: 2-4% of 1t wall. At 4t the 8370C's CPU exceeds four co-resident decodes by about 8%, leaving 0-5% unattributed.
  WHY: The 4t figure depends on the route and the CPU.

8370C:
- +8.3% by the 1t route.
- +4.4% by the 2t route on the same instance: 0.8993 CPU-s × 1.62 = 1.457 against 1.521.
- The same run's walk factor is itself ambiguous (split 1.46x vs as one 1.61x). That moves the worker-CPU scale from 1.114 to 1.055. On both AMD runs split and as one agree within 0.5%, so the 8% gap is that run's, not the method's.

7763:
- Four-copy factors (42011789918) × bench stages (41988588303) predict 1026 ms of worker CPU against 1044 observed (×1.017).
- Whole process: +2.4% by the 1t route, −6.5% by the 2t route. So about 0.

9V74: the decoder's per-block CPU at 4t over 1t is 1.71x (42009406264), below the four-copy worker stages' 1.85x.

A concrete candidate Front A missed: every bench run is a fresh process (examples/ab.rs:1, :29). RSS is 25 MB at 1t and 74-78 MB at 4t, which means at least 12.5k extra first
  CORR: 8370C 0-4% (2t route); 7763 and 9V74 about 0. Up to about 0.5-2% of it may be first-touch faults (bench-only).

- FALLS: The walk is the largest term (40-49% of 4t) but almost none of it is recoverable. On the 8370C its 1.46-1.61x is at the compute baseline, so there is no SMT excess on Intel. Its floor is dependent loads, 5.3-7.3 cycles a row.
  WHY: Lines in flight (INFERRED by Little's law, same-run plain-walk probe):
- 8370C: MLP 6.4 alone, 6.9 in four copies. The walk's 1.46x is about the probe's 1.55x, a latency inflation that matches scan and MTF in size but is not issue sharing (walk IPC about 1.4).
- 7763: 9.8 → 5.5. 9V74: 9.3 → 5.5. The probe rises only 1.18x and 1.28x while the room walk is 2.12x and 2.22x, and the flagged and bitmap walks 2.13-2.34x.
So on AMD the walk is bound by lines in flight under SMT, far above the other passes' IPC trend, and it is 52-54% of four-copy time.

The '5.3-7.3 c/row' are just the observed alone costs on the 8573C and 8370C (5.32 and 7.27 c/B), not a floor. With 32 lanes and only about 6-10 lines in flight, dependence is not what binds.

The grid optimum is AMD-only (9V45, 9V74). No lane or prefetch sweep has run on an Intel runner, so the Intel tuning bound is a transfer.
  CORR: Tuning 0-1.5% (AMD by sweep; Intel unmeasured). AMD co-run excess: lo 0, hi about 16% of 4t wall (7763) to about 18% (9V74); no lever measured. Precedent: #171's staging of the walk's stores gave -5..-14% at 4t (docs/journal/2026-11-05-walk-round.md:23).

- FALLS: MTF and the scanner: only their 1t cost is recoverable, and the scanner is off the critical path. The post-walk passes are co-run-sensitive on Intel, which fits #183 showing at 4t only on the 8370C; about 0 on AMD.
  WHY: The MTF part survives: 1.54x (8370C), 1.33x (7763), 1.23x (9V74), the least-inflated pass on AMD.

The scanner is intermittently critical: i:rd is 2.65% of worker time idle while it reads (42009406264). A scanner cut therefore also recovers part of that idle.

The post-walk reasoning fails:
- #183's resolved gains were 1t gains on both vendors (-2.3% 8370C, -2.5% 9V45), and the 8370C's 4t gain (-1.5%) was smaller than its 1t gain. That is the opposite of a co-run signature (docs/journal/2026-11-05-fused-postwalk.md:140-142).
- On AMD the pass the fusion removes, segment assembly, is the most-inflated non-walk pass (2.97x 7763, 3.53x 9V74). Yet #183 at 4t on a 7763 was +1.2% (-1.2, +3.6) (41988588303).
  CORR: MTF 0-3%. Scanner 0-3%, plus up to about 2% from i:rd. Post-walk 0-2% on both vendors, mechanism unknown.

- SURVIVES: Ordered output and the window do not bind (a wider window ≤0.3%). Condvars 0.2-0.4%. The Read copy 0-1.5%.
  WHY: On today's master (42009406264, 9V74), slot-wait idle is 4.4 worker-ms, 0.44% of worker time (0.3% was the pre-#181 7763 figure). That is still under 0.5%. main's 0.02-0.04 s is quantised to 10-ms clock ticks (±0.01 s), which the 0-1.5% bound covers.
  CORR: window ≤0.5%; the rest unchanged

- FALLS: #183 nogo: 4t -1..-2% on Intel, about 0 on AMD. RLE1 under 1.6x on an AMD four-copy would confirm the about-0.
  WHY: The AMD four-copies now exist. RLE1 is 1.53x on the 7763 but 1.86x on the 9V74, and the pass the fusion removes (assembly) is 2.97x and 3.53x there. RLE1 is the wrong indicator. The Intel prediction rests on one 8370C run whose 1t gain exceeded its 4t gain.
  CORR: 4t effect unresolved on both vendors (7763: -4.2% (-10.0, +1.8) and +1.2% (-1.2, +3.6)); a 1t gain of -2.3..-2.5% is observed

- FALLS: #169 nogo: still no gain, because cutting instructions does not address the build's store streams.
  WHY: The build's excess tracks IPC together with RLE1, not store streams (see the build verdict). #169 cut the build's Ir by 8.6% (5.43M → 4.96M, docs/journal/2026-11-05-build-run-vectors.md:34). Neither of its runs landed on a CPU where a build excess has been measured: 8573C -0.5% (-4.2, +3.4), 9V45 -3.4% (-8.2, +1.6) at 4t (build-run-vectors.md:66-67). That is unresolved, not 'no gain'.
  CORR: On the 8370C or 9V74, about 1-2% of 4t wall at most (build share × Ir cut × co-run ratio); untested

- SURVIVES: Co-resident walk shape nogo: no gain on Intel; on AMD it reopens only if a mixed co-run walk inflation of about 2x appears.
  WHY: The reopening condition is met on two AMD models: mixed 2.12x and 2.22x, the same as the walk-only 2.0-2.3x of the 9V45 and 9V74 sweeps. The shape stays closed by those sweeps. Co-scheduling (pairing) is now an open question on AMD.

That the mixed and walk-only figures match hints that the sibling's being active costs, whatever it runs (INFERRED; consistent with AMD's watermarked miss buffers, docs/journal/2026-10-27-four-thread-ceiling.md:61-64). If so, pairing would be worth little.
  CORR: pairing lever unknown; its ceiling is the walk's AMD excess

MISSED: 1. THE LARGEST AMD TERM IS ABSENT FROM THE RANKING. The walk's lines-in-flight co-run excess is now OBSERVED twice, on two AMD models (7763 42011789918, 9V74 42020095581):
- the room walk is 2.12x and 2.22x against compute passes at 1.23-1.43x;
- that is about 52 ms of the 7763's 324 ms 4t wall (16%), from bench 41988588303's stages × the four-copy factors (cross-instance), and about 22.5% of four-copy CPU on the 9V74;
- per-thread walk MLP converges to about 5.5 under co-run on both CPUs, from 9.3-9.8 alone, while the 1-line probe rises only 18-28%.
Front A ranked the build (about 0 on the 7763) first. The ranking has to be per CPU vendor.

2. THE IPC PATTERN. On each CPU the non-walk passes' inflation rises with their alone IPC. The build and RLE1, the two passes at IPC 2.8-4.9, move together on all three CPUs. Instruction cuts in those passes should pay more at 4t than at 1t, which reopens #169-type work on Intel and the 9V74. Additions to them, such as write-combining, should not help.

3. THE IDLE EXCHANGE RATE. #180's own data put the idle-to-wall rate at about 0.62-0.77, so every idle bound is overstated by about 1.3-1.6x.

4. PER-RUN FIXED COSTS ARE BENCH-ONLY. The bench's 0.18-0.39 s walls contain:
- the stream start (s.input 1.9 ms at 2t/4t on the 9V74 master);
- the tail;
- about 0.5-2% of 4t CPU in first-touch page faults (INFERRED from RSS 25 → 74-78 MB with a fresh process per run).
These amortise to about 0 over a 1.83 TB dump. A one-line test: record ru_minflt in ab.rs's child.

5. SEGMENT ASSEMBLY ON AMD. It is the most co-run-inflated pass there (2.97x, 3.53x), a streaming copy competing for the walk's miss capacity. Its excess is about 1.4-2.2% of four-copy CPU, yet #183's removal of it did not show at 4t on a 7763.

6. A2 IS UNRANKED. A2 is the parallel structure's 2t overhead: CPU 4% above the 1t sequential path on Intel, 7-9% on AMD. Part of it is the parallel path itself (+4% at one pinned CPU on the 7763, cross-instance).

7. THE MASTER BENCH CPU. The 8573C has no co-run factors. The three measured CPUs disagree in opposite directions (build 2.25 / 1.76 / 1.43; walk 1.46 / 2.22 / 2.12). Borrowing either set puts the 8573C's 4t walk at 40-53% of wall and its build at 12-20%, so its split is unpinned.

8. NO INTEL WALK GRID. No lane or prefetch sweep has run on an Intel runner; only the tile arms ran, once each on the 8573C.

9. HUGE PAGES UNDER CO-RUN. 'No change' was measured alone, locally (docs/journal/2026-10-27-compute-frontier.md:470). Under SMT the per-core table footprint doubles. It was never measured on CI and is pending the maintainer as an allocator-adjacent change (docs/journal/2026-11-05-walk-round.md:100).

10. THE NEXT DECISIVE MEASUREMENT. A pair matrix: a walk-only copy on CPU 0 beside a non-walk copy on CPU 1, on AMD. It decides whether the walk's AMD excess depends on what the sibling runs. If it does, pairing or a per-core walk token is a lever. If not, only fewer lines per row helps (a smaller table, or less non-load line traffic as #171 did). The workflows can't do this today; sweep.yml runs the same tool on every CPU.
