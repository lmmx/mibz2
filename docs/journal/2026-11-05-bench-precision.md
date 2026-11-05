# 2026-11-05: the bench's precision: A/A noise, and the chained history against a direct run

Two questions about the CI bench, answered from runs it has already made, plus five
dispatches:

1. Does a run's own 95% interval describe its real uncertainty?
2. Does the history's cumulative change, the per-PR links chained together, agree with a
   direct paired run over the same range?

## The noise

### Harness-only A/A

These are 9 balanced links from `bench-results/links.jsonl`: merges that changed the bench
but not the decoder, so each link's true change is zero. A DerSimonian–Laird fit
(`aa_tau.py`) splits their spread into each run's own standard error and τ, the extra
spread between runs that one run's interval can't see:

| `wikidata` | τ (between runs) | median within-run SE | intervals excluding 0 |
|---|--:|--:|--:|
| 1 thread | 0.69% | 0.42% | 2 of 9 |
| 2 threads | 1.04% | 0.74% | 2 of 9 |
| 4 threads | 0.00% | 0.90% | 0 of 9 |

At 1 and 2 threads, τ is larger than one run's own SE. So one run's interval is too narrow
at those thread counts, and 2 of 9 nulls look like changes. At 4 threads the run's own
noise dominates, and its interval is honest.

### True A/A

These are three dispatches of master against itself: run 42086853040 (AMD EPYC 9V45),
42091515843 (EPYC 7763) and 42106179341 (EPYC 9V74). Each had 10 rounds and every corpus:

- **4 of 63** (corpus, threads) intervals exclude zero, **6.3%** against 5% by chance.
  Over all the corpora, the intervals are about right.
- **`wikidata` τ:** 0.00% at 1 thread, 0.00% at 2 and 2.1% at 4. With three runs, this
  says little.
- **Within-run SE** in these runs was 0.91%, 1.01% and 1.59% at 1, 2 and 4 threads: about
  twice the harness-only links'. A true A/A builds both arms from separate checkouts.
- **One flag:** `wikidata` at 4 threads in the third run was −3.3% (−6.1, −0.5), a null
  flagged as a change. `records` at 1 thread had a 21% round-to-round SD in the same run
  and 0.5% in another: the runners' noise is not stable from run to run.

## The chained history against a direct run

v0.2.1 (c217676d) to master (1d0cde37):

| `wikidata` | direct paired run (EPYC 7763, run 42092169426) | the 14 per-PR links chained |
|---|--:|--:|
| 1 thread | **−15.4%** (−17.5, −13.3) | −4.2% |
| 2 threads | **−25.2%** (−26.5, −23.8) | −17.4% |
| 4 threads | **−25.8%** (−27.3, −24.2) | −27.0% |

- **At 4 threads they agree. At 1 thread they are 11 points apart.**
- **The harness is the same at both ends:** `examples/ab.rs` is unchanged over the range.
- **Every decoder-changing merge in the range has a link** except #204, the Huffman
  scanner's register change. Its own PR runs (Intel 8370C, AMD 9V74) measured a few
  percent at 1 thread at most.

**A bisection** (run 42126025530) measured v0.2.1 against the commit just before #204
(66a9dedd), so the range holds only linked PRs. It drew an AMD EPYC 9V74, not the 7763:

| `wikidata` | v0.2.1 → before #204, direct (9V74) | the same PRs' links chained |
|---|--:|--:|
| 1 thread | −8.3% (−8.7, −7.9) | −4.2% |
| 2 threads | −21.5% (−22.2, −20.8) | −17.4% |
| 4 threads | −26.2% (−27.1, −25.4) | −27.0% |

Its stages at 1 thread:

- the walk −10.0%;
- MTF + RLE2 −9.4%;
- the scanner −0.4%;
- RLE1 +5.6% (slower);
- the build −0.2%.

What this shows:

- **The chain misses a third to a half of the 1-thread gain,** even over the linked PRs
  alone: −4.2% chained against −8.3% direct. At 4 threads it doesn't.
- **The rest of the gap, −8.3% to −15.4%, comes from #204 or the CPU, or both.** The
  bisection and the full direct run drew different CPUs (9V74 and 7763), so the two can't
  be separated.
  - Removing a store-and-reload from a serial chain gains more on Zen 3 than on later
    cores. #207's own runs showed it: −5% on the 7763 against −2% on the 9V45 at 1 thread.
  - So #204 is plausibly much larger on the 7763 than on the CPUs that measured it, but
    that is not shown.
- **A likely mechanism, not shown:** each link measures its PR on whatever CPU its run
  drew. The 1-thread gains depend strongly on the CPU (serial-chain latency), and they
  don't carry across models the way the 4-thread gains do.
  - Of the 14 links, 6 ran on Intel or Zen 4/5 and 8 on the 7763.
  - The chain sums 1-thread effects measured on cores where they are smaller.
- **Non-additivity** (one PR's gain depending on another's) is not ruled out either.

## What this changes

- **The history graph's chained line is a trajectory, not a measurement.**
  - At 1 thread it understates the gain since v0.2.1 by a factor of two to three on the
    7763.
  - Release notes and claims about cumulative speed use a direct paired run of tag
    against tag, with its CPU named, as the 0.2.2 notes did. The chain is shown beside it,
    never instead.
- **Replicate across CPU models, not rounds, at 1 and 2 threads.** There τ exceeds one
  run's SE, so more rounds in one run don't narrow the real uncertainty. A second runner
  does. At 4 threads more rounds help, because there the run's own noise dominates.
- **The adaptive ladder:**
  - **Screen** with the PR tier's 10 rounds.
  - **Escalate** a result that is promising, borderline or possibly harmful to a second
    run on a different CPU model, declared before the first run is read.
  - **Choose that run by CPU alone, blind to its result:** read only `machine.cpu` and set
    aside a same-model draw unread, as #207's cross-model check did.
  - **Interpret** a decision figure at 1 or 2 threads with τ added in quadrature to the
    run's SE, at about 0.7% and 1.0%.
- **Not yet done:**
  - a rounds input on the PR tier, which 4-thread figures would need;
  - showing τ in the history's band;
  - a second direct run of v0.2.1 against master on a non-7763 model, which would
    separate #204's effect from the CPU's.

The bisection's branch `bench/bisect-pre204` is left on the remote: the session's proxy
can't delete branches.
