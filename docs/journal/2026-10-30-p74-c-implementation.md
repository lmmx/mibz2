# P74-C implementation: correctness passes, generated-code gate blocked

This research branch is based on master `ae942bdb135569224473116ffaa706f1c9e389ed`.
It implements the frozen Option C policy, but **is not ready for the frozen stage
benchmark**: extra low-rank instructions remain after the one allowed codegen fix.
No timings, performance counters, benchmark workflows or i9 runs were used. Tests
are correctness evidence only. Production dispatch and the shipped kernels are unchanged.

## Specification and provenance

- [Frozen worst-case note, including §9 and §12a](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-p74-option-c-worst-case.md).
- [Gate 1 results](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-p74-gate1-result.md).
- P74 kernels and their nine integration tests imported from
  `research/p74-gate1`, `e57400e03efe6f9495954417b2bd3fb16665a5b0`.
  Its negative performance verdict is unchanged. The import retains Branch, Free
  and the historical selector; Option C never calls that selector or performs a census.
- The journal's §12 says symbols consumed "before this token", whereas §9 defines
  the boundary after the trigger is consumed. This contradiction was reported before
  implementation. The task's explicit definition and §9's precise boundary are used:
  **S_consumed includes the triggering token**, as well as every RUNA/RUNB digit.
  The token-128 equality regression test distinguishes the two readings.

The constants remain `S < 16384 || 2*S >= N_max` for the standalone bypass, and
`F + 1 > (S_consumed >> 5) + 123` at rank >=32 for the permanent switch. Multiplication
in the initial dispatch is widened to u128 to preserve the inequality without overflow.
There is no alphabet shortcut, threshold adjustment, reverse switch or fast-path counter.

## Architecture and interfaces

`src/mtf/p74_c.rs` exposes the same signatures for all three arms:

```rust,ignore
use mibz2::mtf::p74_c::{self, Arm};
let k = mibz2::Kernels::detect();
p74_c::mtf(&symbols, &mut ll, &mut counts, k, Arm::T2)?;
p74_c::mtf_checkpoints(&symbols, &mut ll, &mut counts, &mut checkpoints, k, Arm::T2)?;
```

| Arm | Entry |
|---|---|
| T0 | Untouched standalone shipped kernel |
| T1 | Combined kernel in shipped mode before any symbol, without the initial bypass |
| T2 | Frozen bypass; otherwise P74 Free and the online guard |

On a scalar `Kernels` configuration all arms use the shipped scalar implementation.
The research AVX2 entry is non-inlined and has a runtime T1/T2 mode argument: both
arms enter the same combined machine code, once for plain and once for checkpoints.
`examples/p74_c.rs` links all six entries and Gate 1 Free into one binary. It checks
outputs, counts, checkpoint records, block CRCs and the whole decoded output. It has
no timer; a later authorized harness can call these identical signatures and charge
dispatch inside the measurement.

The combined functions use one stack frame and one 64-byte-aligned 320-byte store:
prefix slack `[0,32)`, head slot `[32,64)`, tail `[64,288)`, trailing padding `[288,320)`.
P74 updates its head in a vector local and the tail in place. At handoff a single YMM
store materializes the head; all 256 entries then occupy `[32,288)`. No tail copy.

The two loops are separate expansions of a shared research-only control-flow macro.
`CP=0` compiles away checkpoint logic; `CP=16384` uses the shipped interval handling.
The P74 list operations reuse Gate 1's macros, with an optional fallback block that
is absent from the original arms. Only module declarations were appended to
`src/mtf.rs`; no standalone implementation or production caller was refactored.

The iterator already consumed the trigger when the guard runs. The preceding run is
fully emitted, `run=0`, `weight=1`, and capacity/limit checks have passed. If switching,
the head is stored, the shipped byte step applies the pending token exactly once,
and the suffix resumes the same iterator, n, counts, next and checkpoint vector.
This also handles the byte step in the checkpoint outer loop after a split run,
and switches immediately within an interval. The suffix has no monitor or mode test.
P74 reserves `max_len+32` for its existing short-run vector stores; only initialized
bytes through n are published, including on errors.

Tests alone can override the guard and record the handoff list/state. Those hooks and
the four-byte checkpoint interval do not exist in the measurement build.

## Correctness checks

Environment: Codex Cloud, x86_64 AMD EPYC 9V74, AVX2 detected; rustc 1.90.0
`1159e78c4`, LLVM 20.1.8. Cargo's existing test profile uses opt-level 3. Commands
were run from the repository with the installed 1.90.0 toolchain on PATH.

