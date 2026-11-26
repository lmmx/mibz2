"""Figures first drawn for the ideal-parts post. "Where mathematics can make decompression faster" now
uses only `pipeline` from here (docs/scripts/figures_where_mathematics.py draws the rest).

    python3 docs/scripts/figures_ideal_parts.py NAME

NAME is one of: interaction, pipeline, budget, rauzy. Each prints one inline SVG in the blog's
figure style (classes bf-tm, bf-t-strong, bf-rule; colours from the --bf-* tokens). Sources:

- interaction: ideal-parts/results/full/report.txt (the ceiling table: C alone, S alone, and C
  with S ideal) and README.md §4 (the all-real walls 1,306, 243, 195 and 196 ms). The milliseconds
  are the ratios times those walls, as plan.md's table gives them: S+C is S's wall times "C with S
  ideal".
- pipeline: ideal-parts/parts.md and the review's Mermaid diagram (reviews/2026-11-26-chatgpt.md §1).
- budget: math-targets/data/character-stage1/report.txt, the one-worker row (seconds a decode, the
  mean worker): A5 0.1324, A6a 0.2003, A6b 0.2160, A6c 0.1614, A7 0.4903, A9 0.0760, worker busy
  1.2801, wall 1.2831.
- rauzy: monodromy/2026-11-10-rauzy-veech.md §5.6 (steps per run 1.812, 1.279, floor 1.220) and the
  budget arithmetic in reviews/2026-11-26-chatgpt.md §6B, with the 1,306 ms matrix wall.
"""
import math
import sys

W = 720
REAL = {1: 1306, 6: 243, 8: 195, 10: 196}
S_ALONE = {1: 1.0101, 6: 0.9740, 8: 0.9683, 10: 0.9193}
C_ALONE = {1: 0.5792, 6: 0.7540, 8: 0.9437, 10: 0.9467}
C_WITH_S = {1: 0.569, 6: 0.605, 8: 0.661, 10: 0.825}


def svg(h, body, aria, w=W):
    return f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{aria}">' + "".join(body) + "</svg>"


def text(o, x, y, s, size=12, cls="bf-tm", anchor="start", extra=""):
    o.append(f'<text class="{cls}" x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" style="font-size:{size}px{extra}">{s}</text>')


def box(o, x, y, w, h, fill, stroke, rx=6):
    o.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" style="fill:{fill};stroke:{stroke};stroke-width:1.5"/>')


def arrow(o, x1, y1, x2, y2, colour="var(--bf-frame)", dash=""):
    o.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" style="stroke:{colour};stroke-width:1.8{";stroke-dasharray:" + dash if dash else ""}"/>')
    a = math.atan2(y2 - y1, x2 - x1)
    p = [(x2, y2), (x2 - 9 * math.cos(a) + 5 * math.sin(a), y2 - 9 * math.sin(a) - 5 * math.cos(a)),
         (x2 - 9 * math.cos(a) - 5 * math.sin(a), y2 - 9 * math.sin(a) + 5 * math.cos(a))]
    o.append(f'<polygon points="{" ".join(f"{u:.1f},{v:.1f}" for u, v in p)}" style="fill:{colour}"/>')


def cells(k):
    real = REAL[k]
    s = real * S_ALONE[k]
    return {"real": real, "C": real * C_ALONE[k], "S": s, "SC": s * C_WITH_S[k]}


