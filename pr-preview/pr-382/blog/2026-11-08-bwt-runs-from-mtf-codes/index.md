[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/index.md>)

# Reading the BWT's runs off bzip2's move-to-front codes

bzip2's move-to-front codes mark where each run of the BWT's last column starts, so the runs could be listed without writing the column. It is not built.

8 November 2026 · 15 min read · [Open question](<#6-where-does-this-stand>)

bzip2's codes already say where the BWT's runs of repeated bytes are. Today mibz2 expands those codes into an array of every byte, then reads that array again to build the table it navigates the BWT with. It could instead keep the runs as it decodes the codes, and build a smaller navigation structure straight from them, without the big intermediate array.

Take the word `banana`. After the BWT its bytes are \\(L=\\texttt{nnbaaa}\\), and bzip2's codes for them say, in effect: change to `n`, repeat it once; change to `b`; change to `a`, repeat it twice. That is already the three groups `nn | b | aaa`, as three run records: (`n`, 2), (`b`, 1) and (`a`, 3). The navigation structure can be built from those three records, and the six bytes never written out and scanned again.

**What is proved, and what isn't.** That the codes give the runs, and the bound on how many codes there are, are mathematical results (sections 2 and 4). The inversion still has to write every byte, so it is not faster in principle, and the one such structure mibz2 has tried navigated more slowly than today's table (section 5). Nothing here has been built or timed.

Some background, for what follows. Before a decoder can undo the Burrows–Wheeler transform (BWT), it rebuilds the transform's output: a column of bytes, \\(L\\), one for each of the block's \\(n\\) rows, up to 900,000 of them. bzip2 stores that column in coded form. Its *move-to-front* (MTF) stage writes each byte as a small number, its position in a list of recently used bytes, and it writes repeats of the last byte as a separate count. mibz2, our bzip2 decoder, turns those codes back into all \\(n\\) bytes of \\(L\\), then reads \\(L\\) again to build its table for the inverse BWT, the T-vector, at four bytes a row.

Why would a decoder want the runs? [An earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation/index.md>) showed that the inverse BWT's next-row map needs only one record per run of \\(L\\), rather than one entry per row. Published structures built from those records, *move structures*, step from row to row without a search. To build one, a decoder needs the runs.

**This post's observation is that bzip2's codes list the runs of any block, as they do `banana`'s.** A code that names a byte other than the last one starts a run; a repeat count extends it. Apart from the block's first byte (section 2), nothing else starts or ends a run. So the runs, and from them a move structure, could be built from the codes directly, without writing the \\(n\\)-byte column and scanning it again. By our own accounting, building them would take time and memory that grow with the number of codes and runs rather than with \\(n\\), and the number of codes is itself bounded by the runs (section 4). It doesn't make the inversion that follows any shorter: that still takes time proportional to \\(n\\), as any inversion must, and that bound is already published.

That observation is not new to mibz2: it was recorded in [issue \#42](<https://github.com/lmmx/mibz2/issues/42#issuecomment-6451443792>) in October. The same issue then built and timed a move structure for mibz2's walk, and it lost by 90–108%, a loss that a run list handed over for free would not have changed (section 5). That experiment did not time a builder fused with MTF. What this post adds is the mechanism worked through, its edge cases, what it would cost in theory, and a search for prior work. It changes nothing in the decoder. The costs in section 4 are our own accounting, which an independent review has since checked (section 6).

[Section 1](<#1-how-does-bzip2-write-banana>) decodes the codes of `banana` by hand. [Section 2](<#2-why-does-every-literal-start-a-run>) proves that the codes give every run, and section 3 turns the runs into a move structure. [Section 4](<#4-what-would-it-cost-in-theory>) gives the cost in theory and [section 5](<#5-what-has-mibz2-measured>) what mibz2 has measured. [Section 6](<#6-where-does-this-stand>) says what is open.

## 1\. How does bzip2 write `banana`?

Suppose you compress the word `banana` with bzip2. The BWT sorts the word's six rotations and keeps their last letters, \\(L=\\texttt{nnbaaa}\\), and the number of the row that holds the word itself ([the earlier post works it through](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>)). The MTF stage then codes \\(L\\) one byte at a time.

It starts from the list of the bytes the block uses, in byte order: `a b n`. To write a byte, it takes the byte's position in the list, counting from 0, which we call its *MTF index*, and moves the byte to the front. The first byte, `n`, is at index 2, and the list becomes `n a b`. The next `n` is now at the front, index 0.

Index 0 means "the same byte again", and bzip2 never writes it as a number. It counts a stretch of zeros and writes the count in two symbols, `RUNA` and `RUNB`. They are digits worth 1 and 2 in the first place, 2 and 4 in the second, 4 and 8 in the third, and so on, added up: `RUNA` is one zero, `RUNB` two, `RUNA RUNA` three. The symbols for indices 1 and up we call *literals*, and they are stored one higher than their index, so that the symbol values 0 and 1 are free for `RUNA` and `RUNB`. Index 2 goes into the file as the symbol 3.

Coding \\(L=\\texttt{nnbaaa}\\) this way gives five symbols, `3 RUNA 3 3 RUNB`, then an end-of-block symbol. The table gives each step's MTF indices and the bzip2 symbols actually written for them:

| Bytes | List before | MTF indices | bzip2 symbols |
| --- | --- | --- | --- |
| `n` | `a b n` | 2 | 3 |
| `n` | `n a b` | 0 | `RUNA` |
| `b` | `n a b` | 2 | 3 |
| `aaa` | `b n a` | 2 0 0 | 3 `RUNB` |

The `RUNA` is one zero, one more `n`; the `RUNB` is two zeros, two more `a`s.

The decoder does the same in reverse: a literal symbol \\(s\\) takes the byte at index \\(s-1\\) ([mibz2's `s - 1`](<https://github.com/lmmx/mibz2/blob/68d934b/src/mtf.rs#L66>)), writes it, and moves it to the front; a count of zeros writes that many copies of the front byte. [mibz2's scalar loop](<https://github.com/lmmx/mibz2/blob/68d934b/src/mtf.rs#L32-L74>) is about forty lines long, and it writes every byte into \\(L\\) and counts each byte value as it goes.

From `banana`'s symbols to its runs, and to the next-row map

**1\.** Each green literal, symbol 3 for MTF index 2, writes a byte that differs from the one before, so it starts a run. `RUNA` and `RUNB` repeat the last byte: one copy and two.

**2\.** So the symbols give one record per run: its byte, where it starts, its length, and *k*, the copies of that byte before it.

**3\.** At the block's end the counts fix where each byte's rows begin: `a` at 0. The record `a`, start 3, *k* 0 becomes rows 0–2 → positions 3–5.

Conceptual, worked by hand and checked by a script (see the PR). Green marks the literal symbols in the first frame and the run `aaa` in the last. The block-end symbol, which follows `RUNB`, is left out.

The runs came from the symbols, not from \\(L\\). So a decoder that wants a move structure has two routes to it, and only one of them writes \\(L\\):

Two routes from bzip2's symbols to a move structure

**1\.** A decoder that wants a move structure would write all *n* bytes of *L*, then scan them to find the runs.

**2\.** The symbols already mark where each run starts, so the runs could be listed directly and *L* never written.

Conceptual. Neither route is in the decoder: today mibz2 builds its T-vector from *L*, not a move structure, and the route from the codes has not been built or timed.

## 2\. Why does every literal start a run?

Because the front of the list is always the byte just written. A literal names an index of 1 or more, and every byte in the list is different, so a literal always writes a byte that differs from the one before it. A count of zeros always repeats it. So a run of \\(L\\) is exactly one literal followed by at most one count, and the run's length is one plus the count.

That makes the number of runs, \\(r\\), a count of the literals, with one exception, which comes from how the list starts. Before the first byte is written, the front of the list is the block's smallest byte, which hasn't been written yet. If \\(L\\) starts with that byte, bzip2 writes it as index 0, so the block's first run starts with a count, not a literal. The column of `aaaa` is `aaaa`, and bzip2 codes it as `RUNB RUNA` (two, plus two in the second place): one run, no literal at all. So

\\\[ r=(\\text{number of literals})+\[\\,L\\text{ starts with the block's smallest byte}\\,\]. \\\]

The other edge cases are smaller. A count is never zero: each `RUNA` or `RUNB` adds at least one, so a count that is present extends its run. The block's last run ends at the end-of-block symbol, so a decoder that lists runs must close the open run there; mibz2's loop already writes a pending count before it checks for the end. And a list of runs needn't be maximal to be correct. A decoder that cuts a run in two where it crosses a checkpoint, as mibz2's checkpointed MTF loop does, would give two records with the same byte, which still describe \\(L\\) exactly. It only costs a record.

We checked the argument with a short script that models bzip2's MTF and `RUNA`/`RUNB` rules (not mibz2's code). On 20,009 strings, random ones and periodic ones among them, it listed the runs from the symbols, compared them with the runs of the column computed directly, and undid the transform from the runs alone. Every case matched, the first-run exception included.

## 3\. How do the runs become a move structure?

From each run, the decoder needs four numbers: its byte \\(c\\), its start \\(p\\) in \\(L\\), its length \\(\\ell\\), and \\(k\\), the number of copies of \\(c\\) written before it: copies, not earlier runs, so an earlier run `aa` adds 2. The MTF decoder already counts each byte value as it writes, so \\(k\\) is that count at the run's start. In `banana` every run is its byte's first, so every \\(k\\) is 0; in `aababa`, below, two runs share a byte and \\(k\\) is not 0.

The inverse-BWT navigation post showed which rows a run takes over ([its section 2](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation/#2-why-does-every-run-give-one-shift>)). Write \\(C(c)\\) for the number of bytes in the block smaller than \\(c\\): the row where \\(c\\)'s rows begin in \\(F\\), the sorted first column. Then the run's rows begin at \\(a=C(c)+k\\), and the next-row map \\(\\Psi\\) sends each of them to the matching position of the run:

\\\[ \\Psi(a+t)=p+t,\\qquad 0\\le t\<\\ell. \\\]

One value has to wait. \\(C(c)\\) depends on the counts of every smaller byte, so it is known only at the end of the block. The records can be written while decoding, with \\(k\\), and given their rows once the counts are final. In `banana` the counts are `a` 3, `b` 1, `n` 2, so the `a` rows begin at 0, `b` at 3 and `n` at 4: the record (`a`, start 3, length 3, \\(k\\) 0) becomes rows 0–2 to positions 3–5, as in the last frame above.

**\\(k\\) matters when a byte has more than one run.** Take the word `aababa`. Its column is \\(L=\\texttt{babaaa}\\), with the runs `b`, `a`, `b` and `aaa`, and its sorted first column is \\(F=\\texttt{aaaabb}\\), so \\(C(\\texttt{a})=0\\) and \\(C(\\texttt{b})=4\\). The first `a` run, at \\(p=1\\), has \\(k=0\\): it takes row 0, so \\(\\Psi(0)=1\\). The run `aaa`, at \\(p=3\\), comes after one `a`, so \\(k=1\\) and its rows begin at \\(a=C(\\texttt{a})+k=1\\): \\(\\Psi(1)=3\\), \\(\\Psi(2)=4\\) and \\(\\Psi(3)=5\\). The `b` runs do the same from row 4: the first takes row 4, and the second, with \\(k=1\\), takes row 5, so \\(\\Psi(5)=2\\). The runs of one byte share out that byte's rows of \\(F\\), in the order they appear in \\(L\\), without overlapping, and \\(k\\) says where each run's share begins. Without it, both `a` runs would claim row 0.

Two runs of one byte share its rows in *F*

**1\.** In `aababa`'s column, `a` and `b` each have two runs. A byte's second run has *k* 1, so its rows start one past its first run's.

**2\.** So the `a` rows of *F* are shared out in *L* order: row 0 to the first run, rows 1–3 to `aaa`. The two `b` runs take rows 4 and 5.

Conceptual, worked by hand and checked by a script. Green marks the two `a` runs; the band marks `aaa`, whose *k* of 1 sends its rows past the row the first `a` run takes.

Back in `banana`, a walk over the intervals needs one thing more. Each interval's record says where its rows go; its *anchor* names the interval that destination falls in. A walk that knows its current interval then finds the next one by stepping forward from the anchor, instead of searching all the interval starts ([section 1.1 of the inverse-BWT navigation post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation/#11-walking-the-intervals-without-a-search>)). The records arrive in \\(L\\) order, so their destinations rise. Sorted by byte, a counting sort over the \\(r\\) records, their row starts rise too. One merge of the two lists finds every anchor. Brown and Langmead build a *length-capped* structure, which splits long intervals so that the walk's average step is bounded, from exactly such a run list in \\(O(r)\\) time and space ([SEA 2026, Lemma 6](<https://arxiv.org/abs/2602.11029>)).

`banana`'s three run records, three intervals of a move structure

**1\.** Sorting the records by byte, a counting sort over *r* records, puts them in *F* order. Each starts at its byte's first row plus *k*.

**2\.** Each interval's anchor is the interval its destination falls in. The destinations, read in *L* order, rise, so one merge with the interval starts finds every anchor.

Conceptual: the table the earlier post's walk reads, built here from the records alone. A real block would also split its long intervals (length capping), which this example doesn't need.

## 4\. What would it cost, in theory?

**The saving is to handle the block's \\(m\\) symbols and keep \\(r\\) records, instead of writing and rereading \\(n\\) bytes.** That pays only if \\(m\\) is much smaller than \\(n\\), and on long runs it is. A run of length \\(\\ell\\) costs one literal and about \\(\\log\_2\\ell\\) count digits, because the digits' place values double. So, as proved at the end of this section, \\(m\\le r(1+\\log\_2(n/r))+1\\). At the \\(n/r=13.85\\) of the Wikidata corpus in section 5, that is at most about 4.8 symbols a run, or about 0.35 symbols a row: derived from the measured run count, not measured itself. Listing the runs is therefore sublinear in \\(n\\) only when \\(r\\) grows more slowly than \\(n\\). The inversion is not sublinear: it writes \\(n\\) bytes regardless. That sublinearity is not new: it follows from the input having coded the runs already.

In detail: while undoing MTF and the `RUNA`/`RUNB` counts, a decoder could keep one record per maximal run, merging a literal with the count after it (and any pieces of one run it was made to cut), together with a count per byte value. At the block's end it computes each \\(C(c)\\), gives each record its row start \\(a=C(c)+k\\), and builds a run-bounded move structure from the records. The \\(n\\)-byte column is never written or scanned. What follows is our own accounting of what that would cost. It is not a published result.

As above, \\(m\\) is the number of symbols in a block; write \\(\\sigma\\) for the number of byte values it uses, at most 256. On a word RAM, the usual model in which one machine word is read or written in one step, with each symbol handled in constant time (and with blocks of any length: bzip2 caps a block at 900,000 bytes, so the asymptotics are about that generalised model):

1. **Building the records** would take \\(O(m+r+\\sigma)\\) time: one pass over the symbols, a counting sort of the \\(r\\) records by byte, and the counts. It would hold \\(O(r+\\sigma)\\) words of working memory: the records and a count per byte value.
2. **Composed with a move structure built in \\(O(r)\\) time and space,** and walked in \\(O(n)\\) time, the inverse BWT would take \\(O(m+n+r+\\sigma)\\) time and \\(O(r+\\sigma)\\) extra working memory. Brown and Langmead give such a route from a run-length BWT ([SEA 2026, Lemma 6 and Theorem 9](<https://arxiv.org/abs/2602.11029>)), for a text with an end marker; condition 3 below gives one for bzip2's BWT, which has none.

That memory figure leaves out the compressed input, the \\(n\\) output bytes, the block's Huffman tables and other metadata, and the decoder's other buffers. It is not a memory bound for a whole bzip2 decompressor, which also undoes bzip2's first stage, a run-length code applied before the BWT, and checks a CRC. And the time is still \\(O(n)\\), because \\(n\\) bytes must be written. **Brown and Langmead already invert in \\(O(n)\\) time with \\(O(r)\\) extra space,** so the composition is no new inversion bound. What it would change is the route: no \\(n\\)-byte column between MTF and the move structure.

Each of those statements has conditions:

1. **A fixed alphabet.** Moving a byte to the front of the list costs up to \\(\\sigma\\) steps. For bytes that is a constant, and mibz2 does most moves with a few vector shuffles; for a large alphabet it would not be.
2. **The symbols streamed.** The bounds assume a stream of decoded symbols. mibz2 decodes a block's Huffman codes into an array of symbols first, which is \\(O(m)\\) memory, before MTF reads it ([`Symbols`](<https://github.com/lmmx/mibz2/blob/68d934b/src/block.rs#L23-L34>)). The \\(O(r+\\sigma)\\) figure needs Huffman decoding and MTF fused, or the array counted. Even fused, the block's Huffman *selectors* stay: one per group of 50 symbols, naming the table that group uses, all stored ahead of the symbols, so about \\(m/50\\) entries, which is \\(O(m)\\) state too. So an \\(O(r)\\) memory bound for the whole decompressor would have to account for them as well.
3. **No end marker.** Brown and Langmead's text ends in a unique terminator, `$`. bzip2's BWT sorts rotations with no terminator and stores the start row instead. A periodic block's map then has several cycles ([the appendix, section 6](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation-notes/#6-cycle-structure-and-necklaces>)). Their Theorem 9 bounds the walk's steps on average over one traversal of a single cycle, so that proof does not transfer unchanged. A later paper gives a route that needs neither. Brown, Sanaullah, Zhang and Langmead ([*Optimal-Time Move Structure Construction*, Theorem 3.9](<https://arxiv.org/abs/2603.22147>)) build a *balanced* move structure, in which every step takes constant time in the worst case, for any permutation, in \\(O(r)\\) time and space. Its inputs are each interval's start, the start of its image, and the order of the images. For \\(\\Psi\\) built from the records, these are the row starts in \\(F\\) order, the records' starts in \\(L\\), and each record's place in \\(L\\) order, all of which section 3's counting sort already gives. Nothing in that theorem assumes one cycle or a terminator. The inversion then starts at bzip2's stored start row, finds its interval in \\(O(r)\\) steps, and takes \\(n\\) steps of \\(\\Psi\\), writing each row's byte. A periodic block's cycle is shorter than \\(n\\), so the walk goes round it more than once, which writes the period the right number of times. A script that models this walk reproduced every string of up to eight letters over three letters, the 135 periodic ones among them. Applying that theorem to bzip2's \\(\\Psi\\) is our composition, not a result of the paper.

**The number of symbols is bounded by the runs.** This is an elementary lemma, not a new result, and its proof is short. Let the maximal runs of \\(L\\) have lengths \\(\\ell\_1,\\dots,\\ell\_r\\), which add up to \\(n\\), and count \\(m\\) with the end-of-block symbol, as in section 1.

First, a stretch of \\(q\\ge1\\) zeros takes exactly \\(\\lfloor\\log\_2(q+1)\\rfloor\\) count digits. With \\(t\\) digits, each worth 1 or 2 times its place value \\(1,2,4,\\dots\\), the totals run from \\(2^t-1\\) (every digit `RUNA`) to \\(2^{t+1}-2\\) (every digit `RUNB`), and every count in between is reached. A run of length \\(\\ell\\) that starts with a literal is that literal and \\(\\ell-1\\) zeros, so it costs \\(1+\\lfloor\\log\_2\\ell\\rfloor\\) symbols. The one exception of section 2, a first run of the block's smallest byte, has no literal and \\(\\ell\_1\\) zeros, so it costs \\(\\lfloor\\log\_2(\\ell\_1+1)\\rfloor\\). No other run starts with a count (section 2). Adding the end-of-block symbol,

\\\[ m = 1+\\sum\_{j=1}^{r}\\bigl(1+\\lfloor\\log\_2\\ell\_j\\rfloor\\bigr), \\\]

with the first run's term replaced by \\(\\lfloor\\log\_2(\\ell\_1+1)\\rfloor\\) in the exception. For `banana`'s runs of 2, 1 and 3, that is \\(1+2+1+2=6\\), the six symbols of section 1. Since \\(\\log\_2(\\ell+1)\\le1+\\log\_2\\ell\\), both cases give \\(m\\le1+r+\\sum\_j\\log\_2\\ell\_j\\). The logarithm is concave, so by Jensen's inequality the sum is largest when every run has length \\(n/r\\):

\\\[ m\\le r\\left(1+\\log\_2\\frac nr\\right)+1. \\\]

Cutting a run at a checkpoint changes the records, not the symbols, so it doesn't change \\(m\\). A second script checked the exact count, not only the bound, on every word of up to twelve letters over two letters and up to eight over three, and on the columns of 3,000 random and periodic strings.

## 5\. What has mibz2 measured?

**How small is \\(r\\)?** It has been counted once. In October, an uncommitted scratch counter ran over the bench's Wikidata corpus, `wikidata.bz2` (SHA-256 `82f65ace…`), and checked its map against the decoder's on every block. The 233 blocks of at least 65,536 rows hold 209.7 million rows, 99.996% of the corpus, and 15,148,086 runs: \\(r/n=0.0722\\), about 14 rows a run. Block by block, \\(r/n\\) ran from 0.053 to 0.200, with a median of 0.062. 71% of the rows lie in runs of 256 or more ([the journal](<https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md>), [issue \#42](<https://github.com/lmmx/mibz2/issues/42>)). That was one local run, on one corpus. The inverse-BWT navigation post said \\(r\\) had not been counted. It had, on that corpus. Its upper bound, \\(r/n\\le0.28\\), is on a different one, the primary Wikidata corpus `wikidata-20261026.bz2` of 434 blocks, where \\(r\\) has still not been counted. That post now says so.

**Would a move structure help mibz2?** Issue \#42 built one, length-capped as Brown and Langmead describe, and timed it, locally on a shared four-core virtual machine, at one thread, on those 233 blocks, over two runs ([the write-up](<https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-move-structure-walk.md>), [the result](<https://github.com/lmmx/mibz2/issues/42#issuecomment-6452138621>)). Walked as one chain, it took about 60% less time than the T-vector's chain. Walked as mibz2 walks, 32 chains interleaved so that their memory reads overlap, it lost. In cycles per decoded byte, locally:

| Step | Cycles a byte |
| --- | ---: |
| T-vector build + walk | 12.0 |
| Scan of \\(L\\) for runs | 0.9 |
| Build from runs | 6.7–8.1 |
| Move walk alone | 13.5–17.1 |

Each figure covers both runs, and the ranges cover the three interval-length caps tried. **The move walk alone, at 13.5–17.1 cycles a byte, cost more than the T-vector's whole build and walk, at 12.0, so skipping \\(L\\)'s reconstruction cannot make that walker win.** Each of its steps ran about 41 instructions against the T-vector's 16. Built from a run list handed over for free, as if by MTF, the move structure's build and walk together cost 80–99% more than the T-vector's. Reading the runs off the codes would save the scan, under one cycle a byte, and the writes of \\(L\\), which nobody has timed separately.

The scratch runs were about twice CI's figures, so they are indicative only. The result is for that one structure and that walk, on one machine and one corpus. It timed a walker, not the construction of this post: no builder fused with MTF was timed, and other move structures were not tried.

## 6\. Where does this stand?

**Is any of it new?** Not the observation, which mibz2 recorded in issue \#42. The rules it rests on are bzip2's format, as Julian Seward designed it, and they are visible in any decoder: [Go's](<https://go.dev/src/compress/bzip2/bzip2.go>) or the [Wuffs format notes](<https://github.com/google/wuffs/blob/main/std/bzip2/README.md>), which describe `RUNA` and `RUNB` as repeating "the 0th element". We searched for work that builds a run-length BWT or a move structure from MTF- or run-coded input, and found none. That is not evidence that none exists. The nearest work builds a run-length BWT from other inputs: from the plain text in space bounded by \\(r\\) ([Nishimoto, Kanda and Tabei, ICALP 2022](<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2022.99>)), from an LZ77 parse ([Kempa and Kociumaka, FOCS 2020](<https://arxiv.org/abs/1910.10631>)), and a move structure for any permutation, from its intervals, in \\(O(r)\\) time ([Brown, Sanaullah, Zhang and Langmead, 2026 preprint](<https://arxiv.org/abs/2603.22147>)).

**What is open** is the speculative question: does bzip2's coded form give a stronger algorithm or bound than a generic run-length BWT does? One point bears on it. The symbols and the column determine each other exactly, so the symbols hold no information the runs lack. Any gain would have to come from what the decoder computes cheaply on the way: avoiding the expansion of \\(L\\), its scan and an \\(n\\)-byte temporary, which is a shortcut in computation, not extra information about the permutation. So the research question, beyond composing known results, is about the model of input: what bounds on one-pass move-structure construction, upper or lower, can be proved for a decoder that reads only the stream of MTF and `RUNA`/`RUNB` symbols, that don't already follow from known run-length BWT and move-structure results? It is open, and we have not established that an answer would be new.

**How sure are we?** Fairly sure of the run-listing argument, which section 2 proves and a script checked on a model of bzip2's rules, and of the bound on \\(m\\), which section 4 proves. Sure of the costs in section 4 under their conditions: a stream of decoded symbols, with the selectors and any stored symbols counted separately, and no claim for the whole decompressor. Not sure at all that any of it is new, beyond what the format already implies.

**What did the review find?** An independent review, in the method of the project's researcher role and relayed from the maintainer through ChatGPT, read the published post on 8 November ([the review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6723926266>)). It found the substance sound. It gave the proof of the bound on \\(m\\) that section 4 now carries, and the route without an end marker in condition 3, which we checked against the paper before adopting it. It asked that the memory model and the novelty question stay as narrow as they were. It was not a fresh-reader test: the reviewer knew the code and the papers. A test with a reader who knows neither is still to be done.

## 7\. The argument in brief

bzip2's MTF codes write a literal exactly where a run of the BWT's last column starts, and a count of repeats for the rest of it, so the runs can be listed from the codes, with one exception at the block's start. Each run's byte, start, length and earlier count, with the block's byte counts, give its interval of the next-row map, and a merge gives the anchors a move structure needs. In our accounting, the records take \\(O(m+r+\\sigma)\\) time and \\(O(r+\\sigma)\\) words, without the \\(n\\)-byte column, and since \\(m\\) is at most \\(r(1+\\log\_2(n/r))+1\\), that time is sublinear in \\(n\\) on long runs. The inversion stays \\(O(n)\\), as Brown and Langmead already have it, and a balanced move structure gives it without an end marker. In the one setting mibz2 has measured, it would not have rescued the move structure tried there, whose interleaved walk alone cost more than the T-vector's whole build and walk.

The general point is that a stage's input is often already in the form a later stage wants, and the cheapest conversion is the one never done; whether it pays depends on what the later stage costs. The same runs could, for example, count compact30's pure-run groups without the column ([the appendix, section 4](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation-notes/#4-what-the-intervals-say-about-compact30>)).

---

**Sources:**

- [Issue \#42](<https://github.com/lmmx/mibz2/issues/42>): the run counts, the [observation](<https://github.com/lmmx/mibz2/issues/42#issuecomment-6451443792>) that MTF gives the runs, and the [timed move structure](<https://github.com/lmmx/mibz2/issues/42#issuecomment-6452138621>).
- [The run-structures journal](<https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md>) and [the move-structure walk journal](<https://github.com/lmmx/mibz2/blob/68d934b/docs/journal/2026-10-27-move-structure-walk.md>) (27 October 2026).
- [*The mathematical structure of inverse-BWT navigation*](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation/index.md>) and its [technical appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation-notes/index.md>).
- [Nathaniel K. Brown and Ben Langmead, *Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations*](<https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html>), SEA 2026, LIPIcs 371, paper 9 ([arXiv:2602.11029](<https://arxiv.org/abs/2602.11029>)).
- [Nathaniel K. Brown, Ahsan Sanaullah, Shaojie Zhang and Ben Langmead, *Optimal-Time Move Structure Construction*](<https://arxiv.org/abs/2603.22147>), arXiv:2603.22147 (2026; revised 22 October 2026), Theorem 3.9.
- [Takaaki Nishimoto, Shunsuke Kanda and Yasuo Tabei, *An Optimal-Time RLBWT Construction in BWT-Runs Bounded Space*](<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2022.99>), ICALP 2022, LIPIcs 229, 99:1–99:20.
- [Dominik Kempa and Tomasz Kociumaka, *Resolution of the Burrows–Wheeler Transform Conjecture*](<https://arxiv.org/abs/1910.10631>), FOCS 2020.
- bzip2's MTF and `RUNA`/`RUNB` rules: [Go's `compress/bzip2`](<https://go.dev/src/compress/bzip2/bzip2.go>), the [Wuffs bzip2 notes](<https://github.com/google/wuffs/blob/main/std/bzip2/README.md>) and [mibz2's `mtf.rs`](<https://github.com/lmmx/mibz2/blob/68d934b/src/mtf.rs>).
