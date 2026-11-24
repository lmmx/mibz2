# Formalisation of the mibz2 Research Problem

*Council response to `BRIEF.md` (`fd406da9`), 24 November 2026.*

**Provenance tags.** Every substantive claim is tagged `[proved]` (direct mathematical argument here, prose-level), `[argued]` (supported analytical or methodological judgement), `[conjectured]` (open mathematical claim), or `[taken from source: ...]` (fact, theorem, measurement, or position drawn from the pinned sources). No construction or solution is proposed.

**Pinned source abbreviations.**
- **Master (`c282b422`):** `FRONTIER` (`docs/team/research-charter/FRONTIER.md`), `METHOD` (`docs/team/research-charter/METHOD.md`), `PROOF-PROG` (`docs/team/research-charter/PROOF-PROGRAMME.md`), `AM-SRC` (`docs/journal/2026-11-24-against-mathematics-source.md`).
- **Formulation journal (`2a99cd51`):** `J§k` (`docs/journal/2026-11-24-algorithm-level.md` §k), `REV1` (`.../outside-review.md`), `REV2` (`.../chatgpt-round2-response.md`), `AM-GPT` (`.../against-mathematics-chatgpt.md`).
- **MONODROMY (`f86a4d3b`):** `MT` (`docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md`), `M96`–`M110` (MONODROMY records 96–110 in `docs/journal/monodromy/2026-11-24-*.md`), `CHAN#k` (`channel/coordinator-to-neon.md` entry #k), `HANDOFF` (`docs/journal/monodromy/HANDOFF-2026-11-24.md`).
- **Neon (`cc3b76f3`):** `N-k` (`research/neon/n*.md`, items N-1 to N-19; amended N-0 is in `N-8`).
- **External sources:** `DOOM` (doomslide, *"Against Mathematics"*, 24 Oct 2026, `doomslide.substack.com/p/against-mathematics`), `MARIO` (`tra38/sm64-wmotr-abc-proof`, `README.md`, `docs/theorem-scope.md`, `docs/trust-model.md`, `docs/feasibility-and-the-critiques.md`), `BLOG` (`lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/`).

---

## 1. Definitions

### 1.1 The input and domain stratification

`[taken from source: PROOF-PROG §1.1–§1.2, M101, M103 §1]` Fix the byte alphabet $\Sigma := \{0, 1, \dots, 255\}$ (so $|\Sigma| = 256$; in concrete blocks the active alphabet size is $\sigma \le 256$) and the bzip2 block-size ceiling $N_{\max} := 900{,}000$ ($100{,}000 \times \text{level}$ for $\text{level} \in \{1, \dots, 9\}$). Write $[n] := \{0, 1, \dots, n-1\}$.

1. **Column (`Column`).** `[taken from source: PROOF-PROG §1.2]` A *column* is a triple $c = (n, L, p)$ with $1 \le n \le N_{\max}$, $L : [n] \to \Sigma$, and primary index $p = \mathrm{origPtr} \in [n]$ (24-bit unsigned in bzip2 headers). Its derived objects are:
   - byte histogram $H_c(a) := |\{i \in [n] : L[i] = a\}|$ and cumulative base $C_c(a) := \sum_{b < a} H_c(b)$;
   - prefix rank $\mathrm{rank}_c(a, i) := |\{j < i : L[j] = a\}|$;
   - the LF map $\mathrm{LF}_c(i) := C_c(L[i]) + \mathrm{rank}_c(L[i], i)$, which is a bijection on $[n]$, and its inverse $\Psi_c := \mathrm{LF}_c^{-1}$;
   - maximal runs $R_0, \dots, R_{r-1}$ partitioning $[n]$ into maximal contiguous intervals on each of which $L$ is constant, with $r(c) := r$, run widths $w(R_j) := |R_j|$, width-1 row set $J_1(c) := \{x \in [n] : w(R(x)) = 1\}$, and $h_1(c) := |J_1(c)|$.
2. **Entry representation $I_0(c)$.** `[taken from source: PROOF-PROG §1.1, §2.2 O1; M98 §0]` For a standard bzip2 BWT kernel (R2 track), the input supplied to the kernel after Huffman/MTF/RLE2 decoding is the sequence of run records $\mathrm{Runs}(L) = \langle (a_0, w_0), \dots, (a_{r-1}, w_{r-1}) \rangle$ (or capped run records of width $\le 127$ that merge in one $O(r)$ pass to maximal runs), together with $(n, p)$ and $H_c$. (For the R3 track within standard bzip2 streams, the entry point may sit earlier at the block's Huffman/MTF/RLE2 bitstream, subject to the EOB causal barrier `[taken from source: FRONTIER Obligation 5]`.)
3. **Three nested input domains.** `[proved]` The records conflate three distinct domains; they must be separated:
   $$\mathrm{SingleCycle} \subsetneq \mathrm{ValidBWT} \subsetneq \mathrm{Column}.$$
   - $\mathrm{ValidBWT}$: columns $(n, L, p)$ for which there exist $T : [n] \to \Sigma$ and a rotation-sorting bijection $\sigma : [n] \xrightarrow{\sim} [n]$ (with $\mathrm{rot}(T, \sigma(i)) \le_{\text{lex}} \mathrm{rot}(T, \sigma(j))$ for $i \le j$) such that $L[i] = T[(\sigma(i) - 1) \bmod n]$ and $\mathrm{rot}(T, \sigma(p)) = T$ `[taken from source: PROOF-PROG §1.2]`.
   - $\mathrm{SingleCycle}$: columns $(n, L, p)$ whose permutation $\mathrm{LF}_c$ consists of a single $n$-cycle.
   - *Exact relation `[proved]`:* By the Gessel–Reutenauer / Mantaci–Restivo–Sciortino characterisation of cyclic BWTs, $c \in \mathrm{SingleCycle}$ iff $c \in \mathrm{ValidBWT}$ for a **primitive** (aperiodic) word $T \in \Sigma^n$. If $T = u^k$ is periodic with primitive root $u$ of length $n/k$ ($k \ge 2$), then $c \in \mathrm{ValidBWT} \setminus \mathrm{SingleCycle}$: $\mathrm{LF}_c$ splits into $k$ disjoint cycles of length $n/k$, every rotation has multiplicity $k$, and $c$ is still a legal bzip2 block. Conversely, arbitrary corrupt columns (e.g. `L = "aab", p = 0`, `N-1`) lie in $\mathrm{Column} \setminus \mathrm{ValidBWT}$.
4. **Asymptotic uniform family vs finite validation corpus.** `[taken from source: REV1 §1; M103 §4; N-2]` To prevent lookup-table advice and make $O(\cdot)$ meaningful, drop the bound $n \le N_{\max}$ when defining the **uniform structural family** $\mathcal{G}$ (an infinite sequence of probability distributions or families of single-cycle columns over fixed alphabet $|\Sigma| \le 256$, e.g. M103 §4's scaffold-plus-random-slots family $\mathcal{G}(S, m, k)$). Retain the **frozen validation corpus** $\mathcal{F}$ (the 232 distinct two-ended Wikidata blocks of `#805`, which contain the 6 admitted blocks of `data/admitted-bwt` as a subset `[taken from source: N-2 §0]`) strictly as an empirical validation set at $n \le 900{,}000$, never as the sole theorem domain.

### 1.2 The output and correctness condition

`[taken from source: PROOF-PROG §1.2, §2.1]` The specification $\mathrm{decode} : \mathrm{Column} \to \Sigma^*$ is defined on every $c = (n, L, p) \in \mathrm{Column}$ by
$$\mathrm{decode}(c)[j] := L\bigl(\Psi_c^{\,j+1}(p)\bigr), \qquad j \in [n].$$
Equivalently `[proved; taken from source: PROOF-PROG §1.2]`, letting $\ell := \mathrm{per}(\Psi_c, p) \le n$ be the length of $p$'s cycle under $\Psi_c$ (so $\ell = n$ on $\mathrm{SingleCycle}$),
$$\mathrm{decode}(c)[\ell - 1 - k] = L\bigl(\mathrm{LF}_c^k(p)\bigr) \quad (0 \le k < \ell), \qquad \mathrm{decode}(c)[j] = \mathrm{decode}(c)[j \bmod \ell].$$
The output is the RLE1-encoded block byte string $T \in \Sigma^n$ (RLE1 expansion and CRC32 check are downstream).

- **Exactness condition (`Exact(D, D_dom)`).** `[taken from source: PROOF-PROG §1.1, M105 §4, MT §1]` A deterministic decoder $D$ is *exact* on domain $\mathcal{D}_{\text{dom}}$ iff
  $$\forall c \in \mathcal{D}_{\text{dom}}, \qquad \mathrm{Out}(D, c) = \mathrm{decode}(c)$$
  with zero error probability. In particular, speculative decoders verified only by a 32-bit CRC fail `Exact` `[taken from source: M105 §4]`, and no intermediate representation (suffix array, inverse suffix array, or N/R split) is part of the output contract `[taken from source: REV1 §1, FRONTIER FQ4]`.
- **Two exactness levels:** *BWT-exactness* ($\mathcal{D}_{\text{dom}} = \mathrm{SingleCycle}$ or $\mathrm{ValidBWT}$) vs *production total exactness* ($\mathcal{D}_{\text{dom}} = \mathrm{Column}$, matching `walk` identically on corrupt blocks so downstream CRC rejection is preserved `[taken from source: PROOF-PROG §1.1]`).

### 1.3 Model of computation, instruction set, passes, memory, and cost model

`[taken from source: M108 §2–§4, REV1 §1–§2, REV2 §II(c), M107 §4, N-15]` Record 108 proved that trying to charge arbitrary decoders via a single calculus event count $c_1$ plus an unformalised "aggregate" clause collapses into a dichotomy set by convention:
- if every record read by a streaming build pass is charged to $c_1$, then every exact decoder pays $c_1 \ge h_1 - \max_a h_1(a)$ (about $0.93\,h_1$ on $\mathcal{F}$) because every width-1 byte appears nowhere else in the input — an impossibility by fiat `[taken from source: M108 §3, N-19 §2]`;
- if a streaming pass is uncharged, then $j$ pointer-doubling passes give $c_1 = m / 2^j < h_1/2$ at $K = 3$ with nothing new, and $K$ alone sets the verdict `[taken from source: M108 §3]`.

`[argued]` Resolving this dichotomy requires cleanly separating **three formal layers** that the current records conflate: (A) the concrete execution model $(\mathcal{I}, \mathcal{H})$, (B) the abstract derivation calculus $(\Lambda, \kappa_\Lambda)$, and (C) empirical hardware receipts.

#### Layer A — Concrete execution model $(\mathcal{I}, \mathcal{H})$

1. **Uniform machine and declared instruction set $\mathcal{I}$.** `[argued; incorporating M108 §2, REV1 §1, REV2 §II(c)]` A *decoder* is a single finite, advice-free deterministic program $P \in \mathrm{Prog}(\mathcal{I})$ (size $O(1)$, independent of $n$, $\mathcal{F}$, and $c$) executing on a Word-RAM of word width $w = \Theta(\log n)$ bits (concretely $w = 64$ at $n \le 900{,}000$) equipped with an **explicitly declared finite instruction set** $\mathcal{I}$:
   - *Standard scalar Word-RAM $\mathcal{I}_{\text{RAM}}$:* $O(1)$-word arithmetic, bitwise, comparison, branch, and word load/store instructions, each of atomic cost $1$.
   - *Optional declared packed/SIMD extension $\mathcal{I}_{\text{vec}}(W_{\text{vec}})$:* explicitly named broadword/SIMD instructions (e.g. `pshufb`, `pext`, gather, $W_{\text{vec}} = 256$ bits) with declared unit cost $1$ (or explicit cycle/micro-op weight vector $\omega : \mathcal{I} \to \mathbb{R}_{>0}$).
   - **Why declaring $\mathcal{I}$ makes charges meaningful `[proved; extending M108 §2]`:** An input-output function $c \mapsto \mathrm{decode}(c)$ has no intrinsic "event count", and an unconstrained observation function $\iota(L, p)$ trivialises charging via $\iota = \mathrm{id}$ `[taken from source: M108 §2]`. A charge is mathematically well-defined only on executions of a fixed, finite signature $\mathcal{I}$ of primitive operations whose operational semantics and unit costs are fixed prior to any candidate. Every phase — input scan, `Build`, `Advance`, `Emit`, `Glue` — executes solely in $\mathcal{I}$; no uncharged oracle or hidden helper exists.
2. **Typed memory regions (closing the scratch and output-buffer loopholes).** `[argued; repairing REV1 §1]` Machine memory is partitioned into three disjoint regions:
   - **Input tape $\mathsf{In}$ (read-only):** holds $I_0(c)$ (size $|\mathsf{In}| = O(r)$ words for $\mathrm{Runs}(L)$, or $n$ bytes if expanded).
   - **Committed output buffer $\mathsf{Out}[0..n)$ (write-once by default):** each byte $\mathsf{Out}[j]$ is written once with its final value $\mathrm{decode}(c)[j]$. *Anti-cheat rule:* if $P$ reads back from $\mathsf{Out}$ (e.g. `copy(dst, src, len)`) or overwrites any cell of $\mathsf{Out}$, then every read/overwrite is charged in memory traffic $Q_{\mathcal{H}}$ and instructions $W_{\mathcal{I}}$, and any live region of $\mathsf{Out}$ that may still be read back is counted inside working state (or flagged explicitly as a read-write output buffer).
   - **Working RAM $\mathsf{Mem}$ (read/write scratch):** peak live allocation $S(P, c)$ in bytes (or words), taken over the entire execution (acquisition + consumer + glue).
3. **What a "pass" is (resolving $K$'s ambiguity).** `[argued; repairing REV1 §1, M108 §3]` Distinguish:
   - *Input-tape passes* $K_{\text{in}}(P, c)$: unidirectional scans of $\mathsf{In}$. When $S(P, c) = \Omega(r)$, $K_{\text{in}} = 1$ suffices to copy $\mathsf{In}$ into $\mathsf{Mem}$ and run any RAM algorithm, so $K_{\text{in}}$ alone does not bound work `[taken from source: REV1 §1, §4]`.
   - *Sequential bulk sweeps* $K_{\text{sweep}}(P, c)$: the total sequential traffic over $\mathsf{In}$, $\mathsf{Mem}$, and $\mathsf{Out}$ organised as contiguous or $O(\sigma)$-cursor monotone scans (as in counting sort, radix/stable partition, doubling, or sweep composition), measured either by explicit sweep count over $\Omega(r)$-sized arrays or continuously by total cache-line transfers $Q_{\mathcal{H}}(P, c)$.
   - **Resolution of Record 108's dichotomy `[argued]`:** A bulk pass is **neither** free (which lets doubling cheat) **nor** charged as a forbidden dependent single-row event (which makes reading $I_0(c)$ impossible by fiat). Instead:
     - the irreducible I/O baseline $C_{\text{io}}(c)$ — reading $I_0(c)$ once ($O(r)$ sequential reads) and writing $\mathsf{Out}[0..n)$ once ($O(n/w)$ sequential word stores) — is accounted separately as unavoidable I/O;
     - **every** non-I/O instruction executed during `Build`, `Advance`, `Emit`, and `Glue` — including every bulk sweep over working arrays — is charged explicitly in $\mathcal{I}$, split into:
       - $W_{\text{seq}}(P, c)$: sequential / $O(\sigma)$-cursor-monotone instructions in $\mathcal{I}$, and
       - $W_{\text{dep}}(P, c)$: data-dependent non-sequential instructions (loads/stores whose address depends on previously loaded data outside $O(\sigma)$ monotone cursors);
       with total core work $W_{\text{core}}(P, c) := W_{\text{seq}}(P, c) + W_{\text{dep}}(P, c)$ (or weighted $\alpha W_{\text{dep}} + \beta W_{\text{seq}}$ under a declared hardware weight pair).
     - Under this accounting, $j$ doubling passes pay $j \cdot m$ bulk-sweep instructions in $W_{\text{seq}}$ to save $(1 - 2^{-j}) m$ links in $W_{\text{dep}}$: total core instructions $W_{\text{core}}$ **increase** from $m$ to $(j + 2^{-j}) m > m$, so doubling is visibly a *relocation* from $W_{\text{dep}}$ to $W_{\text{seq}}$ (residual (b): unit price), not a reduction of the floor!
4. **Fixed memory hierarchy model $\mathcal{H}$ (for traffic and miss guards).** `[taken from source: REV1 §2, N-15]` Fix $\mathcal{H} = (w, B, M_{\text{L1}}, M_{\text{L2}}, M_{\text{L3}}, W_{\text{workers}})$ with replacement policy (e.g. ideal LRU; concrete i9-10900K instantiation from `N-15`: $w = 64\text{ b}$, $B = 64\text{ B}$, $M_{\text{L1}} = 32\text{ KiB}$, $M_{\text{L2}} = 256\text{ KiB}$, $M_{\text{L3}} = 20\text{ MiB}$, $2{,}048\text{ KiB/worker}$ at $W = 10$). The full execution cost vector is
   $$\mathrm{Cost}_{\mathcal{I},\mathcal{H}}(P, c) := \bigl(W_{\text{core}}(P, c),\; W_{\text{dep}}(P, c),\; W_{\text{seq}}(P, c),\; Q_{\mathcal{H}}(P, c),\; D_{\mathcal{H}}(P, c),\; S_{\mathcal{H}}(P, c),\; S(P, c),\; K_{\text{sweep}}(P, c)\bigr),$$
   where $Q_{\mathcal{H}}$ is total cache-line transfers, $D_{\mathcal{H}}$ is data-dependent cache misses (at L2 and per-worker L3), and $S_{\mathcal{H}}$ is critical-path dependent-miss depth.

#### Layer B — Derivation calculus $(\Lambda, \kappa_\Lambda)$ (structural diagnostic layer)

`[taken from source: M98 §1, M103 §1, M105 §1, M107 §2, §4]` A *derivation calculus* $\Lambda$ is a formal proof system whose derivations live in `Type` (not `Prop`, so distinct computational witnesses are not collapsed by definitional proof irrelevance `[taken from source: DOOM fn. 2, AM-GPT §II.3, M105 §1]`), equipped with:
- a type $\mathrm{Fact}_\Lambda(c)$ of sound assertions about $c$ (for $\mathcal{R}$: rigid forward facts $(I, h, t, w)$ with $I = [a, b)$, $h \ge 1$, $\forall x \in I,\; \mathrm{LF}_c^h(x) = x + t$ and emitted word $w$);
- a finite set of inference rules (for $\mathcal{R}$: `prim`, `restrict`, `compose`, `join`; single orientation forward or backward `[taken from source: M103 §1]`);
- an operational trace semantics (DAG nodes built once, reuse free; output links counted per execution `[taken from source: M103 §1]`);
- a structural event charge $\kappa_\Lambda : \mathrm{Trace}_\Lambda(c) \to \mathbb{N}$ (for $\mathcal{R}$, $c_1$ counts width-1 `compose` DAG nodes attributed to join row $\mathrm{LF}^{h_f}(x)$, plus executed output links attributed to start row).

**Bridge condition (cost-preserving simulation).** `[taken from source: REV1 §3, M103 §2, M106 §4, M107 §4]` An executable program $P \in \mathrm{Prog}(\mathcal{I})$ belongs to a calculus class $\Lambda$ *only* when accompanied by an explicit lowering $\llbracket P \rrbracket : \mathrm{Trace}_\Lambda$ together with a proved distortion bound showing that $\kappa_\Lambda(\llbracket P \rrbracket(c))$ does not artificially inflate $W_{\text{core}}(P, c)$ by expanding bulk operations into singleton derivations. Without a cost-preserving lowering, $P$ is classified **unplaced** in $\Lambda$ — neither closed by $\Lambda$'s lower bound nor certified as an escape by syntax alone.

#### Layer C — Empirical hardware receipts

`[taken from source: MT §7, MARIO trust-model.md, J§23]` Native wall-clock time on real hardware (i9-10900K under `PERFORMANCE-POLICY.md` at declared worker counts $W$) and supplied-capability headroom are empirical receipts outside Layers A and B. Neither Layer A nor Layer B implies Layer C without hardware calibration `[taken from source: METHOD §5, M105 §2]`.

---

## 2. What "a new basis" means, formally

`[taken from source: J§1, J§18, J§22, J§32, CHAN#14, AM-GPT §II.2–3, REV2 §II(c), §III.1]` An **inversion basis** in declared instruction set $\mathcal{I}$ over domain $\mathcal{D}$ is a 6-tuple
$$\mathcal{B} = \bigl(\mathcal{O},\; \mathrm{Valid}_{\mathcal{O}},\; \mathrm{Build},\; \mathrm{Advance},\; \mathrm{Emit},\; \theta\bigr)$$
where:
- $\mathcal{O}(c)$ is a concrete data type (`Type`) of *computational carrier objects* with validity predicate $\mathrm{Valid}_{\mathcal{O}}(c, z)$;
- $\mathrm{Build} : I_0(c) \to \mathcal{O}(c)$ constructs an initial carrier object $z_0$ from the entry representation;
- $\mathrm{Advance} : \mathcal{O}(c) \to \mathcal{O}(c)$ is a closed transport/reduction operator on $\mathcal{O}(c)$ (iterated to a terminal state $z^*$);
- $\mathrm{Emit} : \mathcal{O}(c) \to \Sigma^n$ decodes $z^*$ to the output bytes, satisfying $\mathrm{Valid}_{\mathcal{O}}(c, z^*) \implies \mathrm{Emit}(z^*) = \mathrm{decode}(c)$;
- $\theta : \mathcal{D} \to \mathbb{N}$ is a block-structural parameter (intrinsic to $c$, independent of implementation constants).

Given a set of prior bases $\mathfrak{B}_{\text{prior}}$ (here $\{\mathcal{B}_{\text{walk}}, \mathcal{B}_{\text{RV}}\}$, whose union of derivations is captured by $\mathcal{R}$ with structural floor $F_{\text{prior}}(c)$), a basis $\mathcal{B}'$ is a **new basis** relative to $\mathfrak{B}_{\text{prior}}$ on uniform family $\mathcal{G}$ (validated on $\mathcal{F}$) with margin $\delta > 0$ iff it satisfies three mathematical conditions:

1. **Condition B1 — New computational object (escape of carrier and algebra).** `[argued; formalising CHAN#14 #1, M104 §2, REV2 §III.1]`
   - *(Non-simulation into prior calculus)* There is **no** cost-preserving lowering from executions of $(\mathrm{Build}, \mathrm{Advance}, \mathrm{Emit})$ into the prior derivation calculus $\Lambda_{\text{prior}}$ (for $\mathcal{R}$, this requires dropping the interval-translation invariant of Lemma A: $\mathrm{LF}^j(I)$ contained in one maximal run for all $0 \le j < h$, which forces every fact touching $J_1$ to have width $1$ `[taken from source: M103 §2]`).
   - *(Algebraic closure on the new carrier)* $\mathrm{Advance}$ acts directly on $\mathcal{O}'(c)$ to transform carrier descriptors without materialising or enumerating the atoms of the prior carrier (rows $x \in J_1$) one by one to evaluate $\mathrm{LF}$ (`[taken from source: REV2 §III.1]`: "a descriptor algebra that remains closed under transport").
   - *Anti-renaming test `[proved]`:* Wrapping a prior object in a new struct name or reordering its traversal (e.g. scalar ordered frontier, record 104 §1a, or discrete Morse 1-complex collapses, N-6) admits a 1-to-1 cost-preserving bijection into $\mathcal{R}$ and therefore fails B1.
2. **Condition B2 — New governing cost law in a new parameter $\theta'$.** `[argued; formalising CHAN#14 #2, J§18, FRONTIER viability check]`
   - *(Proved cost law)* Total non-I/O work of consumption and acquisition obeys
     $$W_{\text{core}}(\mathcal{B}', c) = O\bigl(f(\theta'(c))\bigr) \qquad \text{with } S(\mathcal{B}', c) \le S_{\max}(c),\; K_{\text{sweep}}(\mathcal{B}', c) \le K_{\max}.$$
   - *(Separation where the prior basis fails)* On the hard workload of $\mathcal{G}$ (where the prior parameter $F_{\text{prior}}(c)$ is large — specifically $h_1(c) = \Theta(r(c))$ on novel-random-heavy blocks in $\mathcal{G}(S, m, k)$) and on every block of $\mathcal{F}$,
     $$\frac{W_{\text{core}}(\mathcal{B}', c)}{F_{\text{prior}}(c)} \le 1 - \delta$$
     (and asymptotically $\theta'(c) / F_{\text{prior}}(c) \to 0$ or $\le 1 - \delta$ along an infinite sequence in $\mathcal{G}$), while respecting the Shannon entropy floor $\mathrm{bits}(z) \ge H(T)$ (`[proved]`: on novel random content, $\theta'(c) \ll h_1(c)$ can govern *instruction count* — e.g. $\Theta(h_1 \log \sigma / w)$ packed word operations or $\Theta(h_1 / \log_\sigma n)$ multi-byte phrases — never bit entropy).
3. **Condition B3 — Native acquisition without the old mechanism.** `[argued; formalising CHAN#14 #3, METHOD §3a]`
   - $\mathrm{Build}(I_0(c))$ constructs $z_0 \in \mathcal{O}'(c)$ with $\mathrm{Valid}_{\mathcal{O}'}(c, z_0)$ using **total** non-I/O work $W_{\text{core}}(\mathrm{Build}, c)$ already accounted inside B2's bound — i.e.,
     $$W_{\text{core}}(\mathrm{Build}, c) + W_{\text{core}}(\mathrm{Advance} + \mathrm{Emit}, c) \le (1 - \delta)\, F_{\text{prior}}(c).$$
   - In particular, `Build` cannot embed an $\Omega(F_{\text{prior}}(c))$ row-by-row LF walk, nor an $\Omega(h_1)$ run-boundary SA walk, nor $\Omega(m)$ bulk compositions disguised as a free pass.
   - *(Supplied vs acquired basis)* A candidate meeting B1 and B2 with $z_0$ handed over by an oracle is a **supplied basis** (Archimedes' *Method* / Mario's supplied-state stage); it becomes an **acquired new basis** only once B3 is discharged.

### Sanity check: $\mathcal{B}_{\text{walk}} \to \mathcal{B}_{\text{RV}}$ (LF walk → Rauzy–Veech induction / Cells) `[proved]`

Take $\mathfrak{B}_{\text{prior}} = \{\mathcal{B}_{\text{walk}}\}$, where $\mathcal{O}_{\text{walk}}(c) = [n]$ (points/rows), $\mathrm{Advance}(x) = \mathrm{LF}_c(x)$, and $\Lambda_{\text{walk}}$ has only 1-step point transitions with floor $F_{\text{walk}}(c) = n$ dependent row evaluations. Take $\mathcal{B}_{\text{RV}}$ (Cells `[taken from source: PROOF-PROG §0, §2.2; M104 §2]`):

- **B1 holds `[proved]`:** $\mathcal{O}_{\text{RV}}(c)$ consists of **intervals (runs/bands)** $I = [a, b) \subseteq [n]$ carrying a translation shift $t$ and an SLP word-id $w$. $\mathrm{Advance}$ is right-end Rauzy–Veech induction (cutting the longer top domain/image interval by the shorter — *anthyphairesis* on $r$ intervals `[taken from source: BLOG §2, DOOM Boat III]` — composing two SLP ids and updating endpoints). One $\mathrm{Advance}$ step discharges $w(I) - 1$ interior LF evaluations without visiting interior rows; thus no cost-preserving lowering exists into $\Lambda_{\text{walk}}$ (contraction along `compose` collapses $n$ point tiles into $O(r)$ interval tiles `[taken from source: M104 §2]`).
- **B2 holds `[proved; empirical values taken from source: J§18, FRONTIER, N-2]`:** The governing parameter shifts from $\theta_{\text{walk}}(c) = n$ to $\theta_{\text{RV}}(c) = r(c)$ (maximal runs). Non-output work is $O(r)$–$O(r \log r)$ endpoint steps (`Cells` uses $1.013\text{–}1.019\,r$ accelerated induction steps `[taken from source: FRONTIER obstructions]`) plus $O(n)$ sequential SLP byte spelling. Asymptotically, on any repetitive family (e.g. Sturmian words, $r = 2$, $n \to \infty$), $r/n \to 0$; on Wikidata (`#805` census), mean maximal run width is $\approx 16$ ($r/n \approx 1/16$; and Open's retained set has median $m/n = 0.114$), an order-of-magnitude drop below $n$.
- **B3 holds `[proved; taken from source: PROOF-PROG §2.2 O1–O2]`:** $\mathrm{Build}$ reads $\mathrm{Runs}(L)$ directly from the MTF/RLE2 output in **one $O(r)$ sequential pass** using a 256-entry running rank counter to compute each run's base translation $t_R = C_c(a) + \mathrm{rank}_c(a, \mathrm{start}(R)) - \mathrm{start}(R)$, with **zero** row-by-row LF walk and $O(r)$ scratch.
- **Sharpness check (`Cells → Open`) `[proved; taken from source: M104 §2, METHOD §1]`:** By contrast, `Cells → Open` halts interval contraction at `LIMIT = 16`, staying inside the same calculus $\mathcal{R}$ (same interval-translation facts, same invariant $J_1$, same floor $h_1$). It therefore fails B1 relative to $\mathcal{B}_{\text{RV}}$ — matching `METHOD §1`'s classification of `walk → RV` as *Foundational* and `RV → Open` as *Architectural*.

---

## 3. What "lowering the floor" means, formally

### 3.1 Three floors in the records (which floor, in which class, in which currency)

`[proved; sources cited per row]` The records contain three distinct floors that must not be conflated:

| Floor | Class quantified over | Currency | Exact lower bound | Status |
|---|---|---|---|---|
| **1. Rigid provenance floor (G1)** | $\mathcal{R}$ (single-orientation rigid interval-translation DAGs on `SingleCycle`) | $c_1(D, c)$ (width-1 `compose` DAG nodes + executed links) | $c_1(D, c) \ge h_1(c)$, and $c_1\rvert_A \ge \lvert A\rvert$ for all $A \subseteq J_1(c)$ (row-preserving). Also $r(c) \ge h_1(c)$ for monotone induction schedules | Proved in prose (`M103 §2`, verified `N-14 §3`); Open sits at $c_1 \ge m \in [1.21, 3.32]\,h_1$ (`M102 §3`, `N-1`) and meets $c_1\rvert_{J_1} = h_1$ |
| **2. Fibre positional floor (Thm N′ + Lem 18.1)** | $\Lambda'(\mathcal{O})$ (rigid writes in $\mathcal{R}$ + start-row labels $P_1$ [charged] and $P_{\mathcal{O}}$ [uncharged]) | $c_1(D, c)$ | $c_1(D, c) \ge h_1(c) - \lvert J_1(c) \cap U_{\mathcal{O}}(c)\rvert$; for $\mathcal{O} = (\mathcal{P}, D)$, $c_1 \ge h_1 - \lvert\{x \in J_1 : \nu'(x) < \lvert\mathcal{P}\rvert + c_D\lvert D\rvert\}\rvert$ ($c_D \in \{1, 2\}$) | Proved in prose (`M107 §2`, `N-18 Lem 18.1–18.2`, `M108 §1`); $q_{50\%} = 6{,}903\text{–}14{,}120$ on 6 blocks (`N-18`) |
| **3. Execution / information floor** | All exact programs in $\mathrm{Prog}(\mathcal{I})$ | $W_{\text{core}}, Q_{\mathcal{H}}$ in $(\mathcal{I}, \mathcal{H})$ | Trivial: $\Omega(r)$ input reads, $\Omega(n/w)$ output writes, $\Omega(H(T)/w)$ word ops. No $\Omega(h_1)$ word-RAM lower bound is known | Proved trivial floor; non-trivial word-RAM LB open (`CHAN#11`, `REV1 §4`) |

- **Raising the ceiling vs lowering the floor `[argued; formalising J§32, M96 §4–5, M105 §2]`:**
  Write a decoder's kernel time schematically as $T_{\text{kern}} \approx \alpha \cdot N_{\text{ev}} + \beta \cdot B_{\text{io}} + T_{\text{build}}$.
  - *Raising the ceiling* keeps the decoder inside $\mathcal{R}$ (so $N_{\text{ev}} \ge h_1$ is locked by G1) and reduces the unit price $\alpha$ per event (e.g. branchless 16-byte stores, independent lanes, queue packing — route classes R0/R1 and residual (b) of `MT §1a`).
  - *Lowering the floor* means exiting $\mathcal{R}$ (and $\Lambda'(\mathcal{O}_{\text{hist}})$) to a new basis $\mathcal{B}'$ in $\mathrm{Prog}(\mathcal{I})$ whose **proved operational cost** in the common currency $W_{\text{core}}$ (or declared multi-row instruction count $z_{\mathcal{I}}$ over $\mathcal{I}$) drops strictly below the prior class's floor $F_{\text{prior}}(c)$:
    $$W_{\text{core}}(\mathcal{B}', c) \le (1 - \delta)\, F_{\text{prior}}(c)$$
    — concretely, for $\mathcal{B}_{\text{walk}} \to \mathcal{B}_{\text{RV}}$ the floor dropped from $n$ to $O(r)$; here the sought drop is from G1's floor $h_1(c)$ (and Open's $m(c)$ retained steps) to a new structural quantity $f(\theta'(c)) \le (1 - \delta)\,h_1(c)$ in $\mathcal{I}$.

### 3.2 Why lowering the floor is NOT relocating cost into an uncharged pass

`[proved; resolving M108 §3–§4]` Let $D = (\mathrm{Build}, \mathrm{Advance}, \mathrm{Emit}, \mathrm{Glue}) \in \mathrm{Prog}(\mathcal{I})$. Define **cost conservation across phases**:
$$W_{\text{core}}(D, c) = W_{\text{core}}(\mathrm{Build}, c) + W_{\text{core}}(\mathrm{Advance} + \mathrm{Emit}, c) + W_{\text{core}}(\mathrm{Glue}, c).$$
A candidate **relocates** cost rather than lowering the floor whenever $W_{\text{core}}(\mathrm{Emit}, c) < F_{\text{prior}}(c)$ while $W_{\text{core}}(\mathrm{Build}, c) = \Omega(F_{\text{prior}}(c))$ so that the sum $W_{\text{core}}(D, c) \ge F_{\text{prior}}(c)$:
- *Witness 1 (pointer doubling, `M108 §3`):* $j$ doubling passes reduce links from $m$ to $m/2^j$, but `Build` performs $j \cdot m$ compositions, giving total compositions $(j + 2^{-j}) m > m$.
- *Witness 2 (in-place sweep composition, `M110 §2–§3`):* one ascending sweep reduces links from $m$ to $\approx m/e$, but `Build` performs $m$ compositions, giving total $(1 + 1/e) m > m$.
- *Witness 3 (supplied LZ77 copy plan, `N-12/N-16`, `M108 §2`):* `Emit` executes $z \approx 0.54\text{–}0.64\,h_1$ copy/literal instructions at `min_len = 2`, while every known RLBWT-to-LZ77 `Build` spends $\Omega(n)$ work or $\Theta(\log n)$ passes (`N-12 §2`), leaving $W_{\text{core}}(\mathrm{Build} + \mathrm{Emit}) \gg h_1$.

In each case, an uncharged `Build` pass creates the illusion of a lowered floor by moving work from $W_{\text{dep}}$ to $W_{\text{seq}}$. That trade *can* still lower wall time if $\beta_{\text{seq}} \ll \alpha_{\text{dep}}$ (residual (b): unit price), but it is **strictly distinct** from lowering the structural floor $W_{\text{core}}$.

### 3.3 Relation to whole-decoder time (frozen clause 3)

`[proved from measurements; taken from source: M105 §2, M107 §1, N-2, J§30, MT §7]`
1. **Formal non-implication:** Event count $c_1$ and wall-clock time are incomparable: equal $c_1$ yields $0.55\text{–}0.78\times$ emit time under branchless 8-lane execution (`toy-transfer` L187–208 via `REV2 §I`, `M107 §1`), while fewer dependent links in $T^2$ yielded $1.22\times$ slower emit at 10 workers (`M105 §2`, `REV2 §II(e)`).
2. **Amdahl ceiling by worker count $W$:** Decompose whole-decoder time as $T_{\text{dec}}(W) = T_{\text{front}}(W) + T_{\text{construct}}(W) + T_{\text{emit}}(W) + T_{\text{tail}}(W)$. On Wikidata:
   - at $W = 1$, Open's emit is $46.7\%$ of kernel time (`N-2 §1`), and supplying Open's word codes in output order for free halves emit ($0.53\times$) and yields **$0.838\times$ whole-decoder time** ($16.2\%$ gain, clearing `METHOD §4`'s $10\%$ gate) `[taken from source: J§30, MT §7]`;
   - at $W = 10$ (the default), the same free emit order still halves emit ($0.55\times$) yet yields **$0.988\times$ ($[0.957, 1.020]$) whole-decoder time** — statistically indistinguishable from $1.00$ `[taken from source: J§30, MT §7]`.
3. **Consequence for clause 3 `[argued]`:** Lowering the BWT-inversion floor is **neither sufficient nor strictly necessary** for beating `master` on whole-decoder time at $W = 10$ (not sufficient because $W = 10$ is bounded outside emit by construction/front-end/bandwidth; not necessary because R0/R1 unit-price or front-end wins can move wall time without leaving $\mathcal{R}$). Therefore, clause 3 must remain an **independent empirical conjunct** with explicitly stated worker count(s) $W$, never conflated with the formal floor-lowering theorem.

---

## 4. The main theorem's shape, Mario-style

`[argued; modelling MARIO README.md, trust-model.md, feasibility-and-the-critiques.md; J§2, J§23; REV1 §1, §6]` Following `tra38/sm64-wmotr-abc-proof`, the formal architecture rests on four structural rules:
1. **TCB isolation by vocabulary (`Spec` module):** The top-level statement mentions *only* the audited specification (`Column`, `SingleCycle`, `decode`, the execution model $(\mathcal{I}, \mathcal{H})$, and the numeric budgets). Proof-internal devices — such as an N/R split $\sigma$, an invariant $\Phi$, or a calculus $\mathcal{R}$ — never appear in the final statement; they enter only inside proofs via soundness lemmas `[taken from source: MARIO trust-model.md]`.
2. **Explicit named residual premises, never `Admitted`:** Every open research obligation is an explicit hypothesis of the capstone theorem, checked mechanically by `Print Assumptions` and a spine-vs-`Unwired/` firewall `[taken from source: MARIO README.md, trust-model.md]`.
3. **Non-vacuity witness:** Prove alongside the capstone that the pinned baseline (`Open`) satisfies the exactness specification and attains the baseline cost inside $S_{\max}(c)$, ensuring the spec is non-empty `[taken from source: MARIO trust-model.md]`.
4. **Decidability, not classical LEM:** The target is not the tautological `Prop` disjunction $(\exists D, \dots) \lor \neg(\exists D, \dots)$ `[taken from source: MARIO feasibility-and-the-critiques.md §2]`, but either an **explicit constructive witness** ("Yes") or a **universally quantified impossibility theorem** ("No").

### 4.1 The decision target $\mathcal{Q}$

Fix: declared instruction set $\mathcal{I}$, memory model $\mathcal{H}$, uniform single-cycle family $\mathcal{G}$ (with validation corpus $\mathcal{F}$), baseline floor function $F_{\text{base}}(c)$ (e.g. $h_1(c)$ or $W_{\text{core}}(\mathrm{Open}, c)$), margin $\delta \in (0, 1)$, scratch cap $S_{\max}(c)$, sweep cap $K_{\max}$, and traffic/miss guards $(Q_{\max}(c), D_{\mathcal{H},\max}(c))$. Define the predicate $\mathrm{MeetsTarget}(D)$ for a uniform, advice-free $D \in \mathrm{Prog}(\mathcal{I})$:
$$\mathrm{MeetsTarget}(D) :\equiv
\begin{cases}
\text{(1) Exactness:} & \forall c \in \mathcal{D}_{\text{exact}},\; \mathrm{Out}(D, c) = \mathrm{decode}(c), \\
\text{(2) Floor lowered:} & \Pr_{c \sim \mathcal{G}}\!\bigl[W_{\text{core}}(D, c) \le (1-\delta)\, F_{\text{base}}(c)\bigr] = 1 - o(1) \\
& \quad\text{and}\quad \forall c \in \mathcal{F},\; W_{\text{core}}(D, c) \le (1-\delta)\, F_{\text{base}}(c), \\
\text{(3) Resource caps:} & \forall c \in \mathcal{F}\text{ (and w.h.p. on }\mathcal{G}\text{)},\; S(D, c) \le S_{\max}(c) \;\wedge\; K_{\text{sweep}}(D, c) \le K_{\max} \\
& \quad\wedge\; Q_{\mathcal{H}}(D, c) \le Q_{\max}(c) \;\wedge\; D_{\mathcal{H}}(D, c) \le D_{\mathcal{H},\max}(c).
\end{cases}$$

### 4.2 Shape of a successful "Yes" theorem (conditional existence, backward-chain form)

`[argued]` Rather than hard-coding an N/R split (as `MT §3` does), the constructive capstone parameterises over any candidate basis $\mathcal{B}' = (\mathcal{O}', \mathrm{Valid}_{\mathcal{O}'}, \mathrm{Build}, \mathrm{Advance}, \mathrm{Emit}, \theta')$ in $\mathcal{I}$, with four explicit, independently dischargeable premises matching the Mario supplied-state-first order (`J§3.4`, `METHOD §3a`):

```text
Theorem Main_Yes (B' : Basis I)
  (H1_emit_sound : ∀ c ∈ D_exact, ∀ z : O'(c), Valid(c, z) → Emit(Advance*(z)) = decode(c))
  (H2_cons_cost  : ∀ c ∈ F [and w.h.p. G], ∀ z, Valid(c, z) →
                     W_core(Advance* + Emit, z) ≤ g_cons(θ'(c)) ∧ g_cons(θ'(c)) ≤ (1 - δ) F_base(c))
  (H3_build_sound: ∀ c ∈ D_exact, Valid(c, Build(I_0(c))))
  (H4_joint_cap  : ∀ c ∈ F [and w.h.p. G],
                     W_core(Build, c) + W_core(Advance* + Emit, Build(I_0(c))) ≤ (1 - δ) F_base(c)
                     ∧ S(B', c) ≤ S_max(c) ∧ K_sweep(B', c) ≤ K_max
                     ∧ Q_H(B', c) ≤ Q_max(c) ∧ D_H(B', c) ≤ D_H_max(c)) :
  ∃ D : Prog(I), MeetsTarget(D).
```

- **How backward regression works on this capstone `[argued]`:** `H1` + `H2` are discharged *first* on a supplied carrier $z$ (paired with empirical supplied-capability Receipt R7 before investing in acquisition); `H3` and `H4` then state the exact acquisition obligation to discharge the supplied state without relocating cost. Any particular architectural decomposition (such as an N/R split $\sigma$ with glue) is an *auxiliary lemma* that constructs a `B'` discharging `H1`–`H4`, not part of the capstone's statement.
- **External empirical receipt (outside the theorem):** Native implementation of `D` clears `PERFORMANCE-POLICY.md` against `master` at the declared worker count(s) $W$ (Receipt R7).

### 4.3 What a "No" theorem must quantify over to mean anything

`[proved/argued; incorporating REV1 §6, M103 §3, M106 §3, M108 §3–§4]` Because `H1 ∧ H2 ∧ H3 ∧ H4 ⇒ ∃ D, MeetsTarget(D)` is a *sufficient* condition for one basis $\mathcal{B}'$, refuting a premise (`¬H_i`, or refuting `PN` in `MT §3`) never proves `¬∃ D, MeetsTarget(D)` `[taken from source: REV1 §6, MT §6]`. A meaningful "No" theorem must have the universal form:
$$\forall D \in \mathfrak{A},\quad \mathrm{Exact}(D, \mathcal{D}_{\text{exact}}) \;\wedge\; \mathrm{Budgets}(D, S_{\max}, K_{\max}, \dots) \;\implies\; \Pr_{c \sim \mathcal{G}}\!\bigl[\mathrm{Cost}(D, c) > (1-\delta)\,F_{\text{base}}(c)\bigr] \ge 1 - o(1)$$
(or $\exists c \in \mathcal{G}$ in the worst-case form), accompanied by a mandatory **"Does Not Cover"** clause `[taken from source: M106 §3–§4]`. For $\mathfrak{A}$ to be non-trivial (neither tautological by fiat nor merely closing one narrow syntax):
1. **No exclusion by fiat:** $\mathfrak{A}$ must admit (a) writable working RAM up to $S_{\max}(c) = \Theta(r)$ words ($\Theta(n \log n)$ bits on random-heavy blocks — ruling out succinct-space or read-only streaming lower bounds like Sinha–Weinstein or Gamma's G2 as proofs of "No" `[taken from source: REV1 §4, FRONTIER Foundational]`), (b) global order-sensitive summaries and fine-grained input probes (such as $A = \sum i \cdot \mathrm{code}(L[i])$ or rank gaps `[taken from source: REV2 §III.6, N-18]`), (c) non-rigid exact write mechanisms (`fill_remaining`, `copy`, scattered class facts E1, packed word operations `[taken from source: N-19, REV2 §III.3]`), and (d) arbitrary output write orders (not requiring orbit-order emission or an N/R split).
2. **Uniform infinite domain:** Quantification over $\mathcal{G}$ (not finite $\mathcal{F}$), blocking hardcoded advice `[taken from source: REV1 §1, M103 §4]`.
3. **Semantically grounded currency:** Cost measured in $\mathrm{Cost}_{\mathcal{I},\mathcal{H}}$ (e.g. word-RAM instructions, external-memory/cache I/O transfers, or cell probes with charged preprocessing) — never a syntactic charge that forbids reading the input (`M108 §3` strict) or ignores bulk work (`M108 §3` lenient).
4. **Honest strength tiers `[argued]`:**
   - *Tier 1 (Calculus barrier — achieved for $\mathcal{R}$ and $\Lambda'(\mathcal{O}_{\text{hist}})$):* Closes a formal derivation calculus; guides construction by naming invariants to drop, does *not* settle general inversion.
   - *Tier 2 (Scoped execution-model tradeoff — realistic frontier target):* Lower bound in a concrete cache/I/O, pointer-machine, or bounded-sweep RAM model with charged preprocessing on $\mathcal{G}(S, m, k)$ (`REV1 §4, §6`).
   - *Tier 3 (Unconditional Word-RAM lower bound):* $\Omega(h_1)$ word-RAM time on packed inputs is open in complexity theory even for general BWT inversion and list ranking `[taken from source: CHAN#11, M100 §3c]`; a programme should not promise Tier 3 as a routine outcome.

---

## 5. Check against the sources

### 5.1 Where this formalisation matches the sources

- **Louis's intent `[taken from source: J§1, J§12, J§18, J§22, J§32, BRIEF]`:** Preserves the binary (yes/no) structural target, takes `walk → RV` as the formal calibration test (§2), forbids incremental R0/R1 edits from qualifying, separates floor-lowering from ceiling-raising (§3), and retains empirical utility as an explicit requirement (§1.3 Layer C, §3.3).
- **doomslide, *"Against Mathematics"* `[taken from source: DOOM Parts II, IV, VI, VIII, fn. 2; AM-GPT; AM-SRC §4]`:**
  - *Hearers vs Learners (Part IV):* Every obstruction carries witness, scope, escape condition, and transferable insight (the *logos*), not a bare closure verdict.
  - *Archimedes' Method (Part VI):* Separating supplied-state discovery (`H1–H2`) from full native acquisition (`H3–H4`) and preserving the transformation/descriptor algebra as a deliverable.
  - *Hilbert's 24th problem (Part II):* Formalising "new basis" via non-existence of a cost-preserving transformation into prior witnesses (§2 B1), and keeping derivations in `Type` rather than `Prop` (DOOM fn. 2; M105 §1).
  - *Boats I–III (Part VIII):* Boat I (do not accept an oracle's saying without checking the *logos*); Boat II (the Pell matrix preserving $s^2 - 2c^2 = -1$ → closed `Advance` on a new carrier $\mathcal{O}'$); Boat III (anthyphairesis = 2-interval RV induction, and Theaetetus's move from case-by-case checks to naming the invariant → replacing framework-by-framework screening with exact characterisations of $\mathcal{R}$ and fibres $U_{\mathcal{O}}$).
- **Mario sources `[taken from source: MARIO README, trust-model, feasibility; J§2, J§11–12, J§23]`:** Incorporates the Rocq repo's TCB vocabulary isolation, explicit named residual hypotheses, `Print Assumptions` / `Unwired/` auditability, non-vacuity witness, and non-LEM decidability, alongside the SSL transcript's "forbid the obvious mechanism" and "payoff first from a supplied state, discharge one premise at a time".
- **MONODROMY, Neon, and outside reviews `[taken from source: M96–M110, N-1–N-19, REV1, REV2]`:** Incorporates G1's scoped provenance proof (`M103`), Theorem N′ and complete-observation fibres (`M107`, `N-18`), sharp disjoint-span blindness (`N-18 Lem 18.2`), the three neighbourhood decoders (`N-19`), `Build/Advance/Emit` (`REV2 §II(c)`, `M107 §4`), the uniform family $\mathcal{G}(S, m, k)$ (`M103 §4`), the i9 memory model $\mathcal{H}$ (`N-15`), and Record 108's charge-convention diagnosis.

### 5.2 Where this formalisation diverges from or adds to the sources

1. **Replaces Record 108 §4's strict pass-charging decision with Layer A execution accounting `[argued]`:** Record 108 §4 froze a strict convention ("charging every pass that reads a row's record to determine something about it") under which MONODROMY itself noted no known or proposed mechanism can reach "yes" by convention. Here, scalar $c_1$ is demoted to Layer B (diagnostic of $\mathcal{R}$), and Layer A ($\mathrm{Cost}_{\mathcal{I},\mathcal{H}}$) prices `Build`, `Advance`, `Emit`, and `Glue` jointly without making raw input reading illegal.
2. **Removes the N/R split $\sigma$ from the Main Theorem's statement `[argued]`:** `MT §3` hard-codes `σ`, `DN`, `DR`, and `Pglue` into `theorem main`. Following `REV1 §1` and `MARIO trust-model.md`, $\sigma$ is removed from the top-level theorem and treated strictly as an optional proof-internal decomposition.
3. **Stratifies `SingleCycle ⊊ ValidBWT ⊊ Column` explicitly `[proved]`:** Adds periodic valid bzip2 blocks ($T = u^k, k \ge 2$) back into the domain picture rather than equating "valid" with "single cycle".
4. **Closes the bulk-sweep and read-back-output loopholes formally `[argued]`:** Operationalises $K_{\text{sweep}}$ and write-once $\mathsf{Out}$ (§1.3 Layer A), which `MT §1` leaves informal.
5. **Untangles the two Mario sources `[proved — textual check]`:** Explicitly separates `tra38/sm64-wmotr-abc-proof` (WMotR impossibility proof in Rocq) from the SSL "Astral Rejection" backward-synthesis work.

### 5.3 Contradictions and discrepancies across the records

Every item below is `[proved — textual/mathematical comparison]` from the pinned commits:

1. **Conflation of two distinct Super Mario 64 projects (`J§2`, `BLOG §2` vs `MARIO README.md`):** `J§2` attributes Shifting Sand Land's "Astral Rejection" (spawning displacement, 973-unit stopped crawl, supplied-state backward construction) to `tra38/sm64-wmotr-abc-proof`. In fact, `tra38/sm64-wmotr-abc-proof` is a **Wing Mario over the Rainbow (WMotR)** Rocq/CompCert *impossibility* proof (GOAL 1: no-A ⇒ no-fly; GOAL 2: WMotR requires flying) with no Astral Rejection or supplied-state backward search. The programme's two Mario borrowings come from *two different projects*.
2. **Corpus size: 238 vs 232 distinct blocks (`MT §1`, `HANDOFF`, `M103 §4`, `J§25` vs `N-2 §0`, `REV2 §I`):** `MT §1` and `J§25` describe $\mathcal{F}$ as "238 blocks" (`232 + 6`), whereas `N-2` verified an exact 6/6 identity between the 6 admitted blocks and 6 of the 232 distinct `#805` blocks.
3. **"Fixed before any data" (`BLOG §7`, `HANDOFF` vs `MT §1 L19`, `REV2 §I`):** `HANDOFF` says parameters were fixed "before any data" and `BLOG §7` says "before any candidate was tested", whereas `MT §1 L19` explicitly discloses having seen `M96`'s 75–85% hex figures on the 6 blocks before choosing $\delta = 1/2$ (fixed before `N-0`, not before all data).
4. **Open's $c_1$ value (`M96 §4`, `M98 L52`, `J§12`, `FRONTIER L34` vs `M102 §3`, `MT §4.1`, `N-1`, `BLOG §7`):** `M96 §4`, `J§12`, and `FRONTIER L34` state Open meets G1's floor $h_1$ exactly (`M98 L52`: $h_1 + \text{induction w1 pieces}$). After `N-1` break 3, `M102 §3` and `MT §4.1` adopted the stricter link-charging convention under which $c_1(\text{Open}) \ge m \in [1.21, 3.32]\,h_1$ (and $c_1(\text{walk}) = n$), requiring a "yes" to remove most of $c_1W$ as well (`REV2 §IV`). The older docs (`FRONTIER`, `J§12`) were not reconciled.
5. **Withdrawn "work-bound, not latency-bound" claim still live in `MT §1a` (`MT §1a L33` vs `J§29`, `M107 §1`, `REV2 §I–II(e)`):** `MT §1a L33` (at pinned `f86a4d3b`) still cites `toy-transfer` (20 Nov) as proving misses are not emit's cost, even though `J§29` and `M107 §1` withdrew that claim after `REV2` pointed out `toy-transfer` L187–208 itself corrected the early negative (branchless 16B appends + 8 lanes cut emit to $0.55\text{–}0.78\times$).
6. **Copy plans on novel random nibbles (`M102 §2.4`, `N-8 §4` vs `M108 §1 BREAK`, `N-12/N-16`, `CHAN#13–14`):** `M102 §2.4` and `N-8` claimed copy schemes cannot touch novel random nibbles; `M108 §1` broke that claim for short birthday-collision copies (`min_len = 2`: $z/h_1 = 0.48\text{–}0.64$, `N-12/N-16`). Yet `CHAN#14` then cites $z/h_1 = 0.54\text{–}0.64$ as satisfying "small where runs fail", even though it fails $\delta = 1/2$ on 5/6 blocks even with a free oracle and rises to $z/h_1 = 1.27\text{–}1.51$ at `min_len = 8`.
7. **Strict pass-charging freeze vs construction pivot (`MT §1a L37`, `M108 §4` vs `CHAN#13–14`, `J§32 R4`):** `M108 §4` and `MT §1a L37` freeze strict pass-charging (making the frozen "yes" unreachable by convention), while `CHAN#13–14` pivot to construction and `J§32 R4` simultaneously demands beating G1's floor *under record 108's convention* — an internal contradiction.
8. **Type errors and stale clauses inside `MT` (`MT §1a, §3, §5, §6` at `f86a4d3b`):**
   - `theorem main` L92 references `c₁W c`, which is unbound in its signature (`N-14 §3`);
   - L91 bounds state by `≤ S c` (`openPeakScratch c`), while L94 concludes asymptotic `D.state c ∈ O(c.r)` (`N-9 §2`, `N-14 §3`);
   - §6 L165 defines a "no" over $\mathcal{R} \cup E1 \cup E3 \cup E2\text{-local} \cup E2\text{-}O(1)$, omitting $\Lambda_{\text{agg}}$ and global $E2$, contradicting §6 L161 and §5 L153 (where E1 bulk uses are open);
   - §6 L161 still prints `|𝒫| + 3|D| ≤ ν′` after `M108 §1` accepted `N-18 Lem 18.2`'s sharp `1|D| / 2|D|`;
   - §1a numbers clauses `1, 2, 3, 4, 6, 6b, 7, 5`.
9. **Flip distance & blindness evolution (`FRONTIER L228`, `HANDOFF L18` vs `M101 §1` vs `N-9`, `M107` vs `N-18`, `M108`):** `HANDOFF L18` conflated Epsilon's $d_H = 3$ sensitivity with count-preserving flips; `M101` proved valid count-preserving flips have $d_H = 2$ (changing 3 LF successors). Blindness for $D$ moved $2|D|$ (`M101`, broken for run records) → $3|D|$ (`N-9`, `M107`) → sharp $1|D|$ (raw/pointwise LF) and $2|D|$ (full run+LF) (`N-18 Lem 18.2`, `M108 §1`).
10. **Valid column = `SingleCycle` vs `ValidBWT` (`M101 L3`, `M103 §1` vs `PROOF-PROG §1.1–§1.2`):** `M101` and `M103` define "valid" as single $n$-cycle, silently excluding valid periodic bzip2 blocks ($T = u^k, k \ge 2$) and corrupt `Column`s required by the Rust contract.
11. **Order-sensitive summaries and event floors (`M107 §3` vs `N-18 Lem 18.1`, `M108 §1`):** `M107 §3` claimed no event floor survives for $A = \sum i \cdot \mathrm{code}(L[i])$; `N-18 Lem 18.1` proved the exact fibre floor $c_1 \ge h_1 - |J_1 \cap U_{\mathcal{O}}|$ holds for every deterministic $\mathcal{O}$ (accepted in `M108 §1`).
12. **Clause 3 whole-decoder utility at 10 workers (`MT §1a cl. 3` vs `MT §7`, `J§30`):** Clause 3 demands beating `master` on whole-decoder time without specifying $W$, yet `bench-supplied-codes` (`MT §7`) shows an ideal emit-only oracle gets $0.988\times$ at the default $W = 10$ (vs $0.838\times$ at $W = 1$), rendering clause 3 practically unreachable at $W = 10$ for any emit-only basis change.

---

## 6. Critique of the current formulation, with corrected statements

### 6.1 The frozen target (`MT §1, §1a, §3, §6`)

| Defect | Diagnosis | Corrected statement |
|---|---|---|
| **1. Dichotomous currency ($c_1$ + cl. 2)** | `[proved; M108 §2–§3]` "Aggregate" is undefinable over functions; strict pass charging makes "yes" impossible by fiat; lenient pass charging makes $K$ decide and lets doubling win; $c_1$ is incomparable with machine time (`M105 §2`, `REV1 §2`). | Replace $c_1$ as the target currency with Layer A execution cost $W_{\text{core}}(D, c)$ in a declared instruction set $\mathcal{I}$ (charging `Build`, `Advance`, `Emit`, and `Glue` jointly, with raw I/O $C_{\text{io}}$ separated) plus $(\mathcal{H}, S_{\max}, K_{\max})$ caps (§1.3). Retain $c_1$ strictly as a Layer B diagnostic for $\mathcal{R}$ and $\Lambda'$. |
| **2. Ambiguous $K = 2$ and unprotected `out[]`** | `[proved; REV1 §1]` Input-pass count is vacuous when $S = \Theta(r)$; `out[0..n)` can be abused as uncharged scratch or read-back memory. | Bound $K_{\text{sweep}}$ (total sequential sweeps over $\Omega(r)$-sized arrays, input + working) and $Q_{\mathcal{H}}$ (total cache-line transfers). Declare $\mathsf{Out}[0..n)$ write-once, or charge every read/overwrite of $\mathsf{Out}$ in $W_{\mathcal{I}}, Q_{\mathcal{H}}$ and count live read-back regions in $S$. |
| **3. Architecture-locked `theorem main` with unbound `c₁W` and mismatched `state`** | `[proved; REV1 §1, N-9, N-14]` Imposes an N/R split on the conclusion; `c₁W` is unbound; concludes `O(c.r)` from `≤ S c`; conflates `Column`, `ValidBWT`, `SingleCycle`. | Replace `theorem main` with the architecture-agnostic Mario capstone `Main_Yes` (§4.2, `H1`–`H4` + non-vacuity witness `Open`), with explicit domain $\mathcal{D}_{\text{exact}}$ and `D.state c ≤ S c`. Demote N/R splits to optional instantiation lemmas. |
| **4. Mis-scoped "No" (§6) and stale §1a L33** | `[proved; REV1 §6, M107 §1, N-18]` §6 L165 omits $\Lambda_{\text{agg}}$ and global $E2$; §6 L161 uses loose $3\lvert D\rvert$; §1a L33 asserts the withdrawn work-bound claim. | Define "No" universally over a declared uniform execution class $\mathfrak{A}$ with a mandatory "Does Not Cover" boundary (§4.3). Synchronise §1a L33 with `M107 §1` and §6 L161 with `N-18 Lem 18.2` ($1\lvert D\rvert$ / $2\lvert D\rvert$). |
| **5. Unspecified worker count in clause 3** | `[proved; MT §7, J§30]` Emit-only oracle achieves $0.838\times$ at $W = 1$ but $0.988\times$ at $W = 10$; an unindexed "whole-decoder time" gate conflates BWT-kernel merit with front-end/bandwidth saturation. | Split clause 3 into: **(3a) Kernel utility** (beats `master`'s BWT kernel by $\ge \eta_{\text{kern}}$ at $W \in \{1, 10\}$) and **(3b) Whole-decoder utility** (beats `master` end-to-end by $\ge \eta_{\text{e2e}}$ at explicitly pinned worker count(s) $W$). |
| **6. Unclear $\delta = 1/2$ baseline & 238/232 mismatch** | `[proved; REV2 §IV, N-2]` Under strict $c_1$, Open pays $\ge m$, so $c_1 \le h_1/2$ demands removing $\ge 79.6\%$ of retained links (pooled), not halving single-row steps. $\mathcal{F}$ has 232 distinct blocks, not 238; $\mathcal{G}(S, m, k)$ lacks distribution parameters. | Pin the baseline denominator explicitly ($F_{\text{base}} = W_{\text{core}}(\mathrm{Open}, c)$ or $c_1\rvert_{J_1} \le (1-\delta) h_1$ with $c_1\rvert_W$ non-increasing), set $\lvert\mathcal{F}\rvert = 232$ distinct blocks, and fix $(S, m, k)$ scaling and failure probability $\varepsilon(n) = o(1)$ for $\mathcal{G}$. |

### 6.2 Conditions in channel entries #13–#14 (`CHAN#13`, `CHAN#14`)

| Defect | Diagnosis | Corrected statement |
|---|---|---|
| **1. Entropy vs instruction count in Condition 2** | `[proved]` Asking for a parameter "small where runs fail (novel random content)" is ambiguous: by Shannon entropy, $N$ i.i.d. hex nibbles require $\ge 4N$ bits in *every* representation. Citing supplied LZ77 ($z/h_1 = 0.54\text{–}0.64$) as meeting Condition 2 confuses pointer count with entropy and ignores that $0.54\text{–}0.64 > 0.50$ fails $\delta = 1/2$ on 5/6 blocks (`N-12/N-16`). | Require a structural parameter $\theta'(c)$ governing **non-I/O instruction count** $W_{\text{core}}$ in $\mathcal{I}$ such that $O(f(\theta'(c))) \le (1-\delta)\,F_{\text{base}}(c)$ on $\mathcal{G}$ and $\mathcal{F}$ while respecting $\mathrm{bits}(z) \ge H(T)$ — explicitly recognising that beating $h_1$ on novel random content requires $\omega(1)$ random symbols per instruction (word-packing or multi-byte phrases), not $o(N)$ entropy. |
| **2. Uncharged build passes in Condition 3** | `[proved; M108 §3, M110 §3]` Asking for acquisition "in $O(1)$ streaming passes, without a row-by-row LF walk" leaves bulk-pass charging and scratch $S$ unspecified; $T^2$ and sweep composition satisfy it in prose while spending $\ge m$ compositions. | Require `Build` in declared $\mathcal{I}$ to respect $(K_{\text{sweep}}, S_{\max}, Q_{\mathcal{H}})$ AND satisfy the **joint** bound $W_{\text{core}}(\mathrm{Build}) + W_{\text{core}}(\mathrm{Advance}+\mathrm{Emit}) \le (1-\delta)\,F_{\text{base}}(c)$. |
| **3. Blanket dismissal of negative results (`CHAN#14`)** | `[argued; FRONTIER, DOOM Boat III]` Stating "a closure with no construction attempt does not [count as progress]" contradicts the yes/no charter and risks unguided guessing without updated escape conditions. | Deprioritise redundant single-framework screens (Boat I); keep **structural obstruction theorems** that widen the quantified impossibility class $\mathfrak{A}$ or sharpen escape invariants (Boat III / Theaetetus) as first-class progress. |

### 6.3 Critique rubric R1–R7 (`J§32`)

| Item | Defect | Corrected rubric item |
|---|---|---|
| **R1 New object** | `[argued]` Purely ontological ("other than rows or intervals"); vulnerable to renaming (`M104`, `N-6`) and unclear on objects that generalise intervals (e.g. packed bitvectors). | **R1'**: Carrier $\mathcal{O}'(c)$ and closed `Advance`/`Emit` defined in `Type`; proved (or explicit obligation) that executions have no cost-preserving lowering into $\mathcal{R}$ and `Advance` does not enumerate individual $J_1$ rows to apply $\mathrm{LF}$. |
| **R2 New cost parameter** | `[argued]` Same entropy-vs-instruction ambiguity; 6-block-only check invites overfitting; "small" lacks a quantitative threshold. | **R2'**: Explicit cost law $W_{\text{core}} = O(f(\theta'(c)))$ in $\mathcal{I}$; respects $\mathrm{bits}(z) \ge H(T)$; achieves $f(\theta'(c)) \le (1-\delta) F_{\text{base}}(c)$ analytically on $\mathcal{G}$ and measured on the 6 blocks (with script ready for all 232 blocks of $\mathcal{F}$). |
| **R3 Acquisition without old mechanism** | `[proved]` "$O(1)$ streaming passes" is undefined, scratch $S$ is omitted, and bulk $\Theta(m)$ builds are not excluded. | **R3'**: Finite $\mathcal{I}$, typed memory (write-once $\mathsf{Out}$, $S \le S_{\max}(c)$), and explicit `Build` from $I_0(c)$ declared; all instructions and sweeps of `Build` counted in $(W_{\text{core}}, K_{\text{sweep}}, Q_{\mathcal{H}})$; neither hidden LF walk nor $\ge F_{\text{base}}(c)$ bulk per-row compositions. |
| **R4 Floor lowered, not relocated** | `[proved; M108 §3–§4]` **Self-contradictory:** invokes Record 108's strict pass-charging convention, under which reading every width-1 byte already forces $c_1 \ge 0.93\,h_1$ for *every* exact decoder. | **R4'**: In the declared execution model $(\mathcal{I}, \mathcal{H})$ with raw I/O $C_{\text{io}}$ separated and every non-I/O instruction in `Build + Advance + Emit + Glue` charged, joint $W_{\text{core}}(\mathcal{B}', c) \le (1-\delta)\,F_{\text{base}}(c)$ (no relocation into `Build`). |
| **R5 Proof form** | `[argued; MARIO trust-model, N-12]` Omits correctness-vs-cost separation, small-instance exactness verifier (which would have caught `N-12`'s bug), and TCB/assumption audit. | **R5'**: Mario-style conditional theorem (`H1`–`H4`) separating `Exact` from cost, every premise tagged `proved / verified (count) / measured / open`, paired with an executable small-instance verifier (including periodic/multi-cycle tests if total) and explicit TCB list; near misses include a complete phase-by-phase cost ledger. |
| **R6 Not incremental** | `[argued]` Blacklist names only internal mibz2 kernels, omitting external prior art (`LF²`, `libsais`, Kempa, Nishimoto–Tabei, list ranking); overlaps R1/R4. | **R6'**: Formally classified R2 (via R1'/R4') or R3 (explicit problem change), with mandatory prior-art comparison against both internal kernels and external BWT-inversion literature (`CHAN#11`), naming the exact difference. |
| **R7 Utility & Verdict** | `[argued; MT §7]` "Honest statement that none can yet be given" makes R7 vacuous; worker count $W$ is omitted; the Verdict omits R5 and R7 and leaves "R4 partial" undefined. | **R7'**: Pre-declared supplied-capability test naming $z_0$, consumer, $W \in \{1, 10\}$, and $\ge 10\%$ headroom gate (`met` / `pending-experiment` / `not-met`). **Verdict tiers:** *Supplied candidate* = R1', R2', R5', R6' + R7' supplied headroom met; *Starts to overthrow induction* = R1'–R6' met (joint build + emit below floor in $\mathcal{I}$) + R7' pending native run; *Full Yes* = all + R7' native win under `PERFORMANCE-POLICY.md`. |

---

## 7. Open choices (where the sources do not decide)

For each open choice `[argued]`, the sources leave the decision unresolved; options, recommendation, and rationale follow.

1. **Primary cost currency of the Main Theorem.**
   - *Options:* (A) Retain scalar $c_1$ with Record 108 §4's strict pass charge; (B) retain scalar $c_1$ with lenient pass charge and $K = 2$; (C) adopt concrete Word-RAM + hierarchy execution cost $\mathrm{Cost}_{\mathcal{I},\mathcal{H}}$ (Layer A) as the theorem currency, keeping $c_1$ as a Layer B diagnostic for $\mathcal{R}$.
   - *Recommendation:* **(C).** *Reason:* Record 108 proved (A) is impossible by fiat and (B) collapses to pass-counting over doubling; only (C) charges acquisition and emission in one honest currency and can express genuine packed/multi-symbol word-RAM speedups without letting relocations pass.
2. **Exactness domain $\mathcal{D}_{\text{exact}}$.**
   - *Options:* (A) `SingleCycle` only; (B) `ValidBWT` (primitive + periodic); (C) total `Column` ($1 \le n \le 900{,}000, p < n$).
   - *Recommendation:* **Two-tier: require `Exact` on total `Column` (C) for production admission (via a cheap guard/fallback if needed), while stating structural cost bounds on `SingleCycle` / $\mathcal{G}(S, m, k)$ (A).** *Reason:* Preserves `PROOF-PROG §1.1`'s corrupt-block CRC contract and handles legal periodic bzip2 blocks without cluttering single-cycle cost theorems (100% of Wikidata's 232 blocks are single-cycle `[taken from source: N-10 §0]`).
3. **Worker count $W$ and boundary of utility (Clause 3 / R7).**
   - *Options:* (A) Whole-decoder time at $W = 10$ only; (B) BWT-kernel time at $W \in \{1, 10\}$ plus whole-decoder time at $W = 1$; (C) whole-decoder time at $W = 10$ with R3 (construct + emit + front-end) admitted.
   - *Recommendation:* **(B) for the R2 BWT-inversion theorem, paired with (C) for production shipping.** *Reason:* `bench-supplied-codes` (`MT §7`, `J§30`) showed that an emit-only oracle gets $0.988\times$ whole-decoder time at $W = 10$ vs $0.838\times$ at $W = 1$. Gating R2 mathematics on $W = 10$ whole-decoder time conflates BWT inversion with an orthogonal 10-worker front-end/bandwidth bottleneck.
4. **Standing of word-level packing (packed Word-RAM) vs unit-price tuning (residual (b)).**
   - *Options:* (A) Exclude all $O(1)$-word packing from lowering the floor; (B) admit genuine packed algorithms achieving $W_{\text{core}} \le (1-\delta) F_{\text{base}}$ in $\mathcal{I}_{\text{RAM}}$ (processing $\omega(1)$ symbols per word operation, e.g. $O(n \log \sigma / w)$) as floor-lowering, while keeping fixed-factor SIMD/lane scheduling (R0/R1) on residual (b).
   - *Recommendation:* **(B).** *Reason:* By Shannon entropy, novel random hex nibbles ($4\text{ bits/row}$) cannot be compressed below $4 h_{1,\text{rand}}$ bits; reducing non-I/O instruction count below $h_1$ on novel random content *must* process multiple symbols per machine word (and `CHAN#11` notes packed BWT inversion is genuinely unasked in the literature).
5. **Single full-block margin $\delta = 1/2$ vs ladder sub-targets.**
   - *Options:* (A) Single binary gate $\delta = 1/2$ universally on $\mathcal{F}$ and $\mathcal{G}(S, m, k)$ (requires breaking random content); (B) keep (A) as the sole condition for "Full Yes", while formally recognising a **structured-subfamily ladder** $\mathcal{G}_0 \subset \mathcal{G}_1 \subset \dots \subset \mathcal{G}$ (from Sturmian/perfectly-clustering up to non-GUID templates) as intermediate rungs.
   - *Recommendation:* **(B).** *Reason:* Keeps the major-shot standard unrelaxed ($\delta = 1/2$ on random-heavy real blocks) while giving the Sturmian/Mario ladder (`J§3.3`, `J§8`) a formal place to record intermediate theorems without mislabelling them as full Wikidata wins.
6. **Lean 4 formalisation scope and sequencing (`mibz2-lean`).**
   - *Options:* (A) Formalise $\mathcal{R}$ and G1 first (600–1,000 LOC, `M98 §5`); (B) formalise only the tiny, import-isolated `Spec` (`Column`, `ValidBWT`, `SingleCycle`, `decode`, `T1`, and the non-vacuity witness `Open`) first, deferring cost-theorem formalisation until Layer A $(\mathcal{I}, \mathcal{H})$ is frozen; (C) keep all formalisation on paper until a candidate passes a supplied-state payoff test.
   - *Recommendation:* **(B), combined with (C) for candidate-specific proofs.** *Reason:* Matches `MARIO trust-model.md` (freeze and audit the small `Spec` keystone early, in `Type`), avoids spending Lean effort polishing G1 inside a calculus that Record 108 showed is not the decisive cost boundary, and withholds candidate formalisation until supplied headroom clears R7'.
7. **Scope of R3 (format change vs standard-bzip2 upstream pipeline).**
   - *Options:* (A) Strictly standard bzip2 from $I_0(c) = (\mathrm{Runs}(L), n, p)$ only; (B) admit upstream standard-bzip2 stages (Huffman/MTF/RLE2 bitstream) as R3 inside this programme, keeping format changes (non-bzip2 codecs) out of scope.
   - *Recommendation:* **(B).** *Reason:* `mibz2` is a bzip2 decoder (`BRIEF`); changing the wire format solves a different problem, whereas leveraging upstream bzip2 structure preserves exact bzip2 compatibility while keeping `FRONTIER` FQ4 honestly open.
