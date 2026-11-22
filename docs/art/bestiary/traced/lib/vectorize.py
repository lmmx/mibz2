"""Stage 4: Dual-resolution bias-compensated per-layer binary vtracer + lossless SVG path compactor."""
from __future__ import annotations

import io
import math
import re
import xml.etree.ElementTree as ET
from dataclasses import replace
from typing import Any, Dict, List, Optional, Sequence, Tuple

import numpy as np
import vtracer
from PIL import Image, ImageFilter

from .presets import FamilyPreset, LayerTraceConfig

_TOKEN_RE = re.compile(r"[MmLlHhVvCcSsQqTtAaZz]|-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?")
_TRANSLATE_RE = re.compile(r"translate\(\s*([+-]?[\d.eE-]+)[,\s]+([+-]?[\d.eE-]+)\s*\)")


def _fmt_num(v: float, decimals: int = 1) -> str:
    r = round(float(v), decimals)
    if abs(r) < 10 ** (-decimals - 1):
        return "0"
    if decimals == 0:
        return str(int(round(r)))
    s = f"{r:.{decimals}f}".rstrip("0").rstrip(".")
    if s.startswith("0."):
        s = s[1:]
    elif s.startswith("-0."):
        s = "-" + s[2:]
    return "0" if s in ("", "-0") else s


def _join_nums(nums: Sequence[str]) -> str:
    """Join formatted SVG numbers cleanly, omitting spaces before leading minus signs."""
    if not nums:
        return ""
    out = [nums[0]]
    for n in nums[1:]:
        if n.startswith("-"):
            out.append(n)
        else:
            out.append(" " + n)
    return "".join(out)


def _pt_to_seg_dist(px: float, py: float, x0: float, y0: float, x1: float, y1: float) -> Tuple[float, float]:
    """Return (perpendicular distance from P to line P0->P1, normalized projection parameter t)."""
    dx, dy = x1 - x0, y1 - y0
    l2 = dx * dx + dy * dy
    if l2 < 1e-12:
        return math.hypot(px - x0, py - y0), 0.0
    t = ((px - x0) * dx + (py - y0) * dy) / l2
    proj_x = x0 + t * dx
    proj_y = y0 + t * dy
    return math.hypot(px - proj_x, py - proj_y), t


def _parse_vtracer_subpaths(
    svg_text: str,
    scale_x: float,
    scale_y: float,
) -> List[Tuple[Tuple[float, float], List[Tuple[str, Tuple[float, ...]]]]]:
    """Parse vtracer SVG <path> elements, baking translate(tx,ty) and scaling into target viewBox coords."""
    root = ET.fromstring(svg_text)
    subpaths: List[Tuple[Tuple[float, float], List[Tuple[str, Tuple[float, ...]]]]] = []

    for elem in root.iter():
        tag = elem.tag.split("}")[-1]
        if tag != "path":
            continue
        d = elem.attrib.get("d", "")
        if not d:
            continue
        tf = elem.attrib.get("transform", "")
        m_tr = _TRANSLATE_RE.search(tf)
        tx = float(m_tr.group(1)) if m_tr else 0.0
        ty = float(m_tr.group(2)) if m_tr else 0.0

        toks = _TOKEN_RE.findall(d)
        i = 0
        n = len(toks)
        cmd = ""
        cx = cy = sx0 = sy0 = 0.0
        cur_start: Optional[Tuple[float, float]] = None
        cur_segs: List[Tuple[str, Tuple[float, ...]]] = []

        def to_vb(x: float, y: float) -> Tuple[float, float]:
            return ((x + tx) * scale_x, (y + ty) * scale_y)

        while i < n:
            t = toks[i]
            if t.isalpha():
                cmd = t
                i += 1
                if cmd in ("Z", "z"):
                    if cur_start is not None and cur_segs:
                        subpaths.append((cur_start, cur_segs))
                    cur_start = None
                    cur_segs = []
                    cx, cy = sx0, sy0
                continue

            if cmd in ("M", "m"):
                x, y = float(toks[i]), float(toks[i + 1])
                i += 2
                if cur_start is not None and cur_segs:
                    subpaths.append((cur_start, cur_segs))
                cx, cy = (x, y) if cmd == "M" else (cx + x, cy + y)
                sx0, sy0 = cx, cy
                cur_start = to_vb(cx, cy)
                cur_segs = []
                cmd = "L" if cmd == "M" else "l"
            elif cmd in ("L", "l"):
                x, y = float(toks[i]), float(toks[i + 1])
                i += 2
                cx, cy = (x, y) if cmd == "L" else (cx + x, cy + y)
                cur_segs.append(("L", to_vb(cx, cy)))
            elif cmd in ("H", "h"):
                x = float(toks[i])
                i += 1
                cx = x if cmd == "H" else cx + x
                cur_segs.append(("L", to_vb(cx, cy)))
            elif cmd in ("V", "v"):
                y = float(toks[i])
                i += 1
                cy = y if cmd == "V" else cy + y
                cur_segs.append(("L", to_vb(cx, cy)))
            elif cmd in ("C", "c"):
                x1, y1, x2, y2, x3, y3 = (float(toks[i + k]) for k in range(6))
                i += 6
                if cmd == "c":
                    x1, y1 = cx + x1, cy + y1
                    x2, y2 = cx + x2, cy + y2
                    x3, y3 = cx + x3, cy + y3
                p1 = to_vb(x1, y1)
                p2 = to_vb(x2, y2)
                p3 = to_vb(x3, y3)
                cur_segs.append(("C", (p1[0], p1[1], p2[0], p2[1], p3[0], p3[1])))
                cx, cy = x3, y3
            else:
                i += 1

        if cur_start is not None and cur_segs:
            subpaths.append((cur_start, cur_segs))

    return subpaths


