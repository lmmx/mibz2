# 2026-11-18 · MONODROMY 75 · Astra A (#599, permanent-domain composition) verified independently. Every frozen and corpus count reproduces exactly. The retirement proof holds. A per-row audit of every frozen installation finds 0 failures. Output and CRC are exact on all 233 blocks. The model's lazy-Cells traffic is consistent with Astra D and the i9 within about 7%, and its saving is about 1 B/B, 12% of measured IMC traffic. The extra serial reads probably offset it at 7–10 workers (I); a one-thread prototype decides first

**The request.** The Coordinator's CHECKPOINT 6993589653: an independent verification of Astra A on #599 at
4ee40ab3, `docs/journal/astra-a-reinvention/`. The other Astra records are merged, so I may read them.

**Labels:** **P** · **C** · **I** · **Conj**.

**Method.** I implemented the permanent-domain algorithm in my model, from `shortcuts.md` alone
(`mixedcheck.rs.txt` beside this record).
- *Domains:* the original domains are the F-images of L's maximal runs, the same pieces as record 72.
  Each holds one word: length, displacement, source, and a finished bit.
- *The walk:* the forward walk from the primary emits a literal or copies from the completed output.
- *Learning:* for each consecutive pair of visits U, V, if U is wide and unfinished:
  - when U's image [a + e, b + e) lies inside V's domain, install the concatenation on all of U;
  - otherwise retire U permanently;
  - if U = V, forget the current visit as an operand.
- *The end:* a final phrase closes the output with no installation.
- *The frozen audit:* at **every** installation, walk every row of U through the composite length,
  comparing each byte with the source span and checking the exit. At every retirement, check that the
  image straddles two original domains.

## Verdicts

| claim | verdict | basis |
|---|---|---|
| The method: a word and exit learned from two completed writes | **EXACT** | P (checked), C |
| The retirement proof (a failed containment settles U permanently) | **EXACT** within the model | P (checked), C: all 6,902 frozen retirements straddle |
| Frozen: 25,283 installations, 6,902 retirements, 117,399 visits | **EXACT** | C |
| Frozen audit (spelling and exit at every entrance) | **EXACT** | C: 1,154,259 installed-domain rows, 11,265,803 bytes, 0 failures |
| Correctness on all 233 large blocks | **EXACT** | C: output = dense walk, and the bzip2 CRC = the block CRC, on all 233. The libbz2 run itself was not rerun |
| Interleaved 20 MiB model: 360,596 against 455,564 lines at 7 workers; 521,084 against 655,508 at 10 | Model arithmetic, consistent with Astra D and the i9 within about 7% (§3). **UNMEASURED** in time | C, I |
| Adverse facts: 4.21× the output spans; more frozen 32 KiB read fills | **EXACT** spans (copies and visits reproduce); fills are the model's | C |
| Every input obtained at the claimed cost | **Yes:** only the capped runs, n and the primary | C |

## 1. The proofs (P)

**Composition.** U's descriptor is valid on all of U, and V's on all of V.
- *When U's image after its word lies in V,* every row of U ends its first word in V. It then spells
  V's word and exits by f.
- *So:* the concatenation is valid on all of U, with displacement e + f. Its source is the two spans
  just written consecutively, which are complete. No byte comparison or probe is needed.

**Retirement.** If the image protrudes from V, it meets at least two original domains.
- *Why that settles it:* by the word/exit theorem (record 72, verified), rows sharing a common exit must
  share their sequence of pieces. Rows entering different domains have different next pieces, so **no**
  longer word with a single exit holds on all of U.
- *Why it is permanent:* domains are immutable and U's word never changes after failure, so the
  obstruction is permanent.
- *What "settled" means:* it is optimal *within the model.* A model that splits U (the study's split
  variant) can learn more, at the cost of routing.

**The singleton theorem.** A word valid on two or more rows translates them together at every step,
so it never passes through a width-1 domain. Every singleton on the marked cycle is therefore an
individual literal visit.
- *Counted:* frozen has 35,116 singleton visits, equal to its 35,116 singleton domains.

**The self-transition rule** (forget the visit when U = V) is necessary. Without it, the next
composition would use a descriptor whose length no longer matches what was emitted.

**Counted.** The construction is exact on all 14,209 small columns, including short periods, with the
per-row audit clean. **The five-row witness** (L = 10010, primary 0) reproduces: the first wide domain
retires, and nothing is installed.

## 2. Frozen and the corpus (C)

| | frozen | all 233 large blocks |
|---|---|---|
| Installations | **25,283** | **6,379,770** |
| … with both words multi-byte | 9,319 | 2,200,612 |
| Retirements | **6,902** | **2,020,860** |
| Visits (output spans) | **117,399** | **34,619,893** |
| Copies | **40,731** | **11,134,412** |
| Copied bytes | **823,263** | **186,212,181** |
| Copies of at most 16 bytes | **30,404** | **8,854,424** |
| Singleton / wide domains | 35,116 / 16,068 | |
| Wide visits | 82,283 | |

- *Every figure equals `whole.csv`.*
- *Dependent metadata reads,* 2p + V = **317,081** on frozen, as stated.
- *Output:* equals the dense walk on all 233 blocks, and the bzip2 CRC of the RLE1 output matches each
  block's stored CRC.
