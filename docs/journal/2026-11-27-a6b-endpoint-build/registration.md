# Adoption registration: redesigned open-kernel endpoint elimination and dictionary (`a6b-endpoint-redesign`)

- **Claim:** `a6b-endpoint-build`
- **Status:** Frozen before i9-10900K workstation data (RULES C1a, C2, C6); awaits one independent review by a different agent before queueing the workstation run.
- **Feature flag:** `a6b-endpoint-redesign` (opt-in in `Cargo.toml`; default build unchanged)
- **Primary evidence & premises:**
  - `docs/journal/2026-11-26-explore/a6b-combined-redesign.md` (#1046, `05c13b98`)
  - `examples/a6b_combined_probe.rs`
  - `docs/journal/2026-11-24-algorithm-level/rauzy-census/results-2/README.md`
  - `frontier/DECODER.md` (coupled scanner/worker/reader baseline)
  - `docs/team/PERFORMANCE-POLICY.md`

## 1. Mechanism and scope

In `src/open/endpoint.rs` (with fused strip hand-off in `src/open.rs::Kernel::finish`), when `INDUCE && !self.wide(ns)` (`3 * ns + 3 <= 65_536`) under `--features a6b-endpoint-redesign`:

1. **Fused single-pass setup (`setup_soa_64k`):** replaces the separate `strips_raw`/`image_order` pass in `src/open.rs` and two-pass node setup in `endpoint.rs` with one pass over `strips_raw` that writes packed `u64` `image_order64` (`to:20 | width:20 | sid:16`), populates node widths and `u128` literal words, and links domain order/gaps directly with branchless sentinel-0 appends.
2. **Safe zero-bounds-check 64K SoA induction (`eliminate_inline15_soa_64k`):** stores `links: Box<[NodeLinks; 65536]>` (`512 KiB`), `widths: Box<[u32; 65536]>` (`256 KiB`), and `words128: Box<[u128; 65536]>` (`1 MiB`) lazily once per worker thread in `Endpoint`, indexed by `usize::from(u16)` (zero `unsafe`). Encodes words of length `1..=15` inline in `u128` (`-54.6%` rule allocations on wikidata) and handles `aw == bw` (`36.98%` of steps) with direct in-place replacement. Preserves `Wide` fallback (`3 * ns + 3 > 65_536`), `SPIRALS = true` contraction (`widest > CERTIFIED_WIDTH`), `dense-cap-precheck`, and `--features mtf-fold` composition.
3. **Generation-tagged L1d-hot dictionary (`dict_code_u128`):** uses reusable `contents[4096]` (`16 KiB`) and long-word-only `pairs[1024]` (`8 KiB`) generation-tagged caches (eliminating per-block `24 KiB` zero-fill), extracts short `<= 15`-byte words directly from `u128`, caches node state outside the inner band loop, packs `bottom` into dead `(prev1, next1)` `u16` links, and gates diagnostic `Counts` updates in the band loop behind `#[cfg(any(test, feature = "census"))]`.

### Memory and system coupling note (`frontier/DECODER.md`)

- `Fixed64kSoa` reserves `1.75 MiB` virtual memory per worker on first Narrow block; only `0..nodes` (`~273 KiB` average, `~420 KiB` peak on wikidata) is faulted and touched, and it is dead before `A6c`/`A7`.
- At `k = 1`, `C-A6b` (`216.0 ms` on i9) is directly exposed on the critical path; probe `comb_nobc_soa_gen` measured `A6b -27.4%..-28.8%` and `C+E -5.4%..-7.6%`, projecting `+48.7..+67.7 ms` (`+3.73%..+5.18%` of `1,306 ms` 1W wall).
- At `k = 8`, scanner `S` (`184.4 ms`) and workers (`185.0 ms`) co-limit (`frontier/DECODER.md`), so `A6b` savings (`~8.2..9.1 ms/worker` projected) are masked in shipped 8W wall unless paired with an ideal/improved scanner (`S ideal` / `scan-oracle`). This registration therefore tests **both** shipped end-to-end wall at `k = 1, 8, 10` (guarding against L3/RSS regression at `k = 8, 10`) and worker-busy (`wkr_busy`) / `S`-paired wall at `k = 8`.

## 2. Experimental design (i9-10900K workstation)

- **Machine:** Louis's i9-10900K workstation (20 MiB shared L3, pinned/unpinned standard bench harness, queued via `bench/workstation-queue.txt` after independent C6 review and green CI).
- **Arms:**
  - `base`: shipped `Open` (default features)
  - `cand`: redesigned `A6b` (`--features a6b-endpoint-redesign`)
  - At `k = 8`, also run paired against `S ideal` (zero-copy scan oracle, warm paired-oracle protocol `docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md`) if shipped `S` binds `k = 8` wall, so worker-side `wkr_busy` and unmasked 8W gain are directly observable.
- **Corpora:**
  - Primary: `wikidata x4` (`~839 MB` decoded output, SHA-256 verified)
  - Non-wikidata guard (`PERFORMANCE-POLICY.md`): `text` corpus and dense-band fallback check (verifying zero regression when Open refuses or takes mixed routes).
- **Thread grid:** `k in {1, 8, 10}` on `wikidata x4`; `k in {1, 8}` on `text`.
- **Method:** `n = 15` paired interleaved repeats per `(corpus, k)` with alternating arm order (`AB`/`BA`), logging wall time, `wkr_busy` (from trace/CPU accounting), peak RSS, and output SHA-256 on every repeat. Report geometric mean of paired ratios and 95% CI (plus non-overlapping median/CI check).

## 3. Frozen adoption decision rule (C2 — never relaxed)

Matches `frontier/claims/a6b-endpoint-build.toml` verbatim. Adopt `a6b-endpoint-redesign` as the production default **only if all three criteria pass**:

1. **Exactness:** 100% byte- and CRC-identical output on all unit tests (`src/open/{checks,b5,dense,spirals,mux}.rs`), fuzz smoke, `wikidata x4`, and non-wikidata text/dense corpora (including composition with `--features mtf-fold`).
2. **Memory safety:** zero new `unsafe` and Miri clean (`miri_a6b_endpoint_redesign`, `miri_open_kernel_small_columns`, and all existing Miri suites).
3. **Hardware performance on i9-10900K:**
   - On `wikidata x4` at `k = 1`: 1W wall drops by **`>= 3.0%`** (`>= 39 ms`, non-overlapping 95% CI);
   - On `wikidata x4` at `k = 8`: 8W worker-busy (`wkr_busy`) drops by **`>= 3.5%`** (`>= 6.5 ms/w`);
   - Zero wall regression (**`<= +1.0%`**) at `k = 8` or `k = 10` on `wikidata x4` (confirming the `1.75 MiB` virtual / `~273–420 KiB` touched per-worker SoA arrays do not regress `k = 8–10` L3/RSS), and zero regression (`<= +1.0%`) on `text` / non-wikidata fallback per `PERFORMANCE-POLICY.md`.

If any criterion fails: **STOP/KILL** this registered build, keep the default unchanged, and record a scoped row in `frontier/dead-ends.tsv` naming the exact failing criterion and evidence bundle digest. Any subsequent rescue (e.g., shrinking per-worker SoA footprint if L3/RSS regresses at `k = 8–10`) requires a distinct follow-on claim and fresh registration before data.

## 4. Independent review (C6)

- **Author:** `antigravity`
- **Reviewer:** pending (one independent review by a different agent before queueing the i9 workstation run).