def _simplify_and_encode_subpath(
    start: Tuple[float, float],
    segs: List[Tuple[str, Tuple[float, ...]]],
    line_eps: float,
    smooth_eps: float,
    min_subpath_u: float,
    decimals: int = 1,
) -> Tuple[str, Dict[str, int]]:
    """Collapse collinear cubics, merge collinear lines, quantize absolutes, emit compact relative SVG path."""
    # Check bounding box first to drop tiny sub-pixel specks
    xs = [start[0]]
    ys = [start[1]]
    for kind, vals in segs:
        if kind == "L":
            xs.append(vals[0])
            ys.append(vals[1])
        else:
            xs.extend((vals[0], vals[2], vals[4]))
            ys.extend((vals[1], vals[3], vals[5]))
    if max(max(xs) - min(xs), max(ys) - min(ys)) < min_subpath_u:
        return "", {}

    # Step 1: Collapse collinear cubics C -> L
    collapsed: List[Tuple[str, Tuple[float, ...]]] = []
    px, py = start
    for kind, vals in segs:
        if kind == "L":
            ex, ey = vals
            if math.hypot(ex - px, ey - py) > 1e-4:
                collapsed.append(("L", (ex, ey)))
                px, py = ex, ey
        else:
            x1, y1, x2, y2, x3, y3 = vals
            if math.hypot(x3 - px, y3 - py) < 1e-4 and math.hypot(x1 - px, y1 - py) < line_eps:
                continue
            d1, t1 = _pt_to_seg_dist(x1, y1, px, py, x3, y3)
            d2, t2 = _pt_to_seg_dist(x2, y2, px, py, x3, y3)
            if d1 <= line_eps and d2 <= line_eps and -0.15 <= t1 <= 1.15 and -0.15 <= t2 <= 1.15:
                collapsed.append(("L", (x3, y3)))
            else:
                collapsed.append(("C", vals))
            px, py = x3, y3

    if not collapsed:
        return "", {}

    # Step 2: Merge consecutive collinear line runs via chord-deviation check
    merged: List[Tuple[str, Tuple[float, ...]]] = []
    idx = 0
    px, py = start
    while idx < len(collapsed):
        if collapsed[idx][0] != "L":
            merged.append(collapsed[idx])
            px, py = collapsed[idx][1][4], collapsed[idx][1][5]
            idx += 1
            continue
        run_pts = [(px, py)]
        while idx < len(collapsed) and collapsed[idx][0] == "L":
            run_pts.append(collapsed[idx][1])
            idx += 1
        # Greedy RDP-like collinear merge along run_pts
        anchor = 0
        while anchor < len(run_pts) - 1:
            best_j = anchor + 1
            for j in range(anchor + 2, len(run_pts)):
                x0, y0 = run_pts[anchor]
                xj, yj = run_pts[j]
                max_d = 0.0
                for k in range(anchor + 1, j):
                    dk, _ = _pt_to_seg_dist(run_pts[k][0], run_pts[k][1], x0, y0, xj, yj)
                    if dk > max_d:
                        max_d = dk
                        if max_d > line_eps:
                            break
                if max_d <= line_eps:
                    best_j = j
                else:
                    break
            merged.append(("L", run_pts[best_j]))
            anchor = best_j
        px, py = run_pts[-1]

    # Step 3: Quantize ABSOLUTE coordinates before relative differencing (zero drift!)
    q = lambda v: round(float(v), decimals)
    sx, sy = q(start[0]), q(start[1])
    cur_x, cur_y = sx, sy
    prev_cmd = ""
    prev_c2: Optional[Tuple[float, float]] = None

    tokens: List[str] = [f"M{_join_nums([_fmt_num(sx, decimals), _fmt_num(sy, decimals)])}"]
    counts: Dict[str, int] = {"M": 1, "c": 0, "s": 0, "l": 0, "h": 0, "v": 0, "z": 1}

    # Drop final closing line segment if it returns exactly to quantized start
    if merged and merged[-1][0] == "L":
        ex, ey = q(merged[-1][1][0]), q(merged[-1][1][1])
        if ex == sx and ey == sy:
            merged = merged[:-1]

    if not merged:
        return "", {}

    for kind, vals in merged:
        if kind == "L":
            ex, ey = q(vals[0]), q(vals[1])
            dx, dy = q(ex - cur_x), q(ey - cur_y)
            if dx == 0.0 and dy == 0.0:
                continue
            prev_c2 = None
            if dy == 0.0:
                cmd = "h"
                payload = _fmt_num(dx, decimals)
                counts["h"] += 1
            elif dx == 0.0:
                cmd = "v"
                payload = _fmt_num(dy, decimals)
                counts["v"] += 1
            else:
                cmd = "l"
                payload = _join_nums([_fmt_num(dx, decimals), _fmt_num(dy, decimals)])
                counts["l"] += 1
            if cmd == prev_cmd:
                tokens.append(("" if payload.startswith("-") else " ") + payload)
            else:
                tokens.append(cmd + payload)
                prev_cmd = cmd
            cur_x, cur_y = ex, ey
        else:
            x1, y1, x2, y2, x3, y3 = (q(v) for v in vals)
            dx3, dy3 = q(x3 - cur_x), q(y3 - cur_y)
            if dx3 == 0.0 and dy3 == 0.0 and q(x1 - cur_x) == 0.0 and q(y1 - cur_y) == 0.0:
                continue
            # Check smooth cubic reflection shorthand `s`
            use_smooth = False
            if prev_c2 is not None:
                rx = 2.0 * cur_x - prev_c2[0]
                ry = 2.0 * cur_y - prev_c2[1]
                if math.hypot(x1 - rx, y1 - ry) <= smooth_eps:
                    use_smooth = True
            if use_smooth:
                cmd = "s"
                payload = _join_nums([
                    _fmt_num(q(x2 - cur_x), decimals),
                    _fmt_num(q(y2 - cur_y), decimals),
                    _fmt_num(dx3, decimals),
                    _fmt_num(dy3, decimals),
                ])
                counts["s"] += 1
            else:
                cmd = "c"
                payload = _join_nums([
                    _fmt_num(q(x1 - cur_x), decimals),
                    _fmt_num(q(y1 - cur_y), decimals),
                    _fmt_num(q(x2 - cur_x), decimals),
                    _fmt_num(q(y2 - cur_y), decimals),
                    _fmt_num(dx3, decimals),
                    _fmt_num(dy3, decimals),
                ])
                counts["c"] += 1
            if cmd == prev_cmd:
                tokens.append(("" if payload.startswith("-") else " ") + payload)
            else:
                tokens.append(cmd + payload)
                prev_cmd = cmd
            prev_c2 = (x2, y2)
            cur_x, cur_y = x3, y3

    if len(tokens) <= 1:
        return "", {}
    tokens.append("z")
    return "".join(tokens), counts


