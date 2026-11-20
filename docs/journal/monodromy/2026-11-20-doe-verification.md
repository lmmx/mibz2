# 2026-11-20 · MONODROMY 85 · DOE audited. (a) c(E) ≤ 2m holds. (b)'s 4m bound holds, but its middle inequality |P_exit| ≤ c(E) + |∂U \ C̄| is false: a pair can split at entry, on a strip boundary that is a codomain start. Corrected: |P_exit| ≤ c(E) + |∂U| ≤ 4m. DOE is exact. Its elimination is Open's endpoint induction up to bookkeeping, with the same exits on every case and every block

**The request.** The Coordinator's CHECKPOINT 7027995386: DOE ("Decoupled Open Exchange", Neon via
Louis).

**Sources.** I worked from the CHECKPOINT's definitions alone. DOE's own text is not in the
repository, and I did not use Neon's script. For Open: `src/open/endpoint.rs` and
`docs/journal/2026-11-18-open-return/proof.md`.

**Labels:** **proved** · **verified** (exhaustive on the stated domain) · **measured** (counted on the
six authenticated blocks).

**Script:** `doe.py`, beside this record as `.txt`.

## Verdicts

| claim | verdict |
|---|---|
| (a) c(E) ≤ 2m | **proved and verified.** 0 failures in 15,681 cases |
| (b) \|P_exit(E)\| ≤ c(E) + \|∂_int U \ C̄\| | **false.** 18 counterexamples; the smallest is below |
| (b) \|P_exit(E)\| ≤ 4m | **proved with a correction:** \|P_exit\| ≤ c(E) + \|∂_int U\| ≤ 4m. 0 failures |
| Neon's charging | **injective, but the charged point is not always in ∂_int U \ C̄.** At step 0 it can be in ∂_int U ∩ C̄. That is the only failure: 144 of 926 splits, all at t = 0 |
| "A thin-run boundary never splits a part" | **proved and verified.** 800 pairs exit at the same step into different thin runs; thin_id adjacency keeps every one in one part |
| DOE exactness | **verified,** on all 15,681 cases (several cycles and periodic columns included) and all six blocks |
| q ≤ \|A₀\| + \|X₀\| + \|E₀\| ≤ 6m − 1 | **verified** for the right-hand inequality: \|A₀\| + \|X₀\| + \|E₀\| = 0.33–0.36 of 6m − 1 on the blocks. DOE's q itself is not computed, since its elimination text is not available |
| Same as Open's endpoint construction? | **Yes, up to bookkeeping:** the same first-return map, and the same exit partition on every case and block |

**The domain:** every binary column to length 8 and every ternary column to 5, at every mark p₀
and τ ∈ {1, 2, 4}: 15,681 cases.

## Claim 1 (proved, verified)

**(a) c(E) ≤ 2m.**
- *Why:* E = U \ ∪_j LF(U_j), and each LF(U_j) is one interval. U has at most m maximal
  intervals. Removing one interval from a union of intervals raises the count by at most one.
  So E has at most m + m = 2m maximal intervals.
- *Counted:* 0 failures.

**(b), the charging.** Take a split pair (y − 1, y) inside one interval of E, and follow it under
LF while both rows stay in one strip. That stretch translates both by +1, and their words agree.
- *Where the pair separates:* at the first step t where the rows lie in different strips, or one
  of them in S. Let p be the right-hand row at that step.
- *It is injective:* y is the first backward hit of E from p. For t ≥ 1 every intermediate point
  has its predecessor in U, so it is not in E. For t = 0, p = y.
- *If t ≥ 1:* the pair came through one run at step t − 1, so LF⁻¹(p) = LF⁻¹(p − 1) + 1. Hence
  p ∉ C̄, and p ∈ ∂_int U \ C̄, as Neon says.
- *If t = 0, Neon's claim fails:* (y − 1, y) both lie in E but in different strips, so
  p = y = min U_j ∈ ∂_int U. But y can be a codomain start, as y ∈ E = LF(S) for a thin run head.
  Then the split is charged to a point of ∂_int U ∩ C̄, which the bound does not count.

**Counterexample (verified by hand):** L = 1 0 1 1 0 0, p₀ = 2, τ = 1.
- *LF:* 3, 0, 4, 5, 1, 2.
- *Strips:* the thick run [2, 3] is split at p₀ into the retained row 2 and the strip {3}. The
  other strip is {4, 5}. So m = 2.
