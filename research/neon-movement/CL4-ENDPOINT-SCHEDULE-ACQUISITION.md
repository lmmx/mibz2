# CL4 — Zero-Per-Row-Glue Endpoint-Only Acquisition of `CL1`'s $B_P$ Schedule

- **Task:** Third requirement of `chatgpt-closure.md` §5 & Channel `#27` (*"its construction must not require following the transitions it replaces"*), plus the positive answer to Channel `#27`'s sharp enrichment question (*"which later operation is necessary only because an earlier stage failed to retain information it already met?"*).
- **Script & Data:** [`cl4_endpoint_schedule_acquisition.py`](cl4_endpoint_schedule_acquisition.py), [`results_cl4_endpoint_schedule_acquisition.json`](results_cl4_endpoint_schedule_acquisition.json).
- **Verification:** Byte-exact against [`common.reference_decode`](common.py) on all **8/8 blocks** (pure novel hex, templated 40-hex, and all 6 admitted Wikidata blocks), running `CL1` Sweep 1 + Sweep 2 directly from the endpoint-built schedule with **`R[0..m)` and `R_word_id[0..m)` never allocated**.

---

## 1. Exact Construction (Retaining `src/runs.rs` Interval Structure)

In [`common.build_retained_system_genuine`](common.py), wiring `R[0..m)` and `R_word_id[0..m)` after thick-strip contraction required `Q_acq_glue = m` per-row lookups (`89,874–204,281` probes on Wikidata) and `8m` bytes (`702–1,596 KB`) to store `(R, R_word_id)`. That per-row glue pass was necessary **only** because the earlier stage (`src/runs.rs`) discarded the contiguous run intervals of retained rows after classifying thick vs thin runs.

