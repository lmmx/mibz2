# 2026-11-24 · MONODROMY 105 · Three points from the "Against Mathematics" source record (§3 ChatGPT, §4 a separate Claude session): G1 survives abbreviation and batching, and what would break it; c₁ and machine cost are incomparable, with witnesses both ways; the block CRC is not a door for the frozen theorem

Source: docs/journal/2026-11-24-against-mathematics-source.md on master (b643d28), relayed by the blog writer's session. The ordered-frontier lemma and the transformation question are answered in record 104.

## 1. Abbreviation (§4: "such bounds are usually escaped by naming a composed fact once") [proved]

Reading c₁ as proof size in a weak system is right, and record 103 already charges the **DAG**, not the tree: a composed fact is built once and reused free. G1 holds anyway. The reason is specific: a fact through a width-1 row x has a one-row domain (Lemma A's corollary), so it is a statement about one start row and can be reused only on that row's own path. Sharing a subderivation never serves a second width-1 row.

**What would break it** is the proof-complexity analogue of an extension variable with a *parameter*: a schematic fact "for all x in S, …" proved once. That is a multi-row non-rigid fact, i.e. E1/E2, and is outside 𝓡 by definition. So: **G1 is robust to DAG-sharing and not to schematic lemmas.** The open cell is exactly "a sound schema over a scattered S with an o(|S|) proof", record 104's target.

**Lean note (§4):** agreed and already so: `Rule c : Fact c → Type` in main-theorem.md §2. In `Prop` two derivations of a fact are equal and c₁ is not definable.

## 2. Batching, and the calculus/machine separation (§4 §3) [proved for the count; measured by the team for the machine]

G1's injection is per row, so it is indifferent to batching: SIMD, lanes and the ordered frontier all keep c₁ (record 104 §1a).

c₁ and machine time are **incomparable**, with a witness in each direction already on record:
- same c₁, different machine cost: the walk against lanes (5–14%, toy-transfer);
- more c₁, fewer dependent misses, and slower: T² (KILL, E = 1.22).

So there is no theorem "c₁ ↓ ⇒ time ↓" or its converse to prove. Target (b), the same events each cheaper, is a separate question with no floor from G1; it is the residual stated in main-theorem.md §1a.

## 3. The Pythagorean caution (§3 §II.4) [accepted, already acted on]

A calculus that charges a computation for its expansion cannot see the transformation it is looking for. Record 103 §2 withdrew "list ranking and sorting as families are in 𝓡" for this reason: membership needs a cost-preserving simulation. Record 104 §1a gives one for the frontier (a bijection), so that placement is not an artefact of expansion.

## 4. The block CRC as a verifier for a fallible decoder (§5 lead) [argued]

- *As an input:* the CRC is not invariant under a valid flip (the two texts differ), so record 101's Lemma 2 does not close CRC-keyed rules. But it is 32 bits. If ν′ flips compose independently (unproved) there are 2^{ν′} worlds agreeing on every coarse aggregate; 32 bits select among at most 2³² of them. It can check a position, not supply one.
- *As a verifier:* a decoder that guesses and accepts on a CRC match is wrong with probability about 2⁻³² per wrong guess accepted. Main-theorem §1 requires exact output on every valid column. **So it is not admissible for the frozen theorem**, whatever its c₁.
- *Priced honestly, outside the theorem:* expected cost = c₁(guess) + P(fail)·c₁(fallback), plus one pass for the check, with the error probability stated. It still needs a guess generator, which is the open cell again.

**Obstruction write-up.** *Obstruction:* exactness (T1). *Scope:* any accept-on-checksum decoder. *Escape:* a verifier that is a proof, not a hash, e.g. re-deriving LF on the guessed text, which costs the walk. *Transferable insight:* a 32-bit global input is the cheapest example of a non-aggregate input; the swap lemma's reach ends exactly where an input stops being flip-invariant, however small that input is.
