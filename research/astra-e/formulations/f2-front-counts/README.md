# Carry the front's histogram into delimiter discovery

The run front already counts rows by byte while decoding MTF and RLE2. A feature-gated read-only accessor passes that histogram to the dynamic delimiter constructor. Its 256 prefix sums initialize each symbol's LF cursor. During the existing run-record scan, write each maximal run's full LF start immediately and set its page-mask bit, including bit zero for a run exactly on a page boundary. A pass over pages alone counts starts; for a page with h starts somewhere before its left endpoint and boundary bit e, the run covering the page start has index h+e−1. Remove bit zero so the unchanged lookup returns its correct containing run. This eliminates the second pass over the maximal-run array.

Boundary tests check the entire run, directory and mask arrays against the previous constructor on 2,800 layouts through 899,931 rows. They include consecutive 127-row capped records, exact multiples of the 64-row page, offsets near the boundary and singleton runs. The 256-counter histogram is already produced on the charged front; the timed stage includes reading it. The primary source keeps the existing validation for record length and total size.

Across nine systematic admissions in the second 30-pass rotation, setup plus mark cost 0.685 [0.600,0.782] of the unmodified dynamic constructor. The full inverse ratio was 0.940 [0.875,1.011], and 0.968 [0.903,1.038] versus Lazy. An earlier 30-pass epoch gave 0.676 [0.603,0.757] for setup and 0.933 [0.833,1.045] for the full inverse. Its raw instrument labels were inherited from the preceding experiment: in first-epoch.csv the label 'refill' denotes this histogram arm and 'batch' denotes the unmodified dynamic constructor. The repeated harness uses their intended labels. Both checked the entire marked output against Segmented on every lap. The data distinguish a replicated phase gain from an inconclusive full-stage gain. [Raw repeat](phase.csv) · [Phase summary](summary.txt)

```sh
study=research/astra-e/formulations/f2-front-counts
export MOONSHOT_CANDIDATE="$PWD/$study/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::count_checks -- --nocapture
bash "$study/price.sh" /tmp/f2-counted "$PWD/bench/moonshot/target/release/deps"
bash research/astra-e/score.sh counts-confirm "$study/candidate.rs" --with-opend --workers 1,4 --rounds 9 --repeat 2 --settle 2 --no-proxy
```
