# 2026-11-17 · MONODROMY 51 · The miss-cost run scored. L3-hit latency is about 45 cycles plus a term in the L3-miss reads a byte, so misses slow every L3 hit. The stalls missed by the L2 count are loads waiting on prefetched lines in the output-side passes. The lever that ranks first is #183's fused post-walk, re-measured where contention makes re-reads dear; staggering ranks last

Asked by the Coordinator (10:51), after `miss-cost/run-20261117T104648Z`:
- *the bundle:* on `evidence` at 53012ab5; its SHA-256 ce7ea55c…84ff, rechecked here;
- *the asks:* score record 47, say what the fill-buffer share means, and rank the levers. Ranking
  only, with no prototype.

**Terminology,** as of record 50: contention-associated overhead, per-worker CPU utilisation,
LLC-sharing domains.

**Labels:** **C** counted · **M** modelled · **Conj**.

## 1. Record 47 scored (C)

The counts, 10 → 7 lazy workers at 10 threads, are per byte of output unless stated.

| record 47's prediction | measured | score |
|---|---|---|
| Q: L3-hit latency falls 10–25% ("5–10 cycles of 40–55") | derived from demand-read occupancy: **91.0 → 65.2 cycles, −28%** | direction ✓; magnitude ✗ (outside the range, and the baseline is 91, not 40–55) |
| Q: L3-miss latency falls 5–15% | 594 → 552 cycles, −7% | ✓ |
| D: L3-miss latency falls 25% or more | −7% | ✗ for D |
| DRAM traffic at W = 10, 4–9 GB/s, 10–20% of peak | **not measured**: the IMC was skipped (`perf_event_paranoid` 1). L3-miss demand reads alone are 1.8 GB/s. | untested |
| `stalls_l3_miss` at most 45% of the cycle cut | 2.36 of 5.52 cycles = **43%** | ✓ |
| `stalls_l2_miss` at least 70% of the cut | 2.56 of 5.52 = **46%** | ✗ |
| `mem_load_retired.l3_hit` flat within 5% | 0.908 | ✗ (−9%) |
| `mem_load_retired.l3_miss`, 7 against 10, about 0.52 | 0.390 | ✗ (fewer) |
| 70–85% of the miss cut on the walk's and expansion's loads | the top latency sites are `crc::clmul_update`, `rle::unrle_avx2`, `SegScratch::assemble`, `cells::Kernel::write_cell` | ✗ |
| THP on 7/28: 0.97 [0.93, 1.00], page walks −80% | cycles 1.001; walk cycles −25%. Only 93 THP faults, under `madvise` mode, so the arm barely engaged. | ✗ as a test: the arm was ineffective |

**The refuters.**
- *Q:* not refuted. L3-hit latency fell 28%.
- *D:* not refuted formally, since its bandwidth half wasn't measured. L3-miss latency stays near
  600 cycles at half and at full demand miss traffic, which is no sign of a queue near
  saturation.
- *Taken together:* D is unsupported, and Q is right in kind but wrong in its numbers and in its
  site.

**Record 47's 589-cycle argument, rechecked against measured latency.**
- *The measured miss cost:* an L3 miss costs 595 cycles (121 ns), not the 390–490 I assumed.
  Misses in flight average 2.2 whenever any is out.
- *What the miss count explains:* each exposed miss costs about 595 / 2.2 ≈ 270 cycles. The cut in
  retired L3-miss loads, 0.0048 a byte, then explains about 1.3 of the 5.5 cycles (23%), and
  `stalls_l3_miss` attributes 43%.
- *So the conclusion stands on measured numbers:* the miss count is at most about half the
  contention-associated overhead.

**THP: closed regardless.** Page walks are 0.69–0.75% of cycles at every count, so no THP arm can
gain more than that.

## 2. Why L3 hits slow down (M on C)

**The fit.** L3-hit latency, derived per arm, against L3-miss demand reads a byte (M):

  **L3-hit latency ≈ 44.8 + 3,418 × (L3-miss demand reads a byte)** cycles

- *Residuals:* under 1 cycle at all four arms.
- *The floor:* 44.8 cycles, about an unloaded L3 hit on this core at 4.9 GHz.
- *Against W · W_run, the term records 46 and 49 use:* residuals of 3 cycles, and an implausible
  23-cycle floor.

**The reading.** Each L3 miss also slows the L3 hits around it. Fills and evictions occupy the L3
slices and the ring that hits need. Total read bandwidth to the L3 can't be the driver: offcore data
reads *rose* from 57 to 67 GB/s as the latency fell.

