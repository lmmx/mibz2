/**
 * Bestiary Crawler & Heraldic Drape Library
 * Zero-dependency SMIL SVG rig for crawling serpents, burrow masking,
 * bubble-border draping, heraldic rest poses, and border moonwalking.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) {
    module.exports = factory();
  } else {
    root.BestiaryCrawler = factory();
  }
}(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const INK = '#1c100a';
  const LEAD = '#f6efe2';
  const GOLD_LIGHT = '#fce38a';
  const GOLD_MID = '#d9a426';
  const GOLD_DEEP = '#946510';
  const CLAUDE_ORANGE = '#F2683A';

  let uidCounter = 0;
  function uid(prefix) {
    uidCounter += 1;
    return `${prefix}-${uidCounter}`;
  }

  function el(tag, attrs, parent) {
    const node = document.createElementNS(SVG_NS, tag);
    if (attrs) {
      for (const k in attrs) {
        if (attrs[k] !== undefined && attrs[k] !== null) {
          node.setAttribute(k, String(attrs[k]));
        }
      }
    }
    if (parent) parent.appendChild(node);
    return node;
  }

  function f1(n) { return (Math.round(n * 10) / 10).toString(); }
  function f2(n) { return (Math.round(n * 100) / 100).toString(); }
  function f4(n) { return (Math.round(n * 10000) / 10000).toString(); }
  function clamp(v, a, b) { return v < a ? a : (v > b ? b : v); }

  /* ------------------------------------------------------------------ *
   * Species registry: proportions (12–20 head-lengths), head archetypes,
   * appendages, and default washes in engraved & illuminated registers.
   * ------------------------------------------------------------------ */
  const CREATURES = {
    'draco-bipes': {
      label: 'Draco bipes apteros',
      head: 'wyvern',
      legs: 2,
      wings: false,
      twoHeaded: false,
      pair: false,
      bodyLen: 68,       // at scale = 1 (16px text)
      maxWidth: 9.2,
      headScale: 0.34,
      motion: 'gait',
      wash: '#8b6d3f',
      wash2: '#5f4628',
      illum: '#d8342a',
      illum2: '#1d3f9e',
      widthFn(u) {
        if (u < 0.22) return 0.34 + 0.14 * (0.5 - 0.5 * Math.cos((u / 0.22) * Math.PI));
        if (u < 0.40) return 0.48 + 0.52 * (0.5 - 0.5 * Math.cos(((u - 0.22) / 0.18) * Math.PI));
        if (u < 0.60) return 1.00 - 0.54 * (0.5 - 0.5 * Math.cos(((u - 0.40) / 0.20) * Math.PI));
        return 0.46 * Math.pow(Math.max(0, 1 - (u - 0.60) / 0.40), 1.3) + 0.04;
      }
    },
    'draco-alatus': {
      label: 'Draco alatus (Wyvern)',
      head: 'wyvern',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      bodyLen: 72,
      maxWidth: 8.6,
      headScale: 0.34,
      motion: 'gait',
      wash: '#3b83d6',
      wash2: '#7a4a26',
      illum: '#1d3f9e',
      illum2: '#3f9a7a',
      widthFn(u) {
        if (u < 0.24) return 0.32 + 0.18 * (0.5 - 0.5 * Math.cos((u / 0.24) * Math.PI));
        if (u < 0.42) return 0.50 + 0.50 * (0.5 - 0.5 * Math.cos(((u - 0.24) / 0.18) * Math.PI));
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.42) / 0.58), 1.15) + 0.04;
      }
    },
    'aspis': {
      label: 'Aspis (Biscione)',
      head: 'serpent',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: false,
      bodyLen: 82,
      maxWidth: 7.4,
      headScale: 0.30,
      motion: 'undulate',
      wash: '#a3141c',
      wash2: '#6e130f',
      illum: '#1d3f9e',
      illum2: '#d8342a',
      widthFn(u) {
        if (u < 0.14) return 0.42 + 0.32 * (u / 0.14);
        if (u < 0.36) return 0.74 + 0.26 * Math.sin(((u - 0.14) / 0.22) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.36) / 0.64), 1.18) + 0.04;
      }
    },
    'spotted-serpent': {
      label: 'Basiliscus (Gheeraerts)',
      head: 'basilisk',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      bodyLen: 74,
      maxWidth: 8.8,
      headScale: 0.35,
      motion: 'gait',
      wash: '#9c4c1e',
      wash2: '#5c2d12',
      illum: '#3f9a7a',
      illum2: '#d8342a',
      widthFn(u) {
        if (u < 0.18) return 0.36 + 0.30 * (u / 0.18);
        if (u < 0.36) return 0.66 + 0.34 * Math.sin(((u - 0.18) / 0.18) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.36) / 0.64), 1.25) + 0.04;
      }
    },
    'basilisk': {
      label: 'Basiliscus coronatus',
      head: 'basilisk',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      bodyLen: 74,
      maxWidth: 8.8,
      headScale: 0.35,
      motion: 'gait',
      wash: '#9c4c1e',
      wash2: '#5c2d12',
      illum: '#3f9a7a',
      illum2: '#d8342a',
      widthFn(u) {
        if (u < 0.18) return 0.36 + 0.30 * (u / 0.18);
        if (u < 0.36) return 0.66 + 0.34 * Math.sin(((u - 0.18) / 0.18) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.36) / 0.64), 1.25) + 0.04;
      }
    },
    'basilisk-6': {
      label: 'Basiliscus hexapus (Aldrovandi)',
      head: 'basilisk',
      legs: 6,
      wings: false,
      twoHeaded: false,
      pair: false,
      bodyLen: 70,
      maxWidth: 9.6,
      headScale: 0.34,
      motion: 'gait',
      wash: '#7a5a2c',
      wash2: '#4e3718',
      illum: '#d8342a',
      illum2: '#d49a1a',
      widthFn(u) {
        if (u < 0.18) return 0.34 + 0.24 * (u / 0.18);
        if (u < 0.42) return 0.58 + 0.42 * Math.sin(((u - 0.18) / 0.24) * Math.PI * 0.5);
        if (u < 0.64) return 1.00 - 0.48 * ((u - 0.42) / 0.22);
        return 0.52 * Math.pow(Math.max(0, 1 - (u - 0.64) / 0.36), 1.25) + 0.04;
      }
    },
    'knotted-pair': {
      label: 'Serpentes Nexi',
      head: 'serpent',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: true,
      bodyLen: 80,
      maxWidth: 6.8,
      headScale: 0.29,
      motion: 'undulate',
      wash: '#d22f1c',
      wash2: '#1d3f9e',
      illum: '#d8342a',
      illum2: '#1d3f9e',
      widthFn(u) {
        if (u < 0.20) return 0.46 + 0.54 * Math.sin((u / 0.20) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.20) / 0.80), 1.15) + 0.05;
      }
    },
    'ouroboros': {
      label: 'Ouroboros alatus',
      head: 'wyvern',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      bodyLen: 78,
      maxWidth: 8.2,
      headScale: 0.32,
      motion: 'concertina',
      wash: '#7a5c32',
      wash2: '#4f3a1c',
      illum: '#3f9a7a',
      illum2: '#d49a1a',
      widthFn(u) {
        if (u < 0.25) return 0.42 + 0.58 * Math.sin((u / 0.25) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.25) / 0.75), 1.1) + 0.06;
      }
    },
    'amphisbaena': {
      label: 'Amphisbaena',
      head: 'serpent',
      legs: 0,
      wings: false,
      twoHeaded: true,
      pair: false,
      bodyLen: 76,
      maxWidth: 7.2,
      headScale: 0.30,
      motion: 'concertina',
      wash: '#8c3b28',
      wash2: '#4d281c',
      illum: '#d8342a',
      illum2: '#3f9a7a',
      widthFn(u) {
        // Symmetrical spindle, thick in the middle, slender at both necks
        const d = Math.abs(u - 0.5) * 2;
        return 0.42 + 0.58 * Math.cos(d * Math.PI * 0.5);
      }
    },
    'cute': {
      label: 'Wendling (odd one out)',
      head: 'cute',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: false,
      bodyLen: 54,
      maxWidth: 8.4,
      headScale: 0.42,
      motion: 'undulate',
      wash: '#d8342a',
      wash2: '#e85d38',
      illum: '#d8342a',
      illum2: '#f28b30',
      widthFn(u) {
        if (u < 0.28) return 0.72 + 0.28 * Math.sin((u / 0.28) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.28) / 0.72), 0.9) + 0.14;
      }
    }
  };
  // Friendly aliases
  CREATURES.spotted = CREATURES['spotted-serpent'];
  CREATURES.wyvern = CREATURES['draco-alatus'];
  CREATURES.biscione = CREATURES.aspis;

  const REGISTERS = ['engraved', 'illuminated'];

  /* ------------------------------------------------------------------ *
   * Shared SVG <defs> helper: burnished gold leaf, stippled diaper,
   * and lightweight vellum/pigment mottle filter.
   * ------------------------------------------------------------------ */
  function ensureSharedDefs(svgRoot) {
    if (!svgRoot) return null;
    let defs = svgRoot.querySelector('defs[data-bestiary-shared="1"]');
    if (defs) return defs;
    defs = el('defs', { 'data-bestiary-shared': '1' });
    svgRoot.insertBefore(defs, svgRoot.firstChild);

    // Burnished gold leaf linear gradient with bright specular band
    const goldGrad = el('linearGradient', {
      id: 'bestiary-gold-leaf', x1: '0%', y1: '0%', x2: '100%', y2: '100%'
    }, defs);
    el('stop', { offset: '0%', 'stop-color': GOLD_DEEP }, goldGrad);
    el('stop', { offset: '26%', 'stop-color': GOLD_MID }, goldGrad);
    el('stop', { offset: '44%', 'stop-color': GOLD_LIGHT }, goldGrad);
    el('stop', { offset: '52%', 'stop-color': '#fff6cf' }, goldGrad);
    el('stop', { offset: '62%', 'stop-color': GOLD_MID }, goldGrad);
    el('stop', { offset: '100%', 'stop-color': GOLD_DEEP }, goldGrad);

    // Punched / stippled diaper pattern over gold leaf
    const diaper = el('pattern', {
      id: 'bestiary-gold-diaper', width: '6', height: '6', patternUnits: 'userSpaceOnUse'
    }, defs);
    el('rect', { width: '6', height: '6', fill: 'url(#bestiary-gold-leaf)' }, diaper);
    el('path', {
      d: 'M0,3 L3,0 L6,3 L3,6 Z',
      fill: 'none', stroke: '#7a520c', 'stroke-width': '0.35', opacity: '0.45'
    }, diaper);
    el('circle', { cx: '3', cy: '3', r: '0.45', fill: '#fff5c8', opacity: '0.8' }, diaper);
    el('circle', { cx: '0', cy: '0', r: '0.4', fill: '#6b4508', opacity: '0.55' }, diaper);

    // Subtle vellum / pigment mottle filter (single-octave, low cost on mobile)
    const mottle = el('filter', {
      id: 'bestiary-mottle', x: '-4%', y: '-4%', width: '108%', height: '108%'
    }, defs);
    el('feTurbulence', {
      type: 'fractalNoise', baseFrequency: '0.09', numOctaves: '2', seed: '7', result: 'n'
    }, mottle);
    el('feColorMatrix', {
      type: 'matrix', in: 'n', result: 'm',
      values: '0 0 0 0 0.92  0 0 0 0 0.88  0 0 0 0 0.78  0 0 0 0.14 0'
    }, mottle);
    el('feComposite', { in: 'm', in2: 'SourceGraphic', operator: 'atop', result: 't' }, mottle);
    el('feBlend', { in: 'SourceGraphic', in2: 't', mode: 'multiply' }, mottle);

    return defs;
  }

  /* ------------------------------------------------------------------ *
   * Head Library (facing +x, neck at 0,0, ~20 units long)
   * Four distinct archetypes + cute wendling + claude-ruffed collar.
   * ZERO dog heads!
   * ------------------------------------------------------------------ */
  function addClaudeRuff(parent, register) {
    // 18 uneven rounded radiating petals in Claude orange (#F2683A) with modulated ink outline
    // and a tiny hidden ^ω^ kaomoji tucked into the pale negative space at the nape.
    const ruff = el('g', { class: 'bestiary-claude-ruff' }, parent);
    const petalCount = 18;
    for (let i = 0; i < petalCount; i += 1) {
      const a = (i / petalCount) * Math.PI * 2 + 0.14;
      // Leave slight gap right in front of snout so jaws read crisply
      const deg = ((a * 180) / Math.PI) % 360;
      if (deg < 16 || deg > 344) continue;
      // Uneven organic length modulation (wobbling slightly like thebes's spark)
      const len = 8.4 + ((i * 7) % 5) * 0.95 + (i % 2 === 0 ? 1.1 : -0.4);
      const w = 1.45 + ((i * 3) % 3) * 0.28;
      const tilt = ((i % 3) - 1) * 0.08;
      const cos = Math.cos(a + tilt);
      const sin = Math.sin(a + tilt);
      const nx = -sin;
      const ny = cos;
      const bx = -1.2 + cos * 3.1;
      const by = sin * 2.9;
      const tx = bx + cos * len;
      const ty = by + sin * len;
      const mx = bx + cos * (len * 0.62);
      const my = by + sin * (len * 0.62);
      const d = `M${f2(bx + nx * w * 0.65)},${f2(by + ny * w * 0.65)} ` +
        `Q${f2(mx + nx * w)},${f2(my + ny * w)} ${f2(tx)},${f2(ty)} ` +
        `Q${f2(mx - nx * w)},${f2(my - ny * w)} ${f2(bx - nx * w * 0.65)},${f2(by - ny * w * 0.65)} Z`;
      el('path', {
        d,
        fill: CLAUDE_ORANGE,
        'fill-opacity': register === 'illuminated' ? '0.96' : '0.84',
        stroke: 'none'
      }, ruff);
      // Modulated ink shadow edge along one side + subtle tip accent
      const inkEdge = `M${f2(bx - nx * w * 0.65)},${f2(by - ny * w * 0.65)} ` +
        `Q${f2(mx - nx * w * 1.06)},${f2(my - ny * w * 1.06)} ${f2(tx)},${f2(ty)} ` +
        `Q${f2(mx + nx * w * 0.72)},${f2(my + ny * w * 0.72)} ${f2(tx - cos * 2.2 + nx * w * 0.82)},${f2(ty - sin * 2.2 + ny * w * 0.82)}`;
      el('path', {
        d: inkEdge,
        fill: 'none',
        stroke: INK,
        'stroke-width': i % 2 === 0 ? '0.72' : '0.52',
        'stroke-linecap': 'round'
      }, ruff);
      if (register === 'illuminated') {
        el('path', {
          d: `M${f2(bx + cos * 2.0)},${f2(by + sin * 2.0)} L${f2(bx + cos * (len * 0.68))},${f2(by + sin * (len * 0.68))}`,
          stroke: LEAD, 'stroke-width': '0.32', opacity: '0.7', 'stroke-linecap': 'round'
        }, ruff);
      }
    }
    // Tiny subtle white badge & hidden ^ω^ kaomoji in negative space behind occiput
    const badge = el('g', { transform: 'translate(-2.4,0)', opacity: '0.92' }, ruff);
    el('ellipse', { cx: '0', cy: '0', rx: '3.2', ry: '2.5', fill: LEAD, stroke: INK, 'stroke-width': '0.38' }, badge);
    el('path', {
      d: 'M-2.1,-.4 l.7,-.7 .7,.7 M.7,-.4 l.7,-.7 .7,.7 M-1.1,.8 q.55,.8 1.1,0 q.55,.8 1.1,0',
      fill: 'none', stroke: INK, 'stroke-width': '0.34', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
    }, badge);
  }

  function buildHead(type, opts) {
    opts = opts || {};
    const register = opts.register || 'engraved';
    const wash = opts.wash || '#8b6d3f';
    const isIllum = register === 'illuminated';
    const isClaudeRuffed = opts.headVariant === 'claude-ruffed' || type === 'claude-ruffed';
    const baseType = type === 'claude-ruffed' ? (opts.baseHead || 'wyvern') : type;
    const dur = opts.dur || 6.1;
    const flicks = opts.flicks || null;

    const g = el('g', { class: `bestiary-head bestiary-head-${baseType}` });

    if (isClaudeRuffed) {
      addClaudeRuff(g, register);
    }

    const fillMain = isIllum ? wash : wash;
    const fillOp = isIllum ? '0.95' : '0.36';
    const jawFillOp = isIllum ? '0.92' : '0.28';

    // Tongue timing derived from storyboard flicks (or gentle default idle flick)
    const tongueG = el('g', { class: 'bestiary-tongue' });
    let tKeyTimes = '0;.44;.49;.54;.59;.64;1';
    let tVals = '.04 1;.04 1;1 1;.38 1;1.06 1;.04 1;.04 1';
    let tOpVals = '0;0;1;1;1;0;0';
    let jVals = '1;1;14;5;15;2;1';
    if (flicks && flicks.length) {
      const kt = [0];
      const sc = ['.04 1'];
      const op = ['0'];
      const jw = ['1'];
      const sorted = flicks.slice().sort((a, b) => a.t0 - b.t0);
      for (const fl of sorted) {
        const a = clamp(fl.t0, 0.01, 0.96);
        const b = clamp(fl.t1, a + 0.02, 0.99);
        const m1 = a + (b - a) * 0.25;
        const m2 = a + (b - a) * 0.52;
        const m3 = a + (b - a) * 0.78;
        if (a > kt[kt.length - 1] + 0.005) {
          kt.push(a); sc.push('.04 1'); op.push('0'); jw.push('1');
        }
        kt.push(m1, m2, m3, b);
        sc.push('1 1', '.38 1', '1.08 1', '.04 1');
        op.push('1', '1', '1', '0');
        jw.push('14', '6', '15', '1');
      }
      if (kt[kt.length - 1] < 1) {
        kt.push(1); sc.push('.04 1'); op.push('0'); jw.push('1');
      }
      tKeyTimes = kt.map(f2).join(';');
      tVals = sc.join(';');
      tOpVals = op.join(';');
      jVals = jw.join(';');
    }

    if (baseType === 'cute') {
      // Deliberately cute wend-style head (reserved for the "yuck" odd-one-out bubble)
      const headW = el('g', {}, g);
      el('path', {
        d: 'M-3,-4.5 C2,-7.6 10.5,-7.6 14.5,-3.6 C17.2,-0.8 17.0,3.2 13.8,5.4 C9.6,8.0 1.8,7.8 -3.0,4.5 Z',
        fill: fillMain, 'fill-opacity': isIllum ? '0.96' : '0.88',
        stroke: INK, 'stroke-width': '1.05', 'stroke-linejoin': 'round'
      }, headW);
      // Soft blush cheek highlight (lead white, never pink)
      el('ellipse', { cx: '7.5', cy: '3.2', rx: '2.2', ry: '1.2', fill: LEAD, opacity: '0.35' }, headW);
      // Big soft dark endearing eye with bright catchlights
      el('circle', { cx: '9.2', cy: '-1.4', r: '2.65', fill: INK }, headW);
      el('circle', { cx: '8.3', cy: '-2.3', r: '0.95', fill: LEAD }, headW);
      el('circle', { cx: '10.2', cy: '-0.5', r: '0.45', fill: LEAD }, headW);
      // Tiny endearing upturned smile + mini tongue blep
      el('path', {
        d: 'M11.6,2.1 Q13.4,3.5 15.2,1.9',
        fill: 'none', stroke: INK, 'stroke-width': '0.75', 'stroke-linecap': 'round'
      }, headW);
      const blep = el('g', { transform: 'translate(14.8,2.0)' }, headW);
      el('path', {
        d: 'M0,0 C2.6,-0.2 4.6,0.6 5.6,1.3 C4.4,2.1 2.2,1.8 0,0.9 Z',
        fill: '#d8342a', stroke: INK, 'stroke-width': '0.45'
      }, blep);
      el('animateTransform', {
        attributeName: 'transform', type: 'scale',
        values: '.2 1;.2 1;1 1;1 1;.2 1;.2 1',
        keyTimes: '0;.46;.52;.66;.72;1',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, blep);
      return g;
    }

    if (baseType === 'basilisk') {
      // Crowned cockerel-headed basilisk (after Aldrovandi & Gheeraerts):
      // serrated comb + trefoil gold crown, hooked raptorial beak, hanging wattle,
      // brow ridge over sharp eye, split tongue.
      const throat = el('path', { d: 'M7,-.1 L15.8,-.6 L14.5,1.8 L7.2,1.5 Z', fill: INK, opacity: '.84' }, g);
      void throat;

      // Hanging wattle beneath lower beak
      el('path', {
        d: 'M7.5,2.2 C6.4,4.8 7.8,7.2 9.8,6.8 C11.4,6.4 11.8,4.2 10.6,2.2 Z',
        fill: isIllum ? '#d8342a' : wash, 'fill-opacity': isIllum ? '0.96' : '0.48',
        stroke: INK, 'stroke-width': '0.62'
      }, g);
      el('path', {
        d: 'M8.4,3.4 l.6,2.2 M9.8,3.2 l.3,2.1',
        stroke: INK, 'stroke-width': '0.35', 'stroke-linecap': 'round'
      }, g);

      // Lower hooked beak (jaw)
      const jawPiv = el('g', { transform: 'translate(6.6,1.2)' }, g);
      const jaw = el('g', {}, jawPiv);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, jaw);
      el('path', {
        d: 'M-3.0,-.2 L9.8,.2 C10.4,.3 10.2,1.1 9.0,1.5 C5.4,2.4 .4,2.2 -3.6,1.4 Z',
        fill: isIllum ? '#d49a1a' : wash, 'fill-opacity': jawFillOp
      }, jaw);
      el('path', {
        d: 'M-3.6,1.4 C.4,2.3 5.8,2.5 9.2,1.5 C10.2,1.2 10.4,.3 9.8,.2 C7.2,1.1 2.2,1.6 -3.6,1.4 Z',
        fill: INK
      }, jaw);

      // Split tongue
      const tPiv = el('g', { transform: 'translate(10.6,.2)' }, g);
      tPiv.appendChild(tongueG);
      el('animateTransform', {
        attributeName: 'transform', type: 'scale',
        values: tVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, tongueG);
      el('animate', {
        attributeName: 'opacity', values: tOpVals, keyTimes: tKeyTimes,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, tongueG);
      el('path', {
        d: 'M0,0 C3.2,-.2 6.6,.3 9.8,0 M9.6,0 L13.8,-2.0 M9.6,0 L13.8,2.1',
        fill: 'none', stroke: isIllum ? '#d8342a' : INK, 'stroke-width': '0.64', 'stroke-linecap': 'round'
      }, tongueG);

      // Serrated cockerel comb behind crown
      el('path', {
        d: 'M-1.5,-4.0 C-2.6,-6.8 -.8,-8.4 .8,-6.8 C1.6,-8.8 4.0,-8.6 4.4,-6.4 C5.6,-8.0 7.6,-7.2 7.2,-4.6 Z',
        fill: isIllum ? '#d8342a' : wash, 'fill-opacity': isIllum ? '0.96' : '0.46',
        stroke: INK, 'stroke-width': '0.58', 'stroke-linejoin': 'round'
      }, g);

      // Upper cranium + hooked raptorial beak
      el('path', {
        d: 'M-3.2,-3.4 C-.4,-5.4 4.2,-5.8 8.2,-4.8 C11.4,-4.0 15.6,-3.2 18.8,-1.4 C19.8,-.8 20.1,.4 18.8,1.2 C18.2,.3 16.4,-.4 12.2,-.6 C8.4,-.8 3.6,.8 -2.8,2.0 Z',
        fill: fillMain, 'fill-opacity': fillOp
      }, g);
      // Beak tip gold/horn tint in illuminated register
      if (isIllum) {
        el('path', {
          d: 'M11.6,-3.6 C14.8,-2.9 17.6,-2.1 19.0,-1.2 C19.8,-.6 19.8,.3 18.8,1.1 C17.8,.2 15.8,-.5 11.4,-.7 Z',
          fill: 'url(#bestiary-gold-leaf)', stroke: INK, 'stroke-width': '0.5'
        }, g);
      }
      // Fine feather/cheek burin hatching
      el('path', {
        d: 'M-1.6,-2.2 C0,-3.2 2.2,-3.6 4.0,-3.2 M-.6,-0.5 C.6,.4 .6,1.2 -.2,1.8 M1.2,-.7 C2.2,.2 2.2,1.0 1.4,1.6 M3.0,-.8 C4.0,0 4.0,.8 3.2,1.4',
        fill: 'none', stroke: INK, 'stroke-width': '0.44', 'stroke-linecap': 'round'
      }, g);
      // Modulated upper contour + dark hooked beak cutting edge
      el('path', {
        d: 'M-3.0,-3.4 C-.4,-5.4 4.2,-5.8 8.2,-4.8 M11.4,-3.8 C14.8,-3.1 17.6,-2.2 19.2,-1.2',
        fill: 'none', stroke: INK, 'stroke-width': '0.62', 'stroke-linecap': 'round'
      }, g);
      el('path', {
        d: 'M19.2,-1.2 C20.2,-.5 20.1,.6 18.8,1.3 C18.2,.3 16.4,-.2 12.0,-.4 C8.8,-.5 5.4,.2 2.6,1.1 C5.2,-.2 8.8,-1.0 12.6,-1.0 C16.0,-1.0 18.2,-.7 19.2,-1.2 Z',
        fill: INK
      }, g);

      // Small gold heraldic crown perched on brow
      el('path', {
        d: 'M3.2,-5.2 L2.8,-8.4 L5.0,-6.6 L6.8,-9.2 L8.2,-6.4 L10.2,-8.0 L9.4,-4.6 Z',
        fill: isIllum ? 'url(#bestiary-gold-leaf)' : '#d49a1a',
        'fill-opacity': isIllum ? '1' : '0.72',
        stroke: INK, 'stroke-width': '0.52', 'stroke-linejoin': 'round'
      }, g);

      // Sharp round eye under fierce brow ridge
      el('path', {
        d: 'M6.8,-2.4 C7.8,-3.3 9.4,-3.3 10.4,-2.4 C9.4,-1.6 7.8,-1.6 6.8,-2.4 Z',
        fill: LEAD, stroke: INK, 'stroke-width': '0.42'
      }, g);
      el('circle', { cx: '8.7', cy: '-2.4', r: '0.65', fill: INK }, g);
      el('circle', { cx: '8.5', cy: '-2.58', r: '0.18', fill: LEAD }, g);
      el('path', {
        d: 'M6.2,-2.6 C7.5,-3.9 9.6,-3.9 10.9,-2.7 C9.6,-3.4 7.6,-3.4 6.2,-2.6 Z',
        fill: INK
      }, g);
      return g;
    }

    if (baseType === 'serpent') {
      // Visconti Biscione / heraldic serpent head: narrow reptilian wedge,
      // tiny heraldic crown, strong brow ridge, fine forked tongue.
      el('path', { d: 'M6.6,-.2 L16.8,-.8 L15.4,1.5 L6.8,1.3 Z', fill: INK, opacity: '.82' }, g);

      const jawPiv = el('g', { transform: 'translate(5.8,1.1)' }, g);
      const jaw = el('g', {}, jawPiv);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, jaw);
      el('path', {
        d: 'M-3.5,-.2 L11.6,.2 C12.0,.4 11.8,1.1 10.8,1.4 C7.2,2.1 1.6,2.2 -3.8,1.4 Z',
        fill: fillMain, 'fill-opacity': jawFillOp
      }, jaw);
      el('path', {
        d: 'M-3.8,1.4 C.8,2.2 7.0,2.2 10.8,1.4 C11.8,1.1 12.0,.4 11.6,.2 C9.0,1.0 3.0,1.5 -3.8,1.4 Z',
        fill: INK
      }, jaw);

      const tPiv = el('g', { transform: 'translate(11.4,.2)' }, g);
      tPiv.appendChild(tongueG);
      el('animateTransform', {
        attributeName: 'transform', type: 'scale',
        values: tVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, tongueG);
      el('animate', {
        attributeName: 'opacity', values: tOpVals, keyTimes: tKeyTimes,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, tongueG);
      el('path', {
        d: 'M0,0 C3.6,-.1 7.2,.3 10.8,0 M10.6,0 L15.0,-2.0 M10.6,0 L15.0,2.1',
        fill: 'none', stroke: isIllum ? '#d8342a' : INK, 'stroke-width': '0.58', 'stroke-linecap': 'round'
      }, tongueG);

      // Sleek narrow reptilian wedge skull
      el('path', {
        d: 'M-3.0,-2.8 C-.4,-4.2 3.8,-4.7 8.2,-4.0 C11.6,-3.5 16.0,-2.8 19.6,-1.6 C20.2,-1.3 19.8,-.7 18.2,-.7 C14.6,-.8 10.2,-.6 6.2,.2 C3.0,.8 -.4,1.4 -2.8,1.8 Z',
        fill: fillMain, 'fill-opacity': fillOp
      }, g);
      // Tiny fangs
      el('path', {
        d: 'M14.2,-.7 L14.6,.4 L15.0,-.7 M16.4,-.7 L16.8,.3 L17.2,-.7',
        fill: LEAD, stroke: INK, 'stroke-width': '0.24'
      }, g);
      // Burin scale / cheek lines
      el('path', {
        d: 'M-1.4,-2.0 C.6,-3.0 3.0,-3.3 5.2,-2.9 M.4,-.4 C1.4,.3 1.4,1.0 .6,1.5 M2.2,-.6 C3.2,.1 3.2,.8 2.4,1.3 M4.0,-.7 C4.9,-.1 4.9,.5 4.2,1.0',
        fill: 'none', stroke: INK, 'stroke-width': '0.42', 'stroke-linecap': 'round'
      }, g);
      // Modulated upper contour + dark lip
      el('path', {
        d: 'M-2.8,-2.8 C-.4,-4.2 3.8,-4.7 8.0,-4.0 M11.4,-3.5 C14.8,-2.9 17.8,-2.2 19.6,-1.5',
        fill: 'none', stroke: INK, 'stroke-width': '0.58', 'stroke-linecap': 'round'
      }, g);
      el('path', {
        d: 'M19.6,-1.4 C18.8,-.6 15.6,-.6 11.4,-.6 C8.4,-.5 5.4,.1 2.8,.9 C5.2,-.2 8.4,-.9 12.0,-1.0 C15.6,-1.1 18.4,-1.0 19.6,-1.4 Z',
        fill: INK
      }, g);
      // Tiny heraldic serpent crown
      el('path', {
        d: 'M2.2,-4.4 L1.8,-7.2 L4.0,-5.6 L5.6,-7.8 L7.0,-5.4 L8.8,-6.8 L8.0,-4.0 Z',
        fill: isIllum ? 'url(#bestiary-gold-leaf)' : '#d49a1a',
        'fill-opacity': isIllum ? '1' : '0.68',
        stroke: INK, 'stroke-width': '0.46', 'stroke-linejoin': 'round'
      }, g);
      // Small fierce eye under heavy brow
      el('path', {
        d: 'M7.4,-2.5 C8.4,-3.2 9.9,-3.2 10.8,-2.5 C9.9,-1.8 8.4,-1.8 7.4,-2.5 Z',
        fill: LEAD, stroke: INK, 'stroke-width': '0.38'
      }, g);
      el('circle', { cx: '9.2', cy: '-2.5', r: '0.55', fill: INK }, g);
      el('path', { d: 'M6.8,-2.7 C8.0,-3.8 10.0,-3.8 11.2,-2.8 C10.0,-3.3 8.1,-3.3 6.8,-2.7 Z', fill: INK }, g);
      return g;
    }

    // Default: 'wyvern' / drake head (swept-back horns, finned neck crest, sleek reptilian wedge,
    // small eye under brow ridge, fine teeth, barbed forked tongue)
    el('path', { d: 'M6.8,-.2 L17.6,-1.0 L16.2,1.8 L7.2,1.5 Z', fill: INK, opacity: '.84' }, g);

    // Lower jaw
    const jawPiv = el('g', { transform: 'translate(6.2,1.3)' }, g);
    const jaw = el('g', {}, jawPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, jaw);
    el('path', {
      d: 'M-4,-.3 C1,-.3 6.2,-.1 12.2,.2 C12.8,.3 12.7,1.2 11.8,1.5 C8.4,2.4 2.8,2.6 -2.6,2.1 C-4.8,1.9 -6.4,1.5 -7.4,1.1 Z',
      fill: fillMain, 'fill-opacity': jawFillOp
    }, jaw);
    el('path', {
      d: 'M4.6,-.1 L5.0,-1.0 L5.4,0 M6.8,0 L7.2,-.9 L7.6,.1 M9.0,.1 L9.4,-.8 L9.8,.2',
      fill: LEAD, stroke: INK, 'stroke-width': '0.24', 'stroke-linejoin': 'round'
    }, jaw);
    el('path', {
      d: 'M-7.2,1.1 C-4.2,2.1 2.2,2.7 7.6,2.2 C9.8,2.0 11.6,1.7 12.4,1.1 C12.8,.8 12.7,.2 12.2,.2 C10.8,.8 8.2,1.5 4.8,1.7 C.8,1.9 -3.6,1.5 -7.2,1.1 Z',
      fill: INK
    }, jaw);

    // Barbed forked tongue
    const tPiv = el('g', { transform: 'translate(11.6,.25)' }, g);
    tPiv.appendChild(tongueG);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: tVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tongueG);
    el('animate', {
      attributeName: 'opacity', values: tOpVals, keyTimes: tKeyTimes,
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tongueG);
    el('path', {
      d: 'M0,0 C3.2,-.1 6.4,.3 9.8,0 M9.6,0 C11.4,-.3 12.8,-1.2 14.2,-2.1 M9.6,0 C11.4,.5 12.9,1.4 14.3,2.2',
      fill: 'none', stroke: isIllum ? '#d8342a' : INK, 'stroke-width': '0.62', 'stroke-linecap': 'round'
    }, tongueG);

    // Swept-back horns & finned occiput crest
    el('path', {
      d: 'M3.0,-4.8 C-.4,-6.8 -3.4,-7.8 -5.4,-7.2 C-2.6,-6.0 -.2,-4.9 2.4,-4.2 Z',
      fill: isIllum ? 'url(#bestiary-gold-leaf)' : INK, stroke: INK, 'stroke-width': '0.45'
    }, g);
    el('path', {
      d: 'M-.6,-4.0 C-2.8,-5.2 -4.8,-5.4 -6.0,-4.4 C-3.6,-4.0 -1.8,-3.5 -.4,-3.1 Z',
      fill: INK
    }, g);

    // Sleek reptilian wedge skull (no dog muzzle!)
    el('path', {
      d: 'M-3.0,-3.4 C-.6,-4.8 3.6,-5.6 7.8,-5.0 C10.2,-4.6 12.2,-3.7 15.2,-3.3 C17.6,-3.0 19.6,-2.5 20.4,-1.8 C20.6,-1.3 19.8,-.7 18.4,-.8 C15.2,-1.0 11.4,-.7 7.6,.1 C4.6,.7 1.2,1.5 -2.6,2.2 Z',
      fill: fillMain, 'fill-opacity': fillOp
    }, g);
    if (isIllum) {
      el('path', {
        d: 'M.4,-4.4 C4.2,-4.9 8.6,-4.4 13.8,-3.1',
        fill: 'none', stroke: LEAD, 'stroke-width': '0.5', opacity: '0.78', 'stroke-linecap': 'round'
      }, g);
    }
    // Upper teeth
    el('path', {
      d: 'M11.6,-.8 L12.0,.2 L12.4,-.9 M14.2,-.9 L14.6,.1 L15.0,-.9 M16.8,-1.0 L17.2,0 L17.6,-1.0',
      fill: LEAD, stroke: INK, 'stroke-width': '0.24', 'stroke-linejoin': 'round'
    }, g);
    // Cheek hatching
    el('path', {
      d: 'M-1.4,-2.2 C.4,-3.2 2.8,-3.6 5.2,-3.2 M-.6,-.4 C.4,.4 .4,1.2 -.2,1.8 M1.2,-.6 C2.2,.2 2.2,1.0 1.4,1.6 M3.0,-.8 C4.0,0 4.0,.8 3.2,1.4',
      fill: 'none', stroke: INK, 'stroke-width': '0.42', 'stroke-linecap': 'round'
    }, g);
    // Modulated upper contour + dark lip
    el('path', {
      d: 'M-2.8,-3.4 C-.6,-4.8 3.2,-5.5 7.4,-5.0 M10.6,-4.3 C13.2,-3.7 16.4,-3.2 19.6,-2.4',
      fill: 'none', stroke: INK, 'stroke-width': '0.58', 'stroke-linecap': 'round'
    }, g);
    el('path', {
      d: 'M20.2,-1.4 C19.4,-.6 16.4,-.7 12.6,-.6 C9.4,-.5 6.4,.1 3.6,1.0 C5.8,-.2 9.4,-1.0 13.0,-1.1 C16.6,-1.2 19.2,-1.1 20.2,-1.4 Z',
      fill: INK
    }, g);
    // Small intense eye under heavy brow ridge
    el('path', {
      d: 'M7.7,-3.0 C8.6,-3.8 10.3,-3.8 11.2,-3.0 C10.3,-2.4 8.6,-2.4 7.7,-3.0 Z',
      fill: LEAD, stroke: INK, 'stroke-width': '0.38'
    }, g);
    el('circle', { cx: '9.65', cy: '-3.0', r: '0.58', fill: INK }, g);
    el('circle', { cx: '9.45', cy: '-3.14', r: '0.16', fill: LEAD }, g);
    el('path', {
      d: 'M7.2,-3.2 C8.4,-4.4 10.4,-4.4 11.7,-3.3 C10.5,-4.0 8.5,-4.0 7.2,-3.2 Z',
      fill: INK
    }, g);

    return g;
  }

  /* ------------------------------------------------------------------ *
   * Mini rigged legs & wings for crawlers and moonwalkers
   * ------------------------------------------------------------------ */
  function buildMiniLeg(register, wash, scale, phaseIdx, keyTimesStr, stepAngles, dur) {
    const isIllum = register === 'illuminated';
    const g = el('g', { transform: `scale(${f2(scale)})` });
    const upper = el('g', {}, g);
    if (keyTimesStr && stepAngles) {
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: stepAngles.hip, keyTimes: keyTimesStr,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, upper);
    }
    el('path', {
      d: 'M-2.2,-1.5 C-3.2,2.5 -2.8,6.5 -1.0,9.0 C1.2,9.8 3.2,8.0 3.6,4.5 C3.8,1.5 3.2,-1.2 1.8,-2.0 Z',
      fill: wash, 'fill-opacity': isIllum ? '0.92' : '0.45', stroke: INK, 'stroke-width': '0.72'
    }, upper);
    const knee = el('g', { transform: 'translate(0,8.2)' }, upper);
    const lower = el('g', {}, knee);
    if (keyTimesStr && stepAngles) {
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: stepAngles.knee, keyTimes: keyTimesStr,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lower);
    }
    el('path', {
      d: 'M-1.5,-1.0 L.6,6.8 L2.6,6.0 L1.2,-1.2 Z',
      fill: wash, 'fill-opacity': isIllum ? '0.92' : '0.45', stroke: INK, 'stroke-width': '0.68'
    }, lower);
    // Splayed clawed toes
    el('path', {
      d: 'M.4,6.4 L6.4,7.6 L1.4,8.2 M1.0,6.0 L7.6,5.8 L2.0,7.2 M.2,7.2 L4.8,9.8 L-.2,8.4',
      fill: INK, stroke: INK, 'stroke-width': '0.55', 'stroke-linejoin': 'round'
    }, lower);
    void phaseIdx;
    return g;
  }

  function buildMiniWing(register, wash, scale, keyTimesStr, flapAngles, dur) {
    const isIllum = register === 'illuminated';
    const g = el('g', { transform: `scale(${f2(scale)})` });
    const wing = el('g', {}, g);
    if (keyTimesStr && flapAngles) {
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: flapAngles, keyTimes: keyTimesStr,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, wing);
    }
    // Scalloped bat-membrane wing springing from shoulder (0,0), pointing up-back
    el('path', {
      d: 'M0,0 C-4,-8 -10,-15 -18,-17 C-15,-12 -14,-8 -16,-5 C-11,-7 -8,-5 -9,-1 C-5,-4 -2,-2 0,0 Z',
      fill: isIllum ? wash : wash, 'fill-opacity': isIllum ? '0.88' : '0.32',
      stroke: INK, 'stroke-width': '0.78', 'stroke-linejoin': 'round'
    }, wing);
    // Radiating ribs + hatching
    el('path', {
      d: 'M0,0 L-18,-17 M-5,-6 L-16,-5 M-4,-4 L-9,-1 M-8,-11 l2,4 M-11,-13 l2,5 M-14,-14 l1,4',
      fill: 'none', stroke: INK, 'stroke-width': '0.52', 'stroke-linecap': 'round'
    }, wing);
    if (isIllum) {
      el('path', {
        d: 'M-2,-2 L-16,-15',
        fill: 'none', stroke: 'url(#bestiary-gold-leaf)', 'stroke-width': '0.85', 'stroke-linecap': 'round'
      }, wing);
    }
    return g;
  }

  /* ------------------------------------------------------------------ *
   * Path measurement, smoothing, spatial lateral undulation, and
   * post-hole tight coil extension (strict Burrow Contract).
   * ------------------------------------------------------------------ */
  let _measureSvg = null;
  function measurePath(d, numSamples) {
    if (!_measureSvg && typeof document !== 'undefined' && document.body) {
      _measureSvg = el('svg', {
        width: '0', height: '0',
        style: 'position:absolute;width:0;height:0;overflow:hidden;pointer-events:none;visibility:hidden;'
      }, document.body);
    }
    const p = el('path', { d }, _measureSvg);
    const total = p.getTotalLength();
    const n = Math.max(16, numSamples || Math.ceil(total / 2.2));
    const pts = [];
    for (let i = 0; i <= n; i += 1) {
      const s = (i / n) * total;
      const pt = p.getPointAtLength(s);
      pts.push({ x: pt.x, y: pt.y, s });
    }
    if (p.parentNode) p.parentNode.removeChild(p);
    for (let i = 0; i < pts.length; i += 1) {
      const a = pts[Math.max(0, i - 1)];
      const b = pts[Math.min(pts.length - 1, i + 1)];
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const m = Math.hypot(dx, dy) || 1;
      pts[i].tx = dx / m;
      pts[i].ty = dy / m;
      pts[i].nx = -dy / m;
      pts[i].ny = dx / m;
    }
    return { total, pts };
  }

  function ptsToCubicD(pts, closed) {
    const n = pts.length;
    if (n < 2) return '';
    const pAt = closed
      ? (i) => pts[((i % n) + n) % n]
      : (i) => pts[clamp(i, 0, n - 1)];
    const segs = [`M${f1(pts[0].x)},${f1(pts[0].y)}`];
    const count = closed ? n : n - 1;
    for (let i = 0; i < count; i += 1) {
      const p0 = pAt(i - 1);
      const p1 = pAt(i);
      const p2 = pAt(i + 1);
      const p3 = pAt(i + 2);
      const c1x = p1.x + (p2.x - p0.x) / 6;
      const c1y = p1.y + (p2.y - p0.y) / 6;
      const c2x = p2.x - (p3.x - p1.x) / 6;
      const c2y = p2.y - (p3.y - p1.y) / 6;
      segs.push(`C${f1(c1x)},${f1(c1y)} ${f1(c2x)},${f1(c2y)} ${f1(p2.x)},${f1(p2.y)}`);
    }
    if (closed) segs.push('Z');
    return segs.join(' ');
  }

  function applySpatialUndulation(pts, totalLen, amp, wavelength) {
    if (!amp || amp < 0.05) return pts;
    return pts.map((pt) => {
      const u = totalLen > 0 ? pt.s / totalLen : 0;
      // Smoothly taper undulation to 0 at start & hole entry so entry tangent is exact
      const env = Math.sin(clamp(u / 0.12, 0, 1) * Math.PI * 0.5) *
                  Math.sin(clamp((1 - u) / 0.14, 0, 1) * Math.PI * 0.5);
      const off = amp * env * Math.sin((pt.s / wavelength) * Math.PI * 2);
      return { x: pt.x + pt.nx * off, y: pt.y + pt.ny * off, s: pt.s };
    });
  }

  /**
   * Build the extended burrow path + hole mask geometry.
   * Mask disc center is shifted forward along entry tangent by (R_mask - hole.r)
   * so its near rim meets the EXACT rim of {hole.x, hole.y, hole.r} (zero premature
   * clipping), while the post-hole continuation spirals strictly inside radius
   * (R_mask - maxHalfExtent - 2) for >= 1.6 * bodyLen so ZERO pixels ever escape
   * the black mask disc once the tail passes (hole.x, hole.y).
   */
  function buildBurrowGeometry(pathD, hole, bodyLen, maxHalfExtent, undAmp, undWave) {
    const raw = measurePath(pathD);
    let visPts = raw.pts;

    // Ensure the visible path ends at (hole.x, hole.y) when hole is given
    const last = visPts[visPts.length - 1];
    let tx = last.tx;
    let ty = last.ty;
    if (hole) {
      const distToHole = Math.hypot(hole.x - last.x, hole.y - last.y);
      if (distToHole > 0.5) {
        tx = (hole.x - last.x) / distToHole;
        ty = (hole.y - last.y) / distToHole;
        visPts = visPts.concat([{ x: hole.x, y: hole.y, s: raw.total + distToHole, tx, ty, nx: -ty, ny: tx }]);
      }
    }

    // Apply zero-skate spatial lateral undulation along the visible track
    const undPts = applySpatialUndulation(visPts, visPts[visPts.length - 1].s, undAmp, undWave);
    // Keep exact entry tangent at the final two points
    if (hole && undPts.length >= 2) {
      undPts[undPts.length - 1].x = hole.x;
      undPts[undPts.length - 1].y = hole.y;
    }

    // Downsample visible path smoothly (~every 4px)
    const cleanVis = [undPts[0]];
    for (let i = 1; i < undPts.length - 1; i += 1) {
      const prev = cleanVis[cleanVis.length - 1];
      if (Math.hypot(undPts[i].x - prev.x, undPts[i].y - prev.y) >= 4.0) {
        cleanVis.push(undPts[i]);
      }
    }
    cleanVis.push(undPts[undPts.length - 1]);

    const visPathD = ptsToCubicD(cleanVis, false);
    const visLen = measurePath(visPathD, 24).total;

    if (!hole) {
      return {
        fullD: visPathD,
        visLen,
        fullLen: visLen,
        maskDisc: null
      };
    }

    const hr = Math.max(1.2, hole.r || 3);
    const coilR = 1.1; // tight coil radius inside the mask disc
    const margin = 3.0;
    const rMask = Math.max(hr, maxHalfExtent + coilR + margin);
    const shift = Math.max(0, rMask - hr);
    const mx = hole.x + tx * shift;
    const my = hole.y + ty * shift;
    const nx = -ty;
    const ny = tx;

    // Build post-hole continuation: enter along (tx, ty) straight to (mx, my),
    // then spiral tightly inside radius coilR around (mx, my) for >= 1.65 * bodyLen!
    const needLen = Math.max(bodyLen * 1.65 + shift + 20, 120);
    const extPts = [];
    // Lead smoothly from hole to coil perimeter
    const leadSteps = Math.max(2, Math.ceil(shift / 4));
    for (let i = 1; i <= leadSteps; i += 1) {
      const frac = i / leadSteps;
      extPts.push({
        x: hole.x + tx * (shift * frac),
        y: hole.y + ty * (shift * frac)
      });
    }
    let accLen = shift;
    const dTheta = Math.PI / 4; // 8 points per revolution
    let theta = 0;
    while (accLen < needLen) {
      theta += dTheta;
      const px = mx + (Math.cos(theta) * tx + Math.sin(theta) * nx) * coilR;
      const py = my + (Math.cos(theta) * ty + Math.sin(theta) * ny) * coilR;
      const prev = extPts[extPts.length - 1];
      accLen += Math.hypot(px - prev.x, py - prev.y);
      extPts.push({ x: px, y: py });
    }

    const fullPts = cleanVis.concat(extPts);
    const fullD = ptsToCubicD(fullPts, false);
    const fullLen = measurePath(fullD, 64).total;

    return {
      fullD,
      visLen,
      fullLen,
      maskDisc: { cx: mx, cy: my, r: rMask, holeX: hole.x, holeY: hole.y, holeR: hr, shift }
    };
  }

  /* ------------------------------------------------------------------ *
   * Normalize storyboard [{t, u}] into synchronized SMIL keyframes.
   * u = 0: head at start; u = 1: tail past hole center (all masked).
   * ------------------------------------------------------------------ */
  function buildDefaultStoryboard(motion) {
    if (motion === 'concertina') {
      return [
        { t: 0.00, u: 0.00 },
        { t: 0.14, u: 0.16 },
        { t: 0.22, u: 0.16 }, // head pauses, looks & flicks
        { t: 0.40, u: 0.42 },
        { t: 0.48, u: 0.42 }, // pause & taste air
        { t: 0.66, u: 0.68 },
        { t: 0.72, u: 0.70 }, // slight bunch before burrow
        { t: 0.90, u: 1.00 }, // tail completely inside hole disc!
        { t: 1.00, u: 1.00 }  // clean dwell underground before reset
      ];
    }
    return [
      { t: 0.00, u: 0.00 },
      { t: 0.18, u: 0.22 },
      { t: 0.26, u: 0.22 }, // pause, look left & right, flick tongue
      { t: 0.48, u: 0.52 },
      { t: 0.55, u: 0.52 }, // second pause
      { t: 0.74, u: 0.76 },
      { t: 0.90, u: 1.00 }, // tail fully clears hole!
      { t: 1.00, u: 1.00 }  // clean hidden dwell
    ];
  }

  function normalizeStoryboard(sb, motion) {
    const raw = (sb && sb.length >= 2 ? sb : buildDefaultStoryboard(motion))
      .map((k) => ({ t: clamp(k.t, 0, 1), u: k.u }))
      .sort((a, b) => a.t - b.t);
    if (raw[0].t > 0) raw.unshift({ t: 0, u: raw[0].u });
    if (raw[raw.length - 1].t < 1) raw.push({ t: 1, u: raw[raw.length - 1].u });
    // Avoid duplicate keyTimes (Safari SMIL requires strictly non-decreasing, prefers strictly increasing)
    const out = [raw[0]];
    for (let i = 1; i < raw.length; i += 1) {
      const prev = out[out.length - 1];
      const t = raw[i].t <= prev.t ? Math.min(1, prev.t + 0.002) : raw[i].t;
      out.push({ t, u: raw[i].u });
    }
    out[out.length - 1].t = 1;
    return out;
  }

  function buildKeySplines(n) {
    const arr = [];
    for (let i = 0; i < n - 1; i += 1) arr.push('.42 0 .58 1');
    return arr.join(';');
  }

  /* ------------------------------------------------------------------ *
   * 1. makeCrawler(opts) -> SVGGElement
   * ------------------------------------------------------------------ */
  function makeCrawler(opts) {
    opts = opts || {};
    const spec = CREATURES[opts.creature] || CREATURES['draco-bipes'];
    const register = opts.register === 'illuminated' ? 'illuminated' : 'engraved';
    const isIllum = register === 'illuminated';
    const scale = opts.scale !== undefined ? opts.scale : 1;
    const wash = opts.wash || (isIllum ? spec.illum : spec.wash);
    const wash2 = opts.wash2 || (isIllum ? spec.illum2 : spec.wash2);
    const dur = opts.dur || 14;
    const motion = opts.motion && opts.motion !== 'auto' ? opts.motion : spec.motion;
    const hole = opts.hole || null;
    const headType = opts.headVariant === 'claude-ruffed' ? 'claude-ruffed' : (opts.headVariant || spec.head);

    const bodyLen = spec.bodyLen * scale;
    const maxWidth = spec.maxWidth * scale;
    const headScale = spec.headScale * scale;
    const ruffExtra = headType === 'claude-ruffed' ? 11 * headScale : 0;
    const wingExtra = spec.wings ? 14 * scale : 0;
    const legExtra = spec.legs ? 9 * scale : 0;
    const maxHalfExtent = Math.max(maxWidth * 0.72, 11 * headScale + ruffExtra, wingExtra, legExtra);

    const undAmp = motion === 'undulate' ? 2.6 * scale : (motion === 'gait' ? 1.1 * scale : 1.5 * scale);
    const undWave = 22 * scale;
    const geom = buildBurrowGeometry(opts.path, hole, bodyLen, maxHalfExtent, undAmp, undWave);
    const sb = normalizeStoryboard(opts.storyboard, motion);

    // Distance mapping:
    // u = 0 -> sHead = 0 (starts emerging cleanly from path start)
    // u = 1 -> sHead = visLen + bodyLen + holeEntryPad (so tail is cleanly past hole center & inside mask disc!)
    const holeEntryPad = hole ? (geom.maskDisc.shift + 6) : 0;
    const sMax = hole ? (geom.visLen + bodyLen + holeEntryPad) : geom.visLen;
    const safeFullLen = Math.max(geom.fullLen, sMax + 4);

    const rootG = el('g', {
      class: `bestiary-crawler bestiary-${register}`,
      'data-creature': opts.creature || 'draco-bipes',
      'data-register': register
    });
    const localDefs = el('defs', {}, rootG);

    const pathId = uid('crawler-path');
    el('path', { id: pathId, d: geom.fullD, fill: 'none' }, localDefs);

    // Explicit hole mask (Rule 2 & 4): white everywhere, black disc at hole (shifted forward so near rim = exact hole rim)
    let targetG = rootG;
    if (geom.maskDisc) {
      const maskId = uid('crawler-hole-mask');
      const mask = el('mask', {
        id: maskId,
        maskUnits: 'userSpaceOnUse',
        x: '-10000', y: '-10000', width: '20000', height: '20000'
      }, localDefs);
      el('rect', { x: '-10000', y: '-10000', width: '20000', height: '20000', fill: '#ffffff' }, mask);
      el('circle', {
        cx: f2(geom.maskDisc.cx),
        cy: f2(geom.maskDisc.cy),
        r: f2(geom.maskDisc.r),
        fill: '#000000'
      }, mask);
      targetG = el('g', { mask: `url(#${maskId})` }, rootG);
    }

    // Shared SMIL timing strings from the ONE unified timeline u(t)
    const keyTimesStr = sb.map((k) => f4(k.t)).join(';');
    const keySplinesStr = buildKeySplines(sb.length);
    const sHeadAt = (k) => clamp(k.u * sMax, 0, geom.fullLen);

    // Concertina stretch factor: rear segments bunch slightly when stationary and stretch when advancing
    function bandOffsetAt(kIdx, baseFrac) {
      const baseOff = baseFrac * bodyLen;
      if (motion !== 'concertina') return baseOff;
      const k = sb[kIdx];
      const prev = sb[Math.max(0, kIdx - 1)];
      const next = sb[Math.min(sb.length - 1, kIdx + 1)];
      const vel = (next.u - prev.u) / Math.max(0.01, next.t - prev.t);
      const bunch = vel < 0.25 ? 0.88 : 1.04;
      return baseOff * bunch;
    }

    // Helper: add a tapered dash-band along the path using positive wrap-free dashoffset
    const gap = Math.ceil(geom.fullLen + bodyLen + 400);
    function addDashBand(parent, frac0, frac1, strokeColor, strokeWidth, opacity, dashPattern, extraAttrs) {
      const segLen = Math.max(1.2, (frac1 - frac0) * bodyLen);
      const period = segLen + gap;
      const vals = sb.map((k, idx) => {
        const sHead = sHeadAt(k);
        const s0 = sHead - bandOffsetAt(idx, frac1);
        return f1(period - s0);
      }).join(';');
      const attrs = Object.assign({
        d: geom.fullD,
        fill: 'none',
        stroke: strokeColor,
        'stroke-width': f2(strokeWidth),
        'stroke-linecap': 'round',
        'stroke-linejoin': 'round',
        'stroke-dasharray': dashPattern || `${f1(segLen)} ${gap}`,
        'stroke-dashoffset': vals.split(';')[0],
        opacity: opacity !== undefined ? f2(opacity) : '1'
      }, extraAttrs || {});
      const p = el('path', attrs, parent);
      el('animate', {
        attributeName: 'stroke-dashoffset',
        values: vals,
        keyTimes: keyTimesStr,
        calcMode: 'spline',
        keySplines: keySplinesStr,
        dur: `${f1(dur)}s`,
        repeatCount: 'indefinite'
      }, p);
      return p;
    }

    // Helper: build layered engraved or illuminated body along the route
    function buildBodyRibbons(parent, colorMain, colorSec, phaseShiftFrac) {
      const bands = 10;
      const bodyG = el('g', { class: 'bestiary-body' }, parent);

      // Layer 1: Modulated ink contour silhouette (shadow heavy)
      for (let i = 0; i < bands; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(1, (i + 1) / bands + 0.025);
        const uMid = (f0 + f1b) * 0.5;
        const w = Math.max(0.8, spec.widthFn(uMid) * maxWidth);
        addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, INK, w + 1.55 * scale, 0.96);
      }

      // Layer 2: Base body tone (parchment cream under translucent wash, or rich pigment in illuminated)
      for (let i = 0; i < bands; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(1, (i + 1) / bands + 0.025);
        const uMid = (f0 + f1b) * 0.5;
        const w = Math.max(0.5, spec.widthFn(uMid) * maxWidth);
        if (isIllum) {
          addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, colorMain, w, 0.98);
        } else {
          // Pale parchment belly core + translucent watercolour wash over it
          addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, LEAD, w, 0.94);
          addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, colorMain, w * 0.94, 0.42);
        }
      }

      // Layer 3: Engraving chiaroscuro hatching & scale ticks OR illuminated gold & white-dot ornament
      for (let i = 0; i < bands - 1; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(0.95, (i + 1) / bands);
        const uMid = (f0 + f1b) * 0.5;
        const w = spec.widthFn(uMid) * maxWidth;
        if (w < 1.6 * scale) continue;
        if (isIllum) {
          // Burnished gold dorsal stripe + lead-white dot ornament (Aberdeen bestiary)
          addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, '#f6d155', w * 0.28, 0.85);
          // White highlight dots along the body
          const dotGap = Math.max(2.6, 3.8 * scale);
          const segLen = (f1b - f0) * bodyLen;
          const numDots = Math.max(1, Math.floor(segLen / dotGap));
          const pat = [];
          for (let d = 0; d < numDots; d += 1) pat.push(`0.4 ${f1(dotGap - 0.4)}`);
          const used = numDots * dotGap;
          pat.push(`0.01 ${f1(gap + Math.max(0, segLen - used))}`);
          // Use the same period so the white dots stay locked to the body!
          const dotPeriod = segLen + gap;
          const dotVals = sb.map((k, idx) => {
            const sHead = sHeadAt(k);
            const s0 = sHead - bandOffsetAt(idx, f1b + phaseShiftFrac);
            return f1(dotPeriod - s0);
          }).join(';');
          const dp = el('path', {
            d: geom.fullD, fill: 'none', stroke: LEAD,
            'stroke-width': f2(Math.max(0.7, w * 0.22)),
            'stroke-linecap': 'round',
            'stroke-dasharray': pat.join(' '),
            'stroke-dashoffset': dotVals.split(';')[0],
            opacity: '0.92'
          }, bodyG);
          el('animate', {
            attributeName: 'stroke-dashoffset',
            values: dotVals, keyTimes: keyTimesStr,
            calcMode: 'spline', keySplines: keySplinesStr,
            dur: `${f1(dur)}s`, repeatCount: 'indefinite'
          }, dp);
        } else {
          // Fine engraved transverse scale / rib hatching locked to the same timeline
          const tickGap = Math.max(2.0, 2.7 * scale);
          const segLen = (f1b - f0) * bodyLen;
          const numTicks = Math.max(1, Math.floor(segLen / tickGap));
          const pat = [];
          for (let t = 0; t < numTicks; t += 1) pat.push(`0.75 ${f1(tickGap - 0.75)}`);
          const used = numTicks * tickGap;
          pat.push(`0.01 ${f1(gap + Math.max(0, segLen - used))}`);
          const tickPeriod = segLen + gap;
          const tickVals = sb.map((k, idx) => {
            const sHead = sHeadAt(k);
            const s0 = sHead - bandOffsetAt(idx, f1b + phaseShiftFrac);
            return f1(tickPeriod - s0);
          }).join(';');
          const hp = el('path', {
            d: geom.fullD, fill: 'none', stroke: INK,
            'stroke-width': f2(w * 0.74),
            'stroke-linecap': 'butt',
            'stroke-dasharray': pat.join(' '),
            'stroke-dashoffset': tickVals.split(';')[0],
            opacity: '0.52'
          }, bodyG);
          el('animate', {
            attributeName: 'stroke-dashoffset',
            values: tickVals, keyTimes: keyTimesStr,
            calcMode: 'spline', keySplines: keySplinesStr,
            dur: `${f1(dur)}s`, repeatCount: 'indefinite'
          }, hp);
          // Second narrow dorsal shadow band for sculptural cylinder contrast
          addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, colorSec || INK, w * 0.34, 0.28);
        }
      }
    }

    // Helper: attach an appendage (head, leg, wing, rear head) to P_ext on the SAME timeline
    function attachAtBodyOffset(parent, offsetFromHead, childNode, flip180) {
      const holder = el('g', {}, parent);
      const kp = sb.map((k, idx) => {
        const sHead = sHeadAt(k);
        const s = clamp(sHead - bandOffsetAt(idx, offsetFromHead / bodyLen), 0, geom.fullLen);
        return f4(s / geom.fullLen);
      }).join(';');

      // Hide until the appendage actually enters s > 0 (prevents bunched legs/tail at s=0 before entry)
      if (offsetFromHead > 1) {
        const visVals = sb.map((k, idx) => (sHeadAt(k) - bandOffsetAt(idx, offsetFromHead / bodyLen) > 0.5 ? '1' : '0')).join(';');
        el('animate', {
          attributeName: 'opacity', values: visVals, keyTimes: keyTimesStr,
          calcMode: 'discrete', dur: `${f1(dur)}s`, repeatCount: 'indefinite'
        }, holder);
      }

      const am = el('animateMotion', {
        dur: `${f1(dur)}s`,
        repeatCount: 'indefinite',
        rotate: flip180 ? 'auto-reverse' : 'auto',
        calcMode: 'spline',
        keyTimes: keyTimesStr,
        keyPoints: kp,
        keySplines: keySplinesStr
      }, holder);
      el('mpath', { href: `#${pathId}` }, am);
      holder.appendChild(childNode);
      return holder;
    }

    // Build body (plus second trailing serpent if knotted-pair)
    buildBodyRibbons(targetG, wash, wash2, 0);
    if (spec.pair) {
      buildBodyRibbons(targetG, wash2, wash, 0.08);
    }

    // Build legs timed to moving intervals of the SAME storyboard (diagonal stepping gait!)
    if (spec.legs > 0) {
      const legFracs = spec.legs === 6 ? [0.24, 0.38, 0.52] : [0.32, 0.44];
      legFracs.forEach((lf, i) => {
        // Compute diagonal gait hip/knee values synced to storyboard motion vs pauses
        const hips = [];
        const knees = [];
        sb.forEach((k, idx) => {
          const prev = sb[Math.max(0, idx - 1)];
          const moving = idx > 0 && Math.abs(k.u - prev.u) > 0.015;
          if (!moving) {
            hips.push('0'); knees.push('-8');
          } else {
            const phase = (idx + i) % 2 === 0 ? 1 : -1;
            hips.push(String(phase * 18));
            knees.push(phase > 0 ? '-24' : '8');
          }
        });
        const legNode = buildMiniLeg(register, wash, 0.52 * scale, i, keyTimesStr, {
          hip: hips.join(';'), knee: knees.join(';')
        }, dur);
        attachAtBodyOffset(targetG, lf * bodyLen, legNode, false);
      });
    }

    // Build wings timed to the SAME storyboard
    if (spec.wings) {
      const flaps = sb.map((k, idx) => {
        const prev = sb[Math.max(0, idx - 1)];
        const moving = idx > 0 && Math.abs(k.u - prev.u) > 0.015;
        if (!moving) return '-4';
        return idx % 2 === 0 ? '-26' : '12';
      }).join(';');
      const wingNode = buildMiniWing(register, wash, 0.72 * scale, keyTimesStr, flaps, dur);
      attachAtBodyOffset(targetG, 0.30 * bodyLen, wingNode, false);
    }

    // Build front head (with look-turns + tongue flicks on the SAME dur)
    const headWrap = el('g', {});
    const lookG = el('g', {}, headWrap);
    if (opts.looks && opts.looks.length) {
      const ls = opts.looks.slice().sort((a, b) => a.t - b.t);
      if (ls[0].t > 0) ls.unshift({ t: 0, deg: 0 });
      if (ls[ls.length - 1].t < 1) ls.push({ t: 1, deg: 0 });
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: ls.map((l) => f1(l.deg)).join(';'),
        keyTimes: ls.map((l) => f4(l.t)).join(';'),
        calcMode: 'spline',
        keySplines: buildKeySplines(ls.length),
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lookG);
    } else {
      // Gentle inquisitive head turns during pauses
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;0;-14;12;0;0;-10;8;0;0',
        keyTimes: '0;.18;.21;.24;.27;.48;.51;.54;.57;1',
        calcMode: 'spline',
        keySplines: buildKeySplines(10),
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lookG);
    }
    const scaledHead = el('g', { transform: `scale(${f2(headScale)})` }, lookG);
    scaledHead.appendChild(buildHead(headType, {
      register, wash, dur,
      baseHead: spec.head,
      headVariant: opts.headVariant,
      flicks: opts.flicks || [{ t0: 0.20, t1: 0.25 }, { t0: 0.50, t1: 0.55 }]
    }));
    attachAtBodyOffset(targetG, 0, headWrap, false);

    // Amphisbaena rear head (facing backward along tail, taking turns; positioned so snout ends at bodyLen)
    if (spec.twoHeaded) {
      const rearWrap = el('g', { transform: `scale(${f2(headScale * 0.90)})` });
      rearWrap.appendChild(buildHead(headType, {
        register, wash: wash2 || wash, dur,
        baseHead: spec.head,
        headVariant: opts.headVariant,
        flicks: [{ t0: 0.34, t1: 0.40 }]
      }));
      attachAtBodyOffset(targetG, bodyLen * 0.85, rearWrap, true);
    }

    // Optional shed skin (synced to same timeline; fades/clears BEFORE tail entry so Rule 3 always holds)
    if (opts.shed) {
      const shedU = typeof opts.shed === 'object' ? (opts.shed.u || 0.35) : 0.35;
      const shedLen = bodyLen * 0.48;
      const shedS0 = clamp(shedU * geom.visLen - shedLen, 0, geom.visLen - shedLen);
      const shedPeriod = shedLen + gap;
      const shedP = el('path', {
        d: geom.fullD, fill: 'none', stroke: LEAD,
        'stroke-width': f2(maxWidth * 0.66),
        'stroke-linecap': 'round',
        'stroke-dasharray': `${f1(shedLen)} ${gap}`,
        'stroke-dashoffset': f1(shedPeriod - shedS0),
        opacity: '0'
      }, targetG);
      el('animate', {
        attributeName: 'opacity',
        values: '0;0;0.55;0.42;0;0',
        keyTimes: '0;.34;.40;.68;.82;1',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, shedP);
    }

    // Expose metadata for testing & debugging
    rootG._crawlerMeta = {
      visLen: geom.visLen,
      fullLen: safeFullLen,
      bodyLen,
      maxHalfExtent,
      maskDisc: geom.maskDisc,
      storyboard: sb,
      dur
    };
    return rootG;
  }

  /* ------------------------------------------------------------------ *
   * 2. drape(bubbleEl, opts) -> SVGSVGElement
   * Arbitrary rounded-rect border draping (top, loop, left, right,
   * bottom, frame) with smooth offset ribbon, tangent-aligned scales/
   * ornament, optional manuscript fleurons, and resting heraldic poise.
   * ------------------------------------------------------------------ */
  function buildRoundedRectPoints(x, y, w, h, r, side) {
    const cr = Math.min(r, w * 0.45, h * 0.45);
    const arcPts = (cx, cy, a0, a1, steps) => {
      const arr = [];
      for (let i = 0; i <= steps; i += 1) {
        const a = a0 + (a1 - a0) * (i / steps);
        arr.push({ x: cx + Math.cos(a) * cr, y: cy + Math.sin(a) * cr });
      }
      return arr;
    };
    const linePts = (x0, y0, x1, y1, steps) => {
      const arr = [];
      for (let i = 1; i < steps; i += 1) {
        const t = i / steps;
        arr.push({ x: x0 + (x1 - x0) * t, y: y0 + (y1 - y0) * t });
      }
      return arr;
    };

    const tl = arcPts(x + cr, y + cr, Math.PI, Math.PI * 1.5, 6);
    const tr = arcPts(x + w - cr, y + cr, Math.PI * 1.5, Math.PI * 2, 6);
    const br = arcPts(x + w - cr, y + h - cr, 0, Math.PI * 0.5, 6);
    const bl = arcPts(x + cr, y + h - cr, Math.PI * 0.5, Math.PI, 6);

    const topEdge = linePts(x + cr, y, x + w - cr, y, 8);
    const rightEdge = linePts(x + w, y + cr, x + w, y + h - cr, 5);
    const botEdge = linePts(x + w - cr, y + h, x + cr, y + h, 8);
    const leftEdge = linePts(x, y + h - cr, x, y + cr, 5);

    if (side === 'top') {
      return tl.slice(2).concat(topEdge, tr.slice(0, 5));
    }
    if (side === 'bottom') {
      return br.slice(2).concat(botEdge, bl.slice(0, 5));
    }
    if (side === 'left') {
      return bl.slice(2).concat(leftEdge, tl.slice(0, 5));
    }
    if (side === 'right') {
      return tr.slice(2).concat(rightEdge, br.slice(0, 5));
    }
    // Full loop starting top-left clockwise
    return tl.concat(topEdge, tr, rightEdge, br, botEdge, bl, leftEdge);
  }

  function buildRibbonFromSamples(pts, widthFn, maxW) {
    const n = pts.length;
    const left = [];
    const right = [];
    const tangents = [];
    let total = 0;
    const acc = [0];
    for (let i = 1; i < n; i += 1) {
      total += Math.hypot(pts[i].x - pts[i - 1].x, pts[i].y - pts[i - 1].y);
      acc.push(total);
    }
    for (let i = 0; i < n; i += 1) {
      const a = pts[Math.max(0, i - 1)];
      const b = pts[Math.min(n - 1, i + 1)];
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const m = Math.hypot(dx, dy) || 1;
      const tx = dx / m;
      const ty = dy / m;
      const nx = -ty;
      const ny = tx;
      const u = total > 0 ? acc[i] / total : 0;
      const hw = 0.5 * widthFn(u) * maxW;
      left.push({ x: pts[i].x + nx * hw, y: pts[i].y + ny * hw });
      right.push({ x: pts[i].x - nx * hw, y: pts[i].y - ny * hw });
      tangents.push({ x: pts[i].x, y: pts[i].y, tx, ty, nx, ny, u, s: acc[i], hw });
    }
    const silPts = left.concat(right.slice().reverse());
    return {
      silD: ptsToCubicD(silPts, true),
      leftD: ptsToCubicD(left, false),
      rightD: ptsToCubicD(right, false),
      tangents,
      total
    };
  }

  function buildMiniFleuron(x, y, deg, scale) {
    const g = el('g', { transform: `translate(${f1(x)},${f1(y)}) rotate(${f1(deg)}) scale(${f2(scale)})` });
    // Manuscript green acanthus/ivy leaf with blue bud & gold/white highlight
    el('path', {
      d: 'M-8,0 C-5,-4 0,-5 4,-2 C6,-4 9,-3 10,0 C9,3 6,4 4,2 C0,5 -5,4 -8,0 Z',
      fill: '#2f7d4a', stroke: INK, 'stroke-width': '0.65', 'stroke-linejoin': 'round'
    }, g);
    el('path', {
      d: 'M-5,0 C-1,-2 3,-1 7,0',
      fill: 'none', stroke: LEAD, 'stroke-width': '0.45', opacity: '0.85', 'stroke-linecap': 'round'
    }, g);
    el('circle', { cx: '10.6', cy: '0', r: '1.35', fill: '#1d3f9e', stroke: INK, 'stroke-width': '0.45' }, g);
    return g;
  }

  function drape(bubbleEl, opts) {
    opts = opts || {};
    const spec = CREATURES[opts.creature] || CREATURES.aspis;
    const register = opts.register === 'illuminated' ? 'illuminated' : 'engraved';
    const isIllum = register === 'illuminated';
    const side = opts.side || 'top';
    const scale = opts.scale !== undefined ? opts.scale : 1;
    const wash = opts.wash || (isIllum ? spec.illum : spec.wash);
    const wash2 = opts.wash2 || (isIllum ? spec.illum2 : spec.wash2);
    const headType = opts.headVariant === 'claude-ruffed' ? 'claude-ruffed' : (opts.headVariant || spec.head);

    // Measure box (supports DOM element OR plain {width, height, radius} descriptor)
    let w, h, r;
    const isDom = bubbleEl && typeof bubbleEl.getBoundingClientRect === 'function';
    if (isDom) {
      const rect = bubbleEl.getBoundingClientRect();
      const cs = window.getComputedStyle ? window.getComputedStyle(bubbleEl) : null;
      w = rect.width || bubbleEl.offsetWidth || 260;
      h = rect.height || bubbleEl.offsetHeight || 72;
      r = cs ? (parseFloat(cs.borderTopLeftRadius) || 14) : 14;
    } else {
      w = (bubbleEl && bubbleEl.width) || opts.width || 260;
      h = (bubbleEl && bubbleEl.height) || opts.height || 72;
      r = (bubbleEl && bubbleEl.radius) || opts.radius || 14;
    }

    const pad = Math.round(18 * scale);
    const svgW = Math.ceil(w + pad * 2);
    const svgH = Math.ceil(h + pad * 2);
    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: `bestiary-drape bestiary-drape-${side}`,
      style: `position:absolute;left:${-pad}px;top:${-pad}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    ensureSharedDefs(svg);

    function drawSingleVine(pts, colorMain, colorSec, reverseHead) {
      const oriented = reverseHead ? pts.slice().reverse() : pts;
      const maxW = spec.maxWidth * 1.15 * scale;
      const rib = buildRibbonFromSamples(oriented, spec.widthFn, maxW);
      const vineG = el('g', { class: 'bestiary-drape-vine' }, svg);

      // Gentle breathing sway (resting heraldic poise)
      const mid = oriented[Math.floor(oriented.length / 2)];
      const breath = el('g', { transform: `translate(${f1(mid.x)},${f1(mid.y)})` }, vineG);
      const inner = el('g', {}, breath);
      el('animateTransform', {
        attributeName: 'transform', type: 'scale',
        values: '1 1;1.012 1.018;1 1', keyTimes: '0;.5;1',
        calcMode: 'spline', keySplines: '.42 0 .58 1;.42 0 .58 1',
        dur: '5.3s', repeatCount: 'indefinite'
      }, inner);
      const bodyLayer = el('g', { transform: `translate(${f1(-mid.x)},${f1(-mid.y)})` }, inner);

      // Body fill (engraved wash or illuminated pigment)
      if (isIllum) {
        el('path', {
          d: rib.silD, fill: colorMain, 'fill-opacity': '0.96',
          filter: opts.texture === false ? null : 'url(#bestiary-mottle)'
        }, bodyLayer);
        // Burnished gold dorsal vein + white highlight
        const midD = ptsToCubicD(oriented, false);
        el('path', { d: midD, fill: 'none', stroke: 'url(#bestiary-gold-leaf)', 'stroke-width': f2(maxW * 0.24), opacity: '0.9' }, bodyLayer);
      } else {
        el('path', { d: rib.silD, fill: LEAD, 'fill-opacity': '0.92' }, bodyLayer);
        el('path', { d: rib.silD, fill: colorMain, 'fill-opacity': '0.40' }, bodyLayer);
      }

      // Tangent-aligned scales / ventral hatching / white-dot ornament
      const marks = [];
      const dots = [];
      let lastS = -999;
      for (const p of rib.tangents) {
        const sp = Math.max(3.6, 5.2 * scale);
        if (p.s - lastS >= sp && p.hw > 1.2 * scale) {
          lastS = p.s;
          const ax = p.x + p.nx * (p.hw * 0.92);
          const ay = p.y + p.ny * (p.hw * 0.92);
          const bx = p.x - p.nx * (p.hw * 0.92);
          const by = p.y - p.ny * (p.hw * 0.92);
          const qx = p.x + p.tx * (p.hw * 0.32);
          const qy = p.y + p.ty * (p.hw * 0.32);
          marks.push(`M${f1(ax)},${f1(ay)} Q${f1(qx)},${f1(qy)} ${f1(bx)},${f1(by)}`);
          if (isIllum) {
            dots.push({ x: p.x + p.nx * (p.hw * 0.35), y: p.y + p.ny * (p.hw * 0.35), r: Math.max(0.55, p.hw * 0.18) });
          }
        }
      }
      el('path', {
        d: marks.join(' '), fill: 'none',
        stroke: isIllum ? (colorSec || INK) : INK,
        'stroke-width': isIllum ? '0.58' : '0.66',
        opacity: isIllum ? '0.62' : '0.78',
        'stroke-linecap': 'round'
      }, bodyLayer);
      for (const d of dots) {
        el('circle', { cx: f1(d.x), cy: f1(d.y), r: f2(d.r), fill: LEAD, opacity: '0.92' }, bodyLayer);
      }

      // Modulated contours (thick shadow underside, delicate lit ridge)
      el('path', { d: rib.leftD, fill: 'none', stroke: INK, 'stroke-width': f2(0.75 * scale), 'stroke-linecap': 'round' }, bodyLayer);
      el('path', { d: rib.rightD, fill: 'none', stroke: INK, 'stroke-width': f2(1.45 * scale), 'stroke-linecap': 'round' }, bodyLayer);

      // Rigged head at the front of the vine (tangent-aligned, looking & flicking gently)
      const h0 = rib.tangents[0];
      const hDeg = (Math.atan2(-h0.ty, -h0.tx) * 180) / Math.PI;
      const headHolder = el('g', { transform: `translate(${f1(h0.x)},${f1(h0.y)}) rotate(${f1(hDeg)})` }, bodyLayer);
      const sway = el('g', {}, headHolder);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;-7;6;0', keyTimes: '0;.34;.72;1',
        calcMode: 'spline', keySplines: '.4 0 .6 1;.4 0 .6 1;.4 0 .6 1',
        dur: '7.1s', repeatCount: 'indefinite'
      }, sway);
      const hScaled = el('g', { transform: `scale(${f2(spec.headScale * 1.95 * scale)})` }, sway);
      hScaled.appendChild(buildHead(headType, {
        register, wash: colorMain, dur: 6.7,
        baseHead: spec.head, headVariant: opts.headVariant
      }));

      // Optional green/blue manuscript fleurons at tail & midpoint
      if (opts.fleurons) {
        const tailP = rib.tangents[rib.tangents.length - 1];
        const tDeg = (Math.atan2(tailP.ty, tailP.tx) * 180) / Math.PI;
        bodyLayer.appendChild(buildMiniFleuron(tailP.x, tailP.y, tDeg, 0.95 * scale));
      }
    }

    if (side === 'frame') {
      // Heraldic dual-serpent frame leaving the bottom caption band cleanly readable!
      // Both serpents start with heads confronting near top-centre and trail smoothly down each side to the bottom corners.
      const topPts = buildRoundedRectPoints(pad, pad, w, h, r, 'top');
      const midTop = Math.floor(topPts.length / 2);
      const leftEdgeUp = buildRoundedRectPoints(pad, pad, w, h, r, 'left');      // bottom-left -> top-left
      const rightEdgeDown = buildRoundedRectPoints(pad, pad, w, h, r, 'right');  // top-right -> bottom-right

      // Left serpent: head at top-centre-left -> top-left -> down left edge -> tail at bottom-left
      const leftVine = topPts.slice(0, midTop - 1).reverse().concat(leftEdgeUp.slice().reverse());
      // Right serpent: head at top-centre-right -> top-right -> down right edge -> tail at bottom-right
      const rightVine = topPts.slice(midTop + 1).concat(rightEdgeDown);

      drawSingleVine(leftVine, wash, wash2, false);
      drawSingleVine(rightVine, wash2 || wash, wash, false);
    } else {
      const pts = buildRoundedRectPoints(pad, pad, w, h, r, side);
      drawSingleVine(pts, wash, wash2, false);
    }

    if (isDom && opts.attach !== false) {
      if (window.getComputedStyle && window.getComputedStyle(bubbleEl).position === 'static') {
        bubbleEl.style.position = 'relative';
      }
      bubbleEl.appendChild(svg);
    }
    return svg;
  }

  /* ------------------------------------------------------------------ *
   * 3. makeMoonwalker(bubbleEl, opts) -> SVGSVGElement
   * Walks a full circuit round any bubble's rounded-rect border:
   * - planted foot slides backward against direction of travel while
   *   opposite heel lifts & rolls forward (true moonwalk illusion);
   * - subtle step bob;
   * - corner head-turns & brief pause at top-right corner;
   * - stays attached to the exact border on any bubble size.
   * ------------------------------------------------------------------ */
  function makeMoonwalker(bubbleEl, opts) {
    opts = opts || {};
    const spec = CREATURES[opts.creature] || CREATURES['draco-bipes'];
    const register = opts.register === 'illuminated' ? 'illuminated' : 'engraved';
    const isIllum = register === 'illuminated';
    const scale = opts.scale !== undefined ? opts.scale : 1;
    const wash = opts.wash || (isIllum ? spec.illum : spec.wash);
    const dur = opts.dur || 16;
    const headType = opts.headVariant || spec.head;

    let w, h, r;
    const isDom = bubbleEl && typeof bubbleEl.getBoundingClientRect === 'function';
    if (isDom) {
      const rect = bubbleEl.getBoundingClientRect();
      const cs = window.getComputedStyle ? window.getComputedStyle(bubbleEl) : null;
      w = rect.width || bubbleEl.offsetWidth || 240;
      h = rect.height || bubbleEl.offsetHeight || 68;
      r = cs ? (parseFloat(cs.borderTopLeftRadius) || 14) : 14;
    } else {
      w = (bubbleEl && bubbleEl.width) || opts.width || 240;
      h = (bubbleEl && bubbleEl.height) || opts.height || 68;
      r = (bubbleEl && bubbleEl.radius) || opts.radius || 14;
    }

    const pad = Math.round(28 * scale);
    const svgW = Math.ceil(w + pad * 2);
    const svgH = Math.ceil(h + pad * 2);
    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-moonwalker',
      style: `position:absolute;left:${-pad}px;top:${-pad}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    ensureSharedDefs(svg);

    // Closed clockwise rounded-rect perimeter hugging the exact border
    const cr = Math.min(r, w * 0.45, h * 0.45);
    const x0 = pad, y0 = pad, x1 = pad + w, y1 = pad + h;
    const loopD = [
      `M${f1(x0 + cr)},${f1(y0)}`,
      `L${f1(x1 - cr)},${f1(y0)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x1)},${f1(y0 + cr)}`,
      `L${f1(x1)},${f1(y1 - cr)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x1 - cr)},${f1(y1)}`,
      `L${f1(x0 + cr)},${f1(y1)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x0)},${f1(y1 - cr)}`,
      `L${f1(x0)},${f1(y0 + cr)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x0 + cr)},${f1(y0)} Z`
    ].join(' ');

    const defs = el('defs', {}, svg);
    const pid = uid('moonwalk-loop');
    el('path', { id: pid, d: loopD, fill: 'none' }, defs);

    // Exact corner fractions along the perimeter so pause & head-turns land right on corners!
    const topL = Math.max(1, w - 2 * cr);
    const sideL = Math.max(1, h - 2 * cr);
    const arcL = Math.PI * cr * 0.5;
    const perim = 2 * topL + 2 * sideL + 4 * arcL;
    const pTR = (topL + arcL * 0.5) / perim;
    const pBR = (topL + arcL + sideL + arcL * 0.5) / perim;
    const pBL = (2 * topL + 2 * arcL + sideL + arcL * 0.5) / perim;

    // Keyframe progress with a brief curious pause at top-right corner!
    const kt = [0, 0.24, 0.34, 0.52, 0.78, 1.0];
    const kp = [0, pTR, pTR, pBR, pBL, 1.0];

    const walker = el('g', { class: 'bestiary-moonwalker-actor' }, svg);
    const am = el('animateMotion', {
      dur: `${f1(dur)}s`, repeatCount: 'indefinite', rotate: 'auto',
      calcMode: 'spline',
      keyTimes: kt.map(f2).join(';'),
      keyPoints: kp.map(f2).join(';'),
      keySplines: '.4 0 .6 1;0 0 1 1;.4 0 .6 1;.4 0 .6 1;.4 0 .6 1'
    }, walker);
    el('mpath', { href: `#${pid}` }, am);

    // Local coordinates: border surface is y = 0, creature stands above border (y < 0),
    // facing -x (moonwalking backward while gliding +x along the border!).
    const stepDur = '0.95s';
    const standG = el('g', { transform: `scale(${f2(scale)})` }, walker);

    // Subtle body bob synchronized to every moonwalk foot-roll
    const bobG = el('g', {}, standG);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: '0 0;0 -1.6;0 0;0 -1.6;0 0',
      keyTimes: '0;.25;.50;.75;1',
      calcMode: 'spline', keySplines: '.42 0 .58 1;.42 0 .58 1;.42 0 .58 1;.42 0 .58 1',
      dur: stepDur, repeatCount: 'indefinite'
    }, bobG);

    // Moonwalk legs:
    // Planted foot slides smoothly from x=-3 to x=+5 along y=0 (against direction of travel +x),
    // while opposite foot pops up onto its toe-tip, rolls, and plants at x=-3!
    function buildMoonLeg(isBack) {
      const lg = el('g', { transform: isBack ? 'translate(-2,-11)' : 'translate(2,-11)' });
      const slide = el('g', {}, lg);
      el('animateTransform', {
        attributeName: 'transform', type: 'translate',
        values: isBack ? '-4 0;4 0;1 -1.5;-4 0' : '4 0;1 -1.5;-4 0;4 0',
        keyTimes: '0;.50;.76;1',
        calcMode: 'linear', dur: stepDur, repeatCount: 'indefinite'
      }, slide);
      // Upper thigh
      const hip = el('g', {}, slide);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: isBack ? '20;-18;-6;20' : '-18;-6;20;-18',
        keyTimes: '0;.50;.76;1',
        dur: stepDur, repeatCount: 'indefinite'
      }, hip);
      el('path', {
        d: 'M-1.8,-1 C-2.4,2.2 -1.8,5.5 -.4,7.4 C1.2,7.8 2.4,6.4 2.6,3.6 C2.6,1.2 1.8,-1 0,-1.4 Z',
        fill: wash, 'fill-opacity': isIllum ? '0.94' : '0.48', stroke: INK, 'stroke-width': '0.68'
      }, hip);
      // Lower shin + rolling moonwalk ankle/claw touching y = 0
      const ankle = el('g', { transform: 'translate(0,6.8)' }, hip);
      const foot = el('g', {}, ankle);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: isBack ? '-18;16;28;-18' : '16;28;-18;16',
        keyTimes: '0;.50;.76;1',
        dur: stepDur, repeatCount: 'indefinite'
      }, foot);
      el('path', {
        d: 'M-1.2,-.8 L.4,4.2 L2.0,3.8 L1.0,-.8 Z',
        fill: wash, 'fill-opacity': isIllum ? '0.94' : '0.48', stroke: INK, 'stroke-width': '0.62'
      }, foot);
      el('path', {
        d: 'M-4.6,4.2 L2.4,4.2 L-1.0,2.8 Z',
        fill: INK
      }, foot);
      return lg;
    }

    bobG.appendChild(buildMoonLeg(true));

    // Compact coiled engraved/illuminated dragon torso + sweeping tail (facing -x so it moonwalks!)
    const torso = el('g', {}, bobG);
    el('path', {
      d: 'M-10,-15 C-6,-19 4,-19 10,-14 C15,-10 20,-15 24,-21 C27,-25 22,-27 19,-22 C16,-17 14,-11 8,-9 C1,-7 -6,-8 -11,-11 Z',
      fill: isIllum ? wash : LEAD, 'fill-opacity': '0.95', stroke: INK, 'stroke-width': '0.95', 'stroke-linejoin': 'round'
    }, torso);
    if (!isIllum) {
      el('path', {
        d: 'M-10,-15 C-6,-19 4,-19 10,-14 C15,-10 20,-15 24,-21 C22,-18 16,-12 8,-10 C2,-9 -5,-10 -10,-13 Z',
        fill: wash, 'fill-opacity': '0.46'
      }, torso);
    }
    // Engraved dorsal scales / rib hatching
    el('path', {
      d: 'M-6,-15 l2,4 M-2,-16 l2,5 M2,-16 l2,5 M6,-14 l1.5,4 M11,-12 l1,3 M16,-14 l2,2',
      fill: 'none', stroke: INK, 'stroke-width': '0.58', 'stroke-linecap': 'round'
    }, torso);

    // Front moonwalk leg
    bobG.appendChild(buildMoonLeg(false));

    // Proud S-neck & corner-turning head (facing -x, turning curiously at corners & pause)
    const neckG = el('g', { transform: 'translate(-9,-13)' }, bobG);
    el('path', {
      d: 'M0,0 C-5,-2 -7,-7 -4,-12 C-2,-15 0,-17 -3,-19',
      fill: 'none', stroke: INK, 'stroke-width': '4.6', 'stroke-linecap': 'round'
    }, neckG);
    el('path', {
      d: 'M0,0 C-5,-2 -7,-7 -4,-12 C-2,-15 0,-17 -3,-19',
      fill: 'none', stroke: isIllum ? wash : LEAD, 'stroke-width': '3.0', 'stroke-linecap': 'round'
    }, neckG);
    if (!isIllum) {
      el('path', {
        d: 'M0,0 C-5,-2 -7,-7 -4,-12 C-2,-15 0,-17 -3,-19',
        fill: 'none', stroke: wash, 'stroke-width': '2.6', 'stroke-opacity': '0.46', 'stroke-linecap': 'round'
      }, neckG);
    }

    const headPiv = el('g', { transform: 'translate(-3,-19) scale(-0.56,0.56)' }, neckG);
    const headTurn = el('g', {}, headPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;-16;14;0;-14;0;-12;0',
      keyTimes: '0;.23;.29;.35;.52;.58;.78;1',
      calcMode: 'spline', keySplines: buildKeySplines(8),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, headTurn);
    headTurn.appendChild(buildHead(headType, {
      register, wash, dur,
      baseHead: spec.head, headVariant: opts.headVariant,
      flicks: [{ t0: 0.26, t1: 0.32 }]
    }));

    if (isDom && opts.attach !== false) {
      if (window.getComputedStyle && window.getComputedStyle(bubbleEl).position === 'static') {
        bubbleEl.style.position = 'relative';
      }
      bubbleEl.appendChild(svg);
    }
    return svg;
  }

  return {
    CREATURES,
    REGISTERS,
    ensureSharedDefs,
    buildHead,
    makeCrawler,
    drape,
    makeMoonwalker
  };
}));
