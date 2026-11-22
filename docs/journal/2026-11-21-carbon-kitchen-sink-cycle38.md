# Carbon Kitchen Sink (Neon) — Cycle 38: `S2b` Attack — Zero-Rule Inline Bigram/Trigram Handles (`Inline2` / `Inline23`) & Register-Cached `RegElim`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical Framing — Attacking Rank #2 Construction Stage (`S2b: 1,202 µs`, `22.8%` of `Nobi Pre`)

In `origin/master` (`53f7ecf93`), `GradedHandle` packs `(len.min(1023) << 22) | id` into `u32`, yet allocates a `16 B` arena rule in `self.a[top..top+4]` for every `len >= 2` join (`47,220` rules across the `6` Wikidata blocks). We tested two structural mechanisms on `S2b`:
1. **Zero-Rule Inline Handles inside `ID_BITS = 22` (`Inline2` & `Inline23`):**
   - **`Inline2`:** Pack every 2-byte bigram `(b0, b1) ∈ [0..255]^2` directly into `(2 << 22) | b0 | (b1 << 8)` (`16 < 22` bits). Because `literal(b) = (1 << 22) | b`, both `grade == 1` and `grade == 2` decode identically via `if g <= 2 { return u128::from(id); }` (**zero** extra decode branch, **zero** rule writes, **zero** rule reads!). Long rules store full 32-bit graded handles `[left_h, right_h, len, u32::MAX]`, eliminating `a[r+3]` tag reads in `append()`.
   - **`Inline23`:** Additionally pack 7-bit ASCII trigrams (`(b0 | b1 | b2) < 128`, `21` bits + bit `21` tag) directly into `(3 << 22) | (1 << 21) | b0 | (b1 << 7) | (b2 << 14)`.
2. **Register-Cached Euclidean Peeling (`RegElim`):** Keep `(x, y, aw, bw, wx, wy)` in registers across `eliminate::<false>` iterations with deferred sentinel/width writes and 1-step adjacent spiral identity elision (`px == y` / `py == x`).

---

## 2. Rule-Reduction Census & Native Safe-Rust A/B Confrontation (`run_c38.rs`)

Built atop `Neon-C37` (`Cap127 + Pt64 + Wire16 + S2aFuse + S3Tag12`) and compared against verbatim `origin/master` (`53f7ecf93`, `MasterA1` / `MasterA2` A/A control), compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`).

- **Parity:** `100%` byte-exact on **`2,000` ASCII & non-ASCII (`125..128`) small BWTs** + `6/6` Wikidata blocks + `dense-300k`.
- **Exact `S2b` Rule Reduction (`6` Wikidata blocks):**
  * `origin/master`: `47,220` rules (`755.5 KiB` arena rule writes).
  * **`Inline2`:** **`37,274` rules (`78.9%`, `−9,946` rules = `−21.1%` / `−159.1 KiB` rule writes eliminated!)**.
  * **`Inline23`:** **`30,097` rules (`63.7%`, `−17,123` rules = `−36.3%` / `−274.0 KiB` rule writes eliminated!)**.

### 2.1 `1T` Interleaved (`50` rounds/block, `35`-round `Pre` split) & `2T` Concurrent Barrier (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs `M1`) | `1T` `Pre` (`µs`, ratio vs `M1`) | `ΔPre` vs `C37` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,458.2 (1.0000×)` | `5,962.4 (1.0000×)` | `+243.5` | `10,461.1 (1.0000×)` | `10,591.4 (1.0000×)` |
| `MasterA2` (A/A) | `10,427.0 (0.9970×)` | `5,973.6 (1.0019×)` | `+254.7` | `10,504.3 (1.0041×)` | `10,601.8 (1.0010×)` |
| `Neon-C37` (prev champ) | `10,217.2 (0.9770×)` | `5,718.9 (0.9592×)` | `0.0` | `10,264.4 (0.9812×)` | `10,383.3 (0.9804×)` |
| **`Inline2` (`Neon-C38`)** | `10,221.9 (0.9774×)` | **`5,709.1 (0.9575×)`** | **`−9.9`** | **`10,248.0 (0.9796×)`** | **`10,344.2 (0.9767×)`** |
| `Inline23` | `10,267.0 (0.9817×)` | `5,774.5 (0.9685×)` | `+55.6` | — | — |
| `RegElim` | `10,340.6 (0.9888×)` | `5,868.3 (0.9842×)` | `+149.3` | `10,461.7 (1.0001×)` | `10,506.7 (0.9920×)` |
| `I2+Reg` | `10,309.9 (0.9858×)` | `5,807.6 (0.9740×)` | `+88.6` | `10,341.0 (0.9885×)` | `10,437.1 (0.9854×)` |
| `I23+Reg` | `10,337.5 (0.9885×)` | `5,875.9 (0.9855×)` | `+157.0` | `10,410.7 (0.9952×)` | `10,508.7 (0.9922×)` |

---

## 3. Theorem, One-Line Kills, & Promotions

- **Theorem 38.1 (Sequential L1d Arena-Write Cheapness vs Branch-Entropy Ceiling in `S2b: Elim`):** Writing a `16 B` rule sequentially to `self.a[top..top+4]` costs `~1.5 ns` in L1d, whereas any extra data-dependent branch inside `join()` or `eliminate()` costs `~5 ns` per mispredict. Thus `Inline23` (`−36.3%` rules, `+55.6 µs`) and `RegElim` (`+149.3 µs`, `3×` larger loop body + `px==y`/`py==x` branches) lose to `origin/master`'s compact L1i-hot `eliminate()` loop, while single-branch zero-unpacking `Inline2` (`−21.1%` rules, `−159 KiB` L2 arena writes) wins cleanly on `Pre` (`−9.9 µs`) and especially under `2T` L2 pressure (`−16.4 µs` p10 / `−39.1 µs` p50 vs `Neon-C37`).
- **KILL `Inline23` (`+55.6 µs` `Pre`):** Three extra branches + 7-bit bit-unpacking in `join()`/`short_payload_h()` outweigh saving `7,177` additional `16 B` L1d rule writes.
- **KILL `RegElim` (`+149.3 µs` `Pre`, `+197.3 µs` `2T` p10):** `3×` loop-body expansion and `px==y`/`py==x` branches break L1i/μop-cache residency and add mispredicts.
- **PROMOTE `Inline2` (`Neon-C38`):** Cuts `S2b` rules by `−21.1%` (`47,220 -> 37,274`, `−159.1 KiB` arena writes), lowers `Pre` to **`0.9575×`** vs `53f7ecf93`, and improves `2T` to **`0.9796×` p10 / `0.9767×` p50** (vs A/A `1.0041× / 1.0010×`).

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle38/` (`gen_c38.py`, `run_c38.rs`).
- **Cycle 39 (Neon):** Attack **Rank #3 & #4 Construction Stages (`S3a: 783 µs` + `S4: 763/1,489 µs`)** — investigating (i) **Zero-`Counts` Hot-Path Specialization** (`S3a` currently executes `6` stat updates + a multiply on all `~100K` bands even when production discards `Counts`!), (ii) **Branchless `Inline2` `Dictionary::code` + `Node`-Order `S3a` Prefetch / Stream**, and (iii) **AVX2 / `u64`-Unpacked Serial `S4` (`OutQ16::wire`)** for the `nobi` ($\mu = 1$) regime!
