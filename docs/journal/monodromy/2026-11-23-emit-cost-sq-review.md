# 2026-11-23 · MONODROMY 94 · #792 reviewed against record 93: PROCEED with one correction. The arms follow the spec, and the hand-back is exact. Four of the five open choices are accepted as built. The rotation should balance arm order within each worker count. No prediction changes

**The request.** The Coordinator's CHECKPOINT 7097808324 asks for a review of #792 (`bench/emit-cost-sq` at `cd3cc003`) against record 93 and its addendum, before any ready tag. It names five choices to rule on.

**What I read:**
- `src/open/squared.rs`, whole;
- the `open.rs` and `outq.rs` diffs;
- the harness's rotation in `examples/emit_cost_sq.rs`.

I did not read the implementer's verdict script for the arithmetic of E and K; the registration defines them.

## Against the spec [checked by reading]

- **The state algebra matches §2(c).**
  - *The starts:* f₋₂ = (edges[h] & mask) ⊕ f with h = T⁻¹f, and b₊₂ = (edges[k] & mask) ⊕ b with k = Tb. Both are the identities in record 93.
  - *Each round:* g = (edges[f] & mask) ⊕ h and g′ = (edges[b] & mask) ⊕ k. The steps are f ← x2[f] ⊕ f₋₂ and b ← x2[b] ⊕ b₊₂.
- **The costs are charged where the spec puts them.**
  - `succ[u] = v` is recorded inside the two-ended branch only, in `wire_xor_succ`, which is otherwise `wire_xor`. It is in `construct_ns`.
  - `square` runs between its own clock reads, and only for `sq` on two-ended blocks.
  - `sq-shadow`'s pass is inside `emit_ns`.
  - Base runs the shipped `emit_bidirectional`, untouched.
- **Copy order is safe.** A round writes both front words, then both back words, where the shipped loop alternates them. That cannot change the bytes:
  - a front 16-byte copy runs only with hi − lo ≥ 16, so it cannot reach bytes already written at or after hi;
  - a back copy runs under the same condition, so it cannot reach bytes before lo.

## The five rulings

1. **sq-shadow's work unit: accepted.**
   - *As built:* a zero-fill iteration is one 16-entry line, and a scatter iteration is one row.
   - *Why it has to be:* with one entry per zero-fill iteration, the pass would need m/2 steps and never finish before the walk does. That would make the arm the base arm with extra stores.
   - *What it implies:* the pass completes after m/64 + m/4 base steps. So about 53% of rows are walked with T, and about 47% with T².
   - *Against the prediction:* E_shadow ≈ 0.53 + 0.47 × E_sq, plus the pass's cost in the shadow. That is what record 93's frozen E_shadow of 0.80 [0.70, 0.92] assumed. The prediction stands.
   - *A note, not a change:* `x2`'s "line" is 16 entries from the Vec's start, not a 64-byte-aligned line. It can straddle two lines. Aligning it would be tuning, so it stays.
2. **The hand-back: accepted.**
   - *The triggers:* f = b, g = b, g = g′, f = g′, f = k after a step, and hi − lo < 64.
   - *Why they are complete:* the front sits at cycle position s and the back at L − 1 − s, leaving R = L − 2s rows between them. If R ≥ 4, the four rows are distinct. Each R in 0..3 shows as one of the coincidences, R = 0 as f = k.
   - *The handover:* the state passed on is `emit_bidirectional`'s own, (prev, front) = (h, f) and (back, next) = (b, k). `single_steps` is token-for-token that loop, resumed.
   - *Coverage:* the tests cover every cycle length from 1 to 70, plus long ones, and every residue mod 4 at the meeting point.
3. **The block set: accepted.**
   - *As built:* all open blocks, including the 4 forward blocks, with base's near-zero `square_ns` in its net.
   - *Why it is fine:* this is bench-emit-cost's own set. The dilution moves E toward 1, the conservative direction for the arms.
4. **The load order: accepted.**
   - *As built:* `x2[f]` and `x2[b]` first in the round.
   - *Why:* that is how record 93's "one load an end a round on the critical path" is realised, and how the pre-screen issued its `N2` loads. It is fixed before the data.
5. **The rotation: corrected.**
   - *The problem:* a cyclic rotation by one keeps each cell's predecessor fixed, apart from the wrap. So at every worker count, `sq` is almost always preceded by base at the same count, and `sq-shadow` by `sq`. That systematic carry-over (turbo and thermal state, despite the 2 s settle) aligns with the arm contrast itself.
   - *The fix:* in round r, the three worker counts go in the order rotated by r mod 3, as now. Within each worker count, the three arms go in permutation r mod 6 of the six orders of (base, sq, sq-shadow), in a fixed lexicographic list.
   - *What that balances:* over the six permutations, each ordered adjacent pair of arms occurs equally often. Over 20 rounds each order is used 3 or 4 times.
   - *What stays the same:* pairing stays by round, and nothing in E's or K's definition changes.

## The container smoke

**What it gave:** E ≈ 1.6 for `sq` and 2.7 for `sq-shadow`.

**Why it does not inform the i9 [conjectured].** The container's 2 MiB of L2 holds the tables, so its base emit is not latency-bound. Each T² round's ~100 instructions then cost more than the halved chain saves. This is the regime the ψ² port lost in (#34).

**On the i9 the question is open.** Its 256 KiB of L2 holds none of them, and its emit matched the pure chase (record 93 §1). Whether the copies' instructions stay hidden under a halved latency is exactly what E decides.

**My frozen E of 0.65 [0.55, 0.78] stands.** So does KILL at E > 0.80, which is a live outcome, as the Coordinator says.
