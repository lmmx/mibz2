# P74 option C: the worst case of an online switch (read-only analysis)

Option C runs P74 from the block's start and hands off to the shipped kernel inside the
block (`2026-10-30-p74-selection-architecture.md`). This note bounds what that can lose.
Nothing was implemented, built or benchmarked. P74 Gate 1 stays closed and negative, and
nothing here reopens it. The objective is mibz2's own MTF stage and decoder on
representative inputs (Wikidata, and random and records as the negative controls).

Labels: **[M]** measured (file cited), **[D]** deduced from measured facts or definitions,
**[E]** estimate or model (assumptions listed). Scripts and outputs are in this directory.

## 0. Result

- **No provable bound that keeps the gain.** A one-way rule with a bound valid for every
  stream must charge high-rank tokens at an upper bound and credit low-rank tokens at a
  lower bound. From the asm:
  - the upper bound exists only conditionally, on published penalty cycles and IPC ≥ 1;
  - the lower bound on the low-rank saving can't be shown to be positive (α_min = 0).
  
  With α = 0 the rule switches every full Wikidata block within its first 0.1% of
  tokens, keeping **0.01–0.06% of the gain** (§7, kill test 2). The formal-bound version
  is **uneconomic**.
- **An empirical policy dominates the others offline:** the event-triggered budget on
  symbols consumed (§6.4).
  - **The test:** F + 1 > S/32 + 123, made only inside P74's existing ≥ 32 fallback branch.
  - **Cost:** zero new fast-path instructions, by construction of where the test sits.
    Whether the compiler keeps it that way is a disassembly obligation.
  - **Offline, under the fitted cost model:** Wikidata keeps 99.96% of its modelled gain;
    random and records ≤ +0.02%; text keeps 94%; the adversarial suite's sup is +0.23%.
  - **Under the conditional per-token bound,** the same suite's sup is +182%. So its
    safety is **empirical**, on the named corpora and suite, and **not worst-case**.
- **Rejected:**
  - **The one-time prefix:** its exposure equals the unselected worst case for every p (§4).
  - **Frequency-ratio windows:** they are evaded by staying just below the threshold.
  - **The output-cursor (n) denominator, including the suggested τn + b form:** it is
    beaten by run inflation, at **+19% even under the fitted model** (§8).
  - **Any rule that needs E or E_low exactly:** that needs a per-token, per-digit or
    per-run operation, killed by report 65's rule (§5).

## 1. Inputs (measured)

- **The symbol streams** come from each file's own Huffman stream (`bz2syms.py`, pure
  Python, no BWT). They were checked against libbzip2: the full inverse of 3 records
  blocks and 2 text blocks equals `RLE1(bz2.decompress(file))` byte for byte (`check.py`).
- **The Wikidata file is CI's corpus itself** (SHA-256 82f65ace…, 11,854,933 B → 209,706,557 B
  of L, 431 blocks, of which 198 are tiny at under 8 symbols, 8.9 KB in all). Its L total
  equals Gate 1's `of_bytes`.
- **Random, records and text are `ab gen` at scale 1** (3, 3 and 4 blocks). CI uses scale 4,
  from the same generator.

| per output byte [M] (`features.json`) | E | E_low (q < 32) | F (q ≥ 32) | lower tail chunks | RLE2 runs | runs > 32 B | digits | S | F/n | F/S |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| Wikidata | 0.0722 | 0.0711 | 0.00110 | 0.0002 | 0.0205 | 0.0020 | 0.0469 | 0.1191 | 1/911 | 1/108 |
| random | 0.9922 | 0.2404 | 0.7518 | 0.9446 | 0.0078 | 0 | 0.0078 | 0.9999 | 0.752 | 0.752 |
| records | 0.9968 | 0.0947 | 0.9022 | 2.676 | 0.0032 | 0 | 0.0032 | 1.0000 | 0.902 | 0.902 |
| text | 0.164 | 0.155 | 0.0088 | 0.002 | 0.150 | 0.003 | 0.243 | 0.407 | 1/114 | 1/46 |

The timings come from Gate 1's per-round arm figures: `gate1-artifact/` (Zen 4, EPYC 9V74,
2875.7 MHz reported) and `gate1-selector/` (Zen 5, EPYC 9V45, 4548 MHz). Each is the
median over 10 rounds, in ps per output byte, converted to cycles at the reported clock.

## 2. Per-class cost estimate (Q1) [E]

**The model:** Δ = t_P74free − t_shipped, in c/B = −g·E_low + d_F·F + d_c·(lower chunks).
That is fitted to the 3 corpora for each kernel and machine (`fit.py` → `fit.json`).

| fit | Δ Wikidata, random, records (c/B) [M] | M2 (−g·E_low + d_F·F): g, d_F, residual Wikidata | M3 (+ chunk term, exact): g, d_F, d_c |
|---|---|---|---|
| Zen 4 plain | −0.694, +2.363, +5.51 | 12.8, 7.4, +0.21 (30% of Δ) | 9.8, 5.7, 0.50 |
| Zen 4 checkpoints | −0.771, +2.423, +5.77 | 13.8, 7.8, +0.20 (26%) | 10.9, 6.1, 0.48 |
| Zen 5 plain | −0.387, +3.831, +6.09 | 7.2, 7.5, +0.12 (30%) | 5.5, 6.5, 0.27 |
| Zen 5 checkpoints | −0.372, +5.514, +7.65 | 5.3, 9.0, −0.002 | 5.4, 9.1, 0.00 |

