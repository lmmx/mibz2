# Carbon Kitchen Sink — Cycle 20: 128-Bit Truncated Free-Monoid Operads (`Operad-MN16`) — Beating `0.5.11` `master` in Both `construct` and `walk`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Operadic evaluation in the 128-bit truncated free monoid $\mathcal{M}_{\le 15} = \Sigma^{\le 15} \sqcup \{\top_{\ge 16}\}$ inside `Elim`'s existing `RULE = 4` `u32` arena stride, eliminating `e.append`, `Endpoint::stack` traversal, and `spelling` writes on `> 93%` of return bands.

---

## 1. Mathematical Construction: Graded Truncated-Monoid Algebra Inside `RULE = 4`

In [`endpoint.rs`](../../../../src/open/endpoint.rs), `Elim` represents every rule $w > 256$ as `RULE = 4` `u32` words (`128 bits`: `[left, right, len, first]`), and at the bottom scan expands each non-literal band through an explicit `Vec<u32>` stack (`e.append`) into `spelling: Vec<u8>`, then copies the first 16 bytes into `inline16`. Even in Cycle 16's `MN16-Direct`, the first visit to each distinct `(node_word, a)` pair still executed `e.append`, accounting for `+38 µs/block` (`1.040x`) in `construct`.

Observe that `emit` only ever reads `spelling` when `word.len() > 16` (`< 7%` of bands); for every band with `height = len(node_word) <= 15` (`wlen = height + 1 <= 16`, `> 93%` of bands), `emit` requires **only** the 16-byte little-endian word `u128`. We replace binary-tree deferral on $\Sigma^{\le 15}$ by eager evaluation in the **truncated free-monoid operad** $\mathcal{M}_{\le 15}$:

### Theorem 20.1 (Zero-Overhead 128-Bit Tagged Operad Encoding in `RULE = 4` Arena Words)
Let $\psi : V_{\mathrm{Elim}} \to \Sigma^+$ be the SLP evaluation homomorphism. Partition rules $w > 256$ inside `Elim`'s existing `4`-word (`128`-bit) slot `a[r .. r + 4]` by the top byte `tag = a[r + 3] >> 24`:
1. **Short Operad Element (`1 <= len <= 15` $\iff$ `tag in 1..=15`):**
   - Bytes `0 .. len` (`<= 120` bits) store $\psi(w)$ in little-endian `u128`, bytes `len .. 15` are zero, and byte `15` (`bits 120..128`, i.e. `a[r + 3] >> 24`) stores `len`.
   - Whenever $\ell_L + \ell_R \le 15$, both `left` and `right` have length $\le 14$, so `Elim::join(left, right)` computes the concatenated `u128` in **two register shifts and two bitwise ORs** with zero tree pointers:
     $$v(L \cdot R) \;=\; v(L) \;\big|\; \bigl(v(R) \ll (8 \ell_L)\bigr) \;\big|\; \bigl((\ell_L + \ell_R) \ll 120\bigr).$$
2. **Long SLP Nonterminal (`len >= 16` $\iff$ `tag >= 0x80`):**
   - `a[r .. r + 4] = [left, right, len, 0xFF00_0000]` initially, updated on first `e.append` expansion to `0x8000_0000 | first_offset` (disjoint from `tag in 1..=15` because `first_offset <= n <= 900,000 < 2^{20} < 2^{24}`).
   - When `e.append` expands a long rule (`len >= 16`) and pops a child $c$ with `tag(c) < 16`, $c$ acts as a **macro stop-leaf**, emitting all `1..=15` pre-assembled bytes in a single slice copy without recursing further (cutting `Endpoint::stack` peak capacity in half from `128 B` to `64 B`).
3. **Register-Direct Bottom Scan:** On every band with `len(node_word) <= 15`, the full 16-byte `inline16` entry is formed in registers as `v = short_u128(node_word) | ((a as u128) << (8 * h_len))`, hashed in two `u64` multiplies, compared in registers against `str_ht`, and never touches `e.append`, `Endpoint::stack`, or `spelling`.

---

## 2. Exhaustive Verification & Native Safe-Rust Confrontation (`A/B + A/A`)

