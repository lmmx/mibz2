# Inbox 6926191785: plan headroom and layout ranking

Status: evidence extraction in progress. Receipt [4516bb2](https://github.com/lmmx/mibz2/commit/4516bb245f4aab88d759d7e092547973bc96cd35).

## Verified first findings

All 29 selected `evidence` archives (including release-scaling 20261110T160612Z) matched adjacent SHA-256 files. The current-plan comparison is `lazy-place/run-20261116T101100Z` at f40eb4cc, whose parent descends from the post-0.5.1 master. Its `-cores1` arm uses nine workers and an in-flight window sized for workers, whereas the 0.5.1 plan uses ten workers with a window sized for requested threads. On Wikidata ×8, paired wall arm/production is 0.984 [0.979,0.989] at 10, 0.954 [0.945,0.964] at 12, and 0.951 [0.945,0.957] at 20. The one-copy Wikidata arm is 0.971 [0.949,0.994] at 10 and 0.888 [0.869,0.906] at 20. A separately forced ten-worker `-cores` arm is at parity on ×8 at 10 and 12; thus the nine-worker/window combination is the key follow-up, but this run does not isolate worker count from window at nine.

The older `lazy-cap/run-20261116T024424Z` records 0.926 [0.900,0.953] at 12, 0.945 [0.924,0.966] at 16 and 0.950 [0.929,0.972] at 20 against its then-production teams. That cap was merged in #468 and is already in 0.5.1; those ratios are not remaining headroom.

The #503 replay measures W12 saving 2.29 bytes/letter for +12.9 writer instructions/letter (0.178 B/extra instruction), W8 saving 4.57 for +9.3 (0.491), but W8 writer wall is 1.16–1.19 of W16 while W12 is 1.01–1.03. W10 and common-child grouping have no measured writer instruction delta, so a four-way measured efficiency rank is impossible. W10 writer replay is the next proposed build after W12, subject to W12's induction-side count.
