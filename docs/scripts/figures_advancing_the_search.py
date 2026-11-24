"""The ten figures of "Advancing the search" (blog/2026-11-24-advancing-the-search.md), drawn as
the objects they show: rings for cycles, ribbons for intervals moved by one translation,
triangles for triangles, and one colour per letter across the post.

Every value is recomputed here and asserted against the value the post states, so a redraw can
change the drawing and never a number. The script rewrites each figure's drawing in the post in
place (the <svg>, or the two-panel <div>, and a stepper's frames); titles and captions are left as
they are. The fx-* classes are in assets/stylesheets/blog.css.

Run from the repository root: python3 docs/scripts/figures_advancing_the_search.py
"""
import html
import math
from fractions import Fraction

POST = "docs/site/blog/2026-11-24-advancing-the-search.md"
DRAWN = {}
LETTER = {"a": "a", "n": "n", "b": "b", "$": "d", "c": "c"}


def f(v):
    return f"{v:.1f}".rstrip("0").rstrip(".")


def esc(t):
    return html.escape(t, quote=False)


def svg_open(w, h, label, extra="", solo=False):
    style = f' class="fx-solo" style="max-width:{w}px"' if solo else ""
    return (f'<svg{style} viewBox="0 0 {w} {h}" role="img" aria-label="{html.escape(label, quote=True)}"'
            f'{extra}>')


def text(x, y, s, cls="bf-t", anchor="middle", extra=""):
    return f'<text class="{cls}" x="{f(x)}" y="{f(y)}" text-anchor="{anchor}"{extra}>{esc(s)}</text>'


def marker(mid):
    return (f'<defs><marker id="{mid}" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" '
            'markerHeight="7" orient="auto-start-reverse"><path class="fx-head" d="M0,1 L9,5 L0,9 Z"/>'
            '</marker></defs>')


def lf_of(L):
    order = sorted(set(L))
    C = {c: sum(1 for x in L if x < c) for c in order}
    seen, out = {}, []
    for ch in L:
        out.append(C[ch] + seen.get(ch, 0))
        seen[ch] = seen.get(ch, 0) + 1
    return out


def runs_of(L):
    rs, i = [], 0
    while i < len(L):
        j = i
        while j + 1 < len(L) and L[j + 1] == L[i]:
            j += 1
        rs.append((i, j))
        i = j + 1
    return rs


def ribbons(L, xl, xr, top, ch, bw, label_shift=True, cls_of=lambda c: LETTER.get(c, "d"), rows=True):
    """Two columns L and F joined by one ribbon per run of L."""
    n = len(L)
    F = "".join(sorted(L))
    LF = lf_of(L)
    out, labels = [], []
    for i0, i1 in runs_of(L):
        d = LF[i0] - i0
        yl0, yl1 = top + i0 * ch, top + (i1 + 1) * ch
        yr0, yr1 = top + (i0 + d) * ch, top + (i1 + 1 + d) * ch
        x0, x1 = xl + bw, xr
        m = (x0 + x1) / 2
        out.append(f'<path class="fx-ribbon fx-{cls_of(L[i0])}" d="M{f(x0)},{f(yl0)} C{f(m)},{f(yl0)} '
                   f'{f(m)},{f(yr0)} {f(x1)},{f(yr0)} L{f(x1)},{f(yr1)} C{f(m)},{f(yr1)} {f(m)},{f(yl1)} '
                   f'{f(x0)},{f(yl1)} Z"/>')
        if label_shift:
            sign = "+" if d > 0 else ("−" if d < 0 else "±")
            labels.append(text(x0 + 8, (yl0 + yl1) / 2 + 4, f"{sign}{abs(d)}", "fx-shift", "start"))
    for i in range(n):
        y = top + i * ch
        for x, c in ((xl, L[i]), (xr, F[i])):
            out.append(f'<rect class="fx-cell fx-{cls_of(c)}" x="{f(x)}" y="{f(y)}" width="{bw}" height="{ch}"/>')
            out.append(text(x + bw / 2, y + ch / 2 + 5, c, "fx-letter"))
        if rows:
            out.append(text(xl - 9, y + ch / 2 + 4, str(i), "fx-row", "end"))
    for i0, i1 in runs_of(L):
        out.append(f'<rect class="fx-run" x="{f(xl)}" y="{f(top + i0 * ch)}" width="{bw}" '
                   f'height="{f((i1 - i0 + 1) * ch)}"/>')
    out.extend(labels)
    return out


def ring(order, letters, cx, cy, R, mid, r_node=15, steps=True, row_labels=True, cls_of=None):
    """A cycle drawn as a ring of nodes in visiting order, with arcs between them."""
    cls_of = cls_of or (lambda c: LETTER.get(c, "d"))
    k = len(order)
    pos = {}
    out = []
    for s, row in enumerate(order):
        a = -math.pi / 2 + 2 * math.pi * s / k
        pos[row] = (cx + R * math.cos(a), cy + R * math.sin(a), a)
    for s in range(k):
        r0, r1 = order[s], order[(s + 1) % k]
        a0 = pos[r0][2] + r_node / R * 1.15
        a1 = pos[r1][2] - r_node / R * 1.3
        if a1 < a0:
            a1 += 2 * math.pi
        large = 1 if a1 - a0 > math.pi else 0
        out.append(f'<path class="fx-loop" d="M{f(cx + R * math.cos(a0))},{f(cy + R * math.sin(a0))} '
                   f'A{R},{R} 0 {large} 1 {f(cx + R * math.cos(a1))},{f(cy + R * math.sin(a1))}" '
                   f'marker-end="url(#{mid})"/>')
        if steps:
            am = (a0 + a1) / 2
            out.append(text(cx + (R + 14) * math.cos(am), cy + (R + 14) * math.sin(am) + 4, str(s + 1), "fx-step"))
    for row in order:
        x, y, a = pos[row]
        ch = letters[row]
        out.append(f'<circle class="fx-node fx-{cls_of(ch)}" cx="{f(x)}" cy="{f(y)}" r="{r_node}"/>')
        out.append(text(x, y + 5, ch, "fx-letter"))
        if row_labels:
            out.append(text(cx + (R - r_node - 15) * math.cos(a), cy + (R - r_node - 15) * math.sin(a) + 4,
                            str(row), "fx-row"))
    return out



