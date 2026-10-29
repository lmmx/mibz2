# Development

## Layout

| path | contents |
| --- | --- |
| `src/` | the crate: the bit reader, block parsing, MTF, the inverse BWT, RLE1, the CRC, streams, the parallel decoder, `BzReader`, and `python.rs` (the extension module, feature `python`) |
| `tests/` | the Rust tests against libbzip2 (`decode.rs`, `corpus.rs`), their corpora in `tests/common/` |
| `examples/` | `ab` (the A/B harness), `bzcat`, `fuzz` (a mutation fuzz loop), `windows` (real dump windows checked) |
| `benches/` | `cargo bench` |
| `bench/` | the CI benchmark's scripts and its pinned Wikidata windows |
| `python/mibz2/` | the Python package, and `_mibz2.pyi`, the stub of the compiled module |
| `python/tests/` | the Python tests |
| `docs/site/` | this site; `docs/journal/` the development journals |

## Building and testing

The tools: a Rust toolchain, [uv](<https://docs.astral.sh/uv/>), [just](<https://just.systems/>), and for the lints taplo, whitespace-format and cargo-machete.

```sh
just py         # maturin develop, then pytest
just test-ci    # cargo test and the doc tests
just clippy     # clippy on every target and feature, warnings as errors
just pc         # formatting and lints (as pre-commit runs them)
cargo test --release                                   # against libbzip2
cargo run --release --example fuzz -- 600 1            # a mutation fuzz loop, libbzip2 the oracle
cargo run --release --example bzcat -- 4 < f.bz2 > f   # the parallel decoder as a command
```

On aarch64: `rustup target add aarch64-unknown-linux-gnu`, `apt install qemu-user gcc-aarch64-linux-gnu`, then `cargo test --release --target aarch64-unknown-linux-gnu --config .cargo/aarch64-qemu.toml`.

The segmented walks' unsafe code also runs under [Miri](<https://github.com/rust-lang/miri>), on blocks small enough to interpret: `cargo +nightly miri test --lib -- miri_` (about a minute; CI runs it on changes to `src/bwt.rs`). Those tests assert they reached the transitions the walks' handoffs guard (a buffer growing, a pass ending with a buffer full, a lane coming into the tail full, and so on), so a walk change that stops them reaching one fails rather than passing having checked less.

## Benchmarking

Performance is measured on CI (`.github/workflows/bench.yml`, the [benchmark](<https://lmmx.github.io/mibz2/pr-preview/pr-96/benchmarks/index.md>)): a pull request that touches the decoder gets a comment with its head against its base. Locally, the same harness:

```sh
cargo build --release --example ab
target/release/examples/ab gen corpora 4
target/release/examples/ab run runs.jsonl --arms mibz2,pbz2,lbzip2 --threads 1,4 --rounds 5 corpora/*.bz2
target/release/examples/ab summary runs.jsonl
```

Where the parallel decoder's time goes as its thread count grows (\#39) is measured by the occupancy instrument (`examples/occupancy.rs`, feature `trace`), run on the pinned Wikidata corpus by one recipe on any machine, as CI's Occupancy workflow does:

```sh
just bench-occupancy                             # 1, 2, 4, ... up to the CPUs allowed
just bench-occupancy --threads 1,2,4,8,12,16,20
```

It needs cargo, python3, git and lbzip2, and writes `occupancy-<UTC time>.json`: the machine as the process sees it, each thread count's throughput and the cost of tracing, and the decoder's own timeline (workers decoding or idle and why, the scanner, the reader, and per block its CPU time and run-queue wait). It is a diagnostic: the benchmark's paired A/B decides what is faster.

The benchmark has a version of its own (`BENCH` in `bench/report.py`), with its history in [`bench/CHANGELOG.md`](<https://github.com/lmmx/mibz2/blob/master/bench/CHANGELOG.md>): - **A pull request that changes the benchmark** (`bench/`, `examples/ab.rs`, `examples/phases.rs`, `.github/workflows/bench.yml`) either adds a changelog entry with its new version, or says in its description why no version is needed. - **A major version that redefines a family of figures** adds itself to that family in `MEANING` (`bench/report.py`), so the site's history splits that family there and joins the others across it. - **`python3 bench/test_report.py`** checks that `BENCH` and the newest entry agree, and that the history joins versions but never runner models, core counts, corpora or a corpus's different contents.

## Commits

Commit messages follow [Conventional Commits](<https://www.conventionalcommits.org/>) (`feat:`, `fix:`, `docs:`, `ci:` ...); the `conventional-pre-commit` hook checks them. `release-plz.toml` groups the changelog by these prefixes.