def trace_binary_mask_vtracer(
    mask: np.ndarray,
    viewbox: Sequence[float],
    cfg: LayerTraceConfig,
    decimals: int = 1,
) -> Tuple[str, Dict[str, int]]:
    """Trace one boolean mask using bias-compensated binary vtracer -> compact relative SVG path string."""
    if not np.any(mask):
        return "", {}

    H, W = mask.shape
    vb_w, vb_h = float(viewbox[2]), float(viewbox[3])

    # Dual-resolution resize (preserving exact aspect ratio)
    scale_to_trace = float(cfg.scale_size) / float(max(W, H))
    tw = max(4, int(round(W * scale_to_trace)))
    th = max(4, int(round(H * scale_to_trace)))

    im_m = Image.fromarray((mask.astype(np.uint8) * 255), mode="L")
    if (tw, th) != (W, H):
        im_m = im_m.resize((tw, th), Image.Resampling.BILINEAR)
    if cfg.blur_radius > 0.0:
        im_m = im_m.filter(ImageFilter.GaussianBlur(radius=cfg.blur_radius))

    # Threshold after Gaussian blur to cancel visioncortex +0.4px outward staircase bias
    arr = np.asarray(im_m, dtype=np.uint8)
    fg_bin = arr >= cfg.threshold
    if not np.any(fg_bin):
        return "", {}

    # CRITICAL vtracer binary-mode contract: black (#000000) foreground on opaque white (#FFFFFF) RGB PNG!
    bw = np.where(fg_bin, 0, 255).astype(np.uint8)
    rgb_png = Image.fromarray(bw, mode="L").convert("RGB")
    buf = io.BytesIO()
    rgb_png.save(buf, format="PNG", compress_level=1)

    svg_raw = vtracer.convert_raw_image_to_svg(
        buf.getvalue(),
        img_format="png",
        colormode="binary",
        hierarchical="stacked",
        mode="spline",
        filter_speckle=int(cfg.filter_speckle),
        corner_threshold=int(cfg.corner_threshold),
        length_threshold=float(cfg.length_threshold),
        max_iterations=int(cfg.max_iterations),
        splice_threshold=int(cfg.splice_threshold),
        path_precision=int(cfg.path_precision),
    )

    subpaths = _parse_vtracer_subpaths(svg_raw, scale_x=vb_w / float(tw), scale_y=vb_h / float(th))
    if not subpaths:
        return "", {}

    d_parts: List[str] = []
    total_counts: Dict[str, int] = {"M": 0, "c": 0, "s": 0, "l": 0, "h": 0, "v": 0, "z": 0}
    for start, segs in subpaths:
        sub_d, sub_c = _simplify_and_encode_subpath(
            start,
            segs,
            line_eps=cfg.line_eps,
            smooth_eps=cfg.smooth_eps,
            min_subpath_u=cfg.min_subpath_u,
            decimals=decimals,
        )
        if sub_d:
            d_parts.append(sub_d)
            for k, v in sub_c.items():
                total_counts[k] = total_counts.get(k, 0) + v

    return "".join(d_parts), total_counts


