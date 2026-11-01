# External research report 104: approve the real i9 attribution run (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 10:32 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the attribution dry run.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

`````text
(running now)

````
I would approve the real i9 diagnostic run now. Claude has completed the agreed scope, frozen the implementation, validated the instrumentation and passed the dry run.

This is particularly encouraging because the dry run actually uncovered problems—the idle-time accounting mismatch and `perf` silently substituting `cpu-clock` for `cycles`—which Claude corrected before freezing. That's exactly what we wanted the preparation phase to accomplish.

The production-equivalence check is also reassuring: 104 `mibz2` functions and 32,298 instructions match master with tracing disabled.

### What I think of the readiness

Protocol and implementation frozen: yes

Correctness and corpus checks: passed

Production binary equivalence: passed

Trace accounting validation: passed

Counter failure handling: validated in the VM

Real i9 hardware-counter availability: checked automatically at launch

The last item is the only meaningful environmental unknown. The actual i9 might expose fewer events than expected, but the runner is designed to report that rather than fabricate an attribution. I wouldn't spend another Astra call or require an additional rehearsal just to discover this.

## Run it on the i9

With the browser and other memory-intensive programs closed:

```
cd ~/dev/mibz2
git fetch origin
git switch research/decoder-attribution
git pull --ff-only

cat bench/attribution-pin
# Expected:
# 68956ca9e65168181dbaca8802a3b37b5b25e601

just bench-attribution i9-10900K
```

The script will request `sudo` for the memory-controller counters. It will measure the shipped decoder at one and four workers, using the same four-core allocation and the frozen Wikidata, records and random corpora.

Expected archive:

`~/dev/mibz2/target/attribution-results/i9-10900K.tar.gz`

### What I'm most interested in

The key result is not simply whether the decoder is memory-bound. It's how much of the performance ceiling is attributable to each source.

In particular, I'd want to answer:

- Does build plus walk explain most of the 30% rise in CPU cost?
- Is the missing core utilization caused by head-of-line admission blocking, scanner service, or startup/drain?
- How much does effective clock frequency explain?
- Does scanner service capacity permit the 1.5× target without parallelizing the front end?

If the results clearly identify the limiting resource, we'll have a much stronger basis for our next major engineering decision.

Go ahead with the frozen run. Once it completes, share `result.json` and the full archive if possible. Then I'd bring the evidence back to Astra for its independent architectural assessment—that's where its remaining reasoning budget is most valuable.
````
`````
