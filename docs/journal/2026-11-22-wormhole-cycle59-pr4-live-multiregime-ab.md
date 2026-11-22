# Wormhole Cycle 59 (Neon): Charge 4 (Part 3) — Live PR 4 `BlockSpan` Streaming Multi-Regime A/B & EOF Spec Bug Catch

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739` merged; `origin/master` HEAD `10c7673bd` has identical `src/`)  
**Mechanisms Coordination:** `origin/research/carbon-word-transport` `300a29976` (`C38b` disjoint 4-byte anchor `find_magics_anchor_range` + `C38a` block reader)  
**Harness:** [`c59_pr4_live_ab.rs`](2026-11-22-wormhole-cycle59/c59_pr4_live_ab.rs) (`#![forbid(unsafe_code)]`, same-binary A/B + A/A control, fresh-process child `ru_maxrss`/`minflt`/`vcsw`, exact `AbHash` parity)  
**Manifest:** [`c59_pr4_live_ab.toml`](2026-11-22-wormhole-cycle59/c59_pr4_live_ab.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective

Cycle 57 proved that Ping-Pong `2`-window streaming (`P2_Win2_NoBar`) beats both `#739` and single-window barrier `ParScan` (`C38b`) using precomputed `.spans`, and Cycle 58 published the formal written PR 4 `BlockSpan` streaming interface (`SpanCand`, `SpanFinder`, `WindowBuf`, `SpecOutcome`, `ChainValidator`) so Mechanisms' Handoff 1 drops straight into `StreamDecoder` without rework.

Cycle 59 removes **all** training wheels:
1. **Zero precomputed `.spans`:** plug a live `#![forbid(unsafe_code)]` `FastAnchorFinder` (strictly a test driver implementing the Cycle 58 `SpanFinder` trait, patterned after Mechanisms `C38b`'s 4-byte disjoint anchor stride; Mechanisms owns the production scanner) directly into `decode_pr4_live`.
2. **Multi-regime A/B against `origin/master` `3296195e3` (`M739_A` + `M739_B` A/A control):** test `PR4_Live_4M` (`win_cap = 4 MiB`, `MAX_LIVE_WINS = 2`) and `PR4_Live_2M` (`win_cap = 2 MiB`, `MAX_LIVE_WINS = 2`) across **5 regimes**:
   - `wikidata-l9 4W` (`209.83 MB`, `431` blocks, `15.59 MiB` bz2, `12` rounds)
   - `wikidata-l9 10W` (`209.83 MB`, `431` blocks, `5` rounds)
   - `wikidata-l1 4W` (`37.80 MB`, `379` blocks, `3.94 MiB` bz2, `12` rounds — `3×` block rate)
   - `hetero-l9 4W` (`53.52 MB`, `60` blocks, `8.48 MiB` bz2, `12` rounds — fast compressible + heavy incompressible `0.9 MiB` tail)
   - `random-l9 4W` (`10.68 MB`, `12` blocks, `10.73 MiB` bz2, `12` rounds — `100%` incompressible `LazyCells`)

---

## 2. Two Real PR 4 Spec Bugs Caught & Fixed Before Final Measurement

Our very first live run ([`c59_prefx_results.txt`](2026-11-22-wormhole-cycle59/c59_prefx_results.txt)) surfaced an unexpected `+3.6%` slowdown on `hetero-l9` and `+7.6%` slowdown on `random-l9`, together with `fb=1` on every single corpus (`430/431`, `59/60`, `11/12`), even though the live anchor finder itself scanned `4 MiB` windows in only **`0.5–0.7 ms`** (`~5.7 GiB/s` pure safe Rust).

Forensics isolated **two concrete bugs** in the Cycle 58 reference sketch that MUST be explicit invariants in the PR 4 production spec:

### Bug 1 (Spec Invariant): `published_seq` EOF In-Flight Worker Race
In Cycle 58's `take_next_spec`, EOF termination checked:
```rust
if st.win_closed && tq.0.lock().unwrap().q.is_empty() { return None; }
```
**Race:** When the producer hits EOF, pushes the final block into `tq.q`, and sets `st.win_closed = true`, a worker immediately pops that final block — making `tq.q.is_empty() == true` **while the worker is still decoding it**. The consumer then sees `ring[idx] == None && st.win_closed && tq.q.is_empty()`, returns `None`, and **decodes the last block a second time serially on the consumer thread** (`fb=1`). On `12`-block `random-l9`, re-decoding the final heavy `LazyCells` block adds `1/12 = +8.3%` extra CPU (`1.0761×` measured); on `hetero-l9`, the final block is also an incompressible `LazyCells` block (`1.0357×` measured).

**Fix (mandatory PR 4 invariant):** Track `published_seq: usize` inside `ReadyRing` (updated atomically under `ready.lock()` alongside `st.win_closed = true` when the producer finishes). `take_next_spec` returns `None` **only** when `st.win_closed && st.taken_seq >= st.published_seq`. Result: **`fb = 0` (`882/882` = `100.00%` fast hits across all 4 corpora!)**.

### Bug 2 (Hygiene Invariant): Zero-`memset` Window Refill
Cycle 58's refill loop did `buf.clear(); while buf.len() < win_cap { buf.resize(win_cap, 0); reader.read(&mut buf[cur..]); }`. Even on regular files, that zero-fills `4 MiB` on every recycled window before `read()` immediately overwrites it; on a `64 KiB` pipe it degrades to $O(\text{win\_cap}^2 / \text{chunk})$ zeroing!

**Fix:** Keep recycled window buffers sized to `win_cap` (`if buf.len() < win_cap { buf.resize(win_cap, 0); }`), copy `tail_carry` into `&mut buf[..filled]`, read directly into `&mut buf[filled..win_cap]`, and `buf.truncate(filled)` only after the read loop completes. Zero `memset` per recycled window and $O(\text{win\_cap})$ on arbitrary pipe chunk sizes.

---

## 3. Headline Results: Fixed Live PR 4 Multi-Regime A/B (`c59_results.txt`)

All 4 arms verified 100% byte-exact via `AbHash` on every single round of every regime (`431/431`, `379/379`, `60/60`, `12/12` — **`882/882 = 100.0%` fast hits, `0` fallbacks**).

### Table 1 — `wikidata-l9 4W` (`209.83 MB`, `431` blocks, `12` rotated rounds)
| Arm | med_ms | vs `M739_A` `[95% CI]` | sys_ms | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---:|---|---:|---:|---:|---:|---:|
| `M739_A` (`3296195e3`) | `344.75` | `1.0000 [1.0000, 1.0000]` | `23.74` | `10,427` | `214` | `56.32` | — |
| `M739_B` (A/A control) | `345.47` | `0.9917 [0.9697, 1.0142]` | `23.09` | `10,268` | `212` | `56.50` | — |
| **`PR4_Live_4M`** | **`341.88`** | **`0.9782 [0.9535, 1.0034]`** | **`19.01` (`-19.9%`)** | **`6,711` (`-35.6%`)** | **`142` (`-33.6%`)** | **`43.72` (`0.776×`)** | **`431/431 (fb=0)`** |
| `PR4_Live_2M` | `339.96` | `0.9797 [0.9626, 0.9971]` | `14.65` (`-38.3%`) | `5,689` (`-45.4%`) | `137` (`-36.0%`) | `39.69` (`0.705×`) | `431/431 (fb=0)` |

### Table 2 — `wikidata-l9 10W` (`209.83 MB`, `431` blocks, `5` rotated rounds)
| Arm | med_ms | vs `M739_A` `[95% CI]` | sys_ms | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---:|---|---:|---:|---:|---:|---:|
| `M739_A` (`3296195e3`) | `383.06` | `1.0000 [1.0000, 1.0000]` | `35.74` | `15,919` | `481` | `76.82` | — |
| `M739_B` (A/A control) | `383.89` | `1.0016 [0.9835, 1.0200]` | `31.64` | `16,036` | `471` | `76.39` | — |
| **`PR4_Live_4M`** | **`363.93`** | **`0.9521 [0.9391, 0.9654]`** | **`29.02` (`-18.8%`)** | **`13,329` (`-16.3%`)** | **`131` (`-72.8%`)** | **`69.65` (`0.907×`)** | **`431/431 (fb=0)`** |
| `PR4_Live_2M` | `363.71` | `0.9475 [0.9360, 0.9592]` | `28.53` (`-20.2%`) | `12,290` (`-22.8%`) | `123` (`-74.4%`) | `65.43` (`0.852×`) | `431/431 (fb=0)` |

### Table 3 — `wikidata-l1 4W` (`37.80 MB`, `379` blocks, `3×` block rate, `12` rotated rounds)
| Arm | med_ms | vs `M739_A` `[95% CI]` | sys_ms | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---:|---|---:|---:|---:|---:|---:|
| `M739_A` (`3296195e3`) | `96.01` | `1.0000 [1.0000, 1.0000]` | `8.99` | `2,991` | `289` | `18.07` | — |
| `M739_B` (A/A control) | `95.75` | `0.9994 [0.9860, 1.0131]` | `10.33` | `2,991` | `288` | `17.91` | — |
| **`PR4_Live_4M`** | **`90.62`** | **`0.9410 [0.9313, 0.9509]`** | **`7.95` (`-11.6%`)** | **`2,961` (`-1.0%`)** | **`107` (`-63.0%`)** | **`17.92` (`0.992×`)** | **`379/379 (fb=0)`** |
| `PR4_Live_2M` | `90.29` | `0.9398 [0.9237, 0.9563]` | `5.86` (`-34.8%`) | `2,962` (`-1.0%`) | `114` (`-60.6%`) | `17.86` (`0.988×`) | `379/379 (fb=0)` |

### Table 4 — `hetero-l9 4W` (`53.52 MB`, `60` blocks — mixed fast + `0.9 MiB` incompressible tail, `12` rounds)
| Arm | med_ms | vs `M739_A` `[95% CI]` | sys_ms | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---:|---|---:|---:|---:|---:|---:|
| `M739_A` (`3296195e3`) | `155.76` | `1.0000 [1.0000, 1.0000]` | `35.42` | `17,365` | `59` | `77.87` | — |
| `M739_B` (A/A control) | `156.32` | `0.9948 [0.9607, 1.0301]` | `41.76` | `17,608` | `60` | `77.24` | — |
| **`PR4_Live_4M`** | **`151.79`** | **`0.9827 [0.9566, 1.0095]`** | **`34.92` (`-1.4%`)** | **`15,628` (`-10.0%`)** | **`48` (`-18.6%`)** | **`70.86` (`0.910×`)** | **`60/60 (fb=0)`** |
| `PR4_Live_2M` | `165.23` | **`1.0563 [1.0105, 1.1041]`** | `28.55` (`-19.4%`) | `13,930` (`-19.8%`) | `60` (`+1.7%`) | `64.26` (`0.825×`) | `60/60 (fb=0)` |

### Table 5 — `random-l9 4W` (`10.68 MB`, `12` incompressible `LazyCells` blocks, `12` rounds)
| Arm | med_ms | vs `M739_A` `[95% CI]` | sys_ms | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---:|---|---:|---:|---:|---:|---:|
| `M739_A` (`3296195e3`) | `113.77` | `1.0000 [1.0000, 1.0000]` | `30.81` | `13,801` | `31` | `62.63` | — |
| `M739_B` (A/A control) | `117.40` | `1.0194 [0.9781, 1.0625]` | `30.65` | `13,405` | `31` | `62.67` | — |
| **`PR4_Live_4M`** | **`114.82`** | **`0.9887 [0.9518, 1.0271]`** | **`27.68` (`-10.2%`)** | **`12,563` (`-9.0%`)** | `33` (`+6.5%`) | **`60.92` (`0.973×`)** | **`12/12 (fb=0)`** |
| `PR4_Live_2M` | `112.23` | `0.9836 [0.9558, 1.0123]` | `26.71` (`-13.3%`) | `11,057` (`-19.9%`) | `52` (`+67.7%`) | `55.58` (`0.887×`) | `12/12 (fb=0)` |

---

## 4. Mechanism: Why `PR4_Live_4M` Dominates `PR4_Live_2M` (Theorem 59.1 — Window Floor for Incompressible `BZh9`)

Tracing `PR4_Live_2M` on `random-l9` and `hetero-l9` exposed a clean structural floor:
- A worst-case incompressible `BZh9` block is **`~0.90 MiB`** compressed.
- Under `win_cap = 2 MiB`, each window holds at most **`3` block starts**, so `publish_n = cands.len() - 1 = 2` blocks per window (carrying `~289 KiB` of block 3 to the next window).
- With `MAX_LIVE_WINS = 2`, while the consumer holds `cur_win` (Window $k$) and workers decode Window $k+1$, **at most `4` blocks exist in flight across both windows**. As soon as fast blocks drain into the incompressible `0.9 MiB` tail of `hetero-l9`, workers finish early blocks and stall waiting for the consumer to finish Window $k$ and release the `Arc` before the producer can read Window $k+2$ — causing `vcsw` to jump (`33→52` on `random-l9`, `48→60` on `hetero-l9`) and **`hetero-l9` to regress to `1.0563× [1.0105, 1.1041]`**.
- Under **`win_cap = 4 MiB` (`DEFAULT_WIN_CAP = 4 << 20`, `MAX_LIVE_WINS = 2`)**, even `100%` incompressible `BZh9` yields **`4` published blocks per window** (`8` blocks across 2 live windows) and typical text yields **`~115` blocks per window**, eliminating window-boundary starvation completely while keeping peak RSS strictly **below `#739` across all 5 regimes (`0.776×–0.992×` vs `#741`'s `1.28×`!)**.

---

## 5. Kill / Promote Decisions (One-Line Verdicts)

- **PROMOTED `PR4_Live_4M` (`DEFAULT_WIN_CAP = 4 MiB`, `MAX_LIVE_WINS = 2`, `published_seq` EOF invariant, zero-`memset` refill):** `882/882` (`100.0%`) fast hits (`fb=0`), beats/matches `M739_A` on all 5 regimes (`10W 0.9521×`, `l1 0.9410×`, `4W 0.9782×`, `hetero 0.9827×`, `random 0.9887×`) with **lower peak RSS than `#739` everywhere (`0.776×–0.992×`)** and `-63%..-73%` `vcsw` at `10W`/`l1`.
- **KILLED `PR4_Live_2M` (`win_cap = 2 MiB` at `MAX_LIVE_WINS = 2`):** regresses `hetero-l9 4W` to **`1.0563× [1.0105, 1.1041]`** because `pub=2` incompressible `0.9 MiB` blocks/window starves the worker pool across window boundaries.

---

## 6. Next Step

Cycle 60 is **Round 6 Interim Handoff #2 (`Cycles 56–60`)**, synthesizing Charge 2 (the ready-to-queue i9 `workers × slots` grid) and Charge 4 (the complete, bug-hardened, live-verified PR 4 `BlockSpan` streaming interface spec for Mechanisms & the Coordinator).
