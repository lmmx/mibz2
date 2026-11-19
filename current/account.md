mibz2 0.5.7 is published with a faster way to decode selected blocks and a new six-thread worker choice on Louis’s ten-core machine. The open decoder qualified in a second, separately authorised workstation test; the first test’s NOT ADOPTED result remains part of the record. A downstream Wikidata pipeline update is open, while an independent research effort now starts from information the decoder can obtain cheaply.

## What changed in the release?

The [open decoder](https://github.com/lmmx/mibz2/pull/630) runs on blocks that pass the existing gates for the smaller-memory method; other blocks keep their previous route. The first registered [workstation attempt](https://github.com/lmmx/mibz2/pull/145#issuecomment-7011074912) measured faster Wikidata decoding but failed four individual guard limits, so its verdict was NOT ADOPTED and no code shipped from that attempt. Three of the failing guard corpora never selected open, and all four intervals included no effect. That observation prompted a decision review; it did not overturn the verdict.

Louis then [authorised a proportionate new evaluation](https://github.com/lmmx/mibz2/pull/145#issuecomment-7013839409) with a new rule fixed before its data. The Researcher [reviewed the rule](https://github.com/lmmx/mibz2/pull/145#issuecomment-7016164429). The second run [returned ADOPT](https://github.com/lmmx/mibz2/pull/145#issuecomment-7018269565): the main Wikidata workload took about 5–7% less wall time at eight, ten and twenty threads, held-out Wikidata improved, the registered guard checks passed, and peak memory on those main cells fell. This is a release-build comparison, unlike the earlier isolated kernel tests.

Before release, a required [worker-count recalibration](https://github.com/lmmx/mibz2/pull/145#issuecomment-7019274828) found that at six threads the lazy/open method beat the eager method at the same shipped worker count. [The six-thread change](https://github.com/lmmx/mibz2/pull/642) merged; other counts kept their settings. The Coordinator later [corrected](https://github.com/lmmx/mibz2/pull/145#issuecomment-7019749201) his score of a mathematical prediction about pinned eight threads; that correction did not change the adoption verdict. [Version 0.5.7](https://github.com/lmmx/mibz2/releases/tag/v0.5.7) was published on 19 November.

## What reaches the downstream project?

The [wikidata-pq update](https://github.com/lmmx/wikidata-pq/pull/237) to 0.5.7 is open and must meet that project’s own benchmark rules. Its pipeline benefit is not established here. The earlier 0.5.6 update is already merged; its measured quick sample stayed level, while its full-dump estimate showed a small modeled gain.

The release team is also preparing an i9 comparison of 0.5.7 against 0.5.6. A [moonshot lockfile fix](https://github.com/lmmx/mibz2/pull/644) is in CI because that research feature was red on pull requests after the version bump; [release notes](https://github.com/lmmx/mibz2/pull/643) remain open. These follow-ups do not change what the published adoption run established.

## Why did the evaluation method change?

The first test required every guard’s noisy point estimate to stay below a fixed bound. The Coordinator estimated that a truly unchanged set of guards would fail that family rule about 97% of the time. Louis paused a proposed seven-hour successor and asked for a decision review. The resulting [policy change](https://github.com/lmmx/mibz2/pull/637) requires costly tests to state their decision value and causal relevance, and separates evidence of equivalent work from native timing. The successful new evaluation kept a native wall-time check and an independent review; the first outcome remains unchanged.

## What research comes next?

At Louis’s direction, the team has [opened an acquisition-first programme](https://github.com/lmmx/mibz2/pull/645). It starts with the run records and counts available when decoding begins, then asks what exact reconstruction those facts can support. Every intermediate object must have a derivation and a cost for work, serial depth, bytes, irregular accesses and memory. Four certificates, including a cheap falsifier for the likeliest hidden cost, are required before a proposed decoder reaches benchmarking.

MONODROMY has the [first assignment](https://github.com/lmmx/mibz2/issues/346#issuecomment-7021146886): count how a run partition grows under repeated refinement, then state the minimum information needed for chunked output and either a reachable construction or an obstruction. This is research, not a change to 0.5.7. No action is requested from Louis by this record.
