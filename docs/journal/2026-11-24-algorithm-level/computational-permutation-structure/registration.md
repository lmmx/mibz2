# Registration: two operators, frozen before data

Claim `computational-permutation-structure` (P1 explore, smooth). Written 27 November 2026 before any
measurement below was taken. Harness: `examples/permutation_operator_probe.rs` (zero `src/` edits,
`#![forbid(unsafe_code)]`). Corpus `corpora/wikidata.bz2` (SHA-256 `82f65ace…c29915`), Open's admission
rule, train = first 117 admitted blocks, held-out = last 116 (the split of #992/#998).

## Notation and the reference map

`p` = raw LF. `L` = BWT column. `R` = Open's retained rows at `LIMIT = 16` (thin maximal runs, plus the
primary as a singleton if it sits in a thick run). `T` = first return of `p` to `R`, indexed by retained
rank in row order; `w(x)` = the bytes spelled from `x` until the return (backward text order). The
reference `T` is built offline from `p` (it stands in for Open's A6a–A6c output, whose cost is the
anchor `C`, not charged again). Exactness = the emitted stream, reversed, equals `bwt::walk` byte for byte.

## O1 — labelled powers `J_k(i) = (p^k(i), W_k(i))` as interval maps (axis 1)

Rows `j, j+1` lie in one labelled translation piece of `p^k` iff their preceding `k`-mers are equal
(induction: equal `L` gives `p(j+1) = p(j)+1`). So `P_k(p)` = the number of runs of the column of
preceding `k`-mers, `P_k <= min(n, k r)`. Computed by `eq_k[j] = (L[j] = L[j+1]) && eq_{k-1}[p(j)]`.
The same on `T`: `eqT_k[j] = w(j) = w(j+1) && T(j+1) = T(j)+1 && eqT_{k-1}[T(j)]`.

- **Diagnostic (train, counts only):** `P_k(p)/n` for `k ∈ {1,2,4,8,16,32}` and `P_k(T)/m` for
  `k ∈ {1,2,4,8}`.
- **Survives only if** some `k` has `n/k <= m` (no more visits than Open) and a piece table of
  `P_k(p)·(k + 8)` bytes no larger than Open's `4m + dict` on >= 90% of train blocks; or some `k >= 2`
  has median `P_k(T)/m <= 0.25`. Otherwise O1 is structurally excluded and gets no timing.

## O2 — word-capped labelled doubling of `T` by independent gathers (axis 2)

A 16-byte record per retained row: `next` (20 bits), `len` (5 bits; `LONG` = dictionary code), and up
to 12 inline bytes. Round `t+1`, double-buffered, in index order:
`J'(x) = J(J(x))`, `W'(x) = W(x)‖W(J(x))` if both are inline and `|W(x)|+|W(J(x))| <= 12`, else `J(x)`.
Each round is one sequential read, one *independent* gather and one sequential write per row: no
dependent chase, no hashing, no renumbering (skipped rows stay as holes). This differs from the stopped
`mat_f3b_post_open` (#992), which chased the cycle dependently and renumbered; the STOP's stated mechanism
(chasing random edges) is what this tests. The consumer walks `J` until `n` bytes are written (a jump
past the start only overshoots into the 16-byte slack, so it is exact for any composition choice).

- **Arms (same harness, same blocks, warm, pinned, one thread):** `ctrl1` / `ctrl4` = Open-format
  forward walk of `T` (packed `u32` edge + `inline16` dictionary), 1 and 4 lanes; `J_r` 1 and 4 lanes for
  `r ∈ {0,1,2,3}` rounds (`J_0` isolates the record format). `C_r` = pass 0 (edges → records) plus
  `r` rounds, timed separately. Lane starts are the same offline rows for every arm (single-cycle blocks).
- **Selection (train, 3 repeats, baseline build):** the `r` minimising `C_r + E(J_r, 4 lanes)`.
- **Decision (held-out, 7 interleaved repeats, baseline and native):** O2 is an actionable candidate only
  if `E(ctrl4) − (C_r + E(J_r,4))` > 0 with non-overlapping min..max, the same sign in both builds, and
  its ratio projection `(E(ctrl4) − C_r − E(J_r,4)) / E(ctrl4) × 325.2 ms` (#992's warm i9 4-lane `E`)
  is >= 39.18 ms. Otherwise STOP, scoped to word-capped doubling on Open's retained cycle.
- **Exactness:** every arm byte-identical on every admitted block before timing.

Cold-cache, eight-worker and i9 numbers are out of scope; a projection is never a PASS.
