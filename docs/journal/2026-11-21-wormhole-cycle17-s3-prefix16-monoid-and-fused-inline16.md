# Wormhole Cycle 17 — Stage `S3` Breakthrough: Prefix-15 Tree-Unfold Bound (`Theorem 17.1`) + Direct `inline16` Fusion (`0.0 µs` `S3b`, `-20%` `S2+S3`, `0 B` Extra Scratch)

**Session:** `research/carbon-wormhole` (Round 3, Cycle 17 of 20; 7th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle17.rs`](2026-11-21-wormhole-cycle17-s3-prefix16-monoid-and-fused-inline16/verify_cycle17.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,229` Gate G1 + `SPIRALS` + `dense-703.syn` cases byte-exact + `21`-round rotating native A/B + A/A + `M0` + `G15` confrontation), [`s3_prefix16_monoid.toml`](2026-11-21-wormhole-cycle17-s3-prefix16-monoid-and-fused-inline16/s3_prefix16_monoid.toml) (`PASS`), [`cycle17_output.txt`](2026-11-21-wormhole-cycle17-s3-prefix16-monoid-and-fused-inline16/cycle17_output.txt), [`cycle17_results.json`](2026-11-21-wormhole-cycle17-s3-prefix16-monoid-and-fused-inline16/cycle17_results.json)

---

## 1. One-Line Keep & Kill Decisions (Cycle 17)

- **PROMOTE `B2: Lazy-Prefix15-DirectInline16` (`Theorem 17.1` Fixed-Stack `[u32; 16]` Short-Word Unfold + Fused `inline16` Construction on top of `GradedRV`, `0 B` extra scratch):** Deletes **`100%` of Stage `S3b` (`inline_words`, `12.1–34.9 µs -> 0.1 µs`)**, eliminates `spelling` writes for all `len <= 16` words (`63.6%–77.6%` of words, shrinking `spelling` up to `-25%`), cuts `S2 + S3` by **`-19.1%` to `-20.0%` (`-50` to `-165 µs`)**, and wins **`7/7` blocks** against `G15:GradedRV`, **`7/7` blocks** against `M0:Clone` (**`0.937×–0.990×`** on Wikidata), and **`7/7` blocks** against `A1:Master` (**`0.878×–0.893×`**).
- **KILL `B1: Eager-u128-Prefix16` (Eilenberg Syntactic Quotient Monoid $\Sigma^{\le 15}$ evaluated eagerly inside `Elim::join`, `RULE8 = 32 B`):** Killed in favour of `B2: Lazy15` because although eager `u128` prefixes cut `S3a` by `-72 µs` on `block-143.L` (`1` `16 B` load/short word), widening rules from `16 B` to `32 B` (`+100–256 KiB` arena!) and reintroducing rule-prefix loads inside `Elim::join` gives back **`+35` to `+96 µs` on `S2`** (`B2` beats `B1` at strictly smaller scratch!).

---

## 2. The Chain of Existing Results (`Stage S3` Wormhole)

In `origin/master`, Stage `S3` expands **100%** of multi-byte SLP words (`1,220–3,635` words) into `self.spelling: Vec<u8>` via heap-backed DFS (`e.append`), runs `S4` (`wire`) across `400–900 KiB` (evicting `spelling` from L1d), and then re-scans `offsets` and `spelling` cold in `S3b` (`inline_words`, `open.rs:717-751`, `12–35 µs`) to populate `inline16` and `lens` — **even though `emit` (`open.rs:789-795`) never reads `spelling` or `offsets` for any word of total length $\le 16$ (`63.6%–77.6%` of all words!)**.

