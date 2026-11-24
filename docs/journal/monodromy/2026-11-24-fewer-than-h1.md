# 2026-11-24 · MONODROMY 98 · "Fewer than h₁", made well-posed. The class becomes a calculus of rigid facts derived from the run records, and the floor counts its width-1 steps over the whole program. As stated, the target is met trivially twice over: Open's two ends already halve the dependent depth, and sort-based list ranking has no single-row steps at all. It needs a count, not a depth, and a streaming budget. Every escape the backward chain finds is blocked on random content or still open. A Lean statement runs to about 600–1,000 lines

**The request.** The Coordinator's CHECKPOINT 7128531277 relays Louis: "yes put fewer than h₁ to monodromy". It asks for inputs to his formulation (`docs/journal/2026-11-24-algorithm-level.md`, #808, §§1 and 11–13), not the formulation itself:
1. a precise class with its escape condition;
2. whether "fewer than h₁" is well-posed;
3. the backward chain, checked against FRONTIER;
4. whether R0–R3 are the right cut;
5. whether a Lean statement is the right first target, and its size.

**Labels:** **[proved]** · **[argued]** (a proof sketch, unchecked) · **[conjectured]** · **[measured]**.

## 0. The book: what this rests on

**Measured:**
- h₁ and the run widths are exact on six blocks (record 96).
- h₁/m is 0.30 / 0.39 / 0.83 (minimum / median / maximum) over Wikidata's distinct blocks. That is #805's census, which I did not rerun.
- Random GUID digits behave as independent uniform digits on six blocks (record 96, G2 and G3).

**Assumed:**
- The decoder's input is the run records and the primary, with nothing else: a standard bzip2 block carries only L (after MTF/RLE2/Huffman) and `origPtr`.
- Streaming the records and writing the n output bytes are unavoidable in every decoder. The target counts neither.

**Code path:**
- Open: one walk step per retained row; width-1 runs are always retained (`LIMIT` = 16, `src/open.rs:185`). Two ends when m ≥ 65,536 (`:522`).
- Cells: right-end Rauzy–Veech over every run.
- The LF walk: one step per row.

## 1. The class, defined so that the walk can't hide in a helper

**The rigid-fact calculus 𝓡.** A *fact* is a tuple (I, h, t, w):
- I = [a, b) is an interval of rows, and h ≥ 1;
- LF^h(x) = x + t for every x ∈ I;
- every x ∈ I emits the same word w of length h on the way.

**The rules:**
- **Primitive facts** come one per maximal run R, from its record: (R, 1, t_R, L[R]). Their Ψ-direction mirrors, (LF(R), −1, −t_R, ·), are allowed too.
- **Restrict** a fact to a sub-interval.
- **Compose** (I, h, t, w) with (J, h′, t′, w′) when I + t ⊆ J, giving (I, h + h′, t + t′, w w′).
- **Join** two facts on adjacent intervals with equal (h, t, w).

**A decoder in 𝓡** is any program whose knowledge of LF (or Ψ) comes only from facts derived by these rules. It outputs the text by concatenating the words of a chain of facts along the marked orbit. Its emission order may be anything (two ends, k chains, any schedule), and its steps may run in any order and on any hardware.

**Its cost, c₁(D),** is the number of width-1 events **anywhere in the program**: construction, helpers and the walk alike. An event is either:
- a rule application whose subject interval has width 1; or
- a link of the output chain whose fact has width 1.

Streaming the records and writing output bytes are not counted. Chain links must count: otherwise the LF walk, which links width-1 primitive facts directly, would score 0.

**Checking the known decoders against it:**
- **The LF walk:** composes primitive facts one row at a time. It is in 𝓡, with c₁ ≥ h₁.
- **Cells:** a right-end Rauzy–Veech step restricts and composes facts on intervals, so it is in 𝓡.
- **Open:** endpoint induction on the thick strips is in 𝓡. Its walk of retained rows composes their facts. It is in 𝓡, with c₁ = h₁ + (the width-1 pieces its induction creates).
- **The lanes, T² and the band dictionary:** all in 𝓡. T²'s x2 table is the composite fact for each retained row.