def trace_component(
    comp_mask: np.ndarray,
    pigments: Dict[str, Any],
    viewbox: Sequence[float],
    preset: FamilyPreset,
    part_meta: Optional[Dict[str, Any]] = None,
) -> Tuple[List[str], Dict[str, int]]:
    """Trace all stacked semantic layers of one component (base -> washes -> accents -> highlights -> ink)."""
    part_meta = part_meta or {}
    elems: List[str] = []
    totals: Dict[str, int] = {"M": 0, "c": 0, "s": 0, "l": 0, "h": 0, "v": 0, "z": 0}

    def _accum(c: Dict[str, int]) -> None:
        for k, v in c.items():
            totals[k] = totals.get(k, 0) + v

    if not np.any(comp_mask):
        return elems, totals

    # 1. Base silhouette layer (owns exact outer contour + open voids)
    d_base, c_base = trace_binary_mask_vtracer(comp_mask, viewbox, preset.trace_base)
    if not d_base:
        return elems, totals
    _accum(c_base)

    layer_specs = pigments["layer_specs"]
    p_masks = pigments["masks"]
    base_layer_name = part_meta.get("base_layer", "wash")
    if base_layer_name not in layer_specs:
        base_layer_name = "wash"
    base_fill = layer_specs[base_layer_name]["fill"]
    elems.append(f'<path d="{d_base}" fill="{base_fill}"/>')

    # Trace group -> LayerTraceConfig mapping
    cfg_map = {
        "base": preset.trace_base,
        "wash": preset.trace_wash,
        "highlight": preset.trace_highlight,
        "accent": preset.trace_accent,
        "ink": preset.trace_ink,
    }

    # Optional per-component allowed/excluded layers (e.g., head doesn't need sage; body coils don't need crimson)
    exclude_layers = set(part_meta.get("exclude_layers", []))
    min_layer_px = int(part_meta.get("min_layer_px", 12))

    # 2. Stacked interior pigment & ink layers (strictly gated inside comp_mask so silhouette NEVER bloats!)
    for lname in pigments["layer_order"]:
        if lname == base_layer_name or lname in exclude_layers:
            continue
        if lname not in p_masks or lname not in layer_specs:
            continue
        lspec = layer_specs[lname]
        # Strict gating inside comp_mask prevents outward FP bloat on ink and washes!
        sub_mask = p_masks[lname] & comp_mask
        if int(np.sum(sub_mask)) < min_layer_px:
            continue

        tcfg = cfg_map.get(lspec.get("trace_group", "wash"), preset.trace_wash)
        # Apply optional per-component ink detail boost (e.g. crisp head/wing detail)
        if lname == "ink" and part_meta.get("fine_detail", False):
            tcfg = replace(
                tcfg,
                scale_size=max(tcfg.scale_size, 800),
                filter_speckle=max(3, tcfg.filter_speckle - 1),
                length_threshold=max(4.5, tcfg.length_threshold - 0.6),
            )
        d_layer, c_layer = trace_binary_mask_vtracer(sub_mask, viewbox, tcfg)
        if not d_layer:
            continue
        _accum(c_layer)

        fill_val = lspec["fill"]
        op = float(lspec.get("opacity", 1.0))
        op_attr = "" if op >= 0.99 else f' opacity="{op:.2f}"'
        role_attr = ' data-role="ink"' if lname == "ink" else ""
        elems.append(f'<path d="{d_layer}" fill="{fill_val}"{op_attr}{role_attr}/>')

    return elems, totals


