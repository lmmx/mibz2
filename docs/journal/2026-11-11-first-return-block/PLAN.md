# Frozen first-return real-block probe

2026-11-11. Dispatch: [363/6808027528](https://github.com/lmmx/mibz2/issues/363#issuecomment-6808027528),
unchanged queue [145/6793754589](https://github.com/lmmx/mibz2/pull/145#issuecomment-6793754589).
Policy/master: `2635f04e9155dd31388b24fd3f7248f87b3ddcda`.
Independent assessment read at
`abe1cbe5f0823cbff484070d25c0e202dad77e02:state/reports/first-return-6793526035/report.md`.

## Question and prospective decision

Does delimiter 34 on the first stream-order block with n >= 65536 reduce
traversal work after charging construction, origin search, residual visits and
metadata? One block, one delimiter, one attempt. Hard process address-space
limit 256 MiB, hard CPU limit 600 seconds, parent wall timeout 600 seconds.
Preparation is separate. No timing is performance evidence.

Byte/origin/cycle mismatch stops this implementation. Exhaustion or no anchor
is a retained decline. E + origin_search_steps + residual_token_visits >= n
rejects traversal-count saving. A smaller sum AND explicitly accounted candidate
metadata <4n bytes is required to retain this configuration for a later compiled
cost experiment; otherwise revise. No retry, changed delimiter, changed block,
expanded census, workstation execution, production change or speed claim.

## Frozen inputs and preparation

`evidence/identities.json` records the complete identities. Corpus is bench-data
`a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a`, 11854933 bytes, SHA-256
`82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.
The native exporter selected block 1 (zero-based), bit 336, n=899931,
primary=264045, 56386 ordered capped records, output=900000 bytes.

`prepare.py` archives unchanged baseline
`57f51bbc0ca62ac3ee1cc148a1010842eea6222a` into private scratch, adds only a
cfg(test) module to lib.rs and copies `export.rs`. The module forbids unsafe
code. It uses the actual scanner/front, verifies baseline decode_block CRC,
extracts Segmented's RLE1 seam, checks independent MTF column versus every run,
and compares Cells separately. No tracked production file changes.

`oracle.py` separately builds row LF and its inverse, checks every cycle,
reconstructs the selected origin, and compares native output with Python's
libbzip2 prefix. This block has one LF cycle, length n. Its tables and decoded
bytes never enter the candidate; only frozen check digests and scalar cycle
length do. Offline storage is disclosed separately in identities.json.

## Candidate and accounting

The exact originals at `af3327d8aee9fffe26d2d6de76818b6ac69cc145` are vendored
unchanged. `probe.py` adapts phrase_induction's algorithm to the actual capped
run records, coalesces adjacent equal records, and builds run-level LF by byte
prefix counts. It keeps original source/destination coordinates, forward words,
separate origin search, queue splits, hash-cons grammar, coalescing, actual
expanded/reversed dictionary sorting, token-LF assertions, occurrence traversal,
word expansion, origin rotation, RLE1 expansion and CRC. No #369/#370 algorithm
or oracle-generated routes are substituted. All-cycle semantics remain those
of the original: selected cycles without an anchor decline; other cycles are
not silently used to reconstruct the selected one.

Counters distinguish run conversion, E and weighted rows, H created/live,
Q, frontier/completed records, boundary touches/splits, actual binary-search
and sort comparisons, dictionary bytes compared, word-DAG visits, phrase
lengths, copy calls/lengths/bytes, duplicate writes and RLE1/CRC/output work.
They are structural counts, not instruction counts or interchangeable costs.

Memory: cumulative logical record/field charges and growing-container charges
are conservative allocation accounts, not an exact allocator trace. Explicit
live object-graph samples include run records, pieces, completed pieces, grammar
nodes/hash-cons tables and expanded dictionaries. Deduplicated sys.getsizeof
samples are lower bounds on peak metadata, NOT exact peaks. Any sample >=4n
proves failure. Samples below 4n do not prove success and cannot retain the
configuration. This limitation is frozen before execution; the gate is unchanged.
The process limit/RSS also includes interpreter, counters, audit traversal,
2 MiB error-reporting reserve and output, so is not candidate metadata. Output
buffers are separately charged. Instrumentation itself can exhaust the limit;
that remains a decline, not a retry. The exact native baseline scratch peak is
unavailable and labelled as such.

`run_once.py` checks every source/input digest before creating a fresh attempt
directory, applies both hard limits and wall timeout, retains progress, stderr,
result and supervisor records, and never retries. The separate enforcement
smoke check uses no candidate data and passed (evidence/limits.json).

## Preparation commands already run

Run from repository root, with a verified corpus at `$CORPUS` and installed
Rust tools. This environment used Rust 1.99.1 and Python 3.12.14 (the evidence
environment record is authoritative).

```sh
python3 docs/journal/2026-11-11-first-return-block/prepare.py work/frozen-baseline
FIRST_RETURN_CORPUS="$CORPUS" FIRST_RETURN_EXPORT="$PWD/work/export" \
  cargo test --release --locked --manifest-path work/frozen-baseline/Cargo.toml \
  --lib first_return_export::export_first_qualifying_block -- --exact --nocapture
python3 docs/journal/2026-11-11-first-return-block/oracle.py work/export "$CORPUS"
python3 docs/journal/2026-11-11-first-return-block/run_once.py --verify-limits
```

Environment-specific Cargo/Rustup paths are recorded in evidence/environment.json.
Preparation logs are retained. Gzipped artifacts preserve all four native seams;
decompress them to reconstruct work/export without another corpus download.
After committing FREEZE.json and posting its commit to #145, the sole attempt:

```sh
python3 docs/journal/2026-11-11-first-return-block/run_once.py work/export work/attempt-1
```

The supervisor refuses an existing attempt directory. This is reproducible
source, not permission to execute a second attempt in this assignment.
