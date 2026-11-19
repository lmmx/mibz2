# Selecting deeper delimiters without constructing the intermediate token BWT

Let S=I(P) be a nonempty word cylinder in the original marked byte BWT. A complete first-return word X runs from a starting row of S to its next visit to S in forward byte order. Its source rows form one contiguous interval I(XP) inside S. Occurrences of XP are exactly these sources, including the case |X| < |P|. The proof reads the return from its other endpoint. Start from S and prepend letters using LF until the first positive hit of S. Each partial spelling u is a cylinder I(uP). Its child is either entirely inside S or disjoint, because its leading |P| bytes already determine membership. A terminal records all occurrences of that X and their exact source ranks. The most frequent terminal therefore selects an exact next byte cylinder inside the current one. Repeating the operation performs arbitrary-depth return induction by transitivity. An interval's two endpoints suffice to continue searching or reconstructing from the selected section; its spelling is only needed if the caller asks to name the marker. [Earlier one-level conjugacy](../f2-word/)

## Bound on the price of selecting the most frequent return

Keep unresolved intervals in a max-heap by width. The width bounds the frequency of every descendant. Record the widest terminal found, breaking ties by smallest source rank. Stop when the largest remaining width is strictly below the incumbent. Whenever an optimal terminal remains undiscovered a live ancestor has width at least its frequency f. Every expanded partial interval therefore has at least f rows. Nonterminal partial-return cylinders are disjoint across all depths: any row reaches S at a unique first future distance, counting distance zero at the root, and its emitted bytes uniquely identify its partial spelling. Thus at most floor(n/f) intervals can be expanded. If the unvisited cycles are excluded, n can be replaced by the total size of the cycles meeting S.

A binary predecessor search followed by a linear scan of original runs exposes all child intervals. For H expanded, disjoint intervals and r maximal input runs the number of clipped run pieces is at most H+r−1. Every clipped piece after the first in an interval crosses a distinct original run seam. Summing widths by byte within a parent incurs no sorting of row positions: stable LF maps that byte's pieces to adjacent ranks. With a fixed byte alphabet, lazy clearing of touched slots costs one operation per encountered symbol. A binary heap adds logarithmic work for each candidate pushed/popped. A spelling-free implementation needs live heap plus alphabet scratch and the supplied FM run index. The Python proof program retains additional parent pointers to report the winning strings. Cheap certifying selection does not imply cheap final inversion.

**Sharpness.** For any marker length m≥1, choose L>m and the primitive cyclic word aᵐbᴸ⁻ᵐ. Use the cyclic BWT of f repeated copies, with P=aᵐ. There is one P per primitive period. Every one of the f identical section rows has exactly the same length-L return. The width-priority constructor follows L disjoint intervals of width f, so H=L=n/f and the expanded mass is exactly n. Taking f>1 also exercises multiple stable LF cycles with identical spellings. This is sharp for the interval-expansion algorithm. A constructor able to recognize the periodic family can bypass that work. [Executable construction](sharp.py) · [240 exact cases](sharp.txt)

## Evidence

The dense stable-permutation oracle explicitly follows each source to its next section visit, aggregates complete byte strings, selects the maximum with the same tie rule and checks its source interval. All 15,702 exhaustive selections agreed, including 10,962 returns shorter than their current marker. In 4,846 cases the chosen primary's cycle missed the selected section, requiring the existing mark fallback. All 40 selections at four successive depths on ten systematic Wikidata blocks matched. [Programs](audit.py) · [Small-column receipt](small.txt) · [Every selection, heap, run and mark observation](wikidata.csv)

On frozen, the quote section has 135,716 rows and 2,716 distinct return words. The winning two-byte return occurs 24,165 times. Four expanded intervals and 5,243 clipped run pieces certify it. From those 24,165 rows, 39 expansions and 420 pieces select a 19-byte return, leaving 3,232 rows. The same first two winning markers occurred on all ten blocks. At the third selection frozen needs 520 expansions and 1,375 pieces to find 241 occurrences. At the fourth the best frequency is one, and the algorithm needs 874,086 expansions. The fourth section's distinct spellings already total all 899,931 raw bytes. This shows both where selection is cheap and how rapidly dictionary costs can overwhelm the smaller residual. The measured counts exclude timing of the Python oracle and do not imply a worker speedup.

```sh
d=research/astra-e/formulations/f2-heaviest
python3 "$d/audit.py"
python3 "$d/audit.py" research/astra-e/dense/target/wikidata
```

The ten compressed input tapes can be regenerated by the [dense-census recipe](../../dense/README.md).

## Paid native constructor

Source `2b19f2e` implements the two successive choices in the original run index. It keeps just (width, starting rank) per heap item, one table of touched symbols and the winning interval endpoints. It uses the two interval certificates to skip both intermediate token BWTs and both intermediate dictionaries, transfers the original mark once, and reconstructs directly from the final interval. The mark can miss that section on an arbitrary column even when it meets the initial quote section. The experimental worker then uses its existing fallback. The forced small oracle obtained 22,303 exact nested outputs and 10,473 section refusals. The fully integrated worker passed all 338 v1 cases. All 228 real admissions produced 205,197,794 checked inverse bytes. [Source](candidate.rs) · [Native cases](native-small.log) · [Official gate](../../results/v1-heaviest-gate/score/summary.txt)

Thirty rotated paired stage measurements compared the twice-selected section against the fixed three-byte direct constructor and Lazy on nine systematic blocks, with independent byte checks outside every time interval. Selection and mark transfer together took 0.142 ms summed over block medians. The resulting return spellings cost 61.51 ms to discover, compared with 20.00 ms at the fixed marker. Residual output fell to 0.310 [0.266,0.360] of fixed, but the whole inverse took 2.356 [2.238,2.480] of fixed and 3.600 [3.455,3.751] of Lazy. Across all admissions, token visits fell from 5,566,622 to 705,565 while original-column construction lookups rose from 28,053,831 to 89,002,161. Explicit dictionary spelling grew from 41,433,719 bytes to 125,949,883. The cost lies in discovering and representing long return words after the section is known. [Paired results](summary.txt) · [All block counts](native-geometry.csv) · [Individual phases](phase.csv)

For occurrences of word w of length ℓ(w) and frequency f(w), let N be the total size of the LF cycles that meet the section. The disjoint arcs between successive section visits partition those cycles. Therefore Σ f(w)ℓ(w)=N. Any implementation storing one explicit copy of every spelling uses D=Σℓ(w) ≥ ceil(N/max f(w)) bytes for those spellings. In particular f=1 forces D=N; the fourth frozen selection provides a real example. N can be smaller than the original block length on arbitrary multi-cycle inputs. An implicit grammar or a different output owner may avoid this explicit-copy cost. The actual native counters above distinguish the bytewise discovery and final spelling costs from the combinatorial price of selecting the section.

```sh
d=research/astra-e/formulations/f2-heaviest
python3 "$d/prepare.py"
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml selected_cycles_and_marks -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/f2-heaviest-geometry.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
bash "$d/price.sh" /tmp/f2-heaviest-price "$PWD/bench/moonshot/target/release/deps"
python3 "$d/summarize.py"
```
