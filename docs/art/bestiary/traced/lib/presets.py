"""Reusable family presets and hook registry for historical serpent vectorization."""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional


@dataclass
class LayerTraceConfig:
    """Parameters for dual-resolution, staircase-bias-compensated binary vtracer."""

    scale_size: int = 900
    blur_radius: float = 1.0
    threshold: int = 165
    filter_speckle: int = 8
    corner_threshold: int = 60
    length_threshold: float = 6.5
    splice_threshold: int = 58
    max_iterations: int = 10
    path_precision: int = 2
    line_eps: float = 0.14
    smooth_eps: float = 0.15
    min_subpath_u: float = 0.35


@dataclass
class GroundConfig:
    """Generalizable ground, frame, and open-void extraction parameters."""

    mode: str = "gilded_roundel"
    frame_shape: str = "circle"
    bg_clusters: int = 2
    chroma_weight: float = 1.8
    lightness_weight: float = 0.45
    bg_dist_thresh: float = 18.0
    void_area_frac: float = 0.0006
    void_bg_thresh: float = 21.0
    pinhole_max_area: int = 180
    limb_protrusion: bool = True
    frame_ring_width_frac: float = 0.085
    min_fg_area_frac: float = 0.004
    upscale_min_dim: int = 900


@dataclass
class PigmentConfig:
    """Adaptive CIELAB pigment clustering and multi-scale ink extraction parameters."""

    mode: str = "polychrome_lab"
    n_clusters: int = 5
    prefilter_radius: int = 2
    ink_l_max: float = 28.0
    ink_c_max: float = 20.0
    ink_valley_delta: float = 13.0
    ink_valley_l_max: float = 58.0
    sauvola_window: int = 25
    sauvola_k: float = 0.24
    accent_hue_sep_deg: float = 36.0
    wash_open_r: int = 1
    wash_close_r: int = 2
    min_ink_area: int = 6
    min_highlight_area: int = 5
    min_wash_area: int = 24


@dataclass
class FamilyPreset:
    """Complete reusable configuration for one historical visual/medium family."""

    name: str
    family: str
    description: str
    work_size: int = 900
    viewbox_max: float = 300.0
    ground: GroundConfig = field(default_factory=GroundConfig)
    pigments: PigmentConfig = field(default_factory=PigmentConfig)
    trace_base: LayerTraceConfig = field(
        default_factory=lambda: LayerTraceConfig(
            scale_size=1200,
            blur_radius=1.15,
            threshold=168,
            filter_speckle=10,
            length_threshold=7.0,
            splice_threshold=60,
            line_eps=0.12,
            min_subpath_u=0.4,
        )
    )
    trace_wash: LayerTraceConfig = field(
        default_factory=lambda: LayerTraceConfig(
            scale_size=600,
            blur_radius=0.85,
            threshold=152,
            filter_speckle=16,
            length_threshold=9.5,
            splice_threshold=65,
            line_eps=0.18,
            min_subpath_u=0.6,
        )
    )
    trace_highlight: LayerTraceConfig = field(
        default_factory=lambda: LayerTraceConfig(
            scale_size=650,
            blur_radius=0.65,
            threshold=148,
            filter_speckle=4,
            length_threshold=6.5,
            splice_threshold=58,
            line_eps=0.15,
            min_subpath_u=0.35,
        )
    )
    trace_accent: LayerTraceConfig = field(
        default_factory=lambda: LayerTraceConfig(
            scale_size=650,
            blur_radius=0.75,
            threshold=152,
            filter_speckle=8,
            length_threshold=7.5,
            splice_threshold=60,
            line_eps=0.16,
            min_subpath_u=0.45,
        )
    )
    trace_ink: LayerTraceConfig = field(
        default_factory=lambda: LayerTraceConfig(
            scale_size=750,
            blur_radius=0.85,
            threshold=164,
            filter_speckle=5,
            length_threshold=5.8,
            splice_threshold=55,
            line_eps=0.13,
            min_subpath_u=0.35,
        )
    )
    default_palette: Dict[str, str] = field(
        default_factory=lambda: {
            "wash": "#7c3a4c",
            "wash2": "#4c1624",
            "highlight": "#f2eadb",
            "warm_mid": "#d9c3ad",
            "ink": "#19100b",
        }
    )
    preprocess_hook: Optional[Callable[..., Any]] = None
    ground_hook: Optional[Callable[..., Any]] = None
    pigment_hook: Optional[Callable[..., Any]] = None
    segment_hook: Optional[Callable[..., Any]] = None


