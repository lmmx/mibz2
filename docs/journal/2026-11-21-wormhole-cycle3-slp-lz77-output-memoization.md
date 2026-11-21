# Wormhole Cycle 3 — The Grammar-to-LZ77 Output-Memoization Bridge (`OM-Fox`) & Agol–Hass–Thurston Quotienting

*21 November 2026 · Branch `research/carbon-wormhole` · Continuing directly from [Cycle 2](2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff.md)'s `Break 2`*

---

## 1. Explore & Triage: Bridging Cycle 2's Quasideterminant Flattening Cliff

Cycle 2 proved that pushing filtered Fox–Morse collapse $\mathcal{M}_\Lambda$ from $\Lambda = 16$ down to $\Lambda \in \{8, 4, 2, 1\}$ cuts critical rows $m_\Lambda$ by `2.06–3.27×` (`edges` down to `133.8–379.1 KiB`) and halves `>16B` walk steps, **but hits `Break 2`**: eager SLP flattening into `self.spelling` + wasted `inline16` slots on $>16\text{ B}$ words inflates dictionary memory from `53.9–117.6 KiB` to **`632.9–821.3 KiB`** (`61–76%` single-use `width = 1` strings), alongside $O(H_\Lambda)$ induction step growth.

Cycle 3 bridges `Break 2` by chaining three existing results from surface topology and grammar compression:
1. **Link 3.1 (Multiplicative / Euclidean Interval Pairing):** Zorich (*Ann. Inst. Fourier* 46:325–370, 1996) multiplicative Rauzy–Veech cocycle + Agol, Hass & Thurston (*J. AMS* 19(2):461–477, 2006, §3) Euclidean interval-pairing quotienting, collapsing adjacent tandem reductions (`prev0[x] == y` or `prev1[y] == x`) in $O(1)$ list splices and $O(\log q)$ binary-doubling SLP rules (`pow_word`).
2. **Link 3.2 (Short/Long Algebraic Truncation of `inline16`):** Words with $2 \le |W| \le 16$ expand directly into 16-byte `inline16` slots (`17.1–47.0 KiB` at $\Lambda=16$, fitting inside `32–48 KiB` L1d!), evicting all $|W| > 16$ words from `inline16` into 8-byte `(slp_root, tail_byte)` handles.
3. **Link 3.3 (SLP-to-LZ77 Real-Time Output Self-Sourcing):** Rytter (*Theor. Comput. Sci.* 302:211–222, 2003, Theorem 1: every SLP of size $g$ induces an LZ77 parse of $\le g$ phrases via first occurrences) + Gasieniec, Kolpakov, Potapov & Sant (*DCC 2005*, output-referential grammar decoding): during `emit`, each internal SLP node memoizes its first emitted offset `rl->first` directly inside `out[0..at)` and copies subsequent occurrences via `memmove(out + at, out + rl->first, rl->len)`, reducing `self.spelling` to **`0` bytes** across **all** $\Lambda \in \{1, 2, 4, 8, 16\}$.

### Candidate Profile (`METHOD.md` §3a): `OM-Fox<Lambda>`

- **Mode:** Reorientation of word representation (static flattened dictionary $\to$ dynamic output-self-referential SLP/LZ77 bridge) + filtration threshold test.
- **Displaced obligation:** `100%` of `self.spelling` + `self.offsets` (`38–609 KiB`), `stage_spell`, `inline_words` pass, and wasted $>16\text{ B}$ `inline16` slots; plus `m_{16} - m_\Lambda` critical hops at $\Lambda < 16$.
- **Replacement principle:** Short words ($\le 16\text{ B}$, `91–95%` of walk steps) emit from L1d-compact `inline16`; long words ($>16\text{ B}$) memoize SLP nonterminals directly inside `out[0..at)` (Rytter 2003 / Gasieniec et al. 2005).
- **New paid obligations:** Post-order tagged stack traversal + historical `out[0..at)` reads (`900 KiB` span) on the `5–9%` $>16\text{ B}$ steps; $O(H_\Lambda)$ `Endpoint` linked-list work & node arenas at $\Lambda < 16$.
- **Nearest prior art:** `First-occurrence writer` (`CONTEXT.md` §4 row 6, which ran stack-based first-occurrence expansion on `100%` of steps in `Cells` and lost `1.79×`; `OM-Fox` isolates it to the `5–9%` $>16\text{ B}$ cold path).

---

## 2. Exact Theorems & Verification

