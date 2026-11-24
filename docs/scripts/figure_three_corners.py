"""The "three corners" figure for the report on formulating the inversion problem (journal §39 and §49 at 0430e0f7).

    python3 docs/scripts/figure_three_corners.py

Prints one inline SVG in the blog's figure style: each family of routes wins one cost by paying
another, and the target in the middle (fewer than half a read per width-1 row, within two
passes, on novel content) has no route yet. Figures are the records' own.
"""

def box(o, x, y, w, h, fill, stroke, title, lines, dash=False):
    o.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" style="fill:{fill};stroke:{stroke};stroke-width:1.5{";stroke-dasharray:6 4" if dash else ""}"/>')
    o.append(f'<text class="bf-t-strong" x="{x+w/2}" y="{y+24}" text-anchor="middle" style="font-size:14px">{title}</text>')
    for i, (t, cls) in enumerate(lines):
        o.append(f'<text class="{cls}" x="{x+w/2}" y="{y+46+i*17}" text-anchor="middle" style="font-size:12px">{t}</text>')


def arrow(o, x1, y1, x2, y2, label, lx, ly, anchor="middle"):
    o.append(f'<path d="M{x1},{y1} L{x2},{y2}" style="stroke:var(--bf-frame);stroke-width:1.6;stroke-dasharray:4 4;fill:none"/>')
    o.append(f'<text class="bf-tm" x="{lx}" y="{ly}" text-anchor="{anchor}" style="font-size:11.5px;font-style:italic">{label}</text>')


def main():
    W, H = 720, 400
    o = []
    box(o, 20, 16, 250, 112, "var(--bf-steel-tint)", "var(--bf-steel)", "Few reads, many passes",
        [("pointer doubling, list ranking", "bf-tm"), ("117 to about 600 passes", "bf-tm"), ("against a budget of two", "bf-tm")])
    box(o, 450, 16, 250, 112, "var(--bf-amber-tint)", "var(--bf-amber)", "Same reads, shorter chains",
        [("lanes, landmark induction", "bf-tm"), ("chain down to 0.07–0.22% of n", "bf-tm"), ("reads per row unchanged", "bf-tm")])
    box(o, 235, 284, 250, 104, "var(--bf-green-tint)", "var(--bf-green)", "Fewer reads where text repeats",
        [("runs, copy plans", "bf-tm"), ("nothing to share on novel", "bf-tm"), ("random IDs, where width-1 rows live", "bf-tm")])
    box(o, 250, 150, 220, 104, "var(--bf-surface)", "var(--bf-ink)", "The target",
        [("under 0.5 reads per width-1 row", "bf-tm"), ("within two passes, all counted", "bf-tm"), ("no route yet", "bf-t-strong")], dash=True)
    arrow(o, 210, 128, 280, 150, "pays in passes", 196, 150, "end")
    arrow(o, 510, 128, 440, 150, "saves waiting, not reads", 524, 150, "start")
    arrow(o, 360, 284, 360, 254, "fails on novel content", 372, 274, "start")
    o.append('<text class="bf-tm" x="20" y="300" style="font-size:12px">Composed routes, every</text>')
    o.append('<text class="bf-tm" x="20" y="317" style="font-size:12px">charge counted: 1.3–2.8 reads</text>')
    o.append('<text class="bf-tm" x="20" y="334" style="font-size:12px">per width-1 row; Open\'s</text>')
    o.append('<text class="bf-tm" x="20" y="351" style="font-size:12px">own walk: 4.3–6.9</text>')
    aria = ("Three families of routes around an empty target. Pointer doubling and list ranking need few unpredictable "
            "reads but 117 to about 600 passes. Lanes and landmark induction shorten the chain of dependent reads to "
            "0.07 to 0.22 percent of the block but leave the reads unchanged. Runs and copy plans save reads only where "
            "text repeats. The target, under half a read per width-1 row within two passes with every charge counted, "
            "has no route yet; composed routes cost 1.3 to 2.8 reads per width-1 row.")
    print(f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>")


if __name__ == "__main__":
    main()
