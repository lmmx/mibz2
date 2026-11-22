"""Reusable historical bestiary extraction, vtracer vectorization, and QA library."""
from .presets import (
    FamilyPreset,
    GroundConfig,
    LayerTraceConfig,
    PigmentConfig,
    get_preset,
    list_presets,
    register_preset,
)

__all__ = [
    "FamilyPreset",
    "GroundConfig",
    "LayerTraceConfig",
    "PigmentConfig",
    "get_preset",
    "list_presets",
    "register_preset",
]
