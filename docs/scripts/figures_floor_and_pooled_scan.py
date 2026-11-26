"""Figures for "The decoder's true floor and the limits of a pooled scan" (blog, 26 November 2026).

    python3 docs/scripts/figures_floor_and_pooled_scan.py NAME     # print one figure's <svg>
    python3 docs/scripts/figures_floor_and_pooled_scan.py --inject POST.md

`--inject` replaces the line after each `<!-- fig:NAME -->` marker in POST.md with that figure's
<svg> (one line, so Markdown leaves it alone).

The figures follow the dataviz method: colour by role from the blog.css tokens (--bf-vz-* and
--bf-*), validated in light and dark schemes, with solid hairline rules, bars at most 24 px with
4 px rounded data ends, and <title> hover tooltips on every data mark.

Sources:
- taring, reservoir: docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/
  (README.md, report.txt, zero-copy-floor.tsv; origin/evidence cc8ba137), first matrix
  docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md (7dd7b070),
  and traced Reader::copy_out / A9 from docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md
  (re-derived from pooled-scan bundle c71c4a56).
- killtest, redistribution: docs/journal/2026-11-24-algorithm-level/pooled-scan/results/
  (README.md, report.txt, pooled-scan.tsv, phases.tsv; origin/evidence c71c4a56).
"""

import math
import re
import sys

W = 720


def esc(s):
    """Escape XML special characters for SVG text and attributes."""
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def svg(h, body, aria, w=W):
    """Wrap SVG elements in a single-line responsive root element."""
    return (
        f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{esc(aria)}">'
        + "".join(body)
        + "</svg>"
    )


def text(o, x, y, s, size=11.0, cls="bf-tm", anchor="start", fill=None, weight=None, italic=False):
    """Append an SVG text element using blog.css typography classes."""
    st = f"font-size:{size}px"
    if fill:
        st += f";fill:{fill}"
    if weight:
        st += f";font-weight:{weight}"
    if italic:
        st += ";font-style:italic"
    o.append(
        f'<text class="{cls}" x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" style="{st}">{esc(s)}</text>'
    )


def line(o, x1, y1, x2, y2, stroke="var(--bf-vz-grid)", sw=1.0, dash=None):
    """Append an SVG line element."""
    st = f"stroke:{stroke};stroke-width:{sw}"
    if dash:
        st += f";stroke-dasharray:{dash}"
    o.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" style="{st}"/>')


def rect(o, x, y, w, h, fill, rx=0, stroke=None, sw=1.2, title=None, dash=None, opacity=None):
    """Append an SVG rectangle with optional border, dash pattern, opacity, and hover title."""
    st = f"fill:{fill}"
    if opacity is not None:
        st += f";fill-opacity:{opacity}"
    if stroke:
        st += f";stroke:{stroke};stroke-width:{sw}"
    if dash:
        st += f";stroke-dasharray:{dash}"
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(
        f'<rect x="{x:.1f}" y="{y:.1f}" width="{max(w, 0):.1f}" height="{max(h, 0):.1f}" '
        f'rx="{rx}" style="{st}">{t}</rect>'
    )


def bar_h(o, x0, y, length, h, fill, title=None, stroke=None, sw=1.0):
    """Draw a horizontal bar from x0 with a square baseline and 4 px rounded data end."""
    r = min(4.0, h / 2.0, abs(length))
    t = f"<title>{esc(title)}</title>" if title else ""
    st = f"fill:{fill}"
    if stroke:
        st += f";stroke:{stroke};stroke-width:{sw}"
    if length <= 0.5:
        o.append(
            f'<rect x="{x0:.1f}" y="{y:.1f}" width="2.0" height="{h:.1f}" style="{st}">{t}</rect>'
        )
        return
    x1 = x0 + length
    d = (
        f"M{x0:.1f},{y:.1f} L{x1 - r:.1f},{y:.1f} "
        f"A{r:.1f},{r:.1f} 0 0 1 {x1:.1f},{y + r:.1f} "
        f"L{x1:.1f},{y + h - r:.1f} "
        f"A{r:.1f},{r:.1f} 0 0 1 {x1 - r:.1f},{y + h:.1f} "
        f"L{x0:.1f},{y + h:.1f} Z"
    )
    o.append(f'<path d="{d}" style="{st}">{t}</path>')


def circle(o, cx, cy, r, fill, stroke=None, sw=1.2, title=None):
    """Append an SVG circle with optional stroke and hover title."""
    st = f"fill:{fill}"
    if stroke:
        st += f";stroke:{stroke};stroke-width:{sw}"
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}" style="{st}">{t}</circle>')


def diamond(o, cx, cy, r, fill, stroke, sw=1.4, title=None):
    """Append a diamond marker centred at (cx, cy)."""
    pts = f"{cx:.1f},{cy - r:.1f} {cx + r:.1f},{cy:.1f} {cx:.1f},{cy + r:.1f} {cx - r:.1f},{cy:.1f}"
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(
        f'<polygon points="{pts}" style="fill:{fill};stroke:{stroke};stroke-width:{sw}">{t}</polygon>'
    )


def arrow(o, x1, y1, x2, y2, colour="var(--bf-vz-axis)", sw=1.4, dash=""):
    """Append a straight line with a filled triangular arrowhead."""
    st = f"stroke:{colour};stroke-width:{sw}"
    if dash:
        st += f";stroke-dasharray:{dash}"
    o.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" style="{st}"/>')
    a = math.atan2(y2 - y1, x2 - x1)
    hl, hw = 6.5, 3.5
    pts = [
        (x2, y2),
        (x2 - hl * math.cos(a) + hw * math.sin(a), y2 - hl * math.sin(a) - hw * math.cos(a)),
        (x2 - hl * math.cos(a) - hw * math.sin(a), y2 - hl * math.sin(a) + hw * math.cos(a)),
    ]
    o.append(
        f'<polygon points="{" ".join(f"{u:.1f},{v:.1f}" for u, v in pts)}" style="fill:{colour}"/>'
    )


