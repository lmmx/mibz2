"""Stage 1: Aspect-preserving canvas loading, adaptive CIELAB ground/frame removal, and open-void preservation."""
from __future__ import annotations

import math
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage, optimize
from skimage.color import rgb2lab
from skimage.filters import threshold_otsu, threshold_sauvola

from .presets import FamilyPreset, GroundConfig


def _kmeans_lab(samples: np.ndarray, k: int, n_iter: int = 18, seed: int = 42) -> np.ndarray:
    """Lightweight deterministic K-means in CIELAB space (no external sklearn dependency)."""
    if len(samples) == 0:
        return np.zeros((1, 3), dtype=np.float32)
    k = max(1, min(k, len(samples)))
    rng = np.random.default_rng(seed)
    # K-means++ initialization
    centers = np.empty((k, 3), dtype=np.float32)
    idx0 = int(rng.integers(0, len(samples)))
    centers[0] = samples[idx0]
    for c in range(1, k):
        d2 = np.min(np.sum((samples[:, None, :] - centers[None, :c, :]) ** 2, axis=2), axis=1)
        probs = d2 / max(float(np.sum(d2)), 1e-9)
        centers[c] = samples[int(rng.choice(len(samples), p=probs))]

    for _ in range(n_iter):
        dists = np.sum((samples[:, None, :] - centers[None, :, :]) ** 2, axis=2)
        labels = np.argmin(dists, axis=1)
        new_centers = centers.copy()
        for c in range(k):
            pts = samples[labels == c]
            if len(pts) > 0:
                new_centers[c] = np.mean(pts, axis=0)
        if np.max(np.abs(new_centers - centers)) < 0.05:
            centers = new_centers
            break
        centers = new_centers
    return centers


