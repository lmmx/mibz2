# N-12: Supplied Capability 1 (Outside Review §5.1) — Exact Acyclic LZ77 Positional Copy Plans on the 6 Admitted Blocks

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.1 (`origin/claude/sharp-meitner-cx0v78` `e48f904`): measure descriptor count `q_copy`, coverage of expensive rows (`h₁` and `m₁₆`), copy distances (`D_𝓗` miss proxy), and full-path payoff **before** any acquisition work.*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (exact `O(n)` `SA`/`ISA`/Kasai-`LCP`/PSV-NSV-`LPF` and greedy left-to-right acyclic LZ77 parse at `min_len ∈ {2, 4, 8, 16}`, byte-exact reconstruction asserted `recon == T` on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** **CLOSED for `h₁` at (d) Payoff (and (b) Passes)** — `tested failure within defined scope`.
- Even with an **oracle-supplied** optimal acyclic LZ77 plan at `min_len = 2` (exploiting every 2–3 byte coincidental repeat), **`z / h₁ = 0.54–0.64 > 0.50` on 5 of 6 blocks** (only `block-210.L` hits `0.48`), with median copy distance `86k–99k` bytes (L1/L2-missing source reads!).
- At realistic copy lengths `min_len = 8` (filtering out 2–7 byte coincidences), **`z = 48.5k–123.0k = 1.27–1.51 × h₁`** (worse than `h₁` on 6/6 blocks!), leaving **`76.6–80.5%` of `h₁` as uncovered literals**!
- On `m₁₆` (all retained rows, including `w = 2..16`), `z / m₁₆ = 0.19–0.25` at `min_len = 2` (`0.46–0.60` at `min_len = 8`) reflects copying width-`2..16` thin runs (not `h₁`), while acquiring LZ77 from RLBWT in `O(r)` workspace takes `Ω(n)` time (Nishimoto–Tabei) or `Θ(log n)` passes, failing `K = 2` and `d ≤ d(Open)`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T`, `SA`, `ISA`, width-1 mask `h₁`, and thin mask `m₁₆` (`w ≤ 16` plus `p`) on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline: `OpenCells` (`LIMIT = 16`, `m₁₆ = 89,874–204,281` emit steps; `h₁ = 34,262–97,054`; frozen target `c₁ ≤ h₁/2`, `K = 2`, `S ≤ openPeakScratch`, `d(D, c) ≤ d(Open, c)` and `D_𝓗(D, c) ≤ D_𝓗(Open, c)` per §1a clauses 6 & 6b).
3. **Assumptions knowingly unrealistic:**
   - The acyclic LZ77 plan is **supplied free** (zero acquisition cost, zero pass cost, zero workspace cost for finding `LPF`). Every failure here is therefore a hard **supplied-capability ceiling** independent of acquisition.
   - Left-to-right greedy LZ77 is optimal for unidirectional acyclic parsing; bidirectional macro schemes / attractors `γ` can shrink `z` by at most `O(log(n/γ))` (typically `1.5–2×` on repetitive text), but cannot merge unique random GUID nibbles or unique tokens (`85–89%` of `h₁` unique at length 8, `N-2`).

## 1. Exact byte-verified acyclic LZ77 measurements on the 6 admitted blocks (`[measured]`)

Every parse below was executed forward and asserted byte-exact (`recon == T`, 24/24 parses passing).

### (a) `min_len = 2` (absolute minimum acyclic LZ77 phrase count `z`, including 2–3 byte random coincidences)

| Block | `m₁₆` | `h₁` | `z` (`copy + lit`) | `z / h₁` | `z / m₁₆` | `h₁` copied (interior%) | `m₁₆` copied | Med. copy dist (B) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 35,116 | `20,083` (`19,140 + 943`) | **0.57** | 0.22 | 99.2% (71.0%) | 99.4% | 99,357 |
| `block-2.L` | 112,070 | 34,262 | `21,950` (`20,926 + 1,024`) | **0.64** | 0.20 | 99.1% (70.9%) | 99.4% | 86,016 |
| `block-85.L` | 156,404 | 55,504 | `30,001` (`28,870 + 1,131`) | **0.54** | 0.19 | 99.2% (71.5%) | 99.4% | 98,971 |
| `block-143.L` | 196,841 | 72,578 | `40,664` (`39,349 + 1,315`) | **0.56** | 0.21 | 99.2% (71.3%) | 99.5% | 93,021 |
| `block-210.L` | 204,281 | 97,054 | `46,451` (`45,216 + 1,235`) | **0.48** | 0.23 | 99.5% (71.1%) | 99.5% | 96,860 |
| `block-207.L` | 135,921 | 62,407 | `33,608` (`32,346 + 1,262`) | **0.54** | 0.25 | 99.3% (70.2%) | 99.3% | 90,804 |

### (b) `min_len = 8` (real structural copies, excluding 2–7 byte birthday-paradox collisions)

| Block | `h₁` | `z` (`copy + lit`) | `z / h₁` | `z / m₁₆` | `h₁` copied (uncovered `h₁`) | `m₁₆` copied (uncovered `m₁₆`) | Med. copy dist (B) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | `48,523` (`4,814 + 43,709`) | **1.38** | 0.54 | 22.4% (`27,234`) | 55.5% (`39,954`) | 35,314 |
| `block-2.L` | 34,262 | `51,861` (`5,686 + 46,175`) | **1.51** | 0.46 | 23.1% (`26,345`) | 62.9% (`41,610`) | 33,892 |
| `block-85.L` | 55,504 | `74,547` (`7,244 + 67,303`) | **1.34** | 0.48 | 23.4% (`42,495`) | 58.5% (`64,842`) | 21,763 |
| `block-143.L` | 72,578 | `99,707` (`11,349 + 88,358`) | **1.37** | 0.51 | 23.3% (`55,636`) | 58.2% (`82,216`) | 59,464 |
| `block-210.L` | 97,054 | `123,014` (`10,840 + 112,174`) | **1.27** | 0.60 | 19.5% (`78,117`) | 47.0% (`108,241`) | 35,590 |
| `block-207.L` | 62,407 | `81,876` (`8,694 + 73,182`) | **1.31** | 0.60 | 22.8% (`48,204`) | 49.4% (`68,767`) | 31,176 |

*(Intermediate `min_len = 4`: `z / h₁ = 0.74–1.05`, `h₁` copied `54.5–65.3%`, `med_dist = 74k–100k`; `min_len = 16`: `z / h₁ = 1.42–1.81`, `h₁` copied `16.5–21.3%`.)*

## 2. Supplied-capability payoff & `D_𝓗` interpretation (`[argued]` + `[measured]`)

1. **Why `min_len = 2` still fails `δ = 1/2` on `h₁`:** At `min_len = 2`, a 30-nibble random GUID matches earlier random 2–4 hex substrings by the birthday paradox (~8–10 tiny copies per GUID, `med_dist = 86–99 KB`). Each copy descriptor `(src, len)` is still a paid positional descriptor (`q_copy = z`), and **`z > 0.50 · h₁` on 5/6 blocks** BEFORE charging a single acquisition operation!
2. **Real copies (`min_len ≥ 8`) miss `h₁` completely:** Only `19.5–23.4%` of `h₁` sits in `≥ 8`-byte copies (matching `M-7` and `N-8`: real repeats land in width `≥ 2` runs!). Uncovered `h₁` rows alone are `76.6–80.5%` of `h₁`, and `z = 1.27–1.51 × h₁`.
3. **Cache-miss (`D_𝓗`) check (§1a clause 6b):** Each `min_len = 2` copy descriptor reads from a pseudo-random earlier output offset (`med_dist = 86–99 KB`, exceeding the i9's 32 KiB L1d and spanning most of its 256 KiB L2), while `min_len = 2` saves `m₁₆` emit loads (`0.19–0.25 × m₁₆`) **only if** the LZ77 parse were already sitting in the input (like LZ4/zstd — an R3 format change!). Acquiring the LZ77 parse from a bzip2 BWT column `(L, p)` in `O(r)` space requires `Ω(n)` time (Nishimoto–Tabei) or `Θ(log n)` passes, blowing both `K = 2` and `D_𝓗 ≤ D_𝓗(Open)`.

## 3. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Acyclic positional copy plans / LZ77 / Kempa–Prezza macro schemes (`N-12`, Outside Review §5.1) |
| **Route class** | **R2** (if acquired from `(L, p)`); **R3** if stored in the compressed stream |
| **Gate failed** | **(d) Payoff on `h₁`** (`z / h₁ = 0.54–0.64 > 0.50` on 5/6 blocks even at `min_len = 2` with free oracle; `1.27–1.51 × h₁` at `min_len = 8`, `77–81%` of `h₁` uncovered) and **(b) Passes** (RLBWT → LZ77 takes `Ω(n)` time / `≫ K = 2` passes) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`, 24/24 byte-exact parses): `z = 20,083–46,451` at `min_len = 2`; `z = 48,523–123,014` at `min_len = 8` |
| **Scope** | All acyclic left-to-right positional copy plans (LZ77 / macro schemes) operating on `𝓕` |
| **Escape condition** | Format change (R3: store LZ77/macro descriptors in the stream instead of BWT), OR a bidirectional non-copy positional mechanism acting on the `77–81%` non-repeating `h₁` rows |
| **Reach** | Global for unidirectional acyclic LZ77 copy plans on `𝓕` |
| **Strength** | Measured exact (byte-verified on 6 admitted blocks) + argued for acquisition |

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact byte-verified LZ77 parses on all 6 admitted blocks.
2. **Model assumptions:** Supplied-capability oracle (zero acquisition cost) strictly upper-bounds any realisable decoder.
3. **Method:** Exact `O(n)` `SA`/`LCP`/`LPF` via Kasai + PSV/NSV monotone stacks, verified byte-for-byte.
4. **Outcome:** Cleanly answers Outside Review §5.1: acyclic copy plans fail `δ = 1/2` on `h₁` even as a free oracle (`5/6` blocks at `min_len = 2`, `6/6` at `min_len ≥ 4`), and real copies (`min_len ≥ 8`) cover only `19.5–23.4%` of `h₁`.
