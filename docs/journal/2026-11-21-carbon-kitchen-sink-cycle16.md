# Carbon Kitchen Sink — Cycle 16: Right-Cancellative Myhill–Nerode Arena Quotient (`MN16-Direct`) on `0.5.11`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink` (merged with `origin/master` at `90e671966`, `#692` memory diet + `#703` dense admission)  
**Mandate:** Priority native revisit of Cycle-12's `MN16` dictionary memo against `0.5.11` `master` (`Open<16, true, true>` with `OutQ16` + `EdgesMux`), with an in-binary A/A control, stage attribution, dense-band & spiral stress tests, and exact per-buffer scratch accounting.

---

## 1. Mathematical Foundation: Right-Cancellative Suffix Stratification & SLP Memo-Tape Safety

After `#692` (`OutQ16`) and `#703` (`EdgesMux` + width-certified `Elim::<Narrow>`), `Open`'s per-worker persistent scratch fell to `1,172.7 KiB` across the 6 authenticated 900 KB Wikidata blocks. However, [`endpoint.rs`](../../../../src/open/endpoint.rs) (`lines 581–592`) still assigns a distinct word code (`256 + i`), expands `e.append(...)`, appends to `spelling`, pushes to `offsets`, and populates a 16-byte `inline16` slot for **every non-literal band**, without quotienting by syntactic/free-monoid equality. Consequently:
1. On `0.5.11` `master`, `inline16 + lens` still spills the i9-10900K's `32 KiB` L1D cache on **4 of 6** admitted blocks (`36.2 KiB` on `block-2`, `41.2 KiB` on `block-210`, `41.8 KiB` on `block-207`, and `64.6 KiB` on `block-143`, which also spills the Xeon 8481C's `48 KiB` L1D), and reaches `102.7 KiB` (`5,929` words) in `#703`'s dense admission band (`r/n = 0.216`).
2. When `lens[code] == LONG` (`word.len() > 16`), `inline16[code]` (`16 bytes`) sits completely unread in [`emit`](../../../../src/open.rs), while `Kernel` maintains a separate `offsets: Vec<u32>` (`16 KiB`).

Three algebraic observations turn `MN16` on `0.5.11` into a zero-allocation, guaranteed-$O(1)$-probe construction:

### Theorem 16.1 (Right-Cancellative Byte-Epoch Stratification of Return Words)
In [`induced_with`](../../../../src/open/endpoint.rs), every non-literal band emitted during outer iteration $a \in \{0, \dots, 255\}$ spells a return word $w = u \cdot a \in \Sigma^+$ where $u \in \Sigma^+$ is the image of `node_word` under the SLP homomorphism $\psi : V_{\mathrm{Elim}} \to \Sigma^+$. Because the free monoid $\Sigma^*$ is right-cancellative:
$$u_1 \cdot a_1 = u_2 \cdot a_2 \iff a_1 = a_2 \;\wedge\; u_1 = u_2.$$
Therefore:
1. Return words from distinct outer-loop bytes $a_1 \ne a_2$ **never collide** in $\Sigma^+$: global across-block deduplication reduces without loss to **within-byte** deduplication on $u \in \Sigma^+$.
2. Tagging a tiny `512`-slot (`2 KiB`) direct-mapped word cache `str_ht` with the byte epoch `(a as u32) << 16` (`((h8) << 24) | ((a as u32) << 16) | (uidx + 1)`) automatically invalidates prior bytes without per-byte `memset` and restricts active occupancy in `str_ht` to the `~5–150` unique prefixes of byte `a` alone (load factor `< 10%` on typical bytes), capturing **`99.90%–100.0%`** of exact collision-free deduplication in **strictly 1 probe (`O(1)` worst-case, zero probe loops)**.

### Theorem 16.2 (SLP Memo-Tape Truncation Safety)
In [`Elim::append`](../../../../src/open/endpoint.rs), each rule $r$ records its first expansion offset in `rule[3]` inside `spelling` and thereafter copies via `dest.extend_from_within(rule[3]..rule[3] + rule[2])`. When `node_word <= 256` or `e.a[e.rule(node_word) + 3] != u32::MAX` **prior** to calling `e.append(node_word, ...)`, `e.append` executes exclusively the `else` (`extend_from_within`) branch and writes **no** `rule[3]` pointer into `spelling[start..]`. Hence on a duplicate word hit:
- If `already_memo` holds, `spelling.truncate(start)` is 100% alias-safe (`0` bytes retained).
- Otherwise (`first expansion of node_word`), `spelling.pop()` removes the trailing byte `a` while preserving `spelling[start..start + height]` for future `rule[3]` back-references.

