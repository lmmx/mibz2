/**
 * Slot 2: Burrowers (four)
 * Knackering-style bodies with Aberdeen serpent and calendar serpent traced heads.
 * Each crawls along a phrase, weaves, pauses to look around, dives cleanly into
 * an "o" counter or "i" dot through a zero-stray-pixel shifted <mask> (no fade,
 * no drawn hole), and re-emerges cleanly.
 */
(function (root) {
  'use strict';

  const SVG_NS = 'http://www.w3.org/2000/svg';

  const BURROWER_PRESETS = [
    {
      id: 'burrower-1',
      creature: 'aspis',
      headVariant: 'aberdeen-head',
      wash: '#ff2e12',
      wash2: '#ffb800',
      kind: 'o',
      dur: 12.5
    },
    {
      id: 'burrower-2',
      creature: 'spotted-serpent',
      headVariant: 'calendar-head',
      wash: '#1b5cff',
      wash2: '#00d97e',
      kind: 'idot',
      dur: 13.0
    },
    {
      id: 'burrower-3',
      creature: 'aspis',
      headVariant: 'aberdeen-head',
      wash: '#00d97e',
      wash2: '#ffb800',
      kind: 'o',
      dur: 12.0
    },
    {
      id: 'burrower-4',
      creature: 'spotted-serpent',
      headVariant: 'calendar-head',
      wash: '#5b2bff',
      wash2: '#00c8d7',
      kind: 'idot',
      dur: 13.5
    }
  ];

  function makeSvgEl(tag, attrs, parent) {
    const n = document.createElementNS(SVG_NS, tag);
    for (const k in attrs) {
      if (attrs[k] !== undefined && attrs[k] !== null) n.setAttribute(k, String(attrs[k]));
    }
    if (parent) parent.appendChild(n);
    return n;
  }

  /**
   * Mount a single burrower into an inline <svg> containing a phrase.
   * opts: { svgEl, phrase, charIndex, startCharIndex, kind ('o'|'idot'), presetIndex (0..3),
   *         fontSize, width, height, textX, textY, textColor, wash, wash2, dur, autoPause }
   */
  function mountBurrower(opts) {
    if (!opts || !opts.svgEl || !root.BestiaryCrawler) return null;
    const preset = BURROWER_PRESETS[(opts.presetIndex || 0) % BURROWER_PRESETS.length];
    const svg = opts.svgEl;
    const w = opts.width || 300;
    const h = opts.height || 66;
    const fontSize = opts.fontSize || 17;
    const tx = opts.textX !== undefined ? opts.textX : 14;
    const ty = opts.textY !== undefined ? opts.textY : Math.round(h * 0.64);
    const kind = opts.kind || preset.kind;

    svg.setAttribute('viewBox', `0 0 ${w} ${h}`);
    svg.innerHTML = '';
    root.BestiaryCrawler.ensureSharedDefs(svg);

    const textEl = makeSvgEl('text', {
      x: tx,
      y: ty,
      'font-family': 'Georgia, "Iowan Old Style", serif',
      'font-size': fontSize,
      fill: opts.textColor || 'currentColor'
    }, svg);
    textEl.textContent = opts.phrase;

    const chBox = textEl.getExtentOfChar(opts.charIndex);
    const hole = kind === 'idot'
      ? {
          x: chBox.x + chBox.width * 0.50,
          y: chBox.y + chBox.height * 0.16,
          r: Math.max(1.8, chBox.width * 0.28)
        }
      : {
          x: chBox.x + chBox.width * 0.50,
          y: chBox.y + chBox.height * 0.62,
          r: Math.max(2.2, chBox.width * 0.27)
        };

    let startHole = null;
    let x0 = 12;
    let y0 = ty + (kind === 'idot' ? -fontSize * 0.34 : 4);
    if (opts.startCharIndex !== undefined && opts.startCharIndex !== null) {
      const sBox = textEl.getExtentOfChar(opts.startCharIndex);
      startHole = {
        x: sBox.x + sBox.width * 0.50,
        y: sBox.y + sBox.height * 0.62,
        r: Math.max(2.2, sBox.width * 0.27)
      };
      x0 = startHole.x;
      y0 = startHole.y;
    }

    const dx = hole.x - x0;
    const cx1 = x0 + dx * 0.34;
    const cy1 = y0 - (kind === 'idot' ? 12 : 11);
    const cx2 = x0 + dx * 0.72;
    const cy2 = hole.y + (kind === 'idot' ? -4 : 7);
    const pathD = `M${x0.toFixed(1)},${y0.toFixed(1)} C${cx1.toFixed(1)},${cy1.toFixed(1)} ${cx2.toFixed(1)},${cy2.toFixed(1)} ${hole.x.toFixed(1)},${hole.y.toFixed(1)}`;

    const crawler = root.BestiaryCrawler.makeCrawler({
      path: pathD,
      hole,
      startHole,
      creature: opts.creature || preset.creature,
      headVariant: opts.headVariant || preset.headVariant,
      wash: opts.wash || preset.wash,
      wash2: opts.wash2 || preset.wash2,
      scale: opts.scale !== undefined ? opts.scale : (fontSize / 16) * 0.86,
      dur: opts.dur || preset.dur
    });

    svg.appendChild(crawler);
    if (opts.autoPause !== false) {
      root.BestiaryCrawler.observeVisibility(svg);
    }
    return crawler;
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.BURROWER_PRESETS = BURROWER_PRESETS;
  root.BestiarySlots.mountBurrower = mountBurrower;
}(typeof globalThis !== 'undefined' ? globalThis : this));
