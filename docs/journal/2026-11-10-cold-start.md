# W1: the cold start, its arms and the rule (registered before any data)

The spec is W1 of `docs/journal/2026-11-10-next-experiments.md` (at ffd3445c; Pathfinder
PROPOSAL 6764488488 on #145). The fact: on the i9, after a 0.25 s idle gap, 26 of 30 decodes ran
slow (wall 0.303 s against 0.136 s); with no gap, 0 of 30 (`two-modes/20261110T090731Z`).

## The run (`just bench-cold-start`)

The gap block's harness as #318 ran it (`bench/two_modes.py` on bench/two-modes-discriminator,
copied, not imported, into `bench/cold_start.py`): the archived `ab` from the run-fed leader's
bundle (checked by digest), its `child mibz2-default33 10 FILE`, the Wikidata sample, one fresh
process a run, reaped by wait4. 30 runs an arm, in an order drawn once from seed 20261112,
after one conditioning run that is never counted. A 0.25 s gap (from the parent's reap of the
run before) precedes every run but Z's.

| arm | what |
|---|---|
| G | the gap alone |
| Z | no gap |
| P | `taskset` to one CPU a physical core (the lowest-numbered thread of each core: 0-9 on the i9) |
| W | a spinner on every CPU for 20 ms, ending as the child is exec'd (`bench/cold_start_spin.c burst`) |
| B | one spinner on the last CPU (19 on the i9) from the start of the gap to the reap |
| L | the input four times over, after a gap; input consumed by 100 ms window |

Read passively on every run (a parent thread that sleeps 20 ms between reads; no spinning
probe): each thread's last CPU seen in `/proc/PID/task/*/stat`, `/proc/PID/io`'s `rchar`, and
every CPU's `scaling_cur_freq` once, when the child's stdout closes, before the reap.

## The rule (from the spec; fast is the child's own wall under 0.21 s)

- P fast in at least 25 of 30: pin a thread a core, on the target-gated path. Integrate.
- W fast in at least 25 of 30: a start-up spin in the pool, target-gated. Integrate.
- B alone fast in at least 25 of 30: nothing for the library; the bench's settle changes;
  record it.
- G, Z, L and the passive readings are reported, not judged.
- An arm with fewer than 30 runs timed (exit 0 and a wall) is INCOMPLETE, and no decision is
  drawn from it.

`bench/cold_start.py judge` applies it and writes the decision as the first line of
`outcome.txt`.
