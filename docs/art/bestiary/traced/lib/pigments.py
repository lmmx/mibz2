"""Stage 2: Adaptive CIELAB pigment clustering, local-peak highlights, and multi-scale ink extraction."""
from __future__ import annotations

import math
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from scipy import ndimage
from skimage.color import lab2rgb
from skimage.filters import threshold_sauvola

from .ground import _kmeans_lab
from .presets import FamilyPreset, PigmentConfig


def _rgb_to_hex(rgb_f: np.ndarray) -> str:
    r, g, b = [int(np.clip(round(float(v)), 0, 255)) for v in rgb_f]
    return f"#{r:02x}{g:02x}{b:02x}"


def _lab_to_hex(lab_vec: np.ndarray) -> str:
    rgb = np.clip(lab2rgb(np.asarray(lab_vec, dtype=np.float32).reshape(1, 1, 3))[0, 0] * 255.0, 0, 255)
    return _rgb_to_hex(rgb)


def _keep_cc_min(mask: np.ndarray, min_area: int) -> np.ndarray:
    if min_area <= 1 or not np.any(mask):
        return mask.copy()
    lbl, n = ndimage.label(mask)
    if n == 0:
        return np.zeros_like(mask)
    areas = ndimage.sum(mask, lbl, index=np.arange(1, n + 1))
    keep = np.where(areas >= min_area)[0] + 1
    if len(keep) == 0:
        return np.zeros_like(mask)
    return np.isin(lbl, keep)


def _disk(r: int) -> np.ndarray:
    if r <= 0:
        return np.ones((1, 1), dtype=bool)
    y, x = np.ogrid[-r : r + 1, -r : r + 1]
    return (x * x + y * y) <= (r * r + 0.25)


def _hue_diff_deg(h1: float, h2: float) -> float:
    d = abs(h1 - h2) % 360.0
    return min(d, 360.0 - d)


def _extract_polychrome_ink(lab: np.ndarray, fg_mask: np.ndarray, cfg: PigmentConfig) -> np.ndarray:
    """Multi-scale iron-gall ink extraction: global dark-neutral + local dark-valley response."""
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    # 1. Global dark ink: very dark pixels OR dark neutral-chroma iron-gall strokes
    global_ink = fg_mask & (
        (L < 18.0)
        | ((L < cfg.ink_l_max) & (C < cfg.ink_c_max))
        | ((L < cfg.ink_l_max + 5.0) & (C < cfg.ink_c_max * 0.65))
    )

    # 2. Local dark-valley response (catches faded iron-gall strokes on bright faces/wings)
    # Use multi-scale Gaussian local background to highlight thin dark valleys cleanly
    local_mean = ndimage.gaussian_filter(L, sigma=3.8)
    valley_resp = local_mean - L
    valley_ink = fg_mask & (valley_resp > cfg.ink_valley_delta) & (L < cfg.ink_valley_l_max) & (C < cfg.ink_c_max + 2.0)

    # Suppress tiny 3-11px grain crumbs from valley_ink (eliminates pepper-speck noise on sage/ivory washes!)
    # Keep valley pixels either connected to global_ink OR forming genuine stroke segments (>= 18 px)
    valley_labeled, n_v = ndimage.label(valley_ink)
    if n_v > 0:
        v_areas = ndimage.sum(valley_ink, valley_labeled, index=np.arange(1, n_v + 1))
        touch_global = ndimage.binary_dilation(global_ink, iterations=1)
        keep_v = []
        for cid in range(1, n_v + 1):
            if v_areas[cid - 1] >= max(18, cfg.min_ink_area * 3):
                keep_v.append(cid)
            elif v_areas[cid - 1] >= cfg.min_ink_area and np.any(touch_global & (valley_labeled == cid)):
                keep_v.append(cid)
        valley_clean = np.isin(valley_labeled, keep_v) if keep_v else np.zeros_like(valley_ink)
    else:
        valley_clean = np.zeros_like(valley_ink)

    ink_raw = global_ink | valley_clean
    # Bridge 1px breaks along thin strokes using 4-connected cross closing (zero fattening)
    cross3 = np.array([[0, 1, 0], [1, 1, 1], [0, 1, 0]], dtype=bool)
    ink_closed = ndimage.binary_closing(ink_raw, structure=cross3) & fg_mask
    return _keep_cc_min(ink_closed, cfg.min_ink_area)


