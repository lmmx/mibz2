# T1/T2 (Cycle 4) — Lazy Thin-Run Acquisition (`NEEDS M-27`) and the `(byte, successor-byte)` Depth-2 Schedule Costed with Payloads

- **Headline (referee's terms — total reads per lone row, acquisition + build + emission included, like for like):**
  - **No route breaks $h_1 / 2$.** On the 6 admitted Wikidata blocks, replacing Cycle 3 `CL4`'s binary-search wide-strip acquisition (`0.88–4.07 h₁`, total `1.55–4.99 h₁`) with Open's right-end IET elimination (`0.10–0.34 h₁` local list steps, `0` binary searches) + linear endpoint merge (`Q_glue = 0`) brings link+build totals down to **`0.799–1.289 h₁`** (`M-17` 1-sweep, `0.899–1.628 h₁` with IET), **`0.709–1.172 h₁`** (`CL1` 2-sweep flat, `0.810–1.511 h₁` with IET), and **`0.690–1.173 h₁`** for depth-2 `(v, v_R)` `R⁴` under an L1-resident pair-cursor table (`0.790–1.512 h₁` with IET; **`1.216–1.991 h₁`** with flat emit payload reads charged), compared like for like against Open's baseline walk **`2.105–3.271 h₁`** (`2.205–3.610 h₁` with IET; **`4.210–6.542 h₁`** with flat emit dict reads). Under strict $\Pi$ ($s \le 256$ cursors), `R⁴` loses outright (**`2.668–4.180 h₁`**).
  - **Lazy thin-run contraction (`S*_1`, `NEEDS M-27`) is CLOSED negatively on 6/6 blocks** by MONODROMY Record 120's pre-registered rule: $m(S^*_1) / m_{16} = \mathbf{0.891\text{–}0.925} > 0.85$ on all 6 blocks, and punching scattered $S^*_1$ holes inflates $B_P$ by **$4.1\times\text{–}8.7\times$**, making every downstream composed route (`M-17`, `CL1`, `R²`, `R⁴`) **15–81% worse** in total $Q / h_1$.
- **Prior art:** Zero-search label-ordered strip contraction is the move-structure merge (Nishimoto & Tabei 2021) iterated as MONODROMY Record 120 Proposition A; wide-strip right-end IET elimination with spiral contraction is production Open (`src/open/endpoint.rs`, Astra #565 / Carbon cycle 19); `CL1`'s valley cut is N-20 C4 / SA-IS LMS; `R⁴`'s depth-2 squaring payload wall mirrors `T²` / Kärkkäinen–Kempa–Puglisi `LF²` (killed 23 Nov, journal §4).
- **Script & TSVs:** [`t1_t2_lazy_acq_and_depth2.py`](t1_t2_lazy_acq_and_depth2.py) derives [`data/t1_t2_lazy_thin_acquisition.tsv`](data/t1_t2_lazy_thin_acquisition.tsv) (43 rows) and [`data/t1_t2_depth2_schedule_costed.tsv`](data/t1_t2_depth2_schedule_costed.tsv) (261 rows). All 8/8 blocks (6 admitted Wikidata + 2 synthetic hex) and every route (`walk_R`, `M17`, `CL1`, `R²`, `R⁴`, across all $k$) pass byte-exact checks against a standalone independent LF oracle (`independent_lf_decode`, sharing no code with `common.py`).

---

## 1. Wide-Strip Acquisition Diagnosis (`k = 0`, `L_min = 17`)

Cycle 3's `0.88–4.07 h₁` acquisition figure (`CL4`) was an artifact of binary-search doubling over strip endpoints, not an inherent cost of wide-strip contraction:

| Block (`h₁`, `m₁₆`) | `CL4` binary searches (`/ h₁`) | Prop A rounds (visits, `Q_search=0`) | **Open IET steps** (`/ h₁`) | `B_P` (`+1` segs) | Open flat dict (`n_words`, KB) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`35,116`, `89,874`) | `142,917` (`4.070`) | `894` (`1.61M`) | **`9,763`** (`0.278`) | `2,148` | `817` (`32.5 KB`, `92.3% ≤15B`) |
| `block-2.L` (`34,262`, `112,070`) | `133,965` (`3.910`) | `520` (`1.28M`) | **`11,613`** (`0.339`) | `3,136` | `1,106` (`32.3 KB`, `92.8% ≤15B`) |
| `block-85.L` (`55,504`, `156,404`) | `56,496` (`1.018`) | `340` (`0.38M`) | **`5,722`** (`0.103`) | `2,018` | `647` (`15.7 KB`, `95.2% ≤15B`) |
| `block-143.L` (`72,578`, `196,841`) | `163,701` (`2.256`) | `1,450` (`1.83M`) | **`17,895`** (`0.247`) | `5,436` | `1,550` (`33.6 KB`, `93.7% ≤15B`) |
| `block-210.L` (`97,054`, `204,281`) | `85,302` (`0.879`) | `556` (`0.55M`) | **`9,757`** (`0.101`) | `3,564` | `1,057` (`20.3 KB`, `94.2% ≤15B`) |
| `block-207.L` (`62,407`, `135,921`) | `124,059` (`1.988`) | `1,625` (`2.10M`) | **`11,403`** (`0.183`) | `3,547` | `1,042` (`32.2 KB`, `93.9% ≤15B`) |

- **Proposition A (zero-search cursor merge)** achieves `Q_search = 0` on wide strips, **but** long repeated JSON corridors force `340–1,625` sequential rounds (`0.38M–2.10M` piece visits).
- **Open's right-end IET (`src/open/endpoint.rs`)** contracts wide strips in **`5.7k–17.9k` local O(1) linked-list steps (`0.10–0.34 h₁`)** with zero binary searches — `10–15×` cheaper than `CL4` and `25–180×` fewer piece visits than Proposition A.

---

## 2. Lazy Shared Thin-Run Acquisition (`S*_k`) — `NEEDS M-27` Discharged & Closed

Reusing the $B_P$ forward cursor intersection inside thin-run contraction eliminates every binary search (`Q_search = 0`), resolving $S^*_1 = \{y \in L_{\text{thin}} : P_W(\text{LF}(y)) \in H_1\}$ in 1 cursor merge pass and $S^*_k$ in $k$ linear merge passes (plus 1 pass to update wide-strip exits into $S^*_{\le k}$).

### `NEEDS M-27` Measurements (6/6 Wikidata Blocks — Closed by Fixed Rule `m(S*_1) > 0.85 m_16`)

| Block | Thin rows (`2≤w≤16`) | Raw LF → `H₁` / `W` / `thin` | After `P_W` → `H₁` (`S*_1`) | **`m(S*_1) / m₁₆`** (gate `≤0.85`) | `B_P`: `k=0` → `k=1` | `M-17` `Q/h₁`: `k=0` → `k=1` | `CL1` `Q/h₁`: `k=0` → `k=1` | `R⁴` L1-cur `Q/h₁`: `k=0` → `k=1` | `k=∞` rounds (`m=h₁`, `CL1 Q/h₁`) |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `54,758` | `10.3%` / `22.9%` / `66.7%` | `8,192` (`15.0%`) | **`0.909` (FAIL)** | `2,148 → 11,551` (`5.4×`) | `0.996 → 1.180` | `0.895 → 1.091` | `0.882 → 1.593` | `239` (`0.622`) |
| `block-2.L` | `77,808` | `6.9%` / `25.0%` / `68.1%` | `8,426` (`10.8%`) | **`0.925` (FAIL)** | `3,136 → 12,880` (`4.1×`) | `1.289 → 1.480` | `1.172 → 1.375` | `1.173 → 1.917` | `3,952` (`0.636`) |
| `block-85.L` | `100,900` | `8.1%` / `18.3%` / `73.6%` | `12,983` (`12.9%`) | **`0.917` (FAIL)** | `2,018 → 17,580` (`8.7×`) | `1.049 → 1.238` | `0.939 → 1.138` | `0.881 → 1.628` | `264` (`0.622`) |
| `block-143.L` | `124,263` | `8.7%` / `31.8%` / `59.6%` | `16,972` (`13.7%`) | **`0.914` (FAIL)** | `5,436 → 26,603` (`4.9×`) | `1.063 → 1.270` | `0.957 → 1.173` | `0.984 → 1.738` | `5,347` (`0.639`) |
| `block-210.L` | `107,227` | `13.0%` / `28.5%` / `58.5%` | `20,732` (`19.3%`) | **`0.899` (FAIL)** | `3,564 → 28,083` (`7.9×`) | `0.799 → 0.975` | `0.709 → 0.897` | `0.690 → 1.390` | `2,497` (`0.601`) |
| `block-207.L` | `73,514` | `12.5%` / `29.0%` / `58.6%` | `14,803` (`20.1%`) | **`0.891` (FAIL)** | `3,547 → 20,073` (`5.7×`) | `0.854 → 1.022` | `0.768 → 0.950` | `0.794 → 1.485` | `547` (`0.622`) |

**Why lazy thin-run contraction loses on real blocks:**
1. On Wikidata, `59–74%` of thin-run rows land directly in another thin run under raw `LF`, and **`80–89%` still land in a thin run even after wide-strip contraction `P_W`** (contrasting sharply with the `80%` exit heuristic conjectured in Record 120 §4). Even after `k = 4` rounds (`P = 7` passes), `m` only falls to `0.776–0.838 m₁₆`.
2. Removing scattered `S*_1` rows punches thousands of holes in `M₁₆`, **exploding `B_P` by `4.1–8.7×`** (`+9.4k` to `+24.5k` segments) while removing only `8.2k–20.7k` rows (`~2.7k–6.9k` emit links after composition). Every composed scheme therefore pays **more** in extra build restarts than it saves in emit links!
3. Full fixpoint contraction (`k = ∞`, `m = h₁`) requires `239–5,347` rounds (violating $P = O(1)$) and still plateaus at `0.601–0.639 h₁` (`CL1`), above $h_1 / 2$.

---

## 3. The `(byte, successor-byte)` Depth-2 Schedule (`R⁴`) Costed with Payloads (`k = 0`, `M₁₆`)

All figures below are at `k = 0` (`M₁₆`, zero-search acquisition, like-for-like across all 6 Wikidata blocks; full table in [`data/t1_t2_depth2_schedule_costed.tsv`](data/t1_t2_depth2_schedule_costed.tsv)):

| Route (`k = 0`, `M₁₆`) | Cursor model (`s`) | Links + Build `Q / h₁` (`block-0`) | Links + Build `Q / h₁` (6-block range) | **+ Open IET** (6-block range) | **+ Emit payload reads** (6-block range) | Passes `P` | Table `S_tbl` | Payload `S_pay` |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| **Open baseline `walk_R_flat`** | strict $\Pi$ (`256`) | `2.559` | `2.105–3.271` | `2.205–3.610` | `4.210–6.542` | `1` | `0.35–0.80 MB` | **`16–34 KB`** |
| `CL1 + CL4` (Cycle 3, binary-search + DAG) | strict $\Pi$ (`256`) | `4.904` | `1.552–4.990` | *(incl. bsearch)* | `4.246–6.634` *(excl. bsearch)* | `≥3` | `1.25–2.16 MB` | `16–34 KB` |
| `M17_1sweep_flat` (Open IET + `B_P`) | strict $\Pi$ (`256`) | `0.996` | `0.799–1.289` | `0.899–1.628` | `1.561–2.486` | `2` | `0.35–0.80 MB` | `2.39–2.59 MB` |
| `CL1_2sweep_DAG` (Open IET + `B_P`) | strict $\Pi$ (`256`) | `0.895` | `0.709–1.172` | `0.810–1.511` | `4.246–6.634` *(DAG!)* | `3` | `1.05–2.39 MB` | `16–34 KB` |
| `CL1_2sweep_flat` (Open IET + `B_P`) | strict $\Pi$ (`256`) | `0.895` | `0.709–1.172` | `0.810–1.511` | `1.382–2.253` | `3` | `0.35–0.80 MB` | `2.81–3.01 MB` |
| `R2_byte_flat` (`B_P` byte sched) | strict $\Pi$ (`256`) | `1.341` | `1.089–1.727` | `1.190–2.066` | `2.142–3.363` | `2` | `0.70–1.60 MB` | `1.76 MB` (`2n`) |
| **`R4_pair_4codes` (`(v, v_R)` L1-cur)** | L1 table (`1,908–2,928`, `7.5–11.4 KB`) | **`0.882`** | **`0.690–1.173`** | **`0.790–1.512`** | `2.794–4.444` *(4 dict/link!)* | `3` | `1.76–4.00 MB` | **`16–34 KB`** |
| **`R4_pair_flat` (`(v, v_R)` L1-cur)** | L1 table (`1,908–2,928`, `7.5–11.4 KB`) | **`0.882`** | **`0.690–1.173`** | **`0.790–1.512`** | **`1.216–1.991`** | `3` | `0.71–1.61 MB` | **`3.52 MB`** (`4n`) |
| `R4_pair_strictPi_flat` (`(v, v_R)`) | strict $\Pi$ (`256`) | `3.260` | `2.668–4.180` | `2.768–4.519` | `3.194–4.998` | `3` | `0.70–1.60 MB` | `3.52 MB` (`4n`) |

### Verdict on Depth-2 `(v, v_R)` Schedule (`R⁴`)
1. **Cursor table fits L1d, but violates strict $\Pi$:** Only `1,908–2,928` `(v[j], v[R[j]])` pairs are active on Wikidata (`7.5–11.4 KiB` at 4B/cursor, L1d-resident), yielding `6,339–16,785` `+1` segments (`0.13–0.26 h₁`). Under strict $\Pi$ ($s \le 256$), exceeding 256 cursors forces $m$ addressed gathers (`2.67–4.18 h₁`, worse than walking $R$).
2. **Even with the L1 cursor table, payloads impose a hard memory/work wall:**
   - **4-code tuples `(c0,c1,c2,c3)`:** Avoid byte-copying in build and keep the dictionary at `16–34 KB`, **but** still execute `4 × (m/4) = m` dictionary lookups at emit (`2.79–4.44 h₁` with payloads) and inflate the edge table to `1.76–4.00 MB`.
   - **Flattened 4-words (`R4_pair_flat`):** Achieves the **lowest total read count of any constant-pass route (`1.22–1.99 h₁` including payload reads, `-12%` to `-22%` below `M17_flat` and `-69%` to `-71%` below Open's `walk_R_flat`)**, **but** must materialize `4n = 3.52 MB` of flat words per block (`4.2–5.1 MB` total per worker — `>2×` Open's cache share at 10 workers on a 20 MiB L3, repeating the exact memory/build-work trap that killed `T²` on 23 Nov).
   - **DAG payloads (`CL1_2sweep_DAG`):** Confirmed unusable under honest referee accounting (`4.25–6.63 h₁` once `m-1` internal DAG visits + `m` leaf loads are charged).
3. **Summary for T1/T2:** Acquisition on wide strips is already solved by Open's IET (`0.10–0.34 h₁`). Lazy thin-run contraction (`S*_1`) fails `NEEDS M-27` (`0.89–0.93 m₁₆`) and worsens every composed route via `B_P` fragmentation. The depth-2 `(v, v_R)` `R⁴` schedule wins on read counts only with an L1 pair-cursor table *and* a `3.52 MB` (`4n`) flattened word buffer, whose build-copy and L3-footprint penalty must be screened against the worker-split / Q-price exchange rate (#33) before any Rust build.