- **The fit is underdetermined.** There are 3 corpora for 3–5 effects. Rank cost, branch
  prediction, run emission and inter-token dependencies are confounded:
  - a per-run term instead of the chunk term gives g_short_run from −17 to +0.2 c;
  - the shipped kernel's absolute fit gives a negative per-token cost (−3.6 c).

  M3 has zero residual by construction, and M2 misses Wikidata by 26–30% on three of
  the four fits. The timing noise is small beside this: per-round SD of Δ 0.007–0.02 c/B
  on Wikidata, though records on Zen 4 has bimodal rounds (SD 12–15 c/B) and is taken by
  median.
- **Reading, as an estimate only:**
  - a low-rank token saves about 5–14 c;
  - a fallback token costs about 6–9 c more, plus up to 0.5 c per extra tail chunk;
  - the break-even fallback share g/(g + d_F) is about 37% on Zen 5 and 64% on Zen 4. The
    frozen 10% is far inside both.
- **The cross-check against §3.2e:**
  - Zen 4's g of 10–14 c matches the model's 10–12 c saving per token (chain 15–17 → 5–6);
  - Zen 5's 5–7 c is smaller, consistent with its shipped kernel being faster (1.94 c/B
    against 2.32);
  - d_F of 6–9 c is consistent with ΔI = 13 + 12c instructions at about 4 IPC (§3).
- **These are not bounds.** No worst case below uses a fitted value as a guarantee.

## 3. Definitions and the instruction path [M from asm, D]

- **The regret:** R(x) = (T_π(x) − T_T(x)) / T_T(x), where T_T is the shipped kernel on
  stream x, and T_π covers P74 until the switch, monitoring, the handoff H and the
  shipped suffix.
  - **The oracle regret:** R_or uses min(T_P74, T_T) instead.
  - **Safety means sup_X R ≤ 0.03.** Claimed only where proved.
- **The asm read:** `phases-sel.asm`, the selector build e57400e, the measured binary.
  The functions are extracted in `asm/`. Counts are instructions; cmp+jcc pairs fuse.

| path, per explicit token | P74 free, plain | P74 free, checkpoints | shipped `mtf_avx2` |
|---|---|---|---|
| low rank (q < 32), no run | 36 instr, 6 cond. branches | 36, 6 (n lives in `[rsp+0x18]`: a memory RMW per token) | 39, 6 |
| fallback, c = ⌊(q−32)/32⌋ + 1 tail chunks | 53 (c=1), 72 (c=2), +6 per chunk to c=4, 4-unrolled beyond (≈ 93 at c=7) | 55 + 12c (one uniform chunk loop) | 42 + `memmove(q)` (≈ 8–25 inside glibc, by size class) |
| run ≤ 32 B | 3 (broadcast store) | 3 | ≈ 7 + `memset` call |
| run > 32 B | spill V + `memset` + reload V and 3 constants | the same | `memset` + reload constants |

- **What the asm does bound,** per token: instruction and µop counts; conditional
  branches, so the most mispredicts; loads that overlap an earlier store at another
  alignment, so the most failed forwards; spills; loop-carried structure.
- **What it doesn't:** the cycles per mispredict or failed forward; mispredict rates
  (predictor state, aliasing); whether a failed forward lands on the critical path;
  glibc's memmove/memset internals; frequency; i-cache and BTB state at the handoff;
  SMT contention.
- **The conditional per-token excess** (premises P1–P4):
  - **P1:** shared branches behave identically, so low-rank excess ≤ 0. Both kernels
    branch on q < 32 on the same data, and P74 has 3 fewer instructions and no forwarding
    stall;
  - **P2:** ≤ c + 1 extra mispredicts at π ≤ 20 c, and ≤ c + 1 failed forwards at
    φ ≤ 15 c (published ranges, not from the asm);
  - **P3:** IPC ≥ 1 on the extra instructions;
  - **P4:** H ≤ 100 c.

  Then u(c) = ΔI + (c + 1)(π + φ) = **48 + 47c**: 95 c at c = 1, 377 c at c = 7. A long run
  costs ≤ 12 c. That is the "cond" hypothesis below. It is not a proof either: P1–P4 are
  microarchitectural premises.

## 4. The one-time prefix (Q2): unbounded relative exposure [D]

