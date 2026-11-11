# Cells inversion: production slice one

This implements the design on PR #362, without modifying that branch or its research
example. Base: master `edaa984c4f57614f25bb897e808ab12c59c48d8f`. Design and handoff read
at PR #362 head `7611f52bd91c68d3b2c8a224cf7672bc1c69a312`. The controlling gate is
[issue #145 comment 6775049636](https://github.com/lmmx/mibz2/issues/145#issuecomment-6775049636):
unchanged default arms level everywhere, parity/B5/Miri pass; experimental figures are
reported, not judged. This is not a default flip or a release.

## Entry and refusal

`bwt::Cells` is `CellsKernel<false>` with all state, including its `Segmented` fallback,
behind one `Box`. `Scratch`'s default remains `Segmented`. The only change in `bwt.rs` is
an export; the shipped records fallback is shared by calling its existing trait method,
without extracting or rewriting it. Cooperative, pipeline and service code is unchanged.
The ordinary column/in-place route also delegates to `Segmented`.

`decode_with::<bwt::Cells>(data, kernels)` runs it through the real scanner, Huffman/MTF,
RLE1 and CRC decoder. Records are exposed by one crate-private read-only accessor.
`CellsKernel::attempt` handles every admission refusal internally. Only the original
fallback's own broken-tiling result can return false from `invert_runs`.

The checks are ordered:

1. `10 * records.len() >= n`: refuse before traversing records or allocating. The count
   is the **capped records**, as specified, not the coalesced maximal-run count.
2. Validate nonzero lengths <=127, exact tiling of n, and primary row < n. Merge adjoining
   same-byte records, split the primary row out, lay out both orders. Refuse if
   `letters - 1 > floor(4*n/25)`.
3. Before **every** cell, including an equal-length step, refuse if the counter already
   equals the budget. Only then increment it and write the cell.

The floor is conservative on arbitrary columns with disconnected cycles: unmarked closed
letters may be dropped without producing a cell. Such a refusal is harmless because it
uses the unchanged fallback. It is not claimed as a lower bound on the work needed to
recover just one cycle of every malformed column.

This implemented floor is the **marked-split letter count minus one**, not the stronger
union-of-cuts `r2 - 1` filter in the research design (Researcher F02). For `1^51 0^50`
at primary 0 it is 2, while the union-of-cuts floor is 3. The hard cell budget still
bounds construction before any output write; this correction does not change admission.

Refusals leave output contents, length and capacity untouched. Their work bounds, before
running the unchanged fallback, are O(1), O(r) order construction, and O(r + B) with at most
B cells respectively. A newly enlarged order buffer also incurs allocation/initialization.
No new timing claim is made from shared-machine execution; the earlier i9 stage ratios
0.815 / 0.827 / 0.818 remain the research baseline, not end-to-end evidence for this port.

## Allocation bound

Let R be the record count and B the integer cell budget. Each order has R+2+B slots of
24 bytes. The arena has 16+256+16+16B+32 bytes; rules have 16B+16 bytes. Including the
4,096-byte front pad, the byte buffer is exactly **48R + 80B + 4,528 + 4G** bytes,
where G=64 in debug/Miri and zero otherwise. The writer's positions take at most 4B
additional bytes. Thus kernel buffers are bounded by **48R + 84B + 4,528 + 4G**, or at
production admission **48R + 13.44n + 4,528 + 4G**. The extra 4B accounts explicitly for
the writer array which the approximate 13n design estimate omitted.

Buffers retain their high-water size across blocks, without geometric growth. The bound
for a reused kernel uses the largest R and B encountered. Growth releases the old buffer
before allocating its replacement, avoiding simultaneous old/new byte buffers. Positions
are similarly bounded and reused. Allocation failure has the library's usual allocation
failure behavior; this does not introduce a fallible allocator API.

This bound excludes the existing fallback table/scratch, the decoder's existing input and
output, allocator metadata and the fixed kernel state. Mixed inputs may retain both
fallback and cells buffers. The experimental arm reports whole-process RSS, including
these costs; +10% is a gate for a later default flip, not a claim of this slice.

## Unsafe argument

Proof scope: memory safety of this port for records accepted by the safe entry, including
columns that are not the BWT of a single cycle. It does not certify the rest of the shipped
decoder, prove speed, or replace independent adversarial review. No caller can manufacture
`Kernel`, `Layout`, words, rules, or offsets: they are private. The forced test route opens
performance admission only; it retains tiling, primary-row and allocation checks.

| Operations | Invariant and establishment | Scope |
|---|---|---|
| `rd32/64`, `wr32/64` | Integer fields lie inside initialized slots/rules. Unaligned accesses are explicit; integers are native endian and never serialized. The safe entry bounds n below 2^24 and records/budget by n; computed offsets fit u32. | All raw integer access in order/induction/writer. |
| Order pass unchecked record/count indexing | Validated records tile n; byte is `u16 >> 8`, hence <256. Loop bounds guard record indices. Counting sort allocates exactly one counterpart per letter, with at most R+2 letters after the primary-row split. | Two finite order passes; no externally supplied links. |
| Order slot stores | Each slot is 24 bytes, in its own region, with a reciprocal slot in the other order. Predecessors are earlier initial slots, except the first slot's self-sentinel. Exactly one marked unit-length letter is installed on both sides. | Established before induction; explicit Miri layout/mark test. |
| Induction slot reads/stores | Live letters have positive remaining lengths. A cell consumes min(ra,rb); it replaces the corresponding rightmost pieces in both orders. A continuing letter's next piece is promised one fresh slot, with its predecessor initialized before use. Equal lengths consume both letters without a fresh slot. At most one new slot per side per counted cell; R+2+B slots per side suffice. | Budget checked **before** writes. Unmarked closed letters are dropped; the unique marked letter cannot be dropped. This also prevents traversing past the orders' sentinels. |
| Speculative prefetch field reads | Every slot offset is at least PAD=4096; FAR*SLOT=288. The two u32 reads before a slot remain inside initialized buffer storage, even if they read padding or stale data. The resulting pointer uses `wrapping_add` and is only a non-faulting hint. | Intrinsic only on x86_64 and outside Miri; no dereference of the hinted address. |
| `copy16` in induction | Every factor has 16 readable initialized bytes in arena/rules. Single-byte alphabet has trailing slack. Read into a value precedes write, permitting overlap. Two speculative writes reach at most top+32. Arena advances by at most 16/cell and reserves 32 trailing bytes. | Both short/long candidate words are written; only the selected representation advances. |
| Rule stores and word lengths | At most one 16-byte rule per cell, plus one speculative 16-byte store at the end. Factors are older words. The marker occurs in at most one factor; stripping it leaves a length <=n. The mark therefore cannot carry into packed word offsets. | Acyclic grammar, bounded return times of disjoint interval pieces. |
| Position array reads/stores | Only long words name rules; their aligned offsets index an earlier rule in `[0,nr)`, nr<=B. Positions start at NIL. The marked spine seeds unmarked factors; descending rules propagate first positions before ascending rules read them. Marked rules are not expanded by the unmarked pass. | Whole initialized array, separate allocation. Original Miri fixtures did not reach long rules (F01); the added `miri_cells_long_writer*` tests assert actual seeding, propagation, expansion, long copies and marked-spine recopy. Completed-run evidence, not selection alone, establishes Miri coverage. |
| Output pointer writes and `copy_small/long` | Output is resized/initialized to n+32 before taking its pointer. Rotation cuts immediately after the marked unit row, so no unmarked factor straddles that cut. Its destination and length fit the period <=n. Short factors copy exact lengths from the distinct arena allocation. Long factors copy from an already expanded first occurrence; `ptr::copy` also permits overlap. | All output pointers remain in the same stable allocation through the writer. No raw reference survives resizing. |
| Period repetition | A successful marked word has period in 1..=n, checked before writing. The writer covers the cycle, then safe doubling `copy_within` repeats/truncates it to n. | Includes disconnected and short-cycle malformed columns, checked against the shipped walk and decoder. |

Debug and Miri layouts place a 64-byte canary after each of the four regions and check
all four on both success and refusal, and after writing. Miri catches allocation/provenance
errors; the canaries catch crossing between regions in the same allocation. End counters
are also checked before the writer. Canary checks are absent in release, while admission
and budget checks remain. Tests intentionally damage a canary to verify detection.

## Evidence and review

See `VALIDATION.md` for exact commands, outcomes, and missing evidence. Differential tests
include all synthetic benchmark corpora, Fibonacci, Thue–Morse, periods, `(ab)^k a`, zeros,
uniform bytes, two-run columns, arbitrary columns, CRC failures, truncation, concatenated
streams, partial output, exact admission edges and buffer reuse. Output is complemented
before kernel tests to detect stale/missing writes.

The `cells` fuzz mode uses both ordinary admission and `ForcedCells` (all densities,
budget n, including columns rejected by `takes_runs`). It asserts counts of all three
production refusals, successful cells, and short cycles. Forced refusals must be zero on
accepted fronts. The same comparison checks exact shipped error kind, computed CRC, block,
bit and partial output; libbzip2 is an independent complete-output/error oracle.

`miri_cells_*` tests and workflow path/selection checks are included. Admission mutations
use cargo-mutants 27.1.0. `mutations.py` supplies the separate hand-selected unsafe-kernel
mutations and requires each mutant to compile and fail its named test; timeouts and build
failures do not count as kills.

F01's targeted correction adds test-only per-call counters inside the actual writer
branches. Complemented output makes missed writes observable; smaller-block reuse
retains the arena, positions and output allocations. The n=550/R=8/B=88 witness calls
ordinary `Cells::invert_runs`, asserts admission without any refusal, 57 cells and period
55. The writer-specific mutant removes only the final recopy, leaving counters intact;
byte parity must catch it. Original 37-test Miri and five-mutant outcomes remain valid
for their original limited reach. See [F01 evidence](../2026-11-11-cells-f01/HANDOFF.md)
for the corrected source digest, full Miri completion and independent recheck status.

Independent adversarial B5 review is still required. The implementer wrote this proof and
the tests; there is no claim of independent review or merge readiness.

## Benchmark arm

Build the ordinary harness and copy it before building the experimental executable:

```sh
cargo build --release --example ab
mkdir -p bin
cp target/release/examples/ab bin/ab
cargo rustc --release --example ab -- --cfg mibz2_cells_bench
cp target/release/examples/ab bin/cells-ab
bin/cells-ab run cells.jsonl --arms mibz2@bin/ab,mibz2-cells --threads 1 corpora/*.bz2
bin/cells-ab summary cells.jsonl --ref mibz2@bin/ab
```

This selects the experimental type in the real
block decoder. Its sequential `Read` adapter is local to the example, with the shipped
reader's chunking, compaction, error behavior and buffer reuse. Adapter tests compare
bytes and errors with several read sizes. It is explicitly a single-thread arm; it never
selects cooperative paths or claims multi-thread scaling.

CI keeps its existing paired default arms and their schedule unchanged. A separate
same-source, separate-executable paired run writes `cells.jsonl` and `cells-summary.md` (wall, CPU, RSS, byte
hash parity and confidence intervals) into the benchmark artifact and job summary.
Those figures are observational. No statistical method or default acceptance rule changes.
See the [qualification handoff](../2026-11-10-cells-qualification/HANDOFF.md) for why
the benchmark executables are isolated and the original CI/rerun evidence.
