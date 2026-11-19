# Decoder library validation

With Rust 1.99.1 and affinity restricted to the container's four-CPU quota, the normal release library and the release library with the research-only moonshot feature each passed 141 tests with one ignored. The earlier unpinned debug run saw five affinity CPUs and four CPU-equivalents of quota. Its automatic parallelism expectation failed and all other executed tests passed. The same policy test passed with four-CPU affinity. Formatting and Clippy with all targets and features passed. The full scorer's separate worker gates audit byte output and errors against libbzip2. [Default release](default-release.log) · [Research feature release](experimental-feature-release.log) · [Affinity repeat](debug-policy-affinity.log) · [Clippy](clippy.log)

With every feature enabled, the release library had 148 passes, four failures and one ignored. All four failures are assertions in the existing experimental Cells owned-writer tests. A fresh upstream checkout at `6fb7775` reproduced the same four failures. The normal and moonshot-only test counts above are independent runs. This workspace linked its installed Python runtime for the optional Python extension when testing all features. [All-features diagnostics](all-features-release.log) · [Upstream control](upstream-owned-writer.log)

The affinity examples use CPUs 0–3 on this machine.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
taskset -c 0-3 cargo test --locked --release --lib
taskset -c 0-3 cargo test --locked --release --lib --features moonshot
taskset -c 0-3 cargo test --locked --release --lib --all-features
taskset -c 0-3 cargo test --locked --release --lib --all-features owned_writer
```
