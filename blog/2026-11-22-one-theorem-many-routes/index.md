[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# One theorem, many routes: correctness and speed in a decoder that adapts to its input

A correctness proof says which decoders are admissible but not which is fast, so mibz2's speed comes from exact routes whose conditions match the data.

22 November 2026 · 10 min read · [Research in progress](<#6-what-is-proved-measured-and-next>)

mibz2 is a bzip2 decoder in Rust, and most of its time goes on inverting the Burrows–Wheeler transform (BWT) of each block. This post is about how correctness and speed relate in that inversion, and what that says about where to search for faster decoders.

**The takeaways:**

1. **A correctness proof fixes what is admissible, not what is fast.** It is no cost model. On the i9-10900K we test on, two exact changes had opposite fates. One cut CPU time by 5% and left 10-thread wall time unchanged. The other cut wall time by 7–10%.
2. **mibz2's decoder is a family of exact, conditional routes.** Its correctness has the form "for each route i, Pᵢ(x) ⇒ Fᵢ(x) = R(x)". Whenever a block x satisfies route i's condition Pᵢ, the route's output equals the reference specification R.
3. **The conditions Pᵢ are where the substance is.** Each defines a regime, and a regime common in real data is worth a kernel.
4. **When a condition fails on real data, there are three responses:** weaken it; find a cheaper certificate that it holds; or change the representation so that neither the condition nor its test is needed. The third keeps the decoder from becoming an ever-growing dispatch tree.
5. **Scope:** one decoder, bzip2's inverse BWT, on one workstation, with Wikidata as the primary corpus. Nothing here is proved in Lean yet.

## 1\. What does a correctness proof fix, and what doesn't it?

[lean-zip](<https://github.com/kim-em/lean-zip>) is a DEFLATE implementation in Lean whose round trip, `inflate (deflateRaw data level) = .ok data`, rests on "more than 1,100 theorems". Its README calls the result a ratchet: "the obligation `inflate (deflate x) = x` either still holds or the build goes red", so agents could optimise it aggressively without breaking it. It also states where it stands: its decoder trails "the C-grade decoders (miniz\_oxide by \~1.45×, libdeflate by \~2.5×)". The proof guarantees every optimisation is correct. It does not say which optimisation will be fast.

mibz2 has the same gap. Two recent changes are both byte-exact on every block, and the i9 judged them very differently at 10 threads on four copies of the Wikidata benchmark:

Two exact changes on the i9 at 10 threads: CPU time against wall time

Measured on the i9-10900K, Wikidata ×4, 16 paired rounds each, 95% intervals. The construction stack (runs open-r5/run-20261122T051443Z and, on a later master, run-20261122T103311Z) saved CPU but not wall time. The block reader (parscan-c37/run-20261122T082138Z, decision 2; and block-reader-on-wormhole/run-20261122T093349Z, on top of \#739) saved wall time.

The construction stack saved 5% of CPU time, yet its 10-thread wall time did not move. A re-run on the newer master was still inconclusive. The block reader saved less CPU but cut wall time by 7% alone, and by 10% combined with \#739. Saving work and saving time are different things once ten workers share one cache and one memory bus. Only the target machine decides.

## 2\. The decoder is a family of conditional routes

mibz2 does not invert every block the same way. The shipped dispatch, in `src/bwt.rs` and `src/open.rs`, chooses per block:

- **Open** takes the run-route blocks its admission accepts. These are the blocks its count gates admit, plus a dense band up to r/n 0.28 under a cap on its first pass ([\#703](<https://lmmx.github.io/mibz2/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>)). Inside Open, a short-cycle tiling handles periodic blocks, and a spiral contraction bounds long repetitive eliminations.
- **LazyCells, then Segmented,** takes the run-route blocks Open refuses.
- **The column route** takes blocks whose runs are not taken at all.

One specification, the shipped routes, and the condition under which each is exact

The shipped dispatch (src/bwt.rs, src/open.rs). Each arrow is a correctness statement: on blocks meeting the route's condition Pᵢ, its output Fᵢ equals the specification R. Research candidates that never shipped are not shown.

Each route has an admission condition Pᵢ and is exact on the blocks that meet it. The reference R is not a route. It is the mathematical specification, the walk of the LF permutation on a valid block, and every route is tested against it byte for byte.

## 3\. Where a condition fails is where the work is

Two conditions show the pattern on Wikidata.

The pure-arithmetic solver for two-letter Sturmian blocks inverts them in O(log n) steps by the Euclidean algorithm, and it is exact. It never fires on a Wikidata block. An [audit of our classical tools](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md>) finds `gcd(w) = 1` on every block and 0.00% of Rauzy–Veech steps with a quotient of 2 or more. Its condition describes a regime that real data does not visit.

The statement identifiers are the opposite case. They are random 122-bit GUIDs, 28.7% of emission by the audit's count, and their contexts shatter after about four or five hex digits. No condition on local structure admits them cheaply.

The three responses to a failed condition:

- **Weaken it.** Open's dense band widened the admission from r/n 0.15 to 0.28, under a cap that keeps memory bounded.
- **Find a cheaper certificate.** The spiral contraction runs only when a block fails a width certificate, checked once before the loop: the widest initial strip bounds every later step.
- **Change the representation.** Open itself is the example. It replaced the per-row LF walk with first-return maps on whole strips, so the condition "this row is in a wide run" no longer needs testing row by row.

As an image of this process, Gilles Deleuze's essay "He Stuttered" is apt. Deleuze argues that a great writer does not speak a fixed language strangely but makes the language itself stutter, putting it into variation from within. His examples include Melville, Kafka and Beckett. Here the conclusion, recover the text, stays fixed, while the hypothesis returns again and again, transformed by what the data refuses. Deleuze made no claim about algorithms, and we claim none for him: the analogy describes the shape of the process, not evidence that it works.

What this suggests, as a proposal, is a map of transformations between the hypotheses themselves. P₁ is weakened to P₁′, two invariants combine into P₃, and a counterexample to P₂ points at a new representation.

## 4\. When is a family saturated?

One research team built a hindsight oracle: for each of the 431 Wikidata blocks it picks the best setting of Open's two parameters (L, μ). On the team's own machine, not the i9, the oracle would save 3.17% over the shipped settings on one thread.

That says the (L, μ) family is nearly saturated on Wikidata. It says nothing about routes the oracle never considered. An oracle tells you when the family it covers is saturated, and nothing about alternatives outside it. The same team found 12.8–13.1% headroom on six non-Wikidata blocks, so the saturation is a property of the data as well as of the family.

A new representation is what lies outside such a family. Open's first-return maps are a classical idea, the induced map of Poincaré and Kakutani, reached through Rauzy–Veech induction. Its proof is in our [journal](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md>).

## 5\. A new representation can be a different proof of one lemma

The Riemann Hypothesis for curves over finite fields has several proofs. Weil's came first, in 1948; Stepanov's elementary method followed in 1969, completed by Bombieri. The claimed proof for the integers announced on 20 November 2026 is described as a descendant of Stepanov's ([Two centuries of tools](<https://lmmx.github.io/mibz2/blog/2026-11-22-two-centuries-of-tools/index.md>)). A different proof of the same theorem, not a new theorem, opened the new route.

Inverse BWT has the same structure. The LF walk, cycle decomposition and first-return induction are different constructive proofs that the BWT is invertible, and each gives a different decoder. Our [proof programme](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md>) names seven points, L0 to L6, where a step of the proof can be replaced. They are how the permutation is represented, its cycle structure, composition of orbit segments, restriction, the induced map, composition of interval maps, and recovering the orbit from local data. Open replaced the classical route at L3, L4 and L5.

The audit sharpened why this is the right target. Of 66 uses of pre-1950 tools in two research cycles, only 9 were load-bearing. Many were relabellings of code that already existed. A search for "a different proof of this one step" can't be met that way, because it has to change what the step computes.

## 6\. What is proved, measured and next

**Proved:** nothing in Lean yet. The proof programme sets out the theorem and its seven substitution points. **Measured:** the i9 results above, and each route's byte-exact parity on every block. **Next:** formalising the reference and Open's correctness, and searching for a different proof at one of L0–L6 that the i9 then judges.

---

**Sources:** [lean-zip's README](<https://github.com/kim-em/lean-zip>); Gilles Deleuze, "He Stuttered", in *Essays Critical and Clinical* (1997), « Bégaya-t-il… » in *Critique et clinique* (1993).

**Research records:** [the classical-tools audit](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md>); [the proof programme](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md>); [Open's first-return proof](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md>); the i9 evidence bundles named in the figure, on the `evidence` branch.
