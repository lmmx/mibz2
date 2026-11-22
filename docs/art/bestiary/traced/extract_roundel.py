#!/usr/bin/env python3
"""
Reproducible Per-Component Extraction -> Multi-Layer Potrace Vectorization -> Exact Overlay
for docs/art/bestiary/refs/louis/1-coiled-dragon-roundel.jpg (viewBox="0 0 300 300").

See docs/art/bestiary/traced/PIPELINE.md for full methodology and verification gates.
"""
import base64
import json
import math
import re
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from collections import deque
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw, ImageFilter
from playwright.sync_api import sync_playwright

TRACED = Path(__file__).resolve().parent
BESTIARY = TRACED.parent
COMP_DIR = TRACED / "components" / "roundel"
COMP_DIR.mkdir(parents=True, exist_ok=True)
SRC_IMG = BESTIARY / "refs" / "louis" / "1-coiled-dragon-roundel.jpg"

S = 900  # 3px = 1 SVG unit (<0.35u edge precision, suppresses single-pixel parchment noise)
SCALE = S / 300.0

INK = "#19100b"
IVORY = "#ebe0cb"
IVORY_LIT = "#f7f1e3"
SAGE = "#97a988"
CRIMSON = "#92261e"


def poly_mask(pts_u, size=S):
    m = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(m)
    draw.polygon([(x * SCALE, y * SCALE) for x, y in pts_u], fill=255)
    return m


def mask_and(a, b):
    return ImageChops.darker(a, b)


def mask_or(a, b):
    return ImageChops.lighter(a, b)


def mask_sub(a, b):
    return ImageChops.subtract(a, b)


def mask_dilate(m, r=3):
    return m.filter(ImageFilter.MaxFilter(r))


def mask_erode(m, r=3):
    return m.filter(ImageFilter.MinFilter(r))


def mask_open(m, r=3):
    return mask_dilate(mask_erode(m, r), r)


def mask_close(m, r=3):
    return mask_erode(mask_dilate(m, r), r)


def keep_cc(mask, min_area=60):
    """Retain connected components with area >= min_area (or the single largest if min_area is None)."""
    w, h = mask.size
    px = mask.load()
    visited = bytearray(w * h)
    out = Image.new("L", (w, h), 0)
    opx = out.load()

    comps = []
    for y in range(h):
        row = y * w
        for x in range(w):
            if px[x, y] > 127 and not visited[row + x]:
                q = deque([(x, y)])
                visited[row + x] = 1
                pts = []
                while q:
                    cx, cy = q.popleft()
                    pts.append((cx, cy))
                    for nx, ny in ((cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)):
                        if 0 <= nx < w and 0 <= ny < h:
                            idx = ny * w + nx
                            if not visited[idx] and px[nx, ny] > 127:
                                visited[idx] = 1
                                q.append((nx, ny))
                comps.append(pts)

    if not comps:
        return out
    keep = [max(comps, key=len)] if min_area is None else [c for c in comps if len(c) >= min_area]
    for c in keep:
        for cx, cy in c:
            opx[cx, cy] = 255
    return out


def fill_small_holes(mask, max_hole_area=500):
    """Fill tiny internal pinholes without closing intentional open voids (e.g., transparent mouth gap)."""
    w, h = mask.size
    inv = ImageChops.invert(mask)
    px = inv.load()
    visited = bytearray(w * h)
    out = mask.copy()
    opx = out.load()

    for y in range(h):
        row = y * w
        for x in range(w):
            if px[x, y] > 127 and not visited[row + x]:
                q = deque([(x, y)])
                visited[row + x] = 1
                pts = []
                touches_edge = False
                while q:
                    cx, cy = q.popleft()
                    pts.append((cx, cy))
                    if cx == 0 or cy == 0 or cx == w - 1 or cy == h - 1:
                        touches_edge = True
                    for nx, ny in ((cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)):
                        if 0 <= nx < w and 0 <= ny < h:
                            idx = ny * w + nx
                            if not visited[idx] and px[nx, ny] > 127:
                                visited[idx] = 1
                                q.append((nx, ny))
                if not touches_edge and len(pts) <= max_hole_area:
                    for cx, cy in pts:
                        opx[cx, cy] = 255
    return out


