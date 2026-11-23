"""Re-trace a traced creature into clean, flat, smooth shapes.

A first trace of a manuscript photograph is faithful but noisy: specks, hairline cracks between
colour patches, ragged edges, and many near-identical shades. This renders the trace large,
cleans the image, and traces it again:
1. render the SVG at SCALE times its size on a transparent ground;
2. reduce it to at most K flat pigments (k-means over the opaque pixels), so near-identical
   shades merge;
3. per pigment, a majority (mode) filter, then morphological closing, so specks vanish and
   cracks close;
4. trace again with vtracer in spline mode with a large speckle filter, giving smooth curves.

The noisy trace is not kept: the output replaces it.

    uv run --no-project -p 3.13 --with vtracer --with opencv-python-headless --with numpy \
        --with playwright==1.56.0 python docs/scripts/bestiary_retrace.py CHROME IN.svg OUT.svg [K]
"""

import re
import sys
import tempfile
from pathlib import Path

import cv2
import numpy as np
import vtracer
from playwright.sync_api import sync_playwright

SCALE = 3


def render(chrome, svg_path, png_path):
    svg = Path(svg_path).read_text(encoding="utf-8")
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path=chrome)
        pg = b.new_page(device_scale_factor=SCALE)
        # first frame only: no animation mid-pose
        pg.set_content("<html><body style='margin:0;background:transparent'>"
                       "<style>svg{display:block;width:300px;height:auto}</style>" + svg + "</body></html>")
        pg.evaluate("document.querySelectorAll('svg').forEach(s => { try { s.pauseAnimations(); s.setCurrentTime(0); } catch (e) {} })")
        pg.locator("svg").first.screenshot(path=png_path, omit_background=True)
        b.close()


def clean(png_path, out_path, k):
    img = cv2.imread(png_path, cv2.IMREAD_UNCHANGED)
    bgr, alpha = img[..., :3], img[..., 3]
    mask = alpha > 128
    # close the silhouette itself: cracks to the background become filled
    mask = cv2.morphologyEx(mask.astype(np.uint8), cv2.MORPH_CLOSE, np.ones((9, 9), np.uint8)) > 0
    px = bgr[alpha > 128].astype(np.float32)
    crit = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 40, 0.5)
    _, labels, centres = cv2.kmeans(px, k, None, crit, 4, cv2.KMEANS_PP_CENTERS)
    # label every pixel by its nearest pigment (inside the closed silhouette)
    flat = bgr.reshape(-1, 3).astype(np.float32)
    d = ((flat[:, None, :] - centres[None, :, :]) ** 2).sum(-1)
    lab = d.argmin(1).reshape(mask.shape).astype(np.uint8)
    lab[~mask] = 255
    # the ink: the darkest pigment, kept and closed separately so the filter can't eat its lines
    ink_i = int(centres.sum(1).argmin())
    ink = cv2.morphologyEx((lab == ink_i).astype(np.uint8), cv2.MORPH_CLOSE, np.ones((3, 3), np.uint8)) > 0
    # majority filter: each pixel takes the commonest pigment around it, twice over
    for radius in (3, 5):
        votes = np.stack([cv2.blur((lab == i).astype(np.float32), (radius, radius)) for i in range(k)])
        best = votes.argmax(0).astype(np.uint8)
        lab = np.where(mask, best, 255).astype(np.uint8)
    # ink back on top, and a solid ink rim round the whole silhouette: the bold outline
    rim = mask & ~(cv2.erode(mask.astype(np.uint8), np.ones((11, 11), np.uint8)) > 0)
    lab[(ink | rim) & mask] = ink_i
    out = np.zeros((*mask.shape, 4), np.uint8)
    for i in range(k):
        out[lab == i, :3] = centres[i].astype(np.uint8)
        out[lab == i, 3] = 255
    cv2.imwrite(out_path, out)


def main(chrome, src, dst, k=8):
    with tempfile.TemporaryDirectory() as tmp:
        raw, cleaned = f"{tmp}/raw.png", f"{tmp}/clean.png"
        render(chrome, src, raw)
        clean(raw, cleaned, int(k))
        vtracer.convert_image_to_svg_py(cleaned, dst, colormode="color", hierarchical="stacked",
                                        mode="spline", filter_speckle=16, color_precision=6,
                                        layer_difference=24, corner_threshold=60,
                                        length_threshold=8.0, splice_threshold=45, path_precision=2)
    out = Path(dst).read_text(encoding="utf-8")
    out = re.sub(r"<\?xml[^>]*>\s*|<!--[^>]*-->\s*", "", out)
    m = re.search(r'width="(\d+)" height="(\d+)"', out)
    w, h = int(m.group(1)), int(m.group(2))
    out = out.replace(m.group(0), f'viewBox="0 0 {w} {h}"', 1)
    # one decimal is finer than a pixel at this scale, and halves the file
    out = re.sub(r"-?\d+\.\d+", lambda n: f"{float(n.group()):.1f}".rstrip("0").rstrip("."), out)
    Path(dst).write_text(out, encoding="utf-8")
    print(f"{dst}: {len(out):,} bytes")


if __name__ == "__main__":
    main(*sys.argv[1:])
