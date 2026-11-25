"""Figures for "The decoder as a queueing network": the story of applying queueing theory.

    python3 docs/scripts/figures_queueing_story.py NAME

NAME is one of: interaction, timeline, shapes, little, migration, models, slicea, organisations.
Each prints one inline SVG in the blog's figure style (classes bf-tm, bf-t-strong, bf-rule; colours
from the --bf-* tokens, so both themes work). Sources, per figure:

- interaction: journal §44, the four-arm oracle at 10 workers (A 1, B 0.976, C 0.806, D 0.741;
  D/C 0.920 [0.901, 0.939]); B at 1 and 6 workers 0.830 and 0.850.
- timeline: the dated journals and posts named in its rows.
- shapes, little, migration, organisations: schematics; migration's numbers are invented for
  teaching, and the figure says so.
- models: journal §§52, 59-61; reviews/neon-course-mining.md (the surrogate within 1-4 ms at 3-8
  workers); the simulator is proposed, not built.
- slicea: ideal-parts/results/slice-a/README.md (run-20261125T105502Z), the ceiling table.
"""
import sys

W = 720


def svg(h, body, aria):
    return f'<svg viewBox="0 0 {W} {h}" role="img" aria-label="{aria}">' + "".join(body) + "</svg>"


def text(o, x, y, s, size=12, cls="bf-tm", anchor="start", extra=""):
    o.append(f'<text class="{cls}" x="{x}" y="{y}" text-anchor="{anchor}" style="font-size:{size}px{extra}">{s}</text>')


def box(o, x, y, w, h, fill, stroke, rx=6, extra=""):
    o.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" style="fill:{fill};stroke:{stroke};stroke-width:1.5{extra}"/>')


def arrow(o, x1, y1, x2, y2, colour="var(--bf-frame)", width=1.8, dash=""):
    import math
    o.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" style="stroke:{colour};stroke-width:{width}{";stroke-dasharray:" + dash if dash else ""}"/>')
    a = math.atan2(y2 - y1, x2 - x1)
    p = [(x2, y2), (x2 - 9 * math.cos(a) + 5 * math.sin(a), y2 - 9 * math.sin(a) - 5 * math.cos(a)),
         (x2 - 9 * math.cos(a) - 5 * math.sin(a), y2 - 9 * math.sin(a) + 5 * math.cos(a))]
    o.append(f'<polygon points="{" ".join(f"{a:.1f},{b:.1f}" for a, b in p)}" style="fill:{colour}"/>')


def interaction():
    o = []
    # corners of the square
    xl, xr, yt, yb, bw, bh = 150, 470, 60, 250, 170, 74
    corners = [
        (xl, yt, "everything real", "1.000", "var(--bf-surface)", "var(--bf-frame)"),
        (xr, yt, "emission's codes supplied", "0.976 [0.936, 1.017]", "var(--bf-blue-tint)", "var(--bf-blue)"),
        (xl, yb, "scanner's output supplied", "0.806 [0.782, 0.831]", "var(--bf-amber-tint)", "var(--bf-amber)"),
        (xr, yb, "both supplied", "0.741 [0.717, 0.766]", "var(--bf-green-tint)", "var(--bf-green)"),
    ]
    for x, y, lab, val, fill, stroke in corners:
        box(o, x - bw / 2, y - bh / 2, bw, bh, fill, stroke)
        text(o, x, y - 8, lab, 12.5, anchor="middle")
        text(o, x, y + 16, val, 15, "bf-t-strong", "middle")
    arrow(o, xl + bw / 2 + 4, yt, xr - bw / 2 - 6, yt)
    text(o, (xl + xr) / 2, yt - 10, "supply emission's codes", 12, anchor="middle")
    text(o, (xl + xr) / 2, yt + 18, "−2.4%, unresolved", 13, "bf-t-strong", "middle", ";fill:var(--bf-blue)")
    arrow(o, xl + bw / 2 + 4, yb, xr - bw / 2 - 6, yb)
    text(o, (xl + xr) / 2, yb - 10, "supply emission's codes", 12, anchor="middle")
    text(o, (xl + xr) / 2, yb + 18, "−8%", 13, "bf-t-strong", "middle", ";fill:var(--bf-green)")
    text(o, (xl + xr) / 2, yb + 34, "0.920 [0.901, 0.939]", 11.5, anchor="middle", extra=";fill:var(--bf-green)")
    arrow(o, xl, yt + bh / 2 + 4, xl, yb - bh / 2 - 6)
    text(o, xl - 12, (yt + yb) / 2 + 4, "supply the scanner: −19%", 12, anchor="end")
    arrow(o, xr, yt + bh / 2 + 4, xr, yb - bh / 2 - 6)
    text(o, xr + 12, (yt + yb) / 2 + 4, "−24%", 12)
    text(o, W / 2, 322, "The same change, the codes supplied for emission, is worth nothing on the top edge and 8% on the bottom.", 12.5, anchor="middle")
    text(o, W / 2, 340, "At 1 and 6 workers, with the workers the bottleneck, it is worth 17% and 15% (0.830, 0.850).", 12.5, anchor="middle")
    return svg(352, o, "A square of four decoder configurations at 10 workers. Everything real takes 1.000; with "
               "emission's codes supplied 0.976, unresolved; with the scanner's output supplied 0.806; with both "
               "supplied 0.741. Supplying the codes is worth nothing along the top edge and 8% along the bottom, "
               "once the scanner has been relieved.")


