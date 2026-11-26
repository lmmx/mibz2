"""Figures for "Complementary bottlenecks: reading the ideal-parts matrix" (blog, 26 November 2026).

    python3 docs/scripts/figures_where_mathematics.py NAME     # print one figure's <svg>
    python3 docs/scripts/figures_where_mathematics.py --inject POST.md

`--inject` replaces the line after each `<!-- fig:NAME -->` marker in POST.md with that figure's
<svg> (one line, so Markdown leaves it alone).

The figures follow the dataviz method: the form first, then colour by its job, from the chart roles
in blog.css (--bf-vz-*: categorical 1-4 in fixed order, a diverging pair about a grey midpoint, a
sequential ramp and an ordinal ramp, each validated in light and dark against the page surfaces).
Gridlines and axes are solid hairlines; bars are at most 24 px with 4 px data ends; lines are 2 px;
markers have r >= 4 and a 2 px surface ring. Every data mark carries a <title>, the hover layer.

Sources (paths under docs/journal/2026-11-24-algorithm-level/ unless stated):
- sampling: ideal-parts/registration.md clarification 8 (I = +SECMRD, plus the 12 odd arms).
- matrix: ideal-parts/results/full/report.txt, the ceiling table (room alone, its 95% interval and
  label); the reduction is 1 - room.
- interaction, constroom: the same table's "room with Q ideal" column and README.md section 4's
  all-real walls (1,306, 243, 195, 196 ms). The walls are the ratios times those.
- twostation: an illustrative model, T = max(S(1 - a), W(1 - b)/k) with S = W/k = 185 ms. The
  measured points are the 8-worker arms; construction is placed at b = 0.45, its share of a worker's
  busy time in math-targets/data/character-stage1/report.txt (578 of 1,280 ms).
- contraction: a schematic ten-row column, L = bbaaacaabb, with LF computed exactly.
- touches: character-stage1/report.txt, one worker (A5 132, A6a 200, A6b 216, A6c 161, A7 490,
  A9 76 ms of 1,280 busy); the forecasts are research/neon-movement/IDEAL-PARTS-2.md at 7fab624e, §5.
- economics: monodromy/2026-11-10-rauzy-veech.md §5.6 (1.812 and 1.279 steps a run), A6b's 216 ms
  and the 3% gate on the 1,306 ms wall (ideal-parts/reviews/2026-11-26-chatgpt.md §6B).
- fold, lanes, pooled: IDEAL-PARTS-2.md §5, proposals 1-3 (forecasts, not measurements). The ring's
  anchors are an illustrative draw; the expected longest arc m H_K / K is the order-statistics
  result the design analysis cites.
- landscape: report.txt and README.md (195, 125, 112 ms at 8 workers); IDEAL-PARTS-2.md §§3, 5
  (130-145 and 172-175 ms forecasts, Residual B 14-17 ms at 8, the ~21 ms write floor).
- pipeline: ideal-parts/parts.md.
"""
import itertools
import math
import re
import sys

W = 720
REAL = {1: 1306, 6: 243, 8: 195, 10: 196}
KS = (1, 6, 8, 10)
PARTS = "SMCERD"
NAMES = {"S": "Scanner", "M": "MTF, RLE2", "C": "Construction", "E": "Emission", "R": "RLE1, CRC", "D": "Delivery"}
# report.txt, the ceiling table: room alone (ratio), its 95% interval, the label.
ROOM = {
    1: {"S": (1.0101, 0.9924, 1.0282, "U"), "E": (0.8345, 0.8226, 0.8466, "R"), "C": (0.5792, 0.5733, 0.5851, "R"),
        "M": (0.9248, 0.9065, 0.9435, "R"), "R": (0.9828, 0.9691, 0.9966, "U"), "D": (1.0106, 0.9916, 1.0300, "U")},
    6: {"S": (0.9740, 0.9651, 0.9829, "U"), "E": (0.8507, 0.8379, 0.8637, "R"), "C": (0.7540, 0.7447, 0.7635, "R"),
        "M": (0.9160, 0.9049, 0.9272, "R"), "R": (0.9959, 0.9735, 1.0189, "U"), "D": (0.9892, 0.9805, 0.9980, "N")},
    8: {"S": (0.9683, 0.9556, 0.9812, "U"), "E": (0.9652, 0.9460, 0.9848, "U"), "C": (0.9437, 0.9313, 0.9564, "U"),
        "M": (0.9755, 0.9568, 0.9944, "U"), "R": (1.0093, 0.9930, 1.0260, "U"), "D": (0.9916, 0.9705, 1.0133, "U")},
    10: {"S": (0.9193, 0.9033, 0.9356, "R"), "E": (0.9780, 0.9664, 0.9897, "U"), "C": (0.9467, 0.9384, 0.9550, "U"),
         "M": (0.9794, 0.9710, 0.9878, "U"), "R": (1.0212, 1.0095, 1.0331, "U"), "D": (0.9938, 0.9875, 1.0002, "N")},
}
C_WITH_S = {1: 0.569, 6: 0.605, 8: 0.661, 10: 0.825}  # report.txt: C's room with S ideal


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def svg(h, body, aria, w=W):
    return f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{esc(aria)}">' + "".join(body) + "</svg>"


def text(o, x, y, s, size=12, cls="bf-tm", anchor="start", fill=None, weight=None, italic=False):
    st = f"font-size:{size}px"
    if fill:
        st += f";fill:{fill}"
    if weight:
        st += f";font-weight:{weight}"
    if italic:
        st += ";font-style:italic"
    o.append(f'<text class="{cls}" x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" style="{st}">{s}</text>')


def rect(o, x, y, w, h, fill, rx=0, stroke=None, sw=1, title=None, extra=""):
    st = f"fill:{fill}"
    if stroke:
        st += f";stroke:{stroke};stroke-width:{sw}"
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(f'<rect x="{x:.1f}" y="{y:.1f}" width="{max(w, 0):.1f}" height="{max(h, 0):.1f}" rx="{rx}" style="{st}{extra}">{t}</rect>')


def bar_h(o, x0, y, length, h, fill, title=None):
    """A horizontal bar from a baseline at x0: square at the baseline, 4 px rounded data end."""
    r = min(4, h / 2, abs(length))
    if length <= 0:
        return
    x1 = x0 + length
    d = (f"M{x0:.1f},{y:.1f} L{x1 - r:.1f},{y:.1f} Q{x1:.1f},{y:.1f} {x1:.1f},{y + r:.1f} "
         f"L{x1:.1f},{y + h - r:.1f} Q{x1:.1f},{y + h:.1f} {x1 - r:.1f},{y + h:.1f} L{x0:.1f},{y + h:.1f} Z")
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(f'<path d="{d}" style="fill:{fill}">{t}</path>')


def line(o, x1, y1, x2, y2, col="var(--bf-vz-axis)", w=1, extra=""):
    o.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" style="stroke:{col};stroke-width:{w}{extra}"/>')


