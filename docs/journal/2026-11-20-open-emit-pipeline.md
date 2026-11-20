# Open's `emit`, pipelined one step (negative, stopped at the assembly gate)

**Question.** `emit`, the walk over the retained rows, is Open's largest stage (35.6-47.7% of
the shipped kernel's time on the six admitted blocks, `2026-11-20-open-phase-split.md`). Neon
proposed a one-step software-pipelined walk, `emit_pipelined::<WIDE>`: load the next row's
edge (and, wide, its code from `extra`) before appending the current row's word, and defer an
out-of-bounds panic until after the current word is appended and the `out.len() >= n` check has
run. Is it equivalent to the shipped walk? Does its machine code actually schedule the next load
earlier, without paying for it elsewhere? Only if so, is it faster?

**Answer.** It is equivalent: no mismatch in 64,351,462 exhaustive small cases, the open
kernel's audit and b5 suites pass with it swapped in, and the six real blocks are byte-exact.
It fails the assembly gate. In the release build (fat LTO, one codegen unit), the variant's six
`append_code` call sites (three per `WIDE` instance) make LLVM stop inlining `append_code`:
every row, literals included, becomes a call that pushes five registers and reloads four
arguments from the stack. The narrow literal row goes from 26 instructions to 48.
The next edge load does come before the `code < 256` branch, but the shipped walk's next load
already depends only on the loaded edge, not on the append, so out-of-order execution can
issue it early anyway. The gate fails, and by the rule fixed beforehand nothing was timed.

## The variant, adapted

