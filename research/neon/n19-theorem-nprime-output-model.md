# N-19: Attack on MONODROMY's Theorem N′ Output Model (`ASK N`, Retargeted at Record 107)

*Cycle 4 response to Coordinator Note #8 & MONODROMY's `ASK N` (`origin/claude/sharp-meitner-cx0v78` `aea0149`, `ad4f555`, `66a0538`): exhibit a decoder in `Λ′`'s neighbourhood that writes some byte by neither a link nor a rigid-placed fact, or show why none exists.*

**Script:** [`research/neon/scripts/verify_n19_nprime_attack.py`](scripts/verify_n19_nprime_attack.py) (byte-exact `T1` verification of all three constructive decoder families + 6-block quantitative ceiling for histogram completion).

## 0. Pre-declared decision rule & book (METHOD.md §6a)

- **Decision rule (fixed before execution in `CLAIMS.md` `86eb9b8`):**
  1. Audit Theorem N′ (`ad4f555` §2) strictly **inside `Λ′`**: prove whether any decoder over `Λ′` can write a byte by neither a link nor a rigid-placed fact (or exhibit a bug in Record 107 §2's proof).
  2. Probe **`Λ′`'s immediate neighbourhood**: exhibit minimal, byte-exact (`T1`) valid single-cycle decoders extending `Λ′` by one natural non-rigid write rule that write width-1 bytes by neither a link nor a rigid-placed fact and violate `c₁ ≥ h₁ − |B|` (at `|B| = 0`), **and** quantify whether any such extension can approach `δ = 1/2` on the 6 admitted blocks of `𝓕`.
- **Input features relied on:** Exact valid single-cycle columns `(L, p)` verified by `compute_lf` + `orbit_maps`, plus `h₁` byte frequencies across the 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).

## 1. Strictly inside `Λ′`: no counterexample exists (`[proved]`)

Recall Record 107 §2's definition of `Λ′`:
> Bytes are written **only** by rigid facts of `𝓡` (Record 103 §1). To these add positional labels `pos(x) = p` on **start rows** of rigid facts: `P₁` (single-row, charged at `x`) and `P_𝒪` (multi-row from `ι(L, o)`, uncharged). No non-rigid fact carries a word; class facts (E1) are outside `Λ′`.

