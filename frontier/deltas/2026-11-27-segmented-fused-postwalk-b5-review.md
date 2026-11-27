claim: segmented-fused-postwalk
by: claude-code
outcome: partial
evidence: PR #1183 (author head e12e0aa1; merged-with-master head 8121ebdc); src/bwt.rs; src/lib.rs; src/rle.rs; src/open.rs; src/cells.rs; src/cells_lazy.rs; src/runs.rs

**What was asked:** the independent B5 review that #1183 asks for, then integration under the claim's frozen stop_test.

**What I found (B5, adversarial review by an agent other than the implementer): PASS, no blocking objection.**
- *TLS hook (`FusedPostwalk`).* `enter_append` stores `*mut Vec<u8>`. The guard carries `PhantomData<&'a mut Vec<u8>>`, so the borrow checker stops the caller touching `out` (or aliasing it as `s.ll`) while the hook is armed. `take()` removes the pointer from TLS before it is dereferenced, and `Drop` restores the previous state on panic. Every wrapper on the merged tree (`OpenCells` in src/open.rs, `Cells` in src/cells.rs, `LazyCells` in src/cells_lazy.rs) hands the caller's own `out` to its `Segmented` fallback, and only as its last step. So no wrapper can consume a hook armed for another buffer. If the Open/Cells kernel writes the block itself, the state stays `Append`/`Owned`, `take()` returns `None`, and the unfused `finish` runs.
- *Fused AVX2 decoder.* I checked the capacity invariant `out_len + (avail - i) + 32 <= cap`. It is hoisted once per 4 KiB chunk and kept by each step: a literal stride moves `out_len` and `i` by 32 together; a run re-checks `need = run + (avail - (j+5)) + 32`; the scalar tail writes one byte per input byte. The fast loop reads only `p[..i+36]`, within `avail`. `grow_out` commits only initialised bytes (`set_len(cur_len)`) before `reserve_exact`/`reserve`, then refreshes `out_p`/`out_cap`. `RunUnfinished` sets `len = start`. The AVX2 reverse copy writes `dst[chunk-ri-32..chunk-ri]` with `ri + 32 <= chunk`, and its tail writes `dst[..chunk-ri]`. `get_unchecked(sg)` is safe because chain indices come from `first`/`next`.
- *Segment truncation.* `assemble_reversed` truncates the last backward segment; the fused path would truncate the last forward one. They agree because the room walk's segments partition the rows of a permutation, so the sum of segment lengths never exceeds `n` and no truncation ever happens. Either way the reads stay bounded by `b.len()`.
- *`clear_start = false`.* Only the private `Segmented::tt` keeps `START` bits. Every builder (`build_lf`, `build_one_pass`, `build_two_pass`, `runs::fill`) clears or fully overwrites `tt` before the next walk. The two `run_back_fused == false` fallbacks call `walk_back` on an un-walked, clean `tt`. Public `walk_back_room*` still clean up.
- Miri covers the scalar mirror and the hook (CI `miri` green at e12e0aa1). The AVX2 body is checked by the native parity test against `assemble_reversed_with + unrle + crc32` (boundary-straddling runs, 4 KiB/64 KiB edges, short cycles, `RunUnfinished`, `start > 0` rollback) and by fuzz-smoke.

**Evidence so far (CI paired bench, e12e0aa1 vs its merge base 66511da, EPYC 7763):** `text` 1T wall -5.0% (-8.6, -1.2), 2T -7.5% (-11.0, -3.9), 4T -3.8% (-9.2, +1.8); `wikidata` unchanged at 1/2/4T; `runs` -15.7% and `onebyte` -10..-12% at 2/4T; no secondary slower beyond noise; same bytes everywhere. Peak RSS: every cell ≤ +1.2% except `pattern` 4T +2.4%, not significant.

**Reading of the stop_test, written before the merged-tree bench (8121ebdc) reports.** Master moved `src/open*` and `src/runs/fold.rs` after e12e0aa1's run, so that run is not the exact-head evidence. The exact head's own CI bench decides, once, with no re-runs:
- *timing*, read "under PERFORMANCE-POLICY.md §2" as the stop_test says: `text` 1T 95% CI wholly below 1.000; no `wikidata`/`text` cell slower beyond noise; no secondary cell above +2% with its interval excluding no change;
- *peak RSS ≤ 1.02x*, read literally on every cell's point estimate, which is stricter than policy §4's 10% and is not relaxed.

PASS on both: merge, mark `done`. Otherwise mark `blocked` with the failing cell. A dead-ends row goes in only if `text` shows no primary win or a guard breaches beyond noise.

**What's next / caveats:** the CI runner is Zen 3, so i9 behaviour is unmeasured; the claim's projected ≥6% at 1T is a projection, not a bar. The `u64`-store `build_lf` (mechanism 2) was dropped by the author after the first Zen 3 read and is not shipped here.
