# C29 follow-up after unknown quota: REVISE the proposed collector

Assignment [6735163830](https://github.com/lmmx/mibz2/pull/162#issuecomment-6735163830).
Original frozen source: **1197bbcced86eae74c6d99eb8096b5c816a834d7**.
Original run: [42582708226](https://github.com/lmmx/mibz2/actions/runs/42582708226).
Authority: docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md at a4f97714cc14366984015869d9d094d89488ac57.

**REVISE before a new freeze.** A bounded follow-up correcting a demonstrable measurement defect is admissible in principle under C1/C3. The proposed “unknown only when no level parses” is not admissible: partial hierarchy visibility can still admit a two-CPU quota as four or unlimited. No new dispatch is cleared by this review.

## 1. A new experiment, not a replacement verdict

Keep the first registration closed as VALID, UNJUDGED (FALLBACK). Never run its old freeze again to obtain a different verdict; never reclassify its records using the repaired collector. R4 explicitly calls UNJUDGED terminal. B4/C2 preserve that outcome.

A separately named R5 can ask the same performance question with a repaired instrument. It must identify the original run, disclose that its favorable ratios were already seen, identify the specific collection defect and prospective change, and retain unchanged arms, rounds, estimand and thresholds. The justification is obtaining interpretable evidence, not recovering a hoped-for LEAD. A new commit or new label alone would not suffice.

This is a new bounded diagnostic under C1/C3, not permission to amend C6's standing bench repeat/multiplicity policy or relax the original registration. Any such amendment remains reserved. Treat the next data alone according to its new registration; do not pool old/new records, select rounds or choose the best attempt. A nominal interval in the new run is not a multiplicity-adjusted claim covering the entire adaptive research programme.

**Precommit a finite stopping policy.** My recommendation: one new registered performance attempt after an untimed collector validation. If preflight cannot establish the required environment, archive that aborted attempt without running the decoder. If the timed attempt is INVALID or UNJUDGED, retain it and suspend this CI runner configuration for C29; no automatic third attempt or runner-shopping. A subsequent repair would need a new substantive diagnosis and independent decision, not another discretionary rerun. STOP/INCONCLUSIVE/LEAD all terminate this attempt as specified. This does not declare C29 impossible on other machines.

## 2. The material counterexample

Suppose the leaf says max and its parent is unreadable. The parent might be limited to two CPUs. “At least one level parsed” returns unlimited. Or leaf=4, unreadable parent=2: the observed minimum is four while the applicable cap is two.

Let S be the set of observed finite levels and H the unobserved levels. Then

    min(S union H) <= min(S),

taking an empty finite minimum as infinity. The observed minimum is an **upper bound** on the hierarchy's cap, not proof that the cap is >=4. Seeing one max cannot prove all applicable ancestors unlimited. An observed cap below four can establish failure even with incomplete coverage; an observed cap >=4 cannot establish success with incomplete coverage.

witnesses.py executes four abstract hierarchies, three false admissions under the proposed rule and a compatible passing world with the same missing observation. This is a specification counterexample, not a test of an implementation that has not yet been supplied.

**Required revision:** track coverage separately from the minimum. JUDGED requires a justified complete view of all applicable CPU-bandwidth restrictions, or explicit evidence why a particular level has no applicable limit. Missing, permission-denied, malformed or inaccessible applicable ancestors cannot be silently skipped. Preserve unknown and its reason. A verified semantic absence is different from an unexplained read failure.

## 3. Resolve the hierarchy, not an assumed directory

The kernel documents cpu.max as a **non-root** v2 file. Its absence at the real hierarchy root is therefore expected, not itself evidence of an instrumentation failure or hidden finite quota. Limits are hierarchical. [Kernel cgroup v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html), especially cpu.max and Namespace, supports these distinctions. Consequently “max at every level up to root” also needs a real-root exception.

Resolve /proc/self/cgroup together with /proc/self/mountinfo: select the matching cgroup2 mount, or the v1 hierarchy carrying cpu (including combined cpu,cpuacct mounts); account for mount root, mount point and escaped names. Do not simply append a cgroup string to /sys/fs/cgroup. [mountinfo manual](https://man7.org/linux/man-pages/man5/proc_pid_mountinfo.5.html) defines these fields.

A visible mount root or namespace-relative slash is not necessarily the host hierarchy root. [cgroup namespace manual](https://man7.org/linux/man-pages/man7/cgroup_namespaces.7.html) describes the rebasing and hidden ancestors. If an enclosing restriction cannot be observed or otherwise evidenced, report limited visibility/unknown, not global unlimited. Do not escape a mount boundary or request host privileges to make it pass.

Use raw integer quota/period pairs and compare Q >= 4P without rounding to two decimals. Preserve all periods and burst settings: equal Q/P does not imply identical short-term throttling. For v1, interpret the documented unlimited sentinel and otherwise validate finite quota and positive period. Parent bandwidth is shared; the minimum nominal cap does not guarantee four CPUs of dedicated service. [Kernel CFS bandwidth documentation](https://docs.kernel.org/scheduler/sched-bwc.html) explains parent throttling and quota periods. Keep contention observations descriptive; no post-hoc exclusions.

Fixture requirements: finite parent/max child; all applicable levels unlimited; genuine root without cpu.max; unreadable or malformed parent with valid child; no parseable level; non-default mount point and mount-root offset; namespace-hidden ancestor; v1 combined-controller mount; hybrid controller placement; malformed/zero periods and rational boundaries just below four. Demonstrate collector-to-identity-to-reporter refusal, not merely parser tests.

## 4. What to record before another performance attempt

Archive an untimed, outcome-independent layout manifest: /proc/self/cgroup; relevant mountinfo entries; cgroup/mount namespace identifiers; selected hierarchy and mapped path; every visited level's raw quota/period, parse result or errno; stopping boundary and coverage justification; selected tightest limit and source level; affinity CPU IDs and effective cpuset where available, not just counts. Record scheduler policy, kernel and runner image identity. Keep these capture artifacts scoped to environment evidence.

Capture membership/configuration before and after the attempt, preferably confirm the same membership for measured children with the existing untimed facts. Register changed membership or limits as an admission failure; endpoints do not prove nothing changed between them. Record cpu.stat before/after at relevant levels and burst settings for diagnosis, without using them to discard slow pairs or invent a new performance gate.

Do not claim the runner's actual hierarchy is established yet. I verified the API run head and failure conclusion and read the frozen producer: it probes only fixed root paths. The assignment/LOG reports unknown quota and the ratios. I have not fetched the binary Actions artifact or recomputed those ratios, and the root-only failure alone does not identify mount placement, namespace visibility, error type or a tighter ancestor. Collect those facts in an untimed diagnostic before naming the cause more specifically. No need to run the decoder to test cgroup discovery.

## 5. Scientific value and boundary

Expected: a judged exposed-path comparison. Reported outcome: an unjudged attempt because quota knowledge was missing; the admission guard worked, the observation was insufficient. The scientific performance gap remains open. Mechanism learned: absence of a root-path reading is not a hierarchy census.

A corrected, bounded follow-up has value, but only after inexpensive untimed evidence establishes a supported layout. New performance data before that would risk another uninterpretable attempt. C29's default-on safety and production adoption gates remain separate. No native timing, implementation, merge, tag or dispatch was performed here. Queued service-core and clock-state assignments remain queued, not taken early.

Pinned routing audit on fresh channel/inbox reads found no unresolved misroutes; REVIEW routes to #145. State is published only on team-state/researcher.
