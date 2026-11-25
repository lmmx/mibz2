"""Figures for "The decoder as a queueing network", from the scanner profile.

    python3 docs/scripts/figure_decoder_knee.py wall|scanner

Data: math-targets/data/measurements-scan-profile.tsv on docs/journal-algorithm-level at 61f96093
(bundle scan-profile/run-20261124T223748Z), journal §57. Columns kept: workers, untimed median
wall (s), scanner work (s), scanner slot wait (s). The model is §57's: service times from counts
1-4 (scanner 106.6 µs and worker 798.3 µs a block, so a predicted crossover at 7.49), and the fit
wall(k) = c + max(b, a/k) over all counts (a = 1.279 s, b = 0.178 s, c = 0.032 s).
"""
import sys

ROWS = [  # workers, wall_untimed_median_s, scanner_work_s, scanner_slot_wait_s
    (1, 1.3038, 0.1738, 1.1195),
    (2, 0.6829, 0.1819, 0.4983),
    (3, 0.4693, 0.1861, 0.2763),
    (4, 0.3550, 0.1857, 0.1638),
    (5, 0.2840, 0.1854, 0.0925),
    (6, 0.2381, 0.1853, 0.0471),
    (7, 0.2069, 0.1860, 0.0151),
    (8, 0.1911, 0.1862, 0.0006),
    (9, 0.1971, 0.1883, 0.0036),
    (10, 0.2012, 0.1890, 0.0029),
    (11, 0.1967, 0.1899, 0.0015),
    (12, 0.1974, 0.1905, 0.0007),
    (13, 0.1966, 0.1913, 0.0006),
    (14, 0.1978, 0.1916, 0.0002),
    (15, 0.2114, 0.1924, 0.0001),
    (16, 0.2163, 0.2295, 0.0001),
    (17, 0.2332, 0.2258, 0.0001),
    (18, 0.2330, 0.2279, 0.0001),
    (19, 0.2319, 0.2265, 0.0001),
    (20, 0.2303, 0.2255, 0.0001),
]
A, B, C = 1.279, 0.178, 0.032
K_PRED = 798.3 / 106.6


def frame(o, W, H, L, R, T, Bm, ymax, yticks, ylabel):
    y = lambda v: T + (H - T - Bm) * (1 - v / ymax)
    x = lambda k: L + (W - L - R) * (k - 1) / 19
    for v in yticks:
        o.append(f'<line x1="{L}" x2="{W-R}" y1="{y(v):.1f}" y2="{y(v):.1f}" style="stroke:var(--bf-frame);stroke-width:1;stroke-dasharray:3 3"/>')
        o.append(f'<text class="bf-tm" x="{L-8}" y="{y(v)+4:.1f}" text-anchor="end" style="font-size:12px">{v:g}</text>')
    for k in (1, 4, 8, 12, 16, 20):
        o.append(f'<text class="bf-tm" x="{x(k):.1f}" y="{H-Bm+18}" text-anchor="middle" style="font-size:12px">{k}</text>')
    o.append(f'<text class="bf-tm" x="{(L+W-R)/2:.0f}" y="{H-Bm+36}" text-anchor="middle" style="font-size:12px">workers</text>')
    o.append(f'<text class="bf-tm" x="16" y="{(T+H-Bm)/2:.0f}" text-anchor="middle" transform="rotate(-90 16 {(T+H-Bm)/2:.0f})" style="font-size:12px">{ylabel}</text>')
    return x, y


