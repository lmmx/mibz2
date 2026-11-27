# BWT-space digram and byte-bucket contracted return-map constructor: results & verdict

Claim [`bwt-digram-return-map`](../../../../frontier/claims/bwt-digram-return-map.toml) — smooth-zone constructor probe following the frozen [`protocol.md`](protocol.md) (written before any held-out timings were run). Zero edits to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero workstation spending, zero alteration of prior frozen gates (C2).

**Harness:** [`examples/bwt_digram_return_map.rs`](../../../../examples/bwt_digram_return_map.rs).  
**Corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks: `233` production Open-admitted + `198` non-admitted).  
**Split (frozen before data):** Train = first `117` admitted blocks (`105,298,817` rows); Held-out = last `116` admitted blocks (`104,398,845` rows).  
**Guard corpora (`ab gen`):** `runs.bz2` (`1` admitted block, `508,269` rows) and `text.bz2` (`24` forced blocks, `21,095,146` rows).  
**Hardware & method:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3, no vPMU), single-threaded, reusable scratch & `out` buffers, `7` block-interleaved repeats (rotating arm start offset `(k + blk + rep) % n_arms` per block to neutralize shared-VM CPU load transients), compiled for both **baseline x86-64** and **`-C target-cpu=native`**.

---

## Executive summary & verdict (strict C2 evaluation)

**Verdict: STOP — record a scoped dead-end in `frontier/dead-ends.tsv`.**

No tested BWT-space digram, selective `F`-bucket `LIMIT`, 1-hop digram, or whole-`F`-bucket contracted return-map constructor clears the frozen `+39.18 ms` (`3.0%` of the 1,306 ms 1-worker reference wall) end-to-end `C + E` gate against matched `forward-emission` (single-lane or 4-lane) on held-out Wikidata blocks in either baseline x86-64 or `-C target-cpu=native`, and every constructor family also regresses the guard corpora (`runs`, forced `text`):

