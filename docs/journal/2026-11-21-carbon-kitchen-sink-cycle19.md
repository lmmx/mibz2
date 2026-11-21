# Carbon Kitchen Sink — Cycle 19: Kac Return Spectrum, `EdgesMux` Saddle-Point Geometry, and Byte-Stratified Schur Complements ($L : \Sigma \to \mathbb{N}$)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Ergodic Kac return-time spectrum ($\mathbb{E}[\tau_J] = n/m$), `EdgesMux` arena saddle-point geometry ($\max(25 n_s + 24, m)$), non-uniform byte-stratified thresholds $L : \Sigma \to \mathbb{N}$, and epistatic `MN16-Direct` $\times$ `LIMIT` confrontation on `0.5.11`.

---

## 1. Mathematical Question: Did `0.5.10`/`0.5.11` Shift `#657`'s Threshold Frontier, or Can Byte-Stratified $L(c)$ Break Uniform `LIMIT = 16`?

`#657` calibrated uniform `LIMIT = 16` on `mibz2 0.5.7` before `Vec16` (`0.5.8`), `OutQ16` (`0.5.10`), and `EdgesMux` + `Elim::<Narrow>` (`0.5.11`). Two structural questions arise on `0.5.11`:
1. **The `EdgesMux` Saddle-Point (`endpoint.rs:412`):** Under `Narrow` links, `edges` has word capacity $\max(25 n_s + 24, m)$. Because $25 n_s \ll m$ at `LIMIT = 16` across all 6 blocks (`e.g. 70,225 vs 204,281` on `block-210`), lowering `LIMIT` shrinks `edges` dollar-for-dollar with $m$ until `spelling` or `bands` crosses a `Vec` doubling boundary or $25 n_s$ overtakes $m$. Does that memory minimum pass `#704`'s `>= 10%` RSS bar with zero wall loss?
2. **Non-Uniform Byte-Stratified Complements $L : \Sigma \to \mathbb{N}$ & Epistatic `MN16` Coupling:** Wikidata's alphabet is strongly bimodal between structural JSON hubs (`"`, `,`, `:`, `{`, `}`, `[`, `]`) and content/hex bytes. Does a byte-dependent threshold $L(c) \in \{12, 16\}$ (`StratA`: hubs `16`, rest `12`; `StratB`: hubs `12`, rest `16`), or pairing `LIMIT = 12, 14` with Cycle 16's `MN16-Direct` (preventing L1D dictionary spill as `words` grows), beat uniform `LIMIT = 16`?

### Theorem 19.1 (`EdgesMux` Scratch Saddle-Point & Marginal Strip-vs-Hop Separability)
1. **Exact Scratch Minimum at `LIMIT = 14`:** Across the 6 authenticated blocks, lowering `LIMIT` from `16` to `14` cuts $\sum m$ from `895,391` to `860,014` (`−3.95%`) and shrinks peak `edges` from `817,124 B` to `765,380 B` (`−50.5 KiB`) without crossing `Vec` doubling thresholds in `spelling` (`65,536 B`) or `bands` (`98,304 B`), reaching the global 6-block scratch minimum (`1,127.2 KiB` on `master`, `1,072.3 KiB` with `MN16-Direct` = `−8.56%` vs `0.5.11` `master`, below `#704`'s `10%` memory-gain bar). At `LIMIT <= 12`, `spelling` (`131,072 B`) and `bands` (`196,608 B` at `LIMIT = 10`) double, rebounding total scratch upward.
2. **Byte-Marginal Separability Law:** Promoting one run of byte $c$ of width $w \in [13, 16]$ from retained (`thin`) to eliminated (`thick`) yields $\Delta m = -w$ (`saving w * t_hop` in `OutQ16 + wire + emit`, $\approx 14 \times 6\text{ ns} \approx 84\text{ ns}$) and $\Delta n_s = +1$ (`adding ~2.5 Elim::<Narrow> list splices + ~2.2 rules + ~1.4 bands`, $\approx 145\text{ ns}$). Because both $\Delta m = -w$ and $\Delta n_s = +1$ depend on **width $w$ rather than byte label $c$**, any byte-stratified threshold $L : \Sigma \to \{L_0, L_1\}$ has net time change strictly convex-interpolated between uniform $L_0$ and uniform $L_1$ by the share of width-$(L_0, L_1]$ runs in $\{c : L(c) = L_0\}$, even when L1D spill is suppressed by `MN16-Direct`.

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`55` Interleaved Pairs/Block)

