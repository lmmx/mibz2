# Deliberate integration, 12 November 2026

Direct owner instruction following Coordinator6830454082 and Codex6830378631.
Codex performed the bounded integration explicitly requested by the owner; the
Acting Coordinator retains orchestration. No standing role, monitor setting,
other-role cursor or assignment was changed. No release or workstation run.

## Completed integration

**#366 merged once** using the connector's supported
`expected_head_sha=d61e0976b6faa8e8b1dfdcb0e10bffdc28235108` and merge method
`merge`. Resulting master is **d50489823f3c719afe8ea001b7d966275467a6c6**.
Parents are2635f04e9155dd31388b24fd3f7248f87b3ddcda and the exact checked366 head.
Merged treef432dd0b55edd7d8bcfe05f434144b6aa1732ceb is identical to that head's tree.
GitHub independently read back merged=true, exact original head and merge SHA.

Before merging: fresh all29 check-run inventory had26 successes and3 intentional
skips (release, crate publication and branch deployment), seven successful Actions
workflows and successful legacy pre-commit. Four real-Just1.38 untimed recipe tests
passed with no skips. Source correspondence again verified41 reviewed368 files;
the two matching cherry-pick patch IDs are preserved in the prior reconciliation.
Benchmark/Miri workflows and workstation queue were unchanged. Reviewed source
diff has no new unsafe operation or default change; history collection is confined
to trace/test builds. Ordinary-worker and team accounting assertions remain real.

Guardian D1036790889882 at368/bfd5d322651974f9803f4c03942cf3ba3b5161d2 independently
lifted the accounting restriction. That review is reused by exact correspondence,
not self-certified anew. The final diff and original bench were assessed under
OWNERSHIP and Coordinator6820358956/6830454082's diagnostic/accounting disposition.
**This is not optimisation ACCEPT.**366 random2t+3.3%[0.5,6.1] and maximum RSS+4.8%
remain adverse; primary intervals include zero and all hashes match.368's earlier
records2t+3.8%[0.2,7.5]/RSSmax+2.8% and361's adverse finding remain unchanged.

**#368 closed as incorporated**, with comment6830813882, not merged separately.
Its41 files are already in master through366. No double application or false claim
of original368-commit ancestry. Announced completed integration in6830847624.

Postmerge Test, Fingerprint, Documentation site and Benchmark links have passed.
The push-triggered Miri and Benchmark runs are still in progress at the archived
snapshot; packaging is intentionally skipped on this master push. No manual
benchmark dispatch/retry was made. These pending checks are not called passes.
This trace-accounting/diagnostic integration establishes no production speedup or
release qualification; no release workflow was started.

## #373 integration work completed; merge remains blocked

Appended master via merge commit **ed6f424f238851ff8296e7a48c2b96dfb8332e7e**, pushed
normally to `diagnostic/cells-live-path`. Its parents are accepted
b5131679f51c2a0245bc03f62b7afd833b8bff23 and integrated
d50489823f3c719afe8ea001b7d966275467a6c6. No rebase or force-push.

The sole conflict was the two additions at the end of `.just/bench.just`.
Both entire appended stanzas survive byte-for-byte, exactly once; no recipe body
changed. Root source and Occupancy workflow now exactly match integrated master.
All41 repair files match; all133 protected live/old harness and12November evidence
files match accepted373, including R1, PLAN, estimator, source generator and tests.
Generated measured source remains d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2; no root
source substitution. PLAN SHA256 remains
2c285d842413602379a2c3ceb3de4d8e5e50954f06a60a5d23270e3ad9f0b2a0.

Actual fresh-checkout untimed validation at ed6f424:

- Corrected source materialised:53 original files; existing additive hooks only.
- Onecore4/4 and live Python13/13, real Just1.38.0, zero skips; both recipe dry runs.
  The printed CPU0 and dummy path were syntax-test placeholders, not approved
  workstation values. Existing failure wiring tests use stand-ins/fake resources.
- Native live harness5/5 and kernel7/7, including both F01 long-factor witnesses.
  No new Miri round: accepted safety evidence and original unsafe operations stay
  unchanged. Master's normal push Miri is independently pending, as above.
- Four Rust1.99.1 release builds, exact source/compiler/binary manifests retained.
- Real fixture dry run:16 valid children and12 expected failure children;
  full byte parity and error parity, zero clocks read, zero performance measurements.
- Clean checkout `just pc-ci` passed (Taplo0.9.3, whitespace check1358 unchanged
  files, cargo-machete, Rust1.99.1 fmt). Tracked-Python debug-statements hook passed.

Preparation43269375801, Test43269375554, Occupancy43269375729 and
Benchmark43269375558 all succeed at the closing17:33UTC snapshot. Occupancy
job131361477973 passed instrument tests, ordinary-worker accounting, teams-of-two
accounting and artifact upload. The inherited dependency is now cleared by actual
new-head CI. Earlier in-progress observations remain in github-snapshot.json;
closing-ci.json records completion. No373 merge was attempted.