def wall():
    W, H, L, R, T, Bm = 720, 400, 64, 24, 20, 74
    o = []
    x, y = frame(o, W, H, L, R, T, Bm, 0.7, [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7], "decode wall time (s)")
    clip = lambda v: min(v, 0.7)
    pts = []
    k = 1.0
    while k <= 20.001:
        pts.append(f"{x(k):.1f},{y(clip(C + max(B, A / k))):.1f}")
        k += 0.1
    o.append(f'<polyline points="{" ".join(pts)}" style="fill:none;stroke:var(--bf-steel);stroke-width:2"/>')
    o.append(f'<line x1="{x(K_PRED):.1f}" x2="{x(K_PRED):.1f}" y1="{T}" y2="{H-Bm}" style="stroke:var(--bf-amber);stroke-width:1.5;stroke-dasharray:5 4"/>')
    o.append(f'<text class="bf-t-strong" x="{x(K_PRED)+6:.1f}" y="{T+16}" style="font-size:12px;fill:var(--bf-amber)">knee predicted from counts 1–4: 7.49</text>')
    o.append(f'<text class="bf-tm" x="{x(K_PRED)+6:.1f}" y="{T+32}" style="font-size:12px">measured crossover: 7.67</text>')
    for kk, wl, sw, sl in ROWS:
        if wl <= 0.7:
            o.append(f'<circle cx="{x(kk):.1f}" cy="{y(wl):.1f}" r="4.5" style="fill:var(--bf-green)"/>')
        else:
            o.append(f'<path d="M{x(kk):.1f},{T+2} l-5,8 h10 z" style="fill:var(--bf-green)"/>')
            o.append(f'<text class="bf-tm" x="{x(kk)+8:.1f}" y="{T+10}" style="font-size:11px">{wl:.2f} s</text>')
    o.append(f'<text class="bf-tm" x="{x(3.2):.1f}" y="{y(0.45):.1f}" style="font-size:12px">workers bind: wall ≈ a/k</text>')
    o.append(f'<text class="bf-tm" x="{x(11):.1f}" y="{y(0.26):.1f}" style="font-size:12px">scanner binds: wall ≈ b</text>')
    o.append(f'<text class="bf-tm" x="{x(15.6):.1f}" y="{y(0.30):.1f}" style="font-size:12px">step at 16: the scanner slows 23%</text>')
    o.append(f'<circle cx="{L+10}" cy="{H-12}" r="4.5" style="fill:var(--bf-green)"/><text class="bf-tm" x="{L+20}" y="{H-8}" style="font-size:12px">measured (median of 20 decodes)</text>')
    o.append(f'<line x1="{L+260}" x2="{L+290}" y1="{H-12}" y2="{H-12}" style="stroke:var(--bf-steel);stroke-width:2"/><text class="bf-tm" x="{L+298}" y="{H-8}" style="font-size:12px">fitted two-station model, c + max(b, a/k)</text>')
    aria = ("Decode wall time against workers from 1 to 20 on the i9-10900K. It falls as a over k, from 1.30 s at one worker, "
            "to a best of 0.191 s at 8 workers, stays near 0.197 s from 8 to 14, and steps up to about 0.23 s from 16. "
            "The knee predicted from counts 1 to 4 alone is 7.49 workers; the measured crossover is 7.67.")
    return f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"


def scanner():
    W, H, L, R, T, Bm = 720, 350, 64, 24, 20, 74
    o = []
    x, y = frame(o, W, H, L, R, T, Bm, 1.4, [0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4], "scanner thread time (s)")
    bw = (W - L - R) / 20 * 0.62
    for kk, wl, sw, sl in ROWS:
        cx = x(kk)
        o.append(f'<rect x="{cx-bw/2:.1f}" y="{y(sw):.1f}" width="{bw:.1f}" height="{y(0)-y(sw):.1f}" style="fill:var(--bf-amber)"/>')
        if sl > 0.0005:
            o.append(f'<rect x="{cx-bw/2:.1f}" y="{y(sw+sl):.1f}" width="{bw:.1f}" height="{y(sw)-y(sw+sl):.1f}" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:0.8"/>')
    o.append(f'<text class="bf-tm" x="{x(9.5):.1f}" y="{y(0.62):.1f}" style="font-size:12px">from 8 workers the scanner almost never waits:</text>')
    o.append(f'<text class="bf-tm" x="{x(9.5):.1f}" y="{y(0.62)+16:.1f}" style="font-size:12px">it is the bottleneck, and only less work there helps</text>')
    o.append(f'<rect x="{L+4}" y="{H-20}" width="12" height="12" style="fill:var(--bf-amber)"/><text class="bf-tm" x="{L+22}" y="{H-9}" style="font-size:12px">scanner working</text>')
    o.append(f'<rect x="{L+170}" y="{H-20}" width="12" height="12" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel)"/><text class="bf-tm" x="{L+188}" y="{H-9}" style="font-size:12px">scanner waiting for a free slot</text>')
    aria = ("The scanner thread's time against workers. Its work stays near 0.19 s from 3 to 15 workers and rises to about "
            "0.23 s from 16. Its waiting for a free slot falls from 1.12 s at one worker to under a millisecond at 8 and beyond.")
    return f'<svg viewBox="0 0 {W} {H}" role="img" aria-label="{aria}">' + "".join(o) + "</svg>"


if __name__ == "__main__":
    print({"wall": wall, "scanner": scanner}[sys.argv[1]]())
