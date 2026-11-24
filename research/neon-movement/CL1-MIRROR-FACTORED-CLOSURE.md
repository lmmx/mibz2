# CL1 — Mirror-Factored Bi-Directional $P \circ \mathrm{LF}_v$ Closure (Zero-Fragmentation Second Composition)

**Task:** Channel `#27` / `chatgpt-closure.md` §5 — the Closure Problem.  
**Script & Data:** [`cl1_mirror_factored_closure.py`](cl1_mirror_factored_closure.py), [`results_cl1_mirror_factored_closure.json`](results_cl1_mirror_factored_closure.json).

## 1. Exact Reproduction of `chatgpt-closure.md` & Diagnosis of Fragmentation

Under Open's `w > 16` thick-run threshold (`L_min = 17`), our genuine endpoint-only `RetainedSystem` reproduces `chatgpt-closure.md` on `block-0.L` **to the exact integer**:
- Retained rows: `m = 89,874` (`h_1 = 35,116`), original $R = P \circ \mathrm{LF}_v$ exact `+1`-segments: **`B_P = 2,148`** (`0.0612 h_1`).
- One standard M-17 Gauss–Seidel sweep (`for j in 0..m-1: compose j -> nxt[j]`):
  - `cap = 8`: `44,814` segments (`1.276 h_1`), `42,325` emit links;
  - `cap = 16`: `46,671` segments (`1.329 h_1`), `40,009` emit links;
  - `uncapped`: `52,319` segments (`1.490 h_1`, `24.4×` jump), `32,833` emit links, `2,507,119` flat word bytes.
- Across all 6 admitted Wikidata blocks, one uncapped M-17 Gauss–Seidel sweep fragments $B_P$ by **`17.9×–42.7×`** (`2,018 -> 86,199` on `block-85.L`; `3,564 -> 139,880` on `block-210.L`).

**Why M-17 fragments:** M-17 overwrites *every* row `j` in Sweep 1 (both descents $R[j] < j$ and ascents $R[j] > j$), mixing variable-length chains into `next1[j]` so that a second same-direction sweep must read `next1[next1[j]]` at addresses that no longer follow $P \circ \mathrm{LF}_v$.

## 2. Constructive Representation That Survives a Second Composition (`CL1`)

Keep the compact $B_P$-piece $P \circ \mathrm{LF}_v$ cursor schedule (`(target_start, seg_len)` per byte, only `16–43 KB` across the 6 blocks; verified exact under both forward and reverse cursor replay without storing `R[0..m)`) intact, and split $M = D_1 \sqcup A_1$ implicitly by the sign of $R[j] - j$:

1. **Sweep 1 (Ascending `j = 0 .. m - 1`, Descent Contraction on $D_1 = \{j : R[j] < j\}$):**
   - Advance the $|\Sigma|$ cursors of $P \circ \mathrm{LF}_v$ forward (`B_P` pieces).
   - For $j \in D_1$ with $y = R[j] \neq \mathrm{mark}$: read `T_1[y]` (already finalized since $y < j$) and set `next1[j] = next1[y]`, `wid1[j] = dag_concat(base_wid[j], wid1[y])`.
   - Leave $j \in A_1$ ($R[j] > j$) untouched (`next1[j] = R[j]`, `wid1[j] = base_wid[j]`).
2. **Sweep 2 (Descending `j = m - 1 .. 0`, Mirror Ascent-over-Descent Contraction on $A_1 = \{j : R[j] > j\}$):**
   - Advance the **exact same $|\Sigma|$ cursors of $P \circ \mathrm{LF}_v$ in reverse** (`B_P` pieces, **zero fragmentation**).
   - For $j \in A_1$ with $y = R[j] \neq \mathrm{mark}$: read `T_2[y]` (which already holds the full ascent+descent arch if $y \in A_1$, or the Sweep-1 descent chain `T_1[y]` if $y \in D_1$) and set `next2[j] = next2[y]`, `wid2[j] = dag_concat(base_wid[j], wid2[y])`.
3. **Word payloads (Zero-Copy Binary DAG):** Every composition allocates one 8-byte DAG node `(left_wid, right_wid)` with zero byte copying during either sweep, cutting uncapped word overhead from `2.5–3.0 MB` flat to `0.70–1.60 MB` (`8m` bytes).

## 3. Correctness (`[proved]`, 8/8 + Periodic + 200 Multi-Cycle Corrupt Columns Verified)

**Theorem CL1.1 (Exactness on All Columns, Including Periodic & Multi-Cycle Corrupt Columns — proved).**
1. In Sweep 1, every read edge $j \to y$ satisfies $y < j$, forming a strictly decreasing well-founded DAG on $[0, m)$; in Sweep 2, every read edge $j \to y$ satisfies $y > j$, forming a strictly increasing well-founded DAG. Neither sweep can ever cycle on *any* permutation (single-cycle, periodic, or corrupt multi-cycle).
2. Guarding $y \neq \mathrm{mark}$ guarantees no composition ever skips `mark`, so following `next2` from `mark` until returning to `mark` tiles the exact marked cycle of length $\ell$ (with every ascent+descent arch between consecutive local minima contracted into one jump), matching bzip2's `PROOF-PROG` contract (`decode[j] = decode[j mod ell]`) identically.
3. Because every read address in Sweep 1 ($j \in D_1$) and Sweep 2 ($j \in A_1$) is $y = R[j] = P(\mathrm{LF}_v(j))$, both sweeps access `T_1`/`T_2` strictly monotonically inside the $B_P$ pieces of $P \circ \mathrm{LF}_v$, and total successor reads across *both* sweeps combined equal $|D_1| + |A_1| = m - 1$ (the same read count as a *single* M-17 Gauss–Seidel sweep). $\square$

