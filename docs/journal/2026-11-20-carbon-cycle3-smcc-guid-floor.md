# Carbon Cycle 3 · Working Backwards from the Text, the Output-Copy Oracle Ceiling (`SMCC-Paid` & `SMCC-Oracle`), Recursive Band Cuts, and the UUID Format Refutation

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 3).*

## 0. Cycle-Boundary Check (`origin/master` Fetch)

Fetching `origin/master` (`ad48e6ae`) before completing Cycle 3 surfaced:
1. [`2026-11-20-working-backwards.md`](https://github.com/lmmx/mibz2/commit/2b2499d) (`costmap.py`): JSON keys (`33–38%` of bytes) own only `1–4%` of thin rows, whereas unique v4 statement GUIDs own `54–67%` of width-1 rows (`23–38%` of thin rows) and repeated 40-hex `sha1` reference hashes own `6–25%` of thin rows.
2. [`2026-11-20-toy-transfer.md`](https://github.com/lmmx/mibz2/commit/ad48e6a) & [`2026-11-20-research-by-obligation.md`](https://github.com/lmmx/mibz2/commit/555958b):
   - The Coordinator independently reran `g5_bvsmc.c`, reproducing byte-exactness, canary safety, `BVSMC-1P` `emit` at `0.61–0.77×` Open, and whole-kernel at `0.66–0.78×` on 5/6 blocks (`U16Q` at `0.77–0.92×` with `0 B` extra memory), with strict G5 remaining `FAIL` on the all-blocks `emit <= 0.62×` cutoff; `U16Q` and `BVSMC` move to Coordinator native Rust validation and are **not** tuned here.
   - Relayed two concrete questions tied to Cycle 3 (`working-backwards.md` §2 Q1–Q3 & `toy-transfer.md` line 100: *can shared v4 UUID format act as a surrogate for width?* and *what is the oracle ceiling and paid cost of copying repeated thin strings like `sha1` hashes from completed output?*), plus the foundational Alpöge / Toy-to-Hard Transfer provocation, which we take up at full depth in **Cycle 4** immediately following this Cycle 3 handoff.

---

## 1. Explore: Five Genuinely Distinct Formulations

Mode: **Reorientation** (`METHOD.md` §3a), directly targeting `OPEN-QUESTIONS.md` §1 & §4, `FRONTIER.md` Foundational Questions #2 & #5, and `working-backwards.md` §2 Q1–Q3:

| # | Candidate | Mathematical setting | Mechanism & information needed | Likely cost / correctness trap |
|---|---|---|---|---|
| 1 | **`Second-Cut-Bands`** | Recursive first-return / Meyer complement on $g^{-1}$ (`OPEN-QUESTIONS.md` §1; `FRONTIER.md` FQ #2) | Cut Open's retained map $g^{-1}$ a second time using `bands` (`B ≈ 2.4k–6.7k`) to chase $m_2 \ll m$ rows | `bands` live in `thin`-stream rank; `queues[a]` scatters them across $R_{\text{thin}} \approx 48\text{k}–126\text{k}$ pieces |
| 2 | **`UUID-Format-Surrogate`** | Fixed-template constrained coding (`toy-transfer.md` line 100; `working-backwards.md` Q1) | Exploit fixed v4 UUID syntax (`8-4-4-4-12` dashes, version `'4'`, variant `{8,9,a,b}`, 16-hex alphabet) to batch singleton GUID rows | Fixed bytes already coalesce into wider runs; 122 random bits per UUID force independent suffix-sorted bijections |
| 3 | **`Lower-Limit-Dedup`** | Width-$1$ singleton section (`LIMIT -> 1`) with prefix-deduplicated excursion words | Eliminate *all* repeated runs ($w \ge 2$, including all repeated `sha1` hashes) before `emit`, leaving only $m_1 = r_1$ | `endpoint.rs` linked-list node count ($3H_1$) and `282–460 KiB` deduplicated dictionary still grow `9–15×` |
| 4 | **`SMCC-Paid`** | Self-modifying functional graph with online same-cache-line affine-lockstep discovery (`OPEN-QUESTIONS.md` §4) | Use the exact packed identity `edges[z] == edges[z-1] + 1` ($2^{\text{bits}} > m$) to detect width-$\ge 2$ convoys in the *already-loaded* cache line of `edges[]`, rewriting sibling entries in place to jump $k \ge 6$ steps via `out[src_pos..]` | Online slack checks, `edges[]` dirtying, and random reads into `880 KiB` `out[]` may exceed skipped `edges[]` loads |
| 5 | **`SMCC-Oracle`** | Zero-discovery omniscient trajectory-copy ceiling (`working-backwards.md` Q2–Q3; `FRONTIER.md` FQ #5) | Offline uncharged oracle pre-wires *every* maximal $\ge 6$-step repeated code substring of the marked orbit in `edges[]` before `emit` starts | Random L2/L3 reads from `880 KiB` `out[0..pos)` + 3-way dispatch may negate `36–53%` skipped pointer steps |

---

## 2. Triage: Exact Structural Censuses & Refutations (`triage_cycle3.py`)

All counts below are reproduced deterministically by [`triage_cycle3.py`](2026-11-20-carbon-cycle3/triage_cycle3.py).

### Triage 2.1 · `Second-Cut-Bands` (`OPEN-QUESTIONS.md` §1): Sharpening No-Widening & Killing Band-Only Recursion (`proved` + `measured`)

1. **Correction & exact scope of "No-Widening" (`proved` + `measured`):**
   On original column coordinates $[0, n)$, thin runs have width $\le \text{LIMIT} = 16$. However, in *retained* coordinates $[0, m)$, deleting thick strips of symbols $b \neq a$ makes two thin runs of symbol $a$ adjacent in $L[S]$ without inserting any thick-strip gap into $F[a]$; hence $g^{-1}$ **can** form maximal affine pieces (`codes[z] == codes[z-1]` and `next_j[z] == next_j[z-1] + 1`, `mark` isolated) of width $> 16$ (`max_w = 24..33` across the 6 blocks!).
   - *Measured magnitude:* Only **`10` to `45`** pieces on an entire block exceed width `16`, saving `195–890` rows (`0.14%–0.72%` of $m$) — negligible.
2. **Why `O(B)` bands cannot build a second cut without `O(m)` work (`proved` + `measured`):**
   `self.bands` (`2,400–6,700` bands) is contiguous in *per-symbol thin-stream rank*, **not** in retained $L$-rank $[0, m)$. Mapping through `queues[a]` shatters the `2.4k–6.7k` bands into **`47,547`–`125,845` maximal affine pieces** of $g^{-1}$:

   | Block | $m$ (`LIMIT=16`) | Affine pieces of $g^{-1}$ | `max_w` | Cut $w > 16$: $(H_2, m_2)$ | Cut $w \ge 4$: $(H_2, m_2)$ | Cut $w \ge 2$: $(H_2, m_2)$ | Direct `Open<LIMIT=1>` $m_1$ |
   |---|---:|---:|---:|---:|---:|---:|---:|
   | `block-0.L` | 89,874 | 47,547 | 33 | `(30, 89,230)` | `(5,085, 51,670)` | `(12,608, 34,939)` | 35,116 |
   | `block-2.L` | 112,070 | 50,528 | 32 | `(26, 111,521)` | `(7,447, 55,553)` | `(16,425, 34,103)` | 34,262 |
   | `block-85.L` | 156,404 | 77,736 | 30 | `(13, 156,133)` | `(10,577, 83,697)` | `(22,344, 55,392)` | 55,504 |
   | `block-143.L` | 196,841 | 103,105 | 29 | `(45, 195,951)` | `(11,399, 115,555)` | `(30,795, 72,310)` | 72,578 |
   | `block-207.L` | 135,921 | 83,453 | 24 | `(10, 135,726)` | `(5,816, 95,542)` | `(21,313, 62,140)` | 62,407 |
   | `block-210.L` | 204,281 | 125,845 | 32 | `(32, 203,642)` | `(8,777, 140,492)` | `(28,935, 96,910)` | 97,054 |

   Even eliminating **every** piece of width $w \ge 2$ in $g^{-1}$ (`H_2 = 12.6k–30.8k` second-level strips, requiring an $O(m)$ scan to identify plus a second `endpoint.rs` pass larger than Level 1's!) lands at $m_2 = 34,103\text{--}96,910$, within `0.15%–0.50%` of simply running single-level `Open<LIMIT=1>`, while paying both constructions. **Killed.**

### Triage 2.2 · `UUID-Format-Surrogate` (`toy-transfer.md` line 100): Shared Format Does Not Compress Singleton Ordering (`proved` + `measured`)

**Proposition 2.1 (Format-Invariance of High-Entropy Singleton Permutations — `proved`).**
Let $\mathcal{T}_M$ be the family of $2^{122 M}$ blocks obtained by substituting $M$ independent 122-bit version-4 UUIDs into fixed slots of a known JSON template (fixing `-` at offsets `8, 13, 18, 23`, version `'4'` at `14`, and variant $\in \{\texttt{8,9,a,b}\}$ at `19`).
1. Because every assignment in $(\{0,1\}^{122})^M$ satisfies the template and UUID-v4 grammar identically, conditional entropy given the full template and format grammar remains $122 M$ bits; no format recogniser can infer any interior nibble or cross-boundary pairing without reading the column.
2. For every interior UUID offset $t$ followed by $k$ random hex nibbles with $16^k \gg M$ (all offsets $t \le 29$ at $M \sim 10^3$), the $M$ suffixes $\{(i, t)\}_{i=1}^M$ have pairwise distinct right-contexts w.h.p., so $\mathrm{LF}$ maps the $M$ rows at offset $t+1$ (sorted by $u_i[t+2\dots]$) to the $M$ rows at offset $t$ (sorted by $u_i[t+1\dots]$) as a 16-way radix shuffle whose inverse within each hex bucket is determined solely by the random nibbles. Fixed delimiters (`-`, `4`) already coalesce into wider runs where followed by shared prefixes, while adding singleton boundaries immediately before dashes where post-dash hex suffixes diverge. $\square$

- **Controlled test (`triage_cycle3.py`, $M = 600$ JSON statement records with identical 122-bit random payloads — `measured`):**
  - `Raw-122b` (unformatted 31-hex-char tokens): `w1_runs = 17,665` (`29.4 / item`), `Open<16>` `iso1 = 17,669` (`29.4 / item`).
  - `UUID-v4` (`8-4-4-4-12` formatted): `w1_runs = 19,586` (`32.6 / item`), `Open<16>` `iso1 = 19,587` (`32.6 / item`).
  Shared format adds **`+3.2` isolated singleton rows per UUID** rather than merging them. **Conjecture refuted.**

### Triage 2.3 · `Lower-Limit-Dedup` (`LIMIT = 16 -> 1`): Separating `node.word` Still Leaves `282–460 KiB` Dictionary + `9–15×` Strip Count (`measured`)

Factoring the final thin entry byte `a` out of `spelling` cuts dictionary size by `25–56%`, but at `LIMIT = 1` (`m_1 = 35.1k–97.1k`) deduplicated `node.word` bytes still balloon from `18.6–31.0 KiB` (`LIMIT = 16`, L1-resident) to **`281.7–460.2 KiB`** (spilling `i9-10900K`'s `256 KiB` L2!) while `H` jumps from `1.6k–5.0k` to `16k–32k`. **Deferred cleanly; Cycle 4 asks whether non-linked-list elimination can avoid `endpoint.rs` altogether.**

---

## 3. Deepen: In-Place Self-Modifying Convoy-Copy Open (`SMCC-Paid` & `SMCC-Oracle`)

### Mandatory `METHOD.md` §3a Candidate Declaration
- **Mode:** Reorientation (replacing repeated thin-run pointer-chasing in `Open::emit` — e.g. repeated 40-hex `sha1` reference hashes — with zero-table same-cache-line convoy discovery and in-place self-modifying `out`-copy shortcuts, alongside an uncharged omniscient oracle measuring the fundamental ceiling of output-sourced copying; classification deferred to the Coordinator and MONODROMY).
- **Displaced obligation:** Obligation #1 & `OPEN-QUESTIONS.md` §4 (`36–48%` `Open::emit` pointer chasing across repeated $2 \le w \le 16$ thin trajectories, without Astra C's `419–488 KiB` external anchor table).
- **Replacement principle:**
  - **Theorem 3.1 (One-Instruction Affine-Lockstep Identity — `proved`):** When `bits = bit_length(m)` so $2^{\text{bits}} > m$ strictly, for every $1 \le z < m$ with $z \neq \text{mark}$ and $z - 1 \neq \text{mark}$, `next(z - 1) + 1 <= m < 2^bits` never carries into `code`, so:
    $$\text{edges}[z] = \text{edges}[z - 1] + 1 \iff \text{code}(z) = \text{code}(z - 1) \;\land\; g^{-1}(z) = g^{-1}(z - 1) + 1.$$
  - **Theorem 3.2 (Exactness of Self-Modifying In-Place Shortcut Rewriting — `proved`):** Suppose from `start_j` the walk traces $k \ge k_{\min}$ steps writing `out[src_pos .. src_pos + byte_len)` to exit vertex `cur`, while every offset $\text{off} \in [-\text{cur\_l}, \text{cur\_r}]$ avoids `mark` and satisfies step-by-step lockstep with unmodified (`< COPY_BASE`) edges. Then every sibling `start_j + off` has exact $k$-step word equal to `out[src_pos .. src_pos + byte_len)` and exact $k$-step exit `cur + off`. Because `src_pos + byte_len <= pos` holds *before* `edges[start_j + off]` is rewritten to `((COPY_BASE + cid) << bits) | (cur + off)`, any subsequent visit to `start_j + off` copies a strictly completed, non-overlapping slice of `out` (even if the convoy was self-overlapping across overlapping intervals of $[0, m)$, guarded by `(edges[sib] >> bits) < COPY_BASE` and `cdc < COPY_BASE`) and lands at `cur + off`. $\square$
- **New paid obligations (`SMCC-Paid`):** `0 B` producer cost; `32 KiB` L1 `copy_dict`; per-thick-exit same-cache-line neighbor check `slack_around`, in-place RMW writes `edges[sib]`, 3-way `code` dispatch, and random reads from `out[src_pos..]`.
- **Nearest prior art:** Astra C (`2026-11-18-astra-c-output-first.md`) and Neon Cycle 5 Candidate 5A (`419–488 KiB` anchor tables, `+25–87%` demand reads). `SMCC-Paid` needs **zero** external discovery table by reading adjacent lanes of the already-fetched `edges[]` cache line.

### Gate G1 & Gate G2 — Verification & Acquisition Manifest (`verified`)
- **Exhaustive & authenticated verification ([`verify_cycle3.py`](2026-11-20-carbon-cycle3/verify_cycle3.py) — `verified`):**
  - **`10,454 / 10,454` PASS** across all binary ($1 \le n \le 8$) and ternary ($1 \le n \le 5$) `(L, p, limit)` instances with `min_k = 2` (`1,814` paid shortcuts created, `1,880` oracle hits);
  - **`6 / 6` PASS** on periodic, self-overlapping (`abacabadabacabae * 50`), and multi-cycle (`ba * 301`) controls;
  - **`5,399,692 / 5,399,692` bytes exact** on all 6 authenticated Wikidata blocks (`SMCC-Paid` skips **`19.4%–33.3%`** of `emit` steps using `517–1,435` shortcuts = `4–11 KiB`; `SMCC-Oracle` skips **`36.2%–53.0%`** of `emit` steps; isolated singleton rows `iso1` account for **`30.4%–47.4%`** of $m$).
- **Gate G2 ([`smcc.toml`](2026-11-20-carbon-cycle3/smcc.toml) — `verified`):** `check_manifest.py` outputs **`PASS`**.

---

## 4. Confront: Gate G5 Falsifier & Oracle Ceiling (`g5_smcc.c` — **FAIL**)

Pre-registered in [`g5_smcc.c`](2026-11-20-carbon-cycle3/g5_smcc.c) (25 paired interleaved rounds after 3 warmup rounds on `Xeon Platinum 8481C`, `gcc -O3 -march=native` — `measured`):
- `Op`: shipped `Open-qi16`;
- `Paid`: `SMCC-Paid` (`min_k = 6`, `gate_thick = 1`, `32 KiB` L1 `copy_dict`);
- `Ora`: **`SMCC-Oracle`** (offline uncharged greedy oracle pre-wiring *every* $\ge 6$-step repeated orbit substring into `edges[]` before `emit` starts — zero online discovery cost, zero runtime `edges[]` writes).

| Block | $m$ | `iso1 / m` | `Paid` skip | `Ora` skip | `OpEm` (ms) | `PaidEm` (ms) | **`OraEm` (ms)** | `Paid/Op` `emit` | **`Ora/Op` `emit`** | `OpTot` (ms) | `PaidTot` (ms) | **`OraTot` (ms)** | `Paid/Op` `tot` | **`Ora/Op` `tot`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | `38.9%` | `26.4%` | `45.5%` | 0.551 | 0.833 | **0.506** | `1.512×` | **`0.918×`** | 1.491 | 1.773 | **1.437** | `1.189×` | **`0.964×`** |
| `block-2.L` | 112,070 | `30.4%` | `33.3%` | `53.0%` | 0.732 | 0.984 | **0.589** | `1.346×` | **`0.805×`** | 1.855 | 2.104 | **1.727** | `1.134×` | **`0.931×`** |
| `block-85.L` | 156,404 | `35.4%` | `31.4%` | `50.9%` | 0.941 | 1.298 | **0.987** | `1.379×` | **`1.048×`** | 1.913 | 2.270 | **1.951** | `1.187×` | **`1.020×`** |
| `block-143.L` | 196,841 | `36.7%` | `26.6%` | `46.3%` | 1.559 | 2.056 | **1.439** | `1.319×` | **`0.923×`** | 3.404 | 3.901 | **3.282** | `1.146×` | **`0.964×`** |
| `block-207.L` | 135,921 | `45.7%` | `19.4%` | `36.2%` | 0.929 | 1.491 | **0.987** | `1.605×` | **`1.062×`** | 2.216 | 2.770 | **2.263** | `1.250×` | **`1.021×`** |
| `block-210.L` | 204,281 | `47.4%` | `19.7%` | `38.5%` | 1.462 | 2.053 | **1.512** | `1.405×` | **`1.035×`** | 2.899 | 3.499 | **2.948** | `1.207×` | **`1.017×`** |

**Pre-registered Verdict:** **FAIL** (`0/6` blocks cleared `PaidEm <= 0.90×` or `PaidTot <= 0.92×`; byte-exactness and `32 KiB` L1 bound passed).

---

## 5. Learn: The Output-Copy Oracle Obstruction (Closing `OPEN-QUESTIONS.md` §4 & `working-backwards.md` Q2)

The decisive scientific result of Cycle 3 is **not** merely that `SMCC-Paid`'s online discovery cost (`+0.25–0.56 ms`) outweighed its `19.4–33.3%` skipped pointer steps — it is what **`SMCC-Oracle`** revealed when **100% of discovery overhead was removed**:

1. **Even an omniscient zero-cost copy oracle fails Gate G4 (`0/6` blocks reach even an 8% whole-kernel speedup, and `3/6` blocks run *slower* than `Open::emit`!):**
   On `block-85` (`50.9%` of pointer steps skipped!), `block-207` (`36.2%` skipped), and `block-210` (`38.5%` skipped), `SMCC-Oracle` `emit` ran **`3.5%–6.2%` slower** than plain `Open::emit`, and across all 6 blocks `OraTot / OpTot = 0.931×–1.021×`.
2. **Why output-sourced copying loses even with free discovery (`Output-Copy Source-Footprint & Dispatch Obstruction`):**
   - **Witness:** `SMCC-Oracle` in `g5_smcc.c` (`3,970–6,041` pre-wired shortcuts skipping `49,202–79,648` pointer loads with zero online discovery, running `1.035×–1.062×` slower than `Open::emit` on blocks `85, 207, 210`).
   - **Mechanism & Scope:** Plain `Open::emit` reads words exclusively from a compact `32–58 KiB` L1/L2-resident `spelling` buffer and writes `out[0..n)` strictly sequentially (zero random reads from `out`). Any output-sourced copy mechanism (`Astra C`, `OPEN-QUESTIONS.md` §4, `SMCC`, `working-backwards.md` §2 Q2) replaces $k$-step sub-walks in `edges[0..m]` (`360–817 KiB`) with **random L2/L3 read misses across the `0..900 KB` `out` buffer**, a `copy_dict` lookup, and a 3-way branch (`literal` / `spelling` / `out-copy`) + short variable-length `memcpy`.
   - **Escape condition:** Eliminate repeated sequences **before** `out` exists — in a compact dictionary or algebraically composed edge representation — or attack the **producer + representation** directly without random reads from `out[0..n)`. Cycle 4 turns directly to the foundational Toy-to-Hard Mathematical Transfer to investigate what genuinely changes the primitive.
