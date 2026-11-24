# The main theorem (draft section for PROOF-PROGRAMME.md, owned by MONODROMY)

*Draft of 24 November 2026, cycle 1. It is written to be spliced into PROOF-PROGRAMME.md as a new §5.11
by the Coordinator; it builds on §1.2's `Column` and T1's `decode`. Labels: proved / argued /
conjectured / measured. Parameters fixed in §1 (C6, before the data).*

## 1. The parameters, fixed before any construction (MONODROMY, under RULES C6, 24 November, before N-0's data)

```lean
def 𝓕 : Finset Column      -- every one of Wikidata's 232 distinct blocks (#805) and the 6 admitted blocks
def K : ℕ := 2             -- passes over n: the output, plus one working pass of O(1) work a byte
def δ : ℚ := 1/2           -- per block: c₁ ≤ h₁/2 on every block of 𝓕
def S (c) : ℕ := openPeakScratch c   -- state beyond the output: at most shipped Open's own peak scratch on c
```

**The reasons, one line each:**
- **𝓕: every block, none dropped.** Excluding GUID-heavy or high-h₁ blocks would dodge random content, the hard case. The target is per block, with no corpus average.
- **Correctness on every valid column,** not just 𝓕. A decoder exact only on 𝓕 is a lookup, not an algorithm.
- **δ = 1/2 per block,** chosen against what is reachable, not toward it. On six blocks, 75–85% of h₁ is hex (record 96; I saw this before fixing δ, and disclose it). With random rows at aR = 1, δ = 1/2 is unreachable. Meeting it therefore requires beating the floor on random content itself. It also cannot be met by a constant (`origPtr`) or a handful of special rows.
- **K = 2.** Open uses one pass over n, its output. One extra working pass is the most a step change can spend and still plausibly pay. A K that admits several passes would invite wasteful global procedures. Sort-based routes are excluded in any case, by the count (record 99).
- **S at most Open's own peak scratch on the same block.** Costs can't move into memory. At 10 workers, extra bytes pay an L3 tax (record 95).
- **The split σ is not part of the target.** The target is on total c₁. σ is only a proof device (P1): it must be decidable from the records in O(r), and its own cost counts.

### 1a. The anti-cheat clauses (part of the target)

1. **c₁ charges every fact type** (record 100 §1). A single-row determination of a successor or a position is one event, whatever its type.
2. **No bundling.** A rule application with s rows in its subject is charged the number of those rows whose own run records it takes as distinguished inputs, not 1. Packing single-row determinations into one "multi-row" step doesn't reduce c₁. Only rows determined from aggregate information go uncharged.
3. **No cost moved off the ledger.** Every rule must be declared with a sound derivation from the records and the facts already held. A yes must also carry the utility receipts of §7. Fewer events that cost more time is not a yes.
4. **Depth is not the count** (record 98). Parallelism, chains and SIMD count per row.
5. **Fixed before the data.** These values are fixed before Neon's N-0 measurement (h₁R/h₁ on 𝓕). Under C2 they are never relaxed after any data is seen. Where MONODROMY and Neon disagree, the stricter reading holds, logged in CLAIMS.md.

## 2. The calculus 𝓡 and its cost (record 98 §1)

```lean
structure Fact (c : Column) where
  I : Interval c.n            -- [a, b)
  h : ℕ+
  t : ℤ
  w : Word h
  valid : ∀ x ∈ I, c.LF^[h] x = x + t ∧ c.emitted x h = w

inductive Rule (c : Column) : Fact c → Type
  | prim    (R : c.Run)                                : Rule (Fact.ofRun R)
  | mirror  (R : c.Run)                                : Rule (Fact.ofRunΨ R)
  | restrict {f} (d : Rule f) (J ⊆ f.I)                : Rule (f.restrict J)
  | compose {f g} (d : Rule f) (e : Rule g) (f.I + f.t ⊆ g.I) : Rule (f.comp g)
  | join    {f g} (d : Rule f) (e : Rule g) (adj : f.I.adjacent g.I) (same : f ≈ g) : Rule (f.join g)

/-- A decoder over a fact language Λ ⊇ 𝓡: a trace of events, with passes and state. -/
structure Decoder (Λ : FactLanguage) where
  trace   : (c : Column) → Trace Λ c
  passes  : Column → ℕ
  state   : Column → ℕ
  correct : ∀ c, (trace c).output = c.decode          -- T1

/-- An event's join row: a composition's single join point on a width-1 subject, or a
    chain link's start. -/
def Event.row : Event Λ c → Option (Fin c.n)
def c₁ (τ : Trace Λ c) : ℕ := (τ.events.filter (·.subjectWidth = 1)).card
```

## 3. The theorem

```lean
/-- G1 (argued: records 96, 98; Lean target, about 600–1,000 lines) -/
theorem G1 (D : Decoder 𝓡) (c : Column) : c₁ (D.trace c) ≥ c.h₁

/-- P1, additivity (proved here, §4.1) -/
theorem c₁_split (σ : Split) (τ : Trace Λ c) :
    c₁ τ = c₁ (τ.on (σ.rows .N)) + c₁ (τ.on (σ.rows .R))

/-- The main theorem: conditional existence. -/
theorem main (σ : Split) (DN : StreamDecoder .N) (DR : StreamDecoder .R)
    (P0 : σ.cheap)                                       -- decidable from records, O(r)
    (PN : ∀ c ∈ 𝓕, c₁ (DN c) ≤ aN c * c.h₁N)            -- stream N's premise (Neon)
    (PR : ∀ c ∈ 𝓕, c₁ (DR c) ≤ aR c * c.h₁R)            -- stream R's premise (MONODROMY)
    (Pglue : Glue DN DR)                                 -- the two streams' outputs interleave exactly
    (Pbudget : ∀ c ∈ 𝓕, passes DN c + passes DR c + passes Pglue c ≤ K)
    (Pmargin : ∀ c ∈ 𝓕, aN c * c.h₁N + aR c * c.h₁R ≤ (1 - δ) * c.h₁) :
    ∃ D : Decoder (𝓡 ∪ ΛN ∪ ΛR), ∀ c ∈ 𝓕,
      D.passes c ≤ K ∧ D.state c ∈ O(c.r) ∧ c₁ (D.trace c) ≤ (1 - δ) * c.h₁
```