| Command | Result |
|---|---|
| `cargo check --offline` | Pass |
| `cargo test --offline --lib mtf::p74_c -- --nocapture` | 7 pass; rerun after the monitor change and final policy assertions: 7 pass |
| `cargo test --offline --test mtf_p74_c` | 5 pass |
| `cargo test --offline` | 38 unit tests pass, existing `parallel::plan_tests::the_policy_here` fails; integration tests not reached |
| `taskset -c 0,1 cargo test --offline` | 74 tests and 1 doctest pass: 39 unit, 3 cooperative, 2 corpus, 13 decode, 3 checkpoints, 9 Gate 1, 5 Option C |
| `cargo test --offline --profile fuzz --lib mtf::p74_c` | 7 pass with overflow checks and debug assertions |
| `cargo test --offline --profile fuzz --test mtf_p74_c` | 5 pass with overflow checks and debug assertions |
| `cargo test --offline --example ab` | 3 harness unit tests pass; no benchmark execution |
| `cargo fmt --all --check` | Pass |
| `cargo clippy --offline --all-targets --all-features -- -D warnings` | Pass after removing an unnecessary mutable borrow in the correctness driver |
| `cargo metadata --offline --locked --format-version 1 --manifest-path bench/alternatives/Cargo.toml` | Pass |
| `cargo build --offline --release --example p74_c` | Pass |
| `target/release/examples/p74_c /workspace/work/p74/driver-fixture.bz2` | All arms agree, 2 concatenated streams/blocks, 320000 decoded bytes, AVX2=true |

The unrestricted test failure is environmental and reproduces on an unmodified
detached checkout of master with `cargo test --offline --lib
parallel::plan_tests::the_policy_here`: sysfs/affinity exposes 3 physical cores while
the cgroup quota is 2 CPUs (`cpu.max = 200000 100000`), so its assertion reports
"3 cores of 2 CPUs". Restricting affinity to CPUs 0 and 1 makes both views agree.
No test was skipped or production topology code changed to obtain the full-suite pass.

Focused coverage:

- Every legal fallback boundary for every symbol string of length 0..3 at alphabets
  1..6 and 31..34, with run digits included, an appended EOB, and plain/four-byte
  checkpoint modes. This is exhaustive over that stated domain, **not** all strings
  through length 8 in the journal's larger proposed domain.
- Every one of 256 eight-digit RUNA/RUNB patterns interleaved with each tail chunk
  boundary rank; every legal handoff. Every alphabet size 1..256, every rank twice
  in opposite order, all handoffs. Every recorded handoff checks all 256 list entries
  against an independent `Vec::remove/insert` MTF oracle, and checks n/run/weight/next.
- Five 100000-token distributions, including low/high regime changes, sparse high
  ranks, random ranks, and runs. First/last/64 seeded sampled fallback positions and
  positions around interval-sized symbol offsets. The live switch is also compared
  with a separate integer formulation across the whole stream, including no-switch
  cases and F carried across intervals.
- Runs of 1/31/32/33 bytes and interval-sized/multi-interval runs immediately before
  and after a forced switch; prefix output at checkpoint-1/checkpoint/checkpoint+1.
  These test actual output checkpoint crossings and mid-interval switching.
- Dirty reusable buffers; exact output length, counts and every checkpoint record;
  exact partial outputs/counts/checkpoint arrays on AVX2 errors. Empty/truncated symbol
  streams, EOB termination, `RunTooLong`, `BlockTooLong`, level-1/level-9 limits, and
  initial size/ratio boundaries including equality.
- Existing generated corpora and sample shapes compressed by libbzip2. Six adversarial
  symbol distributions written as real `.bz2` streams, then decoded by shipped mibz2,
  all research arms, and the existing independent inverse-BWT/RLE1 and libbzip2 oracles.
  The driver's separate fixture is Python `bz2` compression of 200000 seeded random
  bytes followed by a concatenated stream of `b"abc" * 40000`.

No external Wikidata dump was available or downloaded. These generated correctness
fixtures are not out-of-sample selection validation and are not the complete §8
performance suite. Those inputs and independent seeds/source selection remain part
of a separately registered experiment, if the codegen gate is ever reopened.

## Generated code: one fix, residual defect, stop

Build uses the existing release profile: fat LTO, one codegen unit, baseline target
plus per-function AVX2, no `target-cpu=native`. Inspection commands:

```sh
cargo build --offline --release --example p74_c
objdump -d -C -Mintel --no-show-raw-insn target/release/examples/p74_c
nm -S -C target/release/examples/p74_c
sha256sum target/release/examples/p74_c
```

Inspected final binary SHA-256:
`18feb1e6365424fe9aed2843b758e553b319ecb07c5b6129dfd883807078bff7`.
The [path listing](2026-10-30-p74-c-implementation/low-paths.txt) records the full
ordinary low-rank cycles and their addresses in this binary. Counts exclude alignment
NOPs, include unconditional jumps, and are **instructions, not uops or cycles**.
They describe rank 1..31, no pending run, no limit crossing, with another symbol left.

