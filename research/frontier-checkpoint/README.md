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

## Prototype (read-only)

    python research/frontier-checkpoint/checkpoint.py --repo . --claim CLAIM --cursor .git/frontier-checkpoint
    python -m unittest discover -s research/frontier-checkpoint -p 'test_*.py'
    (cd research/frontier-checkpoint && python measure.py)

One observation fetches master (only the remote-tracking ref moves; nothing merges), compares the working-copy client blob with master's, lists changed control paths, and for `--claim` projects the change onto that claim's declared premises, owned paths, card and open flags. It reports dirty files that master also changed, so integration can be done by hand or in a separate worktree, and the number of commits coalesced since the cursor. The cursor is a local, disposable file (put it in `.git/`), written only after a successful observation; it is not an acknowledgement. Every open flag on the claim is reported on every observation (`open_flags`), so moving the cursor never hides an active restriction; `new_flags` is only a convenience. Exit 1: client mismatch; exit 2: unavailable (a failed fetch never falls back to the cached ref and never moves the cursor); exit 0 grants nothing.

## Tests against the stop test (isolated temporary bare remotes)

Seven hermetic tests (`test_checkpoint.py`), all passing on 26 November 2026:
- *Stale client detected at next observation*, including a locally patched client.
- *Unrelated dirty work preserved:* HEAD and `git status` are byte-identical before and after; a dirty file that master also changed is reported, not touched.
- *Relevant scope change surfaced:* a premise edit, a path-scoped flag and a re-frozen stop test are reported for the claim; an unrelated commit yields `continue current work`.
- *Repeated snapshot idempotent:* identical output from the same cursor; three intermediate commits coalesce into one observation; an unchanged master takes a fast path.
- *Unavailable origin never grants authority:* the cached (stale) ref is not used, the cursor stays put, the CLI exits 2.
- *Mixed old-client limit explicit:* after a protocol change, a client that does not observe can still push a `hold/` ref to the remote. Nothing here fences it; `limits` says so.

## Cost and safe-point latency (`measure.py`)

Fixture: 40 claims, 20 flags, a dirty checkout, local bare remote, 0.2 s indivisible work batches, a 0.5 s observation interval standing in for five minutes. Observation cost: median 0.022 s unchanged (max 0.026), 0.118 s with claim scope (max 0.134; two flag-tree reads, each flag one `git show`). Against the real origin from a cloud session: 0.52 s wall, 0.15 s CPU, dominated by the fetch. Detection latency after the push, five runs: 0.07–0.53 s, all within the analytic bound interval + batch = 0.7 s (plus one observation). At the real five-minute target the bound is five minutes plus the longest indivisible step; a twenty-minute benchmark call is detected when it returns, never during it. Local remote only: the latency numbers exclude network variance.

## Minimal protected integration (proposal; a separate, reviewed PR)

1. In `frontier.py`, `claim`, `verify`, `take_review` and `finish_review` already call `_fetch_master(strict=True)` (as of 8f214b8); at those sites, and in `renew` and `release`, which do not yet fetch, add one comparison of `hash-object` of the running client file (`__file__`) with `origin/master:frontier/tools/frontier.py`, and refuse those mutations on mismatch or fetch failure, naming the refresh step. No new command, ref, queue or state.
2. `brief` prints the same comparison and the claim's `open_flags`/`premise_changes` since a local cursor, so the long-work checkpoint is simply "run `brief --since` between batches"; `renew` (already the long-work boundary) is the natural five-minute safe point.
3. README step 3 gains one sentence: observe at batch boundaries during long work. That is the only change to agent behaviour.

This cannot reach a client that predates it (the old-client test): the first rollout still depends on agents reading master, as #987 did. Fencing a direct ref writer needs server-side protection (`branch-protection`), outside this claim.
