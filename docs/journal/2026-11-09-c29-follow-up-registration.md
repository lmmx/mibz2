# 2026-11-09: C29's follow-up registration, a cgroup quota collector with coverage

**Status: FROZEN registration, NOT RUN** (RULES C1). The frozen commit is the one carrying
this line; the one timed dispatch names it as `registered_head`. It differs from the reviewed
head `0ce4dca4489c4a87b9658d6c2656290173113c99` only in this file (`git diff 0ce4dca4 <frozen>`
lists nothing else): no code, workflow, fixture or reporter changes. The code comments that
still say "DRAFT, NOT FROZEN" are left as reviewed. The first registration
(`docs/journal/2026-11-09-c29-register-checkpoints.md`, frozen at
`1197bbcced86eae74c6d99eb8096b5c816a834d7`) is untouched and stays closed. This file is its
section **R5**. It is kept apart so the frozen journal's text doesn't change.

Authority: the Researcher's REVIEW
[6735626248](https://github.com/lmmx/mibz2/issues/145#issuecomment-6735626248) (REVISE the
collector). Its report and four witnesses are at
`state/reports/c29-reregistration-6735163830/` on commit `e68846149286dbbed19aa564027adf4d7eb61d8a`.

## R5. The follow-up registration

### R5.1 The first attempt: terminal, disclosed, not reused