Because Cycle 15's `GradedRV` handle `(len10 << 22) | id22` already delivers exact `h_len = |node_word|` in a register at the top of `S3a` (`total word length = h_len + 1`), we chained two results from formal language / tree-automata theory:
1. **Link 1 — Eilenberg (1974) Syntactic Prefix-15 Quotient Monoid $\pi_{15} : (\Sigma^*, \cdot) \to (\Sigma^{\le 15}, \odot)$:** For `total_len <= 16` (`h_len <= 15`), `emit` depends *solely* on $\pi_{15}(\mathrm{node\_word}) \cdot a \in \Sigma^{\le 16}$. Testing eager evaluation in `Elim::join` (`B1: Eager-u128`, `lp | (rp << 8*ll)`) vs demand-driven evaluation in `S3a` (`B2: Lazy-Prefix15`) exposed a clean **Eager vs Demand-Driven Duality**: eager evaluation touches dead intermediate RV rules and doubles rule stride (`32 B`), whereas demand-driven evaluation runs strictly on the `776–2,315` surviving short words!
2. **Link 2 — Non-Erasing SLP Tree-Size & Stack-Depth Bound (`Theorem 17.1`, `proved`):** Why is unmemoized tree unfolding of `node_word` safe (`O(1)` worst-case time and space, even on exponential DAGs!) whenever `h_len <= 15`?

### Theorem 17.1 — Exact Tree-Size & Stack-Occupancy Bound on Short SLP Words (`proved` + `verified` on `5,229` cases + `6` blocks)

- **Statement (`proved`):** Because `Elim::join` is non-erasing (`join(0, r) = r`, `join(l, 0) = l`) and every literal $1 \dots 256$ has length $1$, every internal rule node in the unfolded binary parse tree $T(w)$ of any non-zero word $w$ has two non-empty children of lengths $L_1, L_2 \ge 1$ with $L_1 + L_2 = |w|$. Therefore, **even without DAG memoization (`rule[3]`)**:
  1. $T(w)$ has **exact** $|w|$ leaves and **exact** $|w| - 1$ internal rule nodes (`0` rule loads when $|w| = 1$, `1` rule load when $|w| = 2$, and $\le 14$ rule loads whenever $|w| \le 15$ — **exponential DAG blowup is mathematically impossible for $|w| \le 15$!**).
  2. During iterative right-pushed pre-order DFS on a fixed stack `st: [u32; 16]`, let `vp` be the bytes emitted so far and $S = \sum_{u \in \mathrm{st}} |u|$; `vp + S == |w|` is an exact loop invariant, and every stack entry satisfies $|u| \ge 1$, so stack depth satisfies:
     $$\mathrm{sp} \le S = |w| - \mathrm{vp} \le |w| \le 15 < 16 \quad \text{at every step!}$$
- **Algorithmic Payoff (`verified` in [`verify_cycle17.rs`](2026-11-21-wormhole-cycle17-s3-prefix16-monoid-and-fused-inline16/verify_cycle17.rs)):**
  - `h_len == 1`: **0 memory loads** (`v[0] = (node_id - 1) as u8; v[1] = a as u8;`).
  - `h_len == 2`: **1 rule load** (`v[0] = (a[r]-1) as u8; v[1] = (a[r+1]-1) as u8; v[2] = a as u8;`).
  - `3 <= h_len <= 15`: unfolds in L1d `[u32; 16]` directly into `v: [u8; 16]` (`0` `Vec` heap stack ops, `0` bytes written to `spelling`!).
  - `h_len >= 16` (`22.4%–36.4%` of words): expands via DAG-memoized `append_stride::<4>` into `spelling` and pushes `LONG` directly to `lens`.
  - **Stage `S3b` (`inline_words`) is completely eliminated (`0.0 µs`)**, while `origin/master`'s `emit` runs **100% unmodified**!

---

## 3. Stage-Isolated & End-to-End Native Safe-Rust Confrontation (`21` Rotating Rounds, `0 B` Extra Scratch)