def potrace_mask_to_d(mask, turdsize=12, opttolerance=0.65, alphamax=1.0, decimals=1):
    """Trace a binary mask with potrace 1.16 and convert to compact 0..300 SVG path commands."""
    if mask.getbbox() is None:
        return ""
    bw = ImageChops.invert(mask).convert("1")
    with tempfile.TemporaryDirectory() as td:
        pbm = Path(td) / "in.pbm"
        svg_out = Path(td) / "out.svg"
        bw.save(pbm)
        subprocess.run(
            [
                "potrace", str(pbm), "-s", "--flat",
                "-t", str(turdsize),
                "-a", str(alphamax),
                "-O", str(opttolerance),
                "-o", str(svg_out),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        root = ET.fromstring(svg_out.read_text(encoding="utf-8"))

    ns = {"svg": "http://www.w3.org/2000/svg"}
    g = root.find("svg:g", ns)
    if g is None:
        return ""
    paths = g.findall("svg:path", ns)
    if not paths:
        return ""

    tf = g.attrib.get("transform", "")
    m_tr = re.search(r"translate\(([^,]+),([^)]+)\)", tf)
    m_sc = re.search(r"scale\(([^,]+),([^)]+)\)", tf)
    ty = float(m_tr.group(2)) if m_tr else float(S)
    sx = float(m_sc.group(1)) if m_sc else 0.1
    sy = float(m_sc.group(2)) if m_sc else -0.1

    def fmt(v):
        s = f"{v:.{decimals}f}"
        if decimals > 0 and s.endswith(".0"):
            s = s[:-2]
        return "0" if s == "-0" else s

    out_cmds = []
    token_re = re.compile(r"[MmLlCcZz]|-?\d+(?:\.\d+)?")
    for p in paths:
        d = p.attrib.get("d", "")
        toks = token_re.findall(d)
        i = 0
        cx = cy = sx0 = sy0 = 0.0
        cmd = ""
        while i < len(toks):
            t = toks[i]
            if t.isalpha():
                cmd = t
                i += 1
                if cmd in ("Z", "z"):
                    out_cmds.append("Z")
                    cx, cy = sx0, sy0
                continue
            if cmd in ("M", "m"):
                dx, dy = float(toks[i]), float(toks[i + 1])
                i += 2
                cx, cy = (dx, dy) if cmd == "M" else (cx + dx, cy + dy)
                sx0, sy0 = cx, cy
                out_cmds.append(f"M{fmt((cx * sx) / SCALE)},{fmt((ty + cy * sy) / SCALE)}")
                cmd = "L" if cmd == "M" else "l"
            elif cmd in ("L", "l"):
                dx, dy = float(toks[i]), float(toks[i + 1])
                i += 2
                cx, cy = (dx, dy) if cmd == "L" else (cx + dx, cy + dy)
                out_cmds.append(f"L{fmt((cx * sx) / SCALE)},{fmt((ty + cy * sy) / SCALE)}")
            elif cmd in ("C", "c"):
                a1, b1, a2, b2, a3, b3 = (float(toks[i + k]) for k in range(6))
                i += 6
                if cmd == "C":
                    x1, y1, x2, y2, x3, y3 = a1, b1, a2, b2, a3, b3
                else:
                    x1, y1, x2, y2, x3, y3 = cx + a1, cy + b1, cx + a2, cy + b2, cx + a3, cy + b3
                cx, cy = x3, y3
                out_cmds.append(
                    f"C{fmt((x1 * sx) / SCALE)},{fmt((ty + y1 * sy) / SCALE)} "
                    f"{fmt((x2 * sx) / SCALE)},{fmt((ty + y2 * sy) / SCALE)} "
                    f"{fmt((x3 * sx) / SCALE)},{fmt((ty + y3 * sy) / SCALE)}"
                )
            else:
                i += 1
    return " ".join(out_cmds)


def classify_manuscript(im):
    """Chromatic separation of 1-coiled-dragon-roundel.jpg into 6 medieval pigment classes."""
    px = im.load()
    w, h = im.size

    m_dragon = Image.new("L", (w, h), 0)
    m_ink = Image.new("L", (w, h), 0)
    m_dark_wine = Image.new("L", (w, h), 0)
    m_mid_wine = Image.new("L", (w, h), 0)
    m_ivory = Image.new("L", (w, h), 0)
    m_crimson = Image.new("L", (w, h), 0)
    m_sage = Image.new("L", (w, h), 0)

    pd, pi = m_dragon.load(), m_ink.load()
    pdw, pmw = m_dark_wine.load(), m_mid_wine.load()
    piv, pcr, psg = m_ivory.load(), m_crimson.load(), m_sage.load()

    cx_u, cy_u = 150.2, 152.0
    foot_right_allow = poly_mask([
        (204, 220), (229, 217), (242, 212), (252, 215), (253, 234), (245, 243), (228, 243), (204, 242),
    ]).load()
    foot_bot_allow = poly_mask([
        (119, 248), (144, 249), (144, 267), (159, 272), (159, 284), (151, 292), (132, 292), (129, 282), (118, 282),
    ]).load()

    for y in range(h):
        yu = y / SCALE
        for x in range(w):
            xu = x / SCALE
            dist = math.hypot(xu - cx_u, yu - cy_u)
            in_fr = foot_right_allow[x, y] > 127
            in_fb = foot_bot_allow[x, y] > 127

            if (dist > 116.5 or (xu > 261.5 and 105 <= yu <= 174)) and not (in_fr or in_fb):
                continue

            r, g, b = px[x, y]
            lum = 0.299 * r + 0.587 * g + 0.114 * b

            if dist > 115.5 and (in_fr or in_fb):
                if b > r + 6:
                    continue
                if r > 112 and (r - b) > 54 and g < 118:
                    continue

            is_gold = (r > g + 7) and (g - b > 33) and (b < 116) and (r - g < 65)
            if is_gold:
                continue
            is_dark_gold = (r > g + 9) and (g - b > 22) and (b < 70) and (r - g < 50) and (lum >= 58)
            if is_dark_gold:
                continue

            if lum < 52 or (lum < 68 and (r - g) < 15):
                pi[x, y] = 255
                pd[x, y] = 255
                continue

            if (144 <= xu <= 207 and 172 <= yu <= 243) and (g >= r - 11) and (g > b + 4) and lum > 68:
                psg[x, y] = 255
                pd[x, y] = 255
                continue

            if lum > 146 and b > 114 and (r - b) < 54:
                piv[x, y] = 255
                pd[x, y] = 255
                continue

            in_head_box = (107 <= xu <= 176 and 94 <= yu <= 169)
            in_wing_box = (98 <= xu <= 198 and 160 <= yu <= 272)
            if (in_head_box or in_wing_box) and (r - g) > 34 and (r - b) > 46 and r > 94 and g < 112:
                pcr[x, y] = 255
                pd[x, y] = 255
                continue

            if lum < 104 and (r > g + 8) and (g - b < 24):
                pdw[x, y] = 255
                pd[x, y] = 255
                continue

            if (r >= g - 4) and (g - b < 27) and (b > 52 or r > g + 14):
                pmw[x, y] = 255
                pd[x, y] = 255
                continue

    m_dragon_main = keep_cc(mask_close(m_dragon, 3), min_area=None)
    m_dragon_main = fill_small_holes(m_dragon_main, max_hole_area=420)

    return {
        "dragon": m_dragon_main,
        "ink": keep_cc(mask_and(m_ink, m_dragon_main), min_area=9),
        "dark_wine": keep_cc(mask_and(m_dark_wine, m_dragon_main), min_area=24),
        "mid_wine": keep_cc(mask_and(m_mid_wine, m_dragon_main), min_area=24),
        "ivory": keep_cc(mask_and(m_ivory, m_dragon_main), min_area=6),
        "crimson": keep_cc(mask_and(m_crimson, m_dragon_main), min_area=8),
        "sage": keep_cc(mask_and(m_sage, m_dragon_main), min_area=14),
    }


def extract_components(masks):
    """Segment the verified dragon mask into 6 anatomical components with tucked joint overlaps."""
    D = masks["dragon"]

    roi_head_all = poly_mask([
        (104, 114), (138, 92), (179, 98), (178, 162), (171, 167),
        (151, 167), (144, 165), (108, 156), (104, 136),
    ])
    roi_jaw = poly_mask([
        (108, 142), (138, 146), (153, 154), (156, 165), (144, 165), (108, 156),
    ])
    roi_ears = poly_mask([
        (140, 93), (178, 98), (177, 149), (162, 149), (157, 136),
        (161, 121), (150, 118), (143, 108),
    ])
    comp_head_all = keep_cc(mask_and(D, roi_head_all), min_area=250)
    comp_head_jaw = keep_cc(mask_and(comp_head_all, roi_jaw), min_area=80)
    comp_head_ears = keep_cc(mask_and(comp_head_all, roi_ears), min_area=80)
    comp_head_skull = keep_cc(
        mask_sub(
            mask_sub(comp_head_all, comp_head_jaw),
            mask_sub(comp_head_ears, poly_mask([(148, 116), (175, 118), (175, 149), (156, 148)])),
        ),
        min_area=200,
    )

    roi_wing = poly_mask([
        (195, 161), (199, 174), (208, 187), (206, 206), (192, 222),
        (169, 236), (147, 243), (136, 247), (112, 268), (101, 273),
        (95, 266), (118, 232), (146, 202), (174, 174),
    ])
    comp_wing = keep_cc(mask_and(D, roi_wing), min_area=300)

    roi_foot_right = poly_mask([
        (204, 220), (229, 217), (242, 212), (252, 215), (253, 234), (245, 243), (204, 242),
    ])
    comp_foot_right = keep_cc(mask_and(D, roi_foot_right), min_area=180)

    roi_foot_bot = poly_mask([
        (119, 248), (144, 249), (144, 267), (159, 272), (159, 284), (151, 292), (132, 292), (129, 282), (118, 282),
    ])
    comp_foot_bot = keep_cc(mask_and(D, roi_foot_bot), min_area=180)

    roi_inner_zone = poly_mask([
        (76, 125), (86, 86), (124, 68), (170, 68), (212, 88),
        (238, 134), (242, 184), (222, 226), (188, 254), (141, 260),
        (120, 252), (120, 240), (138, 222), (152, 200), (112, 198),
        (82, 172),
    ])
    outer_raw = mask_sub(mask_sub(mask_sub(mask_sub(D, roi_inner_zone), comp_foot_right), comp_foot_bot), mask_dilate(comp_wing, 3))
    outer_tuck = mask_and(D, poly_mask([(115, 206), (144, 206), (142, 244), (115, 244)]))
    comp_outer_coil = keep_cc(mask_open(mask_or(outer_raw, outer_tuck), 3), min_area=None)

    neck_tuck = poly_mask([(150, 159), (175, 159), (175, 171), (150, 171)])
    head_cut = mask_sub(mask_dilate(comp_head_all, 5), neck_tuck)
    wing_cut = mask_sub(mask_dilate(comp_wing, 3), poly_mask([(168, 166), (205, 166), (205, 215), (155, 238)]))
    foot_r_cut = mask_sub(mask_dilate(comp_foot_right, 5), poly_mask([(200, 220), (217, 220), (217, 242), (200, 242)]))
    foot_b_cut = mask_sub(mask_dilate(comp_foot_bot, 5), poly_mask([(120, 245), (144, 245), (144, 258), (120, 258)]))
    inner_raw = mask_sub(mask_sub(mask_sub(mask_sub(mask_and(D, roi_inner_zone), head_cut), wing_cut), foot_r_cut), foot_b_cut)
    comp_inner_coil = keep_cc(mask_open(inner_raw, 3), min_area=None)

    return {
        "outer_coil": comp_outer_coil,
        "inner_coil": comp_inner_coil,
        "foot_right": comp_foot_right,
        "foot_bottom": comp_foot_bot,
        "wing": comp_wing,
        "head_all": comp_head_all,
        "head_skull": comp_head_skull,
        "head_jaw": comp_head_jaw,
        "head_ears": comp_head_ears,
    }


def trace_component_layers(comp_mask, masks, is_body=False, is_wing=False, is_head=False, is_foot=False):
    """Trace each pigment and ink layer of a single extracted component cleanly via potrace."""
    elems = []
    base_m = fill_small_holes(mask_close(comp_mask, 3), max_hole_area=500)
    d_base = potrace_mask_to_d(base_m, turdsize=22, opttolerance=0.65)
    if not d_base:
        return elems

    if is_body:
        elems.append(f'<path d="{d_base}" fill="var(--wash,#7c3a4c)"/>')
    elif is_wing or is_foot:
        elems.append(f'<path d="{d_base}" fill="{IVORY}"/>')
    elif is_head:
        elems.append(f'<path d="{d_base}" fill="{IVORY_LIT}"/>')

    if is_body:
        m_dw = mask_and(masks["dark_wine"], comp_mask)
        d_dw = potrace_mask_to_d(mask_close(mask_open(m_dw, 3), 5), turdsize=45, opttolerance=1.2)
        if d_dw:
            elems.append(f'<path d="{d_dw}" fill="var(--wash2,#4c1624)"/>')
        m_iv = mask_and(masks["ivory"], comp_mask)
        d_iv = potrace_mask_to_d(m_iv, turdsize=5, opttolerance=0.75)
        if d_iv:
            elems.append(f'<path d="{d_iv}" fill="{IVORY_LIT}" opacity="0.93"/>')

    if is_foot:
        m_mw = mask_and(mask_or(masks["mid_wine"], masks["dark_wine"]), comp_mask)
        d_mw = potrace_mask_to_d(mask_close(mask_open(m_mw, 3), 3), turdsize=18, opttolerance=0.9)
        if d_mw:
            elems.append(f'<path d="{d_mw}" fill="var(--wash,#7c3a4c)" opacity="0.80"/>')
        m_iv = mask_and(masks["ivory"], comp_mask)
        d_iv = potrace_mask_to_d(m_iv, turdsize=5, opttolerance=0.65)
        if d_iv:
            elems.append(f'<path d="{d_iv}" fill="{IVORY_LIT}"/>')

    if is_wing:
        m_cr = mask_and(masks["crimson"], comp_mask)
        d_cr = potrace_mask_to_d(mask_close(m_cr, 3), turdsize=12, opttolerance=0.8)
        if d_cr:
            elems.append(f'<path d="{d_cr}" fill="{CRIMSON}" opacity="0.86"/>')
        m_sg = mask_and(masks["sage"], comp_mask)
        d_sg = potrace_mask_to_d(fill_small_holes(mask_close(m_sg, 5), max_hole_area=900), turdsize=25, opttolerance=0.8)
        if d_sg:
            elems.append(f'<path d="{d_sg}" fill="{SAGE}"/>')
        m_iv = mask_and(masks["ivory"], comp_mask)
        d_iv = potrace_mask_to_d(mask_close(m_iv, 3), turdsize=12, opttolerance=0.75)
        if d_iv:
            elems.append(f'<path d="{d_iv}" fill="{IVORY_LIT}" opacity="0.90"/>')

    if is_head:
        m_cr = mask_and(mask_or(masks["crimson"], masks["dark_wine"]), comp_mask)
        d_cr = potrace_mask_to_d(mask_close(m_cr, 3), turdsize=7, opttolerance=0.6)
        if d_cr:
            elems.append(f'<path d="{d_cr}" fill="{CRIMSON}" opacity="0.92"/>')
        m_iv = mask_and(masks["ivory"], comp_mask)
        d_iv = potrace_mask_to_d(m_iv, turdsize=5, opttolerance=0.55)
        if d_iv:
            elems.append(f'<path d="{d_iv}" fill="{IVORY_LIT}"/>')

    m_ink = mask_and(masks["ink"], mask_dilate(comp_mask, 3))
    m_ink_clean = mask_close(m_ink, 3) if not is_head else m_ink
    d_ink = potrace_mask_to_d(m_ink_clean, turdsize=7 if is_head or is_wing else 14, opttolerance=0.55 if is_head or is_wing else 0.70)
    if d_ink:
        elems.append(f'<path d="{d_ink}" fill="{INK}"/>')

    return elems


def wrap_standalone_svg(inner_xml, comp_mask, title):
    bbox = comp_mask.getbbox()
    if not bbox:
        vb, w, h = "0 0 300 300", 300, 300
    else:
        x0 = max(0, math.floor(bbox[0] / SCALE) - 3)
        y0 = max(0, math.floor(bbox[1] / SCALE) - 3)
        x1 = min(300, math.ceil(bbox[2] / SCALE) + 3)
        y1 = min(300, math.ceil(bbox[3] / SCALE) + 3)
        w, h = x1 - x0, y1 - y0
        vb = f"{x0} {y0} {w} {h}"
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" width="{w}" height="{h}"'
        f' role="img" aria-label="{title}">\n{inner_xml}\n</svg>\n'
    )


def build_composite_svg(traced, animated=False):
    def anim_rot(vals, kt, ks, dur):
        if not animated:
            return ""
        return (
            f'<animateTransform attributeName="transform" type="rotate" values="{vals}"'
            f' keyTimes="{kt}" calcMode="spline" keySplines="{ks}" dur="{dur}" repeatCount="indefinite"/>'
        )

    def anim_scale(vals, kt, ks, dur):
        if not animated:
            return ""
        return (
            f'<animateTransform attributeName="transform" type="scale" values="{vals}"'
            f' keyTimes="{kt}" calcMode="spline" keySplines="{ks}" dur="{dur}" repeatCount="indefinite"/>'
        )

    def pivot_wrap(gid, px, py, anim_tag, inner):
        return (
            f'<g transform="translate({px},{py})"><g id="{gid}">{anim_tag}'
            f'<g transform="translate({-px},{-py})">\n{inner}\n</g></g></g>'
        )

    outer_g = '<g id="roundel-outer-tail-coil">\n' + "\n".join(traced["outer_coil"]) + "\n</g>"
    foot_r_g = pivot_wrap(
        "roundel-foot-right", 216, 232,
        anim_rot("0;2.6;-1;0", "0;.38;.72;1", ".4 0 .6 1;.4 0 .6 1;.4 0 .6 1", "6.7s"),
        "\n".join(traced["foot_right"]),
    )
    foot_b_g = pivot_wrap(
        "roundel-foot-bottom", 131, 254,
        anim_rot("0;-2.2;1.2;0", "0;.42;.78;1", ".4 0 .6 1;.4 0 .6 1;.4 0 .6 1", "7.3s"),
        "\n".join(traced["foot_bottom"]),
    )
    feet_g = f'<g id="roundel-feet-assembly">\n{foot_r_g}\n{foot_b_g}\n</g>'

    wing_g = pivot_wrap(
        "roundel-wing", 192, 166,
        anim_rot("0;2.8;-1.2;0", "0;.36;.74;1", ".42 0 .58 1;.42 0 .58 1;.42 0 .58 1", "5.9s"),
        "\n".join(traced["wing"]),
    )
    jaw_g = pivot_wrap(
        "roundel-jaw", 149, 155,
        anim_rot("0;0;-6.5;-2;-7.5;0;0", "0;.42;.50;.58;.65;.76;1",
                 "0 0 1 1;.3 0 .4 1;.4 0 .6 1;.3 0 .4 1;.4 0 .6 1;0 0 1 1", "6.1s"),
        "\n".join(traced["head_jaw"]),
    )
    ears_g = pivot_wrap(
        "roundel-ears", 158, 124,
        anim_rot("0;0;3;-1.5;0;0", "0;.24;.32;.42;.52;1",
                 "0 0 1 1;.3 0 .5 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1", "5.3s"),
        "\n".join(traced["head_ears"]),
    )
    lid_g = (
        f'<g transform="translate(141,129)"><g id="roundel-lid" transform="scale(1 .05)">'
        f'{anim_scale("1 .05;1 .05;1 1.05;1 1.05;1 .05;1 .05", "0;.36;.38;.41;.44;1", "0 0 1 1;.3 0 .6 1;0 0 1 1;.3 0 .7 1;0 0 1 1", "4.7s")}'
        f'<path d="M-5.5,0 L6.5,0 L6,5.5 C2,6.5 -2,6.5 -5,5 Z" fill="{CRIMSON}" stroke="{INK}" stroke-width="0.9"/>'
        f'</g></g>'
    ) if animated else ""
    skull_g = '<g id="roundel-skull">\n' + "\n".join(traced["head_skull"]) + (f"\n{lid_g}" if lid_g else "") + "\n</g>"
    head_g = pivot_wrap(
        "roundel-head", 166, 156,
        anim_rot("0;-3.8;-3.8;2.8;2.8;0", "0;.18;.42;.60;.82;1",
                 ".4 0 .6 1;0 0 1 1;.4 0 .6 1;0 0 1 1;.4 0 .6 1", "7.1s"),
        f"{jaw_g}\n{skull_g}\n{ears_g}",
    )
    inner_g = pivot_wrap(
        "roundel-inner-coil", 158, 158,
        anim_scale("1 1;1.012 .992;1 1", "0;.50;1", ".42 0 .58 1;.42 0 .58 1", "6.7s"),
        "\n".join(traced["inner_coil"]) + f"\n{wing_g}\n{head_g}",
    )

    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300" width="300" height="300"'
        f' role="img" aria-label="Component-extracted Romanesque coiled dragon roundel">\n'
        f'<g id="roundel-root">\n{outer_g}\n{feet_g}\n{inner_g}\n</g>\n</svg>\n'
    )


