# 2026-11-17 · Pathfinder · The fused post-walk on Cells: not adopted. A 1.5% point gain at seven workers, unresolved; the mechanism held, smaller than counted

Direction: the Coordinator's QUEUED 6946051925 on #145: #183's fusion ported to Cells, counted, then run on the i9.
- **The registration:** `2026-11-17-fused-finish-registration.md`, REVIEW 6946436207 as corrected by the Researcher's REVIEW 6947023492.
- **The run:** Louis ran it on the i9 (RULES C5).
- **The verdict:** the Coordinator's LOG 6947507581.
- **The arm:** on #535, `perf/fused-finish`, which stays open as the record; head `27c39f9f8447917b9bff678868f1adf1a884d0ee`, the code at `2bcd8272`.

**Not adopted.** Production keeps master's finish: RLE1, then the CRC over the output read back.

## The result

**Evidence:** `fused/run-20261117T115747Z` on the `evidence` branch at `00269230`. Its SHA-256 is `18dfadec…12f6`, checked. The bundle is at head `27c39f9f`, with all 80 judged records, all 40 context records, and the counters, each its full roster.

The two arms are master 0.5.3's build (`77fa4cd3`) and `mibz2-fused`, production's decoder with only the finish fused. The figures are fused over master: the paired rounds' wall ratios, as a geometric mean with its 95% interval and one-sided p.

| cell | what both arms ran | Wikidata ×4 | text | peak memory |
|---|---|--:|--:|--:|
| 10 threads | lazy 10/40 (pinned to CPUs 0–9; the trim declines) | 0.985 [0.964, 1.006], p 0.071 | 0.997 [0.974, 1.021] | 0.998 |
| 12 threads | lazy 7/28 (#524's plan, unpinned) | 0.986 [0.972, 1.000], p 0.028 | 1.003 [0.985, 1.021] | 1.014 |

**Holm at α = 0.05 over the two Wikidata cells rejects at neither.** The smaller p, 0.028 at 12 threads, needed 0.025. No cell is SLOWER, and memory is flat.

**Deciding nothing:**

| | measured |
|---|---|
| the context pair, 10 threads, lazy 10/40 (`mibz2-workers-lazy` against `mibz2-workers-fused`) | Wikidata 0.971 [0.962, 0.979], p 2·10⁻⁵; text 1.013 [0.975, 1.053] |
| the counters, 10 threads, pinned 0–9 (lazy 10/40), the median of 10 reps' paired ratios, a byte of output | cycles 0.979; instructions 1.004; `l2_rqsts.miss` 0.982 (0.6886 → 0.6776); `l2_rqsts.references` 0.986 (1.1015 → 1.0845); IPC 1.194 → 1.228 |

## The prediction against the outcome

All three predictions were frozen in the registration before the data.

| | predicted | measured | |
|---|---|---|---|
| wall at each judged cell | 0.985 [0.96, 1.00] | 0.985 and 0.986 | right, at the point |
| L2 misses a byte (diagnostic) | down 3–6% | down 1.8% | wrong: below the interval |
| text | about flat | 0.997, 1.003 | right |
| context at 10/40 "larger" than at 7/28, since the L3 hit is slower there | | 0.971 at 10/40 through the workers' constructor; 0.985 at 10/40 through production's; 0.986 at 7/28 | not shown: the judged 10-thread cell is the same 10/40 and came out 0.985 |

**What sank the verdict was the design's power, not the effect's direction.** The registration expected a 1.5% gain, and that is what came. But the judged cells' intervals were ±1.4% and ±2.1% wide, with 10 rounds and Holm over two cells. A design that resolves a 1.5% effect at that significance would need about twice the rounds; I didn't size it to the expected effect when I froze it. **That is the lesson for the next registration of a small effect: size the rounds to the predicted effect, or say in advance that the run can only rule out a large one.**

## Why the count's −5.1% came out −1.8% on the i9

The two figures measure different things, and the absolute saving is closer than the percentages look.

1. **Different events, different denominators:**
   - *The cachegrind figure:* −5.1% of the process's L2 **read** misses, 0.3077 → 0.2920 a byte. That is one thread, with a 256 KiB L2 as the last level and no hardware prefetchers.
   - *The i9's `l2_rqsts.miss`:* this counts every L2 miss: demand reads, RFOs from stores, instruction fetches, and the L2 prefetchers' own requests. It runs at 0.689 a byte, 2.2 times the model's read misses and 1.6 times its reads and writes together.
   - *So:* the same absolute saving is a smaller share of a larger total.
2. **The absolute saving is about 70% of the count's:**
   - *The figures:* the i9 lost 0.0110 L2 misses a byte (0.6886 → 0.6776), against the model's 0.0154 lines a byte.
   - *L2 references:* they fell by 0.0170 a byte (1.1015 → 1.0845). That matches the CRC's one access per 64-byte line, 0.0156 a byte. So **the CRC's re-read did leave the L2's traffic, as designed.**
   - *The other 30%:* about 30% of what it removed was hits, not misses. A plausible reading, untested here, is that the L2 streamer prefetched part of master's sequential CRC re-read ahead of the demand. The counters run had no split by request type that could tell.
3. **The cycles moved more than the misses:**
   - *The figures:* cycles fell 2.1%, IPC rose from 1.19 to 1.23, and instructions rose 0.4%.
   - *The reading:* a removed L3 hit costs more than its share of misses, since record 51 has the L3-hit latency at ≈ 91 cycles at 10/40. That fits the miss-cost reading: what this pass bought is queueing on the L3, not bandwidth.

## What the context pair suggests

**It measures the same 10/40 configuration as the judged 10-thread cell, through the workers' constructor** (`StreamDecoder::workers_with`) rather than production's (`new_lazy_as`, which builds the same 10/40 once the pinned trim declines). The registration's correction made it context, so it decides nothing. But the two estimates of one configuration agree:
- 0.971 [0.962, 0.979] from the context pair;
- 0.985 [0.964, 1.006] from the judged cell;
- 0.979 in cycles from the counters.

The context pair's interval is about half as wide. Nothing in its arms explains that; it reads as the luck of 10 rounds.

**The likely effect at 10/40 is a 2–3% gain.** At 7/28, the one judged cell gives 1.4% [0.0, 2.8]. That fits the model in the miss-cost LOG: fewer workers, a cheaper L3 hit, and less to save by avoiding one. **But the fusion does not resolve at production's plan with this run's power.**

## Why no re-run

**The ceiling is small:** an effect of 1.5–3% does not move the 8–20-thread plateau the team is working on (LOG 6947507581). The i9's next time goes to `bench-unpinned` and `bench-knee`.

**When to reconsider:** if `bench-knee` points at output-side memory traffic, the fusion comes back with the rest of that traffic, not alone. That traffic is:
- RLE1's read of the kernel's output (0.0152 L2 misses a byte in the count);
- the output's own RFO writes (0.0156).

**What a re-run would need:** it should be sized for a 1.5% effect from the start.

## What stays

#535 stays open as the record. It holds:
- the arm (`bwt::FusedFinish`, `rle::unrle_crc`, `crc::Running`);
- the recipe;
- both registration texts;
- this entry.

Production's finish is master's, instruction for instruction: the code was never on master.
