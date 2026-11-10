# Delivered

What the team has shipped. One line per merged change that alters performance or behaviour users see, newest first.

Each line gives the measured effect on the primary workloads (`wikidata`, `text`), any documented secondary regression, the release that carries it, and roughly how much engineering it took.

| date | change | effect (CI paired bench, wall) | regressions accepted | release | effort |
|---|---|---|---|---|---|
| 2026-11-10 | #316: guarded service core as the default on the i9-10900K (target-gated) | i9 (target only): throughput +6.8%, CPU −14.7% at 10 threads (service-guarded/20261109T195345Z, 420 runs). Off target: route not taken; CI paired bench neutral | none: inactive off target | next (0.2.8) | large: a qualification design programme of about 4 days, then 1 hour to ship once the policy changed |
