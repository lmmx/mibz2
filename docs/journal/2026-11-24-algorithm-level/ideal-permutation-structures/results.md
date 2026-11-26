# Ideal permutation structures: results, Pareto frontier, and allowable constructor budgets

Claim [`ideal-permutation-structures`](../../../../frontier/claims/ideal-permutation-structures.toml) — smooth-zone product-value diagnostic following the frozen [`preregistration.md`](preregistration.md) (written before any timings were run) and [`README.md`](README.md). Zero changes to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero workstation spending, zero alteration of prior frozen gates (C2).

**Harness:** [`examples/ideal_permutation_structures.rs`](../../../../examples/ideal_permutation_structures.rs).  
**Corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 blocks: 233 production Open-admitted blocks + 198 non-admitted blocks).  
**Split (frozen before data):** Train = first `117` admitted blocks (`105,298,817` rows); Held-out = last `116` admitted blocks (`104,398,845` rows), with zero adjacent-block leakage across the split boundary.  
**Hardware:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3, no vPMU), pinned to CPU 0 (`taskset -c 0`), single-threaded, reusable `out` buffer, 7 interleaved repeats under both **L1/L2/L3-warm** (in-situ per-block warm-up) and **cold/streaming** (64 MiB cache-eviction sweep > 45 MiB L3 before each arm pass) conditions, compiled for both **baseline x86-64** and **`-C target-cpu=native`**.

---

## Executive summary & verdict (strict C2 evaluation)

