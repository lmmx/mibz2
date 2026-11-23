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
