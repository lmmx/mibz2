"""Animate traced manuscript creatures for their places in the Thomas Murphy post.

The traced masters (docs/art/bestiary/traced/*-master.svg, from art/bestiary-serpents) are
grouped by anatomical part, each in the master's own coordinates. This adds SMIL motion to the
parts in place (a head that turns and nods, a tail that sways, a wing that lifts), drops parts
the page doesn't want (foliage scrolls), crops to the creature, and writes one animated SVG per
creature to docs/art/bestiary/slots/.

    python3 docs/scripts/bestiary_slots.py
"""

import re
from pathlib import Path

TRACED = Path("docs/art/bestiary/traced")
OUT = Path("docs/art/bestiary/slots")


def rot(pivot, angles, dur, begin=0):
    """A rotation about a pivot, eased back and forth through the given angles."""
    x, y = pivot
    vals = ";".join(f"{a} {x} {y}" for a in angles)
    n = len(angles) - 1
    splines = ";".join(["0.45 0 0.55 1"] * n)
    return (f'<animateTransform attributeName="transform" type="rotate" values="{vals}" '
            f'dur="{dur}s" begin="{begin}s" repeatCount="indefinite" calcMode="spline" '
            f'keyTimes="{";".join(f"{i/n:.3f}" for i in range(n + 1))}" keySplines="{splines}"/>')


def bob(dy, dur):
    return (f'<animateTransform attributeName="transform" type="translate" values="0 0;0 {-dy};0 0" '
            f'dur="{dur}s" repeatCount="indefinite" calcMode="spline" keyTimes="0;0.5;1" '
            f'keySplines="0.45 0 0.55 1;0.45 0 0.55 1" additive="sum"/>')


def animate(svg, part, motion):
    """Insert a motion as the first child of the part's group, so the whole part moves."""
    tag = f'<g id="{part}">'
    assert svg.count(tag) == 1, part
    return svg.replace(tag, tag + motion, 1)


def drop(svg, part):
    """Remove a part's group entirely (groups hold paths only, no nested groups)."""
    s = svg.index(f'<g id="{part}">')
    e = svg.index("</g>", s) + 4
    assert "<g" not in svg[s + 3:e - 4], part
    return svg[:s] + svg[e:]


def finish(svg, viewbox, label, name, root):
    svg = re.sub(r'viewBox="[^"]*"', f'viewBox="{viewbox}"', svg, count=1)
    svg = re.sub(r'\s(width|height)="[\d.]+"', "", svg, count=2)
    svg = re.sub(r'aria-label="[^"]*"', f'aria-label="{label}"', svg, count=1)
    # a slow breath for the whole creature, so it never sits dead still
    svg = animate(svg, root, bob(1.6, 3.4))
    (OUT / f"{name}.svg").write_text(svg, encoding="utf-8")
    print(f"{name}.svg: {len(svg):,} bytes")


def main():
    OUT.mkdir(parents=True, exist_ok=True)

    # The Aberdeen Bestiary serpent wound through its pillar: the head turns and nods, the
    # shedding tail sways.
    s = (TRACED / "aberdeen-serpent-pillar-master.svg").read_text(encoding="utf-8")
    s = animate(s, "aberdeen-serpent-pillar-head", rot((168, 104), [0, -7, 4, -3, 0], 4.6))
    s = animate(s, "aberdeen-serpent-pillar-shedding-rear", rot((128, 192), [0, 3.5, -2.5, 0], 5.8))
    finish(s, "40 28 192 236", "A serpent from the Aberdeen Bestiary, wound through a pillar, turning its head",
           "aberdeen-serpent", "aberdeen-serpent-root")

    # The Romanesque S-initial dragon, out of its letter: no scrolls, the head nods, the wing lifts.
    s = (TRACED / "romanesque-s-initial-master.svg").read_text(encoding="utf-8")
    for part in ("romanesque-s-initial-upper-scroll", "romanesque-s-initial-lower-scroll",
                 "romanesque-s-initial-tail-foliage"):
        s = drop(s, part)
    s = animate(s, "romanesque-s-initial-dragon-head", rot((204, 122), [0, -6, 5, 0], 4.2))
    s = animate(s, "romanesque-s-initial-dragon-wing", rot((104, 140), [0, -5, 2, 0], 3.1, 0.4))
    finish(s, "20 0 244 300", "The dragon of a Romanesque S initial, nodding and lifting its wing",
           "s-initial-dragon", "romanesque-s-root")


if __name__ == "__main__":
    main()
