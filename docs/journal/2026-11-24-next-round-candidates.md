# Candidates for the retrospective's next round (not part of the frozen protocol)

*24 November 2026. The retrospective's protocol is frozen (`2026-11-24-retrospective-proposal.md`,
§3.7), and nothing here changes it. These are candidate checks and ideas raised after the freeze,
recorded so that the retrospective's case data can test them in its next round rather than have
them adopted on enthusiasm.*

## Where they come from

- **The SSL-Coq investigation** (Tariq Ali with OpenAI Codex, the *Super Mario 64* "fewer than one A
  press in Shifting Sand Land" project; repo `tra38/sm64-wmotr-abc-proof`, a transcript Louis
  shared on 24 November). Its method: start from a state that succeeds only because conditions
  were *supplied*; replace one supplied condition at a time with one the game generates, re-running
  the target after each; label every fixture as supplied or controller-produced; state every Coq
  hypothesis (no `Admitted`, no axioms); aim, ultimately, at an impossibility result.
- **The Coordinator's reading** (#145, 24 November): T² broke all three disciplines. The pre-screen's
  ALIVE was a supplied-state success, quoted as evidence about the native target, and the work
  jumped from it to the full three-arm build without the rungs between.
- **ChatGPT's response to that reading**, relayed by Louis, who reported that it preferred the
  Coordinator's version and added refinements, recorded below with the Coordinator's assessment.

## Candidate checks for the next round

1. **The assumption-discharge ladder.** For an idea that starts from a supplied success, list the
   supplied conditions and replace them one at a time, cheapest first, re-measuring the target
   after each; stop at the first rung where the effect disappears. T²'s rungs would have been:
   synthetic cycles in a C harness (seconds) → one real block's tables (seconds, with the census
   tooling) → all 232 distinct blocks (a minute) → the shipped emit at 1 worker (one build) → the
   i9 at 10 workers (15 minutes). The retrospective's data should show how often a cheap rung was
   skipped before an expensive build, and what it cost.
2. **Extract the distinguishing constraint at a failed rung.** When an idea dies at rung k, record
   what differs between rung k−1 and rung k that killed it, as a reusable constraint ("predicts
   construct but not emit", "CPU down, wall flat", "pays at 1 worker, not at 10"). The
   retrospective can test whether such constraints, had they been recorded, would have fired on
   later cases.
3. **Empirical obligations beside formal ones.** PROOF-PROGRAMME's conditional theorems name
   hypotheses to discharge; a performance claim can name its empirical obligations the same way
   (the structure occurs; it reduces work; the work is on the critical path; parallel execution
   keeps the benefit), each with its evidence or marked open, and formally proved and
   empirically supported kept distinct.
4. **Cost-weighted applicability, not frequency.** A condition's value is the share of cost in the
   blocks where it holds, not the share of blocks. A first reading from the census (#805's
   `features.tsv`, 232 distinct blocks, 1-worker emit + construct, base arm, i9):
   - the dense tail (m/n > 0.15) is **19.4% of blocks and 27.6% of kernel time** (30.1% of emit,
     25.4% of construct);
   - the costliest 10% of blocks hold **16.2%** of kernel time;
   - emit is 46.5% of the kernel's time across blocks.

   So Wikidata's kernel cost is spread fairly evenly across blocks. A perfect trick for the dense
   tail is capped at about 28% of kernel time, before the kernel's own share of wall time is
   counted (Amdahl). A transformation that applies to a structural subset should state this
   ceiling before it is built.

## The Coordinator's assessment of the broader proposals

- **Agreed:** separating supplied from native results; recording negative results with their scope
  and first unsatisfied condition (the bench verdicts and journals mostly do this already); a
  failed search is not an impossibility proof, while a proved bound is (record 96).
- **Corrected:** mibz2's target is wall time on the i9 under PERFORMANCE-POLICY, not executed
  instructions (RULES C4 retired the instruction ledger as the gate).
- **Held back:** a machine-readable evidence ledger, a promotion gate enforced by infrastructure, a
  new Synthesiser role and a three-graph knowledge system. They may be right eventually, but
  PERFORMANCE-POLICY's standing instruction is Louis's: "Optimise for useful speedups merged per
  unit of engineering effort … Stop allowing process to become the product." They should be built
  only if the retrospective shows the lighter forms (the evidence-scope line, METHOD.md §6a) fail
  to catch what they would.
