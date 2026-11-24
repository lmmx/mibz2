"""Draw the release-by-release throughput figure for a blog post, as static inline SVG.

    python3 docs/scripts/figure_release_throughput.py SCALING.json HISTORY.json [THREADS]

It reproduces the benchmark history page's "CI, merge by merge" hero at one thread count, in
its Absolute view: the i9-10900K's releases on wikidata, each at the last merge to master that
its tag contains, with its throughput modelled from release-against-release runs (lbzip2's
measured MB/s in the newest run over the release's pooled wall ratio, as bench.js's
releaseSeries has it) and its 95% interval; below, each release against the one before it.
The inputs are the site's docs/site/assets/release-scaling.json and bench-history.json (the
deployed copies are the newest). It prints a <figure> to paste into the post.
"""

import json
import math
import sys

MARKS = {
    # release: (label line 1, label line 2), drawn as a guide over both panels
    "0.3.0": ("0.3.0: Cells ships,", "below 6 threads"),
    "0.5.1": ("0.5.1: Cells", "reaches 20 threads"),
}


def series(scaling, history, T):
    g = next(l for l in scaling["lineage"] if l["corpus"] == "wikidata")
    P, k, R = g["per_thread"][T], g["t975"], g["releases"]
    pos = {r["version"]: r for r in history.get("releases", []) if r["link"] < len(history["links"])}
    pts = []
    prev = None
    for i, v in enumerate(R):
        if P["x"][i] is None or v not in pos:
            continue
        mb = lambda z=0: P["anchor_mbps"] / math.exp(P["x"][i] - z * math.sqrt(P["v"][i]))
        step = None
        if prev is not None:
            d = P["x"][prev] - P["x"][i]
            h = k * math.sqrt(P["v"][prev] + P["v"][i])
            step = [100 * (math.exp(d) - 1), 100 * (math.exp(d - h) - 1), 100 * (math.exp(d + h) - 1)]
        pts.append({"v": v, "link": pos[v]["link"], "pr": history["links"][pos[v]["link"]]["pr"],
                    "gb": [mb() / 1000, mb(-k) / 1000, mb(k) / 1000], "step": step})
        prev = i
    return g, pts


