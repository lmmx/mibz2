# Selective Cells materialisation — frozen 13 November 2026

Assignment6856176834, owner relay6856026045, actual pickup6856423064.
One native candidate: move the EXACT `len <= SMALL` predicate, including MARK,
before the two alternative writes. Short unmarked: only original two copy16 calls;
long/marked: only original two wr64 calls. All other production operations remain.

Common integrated sourcebe25366f46458e2f1dbb0bdac1f5fbc3366d0648 is the existing
merge of master3e7e294cc10f29495e9828181345d74233c549d4 and corrected365
d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2. src/cells.rs and its writer match365.
Neither379's guarded writer nor equal-unit forwarding enters either arm.
The original comparison build uses the final experiment tree/harness with ONLY
src/cells.rs restored byte-exactly from365. Pin both source hashes and executable
hashes; identical compiler/release flags, dependencies and input generator.

Before timing: compare per-cell live descriptors/frontier state/selected payloads,
root, initialized live slots, exact output and refusals with the original induction.
A test-only copy of the original function has only its name and snapshot hook
changed; verify this mechanically. Native exhaustive binary columns n<=8/all
origins; ternary n<=6/all origins; fixed word-length15/16/17 and marked short/long
witnesses; original F01 n55/n550; equal/unequal transitions; budgets0/B-1/B;
periodic/arbitrary multiple cycles; large-small-large reuse with independently
poisoned unused slack. Existing malformed/CRC/libbzip2/concatenation tests stay.
Run native suite, full Clippy/fmt, targeted Miri, seed42 forced-route fuzz30s,
original mutation suite and three selected-write mutants (missing bytes, missing
rule, masked MARK predicate). Preserve failed evidence. Independent B5 required.

Ordinary CI qualification, once correctness holds: retain default head/base arms,
threads, schedule and gates unchanged. Separate experimental run: three complete
readers (Segmented, original corrected Cells, selective Cells), one thread, ten
paired rounds, all seven pinned families,210 measured decodes. Use existing ab
arm ordering, settling0.25, paired log-t95% intervals and native getrusage RSS.
No instrumentation/timing hooks. Pinned generator/text57f51bbc0ca62ac3ee1cc148a1010842eea6222a,
scale4, libbzip2 1.0.8; exact seven compressed hashes checked before execution.
Full-byte libbzip2 parity for both Cells builds and Segmented precedes timings.
Record route counts outside measurements; validate native peak-RSS apparatus with
existing1MiB/64MiB touched-memory controls. Inspect both optimized induction bodies
and code sizes; source-addressed writes are not DRAM traffic or runtime evidence.
No separate kernel timing is admitted; codegen/kernel operation diagnostics are
reported separately from the decisive whole-reader result.

Decision: candidate must beat ORIGINAL Cells on a primary beyond noise with no
primary loss and policy-permitted secondary/memory tradeoffs. Segmented reference
and frozen slice-one/default gates remain separate adoption obligations. A kernel
proxy alone cannot qualify it. One CI result decides; no secondary retry-to-pass.
On neutral/negative stop this variant and propose, but do not build, one bounded
long-factor output-scheduling/reuse falsifier through Coordinator/MONODROMY/Pathfinder.
No i9/workstation run, queue/tag, default/API/team/gate change, merge or release.