_REGISTRY: Dict[str, FamilyPreset] = {}
_FAMILIES_LOADED = False


def register_preset(name: str, preset: FamilyPreset) -> None:
    """Register or override a named family preset."""
    _REGISTRY[name] = preset


def _ensure_family_plugins() -> None:
    global _FAMILIES_LOADED
    if _FAMILIES_LOADED:
        return
    _FAMILIES_LOADED = True
    from . import families  # noqa: F401


def get_preset(name: str) -> FamilyPreset:
    """Retrieve a registered preset by name (auto-loading optional family plugins)."""
    _ensure_family_plugins()
    if name not in _REGISTRY:
        available = ", ".join(sorted(_REGISTRY))
        raise KeyError(f"Unknown preset '{name}'. Registered presets: {available}")
    return _REGISTRY[name]


def list_presets() -> List[str]:
    """Return sorted names of all registered presets."""
    _ensure_family_plugins()
    return sorted(_REGISTRY)


def _register_core_presets() -> None:
    register_preset(
        "gilded_romanesque",
        FamilyPreset(
            name="gilded_romanesque",
            family="A",
            description="12th-13th c. gilded Romanesque roundel illuminations on burnished gold leaf.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="gilded_roundel",
                frame_shape="circle",
                bg_clusters=2,
                chroma_weight=1.9,
                lightness_weight=0.40,
                bg_dist_thresh=18.5,
                void_area_frac=0.0008,
                void_bg_thresh=11.5,
                pinhole_max_area=480,
                limb_protrusion=True,
                frame_ring_width_frac=0.085,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=5,
                prefilter_radius=2,
                ink_l_max=25.0,
                ink_c_max=19.0,
                ink_valley_delta=13.5,
                ink_valley_l_max=58.0,
                accent_hue_sep_deg=34.0,
                wash_open_r=1,
                wash_close_r=2,
                min_ink_area=6,
                min_highlight_area=4,
                min_wash_area=28,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200,
                blur_radius=1.30,
                threshold=186,
                filter_speckle=6,
                length_threshold=5.8,
                splice_threshold=56,
                line_eps=0.11,
                min_subpath_u=0.40,
            ),
            trace_wash=LayerTraceConfig(
                scale_size=540,
                blur_radius=0.95,
                threshold=155,
                filter_speckle=20,
                length_threshold=10.5,
                splice_threshold=65,
                line_eps=0.21,
                min_subpath_u=0.80,
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
                scale_size=600,
                blur_radius=0.80,
                threshold=154,
                filter_speckle=10,
                length_threshold=8.5,
                splice_threshold=62,
                line_eps=0.19,
                min_subpath_u=0.55,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=720,
                blur_radius=0.68,
                threshold=156,
                filter_speckle=5,
                length_threshold=6.2,
                splice_threshold=56,
                line_eps=0.13,
                min_subpath_u=0.40,
            ),
        ),
    )

    register_preset(
        "gothic_interlace",
        FamilyPreset(
            name="gothic_interlace",
            family="B",
            description="Gilded inhabited initials and interlaced manuscript beasts on parchment/gold.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="gilded_initial",
                frame_shape="rect",
                bg_clusters=3,
                chroma_weight=1.6,
                lightness_weight=0.55,
                bg_dist_thresh=17.5,
                void_area_frac=0.0004,
                void_bg_thresh=20.0,
                pinhole_max_area=120,
                limb_protrusion=True,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=5,
                prefilter_radius=2,
                ink_l_max=28.0,
                ink_c_max=20.0,
                ink_valley_delta=12.0,
                ink_valley_l_max=56.0,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200, blur_radius=1.1, threshold=168,
                filter_speckle=8, length_threshold=6.5, splice_threshold=58, line_eps=0.12,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=750, blur_radius=0.75, threshold=160,
                filter_speckle=5, length_threshold=5.5, splice_threshold=55, line_eps=0.13,
            ),
        ),
    )

    register_preset(
        "parchment_drollery",
        FamilyPreset(
            name="parchment_drollery",
            family="C",
            description="Polychrome manuscript miniatures and marginal drolleries on vellum or blue ground.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="parchment",
                frame_shape="none",
                bg_clusters=2,
                chroma_weight=1.2,
                lightness_weight=0.85,
                bg_dist_thresh=16.5,
                void_area_frac=0.0005,
                void_bg_thresh=18.5,
                pinhole_max_area=120,
                limb_protrusion=False,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=4,
                prefilter_radius=2,
                ink_l_max=28.0,
                ink_c_max=22.0,
                ink_valley_delta=12.0,
                ink_valley_l_max=56.0,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200, blur_radius=1.1, threshold=168,
                filter_speckle=8, length_threshold=6.2, splice_threshold=58, line_eps=0.12,
            ),
        ),
    )

    register_preset(
        "monochrome_woodcut",
        FamilyPreset(
            name="monochrome_woodcut",
            family="D",
            description="16th-17th c. monochrome woodcuts and copperplate engravings on paper.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="monochrome_paper",
                frame_shape="none",
                bg_clusters=1,
                chroma_weight=0.6,
                lightness_weight=1.2,
                bg_dist_thresh=16.0,
                void_area_frac=0.0008,
                void_bg_thresh=15.0,
                pinhole_max_area=200,
                limb_protrusion=False,
                upscale_min_dim=900,
            ),
            pigments=PigmentConfig(
                mode="monochrome_ink",
                n_clusters=2,
                prefilter_radius=1,
                sauvola_window=25,
                sauvola_k=0.24,
                min_ink_area=4,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200, blur_radius=1.0, threshold=165,
                filter_speckle=6, length_threshold=5.2, splice_threshold=55, line_eps=0.12,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=900, blur_radius=0.55, threshold=150,
                filter_speckle=3, length_threshold=4.5, splice_threshold=50, line_eps=0.11,
            ),
            default_palette={
                "wash": "#f4ebd8",
                "wash2": "#e2d4b7",
                "highlight": "#faf5e9",
                "ink": "#19100b",
            },
        ),
    )

    register_preset(
        "heraldic_plate",
        FamilyPreset(
            name="heraldic_plate",
            family="E",
            description="Heraldic painted supporters and sculptural architectural bas-relief emblems.",
            work_size=900,
            viewbox_max=300.0,
            ground=GroundConfig(
                mode="heraldic",
                frame_shape="none",
                bg_clusters=2,
                chroma_weight=1.2,
                lightness_weight=0.9,
                bg_dist_thresh=16.0,
                void_area_frac=0.0005,
                void_bg_thresh=18.0,
                pinhole_max_area=150,
                limb_protrusion=False,
            ),
            pigments=PigmentConfig(
                mode="polychrome_lab",
                n_clusters=4,
                prefilter_radius=2,
                ink_l_max=26.0,
                ink_c_max=18.0,
                ink_valley_delta=12.0,
                ink_valley_l_max=54.0,
            ),
            trace_base=LayerTraceConfig(
                scale_size=1200, blur_radius=1.1, threshold=168,
                filter_speckle=8, length_threshold=6.0, splice_threshold=56, line_eps=0.12,
            ),
            trace_ink=LayerTraceConfig(
                scale_size=750, blur_radius=0.7, threshold=156,
                filter_speckle=5, length_threshold=5.0, splice_threshold=54, line_eps=0.12,
            ),
        ),
    )


_register_core_presets()
