#!/usr/bin/env python3
"""Family A2 & B gilded illuminated manuscript presets and batch runner.

Registers reusable polychrome gilded-roundel (Family A2) and gilded-initial
(Family B) presets via ``lib.presets.register_preset`` and executes
``run_specimen.run_specimen`` without modifying core library modules.
"""
from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage
from skimage.color import rgb2lab

TRACED_DIR = Path(__file__).resolve().parent
if str(TRACED_DIR) not in sys.path:
    sys.path.insert(0, str(TRACED_DIR))

from lib.ground import (
    _apply_spec_polygons,
    _detect_circular_roundel,
    _kmeans_lab,
    _preserve_open_voids,
    _weighted_lab_dist,
)
from lib.pigments import _disk, _extract_polychrome_ink, _keep_cc_min, _rgb_to_hex
from lib.presets import (
    FamilyPreset,
    GroundConfig,
    LayerTraceConfig,
    PigmentConfig,
    register_preset,
)
from run_specimen import run_specimen


def _draw_poly_u(pts_u: List[List[float]], w: int, h: int, px_per_u: float) -> np.ndarray:
    m = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(m)
    draw.polygon([(p[0] * px_per_u, p[1] * px_per_u) for p in pts_u], fill=255)
    return np.asarray(m, dtype=np.uint8) > 127


def _px_per_u_from_rgb(rgb: np.ndarray, preset: FamilyPreset) -> float:
    H, W = rgb.shape[:2]
    vb_scale = preset.viewbox_max / float(max(W, H))
    vb_w = round(W * vb_scale, 2)
    if abs(vb_w - round(vb_w)) < 0.05:
        vb_w = float(round(vb_w))
    return W / vb_w