def step_svg(first, label):
    """The stepper's own <svg>: frame 1's drawing, the class blog.js looks for, hidden until it runs."""
    head, rest = first.split(">", 1)
    w = __import__("re").search(r'viewBox="0 0 (\d+)', head).group(1)
    head = head.replace("<svg viewBox", f'<svg class="bf-step-svg fx-solo" style="max-width:{w}px" viewBox', 1)
    head = __import__("re").sub(r'aria-label="[^"]*"', f'aria-label="{html.escape(label, quote=True)}"', head)
    return head + " hidden>" + rest


def write(name, content):
    DRAWN[name] = content


# ---------------------------------------------------------------- figure 1
def fig1():
    T = "banana$"
    n = len(T)
    rots = sorted(T[i:] + T[:i] for i in range(n))
    L = "".join(r[-1] for r in rots)
    assert L == "annb$aa"
    LF = lf_of(L)
    walk = [L.index("$")]
    while len(walk) < n:
        walk.append(LF[walk[-1]])
    assert walk == [4, 0, 1, 5, 2, 6, 3] and LF[walk[-1]] == walk[0]
    assert "".join(L[r] for r in reversed(walk)) == "banana$"
    assert [LF[a] - a for a, _ in runs_of(L)] == [1, 4, 1, -4, -3]
    W, H = 360, 312
    a = [svg_open(W, H, "The seven rows of the BWT of banana$ on a loop in LF's order, one dependent "
                        "lookup per step."), marker("fx1-head"),
         text(180, 20, "The walk: one lookup per row", "bf-ts")]
    a += ring(walk, L, 180, 158, 94, "fx1-head", r_node=16)
    a += [text(180, 154, "7 lookups,", "fx-centre"), text(180, 171, "each one waiting", "fx-centre"),
          text(180, 188, "for the last", "fx-centre"),
          text(180, H - 24, "small numbers are rows; read the letters", "bf-tm"),
          text(180, H - 9, "backwards round the loop: banana$", "bf-tm"), "</svg>"]
    b = [svg_open(W, H, "The last column L and the sorted column F as two bars; each of the five runs "
                        "is a ribbon that carries a whole interval of rows across with one translation."),
         text(180, 20, "Induction: one translation per run", "bf-ts"),
         text(83, 46, "L", "fx-math"), text(279, 46, "F", "fx-math")]
    b += ribbons(L, 66, 262, 58, 30, 34)
    b += [text(180, H - 24, "5 runs, so 5 translations", "bf-tm"),
          text(180, H - 9, "carry all 7 rows", "bf-tm"), "</svg>"]
    write("fig1.svg", '<div class="fx-panels">' + "".join(a) + "".join(b) + "</div>")


