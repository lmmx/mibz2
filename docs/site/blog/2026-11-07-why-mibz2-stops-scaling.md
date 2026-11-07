---
title: "Why mibz2 stops scaling at eight threads, and why three teams help"
image: assets/social/blog-2026-11-07-why-mibz2-stops-scaling.png
description: "Each block decoded at once slows the others, and more threads per block barely help, so three teams of three beat four of two by 9%. Why blocks interfere is open."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Why mibz2 stops scaling at eight threads, and why three teams help

Each block decoded at once slows the others, and more threads per block barely help, so three
teams of three beat four of two by 9%. Why blocks interfere is open.
{: .post-dek }

7 November 2026 · 15 min read · [Research in progress](#8-where-do-things-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

On a ten-core desktop processor, mibz2, our bzip2 decoder written in Rust, got faster as we gave it
more threads, from one to eight. Past eight it stopped. Twelve, sixteen or twenty threads
decoded no faster than eight. A decoder that can't use the cores it is given leaves time on
the table for anyone decompressing large files, and the reason it stops says where the next
gain has to come from.

The reason is not the number of threads. It is how many pieces of the file the decoder works
on at the same time. A bzip2 file is a series of blocks that can be decoded independently, and
mibz2 decodes several at once, each with a small group of threads. We measured two things.
Every extra block in progress slows down every other block. And extra threads on the same
block barely speed it up. Together they mean that fewer blocks, each with more threads, beat
more blocks with fewer threads. Our latest release, 0.2.5, decodes three blocks at once with
three threads each, where 0.2.4 decoded four with two each. In two runs at ten threads, it was
8.8% and 8.9% faster, used 11–12% less processor time and 18–19% less peak memory.

What we have not measured is *why* the blocks slow each other down. The leading explanation is
that each block needs a large table, and the tables compete for the processor's shared cache.
The evidence for that comes from a simpler test program, not from the decoder itself, and a
careful review of an earlier draft of this analysis made us withdraw part of it. So this post
keeps three things apart: what was measured, what is inferred, and what we will test next.

[Section 1](#1-how-does-mibz2-decode-with-several-threads) explains how mibz2 splits a file
among its threads. [Section 2](#2-what-happens-when-we-change-the-number-of-blocks-and-the-threads-on-each)
gives the measurements behind the two findings, and sections
[3](#3-why-would-more-blocks-at-once-slow-each-block-down) and
[4](#4-why-dont-more-threads-on-one-block-help) look at each one in turn.
[Section 5](#5-what-did-three-teams-of-three-gain) is the release's own measurement,
[section 6](#6-what-did-the-review-change) what the review changed, and
[section 7](#7-what-will-we-try-next) the next experiments.

## 1. How does mibz2 decode with several threads?

bzip2 compresses a file in blocks of up to 900 kB each (at its default level, 9). Each block is
compressed on its own, with no reference to the others, so each can be decoded on its own too.
That is what makes a parallel decoder possible: different threads can work on different blocks
at the same time, and the decoder puts their outputs back in order at the end.

Decoding one block undoes bzip2's steps in reverse. One thread, which mibz2 calls the scanner,
reads the compressed file and undoes its outermost layer, the Huffman coding, for every block
([an earlier post](2026-11-05-huffman-bit-buffer.md) covers that stage). The rest of a block's
work happens on the threads that decode it, in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the
   block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which bzip2 used to reorder the
   bytes so that similar ones sit together. This puts them back in their original order.
3. **RLE1 and the CRC:** expanding the runs of repeated bytes that bzip2 shortened first of all,
   and checking the result against the block's checksum.

The inverse BWT is the most expensive of the three. It works in two phases. The **build** makes
a table with one 4-byte entry for each byte of the block, saying where the next byte of the
original text is to be found. For a full block that table, which mibz2 calls the T-vector, is
3.6 MB. The **walk** then follows the table from entry to entry, writing out a byte at each
step. Each step's address comes from the entry just read, so the steps form a chain through the
table, jumping to an unpredictable place each time. A jump that finds its entry in a nearby cache
is quick. One that has to go to main memory waits hundreds of cycles.
[Revisiting the space–time tradeoff in BWT inversion](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md)
shows the transform and its table worked through on a small example.

A chain of dependent jumps can't be split between threads directly, since each step needs the
one before. mibz2 gets around this by starting many chains at known points through the block,
so several threads can share one block's build and walk. We call the threads that share a
block a **team**. One of them, the **leader**, also does the block's MTF, RLE1 and CRC alone,
and puts the walk's pieces together at the end (the **assembly**). The others are its
**helpers**: they join in only for the build and the walk. A team decodes one block at a time,
with one table, and then takes the next.

So a thread count is really two numbers: how many teams, which is the number of blocks and
tables in progress at once, and how many threads each team has. We write a plan as teams ×
threads per team: 4 × 2 is four teams of two.

The machine these measurements come from is an Intel Core i9-10900K. It has 10 cores, each able
to run two threads at once (20 in all), and a 20 MiB last-level cache, the L3, which all ten
cores share. Until 0.2.5, mibz2's rule was teams of two, and as many teams as keep their tables
within about 0.7 of the L3. On the test machine that is four teams. Asked for 8, 10, 12, 16 or
20 threads, the decoder ran the same four teams of two, plus the scanner. That is why it
stopped at eight: past eight it declined the extra threads. Under that rule, 0.2.4 levelled off
at about 1.27–1.31 GB/s of output from 8 threads up.

<figure class="bf-fig">
<p class="bf-title">The two plans on the test machine's ten cores</p>
<svg viewBox="0 0 720 236" role="img" aria-label="Two layouts on a 10-core processor. Left, before: four teams of two threads, each team a leader L and a helper H, with four 3.6 MB tables, 14.4 MB, in the 20 MiB shared L3. Right, now: three teams of three, a leader and two helpers each, with three tables, 10.8 MB."><text x="0.0" y="18.0" class="bf-ts">Before (0.2.4): four teams of two</text><text x="0.0" y="36.0" class="bf-tm">8 decoding threads, 4 blocks at once</text><rect x="0" y="51" width="63" height="33" rx="6" class="bf-card-hi"/><rect x="3" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="16.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="33" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="46.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="69" y="51" width="63" height="33" rx="6" class="bf-card-hi"/><rect x="72" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="85.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="102" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="115.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="138" y="51" width="63" height="33" rx="6" class="bf-card-hi"/><rect x="141" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="154.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="171" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="184.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="207" y="51" width="63" height="33" rx="6" class="bf-card-hi"/><rect x="210" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="223.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="240" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="253.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="279" y="54" width="27" height="27" rx="4" class="bf-track"/><rect x="309" y="54" width="27" height="27" rx="4" class="bf-track"/><text x="0.0" y="106.0" class="bf-tm">10 cores: a team in each green outline; grey cores unused</text><text x="0.0" y="142.0" class="bf-tm">The shared L3 cache, 20 MiB</text><rect x="0" y="150" width="330" height="34" rx="4" class="bf-track"/><rect x="1.0" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="28.3" y="172.0" class="bf-tm" text-anchor="middle">table 1</text><rect x="57.7" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="85.0" y="172.0" class="bf-tm" text-anchor="middle">table 2</text><rect x="114.3" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="141.6" y="172.0" class="bf-tm" text-anchor="middle">table 3</text><rect x="171.0" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="198.3" y="172.0" class="bf-tm" text-anchor="middle">table 4</text><text x="0.0" y="204.0" class="bf-t-strong">4 tables × 3.6 MB = 14.4 MB</text><text x="0.0" y="222.0" class="bf-tm">plus each team's other buffers</text><text x="380.0" y="18.0" class="bf-ts">Now (0.2.5): three teams of three</text><text x="380.0" y="36.0" class="bf-tm">9 decoding threads, 3 blocks at once</text><rect x="380" y="51" width="93" height="33" rx="6" class="bf-card-hi"/><rect x="383" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="396.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="413" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="426.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="443" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="456.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="479" y="51" width="93" height="33" rx="6" class="bf-card-hi"/><rect x="482" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="495.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="512" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="525.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="542" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="555.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="578" y="51" width="93" height="33" rx="6" class="bf-card-hi"/><rect x="581" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="594.5" y="73.0" class="bf-t-strong" text-anchor="middle">L</text><rect x="611" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="624.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="641" y="54" width="27" height="27" rx="4" class="bf-pill"/><text x="654.5" y="73.0" class="bf-t-strong" text-anchor="middle">H</text><rect x="680" y="54" width="27" height="27" rx="4" class="bf-track"/><text x="380.0" y="106.0" class="bf-tm">10 cores: a team in each green outline; grey cores unused</text><text x="380.0" y="142.0" class="bf-tm">The shared L3 cache, 20 MiB</text><rect x="380" y="150" width="330" height="34" rx="4" class="bf-track"/><rect x="381.0" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="408.3" y="172.0" class="bf-tm" text-anchor="middle">table 1</text><rect x="437.7" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="465.0" y="172.0" class="bf-tm" text-anchor="middle">table 2</text><rect x="494.3" y="153" width="54.7" height="28" rx="3" class="bf-slot"/><text x="521.6" y="172.0" class="bf-tm" text-anchor="middle">table 3</text><text x="380.0" y="204.0" class="bf-t-strong">3 tables × 3.6 MB = 10.8 MB</text><text x="380.0" y="222.0" class="bf-tm">plus each team's other buffers</text></svg>
<figcaption>A sketch of the plans, not a measurement of what is in the cache. L is a team's leader, H a helper. Each team's table is 3.6 MB for a full block; each team also has other buffers, not drawn.</figcaption>
</figure>

## 2. What happens when we change the number of blocks, and the threads on each?

To see what four teams of two were missing, we ran a grid of plans that varied the two numbers
separately: 2 to 8 teams, of 2, 3 or 4 threads each. Each plan decoded the same sample of the
Wikidata dump, 11.85 MB of bzip2 that decodes to 209.7 MB, 20 times, in rotating order, with
threads placed by the operating system as users run it. The figure below groups the plans by
their number of teams. That is the most tables the plan can have in progress at once, not a
measured count of how many were in progress at each moment.

<figure class="bf-fig">
<p class="bf-title">Throughput by the number of teams, all thread counts per team together</p>
<svg viewBox="0 0 720 252" role="img" aria-label="Two charts against the number of teams, 2 to 8. Left: each team's rate falls from 558 to 583 MB/s at 2 teams to 461 to 466 at 3, about 320 at 4 and 110 at 8. Right: all teams together peak at 3 teams, 1,382 to 1,399 MB/s, against 1,276 to 1,300 at 4 and 882 at 8."><text x="0.0" y="16.0" class="bf-ts">Each team's rate, MB/s</text><path d="M40 210.0H300" class="bf-grid"/><text x="34.0" y="214.0" class="bf-tm" text-anchor="end">0</text><path d="M40 161.4H300" class="bf-grid"/><text x="34.0" y="165.4" class="bf-tm" text-anchor="end">200</text><path d="M40 112.9H300" class="bf-grid"/><text x="34.0" y="116.9" class="bf-tm" text-anchor="end">400</text><path d="M40 64.3H300" class="bf-grid"/><text x="34.0" y="68.3" class="bf-tm" text-anchor="end">600</text><path d="M40 210H300" class="bf-axis"/><text x="40.0" y="228.0" class="bf-tm" text-anchor="middle">2</text><text x="81.7" y="228.0" class="bf-tm" text-anchor="middle">3</text><text x="123.3" y="228.0" class="bf-tm" text-anchor="middle">4</text><text x="165.0" y="228.0" class="bf-tm" text-anchor="middle">5</text><text x="206.7" y="228.0" class="bf-tm" text-anchor="middle">6</text><text x="290.0" y="228.0" class="bf-tm" text-anchor="middle">8</text><text x="170.0" y="246.0" class="bf-tm" text-anchor="middle">teams (blocks decoded at once)</text><path d="M40.0 71.5 L81.7 97.4 L123.3 131.8 L165.0 154.3 L206.7 167.5 L290.0 183.3" class="bf-arrow"/><path d="M40.0 74.5V68.4" class="bf-cut"/><circle cx="40.0" cy="71.5" r="4.5" class="bf-dot-hi"/><path d="M81.7 98.0V96.8" class="bf-cut"/><circle cx="81.7" cy="97.4" r="4.5" class="bf-dot-hi"/><path d="M123.3 132.5V131.1" class="bf-cut"/><circle cx="123.3" cy="131.8" r="4.5" class="bf-dot-hi"/><path d="M165.0 154.4V154.1" class="bf-cut"/><circle cx="165.0" cy="154.3" r="4.5" class="bf-dot-hi"/><circle cx="206.7" cy="167.5" r="4.5" class="bf-dot-hi"/><circle cx="290.0" cy="183.3" r="4.5" class="bf-dot-hi"/><text x="50.0" y="65.6" class="bf-tm">558–583</text><text x="89.7" y="89.6" class="bf-tm">461–466</text><text x="131.3" y="123.8" class="bf-tm">319–325</text><text x="290.0" y="173.3" class="bf-tm" text-anchor="middle">110</text><text x="390.0" y="16.0" class="bf-ts">All teams together, MB/s</text><path d="M430 210.0H690" class="bf-grid"/><text x="424.0" y="214.0" class="bf-tm" text-anchor="end">0</text><path d="M430 167.5H690" class="bf-grid"/><text x="424.0" y="171.5" class="bf-tm" text-anchor="end">400</text><path d="M430 125.0H690" class="bf-grid"/><text x="424.0" y="129.0" class="bf-tm" text-anchor="end">800</text><path d="M430 82.5H690" class="bf-grid"/><text x="424.0" y="86.5" class="bf-tm" text-anchor="end">1,200</text><path d="M430 40.0H690" class="bf-grid"/><text x="424.0" y="44.0" class="bf-tm" text-anchor="end">1,600</text><path d="M430 210H690" class="bf-axis"/><text x="430.0" y="228.0" class="bf-tm" text-anchor="middle">2</text><text x="471.7" y="228.0" class="bf-tm" text-anchor="middle">3</text><text x="513.3" y="228.0" class="bf-tm" text-anchor="middle">4</text><text x="555.0" y="228.0" class="bf-tm" text-anchor="middle">5</text><text x="596.7" y="228.0" class="bf-tm" text-anchor="middle">6</text><text x="680.0" y="228.0" class="bf-tm" text-anchor="middle">8</text><text x="560.0" y="246.0" class="bf-tm" text-anchor="middle">teams (blocks decoded at once)</text><path d="M430.0 88.8 L471.7 62.3 L513.3 73.2 L555.0 87.9 L596.7 98.3 L680.0 116.3" class="bf-arrow"/><path d="M430.0 91.4V86.1" class="bf-cut"/><circle cx="430.0" cy="88.8" r="4.5" class="bf-dot-hi"/><path d="M471.7 63.2V61.4" class="bf-cut"/><circle cx="471.7" cy="62.3" r="7" class="bf-dot-hi"/><path d="M513.3 74.4V71.9" class="bf-cut"/><circle cx="513.3" cy="73.2" r="4.5" class="bf-dot-hi"/><path d="M555.0 88.1V87.6" class="bf-cut"/><circle cx="555.0" cy="87.9" r="4.5" class="bf-dot-hi"/><circle cx="596.7" cy="98.3" r="4.5" class="bf-dot-hi"/><circle cx="680.0" cy="116.3" r="4.5" class="bf-dot-hi"/><text x="471.7" y="50.3" class="bf-t-strong" text-anchor="middle">1,382–1,399</text><text x="513.3" y="99.2" class="bf-tm" text-anchor="middle">1,276–1,300</text><text x="680.0" y="106.3" class="bf-tm" text-anchor="middle">882</text></svg>
<figcaption>Measured, the test machine, 20 rounds a plan (the <a href="https://github.com/lmmx/mibz2/blob/d95ca1ea/docs/journal/2026-11-06-team-size.md">team-size grid</a>). A vertical bar spans the plans with that many teams but different threads per team. The rate is megabytes of decoded output a second.</figcaption>
</figure>

Two things stand out. **Each team gets slower as teams are added**, and steeply: from
558–583 MB/s each with two teams, to 461–466 with three, about 320 with four and 110 with
eight. Adding a fourth team adds a fourth block in progress, but makes every block slow enough
that all four together are slower than three. Total throughput peaks at three teams,
1,382–1,399 MB/s.

And **at a fixed number of teams, the threads per team hardly matter.** With four teams, each
team decoded 319, 325 and 320 MB/s with two, three and four threads. The extra threads cost
processor time without buying speed. The table below gives the four-team plans against the
old default, with the best three-team plans for comparison. "Against 4 × 2" is the ratio of
throughputs, paired round by round, with its 95% interval across the 20 rounds; "CPU" is
total processor time over 4 × 2's.

| plan | threads | MB/s | against 4 × 2 | CPU | peak memory, MB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 4 × 2 (the old default) | 8 | 1,276 | 1 | 1 | 82 |
| 4 × 3 | 12 | 1,300 | 1.017 (1.008–1.026) | 1.26 | 82 |
| 4 × 4 | 16 | 1,279 | 1.003 (0.993–1.014) | 1.52 | 82 |
| 3 × 3 | 9 | 1,382 | 1.075 (1.058–1.093) | 0.89 | 67 |
| 3 × 4 | 12 | 1,399 | 1.097 (1.083–1.111) | 1.04 | 68 |

The four-team rows were the experiment's registered tests: their pass rule, a gain of at
least 3%, was fixed before any run. Neither passed. The
three-team rows were exploratory: picked out after the fact, so they proved nothing on their
own. They were then confirmed in runs registered for them, and the release's plan in two more
([section 5](#5-what-did-three-teams-of-three-gain)). Peak memory, meanwhile, rose by about
15 MB with each team: 52 MB at two teams, 67 at three, 82 at four and 138 at eight.

That leaves two questions. Why does each extra block slow the others? And why doesn't a block
go faster with more threads?

## 3. Why would more blocks at once slow each block down?

**This part is inferred, not measured in the decoder.** The leading explanation is about the
shared cache. Each team's walk jumps unpredictably through its own 3.6 MB table, and is fast
only while the entries it needs are in the L3. All teams share that one cache, so each extra
table takes room the others were using. As an intuition: several people each searching their
own large reference book, all spread out on one desk. With two books, most pages stay open.
With eight, pages keep being put away to make room, and fetching them back is slow.

The evidence comes from a test program that runs only the inverse BWT, the build and the walk,
with each block's thread fixed to a core of its own, and no teams. In October we ran it on the test machine with
hardware counters, which count events inside the processor. One of them is the **L3 hit
share**: of the memory requests that miss the cores' own small caches, the share the L3
answers rather than main memory. Plotted against the table bytes in use at once, it stays near
1 while the tables are small, then falls once they approach the cache's size.

<figure class="bf-fig">
<p class="bf-title">L3 hit share against the table bytes in use at once (inversion only)</p>
<svg viewBox="0 0 720 280" role="img" aria-label="L3 hit share against table bytes in use at once, measured in an inversion-only harness. It stays near 0.94 to 0.99 up to 14.4 MB, then falls to 0.794 at 21.6 MB, 0.640 at 28.8 MB and 0.539 at 36 MB. The L3 is 20 MiB. Level-3 points, with smaller tables, lie on the same curve."><path d="M60 230.0H680" class="bf-grid"/><text x="52.0" y="234.0" class="bf-tm" text-anchor="end">0.4</text><path d="M60 163.3H680" class="bf-grid"/><text x="52.0" y="167.3" class="bf-tm" text-anchor="end">0.6</text><path d="M60 96.7H680" class="bf-grid"/><text x="52.0" y="100.7" class="bf-tm" text-anchor="end">0.8</text><path d="M60 30.0H680" class="bf-grid"/><text x="52.0" y="34.0" class="bf-tm" text-anchor="end">1.0</text><path d="M60 230H680" class="bf-axis"/><text x="60.0" y="248.0" class="bf-tm" text-anchor="middle">0</text><text x="215.0" y="248.0" class="bf-tm" text-anchor="middle">10</text><text x="370.0" y="248.0" class="bf-tm" text-anchor="middle">20</text><text x="525.0" y="248.0" class="bf-tm" text-anchor="middle">30</text><text x="680.0" y="248.0" class="bf-tm" text-anchor="middle">40</text><text x="370.0" y="268.0" class="bf-tm" text-anchor="middle">table bytes in use at once, MB</text><text x="10.0" y="14.0" class="bf-tm">share of requests answered by the L3</text><path d="M385.0 30V230" class="bf-rule"/><text x="391.0" y="44.0" class="bf-tm">L3 size, 20 MiB</text><path d="M171.6 37.0 L283.2 53.7 L394.8 98.7 L506.4 150.0 L618.0 183.7" class="bf-cut"/><circle cx="171.6" cy="37.0" r="5" class="bf-dot-hi"/><circle cx="283.2" cy="53.7" r="5" class="bf-dot-hi"/><circle cx="394.8" cy="98.7" r="5" class="bf-dot-hi"/><circle cx="506.4" cy="150.0" r="5" class="bf-dot-hi"/><circle cx="618.0" cy="183.7" r="5" class="bf-dot-hi"/><rect x="92.7" y="29.8" width="9" height="9" class="bf-steel"/><rect x="129.9" y="31.2" width="9" height="9" class="bf-steel"/><rect x="167.1" y="33.8" width="9" height="9" class="bf-steel"/><rect x="204.3" y="38.5" width="9" height="9" class="bf-steel"/><rect x="241.5" y="45.5" width="9" height="9" class="bf-steel"/><text x="293.2" y="57.7" class="bf-tm">4 tables: 0.929</text><text x="404.8" y="102.7" class="bf-tm">6 tables: 0.794</text><text x="516.4" y="154.0" class="bf-tm">8 tables: 0.640</text><text x="608.0" y="201.7" class="bf-tm" text-anchor="end">10 tables: 0.539</text><circle cx="80" cy="178" r="5" class="bf-dot-hi"/><text x="92.0" y="182.0" class="bf-tm">blocks of a level-9 file, 3.6 MB a table</text><rect x="75.5" y="193.5" width="9" height="9" class="bf-steel"/><text x="92.0" y="202.0" class="bf-tm">blocks of a level-3 file, 1.2 MB a table</text></svg>
<figcaption>Measured, the test machine, in a program that runs only the build and walk, one block a core (<a href="https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-libsais-knee.md">footprint journal</a>). bzip2's compression level sets its block size: at level 3 blocks are 300 kB, so their tables are a third the size of level 9's. Not a measurement of the decoder.</figcaption>
</figure>

Four other observations from that program point the same way:

- **Requests wait longer.** A request that misses the cores' own caches waited on average 67
  cycles with four tables, 137 with six and 361 with eight.
- **Main memory saturates.** From six tables on, traffic to main memory levels off at 28–29 GB/s.
- **A control isolates the table's size.** A variant that changes the walk's code but keeps a
  4-byte table got the same hit share as the standard one at six tables (0.814 against 0.793).
- **A smaller table moves the drop.** A 3-byte table (2.7 MB a block) moved the program's best
  number of blocks from four to six, where it ran 1.100 (1.093–1.108) times as fast as the
  standard table's best.

That is good evidence that the size of the tables matters, *in that program*. It doesn't settle
the question for the decoder, for three reasons.

First, the program isn't the decoder. It has no teams, no MTF, RLE1 or CRC, no scanner, and
none of the decoder's other buffers. Second, table bytes don't explain everything even there.
With the smaller level-3 blocks, ten tables kept a hit share of 0.940 and yet ran slower than eight, mostly
in the build, with requests waiting longer at nearly the same hit share. So a limit on how many
requests the cores can have outstanding, or on the number of cores loading the cache at once,
is not ruled out. Third, the decoder's own counters were never read: the operating system's
settings on the test machine didn't allow it during the team-size run.

There is also a simpler puzzle. Four full tables are 14.4 MB, below the cache's 20 MiB. Each team
has other buffers besides its table, such as space for the walk's pieces and for the block's
output: several megabytes of reserved capacity each. But capacity is not what is being read at
any moment, and how much of those buffers sits in the cache at once is not measured. An earlier
draft of this analysis added those capacities up to a 20 MiB "hot set" that filled the cache. The
review showed that a sum of capacities and peaks can't establish that, and the claim was
withdrawn ([section 6](#6-what-did-the-review-change)).

Two other explanations also fit the measurements. The teams' waits for each other could be
what makes blocks slow, rather than memory. And three teams of three use nine threads on ten
cores, so one core is free, which changes how the operating system schedules everything else.
No measurement of the decoder yet tells these apart.

## 4. Why don't more threads on one block help?

**Measured:** with four teams, a third thread per team bought 1.7% (0.8–2.6%) for 26% more
processor time, and a fourth bought nothing detectable, 0.3% (−0.7% to 1.4%), for 52% more. A
trace of each block's decode, which records when each block starts and finishes, shows the
same thing from inside: with four teams, a block took on average 1.394, 1.380 and 1.403 ms with
two, three and four threads.

Part of the reason is plain arithmetic: not all of a block's work is shared. Only the build and
the walk are, and only inside the call that does the inverse BWT. The leader does the MTF before
that call, and RLE1 and the CRC after it, alone.

<figure class="bf-fig">
<p class="bf-title">One block's work in a team</p>
<svg viewBox="0 0 720 346" role="img" aria-label="A block's six stages in a team, top to bottom. MTF, by the leader alone, outside the inversion call. Inside the call: setup by the leader, the build and the walk by the whole team, and assembly by the leader. Then RLE1 and the CRC check by the leader alone, outside the call. The call takes about 67 to 71 percent of a block's decode, the rest 29 to 33 percent."><rect x="0" y="54" width="720" height="202" rx="10" class="bf-band"/><rect x="16" y="10" width="330" height="40" rx="6" class="bf-pill"/><text x="32.0" y="36.0" class="bf-ts">MTF</text><text x="362.0" y="36.0" class="bf-t">the leader alone</text><rect x="16" y="60" width="330" height="40" rx="6" class="bf-pill"/><text x="32.0" y="86.0" class="bf-ts">Setup</text><text x="362.0" y="86.0" class="bf-t">the leader alone</text><rect x="16" y="110" width="330" height="40" rx="6" class="bf-card-hi"/><text x="32.0" y="136.0" class="bf-ts">Build the table</text><text x="362.0" y="136.0" class="bf-t">the whole team</text><rect x="16" y="160" width="330" height="40" rx="6" class="bf-card-hi"/><text x="32.0" y="186.0" class="bf-ts">Walk the table</text><text x="362.0" y="186.0" class="bf-t">the whole team</text><rect x="16" y="210" width="330" height="40" rx="6" class="bf-pill"/><text x="32.0" y="236.0" class="bf-ts">Assemble the pieces</text><text x="362.0" y="236.0" class="bf-t">the leader alone</text><rect x="16" y="260" width="330" height="40" rx="6" class="bf-pill"/><text x="32.0" y="286.0" class="bf-ts">RLE1 and the CRC</text><text x="362.0" y="286.0" class="bf-t">the leader alone</text><text x="704.0" y="36.0" class="bf-t" text-anchor="end">outside the call</text><text x="704.0" y="286.0" class="bf-t" text-anchor="end">outside the call</text><text x="704.0" y="140.0" class="bf-t-strong" text-anchor="end">inside the</text><text x="704.0" y="158.0" class="bf-t-strong" text-anchor="end">inversion call:</text><text x="704.0" y="176.0" class="bf-t-strong" text-anchor="end">67–71% of a block</text><text x="16.0" y="332.0" class="bf-t">Outside the call: 29–33% of a block's decode, in traces of four teams.</text></svg>
<figcaption>The stages and who runs them, from the decoder's code. The shares are measured, in traces of four teams on the test machine: medians of six runs each at two, three and four threads a team. Inside the call the team also waits for its slowest member; those waits are not timed separately.</figcaption>
</figure>

The traces give the time inside and outside the inversion call for each block:

| plan | a block's decode, ms | inside the inversion call, ms | outside it, ms | outside share |
| --- | ---: | ---: | ---: | ---: |
| 4 × 2 | 1.394 | 0.993 | 0.400 | 28.75% |
| 4 × 3 | 1.380 | 0.935 | 0.441 | 32.14% |
| 4 × 4 | 1.404 | 0.946 | 0.462 | 32.64% |

These are averages over the sample's 431 blocks, some of them small, so they are not the time
for one full 900 kB block. On one of the cloud machines that benchmark every change to mibz2 (CI), with one thread, the
stages cost, per output
byte: 0.41 ns for MTF, 0.04 for assembly, 0.05 for RLE1 and 0.07 for the CRC, which the leader
does alone, against 0.48 for the build and 1.09 for the walk, which a team shares.

So about 29–33% of a block's time is outside the shared part, and more threads can't shorten
it. But that share is not the whole of the work that can't be shared. The inversion call also
includes work the leader does alone (setting up, and assembling the pieces), and time the team
spends waiting for its slowest member. The time outside the call also includes scheduling and the
tracing itself. The traces can't separate these, so the true unshared share isn't known.

**What the measurements don't separate** is why the shared part itself doesn't speed up with a
third or fourth thread. The candidates, each visible in the code, are:

- the cost of waking helpers and of waiting for the last one to finish;
- the setup and assembly the leader does alone, inside the call;
- the build's checks that each thread writes only its own part of the table;
- the number of pieces a block is split into, which grows with the threads per team;
- two threads sharing one core, once there are more threads than cores;
- memory, as in section 3.

Waiting isn't free either: a helper with nothing to do checks for work up to 2,000 times before
it sleeps, which costs processor time.

One more measurement bears on core sharing. With four teams pinned to particular processors,
giving the threads one core each ("spread") against pairing them on cores ("packed") made the
biggest difference at two threads a team: packed was 23% slower (1,055 against 1,369 MB/s
in the median round). With three threads a team it was 8% slower, and with four, 1%. But the
packed arm also had fewer cores to use (4 against 8 for eight threads), so this compares how
many cores were available, not which threads shared them.

## 5. What did three teams of three gain?

The grid's three-team plans were chosen after the fact, so before changing the decoder we
registered a test of the change itself, with its pass rule fixed before any run: 0.2.5's plan
against 0.2.4's, at 8 to 20 threads, 20 rounds each, and a second run to confirm it. Both runs
passed. At ten threads:

| run | speed against 0.2.4 | processor time | peak memory |
| --- | ---: | ---: | ---: |
| first | 1.089 (1.075–1.103) | 0.88 | 0.82 |
| confirmation | 1.088 (1.076–1.101) | 0.89 | 0.81 |

The intervals are 95% intervals across the 20 paired rounds of one run. Neither run detected
harm at 11, 12, 16 or 20 threads, the registered check there. A separate comparison of the two
releases found 0.2.4 taking 11.9% longer than 0.2.5 at ten threads, and 8.1–8.7% longer at 12
to 20.

At ten threads or more on a machine of this shape (ten cores, a 20 MiB L3), 0.2.5 runs three
teams of three. A request for twenty threads still decodes on nine. The result covers this
processor and this workload. It shows that three teams are faster. It doesn't show why, which
is section 3's open question.

## 6. What did the review change?

The first version of this analysis was reviewed adversarially by another member of the project,
who recomputed its numbers from the archived traces. All seven objections were accepted, and
the version this post is based on takes them in. Four changes matter for the argument.

The first draft claimed that the decoder's buffers formed a 20 MiB working set that filled the
cache. That was a sum of reserved capacities and peaks, not a measurement of what is read at
the same time, and it was withdrawn. The first draft also read the flat block times in
section 4 as memory saturation. The review pointed out that they can't separate memory from
waiting, unshared work or core sharing, and that claim was withdrawn too. The outside share in
section 4 was also recomputed: the first draft divided by the wrong total, and called the whole
inversion call "build and walk", when it includes setup, assembly and waits. And the first
draft's 1.10–1.30× estimate for the next experiment was replaced by the model in section 7,
which shows that no such floor follows from the measurements.

## 7. What will we try next?

**First: let the leader work on other blocks while its helpers build and walk.** Today a
team's helpers wait while the leader runs MTF before a block and RLE1 and the CRC after it. In
the proposed pipeline, the leader runs the next block's MTF and the previous block's RLE1 and
CRC while the other threads build and walk the current one. It needs one more buffer for the
next block's bytes, about 1 MB a team. A prototype is being built.

<figure class="bf-fig">
<p class="bf-title">A team's timeline, today and as proposed</p>
<svg viewBox="0 0 720 260" role="img" aria-label="Two timelines. Today, the leader runs MTF, then joins the helpers for the build and walk, then finishes the block alone, while the helpers wait. Proposed, the helpers build and walk one block while the leader finishes the previous block and runs MTF for the next. Conceptual timing."><text x="0.0" y="18.0" class="bf-ts">Today: the helpers wait while the leader works alone</text><text x="0.0" y="52.0" class="bf-t">leader</text><text x="0.0" y="90.0" class="bf-t">helpers</text><rect x="80" y="30" width="70" height="30" rx="4" class="bf-pill"/><text x="115.0" y="50.0" class="bf-t" text-anchor="middle">MTF A</text><rect x="152" y="30" width="150" height="30" rx="4" class="bf-card-hi"/><text x="227.0" y="50.0" class="bf-t" text-anchor="middle">build, walk A</text><rect x="152" y="68" width="150" height="30" rx="4" class="bf-card-hi"/><text x="227.0" y="88.0" class="bf-t" text-anchor="middle">build, walk A</text><rect x="304" y="30" width="92" height="30" rx="4" class="bf-pill"/><text x="350.0" y="50.0" class="bf-t" text-anchor="middle">finish A</text><rect x="80" y="68" width="70" height="30" rx="4" class="bf-track"/><rect x="304" y="68" width="92" height="30" rx="4" class="bf-track"/><rect x="400" y="30" width="70" height="30" rx="4" class="bf-pill"/><text x="435.0" y="50.0" class="bf-t" text-anchor="middle">MTF B</text><rect x="472" y="30" width="150" height="30" rx="4" class="bf-card-hi"/><text x="547.0" y="50.0" class="bf-t" text-anchor="middle">build, walk B</text><rect x="472" y="68" width="150" height="30" rx="4" class="bf-card-hi"/><text x="547.0" y="88.0" class="bf-t" text-anchor="middle">build, walk B</text><rect x="624" y="30" width="92" height="30" rx="4" class="bf-pill"/><text x="670.0" y="50.0" class="bf-t" text-anchor="middle">finish B</text><rect x="400" y="68" width="70" height="30" rx="4" class="bf-track"/><rect x="624" y="68" width="92" height="30" rx="4" class="bf-track"/><text x="0.0" y="140.0" class="bf-ts">Proposed: the leader finishes one block and starts the next</text><text x="0.0" y="174.0" class="bf-t">leader</text><text x="0.0" y="212.0" class="bf-t">helpers</text><rect x="110" y="152" width="70" height="30" rx="4" class="bf-pill"/><text x="145.0" y="172.0" class="bf-t" text-anchor="middle">MTF B</text><rect x="80" y="190" width="200" height="30" rx="4" class="bf-card-hi"/><text x="180.0" y="210.0" class="bf-t" text-anchor="middle">build, walk A</text><rect x="286" y="152" width="104" height="30" rx="4" class="bf-pill"/><text x="338.0" y="172.0" class="bf-t" text-anchor="middle">finish A</text><rect x="392" y="152" width="70" height="30" rx="4" class="bf-pill"/><text x="427.0" y="172.0" class="bf-t" text-anchor="middle">MTF C</text><rect x="286" y="190" width="200" height="30" rx="4" class="bf-card-hi"/><text x="386.0" y="210.0" class="bf-t" text-anchor="middle">build, walk B</text><rect x="492" y="152" width="104" height="30" rx="4" class="bf-pill"/><text x="544.0" y="172.0" class="bf-t" text-anchor="middle">finish B</text><rect x="598" y="152" width="70" height="30" rx="4" class="bf-pill"/><text x="633.0" y="172.0" class="bf-t" text-anchor="middle">MTF D</text><rect x="492" y="190" width="200" height="30" rx="4" class="bf-card-hi"/><text x="592.0" y="210.0" class="bf-t" text-anchor="middle">build, walk C</text><text x="80.0" y="246.0" class="bf-t">“finish” is the assembly, RLE1 and the CRC. Proposed, the helpers are one thread fewer.</text></svg>
<figcaption>The proposal, not a measurement: bar lengths are illustrative. Whether it pays depends on how much slower the build and walk run with one thread fewer, and on what the hand-offs cost.</figcaption>
</figure>

Whether it pays is not obvious, because the shared part loses a thread: in a team of three,
the build and walk would run on two. A simple model, from the review, puts numbers on that.
Let $S$ be the leader's work outside the shared part, $P_0$ the shared part today, $\alpha$ how
much slower the shared part runs with a thread fewer and more blocks in progress, and $H$ the
cost of hand-offs that can't overlap. Then the time a team spends per block, today and
pipelined, and the speed-up, are

$$C_0 = S + P_0, \qquad C_1 = \max(S,\ \alpha P_0) + H,$$

$$\text{speed-up} = C_0 / C_1.$$

For three teams of three, the grid's throughput gives $C_0$ = 1.056 ms a block per team: three
teams, 209.7 MB in 431 blocks, at 1,382 MB/s. That figure includes start-up and the end of the
run, so it is an approximation, not a measured service time. Taking $S$ = 0.40 ms, the time
outside the inversion call in section 4, leaves $P_0$ = 0.656 ms. The model's speed-up then
depends almost entirely on $\alpha$:

<figure class="bf-fig">
<p class="bf-title">Modelled speed-up of the pipeline, against α</p>
<svg viewBox="0 0 720 262" role="img" aria-label="Modelled speed of the pipelined team against today's, as alpha rises from 1.0 to 1.8. With no hand-off cost: 1.61 times at alpha 1.0, 1.34 at 1.2, 1.15 at 1.4, and break-even at alpha 1.61. With 0.05 ms of hand-off a block: 1.50, 1.26, 1.09, break-even at 1.53."><path d="M60 214.0H660" class="bf-grid"/><text x="52.0" y="218.0" class="bf-tm" text-anchor="end">0.8×</text><path d="M60 171.8H660" class="bf-rule"/><text x="52.0" y="175.8" class="bf-tm" text-anchor="end">1.0×</text><path d="M60 129.6H660" class="bf-grid"/><text x="52.0" y="133.6" class="bf-tm" text-anchor="end">1.2×</text><path d="M60 87.3H660" class="bf-grid"/><text x="52.0" y="91.3" class="bf-tm" text-anchor="end">1.4×</text><path d="M60 45.1H660" class="bf-grid"/><text x="52.0" y="49.1" class="bf-tm" text-anchor="end">1.6×</text><path d="M60 214H660" class="bf-axis"/><text x="60.0" y="232.0" class="bf-tm" text-anchor="middle">1.0</text><text x="210.0" y="232.0" class="bf-tm" text-anchor="middle">1.2</text><text x="360.0" y="232.0" class="bf-tm" text-anchor="middle">1.4</text><text x="510.0" y="232.0" class="bf-tm" text-anchor="middle">1.6</text><text x="660.0" y="232.0" class="bf-tm" text-anchor="middle">1.8</text><text x="360.0" y="252.0" class="bf-tm" text-anchor="middle">α: how much slower the build and walk run, pipelined</text><path d="M60.0 43.1 L75.0 49.8 L90.0 56.2 L105.0 62.3 L120.0 68.3 L135.0 74.0 L150.0 79.5 L165.0 84.8 L180.0 90.0 L195.0 94.9 L210.0 99.7 L225.0 104.4 L240.0 108.9 L255.0 113.2 L270.0 117.4 L285.0 121.5 L300.0 125.5 L315.0 129.3 L330.0 133.0 L345.0 136.7 L360.0 140.2 L375.0 143.6 L390.0 146.9 L405.0 150.2 L420.0 153.3 L435.0 156.4 L450.0 159.3 L465.0 162.2 L480.0 165.1 L495.0 167.8 L510.0 170.5 L525.0 173.1 L540.0 175.7 L555.0 178.2 L570.0 180.6 L585.0 183.0 L600.0 185.3 L615.0 187.6 L630.0 189.8 L645.0 192.0 L660.0 194.1" class="bf-cut"/><circle cx="60.0" cy="43.1" r="4" class="bf-dot-hi"/><circle cx="210.0" cy="99.7" r="4" class="bf-dot-hi"/><circle cx="360.0" cy="140.2" r="4" class="bf-dot-hi"/><circle cx="510.0" cy="170.5" r="4" class="bf-dot-hi"/><path d="M60.0 67.1 L75.0 72.9 L90.0 78.5 L105.0 83.8 L120.0 89.0 L135.0 94.0 L150.0 98.8 L165.0 103.5 L180.0 108.0 L195.0 112.4 L210.0 116.6 L225.0 120.7 L240.0 124.7 L255.0 128.6 L270.0 132.3 L285.0 136.0 L300.0 139.5 L315.0 143.0 L330.0 146.3 L345.0 149.5 L360.0 152.7 L375.0 155.8 L390.0 158.8 L405.0 161.7 L420.0 164.5 L435.0 167.3 L450.0 170.0 L465.0 172.6 L480.0 175.2 L495.0 177.7 L510.0 180.2 L525.0 182.6 L540.0 184.9 L555.0 187.2 L570.0 189.4 L585.0 191.6 L600.0 193.7 L615.0 195.8 L630.0 197.8 L645.0 199.8 L660.0 201.8" class="bf-arrow"/><circle cx="60.0" cy="67.1" r="4" class="bf-dot"/><circle cx="210.0" cy="116.6" r="4" class="bf-dot"/><circle cx="360.0" cy="152.7" r="4" class="bf-dot"/><circle cx="510.0" cy="180.2" r="4" class="bf-dot"/><circle cx="517.2" cy="171.8" r="4" class="bf-dot-ready"/><text x="523.2" y="161.8" class="bf-tm">break-even at 1.61</text><circle cx="460.0" cy="171.8" r="4" class="bf-dot-ready"/><text x="454.0" y="193.8" class="bf-tm" text-anchor="end">break-even at 1.53</text><text x="68.0" y="47.1" class="bf-tm">1.61×</text><text x="68.0" y="81.1" class="bf-tm">1.50×</text><text x="360.0" y="130.2" class="bf-tm" text-anchor="middle">1.15×</text><text x="360.0" y="170.7" class="bf-tm" text-anchor="middle">1.09×</text><path d="M435.0 38h24" class="bf-cut"/><text x="465.0" y="42.0" class="bf-tm">no hand-off cost</text><path d="M435.0 56h24" class="bf-arrow"/><text x="465.0" y="60.0" class="bf-tm">0.05 ms a block</text></svg>
<figcaption>Modelled, not measured. The break-even points are where the curves cross 1, at α of about 1.61 and 1.53. The measurements don't bound α.</figcaption>
</figure>

If the shared part ran no slower, the pipeline would be 1.61× as fast, or 1.50× with 0.05 ms
of hand-off cost per block. If it ran 40% slower, 1.15× or 1.09×. At 1.6× slower the gain is
gone. If the shared part slowed to what it takes under four teams of two, 0.94–0.99 ms, the
gain would be 1.07–1.12×, or 1.02–1.07× with the hand-off cost. Nothing measured so far says
which $\alpha$ to expect, so no gain and a loss are both possible outcomes, and will be
registered as such.

The test will compare like with like. To isolate the pipeline, pipelined against not
pipelined at the same plan and thread count: three teams of four against three teams of four,
and three of three against three of three. Separately, pipelined three of four against today's
default at ten threads, which is the comparison a user would care about. Each will report
processor time and peak memory alongside the elapsed time. The prototype must also record when each
thread starts and finishes each stage of each block, so that the trace shows blocks really
overlapping. Without that, a result says nothing about the mechanism.

**Second: placement, as a control.** Pinned against unpinned threads, recording which thread
ran on which processor, for today's plan and for three teams of four. It makes no prediction
of its own. It is needed to read the pipeline's result, since that plan uses more threads than
there are cores.

**Third: a smaller table under teams.** The inversion-only program's 3-byte table moved the
fall in hit share to more blocks, so the table's size is a real factor. But that table's 1.10× gain was against the same
program's standard table, not against the decoder: against the decoder's own team code (four
teams of two), in the same tests, it ran at 0.687 (0.680–0.695) of the speed, on six cores
against eight. A different compact table, which packs
30 entries into each 64-byte record, ran 4.8% and 7.7% slower than the standard one at four
threads on CI. And no smaller table has been measured under teams. So it comes after the first
two.

**Last: shrinking the other buffers.** There is no measurement of how much cache they take, so
nothing yet says this would help.

## 8. Where do things stand?

The chain of the argument runs like this. A thread count is really two numbers, blocks in
progress and threads per block. Each extra block slows every other block, enough that three
blocks beat four. Extra threads on one block barely help, partly because about a third of a
block's time is work its leader does alone. So three teams of three beat four teams of two, as
two registered runs confirmed. Why extra blocks hurt is not settled: competition for the
shared cache is the leading explanation, with good evidence from a simpler program and none
yet from the decoder.

The pipeline prototype, its registration and the placement control are in progress. Nothing in
this post changes the decoder.

The general lesson: more threads help only if the work they add runs alongside the slowest
step. Here the slowest step has two parts, the blocks competing with each other, and each
block's work that one thread does alone.

---

**Research records:**

- [The analysis of the scaling limit, revised after review](https://github.com/lmmx/mibz2/blob/d95ca1ea/docs/journal/2026-11-07-knee-mechanism.md),
  and [the review with its recomputation](https://github.com/lmmx/mibz2/blob/570a89ee/state/reports/knee-mechanism-6699480548/report.md)
- [The team-size grid and traces](https://github.com/lmmx/mibz2/blob/d95ca1ea/docs/journal/2026-11-06-team-size.md)
- [The three-team policy: registration and both runs](https://github.com/lmmx/mibz2/blob/d95ca1ea/docs/journal/2026-11-06-three-team-policy.md)
- [The release comparison, 0.2.4 against 0.2.5](https://github.com/lmmx/mibz2/tree/d95ca1ea/bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.5)
- [The smaller-table test with counters](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-compact-psi.md)
  and [the footprint journal](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-libsais-knee.md)
- [The compact table under the staged walk](https://github.com/lmmx/mibz2/blob/d95ca1ea/docs/journal/2026-11-05-compact30-staged.md)
- [The CI run with the stage costs](https://github.com/lmmx/mibz2/actions/runs/42310037840)
