# External research report 180: the guardian on the i9 G3 preflight failure, and the toolchain repair (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with the console of the attempt and the
  maintainer's question, kept below the message.
- **Source:** the guardian.
- **In reply to:** the approved compact30 traversal capture (`just bench-c30-traversal`,
  branch head `515ab5c`, source pin `9267433`), run on the i9.

**Evidence cutoff:** 2 November 2026. The guardian's inspection of the attempt's archive,
`target/c30-traversal-20261102T170319Z.tar.gz` (sha256 `81545030…`). The archive itself
was not relayed here.

**What happened:**
- The corpus check, the source pin and the 37 analysis tests passed.
- The workstation built with rustc 1.98.0 / LLVM 22.1.8, where the reviewed walker used
  rustc 1.97.0 / LLVM 22.1.6.
- G3 found `room::<false>`'s listing different, and the run stopped before any timing.

It is a G3 preflight failure, not a compact30 performance result.

**This authorises:** a narrow toolchain-reproducibility repair:
- the recipe selects rustc 1.97.0 explicitly through rustup, without changing the global
  default;
- it verifies the compiler and LLVM identities before compiling;
- it fails with a setup instruction if the toolchain is missing;
- the strict G3 stays;
- a VM validation without timing;
- the updated recipe pin and workstation instructions.

**It does not authorise:**
- weakening G3;
- changing the walker, collector, benchmark design, thresholds or corpus;
- adopting the 1.98-built listing;
- rerunning the capture before approval;
- architecture or tuning work.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
No—this is not bad news for compact30. The benchmark correctly refused to run because your i9 compiled a different machine-code version from the one we reviewed.

The safety gate worked exactly as intended. No performance measurements were taken, and the candidate has not failed a benchmark.

I inspected your uploaded archive, including the assembly diff and manifest.

## What happened

Corpus verification

Passed

Frozen source commit

Passed

37 analysis tests

Passed

G3: machine-code identity

Failed

Performance capture

Not started

The crucial difference:

|               | Reviewed development build | Your i9 build         |
| ------------- | -------------------------- | --------------------- |
| Rust compiler | 1.97.0                     | 1.98.0                |
| LLVM          | 22.1.6                     | 22.1.8                |
| Source        | Frozen compact30           | Same frozen compact30 |

Your local Rust installation is newer. That can change register allocation, instruction scheduling and control flow even when the source is identical.

And the diff confirms substantive machine-code differences. For example, the compiler changes register choices, branch layouts and allocation-related instructions. This is not merely an address relocation or an overly strict comparison of harmless offsets.

So G3 was justified in stopping.

## Was it coded wrong?

There was an omission in the benchmark recipe, not a demonstrated bug in the walker.

Claude specified rustc 1.97.0 as part of the frozen experiment but did not make the recipe enforce that compiler version. It checked the resulting assembly instead, which caught the mismatch before timing.

That's a reasonable last line of defence, but it is less convenient than checking and selecting the correct toolchain at the beginning.

## What I recommend

The cleanest repair is to have Claude make the recipe use the exact originally reviewed Rust toolchain, rather than changing the architecture or accepting the new code generation.

It should:

1. Check that rustc 1.97.0 is available, with the expected LLVM version.
2. Build the frozen source with that explicitly selected toolchain.
3. Run G3 on the resulting binary.
4. Only permit the existing capture if the original listing matches.

I would not weaken G3, adopt the Rust 1.98-generated listing without review, or run a comparison between the two compiler versions.

This is a toolchain-reproducibility repair, not a new performance experiment.

## Send Claude this

The approved i9 attempt stopped at G3 before timing. I've preserved its archive.

The workstation uses rustc 1.98.0 / LLVM 22.1.8; the reviewed walker used rustc 1.97.0 / LLVM 22.1.6. The uploaded `room-false.diff` shows substantial machine-code differences, not just relocation. The source pin and corpus hash passed, and 37 tests passed.

**Please make a narrow toolchain-reproducibility repair.**

Update the recipe to select the originally reviewed rustc 1.97.0 explicitly, preferably through rustup's pinned-toolchain invocation without changing my machine's global default. Verify the compiler and LLVM identities before compilation, then retain the existing strict G3 comparison.

If that toolchain isn't installed, fail with a clear setup instruction rather than silently using the default compiler. Do not weaken the listing check or change the Rust walker, collector, benchmark design, decision thresholds or corpus.

Validate the corrected recipe on the VM without performance timing, confirm the frozen walker listing, and return the updated recipe pin and exact workstation instructions. The original failed archive is a G3 preflight failure, not a compact30 performance result.

No new architecture or tuning work is authorized.

Your failed archive is worth keeping: it proves that the experiment correctly rejected a changed binary. Its manifest records the frozen source commit and the local binary hash, so the failure is traceable.

Don't rerun the benchmark command yet. Once Claude has made the toolchain selection reproducible, we can approve the corrected capture recipe.

And this needn't interrupt the blog beyond this small fix. We are dealing with a build-environment mismatch, not a disappointing result from the actual experiment.
`````

## The attempt's console and the maintainer's question, as relayed

`````text
louis 🌟 ~/dev/mibz2 $ git fetch origin research/compact30-walker
git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD
git status --short

just bench-c30-traversal
[fetch and switch output omitted here; the branch head was 515ab5c with no tracked changes]
/home/louis/dev/mibz2/target/walk-decoder/work/corpora/wikidata-20261026.bz2: OK
c30: the frozen commit 9267433661f4a440c92831a4f1b38b6af61e4726, corpus checked, mode capture
rustc 1.98.0 (88d9e12ae 2026-08-18)
binary: rustc
commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea
commit-date: 2026-08-18
host: x86_64-unknown-linux-gnu
release: 1.98.0
LLVM version: 22.1.8
[dependency compile lines omitted]
    Finished `release` profile [optimized + debuginfo] target(s) in 13.60s
37 tests passed
c30: room::<false> is the reviewed listing: no
c30: G3 fails (see room-false.diff); no run
archive: /home/louis/dev/mibz2/target/c30-traversal-20261102T170319Z.tar.gz (28832 bytes, sha256 815450301cd528bb3c292f1ddcb45ea12a7992200fee761168839eb9e7073ef5): 3 files and MANIFEST.json, commit 9267433661f4a440c92831a4f1b38b6af61e4726
error: Recipe `bench-c30-traversal` failed with exit code 1

> oh... this looks bad?

> Was it coded wrong?
`````

The two bracketed omissions were made when filing; nothing else differs from the console
as relayed.