**What the model needs.** The W · W_run term is that relation in disguise: misses grow with the
workers holding footprints, and every access pays for them.

**The cost decomposes, per byte:**
1. *each miss's own exposed latency:* about 270 cycles;
2. *the slowdown it inflicts on L3 hits:* about 3,418 cycles per L3-miss read a byte, over about
   0.05–0.08 L3-hit loads and the prefetched lines.

**Two consequences:**
- *W12's null:* its 7.4% fewer misses at 7 workers lower L3-hit latency by about 1.6 cycles,
  about 1% of cycles with its own misses. That is lost to its +1.6% instructions.
- *Where a lever must act:* on L3 misses, or on the accesses that pay the inflated L3-hit latency.

## 3. What the fill-buffer share means (C, Conj)

**What the 2.7 cycles a byte are (C).** That much of the stall cut sits outside `stalls_l2_miss`,
and 53% of the sampled load latency is LFB/MAB hits:
- loads that find their line already being fetched;
- usually by the L2 streamer;
- that wait for it.

They aren't counted as demand L2 misses. They are the prefetched streams waiting on L3 hits slowed
as in §2.

**Where they are (C).** The sites are the output-side passes:
- the CRC reads the output;
- RLE1 reads the segments and writes the output;
- assembly copies the segments;
- `write_cell` writes the cells.

Each pass streams about 0.9 MB a block, larger than the 256 KiB L2, so each pass is served from
the L3.

**On the Coordinator's candidate, output streams thrashing the fill buffers and the L3 (Conj):**
- *Fill buffers:* not thrashed across workers. They belong to a core. The fill-buffer hits are the
  symptom: each prefetch can't run far enough ahead once an L3 hit costs 91 cycles instead of 45.
- *L3:* yes, in two ways:
  1. the passes pay the inflated L3-hit latency;
  2. their buffers occupy L3 capacity that the walk's random-access structures need, and so add
     to the misses that inflate it. The `seg.bufs` and output buffers are part of each worker's
     16.8 MB of buffers at -9.
- *Is that the W · W_run term?* Yes, as §2's decomposition. The term's two factors are the misses
  (the footprint, the streams included) and the accesses that pay for them.

## 4. The lever ranking (M; kernel count needed for every estimate)

**1. Fused post-walk: RLE1 straight from the walk's segments, with the CRC folded in.**
- *What it is:* #183, `research/fused-postwalk` beef4e8f. It removes the assembly copy and the CRC's
  re-read, two of the four output-side passes.
- *Its history:* NOGO on CI at 4 threads, on power, not mechanism. It gave −1.5% (−2.4, −0.6) on an
  8370C and −1.4% pooled. The census then judged it to survive (nogo.md:349–357).
- *Why it ranks first now:*
  1. it acts on the two top latency sites;
  2. it takes two output-sized streams out of the L3, which by §2 should lower L3-hit latency for
     every access;
  3. at 7/28 on the i9, contention makes each re-read dearer than on CI's runners, where #183 was
     judged.
- *Estimate:* the post-walk passes were 6.4–9.3% of 4-thread CPU on CI, and fusion recovers about
  half. At 7/28 on the i9: **2–5% of wall**, larger at higher W.
- *Caveats:*
  - #183 predates Cells, so the Pathfinder must port it;
  - this is a re-measurement of a measured lever, not a new line.

**2. Non-temporal stores for the final output, only after (1).**
- *What it saves:* the read for ownership and the L3 allocation of output lines.
- *Its risk:* if the reader copies the block to the caller soon after, an NT-stored output is then
  read from DRAM. How soon is the Pathfinder's to check.
- *Estimate:* ±2%, sign unknown.

**3. Staggering the workers' phases (thread 1).**
- *Why it ranks last:* the evidence doesn't point to bursts.
  - L3-hit latency is linear in the average miss traffic (§2), with residuals under a cycle.
  - Misses in flight are flat at 2.2.
  - Staggering helps only if latency is convex in instantaneous load. Nothing here shows that.
- *Estimate:* 0–1%.

**Closed:**
- *THP:* page walks are at most 0.75% of cycles.
- *Footprint cuts above the L2 (W12):* by §2, worth about 1% at W\*.

## 5. Corrections carried

- *Record 47's L3-hit baseline* (40–55 cycles) was assumed. Measured loaded: 91 at 10 workers, 65
  at 7. Unloaded, from the fit, about 45.
- *Record 47's DRAM latency* (80–100 ns) was assumed. Measured: 121 ns.
- *Record 49's c(W) = a + b · W · W_run* stays as the fitted form. §2 names its mechanism as misses
  inflating L3-hit latency.
