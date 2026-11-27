claim: forward-4lane-anchor-probe
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md, examples/forward_4lane_probe.rs, #1050

**What was asked:** Price paid sampled-anchor multi-lane forward wiring (`A6c`) and dynamic 4-lane forward emission (`A7`) end-to-end on `Open` against `shipped_bidi` (`>= 12%` gate) and `fwd_1lane` (`>= 8%` warm & cold gate, disjoint `[min..max]` on baseline x86-64 and `-C target-cpu=native`), reporting both vectored and contiguous-stitched modes (`#![forbid(unsafe_code)]`, zero `src/` edits).
**What I found:**
- **Structural & order-statistics census (233 admitted Open blocks of `wikidata.bz2`):** `single_cycle = 233/233`, `can_tag = 233/233` (`max_bits = 18`, `max_dict = 1,712` -> bit 31 free on 100% of blocks). Uniform BWT row-index sampling `(mark + k*m/S) % m` matches i.i.d. exponential order statistics and golden-ratio hashing: at `S = 16`, `eff_K = 3.153` (`4L% = 63.9%`, `L_max/(m/4) = 0.818x`, `41.6` ordered slices/blk), and slab continuation caps arena at `<= 992 KiB` independent of `S` (0 B extra output buffer in vectored mode).
- **PASS (`fwd_4lane_sampled_S16_vec`, 11,961 / 11,961 byte-exact across all 431 wikidata blocks + 12 synthetic edge cases, 7 interleaved repeats, steal-free CPU `schedstat`):**
  - Baseline x86-64 WARM: `231.80 [219.99..248.94]` ms (**`-30.99%` vs bidi**, **`-14.67%` vs fwd1**, disjoint FASTER vs both; 93% of oracle `215.59`).
  - Baseline x86-64 L3-COLD: `323.61 [307.79..342.39]` ms (**`-33.45%` vs bidi**, **`-29.19%` vs fwd1**, disjoint FASTER vs both; **99.95%** of oracle `323.46`).
  - Native WARM: `221.83 [215.91..231.70]` ms (**`-32.38%` vs bidi**, **`-23.52%` vs fwd1**, disjoint FASTER vs both; 92% of oracle `203.87`).
  - Native L3-COLD: `303.41 [295.48..320.38]` ms (**`-35.78%` vs bidi**, **`-33.43%` vs fwd1**, disjoint FASTER vs both; **97.4%** of oracle `295.66`).
  - Drop-in contiguous stitched `S16_stitch` (`12–24 ms` stitch): **`-27.02%..-31.53%` vs bidi / `-17.45%..-29.03%` vs fwd1** disjoint FASTER on native warm+cold and base cold (`-25.69% / -8.11%` base warm).
  - Modeled i9-10900K `A6c+A7` saving (VM diagnostic): `+208–235 ms` (`+16.0–18.0%` 1W wall, ~`+29–33 ms/w` at 8W) vectored; `+175–208 ms` (`+13.4–16.0%`) stitched.
**What changed:** Added `examples/forward_4lane_probe.rs` and `docs/journal/2026-11-27-explore/forward-4lane-anchor-probe.md`; marked `forward-4lane-anchor-probe` `done`; updated `frontier/claims/forward-emission.toml` with the concrete design and `frontier/surface.tsv` rows `C-A6c` and `E-walk`.
**What's next:** Admit and build `forward-emission` (`src/open.rs`, `src/open/outq.rs`) once `src/open.rs` is free behind `mtf-fold`, pairing its ~41.6 vectored slices/blk with `forward-rle1-crc-fusion`.
**Caveats:** VM 2-vCPU diagnostic (C5); `schedstat` CPU timer avoids concurrent-VM preemption spikes seen in wall `Instant` (4 ms tick quantization, ~0.8%). Real 1W/8W confirmation belongs in `forward-emission`'s frozen i9 registration.
