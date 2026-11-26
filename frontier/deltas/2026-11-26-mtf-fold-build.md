claim: mtf-fold-build
by: claude-code
outcome: done
evidence: branch build/mtf-fold; src/runs/fold.rs; tests/mtf_fold.rs; registration docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md (#960)

**What was asked:** build the MTF-into-construction fold behind research feature `mtf-fold`, exact, under the frozen registration; open the run's follow-on claim, not queue it.

**Premises checked first.**
- *#962 (runts):* every block of real size is Open, and the registration's figures are already over the 233 full blocks. The runts never reach the fold: it needs 4,096 symbols, and theirs are under 1,000.
- *#952 (cold stores):* K1 and K3 use no oracle. K2 pairs FS against S, and both read the same cold symbols store (~50 MiB). That is not #952's asymmetry, a worker-bound candidate against a scanner-bound baseline. It dilutes K2 toward 1, against the fold. Recorded in `mtf-fold-run`'s stop test; no flag.

**What I built:**
- *Two fronts that feed the first pass* (`src/runs/fold.rs`), `runs::front_scalar` and `runs::front_avx2` line for line. They write no records, no `counts`/`per`, and no cut at 127. Each run goes to a sink in `src/open.rs`, which merges adjacent runs of one byte, as `maximal` does, and calls `Pass::take`. The record count for `OpenCells::cap` is kept as `div_ceil(total, 127)`.
- *Out before n:* `decode_owned` takes the pooled buffer by the block's bound before Pass 2; the walk resizes it.
- *Exactness:* any block the fold doesn't finish goes the shipped way from the start (`Bailed`, or `NotOpen`, which is exactly the shipped `LazyCells` set).
- *Switch and counters:* `runs::fold::set_enabled`; `take_counts` gives folded, bailed and not_open.
- *Unsafe:* no new kinds. The AVX2 copy keeps `front_avx2`'s invariants and SAFETY comments, and writes less (no record stores). The scalar fold is safe code.

**Exactness verified (local, x86-64 with AVX2):**
- *Fronts:* each fold front against the shipped one on 12,000 synthetic blocks and the cap/span edges, scalar and AVX2. The maximal runs and record counts are identical, and a fold front gives up only where the shipped front errs or `front_avx2` gives up.
- *Open kernel:* against `front` + `invert_runs` on 2,000 synthetic blocks and 60 run columns, forced and production admission. The bytes are identical, and the fold finishes exactly where the shipped path opens.
- *decode_owned:* against the shipped decode on 400 blocks of up to 40k symbols, with CRCs and errors.
- *The parallel decoder, on and off:* the test corpora at -1 and -9, whole, a byte changed and cut, both kernels. One output and one `Error` in every case, libbzip2's bytes; 94 blocks folded, 0 bailed. `cargo test --features mtf-fold --lib` passes (199). Clippy `--all-features` is clean. MIRI_RESULT

**What changed:**
- *Code and CI:* `Cargo.toml` (the feature, `[[test]] mtf_fold`), `src/{runs.rs,runs/fold.rs,open.rs,bwt.rs,lib.rs}` (all cfg-gated: the default build's source is unchanged), `src/runs/tests.rs`, `tests/mtf_fold.rs`, and the CI steps in `test.yml` and `miri.yml`.
- *Frontier:* surface rows C-A6a and M; new claim `mtf-fold-run` (open).

**What's next:** `mtf-fold-run`. Port `scan-oracle` (S ideal) beside the fold. The fold must read the borrowed syms in Replay: `wants` gates on `b.syms.len()`, which is empty there. Then freeze the recipe and queue it. Its owned paths overlap this hold, so it is claimable only after this PR merges.

**Caveats:**
- *No timing here decides anything.* VM_TIMING
- *Departures in shape, not in bytes:* the symbols' buffer goes back to the scanner's pool after the block, not before invert; the output buffer is pooled by bound, not by n; and the trace's A5 span is empty in F, where `InvertStart..InvertEnd` holds A5 and A6 fused.
- *Fallback cost* on run-route blocks that aren't Open (one front pass wasted) is untimed; the run's text arm covers it.
