#!/usr/bin/env python3
"""Unified CLI runner for generalizable historical bestiary specimen extraction & vtracer vectorization."""
from __future__ import annotations

import argparse
import dataclasses
import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any, Dict, List, Optional

import numpy as np

TRACED_DIR = Path(__file__).resolve().parent
REPO_ROOT = TRACED_DIR.parents[3]

if str(TRACED_DIR) not in sys.path:
    sys.path.insert(0, str(TRACED_DIR))

from lib.ground import extract_ground_and_foreground, load_working_image
from lib.metrics import PlaywrightMetricsEvaluator, inspect_svg_complexity
from lib.pigments import extract_pigments
from lib.presets import FamilyPreset, get_preset
from lib.qa_sheet import generate_qa_sheets
from lib.segment import segment_components
from lib.vectorize import (
    build_master_and_animated_svg,
    pareto_sweep_trace_config,
    trace_component,
    wrap_standalone_component_svg,
)


def _resolve_repo_path(p: str | Path) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    cand = REPO_ROOT / path
    if cand.exists():
        return cand
    return path.resolve()


def _serialize_vtracer_config(preset: FamilyPreset) -> Dict[str, Any]:
    return {
        "colormode": "binary",
        "hierarchical": "stacked",
        "mode": "spline",
        "base": dataclasses.asdict(preset.trace_base),
        "wash": dataclasses.asdict(preset.trace_wash),
        "highlight": dataclasses.asdict(preset.trace_highlight),
        "accent": dataclasses.asdict(preset.trace_accent),
        "ink": dataclasses.asdict(preset.trace_ink),
    }