# ---------------------------------------------------------------- figure 2 (stepper)
def fig2():
    pairs = [(7, 5)]
    while len(pairs) < 5:
        s, c = pairs[-1]
        pairs.append((3 * s + 4 * c, 2 * s + 3 * c))
    assert pairs == [(7, 5), (41, 29), (239, 169), (1393, 985), (8119, 5741)]
    tri = []
    for s, c in pairs:
        assert s * s - 2 * c * c == -1
        p = (s - 1) // 2
        assert p + (p + 1) == s and p * p + (p + 1) ** 2 == c * c
        tri.append((p, p + 1, c))
    ratio = [f"{s / c:.10f}" for s, c in pairs]
    assert ratio == ["1.4000000000", "1.4137931034", "1.4142011834", "1.4142131980", "1.4142135516"]
    dist = [f"{abs(s / c - math.sqrt(2)):.1e}" for s, c in pairs]
    assert dist == ["1.4e-02", "4.2e-04", "1.2e-05", "3.6e-07", "1.1e-08"]
    for (s, c), (s0, c0) in zip(pairs[1:], pairs):
        assert (3 * s - 4 * c, 3 * c - 2 * s) == (s0, c0)
    texts = [
        "Start from the 3-4-5 triangle: its short sides sum to s = 7 and its long side is c = 5, and 7² − 2·5² = −1.",
        "One application of the rule gives (41, 29): the 20-21-29 triangle, still one seed short of the square on the diagonal.",
        "Again: (239, 169), the 119-120-169 triangle. The ratio s/c is now within about 10⁻⁵ of √2.",
        "Again: (1,393, 985). Each answer comes from the last by the same rule, without solving anything afresh.",
        "Run the rule backwards and every pair gets smaller. A pair with s² = 2c² exactly would also get smaller forever, which whole numbers can't do: the same rule that generates the answers proves √2 has no ratio.",
    ]
    W, H = 360, 470
    num = lambda v: f"{v:,}"
    sup = str.maketrans("-0123456789", "⁻⁰¹²³⁴⁵⁶⁷⁸⁹")

    def frame(k):
        back = k == 4
        kk = 4 if back else k
        s, c = pairs[kk]
        p, q, h = tri[kk]
        o = [svg_open(W, H, f"The pair (s, c) = ({num(s)}, {num(c)}), its right triangle {num(p)}, {num(q)}, {num(h)}, "
                            "and how close s/c is to the square root of two."), marker("fx2-head")]
        o.append(text(180, 24, "(s, c) → (3s − 4c, 3c − 2s)" if back else "(s, c) → (3s + 4c, 2s + 3c)", "fx-rule"))
        o.append(text(180, 43, "the rule run backwards: every pair shrinks" if back else "keeps s² − 2c² = −1", "bf-tm"))
        # the triangle to scale, hypotenuse fixed; earlier ones dashed at the same corner
        hyp = 150
        x0, y0 = 120, 214
        sc_ = hyp / h
        if not back:
            for j in range(kk):
                pj, qj, hj = tri[j]
                sj = hyp / hj
                o.append(f'<path class="fx-ghost" d="M{x0},{y0} L{f(x0 + pj * sj)},{y0} L{x0},{f(y0 - qj * sj)} Z"/>')
        x1, y2 = x0 + p * sc_, y0 - q * sc_
        o.append(f'<path class="fx-tri" d="M{x0},{y0} L{f(x1)},{y0} L{x0},{f(y2)} Z"/>')
        o.append(f'<path class="fx-right" d="M{x0 + 11},{y0} L{x0 + 11},{y0 - 11} L{x0},{y0 - 11}"/>')
        o.append(text((x0 + x1) / 2, y0 + 18, num(p), "fx-side"))
        o.append(text(x0 - 8, (y0 + y2) / 2 + 4, num(q), "fx-side", "end"))
        o.append(text((x0 + x1) / 2 + 10, (y0 + y2) / 2 - 4, num(h), "fx-side", "start"))
        o.append(text(180, 256, f"s = {num(s)} · c = {num(c)}", "fx-pair"))
        o.append(text(180, 274, "s² − 2c² = −1 at every size" if back else "short sides differ by one", "bf-tm"))
        # the ladder: distance of s/c from root two, log scale, one rung per pair
        ax, top, bot = 70, 312, 438
        o.append(text(180, 300, "distance of s/c from √2", "bf-tm"))
        o.append(f'<path class="fx-axis" d="M{ax},{top} L{ax},{bot}"/>')
        hi, lo = -1, -9
        ypos = lambda e: top + (e - hi) / (lo - hi) * (bot - top)
        for e in (-1, -3, -5, -7, -9):
            yy = ypos(e)
            o.append(f'<path class="fx-tick" d="M{ax - 4},{f(yy)} L{ax},{f(yy)}"/>')
            o.append(text(ax - 8, yy + 4, "10" + str(e).translate(sup), "fx-row", "end"))
        shown = range(5) if back else range(kk + 1)
        for j in shown:
            sj, cj = pairs[j]
            yy = ypos(math.log10(abs(sj / cj - math.sqrt(2))))
            cur = j == kk and not back
            o.append(f'<circle class="{"fx-dot-on" if cur else "fx-dot"}" cx="{ax}" cy="{f(yy)}" r="{5 if cur else 3.5}"/>')
            o.append(text(ax + 14, yy + 4, f"{num(sj)}/{num(cj)}", "fx-frac" if cur else "fx-row", "start"))
            o.append(text(300, yy + 4, dist[j], "fx-frac" if cur else "fx-row", "end"))
        if back:
            ya = ypos(math.log10(abs(pairs[4][0] / pairs[4][1] - math.sqrt(2))))
            yb = ypos(math.log10(abs(pairs[0][0] / pairs[0][1] - math.sqrt(2))))
            o.append(f'<path class="fx-back" d="M{312},{f(ya)} C{344},{f(ya)} {344},{f(yb)} {314},{f(yb)}" '
                     'marker-end="url(#fx2-head)"/>')
            o.append(text(180, H - 10, "An exact s² = 2c² would shrink forever, so none exists.", "fx-pair"))
        else:
            o.append(text(180, H - 10, f"s / c = {ratio[kk]}", "fx-pair"))
        o.append("</svg>")
        return "".join(o)

    frames = [frame(k) for k in range(5)]
    write("fig2.svg", step_svg(frames[0], "The pairs (7, 5), (41, 29), (239, 169), (1,393, 985) and (8,119, 5,741), each a right triangle with short sides one apart, generated by one rule and closing in on the square root of two; run backwards, the rule shrinks every pair."))
    write("fig2-frames.html", "".join(
        f'<template class="bf-frame" data-text="{html.escape(t, quote=True)}">{fr}</template>'
        for t, fr in zip(texts, frames)))


# ---------------------------------------------------------------- figure 3
def fig3():
    W, H = 360, 330
    a = [svg_open(W, H, "Search by import: the inversion problem at the centre, four distant fields on rays, "
                        "each with its own verdict: three closed, one kept."),
         text(180, 20, "Search by import", "bf-ts")]
    cx, cy = 180, 168
    for r in (58, 96):
        a.append(f'<circle class="fx-orbit" cx="{cx}" cy="{cy}" r="{r}"/>')
    fields = [("topology", "closed", -150), ("algebra", "closed", -30), ("dynamics", "kept", 30),
              ("combinatorics", "closed", 150)]
    for name, verdict, ang in fields:
        t = math.radians(ang)
        ex, ey = cx + 96 * math.cos(t), cy + 96 * math.sin(t)
        sx, sy = cx + 40 * math.cos(t), cy + 40 * math.sin(t)
        a.append(f'<path class="fx-ray" d="M{f(sx)},{f(sy)} L{f(ex)},{f(ey)}"/>')
        if verdict == "kept":
            a.append(f'<circle class="fx-kept" cx="{f(ex)}" cy="{f(ey)}" r="9"/>')
        else:
            nx, ny = -math.sin(t), math.cos(t)
            a.append(f'<path class="fx-shut" d="M{f(ex - 9 * nx)},{f(ey - 9 * ny)} L{f(ex + 9 * nx)},{f(ey + 9 * ny)}"/>')
        lx, ly = cx + 120 * math.cos(t), cy + 120 * math.sin(t) + (16 if math.sin(t) > 0 else -10)
        anchor = "middle"
        a.append(text(lx, ly - 2, name, "bf-t", anchor))
        a.append(text(lx, ly + 13, verdict, "fx-verdict-kept" if verdict == "kept" else "fx-verdict", anchor))
    a.append(f'<circle class="fx-hub" cx="{cx}" cy="{cy}" r="38"/>')
    a += [text(cx, cy - 3, "the inversion", "fx-hub-t"), text(cx, cy + 12, "problem", "fx-hub-t"),
          text(180, H - 24, "each verdict stands alone;", "bf-tm"),
          text(180, H - 9, "a closed door says nothing about the next", "bf-tm"), "</svg>"]
    b = [svg_open(W, H, "Search between the routes: the walk, Cells and Open joined by one contraction, which "
                        "keeps the text and the cycle and removes interior steps; its invariant, the width-1 rows, "
                        "says where a new route must act."), marker("fx3-head"),
         text(180, 20, "Search between the routes", "bf-ts")]
    xs = [60, 180, 300]
    for x, name in zip(xs, ["walk", "Cells", "Open"]):
        b.append(f'<circle class="fx-station" cx="{x}" cy="70" r="9"/>')
        b.append(text(x, 52, name, "bf-ts"))
    for x0, x1 in ((60, 180), (180, 300)):
        b.append(f'<path class="fx-map" d="M{x0 + 12},70 C{x0 + 50},106 {x1 - 50},106 {x1 - 12},70" '
                 'marker-end="url(#fx3-head)"/>')
    b.append(text(180, 124, "the map between them: contraction", "fx-map-t"))
    b.append('<rect class="fx-ledger" x="40" y="140" width="280" height="88" rx="3"/>')
    b += [text(56, 164, "keeps", "fx-key", "start"), text(136, 164, "the text, the cycle", "bf-t", "start"),
          text(56, 188, "removes", "fx-key", "start"), text(136, 188, "interior steps", "bf-t", "start"),
          text(56, 212, "invariant", "fx-key", "start"), text(136, 212, "width-1 rows", "fx-inv", "start")]
    b.append('<path class="fx-flow" d="M180,232 L180,256" marker-end="url(#fx3-head)"/>')
    b.append('<rect class="fx-target" x="70" y="262" width="220" height="34" rx="17"/>')
    b.append(text(180, 284, "where a new route must act", "fx-target-t"))
    b.append(text(180, H - 9, "one object, and what every candidate must change", "bf-tm"))
    b.append("</svg>")
    write("fig3.svg", '<div class="fx-panels">' + "".join(a) + "".join(b) + "</div>")


