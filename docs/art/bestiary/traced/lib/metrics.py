"""Stage 5a: Multi-metric vector fidelity evaluator (True/Legacy Silhouette IoU, Ink IoU, Boundary F1, Color RMSE, KB/gz)."""
from __future__ import annotations

import gzip
import io
import re
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any, Dict, Optional, Sequence, Tuple

import numpy as np
from PIL import Image
from playwright.sync_api import sync_playwright
from scipy import ndimage

_CMD_RE = re.compile(r"[MmLlHhVvCcSsQqTtAaZz]")


def inspect_svg_complexity(svg_text: str) -> Dict[str, Any]:
    """Compute raw KB, gzip-9 KB, path count, and command breakdown for any SVG string."""
    raw_bytes = svg_text.encode("utf-8")
    gz_bytes = gzip.compress(raw_bytes, compresslevel=9)
    root = ET.fromstring(svg_text)

    path_count = 0
    cmd_counts: Dict[str, int] = {}
    has_use_href = False

    for elem in root.iter():
        tag = elem.tag.split("}")[-1]
        if tag == "use":
            for k in elem.attrib:
                if k == "href" or k.endswith("}href"):
                    has_use_href = True
        elif tag == "path":
            path_count += 1
            d = elem.attrib.get("d", "")
            for ch in _CMD_RE.findall(d):
                cmd_counts[ch] = cmd_counts.get(ch, 0) + 1

    return {
        "raw_bytes": len(raw_bytes),
        "gzip_bytes": len(gz_bytes),
        "raw_kb": round(len(raw_bytes) / 1024.0, 2),
        "gzip_kb": round(len(gz_bytes) / 1024.0, 2),
        "path_count": path_count,
        "command_count": int(sum(cmd_counts.values())),
        "cmd_counts": cmd_counts,
        "has_use_href": has_use_href,
    }


def _extract_ink_only_svg(svg_text: str) -> str:
    """Build an SVG keeping only ink paths (data-role='ink' or dark ink hex fills) for Ink IoU evaluation."""
    ET.register_namespace("", "http://www.w3.org/2000/svg")
    root = ET.fromstring(svg_text)
    ink_fills = {"#19100b", "#1a110c", "#000", "#000000", "#201616", "#2c261d"}

    def filter_node(node: ET.Element) -> None:
        for child in list(node):
            tag = child.tag.split("}")[-1]
            if tag == "path":
                role = child.attrib.get("data-role", "")
                fill = child.attrib.get("fill", "").strip().lower()
                is_ink = (role == "ink") or (fill in ink_fills) or ("--ink" in fill)
                if not is_ink:
                    node.remove(child)
            else:
                filter_node(child)

    filter_node(root)
    return ET.tostring(root, encoding="unicode")


def _compute_iou_bool(a: np.ndarray, b: np.ndarray) -> Tuple[float, int, int, int]:
    tp = int(np.sum(a & b))
    fp = int(np.sum((~a) & b))
    fn = int(np.sum(a & (~b)))
    union = tp + fp + fn
    iou = 100.0 * tp / max(1, union)
    return iou, tp, fp, fn


def _compute_boundary_f1(gt: np.ndarray, pred: np.ndarray, tol_px: int = 2) -> float:
    """Boundary F1 score within +-tol_px (e.g. 2px = 1 SVG unit at 600px max_dim)."""
    if not np.any(gt) or not np.any(pred):
        return 0.0
    st = np.ones((3, 3), dtype=bool)
    gt_b = gt ^ ndimage.binary_erosion(gt, structure=st)
    pr_b = pred ^ ndimage.binary_erosion(pred, structure=st)
    if not np.any(gt_b) or not np.any(pr_b):
        return 0.0
    dt_gt = ndimage.distance_transform_edt(~gt_b)
    dt_pr = ndimage.distance_transform_edt(~pr_b)
    prec = float(np.mean(dt_gt[pr_b] <= tol_px))
    rec = float(np.mean(dt_pr[gt_b] <= tol_px))
    if prec + rec < 1e-9:
        return 0.0
    return 100.0 * 2.0 * prec * rec / (prec + rec)


