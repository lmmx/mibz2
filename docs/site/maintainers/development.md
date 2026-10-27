---
description: "The repository's layout, how to build, test and benchmark mibz2, and the commit conventions."
---

# Development

## Layout

| path | contents |
|---|---|
| `src/` | the crate: the bit reader, block parsing, MTF, the inverse BWT, RLE1, the CRC, streams, the parallel decoder, `BzReader`, and `python.rs` (the extension module, feature `python`) |
| `tests/` | the Rust tests against libbzip2 (`decode.rs`, `corpus.rs`), their corpora in `tests/common/` |
| `examples/` | `ab` (the A/B harness), `bzcat`, `fuzz` (a mutation fuzz loop), `windows` (real dump windows checked) |
| `benches/` | `cargo bench` |
| `bench/` | the CI benchmark's scripts and its pinned Wikidata windows |
| `python/mibz2/` | the Python package, and `_mibz2.pyi`, the stub of the compiled module |
| `python/tests/` | the Python tests |
| `docs/site/` | this site; `docs/journal/` the development journals |

## Building and testing

The tools: a Rust toolchain, [uv](https://docs.astral.sh/uv/), [just](https://just.systems/),
and for the lints taplo, whitespace-format and cargo-machete.

```sh
just py         # maturin develop, then pytest
just test-ci    # cargo test and the doc tests
just clippy     # clippy on every target and feature, warnings as errors
just pc         # formatting and lints (as pre-commit runs them)
cargo test --release                                   # against libbzip2
cargo run --release --example fuzz -- 600 1            # a mutation fuzz loop, libbzip2 the oracle
cargo run --release --example bzcat -- 4 < f.bz2 > f   # the parallel decoder as a command
```

On aarch64: `rustup target add aarch64-unknown-linux-gnu`, `apt install qemu-user
gcc-aarch64-linux-gnu`, then `cargo test --release --target aarch64-unknown-linux-gnu
--config .cargo/aarch64-qemu.toml`.

The segmented walks' unsafe code also runs under [Miri](https://github.com/rust-lang/miri),
on blocks small enough to interpret: `cargo +nightly miri test --lib -- miri_` (about a
minute; CI runs it on changes to `src/bwt.rs`). Those tests assert they reached the
transitions the walks' handoffs guard (a buffer growing, a pass ending with a buffer full,
a lane coming into the tail full, and so on), so a walk change that stops them reaching one
fails rather than passing having checked less.

## Benchmarking

Performance is measured on CI (`.github/workflows/bench.yml`, the
[benchmark](../benchmarks/index.md)): a pull request that touches the decoder gets a
comment with its head against its base. Locally, the same harness:

```sh
cargo build --release --example ab
target/release/examples/ab gen corpora 4
target/release/examples/ab run runs.jsonl --arms mibz2,pbz2,lbzip2 --threads 1,4 --rounds 5 corpora/*.bz2
target/release/examples/ab summary runs.jsonl
```

## Commits

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `docs:`, `ci:` ...); the `conventional-pre-commit` hook checks them.
`release-plz.toml` groups the changelog by these prefixes.
