# D083 screen: audit in progress

Brief 6759587407, draft ffb568913ce9bd6eeb807082ac935570e36c3c8d. Migration passed.

Observed benchmark run 42758830562 / job 129639441404: cached Rust target (~64MB), cached windows (~12MB); head build27.08s, base25.14s; lbzip2-only install24.83s. This is not a Valgrind CI measurement. Proposed warm subtotal220–300s; cold465–845s before omitted costs. Parallel merge-latency increment=max(0, Qscreen+Tscreen−existing critical completion); never unconditional zero.

Source check: `outputs` uses `.get` with no schema/required keys; four empty objects return A1 ok / exit0 (executed parser only). Matching outputs do not independently assert intended corruption failed or valid inputs succeeded. Must distinguish canonical errors/block identities, pins, corpus hashes, exact arm records and oracle before a blocking screen.

Preliminary ranking: bounded native team/plain coverage first, whole instruction profiles in separate diagnostic/full tier. Symbol presence does not prove execution; sampling absence does not prove nonexecution. Exact native semantic probes need separate instrumented-binary scope and induced cost. No decoder built or run, no timing performed.
