# N-15: `NEEDS M-8` — i9-10900K Memory Model `𝓗` and Open's Dependent-Miss Counts `D_𝓗(Open, c)`

*Cycle 3 measurement discharging MONODROMY's `NEEDS M-8` (`origin/claude/sharp-meitner-cx0v78` `620eacc`, `CLAIMS.md` L27) for §1a clause 6b (`D_𝓗(D, c) ≤ D_𝓗(Open, c)`).*

**Script:** [`research/neon/scripts/measure_needs_m8.py`](scripts/measure_needs_m8.py) (exact LRU cache-line simulation of `emit_bidirectional` in `src/open.rs:919–970` on the six admitted blocks of `origin/data/admitted-bwt`).

## 0. The book (METHOD.md §6a)

1. **Machine model `𝓗` (Intel Core i9-10900K, Comet Lake — from `bench/moonshot/calibration.json` & `PROOF-PROGRAMME.md` §5.10):**
   - **Cache line size:** `B = 64` bytes (`16` `u32` `edges` entries per line).
   - **L1d per core:** `32 KiB` (`512` lines), latency 5 cycles.
   - **L2 per core:** `256 KiB` (`4,096` lines), latency 14 cycles.
   - **L3 shared:** `20 MiB` (`20,480 KiB`) ⇒ **`2,048 KiB` (`32,768` lines) per worker at `W = 10` workers**, latency 42 cycles.
   - **DRAM ceiling:** `33.39 GB/s`, idle latency 437.7 cycles.
2. **Real code path:**
   - `OpenCells` `emit_bidirectional` (`src/open.rs:919–970`, `LIMIT = 16`): `edges: Vec<u32>` (`4m` bytes), XOR-linked two-ended walk alternating `edges[front]` and `edges[back]` until the two ends meet (`m` total dependent loads on `edges[]`).
3. **Assumptions knowingly unrealistic:**
   - Ideal fully-associative LRU per cache level (vs 8-way L1d / 4-way L2 / 16-way L3 on Comet Lake), and models `edges[]` emit dependent loads only (not `inline`/`spelling` loads or Pass 1–4 construction misses). Under cold start for `edges[]`, L3 compulsory fills equal `⌈m/16⌉` (and co-resident `out[n]` sequential writes are simulated explicitly inside the 2,048 KiB L3 slice).

## 1. Simulated dependent-miss counts `D_𝓗(Open, c)` on the six admitted blocks (`[measured]`)

| Block | `m₁₆` | `edges` (KiB) | Unique 64B lines | `D_L1d` (`%m₁₆`) | **`D_L2` (`%m₁₆`)** | `D_L3` cold | **`D_L3` co-res at 10W (`%m₁₆`)** | `D_L2 / h₁` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 351.1 | 5,618 | 58,547 (`65.1%`) | **14,630 (`16.3%`)** | 5,618 | **5,618 (`6.3%`)** | 0.42 |
| `block-2.L` | 112,070 | 437.8 | 7,005 | 72,721 (`64.9%`) | **27,293 (`24.4%`)** | 7,005 | **7,005 (`6.3%`)** | 0.80 |
| `block-85.L` | 156,404 | 611.0 | 9,776 | 90,017 (`57.6%`) | **42,608 (`27.2%`)** | 9,776 | **9,776 (`6.3%`)** | 0.77 |
| `block-143.L` | 196,841 | 768.9 | 12,303 | 151,662 (`77.0%`) | **86,903 (`44.1%`)** | 12,303 | **12,303 (`6.3%`)** | 1.20 |
| `block-210.L` | 204,281 | 798.0 | 12,768 | 159,851 (`78.3%`) | **95,695 (`46.8%`)** | 12,768 | **12,768 (`6.3%`)** | 0.99 |
| `block-207.L` | 135,921 | 530.9 | 8,496 | 97,343 (`71.6%`) | **41,813 (`30.8%`)** | 8,496 | **8,496 (`6.3%`)** | 0.67 |

## 2. Structural readings for §1a clause 6b (`[measured]` + `[argued]`)

1. **L2 misses (`D_L2`) are the active dependent-miss baseline:** `edges[]` (`351–798 KiB`) exceeds the 256 KiB L2 on all 6 blocks, producing **`14,630–95,695` dependent L2 misses** (`16.3–46.8%` of `m₁₆`, or `0.42–1.20 × h₁`).
2. **L3 capacity boundary at `W = 10` (`2,048 KiB`/worker) explains Record 95:** Even with co-resident `900 KiB` `out[]` sequential writes sharing the `2,048 KiB` L3 slice, `edges[]` (`351–798 KiB`) suffers **zero capacity evictions** (`D_L3_co = ⌈m/16⌉ = 6.25%` of `m₁₆`, compulsory cold fills only). Adding `T²`'s extra `8m` bytes (`702–1,596 KiB`) pushes total live footprint (`edges + T² + out`) above `2,048 KiB` on blocks with `m ≥ 130k`, triggering the L3 capacity cliff observed in Record 95.
3. **Recommended fixed `𝓗` for §1a clause 6b:** State both `D_L2(D, c) ≤ D_L2(Open, c)` (256 KiB per-core L2, 64B line) and `D_L3,10W(D, c) ≤ D_L3,10W(Open, c)` (2,048 KiB per-worker L3 slice at `W = 10`, 64B line, counting all working arrays + `out[]`), so a candidate can neither inflate L2 dependent misses nor cross the 2 MiB/worker L3 cliff.

## 3. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact Open retained-row cycle and machine constants from `calibration.json` and `src/open.rs`.
2. **Model assumptions:** Fully-associative LRU is slightly optimistic vs 4-way L2 / 16-way L3, so real set-associative conflict misses are slightly higher than reported.
3. **Method:** Exact two-ended `emit_bidirectional` simulation (`measure_needs_m8.py`).
4. **Outcome:** Discharges `NEEDS M-8` cleanly on all 6 admitted blocks.
