claim: forward-emission
by: claude-code
outcome: done
evidence: PR #PRNUM; registration docs/journal/2026-11-27-forward-emission/registration.md (frozen in 23825b52, before any timing); src/open/lanes.rs

**What was asked:** freeze the kill-test registration and build the paid S = 16 sampled-anchor forward wiring and four-lane emission in `src/open.rs`, behind an opt-in research feature.
**What I found:**
- **Built:** `forward-anchor-lanes` (src/open/lanes.rs, plus about 25 lines in src/open.rs). It applies to a queued block with narrow edges, codes fitting under bit 31 and `m >= 256`:
  - forward `Reader::wire` replaces `wire_xor`;
  - 16 rows `mark + k*m/16` are tagged with bit 31;
  - four lanes walk from those anchors into 32 KiB slabs of a reused per-worker arena (about 1.03 MiB), with one sign test for the four loaded edges;
  - the segments are chained from the marked row's in O(16), stitched into `out` (phase A, contiguous), and repeated for short cycles.
  - Step and byte budgets fall back to the shipped bounded `emit` after clearing the tags. There is no new `unsafe`.
- **Exact:** the full library suite passes under the feature (205 tests), as do the composition with `a6b-endpoint-redesign` and `mtf-fold` and the default suite. The new tests cover arbitrary columns on three construction arms, periodic BWTs whose anchors are off the marked cycle, 900k-row blocks, non-permutation edges (fallback, no tag left) and a Miri case; Miri is clean. On real data, all 233 admitted wikidata blocks (`82f65ace…`) take the walk with no fallback, and the output is byte-identical to libbzip2's (`3f075e30…`).
- **The first build lost:** the walk was 30% slower than 1-lane on a full block. A Walk struct held the lanes' rows and cursors behind `&mut self`, which serialised the four loads. Moving them into locals and hoisting the arena slice gave 2.7× over 1-lane on a 600k-row real BWT (6.2 vs 15–18 ms a block).
- **VM diagnostic only (C5, 4-vCPU Haswell, decides nothing):** `ab` arm `mibz2-workers-lazy`, 1W pinned, 12 interleaved rounds: median CPU 0.587 → 0.543 s (−7.5%), min 0.546 → 0.517. That is in line with the probe rerun here, warm: bidi 177 ms against the stitched arm's 147 ms for `A6c+A7`.
**What changed:** Cargo.toml (feature); src/open.rs; src/open/lanes.rs (new, added to owned_paths); the CI steps in test.yml and miri.yml (feature tests, Miri); the registration's §5. §5 states the 1W route, the open kernel forced at one worker since the default plan uses Cells below 8 workers, and the data seen; it changes no threshold. New claim `forward-emission-run`.
**What's next:** `forward-emission-run`, the i9 recipe for §2's three cells, reusing #1085's two-build harness. Phase B (vectored output with `forward-rle1-crc-vectored-build`) would remove the ~12 ms/233-block stitch copy. Keep `C-A6c`/`E-walk` in surface.tsv until i9 data.
**Caveats:** the walk is not engaged below 8 workers in the shipped plan, so a 1W gain is only on the forced open route. The 8W effect is untested, and masked by the scanner tie unless `S ideal`. The arena adds about 1 MiB RSS per worker.
