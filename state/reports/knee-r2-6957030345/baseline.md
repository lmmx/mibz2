# Independent evidence baseline, round 2

Trigger 6957030345; reviewed revision afc5c5ab5fb4cd1c9f4fee838417693e42c08ff0. Written before opening revised article or PR550/551 bodies this run. This is not a blinded replication: the same reviewer read round 1 and already knows the held-out question. Sources reread: all 16 checked-in release-scaling runs.jsonl and meta.json, release-attribution ledger at the reviewed revision, LOGs 6946756938, 6952656453, 6948385717, 6949008682, archived knee and cells-imc primary records. Local archives rehashed to b145bb1e596b719eae1cdb0a13168dd4b6c8378f42bae42cdf1a323a7792792d and 2661310a6622534408bd440717d96f171d1ee25d6850064010fd639c1063f2cb.

## Derivation before prose

0.5.4 median wall for 1/2/4/8/10/12/16/20 requested threads: .38510/.24115/.14365/.10470/.10200/.10580/.10615/.10755 s. Output 209700196 B. Thus high counts yield 1.950–2.056 decimal GB/s. Requested threads are not active decoder workers: 7 from 10 in the measured whole-machine mode, 7 at pinned10, 6 at pinned8, 3 at pinned4. At 12–20 the cap is already in 0.5.3. Pinned and unpinned are different placements; affinity selects an allowed set, not exclusive per-thread placement. Full-machine low counts do not take the small-count trims.

The old knee is not universal over releases. 0.2.5–0.3.3 have ten faster than eight; 0.2.4 plateaus; 0.4.0–0.5.0 have the sharper high-count fall. Selected histories must be labelled selected. The ratio of 0.5.4 medians 20/10 is 1.0544; not evidence of a monotonic hardware scaling law. Four-thread wall improvement is unresolved; eight and ten improve in paired release comparisons, about 4.3% and 10.9% less wall.

Raw knee rows reproduce x4 unpinned walls .40535/.37565/.36610/.36565/.36640 for requested4/8/10/12/20. Read proxies/output byte 6.892/12.648/11.057/11.032/11.018. Seven-worker plateau may reflect unchanged work allocation, not failure of twenty active workers. Worker busy about .93–.95; idle .010–.015. Lack of starvation is not proof of bandwidth saturation. Current IMC unavailable. Proxy reads 25.2–25.3 GB/s against 37.592 read proxy in triad; old-kernel IMC writes/reads .743–.784 at8/10 implies total44–45 GB/s only if its ratio transfers. 51.2 GB/s is assumed nominal DDR4 throughput, not measured sustainable mixed-traffic ceiling. Cache/memory contention remains consistent, cause not isolated.

## Pre-answer to known transfer question

If next release halves DRAM bytes/output byte, bandwidth roof B/b doubles at unchanged sustainable bandwidth B. Actual throughput rises only if that roof binds and the change does not introduce another cost. Y=min(B/b, worker compute/latency throughput, front/consumer limits) is a simplified bottleneck model, not a measured identification. Dependent cache-miss latency, insufficient memory-level parallelism, compute, placement, worker cap or newly exposed serial work can leave the plateau unchanged or move it by less than 2x. A halving of L3-miss read proxy alone is not necessarily halving total DRAM traffic. Need current read+write counters, a comparable sustainable ceiling and preferably a controlled traffic-reducing intervention to distinguish mechanisms. Seven is a measured configuration policy, not a portable optimum.

## Provenance and boundary

Migration exact old head and manifest SHA256 verified. Inbox pages1–2 checked oldest first: only this comment unfinished. coord.py wake audit since6956224553 returned []. No timing experiment, implementation, merge, adoption or automation change. Prior judgments are hypotheses to retest, not authority.
