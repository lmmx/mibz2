# Cells stage-to-decoder diagnostic

Separate, safe tooling against **frozen PR #365** revision
`57f51bbc0ca62ac3ee1cc148a1010842eea6222a`. It changes no decoder source, admission,
default, existing benchmark arm or review evidence. `Cargo.lock` fixes the dependency
graph. The diagnostic crate forbids unsafe code; the dependency remains subject to B5.

Integration question: is the next useful change in admission/refusal, the production
port, or the surrounding decoder? This tool is diagnostic, not an adoption benchmark.
The historical i9 research census and whole-decoder CI differ in hardware, cache
conditioning, code, included stages and selected blocks. Do not subtract their times.

## Untimed, reproducible first step

```sh
cargo test --locked --manifest-path bench/cells-gap/Cargo.toml
cargo build --release --locked --manifest-path bench/cells-gap/Cargo.toml
git fetch origin bench-data
mkdir -p target/cells-gap-input
git archive a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a windows/wikidata-20261019 |
  tar -x -C target/cells-gap-input
python3 bench/cells-gap/corpus.py bench/wikidata.json \
  target/cells-gap-input/windows/wikidata-20261019 target/cells-gap-input/wikidata.bz2
bench/cells-gap/target/release/mibz2-cells-gap census \
  target/cells-gap-input/wikidata.bz2 > target/cells-gap-input/census.jsonl
python3 bench/cells-gap/oracle.py target/cells-gap-input/census.jsonl \
  target/cells-gap-input/wikidata.bz2
```

`corpus.py` only accepts the exact hash-pinned windows, marker counts, stream sizes,
decoded sizes and final corpus SHA. Python's libbzip2 validates every reconstructed
stream. `oracle.py` independently compares every census block's output digest with
Python/libbzip2's stream and requires that all output is consumed. This narrow
reconstruction is not a replacement for the general windows tool.

`census` calls the real scanner and `decode_block` with both types. A safe trait wrapper
captures the actual inversion's row count and Cells' before/after counters. Every block
must have matching location, length, output SHA-256 and a valid CRC. No timer is read;
all timing fields are null. Thus short blocks and records-path refusals stay visible.
Tests compare compressed fixtures with their original bytes and require both admitted
and column-route reach. The route counters belong to a reused kernel, as in the reader.

`analyse.py census.jsonl census-1.jsonl census-2.jsonl census-3.jsonl` joins the original
i9 archive by ordered blocks above its 65,536-row minimum, asserting lengths and every
predicted admission decision. It reports historical counterfactuals, never new timing.

## Bounded next measurement (prepare only; no workstation launch)

On one machine, pin one CPU, use one toolchain and the same hashed corpus. Save all
commands, versions, environment facts, source/lock hashes and raw JSONL to a fresh
directory. Three interleaved file passes suffice for the diagnostic; do not repeat to
seek a favourable result. On an i9/workstation, the Coordinator must first freeze and
review the recipe under C5; this README does not queue, signal or authorise that run.

1. **Actual decoder context:** `mibz2-cells-gap trace FILE 3`. Complete passes alternate
   Cells/Segmented order. Each block records scanner, real `decode_block`, and inner
   trait inversion spans. Decode minus inversion is the aggregate front/RLE1/CRC cost;
   this is not a per-stage decomposition of that remainder. Hashing and JSON output
   are outside those spans. The wrappers/clocks can alter code generation and cache
   state, so these spans diagnose costs rather than replace the ordinary paired bench.
2. **Warm-port control:** `mibz2-cells-gap warm FILE 5`. Same public records and kernel
   entry points, ≥65,536 rows, records front outside clocks; keep each kernel's fastest
   of five repeats on each block, checking exact inversion bytes after each pair.
   This intentionally mimics the archived research census's warm-minimum conditioning.
3. Build unchanged `examples/rauzy.rs` at the frozen revision with the same release
   toolchain; run `rauzy FILE 1 65536 5` on the same CPU. Alternate which executable
   runs first across three rounds of steps 2/3. Compare aggregate warm inversion cost
   by admission group; the research report also supplies its order/induction/write breakdown. Different
   executables still permit code-generation effects; a gap warrants a focused control,
   not automatic attribution to one source difference.

Decision: if the warm port loses the research advantage, prioritise a bounded port
cost investigation (especially the changed copy primitives); if warm advantage survives
but decode-context inversion does not, investigate conditioning. If actual inversion
wins but the total does not, price the measured remainder and then add a matched reader
adapter control. Do not pre-emptively tune all three. Report uncertainty and park if no
material recoverable cost is isolated. No default flip or edit to pending #365 follows
from this diagnostic.
