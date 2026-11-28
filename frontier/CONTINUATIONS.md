# Continuations: each chain waits for itself

A claim's work often stops at an asynchronous boundary: CI on its PR head, a merge, a ready tag
landing, a workstation bundle coming back. Before this runbook, a session yielded at every such
boundary and the next gate waited until someone happened to rediscover it. A **continuation**
makes the boundary explicit and resumable without a coordinator:

- **One gate, one key.** `<claim>--<action>`, for example `release-0-5-18--merge-preflight`.
  Its state lives on its own branch, `wait/<key>` on origin, moved only by compare-and-swap.
  Nothing is locked across keys; distinct chains wait and advance independently.
- **An exact pin.** The dependency is named precisely, so readiness is a fact about shared state,
  not a reading of prose:

  | pin | ready when | superseded when |
  |---|---|---|
  | `pr:<N>@<sha>` | PR N is open at exactly that head and every check run and commit status is green (or it merged at that head) | the head moved, or it closed unmerged |
  | `merged:<N>@<sha>` | PR N merged at exactly that head | it closed unmerged, or merged at another head |
  | `evidence:<recipe>@<sha>` | a bundle for that recipe whose commits include that pin is archived on `origin/evidence` | never by itself; other pins are ignored |
  | `ref:<refname>@<sha>` | the ref (a `ready/` tag, a version tag) points at, or peels to, that commit | it points elsewhere |
  | `claim:<id>=<status>[,…]` | the claim on master has one of those statuses | — |
  | `hold-free:<id>` | no live hold on that claim | — |
  | `manual:<text>` | never on its own: resume by hand | — |

  `failed` (a red check at the pin) and `unknown` (GitHub or the evidence branch unreadable) are
  reported as such. Unknown is never absence and never ready.
- **Ready is not PASS.** A ready evidence pin is `returned-unread`. A green head is not a merge.
  `done` refuses without a written disposition: what was read, and what was decided.

## The loop

```sh
T=frontier/tools/continuation.py
# 1. At the boundary, declare the gate (once; a second declare of the same key is refused).
python3 $T declare release-0-5-18 merge-preflight merged:1256@<40-hex head> \
    --priority 1 --critical --next "set ready/<recipe> and queue the preflight" --by <agent>
# 2. Wait in place. One process; polls 45 s, backing off ×1.5 to 300 s; writes only to become
#    the waiter, to renew every half-TTL (15 min TTL) and once on a change. Exit codes:
#    0 ready, 10 timeout (pin kept, waiter released), 11 someone else is already waiting,
#    12 superseded/failed, 13 another peer has acquired it, 14 lost the wait.
python3 $T watch release-0-5-18--merge-preflight --by <agent> --timeout 3300
# 3. Acquire execution (one peer wins; the pin is re-probed first; an independent gate
#    refuses its author). It prints the ordinary next step: take the claim's hold with
#    `frontier.py claim` or `take-review`, `verify`, then act under the usual gates.
python3 $T resume release-0-5-18--merge-preflight --session <s> --by <agent>
# 4. Record what you read and decided; optionally declare the chain's next gate in one step.
python3 $T done release-0-5-18--merge-preflight --session <s> \
    --disposition "#1256 merged at <sha>; preflight recipe frozen" \
    --next-action read-preflight --next-dep evidence:bench-release-preflight@<pin>
```

`view` is the shared, read-only observation: `python3 $T view --can shell,repo-write [--probe]`.
It orders continuations by class (ready, then waiting/unknown, then held, then blocked, failed,
superseded), then critical dependency first, then effective priority (the lower of the
continuation's and its claim's), then oldest ready (else oldest declared), then key. It writes
nothing and assigns nothing: a session with no task may pick a ready row, and acquires it only
through `resume` and the ordinary claim CAS. Owner-blocked (`needs_louis`), capability-blocked
(`--can`) and held rows are labelled, never hidden, and never suppress the others.

## Waiting without spending the model

In Claude Code, run `watch` with the Bash tool's `run_in_background`; the harness re-invokes the
session only when the process exits, so nothing reasons over unchanged state. Other harnesses:
any facility that blocks on a process and resumes on its exit works the same way. A harness that
cannot wait on a process should call `view --probe` at its own checkpoints instead, never a
model-driven poll loop.

**Short waits** (CI, a review in progress, a merge minutes away): wait in place, keeping the
claim's hold if you are still doing its work. **Long waits** (a workstation run, an overnight
review): let `watch` time out, or don't start one. The continuation keeps its pin and next action;
`release` or `yield-review` the hold under the existing rules. Never keep a hold only to wait. A
waiter holds no paths, no queue lock, no admission epoch and no reservation slot.

**Departed peers.** A waiter lapses after its TTL without renewal; an acquisition lapses after
`--hours` (30 min default) without `done`. Either way the record returns to waiting or ready, and
any eligible peer continues from shared state. A takeover re-reads the claim's deltas and branch
first, and never repeats a consequential action that already landed.

**Manual fallback** (no shell, or the tool unavailable): the same record by hand. Write the gate's
exact pin, next action and resume condition in the claim's delta or PR body, check it with
`git ls-remote origin <ref>` or the PR's checks and statuses at your own checkpoints, and say in
the next delta when you saw it ready and when you acted.

## Timing

Each record keeps `declared_at`, `ready_authoritative` (when GitHub says checks completed or the
PR merged, or the evidence commit's time), `ready_observed` (first seen; never overwritten),
`acquired_at`, `disposed_at` and `next_started_at`. `show <key>` prints them with ready-to-acquire
latency measured from the authoritative time where one exists and labelled `observed` or
`unknown` otherwise. A ref pin has no authoritative set time and says so. Times not observed are
not invented. The aim is pickup within one or two minutes of ready when a capable session is
already on the chain; with no session active there is no guarantee, and the record shows the gap.

## What this is not

Not a scheduler, a global watcher, a leader or a queue owner. No pass, merge, tag, dispatch,
queue entry or publication happens through it. It never writes master, a hold, the admission
epoch or the workstation queue, and a poll writes nothing at all. Presence is not required.