def wrap_standalone_component_svg(
    inner_xml: str,
    comp_mask: np.ndarray,
    viewbox: Sequence[float],
    px_per_u: float,
    title: str,
    pad_u: float = 3.0,
) -> Tuple[str, List[float]]:
    """Wrap one component in a tight-cropped standalone SVG with explicit width/height and aria-label."""
    vb_w, vb_h = float(viewbox[2]), float(viewbox[3])
    ys, xs = np.where(comp_mask)
    if len(xs) == 0:
        x0, y0, w, h = 0.0, 0.0, vb_w, vb_h
    else:
        x0 = max(0.0, math.floor(float(xs.min()) / px_per_u - pad_u))
        y0 = max(0.0, math.floor(float(ys.min()) / px_per_u - pad_u))
        x1 = min(vb_w, math.ceil(float(xs.max() + 1) / px_per_u + pad_u))
        y1 = min(vb_h, math.ceil(float(ys.max() + 1) / px_per_u + pad_u))
        w, h = max(1.0, x1 - x0), max(1.0, y1 - y0)

    vb_str = f"{_fmt_num(x0, 1)} {_fmt_num(y0, 1)} {_fmt_num(w, 1)} {_fmt_num(h, 1)}"
    w_str, h_str = _fmt_num(w, 1), _fmt_num(h, 1)
    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb_str}" width="{w_str}" height="{h_str}"'
        f' role="img" aria-label="{title}">\n{inner_xml}\n</svg>\n'
    )
    ET.fromstring(svg)
    return svg, [x0, y0, w, h]


def _anim_tag(cfg: Dict[str, Any], animated: bool) -> str:
    if not animated or not cfg:
        return ""
    atype = cfg.get("type", "rotate")
    vals = cfg["values"]
    kt = cfg["keyTimes"]
    ks = cfg["keySplines"]
    dur = cfg["dur"]
    return (
        f'<animateTransform attributeName="transform" type="{atype}" values="{vals}"'
        f' keyTimes="{kt}" calcMode="spline" keySplines="{ks}" dur="{dur}" repeatCount="indefinite"/>'
    )