[`build_p_schedule_endpoint_only`](cl4_endpoint_schedule_acquisition.py#L43-L350) retains those interval endpoints directly:

1. **Single streaming pass over `IN` (`r` runs, $Q = 0$, $s = 256$ cursors):**
   - Collect maximal contiguous retained intervals `M_intervals` (`1,353–4,115` intervals on Wikidata) with prefix retained-rank offsets `m_rank_off` (and locate `mark_rank` in $O(|\text{M\_intervals}|)$).
   - Collect per-byte sorted LF-image intervals `ret_lf_intervals[b]` covering all $\mathrm{LF}(x)$ of retained rows with $L[x] = b$.
   - Record the retained column bytes `v_ret` (`m` bytes, `88–199 KB`) and initial thick strips ($w \ge L_{\min}$).
2. **Endpoint-only Rauzy–Veech thick-strip contraction (`Q_acq_strips`):**
   - Contract thick strips purely by endpoint binary searches until all `n_strips_final` (`12,278–31,975`) strips land wholly inside `M`.
   - Map each final strip's target `t` to its exact retained rank `t_rank` via 1 lookup into `M_intervals` (`Q_strip_to_rank = n_strips_final`; `M_intervals` is `11–33 KB`, wholly L1-resident).
3. **Linear 2-pointer endpoint merge ($Q = 0$):**
   - Merge sorted `M_intervals` and sorted `final_strips` into a single partition `P_partition` of $[0, n)$ representing $P$ piece-by-piece (`(dom_s, dom_e, target_rank_start, strip_wid_or_-1)`).
   - Because `ret_lf_intervals[b]` lies strictly inside $I_b = [C[b], C[b]+\mathrm{count}[b])$, **one single linear sweep** of `P_partition` across $b = 0, \dots, 255$ emits the entire `(target_rank_start, seg_len, full_wid)` schedule of $P \circ \mathrm{LF}_v$ in only `2,446–6,788` merge steps (`n_endpoint_merges`)!
4. **Direct `CL1` Mirror Two-Sweep from the Schedule (Zero `R[0..m)` Array):**
   - Sweep 1 (ascending $j = 0 \dots m-1$, $D_1$) replays `sched[c]` forward with 256 cursors;
   - Sweep 2 (descending $j = m-1 \dots 0$, $A_1$) replays the **exact same `sched[c]`** backward with 256 cursors.

---

## 2. Correctness (`[proved]`, 8/8 Byte-Exact)

1. Because `M_intervals` are **maximal** contiguous intervals of $M$, every gap between distinct intervals of `M_intervals` is occupied by a thick strip; hence every finished strip $[t, t+w) \subseteq M$ lies wholly inside a single interval of `M_intervals`, so `t_rank + off` is exact for every row in the strip.
2. For each byte $b$, `LF` maps retained rows with $L[x] = b$ in strictly increasing order onto the union of `ret_lf_intervals[b]`, and `P_partition` partitions $[0, n)$ into affine $+1$ pieces of $P$ with constant strip word `swid`. Therefore their intersection yields the exact $(P \circ \mathrm{LF}_v)(j)$ target rank and base word `dag_concat(b, swid)` in exact retained-row order for every $j \in [0, m)$.
3. Replaying `sched` forward on $D_1$ and backward on $A_1$ produces the identical `(next_arr, wid_arr)` transitions as `CL1` (Theorem CL1.1), verified byte-exact on all 8/8 blocks. $\square$

---

## 3. Measurements Across All 8 Blocks (8/8 Byte-Exact)

| Block (`h_1`, `L_min`) | `m` | `M_iv` / `strips` | Merge steps | **`B_P_sched` (`12B` KB)** | **`Q_glue`** (saved!) | `Q_strips + Q_rank` (vs old `Q_strips + m`) | `CL1` emit links (`/ h_1`) | Total DAG KB (`8B`, incl. strips) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` (`57,458`, `2`) | `57,458` | `3,660` / `4,398` | `10,252` | `10,094` (`118.3 KB`) | **`0`** (`-57,458`) | `9,872` vs `62,932` (`6.4×` less) | `18,695` (`0.325`) | `464.7 KB` |
| `synth-tmpl-hex-1000x40` (`39,909`, `17`) | `47,001` | `16` / `49` | `38` | `34` (`0.4 KB`) | **`0`** (`-47,001`) | `361` vs `47,313` (`131×` less) | `14,547` (`0.364`) | `370.5 KB` |
| **`block-0.L`** (`35,116`, `17`) | `89,874` | `2,241` / `27,383` | `3,204` | **`2,180` (`25.5 KB`)** | **`0`** (`-89,874`) | `142,917` vs `205,408` (`-30.4%`) | `29,295` (`0.834`) | `1,247.5 KB` |
| **`block-2.L`** (`34,262`, `17`) | `112,070` | `2,815` / `24,126` | `4,244` | **`3,188` (`37.4 KB`)** | **`0`** (`-112,070`) | `133,965` vs `221,909` (`-39.6%`) | `37,020` (`1.081`) | `1,385.0 KB` |
| **`block-85.L`** (`55,504`, `17`) | `156,404` | `1,353` / `12,278` | `2,446` | **`2,039` (`23.9 KB`)** | **`0`** (`-156,404`) | `56,496` vs `200,622` (`-71.8%`) | `50,096` (`0.903`) | `1,438.8 KB` |
| **`block-143.L`** (`72,578`, `17`) | `196,841` | `4,115` / `31,975` | `6,788` | **`5,552` (`65.1 KB`)** | **`0`** (`-196,841`) | `163,701` vs `328,567` (`-50.2%`) | `63,997` (`0.882`) | `2,157.7 KB` |
| **`block-210.L`** (`97,054`, `17`) | `204,281` | `2,337` / `18,865` | `4,219` | **`3,597` (`42.2 KB`)** | **`0`** (`-204,281`) | `85,302` vs `270,718` (`-68.5%`) | `65,263` (`0.672`) | `1,920.5 KB` |
| **`block-207.L`** (`62,407`, `17`) | `135,921` | `2,871` / `21,323` | `4,596` | **`3,606` (`42.3 KB`)** | **`0`** (`-135,921`) | `124,059` vs `238,657` (`-48.0%`) | `44,400` (`0.712`) | `1,534.1 KB` |

*(Note: `B_P_sched` carries `(target_rank_start, seg_len, full_wid)` — `12 B/piece` — so pieces split when either `target_rank` jumps OR `full_wid` changes; this adds only `1.0–2.1%` over target-only `B_P = 2,018–5,436`.)*

---

## 4. Full End-to-End Referee Verdict (`CL1 + CL4`) Under $\Pi$

1. **Satisfies all three requirements of `chatgpt-closure.md` §5 at depth 2:**
   - **Compact across the second composition:** `2,039–5,552` schedule pieces (`23.9–65.1 KB`), replayed forward in Sweep 1 (`1,041–2,512` cache16 segments) and backward in Sweep 2 (`650–1,939` cache16 segments — **`44×–133×` less fragmented than M-17 GS**).
   - **Cheap word payloads:** Zero-copy 8-byte binary DAG (`0.70–1.60 MB` added in sweeps; `1.25–2.16 MB` total including all thick-strip nodes, vs `2.77–3.01 MB` flat words).
   - **Built without following the transitions it replaces:** Pure endpoint interval merges (`2,446–6,788` merge steps, `Q_acq_glue = 0`, saving `89.9k–204.3k` per-row lookups and cutting total acquisition probes by **`30–72%`** across the 6 Wikidata blocks, `6.4×` on novel hex, `131×` on templated hex), **never allocating `R[0..m)` or `R_word_id[0..m)`**.
2. **Honest `(Q, D, P, S)` accounting:**
   - **Acquisition:** `Q_acq = Q_acq_strips + Q_strip_to_rank = 56.5k–163.7k` (`0.88–4.07 h_1`, `0.063–0.182 n`) + `1` streaming pass over `IN` (`r` runs, `Q = 0`).
   - **Composition sweeps (`CL1`):** `P = 2` sweeps over `m`, reading `|D_1| + |A_1| = m - 1` targets strictly monotonically along the `256` cursors of the `23.9–65.1 KB` schedule (`Q_sweep = m - 1` in strict random-access count, or `1,691–4,451` cache-line jumps across both sweeps combined).
   - **Emit walk:** `Q_emit = 29.3k–65.3k` links (**`0.672–1.081 h_1`**, `10–12%` fewer than uncapped M-17 GS; `0.325 h_1` on novel hex) + sequential DAG expansion of `n` bytes.
   - **Boundary:** Depth 3+ exact-`+1` monotone closure is proved impossible on $M_2$ by `CL2` Theorem CL2.1 ($q_{\min}(R_2) = 0.19–0.25\,|M_2|$). Thus `CL1 + CL4` is the exact, zero-fragmentation, zero-per-row-glue endpoint-acquired depth-2 closure of Open's $P \circ \mathrm{LF}_v$.
