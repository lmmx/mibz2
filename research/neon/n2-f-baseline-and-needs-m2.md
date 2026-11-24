# N-2: Baseline on `𝓕` (232 census blocks + 6 admitted blocks), `NEEDS M-2`, and the Stream N ceiling

*Cycle 1 measurement establishing the book for Stream N screening and answering MONODROMY's `NEEDS M-2` (`origin/claude/sharp-meitner-cx0v78` `6737734`, `CLAIMS.md` L9).*

**Script:** [`research/neon/scripts/measure_f_baseline.py`](scripts/measure_f_baseline.py) (standard-library Python, run on `docs/journal/2026-11-24-wikidata-block-census/{census,features}.tsv` and the six columns of `origin/data/admitted-bwt`).

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - **232 distinct two-ended Wikidata blocks (`#805`):** exact `(n, p, m, runs, runs_1, strips)` from [`census.tsv`](../../docs/journal/2026-11-24-wikidata-block-census/census.tsv) joined with 1-worker i9 `emit_ns_per_row` and `construct_ns_per_row` from [`features.tsv`](../../docs/journal/2026-11-24-wikidata-block-census/features.tsv) (averaged over the 4 identical copies per distinct block, `928 / 4 = 232`).
   - **6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`):** raw last columns `block-{0,2,85,143,207,210}.L` and `manifest.json`, joined by `(n, p, runs)` to `#805` `census.tsv`/`features.tsv` (exact 6/6 match).
   - **Missing data note for `NEEDS M-2`:** `census.tsv` stores run-length histograms only, not per-byte counts or raw `.L` columns, and `corpora/wikidata.bz2` is not present in this repository. Therefore `h₁R / h₁` under `σ_hex` is measured **exactly on the 6 admitted blocks** here; computing `h₁R / h₁` on all 232 census blocks requires running a 10-line extension to `examples/census.rs` on `corpora/wikidata.bz2`.
2. **Real code path:**
   - `OpenCells` (`LIMIT = 16`, two-ended emit for `m ≥ 65,536`), 1-worker i9 costs from `emit-cost-sq/run-20261123T110350Z` (`#805`). Total kernel time across the 232 distinct blocks = **276.23 ms** (emit **129.03 ms = 46.7%**, construct **147.20 ms = 53.3%**).