def taring():
    """Two-panel chart: taring the 8-worker floor (left) + each stage alone real at 8W (right)."""
    o = []

    # ---- Panel A (left): Taring the scale + 8->10 trend inset ----
    text(o, 14, 20, "A. Taring the 8-worker all-ideal floor", 12, "bf-t-strong")
    text(o, 14, 35, "Removing 4.9 GB of oracle copies and word spelling (ms)", 10, "bf-tm")

    ax0, ax1 = 132.0, 332.0
    max_a = 150.0

    def xa(v):
        return ax0 + (v / max_a) * (ax1 - ax0)

    # Top half: 4 horizontal bars decomposing 112 -> 42.4 -> 37.2 + 5.2 ms
    for tick in (0, 50, 100, 150):
        tx = xa(tick)
        line(o, tx, 48, tx, 158, "var(--bf-vz-axis)" if tick == 0 else "var(--bf-vz-grid)", 1.0)
        text(o, tx, 171, f"{tick}", 9.5, "bf-tm", "middle")
    text(o, ax1 + 14, 171, "ms", 9.5, "bf-tm", "start")

    rows_a = [
        (
            54,
            "1st matrix all-ideal",
            "copying oracles + spelling",
            112.0,
            "var(--bf-vz-2)",
            "112 ms",
            "First matrix all-ideal at 8 workers: 112 ms (includes ~4.9 GB oracle copying + word spelling)",
        ),
        (
            82,
            "Zero-copy (SECMRD)",
            "Residual A, spelling kept",
            42.4,
            "var(--bf-vz-1)",
            "42.4 ms",
            "Residual A (SECMRD, zero-copy) at 8 workers: 42.4 [40.3, 44.4] ms (15.2 GB/s)",
        ),
        (
            110,
            "Word spelling (A-B)",
            "E_consume (cold upper bound)",
            37.2,
            "var(--bf-vz-4)",
            "37.2 ms",
            "Emission word spelling (E_consume = A - B) at 8 workers: 37.2 [35.2, 39.1] ms (88% of Residual A; cold-store upper bound)",
        ),
        (
            138,
            "Plumbing (SFCMRD)",
            "Residual B, all free",
            5.2,
            "var(--bf-vz-3)",
            "5.2 ms (2.7%)",
            "Residual B (SFCMRD, plumbing alone) at 8 workers: 5.2 [5.0, 5.5] ms (2.7% of shipped 194.6 ms)",
        ),
    ]
    for ry, lbl, sub, val, col, vtxt, tip in rows_a:
        text(o, ax0 - 7, ry + 8, lbl, 9.8, "bf-t-strong", "end")
        text(o, ax0 - 7, ry + 19, sub, 8.6, "bf-tm", "end")
        bw = xa(val) - ax0
        bar_h(o, ax0, ry + 1, bw, 16, col, title=tip)
        if bw > 56:
            ink = (
                "var(--bf-vz-2-ink)"
                if col == "var(--bf-vz-2)"
                else "var(--bf-vz-1-ink)"
                if col == "var(--bf-vz-1)"
                else "var(--bf-vz-4-ink)"
            )
            text(o, ax0 + bw - 6, ry + 13, vtxt, 9.5, "bf-t-strong", "end", fill=ink)
        else:
            text(o, ax0 + bw + 5, ry + 13, vtxt, 9.5, "bf-t-strong", "start")

    # Bottom half of Panel A: 8 -> 9 -> 10 workers curve (DRAM bandwidth hypothesis refuted)
    rect(o, 14, 188, 336, 168, "var(--bf-surface)", rx=7, stroke="var(--bf-frame)", sw=1.1)
    text(o, 26, 206, "8 to 10 workers: the rising floor vanished", 10.5, "bf-t-strong")
    text(o, 26, 220, "All-ideal wall (ms); bars show 95% intervals", 9.2, "bf-tm")

    ix0, ix1 = 56.0, 232.0
    iy0, iy1 = 328.0, 236.0
    imax = 150.0

    def ix(k):
        return ix0 + ((k - 8.0) / 2.0) * (ix1 - ix0)

    def iy(v):
        return iy0 - (v / imax) * (iy0 - iy1)

    for tv in (0, 50, 100, 150):
        ty = iy(tv)
        line(o, ix0, ty, ix1, ty, "var(--bf-vz-axis)" if tv == 0 else "var(--bf-vz-grid)", 1.0)
        text(o, ix0 - 6, ty + 3, f"{tv}", 9.0, "bf-tm", "end")
    for tk in (8, 9, 10):
        tx = ix(tk)
        line(o, tx, iy1, tx, iy0, "var(--bf-vz-grid)", 0.8)
        text(o, tx, iy0 + 14, f"{tk}W", 9.5, "bf-tm", "middle")

    # First matrix all-ideal: 8W=112, 10W=142 (dashed amber)
    line(o, ix(8), iy(112.0), ix(10), iy(142.0), "var(--bf-vz-2)", 2.0, dash="5 3")
    for tk, tv in ((8, 112.0), (10, 142.0)):
        circle(
            o,
            ix(tk),
            iy(tv),
            3.5,
            "var(--bf-vz-2)",
            title=f"First matrix all-ideal at {tk} workers: {tv:.0f} ms (copying artifact)",
        )
    text(o, ix(10) + 7, iy(142.0) + 3, "1st matrix", 9.2, "bf-t-strong", fill="var(--bf-amber)")
    text(o, ix(10) + 7, iy(142.0) + 14, "112->142 ms", 8.8, "bf-tm")

    # Zero-copy SECMRD: 8W=42.4 [40.3,44.4], 9W=42.9 [40.4,45.3], 10W=40.5 [37.6,43.5]
    sec_pts = [(8, 42.4, 40.3, 44.4), (9, 42.9, 40.4, 45.3), (10, 40.5, 37.6, 43.5)]
    pts_str = " ".join(f"{ix(k):.1f},{iy(v):.1f}" for k, v, _, _ in sec_pts)
    o.append(
        f'<polyline points="{pts_str}" style="fill:none;stroke:var(--bf-vz-1);stroke-width:2.0"/>'
    )
    for k, v, lo, hi in sec_pts:
        line(o, ix(k), iy(lo), ix(k), iy(hi), "var(--bf-vz-1)", 1.5)
        circle(
            o,
            ix(k),
            iy(v),
            3.5,
            "var(--bf-vz-1)",
            title=f"Zero-copy SECMRD at {k} workers: {v:.1f} [{lo:.1f}, {hi:.1f}] ms",
        )
    text(o, ix(10) + 7, iy(40.5) - 13, "Zero-copy (A)", 9.2, "bf-t-strong", fill="var(--bf-blue)")
    text(o, ix(10) + 7, iy(40.5) - 2, "42.4->40.5 ms", 8.8, "bf-tm")
    text(o, ix(10) + 7, iy(40.5) + 9, "15-17 GB/s", 8.5, "bf-tm")

    # Zero-copy SFCMRD: 8W=5.2, 9W=5.3, 10W=5.4
    sfc_pts = [(8, 5.2, 5.0, 5.5), (9, 5.3, 5.1, 5.5), (10, 5.4, 5.3, 5.5)]
    pts_str2 = " ".join(f"{ix(k):.1f},{iy(v):.1f}" for k, v, _, _ in sfc_pts)
    o.append(
        f'<polyline points="{pts_str2}" style="fill:none;stroke:var(--bf-vz-3);stroke-width:1.8"/>'
    )
    for k, v, lo, hi in sfc_pts:
        circle(
            o,
            ix(k),
            iy(v),
            3.0,
            "var(--bf-vz-3)",
            title=f"Zero-copy SFCMRD (plumbing) at {k} workers: {v:.1f} [{lo:.1f}, {hi:.1f}] ms",
        )
    text(
        o,
        ix(10) + 7,
        iy(5.4) + 5,
        "Plumbing: 5.2-5.4",
        8.8,
        "bf-t-strong",
        fill="var(--bf-green-ink)",
    )

    # Divider between Panel A and Panel B
    line(o, 362, 12, 362, 368, "var(--bf-frame)", 1.0)

    # ---- Panel B (right): Each stage alone real at 8 workers ----
    text(o, 376, 20, "B. One stage real, all else free (8 workers)", 12, "bf-t-strong")
    text(o, 376, 35, "Median wall time (ms) against shipped 194.6 ms", 10, "bf-tm")

    bx0, bx1 = 492.0, 694.0
    max_b = 215.0

    def xb(v):
        return bx0 + (v / max_b) * (bx1 - bx0)

    for tick in (0, 50, 100, 150, 200):
        tx = xb(tick)
        line(o, tx, 50, tx, 308, "var(--bf-vz-axis)" if tick == 0 else "var(--bf-vz-grid)", 1.0)
        text(o, tx, 323, f"{tick}", 9.5, "bf-tm", "middle")
    text(o, bx1 + 2, 323, "ms", 9.5, "bf-tm", "start")

    # Shipped wall reference line at 194.6 ms
    rx_ship = xb(194.6)
    line(o, rx_ship, 48, rx_ship, 308, "var(--bf-ink)", 1.3, dash="4 3")
    text(o, rx_ship - 4, 58, "shipped 194.6 ms", 9.0, "bf-t-strong", "end")

    stages_b = [
        (
            70,
            "Scan (FCMRD)",
            "serial; 92% of wall",
            179.1,
            "var(--bf-vz-2)",
            "179.1 ms",
            "Scan alone real (FCMRD) at 8 workers: median 179.1 ms (92% of shipped 194.6 ms; caps worker-only gains at <=8%)",
        ),
        (
            104,
            "Emission (SCMRD)*",
            "walk + spelling; cold C",
            98.1,
            "var(--bf-vz-1)",
            "98.1 ms*",
            "Emission alone real (SCMRD, walk + spelling) at 8 workers: median 98.1 ms (overstated because walk A7 reads borrowed C store cold: 98.9 vs 72.4 ms/w)",
        ),
        (
            138,
            "Construction (SFMRD)",
            "parallel worker stage",
            83.6,
            "var(--bf-vz-1)",
            "83.6 ms",
            "Construction alone real (SFMRD) at 8 workers: median 83.6 ms (A7 is 0.0 ms here)",
        ),
        (
            172,
            "Delivery (D)†",
            "traced vs cold SFCMR",
            34.0,
            "var(--bf-vz-4)",
            "34.0 ms† (52.3 cold)",
            "Delivery real traced copy_out in none at 8 workers: 34.0 ms (dashed outline: 52.3 ms cold-store SFCMR wall artifact)",
        ),
        (
            206,
            "MTF / RLE2 (SFCRD)",
            "parallel worker stage",
            20.1,
            "var(--bf-steel)",
            "20.1 ms",
            "MTF/RLE2 alone real (SFCRD) at 8 workers: median 20.1 ms",
        ),
        (
            240,
            "RLE1 / CRC (wall diff)",
            "104 ms A9 worker CPU",
            14.2,
            "var(--bf-steel)",
            "14.2 ms diff",
            "RLE1/CRC at 8 workers: 14.2 ms wall difference (SECMD 55.0 minus SECMRD 40.8 ms), hiding 104.0 ms traced A9 worker CPU (7% of worker CPU)",
        ),
        (
            274,
            "Nothing (SFCMRD)",
            "plumbing alone (2.7%)",
            5.3,
            "var(--bf-vz-3)",
            "5.3 ms",
            "All stages free (SFCMRD) at 8 workers: median 5.3 ms (mean 5.2 [5.0, 5.5] ms, 2.7% of shipped)",
        ),
    ]
    for ry, lbl, sub, val, col, vtxt, tip in stages_b:
        text(o, bx0 - 8, ry + 8, lbl, 9.8, "bf-t-strong", "end")
        text(o, bx0 - 8, ry + 19, sub, 8.6, "bf-tm", "end")
        bw = xb(val) - bx0
        if lbl.startswith("Delivery"):
            # Solid bar for real traced copy_out (34.0 ms) + dashed ghost outline to cold SFCMR (52.3 ms)
            bar_h(o, bx0, ry + 1, bw, 18, col, title=tip)
            cold_w = xb(52.3) - xb(34.0)
            rect(
                o,
                xb(34.0),
                ry + 1,
                cold_w,
                18,
                "var(--bf-vz-4)",
                rx=2,
                stroke="var(--bf-vz-4)",
                sw=1.1,
                dash="3 2",
                opacity=0.18,
                title="Cold-store SFCMR wall at 8W: 52.3 ms (artifact of reading 209.7 MiB oracle store from DRAM)",
            )
            text(o, xb(52.3) + 5, ry + 14, vtxt, 9.2, "bf-t-strong", "start")
        else:
            bar_h(o, bx0, ry + 1, bw, 18, col, title=tip)
            if bw > 58:
                ink = "var(--bf-vz-2-ink)" if col == "var(--bf-vz-2)" else "var(--bf-vz-1-ink)"
                text(o, bx0 + bw - 6, ry + 14, vtxt, 9.6, "bf-t-strong", "end", fill=ink)
            else:
                text(o, bx0 + bw + 5, ry + 14, vtxt, 9.6, "bf-t-strong", "start")

    text(
        o,
        376,
        342,
        "*Walk (A7) reads borrowed C cold (98.9 vs 72.4 ms/w), overstating E.",
        8.6,
        "bf-tm",
    )
    text(
        o,
        376,
        356,
        "†Traced real copy_out is 34.0 ms at 8W; 52.3 ms is cold-store SFCMR.",
        8.6,
        "bf-tm",
    )

    aria = (
        "Two-panel empirical chart from the zero-copy floor run at 8 workers. "
        "Panel A shows the all-ideal wall dropping from 112 ms in the first matrix to 42.4 ms with zero-copy "
        "oracles (Residual A), splitting into 37.2 ms of emission word spelling (cold-store upper bound) and "
        "5.2 ms (2.7 percent) of pipeline plumbing (Residual B). An inset shows the 8 to 10 worker trend: the "
        "first matrix rose from 112 to 142 ms, whereas zero-copy Residual A stays flat at 42.4, 42.9, and "
        "40.5 ms at 15 to 17 GB/s. Panel B shows each single stage real at 8 workers against the 194.6 ms "
        "shipped wall: scan 179.1 ms (92 percent), emission 98.1 ms (cold-store caveat on borrowed C), "
        "construction 83.6 ms, delivery 34.0 ms traced copy_out (with 52.3 ms cold-store SFCMR ghost), "
        "MTF/RLE2 20.1 ms, RLE1/CRC 14.2 ms wall difference hiding 104.0 ms traced A9 CPU, and nothing 5.3 ms."
    )
    return svg(374, o, aria)


