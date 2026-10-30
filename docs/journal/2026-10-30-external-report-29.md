# External research report 29: research burn-down round 39, P74's fixed head on short alphabets (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 28 (`2026-10-30-external-report-28.md`).

**What it contains:**
- **The invariant:** with the shipped identity initialisation and valid ranks
  1 ≤ q ≤ A − 1, the head's padding entries H[A…31] never move (by induction), so one
  fixed 32-byte head serves every alphabet.
- **The guarantees:** A ≤ 16 means one shuffle only, and A ≤ 32 means no tail fallback.
  For A = 1 there are no explicit updates, though RLE2 output is still possible.

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §3.2e(k).

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost) and is kept as received.

## Round 39, as received

````text
Burn-down round 39: P74’s short-alphabet correctness

Result: P74’s fixed 32-byte register head is correct even when the block uses fewer than 32 symbols. No special short-alphabet representation or fallback is required.

The shipped decoder provides a useful starting point: it initializes a 256-entry MTF list to the identity permutation, irrespective of how many symbols the block actually uses.

P74 can preserve that convention.

1. The potential problem

Suppose a bzip2 block uses only symbols.

The logical MTF list initially contains:

\[0,1,2,3,4,\\underbrace{5,6,\\ldots,31}_{\\text{unused entries}},\\ldots\].

P74 still loads 32 entries into its register.

At first glance, those unused positions could become problematic: the SIMD shuffle operates on all 32 bytes, even though only the first five symbols are valid members of the used alphabet.

The key is that every valid explicit rank is less than .

For this example:

1\le q\le4.

An MTF update at rank modifies only positions .

All positions above are unchanged.

Therefore, no operation can pull an unused entry into the first five positions.

2. Proof of the invariant

Let be the number of used symbols, where .

Initialize the 32-byte head to the identity permutation:

H^{(0)}=0,1,\\ldots,31.

We claim that after every valid explicit token:

\boxed{
HA,HA+1,\ldots,H31=A,A+1,\ldots,31.
}

Base case: true at initialization.

Inductive step: a valid explicit rank satisfies . Move-to-front changes only the prefix ending at . Every position therefore remains unchanged.

The invariant follows by induction.

The first entries are exactly the ordinary MTF permutation of the used symbols. The remaining entries are inert padding.

RUNA/RUNB does not modify the MTF list, so it preserves the invariant too.

3. A useful stronger result

For , every explicit token has rank at most 15.

Consequently:

\boxed{
A\le16\implies\text{every explicit MTF update uses one }VPSHUFB.
}

For , the upper register lane may participate in the update, but the memory tail never does.

Thus:

\boxed{
A\le32\implies\text{no high-rank memory-tail fallback}.
}

These aren’t statistical predictions. They are guaranteed by the block alphabet size.

P74 guarantees from the used alphabet

Alphabet	Guaranteed permitted update paths
1–16	One-shuffle only, when explicit tokens occur
17–32	One-shuffle or cross-lane; no tail fallback
33–256	All three paths possible

4. The one-symbol edge case

When , the only used symbol is already at the front.

There are no ordinary explicit MTF ranks that represent another valid used symbol. The block’s data can be expressed through RUNA/RUNB expansion followed by EOB.

P74 does not need to invoke an explicit-token shuffle at all.

The fixed head remains a valid representation, and its front symbol is unchanged throughout the block.

This also means the absence of ordinary MTF updates must not be mistaken for an empty decoded block: an RLE2 run can produce output without any explicit rank.

5. What this closes

Question	Conclusion
Must P74 initialize fewer than 32 entries for a small alphabet?	No
Can padded entries contaminate the used-symbol prefix?	No, proved by induction
Does guarantee the one-shuffle path?	Yes
Does guarantee zero tail fallbacks?	Yes
Is a distinct one-symbol MTF implementation required for correctness?	No
Does this guarantee a performance gain?	No

The useful outcome is that P74 can use one fixed 32-byte representation across the entire valid alphabet range. There is no need to introduce another selector branch based solely on representation correctness.

Relay to Claude

Research round 39 — fixed-head P74 for A≤32

The fixed 32-byte register head is correct for any bzip2 block using A≤32 symbols, including A=1.

Initialize the full logical 256-entry MTF identity permutation, with head H=[0..31]. Every valid explicit rank satisfies 1≤q≤A−1. Move-to-front modifies positions 0..q only, so positions A..31 remain fixed at their identity values by induction. RUNA/RUNB does not change the list.

Therefore unused head entries cannot contaminate the used-symbol prefix; no special padding, smaller vector or alternate state representation is required.

A≤16 guarantees all explicit updates take the one-VPSHUFB path. A≤32 guarantees the memory-tail fallback is unreachable. For A=1 there may be RLE2 output but no ordinary explicit-rank updates.

This closes short-alphabet representation correctness, not hardware profitability. It is consistent with the existing full-state equivalence tests and supports using alphabet size as a zero-census structural guard.

Round 39 closes the short-alphabet edge case. We can now keep the representation fixed and stop treating varying alphabet size as a correctness complication.
````
