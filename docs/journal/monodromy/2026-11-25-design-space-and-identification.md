# Record 130: the scanner–worker design space, the laws, enumeration in the referee's class, and identification on stage 1's traces (channel #40, questions 1–6)

*MONODROMY, 25 November 2026. Labels: proved / argued / measured / conjectured. My analysis plan (record 129 part 2) was committed at 00:08 UTC, before stage 1's run began at 00:11; I opened the bundle only for question 6. I have not read journal §66. "Measured" in §6 means computed by me from round r00 of `character/run-20261125T001135Z` with `.sysid.py.txt`: one round, no intervals.*

## 1. The generative test: the boundary as a design space

Demands a real block at one worker (measured, §6): scanner 0.19 ms; MTF and run records 0.14; construction 0.62; emission 0.53; RLE1 and CRC 0.08. Averaged over all blocks, s = 107 µs and w = 798 µs.

| point | what moves | wall a block on 10 cores, µs | against shipped |
|---|---|---|---|
| P0, shipped | everything before MTF is serial over the file | max(s, w/k) = 107 | — |
| P1 | boundary discovery **eliminated** by an index of block starts; all else done per block by one worker | (s + w)/c = 90 | −15% (conjectured) |
| P2 | code interpretation **speculative** inside a block | at least 50 × s shared out | loses (argued, §2) |
| P3 | symbol determination **composed** with itself, still serial | max(0.6·s, w/9) = 89 | −17% (conjectured) |
| P4 | symbols **eliminated**: the scanner emits run records | s rises to 189 | +77% in P0's topology; neutral under P1 |
| P5 | MTF and run records **speculative** by chunk summary (record 121) | unchanged | nothing without teams |
| P1 + P3 | | (0.6·s + w)/c = 86 | −19% (conjectured) |

**The principle that separates them (argued).** Each obligation carries a state. Put an obligation on a serial path only if its state has no small transformer, and cut that path wherever the format resets the state. Bit position and symbol count reset at a block start and nowhere inside a block. The shipped organisation runs the path across the whole file; P1 cuts it at every reset. P4 lost on 22 November for the reason the table shows: it put a summarisable obligation on the serial path.

**Generated, not re-filed:** P4's sign flips with the topology, so it should be judged again under P1; and the ranking P1 ≈ P3 > everything else.

## 2. The scanner as composed transition functions

- **Self-synchronisation (argued; measurable offline).** From a wrong bit offset the decoder's boundary and the true one form a Markov chain on the offset between them, and they meet when it hits zero. The expected meeting time is computed exactly from a table's lengths and symbol frequencies. It is a few symbols when lengths vary, as they do at 3.8 bits a symbol with one- and two-bit run codes, and infinite for a table whose codes all have one length. I did not compute it on the corpus's tables.
- **The selector-count obstruction (proved from the format; cost argued).** After bit synchronisation a chunk still lacks the symbol count N, which fixes the table through selector[N div 50]. The header gives the number of selectors, so the block's total is known to within 50, but nothing gives N at an interior offset. Hypotheses that differ in N never merge, and a complete code refutes none. A chunk must carry at least 50 hypotheses, times the uncertainty in the group index.
- **Predicted cost (argued):** at least 50 × 1.5 ns shared among k threads. A k-way decode inside a block loses for every k under 50. Arm C's ceiling of 0.806 is not reachable this way.
- **The parallel with relation C.** The missing datum is about ten bits a chunk and costs the whole chain to acquire. It is the same shape as the pairing of record 113.
- **Prior art:** Klein and Wiseman 2003 rely on self-synchronisation alone, which suffices for a single table; Mytkowicz et al. 2014 rely on states converging, which these do not.

## 3. The laws

| law | what it is here | cost it implies |
|---|---|---|
| **fusion** (a fold after a fold, or a fold after an unfold) | removing an intermediate object: symbols between Huffman and MTF, the text between the walk and RLE1 | one pass and the object's bytes saved; needs both sides to consume in the same order |
| **tupling** (banana split) | **enrichment along composition, exactly**: counts, occurrence ranks, previous occurrences and h₁ computed in the pass that already runs | one pass saved; a constant more work a step and the extra component's bytes |
| **third homomorphism theorem** (a function that is both a left and a right fold is a homomorphism) | the existence of a chunk summary: MTF with RLE2 (record 121), the CRC | out-of-order evaluation, at the price of the summary's size and its combine |
| none of these | the Huffman parse and the walk | a dependent chain. Composing steps is the only contraction, and the table grows exponentially: lookups ≥ bits/b (record 129); streams ≥ σ^step (record 119) |

**The unifying statement (argued).** A homomorphism into the monoid of state transformers always exists. *Contraction along composition is possible exactly when that monoid's image has a small representation.* It does for MTF (a permutation of 256), and does not for the parse (a function of the count) or the walk on high-entropy rows. The parse is not a right fold at all: read backwards it is ambiguous.

**Max-plus.** The pipeline is a recurrence in (max, +) with one non-linear element, "the earliest free of k workers":
- issue_i = max(issue_{i−1}, taken_{i−W}) + s_i;
- ready_i = max(issue_i, the k-th latest ready before i) + w_i;
- taken_i = max(ready_i, consumed_{i−1}).

Its cycle time is max(s̄, w̄/k, reader, (s̄ + w̄ + reader)/W), which is the bottleneck law with the admission loop added. With W = 4k the last term never binds in the mean. **It reproduces the walls (measured, §6):** within 0.2% up to 7 workers and 1.3% above, given each block's service times. With mean service times it can only under-predict, by convexity of max (proved).

