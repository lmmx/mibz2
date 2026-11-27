# Registration: the same kill test, re-pinned to the disjoint-slab four-lane walk

- **Claims:** `forward-emission-run` (the recipe), `lanes-disjoint-slabs-build` (#1217, the implementation).
- **Kind:** kill test (RULES C1a), frozen 27 November 2026 before any i9 data on either arm. Not an adoption registration: a PASS changes no default and calls for a separate C6 adoption registration.
- **Why a new registration:** `registration.md` §4 says any departure from its §2 needs a new registration before data. §2 pins `cand` to "the prototype head" (#1110, `7a7a207a`). Since then #1217 replaced that implementation in `src/open/lanes.rs` (per-lane owned 32 KiB slabs, the inline `LONG` fast path, a non-inlined `run<K>`; zero `unsafe`, Miri clean). The #1110 arm is no longer in the tree. Running it on the i9 would decide the fate of code nobody would ship, so it is **superseded and not run** (disposition, not a verdict: no KILL, PASS or dead-end row is recorded for it).

## What changes (only the arms)

- **`base`:** `v0.5.17` (`01acd74d17bfea0ab891e75eb4fc2ef3838f5cd8`), default features: the released decoder.
- **`cand`:** the same commit, `--features forward-anchor-lanes`: the #1217 disjoint-slab walk.
- Both builds additionally with the trace and `scan-oracle` features for the `k = 8` `S ideal` cell, exactly as before.

One commit for both arms isolates the feature: everything outside the feature (including #1222's recalibrated `OpenCells` admission, which only the default route reads) is identical in the two builds. `src/` on master equals `v0.5.17`'s at the time of freezing.

## What does not change

Everything else in `registration.md` holds verbatim: §1's mechanism (as re-spelt by #1217), §2's machine, corpora, cells, 15 interleaved AB/BA paired repeats and per-repeat SHA-256; **§3's exactness gate and every threshold** (PASS: `r1` hi `<= 0.950`, `r8S` hi `<= 0.975`, `r8` hi `<= 1.010`, `text` 1W hi `<= 1.010`; KILL: `r1` lo `> 0.980`, `r8S` lo `> 1.000`, `r8` lo `> 1.020`; RESCUE otherwise); §5's forced open route at one worker. C2 applies.

A KILL here records a dead-end row naming this arm (`forward-anchor-lanes`, disjoint-slab phase A with contiguous stitch, #1217) and the bundle's digest; it leaves the vectored phase B open as §3 says.

## Data seen before freezing

Only non-decisional VM diagnostics (C5): #1217's paired `ab` on a 4-vCPU VM (about −8..−11% CPU at 1..4 threads against #1110's build) and CI's paired bench. No i9 data on either arm.
