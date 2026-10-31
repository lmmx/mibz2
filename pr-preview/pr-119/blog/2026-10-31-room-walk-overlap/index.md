[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-119/blog/index.md>)

# 32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles

The decoder's 32-lane walk runs 8.6 times faster than one chain. That figure may not mean what it seems: the walk may already hide memory latency, and spend too many instructions on each byte.

31 October 2026 · 7 min read · [Open question](<#6-what-would-count-as-a-breakthrough>)

This post asks how much parallel work the CPU actually does inside mibz2's inverse-BWT decoder, and whether hardware capacity is lost to the way Rust compiles the loop.

The interesting part is that the suspected problem may not be too little parallelism at all. The figures could instead mean that the CPU already hides memory latency well, but spends too many instructions on each decoded byte.

## 1\. Why does the BWT decoder have 32 lanes?

The inverse BWT repeatedly looks up the next position in a table. Simplifying the actual code:

```rust
let q = t[row];
row = q >> 8;
output.push(q as u8);
```

The next lookup depends on the result of the previous one: r<sub>i+1</sub> = Ψ(r<sub>i</sub>).

The CPU cannot know where to look next until the current lookup completes. This is called pointer chasing, and along one chain it is inherently serial.

mibz2's answer is to divide the inverse-BWT traversal into 512 segments and walk up to 32 independent segments at once. Each lane has its own row position, destination and segment state.

Independent traversal chains

Four of the 32 lanes. Within a lane, each lookup waits for the one before it; across lanes, the CPU can overlap the lookups.

The source calls this walk `room_segments::<32, PF_L1>`. The `PF_L1` part enables a prefetch hint for the next T-vector entry. The "room" algorithm batches output writes to avoid a buffer-capacity check on every byte, but it still performs one dependent lookup for every decoded byte.

## 2\. Where does "8.6 effective chains" come from?

Two measured figures:

| Measurement | Cost |
| --- | --- |
| One dependent chain | 38.55 cycles/byte |
| The 32-lane room walk | 4.49 cycles/byte |

Their ratio is 38.55 ÷ 4.49 = **8.59**.

So the walk achieves about 8.6 times the byte rate of the serial chain, provided the two measurements are comparable. It does **not** mean that only 8.6 of the 32 lanes are working.

That distinction decides whether the next optimisation should look for more parallelism, or for less work per byte.

## 3\. Why 32 lanes might yield only an 8.6× speed-up

Imagine a processor that takes 39 cycles to complete a dependent table lookup. If one lookup starts every 4.49 cycles, the average number in progress is about 39 ÷ 4.49 ≈ 8.7.

That is Little's law: concurrency equals request rate multiplied by latency. Computer architecture distinguishes this achieved memory-level parallelism from the number of independent chains a program makes available.

But consider what that means for mibz2. A full round of the walk, one step for each of the 32 lanes, takes 32 × 4.49 ≈ 144 cycles.

One round of the room walk, against one lookup's latency

Illustrative, from the two measured figures above. About 8.6 lookups overlap at any moment, yet every lane's lookup returns long before that lane's next turn.

If a lookup takes roughly 39 cycles, the CPU has ample time to finish a lane's lookup before it comes back to that lane in the next round. So all 32 lanes can be making progress without 32 loads ever being in flight at once.

That would place the bottleneck somewhere else.

There is another caveat. The room walk explicitly prefetches future entries, so its actual demand-load latency may be much lower than the 38.55 cycles of the single chain. The two figures also need the same CPU, revision and cache conditions. So 8.6 is best read as an illustrative latency-hiding factor, not a measured count of outstanding memory requests.

## 4\. What to look for in the machine code

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

| Suspected limit | What it means |
| --- | --- |
| Register pressure | Too few architectural registers to hold all 32 lanes' row and output state, so the generated loop keeps reloading and storing bookkeeping |
| Instruction-window pressure | Too many instructions separate successive independent T-vector loads, which limits what the CPU can have in flight |
| Execution throughput | Address generation, branches, prefetches and stores use enough execution resources to set a floor in cycles per byte |
| Memory-system saturation | The core already overlaps enough lookups, and exposing more chains won't help |

A Zen 3 core, for example, has a 256-entry reorder window and a finite number of load and miss-tracking resources: substantial, but not unlimited.

Reading the disassembly, not just the Rust, shows which dependencies LLVM actually creates. One promising possibility is that the CPU spends too much time maintaining `lane.row`, `lane.seg`, `lane.p` and `lane.end` relative to the useful BWT lookup.

The segment-start branch, by contrast, is rarely taken: about 512 boundaries per 900,000 bytes, or 0.057% of steps. Even a 20-cycle misprediction at every boundary costs only about 0.011 cycles per byte, so boundary mispredictions alone can't explain an opportunity of more than 1 cycle per byte. The throughput of the per-byte branch and bookkeeping could still matter.

## 5\. What does "four groups of eight" mean?

This is scalar batching, not necessarily SIMD. The idea is to arrange the 32 logical lanes as four groups of eight, so that each group's working state is easier for the compiler to keep in registers.

But there's a trap.

Batching the 32 lanes as four groups of eight

Green lanes have a lookup the CPU can overlap. Batching helps only if every group still has lookups in flight within the processor's instruction window.

If an implementation runs several rounds of group 0 before touching groups 1 to 3, it may expose only eight independent chains to the CPU at once. Under the idealised 38.55-cycle latency, eight chains have a latency floor of 38.55 ÷ 8 = **4.82 cycles per byte**, already slower than today's 4.49.

That doesn't prove batching fails: real latency changes with prefetching and caching. But it shows how saving register operations can lose to reduced overlap.

The better transformation reduces lane bookkeeping while still exposing independent loads from all four groups within the processor's instruction window.

## 6\. What would count as a breakthrough?

The question is which of two fundamentally different paths applies.

Two diagnoses, two different optimisations

Path B is an inference from the lane-count curve flattening near 24 lanes and from the 32-lane walk already tolerating one chain's latency. It is not yet a verified machine-code finding.

**Path A, insufficient overlap:** show that the generated code serialises independent loads unnecessarily, then remove those dependencies without giving up register locality or adding costly memory traffic.

**Path B, too much work per byte:** show that overlap is already sufficient and that the core is limited by its instruction stream, then reduce row-state reloads, address calculations or output bookkeeping while keeping all 32 chains.

The current hypothesis leans towards Path B, given that the measured lane-count curve flattens near 24 lanes, and that the 32-lane scheme already tolerates one chain's latency. That is an inference, not a verified finding.

So the question to settle is:

> Are the 32 chains insufficiently exposed, or are they already exposed well enough, while the processor wastes cycles advancing each one?

The two answers call for completely different optimisations, which is why a careful, disassembly-level investigation is worth doing.

A way to save roughly 1.2 cycles per walked byte, on the later decoder baseline of 10.78 cycles per byte, would be worth about 11% of total decoder time, before any gain from the MTF stage. That is why this direction has much greater breakthrough potential than further polishing the MTF selector.
