# External research report 81: benchmark artefacts belong in a configurable, git-ignored directory (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:11 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the P74-C i9 stage run's output path.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Agreed. Writing benchmark artifacts directly into `~/p74c-results` is an untidy default, particularly for a reproducible research harness. It leaves generated data outside the repository workspace, makes cleanup less predictable, and risks accumulating stale results across experiments.

I'd prefer a configurable output directory, defaulting to something like `./bench/results/p74c/i9-10900K/`, excluded from Git. The archive should be created there too, and the runner should print its location.

There's also a more important issue in your output:

The i9 run finished successfully, but P74-C has not passed its overall performance gate.

The fitted and held-out Wikidata primary tests both passed. However, the report explicitly says:

- `controls_joint: false`
- `adversarial_joint: false`

So there is a meaningful Wikidata improvement, but the controls and adversarial requirements haven't cleared. We should inspect the results before deciding why.

You've attached the result archive, so I'll examine the detailed measurements rather than infer too much from those summary booleans.

## What your i9 actually measured

I extracted the archive. These are the 20-round paired results for the P74-C arm (`T2`) against shipped (`T0`).

| Corpus            | Plain MTF change | Checkpoint MTF change |
| ----------------- | ---------------- | --------------------- |
| Wikidata fitted   | −18.1%           | −20.9%                |
| Wikidata held-out | −17.6%           | −20.0%                |
| Random            | +0.5%            | \~0.0%                |
| Records           | −0.3%            | −0.1%                 |
| Text              | −17.4%           | −23.3%                |
| Mixed independent | +0.3%            | +1.7%                 |

Negative means faster. These are MTF-stage measurements, not complete-decoder results.

This is actually quite encouraging. On your i9, P74-C preserves a roughly 20% checkpoint-stage gain on both Wikidata datasets while largely eliminating the severe random/records regressions seen with the original branch-free P74.

The control failure needs a more precise interpretation. Random's plain-kernel 95% interval extends to approximately +2.7%, and mixed-independent checkpoint extends to approximately +3.2%. Those are not enormous regressions, but the frozen rules are strict. The adversarial joint check also fails, despite many adversarial cases showing large gains. We should not override either verdict based on attractive averages.

The good news is that the underlying engineering objective—recovering much of P74's Wikidata improvement without broadly damaging high-rank workloads—looks substantially closer on the i9.

## Fixing the output location

Since this is an existing research checkout, I'd prefer:

```
~/dev/mibz2/
  bench/
    results/
      p74c/
        i9-10900K/
        i9-10900K.tar.gz
```

The directory should be ignored by Git, and ideally selectable with something like `P74C_RESULTS_DIR`.

For the results already created, you can move them safely after the run:

```

mkdir -p ~/dev/mibz2/bench/results/p74c
mv ~/p74c-results/i9-10900K* ~/dev/mibz2/bench/results/p74c/

```

Don't delete or overwrite the uploaded archive; it's our original i9 evidence.

One caution: putting results inside the checkout may conflict with the runner's clean-tree or frozen-hash validation unless that directory is excluded appropriately. For a reproducibility-focused harness, a configurable external output directory is also reasonable; the problem is making a cluttering home-directory path mandatory.

My conclusion: the output-path choice is poor, but the actual i9 result is much more promising than the negative summary flags initially suggest. We have a substantial, reproducible-looking checkpoint-stage improvement on the real workstation, with residual acceptance issues that deserve careful review—not post-hoc relaxation of the protocol.
````
