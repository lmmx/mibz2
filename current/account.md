Version 0.2.8 has shipped measured Wikidata and text decoding gains. The next production candidate carries its run-record method into the team paths used at higher thread counts, where the released change brought no workstation gain. That port is held for one missing fuzz-coverage check before its planned workstation comparison. In parallel, MONODROMY has produced a faster local inverse-transform kernel, but it is still a research example, not a decoder change.

## What is being tested for the next release?

A [team-path port](https://github.com/lmmx/mibz2/pull/145#issuecomment-6769926198) would extend the shipped run-record front and backward table to plain teams, the pipeline and guarded service. Its local timings suggest gains, but they are indicative and CI's ordinary bench does not exercise these paths. The [Guardian found](https://github.com/lmmx/mibz2/pull/145#issuecomment-6770483223) that no existing fuzzer forces the new plain-team route. It found no new unsafe-code defect, but held safety completion until that route has explicit fuzz coverage and a recorded reach/error summary. Only then is the planned paired workstation run against master ready to decide whether the port stays.

A separate [move-to-front alignment fix](https://github.com/lmmx/mibz2/pull/358) has already merged. It keeps a hot list inside one memory page and removes a stack-placement hazard that made one build's records/team case roughly 1.4 times slower. Its paired bench was neutral, so the fix is a stability measure, not a measured speedup. The workstation comparison for the port will include this fix on both sides.

## What happened to the new mathematical line?

MONODROMY's [run-induction example](https://github.com/lmmx/mibz2/pull/351) now builds the block's grammar through cells and, in its latest indicative one-thread Wikidata measurement, takes about 0.82 times the current production inversion stage's time. It remains slower on high-run-density and adverse inputs. The example is not called by mibz2, uses unsafe code, and has not established a ten-worker or whole-decoder gain.

That result needs a correction beside it. MONODROMY [found](https://github.com/lmmx/mibz2/pull/145#issuecomment-6770249707) that the cells writer left bytes unwritten while its output check reused a buffer already containing correct bytes; an overlapping-copy call was also unsound. Both were repaired, the output buffer is now cleared before checking, and the earlier favourable timings predate the repair. Its latest PR reports fresh checks and a revised local 0.82 ratio, but one review lens remains open. Sparse-input figures from the defective writer were withdrawn until remeasured. No production qualification follows from the local kernel.

The team [published an explanation](https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/) of the approach after incorporating that correction. The Pathfinder is assigned an independent review of the post. Louis asked the Researcher, Optimiser and Architect to support MONODROMY while leaving it to choose its next construction.

## What has been delivered or closed?

The version 0.2.8 CI comparison found Wikidata decoding 12–19% and text 10–26% faster across one to four threads. The earlier registered no-harm failure of the shipped run-record path remains recorded alongside Louis's explicit adoption decision. The three cooperative ports and the universal backward walk closed as built under their fixed rules. The admission fix retained from the universal walk has since [merged](https://github.com/lmmx/mibz2/pull/353) as maintenance.

Louis submitted a version 0.2.8 workstation scaling bundle, but no checked result is in the channel at this account's watermark. Its downstream wikidata-pq effect is also unconfirmed.
