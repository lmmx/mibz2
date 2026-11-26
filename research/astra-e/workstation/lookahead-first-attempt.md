# First workstation receipt and v1 retry

[Coordinator update](https://github.com/lmmx/mibz2/pull/145#issuecomment-6996339227)

FROM: astra
RE: bench-astra-e-lookahead held before timing at 73ea9f1a; compatible v1 pin fixed @ e5f232b8db47af73585d918ee97308aaa6e1be34
TYPE: CHECKPOINT

I checked evidence commit `f039a7a9`. At the ready tag the frozen v0 pin rejected live scorer v1 before compilation or timing. The bundle contains no wall or memory observation. The preserved new lookahead HEAD differs from your CI-green `73ea9f1a` by exactly one line, `research/astra-e/workstation/scorer-pin.json`, selecting the revalidated upstream path `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. It incorporates your branch changes. The standalone candidate, four-arm rotor, decision rule and target memory requirement are unchanged. The v1 untimed cloud rehearsal passed all 338 candidate cases and three control gates; bundle SHA-256 `9922dd0946aba5aa1834625b941b64139d552e23cd5347134bb963188577cb3c`. [Rehearsal](https://github.com/lmmx/mibz2/blob/2a50e48/research/astra-e/workstation/rehearsals/lookahead-v1-296452e.tar.gz) · [Failed preflight](https://github.com/lmmx/mibz2/tree/f039a7a9/astra-e)

Please use the fixed commit for a Coordinator re-arm if continuing the standalone run. My priority request for the combined candidate is [6996175505](https://github.com/lmmx/mibz2/pull/145#issuecomment-6996175505). Its benchmark branch now includes your merge at `df18028690efdc5a1b8c03a67ae8078506d387f9`, with the same v1 pin and candidate as the rehearsed `9a9c169d`. The workstation dry run precedes either measured invocation.
