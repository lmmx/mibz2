---
title: "One theorem, many routes: correctness and speed in a decoder that adapts to its input"
image: assets/social/blog-2026-11-22-one-theorem-many-routes.png
description: "A correctness proof says which decoders are admissible but not which is fast, so mibz2's speed comes from exact routes whose conditions match the data."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# One theorem, many routes: correctness and speed in a decoder that adapts to its input

A correctness proof says which decoders are admissible but not which is fast, so mibz2's speed comes from exact routes whose conditions match the data.
{: .post-dek }

22 November 2026 · 10 min read · [Research in progress](#6-what-is-proved-measured-and-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 is a bzip2 decoder in Rust, and most of its time goes on inverting the Burrows–Wheeler transform (BWT) of each block. This post is about how correctness and speed relate in that inversion, and what that says about where to search for faster decoders.

**The takeaways:**

1. **A correctness proof fixes what is admissible, not what is fast.** It is no cost model. On the i9-10900K we test on, two changes that are both exact had opposite fates: one cut CPU time by 5% and left 10-thread wall time unchanged; the other cut wall time by 7–10%.
2. **mibz2's decoder is a family of exact, conditional routes.** Its correctness has the form "for each route i, Pᵢ(x) ⇒ Fᵢ(x) = R(x)": whenever a block x satisfies route i's condition Pᵢ, the route's output equals the reference specification R.
3. **The conditions Pᵢ are where the substance is.** Each defines a regime, and a regime common in real data is worth a kernel.
4. **When a condition fails on real data, there are three responses:** weaken it; find a cheaper certificate that it holds; or change the representation so that neither the condition nor its test is needed. The third keeps the decoder from becoming an ever-growing dispatch tree.
5. **Scope:** one decoder (bzip2's inverse BWT), one workstation (the i9-10900K), and Wikidata as the primary corpus. Nothing here is proved in Lean yet.

## 1. What does a correctness proof fix, and what doesn't it?

[lean-zip](https://github.com/kim-em/lean-zip) is a DEFLATE implementation in Lean whose round trip, `inflate (deflateRaw data level) = .ok data`, rests on "more than 1,100 theorems". Its README calls the result a ratchet: "the obligation `inflate (deflate x) = x` either still holds or the build goes red", so agents could optimise it aggressively without breaking it. It also states where it stands: its decoder trails "the C-grade decoders (miniz_oxide by ~1.45×, libdeflate by ~2.5×)". The proof guarantees every optimisation is correct. It does not say which optimisation will be fast.

mibz2 has the same gap. Two recent changes are both byte-exact on every block, and the i9 judged them very differently at 10 threads on four copies of the Wikidata benchmark:

<figure class="bf-fig" id="cpu-wall">
<p class="bf-title">Two exact changes on the i9 at 10 threads: CPU time against wall time</p>
<svg viewBox="0 0 720 384" role="img" aria-label="CPU against wall time at 10 threads on the i9 for two correct changes. The construction stack cut CPU to 0.950 and 0.955 but wall was 1.000 and 0.968, both inconclusive. The block reader cut wall to 0.932 alone and 0.895 on top of #739, with CPU 0.964 and 0.977."><line x1="90.0" y1="40" x2="90.0" y2="330" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="90.0" y="348" class="bf-tm" text-anchor="middle">0.85</text><line x1="202.0000000000001" y1="40" x2="202.0000000000001" y2="330" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="202.0000000000001" y="348" class="bf-tm" text-anchor="middle">0.90</text><line x1="313.99999999999994" y1="40" x2="313.99999999999994" y2="330" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="313.99999999999994" y="348" class="bf-tm" text-anchor="middle">0.95</text><line x1="426.00000000000006" y1="40" x2="426.00000000000006" y2="330" style="stroke:var(--bf-frame);stroke-width:2"/><text x="426.00000000000006" y="348" class="bf-tm" text-anchor="middle">1.00</text><line x1="538.0000000000002" y1="40" x2="538.0000000000002" y2="330" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="538.0000000000002" y="348" class="bf-tm" text-anchor="middle">1.05</text><line x1="650.0000000000002" y1="40" x2="650.0000000000002" y2="330" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="650.0000000000002" y="348" class="bf-tm" text-anchor="middle">1.10</text><line x1="90" y1="330.0" x2="650" y2="330.0" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="82" y="334.0" class="bf-tm" text-anchor="end">0.85</text><line x1="90" y1="257.49999999999994" x2="650" y2="257.49999999999994" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="82" y="261.49999999999994" class="bf-tm" text-anchor="end">0.90</text><line x1="90" y1="185.00000000000003" x2="650" y2="185.00000000000003" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="82" y="189.00000000000003" class="bf-tm" text-anchor="end">0.95</text><line x1="90" y1="112.49999999999997" x2="650" y2="112.49999999999997" style="stroke:var(--bf-frame);stroke-width:2"/><text x="82" y="116.49999999999997" class="bf-tm" text-anchor="end">1.00</text><line x1="90" y1="39.999999999999886" x2="650" y2="39.999999999999886" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/><text x="82" y="43.999999999999886" class="bf-tm" text-anchor="end">1.05</text><text x="370" y="372" class="bf-tm" text-anchor="middle" style="font-size:12px">CPU time, new ÷ old (10 threads)</text><text x="24" y="185" class="bf-tm" text-anchor="middle" transform="rotate(-90 24 185)" style="font-size:12px">wall time, new ÷ old</text><line x1="291.152" y1="112.35499999999999" x2="337.5200000000001" y2="112.35499999999999" style="stroke:var(--bf-amber);stroke-width:2.5"/><line x1="314.22399999999993" y1="136.86" x2="314.22399999999993" y2="87.41499999999982" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="314.22399999999993" cy="112.35499999999999" r="5.5" style="fill:var(--bf-amber)"/><text x="340.22399999999993" y="102.35499999999999" class="bf-t" text-anchor="start" style="font-size:12px">construction stack, run 1</text><line x1="272.33600000000007" y1="158.3199999999999" x2="378.51200000000006" y2="158.3199999999999" style="stroke:var(--bf-amber);stroke-width:2.5"/><line x1="324.752" y1="208.49" x2="324.752" y2="106.41" style="stroke:var(--bf-amber);stroke-width:2.5"/><circle cx="324.752" cy="158.3199999999999" r="5.5" style="fill:var(--bf-amber)"/><text x="350.752" y="144.3199999999999" class="bf-t" text-anchor="start" style="font-size:12px">construction stack, run 2</text><line x1="315.79200000000003" y1="211.0999999999999" x2="373.3600000000001" y2="211.0999999999999" style="stroke:var(--bf-green);stroke-width:2.5"/><line x1="344.2400000000001" y1="223.71499999999997" x2="344.2400000000001" y2="198.48499999999999" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="344.2400000000001" cy="211.0999999999999" r="5.5" style="fill:var(--bf-green)"/><text x="372.2400000000001" y="215.0999999999999" class="bf-t" text-anchor="start" style="font-size:12px">block reader</text><line x1="350.9600000000001" y1="264.60499999999996" x2="396.4320000000001" y2="264.60499999999996" style="stroke:var(--bf-green);stroke-width:2.5"/><line x1="373.5840000000001" y1="294.4749999999999" x2="373.5840000000001" y2="234.15499999999992" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="373.5840000000001" cy="264.60499999999996" r="5.5" style="fill:var(--bf-green)"/><text x="401.5840000000001" y="268.60499999999996" class="bf-t" text-anchor="start" style="font-size:12px">block reader on #739</text></svg>
<figcaption>Measured on the i9-10900K, Wikidata ×4, 16 paired rounds each, 95% intervals. The construction stack (runs open-r5/run-20261122T051443Z and, on a later master, run-20261122T103311Z) saved CPU but not wall time. The block reader (parscan-c37/run-20261122T082138Z, decision 2; and block-reader-on-wormhole/run-20261122T093349Z, on top of #739) saved wall time.</figcaption>
</figure>

The construction stack saved 5% of CPU time, 0.950 [0.940, 0.961], yet its 10-thread wall time was 1.000 [0.983, 1.017]; a re-run on the newer master gave 0.968 [0.934, 1.004], still inconclusive. The block reader saved less CPU but cut wall time to 0.932 [0.923, 0.941] alone and 0.895 [0.875, 0.916] combined with #739. Saving work and saving time are different things once ten workers share one cache and one memory bus. Only the target machine decides.

## 2. The decoder is a family of conditional routes

mibz2 does not invert every block the same way. The shipped dispatch, in `src/bwt.rs` and `src/open.rs`, chooses per block:

- **Open** takes run-route blocks its admission accepts: those the count gates admit, plus a dense band up to r/n 0.28 under a cap on its first pass ([#703](2026-11-21-a-memory-gain-is-a-gain.md)). Inside Open, a short-cycle tiling handles periodic blocks, and a spiral contraction bounds long repetitive eliminations.
- **LazyCells, then Segmented,** takes the run-route blocks Open refuses.
- **The column route** takes blocks whose runs are not taken at all.

Each route has an admission condition Pᵢ and is exact on the blocks that meet it. The reference R is not a route: it is the mathematical specification, the walk of the LF permutation on a valid block, against which every route is tested byte for byte.

## 3. Where a condition fails is where the work is

Two conditions show the pattern on Wikidata.

The pure-arithmetic solver for two-letter Sturmian blocks inverts them in O(log n) steps by the Euclidean algorithm, and it is exact. It never fires on a Wikidata block: an [audit of our classical tools](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md) finds `gcd(w) = 1` on every block and 0.00% of Rauzy–Veech steps with a quotient of 2 or more. Its condition describes a regime that real data does not visit.

The statement identifiers are the opposite case. They are random 122-bit GUIDs, 28.7% of emission by the audit's count, and their contexts shatter after about four or five hex digits. No condition on local structure admits them cheaply.

The three responses to a failed condition:

- **Weaken it.** Open's dense band widened the admission from r/n 0.15 to 0.28, under a cap that keeps memory bounded.
- **Find a cheaper certificate.** The spiral contraction runs only when a block fails a width certificate, checked once before the loop: the widest initial strip bounds every later step.
- **Change the representation.** Open itself is the example: it replaced the per-row LF walk with first-return maps on whole strips, so the condition "this row is in a wide run" no longer needs testing row by row.

As an image of this process, Gilles Deleuze's essay "He Stuttered" (in *Essays Critical and Clinical*, 1997; French 1993) is apt. Deleuze argues that a great writer does not speak a fixed language strangely but makes the language itself stutter, putting it into variation from within; his examples include Melville, Kafka and Beckett. Here the conclusion, recover the text, stays fixed, while the hypothesis returns again and again, transformed by what the data refuses. Deleuze made no claim about algorithms, and we claim none for him: the analogy describes the shape of the process, not evidence that it works.

What this suggests, as a proposal, is a map of transformations between the hypotheses themselves: P₁ weakened to P₁′, two invariants combined into P₃, a counterexample to P₂ pointing at a new representation.

## 4. When is a family saturated?

One research team built a hindsight oracle: for each of the 431 Wikidata blocks it picks the best setting of Open's two parameters (L, μ). On the team's own machine, not the i9, the oracle would save 3.17% over the shipped settings on one thread.

That says the (L, μ) family is nearly saturated on Wikidata. It says nothing about routes the oracle never considered. An oracle tells you when the family it covers is saturated, and nothing about alternatives outside it. The same team found 12.8–13.1% headroom on six non-Wikidata blocks, so the saturation is a property of the data as well as of the family.

A new representation is what lies outside such a family. Open's first-return maps are a classical idea, the induced map of Poincaré and Kakutani (1943), reached through Rauzy–Veech induction (1979–82); its proof is in our [journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md).

## 5. A new representation can be a different proof of one lemma

The Riemann Hypothesis for curves over finite fields has several proofs: Weil's (1948), and the elementary method of Stepanov (1969), completed by Bombieri (1973). The claimed proof for the integers announced on 20 November 2026 is described as a descendant of Stepanov's ([Two centuries of tools](2026-11-22-two-centuries-of-tools.md)). A different proof of the same theorem, not a new theorem, opened the new route.

Inverse BWT has the same structure. The LF walk, cycle decomposition and first-return induction are different constructive proofs that the BWT is invertible, and each gives a different decoder. Our [proof programme](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md) names seven points, L0 to L6, where a step of the proof can be replaced: how the permutation is represented, its cycle structure, composition of orbit segments, restriction, the induced map, composition of interval maps, and recovering the orbit from local data. Open replaced the classical route at L3, L4 and L5.

The audit sharpened why this is the right target. Of 66 uses of pre-1950 tools in two research cycles, only 9 were load-bearing. Many were relabellings of code that already existed. A search for "a different proof of this one step" can't be met that way, because it has to change what the step computes.

## 6. What is proved, measured and next

**Proved:** nothing in Lean yet. The proof programme sets out the theorem and its seven substitution points. **Measured:** the i9 results above, and each route's byte-exact parity on every block. **Next:** formalising the reference and Open's correctness, and searching for a different proof at one of L0–L6 that the i9 then judges.

---

**Sources:** [lean-zip's README](https://github.com/kim-em/lean-zip); Gilles Deleuze, "He Stuttered", in *Essays Critical and Clinical* (1997), « Bégaya-t-il… » in *Critique et clinique* (1993).

**Research records:** [the classical-tools audit](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-22-classical-tools-audit.md); [the proof programme](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/PROOF-PROGRAMME.md); [Open's first-return proof](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-18-open-return/proof.md); the i9 evidence bundles named in the figure, on the `evidence` branch.