def arrow(o, x1, y1, x2, y2, col="var(--bf-muted)", w=1.6, head=8):
    line(o, x1, y1, x2, y2, col, w, ";stroke-linecap:round")
    a = math.atan2(y2 - y1, x2 - x1)
    p = [(x2, y2), (x2 - head * math.cos(a) + head * 0.55 * math.sin(a), y2 - head * math.sin(a) - head * 0.55 * math.cos(a)),
         (x2 - head * math.cos(a) - head * 0.55 * math.sin(a), y2 - head * math.sin(a) + head * 0.55 * math.cos(a))]
    o.append(f'<polygon points="{" ".join(f"{u:.1f},{v:.1f}" for u, v in p)}" style="fill:{col}"/>')


def dot(o, x, y, fill, r=5, title=None):
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r}" style="fill:{fill};stroke:var(--md-default-bg-color);stroke-width:2">{t}</circle>')


def box(o, x, y, w, h, fill, stroke, rx=6, sw=1.2):
    rect(o, x, y, w, h, fill, rx, stroke, sw)


def swatch(o, x, y, fill, label, outline=False):
    if outline:
        rect(o, x, y - 9, 11, 11, "none", 2, "var(--bf-muted)", 1.2)
    else:
        rect(o, x, y - 9, 11, 11, fill, 2)
    text(o, x + 17, y, label, 12, "bf-t")
    return x + 17 + 6.6 * len(re.sub("<[^>]+>", "", label)) + 22


# 1. How the 64 configurations were sampled ---------------------------------------------------
def sampling():
    o = []
    x = swatch(o, 20, 18, "var(--bf-vz-1)", "the half fraction, even counts ideal (32)")
    x = swatch(o, x, 18, "var(--bf-vz-2)", "added arms (12)")
    swatch(o, x, 18, None, "not run (20)", outline=True)
    top, rh, sq, gap, x0 = 46, 30, 16, 3, 132
    labels = ["no part ideal", "one ideal", "two ideal", "three ideal", "four ideal", "five ideal", "all six ideal"]
    for j in range(7):
        y = top + j * rh
        text(o, x0 - 14, y + 13, labels[j], 12, "bf-t", "end")
        combos = list(itertools.combinations(PARTS, j))
        for i, c in enumerate(combos):
            xx = x0 + i * (sq + gap)
            ideal = ", ".join(c) if c else "none"
            real = ", ".join(p for p in PARTS if p not in c) or "none"
            if j == 3:
                rect(o, xx + 0.6, y + 0.6, sq - 1.2, sq - 1.2, "none", 3, "var(--bf-vz-axis)", 1.2,
                     f"not run: ideal {ideal}; real {real}")
                continue
            fill, ink, kind = (("var(--bf-vz-2)", "var(--bf-vz-2-ink)", "added arm") if j in (1, 5)
                               else ("var(--bf-vz-1)", "var(--bf-vz-1-ink)", "in the half fraction"))
            rect(o, xx, y, sq, sq, fill, 3, title=f"{kind}: ideal {ideal}; real {real}")
            if j == 1:
                text(o, xx + sq / 2, y + 12, c[0], 10, "bf-t", "middle", f"var(--bf-vz-2-ink)", 600)
            if j == 5:
                miss = [p for p in PARTS if p not in c][0]
                text(o, xx + sq / 2, y + 12, miss, 10, "bf-t", "middle", f"var(--bf-vz-2-ink)", 600)
        n = len(combos)
        run = 0 if j == 3 else n
        text(o, 574, y + 12, f"{run} of {n}", 12.5, "bf-t-strong", "end")
        note = {1: "letter: the ideal part", 5: "letter: the real part", 3: "not run"}.get(j)
        if note:
            text(o, 588, y + 12, note, 11, "bf-tm")
    # Stat tiles.
    ty = top + 7 * rh + 18
    for i, (val, lab) in enumerate((("44 of 64", "configurations, at each worker count"),
                                    ("15 of 15", "pairwise interactions estimated"),
                                    ("6 of 6", "rooms read from a measured arm"))):
        tx = 20 + i * 232
        rect(o, tx, ty, 218, 62, "var(--bf-surface)", 8)
        text(o, tx + 14, ty + 30, val, 22, "bf-t", weight=600)
        text(o, tx + 14, ty + 50, lab, 11.5, "bf-tm")
    aria = ("The 64 ways of making six parts real or ideal, one row per number of ideal parts: 1, 6, 15, 20, 15, 6 "
            "and 1. The half fraction keeps the 32 with an even number ideal; the 12 with one part ideal or one part "
            "real were added; the 20 with three ideal were not run. 44 of 64 measured, all 15 pairwise "
            "interactions estimated.")
    return svg(ty + 70, o, aria)


# 2. The full matrix as a diverging table-heatmap ----------------------------------------------
def div_step(v):
    """Diverging bin for a reduction v in percent: (fill, ink)."""
    edges = (1, 3, 6, 12, 24)
    a = abs(v)
    if a < 1:
        return "var(--bf-vz-mid)", "var(--bf-vz-mid-ink)"
    n = sum(a >= e for e in edges)
    arm = "f" if v > 0 else "s"
    return f"var(--bf-vz-{arm}{n})", f"var(--bf-vz-{arm}{n}-ink)"


def matrix():
    o = []
    x0, top, cw, ch, g = 200, 44, 122, 50, 2
    for i, k in enumerate(KS):
        text(o, x0 + i * (cw + g) + cw / 2, top - 12, f"{k} worker{'s' if k > 1 else ''}", 12.5, "bf-t-strong", "middle")
    for r, p in enumerate(PARTS):
        y = top + r * (ch + g)
        name = NAMES[p] + (" ¹" if p == "E" else "")
        text(o, x0 - 14, y + 22, name, 13, "bf-t-strong", "end")
        text(o, x0 - 14, y + 38, f"part {p}", 11, "bf-tm", "end")
        for i, k in enumerate(KS):
            ratio, lo, hi, lab = ROOM[k][p]
            v = (1 - ratio) * 100
            vlo, vhi = (1 - hi) * 100, (1 - lo) * 100
            fill, ink = div_step(v)
            x = x0 + i * (cw + g)
            label = {"R": "ROOM", "N": "NO ROOM", "U": "UNRESOLVED"}[lab]
            rect(o, x, y, cw, ch, fill, 4,
                 title=f"{NAMES[p]}, {k} worker{'s' if k > 1 else ''}: {v:+.1f}% ({vlo:+.1f} to {vhi:+.1f}), {label}")
            fmt = lambda u: ("0.0" if abs(u) < 0.05 else f"{u:.1f}").replace("-", "−")
            vs = fmt(v) + "%"
            text(o, x + cw / 2, y + 22, vs, 15 if lab == "R" else 13.5, "bf-t", "middle", ink, 700 if lab == "R" else 500)
            iv = f"{fmt(vlo)} to {fmt(vhi)}"
            sub = ("ROOM · " if lab == "R" else "no room · " if lab == "N" else "") + iv
            text(o, x + cw / 2, y + 39, sub, 10, "bf-t", "middle", ink)
    # The scale.
    ly = top + 6 * (ch + g) + 22
    steps = [("s5", "≥24"), ("s4", "12"), ("s3", "6"), ("s2", "3"), ("s1", "1"), ("mid", "0"),
             ("f1", "1"), ("f2", "3"), ("f3", "6"), ("f4", "12"), ("f5", "≥24")]
    sx, sw = 230, 36
    for i, (s, _) in enumerate(steps):
        rect(o, sx + i * (sw + 2), ly, sw, 12, f"var(--bf-vz-{s})", 2)
    ticks = ["24", "12", "6", "3", "1", "", "1", "3", "6", "12", "24"]
    for i, t in enumerate(ticks):
        if not t:
            continue
        # A tick sits on the boundary between two bins, at the inner edge of the outer one.
        xb = sx + i * (sw + 2) + (sw + 1 if i < 5 else -1)
        text(o, xb, ly + 26, t, 10.5, "bf-tm", "middle")
    text(o, sx - 12, ly + 10, "slower", 11.5, "bf-t", "end")
    text(o, sx + 11 * (sw + 2) + 6, ly + 10, "faster", 11.5, "bf-t")
    text(o, sx + 5.5 * (sw + 2) - 1, ly + 42, "change in the decode time, %", 10.5, "bf-tm", "middle")
    aria = ("A table of six parts by four worker counts: the percentage by which making each part alone ideal "
            "shortens the decode. Construction 42.1, 24.6, 5.6 and 5.3; emission 16.5, 14.9, 3.5 and 2.2; MTF and "
            "RLE2 7.5, 8.4, 2.5 and 2.1; the scanner -1.0, 2.6, 3.2 and 8.1; RLE1 and CRC 1.7, 0.4, -0.9 and -2.1; "
            "delivery -1.1, 1.1, 0.8 and 0.6.")
    return svg(ly + 50, o, aria)


