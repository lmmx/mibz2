# Physical i9-10900K PMU verdict: `scanner-contention-residual` (`CONFIRMED`)

*28 Nov 2026. Claim `scanner-contention-residual`, evidence bundle `origin/evidence` `a7ac01d8227a4760c2f277cfb0477fa2a5feaf95` (`scanner-contention/run-20261128T052545Z.tar.gz`, SHA-256 `ce86eaf2b812e3e0beae93684941daf98b72caf9864eba8d37873cd029550d09`), built at `c7fa3284` (`scan-oracle` vs `scan-oracle,forward-anchor-lanes`, distinct binaries verified). Independently re-scored via `python3 bench/scanner_contention.py score` with byte-identical `outcome.txt` and `summary.txt`.*

## Verdict (frozen C1a rule, `docs/journal/2026-11-28-scanner-contention-residual/registration.md`)

**`CONFIRMED`:** `lanes 8W/4W` scanner cycles = **`1.0647 [1.0619, 1.0675]`** (`> 1.000`), and `cycle_activity.stalls_l3_miss:u` explains **`76.4%`** (`+37.64M` of `+49.28M` excess cycles, `>= 50%` threshold).

## Apparatus & validity checks

- **Host:** Physical Intel Core i9-10900K (10 physical cores / 20 hardware threads, all 20 CPUs allowed, `strict_i9 = true`, `stub_pmu = false`).
- **Placement & protocol:** Scanner thread pinned to CPU `0` (`worker_mask = 1..9,11..19`, leaving core `0`'s SMT sibling CPU `10` idle); `12` interleaved rounds per cell at `2 s` settle (`wikidata-x4`, `838,800,784 B`, `1,724` scanned blocks, `0` replayed).
- **Parity:** `4/4` untimed SHA-256 parity runs (`base` and `lanes` at `k = 1, 8`) matched `02949df2160357245d83e635cb155950ad0d31d2b4b8703d8e2d1be0bd3577bf` (`838,800,784 B`, `bmi2 = true`).
- **Instruction & clock invariance:** Scanner retired instructions are invariant across all six cells (`1,426.99M–1,427.02M`, every paired ratio within `0.99997..1.00001`). The `cycles/ref-cycles` ratio cleanly isolates the Comet Lake turbo-bin step between `1W` (`1.4315` / `1.4274`) and `4W` (`1.3267` / `1.3264`) while holding flat from `4W` to `8W` (`1.3258` / `1.3261`).

## Six-cell scanner-thread PMU table (medians over `n = 12` rounds)

| Cell | Scanner cycles (M) | `clk_r` (`cyc/ref`) | IPC | L3-miss stalls (M) | L3 stall % | LLC load misses (M) | Peak RSS (MiB) | Wall (s) | Throughput |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `base/1` | 757.71 | 1.4315 | 1.883 | 18.31 | 2.4% | 0.307 | 78.93 | 1.3121 | 0.64 GB/s |
| `base/4` | 758.44 | 1.3267 | 1.882 | 23.98 | 3.2% | 0.373 | 91.60 | 0.3604 | 2.33 GB/s |
| `base/8` | 774.91 | 1.3258 | 1.841 | 39.36 | 5.1% | 0.501 | 104.35 | 0.1926 | **4.36 GB/s** |
| `lanes/1` | 758.49 | 1.4274 | 1.882 | 18.82 | 2.5% | 0.311 | 79.97 | 1.0645 | 0.79 GB/s |
| `lanes/4` | 762.51 | 1.3264 | 1.872 | 28.03 | 3.7% | 0.413 | 95.00 | 0.2963 | 2.83 GB/s |
| `lanes/8` | 811.14 | 1.3261 | 1.759 | 66.01 | 8.1% | 0.591 | 110.88 | 0.1787 | **4.69 GB/s** |

## Paired scanner-cycle contrasts

| Arm | Contrast | Paired cycles ratio `[95% CI]` | Mean Δ cycles | Mean Δ L3 stalls | L3 share of excess | LLC miss ratio |
|---|---|---|---:|---:|---:|---:|
| `base` | `4W / 1W` | `0.9997 [0.9971, 1.0024]` | `-0.21M` | `+5.66M` | n/a | `1.214×` |
| `base` | `8W / 4W` | **`1.0230 [1.0196, 1.0265]`** | `+17.43M` | `+15.40M` | **`88.4%`** | `1.343×` |
| `base` | `8W / 1W` | `1.0227 [1.0212, 1.0243]` | `+17.22M` | `+21.06M` | `122.3%` | `1.631×` |
| `lanes` | `4W / 1W` | `1.0043 [1.0023, 1.0064]` | `+3.29M` | `+9.16M` | `278.2%` | `1.326×` |
| `lanes` | `8W / 4W` | **`1.0647 [1.0619, 1.0675]`** | **`+49.28M`** | **`+37.64M`** | **`76.4%`** | `1.430×` |
| `lanes` | `8W / 1W` | `1.0693 [1.0666, 1.0720]` | `+52.57M` | `+46.80M` | `89.0%` | `1.897×` |

## Mechanism & design consequences

1. **Turbo bin vs L3 eviction cleanly separated:** At `4W`, `base` executes the exact same cycle count as `1W` (`0.9997×`), confirming `2026-11-28-explore-scanner-inflation-is-clock.md`'s claim that the `1W -> 4W` wall-time step is purely the Comet Lake turbo bin (`clk_r` `1.43 -> 1.33`).
2. **Real `8W` contention is L3 capacity eviction:** Moving from `4W` to `8W` at constant clock (`clk_r = 1.326`) adds `+17.43M` scanner cycles on `base` (`88.4%` L3-miss stalls) and **`+49.28M` cycles on `lanes` (`76.4%` L3-miss stalls, IPC `1.872 -> 1.759`, L3 stall share `3.7% -> 8.1%`)**.
3. **Causal link to `#1217`'s unbounded slab pool:** At `8W`, `lanes` carries `+6.53 MiB` higher peak RSS (`110.88` vs `104.35 MiB`, matching `ccfa473f`'s `+6.71 MiB` from `#1217`'s unbounded `Arena::pool` retaining `~32 × 32 KiB` slabs per worker alongside `out`), pushing the 8 workers + scanner past the i9-10900K's 20 MiB L3 and more than doubling the scanner's L3-miss stall excess (`+15.40M -> +37.64M`, `2.44×`).
4. **Action already live (no duplicate claim filed):** P1 `forward-emission-bounded-slab-rescue` (PR `#1312`, `80 KiB` bounded-slab Arena) directly targets this exact L3 slab-pool lever, eliminating both `ccfa473f`'s RSS excess and the L3 eviction penalty on the 8W scanner. Note also that on post-`#1117`/`#1274`/`#1294` master `c7fa3284` with a pinned scanner, `base/8` already reaches `0.1926 s` (`4.36 GB/s`) and `lanes/8` reaches `0.1787 s` (`4.69 GB/s`).
