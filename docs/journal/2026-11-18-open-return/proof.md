# Cutting an interval exchange open

Let the rows be $0,\ldots,n-1$. Write $f(i)=C[L[i]]+\operatorname{rank}_{L[i]}(L[0:i])$, with half-open ranks. A step from $x$ to $f(x)$ emits $L[x]$ in backwards reading order. Each constant run of $L$ is a labelled translation. The named primary $p$ fixes the reading position and the component. All the arguments below apply to arbitrary columns with a valid primary, including multiple-cycle LF permutations.

## Restricting a stable match

Choose any nonempty set $S$ of retained rows containing $p$, and put $E=f(S)$ and $m=|S|$. Write $s_j,e_k$ for their elements in increasing order, indexed from zero. Let $v=L[s_0]\cdots L[s_{m-1}]$.

**Restriction lemma.** If $f(s_j)=e_k$, then $k=\mathrm{LF}_v(j)$.

**Proof.** The full image order is stable sort by the pair $(L[x],x)$. Restricting that sorted list to sources in $S$ produces exactly the stable sort of $v$. This holds for any subset and any byte frequencies. ∎

Now delete the arrows with source in $S$. Each $e_k$ lacks an incoming arrow and each $s_i$ lacks an outgoing arrow. Following the surviving arrows from $e_k$ reaches a unique $s_i$, possibly in zero steps. A path starting at an exposed entrance cannot enter a cycle, since a cycle already uses its incoming arrows. Reversing paths shows that each exit is reached exactly once. The components disjoint from these paths are closed cycles.

Define $P(k)=i$, and let $w_k$ be the word on this open path in LF order. The first return to $S$, in its sorted coordinates, is

$$
     g(j)=P(\mathrm{LF}_v(j)).
$$

The return from $j$ emits $v[j]w_k$. Its inverse arrow from $i$ to $j$ emits $\operatorname{rev}(w_k)v[j]$. Starting at the index of $p$, these inverse arrows spell the marked text forwards. One lap gives the period of the marked LF component in bytes. Repeat its spelling and truncate at $n$, exactly as for the ordinary marked inverse. This includes a mark in a fixed point and components that never meet another cut.

**Independence.** The complementary labelled translations and the two masks $S,E$ determine $P,w$ before the retained arrows are wired. Their internal permutation is used once through the ordinary stable LF of $v$. Ranks in $S$ and $E$ are different coordinates, and the correction $P$ can permute them. The subcolumn can itself have several LF cycles even when the original column has one. This is a factorisation of the inverse problem, with no lexical ordering assumption about return words.

For example, take the column `annb$aa` of `banana$`, primary 4, and retain original rows 0, 4 and 5. The restricted column is `a$a`. Its LF images in index order are `[1,0,2]`, so it has two cycles. The entrances are original rows 0, 1 and 2. Their open words are respectively the empty word, `n` and `nab`, and their exit indices are `[0,2,1]`. Composing gives `[2,0,1]`, one cycle. Starting at retained index 1, the inverse returns emit `bana`, `na` and `$`. The complement restores both the missing letters and the connectivity of the marked orbit.

The corresponding operation on a transition matrix is the **stochastic complement**, or trace chain on a set of states. Take rows as sources, columns as targets, $C=S^c$, and let $M$ be the permutation matrix. After deleting closed components wholly inside $C$, the internal block $M_{CC}$ is nilpotent. The first-return matrix is exactly

$$
M_{SS}+M_{SC}\left(\sum_{a\ge 0}M_{CC}^{a}\right)M_{CS}.
$$

The displayed sum terminates. Giving each arrow its letter and multiplying by concatenation gives the same paths with their spellings, since there is a unique forward path. This identifies a familiar elimination operation. The stable-restriction lemma gives its special computational form for the BWT. Only the complementary translations and their exposed boundaries enter the elimination. There is no matrix in the implementation. For the classical matrix construction see C. D. Meyer, “Stochastic Complementation, Uncoupling Markov Chains, and the Theory of Nearly Reducible Systems,” *SIAM Review* 31 (1989), 240–272.

## How many pieces survive the cut?

Suppose the complement consists of $H$ labelled strips, disjoint in both domain and image. Their lengths can differ. The intervals of $S$ form at most $H+1$ components, and the intervals of $E$ form at most $H+1$. Partition the original row line into its $H$ complementary strips and the $s$ components of $S$. Rows in one component of $S$ are equivalent as exits up to translation in retained rank.

Start at the leftmost uncovered row of an $E$ component. Follow its open path. At each strip and at the terminal component, take the minimum of the remaining rightward slack, including the starting $E$ interval. All rows in the resulting bundle follow the same labelled strip itinerary to consecutive exits. Advancing by that width enumerates the maximal bundles for these cuts. A byte boundary in the ordinary first column may be imposed at the entrance as well, so the final retained literal is constant on a bundle.

