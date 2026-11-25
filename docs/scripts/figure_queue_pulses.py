"""The queue as a reservoir, for "The decoder as a queueing network".

    python3 docs/scripts/figure_queue_pulses.py

Prints one inline SVG. Stage 2a's pulses (journal §74 at 6383f54f, character-stage2/report.txt):
at 8 workers, slowing the scanner by 50% over places 600-800 drops the job queue's mean depth from
4.4 to 0.3, with the rate 0.69 of the undosed run's at the same places; slowing emission by 50% raises
it from 3.9 to 8.9, with the rate at 0.82 (the report's "effect" column, rounded). Each tank shows the depth before (dashed) and during (filled).
"""

PANELS = [  # title, slowed, before, during, rate, which arrow is slowed, gloss
    ("Slow the scanner", "the scanner, +50%", 4.4, 0.3, 0.69, "in", "the workers are starved"),
    ("Slow emission", "the workers' emission, +50%", 3.9, 8.9, 0.82, "out", "work piles up"),
]
DMAX = 10


def main():
    W, H = 720, 344
    o = []
    a = o.append
    for i, (title, slowed, before, during, rate, which, gloss) in enumerate(PANELS):
        cx = 180 + 360 * i
        a(f'<text class="bf-t-strong" x="{cx}" y="22" text-anchor="middle" style="font-size:15px">{title}</text>')
        a(f'<text class="bf-tm" x="{cx}" y="40" text-anchor="middle" style="font-size:12px">slowed: {slowed}</text>')
        # arrows: in from the scanner, out to the workers; the slowed one is thin and amber
        for kind, y1, y2, label in (("in", 52, 92, "scanner"), ("out", 236, 276, "workers")):
            slow = kind == which
            colour = "var(--bf-amber)" if slow else "var(--bf-frame)"
            width = 1.5 if slow else 3
            a(f'<line x1="{cx}" x2="{cx}" y1="{y1}" y2="{y2 - 8}" style="stroke:{colour};stroke-width:{width}{";stroke-dasharray:4 3" if slow else ""}"/>')
            a(f'<path d="M{cx - 6},{y2 - 9} l6,9 l6,-9 z" style="fill:{colour}"/>')
            a(f'<text class="bf-tm" x="{cx + 14}" y="{(y1 + y2) / 2 + 4}" style="font-size:12px">{"in, from the " if kind == "in" else "out, to the "}{label}{" (slowed)" if slow else ""}</text>')
        # the tank
        tx, ty, tw, th = cx - 80, 96, 160, 136
        level = lambda d: ty + th * (1 - d / DMAX)
        a(f'<rect x="{tx + 2}" y="{level(during):.1f}" width="{tw - 4}" height="{ty + th - level(during):.1f}" style="fill:var(--bf-blue);opacity:0.55"/>')
        a(f'<path d="M{tx},{ty} L{tx},{ty + th} L{tx + tw},{ty + th} L{tx + tw},{ty}" style="fill:none;stroke:var(--bf-ink);stroke-width:2"/>')
        a(f'<line x1="{tx}" x2="{tx + tw}" y1="{level(before):.1f}" y2="{level(before):.1f}" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:4 3"/>')
        a(f'<text class="bf-tm" x="{tx - 6}" y="{level(before) + 4:.1f}" text-anchor="end" style="font-size:11.5px">before</text>')
        a(f'<text class="bf-t-strong" x="{cx}" y="{298}" text-anchor="middle" style="font-size:16px">{before} → {during} blocks</text>')
        a(f'<text class="bf-tm" x="{cx}" y="{318}" text-anchor="middle" style="font-size:12px">mean queue depth: {gloss}</text>')
        a(f'<text class="bf-tm" x="{cx}" y="{334}" text-anchor="middle" style="font-size:12px">rate {rate:.2f} of the undosed run’s</text>')
    aria = ("Two reservoirs, the job queue at 8 workers. Slowing the scanner by half drains it: mean "
            "depth 4.4 blocks before, 0.3 during, and the workers starve, the rate falling to 0.69 of the undosed run's. "
            "Slowing emission by half fills it: 3.9 before, 8.9 during, the rate 0.82 of the undosed run's.")
    print(f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>")


if __name__ == "__main__":
    main()
