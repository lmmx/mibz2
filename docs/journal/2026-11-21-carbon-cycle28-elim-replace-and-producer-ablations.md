# Carbon Cycle 28: Producer & Low-$m$ Ablations — `Elim::replace` Direct Splice Win (`0.9711×` `W = 1` / `0.9684×` `W = 4` Corpus)

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree) & Cycle 27 `LeanRegBurst6_256` (`022d8723`)  
**Artifact:** [c28_endpoint.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle28/c28_endpoint.rs) (paired with [c27_lean_regburst6.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle27/c27_lean_regburst6.rs))

---

## 1. Attacking All `233` Blocks: Three Single-Factor Ablations

With `LeanRegBurst6_256` accelerating the $m \ge 140{,}000$ walk, Cycle 28 ran three isolated single-factor native A/B + A/A ablations targeting the remaining `~185` low/mid-$m$ blocks ($m < 140{,}000$) and the `46%` of total runtime spent in Producer Phases 1–3 across **all** `233` Open blocks:

1. **`C28-A` (`SWAR8+32 OutQ` + `1`-elem/`u64` `Reader::wire` + `wide` hoist):**  
   Split `Queues::push` into an `8 B` carry-free `u64` SWAR store for `w <= 4` vs `32 B` for `5 <= w <= 16`, and added a `span == 1` + `4`-wide `u64` unpack path in `Reader::wire`. **KILLED** — adding the data-dependent `w <= 4` branch in `push` and breaking LLVM's 8-wide `vpmovzxwd ymm` auto-vectorization in `Reader::wire` regressed `T1 wiki-431` from `0.9837×` back to `1.0029×`.
2. **`C28-B` (`Tight Serial Emit`):**  
   Hoisted `extra.is_empty()` outside `emit_serial` and coupled `inline16`/`lens`/`out` slice bounds to prove bounds-check safety to LLVM. **KILLED (neutral)** — tied Cycle 27 to four decimal places (`0.9837×` vs `0.9837×` `T1 wiki-431`) because x86-64 predicts never-taken bounds checks at zero cycle cost.
3. **`C28-C` (`Elim::replace` on `aw == bw` + `#[inline(always)] Elim::node` in `endpoint.rs`):**  
   In Rauzy strip elimination (`Elim::eliminate`), equal-width collisions (`aw == bw`, `x != y`) previously executed `self.cut(x, 0); self.insert(y, x, 0); self.set_word(x, word); self.cut(y, 0); self.cut(y, 1);` — **8 packed-`u16` read-modify-writes** on side `0` that write `a[y * 4]` inside `insert(y, x, 0)` and immediately re-read/re-mask the *same* `u32` word inside `cut(y, 0)` (triggering store-to-load forwarding stalls), while `Elim::node` lacked `#[inline(always)]` across its `~10,000` calls/block in Phase 2a. Replacing `insert(y, x, 0) + cut(y, 0)` with a direct 4-RMW in-place `self.replace(y, x, 0)` and inlining `Elim::node` improved **every single block** (`< 1.000×` on all 6 `six` blocks, including `block-0` `0.9951×` and `block-2` `0.9937×`)!