| Loop | Instructions | Conditional branches |
|---|---:|---:|
| Gate 1 Free, plain | 37 | 6 |
| T2 P74 prefix, plain | 38 | 6 |
| Gate 1 Free, checkpoints | 38 | 6 |
| T2 P74 prefix, checkpoints | 41 | 6 |
| T0 shipped, plain | 40 | 6 |
| T1/T2 shipped suffix, plain | 43 | 6 |
| T0 shipped, checkpoints | 42 | 6 |
| T1/T2 shipped suffix, checkpoints | 41 | 6 |

The initial build kept guard-only state live across low tokens. Plain reconstructed
both SHIFT and SWAPPED addresses per low token, and stored EOB on that path;
checkpoint reconstructed all three mask addresses. The **one fix** made the origin
pointer and F local volatile slots, accessed only at high-rank events. This prevents
their hoisting into hot-loop registers; it does not change the policy. No second
codegen restructuring was attempted after inspecting the result.

Concrete final observations:

- Head is `ymm2` across ordinary low tokens in both T2 variants: `vpermq`, two
  `vpshufb`, mask load, `vpblendvb`. **No head spill/reload, call or new conditional
  branch on that path.** Ordinary high-rank updates also keep the head in registers;
  the tail uses the existing blended chunk loop, without a tail `memmove` call.
- Residual plain defect: `lea SWAPPED` at `0x2a901`; Gate 1 keeps that address in a
  register. Net +1 instruction per ordinary low token. Checkpoint has `lea SWAPPED`
  at `0x2b47c` plus changed scalar reloads, net +3 instructions. Thus source-level
  placement in the fallback branch does **not** give zero hot-path overhead.
- Plain monitor at `0x2a92e..0x2a962`: origin subtraction, shift from bytes to u16
  symbols, volatile F load/increment, `shr 5`, `add 0x7b`, compare/branch, store F
  only if staying. Checkpoint's inner-loop monitor starts at `0x2b4a2`; its outer
  byte step has its own expansion. F's memory dependency is between high events
  only. No separately maintained S counter exists.
- Checkpoint P74's n still has a loop-carried stack load/increment/store
  (`[rsp+0x38]` versus Gate 1's `[rsp+0x8]`). Scalar reloads now include the output
  pointer, counts and return pointer; the original reloads s2u instead. This is a
  register-allocation change, not an unchanged dependency graph.
- Long runs still spill/reload the YMM head around `memset`, e.g. plain
  `0x2a891/0x2a8a0`; checkpoint record copying/growth also spills it. Gate 1 already
  does this. "Register-resident" is therefore true of the ordinary token path,
  not across external calls. Checkpoint `memcpy/memmove` calls copy 1024-byte count
  records; they are not P74 tail shifts. Error cleanup has generated drop/free paths.
- The handoff is one head store (`0x2aaa6` plain), followed by the shipped pending
  token's `memmove`. It performs no 224-byte tail copy.
- The shipped suffix preserves its AVX2 blend, overlapping shift load, rank>=32
  `memmove`, run `memset`, and checkpoint compiler fence. It is **not machine-code
  identical to T0**. Plain stores/reloads more scalar state (43 versus 40 instructions).
  Checkpoint n is stack-carried where standalone shipped keeps n in a register,
  despite the suffix's lower static instruction count. T1 remains necessary.
- LLVM lays out the shipped loop *before* the P74 loop in each combined function;
  their ordinary loop bodies are distinct, with shared exit/cleanup blocks. The
  binary footprint and entry branches change even though no suffix instructions
  execute per P74 token. No instruction-cache/BTB benefit is inferred.

| Function | Address | Size | Stack subtraction (before saved registers/alignment) |
|---|---|---:|---:|
| Shipped plain | `0x29380` | 812 B | 448 B |
| Gate 1 Free plain | `0x296b0` | 932 B | 384 B |
| Combined plain | `0x2a4c0` | 1699 B | 576 B |
| Shipped checkpoints | `0x213c0` | 2595 B | 640 B |
| Gate 1 Free checkpoints | `0x29a60` | 2641 B | 544 B |
| Combined checkpoints | `0x2ab70` | 4741 B | 768 B |

The frozen draft calls a new fast-path instruction a defect and stops after one fix.
That gate **fails**, so this commit is a correctness-verified research implementation
with a documented compiler blocker, not authorization or readiness to time it.
The suffix/layout costs also remain unmeasured. No speedup, regression bound, or
production integration is claimed. Other compilers/targets and the crate's MSRV were
not tested; scalar dispatch was tested on this x86_64 host. As with shipped MTF, direct
`Symbols` inputs must respect the scanner's symbol-value invariant (rank <=255).
