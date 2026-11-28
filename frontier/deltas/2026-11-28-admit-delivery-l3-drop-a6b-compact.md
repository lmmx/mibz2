claim: none
by: claude-code
outcome: done
evidence: frontier/deltas/2026-11-27-explore-a6b-compact-l3-rescue.md; frontier/dead-ends.tsv ("compact-footprint safe-Rust A6b rescue"); frontier/deltas/2026-11-26-worker-l3-footprint.md; frontier/deltas/2026-11-27-explore-eight-worker-coupling.md

**What was asked:** a swarm found only `synthesise` and `explore` takeable, with 10 open claims under `max_open_claims = 12`. Admit worthwhile independent ideas that no live hold's `owned_paths` blocks.
**What I found:**
- Every P2 build idea overlaps a live hold: `src/rle.rs` (segmented-fused-postwalk), `src/parallel.rs`/`src/stream.rs` (scanner-adaptive-seam-window), `src/block.rs`/`src/huff.rs` (scanner-a2-fast-build), `src/crc.rs` (crc32-avx2-ring-barrett-build), `src/open.rs` (forward-emission-adopt), `frontier/tools/` (result-to-release). `permutation-compression-incubation` is gated by its own text on a demonstrated decoder gain, which is not on record. None of them can be claimed now, so admitting them would only fill the window.
- `a6b-compact-footprint-rescue` asks the question an unclaimed explore session already answered: STOP against an equivalent frozen gate (4,160/4,160 exact; `Vec`, tiered 8K/16K and inline-7 u64 slabs still add +413 to +550 KiB/worker at k = 8; only `edges`-overlay layouts avoid the RSS jump, and safe Rust caps those at -0.70% cold-8). Its dead-end row is already in `dead-ends.tsv`. Re-running it would repeat a recorded arm.
- `delivery-l3-working-set` is smooth (no `owned_paths`), its premise survives `worker-l3-footprint` (which left it eligible, with a <= 2.2% `none` / <= 2.5% `S` 10W ceiling against its 2.0% / 4.0% gates), and the eight-worker-coupling delta added the caution that `SLOTS_PER_THREAD < 4` worsens k = 8 head-of-line stalls unless runts are slot-exempt.
**What changed:** `delivery-l3-working-set` idea -> open; `a6b-compact-footprint-rescue` idea -> dropped (answered by the 27 November explore, see its dead-end row). Open claims 11 of 12.
**What's next:** whoever takes `delivery-l3-working-set` should price its gates against the 2.2% ceiling before spending: the `none` arm can barely clear 2.0%, so the fast-worker arm is the informative one. The P2 builds become claimable as their overlapping holds release; admit them then.
**Caveats:** admission is a priority judgement on today's holds, not a verdict on the P2 ideas.
