# Consume contiguous subtree intervals in root order

The inverse merge only needs the root token run stream. At node v, let K(v) count the maximal contiguous root intervals whose labels belong to its subtree. Let G(v) count root runs in that subtree. One such interval may contain many different terminal labels, so K(v) ≤ G(v). The original root cursor restarts after each terminal run. A frame can instead retain ownership of an entire interval, including across multiple emitted runs.

Start the root with its whole section width. A call at node v with width h takes the next h positions of its local selector tape. For each overlap with a selector it emits the current node's terminal or calls the selected child with that overlap width. A pure terminal child can be emitted immediately. The Rust loop uses an explicit vector of suspended (node, remaining width) pairs and tail-enters children when their parent's frame ends. The stack grows only with physical depth. No recursion limit applies to the input.

**Invariant.** An invoked frame is one maximal root interval whose labels belong to its node. This holds initially. Intersect a parent's interval with its local selectors. Consecutive distinct choices have disjoint terminal supports, and successive invocations of the parent are separated by an outside label. Thus every child overlap is exactly one maximal root interval belonging to that child. Consumption order is the original root order and the stable child projections advance successively. A terminal overlap has that same label throughout; either bordering root row, if present, takes a different choice or lies outside the frame. Therefore each emission is one maximal root run and the accumulated widths cover the root.

With T root runs, exactly **T + Σ K(v)** selector overlaps are executed, summed over proper nonleaf nodes. Each overlap emits one of the T output runs or enters a nonleaf frame. After leaf absorption the restart visitor instead examines T + Σ G(v) selectors over those same internal nodes, except for its dedicated constant-root case. The number of suspensions can be smaller than the number of entered frames. Both constructions store only the T root pairs. The explicit stack uses two u32 fields per suspended ancestor. The dominantly reopened deep-chain family still has O(n) work and can have Θ(depth × T) overlaps. Multiple contiguous terminal changes in a child create a strict improvement.

An independent dense LF and recursive binary-search oracle counted 18,707 frozen overlaps, compared with 21,885 selectors on restarting at the root after leaf absorption, and 27,156 before absorption. Across all 228 real admissions the corresponding counts are **5,448,240, 6,254,289 and 7,739,733**. The retained-frame implementation made 1,596,200 actual suspensions and its root outputs exactly matched Segmented on 205,197,794 bytes. The independently computed overlap formula agrees on all nine admitted systematic inputs. The geometry table's historical `merge_steps` column counts overlaps and `token_visits` counts explicit suspensions. Its scratch-capacity column omits the stack and second induction. [Native geometry](geometry.csv) · [Ten dense input sections](intervals.csv)

The independent recursive oracle also checked 55,694 small ordered trees and tapes, 252,638 maximal root runs, and 371,724 selector overlaps against known row labels. The native small-column test checked 11,264 admitted marks plus explicit pure-root and single-cycle deep re-entry cases. [Exhaustive evidence](small.json) · [Native checks](property.log)

Forty matched rotations with four arms on nine systematic blocks reduced merge CPU relative to the restarted leaf cursor to .776 [.660,.913]. Complete inverse CPU was .927 [.881,.976] on that comparison. Relative to the materialized dynamic refill, merge CPU was 1.418 [1.188,1.692] and whole inverse CPU was 1.051 [.999,1.107]. Relative to Lazy the whole inverse was 1.038 [.989,1.089]. This improves the tested root-only strategy without establishing a worker, RSS or target-machine victory. [Raw phase observations](phase.csv) · [Paired summary](summary.txt)

The full v1 scorer gate passed 338 cases against libbzip2, including CRCs. The refresh identified scorer source `604dac3`; the scorer printed `6a8a366`. The receipt is preserved at `d5ae56a`. [Scorer report](../../../results/v1-delimiter-frames-gate/score/summary.txt)

From the research worktree root, with the archived inputs, corpus and release dependencies prepared:

```sh
d=research/astra-e/formulations/f2-rootstream/frames
export PYTHONDONTWRITEBYTECODE=1
python3 "$d/small.py" | diff - "$d/small.json"
python3 "$d/intervals.py" | diff - "$d/intervals.csv"
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --release --locked --manifest-path bench/moonshot/Cargo.toml candidate:: -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/frames-geometry.csv cargo test --release --locked --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
taskset -c 4 bash "$d/price.sh" /tmp/f2-frames-price "$PWD/bench/moonshot/target/release/deps"
python3 "$d/summarize.py"
```