# 3. The scanner and construction, normalised, on a log scale ----------------------------------
def walls(k):
    real = REAL[k]
    s = real * ROOM[k]["S"][0]
    return {"real": real, "C": real * ROOM[k]["C"][0], "S": s, "SC": s * C_WITH_S[k]}


def interaction():
    o = []
    x = 20
    x = swatch(o, x, 18, "var(--bf-vz-1)", "construction real")
    swatch(o, x, 18, "var(--bf-vz-2)", "construction supplied")
    top, ph, pw, gx, x0 = 52, 220, 148, 14, 64
    lo, hi = math.log(0.55), math.log(1.06)
    yv = lambda v: top + ph * (hi - math.log(v)) / (hi - lo)
    for t in (0.6, 0.7, 0.8, 0.9, 1.0):
        line(o, x0, yv(t), x0 + 4 * pw + 3 * gx, yv(t), "var(--bf-vz-grid)")
        text(o, x0 - 8, yv(t) + 4, f"{t:.1f}", 11, "bf-tm", "end")
    text(o, 14, top - 14, "wall over all real (log scale)", 11, "bf-tm")
    for j, k in enumerate(KS):
        c = walls(k)
        px = x0 + j * (pw + gx)
        xa, xb = px + 34, px + pw - 34
        text(o, px + pw / 2, top + ph + 40, f"{k} worker{'s' if k > 1 else ''}", 12.5, "bf-t-strong", "middle")
        text(o, xa, top + ph + 20, "S real", 11, "bf-tm", "middle")
        text(o, xb, top + ph + 20, "S supplied", 11, "bf-tm", "middle")
        line(o, px + 6, top + ph + 4, px + pw - 6, top + ph + 4, "var(--bf-vz-axis)")
        for (a, b, col, nm) in ((c["real"], c["S"], "var(--bf-vz-1)", "construction real"),
                                (c["C"], c["SC"], "var(--bf-vz-2)", "construction supplied")):
            ra, rb = a / c["real"], b / c["real"]
            line(o, xa, yv(ra), xb, yv(rb), col, 2, ";stroke-linecap:round")
            dot(o, xa, yv(ra), col, 4.5, f"{k} workers, scanner real, {nm}: {a:,.0f} ms ({ra:.3f})")
            dot(o, xb, yv(rb), col, 4.5, f"{k} workers, scanner supplied, {nm}: {b:,.0f} ms ({rb:.3f})")
        text(o, xb + 8, yv(c["SC"] / c["real"]) + 4, f"{c['SC']:,.0f}", 11.5, "bf-t-strong")
        text(o, xa, yv(1.0) - 10, f"{c['real']:,.0f}", 11.5, "bf-tm", "middle")
    text(o, x0 + 2 * pw + 1.5 * gx, top + ph + 62,
         "labels in ms: the all-real decode (left) and both supplied (right)", 11, "bf-tm", "middle")
    aria = ("Four panels, at 1, 6, 8 and 10 workers, of the decode time relative to all parts real on a log scale, "
            "with the scanner real or supplied and construction real or supplied. At 1 worker the lines are "
            "parallel; at 6 and 8 they diverge sharply; at 10 less so. Both supplied: 751, 143, 125 and 149 ms.")
    return svg(top + ph + 72, o, aria)


# 4. How much construction can remove, with and without the scanner -----------------------------
def constroom():
    o = []
    x = swatch(o, 20, 18, "var(--bf-vz-1)", "with the ordinary scanner")
    swatch(o, x, 18, "var(--bf-vz-2)", "with the scanner supplied")
    x0, x1, top, gh, bh = 120, 470, 50, 70, 20
    xv = lambda v: x0 + (x1 - x0) * v / 50
    for t in range(0, 51, 10):
        line(o, xv(t), top - 6, xv(t), top + 4 * gh - 12, "var(--bf-vz-grid)")
        text(o, xv(t), top + 4 * gh + 4, f"{t}%", 11, "bf-tm", "middle")
    for j, k in enumerate(KS):
        c = walls(k)
        y = top + j * gh
        text(o, x0 - 12, y + 26, f"{k} worker{'s' if k > 1 else ''}", 12.5, "bf-t-strong" if k == 8 else "bf-t", "end")
        rows = ((c["real"], c["C"], "var(--bf-vz-1)", "ordinary scanner"), (c["S"], c["SC"], "var(--bf-vz-2)", "scanner supplied"))
        for i, (a, b, col, nm) in enumerate(rows):
            v = (1 - b / a) * 100
            yy = y + i * (bh + 4)
            bar_h(o, x0, yy, xv(v) - x0, bh, col, f"{k} workers, {nm}: construction removes {v:.1f}% ({a:,.0f} to {b:,.0f} ms)")
            text(o, xv(v) + 8, yy + 15, f"{v:.1f}%", 12.5, "bf-t-strong")
            text(o, xv(v) + 58, yy + 15, f"{a:,.0f} → {b:,.0f} ms", 11.5, "bf-tm")
    line(o, x0, top - 6, x0, top + 4 * gh - 12, "var(--bf-vz-axis)")
    aria = ("Bars of the share of the decode that supplying construction removes. With the ordinary scanner: "
            "42.1% at 1 worker, 24.6% at 6, 5.6% at 8 and 5.3% at 10. With the scanner supplied: 43.1%, 39.5%, "
            "33.9% and 17.5%. At 8 workers that is 195 to 184 ms against 189 to 125 ms.")
    return svg(top + 4 * gh + 14, o, aria)


