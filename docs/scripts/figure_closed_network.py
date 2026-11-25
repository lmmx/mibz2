"""The decoder as a closed queueing network, for "Posing the inversion problem".

    python3 docs/scripts/figure_closed_network.py [--qualified]

--qualified, for "The decoder as a queueing network", replaces the note and the description's
opening, which call the decoder a closed network, with the qualified reading: admission credit
circulates, but the decoder is a finite pipeline with feedback-controlled admission (the course
mining's correction, reports/2026-11-25-course-mining.md).

Prints one inline SVG in the blog's figure style. The structure is the measurement plan's
(briefs/character.md §2 at a4bef380): a feeder and a serial scanner, an admission window of
W = 4k blocks in flight, k workers, and an ordered consumer whose progress returns admission
credit, with a 12 MiB threshold on finished output.
"""


def node(o, x, y, w, h, fill, stroke, title, lines):
    o.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" style="fill:{fill};stroke:{stroke};stroke-width:1.5"/>')
    o.append(f'<text class="bf-t-strong" x="{x+w/2}" y="{y+22}" text-anchor="middle" style="font-size:13px">{title}</text>')
    for i, t in enumerate(lines):
        o.append(f'<text class="bf-tm" x="{x+w/2}" y="{y+40+i*15}" text-anchor="middle" style="font-size:11px">{t}</text>')


def arrow(o, x1, y1, x2, y2, colour="var(--bf-frame)", dash=False):
    o.append(f'<path d="M{x1},{y1} L{x2},{y2}" style="stroke:{colour};stroke-width:1.8;fill:none{";stroke-dasharray:5 4" if dash else ""}"/>')


def head(o, x, y, d, colour="var(--bf-frame)"):
    pts = {"r": f"{x},{y} {x-8},{y-5} {x-8},{y+5}", "l": f"{x},{y} {x+8},{y-5} {x+8},{y+5}",
           "u": f"{x},{y} {x-5},{y+8} {x+5},{y+8}", "d": f"{x},{y} {x-5},{y-8} {x+5},{y-8}"}[d]
    o.append(f'<polygon points="{pts}" style="fill:{colour}"/>')


def main(qualified=False):
    W, H = 720, 300
    o = []
    y0, h = 40, 92
    node(o, 10, y0, 96, h, "var(--bf-surface)", "var(--bf-frame)", "feeder", ["reads and", "copies input"])
    node(o, 128, y0, 122, h, "var(--bf-amber-tint)", "var(--bf-amber)", "scanner", ["one thread:", "headers, tables,", "Huffman decode"])
    # the admission window: a box of slots
    wx, ww = 272, 170
    o.append(f'<rect x="{wx}" y="{y0}" width="{ww}" height="{h}" rx="8" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1.5"/>')
    o.append(f'<text class="bf-t-strong" x="{wx+ww/2}" y="{y0+22}" text-anchor="middle" style="font-size:13px">admission window</text>')
    o.append(f'<text class="bf-tm" x="{wx+ww/2}" y="{y0+40}" text-anchor="middle" style="font-size:11px">W = 4k blocks in flight:</text>')
    o.append(f'<text class="bf-tm" x="{wx+ww/2}" y="{y0+55}" text-anchor="middle" style="font-size:11px">queued, running, or done</text>')
    o.append(f'<text class="bf-tm" x="{wx+ww/2}" y="{y0+70}" text-anchor="middle" style="font-size:11px">but not yet taken in order</text>')
    node(o, 464, y0, 122, h, "var(--bf-green-tint)", "var(--bf-green)", "k workers", ["MTF and runs,", "inversion,", "emission, CRC"])
    node(o, 608, y0, 102, h, "var(--bf-surface)", "var(--bf-frame)", "consumer", ["takes blocks", "in order,", "writes output"])
    for x1, x2 in [(106, 128), (250, 272), (442, 464), (586, 608)]:
        arrow(o, x1, y0 + h / 2, x2 - 8, y0 + h / 2)
        head(o, x2, y0 + h / 2, "r")
    # feedback: credit returns from the consumer to admission
    yb = y0 + h + 52
    arrow(o, 659, y0 + h, 659, yb, "var(--bf-steel)", True)
    arrow(o, 659, yb, 357, yb, "var(--bf-steel)", True)
    arrow(o, 357, yb, 357, y0 + h + 9, "var(--bf-steel)", True)
    head(o, 357, y0 + h + 1, "u", "var(--bf-steel)")
    o.append(f'<text class="bf-tm" x="508" y="{yb+18}" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">a slot is freed only when the next block in order is taken,</text>')
    o.append(f'<text class="bf-tm" x="508" y="{yb+33}" text-anchor="middle" style="font-size:11.5px;fill:var(--bf-steel)">and finished output is capped at 12 MiB</text>')
    notes = (("Admission credit circulates, though each block passes through once: close to a closed network,",
              "not one. One slow early block can hold slots and stall the scanner.") if qualified else
             ("A fixed population of blocks circulates, so the decoder is a closed network: one slow early block can hold slots",
              "and stall the scanner, so a waiting scanner does not by itself show that the workers are too few."))
    for i, line in enumerate(notes):
        o.append(f'<text class="bf-tm" x="10" y="{yb+60+16*i}" style="font-size:11.5px">{line}</text>')
    aria = (("The decoder as a network of queues, close to a closed one. " if qualified else
             "The decoder as a closed queueing network. ") + "A feeder passes input to a single scanner thread, which "
            "decodes headers, tables and Huffman codes. Blocks enter an admission window holding W = 4k blocks in "
            "flight, then k workers undo move-to-front and runs, invert and emit, then a consumer takes blocks in "
            "order and writes the output. A slot returns to the window only when the next block in order is taken, "
            "and finished output is capped at 12 MiB, so one slow early block can stall the scanner.")
    print(f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>")


if __name__ == "__main__":
    import sys
    main("--qualified" in sys.argv[1:])
