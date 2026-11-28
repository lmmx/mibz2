claim: forward-emission-bounded-slab-rescue
by: antigravity
outcome: progress
evidence: branch build/forward-emission-bounded-slab-rescue on origin/master b0b78871; src/open/lanes.rs (#![forbid(unsafe_code)], <= 80 KiB/worker Arena); cargo fmt & clippy PASS; 5/5 open::lanes::tests PASS; 3/3 Miri PASS (0 UB); objdump Walk::run<4..1> 501/423/284/201 insns 0 vzeroupper; 7/7 corpora AB_SHA256=1 libbzip2 parity EXACT (1W lazy & 8W mibz2); paired 9-round ab (base vs #1217 old_lanes vs bounded_4k): baseline 1W RSS 0.9970x (-0.09 MiB vs old +0.98 MiB), 8W RSS 1.0076x (+0.30 MiB vs old +7.69 MiB, -96.1% excess), 1W CPU 0.8863x (99.1% of old's gain), 8W CPU 0.8858x (1.7% faster than old); JCC-32B 1W RSS 1.0060x (+0.11 MiB vs old +1.15 MiB), 8W RSS 1.0220x (+1.29 MiB vs old +7.22 MiB, -82.1% excess), 1W CPU 0.9130x (97.4% of old's gain), 8W CPU 0.8829x (1.3% faster than old)

**What was asked:** eliminate the `+1.72 MiB` 1W / `+6.71 MiB` 8W peak RSS double-buffer overhead of `#1217`'s disjoint-slab four-lane forward walk strictly inside `src/open/lanes.rs` (`#![forbid(unsafe_code)]`, keeping `lanes::walk`'s signature unchanged per 23rd synthesis pass), passing `stop_test` Clauses 1 and 2 before C6 physical i9 adoption (Clause 3).

**What changed (`src/open/lanes.rs` only):**
- Replaced `Arena::pool` / `Arena::used` (~32 × 32 KiB = `~1.0 MiB/worker` live alongside `out`) with strictly bounded scratch (`<= 80 KiB/worker` worst-case, `~48 KiB` typical): `4 × 4 KiB = 16 KiB` active lane slabs (fitting inside half of L1d) plus a bounded `tails: Vec<u8>` capped at `16 × 4 KiB = 64 KiB`.
- During the 4-lane walk, whenever a lane fills its `4 KiB` slab for its current segment, it flushes that exact `4 KiB` chunk directly into `out` (already allocated by the caller, zero extra `900 KiB` buffer) and reuses the same slab. When a lane reaches a tagged anchor (ending a segment), its `< 4 KiB` tail bytes append to `arena.tails`.
- After all segments finish, full `4 KiB` chunks in `out[0..f * 4096]` are permuted in place into cycle order via cycle-following moves reusing `arena.slabs[0]` as the single `4 KiB` temporary buffer (`0 B` extra allocation, each chunk moved at most once), followed by a single backward `copy_within` pass (`dst >= src` invariant) inserting each segment's tail.
- Preserved `Walk::run<4..1>` compact safe codegen (`501 / 423 / 284 / 201` insns, `0 vzeroupper`), all multi-cycle periodic and invalid-edge fallbacks, and added a 40 KiB multi-slab periodic test plus strict `Arena` capacity assertions (`slabs.len() <= 4`, `tails.capacity() <= 64 KiB`).

**Verification (Clauses 1 & 2 PASS):**
- **Safety & exactness:** `#![forbid(unsafe_code)]`, `cargo fmt --check`, `cargo clippy --features forward-anchor-lanes --tests -- -D warnings`, `5/5` `open::lanes::tests::` PASS, **`3/3` Miri tests PASS (`0 UB`)** (`miri_forward_anchor_lanes`, `long_word_slab_boundaries_and_multi_slab_words`, `invalid_edges_fall_back_and_untag`), and **7/7 corpora `AB_SHA256=1` exact vs C `libbzip2`** (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte` at `1W` lazy & `8W` `mibz2`).
- **Paired `ab` (`n = 9` interleaved rounds on `wikidata.bz2`, `b0b78871` `base` vs `#1217` `old_lanes` vs `bounded_4k`):**
  - **Baseline `1W` lazy:** RSS `24.28 -> 24.19 MiB` (`0.9970x`, `-0.09 MiB` vs `#1217` `+0.98 MiB`); CPU `0.950 -> 0.839 s` (`0.8863x`, `99.1%` of `#1217`'s `0.8764x` gain).
  - **Baseline `8W` `mibz2`:** RSS `51.95 -> 52.25 MiB` (`1.0076x`, `+0.30 MiB` vs `#1217` `+7.69 MiB` = **`-96.1%` excess removed**); CPU `0.987 -> 0.876 s` (`0.8858x`, **`1.7%` faster than `#1217`** `0.9062x`).
  - **JCC-32B `1W` lazy:** RSS `24.21 -> 24.32 MiB` (`1.0060x`, `+0.11 MiB` vs `#1217` `+1.15 MiB` = **`-90.9%` excess removed**); CPU `0.866 -> 0.791 s` (`0.9130x`, `97.4%` of `#1217`'s `0.9113x` gain).
  - **JCC-32B `8W` `mibz2`:** RSS `51.29 -> 52.58 MiB` (`1.0220x`, `+1.29 MiB` vs `#1217` `+7.22 MiB` = **`-82.1%` excess removed**); CPU `0.930 -> 0.832 s` (`0.8829x`, **`1.3%` faster than `#1217`** `0.9154x`).

**What's next:** land this `--features forward-anchor-lanes` `src/open/lanes.rs` PR after full CI green, then update/review the C6 `bench-forward-adopt` registration for Clause 3 (`forward-emission-bounded-slab-rescue` thresholds) and run on Louis's physical i9-10900K.
