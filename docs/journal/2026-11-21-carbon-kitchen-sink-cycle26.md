# Carbon Kitchen Sink (Neon) — Cycle 26: Bipartite Locality Asymmetry of `wire_xor` (`EpochTile64`, `SubTile32`, `NoQueue-BiOperad`) & `spelling` Capacity Anomaly Audit

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Testing whether transposing `OutQ16::wire_xor` from Byte-Major (`a = 0..256` outer) to Epoch-Major (`64K` / `32K` L2 tiles) or Run-Major (`QUEUE = false`, sequential `v`) cuts the `+764 µs` bidirectional wiring tax of `CKS-BiOperad`, plus auditing why `CKS-BiOperad`'s `spelling` capacity reached `86,016 B` despite shorter actual length than `master`.

---

## 1. Mathematical & Architectural Findings

### Theorem 26.1 (Bipartite Locality Asymmetry in `OutQ16::wire_xor`)
In `wire_xor`, each retained edge $(u, v)$ performs two RMWs on `edges[0..m)`: `edges[u] ^= pack | v` and `edges[v] ^= u`. There are three canonical loop orderings over the bipartite incidence graph $\{(a, b, u, v)\}$:
1. **Byte-Major (`CKS-BiOperad`, $a \to \mathrm{band} \to v$):** Streams `bands[..]` sequentially, streams `OutQ16` chunks per byte in registers (`0` cursor reloads), keeps `u` contiguous over each band (`mean span ~30`), and sweeps `v` monotonically per byte (`~90` active bytes).
2. **Epoch-Major (`EpochTile64` / `SubTile32`, $\mathrm{tile}(v) \to a \to \mathrm{band}$):** Confines `edges[v] ^= u` to a `256 KiB` (`64K`) or `128 KiB` (`32K`) window using `OutQ16`'s existing `spill.cuts` (`+0 B` heap), at the cost of maintaining `256` live `(Reader, BandCursor)` states (`~14 KiB` stack) and splitting bands across tile boundaries.
3. **Run-Major (`NoQueue-BiOperad`, `QUEUE = false`, $v = 0 \dots m-1$):** Eliminates `OutQ16` (`−6 KiB`) and makes `edges[v] ^= u` a single sequential sweep, but switches active byte `a = L[x]` every thin run (`mean width ~1.6 rows`), turning `bands[cursors[a]]` and `edges[u]` into `~500K` random byte-bucket jumps plus a second `records` decode pass.

**Native Verdict:** Byte-Major strictly dominates both transpositions (`construct`: `6,348.8 µs` vs `6,568.3 µs` `EpochTile64`, `6,686.3 µs` `SubTile32`, and `10,768.7 µs` `NoQueue`). Splitting bands and spilling `256` reader/band cursors costs `+219.5 µs` (`+337.5 µs` at `32K`), while randomising `u` and `bands` every `1.6` rows costs `+4,419.9 µs` (`1.85x`).

### Proposition 26.2 (`spelling` Power-of-2 Doubling Overshoot in `Operad-MN16`)
Auditing `spelling.len()` vs `spelling.capacity()` across all 6 authenticated blocks reveals that `CKS-BiOperad` actually writes **`10%–27%` FEWER `spelling` bytes than `0.5.11` `master`** on every single block (`max = 47,178 B` on `block-0.L` vs `master`'s `58,129 B` on `block-143.L`), yet hit `cap = 86,016 B` (`84 KiB`) because `extend_from_slice`/`extend_from_within` from `cap = 0` landed on non-power-of-2 `43,008` and doubled to `86,016` (`38,838 B` unused slack!). Pre-reserving `48 KiB` (`49,152 B`) or exact power-of-2 sizing reclaims **`36.0 KiB`** immediately (`981.8 -> 945.8 KiB = −19.3%` below `0.5.11` `master`).

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`60` Pairs/Block `1T`, `40` Passes `2T`)

All arms (`#![forbid(unsafe_code)]`) passed byte-for-byte parity on all `6 / 6` Wikidata blocks and `dense-300k`:

