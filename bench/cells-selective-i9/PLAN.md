# Selective Cells on i9 — prospective diagnostic, 13 November 2026

Owner assignment archived at team-state/codex commit
6100e6600a757839b992c119a504f90a8f901b38, intake REVIEW6862129816.
Recipe: `bench-cells-selective-i9`, branch `bench/cells-selective-i9`.
This registration admits no execution by Codex. Coordinator must approve the
exact green head, use Run Signal, and admit it under PROTOCOL §9. Louis then runs
only `just workstation-queue`. No ready tag or queue entry is implied by this file.

## Sources and estimands

Common source/harness a8a23c11607864cb431f87fff38169e10c41685f (#380), derived
from be25366f46458e2f1dbb0bdac1f5fbc3366d0648 (master
3e7e294cc10f29495e9828181345d74233c549d4 plus corrected #365).
Two builds, three reader arms, identical Rust 1.100.0 x86_64-unknown-linux-gnu,
Cargo.lock, release profile and `--cfg mibz2_cells_bench`, no native CPU flags:

- A: `mibz2` (Segmented) in the original-Cells build; Cells is not selected.
- B: `mibz2-cells` in that same binary, with only src/cells.rs restored exactly
  from d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2.
- C: `mibz2-cells` in the unchanged selective source.

A/B share even binary layout; B/C differ only in src/cells.rs. Archive both
source trees' file digests, the sole difference, compiler/linker facts and binary
hashes. No #379 writer, new dispatch, instrumentation, kernel microbenchmark,
team comparison or production change. Benchmark uses the existing ab run once.
AB_SAMPLE_MS=0 disables child thread sampling in all arms. Parent census and
sysfs readings remain outside child decoding. No stage/perf hooks or spinner.
This differs from sampled historical CI; do not multiply its ratios by old i9
ratios or call it a reproduction of identical historical conditions.

## Inputs, preparation and machine

Seven compressed hashes and lengths are fixed in inputs.json. Generator and text
source are 57f51bbc0ca62ac3ee1cc148a1010842eea6222a, scale4, bundled libbzip2.
Freshly generate from that archive, never moving master. Wikidata uses canonical
corpora/wikidata.bz2 if hash-valid; otherwise the existing corpus.py reconstruction
from pinned windows a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a. Verify all hashes
before any decode. No manual owner paths. Untimed full-byte tests in both builds
compare Cells and Segmented with libbzip2, checking CRC and seven route summaries;
compare the route summaries exactly. Two 1/64MiB native memory controls validate
self getrusage, wait4 and /proc peaks. These are not decoder measurements.

Read-only existing share_nothing.py preflight: i9-10900K, all20 CPUs allowed/online,
unlimited CPU quota, intel_pstate/powersave/balance_performance/turbo-on, three
ab sample readings. No host configuration changes. Fail closed on missing or
mismatched facts. Select lowest currently allowed CPU, verify its current sysfs
core/socket/sibling membership, and exclude that entire physical core from
recipe-parent/census affinity. The decoder alone is pinned to the selected CPU;
no recipe process uses its SMT sibling. This is not OS-wide core reservation:
external desktop activity remains subject to the standing idle gate (total busy
<=1.5 cores, no individual CPU >0.5). Record topology, original/parent affinity,
passive ab env facts, pre/post sysfs conditions and per-run busy/steal/throttle.
Unexpected machine conditions invalidate qualification; do not retune or retry.

## Frozen schedule and accounting

12 rounds × 7 corpora × 3 arms × one thread = **252 measured complete decodes**.
Twelve is two complete six-round arm-order cycles; each arm appears in each
position four times per corpus. Existing rotate/reverse ab ordering, corpus order
wikidata, onebyte, pattern, random, records, runs, text. Existing workstation
settling2s and idle gate before every child, no warmup or discarded first round.
Preparation's full-byte oracles warm all inputs; no additional warm comparison.
One ab run invocation, a two-hour process-group deadline including idle waits;
no sample retries/replacement or result-dependent extensions. Build/preparation
has a separate two-hour deadline. Failure archives partial data and terminates.
The old120+9 and AMD210-decode registrations remain historical and unchanged.

All252 rows must match the exact order/arms/pin, successful termination, output
length/hash, finite positive wall/CPU/native RSS. Keep failed/partial rows; never
filter into a success. Existing bench/report.py paired log Student-t95% estimator,
paired by corpus/round, for B/A,C/B,C/A wall, CPU and RSS. Report absolute medians,
maximum RSS and all raw data. No per-family favourable selection.

Primary questions: does B beat A on Wikidata (interval wholly below1), does C
beat B, and what is C/A? Text is also primary. Report secondary pattern, onebyte,
random, records and runs even when adverse; onebyte budget refusal is not an
induction win. Credible further benefit requires C/B primary improvement beyond
noise, no resolved primary loss and policy-permitted secondary/memory tradeoffs.
A/B and C/A adoption costs remain separate; existing >10% secondary owner-signoff
and default-path gates stand. A 14% combined gain is a hypothesis, not a target.
Incomplete/failed apparatus, output or host checks => invalid/inconclusive, no
retry-to-pass. No i9 evidence alone waives existing default CI failures.

## Evidence and admission

One timestamped target/cells-selective-i9 attempt; exit/outcome, commands, full
logs, source/binary/input identities, route/oracle/control results, preflight,
raw252 rows, summaries and failures. Exactly one bundle through bundle.sh and
just submit; failed preparation/run still bundles, submission retry uses the
same archive and never repeats decoding. Large build caches live outside bundle.
Untimed `prepare` mode builds/stages/checks only and cannot call ab run. Unit and
stand-in tests exercise actual Just wrapper and failed/successful submission;
target preflight refusal on the cloud is expected, not i9 validation.

Before READY: exact-head Actions and hosted legacy green; Coordinator frozen
recipe approval; no prior submitted run for this recipe/head; Run Signal ready
at that head; Coordinator queue entry naming this branch/recipe; verify all agree.
After owner execution: Coordinator reconciles bundle, removes entry and clears
tags. No automatic release, integration, default change or additional programme.
Queued attribution6857735998 stays a distinct bounded source/build finding; its
unresolved integration cause does not invalidate preparing this diagnostic.
