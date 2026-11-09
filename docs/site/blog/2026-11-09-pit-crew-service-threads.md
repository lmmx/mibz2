---
title: "mibz2 gets a pit crew"
image: assets/social/blog-2026-11-09-pit-crew-service-threads.png
description: "Handing each block's serial steps to two dedicated threads gives mibz2 a rare double win, decoding 4.5% faster on ten cores with 13% less CPU."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# mibz2 gets a pit crew

Handing each block's serial steps to two dedicated threads gives mibz2 a rare double win,
decoding 4.5% faster on ten cores with 13% less CPU.
{: .post-dek }

9 November 2026 · 16 min read · [Research in progress](#8-where-do-things-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2, our bzip2 decoder written in Rust, decodes the blocks of a file in parallel, each block on a
team of threads. Most of a block's work can be shared among a team's threads, but two steps can't:
the move-to-front step before the shared part, and the finish after it. In release 0.2.6, as the
[last post](2026-11-07-pipelined-teams.md) showed, each team's leader runs those steps itself while
its two helpers work on the shared part. This post is about the next step: taking that one-thread
work out of the teams altogether. Two threads of their own, which we call service threads, do it
for every block, and the teams only do the shared part. They are the teams' pit crew, so the teams
never have to leave the track.

**Threads asked for, and threads that decode.** A caller tells mibz2 how many threads it may use,
and we call that number the thread setting. Not all of those threads work on blocks. One, the
scanner, reads the file, and a few more do bookkeeping. The rest are the decoding threads, and the
two layouts spend them differently. At a setting of ten, 0.2.6 decodes with nine threads, three
teams of three. The service layout decodes with eight: two teams of three plus the two service
threads. So it inverts two blocks at a time instead of three.

**What we found.** On the ten-core machine we test on, the service layout decoded our test file
4.5% faster than 0.2.6's default and spent 12.7% less processor time. That comes from a registered
run, one whose comparisons and decision rules were fixed before it ran. The simplest reading is
that freeing every team member for the shared work more than pays for having one team fewer, as
long as the pit crew keeps up. Two service threads did; one did not.

Three limits come with it. The layout is a prototype on a research branch, not mibz2's default. It
was measured on one processor, an Intel Core i9-10900K, and principally on one file, a sample of
the Wikidata dump. And the gain belongs to the layout as a package. Two teams without service
threads were slower, so the service threads matter. But the runs changed several things at once,
and which of them earns the gain is not yet separated.

The results in brief:

- **The registered gain:** 4.5% faster than the default, with 12.7% less processor time and 10 MB
  less peak memory. A later run, which didn't judge the comparison, measured 7.3%.
- **Two teams alone lose:** without service threads, two teams were slower than the default, so the
  gain doesn't come simply from running fewer teams.
- **One service thread is too few:** with one, the layout was 20% slower, because that thread
  couldn't keep up. Adding threads instead, three teams plus two service threads at a setting of
  twelve, bought speed with more processor time: a trade, not a win.
- **Preparing further ahead didn't detectably help:** letting the service threads prepare three or four blocks
  ahead, instead of two, was inconclusive. The teams still wait about a sixth of the time, and what
  they wait for isn't yet identified.
- **Not yet a default:** a safety review cleared the two research versions that were measured, not
  a production version. A default needs its own measurements and its own review.

The figure sets the comparisons with the default side by side. Each dot is a speed ratio, the
default's elapsed time over the layout's, so above 1 is faster.

<figure class="bf-fig">
<p class="bf-title">The service layout against the 0.2.6 default</p>
<svg viewBox="0 0 720 586" role="img" aria-label="Speed ratios with 95% intervals, against the 0.2.6 default at a thread setting of ten: one service thread with three teams 0.799, registered; two pipelined teams without service threads 0.866, registered; the service layout 1.045 reported in the first run, 1.045 registered in the second, 1.073 reported in the third."><text x="0" y="24" class="bf-ts" style="font-size: 24px">Against the 0.2.6 default</text><path d="M122.5 44V454" class="bf-grid"/><text x="122.5" y="482" class="bf-tm" text-anchor="middle" style="font-size: 21px">0.80</text><path d="M287.5 44V454" class="bf-grid"/><text x="287.5" y="482" class="bf-tm" text-anchor="middle" style="font-size: 21px">0.90</text><path d="M452.5 44V454" class="bf-rule"/><text x="452.5" y="482" class="bf-tm" text-anchor="middle" style="font-size: 21px">1.00</text><path d="M617.5 44V454" class="bf-grid"/><text x="617.5" y="482" class="bf-tm" text-anchor="middle" style="font-size: 21px">1.10</text><text x="370" y="512" class="bf-tm" text-anchor="middle" style="font-size: 21px">speed against the default (above 1 is faster)</text><text x="0" y="66" class="bf-t" style="font-size: 23px">One service thread, 3 × 3 + 1 (pull request 251)</text><path d="M99.4 96H142.3" class="bf-cut"/><circle cx="120.9" cy="96" r="9" class="bf-dot-hi"/><text x="158.3" y="104" class="bf-t-strong" style="font-size: 22px">0.799 (0.786–0.812)</text><text x="0" y="150" class="bf-t" style="font-size: 23px">Two pipelined teams, no service threads (258)</text><path d="M206.7 180H256.2" class="bf-cut"/><circle cx="231.4" cy="180" r="9" class="bf-dot-hi"/><text x="272.2" y="188" class="bf-t-strong" style="font-size: 22px">0.866 (0.851–0.881)</text><text x="0" y="234" class="bf-t" style="font-size: 23px">Service layout, first run (251)</text><path d="M490.4 264H564.7" class="bf-cut"/><circle cx="526.8" cy="264" r="9" class="bf-dot-ready"/><text x="474.4" y="272" class="bf-t" text-anchor="end" style="font-size: 22px">1.045 (1.023–1.068)</text><text x="0" y="318" class="bf-t" style="font-size: 23px">Service layout, registered run (258)</text><path d="M480.5 348H573.0" class="bf-cut"/><circle cx="526.8" cy="348" r="9" class="bf-dot-hi"/><text x="464.5" y="356" class="bf-t-strong" text-anchor="end" style="font-size: 22px">1.045 (1.017–1.073)</text><text x="0" y="402" class="bf-t" style="font-size: 23px">Service layout, later run (271)</text><path d="M520.1 432H627.4" class="bf-cut"/><circle cx="573.0" cy="432" r="9" class="bf-dot-ready"/><text x="504.1" y="440" class="bf-t" text-anchor="end" style="font-size: 22px">1.073 (1.041–1.106)</text><circle cx="12" cy="562" r="9" class="bf-dot-hi"/><text x="30" y="569" class="bf-tm" style="font-size: 21px">registered, judged</text><circle cx="262" cy="562" r="9" class="bf-dot-ready"/><text x="280" y="569" class="bf-tm" style="font-size: 21px">reported, never judged</text></svg>
<figcaption>Measured, the test machine, a thread setting of ten, 20 paired rounds each. Each dot is the geometric mean of a run's paired ratios, and its line the 95% interval of that estimate, within that run; it is not the range of individual rounds. Green dots are comparisons registered as decisions before their run; amber dots were reported beside them and never judged. The numbers in brackets are the pull requests of the runs.</figcaption>
</figure>

[Section 1](#1-what-does-a-service-thread-do) shows what a service thread does, and
[section 2](#2-why-might-it-pay-and-why-might-it-not) why that might pay and why it might not.
Sections [3](#3-why-isnt-one-service-thread-enough) to [5](#5-has-the-result-been-seen-again) are
the three runs: one service thread failing, two beating the default with the controls beside them,
and the result seen again. [Section 6](#6-what-still-holds-the-teams-back) is about what still
holds the teams back, and [section 7](#7-what-does-it-cost-and-where-does-it-apply) about the
costs, the scope and what a default would need.

## 1. What does a service thread do?

A bzip2 file is a series of blocks of up to 900 kB, each compressed with no reference to the
others, so each can be decoded on its own. One thread, the scanner, reads the file and undoes the
outermost layer of every block, the Huffman coding. The rest of a block's work is in three parts:

1. **MTF:** undoing the move-to-front coding, which turns the scanner's symbols back into the
   block's bytes, though not yet in their original order.
2. **The inverse BWT:** undoing the Burrows–Wheeler transform, which puts the bytes back in their
   original order. It has two phases. The **build** fills a table with one 4-byte entry for each
   byte of the block, 3.6 MB for a full block. The **walk** follows the table from entry to entry.
   Both can be split among several threads.
3. **The finish:** putting the walk's pieces together, RLE1 (expanding runs of repeated bytes), and
   the CRC (checking the result against the block's checksum). Then the block is handed to the
   reader, in file order.

MTF and the finish run on one thread each. The inverse BWT is the expensive part, and the only part
a team shares. In 0.2.6, at a setting of ten or more on a machine of the test machine's shape, the
decoder runs three teams of three. Each team has a leader and two helpers. While the helpers build
and walk one block, the leader finishes the block before it and runs MTF on the block after it,
then joins the walk.

The service layout takes MTF and the finish away from the teams. Two service threads do them for
every block, serving both teams. Each team's three threads only build and walk. The figure shows
the two layouts at a setting of ten.

<figure class="bf-fig">
<p class="bf-title">Who does what, at a thread setting of ten</p>
<svg viewBox="0 0 720 516" role="img" aria-label="Two layouts of ten threads. Above, 0.2.6: three teams, each a leader L and two helpers H; the leader runs MTF and the finish and joins the walk. Below, the service layout: two teams of three helpers that only build and walk, and two service threads S that run every block's MTF and finish."><text x="0" y="26" class="bf-ts" style="font-size: 25px">0.2.6: three pipelined teams of three</text><text x="0" y="56" class="bf-tm" style="font-size: 21px">9 decoding threads, 3 blocks inverted at once</text><rect x="0" y="70" width="192" height="62" rx="8" class="bf-card-hi"/><rect x="6" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="34.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">L</text><rect x="68" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="96.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="130" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="158.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="202" y="70" width="192" height="62" rx="8" class="bf-card-hi"/><rect x="208" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="236.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">L</text><rect x="270" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="298.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="332" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="360.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="404" y="70" width="192" height="62" rx="8" class="bf-card-hi"/><rect x="410" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="438.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">L</text><rect x="472" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="500.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="534" y="76" width="56" height="50" rx="5" class="bf-pill"/><text x="562.0" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><text x="0" y="202" class="bf-ts" style="font-size: 25px">Service layout: 2 teams of 3, 2 service threads</text><text x="0" y="232" class="bf-tm" style="font-size: 21px">8 decoding threads, 2 blocks inverted at once</text><rect x="0" y="246" width="192" height="62" rx="8" class="bf-card-hi"/><rect x="6" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="34.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="68" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="96.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="130" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="158.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="202" y="246" width="192" height="62" rx="8" class="bf-card-hi"/><rect x="208" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="236.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="270" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="298.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="332" y="252" width="56" height="50" rx="5" class="bf-pill"/><text x="360.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">H</text><rect x="404" y="252" width="56" height="50" rx="5" class="bf-pill-next"/><text x="432.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">S</text><rect x="466" y="252" width="56" height="50" rx="5" class="bf-pill-next"/><text x="494.0" y="286" class="bf-t-strong" text-anchor="middle" style="font-size: 24px">S</text><path d="M0 172H720" class="bf-rule"/><path d="M0 346H720" class="bf-rule"/><text x="0" y="382" class="bf-t" style="font-size: 22px">L: a leader. It runs MTF and the finish, then</text><text x="30" y="410" class="bf-t" style="font-size: 22px">joins the walk.</text><text x="0" y="444" class="bf-t" style="font-size: 22px">H: a thread that only builds and walks a table.</text><text x="0" y="478" class="bf-t" style="font-size: 22px">S: a service thread: MTF and the finish, for</text><text x="30" y="506" class="bf-t" style="font-size: 22px">both teams.</text></svg>
<figcaption>A sketch of the layouts, from the decoder's code; it shows which thread does which stage, not timing. Each green outline is a team. Both layouts also run the scanner and a few bookkeeping threads, not drawn; the service layout adds one more of these, a feeder that passes the scanner's blocks to the service threads.</figcaption>
</figure>

Work moves between the two kinds of thread through two queues. A service thread takes a block
from the scanner, runs its MTF, and puts it on the **prepared queue**. A team takes the next
prepared block, builds and walks its table, and puts the result on the **finishing queue**. A
service thread then takes it from there, finishes it, and hands it on. The buffers each stage
needs are a fixed set, reused from block to block, so the queues can't grow without bound.

<figure class="bf-fig">
<p class="bf-title">A block's path through the service layout</p>
<svg viewBox="0 0 720 702" role="img" class="bf-play" aria-label="A block's path, drawn as a race circuit with seven numbered stops. 1, the grid: the scanner, Huffman decode. 2, the pit crew: a service thread prepares the block, MTF. 3, a pit bay: the prepared queue, where it waits for a team. 4, on track: a team of three builds and walks. 5, a pit bay: the finishing queue of walked blocks. 6, the pit crew: a service thread finishes the block, RLE1 and CRC. 7, the finish line: the reader writes the output in order. Coral marks the service threads, blue the team, green the scanner, the queues and the reader. Either service thread does either job."><defs><filter id="bf-play-glow" x="-20%" y="-30%" width="140%" height="160%"><feGaussianBlur stdDeviation="9"/></filter><pattern id="bf-play-check" width="16" height="16" patternUnits="userSpaceOnUse" x="40" y="0"><rect width="8" height="8" class="bf-play-checkd"/><rect x="8" y="8" width="8" height="8" class="bf-play-checkd"/></pattern></defs><g filter="url(#bf-play-glow)" class="bf-play-glow"><path d="M210 262H510A80 80 0 0 1 510 422H210A80 80 0 0 1 210 262Z" class="bf-play-blue" stroke-width="44"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-green" stroke-width="44" stroke-dasharray="174.0 9999" stroke-dashoffset="0.0"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-coral" stroke-width="44" stroke-dasharray="208.0 9999" stroke-dashoffset="-176.0"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-green" stroke-width="44" stroke-dasharray="381.6 9999" stroke-dashoffset="-386.0"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-green" stroke-width="44" stroke-dasharray="381.6 9999" stroke-dashoffset="0.0"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-coral" stroke-width="44" stroke-dasharray="208.0 9999" stroke-dashoffset="-383.6"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-green" stroke-width="44" stroke-dasharray="174.0 9999" stroke-dashoffset="-593.6"/></g><g class="bf-play-band"><path d="M210 262H510A80 80 0 0 1 510 422H210A80 80 0 0 1 210 262Z" class="bf-play-blue" stroke-width="32"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-green" stroke-width="32" stroke-dasharray="174.0 9999" stroke-dashoffset="0.0"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-coral" stroke-width="32" stroke-dasharray="208.0 9999" stroke-dashoffset="-176.0"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-green" stroke-width="32" stroke-dasharray="381.6 9999" stroke-dashoffset="-386.0"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-green" stroke-width="32" stroke-dasharray="381.6 9999" stroke-dashoffset="0.0"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-coral" stroke-width="32" stroke-dasharray="208.0 9999" stroke-dashoffset="-383.6"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-green" stroke-width="32" stroke-dasharray="174.0 9999" stroke-dashoffset="-593.6"/></g><path d="M210 262H510A80 80 0 0 1 510 422H210A80 80 0 0 1 210 262Z" class="bf-play-shine"/><path d="M34 172H560C670 172 670 262 510 262" class="bf-play-shine"/><path d="M510 422C670 422 670 512 560 512H34" class="bf-play-shine"/><path d="M56 157V187" class="bf-play-start"/><rect x="40" y="496" width="16" height="32" class="bf-play-checkl"/><rect x="40" y="496" width="16" height="32" fill="url(#bf-play-check)"/><g class="bf-play-token" style="offset-path: path('M34 172H560C670 172 670 262 510 262H210A80 80 0 0 0 210 422H510A80 80 0 0 0 510 262H210A80 80 0 0 0 210 422H510C670 422 670 512 560 512H34')"><g transform="scale(1.2)"><rect x="-13" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-13" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-20" y="-13" width="5" height="26" rx="2" class="bf-play-wing"/><rect x="-16" y="-13" width="34" height="26" rx="8" class="bf-play-chip"/><path d="M-12 -4H-1M-12 4H-1" class="bf-play-stripe"/><circle cx="6" cy="0" r="5.5" class="bf-play-helmet"/></g></g><g class="bf-play-still" transform="translate(156 172) rotate(0)"><g transform="scale(1.2)"><rect x="-13" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-13" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-20" y="-13" width="5" height="26" rx="2" class="bf-play-wing"/><rect x="-16" y="-13" width="34" height="26" rx="8" class="bf-play-chip"/><path d="M-12 -4H-1M-12 4H-1" class="bf-play-stripe"/><circle cx="6" cy="0" r="5.5" class="bf-play-helmet"/></g></g><g class="bf-play-still" transform="translate(578 172) rotate(0)"><g transform="scale(1.2)"><rect x="-13" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-13" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-20" y="-13" width="5" height="26" rx="2" class="bf-play-wing"/><rect x="-16" y="-13" width="34" height="26" rx="8" class="bf-play-chip"/><path d="M-12 -4H-1M-12 4H-1" class="bf-play-stripe"/><circle cx="6" cy="0" r="5.5" class="bf-play-helmet"/></g></g><g class="bf-play-still" transform="translate(460 512) rotate(180)"><g transform="scale(1.2)"><rect x="-13" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="-18" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-13" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="6" y="11" width="10" height="7" rx="2.5" class="bf-play-wheel"/><rect x="-20" y="-13" width="5" height="26" rx="2" class="bf-play-wing"/><rect x="-16" y="-13" width="34" height="26" rx="8" class="bf-play-chip"/><path d="M-12 -4H-1M-12 4H-1" class="bf-play-stripe"/><circle cx="6" cy="0" r="5.5" class="bf-play-helmet"/></g></g><polygon points="100.0,154.0 118.2,185.5 81.8,185.5" class="bf-play-rim bf-play-green"/><polygon points="100.0,154.0 118.2,185.5 81.8,185.5" class="bf-play-badge bf-play-green"/><text x="100.0" y="180.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">1</text><rect x="37.3" y="45" width="125.4" height="32" rx="16" class="bf-play-tag bf-play-green"/><text x="100.0" y="68" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">The grid</text><text x="100" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Scanner</text><text x="100" y="138" class="bf-tm" text-anchor="middle" style="font-size: 21px">Huffman decode</text><polygon points="318.0,190.0 299.8,158.5 336.2,158.5" class="bf-play-rim bf-play-coral"/><polygon points="318.0,190.0 299.8,158.5 336.2,158.5" class="bf-play-badge bf-play-coral"/><text x="318.0" y="180.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">2</text><rect x="255.3" y="45" width="125.4" height="32" rx="16" class="bf-play-tag bf-play-coral"/><text x="318.0" y="68" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">Pit crew</text><text x="318" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Service thread</text><text x="318" y="138" class="bf-tm" text-anchor="middle" style="font-size: 21px">prepare: MTF</text><polygon points="520.0,154.0 538.2,185.5 501.8,185.5" class="bf-play-rim bf-play-green"/><polygon points="520.0,154.0 538.2,185.5 501.8,185.5" class="bf-play-badge bf-play-green"/><text x="520.0" y="180.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">3</text><rect x="463.4" y="45" width="113.3" height="32" rx="16" class="bf-play-tag bf-play-green"/><text x="520.0" y="68" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">Pit bay</text><text x="520" y="110" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Prepared queue</text><text x="520" y="138" class="bf-tm" text-anchor="middle" style="font-size: 21px">waits for a team</text><polygon points="130.0,360.0 111.8,328.5 148.2,328.5" class="bf-play-rim bf-play-blue"/><polygon points="130.0,360.0 111.8,328.5 148.2,328.5" class="bf-play-badge bf-play-blue"/><text x="130.0" y="350.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">4</text><rect x="214.0" y="289.0" width="125.4" height="32" rx="16" class="bf-play-tag bf-play-blue"/><text x="276.7" y="312.0" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">On track</text><text x="214" y="348.0" class="bf-t-strong" style="font-size: 23px">A team of three</text><text x="214" y="376.0" class="bf-tm" style="font-size: 21px">build and walk</text><polygon points="520.0,494.0 538.2,525.5 501.8,525.5" class="bf-play-rim bf-play-green"/><polygon points="520.0,494.0 538.2,525.5 501.8,525.5" class="bf-play-badge bf-play-green"/><text x="520.0" y="520.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">5</text><rect x="463.4" y="551" width="113.3" height="32" rx="16" class="bf-play-tag bf-play-green"/><text x="520.0" y="574" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">Pit bay</text><text x="520" y="614" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Finishing queue</text><text x="520" y="642" class="bf-tm" text-anchor="middle" style="font-size: 21px">walked blocks</text><polygon points="318.0,530.0 299.8,498.5 336.2,498.5" class="bf-play-rim bf-play-coral"/><polygon points="318.0,530.0 299.8,498.5 336.2,498.5" class="bf-play-badge bf-play-coral"/><text x="318.0" y="520.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">6</text><rect x="255.3" y="551" width="125.4" height="32" rx="16" class="bf-play-tag bf-play-coral"/><text x="318.0" y="574" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">Pit crew</text><text x="318" y="614" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Service thread</text><text x="318" y="642" class="bf-tm" text-anchor="middle" style="font-size: 21px">finish: RLE1, CRC</text><polygon points="100.0,494.0 118.2,525.5 81.8,525.5" class="bf-play-rim bf-play-green"/><polygon points="100.0,494.0 118.2,525.5 81.8,525.5" class="bf-play-badge bf-play-green"/><text x="100.0" y="520.0" class="bf-play-num" text-anchor="middle" style="font-size: 22px">7</text><rect x="19.0" y="551" width="162.0" height="32" rx="16" class="bf-play-tag bf-play-green"/><text x="100.0" y="574" class="bf-play-tagt" text-anchor="middle" style="font-size: 21px">Finish line</text><text x="100" y="614" class="bf-t-strong" text-anchor="middle" style="font-size: 23px">Reader</text><text x="100" y="642" class="bf-tm" text-anchor="middle" style="font-size: 21px">output in order</text><polygon points="13.0,675.0 20.8,688.5 5.2,688.5" class="bf-play-key bf-play-coral"/><polygon points="28.0,693.0 20.2,679.5 35.8,679.5" class="bf-play-key bf-play-coral"/><text x="50" y="692" class="bf-tm" style="font-size: 21px">Either service thread does either job: one crew for both pit stops.</text></svg>
<figcaption>A sketch of the stages and queues, not timing: the car is one block, and its drive shows the order of its stops, not how long each takes. Coral is work on a service thread (the pit crew), blue work on a team (on the track), green the scanner, the queues and the reader. A service thread that becomes free finishes a walked block first, unless the prepared queue is empty, in which case it prepares one.</figcaption>
</figure>

A free service thread has to choose between preparing and finishing. Each time it picks its next
job, it finishes first, because finishing frees buffers and gets output to the reader. The
exception is an empty prepared queue: then it prepares, because an idle team wastes three threads.
The rule acts only at that choice. A thread already in the middle of a finish isn't interrupted, so
an empty queue isn't always refilled at once.

A service thread also won't start preparing while a set number of blocks already wait in the
prepared queue. That keeps preparing from running ahead and holding buffers that finishing needs.
The number is the **admission threshold**, and it is two, one per team.
[Section 6](#6-what-still-holds-the-teams-back) comes back to it.

## 2. Why might it pay, and why might it not?

**Why it might pay.** In 0.2.5's plain teams, a team's two helpers had nothing to do for roughly a
third of each block, while the leader ran MTF and the finish alone. 0.2.6 fills that time by
overlapping, at a price: the leader leaves the build, so the shared part of each block runs on one
thread fewer. The figure shows the difference over time.

<figure class="bf-fig">
<p class="bf-title">A team's time, before and after the pit crew</p>
<svg viewBox="0 0 720 632" role="img" class="bf-play" aria-label="Schematic timelines. In 0.2.6 a team's leader finishes block n minus 1 and runs MTF on block n plus 1 while its two helpers build block n, then joins the walk. In the service layout all three team threads build and walk, and two service threads run MTF and the finish for both teams. Widths are not measured."><text x="0" y="26" class="bf-ts" style="font-size: 24px">0.2.6: the leader makes its own pit stops</text><text x="0" y="72" class="bf-t" style="font-size: 22px">Leader</text><rect x="132" y="48" width="127" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="196" y="72" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish n−1</text><rect x="262" y="48" width="165" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="344" y="72" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">MTF n+1</text><rect x="430" y="48" width="278" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="569" y="72" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><text x="0" y="114" class="bf-t" style="font-size: 22px">Helper</text><rect x="132" y="90" width="295" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="280" y="114" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n</text><rect x="430" y="90" width="278" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="569" y="114" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><text x="0" y="156" class="bf-t" style="font-size: 22px">Helper</text><rect x="132" y="132" width="295" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="280" y="156" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n</text><rect x="430" y="132" width="278" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="569" y="156" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><text x="0" y="190" class="bf-tm" style="font-size: 20px">Block n’s build runs on two threads while the leader is in the pits.</text><path d="M0 214H720" class="bf-rule"/><text x="0" y="252" class="bf-ts" style="font-size: 24px">Service layout: a pit crew makes the stops</text><text x="0" y="298" class="bf-t" style="font-size: 22px">Team</text><rect x="132" y="274" width="225" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="244" y="298" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n</text><rect x="360" y="274" width="197" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="458" y="298" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><rect x="560" y="274" width="148" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="634" y="298" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n+2</text><text x="0" y="340" class="bf-t" style="font-size: 22px">Team</text><rect x="132" y="316" width="225" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="244" y="340" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n</text><rect x="360" y="316" width="197" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="458" y="340" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><rect x="560" y="316" width="148" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="634" y="340" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n+2</text><text x="0" y="382" class="bf-t" style="font-size: 22px">Team</text><rect x="132" y="358" width="225" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="244" y="382" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n</text><rect x="360" y="358" width="197" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="458" y="382" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk n</text><rect x="560" y="358" width="148" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="634" y="382" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build n+2</text><text x="0" y="438" class="bf-t" style="font-size: 22px">Service 1</text><rect x="132" y="414" width="115" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="190" y="438" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish</text><rect x="262" y="414" width="97" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="310" y="438" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">MTF</text><rect x="420" y="414" width="122" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="481" y="438" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish</text><text x="0" y="480" class="bf-t" style="font-size: 22px">Service 2</text><rect x="160" y="456" width="99" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="210" y="480" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">MTF</text><rect x="300" y="456" width="122" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="361" y="480" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish</text><rect x="575" y="456" width="99" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="624" y="480" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">MTF</text><text x="0" y="522" class="bf-tm" style="font-size: 20px">Team threads only build and walk; the crew serves both teams.</text><path d="M132 548H708" class="bf-arrow"/><path d="M700 542L708 548L700 554" class="bf-arrow"/><text x="708" y="574" class="bf-tm" text-anchor="end" style="font-size: 20px">time (schematic, not to scale)</text><rect x="0" y="595" width="28" height="22" rx="6" class="bf-play-tag bf-play-blue"/><text x="38" y="612" class="bf-tm" style="font-size: 20px">on track: build and walk</text><rect x="330" y="595" width="28" height="22" rx="6" class="bf-play-tag bf-play-coral"/><text x="368" y="612" class="bf-tm" style="font-size: 20px">in the pits: MTF and finish</text></svg>
<figcaption>Schematic, from the decoder's code, not measured: the widths are not durations. Coral is the one-thread work (in the pits), blue the shared build and walk (on the track). n is the block the team is inverting. In the service layout the service threads' jobs come from both teams' blocks.</figcaption>
</figure>

In the service layout no team member leaves the build and walk to run MTF or the finish. The teams
can still wait for those stages, for a prepared block or for free buffers, as
[section 6](#6-what-still-holds-the-teams-back) shows.

**Why it might not.** There are three costs. First, at a setting of ten the layout makes two teams,
not three, so only two blocks are inverted at once. Second, all the one-thread work of every block
goes through the service threads. If they can't keep up, both teams wait for them. Third, every
block now crosses between threads twice, from a service thread to a team and back, and each
hand-off takes some time.

Before the first run, a model built from earlier single-thread measurements put the one-thread
work at about 0.57 nanoseconds per byte of output. At the rate the decoder then reached on this
machine, that came to about 0.8 of one core. On paper, then, one service thread looked nearly
enough. The model said itself that its inputs came from another machine and another layout.

## 3. Why isn't one service thread enough?

The first run ([pull request 251](https://github.com/lmmx/mibz2/pull/251)) was registered. Its
first attempt failed its noise check, which runs two copies of the same decoder under different
names, and was discarded. An unchanged rerun was registered before it ran, and passed that check
at 1.003. The test file is the Wikidata sample: 11.85 MB of bzip2 that decodes to 209.7 MB. Each
arm decoded it 20 times, the arms taking turns in each round.

**Speed** here and below is the reference's elapsed time over the arm's, so above 1 means the arm
is faster. **CPU** is the reference's processor time over the arm's, all threads added up, so
above 1 means the arm spends less. Both are paired round by round. Each ratio is the geometric
mean of the 20 rounds' ratios, and its 95% interval is the uncertainty of that estimate within
one run. It is not the range of the individual rounds.

With one service thread beside three teams of three, at a setting of ten, the decoder was much
slower than the default. The trace showed why. The service thread was marked busy 96% of the time,
finished blocks backed up behind it, and the teams waited for it. Moving the one-thread work onto
one thread had made that thread the bottleneck. The model had put the work at 0.8 of a core, but
in practice one thread couldn't keep up.

The run's judged comparisons with the default:

| arm (decoding threads, against 9) | setting | speed (95%) | CPU (95%) |
| --- | ---: | ---: | ---: |
| one service thread, 3 × 3 + 1 (10) | 10 | 0.799 (0.786–0.812) | 1.099 (1.079–1.118) |
| two service threads, 3 × 3 + 2 (11) | 12 | 1.058 (1.035–1.081) | 0.901 (0.888–0.914) |

The second row is two service threads beside three teams of three, at a setting of twelve. It was
5.8% faster than the default, but spent 11% more processor time and 8 MB more peak memory, on
eleven decoding threads to the default's nine. A gain bought with more threads and more processor
time is no reason to change a default, so we didn't pursue it.

The run also measured the layout this post is about, two teams of three plus two service threads,
at a setting of ten. It was 4.5% faster than the default, with 12% less processor time and 10 MB
less peak memory. But that comparison was reported, not registered as a decision. And the run had
no plain two-team arm, so it couldn't say whether the gain came from the service threads or simply
from running two teams. The next run was designed to answer both.

## 4. Did two service threads beat the default?

The second run ([pull request 258](https://github.com/lmmx/mibz2/pull/258)) judged three
comparisons, each against an arm chosen to answer one question:

- **Against the default:** is the service layout faster than 0.2.6 at a setting of ten?
- **Against plain two teams:** two teams of three without service threads, each leader doing its
  own MTF and finish, at the same setting. Do the service threads make the difference?
- **Two pipelined teams against the default:** 0.2.6's own pipelined teams, but two of them. Does
  today's decoder gain just from running two teams?

The run passed every validity check, including its noise check at 0.997. The judged comparisons:

| comparison (decoding threads) | speed (95%) | CPU (95%) |
| --- | ---: | ---: |
| service vs default (8 vs 9) | **1.045 (1.017–1.073)** | 1.146 (1.123–1.170) |
| service vs plain two teams (8 vs 6) | 1.402 (1.368–1.437) | 0.941 (0.924–0.959) |
| two pipelined teams vs default (6 vs 9) | 0.866 (0.851–0.881) | 1.214 (1.196–1.233) |

The service layout beat the default, by the margin the first run had reported. Its CPU ratio of
1.146 means it spent 0.873 of the default's processor time. That is 12.7% less, or between about
11% and 14.5% less over the interval. By the rule fixed before the run, a speed gain without a
processor-time cost, together with a gain over plain two teams, was the outcome "for the service
layout".

**The controls rule out the simplest explanation.** Two teams alone did not reproduce the gain.
Plain two teams were far slower than the service layout, and 0.2.6's pipelined teams, cut to two,
reached only 0.866 of the default's speed. So the win doesn't come from having two teams instead
of three. The figure puts the four arms' costs side by side, from the run's medians.

<figure class="bf-fig">
<p class="bf-title">Time, processor time and memory, at a thread setting of ten</p>
<svg viewBox="0 0 720 712" role="img" aria-label="Medians from the registered run. Elapsed: default 0.144 s, service layout 0.137, two pipelined teams 0.166, plain two teams 0.193. Processor time: 1.026, 0.895, 0.850, 0.847 s. Peak memory: 74, 64, 57, 52 MB."><text x="0" y="26" class="bf-ts" style="font-size: 24px">Elapsed, s (lower is faster)</text><text x="0" y="69" class="bf-t" style="font-size: 22px">0.2.6 default</text><rect x="240" y="48" width="266.4" height="30" rx="3" class="bf-steel"/><text x="516.4" y="71" class="bf-t-strong" style="font-size: 22px">0.144</text><text x="0" y="111" class="bf-t" style="font-size: 22px">Service layout</text><rect x="240" y="90" width="253.5" height="30" rx="3" class="bf-green"/><text x="503.5" y="113" class="bf-t-strong" style="font-size: 22px">0.137</text><text x="0" y="153" class="bf-t" style="font-size: 22px">Two pipelined teams</text><rect x="240" y="132" width="307.1" height="30" rx="3" class="bf-slot"/><text x="557.1" y="155" class="bf-t-strong" style="font-size: 22px">0.166</text><text x="0" y="195" class="bf-t" style="font-size: 22px">Plain two teams</text><rect x="240" y="174" width="357.1" height="30" rx="3" class="bf-slot"/><text x="607.1" y="197" class="bf-t-strong" style="font-size: 22px">0.193</text><text x="0" y="256" class="bf-ts" style="font-size: 24px">Processor time, s</text><text x="0" y="299" class="bf-t" style="font-size: 22px">0.2.6 default</text><rect x="240" y="278" width="345.1" height="30" rx="3" class="bf-steel"/><text x="595.1" y="301" class="bf-t-strong" style="font-size: 22px">1.026</text><text x="0" y="341" class="bf-t" style="font-size: 22px">Service layout</text><rect x="240" y="320" width="301.0" height="30" rx="3" class="bf-green"/><text x="551.0" y="343" class="bf-t-strong" style="font-size: 22px">0.895</text><text x="0" y="383" class="bf-t" style="font-size: 22px">Two pipelined teams</text><rect x="240" y="362" width="285.9" height="30" rx="3" class="bf-slot"/><text x="535.9" y="385" class="bf-t-strong" style="font-size: 22px">0.850</text><text x="0" y="425" class="bf-t" style="font-size: 22px">Plain two teams</text><rect x="240" y="404" width="284.9" height="30" rx="3" class="bf-slot"/><text x="534.9" y="427" class="bf-t-strong" style="font-size: 22px">0.847</text><text x="0" y="486" class="bf-ts" style="font-size: 24px">Peak memory, MB</text><text x="0" y="529" class="bf-t" style="font-size: 22px">0.2.6 default</text><rect x="240" y="508" width="342.2" height="30" rx="3" class="bf-steel"/><text x="592.2" y="531" class="bf-t-strong" style="font-size: 22px">74</text><text x="0" y="571" class="bf-t" style="font-size: 22px">Service layout</text><rect x="240" y="550" width="296.0" height="30" rx="3" class="bf-green"/><text x="546.0" y="573" class="bf-t-strong" style="font-size: 22px">64</text><text x="0" y="613" class="bf-t" style="font-size: 22px">Two pipelined teams</text><rect x="240" y="592" width="263.6" height="30" rx="3" class="bf-slot"/><text x="513.6" y="615" class="bf-t-strong" style="font-size: 22px">57</text><text x="0" y="655" class="bf-t" style="font-size: 22px">Plain two teams</text><rect x="240" y="634" width="240.5" height="30" rx="3" class="bf-slot"/><text x="490.5" y="657" class="bf-t-strong" style="font-size: 22px">52</text><text x="0" y="702" class="bf-tm" style="font-size: 21px">Bars start at zero.</text></svg>
<figcaption>Measured, the test machine, medians of 20 rounds of the registered run (pull request 258). The service layout (green) is the only arm both faster and cheaper than the default (blue); the two two-team arms without service threads are cheaper but slower.</figcaption>
</figure>

Every two-team arm spent less processor time than the default. The service threads cost some of
that back, since the service layout spent about 6% more than plain two teams, but it alone is
faster than the default as well. The medians agree with the paired ratios without replacing them,
because the ratios are computed round by round.

**What the controls don't separate.** The comparison with plain two teams measures everything the
layout changes at once: two more threads, the one-thread work moved onto them, the feeder, the
queues and the hand-offs. They were not varied separately. The run is evidence for the layout as a
package, not for any one of its parts.

The model made before the run had predicted each control. Each came out on the predicted side, and
each a little beyond it:

| control | predicted | measured |
| --- | ---: | ---: |
| plain two teams, elapsed (s) | 0.180–0.188 | 0.193 |
| service vs plain two teams, speed | near 1.33 | 1.402 |
| two pipelined teams vs default, speed | 0.82–0.85 | 0.866 |

## 5. Has the result been seen again?

A third run ([pull request 271](https://github.com/lmmx/mibz2/pull/271)) asked another question,
about the admission threshold. Beside its own comparisons, it ran the service layout against the
default again, as a reported comparison. Its noise check came out at 0.994. The three runs that
measured the layout against the default:

| run (pull request) | status | speed (95%) | CPU (95%) | peak memory (MB) |
| --- | --- | ---: | ---: | ---: |
| first (251) | reported | 1.045 (1.023–1.068) | 1.138 (1.118–1.158) | 64 vs 74 |
| second (258) | registered | 1.045 (1.017–1.073) | 1.146 (1.123–1.170) | 64 vs 74 |
| third (271) | reported | 1.073 (1.041–1.106) | 1.169 (1.135–1.204) | 64 vs 74 |

The third run makes the same comparison as the registered one, on the same machine and file. But it
is another run, at a later commit of the prototype, beside other arms. It was never judged, so it
doesn't replace the registered 1.045. What it adds is that the direction held. In all three runs
the layout was faster than the default and spent 12–15% less processor time.

The three estimates also say something about the intervals. Each is the uncertainty of its own
run's estimate. The registered 1.045 and the later 1.073 differ by about half the width of either
interval. Neither interval says how much the figure moves from one run to the next. The two runs
also differ in the prototype's commit and in the arms beside it, so their difference isn't a clean
measure of repeatability. It does suggest the figure can move by a few points. The fair summary is
a gain of a few per cent, measured on one machine, not a precise 4.5% or 7.3%.

## 6. What still holds the teams back?

The registered run traced the service layout as it decoded, over six further rounds. The two
service threads were marked busy for about 70% and 67% of the run, far from the single service
thread's 96%. Yet the teams spent 16.5% of their time idle, in waits that began with no prepared
block available. Why would the teams wait for prepared blocks while the threads that prepare them
have time to spare?

**Two gates.** A block passes two gates before a team can take it, and they are easy to confuse.

**Gate 1 is the admission threshold** from section 1. A service thread won't start preparing while
two prepared blocks already wait. It is checked when a thread picks a job, so it limits when
preparing starts, not how many blocks can wait. With both service threads preparing at once, three
can.

**Gate 2 is the window,** which limits the blocks in flight. The scanner starts a new block only
while fewer than eight are in progress, from scanning to output. A block gives up its place only
when the reader takes it, and the reader takes blocks in file order.

**The first guess was gate 1.** With a threshold of two, a service thread can sit idle because two
blocks already wait. But a service thread idle earlier is not the same as one idle when a team runs
dry. Panel A of the figure below shows how the two can come apart. Service thread 1 is idle at
first, because gate 1 is shut. Later, when team 1 needs a block, both service threads are in the
middle of finishes, and nothing is prepared until one of them ends.

**The test.** The third run raised the threshold from two to three, and to four, with everything
else as in the registered run.

<figure class="bf-fig">
<p class="bf-title">A higher admission threshold, against the service layout at two</p>
<svg viewBox="0 0 720 316" role="img" aria-label="Speed ratios with 95% intervals, against the service layout at admission threshold two: threshold three 0.992 and threshold four 0.979, both registered and inconclusive."><text x="0" y="24" class="bf-ts" style="font-size: 24px">Against the service layout at threshold two</text><path d="M122.5 44V202" class="bf-grid"/><text x="122.5" y="230" class="bf-tm" text-anchor="middle" style="font-size: 21px">0.80</text><path d="M287.5 44V202" class="bf-grid"/><text x="287.5" y="230" class="bf-tm" text-anchor="middle" style="font-size: 21px">0.90</text><path d="M452.5 44V202" class="bf-rule"/><text x="452.5" y="230" class="bf-tm" text-anchor="middle" style="font-size: 21px">1.00</text><path d="M617.5 44V202" class="bf-grid"/><text x="617.5" y="230" class="bf-tm" text-anchor="middle" style="font-size: 21px">1.10</text><text x="370" y="260" class="bf-tm" text-anchor="middle" style="font-size: 21px">speed against threshold two (above 1 is faster)</text><text x="0" y="66" class="bf-t" style="font-size: 23px">Admission threshold three (271)</text><path d="M399.7 96H478.9" class="bf-cut"/><circle cx="439.3" cy="96" r="9" class="bf-dot-hi"/><text x="383.7" y="104" class="bf-t-strong" text-anchor="end" style="font-size: 22px">0.992 (0.968–1.016)</text><text x="0" y="150" class="bf-t" style="font-size: 23px">Admission threshold four (271)</text><path d="M371.7 180H465.7" class="bf-cut"/><circle cx="417.9" cy="180" r="9" class="bf-dot-hi"/><text x="355.7" y="188" class="bf-t-strong" text-anchor="end" style="font-size: 22px">0.979 (0.951–1.008)</text><circle cx="12" cy="290" r="9" class="bf-dot-hi"/><text x="30" y="297" class="bf-tm" style="font-size: 21px">registered, judged</text><circle cx="262" cy="290" r="9" class="bf-dot-ready"/><text x="280" y="297" class="bf-tm" style="font-size: 21px">reported, never judged</text></svg>
<figcaption>Measured, the test machine, a thread setting of ten, 20 paired rounds each (pull request 271); the dot and line are the paired geometric-mean ratio and its 95% interval within the run. The CPU ratios were 1.006 (0.987–1.025) for three and 0.983 (0.958–1.009) for four, also spanning 1.</figcaption>
</figure>

Both intervals include gains and losses of a few per cent, so neither change is shown to help or
to hurt. The point estimate for four leans slower, and a loss of up to about 5% isn't excluded.
Raising the threshold alone did not show a gain in this run.

**What the trace's labels can and can't say.** When the third run's result was first written up,
its traces were read as showing that the bottleneck had moved, from waiting for prepared blocks to
waiting for finishing. A review of the tracing code then showed that reading doesn't follow, and
it was withdrawn ([the review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231),
and the correction in [pull request 289](https://github.com/lmmx/mibz2/pull/289)). A team's wait is
labelled once, by what it saw when it started. So a wait that began with nothing prepared still
counts entirely as a wait for preparation, even if a block was prepared before it ended. And a
service thread's "busy" share is the time it was marked working, not processor time. The figures
stand as recorded, but they don't identify the bottleneck.

**Two possible waits.** The review names two explanations that survive, and the figure draws each
as a schematic. In panel A, both service threads are finishing when a team takes the last prepared
block. In panel B, gate 2 is the one that's shut. Finished blocks stuck behind an earlier,
unfinished one hold their places, so the scanner can't start a new block, and there is nothing to
prepare.

<figure class="bf-fig">
<p class="bf-title">Two ways a team can run out of prepared blocks</p>
<svg viewBox="0 0 720 750" role="img" class="bf-play" aria-label="Two schematic panels of possible waits, not measured. A: service thread 1 is idle early on because two prepared blocks already wait, the threshold; later both service threads are finishing when team 1 needs a block, so it waits until service thread 2 has finished and prepared one. B: the eight places in flight are held by block 3, still being walked, blocks 4 to 9, finished but waiting for block 3 to be taken in file order, and block 10, being finished. The scanner can't start block 11, so nothing new is prepared and team 2 waits."><text x="0" y="26" class="bf-ts" style="font-size: 24px">A. Both crews are finishing, and the bay runs dry</text><text x="0" y="74" class="bf-t" style="font-size: 22px">Service 1</text><rect x="132" y="50" width="165" height="34" rx="6" class="bf-track"/><text x="214" y="74" class="bf-tm" text-anchor="middle" style="font-size: 20px">idle</text><rect x="300" y="50" width="277" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="438" y="74" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish</text><text x="0" y="116" class="bf-t" style="font-size: 22px">Service 2</text><rect x="132" y="92" width="345" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="304" y="116" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">finish</text><rect x="480" y="92" width="97" height="34" rx="6" class="bf-play-tag bf-play-coral"/><text x="528" y="116" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">MTF</text><text x="0" y="158" class="bf-t" style="font-size: 22px">Prepared</text><rect x="132" y="134" width="165" height="34" rx="6" class="bf-pill"/><text x="214" y="158" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">2 waiting</text><rect x="300" y="134" width="97" height="34" rx="6" class="bf-pill"/><text x="348" y="158" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">1</text><rect x="400" y="134" width="177" height="34" rx="6" class="bf-pill"/><text x="488" y="158" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">0</text><rect x="580" y="134" width="128" height="34" rx="6" class="bf-pill"/><text x="644" y="158" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">0</text><text x="0" y="200" class="bf-t" style="font-size: 22px">Team 1</text><rect x="132" y="176" width="165" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="214" y="200" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk</text><rect x="300" y="176" width="157" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="378" y="200" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build, walk</text><rect x="460" y="176" width="117" height="34" rx="6" class="bf-track"/><text x="518" y="200" class="bf-tm" text-anchor="middle" style="font-size: 20px">waits</text><rect x="580" y="176" width="128" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="644" y="200" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build</text><text x="0" y="242" class="bf-t" style="font-size: 22px">Team 2</text><rect x="132" y="218" width="265" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="264" y="242" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">walk</text><rect x="400" y="218" width="308" height="34" rx="6" class="bf-play-tag bf-play-blue"/><text x="554" y="242" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">build, walk</text><circle cx="152" cy="66" r="13" class="bf-dot-ready"/><text x="152" y="72" class="bf-play-num" text-anchor="middle" style="font-size: 16px">1</text><circle cx="478" cy="192" r="13" class="bf-dot-ready"/><text x="478" y="198" class="bf-play-num" text-anchor="middle" style="font-size: 16px">2</text><path d="M132 274H708" class="bf-arrow"/><path d="M700 268L708 274L700 280" class="bf-arrow"/><text x="708" y="300" class="bf-tm" text-anchor="end" style="font-size: 20px">time (schematic, not to scale)</text><circle cx="13" cy="320" r="13" class="bf-dot-ready"/><text x="13" y="326" class="bf-play-num" text-anchor="middle" style="font-size: 16px">1</text><text x="34" y="327" class="bf-tm" style="font-size: 20px">Service 1 is free, but two blocks already wait: gate 1 is shut.</text><circle cx="13" cy="354" r="13" class="bf-dot-ready"/><text x="13" y="360" class="bf-play-num" text-anchor="middle" style="font-size: 16px">2</text><text x="34" y="361" class="bf-tm" style="font-size: 20px">Team 1 waits while both service threads are mid-finish.</text><path d="M0 386H720" class="bf-rule"/><text x="0" y="424" class="bf-ts" style="font-size: 24px">B. A moment when every place in the pit lane is held</text><text x="0" y="456" class="bf-tm" style="font-size: 20px">The eight places in flight, by block number</text><text x="0" y="499" class="bf-t" style="font-size: 22px">Places</text><rect x="132" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-blue"/><text x="166" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">3</text><rect x="204" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="238" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">4</text><rect x="276" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="310" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">5</text><rect x="348" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="382" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">6</text><rect x="420" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="454" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">7</text><rect x="492" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="526" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">8</text><rect x="564" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-green"/><text x="598" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">9</text><rect x="636" y="470" width="69" height="44" rx="6" class="bf-play-tag bf-play-coral"/><text x="670" y="499" class="bf-play-tagt" text-anchor="middle" style="font-size: 20px">10</text><text x="0" y="554" class="bf-t" style="font-size: 22px">Scanner</text><rect x="132" y="530" width="576" height="34" rx="6" class="bf-track"/><text x="420" y="554" class="bf-tm" text-anchor="middle" style="font-size: 20px">block 11 waits for a free place</text><text x="0" y="596" class="bf-t" style="font-size: 22px">Team 2</text><rect x="132" y="572" width="576" height="34" rx="6" class="bf-track"/><text x="420" y="596" class="bf-tm" text-anchor="middle" style="font-size: 20px">waits: nothing prepared</text><text x="0" y="638" class="bf-t" style="font-size: 22px">Reader</text><rect x="132" y="614" width="576" height="34" rx="6" class="bf-track"/><text x="420" y="638" class="bf-tm" text-anchor="middle" style="font-size: 20px">waits for block 3; taking it frees a place</text><rect x="0" y="677" width="28" height="22" rx="6" class="bf-play-tag bf-play-blue"/><text x="38" y="694" class="bf-tm" style="font-size: 20px">being walked</text><rect x="230" y="677" width="28" height="22" rx="6" class="bf-play-tag bf-play-green"/><text x="268" y="694" class="bf-tm" style="font-size: 20px">finished, waiting its turn</text><rect x="0" y="713" width="28" height="22" rx="6" class="bf-play-tag bf-play-coral"/><text x="38" y="730" class="bf-tm" style="font-size: 20px">being finished</text><rect x="230" y="713" width="28" height="22" rx="6" class="bf-track"/><text x="268" y="730" class="bf-tm" style="font-size: 20px">idle or waiting</text></svg>
<figcaption>Possible mechanisms, not a measured attribution: schematic, and the widths are not durations. In A, the pit crew is free early but gate 1 keeps it from preparing; later both crews are mid-finish when a team needs a block. In B, the window's eight places are all held, mostly by blocks that are finished but must wait for block 3 to reach the reader first.</figcaption>
</figure>

A higher threshold doesn't change the finish-first rule, interrupt a finish or free a place in the
window. In panel A it could still help a little. Had service thread 1 prepared more blocks while it
was idle, that stock might last through a short finish. This run didn't show that it does, and it
doesn't pick between the two panels.

One reported, unjudged arm touched gate 2 as well. It raised the threshold to four and also widened the
window to twelve. Against threshold four alone it measured 1.049, with an interval of 1.017 to
1.082. That arm was picked out after the run, so it is a lead for a future registered comparison,
not a result. The next step is to count how often each kind of wait happens, without timing
anything.

## 7. What does it cost, and where does it apply?

**Processor time and memory.** The service layout finishes sooner than 0.2.6's default and uses
less processor time, all threads added up: 12.7% less in the registered run. Processor seconds
don't measure the work itself, in instructions or energy, and where the saving comes from isn't
measured. Its peak memory was 64 MB against 74. Two teams hold two of the 3.6 MB tables at once
rather than three, and the service threads add buffers of their own for the blocks queued between
stages. Peak memory wasn't broken down by buffer.

**Fewer tables at once.** In an [earlier post](2026-11-07-why-mibz2-stops-scaling.md) we measured
that every extra block in progress slows the others. The leading explanation is the tables'
competition for the processor's shared 20 MiB cache, with evidence from a simpler test program.
Inverting two blocks at a time instead of three may help the service layout for that reason. These
runs didn't measure it, and the comparison with two pipelined teams shows that fewer tables alone
don't give the gain.

**Fewer decoding threads.** A caller who asks for ten threads gets eight decoding, two of them
service threads, plus the feeder. The layout depends on the service threads not saturating, and
their margin in the registered run was the trace's 70%. Under other load on the same machine, or on
a file with more small blocks or a different mix of stages, they could saturate first. Three teams
might then slow down more gently. None of that is measured.

**Where it applies.** Everything here was measured on one processor shape, the i9-10900K with 10
cores and a 20 MiB last-level cache, decoding principally one file. The headline comparison is at a
setting of ten; [section 3](#3-why-isnt-one-service-thread-enough) also reports three teams plus
two service threads at twelve. The cloud machines that benchmark every change to mibz2 have four
virtual CPUs, so they can't show the gain. They could show only that a change keyed to the ten-core
shape leaves every other machine alone.

**What a default would need.** The service layout is an opt-in prototype on a research branch,
reached only through hidden functions. The registered run's journal sets out what making it the
default would take, in order:

1. A rule that picks it only where it was measured: the ten-core shape at a setting of exactly ten.
2. The usual paired benchmark on every change, to show the rule changes nothing elsewhere.
3. A registered run of the production build's default against the 0.2.6 release on the test
   machine.
4. A safety review of the production code, with fuzzing that reaches its new paths.

Any step can stop it.

**The safety review so far.** The service layout adds new concurrency: who owns each buffer, how
blocks pass between threads, and what happens on cancellation, a panic or the end of the stream. An
independent review of it
([decision D045](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806))
found the ownership argument coherent. It still held further timing runs and any production use
until the evidence was complete. That meant fuzzing shown to reach the layout's paths, and runs of
Miri, a checker for undefined behaviour in Rust. A second review
([D053](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239))
kept the hold, narrowed to how the fuzz evidence was counted and to unfinished Miri runs. Once that
was repaired, the hold was lifted
([D057](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449))
for the two versions of the code that were measured. The lifting states that it does not qualify a
production candidate. A default would be new code, and needs its own review.

## 8. Where do things stand?

The argument runs like this. In 0.2.6 a team's leader runs each block's one-thread work beside its
helpers, which takes the leader out of the build. Moving that work onto threads of its own frees
every team member for the shared part, at the cost of one team fewer. One service thread couldn't
keep up. Two could: in a registered run, the layout was 4.5% faster than the default, on one
decoding thread fewer and with 12.7% less processor time. Two teams without service threads were
slower, so the service threads are part of the gain. Which part of the layout earns it isn't
separated. Preparing further ahead didn't detectably help, and what the teams still wait for isn't
yet identified.

Two things come next. The first is a count, without timing, of the two possible waits in section 6.
The second is the list of steps a default would need, starting with a rule that selects the layout
only where it was measured.

The general lesson: work that can't be shared can be moved off the threads that share the rest,
but the threads it moves to then set the pace. Here one was too few and two were enough, and only
measuring showed which.

---

**Research records:**

- [The service core's first registered run: its result](https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-08-service-core-result.md),
  [its registration](https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-07-service-core-registration.md)
  and [its bundle](https://github.com/lmmx/mibz2/blob/1e0bf7dcde9f8dd77cc12b9d577554407ec29e33/service-core/20261108T103216Z.tar.gz)
  (pull request 251)
- [Two teams and two service threads: the registered result](https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-2x3-result.md),
  [its registration](https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/docs/journal/2026-11-08-service-2x3-registration.md)
  and [its bundle](https://github.com/lmmx/mibz2/blob/4fccc02b9e95b8b9de5c3be48d0e86e37377de72/service-2x3/20261109T010708Z.tar.gz)
  (pull request 258)
- [The admission threshold: the registered result](https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-lookahead-result.md),
  [its registration](https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/docs/journal/2026-11-09-service-lookahead-registration.md)
  and [its bundle](https://github.com/lmmx/mibz2/blob/705319fd507d79ad545a593b5affa329275d1e00/service-lookahead/20261109T075939Z.tar.gz)
  (pull request 271)
- [The review of the traces' labels](https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231)
  and [the journal's correction](https://github.com/lmmx/mibz2/blob/f9284a4b96bb258454525599a447173e9b201df6/docs/journal/2026-11-09-service-lookahead-result.md)
  (pull request 289)
- The safety reviews:
  [D045](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L1806),
  [D053](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2239)
  and [D057](https://github.com/lmmx/mibz2/blob/726db3a9670be3516b091d2b2eb30abc3ce6f388/state/decisions.md?plain=1#L2449)
- [Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores](2026-11-07-pipelined-teams.md)
- An aside, unrelated to the service layout: a separate line of work, building the inverse BWT's
  table from the runs in bzip2's MTF codes ([an earlier post](2026-11-08-bwt-runs-from-mtf-codes.md)),
  had its first run on the test machine the same day. It was inconclusive, because the machine's
  speed changed during it
  ([its journal](https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-run-fed-leader-workstation-result.md)).
