"""Illuminated-miniature preset variants.

parchment_pale_subject: the parchment drollery preset with a tighter ground threshold, for creatures
painted in tones close to the vellum (buff and fawn heads, pale bellies), which the default threshold
throws out with the ground. Use include_polygons to keep out painted grounds and tinted washes.
"""
from __future__ import annotations

from dataclasses import replace

from ..presets import _REGISTRY, register_preset

_base = _REGISTRY.get("parchment_drollery")
if _base is not None:
    register_preset(
        "parchment_pale_subject",
        replace(
            _base,
            name="parchment_pale_subject",
            description="Parchment drollery preset for subjects painted close to the vellum tone.",
            ground=replace(_base.ground, bg_dist_thresh=8.5, void_bg_thresh=10.0),
        ),
    )


def _fine(base_name: str, name: str, ink_c_max: float = 30.0) -> None:
    """A '-fine' variant keeping the broad fills but tracing the detail layers (ink and highlights)
    at full resolution with almost no smoothing, and taking brown and faded ink as ink."""
    base = _REGISTRY.get(base_name)
    if base is None:
        return
    from ..presets import LayerTraceConfig

    detail = dict(blur_radius=0.35, filter_speckle=2, length_threshold=3.0, splice_threshold=50,
                  line_eps=0.08, min_subpath_u=0.12)
    register_preset(
        name,
        replace(
            base,
            name=name,
            description=base.description + " Detail layers traced at full resolution.",
            work_size=1600,
            pigments=replace(base.pigments, ink_l_max=40.0, ink_c_max=ink_c_max, ink_valley_delta=9.0,
                             ink_valley_l_max=68.0, min_ink_area=2, min_highlight_area=2),
            trace_ink=replace(base.trace_ink, scale_size=2000, threshold=160, **detail),
            trace_highlight=replace(base.trace_highlight, scale_size=2000, **detail),
            trace_accent=replace(base.trace_accent, scale_size=1800, blur_radius=0.5, filter_speckle=3),
        ),
    )


_fine("parchment_pale_subject", "parchment_pale_subject_fine")
_fine("parchment_drollery", "parchment_drollery_fine")
_fine("gilded_romanesque", "gilded_romanesque_fine")
_fine("monochrome_woodcut", "monochrome_woodcut_fine")
# registered by batch_gilded.py before the plugins load, so present when run through it
_fine("gilded_interlace_compact", "gilded_interlace_compact_fine")


def _keyed_ground(rgb, spec, preset):
    """Ground by colour key, for a creature drawn on a painted field (purple, red, blue ...) that
    touches it everywhere, so no flood fill from the border can separate them.

    spec["ground_keys"]: [[r, g, b, tolerance_in_Lab], ...] colours of the field (and its spots are
    dropped as small pieces). include_polygons / exclude_polygons still apply. Pieces smaller than
    spec["min_piece_frac"] of the image (default 0.0008) are dropped, then holes up to the preset's
    pinhole size are filled.
    """
    import numpy as np
    from scipy import ndimage
    from skimage.color import rgb2lab
    from ..ground import _apply_spec_polygons

    H, W = rgb.shape[:2]
    vb_w = round(W * preset.viewbox_max / float(max(W, H)), 2)
    px_per_u = W / vb_w
    lab = rgb2lab(rgb.astype(np.float32) / 255.0).astype(np.float32)
    ground = np.zeros((H, W), dtype=bool)
    dmin = np.full((H, W), 99.0, dtype=np.float32)
    for r, g, b, tol in spec.get("ground_keys", []):
        k = rgb2lab(np.array([[[r, g, b]]], dtype=np.float32) / 255.0)[0, 0]
        d = np.sqrt(((lab - k) ** 2).sum(axis=2))
        dmin = np.minimum(dmin, d)
        ground |= d < tol
    fg = _apply_spec_polygons(~ground, spec, px_per_u)
    fg = ndimage.binary_opening(fg, structure=np.ones((3, 3)))
    lbl, n = ndimage.label(fg)
    if n:
        areas = ndimage.sum(fg, lbl, index=np.arange(1, n + 1))
        keep = [i + 1 for i, ar in enumerate(areas) if ar >= spec.get("min_piece_frac", 0.0008) * H * W]
        fg = np.isin(lbl, keep) if keep else fg
    inv_lbl, m = ndimage.label(~fg)
    if m:
        ar = ndimage.sum(np.ones_like(inv_lbl), inv_lbl, index=np.arange(1, m + 1))
        for i, a in enumerate(ar, start=1):
            if a <= preset.ground.pinhole_max_area * 3:
                fg |= inv_lbl == i
    return {"fg_mask": fg, "bg_mask": ~fg, "void_mask": np.zeros_like(fg),
            "bg_dist": dmin, "lab": lab, "frame_info": {"detected": False}}


_gr = _REGISTRY.get("gilded_romanesque_fine")
if _gr is not None:
    register_preset(
        "painted_field_keyed_fine",
        replace(_gr, name="painted_field_keyed_fine",
                description="Fine-detail preset with the ground removed by colour key (spec ground_keys).",
                ground_hook=_keyed_ground),
    )

# Dark saturated pigment (lapis, deep green) must not count as ink: brown and black ink are far less
# saturated. These variants keep the full-resolution detail with a strict chroma limit on ink.
_fine("gilded_interlace_compact", "gilded_interlace_compact_fine_lapis", ink_c_max=16.0)
_fine("gilded_romanesque", "gilded_romanesque_fine_lapis", ink_c_max=16.0)


def _fine_dark(base_name: str, name: str, ink_c_max: float, ink_l_max: float, ink_valley_l_max: float) -> None:
    """A '-fine' variant for dark, near-black pigment that must not all count as ink: only the
    darkest and least saturated pixels are ink, so scales and spots in a dark body stay as detail."""
    base = _REGISTRY.get(base_name)
    if base is None:
        return
    register_preset(
        name,
        replace(base, name=name,
                pigments=replace(base.pigments, ink_l_max=ink_l_max, ink_c_max=ink_c_max,
                                 ink_valley_l_max=ink_valley_l_max)),
    )


# the strict-ink variants below need the fine presets registered first
_fine("parchment_pale_subject", "parchment_pale_subject_fine_lapis", ink_c_max=16.0)
_fine_dark("parchment_pale_subject_fine_lapis", "parchment_pale_subject_fine_dark", 16.0, 9.0, 22.0)