# 5. The two-station model as a static surface --------------------------------------------------
def twostation():
    o = []
    x0, y0, s = 70, 26, 300
    X = lambda a: x0 + s * a
    Y = lambda b: y0 + s * (1 - b)
    for i, c in enumerate((0.0, 0.2, 0.4, 0.6, 0.8)):
        rect(o, X(c), Y(1), s * (1 - c), s * (1 - c), f"var(--bf-vz-q{i + 1})",
             title=f"model reduction at least {c * 100:.0f}%: elapsed at most {185 * (1 - c):.0f} ms")
    for c in (0.1, 0.3, 0.5, 0.7, 0.9):
        o.append(f'<path d="M{X(c):.1f},{Y(1):.1f} L{X(c):.1f},{Y(c):.1f} L{X(1):.1f},{Y(c):.1f}" '
                 f'style="fill:none;stroke:var(--md-default-bg-color);stroke-width:1;opacity:0.7"/>')
    line(o, X(0), Y(0), X(1), Y(1), "var(--bf-ink)", 1, ";opacity:0.55")
    text(o, X(0.56), Y(0.62), "relieve both alike", 11, "bf-t", "middle", "var(--bf-vz-q4-ink)")
    for c in (0.2, 0.4, 0.6, 0.8):
        i = int(round(c * 5))
        ink = f"var(--bf-vz-q{i + 1}-ink)"
        text(o, X(c) + 5, Y(1) + 14, f"−{c * 100:.0f}%", 10.5, "bf-t", "start", ink, 600)
    for t in (0, 0.25, 0.5, 0.75, 1.0):
        text(o, X(t), Y(0) + 16, f"{t * 100:.0f}%", 11, "bf-tm", "middle")
        text(o, X(0) - 8, Y(t) + 4, f"{t * 100:.0f}%", 11, "bf-tm", "end")
    text(o, X(0.5), Y(0) + 34, "scanner work removed, a", 12, "bf-t", "middle")
    o.append(f'<text class="bf-t" x="18" y="{Y(0.5):.1f}" text-anchor="middle" transform="rotate(-90 18 {Y(0.5):.1f})" '
             f'style="font-size:12px">worker work removed, b</text>')
    pts = [(0, 0, "all real", "0%", "195 ms"), (1, 0, "scanner supplied", "−3%", "189 ms"),
           (0, 0.45, "construction supplied", "−6%", "184 ms"), (1, 0.45, "both supplied", "−36%", "125 ms")]
    for a, b, lab, red, ms in pts:
        px, py = X(a), Y(b)
        dot(o, px, py, "var(--bf-vz-2)", 5.5, f"measured at 8 workers, {lab}: {ms}, {red}; the model gives "
            f"{min(a, b) * 100:.0f}%")
    # Labels for the measured points, placed inside the plot.
    text(o, X(0) + 10, Y(0) - 8, "all real, 195 ms", 11, "bf-t", "start", "var(--bf-vz-q1-ink)")
    text(o, X(1) - 10, Y(0) - 8, "S supplied: −3%", 11, "bf-t", "end", "var(--bf-vz-q1-ink)")
    text(o, X(0) + 10, Y(0.45) + 4, "C supplied: −6%", 11, "bf-t", "start", "var(--bf-vz-q1-ink)")
    text(o, X(1) - 10, Y(0.45) - 10, "both: −36%", 11, "bf-t", "end", "var(--bf-vz-q3-ink)", 600)
    text(o, X(1) - 10, Y(0.45) + 18, "(model: −45%)", 10.5, "bf-t", "end", "var(--bf-vz-q3-ink)")
    # The key and the model, on the right.
    rx = 410
    text(o, rx, 40, "Model elapsed time", 12.5, "bf-t-strong")
    o.append(f'<text class="bf-t" x="{rx}" y="62" style="font-size:12.5px"><tspan style="font-style:italic">T</tspan> = max( '
             f'<tspan style="font-style:italic">S</tspan>(1 − <tspan style="font-style:italic">a</tspan>), '
             f'<tspan style="font-style:italic">W</tspan>(1 − <tspan style="font-style:italic">b</tspan>) / '
             f'<tspan style="font-style:italic">k</tspan> )</text>')
    text(o, rx, 82, "with S = W/k = 185 ms, balanced", 11.5, "bf-tm")
    text(o, rx, 110, "Reduction in elapsed time", 12.5, "bf-t-strong")
    for i, (lo_, hi_) in enumerate(((0, 20), (20, 40), (40, 60), (60, 80), (80, 100))):
        yy = 128 + i * 20
        rect(o, rx, yy - 10, 22, 13, f"var(--bf-vz-q{i + 1})", 2, "var(--bf-vz-axis)", 0.6)
        text(o, rx + 30, yy, f"{lo_}–{hi_}%  ({185 * (1 - lo_ / 100):.0f}–{185 * (1 - hi_ / 100):.0f} ms)", 11.5, "bf-t")
    dot(o, rx + 11, 238, "var(--bf-vz-2)", 5.5)
    text(o, rx + 30, 242, "measured at 8 workers (the matrix)", 11.5, "bf-t")
    for i, s_ in enumerate(("The contours are L-shaped: along either axis",
                            "alone nothing moves until that station is no",
                            "longer the slower one. Construction is placed",
                            "at b ≈ 45%, its share of a worker's time.")):
        text(o, rx, 274 + i * 17, s_, 11.5, "bf-tm")
    aria = ("A square map of the illustrative two-station model's elapsed time, with the scanner's work removed on "
            "the horizontal axis and the workers' on the vertical. The shading shows the reduction, which equals the "
            "smaller of the two removals, so the contours are L-shaped and only the diagonal gains. Four measured "
            "8-worker points: all real 195 ms; scanner supplied, 3% faster; construction supplied, 6%; both, 36%, "
            "where the model gives 45%.")
    return svg(y0 + s + 46, o, aria)