### Theorem 3.1 (Zorich / AHT Quotienting vs Pre-BWT `RLE1` — `proved`; `verified` on `26,495` cases + `30` block decodes)

Whenever $x = \mathrm{prev}_0(0)$ and $y = \mathrm{prev}_1(0)$ satisfy $w_x \le w_y$ and $\mathrm{prev}_0(x) = y$ (symmetrically $w_y < w_x$ and $\mathrm{prev}_1(y) = x$), cutting $x$ and inserting it after $y$ leaves $(x, y)$ at the active ends of both lists for $q = \lfloor w_y / w_x \rfloor$ consecutive steps, collapsing $q$ iterations into one width remainder $w_y \bmod w_x$ and $\le 2\lfloor\log_2 q\rfloor$ binary-power SLP rules $w_x \cdot w_y^q$.
- **Empirical `RLE1` Suppression Discovery (`measured`):** On all 6 authenticated `.bz2` Wikidata blocks, `unaccel_steps -> accel_steps` changes by **`0` steps on 5/6 blocks** and **`2` steps (`21,295 -> 21,293`) on `block-143.L`**, because `bzip2`'s pre-BWT `RLE1` stage already caps monochrome byte runs before BWT sorting! Zorich/AHT quotienting rescues adversarial `O(n)` run-spiral inputs (`pattern`, `rle_spiral`), but buys `0.0%` on admitted Wikidata blocks.

### Theorem 3.2 (Zero-`spelling` Output-Memoized SLP Exactness — `proved`; `verified` on `26,495` small/multi-cycle cases + `30` block decodes)

In `OM-Fox<Lambda>`, every internal SLP rule $r$ visited completely (`at + step_len <= n`) writes its exact expansion into `out[pos .. pos + r.len)` before recording `r.first = pos`, so every later visit copies an already-written, immutable prefix sub-slice of `out[0..at)`. Together with clamped non-memoizing fallback when `at + step_len > n` and `repeat(n, out)`, `OM-Fox<Lambda>` is **byte-for-byte identical** to `Plain::invert` across all primitive, periodic, and multi-cycle columns (`26,495 / 26,495` small + random multi-cycle cases, and `30 / 30` 6-block decodes at $\Lambda \in \{16, 8, 4, 2, 1\}$), with **`0 B` `self.spelling`** (`saving 548.1 KiB` on `block-0` at $\Lambda=1$) and **`17.1–47.0 KiB` `inline16`** at $\Lambda=16$.

---

## 3. Gate G5 Paid-Cost Confrontation (`g5_cycle3_om_fox.c`)

- **Gate G2 (`om_fox.toml`):** **PASS** (`check_manifest.py`).
- **Safety:** `gcc -O1 -g -mavx2 -fsanitize=address,undefined` passed cleanly across all 5 arms × 6 blocks (`30` byte-exact checks against `plain_invert`).
- **Fixed Gate G5 Rule (set prior to run):** Pass iff `OM-Fox<Lambda> / Open058 <= 0.95x` on `>= 4/6` blocks and `<= 1.00x` on `6/6` blocks with lower scratch footprint.

### 15-Round Rotated Confrontation (`-O3 -march=native`, medians in µs)

| Block | `Open058<16>` (baseline) | `OM-Fox<16>` (`/058`) | `OM-Fox<8>` (`/058`) | `OM-Fox<4>` (`/058`) | `OM-Fox<2>` (`/058`) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | **1,206.4** | 1,286.2 (`1.066×`) | 1,450.5 (`1.202×`) | 1,679.0 (`1.392×`) | 2,056.2 (`1.704×`) |
| `block-2.L` | **1,482.1** | 1,596.8 (`1.077×`) | 1,803.2 (`1.217×`) | 2,180.6 (`1.471×`) | 2,617.4 (`1.766×`) |
| `block-85.L` | **1,512.7** | 1,589.8 (`1.051×`) | 1,901.1 (`1.257×`) | 2,335.0 (`1.544×`) | 3,138.1 (`2.074×`) |
| `block-143.L` | **2,685.8** | 2,794.6 (`1.041×`) | 3,252.7 (`1.211×`) | 4,035.5 (`1.503×`) | 5,042.6 (`1.877×`) |
| `block-210.L` | **2,355.0** | 2,470.1 (`1.049×`) | 2,715.1 (`1.153×`) | 3,195.0 (`1.357×`) | 4,185.8 (`1.777×`) |
| `block-207.L` | **1,745.5** | 1,868.6 (`1.071×`) | 2,129.4 (`1.220×`) | 2,503.8 (`1.434×`) | 3,214.3 (`1.842×`) |
| **Scratch WS** (excl. `900K` `out`) | **`4,730.1 KiB`** | `4,612.9 KiB` | `5,012.1 KiB` | `5,673.8 KiB` | `10,705.9 KiB` |