TIMELINE = [  # date, what was asked or found, the queueing name, colour
    ("27 Oct", "on CI, 4 workers give 1.5–1.9×: the serial scan limits, not the window", "serial station", "amber"),
    ("28 Oct", "the decoder peaks at 4 workers; IPC halves as workers are added", "the knee; load-dependent service", "blue"),
    ("29 Oct", "teams of two, as many as keep the tables near 0.7 of the L3", "capacity provisioning", "green"),
    ("31 Oct", "lookups in flight = latency × rate inside one worker's walk", "Little's law", "steel"),
    ("1 Nov", "slow job claims are mostly waiting behind busy workers", "waiting time and utilisation", "steel"),
    ("1 Nov", "four workers spend 32% more CPU a byte on the same instructions", "load-dependent service", "blue"),
    ("5 Nov", "the window from 2 to 4 slots a thread: 8–10% off the wall", "admission; head-of-line blocking", "green"),
    ("5 Nov", "co-run placement: thread pairs, a walk token, interleaving", "discipline and placement", "green"),
    ("7 Nov", "each block in flight slows the others; three teams beat four", "the knee; a cycle time of max(S, αP)", "blue"),
    ("9 Nov", "a pit crew of service threads: the single thread became the bottleneck", "tandem stations; bottleneck", "amber"),
    ("17 Nov", "seven workers of ten finish sooner", "capacity provisioning", "green"),
    ("21 Nov", "fast alone, slow in a crowd", "load-dependent service", "blue"),
    ("24 Nov", "the scanner hides the emitter", "a masked bottleneck", "amber"),
]


def timeline():
    o = []
    w, top, row = 480, 34, 42
    text(o, 16, 18, "Performance questions, and their names in queueing theory", 13, "bf-t-strong")
    lx = 66
    o.append(f'<line x1="{lx}" x2="{lx}" y1="{top - 4}" y2="{top + row * (len(TIMELINE) - 1) + 18}" style="stroke:var(--bf-frame);stroke-width:2"/>')
    for i, (d, what, name, c) in enumerate(TIMELINE):
        y = top + row * i
        text(o, lx - 12, y + 9, d, 12.5, "bf-t-strong", "end")
        o.append(f'<circle cx="{lx}" cy="{y + 5}" r="5" style="fill:var(--bf-{c})"/>')
        text(o, lx + 14, y + 9, name, 13, "bf-t-strong", extra=f";fill:var(--bf-{c})")
        text(o, lx + 14, y + 27, what, 12)
    h = top + row * len(TIMELINE) - 6
    return (f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="A timeline from 27 October to 24 November of the '
            "decoder's performance questions, each with its name in queueing theory: the serial station, the knee, "
            "load-dependent service, capacity provisioning, Little's law, waiting and utilisation, admission and "
            "head-of-line blocking, discipline and placement, tandem stations and a masked bottleneck.\">"
            + "".join(o) + "</svg>")


def server(o, x, y, r=16, c="steel"):
    o.append(f'<circle cx="{x}" cy="{y}" r="{r}" style="fill:var(--bf-{c}-tint);stroke:var(--bf-{c});stroke-width:1.5"/>')


def queue(o, x, y, n=3):
    for i in range(n):
        box(o, x + i * 13, y - 8, 10, 16, "var(--bf-surface)", "var(--bf-frame)", 2)


