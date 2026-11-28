# The bandwidth ledger's evidence

One file a simulated decode: callgrind's command, its first-level and last-level miss totals,
and the functions with the most last-level write misses (`DLmr` read, `DLmw` written, `Dw` all
data writes). The entry is `../2026-11-10-bandwidth-titration.md`.

| file | arm, threads | last-level model | binary |
|---|---|---|---|
| `sim-one.txt` | `mibz2`, 1 | 16 MiB, 16-way | branch head `93235250` (master `64c1eb4`'s decoder) |
| `sim-pipe3.txt` | `mibz2-pipe3-both`, 9 | 16 MiB, 16-way | the same |
| `sim-workers10.txt` | `mibz2-workers`, 10 | 16 MiB, 16-way | the same |
| `sim-team2x4.txt` | `mibz2-team2`, 8 | 16 MiB, 16-way | the same |
| `sim-pipe3-20.txt` | `mibz2-pipe3-both`, 9 | 20 MiB, 20-way | the same |
| `sim-coop3-20.txt` | `mibz2-coop3`, 9 (2 blocks in flight a team) | 20 MiB, 20-way | after merging master `02c2bbfb` (#312's build) |
| `sim-team3-20.txt` | `mibz2-team3`, 9 (4 blocks in flight a team) | 20 MiB, 20-way | the same |

**The window, priced after the entry was frozen** (simulated; the last two rows, which differ in
the blocks in flight alone): 6 slots against 12 write 17.37 M against 17.93 M cold lines (−3%)
and read 0.99 M against 3.28 M. In this model the output window is a small lever on cold writes:
every buffer is still written cold at 6 slots.
