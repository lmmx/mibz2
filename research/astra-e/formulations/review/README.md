# Independent review of delimiter and monotone token BWTs

The one-level first-return and token-order identities hold on arbitrary nonempty last columns with a valid mark. The current F4 Python prototype has an implementational limit when a subsequent token alphabet is substantially wider than the original byte alphabet. A valid 8,075-byte example reproduces the failure. These are correctness and construction tests. They measure no decoder throughput.

The reviewed pins are `488646a95c1ba8f8618df98a7b32cd34af76f9ea` for F2 and `23b2c725c4148a4aa9806ac06aed3d7cdd01e2f4` for F4. [F2 source](https://github.com/lmmx/mibz2/blob/488646a95c1ba8f8618df98a7b32cd34af76f9ea/research/astra-e/formulations/f2/factor.cc) · [F4 source](https://github.com/lmmx/mibz2/blob/23b2c725c4148a4aa9806ac06aed3d7cdd01e2f4/research/astra-e/formulations/f4/phrases.py)

## Proof check

Write F for the sorted last column and Ψ for inverse stable LF. The only tie-order fact needed is that Ψ is strictly order-preserving on each equal-symbol F bucket. For rows emitting the same finite word of length k, their order is therefore preserved by Ψ to the kth power. Conversely, the device can distinguish the first unequal symbols at the first bucket where they differ. No appeal to a unique cyclic suffix order is needed. A return section meeting a component partitions it into paths from one section row up to the next. The restricted return permutation is invertible. Its stable LF equals the proposed token LF precisely when outgoing word classes occur in token order amongst section starts and targets of equal-word paths have the same relative order as their starts.

For F2 the finite forward word to the first delimiter (inclusive) is unique on every component that has a delimiter. These words form a prefix-free family. Their intervals are disjoint; intersecting q such nonempty row intervals with r runs adds at most r−1 splits beyond the q initial pieces. For a suffix w and row y in its interval, Ψ to the power |w| maps y to its associated section endpoint. If L[y] is a delimiter, y is a section start and w is its full outgoing word. Otherwise LF(y) extends the suffix by L[y] without changing the endpoint. Stable LF preserves the ordering of each character-selected subsequence. Thus a selector consuming a particular child's labels from left to right never changes their endpoint assignment. This justifies the inverse merges and the filtered root. Counting section rows before the interval and within it yields the start and mass of the token's first-column bucket. Equal copies stay in order. A component without any delimiter contributes no interval and no token. A mark on that component requires fallback even when other components emit tokens.

For F4 the predicate L[r] ≥ F[r] selects an ascent-phrase start. On a cycle of a finite ordered alphabet, the preceding boundary exists and lies at most σ−1 backward steps away. Order preservation for the common prefix settles equal words, and differing bytes compare by F buckets. If the words share all of the shorter prefix ending at c, the shorter one's next byte is ≤ c while the longer one's is > c. This establishes ordinary shorter-first lexicographic order on ascending phrases, including distinct copies of equal rotations. For a descending phrase the terminal comparison reverses: the shorter one's next byte is ≥ c and the extending byte is < c. Trie edges are still in ascending symbol order; terminals sort after every extension. Both arguments are valid for any ordered token alphabet at the next level. They do not require a primitive whole column. At each lift, recover the full marked period in the outer alphabet, expand it into the inner alphabet, and apply **that level's** swallowed-mark offset in its own symbols. Only at the final byte level may one truncate the repeated period to n.

A compact witness handles several cases at once. The last column `0110000` is formed by tagged copies of cycles `001`, `001`, and `0`. With delimiter 1 the first two token cycles are distinct one-row self-loops, both labelled `001`. Mark 0 requires fallback on the delimiter-free cycle. Mark 6 is in the second copy with a rewind of two bytes; it emits `1001001`, with a partial last period. For the same mark, F4 ascent gives a one-byte skip within phrase `01`, and the exact same seven output bytes.

## Separate construction of the reference rows

The checker forms multisets of primitive necklaces. Each row initially carries an explicit necklace-copy ID and a phase. It sorts periodic strings with the finite test `uv < vu` for infinite repeats of u and v, breaking equal-rotation ties consistently by copy ID. The indexed phases supply the predecessor, successor, word and arbitrarily long marked output directly. The oracle never obtains those from the input column's LF. For the exhaustive populations it checks that the resulting last columns are distinct and that their number equals the total number of arbitrary columns at every tested alphabet bound.

The adapter calls the actual F2 constructor and decoder without the author's dense validator. The script loads the actual F4 transform, then independently checks its token last column, ordered dictionary, residual mark and offset. It lifts nested periods independently. The populations cover binary columns through length 9, ternary through 6, quaternary through 4, and 35 deterministic larger multiset cases containing duplicates. All delimiter masks were checked for F2. Both directions and all primaries were checked for F4. Two-level checks use all four direction pairs on the small population through length 5.

| Observation | Result |
| --- | ---: |
| Columns including larger cases | 2,489 |
| Columns with tied infinite rotations | 990 |
| Columns containing a cycle whose length does not divide n | 1,379 |
| F2 factorizations, exact section arcs, marked cases | 18,792 / 63,372 / 126,744 |
| F2 fallbacks, swallowed marks, cases with both covered and uncovered components | 32,679 / 30,693 / 5,677 |
| F4 exact arcs, marked cases, swallowed marks | 25,873 / 34,646 / 8,773 |
| F4 token words with an occurring proper prefix across orientations and cases | 3,552 |
| Nested F4 marked lifts | 12,604 |

## F4 depth witness

Take the first 1,001 lexicographically ordered distinct byte words of the form `0, increasing subset of 1..11, 255`. Concatenating them gives a primitive cyclic byte string. Each word increases internally and every transition `255,0` is a boundary. Each occurs once, so the next-level text is `0,1,…,1000` and its last column is `1000,0,1,…,999`. Its next increasing phrase has length 1,001. At the default Python recursion limit of 1,000 the recursive dictionary-order trie walk raises `RecursionError`. The exact theoretical answer is one token spelling the full 1,001-token sequence. The included authentic bzip2 block round-trips with libbzip2 and requires no initial RLE1 transformations. The reproduction script reconstructs its original byte BWT by cyclic prefix-doubling, runs both levels, and checks the first level against the predicted dictionary and last column. A stack-based preorder/postorder trie traversal removes this interpreter call-depth limit. The one-level byte theorem is unaffected. [Witness](deep.bz2) · [Observed failure](deep.json)

## Reproduction

```sh
d=research/astra-e/formulations/review
c++ -std=c++17 -O2 "$d/f2_adapter.cc" -o /tmp/astra-f2-review
PYTHONDONTWRITEBYTECODE=1 python3 "$d/check.py" /tmp/astra-f2-review
PYTHONDONTWRITEBYTECODE=1 python3 "$d/deep.py" /tmp/astra-f4-deep.bz2
cmp /tmp/astra-f4-deep.bz2 "$d/deep.bz2"
```

The reference receipt includes independent-query and reply digests. Re-run at the reviewed pins to reproduce the Python prototype failure. Later iterative implementations can return the checked singleton dictionary instead. [Receipt](receipt.json)