# 6. Contraction, schematically -----------------------------------------------------------------
def contraction():
    o = []
    L = "bbaaacaabb"
    cnt = {c: L.count(c) for c in "abc"}
    C = {"a": 0, "b": cnt["a"], "c": cnt["a"] + cnt["b"]}
    seen = {c: 0 for c in "abc"}
    lf = []
    for c in L:
        lf.append(C[c] + seen[c])
        seen[c] += 1
    F = "".join(sorted(L))
    col = {"a": "var(--bf-vz-1)", "b": "var(--bf-vz-2)", "c": "var(--bf-vz-3)"}
    ink = {"a": "var(--bf-vz-1-ink)", "b": "var(--bf-vz-2-ink)", "c": "var(--bf-vz-3-ink)"}
    cw, ch, g = 28, 24, 2
    runs = []
    i = 0
    while i < len(L):
        j = i
        while j + 1 < len(L) and L[j + 1] == L[i]:
            j += 1
        runs.append((i, j))
        i = j + 1
    for p, (px, title, sub) in enumerate(((24, "Ten rows, ten steps", "each row follows LF on its own"),
                                          (378, "Five runs, five steps", "a run of equal bytes moves as one interval"))):
        text(o, px, 22, title, 13, "bf-t-strong")
        text(o, px, 40, sub, 11.5, "bf-tm")
        ly, fy = 70, 176
        text(o, px, ly - 8, "L, the last column", 10.5, "bf-tm")
        text(o, px, fy + ch + 16, "F, the same bytes sorted", 10.5, "bf-tm")
        X = lambda i: px + i * (cw + g)
        if p == 1:
            for (a, b) in runs:
                t0, t1 = lf[a], lf[b]
                c = L[a]
                d = (f"M{X(a):.1f},{ly + ch:.1f} L{X(b) + cw:.1f},{ly + ch:.1f} L{X(t1) + cw:.1f},{fy:.1f} "
                     f"L{X(t0):.1f},{fy:.1f} Z")
                o.append(f'<path d="{d}" style="fill:{col[c]};opacity:0.22"><title>rows {a}–{b} ({c}) map to rows '
                         f'{t0}–{t1} of F in one step</title></path>')
                o.append(f'<path d="M{X(a) + 1:.1f},{ly + ch + 1:.1f} L{X(t0) + 1:.1f},{fy - 1:.1f} M{X(b) + cw - 1:.1f},'
                         f'{ly + ch + 1:.1f} L{X(t1) + cw - 1:.1f},{fy - 1:.1f}" style="fill:none;stroke:{col[c]};stroke-width:1.2"/>')
        else:
            for i, t in enumerate(lf):
                arrow(o, X(i) + cw / 2, ly + ch + 2, X(t) + cw / 2, fy - 2, col[L[i]], 1.4, 6)
        for i, c in enumerate(L):
            rect(o, X(i), ly, cw, ch, col[c], 3, title=f"row {i} of L: {c}, LF = {lf[i]}")
            text(o, X(i) + cw / 2, ly + 16, c, 12, "bf-t", "middle", ink[c], 600)
        for i, c in enumerate(F):
            rect(o, X(i), fy, cw, ch, col[c], 3, title=f"row {i} of F: {c}")
            text(o, X(i) + cw / 2, fy + 16, c, 12, "bf-t", "middle", ink[c], 600)
        if p == 1:
            for (a, b) in runs[1:]:
                xx = X(a) - g / 2
                line(o, xx, ly - 5, xx, ly + ch + 5, "var(--bf-ink)", 1.5)
    by = 238
    rect(o, 24, by, 672, 76, "var(--bf-surface)", 8)
    text(o, 40, by + 24, "The question that remains: what else can be treated as one computation?", 12.5, "bf-t-strong")
    text(o, 40, by + 44, "Equal adjacent bytes are one sense of moving together, and a rigid-interval calculus has a "
         "proved floor for it.", 11.5, "bf-tm")
    text(o, 40, by + 62, "So the search is for other senses: up to a relabelling, repeated transformations, "
         "equivalent paths.", 11.5, "bf-tm")
    aria = ("Two copies of a ten-row column. On the left each of the ten rows of the last column has its own arrow to "
            "its place in the sorted column. On the right the same rows are grouped into five runs of equal bytes, "
            "and each run maps to a contiguous interval in one step. Below: what else can be treated as one "
            "computation?")
    return svg(by + 84, o, aria)


# 7. What each proposal touches: one worker's busy time -----------------------------------------
def touches():
    o = []
    segs = [("A5", "MTF, RLE2", 132, "var(--bf-vz-4)", "var(--bf-vz-4-ink)"),
            ("A6a", "first pass", 200, "var(--bf-vz-1)", "var(--bf-vz-1-ink)"),
            ("A6b", "induction", 216, "var(--bf-vz-1)", "var(--bf-vz-1-ink)"),
            ("A6c", "wiring", 161, "var(--bf-vz-1)", "var(--bf-vz-1-ink)"),
            ("A7", "emission", 490, "var(--bf-vz-2)", "var(--bf-vz-2-ink)"),
            ("A9", "RLE1, CRC", 76, "var(--bf-vz-3)", "var(--bf-vz-3-ink)"),
            ("", "other", 5, "var(--bf-vz-axis)", "")]
    total = 1280
    x0, x1, y, h = 20, 700, 118, 28
    sc = (x1 - x0 - 2 * (len(segs) - 1)) / total
    xs = []
    x = x0
    for code, lab, ms, col, ink in segs:
        w = ms * sc
        xs.append((x, x + w))
        rect(o, x, y, w, h, col, 4, title=f"{code} {lab}: {ms} ms of 1,280" if code else "other: 5 ms")
        if code and w > 48:
            text(o, x + 6, y + 18, f"{ms}", 12, "bf-t", "start", ink, 600)
        if code:
            text(o, x + w / 2, y + h + 16, lab, 11, "bf-tm", "middle")
        x += w + 2
    text(o, x0, y + h + 34, "one worker's busy time at one worker, 1,280 ms; construction (blue) is 578 ms", 11, "bf-tm")

    def bracket(a, b, lev, lab1, lab2):
        xa, xb = xs[a][0], xs[b][1]
        yy = y - 8 - lev * 44
        o.append(f'<path d="M{xa + 1:.1f},{yy + 6} L{xa + 1:.1f},{yy} L{xb - 1:.1f},{yy} L{xb - 1:.1f},{yy + 6}" '
                 f'style="fill:none;stroke:var(--bf-ink);stroke-width:1.2;opacity:0.7"/>')
        text(o, (xa + xb) / 2, yy - 20, lab1, 12, "bf-t-strong", "middle")
        text(o, (xa + xb) / 2, yy - 6, lab2, 11, "bf-tm", "middle")
    bracket(0, 1, 0, "B. direct fold", "forecast 110–145 ms")
    bracket(2, 2, 1, "A. induction schedule", "at most 64 ms; gate 39 ms")
    bracket(3, 5, 0, "C. forward wiring with lanes", "forecast 280–340 ms")
    text(o, x0, 20, "D. pooled scan: moves the scanner's own thread (not in this bar) onto the workers", 12, "bf-t-strong")
    aria = ("A bar of one worker's 1,280 ms at one worker: MTF and RLE2 132 ms, construction's first pass 200, "
            "induction 216 and wiring 161, emission 490, RLE1 and CRC 76. Brackets show what each proposal touches: "
            "the direct fold, MTF and the first pass, forecast 110 to 145 ms; the induction schedule, the induction, "
            "at most 64 ms against a 39 ms gate; forward wiring with lanes, wiring, emission and RLE1, forecast 280 "
            "to 340 ms. The pooled scan moves the scanner's thread.")
    return svg(y + h + 44, o, aria)


