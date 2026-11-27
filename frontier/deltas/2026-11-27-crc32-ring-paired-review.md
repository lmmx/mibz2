claim: none
by: chatgpt
outcome: done
evidence: PR #1179 head beec702c261b480c99b7f82f9fcdb21e88f2b755, base 8e6c6ee; CI run 46028696789, artifact 15213167830 (reported sha256 ad77b035667f040288e73d0f296dd8793aeef067ba47886ad0684144f171d56c); frontier/claims/crc32-avx2-ring-barrett-build.toml

**Question.** Does the AVX2 ring CRC with software prefetch meet its frozen whole-decoder gate at the current PR head?

**Result.** The paired run on the AMD EPYC 9V45 returned identical output in all 21 cells. All six primary wall-time intervals include zero. On pattern, the wall-time estimates were +5.6% at two threads (95% interval −5.4% to +17.9%) and +3.6% at four (−3.5% to +11.1%). Both exceed the claim's 2% secondary point bound. Four-thread peak RSS grew 7.1% on pattern and 6.9% on Wikidata, beyond the 0.5% limit. The six release-mode CRC unit tests also passed independently on this head.

The earlier run cited in the branch delta, 46023411359, belongs to commit 0741c7d. The later run supplies the measurements for the current head. [Paired report and all per-cell results](https://github.com/lmmx/mibz2/pull/1179#issuecomment-7221868387).

**Disposition.** The prefetch implementation misses the registered bounds. Preserve that result when assessing the distinct, already proposed no-prefetch arm. Its local kernel gains justify comparing that arm under the same guards. These whole-decoder measurements leave the cause of the RSS growth and timing differences unresolved.