def extract_pigments(
    rgb: np.ndarray,
    fg_mask: np.ndarray,
    lab: np.ndarray,
    preset: FamilyPreset,
    spec: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Stage 2: Extract ink, adaptive CIELAB pigment layers, local-peak highlights, and sampled historical palette."""
    spec = spec or {}
    if preset.pigment_hook is not None:
        return preset.pigment_hook(rgb, fg_mask, spec, preset)

    cfg = preset.pigments
    H, W = fg_mask.shape
    L, a, b = lab[..., 0], lab[..., 1], lab[..., 2]
    C = np.hypot(a, b)

    if cfg.mode == "monochrome_ink":
        # Paper illumination flattening + adaptive Sauvola woodblock/engraving ink extraction
        gray = L / 100.0
        sigma = max(14.0, 0.035 * max(H, W))
        bg_illum = ndimage.gaussian_filter(gray, sigma=sigma)
        flat = np.clip(gray / np.maximum(bg_illum, 0.15), 0.0, 1.0)
        win = max(15, cfg.sauvola_window | 1)
        thr = threshold_sauvola(flat, window_size=win, k=cfg.sauvola_k)
        ink = fg_mask & (flat < thr)
        ink = _keep_cc_min(ink, cfg.min_ink_area)

        wash_hex = preset.default_palette.get("wash", "#f4ebd8")
        ink_hex = preset.default_palette.get("ink", "#19100b")
        if np.any(fg_mask & ~ink):
            med_paper = np.median(rgb[fg_mask & ~ink], axis=0)
            # Only adopt sampled paper hex if reasonably bright parchment
            if float(np.mean(med_paper)) > 150.0:
                wash_hex = _rgb_to_hex(med_paper)

        return {
            "ink": ink,
            "masks": {
                "wash": fg_mask & ~ink,
                "ink": ink,
            },
            "palette": {
                "wash": wash_hex,
                "wash2": preset.default_palette.get("wash2", "#d8cab0"),
                "ink": ink_hex,
            },
            "layer_order": ["ink"],
            "layer_specs": {
                "wash": {"fill": f"var(--wash,{wash_hex})", "opacity": 1.0, "trace_group": "base"},
                "ink": {"fill": ink_hex, "opacity": 1.0, "trace_group": "ink"},
            },
        }

    if cfg.mode == "tonal_relief":
        # 4-band warm limestone / sculptural luminance posterization + shadow crevice layer
        lab_s = np.stack([ndimage.median_filter(lab[..., c], size=5) for c in range(3)], axis=-1)
        Ls = lab_s[..., 0]
        fg_L = Ls[fg_mask]
        if len(fg_L) < 50:
            q20, q45, q75 = 30.0, 50.0, 70.0
        else:
            q20, q45, q75 = [float(x) for x in np.percentile(fg_L, [18, 45, 76])]
        ink = _keep_cc_min(fg_mask & (L < q20), cfg.min_ink_area)
        wash2 = _keep_cc_min(ndimage.binary_closing(fg_mask & (Ls < q45) & ~ink, structure=_disk(2)), cfg.min_wash_area)
        highlight = _keep_cc_min(ndimage.binary_closing(fg_mask & (Ls >= q75) & ~ink, structure=_disk(1)), cfg.min_highlight_area)
        wash_hex = _rgb_to_hex(np.median(rgb[fg_mask & (Ls >= q45) & (Ls < q75)], axis=0)) if np.any(fg_mask & (Ls >= q45) & (Ls < q75)) else "#9b9384"
        wash2_hex = _rgb_to_hex(np.median(rgb[wash2], axis=0)) if np.any(wash2) else "#6b6152"
        hi_hex = _rgb_to_hex(np.median(rgb[highlight], axis=0)) if np.any(highlight) else "#dbd6cb"
        ink_hex = "#2c261d"
        return {
            "ink": ink,
            "masks": {"wash": fg_mask, "wash2": wash2, "highlight": highlight, "ink": ink},
            "palette": {"wash": wash_hex, "wash2": wash2_hex, "highlight": hi_hex, "ink": ink_hex},
            "layer_order": ["wash2", "highlight", "ink"],
            "layer_specs": {
                "wash": {"fill": f"var(--wash,{wash_hex})", "opacity": 1.0, "trace_group": "base"},
                "wash2": {"fill": f"var(--wash2,{wash2_hex})", "opacity": 0.92, "trace_group": "wash"},
                "highlight": {"fill": hi_hex, "opacity": 0.90, "trace_group": "highlight"},
                "ink": {"fill": ink_hex, "opacity": 1.0, "trace_group": "ink"},
            },
        }

    # Default: polychrome_lab (Families A, B, C, E1)
    # 1. Multi-scale iron-gall ink on unfiltered LAB
    ink = _extract_polychrome_ink(lab, fg_mask, cfg)
    non_ink = fg_mask & (~ink)

    # 2. Edge-preserving smoothing for broad pigment regions (eliminates measles/grain confetti)
    med_k = max(3, (cfg.prefilter_radius * 2) | 1)
    lab_s = np.stack([ndimage.median_filter(lab[..., c], size=med_k) for c in range(3)], axis=-1)
    Ls, as_, bs_ = lab_s[..., 0], lab_s[..., 1], lab_s[..., 2]
    Cs = np.hypot(as_, bs_)

    # 3. Pre-separate distinct cool chromatic accents (e.g. sage green covert: a* < -1.0, L* > 35)
    # so K-means on warm body tones never merges a 6-8% cool accent into warm taupe!
    sage_raw = non_ink & (as_ < -1.0) & (bs_ > 2.0) & (Ls > 35.0) & (Cs >= 6.0)
    sage_clean = np.zeros((H, W), dtype=bool)
    if int(np.sum(sage_raw)) >= max(120, int(0.004 * np.sum(fg_mask))):
        sage_m = ndimage.binary_closing(ndimage.binary_opening(sage_raw, structure=_disk(1)), structure=_disk(2)) & fg_mask
        # Fill small enclosed holes where dark feather strokes sit inside the sage covert
        sage_filled = ndimage.binary_fill_holes(sage_m) & fg_mask
        sage_diff = sage_filled & (~sage_m)
        lbl_h, nh = ndimage.label(sage_diff)
        for hid in range(1, nh + 1):
            hm = lbl_h == hid
            if int(np.sum(hm)) <= 700:
                sage_m |= hm
        sage_clean = _keep_cc_min(sage_m, max(cfg.min_wash_area, 120))

    # 4. Adaptive CIELAB K-means on remaining non-ink, non-sage foreground pixels
    body_pool = non_ink & (~sage_clean)
    pts = lab_s[body_pool]
    k_body = max(3, min(4, cfg.n_clusters - (1 if np.any(sage_clean) else 0)))
    step = max(1, len(pts) // 12000)
    centers = _kmeans_lab(pts[::step], k=k_body, seed=42)

    d2 = np.sum((lab_s[:, :, None, :] - centers[None, None, :, :]) ** 2, axis=3)
    c_map = np.argmin(d2, axis=2)

    # Sort body clusters strictly by lightness L* (darkest -> lightest):
    # k=4: [0]=wash2 (deep shadow), [1]=wash (mid primary), [2]=warm_mid (light rosy-taupe), [3]=highlight (ivory)
    # k=3: [0]=wash2, [1]=wash, [2]=highlight
    stats = []
    for idx in range(k_body):
        m = body_pool & (c_map == idx)
        cnt = int(np.sum(m))
        if cnt == 0:
            continue
        c = centers[idx]
        med_rgb = np.median(rgb[m], axis=0)
        stats.append({
            "idx": idx,
            "count": cnt,
            "share": cnt / max(1.0, float(np.sum(body_pool))),
            "L": float(c[0]),
            "a": float(c[1]),
            "b": float(c[2]),
            "C": float(math.hypot(c[1], c[2])),
            "hue": float(math.degrees(math.atan2(c[2], c[1])) % 360.0),
            "hex": _rgb_to_hex(med_rgb),
            "mask": m,
        })
    stats.sort(key=lambda s: s["L"])

    wash2_cl = stats[0]
    hi_cl = stats[-1]
    mid_stats = stats[1:-1]
    if len(mid_stats) >= 2:
        wash_cl = mid_stats[0]
        warm_mid_cls = mid_stats[1:]
    elif len(mid_stats) == 1:
        wash_cl = mid_stats[0]
        warm_mid_cls = []
    else:
        wash_cl = wash2_cl
        warm_mid_cls = []

    # Build clean morphological layer masks
    open_s = _disk(cfg.wash_open_r)
    close_s = _disk(cfg.wash_close_r)

    # Deep shadow wash (--wash2): smooth painterly gouache band
    wash2_raw = wash2_cl["mask"].copy()
    wash2_clean = ndimage.binary_closing(ndimage.binary_opening(wash2_raw, structure=open_s), structure=close_s) & fg_mask
    wash2_clean = _keep_cc_min(wash2_clean, cfg.min_wash_area)

    # Warm mid-tone (rosy-taupe belly/cheek/foot light wash)
    warm_mid_raw = np.zeros((H, W), dtype=bool)
    for wc in warm_mid_cls:
        warm_mid_raw |= wc["mask"]
    warm_mid_clean = ndimage.binary_closing(ndimage.binary_opening(warm_mid_raw, structure=open_s), structure=close_s) & fg_mask
    warm_mid_clean = _keep_cc_min(warm_mid_clean, cfg.min_wash_area)
    warm_mid_hex = _rgb_to_hex(np.median(rgb[warm_mid_clean], axis=0)) if np.any(warm_mid_clean) else preset.default_palette.get("warm_mid", "#c8b2a4")

    # Crisp lead-white / ivory highlights + local bright-peak pearl dots (unfiltered L*!)
    local_bg_L = ndimage.gaussian_filter(L, sigma=3.5)
    bright_peak = non_ink & ((L - local_bg_L) > 12.5) & (L > 52.0) & (C < 19.0)
    hi_global = non_ink & (L > max(74.0, hi_cl["L"] - 6.0)) & (C < 15.0)
    highlight_raw = hi_global | bright_peak
    highlight_clean = _keep_cc_min(highlight_raw, cfg.min_highlight_area)
    hi_rgb_med = np.median(rgb[highlight_clean], axis=0) if np.any(highlight_clean) else np.array([239, 232, 218])
    if float(np.mean(hi_rgb_med)) < 210.0:
        hi_rgb_med = np.clip(hi_rgb_med * 1.12, 0, 248)
    hi_hex = _rgb_to_hex(hi_rgb_med)

    # Distinct vivid vermilion/crimson pigment (warmer & brighter red than dark wine, e.g., cheek mask & wing pinion ribs)
    lab_fine = np.stack([ndimage.median_filter(lab[..., c], size=3) for c in range(3)], axis=-1)
    crimson_cand = non_ink & (lab_fine[..., 1] > 21.5) & (lab_fine[..., 2] > 12.0) & (lab_fine[..., 0] > 32.0) & (lab_fine[..., 0] < 68.0)
    crimson_clean = ndimage.binary_closing(crimson_cand, structure=_disk(1)) & fg_mask
    crimson_clean = _keep_cc_min(crimson_clean, 10)

    masks: Dict[str, np.ndarray] = {
        "wash": fg_mask & (~ink),
        "wash2": wash2_clean,
        "warm_mid": warm_mid_clean,
        "highlight": highlight_clean,
        "ink": ink,
    }

    wash_hex = wash_cl["hex"]
    wash2_hex = wash2_cl["hex"]
    ink_hex = preset.default_palette.get("ink", "#19100b")

    palette: Dict[str, str] = {
        "wash": wash_hex,
        "wash2": wash2_hex,
        "warm_mid": warm_mid_hex,
        "highlight": hi_hex,
        "ink": ink_hex,
    }

    layer_specs: Dict[str, Dict[str, Any]] = {
        "wash": {"fill": f"var(--wash,{wash_hex})", "opacity": 1.0, "trace_group": "base"},
        "warm_mid": {"fill": warm_mid_hex, "opacity": 0.96, "trace_group": "wash"},
        "wash2": {"fill": f"var(--wash2,{wash2_hex})", "opacity": 0.96, "trace_group": "wash"},
        "highlight": {"fill": hi_hex, "opacity": 0.96, "trace_group": "highlight"},
        "ink": {"fill": ink_hex, "opacity": 1.0, "trace_group": "ink"},
    }
    layer_order: List[str] = ["warm_mid", "wash2"]

    if np.any(sage_clean):
        sage_hex = _rgb_to_hex(np.median(rgb[sage_clean & (~ink)], axis=0)) if np.any(sage_clean & (~ink)) else "#97a988"
        masks["sage"] = sage_clean
        palette["sage"] = sage_hex
        layer_specs["sage"] = {"fill": sage_hex, "opacity": 0.98, "trace_group": "accent"}
        layer_order.append("sage")

    # Add crimson accent if substantial (> 0.15% of foreground)
    if np.sum(crimson_clean) >= max(80, int(0.0015 * np.sum(fg_mask))):
        cr_hex = _rgb_to_hex(np.median(rgb[crimson_clean], axis=0))
        masks["crimson"] = crimson_clean
        palette["crimson"] = cr_hex
        layer_specs["crimson"] = {"fill": cr_hex, "opacity": 0.94, "trace_group": "accent"}
        layer_order.append("crimson")

    layer_order.extend(["highlight", "ink"])

    return {
        "ink": ink,
        "masks": masks,
        "palette": palette,
        "layer_order": layer_order,
        "layer_specs": layer_specs,
    }
