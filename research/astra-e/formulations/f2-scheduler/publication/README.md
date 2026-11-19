# Price the rank queries separately from publication

Trace the real eight-lane scheduler once. In order of retirement, save each node ID, final original interval endpoints, containing L-run index and the complete contraction spelling in a packed byte stream. This tape gives the exact answer that rank discovery would have computed for each physical node. The replay consumes these answers and runs the unchanged branch/selector construction, ancestor dictionary spelling, inverse merge, second-level factor and marked output. The separate ordinary arm executes all rank lookup waves and keeps the same initial preparation and finishing. Every warm and timed output matched Segmented.

The oracle measures a conditional construction price. Capturing the tape requires a real full traversal and the timed replay receives it for free. An implementable shortcut would need to produce its equivalents. The take-up cost includes reading the packed tape and copying each private spelling into the real edge buffer. The result does not impose a lower bound on alternative representations of the dictionary or selectors.

On 48 paired rotations at fixed CPU affinity over nine systematic admissions, summed per-block median first discovery was 7.149 ms physical versus 1.936 ms with prepaid intervals/labels. Complete inversion was 16.474 ms versus 11.189 ms. Oracle/ordinary discovery was 0.2692 [0.2574,0.2816], oracle/ordinary whole inverse 0.6792 [0.6536,0.7059], and oracle/Lazy whole inverse 0.6749 [0.6460,0.7051]. Pending rank work matters even when the real scheduler already fills virtually every modeled lane. A change solely to publication cannot recover the difference between the two arms. No worker or production throughput is inferred. [Raw observations](phase.csv) · [Paired statistics](summary.txt)

```sh
d=research/astra-e/formulations/f2-scheduler/publication
bash "$d/replay.sh" /tmp/f2-publication "$PWD/bench/moonshot/target/release/deps"
```