Byte-exact parity (`warm` + `fresh` + `Plain`) passed on all `431` `wikidata.bz2` blocks (`W = 10` max heap: `1,275.2 KiB` vs `1,268.7 KiB`, `+6.5 KiB`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control | Cycle 27 `LeanRegBurst6_256` | `C28-A` (`SWAR OutQ`) | `C28-B` (`TightSerial`) | **`C28-C` (`Elim::replace` + `LeanRegBurst6_256`)** |
|---|---:|---:|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks) | `W = 1` | `0.9959×` | `0.9670×` | `0.9801×` | `0.9722×` | **`0.9671×`** (`11.16 / 10.80 ms`, all `6/6 < 1.0×`) |
| `dense-sum` (`5` dense blocks) | `W = 1` | `0.9929×` | `0.8957×` | `0.9076×` | `0.8962×` | **`0.8831×`** (`12.50 / 11.04 ms`, `blk-270` **`0.8116×`**) |
| `wiki-open233` (`233` Open blocks) | `W = 1` | `0.9996×` | `0.9837×` | `1.0029×` | `0.9837×` | **`0.9711×`** (`340.4 / 330.5 ms`) |
| `wiki-all431` (`431` blocks) | `W = 1` | `0.9996×` | `0.9837×` | `1.0029×` | `0.9837×` | **`0.9711×`** (`340.6 / 330.8 ms`) |
| `six-pass` (barrier-synced) | `W = 4` | `0.9823×` | `0.9743×` | `0.9618×` | `0.9723×` | **`0.9140×`** (`23.43 / 21.41 ms`) |
| `dense-pass` (barrier-synced) | `W = 4` | `0.9900×` | `0.8153×` | `0.8397×` | `0.8376×` | **`0.8381×`** (`25.88 / 21.69 ms`) |
| `wiki-open233` (barrier-synced) | `W = 4` | `0.9829×` | `0.9816×` | `0.9891×` | `0.9838×` | **`0.9667×`** (`737.2 / 712.7 ms`) |
| `wiki-all431` (barrier-synced) | `W = 4` | `0.9802×` | `0.9816×` | `0.9870×` | `0.9838×` | **`0.9684×`** (`737.6 / 714.3 ms`) |
| `E2E BzReader` (`wikidata.bz2`) | `W = 1` | `0.9857×` | `0.9796×` | `0.9847×` | `0.9891×` | **`0.9730×`** (`430.1 / 418.5 ms`) |
| `E2E BzReader` (`wikidata.bz2`) | `W = 4` | `0.9927×` | `0.9798×` | `0.9747×` | `0.9756×` | **`0.9649×`** (`293.1 / 282.8 ms`) |

---

## 3. `chaingrind` Diagnostic (`C28-C` vs `origin/master` & Cycle 27)

| Arm | Instructions | Loads | Stores | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `rep_stores` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `origin/master` (`six`) | `88,300,666` | — | — | `38,485,418` | `395,446` | `2,881` | `18,528` |
| Cycle 27 `LeanRegBurst6_256` (`six`) | `104,031,156` | `26,585,214` | `11,938,081` | `35,541,062` | `321,210` | `805` | `18,528` |
| **Cycle 28 `C28-C` (`six`)** | **`103,463,542` (`-568k`)** | **`26,402,287` (`-183k`)** | **`11,738,945` (`-199k`)** | **`35,400,993`** | **`321,159` (`-18.8%`)** | **`805` (`-72.1%`)** | **`18,528` (`+0`)** |
| Cycle 27 `LeanRegBurst6_256` (`dense`) | `128,973,597` | `33,304,423` | `16,027,237` | `41,529,194` | `425,156` | `144` | `15,440` |
| **Cycle 28 `C28-C` (`dense`)** | **`128,720,028` (`-254k`)** | **`33,208,417` (`-96k`)** | **`15,922,018` (`-105k`)** | **`41,464,586`** | **`425,079` (`-31.2%`)** | **`144` (`-96.5%`)** | **`15,440` (`+0`)** |

---

## 4. Keep / Kill Decisions (One-Line Reasons)

- **`C28-A` (`SWAR8+32 OutQ` + `1`-elem/`u64` `Reader::wire`):** **KILLED** — data-dependent `w <= 4` branches and defeating LLVM's 8-wide SIMD auto-vectorization in `Reader::wire` regressed `T1 wiki-431` to `1.0029×`.
- **`C28-B` (`Tight Serial Emit`):** **KILLED** — zero net speedup over Cycle 27 (`0.9837×` vs `0.9837×` `T1 wiki-431`) so unnecessary code divergence is dropped.
- **`C28-C` (`Elim::replace` on `aw == bw` + `#[inline(always)] Elim::node`):** **PROMOTED TO FINAL CYCLE 29 CANDIDATE** — cuts `-199k` stores / `-183k` loads on `six` and improves `wikidata` corpus to **`0.9711×` (`W = 1`)**, **`0.9684×` (`W = 4`)**, and **`0.9649×` (`E2E W = 4`)**.
