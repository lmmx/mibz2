# Pull-based checkpoints: observe the environment, not a mailbox

26 November 2026. A small shadow experiment and design follow-on, not a new cap gate or deployed harness hook.

## The failure exposed by the cap rollout

The maintainer observed that an active Antigravity participant was doing permutation probes while the newly merged #974 client changed the admission protocol. An instruction to update master can be durable in the repository and nevertheless invisible to a participant that does not read it. The merged repair is 3ced2be8c6ab1e53756949144cf7aa3174d210d0; #987 is handling independent flag resolution and the current rollout. Do not interrupt or duplicate those actions.

The problem is not lack of interpersonal messaging. It is the absence of a predictable observation boundary while a participant remains active. A commit cannot cause an already executing old client to read itself or acquire a new hook. A queue of directed messages does not solve that bootstrap without a reader either.

## Proposal

Use a pull-based checkpoint of shared state, complementary to [the event-stigmergy shadow](../event-stigmergy-shadow/README.md). Observe before taking work, at natural boundaries within a long task (between probe batches, before another long tool invocation, after a benchmark returns), and before every consequential operation such as claim, renew, verify, release, transfer or merge. Aim to check for changes at the first safe point after roughly five minutes of active work; five minutes is a provisional freshness target, not a deadline for an uninterrupted computation, and it must not create an inference loop. No polling while idle is necessary. A local lightweight timer or existing harness hook may trigger an observation if supported; the portable fallback is an explicit checkpoint in the ordinary work loop. No harness integration is claimed here.

Fetch trusted origin/master and compare against the last observed immutable commit. Project only material state:
- Changes to the protocol client or authoritative settings: require a current client before consequential actions. Keep unfinished research intact; integrate master safely or use a separate clean current tooling worktree where appropriate. Never blindly reset a dirty worktree or execute unmerged PR text.
- New or changed flags and affected claim premises/owned paths: reread the relevant evidence; a change is not automatically a contradiction. Unrelated research continues.
- New claim availability or a completed dependency: expose it as an affordance at the next work-selection point, not an instruction to abandon a valid current task.

An observation is a snapshot, not a consumed event. Multiple readers see the same state. Intermediate changes can coalesce, but an active restriction remains visible until legitimately resolved; retain durable history in the existing flags/deltas. No new shared queue, mailbox, acknowledgement record, role, registry of sessions or recipient-specific prompt. A cursor can be local and disposable. Repeated observations are idempotent. No automatic launch, background agent or new paid infrastructure.

Safety requires a fresh check at the operation boundary, not merely a periodically refreshed display. A future protected tooling change should compare the actual running client version to authoritative master, and refuse stale or unavailable client state before mutation. A new client-side guard cannot retroactively constrain old clients or direct ref writers; verified rollout or server-enforced fencing remains necessary wherever applicable. The existing cap gate must not be weakened; this follow-on must not become an additional arbitrary precondition to the already authorised 3-to-6 transition. No automatic refresh should change a work branch, consume a hold, or grant authority.

## Minimal read-only prototype

From a clone with the prototype available:
    
    python research/frontier-checkpoint/checkpoint.py --repo .
    python research/frontier-checkpoint/checkpoint.py --repo . --since LAST_OBSERVED_MASTER
    python -m unittest discover -s research/frontier-checkpoint -p 'test_*.py'

The prototype fetches master, compares the actual working-copy client blob to the authoritative one, and reports changed control paths since the supplied observation or merge-base. Exit 1 means client mismatch, exit 2 means unavailable. Exit 0 only means matching client blob; it is not permission to mutate. No working tree or hold writes, notifications or acknowledgements. Three hermetic bare-remote tests passed in an isolated local Python run. It does not yet project relevant flags or premises semantically, install a periodic poller, or enforce a production guard. Fetch updates a local remote-tracking ref; it never merges.

## Next bounded step

Test an active long-lived checkout with dirty research, a protocol update, an unrelated commit, a scope-specific flag and network failure. Demonstrate that the next safe checkpoint detects relevant changes, preserves work, and does not require directed messaging. Then propose the smallest protected-client integration that refuses stale authority and uses existing brief/verify rather than adding another management interface. Measure the latency and overhead of the observation path; coalesce and skip unchanged state. Test a mixed-version scenario honestly: an uninstrumented old client will not self-update. The current rollout remains with #987 and the separate cap PR. This is a small lifecycle improvement, not a new coordinator.
