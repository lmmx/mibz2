# Exact optimization of type-preserving cut shifts

Drop the requirement that moving a cut preserve its global rank. The original and physical sections are a stable shuffle of the moved and unmoved sources. Its two monotone streams admit a compact recovery of the original marked permutation. If every original return type A is to have a single distinct new spelling, all types following A must agree on their move bit. Write x_g for the bit of each resulting follower class and t_A for the bit common to A's followers. For A=cV the new spelling is V followed by t_A copies of c, with an additional leading c when A's own class is unmoved. Exclude the empty spelling and collisions between distinct A. Positive arc lengths then make all moved positions distinct in orbit order. [Physical shuffle, raw token interface and marks](../shuffle/)

On frozen, the quote dictionary has 19 class bits and 2,716 word types. The frequent dictionary has 815 bits and 2,950 types. Exact optima for proper-suffix lookup count are **67,930 and 97,695**, respectively. Moving every cut attains each optimum. A mixed choice can still change the number of distinct dictionary bytes or the cost of representing its rank components. A separate [spelling bound](BOUND.md) settles the more permissive per-occurrence case on all twenty real section/block combinations.

**Finite model and independent certificate.** For each allowed terminal bit t, form S=V·cᵗ. Every nonempty proper suffix of S is a lookup state whenever A's follower bit equals t. The whole S is also a proper suffix if A's source bit is zero and S is nonempty. These are all possibilities. Insert reversed spellings in a trie; give each distinct state a unit activation if any one of its owner conditions holds. Conditions entail at most two bit assignments. Identical sets of owners share one activation variable with multiplicity equal to their number of states. Constraints forbid empty new words and multiple original types activating one full spelling. Including the ever-present empty suffix gives precisely the desired count. On frozen the two models have 133 and 5,209 variables including continuous auxiliaries, and 335 and 17,875 linear constraints.

Only the class bits are integral. HiGHS finds feasible witnesses and multipliers from the box-constrained linear relaxation. The separate verifier requires no optimizer invocation. It rebuilds the constraints as Ax≤b and checks a nonnegative scaled multiplier p with integer arithmetic. Every feasible unit-box vector has objective at least the constant plus the sum of the negative coordinates of c+Aᵀp minus pᵀb. Scaling p by a common denominator before computing this expression gives a signed-integer numerator. The ceiling of this lower bound equals the witness's exact suffix count on both dictionaries. The original dense rotations also verify the actual spellings and physical successors for every selected cut. [Frozen quote witness and dual](frozen-quote.json) · [Frozen frequent witness and dual](frozen-frequent.json)

Exhaustive checks covered 3,472 bit assignments for 1,344 small sections. Forty-four independent solver runs agreed with brute-force enumeration. The certificate verifies an optimum among this class of one-byte cut choices. Constructing the old dictionary, finding the assignments, recovering the rank shuffle and building a physical incoming column each cost additional work.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/choice
t=research/astra-e/dense/target/wikidata/0000.irec.xz
OPENBLAS_NUM_THREADS=1 python3 "$d/optimize.py" | diff - "$d/small.json"
python3 "$d/verify.py" "$t" "$d/frozen-quote.json"
python3 "$d/verify.py" "$t" "$d/frozen-frequent.json"
```

The optimization uses SciPy's mixed-integer and continuous solvers. To produce a fresh solution use `python3 "$d/optimize.py" "$t" frequent /tmp/frequent.json`. Solve time and the particular dual returned may vary. [Model](optimize.py) · [Verifier](verify.py)
