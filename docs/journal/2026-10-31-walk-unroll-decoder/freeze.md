# The unrolled room walk's full-decoder trial: the freeze record

**Status:**
- Frozen before any timing. Nothing has been timed.
- Every value here was measured on the preparation VM (Intel Xeon, 4 cores, no SMT, AVX2),
  except where a line says deduced. None of it is a timing.

**What fixes the run:**
- the protocol `2026-10-31-walk-unroll-decoder-protocol.md`;
- `frozen.json`: corpora, sources' SHA-256, `source_base` and `text_source_commit`;
- `bench/walk-decoder-pin`.

## Corpora (`frozen.json` has every hash in full)

**Identical to the P74-C decoder trial's frozen corpora.** Compressed and decoded
SHA-256 were both asserted equal to `reference-p74c-decoder-frozen.json`.

| Corpus | Role | Compressed SHA-256 | Decoded bytes |
|---|---|---|--:|
| `wikidata-fitted` | secondary | `82f65ace0731962d…` | 209,700,196 |
| `wikidata-heldout` | secondary | `7f5c9a0044e5a305…` | 235,800,351 |
| `wikidata-20261026` | primary | `cf13f4394e4ea0a3…` | 207,000,624 |
| `random` | control | `10d444e6204289af…` | 8,000,000 |
| `records` | control | `ff8b5f6ed3dd4445…` | 9,600,000 |
| `mixed-independent` | control | `11639f3599a85391…` | 2,000,000 |
| `text` | control | `59236f2dff3afbd4…` | 12,278,087 |

**New walk-structure controls** (protocol §5 item 4):

| Corpus | Compressed SHA-256 | Decoded bytes |
|---|---|--:|
| `text-l1` (`text` at bzip2 -1) | `d4778f8d965bb4b1…` | 12,278,087 |
| `text-l5` (`text` at bzip2 -5) | `61498135ae2a3585…` | 12,278,087 |
| `zeros` | `07dca4a334909c5f…` | 2,700,000 |
| `period-3` | `69893044ae8bff40…` | 2,700,000 |

## Points for review (deviations or findings; none of them a timing)

1. **The toolchain is rustc 1.97.0**, not the P74-C harness's 1.90.0.
   - This is the toolchain of the registered stage PASS (CI run 41222735026), so the
     decoder trial times the same compiled arm.
   - It was chosen before any build for this trial, and is recorded here and in
     `frozen.json`.
2. **`zeros` doesn't reach the room walk** (deduced from bzip2's format, before timing).
   - Its 2.7 MB of zeros shrink under bzip2's initial run-length coding to a block of
     about 53 thousand rows, below `MIN_SEGMENTED` (65,536). So both arms run the plain
     `walk` on it.
   - The protocol's description, "one symbol, so the segments are degenerate", is wrong
     for this file. It is a control of the short-block fallback, which is identical in both
     arms.
   - `period-3` (`abc` repeated, 900,000-row blocks) does reach the room walk.
   - The corpus stays as registered. The finding is reported, not corrected.
3. **W4 (c), driver inlining, needs review before any timing.**
   - Measured on this VM's build (`audit-preparation-vm.json`).
   - **The same stage functions in both arms:** both workers reach the same out-of-line
     `bwt::build`, `mtf::mtf_avx2`, `mtf::mtf_scalar`, `rle::unrle_avx2`,
     `rle::unrle_scalar`, `crc::crc32` and `crc::clmul_update`.
   - **The per-block driver is laid out differently:**
     - **A:** the worker calls `decode_block::<Segmented>` out of line, with the kernel's
       `invert` inlined into it, and calls `SegScratch::assemble` out of line.
     - **B:** `decode_block::<SegmentedUnrolled>` is inlined into the worker, and
       `<SegmentedUnrolled as InverseBwt>::invert` is out of line, with the assembly
       inlined into it.
     - The worker bodies are 1,665 instructions (A) against 2,892 (B).
   - **Size of the effect (deduced):** a few calls a block, against about a millisecond
     of work a block.
   - **Classification:** the assembly is counted with the walk, the change under test,
     because `walk_segments_room` and its twin both end with it. So it isn't one of the
     stages that must match. It is still listed among the inlining differences for review.
   - **Enforcement:** the runner refuses the timed trial until `frozen.json` records this
     signature as reviewed and accepted (`w4_review`). The dry run reports it and
     continues.

## Structural audit on this VM's build (rustc 1.97.0)

| Gate | Result |
|---|---|
| W1, the arm's amendment-1 gates | pass |
| W2, the shipped hot step unchanged (16 / 3 / 2 / 1) | pass |
| W3, each arm reaches only its own walk | pass. Callers of `room_segments::<32, 1>`: `Segmented::invert`, `decode_block::<Segmented>`; of `room_segments_unrolled::<32, 1>`: `SegmentedUnrolled::invert` |
| W4 (a), `src/` changed only by insertions in `bwt.rs` | pass (+545, −0 against master `45b366d`) |
| W4 (b), the same stage functions | pass |
| W4 (c), driver inlining | **review required** (above) |