- **The setup:** a block of n tokens whose first p favour P74 and whose remainder is
  worst for P74, with per-token excess ρ·t_T (ρ is the remainder's relative excess). The
  prefix test passes, so P74 runs throughout.
- **The regret:** R = (n − p)·ρ·t_T / (p·t_T + (n − p)·t_T) = ρ(1 − p/n) → ρ as n/p → ∞.
- **For every fixed p,** sup R = ρ_max, the same as never selecting. The prefix caps
  nothing; its absolute loss grows linearly in the remainder.
- **The reverse case** (prefix says shipped, remainder P74-favourable) loses only
  opportunity, up to the remainder's whole P74 gain.

| R of the worst remainder | p = 1k | 4k | 16k |
|---|--:|--:|--:|
| 900k-token block, factor (1 − p/n) | 0.999 | 0.996 | 0.982 |
| -1 block (100k tokens) | 0.990 | 0.959 | 0.836 |
| × ρ = +20.7% (records, Zen 5 checkpoints: measured, 90% fallback) | ≈ +20.7% / +20.5% | +20.6% / +19.9% | +20.3% / +17.3% |
| × ρ = +45% (fit d_F at an assumed shipped 20 c/token) [E] | ≈ +45% | ≈ +45% / +43% | +44% / +38% |
| × ρ = 18.9 (cond bound, c = 7) [E] | unbounded for practical purposes | | |

- **As a share of the decoder** [D, different harnesses]:
  - MTF + RLE2 is 17.4% (Zen 4, 2.23 of 12.80 c/B) and 21.5% (Zen 5, 1.97 of 9.17) of a
    one-thread Wikidata decode;
  - on high-rank data it is about **50–54%**: random 5.51 of 11.1 ns/B, records 6.44 of
    11.9 (`gate1-selector/comment.md`, 1-thread wall);
  - so the worst remainder costs about **+10–11% of the decoder** at ρ = 20.7%.
- **The reverse case's opportunity loss** is at most 19.2% (Zen 5) or 33.2% (Zen 4) of
  MTF, 4.1% or 5.8% of the decoder.

## 5. Denominators and counters (Q3) [M from asm, D]

**The identity:** E_t = t − N_RUNA(t) − N_RUNB(t) − N_EOB(t); n_t = E_t + RB_t (run bytes);
S_total = `syms.len()`.

| quantity | held by the kernel? (asm) | cost to maintain |
|---|---|---|
| t = S (symbols consumed) | **yes**: the iterator, `r14` (checkpoints) or `rbx` (plain), minus `syms.ptr` | 0 per token; 3 instructions at a test |
| n (output cursor) | **yes**: `r13`/`rdi` (plain), `[rsp+0x18]` (checkpoints) | 0 |
| N_RUNA + N_RUNB | **no**: only `weight` = 2^k of the *current* run, reset to 1 at each flush | +1 µop per digit (0.65/token, in a 7-instruction loop), or `tzcnt` + `add` per flush (0.57/token, a memory RMW in the checkpoint kernel) |
| N_EOB | 0 before the end | — |
| E, E_low = E − F | **no** | +1 µop per explicit token (2.8% of the 36-instruction path; a memory RMW chain if spilled), or the N_RUN route above |
| F | no (new) | +1 in the fallback branch only (memory RMW, cold path) |

- **The checkpoint loop's progress counter** (`n`, `next`) can't stand in for E:
  n − E = RB, which is not held, and `counts` sums to n.
- **Every exact E or E_low therefore costs a per-symbol or per-run operation on a hot
  path.** A policy needing α·E_low is **killed** (report 65's rule).
- **The statistics,** as thresholds relating to the frozen 10·F ≤ E:
  - **E:** F/E ≤ 1/10 (the frozen rule);
  - **S:** F/S ≤ τ_S. That equals F/E ≤ τ_S·(S/E), with S/E = 1.65 Wikidata, 2.48 text,
    1.008 random, 1.003 records. Free, but an exact-E rule only on run-free data;
  - **n:** F/n ≤ τ_n. That equals F/E ≤ τ_n·(n/E), with n/E = 13.9 Wikidata and 6.1 text.
    Free, but runs inflate it (§8).

## 6. One-way policies (Q3, Q4)

All switch P74 → shipped, irreversibly, at a clean boundary (§9). Reverse switching is
dropped: the plain kernel has no free periodic test point, low-rank opportunity
accumulates without events to trigger a test, it would need hysteresis, and §3 of the
architecture note found no regime change in any measured block.

### 6.1 One-time prefix p (needs a per-token count of E: killed by §5)

Its exposure is §4's. Offline (Zen 5 checkpoints fit; `policy.json`):

| | p = 1k | 4k | 16k |
|---|--:|--:|--:|
| Wikidata gain kept | 83.3% | 99.99% | 99.99% |
| random/records corpus R | +0.03% | +0.14% | +0.54% |
| -1 high-rank block R | +0.5% | +1.9% | +7.4% |

### 6.2 Checkpoint window (free: the outer loop's 16 KiB boundary)

- **The rule:** switch at a boundary if 10·F_w > S_w.
- **The boundary is free:** the checkpoint kernel's outer iteration runs once per 16 KiB
  of output, so the test adds nothing to the inner loop.
- **Detection delay:** one window of output, up to 16k high-rank tokens.
- **Offline:** Wikidata keeps 98.5% (7 blocks switch early); random +0.54% (corpus),
  +1.8% (a 200k block); a -1 high-rank block +7.4% [E].
- **Evaded by any stream at 9% of S per window:** the number of disadvantageous tokens is
  unbounded, up to 0.1·S a block (§8).

### 6.3 Weighted debt D̂ = Σ_{q≥32} w(q) + Σ_{long runs} w_R − α·t, tested on its events

- **Deduction (A), verified with corrections.** If every high-rank w(q) ≥ the true
  excess, every long-run w_R ≥ its excess, and every other symbol saves ≥ α, then
  D̂_t ≥ D_t for every prefix of every stream (a sum of per-token inequalities).
  - The rule "switch when D̂ > B", tested at each event, then gives
    R ≤ (B + w_max + H + M)/T_T(x).
  - The one-event overshoot w_max must be added, unless the test runs before the token.
  - w must include the test's own instructions, and H the shipped suffix's cold start.
  - **Evasion buys real savings only if α is a true lower bound** on every context's
    saving. §3 can't show α > 0, so a conservative rule has α = 0.
- **Model error accumulates.** For any estimated weights d̂, |D_n − D̂_n| ≤ Σ|d_i − d̂_i|,
  which is O(n·ε), with d_i dependent on preceding ranks and predictor state. Weights from
  the fit therefore guarantee nothing for arbitrary streams.
- **With α credited on t (free):** charge long runs w_R + α·k (k = digits, `tzcnt(weight)`
  in the cold long-run branch). It is then conservative only if every digit of a short run
  and every low token saves ≥ α. That is P1 plus "short runs save", both premises.
- **Its approximation error against the fit** [E]:
  - a constant w per fallback errs by d_c(c − 1)/d_F: 0 (Zen 5 checkpoints), 25%
    (Zen 5 plain) and 47% (Zen 4) at c = 7;
  - crediting per symbol instead of per low token over-credits by S/E_low: 1.68×
    Wikidata, 2.6× text, 4.2× random, 10.6× records, against the Wikidata calibration.

### 6.4 Event-triggered budget (added by the coordinator), and the S variant

- **The rule:** inside the fallback branch, after the run flush and before the list
  update, switch if F + 1 > τ·x + b; otherwise F += 1. Here x = n (output cursor) or S.
- **The invariant (proof, verified):**
  - F changes only at fallback events, and x is non-decreasing between them;
  - so the predicate F + 1 > τx + b can only turn true at an event;
  - it is tested before that token's update, and the token goes to the shipped kernel.

  So there is no monitoring delay and no overshoot (W = 1, d̂⁺ = 0).
- **The bound (verified):** before the switch, F ≤ τx + b at every event. With
  d_high ≤ u and all other excess ≤ v per unit of x:
  
  excess ≤ u(τx_s + b) + v·x_s + H + M,
  
  so R ≤ (uτ + v)⁺/c_min + (u·b + H + M)/(c_min·x), where c_min is the shipped kernel's
  cost per unit of x.
  - **v's sign is not provable per context.** P1 makes the low-rank path v ≤ 0, but long
    runs give v > 0 (≤ 12 c per run, ≤ 0.36 c/B).
  - **Per byte,** c_min over arbitrary streams is about 1/32 c/B (one long memset), so
    the n form's bound is vacuous.
  - **Per symbol,** c_min ≥ 1 c formally (the iterator's 1-cycle add chain). The modelled
    floor is about 6.2 c (a token and a 7-digit run per 8 symbols).
- **b_min(τ)** = max over Wikidata blocks and fallback events of F_t − τx_t, from all 431
  blocks of CI's corpus [M] (`policy.json`). b is set to ⌈1.25·b_min⌉ + 8.

| τ | b_min, x = n | b | random / records switch at (≈ b/(ρ − τ)), bytes | text kept | b_min, x = S | b | text kept |
|---|--:|--:|---|--:|--:|--:|--:|
| 1/256 | 206.5 | 267 | 356 / 297 | 11% | 1798 | 2256 | 33% |
| 1/128 | 154.2 | 201 | 270 / 225 | 20% | 1031 | 1297 | 22% |
| 1/64 | 106.0 | 141 | 191 / 158 | 95% | 147.3 | 193 | 9% |
| 1/32 | 64.1 | 89 | 124 / 102 | 95% | 91.5 | 123 | **94%** |

- **On Wikidata,** every τ in the table keeps 99.96% of the modelled gain. The rest is the
  198 tiny blocks sent to the shipped kernel by the guard; no full block switches.
- **On random and records,** the corpus R is ≤ +0.02% for every row [E].
- **The short-block guard:** dispatch to the shipped kernel when `syms.len()` < S₀.
  - n ≥ S − 1 for a valid block: every token gives one byte, a k-digit run gives
    ≥ 2^k − 1 ≥ k bytes, and the EOB gives none.
  - The fixed term needs S ≥ (u·b + H)/(0.03·c_S): about 8.5k symbols with the fit
    (u ≈ 12 c, c_S ≈ 6.2), and about 250k under the cond bound.
  - **S₀ = 16,384** is chosen. No measured block lies in 4,096–16,384.
- **Feasibility** of uτ + v + (u·b_min + H + M)/x_min ≤ 0.03·c_min:

| | x = n | x = S |
|---|---|---|
| cond bound (u ≥ 95) | infeasible for any τ that keeps Wikidata (it needs τ ≤ 1/3,200 against Wikidata's F/n = 1/911) | infeasible (τ ≤ 1/1,000 against Wikidata's F/S = 1/108) |
| fit (u ≈ 12, v = 0) on arbitrary streams | infeasible (c_min ≈ 0.03–0.75 c/B) | τ ≤ 1/65 only, and then only for S ≥ 170k at τ = 1/128; τ = 1/32, which text needs, fails |
| fit, crediting the measured low-rank saving (v < 0) | — | holds on the corpora and suite, **empirically** |

### 6.5 Zero-scan dispatch σ = S/N_max (added) [M traces, D]

- **n_true isn't knowable before MTF.** The bz2 block header holds the CRC, the
  randomised bit and origPtr, but no length. `Symbols` carries `max_len` (from the level)
  and `syms.len()` = S. n needs a scan of the runs.
- **So σ uses N_max = level × 100k:** 900k for every block here (`additions.py` →
  `additions.json`).

| corpus | full blocks: σ range | short final blocks: σ (n, frozen rule's choice) |
|---|---|---|
| Wikidata | 233 blocks, 0.078–0.222 | 1 at 0.0009 (1,185 B, frozen: shipped), plus 197 with n ≤ 1,000 |
| random | 2 blocks, 0.99991–0.99992 | 1 at 0.222 (200,039 B, frozen: shipped) |
| records | 2 blocks, 0.99997 | 1 at 0.667 (600,040 B, frozen: shipped) |
| text | 3 blocks, 0.399–0.402 | 1 at 0.054 (80,689 B, frozen: P74) |

- **The θ that keeps Wikidata and text and rejects every full control block:**
  0.402 < θ < 0.9999. θ = 0.5 is set on this data.
- **False selections at θ = 0.5, against the frozen whole-block rule:**
  - Wikidata: 15 blocks, 3,567 B (0.002%). These are the frozen rule's tiny shipped
    blocks; σ sends them to P74, and the S₀ guard sends them back.
  - random: 1 block, its short final 200k (2.5% of the corpus's bytes), to P74; the online
    guard switches it after about 170 B.
  - records: 0 at θ = 0.5. The short final 600k block goes wrong for θ > 0.667.
  - text: 0 for θ ≥ 0.45; 2–3 blocks for θ ≤ 0.40.
- **What σ adds:** nothing a full high-rank block wouldn't get from the guard, except that
  it skips the guard's first 60–170 B of P74 (about +0.01%) at zero cost. It is a block
  density statistic only:
  - it can't catch high-rank tokens hidden among runs (the S-adversary's σ ≈ 0.11);
  - short final blocks have small σ at any density.

  **The guard stays mandatory.**

### 6.6 Chunk-weighted guard K_t + k(q) > τx + b (added) [M traces, E]

- **k(q) = 1 + ⌊(q − 32)/32⌋, verified in the source.** `moved!`'s fallback runs
  `for c in (0..=j / 32).rev()` with j = q − 32. That is j/32 + 1 iterations of the
  checkpoint kernel's uniform 12-instruction chunk loop (§3).
- **The same monotonicity proof holds:** K is constant between high-rank events.

| guard (b = ⌈1.25·b_min⌉ + 8) | τ | b_min | random / records switch at, median bytes | text kept | Wikidata kept |
|---|--:|--:|---|--:|--:|
| F over S (§6.4) | 1/32 | 91.5 | 170 / 142 | 97.6% | 100% |
| K over S | 1/32 | 155.5 | 66 / 63 | 26.6% | 100% |
| K over S | 1/64 | 211.2 | 94 / 80 | 15.6% | 100% |
| K over n | 1/32 | 101.1 | 43 / 42 | 97.7% | 100% |
| K over n | 1/64 | 169.4 | 68 / 67 | 97.6% | 100% |
| F over n | 1/64 | 106.0 | 189 / 160 | 97.6% | 100% |

- **The conditional comparison:** per unit, u_K = max_c u(c)/c = 95 c (at c = 1),
  against the F guard's u_F = max_c u(c) = 377 c (at c = 7). So K tightens the conditional
  worst case by 4× on wide tails. It doesn't change feasibility (§6.4): it still needs
  u_K·τ ≤ 0.03·c_min.
- **Under the fit,** the chunk term is 0–0.5 c against d_F = 6–9 c, so K's weighting is
  mostly noise there.
- **The trade-off:**
  - K over S loses text, since text's K/S of 1/38 is above 1/32;
  - K over n keeps text but has n's run-inflation weakness (§8).

  **The candidate stays F over S, τ = 1/32.** K over S at τ ≥ 1/16 is an untested
  alternative.

## 7. Kill tests [M asm, E model]

1. **Conservative debt with no extra operations on the low-rank path.**
   - With α = 0: **yes**. F, the test and the switch sit in the fallback branch only;
     long-run charges go in the memset branch.
   - With α > 0: credit on t or n is free but not conservative (§5, §6.3). Credit on
     E_low needs a hot-path operation, which is **killed**.
2. **Wikidata under the conservative rule** (α = 0, w = 48 + 47c, w_R = 12; `killtest2.py`).
   The B that satisfies a bound valid for all streams is B ≤ 0.03·T_lb − w_max − H, with
   T_lb ≥ S cycles. That is about 2.1k c on a median Wikidata block.

| B (cycles) | switch position, median (max) share of the block's tokens | share of P74-eligible bytes decoded in P74 | gain kept, Zen 5 / Zen 4 fits |
|--:|---|--:|--:|
| 477 | 0.02% (0.10%) | 0.01% | 0.01% / 0.02% |
| 2,000 | 0.12% (0.33%) | 0.03% | 0.06% / 0.08% |
| 20,000 (T_lb = 6.2 c/symbol, modelled) | 8.7% (36%) | 12.5% | 9.7% / 9.8% |
| 75,000 (needs T_T ≥ 2.5M c: true of controls' full blocks, not of -1 blocks) | 92% (100%) | 58% | 85% / 85% |
| 200,000 | 100% | 98.3% | 99.4% / 99.4% |

   **Test 2 fails** at every B a valid bound allows. The formal-bound version is
   **uneconomic**, and only the empirical policy is drafted (§12).

## 8. Adversarial suite (Q2, Q3) [E: generated streams, model costs]

**Validity:** every generated stream is a valid `Symbols` stream (`policy.py`'s `valid`):
- ranks 1 ≤ q ≤ A − 1 with eob = A + 1, A from 1 to 256;
- runs as bijective RUNA/RUNB digits, with weight < 2^21 and ≤ 21 digits;
- 1 ≤ n ≤ 900,000 (level 9; -1 sizes as noted), and one EOB at the end.

For a stage test, each stream must also be wrapped as a .bz2 (Huffman tables from the
stream, CRC from the reference inverse), accepted by libbzip2 (Python `bz2`) and by
mibz2's scalar path. Specified here, not built.

**The generators:**
- all-high (q 224–255, and q 32–63);
- a low prefix of 1k, 4k, 16k or 64k tokens (Wikidata-sampled, clipped below 32), then all-high;
- alternating 1–15 and 224–255 phases of 256, 1k, 4k and 16k tokens;
- just below each trigger:
  - 9% fallbacks;
  - the debt kept at B − 1;
  - runs of 1/τ − 1 bytes per fallback (n form);
  - fallbacks with low tokens and 63-byte runs (S form);
- short blocks of 2 to 99,999 high tokens;
- one 800k-byte run then high tokens (n inflation);
- long runs (1, 32, 33, 64, 1000, 16,385, 2^18 B) around switch points;
- alphabet sweeps at A ∈ {2, 16, 17, 32, 33, 34, 48, 64, 128, 200, 256}.

**The costs:**
- **T_T [assumption]:** 20 c per explicit token, 1 c per digit, 20 c per run (a memset
  call) and 1/32 c per long-run byte;
- **fit:** Zen 5 checkpoints M3;
- **cond:** §3's bound.

Full table in `synthetic_table.txt`; the sup over the suite:

| policy | sup R, fit [E] | its worst stream | sup R, cond [E] |
|---|--:|---|--:|
| P74 always | +45% | all-high | +1,885% |
| prefix 4k | +40% | low prefix 4k, then high | +1,731% |
| checkpoint window | +45% | short block 1k (no boundary reached) | +1,885% |
| debt, empirical weights (w = 18 + (c − 1), α = 1.6/symbol, B = 4,096) | +1.1% | short block 8k | +127% |
| debt, conditional bound (α = 0) | +0.02% | — | +0.4% (uneconomic: §7) |
| event n, τ = 1/64 | **+18.9%** | 63-byte runs, F/n = 1/64 | +811% |
| event n, τ = 1/32 | **+20.1%** | 31-byte runs | +838% |
| event S, τ = 1/64 | +0.31% | 63-byte runs | +104% |
| **event S, τ = 1/32 (the candidate)** | **+0.23%** | 63-byte runs | **+182%** |

- **The n form is broken empirically.** Under the fitted costs, a run of 1/τ − 1 bytes
  before each fallback keeps F/n at τ. Each group then carries about 9 c of excess
  against about 48 c of shipped time.
- **The S form is not, under the fit.** Staying under F ≤ S/32 + b forces at least 3 low
  tokens per fallback (a run adds ≤ about 6 digits per 64 bytes), and under the fit
  their savings exceed the fallback's excess.
- **Under the cond bound,** low tokens save nothing and the same stream costs +182%. So
  the S form's safety rests on the low-rank saving holding per context.
- **The total disadvantageous tokens before the handoff,** an exact sup per policy:
  - event: F ≤ τx + b, with ≥ 1/τ − 1 non-fallback units per fallback;
  - debt: Σw ≤ B + α·t;
  - window: ≤ 0.1·S_w a window, never triggering;
  - prefix: unbounded beyond p.

## 9. The handoff: state, proof, obligations, tests

**The state:**
- σ_P = (H, T, it, run, weight, n, out[0..n), counts, next, checkpoints, result);
- σ_T = (L[0..256), the same rest);
- φ(σ_P) = σ_T with L = H‖T. At the start, H = 0..31 and T = 32..255, so φ(σ⁰_P) = σ⁰_T.

**The boundary:** the fallback branch's entry. There:
- the run has been flushed (run = 0, weight = 1);
- `s == eob` was false, and n < limit (the `n >= limit` test has just passed);
- the token s has been consumed from `it` but not applied.

**The proof:**
- **The claim:** Step_T(φ(σ), s) = φ(Step_P(σ, s)) for every symbol s.
  - **List:** H′‖T′ = MTF(H‖T, q) on every path (§3.2d–e; emulated exhaustively; Gate 1's
    `tests/mtf_p74.rs`).
  - **Emitted byte:** P74's is s2u[H′[0]] and shipped's s2u[L′[0]], the same.
  - **Run flush:** it uses H[0], which is L[0].
  - **Everything else:** n, out, counts, next, checkpoints and the error checks are
    textually the same statements. P74's checkpoint loop is the shipped loop with the list
    operations replaced (read from `wt-p74/src/mtf.rs` against master's).
  - **P74's 32-byte run store** writes past n, inside capacity max + 32. Each position
    below the final n is rewritten at its own time, and `set_len` covers only those.
- **By induction:** Step_T^(m−k) ∘ φ ∘ Step_P^k = φ ∘ Step_P^m = Step_T^m ∘ φ. So a
  switch after any k symbols at a clean boundary gives the shipped kernel's result:
  `ll`, counts, checkpoints and errors.

**The engineering obligations:**
- **(a) H and the layout** (shipped side as verified at origin/master `src/mtf.rs`
  lines 86–145).
  - **Shipped:** the list is at `store[32..288]` of a 64-aligned 320-byte `L`. The < 32
    path does an aligned load at base and `loadu(base − 1)`; the ≥ 32 path does
    `ptr::copy(base, base + 1, nn)`, nn ≤ 255.
  - **P74 (correction):** `Tail([u8; 32 + 224])` does have 32 bytes of slack before its
    tail base (`add(32)`), used by chunk 0's `loadu(p − 1)`. The tail is offset by 32
    from where the shipped list would want it, so the switch as is costs a 32-byte head
    store plus a 224-byte copy (7 load/store pairs, about 16 instructions).
  - **The proposed layout,** one 64-aligned 320-byte buffer: slack 0..32, head slot 32..64,
    P74 tail 64..288. Checked:
    - P74's tail base is buf + 64, so element j is at buf[64 + j], j ≤ 223 → buf[287].
    - Chunk stores land at buf + 64 + 32c, which is 32-aligned (`_mm256_store_si256`).
    - The highest chunk, c = 6, spans buf[256..288].
    - Chunk 0's `loadu(base − 1)` reads buf[63], the head slot's stale byte 31, into
      T[0], which `*base = last` then overwrites. Harmless.
    - After the one ymm store of H to buf[32..64], base = buf + 32 (32-aligned):
      - the list buf[32..288] is H‖T;
      - `loadu(base − 1)` reads the slack byte buf[31], blended only into i = 0, which
        `*base = v` overwrites;
      - `ptr::copy` reaches at most buf[287].
  - **H** is one store. The shipped loop's cold i-cache and BTB at entry is not bounded
    by the asm; it is held in H.
- **(a′) The shipped kernel's resume entry.** Both shipped kernels keep the list as a
  local in their own frame (`store`, addressed `[rsp+0x60]` in the asm), with n, run,
  weight, the iterator (and next) as locals. Two ways to restructure:
  1. **A resumable `run_from(state)`,** used by both the fresh entry and the switch. State:
     the list pointer, `syms` cursor, n, run = 0, weight = 1, and (checkpoints) next. counts
     and checkpoints are already `&mut` parameters.
     - The cost: the list moves from rsp-relative addressing to a pointer register. The
       shipped plain loop already reloads 3–4 stack slots per token, so this risks a new
       spill on the **production** fast path, in both kernels.
  2. **Recommended: one combined function per kernel** (plain, checkpoints), with P74's
     loop followed by a copy of the shipped loop over the same frame-local buffer and
     locals. The switch jumps from P74's fallback branch into the second loop, after
     applying the pending token with the shipped `byte!` step.
     - The standalone shipped kernels stay untouched for σ- and S₀-dispatch.
     - The cost: about 2× the code size of each P74 kernel, and register allocation over
       both loops, which risks a spill in either.
     - The suffix's speed is measurable directly: force the switch at token 0 and
       compare with the standalone shipped kernel (a row in §12). That replaces the
       premise "no penalty in the shipped suffix" with a measurement.
- **(b) The monitor's cost.** F (a stack slot), x (S = iterator − `syms.ptr`; n is
  `r13` or `[rsp+0x18]`), `shr 5`, `add b`, `cmp` and `jcc` are about 7–8 instructions
  and one predictable branch, in the fallback branch only.
  - **Wikidata:** about 0.12 instructions per explicit token on average.
  - **Fast path:** zero new µops or branches.
  - **Loop-carried:** only F's memory RMW across consecutive fallbacks, about 5 c, beside
    a fallback's own ≥ 20 c.
  - **The shipped side:** after the switch it runs unmodified, with no monitoring.
  - **Risk:** the new exit edge and live F can change register allocation. A fast-path
    spill or a new fast-path instruction, read by diffing the fast-path basic blocks, is a
    defect.
- **(c) Resuming mid-interval: yes, after a source refactor and without waiting.**
  - The shipped checkpoint kernel's outer loop recomputes `limit = min(max, next)` at
    each iteration, and needs only n < limit, run = 0 and coherent next/checkpoints.
  - All of these hold at the boundary. So a resumable entry, taking (list, it, n, next,
    counts, checkpoints) and applying the pending token with `byte!(s)`, is exact.
  - **If the switch instead waits for the next 16 KiB boundary:** the excess is up to
    u × 16k tokens: about +0.65% on a full random block and +5.9% on a -1 block (fit).
    That fails 3%.

**The test plan** (each switch forced by a hook, against the shipped kernel and the scalar
reference, comparing `ll`, counts, checkpoints and the result or error):
- **Exhaustive small inputs:** every valid `Symbols` stream with A ∈ {1…6, 31…34} and up to
  8 symbols (digits included), at **every** fallback event. CHECKPOINT is made a
  test-only parameter (for example 4 B), so splits and boundary-ending runs occur.
- **Sampled positions on full blocks:**
  - every Wikidata, random, records and text block, and every suite stream;
  - the first and last fallback, 64 random fallbacks, and the fallbacks just before and
    after a checkpoint split;
  - long runs adjacent;
  - streams that raise `RunTooLong` or `BlockTooLong` after the switch.

## 10. Proved and empirical

- **Proved,** as definitions and algebra:
  - the prefix's unbounded exposure (§4);
  - the event invariant, with no delay or overshoot (§6.4);
  - D̂ ≥ D under per-token bounds (§6.3);
  - handoff equivalence at the boundary (§9);
  - n ≥ S − 1.
- **Conditionally bounded (P1–P4) but uneconomic:** the α = 0 debt. It keeps ≤ 0.1% of
  Wikidata's gain at any B a valid bound admits.
- **Empirical only:** the event-S policy's ≤ +3% on random, records and the suite. It
  rests on the fitted low-rank saving holding per context. That saving is measured on
  Wikidata and text, Zen 4 and Zen 5 only.

## 11. The assumptions whose failure would reverse it

- **The low-rank saving g > 0 in every context.** On a CPU where P74's 5-cycle chain or
  extra branch costs as much as the shipped forward stall, near-threshold S-streams
  become losses (+182% under cond).
- **d_F ≈ 6–9 c on all A.** It was fitted at A = 256 only; c up to 7 chunks could weigh
  more elsewhere (d_c 0–0.5 c, unstable).
- **The shipped kernel's cost per symbol** (≥ about 6 c modelled) sets the fixed-term
  floor. A shipped kernel made faster (for example its own inline short runs) shrinks
  c_min and P74's margin.
- **Codegen:** no new fast-path spill once F and the exit edge exist (§9b). Layout effects
  of 1.4–3.1% were measured on unchanged code, against the required gain of more than
  twice that.
- **b is fitted in-sample** on CI's own Wikidata corpus. Another dump's blocks could have
  earlier fallback bursts (b_min is set by block starts) and switch early. That loses only
  gain.
- **The cold shipped loop at the handoff** (H ≤ 100 c assumed).

## 12. Draft pre-registration (drafted, not proposed or authorised)

**The policy, frozen from offline data only** (this note's traces; no CI timing is used to
tune it):
- P74 free from the block's start, with these rules:
  - `syms.len()` < 16,384, or σ = `syms.len()`/`max_len` ≥ 0.5 → the shipped kernel
    from the start;
  - in P74's ≥ 32 fallback branch, after the run flush and before the list update: if
    F + 1 > (S ≫ 5) + 123, where S is symbols consumed before this token, switch one-way
    to the shipped kernel, which runs this token and the rest of the block;
  - otherwise F += 1;
  - A ≤ 32 needs no special case (no fallback exists).
- The head-slot layout of §9a, and the combined functions of §9a′ (option 2) for the
  plain and checkpoint kernels.

**The arms** are shipped, P74 free (Gate 1's code) and P74-switch, in one binary
(`phases`), rotated, with every block's output checked every pass.

**Disassembly, before measuring:** the P74-switch fast-path basic blocks match P74 free's
(no new µop, branch or spill), and the monitor sits in the fallback branch only. One fix
is allowed, then stop.

**The rows,** over 10 paired rounds × 2 passes, as 95% t-intervals:
- Wikidata, checkpoint kernel: ≥ 5% faster than shipped, with the interval excluding
  zero, and the gain more than twice the layout control's |1 − ratio|;
- random and records, plain and checkpoints: upper bound ≤ +3%;
- the adversarial suite (§8 streams as .bz2, correctness mandatory on every pass):
  - every stream with S ≥ 16,384: upper bound ≤ +3% against shipped;
  - every guard-dispatched stream: ≤ +1%;
  - any output mismatch: fail;
- the shipped suffix: P74-switch forced to switch at token 0, against the standalone
  shipped kernel, within ± twice the layout noise on every corpus;
- the layout control: the head's unchanged `MTF + RLE2` over the base's, in the same run;
- recorded: the CPU model, cores and SMT topology, how the clock was measured (the
  reported MHz and how it was read), the kernel and the rustc version.

**Stop conditions:**
- any correctness failure;
- a fast-path defect that persists after the one fix;
- any control or suite row over its limit gives **negative**, and option C closes;
- no tuning of τ, b or S₀ against the suite or CI after results.

**What success qualifies for:** a separately authorised decoder-level test only:
- the full decoder at 1, 2 and 4 workers, each measured separately;
- CPU-s/GB no worse;
- peak RSS within +1%;
- random and records decoder figures within the registered limits.

### 12a. Amendments (report 73; still a draft, not authorised)

- **σ as a hard bypass.** For σ ≥ θ the *untouched standalone* shipped kernel runs, not the
  combined function. Full random and records blocks then pay only the dispatch, and never
  touch the resumable code.
- **Three same-binary arms:**
  - T₀: the original shipped kernel;
  - T₁: the combined kernel forced to switch at token 0;
  - T₂: the live policy.

  T₁ − T₀ is the refactor and layout tax. T₂ − T₁ is the net P74 benefit, monitoring
  included. If T₁ − T₀ is material, fix the cold handoff structure before reading T₂. The
  shipped suffix stays out of the hot loop's instruction footprint, with no spills, checked
  in the disassembly.
- **Out-of-sample validation.** b = 123 was fitted on the exact Wikidata bench corpus, so
  re-running that corpus measures performance, not selection robustness. τ, b, S₀ and θ
  are frozen now. Acceptance needs:
  - independent Wikidata blocks (a slice of the dump disjoint from the bench corpus, its
    source named before the run);
  - independently seeded mixed-distribution controls.

  No retuning.
- **Correctness before economics:**
  - forced switches at every legal transition on small blocks, validated against shipped
    for counts, output, checkpoints and errors;
  - malformed streams;
  - the adversarial suite built as real .bz2 inputs and decoded by mibz2 before any of its
    timings are trusted.
- **Shipping inequality:** with g the measured P74 stage saving and f the retained
  fraction, f·g > C_monitor + C_handoff + C_layout, with confidence intervals on complete
  decoder time, plus the independent control, CPU-s/GB and RSS gates.
- **P74-H** (report 72) is killed (`2026-10-30-p74-h-fallback.md`): its predicted saving is
  about 0 ns per event on Zen 5, against a hurdle of 1.27–1.37 ns, because P74's excess is
  a fixed ~7–9 cycles per fallback. This draft stands.

## 13. Limits, and what wasn't resolved

- **No timing was taken.** Every R is model-based. The fit has 3 points per kernel and
  machine, from two runners.
- **The controls are scale 1 locally;** CI uses scale 4, from the same generator.
- The shipped checkpoint kernel's fallback path wasn't counted separately; it was taken to
  be the plain kernel's, the same `byte!` macro.
- glibc's memmove and memset are counted by size class, not disassembled.
- The suite's streams were not encoded as .bz2 or decoded by mibz2.
- `traces.npz` (24 MB) was deleted; `traces.py` regenerates it in about 20 s.

**Files** (all in this directory):
- `bz2syms.py`, `check.py`;
- `traces.py` → `features.json`;
- `fit.py` → `fit.json`;
- `policy.py` → `policy.json`, `summarise.py` → `synthetic_table.txt`;
- `killtest2.py` → `killtest2.json`;
- `additions.py` → `additions.json` (σ, the chunk guard);
- `asm/` (extracted functions).
