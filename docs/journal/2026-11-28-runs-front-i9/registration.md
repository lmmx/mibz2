# `runs::front_avx2` rescue: i9-10900K adoption run against its master parent (registration)

*Frozen 28 November 2026, before any workstation data, under the claim [`runs-front-i9-rescue`](../../../frontier/claims/runs-front-i9-rescue.toml). This is an adoption registration (RULES C6). It decides whether #1238's final `front_avx2` merges into the default decoder, so one review by an agent other than its author is needed before its ready tag is set. C2 is never relaxed after the data is seen. The CI runs 46078359802 and 46086718747 are not evidence here; the claim says so.*

## 1. What is compared

- **Candidate:** the ready-tagged commit of this registration's PR. It is master at `BASE` plus #1238's final head `791ee415`'s `src/runs.rs` and `src/runs/tests.rs`, applied unchanged. That code came from the `runs-front-and-fill-build` handoff:
  - Pass 1 builds the literal and one-run bitmaps 64 symbols at a time, with a `u64` `vptest`.
  - The dead `Runs.per` is compiled out in production, so counting uses a `[u32; 256]` stack array and frees a register to hoist the step table.
  - The step table is a two-cycle 128-bit `(f0, f1)` pair with no `vpermq` in the hot loop. Removing `vpermq` is what fixed Attempt 1's Zen 3 regression.
- **Control:** `BASE = 4bb6591087c3d007ff508905446c1b333096aa76`, master when this was frozen. It includes #1117's scanner setup and #1279's fix to the quota check, which this recipe imports. The candidate was first drafted on `629222aa`; it was re-pinned before any data, because #1117 then touched `src/` and #1279 fixed a check that would have voided the run.
- **The recipe refuses to run (VOID)** unless `BASE` is an ancestor of `HEAD`, and unless the only files that differ under `src/`, `examples/`, `benches/`, `Cargo.toml`, `Cargo.lock` and `build.rs` are exactly `src/runs.rs` and `src/runs/tests.rs`. Recipe, test and this file differ too, but no binary reads them. The base is built from a `git worktree` at `BASE`, the candidate from the tagged checkout itself.
- **Arm:** `ab`'s `mibz2`, the production default, at every thread count. `runs::front` is reached through every path that takes runs:
  - `src/lib.rs:484`, `:665` and `:682` (sequential);
  - `src/parallel.rs:1650` (workers);
  - Wikidata's 233 full Open blocks and every Runs-routed text-like block (`docs/journal/2026-11-27-explore/runs-front-and-fill-dissection.md` §2).

## 2. Correctness and identity (VOID if any fails; never STOP)

- **Before this run:** #1238's two heads were green on all of CI, including Miri on the default build and with `+avx2,+bmi1,+bmi2,+lzcnt,+popcnt`, with 7/7 SHA-256 identical. The B5 review by an agent other than the implementer is a merge precondition (§5), not a run precondition.
- **On the workstation, the run is VOID if any of these fails:**
  - **Exact output:** every cell decodes in both arms with size and SHA-256 equal to C libbzip2's, untimed, before the timed sweep. Every timed sample checks size and the 64-bit hash.
  - **Codegen identity:** the candidate's `runs::front_avx2` (`nm`/`objdump`) contains at least one `vptest`, and the base's contains none. Both disassemblies are archived.
  - **Prerequisites:** the host is an i9-10900K with an unrestricted 10C/20T affinity and no CPU quota; Rust is 1.99.1; no `RUSTFLAGS` is set (baseline x86-64); the tree is clean; the two `ab` binaries differ.

## 3. Grid

