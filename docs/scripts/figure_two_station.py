"""The interactive two-station bound for "The decoder as a queueing network".

    python3 docs/scripts/figure_two_station.py

Prints a <figure class="bf-fig bf-twostation"> to paste into the post. Its SVG is drawn here at
the measured service times, so the figure reads correctly without JavaScript; blog.js
(.bf-twostation) shows the two sliders and redraws the bound, the knee and the read-outs when
they move. The measured points never move.

The model is §57's: a scanner taking s a block and k workers each taking w, so throughput is at
most X(k) = min(k/w, 1/s) blocks a second, and the knee is at k* = w/s. s = 106.6 µs and
w = 798.3 µs come from counts 1-4. The measured points are 1,724 blocks over the median wall at
each count (figure_decoder_knee.py's ROWS, from measurements-scan-profile.tsv at 61f96093).
"""
from figure_decoder_knee import ROWS

BLOCKS = 1724
S, WK = 106.6, 798.3  # µs a block
W, H, L, R, T, B = 720, 360, 70, 20, 24, 56
YMAX = 20000


def x(k):
    return L + (W - L - R) * (k - 1) / 19


def y(v):
    return T + (H - T - B) * (1 - min(v, YMAX) / YMAX)


def main():
    o = []
    a = o.append
    a(f'<defs><clipPath id="ts-clip"><rect x="{L}" y="{T}" width="{W - L - R}" height="{H - T - B}"/></clipPath></defs>')
    for v in range(0, YMAX + 1, 5000):
        a(f'<line class="bf-rule" x1="{L}" x2="{W - R}" y1="{y(v):.1f}" y2="{y(v):.1f}" opacity="{0.9 if v == 0 else 0.35}"/>')
        a(f'<text class="bf-tm" x="{L - 8}" y="{y(v) + 4:.1f}" text-anchor="end" style="font-size:12px">{v // 1000}k</text>')
    for k in (1, 4, 8, 12, 16, 20):
        a(f'<text class="bf-tm" x="{x(k):.1f}" y="{H - B + 18}" text-anchor="middle" style="font-size:12px">{k}</text>')
    a(f'<text class="bf-tm" x="{(L + W - R) / 2:.0f}" y="{H - B + 38}" text-anchor="middle" style="font-size:12px">workers, k</text>')
    a(f'<text class="bf-tm" x="16" y="{(T + H - B) / 2:.0f}" text-anchor="middle" transform="rotate(-90 16 {(T + H - B) / 2:.0f})" style="font-size:12px">blocks a second</text>')
    ks = [1 + i * 0.1 for i in range(191)]
    work = " ".join(f"{x(k):.1f},{y(k / WK * 1e6):.1f}" for k in ks)
    bound = " ".join(f"{x(k):.1f},{y(min(k / WK, 1 / S) * 1e6):.1f}" for k in ks)
    kst = WK / S
    a(f'<g clip-path="url(#ts-clip)">')
    a(f'<polyline class="bf-ts-work" points="{work}" style="fill:none;stroke:var(--bf-blue);stroke-width:1.5;stroke-dasharray:5 4"/>')
    a(f'<line class="bf-ts-scan" x1="{L}" x2="{W - R}" y1="{y(1e6 / S):.1f}" y2="{y(1e6 / S):.1f}" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/>')
    a(f'<polyline class="bf-ts-bound" points="{bound}" style="fill:none;stroke:var(--bf-steel);stroke-width:3"/>')
    a(f'<line class="bf-ts-kline" x1="{x(kst):.1f}" x2="{x(kst):.1f}" y1="{y(1e6 / S):.1f}" y2="{H - B}" style="stroke:var(--bf-steel);stroke-width:1;stroke-dasharray:3 3"/>')
    a(f'<circle class="bf-ts-knee" cx="{x(kst):.1f}" cy="{y(1e6 / S):.1f}" r="5" style="fill:var(--bf-steel)"/>')
    a('</g>')
    for k, wall, *_ in ROWS:
        a(f'<circle cx="{x(k):.1f}" cy="{y(BLOCKS / wall):.1f}" r="3.5" style="fill:var(--bf-green)"/>')
    a(f'<text class="bf-t-strong bf-ts-kstar" x="{W - R}" y="{T + 16}" text-anchor="end" style="font-size:15px">knee k* = w/s = {kst:.2f} workers</text>')
    a(f'<text class="bf-tm bf-ts-ceil" x="{W - R}" y="{T + 36}" text-anchor="end" style="font-size:13px">scanner ceiling 1/s = {1e6 / S:,.0f} blocks a second</text>')
    lx, ly = x(10.2), y(5600)  # below the measured points' plateau, clear of both limits
    a(f'<rect x="{lx - 8}" y="{ly - 12}" width="{W - R - lx + 4}" height="76" rx="4" style="fill:var(--bf-surface);opacity:0.9"/>')
    for i, (colour, dash, text) in enumerate([("var(--bf-steel)", "", "the bound, min(k/w, 1/s)"),
                                              ("var(--bf-blue)", "5 4", "the workers' limit, k/w"),
                                              ("var(--bf-amber)", "5 4", "the scanner's limit, 1/s")]):
        yy = ly + 18 * i
        a(f'<line x1="{lx}" x2="{lx + 22}" y1="{yy}" y2="{yy}" style="stroke:{colour};stroke-width:{3 if i == 0 else 1.5}{";stroke-dasharray:" + dash if dash else ""}"/>')
        a(f'<text class="bf-tm" x="{lx + 30}" y="{yy + 4}" style="font-size:12.5px">{text}</text>')
    yy = ly + 54
    a(f'<circle cx="{lx + 11}" cy="{yy}" r="3.5" style="fill:var(--bf-green)"/>')
    a(f'<text class="bf-tm" x="{lx + 30}" y="{yy + 4}" style="font-size:12.5px">measured: 1,724 blocks over the median wall</text>')
    aria = ("The two-station bound on throughput against workers from 1 to 20: the workers' limit k/w "
            "rises in a straight line until it meets the scanner's ceiling 1/s at the knee, 7.49 "
            "workers with the measured service times, 106.6 µs for the scanner and 798.3 µs for a "
            "worker; the measured decoder follows the bound closely, just below it, peaking at about "
            "9,000 blocks a second at 8 workers.")
    svg = f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"
    controls = (
        '<div class="bf-ts-controls" hidden>'
        f'<label>scanner, s <input class="bf-ts-s" type="range" min="50" max="250" step="0.1" value="{S}" aria-label="scanner service time, microseconds a block"> <output class="bf-ts-sv">{S} µs</output></label>'
        f'<label>worker, w <input class="bf-ts-w" type="range" min="200" max="1600" step="0.1" value="{WK}" aria-label="worker service time, microseconds a block"> <output class="bf-ts-wv">{WK} µs</output></label>'
        '<button type="button" class="bf-ts-reset">Reset to the measured values</button>'
        '</div>'
    )
    print(f'<figure class="bf-fig bf-twostation" id="two-station" data-x0="{L}" data-x1="{W - R}" '
          f'data-y0="{T}" data-y1="{H - B}" data-ymax="{YMAX}">\n'
          '<p class="bf-title">The bottleneck law, with the measured decoder</p>\n'
          f'{svg}\n{controls}\n'
          '<figcaption>The idealised bound from the operational laws, with no start-up, drain, '
          'contention or blocking, against the decoder measured on an i9-10900K (median of 20 '
          'decodes at each count). With JavaScript, the sliders set the two service times: a '
          'cheaper scanner raises the ceiling and moves the knee right, a cheaper worker steepens '
          'the rise and moves it left.</figcaption>\n</figure>')


if __name__ == "__main__":
    main()