# ---------------------------------------------------------------- figures 4 and 9, drawn downwards for a phone
def fig4_narrow(walk, cells, opn, w1, N):
    W = 340
    Y0, RH = 52, 22
    H = Y0 + N * RH + 56
    cols = [("The walk", walk, 58), ("Cells", cells, 152), ("Open", opn, 246)]
    CWD = 84
    o = [svg_open(W, H, "Three tilings of the same stretch of a block's cycle, read downwards: the walk takes one step "
                        "per row, Cells merges every wide stretch into one step, Open merges only the thick stretches; "
                        "the width-1 rows stay single steps in all three.", solo=True),
         text(14, 18, "one stretch of a block's cycle, in output order ↓", "bf-tm", "start")]
    for i in sorted(w1):
        o.append(f'<rect class="fx-w1-band" x="44" y="{Y0 + i * RH}" width="{W - 48}" height="{RH}"/>')
    for i in range(N + 1):
        o.append(f'<path class="fx-grid" d="M44,{Y0 + i * RH} L{W - 4},{Y0 + i * RH}"/>')
    for i in range(N):
        o.append(text(36, Y0 + i * RH + 15, str(i), "fx-row", "end"))
    for name, tiling, x in cols:
        o.append(text(x + CWD / 2, Y0 - 10, name, "bf-ts"))
        for a, b, lab in tiling:
            y = Y0 + a * RH + 2
            h = (b - a + 1) * RH - 4
            cls = "fx-tile-w1" if lab == "w1" else ("fx-tile-big" if b > a else "fx-tile")
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{CWD}" height="{h}" rx="3"/>')
            if b > a:
                for i in range(a + 1, b + 1):
                    yy = Y0 + i * RH
                    o.append(f'<path class="fx-seam" d="M{x + 8},{yy} L{x + CWD - 8},{yy}"/>')
                parts = lab.split(", ")
                for j, part in enumerate(parts):
                    o.append(text(x + CWD / 2, y + h / 2 + 4 + (j - (len(parts) - 1) / 2) * 14, part, "fx-tile-t"))
            elif lab == "w1":
                o.append(text(x + CWD / 2, y + h / 2 + 4, "w1", "fx-w1-t"))
    o.append(text(14, H - 26, "w1: a width-1 row, a single tile under every", "bf-tm", "start"))
    o.append(text(14, H - 10, "contraction; a longer tile is many rows in one step", "bf-tm", "start"))
    o.append("</svg>")
    return "".join(o)


def fig9_narrow(L, p1, p3):
    n = len(L)
    W, RH, Y0 = 340, 24, 58
    H = Y0 + n * RH + 112
    cx, cw = 170, 34
    o = [svg_open(W, H, "The 18-row column Paaabxbccddeeff$MZ read downwards, one LF cycle. Paths from rows 1 and 3 "
                        "both spell abcdef; their second rows, 4 and 6, are single b's.", solo=True),
         marker("fx9n-head"),
         text(cx, 20, "row 1 spells a b c d e f", "fx-path-a-t"),
         text(cx, 37, "along 1 → 4 → 7 → 9 → 11 → 13", "fx-path-a-t"),
         text(cx, Y0 + n * RH + 24, "row 3 spells a b c d e f", "fx-path-b-t"),
         text(cx, Y0 + n * RH + 41, "along 3 → 6 → 8 → 10 → 12 → 14", "fx-path-b-t")]
    for i, ch in enumerate(L):
        y = Y0 + i * RH
        cls = "fx-cell9"
        if i in (1, 3):
            cls += " fx-start"
        if i in (4, 6):
            cls += " fx-w1cell"
        o.append(f'<rect class="{cls}" x="{cx - cw / 2}" y="{y + 1}" width="{cw}" height="{RH - 2}" rx="3"/>')
        o.append(text(cx, y + 17, ch, "fx-letter"))
        o.append(text(cx + cw / 2 + 64, y + 16, str(i), "fx-row", "end"))
    for a, b in runs_of(L):
        if b > a:
            o.append(f'<path class="fx-runline" d="M{cx + cw / 2 + 6},{Y0 + a * RH + 3} L{cx + cw / 2 + 6},{Y0 + b * RH + RH - 3}"/>')
    cy = lambda i: Y0 + i * RH + RH / 2
    for pth, left, cls in ((p1, True, "fx-path-a"), (p3, False, "fx-path-b")):
        for u, v in zip(pth, pth[1:]):
            y0, y1 = cy(u), cy(v)
            reach = 14 + 7 * (v - u)
            if left:
                x0 = cx - cw / 2 - 3
                d = f"M{f(x0)},{f(y0)} C{f(x0 - reach)},{f(y0)} {f(x0 - reach)},{f(y1)} {f(x0 - 5)},{f(y1)}"
            else:
                x0 = cx + cw / 2 + 14
                d = f"M{f(x0)},{f(y0)} C{f(x0 + reach)},{f(y0)} {f(x0 + reach)},{f(y1)} {f(x0 + 5)},{f(y1)}"
            o.append(f'<path class="{cls}" d="{d}" marker-end="url(#fx9n-head)"/>')
    o += [text(cx, H - 42, "rows 4 and 6 are width-1 runs of b;", "bf-tm"),
          text(cx, H - 26, "their own six-letter contexts,", "bf-tm"),
          text(cx, H - 10, "bcdef$ and bcdefM, are unique", "bf-tm"), "</svg>"]
    return "".join(o)