New benchmark against integrated master has every wall interval including zero
and all output hashes equal. Preserve the unresolved point estimates, including
Wikidata4t+3.5%[-2.2,9.5], pattern2t+5.1%[-4.7,15.8] and
onebyte4t+5.2%[-9.9,22.9]; no optimisation gain or retry follows. Full report is
benchmark373.md. This does not erase366/368's earlier adverse findings.

## Remaining CI failures, without guessed causes

Old373 Occupancy43207829422 was a genuine project assertion on inherited ledger
bounds. Reviewed accounting integration now carries the per-worker histories,
finite bounds, ordinary assertion and failure artifact preservation unchanged.

Old macOS jobs131147445829 and131147445942 were cancelled with no steps. Decoded
logs were404 BlobNotFound; check annotation endpoint is unsupported by the connector.
They remain Class D, not demonstrated infrastructure failures. On new head,
packaging43269375621's sdist, Linux x86_64/aarch64 and Windows jobs pass; macOS
jobs131361540182 and131361540210 are queued at this snapshot, not validated.
No manual retry was used; new runs follow the substantive integration commit.

**New legacy pre-commit is FAILED**, result
1794504358.Y40p1JvNSESWR89bgRGktA. Old result
1794482231.FWhSnWB4TjeLnaE7eIXJmg remains failed too. GitHub only says “checks
completed with failures”; no bot failure comment supplies the cause. Results site
is blocked by the execution proxy; web fallback also cannot open it. Class D.
Local clean `pc-ci` success is not a substitute for that hosted job. Its CI branch
runs code-quality, not the local Ruff-formatting recipe, so the historical Ruff
format failures alone cannot explain the legacy failure.

One local reproduction initially hit an unformatted ignored generated `.source`
Cargo.toml in the existing development worktree. A clean tracked checkout without
generated material passed. Both logs are retained; that local artifact is not
asserted as the hosted cause. Likewise the first local uv invocation failed because
its default tool directory was read-only; using the existing writable caches ran
the actual no-skip tests. No tracked files were reformatted to manufacture green CI.

## Workstation readiness: explicitly deferred configuration

The owner clarified during this run that **neither CPU nor corpus directory is
verified or approved**. Do not infer either from historical topology or paths.
Missing bindings block final Cells queue admission/execution only; they do not
block independent integration, CI reconciliation or documentation. Do not request
them again prematurely in this integration lane.

At the eventual separately authorised i9 preflight:

1. Read current topology, affinity and permitted CPU set on the actual workstation;
   explicitly select/approve one logical CPU. No selection from archived0/10..9/19.
2. Verify the absolute corpus directory and identities of wikidata.bz2, pattern.bz2
   and onebyte.bz2 against the frozen PLAN; record source/compiler/input identities.
3. Freeze an argument-free queue recipe binding those concrete values to the
   unchanged parameterised recipe; verify wiring untimed and fail closed on invalid
   bindings. Do not rewrite the generic queue or invent default/environment values.
4. Check exact final recipe/run SHA, clean source, integrated dependencies, all
   required Actions+legacy, accepted review and Coordinator frozen-recipe check.
5. Only the separately authorised PROTOCOL9 procedure may then add a master queue
   entry and set its exact ready tag/Run signal. None is created in this task.

Programme stays120 measured decodes+9 untimed/oracle checks, three corpora, ten
balanced pairs, four trace-off/on builds, one pinned CPU and0.25s settle. Warm20+20
remains unadmitted; no current or historical timing is relabelled. Historical
i9-10900K topology is evidence about its old run, not current configuration.

## Handoff and single next action

**Obtain the actual legacy pre-commit failure output for ed6f424 and classify it.**
That is the next concrete integration blocker. Preserve the failed run; make only
an evidenced correction or proportionate CI-INCIDENTS recovery. Do not rerun the
benchmark to chase green. Pending macOS packaging and master postmerge checks
must finish and be read before further merge/readiness claims. No additional
Codex conflict-resolution assignment is needed: that mechanical work is delivered.

#374 remains wholly separate; this run neither reopens its delivered assignment
nor blocks the Coordinator's authorised preview/publication procedure.

Direct urllib coord.py transport still gets proxy403. To retain the checks, the
unchanged repository validation functions ran with authenticated connector responses
as their read transport: audit passed (six resolved historical findings), exact
head check passed, wrong/prefix heads rejected, and postmerge --merged passed.
No validation rule was changed or waived. The merge API's expected_head_sha
enforces the atomic head check on GitHub. Other direct transport failures remain
access limitations, not successful native HTTP checks.

Completion evidence stays on Codex's own state branch; no fabricated inbox completion
ID. Acting Coordinator remains the next orchestrator. GitHub does not restart this
Codex session; any further delegated session still needs a manual launch.
