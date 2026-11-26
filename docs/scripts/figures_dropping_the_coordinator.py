"""Figures for "Dropping the coordinator: atomic file locks on a measured frontier" (blog, 26 November 2026).

    python3 docs/scripts/figures_dropping_the_coordinator.py NAME     # print one figure's <svg>
    python3 docs/scripts/figures_dropping_the_coordinator.py --inject POST.md

`--inject` replaces the line after each `<!-- fig:NAME -->` marker in POST.md with that figure's
<svg> (one line, so Markdown leaves it alone).

The figures follow the dataviz method: colour by role from the blog.css tokens (--bf-vz-* and
--bf-*), validated in light and dark schemes, with solid hairline rules, bars at most 24 px with
4 px rounded data ends, and <title> hover tooltips on every data mark.

Sources:
- topology: frontier/README.md; docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md §1.1, §3.2.
- evidence: Paradigm 3 ("Swarms do not yet raise the roof", 24 November 2026) via
  docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md intro and §1.1 (269 comments on #145
  over 22-26 November: 124 coordinating agent, 60 workstation pings, 85 peer reviews/notes, 0 from
  four dormant inboxes).
- lifecycle: frontier/README.md; frontier/config.toml; frontier/tools/frontier.py.
- experiment: frontier/claims/three-arm-experiment.toml;
  docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md §3.4.
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


def text(o, x, y, s, size=11.5, cls="bf-tm", anchor="start", fill=None, weight=None, italic=False):
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


def rect(o, x, y, w, h, fill, rx=0, stroke=None, sw=1.2, title=None, dash=None):
    """Append an SVG rectangle with optional border, dash pattern, and hover title."""
    st = f"fill:{fill}"
    if stroke:
        st += f";stroke:{stroke};stroke-width:{sw}"
    if dash:
        st += f";stroke-dasharray:{dash}"
    t = f"<title>{esc(title)}</title>" if title else ""
    o.append(
        f'<rect x="{x:.1f}" y="{y:.1f}" width="{max(w, 0):.1f}" height="{max(h, 0):.1f}" '
        f'rx="{rx}" style="{st}">{t}</rect>'
    )


def bar_h(o, x0, y, length, h, fill, title=None):
    """Draw a horizontal bar from x0 with a square baseline and 4 px rounded data end."""
    r = min(4.0, h / 2.0, abs(length))
    t = f"<title>{esc(title)}</title>" if title else ""
    if length <= 0.5:
        o.append(
            f'<rect x="{x0:.1f}" y="{y:.1f}" width="2.0" height="{h:.1f}" '
            f'style="fill:{fill}">{t}</rect>'
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
    o.append(f'<path d="{d}" style="fill:{fill}">{t}</path>')


def arrow(o, x1, y1, x2, y2, colour="var(--bf-vz-axis)", sw=1.6, dash=""):
    """Append a straight line with a filled triangular arrowhead."""
    st = f"stroke:{colour};stroke-width:{sw}"
    if dash:
        st += f";stroke-dasharray:{dash}"
    o.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" style="{st}"/>')
    a = math.atan2(y2 - y1, x2 - x1)
    hl, hw = 7.5, 4.0
    pts = [
        (x2, y2),
        (x2 - hl * math.cos(a) + hw * math.sin(a), y2 - hl * math.sin(a) - hw * math.cos(a)),
        (x2 - hl * math.cos(a) - hw * math.sin(a), y2 - hl * math.sin(a) + hw * math.cos(a)),
    ]
    o.append(
        f'<polygon points="{" ".join(f"{u:.1f},{v:.1f}" for u, v in pts)}" style="fill:{colour}"/>'
    )


def topology():
    """Side-by-side architecture: Star hub (switchboard) vs. Shared frontier (harbour chart)."""
    o = []
    # Left panel: Star hub
    rect(o, 12, 12, 336, 306, "var(--bf-surface)", rx=8, stroke="var(--bf-frame)", sw=1.2)
    text(o, 26, 34, "Before: central hub (telephone switchboard)", 12.5, "bf-t-strong")
    text(o, 26, 51, "One operator re-phrases briefs and holds state in memory", 10.5, "bf-tm")

    # Central hub box
    rect(
        o,
        106,
        74,
        148,
        66,
        "var(--bf-amber-tint)",
        rx=7,
        stroke="var(--bf-amber)",
        sw=1.6,
        title="Coordinating agent: 124 of 269 recent #145 comments (46.1%), 4 single-day translation defects",
    )
    text(o, 180, 96, "Coordinating agent", 12, "bf-t-strong", "middle")
    text(o, 180, 113, "46.1% of #145 traffic", 10.5, "bf-tm", "middle")
    text(o, 180, 128, "long-lived, compacted context", 10, "bf-tm", "middle")

    # Maintainer relay box above/side
    rect(
        o,
        28,
        166,
        124,
        46,
        "var(--bf-surface)",
        rx=6,
        stroke="var(--bf-amber)",
        sw=1.2,
        dash="4 3",
        title="Manual copy-paste relay of prompts and reviews",
    )
    text(o, 90, 185, "Maintainer relay", 11, "bf-t-strong", "middle")
    text(o, 90, 201, "pasted briefs & reviews", 10, "bf-tm", "middle")

    # Shared chat thread
    rect(
        o,
        196,
        166,
        136,
        46,
        "var(--bf-surface)",
        rx=6,
        stroke="var(--bf-vz-axis)",
        sw=1.2,
        title="PR #145 (2,269 comments) plus 7 role inboxes (4 dormant at 0 posts)",
    )
    text(o, 264, 185, "Channel #145 + 7 inboxes", 10.5, "bf-t-strong", "middle")
    text(o, 264, 201, "2,269 comments; 4 dormant", 10, "bf-tm", "middle")

    arrow(o, 146, 140, 108, 165, "var(--bf-amber)")
    arrow(o, 214, 140, 248, 165, "var(--bf-amber)")

    # Bottom worker boxes (left)
    workers_l = [
        (26, "Worker A", "waits on brief"),
        (134, "Worker B", "waits on brief"),
        (242, "Cold review", "pasted back"),
    ]
    for wx, wname, wsub in workers_l:
        rect(o, wx, 244, 92, 46, "var(--bf-surface)", rx=6, stroke="var(--bf-frame)", sw=1.2)
        text(o, wx + 46, 263, wname, 11, "bf-t-strong", "middle")
        text(o, wx + 46, 279, wsub, 10, "bf-tm", "middle")
    arrow(o, 78, 212, 72, 243, "var(--bf-vz-axis)")
    arrow(o, 126, 212, 174, 243, "var(--bf-vz-axis)")
    arrow(o, 264, 212, 286, 243, "var(--bf-vz-axis)")
    text(o, 180, 308, "Lossy summaries · stale context · single routing bottleneck", 10, "bf-tm", "middle")

    # Right panel: Shared frontier
    rect(o, 372, 12, 336, 306, "var(--bf-surface)", rx=8, stroke="var(--bf-frame)", sw=1.2)
    text(o, 386, 34, "After: shared frontier (harbour chart & locks)", 12.5, "bf-t-strong")
    text(o, 386, 51, "Each fresh session reads the map, locks a berth, logs a delta", 10.5, "bf-tm")

    # Shared files layer
    rect(
        o,
        388,
        70,
        304,
        126,
        "var(--bf-blue-tint)",
        rx=7,
        stroke="var(--bf-blue)",
        sw=1.5,
        title="Repository files under frontier/: shared lossless state",
    )
    text(o, 540, 88, "Shared repository state (frontier/)", 11.5, "bf-t-strong", "middle")

    files = [
        (398, 98, 136, 38, "surface.tsv", "measured headroom", "var(--bf-green-tint)", "var(--bf-green)"),
        (546, 98, 136, 38, "hold/<id> (CAS)", "atomic git lease", "var(--bf-surface)", "var(--bf-blue)"),
        (398, 146, 136, 38, "claims/ & flags/", "disjoint paths & vetoes", "var(--bf-surface)", "var(--bf-blue)"),
        (546, 146, 136, 38, "deltas/ & dead-ends", "<=40-line memory", "var(--bf-surface)", "var(--bf-steel)"),
    ]
    for fx, fy, fw, fh, fn, fsub, ffill, fstr in files:
        rect(o, fx, fy, fw, fh, ffill, rx=5, stroke=fstr, sw=1.2, title=f"{fn}: {fsub}")
        text(o, fx + fw / 2, fy + 16, fn, 10.5, "bf-t-strong", "middle")
        text(o, fx + fw / 2, fy + 30, fsub, 9.5, "bf-tm", "middle")

    # Three fresh peer sessions below
    sessions = [
        (388, "Session 1", "smooth probe", "var(--bf-green)"),
        (494, "Session 2", "striated build", "var(--bf-blue)"),
        (600, "Session 3", "cold review", "var(--bf-ultra)"),
    ]
    for sx, sname, ssub, scol in sessions:
        rect(
            o,
            sx,
            228,
            92,
            48,
            "var(--bf-surface)",
            rx=6,
            stroke=scol,
            sw=1.5,
            title=f"{sname}: {ssub}; fresh context, ruptures after merge",
        )
        text(o, sx + 46, 248, sname, 11, "bf-t-strong", "middle")
        text(o, sx + 46, 264, ssub, 10, "bf-tm", "middle")
        arrow(o, sx + 34, 227, sx + 34, 198, scol)
        arrow(o, sx + 58, 198, sx + 58, 227, "var(--bf-vz-axis)", dash="3 2")

    # Generated status line
    text(o, 540, 294, "STATUS.md (local, mirrored to #943) · #944 Needs maintainer", 9.8, "bf-tm", "middle")
    text(o, 540, 309, "Zero routing hub · atomic CAS exclusion · fresh context per claim", 9.8, "bf-tm", "middle")

    aria = (
        "Side-by-side comparison of the old star hub and the new shared frontier. "
        "Before: a central coordinating agent wrote 46.1 percent of recent channel #145 comments, "
        "relayed pasted briefs through the maintainer across seven role inboxes, four of them dormant, "
        "and held state in a long-lived context. "
        "After: fresh independent sessions read surface.tsv, acquire atomic git compare-and-swap locks on hold branches, "
        "work on disjoint smooth or striated claims, record deltas of at most 40 lines plus dead ends, "
        "and leave STATUS.md generated locally and mirrored to issue #943."
    )
    return svg(328, o, aria)


def evidence():
    """Draw the two-panel evidence chart (Paradigm 3 stale-context reversal and #145 traffic)."""
    o = []

    # --- Left Panel: Paradigm 3 git-to-zig at 32M tokens ---
    lx0, lx1 = 128, 344
    lw = lx1 - lx0
    text(o, 14, 20, "A. Fresh resets beat swarms (32M tokens)", 12, "bf-t-strong")
    text(o, 14, 36, "Paradigm 3 git-to-zig test pass rate (%)", 10.5, "bf-tm")

    # Gridlines 0, 25, 50, 75, 100
    top_y = 54
    bot_y = 266
    for pct in (0, 25, 50, 75, 100):
        gx = lx0 + lw * pct / 100.0
        col = "var(--bf-vz-axis)" if pct == 0 else "var(--bf-vz-grid)"
        o.append(f'<line x1="{gx:.1f}" y1="{top_y}" x2="{gx:.1f}" y2="{bot_y}" style="stroke:{col};stroke-width:1"/>')
        text(o, gx, bot_y + 15, f"{pct}%", 10, "bf-tm", "middle")

    rows_l = [
        ("Model A", None, None, None),
        ("Continuous solo", 83.7, "var(--bf-vz-2)", "Model A continuous solo: 83.7% of 743 git tests at 32M weighted tokens"),
        ("Team of 8", 93.2, "var(--bf-vz-1)", "Model A self-organised team of 8: 93.2% at 32M weighted tokens"),
        ("Restarted solo", 97.0, "var(--bf-vz-3)", "Model A restarted solo across 8 fresh sessions: 97.0% at 32M weighted tokens"),
        ("Model B", None, None, None),
        ("Continuous solo", 26.6, "var(--bf-vz-2)", "Model B continuous solo: 26.6% at 32M weighted tokens"),
        ("Team of 8", 32.0, "var(--bf-vz-1)", "Model B self-organised team of 8: 32.0% at 32M weighted tokens"),
        ("Team of 16", 33.3, "var(--bf-vz-1)", "Model B self-organised team of 16: 33.3% (plateau) at 32M weighted tokens"),
        ("Team of 32", 23.2, "var(--bf-vz-4)", "Model B self-organised team of 32: 23.2% (below continuous solo) at 32M tokens"),
        ("Restarted solo", 41.9, "var(--bf-vz-3)", "Model B restarted solo across 8 fresh sessions: 41.9% at 32M weighted tokens"),
    ]
    y = 58
    for label, val, col, tip in rows_l:
        if val is None:
            text(o, 14, y + 12, label, 10.5, "bf-t-strong")
            y += 18
            continue
        bh = 14
        blen = lw * val / 100.0
        text(o, lx0 - 8, y + 11, label, 10.5, "bf-tm", "end")
        bar_h(o, lx0, y, blen, bh, col, title=tip)
        tx = lx0 + blen + 5 if val < 82 else lx0 + blen - 6
        anch = "start" if val < 82 else "end"
        tfill = "var(--bf-vz-3-ink)" if (val >= 82 and col == "var(--bf-vz-3)") else (
            "var(--bf-vz-1-ink)" if (val >= 82 and col == "var(--bf-vz-1)") else (
                "var(--bf-vz-2-ink)" if (val >= 82 and col == "var(--bf-vz-2)") else None
            )
        )
        text(o, tx, y + 11, f"{val:.1f}%", 10, "bf-t-strong", anch, fill=tfill)
        y += 20

    # Vertical divider
    o.append('<line x1="366" y1="12" x2="366" y2="292" style="stroke:var(--bf-frame);stroke-width:1"/>')

    # --- Right Panel: 5-day #145 traffic breakdown ---
    rx0, rx1 = 496, 646
    rw = rx1 - rx0
    text(o, 380, 20, "B. Where #145 traffic went (22-26 Nov)", 12, "bf-t-strong")
    text(o, 380, 36, "Share of 269 channel comments (%)", 10.5, "bf-tm")

    for pct in (0, 20, 40, 60):
        gx = rx0 + rw * pct / 60.0
        col = "var(--bf-vz-axis)" if pct == 0 else "var(--bf-vz-grid)"
        o.append(f'<line x1="{gx:.1f}" y1="{top_y}" x2="{gx:.1f}" y2="{bot_y}" style="stroke:{col};stroke-width:1"/>')
        text(o, gx, bot_y + 15, f"{pct}%", 10, "bf-tm", "middle")

    rows_r = [
        ("Coordinating agent", "107 log, 13 queued, 4 fix", 124, 46.1, "var(--bf-vz-2)"),
        ("Peer reviews & notes", "code & math reviews", 85, 31.6, "var(--bf-vz-1)"),
        ("Workstation pings", "automated SHA-256 logs", 60, 22.3, "var(--bf-vz-3)"),
        ("4 dormant inboxes", "still wired into routing", 0, 0.0, "var(--bf-vz-axis)"),
    ]
    ry = 72
    for label, sub, count, pct, col in rows_r:
        bh = 20
        blen = rw * pct / 60.0
        text(o, rx0 - 8, ry + 9, label, 10.2, "bf-t-strong" if count == 124 else "bf-tm", "end")
        text(o, rx0 - 8, ry + 22, sub, 9.2, "bf-tm", "end")
        bar_h(
            o,
            rx0,
            ry,
            blen,
            bh,
            col,
            title=f"{label}: {count} of 269 comments ({pct:.1f}%)",
        )
        text(o, rx0 + blen + 6, ry + 14, f"{pct:.1f}% ({count})", 10.2, "bf-t-strong")
        ry += 48

    text(o, 380, 294, "22-day total: 2,269 comments on #145 (2,748 across 8 threads).", 10, "bf-tm")

    aria = (
        "Two-panel chart. Panel A shows Paradigm 3 git-to-zig test pass rates at 32 million weighted tokens: "
        "for Model A, continuous solo 83.7 percent, team of 8 93.2 percent, restarted solo 97.0 percent; "
        "for Model B, continuous solo 26.6 percent, team of 8 32.0 percent, team of 16 33.3 percent, "
        "team of 32 23.2 percent, restarted solo 41.9 percent. "
        "Panel B shows the 269 comments posted to shared channel #145 from 22 to 26 November: "
        "coordinating agent 124 comments or 46.1 percent, peer reviews and notes 85 or 31.6 percent, "
        "automated workstation pings 60 or 22.3 percent, and four dormant role inboxes 0 comments."
    )
    return svg(308, o, aria)


def lifecycle():
    """Hybrid-system lifecycle of one claim (smooth/striated fork) + 7-stage blog board inset."""
    o = []
    text(o, 16, 22, "One claim's lifecycle from idea to rupture (with the smooth/striated boundary)", 12.5, "bf-t-strong")

    # Top row: idea -> open -> CAS claim
    nodes_top = [
        (16, 46, 86, 44, "idea", "free to open", "var(--bf-surface)", "var(--bf-frame)"),
        (134, 46, 102, 44, "open", "<=12 admitted", "var(--bf-surface)", "var(--bf-blue)"),
        (268, 46, 134, 44, "hold/<id> CAS", "acquired; <=3 live", "var(--bf-blue-tint)", "var(--bf-blue)"),
    ]
    for nx, ny, nw, nh, nt, ns, nfill, nstr in nodes_top:
        rect(o, nx, ny, nw, nh, nfill, rx=6, stroke=nstr, sw=1.4, title=f"{nt}: {ns}")
        text(o, nx + nw / 2, ny + 19, nt, 11, "bf-t-strong", "middle")
        text(o, nx + nw / 2, ny + 34, ns, 9.8, "bf-tm", "middle")
    arrow(o, 102, 68, 133, 68, "var(--bf-vz-axis)")
    text(o, 118, 61, "admit", 9.2, "bf-tm", "middle")
    arrow(o, 236, 68, 267, 68, "var(--bf-blue)")
    text(o, 252, 61, "claim", 9.2, "bf-tm", "middle")

    # Fork into Smooth (top-right) and Striated (bottom-right)
    rect(
        o,
        436,
        36,
        134,
        44,
        "var(--bf-green-tint)",
        rx=6,
        stroke="var(--bf-green)",
        sw=1.4,
        title="Smooth zone: docs, probes, examples; author merges once green",
    )
    text(o, 503, 55, "Smooth zone", 11, "bf-t-strong", "middle")
    text(o, 503, 70, "docs, probes; self-merge", 9.5, "bf-tm", "middle")

    rect(
        o,
        436,
        92,
        134,
        52,
        "var(--bf-amber-tint)",
        rx=6,
        stroke="var(--bf-amber)",
        sw=1.4,
        title="Striated zone: src, bench, CI, protected paths; disjoint paths, frozen stop test, different-agent review",
    )
    text(o, 503, 110, "Striated zone", 11, "bf-t-strong", "middle")
    text(o, 503, 124, "disjoint paths, frozen gate", 9.3, "bf-tm", "middle")
    text(o, 503, 137, "different-agent merge", 9.3, "bf-tm", "middle")

    arrow(o, 402, 62, 435, 58, "var(--bf-green)")
    arrow(o, 402, 76, 435, 114, "var(--bf-amber)")

    # PR + Review + Verify + Release + Rupture (middle row flowing left to right -> down -> left)
    rect(
        o,
        600,
        54,
        104,
        64,
        "var(--bf-surface)",
        rx=6,
        stroke="var(--bf-blue)",
        sw=1.4,
        title="PR opens with Claim: <id>, status=review; hold stays live until merge",
    )
    text(o, 652, 74, "Open PR", 11, "bf-t-strong", "middle")
    text(o, 652, 89, "Claim: <id>", 9.8, "bf-tm", "middle")
    text(o, 652, 104, "status = review", 9.5, "bf-tm", "middle")

    arrow(o, 570, 58, 599, 74, "var(--bf-green)")
    arrow(o, 570, 114, 599, 98, "var(--bf-amber)")

    # Lower pipeline: verify & merge -> release & delta -> rupture
    low_y = 166
    rect(
        o,
        474,
        low_y,
        152,
        46,
        "var(--bf-surface)",
        rx=6,
        stroke="var(--bf-blue)",
        sw=1.4,
        title="verify --session, head-sha check, expectedHeadSha squash merge",
    )
    text(o, 550, low_y + 19, "verify + merge gates", 10.8, "bf-t-strong", "middle")
    text(o, 550, low_y + 34, "head-sha + expectedHeadSha", 9.5, "bf-tm", "middle")

    rect(
        o,
        282,
        low_y,
        156,
        46,
        "var(--bf-surface)",
        rx=6,
        stroke="var(--bf-steel)",
        sw=1.4,
        title="release hold/<id>, append <=40-line delta, update surface.tsv / dead-ends.tsv",
    )
    text(o, 360, low_y + 19, "release + <=40-line delta", 10.8, "bf-t-strong", "middle")
    text(o, 360, low_y + 34, "update surface / dead-ends", 9.5, "bf-tm", "middle")

    rect(
        o,
        126,
        low_y,
        118,
        46,
        "var(--bf-ultra-tint)",
        rx=6,
        stroke="var(--bf-ultra)",
        sw=1.5,
        title="Asignifying rupture: session ends; next claim starts on a fresh context",
    )
    text(o, 185, low_y + 19, "Rupture", 11, "bf-t-strong", "middle")
    text(o, 185, low_y + 34, "session ends fresh", 9.5, "bf-tm", "middle")

    # Flag immune-system callout on far left
    rect(
        o,
        16,
        118,
        92,
        94,
        "var(--bf-amber-tint)",
        rx=6,
        stroke="var(--bf-amber)",
        sw=1.2,
        dash="4 3",
        title="Local flag blocks only its named claim; unrelated claims keep running",
    )
    text(o, 62, 138, "Local flag", 10.5, "bf-t-strong", "middle")
    text(o, 62, 154, "blocks only", 9.5, "bf-tm", "middle")
    text(o, 62, 168, "this claim;", 9.5, "bf-tm", "middle")
    text(o, 62, 182, "resolved by", 9.5, "bf-tm", "middle")
    text(o, 62, 196, "another peer", 9.5, "bf-tm", "middle")

    # Connect Open PR down to verify + merge gates, then left to release, then left to rupture
    o.append(
        f'<path d="M652,118 L652,{low_y + 23} L627,{low_y + 23}" '
        f'style="fill:none;stroke:var(--bf-blue);stroke-width:1.6"/>'
    )
    o.append(
        f'<polygon points="626,{low_y + 23} 634,{low_y + 19} 634,{low_y + 27}" style="fill:var(--bf-blue)"/>'
    )
    arrow(o, 474, low_y + 23, 439, low_y + 23, "var(--bf-steel)")
    arrow(o, 282, low_y + 23, 245, low_y + 23, "var(--bf-ultra)")
    # Flag arrow pointing at hold/<id>
    o.append(
        '<path d="M108,140 L335,140 L335,91" style="fill:none;stroke:var(--bf-amber);stroke-width:1.3;stroke-dasharray:3 3"/>'
    )
    o.append('<polygon points="335,90 331,98 339,98" style="fill:var(--bf-amber)"/>')

    # Inset: 7-stage blog board
    o.append('<line x1="16" y1="230" x2="704" y2="230" style="stroke:var(--bf-frame);stroke-width:1"/>')
    text(o, 16, 250, "Blog board inset (frontier/blog/*.toml): seven stages, with review enforced by a different agent", 11, "bf-t-strong")

    stages = [
        ("idea", ""),
        ("outline", ""),
        ("draft", "one agent"),
        ("review", "a different one"),
        ("revise", ""),
        ("ready", "publish check"),
        ("published", "merged"),
    ]
    sw_box = 84
    gap = 16
    sx0 = 16
    sy = 262
    for i, (st, sub) in enumerate(stages):
        x = sx0 + i * (sw_box + gap)
        hi = st == "draft"
        rev = st == "review"
        fill = (
            "var(--bf-blue-tint)"
            if hi
            else ("var(--bf-amber-tint)" if rev else "var(--bf-surface)")
        )
        stroke = (
            "var(--bf-blue)"
            if hi
            else ("var(--bf-amber)" if rev else "var(--bf-frame)")
        )
        rect(o, x, sy, sw_box, 38, fill, rx=5, stroke=stroke, sw=1.5 if (hi or rev) else 1.2)
        text(o, x + sw_box / 2, sy + 16, st, 10.5, "bf-t-strong" if (hi or rev) else "bf-tm", "middle")
        if sub:
            text(o, x + sw_box / 2, sy + 30, sub, 9.0, "bf-tm", "middle")
        if i < len(stages) - 1:
            arrow(o, x + sw_box, sy + 19, x + sw_box + gap - 1, sy + 19, "var(--bf-vz-axis)", sw=1.3)

    aria = (
        "Lifecycle of one claim on the frontier. An idea is admitted to open within the 12-claim cap, "
        "acquired via atomic compare-and-swap on hold/<id> within the 3-hold canary cap, "
        "and enters either the smooth zone (self-merged when green) or the striated zone "
        "(disjoint paths, frozen stop test, different-agent review). "
        "Opening a pull request sets status to review while keeping the hold live until verify and merge gates pass, "
        "followed by release, an append-only delta of at most 40 lines, and session rupture. "
        "A local flag blocks only its named claim. An inset shows the seven-stage blog board "
        "from idea to published, with draft highlighted and review marked as enforced by a different agent."
    )
    return svg(312, o, aria)


def experiment():
    """Three-arm matched-token falsification design (6M/arm + 2M anchor) and frozen C2 decision regions."""
    o = []
    text(o, 16, 22, "Pre-registered three-arm design: 6M weighted tokens per arm (plus 2M anchor)", 12.5, "bf-t-strong")

    bx0 = 166
    bw = 380  # 6.0M = 380 px
    scale = bw / 6.0

    # Axis gridlines 0..6M
    top_y = 42
    bot_y = 190
    for m in (0, 2, 4, 6):
        gx = bx0 + m * scale
        col = "var(--bf-vz-axis)" if m == 0 else "var(--bf-vz-grid)"
        o.append(f'<line x1="{gx:.1f}" y1="{top_y}" x2="{gx:.1f}" y2="{bot_y}" style="stroke:{col};stroke-width:1"/>')
        text(o, gx, bot_y + 15, f"{m}M", 10, "bf-tm", "middle")

    arms = [
        (
            "Calibration anchor",
            "fits solo curve f(2M)",
            [(2.0, "Solo 2M", "var(--bf-vz-3)", "var(--bf-vz-3-ink)", "Single fresh session (2M) to anchor scaling exponent lambda")],
        ),
        (
            "Arm 1: Orchestrated",
            "old hub + pasted briefs",
            [
                (1.5, "Hub 1.5M", "var(--bf-vz-2)", "var(--bf-vz-2-ink)", "Coordinating agent: 1.5M tokens"),
                (3.5, "Worker 3.5M", "var(--bf-vz-1)", "var(--bf-vz-1-ink)", "Dispatched worker sessions: 3.5M tokens"),
                (1.0, "Int 1.0M", "var(--bf-vz-4)", "var(--bf-vz-4-ink)", "Integration & review: 1.0M tokens"),
            ],
        ),
        (
            "Arm 2: Restarted solo",
            "3 fresh sessions, no hub",
            [
                (2.0, "Run 1 (2M)", "var(--bf-vz-3)", "var(--bf-vz-3-ink)", "Fresh solo session 1: 2.0M tokens + <=40-line delta"),
                (2.0, "Run 2 (2M)", "var(--bf-vz-3)", "var(--bf-vz-3-ink)", "Fresh solo session 2: 2.0M tokens + <=40-line delta"),
                (2.0, "Run 3 (2M)", "var(--bf-vz-3)", "var(--bf-vz-3-ink)", "Fresh solo session 3: 2.0M tokens + <=40-line delta"),
            ],
        ),
        (
            "Arm 3: Flat frontier",
            "disjoint claims + audit",
            [
                (2.0, "Claim A 2M", "var(--bf-vz-1)", "var(--bf-vz-1-ink)", "Disjoint builder A (A5+A6a fold): 2.0M tokens"),
                (2.0, "Claim B 2M", "var(--bf-vz-1)", "var(--bf-vz-1-ink)", "Disjoint builder B (E_consume / A6c+E): 2.0M tokens"),
                (1.2, "Int 1.2M", "var(--bf-vz-4)", "var(--bf-vz-4-ink)", "Systems integration + bench recipe: 1.2M tokens"),
                (0.8, "Audit", "var(--bf-vz-2)", "var(--bf-vz-2-ink)", "Cold independent audit: 0.8M tokens"),
            ],
        ),
    ]

    y = 48
    bh = 24
    for name, sub, segs in arms:
        text(o, bx0 - 10, y + 10, name, 10.5, "bf-t-strong", "end")
        text(o, bx0 - 10, y + 23, sub, 9.5, "bf-tm", "end")
        cx = bx0
        for val, lab, fill, ink, tip in segs:
            seg_w = val * scale
            rect(o, cx, y, seg_w - 1.5, bh, fill, rx=3, stroke="var(--bf-surface)", sw=1.0, title=tip)
            if seg_w >= 46:
                text(o, cx + seg_w / 2 - 1, y + 15.5, lab, 9.8, "bf-t-strong", "middle", fill=ink)
            cx += seg_w
        y += 36

    # Right-side annotation box: shared target
    rect(o, 562, 46, 144, 144, "var(--bf-surface)", rx=6, stroke="var(--bf-frame)", sw=1.2)
    text(o, 634, 66, "Shared target", 11, "bf-t-strong", "middle")
    lines_r = [
        "Worker speed behind",
        "zero-copy S oracle",
        "(8W wikidata x4)",
        "",
        "Byte-exact SHA-256",
        "Miri + clippy green",
        "RSS <= 1.10x base",
        "9 scored dimensions",
    ]
    ly = 84
    for ln in lines_r:
        if ln:
            text(o, 634, ly, ln, 9.5, "bf-tm", "middle")
        ly += 13

    # Frozen C2 decision regions below
    o.append('<line x1="16" y1="216" x2="704" y2="216" style="stroke:var(--bf-frame);stroke-width:1"/>')
    text(o, 16, 236, "Frozen C2 decision rule (written before data; never relaxed after seeing results)", 11, "bf-t-strong")

    # Three outcome boxes
    outcomes = [
        (
            16,
            234,
            "Adopt flat frontier (Arm 3)",
            "Beat/match Arm 1, cut coord tax >=50%,",
            "<=1 relay; beat solo >=2pp OR within 2pp",
            "+ >=2x faster + >=1 defect caught",
            "var(--bf-green-tint)",
            "var(--bf-green)",
        ),
        (
            262,
            218,
            "Fall back to restarted solo",
            "Arm 2 beats Arm 3 by >3pp at 6M",
            "with zero uncaught defects.",
            "Keep frontier/ as solo restart harness",
            "var(--bf-amber-tint)",
            "var(--bf-amber)",
        ),
        (
            492,
            212,
            "Void / inconclusive",
            "Any byte-exact failure, >6.3M spend,",
            "or bench control edit voids the arm.",
            "1 run/arm is a pilot; check variance",
            "var(--bf-surface)",
            "var(--bf-vz-axis)",
        ),
    ]
    for ox, ow, ot, l1, l2, l3, ofill, ostr in outcomes:
        rect(o, ox, 246, ow, 66, ofill, rx=6, stroke=ostr, sw=1.4)
        text(o, ox + 10, 263, ot, 10.5, "bf-t-strong")
        text(o, ox + 10, 278, l1, 9.5, "bf-tm")
        text(o, ox + 10, 291, l2, 9.5, "bf-tm")
        text(o, ox + 10, 304, l3, 9.5, "bf-tm")

    aria = (
        "Pre-registered three-arm falsification design at 6 million weighted tokens per arm plus a 2 million "
        "solo calibration anchor. Arm 1 splits 1.5M hub, 3.5M workers, 1.0M integration. "
        "Arm 2 runs three sequential 2M fresh-context solo sessions. "
        "Arm 3 runs two 2M disjoint builder claims, 1.2M systems integration, and 0.8M cold independent audit. "
        "All arms target worker-side speedup behind the zero-copy scanner oracle at 8 workers, gated on byte-exactness. "
        "Frozen decision rules adopt Arm 3 if it cuts coordination tax by at least 50 percent with at most one relay "
        "and beats or matches solo with speedup and defect catch, or fall back to restarted solo if Arm 2 wins by over 3 percentage points."
    )
    return svg(322, o, aria)


FIGS = {
    "topology": topology,
    "evidence": evidence,
    "lifecycle": lifecycle,
    "experiment": experiment,
}


def inject(path):
    """Replace each `<!-- fig:NAME -->` SVG line in the target Markdown file."""
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    out = []
    i = 0
    replaced = 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        m = re.match(r"<!--\s*fig:(\w+)\s*-->", line.strip())
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
