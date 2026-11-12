# External research report 88: how much parallel work the inverse-BWT walk really performs (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 11:03 UTC.
- **Source:** the researcher (recorded as received; the maintainer had it made into a blog post).
- **In reply to:** the walker's evidence.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

`````text
can you turn this message from the researcher into a blog post and put it on the docs site? it should remove reference to astra / "you" etc and (very lightly edit) make it fit to be shown on the docs site as a blog like the previous one, including recreate any diagrams


````
It's a question about how much parallel work the CPU is actually performing inside mibz2's inverse-BWT decoder, and whether we're wasting hardware capacity because of how Rust compiles the loop.

The interesting part is that the suspected problem may not be insufficient parallelism at all. The figures could instead indicate that the CPU is already hiding memory latency effectively, but spending too many instructions on each decoded byte.

## 1. Why does the BWT decoder have 32 lanes?

Inverse BWT in mibz2 involves repeatedly looking up the next position in a table. Simplifying the actual code:

```
let q = t[row];
row = q >> 8;
output.push(q as u8);
```

The next lookup depends on the previous result:

\\[ r\_{i+1}=\Psi(r_i) \\]

The CPU cannot know where to look next until the current lookup completes. This is called pointer chasing. It's inherently serial along one chain.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://learn.arm.com\&sz=32)

Arm Learning Paths+1



Mibz2's solution is to divide the inverse-BWT traversal into 512 segments and process up to 32 independent segments at once. Each lane has its own row position, destination and segment state.

Simplified: independent traversal chains

Within a row, each lookup waits for the previous one. Across rows, the CPU can overlap the lookups. Four lanes illustrated; the actual implementation uses 32.

The source uses `room_segments::<32, PF_L1>`. The `PF_L1` part enables a prefetch hint for the next T-vector entry. The room algorithm batches output writes to avoid per-byte buffer-capacity checks, but still performs a dependent lookup for every decoded BWT byte.

## 2. Where does “8.6 effective chains” come from?

Two measured figures in Claude's evidence package:

| Measurement            | Cost              |
| ---------------------- | ----------------- |
| Single dependent chain | 38.55 cycles/byte |
| Optimized room walker  | 4.49 cycles/byte  |

The ratio is:

\\[ \frac{38.55}{4.49}=\boxed{8.59} \\]

That means the walker achieves approximately 8.6 times the byte rate of the serial walker, under the assumptions that make the measurements comparable.

It does not mean that only 8.6 of the 32 lanes are working.

That distinction is central to whether Astra should pursue more parallelism or less work per byte.

## 3. Why 32 lanes might yield only 8.6× speedup

Imagine a processor that takes 39 cycles to complete a dependent table lookup.

If one lookup is started every 4.49 cycles, the average number in progress would be approximately:

\\[ N\_{\mathrm{flight}}\approx\frac{39}{4.49}=8.7. \\]

That's an application of Little's law: concurrency equals request rate multiplied by latency. Computer-architecture research distinguishes this achieved memory-level parallelism from the number of independent chains the application makes available.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://scottbeamer.net\&sz=32)

scottbeamer.net



But consider what happens in mibz2:

\\[ 32\text{ lanes}\times4.49\text{ cycles/byte} \approx144\text{ cycles per complete round}. \\]

If a lookup takes roughly 39 cycles, the CPU has ample time to finish a lane's lookup before revisiting it on the next round.

So the processor could have all 32 lanes working correctly, without needing 32 simultaneous outstanding loads.

That would mean the bottleneck is elsewhere.

There's another caveat: the room walker explicitly prefetches future entries, so its actual demand-load latency may be much less than the 38.55-cycle single-chain measurement. The two figures also need matching CPU, revision and cache conditions. Consequently, 8.6 is best regarded as an illustrative latency-hiding factor, not a measured hardware-occupancy figure.

## 4. What Astra is looking for in the machine code

Here is one iteration of the actual hot loop, simplified:

```
for lane in lanes.iter_mut() {
    let q = t[lane.row];                 // dependent load
    if q & START != 0 { /* handoff */ } // branch
    lane.row = q >> 8;                  // next link
    prefetch(t, lane.row);              // future memory request
    *base[lane_index].add(off) = q as u8; // output store
}
```

Several possible explanations compete:

| Suspected limit             | What it means                                                                                                                                       |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Register pressure           | The CPU has too few architectural registers to retain all 32 lanes' row and output state; the generated loop continually reloads/stores bookkeeping |
| Instruction-window pressure | Too many instructions separate successive independent T-vector loads, limiting what the CPU can have in flight                                      |
| Execution throughput        | Address generation, branches, prefetch instructions and stores consume enough execution resources to impose a cycles/byte floor                     |
| Memory-system saturation    | The core already overlaps enough lookups; more exposed chains won't help                                                                            |

For example, Zen 3 has a 256-macro-op reorder/retirement window and finite load and miss-tracking resources. Those are substantial but not unlimited.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://x86-64.net\&sz=32)

x86-64.net



Astra reads the disassembly, rather than just the Rust, to determine which dependencies LLVM actually creates.

A particularly promising possibility is that the CPU spends too much time maintaining `lane.row`, `lane.seg`, `lane.p` and `lane.end` relative to the useful BWT lookup.

Notice also that the segment-start branch is rarely taken: roughly 512 boundaries per 900,000 bytes, or 0.057% of steps. Even charging a hypothetical 20-cycle misprediction for every boundary contributes only about 0.011 cycles/byte. Thus boundary mispredictions alone cannot plausibly explain a 1+ c/B opportunity. Per-byte branch and bookkeeping throughput could still matter.

## 5. What does “four groups of eight” mean?

This is scalar batching, not necessarily SIMD.

The idea is to reorganize 32 logical lanes as four groups of eight so that each group's working state is easier for the compiler to keep in registers.

But there's a trap.

If the implementation completes several lookup rounds for group 0 before touching groups 1–3, it might expose only eight independent chains to the CPU at once.

Under the idealized 38.55-cycle latency model, eight chains have a latency-imposed floor of

\\[ C_8\ge\frac{38.55}{8}=\boxed{4.82\text{ c/B}}, \\]

already slower than the observed 4.49 c/B.

That does not prove batching fails—the real latency changes with prefetching and caches—but it shows why saving register operations can lose to reduced overlap.

The better transformation would reduce lane bookkeeping while still exposing independent loads from all four groups within the processor's instruction window.

## 6. What would constitute a breakthrough?

Astra's task is to decide between two fundamentally different paths:

Path A: insufficient overlap

Show that independent loads are unnecessarily serialized by generated dependencies. Remove those dependencies without sacrificing register locality or adding costly memory traffic.

Path B: excessive work per byte

Show that overlap is already sufficient and the core is limited by the instruction stream. Reduce row-state reloads, address calculations or output bookkeeping while preserving all 32 chains.

My current research hypothesis leans toward Path B, given the reported lane-count curve flattening near 24 lanes and the fact that the existing 32-lane scheme already tolerates single-chain latency. This is an inference, not a verified machine-code finding.

The key question I'd want Astra to answer is therefore slightly different from Claude's wording:

“Are the 32 chains insufficiently exposed, or are they already sufficiently exposed while the processor wastes cycles advancing each one?”

Those require completely different optimizations. Establishing which is true is precisely why an expert disassembly-level investigation could be valuable here.

If Astra identifies a way to save roughly 1.2 cycles per walked byte on the later decoder baseline, that's potentially worth around 11% of total decoder time before any P74 gains. That's why this research direction has much greater breakthrough potential than further polishing the MTF selector.
````


Feel free to embellish the diagram, it was a bit basic, but use only svg and if any interactivity is needed (it's not) we use light JS. It should look nice like the rest of the blog. You can hand that off to a subagent or if you're free just DIY (i think DIY is cheaper than using a subagent).

(Please do that as a docs PR and merge it autonomously, there is no need to be wary of merging a docs PR)
`````
