# Corrected-head Occupancy CI evidence

PR #368 head `bfd5d322651974f9803f4c03942cf3ba3b5161d2`, assignment #363/6789867142.
Run [42984639389](https://github.com/lmmx/mibz2/actions/runs/42984639389),
job130402788066, checkout merge `fa03b46de14cd507e9672462e653f5343f806365`.
Instrument tests pass99 library +5 occupancy (one census ignored). Both ordinary
and forced-team steps succeed; events reconcile. Artifact13838404688 ZIP SHA256
`f2e7c1d0d421a858e8efde116aa9bf3d2e06650552746a036022589d251af2e7` verified after download.
The exact ZIP and complete job log are gzip-wrapped here; no source-head change.

Raw JSON checks:

- occupancy.json: parity; 36 complete traced runs; 15516 snapshots; zero worker-ledger violations; plans ['workers1', 'workers2', 'workers4']
- occupancy-teams.json: parity; 4 complete traced runs; 1724 snapshots; zero worker-ledger violations; plans ['teams2x2']

On AMD EPYC7763, 4 allowed logical CPUs/2 physical cores, ordinary timeline overhead
intervals are [-5.6,+6.7], [-1.2,+13.0], [-1.0,+2.1]% at1/2/4t; full intervals
[-6.0,+6.1], [-3.9,+6.2], [-2.1,+2.0]%. Forced2x2 timeline[-18.3,+23.7]% and
full[-42.9,+62.1]%, only two rounds. Every interval includes zero; no precise claim of
zero overhead. This is CI instrument evidence, not an i9 result or an independent
Guardian clearance. D100 and #366 readiness still require the targeted disposition.

The initial failed run42983415379 remains archived in the PR's immutable handoff.
Its Class A SIMD slack omission was corrected in source, not suppressed or rerun at
the unchanged head. Benchmark/Miri completion is separate from this Occupancy result.
