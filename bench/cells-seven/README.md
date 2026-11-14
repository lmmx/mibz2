# Seven-family untimed screen

The frozen [PLAN](PLAN.md) answers writer eligibility, correctness and memory
apparatus questions before selecting a production candidate. It does not measure
performance. Existing live-path scripts and the completed 120+9 stay unchanged.

With Rust 1.99.1 and a C compiler on Linux:

```sh
python3 bench/cells-seven/prepare.py prepare
python3 bench/cells-seven/prepare.py build
python3 bench/cells-seven/test_screen.py
cargo +1.99.1 test --locked --manifest-path bench/cells-seven/target/reader/Cargo.toml --features live-trace,cells-reader
python3 bench/cells-seven/stage.py corpora/wikidata.bz2
python3 bench/cells-seven/screen.py
```

The source and attempt directories must not already exist. The workflow also runs
the original F01 and corruption/concatenated-stream checks. The existing canonical
Wikidata reconstruction, when needed, validates each archived stream as part of
input construction; it is separate from the 21 whole-corpus comparisons. Do not
substitute another compressed input, text source tree or failed attempt.

`target/source.json` records original live preparation plus the additive grammar
inspection patch. No original decoder operation is removed. `text-manifest.json`
records the ordered historical file inputs. `build.json` pins binaries/compiler.
The screen retains native PID/wait4 resources, child status, JSON counters, full
outputs (losslessly compressed after equality) and partial failures. Its only
clock is a deadline; no execution here is a timing sample.

The instrumented readers report memory including tracing and the diagnostic depth
table. This is not the memory footprint of a production candidate. Eligibility
scratch uses one `usize` per stored rule, bounded by the existing induction budget;
its bytes and traversal visits are recorded. The all-small-spine premise requires
`long_seeds == 0`, not merely absence of long-copy counters. The `64*r < n`
hypothesis uses pre-induction capped records; eligibility is measured later and
cannot be used to call that hypothesis an early classifier proof.

## Safety and failure paths

The additional Rust grammar inspector uses bounds-checked slices and validates
strictly earlier child offsets; it does not dereference pointers or alter decoder
state. Iteration is bounded by stored rules, with no recursion. The existing unsafe
writer, original-row phase, slack, repetition, fallback and F01 recopy remain exact.
This adds no production unsafe code and does not replace independent B5 review.

The C launcher is an isolated diagnostic executable, never linked into mibz2. It
execs exactly the supplied child, keeps the result file out of that exec, reaps the
exact PID with `wait4`, and uses a signal handler only to set `sig_atomic_t`.
The normal loop performs process-group termination on timeout, then records the
reaped status and resources. Tests cover error exit, missing executable, signal and
timeout. Rust reports native memory on successful and ordinary-error returns;
abrupt exits can lack that record and therefore fail the screen while retaining
launcher evidence. Parent Python memory is never attributed to the decoder.

There are no new release/performance assertions in this preparation. Counter
changes cannot establish an end-to-end gain. Candidate qualification and the
proposed seven-family timing inventory require the separate Coordinator decision.