# 8. The economics of a cheaper induction schedule ----------------------------------------------
def economics():
    o = []
    a6b, cut = 216.0, 1 - 1.279 / 1.812
    gate = 0.03 * 1306
    x0, x1, y0, y1 = 64, 560, 36, 296
    ymin, ymax = -12, 66
    X = lambda f: x0 + (x1 - x0) * f
    Y = lambda v: y1 - (y1 - y0) * (v - ymin) / (ymax - ymin)
    for v in (0, 20, 40, 60):
        line(o, x0, Y(v), x1, Y(v), "var(--bf-vz-grid)" if v else "var(--bf-vz-axis)")
        text(o, x0 - 8, Y(v) + 4, f"{v}", 11, "bf-tm", "end")
    for f in (0, 0.25, 0.5, 0.75, 1.0):
        text(o, X(f), y1 + 18, f"{f * 100:.0f}%", 11, "bf-tm", "middle")
    text(o, (x0 + x1) / 2, y1 + 38, "share of the induction sub-phase whose cost scales with the step count", 12, "bf-t", "middle")
    text(o, 14, y0 - 14, "net saving at one worker, ms", 11, "bf-tm")
    line(o, x0, Y(gate), x1, Y(gate), "var(--bf-ink)", 1.2, ";opacity:0.75")
    text(o, x0 + 6, Y(gate) - 7, f"the 3% gate, {gate:.1f} ms", 11.5, "bf-t-strong")
    cols = ["var(--bf-vz-o1)", "var(--bf-vz-o2)", "var(--bf-vz-o3)"]
    for i, ov in enumerate((0, 5, 10)):
        col = cols[i]
        net = lambda f: a6b * f * cut - ov
        o.append(f'<path d="M{X(0):.1f},{Y(net(0)):.1f} L{X(1):.1f},{Y(net(1)):.1f}" style="fill:none;stroke:{col};'
                 f'stroke-width:2;stroke-linecap:round"><title>scheduling overhead {ov} ms</title></path>')
        fx = (gate + ov) / (a6b * cut)
        dot(o, X(fx), Y(gate), col, 4.5, f"overhead {ov} ms: meets the gate at {fx * 100:.1f}% of the sub-phase")
        line(o, X(fx), Y(gate) + 6, X(fx), y1, col, 1, ";opacity:0.6")
        text(o, X(fx) + (-4, 0, 4)[i], y1 - 6, f"{fx * 100:.0f}%", 11, "bf-t-strong", ("end", "middle", "start")[i])
        lab = "no overhead" if ov == 0 else f"{ov} ms overhead"
        text(o, X(1) + 8, Y(net(1)) + 4, lab, 11.5, "bf-t")
    ex = 0.65
    ev = a6b * ex * cut - 5
    o.append(f'<circle cx="{X(ex):.1f}" cy="{Y(ev):.1f}" r="5" style="fill:none;stroke:var(--bf-ink);stroke-width:1.5"/>')
    arrow(o, X(ex) - 60, Y(ev) - 50, X(ex) - 6, Y(ev) - 4, "var(--bf-muted)", 1.2, 6)
    text(o, X(ex) - 62, Y(ev) - 70, f"65% with 5 ms overhead: {ev:.1f} ms,", 11.5, "bf-t", "end")
    text(o, X(ex) - 62, Y(ev) - 55, f"{ev / 1306 * 100:.2f}% of the decode, {ev / gate * 100:.0f}% of the gate", 11.5, "bf-t", "end")
    aria = ("Three rising lines of the net saving from the cheaper induction order against the share of the induction "
            "sub-phase whose cost scales with the step count, for scheduling overheads of 0, 5 and 10 ms. They cross "
            "the 39.2 ms gate at about 62%, 70% and 77%. At 65% with 5 ms overhead the saving is 36.3 ms, 2.78% of "
            "the decode and 93% of the gate. If all of it scaled, 63.5 ms at most.")
    return svg(y1 + 48, o, aria)


# 9. Proposal B, the direct fold ----------------------------------------------------------------
def fold():
    o = []
    def stage(x, y, w, title, sub, fill, stroke, sw=1.2):
        box(o, x, y, w, 52, fill, stroke, 8, sw)
        text(o, x + w / 2, y + 22, title, 12.5, "bf-t-strong", "middle")
        text(o, x + w / 2, y + 39, sub, 11, "bf-tm", "middle")
    text(o, 20, 22, "Today", 13, "bf-t-strong")
    y = 34
    stage(20, y, 190, "MTF, RLE2", "decodes the symbols", "var(--bf-blue-tint)", "var(--bf-blue)")
    arrow(o, 212, y + 26, 248, y + 26)
    stage(250, y, 210, "capped runs, ≤ 127 each", "written out, then read back", "var(--bf-amber-tint)", "var(--bf-amber)", 2)
    arrow(o, 462, y + 26, 498, y + 26)
    stage(500, y, 200, "construction, first pass", "rejoins the maximal runs", "var(--bf-blue-tint)", "var(--bf-blue)")
    for i, s_ in enumerate(("3,990–5,284 extra records a block, 3.5–10.2% of the runs",
                            "about 360 MB written and read back over the workload")):
        text(o, 355, y + 72 + i * 16, s_, 11, "bf-tm", "middle")
    y2 = 160
    text(o, 20, y2 - 12, "The direct fold", 13, "bf-t-strong")
    stage(20, y2, 300, "MTF, RLE2", "emits each maximal run once", "var(--bf-blue-tint)", "var(--bf-blue)")
    arrow(o, 322, y2 + 26, 398, y2 + 26)
    stage(400, y2, 300, "construction", "accumulates the runs as they arrive", "var(--bf-blue-tint)", "var(--bf-blue)")
    text(o, 360, y2 + 18, "directly", 11, "bf-tm", "middle")
    for i, s_ in enumerate(("Forecast (the design analysis): 110–145 ms at one worker; one pass and 180 KB a worker fewer.",
                            "Rejection test: if the first pass's queue pushes, not the rejoining, take most of its time, stop.")):
        text(o, 20, y2 + 80 + i * 17, s_, 11.5, "bf-t" if i == 0 else "bf-tm")
    aria = ("Today: MTF and RLE2 decode the symbols and write capped runs of at most 127, which construction's first "
            "pass reads back and rejoins into maximal runs, about 360 MB of traffic. The direct fold: MTF emits each "
            "maximal run once and construction accumulates it directly, forecast to save 110 to 145 ms at one worker.")
    return svg(y2 + 108, o, aria)


# 10. Proposal C: two-ended against forward lanes on a ring ---------------------------------------
def arc_path(cx, cy, r, f0, f1):
    a0, a1 = 2 * math.pi * f0 - math.pi / 2, 2 * math.pi * f1 - math.pi / 2
    x0, y0 = cx + r * math.cos(a0), cy + r * math.sin(a0)
    x1, y1 = cx + r * math.cos(a1), cy + r * math.sin(a1)
    large = 1 if (f1 - f0) > 0.5 else 0
    return f"M{x0:.1f},{y0:.1f} A{r},{r} 0 {large} 1 {x1:.1f},{y1:.1f}"


