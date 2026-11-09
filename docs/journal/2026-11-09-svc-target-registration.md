# The service core's target preflight, alone: registration

9 November 2026. Registered before any run, as an untimed diagnostic. It answers the
Researcher's question 5 (REVIEW 6742157313 on #145): before a timed slot is spent, read the
service core's target predicate and dispatch back on the i9 once, so that a predicate that is
false there (an unreadable or namespaced cgroup, a fact read differently than expected)
shows up as a cheap, archived diagnostic, not as a refused timed attempt. It times no
decoder, runs no `ab` grid, and is no performance evidence. It freezes when the Coordinator
sets `bench/svc-target.frozen` (the telemetry check's pattern); until then the recipe
refuses.

**Revision (the Researcher's narrow REVISE, REVIEW 6743780326 on #145; report and probes at
`state/reports/svc-target-preflight-6742323424/` on 05979b91):** the record checker is strict
(six malformed records it accepted as TARGET are now INVALID RECORD); a failed decode or
dispatch anywhere is FAILED, never hidden under NOT TARGET; `raw/` keeps the per-CPU, cache and
cgroup-level reads with their errors, and its claim is narrowed; `stages.txt` says which stages
an attempt reached; the timeout's termination is bounded (KILL after TERM) and stated.

## The objective

One question: **on the workstation as Louis runs it, does `parallel::service_target` hold at
10 threads and not at 9 or 11, from facts it could establish, and does the default reader
(`BzReader::parallel`) then build svc2 at 10 and not at 9 or 11?** And whatever the answer,
what did discovery report, and what do the files it reads hold (read again beside it)?

## The run

`just submit bench-svc-target`, once, at the frozen commit:
- **refuses** (REFUSED, exit 1) if the frozen file names no commit, the checkout has
  uncommitted changes or untracked measured inputs, the frozen commit is missing, or HEAD's
  measured inputs differ from it (`src`, Cargo's files and configuration, `examples`,
  `tests/common`, `bench/svc_target_check.py`, the bundler, the recipe and this file);
- **builds** `examples/svc_target.rs` (release) and runs the executable Cargo reports,
  recording its path, its SHA-256, the toolchain with rustc's host triple, and `uname`
  (`build.txt`, `inputs.txt`: the build's platform is the provenance for the predicate's
  compile-time x86-64 Linux condition, which the record's facts alone don't show), and checks
  the digest again after the run;
- **records** the files discovery reads, read again by the recipe's shell just before the
  preflight (`raw/reads.txt`, each value or its read error, and `raw/lscpu.txt`):
  `/proc/self/status`, `/proc/self/cgroup`, `/proc/self/mountinfo`, `/proc/cpuinfo`, sysfs's
  `online` and `present`, every CPU's `topology/core_id`, `topology/physical_package_id`,
  `cache/index3/level` and `cache/index3/size`, and every cgroup v2 level's `cpu.max` and
  `cgroup.type` from the process's cgroup up to the mount point. **These are the same paths,
  not the same reads**: diagnostics beside the record (which is what is checked), for
  diagnosing an Unknown or a mismatch, not a replay of discovery's own reads;
- **runs** `svc_target` once, sent TERM at 120 s and KILL 10 s later (so at most 130 s; a
  timeout is INCOMPLETE either way, said which): its record (`preflight.json`, schema
  `mibz2.svc_target/1`) holds the kernels, `Machine::here()`'s facts (the plan policy's and
  the predicate's topology snapshot and quota, each structured, `null` or `unknown` where not
  established), and at 9, 10 and 11 threads the predicate, `Plan::for_threads`, and what
  `BzReader::parallel` built (service or not, pipelined, decoding threads, window, the service
  counts after a join), and whether a small decode gave back its input;
- **checks the record whole** (`bench/svc_target_check.py`, `preflight-check.json`): the
  schema and every field strictly (present, of its type, integers never floats or booleans,
  numbers finite, a quota finite and positive, plans complete for their kind, sizes positive,
  counts nonnegative); the predicate recomputed from the record's own facts and
  compared with the binary's answers; the dispatch against the predicate; the exit status
  against `ok`;