### Proposition 16.3 (Arena Hosting & `LONG` Offset Folding)
1. **Zero-Allocation Arena Hosting:** After `Elim::eliminate` terminates, `edges[ns + e.top ..]` is dead until `wire`. On all 6 admitted blocks (`m` in `89,874..204,281`), `ns + e.top <= 106,867`, leaving `25,942..161,000+` already-allocated `u32` words in `edges` to host `pair_ht` (`2 KiB`–`8 KiB`), `str_ht` (`2 KiB`–`16 KiB`), and `u_meta` (`[start, len]` for exact `inline16.reserve_exact(unique_words)`) at **zero extra heap bytes**. When `e.a.len() < etop + SCRATCH_NEED` (only possible on tiny-$m$ blocks where `Dict << L1D`), `MN16-Direct` bypasses the tables so `edges` never grows by a single byte.
2. **Zero-`offsets` Folding:** Whenever `wlen > 16` (`lens[code] == LONG`), storing `[start.to_le_bytes(), end.to_le_bytes()]` inside the otherwise-dead `inline16[code][0..8]` eliminates `Kernel::offsets: Vec<u32>` (`-16 KiB`) while leaving the hot `len != LONG` path of `emit` instruction-for-instruction unchanged.

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`A/B + A/A`)

All arms were compiled in a single `#![forbid(unsafe_code)]` release binary (`rustc 1.100.0 -C opt-level=3 -C target-cpu=native -C codegen-units=1`) containing two independent copies of verbatim `origin/master` (`90e671966`) (`MasterA1`, `MasterA2` for interleaved A/A control) alongside four `MN16` variants:
- `PairOnly`: 1-probe `(node_word, a)` direct-mapped cache only (no string table) + Zero-`offsets`.
- `MN16-Linear`: Two-tier `(node_word, a)` + open-addressed linear-probing `str_ht` + Zero-`offsets`.
- `MN16-Direct` (`24 KiB` & `4 KiB` right-cancellative 1-probe direct-mapped variants): Two-tier 1-probe `(node_word, a)` + 1-probe `str_ht` in dead `edges` + Zero-`offsets`.
- `MN16-OutTape` (`MN16-Diet`): `MN16` using unused `out[q_bytes..]` as temporary `e.append` tape so `spelling` retains only `LONG` (`> 16 B`) unique words.

### 2.1 Word Quotient, L1D Dictionary Footprint, and Scratch Audit

