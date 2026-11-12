---
title: "Reconstructing text from compact programs"
image: assets/social/blog-2026-11-12-rauzy-technical-companion.png
description: "This technical companion traces how compact word programs reconstruct text from the BWT, including the costs and limits of building and expanding them."
---

<div class="post-hero" markdown>
<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Reconstructing text from compact programs

This technical companion traces how compact word programs reconstruct text from the BWT, including the costs and limits of building and expanding them.
{: .post-dek }

12 November 2026 · 32 min read · [Research in progress](#10-where-the-live-diagnostic-fits)
{: .post-meta }

</div>
<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>
</div>

[A new way back from the BWT](2026-11-10-rauzy-veech-reconstruction.md) already introduced the interval-exchange structure and Rauzy–Veech machinery behind Cells, an experimental inverse transform. This technical companion develops the reconstruction programme in full: how compact word rules can recover the original text, what building and expanding those rules costs, and what the evidence means for a complete decoder. It preserves the 12 November report’s worked constructions, memory analysis and qualifications, keeping endpoint Cells separate from delimiter-based first-return research. The [8 November mathematical article](2026-11-08-inverse-bwt-navigation.md) established the underlying run translations.

The Burrows–Wheeler transform (BWT) reorders a block into the last column of its sorted cyclic rotations and records the original row. A run is a maximal stretch of equal bytes in that column. LF, the last-to-first map, is a translation on each run; induction composes those translations into return maps and records their exact emitted words. MONODROMY is the project’s mathematical research programme, whose [journal](https://github.com/lmmx/mibz2/blob/2635f04e9155dd31388b24fd3f7248f87b3ddcda/docs/journal/monodromy/2026-11-10-rauzy-veech.md) supplies the formal construction. Here the mathematical detail is retained alongside the distinction between a counted reduction and a measured decoding gain.

<style>
.md-typeset #rv-levels td:first-child,
.md-typeset #rv-storage td:first-child,
.md-typeset #rv-history td:first-child,
.md-typeset #rv-arms td:first-child { white-space: normal; }
.md-typeset #rv-cells-pseudocode pre, #rv-cells-pseudocode code { white-space: pre-wrap; }
@media (max-width: 600px) {
.md-typeset #rv-history th, .md-typeset #rv-history td { padding: .6em .65em; overflow-wrap: anywhere; }
}
</style>

## Executive summary

The central idea is more radical than making the conventional Burrows–Wheeler inverse permutation faster.

**We are investigating whether we can reconstruct the original text from a compact description of transformations of that permutation, without first materialising and then traversing the full permutation in the conventional way.**

The mathematical starting point is that the BWT's LF permutation is not an arbitrary permutation: on each maximal equal-byte run, it acts as a translation of an integer interval. This makes it a *discrete interval exchange transformation*.

Rauzy–Veech induction gives us a way to repeatedly reduce such a transformation to a smaller interval exchange, while recording the return itineraries of the removed pieces. Those itineraries can be represented as concatenation rules.

In principle, we can exchange:

- An $n$-entry permutation table followed by $n$ dependent permutation lookups;
- For operations on $r$ run intervals, construction of a compact word grammar, and sequential reconstruction of the output.

Here $n$ is the BWT block length and $r$ the number of maximal equal-byte runs, often much smaller than $n$.

This is not merely theoretical. The original **Cells** implementation demonstrated a promising research-kernel result on Wikidata. A separate **first-return phrase** construction later passed an exact real-block test with substantially fewer counted traversal operations.

What we have *not* yet established is that either approach makes the complete production decoder faster after charging all construction, memory-management, copying, cache and downstream costs.

**Our current evidence, at three distinct levels**

<div id="rv-levels" markdown>

| Mathematics | Research algorithms | Production speedup |
|---|---|---|
| **Established:** exact first-return reconstruction invariants, with finite correctness tests | **Promising:** compressed reconstruction programs and fewer counted visits on selected inputs | **Unproven:** whole-decoder timing and memory effects remain unresolved |

</div>

A particularly important distinction for understanding the recent work is that **we have two related but different induction algorithms**:

1. **Endpoint Rauzy induction (Cells):** successively removes a terminal portion of the current interval exchange, producing a grammar for the selected permutation cycle.
2. **Delimiter-based first returns:** retains chosen rows and constructs a new exchange describing how trajectories move between those rows, together with the emitted words.

They share the same first-return mathematics, but they use different inducing sets, algorithms, state representations and cost profiles. The subsequent experiments with labelling, reduced dynamics, boosted dynamics and combinatorial surgery explored how to make these constructions cheaper.

---

## 1. What the conventional decoder actually computes

Bzip2 decoding first reconstructs the BWT last column $L$, its byte counts, and a distinguished row called `orig_ptr`.

For each byte value $c$, let

$$
C(c)=\sum_{d<c}\#\{i:L[i]=d\}.
$$

The LF permutation is

$$
\pi(i)=C(L[i])+\#\{j<i:L[j]=L[i]\}.
$$

This maps row $i$ in the last-column order to the corresponding row in the first-column order. Equal bytes retain their relative order.

The inverse permutation, $\Psi=\pi^{-1}$, is what the existing inversion kernel effectively traverses to produce bytes in forward text order.

The conventional algorithm therefore has two significant components:

**Permutation construction.** Build a mapping for all $n$ rows, ordinarily using a counting-sort-style scatter into a table.

**Permutation traversal.** Starting from the distinguished row, follow $n$ inverse links, emitting one byte per link. The next table address depends on the preceding lookup.

That second dependency is especially important. Even when a table fits within last-level cache, its access pattern is not the same as reading a sequential array.

### What resides in memory today

The production representation is a `Vec<u32>` of packed entries:

$$
T[s]=L[s]\;|\;(\Psi(s)\ll8).
$$

Each entry contains an 8-bit byte and a 24-bit successor field. The actual walker advances to $\Psi(s)$ before emitting that entry’s low byte, so the next emitted byte is $L[\Psi(s)]$. This matches the equivalent first-column convention used in the earlier navigation article; the stored low byte here is $L[s]$.

At the maximum usual bzip2 block length:

| BWT rows | Permutation table |
|---:|---:|
| 900,000 | **3.6 MB** (4 bytes per row, decimal) |

The current production `Segmented` kernel is already considerably more sophisticated than a single serial linked-list traversal. It uses multiple independent traversal lanes, segment starts, software prefetching and segmented assembly to expose memory-level parallelism.

But the fundamental object remains the full, row-indexed permutation table. Independent lanes hide some memory latency; they do not eliminate the table or the dependency within each lane.

**Our mathematical research asks whether that representation is necessary in the first place.**

Source: [MONODROMY orientation](https://github.com/lmmx/mibz2/blob/2635f04e9155dd31388b24fd3f7248f87b3ddcda/docs/team/MONODROMY-ORIENTATION.md), especially §§2–4.

---

## 2. The mathematical transformation

### 2.1 LF is piecewise translation

Partition $L$ into its maximal equal-byte runs:

$$
L=\rho_1\rho_2\cdots\rho_r,
$$

with lengths $\lambda_1,\ldots,\lambda_r$.

Within a single run of byte $c$, the occurrence count increases by one whenever the row index increases by one. Consequently,

$$
\pi(i+1)=\pi(i)+1
$$

whenever $i$ and $i+1$ belong to the same run.

Thus, for an interval $I_\alpha$ representing one run,

$$
\pi(x)=x+\delta_\alpha,\qquad x\in I_\alpha.
$$

Here an interval $[a,b)$ consists of the integer rows $a,a+1,\ldots,b-1$; the continuous bars below display those discrete lengths. Each run maps to an interval of equal length. There are two natural orderings of these intervals:

- **Top order:** the runs' order in $L$, which partitions the domain.
- **Bottom order:** their images in first-column order, equivalently the stable byte-sorted order of runs.

<figure class="bf-fig" id="rv-exchange">
<p class="bf-title">Two orders, one interval exchange</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 450" role="img" aria-labelledby="rv-exchange-title rv-exchange-desc"><title id="rv-exchange-title">Two orders, one interval exchange</title><desc id="rv-exchange-desc">Top A length 2, B length 3, C length 1; bottom C,A,B on the same 0 to 6 scale. A and B translate by +1, C by −5.</desc><text x="26" y="30" class="bf-t-strong" text-anchor="start" style="font-size:24px">Same coordinate domain [0, 6)</text><text x="26" y="66" class="bf-t" text-anchor="start" style="font-size:22px">Top: domain intervals</text><rect x="66" y="86" width="136" height="46" class="bf-card-hi" /><text x="134.0" y="116" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><rect x="202" y="86" width="204" height="46" class="bf-pill-ready" /><text x="304.0" y="116" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><rect x="406" y="86" width="68" height="46" class="bf-pill-next" /><text x="440.0" y="116" class="bf-t-strong" text-anchor="middle" style="font-size:24px">C</text><text x="26" y="315" class="bf-t" text-anchor="start" style="font-size:22px">Bottom: image intervals</text><rect x="66" y="240" width="68" height="46" class="bf-pill-next" /><text x="100.0" y="270" class="bf-t-strong" text-anchor="middle" style="font-size:24px">C</text><rect x="134" y="240" width="136" height="46" class="bf-card-hi" /><text x="202.0" y="270" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><rect x="270" y="240" width="204" height="46" class="bf-pill-ready" /><text x="372.0" y="270" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><path d="M134 138L202 232" class="bf-arrow" /><path d="M197 223L202 232L207 223" class="bf-arrow" transform="rotate(-35.882137 202 232)"/><path d="M304 138L372 232" class="bf-arrow" /><path d="M367 223L372 232L377 223" class="bf-arrow" transform="rotate(-35.882137 372 232)"/><path d="M440 138L100 232" class="bf-arrow" /><path d="M95 223L100 232L105 223" class="bf-arrow" transform="rotate(74.545444 100 232)"/><text x="66" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">0</text><text x="134" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">1</text><text x="202" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">2</text><text x="270" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">3</text><text x="338" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">4</text><text x="406" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">5</text><text x="474" y="354" class="bf-tm" text-anchor="middle" style="font-size:19px">6</text><text x="26" y="397" class="bf-t" text-anchor="start" style="font-size:22px">A: +1     B: +1     C: −5</text><text x="26" y="435" class="bf-t" text-anchor="start" style="font-size:22px">Translation preserves point order.</text></svg>
<figcaption>Illustrative interval exchange, not a particular BWT block. Equal letters have equal widths in both rows. Arrows join corresponding interval centres; every point of an interval receives the same translation without reversal.</figcaption>
</figure>

Instead of storing a destination for every row, we can describe the permutation using interval lengths and two permutations of the interval identifiers.

This is the first compression: **a piecewise translation specified by $r$ intervals rather than $n$ individual links.**

However, representing LF compactly does not automatically make it fast to invert. To evaluate $\pi(x)$ for arbitrary $x$, we still need to locate the interval containing $x$, determine its translation, and repeat. A binary search over run boundaries at every row could easily be worse than the original table traversal.

The distinctive contribution of induction is that we aim to avoid those repeated point queries altogether.

### 2.2 The state of an induced exchange

At an intermediate stage, let the active interval have length $m$, and let

$$
F_m:[0,m)\longrightarrow[0,m)
$$

be the first-return map of the original LF permutation to that interval.

A first-return map sends each retained point to the next point in the retained set along its original LF orbit. The original permutation may make several transitions between these visits.

We represent $F_m$ by letters $\alpha$ with four associated properties:

$$
\bigl(\lambda_\alpha,\ I_\alpha,\ J_\alpha,\ w_\alpha\bigr).
$$

Here:

- $\lambda_\alpha$ is the interval length;
- $I_\alpha$ is its domain interval in top order;
- $J_\alpha$ is its image interval in bottom order;
- $w_\alpha$ is the word emitted by the original permutation during this return excursion, stored in forward text order.

The maintained invariant is

$$
F_m(x)=x+\delta_\alpha,\qquad x\in I_\alpha,
$$

and every point in $I_\alpha$ has the same return-word content and return time $|w_\alpha|$.

Notice the subtle but important separation: **the current induced permutation tells us where retained points map; the words tell us what the original permutation did between those retained points.**

We are progressively replacing detailed trajectories with symbolic descriptions.

### 2.3 One right-end Rauzy–Veech step

Let $A$ be the last interval in top order and $B$ the last interval in bottom order, with $A\ne B$.

Both orders partition the same coordinate interval $[0,m)$. Their right endpoints have the same coordinate $m$, but the intervals ending there have different identities. Compare their lengths $\lambda_A$ and $\lambda_B$.

The induction removes the shorter terminal overlap from the active interval:

$$
m'=m-\min(\lambda_A,\lambda_B).
$$

It constructs the first-return map to $[0,m')$.

Where a transition now passes through the removed region, two previously represented excursions are composed. Their emitted words are concatenated; the original trajectory through the removed suffix is still represented.

<figure class="bf-fig" id="rv-right-end">
<p class="bf-title">A right-end cut composes returns</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 770" role="img" aria-labelledby="rv-right-end-title rv-right-end-desc"><title id="rv-right-end-title">A right-end cut composes returns</title><desc id="rv-right-end-desc">Toy exchange top B3, A5 and bottom A5, B3, total 8. Remove retained-section suffix 5 to 8. B points first go +5 then −3, returning +2. A retains width 2 and translates −3. Both resulting orders span 0 to 5.</desc><text x="26" y="30" class="bf-t-strong" text-anchor="start" style="font-size:24px">Before: m = 8; A = 5, B = 3</text><text x="26" y="65" class="bf-t" text-anchor="start" style="font-size:22px">Top / domain</text><rect x="50" y="85" width="165" height="46" class="bf-pill-ready" /><text x="132.5" y="115" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><rect x="215" y="85" width="275" height="46" class="bf-card-hi" /><text x="352.5" y="115" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><text x="26" y="165" class="bf-t" text-anchor="start" style="font-size:22px">Bottom / image</text><rect x="50" y="185" width="275" height="46" class="bf-card-hi" /><text x="187.5" y="215" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><rect x="325" y="185" width="165" height="46" class="bf-pill-ready" /><text x="407.5" y="215" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><rect x="325" y="85" width="165" height="46" class="bf-box" style="stroke:var(--bf-amber);stroke-width:3;stroke-dasharray:6 4"/><rect x="325" y="185" width="165" height="46" class="bf-box" style="stroke:var(--bf-amber);stroke-width:3;stroke-dasharray:6 4"/><path d="M325 75L325 255" class="bf-cut" style="stroke:var(--bf-amber);stroke-dasharray:5 4"/><text x="50" y="267" class="bf-tm" text-anchor="middle" style="font-size:20px">0</text><text x="215" y="267" class="bf-tm" text-anchor="middle" style="font-size:20px">3</text><text x="325" y="267" class="bf-tm" text-anchor="middle" style="font-size:20px">5</text><text x="490" y="267" class="bf-tm" text-anchor="middle" style="font-size:20px">8</text><text x="26" y="306" class="bf-t-strong" text-anchor="start" style="font-size:23px">Retain [0, 5); compose across [5, 8).</text><text x="26" y="347" class="bf-t" text-anchor="start" style="font-size:22px">Affected B point: 1 → 6 → 3</text><text x="26" y="382" class="bf-t" text-anchor="start" style="font-size:22px">F′(1) = F(F(1)) = 3</text><text x="26" y="429" class="bf-t-strong" text-anchor="start" style="font-size:24px">After: same scale; m′ = 5</text><text x="26" y="464" class="bf-t" text-anchor="start" style="font-size:22px">Top / domain</text><rect x="50" y="484" width="165" height="46" class="bf-pill-ready" /><text x="132.5" y="514" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><rect x="215" y="484" width="110" height="46" class="bf-card-hi" /><text x="270.0" y="514" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><text x="26" y="565" class="bf-t" text-anchor="start" style="font-size:22px">Bottom / image</text><rect x="50" y="585" width="110" height="46" class="bf-card-hi" /><text x="105.0" y="615" class="bf-t-strong" text-anchor="middle" style="font-size:24px">A</text><rect x="160" y="585" width="165" height="46" class="bf-pill-ready" /><text x="242.5" y="615" class="bf-t-strong" text-anchor="middle" style="font-size:24px">B</text><text x="50" y="664" class="bf-tm" text-anchor="middle" style="font-size:20px">0</text><text x="160" y="664" class="bf-tm" text-anchor="middle" style="font-size:20px">2</text><text x="215" y="664" class="bf-tm" text-anchor="middle" style="font-size:20px">3</text><text x="325" y="664" class="bf-tm" text-anchor="middle" style="font-size:20px">5</text><text x="490" y="664" class="bf-tm" text-anchor="middle" style="font-size:20px">8</text><text x="26" y="706" class="bf-t" text-anchor="start" style="font-size:22px">B: F′(x) = x + 2; A: F′(x) = x − 3</text><text x="26" y="744" class="bf-t" text-anchor="start" style="font-size:21px">New B word: wA wB; time: |wB| + |wA|</text></svg>
<figcaption>A concrete two-interval example: before the cut, B maps [0,3) to [5,8), then A maps that image to [2,5). After the cut, B has width 3 and A width 2. The dashed suffix is excluded from the <em>section</em>, not forgotten from the trajectory. LF visits B then A; the stored forward-text word is w<sub>A</sub>w<sub>B</sub>, the reverse itinerary convention of the journal. This is one illustrative induced exchange, not a measured block.</figcaption>
</figure>

In the convention of the MONODROMY construction, the new rule is

$$
N\longrightarrow w_Aw_B.
$$

The update depends on which interval wins:

| Case | Interval-length change | Symbolic change |
|---|---|---|
| $\lambda_A>\lambda_B$ | Shorten $A$ by $\lambda_B$ | Replace $w_B$ by $N$; change bottom order |
| $\lambda_A<\lambda_B$ | Shorten $B$ by $\lambda_A$ | Replace $w_A$ by $N$; change top order |
| $\lambda_A=\lambda_B$ | Consume the equal terminal pieces | Replace $w_B$ by $N$, delete $A$ |

For a top win, move $B$ immediately after $A$ in the bottom order; for a bottom win, move $A$ immediately after $B$ in the top order. In the equal-length case, $B$ takes $A$'s bottom-order place. If the two terminal labels are already the same, that interval closes: its induced map is the identity, and its word records the corresponding cycle.

These are transformations of a **weighted interval exchange with symbolic return words**. An itinerary substitution must preserve the destination, return time, emitted bytes and marked occurrence together. In this convention LF reads the stored word backwards: an affected trajectory visits B then A, so its forward-text rule is $w_Aw_B$, not $w_Bw_A$.

They are not arbitrary transpositions of positions in a permutation cycle. Indeed, the cycle structure of the original permutation must be preserved through the induced-return interpretation, not through a claim that its row-level links remain unchanged.

The mathematical reason the update is correct is that the new return map is the composition of precisely those old returns that cross the removed terminal interval. For an affected point,

$$
F_{m'}(x)=F_m^{\,2}(x)
$$

in the elementary two-transition case, while its return word becomes the concatenation of the two old return words in their appropriate orientation.

We can therefore reduce the number of represented points without losing their omitted byte itineraries.

### 2.4 Why this might reduce work

A single induction step manipulates the terminal intervals' lengths, order relations and word references. It does **not** traverse each row of the interval being removed.

Let $S$ count elementary induction steps. For the original right-end construction, the Wikidata research census reported:

| Quantity | Value |
|---|---:|
| Run density $r/n$ | 0.0722 |
| Induction steps per run $S/r$ | 1.812 |
| Induction steps per row $S/n$ | 0.1309 |

**Approximately 131,000 induction steps per million BWT rows**, for this recorded Wikidata workload and the right-end schedule, not a general complexity bound.

This is where the original performance hypothesis comes from. We replace approximately one permutation transition per output byte with substantially fewer, more regular interval operations, followed by grammar expansion.

There are important limits. An adversarial text such as $(ab)^ka$, whose BWT has only two runs, can take linearly many ($O(n)$) elementary right-end steps. The journal identifies a possible *division rule* for batching repeated subtraction, but that is not part of the current Cells kernel. This accelerates induction itself: a repeated-winner sequence such as $17-3-3-3-3-3=2$ can be represented by $17=5\cdot3+2$, provided its word, order and marker updates are composed exactly. The connection is to Zorich-style acceleration, distinct from choosing a different induction schedule or optimising the conventional table walker.

There are also alternative induction orders with fewer steps. The journal found one reaching approximately $1.279r$ steps versus $1.812r$ at the right end. **Choosing those steps cheaply is itself an unresolved algorithmic problem.** Fewer transformations do not guarantee lower total construction cost.

Source: [MONODROMY mathematical construction, §§1–5](https://github.com/lmmx/mibz2/blob/2635f04e9155dd31388b24fd3f7248f87b3ddcda/docs/journal/monodromy/2026-11-10-rauzy-veech.md).

---

## 3. The reconstruction program produced by induction

The crucial second compression happens in the words.

Initially, each interval has a one-byte word. As induction proceeds, the algorithm creates rules of the form

$$
N_k\longrightarrow U_iU_j.
$$

Rather than copying the contents of $U_i$ and $U_j$ every time, it can retain their identities and lengths.

The resulting structure is a **straight-line program**, a directed acyclic grammar with exactly one derivation for each nonterminal.

For example, an illustrative grammar could be

$$
\begin{aligned}
U_0&\to \texttt{a},\\
U_1&\to \texttt{b},\\
U_2&\to U_0U_1,\\
U_3&\to U_2U_2,\\
U_4&\to U_3U_2.
\end{aligned}
$$

The root $U_4$ expands to six bytes, but the description contains reusable references to smaller words.

<figure class="bf-fig" id="rv-grammar">
<p class="bf-title">A word identity does not identify its occurrence</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 490" role="img" aria-labelledby="rv-grammar-title rv-grammar-desc"><title id="rv-grammar-title">A word identity does not identify its occurrence</title><desc id="rv-grammar-desc">The illustrative grammar expands U4 to ababab. Highlight the b at expansion offset 3, distinct from b at offsets 1 and 5. Rotating there yields bababa; repetition to 10 bytes yields bababababa.</desc><text x="26" y="30" class="bf-t-strong" text-anchor="start" style="font-size:24px">One definition; several occurrences</text><text x="40" y="75" class="bf-t" text-anchor="start" style="font-size:24px">U₀ = a       U₁ = b</text><text x="40" y="125" class="bf-t" text-anchor="start" style="font-size:24px">U₂ = U₀U₁ = ab</text><text x="40" y="175" class="bf-t" text-anchor="start" style="font-size:24px">U₃ = U₂U₂ = abab</text><text x="40" y="225" class="bf-t" text-anchor="start" style="font-size:24px">U₄ = U₃U₂ = ababab</text><text x="26" y="282" class="bf-t" text-anchor="start" style="font-size:22px">Expansion offsets (not BWT row IDs)</text><rect x="48" y="305" width="74" height="48" class="bf-card-hi" /><text x="85" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">a</text><text x="85" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">0</text><rect x="122" y="305" width="74" height="48" class="bf-card-hi" /><text x="159" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">b</text><text x="159" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">1</text><rect x="196" y="305" width="74" height="48" class="bf-card-hi" /><text x="233" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">a</text><text x="233" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">2</text><rect x="270" y="305" width="74" height="48" class="bf-pill-ready" /><text x="307" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">b</text><text x="307" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">3</text><rect x="344" y="305" width="74" height="48" class="bf-card-hi" /><text x="381" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">a</text><text x="381" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">4</text><rect x="418" y="305" width="74" height="48" class="bf-card-hi" /><text x="455" y="337" class="bf-t-strong" text-anchor="middle" style="font-size:26px">b</text><text x="455" y="383" class="bf-tm" text-anchor="middle" style="font-size:20px">5</text><text x="26" y="425" class="bf-t" text-anchor="start" style="font-size:23px">Start at marked offset 3: bababa</text><text x="26" y="464" class="bf-t" text-anchor="start" style="font-size:22px">Repeat / truncate to 10: bababababa</text></svg>
<figcaption>The source grammar, expanded exactly. The highlighted offset illustrates output phase; it is not numerically equated with <code>orig_ptr</code>, and this toy grammar is not claimed to be a particular LF cycle. Equal bytes and equal subwords can occur at distinct positions. A real decoder must obtain the marked occurrence from its induction invariant.</figcaption>
</figure>

This is not an output-compression format being added to bzip2. It is an **intermediate reconstruction program**. Its purpose is to replace expensive permutation navigation with cheaper grammar construction and memory copying.

The MONODROMY journal makes a stronger structural observation about the right-end construction: its grammar corresponds to a Lyndon-related decomposition of the reconstructed text. That helps explain why these particular concatenations arise, although the decoder's correctness does not require us to implement a general Lyndon-factorisation library.

### Recovering the correct BWT origin

It is insufficient to construct just any cyclic rotation of the text.

The decoded output must begin at the phase specified by `orig_ptr`. For periodic inputs, the selected permutation cycle can also be shorter than $n$.

The original mathematical construction has a marked-point invariant. One particularly useful implementation replaces the marked row with a singleton letter and propagates a marker through the grammar.

When induction terminates, the marker identifies a path through the grammar to that original row.

This gives both:

1. The correct permutation cycle to reconstruct.
2. The position at which to rotate its word to produce the required output.

For a cycle of length $p$, the output is the correctly rotated cycle repeated and truncated to $n$ bytes.

The marker has to identify a **particular occurrence or trajectory**, not merely the contents of a word. Two equal grammar words may represent different return destinations.

### Converting the grammar into actual bytes

The reconstruction phase must still emit $n$ bytes. No induction theorem eliminates that work.

The original research writer uses short inline words and longer grammar rules. It finds suitable output positions for rule expansions, expands factors, and uses copies from already materialised occurrences.

Long factors, repeated occurrences, the marked path and the final cyclic rotation require careful handling. These are not incidental implementation details: they determine both memory safety and the actual number of writes.

A previous writer version left certain marked-spine bytes unwritten—the marked spine is the grammar path carrying the selected occurrence. The corrected implementation includes targeted long-factor tests and independent review of the writer-safety finding F01. That episode is a useful reminder that **the grammar representation and its output schedule are separate correctness problems**.

At the end of the process, the original RLE1 decoding and CRC obligations remain.

This gives us a complete theoretical pipeline:

<figure class="bf-fig" id="rv-pipeline">
<p class="bf-title">From run intervals to verified output</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 585" role="img" aria-labelledby="rv-pipeline-title rv-pipeline-desc"><title id="rv-pipeline-title">From run intervals to verified output</title><desc id="rv-pipeline-desc">Five transformations: run records, weighted interval exchange, induction grammar, cycle-word reconstruction, RLE1 and CRC output. Each arrow changes the representation rather than transporting a message.</desc><rect x="25" y="15" width="490" height="85" class="bf-card" rx="6"/><text x="270" y="48" class="bf-t-strong" text-anchor="middle" style="font-size:24px">Run representation of L</text><text x="270" y="80" class="bf-t" text-anchor="middle" style="font-size:21px">byte values · lengths · orig_ptr</text><path d="M270 102L270 129" class="bf-arrow" style="stroke-width:2"/><path d="M264 121L270 129" class="bf-arrow" /><path d="M276 121L270 129" class="bf-arrow" /><rect x="25" y="131" width="490" height="85" class="bf-card" rx="6"/><text x="270" y="164" class="bf-t-strong" text-anchor="middle" style="font-size:24px">Weighted interval exchange</text><text x="270" y="196" class="bf-t" text-anchor="middle" style="font-size:21px">top / bottom orders · lengths · words</text><path d="M270 218L270 245" class="bf-arrow" style="stroke-width:2"/><path d="M264 237L270 245" class="bf-arrow" /><path d="M276 237L270 245" class="bf-arrow" /><rect x="25" y="247" width="490" height="85" class="bf-card-hi" rx="6"/><text x="270" y="280" class="bf-t-strong" text-anchor="middle" style="font-size:24px">Induction and grammar</text><text x="270" y="312" class="bf-t" text-anchor="middle" style="font-size:21px">compose returns · preserve the mark</text><path d="M270 334L270 361" class="bf-arrow" style="stroke-width:2"/><path d="M264 353L270 361" class="bf-arrow" /><path d="M276 353L270 361" class="bf-arrow" /><rect x="25" y="363" width="490" height="85" class="bf-card" rx="6"/><text x="270" y="396" class="bf-t-strong" text-anchor="middle" style="font-size:24px">Cycle-word reconstruction</text><text x="270" y="428" class="bf-t" text-anchor="middle" style="font-size:21px">expand · position · copy · repeat</text><path d="M270 450L270 477" class="bf-arrow" style="stroke-width:2"/><path d="M264 469L270 477" class="bf-arrow" /><path d="M276 469L270 477" class="bf-arrow" /><rect x="25" y="479" width="490" height="85" class="bf-card" rx="6"/><text x="270" y="512" class="bf-t-strong" text-anchor="middle" style="font-size:24px">RLE1, CRC and output</text><text x="270" y="544" class="bf-t" text-anchor="middle" style="font-size:21px">decode byte runs · verify checksum</text></svg>
<figcaption>Structural pipeline, not a timing or scale diagram. The grammar replaces permutation navigation; it does not remove output stores, run-length decoding (RLE1), or checksum (CRC) verification.</figcaption>
</figure>

**This is the central theoretical approach.** Everything in the later labelling, surgery, first-return and performance experiments concerns how to represent or execute some part of this pipeline more economically.

---

## 4. How the Cells algorithm implements that mathematics

The early research considered a fairly direct representation: two doubly linked orders of interval records, with each step updating links and creating one grammar rule.

That representation was correct, but fairly expensive. The important implementation advance was **Cells**.

Instead of continually modifying linked records, Cells processes the two orders from their right-hand ends, creates a permanent record for each consumed interval piece, and leaves surviving portions in a small amount of register state.

### A cell is a completed piece of the induced map

A cell contains the information needed for the represented interval piece: its width, word identity and relationships to its position in the other order.

Each step creates one cell corresponding to the shorter terminal overlap. Once written, the cell itself does not need to be repositioned or repeatedly patched.

The algorithm maintains two fronts:

- The last currently unconsumed interval in top order.
- The last currently unconsumed interval in bottom order.

For each front it carries the remaining length, word, predecessor information and the location where its next cell belongs in the other order.

This makes a substantial difference to the inner loop.

The original linked-order construction needed to maintain the connections of changing objects. The Cells construction instead uses *promised slots*: when an interval still has an unconsumed remainder, a location is reserved for its next completed piece. That allows the current cell's references to be finalised without later traversing a chain of displaced objects.

The central induction work is essentially:

<div id="rv-cells-pseudocode" markdown>

```text
Read terminal letters A and B from the two orders.

While the marked component is not closed:
    s = min(remaining_length(A), remaining_length(B))
    N = concatenate(word(A), word(B))

    Construct the completed cell of width s.
    Write its top- and bottom-order records.

    Consume s from both active terminal portions.
    Keep any winning remainder in register state.

    When a front is exhausted, read its predecessor.
    When a component closes, retain or discard it
    according to whether it contains the marked row.

Recover the marked cycle's grammar root.
Expand its word in the required origin phase.
```

</div>

That is a structural summary, not a substitute for the implementation's equal-length, closure and buffer-boundary logic.

### The physical representation

The implemented research design uses two arrays of **24-byte slots**, holding the two orders. It also has a small-word arena, a longer-rule arena and writer-position storage.

References within the principal buffer are **32-bit offsets**, rather than separately allocated Rust objects with full pointers.

That matters because a graph of heap-allocated nodes would introduce additional allocation, pointer loading and cache-line traffic. Cells is deliberately laid out to avoid much of that.

The protected #365 implementation records the explicit bound

$$
\boxed{\begin{gathered}
\text{Kernel buffers}\leq\\
48R+84B+4{,}528+4G\text{ bytes}
\end{gathered}}
$$

where:

- $R$ is the record count used for allocation;
- $B$ is the permitted number of cells;
- $G=64$ in debug/Miri and zero in release.

The coefficient $48R$ corresponds to two 24-byte order records per initial record. The $84B$ term covers additional slots, word/rule storage and writer positions.

In production admission, the cell budget is bounded by

$$
B\leq\left\lfloor\frac{4n}{25}\right\rfloor.
$$

The implementation also refuses high-density inputs before induction, checks the marked-letter floor (the number of letters after splitting out the marked row, minus one), and checks the cell budget before every cell.

On refusal, it calls the existing `Segmented` inversion path. This is not a loss of correctness; it is an attempt to prevent pathological induction costs.

These bounds describe the experimental Cells kernel's allocated working storage, not total decoder RSS. They exclude existing fallback scratch, input/output buffers, allocator metadata and fixed kernel state. Buffers also retain their high-water capacity across blocks.

<figure class="bf-fig" id="rv-slots">
<p class="bf-title">Cells fixes each completed piece in two orders</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 535" role="img" aria-labelledby="rv-slots-title rv-slots-desc"><title id="rv-slots-title">Cells fixes each completed piece in two orders</title><desc id="rv-slots-desc">Two illustrative arrays of equal 24-byte slots. The same cell c occupies different indexes, connected by a cross-order offset. Active remainder fields stay in registers; the next cell has a promised slot.</desc><text x="26" y="32" class="bf-t-strong" text-anchor="start" style="font-size:24px">Two physical orders, one cell identity</text><text x="26" y="76" class="bf-t" text-anchor="start" style="font-size:22px">Top-order slots (24 bytes each)</text><rect x="30" y="97" width="96" height="60" class="bf-slot" /><text x="78" y="135" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="126" y="97" width="96" height="60" class="bf-card-hi" /><text x="174" y="135" class="bf-t" text-anchor="middle" style="font-size:24px">c</text><rect x="222" y="97" width="96" height="60" class="bf-slot" /><text x="270" y="135" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="318" y="97" width="96" height="60" class="bf-slot" /><text x="366" y="135" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="414" y="97" width="96" height="60" class="bf-slot" /><text x="462" y="135" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><text x="26" y="334" class="bf-t" text-anchor="start" style="font-size:22px">Bottom-order slots (24 bytes each)</text><rect x="30" y="246" width="96" height="60" class="bf-slot" /><text x="78" y="284" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="126" y="246" width="96" height="60" class="bf-slot" /><text x="174" y="284" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="222" y="246" width="96" height="60" class="bf-slot" /><text x="270" y="284" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><rect x="318" y="246" width="96" height="60" class="bf-card-hi" /><text x="366" y="284" class="bf-t" text-anchor="middle" style="font-size:24px">c</text><rect x="414" y="246" width="96" height="60" class="bf-slot" /><text x="462" y="284" class="bf-t" text-anchor="middle" style="font-size:24px">…</text><path d="M174 162L366 237" class="bf-arrow" /><text x="325" y="190" class="bf-tm" text-anchor="middle" style="font-size:20px">cross-order offset</text><text x="26" y="380" class="bf-t-strong" text-anchor="start" style="font-size:23px">Active remainders: register state</text><text x="26" y="419" class="bf-t" text-anchor="start" style="font-size:21px">Lengths · words · predecessor offsets</text><text x="26" y="460" class="bf-t" text-anchor="start" style="font-size:21px">Next cell: reserved slot in the other order</text><text x="26" y="504" class="bf-t" text-anchor="start" style="font-size:22px">32-bit offsets address a common buffer.</text></svg>
<figcaption>Logical array layout, not a cache-line map or a specific execution snapshot. The slot indexes shown are illustrative. Each completed cell has a record in each order; the front keeps the winning remainder and reserves its next slot. A 24-byte slot may straddle a cache line.</figcaption>
</figure>

### Why 24-byte slots and a common buffer matter

This is where the algorithm starts becoming a physical-computing design rather than just a combinatorial one.

The slots are fixed-width and reached through offsets in contiguous allocations. A completed cell is generally written in two places determined by the two orderings, so this is **not a purely sequential array algorithm**. Nevertheless, those destinations are often knowable before the step that writes them.

The research implementation prefetches information approximately **12 slots ahead** of the current frontier. On its original research machine, that change reduced a cell's observed time from roughly 9.8 ns to 7.2–7.7 ns in the tested samples. These were the same ten sampled Wikidata blocks, one run per variant on the shared research container; the journal warns of roughly 10% run-to-run variation, and the intermediate variants were not committed. This is a historical observation, not a portable prefetch-distance or speed guarantee.

This is a direct example of mathematical structure informing microarchitecture: because the induction determines future slot destinations, the implementation can request the associated cache lines before they are needed.

The 24-byte layout is a logical packing choice; it is not a promise that every record is contained within one cache line. Explicit unaligned accesses are part of the implementation's safety argument.

The 16-byte short-word representation is another deliberate choice. Frequently used small words can be stored as bytes, while longer words remain grammar references.

Source: [Cells safety and layout argument](https://github.com/lmmx/mibz2/blob/d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2/docs/journal/monodromy/2026-11-10-cells-slice-one/SAFETY.md).

---

[Layout and bound record](https://github.com/lmmx/mibz2/blob/d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2/docs/journal/monodromy/2026-11-10-cells-slice-one/SAFETY.md).

## 5. What physically happens in memory and the cache hierarchy

It is useful to distinguish the idealised algorithmic objects from their actual storage and access patterns.

<div id="rv-storage" markdown>

| Object | Physical representation | Main cost or concern |
|---|---|---|
| Conventional inverse table | Contiguous 32-bit entries, $4n$ bytes | Dependent and irregular reads during traversal |
| BWT run input | Compact byte/length records | Sequential scanning, conversion and record coalescing |
| Cells interval orders | Two arrays of 24-byte slots | Reads and writes through two orderings; prefetchability |
| Cells short words | Inline bytes in an arena | Small writes and short copies |
| Longer return words | Grammar rules referring to earlier words | Grammar-node creation, traversal and output copies |
| Delimiter first-return state | Interval records, frontier and completed pieces | Boundary splitting, lookups and metadata |
| Phrase dictionary/hash cons | Word IDs, child references, hash-table slots | Lookup traffic, growth, allocation and retained metadata |
| Final output | Byte buffers for BWT inversion, RLE1 and writer stages | Unavoidable output plus any additional copies |

</div>

### The key distinction: bytes resident versus bytes accessed

The original research journal estimated a Cells live working set of several megabytes for a large Wikidata block, **potentially larger than the conventional permutation table**.

That sounds counterintuitive if we're expecting compression to save memory.

But its intended advantage was not necessarily a smaller total allocation. It was a more favourable pattern of *accessing* that allocation.

For a conventional permutation table, successive accesses within a chain depend on previously loaded destinations. If the next entry misses private cache, execution can stall waiting for the appropriate line.

With Cells, the principal operations include fixed-size records, short-word writes, prefetchable order fronts and eventual output copies. These can offer more predictable memory traffic and more opportunity for overlap.

That distinction showed up in the journal's instrumented analysis: on selected Wikidata blocks, Cells used fewer instructions and substantially fewer **simulated private-cache misses** than the conventional build-plus-walk path.

However, simulated misses on one machine are not measurements of L1/L2/L3 residency on the i9 workstation, and certainly not proof of equal behaviour with several decoder workers contending for the shared last-level cache.

### A cache-level interpretation

<figure class="bf-fig" id="rv-cache">
<p class="bf-title">Access patterns through the memory hierarchy</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 510" role="img" aria-labelledby="rv-cache-title rv-cache-desc"><title id="rv-cache-title">Access patterns through the memory hierarchy</title><desc id="rv-cache-desc">Conceptual list of registers, L1, private L2, shared L3 and DRAM with possible data accesses. No sizes, timings, measured residency or cache hit guarantees are encoded.</desc><text x="26" y="32" class="bf-t-strong" text-anchor="start" style="font-size:23px">Possible access targets—not residency</text><rect x="25" y="64" width="490" height="74" class="bf-card-hi" rx="5"/><text x="44" y="92" class="bf-t-strong" text-anchor="start" style="font-size:23px">Registers</text><text x="44" y="122" class="bf-t" text-anchor="start" style="font-size:21px">active lengths, words and offsets</text><rect x="25" y="151" width="490" height="74" class="bf-card" rx="5"/><text x="44" y="179" class="bf-t-strong" text-anchor="start" style="font-size:23px">L1</text><text x="44" y="209" class="bf-t" text-anchor="start" style="font-size:21px">hot slot fields and short words</text><rect x="25" y="238" width="490" height="74" class="bf-card" rx="5"/><text x="44" y="266" class="bf-t-strong" text-anchor="start" style="font-size:23px">Private L2</text><text x="44" y="296" class="bf-t" text-anchor="start" style="font-size:21px">hot parts of order / grammar arrays</text><rect x="25" y="325" width="490" height="74" class="bf-card" rx="5"/><text x="44" y="353" class="bf-t-strong" text-anchor="start" style="font-size:23px">Shared L3 / LLC</text><text x="44" y="383" class="bf-t" text-anchor="start" style="font-size:21px">arenas and table; competing workers</text><rect x="25" y="412" width="490" height="74" class="bf-card" rx="5"/><text x="44" y="440" class="bf-t-strong" text-anchor="start" style="font-size:23px">DRAM</text><text x="44" y="470" class="bf-t" text-anchor="start" style="font-size:21px">misses, allocation and buffers</text></svg>
<figcaption>Conceptual hierarchy only: these are possible access targets, <strong>not measured or guaranteed residency</strong>. Equal box widths deliberately encode no cache capacity. Independent lanes can hide dependency latency; prefetch and copies may change exposure, but actual cache traffic and concurrency effects require measurement.</figcaption>
</figure>

There are several points where the implementation choices could change cache behaviour substantially:

**First, slot width and locality.** Reducing two 24-byte records to smaller fields could reduce the number of cache lines touched, but may require additional loads or arithmetic. It is not automatically worthwhile.

**Second, prefetch distance.** Twelve slots ahead was empirically useful for the initial Cells loop. A different CPU or changed frontier schedule might favour a different distance.

**Third, grammar locality.** A tightly packed grammar can be much cheaper to traverse than Python objects linked through dictionaries. But even a compact grammar can incur expensive random lookups and repeated expansion.

**Fourth, concurrent decoder workers.** A working set that behaves well alone can create pressure on shared cache and memory bandwidth when many block decoders operate concurrently. The high-water allocations matter as well as the live logical records.

**Fifth, the writer.** A sequential copy is not free, but it can exploit memory bandwidth and cache-line locality in a way that a dependent permutation walk cannot. The number of copies, intermediate buffers and actual stores still determine whether that is advantageous.

These are hardware-performance hypotheses, not equivalent descriptions of the same algorithmic count.

### What is stored on disk?

During ordinary decoding, the compressed bzip2 file is the persistent input; the interval exchange, permutation table, grammar and output scratch are in-memory structures, not disk-backed databases.

The research infrastructure is different. It persists frozen corpora or their hashes, exported run streams, immutable schedules, source identities, operation counters, validation logs and experiment reports to GitHub branches or retained artifacts.

For example, PR #371 retained compressed input/run evidence and the result of its single block attempt. That material is for reproducibility and independent verification. The candidate constructor was not permitted to obtain its answer from a precomputed oracle path.

So **the proposed performance gain is primarily about CPU instructions, cache latency, memory traffic, allocation and reconstruction work—not reducing disk seeks**. Any end-to-end file-reading effect is a separate measurement question.

---

## 6. The second algorithm: delimiter-based first returns

This is where the last two days' mathematical experiments diverged from the original right-end Cells algorithm.

Instead of always inducing on a progressively shorter prefix $[0,m)$, suppose we retain a set of rows

$$
K=\{i:L[i]=c\}
$$

for some chosen delimiter byte $c$.

Define the first-return map

$$
F_K(x)=\pi^{t(x)}(x),
$$

where $t(x)$ is the smallest positive integer for which $\pi^{t(x)}(x)\in K$.

Between two consecutive retained rows, the original permutation emits a string. We store that string as a **return word**.

The resulting representation consists of a smaller permutation on the retained rows, together with words describing the excursions between them.

Unlike the right-end method, the retained set is not generally a contiguous interval. The constructor must determine the correct return itineraries by propagating intervals through the LF run map and splitting them where their trajectories diverge.

<figure class="bf-fig" id="rv-delimiter">
<p class="bf-title">First returns to a separated section</p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 505" role="img" aria-labelledby="rv-delimiter-title rv-delimiter-desc"><title id="rv-delimiter-title">First returns to a separated section</title><desc id="rv-delimiter-desc">Illustrative six-point LF orbit 0,1,2,3,4,5,0. Retain 0 and 3. First returns swap 0 and 3 with return time 3 and itineraries 0,1,2 and 3,4,5.</desc><text x="26" y="32" class="bf-t-strong" text-anchor="start" style="font-size:24px">Retain K = {0, 3} on one LF orbit</text><text x="26" y="75" class="bf-t" text-anchor="start" style="font-size:22px">Original orbit (illustrative):</text><rect x="47" y="100" width="64" height="50" class="bf-card-hi" /><text x="79" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">0</text><path d="M111 125L127 125" class="bf-arrow" /><path d="M120 119L127 125" class="bf-arrow" /><rect x="129" y="100" width="64" height="50" class="bf-card" /><text x="161" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">1</text><path d="M193 125L209 125" class="bf-arrow" /><path d="M202 119L209 125" class="bf-arrow" /><rect x="211" y="100" width="64" height="50" class="bf-card" /><text x="243" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">2</text><path d="M275 125L291 125" class="bf-arrow" /><path d="M284 119L291 125" class="bf-arrow" /><rect x="293" y="100" width="64" height="50" class="bf-card-hi" /><text x="325" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">3</text><path d="M357 125L373 125" class="bf-arrow" /><path d="M366 119L373 125" class="bf-arrow" /><rect x="375" y="100" width="64" height="50" class="bf-card" /><text x="407" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">4</text><path d="M439 125L455 125" class="bf-arrow" /><path d="M448 119L455 125" class="bf-arrow" /><rect x="457" y="100" width="64" height="50" class="bf-card" /><text x="489" y="134" class="bf-t-strong" text-anchor="middle" style="font-size:25px">5</text><text x="26" y="190" class="bf-t" text-anchor="start" style="font-size:21px">5 returns to 0; gray rows are deferred.</text><text x="26" y="245" class="bf-t-strong" text-anchor="start" style="font-size:23px">First-return map on the retained rows</text><rect x="70" y="278" width="120" height="60" class="bf-card-hi" /><text x="130" y="317" class="bf-t-strong" text-anchor="middle" style="font-size:26px">0</text><rect x="350" y="278" width="120" height="60" class="bf-card-hi" /><text x="410" y="317" class="bf-t-strong" text-anchor="middle" style="font-size:26px">3</text><path d="M195 300L345 300" class="bf-arrow" /><path d="M337 294L345 300" class="bf-arrow" /><path d="M345 323L195 323" class="bf-arrow" /><path d="M203 329L195 323" class="bf-arrow" /><text x="26" y="390" class="bf-t" text-anchor="start" style="font-size:22px">0 → 3: itinerary (0, 1, 2), time 3</text><text x="26" y="430" class="bf-t" text-anchor="start" style="font-size:22px">3 → 0: itinerary (3, 4, 5), time 3</text><text x="26" y="476" class="bf-t" text-anchor="start" style="font-size:21px">Each arrow retains its exact emitted word.</text></svg>
<figcaption>Orbit-coordinate example of a first-return section, not a claimed LF column or token-BWT ordering. Retaining rows need not mean retaining a prefix. The return word records the visited bytes in the constructor’s declared orientation; this picture alone does not establish delimiter-token lexicographic correspondence.</figcaption>
</figure>

### The practical constructor

The experimental delimiter constructor works approximately as follows:

1. Find the delimiter intervals among the input runs.
2. Initialise a frontier of interval fragments, each carrying its source-anchor identity, current interval coordinates and accumulated return-word reference.
3. Apply LF's translation to a fragment.
4. If its image crosses run or delimiter boundaries, split it into fragments with uniform return behaviour.
5. When a fragment reaches a delimiter interval, record its first-return destination and word.
6. Assemble the resulting token-level permutation, recover the marked cycle and expand its return words.

The experimental representation also exploits the fact that delimiter-separated phrases admit a useful lexicographic token ordering. The tests verify when the induced map agrees with the token-BWT construction.

That property is conditional. Arbitrarily adding the original row to the retained set, for example, can destroy the token-order correspondence even though the first-return map remains mathematically valid.

### What happened on real data?

[PR #371](https://github.com/lmmx/mibz2/pull/371) tested one frozen Wikidata block:

| Quantity | Observed |
|---|---:|
| Original LF rows $n$ | 899,931 |
| Maximal LF runs | 51,184 |
| Delimiter intervals | 679 |
| Delimiter rows | 135,716 |
| Constructor interval visits $E$ | 124,080 |
| Completed phrase pieces $Q$ | 6,133 |
| Grammar nodes $H$ | 70,559 |
| Constructor plus residual token visits | 259,796 |

The output bytes, origin, selected cycle, token permutation, RLE1 result and CRC agreed with the reference.



The traversal count was approximately 71% below $n$. That is meaningful evidence that a smaller return-level description exists for this block.

However, it is **not** evidence of a 71% reduction in CPU time.

The constructor still represented trajectories covering all 899,931 rows in its weighted accounting. It also performed approximately 5.69 million binary-search comparisons, and residual reconstruction involved 1.66 million grammar visits.

Most decisively, the Python implementation's live metadata was at least **23.5 MB**, against the experiment's $4n$, or **3.6 MB**, limit.

It therefore failed its predeclared memory criterion.

The conclusion was not that first returns cannot work. It was that **this representation did not earn a compiled performance experiment**.

[Frozen block result and accounting](https://github.com/lmmx/mibz2/blob/bf88365ba80388e17488a31c80d2f3f250b19fb2/docs/journal/2026-11-11-first-return-block/README.md).

### Why this is genuinely a different design choice

With Cells, the question is:

> Can we reduce the complete interval exchange quickly enough that a grammar of its selected cycle is cheap to construct?

With delimiter first returns, it is:

> Can we choose a small retained section whose return words are inexpensive to construct and whose token-level permutation is inexpensive to traverse?

Both produce grammars, but their construction costs arise in different places.

Cells concentrates effort in right-end length comparisons and permanent cells. Delimiter first returns concentrate effort in fragment propagation, boundary splitting, return-word accumulation, dictionary construction and token traversal.

It is entirely possible for one to win on an input where the other loses.

---

## 7. Why we investigated labelling and combinatorial surgery

We were looking for ways to make the **construction** of these compressed descriptions cheaper.

This was the connection with de Mourgues's work on reduced/boosted dynamics, labelling and combinatorial surgery.

### The labelling hypothesis

If a portion of a permutation changes, perhaps we can retain stable identities for the unaffected pieces rather than renumber their positions repeatedly.

The thesis's labelling method preserves information about where distinguished edges attach as combinatorial transformations change the surrounding permutation. The repository study could not directly retrieve the thesis PDF; it checked the author's [first paper, §3.4](https://arxiv.org/pdf/1705.01641v2) and [second paper, §§2 and 5](https://arxiv.org/html/1801.05973v2). The thesis remains the historical reference, while those papers and the pinned experiment delimit what was directly verified.

An LF implementation could use a related idea to preserve interval identities, return-word occurrences and the marked origin.

But the thesis's combinatorial equivalence is not itself an equivalence of *weighted LF maps with emitted words*. That additional structure must be preserved explicitly.

PR #369 found small counterexamples to naive edge deletion and positional lifting. Exact first-return construction requires the proper interval refinements.

### The surgery hypothesis

Suppose a current return map is $F$, and we remove a set $D$ containing no consecutive points of the $F$-orbit:

$$
D\cap F(D)=\varnothing.
$$

For a surviving point $x$, the new first-return map is

$$
F'(x)=
\begin{cases}
F(F(x)),&F(x)\in D,\\
F(x),&\text{otherwise}.
\end{cases}
$$

The emitted word is the concatenation of the transitions actually taken.

That is an **exact local contraction**. For interval fragments sharing the same itinerary, it becomes composition of translations and concatenation of word references.

Our hoped-for optimisation was to apply many such changes locally while retaining the rest of the representation.

### What the experiments revealed

[PR #370](https://github.com/lmmx/mibz2/pull/370) tested an implementation maintaining the two orders through treaps.

It preserved the correct destinations, words, marked origin and final coalescing.

It also reduced sorting comparisons from 406,949 to 99,440 and eliminated 105,708 persistent coordinate writes.

But it introduced approximately 883,875 pointer writes and 893,297 tree-size writes, together with considerable allocation and metadata.

**We had eliminated coordinate rebuilding but created a more expensive maintenance problem.** Here the evidence is the counted metadata work, not a native runtime comparison.

[PR #372](https://github.com/lmmx/mibz2/pull/372) then evaluated the alternatives against the saved traces. Its preferred comparator was no longer a general dynamic tree, but a compact global-array rebuild.

A potential hybrid remains:

$$
\text{Choose between }
\begin{cases}
\text{batch-local update},\\
\text{global compact rebuild}
\end{cases}
$$

according to how much of the representation actually changes.

In a chunked representation with chunk capacity $b$, $D$ dirty chunks, $R$ total records and $S$ distinct interior cuts, local maintenance might pay roughly $O(bD+S)$ work versus $O(R+S)$ for a global pass, excluding common selection and other overheads. Here $S$ counts cuts, not the induction steps of §2.4.

The essential unknown is $D$, the distribution of dirty chunks in **both** interval orders. Sparse deletions do not imply cheap local updates.

PR #372 did not have that incidence measurement, so it did not authorise another implementation.

This is where our optimistic idea stands: **mathematically sound, potentially useful, but currently unpriced well enough to justify another backend.**

---

[Contraction counts and semantics](https://github.com/lmmx/mibz2/blob/1ab6cae1a1281c7ba1ba31c0eb25d29bd8779293/docs/journal/2026-11-11-labelled-contraction/README.md); [saved-evidence pricing](https://github.com/lmmx/mibz2/blob/0596240ef7b61a23c3c4135c910d569f9fbeb8a4/docs/journal/2026-11-12-performance-pricing/REPORT.md).

## 8. The next promising representation problem: grammar allocation and output

PRs #371–372 also exposed a different opportunity.

The real-block phrase constructor had 70,559 live grammar nodes, but only 2,716 distinct expanded dictionary words across 6,133 completed pieces.

Constructing each distinct word once, rather than repeatedly expanding shared DAG content, could eliminate **103,233 grammar visits** in the recorded dictionary-building phase.

Further savings may be possible by reusing those expanded words directly while writing output, rather than expanding the same grammar through every token occurrence.

But that changes both the CPU and memory trade-off.

For example, a hypothetical compact native representation for #371 might use 32-bit indices for grammar nodes, interval identifiers and child references.

The operation counts suggest such a representation is worth understanding, but ordinary hash-table growth remains problematic.

At a 75% hash-table load threshold, doubling from 65,536 to 131,072 slots while both tables are live creates a temporary allocation peak. In the specified model, that alone brings the accounted working set above the original $4n$ limit.

There are alternatives: preallocation, segmented tables, fixed-capacity arenas, separate construction and expansion lifetimes, and so forth. Each requires its own construction and allocation-cost accounting.

**The unresolved question is not simply whether the final grammar is small. It is whether we can construct and use it without a larger, more expensive temporary representation.**

This is an area where physical layout is particularly relevant. The lifetime of the hash-cons table, the grammar node arena, completed interval records and expanded dictionary buffers may matter as much as their individual sizes.

---

[Grammar expansion and temporary-allocation model](https://github.com/lmmx/mibz2/blob/0596240ef7b61a23c3c4135c910d569f9fbeb8a4/docs/journal/2026-11-12-performance-pricing/REPORT.md).

## 9. What actually happened from 10–12 November

Here is the recent work separated by purpose. The PRs are not successive versions of one decoder kernel.

<div id="rv-history" markdown>

| Work | What it established | Consequence |
|---|---|---|
| **MONODROMY Cells research** | Endpoint induction can generate reconstruction grammars, with an indicative inversion-stage advantage | Build an opt-in real-decoder version |
| [PR #365](https://github.com/lmmx/mibz2/pull/365) | Correctness, bounded admission, fallback and writer safety; no convincing whole-decoder speedup | Defer deployment pending attribution |
| [PR #367](https://github.com/lmmx/mibz2/pull/367) | Most Wikidata rows qualify for Cells; admission dilution alone does not explain the stage-to-decoder discrepancy | Measure the actual decoder phases |
| [PR #369](https://github.com/lmmx/mibz2/pull/369) | Reduced/boosted-style LF lifting needs exact refinement and more than positional labels | Reject naive reduction |
| [PR #370](https://github.com/lmmx/mibz2/pull/370) | Labelled local contraction is exact, but dynamic-tree maintenance is expensive | Reconsider representation |
| [PR #371](https://github.com/lmmx/mibz2/pull/371) | Real-block first returns substantially reduce counted visits but exceed metadata budget | Revise memory/grammar construction |
| [PR #372](https://github.com/lmmx/mibz2/pull/372) | Compact rebuilding is a stronger comparator; dictionary reuse has concrete potential | Prefer existing decoder diagnostic before another backend |
| [PR #366](https://github.com/lmmx/mibz2/pull/366) | Prepared a separate frozen one-core diagnostic and incorporated the reviewed accounting correction | Await deliberate integration and readiness |
| [PR #373](https://github.com/lmmx/mibz2/pull/373) | Prepared the matched live-reader Cells diagnostic, with source correspondence and phase instrumentation | Correct review finding, reconcile checks, then evaluate readiness |

</div>

### The important discrepancy we have been pursuing

The original Cells research measured an inversion-stage ratio around **0.82–0.86×** the existing run-record inversion path on its Wikidata test environment.

But the subsequent opt-in production-style benchmark did not reproduce that advantage: the reported Wikidata one-thread whole-decoder ratio was approximately **1.000**, while some other inputs were considerably slower.

Those measurements have different scopes, implementations, hardware and cache conditioning. They cannot be compared as if one were simply a timed subroutine inside the other.

The #367 census established that 196 of 431 Wikidata blocks were admitted, representing approximately **84.1% of rows**. That makes a pure admission-rate explanation insufficient.

Other candidates remain: the production writer's different copy operations, source-layout differences, cache conditioning, instrumentation effects, or costs elsewhere in the decoder.

These are precisely the alternatives that the current diagnostic is intended to distinguish.

---

[Admission census and comparison limits](https://github.com/lmmx/mibz2/blob/a08277e1c36e276cfea7b3116d99ea5fd93f0e84/docs/journal/monodromy/2026-11-10-cells-gap/HANDOFF.md).

## 10. Where the live diagnostic fits

The prepared #373 harness uses the actual reader and reconstructs an experimental decoder from the corrected #365 source, with observation hooks compiled either in or out.

The measurement plan has four arms:

<div id="rv-arms" markdown>

| | Hooks off | Hooks on |
|---|---|---|
| Existing baseline | Baseline, uninstrumented | Baseline, instrumented |
| Experimental Cells | Cells, uninstrumented | Cells, instrumented |

</div>

The frozen design specifies ten balanced pairs on each of Wikidata, pattern and onebyte: **120 eventual measured decodes**, plus nine untimed/oracle checks.

The instrumented build records exclusive costs for stages such as scanning, run-front construction, ordering, induction, writing, fallback and finishing, together with the reader and complete-decode spans.

This is valuable because it provides a way to separate several possibilities.

If Cells is slower even within a matched inversion-stage comparison, attention should return to its implementation and memory behaviour.

If Cells wins inside inversion but the total decoder does not, the issue may be other stages, instrumentation or integration effects.

If the instrumented and uninstrumented comparisons differ materially, the phase attribution itself may be misleading.

The harness explicitly guards against counting inclusive inversion time again as if it were an additional exclusive stage. Matching off/on conclusion categories would not establish equal effect sizes or causal savings in the uninstrumented build. For example, Cells/baseline ratios of 0.9 with hooks off and 0.45 with hooks on could both qualify as faster under the frozen interval rule. Detailed stage attribution describes the instrumented path, whose observation overhead must still be considered.

### Historical status in the 12 November report

!!! note "Editorial update · 12 November 2026"

    After the source report’s cutoff, the Coordinator [accepted R1](https://github.com/lmmx/mibz2/pull/145#issuecomment-6822234208) after checking the correction’s diff and hashes and rerunning three focused mocked tests. Integration remained deferred. The [subsequent coordination record](https://github.com/lmmx/mibz2/pull/145#issuecomment-6823246963) reports preparation, Test and Benchmark workflow success, with Occupancy and legacy pre-commit still failing and both macOS packaging jobs cancelled without validation. Neither correction acceptance nor workflow success establishes a speedup. Separate integration, exact-head CI and final workstation readiness remain outstanding; no authorised live workstation timing is recorded. The original report is preserved unchanged in the source archive.

At the source report’s 12 November cutoff, the independent Researcher review of #373 identified a real failure-path issue: resource measurements could be discarded when a child process exited unsuccessfully or produced invalid/error JSON.

**The correction had been implemented** in [the corrected PR #373 head](https://github.com/lmmx/mibz2/commit/b5131679f51c2a0245bc03f62b7afd833b8bff23), with focused failure-path tests. The source report recorded Coordinator reconciliation as outstanding at its cutoff.

At that historical cutoff, the changed-head untimed workflow had succeeded. Occupancy still fails an inherited accounting assertion; legacy pre-commit remains red with its cause unverified, and some platform/benchmark checks were still pending at the last check. The reviewed accounting repair exists separately but has not yet been deliberately integrated into the relevant baseline.

No authorised live workstation timing has yet been performed.

Sources: [Researcher review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6821626797), [Codex's R1 correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6822169425), and [#373 handoff](https://github.com/lmmx/mibz2/blob/b5131679f51c2a0245bc03f62b7afd833b8bff23/docs/journal/2026-11-12-cells-live/HANDOFF.md).

---

## 11. Where further interventions would be most useful

I would keep five questions distinct, because they correspond to different kinds of possible progress.

**A. Which induced map do we want?**

The right-end Cells construction and delimiter first-return construction are two choices of section. Neither dominates the other mathematically or computationally.

An intervention here would concern the inducing set or the order of reductions: for example, choosing boundaries that avoid later fragmentation or repeated winner steps.

**B. What is the minimum reconstruction information?**

We need to preserve exact destinations, emitted words, return times and the original marked occurrence. But it is still open whether we can represent these with substantially less metadata or fewer transient structures.

An intervention here concerns the *semantics of the intermediate representation*: what needs to be stored and what can be inferred or shared.

**C. How should we maintain that representation?**

Linked orders, permanent cells, packed arrays, treaps and chunked arrays are distinct implementations of related mathematical state.

Here we need to price writes, lookups, allocations, cache misses, cross-order indices, coalescing and rebuild costs. Local contraction remains interesting, but a compact global pass is currently the comparison it must beat.

**D. How should we execute the reconstruction program?**

This includes grammar expansion, dictionary reuse, output scheduling, origin rotation and RLE1/CRC work. A small grammar is no use if traversing it repeatedly costs more than following the original permutation.

The possible direct dictionary-output savings identified in #372 belong here.

**E. Where does the actual decoder spend time?**

Before making another substantial implementation investment, we need the matched Cells diagnostic. It determines whether the principal issue is mathematical construction, the native port, the writer, surrounding decoding stages or runtime conditioning.

This is the immediate experimental priority.

## My overall assessment

The central theoretical programme remains coherent:

$$
\boxed{\begin{gathered}
\text{LF interval exchange}\\
\downarrow\\
\text{induced return maps}\\
\downarrow\\
\text{compact word program}\\
\downarrow\\
\text{exact text}
\end{gathered}}
$$

The theory explains why it is possible to replace row-level permutation traversal with transformations on run intervals and symbolic words.

The engineering programme is to make **every stage of that replacement cheaper than the computation it eliminates**.

We have already demonstrated important parts of this: exact induction, correct origin preservation, compact symbolic programs, and favourable operation counts in selected circumstances. We have also found costly representations that should not be deployed.

My preference is to maintain both mathematical lines—endpoint Cells and phrase-level first returns—as distinct research possibilities, while using the present diagnostic to guide the next implementation investment. I would not rule out novel induction schedules, better local contraction or more compact grammar construction; I simply wouldn't assume their benefits without charging their full costs.

**The next decisive result is not another proof of correctness. It's a trustworthy explanation of why the research inversion advantage has not yet appeared in full decoding.** Once we have that, we can decide intelligently whether to improve the mathematics, change the storage representation, reorganise the writer, or focus on the surrounding decoder.

---

**Research records:** [original technical report and internal reading notes](https://github.com/lmmx/mibz2/pull/145#issuecomment-6823145996); [mathematical construction and historical timings](https://github.com/lmmx/mibz2/blob/2635f04e9155dd31388b24fd3f7248f87b3ddcda/docs/journal/monodromy/2026-11-10-rauzy-veech.md); [reduced/boosted counterexamples](https://github.com/lmmx/mibz2/blob/85735b38ebfa804acd31a6291bb7c08b6bf57760/docs/journal/2026-11-11-reduced-lf/README.md); [de Mourgues’s thesis](https://theses.hal.science/tel-02285651/), §§4.1.4–4.1.5 and Chapter 5; [live diagnostic plan](https://github.com/lmmx/mibz2/blob/3b43af956d553905ea744f6cb6e1c785a0f8297a/docs/journal/2026-11-12-cells-live/PLAN.md).