def lanes():
    o = []
    r = 82
    # Left: today's two ends.
    cx, cy = 150, 150
    text(o, cx, 24, "Today: two ends of one cycle", 13, "bf-t-strong", "middle")
    gap = 0.008
    o.append(f'<path d="{arc_path(cx, cy, r, gap, 0.5 - gap)}" style="fill:none;stroke:var(--bf-vz-1);stroke-width:9">'
             f'<title>forward from the mark over half the cycle</title></path>')
    o.append(f'<path d="{arc_path(cx, cy, r, 0.5 + gap, 1 - gap)}" style="fill:none;stroke:var(--bf-vz-2);stroke-width:9">'
             f'<title>backward from the mark over the other half</title></path>')
    arrow(o, cx + r * math.cos(0.42 * 2 * math.pi - math.pi / 2) + 4, cy + r * math.sin(0.42 * 2 * math.pi - math.pi / 2) - 8,
          cx + r * math.cos(0.47 * 2 * math.pi - math.pi / 2), cy + r * math.sin(0.47 * 2 * math.pi - math.pi / 2), "var(--bf-ink)", 1.4, 7)
    arrow(o, cx + r * math.cos(0.58 * 2 * math.pi - math.pi / 2) - 4, cy + r * math.sin(0.58 * 2 * math.pi - math.pi / 2) - 8,
          cx + r * math.cos(0.53 * 2 * math.pi - math.pi / 2), cy + r * math.sin(0.53 * 2 * math.pi - math.pi / 2), "var(--bf-ink)", 1.4, 7)
    dot(o, cx, cy - r, "var(--bf-ink)", 5, "the mark: both walks start here")
    text(o, cx, cy - r - 12, "mark", 11, "bf-t", "middle")
    text(o, cx, cy - 6, "forward", 11.5, "bf-t", "middle")
    text(o, cx, cy + 10, "and backward", 11.5, "bf-t", "middle")
    for i, s_ in enumerate(("XOR-linked: each link is prev ⊕ next,", "so a step must know where it came from;",
                            "two chains of dependent reads at once")):
        text(o, cx, cy + r + 30 + i * 16, s_, 11, "bf-tm", "middle")
    # Right: K anchors, forward lanes, with work stealing.
    cx2 = 420
    text(o, cx2, 24, "Proposed: K anchors, forward lanes", 13, "bf-t-strong", "middle")
    anchors = [0.0, 0.07, 0.21, 0.26, 0.48, 0.55, 0.71, 0.93]
    arcs = [(anchors[i], anchors[(i + 1) % len(anchors)] or 1.0) for i in range(len(anchors))]
    lanes_n = 4
    free = [0.0] * lanes_n
    assign = []
    for i, (a, b) in enumerate(arcs):
        ln = min(range(lanes_n), key=lambda j: (free[j], j))
        assign.append((ln, free[ln], free[ln] + (b - a)))
        free[ln] += b - a
    cols = ["var(--bf-vz-1)", "var(--bf-vz-2)", "var(--bf-vz-4)", "var(--bf-vz-3)"]
    for (a, b), (ln, t0, t1) in zip(arcs, assign):
        o.append(f'<path d="{arc_path(cx2, cy, r, a + gap, b - gap)}" style="fill:none;stroke:{cols[ln]};stroke-width:9">'
                 f'<title>arc of {(b - a) * 100:.0f}% of the cycle, walked forward by lane {ln + 1}</title></path>')
    for a in anchors:
        ang = 2 * math.pi * a - math.pi / 2
        dot(o, cx2 + r * math.cos(ang), cy + r * math.sin(ang), "var(--bf-ink)", 4.5, "an anchor: a lane starts here")
    text(o, cx2, cy - 6, "K = 8 anchors", 11.5, "bf-t", "middle")
    text(o, cx2, cy + 10, "L = 4 lanes", 11.5, "bf-t", "middle")
    for i, s_ in enumerate(("forward links only: link = next;", "a lane that finishes its arc", "takes the next unvisited anchor")):
        text(o, cx2, cy + r + 30 + i * 16, s_, 11, "bf-tm", "middle")
    # The lanes' timeline.
    gx0, gx1, gy = 574, 704, 64
    text(o, gx0, 50, "the lanes, in time", 11.5, "bf-t-strong")
    span = max(t for _, _, t in assign)
    sx = (gx1 - gx0) / span
    for ln in range(lanes_n):
        yy = gy + ln * 24
        text(o, gx0 - 6, yy + 12, f"{ln + 1}", 10.5, "bf-tm", "end")
        for (l2, t0, t1) in assign:
            if l2 == ln:
                rect(o, gx0 + t0 * sx + 1, yy, (t1 - t0) * sx - 2, 16, cols[ln], 3, title=f"lane {ln + 1}: {t1 - t0:.2f} of the cycle")
    line(o, gx0 + span * sx, gy - 6, gx0 + span * sx, gy + 4 * 24 - 4, "var(--bf-ink)", 1.2)
    text(o, gx1, gy + 4 * 24 + 12, f"done at {span:.2f} m", 10.5, "bf-tm", "end")
    text(o, gx0, gy + 4 * 24 + 28, "an even split: 0.25 m", 10.5, "bf-tm")
    ty = gy + 4 * 24 + 56
    text(o, gx0 - 14, ty, "Longest of K arcs,", 11.5, "bf-t-strong")
    o.append(f'<text class="bf-t-strong" x="{gx0 - 14}" y="{ty + 16}" style="font-size:11.5px">on average: <tspan style="font-style:italic">m H</tspan><tspan baseline-shift="sub" style="font-size:9px;font-style:italic">K</tspan> / <tspan style="font-style:italic">K</tspan></text>')
    for i, (kk, v) in enumerate(((8, None), (32, None))):
        hk = sum(1 / j for j in range(1, kk + 1))
        text(o, gx0 - 14, ty + 36 + i * 16, f"K = {kk}: {hk / kk:.3f} m", 11.5, "bf-t")
    text(o, gx0 - 14, ty + 68, "8 lanes, even: 0.125 m", 11.5, "bf-tm")
    aria = ("Left: one cycle walked from a mark forward over one half and backward over the other, with XOR links. "
            "Right: the same cycle cut at eight anchors into uneven arcs, walked forward by four lanes, each taking the "
            "next unvisited anchor when it finishes; a timeline shows the lanes finishing near an even split. The "
            "longest of K uniformly placed arcs averages m times H_K over K: 0.340 m for K = 8 and 0.127 m for K = 32, "
            "against 0.125 m for an even split over 8 lanes.")
    return svg(ty + 80, o, aria)


