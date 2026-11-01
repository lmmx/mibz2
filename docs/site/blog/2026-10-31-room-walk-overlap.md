---
title: "32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles"
image: assets/social/blog-2026-10-31-room-walk-overlap.png
description: "The decoder's 32-lane walk runs 8.6 times faster than one chain. That figure may not mean what it seems: the walk may already hide memory latency, and spend too many instructions on each byte."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# 32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles

The decoder's 32-lane walk runs 8.6 times faster than one chain. That figure may not mean
what it seems: the walk may already hide memory latency, and spend too many instructions on
each byte.
{: .post-dek }

31 October 2026 · 7 min read · [Open question](#6-what-would-count-as-a-breakthrough)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

This post asks how much parallel work the CPU actually does inside mibz2's inverse-BWT
decoder, and whether hardware capacity is lost to the way Rust compiles the loop.

The interesting part is that the suspected problem may not be too little parallelism at all.
The figures could instead mean that the CPU already hides memory latency well, but spends too
many instructions on each decoded byte.

## 1. Why does the BWT decoder have 32 lanes?

The inverse BWT repeatedly looks up the next position in a table. Simplifying the actual
code:

```rust
let q = t[row];
row = q >> 8;
output.push(q as u8);
```

The next lookup depends on the result of the previous one: r<sub>i+1</sub> = Ψ(r<sub>i</sub>).

The CPU cannot know where to look next until the current lookup completes. This is called
pointer chasing, and along one chain it is inherently serial.

mibz2's answer is to divide the inverse-BWT traversal into 512 segments and walk up to 32
independent segments at once. Each lane has its own row position, destination and segment
state.

<figure class="bf-fig">
<p class="bf-title">Independent traversal chains</p>
<svg viewBox="0 0 720 286" role="img" aria-label="Four of 32 lanes, each a chain of dependent lookups; the third lookup of every lane is highlighted as being in flight at the same time."><rect x="326" y="22" width="112" height="204" rx="10" class="bf-band"/><text x="382" y="16" class="bf-t-green" text-anchor="middle">in flight together</text><text x="0" y="56.0" class="bf-ts">Lane 0</text><rect x="74" y="34" width="96" height="34" rx="7" class="bf-cell"/><text x="122" y="56.0" class="bf-t-strong" text-anchor="middle">lookup 1</text><path d="M174 51H200m-5-5 5 5-5 5" class="bf-arrow"/><rect x="204" y="34" width="96" height="34" rx="7" class="bf-cell"/><text x="252" y="56.0" class="bf-t-strong" text-anchor="middle">lookup 2</text><path d="M304 51H330m-5-5 5 5-5 5" class="bf-arrow"/><rect x="334" y="34" width="96" height="34" rx="7" class="bf-cell-hi"/><text x="382" y="56.0" class="bf-t-strong" text-anchor="middle">lookup 3</text><path d="M434 51H460m-5-5 5 5-5 5" class="bf-arrow"/><rect x="464" y="34" width="96" height="34" rx="7" class="bf-cell"/><text x="512" y="56.0" class="bf-t-strong" text-anchor="middle">lookup 4</text><path d="M564 51H590m-5-5 5 5-5 5" class="bf-arrow"/><rect x="594" y="34" width="96" height="34" rx="7" class="bf-cell"/><text x="642" y="56.0" class="bf-t-strong" text-anchor="middle">lookup 5</text><text x="0" y="106.0" class="bf-ts">Lane 1</text><rect x="74" y="84" width="96" height="34" rx="7" class="bf-cell"/><text x="122" y="106.0" class="bf-t-strong" text-anchor="middle">lookup 1</text><path d="M174 101H200m-5-5 5 5-5 5" class="bf-arrow"/><rect x="204" y="84" width="96" height="34" rx="7" class="bf-cell"/><text x="252" y="106.0" class="bf-t-strong" text-anchor="middle">lookup 2</text><path d="M304 101H330m-5-5 5 5-5 5" class="bf-arrow"/><rect x="334" y="84" width="96" height="34" rx="7" class="bf-cell-hi"/><text x="382" y="106.0" class="bf-t-strong" text-anchor="middle">lookup 3</text><path d="M434 101H460m-5-5 5 5-5 5" class="bf-arrow"/><rect x="464" y="84" width="96" height="34" rx="7" class="bf-cell"/><text x="512" y="106.0" class="bf-t-strong" text-anchor="middle">lookup 4</text><path d="M564 101H590m-5-5 5 5-5 5" class="bf-arrow"/><rect x="594" y="84" width="96" height="34" rx="7" class="bf-cell"/><text x="642" y="106.0" class="bf-t-strong" text-anchor="middle">lookup 5</text><text x="0" y="156.0" class="bf-ts">Lane 2</text><rect x="74" y="134" width="96" height="34" rx="7" class="bf-cell"/><text x="122" y="156.0" class="bf-t-strong" text-anchor="middle">lookup 1</text><path d="M174 151H200m-5-5 5 5-5 5" class="bf-arrow"/><rect x="204" y="134" width="96" height="34" rx="7" class="bf-cell"/><text x="252" y="156.0" class="bf-t-strong" text-anchor="middle">lookup 2</text><path d="M304 151H330m-5-5 5 5-5 5" class="bf-arrow"/><rect x="334" y="134" width="96" height="34" rx="7" class="bf-cell-hi"/><text x="382" y="156.0" class="bf-t-strong" text-anchor="middle">lookup 3</text><path d="M434 151H460m-5-5 5 5-5 5" class="bf-arrow"/><rect x="464" y="134" width="96" height="34" rx="7" class="bf-cell"/><text x="512" y="156.0" class="bf-t-strong" text-anchor="middle">lookup 4</text><path d="M564 151H590m-5-5 5 5-5 5" class="bf-arrow"/><rect x="594" y="134" width="96" height="34" rx="7" class="bf-cell"/><text x="642" y="156.0" class="bf-t-strong" text-anchor="middle">lookup 5</text><text x="34" y="186" class="bf-tm" text-anchor="middle">⋮</text><text x="382" y="186" class="bf-tm" text-anchor="middle">⋮ 28 more lanes ⋮</text><text x="0" y="220.0" class="bf-ts">Lane 31</text><rect x="74" y="198" width="96" height="34" rx="7" class="bf-cell"/><text x="122" y="220.0" class="bf-t-strong" text-anchor="middle">lookup 1</text><path d="M174 215H200m-5-5 5 5-5 5" class="bf-arrow"/><rect x="204" y="198" width="96" height="34" rx="7" class="bf-cell"/><text x="252" y="220.0" class="bf-t-strong" text-anchor="middle">lookup 2</text><path d="M304 215H330m-5-5 5 5-5 5" class="bf-arrow"/><rect x="334" y="198" width="96" height="34" rx="7" class="bf-cell-hi"/><text x="382" y="220.0" class="bf-t-strong" text-anchor="middle">lookup 3</text><path d="M434 215H460m-5-5 5 5-5 5" class="bf-arrow"/><rect x="464" y="198" width="96" height="34" rx="7" class="bf-cell"/><text x="512" y="220.0" class="bf-t-strong" text-anchor="middle">lookup 4</text><path d="M564 215H590m-5-5 5 5-5 5" class="bf-arrow"/><rect x="594" y="198" width="96" height="34" rx="7" class="bf-cell"/><text x="642" y="220.0" class="bf-t-strong" text-anchor="middle">lookup 5</text><path d="M74 258H716m-6-6 6 6-6 6" class="bf-arrow"/><text x="716" y="278" class="bf-tm" text-anchor="end">time</text><text x="74" y="278" class="bf-tm">Along a lane, each lookup waits for the previous one. Across lanes, nothing waits.</text></svg>
<figcaption>Four of the 32 lanes. Within a lane, each lookup waits for the one before it; across lanes, the CPU can overlap the lookups.</figcaption>
</figure>

The source calls this walk `room_segments::<32, PF_L1>`. The `PF_L1` part enables a
prefetch hint for the next T-vector entry. The "room" algorithm batches output writes to
avoid a buffer-capacity check on every byte, but it still performs one dependent lookup for
every decoded byte.

## 2. Where does "8.6 effective chains" come from?

Two measured figures:

| Measurement            | Cost              |
| ---------------------- | ----------------- |
| One dependent chain    | 38.55 cycles/byte |
| The 32-lane room walk  | 4.49 cycles/byte  |

Their ratio is 38.55 ÷ 4.49 = **8.59**.

So the walk achieves about 8.6 times the byte rate of the serial chain, provided the two
measurements are comparable. It does **not** mean that only 8.6 of the 32 lanes are working.

That distinction decides whether the next optimisation should look for more parallelism, or
for less work per byte.

## 3. Why 32 lanes might yield only an 8.6× speed-up

Imagine a processor that takes 39 cycles to complete a dependent table lookup. If one lookup
starts every 4.49 cycles, the average number in progress is about 39 ÷ 4.49 ≈ 8.7.

That is Little's law: concurrency equals request rate multiplied by latency. Computer
architecture distinguishes this achieved memory-level parallelism from the number of
independent chains a program makes available.

But consider what that means for mibz2. A full round of the walk, one step for each of the 32
lanes, takes 32 × 4.49 ≈ 144 cycles.

<figure class="bf-fig">
<p class="bf-title">One round of the room walk, against one lookup's latency</p>
<svg viewBox="0 -6 720 326" role="img" aria-label="A round of 32 steps takes about 144 cycles; each step starts a lookup of about 38.55 cycles, so about 8.6 overlap at any moment, and each lane’s lookup finishes well before its next turn."><text x="0" y="8" class="bf-tm">One round: the walker steps each of its 32 lanes once, one step every 4.49 cycles</text><rect x="0.6" y="20" width="19.75" height="20" rx="2" class="bf-slot-hi"/><rect x="21.55" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="42.51" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="63.46" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="84.41" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="105.37" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="126.32" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="147.27" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="168.23" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="189.18" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="210.13" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="231.09" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="252.04" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="272.99" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="293.95" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="314.9" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="335.85" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="356.81" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="377.76" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="398.71" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="419.67" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="440.62" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="461.57" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="482.53" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="503.48" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="524.43" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="545.39" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="566.34" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="587.29" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="608.25" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="629.2" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="650.15" y="20" width="19.75" height="20" rx="2" class="bf-slot"/><rect x="671.11" y="20" width="19.75" height="20" rx="2" class="bf-slot-hi"/><text x="2" y="54" class="bf-ttiny">lane 0</text><text x="691.46" y="54" class="bf-ttiny" text-anchor="end">lane 0 again</text><text x="335.25" y="54" class="bf-tm" text-anchor="middle">32 × 4.49 ≈ 144 cycles</text><text x="0" y="74" class="bf-tm">Each step starts a lookup that takes about 38.55 cycles to come back</text><rect x="0" y="84" width="179.9" height="9" rx="3" class="bf-lat"/><rect x="20.95" y="96" width="179.9" height="9" rx="3" class="bf-lat"/><rect x="41.91" y="108" width="179.9" height="9" rx="3" class="bf-lat"/><rect x="62.86" y="120" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="83.81" y="132" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="104.77" y="144" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="125.72" y="156" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="146.67" y="168" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="167.63" y="180" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="188.58" y="192" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="209.53" y="204" width="179.9" height="9" rx="3" class="bf-lat-hi"/><rect x="230.49" y="216" width="179.9" height="9" rx="3" class="bf-lat"/><rect x="251.44" y="228" width="179.9" height="9" rx="3" class="bf-lat"/><path d="M224 78V242" class="bf-cut"/><text x="232" y="256" class="bf-t-green">at any moment ≈ 38.55 ÷ 4.49 ≈ 8.6 lookups in progress</text><rect x="0" y="284" width="179.9" height="12" rx="3" class="bf-lat-hi"/><path d="M185.9 290H664.51" class="bf-grid"/><rect x="671.11" y="281" width="19.75" height="18" rx="2" class="bf-slot-hi"/><text x="191.9" y="312" class="bf-tm">lane 0’s lookup is back long before its next turn: all 32 lanes keep moving without 32 loads in flight</text></svg>
<figcaption>Illustrative, from the two measured figures above. About 8.6 lookups overlap at any moment, yet every lane's lookup returns long before that lane's next turn.</figcaption>
</figure>

If a lookup takes roughly 39 cycles, the CPU has ample time to finish a lane's lookup before
it comes back to that lane in the next round. So all 32 lanes can be making progress without
32 loads ever being in flight at once.

That would place the bottleneck somewhere else.

There is another caveat. The room walk explicitly prefetches future entries, so its actual
demand-load latency may be much lower than the 38.55 cycles of the single chain. The two
figures also need the same CPU, revision and cache conditions. So 8.6 is best read as an
illustrative latency-hiding factor, not a measured count of outstanding memory requests.

## 4. What to look for in the machine code

Here is one iteration of the hot loop, simplified:

```rust
for lane in lanes.iter_mut() {
    let q = t[lane.row];                   // dependent load
    if q & START != 0 { /* handoff */ }    // branch
    lane.row = q >> 8;                     // next link
    prefetch(t, lane.row);                 // future memory request
    *base[lane_index].add(off) = q as u8;  // output store
}
```

Several explanations compete:

| Suspected limit             | What it means |
| --------------------------- | ------------- |
| Register pressure           | Too few architectural registers to hold all 32 lanes' row and output state, so the generated loop keeps reloading and storing bookkeeping |
| Instruction-window pressure | Too many instructions separate successive independent T-vector loads, which limits what the CPU can have in flight |
| Execution throughput        | Address generation, branches, prefetches and stores use enough execution resources to set a floor in cycles per byte |
| Memory-system saturation    | The core already overlaps enough lookups, and exposing more chains won't help |

A Zen 3 core, for example, has a 256-entry reorder window and a finite number of load and
miss-tracking resources: substantial, but not unlimited.

Reading the disassembly, not just the Rust, shows which dependencies LLVM actually creates.
One promising possibility is that the CPU spends too much time maintaining `lane.row`,
`lane.seg`, `lane.p` and `lane.end` relative to the useful BWT lookup.

The segment-start branch, by contrast, is rarely taken: about 512 boundaries per 900,000
bytes, or 0.057% of steps. Even a 20-cycle misprediction at every boundary costs only about
0.011 cycles per byte, so boundary mispredictions alone can't explain an opportunity of more
than 1 cycle per byte. The throughput of the per-byte branch and bookkeeping could still
matter.

## 5. What does "four groups of eight" mean?

This is scalar batching, not necessarily SIMD. The idea is to arrange the 32 logical lanes as
four groups of eight, so that each group's working state is easier for the compiler to keep
in registers.

But there's a trap.

<figure class="bf-fig">
<p class="bf-title">Batching the 32 lanes as four groups of eight</p>
<svg viewBox="0 0 720 232" role="img" aria-label="Left: four groups of eight lanes interleaved, all 32 chains exposed. Right: one group at a time, only 8 chains exposed, with a latency floor of 4.82 cycles per byte against 4.49 today."><rect x="0" y="0" width="340" height="232" rx="10" class="bf-card-hi"/><text x="170.0" y="28" class="bf-t-green" text-anchor="middle">Interleaved groups</text><text x="170.0" y="48" class="bf-tm" text-anchor="middle">each step touches every group</text><text x="22" y="79" class="bf-tm">group 0</text><rect x="82" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="112" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="142" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="172" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="202" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="232" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="262" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="292" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><text x="22" y="109" class="bf-tm">group 1</text><rect x="82" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="112" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="142" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="172" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="202" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="232" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="262" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="292" y="94" width="24" height="20" rx="4" class="bf-chip-on"/><text x="22" y="139" class="bf-tm">group 2</text><rect x="82" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="112" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="142" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="172" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="202" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="232" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="262" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="292" y="124" width="24" height="20" rx="4" class="bf-chip-on"/><text x="22" y="169" class="bf-tm">group 3</text><rect x="82" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="112" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="142" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="172" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="202" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="232" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="262" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="292" y="154" width="24" height="20" rx="4" class="bf-chip-on"/><text x="170.0" y="200" class="bf-t-strong" text-anchor="middle">32 chains exposed</text><text x="170.0" y="220" class="bf-tm" text-anchor="middle">bookkeeping per group, overlap kept</text><rect x="380" y="0" width="340" height="232" rx="10" class="bf-panel"/><text x="550.0" y="28" class="bf-ts" text-anchor="middle">One group at a time</text><text x="550.0" y="48" class="bf-tm" text-anchor="middle">several rounds of group 0, then group 1, …</text><text x="402" y="79" class="bf-tm">group 0</text><rect x="462" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="492" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="522" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="552" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="582" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="612" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="642" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><rect x="672" y="64" width="24" height="20" rx="4" class="bf-chip-on"/><text x="402" y="109" class="bf-tm">group 1</text><rect x="462" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="492" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="522" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="552" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="582" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="612" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="642" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="672" y="94" width="24" height="20" rx="4" class="bf-chip-off"/><text x="402" y="139" class="bf-tm">group 2</text><rect x="462" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="492" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="522" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="552" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="582" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="612" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="642" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="672" y="124" width="24" height="20" rx="4" class="bf-chip-off"/><text x="402" y="169" class="bf-tm">group 3</text><rect x="462" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="492" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="522" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="552" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="582" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="612" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="642" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><rect x="672" y="154" width="24" height="20" rx="4" class="bf-chip-off"/><text x="550.0" y="200" class="bf-t-strong" text-anchor="middle">8 chains exposed</text><text x="550.0" y="220" class="bf-tm" text-anchor="middle">latency floor ≥ 38.55 ÷ 8 = 4.82 c/B (today: 4.49)</text></svg>
<figcaption>Green lanes have a lookup the CPU can overlap. Batching helps only if every group still has lookups in flight within the processor's instruction window.</figcaption>
</figure>

If an implementation runs several rounds of group 0 before touching groups 1 to 3, it may
expose only eight independent chains to the CPU at once. Under the idealised 38.55-cycle
latency, eight chains have a latency floor of 38.55 ÷ 8 = **4.82 cycles per byte**, already
slower than today's 4.49.

That doesn't prove batching fails: real latency changes with prefetching and caching. But it
shows how saving register operations can lose to reduced overlap.

The better transformation reduces lane bookkeeping while still exposing independent loads
from all four groups within the processor's instruction window.

## 6. What would count as a breakthrough?

The question is which of two fundamentally different paths applies.

<figure class="bf-fig">
<p class="bf-title">Two diagnoses, two different optimisations</p>
<svg viewBox="0 0 720 168" role="img" aria-label="Path A: insufficient overlap, remove generated dependencies. Path B, the current hypothesis: too much work per byte, do less per step while keeping all 32 chains."><rect x="0" y="0" width="346" height="168" rx="10" class="bf-panel"/><text x="20" y="30" class="bf-ts">Path A</text><text x="20" y="54" class="bf-ts">Insufficient overlap</text><text x="20" y="84" class="bf-t">Generated code serialises loads that</text><text x="20" y="106" class="bf-t">the algorithm keeps independent.</text><text x="20" y="128" class="bf-t">Fix: remove those dependencies,</text><text x="20" y="150" class="bf-t">without spilling lane state.</text><rect x="374" y="0" width="346" height="168" rx="10" class="bf-card-hi"/><text x="394" y="30" class="bf-t-green">Path B · current hypothesis</text><text x="394" y="54" class="bf-ts">Too much work per byte</text><text x="394" y="84" class="bf-t">Overlap is already sufficient; the</text><text x="394" y="106" class="bf-t">instruction stream sets the pace.</text><text x="394" y="128" class="bf-t">Fix: fewer reloads, address and output</text><text x="394" y="150" class="bf-t">steps per byte, all 32 chains kept.</text></svg>
<figcaption>Path B is an inference from the lane-count curve flattening near 24 lanes and from the 32-lane walk already tolerating one chain's latency. It is not yet a verified machine-code finding.</figcaption>
</figure>

**Path A, insufficient overlap:** show that the generated code serialises independent loads
unnecessarily, then remove those dependencies without giving up register locality or adding
costly memory traffic.

**Path B, too much work per byte:** show that overlap is already sufficient and that the core
is limited by its instruction stream, then reduce row-state reloads, address calculations or
output bookkeeping while keeping all 32 chains.

The current hypothesis leans towards Path B, given that the measured lane-count curve flattens
near 24 lanes, and that the 32-lane scheme already tolerates one chain's latency. That is an
inference, not a verified finding.

So the question to settle is:

> Are the 32 chains insufficiently exposed, or are they already exposed well enough, while the
> processor wastes cycles advancing each one?

The two answers call for completely different optimisations, which is why a careful,
disassembly-level investigation is worth doing.

A way to save roughly 1.2 cycles per walked byte, on the later decoder baseline of 10.78 cycles
per byte, would be worth about 11% of total decoder time, before any gain from the MTF stage.
That is why this direction has much greater breakthrough potential than further polishing the
MTF selector.
