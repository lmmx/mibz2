"""Re-trace components of a traced specimen from its source photograph, keeping its detail.

The first trace of a manuscript creature kept its silhouette but dropped detail (eyes, mouths,
scales, wing ribs). This takes each component's silhouette from that trace, masks the source
photograph with it at full resolution, traces the masked photograph again with vtracer at fine
settings, and maps each traced colour to one of four tones by lightness, so a page can still
recolour them:
- ink: the darkest, outlines and line work;
- wash2: dark mid-tones;
- wash: light mid-tones;
- hi: highlights.

The output is a JSON of layers in the specimen's own units ({"d", "role"}, the format the brief-6
rig's buildTracedGroup reads).

    uv run --no-project -p 3.13 --with vtracer --with opencv-python-headless --with numpy \
        --with playwright==1.56.0 python docs/scripts/bestiary_detail_trace.py \
        CHROME PHOTO MASTER_VIEWBOX_W MASTER_VIEWBOX_H OUT.json NAME=COMPONENT.svg [...]
"""

import json
import re
import sys
import tempfile

import cv2
import numpy as np
import vtracer
from playwright.sync_api import sync_playwright


def silhouette(chrome, comp_svg, vw, vh, W, H, path):
    svg = open(comp_svg, encoding="utf-8").read()
    svg = re.sub(r'viewBox="[^"]*"', f'viewBox="0 0 {vw} {vh}"', svg, count=1)
    svg = re.sub(r'\s(width|height)="[\d.]+"', "", svg, count=2)
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path=chrome)
        pg = b.new_page(viewport={"width": W, "height": H})
        pg.set_content(f"<html><body style='margin:0;background:transparent'><style>svg{{display:block;width:{W}px;height:{H}px}}</style>{svg}</body></html>")
        pg.locator("svg").screenshot(path=path, omit_background=True)
        b.close()


def to_units(d, tx, ty, s):
    out, nums = [], re.findall(r"[MLCZmlcz]|-?\d*\.?\d+(?:e-?\d+)?", d)
    i, axis = 0, 0
    for tok in nums:
        if tok.isalpha():
            out.append(tok.upper()); axis = 0
        else:
            v = float(tok) + (tx if axis == 0 else ty)
            out.append(f"{v / s:.1f}".rstrip("0").rstrip("."))
            axis ^= 1
    return " ".join(out).replace(" Z", "Z")


def main(chrome, photo, vw, vh, out_json, *comps):
    vw, vh = float(vw), float(vh)
    img = cv2.imread(photo)
    H, W = img.shape[:2]
    s = W / vw
    lab = cv2.cvtColor(img, cv2.COLOR_BGR2LAB)[..., 0]
    result = {}
    with tempfile.TemporaryDirectory() as tmp:
        for spec in comps:
            name, comp = spec.split("=", 1)
            sil = f"{tmp}/{name}-sil.png"
            silhouette(chrome, comp, vw, vh, W, H, sil)
            a = cv2.imread(sil, cv2.IMREAD_UNCHANGED)[..., 3]
            a = cv2.resize(a, (W, H))
            m = cv2.morphologyEx((a > 40).astype(np.uint8), cv2.MORPH_CLOSE, np.ones((15, 15), np.uint8))
            m = cv2.dilate(m, np.ones((5, 5), np.uint8)) > 0
            # quantise the masked photo to four tones by lightness, so each traced patch has one role
            L = lab[m]
            q = np.percentile(L, [18, 50, 86])
            tone = np.digitize(lab, q)  # 0 ink, 1 wash2, 2 wash, 3 hi
            pal = np.array([[11, 16, 25], [60, 60, 160], [120, 160, 220], [214, 243, 255]], np.uint8)  # BGR stand-ins
            rgba = np.zeros((H, W, 4), np.uint8)
            rgba[..., :3] = pal[tone]
            rgba[..., 3] = np.where(m, 255, 0)
            src = f"{tmp}/{name}.png"; dst = f"{tmp}/{name}.svg"
            cv2.imwrite(src, rgba)
            vtracer.convert_image_to_svg_py(src, dst, colormode="color", hierarchical="stacked", mode="spline",
                                            filter_speckle=6, color_precision=8, layer_difference=8,
                                            corner_threshold=60, length_threshold=4.0, splice_threshold=45,
                                            path_precision=2)
            role_of = {tuple(pal[i][::-1]): r for i, r in enumerate(["ink", "wash2", "wash", "hi"])}
            layers = []
            for pm in re.finditer(r'<path d="([^"]+)" fill="#([0-9A-Fa-f]{6})"(?: transform="translate\(([-\d.]+),([-\d.]+)\)")?', open(dst).read()):
                rgb = tuple(int(pm.group(2)[k:k + 2], 16) for k in (0, 2, 4))
                role = min(role_of, key=lambda c: sum((int(x) - int(y)) ** 2 for x, y in zip(c, rgb)))
                tx, ty = float(pm.group(3) or 0), float(pm.group(4) or 0)
                layers.append({"d": to_units(pm.group(1), tx, ty, s), "role": role_of[role]})
            result[name] = layers
            print(f"{name}: {len(layers)} paths, {sum(len(l['d']) for l in layers):,} bytes of path data")
    json.dump(result, open(out_json, "w"), separators=(",", ":"))


if __name__ == "__main__":
    main(*sys.argv[1:])
