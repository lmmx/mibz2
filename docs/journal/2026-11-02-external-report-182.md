# External research report 182: the guardian on the second i9 G3 failure, and the G3 boundary repair (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with the attempt's console and its archive,
  kept below the message.
- **Source:** the guardian.
- **In reply to:** the approved compact30 traversal capture (`just bench-c30-traversal`,
  branch head `0d85566`, source pin `9267433`, rustc 1.97.0), run on the i9.

**Evidence cutoff:** 2 November 2026. The guardian's inspection of the attempt's archive,
`target/c30-traversal-20261102T184101Z.tar.gz` (sha256 `fe6ad4e5…`), compared with the
frozen reference listing. The archive is kept beside this report, in
`2026-11-02-external-report-182/`.

**What happened:**
- The corpus check, the source pin, the toolchain identity (rustc 1.97.0, `2d8144b78`,
  LLVM 22.1.6) and the 37 analysis tests passed.
- G3 found `room::<false>`'s listing different, and the run stopped before any timing.
- The whole difference was after the function's last instruction (a call to
  `_Unwind_Resume`): six `int3` in the reference, one multi-byte `nop` on the i9. The first
  1,552 normalised instructions matched, the 21-instruction ordinary path included.

It is a G3 preflight failure, not a compact30 performance result. The walker stays
accepted.

**This authorises:** a narrow G3 comparison-boundary repair:
- compare the complete executable function and all its control flow, excluding only
  demonstrable trailing linker alignment padding, preferably bounded by the symbol's size;
- a regression on the archived i9 mismatch, and a negative test changing a real
  instruction, which must still fail G3;
- the new recipe pin and its tests, showing this exact padding difference accepted before
  another capture is authorised.

**It does not authorise:**
- ignoring NOPs globally, trimming internal differences, or weakening the comparison of the
  21-instruction ordinary path;
- changing the walker, the Rust collector, the analysis, the thresholds or the pinned
  compiler;
- timing, or new optimisation work;
- another capture before approval.

The two earlier failed archives remain preflight evidence, not performance results.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
I found it. This time, the mismatch is in the assembly comparison's treatment of padding—not in the compact30 walker. You were right to be frustrated.
I inspected the archive you uploaded, including `room-false.diff`, and compared it with the frozen reference assembly.
The exact failure
Your second attempt successfully passed the compiler check:

* Rust 1.97.0, correct commit.
* LLVM 22.1.6, correct version.
* Frozen source commit `9267433`.
* Corpus hash correct.
* All 37 tests passed.

The entire assembly diff is just:

```
1552: reference 'int3'
      this binary 'nop WORD PTR [rax+rax*1+0x0]'

length: reference 1558
        this binary 1553 instructions
```

The frozen reference ends with six `int3` instructions after a call to `_Unwind_Resume`. Your i9 build instead has a multi-byte `nop` in that trailing alignment area.
The first 1,552 normalized instructions match. That includes the reviewed 21-instruction ordinary walker path.
The comparison script uses the address of the next symbol to determine where the function ends. As a result, it also compares bytes occupying the alignment space after the function. Different linker placement can change that padding without changing the executable walker.
This is substantially different from the first failed attempt, where Rust 1.98 actually changed the compiled instructions.
My decision
The walker remains accepted. This is an overly strict G3 boundary, not a reason for another performance-review cycle.
I would not ask you to perform a third attempt until Claude makes one final, very small adjustment: compare the executable function body while excluding only verified trailing alignment padding.
Crucially, that does not mean ignoring all NOPs or allowing arbitrary assembly differences.
Send Claude this:
The second i9 attempt stopped before timing at G3 again. I inspected its archive.
The reviewed Rust 1.97.0 / LLVM 22.1.6 compiler identity now passes. The assembly comparison differs only at the trailing alignment padding:

* Reference: 1,558 normalized instructions, ending in six `int3` padding instructions.
* i9: 1,553 normalized instructions, ending in one multi-byte `nop`.
* The first 1,552 normalized instructions match exactly, including the reviewed ordinary walker.

The frozen reference's last executable sequence ends in a call to `_Unwind_Resume`; the difference is in the following alignment space.
Please make a narrow G3 comparison-boundary repair. Compare the complete executable function and all real control-flow instructions, but exclude only demonstrable trailing linker alignment padding. Prefer symbol-size or function-boundary evidence where available. Do not globally ignore NOPs, trim internal differences, or weaken comparisons of the 21-instruction ordinary path.
Add a regression using the archived i9 mismatch, plus a negative test that changes a real instruction and must still fail G3. Keep the walker, Rust collector, analysis, experimental thresholds and pinned compiler unchanged.
Return the new recipe pin and tests. No timing or new optimization work. The two earlier failed archives remain preflight evidence, not performance results.
This correction is materially smaller than our earlier repairs. The fact that 1,552 instructions matched exactly is strong confirmation that the correct compiler fixed the substantive reproducibility issue.
I wouldn't put you through another blind retry. The next recipe should specifically demonstrate acceptance of this exact padding difference before we authorize another capture.
Your uploaded archive has the evidence needed to do that. No need to run anything locally for now.
`````

## The attempt's console and the maintainer's remark, as relayed

`````text
> not again...

