# Brief: the serial scanner at high worker counts. A profile first, then a gated parallel scan

*Commissioned by the Coordinator on 24 November 2026. Louis: "yes commission both … be autonomous". It follows the four-arm verdict (journal §44): at 10 workers, supplying the scanner's output saves 19% (C/A 0.806 [0.782, 0.831]), while at 6 workers it saves about 2% and at 1 worker nothing. The scanner reads the bits and does the Huffman decode only; MTF/RLE2, the inversion and emission run in the workers. Assigned to the Pathfinder, in the order below.*

## Part 1 — profile the scanner (a diagnostic; it decides nothing)

**The question:** what is the serial scanner's time made of at 10 workers? Specifically:
- the Huffman bit decode itself;
- table and selector switching;
- block-header and boundary work;
- copying or handing symbols to the job queue;
- time spent blocked on a full queue, or waiting for input.

How much of it is scanner *work*, and how much is *waiting*?

**What to build:** a frozen workstation recipe. It runs the shipped decoder on wikidata ×4 at 1, 6 and 10 workers, instrumented with research-only timers or counters behind a cfg feature; production code is untouched. Report per block and in total:
- scanner busy and idle time;
- bits and symbols decoded per second;
- the share of each component above;
- how many workers sit idle waiting for jobs.

The point is to tell the mathematicians, and part 2, exactly what a cheaper or parallel scanner must beat. The rules (C1a) are a diagnostic rule posted on #145 before the data; it decides nothing. Keep the recipe short, about 10 minutes.

## Part 2 — a parallel scanner gated by worker count (adoption: changes a production default)

**History.**
- The parallel front end of 22 November (#741, `perf/parscan-c37`; registration `docs/journal/2026-11-22-parscan-c37-registration.md`) got about 0.883 wall at 10 workers. It regressed at 6, used more memory, and was not adopted.
- The bounded streaming version (#770, `bench-parscan-stream`) lost badly, at about 1.54× at 10 workers.

The four-arm result explains the first pattern: the scanner only binds at about 10 workers.

**The hypothesis.** A parallel scan, switched on only above a worker-count threshold (with the threshold and its rationale fixed before the data), captures most of the 19% at 10 workers without the 6-worker regression or unacceptable memory.

**What to build.**
- Start from the better of #741 and #770, with the gate.
- If the profile (part 1) shows a cheaper fix, such as queue depth, hand-off or scheduling, propose that instead or as well. Say why in the registration.
- Keep the shipped scanner below the threshold, so behaviour there is byte-identical.

**The registration.** This is an **adoption registration** (CLAUDE.md "Delegated as a class"): frozen before the data, with one review. The Researcher's PROCEED, or one targeted correction followed by the Coordinator's decision. Post it on #145, and the Coordinator routes the review. Suggested shape, which the Pathfinder may sharpen before freezing:
- ADOPTS iff wikidata is FASTER at 10 workers (interval below 1);
- not SLOWER at 1, 6, 8, 12, 16 and 20;
- text not SLOWER anywhere;
- peak RSS ≤ 1.10 at every count.

**Rules throughout:**
- commits as Louis Maddox <louismmx@gmail.com>, with no AI attribution;
- no allocator changes, and no Hub access;
- `src/` changes go behind a feature until adoption;
- frozen recipes are run with `just submit`;
- the Coordinator reviews, merges and sets ready tags under the standing authorisation.