def reservoir():
    """Multi-series scaling chart (1..10W): word spelling drains while traced delivery copy_out rises under L3 pressure."""
    o = []
    text(
        o,
        16,
        20,
        "Word spelling drains with workers while delivery copy-out rises under L3 pressure",
        11.8,
        "bf-t-strong",
    )
    text(
        o,
        16,
        36,
        "Floor wall times (ms, 95% intervals) beside traced reader copy_out (pooled-scan medians) on wikidata x4",
        9.6,
        "bf-tm",
    )

    # Discrete x positions with a clean visual gap between 1W and 6W
    xs = {1: 86.0, 6: 212.0, 7: 294.0, 8: 376.0, 9: 458.0, 10: 540.0}
    y0, y1 = 286.0, 54.0
    ymax = 320.0

    def yv(v):
        return y0 - (v / ymax) * (y0 - y1)

    # Horizontal gridlines
    for tv in (0, 50, 100, 150, 200, 250, 300):
        ty = yv(tv)
        line(o, 56, ty, 554, ty, "var(--bf-vz-axis)" if tv == 0 else "var(--bf-vz-grid)", 1.0)
        text(o, 48, ty + 3.5, f"{tv}", 9.8, "bf-tm", "end")
    text(o, 16, 52, "ms", 9.8, "bf-tm", "start")

    # Axis break indicator between 1W and 6W
    bx = 149.0
    line(o, bx - 4, y0 + 5, bx + 1, y0 - 5, "var(--bf-vz-axis)", 1.2)
    line(o, bx + 1, y0 + 5, bx + 6, y0 - 5, "var(--bf-vz-axis)", 1.2)
    for k, xk in xs.items():
        line(o, xk, y1, xk, y0, "var(--bf-vz-grid)", 0.8)
        text(o, xk, y0 + 16, f"{k}W", 10.2, "bf-t-strong" if k in (1, 8) else "bf-tm", "middle")
    text(o, 310, y0 + 32, "Worker threads (break between 1 and 6 workers)", 9.8, "bf-tm", "middle")

    # Floor-run wall times / wall differences (cc8ba137)
    e_consume = [
        (1, 298.2, 297.0, 299.4),
        (6, 48.7, 45.9, 51.4),
        (7, 41.5, 40.0, 43.0),
        (8, 37.2, 35.2, 39.1),
        (9, 37.6, 35.1, 40.1),
        (10, 35.2, 32.3, 38.0),
    ]
    scan_only = [
        (1, 169.7),
        (6, 177.0),
        (7, 177.8),
        (8, 179.1),
        (9, 177.6),
        (10, 178.7),
    ]
    plumbing = [
        (1, 5.7, 5.5, 5.8),
        (6, 5.3, 5.3, 5.4),
        (7, 5.2, 4.9, 5.5),
        (8, 5.2, 5.0, 5.5),
        (9, 5.3, 5.1, 5.5),
        (10, 5.4, 5.3, 5.5),
    ]
    # Real Reader::copy_out from pooled-scan traces (n=3 medians, activity time, not wall diff)
    delivery_none = [
        (1, 29.1),
        (6, 27.6),
        (7, 30.6),
        (8, 34.0),
        (9, 43.2),
        (10, 43.7),
    ]
    delivery_sc = [
        (6, 29.0),
        (7, 32.5),
        (8, 39.5),
        (9, 54.9),
        (10, 77.9),
    ]

    # Translucent "draining reservoir" fill under E_consume
    poly_pts = [(xs[1], y0)] + [(xs[k], yv(v)) for k, v, _, _ in e_consume] + [(xs[10], y0)]
    o.append(
        f'<polygon points="{" ".join(f"{u:.1f},{v:.1f}" for u, v in poly_pts)}" '
        f'style="fill:var(--bf-vz-1);fill-opacity:0.12"/>'
    )

    # Series 1: Scan alone (FCMRD) - amber dashed reef
    scan_str = " ".join(f"{xs[k]:.1f},{yv(v):.1f}" for k, v in scan_only)
    o.append(
        f'<polyline points="{scan_str}" style="fill:none;stroke:var(--bf-vz-2);stroke-width:2.2;stroke-dasharray:6 3"/>'
    )
    for k, v in scan_only:
        circle(
            o,
            xs[k],
            yv(v),
            3.8,
            "var(--bf-vz-2)",
            title=f"Scan alone real (FCMRD) at {k}W: median {v:.1f} ms",
        )
    text(o, xs[1] + 8, yv(169.7) - 9, "169.7 ms", 9.4, "bf-t-strong", fill="var(--bf-amber)")
    text(o, xs[8], yv(179.1) - 10, "179.1 ms", 9.4, "bf-t-strong", "middle", fill="var(--bf-amber)")

    # Series 2: E_consume - draining blue curve
    ec_str = " ".join(f"{xs[k]:.1f},{yv(v):.1f}" for k, v, _, _ in e_consume)
    o.append(
        f'<polyline points="{ec_str}" style="fill:none;stroke:var(--bf-vz-1);stroke-width:2.4"/>'
    )
    for k, v, lo, hi in e_consume:
        line(o, xs[k], yv(lo), xs[k], yv(hi), "var(--bf-vz-1)", 1.5)
        circle(
            o,
            xs[k],
            yv(v),
            4.0,
            "var(--bf-vz-1)",
            title=f"Emission word spelling (E_consume = A - B) at {k}W: {v:.1f} [{lo:.1f}, {hi:.1f}] ms (cold-store upper bound)",
        )
    text(
        o,
        xs[1] + 9,
        yv(298.2) + 4,
        "298.2 ms (cold upper bound; ~230-245 in situ)",
        9.4,
        "bf-t-strong",
        fill="var(--bf-blue)",
    )
    text(o, xs[8], yv(37.2) + 15, "37.2 ms", 9.0, "bf-t-strong", "middle", fill="var(--bf-blue)")

    # Series 3a: Real delivery copy_out in SC (fast workers) - rising purple dotted curve
    del_sc_str = " ".join(f"{xs[k]:.1f},{yv(v):.1f}" for k, v in delivery_sc)
    o.append(
        f'<polyline points="{del_sc_str}" style="fill:none;stroke:var(--bf-vz-4);stroke-width:2.0;stroke-dasharray:2 2"/>'
    )
    for k, v in delivery_sc:
        diamond(
            o,
            xs[k],
            yv(v),
            3.8,
            "var(--bf-vz-4)",
            "var(--bf-vz-4)",
            1.4,
            title=f"Traced delivery copy_out in SC (fast workers) at {k}W: median {v:.1f} ms (reader activity time)",
        )

    # Series 3b: Real delivery copy_out in none (shipped) - rising purple dashed curve
    del_none_str = " ".join(f"{xs[k]:.1f},{yv(v):.1f}" for k, v in delivery_none)
    o.append(
        f'<polyline points="{del_none_str}" style="fill:none;stroke:var(--bf-vz-4);stroke-width:2.2;stroke-dasharray:4 2"/>'
    )
    for k, v in delivery_none:
        diamond(
            o,
            xs[k],
            yv(v),
            4.0,
            "var(--bf-surface)",
            "var(--bf-vz-4)",
            1.6,
            title=f"Traced delivery copy_out in none (shipped) at {k}W: median {v:.1f} ms (reader activity time)",
        )
    text(o, xs[1] + 8, yv(29.1) - 8, "29.1 ms", 9.0, "bf-t-strong", fill="var(--bf-ultra)")

    # Series 4: Plumbing (SFCMRD) - flat green riverbed
    pl_str = " ".join(f"{xs[k]:.1f},{yv(v):.1f}" for k, v, _, _ in plumbing)
    o.append(
        f'<polyline points="{pl_str}" style="fill:none;stroke:var(--bf-vz-3);stroke-width:2.0"/>'
    )
    for k, v, lo, hi in plumbing:
        circle(
            o,
            xs[k],
            yv(v),
            3.2,
            "var(--bf-vz-3)",
            title=f"Pipeline plumbing (SFCMRD) at {k}W: {v:.1f} [{lo:.1f}, {hi:.1f}] ms",
        )

    # Right-hand direct labels with thin leader ticks where needed (zero overlap)
    lx = 558.0
    text(o, lx, 153, "Serial scan (FCMRD)", 9.8, "bf-t-strong", fill="var(--bf-amber)")
    text(o, lx, 165, "170-179 ms (wall)", 9.0, "bf-tm")

    line(o, 545, yv(77.9), 555, 214, "var(--bf-vz-4)", 0.9)
    text(o, lx, 211, "Delivery copy_out (SC)", 9.6, "bf-t-strong", fill="var(--bf-ultra)")
    text(o, lx, 222, "29 -> 78 ms (traced)", 8.8, "bf-tm")

    line(o, 545, yv(43.7), 555, 241, "var(--bf-vz-4)", 0.9)
    text(o, lx, 239, "Delivery copy_out (none)", 9.6, "bf-t-strong", fill="var(--bf-ultra)")
    text(o, lx, 250, "29 -> 44 ms (traced)", 8.8, "bf-tm")

    line(o, 545, yv(35.2), 555, 268, "var(--bf-vz-1)", 0.9)
    text(o, lx, 267, "Word spelling (E_consume)", 9.6, "bf-t-strong", fill="var(--bf-blue)")
    text(o, lx, 278, "298 -> 35 ms (wall diff)", 8.8, "bf-tm")

    line(o, 545, yv(5.4), 555, 291, "var(--bf-vz-3)", 0.9)
    text(o, lx, 294, "Plumbing: 5.2-5.7 ms", 9.4, "bf-t-strong", fill="var(--bf-green-ink)")

    # Callout highlighting L3-eviction rise at 8-10W
    text(o, 258, 198, "At 8-10W, 20 MiB L3 eviction pushes", 9.1, "bf-tm", "start")
    text(o, 258, 210, "real delivery copy_out above spelling", 9.1, "bf-tm", "start")
    arrow(o, 434, 206, xs[10] - 6, yv(77.9) - 2, "var(--bf-vz-axis)", 1.1)

    aria = (
        "Line chart across 1, 6, 7, 8, 9, and 10 workers showing emission word spelling draining as workers "
        "increase, from 298.2 ms at 1 worker (cold-store upper bound, roughly 230 to 245 ms in situ) down to "
        "48.7, 41.5, 37.2, 37.6, and 35.2 ms, alongside two flat floor baselines (scan alone at 169.7 to "
        "179.1 ms and pipeline plumbing at 5.2 to 5.7 ms) and traced delivery copy_out rising under 20 MiB "
        "L3 eviction: in none from 29.1 ms at 1 worker and 34.0 ms at 8 to 43.7 ms at 10, and with fast "
        "workers in SC from 29.0 ms at 6 to 77.9 ms at 10."
    )
    return svg(334, o, aria)


