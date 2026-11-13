[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/index.md>)

# Revisiting the space–⁠time tradeoff in BWT inversion

A packed table saved memory but cost too much to read. A cheaper compact lookup gives the first positive traversal result.

2 November 2026 · 27 min read · [Result](<#7-timing-the-walk-on-one-core-and-on-four>)

In 2001, [Julian Seward](<https://en.wikipedia.org/wiki/Julian_Seward>), the author of [bzip2](<https://en.wikipedia.org/wiki/Bzip2>) and the original designer of [Valgrind](<https://valgrind.org/info/developers.html>), was working at Microsoft Research in Cambridge. That year he presented a paper at the IEEE Data Compression Conference, *[Space-time tradeoffs in the Inverse B-W Transform](<https://doi.org/10.1109/DCC.2001.917175>)*, on what it costs to undo the [Burrows–Wheeler transform](<https://en.wikipedia.org/wiki/Burrows–Wheeler_transform>) (BWT) that bzip2 is built on.

Compression with the BWT had been studied closely, but *"decompression-time issues remain underinvestigated"*. He had two uses in mind that pull in opposite directions: booting a Linux kernel from a compressed image needs little memory, and an on-the-fly disk compressor needs speed. So he offers a family of algorithms that trade one for the other. At one extreme, the BWT can be inverted, slowly, in one byte of memory per byte of block. At the other:

> we argue that the maximum speed of BWT decompression is unavoidably constrained by the rate at which cache misses are serviced.

mibz2 is our bzip2 decoder, written for the Wikidata dumps, and that sentence could stand over most of what we have written about it. Our posts on [32 lanes](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-31-room-walk-overlap/index.md>) and [memory-level parallelism](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-02-memory-level-parallelism/index.md>) are about keeping many misses in flight at once; [memory interference](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-01-inverse-bwt-memory-interference/index.md>) is about four workers' misses competing for one memory system; [compact navigation tables](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-29-compact-navigation-tables/index.md>) is about taking fewer misses with a smaller table. Seward said as much in [bzip2's manual](<https://sourceware.org/bzip2/manual/manual.html>): *"performance, both for compressing and decompressing, is largely determined by the speed at which your machine can service cache misses."*

The post has two halves: how Seward's algorithms work and what he measured, then what happens to his tradeoff in mibz2, where several blocks decode at once and share one cache. The result in the second half is a controlled test of the walk alone, not yet of the whole decoder. Readers who know how BWT inversion works can go straight to [the compact table](<#6-a-smaller-table-a-busier-lookup>).

Seward's paper, 2001

1. [1From sorted rotations to the transformation vectorHow a decoder gets `banana` back: a chain of lookups, each depending on the last](<#1-from-sorted-rotations-to-the-transformation-vector>)
2. [2Making inversion fast: basis, bw94 and mergedTLKeeping the byte and the link in one word](<#2-making-inversion-fast-basis-bw94-and-mergedtl>)
3. [3Spending less memory: indexF and the count treesRecomputing what the fast algorithms store](<#3-spending-less-memory-indexf-and-the-count-trees>)
4. [4The space–time tradeoffSeward's measurements of memory against speed](<#4-the-spacetime-tradeoff>)

mibz2, 2026

1. [5Four blocks sharing one cacheWhy Seward's tradeoff changes when blocks decode at once](<#5-four-blocks-sharing-one-cache>)
2. [6A smaller table, a busier lookupA little over half the space, more work per lookup](<#6-a-smaller-table-a-busier-lookup>)
3. [7Timing the walk on one core and on fourThe walk alone: slower on one core, faster on four](<#7-timing-the-walk-on-one-core-and-on-four>)
4. [8Four workers cost the conventional walk far moreWhy the usual table pays more for sharing the cache](<#8-four-workers-cost-the-conventional-walk-far-more>)
5. [9The table still has to be builtA cheap way to construct it for the decoder](<#9-the-table-still-has-to-be-built>)

## 1\. From sorted rotations to the transformation vector

Let's start with the word \\(\\mathtt{banana}\\) and see what the Burrows–Wheeler transform does to it.

First, write out six versions of the word, each starting at a different letter and wrapping round to the beginning when it reaches the end: \\(\\mathtt{banana}\\), \\(\\mathtt{ananab}\\), \\(\\mathtt{nanaba}\\), \\(\\mathtt{anaban}\\), \\(\\mathtt{nabana}\\), \\(\\mathtt{abanan}\\). These are the word's *cyclic rotations*: the same six letters, each time in a different order. (As permutations of the positions, they are the six powers of one, \\(\\sigma = (0\\;1\\;2\\;3\\;4\\;5)\\) in [cycle notation](<https://en.wikipedia.org/wiki/Permutation#Cycle_notation>): \\(\\sigma^0\\), the identity, is the word itself, and \\(\\sigma^k\\) starts it \\(k\\) letters in. Stacked in the order \\(\\sigma^0\\) to \\(\\sigma^5\\), each row is the one above shifted one letter: a [circulant matrix](<https://en.wikipedia.org/wiki/Circulant_matrix>), strictly its mirror image, since a circulant matrix shifts each row to the right. Sorting those same rows alphabetically shuffles them into what is called the Burrows–Wheeler matrix, which Seward's square is.) Then sort the six rotations alphabetically and stack them as rows:

Sorting the rotations into the Burrows–Wheeler matrix

Left, the rotations of banana built in their natural order: each row, σ¹ to σ⁵, is a copy of the one above with its first letter moved to the end. Right, the same rows sorted alphabetically. Before each row moves, the letters it shares with its closest rivals are tinted and the letter that settles the order is outlined: abanan goes first because after a, b comes before n. Each row keeps its label, so the powers end up shuffled, 5, 3, 1, 0, 4, 2. The sorted square's first column is shaded green and its last blue; banana, σ⁰, lands in row 3.

Each row now has two numbers. Its row number, on the left, says where sorting put it; its power of \\(\\sigma\\), on the right, says which rotation it is. Sorting scrambled the first but not the second: \\(\\sigma^5\\) and \\(\\sigma^4\\) are still one letter apart round the word, even though they now sit in rows 0 and 4. Recovering that relationship from the row numbers alone is what decoding comes down to.

Read down the last column of the sorted rows, and it spells \\(\\mathtt{nnbaaa}\\). The original \\(\\mathtt{banana}\\), \\(\\sigma^0\\), is in row 3, counting from zero. The transform keeps just those two things, the last column and the row number, and stores nothing else. It doesn't make the block smaller; the reordered letters compress better in bzip2's later stages, for a reason that comes below.

How can a decoder get \\(\\mathtt{banana}\\) back from only \\(\\mathtt{nnbaaa}\\) and 3? Each of Seward's algorithms answers that question in a different way.

Seward names the pieces with letters, following Burrows and Wheeler's 1994 report *[A Block-sorting Lossless Data Compression Algorithm](<https://www.cs.jhu.edu/~langmea/resources/burrows_wheeler.pdf>)*. On the same square:

Seward's names for the pieces

The square of sorted rotations again, with each symbol pointing at what it names. In bzip2, N can be up to 900,000.

So the decoder is given \\(L\\) and \\(I\\) and must produce \\(S\\). The square has been a way to understand the transform, but the decoder never builds it, and that matters for everything that follows. For a block of a million bytes, \\(M\\) would be a million rows of a million letters: a terabyte. The decoder keeps only \\(L\\), one byte a row, and whatever it needs to find its way from one row to the next. None of Seward's algorithms builds \\(M\\); what separates them is how much of that navigation they store, and in what form. That is the space in his space–time tradeoff.

**Why only \\(L\\) is stored.** Of the square's two outer columns, bzip2 keeps only the last:

- **\\(F\\)** is the letters of the word in sorted order, so the counts alone fix it. A decoder holding \\(L\\) can rebuild \\(F\\) by counting, so it is never sent.
- **\\(L\\)** holds the letter that comes before each row's opening, with the rows sorted by what follows. Letters before similar text tend to repeat (in English, \\(\\mathtt{he}\\) is often preceded by \\(\\mathtt{t}\\)), so \\(L\\) has long runs, which bzip2's later stages compress well.

**Moving backwards through the word.** Look again at row 3, \\(\\mathtt{banana}\\). Move its last letter to the front and you get \\(\\mathtt{abanan}\\). That is another rotation of the word, so it is already in the square, at row 0. Do the same to \\(\\mathtt{abanan}\\) and you get \\(\\mathtt{nabana}\\), at row 4. Each move steps one letter backwards through the word, treating its end as joined to its beginning, as on a ring:

Stepping backwards round the word

banana written round a circle, positions numbered from the top. Every row of the square reads the word clockwise from one letter (green), so its last letter (blue) is the one just before its first. Moving that last letter to the front starts the reading one letter earlier, so the walk steps anticlockwise: rows 3, 0, 4, 1, 5, 2.

| Row | Move its last letter to the front | Lands on row |
| --- | --- | --- |
| 3: \\(\\mathtt{banana}\\) | \\(\\mathtt{a} + \\mathtt{banan}\\) | 0: \\(\\mathtt{abanan}\\) |
| 0: \\(\\mathtt{abanan}\\) | \\(\\mathtt{n} + \\mathtt{abana}\\) | 4: \\(\\mathtt{nabana}\\) |
| 4: \\(\\mathtt{nabana}\\) | \\(\\mathtt{a} + \\mathtt{naban}\\) | 1: \\(\\mathtt{anaban}\\) |

The letters moved, \\(\\mathtt{a}\\), \\(\\mathtt{n}\\), \\(\\mathtt{a}\\), are each row's last letter, in \\(L\\), and they spell \\(\\mathtt{banana}\\) backwards from its end. So a decoder could recover the whole word from \\(L\\) alone, starting at row \\(I\\), if it knew where each move lands. Seward records the landing rows in \\(T\\), the *"transformation vector"*: here \\(T\[3\] = 0\\), \\(T\[0\] = 4\\) and \\(T\[4\] = 1\\). As a permutation of the rows, \\(T = (3\\;0\\;4\\;1\\;5\\;2)\\): one cycle through all six, so following it from row 3 visits every row once before coming back, reading the whole word on the way.

The \\(\\sigma\\) labels on the square show why: rows 3, 0, 4, 1, 5, 2 hold \\(\\sigma^0, \\sigma^5, \\sigma^4, \\sigma^3, \\sigma^2, \\sigma^1\\), so each step of \\(T\\) counts the power down by one. Stacked in that order, \\(\\mathtt{banana}\\), \\(\\mathtt{abanan}\\), \\(\\mathtt{nabana}\\), …, each row is the one above shifted one letter to the right: a circulant matrix in the strict sense. Following \\(T\\) rebuilds it row by row, undoing the sort. \\(T\\) is the one-letter step back, \\(\\sigma^{-1}\\), written in row numbers instead of powers ([conjugated](<https://en.wikipedia.org/wiki/Conjugacy_class>) by the sort, in group-theory terms). (Read down the square, those powers, 5 3 1 0 4 2, are what string algorithms call the [suffix array](<https://en.wikipedia.org/wiki/Suffix_array>): the compressor computes it by sorting, and the decoder never has it.)

His theory section, *"a more detailed exposition of Algorithm D in §2 of \[1\]"*, puts it in symbols:

\\\[ M\[j, N-1\]\\; M\[j, 0..N-2\] = M\[i, 0..N-1\] \\quad\\Longrightarrow\\quad T\[j\] = i. \\\]

The left-hand side is row \\(j\\) with its last letter moved to the front; \\(T\[j\]\\) is the row where that lands.

**Which row to land on.** The moved letter becomes the first letter of the landing row, so the landing row is one of those that start with it in \\(F\\). But \\(\\mathtt{banana}\\) has three \\(\\mathtt{a}\\)s, so three rows start with \\(\\mathtt{a}\\). Moving the \\(\\mathtt{a}\\) off the end of row 3 lands on row 0, not row 1 or 2; how could a decoder know that without the square?

Up to now the three \\(\\mathtt{a}\\)s have looked identical. To tell them apart, label each by where it falls in the word: \\(\\mathtt{b}\\;\\mathtt{a}\_1\\;\\mathtt{n}\_1\\;\\mathtt{a}\_2\\;\\mathtt{n}\_2\\;\\mathtt{a}\_3\\). The labels are only for following the letters; nothing in bzip2 stores them. This way of making the matching visible is borrowed from Ben Langmead's [lecture slides on the transform](<https://www.cs.jhu.edu/~langmea/resources/lecture_notes/bwt_and_fm_index.pdf#page=9>). Langmead also wrote [Bowtie](<https://genomebiology.biomedcentral.com/articles/10.1186/gb-2009-10-3-r25>), a DNA read aligner that uses the same correspondence to search a genome rather than to decompress it. Rows 3, 4 and 5 end in \\(\\mathtt{a}\_3\\), \\(\\mathtt{a}\_2\\) and \\(\\mathtt{a}\_1\\), and rows 0, 1 and 2 start with \\(\\mathtt{a}\_3\\), \\(\\mathtt{a}\_2\\) and \\(\\mathtt{a}\_1\\): the same occurrences, in the same order. This preserved order is what makes inversion possible, and the sort is why it holds. Rows 3, 4 and 5 are in that order because of what comes before their last \\(\\mathtt{a}\\): \\(\\mathtt{banan} \< \\mathtt{naban} \< \\mathtt{nanab}\\). Moving the \\(\\mathtt{a}\\) to the front puts those same letters right behind it, where they decide the order again:

Why the a’s keep their order

The square twice, its a’s and n’s labelled by where they fall in banana. Left, rows 3, 4 and 5 end in a; what comes before that a (tinted) puts them in order. Each moves its a to the front, carrying the tinted letters one place right, and lands on rows 0, 1 and 2 on the right, in the same order. So the a’s run a<sub>3</sub>, a<sub>2</sub>, a<sub>1</sub> down both L and F.

The same goes for the two \\(\\mathtt{n}\\)s, and for every letter of any block: the first \\(\\mathtt{a}\\) in \\(L\\) (row 3's) lands on the first row starting with \\(\\mathtt{a}\\) (row 0), the second on the second, and so on.

So the matching can be worked out from \\(L\\) and a count of each letter. Because \\(F\\) is sorted, its letters occur in runs, three \\(\\mathtt{a}\\)s, one \\(\\mathtt{b}\\) and two \\(\\mathtt{n}\\)s, and those counts describe it completely, much like [run-length encoding](<https://en.wikipedia.org/wiki/Run-length_encoding>). Seward calls the count table \\(C\\); once the counts are accumulated, it says where each letter's run begins in \\(F\\). The decoder's whole path, from what is stored back to the word:

What the decoder rebuilds

Each arrow is a step explained in the rest of this section: counting, expanding the counts into F, matching occurrences to get T, and the walk.

**Walking \\(T\\).** Row \\(T\[j\]\\) starts with the byte row \\(j\\) ends with, which comes just before row \\(j\\)'s first byte in \\(S\\). So \\(F\[T\[i\]\]\\) precedes \\(F\[i\]\\). The decoder has \\(L\\), not \\(F\\), but \\(L\[k\] = F\[T\[k\]\]\\), and substituting gives a relation in \\(L\\) alone:

\\\[ L\[T\[k\]\] \\text{ immediately precedes } L\[k\] \\text{ in } S. \\\]

\\(L\[I\]\\) is the last byte of \\(S\\), so starting at \\(I\\) and following \\(T\\) gives \\(S\\) from the end:

Decoding banana by following T

Across the top, the rows the walk visits and the byte it reads at each. Below, each step reads the byte in L at the current row, writes it into S from the end, and moves to the row T names. Every jump depends on the entry just read, which is why the walk is one dependent load after another.

That is \\(\\mathtt{banana}\\) backwards, and it is the whole inner loop of the inverse transform: load the entry at \\(p\\), emit a byte, jump where it says. Where each jump lands depends on the content of \\(S\\), not on the last jump. In a million-byte block that is a million loads from places the cache has no reason to hold.

To write \\(S\\) forwards, Seward assumes the compressor reverses each block before sorting. bzip2 doesn't, which a footnote calls *"a design flaw"*; by the manual's account, *"the bzip2 format was frozen before I properly and fully understood the performance consequences"*. Its decoder builds the links the other way instead, writing \\(i\\) into slot \\(s\\) where Seward would set \\(T\[i\] = s\\) (`tt[cftab[uc]++] |= i << 8`), and following that \\(T^{-1}\\) emits \\(S\\) forwards. mibz2 does the same, so its navigation table holds \\(T^{-1}\\): each entry points forwards, to the row whose string starts one byte later in \\(S\\).

**Building \\(T\\) from counts.** Since the \\(k\\)-th \\(c\\) in \\(L\\) lands on the \\(k\\)-th row starting with \\(c\\), Seward can state the rule directly: *"if \\(c\\) occurs \\(k\\) times in \\(L\[0..j\]\\) and \\(L\[j\] = c\\), then \\(T\[j\]\\) must be \\(i\\), where \\(i\\) is the position of the \\(k\\)-th \\(c\\) in \\(F\\)."*

T matches each L byte to the same occurrence in F

With the occurrences labelled, each link joins a letter in L to itself in F: L\[3\] is a<sub>3</sub>, which is F\[0\], so T\[3\] = 0. The links never cross within one letter, which is why counts are enough to rebuild T.

The \\(c\\)'s in \\(F\\) form one bucket. Turning the counts in \\(C\\) into running totals gives where each bucket starts: \\(C\[c\]\\) becomes the number of bytes smaller than \\(c\\) (for \\(\\mathtt{banana}\\), a 0, b 3, n 4). So

\\\[ T\[j\] = C\[c\] + k - 1, \\qquad c = L\[j\], \\\]

and one scan of \\(L\\) with a running count per byte value builds \\(T\\). Within each byte value the links increase with \\(j\\). Every BWT has this ordering, and our compact table depends on it.

## 2\. Making inversion fast: basis, bw94 and mergedTL

*"How fast can inversion be?"* Three algorithms, each refining the last, answer it.

**basis** is the theory written out: count, build \\(T\\), walk. It reads \\(L\\) three times and keeps \\(L\\) (one byte a row) and \\(T\\) (four) alive: \\(5N\\) bytes.

**bw94**, named for Burrows and Wheeler's report, merges the first two passes by recording \\(T'\[i\]\\), the number of earlier occurrences of \\(L\[i\]\\), while counting. That is \\(T\\) without its bucket's start, \\(T\[i\] = T'\[i\] + C\[L\[i\]\]\\), and the walk adds the start back. Two passes, still \\(5N\\) bytes.

**mergedTL** comes from looking at bw94's walk, which reads \\(L\[p\]\\) and \\(T'\[p\]\\) at the same arbitrary index: *"Unless both \\(T\\) and \\(L\\) fit in the machine's cache, this could cause two (capacity) misses per iteration."* So it packs both into one 32-bit word, limiting \\(N\\) to \\(2^{24}\\):

\\\[ U\[i\] = 256 \\cdot T'\[i\] + L\[i\]. \\\]

The name is literal: \\(T\\) and \\(L\\), merged. Each step unpacks the byte with a mask and the link with a shift, which costs instructions, *"but the near-halving of the cache miss rate is potentially valuable"*. The table is also a byte a row smaller: \\(4N\\).

What one row costs, bit by bit

bw94 keeps T′ and L in separate arrays, so a step can miss twice. mergedTL packs both into one word. indexF keeps only T, in 20 bits, and finds the byte from the counts. bzip2 and mibz2 pack the byte with the reversed link.

bzip2 uses mergedTL, with the reversed link stored as an absolute position, so its walk needs no \\(C\[c\]\\) addition. Its manual credits the change with a real gain: *"Single array implementation of the inverse BWT. This significantly speeds up decompression, presumably because it reduces the number of cache misses."*

## 3\. Spending less memory: indexF and the count trees

*"Can we reduce memory use whilst retaining useful decompression speed? Interestingly, yes."* The relation in \\(F\\), \\(F\[T\[i\]\]\\) precedes \\(F\[i\]\\), needs only \\(T\\) to walk. If the decoder reads the output bytes from \\(F\\) instead of \\(L\\), it needs \\(L\\) only while it builds \\(T\\), so \\(T\\) can be written over it. \\(F\\) needn't be stored either: it is sorted, so the 256 running totals in \\(C\\) describe it, and \\(F\[p\]\\) is the byte whose bucket contains \\(p\\). *"No free lunch, alas"*: finding it is a binary search in \\(C\\).

**indexF**, named for indexing into \\(F\\), keeps \\(T\\) alone in \\(N\\) 20-bit words (so \\(N \\le 2^{20}\\)): a 16-bit array plus a nibble array, \\(2.5N\\) bytes. It starts the walk at \\(T\[I\]\\), because reading \\(F\\), the byte \\(L\[I\]\\) is found at \\(F\[T\[I\]\]\\). bzip2 ships it as `-s`, *"a modified algorithm which only requires 2.5 bytes per block byte … albeit at about half the normal speed"*, with the search as `BZ2_indexIntoF`. What it gives up:

- **A search per byte,** eight or nine comparisons where mergedTL has a mask. The 1 KB of counts stays in cache, so the search costs instructions, not misses, and since the next position is \\(A\[p\]\\) alone, a processor can overlap it with the next load.
- **Two arrays again.** The 20-bit entry is split between the 16-bit array and the nibbles, so a step can touch two cache lines, the cost mergedTL removed.
- **A cap of \\(2^{20}\\) bytes per block,** which bzip2's 900,000-byte maximum fits.

**The trees.** *"It is instructive to ask just how little storage we can get away with"*: \\(L\\) and the counts alone. Any \\(T\[j\]\\) is \\(C\[c\]\\) plus the number of \\(c\\)'s before \\(j\\) in \\(L\\), but counting them means scanning \\(L\\), \\(O(N)\\) a step and \\(O(N^2)\\) in all: *"absurdly inefficient"*. Seward buys speed back with 256 binary trees of partial counts, one per byte value. Each node counts its byte in an aligned power-of-two segment of \\(L\\), an empty subtree costs nothing, and a node budget sets how deep the trees grow. To count the \\(c\\)'s up to \\(j\\), descend \\(c\\)'s tree, adding the left child's count whenever \\(j\\) lies to the right, and scan \\(L\\) only for the last segment. The [FM-index](<https://en.wikipedia.org/wiki/FM-index>), published the year before, asks the same question of \\(L\\) to search text without decompressing it, and answers it with counts stored every so many rows and a scan between them.

A count tree answers a rank query

The tree for byte a over L = nnbaaa, padded to 8 positions. Counting the a's up to position 4 takes one left-child count and a one-byte scan, so T\[4\] = C\[a\] + 2 − 1 = 1.

**tree** stores a node in three 32-bit words. **treeopt** packs a node into 54 bits, two words, and a leaf into one, fitting nearly twice as many nodes in the same space, which halves the scans. With \\(L\\), tree-1M and treeopt-1M take \\(2N\\) bytes and treeopt-500k \\(1.5N\\).

## 4\. The space–time tradeoff

The machine was a 400 MHz AMD K6-III with 128 MB of memory and a 256 KB second-level cache. Blocks were at most 10⁶ bytes, 1 MB, and each was inverted ten times. He tested six files of at least a megabyte, four of them from the [Canterbury Corpus](<https://corpus.canterbury.ac.nz/descriptions/>). The last column says what each shares with mibz2's own [benchmark corpora](<https://lmmx.github.io/mibz2/pr-preview/pr-379/benchmarks/index.md>):

| File | MB | What it is | Closest in mibz2's corpora |
| --- | ---: | --- | --- |
| bible | 4.05 | The King James Bible: English prose | `text`, though ours is the crate's own source code, repeated, so more redundant |
| world192 | 2.47 | The CIA World Factbook, 1992: an English record per country | Wikidata's subject matter, though not its JSON format |
| ecoli | 4.64 | The *E. coli* genome: four letters, a, c, g and t, and notoriously hard to compress | No counterpart. Its tiny alphabet is the case the tree algorithms handle best |
| kennedy | 1.03 | An Excel spreadsheet, full of long runs of one byte | `runs`, and at the extreme `onebyte` |
| netscape | 7.68 | Netscape Navigator 4.75: x86 machine code | `records`, binary and structured |
| bashtgz | 1.71 | bash 2.04's source, gzipped: already compressed, so close to random | `random` |

Each algorithm's memory for a 1 MB block, and how fast it inverted each file, in MB/s. The fastest on each file is in bold, and mergedTL is the fastest on every one:

|  | MB | bashtgz | bible | ecoli | kennedy | netscape | world192 | Mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| basis | 5.0 | 1.54 | 1.72 | 1.53 | 2.46 | 2.32 | 1.65 | 1.86 |
| bw94 | 5.0 | 1.58 | 1.77 | 1.57 | 2.54 | 2.41 | 1.69 | 1.93 |
| mergedTL | 4.0 | **2.49** | **2.76** | **2.48** | **4.72** | **3.51** | **2.65** | **3.10** |
| indexF | 2.5 | 1.20 | 1.36 | 1.30 | 1.71 | 1.67 | 1.31 | 1.43 |
| tree-1M | 2.0 | 0.032 | 0.211 | 0.325 | 0.099 | 0.044 | 0.207 | 0.153 |
| treeopt-1M | 2.0 | 0.059 | 0.301 | 0.342 | 0.167 | 0.099 | 0.298 | 0.211 |
| treeopt-500k | 1.5 | 0.032 | 0.218 | 0.339 | 0.100 | 0.036 | 0.198 | 0.154 |

The paper gives these numbers only as tables. Plotting each algorithm's average speed against its memory shows the space–time tradeoff between them:

Seward's measured speed–memory choices

Each algorithm's average rate over the six files against its memory, from Seward's Tables 1 and 3, on a 400 MHz K6-III in 2001. The paper gives the numbers as tables; the plot is ours.

It isn't a smooth curve on which each saved byte costs speed. mergedTL is *"on average 61% and 67% faster than bw94 and basis respectively, at the same time using less memory"*: it beats both on both axes, by changing how the next link is read. indexF runs at 46% of its speed in 62.5% of its memory. Below that, half a byte a row less costs a factor of seven.

Seward then asks why. Simulating the K6-III's cache with cacheprof, his own profiler, he counted what each does per byte of bible. bw94 runs 18 instructions, with 9 memory references and 1.63 cache misses; mergedTL runs 20 instructions, with 8 references and 0.88 misses. So mergedTL does 11% more work and takes 54% as many misses. 89.4% of those misses are the walk's reads of \\(U\[p\]\\), and 78.6% of those reads miss, because \\(T\\) holds *"a completely arbitrary permutation"* that the decoder *"is obliged to traverse"*. Seward concludes that *"there appears to be no way to avoid a miss rate approaching one per \\(L\\)-byte"*, and mergedTL is *"a realistic upper bound on inversion performance"*.

kennedy inverts nearly twice as fast as bashtgz because its long runs make \\(T\\) point each row at its neighbour, which is usually already cached; random input sets the floor near 2.5 MB/s. And the tree algorithms vary 10:1 across files, against mergedTL's 1.9:1, because a small alphabet such as ecoli's four letters lets a few deep trees shorten every scan.

bzip2 itself decompresses bible at 67% of mergedTL's inversion rate, while also doing its Huffman and move-to-front decoding, so *"no amount of re-engineering of bzip2 can increase its decompression speed much further"*. The paper ends on a caveat:

> Such observations only hold true whilst cache sizes are much smaller than block sizes. Future growth in cache sizes may invalidate such findings.

## 5\. Four blocks sharing one cache

The K6-III's cache held 256 KB, against mergedTL's 4 MB for one block. Our workstation has a 20 MiB last-level cache, and a full bzip2 block, up to 900,000 rows at 4 bytes each, needs a 3.6 MB table. Read literally, the caveat has come true.

Caches against tables, then and now

One scale, in megabytes. In 2001 one block's table was far larger than the cache; today one fits several times over, but four take most of it. Table sizes are computed for full blocks, not measured cache occupancy.

Two things kept Seward's sentence true anyway. A modern core keeps many misses in flight if it is given independent loads, so mibz2's [walk](<https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs#L927-L961>) advances 32 separate pieces of the chain in turn and [runs 8.6 times as fast as one chain](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-31-room-walk-overlap/index.md>).

And a decoder doesn't invert one block at a time. Four workers keep four tables live, 14.4 MB before any of their other buffers, in one cache they all share, and [decoding with four costs more CPU time per byte than decoding with one](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-01-inverse-bwt-memory-interference/index.md>): the workers finish sooner together, but compete for the memory system on the way. With the cache divided between blocks, caches are again much smaller than the tables being walked.

On Seward's axes a smaller table is a step down in speed. With four blocks sharing a cache, it also leaves more of the cache to the other three. Could the same links be stored in much less space, and would the lighter memory load make up for the extra work of reading a compressed entry? The first valid comparison answers both ways at once. The compact walk costs 14.85% more CPU time on one core and 15.95% less with four workers walking together. That result is about this traversal, measured on its own; what it means for the whole decoder is still open.

## 6\. A smaller table, a busier lookup

The [census of the navigation table](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-29-compact-navigation-tables/index.md>) found that most of its links follow simple patterns. An earlier format built on that, [FOR-14](<https://github.com/lmmx/mibz2/pull/130>), packed 14-bit offsets into fixed records. It halved the table, but extracting the packed bits made each step so much longer that the walk lost even with four workers, where it took 1.45 times as long as the ordinary walk.

Compact30 keeps each offset directly addressable instead. Its name means thirty rows per record, not thirty-bit pointers. A 64-byte record, aligned to a cache line, holds thirty 16-bit offsets and a 4-byte header. In an ordinary record the header carries a base position and the byte the thirty rows all output, and a row's next position is the base plus its offset. A group of rows that start with different bytes, or whose next positions are too far apart for 16-bit offsets, is stored instead as a RAW group: thirty full 4-byte entries, padded to 128 bytes.

One ordinary compact30 record

Thirty rows in one cache line. RAW groups, segment starts and parking are handled outside this ordinary case.

On the measured corpus that brings the table from 4 bytes per row to 2.164, 45.9% less: about 1.95 MB for a full block instead of 3.6 MB, and 7.79 MB instead of 14.4 MB for four blocks at once.

Read against Seward's paper, the record combines three of its ideas. Its offsets rest on the ordering that makes \\(T\\) cheap to build: within one byte's bucket the links increase with the row, so a group's links are a base plus small offsets, smallest where the byte's occurrences in \\(L\\) sit close together, as in kennedy's runs. In Wikidata's blocks they usually do: about 77% of a full block's rows fall in 32-row groups that each come from a single run in \\(L\\). The header's byte is indexF's idea, emitting from \\(F\\) so that the walk needs no \\(L\\), with one byte stored per group in place of a search. And keeping the header and the offset in one cache line is mergedTL's. At 2.164 bytes a row, the record sits between treeopt-1M and indexF on Seward's memory axis, and aims at mergedTL's speed.

The step costs more, though. An ordinary compact30 step compiles to 21 instructions and four data loads; the conventional step is 16 instructions and three loads. Two of compact30's loads, the offset and the header, are both addressed from the current row, so neither waits for the other, but they are still two loads where the conventional step needs one. The [walker's journal](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md>) has the annotated disassembly.

So the question is which matters more when several cores share the machine: the extra instructions, or the smaller table.

## 7\. Timing the walk on one core and on four

The comparison times two versions of the same traversal and assembly. The control walks mibz2's 4-byte table; compact30 walks its records with the same 32-lane walk. Every walk's output was checked byte for byte against the decoder's.

Only the walk is timed. Before each walk, the control builds its table with the decoder's own builder, while compact30 copies in records made beforehand by a reference builder, so the comparison measures how fast each prepared table is walked, not what it costs to build. The two preparations may also leave different data in cache.

The [protocol](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md>) used one corpus of 434 blocks (207 million rows) and 20 paired rounds, on one pinned core or four. With four, each worker prepared its next block, all four started walking together, and none checked its output until all had finished, with blocks dealt so that each worker had about the same number of rows. CPU cost is the threads' own CPU time, summed, per row. Completion time is the time from the first walk's start to the last walk's end in each step, summed over the steps. Neither is the decoder's elapsed time.

|  | Conventional | Compact30 | Compact30 ÷ conventional (95% interval) |
| --- | ---: | ---: | ---: |
| CPU time per row, one core | 1.221 ns | 1.402 ns | 1.148 (1.147–1.149) |
| CPU time per row, four workers | 2.055 ns | 1.727 ns | 0.840 (0.838–0.843) |
| Completion time per pass, four workers | 114.45 ms | 97.97 ms | 0.856 (0.852–0.860) |

The intervals describe variation across the twenty rounds, on one machine and one corpus. Before the run, the protocol fixed what would justify building a direct constructor: the upper end of the four-worker CPU ratio at most 0.865, and of the completion ratio at most 1.02. The upper ends were 0.843 and 0.860. The [analysis](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/c30-analysis.json>) and an [independent reconstruction from the individual walks](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/researcher-review/independent_review.md>) agree on every figure.

CPU time per row, one core and four workers

Each walking thread's CPU time, summed and divided by rows. Building the tables is not included.

The gain is spread across the workload. All 230 full-sized blocks were cheaper with compact30 at four workers, taking each block's average over the rounds, and compact30 used less CPU in 1,138 of the 1,140 full-sized four-worker steps. These are checks within one experiment, on blocks that repeat each round, not independent trials. The 204 short blocks were slower with compact30, but together they hold 8,324 rows, 0.004% of the total.

The run also had to show that the four walks started together, within 2% of each step's length. An earlier complete run failed that check, because waking sleeping workers delayed some walks by tens of microseconds, and it doesn't count. This run used a spinning start barrier instead, with nothing else changed; its worst start skew was 0.023%.

## 8\. Four workers cost the conventional walk far more

From one core to four workers, the conventional walk's CPU cost per row rises from 1.221 ns to 2.055 ns, 68% more. Compact30's rises from 1.402 ns to 1.727 ns, 23% more. Both run under the same four-worker conditions, and the conventional table pays much more for them.

FOR-14 showed the same pattern earlier: from one block to four, the conventional walk's cost per row rose 56% and FOR-14's only 8%. Its smaller table removed most of the four-worker penalty, but each of its steps started 1.40 ns per row behind, three times what it saved. Compact30's step is cheap enough to come out ahead.

The obvious explanation is the table's size. Four conventional tables take about 14.4 MB, four compact30 tables about 7.79 MB, and the test machine's four cores share a 20 MiB last-level cache with everything else the workers use. Smaller tables could mean fewer evictions and less contention for memory, and they could also change address-translation costs.

Navigation tables for four full blocks

The tables' sizes on the measured corpus. How much of each stays in cache, and how much memory traffic each causes, was not measured.

That explanation hasn't been tested. The run recorded no cache-miss or memory-controller counters, and the two arms prepared their tables differently before each walk, which by itself could change what is in cache when the walk starts. The size difference makes the cache explanation credible; it doesn't measure it.

The result also stops short of the decoder. A decoding worker doesn't spend all its time walking tables: it also reads input, reconstructs the last column, builds the table, assembles segments and writes output. Some of its walks run alone, and others overlap two or three neighbours rather than all four. So compact30's 14.85% loss on one core matters as much as its gain at four, and how much of a whole decode runs under four-way contention hasn't been measured.

## 9\. The table still has to be built

The records in this comparison came from a reference builder that first computes every row's successor in a full-sized temporary array, then converts the result into compact30 records. That kept construction out of the measurement, but a decoder can't afford it. A direct builder has to produce the records from the block's last column and its byte counts, without the temporary array.

It also has to compete with the [decoder's current builder](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/src/bwt.rs#L253-L318>), which places every row in one counting-sort pass and writes runs of sixteen equal bytes sixteen entries at a time. Compact30's builder has to write group headers, check offsets against the 16-bit limit, and move groups to RAW, and still stay close to that.

The traversal result sets the budget. A rough planning model puts the decoder's current table building at 0.592 CPU-seconds per decoded GB, and the traversal with its assembly at 1.847. If b and w are compact30's building and traversal costs as fractions of today's, the saving is about 0.592 × (1 − b) + 1.847 × (1 − w) CPU-seconds per GB, and the target is at least 0.25.

If the four-worker traversal ratio carried over to the decoder, a builder costing the same as today's would save about 0.295 CPU-seconds per GB. The builder could be up to 7.5% slower than today's at the central estimate, or 6.7% using the conservative end of the interval. That is a budget conditional on the traversal gain surviving inside the decoder, built on approximate phase costs; meeting it would justify the next test, not settle the question.

The next step is a design for that builder, set out in the [constructor brief](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md>). Its main difficulty is already clear. Groups that straddle two bytes' rows can be found from the byte counts before any row is placed, but a group whose offsets turn out too wide is only known when its last row lands, and has to move to RAW then. The builder has to produce exactly the records tested here, and the first measurements will be whether it does and what it costs. After that, a whole-decoder comparison can show how much of the four-worker gain survives in practice.

For now, a smaller navigation table has beaten the conventional one in a controlled four-worker traversal while losing on a single core. That is a strong reason to study the memory contention behind it, and a concrete budget for building the table.

---

**Research records:** [Traversal protocol and the runs behind it](<https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-traversal-protocol.md>) · [This result's records](<https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4>) · [Independent reconstruction](<https://github.com/lmmx/mibz2/tree/f567817/docs/journal/2026-11-02-compact30-traversal-protocol/i9-attempt-4/researcher-review>) · [Compact30 walker](<https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/examples/c30walk.rs>) · [FOR-14 trial](<https://github.com/lmmx/mibz2/pull/130>).