*Verification:* Verified `decoded == T_ref` on **8/8 blocks**, on periodic `b"abcd" * 25`, and on **200 random corrupt multi-cycle columns** (`174` with $\ell \nmid n$) — 100% exact.

## 4. Measurements Across Novel Random Hex & All Six Admitted Blocks (8/8 Byte-Exact)

| Block | `m` (`L_min`) | Orig `B_P` | M-17 GS `seg` (frag) / `links` (`!/h_1`) | **CL1 S1** `P`-pcs / `c16` | **CL1 S2** `P`-pcs / `c16` | **CL1 `links`** (`!/h_1`, vs M-17) | CL1 DAG vs Flat |
|---|---:|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 57,458 (`2`) | 10,094 | `57,377` (`5.7×`) / `20,920` (`0.364`) | `5,041` / `3,130` | `5,064` / **`3,169`** | **`18,695`** (`0.325`, `-10.6%`) | `449 KB` / `208 KB` |
| `synth-tmpl-hex-1000x40` | 47,001 (`17`) | 33 | `45,571` (`1,381×`) / `16,379` (`0.410`) | `24` / `18` | `23` / **`16`** | **`14,547`** (`0.364`, `-11.2%`) | `367 KB` / `316 KB` |
| `block-0.L` | 89,874 (`17`) | 2,148 | `52,319` (`24.4×`) / `32,833` (`0.935`) | `1,222` / `1,041` | `979` / **`753`** | **`29,295`** (`0.834`, **`-10.8%`**) | **`702 KB`** / `2,917 KB` |
| `block-2.L` | 112,070 (`17`) | 3,136 | `56,117` (`17.9×`) / `41,022` (`1.197`) | `1,789` / `1,503` | `1,408` / **`1,066`** | **`37,020`** (`1.081`, **`-9.8%`**) | **`876 KB`** / `2,805 KB` |
| `block-85.L` | 156,404 (`17`) | 2,018 | `86,199` (`42.7×`) / `56,212` (`1.013`) | `1,216` / `1,045` | `862` / **`650`** | **`50,096`** (`0.903`, **`-10.9%`**) | **`1,222 KB`** / `3,006 KB` |
| `block-143.L` | 196,841 (`17`) | 5,436 | `114,445` (`21.1×`) / `71,720` (`0.988`) | `3,020` / `2,512` | `2,492` / **`1,939`** | **`63,997`** (`0.882`, **`-10.8%`**) | **`1,538 KB`** / `2,767 KB` |
| `block-210.L` | 204,281 (`17`) | 3,564 | `139,880` (`39.2×`) / `73,953` (`0.762`) | `2,099` / `1,776` | `1,544` / **`1,198`** | **`65,263`** (`0.672`, **`-11.8%`**) | **`1,596 KB`** / `2,929 KB` |
| `block-207.L` | 135,921 (`17`) | 3,547 | `93,074` (`26.2×`) / `49,729` (`0.797`) | `1,963` / `1,565` | `1,663` / **`1,268`** | **`44,400`** (`0.712`, **`-10.7%`**) | **`1,062 KB`** / `2,835 KB` |

*(Capped comparison on `block-0.L`: `cap=8` gives `37,110` links / `578 KB` DAG vs M-17 `42,325` links / `1,034 KB` flat; `cap=16` gives `35,164` links / `609 KB` DAG vs M-17 `40,009` links / `1,070 KB` flat.)*

## 5. Referee Under $\Pi$

- **Successor schedule:** Sweep 1 touches `1,216–3,020` pieces (`1,041–2,512` cache-line `<= 16` segments); **Sweep 2 touches `862–2,492` pieces (`650–1,939` cache-line segments) — `44×–133×` less fragmented than M-17 GS**, with **zero non-$P \circ \mathrm{LF}_v$ reads** across both sweeps!
- **Read count & emit links:** Total build successor reads = $|D_1| + |A_1| = m$ (same as 1 M-17 sweep, streamed along the $|\Sigma|$ monotone $P \circ \mathrm{LF}_v$ cursors with `2,018–5,436` piece boundaries). Emit links fall to **`0.672–1.081 h_1`** (`0.325 h_1` on pure novel hex; **`10–12%` fewer than uncapped M-17 GS** on all 8/8 blocks!).
- **Passes & State:** `P = 2` sweeps over the `m`-row table (plus thick-strip acquisition `Q_acq_strips` from Open), `S` = `~17–43 KB` for the $B_P$ schedule + `8m` bytes for the zero-copy binary word DAG (`0.70–1.60 MB` vs `2.77–3.01 MB` flat) + `4m` for `next`.
- **What remains open (investigated next in `CL2`):** `CL1` solves the **second** composition with zero fragmentation; `CL2` tests whether a **third** composition can stay compact or hits an exact representation-independent obstruction.
