"""Draw stage 2a's screen for "The decoder as a queueing network", as static inline SVG.

    python3 docs/scripts/figure_stage2_screen.py STAGE2.tsv [PINNED.tsv]

STAGE2.tsv is math-targets/data/character-stage2/stage2.tsv (journal §74). For each of 6, 8 and
10 workers, stacked, it draws four activities' elasticity of the wall (C, the screen's coefficient for a
+20% slowing), with its 95% interval; an activity the report labels CONTROLS is drawn solid, the
rest pale. Given PINNED.tsv (character-stage2b/stage2.tsv, journal §78), 8 and 10 workers come
from that run instead, with the scanner pinned to its own core and narrower intervals. It prints an
<svg> to paste into the post.
"""

import csv
import sys

ROWS = [("A3", "scanner: Huffman decode"), ("A6a", "workers: construction, first pass"),
        ("A6b", "workers: construction, setup"), ("A7", "workers: emission")]
KS = ["6", "8", "10"]


def read(path):
    data = {}
    for r in csv.DictReader(open(path), delimiter="\t"):
        if r["section"] == "screen":
            data[(r["k"], r["coord"])] = (float(r["C"]), float(r["lo"]), float(r["hi"]), r["label"])
    return data


def main(path, pinned=None):
    data = read(path)
    if pinned:
        data.update({key: v for key, v in read(pinned).items() if key[0] in ("8", "10")})
    W, L, R, rowh, head, gap = 520, 236, 16, 24, 30, 18
    pw = W - L - R
    lo_v, hi_v = -1.0, 1.2
    x = lambda v: L + pw * (v - lo_v) / (hi_v - lo_v)
    panel = head + rowh * len(ROWS)
    H = len(KS) * (panel + gap) + 40
    o = []
    a = o.append
    for j, k in enumerate(KS):
        top = j * (panel + gap) + head
        a(f'<text class="bf-t-strong" x="{L}" y="{top - 10}" style="font-size:15px">{k} workers{" (scanner pinned)" if pinned and k != "6" else ""}</text>')
        for v in (-1.0, 0.0, 1.0):
            a(f'<line class="bf-rule" x1="{x(v):.1f}" x2="{x(v):.1f}" y1="{top}" y2="{top + rowh * len(ROWS)}" opacity="{0.9 if v == 0 else 0.35}"/>')
        for i, (c, name) in enumerate(ROWS):
            C, lo, hi, label = data[(k, c)]
            y = top + rowh * i + rowh / 2
            strong = label.startswith("CONTROLS")
            colour = "var(--bf-amber)" if c == "A3" else "var(--bf-blue)"
            op = 1 if strong else 0.35
            a(f'<text class="bf-tm" style="font-size:14px" x="{L - 12}" y="{y + 5:.1f}" text-anchor="end">{name}</text>')
            a(f'<line x1="{x(max(lo, lo_v)):.1f}" x2="{x(min(hi, hi_v)):.1f}" y1="{y:.1f}" y2="{y:.1f}" style="stroke:{colour};stroke-width:2;opacity:{op}"/>')
            a(f'<circle cx="{x(C):.1f}" cy="{y:.1f}" r="{5 if strong else 4}" style="fill:{colour};opacity:{op}"/>')
    base = len(KS) * (panel + gap) - gap
    for v, s in ((-1.0, "\u22121"), (0.0, "0"), (1.0, "1")):
        a(f'<text class="bf-tm" style="font-size:13px" x="{x(v):.1f}" y="{base + 16}" text-anchor="middle">{s}</text>')
    a(f'<text class="bf-tm" style="font-size:13px" x="{W - R}" y="{H - 4}" text-anchor="end">elasticity of the wall, with 95% intervals</text>')
    c8, c10 = data[("8", "A3")][0], data[("10", "A3")][0]
    aria = ("Stage 2's screen at 6, 8 and 10 workers. At 6, construction (0.14, 0.15) and emission "
            "(0.37) control the wall and the scanner's Huffman decode does not; at 8 and 10 only "
            f"the scanner's Huffman decode controls it ({c8:.2f}, {c10:.2f}).")
    print(f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>")


if __name__ == "__main__":
    main(*sys.argv[1:])
