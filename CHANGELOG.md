# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### <!-- 7 -->Build System and CI

- Tag Release, a commit tagged at the version it already holds and CI started on the tag ([#2](https://github.com/lmmx/mibz2/pull/2))
- Release Notes, a version's GitHub release made or updated from this changelog, on the commit the version was released from

## [0.1.1](https://github.com/lmmx/mibz2/compare/v0.1.0...v0.1.1) - 2026-10-27

### <!-- 9 -->Other

- the same code as 0.1.0, released at the next version by Deploy Release: the crate and the Python package (wheels for the same platforms) at one version, from one commit

## [0.1.0](https://github.com/lmmx/mibz2/releases/tag/v0.1.0) - 2026-10-27

### <!-- 1 -->Features

- a bzip2 decoder, block-parallel, its inverse BWT walked as 12 interleaved chains within each block, SIMD kernels (AVX2, PCLMULQDQ) chosen at run time with a portable twin each; byte for byte libbzip2's output, every block's and every stream's CRC checked, randomised blocks (bzip2 0.9.0 and earlier) refused
- `decode`, `decode_parallel`, `BzReader` (on the calling thread, or block-parallel), `StreamDecoder` (compressed chunks written from one thread, decoded bytes read from another), `scan` (block boundaries without decoding) and `decode_blocks` (any range of whole blocks), errors with the block and bit where decoding failed
- the Python package: `decompress`, `open` (a buffered binary file over a path or a binary file object) and `BZ2Reader.verified`, true only once a read reaches the end with every CRC matched; `DecodeError`; abi3 wheels for Python 3.10+ on Linux x86-64 and aarch64, macOS x86-64 and arm64, and Windows x64 ([#1](https://github.com/lmmx/mibz2/pull/1))

### <!-- 4 -->Documentation

- the decoder's development journals from lmmx/wikidata-pq (its design and CI figures, the v2 plan, the bake-off against pbz2, lbzip2, fbz, the `lbzip2` crate and indexed_bzip2, the BWT inversion literature inventory, the compute frontier) ([#1](https://github.com/lmmx/mibz2/pull/1))

### <!-- 7 -->Build System and CI

- the decoder's history moved from lmmx/wikidata-pq, and the packaging and release workflows of lmmx/pq-glue: the crate and the Python package at one version, published to crates.io and PyPI by Trusted Publishing ([#1](https://github.com/lmmx/mibz2/pull/1))