def shapes():
    o = []
    cols = [(72, "single server", "one queue, one processor"), (216, "parallel group", "one queue, k servers"),
            (360, "tandem", "stages in series"), (504, "fork and join", "parts that must all finish"),
            (648, "feedback", "progress returns permits")]
    for cx, name, gloss in cols:
        text(o, cx, 24, name, 13, "bf-t-strong", "middle")
        text(o, cx, 42, gloss, 11.5, anchor="middle")
    y = 110
    # single
    queue(o, 26, y); arrow(o, 64, y, 82, y); server(o, 100, y)
    # parallel
    queue(o, 160, y); arrow(o, 198, y, 222, y - 30); arrow(o, 198, y, 222, y); arrow(o, 198, y, 222, y + 30)
    for dy in (-30, 0, 30):
        server(o, 240, y + dy, 13)
    # tandem
    queue(o, 300, y, 2); arrow(o, 326, y, 338, y); server(o, 352, y, 13); queue(o, 370, y, 2); arrow(o, 396, y, 404, y); server(o, 418, y, 13, "amber")
    # fork-join
    arrow(o, 452, y, 476, y - 26); arrow(o, 452, y, 476, y + 26)
    server(o, 490, y - 30, 12); server(o, 490, y + 30, 12)
    arrow(o, 504, y - 26, 528, y - 4); arrow(o, 504, y + 26, 528, y + 4)
    box(o, 530, y - 12, 26, 24, "var(--bf-green-tint)", "var(--bf-green)", 4)
    text(o, 543, y + 4, "J", 12, "bf-t-strong", "middle")
    text(o, 446, y + 4, "F", 12, "bf-t-strong", "end")
    # feedback
    queue(o, 600, y, 2); arrow(o, 626, y, 640, y); server(o, 656, y, 14)
    o.append(f'<path d="M672,{y} L700,{y} L700,{y + 44} L596,{y + 44} L596,{y + 12}" style="fill:none;stroke:var(--bf-green);stroke-width:1.6;stroke-dasharray:5 4"/>')
    o.append(f'<polygon points="596,{y + 10} 591,{y + 19} 601,{y + 19}" style="fill:var(--bf-green)"/>')
    text(o, 648, y + 62, "a permit returns", 11, anchor="middle", extra=";fill:var(--bf-green)")
    text(o, W / 2, 204, "The decoder uses three of them: a tandem (scanner, then workers), a parallel group (the workers)", 12.5, anchor="middle")
    text(o, W / 2, 222, "and feedback (the in-order reader returns admission). Each block goes to one worker, so it is not a fork and join.", 12.5, anchor="middle")
    return svg(234, o, "Five shapes of queueing system: a single server, a parallel group of servers sharing a queue, a "
               "tandem of stages in series, a fork and join, and a feedback loop that returns permits. The decoder "
               "combines a tandem, a parallel group and feedback.")


def little():
    o = []
    # five jobs, arrival and departure times (illustrative), on t in 0..10
    jobs = [(0.5, 3.5), (1.0, 5.0), (2.5, 6.0), (4.0, 8.5), (6.5, 9.0)]
    L, R0 = 60, 690
    x = lambda t: L + (R0 - L) * t / 10
    # top: n(t) as a step function
    ty0, ty1 = 40, 150
    ymax = 4
    y = lambda n: ty1 - (ty1 - ty0) * n / ymax
    ts = sorted({0, 10, *[a for a, _ in jobs], *[d for _, d in jobs]})
    n = lambda t: sum(1 for a, d in jobs if a <= t < d)
    pts = []
    for i in range(len(ts) - 1):
        k = n(ts[i])
        pts += [(x(ts[i]), y(k)), (x(ts[i + 1]), y(k))]
    poly = f"{x(0)},{y(0)} " + " ".join(f"{a:.1f},{b:.1f}" for a, b in pts) + f" {x(10)},{y(0)}"
    o.append(f'<polygon points="{poly}" style="fill:var(--bf-blue);opacity:0.3"/>')
    o.append(f'<polyline points="{" ".join(f"{a:.1f},{b:.1f}" for a, b in pts)}" style="fill:none;stroke:var(--bf-blue);stroke-width:2"/>')
    o.append(f'<line class="bf-rule" x1="{L}" x2="{R0}" y1="{ty1}" y2="{ty1}"/>')
    for k in range(ymax + 1):
        text(o, L - 8, y(k) + 4, str(k), 11, anchor="end")
    text(o, 20, 24, "Count the jobs present at every instant, and add up the area under the curve ...", 13, "bf-t-strong")
    # bottom: each job's residence as a bar
    by0 = 200
    text(o, 20, by0 - 14, "... or add up how long each job stayed. For a trace that starts and ends empty, the two totals are equal.", 13, "bf-t-strong")
    for i, (a, d) in enumerate(jobs):
        yy = by0 + i * 20
        o.append(f'<rect x="{x(a):.1f}" y="{yy}" width="{x(d) - x(a):.1f}" height="14" rx="3" style="fill:var(--bf-blue);opacity:0.55"/>')
        text(o, x(a) - 6, yy + 11, f"job {i + 1}", 11, anchor="end")
    o.append(f'<line class="bf-rule" x1="{L}" x2="{R0}" y1="{by0 + 104}" y2="{by0 + 104}"/>')
    text(o, (L + R0) / 2, by0 + 122, "time", 12, anchor="middle")
    text(o, W / 2, by0 + 146, "Divide both by the length of the trace: mean number present L = throughput X × mean residence R.", 12.5, anchor="middle")
    return svg(by0 + 158, o, "Little's law as two ways of counting the same quantity. Above, the number of jobs present "
               "over time, with the area under it shaded; below, each of five jobs' residence as a bar. The area "
               "equals the sum of the bars, so the mean number present equals throughput times mean residence. "
               "The jobs are illustrative.")


