"""Stage 5b: Standardized Playwright/Chrome QA master sheet and 1:1 per-component overlay sheet generator."""
from __future__ import annotations

import base64
import io
import math
import re
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

import numpy as np
from PIL import Image
from playwright.sync_api import sync_playwright


def _pil_to_data_uri(im: Image.Image, fmt: str = "PNG") -> str:
    buf = io.BytesIO()
    if fmt.upper() == "JPEG":
        im.convert("RGB").save(buf, format="JPEG", quality=92)
        mime = "image/jpeg"
    else:
        im.save(buf, format="PNG")
        mime = "image/png"
    return f"data:{mime};base64," + base64.b64encode(buf.getvalue()).decode("ascii")


def _replace_viewbox(svg_text: str, vb: Sequence[float]) -> str:
    vb_s = f"{vb[0]:.1f} {vb[1]:.1f} {vb[2]:.1f} {vb[3]:.1f}"
    return re.sub(r'viewBox="[^"]+"', f'viewBox="{vb_s}"', svg_text, count=1)


def _auto_zoom_crops(
    fg_mask: np.ndarray,
    ink_mask: np.ndarray,
    viewbox: Sequence[float],
    px_per_u: float,
) -> List[Dict[str, Any]]:
    """Auto-select two informative zoomed detail boxes when spec does not specify zoom_crops."""
    vb_w, vb_h = float(viewbox[2]), float(viewbox[3])
    ys, xs = np.where(fg_mask)
    if len(xs) == 0:
        return [
            {"label": "Zoom Detail #1", "viewbox": [0, 0, vb_w * 0.5, vb_h * 0.5]},
            {"label": "Zoom Detail #2", "viewbox": [vb_w * 0.4, vb_h * 0.4, vb_w * 0.5, vb_h * 0.5]},
        ]
    x0, x1 = float(xs.min()) / px_per_u, float(xs.max()) / px_per_u
    y0, y1 = float(ys.min()) / px_per_u, float(ys.max()) / px_per_u
    bw, bh = max(20.0, x1 - x0), max(20.0, y1 - y0)
    z1_w, z1_h = min(vb_w, max(55.0, bw * 0.45)), min(vb_h, max(55.0, bh * 0.45))
    return [
        {"label": "Zoomed Upper Detail (50%)", "viewbox": [max(0.0, x0), max(0.0, y0), z1_w, z1_h]},
        {
            "label": "Zoomed Lower Detail (50%)",
            "viewbox": [max(0.0, x1 - z1_w), max(0.0, y1 - z1_h), z1_w, z1_h],
        },
    ]