class PlaywrightMetricsEvaluator:
    """Reusable single-page headless Chrome rasterizer for fast, accurate SVG metric evaluation."""

    def __init__(self, max_dim: int = 600, chrome_path: str = "/usr/bin/google-chrome"):
        self.max_dim = max_dim
        self.chrome_path = chrome_path
        self._pw = None
        self._browser = None
        self._page = None

    def __enter__(self) -> "PlaywrightMetricsEvaluator":
        self._pw = sync_playwright().start()
        self._browser = self._pw.chromium.launch(executable_path=self.chrome_path, headless=True)
        self._page = self._browser.new_page(viewport={"width": self.max_dim, "height": self.max_dim})
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        if self._browser is not None:
            self._browser.close()
        if self._pw is not None:
            self._pw.stop()
        self._page = None
        self._browser = None
        self._pw = None

    def _render_html_to_rgb(self, html: str, rw: int, rh: int) -> np.ndarray:
        assert self._page is not None, "PlaywrightMetricsEvaluator must be used as a context manager"
        self._page.set_viewport_size({"width": rw, "height": rh})
        self._page.set_content(html, wait_until="load")
        png_bytes = self._page.screenshot(type="png")
        im = Image.open(io.BytesIO(png_bytes)).convert("RGB")
        return np.asarray(im, dtype=np.uint8)

    def _prep_svg_dims(self, svg_text: str, rw: int, rh: int) -> str:
        # Replace width/height on root <svg> to exact render dimensions
        return re.sub(
            r'(<svg\b[^>]*?)\s+width="[^"]*"\s+height="[^"]*"',
            rf'\1 width="{rw}" height="{rh}"',
            svg_text,
            count=1,
        )

    def evaluate_svg(
        self,
        svg_text: str,
        fg_mask: np.ndarray,
        ink_mask: np.ndarray,
        src_rgb: np.ndarray,
        viewbox: Sequence[float],
        include_legacy: bool = True,
        extra_ref_mask: Optional[np.ndarray] = None,
    ) -> Dict[str, Any]:
        """Compute full suite of silhouette, ink, boundary, color RMSE, and byte-size metrics."""
        vb_w, vb_h = float(viewbox[2]), float(viewbox[3])
        scale = float(self.max_dim) / max(vb_w, vb_h)
        rw = max(1, int(round(vb_w * scale)))
        rh = max(1, int(round(vb_h * scale)))

        svg_sized = self._prep_svg_dims(svg_text, rw, rh)
        comp = inspect_svg_complexity(svg_text)

        # Resize ground-truth masks with NEAREST (matching extract_roundel.py convention)
        fg_r = np.asarray(
            Image.fromarray((fg_mask.astype(np.uint8) * 255), mode="L").resize((rw, rh), Image.Resampling.NEAREST)
        ) > 127
        ink_r = np.asarray(
            Image.fromarray((ink_mask.astype(np.uint8) * 255), mode="L").resize((rw, rh), Image.Resampling.NEAREST)
        ) > 127
        src_r = np.asarray(
            Image.fromarray(src_rgb, mode="RGB").resize((rw, rh), Image.Resampling.LANCZOS), dtype=np.float32
        )

        # 1. True Fill-Only Silhouette IoU (stroke:none !important)
        html_true = (
            f'<!doctype html><html><body style="margin:0;background:#fff">'
            f'<style>svg * {{ fill:#000 !important; stroke:none !important; }}</style>'
            f'{svg_sized}</body></html>'
        )
        sil_true_rgb = self._render_html_to_rgb(html_true, rw, rh)
        pred_true = sil_true_rgb.mean(axis=2) < 128.0
        iou_true, tp_t, fp_t, fn_t = _compute_iou_bool(fg_r, pred_true)

        # 2. Boundary F1 within +-1 SVG unit (= scale px, i.e. 2px at 600px max_dim)
        tol_px = max(1, int(round(scale)))
        bf1 = _compute_boundary_f1(fg_r, pred_true, tol_px=tol_px)

        # 3. Legacy Silhouette IoU (stroke:#000 !important — exact baseline extract_roundel.py formula)
        iou_legacy = None
        if include_legacy:
            html_leg = (
                f'<!doctype html><html><body style="margin:0;background:#fff">'
                f'<style>svg * {{ fill:#000 !important; stroke:#000 !important; }}</style>'
                f'{svg_sized}</body></html>'
            )
            sil_leg_rgb = self._render_html_to_rgb(html_leg, rw, rh)
            pred_leg = sil_leg_rgb.mean(axis=2) < 128.0
            iou_legacy, _, _, _ = _compute_iou_bool(fg_r, pred_leg)

        # 4. Ink IoU (ink paths only)
        ink_svg = self._prep_svg_dims(_extract_ink_only_svg(svg_text), rw, rh)
        html_ink = (
            f'<!doctype html><html><body style="margin:0;background:#fff">'
            f'<style>svg * {{ fill:#000 !important; stroke:none !important; }}</style>'
            f'{ink_svg}</body></html>'
        )
        ink_ren_rgb = self._render_html_to_rgb(html_ink, rw, rh)
        pred_ink = ink_ren_rgb.mean(axis=2) < 128.0
        ink_iou, _, _, _ = _compute_iou_bool(ink_r, pred_ink)

        # 5. Full-color masked RGB RMSE inside fg_mask
        html_col = f'<!doctype html><html><body style="margin:0;background:#fff">{svg_sized}</body></html>'
        col_ren = self._render_html_to_rgb(html_col, rw, rh).astype(np.float32)
        if np.any(fg_r):
            color_rmse = float(np.sqrt(np.mean((col_ren[fg_r] - src_r[fg_r]) ** 2)))
        else:
            color_rmse = 0.0

        metrics: Dict[str, Any] = {
            "silhouette_iou_true_pct": round(iou_true, 2),
            "silhouette_iou_legacy_pct": round(iou_legacy, 2) if iou_legacy is not None else None,
            "ink_iou_pct": round(ink_iou, 2),
            "boundary_f1_1u_pct": round(bf1, 2),
            "masked_color_rmse": round(color_rmse, 2),
            "raw_kb": comp["raw_kb"],
            "gzip_kb": comp["gzip_kb"],
            "path_count": comp["path_count"],
            "command_count": comp["command_count"],
            "cmd_counts": comp["cmd_counts"],
            "true_fp_px": fp_t,
            "true_fn_px": fn_t,
        }

        if extra_ref_mask is not None:
            ref_r = np.asarray(
                Image.fromarray((extra_ref_mask.astype(np.uint8) * 255), mode="L").resize((rw, rh), Image.Resampling.NEAREST)
            ) > 127
            cross_true, _, _, _ = _compute_iou_bool(ref_r, pred_true)
            metrics["crosscheck_baseline_mask_true_iou_pct"] = round(cross_true, 2)
            if include_legacy:
                cross_leg, _, _, _ = _compute_iou_bool(ref_r, pred_leg)
                metrics["crosscheck_baseline_mask_legacy_iou_pct"] = round(cross_leg, 2)

        return metrics
