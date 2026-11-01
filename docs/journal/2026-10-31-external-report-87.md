# External research report 87: don't rush the freeze; the two research efforts kept separate (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:40 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the P74-C freeze timing.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
That's the right decision. Don't rush the freeze to catch a ten-minute window. The integration and correctness checks are more important than saving ten hours.
I approve Claude's plan as written. It keeps the two research efforts moving independently:

* P74-C: finish integration, freeze the exact binary and protocol, run exploratory CI, then execute the definitive four-core trial on your i9 when you're available.
* Astra: prepare the BWT-walker machine-code evidence pack independently, so its limited quota can be spent on technical analysis rather than gathering context.

One small operational detail: the proposed housekeeping command moves the old results into `bench/results/p74c/`, whereas the new trial will write to `bench/results/p74c-decoder/`. That's fine; they're different experiments and should remain separate.
I would also retain the original i9 tarball before deleting anything. There's no need to uninstall Rust or clean up the checkout unless you want to.
The one thing I'll pay special attention to when the next report arrives is the exact timed decoder's implementation and affinity. Specifically, I want proof that the shipped and P74-C arms differ only in their MTF choice, and that scanner, feeder and workers are all within the designated four physical cores.
Otherwise, I see no reason to add more instructions or reopen protocol debates.
Next handoff: send me either Claude's frozen-trial report for review or the Astra evidence pack and proposed Codex prompt. We'll handle whichever is ready first, without consuming another round of coordination.

---

louis 🌟 ~ $ mkdir -p ~/dev/mibz2/bench/results/p74c && mv ~/p74c-results/i9-10900K* ~/dev/mibz2/bench/results/p74c/
cd ~/dev/mibz2 && git switch -    # leave the detached 8b6caf1 checkout
Previous HEAD position was 8b6caf1 research(mtf): pin the P74-C confirmatory freeze commit for the runner
Switched to branch 'master'
Your branch is behind 'origin/master' by 46 commits, and can be fast-forwarded.
  (use "git pull" to update your local branch)
louis 🌟 ~/dev/mibz2 $ 


(then did ff pull)
````