**Gate G5 Verdict: FAIL (`0 / 6` blocks `<= 1.00x`).**

---

## 4. Learn: What `Break 3` Proves, and the Exact Missing Link for Cycle 4

`g5_cycle3_om_fox.c` is a textbook `METHOD.md` §7 result — **it closes two doors cleanly and exposes the true multi-worker bottleneck of `Open 0.5.8`:**

1. **Why `OM-Fox<16>` lost `4.1%–7.7%` despite `0 B` `spelling` (`cheap to make, dear to use`):**
   At $\Lambda = 16$, `self.spelling` is only `37.8–56.8 KiB` (100% L2-resident!) and built once over `1,326–3,635` words. Replacing one flat L2-hot `memcpy` in `emit` with a tagged SLP stack walk + random `memmove` reads across the `900 KiB` `out` buffer adds `+60–120 µs` of branch mispredictions and L2 misses (`4.1%–7.7%` loss) while saving only `117 KiB` out of **`4,730 KiB`**!
2. **Why $\Lambda < 16$ (`8, 4, 2`) loses unconditionally even with `0 B` `spelling`:**
   Even with dictionary flattening completely removed (`0 B` `spelling`), lowering $\Lambda : 16 \to 8 \to 4 \to 2$ slows the whole kernel to **`1.15–1.26×` $\to$ `1.36–1.54×` $\to$ `1.70–2.07×`** AND grows per-worker scratch from `4.73 MiB` to **`5.01–10.71 MiB`**, because `Endpoint`'s 24-byte linked-list `Node`s + 5 auxiliary index arrays cost ~15× more per added strip than `emit` saves per removed thin row!
3. **The Exact Structural Break (`Break 3`) $\implies$ Cycle 4's Target:**
   Look at `Open058<16>`'s **`4,730.1 KiB` (`4.62 MiB`)** per-worker scratch footprint (`5.5 MiB` with `out` = **`38–55 MiB` across 7–10 workers on the i9-10900K's `20 MiB` shared L3 cache!**):
   - **`~1,700 KiB`** sits in `q_buf[256]` (256 separate heap `Vec<u32>`s thrashing the Skylake 64-entry L1 dTLB during `3.1–8.1 MiB` of `Vec16` writes, followed by random scatter RFO writes into `edges[dst]`),
   - **`~1,600 KiB`** sits in `Endpoint`'s 5-array + 24B-`Node` linked-list scratch (dead before `wiring` even starts!), and
   - **`~50%` of runtime** sits in `emit`'s single-chain `MLP = 1` pointer walk (`edges[j]`), where previous multi-lane attempts (`#675`, `BP4K` 8-lane) lost at 8 threads on the i9 because staging pages copied `87.5%` of `out` twice (`+1.8 MiB` L3/DRAM traffic) and flooded the L3 ring with 64 concurrent streams!

   **Cycle 4** attacks `Break 3` directly via **two mathematical links**:
   1. **Dihedral / Time-Reversal Bidirectional Meet-in-the-Middle Orbit Decomposition** (walking $g^{-1}$ forward from `(mark, 0)` AND $g$ backward from `(mark, n)` simultaneously inside `out[0..n)` with **zero staging buffers and zero extra `out` copies**, escaping Carbon Depth Theorem 6's unknown-offset surcharge!), tested head-to-head against its `+4m` dual-edge table cost; AND
   2. **Exact `I_0` Pigeonhole Prefix-Arena + Disjoint-Lifetime Memory Overlay + Fenwick/Radix-Free Direct `Endpoint` Compaction**, collapsing per-worker scratch from `4.73 MiB` into **`< 1.1 MiB`** so all 7–10 workers fit inside the i9-10900K's `20 MiB` shared L3!

---

## 5. Reproducibility

```bash
BLOCKS_DIR=/home/antigravity/admitted-bwt python3 -u docs/journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization/verify_cycle3.py
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization/om_fox.toml
gcc -O3 -march=native -Wall -Wextra -Werror docs/journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization/g5_cycle3_om_fox.c -o /tmp/g5_c3 && /tmp/g5_c3 /home/antigravity/admitted-bwt 15
```
