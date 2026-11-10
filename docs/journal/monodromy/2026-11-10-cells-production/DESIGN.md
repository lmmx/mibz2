# Cells kernel into production, slice one: design note (10 Nov 2026; before any code)

For the Coordinator's QUEUED 6774426061. Written from two read-only maps of master at 1a871bb
(`prod_call-path.json`, `prod_safety-tests.json`, beside this file: facts with path:line) and
from the two journal entries. The maps of the bench gates and of what the example kernel
lacks had not returned when this was written; what depends on them is marked **open**.
Nothing here is built.

## 1. What slice one is

A second inverse-BWT kernel type beside `Segmented`, off by default, that tries the cells
kernel on an admitted block and otherwise does exactly what `Segmented` does.

- **Not a cargo feature and not an environment switch.** Master selects routes by the type
  parameter of `decode_block<I: bwt::InverseBwt>` (`src/lib.rs:332`; `Scratch<I =
  bwt::Segmented>`, `src/lib.rs:278`); there is no feature, variable or builder option for
  it. A feature that is off by default is built by none of `cargo test`, the Miri job,
  fuzz-smoke or the bench (only clippy). A type is built and tested by all of them, and
  "off by default" is simply that the default stays `Segmented`.
- **Where it enters:** the new type's `invert_runs` (the seam #324 made:
  `InverseBwt::invert_runs(&s.runs, n, orig_ptr, &mut s.ll, k)`, called at
  `src/lib.rs:341-362`). `Segmented::invert_runs` (`src/bwt.rs:152-165`) is not edited: its
  body (`runs::fill`, then the room walk backward) becomes a function both types call.
- **A refusal is not a failure.** `invert_runs` must not return `false` for a refused
  block (`decode_block` then reaches `debug_assert!(false)`, `src/lib.rs:361`): it falls
  through to the shared body and returns what that returns.
- **The records** are private to module `runs`: one `pub(crate)` accessor, no change of
  format.
- **The flip** (slice two) changes the default type, or makes `Segmented::invert_runs` call
  the same function, on its own CI bench. Not in this PR.
- **Scope.** `Workers(n)` rows also run `decode_block::<Segmented>`; since `Segmented` is
  untouched, no path changes by default. The team path (`Cooperative`) is not touched.

## 2. Admission: three tests, each refusing with nothing written

The kernel writes to its own buffer until its writer runs, so every refusal leaves `out`
as the caller gave it.

| test | when | refuses if | cost of a refusal |
|---|---|---|---|
| r/n | before any work; r is the records' count | r/n ≥ T (0.10 fits Wikidata; fitted on the same blocks) | a compare |
| the floor | after the order stage | r₂ − 1 > the budget, r₂ counted while the orders are laid out | the order stage: 0.37 ns a row pooled on Wikidata, 0.6 on its high-r blocks |
| the budget | in the induction, a counter a cell | cells > B·n (B = 0.16: break-even on Wikidata and JSON) | order + B·n cells: about 0.9 of production's time on the Wikidata blocks it refuses |

- The floor is sound: S ≥ r₂ − 1 (entry 1, Theorem 4; entry 2, §6 item 1).
- `random` and `records` must stay on the old route (PERFORMANCE-POLICY rule 3): the r/n
  test is what does it, before any work. **Open:** the `records` corpus's r/n, from the
  bench map.
- Periodic blocks (`(ab)^k a`, S = n/2) are refused by the budget, at its cost. The
  division rule is not in slice one.
- The PR states the three costs above from its own counts.

## 3. The kernel as `src/` code

- **One safe entry, `unsafe` inside.** The safe function checks: the records tile n;
  orig_ptr < n (already checked at `src/lib.rs`, checked again); the buffer is sized for
  (r, the budget). `#![deny(unsafe_op_in_unsafe_fn)]` in the module (master enables no
  unsafe lint); a SAFETY comment on every block, naming the invariant.
- **A buffer bound that follows from the budget.** The example sizes one zero-filled
  buffer for n cells: 128 n + 4,912 bytes. With cells ≤ B·n enforced by the counter:
  slots 24 B × 2 × (r + cells), words as bytes 16 B × cells, rules 16 B × cells, pads.
  At B = 0.16 that is about 13 n + 48 r bytes of address space against 128 n; what is
  touched stays about 5.5 MB a 900,000-row block. **The same counter that is the budget
  bounds all four regions:** that is the core of the safety argument.
- **No panic on the path.** The example's three asserts (`examples/rauzy.rs:902`,
  `:931-934`, `:1036-1039`) become refusals in release and `debug_assert!` besides.
- **Portability.** The prefetch only under `cfg(target_arch = "x86_64")` and not under
  `cfg(miri)`; nothing else is architecture-specific. Offsets are read and written with
  `read_unaligned` / `write_unaligned` on native-endian integers that never leave the
  buffer. aarch64 and Windows build and run it without the prefetch in slice one.
- **A column that is no BWT** (a corrupt stream) must decode to the walk's bytes and the
  same error with the same computed CRC (PERFORMANCE-POLICY.md:16). The kernel's drop rule
  and its writer's repeat for a cycle shorter than n do this in the example, checked there
  only against `bwt::walk`; here it is checked through `decode_block` against libbzip2.

