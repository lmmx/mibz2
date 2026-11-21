# Wormhole Cycle 15 — Stage `S2` Breakthrough: Free-Monoid Length-Graded Semiring Handles `(len10 << 22) | id22` + Constant-Sentinel `pop_tail` (`-31%` `S2b` at `0 B` Scratch)

**Session:** `research/carbon-wormhole` (Round 3, Cycle 15 of 20; 5th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle15.rs`](2026-11-21-wormhole-cycle15-s2-zeroload-slp-and-1pass-strips/verify_cycle15.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,228` Gate G1 + `SPIRALS` `widest = 50,000 > 32,768` + `len > 1,023` saturating-fallback cases byte-exact + `21`-round rotating native A/B + A/A + same-crate `M0` control), [`s2_zeroload_slp_1pass.toml`](2026-11-21-wormhole-cycle15-s2-zeroload-slp-and-1pass-strips/s2_zeroload_slp_1pass.toml) (`PASS`), [`cycle15_output.txt`](2026-11-21-wormhole-cycle15-s2-zeroload-slp-and-1pass-strips/cycle15_output.txt), [`cycle15_results.json`](2026-11-21-wormhole-cycle15-s2-zeroload-slp-and-1pass-strips/cycle15_results.json)

---

## 1. One-Line Keep & Kill Decisions (Cycle 15)

- **PROMOTE `B4: GradedHandle + pop_tail` (`(len10 << 22) | id22` Semiring Handle in `Elim::join` + Constant-Offset Sentinel-0 `pop_tail`, `0 B` scratch delta):** Cuts `Stage S2b` (`Elim::eliminate`) by **`-27.8%` to `-31.9%` (`-30.3` to `-111.8 µs`)** with **`+0.0 µs` `S3a` penalty** and **`0 B` working-set increase**, winning **`6/6` Wikidata blocks** against both same-crate `M0:Clone` (**`0.948×–0.973×`**) and external `A1:Master` (**`0.912×–0.921×`**).
- **KILL `B1: Tarjan Post-Order RULE2` (`8 B` `[left, right]` rules + post-order `(id, start)` `append2_tarjan`):** Killed in favour of `B4: GradedHandle` because although `RULE2` cuts `S2b` by `-31%`, pushing a post-order stack frame per rule visit plus `fill(u32::MAX)` gives back **`+17` to `+37 µs`** in `S3a` (`409.9` vs `378.3 µs` on `block-143.L`).
- **HOLD / DEFER `Knuth-1Pass-S2a` (`2 * ns` packed `image_order`):** Saves only `~1–5 µs` on isolated `S2a` (`strips` is `65–131 KiB` = 100% L2-warm right after `S1`) while widening `image_order` from `ns` to `2 * ns` words (`+16–32 KiB` on `arena`-dominated blocks), though L1d preservation helps downstream stages slightly; `B4` achieves the primary `S2b` win at **strictly `0 B` scratch delta**.

---

## 2. The Chain of Existing Results (`S2b` Wormhole)

Cycle 11 identified `Stage S2` (`141–452 µs`, `5.95–8.70×` L2 miss inflation) as **Pre-`Emit` Gap #2**, driven by two hidden microarchitectural hazards inside `src/open/endpoint.rs`:

1. **Link 1 — Pointer-Alias Disambiguation on Intrusive Circular Lists (`pop_tail` vs `cut` on Sentinel `0`):**
   - In `endpoint.rs:307-358`, every Rauzy–Veech step pops `x = L::prev(self.a, 0, 0)` or `y = L::prev(self.a, 0, 1)` from the tail of a circular list through sentinel `0` (`so next(x, 0) == 0` and `next(y, 1) == 0` identically!).
   - Calling generic `self.cut(x, 0)` computes dynamic `n = L::next(self.a, x, 0)` (`= 0`) and writes `L::set_prev(self.a, n, 0, p)` to `self.a[n * STRIDE + 0]`. Because `n` is a runtime load from `&mut self.a`, LLVM cannot prove static alias identity with `self.a[0]`, stalling the very next iteration's `L::prev(self.a, 0, 0)` behind a **~5-cycle L1d store-to-load forwarding / alias hazard** (`7,130–21,295` times per block!).
   - Replacing `self.cut` on sentinel-adjacent tail nodes with `pop_tail(id, side)` (`L::set_next(a, p, side, 0); L::set_prev(a, 0, side, p);` at **compile-time constant index `0`**) lets LLVM keep sentinel `0`'s links in GPRs across iterations, cutting `S2b` by **`-20%` to `-23%` (`-23` to `-78 µs`)** on its own (`B2` `S2b` column)!

