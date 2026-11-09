## The i9-10900K (10 cores, 20 MiB L3)

| threads | today (0.2.6) | rule A | rule B |
|---|---|---|---|
| 4 | 4 workers | 4 workers | 4 workers |
| 5 | 5 workers | 5 workers | 5 workers |
| 6 | 3×2 | 3×2 | 3×2 |
| 7 | 3×2 | 3×2 | 3×2 |
| 8 | 4×2 | 4×2 | 3×2 |
| 9 | 4×2 | 3×3 pipelined | 3×3 pipelined |
| 10 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 11 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 12 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 13 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 14 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 15 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 16 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 17 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 18 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 19 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 20 | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |

## CPU masks on the i9 (c cores; the L3 stays 20 MiB; the threads asked for = the CPUs in the mask)

| cores | mask | threads | today | rule A | rule B |
|---|---|---|---|---|---|
| 4 | one CPU a core | 4 | 4 workers | 4 workers | 4 workers |
| 4 | both CPUs a core | 8 | 8 workers | 8 workers | 8 workers |
| 6 | one CPU a core | 6 | 3×2 | 3×2 | 3×2 |
| 6 | both CPUs a core | 12 | 3×2 | 3×2 | 3×2 |
| 8 | one CPU a core | 8 | 4×2 | 4×2 | 3×2 |
| 8 | both CPUs a core | 16 | 4×2 | 4×2 | 3×2 |
| 9 | one CPU a core | 9 | 4×2 | 4×2 | 3×2 |
| 9 | both CPUs a core | 18 | 4×2 | 4×2 | 3×2 |

## CI's runners (4 CPUs on 2 cores; macOS and Windows: topology unread)

| threads | today | rule A | rule B |
|---|---|---|---|
| 1 | 1 workers | 1 workers | 1 workers |
| 2 | 2 workers | 2 workers | 2 workers |
| 4 | 4 workers | 4 workers | 4 workers |

## Hypothetical parts, all their cores asked for (threads = 2 x cores)

| cores | L3 | today | rule A | rule B |
|---|---|---|---|---|
| 6 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 6 | 12 MiB | 2×2 | 2×2 | 2×2 |
| 6 | 16 MiB | 3×2 | 3×2 | 3×2 |
| 6 | 20 MiB | 3×2 | 3×2 | 3×2 |
| 6 | 32 MiB | 3×2 | 3×2 | 3×2 |
| 6 | 64 MiB | 3×2 | 3×2 | 3×2 |
| 8 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 8 | 12 MiB | 2×2 | 2×2 | 2×3 pipelined ← |
| 8 | 16 MiB | 3×2 | 2×3 pipelined ← | 2×3 pipelined ← |
| 8 | 20 MiB | 4×2 | 4×2 | 3×2 ← |
| 8 | 32 MiB | 4×2 | 4×2 | 4×2 |
| 8 | 64 MiB | 4×2 | 4×2 | 4×2 |
| 10 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 10 | 12 MiB | 2×2 | 2×2 | 2×3 pipelined ← |
| 10 | 16 MiB | 3×2 | 2×3 pipelined ← | 3×3 pipelined ← |
| 10 | 20 MiB | 3×3 pipelined | 3×3 pipelined | 3×3 pipelined |
| 10 | 32 MiB | 5×2 | 5×2 | 5×2 |
| 10 | 64 MiB | 5×2 | 5×2 | 5×2 |
| 12 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 12 | 12 MiB | 2×2 | 2×2 | 2×3 pipelined ← |
| 12 | 16 MiB | 3×2 | 2×3 pipelined ← | 3×3 pipelined ← |
| 12 | 20 MiB | 4×2 | 3×3 pipelined ← | 3×3 pipelined ← |
| 12 | 32 MiB | 6×2 | 6×2 | 5×2 ← |
| 12 | 64 MiB | 6×2 | 6×2 | 6×2 |
| 16 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 16 | 12 MiB | 2×2 | 2×2 | 2×3 pipelined ← |
| 16 | 16 MiB | 3×2 | 2×3 pipelined ← | 3×3 pipelined ← |
| 16 | 20 MiB | 4×2 | 3×3 pipelined ← | 3×3 pipelined ← |
| 16 | 32 MiB | 7×2 | 5×3 pipelined ← | 5×3 pipelined ← |
| 16 | 64 MiB | 8×2 | 8×2 | 8×2 |
| 32 | 8 MiB | 2×2 | 2×2 | 1×3 pipelined ← |
| 32 | 12 MiB | 2×2 | 2×2 | 2×3 pipelined ← |
| 32 | 16 MiB | 3×2 | 2×3 pipelined ← | 3×3 pipelined ← |
| 32 | 20 MiB | 4×2 | 3×3 pipelined ← | 3×3 pipelined ← |
| 32 | 32 MiB | 7×2 | 5×3 pipelined ← | 5×3 pipelined ← |
| 32 | 64 MiB | 13×2 | 13×2 | 10×3 pipelined ← |

## Monotonicity on a grid (1-64 cores, L3 2-128 MiB in 1 MiB steps, threads 1 to 2 x cores)

| rule | threads used fall (teams) | threads used fall (workers to teams) | modelled throughput falls > 2% | of which a width-3 plan is involved | L3 cliffs to a lower-scored width-2 plan |
|---|---|---|---|---|---|
| today | 0 | 649 | 7057 | 0 | 1 (0.903-0.903) |
| rule A, first version | 0 | 649 | 7031 | 1519 | 40 (0.877-0.955) |
| rule A | 0 | 649 | 6953 | 0 | 57 (0.849-0.955) |
| rule A, partial variant | 0 | 649 | 8424 | 2070 | 0 |
| rule B | 0 | 420 | 0 | 0 | 42 (0.981-0.999) |

## Rule A's one fitted constant: L3 per fast team

| L3 per fast team | 10 c, 20 MiB | 12 c, 20 MiB | 8 c, 16 MiB | 16 c, 32 MiB | 32 c, 64 MiB |
|---|---|---|---|---|---|
| 5.30 MB | 3×3 pipelined | 3×3 pipelined | 3×2 | 7×2 | 13×2 |
| 6.00 MB | 3×3 pipelined | 3×3 pipelined | 2×3 pipelined | 5×3 pipelined | 13×2 |
| 6.99 MB | 3×3 pipelined | 3×3 pipelined | 2×3 pipelined | 4×3 pipelined | 9×3 pipelined |
