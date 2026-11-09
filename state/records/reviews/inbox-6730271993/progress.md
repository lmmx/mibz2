# Pair consumer: source findings (in progress)

Brief [6730271993](https://github.com/lmmx/mibz2/pull/160#issuecomment-6730271993); inspected #272 at 280643cef83b09ac89707b747670e8198b21b68e. Specs a4f97714cc14366984015869d9d094d89488ac57.

1. The current word is L[r] | Psi²(r)<<8, not the Researcher's F-coordinate layout. Bit31 is START, bit30 PAIR_LF. Only bits28–29 are immediately spare. Three symbol bits require a marker redesign.
2. first_byte performs first[r>>shift] then f[c+1] even with zero corrections. The reported .002 corrections/step does not price the ordinary dependent ends load.
3. Current walk already has 32 lanes, flags, next-row L1 prefetch and u16 output stores. Shared-offset rooms and staged compact row state are the absent mechanisms.
4. Ordinary path still updates each lane's output pointer and tests two spare bytes each pair step. A byte-sized 256-byte tile gives 128 pair rounds, not 256.
5. Whole-entry break-even requires walk savings to pay extra build/setup/assembly. 12.4 versus 6.2 Ir/row is not a cycle conversion. The archive's plus-one-c/B build reopening condition has not been demonstrated by those counts.
6. Fresh pinned routing audit: five historical, explicitly resolved issues; no new unresolved routing issue. Service-core D045 HOLD remains scoped to that work. No candidate implemented, compiled, tested or timed.

Next: finish flag-compatible tag derivation, latency/MLP prices, ranked falsifiers and whole-decoder comparison.