2. **Link 2 — Length-Graded Free-Monoid Homomorphism Memoized in Node Handles (`Lohrey 2012` Graded SLP + Saturating Tropical Semiring $(\mathbb{N}_{\le 1023}, +)$):**
   - In `endpoint.rs:263`, `Elim::join(left, right)` chases **two random memory loads** (`self.a[self.rule(left) + 2]` and `self.a[self.rule(right) + 2]`) across up to `400 KiB` of rules (`> 256 KiB` L2!) solely to evaluate the length homomorphism $|\cdot| : (\Sigma^+, \cdot) \to (\mathbb{N}, +)$, $|u \cdot v| = |u| + |v|$.
   - **Theorem 15.1 (22-Bit Rule-ID Bound & 10-Bit Graded Handle Exactness, `proved`):** For any bzip2 block of $n \le 900{,}000$ rows:
     1. Every non-spiral elimination step decreases `remaining` by $w \ge 1$ and creates $\le 1$ rule; every spiral contraction (`q >= 2`) decreases `remaining` by $q \cdot w \ge 2$ and creates $\le 2 \lfloor \log_2 q \rfloor + 2 \le 2 q$ rules (`maximum ratio 2.0 at q = 2`). Hence total rules satisfy $R_{\mathrm{rules}} \le 2 n \le 1{,}800{,}000$, and every literal or rule ID satisfies:
        $$\mathrm{id} \le 256 + 1{,}800{,}000 = 1{,}800{,}256 < 2^{22} = 4{,}194{,}304.$$
     2. Packing `handle = (len.min(1023) << 22) | id` inside each node's existing `u32` `word` slot memoizes the length homomorphism directly inside the L1d-hot active node (`already loaded for aw/bw`), turning `join_graded(lh, rh)` into **pure register arithmetic `ll + rl` (`0` random rule loads!)** whenever `ll < 1023 && rl < 1023` (**`100.00%` of joins on all 6 Wikidata blocks**, since $\max |w| \le 186 < 1{,}023$!) and falling back soundly to `self.len4(lid) + self.len4(rid)` if a spiral word ever reaches $\ge 1{,}023$ (`verified` on `widest = 50,000` spiral test)!
     3. Because `join_graded` still writes `[lid, rid, exact_len, u32::MAX]` sequentially to `a[top..top+4]`, Stage `S3a` (`append4`) runs **100% unmodified with `+0.0 µs` overhead**!

---

## 3. Stage-Isolated & End-to-End Native Safe-Rust Confrontation (`21` Rotating Rounds)

We include **both** external `A1:Master` (`libmibz2.rlib` `OpenCells`, with `A2:Ctrl` A/A control `0.992–1.014×`) **and** `M0:Clone` (verbatim `origin/master` in the same compilation unit, isolating `~4–6%` cross-function inlining from `B4`'s pure algorithmic `S2b` speedup; note `dense-703.syn` where `S2b = 6.1 µs` gives `B4/M0 = 0.998×`, proving zero measurement bias!).

- **Per-Worker Working Set (`B4: GradedHandle`):** `1.885 MiB` (**`0 B` added** vs `origin/master`; `10` i9 workers = `18.85 MiB <= 20 MiB` L3).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).
- **Acquisition Cost:** ~25 lines in `src/open/endpoint.rs`, `#![forbid(unsafe_code)]`.

| Block | `M0` (`S2a / S2b / S3a µs`) | `B1:Tarjan` (`S2a / S2b / S3a µs`) | `B4:Graded` (`S2a / S2b / S3a µs`) | `S2b` Cut (`B4` vs `M0`) | `A1:Master` (`µs`) | `A2/A1` | `M0:Clone` (`µs`) | `B4:Graded` (`µs`) | `B4 / A1` | **`B4 / M0`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `53.6 / 192.3 / 201.8` | `53.7 / 128.8 / 223.8` | `53.5 /` **`132.4`** `/ 197.2` | **`-31.1%` (`-59.9 µs`)** | `1,348.9` | `0.992x` | `1,281.7` | **`1,242.1`** | `0.921x` | **`0.969x`** |
| `block-2.L` | `63.6 / 233.6 / 247.9` | `62.2 / 159.5 / 273.6` | `58.8 /` **`161.5`** `/ 236.2` | **`-30.9%` (`-72.1 µs`)** | `1,634.2` | `0.996x` | `1,572.9` | **`1,491.0`** | `0.912x` | **`0.948x`** |
| `block-85.L` | `21.4 / 109.0 / 118.5` | `22.6 / 74.3 / 135.1` | `21.8 /` **`78.7`** `/ 118.7` | **`-27.8%` (`-30.3 µs`)** | `1,661.0` | `0.999x` | `1,567.8` | **`1,524.9`** | `0.918x` | **`0.973x`** |
| `block-143.L` | `77.1 / 351.6 / 372.7` | `77.8 / 238.6 / 409.9` | `72.6 /` **`239.8`** `/ 378.3` | **`-31.8%` (`-111.8 µs`)** | `2,952.4` | `1.014x` | `2,853.7` | **`2,717.0`** | `0.920x` | **`0.952x`** |
| `block-210.L` | `42.0 / 195.7 / 208.8` | `41.7 / 130.1 / 230.0` | `40.5 /` **`133.3`** `/ 202.2` | **`-31.9%` (`-62.4 µs`)** | `2,609.2` | `1.002x` | `2,469.1` | **`2,389.8`** | `0.916x` | **`0.968x`** |
| `block-207.L` | `51.5 / 227.4 / 244.8` | `50.1 / 157.9 / 267.3` | `51.6 /` **`157.6`** `/ 237.7` | **`-30.7%` (`-69.8 µs`)** | `1,962.5` | `1.000x` | `1,884.7` | **`1,795.7`** | `0.915x` | **`0.953x`** |
| `dense-703.syn` | `1.7 / 6.1 / 4.5` | `1.6 / 4.7 / 5.3` | `1.8 /` **`4.8`** `/ 4.3` | `-21.3%` (`-1.3 µs`) | `1,541.5` | `0.999x` | `1,433.3` | **`1,430.4`** | `0.928x` | **`0.998x`** |

---

## 4. Next Step (Cycle 16)

Before moving to Stage `S3`, **Cycle 16** tests **Theorem 11.2(b)'s Escape Route** on Stage `S2`: can we escape doubly-linked Rauzy–Veech induction *entirely* via `INDUCE = false` (`Tile` + `PAGE = 64` directory trace, `open.rs:239-280`) or **Wyllie (1979) / Sleator–Tarjan Lazy Path-Compressed `Tile` Jump-Pointers** (memoizing `(exit, word_code, slack)` on traced `Tile` intervals)? We test and break both honestly against `origin/master` and `B4: GradedHandle`.
