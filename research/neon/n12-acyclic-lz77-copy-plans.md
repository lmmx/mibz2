# N-12: Supplied Capability 1 (Outside Review §5.1) — Exact Acyclic LZ77 Positional Copy Plans on the 6 Admitted Blocks

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.1 (`origin/claude/sharp-meitner-cx0v78` `e48f904`): measure descriptor count `q_copy`, coverage of expensive rows (`h₁` and `m₁₆`), copy distances (`D_𝓗` miss proxy), and full-path payoff **before** any acquisition work.*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (repaired in **N-16**: exact `O(n)` SA-IS non-cyclic `SA_nc`/`ISA_nc` with an `O(n)` lexicographic order certificate, Kasai `LCP`, PSV/NSV `LPF`, automated regression test `test_n12_witness()` on `L = baaaa, p = 2` plus 47 brute-force string checks, and greedy left-to-right acyclic LZ77 parse at `min_len ∈ {2, 4, 8, 16}`, byte-exact reconstruction asserted `recon == T` on all 6 admitted blocks of `origin/data/admitted-bwt`).

**N-16 Repair & Verdict:** **Closure STANDS for unidirectional acyclic left-to-right LZ77 copy plans on `h₁` at (d) Payoff (and (b) Passes)** — `tested failure within defined scope`.
- **Bug repaired (`[proved]`):** Cycle 3's script passed BWT cyclic-rotation order `SA_cyc` into non-cyclic `compute_lcp_kasai` / `compute_lpf`. On the round-2 witness `L = baaaa`, `p = 2` (`T = aabaa`), `SA_cyc = [3, 4, 0, 1, 2]` yielded buggy `LPF = [0, 1, 0, 1, 1]` (missing the length-2 copy `T[3:5] = "aa"`), whereas true non-cyclic `SA_nc = [4, 3, 0, 1, 2]` yields `LPF = [0, 1, 0, 2, 1]`. Both the bug reproduction and the repaired output are now enforced by `test_n12_witness()`.
- **Pre-declared decision rule (`CLAIMS.md` `86eb9b8`, fixed before rerun):** Closure on `h₁` under `δ = 1/2` stands iff after repairing `SA`/`LCP`/`LPF` to true non-cyclic suffix order, `z / h₁ > 0.50` on at least one admitted block at `min_len = 2`.
- **Rerun outcome (`[measured]`):** Across all 6 admitted blocks and all four thresholds `min_len ∈ {2, 4, 8, 16}`, exact non-cyclic `z` is unchanged (`Δz = 0` on `24/24` parses; only tie-break `med_dist` shifts slightly on `block-2.L` `-3 B` and `block-85.L` `-16 B`, because `SA_cyc` and `SA_nc` only diverge near the end-of-block boundary of a 900 KB block).
  - At `min_len = 2` (free oracle exploiting 2–3 B coincidental repeats): **`z / h₁ = 0.54–0.64 > 0.50` on 5 of 6 blocks** (`0.48` on `block-210.L`), median copy distance `86k–99k` bytes.
  - At `min_len = 8` (filtering out 2–7 B birthday collisions): **`z = 48.5k–123.0k = 1.27–1.51 × h₁`** on 6/6 blocks, leaving **`76.6–80.5%` of `h₁` as uncovered literals**.
  - On `m₁₆`, `z / m₁₆ = 0.19–0.25` at `min_len = 2` (`0.46–0.60` at `min_len = 8`) reflects copying width-`2..16` thin runs, while acquiring LZ77 from RLBWT in `O(r)` workspace takes `Ω(n)` time or `Θ(log n)` passes (`[argued]`), failing `K = 2` and `d ≤ d(Open)`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T`, BWT cyclic `ISA_cyc` (to map text position `j` to its emitting BWT row `ISA_cyc[(j + 1) mod n]`), exact non-cyclic `SA_nc`/`ISA_nc` (via SA-IS, verified by `O(n)` adjacent-rank order certificate), width-1 mask `h₁`, and thin mask `m₁₆` (`w ≤ 16` plus `p`) on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline: `OpenCells` (`LIMIT = 16`, `m₁₆ = 89,874–204,281` emit steps; `h₁ = 34,262–97,054`; frozen target `c₁ ≤ h₁/2`, `K = 2`, `S ≤ openPeakScratch`, `d(D, c) ≤ d(Open, c)` and `D_𝓗(D, c) ≤ D_𝓗(Open, c)` per §1a clauses 6 & 6b).
3. **Assumptions knowingly unrealistic:**
   - The acyclic LZ77 plan is **supplied free** (zero acquisition cost, zero pass cost, zero workspace cost for finding `LPF`). Every failure here is therefore a **supplied-capability ceiling** for unidirectional greedy LZ77 parsing independent of acquisition.
   - Left-to-right greedy LZ77 is optimal for unidirectional acyclic parsing (`[proved]`, Storer–Szymanski); bidirectional macro schemes / attractors `γ` can shrink `z` by up to `O(log(n/γ))` on repetitive text (`[argued]`), so this obstruction is scoped to unidirectional acyclic LZ77 (while bidirectional schemes still face the `76.6–80.5%` non-repeating `h₁` ceiling at length 8 `[argued]`).

## 1. Exact byte-verified non-cyclic acyclic LZ77 measurements on the 6 admitted blocks (`[measured]`, N-16 rerun)

Every parse below used exact non-cyclic `SA_nc`/`LCP`/`LPF`, passed the `O(n)` SA order certificate, and was executed forward and asserted byte-exact (`recon == T`, 24/24 parses passing).

### (a) `min_len = 2` (absolute minimum unidirectional acyclic LZ77 phrase count `z`, including 2–3 byte random coincidences)

| Block | `m₁₆` | `h₁` | `z` (`copy + lit`) | `z / h₁` | `z / m₁₆` | `h₁` copied (interior%) | `m₁₆` copied | Med. copy dist (B) | `Δz` vs N-12 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 35,116 | `20,083` (`19,140 + 943`) | **0.57** | 0.22 | 99.2% (71.0%) | 99.4% | 99,357 | 0 |
| `block-2.L` | 112,070 | 34,262 | `21,950` (`20,926 + 1,024`) | **0.64** | 0.20 | 99.1% (70.9%) | 99.4% | 86,013 | 0 |
| `block-85.L` | 156,404 | 55,504 | `30,001` (`28,870 + 1,131`) | **0.54** | 0.19 | 99.2% (71.5%) | 99.4% | 98,955 | 0 |
| `block-143.L` | 196,841 | 72,578 | `40,664` (`39,349 + 1,315`) | **0.56** | 0.21 | 99.2% (71.3%) | 99.5% | 93,021 | 0 |
| `block-210.L` | 204,281 | 97,054 | `46,451` (`45,216 + 1,235`) | **0.48** | 0.23 | 99.5% (71.1%) | 99.5% | 96,860 | 0 |
| `block-207.L` | 135,921 | 62,407 | `33,608` (`32,346 + 1,262`) | **0.54** | 0.25 | 99.3% (70.2%) | 99.3% | 90,804 | 0 |

### (b) `min_len = 8` (real structural copies, excluding 2–7 byte birthday-paradox collisions)

| Block | `h₁` | `z` (`copy + lit`) | `z / h₁` | `z / m₁₆` | `h₁` copied (uncovered `h₁`) | `m₁₆` copied (uncovered `m₁₆`) | Med. copy dist (B) | `Δz` vs N-12 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | `48,523` (`4,814 + 43,709`) | **1.38** | 0.54 | 22.4% (`27,234`) | 55.5% (`39,954`) | 35,314 | 0 |
| `block-2.L` | 34,262 | `51,861` (`5,686 + 46,175`) | **1.51** | 0.46 | 23.1% (`26,345`) | 62.9% (`41,610`) | 33,892 | 0 |
| `block-85.L` | 55,504 | `74,547` (`7,244 + 67,303`) | **1.34** | 0.48 | 23.4% (`42,495`) | 58.5% (`64,842`) | 21,763 | 0 |
| `block-143.L` | 72,578 | `99,707` (`11,349 + 88,358`) | **1.37** | 0.51 | 23.3% (`55,636`) | 58.2% (`82,216`) | 59,464 | 0 |
| `block-210.L` | 97,054 | `123,014` (`10,840 + 112,174`) | **1.27** | 0.60 | 19.5% (`78,117`) | 47.0% (`108,241`) | 35,590 | 0 |
| `block-207.L` | 62,407 | `81,876` (`8,694 + 73,182`) | **1.31** | 0.60 | 22.8% (`48,204`) | 49.4% (`68,767`) | 31,176 | 0 |

*(Intermediate `min_len = 4`: `z / h₁ = 0.74–1.05`, `h₁` copied `54.5–65.3%`, `med_dist = 74k–100k`, `Δz = 0`; `min_len = 16`: `z / h₁ = 1.42–1.81`, `h₁` copied `16.5–21.3%`, `Δz = 0`.)*

## 2. Supplied-capability payoff & `D_𝓗` interpretation (`[argued]` + `[measured]`)

1. **Why `min_len = 2` still fails `δ = 1/2` on `h₁` (`[measured]` + `[argued]`):** At `min_len = 2`, a 30-nibble random GUID matches earlier random 2–4 hex substrings by the birthday paradox (~8–10 tiny copies per GUID, `med_dist = 86–99 KB`). Each copy descriptor `(src, len)` is still a paid positional descriptor (`q_copy = z`), and **`z > 0.50 · h₁` on 5/6 blocks** BEFORE charging a single acquisition operation.
2. **Real copies (`min_len ≥ 8`) miss `h₁` (`[measured]`):** Only `19.5–23.4%` of `h₁` sits in `≥ 8`-byte copies (matching `M-7` and `N-8`: real repeats land in width `≥ 2` runs). Uncovered `h₁` rows alone are `76.6–80.5%` of `h₁`, and `z = 1.27–1.51 × h₁`.
3. **Cache-miss (`D_𝓗`) and acquisition check (`[argued]`):** Each `min_len = 2` copy descriptor reads from a pseudo-random earlier output offset (`med_dist = 86–99 KB`, exceeding L1d and spanning most of 256 KiB L2), while acquiring the LZ77 parse from `(L, p)` in `O(r)` workspace requires `Ω(n)` time (Nishimoto–Tabei) or `Θ(log n)` passes, failing `K = 2`.

## 3. Obstruction table entry (repaired in N-16, all six fields)

| Field | Value |
|---|---|
| **Framework** | Unidirectional acyclic positional copy plans / greedy LZ77 (`N-12`, repaired in `N-16`) |
| **Route class** | **R2** (if acquired from `(L, p)`); **R3** if stored in the compressed stream |
| **Gate failed** | **(d) Payoff on `h₁`** (`z / h₁ = 0.54–0.64 > 0.50` on 5/6 blocks even at `min_len = 2` with free oracle; `1.27–1.51 × h₁` at `min_len = 8`, `76.6–80.5%` of `h₁` uncovered) and **(b) Passes** (RLBWT → LZ77 takes `Ω(n)` time / `≫ K = 2` passes) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`, repaired non-cyclic SA-IS + `O(n)` SA certificate + `test_n12_witness()` on `L = baaaa, p = 2`, 24/24 byte-exact parses): `z = 20,083–46,451` at `min_len = 2`; `z = 48,523–123,014` at `min_len = 8` |
| **Scope** | Unidirectional acyclic left-to-right positional copy plans (LZ77) operating on the 6 admitted blocks of `𝓕` (bidirectional macro schemes / attractors `γ` are argued via the `76.6–80.5%` non-repeating length-8 `h₁` floor, not exhaustively optimised) |
| **Escape condition** | Format change (R3: store copy descriptors in the compressed stream), OR a bidirectional non-copy mechanism acting on the `76.6–80.5%` of `h₁` outside `≥ 8`-byte repeats |
| **Reach** | **Global for unidirectional acyclic LZ77 copy plans on the 6 admitted blocks** (local/argued for bidirectional macro schemes) |
| **Strength** | **Measured exact** on 6 admitted blocks (exact non-cyclic SA-IS + byte-exact verification + `L = baaaa, p = 2` regression test) + **argued** (acquisition `K = 2` and bidirectional schemes) |
| **Transferable insight** | Output-to-output copying IS a genuine exact write mechanism outside `Λ′` (see `N-19`: one copy can write repeated width-1 boundary bytes without any rigid fact through their rows); its failure on `𝓕` is quantitative (random GUID nibbles force `min_len = 2` copies of length 2–4, making `z > h₁/2`), not a calculus-level impossibility — and byte-exact reconstruction alone never certifies phrase-count optimality (requiring an independent `O(n)` suffix-order certificate) |

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact inverted texts `T` and BWT width-1/thin masks on all 6 admitted blocks.
2. **Model assumptions:** Supplied-capability oracle (zero acquisition cost) upper-bounds any realisable unidirectional LZ77 decoder.
3. **Method (repaired in N-16):** Exact `O(n)` SA-IS non-cyclic suffix array verified by an `O(n)` adjacent-rank order certificate, Kasai `LCP`, PSV/NSV `LPF`, regression-tested against `L = baaaa, p = 2` (`LPF = [0,1,0,2,1]`) and 47 brute-force string checks.
4. **Outcome (N-16):** Cyclic-vs-suffix bug fixed and locked by automated witness test. Exact `z` is unchanged across all 6 blocks (`Δz = 0`), so **the closure of unidirectional acyclic LZ77 copy plans on `h₁` stands** under our pre-declared decision rule.