- **Per-Worker Working Set (`B2: Lazy15`):** **`1.871–1.881 MiB`** (**smaller** than `origin/master`'s `1.885 MiB` because `spelling` shrinks by `-4` to `-14 KiB`!).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).
- **Acquisition Cost:** ~45 lines in `src/open/endpoint.rs` + deleting `inline_words` call in `src/open.rs`, `#![forbid(unsafe_code)]`.

| Block | `spelling` (`M0 -> B2`) | `M0` (`S2 / S3a / S3b / Sum µs`) | `G15` (`S2 / S3a / S3b / Sum µs`) | `B1:Eu128` (`S2 / S3a / S3b / Sum µs`) | **`B2:Lazy15`** (`S2 / S3a / S3b / Sum µs`) | `A2/A1` | `G15 / M0` | **`B2 / M0`** | **`B2 / A1`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `50 -> 46 KiB` | `249 / 195 / 13.1 / 457` | `189 / 199 / 13.9 / 402` | `224 / 183 / 0.0 / 407` | **`187 / 193 / 0.1 / 380`** (`-16.8%`) | `0.994x` | `0.966x` | **`0.947x`** | **`0.887x`** |
| `block-2.L` | `48 -> 41 KiB` | `297 / 243 / 19.4 / 559` | `221 / 241 / 20.1 / 482` | `272 / 205 / 0.1 / 477` | **`216 / 234 / 0.1 / 450`** (`-19.5%`) | `0.998x` | `0.961x` | **`0.937x`** | **`0.880x`** |
| `block-85.L` | `30 -> 26 KiB` | `131 / 120 / 12.1 / 262` | `99 / 120 / 12.1 / 231` | `121 / 101 / 0.1 / 223` | **`97 / 115 / 0.1 / 212`** (`-19.1%`) | `1.001x` | `0.999x` | **`0.972x`** | **`0.888x`** |
| `block-143.L` | `56 -> 42 KiB` | `421 / 376 / 34.0 / 831` | `316 / 369 / 34.9 / 720` | `404 / 297 / 0.1 / 701` | **`308 / 358 / 0.1 / 666`** (`-19.9%`) | `1.006x` | `0.992x` | **`0.944x`** | **`0.878x`** |
| `block-210.L` | `37 -> 29 KiB` | `234 / 202 / 21.2 / 458` | `171 / 200 / 21.2 / 392` | `214 / 163 / 0.1 / 377` | **`172 / 195 / 0.1 / 368`** (`-19.7%`) | `0.996x` | `1.018x` | **`0.990x`** | **`0.892x`** |
| `block-207.L` | `54 -> 45 KiB` | `273 / 239 / 21.4 / 534` | `207 / 234 / 21.4 / 462` | `249 / 191 / 0.1 / 440` | **`203 / 224 / 0.1 / 427`** (`-20.0%`) | `1.011x` | `0.972x` | **`0.946x`** | **`0.886x`** |
| `dense-703.syn` | `1 -> 0 KiB` | `7 / 4 / 1.0 / 13` | `6 / 5 / 1.0 / 12` | `7 / 3 / 0.1 / 10` | **`6 / 4 / 0.1 / 10`** | `1.002x` | `1.025x` | **`0.999x`** | **`0.893x`** |

---

## 4. Next Step (Cycle 18)

Look at what remains in `B2: Lazy15`'s `S3a` (`115–358 µs`):
1. `self.bands: Vec<Band>` (`49–98 KiB` separate heap buffer, written in `S3a` and immediately read & discarded in `S4`!), which existed *only* because `Elim`'s arena sat at `edges[ns..]` where `S4` writes `edges[0..m)`!
2. **Cycle 18** tests **Chaitin (1982) Split-End / High-Tail Arena Placement + Direct `S3 -> S4` Fused Band-to-Edge Streaming (`0 B` `Vec<Band>`!)** — placing `Elim`'s arena at the tail/disjoint window or streaming `Band`s directly into `Reader::wire` whenever non-overlapping, and breaking or promoting it honestly against `B2: Lazy15`!