def _extract_gilded_roundel_a2_ground(
    rgb: np.ndarray,
    spec: Dict[str, Any],
    preset: FamilyPreset,
) -> Dict[str, Any]:
    """Family A2 gilded roundel ground extraction with specular-belly protection."""
    cfg = preset.ground
    H, W = rgb.shape[:2]
    px_per_u = _px_per_u_from_rgb(rgb, preset)
    lab = rgb2lab(rgb.astype(np.float32) / 255.0).astype(np.float32)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    frame_info = _detect_circular_roundel(lab, cfg)
    if not frame_info.get("detected"):
        raise RuntimeError("Failed to detect circular gilded roundel frame.")

    r_g = frame_info["r_g"]
    r_field_map = frame_info["r_field_map"]
    inside_field = r_g <= (r_field_map - float(spec.get("roundel_inset_px", 1.0)))

    # Sample burnished gold leaf strictly inside the circular field
    gold_seed = (
        inside_field
        & (b > 17.0)
        & (a > -4.0)
        & (a < 15.0)
        & (L > 24.0)
        & (L < 88.0)
        & (b > 1.50 * np.maximum(a, 1.0))
    )
    seed_pts = lab[gold_seed]
    step = max(1, len(seed_pts) // 6000)
    bg_centers = _kmeans_lab(seed_pts[::step], k=cfg.bg_clusters)
    bg_dist = _weighted_lab_dist(lab, bg_centers, cfg.lightness_weight, cfg.chroma_weight)

    # Protect neutral iron-gall ink, ultramarine blue, sage green, terracotta/orange, and dusty rose stone
    is_ink = (L < 26.0) & (C < 20.0)
    is_blue = (b < -6.0) & (C > 10.0)
    is_sage = ((a < -1.0) & (b < 24.0) & (L > 30.0)) | ((a < -4.0) & (b < 32.0) & (L > 30.0))
    is_warm_pigment = (a > 12.0) & (b < 58.0) & (L > 18.0) & (L < 80.0) & (a > 0.42 * np.maximum(b, 1.0))
    is_ivory_head = (L > 72.0) & (b < 14.0) & (C < 18.0)

    is_gold_core = (
        (
            (bg_dist < cfg.bg_dist_thresh)
            | ((b > 21.0) & (b > 1.55 * np.maximum(a, 1.0)) & (L > 20.0))
        )
        & (~is_ink)
        & (~is_blue)
        & (~is_sage)
        & (~is_warm_pigment)
        & (~is_ivory_head)
    )

    # Specular gold glare ONLY when directly connected to gold background (never jumps across ink outlines!)
    is_spec_cand = ((L > 82.0) & (b > 14.0) & (a < 8.0)) | ((L > 91.0) & (b > 8.0) & (a < 8.0))
    lbl_gl, _ = ndimage.label(is_gold_core | is_spec_cand)
    gold_lbls = np.unique(lbl_gl[is_gold_core & inside_field])
    gold_lbls = gold_lbls[gold_lbls > 0]
    is_gold_bg = is_gold_core | (is_spec_cand & np.isin(lbl_gl, gold_lbls))

    fg_int = inside_field & (~is_gold_bg)
    # Close tiny 1px breaks along dark outlines and fill small specular highlights trapped inside body
    fg_closed = ndimage.binary_closing(fg_int, structure=_disk(1)) & inside_field
    fg_closed = _apply_spec_polygons(fg_closed, spec, px_per_u)

    # Keep main figure (and any substantial connected foreground pieces)
    lbl_fg, n_fg = ndimage.label(fg_closed)
    if n_fg > 0:
        areas = ndimage.sum(fg_closed, lbl_fg, index=np.arange(1, n_fg + 1))
        min_area = max(250, int(round(cfg.min_fg_area_frac * H * W)))
        largest = int(np.argmax(areas)) + 1
        keep_ids = [largest]
        if spec.get("multi_component_fg", False):
            keep_ids = [i + 1 for i, ar in enumerate(areas) if ar >= min_area]
        fg_raw = np.isin(lbl_fg, keep_ids)
    else:
        fg_raw = fg_closed

    # Preserve genuine open voids (e.g., open mouth showing gold leaf), fill tiny interior pinholes/glare specks
    fg_clean, ext_bg, void_mask = _preserve_open_voids(fg_raw, bg_dist, cfg)

    return {
        "fg_mask": fg_clean,
        "bg_mask": ext_bg,
        "void_mask": void_mask,
        "bg_dist": bg_dist,
        "lab": lab,
        "frame_info": {
            "detected": True,
            "cx": frame_info["cx"],
            "cy": frame_info["cy"],
            "inner_r": frame_info["inner_r"],
            "rim_inner_r": frame_info["rim_inner_r"],
            "outer_r": frame_info["outer_r"],
            "protrusion_angles": [],
        },
    }


def _extract_gilded_initial_b_ground(
    rgb: np.ndarray,
    spec: Dict[str, Any],
    preset: FamilyPreset,
) -> Dict[str, Any]:
    """Family B dual-ground (vellum margin + hammered/burnished gold leaf) extractor."""
    cfg = preset.ground
    H, W = rgb.shape[:2]
    px_per_u = _px_per_u_from_rgb(rgb, preset)
    lab = rgb2lab(rgb.astype(np.float32) / 255.0).astype(np.float32)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    # 1. Sample pale vellum/parchment margin from image perimeter
    bw = max(5, int(0.03 * min(H, W)))
    border = np.zeros((H, W), dtype=bool)
    border[:bw, :] = True
    border[-bw:, :] = True
    border[:, :bw] = True
    border[:, -bw:] = True
    vellum_seeds = lab[border & (L > 58.0) & (C < 18.0)]
    if len(vellum_seeds) < 200:
        vellum_seeds = lab[border]
    v_step = max(1, len(vellum_seeds) // 3000)
    v_centers = _kmeans_lab(vellum_seeds[::v_step], k=2)
    v_dist = _weighted_lab_dist(lab, v_centers, w_l=0.85, w_c=1.6)

    # 2. Sample burnished/hammered gold leaf inside the initial's interior box
    gbox = spec.get("gold_sample_box", [0.15, 0.12, 0.85, 0.90])
    gx0, gy0 = int(round(gbox[0] * W)), int(round(gbox[1] * H))
    gx1, gy1 = int(round(gbox[2] * W)), int(round(gbox[3] * H))
    in_gbox = np.zeros((H, W), dtype=bool)
    in_gbox[max(0, gy0) : min(H, gy1), max(0, gx0) : min(W, gx1)] = True

    gold_seed_mask = (
        in_gbox
        & (b > 13.0)
        & (a > -5.0)
        & (a < 14.0)
        & (L > 22.0)
        & (L < 92.0)
        & (b > 1.35 * np.maximum(a, 1.0))
    )
    if int(np.sum(gold_seed_mask)) < 300:
        gold_seed_mask = in_gbox & (b > 10.0) & (L > 20.0) & (a < 15.0)
    g_seeds = lab[gold_seed_mask]
    g_step = max(1, len(g_seeds) // 5000)
    g_centers = _kmeans_lab(g_seeds[::g_step], k=cfg.bg_clusters)
    g_dist = _weighted_lab_dist(lab, g_centers, w_l=cfg.lightness_weight, w_c=cfg.chroma_weight)

    bg_dist = np.minimum(v_dist, g_dist)

    # Protect authentic manuscript pigments (ink, lapis blue, emerald/mint green, vermilion/ochre, rose-mauve)
    ink_l_thr = float(spec.get("ground_ink_l_max", 24.0))
    is_ink = (L < ink_l_thr) & (C < 16.0)
    is_blue = (b < -7.0) & (C > 10.0)
    is_green = (a < -3.5) & (b > -8.0) & (L > 18.0) & (L < 86.0) & (C > 8.5)
    is_crimson = (a > 14.0) & (b > 6.0) & (L > 18.0) & (L < 76.0)
    is_mauve = (a > 5.5) & (b < 16.0) & (L > 24.0) & (L < 74.0)

    # Reject outer vellum (+ optional faint pale-green wash halo outside rectangular frame)
    v_thresh = float(spec.get("vellum_dist_thresh", 16.5))
    is_vellum = (v_dist < v_thresh) & (~is_ink) & (~is_blue) & (~is_crimson) & (~is_mauve)
    if spec.get("reject_faint_margin_halo", False):
        # Pale desaturated mint wash painted around the outside of S-initial border (L > 64, C < 12)
        faint_halo = (L > 60.0) & (C < 13.5) & (a < 1.0) & (a > -10.0) & (b > -4.0) & (b < 12.0)
        is_vellum = is_vellum | (faint_halo & (~is_ink))
    else:
        is_vellum = is_vellum & (~is_green)

    # Flood-fill vellum strictly from image borders so interior white lead highlights stay intact!
    lbl_v, _ = ndimage.label(is_vellum)
    edge_ids = set(np.unique(lbl_v[0, :]).tolist())
    edge_ids.update(np.unique(lbl_v[-1, :]).tolist())
    edge_ids.update(np.unique(lbl_v[:, 0]).tolist())
    edge_ids.update(np.unique(lbl_v[:, -1]).tolist())
    edge_ids.discard(0)
    ext_vellum = np.isin(lbl_v, list(edge_ids))

    # Reject burnished/hammered gold leaf inside the initial (including specular glints connected to gold)
    g_thresh = float(spec.get("gold_dist_thresh", cfg.bg_dist_thresh))
    is_gold_core = (
        in_gbox
        & (
            (g_dist < g_thresh)
            | ((b > 17.0) & (b > 1.45 * np.maximum(a, 1.0)) & (L > 18.0))
        )
        & (~is_ink)
        & (~is_blue)
        & (~is_green)
        & (~is_crimson)
        & (~is_mauve)
    )
    is_spec = in_gbox & ((L > 82.0) & (b > 8.0) & (a < 8.0))
    # Also reject dark hammered-gold dimples completely surrounded by gold (if hammered_gold=True)
    if spec.get("hammered_gold", False):
        gold_seed_dil = ndimage.binary_dilation(is_gold_core, iterations=2)
        dark_dimple = in_gbox & (L < 38.0) & (b > 7.0) & (b > 1.2 * np.maximum(a, 1.0)) & gold_seed_dil
        is_gold_core = is_gold_core | dark_dimple

    lbl_g, _ = ndimage.label(is_gold_core | is_spec)
    g_lbls = np.unique(lbl_g[is_gold_core])
    g_lbls = g_lbls[g_lbls > 0]
    is_gold_bg = is_gold_core | (is_spec & np.isin(lbl_g, g_lbls))

    # Optional rectangular border frame exclusion (strips orange/vermilion or pink diapered outer border
    # ONLY outside explicit protrusion zones where dragon head/wings/claws cross the frame!)
    fg_raw = (~ext_vellum) & (~is_gold_bg)
    if spec.get("frame_rect_u"):
        fx0, fy0, fx1, fy1, fw = [float(v) for v in spec["frame_rect_u"]]
        yy, xx = np.mgrid[0:H, 0:W]
        xu = xx / px_per_u
        yu = yy / px_per_u
        in_outer_rect = (xu >= fx0) & (xu <= fx1) & (yu >= fy0) & (yu <= fy1)
        in_inner_rect = (xu >= fx0 + fw) & (xu <= fx1 - fw) & (yu >= fy0 + fw) & (yu <= fy1 - fw)
        frame_band = in_outer_rect & (~in_inner_rect)
        # Keep protruding dragon parts defined in spec["protrusion_polygons"]
        prot_mask = np.zeros((H, W), dtype=bool)
        for poly in spec.get("protrusion_polygons", []):
            prot_mask |= _draw_poly_u(poly, W, H, px_per_u)
        fg_raw &= ~(frame_band & (~prot_mask))

    # Apply declarative include/exclude polygons (strips header rubric, REGORIUS capitals, penwork, etc.)
    fg_raw = _apply_spec_polygons(fg_raw, spec, px_per_u)

    # Morphological cleanup: close 1px gaps along thin ink outlines, drop tiny specks
    fg_raw = ndimage.binary_closing(fg_raw, structure=_disk(1))
    fg_raw = _apply_spec_polygons(fg_raw, spec, px_per_u)

    lbl_fg, n_fg = ndimage.label(fg_raw)
    if n_fg > 0:
        areas = ndimage.sum(fg_raw, lbl_fg, index=np.arange(1, n_fg + 1))
        min_area = max(180, int(round(cfg.min_fg_area_frac * H * W)))
        largest = int(np.argmax(areas)) + 1
        if spec.get("multi_component_fg", False):
            keep_ids = [i + 1 for i, ar in enumerate(areas) if ar >= min_area]
            if not keep_ids:
                keep_ids = [largest]
        else:
            keep_ids = [largest]
        fg_raw = np.isin(lbl_fg, keep_ids)

    fg_clean, ext_bg, void_mask = _preserve_open_voids(fg_raw, bg_dist, cfg)

    return {
        "fg_mask": fg_clean,
        "bg_mask": ext_bg,
        "void_mask": void_mask,
        "bg_dist": bg_dist,
        "lab": lab,
        "frame_info": {"detected": bool(spec.get("frame_rect_u"))},
    }


def _extract_gilded_polychrome_pigments(
    rgb: np.ndarray,
    fg_mask: np.ndarray,
    spec: Dict[str, Any],
    preset: FamilyPreset,
) -> Dict[str, Any]:
    """Generalizable medieval mineral-pigment classifier for gilded manuscripts (Families A2 & B).

    Separates authentic manuscript mineral pigments in CIELAB:
      - ink (iron-gall black/brown outlines & hatch)
      - blue (lapis ultramarine / cobalt)
      - green (verdigris / malachite / sage / mint)
      - crimson (vermilion / minium / burnt-orange / ochre)
      - mauve (folium / rose-mauve / dusty stone)
      - highlight (lead-white / ivory / pearl dots)
    Maps ``spec['primary_wash_channel']`` to recolorable ``var(--wash)`` / ``var(--wash2)``.
    """
    cfg = preset.pigments
    H, W = fg_mask.shape
    lab = rgb2lab(rgb.astype(np.float32) / 255.0).astype(np.float32)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    # 1. Multi-scale iron-gall ink on sharp unfiltered LAB
    ink = _extract_polychrome_ink(lab, fg_mask, cfg)
    non_ink = fg_mask & (~ink)

    # 2. Edge-preserving median filter for smooth gouache pigment regions
    med_k = max(3, (cfg.prefilter_radius * 2) | 1)
    lab_s = np.stack([ndimage.median_filter(lab[..., c], size=med_k) for c in range(3)], axis=-1)
    Ls, as_, bs_ = lab_s[..., 0], lab_s[..., 1], lab_s[..., 2]
    Cs = np.hypot(as_, bs_)

    open_s = _disk(cfg.wash_open_r)
    close_s = _disk(cfg.wash_close_r)
    fg_n = max(1, int(np.sum(fg_mask)))
    min_chan_px = max(cfg.min_wash_area, int(0.0018 * fg_n))

    def _clean_chan(raw_m: np.ndarray, min_px: int = min_chan_px) -> np.ndarray:
        if int(np.sum(raw_m)) < min_px:
            return np.zeros((H, W), dtype=bool)
        m = ndimage.binary_closing(ndimage.binary_opening(raw_m, structure=open_s), structure=close_s) & fg_mask
        return _keep_cc_min(m, cfg.min_wash_area)

    # 3. Crisp lead-white / ivory highlights + local bright-peak pearl dots (unfiltered L*)
    local_bg_L = ndimage.gaussian_filter(L, sigma=3.5)
    bright_peak = non_ink & ((L - local_bg_L) > 12.0) & (L > 54.0) & (C < 20.0)
    hi_global = non_ink & (L > float(spec.get("highlight_l_min", 73.0))) & (C < 16.0)
    highlight_clean = _keep_cc_min(hi_global | bright_peak, cfg.min_highlight_area)
    hi_rgb_med = np.median(rgb[highlight_clean], axis=0) if np.any(highlight_clean) else np.array([240, 234, 222])
    if float(np.mean(hi_rgb_med)) < 212.0:
        hi_rgb_med = np.clip(hi_rgb_med * 1.10, 0, 248)
    hi_hex = _rgb_to_hex(hi_rgb_med)

    # 4. Separate distinct medieval mineral pigment hues on non-ink pixels
    blue_raw = non_ink & (bs_ < -7.5) & (Cs > 10.0)
    green_raw = non_ink & (~blue_raw) & (as_ < -2.0) & (bs_ > -8.0) & (Ls > 18.0) & (Cs > 5.5)
    crimson_raw = (
        non_ink
        & (~blue_raw)
        & (~green_raw)
        & (
            ((as_ > 15.0) & (bs_ > 8.0) & (Ls > 24.0) & (Ls < 76.0))
            | ((as_ > 9.0) & (bs_ > 22.0) & (Ls > 32.0) & (Ls < 78.0))  # golden-ochre / orange beasts
        )
    )
    mauve_raw = (
        non_ink
        & (~blue_raw)
        & (~green_raw)
        & (~crimson_raw)
        & (~highlight_clean)
        & (Ls > 22.0)
        & (Ls < 80.0)
    )

    blue_m = _clean_chan(blue_raw)
    green_m = _clean_chan(green_raw)
    crimson_m = _clean_chan(crimson_raw)
    mauve_m = _clean_chan(mauve_raw)

    # 5. Primary recolorable wash channel -> split into wash (mid) + wash2 (deep shadow)
    primary_ch = spec.get("primary_wash_channel", "blue")
    chan_masks: Dict[str, np.ndarray] = {
        "blue": blue_m,
        "green": green_m,
        "crimson": crimson_m,
        "mauve": mauve_m,
    }
    prim_m = chan_masks.get(primary_ch, np.zeros((H, W), dtype=bool))
    if not np.any(prim_m):
        # Fallback to largest pigment channel
        primary_ch = max(chan_masks, key=lambda k: int(np.sum(chan_masks[k])))
        prim_m = chan_masks[primary_ch]

    prim_L = Ls[prim_m & (~ink)]
    if len(prim_L) > 50:
        q_dark = float(np.percentile(prim_L, float(spec.get("wash2_percentile", 32.0))))
        wash2_raw = prim_m & (Ls <= q_dark) & (~ink)
        # Also include local dark markings inside primary wash (e.g. dark green O/X jewel rings on S-initial)
        local_prim_dark = prim_m & ((local_bg_L - L) > 6.5) & (L < q_dark + 12.0) & (~ink)
        wash2_clean = _keep_cc_min(
            ndimage.binary_closing(wash2_raw | local_prim_dark, structure=_disk(1)) & fg_mask,
            max(10, cfg.min_wash_area // 2),
        )
        wash_mid_m = prim_m & (~wash2_clean) & (~ink)
        if not np.any(wash_mid_m):
            wash_mid_m = prim_m & (~ink)
    else:
        wash2_clean = np.zeros((H, W), dtype=bool)
        wash_mid_m = prim_m & (~ink)

    wash_hex = _rgb_to_hex(np.median(rgb[wash_mid_m], axis=0)) if np.any(wash_mid_m) else "#46648a"
    wash2_hex = _rgb_to_hex(np.median(rgb[wash2_clean], axis=0)) if np.any(wash2_clean) else "#263b5a"
    ink_hex = preset.default_palette.get("ink", "#19100b")

    # Warm mid-tone (e.g., cream/pale-ochre head or pinkish belly highlight)
    warm_mid_m = non_ink & (Ls >= 64.0) & (Ls <= 82.0) & (as_ > 0.0) & (as_ < 16.0) & (bs_ > 2.0) & (bs_ < 20.0)
    warm_mid_m = _clean_chan(warm_mid_m, min_px=max(40, cfg.min_wash_area))
    warm_mid_hex = _rgb_to_hex(np.median(rgb[warm_mid_m], axis=0)) if np.any(warm_mid_m) else "#e6d8c5"

    masks: Dict[str, np.ndarray] = {
        "wash": prim_m,
        "wash2": wash2_clean,
        "warm_mid": warm_mid_m,
        "highlight": highlight_clean,
        "ink": ink,
    }
    palette: Dict[str, str] = {
        "wash": wash_hex,
        "wash2": wash2_hex,
        "warm_mid": warm_mid_hex,
        "highlight": hi_hex,
        "ink": ink_hex,
    }
    layer_specs: Dict[str, Dict[str, Any]] = {
        "wash": {"fill": f"var(--wash,{wash_hex})", "opacity": 1.0, "trace_group": "base"},
        "wash2": {"fill": f"var(--wash2,{wash2_hex})", "opacity": 0.96, "trace_group": "wash"},
        "warm_mid": {"fill": warm_mid_hex, "opacity": 0.96, "trace_group": "wash"},
        "highlight": {"fill": hi_hex, "opacity": 0.96, "trace_group": "highlight"},
        "ink": {"fill": ink_hex, "opacity": 1.0, "trace_group": "ink"},
    }

    # Add every secondary mineral pigment channel with its sampled authentic hex
    layer_order: List[str] = ["wash", "wash2", "warm_mid"]
    for cname, cmask in [("mauve", mauve_m), ("blue", blue_m), ("green", green_m), ("crimson", crimson_m)]:
        if cname == primary_ch:
            # Also expose under its semantic channel name pointing to var(--wash) for component base_layer convenience
            masks[cname] = prim_m
            layer_specs[cname] = {"fill": f"var(--wash,{wash_hex})", "opacity": 1.0, "trace_group": "base"}
            continue
        if np.any(cmask):
            sample_m = cmask & (~ink)
            chex = _rgb_to_hex(np.median(rgb[sample_m], axis=0)) if np.any(sample_m) else "#887766"
            masks[cname] = cmask
            palette[cname] = chex
            layer_specs[cname] = {"fill": chex, "opacity": 0.98, "trace_group": "accent"}
            layer_order.append(cname)

    layer_order.extend(["highlight", "ink"])

    return {
        "ink": ink,
        "masks": masks,
        "palette": palette,
        "layer_order": layer_order,
        "layer_specs": layer_specs,
    }


def register_gilded_presets() -> None:
    """Register Family A2 and Family B gilded illuminated manuscript presets."""
    register_preset(
        "gilded_roundel_polychrome",
        FamilyPreset(
            name="gilded_roundel_polychrome",
            family="A",
            description="12th c. gilded English Bestiary roundel with multi-mineral polychrome gouache (lapis, sage, vermilion, mauve).",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="gilded_roundel",
                frame_shape="circle",
                bg_clusters=2,
                chroma_weight=1.9,
                lightness_weight=0.40,
                bg_dist_thresh=18.0,
                void_area_frac=0.0008,
                void_bg_thresh=12.0,
                pinhole_max_area=480,
                limb_protrusion=False,
                frame_ring_width_frac=0.085,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=6,
                prefilter_radius=2,
                ink_l_max=26.0,
                ink_c_max=20.0,
                ink_valley_delta=12.5,
                ink_valley_l_max=58.0,
                wash_open_r=1,
                wash_close_r=2,
                min_ink_area=5,
                min_highlight_area=4,
                min_wash_area=24,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200,
                blur_radius=1.26,
                threshold=182,
                filter_speckle=6,
                length_threshold=5.8,
                splice_threshold=56,
                line_eps=0.11,
                min_subpath_u=0.40,
            ),
            trace_wash=LayerTraceConfig(
                scale_size=540,
                blur_radius=0.90,
                threshold=155,
                filter_speckle=16,
                length_threshold=9.5,
                splice_threshold=64,
                line_eps=0.19,
                min_subpath_u=0.65,
            ),
            trace_highlight=LayerTraceConfig(
                scale_size=620,
                blur_radius=0.65,
                threshold=148,
                filter_speckle=4,
                length_threshold=6.5,
                splice_threshold=58,
                line_eps=0.15,
                min_subpath_u=0.35,
            ),
            trace_accent=LayerTraceConfig(
                scale_size=620,
                blur_radius=0.80,
                threshold=154,
                filter_speckle=10,
                length_threshold=8.0,
                splice_threshold=62,
                line_eps=0.17,
                min_subpath_u=0.45,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=750,
                blur_radius=0.65,
                threshold=155,
                filter_speckle=4,
                length_threshold=5.8,
                splice_threshold=55,
                line_eps=0.13,
                min_subpath_u=0.35,
            ),
            ground_hook=_extract_gilded_roundel_a2_ground,
            pigment_hook=_extract_gilded_polychrome_pigments,
        ),
    )

    register_preset(
        "gilded_initial_polychrome",
        FamilyPreset(
            name="gilded_initial_polychrome",
            family="B",
            description="Gilded inhabited Romanesque/Gothic initials on dual gold-leaf + parchment ground.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="gilded_initial",
                frame_shape="rect",
                bg_clusters=3,
                chroma_weight=1.75,
                lightness_weight=0.42,
                bg_dist_thresh=17.5,
                void_area_frac=0.00035,
                void_bg_thresh=15.0,
                pinhole_max_area=180,
                limb_protrusion=True,
                min_fg_area_frac=0.002,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=6,
                prefilter_radius=2,
                ink_l_max=25.0,
                ink_c_max=18.0,
                ink_valley_delta=12.0,
                ink_valley_l_max=56.0,
                wash_open_r=1,
                wash_close_r=2,
                min_ink_area=5,
                min_highlight_area=4,
                min_wash_area=24,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200,
                blur_radius=1.22,
                threshold=180,
                filter_speckle=6,
                length_threshold=6.0,
                splice_threshold=58,
                line_eps=0.12,
                min_subpath_u=0.40,
            ),
            trace_wash=LayerTraceConfig(
                scale_size=540,
                blur_radius=0.88,
                threshold=154,
                filter_speckle=16,
                length_threshold=9.5,
                splice_threshold=64,
                line_eps=0.19,
                min_subpath_u=0.65,
            ),
            trace_highlight=LayerTraceConfig(
                scale_size=600,
                blur_radius=0.65,
                threshold=148,
                filter_speckle=4,
                length_threshold=6.8,
                splice_threshold=60,
                line_eps=0.16,
                min_subpath_u=0.40,
            ),
            trace_accent=LayerTraceConfig(
                scale_size=580,
                blur_radius=0.82,
                threshold=154,
                filter_speckle=12,
                length_threshold=8.8,
                splice_threshold=62,
                line_eps=0.19,
                min_subpath_u=0.55,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=720,
                blur_radius=0.68,
                threshold=158,
                filter_speckle=6,
                length_threshold=6.4,
                splice_threshold=58,
                line_eps=0.14,
                min_subpath_u=0.40,
            ),
            ground_hook=_extract_gilded_initial_b_ground,
            pigment_hook=_extract_gilded_polychrome_pigments,
        ),
    )

    register_preset(
        "gilded_interlace_compact",
        FamilyPreset(
            name="gilded_interlace_compact",
            family="B",
            description="High-complexity gilded inhabited initial G with braided interlace tuned for compact SVG byte size.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="gilded_initial",
                frame_shape="rect",
                bg_clusters=3,
                chroma_weight=1.75,
                lightness_weight=0.42,
                bg_dist_thresh=17.5,
                void_area_frac=0.0003,
                void_bg_thresh=15.0,
                pinhole_max_area=140,
                limb_protrusion=True,
                min_fg_area_frac=0.002,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=6,
                prefilter_radius=2,
                ink_l_max=25.0,
                ink_c_max=18.0,
                ink_valley_delta=13.0,
                ink_valley_l_max=54.0,
                wash_open_r=1,
                wash_close_r=2,
                min_ink_area=7,
                min_highlight_area=5,
                min_wash_area=32,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200,
                blur_radius=1.24,
                threshold=182,
                filter_speckle=8,
                length_threshold=6.8,
                splice_threshold=60,
                line_eps=0.14,
                min_subpath_u=0.45,
            ),
            trace_wash=LayerTraceConfig(
                scale_size=500,
                blur_radius=0.95,
                threshold=155,
                filter_speckle=22,
                length_threshold=11.0,
                splice_threshold=66,
                line_eps=0.22,
                min_subpath_u=0.80,
            ),
            trace_highlight=LayerTraceConfig(
                scale_size=540,
                blur_radius=0.70,
                threshold=150,
                filter_speckle=6,
                length_threshold=7.8,
                splice_threshold=62,
                line_eps=0.18,
                min_subpath_u=0.50,
            ),
            trace_accent=LayerTraceConfig(
                scale_size=520,
                blur_radius=0.90,
                threshold=155,
                filter_speckle=18,
                length_threshold=10.5,
                splice_threshold=66,
                line_eps=0.22,
                min_subpath_u=0.70,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=660,
                blur_radius=0.74,
                threshold=162,
                filter_speckle=9,
                length_threshold=7.6,
                splice_threshold=62,
                line_eps=0.17,
                min_subpath_u=0.50,
            ),
            ground_hook=_extract_gilded_initial_b_ground,
            pigment_hook=_extract_gilded_polychrome_pigments,
        ),
    )


DEFAULT_SPECS = [
    TRACED_DIR / "specs" / "aberdeen-serpent-pillar.json",
    TRACED_DIR / "specs" / "romanesque-s-initial.json",
    TRACED_DIR / "specs" / "gothic-initial-interlace.json",
]


def main() -> None:
    register_gilded_presets()
    parser = argparse.ArgumentParser(description="Run Family A2 & B gilded illuminated manuscript extractions.")
    parser.add_argument("specs", nargs="*", help="Optional spec JSON path(s); defaults to all 3 Batch 1 specs.")
    parser.add_argument("--sweep", action="store_true", help="Run Pareto sweep before emitting final SVGs.")
    parser.add_argument("--skip-sheets", action="store_true", help="Skip Playwright QA sheet rendering.")
    args = parser.parse_args()

    spec_paths = [Path(s) for s in args.specs] if args.specs else [p for p in DEFAULT_SPECS if p.exists()]
    for sp in spec_paths:
        res = run_specimen(spec_path=sp, sweep=args.sweep, skip_sheets=args.skip_sheets)
        m = res["metrics"]
        print(f"Specimen: {res['meta']['specimen']} (union_cov={res['union_coverage_pct']:.2f}%)")
        print(
            f"  True IoU: {m['silhouette_iou_true_pct']:.2f}% | Legacy IoU: {m['silhouette_iou_legacy_pct']:.2f}% | "
            f"Ink IoU: {m['ink_iou_pct']:.2f}% | BF1(1u): {m['boundary_f1_1u_pct']:.2f}% | "
            f"Color RMSE: {m['masked_color_rmse']:.2f}"
        )
        print(
            f"  Size: {m['raw_kb']:.2f} KB raw / {m['gzip_kb']:.2f} KB gz | "
            f"Paths: {m['path_count']} | Cmds: {m['command_count']}"
        )


if __name__ == "__main__":
    main()
