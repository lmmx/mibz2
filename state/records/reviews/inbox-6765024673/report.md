# Held-spinner brief: scoped review and build handoff

Inbox [6765024673](https://github.com/lmmx/mibz2/pull/160#issuecomment-6765024673). Disposition: **implementation not performed; pinned-role conflict reported.** This completes the response to the brief, not the requested feature.

## Exact authority mismatch

The user explicitly pins this run to a2da49638ac54d7763de1c167bd054f67ffdc763. At that commit, [SPEC.OPTIMIZER.md](https://github.com/lmmx/mibz2/blob/a2da49638ac54d7763de1c167bd054f67ffdc763/docs/team/SPEC.OPTIMIZER.md) says: “Proposals are proposals only. Never implement, time anything, merge, or widen authority.” SPEC limits writes to the state branch and REVIEWs on145; only Louis moves the instruction pin.

The Coordinator cites O1/#345. I resolved and read the newer Optimiser spec and OWNERSHIP at **3664ada282fb6e3cedf4c6ad91053725d9ced18f** as evidence. They do add implementation ownership. They are not the user-selected governing pin. Moreover the newer Optimiser spec still says **“Never merge”** and assigns the merge decision to the Coordinator. Thus even that spec would not authorize the brief's literal instruction to merge independently.

The cheapest route is for an already-authorized implementer to take the build now, or for Louis to re-pin this task through the established procedure before implementation is dispatched again. No automatic prompt change, role expansion, feature branch, PR, bench run or merge is made here.

## Concrete engineering handoff

Source inspected: [examples/ab.rs](https://github.com/lmmx/mibz2/blob/3664ada282fb6e3cedf4c6ad91053725d9ced18f/examples/ab.rs). The current run options include max-busy (default1.5), settle, pin/spread and parent/child telemetry; the parent observes the child and waits for output. A held spinner must fit those existing boundaries.

1. **One owner, outside the measured child.** For ab run, a supervisor owns the spinner from before settling until the decoder is reaped. A direct ab child invocation needs an equivalent outer owner. Avoid spawning once in run and again in child. An in-process spinner in the timed decoder contaminates process CPU and thread accounting; exclude it from decoder metrics and explicitly report its auxiliary resource use.
2. **Readiness before settling.** Validate affinity against the actual allowed CPU set, pin successfully, then acknowledge readiness. Failure must be an explicit failed run, not a recorded held_spinner value with no spinner. Record requested and actual CPU, startup result and lifetime.
3. **Settle interaction.** The existing max-busy gate measures other-process activity. A busy auxiliary core can consume its allowance or prevent settling under a lower threshold. Preserve raw host observations, identify/subtract only measured spinner work for the gate if appropriate, or define a documented explicit adjustment. Do not silently change CI defaults or subtract an assumed full CPU.
4. **Topology, not thread count alone.** Highest-numbered allowed logical CPU need not be a spare physical core. At10 decoder threads on a10-core/20-thread i9, CPU19 can share a core with a decoder thread. Scanner/reader work also exists. Verify cpuset/core/sibling topology and report overlap; the claim “one idle CPU” is not guaranteed by <=10 threads. The observed W1 result can still justify this precise benchmark condition.
5. **Cleanup covers all exits.** Normal completion, decode error, spawn/affinity failure, interruption and supervisor death must not leave a spinner running. Use explicit lifetime ownership and reap, not just a happy-path stop flag. Test invalid/disallowed CPUs, double-spawn prevention, nonzero child exit and interruption as well as start/stop. Process-based cleanup needs a parent-death strategy; a destructor alone does not cover abrupt termination.
6. **Facts and compatibility.** Off by default, include held_spinner:null when disabled and the actual CPU when enabled. Keep baseline/alternative arms compatible: the current supervisor should manage the helper without requiring old ARM@EXE binaries to parse a new flag. Record conditioning consistently in raw facts, report and recipe identity; version it. Apply only to intended i9 recipes after checking their complete thread grids.

## Evidence and cost

W1 [LOG6765016578](https://github.com/lmmx/mibz2/pull/145#issuecomment-6765016578) reports B30/30 fast, median0.141s, against G9/30, median0.293s. I have **not independently replayed its submitted bundle**, so these remain reported findings. They support a held-spinner benchmark option on that setup, not a universal idle-state mechanism or a library cure.

This is a changed workstation measurement condition: it can reduce mode mixing but does not make cold-start latency disappear for users. Existing archived results retain their conditions and verdicts. “CI unchanged” does not imply workstation figures remain directly comparable across this change.

At continuous one-core utilization, auxiliary cost is approximately one CPU-second per elapsed second held, including settling and gaps. A2s settle would alone add about2CPU-s, larger than a0.141s decode's spinner cost; this is a model, not new timing. Decoder-only CPU metrics must remain separate from total experiment cost. Keep mode consistency and conditioning facts; do not claim an application speedup from excluding that core's cost.

Priority: this small measurement-hygiene change is useful ahead of further affected i9 comparisons. It does not supersede the production performance backlog or warrant another mechanism investigation. No benchmark-acceptance change or new run is authorized by this review.

## Recorded outcome

Expected: an implementation under newly announced ownership. Found: that ownership is absent from this run's explicit instruction pin. Feature completion: none. Useful advance: precise owner/lifetime, settle-accounting, topology and baseline-compatibility requirements identified for immediate handoff. Next action: authorized implementer builds; Coordinator decides merge after review. This task stays available for subsequent properly scoped briefs.
