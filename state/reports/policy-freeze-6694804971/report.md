# Policy freeze re-review
Assignment [6694804971](https://github.com/lmmx/mibz2/pull/162#issuecomment-6694804971); candidate **7c341a875049f8c95d27448046eb4923387687cf**.
**HOLD remains.** H1's reproduced malformed-output/mixed-baseline defects are fixed. Retry wording and performance disclosures are substantially resolved. H2 is incomplete, and the added census cannot be supplied by the old-baseline execution path at this pin.

## Evidence and checks
Read the candidate's bench/policy_ab.py, bench/test_policy_ab.py, bench/three-team-policy.frozen, .just/bench.just, registration, examples/ab.rs, src/read.rs, src/parallel.rs and corpus builder/spec. Read the Coordinator's [fix log](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694798888), but checked its claims against code. PR #234's base was 818d521c6d0d5da09b4b0f34322baaf54cbf39af; read that revision's ab.rs, read.rs and parallel.rs too. The future frozen base is still unset, so these findings identify the currently proposed baseline, not an invented final pin.

Ran all 11 test functions from the pinned test module directly, supplying a temporary directory for the CLI test and None for the unused monkeypatch fixture; all passed. pytest is unavailable in this environment, so this is not a claim that pytest itself ran. No decoder, benchmark or timing was run.

Replayed old counterexamples: missing output fields now returns valid=false/pass=false; mixed baseline identifiers likewise fail. Removed thread_names only from baseline rows of the valid synthetic grid: **140 malformed rows, valid=false/pass=false**. The clean fixture passes because it invents census fields for both arms; it does not test the actual old-binary boundary.

## Remaining blocker: the census does not cross the old-binary boundary

The LOG says the base process is sampled externally. That is not what run() does at this pin:
1. For mibz2@BASEAB it selects BASEAB as exe.
2. It executes that executable as child mibz2 T FILE with cmd.output().
3. It incorporates the child's JSON into the output row without adding thread_names.

At base 818d521c, examples/ab.rs has no thread_names field. Its child therefore cannot emit one. The new field exists in the candidate's child()/Threads::finish(), but the candidate child is not the program run for the base arm.

piped() does sample an external process, but it is selected through program() for external decoder commands such as lbzip2, bz2ng and indexed_bzip2. It is not the @BASEAB branch of run(). The new diff has no parent-side census collection around cmd.output().

Thus this recipe cannot produce a valid complete baseline grid against that old harness. Replacing an invalid attempt cannot fix a deterministic protocol mismatch. Repair and test this boundary before asking Louis to run it.

Minimum acceptance evidence: a harness-level fixture with an old-format baseline child lacking thread_names, proving the declared collection path obtains actual observed names and preserves the child's wall/CPU/RSS/hash fields. No need for a performance run. Do not synthesize a census from the expected plan or copy thread_cpu into it: that would lose the new idle-thread check. Do not quietly substitute a new decoder baseline merely to gain the field. Any measurement wrapper added here must disclose whether it runs on both arms and what overhead it introduces; parent timing must not replace the child's measured wall.

## H2: good pin structure, incomplete identity checks

The unset pin file correctly blocks execution, and candidate/source equality plus clean tracked base-worktree checks remove much of the earlier moving-baseline risk. Leaving pins unset until reviews finish is correct, not a defect.

Two gaps remain:
- Both “clean” checks use --untracked-files=no. In an isolated Git fixture I added an untracked .cargo/config.toml containing build rustflags. The recipe's exact status command returned an empty string, while full status showed the file. This input can affect cargo build without changing any tracked candidate file. Same issue applies to the reused base worktree; an untracked build.rs is another possible build input.
- The measured allowlist excludes **bench/wikidata.json and bench/wikidata.py**, although the recipe executes both to construct the workload, and excludes other executed supporting files such as bench/bundle.sh. A later clean HEAD changing the corpus specification/builder alone satisfies the current candidate comparison while changing the workload under the same “wikidata” basename. Corpus self-checks against a newly changed specification do not establish the original registered corpus.

Close the identity boundary for actual build/workload inputs. A clean isolated checkout of the exact candidate, with a separately recorded freeze manifest, is one solution; a complete explicit input comparison plus untracked/ignored build-configuration checks is another. Preserve exact base/candidate pins and archive toolchain/flags/corpus provenance. No requirement to reject harmless generated output directories; the point is inputs used by Cargo and the recipe. Retain both attempt slots at one identified experiment, not merely one recipe name.

## Window derivation is sufficient, but correct the prose

I accept deriving the default window from pinned source and actual arm arguments; an added runtime window counter is unnecessary here.

For requested t in this grid, slots=4t from BzReader::parallel(), and Teams window=max(floor(slots × teams / t), teams). Therefore:
- 4×2: **16 blocks**, at every registered count;
- 3×3: **12 blocks**, at every registered count.

Both read.rs and window() at candidate and inspected base give this result. The registration's “4 a team times the threads over 8” is misleading if read as growing with requested threads. Replace it with “16 blocks for 4×2, 12 for 3×3”. This is a textual correction, not a changed estimand.

The comparison remains head default versus base default at the same request, with their respective default windows. Different 16/12 windows are part of that intended treatment, not a confound requiring matching them away.

## Census and estimand

Requiring both busy-list and census agreement strengthens eligibility; it does not replace the default-versus-default estimand or relax the existing busy-thread predicate. In the candidate child, name sorting/joining happens in finish() after wall has been read. It does not add another sampling loop to that child's measured decode. This observation does not establish zero CPU/instrumentation effect; CPU is read afterward.

“Full census” means all threads the sampler saw, not a proof that a periodic sampler sees every possible short-lived thread. The registration's actual wording acknowledges that. The critical problem is the missing baseline path, not the mathematical acceptability of a census. Retaining the busy predicate means a genuinely idle helper still invalidates the attempt even if census sees it; that is expressly tested and should not become a discretionary reason to rerun a valid performance failure.

## Resolved items and scope
- H1's prior counterexamples rejected; numeric positivity/finiteness, corpus, stray-arm and missing-census checks added. The reported malformed-row reason is generic even when census is missing; improve its diagnostic wording when repairing the path, but that alone is not a freeze blocker.
- Retry now states two slots, at most one invalid replacement, at most three attempts, no replacement for a valid failure, all attempts retained and pass-field verdict. Accepted. An execution that aborts before producing a verdict must be recorded as an aborted attempt, not disappear from the attempt count.
- Disclosures now cover unmeasured requests, topology-equivalent CPUs, custom windows, workloads/contention, non-equivalence controls and point-versus-confidence claims. Accepted in substance. “Normal log-ratio” should also say independence of paired rounds for the nominal t model; no interval recalculation requested.
- Intervals, thresholds and policy choice were out of scope and were not reopened.

Expected repair: freeze a reproducible comparison and validate both arms' plan evidence. Observed: row-level defects closed, but end-to-end census transport absent and build/workload identity still porous. These are pre-run blockers, not evidence against the policy's speed. The highest-value next step is a boundary test and input-identity repair, then freeze; no extra performance experiment is warranted by this re-review.

Migration source head and manifest digest matched. Fresh channel/inbox audit found no unresolved misroutes; REVIEW route is #145. No implementation, timing, merge, authority, instruction-pin or automation changes.
