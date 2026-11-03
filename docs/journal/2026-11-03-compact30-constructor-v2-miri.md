# Compact30: targeted Miri cases for constructor v2 (report 212)

**All 69 targeted cases passed under Miri:** every byte exact, no undefined behaviour reported.
They cover the classes that the interrupted `ctor2-synthetic --quick` run never reached
([the prototype report](2026-11-03-compact30-constructor-v2-prototype.md), §3). This is
optional supporting evidence for the
[independent audit's PASS](2026-11-03-compact30-constructor-v2-audit.md). Nothing was timed.

**Authority.** The guardian permitted an optional, untimed, targeted Miri pass, provided it
adds only test selection or focused cases and no change to the audited constructor. It must
not block the v2 cost-protocol work, and must never run beside a performance capture. These
runs were made on the shared VM, not the i9, and no capture was running.

**What was run.**
- **Code:**
  - `examples/c30walk/ctor2miri.rs` (new) and a dispatch line for `c2miri` in
    `examples/c30walk.rs`, at `33af86f` on `research/compact30-ctor2-miri`, branched from
    `7f723b2`.
  - `examples/c30walk/ctor2.rs` is unchanged: the audited blob at `8beffe2`.
  - The module calls `Ctor2::build::<true>` and `::<false>` and the oracle `C30::build`
    read-only, and compares every byte of `rec` and `raw` with the oracle, after both builds.
  - The arenas are never poisoned, so Miri would report a read of any unwritten byte.
  - The walker isn't run here; the release suites cover it.
- **Commands:**
  `cargo +nightly miri run --example c30walk -- c2miri {small,span,large}`.
- **Miri setup:** nightly rustc 1.101.0 (`16d1ffac1`, 2026-10-26), Miri `16d1ffac17`;
  Stacked Borrows, no flags.

| set | cases | wall time | covers |
|---|--:|--:|---|
| `small` | 61 | 49 s, including the build | forced RAW at permille 50 and 1000 (runs, random, a crossing group, one symbol); 15 malformed counts (the five kinds of ctor2's class 12 × three configurations), each rejected with both arrays empty and followed by a valid build in the same arenas; portions finished only at the end of input (counts 1, 29, 30, 31; permille 0 and 1000); reuse over sizes 1, 0, 31, 200, 95, 0, 150, 1, 230 with the configuration changing |
| `span` | 2 | 4 min 16 s | the exact cases: 3 → 61,440 (span 65,533) **stays ordinary**, 0 wide groups asserted; 2 → 61,440 (span 65,534) **becomes RAW**, 1 wide group asserted |
| `large` | 6 | 13 min 1 s | portions left open across an epoch until the end of input (n 61,500, counts 1 and 31); forced RAW across an epoch (permille 1000: 1,604 forced groups); reuse shrinking to 31 rows after an epoch-scale build, then growing to 61,500 at permille 50; a group spanning two epochs plus one row (n 123,081), whose truncated 16-bit difference looks small, **becomes RAW**, 1 wide group asserted |

Each case's line in the consoles gives its finish counters: fast, ordinary, wide, crossing,
forced, and epochs.

**Coverage, combined with the interrupted run.** The earlier run's 473 cases reached small
blocks, crossing RAW, an epoch boundary and wide RAW above 61,440 rows. These runs add every
class listed as unreached there:
- the exact 65,533/65,534 span cases;
- portions pending to the end of input, below and across an epoch;
- repeated construction, shrinking and growing;
- malformed counts;
- forced RAW.

**Limits.**
- These cases were chosen to reach those classes. Miri doesn't make the suite exhaustive, and
  it checks only the executions it runs.
- The cases match ctor2's own suite in form, not in number. The ~54,000-case release suite
  remains the breadth evidence.
- Stacked Borrows was used, not Tree Borrows.
- The arenas were unpoisoned throughout. This is intentional for Miri, but it means the
  poison-pattern checks don't apply here.
- The walker wasn't run under Miri in these cases.
- Each 61,000-row case took about 2 minutes. The interrupted run's large cases took about
  9 minutes each, because it also ran the oracle's and production's walks.
- `ctor2miri.rs` isn't rustfmt-formatted. The runs were made at this exact source, so it is
  kept unchanged.

## Evidence

In [2026-11-03-compact30-constructor-v2-miri/](2026-11-03-compact30-constructor-v2-miri/)
(colour codes stripped):

| file | cases | sha256 |
|---|--:|---|
| `c2miri-small.console` | 61 | `7c724df994f0…` |
| `c2miri-span.console` | 2 | `7194972bda99…` |
| `c2miri-large.console` | 6 | `9ecf05501e8f…` |
| `toolchain.txt` | — | — |
