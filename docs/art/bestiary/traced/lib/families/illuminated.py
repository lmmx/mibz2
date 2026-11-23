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


def _fine(base_name: str, name: str) -> None:
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
            pigments=replace(base.pigments, ink_l_max=40.0, ink_c_max=30.0, ink_valley_delta=9.0,
                             ink_valley_l_max=68.0, min_ink_area=2, min_highlight_area=2),
            trace_ink=replace(base.trace_ink, scale_size=2000, threshold=160, **detail),
            trace_highlight=replace(base.trace_highlight, scale_size=2000, **detail),
            trace_accent=replace(base.trace_accent, scale_size=1800, blur_radius=0.5, filter_speckle=3),
        ),
    )


_fine("parchment_pale_subject", "parchment_pale_subject_fine")
_fine("parchment_drollery", "parchment_drollery_fine")
_fine("gilded_romanesque", "gilded_romanesque_fine")