louis 🌟 ~/dev/mibz2 $ rustup toolchain install 1.97.0 --profile minimal

git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD

just bench-c30-traversal
info: syncing channel updates for 1.97.0-x86_64-unknown-linux-gnu

  1.97.0-x86_64-unknown-linux-gnu unchanged - rustc 1.97.0 (2d8144b78 2026-07-07)

info: checking for self-update (current version: 1.29.2)
Already on 'research/compact30-walker'
Your branch is up-to-date with 'origin/research/compact30-walker'.
remote: Enumerating objects: 700, done.
remote: Counting objects: 100% (649/649), done.
remote: Compressing objects: 100% (319/319), done.
remote: Total 538 (delta 279), reused 471 (delta 213), pack-reused 0 (from 0)
Receiving objects: 100% (538/538), 1.73 MiB | 4.69 MiB/s, done.
Resolving deltas: 100% (279/279), completed with 49 local objects.
From github.com:lmmx/mibz2
   515ab5c..0d85566  research/compact30-walker -> origin/research/compact30-walker
 * [new branch]      docs/blog-mlp-edits       -> origin/docs/blog-mlp-edits
 * [new branch]      docs/blog-mlp-ilp         -> origin/docs/blog-mlp-ilp
   8ed9ec5..8823dcf  docs/external-reports-74  -> origin/docs/external-reports-74
   5e8bc1e..a04e8b5  gh-pages                  -> origin/gh-pages
   13e9854..c74f5e2  master                    -> origin/master
Updating 515ab5c..0d85566
Fast-forward
 .just/bench.just                                                                                                     | 31 ++++++++++++++++++++++++++-----
 docs/journal/2026-11-02-compact30-traversal-protocol.md                                                              | 44 +++++++++++++++++++++++++++++++++++++-------
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/MANIFEST.json     | 54 ++++++++++++++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/c30-analysis.json | 59 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/console.log       | 42 ++++++++++++++++++++++++++++++++++++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/room-false.diff   |  0
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/rustc-vV.txt      |  8 ++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/refused-toolchain-missing.console        |  8 ++++++++
 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/refused-wrong-identity.console           | 20 ++++++++++++++++++++
 9 files changed, 254 insertions(+), 12 deletions(-)
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/MANIFEST.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/c30-analysis.json
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/console.log
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/room-false.diff
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/dry-run-default-1.98.1/rustc-vV.txt
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/refused-toolchain-missing.console
 create mode 100644 docs/journal/2026-11-02-compact30-traversal-protocol/runs-9267433-toolchain/refused-wrong-identity.console
0d85566
/home/louis/dev/mibz2/target/walk-decoder/work/corpora/wikidata-20261026.bz2: OK
c30: the frozen commit 9267433661f4a440c92831a4f1b38b6af61e4726, corpus checked, mode capture
rustc 1.97.0 (2d8144b78 2026-07-07)
binary: rustc
commit-hash: 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
commit-date: 2026-07-07
host: x86_64-unknown-linux-gnu
release: 1.97.0
LLVM version: 22.1.6
cargo 1.97.0 (c980f4866 2026-06-30)
c30: building with the reviewed toolchain rustc 1.97.0, LLVM 22.1.6, no rustflags
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.27
   Compiling quote v1.0.47
   Compiling libm v0.2.16
   Compiling find-msvc-tools v0.1.15
   Compiling shlex v2.0.1
   Compiling pkg-config v0.3.34
   Compiling thiserror v2.0.21
   Compiling libc v0.2.191
   Compiling bytemuck v1.25.2
   Compiling mibz2 v0.2.1 (/home/louis/dev/mibz2/target/c30-traversal-pin)
   Compiling cc v1.10.0
   Compiling syn v3.0.9
   Compiling fearless_simd v1.2.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling thiserror-impl v2.0.21
   Compiling bzip2 v0.4.4
   Compiling pbz2-core v0.2.0
   Compiling pbz2 v0.2.0
    Finished `release` profile [optimized + debuginfo] target(s) in 13.50s
37 tests passed
c30: room::<false> is the reviewed listing: no
c30: G3 fails (see room-false.diff); no run
archive: /home/louis/dev/mibz2/target/c30-traversal-20261102T184101Z.tar.gz (14039 bytes, sha256 fe6ad4e515c3b4922ad590d0b3cc877935739e5f035dae45880f42e26b19fa38): 4 files and MANIFEST.json, commit 9267433661f4a440c92831a4f1b38b6af61e4726
error: Recipe `bench-c30-traversal` failed with exit code 1
louis 🌟 ~/dev/mibz2 $
`````
