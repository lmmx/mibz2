---
title: "Raising the knee: scaling throughput to the limit of physical cores"
image: assets/social/blog-2026-11-17-raising-the-knee.png
description: "mibz2 0.5.4 now speeds up until all ten physical cores of its test processor are busy, then holds level on the whole machine up to twenty threads."
releases: ["0.5.4"]
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Raising the knee: scaling throughput to the limit of physical cores

mibz2 0.5.4 now speeds up until all ten physical cores of its test processor are busy, then holds level on the whole machine up to twenty threads.
{: .post-dek }

17 November 2026 · 13 min read · [Research result](#6-what-comes-next-for-the-knee)
{: .post-meta }

Shipped in mibz2 [0.5.4](../benchmarks/history.md#release-0-5-4) ([changelog](https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#054---2026-11-17))
{: .post-shipped }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[Ten days ago](2026-11-07-why-mibz2-stops-scaling.md), mibz2's problem was that giving its
bzip2 decoder more work at once could make it slower. The decoder processes a compressed file in
independent blocks, so it can decode several simultaneously. But each block has its own data
structures, and those structures compete for the processor's shared cache. More blocks promise
more parallelism while making each block more expensive. Past a certain point the extra cost
wins. The throughput curve bends over: the *scaling knee*.

The default decoder once dealt with that risk by stopping at a fixed number of decoding teams.
It could avoid a fall, but only by declining to use the extra threads. Since then, mibz2 has
changed both how it reconstructs a block and how many workers it starts. Version 0.5.4 puts the
result in view. Our test machine is a ten-core Intel desktop (an i9-10900K, with 20 logical CPUs
and a 20 MiB level-3 cache that all ten cores share). On it, ten requested threads decode a
210 MB Wikidata sample in **0.102 seconds**, the fastest median in the release sweep. The knee is still there, but it has been raised and softened. It is raised because the best
point has moved from eight threads to ten, the machine's physical-core count. It is softened
because asking for more now costs little or nothing. In 0.4.0 and 0.5.0, 12 to 20 threads took
11–18% longer than the best point. In 0.5.4's release sweep they take 4–5% longer by the medians,
but that sweep pins ten threads and gives twelve or more the whole machine. With placement held
constant, 10, 12 and 20 threads decode in the same time, to within half a per cent. It is not straightened:
the extra hardware threads add nothing yet. At 12 to 20 requested threads 0.5.4 runs seven active
workers, not twelve or twenty. Memory bandwidth is close to binding there: the plateau
moves 75% of the memory traffic the machine can sustain, but that is not yet a hard ceiling.

<figure class="bf-fig">
<p class="bf-title">Throughput against threads, five releases on the test machine</p>
<svg viewBox="0 0 720 380" role="img" aria-label="Throughput against threads for five releases: 0.5.0 peaks at 8 threads and falls about 14% by 12; 0.5.4 peaks at 2.06 GB a second at 10 threads and stays between 1.95 and 2.06 from 8 to 20."><line x1="70" y1="330.0" x2="690" y2="330.0" class="bf-axis"/><text x="62" y="334.0" text-anchor="end" class="bf-tm" style="font-size:14px">0.0</text><line x1="70" y1="261.8" x2="690" y2="261.8" class="bf-grid"/><text x="62" y="265.8" text-anchor="end" class="bf-tm" style="font-size:14px">0.5</text><line x1="70" y1="193.6" x2="690" y2="193.6" class="bf-grid"/><text x="62" y="197.6" text-anchor="end" class="bf-tm" style="font-size:14px">1.0</text><line x1="70" y1="125.5" x2="690" y2="125.5" class="bf-grid"/><text x="62" y="129.5" text-anchor="end" class="bf-tm" style="font-size:14px">1.5</text><line x1="70" y1="57.3" x2="690" y2="57.3" class="bf-grid"/><text x="62" y="61.3" text-anchor="end" class="bf-tm" style="font-size:14px">2.0</text><text x="101.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">1</text><text x="132.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">2</text><text x="194.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">4</text><text x="318.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">8</text><text x="380.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">10</text><text x="442.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">12</text><text x="566.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">16</text><text x="690.0" y="348" text-anchor="middle" class="bf-tm" style="font-size:14px">20</text><text x="380.0" y="368" text-anchor="middle" class="bf-t" style="font-size:15px">threads asked for</text><text x="18" y="180.0" text-anchor="middle" transform="rotate(-90 18 180.0)" class="bf-t" style="font-size:15px">GB of output a second</text><line x1="411.0" y1="30" x2="411.0" y2="330" class="bf-grid"/><text x="405.0" y="42" text-anchor="end" class="bf-tm" style="font-size:14px">pinned, one CPU a core</text><text x="417.0" y="42" class="bf-tm" style="font-size:14px">whole machine</text><polyline points="101.0,275.1 132.0,233.4 194.0,189.6 318.0,152.1 380.0,156.3 442.0,155.4 566.0,156.8 690.0,157.5" style="fill:none;stroke:var(--bf-muted);stroke-width:2;stroke-linejoin:round;stroke-dasharray:4 3"/><circle cx="101.0" cy="275.1" r="2.2" style="fill:var(--bf-muted)"/><circle cx="132.0" cy="233.4" r="2.2" style="fill:var(--bf-muted)"/><circle cx="194.0" cy="189.6" r="2.2" style="fill:var(--bf-muted)"/><circle cx="318.0" cy="152.1" r="2.2" style="fill:var(--bf-muted)"/><circle cx="380.0" cy="156.3" r="2.2" style="fill:var(--bf-muted)"/><circle cx="442.0" cy="155.4" r="2.2" style="fill:var(--bf-muted)"/><circle cx="566.0" cy="156.8" r="2.2" style="fill:var(--bf-muted)"/><circle cx="690.0" cy="157.5" r="2.2" style="fill:var(--bf-muted)"/><polyline points="101.0,255.9 132.0,209.7 194.0,133.9 318.0,71.0 380.0,94.8 442.0,107.1 566.0,105.0 690.0,106.4" style="fill:none;stroke:var(--bf-amber);stroke-width:2;stroke-linejoin:round"/><circle cx="101.0" cy="255.9" r="2.2" style="fill:var(--bf-amber)"/><circle cx="132.0" cy="209.7" r="2.2" style="fill:var(--bf-amber)"/><circle cx="194.0" cy="133.9" r="2.2" style="fill:var(--bf-amber)"/><circle cx="318.0" cy="71.0" r="2.2" style="fill:var(--bf-amber)"/><circle cx="380.0" cy="94.8" r="2.2" style="fill:var(--bf-amber)"/><circle cx="442.0" cy="107.1" r="2.2" style="fill:var(--bf-amber)"/><circle cx="566.0" cy="105.0" r="2.2" style="fill:var(--bf-amber)"/><circle cx="690.0" cy="106.4" r="2.2" style="fill:var(--bf-amber)"/><polyline points="101.0,259.2 132.0,211.2 194.0,140.4 318.0,67.9 380.0,77.8 442.0,90.9 566.0,90.7 690.0,93.3" style="fill:none;stroke:var(--bf-steel);stroke-width:2.2;stroke-linejoin:round;stroke-dasharray:7 4"/><circle cx="101.0" cy="259.2" r="2.2" style="fill:var(--bf-steel)"/><circle cx="132.0" cy="211.2" r="2.2" style="fill:var(--bf-steel)"/><circle cx="194.0" cy="140.4" r="2.2" style="fill:var(--bf-steel)"/><circle cx="318.0" cy="67.9" r="2.2" style="fill:var(--bf-steel)"/><circle cx="380.0" cy="77.8" r="2.2" style="fill:var(--bf-steel)"/><circle cx="442.0" cy="90.9" r="2.2" style="fill:var(--bf-steel)"/><circle cx="566.0" cy="90.7" r="2.2" style="fill:var(--bf-steel)"/><circle cx="690.0" cy="93.3" r="2.2" style="fill:var(--bf-steel)"/><polyline points="101.0,256.1 132.0,212.7 194.0,136.4 318.0,69.6 380.0,82.4 442.0,61.5 566.0,63.0 690.0,60.5" style="fill:none;stroke:var(--bf-blue);stroke-width:2.2;stroke-linejoin:round;stroke-dasharray:2 4"/><circle cx="101.0" cy="256.1" r="2.2" style="fill:var(--bf-blue)"/><circle cx="132.0" cy="212.7" r="2.2" style="fill:var(--bf-blue)"/><circle cx="194.0" cy="136.4" r="2.2" style="fill:var(--bf-blue)"/><circle cx="318.0" cy="69.6" r="2.2" style="fill:var(--bf-blue)"/><circle cx="380.0" cy="82.4" r="2.2" style="fill:var(--bf-blue)"/><circle cx="442.0" cy="61.5" r="2.2" style="fill:var(--bf-blue)"/><circle cx="566.0" cy="63.0" r="2.2" style="fill:var(--bf-blue)"/><circle cx="690.0" cy="60.5" r="2.2" style="fill:var(--bf-blue)"/><polyline points="101.0,255.7 132.0,211.4 194.0,130.9 318.0,56.9 380.0,49.7 442.0,59.7 566.0,60.5 690.0,64.2" style="fill:none;stroke:var(--bf-green);stroke-width:3;stroke-linejoin:round"/><circle cx="101.0" cy="255.7" r="3" style="fill:var(--bf-green)"/><circle cx="132.0" cy="211.4" r="3" style="fill:var(--bf-green)"/><circle cx="194.0" cy="130.9" r="3" style="fill:var(--bf-green)"/><circle cx="318.0" cy="56.9" r="3" style="fill:var(--bf-green)"/><circle cx="380.0" cy="49.7" r="3" style="fill:var(--bf-green)"/><circle cx="442.0" cy="59.7" r="3" style="fill:var(--bf-green)"/><circle cx="566.0" cy="60.5" r="3" style="fill:var(--bf-green)"/><circle cx="690.0" cy="64.2" r="3" style="fill:var(--bf-green)"/><rect x="428.9" y="178.36363636363637" width="265.1" height="120" rx="6" class="bf-card"/><line x1="438.9" y1="192.36363636363637" x2="464.9" y2="192.36363636363637" style="stroke:var(--bf-muted);stroke-width:2;stroke-dasharray:4 3"/><text x="472.9" y="196.36363636363637" class="bf-t" style="font-size:15px">0.2.4 (6 November)</text><line x1="438.9" y1="214.36363636363637" x2="464.9" y2="214.36363636363637" style="stroke:var(--bf-amber);stroke-width:2"/><text x="472.9" y="218.36363636363637" class="bf-t" style="font-size:15px">0.5.0: the fall at its deepest</text><line x1="438.9" y1="236.36363636363637" x2="464.9" y2="236.36363636363637" style="stroke:var(--bf-steel);stroke-width:2.2;stroke-dasharray:7 4"/><text x="472.9" y="240.36363636363637" class="bf-t" style="font-size:15px">0.5.1</text><line x1="438.9" y1="258.3636363636364" x2="464.9" y2="258.3636363636364" style="stroke:var(--bf-blue);stroke-width:2.2;stroke-dasharray:2 4"/><text x="472.9" y="262.3636363636364" class="bf-t" style="font-size:15px">0.5.3</text><line x1="438.9" y1="280.3636363636364" x2="464.9" y2="280.3636363636364" style="stroke:var(--bf-green);stroke-width:3"/><text x="472.9" y="284.3636363636364" class="bf-t" style="font-size:15px">0.5.4: best at 10</text></svg>
<figcaption>Measured on the test machine: the median of 10 paired rounds per point, decoding an 11.9 MB extract of a Wikidata dump to 209.7 MB of output. Each release is taken from its own <a href="https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/">release-scaling run</a> (0.2.4 from 6 November; 0.5.0, 0.5.1 from 16 November; 0.5.3, 0.5.4 from 17 November). The runs pin 10 threads or fewer to one CPU a core, and give 12 threads or more the whole machine.</figcaption>
</figure>

**The result is a flatter *default* curve, not the disappearance of the knee or of contention.** The decoder
now starts fewer workers than threads in the measured configurations where extra workers made
decoding slower. In effect, 0.5.4 caps its workers at the number that pays off: seven, from ten
requested threads.
The cost of competing for the cache remains; it is now part of the plan rather than a surprise in
the benchmark. Here is how we reached that result, what the release run establishes, and what the
plateau still leaves open.

## 1. What was the knee?

A bzip2 file is made of blocks that can be decoded independently. mibz2 has two ways to spend
its threads. In a *team*, several threads share the work of one block. With *workers*, each
thread decodes a block of its own. The second arrangement can keep more cores busy, but it also
puts more blocks' working data in the shared last-level cache (the L3 on the test machine).

The first scaling investigation measured the trade-off directly. On the test machine, each
additional block in progress slowed the others. Three teams of three decoded faster than four
teams of two. Version 0.2.5 adopted that smaller number of teams and gained about 9% at ten
threads against 0.2.4, in two registered runs.

!!! info "Levelling off is not the same as falling"

    The old **shipped** curve mostly *levelled off*: version 0.2.4 ran four teams of two from
    eight requested threads upwards, instead of allowing concurrency to keep growing. The falling
    side of the knee was visible when experiments forced too many blocks to run at once. The
    policy avoided that fall by leaving extra threads unused. It had contained the problem, not
    made more concurrency productive.

The kernel changed next. [Cells](2026-11-10-rauzy-veech-reconstruction.md) reconstructed the
text from compact rules rather than the old row-by-row navigation table.
[Lazy letters](2026-11-16-lazy-letters.md) then kept working state for only the letters that
needed it, reducing the amount each worker touched. Those gains reached the workers, which ran below
ten threads, but not the teams, which still ran from ten. So by 0.5.0 the shipped curve
really did fall: eight threads ran eight workers, ten or more ran three teams of three, and 12 to
20 threads took 16–18% longer than 8. In 0.5.1, workers with lazy letters displaced teams at ten
threads and above. What was now faster than teams could still become slower if too many workers
ran together. That was the next knee to control.

<figure class="bf-fig">
<p class="bf-title">What mibz2 runs at each thread count, release by release</p>
<svg viewBox="0 0 720 362" role="img" aria-label="The plan mibz2 runs at 8, 10 and 12 to 20 threads, release by release: teams in 0.2.4 and above 8 threads in 0.5.0; one-block workers everywhere from 0.5.1, trimmed to 9 and then 7 past the cores, and to 7 at 10 threads and 6 at 8 when pinned in 0.5.4."><text x="241.0" y="20.0" text-anchor="middle" class="bf-ts" style="font-size:16px">8 threads</text><text x="431.0" y="20.0" text-anchor="middle" class="bf-ts" style="font-size:16px">10 threads</text><text x="621.0" y="20.0" text-anchor="middle" class="bf-ts" style="font-size:16px">12 to 20 threads</text><text x="0.0" y="55.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.2.4</text><text x="0.0" y="72.0" text-anchor="start" class="bf-tm" style="font-size:14px">6 November</text><rect x="150" y="34" width="182" height="48" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:0.8"/><text x="241.0" y="55.0" text-anchor="middle" class="bf-ts" style="font-size:16px">4</text><text x="241.0" y="72.0" text-anchor="middle" class="bf-tm" style="font-size:14px">teams of 2</text><rect x="340" y="34" width="182" height="48" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:0.8"/><text x="431.0" y="55.0" text-anchor="middle" class="bf-ts" style="font-size:16px">4</text><text x="431.0" y="72.0" text-anchor="middle" class="bf-tm" style="font-size:14px">teams of 2</text><rect x="530" y="34" width="182" height="48" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:0.8"/><text x="621.0" y="55.0" text-anchor="middle" class="bf-ts" style="font-size:16px">4</text><text x="621.0" y="72.0" text-anchor="middle" class="bf-tm" style="font-size:14px">teams of 2</text><text x="0.0" y="109.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.5.0</text><text x="0.0" y="126.0" text-anchor="start" class="bf-tm" style="font-size:14px">16 November</text><rect x="150" y="88" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="241.0" y="109.0" text-anchor="middle" class="bf-ts" style="font-size:16px">8</text><text x="241.0" y="126.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="340" y="88" width="182" height="48" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:2.5"/><text x="431.0" y="109.0" text-anchor="middle" class="bf-ts" style="font-size:16px">3</text><text x="431.0" y="126.0" text-anchor="middle" class="bf-tm" style="font-size:14px">teams of 3</text><rect x="530" y="88" width="182" height="48" rx="7" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:2.5"/><text x="621.0" y="109.0" text-anchor="middle" class="bf-ts" style="font-size:16px">3</text><text x="621.0" y="126.0" text-anchor="middle" class="bf-tm" style="font-size:14px">teams of 3</text><text x="0.0" y="163.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.5.1</text><text x="0.0" y="180.0" text-anchor="start" class="bf-tm" style="font-size:14px">#462, #468</text><rect x="150" y="142" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:0.8"/><text x="241.0" y="163.0" text-anchor="middle" class="bf-ts" style="font-size:16px">8</text><text x="241.0" y="180.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="340" y="142" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="431.0" y="163.0" text-anchor="middle" class="bf-ts" style="font-size:16px">10</text><text x="431.0" y="180.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="530" y="142" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="621.0" y="163.0" text-anchor="middle" class="bf-ts" style="font-size:16px">10</text><text x="621.0" y="180.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers, one a core</text><text x="0.0" y="217.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.5.2</text><text x="0.0" y="234.0" text-anchor="start" class="bf-tm" style="font-size:14px">#481</text><rect x="150" y="196" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:0.8"/><text x="241.0" y="217.0" text-anchor="middle" class="bf-ts" style="font-size:16px">8</text><text x="241.0" y="234.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="340" y="196" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:0.8"/><text x="431.0" y="217.0" text-anchor="middle" class="bf-ts" style="font-size:16px">10</text><text x="431.0" y="234.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="530" y="196" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="621.0" y="217.0" text-anchor="middle" class="bf-ts" style="font-size:16px">9</text><text x="621.0" y="234.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><text x="0.0" y="271.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.5.3</text><text x="0.0" y="288.0" text-anchor="start" class="bf-tm" style="font-size:14px">#524</text><rect x="150" y="250" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:0.8"/><text x="241.0" y="271.0" text-anchor="middle" class="bf-ts" style="font-size:16px">8</text><text x="241.0" y="288.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="340" y="250" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="431.0" y="271.0" text-anchor="middle" class="bf-ts" style="font-size:16px">10 / 7</text><text x="431.0" y="288.0" text-anchor="middle" class="bf-tm" style="font-size:14px">pinned / whole machine</text><rect x="530" y="250" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="621.0" y="271.0" text-anchor="middle" class="bf-ts" style="font-size:16px">7</text><text x="621.0" y="288.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><text x="0.0" y="325.0" text-anchor="start" class="bf-ts" style="font-size:16px">mibz2 0.5.4</text><text x="0.0" y="342.0" text-anchor="start" class="bf-tm" style="font-size:14px">#537</text><rect x="150" y="304" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="241.0" y="325.0" text-anchor="middle" class="bf-ts" style="font-size:16px">6 / 8</text><text x="241.0" y="342.0" text-anchor="middle" class="bf-tm" style="font-size:14px">pinned / whole machine</text><rect x="340" y="304" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:2.5"/><text x="431.0" y="325.0" text-anchor="middle" class="bf-ts" style="font-size:16px">7</text><text x="431.0" y="342.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text><rect x="530" y="304" width="182" height="48" rx="7" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:0.8"/><text x="621.0" y="325.0" text-anchor="middle" class="bf-ts" style="font-size:16px">7</text><text x="621.0" y="342.0" text-anchor="middle" class="bf-tm" style="font-size:14px">workers</text></svg>
<figcaption>From the decoder's thread names, recorded in each release's release-scaling run, and the release notes. A team (amber) shares one block between several threads; a worker (green) decodes a whole block on its own. A thick border marks a cell that changed from the release before. Where a cell has two counts, the first is the plan when the process is pinned one CPU a core, which is how the release-scaling runs measure 10 threads or fewer, and the second the plan on the whole machine.</figcaption>
</figure>

## 2. What does the new curve look like?

The [release-scaling benchmark](../benchmarks/history.md) decodes the same Wikidata sample on
the test machine at each requested thread count. It reports the time to produce about 210 MB of
output. Lower is better. These are the medians from the 0.5.3 and 0.5.4 release runs:

| threads requested | 0.5.3, seconds | 0.5.4, seconds |
| ---: | ---: | ---: |
| 1 | 0.387 | 0.385 |
| 2 | 0.244 | 0.241 |
| 4 | 0.148 | 0.144 |
| 8 | 0.110 | 0.105 |
| **10** | **0.115** | **0.102** |
| 12 | 0.107 | 0.106 |
| 16 | 0.107 | 0.106 |
| 20 | 0.106 | 0.108 |

The runs of ten threads or fewer are pinned one logical CPU per physical core; the higher
counts use the whole machine. The paired version comparisons preserve placement within each
count.

The shape matters more than the last decimal place. In 0.5.3, asking for ten threads made the
decoder slower than asking for eight. At twelve and above it recovered. In 0.5.4, **ten threads
are the fastest point**, while twelve to twenty sit between 0.106 and 0.108 seconds. Across eight
to twenty threads that is **1.95–2.06 GB/s of decoded output**. Twenty is still about 5% slower
than ten by these medians. As the box below explains, most or all of that is a change of
placement, not of thread count.

The paired comparison is stronger evidence for the version change than the medians alone.
Across ten paired rounds, 0.5.3 took **1.123 times** 0.5.4's wall time at ten threads (95%
interval 1.094–1.152), or about 11% less elapsed time in 0.5.4. At eight threads the ratio was
**1.045** (1.038–1.053). The new version also used less processor time and peak memory at those
counts. At one, two and twelve to twenty threads, the release comparison detected no change. The
apparent improvement at four threads was too small to resolve from the run's variation.

These comparisons show a real improvement at eight and ten threads. They do not by themselves
establish that ten is statistically faster than every other count within 0.5.4; the table shows
the median ordering, not paired confidence intervals between different thread counts. The figure
below gives those within-release comparisons against eight threads, for selected releases since
0.2.4. It leaves out 0.2.5 to 0.3.3. The history between was not one continuous fall. In 0.2.5,
ten threads took 0.918 of eight threads' wall time (95% interval 0.910–0.927).

!!! info "The decline past ten is a change of placement"

    Paired round by round within 0.5.4's run, the higher counts took longer than ten threads:

    | threads | wall time against ten threads (95% interval) |
    | ---: | ---: |
    | 12 | 1.049 (1.022–1.076) |
    | 16 | 1.044 (1.025–1.064) |
    | 20 | 1.074 (1.031–1.117) |

    Past ten, extra threads would land on the cores' second hardware threads. But the run also
    changes placement at that point. Ten threads ran pinned to one logical CPU a core. Twelve to
    twenty ran on the whole machine, with the same seven workers. So the decline can't be put down
    to hyperthreading.

    The knee diagnostic did hold placement constant: it ran every count on the whole machine,
    in one session. Its medians, in seconds, with the interquartile range for four copies:

    | threads | four copies of the sample | one copy |
    | ---: | ---: | ---: |
    | 10 | 0.3661 (0.365–0.368) | 0.1013 |
    | 12 | 0.3657 (0.365–0.369) | 0.1009 |
    | 20 | 0.3664 (0.364–0.370) | 0.1010 |
 Its build falls between
    0.5.3 and 0.5.4, which run the same plan on the whole machine. A release-build sweep from
    eight to twenty threads, all on the whole machine, would confirm it for 0.5.4 itself.

<figure class="bf-fig">
<p class="bf-title">Placement held constant: faster up to the cores, level beyond</p>
<svg viewBox="0 0 720 372" role="img" aria-label="Throughput against threads asked for. With the whole machine at every count, mibz2 gets faster up to 10 threads and then holds level to 20, within half a per cent, on one copy and four copies of the sample. The release sweep, pinned up to 10 threads and on the whole machine from 12, peaks at 10 and is about 4 to 5% lower from 12 to 20."><line x1="80" y1="292.7" x2="690" y2="292.7" class="bf-grid"/><text x="72.0" y="297.7" text-anchor="end" class="bf-tm" style="font-size:14px">1.4</text><line x1="80" y1="238.2" x2="690" y2="238.2" class="bf-grid"/><text x="72.0" y="243.2" text-anchor="end" class="bf-tm" style="font-size:14px">1.6</text><line x1="80" y1="183.6" x2="690" y2="183.6" class="bf-grid"/><text x="72.0" y="188.6" text-anchor="end" class="bf-tm" style="font-size:14px">1.8</text><line x1="80" y1="129.1" x2="690" y2="129.1" class="bf-grid"/><text x="72.0" y="134.1" text-anchor="end" class="bf-tm" style="font-size:14px">2.0</text><line x1="80" y1="74.5" x2="690" y2="74.5" class="bf-grid"/><text x="72.0" y="79.5" text-anchor="end" class="bf-tm" style="font-size:14px">2.2</text><line x1="80" y1="20.0" x2="690" y2="20.0" class="bf-grid"/><text x="72.0" y="25.0" text-anchor="end" class="bf-tm" style="font-size:14px">2.4</text><line x1="80" y1="320" x2="690" y2="320" class="bf-axis"/><text x="147.8" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">4</text><text x="283.3" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">8</text><text x="351.1" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">10</text><text x="418.9" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">12</text><text x="554.4" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">16</text><text x="690.0" y="340.0" text-anchor="middle" class="bf-tm" style="font-size:14px">20</text><text x="385.0" y="362.0" text-anchor="middle" class="bf-t" style="font-size:15px">threads asked for</text><text x="20" y="170.0" text-anchor="middle" transform="rotate(-90 20 170.0)" class="bf-t" style="font-size:15px">GB of output a second (axis from 1.3)</text><line x1="385.0" y1="20" x2="385.0" y2="320" style="stroke:var(--bf-muted);stroke-width:1;stroke-dasharray:3 3"/><text x="391.0" y="303.6" text-anchor="start" class="bf-tm" style="font-size:13px">10 physical cores; hardware threads beyond</text><polyline points="147.8,110.2 283.3,65.6 351.1,49.7 418.9,48.9 690.0,50.2" style="fill:none;stroke:var(--bf-green);stroke-width:3;stroke-linejoin:round"/><line x1="147.8" y1="117.8" x2="147.8" y2="107.6" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="147.8" cy="110.2" r="4" style="fill:var(--bf-green)"/><line x1="283.3" y1="72.7" x2="283.3" y2="63.4" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="283.3" cy="65.6" r="4" style="fill:var(--bf-green)"/><line x1="351.1" y1="52.7" x2="351.1" y2="48.7" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="351.1" cy="49.7" r="4" style="fill:var(--bf-green)"/><line x1="418.9" y1="54.2" x2="418.9" y2="47.3" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="418.9" cy="48.9" r="4" style="fill:var(--bf-green)"/><line x1="690.0" y1="55.5" x2="690.0" y2="46.0" style="stroke:var(--bf-green);stroke-width:2.5"/><circle cx="690.0" cy="50.2" r="4" style="fill:var(--bf-green)"/><polyline points="147.8,172.4 283.3,120.6 351.1,110.3 418.9,107.7 690.0,108.0" style="fill:none;stroke:var(--bf-steel);stroke-width:2.2;stroke-linejoin:round"/><line x1="147.8" y1="181.4" x2="147.8" y2="168.2" style="stroke:var(--bf-steel);stroke-width:2.5"/><circle cx="147.8" cy="172.4" r="4" style="fill:var(--bf-steel)"/><line x1="283.3" y1="125.0" x2="283.3" y2="117.3" style="stroke:var(--bf-steel);stroke-width:2.5"/><circle cx="283.3" cy="120.6" r="4" style="fill:var(--bf-steel)"/><line x1="351.1" y1="116.0" x2="351.1" y2="104.6" style="stroke:var(--bf-steel);stroke-width:2.5"/><circle cx="351.1" cy="110.3" r="4" style="fill:var(--bf-steel)"/><line x1="418.9" y1="115.1" x2="418.9" y2="101.9" style="stroke:var(--bf-steel);stroke-width:2.5"/><circle cx="418.9" cy="107.7" r="4" style="fill:var(--bf-steel)"/><line x1="690.0" y1="111.6" x2="690.0" y2="101.5" style="stroke:var(--bf-steel);stroke-width:2.5"/><circle cx="690.0" cy="108.0" r="4" style="fill:var(--bf-steel)"/><polyline points="147.8,276.3 283.3,128.3 351.1,113.8 418.9,134.0 554.4,135.5 690.0,143.0" style="fill:none;stroke:var(--bf-amber);stroke-width:2;stroke-linejoin:round;stroke-dasharray:6 4"/><circle cx="147.8" cy="276.3" r="4" style="fill:var(--bf-amber)"/><circle cx="283.3" cy="128.3" r="4" style="fill:var(--bf-amber)"/><circle cx="351.1" cy="113.8" r="4" style="fill:var(--bf-amber)"/><circle cx="418.9" cy="134.0" r="4.5" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:2"/><circle cx="554.4" cy="135.5" r="4.5" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:2"/><circle cx="690.0" cy="143.0" r="4.5" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:2"/><rect x="395.3333333333333" y="169.09090909090907" width="302.6666666666667" height="96" rx="6" class="bf-card"/><line x1="405.3333333333333" y1="184.09090909090907" x2="431.3333333333333" y2="184.09090909090907" style="stroke:var(--bf-green);stroke-width:3"/><text x="439.3" y="189.1" text-anchor="start" class="bf-t" style="font-size:15px">whole machine, four copies</text><line x1="405.3333333333333" y1="208.09090909090907" x2="431.3333333333333" y2="208.09090909090907" style="stroke:var(--bf-steel);stroke-width:2.2"/><text x="439.3" y="213.1" text-anchor="start" class="bf-t" style="font-size:15px">whole machine, one copy</text><line x1="405.3333333333333" y1="232.09090909090907" x2="431.3333333333333" y2="232.09090909090907" style="stroke:var(--bf-amber);stroke-width:2;stroke-dasharray:6 4"/><text x="439.3" y="237.1" text-anchor="start" class="bf-t" style="font-size:15px">release sweep, one copy</text><text x="405.3" y="259.1" text-anchor="start" class="bf-tm" style="font-size:13px">hollow: whole machine; filled: pinned</text></svg>
<figcaption>Measured on the test machine. Solid lines: the <a href="https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T123939Z.tar.gz">knee diagnostic</a> of 17 November, every count on the whole machine in one session, 10 rounds a point; the dots are medians and the bars the interquartile range, decoding one copy or four copies of the Wikidata sample. Its build (399d33d6) falls between 0.5.3 and 0.5.4; on the whole machine both releases run the same plan, 4 and 8 workers at 4 and 8 threads and 7 workers from 10. Dashed: 0.5.4's release sweep, medians of 10 rounds, pinned one CPU a core up to 10 threads (filled) and on the whole machine from 12 (hollow). The vertical axis starts at 1.3 GB/s.</figcaption>
</figure>

<figure class="bf-fig">
<p class="bf-title">The fall past eight threads, release by release</p>
<svg viewBox="0 0 720 364" role="img" aria-label="For each release, the wall time at 12, 16 and 20 threads against the same release at 8 threads: about 2 to 3% slower in 0.2.4, 16 to 18% slower in 0.5.0, then 10%, 5 to 7%, about 3% faster, and 1 to 4% slower in 0.5.4."><line x1="219.7" y1="38" x2="219.7" y2="318" class="bf-grid"/><text x="219.7" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">−5%</text><line x1="302.5" y1="38" x2="302.5" y2="318" class="bf-axis"/><text x="302.5" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">as fast as 8</text><line x1="385.3" y1="38" x2="385.3" y2="318" class="bf-grid"/><text x="385.3" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">+5%</text><line x1="468.1" y1="38" x2="468.1" y2="318" class="bf-grid"/><text x="468.1" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">+10%</text><line x1="550.9" y1="38" x2="550.9" y2="318" class="bf-grid"/><text x="550.9" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">+15%</text><line x1="633.7" y1="38" x2="633.7" y2="318" class="bf-grid"/><text x="633.7" y="334.0" text-anchor="middle" class="bf-tm" style="font-size:14px">+20%</text><text x="435.0" y="354.0" text-anchor="middle" class="bf-t" style="font-size:15px">wall time against the same release at 8 threads</text><circle cx="174" cy="14" r="4.5" style="fill:var(--bf-amber)"/><text x="184.0" y="18.0" text-anchor="start" class="bf-tm" style="font-size:14px">12, 16 and 20 threads, with 95% intervals</text><circle cx="500" cy="14" r="4.5" style="fill:var(--bf-surface);stroke:var(--bf-ink);stroke-width:1.5"/><text x="510.0" y="18.0" text-anchor="start" class="bf-tm" style="font-size:14px">10 threads</text><rect x="0" y="46.0" width="720" height="34" class="bf-track" style="opacity:0.45"/><text x="0.0" y="68.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.2.4</text><text x="60.0" y="68.0" text-anchor="start" class="bf-tm" style="font-size:14px">6 Nov</text><line x1="311.8" y1="56.0" x2="355.4" y2="56.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="333.5" cy="56.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="313.7" y1="63.0" x2="386.8" y2="63.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="349.9" cy="63.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="325.5" y1="70.0" x2="389.0" y2="70.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="357.0" cy="70.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="335.3" cy="63.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><text x="0.0" y="102.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.3.4</text><text x="60.0" y="102.0" text-anchor="start" class="bf-tm" style="font-size:14px">15 Nov</text><line x1="269.9" y1="90.0" x2="379.7" y2="90.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="323.9" cy="90.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="292.2" y1="97.0" x2="360.2" y2="97.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="325.9" cy="97.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="321.5" y1="104.0" x2="371.6" y2="104.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="346.3" cy="104.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="245.8" cy="97.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><rect x="0" y="114.0" width="720" height="34" class="bf-track" style="opacity:0.45"/><text x="0.0" y="136.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.4.0</text><text x="60.0" y="136.0" text-anchor="start" class="bf-tm" style="font-size:14px">16 Nov</text><line x1="447.9" y1="124.0" x2="527.6" y2="124.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="487.3" cy="124.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="463.7" y1="131.0" x2="542.1" y2="131.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="502.5" cy="131.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="470.4" y1="138.0" x2="544.4" y2="138.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="507.0" cy="138.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="395.1" cy="131.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><text x="0.0" y="170.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.5.0</text><text x="60.0" y="170.0" text-anchor="start" class="bf-tm" style="font-size:14px">16 Nov</text><line x1="548.6" y1="158.0" x2="656.6" y2="158.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="601.8" cy="158.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="549.8" y1="165.0" x2="597.8" y2="165.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="573.7" cy="165.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="551.9" y1="172.0" x2="612.2" y2="172.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="581.8" cy="172.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="476.1" cy="165.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><rect x="0" y="182.0" width="720" height="34" class="bf-track" style="opacity:0.45"/><text x="0.0" y="204.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.5.1</text><text x="60.0" y="204.0" text-anchor="start" class="bf-tm" style="font-size:14px">16 Nov</text><line x1="416.2" y1="192.0" x2="511.4" y2="192.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="463.1" cy="192.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="435.5" y1="199.0" x2="494.7" y2="199.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="464.8" cy="199.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="449.4" y1="206.0" x2="522.2" y2="206.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="485.4" cy="206.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="374.0" cy="199.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><text x="0.0" y="238.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.5.2</text><text x="60.0" y="238.0" text-anchor="start" class="bf-tm" style="font-size:14px">16 Nov</text><line x1="364.8" y1="226.0" x2="408.2" y2="226.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="386.4" cy="226.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="364.2" y1="233.0" x2="486.8" y2="233.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="424.4" cy="233.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="355.7" y1="240.0" x2="448.7" y2="240.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="401.6" cy="240.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="365.3" cy="233.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><rect x="0" y="250.0" width="720" height="34" class="bf-track" style="opacity:0.45"/><text x="0.0" y="272.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.5.3</text><text x="60.0" y="272.0" text-anchor="start" class="bf-tm" style="font-size:14px">17 Nov</text><line x1="207.9" y1="260.0" x2="283.9" y2="260.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="245.4" cy="260.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="209.2" y1="267.0" x2="306.7" y2="267.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="257.2" cy="267.0" r="3.6" style="fill:var(--bf-amber)"/><line x1="224.9" y1="274.0" x2="279.7" y2="274.0" style="stroke:var(--bf-amber);stroke-width:1.5"/><circle cx="252.1" cy="274.0" r="3.6" style="fill:var(--bf-amber)"/><circle cx="380.3" cy="267.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/><text x="0.0" y="306.0" text-anchor="start" class="bf-ts" style="font-size:16px">0.5.4</text><text x="60.0" y="306.0" text-anchor="start" class="bf-tm" style="font-size:14px">17 Nov</text><line x1="286.0" y1="294.0" x2="380.8" y2="294.0" style="stroke:var(--bf-green);stroke-width:1.5"/><circle cx="332.7" cy="294.0" r="3.6" style="fill:var(--bf-green)"/><line x1="313.0" y1="301.0" x2="337.6" y2="301.0" style="stroke:var(--bf-green);stroke-width:1.5"/><circle cx="325.3" cy="301.0" r="3.6" style="fill:var(--bf-green)"/><line x1="300.9" y1="308.0" x2="447.9" y2="308.0" style="stroke:var(--bf-green);stroke-width:1.5"/><circle cx="372.8" cy="308.0" r="3.6" style="fill:var(--bf-green)"/><circle cx="254.5" cy="301.0" r="4.5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/></svg>
<figcaption>Measured on the test machine. Each mark is a release's wall time at 10, 12, 16 or 20 threads divided by its own wall time at 8 threads, as the geometric mean of 10 rounds, paired by round, with its 95% interval across those rounds (one run, one machine, one file). Right of the axis is slower than 8 threads. 0.4.0 comes from 0.5.0's run, where it was the older release. 0.5.3's 10-thread mark is its pinned plan of 10 workers. Selected releases: 0.2.5 to 0.3.3 are not shown. Sources: the <a href="https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling/">release-scaling runs</a>.</figcaption>
</figure>

## 3. Why did ten threads dip, and what fixed it?

The ten-thread regression in 0.5.3 was a placement mistake in the planner. Version 0.5.3 tried
to recognise the test machine from the CPUs the *process was allowed to use*. With the process
pinned to ten CPUs, it no longer recognised the full machine and declined to apply its
seven-worker limit. It started **ten workers**, with forty blocks allowed in flight.

!!! info "How the release runs place their threads"

    The release harness pins runs of ten threads or fewer to one logical CPU per physical core.
    At ten requested threads, the process may use CPUs 0–9: one logical CPU from each physical
    core. Its workers, scanner and reader share that set. Runs of twelve threads or more may use
    all 20 logical CPUs. Pinning is how mibz2 runs under `taskset` or in a container limited
    to some CPUs; the whole machine is how it usually runs.

Unpinned, 0.5.3 did apply the limit: a later check of production on the whole machine found it
building seven workers in all six runs. That check, run after 0.5.3 shipped, also timed seven
workers against ten directly, with the counts set explicitly, and seven took 0.873 of ten
workers' time (95% interval 0.866–0.880). The release benchmark's ten-thread dip was therefore
not evidence that the seven-worker idea failed. It was evidence that the intended plan had not
run.

Version 0.5.4 [separates the two questions](https://github.com/lmmx/mibz2/pull/537). It
identifies the machine from all present and online CPUs, but still reads the process's affinity
to decide which measured plan applies. Pinned to ten cores, it now starts **seven lazy-letters
workers**, with 28 blocks in flight. Pinned to eight, it starts six, and pinned to six and four
it trims one worker as well. Those three pinned trims did not carry over to the whole machine: in
the same check, six workers at eight threads took 1.000 of eight workers' time, the six-thread
trim was no faster, and the four-thread trim took 1.21 times as long. Competition for CPUs with the decoder's scanner and
reader, which share the workers' cores when the process is pinned, is implicated, so 0.5.4 applies
those trims only when pinned. The ten-thread trim holds in both modes; why exactly it pays is not
settled.

The machine-wide check is cached once per process. An early release check caught repeated
hardware discovery adding overhead to short decodes; that was fixed before 0.5.4 shipped. The
result is a narrower policy than a universal formula: it applies to the test machine and its
tested placement modes, not automatically to every ten-core processor.

## 4. Does the plateau mean memory bandwidth is full?

Nearly, but not yet. The plateau uses 75% of the memory bandwidth the machine can sustain, and
every worker past seven adds traffic faster than it adds output.

With ten workers reduced to seven, the
[worker-count study](2026-11-17-modelling-the-cache-to-rightsize-workers.md) found about **half
as many last-level-cache misses per decoded byte** and **25% fewer cycles per byte**.
Instructions and clock frequency stayed approximately level.

At ten workers, each was busy about 76% of the time; at seven, each was busy about 92%. The
smaller group offered less aggregate worker time, yet it decoded each byte much more cheaply.
That is why removing workers can make a parallel program finish sooner. The utilisation
arithmetic alone does not establish the cause; the cache counters supply the evidence for
interference.

A diagnostic run on the plateau itself rules out the simplest alternatives. Decoding four copies of the
sample on the whole machine, the workers compute 93–95% of the time at every thread count, so no serial stage, such as
the scanner that reads the file or the reader that hands bytes back in order, is starving them.
What rises with the workers is memory traffic. Counted at the memory controller, reads and writes
come to 7.5 bytes for every byte of output at four workers, 11.0 at seven and 12.4 at eight.

<figure class="bf-fig">
<p class="bf-title">More workers, more memory traffic for every byte decoded</p>
<svg viewBox="0 0 720 454" role="img" aria-label="Bytes of memory traffic per byte of output rise with the number of workers: on the current decoder, memory reads and writes go from 7.5 bytes at 4 workers to 11.0 at 7, 12.4 at 8 and 13.9 at 10 pinned; on an older decoder, memory reads and writes together went from 15.5 bytes at 4 workers to 22.7 at 8 and 24.2 at 10."><rect x="0" y="0" width="720" height="72" rx="8" class="bf-card"/><text x="14.0" y="22.0" text-anchor="start" class="bf-t" style="font-size:15px">The analogy: each worker is a cook at one shared worktop, the 20 MiB L3 cache.</text><text x="14.0" y="42.0" text-anchor="start" class="bf-t" style="font-size:15px">Whatever does not fit on the worktop is fetched from the pantry, main memory.</text><text x="14.0" y="62.0" text-anchor="start" class="bf-tm" style="font-size:14px">More cooks crowd the worktop, so every dish needs more trips to the pantry.</text><line x1="60" y1="330.0" x2="700" y2="330.0" class="bf-axis"/><text x="52.0" y="334.0" text-anchor="end" class="bf-tm" style="font-size:14px">0</text><line x1="60" y1="286.5" x2="700" y2="286.5" class="bf-grid"/><text x="52.0" y="290.5" text-anchor="end" class="bf-tm" style="font-size:14px">5</text><line x1="60" y1="243.1" x2="700" y2="243.1" class="bf-grid"/><text x="52.0" y="247.1" text-anchor="end" class="bf-tm" style="font-size:14px">10</text><line x1="60" y1="199.6" x2="700" y2="199.6" class="bf-grid"/><text x="52.0" y="203.6" text-anchor="end" class="bf-tm" style="font-size:14px">15</text><line x1="60" y1="156.2" x2="700" y2="156.2" class="bf-grid"/><text x="52.0" y="160.2" text-anchor="end" class="bf-tm" style="font-size:14px">20</text><line x1="60" y1="112.7" x2="700" y2="112.7" class="bf-grid"/><text x="52.0" y="116.7" text-anchor="end" class="bf-tm" style="font-size:14px">25</text><text x="92.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">1</text><text x="156.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">2</text><text x="220.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">3</text><text x="284.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">4</text><text x="348.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">5</text><text x="412.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">6</text><text x="476.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">7</text><text x="540.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">8</text><text x="604.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">9</text><text x="668.0" y="348.0" text-anchor="middle" class="bf-tm" style="font-size:14px">10</text><text x="380.0" y="368.0" text-anchor="middle" class="bf-t" style="font-size:15px">workers decoding at once</text><text x="14.0" y="217.0" text-anchor="middle" class="bf-t" style="font-size:15px" transform="rotate(-90 14 217.0)">bytes moved per byte of output</text><polyline points="92.0,319.1 156.0,279.5 220.0,230.0 284.0,194.9 412.0,152.5 540.0,132.7 668.0,119.7" style="fill:none;stroke:var(--bf-amber);stroke-width:2;stroke-linejoin:round"/><polyline points="92.0,324.4 156.0,303.8 220.0,278.4 284.0,258.8 412.0,234.3 540.0,219.4 668.0,209.4" style="fill:none;stroke:var(--bf-amber);stroke-width:1.5;stroke-linejoin:round;stroke-dasharray:5 4"/><circle cx="92.0" cy="319.1" r="3" style="fill:var(--bf-amber)"/><circle cx="156.0" cy="279.5" r="3" style="fill:var(--bf-amber)"/><circle cx="220.0" cy="230.0" r="3" style="fill:var(--bf-amber)"/><circle cx="284.0" cy="194.9" r="3" style="fill:var(--bf-amber)"/><circle cx="412.0" cy="152.5" r="3" style="fill:var(--bf-amber)"/><circle cx="540.0" cy="132.7" r="3" style="fill:var(--bf-amber)"/><circle cx="668.0" cy="119.7" r="3" style="fill:var(--bf-amber)"/><circle cx="92.0" cy="324.4" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="156.0" cy="303.8" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="220.0" cy="278.4" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="284.0" cy="258.8" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="412.0" cy="234.3" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="540.0" cy="219.4" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><circle cx="668.0" cy="209.4" r="2.4" style="fill:var(--bf-surface);stroke:var(--bf-amber);stroke-width:1.2"/><polyline points="284.0,264.8 476.0,234.4 540.0,222.2" style="fill:none;stroke:var(--bf-green);stroke-width:3;stroke-linejoin:round"/><circle cx="284.0" cy="264.8" r="4.5" style="fill:var(--bf-green)"/><text x="292.0" y="280.8" text-anchor="start" class="bf-t-strong" style="font-size:15px">7.5</text><circle cx="476.0" cy="234.4" r="4.5" style="fill:var(--bf-green)"/><text x="484.0" y="250.4" text-anchor="start" class="bf-t-strong" style="font-size:15px">11.0</text><circle cx="540.0" cy="222.2" r="4.5" style="fill:var(--bf-green)"/><text x="548.0" y="238.2" text-anchor="start" class="bf-t-strong" style="font-size:15px">12.4</text><circle cx="668.0" cy="209.2" r="4.5" style="fill:var(--bf-surface);stroke:var(--bf-green);stroke-width:2"/><text x="658.0" y="199.2" text-anchor="end" class="bf-tm" style="font-size:14px">13.9, pinned</text><text x="660.0" y="111.7" text-anchor="end" class="bf-t-strong" style="font-size:15px">24.2</text><text x="540.0" y="122.7" text-anchor="middle" class="bf-t-strong" style="font-size:15px">22.7</text><line x1="60" y1="396" x2="84" y2="396" style="stroke:var(--bf-green);stroke-width:3"/><text x="92.0" y="400.0" text-anchor="start" class="bf-tm" style="font-size:14px">current decoder: memory reads + writes (17 November)</text><line x1="60" y1="418" x2="84" y2="418" style="stroke:var(--bf-amber);stroke-width:2"/><text x="92.0" y="422.0" text-anchor="start" class="bf-tm" style="font-size:14px">older decoder: memory reads + writes (14 November)</text><line x1="60" y1="440" x2="84" y2="440" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/><text x="92.0" y="444.0" text-anchor="start" class="bf-tm" style="font-size:14px">older decoder: memory reads only</text></svg>
<figcaption>Measured on the test machine at the memory controller, reads and writes per byte of output. Green: the current decoder, decoding four copies of the Wikidata extract, in the <a href="https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T230729Z.tar.gz">knee repeat with the memory-controller counters</a> of 17 November (the whole machine, except the hollow point, pinned). Amber: an older decoder that held a slot for every letter of a block, over 15 repeated decodes of the single Wikidata sample a reading (<a href="https://github.com/lmmx/mibz2/blob/evidence/cells-imc/run-20261114T180428Z.tar.gz">memory-controller run</a>, 14 November). The two runs used different decoders and workloads, so they are drawn side by side, not joined. The first knee diagnostic counted only reads that missed the L3, 6.9, 11.0 and 12.7 bytes at 4, 7 and 8 workers, close to the green totals. The analogy stops where caches differ from kitchens: data is not used up, it is copied in 64-byte lines.</figcaption>
</figure>

It is tempting to read the two-gigabyte-per-second plateau as a DRAM bandwidth limit, so we
measured it. A repeat of the diagnostic with the memory controller's own counters found the
seven-worker plateau moving 24.8 GB/s of DRAM traffic: 11.0 bytes per output byte, of which 7.0
are reads and 4.0 writes. A separate test that streams memory as fast as it can reaches 33.1 GB/s
on the same counters, so the plateau runs at 75% of the measured ceiling. On an older version of
the Cells kernel, on 14 November, the same counters measured 22.7 and 24.2 bytes per output byte
at eight and ten workers. The decoder has roughly halved its DRAM bytes per output byte since.

More workers push the traffic up faster than the output. Eight workers reach 83% of the ceiling
at 12.4 bytes per output byte, and ten pinned workers 85% at 13.9, and both decode more slowly
than seven. So bandwidth is close to binding, and every worker past seven spends it on traffic
rather than output. It is not the hard ceiling at seven: the registered rule called bandwidth
the limit only at 80% of the ceiling or more, and 75% leaves the question open.

<figure class="bf-fig">
<p class="bf-title">At the plateau: how close is memory traffic to its ceiling?</p>
<svg viewBox="0 0 720 232" role="img" aria-label="DRAM traffic measured at the memory controller: 24.8 GB a second with 7 workers, 75% of the measured 33.1 GB a second ceiling; 27.4 with 8 workers, 83%; 28.2 with 10 pinned workers, 85%. Both larger counts decode more slowly than 7."><text x="0.0" y="55.0" text-anchor="start" class="bf-ts" style="font-size:16px">7 workers: 24.8 GB/s, 75%</text><text x="0.0" y="72.0" text-anchor="start" class="bf-tm" style="font-size:14px">10–20 threads, whole machine</text><rect x="270" y="40" width="184.3" height="28" rx="3" class="bf-bar"/><rect x="454.3" y="40" width="105.0" height="28" rx="3" style="fill:var(--bf-steel)"/><text x="278.0" y="59.0" text-anchor="start" class="bf-t-strong" style="font-size:15px">reads 15.8</text><text x="462.3" y="59.0" text-anchor="start" class="bf-t-strong" style="font-size:15px">writes 9.0</text><text x="0.0" y="105.0" text-anchor="start" class="bf-ts" style="font-size:16px">8 workers: 27.4 GB/s, 83%</text><text x="0.0" y="122.0" text-anchor="start" class="bf-tm" style="font-size:14px">8 threads, whole machine</text><rect x="270" y="90" width="319.7" height="28" rx="3" style="fill:var(--bf-amber)"/><text x="0.0" y="155.0" text-anchor="start" class="bf-ts" style="font-size:16px">10 workers: 28.2 GB/s, 85%</text><text x="0.0" y="172.0" text-anchor="start" class="bf-tm" style="font-size:14px">10 threads, pinned</text><rect x="270" y="140" width="329.0" height="28" rx="3" style="fill:var(--bf-amber)"/><line x1="656.2" y1="24" x2="656.2" y2="176" style="stroke:var(--bf-ink);stroke-width:1.5;stroke-dasharray:4 3"/><text x="656.2" y="18.0" text-anchor="middle" class="bf-tm" style="font-size:14px">measured ceiling, 33.1</text><line x1="270.0" y1="180" x2="270.0" y2="186" class="bf-axis"/><text x="270.0" y="202.0" text-anchor="middle" class="bf-tm" style="font-size:14px">0</text><line x1="386.7" y1="180" x2="386.7" y2="186" class="bf-axis"/><text x="386.7" y="202.0" text-anchor="middle" class="bf-tm" style="font-size:14px">10</text><line x1="503.3" y1="180" x2="503.3" y2="186" class="bf-axis"/><text x="503.3" y="202.0" text-anchor="middle" class="bf-tm" style="font-size:14px">20</text><line x1="620.0" y1="180" x2="620.0" y2="186" class="bf-axis"/><text x="620.0" y="202.0" text-anchor="middle" class="bf-tm" style="font-size:14px">30</text><line x1="270" y1="180" x2="690" y2="180" class="bf-axis"/><text x="480.0" y="222.0" text-anchor="middle" class="bf-t" style="font-size:15px">GB a second of DRAM reads and writes, four copies of the sample</text></svg>
<figcaption>Measured on the test machine, decoding four copies of the Wikidata extract (<a href="https://github.com/lmmx/mibz2/blob/evidence/knee/run-20261117T230729Z.tar.gz">knee repeat with the memory-controller counters</a>, 17 November). Each bar is DRAM reads plus writes counted at the memory controller, the median of 10 rounds, with idle traffic subtracted. The ceiling is measured too: the best median of a separate test that streams memory as fast as it can (<code>ab stream</code>, its scale kernel), read on the same counters. At 7 workers the decoder writes 2.27 GB of output a second; at 8 and at 10 pinned workers it is slower than at 7, while moving more memory traffic.</figcaption>
</figure>

There is another reason to be precise about the word *plateau*. At twelve to twenty
**requested** threads on this machine, 0.5.4 still runs **seven decoding workers**. The decoder
is not starting twelve, sixteen or twenty competing workers; the plan has capped the number of
blocks being decoded simultaneously. A nearly flat result there means the planner has kept
the most expensive part of the curve out of the default path. It does not prove that the hardware would
scale if twenty independent decoding workers were set loose.

## 5. How would we straighten the knee?

Straightening the knee means a curve that keeps rising past the ten physical cores. The second
hardware thread on each core (simultaneous multithreading, which Intel calls hyperthreading)
would then add speed. Today it adds none. Three routes could change that. Each is a
hypothesis with a measurement that tests it; none is a result yet.

**Cheaper bytes per worker.** At seven workers, the reads that miss the L3 come to 11 bytes for
every byte of output. A smaller hot set per worker would cut that: the store and the tables the
kernel keeps for a block. So would less traffic on the output side. One piece of the
output side is folding the block's checksum into the final pass. That is an open research arm
([#535](https://github.com/lmmx/mibz2/pull/535)), and its measured 1.5–3% gain is not yet settled.
Fewer bytes moved per output byte raise the number of workers that pays, so more workers, and
eventually the second hardware threads, can contribute. With bandwidth close to binding, this is the
route that raises the ceiling. The test: bytes per output byte and wall time by worker count,
before and after each cut.

**Latency hiding with the second hardware threads.** The counters run at ten threads measured
about 2.2 reads that missed the L3 in flight at once, whenever any were. That is little
memory-level parallelism. If the plateau is bound by latency rather than bandwidth, two workers
sharing a core can overlap each other's misses: one computes while the other waits. That is the
classic case where hyperthreading helps code that waits on memory. The test: the same number of
workers placed as pairs on shared cores, against one worker a core.

**Helpers on the second hardware threads.** The decoder's other threads already gain from spare
CPUs. Decoding four copies of the sample with four threads took 0.405 seconds on the whole
machine, against 0.535 seconds pinned. Pinned, the scanner, reader and main thread share the
workers' cores. The second hardware threads could host prefetch or helper threads that bring the
next block's data into cache ahead of the workers. The test: a helper arm against none, at the
same worker count, with the misses per output byte counted.

**Which first.** The memory-controller run in §4 puts the plateau at 75% of the measured
bandwidth ceiling, with traffic per byte rising faster than output past seven workers. That makes
cheaper bytes per worker the main route: every byte saved per output byte is room for more
workers before the ceiling. Latency hiding with the second hardware threads stays open, since a
quarter of the bandwidth is still unused at seven workers.

## 6. What comes next for the knee?

The other task is to make the planner portable. The measured seven-worker choice is not a
general law. The cache model under test correctly recovered some counts on this machine, then
predicted the wrong optimum for smaller bzip2 blocks. Other processors have different core
counts, cache sizes, cache-sharing domains and memory systems. The right worker count needs to
follow those facts, with tests that can prove a proposed rule wrong.

For now the release result is a good one, with a precise boundary. **mibz2 no longer punishes a
request for more threads with a large fall in throughput on this measured workload and
machine.** In the release sweep the penalty past the best point is 4–5%, down from 11–18%. With placement
held constant, no decline is seen. It gets there by doing *less work in parallel*, more efficiently. The old knee exposed
a bad trade-off between concurrency and interference. Version 0.5.4 runs on the better side of
it.

What remains is a flat stretch past the cores to turn into a rising curve, a ceiling to raise,
and a policy to generalise. That is a more useful problem than a curve that falls whenever we give it another
core.

---

**Research records:**

- [The original eight-thread scaling investigation](2026-11-07-why-mibz2-stops-scaling.md),
  [lazy letters](2026-11-16-lazy-letters.md), and
  [the cache-model post](2026-11-17-modelling-the-cache-to-rightsize-workers.md) with its
  [technical notes](2026-11-17-modelling-the-cache-to-rightsize-workers-notes.md)
- [0.5.3's release-scaling run and the pinned ten-thread diagnosis](https://github.com/lmmx/mibz2/issues/145#issuecomment-6946756938),
  [the unpinned control](https://github.com/lmmx/mibz2/issues/145#issuecomment-6949008682),
  [the knee diagnostic](https://github.com/lmmx/mibz2/issues/145#issuecomment-6948385717) and
  [0.5.4's release-scaling run](https://github.com/lmmx/mibz2/issues/145#issuecomment-6952656453)
- [The worker-plan fix](https://github.com/lmmx/mibz2/pull/537),
  [the once-per-process machine read](https://github.com/lmmx/mibz2/pull/545), and the
  [release attribution ledger](https://github.com/lmmx/mibz2/blob/745a7381ab72ca07675ddfcb000c4156f926c0eb/docs/journal/release-attribution.md)
- The recorded runs: [release scaling](https://github.com/lmmx/mibz2/tree/745a7381ab72ca07675ddfcb000c4156f926c0eb/bench/release-scaling),
  and on the `evidence` branch the knee diagnostic (`knee/run-20261117T123939Z`), the unpinned
  control (`unpinned/run-20261117T130353Z`), the counters runs (`miss-cost/run-20261117T104648Z`)
  and the memory-controller runs (`cells-imc/run-20261114T180428Z` and `knee/run-20261117T230729Z`)
