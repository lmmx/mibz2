# External research report 28: research burn-down round 38, the alphabet bound on P74's fallback traffic (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 27 (`2026-10-30-external-report-27.md`).

**What it contains:**
- **The bound:** P74's tail movement on a rank-q fallback is 2q − 63 logical bytes. With
  q ≤ A − 1 that is at most 2A − 65, so 93–139 bytes on Wikidata's alphabets of 79–102
  symbols, against 447 for 256 symbols.
- **The average:** at most 2.224 B per explicit token, about 0.136 B per output byte.
- **Against the shipped fallback:** that moves 2q + 1 bytes, exactly 64 more an event.

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §3.2e(j).

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost) and is kept as received.

## Round 38, as received

````text
Burn-down round 38: Wikidata’s alphabet sharply limits P74 fallback traffic

The next useful result comes from combining two facts already established independently: P74’s exact high-rank state transition and the measured Wikidata alphabet sizes.

Although P74 supports all 256 possible MTF positions, its high-rank fallback on the measured Wikidata blocks only needs to modify a tail of at most 70 bytes.

That is substantially tighter than the 223-byte worst case for an unrestricted alphabet.

1. The exact bound

Let be the number of symbols in the block’s used alphabet. Every explicit MTF rank satisfies

1\le q\le A-1.

For a high-rank update, , define

j=q-32.

P74’s tail transition is

T’=H_{31},T_0,\\ldots,T_{j-1},T_{j+1},\\ldots.

Only the first tail positions change. Implemented as an overlapping move and insertion, that requires:

* bytes read and bytes written for the shift;
* one additional byte written for the displaced head entry.

The logical tail movement is therefore

\boxed{B_{\mathrm{tail}}(q)=2(q-32)+1=2q-63}

excluding the separate read of the selected tail byte and cache-line effects.

Since ,

\boxed{B_{\mathrm{tail}}\le 2A-65.}

For the measured Wikidata alphabet range:

Used alphabet	Largest possible high rank	Maximum tail read/write bytes
79	78	93
90	89	115
102	101	139

By comparison, an unrestricted 256-symbol alphabet permits 447 logical tail read/write bytes.

So 139 bytes, not 447, is the relevant per-event upper bound for these Wikidata blocks.

2. Convert that to decoded-byte traffic

The census gives approximately 1.6% high-rank fallbacks among explicit Wikidata tokens, and 0.061 explicit tokens per decoded byte.

Using the conservative maximum of 139 bytes per fallback:

0.016\times139=2.224

logical tail read/write bytes per explicit token, averaged across all explicit tokens.

Converting to decoded output bytes:

\boxed{
0.061\times2.224\approx0.136\text{ B/B}.
}

This excludes selected-symbol loads, register-head operations, and any save/restore around calls.

It’s an upper bound under the measured alphabet and rank-frequency assumptions, not measured cache or DRAM traffic. It also does not bound the instruction overhead of entering memmove.

Nevertheless, the result matters: high-rank fallback’s intrinsic tail data movement is small compared with the 4 B/B written by the shipped T-vector build alone.

3. P74 also moves less list data than the shipped fallback

For rank , the shipped implementation effectively shifts list bytes in memory, then writes the selected symbol at the front.

Ignoring the selected-symbol read, that amounts to approximately

B_{\mathrm{shipped}}(q)=2q+1

logical list-movement bytes.

P74 shifts only bytes in its memory tail and inserts the displaced head byte:

B_{\mathrm{P74}}(q)=2q-63.

The difference is exact at this accounting boundary:

\boxed{
B_{\mathrm{shipped}}(q)-B_{\mathrm{P74}}(q)=64\text{ bytes}.
}

P74 avoids moving 32 bytes of list data through memory on each high-rank update by handling that portion in its register-held head.

That does not make the high-rank path free. Reconstructing the head, extracting the selected byte and handling a possible call boundary still cost instructions.

But it establishes that the fallback is not merely a correctness concession: it has a genuine memory-work advantage over the shipped fallback.

4. What this closes—and what it doesn’t

Question	Finding
Does P74 need to move up to 223 tail bytes on Wikidata?	No; at most 69 shifted bytes plus one insertion
Maximum logical tail movement on measured Wikidata blocks	139 bytes per high-rank event
Conservative average across explicit tokens	≤2.224 B per explicit token
Conservative average across decoded bytes	≈0.136 B/B
Memory-movement advantage against shipped high-rank fallback	64 logical bytes/event
Does this establish cheaper fallback CPU cycles?	No

This narrows the remaining fallback uncertainty: the dominant unknown is likely the fixed instruction and call-boundary overhead, not the number of tail bytes that must be moved. That last statement is a cost-model hypothesis, not a measurement.

Relay to Claude

Research round 38 — alphabet-bounded P74 fallback

For P74 high-rank q≥32, let j=q−32. The tail shift reads j bytes, writes j bytes, then writes the displaced H[31], for logical tail movement 2j+1=2q−63 bytes, excluding the selected-symbol read.

A block with A used symbols has q≤A−1, so movement is bounded by 2A−65.

Measured Wikidata A=79–102 therefore implies at most 93–139 logical bytes/event, rather than the unrestricted 447-byte maximum. At the observed 1.6% high-rank rate and 0.061 explicit tokens/output byte, the conservative traffic bound is approximately 0.136 B/output byte.

The shipped memory-resident fallback moves q list bytes plus a front write, yielding 2q+1 logical movement bytes under equivalent read/write accounting. P74’s tail movement is exactly 64 bytes less per high-rank event, excluding head-register operations and selected-byte loads.

This closes the alphabet-dependent tail-movement uncertainty. It does not establish lower cycles: possible memmove calls, ABI preservation, and instruction scheduling remain unresolved.

Round 38 result: we’ve replaced a 256-symbol theoretical worst case with a much tighter corpus-specific bound, and shown exactly how the register-held representation reduces high-rank memory movement.

I would now stop treating high-rank tail size as a serious unknown. The remaining evidence boundary is the actual compiled fallback sequence and its fixed costs.
````