- *My audit:* checked every row of every installed domain on frozen (1,154,259 rows, 11.27 M spelled
  bytes): 0 failures. The study's audit additionally re-spells the retiring words (1.79 M entrances in
  all); mine checks every retirement's straddle instead.

## 3. Calibration and the address ledger (C, I)

**Units.** The model prices the induction and writer only: setup, induction, writer and the inverse
output. It leaves out the entropy front, RLE1, CRC and the final output.

**Its lazy-Cells figure is consistent.**
- *Interleaved, 7 workers:* 455,564 lines × 64 / (7 blocks × ~0.9 MB) ≈ **4.63 B/B**. Ten workers:
  4.66.
- *Astra D's matching rows* (record 74, verified), frozen at 2 MiB: Cells slots, entries, arena,
  rules, positions, commands and inverse output sum to **4.46 B/B**. **Consistent within 4%.**
- *Adding back D's out-of-scope rows* (input, symbols, runs, MTF, final output, other; 2.86 B/B) gives
  about **7.5 B/B**, against the i9's whole-machine **7.97–8.10** at 7 workers. **Consistent within
  about 7%.** The gap is about the consumer's read of the output.

**So the saving, in machine terms:**
- *Size:* 94,968 lines over 7 blocks ≈ **0.97 B/B** (0.96 at 10 workers). That is about **12%** of the
  i9's measured IMC traffic, not the 21% of the in-scope model.
- *Composition:* at frozen 32 KiB, read fills go *up* (198,333 against 152,136), and store fills and
  dirty writebacks go *down* (25,093 against 96,425, and 54,895 against 185,712).
- *Caveat:* record 74 found these models overstate writes against the IMC (the terminal drain, and
  buffer reuse in steady state). The realised saving is likely below about 1 B/B.

**The address ledger.**
- *The candidate:* every visit is a serial chain of row → tile → header (→ record for a wide visit)
  → next row. That is 317,081 dependent loads per frozen block, 0.35 per output byte. **58,485 of
  them miss 256 KiB** (L3 hits). Its 40,731 copies read data-dependent sources, but those reads are
  *off* the row chain, so they can overlap.
- *Lazy Cells,* for comparison: about 95,380 value-addressed slot reads, plus about 38,339 writer
  selections, plus link chases for its 92,570 cuts. Roughly 0.2–0.25 dependent loads per byte (record
  72).
- *So:* the candidate's serial chain is about **1.5–2× Cells'.** The study prices this honestly: a
  1.67-unit premium per dependent L2 miss exhausts the transfer advantage.

## 4. Which way the serial reads cut at 7–10 contended workers (I)

**What binds.** Records 47–53 and 71: what inflates under contention is the latency of dependent loads
at L3 and DRAM. Bandwidth is not binding at 7 workers (20–24.5 GB/s against ~32) and is nearer its
ceiling at 10.

**What the candidate trades.**
- *It gains:* about 1 B/B less DRAM traffic, at most, mostly writes. That helps chiefly through
  less fill-buffer and queue occupancy.
- *It pays:* about 58k extra *serial* L3-hit loads per block on the critical path. L3-hit latency rises
  with contention (44.8 + 3,418 × misses per byte, record 53).

**My reading.** The candidate is plausibly competitive at one thread: it does less resident work in
its scalar model (−7.8% against lazy). At 7–10 contended workers, the serial L3-hit chain probably
**offsets** the traffic saving, so I'd expect roughly neutral against lazy, and slightly better only
at 10, where bandwidth tightens. That direction is the opposite of opend (record 71), whose gain came
from *fewer* dependent loads.

**The cheapest decider, in two stages.**
1. **A native prototype, timed single-thread** on frozen and the ten blocks, in ns per row against
   the lazy kernel. If it isn't at least 5% faster at one thread, stop: contention can only widen a
   serial deficit.
2. **If it passes,** a kernel-only paired run on the i9.
   - *Cells:* unpinned 7/28 and 10/40, 2 s settle, Wikidata ×4.
   - *Recorded:* IMC read and write, `cycle_activity.stalls_l2_miss` and `stalls_l3_miss`, wall, and
     peak RSS.
   - *Predicted:* about 1 B/B less IMC, more L2-miss stall cycles per byte, and a wall ratio that
     *rises* with worker count.
   - *Refuted if* it falls.

## 5. Is A's mechanism distinct from B, C and D? (I)

**Distinct from B (record 73).** B contracts structure *before* induction: equal-byte residuals and
tiles. A runs no induction at all.

**Distinct from D (record 74).** D's tiles are a plain forward walk with no learning, and its final
slot keeps Cells.

**Closest to C (record 72),** but different.
- *What they share:* both walk the output forward over the run pieces, learn affine words, and copy
  from the completed output.
- *How C learns:* it intersects maximal certificates along a full first-return path at each miss,
  walking byte by byte, keyed to an anchor byte's section with a direct-address table.
- *How A learns:* at *domain* granularity, by **composing two adjacent visits** (Miller–Wegman style),
  with only a containment test, no anchor, and permanent retirement.
- *What each pays:* A's visits cost 2–3 dependent loads each. C walks 27% of bytes singly.
- *Both have the same weakness:* a longer serial dependent chain than Cells, the shape records 72 and
  74 found unfavourable under contention.

**What this decides:** nothing in production.
