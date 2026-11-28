# Synthesis, twenty-third pass: the lanes rescue and the wire redesign share a file, not a mechanism

*28 Nov 2026, master `757a91b7`. Claim `synthesise`. No new runs. The figures come from the forward-adopt i9 bundle (`origin/evidence ccfa473f`, read in `docs/notebook/2026-11-28-forward-emission-adopt-i9-verdict.md`), from #1294's CI bench comment (run 46101739416, head `68f0aa5` against `e42326f`), and from the source at master. C2 holds: no frozen verdict changes.*

Merged since the twenty-second pass (`e0184b96`): #1290 (the RSS pricing of segmented-fused-postwalk), #1241 (result-to-release visibility), #1293 (release of a parked hold after its head moved), #1292 (reviews), #1296 (forward-emission-adopt: DO NOT ADOPT, and the P1 bounded-slab rescue filed), #1297 (segmented-fused-postwalk dropped, rescue admitted) and #1298 (bench-crc32-ring-rescue queued).

## 1. The overlap is one shared file, and the rescue doesn't need it

`forward-emission-bounded-slab-rescue` (P1, idea) owns `Cargo.toml`, `src/open.rs` and `src/open/lanes.rs`. The live P3 hold `a6c-wire-redesign` (#1294) owns `src/open/outq.rs` and `src/open.rs`. So `claim` refuses the rescue for as long as a6c holds.

The two claims change different things:

- **#1294's `src/open.rs` diff** factors the two wiring loops into `wire_xor_edges` and `wire_fwd_edges`, and carries `mark_byte` through `First`. It doesn't touch `lanes::walk`'s call site or the `forward-anchor-lanes` cfg gates.
- **The rescue's mechanism is all inside `src/open/lanes.rs`.** `Kernel::walk` (`src/open.rs:773`) already passes `lanes::walk` its `&mut lanes::Arena` and the worker's `&mut out`. The slab pool, the per-lane slabs, the slice list and the final stitch (`out.clear(); out.reserve(n); … extend_from_slice`, `lanes.rs:396–406`) are all private to that module. A bounded-slab walk can keep that signature.
- **`Cargo.toml` and `src/open.rs` matter only to the default flip.** The flip is promoting `forward-anchor-lanes` from a research feature to the default walk. It can only follow a C6 PASS, which is two gates and an i9 run away.

**Recommendation: narrow the rescue, and sequence only the flip.** The build and its C6 run own `src/open/lanes.rs` alone, so the rescue can be admitted and claimed now, beside a6c's hold. The flip goes in its own follow-on PR, after `a6c-wire-redesign` has released `src/open.rs` (merged or dropped). Then nobody waits on a P3 to start a P1, and a6c's holder loses nothing. This pass makes that change in the rescue's claim file. Its stop test is untouched, and the claim was still an idea, with no data against it.

## 2. The RSS diagnosis needs one correction before anyone builds

The verdict puts the +0.84 MiB a worker down to `Arena::reset()` keeping its slabs in `pool` from one block to the next. The source says retention isn't what sets the peak:

- `alloc_slab` pops from `pool` before it allocates, so the pool settles at one block's slab demand and doesn't grow.
- The peak comes within each block. Every slab stays live until the walk ends, and the stitch then copies them into an `out` already reserved to `n`. For that moment the block exists twice, once in slabs and once in `out`. +0.84 MiB a worker is about one Wikidata block of output (~900 kB), which fits the 1-, 6-, 8-, 10- and 12-worker cells (0.82–0.87 MiB a worker).
- Freeing the pool between blocks wouldn't lower the peak. Freed 32 KiB chunks sit below glibc's mmap threshold, so they stay in the heap and stay resident.

So the claim's bound of 4 × 32 KiB a worker has a definite meaning: segment bytes have to land in `out` itself, not in a second copy. Two mechanisms fit inside `lanes.rs`:

1. **Flush on chain.** Copy a finished segment into `out` as soon as it is next in cycle order, and recycle its slabs. Whether this bounds memory depends on the backlog. Anchors are rows `mark + k·m/16` by row index, so their cycle order is unknown until they are walked, and segments may finish in close to random order. If they do, the backlog stays near one block.
2. **Carve the slabs out of `out` and compact in place.** Size `out` to `n` plus the lanes' tails (≤ 4 partial slabs, since a lane continues its next segment in the same slab). Then reorder the slices into cycle order with bounded scratch. That costs a reorder in place of today's stitch copy, which isn't free.

**The cheapest test that tells them apart needs no i9:** instrument the peak number of live slabs under flush on chain, on the Wikidata corpus. If the peak is small, mechanism 1 is enough. If it is near ~28 slabs, only mechanism 2 (or a redesign of how anchors are chosen) can meet the bound.

## 3. If a6c merges first, the rescue's base moves, most of all at 8 workers

a6c's dissection projects cuts of 33–44 ms (at 1 worker) to the shipped bidirectional `wire_xor`, which is the base arm's wiring. It projects only 9–20 ms to the forward `wire` that the lanes walk uses. If a6c merges before the rescue is queued, the base gets faster than the candidate. These are projections from the dissection, not measurements, applied to the ccfa473f cells and assuming worker-bound cells:

| cell | measured ratio (ccfa473f) | if a6c lands first (projected) | rescue gate (upper CI) |
|---|---:|---:|---:|
| wikidata/1 | 0.846 | ~0.859 (0.865 at the worst end) | ≤ 0.900 |
| wikidata/6 | 0.865 | ~0.878 (0.885 at the worst end) | ≤ 0.900 |
| wikidata/8 | 0.964, CI hi 0.986 | ~0.964 if the scanner binds; ~0.974 if fully worker-bound | ≤ 0.975, median ≤ 0.960 |

The 1- and 6-worker gates keep their margin, though 6 workers gets thin. The 8-worker cell decides the run. Two notes for whoever reviews the rescue's C6 registration:

- **Name the base exactly** ("master at queue time", with its SHA, and whether a6c is in it), so that a lost margin isn't read afterwards as a defect in the rescue.
- **The rescue's 8-worker bar is looser than the dropped run's** (≤ 0.975 against ≤ 0.965). It was set after the 0.9859 was seen. That is legitimate for a new candidate under a new registration, but the registration should give its reason (the three unpinned rounds with the SMT-placement tail, in the verdict notebook's §3). The C6 review should check it.

**#1294's CI bench can't confirm or exclude a6c's projection.** The bench shows wikidata −0.5% (−3.7%, +2.8%) at 1 thread, and the projection is about −2 to −3%. Its stage table can't see the change at all: `examples/phases.rs` times the classic T-vector path, not OpenCells, so its "T-vector build +0.0%" says nothing about A6c. The one flag, pattern 4T CPU +7.0%, is one cell in 21, about the chance rate. a6c's own stop test (3) asks for an isolated A6c cut of at least 15% *and* the bench's section 2. That is the holder's to read, and this pass takes no view on it.

## 4. Release and loose ends

- **0.5.18 is still P1 and blocked** on `pre-release-i9-gate` (#1256). a6c is a default-path change on the ordinary-CI route, so if it merges before the release pin it joins the candidate, and the §3.5 i9 gate covers it. The rescue can't join this release: it needs its own C6 run.
- **`result-to-release` is still at `status = "review"` with `pr = 1241`, which has merged.** `brief` reports it as a stale pointer, so `finish-review` didn't reconcile it. It needs marking done by whoever holds or reviewed it. This pass doesn't own it.
- **The `runs-front-i9` bundle (`95edea2a`, 03:01Z) is the live holder's to read** (`runs-front-i9-rescue`, until 07:28Z).
