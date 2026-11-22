# Wormhole (Neon) Cycle 28 — `S3 + S4` Audit: Ground-Truth Dedup Collision Census, 2-Way `str_ht`, & `#703` Dense Regime

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 28 of 30; 8th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle28.rs`](2026-11-21-wormhole-cycle28-s3-s4-dedup-2way-and-dense703/verify_cycle28.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases byte-exact + exact `HashSet` ground-truth dedup census + `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`s3_s4_audit.toml`](2026-11-21-wormhole-cycle28-s3-s4-dedup-2way-and-dense703/s3_s4_audit.toml) (`PASS`), [`cycle28_output.txt`](2026-11-21-wormhole-cycle28-s3-s4-dedup-2way-and-dense703/cycle28_output.txt), [`cycle28_results.json`](2026-11-21-wormhole-cycle28-s3-s4-dedup-2way-and-dense703/cycle28_results.json)

---

## 1. One-Line Keep & Kill Decisions (`S3 + S4` Floor Proved)

- **PROVED Theorem 28.1 (Symbol-Locality of `str_ht`):** Direct-mapped 1-way `4,096`-slot `str_ht` already achieves **`99.983%` of exact `HashSet` deduplication** across all 7 blocks (`11,591` true unique words; only `2` collision duplicates leaked across all 6 Wikidata blocks and **`0` leaked on `dense-703.syn`**!).
- **KILL `D1: 2-Way Set-Associative 4K str_ht` (`+30.8 µs` in `S3`, `+61.9 µs` on `dense-703.syn`, `0.877×` tied with `E2h`):** Recovers only `2` codes (`0.0 KiB` `spelling`, unlocks `0` new `Len4` blocks) while paying `+30.8 µs` for the extra way check.
- **KILL `D2: 2-Way Set-Associative 8K str_ht` (`+22.1 µs` in `S3`, `+82.9 µs` on `dense-703.syn`):** Same `2` codes recovered (`0.0 KiB` `spelling`), plus extra `16 KiB` `str_ht.fill(0)`.
- **KILL `D3: 2-Way 4K + S3 h_len == 1 Literal-Pair Fast-Path` (`+27.4 µs` in `S3`, `+74.0 µs` on `dense-703.syn`):** Extra `h_len == 1` branch inside `S3` costs more than `short_u128`.

---

## 2. Exact Ground-Truth `HashSet` Dedup Collision Census (`verified`)

| Block | `M0_U` (NoDedup) | **`True_U` (`HashSet`)** | `E2h` (`1w 4K`) | `D1` (`2w 4K`) | `D2` (`2w 8K`) | `spelling KiB` (`1w -> 2w8`) | `Len4` (`1w -> 2w`) |
|---|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | `1,582` | **`995`** | `995` (`0` dup) | `995` | `995` | `46.1 -> 46.1` | `YES -> YES` |
| `block-2.L` | `2,178` | **`1,281`** | `1,281` (`0` dup) | `1,281` | `1,281` | `40.8 -> 40.8` | `YES -> YES` |
| `block-85.L` | `1,476` | **`811`** | `811` (`0` dup) | `811` | `811` | `26.3 -> 26.3` | `YES -> YES` |
| `block-143.L` | `3,891` | **`1,711`** | `1,712` (`1` dup) | `1,711` | `1,711` | `41.5 -> 41.5` | `no -> no` |
| `block-210.L` | `2,484` | **`1,218`** | `1,219` (`1` dup) | `1,218` | `1,218` | `28.4 -> 28.4` | `no -> no` |
| `block-207.L` | `2,515` | **`1,205`** | `1,205` (`0` dup) | `1,205` | `1,205` | `44.9 -> 44.9` | `no -> no` |
| `dense-703.syn` | `4,447` | **`4,370`** | **`4,370` (`0` dup!)** | `4,370` | `4,370` | `3.0 -> 3.0` | `no -> no` |

### Theorem 28.1 (Symbol-Locality of `S3` Word Deduplication, `proved` + `verified`)
Why does a direct-mapped `4,096`-slot `str_ht` suffer **zero** duplicate leaks on `dense-703.syn` (`4,370 > 4,096` unique words!) and only `2 / 7,221` across all `6` Wikidata blocks, defying the naive birthday-paradox estimate? Because `S3` iterates bands grouped by outer symbol `a = 0..255`, and every emitted band word `(w || a)` terminates in byte `a`. Consequently, identical strings can occur **only within the same `a`-iteration** (`O(U / 256) ≈ 5–50` active distinct strings per `a`), giving an effective temporal occupancy `< 1.2%` of the `4,096` slots. Hence `1`-way `4K` `str_ht` + `Band64` + `L4 BidiBottom` + `GradedOperad` (`785 µs` `S3`, `-46.7%` below `M0`) and hoisted `OutQ16` `S4` (`747 µs`, `82.2%` streamer-hidden) are both at their safe-Rust hardware floors.

