# Sparse source precompilation with the ordinary contracted trie

The native immediate-branch policy recovered the ordinary physical trie exactly on all ten real inputs at named, h4 and h12. It removes 58,329 h4 rank records and 65,151 h12 records while performing only 80 and 100 fewer random discovery lookups respectively. The complete 24,906-case tiny marked suite, six modes on every one of 228 admissions, all 3,600 timed outputs and 1,368 fresh-process hashes passed. [Rank records and actual queries](metrics.csv) · [Small adversaries](small.txt) · [All admissions](admissions.csv)

| Precompiled / ordinary | Ordinary → sparse index records | Ordinary and sparse physical trie nodes | Complete one-core inverse CPU | Combined second index and discovery CPU |
|---|---:|---:|---:|---:|
| Named | 84,317 → 75,570 | 40,469 | 1.05630 [1.01925, 1.09469] | 1.20296 [1.12486, 1.28648] |
| Explicit h4 | 84,427 → 26,098 | 34,994 | 1.02136 [0.96366, 1.08251] | 1.22286 [1.18505, 1.26188] |
| Explicit h12 | 84,641 → 19,490 | 39,064 | 1.03717 [1.00099, 1.07466] | 1.24364 [1.17765, 1.31333] |

The full h4 interval spans parity. The nominal h12 interval's lower edge is 1.00099; an effect this close requires independent worker evidence for a production claim. The full h4 and h12 comparisons against named Compact were 1.10046 [1.05757, 1.14508] and 1.10997 [1.07466, 1.14644]. This construction restores ordinary trie size and avoids the earlier full precompiler's large slowdown. It still pays recognition, two more source-boundary coordinates and the combined cost of building the smaller rank index and streaming selectors. [Raw CPU](phase.csv) · [Matched cluster results](summary.txt) · [Decision protocol](PLAN.md)

| Policy | Maximum reusable capacity including output, bytes | Single fresh process maximum RSS, KiB |
|---|---:|---:|
| Named / sparse | 8,473,542 / 8,634,798 | 8,220 / 8,620 |
| h4 / sparse | 7,896,294 / 7,764,038 | 8,168 / 8,308 |
| h12 / sparse | 7,934,870 / 7,801,894 | 8,284 / 8,080 |

The h12 sparse variant is a separate inverse-stage memory point. The fixed-child follow-up uses an algebraic consequence of the producer to avoid all-type recognition rather than carrying the full selector-discovery machinery forward. [All per-process peaks](rss.csv) · [Frozen literal screen](../literal/PLAN.md)
