# 2026-11-10: Borrowed tricks: what memchr, ripgrep, regex and uv suggest, tested or mapped

**Status: two build-level levers timed locally and found null; one API gap found; three ideas
mapped and not built. Indicative only (RULES C5). Nothing in the decoder changes.**

- **The question** (Louis): what do Andrew Gallant's libraries (memchr, aho-corasick, regex,
  ripgrep) and Astral's tools (uv, ruff) do for speed that this decoder does not?
- **Tested, null:** profile-guided optimisation and a whole-crate `x86-64-v3` build. Neither
  moves a decode beyond the noise here.
- **Not applicable:** a faster allocator. The parallel path already reuses every buffer.
- **Found:** the reader implements `Read` only. Every decoded byte is copied once more into the
  caller's buffer, and a line-reading consumer copies it twice.
- **Mapped, not built:** kernel choice by measured effect with back-off; a consumer API that
  gives up order; skipping the inversion of blocks nobody reads.

Labels: **measured** (local, one container), **read** (from the source or the record),
**model**.

## 1. The build levers (measured, null)

`ab child mibz2 T FILE`, whole-process wall, 9 runs an arm, alternating; the paired median of
arm over base. The process includes the harness's read and hash.

| | PGO, 1 thread | PGO, 4 | `x86-64-v3`, 1 thread | `x86-64-v3`, 4 |
|---|--:|--:|--:|--:|
| `wikidata` | 0.974 | 0.981 | 1.006 | 0.968 |
| `text` | 0.993 | 0.987 | 1.012 | 0.920 |
| `records` | 1.009 | 1.042 | 0.974 | 0.954 |

- **PGO:** instrumented build, trained on the three files at 1 and 4 threads, rebuilt with the
  profile. Within 3% either way, and the two files it "gains" on are inside their own spread.
- **`x86-64-v3` for the whole crate** (BMI2 shifts in the scanner, AVX2 everywhere): nothing at
  one thread; the 4-thread cells are the noisiest (`text` at 4 threads runs 0.08 s).
- **Why, read:** the profile is already fat LTO with one codegen unit, and the hot loops are
  written against their own listings (#204's register state, the two-level MTF). What these
  levers find in ordinary code has been done here by hand.
- **Where PGO would still apply:** the wheels, since maturin can build them with a profile. On
  this evidence it is not worth a release step.

## 2. The allocator (read: not applicable)

uv and ruff took about 10% from replacing the system allocator. This decoder allocates almost
nothing per block: `parallel.rs` keeps decoded-block and symbol buffers on free lists, and each
worker keeps its table and scratch for its life.

## 3. The copy the API forces (read, then model)

- **`BzReader` implements `Read` and not `BufRead`.** A decoded block sits whole in the
  reader's buffer, and `read` copies it out.
- **Counted, the ledger's ten-thread run:** that copy is 209.7 M bytes written, 17.7% of all
  data writes, and 52% of the run's cold reads: every output byte crosses from a worker's cache
  to the consumer's.
- **A line reader pays twice:** `BufReader::new(BzReader)` copies into its 8 KiB buffer, and
  `read_line` copies again. Wikidata's dumps are read by line.
- **The design:** `impl BufRead for BzReader`, with `fill_buf` lending the current block's
  bytes and `consume` advancing. `read_to_end` and `read_buf` appending from the block with no
  zero-fill first. In the Python binding, the same block handed to `PyBytes` once.
- **Worth, model:** 1.3 of 26.7 instructions a byte at one thread. At ten threads the copy is
  the consumer's whole job; it does not bind at 1.5 GB/s, and it is the next serial stage
  after the scanner once the decoder passes about 5 GB/s.

## 4. Mapped, not built

| the trick | where it lives | what it would be here |
|---|---|---|
| a prefilter that measures its own hit rate and turns itself off | memchr's `memmem` | choose the MTF kernel, and a table's k, a block by what the last blocks cost, not by a census; the emitter loses 5% on incompressible input and nothing switches it off |
| output order given up unless asked for | ripgrep's parallel printer | a block iterator for consumers that take records in any order: no ordered sink, no cross-core copy |
| read the metadata, skip the body | uv's range requests for wheel metadata | a block's decoded length without its inversion (`2026-11-10-share-nothing.md` §5), so `tar` listing and single-member extraction invert only the blocks that hold a header or the member |
| four vectors a branch, the tail as one overlapping load | memchr's search loops | already the shape of the short-run stores (#170); the RLE1 scan could take four vectors a branch, on 3 to 7% of a decode |
| candidates from a cheap fingerprint, then verified | memchr's packed pair, aho-corasick's Teddy | already the block finder's shape (`par_scan`: a bit-aligned magic search, then the chain) |
| byte classes by two nibble shuffles | Teddy | already the MTF list's shape (two `pshufb` on the first 32 bytes) |

## Reproduce

```text
rustup component add llvm-tools
RUSTFLAGS="-Cprofile-generate=DIR" cargo build --release --example ab --target-dir GEN
GEN/release/examples/ab child mibz2 1 FILE     # and 4; each training file
llvm-profdata merge -o merged.profdata DIR
RUSTFLAGS="-Cprofile-use=merged.profdata" cargo build --release --example ab --target-dir USE
RUSTFLAGS="-Ctarget-cpu=x86-64-v3" cargo build --release --example ab --target-dir V3
```