# ---------------------------------------------------------------- figure 4
def fig4():
    w1 = {3, 8, 9, 13}
    N = 16
    cells = [(0, 2, "3 rows, 1 step"), (3, 3, "w1"), (4, 7, "4 rows, 1 step"), (8, 8, "w1"), (9, 9, "w1"),
             (10, 12, "3 rows, 1 step"), (13, 13, "w1"), (14, 15, "2 → 1")]
    opn = [(0, 2, "3 rows, 1 step"), (3, 3, "w1"), (4, 7, "4 rows, 1 step"), (8, 8, "w1"), (9, 9, "w1"),
           (10, 10, ""), (11, 11, ""), (12, 12, ""), (13, 13, "w1"), (14, 15, "2 → 1")]
    walk = [(i, i, "w1" if i in w1 else "") for i in range(N)]
    for tiling in (walk, cells, opn):
        cover = sorted(i for a, b, _ in tiling for i in range(a, b + 1))
        assert cover == list(range(N))
        for a, b, lab in tiling:
            assert (lab == "w1") == (a == b and a in w1)
    W, H = 640, 236
    X0, CW = 96, 33
    o = [svg_open(W, H, "Three tilings of the same stretch of a block's cycle: the walk takes one step per row, "
                        "Cells merges every wide stretch into one step, Open merges only the thick stretches; "
                        "the width-1 rows stay single steps in all three."),
         text(X0, 22, "a stretch of one block's cycle, in output order →", "bf-tm", "start")]
    for i in range(N + 1):
        x = X0 + i * CW
        o.append(f'<path class="fx-grid" d="M{x},34 L{x},196"/>')
    for i in sorted(w1):
        o.append(f'<rect class="fx-w1-band" x="{X0 + i * CW}" y="34" width="{CW}" height="162"/>')
    rows = [("The walk", walk, 44), ("Cells", cells, 98), ("Open", opn, 152)]
    for name, tiling, y in rows:
        o.append(text(X0 - 12, y + 22, name, "bf-ts", "end"))
        for a, b, lab in tiling:
            x = X0 + a * CW + 2
            w = (b - a + 1) * CW - 4
            cls = "fx-tile-w1" if lab == "w1" else ("fx-tile-big" if b > a else "fx-tile")
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="32" rx="3"/>')
            if b > a:
                for i in range(a + 1, b + 1):
                    xx = X0 + i * CW
                    o.append(f'<path class="fx-seam" d="M{xx},{y + 6} L{xx},{y + 26}"/>')
                o.append(text(x + w / 2, y + 20, lab, "fx-tile-t"))
            elif lab == "w1":
                o.append(text(x + w / 2, y + 20, "w1", "fx-w1-t"))
    o.append(text(X0, 220, "w1: a width-1 row, a single tile under every contraction; "
                           "a longer tile is many rows taken in one step", "bf-tm", "start"))
    o.append("</svg>")
    write("fig4.svg", '<div class="fx-switch"><div class="fx-wide">' + "".join(o) + '</div><div class="fx-narrow">'
          + fig4_narrow(walk, cells, opn, w1, N) + "</div></div>")