def generate_qa_sheets(
    src_rgb: np.ndarray,
    fg_mask: np.ndarray,
    ink_mask: np.ndarray,
    master_svg: str,
    comp_items: List[Dict[str, Any]],
    viewbox: Sequence[float],
    px_per_u: float,
    metrics: Dict[str, Any],
    spec: Dict[str, Any],
    sheet_out: Path,
    comp_sheet_out: Path,
    anim_svg: Optional[str] = None,
    chrome_path: str = "/usr/bin/google-chrome",
) -> None:
    """Render standardized <specimen>-sheet.png and <specimen>-components.png via Playwright Chrome."""
    vb_w, vb_h = float(viewbox[2]), float(viewbox[3])
    src_pil = Image.fromarray(src_rgb, mode="RGB")
    ref_uri = _pil_to_data_uri(src_pil, fmt="JPEG")

    true_iou = metrics["silhouette_iou_true_pct"]
    leg_iou = metrics.get("silhouette_iou_legacy_pct")
    ink_iou = metrics["ink_iou_pct"]
    bf1 = metrics["boundary_f1_1u_pct"]
    rmse = metrics["masked_color_rmse"]
    raw_kb = metrics["raw_kb"]
    gz_kb = metrics["gzip_kb"]

    title = spec.get("title", spec.get("specimen", "Specimen"))
    src_label = Path(spec.get("source_ref", "source")).name
    palette = spec.get("palette_override") or {}
    wash_def = palette.get("wash", "#7c3a4c")
    wash2_def = palette.get("wash2", "#4c1624")
    context_style = spec.get("context_style", "gilded_roundel" if spec.get("family") == "A" else "parchment")

    # Zoom crops (from spec or automatic)
    zoom_crops = spec.get("zoom_crops") or _auto_zoom_crops(fg_mask, ink_mask, viewbox, px_per_u)

    def make_zoom_card(idx_num: int, crop: Dict[str, Any]) -> str:
        zvb = crop["viewbox"]
        lbl = crop["label"]
        svg_zoom = _replace_viewbox(master_svg, zvb)
        zvb_s = f"{zvb[0]} {zvb[1]} {zvb[2]} {zvb[3]}"
        return (
            f'<div class="card"><div class="tag">{idx_num}. {lbl}</div>'
            f'<div class="stage overlay">'
            f'<svg viewBox="{zvb_s}" width="256" height="256" preserveAspectRatio="xMidYMid meet">'
            f'<image href="{ref_uri}" x="0" y="0" width="{vb_w}" height="{vb_h}"/></svg>'
            f'<div class="ov-svg">{svg_zoom}</div>'
            f'</div></div>'
        )

    leg_badge = f" · Legacy {leg_iou:.2f}%" if leg_iou is not None else ""
    badge_str = f"True IoU {true_iou:.2f}%{leg_badge} · Ink {ink_iou:.1f}% · BF1 {bf1:.1f}% · RMSE {rmse:.1f} · {raw_kb:.1f} KB ({gz_kb:.1f} KB gz)"

    # Aspect-aware stage box dimensions
    aspect = vb_w / max(1.0, vb_h)
    if aspect >= 1.0:
        st_w, st_h = 256, max(120, int(round(256 / aspect)))
    else:
        st_h, st_w = 256, max(120, int(round(256 * aspect)))

    ctx_class = "gold-bg" if context_style == "gilded_roundel" else "vellum-bg"
    ctx_title = "7. Master in Gilded Roundel Context" if context_style == "gilded_roundel" else "7. Master on Aged Parchment Context"

    row1_cards = [
        f'<div class="card"><div class="tag">1. Source: {src_label}</div><div class="stage"><img src="{ref_uri}"></div></div>',
        f'<div class="card"><div class="tag">2. Full 50% Overlay (True {true_iou:.2f}% · {raw_kb:.1f}/{gz_kb:.1f} KB)</div>'
        f'<div class="stage overlay"><img src="{ref_uri}"><div class="ov-svg">{master_svg}</div></div></div>',
        make_zoom_card(3, zoom_crops[0]),
        make_zoom_card(4, zoom_crops[1] if len(zoom_crops) > 1 else zoom_crops[0]),
    ]

    row2_cards = [
        f'<div class="card"><div class="tag">5. Stage A Static Master (vtracer)</div><div class="stage v1">{master_svg}</div></div>',
        f'<div class="card"><div class="tag">6. Pure Black Silhouette Test (stroke:none)</div><div class="stage sil">{master_svg}</div></div>',
        f'<div class="card"><div class="tag">{ctx_title}</div><div class="stage v1 {ctx_class}">{master_svg}</div></div>',
        f'<div class="card"><div class="tag">8. Mineral Lapis Recolour (--wash/--wash2)</div><div class="stage v2">{master_svg}</div></div>',
    ]

    row3_html = ""
    if anim_svg is not None or len(zoom_crops) >= 3:
        r3_cards: List[str] = []
        if len(zoom_crops) >= 3:
            r3_cards.append(make_zoom_card(9, zoom_crops[2]))
        if anim_svg is not None:
            slots = [
                ("1.85", "Stage C · t = 1.85s (blink &amp; turn)"),
                ("3.15", "Stage C · t = 3.15s (jaw open &amp; breathe)"),
                ("4.95", "Stage C · t = 4.95s (glance &amp; wing)"),
            ]
            if len(r3_cards) == 0:
                slots.insert(0, ("0.00", "Stage C · t = 0.00s (poised)"))
            for t_val, lbl in slots[: 4 - len(r3_cards)]:
                r3_cards.append(
                    f'<div class="card"><div class="tag">{lbl}</div>'
                    f'<div class="stage v1 anim-slot" data-t="{t_val}">{anim_svg}</div></div>'
                )
        row3_html = f'<div class="grid4">{"".join(r3_cards)}</div>'

    sheet_html = f"""<!doctype html><html><head><meta charset="utf-8"><style>
      * {{ box-sizing: border-box; }}
      body {{ margin: 0; padding: 18px; background: #e8dec8; color: #1c100a; font: 13px/1.4 Georgia, serif; }}
      h1 {{ margin: 0 0 4px; font-size: 17px; }}
      p.sub {{ margin: 0 0 12px; font: 11px/1.35 ui-monospace, monospace; color: #544232; }}
      .grid4 {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 12px; }}
      .card {{ background: #faf4e6; border: 1px solid #b8a582; padding: 8px; }}
      .tag {{ font: 10px/1.25 ui-monospace, monospace; color: #5c4632; margin-bottom: 6px; font-weight: 600; }}
      .stage {{ position: relative; width: {st_w}px; height: {st_h}px; margin: 0 auto; overflow: hidden; background: #faf4e6; }}
      .stage img, .stage svg {{ position: absolute; inset: 0; width: 100%; height: 100%; display: block; object-fit: contain; }}
      .overlay .ov-svg {{ position: absolute; inset: 0; opacity: 0.52; }}
      .sil svg * {{ fill: #1a110c !important; stroke: none !important; }}
      .gold-bg {{ background: radial-gradient(circle at 45% 42%, #e8c44a 0%, #bf9124 62%, #8c6412 100%); border-radius: 50%; border: 8px solid #c2381c; outline: 6px solid #2e4898; }}
      .vellum-bg {{ background: radial-gradient(circle at 50% 45%, #f5ebd4 0%, #e4d4b4 100%); border: 3px double #8a7352; }}
      .v2 {{ --wash: #2a4d8e; --wash2: #172d5c; }}
    </style></head><body>
      <h1>{title} — Dual-Res vtracer Verification Sheet</h1>
      <p class="sub">{badge_str}</p>
      <div class="grid4">{"".join(row1_cards)}</div>
      <div class="grid4">{"".join(row2_cards)}</div>
      {row3_html}
    </body></html>"""

    # Build per-component 1:1 overlay sheet
    H, W = fg_mask.shape
    comp_cards: List[str] = []
    for item in comp_items:
        fname = item["file"]
        cmask = item["mask"]
        svg_s = item["svg"]
        c_raw_kb = item["raw_kb"]
        c_gz_kb = item["gzip_kb"]

        ys, xs = np.where(cmask)
        if len(xs) == 0:
            continue
        pad_px = int(round(3.0 * px_per_u))
        x0 = max(0, int(xs.min()) - pad_px)
        y0 = max(0, int(ys.min()) - pad_px)
        x1 = min(W, int(xs.max()) + 1 + pad_px)
        y1 = min(H, int(ys.max()) + 1 + pad_px)

        rgba = np.zeros((y1 - y0, x1 - x0, 4), dtype=np.uint8)
        sub_m = cmask[y0:y1, x0:x1]
        rgba[sub_m, :3] = src_rgb[y0:y1, x0:x1][sub_m]
        rgba[sub_m, 3] = 255
        b64 = _pil_to_data_uri(Image.fromarray(rgba, mode="RGBA"), fmt="PNG")
        svg_fluid = re.sub(r'\s+width="[^"]+"\s+height="[^"]+"', "", svg_s, count=1)

        comp_cards.append(
            f'<div class="ccard">'
            f'<div class="tag">{fname} ({c_raw_kb:.1f} KB raw · {c_gz_kb:.1f} KB gz)</div>'
            f'<div class="pair">'
            f'<div class="box"><div class="sub">Extracted Pixels</div><img src="{b64}"></div>'
            f'<div class="box"><div class="sub">Traced SVG</div>{svg_fluid}</div>'
            f'<div class="box ov"><div class="sub">50% Overlay</div><img src="{b64}">{svg_fluid}</div>'
            f'</div></div>'
        )

    cols = 2 if len(comp_cards) >= 2 else 1
    comp_html = f"""<!doctype html><html><head><meta charset="utf-8"><style>
      * {{ box-sizing: border-box; }}
      body {{ margin: 0; padding: 18px; background: #e8dec8; color: #1c100a; font: 13px/1.4 Georgia, serif; }}
      h1 {{ margin: 0 0 4px; font-size: 17px; }}
      p.sub {{ margin: 0 0 12px; font: 11px/1.35 ui-monospace, monospace; color: #544232; }}
      .grid2 {{ display: grid; grid-template-columns: repeat({cols}, 1fr); gap: 12px; }}
      .ccard {{ background: #faf4e6; border: 1px solid #b8a582; padding: 10px; }}
      .tag {{ font: 11px/1.2 ui-monospace, monospace; color: #4a3624; margin-bottom: 6px; font-weight: 700; }}
      .pair {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }}
      .box {{ position: relative; height: 192px; background: #f3ead8; border: 1px solid #d4c5a8; overflow: hidden; }}
      .box .sub {{ position: absolute; top: 3px; left: 6px; font: 9.5px ui-monospace, monospace; color: #6e5840; z-index: 3; }}
      .box img, .box svg {{ position: absolute; inset: 18px 8px 8px; width: calc(100% - 16px); height: calc(100% - 26px); object-fit: contain; }}
      .box.ov svg {{ opacity: 0.55; z-index: 2; }}
    </style></head><body>
      <h1>{title} — Extracted Component Library ({len(comp_cards)} Parts · {raw_kb:.1f} KB Master)</h1>
      <p class="sub">{badge_str}</p>
      <div class="grid2">{"".join(comp_cards)}</div>
    </body></html>"""

    with tempfile.TemporaryDirectory() as td_s:
        td = Path(td_s)
        comp_path = td / "comp_sheet.html"
        master_path = td / "master_sheet.html"
        comp_path.write_text(comp_html, encoding="utf-8")
        master_path.write_text(sheet_html, encoding="utf-8")

        with sync_playwright() as pw:
            browser = pw.chromium.launch(executable_path=chrome_path, headless=True)
            pg1 = browser.new_page(viewport={"width": 1220, "height": 860}, device_scale_factor=1.75)
            pg1.goto(comp_path.as_uri(), wait_until="load")
            pg1.wait_for_timeout(120)
            pg1.screenshot(path=str(comp_sheet_out), full_page=True)

            pg2 = browser.new_page(viewport={"width": 1160, "height": 1000}, device_scale_factor=2)
            pg2.goto(master_path.as_uri(), wait_until="load")
            pg2.wait_for_timeout(120)
            pg2.evaluate(
                """() => {
                  document.querySelectorAll('.anim-slot').forEach(slot => {
                    const t = parseFloat(slot.getAttribute('data-t'));
                    const svg = slot.querySelector('svg');
                    if (svg && svg.pauseAnimations) { svg.pauseAnimations(); svg.setCurrentTime(t); }
                  });
                }"""
            )
            pg2.wait_for_timeout(60)
            pg2.screenshot(path=str(sheet_out), full_page=True)
            browser.close()