- *The sets:* U = {3, 4, 5} and S = {0, 1, 2}, so E = {3, 4} and c(E) = 1.
- *The boundary:* ∂_int U = {3, 4} and C̄ = {0, 1, 3, 4}, so ∂_int U \ C̄ = ∅. The bound gives 1.
- *The exits:* row 3 exits via 5 to row 2, with word 10. Row 4 exits to row 1, with word 0. So
  |P_exit| = 2.

**Corrected statement (proved; 0 failures on all cases and blocks):**
|P_exit(E)| ≤ c(E) + |∂_int U \ C̄| + |{p ∈ ∂_int U ∩ C̄ : p − 1, p ∈ E}| ≤ c(E) + |∂_int U| ≤ 4m.
- *Why the combined charging stays injective:* the t = 0 charges are rows of E, and the t ≥ 1
  charges are not.
- *Why 4m still holds:* |∂_int U| ≤ 2m.

**The same-step case.** Two rows that exit together land on adjacent rows of S. Their thin_id
values are consecutive whatever the runs, and their words are equal, so a thin-run boundary never
splits a part. 800 such pairs were found. **Neon is right here.**

## The six blocks, τ = 16 (measured)

| block | m | c(E) | \|P_exit\| | \|∂U \ C̄\| | c(E) + \|∂U \ C̄\| | 4m | t = 0 splits in ∂U ∩ C̄ | \|A₀\|+\|X₀\|+\|E₀\| | 6m − 1 | exact |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| 0 | 3,448 | 954 | 1,321 | 1,627 | 2,581 | 13,792 | 24 | 5,413 | 20,687 | yes |
| 2 | 3,827 | 1,458 | 1,921 | 1,978 | 3,436 | 15,308 | 34 | 6,685 | 22,961 | yes |
| 85 | 1,565 | 840 | 1,219 | 971 | 1,811 | 6,260 | 9 | 3,157 | 9,389 | yes |
| 143 | 5,047 | 2,582 | 3,634 | 3,278 | 5,860 | 20,188 | 61 | 10,227 | 30,281 | yes |
| 207 | 3,474 | 1,681 | 2,259 | 2,138 | 3,819 | 13,896 | 20 | 6,786 | 20,843 | yes |
| 210 | 2,809 | 1,613 | 2,228 | 1,706 | 3,319 | 11,236 | 24 | 5,695 | 16,853 | yes |

**On these blocks the false middle inequality holds numerically**, with wide slack. The
uncovered t = 0 splits are 9–61 per block. |P_exit| is 0.38–0.79 m: far below 4m.

## Claim 2: exactness (proved, verified)

**What the wiring gives.** For x ∈ S, the retained walk takes LF(x) if it is in S. Otherwise it
takes the exit of the part containing LF(x) ∈ E, at offset thin_id(Exit) = out_thin + offset, with
word L[x]·W_U(LF x). That is the first return to S, so the walk from p₀ spells LF's output from
p₀.
- *Several cycles and periodic columns:* repeat until n bytes, as Open does.
- *Cycles wholly inside U:* never visited, and never needed.

**Verified:** every case and all six blocks match the LF walk byte for byte. Exactness rests only
on the exits being the true first-exit data. The elimination order does not matter.

## DOE against Open's endpoint construction

**The same elimination, with different bookkeeping.**
- *Open's auxiliary exchange:* it has three families of arrows: terminal k → m + e_k, then the
  H strips, then m + s_i → i. It is induced from the right until the prefix alone remains (at
  most 3H + 2 arrows).
- *DOE's A, X and E nodes are the same objects:*
  - E nodes are Open's terminal-to-entrance arrows, restricted to entrances inside U. Open's
    entrances into S are zero-length paths, which DOE wires directly in its merge instead.
  - X nodes are the strip pieces whose image lies in S, carrying out_thin. Open represents them as
    a strip arrow followed by its m + s_i → i exit arrow. Folding the two together is bookkeeping.
  - A nodes are the strip pieces with image in U.
- *The same result:* both compute the first return to S on the strips, by right-end induction.
- *thin_id against rows:* DOE keys exits by thin_id, Open by S-component rows. thin_id adjacency
  could in principle merge two exits across a U gap, but it never does. The exit partition with
  row-contiguous destinations, as Open's terminals coalesce, equals P_exit on all 15,681 cases and
  all six blocks.
- *So DOE adds no new mechanism.* Its claimed benefit is decoupling the thin wiring into one
  sequential merge, a cost question. The phase split being measured on Open decides it, not this
  audit.
- *The two band bounds:* Open's proof bounds the bundles by e + H + s − 1, or 3H + σ with byte
  cuts. DOE's corrected 4m is the same kind of charging: each split goes to the first strip or
  component boundary it meets.