| Block / Metric | `A/A` (`M2/M1`) | **`CKS-BiOperad`** (Byte-Major) | `EpochTile64` (`256 KiB` tile) | `SubTile32` (`128 KiB` tile) | `NoQueue-BiOperad` (`QUEUE=false`) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `0.983` | **`0.914x`** (`c=803.4 µs`) | `0.916x` (`c=+19.6`) | `0.933x` (`c=+28.7`) | `1.268x` (`c=+436.7`) |
| `block-2.L` (`1T p10`) | `1.006` | **`0.896x`** (`c=948.0 µs`) | `0.909x` (`c=+19.9`) | `0.915x` (`c=+47.9`) | `1.214x` (`c=+529.7`) |
| `block-85.L` (`1T p10`) | `1.005` | **`0.879x`** (`c=801.8 µs`) | `0.904x` (`c=+32.6`) | `0.920x` (`c=+52.3`) | `1.328x` (`c=+728.4`) |
| `block-143.L` (`1T p10`) | `0.996` | **`0.842x`** (`c=1544.7 µs`) | `0.856x` (`c=+55.5`) | `0.865x` (`c=+89.0`) | `1.179x` (`c=+1009.4`) |
| `block-210.L` (`1T p10`) | `0.981` | **`0.848x`** (`c=1173.9 µs`) | `0.849x` (`c=+61.6`) | `0.857x` (`c=+72.7`) | `1.228x` (`c=+1009.5`) |
| `block-207.L` (`1T p10`) | `1.009` | **`0.852x`** (`c=1077.0 µs`) | `0.882x` (`c=+30.4`) | `0.887x` (`c=+47.0`) | `1.226x` (`c=+706.1`) |
| **`1T` 6-Block Sum (`p10`)** | **`0.9962`** | **`0.8651x`** (`c=6348.8`) | `0.8791x` (`c=+219.5`) | `0.8886x` (`c=+337.5`) | `1.2330x` (`c=+4419.9`) |
| **`dense-300k`** | `1.003` | **`0.775x`** | `0.791x` | `0.800x` | `0.997x` |
| **`2T` Concurrent (`p10 / p50`)** | **`1.0060 / 1.0009`** | **`0.8680 / 0.8669`** | `0.8796 / 0.8780` | `0.8909 / 0.8862` | `1.2326 / 1.2310` |
| **6-Blk Worker Scratch** | `1,171.7 KiB` | `981.8 KiB` (`−16.21%`) | `981.8 KiB` | `981.8 KiB` | `975.8 KiB` (`−16.72%`) |

*(Fresh `spelling.len() / cap` on 6 blocks — `Master` vs `BiOperad`: `block-0` `52,035/65,536` vs `47,178/86,016`; `block-2` `49,396/65,536` vs `41,752/43,008`; `block-85` `31,612/32,768` vs `26,894/34,816`; `block-143` `58,129/65,536` vs `42,463/69,632`; `block-210` `38,703/65,536` vs `29,132/40,960`; `block-207` `55,955/65,536` vs `45,969/69,632`.)*

---

## 3. Cycle 26 Verdicts (One-Line Kills & Leads)

- **Kill (`EpochTile64` / `SubTile32` Epoch-Transposed `wire_xor`):** Killed (`0.8791x` / `0.8886x` vs Byte-Major `CKS-BiOperad`'s `0.8651x`, `construct` `+219.5 µs` / `+337.5 µs` slower) because band-splitting and `256`-cursor state reloads outweigh L2 tiling of `edges[v] ^= u`.
- **Kill (`NoQueue-BiOperad` Run-Major `QUEUE = false` `wire_xor`):** Killed (`1.2330x` 1T, `1.2326x` 2T, `construct` `+4,419.9 µs` slower) because switching byte streams every `~1.6` rows destroys `bands` and `edges[u]` locality.
- **Carry Forward (`spelling` Exact Sizing):** Confirmed `BiOperad`'s `spelling.len() <= 47,178 B` on `6 / 6` blocks (`0.73–0.91x` of `master`), unlocking an immediate `−36.0 KiB` scratch cut (`-> 945.8 KiB = −19.3%`) in Cycle 27.
