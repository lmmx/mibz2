# Producer plus consumer cost: what three negative results taught the acquisition-first method

*20 November 2026. The Coordinator.*

- **The occasion:** three experiments on the open kernel and its alternatives, each correct and each
  slower. They are NSPC (#652), the fused band wiring F, and the run-granularity queue Q (#654,
  on the phase split #653).
- **What it decides:** a research-method rule for outside proposals (Neon, ChatGPT) and our own.
  It changes nothing in production or in the release policy.

**The finding:** an optimisation that removes information or storage can still add work, because
the work moves to the consumer. Each of the three removed something real and lost on every block.

| Experiment | What it removed | Where the work went | ÷ shipped Open |
|---|---|---|---|
| NSPC (#652) | 3.5–7× fewer dependent walk steps | building the jump table cost about one Open on its own, and the walk's scans another | 1.82–2.38 |
| F, fused bands and wiring (#654) | the `bands` and `e.bottom` arrays and one pass | the fused loop ran 2–8% slower than the two loops it replaced | 1.023–1.044 |
| Q, packed run queues (#654) | queue fill −46–54%, queue memory −42–52% | the scatter splits by run and by band at once, 4.5–6.4× slower | 1.046–1.093 |

- **The common cause:** every edge needs one entry per retained row. Expanding runs into rows is
  required work. Shipped Open does it in a simple sequential fill. Q moved it into the band
  scatter, which is already segmented, where thin runs of 1.6–2.2 rows add a second set of
  boundaries.
- **Separate phases:** F shows that two simple loops can beat one fused loop, even though the
  fused one moves less data.

## The rule

Every proposal targets the minimum complete cost: producing, representing, consuming and retiring
the information. The least information acquired, the smallest representation and the fewest passes
are not targets in themselves.

Alongside its acquisition certificate (no new information), a proposal gives a **consumer-cost
certificate**:
1. **Removed:** which operations disappear, by source line.
2. **Moved:** which operations move, and where to. Relocated work is never counted as removed.
3. **Consumer:** what new branches, segment boundaries, indexed accesses or dependency chains the
   consumer gains.
4. **Simple loops:** whether the proposal complicates a simple sequential loop. Treat any proposal
   that moves work from a simple loop into an irregular or dependency-bound phase as suspect.
5. **Working set:** what happens to the live working set and the register pressure, especially
   when loops are fused.
6. **The total:** why the producer and the consumer together should cost less on a real CPU.
   Instruction counts and the measured stage split are the evidence. A count of operations is not.

**The ceiling:** a measured stage share bounds what eliminating that stage could save. It does not
show that much of the stage can be avoided. Emission's 36–48% includes writing every output byte,
which any decoder must do.

## The thin-run limit, from data already on record

The open-return journal's phase file (`2026-11-18-open-return/evidence/phases-wikidata.csv`, 233
large Wikidata blocks) already measures the limit on the induced, non-queued arm.

| Arm | Median ÷ `i16` | 10th percentile | 90th percentile | Blocks faster than `i16` |
|---|---|---|---|---|
| `i8` | 1.084 | 0.878 | 1.273 | 30% |
| `i4` | 1.160 | 0.914 | 1.415 | 20% |

On the one frozen block, the width sweep has `qi8` level with `qi16`, and `qi24` and `qi32` about 3%
slower. So no single limit beats 16 on these figures. A third of the blocks prefer a smaller limit,
though, so a per-block choice could gain on them. It would need a predictor computed from
information Open already holds, and so it would be a separate experiment, registered first.
