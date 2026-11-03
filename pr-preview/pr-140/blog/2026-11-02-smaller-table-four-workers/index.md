[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/index.md>)

# Revisiting the space–⁠time tradeoff in BWT inversion

In 2001 Julian Seward set out how much memory the inverse Burrows–Wheeler transform needs against how fast it can run. Read closely, his paper explains why a compact table that costs more per step could still win once four cores share a cache, and a first measurement of ours shows it doing so.

2 November 2026 · 30 min read · [Research result](<#what-the-comparison-found>)

[Julian Seward](<https://en.wikipedia.org/wiki/Julian_Seward>) wrote [bzip2](<https://en.wikipedia.org/wiki/Bzip2>), first released in 1996, and was the original designer and author of [Valgrind](<https://valgrind.org/info/developers.html>). In 2001, while at Microsoft Research in Cambridge, he presented a ten-page paper at the IEEE Data Compression Conference: *[Space-time tradeoffs in the Inverse B-W Transform](<https://doi.org/10.1109/DCC.2001.917175>)*. bzip2 compresses with the [Burrows–Wheeler transform](<https://en.wikipedia.org/wiki/Burrows–Wheeler_transform>) (BWT), and decompressing means undoing it. The paper asks what the undoing costs in memory and in time, and lays out a family of algorithms that trade one for the other, from five bytes of working memory per byte of block down to one.

Its introduction contains the sentence that most of our work on mibz2's decoder has turned out to be about:

> At the other extreme, we argue that the maximum speed of BWT decompression is unavoidably constrained by the rate at which cache misses are serviced.

mibz2 is our bzip2 decoder, written to decompress the Wikidata dumps, and its inverse BWT has been the subject of most of this blog:

- [32 lanes](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-31-room-walk-overlap/index.md>) that keep many misses in flight at once;
- [memory-level parallelism](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-11-02-memory-level-parallelism/index.md>), on what limits how many can overlap;
- [memory interference](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-11-01-inverse-bwt-memory-interference/index.md>), on what happens when four workers share the memory system;
- [compact navigation tables](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-29-compact-navigation-tables/index.md>), on taking fewer misses by making the table smaller.

Those are the parts of one rate: how many misses a decoded byte costs, how many can be serviced at once, and what else is asking. We came to them one experiment at a time, and Seward's sentence names what they share. On 29 October we wrote down the question this post returns to, in a [journal entry on the throughput–memory frontier](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier.md>) and the [benchmark page](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/frontier/index.md>) that grew from it. His tradeoff was drawn for one block decoded alone. Ours decodes several blocks at once, in one shared cache.

Seward left the same thought in [bzip2's manual](<https://sourceware.org/bzip2/manual/manual.html>), among its performance notes: "performance, both for compressing and decompressing, is largely determined by the speed at which your machine can service cache misses. … I imagine bzip2 will perform best on machines with very large caches."

This post is about a compact table of ours that revisits that tradeoff, and the result it has given so far. The table only makes sense against the paper, so the post reads the paper first, in order, sections 1 to 6. It works a six-byte example by hand, reproduces his tables, and plots his results as the tradeoff they describe. Then it places our table on that tradeoff, and reports what changed when four cores walked their tables at once.

## The question (Seward §1)

The introduction notes that the compression side of the BWT had been well studied: move-to-front coding, encoding its output, and fast sorting, on which Seward had published the year before. "Decompression-time issues remain underinvestigated." He gives two reasons they matter, and they pull in opposite directions. Booting a Linux kernel from a compressed image has to decompress in limited memory, while on-the-fly disk compressors need it to be fast. So the paper offers a range of algorithms rather than one: "We show that, at one extreme, it is possible to invert the BWT, slowly, using just one byte per block-byte. At the other extreme," the sentence above.

## The notation, on a six-byte block (§1.1)

Seward takes his notation from §2 of Burrows and Wheeler's 1994 report, *[A Block-sorting Lossless Data Compression Algorithm](<https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf>)*, and assumes the reader knows it. Here it is, with `banana` as the block:

| Symbol | Seward's definition | For `banana` |
| --- | --- | --- |
| N | the number of bytes in the block, "typically in the range of one million" | 6 |
| S | the block itself, S\[0..N−1\] | `banana` |
| M | an N × N matrix of bytes: every rotation of S, sorted | the six rows below |
| L | the last column of M: L\[i\] = M\[i, N−1\] | `nnbaaa` |
| F | the first column of M: F\[i\] = M\[i, 0\] | `aaabnn` |
| I | the row of M that holds S: S = M\[I, 0..N−1\] | 3 |
| T | a vector of N integers, each in 0..N−1, the "transformation vector" | 4 5 3 0 1 2 |
| C | 256 counts of the byte values, or their running totals | a 3, b 1, n 2 |

The decoder is given N, L and I, and must produce S. The paper writes sequences by concatenation, so S\[1..N−1\]S\[0\] is S rotated left by one place.

The six rotations of banana, sorted

M for S = banana. F, highlighted, is the bytes of S sorted; L is the column a bzip2 block carries. I = 3 is the row holding S.

A rotation moves some bytes from the front of a string to its back. Write out every rotation of `banana` and sort the six strings, and the result is M. Row 3 is `banana` itself, so I = 3. A decoder never holds M. The inverse transform recovers just enough of it, from L, to read off row I.

### Why the first and last columns

Every row of M is S read round from some starting point, so in every row the last byte is the one that comes just before the first, going round S. Row 3, `banana`, ends in `a` and starts with `b`, and going round, the final `a` comes before the opening `b`. Row 0, `abanan`, starts at that final `a` and ends in the `n` before it. This is the one relation between the columns that the whole paper builds on: L\[i\] comes just before F\[i\] in S.

The two columns carry different information: - **F** is the bytes of S in sorted order, `aaabnn`, so it is fixed by the counts alone: three `a`, one `b`, two `n`. A decoder holding L can rebuild F by counting, so it is never sent. - **L** holds, for each row, the byte that precedes that row's opening context, with the rows sorted by context. Bytes that precede similar contexts tend to repeat (in English, the context `he` is often preceded by `t`), so L has long runs of equal bytes. bzip2's later stages compress those runs.

So L is what a bzip2 block carries, with I to say which row is S. What the decoder needs is a way from "each L byte comes before its F byte" to the order of S, and that is the vector T.

## The transformation vector T (§2.1)

Seward's §2 is, in his words, "a more detailed exposition of Algorithm D in §2 of \[1\]". Algorithm D is the decompression half of Burrows and Wheeler's report. He gives his reason for the extra detail: "The increased detail exposes a range of possible implementations of the inverse transformation." Every algorithm in sections 3 to 5 is one of them.

"Undoing the BWT hinges on establishing the relationship between the F and L vectors, and recording this in the 'transformation vector' T." The definition is his equation (1):

```text
(1)   M[j, N−1] M[j, 0..N−2] = M[i, 0..N−1],   and then T[j] = i
```

Read the left-hand side as: take row j and move its last byte to the front. A rotation of a rotation of S is another rotation of S, so the result is some row i of M, and T\[j\] records which. Row 0, `abanan`, becomes `nabana`, which is row 4, so T\[0\] = 4. Row 3, `banana`, becomes `abanan`, row 0, so T\[3\] = 0. In full, T = 4 5 3 0 1 2.

Moving the last byte to the front starts the string one byte earlier in S. So T\[j\] is the row whose string starts one byte before row j's, and following T walks S backwards.

## Walking T back to the text (§2.2)

Row T\[j\] starts with the byte that row j ends with. That byte comes just before row j's first byte in S, which in columns is the paper's second result:

```text
(2)   F[T[i]] immediately precedes F[i] in S, for every i
```

The decoder has L, not F, so Seward restates (2) in terms of L. The relation just used says L\[k\] = F\[T\[k\]\] for every k: row k's last byte is row T\[k\]'s first. Put T\[k\] in place of i in (2), and both sides become L:

```text
F[T[T[k]]] immediately precedes F[T[k]]   ⇒   L[T[k]] immediately precedes L[k]
```

Every term is now something the decoder holds. Row I is S itself, so L\[I\] is the last byte of S. Starting there and following T gives S from the end:

```text
S[0, 1, .., N−2, N−1] = [ L[T^(N−1)[I]], L[T^(N−2)[I]], .., L[T[I]], L[I] ]
```

For `banana`, starting at p = I = 3:

| step | p | emit L\[p\] | next p = T\[p\] |
| ---: | ---: | :---: | ---: |
| 0 | 3 | `a` | 0 |
| 1 | 0 | `n` | 4 |
| 2 | 4 | `a` | 1 |
| 3 | 1 | `n` | 5 |
| 4 | 5 | `a` | 2 |
| 5 | 2 | `b` |  |

The walk emits `ananab`: `banana`, backwards.

This walk is the whole of the inverse transform's inner loop: load the entry at p, emit a byte, jump to the position it names. Where each jump lands depends on the content of S, not on where the last one landed. In a block of a million bytes that is a million loads from places the cache has no particular reason to hold. Section 6 measures what that costs.

### Backwards, and bzip2's "design flaw"

"Generating S backwards increases the decompressor's storage requirements since the whole of S has to be created before any of it can be output. The obvious solution is to reverse S in the compressor, before sorting." Peter Fenwick had measured no consistent loss or gain in compression from doing so. The rest of the paper assumes it, so each algorithm's walk starts at p = I and writes S forwards.

bzip2 doesn't reverse its blocks, and a footnote in §4 says so: "As is the case with the bzip2 compressor – a design flaw." The bzip2 manual says how that came about: "It would be fair to say that the bzip2 format was frozen before I properly and fully understood the performance consequences of doing so." Its decoder compensates by building the links the other way round. In `decompress.c`, the fast path does this after counting:

```c
/*-- compute the T^(-1) vector --*/
for (i = 0; i < nblock; i++) {
   uc = (UChar)(s->tt[i] & 0xff);
   s->tt[s->cftab[uc]] |= (i << 8);
   s->cftab[uc]++;
}
```

Where §2.3 below would set T\[i\] = s, this writes i into slot s, so slot s records where it came from. That is T⁻¹, and following T⁻¹ emits S forwards. For `banana`, T⁻¹ = 3 4 5 2 0 1. Starting from T⁻¹\[I\] = 2, the walk reads L\[2\], L\[5\], L\[1\], L\[4\], L\[0\], L\[3\]: `banana`. mibz2's builder does the same, writing `ll[s] | i << 8` into slot s.

## Building T from counts alone (§2.3)

A decoder can't compute T from M, which it doesn't have. §2.3 shows T comes from L with one counting pass and one scan. It rests on two observations: 1. The rows of M are sorted. 2. Two rows that end in the same byte keep their order when that byte moves to the front. They now start with the same byte, so the rest of each original row decides their order, and those rows were already sorted.

So if L\[j\] is the k-th occurrence of byte c in L, the row it rotates into is the k-th row starting with c: the k-th c in F. "In short, if c occurs k times in L\[0..j\] and L\[j\] = c, then T\[j\] must be i, where i is the position of the k-th c in F."

T matches each L byte to the same occurrence in F

The k-th a in L goes to the k-th a in F, and likewise for n and b. That is §2.3's observation, and it gives T = 4 5 3 0 1 2 from counts and one scan.

F is sorted, so the c's in F form one contiguous bucket. It starts at C\[c\], the number of bytes in the block smaller than c: for `banana`, C\[a\] = 0, C\[b\] = 3 and C\[n\] = 4. Then T\[j\] = C\[c\] + k − 1. Scan L once, keeping a running count for each byte value, and each byte's next free slot in its bucket is its T entry.

Within one byte value, the links increase with j: the k-th c in L goes to the k-th slot of c's bucket. Every BWT has this ordering, and it comes back at the end of this post.

## As fast as possible: basis, bw94 and mergedTL (§3)

"Newcomers to BWT-based compression are often delighted at the improvement in compression ratio over LZ77/78 schemes, but dismayed at the speed, especially the speed of decompression. How fast can inversion be?" §3 answers with three algorithms, each refining the last.

**basis** is §2 written out: 1. Count each byte value in L, turn the counts into running totals, and shift them right by one, so that C\[c\] is where c's bucket starts in F. 2. For each i, set T\[i\] = C\[L\[i\]\] and increment C\[L\[i\]\]. 3. Start with p = I. For each i, set S\[i\] = L\[p\] and p = T\[p\].

It reads L three times, and keeps L (one byte a row) and T (four) alive: 5N bytes.

**bw94** takes its name from Burrows and Wheeler's 1994 report, whose §4.2 merges the first two passes. While counting, it records how many of each byte it has seen so far, so T′\[i\] = the number of earlier occurrences of L\[i\]. That is T without its bucket's starting point, T\[i\] = T′\[i\] + C\[L\[i\]\]: in Seward's words, "displacements relative to the start of each character run in F". The walk adds the start back: S\[i\] = L\[p\], then p = T′\[p\] + C\[L\[p\]\]. Two passes, and still 5N bytes.

**mergedTL** comes from looking at bw94's walk: "Each iteration indexes both L and T′ at some arbitrary point, but both at the same index p. Unless both T and L fit in the machine's cache, this could cause two (capacity) misses per iteration." The fix puts both in one 32-bit word, limiting N to 2²⁴:

```text
U[i] = 256 · T′[i] + L[i]
```

The name is literal: T (as T′) and L, merged into one array. The walk becomes:

```text
u = U[p];   c = u mod 256;   S[i] = c;   p = u div 256 + C[c]
```

The mod and div are a mask and a shift. Unpacking costs some instructions, "but the near-halving of the cache miss rate is potentially valuable". The table is also a byte a row smaller: 4N.

What one row costs, bit by bit

bw94 keeps T′ and L in separate arrays, so a step can miss twice. mergedTL packs both into one word. indexF keeps only T, in 20 bits, and finds the byte from the counts. bzip2 and mibz2 pack the byte with the reversed link.

Seward notes that mergedTL is "employed in bzip2", in the form adapted to an unreversed block. bzip2's word at slot s holds L\[s\] in its low 8 bits and the reversed link above it. The link is an absolute position, so the walk needs no C\[c\] addition. The manual lists the change among the improvements in bzip2 0.9.0: "Single array implementation of the inverse BWT. This significantly speeds up decompression, presumably because it reduces the number of cache misses."

## Less memory: T overwrites L (§4)

"Algorithm mergedTL aims for maximum speed by packing T and L into the same array of 32-bit words. Can we reduce memory use whilst retaining useful decompression speed? Interestingly, yes."

The way in is to go back to equation (2), which is about F: F\[T\[i\]\] precedes F\[i\]. The §3 algorithms use its derived form, L\[T\[i\]\] precedes L\[i\], only because L is what the decoder holds, and that obliges them to keep L alive through the whole walk. Indexing F instead means L is needed only while T is built, so T can be written over it: hence §4's title.

F needn't be stored either. It is sorted, so the 256 running totals in C describe it, and F\[p\] is the byte whose bucket contains p. "No free lunch, alas: subscripting F then becomes a binary search in the cumulative frequency table, rather than a cheap array indexing operation."

**indexF** keeps one array, A, of N 20-bit words, so N ≤ 2²⁰: 1. A\[i\] starts out holding L\[i\] in its low 8 bits. (Seward numbers this step 0.) 2. Count as in basis, reading the bytes from A. 3. Copy C to C′. 4. For each i, increment C′\[A\[i\]\] and then set A\[i\] = C′\[A\[i\]\] − 1. A\[i\] is now T\[i\], written over L\[i\]. 5. Start with p = A\[I\]. For each i, set S\[i\] = indexF(p) and p = A\[p\].

indexF(p) is the binary search: it finds the byte whose bucket in C contains p. "Since C has 256 entries this search always completes in nine steps." The name is the operation, and Table 1 describes the whole algorithm as "Indexes into F rather than L".

The walk starts at p = A\[I\] = T\[I\], not at I. Indexing L, the first byte out is L\[I\]. Indexing F, the same byte is F\[T\[I\]\], since L\[k\] = F\[T\[k\]\], so p is "stepped backwards once before the loop".

If the block wasn't reversed, as with bzip2, Seward adds a pass between the last two steps. It reverses the pointers in A, turning T into T⁻¹ in place, with no second array. His 20-bit words are a 16-bit array of N entries plus a byte array of ⌈N/2⌉ entries holding the top 4 bits, two to a byte: 2.5N bytes in all.

bzip2 ships this as its `-s` option, documented as "a modified algorithm which only requires 2.5 bytes per block byte … albeit at about half the normal speed". Its source has §4's pieces: - the 16-bit array `ll16` and the nibble array `ll4`; - the reversal: "Compute T^(-1) by pointer reversal on T"; - the search itself, `BZ2_indexIntoF`:

```c
Int32 BZ2_indexIntoF ( Int32 indx, Int32 *cftab )
{
   Int32 nb, na, mid;
   nb = 0;
   na = 256;
   do {
      mid = (nb + na) >> 1;
      if (indx >= cftab[mid]) nb = mid; else na = mid;
   }
   while (na - nb != 1);
   return nb;
}
```

### What indexF gives up

Seward presents the cost as one trade, a search for memory. It has several parts: - **A block-size cap** of 2²⁰ bytes (1,048,576). bzip2's largest block, 900,000 bytes, fits. - **A search per output byte:** eight or nine dependent comparisons, where mergedTL has one mask. The search runs over 1 KB of counts, which stays in cache, so it costs instructions and not misses. - **The search is off the chain.** In mergedTL the next position, u div 256 + C\[c\], depends on the byte just unpacked. In indexF it is p = A\[p\], the link alone. A processor that runs ahead can overlap the search for one byte with the load for the next. - **Two arrays again.** The 20-bit entry is split, 16 bits in one array and the nibble in another, so a step can touch two cache lines: the cost mergedTL was built to remove. Both arrays are smaller than mergedTL's, though, at 2N and N/2 bytes against 4N, so more of each stays in cache. - **An extra pass** over A, the pointer reversal, for a block that wasn't reversed.

Section 6 prices the bundle: indexF runs at 46% of mergedTL's speed, in 62.5% of its memory.

## The least memory: no T at all (§5)

"It is instructive to ask just how little storage we can get away with, and the answer is surprising: only L and the 'compressed' frequency-count version of F are actually needed."

T needn't be stored, since any entry can be computed when it's needed: with c = L\[j\], T\[j\] = C\[c\] + the number of c's in L\[0..j−1\]. (The printed text says L\[j\] can be calculated this way; T\[j\] is meant.) Counting means scanning L up to j, so each step costs O(N) and the walk O(N²). Seward calls it "absurdly inefficient", but it is the minimal-storage baseline: N bytes of L, plus the counts. The rest of §5 buys speed back with memory.

Two tables of counts, one for each half of L, would halve the average scan. More tables shorten it further but waste space: the shorter the segment, the more of its 256 counts are zero. Instead, Seward grows 256 binary trees, one per byte value: - **A node** holds a count and two child pointers, and covers a segment of L. An empty subtree means that byte doesn't occur in its segment, and costs nothing to store. - **Segments are aligned powers of two.** With Q the least power of two ≥ N, each root covers L\[0..Q−1\], its children the two halves, and so on. - **A node budget, K\_max,** sets how deep the trees grow, and so the trade between memory and scan length. The trees can stop at any depth and still answer correctly.

Building the trees one at a time would scan L up to 256 times per level. Instead, each deepening pass scans L once, in steps of Δ = Q/2^(d−1). At each step it counts every byte value in the step's two halves, and gives each tree's node for that segment its two children.

To answer "how many c's are in L\[0..j\]?", descend c's tree from the root with a running count. When j is in the left half, go left. When it is in the right half, add the left child's count and go right. On falling off the bottom, scan L from the start of the last segment to j.

A count tree answers a rank query

The tree for byte a over L = nnbaaa, padded to 8 positions. A count of a's up to position 4 takes one left-child count and a one-byte scan. With the 2 a's before it, the a at L\[4\] goes to T\[4\] = C\[a\] + 2 − 1 = 1.

**tree** stores a node as three 32-bit words, 12 bytes, so 1 MB holds 83,333 nodes.

**treeopt** fits nearly twice as many in the same space, in two ways: - **Packed nodes.** With N ≤ 2²⁰ and about 10⁶/12 nodes, a node needs ⌈log₂(10⁶/12)⌉ + 20 + ⌈log₂(10⁶/12)⌉ = 17 + 20 + 17 = 54 bits: two 17-bit child indices and a 20-bit count. That fits in two words. - **One-word leaves.** A leaf has no children, so it needs no pointers.

treeopt keeps interior nodes at the left of one array and leaves at the right. After each deepening pass, it turns the new leaves into interior nodes. "Scanning is much more expensive than tree-building", so the doubled node count, which halves the average scan, is worth the complication.

The versions tested were: - tree-1M, with 1 MB of nodes: 2N in all, with L; - treeopt-1M, also 2N; - treeopt-500k, 1.5N.

## The measurements (§6)

The experiments ran on a 400 MHz AMD K6-III with 128 MB of memory and a 256 KB second-level cache, under Linux 2.0.36, cleared of other processes. Blocks were capped at 10⁶ bytes, and each file's L vectors were prepared in advance. Once read from disk, each block was inverted ten times into an array that was never written out, with CPU use of at least 99%.

**Table 1** gives each algorithm's memory for a 10⁶-byte block, not counting the at most 2 kB of C and C′:

| Algorithm | Memory | Seward's description |
| --- | ---: | --- |
| `basis` | 5000 kB | Straightforward derivation from theory |
| `bw94` | 5000 kB | As per \[1\], does 2 passes, not 3 |
| `mergedTL` | 4000 kB | As bw94 but more cache-friendly |
| `indexF` | 2500 kB | Indexes into F rather than L |
| `tree-1M` | 2000 kB | Computes T incrementally; 1 megabyte extra space |
| `treeopt-1M` | 2000 kB | More space-economical version of tree-1M |
| `treeopt-500k` | 1500 kB | As treeopt-1M but with only 500k auxiliary space |

Seward's Table 1: memory per block byte

Working storage for a 10⁶-byte block, in bytes per block byte. treeopt-1M, not drawn, also takes 2N. Output and the count tables are excluded.

**Table 2** lists six test files. They are all at least a megabyte, since smaller files "don't stress algorithms or memory systems realistically". Four come from the [Canterbury Corpus](<https://corpus.canterbury.ac.nz/descriptions/>). mibz2's [benchmark](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/index.md>) has its own corpora, and the last column names the nearest of them, loosely.

| File | kB | What it is | Nearest in mibz2's corpora |
| --- | ---: | --- | --- |
| bible | 4047.4 | The King James Bible | `text`, though ours is the crate's own sources repeated, and more redundant |
| world192 | 2473.4 | The CIA World Factbook, 1992 | none. Country records in English: Wikidata's subject matter, not its format |
| ecoli | 4638.7 | The *E. coli* genome, in four letters | none |
| kennedy | 1029.7 | An Excel spreadsheet | `runs`, and at the extreme `onebyte` and `pattern` |
| netscape | 7678.8 | Netscape Navigator 4.75, an x86 executable | `records`, binary |
| bashtgz | 1708.8 | bash 2.04's source, gzipped | `random`: gzip's output is close to random bytes |

**Table 3** gives the inversion rates, in MB/s (of L in, or of S out). mergedTL is the fastest on every file:

|  | basis | bw94 | mergedTL | indexF | tree-1M | treeopt-1M | treeopt-500k |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| bashtgz | 1.54 | 1.58 | 2.49 | 1.20 | 0.032 | 0.059 | 0.032 |
| bible | 1.72 | 1.77 | 2.76 | 1.36 | 0.211 | 0.301 | 0.218 |
| ecoli | 1.53 | 1.57 | 2.48 | 1.30 | 0.325 | 0.342 | 0.339 |
| kennedy | 2.46 | 2.54 | 4.72 | 1.71 | 0.099 | 0.167 | 0.100 |
| netscape | 2.32 | 2.41 | 3.51 | 1.67 | 0.044 | 0.099 | 0.036 |
| world192 | 1.65 | 1.69 | 2.65 | 1.31 | 0.207 | 0.298 | 0.198 |
| Average | 1.86 | 1.93 | 3.10 | 1.43 | 0.153 | 0.211 | 0.154 |

"mergedTL is the clear speed winner, on average 61% and 67% faster than bw94 and basis respectively, at the same time using less memory." indexF averages 46% of mergedTL's speed. It "still inverts well over a megabyte per second on this platform, which may be an acceptable tradeoff given the 2.5N bytes memory use."

The tree algorithms run at 5–7% of mergedTL's rate, and Seward expects them "to remain no more than proof-of-concept implementations". Even so, treeopt-500k matches tree-1M with half its node memory: the packed tree representation does what it was designed to do.

The paper has no figure. Plotting Table 3's averages against Table 1's memory draws the tradeoff of its title:

Seward's Tables 1 and 3, plotted

Each algorithm's average rate over the six files against its memory, on a 400 MHz K6-III in 2001. Highlighted points are beaten on neither axis. The paper prints the numbers as tables; this plot is ours.

Four points are beaten on neither axis: treeopt-500k, treeopt-1M, indexF and mergedTL. mergedTL beats basis and bw94 on both, and treeopt-1M beats tree-1M. From indexF to treeopt-1M, half a byte a row less costs a factor of seven in rate. The rate axis is logarithmic, so that step is much steeper than it looks.

## Three anomalies, and a bound (§6.3)

Seward ends the results with three questions that Table 3 raises. His answers carry the paper's argument about cache misses.

**Why is mergedTL so much faster than bw94, when it does more work to unpack?** Seward inverted bible under cacheprof, his own cache profiler, simulating the K6-III's 256 KB L2. It counted per byte of L:

|  | instructions | memory references | misses |
| --- | ---: | ---: | ---: |
| bw94 | 18.00 | 9.00 | 1.63 |
| mergedTL | 20.00 | 8.00 | 0.88 |

mergedTL executes 11% more instructions, takes 54% as many misses, and runs 56% faster on bible. Of mergedTL's misses, 89.4% come from the walk's reads of U\[p\], one read per byte, and 78.6% of those reads miss.

They miss for the reason §2.2 showed. T "effectively contains the inverse permutation to that created by the BWT's sorting operation", and nothing ties the sorted order of the rotations to their order in S. So "T will in general hold a completely arbitrary permutation of 0..N−1. Since the decompressor is obliged to traverse this permutation, there appears to be no way to avoid a miss rate approaching one per L-byte" once N is much larger than the cache. Hence "it is plausible to argue that algorithm mergedTL represents a realistic upper bound on inversion performance": the introduction's sentence, now with numbers under it.

**If misses set the rate, why does kennedy invert nearly twice as fast as bashtgz?** Runs. kennedy contains long runs of one byte. Following Burrows and Wheeler's own analysis, a stretch such as `ccc…ccc` followed by something unique sorts in order (or in reverse order), so T maps each of its rows to the next (or previous) one. A walk through such a stretch moves to the neighbouring entry, which is usually already in the cache. Random input has nothing like this and should set the floor. The table bears that out: bashtgz, ecoli and world192 invert at 2.49, 2.48 and 2.65 MB/s.

Why the walk misses, and why runs help

An illustration, not data. Following T through an arbitrary permutation lands somewhere new each step. Through a run of equal bytes, as in kennedy, T points next door.

**Why do the tree algorithms vary 10:1 across the files, against 1.9:1 for mergedTL?** Alphabet size. ecoli has four distinct bytes, so the node budget goes to four deep trees with small leaves and short scans, and it is the tree algorithms' best file. bashtgz uses all 256 byte values almost uniformly. Each tree can then be only a 64th the size, giving "much longer L scans and lower throughput".

Then the bound is put to use. bzip2 decompresses bible at 1.86 MB/s, 67% of mergedTL's inversion rate, and the same on kennedy, while also doing its Huffman and move-to-front decoding. "These measurements, though crude, are interesting, since they suggest that no amount of re-engineering of bzip2 can increase its decompression speed much further."

The paper's last paragraph is two sentences:

> Such observations only hold true whilst cache sizes are much smaller than block sizes. Future growth in cache sizes may invalidate such findings.

## What changed since 2001

The K6-III's cache held 256 KB, against mergedTL's 4 MB for one block: about a sixteenth of the table. The workstation behind our measurements has a 20 MiB last-level cache, and a 900,000-byte bzip2 block needs a 3.6 MB table. Read literally, the caveat has come true: one block's table fits in the cache several times over.

Two things kept the sentence from the introduction relevant anyway.

**A miss is no longer a fixed price.** A modern core keeps many misses in flight at once, if the program gives it independent loads to issue. One chain through T gives it none, since each address comes from the load before. mibz2's walk follows 32 chains at once, independent segments of the same block, and runs [8.6 times as fast as one chain](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-31-room-walk-overlap/index.md>). Kärkkäinen, Kempa and Puglisi's *[Slashing the time for BWT inversion](<https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf>)* (DCC 2012) attacked the same rate from the other side, with fewer and cheaper misses. Our [memory-level parallelism post](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-11-02-memory-level-parallelism/index.md>) looks at what limits the overlap.

**A decoder doesn't invert one block at a time.** mibz2 runs a worker per core, and four workers keep four tables live: 14.4 MB, in a cache all four cores share. On the workstation, throughput falls off at [about 14.4 MB of live tables](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/frontier/index.md>), 0.7 of its L3 ([workstation journal §9](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md>)). Four workers on four cores spend [32% more CPU time per decoded byte](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-11-01-inverse-bwt-memory-interference/index.md>) than one does, on nearly the same instructions. With the cache divided between blocks, Seward's condition, caches much smaller than the tables being walked, holds again.

That reopens his tradeoff in a form the paper had no reason to consider. His axes are one block's rate and one block's memory, and on them a smaller table is a step down in speed. With four blocks sharing a cache, a smaller table also leaves more of the cache to the other three. Its rate with four workers could then beat what its single-block rate predicts, and perhaps mergedTL's own. The [journal entry of 29 October](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier.md>) asked whether a table that is slower for one block could move the tradeoff outward for the machine, rather than along it.

## A table between mergedTL and indexF

mibz2's production table is bzip2's: one 32-bit word per row, the byte in the low 8 bits and the reversed link above it, 4N bytes. In Seward's terms it is mergedTL. Read against the paper, our compact table combines three of its ideas.

**From §2.3, the ordering.** Within one byte value's bucket, the links increase with the row. A group of consecutive rows in one bucket can therefore store its links as one base and small non-negative offsets. The offsets are smallest when the byte's occurrences in L are close together, as in kennedy's runs. In Wikidata's blocks they usually are: in our census, about 77% of a full block's rows fall in 32-row groups that each come from a single run in L, from 63.7% to 88.0% per block ([early findings](<https://lmmx.github.io/mibz2/pr-preview/pr-140/blog/2026-10-29-compact-navigation-tables/index.md>)). To Seward, runs gave locality. Here they also give compressibility.

**From §4, the emitted byte.** As in indexF, the walk emits F, so L isn't needed once the table is built. Instead of a binary search through C for every byte, F is stored once per group, in the group's header. Every row of an ordinary group is in one bucket, so they all share it.

**From §3, the cache line.** As in mergedTL, an ordinary step finds everything it needs in one place: the header and the row's offset are in the same 64-byte record.

A 64-byte record for thirty rows

An ordinary group: thirty 16-bit offsets and a 4-byte header with the base link and the F byte the rows share. Groups that straddle a bucket or span too far are stored separately in full.

Each 64-byte record holds thirty 16-bit offsets in 60 bytes, and a 4-byte header with the group's base link and its F byte. The ordinary step adds the row's offset to the base. Some groups are stored separately in full, four bytes a row in 128-byte groups: - groups that straddle a bucket boundary; - groups whose links span more than the offsets can hold.

On our corpus the whole table averages 2.164 bytes a row. That is about 2.16N, below indexF's 2.5N and a little over half of mergedTL's 4N. On Seward's memory axis it sits between treeopt-1M and indexF, and it aims at mergedTL's speed.

It was not the first design: - **FOR-14** packed thirty-two 14-bit offsets into each record, for 2.076 bytes a row. Finding and unpacking a 14-bit field cost 38 instructions a step (37 with BMI2), against production's 16. - On one block it took 2.676 ns a row, against 1.275 for its control. - With four blocks its cost rose only 8% while the control's rose 56%: the effect we were looking for. - But it started too far behind, and still finished the four-block work 1.45 times slower. - **Two variants with packed tokens** came to 31 and 27 instructions. They were checked for correctness but never timed. - **The 30-row record above,** which we call compact30, gives up a little density for whole 16-bit fields and an F byte read from the header. It comes to 21 instructions.

| Table | Bytes a row | Instructions a step | Loads | Stores | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| production, bzip2's mergedTL | 4 | 16 | 3 | 2 | the baseline |
| FOR-14, packed 14-bit offsets | 2.076 | 38 (37 with BMI2) | 5 | 3 | measured; rejected |
| packed-token variants |  | 31 / 27 | 4 | 2 | correct; not timed |
| compact30, 16-bit offsets | 2.164 | 21 | 4 | 2 | measured |

The instruction counts are for each design's compiled ordinary path, from the [annotated machine code](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md>). They are static counts, not instructions retired per byte. The designs were also measured in separate experiments, each against its own control, so the column traces a lineage rather than a timing curve.

## What the comparison found

The comparison walked both tables over 434 blocks of the Wikidata dump, 207 million rows, checking every output against the production decoder's. It ran twenty paired rounds, first on one pinned core and then with four workers on four cores starting each step together.

The timed interval is the walk and the assembly of the block, not the building of the table. Production builds its table before each walk, while compact30 copies a prebuilt image, and both happen outside the timer. Those preparations may leave the caches in different states. The comparison's design, checks and decision rule were [registered before it ran](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md>), and every check passed. The rule was to go on to a builder if the upper 95% bound of the four-worker CPU ratio was at most 0.865 and that of the completion ratio at most 1.02. They came to 0.843 and 0.860.

The CPU-time figures sum the time the walking threads spent running, divided by the rows walked. The completion figure runs from the earliest start to the latest finish of each four-worker step, summed over the corpus. The 95% intervals describe paired variation across the twenty rounds on this machine and workload.

| Walk over a prepared table | Production | Compact30 | Compact30 ÷ production (95% interval) |
| --- | ---: | ---: | ---: |
| CPU time per row, one core | 1.221 ns | 1.402 ns | 1.148 (1.147–1.149) |
| CPU time per row, four workers | 2.055 ns | 1.727 ns | 0.840 (0.838–0.843) |
| Four-worker completion, one corpus pass | 114.45 ms | 97.97 ms | 0.856 (0.852–0.860) |

Slower alone, faster with four workers

CPU time per row, summed over the walking threads, from the comparison above. Lower is better. Table construction is excluded, and the chart shows the reversal, not its cause.

On one core, compact30 costs 14.85% more CPU time per row. One block's table fits in the cache with room to spare, so the extra instructions have nothing to pay for them. That is the outcome Seward's paper predicts for a table that does more work per step. With four workers it costs 15.95% less, and the four-worker completion time is 14.40% lower.

The absolute times show the crossover's arithmetic, though not its cause: - On one core, compact30 starts 0.181 ns a row behind. - With four workers, production's cost rises by 0.834 ns a row and compact30's by 0.325. - That is a 68% rise for production and 23% for compact30. - The difference between the rises, 0.509 ns, exceeds the starting gap, which leaves compact30 0.328 ns a row ahead.

The comparison didn't isolate why the rises differ. The leading explanation is the smaller shared footprint: about 7.79 MB for four full compact30 tables, against 14.4 MB for four production tables. That is computed table size, not measured cache occupancy or memory traffic, and the counters that would separate fewer evictions from other effects weren't collected.

The result doesn't rest on a few blocks: - **Full blocks:** all 230 full-sized blocks had lower four-worker CPU cost with compact30, averaged over rounds. - **Steps:** in 1,138 of the 1,140 full-sized four-worker steps, total CPU time was lower. - **Short blocks:** the 204 short blocks were slower, but they hold 8,324 of the corpus's 207 million rows.

The same blocks recur in every round, so these are robustness checks within one experiment, not hundreds of independent replications. The [records and their reconstruction](<https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4>) are kept.

## What it doesn't show yet

This is a faster walk over a prepared table, under controlled contention. It isn't yet a faster decoder.

**The table has to be built.** The reference builder makes a full temporary successor table and converts it. A real builder must produce the records directly from L, at a cost close to production's [single counting pass](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L253-L318>), which is §2.3 in Rust. In the [planning model](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md#6-the-construction-budget-and-the-decision-rules-fixed-before-any-run>) fixed before the comparison, keeping the measured four-worker gain allows a builder costing at most about 6.7% more than production's. That allowance assumes the gain survives everything else. The [constructor design brief](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md>) asks whether one can be built within it.

**A decoder isn't always four walks in lockstep.** Its workers also read and decode the compressed stream, build tables and assemble output, and they don't all walk at once. When only one or two are walking, the one-core penalty applies.

**The frontier claim is reserved for the whole decoder.** A table that uses less memory and walks faster under contention is a new point for one stage of the computation. Moving the [decoder's throughput–memory frontier](<https://lmmx.github.io/mibz2/pr-preview/pr-140/benchmarks/frontier/index.md>) means beating the best production configuration on throughput and measured peak memory together, on the same machine and workload.

Seward closed by allowing that bigger caches might one day make his findings obsolete. Twenty-five years on, the caches are eighty times larger, and the tables still don't fit: there are four of them, sharing. So far, in a walk with four tables live, a smaller table that costs more per step has given back more than it costs.

---

Research state, 2 November 2026: the comparison of walks over prepared tables is valid, and it justified the work on a builder. The builder's cost, the full decoder's throughput and peak memory, and the cause of the four-worker advantage are not yet established. New results will be added with their own dates and measurement boundaries, and this result and its records stay as they are.
