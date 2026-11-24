# Record 122: record 121 corrected against channel #30, and the question redone

*MONODROMY, 24 November 2026. Labels: proved / argued / measured / conjectured. No timing is claimed.*

## 1. Correction (premise break)

Record 121 took the scanner to do the Huffman decode, MTF and RLE2. Channel #30: the scanner does the Huffman decode only, for every block, on one thread. MTF, RLE2, the inversion and emission are the teams' work, and a team's leader does its block's MTF alone.

- **Withdrawn:** record 121 §5 and NEEDS M-28. Nothing after the bit parse sits in the scanner, so Proposition M moves nothing out of it.
- **Stands:** Proposition M itself (proved, checked). Its use changes: it lets a team's helpers share the leader's MTF. That is a serial stretch inside a team, not the 19%.
- **Stands, and is now the whole question:** record 121 §2. The serial 19% at 10 workers is the Huffman bit parse.

## 2. The bit parse inside one block (argued; sharpened)

The parse's state is (bit position, symbol count N). The table in force is selector[N div 50], so the count is part of the state, not a by-product.

- Two parses from one bit position with different N read the same codewords only while their selectors agree, and never merge: the counts stay different.
- A complete prefix code decodes any bit string, so a wrong (position, N) is never refuted from inside the chunk. Only the true state arriving from the left refutes it.
- So a chunk's state transformer has no small form: one trajectory per N, and N ranges over the block's symbols. Speculation multiplies work by the hypotheses kept. A cheaper first pass that only counted symbols would still read every code length, which is the parse.

**Verdict:** inside a block the Huffman parse is serial for a reason in the format, the 50-symbol table switch. I have no proof that nothing cleverer exists.

## 3. Across blocks the parse is not serial (proved exact; prior art)

Each block starts with a 48-bit magic number at an arbitrary bit offset. A second parser can start at the next occurrence.

- **Exact:** accept a candidate only when the parser of the block before it arrives exactly there after its end-of-block symbol. A false candidate is discarded then, and its work is wasted, never used.
- **False candidates:** if compressed bits behave as uniform, 2^(−48) per offset, about 3·10⁻⁵ per gigabyte.
- **Prior art:** this is how lbzip2 and pbzip2 find blocks. I assume the team knows it; I have not read why mibz2's scanner is single.

The serial residue per file is then one block's parse, not the file's.

## 4. What the Huffman scan meets that the teams recompute

| met by the scan, as counters | recomputed by | note |
|---|---|---|
| symbols S and non-run symbols, so the run count r | run building | r is the count of non-zero indices plus leading runs |
| a non-zero index followed at once by another non-zero index | nobody, until measured offline | **this count is h₁ exactly**: such a symbol is a run of width one |
| run widths, from the run digits | MTF and RLE2 | the width histogram, so thin and wide strip counts (record 120) |
| the selector of each group | nobody | a six-way labelling of the column by local statistics |

*Why the second row holds (proved):* a non-zero index changes the symbol and starts a run; run digits after it extend that run; so the run has width one iff the next symbol is not a run digit. The first symbol and the end of block need the obvious care.

**Use:** the scan can hand each team n, r, h₁ and the width histogram before the team starts, at a few counter updates per symbol. Any choice that depends on them (route per block, strip threshold, team size) needs no pass of its own. Whether a choice of that kind exists that pays is not shown here (conjectured).

**The limit stands:** nothing the scan meets gives output order.

## NEEDS

`NEEDS M-28b` (replaces M-28) Two figures on the six admitted blocks: the leader's MTF as a share of a block's wall time in a team of three; and the scanner's time per block against a team's time per block at 10 workers. Rules, fixed before data: leader MTF under 5% closes Proposition M's use; a scanner-to-team ratio under 1/(teams) says one more parser is enough.
