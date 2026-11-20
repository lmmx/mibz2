# Carbon Kitchen Sink — Cycle 2: Étale Sheet Shortcutting (`OIBS-H`) and Left-Regular-Band Stream Fusion (`LRB-Open`)

*20 November 2026 — Branch `research/carbon-kitchen-sink` (autonomous cycle 2)*

## 1. Escape Routes from Cycle 1's Ephemeral-Run Obstruction

Cycle 1 established the **Ephemeral-Run Materialisation Obstruction** (`docs/journal/2026-11-20-carbon-kitchen-sink-cycle1.md` §5): on the six authenticated Wikidata blocks (`1/64 ≤ R/n < 0.15`), retained rows satisfy $m/r \in [1.56, 1.97]$, so materialising explicit $O(r)$ run arrays plus one $2r$ merge (`649–1,739 µs`) already exceeds `Open`'s streaming register-only `pass1` (`407–914 µs`) by more than the entire `emit` savings (`152–503 µs`).

Cycle 1 recorded two concrete escape conditions that keep runs completely ephemeral:
1. **Zero-producer-cost online étale sheet shortcutting inside `Open::emit` (`OPEN-QUESTIONS.md` §4):** view `Open`'s packed transition table `edges[0..m]` as a finite étale partial bijection on retained intervals, certify local translation sheets (`edges[j ± d] == edges[j] ± d`) in the same cache line during `emit`, and overwrite unvisited sibling entries in place with output-buffer shortcuts (`OIBS-H`).
2. **Pre-column Left-Regular-Band stream fusion (`FRONTIER.md` Obligation 5, "Starting from the decoded column"):** apply Cycle 1 Theorem 3 (every literal symbol $s \ge 2$ before `eob` in libbzip2's canonical MTF+RLE2 stream is a guaranteed maximal run boundary) to fuse `src/runs.rs:front_avx2` directly into `Open`'s `(queues, strips)` (`LRB-Open-AVX2`), eliminating the intermediate `Runs.runs: Vec<u16>` buffer (`112–266 KB`), `CAP=127` run splitting (`3,990–5,284` artificial split records per block), and `Open::each_run`.

---

## 2. Mathematical Constructions and Exact Proofs

### 2.1 Theorem 1 — Packed Étale Sheet Coherence Certificate (**proved**)

Let `Open` construct `edges[0..m]` with shift width $\text{bits} = \lfloor \log_2 m \rfloor + 1$ so that $m < 2^{\text{bits}}$ strictly (leaving bit 31 clear whenever $255 + n_{\text{words}} \le \lfloor \text{0x7FFFFFFF} / 2^{\text{bits}} \rfloor$, which holds on all six blocks where $m \le 204,281 \implies \text{bits} \le 18$ and $n_{\text{words}} \le 12,000$). Pad `edges` on both ends with 32 sentinel slots equal to $\text{EDGE\_SENTINEL} = \text{0x7FFFFFFD}$.

**Claim (proved).** For any retained index $j \in [0, m)$ carrying an unmodified packed edge $e = (\text{code}(j) \ll \text{bits}) \mid \text{next}(j)$ and any integer offset $d \in [-15, 15]$, the single 32-bit integer equality
$$\text{edges}[j + d] = e + d$$
holds if and only if:
1. $j + d \in [0, m)$ is a valid unmodified retained index (not a sentinel, visited leader `0x7FFFFFFE`, or shortcut `≥ 0x80000000`),
2. $\text{code}(j + d) = \text{code}(j)$ (identical emitted byte or grammar word), and
3. $\text{next}(j + d) = \text{next}(j) + d \in [0, m)$.

*Proof.* ($\Leftarrow$) Immediate from $e + d = (\text{code}(j) \ll \text{bits}) + (\text{next}(j) + d)$ when $\text{next}(j) + d \in [0, m) \subset [0, 2^{\text{bits}})$.
($\Rightarrow$) With $\text{bits}$ chosen so that $m + 15 \le 2^{\text{bits}}$ (which holds with margin $\ge 19,002$ on all six admitted blocks, where $m \le 204,281$ and $2^{\text{bits}} \in \{131,072, \, 262,144\}$) and `!wide` keeping bit 31 clear, $\text{next}(j) \in [0, m - 1]$ satisfies $\text{next}(j) + d \in [-15, m + 14] \subset [-15, 2^{\text{bits}} - 1]$. If $\text{next}(j) + d < 0$ ($d < 0$), $e + d$ borrows 1 from $\text{code}(j)$ and leaves low $\text{bits}$ bits in $[2^{\text{bits}} - 15, 2^{\text{bits}} - 1]$, which lies strictly above $m - 1$ and therefore cannot match any valid unmodified entry in `edges[0..m]`. Nor can $e + d$ equal `EDGE_SENTINEL` (`0x7FFFFFFD`), `VISITED_LEADER` (`0x7FFFFFFE`), or a shortcut (`≥ 0x80000000`). Hence $\text{edges}[j + d] = e + d$ forces $j + d \in [0, m)$ to be an unmodified retained index with $\text{next}(j + d) = \text{next}(j) + d \ge 0$ and $\text{code}(j + d) = \text{code}(j)$. $\square$

### 2.2 Theorem 2 — Cyclic No-Wraparound Lemma (**proved**, witness caught in Gate G1)

During initial small-case confrontation of `OIBS`, the exhaustive verifier caught an `IndexError` on the 7-byte binary column
$$L = [0, 0, 1, 1, 0, 0, 0], \qquad p = 3, \qquad \text{LIMIT} = 16, \qquad h_{\text{trig}} = 1.$$
On this witness (`m = 7, mark = 3`), at `start_j = 5` (`out_len = 5`), sibling $d = +1$ (`sj = 6`) satisfied sheet coherence for two steps (`6 -> 3 -> 5`): its **intermediate** step 1 passed through `mark = 3` and step 2 wrapped onto `start_j = 5` (`VISITED_LEADER`), recording a shortcut on an already-visited orbit prefix!

**Lemma (No-Wraparound on $\mathbb{Z}/P\mathbb{Z}$, proved).** Let $C = (j_0 = \text{mark}, j_1, \dots, j_{P-1})$ be the cycle of retained indices visited from $\text{mark}$, with cycle index $\tau : C \xrightarrow{\sim} \mathbb{Z}/P\mathbb{Z}$ satisfying $\tau(\text{next}(x)) \equiv \tau(x) + 1 \pmod{P}$. Suppose at $\tau(j_t) = t$ a coherent sheet walks $K \ge 1$ steps along $x_0 = j_t, x_1 = j_{t+1}, \dots, x_K = j_{t+K}$ ($t + K \le P$), and for an offset $d \ne 0$:
1. $x_0 + d$ is unvisited at time $t$ (so either $x_0 + d \notin C$, or $\tau(x_0 + d) \in \{t + 1, \dots, P - 1\}$), and
2. For every step $0 \le s < K$, $x_s + d$ is unmodified (`!= VISITED_LEADER`, unshortcutted) **and** $x_s + d \ne \text{mark}$.

Then:
- If $x_0 + d \notin C$, $x_s + d \notin C$ for all $0 \le s \le K$ (the shortcut is never reached and harmless).
- If $x_0 + d \in C$, let $u = \tau(x_0 + d) \in \{t + 1, \dots, P - 1\}$. Because $x_s + d \ne \text{mark}$ ($\tau \ne 0$) for all $0 \le s < K$, incrementing $\tau$ by $+1 \pmod P$ $K$ times cannot wrap $P - 1 \to 0$, so $u + K \le P$. Furthermore, since $x_s + d \ne \text{VISITED\_LEADER}$ during the walk, $x_0 + d \notin \{x_0, \dots, x_{K-1}\}$, which forces $u \ge t + K$. Hence
  $$t < t + K \le u < u + K \le P.$$
  Therefore the sibling segment $(x_0 + d, \dots, x_K + d)$ is strictly in the future of $t + K$, disjoint from $(x_0, \dots, x_K)$, emits the identical byte substring `out[start_out .. start_out + blen]`, and when later reached at $\tau = u \ge t + K$ copies from `[start_out, start_out + blen)` strictly below the current `out_len`. $\square$

### 2.3 Theorem 3 — Left-Regular-Band Direct Pre-Column Fusion (`LRB-Open`, **proved**)

By Cycle 1 Theorem 3 (`docs/journal/2026-11-20-carbon-kitchen-sink-cycle1.md` §3.4), in any canonical bzip2 MTF+RLE2 `Symbols` stream (`syms[0..M_syms]` ending at `eob = alpha + 1`):
1. Every literal symbol $s \in \{2, \dots, \alpha\}$ moves `yy[s - 1]` to `yy[0]`; since MTF maintains distinct bytes in `yy[0..\alpha]`, `yy[s - 1] != yy[0]`, so every literal marks a **strictly distinct** maximal run byte from its predecessor.
2. All `RUNA`/`RUNB` (`0`/`1`) bijective base-2 digits between consecutive literals encode `width - 1` of that single maximal run (`width` for a leading initial run of `yy[0]` before the first literal).

Consequently, replacing `EMIT_CAP127` inside `src/runs.rs:front_avx2` (`lit`/`one` bitmaps + `_bzhi_u64` + 2-register `STEPS[32]` PSHUFB MTF kernel) with direct emission into `Open`'s `(queues, strips)` (`VISIT_MAXIMAL_RUN` in `g5_cycle2_bench.c:957-987`) produces the **exact identical** `(all, thin, thick, m, mark, queues, strips)` state as `runs::front` followed by `Open::each_run`, with zero `Runs.runs` allocation, zero `CAP=127` splitting, and zero second-stage record scan. $\square$

---

## 3. Static Simulation, Gate G1 Verification, and Gate G2 Manifests

### 3.1 Static `OIBS` Potential & `CAP=127` Record Overhead (`probe_cycle2.py`, **measured**)

| Block | Maximal runs $r$ | Capped records $R$ | Extra `CAP=127` records | Retained $m$ | Static coherent `%` of $m$ | `OIBS` ($k_{\min}=4$) hops / $m$ | Shortcut hit rate (`taken / written`) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 56,386 | 5,202 (`+10.2%`) | 89,874 | 61.12% | **0.6752×** (`60,686`) | `1,661 / 1,855` (**89.5%**) |
| `block-2.L` | 54,532 | 59,478 | 4,946 (`+9.1%`) | 112,070 | 69.57% | **0.6087×** (`68,215`) | `1,886 / 2,251` (**83.8%**) |
| `block-85.L` | 79,375 | 84,659 | 5,284 (`+6.7%`) | 156,404 | 64.58% | **0.6575×** (`102,831`) | `2,847 / 3,355` (**84.9%**) |
| `block-143.L` | 108,335 | 112,325 | 3,990 (`+3.7%`) | 196,841 | 63.26% | **0.6996×** (`137,703`) | `3,380 / 3,810` (**88.7%**) |
| `block-207.L` | 87,111 | 91,416 | 4,305 (`+4.9%`) | 135,921 | 54.28% | **0.7767×** (`105,573`) | `2,683 / 2,979` (**90.1%**) |
| `block-210.L` | 128,732 | 133,233 | 4,501 (`+3.5%`) | 204,281 | 52.56% | **0.7657×** (`156,416`) | `3,561 / 3,897` (**91.4%**) |

### 3.2 Gate G1 Exhaustive Small-Case & Real-Block Verification (`verify_cycle2.py`, **verified**)

- **Small-case universe (`38,838 / 38,838` configurations passed byte-exact against dense `LF`):**
  - Every binary column $L \in \{0, 1\}^n$ for $1 \le n \le 8$ and every mark $0 \le p < n$, across `LIMIT ∈ {1, 2, 4, 16}` and `h_trig ∈ {1, 4}` (`32,274` configurations, covering periodic, aperiodic, multi-cycle, and non-BWT columns).
  - Every ternary column $L \in \{0, 1, 2\}^n$ for $1 \le n \le 5$ and every mark $0 \le p < n$, across `LIMIT ∈ {1, 2, 4, 16}` and `h_trig ∈ {1, 4}` (`6,564` configurations).
  - Verified **both** `LRB-Open` (reconstructing canonical bzip2 `Symbols` and fusing directly into `Open`) and `OIBS-H` simultaneously on every configuration.
- **Authenticated Wikidata blocks (`6 / 6` passed byte-exact):**
  - `OIBS-H4` (`h_trig = 4, k_min = 4`) enters the cold path on only **`3.25%–4.41%`** of executed hops (`89.4%–95.7%` shortcut hit rate) and reduces executed hops by **`15.3%–36.2%`**.

### 3.3 Gate G2 Acquisition Manifests (**verified**)

Both manifests pass `python3 docs/team/acquisition/check_manifest.py`:
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/oibs.toml` (`PASS: Online In-Place Bundle Shortcutting on Open (OIBS-H)`)
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/lrb_open.toml` (`PASS: Left-Regular-Band Direct Pre-Column Fusion into Open (LRB-Open-AVX2)`)

---

## 4. Gate G5 Hardware Confrontation (`g5_cycle2_bench.c`, **measured**)

Compiled with `gcc -O3 -mavx2 -mbmi -mbmi2 -mlzcnt -mpopcnt -Wall -Wextra` (matching `mibz2`'s x86-64 AVX2/BMI2 target without AVX-512; verified zero memory/UB errors under `-fsanitize=address,undefined`, plus byte-exactness against `dense_lf_invert` and 64-byte `0xA5` tail canary across all 7 arms × 6 blocks; 21 paired interleaved rounds after 3 warm-up rounds).

**Calibration check against native Rust `Open` (`docs/journal/2026-11-20-open-u16q.md`):** our C baseline `A0:Open-qi16` (`1.432, 1.834, 1.873, 3.328, 2.150, 2.845 ms`) matches the Coordinator's native Rust `Open` (`master`: `1.426, 1.812, 1.887, 3.415, 2.172, 3.056 ms`) within **`0.4%–2.5%`** on blocks 0, 2, 85, 143, 207 (`6.9%` faster on 210).

### 4.1 Group A — `records` Entry (`OIBS-H4` / `OIBS-H6` vs `Open-fast16`)

*Pre-registered G5 rule:* `OIBS-H4-fast16` or `OIBS-H6-fast16` passes iff median total time $\le 0.95\times$ `A1:Open-fast16` (and `emitMed` $\le 0.90\times$ `A1` `emitMed`) on $\ge 4/6$ blocks with none $> 1.02\times$.

*Note on Theorem 1 bound in `g5_cycle2_bench.c`:* on all six blocks $m \le 204,281$ and $\text{bits} = \lfloor \log_2 m \rfloor + 1 \in \{17, 18\}$, with $2^{17} - m = 131,072 - 89,874 = 41,198 > 15$ (`block-0`), $131,072 - 112,070 = 19,002 > 15$ (`block-2`), and $2^{18} - m \ge 262,144 - 204,281 = 57,863 > 15$ (`blocks 85, 143, 207, 210`), so $m + 15 < 2^{\text{bits}}$ holds with huge margin on every block.

| Block | `A0:Open-qi16` (µs) | `A1:Open-fast16` (µs) | `A2:OIBS-H4` (µs / ratio vs A1) | `A3:OIBS-H6` (µs / ratio vs A1) | `A1` `emitMed` (µs) | `A2` `emitMed` (µs) | `A2` hops vs `m` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 1,432.4 | 1,371.4 | 1,622.9 (**1.183×**) | 1,634.5 (**1.192×**) | 496.7 | 748.4 (`1.507×`) | `65,085 / 89,874` (`-27.6%`) |
| `block-2.L` | 1,833.5 | 1,728.1 | 1,947.5 (**1.127×**) | 1,978.4 (**1.145×**) | 655.7 | 883.3 (`1.347×`) | `71,549 / 112,070` (`-36.2%`) |
| `block-85.L` | 1,873.1 | 1,825.5 | 2,175.3 (**1.192×**) | 2,138.5 (**1.171×**) | 885.7 | 1,254.7 (`1.417×`) | `106,416 / 156,404` (`-32.0%`) |
| `block-143.L` | 3,327.8 | 3,152.2 | 3,659.9 (**1.161×**) | 3,644.0 (**1.156×**) | 1,372.3 | 1,862.9 (`1.358×`) | `148,287 / 196,841` (`-24.7%`) |
| `block-207.L` | 2,149.7 | 2,099.4 | 2,664.7 (**1.269×**) | 2,551.5 (**1.215×**) | 867.2 | 1,405.3 (`1.620×`) | `115,156 / 135,921` (`-15.3%`) |
| `block-210.L` | 2,845.1 | 2,795.0 | 3,350.1 (**1.199×**) | 3,298.6 (**1.180×**) | 1,356.1 | 1,918.2 (`1.415×`) | `169,325 / 204,281` (`-17.1%`) |

**Verdict on Group A (`OIBS-H`): FAILS Gate G5 (`0 / 6` blocks) — closes `OPEN-QUESTIONS.md` §4.**

**Why `OIBS-H` loses despite cutting `15.3%–36.2%` of dependent pointer-chase hops (L2-Resident Online-Shortcutting Obstruction):**
- **Witness:** `g5_cycle2_bench.c` (`A2:OIBS-H4-fast16` vs `A1:Open-fast16`): on `block-2.L`, `OIBS-H4` skips `40,521` dependent hops (`36.2%` of `112,070`), yet `emitMed` rises from `655.7 µs` to `883.3 µs` (`+227.6 µs`); on `block-207.L`, `OIBS-H4` skips `20,765` hops (`15.3%`), yet `emitMed` rises from `867.2 µs` to `1,405.3 µs` (`+538.1 µs`).
- **Exact microarchitectural mechanism:** `Open`'s `edges[0..m]` occupies `360–817 KB`, sitting **entirely inside the 2 MiB private L2 cache**. `A1:Open-fast16` pays only **`5.53–6.97 ns/hop`** (~15 cycles/hop), because out-of-order execution hides all loop arithmetic and `memcpy` work behind the clean single-dependency recurrence `j = edges[j] & mask`. Online sheet shortcutting perturbs that recurrence in four ways:
  1. `edges[j - 1]` and `edges[j + 1]` cross a 64-byte cache-line boundary on `2 / 16 = 12.5%` of hops, issuing a second random L2 line fetch;
  2. The hot/cold split `!((e & SHORTCUT_BIT) || streak >= h_trig)` adds `3,000–6,700` data-dependent unpredictable branch transitions per block (~6–7 ns misprediction penalty each);
  3. Inside the cold sheet walk, scanning `[-cur_L, +cur_R]` and mutating `edges[cur_j] = VISITED_LEADER` and `edges[sj] = SHORTCUT_BIT | sid` dirties previously read-only L2 cache lines and adds data-dependent inner loop exits (`nl < cur_L`);
  4. Saving `20,765–49,988` L2-resident hops recoups at most `130–330 µs` (`~6 ns/hop`), which is smaller than the `360–670 µs` branch-mispredict + cache-line-split + dirty-writeback tax.
- **Scope:** Any single-walk online bundle-shortcutting scheme on `Open`'s `edges[0..m]` (`OPEN-QUESTIONS.md` §4) when `4m` fits in private L2 (`m ≲ 2.5 × 10^5` on 2 MiB L2) that discovers coherence dynamically during `emit` and mutates `edges` with shortcuts.
- **Escape condition:** Either shrink $m$ *statically* before `emit` so `emit` remains a branch-free read-only walk (`DECO` in Cycle 1 proved `emit` drops to **`0.630–0.694× Open`** when `edges` stays clean!), or operate in a regime where each hop is DRAM-bound (`> 40 ns/hop`) rather than L2-resident (`~6 ns/hop`).

### 4.2 Group B — Pre-Column `Symbols` Entry (`LRB-Open-AVX2` vs `TwoStage-AVX2`, Obligation 5)

*Pre-registered G5 rule:* `B1:LRB-Open-AVX2` passes iff median `Symbols`-to-`out` time $\le 0.900\times$ `B0:2Stage-AVX2` on $\ge 4/6$ blocks with none $> 1.000\times$.

| Block | `B0:2Stage-AVX2` total (µs) | `B1:LRB-Open-AVX2` total (µs) | Ratio `B1 / B0` | `B0` `front + pass1` (µs) | `B1` fused `front` (µs) | Fused front+pass1 speedup |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 1,540.6 | **1,431.5** (`-109.1`) | **0.929×** | `149.5 + 420.5 = 570.0` | **466.8** (`-103.2`) | **0.819× (`-18.1%`)** |
| `block-2.L` | 1,930.3 | **1,785.8** (`-144.5`) | **0.925×** | `159.6 + 507.0 = 666.6` | **548.8** (`-117.8`) | **0.823× (`-17.7%`)** |
| `block-85.L` | 2,033.0 | **1,896.8** (`-136.2`) | **0.933×** | `217.9 + 638.9 = 856.8` | **727.4** (`-129.4`) | **0.849× (`-15.1%`)** |
| `block-143.L` | 3,519.2 | **3,295.4** (`-223.8`) | **0.936×** | `301.8 + 927.4 = 1,229.2` | **1,028.2** (`-201.0`) | **0.836× (`-16.4%`)** |
| `block-207.L` | 2,341.8 | **2,177.1** (`-164.7`) | **0.930×** | `245.5 + 657.3 = 902.8` | **750.8** (`-152.0`) | **0.832× (`-16.8%`)** |
| `block-210.L` | 3,096.9 | **2,923.3** (`-173.6`) | **0.944×** | `337.4 + 879.2 = 1,216.6` | **1,037.0** (`-179.6`) | **0.852× (`-14.8%`)** |

**Verdict on Group B (`LRB-Open-AVX2`):**
1. **Uniform positive hardware measurement on all `6 / 6` blocks (`0.925×–0.944×` total `Symbols`-to-`out`, `0.819×–0.852×` on `front + pass1`, zero regressions, saving `109–224 µs` per block and `112–266 KB` of `Runs.runs` memory).**
2. **Pre-registered G5 gate accounting:** `0.925×–0.944×` beats `1.000×` on `6 / 6` blocks, but sits `2.5%–4.4%` above our strict pre-registered `≤ 0.900×` whole-pipeline promotion bar (`0 / 6 ≤ 0.900×`), so we do **not** claim G5 promotion yet.
3. **Diagnostic anatomy of the remaining `317–700 µs` inside `B1`'s fused `front`:**
   - Subtracting `B0`'s standalone `runs::front_avx2` (`149.5–337.4 µs`) from `B1`'s fused `front` (`466.8–1,037.0 µs`) reveals that eliminating `Runs.runs`, `CAP=127` splitting, and `each_run`'s loop saved **`103–201 µs`** (`~25%` of `pass1`), while **`317–700 µs` (`~75%` of `pass1`) remains inside `VISIT_MAXIMAL_RUN`**.
   - What is `VISIT_MAXIMAL_RUN` doing for `317–700 µs`? Almost entirely **`q_push_range` into 256 separate dynamic `q_vecs[a]` queues** (`~90` simultaneous active write streams thrashing L1d / write-combining buffers, plus variable-trip `for (i = 0; i < w; i++)` branch mispredictions on thin widths $w \in [1, 16]$)!

---

## 5. Artefacts in This Directory

- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/probe_cycle2.py` & `probe_cycle2_results.json` — static `OIBS` and `CAP=127` measurements across all 6 blocks.
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/verify_cycle2.py` — Gate G1 exhaustive verifier (`38,838 / 38,838` small configurations + `6 / 6` blocks byte-exact; caught the 7-byte wraparound witness motivating Theorem 2).
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/oibs.toml` & `lrb_open.toml` — Gate G2 acquisition manifests (both pass `check_manifest.py`).
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle2/g5_cycle2_bench.c` & `g5_cycle2_bench_output.txt` — Gate G5 21-round paired C benchmark (ASan/UBSan clean, byte-exact + 64B canary verified).
