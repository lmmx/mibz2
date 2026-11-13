# Bounded source/build finding — queued6857735998

13 November 2026; reconciled into the owner-prioritised queue-preparation lane by
Coordinator6862221345. One source pass; no production edit, new timings or rerun.
Pins: master3e7e294cc10f29495e9828181345d74233c549d4,
common original-Cells basebe25366f46458e2f1dbb0bdac1f5fbc3366d0648,
selectivea8a23c11607864cb431f87fff38169e10c41685f.

`git diff 3e7e294 be25366 -- src` changes exactly four files: new cells.rs;
bwt.rs adds only the Cells reexport; lib.rs adds the module and the Cells-specific
Scratch counts accessor; runs.rs adds a read-only record accessor. Existing
Segmented/default-reader/parallel/team function bodies and structures are
unchanged. Cargo.lock and build.rs are identical. Cargo.toml only adds the known
benchmark cfg to lint declarations. The harness's default build excludes the
added Cells reader module, parallel classification and decode arm via that cfg.

`git diff be25366 a8a23c1 -- src` changes only cells.rs and two test-only include
files. The sole non-test execution change is the selective-materialisation
predicate-before-write branch in Cells::induce_cells. MARK/writer/admission and
Segmented paths are unchanged. Test tracing has cfg(test) guards.

Thus no newly executed default-route source operation was identified. This does
not establish neutral machine code: adding a module/type can change codegen,
monomorphisation placement or link layout without changing reachable function
bodies. Existing default .text differences are evidence of nonidentical binaries,
not a causal explanation of the 4t Wikidata/text/random regressions. Existing
CI compared master with the whole inherited365+selective change; it cannot isolate
selective materialisation. Failed gate6775049636 remains failed.

No justified native correction follows this source pass. The smallest decisive
follow-up, if Coordinator admits it, is a matched default-only three-source
comparison (master/common365/selective), identical compiler/build/harness and
paired ordinary CI on the already flagged corpora/thread count, coupled to
symbol-level disassembly. Do not repeat the old head/base measurement to seek a
pass. No such timing is run or newly authorised here. The present i9 job compares
three explicit reader routes from one common surrounding tree and answers a
different question; it cannot clear the default-path integration gate.

Disposition: bounded source finding complete; default performance causality and
integration remain unresolved. This does not block diagnostic preparation.