def migration():
    o = []
    rows = [("baseline", 100, 95), ("scanner −20%", 80, 95), ("workers −20%", 100, 76), ("both −20%", 80, 76)]
    L, top, rh = 150, 44, 50
    x = lambda v: L + 4.6 * v
    text(o, 20, 24, "Work a block needs from each station (illustrative numbers, not measurements)", 13, "bf-t-strong")
    for i, (name, s, w) in enumerate(rows):
        y = top + i * rh
        text(o, L - 12, y + 18, name, 13, "bf-t-strong", "end")
        o.append(f'<rect x="{L}" y="{y}" width="{x(s) - L:.1f}" height="14" style="fill:var(--bf-amber);opacity:0.75"/>')
        o.append(f'<rect x="{L}" y="{y + 18}" width="{x(w) - L:.1f}" height="14" style="fill:var(--bf-blue);opacity:0.75"/>')
        wall = max(s, w)
        o.append(f'<line x1="{x(wall):.1f}" x2="{x(wall):.1f}" y1="{y - 4}" y2="{y + 36}" style="stroke:var(--bf-ink);stroke-width:2"/>')
        text(o, x(wall) + 8, y + 22, f"wall ≈ {wall}", 13, "bf-t-strong")
    y = top + 4 * rh + 6
    o.append(f'<rect x="{L}" y="{y}" width="14" height="10" style="fill:var(--bf-amber);opacity:0.75"/>')
    text(o, L + 20, y + 10, "scanner", 12)
    o.append(f'<rect x="{L + 100}" y="{y}" width="14" height="10" style="fill:var(--bf-blue);opacity:0.75"/>')
    text(o, L + 120, y + 10, "workers", 12)
    text(o, L + 200, y + 10, "the wall follows the longer bar, when the stages overlap well", 12)
    return svg(y + 22, o, "Illustrative bottleneck migration. Baseline: scanner 100, workers 95, wall about 100. Cut the "
               "scanner 20%: 80 and 95, wall 95, so only 5 of the 20 saved reach the wall. Cut the workers 20%: 100 "
               "and 76, wall still 100. Cut both: 80 and 76, wall 80.")


def models():
    o = []
    rungs = [
        ("operational laws", "assume nothing about distributions: bounds, and identities a trace must satisfy",
         "held: nine identities in 840 decodes", "green"),
        ("product form and MVA", "assume a network the decoder only resembles: exact means for that network",
         "a surrogate within 1–4 ms of the wall at 3–8 workers", "steel"),
        ("trace-driven simulation", "keeps the decoder's own rules: the window, the byte cap, in-order taking",
         "proposed, not built", "amber"),
        ("intervention", "slow, pin or supply one part, and see whether the prediction moves with it",
         "stage 2 and the ideal-parts matrix", "blue"),
    ]
    top, rh = 34, 62
    text(o, 20, 22, "Each rung assumes more, or tests more; the measurements decide between them", 13, "bf-t-strong")
    for i, (name, what, status, c) in enumerate(rungs):
        y = top + i * rh
        box(o, 20, y, 680, rh - 12, f"var(--bf-{c}-tint)", f"var(--bf-{c})")
        text(o, 36, y + 20, name, 14, "bf-t-strong")
        text(o, 36, y + 38, what, 12)
        text(o, 684, y + 20, status, 12, "bf-t-strong", "end", f";fill:var(--bf-{c})")
    return svg(top + 4 * rh - 4, o, "A ladder of four kinds of model. Operational laws assume nothing about distributions "
               "and held in all 840 decodes. Product-form networks and mean-value analysis give exact means for a "
               "surrogate network, which came within 1 to 4 ms of the measured wall. A trace-driven simulation would "
               "keep the decoder's own rules; it is proposed, not built. Interventions test the predictions.")