def compute_iou(src_mask, svg_text, td):
    sil_html = f"""<!doctype html><html><body style="margin:0;background:#fff">
    <style>svg * {{ fill:#000 !important; stroke:#000 !important; }}</style>
    {svg_text.replace('width="300" height="300"', 'width="600" height="600"')}
    </body></html>"""
    tmp = td / "iou_check.html"
    tmp.write_text(sil_html, encoding="utf-8")
    out_png = td / "iou_svg.png"
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path="/usr/bin/google-chrome", headless=True)
        pg = b.new_page(viewport={"width": 600, "height": 600})
        pg.goto(tmp.as_uri(), wait_until="load")
        pg.screenshot(path=str(out_png))
        b.close()

    svg_im = Image.open(out_png).convert("L")
    src_im = src_mask.resize((600, 600), Image.Resampling.NEAREST)
    sp, vp = src_im.load(), svg_im.load()
    inter = union = 0
    for y in range(600):
        for x in range(600):
            a = sp[x, y] > 127
            b = vp[x, y] < 128
            if a and b:
                inter += 1
            if a or b:
                union += 1
    return 100.0 * inter / max(1, union)


def main():
    im = Image.open(SRC_IMG).convert("RGB").resize((S, S), Image.Resampling.LANCZOS)
    masks = classify_manuscript(im)
    comps = extract_components(masks)

    traced = {
        "outer_coil": trace_component_layers(comps["outer_coil"], masks, is_body=True),
        "inner_coil": trace_component_layers(comps["inner_coil"], masks, is_body=True),
        "foot_right": trace_component_layers(comps["foot_right"], masks, is_foot=True),
        "foot_bottom": trace_component_layers(comps["foot_bottom"], masks, is_foot=True),
        "wing": trace_component_layers(comps["wing"], masks, is_wing=True),
        "head_jaw": trace_component_layers(comps["head_jaw"], masks, is_head=True),
        "head_skull": trace_component_layers(comps["head_skull"], masks, is_head=True),
        "head_ears": trace_component_layers(comps["head_ears"], masks, is_head=True),
    }

    standalone_specs = [
        ("head.svg", comps["head_all"], "Romanesque roundel dragon head",
         '<g id="head-jaw">\n' + "\n".join(traced["head_jaw"]) + "\n</g>\n"
         '<g id="head-skull">\n' + "\n".join(traced["head_skull"]) + "\n</g>\n"
         '<g id="head-ears">\n' + "\n".join(traced["head_ears"]) + "\n</g>"),
        ("wing.svg", comps["wing"], "Romanesque roundel dragon wing",
         '<g id="wing">\n' + "\n".join(traced["wing"]) + "\n</g>"),
        ("foot-right.svg", comps["foot_right"], "Romanesque roundel dragon right foot",
         '<g id="foot-right">\n' + "\n".join(traced["foot_right"]) + "\n</g>"),
        ("foot-bottom.svg", comps["foot_bottom"], "Romanesque roundel dragon bottom foot",
         '<g id="foot-bottom">\n' + "\n".join(traced["foot_bottom"]) + "\n</g>"),
        ("inner-coil.svg", comps["inner_coil"], "Romanesque roundel dragon inner coil",
         '<g id="inner-coil">\n' + "\n".join(traced["inner_coil"]) + "\n</g>"),
        ("outer-coil.svg", comps["outer_coil"], "Romanesque roundel dragon outer tail coil",
         '<g id="outer-coil">\n' + "\n".join(traced["outer_coil"]) + "\n</g>"),
    ]
    comp_svgs = {}
    for fname, cmask, title, inner in standalone_specs:
        svg_s = wrap_standalone_svg(inner, cmask, title)
        ET.fromstring(svg_s)
        (COMP_DIR / fname).write_text(svg_s, encoding="utf-8")
        comp_svgs[fname] = svg_s

    master_svg = build_composite_svg(traced, animated=False)
    anim_svg = build_composite_svg(traced, animated=True)
    ET.fromstring(master_svg)
    ET.fromstring(anim_svg)

    (TRACED / "romanesque-roundel-master.svg").write_text(master_svg, encoding="utf-8")
    (TRACED / "romanesque-roundel-animated.svg").write_text(anim_svg, encoding="utf-8")

    with tempfile.TemporaryDirectory() as tmpdir:
        td = Path(tmpdir)
        iou = compute_iou(masks["dragon"], master_svg, td)
        print(f"Silhouette IoU: {iou:.2f}%")

        meta = {
            "component": "romanesque_coiled_dragon_roundel",
            "source_ref": "docs/art/bestiary/refs/louis/1-coiled-dragon-roundel.jpg",
            "source_style": "Romanesque manuscript illumination",
            "date_range": "12th-13th century",
            "medium": "ink and opaque mineral pigment on gilded parchment",
            "tracing_method": "per-component chromatic pigment extraction at 900x900 followed by clean multi-layer potrace Bezier vectorization and exact semantic re-assembly",
            "silhouette_iou_pct": round(iou, 2),
            "components_dir": "docs/art/bestiary/traced/components/roundel/",
            "components": [s[0] for s in standalone_specs],
            "assemblies": [
                "roundel-outer-tail-coil (static ground truth)",
                "roundel-feet-assembly (roundel-foot-right, roundel-foot-bottom)",
                "roundel-inner-coil (subtle breath tighten)",
                "roundel-wing (pinions + sage leaf covert)",
                "roundel-head (roundel-jaw, roundel-skull, roundel-ears, blink lid)"
            ],
            "palette": ["madder-wine (--wash)", "deep-wine (--wash2)", "parchment-ivory", "sage-green", "crimson", "iron-gall ink"],
            "stages": {
                "stage_a_master": "traced/romanesque-roundel-master.svg",
                "stage_c_animated": "traced/romanesque-roundel-animated.svg"
            }
        }
        (TRACED / "romanesque-roundel.meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")

        comp_cards = []
        for fname, cmask, _, _ in standalone_specs:
            bbox = cmask.getbbox()
            x0, y0 = max(0, bbox[0] - int(3 * SCALE)), max(0, bbox[1] - int(3 * SCALE))
            x1, y1 = min(S, bbox[2] + int(3 * SCALE)), min(S, bbox[3] + int(3 * SCALE))
            cut = Image.new("RGBA", (S, S), (0, 0, 0, 0))
            cut.paste(im.convert("RGBA"), mask=cmask)
            tmp_png = td / f"cut_{fname}.png"
            cut.crop((x0, y0, x1, y1)).save(tmp_png)
            b64 = "data:image/png;base64," + base64.b64encode(tmp_png.read_bytes()).decode("ascii")
            svg_fluid = re.sub(r'\s+width="[^"]+"\s+height="[^"]+"', '', comp_svgs[fname], count=1)
            comp_cards.append(f"""
              <div class="ccard">
                <div class="tag">{fname} ({len(comp_svgs[fname].encode('utf-8'))/1024:.1f} KB)</div>
                <div class="pair">
                  <div class="box"><div class="sub">Extracted Pixels</div><img src="{b64}"></div>
                  <div class="box"><div class="sub">Traced SVG</div>{svg_fluid}</div>
                  <div class="box ov"><div class="sub">50% Overlay</div><img src="{b64}">{svg_fluid}</div>
                </div>
              </div>""")

        comp_html = f"""<!doctype html><html><head><meta charset="utf-8"><style>
          * {{ box-sizing: border-box; }}
          body {{ margin: 0; padding: 18px; background: #e8dec8; color: #1c100a; font: 13px/1.4 Georgia, serif; }}
          h1 {{ margin: 0 0 4px; font-size: 18px; }}
          p.sub {{ margin: 0 0 12px; font-style: italic; color: #544232; font-size: 12px; }}
          .grid2 {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; }}
          .ccard {{ background: #faf4e6; border: 1px solid #b8a582; padding: 10px; }}
          .tag {{ font: 11px/1.2 ui-monospace, monospace; color: #4a3624; margin-bottom: 6px; font-weight: 700; }}
          .pair {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }}
          .box {{ position: relative; height: 188px; background: #f3ead8; border: 1px solid #d4c5a8; overflow: hidden; }}
          .box .sub {{ position: absolute; top: 3px; left: 6px; font: 9.5px ui-monospace, monospace; color: #6e5840; z-index: 3; }}
          .box img, .box svg {{ position: absolute; inset: 18px 8px 8px; width: calc(100% - 16px); height: calc(100% - 26px); object-fit: contain; }}
          .box.ov svg {{ opacity: 0.55; z-index: 2; }}
        </style></head><body>
          <h1>Extracted Historical Components &rarr; Clean Potrace Vector Library (IoU: {iou:.2f}%)</h1>
          <p class="sub">Each anatomical component isolated cleanly from 1-coiled-dragon-roundel.jpg, traced layer-by-layer with potrace, and verified at 1:1 scale via 50% overlay.</p>
          <div class="grid2">{''.join(comp_cards)}</div>
        </body></html>"""

        ref_b64 = "data:image/jpeg;base64," + base64.b64encode(SRC_IMG.read_bytes()).decode("ascii")
        master_head_vb = master_svg.replace('viewBox="0 0 300 300"', 'viewBox="102 92 80 80"')
        master_wf_vb = master_svg.replace('viewBox="0 0 300 300"', 'viewBox="95 160 162 135"')

        sheet_html = f"""<!doctype html><html><head><meta charset="utf-8"><style>
          * {{ box-sizing: border-box; }}
          body {{ margin: 0; padding: 18px; background: #e8dec8; color: #1c100a; font: 13px/1.4 Georgia, serif; }}
          h1 {{ margin: 0 0 4px; font-size: 18px; }}
          p.sub {{ margin: 0 0 12px; font-style: italic; color: #544232; font-size: 12px; }}
          .grid4 {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 12px; }}
          .card {{ background: #faf4e6; border: 1px solid #b8a582; padding: 8px; }}
          .tag {{ font: 10.5px/1.2 ui-monospace, monospace; color: #5c4632; margin-bottom: 6px; }}
          .stage {{ position: relative; width: 256px; height: 256px; margin: 0 auto; overflow: hidden; }}
          .stage img, .stage svg {{ position: absolute; inset: 0; width: 100%; height: 100%; display: block; }}
          .overlay .ov-svg {{ position: absolute; inset: 0; opacity: 0.52; }}
          .sil svg * {{ fill: #1a110c !important; stroke: #1a110c !important; }}
          .gold-bg {{ background: radial-gradient(circle at 45% 42%, #e8c44a 0%, #bf9124 62%, #8c6412 100%); border-radius: 50%; border: 8px solid #c2381c; outline: 6px solid #2e4898; }}
          .v1 {{ --wash: #7c3a4c; --wash2: #4c1624; }}
          .v2 {{ --wash: #2a4d8e; --wash2: #172d5c; }}
        </style></head><body>
          <h1>Component-Extracted Master Trace — Romanesque Coiled Dragon Roundel (IoU: {iou:.2f}% · {len(master_svg.encode('utf-8'))/1024:.1f} KB)</h1>
          <p class="sub">6 extracted components traced individually with potrace 1.16 and overlaid back onto 1-coiled-dragon-roundel.jpg (Stage A static master &rarr; Stage C SMIL).</p>
          <div class="grid4">
            <div class="card"><div class="tag">1. Source: 1-coiled-dragon-roundel.jpg</div><div class="stage"><img src="{ref_b64}"></div></div>
            <div class="card"><div class="tag">2. Full 50% Overlay (IoU {iou:.1f}%)</div><div class="stage overlay"><img src="{ref_b64}"><div class="ov-svg">{master_svg}</div></div></div>
            <div class="card"><div class="tag">3. Zoomed Head 50% Overlay</div><div class="stage overlay">
              <svg viewBox="102 92 80 80" width="256" height="256"><image href="{ref_b64}" x="0" y="0" width="300" height="300"/></svg>
              <div class="ov-svg">{master_head_vb}</div>
            </div></div>
            <div class="card"><div class="tag">4. Zoomed Wing &amp; Feet 50% Overlay</div><div class="stage overlay">
              <svg viewBox="95 160 162 135" width="256" height="256"><image href="{ref_b64}" x="0" y="0" width="300" height="300"/></svg>
              <div class="ov-svg">{master_wf_vb}</div>
            </div></div>
          </div>
          <div class="grid4">
            <div class="card"><div class="tag">Stage A Master (Composited Ground Truth)</div><div class="stage v1">{master_svg}</div></div>
            <div class="card"><div class="tag">Pure Black Silhouette Test</div><div class="stage sil">{master_svg}</div></div>
            <div class="card"><div class="tag">Master in Gilded Roundel Context</div><div class="stage v1 gold-bg">{master_svg}</div></div>
            <div class="card"><div class="tag">Mineral Lapis Variant (--wash)</div><div class="stage v2">{master_svg}</div></div>
          </div>
          <div class="grid4">
            <div class="card"><div class="tag">Stage C Animated · t = 0.00s (poised)</div><div class="stage v1 anim-slot" data-t="0.00">{anim_svg}</div></div>
            <div class="card"><div class="tag">Stage C Animated · t = 1.85s (blink &amp; turn)</div><div class="stage v1 anim-slot" data-t="1.85">{anim_svg}</div></div>
            <div class="card"><div class="tag">Stage C Animated · t = 3.15s (jaw open &amp; breathe)</div><div class="stage v1 anim-slot" data-t="3.15">{anim_svg}</div></div>
            <div class="card"><div class="tag">Stage C Animated · t = 4.95s (glance &amp; wing)</div><div class="stage v1 anim-slot" data-t="4.95">{anim_svg}</div></div>
          </div>
        </body></html>"""

        (td / "comp_sheet.html").write_text(comp_html, encoding="utf-8")
        (td / "master_sheet.html").write_text(sheet_html, encoding="utf-8")

        with sync_playwright() as p:
            b = p.chromium.launch(executable_path="/usr/bin/google-chrome", headless=True)
            pg1 = b.new_page(viewport={"width": 1220, "height": 860}, device_scale_factor=1.75)
            pg1.goto((td / "comp_sheet.html").as_uri(), wait_until="load")
            pg1.wait_for_timeout(120)
            pg1.screenshot(path=str(TRACED / "romanesque-roundel-components.png"), full_page=True)

            pg2 = b.new_page(viewport={"width": 1160, "height": 1000}, device_scale_factor=2)
            pg2.goto((td / "master_sheet.html").as_uri(), wait_until="load")
            pg2.wait_for_timeout(120)
            pg2.evaluate("""() => {
              document.querySelectorAll('.anim-slot').forEach(slot => {
                const t = parseFloat(slot.getAttribute('data-t'));
                const svg = slot.querySelector('svg');
                if (svg && svg.pauseAnimations) { svg.pauseAnimations(); svg.setCurrentTime(t); }
              });
            }""")
            pg2.wait_for_timeout(60)
            pg2.screenshot(path=str(TRACED / "romanesque-roundel-sheet.png"), full_page=True)
            b.close()

    proof_path = BESTIARY / "proofs" / "proof-traced-roundel.html"
    if proof_path.exists():
        old = proof_path.read_text(encoding="utf-8")
        updated = re.sub(
            r'<div class="stage gold" id="anim-stage">.*?</div>\s*</div>',
            f'<div class="stage gold" id="anim-stage">{anim_svg}</div>\n      </div>',
            old, count=1, flags=re.S,
        )
        proof_path.write_text(updated, encoding="utf-8")


if __name__ == "__main__":
    main()
