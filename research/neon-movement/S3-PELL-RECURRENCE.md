# Sense 3 (`pell-recurrence`): A Recurrence Among the Induction's Own Steps (Doomslide's Pell Rule)

## 1. Exact Definition of "Moves Together"

In doomslide's essay (Boat II), the Learner replaces a wall of triangles with a generating recurrence (`M_{j+1} = M_j^2` / Pell matrix squaring) that constructs new instances from earlier ones. Applied to BWT inversion, the induction's own steps obey the algebraic squaring recurrence `R_{j+1} = R_j \circ R_j`:

- **3A (Algebraic Matrix-Squaring / Pointer-Free Sort-Merge Doubling):** On a cycle `(M, R)` (either raw `LF` on `n` rows or `Open`'s thick-contracted retained map `R` on `m = m_{16}` rows), cut the cycle at the unique predecessor `t` of `mark` (`R(t) = \text{mark}`). Represent `R_j` as a stream of triples `(x, \mathrm{nxt}_j(x), \mathrm{dist}_j(x))`. Two transitions `(x \to y)` and `(y \to z)` **move together in join-key order** (`\mathrm{nxt}_j(x) = y`): a base-256 LSD radix sort (`256` sequential write cursors, `Q = 0` under `Π`) brings every `x` adjacent to its target `y = R_j(x)` in a single two-pointer linear merge (`Q = 0`), composing all `m` lone rows in parallel without ever knowing text order or relying on row adjacency.
- **3B (Zorich–Pell Self-Overlap Cycle Recurrence):** Thick runs `I_i = [s_i, s_i+w_i)` whose image `\mathrm{LF}(I_i)` overlaps `I_i` with shift `\delta = |\mathrm{LF}(s_i) - s_i|` generate `q = \lfloor w_i / \delta \rfloor` induction steps by one Euclidean/Pell rule `x \mapsto x + q\delta`.

## 2. Composition Rule and Exactness

- **Composition rule (3A):** At round `j = 0`, set `\mathrm{nxt}_0(t) = -1, \mathrm{dist}_0(t) = 0`, and for all `x \ne t`, `\mathrm{nxt}_0(x) = R(x), \mathrm{dist}_0(x) = 1`. At round `j \to j+1`:
  1. Partition active triples (`\mathrm{nxt}_j(x) \ne -1`) and base-256 radix-sort by `\mathrm{nxt}_j(x)` (`Q = 0`).
  2. Two-pointer merge against the `y`-sorted table of round `j`: replace `(x, y, d_x)` with `(x, \mathrm{nxt}_j(y), d_x + \mathrm{dist}_j(y))` (`Q = 0`).
  3. Base-256 radix-sort back by `x` (`Q = 0`).
- **Exactness — PROVED (and byte-exact verified on 8/8 blocks):** By induction on `j`, after `j` rounds every node `x` satisfies `\mathrm{dist}_j(x) = \min(2^j, \text{steps from } x \text{ to } t)` and `\mathrm{nxt}_j(x) = R^{\mathrm{dist}_j(x)}(x)` (or `-1` once `t` is reached). At `j = \lceil \log_2 m \rceil`, `\mathrm{nxt}_j(x) = -1` for every `x \in M` and `(m - 1) - \mathrm{dist}_j(x)` equals the **exact orbit position `pos[x] \in \{0,\dots,m-1\}`** of `x` starting from `mark`. One final base-256 radix sort of `(pos[x], \text{word\_id}[x])` by `pos[x]` orders all `m` transitions in exact backward text order, and one sequential scan emits `T` byte-for-byte. No `O(n)` word materialisation occurs during doubling (`12` bytes per triple regardless of `j`).
- **Hybrid checkpoints (`j = 1, 2, 3, 4`):** Stopping early yields `\lceil m / 2^j \rceil` macro-steps (byte-exact verified via recursive jump-tree emission), **with an honest accounting catch**: zero-copy jump-tree emission still addresses `m` leaves, whereas full `j_{\max}` list ranking achieves `Q_{\text{run,lone}} = 0`.

## 3. Prior Art

- Kitchen-sink Cycle 7 Theorem 7.2 tested IET interval-merge squaring `\mathrm{LF}^{2^j}` on ALL `n` rows (`r(\mathrm{LF}^{2^j})` saturated at `n` by `j=10`, costing `12.87\text{--}14.56 n` interval records).
- MONODROMY 108/109/112 analyzed `T^2` doubling and list ranking; Chiang–Goodrich–Grove–Tamassia–Vengroff–Vitter (1995) introduced external-memory sort-merge list ranking; kitchen-sink Cycle 12 Theorem 12.1 studied Zorich acceleration.
- **What was missed / completed here:** Running pointer-free radix-sort-merge doubling on the **`4.5\text{--}10\times` smaller retained map `R`** (`m_{16}`) without `O(n)` word materialisation slashes `P_{\mathrm{bytes}}` by `4.6\text{--}11.1\times` and `S` by `2.5\text{--}5.0\times` vs raw `LF`, while verifying **literal `Q = 0, D = 0`** on pure raw-`LF` novel random hex!

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s3_pell_recurrence.py`](s3_pell_recurrence.py); raw JSON: [`results_s3_pell_recurrence.json`](results_s3_pell_recurrence.json). All 8 blocks pass byte-exact verification.

| Block | `m_{16}` | Rounds (`m_{16}`) | Retained `Q_{\mathrm{doub}}` / `Q_{\mathrm{acq}}` / `Q_{\mathrm{run,dag}}` | Retained `D` / `P` / `P_{\mathrm{bytes}}` / `S` | Raw-`LF` `Q` / `D` / `P` / `P_{\mathrm{bytes}}` / `S` | 3B Zorich self-overlap runs (`w_1`) |
|---|---:|---:|---|---:|---:|---:|
| `synth-hex-iid-65536` | 65,536 | 16 | **0** / 65,536 / 0 | 0 / 117 / 1,518.6 / 2.36 MB | **0 / 0** / 117 / 1,525 / 1.57 MB (`EXACT`) | 0 (0) |
| `synth-tmpl-hex-1000x40` | 47,001 | 16 | **0** / 47,313 / 33,999 | 5 / 84 / 652.8 / 1.71 MB | **0 / 0** / 124 / 1,488 / 1.94 MB | 1 (0) |
| `block-0.L` | 88,290 | 17 | **0** / 210,406 / 811,641 | 10 / 124 / **156.8** / 4.35 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 0 (0) |
| `block-2.L` | 109,542 | 17 | **0** / 233,151 / 790,458 | 10 / 124 / **195.9** / 5.13 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 0 (0) |
| `block-85.L` | 155,124 | 18 | **0** / 202,918 / 744,811 | 9 / 131 / **289.5** / 6.21 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 0 (0) |
| `block-143.L` | 193,449 | 18 | **0** / 329,825 / 706,420 | 11 / 131 / **364.4** / 8.48 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 1 (0) |
| `block-210.L` | 202,025 | 18 | **0** / 272,108 / 697,934 | 10 / 131 / **381.2** / 8.22 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 0 (0) |
| `block-207.L` | 134,609 | 18 | **0** / 240,191 / 765,389 | 11 / 131 / **249.6** / 5.89 MB | **0 / 0** / 145 / 1,740 / 21.60 MB | 1 (0) |

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **Contraction on novel random hex:** **TOTAL.** Pure raw-`LF` Pell doubling contracts EVERY lone row regardless of randomness, achieving **literal `Q = 0, D = 0`** with `s = 256 \le s_{\max}` cursors (byte-exact verified on `synth-hex-iid-65536`).
- **Where the cost goes:** Entirely into **passes `P`** (`117\text{--}145` sweeps `\gg K = 2`) and **`O(m \log m)` streamed volume** (`P_{\mathrm{bytes}} = 156.8\text{--}381.2` on retained `m_{16}`, `1{,}740` on raw `LF`), plus `S = 4.35\text{--}8.48\text{ MB}` (`21.6\text{ MB}` raw). Note also that expanding binary concat DAG nodes for thick strips adds `Q_{\mathrm{run,dag}} \approx 700\text{--}812\text{k}` unless thick strips are stored flat (`O(r)` unique thick bytes).
- **Zorich–Pell audit (3B):** `0\text{--}1` self-overlapping runs across all 6 blocks and `0` lone rows affected.
- **Verdict:** Pell doubling proves that **`Q` and `D` can BOTH be driven literally to `0` on 100% novel random hex** within `s = 256` cursors — cleanly relocating the entire floor into `P = \Theta(\log m) \gg K = 2`. Sense 4 (`independent-set`) immediately attacks Sense 3's `O(m \log m)` streamed-volume blowup by shrinking it to `O(m)`!
