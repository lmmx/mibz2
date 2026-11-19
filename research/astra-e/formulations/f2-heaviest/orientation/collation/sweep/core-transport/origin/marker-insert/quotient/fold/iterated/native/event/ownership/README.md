# Output-owned expansion of the immutable event grammar

The native terminal writer copied 162,605,546 of 205,197,794 marked production bytes from earlier complete subexpression occurrences in its own output. It passed independent full byte-vector comparison on every one of the 228 inputs. Resident emission CPU was 0.5783 [0.5228, 0.6396] and 0.5150 [0.4483, 0.5916] of unchanged depth-first expansion in the two independently labelled comparisons. **The complete inverse took 0.8572 [0.7991, 0.9196] of the dense terminal predecessor's CPU, while still taking 2.1134 [1.8992, 2.3517] times the unchanged named writer's CPU.** [Raw full inverses](phase.csv) · [Raw resident emissions](resident.csv) · [Paired calculations](summary.txt)

The compiler sees one u32 offset per internal node, with the invalid sentinel in the same word. On first entry to an unowned complete production, the writer records the current output offset. Its children have strictly older identities, so no hit can read an incomplete occurrence. Partial visits without an owner split recursively and leave its slot unset. Any later complete or partial visit can copy its desired interval from the complete owner in the output prefix. The same slots remain valid across both ranges of the cyclically marked root. A new output clears and initializes them. The immutable grammar and original leaf dictionary are unmodified. Previously published ownership schedules and the exact emit convention are described in [the proof and provenance](PROVENANCE.md).

## Work and memory

Across the ten systematic blocks, the natural dense FIFO grammar used 878,498 node visits, 178,950 output hits and 7,029,714 copied bytes out of 8,999,915. These are native counts on this grammar. The independent global-anchor Python study has a slightly different contraction schedule. On all 228 inputs we saw 18,958,866 visits, 3,971,143 hits and 42,592,248 bytes from original leaves. Four hits requested a genuine partial subrange from a previously completed owner. There were 61,366 partial misses and 9,433,204 completed publications. The largest explicit traversal stack held 1,512 entries in both writers. Potential long chains remain iterative. The short-period fill is charged; it starts after the ownership pass over the marked period. [All production counts](rss-3.csv) · [Ten-block counts](ten.csv)

The largest block had 66,910 active offsets, or 267,640 live bytes. Normal vector growth retained up to 304,352 bytes over the full sequence. These four bytes per internal node include validity, and the emitter adds zero completion frames. Total retained capacity including output was 11,224,358 bytes versus 10,920,006 for dense events. Fresh dedicated processes reached 9,700 KiB and 9,416 KiB RSS respectively. The indexed and named controls reached 7,316 and 8,184 KiB. These are native inverse-process measurements. [Dense memory](rss-2.csv) · [Indexed memory](rss-1.csv) · [Named memory](rss-0.csv)

## Paid comparison

The full four-arm test used 32 rotated rounds on ten public blocks. The second panel rebuilt one terminal DAG per block and used it for four resident labels over 32 position-balanced repetitions. Every resident invocation resets its offset table and all four output buffers have their own retained capacity. The resident plain twin ratio was 1.0093 [0.9150, 1.1134] and the owner twin ratio 0.8989 [0.7718, 1.0469]. All 2,560 outputs equalled the independent dense LF bytes. The paired t intervals use eight disjoint four-round cluster means. Both panels were pinned to CPU0 at nice +16. The original named stage averaged 7.20 ms in this panel, materially above the 1.83 and 1.90 ms in the preceding event and ribbon panels. Ratios here compare contemporaneous arms only.

Full owner/indexed CPU remained 1.6778 [1.5621, 1.8021]. With an ideal zero-cost emitter the dense inverse would take 0.6512 [0.6187, 0.6853] of its measured complete time, still 1.6054 [1.4601, 1.7652] times the unchanged named complete time. Conversely, even making everything before the owned emitter free leaves its emission alone at 1.2148 [1.1156, 1.3228] times the named writer's second stage. This increment advances terminal reconstruction. It does not qualify for independent whole-worker benchmarking against the production incumbent. A further scratch opportunity follows from #internal productions = #successful events ≤ original token rows−1. One of the now-dead dense membership arrays already has enough slots to hold all owner offsets.

The 13,567 arbitrary marked inputs exercised nonterminal fallbacks, disjoint cycles and 5,241 partial period fills. An additional 1,048 explicit shared-DAG cases included both nonpalindromic multi-byte leaves, duplicated children, marked partial first and last ranges and multiple-period repetition. They exercised 928 partial owner hits, 5,744 partial misses and 60,704 copied bytes. The paid and audited kernels were checked byte-for-byte on the full 228-block sequence. [Synthetic fixtures](synthetic.txt) · [Small raw-column checks](small.csv) · [Dense output receipt](dense.txt)

## Reproduction

The source freeze is ac7763a. Input tapes, source hashes, release library and toolchain are recorded in [manifest.json](manifest.json). Launch from the research checkout after loading Rust 1.99.1 and generating the cap tapes using the predecessor's [input recipe](../README.md).

```sh
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/native/event/ownership
out=/tmp/astra-terminal-ownership
dst=/tmp/astra-owner-results
taskset -c 0 nice -n16 bash "$d/gate.sh" "$out" "$PWD/target/release/deps" "$dst"
taskset -c 0 nice -n16 "$out/owner" price /tmp/astra-clipped/ten.bin > "$dst/phase.csv"
taskset -c 0 nice -n16 "$out/owner" resident /tmp/astra-clipped/ten.bin > "$dst/resident.csv"
python3 "$d/report.py" "$dst"
```