- **bundles** the directory (`bench/bundle.sh`) however it ends, its outcome in `outcome.txt`
  and in `stages.txt` every stage, reached or not (the checks, the build, the raw reads, the
  preflight's run, the record's check). **An attempt keeps the files of the stages it reached
  and no others**: an early refusal has no build, raw reads or record, a crashed preflight no
  checked record, and none is made up.

## The interpretation (every outcome, fixed now)

| outcome | exit | means |
|---|---|---|
| TARGET | 0 | the facts are the target's and complete; the predicate holds at 10 only; svc2 was built at 10 (8 threads, window 8, completed) and not at 9 or 11; every decode correct |
| NOT TARGET | 4 | the facts, as established, are not the target's, an unknown quota or unread snapshot among them (fail-closed); the record is consistent and clean: the service core built nowhere, one decoder at each budget, every decode correct. The timed A/B cannot run here as frozen; the archived facts say why |
| FAILED | 5 | a dispatch or decode check failed: on the target's facts, svc2 not as expected at 10; anywhere (off the target too), a budget without exactly one decoder built, or a decode that did not give back its input. **It takes precedence over NOT TARGET**: a defect to investigate, never hidden under a machine mismatch |
| INVALID RECORD | 1 | the record is malformed or contradicts itself (an `ok` beside an unknown quota, a predicate answer that disagrees with the facts, the service core where the predicate is false, an exit status that disagrees) |
| REFUSED | 1 | nothing was run (the refusals above) |
| INCOMPLETE | 3 | the preflight crashed, ended on TERM at 120 s or KILL at 130 s, or its executable changed |

`Unknown` stays fail-closed: NOT TARGET, never TARGET. **A TARGET outcome does not qualify
anything**: it shows the facts at one moment (affinity, quota and hotplug can change later),
on this one machine, and it does not replace the timed A/B's own preflight, which runs again
before any measurement. It says nothing about speed, and nothing about the predicate's other
admitted machines.

## Bounds

One run, about a minute once built (the build's time is outside the bound); the preflight
itself is sent TERM at 120 s and KILL 10 s later. Read-only: it
changes no setting. **No retry-to-pass:** whatever the outcome, it is reported as it is. A
REFUSED or INCOMPLETE attempt (nothing judged) may be run again only after its cause is fixed
and stated; a NOT TARGET, FAILED or INVALID RECORD outcome is an answer, and any change
it leads to (to discovery, or to the target) is a source change and a new freeze.

## Checks of the recipe itself (CI)

`bench/test_svc_target.py` runs the recipe end to end with stand-ins for Cargo, the
preflight, `lscpu`, `gh` and `timeout`, every exit path through the bundle (TARGET, NOT
TARGET, FAILED, INVALID RECORD, INCOMPLETE by crash, by TERM and by KILL, REFUSED by dirty
inputs, an unfrozen file and changed inputs), each with its `stages.txt` (an early refusal's
bundle holding no later stage's files), and through `just submit`; and the record checker on
its witnesses, the Researcher's six (an infinite quota, the same through `1e309`, an
incomplete plan, a float budget, a float service-plan field, a negative count) and a failed
fallback decode off the target among them (`bench/test_svc_target_check.py`). Both run in CI's submission-tests job.

**The real path, exercised once (not evidence), at 7f691929, before this revision.** On the
shared 4-core machine (Intel family
6 model 173; not the target), in a throwaway clone with a pointer-only freeze at the
recipe's commit (never pushed): the real release build of `svc_target`, Cargo's reported
executable and its digest, the raw files, the record and its check, and the bundle.
Outcome NOT TARGET, exit 4, as this machine's facts say (4 cores; quota established as
unlimited). It shows the recipe runs past its refusals; it says nothing about the i9.
