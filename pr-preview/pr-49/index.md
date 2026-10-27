# mibz2

A bzip2 decoder built for memory-level parallelism.

mibz2 decodes bzip2. It is block-parallel, as lbzip2 is. Within a block, its inverse BWT is walked as 32 interleaved chains, so a core keeps many of the walk's cache-missing loads in flight instead of one.

Think of a courier who learns each address only on arriving at the last one: the bike is fast, but most of the time goes on waiting to find out where to go next. That is the ordinary inverse BWT. mibz2 runs 32 such delivery chains through one dispatch centre, so while one waits for its next address, the others move. None of them travels faster; the waits overlap. [Why 32 couriers beat one](<https://lmmx.github.io/mibz2/pr-preview/pr-49/concepts/couriers/index.md>) explains the idea, where it stops paying, and what has been measured.

- Its output is byte for byte libbzip2's, and every block CRC and stream CRC is checked.
- It runs on any x86-64 or aarch64 CPU. Its SIMD kernels (AVX2, PCLMULQDQ) are chosen at run time, and each has a portable scalar twin.
- The library depends on std only.

```pycon
>>> import mibz2
>>> data = mibz2.decompress(open("file.bz2", "rb").read())
>>> data[:18]
b'line one\nline two\n'
>>> with mibz2.open("file.bz2") as f:
...     lines = f.readlines()
...     verified = f.raw.verified
>>> len(lines), verified
(2000, True)
```

Its name is for "multi-chain inversion of bzip2". It was written in [wikidata-pq](<https://github.com/lmmx/wikidata-pq>), to decode the Wikidata dump, and knows nothing of Wikidata ([history](<https://lmmx.github.io/mibz2/pr-preview/pr-49/background/history/index.md>)).

## Where to go

- [Installation](<https://lmmx.github.io/mibz2/pr-preview/pr-49/getting-started/installation/index.md>) and the [quickstart](<https://lmmx.github.io/mibz2/pr-preview/pr-49/getting-started/quickstart/index.md>), in Python and in Rust.
- [What is checked, and when](<https://lmmx.github.io/mibz2/pr-preview/pr-49/concepts/checks/index.md>): each block's CRC before its bytes are returned, each stream's at its end, and what an error reports.
- [Why 32 couriers beat one](<https://lmmx.github.io/mibz2/pr-preview/pr-49/concepts/couriers/index.md>): memory-level parallelism, the idea the decoder is built on.
- [How it works](<https://lmmx.github.io/mibz2/pr-preview/pr-49/concepts/how-it-works/index.md>): the scanner, the workers and the multi-chain walk.
- [Benchmarks](<https://lmmx.github.io/mibz2/pr-preview/pr-49/benchmarks/index.md>): master's figures over its commits, and mibz2 against the other bzip2 decoders.
- [Reference](<https://lmmx.github.io/mibz2/pr-preview/pr-49/reference/index.md>): the Python API, and the Rust API on docs.rs.

## Install

```sh
pip install mibz2   # Python 3.10+
cargo add mibz2     # Rust
```
