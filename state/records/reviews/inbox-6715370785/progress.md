# Pipeline critical-path audit: progress

Inbox 6715370785, resumed by 6729116363. Spec and current master: a4f97714cc14366984015869d9d094d89488ac57.

Migration checks pass; routing audit has only five previously resolved historical misroutes. Receipt fe5827cfff1a2fa9b6279f386e10e15ae7a090d9.

- Production parallel/coop/MTF/RLE/CRC blobs match original assignment e01db6a2895711eb13897fdc378816075d927c1e exactly.
- Important denominator correction: 236.5 overlaps / 26 ms joins describe small-only. Default park+small has 229.5 overlaps and 22.4 ms summed join delay. Neither count measures critical-path exposure.
- FusedBuild::new validates every checkpoint before pool dispatch. At n=900000 there are55 count vectors, but a pipelined3-thread team has2 build shares and consumes only zero, one midpoint and total counts. Sparse boundary validation is a new proposal, not removal of safety obligations.
- Segment assembly is on the last walker, not necessarily the leader; RLE1+CRC is the leader's work on the preceding assembled block. Per-segment CRC must operate after RLE1 and preserve ordered boundary state.
- Service-core decision run is VALID in the journal; one service thread regresses, two at12 are a trade not selected for production. Exploratory svc2@10 merits existing #258 confirmation before duplicate architecture work.
- Replayed240 archived release-scaling rows, one output/hash and no errors. At10 threads v0.2.6/v0.2.5 speed1.03730, paired95% interval[0.98401,1.09349], CPU multiplier1.09415. Separate from lever qualification; no rewrite of that verdict.
- Pipeline-levers raw binary bundle could not be decoded through available text-only GitHub readers (UnicodeDecodeError / UTF-8-only400). Journal summaries remain reported evidence, not independently replayed traces.

Next: finish critical-path sensitivity model, ranked constructions/CPU/risk/CI falsifiers, ledger and one REVIEW citing both IDs. No constructors, compilation or timing.
