# #317 final readiness: narrow REVISE of supervisor failure handling

Inbox [6756992499](https://github.com/lmmx/mibz2/pull/162#issuecomment-6756992499). Reviewed ready head c2f461491b97a563ca3f8669b5c3d7e14eeff07f, frozen inputs 2e5d0bd2008c9210df80e32958dc2c67f8800e8c. Instructions docs/team at a2da49638ac54d7763de1c167bd054f67ffdc763.

**REVISE before queue admission, limited to the supervisor's cleanup-failure path.** The sampler repair and ordinary grandchild cleanup address my prior concrete counterexamples. The diagnostic's scientific framing remains acceptable. Do not reopen those conclusions merely because a residual failure path remains. Guardian separately decides B5/admissibility.

## What now closes

- In examples/par_scan.rs:1132, the OnDrop guard is inside the scope closure, before run(). During unwinding it sets done before thread::scope joins the sampler. The atomic flag carries only stop intent; sampler results synchronize through join. The four additional sampled-joiner fault cases exercise the previously missing path. I inspected the guard and injection; I did not rebuild/replay the native 16 cases here.
- The unchanged bench/bounded.py and test_bounded.py from the reviewed head passed all four supplied tests locally: status passthrough, responsive grandchild, TERM-ignoring grandchild beneath a terminating parent, and leftovers after normal exit. pytest was unavailable; I invoked the same test functions directly with fresh temporary directories, preserving their assertions. These are process-liveness tests, not decoder performance measurements.
- stop_group sends KILL to the process group independently of direct-child completion. This closes the old GNU timeout counterexample under the documented same-process-group scope. Escaping sessions remain explicitly outside scope.
- Recipe input generation/anchoring statuses 124 and 125 now go through late() to INVALID. Placement is recorded, not controlled; the cgroup path is recorded. Earlier acceptance of independently anchored bytes and the descriptive paired-ratio reading stands.
- GitHub comparison frozen...ready is ahead and changes only bench/par-scan.frozen. Thus measured-file identity at these two commits is consistent. The ready head's checks are success or intentionally skipped (Release/Publish crate), with no failed/pending check in the fetched response.

## Residual: failed cleanup can bypass its own bounded refusal

bench/bounded.py run() does:

    emptied = stop_group(...) ...
    rc = child.wait()
    if not emptied:
        ... return 125

stop_group has already exhausted its bounded cleanup interval when it returns False. Yet run() then calls wait() without a timeout **before** testing that result. If the direct child has not exited, this blocks instead of returning the promised 125. Sending KILL is not an observation that a process has exited. The script itself explicitly models failure to empty the group, so its caller must implement that branch.

The signal branch is worse in the same respect:

    stop_group(child, pgid, grace)
    child.wait()
    return 143

It ignores cleanup failure entirely. Even if the direct child has exited while another group member survives, it returns 143 rather than the documented 125 and omits the surviving-member failure diagnostic.

**Independent reproduction:** probe_failure.py uses mocks at the Popen and stop_group boundaries only; it executes unchanged run(). No unkillable real process is created. With cleanup=False and a child not known exited, the probe observes an unbounded wait before any refusal. With a signal, direct child exited and cleanup=False, it observes return 143. Output:

    deadline_cleanup_failure: wait called without timeout after failed cleanup
    signal_cleanup_failure: 143

This is a deterministic control-flow counterexample, not evidence that a real tested child resisted KILL. Ordinary happy cleanup tests all pass; they do not exercise the branch the API promises for unsuccessful cleanup.

**Required correction:** on every shutdown path, preserve cleanup success/failure and check failure before any blocking reap. Bound any remaining wait by the remaining cleanup budget; report unresolved PIDs/group and return 125 on failure. On successful cleanup, reap without reopening an unbounded wait. Signal interruption must follow the same cleanup-failure priority. Add failure-path tests with a child that remains unreaped and with an exited parent plus a surviving descendant; use deterministic mocks rather than trying to manufacture kernel stalls.

No supervisor can promise to force an arbitrary kernel-blocked task to disappear. State the contract honestly: bounded cleanup attempt, bounded refusal if not empty, recorded survivors and no progression to later measurement cells. Keep the existing actual-child tests too. Re-freeze because bounded.py is in the measured identity list, and repeat the untimed dry run and exact-ready-head CI.

## B5 evidence and scope

I read #320's independent review (head e10d1814beff2e2471c6852a9966558b644c30fc; merged) and journal §8.6, then inspected the integrated constructor/range fixes and regression tests. Finder::new now gates the requested AVX2 path on CPU detection; find returns when from >= the clamped to, removing the cited wrapping-bound entry. The Miri length subset includes the previously missed AVX2 boundary lengths.

The independent addendum is substantive: two seeds, about 43.6M reported differential inputs with guard placements, an enumerated mutation matrix, and before/after Miri reproducers. These are attributed run reports; I did not rerun the 600-second fuzz campaign, Miri, or raw mutation executions. The submitted untimed end-to-end dry run is likewise reported, not replayed here.

M28 remains explicitly non-equivalent and surviving: removal of the getrusage return check. Keeping the defensive check is good implementation, but does not itself turn that mutation into killed evidence. Guardian must explicitly resolve its B5/B6 scope/admissibility; this review supplies no waiver. An injected failing FFI result is a direct way to test panic-to-INVALID if that obligation is material. Distinguish test gaps from a demonstrated reachable memory-safety bug.

## Decision and value

Expected: final queue readiness after sampler, process-group and safety repairs. Achieved: the old sampler and grandchild mechanisms are addressed, the four real process tests pass, and freeze/ready identity and green checks are verified. Remaining gap: the cleanup-failure return path defeats its own bounded contract.

Next work is a small supervisor correction plus failure-path tests, not another performance experiment or a redesign of the scanner study. No workstation run, production implementation, merge, queue/tag mutation, or authority change was performed. Report and reproduction only on team-state/researcher. Migration digest/head checks passed; pinned routing audit after 6756808726 returned no issue. Persistent task settings unchanged.
