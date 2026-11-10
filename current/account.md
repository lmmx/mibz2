A workstation diagnostic has shown why a parallel scanner is promising only with small decoding tables. On a small-table input, ten plain workers with a parallel block finder reached about 2.56 gigabytes per second, against about 1.6 for the single-scanner version. With the ordinary larger tables, both versions peaked at four workers and slowed by ten. That is a measured clue for a different decoder design, not a qualified production gain.

## What did the scanner test settle?

Louis's [registered run](https://github.com/lmmx/mibz2/pull/145#issuecomment-6757860786) completed validly, and its [result and diagnostic code merged](https://github.com/lmmx/mibz2/pull/317). At ten workers, the finder was 1.615 times the scanner version on the small-table Wikidata input, and 1.780 times on records. On ordinary level-nine Wikidata, it was **not faster**: both arms fell after four workers while processor time rose sharply. The test used one attempt and five rounds per cell. Its thresholds select a construction for further study; they do not qualify production or identify the cause of the remaining large-table limit.

The Pathfinder's first compact [line-table prototype](https://github.com/lmmx/mibz2/pull/323) failed its registered prediction as built: one-thread inversion took 3.665 times production's time. It stays a negative result. A different four-byte-step table has earned only a bounded instruction-count probe from the Optimiser. Its estimated size and arithmetic are not measured speed, and no table currently clears the large-table gap.

## What is happening with guarded service?

The [tooling for the new acceptance test](https://github.com/lmmx/mibz2/pull/319) merged separately from the guarded-service candidate. A two-round dispatch-path smoke test passed its structural checks; its result is explicitly not evidence for qualification. A single, [600-round pilot](https://github.com/lmmx/mibz2/pull/145#issuecomment-6758013659) was then dispatched to learn dependence, variance and timing before the final comparison is frozen. The Researcher had found that the draft's short batches could overstate confidence under strong dependence, and required admission repairs before this pilot. At this account's watermark the pilot was running, with no qualifying verdict.

The guarded-service target run also waits for a separate [two-modes diagnostic](https://github.com/lmmx/mibz2/pull/318). The Researcher found that its first instrument could create misleading signatures, so it is being revised. Neither the pilot nor the diagnostic authorises a target result or adoption.

## What else is open?

An opt-in [pool wait-budget screen](https://github.com/lmmx/mibz2/pull/145#issuecomment-6757184698) passed, but the Guardian [held its proposed measurement-only merge](https://github.com/lmmx/mibz2/pull/145#issuecomment-6757246555): production pool code reads the new field even at the default budget, so neutrality on a CI path without pools does not establish neutrality where it matters. A fresh qualifying registration would be needed for promotion. The corrective build-window change remains merged but unreleased. The [workstation queue](https://github.com/lmmx/mibz2/blob/5e7b9954/bench/workstation-queue.txt) again holds only version 0.2.7 release scaling, with no submitted bundle confirmed.
