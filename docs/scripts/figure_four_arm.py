"""Figures for the four-arm post (journal §44 at 2f034506; bundle four-arm/run-20261124T204154Z).

    python3 docs/scripts/figure_four_arm.py factory|results

Prints one inline SVG in the blog's figure style. The ratios are §44's, against arm A, with
95% intervals.
"""
import sys

RES = {  # workers: arm: (ratio, lo, hi)
    1: {"B": (0.830, 0.825, 0.834), "C": (0.996, 0.991, 1.001), "D": (0.825, 0.820, 0.831)},
    6: {"B": (0.850, 0.844, 0.855), "C": (0.982, 0.979, 0.986), "D": (0.830, 0.827, 0.834)},
    10: {"B": (0.976, 0.936, 1.017), "C": (0.806, 0.782, 0.831), "D": (0.741, 0.717, 0.766)},
}
COL = {"B": "var(--bf-steel)", "C": "var(--bf-amber)", "D": "var(--bf-green)"}
NAME = {"B": "B: codes supplied", "C": "C: scanner supplied", "D": "D: both supplied"}


def results():
    W, H, L, R, T, Bm = 720, 380, 70, 24, 24, 78
    lo, hi = 0.70, 1.05
    y = lambda v: T + (H - T - Bm) * (hi - v) / (hi - lo)
    o = []
    a = o.append
    v = 0.70
    while v <= 1.0501:
        a(f'<line x1="{L}" x2="{W-R}" y1="{y(v):.1f}" y2="{y(v):.1f}" style="stroke:var(--bf-frame);stroke-width:{2 if abs(v-1)<1e-9 else 1};{"" if abs(v-1)<1e-9 else "stroke-dasharray:3 3"}"/>')
        a(f'<text class="bf-tm" x="{L-8}" y="{y(v)+4:.1f}" text-anchor="end" style="font-size:13px">{v:.2f}</text>')
        v = round(v + 0.05, 2)
    a(f'<text class="bf-tm" x="{L+6}" y="{y(1.0)-6:.1f}" style="font-size:12px">as shipped (arm A)</text>')
    a(f'<text class="bf-tm" x="18" y="{(T+H-Bm)/2:.0f}" text-anchor="middle" transform="rotate(-90 18 {(T+H-Bm)/2:.0f})" style="font-size:12px">whole-decoder time ÷ arm A (lower is faster)</text>')
    groups = [1, 6, 10]
    gw = (W - L - R) / 3
    for gi, w in enumerate(groups):
        cx = L + gw * gi + gw / 2
        a(f'<text class="bf-ts" x="{cx:.0f}" y="{H-Bm+24}" text-anchor="middle" style="font-size:14px">{w} worker{"s" if w>1 else ""}</text>')
        for k, arm in enumerate("BCD"):
            r, l, h = RES[w][arm]
            px = cx + (k - 1) * 46
            a(f'<line x1="{px:.0f}" x2="{px:.0f}" y1="{y(h):.1f}" y2="{y(l):.1f}" style="stroke:{COL[arm]};stroke-width:3"/>')
            a(f'<circle cx="{px:.0f}" cy="{y(r):.1f}" r="6" style="fill:{COL[arm]}"/>')
            a(f'<text class="bf-t-strong" x="{px:.0f}" y="{y(l)+18:.1f}" text-anchor="middle" style="font-size:12px">{r:.3f}</text>')
    lx = L + 10
    for k, arm in enumerate("BCD"):
        x0 = lx + k * 210
        a(f'<circle cx="{x0}" cy="{H-22}" r="6" style="fill:{COL[arm]}"/>')
        a(f'<text class="bf-tm" x="{x0+12}" y="{H-18}" style="font-size:13px">{NAME[arm]}</text>')
    aria = ("Whole-decoder time against arm A at 1, 6 and 10 workers, with 95% intervals. At 1 worker: "
            "codes 0.83, scanner 1.00, both 0.83. At 6: 0.85, 0.98, 0.83. At 10: codes 0.98, scanner 0.81, both 0.74.")
    return f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"


def factory():
    W, H = 720, 330
    o = []
    a = o.append

    def row(y0, n, label, sub, busy):
        a(f'<text class="bf-t-strong" x="20" y="{y0}" style="font-size:14px">{label}</text>')
        a(f'<text class="bf-tm" x="20" y="{y0+17}" style="font-size:12px">{sub}</text>')
        yb = y0 + 34
        # the doorman: the scanner
        a(f'<rect x="20" y="{yb}" width="150" height="84" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1.5"/>')
        a(f'<text class="bf-t-strong" x="95" y="{yb+26}" text-anchor="middle" style="font-size:13px">the scanner</text>')
        a(f'<text class="bf-tm" x="95" y="{yb+44}" text-anchor="middle" style="font-size:11.5px">reads the bits, decodes</text>')
        a(f'<text class="bf-tm" x="95" y="{yb+60}" text-anchor="middle" style="font-size:11.5px">Huffman: one thread</text>')
        a(f'<text class="bf-tm" x="95" y="{yb+76}" text-anchor="middle" style="font-size:11px;fill:var(--bf-amber)">arm C supplies this</text>')
        # the conveyor of unpacked blocks
        a(f'<line x1="176" x2="372" y1="{yb+42}" y2="{yb+42}" style="stroke:var(--bf-frame);stroke-width:2"/>')
        a(f'<path d="M372,{yb+36} l9,6 l-9,6 z" style="fill:var(--bf-frame)"/>')
        q = 5 if busy else 0
        for i in range(q):
            a(f'<rect x="{350-i*30}" y="{yb+28}" width="22" height="14" rx="2" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/>')
        a(f'<text class="bf-tm" x="274" y="{yb+66}" text-anchor="middle" style="font-size:11.5px">{"blocks queue for the workers" if busy else "no blocks waiting"}</text>')
        # the workers
        cols = 5 if n > 1 else 1
        for i in range(n):
            cx = 400 + (i % cols) * 62
            cy = yb + (i // cols) * 44
            a(f'<rect x="{cx}" y="{cy}" width="54" height="36" rx="5" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1.3;{"" if busy else "opacity:0.55"}"/>')
            a(f'<text class="bf-tm" x="{cx+27}" y="{cy+22}" text-anchor="middle" style="font-size:11px">{"working" if busy else "waiting"}</text>')
        if n == 1:
            a(f'<text class="bf-tm" x="470" y="{yb+16}" style="font-size:11.5px">a worker: undoes MTF and RLE2,</text>')
            a(f'<text class="bf-tm" x="470" y="{yb+32}" style="font-size:11.5px">inverts the BWT, emits the text</text>')
            a(f'<text class="bf-tm" x="470" y="{yb+48}" style="font-size:11px;fill:var(--bf-steel)">arm B supplies its codes</text>')

    row(22, 1, "One worker: the worker is the bottleneck", "supplied codes save 17%; a supplied scanner saves nothing", True)
    row(178, 10, "Ten workers: everyone waits at the door", "a supplied scanner saves 19%; with it, supplied codes save 8% more", False)
    aria = ("The decoder as a small factory. One scanner thread reads the compressed bits and decodes each "
            "block's Huffman codes in order, and queues the blocks. Workers undo the move-to-front and run-length "
            "coding, invert the block and emit its text. With one worker, blocks queue at the worker; with ten "
            "workers, the workers wait for the scanner.")
    return f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"


if __name__ == "__main__":
    print({"factory": factory, "results": results}[sys.argv[1]]())