`src/open/pipelined.rs`, behind a new research feature `open-emit-pipelined`. With the feature,
`emit` (in `Kernel::run` and the phase harness's `stage_emit`) is `emit_pipelined`. Without it,
`emit` is `emit_shipped`, the shipped body renamed, unchanged. The module is also compiled for
tests, where both walks are present.

Neon's code already matches the shipped walk's termination: at most `edges.len()` rows appended,
a stop after the row whose successor is the mark, and a stop once `out` holds `n` bytes. Two
adaptations:

- **The deferred panic.** Neon wrote `let _ = edges[j as usize]; unreachable!()`. Here it is a
  `#[cold]` `out_of_bounds(index, len) -> !` with the slice index's own message, so the panic
  can't depend on whether `let _ = place` evaluates its index.
- **The dispatch.** As proposed, `extra.is_empty()` picks `walk::<false>` or `walk::<true>`
  once (`#[inline(always)]`). The shipped walk tests it every row.

No behavioural difference from the shipped walk was found or needed.

## Step 1: equivalence

**The sweep** (`pipelined_emit_matches_shipped_exhaustively`, release, about 6 minutes):

- **Dictionary:** `spelling = "xyz"`, `offsets = [0, 2, 2, 5]`. Word 0 is "xy", word 1 is
  empty, word 2's slice runs past `spelling`, and word 3 runs past `offsets`.
- **Codes:** 65 (a literal) and 256-259.
- **Shape:** `edges.len()` 0-4, `bits` 1-3, each successor in `0..=len` as far as the mask
  allows (`len` is out of bounds), every `mark` in `0..=len` (out of bounds included), and `n`
  0-6.
- **Narrow:** each edge's code and successor varied.
- **Wide:** each edge's successor varied, with its own code bits set to 7 (and ignored), and
  `extra` of length `len - 1`, `len` and `len + 1` with every code. `extra[len]` is never read,
  since its row is out of bounds of `edges` first, so it is fixed.
- **The check:** each case runs both walks under `catch_unwind` on a dirty 3-byte buffer, and
  compares panic or no panic and, without a panic, `out`.

| | cases | panics (both) | mismatches |
|---|---:|---:|---:|
| narrow | 20,109,866 | 16,090,634 | 0 |
| wide | 44,241,596 | 35,711,348 | 0 |

No test was run to show that the sweep would catch a deliberately broken walk.

**The suites, swapped in.** `cargo test --release --features open-emit-pipelined --lib open::`
ran `checks::all_port_arrows` (the exhaustive arrow audit), `capped_runs_and_shifted_marks`,
`residual_segments_on_short_cycles` and the b5 suite (`every_small_column_at_every_row`,
`arbitrary_columns_through_one_kernel`, `stress_streams_decode_exactly`, the gate and scratch
tests, `miri_open_kernel_small_columns`). All passed: 11 passed, 2 ignored (`scratch_search`
and the real-block test below).

**Real blocks** (`pipelined_emit_matches_shipped_on_real_blocks`, ignored without
`MIBZ2_EMIT_BLOCKS`). The kernel's own construction of each admitted column, walked both
ways, gives identical output, equal to a dense-LF inversion from `primary_index`:

| block | n | edges | extra |
|---:|---:|---:|---:|
| 0 | 899,931 | 89,874 | 0 |
| 2 | 900,000 | 112,070 | 0 |
| 85 | 899,935 | 156,404 | 0 |
| 143 | 899,869 | 196,841 | 0 |
| 207 | 899,998 | 135,921 | 0 |
| 210 | 899,959 | 204,281 | 0 |

All six are narrow (`extra` empty), so on real data only `walk::<false>` matters.

## Step 2: assembly

**The binaries.** `examples/open_phases` from the release profile (fat LTO, one codegen
unit), built with `open-phases` (shipped) and with `open-phases,open-emit-pipelined`, then
`objdump -d -C`. In both, the walk is one out-of-line function, called from `Kernel::run` and
from `stage_emit`.

| | shipped `emit_shipped` | variant `emit_pipelined` |
|---|---|---|
| code size | 884 B | 878 B + `append_code` 292 B, out of line |
| narrow literal row | 26 instructions | 27 in the loop + 21 in `append_code` = 48 |
| stack traffic per literal row | 2 reloads (`bits`, `n`) | 7 reloads (`edges`, `mark`, `n`, four `append_code` arguments), 5 pushes and 4 pops, `call`/`ret` |
| reloads after the word's `memcpy` | 7 | 1 inside `append_code` (it saves its registers on entry, on every row) |
| `extra.is_empty()` test per row | yes | no (dispatched once) |
| bounds checks in the loop | `edges[j]`, `extra[j]`, `offsets[w]`, `offsets[w+1]`, the `spelling` slice | the same five sites; `edges`/`extra` as `get` with a cold deferred panic |

**The shipped narrow literal row** is a load, a few ALU ops and the push:

```text
cmp %rdi,%rsi ; jbe oob          ; edges[j] bounds
mov (%r8,%rdi,4),%r13d           ; edge = edges[j]
test %r15,%r15 ; je narrow       ; extra.is_empty(), every row
mov %r13d,%r12d ; mov 0x2c(%rsp),%ecx ; shr %cl,%r12d   ; code (bits reloaded)
cmp $0x100,%r12d ; jb literal
mov 0x10(%rbx),%r14 ; cmp (%rbx),%r14 ; je grow ; mov 0x8(%rbx),%rax
mov %r12b,(%rax,%r14,1) ; inc %r14
and %edx,%r13d                   ; j = edge & mask
mov %r14,0x10(%rbx) ; cmp %r11d,%r13d ; jne
cmp 0x90(%rsp),%r14 ; jae ; cmp %rsi,%rbp ; lea 0x1(%rbp),%rbp ; jb top
```

**The variant's narrow loop**, with the load hoisted above the branch, but the branch now
inside a call:

```text
mov %r12d,%edi ; mov %esi,%ecx ; shr %cl,%edi   ; code of the current row
inc %rbp ; cmp %rbp,%r15 ; je tail
and %r13d,%r12d ; cmp 0x88(%rsp),%r12d ; je tail   ; j == mark (mark on the stack)
cmp %r12,%r15 ; jbe deferred_oob
mov 0x28(%rsp),%rax ; mov (%rax,%r12,4),%r12d     ; next edge, loaded first
mov %r15,%r14 ; mov %esi,%r15d
mov 0x8(%rsp),%rsi ; mov 0x10(%rsp),%rdx ; mov 0x70(%rsp),%rcx ; mov 0x78(%rsp),%r8 ; mov %rbx,%r9
call append_code                 ; push x5, cmp $0x100, push the byte, pop x4, ret
mov %r15d,%esi ; mov %r14,%r15 ; mov 0x80(%rsp),%r14
cmp %r14,0x10(%rbx) ; jb top
```

**The answers to the brief's questions:**

- **Does the shipped code already issue the next `edges` load before the append?** Not in
  program order: the load is at the loop top, after the previous row's branch and push. But
  `j = edge & mask` depends only on the loaded edge, so with the branches predicted the
  out-of-order core can issue the next load as soon as the current one returns, whatever the
  append is doing. The pipelining changes program order, not the dependence chain
  (`edges[j] -> j -> edges[j']`), which is the walk's real latency.
- **Does the variant's load really come before the `code < 256` branch?** Yes: the load is
  issued before `call append_code`, and the branch is inside it.
- **Spills across the `extend_from_slice`/`memcpy` path.** The shipped walk reloads seven values
  from the stack after each word's `memcpy`. The variant's narrow loop pays its call-boundary
  cost on every row, literals included.
- **The bounds-check count** is the same: five panic sites a row (four in narrow).
- **The inner loop's instruction count:** 26 for a shipped narrow literal row, 48 for the
  variant's.
- **Code size:** 884 B shipped, 1,170 B for the variant (878 + 292).

**A probe, outside the candidate.** With a local `#[inline(always)]` copy of `append_code`, the
variant inlines again (2,710 B, three copies of the append). Its narrow literal row is 25
instructions, with the edge load before `cmp $0x100`. It reloads `out` from the stack every row
and eight values after each `memcpy` (the shipped walk reloads `bits` and seven). That is a real
scheduling difference at about equal spill cost. It is a different variant from the one
proposed, so under the no-rescue rule it was not timed. Its asm was only inspected, and its
code is not in this PR.

## Step 3: timing

Not run. Before the data, the brief made the timing conditional on Step 2 showing "a real
scheduling difference with no offsetting spills". The candidate as proposed shows an
offsetting cost on every row (the out-of-line `append_code`), so the gate fails. The decision
rule fixed for the timing (PASS if complete decode is at most 0.98x shipped Open on at least 4
of the 6 blocks, and none above 1.01x) was therefore never applied.

## Verdict

**Equivalent, not adopted.** The pipelined walk is a correct replacement, but compiled as
proposed it is worse code than the shipped walk: almost twice the instructions on a literal
row, with call overhead. The scheduling it aims at is one that out-of-order hardware already
gets from the shipped loop, whose next address depends only on the loaded edge.

- **What remains open.** Whether a force-inlined form (the probe above) is faster is untested.
  Since it is a new variant, it needs its own registration if anyone wants it. The phase split
  points at `emit`'s dependent-load chain and its word copies, which a one-step reordering
  doesn't shorten.
- **Reproduce:**
  `cargo test --release --lib open::pipelined -- --nocapture --test-threads=1`, then the same
  with `--features open-emit-pipelined --lib open::`. Run with
  `MIBZ2_EMIT_BLOCKS=<dir> ... real_blocks -- --ignored` for the blocks.