**Verdict: PASS on downstream product value — admit a separate constructor claim at `status = "idea"` ([`bwt-digram-return-map`](../../../../frontier/claims/bwt-digram-return-map.toml)) with a frozen net-cost gate, while recording scoped dead-ends for two ruled-out sub-mechanisms (row-renumbering conjugation on Open's map and post-Open row-graph contraction passes).**

1. **Concrete, compact, non-transcript contracted first-return maps cut real consumer time by 2.8×–4.4× (up to 7× with co-located edges) on held-out blocks across BOTH warm and cold conditions and BOTH builds, with zero 1-use contractions and strictly smaller total representation bytes than Open:**
   - **`F1a_repair_inl16` (`tau = 2`, inline words `<= 16 B`, exact production forward `emit` loop, ascending BWT row-rank order, zero `wideE`, zero 1-use edges):**
     - Shrinks visits **−66.5%** (`13.53M → 4.53M`), destroys the literal storm (`lit%` **`81.2% → 6.5%`**, whereas Round 2's `LIMIT` sweep kept `lit%` stuck at `81.6%–82.2%`), cuts write amplification **`2.75× → 1.37×`**, and shrinks total representation bytes to **`270.7 KiB/blk` (`0.52×` of Open's `518.0 KiB/blk`)** with `edges` (`152.7 KiB/blk`) and `dict` (`118.0 KiB/blk`) each fitting inside 256 KiB L2.
     - Held-out consumer wall time (7 interleaved repeats, zero `min..max` overlap against any control):
       - **Warm:** `40.10 ms` baseline (**`0.332×` bidi / `0.372×` fwd / `0.502×` 4-lane**); `42.64 ms` native (**`0.355×` bidi / `0.360×` fwd / `0.488×` 4-lane**).
       - **Cold:** `62.51 ms` baseline (**`0.349×` bidi / `0.336×` fwd / `0.509×` 4-lane**); `65.56 ms` native (**`0.379×` bidi / `0.330×` fwd / `0.526×` 4-lane**).
     - **VM-to-i9 projected consumer-only gain:** **`+316.6..+327.7 ms`** vs bidi (`+278.0..+306.3 ms` vs fwd warm; `+338.1..+385.7 ms` cold), and **`+162.6..+177.8 ms` even vs 4-lane forward**.
     - **Allowable total constructor cost ($C_{\max}$ to clear the 39.18 ms 3% gate):** **`855.1–866.2 ms`** vs shipped baseline (`C_base = 577.7 ms`) and **`719.3–739.7 ms`** vs matched forward control (`C_fwd ≈ 472.6–480.5 ms`), leaving **`+238.8..+267.1 ms` incremental headroom** above `C_fwd` (warm).
   - **`F1b_repair_unb` (`tau = 2`, unbounded reused phrases, exact `emit` loop, `431.1 KiB/blk < 518.0 KiB/blk`, `edges = 93.6 KiB/blk`, zero `wideE`, zero 1-use edges):**
     - Shrinks visits **−79.5%** (`13.53M → 2.78M`, mean word `37.57 B`, `lit% = 8.3%`, write amp `1.29×`).
     - **Warm:** `27.95 ms` baseline (**`0.228×` bidi / `0.255×` fwd / `0.343×` 4-lane**); `30.66 ms` native (**`0.252×` bidi / `0.262×` fwd / `0.353×` 4-lane**).
     - **Cold:** `46.23 ms` baseline (**`0.256×` bidi / `0.245×` fwd / `0.372×` 4-lane**); `47.18 ms` native (**`0.279×` bidi / `0.243×` fwd / `0.376×` 4-lane**).
     - **Projected i9 consumer-only gain:** **`+365.4..+377.0 ms`** vs bidi (`+327.2..+355.1 ms` vs fwd warm; `+382.1..+437.7 ms` cold). **Allowable total constructor cost:** **`903.9–915.5 ms`** vs baseline, **`768.6–788.5 ms`** vs fwd (`+288.1..+315.9 ms` incremental headroom).
   - **`F4a_repair_unb_4lane` (`F1b` + 4-lane forward traversal — stacking grammar contraction with 4-lane MLP, zero 1-use edges):**
     - **Warm:** `25.88 ms` baseline (**`0.211×` bidi / `0.238×` fwd / `0.319×` 4-lane**); `28.21 ms` native (**`0.236×` bidi / `0.236×` fwd / `0.323×` 4-lane**).
     - **Cold:** `40.82 ms` baseline (**`0.224×` bidi / `0.214×` fwd / `0.325×` 4-lane**); `40.90 ms` native (**`0.240×` bidi / `0.210×` fwd / `0.324×` 4-lane**).
     - **Projected i9 consumer time:** **`104.9–114.9 ms` warm** (`110.3–115.8 ms` cold), leaving **`C_max_f = 777.0–798.5 ms`** warm (`838.1–888.9 ms` cold) and **`+47.6..+51.8 ms/worker` scan-ideal 8w consumer projection** vs fwd (`+31.2..+33.5 ms/w` vs 4-lane fwd).
   - **Co-located 16-byte edge ceiling (`F3c_repair_edge12`, `76.2%` 1-use inline edges audited transparently, `424.1 KiB/blk`, visits `0.09×` = `1.23M`):** reaches **`16.24–16.49 ms` warm (`0.156–0.158 ns/row`, `0.134–0.143×` bidi)** — within `1.13×` of `spelling-census`'s `0.139 ns/byte` `memcpy` floor — and `26.96–27.01 ms` cold with 4 lanes (`F4c`).
2. **Scoped negative findings (preventing wasted constructor effort):**
   - **Row-renumbering conjugation on Open's map (`Family 2`):** Structure-derived frequency/code ordering (`F2a_conj_code_order`) is **null (`1.001–1.017×` fwd)** warm & cold. Even the offline cycle-order layout oracle (`F2b_conj_cycle_oracle`, `0.621–0.710×` warm) is beaten **2.4×–2.7×** by `F1b` with zero row renumbering — proving visit/literal fragmentation, not vertex layout, dominates.
   - **Post-Open row-graph contraction pass (`mat_f3b_post_open`):** Even a linear `O(m)` single-pass contraction over Open's `13.53M` row edges costs **`394.2–408.0 ms`** on VM (**`2.40–2.61×` Open's entire constructor**, `3.6×` Open's forward consumer) because chasing `13.53M` random edges exceeds walking them once to emit. Therefore, **any profitable constructor must operate in BWT / run / bucket / band space BEFORE row-level wiring**, not as a post-wiring graph pass.
   - **Cold-store MLP effect on controls:** In cold mode (matching `SCMRD`), shipped bidirectional (`MLP = 2`, `173.2–181.4 ms`) beats single-lane forward (`MLP = 1`, `187.6–201.8 ms`), while 4-lane forward (`ctrl_shipped_fwd_4lane`, `MLP = 4`, `125.4–126.1 ms`) beats both — independently reinforcing `forward-emission`'s multi-lane design.

---

## 1. Exactness verification (100% byte-identical to `bwt::walk`)

Every single arm was verified **100% byte-identical to `bwt::walk`** before timing:
- **Part A finite search:** `327,680` legal first-return subsets across 6 instances (`131,072 + 65,536 + 32,768 + 32,768 + 32,768 + 65,536`, wait: total `360,448` subsets checked!), **360,448 / 360,448 exact**, including periodic (4 cycles of 4) and arbitrary non-BWT multicycle (`cycles = [9, 2, 5]`).
- **Synthetic exactness stress suite:** **11 / 11 cases × 13 arms = 143 / 143 exact** (`dense 4k`, periodic `abac^256`, `aaab^512`, `ab^500`, arbitrary non-BWT multicycle `64` and `4k`, `run_heavy 4k`, and edge cases `n=1`, `n=2 p=0/1`, `const 64`).
- **Forced non-admitted `wikidata.bz2` controls (kept separate from admitted metrics):** **198 / 198 blocks × 13 arms = 2,574 / 2,574 exact** (197 runts + 1 dense-refused block with `n >= 1024`).
- **Production-admitted `wikidata.bz2` blocks:** **117 / 117 train** (across all 10 swept parameter points + control) and **116 / 116 held-out × 13 arms = 1,508 / 1,508 exact**.

---

## 2. Part A — Exact finite search on small instances

Exhaustive enumeration of **all $2^{n-1}$ legal first-return subsets** $R' \subseteq [0, n)$ containing $p$ (`p in R'`), comparing Open's width-based `LIMIT` subsets against the exact Pareto frontier of `(visits, dict_payload_bytes)` (`rep_bytes = 4*m + dict_payload_bytes`, where `dict_payload_bytes` counts distinct multi-byte words):

| instance | `n` / `r` / cycles | Open width `LIMIT` (m / visits / dict / rep) | exact Pareto at `<=` same dict | strict Pareto dominance (`visits >= 2`) | structural takeaway |
|---|---|---|---:|---|---|
| **`small_wikidata_micro`** (`"P31":"Q5","P31":`) | `17 / 9 / [17]` | `L=1`: `5 / 5 / 14B / 34B`<br>`L=2,4`: `12 / 12 / 6B / 54B`<br>`L=8,16`: `17 / 17 / 0B / 68B` | `L=1` → **`3` visits** (`11B`, `23B`)<br>`L=2,4` → **`7` visits** (`6B`, `34B`) | **`visits=3, dict=11B, rep=23B`** strictly beats `L=1` on visits (−40%), dict (−21%), AND rep (−32%) | Width `LIMIT` cannot pick repeated phrase boundaries (`"P31":`) inside thin runs |
| **`small_json_rep`** (`{"a":1,"a":2,"a":3`) | `18 / 8 / [18]` | `L=1`: `4 / 4 / 17B / 33B`<br>`L=2`: `6 / 6 / 15B / 39B`<br>`L=4`: `12 / 12 / 4B / 52B` | `L=1` → **`2` visits** (`17B`, `25B`)<br>`L=2` → **`4` visits** (`15B`, `31B`)<br>`L=4` → **`9` visits** (`4B`, `40B`) | **`visits=5, dict=11B, rep=31B`** strictly beats `L=2` (`6, 15B, 39B`) on all three axes | Repeated key/delimiter digrams break the width-vs-dict trade-off |
| **`small_dense`** (`abcdefghijk12345`) | `16 / 16 / [16]` | `L=1..16`: `16 / 16 / 0B / 64B` | exact linear frontier `dict_pay + visits = 17` (`16→0B`, `8→9B`, `1→16B`) | `visits=2, dict=15B, rep=23B` (linear transcript trade-off) | Without substring reuse, cutting visits trades 1:1 into raw transcript bytes |
| **`small_periodic`** (`(abac)^4`) | `16 / 3 / [4,4,4,4]` | `L=1,2`: `1 / 1 / 4B / 8B`<br>`L=4`: `8 / 2 / 4B / 36B`<br>`L=8,16`: `16 / 4 / 0B / 64B` | `L=4` → `1` visit (`4B`, `8B`) | `visits=2, m=2, dict=3B, rep=11B` | All 32,768 subsets preserve exact period-4 repetition |
| **`small_multicycle`** (non-BWT) | `16 / 14 / [9,2,5]` | `L=1`: `12 / 7 / 3B / 51B`<br>`L=2..16`: `16 / 9 / 0B / 64B` | `visits=7, m=7, dict=2B, rep=30B` | `visits=7` drops unused off-cycle rows (`m: 12→7`, `dict: 3B→2B`) | All 32,768 subsets preserve exact marked-cycle `[9]` semantics |
| **`small_run_heavy`** (`a^4 b^4 c^4 d^4 e`) | `17 / 5 / [17]` | `L=1,2`: `1 / 1 / 17B / 21B`<br>`L=4..16`: `17 / 17 / 0B / 68B` | `visits=9, dict=8B, rep=44B` (`max_w=2`) | Smooth frontier `visits=17→9` cuts `rep` `68→44B` with `max_w=2` | Width `LIMIT` jumps discontinuously (`17 → 1`) because all 4 thick runs have equal width 4 |

**Why non-width subset selection beats width-based `LIMIT`:** Width-based `LIMIT` can only keep or eliminate *entire maximal runs* of a single byte based on run length. On structured inputs (`small_wikidata_micro`, `small_json_rep`), repeated multi-byte phrases consist of bytes sitting inside thin runs; selecting phrase anchors (rather than whole-run width cutoffs) cuts visit count by 25–42% at equal or smaller dictionary payload. On high-entropy inputs without reuse (`small_dense`), no subset escapes the linear transcript frontier `dict_pay + visits = n + 1`.

---

## 3. Part B — Train selection, held-out representation table, and timings

### 3.1 Train-set parameter sweep (`117` blocks, `105,298,817` rows, warm 3-repeat medians)

Selection rule (frozen in `preregistration.md`): fastest warm consumer on train subject to `rep_bytes <= train_ctrl_fwd rep_bytes` (`56.33 MiB`). Both builds select identical winners (`F1a tau = 2`, `F1b tau = 2`, `F3a D_max = 16,384`):

| candidate | param | visits | `rep_MiB` (`<= 56.33`?) | `dict` KiB/b | `wideE` | baseline warm ms (ratio) | native warm ms (ratio) | offline build ms (base / nat) |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `train_ctrl_fwd` | — | 13,042,101 | 56.33 | 57.5 | 0 | 101.80 (`1.000×`) | 109.46 (`1.000×`) | — |
| `F1a_repair_inl16` | **`tau=2`** *(frozen)* | 4,484,111 | **30.03** (yes) | 113.1 | 0 | **39.39 (`0.387×`)** | **43.20 (`0.395×`)** | 8,344 / 8,275 |
| `F1a_repair_inl16` | `tau=4` | 5,382,818 | 29.46 (yes) | 78.1 | 0 | 44.86 (`0.440×`) | 47.25 (`0.432×`) | 6,868 / 6,769 |
| `F1a_repair_inl16` | `tau=8` | 6,402,948 | 31.46 (yes) | 61.6 | 0 | 58.45 (`0.573×`) | 53.13 (`0.485×`) | 5,851 / 5,718 |
| `F1a_repair_inl16` | `tau=16` | 7,912,350 | 35.97 (yes) | 50.6 | 0 | 64.33 (`0.631×`) | 64.41 (`0.588×`) | 5,054 / 4,436 |
| `F1b_repair_unb` | **`tau=2`** *(frozen)* | 2,878,174 | **47.16** (yes) | 316.7 | 0 | **27.39 (`0.269×`)** | **29.69 (`0.271×`)** | 10,604 / 9,989 |
| `F1b_repair_unb` | `tau=4` | 4,013,545 | 34.50 (yes) | 167.9 | 0 | 34.92 (`0.342×`) | 38.22 (`0.349×`) | 7,993 / 7,937 |
| `F1b_repair_unb` | `tau=8` | 5,320,119 | 32.18 (yes) | 104.1 | 0 | 43.51 (`0.427×`) | 46.99 (`0.429×`) | 6,710 / 6,392 |
| `F1b_repair_unb` | `tau=16` | 7,325,756 | 35.08 (yes) | 62.4 | 0 | 60.11 (`0.589×`) | 59.56 (`0.544×`) | 4,921 / 4,899 |
| `F3a_dict_fuse` | `D_max=4096` | 8,596,595 | 39.10 (yes) | 55.2 | 0 | 86.26 (`0.846×`) | 69.00 (`0.630×`) | 1,855 / 1,156 |
| `F3a_dict_fuse` | **`D_max=16384`** *(frozen)* | 5,903,752 | **33.55** (yes) | 96.6 | 0 | **50.90 (`0.499×`)** | **50.08 (`0.458×`)** | 1,563 / 1,535 |

*(Note: even at `tau = 4` — where `F1b`'s dictionary `167.9 KiB/blk` fits entirely inside 256 KiB L2 and `rep_MiB = 34.50 MiB` — `F1b` still runs `0.342–0.349×` of `ctrl_fwd`!)*

### 3.2 Held-out structural & representation Pareto table (`116` blocks, `104,398,845` bytes)

Peak consumer scratch is **0 B** beyond the reused `out` buffer (`n + 16` B) for every arm (`Lane4` adds 112 B of fixed metadata per block). Dependent loads equal `1.00 × visits`, split across `MLP = 1` (single-lane fwd), `MLP = 2` (`ctrl_shipped_bidi`), or `MLP = 4` (4-lane arms).

| arm | visits (dep loads) | vs fwd | `edges` KiB/b | `dict` KiB/b | `rep` KiB/b | mean / p50 / max word | `lit%` | `long%` | write amp | `wideE` | 1-use inline% | offline build ms (base / nat) |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 13,525,480 | 1.00× | 455.5 | 62.5 | 518.0 | 7.72 / 1 / 7,173 | 81.2% | 6.5% | 2.75× | 0 | n/a | 164 / 159 |
| `ctrl_shipped_fwd` | 13,525,480 | 1.00× | 455.5 | 62.5 | 518.0 | 7.72 / 1 / 7,173 | 81.2% | 6.5% | 2.75× | 0 | n/a | 164 / 159 |
| `ctrl_shipped_fwd_4lane` | 13,525,480 | 1.00× | 455.5 | 62.5 | 518.0 | 7.72 / 1 / 7,173 | 81.2% | 6.5% | 2.75× | 0 | n/a | 242 / 237 |
| **`F1a_repair_inl16`** | **4,534,061** | **0.34×** | **152.7** | **118.0** | **270.7** | 23.03 / 4 / 7,173 | **6.5%** | 19.2% | 1.37× | 0 | **0% (reused)** | 8,772 / 8,335 |
| **`F1b_repair_unb`** | **2,779,132** | **0.21×** | **93.6** | 337.5 | **431.1** | 37.57 / 3 / 17,625 | **8.3%** | 16.8% | 1.29× | 0 | **0% (reused)** | 11,280 / 10,968 |
| `F2a_conj_code_order` | 13,525,480 | 1.00× | 455.5 | 62.5 | 518.0 | 7.72 / 1 / 7,173 | 81.2% | 6.5% | 2.75× | 0 | n/a | 1,503 / 1,495 |
| `F2b_conj_cycle_oracle`* | 13,525,480 | 1.00× | 455.5 | 62.5 | 518.0 | 7.72 / 1 / 7,173 | 81.2% | 6.5% | 2.75× | 0 | n/a | 656 / 661 |
| `F3a_dict_capped_fuse` | 6,063,878 | 0.45× | 204.2 | 98.7 | 302.9 | 17.22 / 2 / 7,173 | 23.4% | 14.4% | 1.60× | 0 | n/a | 1,991 / 2,046 |
| `F3b_inline_edge12` | 3,011,425 | 0.22× | 405.6 | 30.2 | 435.8 | 34.67 / 12 / 7,173 | 4.2% | 34.0% | 1.06× | 0 | 34.7% | 1,026 / 975 |
| `F3c_repair_edge12` | 1,225,446 | 0.09× | 165.1 | 259.0 | 424.1 | 85.19 / 12 / 17,625 | 1.0% | 39.8% | 1.03× | 0 | 76.2% | 11,885 / 11,573 |
| **`F4a_repair_unb_4lane`** | **2,779,132** | **0.21×** | **93.6** | 337.5 | **431.1** | 37.57 / 3 / 17,625 | **8.3%** | 16.8% | 1.29× | 0 | **0% (reused)** | 11,319 / 11,008 |
| `F4b_edge12_4lane` | 3,011,425 | 0.22× | 405.7 | 30.2 | 435.9 | 34.67 / 12 / 7,173 | 4.2% | 34.0% | 1.06× | 0 | 34.7% | 1,119 / 1,075 |
| `F4c_repair_edge12_4l` | 1,225,446 | 0.09× | 165.1 | 259.0 | 424.1 | 85.19 / 12 / 17,625 | 1.0% | 39.8% | 1.03× | 0 | 76.2% | 11,914 / 11,602 |

*\*`F2b_conj_cycle_oracle` is explicitly labelled as an offline traversal-order upper bound on conjugation layout.*

### 3.3 Actual materialisation cost (held-out `116` blocks, 5 repeats, separated from consumer)

| materialisation arm | baseline x86-64 ms (`min..max`) | `-C target-cpu=native` ms (`min..max`) | ratio vs bidi `C` | projected i9 `C` (ms) |
|---|---:|---:|---:|---:|
| `mat_shipped_bidi` (`A6a+b+c_xor`) | 164.37 (`164.13..169.56`), `A6c=42.48` | 156.07 (`150.60..158.56`), `A6c=47.37` | 1.000× | 577.7 (exact anchor) |
| `mat_shipped_fwd` (`A6a+b+c_fwd`) | 141.20 (`135.61..147.24`), `A6c=16.90` | 125.97 (`125.06..132.42`), `A6c=16.53` | 0.807–0.859× (`A6c` 0.349–0.398×) | **472.6–480.5** (proj) |
| `mat_f3b_post_open` (linear `O(m)` pass on fwd) | 394.20 (`389.26..406.76`) (+ `C_fwd` = 535.39) | 407.97 (`390.33..418.16`) (+ `C_fwd` = 533.95) | **3.257–3.421×** | **~1,880–1,976** (kills net gain!) |

### 3.4 Held-out consumer timings & derived i9 budgets — Baseline x86-64 (7 interleaved repeats)

All i9 columns (`E_i9`, `dE_*`, `Cmax_*`, `dC_ext`, `8w_dE`) are **VM-to-i9 ratio projections** anchored at `E_base = 490.3 ms`, `C_base = 577.7 ms`, `C_fwd_proj = 480.5 ms`, `W_1w = 1,306.0 ms`, `3% gate = 39.18 ms`, and `8w_dE = dE_fwd × (69.5 / 490.3)` (scan-ideal 8w model, not measured on i9).

#### Warm (in-situ per-block L1/L2/L3 warm) — baseline x86-64

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 120.92 (`119.29..125.16`) | 1.158 | 8.94 | 1.000× | 1.100× | 1.508× | 490.3 | +0.0 | −49.7 | 538.5 | 391.6 | −88.9 | −7.0 |
| `ctrl_shipped_fwd` | 108.66 (`106.90..113.83`) | 1.041 | 8.03 | 0.909× | 1.000× | 1.368× | 440.6 | +49.7 | +0.0 | 588.2 | 441.3 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 80.21 (`79.27..83.22`) | 0.768 | 5.93 | 0.663× | 0.731× | 1.000× | 325.2 | +165.1 | +115.3 | 703.6 | 556.7 | +76.2 | +16.3 |
| **`F1a_repair_inl16`** | **40.10** (`38.70..43.03`) | 0.384 | 8.84 | **0.332×** | **0.372×** | **0.502×** | 162.6 | **+327.7** | **+278.0** | **866.2** | **719.3** | **+238.8** | +39.4 |
| **`F1b_repair_unb`** | **27.95** (`26.72..28.59`) | 0.268 | 10.06 | **0.228×** | **0.255×** | **0.343×** | 113.3 | **+377.0** | **+327.2** | **915.5** | **768.6** | **+288.1** | +46.4 |
| `F2a_conj_code_order` | 109.23 (`107.13..116.47`) | 1.046 | 8.08 | 0.901× | 1.001× | 1.369× | 442.9 | +47.4 | −2.3 | 585.9 | 439.0 | −41.5 | −0.3 |
| `F2b_conj_cycle_oracle`* | 76.86 (`75.84..84.46`) | 0.736 | 5.68 | 0.643× | 0.710× | 0.967× | 311.6 | +178.7 | +128.9 | 717.2 | 570.3 | +89.8 | +18.3 |
| `F3a_dict_capped_fuse` | 49.59 (`48.18..52.40`) | 0.475 | 8.18 | 0.409× | 0.457× | 0.617× | 201.1 | +289.2 | +239.5 | 827.7 | 680.8 | +200.3 | +33.9 |
| `F3b_inline_edge12` | 36.15 (`35.27..40.44`) | 0.346 | 12.01 | 0.302× | 0.333× | 0.456× | 146.6 | +343.7 | +294.0 | 882.2 | 735.3 | +254.8 | +41.7 |
| `F3c_repair_edge12` | **16.24** (`15.92..17.28`) | **0.156** | 13.25 | **0.134×** | **0.150×** | **0.202×** | 65.9 | +424.4 | +374.7 | 963.0 | 816.0 | +335.5 | +53.1 |
| **`F4a_repair_unb_4lane`** | **25.88** (`24.84..27.27`) | 0.248 | 9.31 | **0.211×** | **0.238×** | **0.319×** | 104.9 | **+385.4** | **+335.6** | **923.9** | **777.0** | **+296.5** | +47.6 |
| `F4b_edge12_4lane` | 37.65 (`36.18..38.56`) | 0.361 | 12.50 | 0.307× | 0.339× | 0.463× | 152.7 | +337.6 | +287.9 | 876.2 | 729.2 | +248.7 | +40.8 |
| `F4c_repair_edge12_4l` | 17.43 (`16.59..17.94`) | 0.167 | 14.22 | 0.145× | 0.159× | 0.214× | 70.7 | +419.6 | +369.9 | 958.1 | 811.2 | +330.7 | +52.4 |

#### Cold (64 MiB flush before each arm pass) — baseline x86-64

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 181.35 (`174.20..186.81`) | 1.737 | 13.41 | 1.000× | 0.962× | 1.449× | 490.3 | +0.0 | +16.8 | 538.5 | 458.1 | −22.4 | +2.4 |
| `ctrl_shipped_fwd` | 187.57 (`183.37..196.61`) | 1.797 | 13.87 | 1.040× | 1.000× | 1.523× | 507.1 | −16.8 | +0.0 | 521.7 | 441.3 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 125.40 (`120.96..128.04`) | 1.201 | 9.27 | 0.690× | 0.657× | 1.000× | 339.0 | +151.3 | +168.1 | 689.8 | 609.4 | +128.9 | +23.8 |
| **`F1a_repair_inl16`** | **62.51** (`62.11..68.12`) | 0.599 | 13.79 | **0.349×** | **0.336×** | **0.509×** | 169.0 | **+321.3** | **+338.1** | **859.8** | **779.4** | **+298.9** | +47.9 |
| **`F1b_repair_unb`** | **46.23** (`45.25..47.07`) | 0.443 | 16.64 | **0.256×** | **0.245×** | **0.372×** | 125.0 | **+365.3** | **+382.1** | **903.8** | **823.5** | **+342.9** | +54.2 |
| `F2a_conj_code_order` | 190.83 (`185.82..200.43`) | 1.828 | 14.11 | 1.073× | 1.012× | 1.544× | 515.9 | −25.6 | −8.8 | 512.9 | 432.5 | −48.0 | −1.2 |
| `F2b_conj_cycle_oracle`* | 80.88 (`77.11..85.58`) | 0.775 | 5.98 | 0.450× | 0.432× | 0.652× | 218.7 | +271.6 | +288.5 | 810.2 | 729.8 | +249.3 | +40.9 |
| `F3a_dict_capped_fuse` | 78.41 (`76.85..79.05`) | 0.751 | 12.93 | 0.433× | 0.410× | 0.621× | 212.0 | +278.3 | +295.1 | 816.8 | 736.5 | +256.0 | +41.8 |
| `F3b_inline_edge12` | 87.95 (`86.22..90.74`) | 0.842 | 29.20 | 0.488× | 0.471× | 0.707× | 237.8 | +252.5 | +269.3 | 791.0 | 710.7 | +230.2 | +38.2 |
| `F3c_repair_edge12` | 30.58 (`29.65..31.34`) | 0.293 | 24.95 | 0.168× | 0.161× | 0.244× | 82.7 | +407.6 | +424.5 | 946.2 | 865.8 | +385.3 | +60.2 |
| **`F4a_repair_unb_4lane`** | **40.82** (`38.88..45.57`) | 0.391 | 14.69 | **0.224×** | **0.214×** | **0.325×** | 110.3 | **+380.0** | **+396.8** | **918.5** | **838.1** | **+357.6** | +56.2 |
| `F4b_edge12_4lane` | 60.48 (`58.02..64.65`) | 0.579 | 20.08 | 0.329× | 0.322× | 0.480× | 163.5 | +326.8 | +343.6 | 865.3 | 784.9 | +304.4 | +48.7 |
| `F4c_repair_edge12_4l` | **26.96** (`26.42..27.29`) | **0.258** | 22.00 | **0.149×** | **0.144×** | **0.214×** | 72.9 | +417.4 | +434.2 | 955.9 | 875.5 | +395.0 | +61.6 |

### 3.5 Held-out consumer timings & derived i9 budgets — `-C target-cpu=native` (Haswell/AVX2, 7 interleaved repeats)

(`C_fwd_proj = 472.6 ms` from native `A6c_fwd / A6c_bidi = 0.349×`.)

#### Warm (in-situ per-block L1/L2/L3 warm) — `-C target-cpu=native`

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 120.35 (`113.28..129.26`) | 1.153 | 8.90 | 1.000× | 1.007× | 1.402× | 490.3 | +0.0 | −10.3 | 538.5 | 423.1 | −49.5 | −1.5 |
| `ctrl_shipped_fwd` | 117.82 (`112.68..130.33`) | 1.129 | 8.71 | 0.993× | 1.000× | 1.347× | 480.0 | +10.3 | +0.0 | 548.8 | 433.4 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 86.27 (`85.34..90.48`) | 0.826 | 6.38 | 0.713× | 0.742× | 1.000× | 351.5 | +138.8 | +128.5 | 677.4 | 562.0 | +89.4 | +18.2 |
| **`F1a_repair_inl16`** | **42.64** (`41.02..47.47`) | 0.408 | 9.41 | **0.355×** | **0.360×** | **0.488×** | 173.7 | **+316.6** | **+306.3** | **855.1** | **739.7** | **+267.1** | +43.4 |
| **`F1b_repair_unb`** | **30.66** (`28.71..31.96`) | 0.294 | 11.03 | **0.252×** | **0.262×** | **0.353×** | 124.9 | **+365.4** | **+355.1** | **903.9** | **788.5** | **+315.9** | +50.3 |
| `F2a_conj_code_order` | 119.81 (`116.56..127.03`) | 1.148 | 8.86 | 0.996× | 1.017× | 1.388× | 488.1 | +2.2 | −8.1 | 540.7 | 425.3 | −47.3 | −1.1 |
| `F2b_conj_cycle_oracle`* | 74.26 (`73.14..76.07`) | 0.711 | 5.49 | 0.608× | 0.621× | 0.858× | 302.5 | +187.8 | +177.5 | 726.3 | 610.9 | +138.3 | +25.2 |
| `F3a_dict_capped_fuse` | 53.88 (`51.62..58.14`) | 0.516 | 8.88 | 0.460× | 0.462× | 0.619× | 219.5 | +270.8 | +260.5 | 809.3 | 694.0 | +221.3 | +36.9 |
| `F3b_inline_edge12` | 36.10 (`35.57..39.34`) | 0.346 | 11.99 | 0.300× | 0.306× | 0.420× | 147.1 | +343.2 | +332.9 | 881.7 | 766.4 | +293.7 | +47.2 |
| `F3c_repair_edge12` | **16.49** (`15.95..17.76`) | **0.158** | 13.45 | **0.143×** | **0.143×** | **0.192×** | 67.2 | +423.1 | +412.8 | 961.7 | 846.3 | +373.7 | +58.5 |
| **`F4a_repair_unb_4lane`** | **28.21** (`26.55..31.96`) | 0.270 | 10.15 | **0.236×** | **0.236×** | **0.323×** | 114.9 | **+375.4** | **+365.1** | **913.9** | **798.5** | **+325.9** | +51.8 |
| `F4b_edge12_4lane` | 37.65 (`36.99..38.00`) | 0.361 | 12.50 | 0.310× | 0.317× | 0.435× | 153.4 | +336.9 | +326.6 | 875.4 | 760.0 | +287.4 | +46.3 |
| `F4c_repair_edge12_4l` | 18.02 (`16.83..18.84`) | 0.173 | 14.71 | 0.148× | 0.150× | 0.208× | 73.4 | +416.9 | +406.6 | 955.4 | 840.0 | +367.4 | +57.6 |

#### Cold (64 MiB flush before each arm pass) — `-C target-cpu=native`

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 173.19 (`167.19..179.85`) | 1.659 | 12.80 | 1.000× | 0.874× | 1.372× | 490.3 | +0.0 | +81.0 | 538.5 | 514.4 | +41.8 | +11.5 |
| `ctrl_shipped_fwd` | 201.80 (`191.94..212.38`) | 1.933 | 14.92 | 1.144× | 1.000× | 1.554× | 571.3 | −81.0 | +0.0 | 457.5 | 433.4 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 126.13 (`124.10..136.65`) | 1.208 | 9.33 | 0.729× | 0.643× | 1.000× | 357.1 | +133.2 | +214.2 | 671.7 | 647.7 | +175.0 | +30.4 |
| **`F1a_repair_inl16`** | **65.56** (`62.56..72.76`) | 0.628 | 14.46 | **0.379×** | **0.330×** | **0.526×** | 185.6 | **+304.7** | **+385.7** | **843.2** | **819.1** | **+346.5** | +54.7 |
| **`F1b_repair_unb`** | **47.18** (`45.94..56.99`) | 0.452 | 16.98 | **0.279×** | **0.243×** | **0.376×** | 133.6 | **+356.7** | **+437.7** | **895.2** | **871.2** | **+398.6** | +62.0 |
| `F2a_conj_code_order` | 199.47 (`194.76..221.08`) | 1.911 | 14.75 | 1.152× | 1.017× | 1.606× | 564.7 | −74.4 | +6.6 | 464.1 | 440.0 | −32.6 | +0.9 |
| `F2b_conj_cycle_oracle`* | 77.77 (`76.49..81.99`) | 0.745 | 5.75 | 0.453× | 0.384× | 0.617× | 220.2 | +270.1 | +351.1 | 808.6 | 784.6 | +312.0 | +49.8 |
| `F3a_dict_capped_fuse` | 81.05 (`77.79..96.21`) | 0.776 | 13.37 | 0.468× | 0.418× | 0.643× | 229.5 | +260.8 | +341.9 | 799.4 | 775.3 | +302.7 | +48.5 |
| `F3b_inline_edge12` | 87.73 (`84.53..94.35`) | 0.840 | 29.13 | 0.516× | 0.439× | 0.690× | 248.4 | +241.9 | +322.9 | 780.4 | 756.4 | +283.7 | +45.8 |
| `F3c_repair_edge12` | 30.31 (`28.89..31.11`) | 0.290 | 24.73 | 0.174× | 0.150× | 0.241× | 85.8 | +404.5 | +485.5 | 943.0 | 918.9 | +446.3 | +68.8 |
| **`F4a_repair_unb_4lane`** | **40.90** (`39.86..47.37`) | 0.392 | 14.72 | **0.240×** | **0.210×** | **0.324×** | 115.8 | **+374.5** | **+455.5** | **913.0** | **888.9** | **+416.3** | +64.6 |
| `F4b_edge12_4lane` | 60.45 (`58.77..69.67`) | 0.579 | 20.07 | 0.354× | 0.310× | 0.483× | 171.1 | +319.2 | +400.2 | 857.7 | 833.6 | +361.0 | +56.7 |
| `F4c_repair_edge12_4l` | **27.01** (`26.45..30.24`) | **0.259** | 22.04 | **0.158×** | **0.138×** | **0.218×** | 76.5 | +413.8 | +494.8 | 952.3 | 928.3 | +455.6 | +70.1 |

---

## 4. Physical interpretation & implications for constructor design

1. **Why Round 2's `LIMIT` sweep failed while `F1a`/`F1b` win by 3×–4×:** Lowering Open's run-width `LIMIT` from `16 → 2` left `lit%` completely unchanged (`81.6% → 81.6%`) while exploding thick strips `3.56×` and spelling `6.6×`. In contrast, contracting **reused multi-byte phrases** (`F1a_repair_inl16`) attacks the 81.2% literal storm directly (`81.2% → 6.5%`), cutting visits to `0.34×`, shrinking `edges` into 256 KiB L2 (`455.5 → 152.7 KiB/blk`), and cutting total representation bytes nearly in half (`518.0 → 270.7 KiB/blk`) through the **exact same `emit` consumer**!
2. **Contraction beats layout, and contraction + 4-lane MLP stack modestly:** `F1b` (non-sequential BWT row-rank order, 1 lane) beats even the offline cycle-order layout oracle `F2b` by **2.4×–2.7× warm** and **1.7× cold**. Adding 4-lane MLP (`F4a`) gives a further `7–8%` warm / `12–13%` cold speedup on top of `F1b` (`0.211–0.240×` bidi), because once `edges` drops to `93.6 KiB/blk` (deep inside L2), 4 lanes mainly hides remaining L2/L3 dictionary misses.
3. **Why `F3b_inline_edge12` loses to `F1b` when cold (`87.7 vs 47.2 ms`):** Co-located 16-byte edges are 4× wider per vertex (`405.6 KiB/blk` vs `93.6 KiB/blk`), spilling 256 KiB L2 and paying `29.1 ns/visit` cold vs `16.9 ns/visit` for `F1b`. Combining RePair with `Edge16` (`F3c`/`F4c`) shrinks `edges` back into L2 (`165.1 KiB/blk`), dropping warm emission to `0.156 ns/row` (`~1.13×` `memcpy`), though `76.2%` of its residual inline edges are 1-use gap chunks.
4. **The constructor constraint — why BWT/bucket-space contraction is the only viable path:**
   - Offline RePair (`8.3–11.3 s`) is a pure product oracle, and even a single-pass `O(m)` contraction over Open's `13.53M` row edges (`mat_f3b_post_open`, `394–408 ms` on VM) costs **3.26–3.42×** Open's constructor — proving no post-Open row-graph pass can win.
   - However, the **allowable total constructor cost** is large: **`C_max_f = 719–799 ms` warm** (`+239..+326 ms` above `C_fwd ≈ 473–481 ms`) and **`855–924 ms` vs shipped `C_base = 577.7 ms`**!
   - Crucially, in a BWT, **every occurrence of a frequent digram `(a, b)` in the text is a run of `L`-byte `a` inside the contiguous `F`-bucket `[C[b], C[b+1])`**! Eliminating high-frequency literal `F`-buckets or digram intervals (e.g., selective byte-class / bucket elimination inside Open's auxiliary IET, or band-level contraction before `A6c` row wiring) operates on runs/buckets/bands (`~1.8M` nodes / `828k` bands) rather than chasing `26.6M` individual row edges.
   - **Action taken:** Opened separate constructor probe claim [`bwt-digram-return-map`](../../../../frontier/claims/bwt-digram-return-map.toml) at `status = "idea"` with a frozen net-cost gate (`>= 39.18 ms = 3.0%` of 1,306 ms end-to-end `C + E` against both shipped baseline and matched `forward-emission`, byte-exact, smooth-zone prototype first).

---

## 5. Caveats & scope

- **VM vs i9-10900K:** All consumer timings were measured on a 2-vCPU Haswell Xeon VM (`2.30 GHz`, `256 KiB` L2, `45 MiB` L3, no vPMU); i9-10900K millisecond budgets (`E_i9`, `C_max_*`, `8w_dE`) are transparent VM-to-i9 ratio projections anchored at `character-stage1`, not direct i9 measurements. Note that both machines have a `256 KiB` per-core L2 cache, so the L2-residency transition (`455.5 → 93.6–152.7 KiB/blk`) directly mirrors i9 L2 geometry.
- **8-worker caveat:** Worker-side emission savings at $k = 8$ remain gated behind the serial Huffman scan (`179.1 ms`) until scan relief lands; `8w ΔE` is strictly a scan-ideal model projection.
- **Prior gates untouched:** All earlier STOP/KILL verdicts (`rauzy-census` order STOP, `rauzy-output` LIMIT/SLP/step-compaction STOP/KILL, `forward-emission`) remain completely intact.