SLICE = [  # workers, S alone, E alone, both (floor); S with E ideal; E with S ideal
    (1, 1.001, 0.827, 0.820),
    (6, 0.983, 0.849, 0.817),
    (8, 0.964, 0.960, 0.827),
    (10, 0.919, 0.975, 0.838),
]


def slicea():
    o = []
    L, R0, top, bot = 70, 700, 40, 250
    lo, hi = 0.75, 1.02
    y = lambda v: top + (bot - top) * (hi - v) / (hi - lo)
    for v in (0.8, 0.85, 0.9, 0.95, 1.0):
        o.append(f'<line class="bf-rule" x1="{L}" x2="{R0}" y1="{y(v):.1f}" y2="{y(v):.1f}" opacity="{0.9 if v == 1.0 else 0.35}"/>')
        text(o, L - 8, y(v) + 4, f"{v:.2f}", 11.5, anchor="end")
    gw = (R0 - L) / len(SLICE)
    for i, (k, s, e, both) in enumerate(SLICE):
        gx = L + gw * i + 22
        for j, (v, c) in enumerate(((s, "amber"), (e, "blue"), (both, "green"))):
            bx = gx + j * 36
            o.append(f'<rect x="{bx}" y="{y(1.0):.1f}" width="28" height="{y(v) - y(1.0):.1f}" style="fill:var(--bf-{c});opacity:0.8"/>')
            text(o, bx + 14, y(v) + 14, f"{v:.3f}".lstrip("0"), 10.5, anchor="middle")
        text(o, gx + 50, bot + 18, f"{k} worker{'s' if k > 1 else ''}", 12.5, "bf-t-strong", "middle")
    text(o, 20, 22, "Wall with a part made ideal, over everything real (pinned scanner; bars hang down from 1)", 13, "bf-t-strong")
    lx = L + 10
    for j, (name, c) in enumerate((("the scanner ideal", "amber"), ("emission ideal", "blue"), ("both ideal: the floor", "green"))):
        o.append(f'<rect x="{lx + j * 190}" y="{bot + 34}" width="14" height="10" style="fill:var(--bf-{c});opacity:0.8"/>')
        text(o, lx + j * 190 + 20, bot + 44, name, 12)
    return svg(bot + 54, o, "Ideal-parts slice A. With emission ideal the wall falls to 0.827 and 0.849 at 1 and 6 "
               "workers, where the scanner ideal does nothing; at 8 each alone is about 0.96 but both together 0.827; "
               "at 10 the scanner ideal gives 0.919 and emission 0.975. With both ideal the wall is still 0.82 to 0.84 "
               "at every count.")


def organisations():
    o = []
    rows = [("Today", [("serial Huffman scan", "amber"), ("scanned blocks", "frame"), ("parallel inversion", "blue"), ("in-order output", "green")]),
            ("A question", [("find block starts", "amber"), ("block queue", "frame"), ("parallel scan|and inversion", "blue"), ("in-order output", "green")])]
    for r, (name, steps) in enumerate(rows):
        y = 40 + r * 96
        text(o, 20, y - 8, name, 13, "bf-t-strong")
        bw = 150
        for i, (lab, c) in enumerate(steps):
            x = 20 + i * (bw + 26)
            fill = "var(--bf-surface)" if c == "frame" else f"var(--bf-{c}-tint)"
            stroke = "var(--bf-frame)" if c == "frame" else f"var(--bf-{c})"
            box(o, x, y, bw, 44, fill, stroke)
            parts = lab.split("|")
            for j, part in enumerate(parts):
                text(o, x + bw / 2, y + 27 + 15 * (j - (len(parts) - 1) / 2), part, 12.5, "bf-t-strong", "middle")
            if i:
                arrow(o, x - 24, y + 22, x - 3, y + 22)
    text(o, W / 2, 222, "The second row moves the scan behind the parallel boundary. Whether it pays depends on what finding", 12.5, anchor="middle")
    text(o, W / 2, 240, "the starts costs, what state must be carried across, and how the new demands compete for the same cores.", 12.5, anchor="middle")
    return svg(252, o, "Two organisations of the decoder. Today: a serial Huffman scan feeds scanned blocks to parallel "
               "inversion, then in-order output. A question: find block starts first, then let workers both scan and "
               "invert in parallel, then in-order output.")


if __name__ == "__main__":
    print(globals()[sys.argv[1]]())
