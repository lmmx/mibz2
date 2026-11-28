# Record 129: the performance courses read against the scanner profile, through the algorithms; and an analysis plan for stages 1 and 2 (channel #39; #346 comment 7148095764)

*MONODROMY, 25 November 2026. Part 1 §§1–3 were written before I read the other three readings; §4 compares. Part 2 is registered before stage 1's data. Labels: proved / argued / measured (by others, cited) / conjectured. I read no decoder source.*

**Inputs used (measured, journal §57):** scanner 106.6 µs a block, workers 798.3 µs a block, knee 7.5; Huffman decode 81% of scanner work at 1.41–1.51 ns a symbol, 2,183 Mbit/s and 575 M symbols/s, so 3.8 bits a symbol; no cost per table switch; efficiency against the busiest server 0.945–0.998; 932 real blocks and 792 tiny ones.

# Part 1. Where the courses' tools touch the algorithms

## 1. Opportunities

### 1.1 The Huffman scan is a dependent chain at its latency floor; the device is composition of the decode step

- **Tool:** service demand = visits × time a visit (Jain ch. 33); modification analysis (Harchol-Balter ch. 7).
- **Mapping.** One symbol costs 1.41 ns at one worker and 1.51 ns at lower turbo. Both are about 7 cycles. A first-level cache load takes 5 cycles to feed the next address, and a shift and an add take the rest. So the scan is one visit a symbol to a device whose service time is a load's latency, and the chain cannot overlap with itself (argued from the two figures; no counter read).
- **The device.** Lower the visit count, not the visit time: let one lookup decode several symbols. That is the decode automaton composed with itself, which is contraction along composition in `essence.md`'s sense, applied to T4.
- **A floor (proved, counting):** lookups ≥ bits / b for a b-bit window. At 3.8 bits a symbol and b = 10 that is 2.6 symbols a lookup at best. Six tables of 2¹⁰ four-byte entries are 24 kB and stay in first-level cache; a larger window leaves it and the visit time more than doubles.
- **The 50-symbol groups cost nothing here.** The profile's fit finds no time per table switch, and a multi-symbol entry only has to stop at a group boundary, 2 lookups in 50.
- **Predicted size (conjectured):** decode at about 0.75 ns a symbol, so the scanner at about 63 µs a block and the knee at 12.6. On 10 cores with 9 workers the wall a block is then max(63, 798/9 = 89) µs against 107 now: **13–17% off the best wall**, the range allowing for worker inflation at 9.
- **Cheapest test:** cycles and first-level misses a symbol on the scanner thread at k = 1. About 7 cycles and no misses confirms the floor. Under 5 cycles kills it: the chain is not what limits.
- **Changes a default:** yes, the decode kernel.
- **Unknown:** whether multi-symbol entries were tried. The 22 November fused table carried MTF actions too, which is a different and larger entry.

### 1.2 Under pooling, a target's elasticity is its share of total demand

- **Tool:** the bottleneck law and total demand over capacity (Jain ch. 33–34; Harchol-Balter ch. 6–7); flexible servers.
- **Mapping (proved in the fluid model).** With one scanner and k workers, the wall a block is max(s, w/k). With σ scanners and k workers on c cores it is minimised at σ/k = s/w, where it equals (s + w)/c. Only the sum matters then.
- **What it does to the targets.** Today the elasticity of the wall is 1 for the scanner and 0 for the workers above the knee, and the reverse below. Pooled, it is the demand share for each: 0.12 for the scanner and 0.88 for the workers. **T1's measured 15–17% then applies at every count**, not only below the knee.
- **Predicted size (conjectured):** (107 + 798)/10 = 90.5 µs a block against 107: 15% off the best wall, 18–23% on the k = 1 demands. It ignores sibling threads, memory and the serial tail.
- **How it is realised:** each worker scans its own block, which needs block starts found without decoding (record 122 §3).
- **Cheapest test:** arm C extended to k = 8–10 gives the worker side; the scanner alone gives the other.
- **Changes a default:** yes, the topology.

### 1.3 The saturation law, applied inside a worker, prices dependent depth against dependent reads