Byte-for-byte parity (both fresh `Kernel::default()` and reused `Kernel`) passed `8 / 8` across all 6 authenticated Wikidata blocks, an authentic-derived dense block (`dense-300k`, `r/n = 0.216`, inside `#703`'s `[0.15, 0.28)` dense band), and a spiral stress block (`spiral-200k`, `widest > 32,768`):

| Block | `m` | `master` Words | `PairOnly` Words | `MN16-Direct` (`24K` / `4K` RC) | Exact Unique Words | `master` `inline16+lens` | `MN16-Direct` `inline16+lens` | `master` Fresh Scratch | `MN16-Direct` Fresh Scratch |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `89,874` | `1,326` | `1,146` | `739` / `739` | `739` | `26.3 KiB` | **`16.5 KiB`** | `566.4 KiB` | `548.7 KiB` |
| `block-2.L` | `112,070` | `1,922` | `1,637` | `1,025` / `1,028` | `1,025` | `36.2 KiB` (>32K) | **`21.3 KiB`** | `712.0 KiB` | `689.1 KiB` |
| `block-85.L` | `156,404` | `1,220` | `1,012` | `555` / `555` | `555` | `24.5 KiB` | **`13.5 KiB`** | `761.5 KiB` | `742.5 KiB` |
| `block-143.L` | `196,841` | `3,635` | `3,089` | `1,455` / `1,457` | `1,455` | `64.6 KiB` (>48K) | **`28.4 KiB`** | `1,143.6 KiB` | `1,091.4 KiB` |
| `block-210.L` | `204,281` | `2,228` | `1,868` | `963` / `963` | `962` | `41.2 KiB` (>32K) | **`20.2 KiB`** | `1,085.3 KiB` | `1,048.3 KiB` |
| `block-207.L` | `135,921` | `2,259` | `1,841` | `949` / `949` | `949` | `41.8 KiB` (>32K) | **`20.0 KiB`** | `818.8 KiB` | `781.1 KiB` |
| `dense-300k` (`0.216`) | `110,652` | `5,929` | `5,057` | `2,871` / `2,885` | `2,870` | `102.7 KiB` | **`51.9 KiB`** | `797.0 KiB` | `714.2 KiB` |
| `spiral-200k` | `2` | `2` | `2` | `2` / `2` | `2` | `4.3 KiB` | **`4.3 KiB`** | `304.0 KiB` | `308.2 KiB` |

**Cumulative 6-block worker scratch (`diet_scratch`):**
- `master` (`0.5.11`): **`1,200,839 B` (`1,172.7 KiB`)** (`offsets = 16,384 B`, `inline16 = 62,256 B`, `lens = 3,891 B`).
- `MN16-Direct`: **`1,147,395 B` (`1,120.5 KiB`, `−52.2 KiB` / `−4.5%`)** (`offsets = 0 B`, `inline16 = 27,376 B`, `lens = 1,711 B`, **`inline16 + lens <= 28.4 KiB < 32 KiB` L1D on 6/6 blocks**).

### 2.2 Stage Attribution & End-to-End Interleaved A/A + A/B (`60` Alternated Pairs/Block at `1T`, `40` Barrier Passes at `2T`)

*(Note: in `MN16-Direct`, `inline16` materialization moves out of `walk` into `construct`, accounting for `+38 µs/block` in `construct` while accelerating `walk/emit` by **`9.5%`** across the 6 blocks and **`15.9%`** on `dense-300k`.)*

| Block / Suite | A/A (`Master2/Master1`) | `MN16-Direct` (`24K`) / `Master1` | `MN16-Direct` (`4K` RC) / `Master1` | `walk/emit` Ratio (`MN16-Direct` / `Master1`) |
|---|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `0.991` / `0.997` | **`0.985`** (`1283.5 / 1302.6 µs`) | **`0.983`** (`1288.4 / 1310.9 µs`) | **`0.916x`** (`511.1 / 558.3 µs`) |
| `block-2.L` (`1T p10`) | `0.990` / `1.003` | `1.002` (`1546.2 / 1543.0 µs`) | **`0.977`** (`1560.7 / 1597.4 µs`) | **`0.892x`** (`638.4 / 716.0 µs`) |
| `block-85.L` (`1T p10`) | `0.985` / `0.997` | **`0.963`** (`1562.3 / 1623.0 µs`) | **`0.973`** (`1566.1 / 1609.8 µs`) | **`0.916x`** (`892.9 / 975.3 µs`) |
| `block-143.L` (`1T p10`) | `0.989` / `0.982` | **`0.968`** (`2813.5 / 2907.1 µs`) | **`0.974`** (`2818.8 / 2892.9 µs`) | **`0.899x`** (`1318.6 / 1466.0 µs`) |
| `block-210.L` (`1T p10`) | `0.988` / `1.011` | **`0.943`** (`2407.7 / 2554.2 µs`) | **`0.962`** (`2458.3 / 2555.9 µs`) | **`0.909x`** (`1346.6 / 1480.9 µs`) |
| `block-207.L` (`1T p10`) | `0.997` / `1.004` | **`0.966`** (`1845.3 / 1911.1 µs`) | **`0.978`** (`1899.7 / 1943.0 µs`) | **`0.901x`** (`837.6 / 929.8 µs`) |
| **`1T` 6-Block Total** | **`0.9901` / `0.9981`** | **`0.9677x` (`−3.23%`)** | **`0.9733x` (`−2.67%`)** | **`0.9052x` (`−9.48%`)** |
| `dense-300k` (`r/n=0.216`) | `1.002` / `1.003` | **`0.964x`** (`1947.5 / 2019.5 µs`) | `1.000x` (`2075.1 / 2076.0 µs`) | **`0.841x` (`−15.9%`)** |
| `spiral-200k` | `0.976` / `0.982` | **`0.983x`** (`46.5 / 47.3 µs`) | **`0.980x`** (`46.0 / 47.0 µs`) | — |
| **`2T` Concurrent Pass (`p10`)** | **`0.9971` / `0.9911`** | **`0.9767x` (`−2.33%`)** | **`0.9782x` (`−2.18%`)** | — |
| **`2T` Concurrent Pass (`p50`)** | **`0.9896` / `0.9923`** | **`0.9800x` (`−2.00%`)** | **`0.9668x` (`−3.32%`)** | — |

---

## 3. Cycle 16 Verdicts (Keep & One-Line Kills)

- **SURVIVOR (Advance to Cycle 25 Synthesis): `MN16-Direct` (`24 KiB` Arena-Hosted 1-Probe Two-Tier Quotient + Zero-`offsets`)** — Beats `0.5.11` `master` natively in safe Rust at both `1T` (`0.9677x` vs A/A `0.9901`, `0.964x` on dense `r/n=0.216`, `walk/emit` **`0.905x`**) and `2T` (`0.9767x` p10 / `0.9800x` p50 vs A/A `0.9971`), cuts persistent scratch by `−52.2 KiB` (`1,172.7 -> 1,120.5 KiB`), and pulls `inline16 + lens` inside `32 KiB` L1D (`<= 28.4 KiB`) on `6 / 6` blocks.
- **Kill (`PairOnly`):** Killed because `(node_word, a)` rule-ID memoization without string quotienting misses cross-strip isomorphic peelings (`only 15% word reduction`, leaving `block-143` at `55.5 KiB > 32 KiB` L1D).
- **Kill (`MN16-Linear`):** Killed because open-addressed linear probing hits `70%` load-factor probe chains on dense blocks (`1.016x` on `dense-300k`), whereas 1-probe right-cancellative `MN16-Direct` captures `99.95%` of exact deduplication in guaranteed `O(1)` time (`0.964x` on `dense-300k`).
- **Kill (`MN16-OutTape`):** Killed because borrowing `out`'s unused tail for `e.append` regresses `spiral-200k` (`1.40x`) and `dense-300k` (`1.015–1.026x`) for only `33.8 KiB` additional `spelling` reduction.