1. **Whole `F`-bucket elimination (`fb_elim_75_L16`, `DictReturnMap`, exact forward `emit`) reproduces the offline `ideal-permutation-structures` consumer speedup — yet loses end-to-end by `+13.8%..+14.7%` vs `ctrl_fwd_1lane` (`+28.9%..+32.9%` vs `ctrl_fwd_4lane_ora`):**
   - On train, eliminating the top literal-heavy `F`-buckets (`'0'..'9'`, `'-'`, `'A'..'F'`, `'a'..'f'`, etc. — Wikidata's UUID/hex/ID literal storm, `lit_frac = 92.7%–100.0%`) shrinks retained rows `m` by **−75.7%** (`13.04M → 3.17M`), cuts `lit%` **`82.0% → 17.4%`**, and shrinks `edges` **`435.4 → 105.9 KiB/blk`** (deep inside 256 KiB L2).
   - On held-out Wikidata, that cuts `A7` emission by **−61.3%** (`113.12 → 43.75 ms` baseline; `118.27 → 45.80 ms` native = **`0.387×`** of `ctrl_fwd_1lane`, matching `F1a_repair_inl16`'s `0.37×`).
   - **Charging the constructor (`contr`):** Even with zero `HashMap`, zero `remap_v` (storing compact `new_id` directly in `edges[K]`), zero cycle detection (proved unnecessary when `mark ∈ K`), and a 4 KiB L1d direct-mapped inline cache, `contr` costs **`108.02 ms` baseline / `113.49 ms` native** — erasing the entire `−69..−72 ms` `A7` gain! End-to-end `C + E` is **`1.147×` (`1.095..1.207`) baseline / `1.138×` (`1.114..1.180`) native** vs `ctrl_fwd_1lane` (`−136.1 / −147.5 ms` projected i9) and **`1.289× / 1.329×`** vs `ctrl_fwd_4lane_ora`. Raising Open `LIMIT` to 24 (`fb_elim_75_L24`) trims `A6b` (`−15.2..−15.7 ms`) but grows `contr` (`+6.5..+10.4 ms`), still losing at **`1.131× / 1.129×`** vs `ctrl_fwd_1lane`.
2. **Co-located `Edge16` (`fb_e16_75_L16`) and pure 1-hop digram contraction (`seq1hop_digram_L16`) both lose harder (`1.169×–1.206×` vs `ctrl_fwd_1lane`):**
   - `Edge16` eliminates inline dictionary hashing (`contr = 101.83 / 98.62 ms`), but 4× wider 16 B edges regress `A7` by `+22 ms` (`65.51 / 67.76 ms`), landing at **`1.206× / 1.179×`** vs `ctrl_fwd_1lane` (`2.25–2.27×` on `runs`, `1.51–1.59×` on forced `text`).
   - `seq1hop_digram_L16` avoids multi-hop chasing and `spelling` growth completely via a direct `[u16; 65536]` table (`contr = 56.49 / 52.76 ms`), but because Wikidata's literal buckets (`0-9`, `A-F`, `a-f`) transition heavily among themselves (`E → E`), 1-hop `K → E → K` only cuts `A7` by `−8.3..−8.9 ms` (`1.179× / 1.169×` vs `ctrl_fwd_1lane`).
3. **BWT-space selective `F`-bucket / digram-interval `LIMIT` before `A6b` (`sel_bwt_lim_*`) fails even with FREE oracle `C[0..256]`:**
   - Even without charging the extra `records` pass required to know `C[0..256]`, `sel_bwt_lim_8_16`, `sel_bwt_lim_12_16`, and `sel_bwt_lim_16_24` all lose in baseline x86-64 (**`1.030×–1.063×`** vs `ctrl_fwd_1lane`, `−35.7..−63.6 ms` i9) and are neutral/overlapping in native (**`0.983×–1.023×`**, best `+9.8 ms` i9 with overlapping spread `0.971..1.027` vs `ctrl_fwd_1lane`, and **`1.157×–1.197×`** slower than `ctrl_fwd_4lane_ora`).
4. **Hardware lower-bound ablations (`ablat_fb75_chase1`, `ablat_fb75_chase4`) expose the two-horn structural trap:**
   - **Before `A6c` (BWT/run/band space):** `76.45%` of all thin runs (`5.55M` runs, `42.54%` of `m`) are `w = 1` singletons (`87.10%` have `w <= 2`, `54.40%` of `m`), and bands before `A6c` do not know target row ranks `j` (which live in `outq`). Hence no pre-wiring strip/band pass can amortize the singleton storm.
   - **After `A6c` (bucket/row-graph space — Pointer-Chase Conservation Law):** Pure `ELIM_BIT` tagging + chasing (`ablat_fb75_chase1`, zero word assembly, zero dict, zero `spelling`, zero `new_edges`, zero `A7`) alone costs **`52.92 / 51.10 ms`**; register-only 4-way chasing (`ablat_fb75_chase4`) lowers that bare floor to **`37.53 / 37.97 ms`**. Adding `fb_elim_75`'s `A7` (`43.75 / 45.80 ms`) gives **`81.28 / 83.77 ms`**, which already matches or exceeds `ctrl_fwd_4lane_ora`'s entire `A7` (`84.39 / 81.57 ms`) *before* paying a single nanosecond for word assembly, dictionary interning, `spelling`, or `new_edges` writes (`+55.1 / +62.4 ms` in real code).

---

## 1. Exactness verification (100% byte-identical to `bwt::walk`)

Every exact arm (`13` arms: `4` controls + `9` frozen candidate constructors) was verified **100% byte-identical to `bwt::walk`** across every tested corpus and synthetic edge case before timing:

| suite | blocks | exact arms | exact / total |
|---|---:|---:|---:|
| Synthetic stress suite (`dense_4k`, periodic `abac256`/`aaab512`/`ab500`, arbitrary non-BWT multicycle `64`/`4k`, `run_heavy_4k`, edges `n=1`/`n=2`/`const64`) | 11 | 13 | **143 / 143** |
| `corpora/wikidata.bz2` production-admitted (`OpenCells`) | 233 | 13 | **3,029 / 3,029** |
| `corpora/wikidata.bz2` forced non-admitted (`Cap::NONE`) | 198 | 13 | **2,574 / 2,574** |
| Guard `runs.bz2` (production-admitted) | 1 | 13 | **13 / 13** |
| Guard `text.bz2` (forced `Cap::NONE`) | 24 | 13 | **312 / 312** |
| **Total** | **467** | **13** | **6,071 / 6,071** |

---

## 2. Train-set structural census (`117` admitted blocks, `105,298,817` rows)

### 2.1 Why BWT-space run/band selection cannot absorb the literal storm

In Open `LIMIT = 16` (`m = 13,042,101` retained rows across `7,257,389` thin runs; `375,526` thick strips covering `92,256,716` rows):
- **`w = 1` singletons:** `5,547,959` runs (**`76.45%`** of thin runs) and `5,547,959` rows (**`42.54%`** of `m`) — zero run-level amortization (`1` strip = `1` row).
- **`w = 2` runs:** `773,181` runs (`10.65%`, cumulative **`87.10%`**) and `1,546,362` rows (`11.86%`, cumulative **`54.40%`**).
- **`w = 3..=4` runs:** `431,248` runs (`5.94%`, cumulative **`93.04%`**) and `1,477,180` rows (`11.33%`, cumulative **`65.72%`**).
- **`w = 5..=16` runs:** `505,001` runs (`6.96%`) and `4,470,599` rows (`34.28%`).

### 2.2 Retained `F`-bucket anatomy & edge-class transitions

Edge-class transition matrix on Open `LIMIT = 16` (`m = 13,042,101`):
- `lit(1 B) → lit(1 B)`: **`73.10%`** (`lit → inl`: `5.18%`, `lit → long`: `3.70%`)
- `inl(2..16 B) → lit(1 B)`: `7.37%` (`inl → inl`: `3.70%`, `inl → long`: `0.54%`)
- `long(>16 B) → lit(1 B)`: `2.32%` (`long → inl`: `2.25%`, `long → long`: `1.83%`)

Top retained `F`-buckets on train (`111` active `F`-buckets; top-32 cover **`92.55%`**, top-64 cover **`99.63%`**):
- **`'"'` (#1, `10.14%` of `m`):** structural anchor — only `6.3%` literals, **`50.3%` long words**, `38.4%` self-edges.
- **`'0'..'9'` (#2–#11, `46.87%` of `m`):** **`92.7%–98.6%` 1-byte literals**, `0.0%–0.6%` long, `5.5%–6.8%` self-edges.
- **`'-'` (#12, `3.97%` of `m`):** `98.1%` literals, `0.3%` long, `0.1%` self-edges (UUID/date separators).
- **`'A'..'F'` & `'a'..'f'` (hex digits, ~`21%` of `m`):** **`93.0%–100.0%` 1-byte literals**, `0.0%–0.5%` long (Wikidata UUIDs/hashes).
- **`'\'` (#14, `2.47%`) & `','` (#27, `1.33%`):** structural anchors (`0.4%` / `8.7%` literals).

### 2.3 Whole `F`-bucket elimination structural trade-off (train `117` blocks, exact dedup census)

| strategy | `new_m` | `m'/m` | `lit%` | `long%` | `dict/b` | `edges` KiB/b | `spelling` KiB/b | `rep` KiB/b | `maxCh` | `wide` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `baseline_open_L16` | 13,042,101 | 1.000 | 82.0% | 6.4% | 1,014 | 435.4 | 40.7 | 493.0 | 0 | 0 |
| `top_lit_50pct_full` | 6,299,868 | 0.483 | 36.2% | 13.7% | 8,222 | 210.3 | 96.9 | 443.7 | 36 | 0 |
| `top_lit_65pct_full` | 4,432,068 | 0.340 | 25.4% | 22.6% | 7,623 | 148.0 | 148.3 | 422.9 | 128 | 0 |
| `top_lit_75pct_full` | 3,173,114 | 0.243 | 17.4% | 34.6% | 6,277 | 105.9 | 224.5 | 434.6 | 219 | 0 |
| `top_lit_85pct_full` | 1,782,634 | 0.137 | 7.9% | 50.7% | 5,493 | 59.5 | 837.3 | 988.0 | 983 | 0 |
| `maxcut_nolong` | 6,117,502 | 0.469 | 33.2% | 15.2% | 6,937 | 204.2 | 88.5 | 407.9 | 45 | 0 |

---

## 3. Held-out end-to-end `C + E` timings (`116` blocks, `104,398,845` rows, `7` block-interleaved repeats)

All i9 columns (`i9_CE`, `d_vsF1`, `d_vsF4r`) are transparent VM-to-i9 proportional projections anchored at `C_base + E_base = 1,068.0 ms` (`W_1w = 1,306.0 ms`, 3.0% gate = `+39.18 ms`). Paired per-repeat ratios (`vsF1`, `vsF4o`, `vsF4r`) are computed within each repeat (`arm[r] / ctrl[r]`) so transient shared-VM CPU load cancels cleanly across arms. Non-exact hardware lower-bound ablations are marked `*`.

### 3.1 Baseline x86-64 (`cargo build --release --example bwt_digram_return_map`)

| arm | `A6a` | `A6b` | `A6c` | `contr` | `A7` | `C+E` ms med (`min..max`) | `vsF1` med (`min..max`) | `vsF4o` | `vsF4r` | `i9_CE` (proj) | `Δ` vs F1 | `Δ` vs F4r |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 68.33 | 63.95 | 41.24 | 0.00 | 121.44 | 297.16 (`287.94..428.88`) | 1.128 (`1.071..1.154`) | 1.113× | 1.266× | 1,068.0 | −122.9 | −228.0 |
| **`ctrl_fwd_1lane`** | 68.54 | 65.06 | 17.26 | 0.00 | 113.12 | **262.96** (`254.78..381.96`) | **1.000** (`1.000..1.000`) | 0.992× | 1.123× | **947.0** | 0.0 | −107.0 |
| `ctrl_fwd_4lane_onl` | 68.37 | 63.68 | 16.72 | 0.00 | 113.84 | 264.57 (`256.57..392.95`) | 1.008 (`0.978..1.029`) | 1.000× | 1.128× | 959.2 | −14.1 | −119.3 |
| **`ctrl_fwd_4lane_ora`** | 68.37 | 64.49 | 16.99 | 0.00 | 84.39 | **233.72** (`227.39..348.58`) | **0.890** (`0.877..0.913`) | 0.886× | **1.000×** | **843.4** | +101.7 | 0.0 |
| `fb_elim_75_L16` | 68.03 | 64.69 | 16.93 | 108.02 | **43.75** | 301.17 (`296.57..418.37`) | 1.147 (`1.095..1.207`) | 1.151× | 1.289× | 1,081.2 | **−136.1** | **−241.3** |
| `fb_elim_55_L16` | 68.22 | 65.45 | 16.90 | 123.95 | 61.75 | 336.86 (`327.68..482.90`) | 1.266 (`1.251..1.286`) | 1.273× | 1.428× | 1,203.9 | −258.8 | −363.9 |
| `fb_elim_75_L24` | 71.01 | 49.88 | 15.52 | 118.43 | 42.82 | 298.15 (`289.53..431.91`) | 1.131 (`1.108..1.136`) | 1.127× | 1.273× | 1,073.9 | −128.8 | −233.9 |
| `fb_elim_75_4L_ora` | 69.80 | 65.38 | 16.89 | 104.47 | 43.92 | 299.70 (`292.35..423.79`) | 1.136 (`1.110..1.147`) | 1.133× | 1.282× | 1,078.5 | −133.4 | −238.5 |
| `fb_e16_75_L16` | 69.21 | 65.38 | 17.27 | 101.83 | 65.51 | 317.47 (`310.39..478.63`) | 1.206 (`1.196..1.253`) | 1.210× | 1.364× | 1,151.0 | −205.9 | −311.1 |
| `seq1hop_digram_L16` | 68.14 | 65.72 | 17.37 | 56.49 | 104.25 | 311.50 (`301.72..446.21`) | 1.179 (`1.162..1.196`) | 1.169× | 1.327× | 1,119.1 | −174.0 | −279.1 |
| `sel_bwt_lim_8_16` | 76.76 | 89.11 | 20.99 | 0.00 | 96.14 | 281.08 (`273.45..395.92`) | 1.063 (`1.037..1.079`) | 1.052× | 1.201× | 1,008.7 | −63.6 | −168.7 |
| `sel_bwt_lim_12_16` | 78.41 | 71.92 | 18.33 | 0.00 | 105.70 | 274.64 (`266.87..396.32`) | 1.044 (`1.026..1.068`) | 1.040× | 1.173× | 987.1 | −42.0 | −147.1 |
| `sel_bwt_lim_16_24` | 77.34 | 56.83 | 16.84 | 0.00 | 121.60 | 273.54 (`262.30..393.87`) | 1.030 (`1.006..1.062`) | 1.022× | 1.156× | 980.8 | −35.7 | −140.8 |
| `ablat_fb75_chase1`* | 68.00 | 66.40 | 16.93 | **52.92** | 0.00 | 205.64 (`197.92..292.40`) | 0.776 (`0.764..0.784`) | 0.771× | 0.873× | 734.1* | *(chase floor)* | — |
| `ablat_fb75_chase4`* | 67.99 | 64.83 | 16.73 | **37.53** | 0.00 | 187.53 (`182.92..300.13`) | 0.716 (`0.687..0.786`) | 0.713× | 0.804× | 678.5* | *(4w chase floor)* | — |

### 3.2 `-C target-cpu=native` (Haswell/AVX2)

| arm | `A6a` | `A6b` | `A6c` | `contr` | `A7` | `C+E` ms med (`min..max`) | `vsF1` med (`min..max`) | `vsF4o` | `vsF4r` | `i9_CE` (proj) | `Δ` vs F1 | `Δ` vs F4r |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 65.13 | 62.56 | 46.42 | 0.00 | 120.31 | 295.33 (`281.88..426.44`) | 1.108 (`1.093..1.118`) | 1.081× | 1.287× | 1,068.0 | −109.6 | −239.6 |
| **`ctrl_fwd_1lane`** | 65.24 | 64.15 | 16.66 | 0.00 | 118.27 | **265.01** (`257.71..386.36`) | **1.000** (`1.000..1.000`) | 0.986× | 1.163× | **964.2** | 0.0 | −135.8 |
| `ctrl_fwd_4lane_onl` | 66.49 | 63.14 | 16.60 | 0.00 | 124.18 | 271.09 (`261.50..421.24`) | 1.015 (`0.979..1.090`) | 1.000× | 1.177× | 988.0 | −29.6 | −159.5 |
| **`ctrl_fwd_4lane_ora`** | 66.93 | 62.25 | 16.65 | 0.00 | 81.57 | **229.08** (`218.98..333.90`) | **0.860** (`0.838..0.882`) | 0.849× | **1.000×** | **829.7** | +128.7 | 0.0 |
| `fb_elim_75_L16` | 66.65 | 63.98 | 16.59 | 113.49 | **45.80** | 305.79 (`292.05..442.40`) | 1.138 (`1.114..1.180`) | 1.124× | 1.329× | 1,105.8 | **−147.5** | **−277.4** |
| `fb_elim_55_L16` | 65.67 | 63.34 | 16.52 | 131.36 | 64.97 | 339.95 (`328.49..504.01`) | 1.291 (`1.268..1.305`) | 1.256× | 1.500× | 1,244.6 | −286.2 | −416.2 |
| `fb_elim_75_L24` | 69.41 | 48.46 | 15.43 | 119.98 | 43.66 | 298.95 (`286.60..436.32`) | 1.129 (`1.108..1.144`) | 1.103× | 1.315× | 1,089.5 | −131.1 | −261.1 |
| `fb_elim_75_4L_ora` | 66.38 | 62.70 | 16.58 | 107.15 | 44.52 | 298.29 (`289.65..449.81`) | 1.126 (`1.120..1.164`) | 1.106× | 1.323× | 1,091.5 | −133.1 | −263.1 |
| `fb_e16_75_L16` | 65.06 | 62.79 | 16.47 | 98.62 | 67.76 | 311.81 (`305.93..477.08`) | 1.179 (`1.141..1.235`) | 1.166× | 1.379× | 1,129.8 | −171.5 | −301.4 |
| `seq1hop_digram_L16` | 66.86 | 64.40 | 16.61 | 52.76 | 109.97 | 312.93 (`302.18..451.50`) | 1.169 (`1.145..1.181`) | 1.156× | 1.365× | 1,130.8 | −172.4 | −302.4 |
| `sel_bwt_lim_8_16` | 64.53 | 86.00 | 19.64 | 0.00 | 100.59 | 269.69 (`263.75..431.08`) | 1.023 (`0.994..1.116`) | 1.009× | 1.197× | 982.9 | −24.5 | −154.5 |
| `sel_bwt_lim_12_16` | 63.21 | 71.70 | 17.92 | 0.00 | 111.85 | 265.34 (`254.23..374.97`) | 0.983 (`0.971..1.027`) | 0.980× | 1.158× | 948.5 | +9.8 | −120.1 |
| `sel_bwt_lim_16_24` | 63.15 | 54.72 | 16.36 | 0.00 | 127.30 | 260.95 (`253.35..406.80`) | 0.996 (`0.976..1.053`) | 0.969× | 1.157× | 959.9 | −1.5 | −131.5 |
| `ablat_fb75_chase1`* | 65.59 | 62.99 | 17.41 | **51.10** | 0.00 | 197.01 (`190.72..282.41`) | 0.740 (`0.721..0.769`) | 0.729× | 0.860× | 708.2* | *(chase floor)* | — |
| `ablat_fb75_chase4`* | 66.62 | 63.08 | 16.30 | **37.97** | 0.00 | 185.65 (`176.35..278.94`) | 0.695 (`0.684..0.722`) | 0.682× | 0.808× | 670.3* | *(4w chase floor)* | — |

---

## 4. Guard corpora (`runs.bz2` admitted & `text.bz2` forced, `7` block-interleaved repeats)

Paired median ratio against `ctrl_fwd_1lane` (`1.000×`):

| arm | `runs` baseline (`min..max`) | `runs` native (`min..max`) | `text` forced baseline (`min..max`) | `text` forced native (`min..max`) |
|---|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 1.032 (`0.764..1.084`) | 1.034 (`0.976..3.003`) | 0.926 (`0.894..0.980`) | 0.929 (`0.853..0.933`) |
| **`ctrl_fwd_1lane`** | **1.000** | **1.000** | **1.000** | **1.000** |
| `ctrl_fwd_4lane_onl` | **0.872** (`0.575..0.992`) | **0.950** (`0.906..2.898`) | **0.768** (`0.760..0.828`) | **0.823** (`0.748..0.904`) |
| `ctrl_fwd_4lane_ora` | **0.704** (`0.500..0.779`) | **0.650** (`0.602..0.765`) | **0.655** (`0.600..0.754`) | **0.652** (`0.647..0.691`) |
| `fb_elim_75_L16` | 1.351 (`1.199..1.529`) | 1.414 (`1.284..3.460`) | 1.277 (`1.184..1.409`) | 1.303 (`1.284..1.403`) |
| `fb_elim_55_L16` | 1.289 (`0.813..1.376`) | 1.422 (`1.283..3.453`) | 1.321 (`1.133..1.490`) | 1.289 (`1.226..1.423`) |
| `fb_elim_75_L24` | 1.251 (`0.872..1.757`) | 1.342 (`1.174..4.941`) | 1.251 (`1.138..1.508`) | 1.268 (`1.113..1.331`) |
| `fb_elim_75_4L_ora` | 1.106 (`0.734..1.186`) | 1.133 (`1.010..1.742`) | 1.266 (`1.171..1.498`) | 1.262 (`1.163..1.385`) |
| `fb_e16_75_L16` | 2.250 (`1.917..2.327`) | 2.273 (`2.050..6.497`) | 1.594 (`1.457..1.726`) | 1.507 (`1.292..1.606`) |
| `seq1hop_digram_L16` | 1.192 (`0.881..1.228`) | 1.245 (`1.164..3.236`) | 1.071 (`0.987..1.182`) | 1.026 (`0.903..1.188`) |
| `sel_bwt_lim_8_16` | 1.074 (`0.780..1.114`) | 0.975 (`0.926..1.053`) | 1.016 (`0.919..1.143`) | 0.959 (`0.878..1.057`) |
| `sel_bwt_lim_12_16` | 1.033 (`0.756..1.051`) | 1.131 (`0.958..3.033`) | 1.065 (`0.886..1.104`) | 0.974 (`0.856..0.996`) |
| `sel_bwt_lim_16_24` | 1.012 (`0.985..1.803`) | 0.960 (`0.918..1.045`) | 1.077 (`1.028..1.199`) | 1.022 (`0.865..1.034`) |

Every bucket/digram contraction arm regresses both `runs` (`1.11–2.27×`) and forced `text` (`1.03–1.59×`), whereas `ctrl_fwd_4lane_onl` (using bit-31 anchor tags in `edges` with zero auxiliary array) speeds up both `runs` (`0.872–0.950×`) and forced `text` (`0.768–0.823×`).

---

## 5. Physical diagnosis: the two-horn structural trap

Why did offline RePair (`ideal-permutation-structures`) promise `+278..+365 ms` of consumer room, yet every online BWT-space and `F`-bucket-space constructor fails to realize a net `C + E` win?

1. **Horn 1 — Before `A6c` (BWT run / digram-interval / band space):**
   - `76.45%` of all retained thin runs (`42.54%` of `m`) are `w = 1` singletons, and `87.10%` have `w <= 2` (`54.40%` of `m`). Inside a literal `F`-bucket like `'4'`, adjacent rows carry different `L`-bytes (`'0'..'9'`, `'-'`, `'"'`), so `LF` scatters them into 16–20 different `F`-buckets rather than acting as a single translation. Eliminating those thin runs via `endpoint.rs` pays `~84 ns/strip` for 1–2 rows, explaining why both the global `LIMIT` sweep (`rauzy-census/results-2`) and free-oracle selective `F`-bucket `LIMIT` (`sel_bwt_lim_*`) lose to `LIMIT = 16`.
   - Equally, Open's `bands` before `A6c` know only the `L`-byte `a` of their target rows, not their target row ranks `j` (which live inside `outq`), so two bands cannot be composed before reading `outq` (`A6c`) except along contiguous `w > 1` runs (which reduces back to `endpoint.rs` strips).
2. **Horn 2 — After `A6c` (`F`-bucket / row-graph space — Pointer-Chase Conservation Law):**
   - Single-lane `ctrl_fwd_1lane` (`A7`) visits each of the `m = 13.53M` retained vertices **exactly once**, writing decoded bytes straight to `out`.
   - Eliminating a subset `E` of `75%` of retained vertices (`m_E = 10.2M`, `m_K = 3.3M`) via `F`-bucket contraction must still perform `13.53M` random loads in `edges[0..m]` (`455 KiB`) to follow `K → E^+ → K` chains, *plus* assemble `~10M` eliminated bytes into `u128`/`spelling` (`167–370 KiB/blk`, dominated by unique Wikidata UUIDs/IDs), intern `~6,200` words/blk, write `new_edges[0..m_K]`, *and then* perform `3.3M` more random loads in `A7` — paying **`+24%` more random edge loads (`16.8M` vs `13.5M`)** and **`+43 MB` more byte writes** than emitting directly!
   - Even the pure chase-only lower-bound ablations (`ablat_fb75_chase1 = 51.1–52.9 ms`, `ablat_fb75_chase4 = 37.5–38.0 ms`) leave at most `16–21 ms` (1-way) vs `ctrl_fwd_1lane` and **`−2.2..+3.1 ms` vs `ctrl_fwd_4lane_ora`** *before* charging a single nanosecond of word assembly, dictionary interning, `spelling`, or `new_edges` writes (`+55.1..+62.4 ms` real cost).
3. **Implication for the frontier:** Single-pass contracted return-map construction over Open's retained graph (post-A6c) or via BWT run-width selection (pre-A6b) is closed as a net `C + E` lever against `forward-emission`. On `C-A6c + E-walk + E-spelling`, **`forward-emission`** (single-lane + multi-lane forward links, which removes bidirectional wiring/shift overhead and exploits MLP directly during `A7` without adding a contraction pass) remains the unchallenged representation win.

---

## 6. Caveats & scope

- **VM vs i9-10900K:** All measurements ran on a 2-vCPU Haswell Xeon VM (`2.30 GHz`, `256 KiB` L2, `45 MiB` L3, no vPMU); i9 millisecond columns are VM-to-i9 ratio projections anchored at `C_base + E_base = 1,068.0 ms`. Because both machines share a `256 KiB` per-core L2 cache and every tested constructor loses by `1.13–1.29×` (and fails even the bare chase-only floor against `ctrl_fwd_4lane_ora`), the STOP verdict is robust to cross-machine scaling.
- **Scoped STOP (no broad no-go beyond tested mechanisms):** This closes (1) post-A6c whole-`F`-bucket and 1-hop digram return-map contraction (`DictReturnMap` and `Edge16`), and (2) pre-A6b selective `F`-bucket / digram-interval `LIMIT` rules inside Open's auxiliary IET, against matched `forward-emission`. Prior frozen gates and verdicts (`ideal-permutation-structures`, `rauzy-output`, `forward-emission`) remain untouched.
