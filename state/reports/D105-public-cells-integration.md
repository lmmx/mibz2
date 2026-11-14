# D105 — scoped public-path integration review, PR382

Checkpoint [6868538108](https://github.com/lmmx/mibz2/pull/161#issuecomment-6868538108).
Reviewed head **a9c7716aaf664a7898f3d8f37c955e09d5204a23**, base **6dec95ba6aaf59d5134e5ab119fd52433e2032ee**, 2026-11-14.
Authority: team SPEC/SPEC.GUARDIAN/MODE/RULES/PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763. Independent reviewer; no implementation involvement.

**G1: scoped PROCEED for the inspected caller/lifetime/error/fallback design. No new concrete memory-safety or decoding-correctness defect established. Validation remains PARTIAL, not a completed integration/B5/CI pass.** No new G2/G3/G8 stop. Existing admission, default-performance, owner-exception and exact-head evidence requirements remain. This is not permission to select the default in production, merge, release, run the workstation or alter a gate.

## Source correspondence and scope

Fetched full source separately at the reviewed head and a8a23c11607864cb431f87fff38169e10c41685f. src/cells.rs differs in exactly three documentation lines (1,7,25); executable text, assertions and line count match. SHA256 old **3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee**, new **1221a9a5f0993daee73d8352fc559b11557ebd4929f16de72bf8eeb50e5f0b58**. src/cells_original_test.rs and src/cells_selective_test.rs are byte-identical.

Read Researcher's original-reviewer H1 closure6857443323 and immutable report at e1500ba4539924e0e8407f11a90f3d93c4217318. Carry that explicitly scoped native-kernel acceptance via the correspondence above; do not relabel it a review of these callers. Its three mutation kills are retained-payload/representation assertion kills, not three output-byte kills. I did not repeat that completed algorithm/mutation campaign.

The base-to-head changed-file inventory excludes src/parallel.rs, src/coop.rs, src/run_fed.rs, src/python.rs, Cargo files and ordinary benchmark harness/workflow. The new module/re-export and run-record accessor connect the existing kernel; the three default type selections are the consequential caller changes. A fresh PR read still named the pinned head. No certification follows a later head.

## Actual reach and lifetimes

- src/read.rs:46 selects Scratch<Cells> only inside the already boxed Seq. new delegates to with_kernels, whose fresh owned scratch, scanner, output and indices are initialized at lines59–88. Par remains separate; parallel/parallel_with/planned construct StreamDecoder even with one worker.
- src/lib.rs:457 selects decode_with<Cells>. The generic body at496 owns one scratch for the whole call, including concatenated streams. No input/output pointer escapes that call.
- src/lib.rs:548 selects Cells for decode_blocks. Each invocation owns fresh scratch; blocks inside that range reuse it. Separate concurrent calls do not share Cells state; the caller supplies each mutable output under Rust's aliasing rules. No internal worker is introduced.
- Scratch's omitted parameter remains Segmented (lib.rs:280); explicit decode_with<Segmented> remains the reference. decode_parallel uses the existing explicit parallel route.
- Python's unchanged threads_or_all (python.rs:24) resolves None from available_parallelism and clamps to at least1; explicit0 becomes1, not “all CPUs.” Resolved1 selects BzReader::new/decode; other counts select parallel paths. Thus concurrent Python single-thread callers also incur independent Cells footprints. No new global/shared instrumentation or unsafe Send/Sync implementation appears.

CellsKernel owns Box<State>, including both fallback and kernel vectors (cells.rs:27–43). Offsets, not retained cross-call raw pointers, describe its arena. The writer derives pointers after sizing output/positions; mutations occur through exclusive self/output borrows. Reading the source found no new alias, dangling reference or unsynchronised shared state at these integration boundaries. This is source reasoning, not an assertion of exhaustive concurrency testing.

## Error, refusal and reuse boundaries

decode_block's shared front checks origin before inversion, retains the run/column routing and invokes the same finish function (lib.rs:367–447). Normal Cells only attempts induction on the run route; its ordinary column in-place route delegates to Segmented (cells.rs:127–150). Density/floor/budget refusal delegates to the original run fallback rather than becoming a scanner error or escaping as false just because admission refused.

The safe Cells boundary validates sizing, tiling and primary row before unsafe induction; refusal precedes writing caller output (cells.rs:317 onward). Reused layout/positions are reinitialized on the next admitted block; fallback retains its own storage. finish rolls back the current block's appended output on RLE/CRC failure. The sequential reader retains earlier delivered blocks, latches decoding failure, and does not publish a failed current block (read.rs:270 onward). decode's Result hides its local partial vector on error. Range decoding preserves its existing prefix/error convention and block/bit attribution; range overshoot/truncation checks remain before decode_block (lib.rs:558–583). Concatenated stream handling remains the existing scanner's responsibility. No evidence of a new error-identity or CRC bypass was found.

## Resource accounting is not an RSS exemption

At cells.rs:409–436 the requested arena is **48R+80B+4528+4G** bytes; positions add at most4B (cells.rs:733–739), B=floor(4n/25), G64 debug/Miri or0 release. Old arena/position allocation is dropped before replacement; growing calls do not deliberately retain two generations of that same vector. These are allocation-request bounds, excluding allocator overhead.

For reuse, use **max over prior arena requests +4×max prior position count**, not the current block's R/B and not a claim that both maxima occur on one block. Density admission requires10R<n and decoder n<=900000: a conservative release retained Cells request bound is about16.42MB per scratch. Segmented storage can coexist after a refusal or column block; outer ll/rle/runs, scanner, decoded output, allocator costs and other callers are additional. A call returning/dropping its scratch releases its ownership; a live reader retains high-water storage. There is no new process-wide cap, and m concurrent callers can carry roughly m independent scratch costs. The documented RSS cost is material, but is not by itself an unbounded-growth or memory-safety finding.

## Evidence actually available and coverage gaps

The bounded check snapshot associated with the exact head had fuzz-smoke, occupancy, walk-stats, Python versions and platform build jobs successful; **rust, miri and bench were in progress**. These are not promoted to PASS. No CI polling or rerun was used.

Read full completed fuzz-smoke job132260143096, run43534513410. Checkout log names merge739e56f99c9190aea515065d5f03ab45826b0fab, merging this exact head into the stated base. Forced Cells step ran seed42 for30s and reports **34,869 cases**, production admitted3429/density5078/floor835/budget2524/invalid0/short_cycles2772; forced admitted18419/short_cycles12959/invalid0. Source examples/fuzz.rs:717–765 asserts admission/refusal and forced reach and compares via tests/common/cells.rs. This is scanner/decode_block differential coverage, **not fuzzing of every BzReader/decode_blocks wrapper**, and the counts are not proof of all unsafe branches.

New read.rs:449 Miri test asserts actual BzReader Cells admission and short-cycle count1 using seven-byte reads; decode byte parity is also checked. Workflow discovery greps that test and six kernel Miri names before execution. Discovery guards are useful, but no completed Miri log was available in this review; selecting tests is not executing them.

tests/cells_default.rs exercises actual decode and chunked BzReader against Segmented for admitted/refused witnesses, periods/origins, CRC failure after a good block, truncation and mixed concatenated input. The new decode_blocks calls are inside **if let Ok(want)=reference** (line34): those cover successful ranges only. The ignored seven-corpus public-path test is opt-in; ordinary suite discovery alone does not prove its execution. I have not verified a completed exact-head log for that ignored test or the new public-default native suite. The author's earlier native reach/Clippy claims remain attributed claims, not my independently observed passes.

**Coverage recommendation, not a new stop:** cheaply extend the range test with a nonempty output prefix, a good block followed by bad CRC, a truncated/overshooting range, and exact retained-prefix/error assertions. One mixed admitted→refused→admitted range can check reuse in the same invocation. Add a wrapper-selection assertion for decode/decode_blocks if mutation-resistant default-selection coverage is intended: byte parity alone would survive reverting either wrapper to Segmented. These are precise limitations of the new tests, not demonstrated source failures or grounds to duplicate the completed kernel review.

## Disposition boundaries

Expected: reuse proven kernel, inspect new public boundaries. Observed: owned-state/type-only integration with unchanged rollback/fallback paths; exact kernel correspondence and completed forced-route fuzz; public-wrapper native/Miri completion still missing at the snapshot. Source review gap closed; execution evidence is not declared closed. Coordinator can reconcile existing exact-head evidence when produced without treating another routine Guardian approval as a prerequisite; concrete safety failures still require resolution.

Keep matched i9 pattern wall+43.5%, WikiRSS+31.1%, patternRSS+15.1%, historical #380 default4t failure and its unresolved cause, current6775049636 gate and applicable owner exceptions. Bench completion alone cannot waive them. This review neither changes those rules nor certifies actual public-default i9 performance. No benchmark, workstation execution, implementation, merge, queue/tag/Run signal, or task configuration change performed.

Migration old head and manifest verified; prior history preserved. Routing audit since6867883717 found no new misroutes; REVIEW routes to145. One report and REVIEW for this checkpoint.