- **Pairing:** `ab run --arms mibz2,mibz2@<cand>`. Each sample is a fresh process, with a 2 s settle (the i9-clock-modes verdict, `origin/evidence` `d5658cfb`, found the slow clock mode only at a 0.2 s gap: 0/48 slow at 2 s), the busy check at 1.5 cores, a 100 ms prespin, `--pin` at core 0's first CPU and `--machine-state` counters. There are 16 paired rounds. `ab`'s `arm_order` rotates the arms each round, so each arm goes first in 8 of them.
- **Wikidata ×4** (`0b820d14…ce08e`): 1, 6, 8 and 10 threads.
- **`ab gen` corpora at scale 4:** `text` (primary), plus `records`, `random`, `runs`, `pattern` and `onebyte` (secondary), each at 1 and 8 threads.
- **Size:** 16 cells and 512 timed processes. That is about 20–25 minutes plus three builds, under the 30-minute threshold of PERFORMANCE-POLICY.md §4a.
- **Why these cells:**
  - `front` is 132.4 ms of Wikidata's single-worker time (19.3 ms per worker at 8), and Pass 2 is 85–91% of it, so k = 1 is where the cut is largest.
  - At 8 and 10 threads the scanner and the workers tie, and the cut may be masked (`frontier/DECODER.md`). Those cells test for a paid gain or a loss, not for the mechanism.
  - The secondary corpora are guards. `runs`, `pattern` and `onebyte` take `front` too, with long-run profiles (`pattern` 10.6% in Pass 3).
- **Prediction, written down but not part of the rule:** Wikidata 1T about 0.97 and text 1T about 0.97–0.98, from the VM's −2.9..−4.5% and the smaller i9 share. 8 and 10 threads near 1.00. Secondary within ±2%. Peak RSS unchanged (no heap change, −1 KiB of `Runs`).

## 4. Decision (frozen)

The statistic per cell is the paired geometric-mean wall ratio candidate/base, with its 95% t-interval (`mtf_fold.paired_ratio_ci`). The median of paired peak-RSS ratios is the memory statistic. PERFORMANCE-POLICY.md §2 rules 1–4 apply, verbatim in effect, to this target-machine run. The result is **PASS** only if all four hold, otherwise **STOP**:

1. **Primary gain:** at least one primary cell (Wikidata at 1, 6, 8 or 10; text at 1 or 8) has its 95% upper bound below 1.000.
2. **No primary loss:** no primary cell has its 95% lower bound above 1.000.
3. **Bounded secondary losses:**
   - A secondary point ratio up to 1.050 is accepted and listed.
   - From 1.050 to 1.100 it is accepted only if a primary cell at the same thread count passes rule 1 with a gain of at least twice the loss.
   - Above 1.100 the result is STOP. Policy needs Louis's sign-off there, so this run cannot adopt.
4. **Memory:** the median paired peak-RSS ratio is ≤ 1.10 at every cell. This is the policy's line, and it calibrates the noise. The STOP on `runs-front-and-fill-build` came from a point bound of ≤ 1.005 that CI's RSS jitter alone exceeded (text 2T +3.6%, with no heap change).

Position-stratified ratios, bootstrap medians and `cycles_per_ref` are reported for every cell. They are diagnostic and gate nothing.

## 5. What each outcome does

- **PASS:** the registration's PR, which carries the code, merges into the default decoder under its exact-head gates. Those gates are:
  - green CI, Miri included;
  - CI's paired bench showing no primary loss beyond noise (policy rule 2);
  - the independent B5 review of `src/runs.rs` recorded with no open objection.

  This run then stands in for policy §2 rule 1, which CI's AMD runners cannot see for an Intel-specific kernel cut. The change goes into the next release, and since it changes the default path, RELEASING §3.5's i9 gate applies there.
- **STOP:** the PR is closed unmerged, with the code kept on its branch and the ready tag. A scoped row goes in `frontier/dead-ends.tsv` naming this registration. A rescue is allowed only as a distinct mechanism, such as #1238 Attempt 1's 256-bit `YMM_STEPS` (`d5e95794`) for Intel's single shuffle port. This arm is never rerun unchanged.
- **VOID:** fix the named prerequisite and run again at a new ready tag, saying why in a delta.

## 6. Why cheaper evidence won't do (§4a)

- **Static and instruction-level evidence:** it shows the hot loop is shorter (25 instructions and 118 B, against 27 and 142 B), but not that the wall time falls on the target machine.
- **The CI bench:** it runs on AMD Zen 3 and Zen 5. There `front` is under 10% of the 1T wall, and the two shuffle pipes already hide the change. Both heads' CI runs were neutral to slightly favourable, with no primary loss.
- **The Intel VM:** −2.9..−5.0% CPU there, but it is not the target machine, and PERFORMANCE-POLICY.md §2 requires C5 evidence for a gain CI can't show.