def main(scaling_path, history_path, T="20"):
    scaling, history = json.load(open(scaling_path)), json.load(open(history_path))
    g, pts = series(scaling, history, T)
    n_links = len(history["links"])
    W, L, Rm = 720, 64, 24
    top0, top1 = 40, 260          # throughput panel
    bot0, bot1 = 300, 400         # change panel
    H = 452
    x = lambda link: L + (W - L - Rm) * link / (n_links - 1)
    gmax = math.ceil(max(p["gb"][2] for p in pts) * 2) / 2
    gmin = math.floor(min(p["gb"][1] for p in pts) * 2) / 2
    yt = lambda v: top1 - (top1 - top0) * (v - gmin) / (gmax - gmin)
    steps = [p["step"] for p in pts if p["step"]]
    cmax = max(10, math.ceil(max(s[2] for s in steps) / 5) * 5)
    cmin = min(-5, math.floor(min(s[1] for s in steps) / 5) * 5)
    yb = lambda v: bot1 - (bot1 - bot0) * (v - cmin) / (cmax - cmin)
    o = []
    a = o.append
    # grids and axes
    v = gmin
    while v <= gmax + 1e-9:
        a(f'<line class="bf-rule" x1="{L}" x2="{W - Rm}" y1="{yt(v):.1f}" y2="{yt(v):.1f}" opacity="0.4"/>')
        a(f'<text class="bf-tm" style="font-size:13px" x="{L - 6}" y="{yt(v) + 4:.1f}" text-anchor="end">{v:.1f}</text>')
        v += 0.5
    a(f'<text class="bf-tm" style="font-size:13px" x="{L}" y="{top0 - 22}">GB/s of output, wikidata, {T} threads on one i9-10900K (modelled, 95% intervals)</text>')
    for c in range(cmin, cmax + 1, 5):
        a(f'<line class="bf-rule" x1="{L}" x2="{W - Rm}" y1="{yb(c):.1f}" y2="{yb(c):.1f}" opacity="{0.9 if c == 0 else 0.4}"/>')
        if c % 10 == 0 or c == cmin:
            a(f'<text class="bf-tm" style="font-size:13px" x="{L - 6}" y="{yb(c) + 4:.1f}" text-anchor="end">{c:+d}%</text>'.replace("+0%", "0%"))
    a(f'<text class="bf-tm" style="font-size:13px" x="{L}" y="{bot0 - 10}">each release against the one before it</text>')
    # guides for the marked releases
    for p in pts:
        if p["v"] in MARKS:
            gx = x(p["link"])
            a(f'<line x1="{gx:.1f}" x2="{gx:.1f}" y1="{top0 - 6}" y2="{bot1}" style="stroke:var(--bf-green);stroke-width:1.5;stroke-dasharray:4 3"/>')
    # releases: dotted join, whiskers, diamonds
    a('<polyline points="' + " ".join(f'{x(p["link"]):.1f},{yt(p["gb"][0]):.1f}' for p in pts)
      + '" style="fill:none;stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:2 3"/>')
    for p in pts:
        px = x(p["link"])
        a(f'<line x1="{px:.1f}" x2="{px:.1f}" y1="{yt(p["gb"][1]):.1f}" y2="{yt(p["gb"][2]):.1f}" style="stroke:var(--bf-amber);stroke-width:2;opacity:0.6"/>')
        y0 = yt(p["gb"][0])
        a(f'<path d="M{px:.1f},{y0 - 5:.1f} l5,5 l-5,5 l-5,-5 z" style="fill:var(--bf-amber)"/>')
        if p["step"]:
            s = p["step"]
            a(f'<line x1="{px:.1f}" x2="{px:.1f}" y1="{yb(s[1]):.1f}" y2="{yb(s[2]):.1f}" style="stroke:var(--bf-amber);stroke-width:2;opacity:0.6"/>')
            a(f'<path d="M{px:.1f},{yb(s[0]) - 4:.1f} l4,4 l-4,4 l-4,-4 z" style="fill:var(--bf-amber)"/>')
    # labels for the marked releases, at the top of the panel: the first to the left of its
    # guide, the second to its right, so they never meet
    marked = [p for p in pts if p["v"] in MARKS]
    for j, p in enumerate(marked):
        px = x(p["link"])
        l1, l2 = MARKS[p["v"]]
        side, dx = ("end", -7) if j == 0 else ("start", 7)
        for r, line in enumerate((l1, l2)):
            a(f'<text class="bf-t-strong" x="{px + dx:.1f}" y="{top0 + 8 + 17 * r:.1f}" text-anchor="{side}" '
              f'style="font-size:15px;fill:var(--bf-green-ink)">{line}</text>')
    first, last = pts[0], pts[-1]
    a(f'<text class="bf-t-strong" x="{x(first["link"]) + 8:.1f}" y="{yt(first["gb"][0]) + 18:.1f}" style="font-size:15px">{first["v"]}: {first["gb"][0]:.2f} GB/s</text>')
    a(f'<text class="bf-t-strong" x="{x(last["link"]) - 8:.1f}" y="{yt(last["gb"][0]) - 10:.1f}" text-anchor="end" style="font-size:15px">{last["v"]}: {last["gb"][0]:.2f} GB/s</text>')
    # x axis: merges to master, in order, labelled by pull request
    a(f'<line class="bf-axis" x1="{L}" x2="{W - Rm}" y1="{bot1 + 4}" y2="{bot1 + 4}"/>')
    ticks = [0] + [round(f * (n_links - 1)) for f in (0.2, 0.4, 0.6, 0.8)] + [n_links - 1]
    for t in ticks:
        a(f'<text class="bf-tm" style="font-size:13px" x="{x(t):.1f}" y="{bot1 + 20}" text-anchor="middle">#{history["links"][t]["pr"]}</text>')
    a(f'<text class="bf-tm" style="font-size:13px" x="{(L + W - Rm) / 2:.1f}" y="{bot1 + 42}" text-anchor="middle">merges to master, in order (pull request number); each release sits at the last merge it contains</text>')
    aria = (f"Wikidata throughput at {T} threads on an i9-10900K, release by release from {first['v']} "
            f"({first['gb'][0]:.2f} GB/s) to {last['v']} ({last['gb'][0]:.2f} GB/s), placed by merge to master; "
            "flat near 1.6 GB/s from 0.2.9 to 0.5.0, rising from 0.5.1, when Cells reached 20 threads.")
    svg = f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"
    print(svg)


if __name__ == "__main__":
    main(*sys.argv[1:])
