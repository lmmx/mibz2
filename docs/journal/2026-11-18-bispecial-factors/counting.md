# Counting the ambiguity left by a set of contexts

There are two finite counting problems here. An arbitrary last column describes a multiset of
primitive necklaces. Requiring the decoded block to be one primitive necklace constrains the
local orders. The difference can be computed exactly. All spectra below are cyclic and count
positions with multiplicity. The empty context is allowed.

## A fixed spectrum of length $k+1$

Write $N_w$ for the number of occurrences of $w$. For every context of length $k$, the preceding
symbol histogram in the row interval $I(w)$ is $(N_{aw})_a$. The interval's width is $N_w$ and
the intervals occur in lexical order. The number of possible last columns with the given spectrum is

$$
\Omega_k=\prod_{|w|=k}\frac{N_w!}{\prod_a N_{aw}!}.
$$

The local choices really are independent in this counting problem. Within each interval, arrange
its stated incoming symbols arbitrarily. Stable LF puts its $a$ rows into $I(aw)$ in order.
Summing the supplied counts gives the corresponding margins at each shorter context. Starting
at the alphabet partition and refining successively produces exactly the specified context
intervals for every such arrangement. Conversely, a column with this spectrum has to realize these
histograms. This argument also covers disconnected inputs.

The matrix $M_{ab}=N_{awb}$ refines $I(w)$ into its right-extension intervals. Write
$r_a=N_{aw}$, $c_b=N_{wb}$, and $N=N_w$. The fraction of that context's previous arrangements
which survive the refinement is

$$
\frac{\prod_a r_a!\prod_b c_b!}{N!\prod_{a,b}M_{ab}!}.
$$

It is the probability of the particular contingency table when incoming symbols are uniformly
shuffled among the fixed right groups. The product of these fractions over a level is
$\Omega_{k+1}/\Omega_k$. If either side has only one possible symbol, the factor is one.
Taking logarithms makes the reduction in ambiguity additive. On the frozen block, knowing the
byte histogram leaves $\log_2\Omega_0=4{,}590{,}793.074$. The bigram spectrum leaves
2,634,155.238 and the trigram spectrum approximately 1,305,406. These are base-two logarithms of
class sizes. They depend on the supplied spectrum. They do not price the instructions to discover
a spectrum or the best code for communicating a structured table.
[Full depth trace](evidence/frozen-levels.csv)

There is a similar exact interpretation for a delimiter dictionary. With $m$ markers, phrase
multiplicities $h_w$ and $n$ byte positions, the number of **indexed texts** with that phrase
multiset is $(n/m)\,m!/\prod h_w!$. Count pairs consisting of an indexed text and one distinguished
marker occurrence. One can choose its byte position in $n$ ways and then a linear order of the
phrases. Every indexed text contributes exactly $m$ such pairs, including periodic texts.
The marker measurements use this formula. The examples in the journal fix the unique origin.
[Marker census](evidence/frozen-markers.csv)

## Where independent weights could enter

For one observed extension graph let $E$ be its number of edges, $L,R$ the numbers of left and
right vertices, and $c$ the number of connected components. Given its support and margins,
one can specify the edge counts by at most $\beta=E-L-R+c$ independent integer fields. Take a
spanning forest. Supply the counts on the nontree edges, subtract them from the margins, then
strip leaves. At a leaf the one remaining edge must carry the outstanding count. Positive
integrality may force even a nontree count. The weighted reference codec subtracts one on every
present edge and skips a chord field if its residual upper bound is zero.

There is an exact consistency check on the total cycle rank. Let $p(k)$ denote cyclic factor
complexity. Summing $E-L-R+1$ over all length-$k$ factors gives
$p(k+2)-2p(k+1)+p(k)$ by counting all two-sided, left, and right extensions. A one-sided
branch has a star graph and contributes zero. Sum over $k$ until $p(k)$ stabilizes. If there are
$s$ occurring bytes,

$$
\sum_{w\,\text{bispecial}}(c_w-1)
=\sum_{w\,\text{bispecial}}\beta_w+s-1.
$$

The frozen totals are $32{,}328=32{,}240+89-1$. Of its 22,261 bispecial factors, 20,296 have
forest support, 32,240 chords occur across the others, and 22,945 residual chord counts are
actually sent by the weighted codec. In the unweighted inverse **all** these fields disappear.
The weighted calculation answers a conditional question about a single level. The later support
can determine the weights of an earlier level.

## Requiring one cycle

