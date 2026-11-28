# Zero-table AVX2 ring CRC-32: i9-10900K adoption run against its merge base (registration)

*Frozen 28 November 2026 before any new data, under claim [`crc32-avx2-ring-rescue`](../../../frontier/claims/crc32-avx2-ring-rescue.toml). This is an adoption registration (RULES C6): it decides whether the kernel merges into the default decoder on evidence other than CI's paired bench (PERFORMANCE-POLICY.md §2 rule 1), so it needs one review by an agent other than its author before its ready tag is set. C2 is never relaxed after the data is seen. CI run 46054981092 (the predecessor's STOP) is not evidence for this rule.*

## 1. Why a second attempt, and what changed

PR #1179's no-prefetch arm (8-slot AVX2 `vpshufb` ring feeding 8-lane `vpclmulqdq`, zero-table Barrett tail; `src/crc.rs` only, no heap allocation, a 1 KiB stack ring) was exact and B5-sound, and stopped on its frozen whole-decoder gate only on a peak-RSS point bound of 1.005× (delta `2026-11-27-crc32-avx2-ring-barrett-build-2.md`). The RSS calibration census (`docs/notebook/2026-11-28-rss-bound-calibration.md`) shows that bound fails on 27 of 27 identical-binary CI runs at 2–4 threads: it sat below the instrument's own noise. Its CI runner (Zen 4) also cannot see the mechanism, an Intel single-port Port-5 saving (CRC stage 0.52 → 0.29 c/B on wikidata). This attempt therefore measures where the gain lives, against a memory rule calibrated before the data.

The candidate is #1179's `src/crc.rs` (blob `b67187a1`, identical at its evaluated head `af34b183` and its rebased head `8cb68208`) with only `# Safety` sections added to its three `unsafe fn` entry points, on master `31c35662`. No other file under `src/`, `Cargo.*`, `examples/ab.rs` or `examples/station_balance.rs` differs; the recipe refuses to run otherwise.

## 2. What is compared, and how (VOID if any check fails)

- **Head:** this checkout, whose `src` tree must equal the pinned candidate tree. **Base:** master `31c35662`, in a private worktree; it must be an ancestor of the head, with equal `Cargo.toml`, `Cargo.lock`, `examples/ab.rs`, `examples/station_balance.rs`.
- Rust 1.99.1, release profile as shipped (baseline x86-64, fat LTO), no `RUSTFLAGS`. The head's `ab` must contain `mibz2::crc::avx2_ring8_update` and the base's must not.
- i9-10900K, unrestricted 10C/20T, no CPU quota. Inputs: `wikidata ×4` (`0b820d14…ce08e`) and `ab gen … 4` `text`, `records`, `random`, `runs`, `pattern`, `onebyte`.
- Parity: every cell decodes in both arms with SHA-256 equal to libbz2's and the same hash between arms; every timed sample checks size and hash.
- Timing: `ab run --arms mibz2,mibz2@<head ab>` (bench 3.9.0), 16 paired rounds, held pin on core 0, 100 ms pre-spin, **2 s settle** (a 0.2 s settle lands in HWP's ramp hysteresis: `docs/notebook/2026-11-28-i9-clock-modes-verdict.md`), `--max-busy 1.5`. Cells: `wikidata ×4` at 1, 4, 8 and 10 workers; each guard corpus at 1 and 8.
- Diagnostic only: 1T `station_balance` (5 rounds × 5 passes, pinned, alternating order) on wikidata, text, records, random, reporting S and W per arm.

*Re-pinned 28 November after the first attempt was VOID (no data; evidence 0390e19f): base moved from `0179ca9d` to `31c35662`, the master the candidate now sits on, so the run measures the change as it would merge. Rule and grid unchanged.*

Estimated time: about 30–40 minutes with cached inputs, plus two builds.

## 3. Decision (frozen)

All ratios are head/base paired geometric means with two-sided 95% t-intervals (`bench/mtf_fold.py::paired_ratio_ci`) over the 16 rounds, for wall time and for peak RSS alike (never a ratio of arm medians). **PASS** only if every rule holds, otherwise **STOP**:

1. **A paid whole-decoder gain:** `wikidata ×4` at 1 worker, wall upper bound ≤ 0.985 (at least 1.5% faster beyond noise). At one worker the worker station is nearly the whole cost, so this is where a worker-only saving must show.
2. **No primary loss:** no `wikidata` or `text` cell slower beyond noise (lower bound > 1.000) at any count (PERFORMANCE-POLICY.md §2 rule 2).
3. **Bounded secondary losses:** no `records`, `random`, `runs`, `pattern`, `onebyte` cell with wall point > 1.020 *and* lower bound > 1.000.
4. **Memory, calibrated (census §5 rule 2, zero-heap change):**
   - every k = 1 cell: RSS paired GM ≤ 1.005, or median paired delta ≤ +256 KiB (k = 1 null SD ≤ 0.005 on the i9; a real per-worker slab shows at t > 17);
   - `wikidata`/`text` at k ≥ 4: RSS paired GM ≤ 1.030;
   - secondary guards at k = 8: RSS paired GM ≤ 1.050.

**Predictions (not gates):** W/byte falls ~5% on wikidata at 1T (CRC is 0.23 of ≈ 4.5–5 c/B of worker cost); `wikidata`/1 wall 0.95–0.98; k = 4 similar or smaller; k = 8 0.97–1.00 (the knee is a scanner/worker tie, `frontier/DECODER.md`), k = 10 ≈ 1.00 (scanner-bound); RSS ≈ 1.00 everywhere.

## 4. What follows

- **On PASS:** the PR merges once its CI is green and CI's paired bench shows no regression under PERFORMANCE-POLICY.md §2 rules 2–4, with memory judged by census §5 rule 1 (every 1T cell RSS paired GM ≤ 1.010 or ≤ +0.5 MiB; 2T/4T ≤ 1.10). CI's rule 1 (a primary gain on CI) is replaced by rule 1 above, because CI's runners cannot see an Intel Port-5 saving; that substitution is why this is an adoption registration.
- **On STOP:** the PR is closed unmerged, the claim is marked dropped, and a scoped row goes in `frontier/dead-ends.tsv` naming this registration. If rule 1 alone fails while the diagnostic W falls, the advantage is masked rather than absent, and a rescue that targets it (fusing CRC into RLE1, `forward-rle1-crc`) stays open on that evidence.
- **VOID:** a failed check in §2; no verdict, the run may be repeated once fixed.