**Proposition 19.1 (`[proved]`):** Strictly inside `Λ′` (under the tacit soundness condition that placing a rigid fact `f = (I, h, t, w)` via `pos(x) = p` with `x ∈ I` writes `w` only at `x`'s orbit positions `p, p + 1, …, p + h - 1 mod n`), **no decoder exists that writes a byte by neither a link nor a rigid-placed fact**, and Theorem N′ (`c₁ ≥ h₁ − |B|`) holds without exception.

*Proof.* By syntactic restriction of `Λ′`, every output byte `T[j]` (whose unique emitting row on the single `n`-cycle is `x`) must be written by some rigid fact `f` whose path contains `x`:
- If `x` is interior to `f` (`x = LF^k(y)` with `0 < k < h_f`, `y ∈ I_f`), Lemma A's corollary (Record 103 §2) applies validly because `f ∈ 𝓡` is rigid, forcing `|I_f| = 1` and locating a charged `compose` node joined at `x`.
- If `x` is `f`'s start row (`x = y`), `f` must be invoked either by an output chain link at `x` (charged at `x`), by a `P₁` label at `x` (charged at `x`), or by a `P_𝒪` label at `x` (`x ∈ B`).
Because every charged event has a single attribution row, `c₁ ≥ h₁ − |B|`. ∎

*(Subtle specification note for Record 107 §2 `[argued]`):* The proof relies on `f`'s word `w` being written **at `pos(y)` for `y ∈ I_f`** (not copied to an unrelated output offset `p' ≠ pos(y)`!). If `Λ′` permitted reusing a rigid fact's word `w` at an arbitrary offset supplied by a copy pointer, that collapses immediately into Mechanism 2 below.

## 2. In `Λ′`'s immediate neighbourhood: three exact (`T1`) non-rigid write decoders (`[proved]` + `[measured]`)

Adding **any one** of three natural, deterministic, `T1`-exact output operations to `Λ′` yields concrete valid single-cycle decoders that write width-1 bytes by **neither a link nor a rigid-placed fact** and strictly beat `c₁ ≥ h₁ − |B|` with **`|B| = 0`**!

### Mechanism 1 — `Λ′ + fill_remaining(c*)` (Histogram-Completion / Omission Write) `[proved]`

**Operation:** Maintain the count of emitted symbols against the global histogram `H` (already free in `𝒪` for `C[]`). Whenever all `k` remaining unwritten slots in `out[0..n)` must carry a single remaining symbol `c*` (`H[c*] - written[c*] == k` and `H[c] - written[c] == 0` for all `c ≠ c*`), write `c*` into all `k` unwritten slots. No link, rigid fact, `P₁`, or `P_𝒪` touches those `k` rows!

- **Minimal witness (`n = 3, k = 1`, `verify_n19_nprime_attack.py` `[proved]`):**
  - `L = "cab"`, `p = 0`, `LF = [2, 0, 1]` (single 3-cycle), `T = "abc"`, `J₁ = {0, 1, 2}` (`h₁ = 3`), `H = {'a':1, 'b':1, 'c':1}`.
  - Decoder executes **2** links (`row 0 → T[2]='c'`, `row 2 → T[1]='b'`), never visits width-1 row `1`, and writes `T[0] = 'a'` from `H - written`.
  - **Result:** Byte-exact (`T1`), `|B| = 0`, **`c₁ = 2 < 3 = h₁ − |B|`** (width-1 row `1` written by neither a link nor a rigid-placed fact).
- **Amplified arbitrary-`k` family (`k = 1..6`, `verify_n19_nprime_attack.py` `[proved]`):**
  - Valid all-width-1 columns (`h₁ = n`) whose last `k` LF-orbit rows all emit `'a'`: stopping one forward chain `k` steps early and filling `T[0..k) = 'a'^k` by `H - written` achieves **`c₁ = h₁ − k`** with **`|B| = 0`** using zero extra input beyond `H`:
    - `k=1`: `n=2, h₁=2, c₁=1`; `k=2`: `n=5, h₁=5, c₁=3`; `k=3`: `n=11, h₁=11, c₁=8`; `k=4`: `n=23, h₁=23, c₁=19`; `k=5`: `n=47, h₁=47, c₁=42`; `k=6`: `n=108, h₁=108, c₁=102` (all 6 byte-exact `T1` verified).
  - *(Wide-row `P_𝒪` amplification `[argued]`):* Because Record 107 defines `B` over **width-1 start rows only**, pairing `fill_remaining(c*)` with `P_𝒪` labels restricted exclusively to *wide* rows `y ∉ J₁` immediately following width-1 `c*` rows keeps `|B| = 0` while omitting every such width-1 `c*` row.
- **Quantitative ceiling on the 6 admitted blocks (`[measured]`):**
  - Even with an oracle that skipped **every** width-1 occurrence of the single most frequent byte `c*` at zero re-entry cost, the ceiling is `max_c h₁(c) / h₁`:
    - `block-0.L`: `'4'` = `2,398 / 35,116` (**6.83%**); `block-2.L`: `'4'` = `2,327 / 34,262` (**6.79%**); `block-85.L`: `'4'` = `3,699 / 55,504` (**6.66%**); `block-143.L`: `'4'` = `4,738 / 72,578` (**6.53%**); `block-210.L`: `'4'` = `6,626 / 97,054` (**6.83%**); `block-207.L`: `'4'` = `3,968 / 62,407` (**6.36%**). *(Byte `'4'` leads on all 6 blocks due to UUIDv4's fixed version nibble.)*
  - On a single chain without free re-entry, skipping only the trailing constant-symbol run saves `O(1)` rows. Thus `fill_remaining(c*)` breaks Theorem N′ formally, **but cannot approach `δ = 50%` on `𝓕`** (`[measured]` + `[argued]`). *(Generalising to multi-symbol omission — leaving `m` slots of distinct symbols unwritten and solving via BWT consistency `BWT(T) = L` — is exact in principle but costs `Ω(n)` work per candidate `[argued]`.)*

### Mechanism 2 — `Λ′ + copy(dst, src, len)` (Output-to-Output Positional Copy Write) `[proved]`

**Operation:** Copy an already-written output slice `out[src .. src + len)` to `out[dst .. dst + len)` (charged 1 event for the copy instruction itself). Any width-1 row `x` whose byte lands in `out[dst .. dst + len)` is written without any link or rigid fact through `x`.

- **Verified witness (`n = 33, h₁ = 33`, `verify_n19_nprime_attack.py` `[proved]`):**
  - `T = "8z1abc2p2Ayc3q3Bxbc4r4Cwc6s6D7abc"`, `L = "zpcqcrcscDc234617axabybwb2346CBA8"`, `p = 10` (single 33-cycle, **all 33 rows width-1**, `h₁ = 33`, `|B| = 0`).
  - Decoder starts a forward chain at `y = LF³(p)` via one charged `P₁` label (emitting `T[0..30)` in `30` charged events), then executes one charged `copy(dst=30, src=3, len=3)` writing `"abc"` at width-1 rows `[10, 19, 24]` with **zero** rigid facts or links through them.
  - **Result:** Byte-exact (`T1`), `|B| = 0`, **`c₁ = 30 + 1 (copy) = 31 < 33 = h₁ − |B|`** (even charging `1` for `copy`; and `c₁|_{J₁} = 30 < 33`).
- **Quantitative ceiling on the 6 admitted blocks (`[measured]`, repaired `N-16`):**
  - On `𝓕`, unidirectional acyclic `copy` plans fail `δ = 1/2` even as a free oracle (`z / h₁ = 0.54–0.64 > 0.50` on `5/6` blocks at `min_len = 2`; `1.27–1.51` at `min_len = 8`, with `76.6–80.5%` of `h₁` uncovered literals).

### Mechanism 3 — `Λ′ + E1` (Scattered Class-Fact Write) `[proved]`

**Operation:** Admit multi-row non-rigid class facts `(S, h, w)` on a scattered row set `S` sharing `h`-byte word `w` (explicitly placed outside `Λ′` by Record 107 §2).

- **Witness 3a (Round-2 18-row witness completed to a full byte-exact decoder `[proved]`):**
  - `L = "Paaabxbccddeeff$MZ"`, `p = 0`, `T = "$fedcbaMfedcbaZxaP"`, `J₁ = {0, 4, 5, 6, 15, 16, 17}` (`h₁ = 7`, `|B| = 0`).
  - One E1 class fact on `S = {1, 3}` (`w = "abcdef"`) writes interior width-1 rows `4` and `6` (`'b'`) with **zero** rigid facts or links through them: **`c₁|_{J₁} = 5 < 7 = h₁`** (byte-exact `T1` verified).
- **Witness 3b (All-width-1 E1 witness beating TOTAL `c₁ < h₁ − |B|` `[proved]`):**
  - `L = "a$aAbvbwbcxcaBaCa"`, `p = 0`, `T = "$ACwavaBcbaxbacba"`, `n = 17`, **all 17 rows width-1** (`h₁ = 17`, `|B| = 0`).
  - One E1 class fact on `S = {0, 16}` (`h = 3, w = "abc"`, paths `[0, 4, 9]` and `[16, 8, 11]`) discharges four interior width-1 rows `{4, 9, 8, 11}` with **zero** rigid facts or links through them.
  - **Result:** Byte-exact (`T1`), `|B| = 0`, **`c₁ = 11 (links) + 2 (E1 links) + 1 (E1 derive) = 14 < 17 = h₁ − |B|`**.
- **Quantitative ceiling on the 6 admitted blocks (`[argued]`, Record 96 `G3`):** On novel random hex nibbles (`61.3–68.2%` of `h₁`, `N-8`), `k`-byte classes shatter to singletons past `k ≈ 5`.

## 3. Summary & obstruction table entry (all six fields)

| Decoder class | Writes width-1 byte w/o link or rigid-placed fact? | Beats `c₁ ≥ h₁ − \|B\|` at `\|B\| = 0`? | Quantitative ceiling on `𝓕` (6 blocks) |
|---|---|---|---|
| **Strictly inside `Λ′`** | **No** (`[proved]`, Prop. 19.1) | **No** | Theorem N′ holds verbatim |
| **`Λ′ + fill_remaining(c*)`** | **Yes** (`L="cab", p=0`, and `c₁ = h₁ - k` family `[proved]`) | **Yes** (`c₁ = h₁ - k`) | `≤ 6.36–6.83%` of `h₁` (`max_c h₁(c)/h₁` at `'4'` `[measured]`) |
| **`Λ′ + copy(dst, src, len)`** | **Yes** (`n=33, h₁=33, c₁=31` `[proved]`) | **Yes** (even charging `1` per copy) | Fails `δ = 1/2`: `z/h₁ = 0.54–0.64` (`min_len=2`), `1.27–1.51` (`min_len=8`, `N-16` `[measured]`) |
| **`Λ′ + E1` (scattered class facts)** | **Yes** (`L="a$aAbvbwbcxcaBaCa", n=17, h₁=17, c₁=14` `[proved]`) | **Yes** (even charging derive + start links) | Shatters past `k ≈ 5` on novel hex (`G3`, Record 96 `[argued]`) |

| Field | Value |
|---|---|
| **Framework** | Non-rigid exact write extensions in `Λ′`'s immediate neighbourhood (`fill_remaining`, `copy`, `E1`) attacking Theorem N′'s output model (`N-19`, MONODROMY `ASK N`) |
| **Route class** | **Outside `Λ′`** (exact `T1` decoders with non-rigid byte writes) |
| **Gate failed** | **Inside `Λ′`:** syntactic impossibility (`[proved]`, Prop. 19.1). **In `Λ′`'s neighbourhood on `𝓕`:** **(d) Payoff** against `δ = 1/2` (`fill_remaining` ≤ `6.83%` of `h₁`; `copy` `z/h₁ = 0.54–0.64` on 5/6 blocks at `min_len=2` and `1.27–1.51` at `min_len=8`; `E1` shatters past `k ≈ 5` on novel hex) |
| **Witness** | All byte-exact `T1`-verified in `verify_n19_nprime_attack.py`: (1) `L="cab", p=0` (`h₁=3, c₁=2`) and `k=1..6` all-`w1` family (`c₁=h₁-k`); (2) `L="zpcqcrcscDc234617axabybwb2346CBA8", p=10` (`h₁=33, c₁=31`); (3) `L="a$aAbvbwbcxcaBaCa", p=0` (`h₁=17, c₁=14`) and completed 18-row `L="Paaabxbccddeeff$MZ"` (`c₁\|_J1=5<7`) |
| **Scope** | Strictly inside `Λ′` (Record 107 §2), Theorem N′'s output model is airtight; in `Λ′`'s immediate neighbourhood, the three extensions above show Theorem N′ is **not** a theorem over all exact decoders |
| **Escape condition** | To extend Theorem N′ beyond `Λ′`, MONODROMY must either (a) keep `fill_remaining`/`copy`/`E1` explicitly outside the scoped class (and record their separate quantitative ceilings on `𝓕`), or (b) add a charging invariant for non-rigid output writes (e.g., bounding omitted bytes by `max_c h₁(c)` and copy/E1 writes by repeat multiplicity) |
| **Reach** | **Exact** for `Λ′` and all three neighbourhood extensions (`[proved]`); quantitative ceilings measured on the 6 admitted blocks of `𝓕` |
| **Strength** | **Proved** (Proposition 19.1 + 3 explicit byte-exact `T1` decoder families beating `c₁ ≥ h₁ − |B|` at `|B| = 0`) + **measured exact** (`max_c h₁(c)/h₁` on 6 blocks; `N-16` copy ceiling) |
| **Transferable insight** | Theorem N′'s real vulnerability is **twofold**, not onefold: (i) its **output model** excludes exact non-rigid writes (`fill_remaining`, `copy`, `E1`), which break `c₁ ≥ h₁ − |B|` even at `|B| = 0` on small valid columns — though all three hit quantitative walls well short of `δ = 1/2` on Wikidata `𝓕`; and (ii) its **input model (`|B|`)** remains the genuine open frontier for order-sensitive `𝒪`, where `N-18` Lemma 18.1 replaces single-flip blindness with complete-observation fibres `U_𝒪` |

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact valid single-cycle columns verified by `LF` + `orbit_maps`; exact `h₁` byte counts on 6 admitted blocks.
2. **Model assumptions:** Record 107 §2's `Λ′` taken verbatim; each neighbourhood extension adds a single deterministic, `T1`-exact write primitive and still charges the primitive itself (`1` for `copy`, `1` for `E1_derive` + start links).
3. **Method:** Formal proof strictly inside `Λ′` + constructive synthesis and byte-exact execution of three counterexample families in `Λ′`'s neighbourhood.
4. **Outcome:** Directly answers MONODROMY's retargeted `ASK N`: Theorem N′ is sound **strictly inside `Λ′`** (by syntactic fiat), **broken immediately outside `Λ′`** by `fill_remaining`, `copy`, and `E1` even at `|B| = 0`, yet none of the three provides a `δ = 1/2` route on `𝓕`.