def killtest():
    """Two-panel interval strip: K1 paired-oracle capture (PC/C) and K2 standalone ratio (P/none)."""
    o = []

    # ---- Panel A (left): K1 paired-oracle PC/C against sqrt(SC/C) bar and SC/C ideal ----
    text(o, 14, 20, "A. Paired-oracle test behind ideal C (K1)", 12, "bf-t-strong")
    text(o, 14, 35, "Ratio PC/C (95% CI) vs K1 bar sqrt(SC/C) and ideal SC/C", 9.8, "bf-tm")

    ax0, ax1 = 82.0, 344.0
    amin, amax = 0.70, 1.12

    def xa(v):
        return ax0 + ((v - amin) / (amax - amin)) * (ax1 - ax0)

    # Reference ticks
    for tv in (0.75, 0.85, 0.95, 1.00, 1.05):
        tx = xa(tv)
        is_one = abs(tv - 1.0) < 1e-6
        line(
            o,
            tx,
            52,
            tx,
            244,
            "var(--bf-ink)" if is_one else "var(--bf-vz-grid)",
            1.2 if is_one else 1.0,
            dash="4 3" if is_one else None,
        )
        text(o, tx, 258, f"{tv:.2f}", 9.2, "bf-t-strong" if is_one else "bf-tm", "middle")

    # Panel B geometry
    line(o, 368, 12, 368, 274, "var(--bf-frame)", 1.0)
    text(o, 382, 20, "B. Standalone wall ratio P/none (K2)", 12, "bf-t-strong")
    text(o, 382, 35, "Ratio P/none (95% CI); K2 kills if LB > 1.03 at 8 or 9", 9.8, "bf-tm")

    bx0, bx1 = 420.0, 696.0
    bmin, bmax = 0.88, 1.15

    def xb(v):
        return bx0 + ((v - bmin) / (bmax - bmin)) * (bx1 - bx0)

    for tv in (0.90, 0.95, 1.00, 1.03, 1.05, 1.10):
        tx = xb(tv)
        if abs(tv - 1.00) < 1e-6:
            line(o, tx, 52, tx, 244, "var(--bf-ink)", 1.2, dash="4 3")
            text(o, tx, 258, "1.00", 9.2, "bf-t-strong", "middle")
        elif abs(tv - 1.03) < 1e-6:
            line(o, tx, 52, tx, 244, "var(--bf-vz-2)", 1.4, dash="3 2")
            text(o, tx + 2, 49, "K2 1.03", 8.8, "bf-t-strong", "start", fill="var(--bf-amber)")
        else:
            line(o, tx, 52, tx, 244, "var(--bf-vz-grid)", 1.0)
            text(o, tx, 258, f"{tv:.2f}", 9.2, "bf-tm", "middle")

    # Data rows from pooled-scan.tsv
    # (k, judged, pc_val, pc_lo, pc_hi, bar, sc_val, cap, pn_val, pn_lo, pn_hi)
    rows = [
        (6, False, 1.0646, 1.0401, 1.0898, 0.9400, 0.8836, -0.51, 1.1045, 1.0942, 1.1148),
        (7, False, 0.9351, 0.9221, 0.9484, 0.8912, 0.7942, 0.29, 1.1004, 1.0871, 1.1140),
        (8, True, 0.8618, 0.8481, 0.8758, 0.8670, 0.7516, 0.52, 1.0519, 1.0233, 1.0812),
        (9, True, 0.8685, 0.8328, 0.9058, 0.8636, 0.7459, 0.48, 0.9999, 0.9832, 1.0170),
        (10, False, 0.8484, 0.8249, 0.8725, 0.8772, 0.7694, 0.63, 0.9357, 0.9086, 0.9637),
    ]

    for idx, (k, judged, pc, pclo, pchi, bar, sc, cap, pn, pnlo, pnhi) in enumerate(rows):
        ry = 72.0 + idx * 38.0
        if judged:
            rect(o, 10, ry - 17, 348, 34, "var(--bf-amber-tint)", rx=4, opacity=0.55)
            rect(o, 378, ry - 17, 332, 34, "var(--bf-amber-tint)", rx=4, opacity=0.55)

        # Row labels (left panel)
        text(o, 74, ry - 1, f"{k} workers", 10.0, "bf-t-strong" if judged else "bf-tm", "end")
        text(
            o,
            74,
            ry + 11,
            "judged" if judged else "reported",
            8.5,
            "bf-t-strong" if judged else "bf-tm",
            "end",
            fill="var(--bf-amber)" if judged else None,
        )

        # Panel A marks: ideal SC/C diamond, K1 bar tick, PC/C CI + point
        diamond(
            o,
            xa(sc),
            ry,
            4.2,
            "var(--bf-surface)",
            "var(--bf-vz-3)",
            1.6,
            title=f"{k}W ideal scan behind C (SC/C): {sc:.4f}",
        )
        # K1 threshold bar sqrt(SC/C)
        line(o, xa(bar), ry - 10, xa(bar), ry + 10, "var(--bf-vz-2)", 2.0)
        rect(
            o,
            xa(bar) - 2,
            ry - 10,
            4,
            20,
            "transparent",
            title=f"{k}W K1 bar sqrt(SC/C): {bar:.4f}",
        )
        # PC/C interval + point
        col_a = "var(--bf-vz-2)" if judged else "var(--bf-vz-1)"
        line(o, xa(pclo), ry, xa(pchi), ry, col_a, 2.4)
        line(o, xa(pclo), ry - 4, xa(pclo), ry + 4, col_a, 1.5)
        line(o, xa(pchi), ry - 4, xa(pchi), ry + 4, col_a, 1.5)
        circle(
            o,
            xa(pc),
            ry,
            4.2,
            col_a,
            title=f"{k}W PC/C: {pc:.4f} [{pclo:.4f}, {pchi:.4f}] vs bar {bar:.4f} (capture {cap:+.2f})",
        )
        if judged:
            miss = pchi - bar
            text(
                o,
                xa(pchi) + 6,
                ry - 2,
                f"cap {cap:.2f}",
                8.8,
                "bf-t-strong",
                "start",
                fill="var(--bf-amber)",
            )
            text(
                o,
                xa(pchi) + 6,
                ry + 9,
                f"UB +{miss:.3f} (K1)",
                8.4,
                "bf-t-strong",
                "start",
                fill="var(--bf-amber)",
            )
        else:
            lbl_x = xa(pchi) + 6 if k != 6 else xa(pclo) - 6
            anch = "start" if k != 6 else "end"
            text(o, lbl_x, ry + 3, f"cap {cap:+.2f}", 8.8, "bf-tm", anch)

        # Panel B marks: row label + P/none CI + point
        text(o, 412, ry + 3, f"{k}W", 9.6, "bf-t-strong" if judged else "bf-tm", "end")
        col_b = "var(--bf-vz-3)" if pn < 0.98 else "var(--bf-vz-2)" if judged else "var(--bf-vz-1)"
        line(o, xb(pnlo), ry, xb(pnhi), ry, col_b, 2.4)
        line(o, xb(pnlo), ry - 4, xb(pnlo), ry + 4, col_b, 1.5)
        line(o, xb(pnhi), ry - 4, xb(pnhi), ry + 4, col_b, 1.5)
        circle(
            o,
            xb(pn),
            ry,
            4.2,
            col_b,
            title=f"{k}W standalone P/none: {pn:.4f} [{pnlo:.4f}, {pnhi:.4f}]",
        )
        if k == 10:
            text(
                o,
                xb(pnhi) + 6,
                ry + 3,
                "0.936 (-6.4%)",
                9.0,
                "bf-t-strong",
                fill="var(--bf-green-ink)",
            )
        elif k == 9:
            text(o, xb(pnhi) + 6, ry + 3, "1.000 (even)", 9.0, "bf-t-strong")
        elif k == 8:
            text(o, xb(pnhi) + 5, ry + 3, "1.052 (K2 ok)", 8.8, "bf-t-strong")
        else:
            text(o, xb(pnlo) - 6, ry + 3, f"{pn:.3f}", 8.8, "bf-tm", "end")

    # Compact legend inside Panel A bottom-left
    diamond(o, 22, 276, 4.0, "var(--bf-surface)", "var(--bf-vz-3)", 1.5)
    text(o, 31, 279, "SC/C (ideal scan)", 8.8, "bf-tm")
    line(o, 128, 271, 128, 281, "var(--bf-vz-2)", 2.0)
    text(o, 134, 279, "K1 bar sqrt(SC/C)", 8.8, "bf-tm")
    circle(o, 232, 276, 3.8, "var(--bf-vz-1)")
    text(o, 240, 279, "PC/C (95% CI)", 8.8, "bf-tm")

    # Passing gates footer banner
    rect(o, 14, 290, 692, 26, "var(--bf-surface)", rx=5, stroke="var(--bf-frame)", sw=1.0)
    text(
        o,
        360,
        307,
        "Passing gates:  K2 standalone LB <= 1.03 (1.023 at 8W)  ·  K3 peak RSS 0.988-1.014 (<= 1.10)  ·  K4 finder 4.82 ms / 9.84 GB/s (<= 10 ms)  ·  0 fallbacks",
        9.1,
        "bf-t-strong",
        "middle",
    )

    aria = (
        "Two-panel interval strip chart of the pooled-scan kill test across 6 to 10 workers, with 8 and 9 "
        "workers highlighted as judged and 6, 7, and 10 marked reported. Panel A shows PC/C with 95 percent "
        "intervals against the ideal SC/C floor and K1 threshold sqrt(SC/C): at 8 workers PC/C is 0.862 "
        "[0.848, 0.876] versus bar 0.867 (capture 0.52, upper bound misses by 0.009, K1); at 9 workers "
        "0.869 [0.833, 0.906] versus bar 0.864 (capture 0.48, upper bound misses by 0.042, K1); at 10 workers "
        "0.848 [0.825, 0.873] (capture 0.63). Panel B shows standalone P/none: 1.105 at 6, 1.100 at 7, "
        "1.052 [1.023, 1.081] at 8 (passes K2 1.03 lower-bound gate), 1.000 [0.983, 1.017] at 9, and "
        "0.936 [0.909, 0.964] (-6.4 percent) at 10. K3 peak RSS stays within 1.4 percent and K4 finder is 4.82 ms."
    )
    return svg(324, o, aria)


