"""Summarise the scan-profile bundle's runs.tsv into measurements-scan-profile.tsv, one row per worker count.

Usage: python3 extract_scan_profile.py data/scan-profile-runs.tsv > data/measurements-scan-profile.tsv

Wall, memory, CPU and context switches are medians over the untimed decodes (timed = 0), which carry no
per-block timers. Scanner and worker times are medians over the timed decodes (timed = 1).
workers_busy_s in runs.tsv is the sum over the k workers; it is divided by k here.

Derived columns:
- rho = (workers' total busy) / (k * scanner work), the report's definition (the scanner's rate over
  the workers' combined rate): above 1 the workers bind, below 1 the scanner does.
- efficiency = max(scanner work, one worker's busy) / timed wall: how close the decode comes to its
  busiest server's demand. 1 means no time is lost to the queueing discipline at that count.
- rss_own_peak_mb = peak RSS less the process's base (the input and the runtime).
"""
import csv, statistics as st, sys
from collections import defaultdict

untimed, timed = defaultdict(list), defaultdict(list)
with open(sys.argv[1]) as f:
    for r in csv.DictReader(f, delimiter="\t"):
        (timed if r["timed"] == "1" else untimed)[int(r["workers"])].append(r)

def med(rows, col):
    return st.median(float(r[col]) for r in rows)

cols = ["source", "workers", "n_untimed", "n_timed", "wall_untimed_median_s", "wall_timed_median_s",
        "scanner_work_s", "scanner_slot_wait_s", "worker_busy_s_each", "worker_idle_s_each", "rho",
        "efficiency_vs_busiest_server", "rss_own_peak_mb", "rss_own_mean_mb", "user_s", "sys_s",
        "cpu_over_wall", "minor_faults", "vol_ctxsw", "invol_ctxsw"]
print("\t".join(cols))
for k in sorted(untimed):
    u, t = untimed[k], timed[k]
    scan, busy_all = med(t, "scanner_work_s"), med(t, "workers_busy_s")
    busy = busy_all / k
    wall_t = med(t, "wall_s")
    base = med(u, "rss_base_kb")
    row = ["scan-profile/run-20261124T223748Z", k, len(u), len(t),
           f"{med(u, 'wall_s'):.4f}", f"{wall_t:.4f}", f"{scan:.4f}", f"{med(t, 'scanner_slot_wait_s'):.4f}",
           f"{busy:.4f}", f"{med(t, 'workers_idle_s') / k:.4f}", f"{busy_all / (k * scan):.3f}",
           f"{max(scan, busy) / wall_t:.3f}",
           f"{(med(u, 'rss_peak_kb') - base) / 1024:.1f}", f"{(med(u, 'rss_mean_kb') - base) / 1024:.1f}",
           f"{med(u, 'user_s'):.3f}", f"{med(u, 'sys_s'):.3f}",
           f"{(med(u, 'user_s') + med(u, 'sys_s')) / med(u, 'wall_s'):.2f}",
           int(med(u, "minflt")), int(med(u, "nvcsw")), int(med(u, "nivcsw"))]
    print("\t".join(map(str, row)))