def run_specimen(
    spec_path: Path | str,
    output_dir: Optional[Path | str] = None,
    sweep: bool = False,
    skip_sheets: bool = False,
    crosscheck_mask_path: Optional[Path | str] = None,
) -> Dict[str, Any]:
    """Run all 5 pipeline stages for one specimen spec JSON and emit SVGs, meta JSON, and QA sheets."""
    spec_p = Path(spec_path).resolve()
    spec = json.loads(spec_p.read_text(encoding="utf-8"))
    slug = spec["specimen"]
    out_dir = Path(output_dir).resolve() if output_dir else TRACED_DIR
    out_dir.mkdir(parents=True, exist_ok=True)

    comp_subdir = spec.get("components_subdir", slug)
    comp_dir = out_dir / "components" / comp_subdir
    comp_dir.mkdir(parents=True, exist_ok=True)

    preset = get_preset(spec["preset"])
    src_path = _resolve_repo_path(spec["source_ref"])

    # Stage 1: Load & Ground/Frame Removal
    work = load_working_image(src_path, preset, spec=spec)
    rgb = work["rgb"]
    viewbox = work["viewbox"]
    px_per_u = work["px_per_u"]

    ground = extract_ground_and_foreground(rgb, preset, spec=spec, px_per_u=px_per_u)
    fg_mask = ground["fg_mask"]
    lab = ground["lab"]

    # Stage 2: Adaptive CIELAB Pigments & Multi-Scale Ink
    pigments = extract_pigments(rgb, fg_mask, lab, preset, spec=spec)
    ink_mask = pigments["ink"]

    # Stage 3: Non-Destructive Ink-Guided Anatomical Segmentation
    seg = segment_components(rgb, fg_mask, lab, pigments, preset, spec=spec, px_per_u=px_per_u)

    # Optional cross-check reference mask
    extra_ref_mask = None
    if crosscheck_mask_path:
        from PIL import Image
        extra_ref_mask = np.asarray(Image.open(crosscheck_mask_path).convert("L")) > 127

    sweep_log: List[Dict[str, Any]] = []
    with PlaywrightMetricsEvaluator(max_dim=600) as evaluator:
        # Optional Stage 4 Pareto sweep
        if sweep:
            preset, sweep_log = pareto_sweep_trace_config(
                fg_mask, ink_mask, rgb, seg, pigments, viewbox, preset, spec, evaluator
            )

        # Stage 4: Dual-Res Bias-Compensated Per-Layer Binary vtracer
        traced_parts: Dict[str, List[str]] = {}
        traced_subparts: Dict[str, Dict[str, List[str]]] = {}
        comp_items: List[Dict[str, Any]] = []
        comp_meta_list: List[Dict[str, Any]] = []

        for cid in seg["order"]:
            pmeta = seg["part_meta"].get(cid, {})
            cmask = seg["parts"][cid]
            fname = pmeta.get("file", f"{cid}.svg")
            ctitle = pmeta.get("title", f"{spec.get('title', slug)} {cid}")

            if cid in seg["subparts"]:
                traced_subparts[cid] = {}
                inner_blocks: List[str] = []
                for sp_id, sp_m in seg["subparts"][cid].items():
                    sp_meta = {**pmeta, "fine_detail": True}
                    elems, _ = trace_component(sp_m, pigments, viewbox, preset, sp_meta)
                    traced_subparts[cid][sp_id] = elems
                    inner_blocks.append(f'<g id="{sp_id}">\n' + "\n".join(elems) + "\n</g>")
                inner_xml = "\n".join(inner_blocks)
            else:
                elems, _ = trace_component(cmask, pigments, viewbox, preset, pmeta)
                traced_parts[cid] = elems
                inner_xml = f'<g id="{cid}">\n' + "\n".join(elems) + "\n</g>"

            comp_svg, bbox = wrap_standalone_component_svg(inner_xml, cmask, viewbox, px_per_u, ctitle)
            comp_file = comp_dir / fname
            comp_file.write_text(comp_svg, encoding="utf-8")
            c_info = inspect_svg_complexity(comp_svg)

            comp_items.append({
                "id": cid,
                "file": fname,
                "mask": cmask,
                "svg": comp_svg,
                "raw_kb": c_info["raw_kb"],
                "gzip_kb": c_info["gzip_kb"],
                "bbox": bbox,
            })
            comp_meta_list.append({
                "id": cid,
                "file": fname,
                "raw_kb": c_info["raw_kb"],
                "gzip_kb": c_info["gzip_kb"],
                "bbox": bbox,
            })

        master_svg = build_master_and_animated_svg(
            traced_parts, traced_subparts, seg["order"], viewbox, spec, animated=False
        )
        ET.fromstring(master_svg)
        master_path = out_dir / f"{slug}-master.svg"
        master_path.write_text(master_svg, encoding="utf-8")

        anim_svg: Optional[str] = None
        anim_path: Optional[Path] = None
        if spec.get("animation"):
            anim_svg = build_master_and_animated_svg(
                traced_parts, traced_subparts, seg["order"], viewbox, spec, animated=True
            )
            ET.fromstring(anim_svg)
            anim_path = out_dir / f"{slug}-animated.svg"
            anim_path.write_text(anim_svg, encoding="utf-8")

        # Stage 5a: Multi-metric evaluation
        metrics = evaluator.evaluate_svg(
            master_svg,
            fg_mask,
            ink_mask,
            rgb,
            viewbox,
            include_legacy=True,
            extra_ref_mask=extra_ref_mask,
        )

    # Stage 5b: Standardized QA sheets
    sheet_path = out_dir / f"{slug}-sheet.png"
    comp_sheet_path = out_dir / f"{slug}-components.png"
    if not skip_sheets:
        generate_qa_sheets(
            src_rgb=rgb,
            fg_mask=fg_mask,
            ink_mask=ink_mask,
            master_svg=master_svg,
            comp_items=comp_items,
            viewbox=viewbox,
            px_per_u=px_per_u,
            metrics=metrics,
            spec=spec,
            sheet_out=sheet_path,
            comp_sheet_out=comp_sheet_path,
            anim_svg=anim_svg,
        )

    # Update interactive proof HTML if declared (only when writing to canonical traced dir)
    if output_dir is None and anim_svg is not None and spec.get("proof_html"):
        proof_path = _resolve_repo_path(spec["proof_html"])
        if proof_path.exists():
            old_html = proof_path.read_text(encoding="utf-8")
            updated_html = re.sub(
                r'<div class="stage gold" id="anim-stage">.*?</div>\s*</div>',
                f'<div class="stage gold" id="anim-stage">{anim_svg}</div>\n      </div>',
                old_html,
                count=1,
                flags=re.S,
            )
            proof_path.write_text(updated_html, encoding="utf-8")

    # Assemble contract-compliant metadata JSON
    rel_comp_dir = f"docs/art/bestiary/traced/components/{comp_subdir}"
    assemblies_dict: Dict[str, str] = {
        "master": f"docs/art/bestiary/traced/{slug}-master.svg",
        "sheet": f"docs/art/bestiary/traced/{slug}-sheet.png",
        "components_sheet": f"docs/art/bestiary/traced/{slug}-components.png",
    }
    if anim_path is not None:
        assemblies_dict["animated"] = f"docs/art/bestiary/traced/{slug}-animated.svg"

    meta: Dict[str, Any] = {
        "specimen": slug,
        "source_ref": spec["source_ref"],
        "family": spec.get("family", preset.family),
        "preset": preset.name,
        "source_style": spec.get("source_style", preset.description),
        "date_range": spec.get("date_range", ""),
        "medium": spec.get("medium", ""),
        "tracing_method": "vtracer-dual-res-stacked",
        "viewbox": viewbox,
        "metrics": {
            "silhouette_iou_true_pct": metrics["silhouette_iou_true_pct"],
            "silhouette_iou_legacy_pct": metrics["silhouette_iou_legacy_pct"],
            "ink_iou_pct": metrics["ink_iou_pct"],
            "boundary_f1_1u_pct": metrics["boundary_f1_1u_pct"],
            "masked_color_rmse": metrics["masked_color_rmse"],
            "raw_kb": metrics["raw_kb"],
            "gzip_kb": metrics["gzip_kb"],
            "path_count": metrics["path_count"],
            "command_count": metrics["command_count"],
            "cmd_counts": metrics["cmd_counts"],
        },
        "vtracer_config": _serialize_vtracer_config(preset),
        "components_dir": rel_comp_dir,
        "components": comp_meta_list,
        "assemblies": assemblies_dict,
        "palette": pigments["palette"],
        "flaws_fixed": spec.get("flaws_fixed", []),
    }

    # Preserve backward-compatible top-level keys if declared (e.g., romanesque-roundel.meta.json)
    if spec.get("legacy_meta"):
        lm = spec["legacy_meta"]
        if "component" in lm:
            meta["component"] = lm["component"]
        meta["silhouette_iou_pct"] = metrics["silhouette_iou_true_pct"]
        if "stages" in lm:
            meta["stages"] = lm["stages"]

    meta_path = out_dir / f"{slug}.meta.json"
    meta_path.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")

    return {
        "meta": meta,
        "metrics": metrics,
        "sweep_log": sweep_log,
        "union_coverage_pct": seg["union_coverage_pct"],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Extract and vectorize a historical bestiary specimen with vtracer.")
    parser.add_argument("spec", help="Path to specimen JSON specification")
    parser.add_argument("--output-dir", default=None, help="Output directory (defaults to docs/art/bestiary/traced)")
    parser.add_argument("--sweep", action="store_true", help="Run automated Pareto parameter sweep before final output")
    parser.add_argument("--skip-sheets", action="store_true", help="Skip Playwright QA PNG sheet generation")
    parser.add_argument("--crosscheck-mask", default=None, help="Optional reference mask PNG to cross-check IoU")
    args = parser.parse_args()

    res = run_specimen(
        spec_path=args.spec,
        output_dir=args.output_dir,
        sweep=args.sweep,
        skip_sheets=args.skip_sheets,
        crosscheck_mask_path=args.crosscheck_mask,
    )
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
    if "crosscheck_baseline_mask_true_iou_pct" in m:
        print(
            f"  Cross-check vs ref mask: True IoU={m['crosscheck_baseline_mask_true_iou_pct']:.2f}%, "
            f"Legacy IoU={m.get('crosscheck_baseline_mask_legacy_iou_pct', 0):.2f}%"
        )
    if res["sweep_log"]:
        print("  Pareto Sweep Summary:")
        for row in res["sweep_log"]:
            print(
                f"    {row['config']:14s}: True={row['silhouette_iou_true_pct']:6.2f}% "
                f"Leg={row['silhouette_iou_legacy_pct']:6.2f}% Ink={row['ink_iou_pct']:5.2f}% "
                f"KB={row['raw_kb']:5.2f} ({row['gzip_kb']:5.2f} gz) score={row['score']:.3f}"
            )


if __name__ == "__main__":
    main()