def redistribution():
    """Causal capacity diagram: dedicated scanner vs full pool vs slack-only hybrid at 8W and 10W."""
    o = []

    text(
        o,
        14,
        20,
        "Moving work vs. stealing slack: why pooling loses at 8 workers and wins at 10",
        12,
        "bf-t-strong",
    )
    text(
        o,
        14,
        35,
        "Per-decode wall-time budgets (ms) from pooled-scan medians and traces; workers spend the same 173-193 ms scanning",
        9.8,
        "bf-tm",
    )

    x0, x1 = 152.0, 626.0
    max_ms = 230.0

    def xm(ms):
        return x0 + (ms / max_ms) * (x1 - x0)

    # ---- Top section: k = 8 workers (at the balanced knee, little slack) ----
    rect(o, 10, 48, 700, 148, "var(--bf-surface)", rx=7, stroke="var(--bf-frame)", sw=1.1)
    text(
        o,
        22,
        66,
        "At 8 workers (balanced knee: dedicated scanner 177.5 ms, workers ~185 ms, only 6.5% idle)",
        10.5,
        "bf-t-strong",
    )

    # Axis grid inside top box
    for tv in (0, 50, 100, 150, 200):
        tx = xm(tv)
        line(o, tx, 74, tx, 184, "var(--bf-vz-axis)" if tv == 0 else "var(--bf-vz-grid)", 0.9)

    # Row 1: Shipped none (8W) = 192.9 ms (scanner 177.5 + overlap)
    y_r1 = 80.0
    text(o, x0 - 8, y_r1 + 9, "Shipped (none)", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r1 + 20, "1 scanner + 8 workers", 8.5, "bf-tm", "end")
    # Worker bar (~180 ms busy + 12.9 ms idle/tail = 192.9 ms)
    bar_h(
        o,
        x0,
        y_r1,
        xm(180.4) - x0,
        18,
        "var(--bf-vz-1)",
        title="Shipped 8W workers busy: ~180 ms; worker idle share 6.5%",
    )
    rect(
        o,
        xm(180.4),
        y_r1,
        xm(192.9) - xm(180.4),
        18,
        "var(--bf-track)",
        stroke="var(--bf-vz-axis)",
        sw=1.0,
        dash="3 2",
        title="6.5% worker idle share waiting on serial scanner (none, 8W)",
    )
    text(
        o,
        x0 + 8,
        y_r1 + 13,
        "workers: M · C · E · R (in parallel with 177.5 ms serial scan)",
        9.2,
        "bf-t-strong",
        fill="var(--bf-vz-1-ink)",
    )
    text(o, xm(192.9) + 6, y_r1 + 13, "192.9 ms", 9.6, "bf-t-strong")

    # Row 2: Full pool P (8W) = 207.7 ms (workers carry w/8 + s/8 = +22.2 ms scan share + ~5 ms chain)
    y_r2 = 114.0
    text(o, x0 - 8, y_r2 + 9, "Full pool (P)", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r2 + 20, "8 workers absorb s/8", 8.5, "bf-tm", "end")
    bar_h(
        o,
        x0,
        y_r2,
        xm(180.0) - x0,
        18,
        "var(--bf-vz-1)",
        title="Worker M, C, E, R share at 8 workers",
    )
    rect(
        o,
        xm(180.0),
        y_r2,
        xm(202.2) - xm(180.0),
        18,
        "var(--bf-vz-2)",
        title="Scan CPU moved onto 8 workers: 177.7 / 8 = 22.2 ms per worker",
    )
    bar_h(
        o,
        xm(202.2),
        y_r2,
        xm(207.7) - xm(202.2),
        18,
        "var(--bf-vz-4)",
        title="Finder/copy/tail overhead (~5-6 ms)",
    )
    text(
        o,
        x0 + 8,
        y_r2 + 13,
        "workers: M · C · E · R",
        9.2,
        "bf-t-strong",
        fill="var(--bf-vz-1-ink)",
    )
    text(o, xm(191.1), y_r2 + 13, "+s/8", 8.8, "bf-t-strong", "middle", fill="var(--bf-vz-2-ink)")
    text(
        o,
        xm(207.7) + 6,
        y_r2 + 13,
        "207.7 ms (1.052 gm)",
        9.2,
        "bf-t-strong",
        fill="var(--bf-amber)",
    )

    # Row 3: Behind ideal C (SC 135.3 ms vs PC 158.1 ms -> exact +22.8 ms = s/8 gap!)
    y_r3 = 148.0
    text(o, x0 - 8, y_r3 + 9, "Behind ideal C", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r3 + 20, "SC vs PC (+s/8 gap)", 8.5, "bf-tm", "end")
    bar_h(
        o,
        x0,
        y_r3,
        xm(135.3) - x0,
        18,
        "var(--bf-vz-3)",
        title="SC (ideal scan + ideal C) at 8 workers: median 135.3 ms",
    )
    bar_h(
        o,
        xm(135.3),
        y_r3,
        xm(158.1) - xm(135.3),
        18,
        "var(--bf-vz-2)",
        title="PC minus SC at 8 workers: 158.1 - 135.3 = 22.8 ms (matches s/8 = 177.7/8 = 22.2 ms; raw capture 0.52, diagnostic ~0.73 after cold-store A7 adjustment)",
    )
    text(o, x0 + 8, y_r3 + 13, "SC ideal: 135.3 ms", 9.2, "bf-t-strong", fill="var(--bf-vz-3-ink)")
    text(o, xm(146.7), y_r3 + 13, "+22.8", 8.6, "bf-t-strong", "middle", fill="var(--bf-vz-2-ink)")
    text(o, xm(158.1) + 6, y_r3 + 13, "PC 158.1 ms (0.52 raw cap)", 9.2, "bf-t-strong")

    # ---- Bottom section: k = 10 workers (worker slack) + proposed hybrid ----
    rect(o, 10, 206, 700, 148, "var(--bf-surface)", rx=7, stroke="var(--bf-frame)", sw=1.1)
    text(
        o,
        22,
        224,
        "At 10 workers (20.2% worker slack) — and the slack-only hybrid for all counts",
        10.5,
        "bf-t-strong",
    )

    for tv in (0, 50, 100, 150, 200):
        tx = xm(tv)
        line(o, tx, 232, tx, 342, "var(--bf-vz-axis)" if tv == 0 else "var(--bf-vz-grid)", 0.9)
        text(o, tx, 366, f"{tv} ms", 9.4, "bf-tm", "middle")

    # Row 4: Shipped none (10W) = 197.6 ms (workers busy ~158 ms, 20.2% idle waiting for 189.6 ms scan)
    y_r4 = 238.0
    text(o, x0 - 8, y_r4 + 9, "Shipped (none)", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r4 + 20, "10W sit 20.2% idle", 8.5, "bf-tm", "end")
    bar_h(
        o,
        x0,
        y_r4,
        xm(157.7) - x0,
        18,
        "var(--bf-vz-1)",
        title="Shipped 10W workers busy (~79.8% of 197.6 ms wall)",
    )
    rect(
        o,
        xm(157.7),
        y_r4,
        xm(197.6) - xm(157.7),
        18,
        "var(--bf-track)",
        stroke="var(--bf-vz-axis)",
        sw=1.1,
        dash="3 2",
        title="20.2% worker idle share waiting on 189.6 ms serial scanner (none, 10W)",
    )
    text(
        o,
        x0 + 8,
        y_r4 + 13,
        "10 workers busy (79.8%)",
        9.2,
        "bf-t-strong",
        fill="var(--bf-vz-1-ink)",
    )
    text(o, xm(177.6), y_r4 + 13, "20.2% idle", 8.6, "bf-tm", "middle")
    text(o, xm(197.6) + 6, y_r4 + 13, "197.6 ms", 9.6, "bf-t-strong")

    # Row 5: Full pool P (10W) = 187.2 ms median (-5.3% med; P/none geometric mean 0.936, -6.4%)
    y_r5 = 272.0
    text(o, x0 - 8, y_r5 + 9, "Full pool (P)", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r5 + 20, "scan fits inside slack", 8.5, "bf-tm", "end")
    bar_h(
        o,
        x0,
        y_r5,
        xm(162.0) - x0,
        18,
        "var(--bf-vz-1)",
        title="Worker M, C, E, R share at 10 workers",
    )
    bar_h(
        o,
        xm(162.0),
        y_r5,
        xm(181.3) - xm(162.0),
        18,
        "var(--bf-vz-2)",
        title="Scan CPU moved onto 10 workers: 192.7 / 10 = 19.3 ms per worker",
    )
    bar_h(
        o,
        xm(181.3),
        y_r5,
        xm(187.2) - xm(181.3),
        18,
        "var(--bf-vz-4)",
        title="Finder/copy/idle residual (2.5% idle; median 187.2 ms vs 197.6 ms, -5.3%; P/none geometric mean 0.936, -6.4%)",
    )
    text(
        o,
        x0 + 8,
        y_r5 + 13,
        "10 workers: M · C · E · R",
        9.2,
        "bf-t-strong",
        fill="var(--bf-vz-1-ink)",
    )
    text(o, xm(171.6), y_r5 + 13, "+s/10", 8.4, "bf-t-strong", "middle", fill="var(--bf-vz-2-ink)")
    text(
        o,
        xm(187.2) + 6,
        y_r5 + 13,
        "187.2 ms (0.936 gm)",
        9.2,
        "bf-t-strong",
        fill="var(--bf-green-ink)",
    )

    # Row 6: Proposed slack-only hybrid (design, not measured)
    y_r6 = 306.0
    text(o, x0 - 8, y_r6 + 9, "Hybrid (proposed)", 9.6, "bf-t-strong", "end")
    text(o, x0 - 8, y_r6 + 20, "design, not measured", 8.5, "bf-tm", "end", italic=True)
    rect(
        o,
        x0,
        y_r6,
        xm(168.0) - x0,
        18,
        "var(--bf-green-tint)",
        rx=4,
        stroke="var(--bf-green)",
        sw=1.3,
        dash="4 2",
        title="Proposed slack-only hybrid: keep dedicated scanner; workers scan ahead only from idle job_wait slack",
    )
    text(
        o,
        x0 + 8,
        y_r6 + 13,
        "keep serial scanner; idle workers scan ahead only when waiting",
        9.1,
        "bf-t-strong",
        fill="var(--bf-green-ink)",
    )
    text(o, xm(168.0) + 6, y_r6 + 13, "no <=8W tax", 9.2, "bf-t-strong", fill="var(--bf-green-ink)")

    aria = (
        "Capacity breakdown comparing dedicated scanning, full worker pooling, and a proposed slack-only "
        "hybrid at 8 and 10 workers. At 8 workers, shipped workers sit only 6.5 percent idle (192.9 ms median), "
        "so moving 177.7 ms of scan CPU onto 8 workers adds 22.2 ms per worker and slows standalone decoding "
        "to 207.7 ms median (P/none geometric mean 1.052); behind ideal construction, the same 22.8 ms separates "
        "PC (158.1 ms) from SC (135.3 ms). At 10 workers, shipped workers sit 20.2 percent idle (197.6 ms median), "
        "so absorbing 19.3 ms per worker of scan work into slack speeds decoding to 187.2 ms median (-5.3 percent; "
        "P/none geometric mean 0.936, -6.4 percent). A proposed slack-only hybrid keeps the serial scanner and "
        "lets idle workers scan ahead only when waiting."
    )
    return svg(376, o, aria)