## 4. Evidence (B5: `docs/team/RULES.md:58`)

1. **A written safety argument,** as a table in the PR's journal entry: each unsafe
   operation, its invariant, where the invariant is established, and its proof scope.
2. **Differential.** The oracle is libbzip2 1.0.8 through the `bzip2` dev-dependency
   (`tests/common/mod.rs:11-38`). The new type against `Segmented` and against the oracle
   on the test corpora and on: Fibonacci, Thue–Morse, a period and one byte, `(ab)^k a`,
   the two-run columns `b^k a^l` (closed form: Euclid's subtractions; entry 2, §4.7),
   zeros, uniform bytes, and blocks at each admission edge.
3. **Fuzz.** A new MODE in `examples/fuzz.rs` (the repository's fuzzer; there is no
   cargo-fuzz) that forces the route with a type whose thresholds are open (T = 1, B = 1),
   and counts: blocks through the cells, refusals by each test, cycles shorter than n. A
   step in fuzz-smoke. **The reach is asserted, not printed** (existing modes only print).
4. **Miri.** `miri_*` unit tests beside the kernel (CI's job runs `cargo miri test --lib --
   miri_`), and the new paths added to `.github/workflows/miri.yml` (a new file is not
   covered until listed). Miri cannot see a write from one region of one allocation into
   the next. So under `cfg(any(miri, debug_assertions))` the four regions are separated by
   canary gaps that are checked after every block, and the tests include exact-fit cases at
   the budget. There is no Miri and no nightly on this container: the evidence is CI's.
5. **Mutation.** No configuration, recipe or job exists; #324 ran cargo-mutants 27.1.0 by
   hand and made a hand pass over the `unsafe fn`s, which the tool skips. The same here:
   the three admission tests are safe code and get the tool; the kernel gets the hand
   pass. cargo-mutants is not installed on this container.
6. **An adversarial review by another agent.** **Open:** the assignment names the
   Guardian (#161); ROLES.md:73 and SPEC.RESEARCHER.md:22 call the Researcher's B5 review a
   hard gate (#162). The Coordinator's to say.

## 5. Bench

- Slice one changes no default path, so the PR bench (head against base, which is itself a
  byte comparison and exits 1 on a mismatch) must show nothing: that is its gate.
- The flip is what CI's one-thread rows decide (`wikidata` and `text` faster beyond noise,
  no primary-corpus loss).
- **Open:** how the bench binary and the workstation recipe select a kernel type on one
  build (the bench map). If by nothing today, slice one adds a kernel name to the bench's
  own options, not to the library.

## 6. Order of work

1. This note on #145 (first milestone).
2. `src/cells.rs` and its tests in a scratch worktree off master: the kernel moved, the
   bounded buffer, the three tests of admission, the shared fallback body.
3. The differential tests and the forced fuzz mode; the Miri paths.
4. The journal entry with the safety table and the counted costs of a refusal.
5. The PR on `claude/sharp-meitner-cx0v78` once #362 has merged; milestones on #145.

## 7. After the bench map (`prod_bench-gates.json`; added 18:00Z)

The third map returned after this note was posted. It settles §5's open point and adds
four things the PR must be shaped round.

- **How a merge is judged.** One workflow, `Benchmark` (`.github/workflows/bench.yml`): the
  head and its merge base as two arms of `examples/ab.rs`, 7 corpora, threads 1, 2 and
  nproc, 10 paired rounds; a row is "faster" or "slower" only when the 95% interval of the
  head/base ratio excludes 1. The arms are hard-coded and drive `BzReader::new` and
  `BzReader::parallel`, which reach `decode_block` with `Segmented` at every thread count
  on CI's runners. **So slice one, with the default unchanged, is neutral by construction,
  whether the switch is a feature or a type.** To have CI measure the kernel before the
  flip, slice one adds a third arm to `examples/ab.rs` that decodes with the new type: the
  bench's own code, not the library's. The same arm is what a workstation A/B on one build
  needs.
- **`text` will not be faster.** The assignment's gate reads "wikidata and text faster
  beyond noise". The kernel's own record puts text outside its domain (prose, r/n 0.367:
  4.0× slower), so the admission refuses text blocks at the first test and the `text` rows
  can only be level. What the flip can show is `wikidata` faster and every other corpus
  level. Whether that meets PERFORMANCE-POLICY's rule 1 is the Coordinator's reading; the
  bench's `text` corpus was never measured by this lane.
- **The whole-decoder gain is unmeasured and smaller than the stage's.** 0.82 to 0.86 is
  the inversion alone; the inversion is a part of a block's decode. The first number that
  matters is the third arm's row on `wikidata` at one thread.
- **Peak resident memory has a gate** (rule 4: +10% on any corpus, or a justification).
  Address space is not counted, touched pages are. A refused block still uses production's
  table, so on a mixed corpus both sets of pages are resident. The budget-bounded buffer of
  §3 is what keeps this small; the PR reports the bench's RSS column.
- **Layout.** #324's two harms (`random`, `records` at one thread) were on blocks that ran
  none of its code, put down to `Scratch` growing inline. The new type's state goes behind
  a pointer in `Scratch`, not inline, and the neutral rows of slice one are the check.
- **One bench run decides, on the final head:** no fix-up pushes to `src/` after the run
  that is to count.
