# External research report 39: the researcher and the guardian on the selector's exact rule (verbatim)

A research input, not a finding of this repository. The maintainer relayed both on
2026-10-30, after external report 38 (`2026-10-30-external-report-38.md`).

**What they add to the selector rule:**
- **An exact integer test,** 10·F ≤ E, where E is the explicit (post-RLE2) MTF tokens and
  F those of rank ≥ 32.
- **A defined zero-token case.**
- **The guard as early acceptance only:** A ≤ 32 accepts at once; otherwise the 10%
  test decides.
- **The denominator is explicit tokens,** never output bytes.
- **The report gives each arm's share of decoded bytes,** as well as its count of
  blocks.
- **The 3% control limit still binds** the selector on random and records, since its
  count is extra work there.
- **Not an authorisation.**

**The coordinator's notes:**
- **When E = 0 and A > 32.** A valid block whose used symbols all appear has
  E ≥ A − 1, so E = 0 with A > 32 needs an in-use table naming symbols the block never
  uses. That is legal in the format, but no encoder writes it. With no explicit token
  there's no fallback, so P74 there is the consistent choice, as the researcher says.
- **The selector arm is still not started.** These messages were relayed, not adopted.

The text reached us as rendered text (formulas partly flattened; one stray Cyrillic
word, "предусмотрed", is in the original) and is kept as received.

## The messages, as received

````text
researcher on guardian:

===

The Guardian’s recommendation is sound, and I agree with its final position: retain the preregistered alphabet guard, fix the high-rank threshold at 10%, and give P74 its one remaining selector experiment.

There’s a useful technical nuance here.

The alphabet guard and the rank-frequency test serve different purposes. If a block has at most 32 used symbols, its high-rank fraction is provably zero. So the guard doesn’t actually change the classification decision relative to a 10% threshold—it simply avoids scanning the rank stream in that case.

That makes the combined rule both faithful to the preregistration and logically equivalent to a rank-only rule for valid blocks with nonzero explicit-token counts.

The precise decision is:

\text{Use P74} \iff A\le32
\quad\lor\quad
\left(E>0\ \land\ \frac{F}{E}\le0.10\right),

where is the number of ordinary explicit MTF tokens and is the number with rank at least 32.

For blocks with zero explicit tokens, the rank fraction is undefined. The implementation should define that edge case before measurement. Choosing P74 is mathematically consistent with the alphabet guard’s intent because there are no high-rank updates, although it doesn’t guarantee a speed advantage for run-only data.

One refinement: avoid floating-point classification

The 10% threshold can be tested exactly using integers:

\boxed{10F\le E.}

That avoids division, rounding and an unnecessary source of boundary ambiguity. With bzip2’s block-size limits, the multiplication is easily representable in an ordinary 32-bit unsigned integer when using the bounded number of encoded symbols.

There’s also a modest opportunity for an exact early rejection: while scanning the token stream, once , where is the exact total number of explicit tokens, no remaining observations can make the block eligible. But discovering itself requires prior counting or an existing summary, so I would not introduce an extra pass merely to enable this optimization.

For the frozen experiment, the simplest exact census is preferable.

Why the test is worthwhile

The measured Wikidata checkpoint result gives an unusually large margin:

Shipped

0.807

P74 branch-free

0.539

Available saving

0.268

Nanoseconds per decoded byte, Wikidata checkpoint MTF. Before selector overhead.

Even after adding a rank census, there’s meaningful room to retain a gain above the registered 5% threshold. Whether it actually does is the purpose of the experiment, not something we should assume.

One caution about the Guardian’s wording: the selector may rescue random and records by routing them to shipped MTF, but the classification pass itself can cause a small regression relative to shipped. The registered 3% control limit still applies; selecting the original kernel does not automatically guarantee parity.

I would send Claude the Guardian’s proposed authorization unchanged, with a small clarification to use the exact integer comparison 10 * high_rank_count <= explicit_count and explicitly define the zero-explicit-token case.

Once that is frozen, I see no reason for further theoretical intervention before the result.

My research verdict: P74 has earned this final controlled selector test. The 33.2% measured stage gain is too substantial to dismiss, while the negative-control regressions are too serious to ignore. The selector is the properly bounded experiment that can reconcile both.

===

guardian on researcher:

===

The researcher and I agree on the substantive decision: P74 has earned its one remaining selector experiment. A 33.2% improvement in the checkpoint-aware MTF stage is too substantial to discard without testing the safeguard explicitly предусмотрed by the protocol.

But I want to flag one subtlety in the proposed selector.

The alphabet guard needs careful treatment

The researcher recommends:

\text{Use P74 if } A\leq32 \quad\text{or}\quad f_{\geq32}\leq10\%.

That’s defensible as the original Option 1, but it contains two separate mechanisms.

For a block with at most 32 used symbols, high-rank fallback is impossible because the MTF rank cannot exceed . So the alphabet guard should be a cheap early acceptance, not a separate heuristic that overrides evidence of expensive fallbacks.

If the alphabet size is greater than 32, inspect the explicit ranks and apply the fixed 10% threshold.

This makes the rule:

\text{P74}=
\begin{cases}
\text{yes}, & A\leq32,\\
\text{yes}, & A>32\ \text{and}\ f_{\geq32}\leq0.10,\\
\text{no}, & \text{otherwise}.
\end{cases}

Importantly, the fraction must be computed from explicit post-RLE2 MTF ranks, with the appropriate denominator, not from all emitted bytes. Counting repeated zero-run output bytes would artificially make fallback-heavy blocks look safer.

What I particularly want from the selector result

The researcher is right to request per-block selection counts. I’d go one further: Claude should report the share of decoded bytes, not just the number of blocks, assigned to each arm. A selector could choose P74 for many small blocks while leaving most actual work on the shipped kernel.

That’s a reporting addition, not a new experimental arm.

And the CPU-efficiency question remains central. Gate 1 is still a stage-level measurement. Even a successful selector must clear the later full-decoder comparison before we can claim an improvement for production.

One important restraint

I would avoid calling P74 a full-decoder breakthrough at this point.

It’s an impressive stage-level result: roughly one-third less time in an important kernel, under the tested Zen 4 conditions.

The projected full-decoder improvement is around 5.7%, and hasn’t been measured. That’s valuable engineering progress, but our larger ambition remains to change the fundamental cost of decompression by much more.

The distinction doesn’t diminish P74’s result. It keeps our claims accurately matched to the evidence.

My recommendation

If you’ve already sent Claude the selector authorization, don’t send another instruction merely to restate it. The researcher has confirmed the same choice.

When Claude returns, I want to see whether the selector preserves most of that 33.2% checkpoint-stage gain, whether it successfully protects random and records, and whether the classification cost remains small.

A positive result would justify considering Phase 2—but not automatically authorizing it.

Meanwhile, N1 remains the higher-upside research direction. Its next obstacle is construction economics rather than mathematical correctness.

This is encouraging progress: P74 has produced a strong, reproducible stage-level gain, and the preregistered experiment now has one precise remaining question to answer.
````
