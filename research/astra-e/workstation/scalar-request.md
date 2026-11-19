# Scalar cap shortcut on the i9-10900K

The scalar worker at `d0a42980e896211906234c7679b38760ae44a46e` skips a neighboring label comparison whenever the current maximal run begins with a cap shorter than 127. It reconstructs the same marked pieces, uses the same edge budget and adds no scratch. Its frozen predecessor is the independently measured i9 edge worker at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. That predecessor's binary also supplies the LazyCells and opend reference arms. The two decoder binaries are separately compiled with an identical refreshed scorer projection and pinned Rust 1.99.1. [Kernel and count proofs](../front-fusion/cap-batch/README.md) · [Historical i9 edge receipt](edge-i9.md)

Independent Xeon CI gave scalar/edge wall ratios of 1.0119 [1.0023,1.0216] at W1, 1.0056 [1.0015,1.0097] at W2 and 0.9956 [0.9915,0.9998] at W4. W4 median peak RSS was 130,682 KiB scalar versus 130,950 KiB edge; W4 CPU ratio was 0.9919 [0.9891,0.9946]. The target comparison must distinguish a small possible incremental gain from the independently demonstrated low-concurrency regressions. [Full five-arm CI and archive](../ci/evidence/scalar-cap-44450850135.md)

## Frozen primary and production rules

Use 48 rounds at W7, W8, W9 and W10, four arms, 4W slots, two-second settles and four copies of the SHA-verified Wikidata input, totaling 768 observations. For each round take the arithmetic mean of the four paired scalar/edge log wall ratios; apply a two-sided 95% Student interval across these 48 round means. The incremental condition requires the pooled upper ratio below 1, *each* individual worker-count upper wall ratio at most 1.02, and the original no-growth peak checks against LazyCells at all four worker counts. The latter require the paired RSS upper endpoint at most one, median peak at most the reference median, and maximum observed peak at most the reference maximum. Per-worker wall intervals and peaks are retained separately. A favorable pooled result makes an equal-weight incremental statement only.

The 1.25× stretch condition is tested separately against 0.5.6 LazyCells at every W. All four individual upper wall endpoints must be at most 0.80 and each worker count must meet all three peak conditions. The dry synthetic rotor deliberately gives W9 a 5% regression while the pooled upper bound is still below one; it also tests a one-W peak violation. [Executable decision](scalar.py) · [Synthetic outcome](scalar-protocol-check.txt)

The historical 16×4 edge/predecessor trial had a standard deviation of 0.012709 in these equal-weight, same-round log means. Under iid normal round means and the conventional two-sided interval, 48 rounds give prospective power of 65.3% at a 0.44% effect, 99.0% at 0.8%, and over 99.99% at 2%. The corresponding values at 32 rounds are 47.7%, 93.4% and over 99.99%. Historical standard-deviation uncertainty alone gives a 33–89% power range at a 0.44% effect for 48 rounds. A four-round block resample also suggests that residual temporal dependence could widen the interval. These are power estimates for the pooled wall endpoint alone. The W-specific and memory conditions can further reduce the chance of an incremental pass. [Blind calculations and original round vectors](scalar-power/README.md)

Independent registration is `bench/astra-e-scalar-cap` at `23ee31abc85851d8809cfccc2273f452ca99f204`. At this exact checkout, use the following commands for the untimed target rehearsal and the Coordinator's measurement submission.

```sh
just bench-astra-e-scalar-cap-dry
just submit bench-astra-e-scalar-cap
```

On the unpinned target all twenty logical CPUs must be available. The dry invokes every explicit arm at all four requested worker counts. The target also runs the availability-limited quick gate under each W-specific CPU mask. No dry wall time enters the decision. Each measured sample returns 838,800,784 bytes. The bundle includes both compiler hashes, refreshed scorer identity, exact roster, both full oracle gates, control gates, CPU topology, raw sample rows and replayable decisions. The original edge recipe remains independently frozen. [Registered run script](scalar-run.sh) · [Bundle verifier](scalar-verify.py)