3. **Assumptions knowingly unrealistic:**
   - Content-class attribution (`GUID`, `SHA1`, `Entity_QPL`, `JSON_key`, `Num4+`, `Other`) uses regexes on the inverted text `T` (same methodology as `working-backwards.md` §2 and Record 96 §3); it is diagnostic for the ceiling, never used as a decoder recogniser.
   - Proportional emit-time attribution (`emit_ns * (h₁ / m)`) assumes uniform emit cost across retained rows (consistent with Record 95's flat 4.8–4.9 ns/row at 1 worker, and Record 96 §6 item 4).

## 1. Cost-weighted `h₁` baseline on the 232 distinct census blocks (`[measured]`)

| Quantity | Value across 232 distinct blocks |
|---|---:|
| Total 1W i9 Open kernel time | 276.23 ms (emit 129.03 ms / 46.7%, construct 147.20 ms / 53.3%) |
| Total retained rows `m` | 26,502,469 |
| Total width-1 runs `h₁` | 10,808,830 (`40.78%` of `m`; `71.63%` of all `15,090,427` maximal runs) |
| `h₁ / m` distribution (min / p10 / median / p90 / max) | `0.3015` / `0.3336` / `0.3868` / `0.5043` / `0.8288` |
| **Cost-weighted mean `h₁ / m`** | **`0.4087`** |
| **Cost-weighted emit share of all `h₁` rows** | **`19.09%` of total kernel time** (`40.88%` of emit time) |
| Main mode (`m/n < 0.15`, 187 blocks) | `72.4%` of kernel time; pooled `h₁/m = 0.3964` |
| Dense tail (`m/n ≥ 0.15`, 45 blocks) | `27.6%` of kernel time; pooled `h₁/m = 0.4354` |

## 2. `NEEDS M-2`: `h₁R / h₁` under `σ_hex` and `Pglue` boundary counts on the 6 admitted blocks (`[measured]`)

Under `σ_hex` (`R` = maximal runs of width 1 whose byte is in `[0-9a-fA-F]`, `N` = remaining width-1 runs):

| Block | `n` | `r` | `m₁₆` | `h₁` | `h₁R` (hex) | `h₁R / h₁` | `h₁N` | `h₁N / h₁` | 1W cost (ms) | `Pglue` switches `b` (w1) | `b / h₁N` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 899,931 | 51,184 | 89,874 | 35,116 | 29,638 | **0.8440** | 5,478 | **0.1560** | 1.005 | 8,948 | **1.63** |
| `block-2.L` | 900,000 | 54,532 | 112,070 | 34,262 | 28,254 | **0.8246** | 6,008 | **0.1754** | 1.207 | 8,746 | **1.46** |
| `block-85.L` | 899,935 | 79,375 | 156,404 | 55,504 | 45,874 | **0.8265** | 9,630 | **0.1735** | 1.352 | 13,798 | **1.43** |
| `block-143.L` | 899,869 | 108,335 | 196,841 | 72,578 | 58,922 | **0.8118** | 13,656 | **0.1882** | 2.151 | 18,100 | **1.33** |
| `block-210.L` | 899,959 | 128,732 | 204,281 | 97,054 | 82,630 | **0.8514** | 14,424 | **0.1486** | 2.034 | 21,750 | **1.51** |
| `block-207.L` | 899,998 | 87,111 | 135,921 | 62,407 | 49,791 | **0.7978** | 12,616 | **0.2022** | 1.450 | 15,012 | **1.19** |

Cost-weighted over the 6 admitted blocks (total 9.199 ms): `h₁R / h₁` = **0.8256**, `h₁N / h₁` = **0.1744**.

### Content-class decomposition of width-1 runs (`[measured]`)

| Block | `GUID_rand` | `GUID_fix` | `SHA1` | `Entity_QPL` | `JSON_key` | `Num4+` | `Other` | Non-GUID `/ h₁` | All-row R↔non-R switches `/ h₁` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 57.9% | 8.7% | 4.8% | 6.7% | 1.8% | 8.9% | 11.2% | **33.4%** | 0.537 |
| `block-2.L` | 53.6% | 8.0% | 7.3% | 5.0% | 1.5% | 8.6% | 16.1% | **38.5%** | 0.543 |
| `block-85.L` | 50.1% | 7.3% | 1.5% | 4.1% | 1.3% | 11.9% | 23.9% | **42.6%** | 0.537 |
| `block-143.L` | 23.4% | 3.5% | 8.3% | 6.2% | 1.5% | 12.3% | 44.8% | **73.1%** | 0.525 |
| `block-210.L` | 52.6% | 7.1% | 4.9% | 3.5% | 1.0% | 9.8% | 21.1% | **40.3%** | 0.520 |
| `block-207.L` | 42.5% | 6.2% | 8.2% | 3.0% | 3.2% | 9.1% | 27.8% | **51.3%** | 0.513 |

Inside `h₁R` (`σ_hex`):
- `block-0.L`: GUID 72.8%, SHA1 5.7%, QPL 6.3%, Num4+ 10.6%, OtherHex 4.6%
- `block-2.L`: GUID 68.9%, SHA1 8.8%, QPL 4.9%, Num4+ 10.4%, OtherHex 7.0%
- `block-85.L`: GUID 64.3%, SHA1 1.8%, QPL 4.6%, Num4+ 14.3%, OtherHex 15.0%
- `block-143.L`: GUID 30.6%, SHA1 10.2%, QPL 6.9%, Num4+ 15.2%, OtherHex 37.1%
- `block-210.L`: GUID 65.1%, SHA1 5.7%, QPL 3.9%, Num4+ 11.6%, OtherHex 13.7%
- `block-207.L`: GUID 56.5%, SHA1 10.3%, QPL 3.4%, Num4+ 11.5%, OtherHex 18.3%

## 3. Three immediate consequences for the Main Theorem (`[proved]` from the numbers above)

1. **`δ = 1/2` is unreachable with `aR = 1` under `σ_hex` — and even under an oracle GUID/non-GUID split:**
   - Under `σ_hex` + `aR = 1`, even with `aN = 0` and `c₁(Pglue) = 0`, the worst-case block (`block-210.L`) caps the universal margin across `𝓕` at **`δ ≤ 0.1486`** (`14.9%`).
   - Under an oracle split separating true GUIDs from non-GUIDs (`aR = 1`, `aN = 0`, `c₁(Pglue) = 0`), `block-0.L` (`66.6%` GUID) caps the universal margin across `𝓕` at **`δ ≤ 0.334`** (`33.4%`).
   - Therefore, meeting MONODROMY's proposed `δ = 1/2` on `𝓕` **requires** `aR < 1` (breaking Stream R), whereas a Stream-N-only victory on `𝓕` must set **`δ ≤ 0.14` under `σ_hex`** (or `δ ≤ 0.30` with a sharper record-local split `σ` that separates SHA1/QPL/decimal/prose `[a-f]` from GUIDs).
2. **`Pglue` kills `σ_hex` unless gluing is positional (`c₁(Pglue) = 0`):**
   - On every admitted block, width-1 orbit switches between `R` and `N` under `σ_hex` satisfy **`b = 1.19–1.63 × h₁N`** (and full-orbit switches are `0.513–0.543 × h₁`).
   - If `Pglue` pays even 1 width-1 event per `N↔R` switch, `c₁(DR) + c₁(Pglue) > h₁` even when `c₁(DN) = 0`!
   - Thus a valid split `σ` for non-positional gluing would have to produce **long contiguous orbit segments** (`b ≪ h₁N`), which a byte-local rule like `σ_hex` cannot do because hex characters `[0-9a-fA-F]` appear inside English words, numbers, and timestamps interspersed with non-hex characters.
3. **Real-world utility ceiling of Stream N on `#805` (receipts, outside the theorem):**
   - Total `h₁` emit work is `19.09%` of 1W Open kernel time across the 232 blocks.
   - Under `σ_hex`, Stream N (`~17.4%` of `h₁`) represents **`~3.3%` of 1W Open kernel time** (hard ceiling even if Stream N + its gluing cost 0 ns).
   - Under an oracle non-GUID split (`33.4%–73.1%` of `h₁`, cost-weighted `~48%` on the 6 admitted blocks), non-GUID width-1 rows represent **`~9%` of 1W Open kernel time** (still below G4's 10% end-to-end bar unless the mechanism also removes width `2..16` thin rows or construction cost!).

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** 232 census blocks and 6 admitted blocks joined with 0 discrepancies; `corpora/wikidata.bz2` is absent locally, so `h₁R` on the remaining 226 census blocks is flagged as unmeasured here.
2. **Model assumptions:** Regex classification of `T` matches Record 96's GUID shares exactly (0.666, 0.615, 0.574, 0.269, 0.597, 0.487).
3. **Method:** Exact Python inversion and counting (`measure_f_baseline.py`).
4. **Outcome:** Delivers `NEEDS M-2` on the 6 admitted blocks + full census cost weights on 232 blocks, and exposes two structural constraints on the main theorem (`δ` ceiling and `Pglue` switch explosion under `σ_hex`).