# ---------------------------------------------------------------- figure 5 (stepper)
def fig5():
    L = "annb$na"
    F = "".join(sorted(L))
    assert F == "$aabnnn"
    LF = lf_of(L)
    sel = [0, 1, 3, 5, 6]
    dest = [LF[x] for x in sel]
    assert dest == [1, 4, 3, 6, 2]
    grouped = sorted(dest)
    assert grouped == [1, 2, 3, 4, 6]
    by = {}
    for x in sel:
        by.setdefault(L[x], []).append(LF[x])
    assert by == {"a": [1, 2], "n": [4, 6], "b": [3]}
    assert [d for ch in sorted(by) for d in by[ch]] == grouped
    assert LF[6] - LF[0] == L[0:6].count("a") == 1
    assert LF[5] - LF[1] == L[1:5].count("n") == 2
    texts = [
        "The column L = annb$na, with five scattered rows selected and listed in row order.",
        "Apply LF to each: LF(x) is the first sorted row of x's letter plus the number of earlier copies of that letter. The destinations 1, 4, 3, 6, 2 are out of order.",
        "Group them by the letter each row emits, keeping order inside each group. Within one letter LF keeps order, and smaller letters land lower, so the result 1, 2, 3, 4, 6 is sorted again.",
        "Why: within one letter, the gap between two destinations is the number of that letter between the two source rows. One step of the frontier is a set of gap counts.",
    ]
    W, H = 380, 360
    X0, CW, CH = 46, 46, 34
    yL, yF = 72, 230
    mid = 190

    def column(y, s, name, mark=(), dim=False, nums_below=False):
        o = [text(X0 - 12, y + CH / 2 + 6, name, "fx-math", "end")]
        for i, ch in enumerate(s):
            x = X0 + i * CW
            cls = f"fx-cell fx-{LETTER[ch]}" + (" fx-dim" if dim and i not in mark else "")
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{CW - 4}" height="{CH}" rx="3"/>')
            o.append(text(x + (CW - 4) / 2, y + CH / 2 + 5, ch, "fx-letter"))
            o.append(text(x + (CW - 4) / 2, y + CH + 14 if nums_below else y - 7, str(i), "fx-row"))
            if i in mark:
                o.append(f'<rect class="fx-sel" x="{x - 3}" y="{y - 3}" width="{CW + 2}" height="{CH + 6}" rx="5"/>')
        return o

    def arrow(i, j, cls):
        x0 = X0 + i * CW + (CW - 4) / 2
        x1 = X0 + j * CW + (CW - 4) / 2
        return (f'<path class="fx-lf {cls}" d="M{f(x0)},{yL + CH + 5} C{f(x0)},{(yL + yF) / 2 + 24} '
                f'{f(x1)},{(yL + yF) / 2 - 24} {f(x1)},{yF - 7}" marker-end="url(#fx5-head)"/>')

    def lines(y, ls, cls="fx-pair"):
        return [text(mid, y + 17 * k, t, cls) for k, t in enumerate(ls)]

    frames = []
    o = [svg_open(W, H, texts[0]), marker("fx5-head")] + column(yL, L, "L", sel, dim=True)
    o += lines(160, ["selected rows, in row order:", "0, 1, 3, 5, 6"])
    o += lines(206, ["scattered: no two of them need be neighbours"], "bf-tm")
    o.append("</svg>")
    frames.append("".join(o))
    o = [svg_open(W, H, texts[1]), marker("fx5-head")] + column(yL, L, "L", sel, dim=True) + column(yF, F, "F", nums_below=True)
    for x in sel:
        o.append(arrow(x, LF[x], f"fx-{LETTER[L[x]]}"))
    o += lines(H - 42, ["LF of each, in source order:", "1, 4, 3, 6, 2 (not sorted)"])
    o.append("</svg>")
    frames.append("".join(o))
    o = [svg_open(W, H, texts[2]), marker("fx5-head")] + column(yL, L, "L", sel, dim=True) + column(yF, F, "F", grouped, dim=True, nums_below=True)
    for x in sel:
        o.append(arrow(x, LF[x], f"fx-{LETTER[L[x]]}"))
    for ch, ds in sorted(by.items()):
        xa = X0 + min(ds) * CW
        xb = X0 + max(ds) * CW + CW - 4
        yb = yF + CH + 24
        o.append(f'<path class="fx-brace fx-{LETTER[ch]}" d="M{xa},{yb} L{xa},{yb + 5} L{xb},{yb + 5} L{xb},{yb}"/>')
        o.append(text((xa + xb) / 2, yb + 21, f"{ch} → {', '.join(map(str, ds))}", "fx-brace-t"))
    o += lines(22, ["the new frontier 1, 2, 3, 4, 6 is sorted,", "with no comparisons"])
    o.append("</svg>")
    frames.append("".join(o))
    o = [svg_open(W, H, texts[3]), marker("fx5-head")] + column(yL, L, "L", [0, 6, 1, 5], dim=True) + column(yF, F, "F", [1, 2, 4, 6], dim=True, nums_below=True)
    for x in (0, 6, 1, 5):
        o.append(arrow(x, LF[x], f"fx-{LETTER[L[x]]}"))
    o += lines(22, ["within one letter, destinations differ by a count"])
    o += lines(H - 60, ["a at rows 0 and 6: one a in rows 0–5, so", "LF(6) − LF(0) = 2 − 1 = 1"], "bf-t")
    o += lines(H - 24, ["n at rows 1 and 5: two n's in rows 1–4, so", "LF(5) − LF(1) = 6 − 4 = 2"], "bf-t")
    o.append("</svg>")
    frames.append("".join(o))
    write("fig5.svg", step_svg(frames[0], "The column annb$na with rows 0, 1, 3, 5 and 6 selected; LF sends them to 1, 4, 3, 6, 2, and grouping by letter sorts them to 1, 2, 3, 4, 6 with no comparisons."))
    write("fig5-frames.html", "".join(
        f'<template class="bf-frame" data-text="{html.escape(t, quote=True)}">{fr}</template>'
        for t, fr in zip(texts, frames)))


# ---------------------------------------------------------------- figure 6
def fig6():
    ids = ["a7f92b01", "3bc94e72", "fe7204a9"]
    assert sum(len(s) for s in ids) == 24 and all(len(s) == 8 for s in ids)
    W, H = 360, 300
    a = [svg_open(W, H, "One ID at a time: three chains of eight dependent steps, each letter waiting for the one before."),
         marker("fx6-head"), text(180, 20, "One ID at a time", "bf-ts")]
    for r, s in enumerate(ids):
        y = 70 + r * 62
        a.append(text(28, y + 4, f"ID {r + 1}", "fx-row", "middle"))
        for k, ch in enumerate(s):
            x = 64 + k * 37
            if k:
                a.append(f'<path class="fx-chain" d="M{x - 26},{y} L{x - 12},{y}" marker-end="url(#fx6-head)"/>')
            a.append(f'<circle class="fx-bead" cx="{x}" cy="{y}" r="12"/>')
            a.append(text(x, y + 5, ch, "fx-letter-s"))
    a += [text(180, H - 46, "24 letters in 3 chains", "fx-pair"),
          text(180, H - 26, "of 8 dependent steps", "fx-pair"),
          text(180, H - 8, "each step waits for the one before", "bf-tm"), "</svg>"]
    b = [svg_open(W, H, "Phase by phase: the same 24 letters in eight rounds; each round moves all three records "
                        "and re-sorts them."),
         text(180, 20, "Phase by phase", "bf-ts")]
    X0, CW, Y0, RH = 48, 36, 46, 44
    for k in range(8):
        x = X0 + k * CW
        b.append(f'<rect class="fx-round" x="{x - 15}" y="{Y0}" width="30" height="{RH * 3 + 8}" rx="15"/>')
        for r in range(3):
            y = Y0 + 26 + r * RH
            b.append(text(x, y, ids[r][k], "fx-letter-s"))
        b.append(text(x, Y0 + RH * 3 + 26, str(k + 1), "fx-row"))
    yb = Y0 + RH * 3 + 40
    b.append(f'<path class="fx-brace" d="M{X0 - 15},{yb} L{X0 - 15},{yb + 5} L{X0 + 7 * CW + 15},{yb + 5} L{X0 + 7 * CW + 15},{yb}"/>')
    b.append(text(X0 + 3.5 * CW, yb + 22, "8 rounds", "fx-pair"))
    xr = X0 + 7 * CW + 26
    b.append(f'<path class="fx-brace" d="M{xr},{Y0 + 4} L{xr + 5},{Y0 + 4} L{xr + 5},{Y0 + RH * 3 + 4} L{xr},{Y0 + RH * 3 + 4}"/>')
    b.append(text(xr + 12, Y0 + RH * 1.5 - 4, "3 in", "fx-pair", "start"))
    b.append(text(xr + 12, Y0 + RH * 1.5 + 14, "flight", "fx-pair", "start"))
    b += [text(180, H - 26, "each round moves all 3 records, re-sorted", "bf-tm"),
          text(180, H - 8, "the rounds don't reduce the 24 letters' worth of work", "bf-tm"), "</svg>"]
    write("fig6.svg", '<div class="fx-panels">' + "".join(a) + "".join(b) + "</div>")