- **Tool:** the interactive response-time law and its saturation point N* = (Z + D)/D_max (Jain ch. 33); the machine-repairman model.
- **Mapping.** Inside the walk, a lane is a customer. A miss is think time Z (the memory's latency ℓ). The core is the one server, with demand c a step. Throughput in steps rises linearly with the number of lanes L up to L* = (ℓ + c)/c, capped by the core's fill buffers (about ten), and is flat at 1/c beyond.
- **What it says about the referee.** A cut in dependent depth D is worth up to a factor L* in emission time and nothing past it. Past it only a cut in reads Q helps. So Q and D are not two currencies: L* is their exchange rate, and it is a measurable property of the machine and the block.
- **Predicted size (conjectured):** for reads that miss to memory, ℓ/c is tens, so the cap of about ten binds; for reads served by the shared cache, L* is 5–10. If production already runs near L* lanes, every D result on record (114, Neon S8) is worth nothing more and Q is the only lever.
- **Cheapest test:** stalled cycles a miss in emission against the lane count, at k = 1 and at the knee.
- **Changes a default:** possibly the lane count.

### 1.4 Allocation of variation over the distinct real blocks gives the price of a read

- **Tool:** multiple regression and allocation of variation (Jain ch. 14–15, 17).
- **Mapping.** Regress worker stage time on bytes, rows, runs, h₁, m₁₆ and links over the 233 distinct real blocks. The slopes are record 124's unit prices; the shares say which structural quantity drives w.
- **Predicted (conjectured):** the dependent term carries about 15% of worker time at one worker, matching oracle B's 0.83.
- **Changes a default:** no. It ranks the mathematics.

### 1.5 The utilisation law at the bandwidth device

- **Tool:** U = X × D (Jain ch. 33).
- **Mapping.** Bytes moved an output byte follow from the structures' sizes and passes (P and S). At 4.39 GB/s of output the product may be a large fraction of the machine's bandwidth.
- **Status:** conjectured, with no figure. It may explain part of the flat wall from 8 to 14.
- **Cheapest test:** memory-controller counters at 8 and 14.

## 2. What the measurements rule out, on the algorithm side

- **Mathematics on table switching for speed.** No cost a switch (measured). The group structure blocks only a parallel parse inside a block (record 122).
- **Ordering and discipline devices.** At most 5% (measured efficiency).
- **A wall gain from lower worker demand at 8 or more explicit workers,** until the scanner moves. At the shipped cap of 7 workers the workers bind by 114 against 107 µs, so the first 7% of a worker cut passes into the wall and the rest does not (argued).

## 3. Where composition meets a queueing quantity

| composition idea | queueing quantity | law |
|---|---|---|
| contraction of the decode step | scanner demand s, through visits | lookups ≥ bits/b; knee = w/s |
| contraction of the walk (fewer links) | worker demand w, its dependent part | elasticity 1 below the knee, 0.88 pooled |
| shorter dependent depth | concurrency inside a worker | saturation at L* lanes |
| enrichment (a rider removes a pass) | visits to the bandwidth device | U = X × bytes |
| closure (record 119) | the number of streams a schedule needs | streams ≥ σ^step |

# 4. Comparison with the other three readings

- **Agree with all three:** disciplines are worth little; the worker count should be right-sized; the scanner belongs on an undisturbed core; scanner and worker changes must be priced jointly, with the count retuned.
- **Agree with ChatGPT Pro against the lead:** the 0.109 s "machine bound" is a conditional scenario. My pooled figure, 0.147–0.156 s on 10 cores, is ChatGPT's "perfectly flexible sharing" (0.156 s, 18% lower). Same arithmetic, same label.
- **Agree with Neon's arithmetic:** a 25% scanner cut moves the knee to 10.0; two scanners with eight workers give 0.179 s.
- **Disagree, in emphasis:** all three read "above the knee the scanner binds" as a reason to treat T4 as engineering. The 7-cycle figure says the scan's cost is a dependent chain, the same object as the walk, and the same mathematics applies (§1.1).
- **What all three missed:**
  1. the bits-per-lookup floor and the reading of 1.41 and 1.51 ns as one cycle count;
  2. the saturation law inside a worker, which ties D to Q (§1.3);
  3. that under pooling the worker split is the ranking of mathematical targets, so the worker-split oracle becomes the most valuable measurement for the mathematics (§1.2).

# Part 2. Analysis plan for stage 1 and stage 2, registered before the data

## 5. Stage 1 (every count from 1 to 20)

| # | computed | points to mathematics if | points to engineering if |
|---|---|---|---|
| A1 | s(k) and w(k) a block, by class (tiny, Open, other), medians with bootstrap intervals over rounds; wall predicted as max(s(k), w(k)/k) | — | the prediction misses by over 5% at three counts: the two-station model is short |
| A2 | inflation w(k)/w(1) and s(k)/s(1), after dividing out the logged frequency | worker inflation above 10% by the knee: reads are getting dearer with k, so Q and S cuts gain value | flat: workers are independent; pooling comes first |
| A3 | regression of each worker stage's time on bytes, rows, runs, h₁, m₁₆, links, over distinct real blocks, at k = 1 and at the knee | the dependent term carries 15% or more of worker time | under 5%: the corpus is run-dominated and T1's ceiling is not about reads |
| A4 | demand shares of every activity; the pooled figure (Σ demands)/c | the largest worker shares are construction or emission | they are MTF, RLE1, CRC or output |
| A5 | the knee's roundness: the exponent of a soft minimum fitted to the 20 walls | — | a round knee: variability or gates cost something |
| A6 | 1 − efficiency split into admission waits by cause, head-of-line block, starvation | — | any part over 2% |
| A7 | cycles a symbol on the scanner where counters exist | about 7 with no misses: §1.1 stands | under 5 |
| A8 | fit on k ∈ {1, 2, 4, 8, 12, 16, 20}, predict the rest | — | a miss over 5% at three held-out counts: an element is missing |

**Fixed before the data:** medians per distinct block before pooling; intervals by bootstrap over complete rounds; no interval is narrowed by adding runs after looking.

## 6. Stage 2 (`briefs/character-review-decisions.md` §2)

| step | computed | reading |
|---|---|---|
| screen | each coordinate's coefficient against its stage-1 prediction: the time share while its side binds, zero otherwise | a difference over 0.1 marks coupling through a shared resource |
| 3×3 surface | the soft-minimum exponent and the interaction; compared with A5's independent estimate | the two exponents disagreeing by more than their intervals means the pacing loop is not a faithful slowdown |
| gates | effect of the window and the byte threshold | over 2%: admission binds, engineering |
| placement × clock | a sibling-sharing factor and a turbo factor | they define the capacity curve every prediction uses |
| order | simulation error on the permuted windows | over 5%: order matters and the simulation lacks it |
| **memory dose, stream against chase** | each worker stage's response to each | emission or construction responding to *chase* with elasticity over 0.3: latency-bound, so Q and D mathematics. Responding to *stream*: bandwidth-bound, so pass removal and riders. Neither: compute-bound, engineering |
| oracles B and C at the knee | both sides of the kink | B's gain at the knee, where workers still bind, is T1's real size |

## 7. What would make me revisit a rejected idea

The condition in every case: **a change that takes the scanner's control below one half at the operating point** (a faster scan, a second scanner, or pooling).

| idea | why it was set aside | revisit when |
|---|---|---|
| output-order codes at 10 workers (arm B, 0.976) | the scanner masked it | the scanner no longer binds; expect about 0.85 |
| dependent-depth cuts (record 114, Neon S8) | Q did not move | §1.3's test shows production runs under L* lanes |
| the fused Huffman table of 22 November | it slowed the scanner | pooled: only the sum matters, so judge it on total demand |
| Proposition M (helpers share MTF) | not the 19% | A4 shows leader-only stages over 10% of a team's time |
| composed routes at about h₁ (records 119–120) | no route under h₁ | A3 gives the price of a link, and the screening inequality of record 124 can be applied |
| the min-key order | pending M-25 | M-25 |

**Not to be revisited whatever moves:** lazy thin-strip removal (record 120 §3). It failed on the structure of real blocks, not on where the bottleneck sat.


> **Corrected by record 133** (`2026-11-25-corrections-129-132.md`).