- **The run:** [42582708226](https://github.com/lmmx/mibz2/actions/runs/42582708226)
  (`sweep.yml`, `tool: c29`, `registered_head` `1197bbcc…`). It is **VALID, UNJUDGED
  (FALLBACK)**, and that is terminal (R4). It will never be rejudged, and its freeze will
  never be rerun to get a different verdict. Its records won't be reclassified with the
  repaired collector.
- **Its favourable ratios are disclosed:** the 9-thread wall ratio was **0.9700 [0.9647,
  0.9753]**. They were seen before this follow-up was designed. They are **never used** for
  the new verdict, and never pooled with the new run's records. Rounds won't be selected
  across attempts, and the better attempt won't be chosen. This registration's verdict comes
  from its own run's data alone.
- **The defect is in the collection, not the measurement.** The frozen `ab env` read the CPU
  quota only from the root `/sys/fs/cgroup/cpu.max`, or v1's root `cpu.cfs_quota_us`. The
  reading was unknown, and the reporter correctly refused JUDGED (R3's guard worked). On cgroup
  v2, `cpu.max` exists only on non-root cgroups, so the root's lack of one is expected, not
  a hidden limit. A root-only read was never a census of the hierarchy.
- **The runner's hierarchy is not established yet.** The root-only failure doesn't show the
  runner's mount placement, its namespace visibility, the error type, or whether a tighter
  ancestor exists. The untimed preflight (R5.4) collects those facts before any new timing.
  Until it has run, the cause won't be described more specifically.
- **The justification is interpretable evidence, not recovering a hoped-for LEAD.** This is a
  new bounded diagnostic under C1/C3. It doesn't amend C6's standing bench repeat and
  multiplicity policy, and doesn't relax the first registration. A nominal interval in the new
  run is not a multiplicity-adjusted claim over the whole adaptive programme.

### R5.2 The collector (`examples/cgroup/mod.rs`, used by `ab env`)

Coverage is tracked separately from the minimum. Let S be the observed finite levels and H the
unobserved ones. Then min(S ∪ H) ≤ min(S): the observed minimum is only an upper bound on the
hierarchy's cap. A `max` seen at one level can't prove its ancestors unlimited.

- **Membership:** from `/proc/self/cgroup` and `/proc/self/mountinfo` together, not by
  appending a path to `/sys/fs/cgroup`.
  - The hierarchy carrying the CPU controller is the v1 line whose controllers include `cpu`
    (including a combined `cpu,cpuacct`), or else the v2 line `0::PATH`. Hybrid layouts are
    recorded as hybrid.
  - The mount is a `cgroup2` mount (v2), or a `cgroup` mount whose super options carry every
    controller of that line (v1). Its root (mountinfo's field 4, octal escapes decoded) must
    be the membership path or an ancestor of it; the shortest such root is used.
  - Each level's directory is the mount point plus the path below the mount's root. A level
    whose directory resolves through another mount (an overmount) is unknown. So is a path or
    root with a `..` component, which is a process outside its namespace's root.
- **Every level, from the leaf to the mount's root, is read and recorded:** its path,
  directory, raw files, and parse result or errno.
  - **v2:** `cpu.max` must be `Q P` or `max P`, both positive integers. Otherwise the level is
    malformed, and so unknown. A missing `cpu.max` is a justified absence only when that
    level's `cgroup.controllers` is readable and lacks `cpu`: the level then has no bandwidth
    limit of its own, and its usage is charged to an ancestor on the walk. A missing
    `cpu.max` beside a `cgroup.controllers` that lists `cpu` is unknown.
  - **v1:** `cpu.cfs_quota_us` must be `-1` (unlimited, the documented sentinel) or a
    positive quota, with a positive `cpu.cfs_period_us`.
  - **Configuration and counters are kept too:** v2's `cpu.max.burst`, `cpu.weight` and
    `cpu.stat`, and v1's `cpu.cfs_burst_us`, `cpu.shares` and `cpu.stat`. Equal Q/P doesn't
    mean identical short-term throttling, so every period and burst setting is kept.
- **The boundary.** The top of the walk is the hierarchy's real root only when two things
  hold: the mount's root is `/`, and the process's cgroup namespace is the initial one,
  `cgroup:[4026531835]` (the kernel's `PROC_CGROUP_INIT_INO`). A visible `/` alone is no
  proof of host-root visibility: in a cgroup namespace, `/` is the namespace's root, and its
  ancestors are hidden.
  - At v2's real root, `cpu.max` and `cgroup.type` must both be absent, since both exist only
    on non-root cgroups. That is a justified semantic absence. Either one present there
    contradicts the claim, and the level is unknown.
  - Elsewhere the boundary is recorded with its reason (a mount-root offset, or a non-initial
    namespace). Coverage is then incomplete. The census never escapes a mount boundary or asks
    for host privileges to pass.
- **The state:**
  - `unlimited` or `finite` only with complete coverage: every level known or justified, and
    the boundary the real root.
  - The tightest limit is found by exact integer comparison: q₁/p₁ < q₂/p₂ iff q₁p₂ < q₂p₁,
    in u128.
  - Anything else is `unknown`, its gaps listed. The tightest observed limit is still
    recorded (`observed_tightest`), as an upper bound only. An observed cap below four can
    show failure even with incomplete coverage; an observed cap of four or more can never
    show success.
- **`ab env`'s line** keeps every existing field, so bench.yml's `bench/report.py` reads it as
  before. `cpu_quota` is the tightest complete reading to two decimals, else null, and
  `cpu_quota_state` is the census's state. Added at the end:
  - `cpus_allowed_ids`: the affinity mask's CPU IDs, not just their count;
  - `sched_policy` (`sched_getscheduler`);
  - `kernel` and `kernel_version`;
  - `image`: GitHub's `ImageOS` and `ImageVersion`;
  - `cgroup`: the census, schema `ab/cgroup/1`. It holds the raw membership, the cgroup
    mountinfo lines, the cgroup and mount namespace identifiers, the selected hierarchy, line
    and mount, every level, the boundary and its reason, the coverage and its gaps, the state,
    the tightest and the observed tightest limit, and the effective cpuset where it can be
    read.

### R5.3 The identity and the reporter

- **`bench/c29_identity.py`** (schema `c29_checkpoints/identity/4`) **re-derives the quota from
  the census's raw files.** It re-parses each level's `cpu.max`, `cgroup.controllers`,
  `cgroup.type` or `cfs` files, and doesn't trust the producer's summary.
  - It checks the chain: the leaf is the membership path, and each level is its predecessor's
    parent. It also checks that the last level is `/` at a real root, with the mount root `/`
    and the initial namespace.
  - Any disagreement with the producer's kinds, numbers, state or legacy `cpu_quota` is
    unknown.
  - It writes `cpu_quota` from `ab env` before the run, `cpu_quota_after` from `ab env` after
    the run, and `cgroup_stable`. `cgroup_stable` compares the configuration between the two
    readings: membership, mount lines, namespaces, hierarchy, mount, every level's limits and
    settings, the effective cpuset, and the affinity's CPU IDs. `cpu.stat`'s counters are
    left out.
- **`bench/c29_checkpoints.py`** admits JUDGED only with all of these:
  - `cpu_quota` and `cpu_quota_after` both at complete coverage, and each either unlimited or
    finite with integer Q ≥ 4P (exactly, no rounding: 1199999/300000 prints as 4.00 and is
    refused);
  - `cgroup_stable` true. Changed membership or limits is an admission failure. Endpoints
    alone don't prove nothing changed between them; they are what is recorded.
  - every R2/R3 predicate as before: AVX2, the 4-CPU allocation by `nproc`, the mask and
    `available_parallelism`, unpinned, and `ab run`'s status 0.

  Anything else is UNJUDGED (FALLBACK), exit 3, as before. `cpu.stat` and burst are recorded
  for diagnosis only. They never discard pairs and are not a new performance gate. Contention
  observations stay descriptive, with no post-hoc exclusions.
- **`preflight`** is the environment's half alone (`c29_checkpoints.py preflight`). It needs no
  records and no run status. It exits 0 if a run here would be judged, 3 if not.

**Fixtures.** Every case is the Rust collector's own output over a fake `/proc` and `/sys`
tree. They are committed in `bench/c29_fixtures/`, and `cargo test --example ab` checks that
they match (`AB_BLESS=1` rewrites them). `bench/test_c29_checkpoints.py` runs them through the
identity builder to the reporter, against a table of expected outcomes written by hand:
- **Judged, unlimited:**
  - every level `max`, with the real root lacking `cpu.max` (a runner's shape);
  - the process in the root itself;
  - a parent without the cpu controller;
  - a non-default, escaped mount point;
  - v1 `cpu,cpuacct` at `-1`.
- **Judged, finite:**
  - Q = 4P exactly (200000/50000);
  - a parent of 8 with a child of 4;
  - hybrid, with cpu on v1 at 4.
- **Refused, below four** (complete coverage):
  - nested v2, finite at a parent (2) with `max` below;
  - 399999/100000;
  - 1199999/300000;
  - v1 at 2.
- **Refused, unknown:**
  - the Researcher's four witnesses: an unreadable (EACCES) parent under a `max`, 4, 8 and
    `max` child. "Unknown only when no level parses" admits three of them; witnesses 1 and 4
    produce the identical census, the same observation in two worlds;
  - a malformed parent, a zero period, three fields;
  - `cpu.max` missing while `cpu` is enabled;
  - a cgroup namespace whose visible `/` carries a `cpu.max`;
  - a mount-root offset (`/docker/abc`);
  - a path outside the namespace (`/../sibling`);
  - an overmounted hierarchy;
  - a malformed v1 quota;
  - no membership, and malformed membership.
- **Tampering:** a census whose kinds and state are rewritten to unlimited over an EACCES file,
  a non-initial namespace, a boundary not claimed, a mount-root offset, a dropped level, a gap
  in the chain, a bool quota, or no schema is unknown.
- **Before and after:** a changed limit, an absent after-reading, or changed CPU IDs is not
  judged. Changed `cpu.stat` alone is still judged.
- **The preflight's command:** PASS and FAIL.

### R5.4 The untimed preflight (`sweep.yml`, `tool: c29-preflight`)

It builds `ab` alone, and runs no corpus and no decoder:
- `ab env` twice, 2 s apart;
- the raw `/proc/self/cgroup`, `/proc/self/mountinfo`, `/proc/self/status` and
  `/proc/self/ns`;
- `/sys/fs/cgroup`'s type and listing;
- every visited level's directory listing and every regular file's contents or error;
- the runner's image, kernel, identity, affinity and scheduling policy.

Then it runs `c29_checkpoints.py preflight`. Everything is uploaded in the run's artifact
(`c29-*`, `c29-preflight-raw/`). It exits 0 if the environment would be judged, else 3. It
does no timing. Its outcome doesn't depend on any performance figure.

The timed tool (`tool: c29`) runs the same preflight first, and runs no decoder if it fails.
After the run it reads `ab env` again, for `cpu_quota_after` and `cgroup_stable`.

### R5.5 The precommitted stopping policy

- **One new registered performance attempt,** and only after the untimed preflight has shown a
  supported layout on this runner configuration (`ubuntu-latest`, GitHub-hosted, 4 vCPUs).
- **A failed preflight is archived** with its artifact, and no decoder is run.
- **A timed attempt that is INVALID or UNJUDGED is retained,** and C29 timing on this CI runner
  configuration is suspended. There is no automatic third attempt and no runner-shopping. A
  later repair would need a new substantive diagnosis and an independent decision, not
  another discretionary rerun.
- **STOP, INCONCLUSIVE and LEAD each terminate the attempt,** read as the first
  registration's §3 specifies.
- **None of this declares C29 impossible on other machines.** The target-machine question,
  C29's default-on safety and production adoption gates, and D060's obligations stay
  separate.

### R5.6 Unchanged

- **The question and arms:** `mibz2-pipe3-both`, the control and every ratio's denominator,
  against `mibz2-pipe3-both-c29`, in one build, in `--arms` order.
- **The rounds:** 20, interleaved in ab's rotation.
- **The threads:** 9 is the single judged primary (three teams of three); 3 is secondary,
  never judged.
- **The corpus:** Wikidata, SHA-256 `82f65ace…c29915`.
- **The estimand:** the geometric mean of the per-round paired ratios, with its 95%
  t-interval, df 19.
- **The thresholds:**
  - LEAD: wall ≤ 0.98, its upper end < 1, and CPU's upper end ≤ 1.02;
  - STOP: wall's lower end > 0.99;
  - INCONCLUSIVE: otherwise.
- **The other predicates:** every VALID predicate, and R2's and R3's environment predicates.
- **The status of the outcomes:** UNJUDGED (FALLBACK) is terminal, there is no retry to pass,
  and the readings permit what the first registration's §3 says.

### R5.7 Before a freeze, and the freeze

1. This PR's CI is green at its exact head. That covers `cargo test --example ab`, which
   includes the fixtures, and the reporter's tests.
2. The Researcher reviews the collector and the precommit.
3. The preflight is dispatched (`tool: c29-preflight`) on this branch's reviewed head, and its
   artifact is archived. If it fails, the attempt is archived without a decoder run (R5.5).
4. The registration is frozen at a named head, and dispatched once (`tool: c29`, `rounds: 20`,
   `registered_head` that head).

**The chronology:**
1. **CI at the reviewed head `0ce4dca4`:** complete, 27 success and 3 expected skips, none
   failed or pending.
2. **The untimed preflight:** [run 42589756805](https://github.com/lmmx/mibz2/actions/runs/42589756805),
   `tool: c29-preflight` at `0ce4dca4`: PASS, unlimited with complete coverage, the
   configuration stable. Its artifact 13660767430 is archived with the run (ZIP SHA-256
   `ff50767ab52c399137cf4830a17ae0f17dd1b05460dd10b53714b4745022206f`).
3. **The Researcher's REVIEW
   [6739117701](https://github.com/lmmx/mibz2/issues/145#issuecomment-6739117701): PROCEED**
   to freeze R5 at `0ce4dca4` for one timed attempt. Its assessment and audit are at
   `state/reports/c29-preflight-6738933099/report.md` on commit
   `77143028861a07df4b67ea1dbc3e738099527581`. The review bounds the claim:
   - the identity re-parses the quota files and the path chain independently, but mount
     selection still depends on the Rust collector;
   - an unlimited quota does not mean exclusive CPU service;
   - equal endpoints don't prove the configuration stayed stable in between.

   The timed job runs its own preflight and keeps its after-sample (R5.4).
4. **The freeze:** this commit, after CI is green at it. Then one dispatch: `tool: c29`,
   `rounds: 20`, `registered_head` this commit's full SHA. The attempt answers an unjudged
   performance question. This freeze establishes no speedup and no production qualification.

Earlier, local and untimed (this session's 4-core machine, a hybrid v1/v2 host): its census
read v1 `cpu` at `/` as `-1` in the initial namespace, which is unlimited with complete
coverage, and the preflight block passed there. That is not the runner's evidence.