def _pivot_wrap(gid: str, pivot: Optional[Sequence[float]], anim_xml: str, inner: str, extra_tf: str = "") -> str:
    if not pivot:
        return f'<g id="{gid}">{anim_xml}\n{inner}\n</g>'
    px, py = pivot[0], pivot[1]
    tf_attr = f' transform="{extra_tf}"' if extra_tf else ""
    return (
        f'<g transform="translate({px},{py})"><g id="{gid}"{tf_attr}>{anim_xml}'
        f'<g transform="translate({-px},{-py})">\n{inner}\n</g></g></g>'
    )


def build_master_and_animated_svg(
    traced_parts: Dict[str, List[str]],
    traced_subparts: Dict[str, Dict[str, List[str]]],
    order: List[str],
    viewbox: Sequence[float],
    spec: Dict[str, Any],
    animated: bool = False,
) -> str:
    """Assemble Stage A static master or Stage C SMIL animated SVG from traced components (zero <use href=>)."""
    vb_w, vb_h = float(viewbox[2]), float(viewbox[3])
    w_s, h_s = _fmt_num(vb_w, 1), _fmt_num(vb_h, 1)
    vb_str = f"0 0 {w_s} {h_s}"
    slug = spec.get("specimen", "specimen")
    root_id = spec.get("root_id", f"{slug}-root")
    aria = spec.get("aria_label", spec.get("title", f"Traced historical {slug}"))
    anim_cfg = spec.get("animation", {})
    nodes_cfg = anim_cfg.get("nodes", {})

    def render_part_content(pid: str) -> str:
        if pid in traced_subparts:
            sub_blocks = []
            for sp_id, sp_elems in traced_subparts[pid].items():
                scfg = nodes_cfg.get(sp_id, {})
                gid = scfg.get("gid", sp_id)
                body = "\n".join(sp_elems)
                # Optional blink lid inside skull subpart during Stage C animation
                if animated and scfg.get("lid"):
                    lid = scfg["lid"]
                    l_anim = _anim_tag(lid.get("anim", {}), animated=True)
                    lp = lid["pivot"]
                    lid_xml = (
                        f'<g transform="translate({lp[0]},{lp[1]})">'
                        f'<g id="{lid["gid"]}" transform="{lid.get("init_transform", "scale(1 .05)")}">'
                        f'{l_anim}{lid["path"]}</g></g>'
                    )
                    body = f"{body}\n{lid_xml}"
                sub_blocks.append(_pivot_wrap(gid, scfg.get("pivot"), _anim_tag(scfg.get("anim", {}), animated), body))
            return "\n".join(sub_blocks)
        return "\n".join(traced_parts.get(pid, []))

    # Support hierarchical grouping from spec["assembly_tree"] if provided (preserves exact Roundel Stage C rig!)
    tree = spec.get("assembly_tree")
    if tree:
        def render_node(node: Dict[str, Any]) -> str:
            pid = node.get("part")
            ncfg = nodes_cfg.get(node.get("id", pid or ""), {})
            gid = node.get("gid", ncfg.get("gid", node.get("id", pid)))
            parts_xml: List[str] = []
            if pid:
                parts_xml.append(render_part_content(pid))
            for child in node.get("children", []):
                parts_xml.append(render_node(child))
            inner = "\n".join(x for x in parts_xml if x)
            pivot = node.get("pivot", ncfg.get("pivot"))
            anim_xml = _anim_tag(node.get("anim", ncfg.get("anim", {})), animated)
            return _pivot_wrap(gid, pivot, anim_xml, inner)

        body_xml = "\n".join(render_node(n) for n in tree)
    else:
        blocks = []
        for pid in order:
            ncfg = nodes_cfg.get(pid, {})
            gid = ncfg.get("gid", f"{slug}-{pid}")
            blocks.append(
                _pivot_wrap(gid, ncfg.get("pivot"), _anim_tag(ncfg.get("anim", {}), animated), render_part_content(pid))
            )
        body_xml = "\n".join(blocks)

    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb_str}" width="{w_s}" height="{h_s}"'
        f' role="img" aria-label="{aria}">\n'
        f'<g id="{root_id}">\n{body_xml}\n</g>\n</svg>\n'
    )
    ET.fromstring(svg)
    return svg


