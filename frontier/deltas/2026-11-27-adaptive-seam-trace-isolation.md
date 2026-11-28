claim: none
by: chatgpt
outcome: done
evidence: PR #1144, e8b58598e3da7716880c101a641d85cdbc6b075e, src/parallel.rs:1515, src/trace.rs:477, Occupancy workflow 45973151284

**Question.** Does the adaptive scanner test measure the decoder that consumed its generated input when the library suite runs concurrently?

**Result.** On the PR head with Rust 1.90.0, the trace test passed alone. With four test threads, 54 parallel-module tests passed and the trace test failed at src/parallel.rs:1582 with `unclosed or empty spans=0`. The head's earlier green Occupancy workflow is compatible with this scheduling-dependent result.

**Reproduce** on the PR head, using the locked dependencies:

```sh
cargo test --locked --features trace --lib traced_window_maintenance_spans_balance_and_stay_bounded
cargo test --locked --features trace --lib parallel:: -- --skip the_policy_here --test-threads=4
```

The skip removes a machine-topology assertion unrelated to the scanner. The unfiltered module run also reproduced the empty-span failure.

**Cause.** The test enables tracing process-wide. Every concurrent decoder names its scanner thread `mibz2-scan`. `trace::take()` drains their shared completed traces, and the test checks the first scanner with that name. The selected scanner can belong to another test. Its spans and maximum capacity then describe that other input. Filtering out empty traces would still allow a different decoder with nonempty spans to satisfy the assertions.

**Repair.** Move the trace assertion into its own trace-feature integration test binary with one decoder, and run that target in the Occupancy workflow alongside the existing library tests. Keep the decoded-byte, span-balance and peak-capacity assertions. This review concerns the test's attribution. The registered i9 adoption thresholds and the predecessor's hardware verdict stand.
