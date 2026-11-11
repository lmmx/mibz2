# Labelled first-return sanity checks

Created during a research discussion of Quentin de Mourgues's labelling method
and combinatorial surgery in relation to mibz2. These are independent, tiny
mathematical checks, not a mibz2 patch, reproduced real-corpus experiment, or
performance claim. No repository was modified.

## Run

    python check_splice.py
    python check_origin_labels.py

Only the Python standard library is required. Saved output is in results.json
and origin_results.json, respectively.

## What is checked

check_splice.py checks the elementary first-return splice for a finite permutation
F when the removed set D has no consecutive points (D intersect F(D) is empty).
It preserves forward-travel itinerary words and is conjugate to its packed-coordinate
version. All permutations through size 6 and all eligible D are enumerated.
Nested first-return composition is checked for all nested nonempty retained subsets
of permutations through size 5.

check_origin_labels.py tests a different origin-handling method for delimiter-based
first-return construction. Instead of separately walking the original marked row
to an anchor, it notices which already-processed interval contains that row and
records the source-anchor rank plus the word-prefix length. All binary columns
of lengths 1 through 8, both delimiter choices, and every origin are checked against
an explicit row oracle. Unanchored selected cycles correctly decline. The implementation
uses expanded words and point-level output maps for tiny-test clarity, not economy.

It also computes a counterexample to blindly adding the origin as an extra anchor:
text bab, BWT bba, origin 1, delimiter a. The augmented first-return map is valid,
but is not LF of its phrase IDs under the prototype's lexicographic token ordering.

## Limits

These checks do not establish a transplantation of de Mourgues's monodromy or surgery
theorems to weighted LF dynamics. They do not establish an efficient sparse interval
representation, bounds on fragmentation, real-world savings, or production decoder
error/CRC/concurrency compatibility. Forward-LF word orientation is explicitly reversed
when comparing decoder output. Finite tests corroborate the stated elementary identities;
they do not replace proofs of more general claims.