def pareto_sweep_trace_config(
    fg_mask: np.ndarray,
    ink_mask: np.ndarray,
    rgb: np.ndarray,
    seg: Dict[str, Any],
    pigments: Dict[str, Any],
    viewbox: Sequence[float],
    preset: FamilyPreset,
    spec: Dict[str, Any],
    evaluator: Any,
) -> Tuple[FamilyPreset, List[Dict[str, Any]]]:
    """Automated Pareto search over bias-compensation and vtracer spline compaction parameters."""
    candidates = [
        # (name, base_blur, base_thr, base_lt, base_fs, base_eps, det_lt, det_eps, ink_blur, ink_thr, ink_lt, ink_fs)
        ("crisp-174", 1.18, 174, 5.5, 6, 0.10, 10.0, 0.20, 0.62, 152, 5.8, 5),
        ("crisp-178", 1.22, 178, 5.6, 6, 0.10, 10.0, 0.20, 0.62, 152, 6.0, 5),
        ("crisp-182", 1.26, 182, 5.6, 6, 0.10, 10.2, 0.20, 0.65, 154, 6.0, 5),
        ("crisp-186", 1.30, 186, 5.8, 6, 0.11, 10.5, 0.21, 0.68, 156, 6.2, 5),
        ("fine-176",  1.15, 176, 5.0, 5, 0.09,  9.5, 0.19, 0.58, 150, 5.5, 4),
    ]

    max_raw_kb = float(spec.get("sweep_max_raw_kb", 60.0))
    max_gzip_kb = float(spec.get("sweep_max_gzip_kb", 21.5))

    results: List[Dict[str, Any]] = []
    best_score = -1e9
    best_preset: Optional[FamilyPreset] = None
    fallback_score = -1e9
    fallback_preset = preset

    for name, b_blur, b_thr, b_lt, b_fs, b_eps, d_lt, d_eps, i_blur, i_thr, i_lt, i_fs in candidates:
        cand_preset = replace(
            preset,
            trace_base=replace(
                preset.trace_base,
                blur_radius=b_blur,
                threshold=b_thr,
                length_threshold=b_lt,
                filter_speckle=b_fs,
                line_eps=b_eps,
            ),
            trace_wash=replace(preset.trace_wash, length_threshold=d_lt, line_eps=d_eps),
            trace_accent=replace(preset.trace_accent, length_threshold=max(7.0, d_lt - 2.0), line_eps=max(0.16, d_eps - 0.02)),
            trace_ink=replace(
                preset.trace_ink,
                blur_radius=i_blur,
                threshold=i_thr,
                length_threshold=i_lt,
                filter_speckle=i_fs,
            ),
        )

        traced_parts: Dict[str, List[str]] = {}
        traced_subs: Dict[str, Dict[str, List[str]]] = {}
        for cid in seg["order"]:
            pmeta = seg["part_meta"].get(cid, {})
            if cid in seg["subparts"]:
                traced_subs[cid] = {}
                for sp_id, sp_m in seg["subparts"][cid].items():
                    sp_meta = {**pmeta, "fine_detail": True}
                    elems, _ = trace_component(sp_m, pigments, viewbox, cand_preset, sp_meta)
                    traced_subs[cid][sp_id] = elems
            else:
                elems, _ = trace_component(seg["parts"][cid], pigments, viewbox, cand_preset, pmeta)
                traced_parts[cid] = elems

        svg_s = build_master_and_animated_svg(traced_parts, traced_subs, seg["order"], viewbox, spec, animated=False)
        m = evaluator.evaluate_svg(svg_s, fg_mask, ink_mask, rgb, viewbox)
        leg = m.get("silhouette_iou_legacy_pct") or m["silhouette_iou_true_pct"]
        score = m["silhouette_iou_true_pct"] + 0.35 * leg + 0.06 * m["ink_iou_pct"] - 0.03 * m["raw_kb"]
        row = {"config": name, "score": round(score, 3), **m}
        results.append(row)
        if score > fallback_score:
            fallback_score = score
            fallback_preset = cand_preset
        if score > best_score and m["raw_kb"] <= max_raw_kb and m["gzip_kb"] <= max_gzip_kb:
            best_score = score
            best_preset = cand_preset

    return (best_preset if best_preset is not None else fallback_preset), results
