#!/usr/bin/env python3
"""
Render a 12-frame burrow-contract proof sheet across the last 30% of the SMIL cycle
for o@16px, o@48px, and i-dot, and verify 0 creature pixels remain outside the hole
disc after tail entry.

Usage:
  python3 render_proof.py draco-bipes [engraved|illuminated] [headVariant]
"""
import io
import math
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
from playwright.sync_api import sync_playwright

CRAWLER_DIR = Path(__file__).resolve().parent
TEST_HTML = CRAWLER_DIR / "test.html"

STAGES = [
    ("o16", "svg-o16", "o at 16px"),
    ("o48", "svg-o48", "o at 48px"),
    ("idot", "svg-idot", "dot of i"),
]

# 12 frames across the last 30% of the cycle (70% -> 99.5%)
FRACS = [0.70 + (0.295 * i / 11.0) for i in range(12)]


def count_stray_pixels(frame_img: Image.Image, base_img: Image.Image, hole_px, r_px) -> tuple[int, int]:
    """Return (total_diff_pixels, outside_disc_diff_pixels)."""
    fa = frame_img.convert("RGB")
    ba = base_img.convert("RGB")
    w, h = fa.size
    if ba.size != (w, h):
        ba = ba.resize((w, h))
    fp = fa.load()
    bp = ba.load()
    hx, hy = hole_px
    r2 = (r_px + 1.5) ** 2
    total_diff = 0
    outside = 0
    for y in range(h):
        for x in range(w):
            r1, g1, b1 = fp[x, y]
            r0, g0, b0 = bp[x, y]
            if abs(r1 - r0) + abs(g1 - g0) + abs(b1 - b0) > 18:
                total_diff += 1
                if (x - hx) ** 2 + (y - hy) ** 2 > r2:
                    outside += 1
    return total_diff, outside


