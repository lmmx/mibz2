#!/usr/bin/env python3
"""Seeding aids for a new specimen spec.

    seed_grid.py grid SRC OUT.png        draw a labelled pixel grid over SRC (2x), to read positions off
    seed_grid.py check SRC SEEDS.json    SEEDS.json: {"component": [[x_px, y_px], ...]}; snaps each seed
                                         off gold/ground/ink onto the creature and prints them in viewBox
                                         units (u = px * 300 / max(w, h)), ready to paste into the spec
"""
import json, sys
from PIL import Image, ImageDraw

def grid(src, out):
    im = Image.open(src).convert("RGB"); w, h = im.size; s = 2 if max(w, h) < 900 else 1
    im = im.resize((w * s, h * s)); d = ImageDraw.Draw(im); step = 25 if max(w, h) < 900 else 50
    for x in range(0, w, step):
        d.line([(x * s, 0), (x * s, h * s)], fill=(255, 0, 255) if x % (step * 4) == 0 else (0, 255, 255), width=1)
        if x % (step * 2) == 0: d.text((x * s + 2, 2), str(x), fill=(0, 0, 0))
    for y in range(0, h, step):
        d.line([(0, y * s), (w * s, y * s)], fill=(255, 0, 255) if y % (step * 4) == 0 else (0, 255, 255), width=1)
        if y % (step * 2) == 0: d.text((2, y * s + 2), str(y), fill=(0, 0, 0))
    im.save(out)

def check(src, seeds):
    im = Image.open(src).convert("RGB"); w, h = im.size; K = 300 / max(w, h)
    def bad(p):
        if not (0 <= p[0] < w and 0 <= p[1] < h): return True
        r, g, b = im.getpixel(p)
        gold = r > g + 7 and g - b > 33 and b < 116 and r - g < 65
        vellum = r > 200 and g > 185 and b > 150 and max(r, g, b) - min(r, g, b) < 45
        return gold or vellum or (r + g + b) < 150
    def snap(p):
        if not bad(p): return p
        for rad in range(1, 20):
            for dx in range(-rad, rad + 1):
                for q in ((p[0] + dx, p[1] - rad), (p[0] + dx, p[1] + rad), (p[0] - rad, p[1] + dx), (p[0] + rad, p[1] + dx)):
                    if not bad(q): return q
        return None
    out = {}
    for k, v in json.load(open(seeds)).items():
        pts = [snap(tuple(p)) for p in v]
        miss = [p for p, q in zip(v, pts) if q is None]
        if miss: print(f"# {k}: no creature pixel near {miss}", file=sys.stderr)
        out[k] = [[round(x * K, 1), round(y * K, 1)] for x, y in (q for q in pts if q)]
    print(json.dumps(out))

if __name__ == "__main__":
    {"grid": grid, "check": check}[sys.argv[1]](*sys.argv[2:4])