**Theorem G1 restated [argued; the prose proof is record 96's].**
- *The statement:* every decoder in 𝓡 has c₁(D) ≥ h₁ on every column.
- *The proof sketch:*
  - By induction on derivations, every fact's interval lies inside one run at each of its h steps. Primitive facts do; restriction and composition preserve it; a join merges only intervals with the same itinerary.
  - Every row of the marked orbit is either the start of a chain link, or a join point (I + t) of a composition somewhere in a used fact's derivation. Primitive facts take one step, so every interior row of a derived fact is a join point of its derivation.
  - For the row x of a width-1 run, that link or composition has a width-1 subject. The fact reaching x, or starting at x, is rigid and lies in one run at that step.
  - A composition has exactly one join row on width-1 subjects, and the orbit visits each row once. So distinct width-1 rows give distinct events, and c₁ ≥ h₁. Joins and restrictions add no steps.

**The escape condition** (FRONTIER's sense): drop "every fact is a translation on an interval". Concretely, any of:
- **E1, monotone class facts:** "LF^k maps the scattered set D_u, the rows sharing their next k symbols, order-preservingly onto the interval I_u" (record 92 §4).
- **E2, positional facts:** a row's text position, or a run's, obtained by arithmetic rather than by a chain of facts (FRONTIER foundational question 1).
- **E3, sort-based facts:** a whole permutation pass, by sorting or by list ranking with independent gathers.
- **E4, a different input or problem:** R3.

## 2. Is "fewer than h₁" well-posed? Not as worded. Three repairs

**(a) "Dependent" must not mean depth [proved, by example].**
- *What happens if it does:* Open's two-ended walk already processes the retained cycle in about m/2 dependent rounds. With width-1 rows spread around the cycle, that puts about h₁/2 width-1 rows on the critical path. Shipped Open would already meet a depth reading of the target.
- *So:* the target must be the **count** c₁, which G1 bounds, not the depth.

**(b) A count with no budget is met by a known algorithm [argued].**
- *The algorithm:* sort-based inversion, that is, external-memory list ranking (Chiang et al.; costed by the team at 4 × (16 B + 12 ops) per element in `2026-11-20-carbon-depth-audit.md`). It determines every row's text position by O(log n) sorting or pointer-jumping passes, with no single-row rule application at all, so c₁ = 0.
- *Why that doesn't count:* it is E3, outside 𝓡, a known algorithm, and slower: Θ(n log n) streaming work, or several full passes.
- *So:* the target needs a streaming budget.

**(c) Per block, with a stated family.** For a yes/no target, my suggestion:

> **Target.** An exact inversion of every column in a stated family 𝓕 with
> c₁ < h₁(column) single-row determinations, using at most K streaming passes over n
> (K fixed in advance, e.g. 3: records in, one working pass, output out) and O(r)
> words of state beyond the output.

- **Per block,** because G1 is per block and a corpus total could be met on easy blocks alone.
- **𝓕 stated in advance.** "All columns" is probably false: §3 shows the known escapes fail on random content. A structural family, chosen before any construction, keeps the target yes/no.
- **The payoff stays a separate condition,** measured at the end (§12's condition 2).
- **SIMD gathers and k chains count per row,** under c₁'s definition.

## 3. The backward chain from the target

**Goal.** Output the text with c₁ < h₁ and K passes. So some width-1 row x has LF(x), or its text position, determined without a width-1 rule application on x. One step back, that needs a fact outside 𝓡 that covers x together with other rows. There are three ways:

| step | what the decoder must have | status | against FRONTIER |
|---|---|---|---|
| **E1: class facts** (x jointly with the other rows sharing its next k symbols) | D_u's membership for x's key u | **blocked on random content.** Learning that x ∈ D_u needs x's next k labels, whose first step is x's own run fact: one width-1 application per row [argued, by G1's induction]. Membership from global counts alone is a **named missing step** with no known construction. And on random digits, D_u is a singleton beyond k ≈ 5 (record 96, G3 [proved under model I]), so a class fact covers one row anyway | consistent with **3-byte global sensitivity** (local k-context placement ruled out) and **cross-block width-1 indistinguishability** |
| **E2: positional facts** (x's text position, or its run's) | a formula or procedure giving many positions jointly | **blocked in three forms.** *Labels first:* run labels determine the text (record 84), so positions must cost less than the labels they imply. *O(1) depth:* ruled out by **Gamma's G1** (L-complete under uniform TC⁰, under audit). *Composed along orbit segments,* FRONTIER's "cheap composition of long orbit segments": any rigid summary of a segment through a width-1 run has width 1 (Lemma 3), so it batches nothing; a non-rigid summary is E1, blocked as above. **Open:** a global, non-rigid, non-O(1)-depth positional procedure within K passes. No construction is known, and no obstruction covers it | record 84; Gamma's G1; the toy-transfer lead; foundational question 1 |
| **E3: sort-based** | every position by permutation passes | **native** (correct and known), but it **fails the K-pass budget** at n ≤ 900k: Θ(log n) passes | the carbon depth audit |
| **(inside 𝓡)** | c₁ ≥ h₁ by G1 | **closed** | the induction floor r (records 79–80) is the same kind of floor for monotone induction schedules; r ≥ h₁ |

**What the chain gives [conjectured].** For the family of columns whose width-1 rows are random digits (model I), no decoder with K = O(1) passes meets the target. Every route the chain finds is either:
- blocked on that content (E1, and E2's composed form); or
- excluded by the budget (E3); or
- open with no construction (E2's global form).

**So a yes, if one exists, lives in E2's open cell, on a family 𝓕 whose width-1 rows are not random.** Copy-sector content is one such: sha1 references and numbers that repeat. There E1's classes do group rows, since G3 fails where content repeats. Whether that group structure can be obtained without the per-row first step is the precise missing step.

## 4. The route classes R0–R3

**Mostly the right cut, with three amendments.**
1. **Define R1 and R2 by the calculus, not by "inside the class" in prose.**
   - *R1:* derivations in 𝓡 under any representation, order, layout or parallelism. G1 gives best outcome ≥ h₁.
   - *R2:* uses a fact type outside 𝓡 (E1, E2 or E3).
   - *R0:* R1 with the same derivation and different constants.
2. **R2 needs the budget,** or E3 (sort-based) lands in R2 with "a best outcome below h₁" and no merit. With the target of §2(c), R2 means: outside 𝓡, and within K passes.
3. **R3 within standard bzip2 streams is narrower than it looks.**
   - *Why:* the stream encodes L and `origPtr` and nothing more, so "using what the compressed stream already contains" (FRONTIER question 4) can add only the MTF and Huffman structure of L itself.
   - *Real R3:* a format change, or a different problem, such as cross-block or service-level decoding.

**Is "best outcome ≥ h₁?" decidable from record 96 alone?**
- **For a candidate written in the calculus:** yes, by inspecting which fact types it uses. It is a syntactic check: G1 covers every derivation in 𝓡.
- **For a candidate given as Rust code:** not in general (Rice's theorem). It needs either an audit that maps each of its operations to a rule of 𝓡, or the candidate stated in the calculus first. That is one more argument for the Lean statement: the classification becomes a type check.

## 5. Formalisation

**Yes: the calculus and G1 are a good early target, as the second after PROOF-PROGRAMME's shared specification** (§1.2: the column, LF and T1's definitions). Both are stated on the same objects. G1 is the formulation's yes/no boundary, so checking it is what makes "R1 or R2" a matter of type, not taste.

**Rough size,** in Lean 4 with Mathlib, on top of the shared specification:

| part | lines |
|---|--:|
| Facts, rules and derivations: an inductive type of derivations with their subjects, and validity | 150–250 |
| Lemma 3: a derived fact's interval lies in one run at each step (induction on derivations) | 150–300 |
| G1: each width-1 run needs a width-1 rule application, and distinct runs need distinct ones | 250–400 |
| Glue: chains covering the marked orbit, the output as concatenated words, c₁ | 100–200 |
| **Total** | **about 600–1,000** |

The longest part is G1's injectivity argument: matching each width-1 run to its own application.

**Not in that estimate:**
- E1–E3's blocked status. The G3 part is probabilistic, under model I, and would need a separate development.
- Gamma's G1, a complexity-class result well beyond this size.

**What a checked G1 would and would not give:**
- *It would make the class boundary exact.*
- *It would not make the escapes impossible.* It certifies only that R1 cannot reach the target.

## 6. Postmortem (by the book)

1. **Input assumptions.**
   - *Measured:* h₁ (six blocks here; distinct blocks in #805), and model I on six blocks.
   - *Assumed:* the input is records plus primary. Streaming and writing are free in the count.
2. **Model assumptions.**
   - The calculus fixes what "rigid" means. A candidate that uses rank queries per row is still in 𝓡: a rank query determines one row's LF, which is that row's primitive fact.
   - The count is c₁, not depth and not instructions. The budget K is a choice, offered at 3, not derived.
3. **Method.**
   - G1 is record 96's argument, recast on derivations: argued here, not machine-checked.
   - The well-posedness repairs come from two known algorithms: two-ended Open and sort-based list ranking.
   - The chain classifies the escapes by fact type, checked against FRONTIER's table.
4. **Outcome against observation.**
   - Nothing here is measured beyond record 96.
   - The conjecture of §3, that no K-pass decoder meets the target on random-digit content, is **unproved**. It is the statement a formulation could adopt as its hardest case, or exclude through 𝓕.
