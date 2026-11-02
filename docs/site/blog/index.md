---
description: "What mibz2's experiments are finding, and where they go next."
---

# Blog

What mibz2's experiments are finding, and where they go next. The evidence behind each
post is in the [journals](../background/journals.md) and the pull requests they link.

<div class="post-list">
<a class="post-card" href="2026-11-02-memory-level-parallelism.md"><img src="../assets/social/blog-2026-11-02-memory-level-parallelism.png" alt="" loading="lazy"><span class="post-card-text"><span class="post-card-date">2 November 2026 · Research synthesis</span><span class="post-card-title">Memory-level parallelism in mibz2: instructions, overlap and evidence</span><span class="post-card-dek">Interleaving pointer chases lets memory waits overlap. The speedup shows that this pays, but not what limits it: a 2022 exchange about Daniel Lemire's benchmark, read against mibz2's inverse-BWT walk.</span></span></a>
<a class="post-card" href="2026-11-01-inverse-bwt-memory-interference.md"><img src="../assets/social/blog-2026-11-01-inverse-bwt-memory-interference.png" alt="" loading="lazy"><span class="post-card-text"><span class="post-card-date">1 November 2026 · Research in progress</span><span class="post-card-title">Investigating inverse-BWT memory interference in mibz2</span><span class="post-card-dek">Four workers finish decompression sooner, but together spend 32% more processor time per byte. We're testing whether competition for shared memory is responsible.</span></span></a>
<a class="post-card" href="2026-11-01-block-dispatch.md"><img src="../assets/social/blog-2026-11-01-block-dispatch.png" alt="" loading="lazy"><span class="post-card-text"><span class="post-card-date">1 November 2026 · Investigation in progress</span><span class="post-card-title">Block dispatch in mibz2: mutexes, futexes and the scheduler</span><span class="post-card-dek">How mibz2's parallel decoder hands blocks to its workers, why some take up to a millisecond to be picked up, and how to tell lock contention from scheduling delay.</span></span></a>
<a class="post-card" href="2026-10-31-room-walk-overlap.md"><img src="../assets/social/blog-2026-10-31-room-walk-overlap.png" alt="" loading="lazy"><span class="post-card-text"><span class="post-card-date">31 October 2026 · Open question</span><span class="post-card-title">32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles</span><span class="post-card-dek">The decoder's 32-lane walk runs 8.6 times faster than one chain. That figure may not mean what it seems: the walk may already hide memory latency, and spend too many instructions on each byte.</span></span></a>
<a class="post-card" href="2026-10-29-compact-navigation-tables.md"><img src="../assets/social/blog-2026-10-29-compact-navigation-tables.png" alt="" loading="lazy"><span class="post-card-text"><span class="post-card-date">29 October 2026 · Work in progress</span><span class="post-card-title">Compact navigation tables for the inverse BWT: early findings</span><span class="post-card-dek">Most of the inverse BWT's navigation table follows simple patterns: what a census of real blocks found, and what comes next.</span></span></a>
</div>
