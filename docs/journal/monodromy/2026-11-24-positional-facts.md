# 2026-11-24 · MONODROMY 100 · M-3: the random-content conjecture is model-relative. c₁ must charge single-row facts of every type, or E2 is met trivially. Given that, an escape is a sound multi-row fact type. On model-I rows, every such type keyed locally is closed; global keys stay open. The conjecture can't be proved over all sound fact types with present tools

## 1. A repair: c₁ counts every single-row fact [argued]

**The problem.** As defined, c₁ counts only 𝓡's width-1 events. A decoder that derives "pos(x) = k" one row at a time, by any computation, has c₁ = 0. It meets the target while being the walk in disguise, since knowing every pos determines every LF(x).

**The repair.** A decoder is a derivation in a declared fact language Λ ⊇ 𝓡. Each new fact type comes with a sound rule deriving it from the run records, and the facts already held.
- **c₁(D)** = the number of rule applications, of **any** type, whose subject (the set of rows the fact determines) has size 1.
- Chain links count as before.

**With that,** G1 still holds for Λ = 𝓡. For a single-row positional fact, the target can no longer be met by renaming.

## 2. What an escape must be

**The definition.** An escape is a fact type Φ whose facts determine a property (a successor, or a text position) of **at least two rows at once**, and that is not a rigid interval fact. The rows can be scattered, and the map need not be a translation.

**The test of utility:** how many of the width-1 rows lie in subjects of size ≥ 2.

## 3. Stream R: what closes, what stays open

**Model I:** the bytes emitted by GUID random nibbles are independent and uniform (record 96).

**(a) Locally keyed multi-row facts close [proved under model I].**
- *The type:* the subject is {x : key_k(x) = u}, where key_k reads at most k bytes of the row's left or right context, or of the records within k of its run.
- *The bound:* on random-nibble rows, a subject of size ≥ 2 needs a key collision. Record 96's G3 gives expected partners per row ≤ (N − 1)/16^k + (non-random context terms). That is ≤ 0.0034 at k = 6 on 𝓕's six admitted blocks.
- *The other side:* for k ≤ 5, subjects group rows, but each such fact fixes at most k emitted bytes. To place them, a position must still be supplied, by a fact of §(b) or a chain.
- *The closure:* that is 3-byte global sensitivity (FRONTIER, Epsilon E1). Valid columns at Hamming distance 3 agree on every k-local key, for k far below the radius, yet differ in global order. So no k-local positional fact is sound on both. Hence no sound k-local positional Φ exists for k below that radius, on any content.

**(b) Globally keyed multi-row facts: open.**
- *The type:* the subject or the formula depends on unboundedly many records. One example is "these rows' positions are an arithmetic progression", derived from global counts.
- *What is known:*
  - Gamma's G1 excludes O(1)-depth uniform-TC⁰ derivations of all positions (under audit).
  - Record 84 excludes derivations that need the run labels first.
- *What is not known:* nothing excludes a global Φ of polylog depth whose subjects are large on random rows.

**(c) Why the full conjecture isn't provable here.** It quantifies over all sound fact types with all derivation rules. Such a "no" is a lower bound for a general computation of permutation positions, of the kind that is open even for list ranking in the word RAM. A no is reachable only relative to a fixed, declared Λ. **Status: a conditional effect with an unproduced prerequisite.** The prerequisite is a declared Λ, which the formulation must fix.

## 4. For the main theorem

- **PR, stated relative to Λ:** for Λ = 𝓡 ∪ {k-local Φ}, aR ≥ 1 − (expected partners) ≥ 0.996 at k ≥ 6 on model-I rows [proved under model I, plus G1 argued].
- **A "no" (§6 of the draft):** must name its Λ. The strongest available is 𝓡 ∪ E1 ∪ k-local Φ ∪ O(1)-depth Φ.
- **For Louis's formulation:** the target is meaningful only with c₁ charged on every fact type (§1), and its no only relative to a declared Λ.