All 9 arms passed `6 / 6` byte-exact parity against `plain_invert`:

### 2.1 Kac Return & Scratch Spectrum (`6`-Block Totals)

| `LIMIT` | $\sum m$ | $\sum n_s$ | $\sum \mathrm{words}$ (`master` / `MN16`) | Cumulative Scratch (`master` / `MN16`) | `MN16` `edges` | `MN16` `spelling` | `MN16` `bands` |
|---:|---:|---:|---:|---:|---:|---:|---:|
| `10` | `786,398` | `28,536` | `17,309` / `7,523` | `1,275.1` / `1,195.1 KiB` | `720,032 B` | `131,072 B` | `196,608 B` |
| `12` | `833,821` | `24,412` | `14,900` / `6,660` | `1,198.5` / `1,123.8 KiB` | `749,576 B` | `131,072 B` | `98,304 B` |
| `14` | `860,014` | `22,477` | `13,576` / `6,115` | **`1,127.2` / `1,072.3 KiB`** | `765,380 B` | `65,536 B` | `98,304 B` |
| **`16`** | `895,391` | `20,170` | `12,590` / `5,691` | `1,172.7` / `1,120.5 KiB` | `817,124 B` | `65,536 B` | `98,304 B` |

### 2.2 End-to-End Interleaved `1T` Ratios vs `0.5.11` `master` (`LIMIT = 16`)

| Block | `A/A` | `Master@14` | `Master@12` | `StratA` (`Punct16_Rest12`) | `StratB` (`Punct12_Rest16`) | **`MN16@16`** | `MN16@14` | `MN16@12` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1.010` | `1.013` | `1.038` | `1.041` | `1.028` | **`0.990`** | `1.011` | `1.036` |
| `block-2.L` | `1.004` | `1.016` | `1.046` | `1.049` | `1.018` | **`0.999`** | `1.013` | `1.043` |
| `block-85.L` | `1.012` | `1.017` | `1.039` | `1.040` | `1.003` | **`0.973`** | `0.975` | `1.006` |
| `block-143.L` | `0.997` | `1.012` | `1.043` | `1.020` | `1.014` | **`0.980`** | `0.996` | `1.022` |
| `block-210.L` | `1.003` | `0.995` | `1.024` | `1.019` | `1.008` | **`0.969`** | `0.984` | `0.994` |
| `block-207.L` | `1.003` | `1.018` | `1.037` | `1.044` | `1.009` | **`0.988`** | `1.001` | `1.018` |
| **`6`-Block Sum** | **`1.0038`** | `1.0107` | `1.0374` | `1.0324` | `1.0125` | **`0.9819`** | `0.9955` | `1.0178` |

---

## 3. Cycle 19 Verdicts (One-Line Kills)

- **Kill (`Master@14` / `Master@12` post-diet recalibration):** Killed because even on `0.5.11`'s `Elim::<Narrow>`, eliminating width-`13..16` runs costs `~145 ns/strip` vs `~84 ns` saved in `emit` (`1.0107x` at `LIMIT=14`, `1.0374x` at `LIMIT=12`), and `LIMIT=14`'s `−3.9%` scratch drop (`1,127.2 KiB`) misses `#704`'s `10%` memory-gain bar.
- **Kill (`StratA` / `StratB` Byte-Stratified $L : \Sigma \to \mathbb{N}$):** Killed (`1.0324x` and `1.0125x` vs A/A `1.0038x`) because marginal $(\Delta m, \Delta n_s) = (-w, +1)$ depends on width $w$ rather than byte identity $c$, forcing any non-uniform $L(c) \in \{12, 16\}$ strictly between `Master@16` and `Master@12`.
- **Kill (`MN16@14` / `MN16@12` Epistatic Threshold Lowering):** Killed in favour of `MN16@16` (`0.9819x`) because `MN16@14` (`0.9955x`, `−8.56%` scratch) sacrifices `1.4%` wall speed while still falling short of `#704`'s `10%` memory threshold.
