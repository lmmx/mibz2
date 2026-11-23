"""Clean traced creatures: drop specks, seal cracks, and give one bold outline.

Tracing leaves three kinds of damage: specks (tiny stray fragments), white cracks between
neighbouring colour patches, and ragged, broken outer edges. In a headless browser, so that
every path's real geometry is known, this:
- removes any path whose box is smaller than MIN_AREA square units, unless it is ink (eyes,
  nostrils and claw lines are small but ink);
- strokes every patch thinly in its own colour, painted under its fill, which seals hairline
  cracks without changing the drawing;
- draws one continuous iron-gall outline round the whole creature, a dilation of its silhouette
  painted beneath it (the bold, confident line of the manuscripts).

    uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/scripts/bestiary_clean.py \
        CHROME IN.svg [IN.svg ...]

Each file is cleaned in place.
"""

import sys
from pathlib import Path

from playwright.sync_api import sync_playwright

MIN_AREA = 9.0
INK = "#19100b"
CLEAN_JS = """
([minArea, ink]) => {
  const svg = document.querySelector('svg');
  const isInk = c => { const m = /^#([0-9a-f]{6})$/i.exec(c || ''); if (!m) return false;
    const v = parseInt(m[1], 16), r = v >> 16, g = (v >> 8) & 255, b = v & 255; return r + g + b < 140; };
  let dropped = 0;
  for (const p of [...svg.querySelectorAll('path')]) {
    const b = p.getBBox(), fill = p.getAttribute('fill') || '';
    if (b.width * b.height < minArea && !isInk(fill) && !fill.startsWith('var(')) { p.remove(); dropped++; continue; }
    const f = fill.startsWith('var(') ? fill : fill;
    if (f && f !== 'none') { p.setAttribute('stroke', f); p.setAttribute('stroke-width', '0.7');
      p.setAttribute('stroke-linejoin', 'round'); p.setAttribute('paint-order', 'stroke'); }
  }
  // the outline: a dilated silhouette in ink beneath the creature
  const NS = 'http://www.w3.org/2000/svg';
  let defs = svg.querySelector(':scope > defs');
  if (!defs) { defs = document.createElementNS(NS, 'defs'); svg.insertBefore(defs, svg.firstChild); }
  const id = 'bold-outline-' + Math.random().toString(36).slice(2, 7);
  defs.insertAdjacentHTML('beforeend',
    `<filter id="${id}" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">` +
    `<feMorphology in="SourceAlpha" operator="dilate" radius="1.6" result="d"/>` +
    `<feFlood flood-color="${ink}"/><feComposite in2="d" operator="in" result="o"/>` +
    `<feMerge><feMergeNode in="o"/><feMergeNode in="SourceGraphic"/></feMerge></filter>`);
  const root = [...svg.children].find(c => c.tagName === 'g');
  root.setAttribute('filter', `url(#${id})`);
  return [dropped, new XMLSerializer().serializeToString(svg)];
}
"""


def main(chrome, files):
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path=chrome)
        pg = b.new_page()
        for f in files:
            src = Path(f).read_text(encoding="utf-8")
            pg.set_content(f"<html><body>{src}</body></html>")
            dropped, out = pg.evaluate(CLEAN_JS, [MIN_AREA, INK])
            Path(f).write_text(out + "\n", encoding="utf-8")
            print(f"{f}: {dropped} specks dropped, {len(out):,} bytes")
        b.close()


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2:])
