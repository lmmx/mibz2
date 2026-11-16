mibz2 0.5.1 is published. On Louis's ten-core workstation, the decoder now gives separate blocks to workers from six threads onward, using a smaller-memory method from eight and limiting the worker count to one per physical core above ten threads. That replaced the previous team-of-workers plan at high thread counts after measured wins. The downstream Wikidata pipeline still caps decoding at four threads, so whether it benefits depends on a separate cap test; mathematical work on the decoder's induction continues independently.

## Why did the high-thread plan change?

The [smaller-memory method](https://github.com/lmmx/mibz2/pull/145#issuecomment-6903318530) keeps records only for letters that have acquired nontrivial words. It costs more instructions but less memory traffic. On Louis's workstation it beat the existing teams at ten threads. A first confirmation run failed a predeclared sanity check, so it [decided nothing](https://github.com/lmmx/mibz2/pull/145#issuecomment-6905312941). A [valid repeat](https://github.com/lmmx/mibz2/pull/145#issuecomment-6905778670) supported the ten-thread switch and kept the changeover to the smaller-memory method at eight threads. The team then [merged the ten-thread plan](https://github.com/lmmx/mibz2/pull/462).

Above the physical cores, starting still more workers was slower. A [capped comparison](https://github.com/lmmx/mibz2/pull/145#issuecomment-6906090872) found that one worker per core beat the old teams at twelve to twenty requested threads, and a [longer-file test](https://github.com/lmmx/mibz2/pull/145#issuecomment-6906620039) confirmed that result. The cap [merged](https://github.com/lmmx/mibz2/pull/468). On that longer file workers were busy about 91% of the time. The team looked for another cheap memory reduction but [did not find one](https://github.com/lmmx/mibz2/pull/145#issuecomment-6907104251) in its desk analysis. The next engineering question is how to reduce each worker's instruction cost; the Pathfinder has that question queued.

## What is the research doing?

MONODROMY studies the mathematical structure of the induction separately from production tuning. It found that taking zero-cost steps early cuts peak live symbolic state from about 0.32 to 0.11 times the number of source runs on the measured blocks. It [proved](https://github.com/lmmx/mibz2/pull/145#issuecomment-6900899853) that the zero-cost closure is independent of the order of those steps. This does not make a faster decoder automatically: an engineering [price check](https://github.com/lmmx/mibz2/pull/145#issuecomment-6904481722) found that changing the current kernel's step order saves too little total traffic to clear its rule, because it needs additional indexing.

Other mathematical and prototype routes have closed with specific limits. A parallel compaction prototype was byte-correct but [too expensive](https://github.com/lmmx/mibz2/pull/145#issuecomment-6901428255) for a block on a few cores. Structural rules for repeated words were found, but their implementation price did not justify another production arm in this round. Those findings remain research, not speed qualifications.

## What reaches users next?

The [0.5.1 release](https://github.com/lmmx/mibz2/pull/145#issuecomment-6912414999) has passed its release and tag checks. Its four-core paired comparison was level, as expected for changes that activate at eight or more threads. Louis has [started a release-scaling run](https://github.com/lmmx/mibz2/pull/145#issuecomment-6912478712) at the tag; no result was covered when this account was settled. A fresh crossover confirmation is also queued at the tag.

The [wikidata-pq update](https://github.com/lmmx/wikidata-pq/pull/227) is open and has been retargeted to 0.5.1. Its current cap of four keeps the old team plan. Its branch now prepares a four-versus-six/eight/ten/twenty-thread cap run before the version ladder. That run, rather than decoder-only speed, will decide whether a higher cap helps the whole pipeline. No downstream gain is established here.

## What remains uncertain?

The workstation comparisons describe Louis's ten-core machine and their tested files. They do not settle machines with more than ten physical cores. The first crossover confirmation is still invalid under its frozen sanity rule even though a later repeat supports the change. The 0.5.1 tag's release-scaling and crossover results, and the downstream cap result, were not covered at this watermark. The research width counts describe live symbolic state, not the memory or speed of an implemented alternative.