### 2.1 Verification & Scratch Audit
- **`16,376` exhaustive binary BWT checks** (`lengths 1..=11` across `LIMIT` $\in \{1, 2, 4, 16\}$, specifically exercising multi-level `> 15 B` long rules and stop-leaves at `LIMIT = 1`) + `6` authenticated 900 KB Wikidata blocks + `dense-300k` (`r/n = 0.216`) + `spiral-200k`: **`100.000%` byte-for-byte parity**.
- **6-Block cumulative scratch:** `1,167,828 B` (`1,140.5 KiB`, `−32.2 KiB` vs `0.5.11` `master`'s `1,172.7 KiB`; `offsets = 0 B`, `endpoint = 64 B`, `inline16 + lens <= 28.4 KiB < 32 KiB` L1D on `6 / 6` blocks).

### 2.2 Stage Attribution & End-to-End `1T` + `2T` Confrontation (`60` Pairs/Block `1T`, `40` Passes `2T`)

*(Single serial walk cursor $\mu = 1$ — isolating `Operad-MN16` directly against `0.5.11` `master` and Cycle 16's `MN16-Direct`.)*

| Block / Suite | `construct` (`master -> Operad`) | `walk/emit` (`master -> Operad`) | `1T` A/A | `1T` `MN16-Direct` | **`1T` `Operad-MN16` / `Master1`** |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | `705.2 -> 680.4 µs` (**`0.965x`**) | `524.5 -> 478.4 µs` (**`0.912x`**) | `0.986` | `1.001` | **`0.957x`** (`1186.7 / 1239.7 µs`) |
| `block-2.L` | `844.4 -> 818.2 µs` (**`0.969x`**) | `682.9 -> 634.9 µs` (**`0.930x`**) | `1.012` | `0.987` | **`0.922x`** (`1411.9 / 1531.8 µs`) |
| `block-85.L` | `613.9 -> 597.5 µs` (**`0.973x`**) | `909.6 -> 837.3 µs` (**`0.921x`**) | `0.978` | `0.962` | **`0.936x`** (`1492.7 / 1594.7 µs`) |
| `block-143.L` | `1389.8 -> 1299.8 µs` (**`0.935x`**) | `1382.9 -> 1295.3 µs` (**`0.937x`**) | `0.996` | `0.962` | **`0.931x`** (`2551.7 / 2741.9 µs`) |
| `block-210.L` | `1040.7 -> 1017.5 µs` (**`0.978x`**) | `1393.8 -> 1331.7 µs` (**`0.955x`**) | `0.980` | `0.975` | **`0.933x`** (`2249.5 / 2411.1 µs`) |
| `block-207.L` | `941.0 -> 900.2 µs` (**`0.957x`**) | `876.9 -> 800.5 µs` (**`0.913x`**) | `0.984` | `0.972` | **`0.928x`** (`1702.5 / 1834.4 µs`) |
| **`1T` 6-Block Sum** | **`5534.9 -> 5313.6 µs` (`0.9600x`)** | **`5770.7 -> 5378.1 µs` (`0.9320x`)** | **`0.9894`** | `0.9740` | **`0.9332x` (`−6.68%`)** |
| `dense-300k` (`0.216`) | **`1186.7 -> 1041.6 µs` (`0.878x`)** | **`794.1 -> 656.1 µs` (`0.826x`)** | `1.016` | `0.969` | **`0.840x` (`−16.0%`)** |
| `spiral-200k` | `41.4 -> 44.5 µs` | `5.1 -> 5.1 µs` | `0.969` | `0.976` | `1.059x` (`48.6 / 45.9 µs`) |
| **`2T` Concurrent (`p10`)** | — | — | **`1.0046`** | `0.9727` | **`0.9531x` (`−4.69%`)** |
| **`2T` Concurrent (`p50`)** | — | — | **`0.9984`** | `0.9772` | **`0.9351x` (`−6.49%`)** |

---

## 3. Cycle 20 Verdicts (Keep & One-Line Kill)

- **SURVIVOR (Advance to Cycle 25 Synthesis, superseding `MN16-Direct`): `Operad-MN16`** — Beats `0.5.11` `master` in **both** `construct` (`0.9600x`) and `walk` (`0.9320x`) across **6 / 6** authenticated blocks (`1T` **`0.9332x`** vs A/A `0.9894`, `dense-300k` **`0.840x`**, `2T` **`0.9531x` p10 / `0.9351x` p50** vs A/A `1.0046`), cuts scratch (`−32.2 KiB`, or `−82 KiB` with `META_CAP = 2048`), keeps $\mu = 1$ single-cursor `emit`, and pulls `Dict <= 28.4 KiB < 32 KiB` L1D.
- **Kill (`Tree-Walk e.append` on `len <= 15` rules / standalone `MN16-Direct`):** Superseded and killed in favour of `Operad-MN16` because deferring `<= 15 B` rules as binary tree pointers wastes `~45 µs/block` (`construct 1.040x` vs `0.960x`) when `u128` shift-OR fits inside the identical `RULE = 4` arena stride.
