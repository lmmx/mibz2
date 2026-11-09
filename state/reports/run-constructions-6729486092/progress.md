# Progress: run-based constructions

Assignment #162/6729486092, relayed #145/6729427506. Specs a4f97714cc14366984015869d9d094d89488ac57. Receipt cf3a9541c7e8a7d7c63d21fae10d016f029a9f23. Research in progress; not a performance verdict.

Read #266 census and #269 at 187c0ba517c3324aa28c5e7ccd2667f6c0572145; both 8 November blog posts at master snapshot 31e6a0c5e1e68db64555ee9c630dc679c3ca8d18; #34 dense pair and #36 Precopy negative records. Inspected libsais c284ba843dd16c159acb976f17bc19382793ea75 symbol_rows/OpenMP. Literature: Nishimoto/Tabei 2021; Brown/Langmead SEA2026; Brown/Sanaullah/Zhang/Langmead arXiv2603.22147v2; Karkkainen/Puglisi2011; Karkkainen/Kempa/Puglisi2012; Ferragina/Gagie/Manzini arXiv0909.4341v1. Page references being checked.

Leading candidates: off-leader run sorting plus existing word-table walk; run-intersection construction of dense two-hop table without per-row bigram histogram/scatter; exact tiled stores. Pair walker itself already failed #34: not a new discovery. New analytical variant to check: 20-bit pointer + second byte + three first-symbol low bits + marker fits one u32. First-symbol high bits from a 256-row directory, slow fallback only in directory blocks crossing a group-of-eight symbol boundary. At most31 such boundaries, at most31*255 exceptional rows per complete distinct-row traversal, not necessarily per periodic/segmented execution. No claimed measured saving. No implementation or timing.

Coord audit on fetched five channels has only five historical findings already marked resolved by pinned helper. Optimiser's assignment read; no substantive response to it present in fetched channel snapshot. Will recheck once before publication.
