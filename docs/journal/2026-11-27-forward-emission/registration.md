# Registration: sampled-anchor forward wiring and 4-lane emission in `Open` (`forward-anchor-lanes`)

- **Claim:** `forward-emission`
- **Kind:** kill test (RULES C1a), frozen before any timing of the `src/` prototype. It is **not** an adoption registration: a PASS here does not change any default. Adoption needs its own C6 registration, frozen before its data and reviewed by a different agent, which may reuse this run's harness.
- **Status:** frozen 27 November 2026, before data. C2 applies: nothing below is relaxed after any data is seen.
- **Feature flag:** `forward-anchor-lanes` (opt-in, research only; the default build is unchanged).
- **Premises:** the PASS of the VM probe `forward-4lane-anchor-probe` (#1050, `docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md`, `examples/forward_4lane_probe.rs`); `frontier/DECODER.md`; `docs/team/PERFORMANCE-POLICY.md`; the claim's `why`.

## 1. Mechanism (what the prototype changes)

The worker's construction wiring (`A6c`) and emission walk (`A7`) in `src/open.rs`, for a queued block whose codes fit beside an anchor tag (`!wide`, largest code `<= (u32::MAX >> 1) >> bits`) and whose retained rows `m >= 256`:

1. **Forward wiring.** The XOR-linked wiring `outq::Reader::wire_xor` (used when `m >= 65,536`) is replaced by the forward `outq::Reader::wire` that already ships for smaller blocks. Each edge holds `code << bits | next`.
2. **Sampled anchors.** Right after wiring, 16 rows `a_k = (mark + k*m/16) mod m` are tagged with bit 31 (16 scalar stores). For `m >= 256` the 16 rows are distinct.
3. **Four lanes.** Four cursors walk forward from four anchors at once, each writing into its own 32 KiB slab of a per-worker arena. One branch tests all four loaded edges for a tag (`(e0 | e1 | e2 | e3) as i32 < 0`). A lane that reaches a tagged row closes its segment and takes the next unstarted anchor, continuing in the same slab. Once all 16 anchors have started, the lanes drain through 3-, 2- and 1-lane loops.
4. **Chain and stitch.** The tags are cleared. The segments are chained from the marked row's segment, in cycle order, in O(16). Their slices are copied into the contiguous `out` (the probe's "stitch", phase A), and a cycle shorter than the block is repeated to `n` bytes, as now. Nothing downstream changes (RLE1, CRC, delivery).
5. **Fallbacks.** Blocks failing the tag condition, or with `m < 256`, take the shipped path unchanged. If a lane exceeds the total step budget `m`, or the arena's byte budget (an invalid construction), the tags are cleared and the shipped 1-lane bounded `emit` runs; a wrong block is still rejected by its CRC, never an unbounded walk.

Memory: one per-worker arena of `n + 4 * 32 KiB + 4 * 16` bytes (about 1.03 MiB at `n = 900,000`), allocated once and reused. Zero new `unsafe`.

**Station (DECODER.md):** worker-side `C`+`E` (`A6c` 161 ms + `A7` 490 ms of the 1,306 ms 1W wall on the i9, `character-stage1`). At `k = 1` it is on the critical path. At `k = 8` the scanner and workers co-limit, so a worker cut is partly masked in shipped wall; `S ideal` (the `scan-oracle` feature) unmasks it. The probe modelled phase A at +175..+208 ms at 1W and about +25..+29 ms per worker at 8W: model, not measurement.

## 2. Design (i9-10900K workstation)

- **Machine:** Louis's i9-10900K, through `bench/workstation-queue.txt` once the recipe is frozen, reviewed and green.
- **Builds (arms):**
  - `base`: master at the prototype PR's merge base, default features.
  - `cand`: the prototype head with `--features forward-anchor-lanes`.
  - For the 8-worker scan-unmasked cells, both are additionally built with `scan-oracle` (`S ideal`, warm paired-oracle protocol, `docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md`).
- **Corpus:** `wikidata x4` (about 839 MB output, SHA-256 checked) for the decision. `text` (prose) at `k = 1` as the non-wikidata guard (PERFORMANCE-POLICY.md).
- **Cells:** `k = 1` shipped scan; `k = 8` shipped scan; `k = 8` `S ideal`. All three are decision cells, and all three are run regardless of what the others show.
- **Method:** 15 paired repeats per cell, interleaved, the arm order alternating (AB/BA). Every repeat records wall time, `wkr_busy` where the harness has it, peak RSS, and the output's SHA-256 against libbzip2's. Report medians with [min..max], the median of paired ratios, and its 95% CI by paired bootstrap.

## 3. Frozen decision rule (C2)

**Exactness gate** (precondition): every repeat's output identical to libbzip2's, every unit test and the forced-path tests pass under the feature, and Miri is clean on the feature's tests. Any failure is a **KILL** of this build, whatever the timings show.

Let `r1` be the 1W paired wall ratio (`cand/base`) on `wikidata x4`, `r8` the 8W shipped-scan ratio, and `r8S` the 8W `S ideal` ratio. Upper/lower bounds below refer to each ratio's 95% CI.

- **PASS** (proceed to an adoption registration): `r1`'s upper bound `<= 0.950` (a 1W cut of at least 5%), **and** `r8S`'s upper bound `<= 0.975` (an unmasked 8W cut of at least 2.5%), **and** `r8`'s upper bound `<= 1.010` (no shipped 8W regression beyond 1%), **and** the `text` 1W ratio's upper bound `<= 1.010`.
- **KILL** of this registered implementation: `r1`'s lower bound `> 0.980` (less than 2% at 1W), **or** `r8S`'s lower bound `> 1.000` (no unmasked 8W gain), **or** `r8`'s lower bound `> 1.020`.
- **Otherwise (between):** RESCUE. The opportunity stays live (CLAUDE.md, "optimise and rescue"). The next step is a distinct, smaller change registered afresh before its data (vectored emission, phase B, with `forward-rle1-crc-vectored-build`; a smaller arena; the shipped bidirectional walk kept for `m < 65,536`), not a re-run of this arm.

A KILL records a scoped row in `frontier/dead-ends.tsv` naming this arm (`forward-anchor-lanes` phase A, contiguous stitch), the failing clause and the evidence bundle's digest. It does not rule out the vectored phase B, which removes the stitch copy.

## 4. What this registration does not decide

- No default changes. Any adoption has its own C6 registration and review.
- The recipe (a `just` recipe that runs the cells above unchanged) is a follow-on (`forward-emission-run`). It must implement §2 as written; any departure from §2 needs a new registration before data.
