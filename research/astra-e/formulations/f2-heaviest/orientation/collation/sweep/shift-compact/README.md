# Delete the conjugating section images before native Compact induction

Let U be a contiguous section, Vj=Ψ^j(U), and require each V0,…,V(h−1) to lie in one outgoing-byte bucket. The resulting shared prefix P is scanned from the original capped L runs; no return dictionary, first-return length, or dense Ψ is constructor input. The sorted order of every level agrees under Ψ because it is stable on a common F byte. The target is Vh, with raw source-oriented words. A return formerly labelled W is rotated left h times, including modulo its return length for short cycles. The compressed section permutation is unchanged. [Shift theorem](../rotation.md)

**Rank projection.** Put F=V1 ∪ … ∪ Vh, D=[0,n)∖F. A fused live suffix class is its old contiguous U interval with F removed. Deleting F from the rank coordinate makes every surviving class exactly contiguous again. In particular the native eight-lane discovery needs neither sparse packet lists nor a per-lookup hole test. Since y=LF(x)∈Vj for j<h would imply x=Ψ(y)∈V(j+1)⊆F, the image of every x∈D lies in D or in the terminal Vh. For h=0, D is the entire original row universe and arrivals into U terminate as usual.

The constructor scans original L runs in input order. It removes pieces whose sources lie in F. A remaining input piece maps either into D, in complement rank, or into Vh, in target ordinal; a tagged terminal descriptor distinguishes these two namespaces. It coalesces adjacent pieces only when byte, namespace and new displacement agree. The resulting 8-byte descriptors use the same packed retirement comparison and popcount location as Compact. Every unary lookup and every resulting nonterminal child uses a single contiguous compressed interval. The target spans, forbidden union, induced ordinals, original-run clipping and rank directory are all built on the measured path.

**Early words and mark.** Beginning at a target destination, the first h backward steps follow the common prefix unless they encounter Vh sooner. The root checks earlier levels in decreasing order, by the shared level ordinal. It emits the earliest positive return as P[j:] with its actual compressed source base, excludes those destination ordinals from further search, and splices the early labels into their exact destination positions after regular stable merging. Remaining destination ordinals descend from U∖F, one contiguous interval in D. A normal terminal spells its crossing byte, the backwards discovery path in reverse order, and P. Generic terminal ownership supplies one first-level word per distinct spelling; cases with a common terminal byte and no early hits use one owner per parent.

For a mark in F∖Vh, move one original LF step via any containing level's shared ordinal and the preceding level. At most h−1 such steps occur before hitting the target or reaching D. For a mark in D, follow the projected LF until a tagged terminal returns the true target ordinal and record every skipped byte. A marked orbit missing Vh lies entirely in D, so one reverse projected cycle gives its exact period for independent repeat/clipping to n. The original second token decoder consumes the raw first dictionary, recorded bases, stable root and marked offset. Exact first-token links handle a refused second induction, including the wide-link fallback. The compact table never interprets a target ordinal as a live D rank.

**Checks.** Every binary column through length seven and ternary column through five, every contiguous U, every h≤3 with a monochromatic shift and every primary mark passed an independently stable-sorted dense construction. This covers 143,450 marked objects, including 53,372 with early returns, 36,284 entirely early roots, and 48,772 disjoint marks. All forty input/shift objects on the ten registered Wikidata tapes matched their independently walked raw dictionary bytes, bases, maximal destination runs, marks and complete output. All used the unchanged Rust second induction. Frozen h3 removes 72,495 interior ranks, executes 91,846 logical suffix lookups, and produces 50,741 terminal-labelled LF pieces versus 51,184 original maximal L runs. The earlier physical implementation needed 51,446 LF pieces and 29 extra live packets there. [Tiny receipts](small.txt) · [All forty geometries](ten.csv) · [Dense forbidden partition](../shift-constructor/partition.csv)

Build at the repository root using the release benchmark library, Rust toolchain and the checked Compact/clipped sources. The named frequent interval for numerical comparisons is the same fixed prior registration across all shift arms. Its selection is outside the shift kernel, separately verified against original Compact's source bucket counts and the independent C++ input manifest. Constructing the images of that fixed interval and recognizing when the requested shift exists are always charged.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/shift-compact
out=/tmp/astra-shift-rust
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/check"
python3 -B "$d/../../clipped/native/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check" check "$out/ten.bin"
python3 -B "$d/check.py"
```

[Projected LF construction and mark transfer](index.rs) · [Section images and early-return plan](projection.rs) · [Native wrapper and oracle](main.rs)
