# Replacing medium rules with stored literals

The five-byte control at `ffb35cf50a90e259ee3852382df2247247c26020` generated 13,655,080 rules across 228 admitted large Wikidata blocks. At that control's cutoff every 6–16 byte product creates a parent record, assigns positions to its children and enters the reverse writer scan. The hybrid holds those words as little-endian literal values in an indexed array of 128-bit entries. Products of at most five bytes still fit in their descriptors, and longer products still use rules. This change leaves initial slots, widths, cuts, mark tracking and rotation intact.

For each medium product, both children are short literals. Reading their two values and shifting the right value left by eight times the left word length gives their concatenation. High padding is zero, so a bitwise OR is sufficient. Only actually written bytes are visible to the block decoder. The largest shift comes from a left word of fifteen bytes and fits the 128-bit operation. The full-period owner test now applies to children longer than sixteen. A repeated rule child necessarily creates another stored parent record at the immediately following cut. The cyclic literal writer still handles the single factor covering the boundary.

All 228 large-block outputs match Segmented. The 26,292,727 cuts, letter counts, marked primary and admissions agree with the packed control on every large block. The hybrid built 7,895,004 rules and 5,760,076 medium literal records. Frozen built 28,706 rules and 19,478 medium literals. Each replaced rule had the same 16-byte construction store as the new literal, so the measured structural saving is in writer visits and position updates. Induction may now need to fetch one or two medium children before a new literal can be created. Separate reusable vectors can retain both high-water capacities. The worker timings and RSS decide this tradeoff. [Control geometry](../geometry.csv) · [Hybrid geometry](geometry.csv)

The directly buildable scorer entry is `research/astra-e/cells/hybrid/candidate.rs`. The patch rebuilds it byte for byte from the packed control. From a research checkout with the corpus prepared:

```sh
research/astra-e/cells/hybrid/prepare.sh target/astra-e-hybrid
cmp target/astra-e-hybrid/candidate.rs research/astra-e/cells/hybrid/candidate.rs
bash research/astra-e/score.sh packed-hybrid-round1 research/astra-e/cells/hybrid/candidate.rs --with-opend
```

The generated candidate's SHA-256 is `92f0b650c75702bebaa87c7243f114992fd58db3999410cbd7627176e5110b54`. To repeat the property and geometry suites, use the parent directory's commands with `MOONSHOT_CANDIDATE=target/astra-e-hybrid/candidate.rs`.