def load_working_image(
    src_path: Path | str,
    preset: FamilyPreset,
    spec: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Load source image, apply optional normalized ROI crop, and resize preserving aspect ratio."""
    spec = spec or {}
    im = Image.open(src_path).convert("RGB")
    orig_w, orig_h = im.size

    crop_box = (0, 0, orig_w, orig_h)
    roi = spec.get("roi")
    if roi and len(roi) == 4:
        x0 = max(0, min(orig_w - 1, int(round(roi[0] * orig_w))))
        y0 = max(0, min(orig_h - 1, int(round(roi[1] * orig_h))))
        x1 = max(x0 + 1, min(orig_w, int(round(roi[2] * orig_w))))
        y1 = max(y0 + 1, min(orig_h, int(round(roi[3] * orig_h))))
        crop_box = (x0, y0, x1, y1)
        im = im.crop(crop_box)

    cw, ch = im.size
    work_max = max(preset.work_size, preset.ground.upscale_min_dim if max(cw, ch) < preset.ground.upscale_min_dim else preset.work_size)
    scale_to_work = work_max / float(max(cw, ch))
    work_w = max(1, int(round(cw * scale_to_work)))
    work_h = max(1, int(round(ch * scale_to_work)))
    # Snap near-square images (within 0.3% of square, e.g. 1670x1668) to exact square when requested or natural
    if abs(work_w - work_h) <= 2 and work_max == preset.work_size:
        work_w = work_h = work_max

    im_work = im.resize((work_w, work_h), Image.Resampling.LANCZOS)
    rgb = np.asarray(im_work, dtype=np.uint8)

    if preset.preprocess_hook is not None:
        rgb = preset.preprocess_hook(rgb, spec, preset)
        work_h, work_w = rgb.shape[:2]

    vb_scale = preset.viewbox_max / float(max(work_w, work_h))
    vb_w = round(work_w * vb_scale, 2)
    vb_h = round(work_h * vb_scale, 2)
    if abs(vb_w - round(vb_w)) < 0.05:
        vb_w = float(round(vb_w))
    if abs(vb_h - round(vb_h)) < 0.05:
        vb_h = float(round(vb_h))
    px_per_u = work_w / vb_w

    return {
        "rgb": rgb,
        "pil": Image.fromarray(rgb),
        "orig_size": (orig_w, orig_h),
        "crop_box": crop_box,
        "work_size": (work_w, work_h),
        "viewbox": [0.0, 0.0, vb_w, vb_h],
        "px_per_u": px_per_u,
    }


def _poly_mask_u(pts_u: List[List[float]], w: int, h: int, px_per_u: float) -> np.ndarray:
    m = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(m)
    draw.polygon([(p[0] * px_per_u, p[1] * px_per_u) for p in pts_u], fill=255)
    return np.asarray(m, dtype=np.uint8) > 127


def _apply_spec_polygons(mask: np.ndarray, spec: Dict[str, Any], px_per_u: float) -> np.ndarray:
    """Apply optional declarative include_polygons / exclude_polygons from spec in viewBox units."""
    inc_polys = spec.get("include_polygons")
    exc_polys = spec.get("exclude_polygons")
    if not inc_polys and not exc_polys:
        return mask
    H, W = mask.shape
    out = mask.copy()
    if inc_polys:
        inc = np.zeros((H, W), dtype=bool)
        for poly in inc_polys:
            inc |= _poly_mask_u(poly, W, H, px_per_u)
        out &= inc
    if exc_polys:
        for poly in exc_polys:
            out &= ~_poly_mask_u(poly, W, H, px_per_u)
    return out


def _detect_circular_roundel(lab: np.ndarray, cfg: GroundConfig) -> Dict[str, Any]:
    """Detect concentric roundel border ring, dark inner rim, and limb-over-frame protrusion sectors."""
    H, W = lab.shape[:2]
    min_dim = min(H, W)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    # Chromatic medieval roundel border bands (ultramarine blue + vermilion/terracotta)
    blue_ring = (b < -14.0) & (C > 18.0)
    red_ring = (a > 22.0) & (b > 12.0) & (L > 20.0) & (L < 68.0)
    ring_cand = blue_ring | red_ring

    cy0, cx0 = H / 2.0, W / 2.0
    n_rays = 360
    angles = np.linspace(0.0, 2.0 * np.pi, n_rays, endpoint=False)
    r_lo = int(0.30 * min_dim)
    r_hi = int(0.495 * min_dim)
    rs = np.arange(r_lo, r_hi)

    ray_hits = np.full(n_rays, np.nan, dtype=np.float64)
    pts = []
    for i, ang in enumerate(angles):
        xs = np.round(cx0 + rs * np.cos(ang)).astype(int)
        ys = np.round(cy0 + rs * np.sin(ang)).astype(int)
        valid = (xs >= 0) & (xs < W) & (ys >= 0) & (ys < H)
        vals = ring_cand[ys[valid], xs[valid]]
        if len(vals) >= 8:
            conv = np.convolve(vals.astype(int), np.ones(6, dtype=int), mode="valid")
            hits = np.where(conv >= 5)[0]
            if len(hits) > 0:
                r_hit = float(rs[valid][hits[0]])
                ray_hits[i] = r_hit
                pts.append((cx0 + r_hit * np.cos(ang), cy0 + r_hit * np.sin(ang)))

    if len(pts) < 40:
        return {"detected": False}

    pts_arr = np.asarray(pts, dtype=np.float64)
    res = optimize.least_squares(
        lambda p: np.hypot(pts_arr[:, 0] - p[0], pts_arr[:, 1] - p[1]) - p[2],
        [cx0, cy0, 0.40 * min_dim],
        loss="soft_l1",
        f_scale=3.0,
    )
    cx, cy, inner_r = float(res.x[0]), float(res.x[1]), float(res.x[2])

    # Re-scan rays from fitted center (cx, cy) to get exact per-angle ring inner radius & detect ring interruptions
    ys_g, xs_g = np.mgrid[0:H, 0:W]
    r_g = np.hypot(xs_g - cx, ys_g - cy)
    th_deg = (np.degrees(np.arctan2(ys_g - cy, xs_g - cx)) + 360.0) % 360.0

    # Measure dark inner rim width automatically from global radial dark fraction just inside inner_r
    rim_scan_lo = max(1, int(inner_r - 0.035 * min_dim))
    dark_mask = L < 28.0
    rim_inner_r = inner_r - 0.009 * min_dim
    for r_test in range(int(inner_r), rim_scan_lo, -1):
        band = np.abs(r_g - r_test) < 1.0
        if np.any(band) and float(np.mean(dark_mask[band])) < 0.42:
            rim_inner_r = float(r_test)
            break
    rim_width = max(3.5, min(9.0, inner_r - rim_inner_r))

    # Outer edge of chromatic ring
    rmax = int(0.55 * min_dim)
    r_int = np.clip(np.round(r_g).astype(int), 0, rmax).ravel()
    ring_frac_rad = np.bincount(r_int, weights=ring_cand.astype(float).ravel(), minlength=rmax + 1) / np.maximum(
        1, np.bincount(r_int, minlength=rmax + 1)
    )
    outer_r = int(inner_r + 3)
    while outer_r < rmax and ring_frac_rad[outer_r] > 0.12:
        outer_r += 1
    outer_r = max(outer_r, int(inner_r + cfg.frame_ring_width_frac * min_dim))

    # Measure smooth local ring inner radius r_local(deg) around 360 degrees
    r_local = np.full(360, np.nan, dtype=np.float64)
    # Detect angular sectors where chromatic ring is occluded by protruding limbs
    ring_band_occ = np.zeros(360, dtype=np.float64)
    v_lo = inner_r + 0.004 * min_dim
    v_hi = inner_r + 0.022 * min_dim
    for deg in range(360):
        ang = math.radians(deg)
        rs_s = np.arange(max(1, int(inner_r - 0.04 * min_dim)), min(int(outer_r + 8), int(0.52 * min_dim)))
        xs = np.round(cx + rs_s * math.cos(ang)).astype(int)
        ys = np.round(cy + rs_s * math.sin(ang)).astype(int)
        valid = (xs >= 0) & (xs < W) & (ys >= 0) & (ys < H)
        vals = ring_cand[ys[valid], xs[valid]]
        if len(vals) >= 6:
            conv = np.convolve(vals.astype(int), np.ones(5, dtype=int), mode="valid")
            hits = np.where(conv >= 4)[0]
            if len(hits) > 0:
                r_local[deg] = float(rs_s[valid][hits[0]])
        rs_v = np.arange(int(v_lo), int(v_hi) + 1)
        xv = np.round(cx + rs_v * math.cos(ang)).astype(int)
        yv = np.round(cy + rs_v * math.sin(ang)).astype(int)
        vv = (xv >= 0) & (xv < W) & (yv >= 0) & (yv < H)
        if np.any(vv):
            ring_band_occ[deg] = float(np.mean(ring_cand[yv[vv], xv[vv]]))

    # Smooth circular interpolation of r_local across occluded/noisy rays
    valid_deg = np.isfinite(r_local) & (np.abs(r_local - inner_r) < 0.03 * min_dim)
    if np.sum(valid_deg) >= 30:
        idx_v = np.where(valid_deg)[0]
        idx_ext = np.concatenate([idx_v - 360, idx_v, idx_v + 360])
        val_ext = np.concatenate([r_local[idx_v], r_local[idx_v], r_local[idx_v]])
        r_interp = np.interp(np.arange(360), idx_ext, val_ext)
        r_pad = np.concatenate([r_interp[-15:], r_interp, r_interp[:15]])
        r_smooth = ndimage.gaussian_filter1d(ndimage.median_filter(r_pad, size=11), sigma=4.0)[15:-15]
    else:
        r_smooth = np.full(360, inner_r, dtype=np.float64)

    # Identify angular sectors where limbs interrupt the chromatic ring
    occ_pad = np.concatenate([ring_band_occ[-5:], ring_band_occ, ring_band_occ[:5]])
    occ_smooth = ndimage.uniform_filter1d(occ_pad, size=5)[5:-5]
    interrupted = occ_smooth < 0.45
    protrusion_deg = np.zeros(360, dtype=bool)
    if cfg.limb_protrusion and np.any(interrupted):
        int_pad = np.concatenate([interrupted[-10:], interrupted, interrupted[:10]])
        int_open = ndimage.binary_opening(int_pad, structure=np.ones(3))
        # Modest +-4 deg margin hugs full spread of toes/claws
        int_dil = ndimage.binary_dilation(int_open, structure=np.ones(9))
        protrusion_deg = int_dil[10:-10]

    deg_idx = np.clip(np.floor(th_deg).astype(int), 0, 359)
    r_ring_map = r_smooth[deg_idx]
    r_field_map = r_ring_map - rim_width
    in_protrusion = protrusion_deg[deg_idx]

    return {
        "detected": True,
        "cx": cx,
        "cy": cy,
        "inner_r": inner_r,
        "rim_inner_r": rim_inner_r,
        "rim_width": rim_width,
        "outer_r": float(outer_r),
        "r_g": r_g,
        "r_ring_map": r_ring_map,
        "r_field_map": r_field_map,
        "in_protrusion": in_protrusion,
        "protrusion_angles": [int(d) for d in np.where(protrusion_deg)[0]],
        "blue_ring": blue_ring,
        "red_ring": red_ring,
    }


def _weighted_lab_dist(lab: np.ndarray, centers: np.ndarray, w_l: float, w_c: float) -> np.ndarray:
    """Compute minimum weighted CIELAB Euclidean distance to a set of ground cluster centers."""
    d_min = np.full(lab.shape[:2], np.inf, dtype=np.float32)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    for c in centers:
        d = np.sqrt((w_l * (L - c[0])) ** 2 + (w_c * (a - c[1])) ** 2 + (w_c * (b - c[2])) ** 2)
        d_min = np.minimum(d_min, d)
    return d_min


def _preserve_open_voids(
    fg_raw: np.ndarray,
    bg_dist: np.ndarray,
    cfg: GroundConfig,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Separate exterior background from interior holes, preserving genuine open voids (e.g., open mouth)."""
    H, W = fg_raw.shape
    total_area = float(H * W)
    inv = ~fg_raw
    labeled, n_cc = ndimage.label(inv)
    if n_cc == 0:
        return fg_raw.copy(), np.zeros_like(fg_raw), np.zeros_like(fg_raw)

    border_labels = set(np.unique(labeled[0, :]).tolist())
    border_labels.update(np.unique(labeled[-1, :]).tolist())
    border_labels.update(np.unique(labeled[:, 0]).tolist())
    border_labels.update(np.unique(labeled[:, -1]).tolist())
    border_labels.discard(0)

    ext_bg = np.isin(labeled, list(border_labels))
    void_mask = np.zeros_like(fg_raw)
    fg_clean = fg_raw.copy()

    areas = ndimage.sum(np.ones_like(inv, dtype=np.int32), labeled, index=np.arange(1, n_cc + 1))
    void_area_min = max(cfg.pinhole_max_area, int(round(cfg.void_area_frac * total_area)))

    for cid in range(1, n_cc + 1):
        if cid in border_labels:
            continue
        area = int(areas[cid - 1])
        hole = labeled == cid
        med_d = float(np.median(bg_dist[hole]))
        # Keep as transparent open void only if large enough OR strongly matching ground color
        if area > void_area_min or (area > 60 and med_d < cfg.void_bg_thresh):
            void_mask |= hole
        elif area <= cfg.pinhole_max_area:
            fg_clean |= hole

    return fg_clean, ext_bg, void_mask


def extract_ground_and_foreground(
    rgb: np.ndarray,
    preset: FamilyPreset,
    spec: Optional[Dict[str, Any]] = None,
    px_per_u: float = 3.0,
) -> Dict[str, Any]:
    """Stage 1: Adaptive CIELAB ground/frame removal with limb-over-frame continuity and open-void preservation."""
    spec = spec or {}
    if preset.ground_hook is not None:
        return preset.ground_hook(rgb, spec, preset)

    cfg = preset.ground
    H, W = rgb.shape[:2]
    lab = rgb2lab(rgb.astype(np.float32) / 255.0).astype(np.float32)
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    frame_info: Dict[str, Any] = {"detected": False}

    if cfg.mode == "gilded_roundel":
        frame_info = _detect_circular_roundel(lab, cfg)
        if frame_info.get("detected"):
            r_g = frame_info["r_g"]
            r_field_map = frame_info["r_field_map"]
            r_ring_map = frame_info["r_ring_map"]
            in_protrusion = frame_info["in_protrusion"]
            outer_r = frame_info["outer_r"]
            inside_field = r_g <= r_field_map

            # Adaptive gold-leaf ground model sampled strictly from warm-yellow pixels inside the roundel field
            gold_seed = inside_field & (b > 18.0) & (a > -4.0) & (a < 16.0) & (L > 22.0) & (L < 88.0) & (b > 1.45 * np.maximum(a, 1.0))
            if np.sum(gold_seed) < 500:
                gold_seed = inside_field & (b > 12.0) & (L > 25.0)
            seed_pts = lab[gold_seed]
            step = max(1, len(seed_pts) // 6000)
            bg_centers = _kmeans_lab(seed_pts[::step], k=cfg.bg_clusters)
            bg_dist = _weighted_lab_dist(lab, bg_centers, cfg.lightness_weight, cfg.chroma_weight)

            # Classify gold ground inside the field while protecting neutral ink, sage green (a < 0), and warm crimson/mauve
            is_neutral_ink = (L < 25.0) & (C < 18.0)
            is_sage_like = (a < -0.5) & (b < 24.0) & (L > 35.0)
            is_gold_core = (
                (
                    (bg_dist < cfg.bg_dist_thresh)
                    | ((b > 20.0) & (b > 1.45 * np.maximum(a, 1.0)) & (L > 18.0))
                )
                & (~is_neutral_ink)
                & (~is_sage_like)
            )
            # Also reject specular gold glare connected to gold ground (matte lead-white ivory stays < L* 86)
            is_spec_cand = ((L > 80.0) & (b > 16.0) & (a < 10.0)) | ((L > 88.0) & (b > 9.0) & (a < 10.0))
            lbl_gl, _ = ndimage.label(is_gold_core | is_spec_cand)
            gold_lbls = np.unique(lbl_gl[is_gold_core & inside_field])
            gold_lbls = gold_lbls[gold_lbls > 0]
            is_gold_bg = is_gold_core | (is_spec_cand & np.isin(lbl_gl, gold_lbls))

            fg_interior = inside_field & (~is_gold_bg)
            fg_interior_closed = ndimage.binary_closing(fg_interior, structure=np.ones((3, 3))) & inside_field

            # Limb-over-frame continuity rule
            fg_combined = fg_interior_closed.copy()
            if np.any(in_protrusion):
                protrusion_zone = in_protrusion & (r_g > r_field_map - 2.0) & (r_g <= outer_r + 8.0)
                non_prot_ring = (~in_protrusion) & (r_g >= r_ring_map) & (r_g <= outer_r)
                vermilion_pts = lab[non_prot_ring & frame_info["red_ring"]]
                blue_pts = lab[non_prot_ring & frame_info["blue_ring"]]
                is_border_red = np.zeros((H, W), dtype=bool)
                is_border_blue = np.zeros((H, W), dtype=bool)
                if len(vermilion_pts) > 50:
                    v_center = np.median(vermilion_pts, axis=0)
                    dv = np.sqrt((0.5 * (L - v_center[0])) ** 2 + (a - v_center[1]) ** 2 + (b - v_center[2]) ** 2)
                    is_border_red = (dv < 18.5) | ((a > 21.0) & (b > 13.0) & (a > 0.65 * b))
                if len(blue_pts) > 50:
                    u_center = np.median(blue_pts, axis=0)
                    du = np.sqrt((0.5 * (L - u_center[0])) ** 2 + (a - u_center[1]) ** 2 + (b - u_center[2]) ** 2)
                    is_border_blue = (du < 21.0) | (b < -7.0)

                is_outer_spandrel = (r_g > outer_r - 2.0) & (is_gold_bg | (L > 62.0))
                border_chromatic = is_border_red | is_border_blue | is_outer_spandrel
                border_chrom_dil = ndimage.binary_dilation(border_chromatic, iterations=3)
                # White zigzag sits along vermilion/blue border (r > r_ring_map + 5) touching chromatic border
                is_zigzag_white = (L > 58.0) & (C < 16.0) & border_chrom_dil & (r_g > r_ring_map + 5.0)
                limb_cand = protrusion_zone & (~border_chromatic) & (~is_gold_bg) & (~is_zigzag_white)

                # Separate genuine solid limb/toes from thin lateral dark inner-rim and zigzag whiskers:
                # 1. Identify solid non-ink limb interior paint (toes, ankle, flesh) inside protrusion_zone
                limb_paint = limb_cand & (L >= 26.0)
                limb_paint_core = ndimage.binary_opening(fg_interior_closed | limb_paint, structure=np.ones((3, 3))) & protrusion_zone
                # 2. Dark pixels in the circular rim band are only kept if within 4px of solid limb paint
                in_rim_band = (r_g >= r_field_map - 2.0) & (r_g <= r_ring_map + 4.0)
                near_paint = ndimage.binary_dilation(limb_paint_core | (fg_interior_closed & (L >= 26.0)), iterations=4)
                limb_cand = limb_cand & (~(in_rim_band & (L < 26.0) & (~near_paint)))

                # 3. Also require solid thickness via 7px disk opening before restoring 2px claw contour
                struct_disk7 = np.zeros((7, 7), dtype=bool)
                yy, xx = np.ogrid[-3:4, -3:4]
                struct_disk7[(xx * xx + yy * yy) <= 10] = True
                limb_core = ndimage.binary_opening(fg_interior_closed | limb_cand, structure=struct_disk7) & protrusion_zone
                limb_clean = ndimage.binary_dilation(limb_core, iterations=2) & limb_cand

                seed_band = fg_interior_closed & in_protrusion & (r_g >= r_field_map - 15.0)
                if np.any(seed_band):
                    union_cand = fg_interior_closed | limb_clean
                    lbl, _ = ndimage.label(union_cand)
                    seed_lbls = np.unique(lbl[seed_band])
                    seed_lbls = seed_lbls[seed_lbls > 0]
                    connected_limb = np.isin(lbl, seed_lbls) & limb_clean
                    connected_filled = ndimage.binary_fill_holes(fg_interior_closed | connected_limb)
                    fg_combined = fg_interior_closed | (connected_filled & protrusion_zone & (~border_chromatic))
                    lbl2, _ = ndimage.label(fg_combined)
                    keep_l = np.unique(lbl2[fg_interior_closed])
                    keep_l = keep_l[keep_l > 0]
                    fg_combined = np.isin(lbl2, keep_l)

            fg_combined = _apply_spec_polygons(fg_combined, spec, px_per_u)
            lbl_fg, n_fg = ndimage.label(fg_combined)
            if n_fg > 0:
                areas = ndimage.sum(fg_combined, lbl_fg, index=np.arange(1, n_fg + 1))
                min_area = max(200, int(round(cfg.min_fg_area_frac * H * W)))
                largest = int(np.argmax(areas)) + 1
                keep_ids = [largest]
                if spec.get("multi_component_fg", False):
                    keep_ids = [i + 1 for i, ar in enumerate(areas) if ar >= min_area]
                fg_raw = np.isin(lbl_fg, keep_ids)
            else:
                fg_raw = fg_combined

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
                    "protrusion_angles": frame_info["protrusion_angles"],
                },
            }

    elif cfg.mode == "monochrome_paper":
        # Flatten uneven paper illumination using large Gaussian background division
        gray = L / 100.0
        sigma = max(15.0, 0.04 * max(H, W))
        bg_illum = ndimage.gaussian_filter(gray, sigma=sigma)
        flat = np.clip(gray / np.maximum(bg_illum, 0.15), 0.0, 1.0)
        win = cfg.upscale_min_dim // 36
        win = max(15, win | 1)
        thr = threshold_sauvola(flat, window_size=win, k=0.22)
        ink = _apply_spec_polygons(flat < thr, spec, px_per_u)
        # Build figure silhouette by closing ink and filling interior body space
        env = ndimage.binary_closing(ink, structure=np.ones((7, 7)))
        env = ndimage.binary_fill_holes(env)
        env = _apply_spec_polygons(env, spec, px_per_u)
        # Keep substantial CCs
        lbl, n_cc = ndimage.label(env)
        if n_cc > 0:
            areas = ndimage.sum(env, lbl, index=np.arange(1, n_cc + 1))
            min_area = max(150, int(round(cfg.min_fg_area_frac * H * W)))
            keep = [i + 1 for i, ar in enumerate(areas) if ar >= min_area]
            fg_clean = np.isin(lbl, keep) if keep else (lbl == (int(np.argmax(areas)) + 1))
        else:
            fg_clean = env
        bg_dist = (1.0 - flat) * 100.0
        return {
            "fg_mask": fg_clean,
            "bg_mask": ~fg_clean,
            "void_mask": np.zeros_like(fg_clean),
            "bg_dist": bg_dist.astype(np.float32),
            "lab": lab,
            "frame_info": frame_info,
        }

    # General fallback: border-sampled CIELAB K-means + border-connected flood fill (Families B, C, E)
    border_strip = np.zeros((H, W), dtype=bool)
    bw = max(4, int(0.03 * min(H, W)))
    border_strip[:bw, :] = True
    border_strip[-bw:, :] = True
    border_strip[:, :bw] = True
    border_strip[:, -bw:] = True
    seed_pts = lab[border_strip]
    step = max(1, len(seed_pts) // 4000)
    bg_centers = _kmeans_lab(seed_pts[::step], k=cfg.bg_clusters)
    bg_dist = _weighted_lab_dist(lab, bg_centers, cfg.lightness_weight, cfg.chroma_weight)

    bg_cand = bg_dist < cfg.bg_dist_thresh
    lbl_b, _ = ndimage.label(bg_cand)
    edge_ids = set(np.unique(lbl_b[0, :]).tolist())
    edge_ids.update(np.unique(lbl_b[-1, :]).tolist())
    edge_ids.update(np.unique(lbl_b[:, 0]).tolist())
    edge_ids.update(np.unique(lbl_b[:, -1]).tolist())
    edge_ids.discard(0)
    ext_bg = np.isin(lbl_b, list(edge_ids))
    fg_raw = ~ext_bg

    # Apply optional declarative include/exclude polygons from spec (for multi-figure/scenic plates)
    fg_raw = _apply_spec_polygons(fg_raw, spec, px_per_u)

    lbl_fg, n_fg = ndimage.label(fg_raw)
    if n_fg > 0:
        areas = ndimage.sum(fg_raw, lbl_fg, index=np.arange(1, n_fg + 1))
        min_area = max(150, int(round(cfg.min_fg_area_frac * H * W)))
        keep = [i + 1 for i, ar in enumerate(areas) if ar >= min_area]
        fg_raw = np.isin(lbl_fg, keep) if keep else (lbl_fg == (int(np.argmax(areas)) + 1))

    fg_clean, ext_bg, void_mask = _preserve_open_voids(fg_raw, bg_dist, cfg)
    return {
        "fg_mask": fg_clean,
        "bg_mask": ext_bg,
        "void_mask": void_mask,
        "bg_dist": bg_dist,
        "lab": lab,
        "frame_info": frame_info,
    }
