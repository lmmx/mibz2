# 2026-11-15 · MONODROMY 6 · The locality bound under the amended registration: CLOSED at 5 MiB, on a certified bound

Correction to `2026-11-14-locality-bound.md`, which answered the original registration
(LOG 6883302716). This answers the amended one (LOG 6883444614; QUEUED 6884173025), which was
fixed before any data. Induction phase only; no whole-decoder claim; untimed. Labels: **P**
proved · **C** counted · **Conj** conjecture. Method and proofs: `2026-11-15-locality-amended/method.md`.

## The amended rule

WORTH if a verified, realizable order has H/R ≤ 0.70. CLOSED only if a certified lower bound
has L/R ≥ 0.85. Otherwise NOT SETTLED, given as L/R ≤ O/R ≤ H/R. Read at 5 MiB. Model: 64 B
write-back, write-allocate LRU; Q_model = 64 × (fills + dirty evictions + lines still dirty at
the end, rule A). Capacities 256 KiB (the i9's L2), 2 and 5 MiB (equal L3 shares), 1.25 MiB as a
sensitivity point.

**Stated initial state:** cold before `slots`; `slots` runs through the same cache, and the
induction starts with what it leaves. Cold-start and rule B (end-dirty lines uncounted) are
sensitivities.

## Result

| capacity | Q(R) lines | Q(H) | L | **L/R** | **H/R** | verdict |
|---|--:|--:|--:|--:|--:|---|
| **5 MiB** | 18,202,069 | 18,202,057 | 18,201,795 | **0.999985** | 0.999999 | **CLOSED** |
| 2 MiB | 23,000,938 | 22,586,866 | 18,641,760 | 0.8105 | 0.9820 | not settled: [0.81, 0.98] |
| 256 KiB | 52,568,171 | 51,725,543 | 22,114,145 | 0.4207 | 0.9840 | not settled: [0.42, 0.98] |
| 1.25 MiB (sensitivity) | 26,818,914 | 26,435,464 | 20,009,373 | 0.7461 | 0.9857 | not settled: [0.75, 0.99] |

Modelled bytes per output byte at 5 MiB: R, H and L all 6.604. The 5 MiB verdict holds under
every sensitivity (cold start, rule B). At 256 KiB and 2 MiB nothing is settled, and under the
rule they decide nothing: L there counts only compulsory traffic.

## What is certified (P)

- Under master's layout, every realizable order touches the same lines: accesses outside the
  three bump-allocated regions have fixed addresses, and each bump region is the same contiguous
  tiling whatever the order (C: 12,567,184 guaranteed lines against R's 12,567,310).
- Fills ≥ the guaranteed lines not resident after `slots`; write-backs (rule A) ≥ the lines every
  order writes or `slots` left dirty, since each stays dirty until evicted or counted at the
  end; under rule B, that set less the capacity.
- An allocator that reuses dead slots or packs differently changes the line set and is outside
  this bound.

## What is realizable, and a correction

H is the window-64 greedy with fresh slots, arena words and rules allocated by the kernel's own
cursors in execution order. All 196 × 4 orders are linear extensions of the step graph and keep
every byte-exact read/write and write/write hazard of R (C); with allocation in execution order,
objects move only as wholes and copy16's slack lands past the cursor, so slots, arena and rules
end equal to R's up to renaming (P).

**The 14 November entry's fixed-layout orders were not realizable.** At R's fixed addresses, a
step run early overwrites a word that R had not yet allocated; a full-width check including
copy16's slack fails on all 784 of them, while R passes (C). The earlier check looked at the
words' own bytes only. Its verdict stands only because the compulsory floor, which does not
depend on any found order, was already above 0.85.

## Limits

One corpus. Fully associative LRU, no prefetchers. Fetch-ahead loads and the stack are not
traced. The greedy is bounded (window 64). That the legal tree-preserving schedules are exactly
the step graph's linear extensions remains a conjecture beyond small words; H does not rely on it.
