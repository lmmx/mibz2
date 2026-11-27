claim: frontier-checkpoint-freshness
by: claude-code
outcome: done
evidence: research/frontier-checkpoint/{checkpoint.py,test_checkpoint.py,measure.py,README.md}; master 8f214b8

**What was asked:** test pull-based safe-point observation of master, claim scope and client freshness against the frozen stop test, on isolated temporary remotes, and propose the smallest protected integration.

**What I found:** every stop-test clause holds in seven hermetic bare-remote tests:
- a stale (or locally patched) client is detected at the next observation;
- dirty work is preserved, with HEAD and status byte-identical, and a dirty file that master also changed is reported, not touched;
- a premise edit, a path-scoped flag and a re-frozen stop test surface for the claim, while an unrelated commit yields "continue";
- the same cursor gives identical output, intermediate commits coalesce, and an unchanged master takes a fast path;
- a failed fetch never uses the cached ref or moves the cursor (exit 2);
- an old client that never observes can still push a hold ref, and `limits` states this.
Open flags are reported on every observation, so a cursor never hides a restriction.

Cost (40 claims, 20 flags): median 0.022 s unchanged, 0.118 s scoped; 0.52 s wall against the real origin, mostly fetch. Detection latency was 0.07–0.53 s over five runs, all within interval + batch (0.7 s scaled). At five minutes, the bound is five minutes plus the longest indivisible step.

**What changed:**
- `checkpoint.py` gained `--claim` scope projection, the local `--cursor`, coalescing, dirty-overlap reporting and an `Unavailable` failure that fails closed.
- Tests went from 3 to 7, and `measure.py` is new.
- README: results and the integration proposal.
No protected path, cap gate or rollout was touched.

**What's next:** a separate, reviewed protected PR could add the running-client blob check at the strict-fetch sites in `frontier.py` (claim, verify, take_review, finish_review), plus renew and release, and add `brief --since`. See the README proposal. Server-side fencing of direct ref writers is `branch-protection`.

**Caveats:**
- Local remotes only: the latency figures exclude network variance.
- Time was scaled (0.5 s stood in for 5 min).
- Path matching is not semantic relevance.
- Nothing is deployed, and an uninstrumented old client stays unreachable.
