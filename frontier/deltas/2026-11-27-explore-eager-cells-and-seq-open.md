claim: explore
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/eager-cells-and-seq-open-dissection.md; examples/eager_cells_seq_open_probe.rs

## Verdict: STOP on both immediate pre-#1110 `t = 1..2` switch and standalone `src/cells.rs` micro-optimization (strict C2; two scoped rows added to `frontier/dead-ends.tsv`, no premature `idea` claim opened)

Dissected `src/cells.rs` (1,733 LOC), `src/cells_owned.rs` (348 LOC), `src/run_fed.rs` (830 LOC), and inline `Seq` (`t = 1`, zero thread overhead) + `t = 2, 3` worker pools (`Cells` vs `OpenCells` vs `OpenR2Cells` across all 7 CI corpora, 63/63 exact SHA-256 parity vs `libbzip2`, base x86-64 / JCC-32B / `+ #1110 forward-anchor-lanes`, warm & L3-cold, true subprocess `VmHWM` peak RSS + steal-free CPU, zero `src/` edits).

## Key Findings
1. **Track 1 — Pre-`#1110` immediate `t = 1..2` switch FAILS frozen gate (`STOP`):**
   - On default shipping **baseline x86-64** warm inline `Seq (t = 1)` `wikidata`, shipped `Seq(Cells)` (`0.8505 s`, `28.6 MiB`) still beats `Seq(OpenCells)` (`0.9255 s`, **`+12.08%`**) and `Seq(OpenR2Cells)` (`0.9013 s`, **`+9.21% [+2.38%, +16.50%]` DISJOINT SLOWER**), reproducing physical i9-10900K `k = 1` (`1.123×` wall, `27.7 MiB` RSS in `run-20261125T022939Z` / `bce3914d`) to 0.2 pp.
   - Under L3-cold (`64 MiB` flush/block), `Cells`'s `6.01 MiB` `cells.buf` suffers `+70..115 ms` cold-miss penalty, erasing its warm edge on Base (`+1.09%`) and losing `-7.27%..-8.00%` DISJOINT on JCC-32B.
2. **Strategic Discovery 1 — `#1110` (`forward-anchor-lanes`) completely flips `t = 1, 2, 3`:**
   - With `--features forward-anchor-lanes`, `OpenCells` beats `Cells` at **every** thread count on both shipping Base x86-64 (`t = 1 -2.01%`, `t = 2 -4.96%`, `t = 3 -10.89%` DISJ; RSS `0.61× / 0.55× / 0.46×`) and JCC-32B (`t = 1 -9.08%`, `t = 2 -14.44%` DISJ, `t = 3 -13.66%` DISJ; RSS `0.61× / 0.54× / 0.44×`). Unifying `t = 1, 2, 3 -> OpenCells` (retiring `EagerCells`) should wait until `#1110` lands, then run a single physical i9 `k = 1, 2, 3` gate.
3. **Strategic Discovery 2 — Actionable RSS footgun in `R2_2tier_m22` for `post-1080-open-admission`:**
   - Shipped `OpenCells` admits 233/233 full `wikidata` blocks, keeping `Segmented` (`tt` + `seg.bufs`) at `0 B` (`16.3 / 28.5 / 32.3 MiB` RSS at `t = 1, 2, 3`).
   - `R2_2tier_m22` refuses 9 full `wikidata` blocks (`4 MidRef + 5 Dense`, `m/n <= 0.257`), forcing `Segmented` to permanently allocate `tt = 7.03 MiB` + `seg.bufs = 4.14 MiB` per worker (`24.1 / 41.3 / 48.0 MiB` RSS = `+7.8 / +12.8 / +15.7 MiB` vs shipped `OpenCells`, projecting `+55–65 MiB` at 8W on i9-10900K!). `post-1080-open-admission` should raise the dense `m/n` cap to `0.26` over `0.10 <= r/n < 0.17` (or free rare fallback buffers) so `wikidata` stays 233/233 in `Open` (`0 B` `Segmented`) while `text` (`m/n >= 0.30`) still routes to `Segmented`.
4. **Track 2 — `src/cells.rs` & `src/run_fed.rs` micro-dissection (`STOP`):**
   - `src/run_fed.rs` (830 LOC) is 100% `#[cfg(test)]` research-only code (`src/lib.rs:134`).
   - `Cells`'s `1.75–2.4×` RSS comes from `cells.buf = 6.01 MiB` + permanent `Segmented` fallback (`tt 3.43 MiB + seg.bufs 5.14 MiB`) allocated by 37 refused full `wikidata` blocks (`r/n >= 0.10` or budget).
   - `objdump -d` + `writer_owned` phase attribution: `induce_cells` is already an ultra-tight 228-insn, 0-call, 0-bounds-check, 0-JCC-crossing, 8-`prefetcht0` loop consuming `~85–88%` (`445–485 ms`) of `Cells` invBWT over `20.46M` steps (`~22–24 ns/step`). `write_owned` (`~71 ms` CPU = `~12–15%` of invBWT, `~8–10%` of file CPU) invokes GOT-indirect `memcpy@GLIBC_2.14` twice per arena child in `owned_scan` (`6.34M` rules), giving a `~2.5–3.5%` ceiling (`< 5.0%` frozen gate) that is strategically dominated by post-`#1110` `OpenCells` unification.