# ---------------------------------------------------------------- figure 7
def fig7():
    st = [("run records", ["what bzip2 gives"], "in hand"),
          ("find the ID starts", ["one scan of a contiguous range"], "in hand"),
          ("move the frontier", ["G·d events record by record;", "a packed partition unpriced"], "open"),
          ("place each ID in the output", ["not yet constructed"], "open"),
          ("assemble with Open's residual", ["not yet constructed"], "open")]
    W, H = 340, 330
    o = [svg_open(W, H, "Five stages from run records to assembled text; the first two are in hand, the last three open.", solo=True)]
    x = 22
    ys = [28, 90, 152, 228, 290]
    o.append(f'<path class="fx-line-done" d="M{x},{ys[0]} L{x},{ys[1]}"/>')
    o.append(f'<path class="fx-line-open" d="M{x},{ys[1]} L{x},{ys[4]}"/>')
    for (name, sub, state), y in zip(st, ys):
        frontier = name == "move the frontier"
        cls = "fx-stop-done" if state == "in hand" else ("fx-stop-front" if frontier else "fx-stop-open")
        o.append(f'<circle class="{cls}" cx="{x}" cy="{y}" r="{11 if frontier else 8}"/>')
        o.append(text(x + 24, y + 5, name, "bf-ts", "start"))
        o.append(text(W - 4, y + 5, state, "fx-state-done" if state == "in hand" else "fx-state-open", "end"))
        for j, line in enumerate(sub):
            o.append(text(x + 24, y + 23 + 15 * j, line, "bf-tm", "start"))
    o.append("</svg>")
    write("fig7.svg", "".join(o))


# ---------------------------------------------------------------- figure 8
def fig8():
    code = {"a": 0, "b": 1, "c": 2}
    panels = []
    for L, A_txt, LF_txt in (("bca", "A = 0·1 + 1·2 + 2·0 = 2", "LF = [1, 2, 0]"),
                             ("cab", "A = 0·2 + 1·0 + 2·1 = 2", "LF = [2, 0, 1]")):
        LF = lf_of(L)
        assert f"LF = [{', '.join(map(str, LF))}]" == LF_txt
        A = sum(i * code[ch] for i, ch in enumerate(L))
        assert A == 2 and A_txt.endswith(f"= {A}")
        cyc = [0]
        while len(cyc) < 3:
            cyc.append(LF[cyc[-1]])
        assert LF[cyc[-1]] == 0
        W, H = 340, 290
        mid = f"fx8-{L}"
        o = [svg_open(W, H, f"L = {L}: first moment A = 2; LF = {LF}, a single cycle {' → '.join(map(str, cyc))} → 0."),
             marker(mid), text(170, 20, f"L = {L}", "fx-rule")]
        o += ribbons(L, 44, 150, 44, 32, 30, label_shift=False, cls_of=lambda c: {"a": "a", "b": "b", "c": "c"}[c], rows=False)
        for i, ch in enumerate(L):
            o.append(text(18, 44 + i * 32 + 20, f"{i}·{code[ch]}", "fx-row", "middle"))
        o += ring(cyc, L, 262, 92, 46, mid, r_node=14, steps=False, row_labels=True,
                  cls_of=lambda c: {"a": "a", "b": "b", "c": "c"}[c])
        o += [text(170, 186, A_txt, "fx-pair"), text(170, 210, LF_txt, "fx-pair"),
              text(170, 234, "the ring turns " + ("forwards" if cyc == [0, 1, 2] else "backwards"), "bf-tm"),
              "</svg>"]
        panels.append("".join(o))
    write("fig8.svg", '<div class="fx-panels">' + "".join(panels) + "</div>")