Here h₁N + h₁R = h₁ is the split of the width-1 runs by σ.

## 4. The premises, by stream

### 4.1 P1 (proved)

**The argument.** Every event of positive count has exactly one row, its join point or link start (record 98, G1's sketch).
- A width-1 composition has a single join row.
- A width-1 link has a single start.

So c₁ is a sum over rows, and any partition of the rows splits it.

**Not covered:** events on wider subjects cost nothing in c₁ and are not split. That is the point of counting width 1 only.

### 4.2 P0, the split (open: a choice)

**What it needs:** the split must be decidable from the run records in O(r).

**Candidate σ_hex:** R = the width-1 runs whose byte is a hex digit (record 96's proxy).
- *Recall* for GUID width-1 rows: 0.92–0.93 [measured, six blocks].
- *Precision* for GUIDs: 0.31–0.73. The rest of R is other hex content (sha1, numbers, Q-id digits).

**TODO(M, Coordinator or an agent):** measure σ_hex's h₁R/h₁ on 𝓕 from the census. It needs a column, not mathematics.

### 4.3 PR: stream R (MONODROMY)

**Worst case** [proved for 𝓡, by G1 restricted with P1]: aR ≥ 1 for every decoder in 𝓡.

**Outside 𝓡,** see §5. The conjecture is that aR ≥ 1 − o(1) on model-I content within K passes.

**Best case:** aR = 0, possible only in E2's open cell. With no construction known, PR's best supportable value is **aR = 1**.
- *Then Pmargin needs:* aN·h₁N ≤ (1 − δ)·h₁ − h₁R. With δ = 1/2 that means h₁R ≤ h₁/2 and stream N nearly free.
- *It turns on:* h₁R/h₁, the TODO above.

### 4.4 PN: stream N (Neon)

Neon's E2 screen, on its branch. Its premises go here with citations when they land.

### 4.5 Pglue (open)

**What it needs:** the two streams' outputs must interleave into the text exactly.

**Why it is nontrivial:** a row's stream, N or R, says nothing about where its bytes go. Gluing needs the positions at the boundaries between N- and R-segments of the orbit. Each boundary row is an event (a link or a join), so the number of boundaries is charged to c₁ unless it is computed positionally.

**Open:** the boundary count is at most 2·min(h₁N, h₁R) + O(1) chain links [argued: each boundary is a switch on the orbit]. Whether Pglue is free is a named missing step.

## 5. The closed branches (escapes that are not escapes)

| branch | status | why |
|---|---|---|
| everything in 𝓡: the walk, Cells and Rauzy–Veech, Open, lanes, T² | **closed** (G1, argued) | one currency: an RV step on a width-1 piece is a rule application on a width-1 subject. RV is not the answer |
| E3, list ranking, pointer jumping, sort-based contraction | **closed: inside 𝓡** (record 99 corrects record 98) | a pointer jump composes x→LF(x) with LF(x)→LF²(x), a rigid fact on a width-1 interval. Contraction splices x by composing pred(x)'s fact with x's. Each costs a width-1 event, so c₁ = Θ(n log n) and Θ(n) respectively. Depth falls, count doesn't |
| E1, class facts ("LF^k restricted to D_u is order-preserving onto I_u") | **closed as an escape** [argued, record 99] | at k = 1 it is LF's definition in rank form. Evaluating it at a row is that row's primitive fact. A class fact gives successors in bulk, not text positions. Turning it into output order needs the chain or E2 |
| E2, local (k-context or landmark placement) | **closed** | 3-byte global sensitivity (FRONTIER, Epsilon E1, proved and verified) |
| E2 at O(1) depth | **closed under audit** | Gamma's G1 (L-complete under uniform TC⁰) |
| E2, labels first | **closed** | run labels determine the text (record 84), so positions cost at least the labels |
| **E2, global, non-rigid, not O(1) depth, within K passes** | **OPEN** | no construction, no obstruction: stream N's territory |

## 6. What a "no" is, and the margin

**A no:** a theorem that for every decoder over Λ = 𝓡 ∪ E1 ∪ E3 ∪ E2-local ∪ E2-O(1), some c ∈ 𝓕 has c₁ > (1 − δ)·h₁. That is, the open cell is empty on 𝓕. It ends the major shot cleanly.

**Why a margin δ.** A strict "c₁ < h₁" is met by saving one event.
- *For example,* the marked row's position is given by `origPtr`, so one positional fact is free on every block.
- *The fix:* the target needs c₁ ≤ (1 − δ)·h₁ with δ fixed in advance.
- *The value:* δ = 1/2, fixed in §1.

## 7. What stays outside the theorem: utility, as receipts

The theorem proves existence on 𝓕. A yes must also present three receipts, none of them proved:
1. cost-weighted applicability on the census, before formal work;
2. the supplied-capability payoff on real blocks, before construction;
3. a win on the bench against master, at the end.

These are the receipts of FRONTIER's "The target".