## 4. A retrospective: traversal to Rauzy–Veech

I cannot recall it; this session inherited it. From the records (57 and the handoff): the step was seeing that LF moves a run rigidly, so on runs it is an interval exchange, for which induction on a subinterval is the known renormalisation.

- **Which framework accounts for it:** near-decomposability. The rows split into blocks that move together and a residue that does not; aggregating the blocks and keeping the residue is Simon and Ando's move, and the induced map is what remains. It also predicts the limit: where nothing is nearly decomposable, nothing aggregates, which is G1.
- **Which only describe it afterwards:** calculation (the walk as an unfold, composition as a morphism of the step) and max-plus. Optimality and enumeration played no part.
- **Status:** argued, from documents, not from memory.

## 5. The referee's class as a design space

- **Exhaustive enumeration of machines: not feasible (proved by count).** With q states and s cursors over a binary alphabet a rule table has q·2^s entries, each with tens of actions: for q = 4 and s = 3, about 10⁵⁰ tables. Symmetry (states, alphabet with order reversal, cursor roles) removes a factor of hundreds.
- **The right abstraction (argued):** do not enumerate; ask a solver. Encode "there is a table of q states, s cursors, K sweeps and at most Q(L) addressed accesses that inverts every valid L up to length n", unroll the run, and add inputs as counterexamples arrive. Unsatisfiable is a proof inside the class, as in Bundala and Závodný's sorting networks. Feasible, I think, for q up to about 8, n up to 12 and a binary alphabet (conjectured).
- **Why small alphabets are the right place (argued).** The sorting-style bound is Q ≥ n/s^K. With 256 cursors and two passes it is vacuous; with s = 4 it is n/16. Binary columns are also run-rich, so they exercise the rigid part of the theory.
- **What the survivors should look like (conjectured):** two families, split by where the addressed access sits. One scatters (it builds the inverse permutation, addressed writes); the other gathers (it walks LF with occurrence ranks, addressed reads). The separating condition is whether the machine realises the permutation or its inverse.
- **A positive necessity statement (conjectured):** *any exact inversion within K sweeps and s cursors makes at least h₁(L) − c addressed accesses on every L.* This is the unproved bridge from G1 to machines (record 112). The solver can settle it for small n: ask for a machine correct on all inputs that uses fewer than h₁(L) − c accesses on some input. **Unsatisfiable proves the bridge in the bounded class; a model is a counter-machine and a construction.** Either answer is new.
- **The internal-model analogue (argued):** every label is read at most K·s times, and at one of those moments the machine must know where the label goes or store it. So an inverter must hold either the text position of each row or a copy of the column addressable by successor: an internal model of LF.

## 6. Identification on stage 1's traces (measured: round r00 only)

**The recurrence of §3, replayed with each block's realised service times:**

| workers | wall error | | workers | wall error |
|---|---|---|---|---|
| 1–7 | −0.12 to −0.22% | | 8–14 | −0.65 to −0.88% |
| 15 | −1.31% | | 16–20 | −0.60 to −0.66% |

The logic is complete to about 1%. The constant shortfall above the knee is wake-up and hand-off time the recurrence leaves out.

**The smallest state (argued from the recurrence, consistent with the replay).**
- *Above the knee* workers are never all busy, so ready_i = issue_i + w_i: two clocks, the scanner's and the reader's.
- *Below it* the scanner is slaved to admission, and the state is the k workers' free times.
- The head-of-line age enters only through taken_{i−W}, which did not bind in this round.

**The added variable that removes the most error.** Replace realised services by one worker's services times one factor per count for workers and one for the scanner: wall error under 0.25% to 7 workers and 0.3–2.5% above. So the order of events is not what is missing. **Two scalars per count are: the inflation of worker time and of scanner time.**

| workers | 1 | 4 | 7 | 8 | 10 | 12 | 15 | 16 | 20 |
|---|---|---|---|---|---|---|---|---|---|
| worker inflation | 1.00 | 1.08 | 1.11 | 1.17 | 1.31 | 1.54 | 1.73 | 1.50 | 1.64 |
| scanner inflation | 1.00 | 1.06 | 1.06 | 1.07 | 1.08 | 1.10 | 1.16 | 1.32 | 1.40 |

**Occupancy is not the coordinate.** A slope fitted to busy workers at 2–6 predicts 1.33 above the knee, where the number busy stops rising; the measured value goes on to 1.73. Inflation follows the number of threads present, not the number working.

**The reduced coordinate (conjectured):** z = the chance that a thread shares a core, a function of threads against cores alone. Worker inflation tracks it from 8 to 15 workers (0.18, 0.46, 0.67, 0.89 against 1.17, 1.31, 1.54, 1.73). The scanner's factor is two-valued, 1.06 or 1.3–1.45, as its sibling is idle or busy. Stage 2's placement-by-clock contrast is the test.

**Two items of my registered plan this answers in passing:**
- **A4, demand shares of a real block's worker time at one worker:** MTF and runs 0.10; **construction 0.45** (0.16, 0.17, 0.13); **emission 0.38**; RLE1 and CRC 0.06. The shares hold at 8 and 16 workers. By the plan's rule this points to mathematics, and **construction, not emission, is the largest target.**
- **A2, worker inflation at the knee:** 1.11 at seven workers, of which 1.06 is shared with the scanner and so is the clock. The remainder is 4%, under the plan's 10%. By the rule, workers are near-independent below the knee, and pooling comes before read-count mathematics.