**Band bound.** With $e$ entrance intervals after such optional cuts, the number of bundles is at most $e+H+s-1$. With the byte cuts, it is at most $3H+\sigma$, where $\sigma$ is the number of occupied bytes.

**Proof.** Represent an interior gap by its right row. Inside one complementary strip it moves to the gap between two consecutive images. This partial map of gaps is injective. Its image lies inside a complementary image strip and therefore contains no gap internal to $E$. Thus distinct internal entrance gaps start disjoint gap paths. A gap between different itineraries must hit one of the $H+s-1$ internal boundaries of the domain partition before both sides exit in the same component. Charge it to its first such boundary. Each boundary receives at most one charge. Add one first bundle per entrance interval. Finally $e\le H+1+\sigma-1$ and $s\le H+1$. ∎

The count controls the number of instructed intervals. For bundle width $b_i$ and complementary path height $h_i$, a representative tracer pays $U=\sum_i h_i$ and accounts for area $A=\sum_i b_i h_i$. In a primitive block $A=n-m$. In a general column the omitted area consists of cycles wholly in the complement. The quotient $A/U$, when defined, is the height-weighted mean multiplicity. Long words shared across distinct bundles can reduce actual dictionary storage further. Neither the bound on bundles nor a small $H$ bounds $U$ by a constant multiple of $H$.

A single thick strip of length $w$ translated by $d\ne0$ makes the distinction explicit. There are $\min(w,|d|)$ entrances to that strip. With $0<|d|\le w$, successive steps advance by $d$ until exit, so the heights are $\lfloor w/|d|\rfloor$ and $\lceil w/|d|\rceil$, omitting a height with zero multiplicity. A shift by one and a disjoint image realise the vertical and horizontal extremes. The executable twelve-row examples keep the ordered widths, the alphabet counts and the primary fixed. [Independent control](screen.py)

## Computing the open map by endpoint induction

Make an auxiliary exchange on a prefix of $m$ terminal points followed by a shifted copy of the $n$ original rows. Its arrows are

- terminal $k\to m+e_k$, with empty word,
- $m+x\to m+f(x)$ on the $H$ complementary strips, with their original bytes,
- $m+s_i\to i$, with empty word.

Both terminal families coalesce on their original contiguous components. At most $3H+2$ arrows initialise the entire exchange. Its first return to the prefix is exactly $P$, labelled by $w$, because the two empty arrows delimit each path. This construction consumes no internal retained arrow. Cycles confined to the suffix have no prefix output.

Remove the rightmost rows until only the prefix remains. If the last domain strip is $X$ and the last image strip is $Y$, remove the common suffix of their widths. The inverse writer needs the reversal of the path entering through $Y$ and leaving through $X$, hence the concatenation of the reverse words of $X$ and $Y$ in that order. The shorter strip's record can hold the composite. If $X=Y$, its rightmost interval is fixed and disconnected from the prefix, so remove it. The prefix boundary is present in both orders initially and can never be crossed by a suffix cut. The live strip count never increases. This proves that endpoint induction and the gap tracer give the same labelled map pointwise, including unused components of the *retained* system.

The executable uses $K$ endpoint steps and $G$ nonempty binary rules. It materialises the resulting band dictionary with cached first occurrences of rules. If $R_c$ is the number of capped run records, its work before ordinary byte output is $O(R_c+H+K+G+B+m+U)$, where $B$ is the number of bands. The tracer replaces $K,G$ by representative visits and a directory of $\lceil n/64\rceil$ entries. A fixed page bounds a tile search by 64 integer positions. The packed residual table uses four bytes per retained row when the row and word code fit in one word, otherwise eight. The implementation also stores the dictionary, construction scratch and ordinary decoder buffers. Every successful decode still writes the requested output bytes.

## Cutting only at colour changes

Colour each original arrow narrow or wide. A switch row $x$ satisfies $c(f^{-1}x)\ne c(x)$. The domain and image run tilings give the whole switch set in a simultaneous sweep. The number of switches in either direction is equal, since the cyclic indicator has as many rises as falls on every component. If $S$ is the narrow domain and $E=f(S)$, that number is

$$
       h=|E\setminus S|=|S\setminus E|=m-|E\cap S|.
$$

Consequently the alternating cycle has $h$ pairs of monochromatic excursions across all mixed components. The program takes all $2h$ switch rows and adds $p$ if necessary. Its one-arrow interpreter visits the switch rows individually. A marked cut can split one excursion. A monochromatic marked component becomes a single marked return. An arbitrary disconnected component without a switch or mark is irrelevant to the output.

This census is an exact directed cut of the weighted run adjacency graph. The edge from runs $I,J$ has weight $|f(I)\cap J|$. It can be counted before any return word is discovered. The cut measures serial return visits for this representation. The two height-weighted multiplicities measure the independent cost of spelling each colour. A small cut alone gives no bound on that cost.