# 11. Proposal D: the pooled scan ---------------------------------------------------------------
def pooled():
    o = []
    def node(x, y, w, h, title, sub, fill, stroke, sw=1.2):
        box(o, x, y, w, h, fill, stroke, 8, sw)
        text(o, x + w / 2, y + h / 2 - 2, title, 12, "bf-t-strong", "middle")
        if sub:
            text(o, x + w / 2, y + h / 2 + 14, sub, 10.5, "bf-tm", "middle")
    for row, (y, label) in enumerate(((40, "Today: one scanner thread, then the workers"),
                                      (232, "Pooled: each worker scans its own block"))):
        text(o, 20, y - 14, label, 13, "bf-t-strong")
        node(20, y + 40, 96, 52, "compressed", "file", "var(--bf-surface)", "var(--bf-frame)")
        arrow(o, 118, y + 66, 140, y + 66)
        if row == 0:
            node(142, y + 30, 150, 72, "scanner thread", "Huffman, block after block", "var(--bf-amber-tint)", "var(--bf-amber)", 2)
            arrow(o, 294, y + 66, 316, y + 66)
            node(318, y + 40, 82, 52, "symbols", "queue", "var(--bf-surface)", "var(--bf-frame)")
            arrow(o, 402, y + 66, 424, y + 66)
            wx, ww, stages = 426, 176, "M · C · E · R"
        else:
            node(142, y + 30, 150, 72, "boundary finder", "48-bit block marker", "var(--bf-surface)", "var(--bf-frame)")
            arrow(o, 294, y + 66, 316, y + 66)
            wx, ww, stages = 318, 284, "scan · M · C · E · R"
        for i in range(3):
            yy = y + i * 46
            col = "var(--bf-blue)"
            node(wx, yy, ww, 38, f"worker {i + 1 if i < 2 else 'k'}", "", "var(--bf-blue-tint)", col)
            o[-2] = o[-2].replace(f'y="{yy + 38 / 2 - 2:.1f}"', f'y="{yy + 15:.1f}"')
            text(o, wx + ww / 2, yy + 31, stages, 10.5, "bf-tm", "middle")
            arrow(o, wx + ww + 2, yy + 19, 626, y + 66, "var(--bf-muted)", 1.2, 6)
        node(628, y + 40, 76, 52, "reader", "in order", "var(--bf-green-tint)", "var(--bf-green)")
    for i, s_ in enumerate(("The same Huffman work, spread over the workers' cores instead of one; the symbols queue, about 400 MB of",
                            "traffic, disappears. A wrong boundary is caught because each block must end where the next begins.")):
        text(o, 20, 392 + i * 16, s_, 11, "bf-tm")
    aria = ("Today the compressed file goes to one scanner thread, which decodes every block's Huffman codes in turn "
            "into a queue of symbols for the workers, which run MTF, construction, emission and RLE1, and a reader "
            "delivers in order. Pooled, a fast boundary finder locates each block's 48-bit marker and every worker "
            "scans its own block before the same stages, removing the serial station and the symbols queue.")
    return svg(424, o, aria)


# 12. Four kinds of number, on one axis at eight workers -----------------------------------------
def landscape():
    o = []
    x0, x1 = 190, 690
    X = lambda ms: x0 + (x1 - x0) * ms / 200
    rows = ["lower bound", "oracle measurement", "predictor", "upper bound"]
    top, rh = 40, 54
    for t in range(0, 201, 25):
        line(o, X(t), top - 8, X(t), top + 4 * rh - 14, "var(--bf-vz-grid)")
        text(o, X(t), top + 4 * rh + 2, f"{t}", 11, "bf-tm", "middle")
    text(o, (x0 + x1) / 2, top + 4 * rh + 22, "elapsed time of one decode at 8 workers, ms", 11.5, "bf-t", "middle")
    for i, r_ in enumerate(rows):
        text(o, x0 - 14, top + i * rh + 14, r_, 12.5, "bf-t-strong", "end")
    line(o, X(195), top - 14, X(195), top + 4 * rh - 14, "var(--bf-ink)", 1.2, ";opacity:0.6")
    text(o, X(195), top - 20, "today, 195", 11, "bf-t", "middle")
    # lower bound
    y = top + 10
    o.append(f'<polygon points="{X(21) - 6:.1f},{y + 6} {X(21) + 6:.1f},{y + 6} {X(21):.1f},{y - 5}" style="fill:var(--bf-vz-3)">'
             f'<title>writing the output once at 40 GB/s: about 21 ms</title></polygon>')
    text(o, X(21) + 10, y + 5, "≈ 21: writing the output once to memory", 11.5, "bf-t")
    # oracle measurements
    y = top + rh + 10
    for ms, lab, anchor, dx in ((125, "125: scanner and construction supplied", "end", -10),
                                (112, "", "end", 0)):
        dot(o, X(ms), y, "var(--bf-vz-1)", 5, f"measured: {ms} ms")
    text(o, X(112) - 10, y + 4, "112: every part supplied", 11.5, "bf-t", "end")
    text(o, X(125) + 10, y - 6, "125: S and C supplied", 11.5, "bf-t")
    # predictors, one sub-row each
    y = top + 2 * rh - 2
    for i, (a, b, lab, side) in enumerate(((14, 17, "14–17: the pipeline's own residual", "right"),
                                           (130, 145, "pooled scan with cheaper construction: 130–145", "left"),
                                           (172, 175, "pooled scan alone: 172–175", "left"))):
        yy = y + i * 15
        rect(o, X(a), yy - 4, max(X(b) - X(a), 4), 8, "var(--bf-vz-2)", 3, title=f"forecast: {a}–{b} ms")
        if side == "right":
            text(o, X(b) + 8, yy + 4, lab, 11, "bf-t")
        else:
            text(o, X(a) - 8, yy + 4, lab, 11, "bf-t", "end")
    # upper bound
    y = top + 3 * rh + 10
    text(o, x0, y + 4, "none proved yet for this decoder", 11.5, "bf-tm", italic=True)
    aria = ("One axis of elapsed time at 8 workers with four rows. Lower bound: about 21 ms to write the output once. "
            "Oracle measurements: 125 ms with the scanner and construction supplied, 112 ms with every part supplied. "
            "Predictors: 130 to 145 ms for a pooled scan with cheaper construction, 172 to 175 ms for the pooled scan "
            "alone, 14 to 17 ms for the pipeline's own residual. Upper bound: none proved yet. Today's decode is "
            "195 ms.")
    return svg(top + 4 * rh + 32, o, aria)


# 13. The decoder's six parts (from figures_ideal_parts.py, unchanged) ----------------------------
def pipeline():
    sys.path.insert(0, __file__.rsplit("/", 1)[0])
    import figures_ideal_parts
    return figures_ideal_parts.pipeline()


FIGS = ["sampling", "matrix", "interaction", "constroom", "twostation", "contraction", "touches", "economics",
        "fold", "lanes", "pooled", "landscape", "pipeline"]


def inject(path):
    lines = open(path).read().split("\n")
    out = []
    i = 0
    while i < len(lines):
        out.append(lines[i])
        m = re.match(r"<!-- fig:(\w+) -->$", lines[i])
        if m:
            out.append(globals()[m.group(1)]())
            if i + 1 < len(lines) and lines[i + 1].startswith("<svg"):
                i += 1
        i += 1
    open(path, "w").write("\n".join(out))


if __name__ == "__main__":
    if sys.argv[1] == "--inject":
        inject(sys.argv[2])
    else:
        print(globals()[sys.argv[1]]())