def render_proof(creature: str, register: str = "engraved", head_variant: str = "") -> Path:
    out_png = CRAWLER_DIR / f"proof-{creature}.png"
    qs_base = f"?creature={creature}&register={register}&baseline=1"
    qs_live = f"?creature={creature}&register={register}" + (f"&head={head_variant}" if head_variant else "")
    url_base = TEST_HTML.as_uri() + qs_base
    url_live = TEST_HTML.as_uri() + qs_live

    baselines = {}
    frames = {k: [] for k, _, _ in STAGES}
    audit_results = {k: [] for k, _, _ in STAGES}
    metas = {}

    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path="/usr/bin/google-chrome", headless=True)
        page = browser.new_page(viewport={"width": 1280, "height": 900}, device_scale_factor=2)

        # 1. Capture clean creature-free baselines
        page.goto(url_base, wait_until="load")
        page.wait_for_timeout(120)
        for key, svg_id, _ in STAGES:
            raw = page.locator(f"#{svg_id}").screenshot()
            baselines[key] = Image.open(io.BytesIO(raw)).convert("RGB")

        # 2. Capture 12 frames across the last 30% of the cycle
        page.goto(url_live, wait_until="load")
        page.wait_for_timeout(120)

        stage_info = page.evaluate("""() => {
          const out = {};
          for (const k of ['o16', 'o48', 'idot']) {
            const st = window.__BURROW_STAGES[k];
            const rect = st.svg.getBoundingClientRect();
            const vb = st.svg.viewBox.baseVal;
            out[k] = {
              dur: st.meta.dur,
              visLen: st.meta.visLen,
              bodyLen: st.meta.bodyLen,
              maskDisc: st.meta.maskDisc,
              storyboard: st.meta.storyboard,
              sx: rect.width / vb.width,
              sy: rect.height / vb.height
            };
            st.svg.pauseAnimations();
          }
          return out;
        }""")
        metas = stage_info

        for frac in FRACS:
            for key, svg_id, _ in STAGES:
                dur = metas[key]["dur"]
                t = frac * dur
                page.evaluate(f"document.getElementById('{svg_id}').setCurrentTime({t:.4f});")
            page.wait_for_timeout(25)

            for key, svg_id, _ in STAGES:
                raw = page.locator(f"#{svg_id}").screenshot()
                img = Image.open(io.BytesIO(raw)).convert("RGB")
                # Scale hole disc center/radius to screenshot pixels (device_scale_factor=2)
                md = metas[key]["maskDisc"]
                dpr = 2.0
                hx = md["cx"] * metas[key]["sx"] * dpr
                hy = md["cy"] * metas[key]["sy"] * dpr
                hr = md["r"] * max(metas[key]["sx"], metas[key]["sy"]) * dpr
                total_diff, outside_diff = count_stray_pixels(img, baselines[key], (hx, hy), hr)
                frames[key].append((frac, img))
                audit_results[key].append((frac, total_diff, outside_diff))

        browser.close()

    # Compose contact sheet: 3 sections (o@16px, o@48px, i-dot), each a 2x6 grid of frames
    cell_w = 240
    pad = 12
    gap = 8
    header_h = 56
    section_title_h = 26
    label_h = 18

    section_heights = []
    cell_heights = {}
    for key, _, _ in STAGES:
        im0 = frames[key][0][1]
        ch = int(round(cell_w * (im0.height / im0.width)))
        cell_heights[key] = ch
        section_heights.append(section_title_h + 2 * (ch + label_h + gap) + 12)

    cols = 6
    total_w = pad * 2 + cols * cell_w + (cols - 1) * gap
    total_h = header_h + sum(section_heights) + pad

    sheet = Image.new("RGB", (total_w, total_h), (235, 226, 206))
    draw = ImageDraw.Draw(sheet)

    try:
        font_title = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf", 18)
        font_sec = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 13)
        font_lbl = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 11)
    except Exception:
        font_title = font_sec = font_lbl = ImageFont.load_default()

    # Audit summary: every frame at frac >= 0.90 (after tail passes hole center) must have 0 pixels outside disc!
    all_pass = True
    post_entry_max_stray = 0
    for key, _, label in STAGES:
        for frac, total_diff, outside_diff in audit_results[key]:
            if frac >= 0.90:
                post_entry_max_stray = max(post_entry_max_stray, outside_diff)
                if outside_diff > 0:
                    all_pass = False

    status = "PASS (0 stray px after tail entry)" if all_pass else f"FAIL ({post_entry_max_stray} stray px)"
    draw.text(
        (pad, 14),
        f"Burrow Proof: {creature} ({register}) — 12 frames across last 30% of cycle — {status}",
        fill=(28, 16, 10),
        font=font_title,
    )

    y_cur = header_h
    for key, _, label in STAGES:
        ch = cell_heights[key]
        draw.text((pad, y_cur), f"Target: {label}", fill=(88, 52, 26), font=font_sec)
        y_cur += section_title_h
        for idx, (frac, im) in enumerate(frames[key]):
            row = idx // cols
            col = idx % cols
            x = pad + col * (cell_w + gap)
            y = y_cur + row * (ch + label_h + gap)
            resized = im.resize((cell_w, ch), Image.Resampling.LANCZOS)
            sheet.paste(resized, (x, y))
            draw.rectangle([x, y, x + cell_w - 1, y + ch - 1], outline=(182, 166, 136), width=1)
            _, total_diff, outside_diff = audit_results[key][idx]
            tag = f"t={frac*100:4.1f}%  px:{total_diff:4d}"
            if frac >= 0.90:
                tag += "  [IN:0]" if outside_diff == 0 else f"  [STRAY:{outside_diff}]"
            color = (34, 110, 56) if (frac >= 0.90 and outside_diff == 0) else ((180, 24, 24) if (frac >= 0.90) else (70, 55, 42))
            draw.text((x + 2, y + ch + 3), tag, fill=color, font=font_lbl)
        y_cur += 2 * (ch + label_h + gap) + 12

    sheet.save(out_png)
    print(f"Saved {out_png} — {status}")
    for key, _, label in STAGES:
        post = [(f"{f*100:.1f}%", td, od) for f, td, od in audit_results[key] if f >= 0.90]
        print(f"  {label:10s} post-entry frames: {post}")
    if not all_pass:
        sys.exit(2)
    return out_png


if __name__ == "__main__":
    creature = sys.argv[1] if len(sys.argv) > 1 else "draco-bipes"
    register = sys.argv[2] if len(sys.argv) > 2 else "engraved"
    head_var = sys.argv[3] if len(sys.argv) > 3 else ""
    render_proof(creature, register, head_var)