Build the directed graph $G$ of length-$k$ factors. The edge of type $e$ for a length-$(k+1)$
word goes from its length-$k$ prefix to its length-$k$ suffix and has multiplicity $N_e$.
The degrees $N_v$ are balanced. One may reverse all edges to follow the BWT in the LF direction.
Let $t$ be the number of directed spanning trees toward any chosen root, with parallel edges
distinguished. The directed matrix-tree theorem gives $t$ as a cofactor of the outgoing graph
Laplacian. The cofactors are equal on a connected Eulerian graph. A disconnected graph has $t=0$.

First distinguish every edge copy. A tour starting at vertex $v$ specifies a linear order of
the exits at each vertex. The last exit at every vertex except $v$ forms a tree toward $v$.
Conversely, given such a tree, put its edge last at each nonroot vertex and order the other
exits freely. Consuming exits in those orders traverses all edges, because leaving unused
edges behind would violate the last-exit tree. This is the BEST theorem. The count for that
root is $t\,N_v!\prod_{u\ne v}(N_u-1)!$. Divide by $\prod_e N_e!$ to forget copy labels.
Summing over $v$ gives the number of indexed cyclic texts with this spectrum, including powers,

$$
W(E)=n\,\Omega_k\,\frac{t}{\prod_v N_v}.
$$

If the edge counts have greatest common divisor one, all such texts are primitive. Dividing
$W(E)$ by $n$ then gives the number of compatible single-cycle last columns. For general
multiplicities first count primitive indexed words by subtracting repetitions, equivalently
$\sum_{d\mid\gcd_e N_e}\mu(d)W(E/d)$, and then divide by $n$. The Python check enumerated
3,259 complete spectra of small binary and ternary inputs and verified all three counts.
For the two binary cyclic de Bruijn words `00010111` and `00011101`, every trigram occurs once.
The unrestricted spectrum admits 16 byte columns and just two primitive necklaces.
[Whittle on Markov type counts](https://doi.org/10.1111/j.2517-6161.1955.tb00197.x) · [Proof of the BEST theorem](https://www.cip.ifi.lmu.de/~grinberg/t/22s/lec16.pdf) · [Executable enumeration](count_classes.py)

For the frozen block the byte frequencies have gcd one. Every higher-order gcd divides it, since
summing extensions recovers the byte frequencies. At order zero the single-cycle restriction
removes $\log_2 n=19.779$ from the log class size. At order one the exact tree correction is
22.541 bits. The number of mark choices is a separate quantity.
[Exact integer cofactors](evidence/frozen-trees.csv)

## Computing the threshold without a huge determinant

The frozen block is one primitive cycle. Work in its backwards order and retain vertices $B$
which have more than one distinct outgoing edge type, equivalently more than one preceding
byte in their context interval. At a nonbranching vertex $v$ all $N_v$ exits lead to the same
next vertex. If $B$ is nonempty, these deterministic paths eventually reach $B$ by strong
connectivity. Contract them. Let $A_{uv}$ count exits from $u\in B$ whose next visit to $B$
is at $v$. These numbers can be read off one traversal of the marked text.

Choose a root in $B$. Every omitted vertex of an arborescence can choose any of its $N_v$
parallel edges. After contracting the deterministic paths, choices at the retained vertices
form a tree exactly when the original choices did. Thus the omitted degree factors cancel,

$$
\frac{t(G)}{\prod_{v\in G}N_v}
=\frac{t(A)}{\prod_{v\in B}N_v}.
$$

Likewise only mixed incoming histograms contribute to $\Omega_k$. If $B$ is empty,
$\Omega_k=1$ and the supplied primitive text already realizes that unique column. The script
uses cyclic suffix order and adjacent longest common prefixes to find the mixed intervals.
It uses the known frozen raw bytes for this **identifiability calculation**. The separate support
decoder receives only its tape. The contraction was checked against the uncontracted count in
2,498 small cases before the frozen calculation.

At spectrum length 4,744 one interval has three distinct preceding bytes. Six possible orders
give four multicycle columns and two primitive alternatives. At length 4,745 one interval has
two distinct incoming bytes. Its alternative breaks the marked cycle into cycles of lengths
442,958 and 456,973. An independent verifier constructs the two competing texts at the earlier
length and compares their cyclic spectra by exact substring equality. It assigns integer IDs
to pairs of equal-length halves and compares two covering spans for the requested length.
[Contraction and witness generator](assembly.py) · [Independent verifier](compare_spectra.rs)
