claim: forward-lanes-mlp-and-prefetch
by: antigravity
outcome: done
evidence: docs/notebook/2026-11-28-forward-lanes-mlp-and-prefetch.md; frontier/dead-ends.tsv ("K=6/8 forward lanes (<=92 KiB bounded slabs) and edges[next] prefetch under forward-lanes-mlp-and-prefetch's frozen >=12.0% L3-cold gate")

**What was asked:** Probe `K = 4` (`#1312` bounded-slab `in-out` `4 KiB` chunk permutation, `4f3c9ac2`, `80 KiB/w`) vs `K = 6` (`88 KiB/w`) and `K = 8` (`92 KiB/w <= 128 KiB`) with and without 1-step `edges[next]` prefetch (`PREFETCHT0`/`PREFETCHT1`) and `edges+inline16` prefetch across all 233 Wikidata Open blocks + `runs.bz2` (`n = 7` interleaved repeats, CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID`, L3-warm & L3-cold `24 MiB` flush/block, `base` & `jcc`).
**What I found (unrelaxed C2 verdict: `STOP` on the `>= 12.0%` `base` L3-cold gate; real disjoint gain preserved):**
- **Exactness, scratch, & disassembly:** `4,680 / 4,680` byte+CRC parity checks vs `bwt::Plain` PASS (`233` Wikidata + `1` forced-Open `runs.bz2` × 10 arms × `base`/`jcc`); active scratch `80 / 88 / 92 KiB <= 128 KiB`; `0` `vzeroupper` / `0` `ymm` across all `Walk::run::<4/6/8>` functions (`M7`/`M8` emit `8` `prefetcht0`/`prefetcht1` and cut stack reloads `70 -> 57` vs `run::<8,0>`).
- **Best variants (`M7_k8_pf_t0` / `M8_k8_pf_t1`, `92 KiB/w`, `n = 7`):**
  - **`jcc` (PASSES `>= 12.0%` cold & warm):** L3-cold `A7` **`-16.57% [0.733, 0.949]` (`M7`) / `-17.12% [0.724, 0.949]` (`M8`)** (`-14.59% / -15.03%` E2E `A7+A8+A9`); L3-warm `A7` **`-18.49% [0.746, 0.891]` / `-15.69% [0.772, 0.921]`** (`-14.40% / -14.48%` E2E).
  - **`base` (DISJOINT FASTER, misses frozen `>= 12.0%` cold bar):** L3-cold `A7` **`-5.49% [0.922, 0.969]` (`M7`) / `-6.19% [0.908, 0.969]` (`M8`)** (`-4.70% / -5.25%` E2E); L3-warm `A7` **`-11.17% [0.865, 0.913]` / `-11.03% [0.859, 0.922]`** (`-9.45% / -9.81%` E2E).
- **Two true negative ablations:** (1) `K = 6/8` without prefetch (`M3`, `M6`) regresses `+3.7%..+5.6%` on `base` (LLVM rolls `for i in 0..K` for `K >= 6` vs unrolling `K = 4`); (2) prefetching `inline16` (`M9`) hurts vs `M7`/`M8` because `avg inline16 = 16.5 KiB` fits in L1d.
- **Root cause of `-6.2%` `base` cold vs `-17.1%` `jcc` cold:** `avg edges = 445.4 KiB` fits in L3 (`> 256 KiB` L2) and `#1312`'s invariant `4 KiB` chunk permutation + tail stitch is ~20% of `A7`; `M7`/`M8` hit identical absolute cold times on `base` (`282.7 ms`) and `jcc` (`280.6 ms`), while `M0` `jcc` was slower (`341.0` vs `301.0 ms`).
**What changed:** Closed `forward-lanes-mlp-and-prefetch` (`status = "done"`), wrote `docs/notebook/2026-11-28-forward-lanes-mlp-and-prefetch.md`, appended scoped `STOP` row to `frontier/dead-ends.tsv`; zero `src/` edits.
**What's next:** `K = 6/8` without prefetch and `inline16` prefetch are closed; `K = 8 + edges[next]` prefetch (`-6.2%..-18.5%` `A7`, `-4.7%..-15.0%` E2E at `92 KiB/w`) remains unclosed under a calibrated gate or once paired with `L6a` zero-stitch streaming (removing chunk-permutation overhead) and multi-worker i9 L3 contention.
**Caveats:** Shared KVM VM probe; single-thread LLC flush warms `445 KiB` `edges` into L3 early in the block.