FIGS = {
    "taring": taring,
    "reservoir": reservoir,
    "killtest": killtest,
    "redistribution": redistribution,
}


def inject(path):
    """Replace each `<!-- fig:NAME -->` SVG line in the target Markdown file."""
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    out = []
    i = 0
    replaced = 0
    while i < len(lines):
        cur_line = lines[i]
        out.append(cur_line)
        m = re.match(r"<!--\s*fig:(\w+)\s*-->", cur_line.strip())
        if m and m.group(1) in FIGS:
            svg_line = FIGS[m.group(1)]()
            if i + 1 < len(lines) and lines[i + 1].lstrip().startswith("<svg"):
                out.append(svg_line)
                i += 2
            else:
                out.append(svg_line)
                i += 1
            replaced += 1
            continue
        i += 1
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(out) + "\n")
    print(f"injected {replaced} figure(s) into {path}")


def main(argv):
    """Print one named figure SVG or inject all figures into a Markdown post."""
    if len(argv) == 3 and argv[1] == "--inject":
        inject(argv[2])
        return 0
    if len(argv) == 2 and argv[1] in FIGS:
        print(FIGS[argv[1]]())
        return 0
    print(f"usage: {argv[0]} {{{'|'.join(FIGS)}}} | --inject POST.md", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
