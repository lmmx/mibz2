# Wormhole Cycle 7 (Round 2, Cycle 2 of 5) — Move D Under $\mathsf{BMCP}$: Closing $\Phi$/Toehold Bootstrap, Bijective Lyndon Cycles, and Backward SA-IS LMS Sectioning

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole`  
**Latest `origin/master` SHA:** `d425d772c0a0e6b82e9626b36bbfa4b28a6c9613` (`mibz2 0.5.9`)  
**Cycle:** 7 (Round 2, Cycle 2 of 5)

---

## 1. The Question Forced by Cycle 6's Gap

Cycle 6 (`docs/journal/2026-11-21-wormhole-cycle6-bmcp-floor-and-gap.md`) showed that Dieted `Open` meets the `2,048 KiB` 10-worker L3 floor (`479–831` L3 misses/block) and single-cursor random-access floor ($Q_{\mathrm{rand}} = m_{16}$), but chains `100%` of its `25,533–94,739` `edges` L2 misses serially ($\mu = 1$), sitting `4×–8×` above the $\mu$-MLP lower bound while leaving only `~270 KiB`/worker of L3 headroom at $P = 10$ workers (ruling out `900 KiB` 1-pass staging).

Move D of `docs/journal/2026-11-21-the-shape-of-the-rh-proof.md` proposes escaping LF's single-mark offset barrier by **building global text order via another construction**:
- **D1:** r-index **$\Phi$ and toehold sampling** (Gagie–Navarro–Prezza *JACM* 2020; Kempa *SODA* 2019);
- **D2:** **Bijective BWT and Chen–Fox–Lyndon factorisation** (Mantaci–Restivo–Sciortino 2007; Gil–Scott 2012);
- **D3:** **SA-IS run backwards** via L/S/LMS suffix types (Nong–Zhang–Chan *IEEE TC* 2011).

Cycle 7 tests all three in `100%` safe Rust (`verify_cycle7.rs`, `#![forbid(unsafe_code)]`) across small cases and the six authenticated blocks, keeping all six $\mathsf{BMCP}$ cost coordinates strictly separate.

---

## 2. Three Theorems on Move D's Global-Order Constructions

### Theorem 7.1 — The $\Phi$/Toehold Acquisition Inversion (`proved` + `verified` on `14,204` small `(L,p)` cases + `6/6` blocks in `verify_cycle7.rs`)

In the $r$-index, $\Phi(\mathrm{SA}[x]) = \mathrm{SA}[x-1]$ is piecewise affine on the *text-ordered* run-tail partition $\{\mathrm{SA}[\mathrm{tail}_j]\}_{j=1}^r$. Starting from $I_0 = (\text{records}, C, p, n)$ where only $\mathrm{SA}[p] = 0$ is given:

1. **Bootstrap Necessity:** By Theorem 6.2 (`4-Hamming Splice`), no sub-$r_1$ probe subset of $I_0$ determines $\mathrm{SA}[x]$ for non-marked boundaries. Acquiring exact boundary/anchor text coordinates via **Weight-Only Rauzy–Veech induction** (`boundary_text_positions` in `primitives.toml`, carrying only integer lengths `path_len: u32` with zero words and zero SLP rules) requires eliminating all $r$ maximal runs (`S_RV = 92,571 .. 216,769` serial doubly-linked pointer splices, matching MONODROMY record 82 identically).
2. **Forced 24-Byte `Node32` & L2/L3 Cost Inversion (`proved` + `measured`):** Because $r \in [51{,}184,\; 128{,}732] > 21{,}842$, total RV nodes $N_{\mathrm{nodes}} \ge r + 2$ exceed `u16::MAX` on blocks 85, 143, 207, 210 (`51,186–128,734` active strip nodes plus gap nodes), requiring 24-byte `u32`-linked nodes (`2,199.3–5,531.5 KiB` scratch $\gg 2{,}048\text{ KiB}$ L3 share on `6/6` blocks). Inside the serial elimination loop, weight-only RV incurs:
   - **`93,934 – 335,586` L2 (`256 KiB`) misses** (**`2.61× – 3.77×` MORE than Dieted `Open`'s entire `emit` walk**),
   - **`19,195 – 74,242` L3 (`2,048 KiB`) misses** (**`24× – 132×` MORE DRAM misses** than Dieted `Open`), and
   - **`2,508.8 – 6,365.1 µs`** native safe-Rust time just to acquire the anchor offsets (`phi_toehold_anchors.toml`).

---

### Theorem 7.2 — Bijective BWT Single-Cycle Lock & Giant Lyndon Factor Obstruction (`proved` + `measured` via Duval 1983 on `6/6` blocks)

1. **Standard `.bz2` Cycle Count:** All 6 authenticated blocks are primitive (`LF` has **`1` cycle** on `6/6` blocks); converting cyclic $(L, p)$ into Mantaci–Restivo–Sciortino's Bijective BWT requires splicing $\mathrm{LF}$ at the lexicographic ranks of the Chen–Fox–Lyndon suffixes of $T$, which by Theorem 6.2 requires global cycle order.
2. **Oracle Lyndon Imbalance (`measured`, `verify_cycle7.rs`):** Even under a free oracle supplying the exact Duval (`1983`) Chen–Fox–Lyndon factorisation $T = L_1 \cdots L_c$ (`c = 9..19` factors across the 6 blocks), **a single giant Lyndon factor spans `34.59% – 93.79%` of the entire 900 KB block** (`86.06%` on `block-85`, `93.79%` on `block-143`, `75.87%` on `block-210`, `76.02%` on `block-207`, reflecting the Golomb–Dickman constant $\lambda \approx 62.4\%$ for random permutations). Thus even oracle BBWT cannot reduce serial depth below `0.76–0.94 m` on 4 of the 6 blocks.

---

### Theorem 7.3 — Exact $O(r)$ BWT-Domain SA-IS LMS Identity & Section Dominance (`proved`; `verified` on `2,012 / 2,012` exhaustive primitive BWTs + `6 / 6` 900 KB blocks)

Let $T \in \Sigma^n$ be any primitive string ($|\text{cycles}(\mathrm{LF})| = 1$). Define cyclic S-type at text position $t \in \mathbb{Z}_n$ by $T[t..] <_{\mathrm{lex}} T[t+1..]$, L-type by $T[t..] >_{\mathrm{lex}} T[t+1..]$, and cyclic LMS (Left-Most S-type, Nong–Zhang–Chan 2011) by $\text{S-type}(t) \land \text{L-type}(t-1)$.

1. **Exact BWT-Space Formula (`proved`):** Because row $x$ represents rotation $T[\tau(x)..]$ and $y = \mathrm{LF}(x)$ represents $T[\tau(x)-1..]$, lexicographic order of rotations equals row-index order:
   $$\mathrm{S\text{-type}}\big(\tau(x)-1\big) \iff \mathrm{LF}(x) < x, \qquad \mathrm{LMS}\big(\tau(x)-1\big) \iff \big(\mathrm{LF}(x) < x\big) \;\land\; \big(\mathrm{LF}(\mathrm{LF}(x)) > \mathrm{LF}(x)\big).$$
   Since $\mathrm{LF}$ is unit-slope affine on each of the $r$ maximal runs of $L$, the exact LMS subset $S_{\mathrm{LMS}}$ is computable in $O(r)$ interval intersections without knowing $T$ (`verified` to exact integer equality against Nong–Zhang–Chan's text scan across **`2,012 / 2,012` exhaustive small primitive BWTs** and **`6 / 6` 900 KB blocks** in `verify_cycle7.rs`).
2. **Strict Dominance by `Open<16>` (`proved` + `measured`):** Backward SA-IS induces L-type and S-type positions deterministically *only after* recovering the reduced text $T_1$, which is the cycle order of the LMS first-return map on $S_{\mathrm{LMS}}$. On all 6 authenticated blocks, $|S_{\mathrm{LMS}}| \in [304{,}973,\; 316{,}819] \approx 0.34 n$ — **`1.51× to 3.42×` LARGER than `Open<16>`'s retained set $m_{16} \in [89{,}874,\; 204{,}281]$** (`1.16–1.21 MiB` table vs `351–798 KiB`)! Why: SA-IS places an LMS cut at *every* lexical L$\to$S valley (`every ~2.9 bytes`), splitting repeated 40-byte JSON substrings (`"property":"P...`) into a dozen LMS segments even when all copies move together as a single thick BWT strip.

---

## 3. Six-Block Summary Table (`verify_cycle7.rs`, Safe Rust)

| Block | Dieted `Open` $m_{16}$ | Dieted `emit` $D_{\mathrm{past-}M_2}$ | **D1: $\Phi$-RV Steps** | **D1: $\Phi$-RV Scratch** | **D1: $\Phi$-RV L2 Misses** | **D1: $\Phi$-RV L3 (`2048K`) Misses** | **D1: Safe-Rust Time** | **D2: Lyndon Factors / Max%** | **D3: SA-IS $n_{\mathrm{LMS}}$ (`vs` $m_{16}$)** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `89,874` | `25,533` | `92,571` | `2,199.3 KiB` | `93,934` (`3.68×`) | `19,195` (`23.7×`) | `2,767.1 µs` | `16` / **`34.6%`** | `307,346` (**`3.42×`**) |
| `block-2.L` | `112,070` | `37,778` | `101,907` | `2,343.2 KiB` | `98,571` (`2.61×`) | `20,450` (`27.4×`) | `2,508.8 µs` | `19` / **`47.7%`** | `309,493` (**`2.76×`**) |
| `block-85.L` | `156,404` | `47,553` | `140,544` | `3,410.7 KiB` | `177,387` (`3.73×`) | `29,767` (`62.1×`) | `3,489.5 µs` | `17` / **`86.1%`** | `304,973` (**`1.95×`**) |
| `block-143.L` | `196,841` | `94,739` | `193,866` | `4,655.0 KiB` | `272,278` (`2.87×`) | `50,361` (`60.6×`) | `5,512.7 µs` | `12` / **`93.8%`** | `313,734` (**`1.59×`**) |
| `block-210.L` | `204,281` | `89,106` | `216,769` | `5,531.5 KiB` | `335,586` (`3.77×`) | `74,242` (`131.9×`) | `6,365.1 µs` | `16` / **`75.9%`** | `309,085` (**`1.51×`**) |
| `block-207.L` | `135,921` | `51,240` | `153,650` | `3,743.1 KiB` | `194,073` (`3.79×`) | `32,668` (`40.1×`) | `4,022.4 µs` | `9` / **`76.0%`** | `316,819` (**`2.33×`**) |

*Mandatory candidate ledger for D1 (`phi_toehold_anchors.toml`):* per-worker working set `2.20–5.53 MiB` (`+900 KiB` `out` $\to$ `31–64 MiB` at 10w, overflowing `20 MiB` L3); modelled DRAM traffic at 10w `~2.6–6.5 B/B`; acquisition cost `D = S_RV = 1.70–1.81r` dependent steps (`PASS` under `check_manifest.py`, **killed** by preprocessing cost inversion).

---

## 4. What Break 7 Proves, and Cycle 8's Exact Target

Move D's failure is **constructive**: it proves that **Dieted `Open<16>`'s $m_{16}$-state return table `edges[0..m)` is ALREADY the smallest, L3-resident global-order object** (`351–798 KiB` vs `2.2–5.5 MiB` for $\Phi$-RV and `1.2 MiB` for SA-IS LMS).

Therefore, the **only** viable wormhole into Cycle 6's $\mu$-MLP gap must extract $\mu \in \{4, 8\}$ independent walk chains **from `edges[0..m)` itself**, subject to:
1. **Zero extra bits in `edges[j]`** and **$O(1)$ cut detection**;
2. **Zero heap staging (`0 B` added to `S_mem`)**, staying strictly inside the `2,048 KiB`/worker 10-thread L3 budget;
3. **Exact handling of multi-cycle columns** (`Gate G1`).

Cycle 8 introduces **Zero-Bit Xor-Mask Cut Sets** $S_{\mathrm{cut}} = \{j < m : ((j \oplus \mathrm{mark}) \ \& \ \mathrm{cm}) == 0\}$ and confronts **1-Pass L3-Safe Bounded Staging (`1P-BoundedStage`)** against **2-Pass Work-Stealing Zero-Copy MLP (`2P-WS`)** and **Scout-Then-Emit Relay MLP (`Relay-MLP`)** in `100%` safe Rust against `origin/master` `OpenCells`.

---

## 5. Artefacts & Citations

- `verify_cycle7.rs` (`#![forbid(unsafe_code)]`), `phi_toehold_anchors.toml`, `cycle7_results.json`, `cycle7_output.txt`.
- **Citations:** K. T. Chen, R. H. Fox & R. C. Lyndon (*Ann. Math.* **68**, 1958); J.-P. Duval (*J. Algorithms* **4**(4), 1983); S. Mantaci, A. Restivo, G. Rosone & M. Sciortino (*Theor. Comput. Sci.* **387**, 2007); G. Nong, S. Zhang & W. H. Chan (*IEEE Trans. Comput.* **60**(10), 2011); J. Gil & D. A. Scott (arXiv:1201.3077, 2012); D. Kempa (*SODA* 2019); T. Gagie, G. Navarro & N. Prezza (*J. ACM* **67**(1), 2020).
