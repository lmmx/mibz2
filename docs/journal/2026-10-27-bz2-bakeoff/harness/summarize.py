"""summarize.py RESULTS.jsonl [REF_SHA]: a Markdown table per workload x thread count.

Timed runs are runs 1.. (run 0 hashed the output). c/B is cycles per decompressed byte at
the 2.1 GHz TSC: wall c/B = wall x 2.1e9 / bytes, CPU c/B = (user+sys) x 2.1e9 / bytes.
The sha column compares run 0's sha256 with REF_SHA (libbzip2's), or with bzip2's own
record in the same file when there is one.
"""
import json, statistics as st, sys
from collections import defaultdict

GHZ = 2.1e9


def main() -> None:
    recs = [json.loads(l) for l in open(sys.argv[1])]
    ref = {}
    for r in recs:
        if r["decoder"] == "bzip2":
            ref[r["workload"]] = r["runs"][0]["sha256"]
    given = sys.argv[2] if len(sys.argv) > 2 else None
    groups = defaultdict(list)
    for r in recs:
        groups[(r["workload"], r["threads"])].append(r)
    for (w, t), rs in groups.items():
        print(f"\n#### {w}, {t} thread{'s' if t > 1 else ''}\n")
        print("| decoder | wall best / median (s) | MB/s (best) | CPU-s best / median | wall c/B | CPU c/B | peak RSS (MB) | TTFO (s) | sha = libbzip2 | load (1 min) | steal % | exit |")
        print("|---|--:|--:|--:|--:|--:|--:|--:|---|--:|--:|--:|")
        for r in sorted(rs, key=lambda r: min(x["wall_s"] for x in (r["runs"][1:] or r["runs"]))):
            runs = r["runs"][1:] or r["runs"]
            n = r["runs"][0]["out_bytes"]
            walls = [x["wall_s"] for x in runs]
            cpus = [x["cpu_s"] for x in runs]
            ttfo = [x["ttfo_s"] for x in runs if x["ttfo_s"] is not None]
            sha = r["runs"][0]["sha256"]
            want = ref.get(w, given)
            ok = "ref" if r["decoder"] == "bzip2" else ("?" if want is None else "yes" if want == sha else "**NO**")
            steal = [x.get("steal_pct") for x in runs if x.get("steal_pct") is not None]
            loads = r["load1_before"] if isinstance(r["load1_before"], list) else [r["load1_before"]]
            print(f"| {r['decoder']} | {min(walls):.2f} / {st.median(walls):.2f} | {n / min(walls) / 1e6:.1f} | "
                  f"{min(cpus):.1f} / {st.median(cpus):.1f} | {min(walls) * GHZ / n:.1f} | {st.median(cpus) * GHZ / n:.1f} | "
                  f"{max(x['rss_mb'] for x in r['runs']):.0f} | {min(ttfo) if ttfo else float('nan'):.2f} | {ok} | "
                  f"{min(loads):.1f}-{max(loads):.1f} | {(f'{min(steal):.0f}-{max(steal):.0f}' if steal else '-')} | "
                  f"{','.join(sorted({str(x['exit']) for x in r['runs']}))} |")


if __name__ == "__main__":
    main()