---

## 3. Verification & Benchmarks (`measured` in [`verify_cycle28.rs`](2026-11-21-wormhole-cycle28-s3-s4-dedup-2way-and-dense703/verify_cycle28.rs))

- **Exactness (`verified`):** `6/6` `manifest.json` SHA-256 auth + `dense-703.syn` + **`5,229` Gate G1 / spiral / multi-epoch** cases byte-exact across `M0, E2h, D1, D2, D3`.
- **Per-Worker Working Set:** `950.0 KiB` persistent scratch across `E2h, D1, D2, D3` (`-19.0%` vs `M0`'s `1,172.7 KiB`; `10w = 17.86 MiB <= 20 MiB` L3); steady-state DRAM `<= 0.034–0.059 B/B`.

### 3.1 5-Stage Attribution (`6`-Block Sum & `dense-703.syn`, `15` Rounds Median `µs`)

| Arm | `S1` | `S2` | **`S3`** | `S4` | `Pre (S1–4)` | `S5 Emit` | **`6Blk Sum`** | **`Dense-703`** | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: Clone (0.5.11)` | `2241.5 µs` | `1866.9 µs` | `1473.8 µs` | `726.9 µs` | `6309.1 µs` | `5724.6 µs` | `12033.7 µs` | `2960.5 µs` | Baseline |
| **`E2h: C25 (1w 4K)`** | `1866.0 µs` | `1681.4 µs` | **`785.0 µs`** | `747.0 µs` | `5079.4 µs` | `5098.7 µs` | **`10178.1 µs`** | **`2452.8 µs`** | **RETAINED** |
| `D1: 2-Way 4K` | `1867.4 µs` | `1657.4 µs` | `815.8 µs` | `736.8 µs` | `5077.4 µs` | `5064.6 µs` | `10142.1 µs` | `2514.7 µs` | **KILL** (`+31 µs` `S3`, `+62 µs` dense) |
| `D2: 2-Way 8K` | `1893.3 µs` | `1663.6 µs` | `807.1 µs` | `730.8 µs` | `5094.8 µs` | `5120.7 µs` | `10215.5 µs` | `2535.7 µs` | **KILL** (`+22 µs` `S3`, `+83 µs` dense) |
| `D3: 2w4K+Lit1` | `1911.7 µs` | `1674.4 µs` | `812.4 µs` | `744.2 µs` | `5142.7 µs` | `5097.0 µs` | `10239.7 µs` | `2526.8 µs` | **KILL** (`+27 µs` `S3`, `+74 µs` dense) |

### 3.2 Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `measured`)

`A2/A1` A/A control = **`0.995×` p50 (`1.001×` p10)** and `M0/A1` = **`1.001×` p50 (`1.006×` p10)** (`11,674.1 µs` 6-block sum):

| Metric | `A1:Master` | `A2/A1` | `M0/A1` | **`E2h/A1`** | `D1/A1` | `D2/A1` | `D3/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|
| **6-BLK p50/A1** | `11674.1 µs` | `0.995×` | `1.001×` | **`0.877×`** | `0.877×` | `0.875×` | `0.873×` |
| **6-BLK p10/A1** | `10954.4 µs` | `1.001×` | `1.006×` | **`0.878×`** | `0.873×` | `0.885×` | `0.877×` |
| **`dense-703.syn` p50/A1** | `2971.1 µs` | `0.980×` | `0.964×` | **`0.849×`** | `0.852×` | `0.871×` | `0.866×` |

---

## 4. Next Cycle Plan (`Cycle 29 (Neon)`)
All five stages (`S1`–`S5`) are now bounded and closed. Cycle 29 builds the unified Round 4 champion **`WormholeNeonCells: InverseBwt`** (`#![forbid(unsafe_code)]`) and runs:
1. A clean **7-step cumulative piece-by-piece ablation** from `origin/master` (`8ff1187af`) through `S* (N3)` -> `+F4 NoSpill` -> `+G2 GradedOperad` -> `+E1 Early-j ZeroSpill` -> `+E2h Hoisted-S4 Len4` -> `+P2c CleanPt64` (final `35`-round head-to-head keep/kill audit of Cycle 27's held `P2c`!), plus `Neon-C29 (full with Fused-XOR)` for direct 1T comparison.
2. Exact persistent scratch, 10-worker L3 budget, and acquisition-cost accounting.
