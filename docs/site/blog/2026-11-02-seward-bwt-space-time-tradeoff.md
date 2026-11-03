---
title: "Revisiting Julian Seward's space–time tradeoff in BWT inversion"
description: "From Seward's 2001 inverse-BWT algorithms to a compact navigation table that finally passes a four-core traversal test: why more work per lookup can mean less work for the machine."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Revisiting Julian Seward's space–time tradeoff in BWT inversion

In 2001, Seward showed why the fastest way to undo a Burrows–Wheeler transform could use *more* instructions but *less* memory traffic. Twenty-five years later, that observation led us to test a smaller navigation table under contention.
{: .post-dek }

2 November 2026 · Research article
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

In 2001, [Julian Seward](https://en.wikipedia.org/wiki/Julian_Seward), the author of [bzip2](https://en.wikipedia.org/wiki/Bzip2) and original designer and author of [Valgrind](https://valgrind.org/info/developers.html), was working at **Microsoft Research in Cambridge**. His paper at the Data Compression Conference, [*Space-time Tradeoffs in the Inverse B-W Transform*](https://doi.org/10.1109/DCC.2001.917175), begins from an unfashionable question. Compression algorithms had attracted extensive work; what about the time and memory needed to *undo* the transformation?

In its introduction he sets out a limit that has stayed with our investigation. His examples were practical ones: decompressing a Linux kernel at boot with limited memory, or decompressing disk data quickly enough to keep up with its use:

> we argue that the maximum speed of BWT decompression is unavoidably constrained by the rate at which cache misses are serviced.

This is not a casual observation about fast hardware. The inverse Burrows–Wheeler transform has to follow a chain of positions, and the address of the next position is learned by reading the current one. If the table is larger than the useful cache, those dependent reads dominate. A fast arithmetic loop cannot run ahead of an address it does not yet know.

Seward's paper explores two apparently opposed ambitions: make each inversion as fast as possible, or use as little memory as possible while remaining acceptably fast. His implementations range from a five-byte-per-character arrangement to one that keeps just the transformed bytes and reconstructs the links as needed. What makes the paper worth revisiting is that its fastest design, `mergedTL`, **uses fewer table bytes and executes more instructions** than its immediate predecessor.

That old result eventually led us to a newer question, recorded in our [October 2026 research discussion](https://github.com/lmmx/mibz2/blob/f384fb424b19e5e5e1c5e27559dd0eb916e98123/docs/journal/2026-10-29-frontier/seward-frontier-research.md): what if a compact table is slower when one block is decoded alone, but faster when several blocks compete for the same cache? After an unsuccessful first prototype and several revisions, we have a first positive traversal result. To understand what it means, we should first understand what Seward was actually inverting.

## From the sorted rotations to the transformation vector

The Burrows–Wheeler transform (BWT) rearranges the bytes of a block into an order that is easier for later compression stages to encode. It does not, by itself, make the block shorter. The elementary construction is an illuminating thought experiment, even though a real compressor does not need to materialise the enormous matrix it describes.

Take the seven-character string `banana$`, where `$` is a unique character that sorts before the letters. Form every cyclic rotation: move a prefix to the end, wrapping around. Sort these seven strings alphabetically. Seward calls the resulting **N by N matrix** `M`, where `N` is the number of bytes and each row is one rotation. At real block sizes that matrix would be extravagantly large; it is a way to understand the transform, not a table the decompressor actually builds. The original string `S` is one of its rows, at position `I`.

<figure class="bf-fig">
<p class="bf-title">From sorted rotations to the first and last columns</p>
<svg role="img" aria-label="The sorted rotations of banana dollar are shown in a seven-row matrix. F is $aaabnn, L is annb$aa, the original string is row 4, and T has entries 1,5,6,4,0,2,3." viewBox="0 0 720 384"><rect x="58" y="35" width="76" height="304" rx="5" class="bf-cell-hi" /><rect x="520" y="35" width="74" height="304" rx="5" class="bf-cell-hi" /><text x="22" y="22" class="bf-ts">j</text><text x="96" y="22" class="bf-ts" text-anchor="middle">F</text><text x="328" y="22" class="bf-ts" text-anchor="middle">M[j], sorted rotations</text><text x="556" y="22" class="bf-ts" text-anchor="middle">L</text><text x="652" y="22" class="bf-ts" text-anchor="middle">T[j]</text><text x="24" y="62" class="bf-t-strong">0</text><text x="96" y="62" class="bf-t-strong" text-anchor="middle">$</text><text x="298" y="62" class="bf-t-strong" text-anchor="middle">$banana</text><text x="556" y="62" class="bf-t-strong" text-anchor="middle">a</text><text x="650" y="62" class="bf-t-strong" text-anchor="middle">1</text><path d="M7 74L704 74" class="bf-rule" /><text x="24" y="101" class="bf-t-strong">1</text><text x="96" y="101" class="bf-t-strong" text-anchor="middle">a</text><text x="298" y="101" class="bf-t-strong" text-anchor="middle">a$banan</text><text x="556" y="101" class="bf-t-strong" text-anchor="middle">n</text><text x="650" y="101" class="bf-t-strong" text-anchor="middle">5</text><path d="M7 113L704 113" class="bf-rule" /><text x="24" y="140" class="bf-t-strong">2</text><text x="96" y="140" class="bf-t-strong" text-anchor="middle">a</text><text x="298" y="140" class="bf-t-strong" text-anchor="middle">ana$ban</text><text x="556" y="140" class="bf-t-strong" text-anchor="middle">n</text><text x="650" y="140" class="bf-t-strong" text-anchor="middle">6</text><path d="M7 152L704 152" class="bf-rule" /><text x="24" y="179" class="bf-t-strong">3</text><text x="96" y="179" class="bf-t-strong" text-anchor="middle">a</text><text x="298" y="179" class="bf-t-strong" text-anchor="middle">anana$b</text><text x="556" y="179" class="bf-t-strong" text-anchor="middle">b</text><text x="650" y="179" class="bf-t-strong" text-anchor="middle">4</text><path d="M7 191L704 191" class="bf-rule" /><rect x="2" y="194" width="710" height="33" rx="5" class="bf-band" /><text x="24" y="218" class="bf-t-strong">4</text><text x="96" y="218" class="bf-t-strong" text-anchor="middle">b</text><text x="298" y="218" class="bf-t-strong" text-anchor="middle">banana$</text><text x="556" y="218" class="bf-t-strong" text-anchor="middle">$</text><text x="650" y="218" class="bf-t-strong" text-anchor="middle">0</text><path d="M7 230L704 230" class="bf-rule" /><text x="24" y="257" class="bf-t-strong">5</text><text x="96" y="257" class="bf-t-strong" text-anchor="middle">n</text><text x="298" y="257" class="bf-t-strong" text-anchor="middle">na$bana</text><text x="556" y="257" class="bf-t-strong" text-anchor="middle">a</text><text x="650" y="257" class="bf-t-strong" text-anchor="middle">2</text><path d="M7 269L704 269" class="bf-rule" /><text x="24" y="296" class="bf-t-strong">6</text><text x="96" y="296" class="bf-t-strong" text-anchor="middle">n</text><text x="298" y="296" class="bf-t-strong" text-anchor="middle">nana$ba</text><text x="556" y="296" class="bf-t-strong" text-anchor="middle">a</text><text x="650" y="296" class="bf-t-strong" text-anchor="middle">3</text><text x="14" y="372" class="bf-tm">I = 4 is the original string banana$; each row is a cyclic rotation.</text></svg>
<figcaption>M contains all seven sorted rotations. The highlighted row is the original string at <code>I = 4</code>. Only F, L and this row index are needed to reverse the transform; T is calculated from the order of repeated characters.</figcaption>
</figure>

Here `N = 7`, `S = banana$`, and `I = 4` using zero-based row numbers. The **first column**, `F`, is `$aaabnn`; the **last column**, `L`, is `annb$aa`. The last column, together with `I`, is the BWT representation from which inversion begins. The `$` marker is useful for this example; Seward's paper formulates inversion using the primary row index `I`, without requiring this sentinel.

Why do only those two columns matter when the matrix has seven? Every row is a rotation of the same string. They contain precisely the same characters, merely in different orders. **F is L sorted alphabetically**, so we can recover F immediately from L without keeping the whole matrix. But sorting by itself does not tell us how to reconstruct the order of the original text. For that we need to recover which occurrence of a letter in L corresponds to which occurrence in F.

Seward calls that relationship the *transformation vector* `T` (his §2, pages 2–3). He was expanding [Algorithm D in §2 of Burrows and Wheeler’s original 1994 paper](https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf), not proposing a different transformation: the extra proof makes several practical inverse algorithms possible. Suppose row `j` of M ends in a character `c`. Move that last character to the front: the resulting rotation must also be a row of M. **`T[j]` is the row number of that rotated string.** In other words, `T` tells us how to move one position backward through the original cyclic text.

For `banana$`, row 5 is `na$bana`. Rotating it one position right gives `ana$ban`, row 2. So `T[5] = 2`. The complete vector is:

```text
row j:   0  1  2  3  4  5  6
L[j]:    a  n  n  b  $  a  a
T[j]:    1  5  6  4  0  2  3
```

This is the same sort of navigation table mibz2 calls its **T-vector**. Once a current row is known, looking up its next row is a simple array access. The price is having to store the array and read it in an order that often looks random to the memory system.

### Why matching the same occurrence works

The clever part is that `T` need not be derived by building and comparing rotations. Seward's §2.3 establishes the relevant ordering property: **the first `a` in L corresponds to the first `a` in F; the second to the second; and likewise for every other character**.

Why? All the rows of M are alphabetically sorted. Among rows whose first character is the same, their order is determined by the remaining characters. Moving that common first character to the end preserves the relative order of those tails. So when the occurrences of a given character move from the last column back to the first, their *relative order* is unchanged. This is the stable first-to-last correspondence, usually called the **LF mapping**.

<figure class="bf-fig">
<p class="bf-title">The same occurrence, in a different column</p>
<svg role="img" aria-label="L and F show seven indexed character slots each. The second a in L at row 5 is connected to the second a in F at row 2." viewBox="0 0 720 236"><text x="12" y="50" class="bf-ts">L: last column</text><text x="12" y="146" class="bf-ts">F: first column</text><rect x="190" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="218" y="45" class="bf-t-strong" text-anchor="middle">a</text><text x="218" y="76" class="bf-tm" text-anchor="middle">0</text><rect x="190" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="218" y="138" class="bf-t-strong" text-anchor="middle">$</text><text x="218" y="168" class="bf-tm" text-anchor="middle">0</text><rect x="257" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="285" y="45" class="bf-t-strong" text-anchor="middle">n</text><text x="285" y="76" class="bf-tm" text-anchor="middle">1</text><rect x="257" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="285" y="138" class="bf-t-strong" text-anchor="middle">a</text><text x="285" y="168" class="bf-tm" text-anchor="middle">1</text><rect x="324" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="352" y="45" class="bf-t-strong" text-anchor="middle">n</text><text x="352" y="76" class="bf-tm" text-anchor="middle">2</text><rect x="324" y="113" width="56" height="40" rx="6" class="bf-cell-hi" /><text x="352" y="138" class="bf-t-strong" text-anchor="middle">a</text><text x="352" y="168" class="bf-tm" text-anchor="middle">2</text><rect x="391" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="419" y="45" class="bf-t-strong" text-anchor="middle">b</text><text x="419" y="76" class="bf-tm" text-anchor="middle">3</text><rect x="391" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="419" y="138" class="bf-t-strong" text-anchor="middle">a</text><text x="419" y="168" class="bf-tm" text-anchor="middle">3</text><rect x="458" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="486" y="45" class="bf-t-strong" text-anchor="middle">$</text><text x="486" y="76" class="bf-tm" text-anchor="middle">4</text><rect x="458" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="486" y="138" class="bf-t-strong" text-anchor="middle">b</text><text x="486" y="168" class="bf-tm" text-anchor="middle">4</text><rect x="525" y="20" width="56" height="40" rx="6" class="bf-cell-hi" /><text x="553" y="45" class="bf-t-strong" text-anchor="middle">a</text><text x="553" y="76" class="bf-tm" text-anchor="middle">5</text><rect x="525" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="553" y="138" class="bf-t-strong" text-anchor="middle">n</text><text x="553" y="168" class="bf-tm" text-anchor="middle">5</text><rect x="592" y="20" width="56" height="40" rx="6" class="bf-cell" /><text x="620" y="45" class="bf-t-strong" text-anchor="middle">a</text><text x="620" y="76" class="bf-tm" text-anchor="middle">6</text><rect x="592" y="113" width="56" height="40" rx="6" class="bf-cell" /><text x="620" y="138" class="bf-t-strong" text-anchor="middle">n</text><text x="620" y="168" class="bf-tm" text-anchor="middle">6</text><path d="M553 82 V96 H352 V106" class="bf-cut"/><text x="20" y="204" class="bf-ts">2nd &#x27;a&#x27; in L maps to 2nd &#x27;a&#x27; in F</text><text x="20" y="226" class="bf-tm">L[5] = a   →   T[5] = 2, because F[2] = a</text></svg>
<figcaption>The second <code>a</code> in L is at index 5; the second <code>a</code> in sorted F is at index 2. This rank-preserving match tells us <code>T[5] = 2</code>.</figcaption>
</figure>

In the example, `L[5]` is the **second** `a` in L. The second `a` in F occupies row 2. Therefore `T[5] = 2`, without ever constructing M. We can compute the same relationship for every row in one sequential scan.

Seward uses a little array `C` for byte counts and, after accumulating them, the starting position of each character's run in F. For any character `c`, let `C[c]` be the number of characters in L strictly smaller than `c`. Let `rank(c, j)` count how often `c` occurs in `L[0..j]`, including position `j`. Then the rule is:

\[
T[j] = C[L[j]] + \operatorname{rank}(L[j],j) - 1.
\]

For example, the `a` bucket in F begins at row 1, so `C['a'] = 1`. There are two `a`s in L up through position 5. Hence `T[5] = 1 + 2 - 1 = 2`. This is the foundation of every algorithm in the paper: one array lookup if we **store** `T`; character counts and a rank query if we **reconstruct** it.

Finally, how do we recover the original text? The row `I` is the original rotation. Its last-column byte is the final byte of S. Read `L[I]`, then move to `T[I]`, and repeat. For this example:

```text
rows visited:  4 → 0 → 1 → 5 → 2 → 6 → 3
bytes read:    $   a   n   a   n   a   b
```

Those bytes spell `$ananab`, the original text **backward**. Reversing them restores `banana$`. In his §2.2 Seward notes the practical difficulty: if a decoder produces the block backward, it has to retain the block until it can emit it in the correct order. The paper's later forward-output algorithms assume that the compressor reverses the input before the BWT, so the inverse can emit bytes forwards. The paper also describes a pointer-reversal alternative when that preparation has not been done. This distinction is part of the implementation cost, not a mathematical defect in the LF mapping.

We can now put the huge matrix aside. The inverse problem is reduced to **L, the starting row I, and a way to evaluate T at the current row**. Seward's remaining algorithms differ in how much of that information they store and how much work they do to recover it.

## Making inversion fast: `basis`, `bw94` and `mergedTL`

Seward's §3 (pages 3–4) starts with the direct consequence of his proof, then improves it twice. Each version reconstructs the same sequence. Their differences concern the *construction passes*, the *data kept while walking*, and the *number of unpredictable memory reads* per output byte.

**`basis`** is the straightforward implementation. It first scans L to count the frequency of every byte and turns those counts into the starting positions of the byte runs in F. It scans L a second time to construct T, using the occurrence ranks we just derived. Then it traverses T from I, taking the output character from L at the current row. Thus it makes **three passes over L**, with L occupying N bytes and T occupying 4N bytes in the implementation used for his experiments. That is **5N bytes** of navigation storage, excluding the output and small count arrays.

One extra pass can be saved. In **`bw94`**—a reference to Burrows and Wheeler's 1994 algorithm—Seward first writes, for every row, the occurrence number of L's character *within that character's bucket*, rather than its final absolute position in F. Call that local number `T′[j]`. He can record it while counting, because the final bucket starts are not yet needed. After the counts have been accumulated, the walk recovers the absolute next row as:

\[
p_{\mathrm{next}}=T'[p]+C[L[p]].
\]

For our example, `T′[5] = 1` (zero-based: the second `a`), and `C['a'] = 1`, yielding the same successor row 2. This saves a construction pass; it still stores the byte in L and the local offset in T′ separately. Every traversal step must fetch both at the current row.

Then comes **`mergedTL`**, the design Seward says was used by bzip2 itself. In `bw94`, the walk reads L[p] and T′[p] at the same unpredictable index p. If those arrays have fallen out of cache, one useful output byte can require *two* costly reads. Why not put both values together? `mergedTL` packs the character in the low eight bits of a 32-bit word and the local successor index in the upper 24:

\[
U[p] = 256\,T'[p] + L[p].
\]

Now one indexed read supplies both pieces. The walk extracts the output byte using the low eight bits, uses the upper bits for the local successor, and adds the byte's bucket start. Division by 256 and remainder by 256 compile to shifts and masks on this representation. The table becomes **4N bytes**, down from the 5N bytes of the separate-array versions.

<figure class="bf-fig">
<p class="bf-title">Store the character and successor separately—or together</p>
<svg role="img" aria-label="Horizontal memory diagrams comparing L and T separate at five bytes per row, merged U at four, packed A at two and a half, and treeopt at one and a half." viewBox="0 0 720 285"><text x="6" y="44" class="bf-ts">basis / bw94</text><rect x="192" y="28" width="86" height="32" rx="5" class="bf-slot" /><text x="235.0" y="48" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">L (1N)</text><rect x="278" y="28" width="344" height="32" rx="5" class="bf-cell-hi" /><text x="450.0" y="48" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">T or T′ (4N)</text><text x="630" y="47" class="bf-tm">5.0N</text><text x="6" y="106" class="bf-ts">mergedTL</text><rect x="192" y="90" width="344" height="32" rx="5" class="bf-cell-hi" /><text x="364.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">packed U: byte + offset</text><text x="630" y="109" class="bf-tm">4.0N</text><text x="6" y="168" class="bf-ts">indexF</text><rect x="192" y="152" width="215.0" height="32" rx="5" class="bf-cell-hi" /><text x="299.5" y="172" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">packed successor A</text><text x="630" y="171" class="bf-tm">2.5N</text><text x="6" y="230" class="bf-ts">treeopt-500k</text><rect x="192" y="214" width="86.0" height="32" rx="5" class="bf-slot" /><text x="235.0" y="234" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">L</text><rect x="278.0" y="214" width="43.0" height="32" rx="5" class="bf-cell-hi" /><text x="299.5" y="234" class="bf-t-strong" text-anchor="middle" style="font-size: 11px">count-tree nodes</text><text x="630" y="233" class="bf-tm">1.5N</text><text x="190" y="276" class="bf-tm">Memory per N-byte block (output array excluded); schematic proportions.</text></svg>
<figcaption>Seward’s experimental memory totals for a one-million-byte block: <code>basis</code> and <code>bw94</code> retain L and T separately; <code>mergedTL</code> keeps both values in one 32-bit word; <code>indexF</code> saves the L bytes and reconstructs them from F. The tree example uses L plus 500 kB of auxiliary counts.</figcaption>
</figure>

Notice the peculiar achievement. `mergedTL` involves extra packing and unpacking instructions, but reduces the amount of randomly addressed data, and can replace two separate unpredictable reads with one. The original question was not *how few instructions can we execute?* It was *how quickly can we produce the next useful byte when memory is expensive?* Seward's §6 would show why this distinction mattered.

## Spending less memory: `indexF` and the trees

Section 4 of the paper asks whether the table can become smaller still without making inversion uselessly slow. Its crucial move is to ask **which column we recover the output character from**.

The fast variants use L[p] because L is supplied by the transform and directly addressable. But the theory gives us the equivalent predecessor relationship through F. Unlike L, **F is sorted**. We do not need a full byte array to recover the character at row p in F: the 256 bucket boundaries in C tell us which character occupies that position.

In **`indexF`**, the decoder stores T in an array A of **20-bit integers**, using the paper's maximum block size of \(2^{20}\) positions. Initially A holds the supplied L bytes. The builder counts the bytes, saves the cumulative counts, and then overwrites those initial bytes with the T values. During traversal, the successor is simply `A[p]`; the character is recovered from the position p in F by a binary search in the 256-entry cumulative-count table. A binary search of 256 buckets takes at most nine comparisons in Seward's account.

This explains its name: **`indexF` means index into the first column** to recover the character instead of retaining and indexing the last column. It is not a new method for finding T. It is a way of discarding L after T has been built, while recovering output from the much more compact description of F.

Seward also shifts the initial position by one T step, because reading F at the original row begins at a different place in the cyclic order from reading L there. His §4 explains that the starting row is `T[I]`. If the compressor has not reversed the input in advance, the traversal naturally produces the text backwards. Seward describes reversing the links along T *in place* before the output pass, so the original text can still be emitted forwards without keeping a second full-size output buffer. That extra pass is another real time cost of a smaller in-memory representation.

The paper's packed representation uses N 16-bit values plus another approximately N/2 bytes for the upper four bits: **2.5N bytes** rather than 4N. The trade-off is repeated character lookup during inversion instead of a byte already carried in the navigation word.

Section 5 goes further. If we can compute T[j] from the rank of L[j], why store any T array at all? In principle we need only L and the small frequency table. To follow a link from row j, count all occurrences of L[j] in L up to j, then add its bucket start in F. That can be painfully slow: scanning from the beginning for each of N links makes the most naive method **quadratic in N**. But it gives a useful extreme: nearly **one byte of navigation storage per input byte**, if we are willing to recompute almost everything.

The intermediate solutions use trees of partial counts. Imagine a tree for each possible byte value: **256 trees**, although many may remain tiny because their byte scarcely occurs in L. A node represents an interval of positions in L and stores how many times its character occurs there. Its two children represent the left and right halves. A subtree for an interval that contains no copies of the character need not be stored at all.

The query now becomes a journey down one tree. To count how many copies of c occur up through row j, begin at the root. Whenever the requested prefix includes a complete left child, add that child's stored count and continue to the right. When the tree stops, scan the unresolved tail of L directly. More nodes mean smaller remaining scans; fewer nodes save memory but make each navigation step slower. **The tree holds answers to parts of the rank query, not the successor array itself.**

Building this structure is a separate problem. Repeatedly dividing the intervals of each character one at a time would scan L up to 256 times per level. Seward instead **deepens the trees together**: at a given depth, a pass over L counts characters in intervals of the appropriate size, allowing all relevant nonempty nodes to be formed from one scan. He uses a power-of-two extent Q at least as large as N so that every root-to-node path identifies an interval unambiguously; the notional positions beyond the real L contain zero characters. The construction has more passes and more bookkeeping than the original T-vector build, but it keeps the *inversion* memory adjustable through a fixed node budget.

<figure class="bf-fig">
<p class="bf-title">Answer a rank query without keeping every successor</p>
<svg role="img" aria-label="A count tree divides L positions zero to seven into halves. There is one a in positions 0 to 3 and two in 4 to 7; position range 4 to 5 contains one a. A prefix query to row 5 totals two a characters." viewBox="0 0 720 300"><text x="14" y="23" class="bf-ts">L = a n n b $ a a</text><text x="430" y="23" class="bf-ts">rank(&#x27;a&#x27;, 5) = 2</text><path d="M435 97L262 135" class="bf-arrow" /><path d="M435 97L556 135" class="bf-arrow" /><path d="M556 170L473 210" class="bf-arrow" /><path d="M556 170L642 210" class="bf-arrow" /><rect x="358" y="63" width="160" height="34" rx="6" class="bf-cell" /><text x="438.0" y="85" class="bf-t-strong" text-anchor="middle" style="font-size: 12px">0–7 : 3 a’s</text><rect x="187" y="135" width="150" height="35" rx="6" class="bf-cell-hi" /><text x="262.0" y="157" class="bf-t-strong" text-anchor="middle" style="font-size: 12px">0–3 : 1 a</text><rect x="482" y="135" width="150" height="35" rx="6" class="bf-cell" /><text x="557.0" y="157" class="bf-t-strong" text-anchor="middle" style="font-size: 12px">4–7 : 2 a’s</text><rect x="398" y="210" width="150" height="35" rx="6" class="bf-cell-hi" /><text x="473.0" y="232" class="bf-t-strong" text-anchor="middle" style="font-size: 12px">4–5 : 1 a</text><rect x="571" y="210" width="142" height="35" rx="6" class="bf-cell" /><text x="642.0" y="232" class="bf-t-strong" text-anchor="middle" style="font-size: 12px">6–7 : 1 a</text><text x="10" y="275" class="bf-tm">For a prefix ending at row 5, add the highlighted subrange counts: 1 + 1 = 2.</text><text x="10" y="293" class="bf-tm">If the tree stops earlier, the decoder scans the unresolved part of L.</text></svg>
<figcaption>A conceptual count tree for the three <code>a</code>s in <code>L = annb$aa</code>. Highlighted subranges cover the prefix ending at row 5. A sparse tree may omit further subdivision and scan the remaining interval instead.</figcaption>
</figure>

For our `banana$` example, L is `annb$aa`: there is one `a` in positions 0–3 and two in positions 4–6. To find the rank of the `a` at position 5, we can count the complete left half (one `a`) and the part of the right half covering positions 4–5 (one more). That recovers rank 2—and therefore successor T[5] = 2—without storing T[5] explicitly. A deeper tree makes the remaining scan shorter. A shallower tree uses less memory, but makes us inspect more of L at each step.

Seward's **`tree`** stores each node naively as a count and two 32-bit child pointers: 12 bytes per node. **`treeopt`** saves space through more compact internal nodes and by giving leaves only one 32-bit word, since leaves have no children to point to. With the same storage allowance, it can keep approximately twice as many nodes. This makes the trees deeper and the final scans shorter; in the paper's measurements that is worth much more than the additional work of managing the compact tree representation.

The sophistication here is easy to miss. The tree's role is not to reorganise S, or to compress it again. It is to answer one recurring query quickly enough: **how many times has this particular character appeared so far in L?** Each saved count avoids rescanning bytes; each omitted count gives memory back at the price of more searching. Seward is building an entire spectrum between storing the successor of every row and recomputing successors from the text.

## What Seward measured

His §6 tests **seven configurations**: `basis`, `bw94`, `mergedTL`, `indexF`, `tree-1M`, `treeopt-1M` and `treeopt-500k`. The machine was a 400 MHz AMD K6-III with 128 MB of memory, running Linux 2.0.36. Each block's L column was read from disk, and then the block was inverted ten times; the recovered output was written to an array but not sent to an output stream. This was a measurement of the inverse transform, not full bzip2 decompression.

Seward's **Table 1** begins with the storage cost for a one-million-byte block. These figures count the navigation working storage, excluding the final output and small byte-frequency tables. The names now have concrete meanings: each is a different answer to the question of how to find the next row.

| Algorithm | Storage for N = 10⁶ | What it keeps or recomputes |
| --- | ---: | --- |
| `basis` | 5.0 MB | L and absolute T, separately |
| `bw94` | 5.0 MB | L and bucket-relative T′, separately |
| **`mergedTL`** | **4.0 MB** | Byte and bucket-relative successor in one 32-bit word |
| `indexF` | 2.5 MB | Packed T; output byte recovered from sorted F's boundaries |
| `tree-1M` | 2.0 MB | L and a 1 MB tree of partial character counts |
| `treeopt-1M` | 2.0 MB | L and a more economical 1 MB count tree |
| `treeopt-500k` | 1.5 MB | L and only 0.5 MB of count-tree storage |

His **Table 2** identifies the inputs. They were deliberately varied, and none is a controlled analogue of our contemporary 434-block Wikidata corpus.

| Input | Approximate size in paper | What it contains |
| --- | ---: | --- |
| `bible` | 4,047 kB | The King James Bible, plain English text |
| `world192` | 2,473 kB | A CIA World Factbook text, not a Wikidata dump |
| `ecoli` | 4,639 kB | An *E. coli* genome; only four alphabet symbols in Seward's analysis |
| `kennedy` | 1,030 kB | The Canterbury corpus's **`kennedy.xls` Excel spreadsheet**, not speeches or other prose |
| `netscape` | 7,679 kB | A Netscape Navigator x86 executable for Linux |
| `bashtgz` | 1,709 kB | A gzipped archive of GNU bash source code |

The first four names come from the Canterbury collections; the [corpus descriptions](https://corpus.canterbury.ac.nz/descriptions/) identify the Bible, the genome, the factbook and the spreadsheet. The source types matter: `ecoli` is not a random-byte benchmark, and `world192` shares the subject of country information with Wikidata without sharing the latter's format, size or workload. We should not use either as a direct substitute for our own data.

His **Table 3** reports the rate of reconstructing the original block from L, in megabytes per second. All seven algorithms were tested on all six inputs. These are Seward's published figures, not mibz2 benchmark data:

| Input | `basis` | `bw94` | `mergedTL` | `indexF` | `tree-1M` | `treeopt-1M` | `treeopt-500k` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bashtgz` | 1.54 | 1.58 | **2.49** | 1.20 | 0.032 | 0.059 | 0.032 |
| `bible` | 1.72 | 1.77 | **2.76** | 1.36 | 0.211 | 0.301 | 0.218 |
| `ecoli` | 1.53 | 1.57 | **2.48** | 1.30 | 0.325 | 0.342 | 0.339 |
| `kennedy` | 2.46 | 2.54 | **4.72** | 1.71 | 0.099 | 0.167 | 0.100 |
| `netscape` | 2.32 | 2.41 | **3.51** | 1.67 | 0.044 | 0.099 | 0.036 |
| `world192` | 1.65 | 1.69 | **2.65** | 1.31 | 0.207 | 0.298 | 0.198 |
| **Mean** | **1.86** | **1.93** | **3.10** | **1.43** | **0.153** | **0.211** | **0.154** |

Plotting Table 3's means against Table 1's storage reveals the discrete choices Seward was studying:

<figure class="bf-fig">
<p class="bf-title">Seward’s measured speed–memory choices, reconstructed</p>
<svg role="img" aria-label="Scatter plot with average inversion rate versus navigation memory. mergedTL at 4 MB and 3.10 MB/s outperforms basis and bw94 at 5 MB and around 1.9 MB/s; indexF at 2.5 MB and 1.43 MB/s; three tree variants below 0.22 MB/s at 1.5 to 2.0 MB." viewBox="0 0 720 395"><path d="M85 335.0L681 335.0" class="bf-grid" /><text x="78" y="340.0" class="bf-tm" text-anchor="end">0</text><path d="M85 256.14285714285717L681 256.14285714285717" class="bf-grid" /><text x="78" y="261.14285714285717" class="bf-tm" text-anchor="end">1</text><path d="M85 177.2857142857143L681 177.2857142857143" class="bf-grid" /><text x="78" y="182.2857142857143" class="bf-tm" text-anchor="end">2</text><path d="M85 98.42857142857144L681 98.42857142857144" class="bf-grid" /><text x="78" y="103.42857142857144" class="bf-tm" text-anchor="end">3</text><path d="M157.22222222222223 43L157.22222222222223 339" class="bf-grid" /><text x="157.22222222222223" y="356" class="bf-tm" text-anchor="middle">1.5</text><path d="M281.66666666666663 43L281.66666666666663 339" class="bf-grid" /><text x="281.66666666666663" y="356" class="bf-tm" text-anchor="middle">2.5</text><path d="M406.11111111111114 43L406.11111111111114 339" class="bf-grid" /><text x="406.11111111111114" y="356" class="bf-tm" text-anchor="middle">3.5</text><path d="M530.5555555555555 43L530.5555555555555 339" class="bf-grid" /><text x="530.5555555555555" y="356" class="bf-tm" text-anchor="middle">4.5</text><path d="M592.7777777777778 43L592.7777777777778 339" class="bf-grid" /><text x="592.7777777777778" y="356" class="bf-tm" text-anchor="middle">5.0</text><text x="10" y="20" class="bf-ts">Average inverse-BWT rate (MB/s)</text><text x="474" y="379" class="bf-tm">Navigation memory (MB, N = 10⁶)</text><circle cx="592.78" cy="188.33" r="6" class="bf-slot-hi"/><circle cx="592.78" cy="182.81" r="6" class="bf-slot-hi"/><circle cx="468.33" cy="90.54" r="8" class="bf-green"/><circle cx="281.67" cy="222.23" r="6" class="bf-slot-hi"/><circle cx="219.44" cy="322.93" r="6" class="bf-slot-hi"/><circle cx="219.44" cy="318.36" r="6" class="bf-slot-hi"/><circle cx="157.22" cy="322.86" r="6" class="bf-slot-hi"/><text x="452.3333333333333" y="74.54285714285712" class="bf-t-strong" text-anchor="end">mergedTL 3.10</text><text x="269.66666666666663" y="208.23428571428573" class="bf-t-strong" text-anchor="end">indexF 1.43</text><text x="577.7777777777778" y="165.8057142857143" class="bf-t-strong" text-anchor="end">basis / bw94</text><text x="179.44444444444446" y="307.93485714285714" class="bf-tm" text-anchor="end">three tree variants</text><text x="14" y="386" class="bf-tm">Points are algorithms, not a smooth frontier. Each datum is from Seward’s Tables 1 and 3.</text></svg>
<figcaption>An original plot of the seven configurations from Seward’s **Table 1** (working memory) and **Table 3** (mean inverse-transform throughput). These are his 2001 measurements on a 400 MHz K6-III, not a figure reproduced from the paper and not mibz2 benchmark data. The exact values are tabulated above.</figcaption>
</figure>

`mergedTL` is fastest on **every input**. At the mean it is about **61% faster than `bw94`** and **67% faster than `basis`**, while needing *less* table memory than either. `indexF` uses 2.5 MB but runs at approximately 46% of `mergedTL`'s mean rate; the trees use less still but are dramatically slower. So this is **not** a smooth universal curve in which each saved byte imposes an inevitable speed penalty. These are seven particular algorithms on one machine. `mergedTL` improves both dimensions relative to the separate-array designs by changing how the next link is accessed.

In §6.3 Seward investigates why `mergedTL` beat `bw94` despite its packing instructions. He used **`cacheprof`**, his own earlier cache-profiling tool (cited in the paper's references), rather than assuming that fewer instructions were the whole story. This was distinct from the later Valgrind tool suite. For the `bible` input, his simulation of the K6-III's 256 kB second-level cache counted **20 instructions, eight memory references and 0.88 cache misses per byte** for `mergedTL`, versus **18 instructions, nine references and 1.63 misses** for `bw94`. That is about 11% more instructions but roughly 46% fewer simulated misses. It supports his explanation of why merging L and T improves speed; it is not a direct measurement of DRAM traffic.

He also examined anomalies rather than treating the mean as the whole story. The very fast `kennedy` result was plausibly associated with long runs of one character creating local stretches of the navigation permutation. Tree algorithms depended strongly on alphabet size and the amount of L that still had to be scanned. And Seward's suggested cache-miss ceiling was explicitly conditional: changes in cache capacity could make his old estimate inapplicable.

That last qualification brings the paper remarkably close to our question. If the cache matters so much for *one* block, what happens when several independent inversions share it?

## The same problem with several blocks alive

By October 2026 we had two observations in mibz2. The [workstation scaling experiments](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/journal/2026-10-28-workstation-scaling.md) had shown a throughput knee as the machine ran more independent block decodes: each standard 900,000-row navigation table occupies **3.6 MB**, so four live tables alone require 14.4 MB of navigation storage in a shared cache of nominally 20 MiB. Separately, a [census of real successor positions](2026-10-29-compact-navigation-tables.md) showed enough regularity to make substantially smaller representations plausible.

Our [October discussion of Seward's work](https://github.com/lmmx/mibz2/blob/f384fb424b19e5e5e1c5e27559dd0eb916e98123/docs/journal/2026-10-29-frontier/seward-frontier-research.md) brought these together. Seward had measured speed against memory for one inverse transform on his machine. We wanted to know if **the preferred representation could change with the number of simultaneous tables**. A compact lookup might take more instructions when it runs alone but avoid enough shared-cache pressure to become cheaper when four workers are active. The [throughput–memory frontier](../benchmarks/frontier.md) became a continuing question in the project, not a retrospective interpretation of one favourable result.

We tried a tightly packed 14-bit-offset format first, **FOR-14**. Its ordinary lookup path required **38 instructions and five data loads**, against **16 instructions and three loads** for the conventional table. The table used approximately **2.076 logical bytes per row**, but the walker was much slower: 2.676 ns per row in the isolated comparison against 1.275 for production, and 2.899 against 1.991 when four blocks were active. Although FOR-14's cost rose just **8%** when the four-way workload was imposed, while the control's rose **56%**, it had already paid far too much per lookup. Its [registered trial stopped](https://github.com/lmmx/mibz2/pull/130).

Two follow-up implementations reduced the compiled ordinary path to **31 and 27 instructions**, and passed correctness checks, but **were not timed**. They therefore cannot be counted as experimental stepping stones showing gradually improving speed. The negative result had nevertheless made the engineering problem concrete: keeping the table compact was not enough; we also had to keep the instructions for recovering each link cheap.

The next representation—called *compact30* in the research code—chose a slightly less dense record. Instead of squeezing 32 fourteen-bit offsets into one cache line and extracting them, it uses **thirty directly addressable 16-bit offsets** (60 bytes) and a **four-byte header** in a 64-byte aligned record. The header carries a shared base and character; exceptional groups use separate RAW records. The refined walker no longer carries the output character as lane state, and its offset and header loads can begin independently from the current row. Its reviewed ordinary path uses **21 instructions and four data loads**. The original FOR-14 implementation used 38; these are distinct designs, not before-and-after timings of one binary.

<figure class="bf-fig">
<p class="bf-title">The compact navigation record that reached the trial</p>
<svg role="img" aria-label="A 64-byte record has a 60-byte array of thirty 16-bit offsets and a four-byte header containing a base position and output byte. Exceptional groups use separately stored full entries." viewBox="0 0 720 182"><text x="5" y="24" class="bf-ts">One 64-byte aligned record</text><rect x="5" y="38" width="650" height="60" rx="5" class="bf-cell" /><rect x="610" y="38" width="45" height="60" rx="4" class="bf-cell-hi" /><text x="302" y="62" class="bf-t-strong" text-anchor="middle">30 directly addressable u16 offsets</text><text x="302" y="82" class="bf-tm" text-anchor="middle">60 bytes</text><text x="632" y="64" class="bf-t-strong" text-anchor="middle">H</text><text x="632" y="84" class="bf-tm" text-anchor="middle">4 B</text><text x="6" y="123" class="bf-t">H supplies a shared base position and output byte; a row supplies its offset.</text><text x="6" y="149" class="bf-tm">For exceptions (crossing or wide groups), a separate 128-byte RAW group stores full entries.</text></svg>
<figcaption>Thirty rows share one cache-line record. Two unused 16-bit slots from a 32-entry arrangement give room for a four-byte header, avoiding packed-bit extraction. This is the tested representation; the earlier 30-row census used an older reference encoding.</figcaption>
</figure>

The new format occupies **2.164 logical bytes per row** on our primary corpus, about **45.9% less than the conventional four-byte table**, though about 4.2% more than FOR-14. That small surrender of density was deliberate: the table had to be compact *and* straightforward to navigate. The initial 30-row census proved encoding feasibility and exact round trips on eleven corpora, using an earlier reference encoding. The final field layout, walker and disassembly were [refined and checked separately](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md).

When we finally timed the two prepared-table traversals, the result crossed the threshold that the previous design had missed:

| Traversal measurement | Conventional | Compact navigation | Compact / conventional |
| --- | ---: | ---: | ---: |
| One core: CPU nanoseconds per row | 1.221 | 1.402 | **1.148** — 14.85% more CPU |
| Four concurrent workers: CPU nanoseconds per row | 2.055 | 1.727 | **0.840** — 15.95% less CPU |
| Four-worker lockstep completion, summed steps | 114.45 ms | 97.97 ms | **0.856** — 14.40% less time |

<figure class="bf-fig">
<p class="bf-title">The measured crossover: isolated loss, contended benefit</p>
<svg role="img" aria-label="Grouped horizontal bars: conventional CPU cost is 1.221 ns per row alone and 2.055 with four; compact navigation is 1.402 alone and 1.727 with four." viewBox="0 0 720 271"><path d="M177 28L177 227" class="bf-grid" /><text x="177" y="243" class="bf-tm" text-anchor="middle">0</text><path d="M283.5 28L283.5 227" class="bf-grid" /><text x="283.5" y="243" class="bf-tm" text-anchor="middle">0.5</text><path d="M390 28L390 227" class="bf-grid" /><text x="390" y="243" class="bf-tm" text-anchor="middle">1</text><path d="M496.5 28L496.5 227" class="bf-grid" /><text x="496.5" y="243" class="bf-tm" text-anchor="middle">1.5</text><path d="M603 28L603 227" class="bf-grid" /><text x="603" y="243" class="bf-tm" text-anchor="middle">2</text><text x="5" y="21" class="bf-ts">CPU ns per BWT row (lower is better)</text><text x="8" y="67" class="bf-ts">One active walker</text><rect x="177" y="73" width="259.98993" height="26" rx="5" class="bf-slot" /><rect x="177" y="106" width="298.58766" height="26" rx="5" class="bf-cell-hi" /><text x="443.98993" y="91" class="bf-t-strong" style="font-size: 12px">1.221</text><text x="482.58766" y="125" class="bf-t-strong" style="font-size: 12px">1.402</text><text x="8" y="172" class="bf-ts">Four concurrent walkers</text><rect x="177" y="178" width="437.71287" height="26" rx="5" class="bf-slot" /><rect x="177" y="211" width="367.88721000000004" height="26" rx="5" class="bf-cell-hi" /><text x="621.7128700000001" y="196" class="bf-t-strong" style="font-size: 12px">2.055</text><text x="551.8872100000001" y="230" class="bf-t-strong" style="font-size: 12px">1.727</text><rect x="10" y="258" width="14" height="10" rx="2" class="bf-slot" /><text x="30" y="267" class="bf-tm">conventional</text><rect x="147" y="258" width="14" height="10" rx="2" class="bf-cell-hi" /><text x="168" y="267" class="bf-tm">compact</text></svg>
<figcaption>Average thread CPU time per inverse-BWT row, from the registered prepared-table traversal. With one worker, compact navigation costs more; with four concurrent workers, it costs less. Construction and the rest of the decoder are excluded.</figcaption>
</figure>

The large-block advantage was broad: averaged across twenty paired rounds, all **230 full-sized blocks** had lower four-worker CPU cost with compact navigation; **1,138 of 1,140** full-sized four-worker steps also cost less CPU in aggregate. The conventional traversal's CPU cost per row rose approximately **68%** from one active worker to four. The compact traversal's rose approximately **23%**. The **0.328 ns-per-row** four-worker advantage is what remains after the compact design's higher isolated lookup cost is outweighed by its smaller growth under concurrent work.

These are measurements of a **prepared table's traversal and assembly**, not a production decoder. Both arms used the same source blocks, but the conventional table was built immediately before each traversal while the compact records were copied from a prebuilt reference image. Both preparations were untimed and could leave different cache states. No cache-miss, translation or memory-controller counters were taken. So **reduced shared-cache interference is a credible explanation, not yet an isolated cause**. The archive, complete schedule checks and independent reconstruction are retained with the [successful traversal record](https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4). Earlier failed validity checks remain recorded rather than being folded into this result.

Still, the tested result met the [predeclared decision rule](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md): the upper 95% confidence bound on its four-worker CPU-cost ratio was **0.843**, below the 0.865 investment screen, while the completion-time bound was **0.860**, below the allowed 1.02. The decision was **ADVANCE**—permission to investigate whether this table can be constructed economically, not permission to replace the decoder's implementation.

The one-core regression is not a footnote to that decision. A real worker alternates among constructing the table, following links and processing output, while its neighbours do the same. Four-way simultaneous traversal is not a permanent state of the decoder. A new builder could also alter the cache state that made the prototype promising. The first authorised next step is therefore a direct constructor that produces these same records without first materialising the full successor array, followed by a separate build-cost test and, if justified, a complete-decoder comparison. The approximate phase model leaves it a conservative budget of **1.067× the conventional builder's CPU cost**, assuming the four-worker traversal benefit survives. That assumption remains to be tested.

Seward's work is not being contradicted here. It taught us to look at how the representation changes the *cost of following a link*, not just at its nominal width or the number of instructions needed to decode it. His measured 2001 preference for `mergedTL` was grounded in memory behaviour, and he explicitly allowed that different cache capacities could alter the result. Our new observation adds a different workload regime: **several live tables competing for the same cache**. Under that particular test, a more compact representation has finally become faster despite needing more instructions per step.

We have not yet demonstrated an outward shift of the complete decoder's throughput–memory frontier. But we now have a sounder basis for asking whether the space saved by compact navigation can buy back more performance than it costs. Twenty-five years after Seward asked how little memory an inverse BWT could use while remaining fast, that is a question worth carrying through to construction and end-to-end decoding.

---

**Sources:** [Seward, *Space-time Tradeoffs in the Inverse B-W Transform* (DCC 2001)](https://doi.org/10.1109/DCC.2001.917175), especially §§2–6 and Tables 1–3 · [The early compact-navigation census](2026-10-29-compact-navigation-tables.md) · [The October frontier hypothesis](https://github.com/lmmx/mibz2/blob/f384fb424b19e5e5e1c5e27559dd0eb916e98123/docs/journal/2026-10-29-frontier/seward-frontier-research.md) · [FOR-14](https://github.com/lmmx/mibz2/pull/130) · [Compact30's validated walker](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md) · [The accepted capture and independent analysis](https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4).
