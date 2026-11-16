# 2026-11-16 · MONODROMY 36 · My cost model, calibrated against the builds: every lever handed off since lazy letters was underpriced, by +20 to +100 instructions a letter, and none was overpriced. Six terms account for the misses. From now on, no handoff without an instruction estimate checked against a counted prototype in the kernel's own layout

Asked by the Coordinator, at Louis's direction (18:51). It also closes the tail-order line: record 35
stands as written, and its open question is not pursued.

**Labels:** **C** counted · **M** modelled.

## 1. Model against build

The model column is what my record or HANDOFF said, in instructions a letter; the build column is
the Pathfinder's count or the i9 run.

| lever (record; handoff) | my model | built | the term missed |
|---|---|---|---|
| Ring of eight (record 14) | "one compare of the new pair against all 8, two 256-bit compares": a few instructions a step, about +5 to +10 a letter (Conj, not built) | **+69 a letter** (+38% induction); slower at 4 and 8 threads, level at 9 and 10 (LOG 6906106922) | The ring's state kept live in the hot loop, so spills. I priced the compare, not the loop. |
| Stop-and-walk at n/32 (record 19; HANDOFF 6915365756) | 0 to +9 without pass 1; +12 to +20 with it (C prototype + M) | **+97** (REVIEW 6916500604) | The stop test on every cell (+18). Pass 2 on the kernel's word layout (104, against my 78 on a pre-built arena). Reading the exchange and ranking (11). Scratch zeroing (12). |
| Stop-and-walk on the budget counter, one lane (record 21; HANDOFF 6916839849) | about −20 (−32 before the reader, taken at 10) | **+10.8, then +12.3**: passed Gate 1, then NOT ADOPTED on the i9, at 1-thread time 1.064 | The reader (18.9 against 10). The compiler: the stop instance is +22.6 under rustc 1.99.1 and −37.6 under 1.97 (LOG 6925397308). Memory stalls, which instructions don't show (+0.39 cycles a byte; LOG 6925144719). |
| Two chains (record 23; HANDOFF 6922450227) | +11.9 over one chain, to halve the latency | the chain halved, and the walk's time did not fall (REVIEW 6924288835) | The walk was bound by its writes and stalls, not the chain's latency. |
| Runs (record 26; HANDOFF 6923667034) | write calls halved (C); no total instruction estimate | runs covered 79% of points, and the walk's time did not fall (LOG 6923867887) | Per-point bookkeeping: each walked point recorded and scanned, about 22 instructions, 17 a letter before any byte moved. |
| Certificates (record 28 counted the shares; the Pathfinder built the relay's §3) | no instruction estimate from me | 49.2 → 76.3 for the walk, about level once slimmed, slower in time | A lookup on every point, and spills from the certificate state. |
| Compact tail (record 27; HANDOFF 6923908876) | "sign unknown"; 4.9 B a letter saved | **+70.6** at s = 1.2 r under 1.99.1, +29.1 under 1.98.0 (LOG 6925397308) | The stop and resume (+26.8). Joins in place of rules (+11.4). The writer rebuilding lengths a join drops (+29.4). |

Records 25 (staged chains) and 32 (the tail order) were not built.

## 2. The systematic error

**Every miss is in one direction:** each model underpriced its build, by +20 to +100 instructions a
letter, or by a time cost the instructions did not carry. The Coordinator's guess, per-point and
per-cell bookkeeping, is one of six terms. Each term is named by the rows above.

1. **My prototype's data layout, not the kernel's.** My passes ran on flat tables and arenas I built
   in the simulator, and that build was not counted. The kernel reads its own slots, streams and
   rules: pass 2 104 against 78, the reader 18.9 against 10.
2. **Fixed and transition costs.** A stop, a second loop instance, a reader, scratch zeroing. These
   are terms that exist only in the built design (+11 to +27 each).
3. **State added to the hot loop.** A ring, a certificate, a stop test, a switch. I priced these by
   their own instructions; they cost by the spills they cause in a loop already at its register
   limit. The lazy-overhead entry recorded the same lesson on the Pathfinder's side.
4. **Per-point and per-cell bookkeeping.** Recording what a later step needs (rooms, first places,
   stamps) on every unit, where only some units use it.
5. **Information a representation drops must be rebuilt.** A join without a length forces a forward
   pass with dependent loads (+29.4; W8's +17% writer time).
6. **Instructions are not time, and not one number.** The 1-thread loss came from memory stalls
   with flat cache misses. The same source compiles 60 instructions a letter apart under two rustc
   versions.

**The common root.** I netted a prototype's own instructions against an average kernel cost per
cell, which is a difference of two numbers measured in different places. Every term above lives in
that difference.

## 3. The rule from now on

1. **No handoff without an instruction estimate checked against a counted prototype** built in the
   kernel's own layout and loop structure: its slots, streams, rules and word layout, not
   simulator tables.
2. **Where only the kernel can count it,** in particular any change to the hot induction loop, the
   handoff says so in its title, gives no net estimate, and asks for a count before any rule.
3. **Every handoff lists its terms explicitly,** including the four I missed: transition costs,
   readers, scratch, and information rebuilt.
4. **Time claims are labelled M,** with the memory-stall caveat.
5. **The rustc version is named** beside any count I cite.

## 4. The corrected cost rule, on paper (added after the Coordinator's revision, 19:01)

**Where the calibration lives now.** On paper, against what the Pathfinder and the i9 have already
counted. I don't build calibration prototypes; that is the Pathfinder's lane. Before any proposal, I
add these measured unit costs to whatever the mechanism's own work is:

| unit (measured, with its source) | cost |
|---|--:|
| a tail cell made, lazy kernel (REVIEW 6916500604) | about 38 to 71 instructions |
| a walk point handled, bytes included (LOG 6923867887) | about 73 instructions |
| per-point bookkeeping: record and scan (LOG 6923867887) | about 22 instructions a point |
| reading a stopped exchange (REVIEW 6924288835, after §9) | about 17 instructions a letter |
| a stop, or a second loop instance (LOG 6925397308) | +18 to +27 a letter, compiler-dependent |
| any state added to the hot induction loop | at least +20 a letter: the ring's +69, the certificates' +27, the one-pass switch +75 |
| rebuilding a dropped length (LOG 6925397308; #503 W8) | about 17 instructions a node, or +17% writer time |
| a byte cut of the working set at 10 threads (LOG 6925144719) | **no LLC relief measured:** 5 B a letter left LLC misses at 1.000 |

**The rule.** A proposal's estimate is its own work plus every unit above that it incurs. If that
sum does not clear the gate with margin, I do not propose it. A change to the hot loop is never
estimated net: its title says "kernel count needed".
