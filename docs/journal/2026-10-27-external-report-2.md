# External research report 2: further detail from the sources (verbatim)

A research input, not a finding of this repository. It is the follow-up to
[report 1](2026-10-27-external-report-1.md), relayed by the maintainer on 2026-10-27 and
preserved verbatim in the block below. Its claims are checked against the primary sources
in [2026-10-27-research-synthesis.md](2026-10-27-research-synthesis.md).

The report's inline links reached us as placeholder characters (`⁠￼`), which are kept as
received. Its closing "Sources" lines list two URLs, and those came through intact.

## The report, verbatim

```text
Yes. Digging further into the sources turned up three concrete techniques worth passing to Claude, plus two smaller checks. The most promising findings concern the memory-reduction work, rather than another attempt to rescue two-step BWT traversal.

What I found

Priority	Technique	Relevant mibz2 work	Why it matters
1	Eliminate unnecessary intermediate buffers and copies	Fused decoding (#8)	Another decompressor achieved a measured bzip2 speedup by removing redundant buffering
2	Reduce Huffman lookup-table footprint	Huffman decoding (#8)	Smaller lookup structures can outperform larger ones by improving cache residency
3	Reuse or overwrite existing storage rather than allocate new scratch	Memory reduction (#8)	Could reduce mibz2’s approximately 7 MB of scratch per worker
4	Examine memory traffic during construction, not just the number of traversal steps	Two-step BWT traversal (#6)	Relevant to why cheaper theoretical traversal still loses end to end
5	Avoid unnecessary work in block discovery	Parallel scanner (#10)	Potentially useful at higher thread counts, but still lower priority

1. Removing buffers produced a real bzip2 speedup

The rapidgzip changelog⁠￼ records a specific improvement in version 0.14.0: avoiding unnecessary windows during bzip2 decompression reduced memory usage substantially and improved throughput from 344 MB/s to 388 MB/s, about 13% higher throughput.

That does not prove mibz2 has equivalent redundant buffers—rapidgzip’s windows serve its seeking architecture. But it supports a useful engineering question:

Can the output of one decoding stage become the input storage of the next, rather than allocating a separate array?

For mibz2, the place to examine is the Huffman → MTF → inverse-BWT construction pipeline.

2. Smaller Huffman tables can be faster

rapidgzip reports two notable changes in an earlier gzip/Deflate decoder:

* Restricting Huffman lookup-table construction to the maximum code length actually present in a block improved throughput from 140 to 195 MB/s.
* Reducing the number of cached lookup bits improved throughput from 165 to 255 MB/s.

Those are Deflate results, not bzip2 results, so the implementation cannot be copied directly.

But the lesson is relevant to mibz2’s recently merged two-symbol Huffman decoder: increasing lookup width isn’t automatically faster if the resulting table puts pressure on cache.

A cheap check would be to profile the current table sizes and cache footprint before changing them.

3. Avoiding copies deserves attention before SIMD

The rapidgzip changelog also records a speedup of 11–36% on selected Deflate workloads from replacing marker buffers in place, rather than allocating separate storage.

That suggests a useful ordering for mibz2’s memory work:

First eliminate unnecessary memory traffic; only then consider vectorising the traffic that remains.

This could make some of the proposed SIMD construction work unnecessary—or improve its effectiveness.

Two more findings worth keeping

Two-step BWT traversal (LF², issue #6): libsais documents the importance of temporary array buffers and auxiliary indices for inverse-BWT performance. The Rust wrapper explicitly recommends auxiliary indices for optimal inversion speed.

This suggests checking whether the reference implementation’s impressive traversal speed depends on preprocessing that mibz2 would have to perform for every block. It reinforces the need to measure construction and traversal together, rather than offering evidence of a new speedup.

Parallel block scanning (issue #10): rapidgzip’s newer changelog reports improving its block finder from 64 to 92 MB/s by simplifying it. That’s a gzip/Deflate block finder, not a bzip2 scanner, but the principle is useful: remove unnecessary decoding and validation work from the scanner before adding parallelism.

What I’d send Claude

I found some additional practical optimisation leads in rapidgzip’s engineering changelog and libsais’s inverse-BWT API.

The most relevant is for mibz2’s fused decoding/memory work (#8):

* rapidgzip v0.14.0 improved bzip2 throughput from 344 to 388 MB/s by eliminating unnecessary decompression windows. Its earlier Deflate optimisations also showed large gains from operating in-place and reducing Huffman lookup-table footprints.
* For mibz2, investigate whether any Huffman → MTF → BWT intermediate buffers can be eliminated, reused, or overwritten safely, and whether the current Huffman tables have avoidable cache costs.
* Prioritise reducing actual memory traffic before adding SIMD to move the same data faster. Measure full-decoder wall time and peak RSS, not only individual stage timings.

For LF² (#6), confirm that any performance comparisons with libsais include the preprocessing/auxiliary-index costs that mibz2 would incur per bzip2 block.

For the scanner (#10), rapidgzip has benefited from simplifying block finding, but this remains deferred unless our own profiling shows it matters.

Sources:
https://github.com/mxmlnkn/rapidgzip/blob/main/CHANGELOG.md
https://github.com/IlyaGrebnov/libsais

These are engineering leads, not guaranteed improvements. Please incorporate whichever checks fit the existing experiment plan without creating new issues or delaying current work.

My recommendation: send that once, then let Claude continue. The strongest actionable finding is to inspect memory ownership and buffer lifetimes before rewriting the front end. It has the potential to improve both throughput and memory use, which is a particularly attractive combination for mibz2’s multi-threaded decoder.
```
