# External research report 1: practitioner sources (verbatim)

A research input, not a finding of this repository. It is a ChatGPT research report that
the maintainer relayed into the campaign on 2026-10-27, preserved verbatim in the block
below. Its claims are leads, checked against the primary sources in
[2026-10-27-research-synthesis.md](2026-10-27-research-synthesis.md).

The report's inline links reached us as placeholder characters (`⁠￼`), which are kept as
received. The "Links" section after the verbatim block reconstructs their targets. It is
ours, not the report's.

## The report, verbatim

```text
Yes. I found several useful sources, including one I think deserves more attention than another theoretical paper: the engineering notes from people who’ve built fast decompression libraries and measured what actually made them faster.

The interesting pattern is that their biggest gains often come from mundane details—memory layout, cache pollution, buffer lifetimes, lookup-table sizes—not necessarily more sophisticated algorithms.

The most useful sources

1. libsais — source and engineering notes⁠￼

Closest to our inverse-BWT work. Its author explicitly warns that performance is sensitive to memory speed and software prefetching, and that more threads do not necessarily help once memory bandwidth saturates.

For mibz2: compare its table representation, prefetch distance and cache behaviour with our two-step traversal experiment (issue #6). The key question is why its extra construction work pays off in its workload but not ours.

2. rapidgzip — performance changelog⁠￼

A remarkably detailed record of actual optimisations. For example, removing unnecessary buffers improved bzip2 decompression from 344 to 388 MB/s in one reported test. Other changes reduced memory use, avoided extra allocations and improved Huffman decoding.

For mibz2: particularly relevant to fused decoding and reducing per-worker scratch memory (issue #8). This is a practical source of small, measurable improvements.

3. indexed_bzip2 — parallel bzip2 decoding⁠￼

Particularly relevant because it actually handles the same compression format. The project discusses parallel bzip2 decoding, random seeking and practical performance across thread counts.

For mibz2: block discovery (issue #10), scheduling, and the interaction between decoding throughput and thread count. Unlike libsais, this can be tested directly on identical .bz2 inputs.

4. Slashing the Time for BWT Inversion — Kärkkäinen, Kempa & Puglisi (2012)⁠￼

The foundational paper behind much of our work. It demonstrates that cache misses during BWT inversion can be reduced through changes in traversal and representation, reporting substantial speedups over the previous approach.

For mibz2: still worth consulting against actual code, especially to distinguish the complete construction cost from the traversal-only benefit.

The particularly useful lesson

The rapidgzip changelog⁠￼ reads almost like a diary of the work we’re doing.

It documents not only wins, but why the wins happened: avoiding unnecessary allocations, changing Huffman lookup tables, improving memory locality, removing redundant data structures, and sometimes replacing complicated work with ordinary memcpy. It even includes work on bzip2 decoding.

I also found the Rapidgzip research paper⁠￼, which describes parallel decompression and cache prefetching. Although its primary subject is gzip, the engineering approach is highly relevant.

What I’d actually do with these findings

Our work	Most relevant external experience	Practical next question
Two-step BWT traversal (#6)	libsais and the 2012 inversion paper	Are we paying for a construction pass the reference implementation avoids or combines with other work?
Precopy (#9)	Kärkkäinen–Puglisi’s repeat detection	Can copying opportunities be discovered without recording every visited row?
Vectorised table construction (#7)	libsais memory-layout and prefetch techniques	Can we increase store throughput without adding expensive setup?
Fused front end (#8)	rapidgzip’s buffer and Huffman optimisations	Which intermediate allocations can we eliminate while preserving decoder throughput?
Parallel block scanner (#10)	indexed_bzip2 and rapidgzip	At what worker count does parallel discovery outweigh its coordination cost?

I wouldn’t turn these into five additional projects. They mostly provide better reference implementations and sharper questions for work already planned.

My strongest recommendation is to have Claude examine rapidgzip’s bzip2-specific implementation history alongside the existing research note, especially before tackling memory reduction (issue #8). It’s likely to be more immediately useful than another broad literature search.

And the important distinction remains: a technique is promising because it addresses a cost we’ve measured in mibz2, not merely because another library uses it.

That’s the practical value of this research: fewer speculative experiments, better-informed implementations, and more actual decoder speedups.
```

## Links (reconstructed by us, not part of the report)

The report names these sources. Their targets are filled in from the names:

- libsais: https://github.com/IlyaGrebnov/libsais
- The rapidgzip changelog: https://github.com/mxmlnkn/rapidgzip/blob/main/CHANGELOG.md
- indexed_bzip2: https://github.com/mxmlnkn/indexed_bzip2
- Kärkkäinen, Kempa and Puglisi (2012), "Slashing the Time for BWT Inversion":
  https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf
- The Rapidgzip paper (Knespel and Brunst, HPDC 2023): https://arxiv.org/abs/2308.08955
