# Clock acquisition independent of represented event widths

The range constructor represents a consecutive set of physical exception births by their affine origin labels, first LF image, width and common elapsed step count. It splits ordered sequences only at a stopped range or a default destination boundary. Whole virtual anchor returns use a stable range FIFO. A positive- or negative-excess sweep changes direction only at a balanced cut; one sloped region can cross zero at most once. Every finished source/target range has slope +1 and constant elapsed time. For K physical context states and P elementary birth intervals, the DAG, stable-hole closure and canonical publication create O(K+P) logical fragments independent of the number of physical exception rows. Sequence operations use expected logarithmic time for the randomized treap; a deterministic balanced join/split sequence gives the worst-case logarithmic version. Physical model acquisition and chronological output are separate stages. [Bounds and orientation](PROOF.md) · [Constructor](rangemap.py) · [Weighted hole sweep](sweep.py)

All 498,084 canonical triples on ten original blocks at three depths matched the independent run-membership packet implementation byte-for-byte. They also reproduced each marked time, byte and poststate in the independent temporal clock. Per-record hashes are stored per block/depth, without duplicating the large ABI output. [Thirty original-input receipts](real.csv) · [Native reference adapter](independent.cpp)

| k | Physical event ranks incl. primary | Birth intervals | Stopped intervals within contexts | Raw returns | Canonical returns | Reference rope nodes |
|---:|---:|---:|---:|---:|---:|---:|
| 4 | 1,122,586 | 261,834 | 135,267 | 283,463 | 269,448 | 305,714 |
| 8 | 409,433 | 121,004 | 47,909 | 138,976 | 127,360 | 159,347 |
| 16 | 229,014 | 94,261 | 37,868 | 108,814 | 101,276 | 123,648 |

The verifier covered all 17,576 small annotated monotone partial maps, including 2,081 fractional sign changes, and all 9,789 marked context models of the occurrence test. A stable-cover case with 9,895,604,650,047 physical rows and 5,497,558,138,916 exceptional ranks uses six birth pieces, sixteen rope nodes and eight returned bands. On a second instance, 1,000,000,000,007 holes on each side of a 4,097,333,333,362,014-row line generate just two different lap-count ranges. That number of physical holes is never expanded. [Adversarial source](check.py) · [Counts and witness](small.json)

A native C++ constructor uses retained ordinals on real birth/termination labels directly. It checks the same canonical triples against the independent SHA receipts and uses the frozen Rust scalar control separately for the favorable timing gate. [Frozen native plan](native/PLAN.md) · [Source](native/ranges.hpp) · [Real native checks](native/checked-real.json)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
r="$e/acquire/ranges"
python3 -B "$r/check.py" | diff - "$r/small.json"
g++ -O3 -std=c++20 "$r/independent.cpp" -o "$tmp/independent"
python3 -B "$r/real_ranges.py" "$original" "$models" "$e/contexts/inputs.csv" "$wires" "$tmp/independent"
```