# ---------------------------------------------------------------- figure 9
def fig9():
    L = "Paaabxbccddeeff$MZ"
    n = len(L)
    LF = lf_of(L)
    cyc = [0]
    while LF[cyc[-1]] != 0:
        cyc.append(LF[cyc[-1]])
    assert len(cyc) == n, "LF must be one cycle of 18 rows"
    def path(r, k):
        p = [r]
        while len(p) < k:
            p.append(LF[p[-1]])
        return p
    p1, p3 = path(1, 6), path(3, 6)
    assert p1 == [1, 4, 7, 9, 11, 13] and p3 == [3, 6, 8, 10, 12, 14]
    assert "".join(L[i] for i in p1) == "abcdef" == "".join(L[i] for i in p3)
    assert L[4] == L[6] == "b" and L[5] != "b" and L[3] != "b" and L[7] != "b"
    W, H = 660, 286
    X0, CW, Y = 30, 34, 112
    o = [svg_open(W, H, "The 18-row column Paaabxbccddeeff$MZ, one LF cycle. Paths from rows 1 and 3 both spell "
                        "abcdef; their second rows, 4 and 6, are single b's."), marker("fx9-head")]
    for i, ch in enumerate(L):
        x = X0 + i * CW
        cls = "fx-cell9"
        if i in (1, 3):
            cls += " fx-start"
        if i in (4, 6):
            cls += " fx-w1cell"
        o.append(f'<rect class="{cls}" x="{x}" y="{Y}" width="{CW - 3}" height="34" rx="3"/>')
        o.append(text(x + (CW - 3) / 2, Y + 22, ch, "fx-letter"))
        o.append(text(x + (CW - 3) / 2, Y + 50, str(i), "fx-row"))
    for a, b in runs_of(L):
        if b > a:
            o.append(f'<path class="fx-runline" d="M{X0 + a * CW + 2},{Y + 40} L{X0 + b * CW + CW - 5},{Y + 40}"/>')
    cxx = lambda i: X0 + i * CW + (CW - 3) / 2
    for pth, up, cls in ((p1, True, "fx-path-a"), (p3, False, "fx-path-b")):
        for u, v in zip(pth, pth[1:]):
            x0, x1 = cxx(u), cxx(v)
            hgt = 16 + 6 * (v - u)
            if up:
                d = f"M{f(x0)},{Y - 4} C{f(x0)},{Y - hgt} {f(x1)},{Y - hgt} {f(x1)},{Y - 6}"
            else:
                d = f"M{f(x0)},{Y + 58} C{f(x0)},{Y + 54 + hgt} {f(x1)},{Y + 54 + hgt} {f(x1)},{Y + 60}"
            o.append(f'<path class="{cls}" d="{d}" marker-end="url(#fx9-head)"/>')
    o += [text(X0 + 9 * CW, 26, "row 1 spells a b c d e f along 1 → 4 → 7 → 9 → 11 → 13", "fx-path-a-t"),
          text(X0 + 9 * CW, 230, "row 3 spells a b c d e f along 3 → 6 → 8 → 10 → 12 → 14", "fx-path-b-t"),
          text(X0 + 9 * CW, 262, "rows 4 and 6 are width-1 runs of b; their own six-letter contexts,", "bf-tm"),
          text(X0 + 9 * CW, 278, "bcdef$ and bcdefM, are unique", "bf-tm"), "</svg>"]
    write("fig9.svg", '<div class="fx-switch"><div class="fx-wide">' + "".join(o) + '</div><div class="fx-narrow">'
          + fig9_narrow(L, p1, p3) + "</div></div>")


# ---------------------------------------------------------------- figure 10
def fig10():
    W, H = 360, 430
    o = [svg_open(W, H, "Two pipelines. Today's Open builds its structures, walks the retained cycle and appends "
                        "words. The test receives the word codes already in output order and runs the same append code.",
                  solo=True)]
    xs = [36, 136, 240, 328]
    rows = ((50, "today's Open", ["walk the", "retained cycle"], False, ["the baseline"]),
            (196, "the supplied test", ["word codes in", "output order,", "supplied free"], True,
             ["an upper bound on what any", "producer of that order can win"]))
    for y, label, mid_lines, test, note in rows:
        o.append(text(4, y - (44 if test else 26), label, "bf-ts", "start"))
        o.append(f'<path class="fx-rail" d="M{xs[0]},{y} L{xs[-1]},{y}"/>')
        for k, x in enumerate(xs):
            if k == 1 and test:
                o.append(f'<rect class="fx-inset" x="{x - 54}" y="{y - 30}" width="108" height="60" rx="14"/>')
            else:
                o.append(f'<circle class="{"fx-stop-text" if k == 3 else "fx-stop-done"}" cx="{x}" cy="{y}" r="8"/>')
        lab = [["Open's", "construction"], mid_lines, ["same", "append code"], ["text"]]
        for k, (x, ls) in enumerate(zip(xs, lab)):
            inset = k == 1 and test
            for j, line in enumerate(ls):
                yy = y - 10 + j * 14 if inset else y + 26 + j * 14
                o.append(text(x, yy, line, "fx-inset-t" if inset else "bf-tm"))
        for j, line in enumerate(note):
            o.append(text(180, y + (74 if test else 66) + 17 * j, line, "fx-pair"))
    ax0, ax1, y = 30, 330, 376
    o.append(text(180, y - 34, "if even the free order leaves less than 10%", "bf-tm"))
    o.append(text(180, y - 19, "of whole-decoder time to win, this representation is closed", "bf-tm"))
    o.append(f'<path class="fx-axis" d="M{ax0},{y} L{ax1},{y}"/>')
    gx = ax0 + 0.10 * (ax1 - ax0)
    o.append(f'<rect class="fx-gate-zone" x="{ax0}" y="{y - 9}" width="{f(gx - ax0)}" height="18"/>')
    o.append(f'<path class="fx-gate" d="M{f(gx)},{y - 13} L{f(gx)},{y + 13}"/>')
    o.append(text(gx, y + 28, "10%", "fx-pair"))
    o.append(text(ax0, y + 28, "0", "fx-row"))
    o.append(text(ax1, y + 28, "whole-decoder time", "fx-row", "end"))
    o.append("</svg>")
    write("fig10.svg", "".join(o))


def apply():
    """Put each drawing into its figure in the post, last figure first so offsets stay valid."""
    import re

    s = open(POST).read()
    for idx in range(10, 0, -1):
        starts = [m.start() for m in re.finditer(r'<figure class="bf-fig', s)]
        a = starts[idx - 1]
        b = s.index("</figure>", a)
        fig = s[a:b]
        m = re.search(r'<div class="fx-switch">.*?</svg></div></div>|<div class="fx-(?:panels|wide)">.*?</svg></div>'
                      r'|<svg\b.*?</svg>', fig, re.S)
        fig = fig[: m.start()] + DRAWN[f"fig{idx}.svg"] + fig[m.end():]
        frames = DRAWN.get(f"fig{idx}-frames.html")
        if frames:
            t = re.search(r'<template class="bf-frame".*</template>', fig, re.S)
            fig = fig[: t.start()] + frames + fig[t.end():]
        s = s[:a] + fig + s[b:]
    open(POST, "w").write(s)


for fn in (fig1, fig2, fig3, fig4, fig5, fig6, fig7, fig8, fig9, fig10):
    fn()
apply()
print("ten figures drawn and placed; every value asserted")