def interaction():
    """Four small interaction plots, two by two: wall (ms) against the scanner real or supplied, one
    line for construction real and one for construction supplied. Parallel lines mean the two act
    independently; lines that diverge mean each matters more once the other is relieved."""
    o = []
    w = 520
    text(o, 16, 20, "Wall time in ms; the scanner real or supplied, construction real or supplied", 12.5, "bf-t-strong")
    pw, ph, gx, gy, top = 228, 132, 32, 62, 64
    for j, k in enumerate((1, 6, 8, 10)):
        c = cells(k)
        x0 = 20 + (j % 2) * (pw + gx)
        y0 = top + (j // 2) * (ph + gy)
        lo, hi = min(c.values()), max(c.values())
        pad = (hi - lo) * 0.18
        lo, hi = lo - pad, hi + pad
        y = lambda v: y0 + ph * (hi - v) / (hi - lo)
        xa, xb = x0 + 52, x0 + pw - 52
        text(o, x0 + pw / 2, y0 - 14, f"{k} worker{'s' if k > 1 else ''}", 13.5, "bf-t-strong", "middle")
        o.append(f'<line class="bf-rule" x1="{x0 + 8}" x2="{x0 + pw - 8}" y1="{y0 + ph + 6}" y2="{y0 + ph + 6}"/>')
        text(o, xa, y0 + ph + 22, "S real", 12, anchor="middle")
        text(o, xb, y0 + ph + 22, "S supplied", 12, anchor="middle")
        for (a, b, col) in ((c["real"], c["S"], "amber"), (c["C"], c["SC"], "blue")):
            o.append(f'<line x1="{xa}" x2="{xb}" y1="{y(a):.1f}" y2="{y(b):.1f}" style="stroke:var(--bf-{col});stroke-width:2.5"/>')
            for xx, v in ((xa, a), (xb, b)):
                o.append(f'<circle cx="{xx}" cy="{y(v):.1f}" r="4.5" style="fill:var(--bf-{col})"/>')
            text(o, xa - 8, y(a) + 4, f"{a:,.0f}", 12.5, anchor="end")
            text(o, xb + 8, y(b) + 4, f"{b:,.0f}", 12.5, "bf-t-strong" if col == "blue" else "bf-tm")
    ly = top + 2 * (ph + gy) - 14
    o.append(f'<line x1="70" x2="94" y1="{ly - 4}" y2="{ly - 4}" style="stroke:var(--bf-amber);stroke-width:2.5"/>')
    text(o, 100, ly, "construction real", 12.5)
    o.append(f'<line x1="270" x2="294" y1="{ly - 4}" y2="{ly - 4}" style="stroke:var(--bf-blue);stroke-width:2.5"/>')
    text(o, 300, ly, "construction supplied", 12.5)
    aria = ("Interaction plots of wall time at 1, 6, 8 and 10 workers. At 8 workers: everything real 195 ms, "
            "the scanner supplied 189, construction supplied 184, both supplied 125. At 1 worker: 1,306, "
            "1,319, 756 and 751. At 6: 243, 237, 183 and 143. At 10: 196, 180, 186 and 149.")
    return svg(ly + 12, o, aria, w)


def pipeline():
    o = []
    text(o, 16, 20, "One block's path through the decoder, and the shared memory every stage leans on", 13, "bf-t-strong")
    y = 60
    box(o, 20, y, 120, 58, "var(--bf-amber-tint)", "var(--bf-amber)")
    text(o, 80, y + 24, "S  scan", 13, "bf-t-strong", "middle")
    text(o, 80, y + 42, "one thread", 11.5, anchor="middle")
    arrow(o, 142, y + 29, 168, y + 29)
    box(o, 170, y + 8, 80, 42, "var(--bf-surface)", "var(--bf-frame)")
    text(o, 210, y + 27, "scanned", 11.5, anchor="middle")
    text(o, 210, y + 41, "blocks", 11.5, anchor="middle")
    arrow(o, 252, y + 29, 276, y + 29)
    box(o, 278, y - 6, 300, 70, "var(--bf-blue-tint)", "var(--bf-blue)")
    text(o, 428, y + 12, "each worker, one block at a time", 11.5, anchor="middle")
    for i, (lab, sub) in enumerate((("M", "MTF, RLE2"), ("C", "construction"), ("E", "emission"), ("R", "RLE1, CRC"))):
        bx = 290 + i * 72
        box(o, bx, y + 20, 62, 38, "var(--bf-surface)", "var(--bf-blue)", 4)
        text(o, bx + 31, y + 36, lab, 13, "bf-t-strong", "middle")
        text(o, bx + 31, y + 51, sub, 9.5, anchor="middle")
        if i:
            arrow(o, bx - 9, y + 39, bx - 1, y + 39, "var(--bf-blue)")
    arrow(o, 580, y + 29, 604, y + 29)
    box(o, 606, y, 96, 58, "var(--bf-green-tint)", "var(--bf-green)")
    text(o, 654, y + 24, "D  delivery", 13, "bf-t-strong", "middle")
    text(o, 654, y + 42, "in order", 11.5, anchor="middle")
    o.append(f'<path d="M654,{y + 60} L654,{y + 92} L80,{y + 92} L80,{y + 62}" style="fill:none;stroke:var(--bf-green);stroke-width:1.6;stroke-dasharray:5 4"/>')
    o.append(f'<polygon points="80,{y + 60} 75,{y + 69} 85,{y + 69}" style="fill:var(--bf-green)"/>')
    text(o, 367, y + 86, "taking the next block in order releases an admission slot", 11.5, anchor="middle", extra=";fill:var(--bf-green)")
    hy = y + 116
    box(o, 20, hy, 682, 34, "var(--bf-surface)", "var(--bf-frame)")
    text(o, 361, hy + 22, "shared caches and memory: every stage competes for them", 12, anchor="middle")
    for x in (80, 428, 654):
        o.append(f'<line x1="{x}" x2="{x}" y1="{y + 64 if x != 428 else y + 66}" y2="{hy}" style="stroke:var(--bf-frame);stroke-width:1.2;stroke-dasharray:2 3"/>')
    aria = ("The decoder's pipeline. One scanner thread decodes each block's Huffman symbols into a queue of "
            "scanned blocks; each worker takes one block through MTF and RLE2, construction, emission, and "
            "RLE1 and CRC; the reader delivers blocks in order, and taking the next block releases an "
            "admission slot back to the scanner. Every stage shares the caches and memory.")
    return svg(hy + 44, o, aria)


def budget():
    o = []
    text(o, 16, 20, "Where one worker's 1,280 ms goes: construction is three sub-phases", 13, "bf-t-strong")
    parts = [("A5", "MTF, RLE2", 132, "var(--bf-steel)", 0.45), ("A6a", "first pass", 200, "var(--bf-blue)", 0.95),
             ("A6b", "induction, words", 216, "var(--bf-blue)", 0.75), ("A6c", "wiring", 161, "var(--bf-blue)", 0.55),
             ("A7", "emission", 490, "var(--bf-amber)", 0.6), ("A9", "RLE1, CRC", 76, "var(--bf-steel)", 0.35),
             ("", "", 5, "var(--bf-frame)", 0.3)]
    total = sum(p[2] for p in parts)
    L, R0, y, h = 20, 700, 64, 46
    x = L
    for code, lab, ms, col, op in parts:
        w = (R0 - L) * ms / total
        o.append(f'<rect x="{x:.1f}" y="{y}" width="{w:.1f}" height="{h}" style="fill:{col};opacity:{op};stroke:var(--bf-surface);stroke-width:1"/>')
        if code:
            text(o, x + w / 2, y + 20, code, 12.5, "bf-t-strong", "middle")
            text(o, x + w / 2, y + 36, f"{ms} ms", 11, anchor="middle")
            text(o, x + w / 2, y + h + 18, lab, 11, anchor="middle")
        x += w
    cx0 = L + (R0 - L) * 132 / total
    cx1 = L + (R0 - L) * (132 + 200 + 216 + 161) / total
    o.append(f'<path d="M{cx0:.1f},{y - 8} L{cx0:.1f},{y - 16} L{cx1:.1f},{y - 16} L{cx1:.1f},{y - 8}" style="fill:none;stroke:var(--bf-blue);stroke-width:1.6"/>')
    text(o, (cx0 + cx1) / 2, y - 22, "construction, C: 578 ms, about 45% of the 1,283 ms decode", 12, "bf-t-strong", "middle", ";fill:var(--bf-blue)")
    text(o, 361, y + h + 46, "An idea that changes one sub-phase can save at most that sub-phase's time.", 12.5, anchor="middle")
    aria = ("A bar of one worker's busy time, 1,280 ms, at one worker: MTF and RLE2 132 ms; construction's three "
            "sub-phases, the first pass 200 ms, induction and words 216 ms and wiring 161 ms, together 578 ms; "
            "emission 490 ms; RLE1 and CRC 76 ms.")
    return svg(y + h + 58, o, aria)


def rauzy():
    o = []
    text(o, 16, 20, "Induction steps per original run, and what the best known order could save", 13, "bf-t-strong")
    rows = [("right end (today's order)", 1.812, "var(--bf-amber)"),
            ("free steps first, else the largest cell", 1.279, "var(--bf-blue)"),
            ("floor for this class of steps, r<tspan baseline-shift=\"sub\" font-size=\"9\">2</tspan> − 1", 1.220, "var(--bf-frame)")]
    L, R0, top, rh = 300, 640, 44, 30
    x = lambda v: L + (R0 - L) * v / 2.0
    for i, (lab, v, col) in enumerate(rows):
        y = top + i * rh
        text(o, L - 10, y + 15, lab, 12, anchor="end")
        o.append(f'<rect x="{L}" y="{y}" width="{x(v) - L:.1f}" height="20" rx="3" style="fill:{col};opacity:0.75"/>')
        text(o, x(v) + 8, y + 15, f"{v:.3f}", 12.5, "bf-t-strong")
    y0 = top + 3 * rh + 24
    steps = [("A6b at one worker", "216 ms"), ("× the step cut, 1 − 1.279/1.812", "× 29.4%"),
             ("= the most it could save, if all of A6b scaled with steps", "≈ 64 ms, 4.9% of 1,306 ms"),
             ("the 3% gate", "≈ 39 ms"), ("so the share of A6b that must be step time", "≈ 62%, before the new order's own cost")]
    for i, (a, b) in enumerate(steps):
        yy = y0 + i * 22
        strong = i in (2, 4)
        text(o, L - 10, yy, a, 12, "bf-t-strong" if strong else "bf-tm", "end")
        text(o, L + 6, yy, b, 12.5, "bf-t-strong" if strong else "bf-tm")
    aria = ("Bars of induction steps per original run: today's right-end order 1.812, the best recorded order "
            "1.279, the floor 1.220. Below, the budget: A6b's 216 ms at one worker times a 29.4% cut is at most "
            "64 ms, 4.9% of the wall; the 3% gate is 39 ms, so about 62% of A6b would have to be step time.")
    return svg(y0 + 5 * 22, o, aria)


if __name__ == "__main__":
    print(globals()[sys.argv[1]]())
