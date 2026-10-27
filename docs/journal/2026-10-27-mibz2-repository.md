# 2026-10-27: mibz2 as a repository of its own

## Current State

- The decoder's history moved here from lmmx/wikidata-pq by `git filter-repo`, its paths kept and renamed: `crates/mibz2/` and `crates/bunzip/` to the root, `reshape/src/bunzip/` to `src/`, `reshape/bunzip-tests/` to `bunzip-tests/` (removed later in that history), and the journals `docs/journal/2026-10-27-{bz2-decoder,mibz2-v2,bz2-bakeoff,bwt-inversion-literature,compute-frontier}*` as they were — each `#N` in the commit messages rewritten to `lmmx/wikidata-pq#N`
- The history enters as merges onto the repository's first commit (`2b9eb9b`, GitHub's README): wikidata-pq's `mibz2-v2` branch (the decoder's commits, from the native decoder in reshape to the A/B harness and the licence files), then wikidata-pq's `master` (the bake-off journal, #199 there), then its `research/bwt-inversion-literature` branch (the literature inventory, #198 there) — no history was rewritten after it was pushed
- The crate's code at the merges equals wikidata-pq's `mibz2-v2` at `9260303`; a `style:` commit then formats it with rustfmt's defaults, and clippy's `-D warnings` passes with `large_enum_variant` allowed on `block::Item` and `stream::Event` (src/block.rs, src/stream.rs)
- The Python package `mibz2` (python/mibz2/__init__.py) wraps the extension module `mibz2._mibz2` (src/python.rs, feature `python`, PyO3 0.29, abi3 for Python 3.10+): `decompress(data, threads=None)`, `open(source, threads=None, buffer_size=1 MiB)` returning an `io.BufferedReader` over a `BZ2Reader`, and `DecodeError` (a `ValueError`)
- `BZ2Reader.verified` turns true only when a read returns the end with every block's and stream's CRC matched (src/python.rs, `Reader::read` sets it on a 0-byte read) — the bytes read before then have each passed their block's CRC only
- A `BZ2Reader` source is a path (opened in Rust) or a Python binary file object (its `read` called with the GIL taken, from the decoder's feeder thread when `threads` > 1); an exception the source raises is raised again from the read (src/python.rs, `PyFile`)
- `threads=None` decodes on `std::thread::available_parallelism()` threads, `threads=1` on the calling thread (src/python.rs, `threads_or_all`)
- The Python tests (python/tests/test_decode.py, and the doctest in python/mibz2/__init__.py) check output against the standard library's `bz2` at 1 and 4 threads, multi-stream input, paths and file objects, lines, a damaged block (the bytes returned are a prefix of the true output), truncation, and a source's exception
- The release infrastructure follows lmmx/pq-glue's: `.github/workflows/CI.yml` (abi3 wheels for Linux x86-64 and aarch64, Windows x64, macOS x86-64 and arm64, an sdist; PyPI by Trusted Publishing in the `pypi` environment; the crate by release-plz in the `crates-io` environment), `deploy.yml` (`just release`, then CI dispatched on the `py-*` tag), `test.yml` (fmt, clippy, cargo test, pytest on each Python from 3.10, ty), the Justfile and `.just/`, `release-plz.toml`, `taplo.toml`, `.pre-commit-config.yaml`
- The aarch64-under-qemu cargo config moved from `.cargo/config.toml` to `.cargo/aarch64-qemu.toml`, used with `--config` — a default config would set the linker for maturin's aarch64 cross builds too

## Missing

- No documentation site: no `zensical.toml`, `docs/site/` or `site.yml` (pq-glue's `site.yml` and `docs/scripts/check_site.py` were left out of the first release)
- No armv7 wheel: the 32-bit build has not been tested (CI.yml's Linux matrix has x86-64 and aarch64 only)
- The aarch64 wheel's tests run on no aarch64 machine: aarch64 correctness was checked under qemu-user in wikidata-pq (docs/journal/2026-10-27-bz2-decoder.md)
- No benchmark workflow: `cargo bench` and `examples/ab.rs` run by hand only
