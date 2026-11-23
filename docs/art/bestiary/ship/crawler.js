/**
 * Bestiary Crawler & Travelling Rig Library (Brief 3 — Textured Illuminated Manuscript & Woodcut)
 * Zero-dependency SMIL SVG motion engine combining rich historical mineral tempera
 * (multi-band gouache modelling, fine parallel brushwork, lead-white seam dot rows)
 * with monochrome woodcut linework (after 1-coiled-dragon-roundel.jpg,
 * 2-winged-dragon-initial-interlace.jpg, 3-dragon-border-drollery.jpg, and DE DRACONE):
 * - Crisp, decisively stroked iron-gall outlines (#1c100a, ~1.2-1.8px — never crude chunky cartoon lines)
 * - Rich mineral pigments (cinnabar, ultramarine, Romanesque wine-violet, verdigris-sage,
 *   smoky olive-charcoal, burnished ochre-gold, aged parchment ivory — zero neon primaries, zero pink)
 * - Six unified single-timeline travelling rigs + IntersectionObserver on-screen-only pausing
 * - Preserved 0-stray-pixel shifted circular <mask> burrow contract into "o" and "i"
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

  /* ------------------------------------------------------------------ *
   * Neon, Saturated Bestiary Palette (zero pink, recolourable via
   * --wash and --wash2 CSS custom properties)
   * ------------------------------------------------------------------ */
  const PALETTE = {
    ink: '#140b06',          // bold continuous dark outline
    inkSoft: '#2b1a10',      // rib band accent
    ivory: '#fff6dd',        // bright parchment highlight
    lead: '#fffdf5',         // crisp lead-white dot rail & eye sclera
    belly: '#fff0c2',        // lighter belly band
    cinnabar: '#ff4518',     // neon flame vermilion-orange
    cinnabarDeep: '#b31e00', // deep flame shadow
    vermilion: '#ff2e12',    // electric vermilion
    lapis: '#1b5cff',        // electric cobalt / ultramarine
    lapisLight: '#70a4ff',   // bright cobalt highlight
    lapisDeep: '#0a2999',    // deep cobalt shadow
    wine: '#5b2bff',         // neon electric indigo-violet (zero pink)
    wineLight: '#977aff',    // bright indigo highlight
    wineDeep: '#2c0da3',     // deep indigo shadow
    sage: '#00d97e',         // neon emerald-verdigris
    sageLight: '#7affc2',    // bright emerald highlight
    sageDeep: '#007a44',     // deep emerald shadow
    charcoal: '#00c8d7',     // neon cyan-teal
    charcoalLight: '#7ef2fa',// bright cyan highlight
    charcoalDeep: '#006f7a', // deep teal shadow
    gold: '#ffb800',         // neon saturated gold
    goldLight: '#ffe169',    // bright gold highlight
    goldDeep: '#b87200',     // deep gold shadow
    claude: '#ff5a1f',       // neon Claude orange
    claudeDeep: '#b82d00',
    blue3b1b: '#1b5cff',     // electric ultramarine
    chestnut: '#ff7a00',     // neon amber-orange
    fleuronGreen: '#00c96e', // neon verdigris fleuron
    verdigris: '#00d97e',
    orpiment: '#ffb800'
  };

  const INK = PALETTE.ink;
  const LEAD = PALETTE.lead;
  const IVORY = PALETTE.ivory;
  const BELLY = PALETTE.belly;

  let uidCounter = 0;
  function uid(prefix) {
    uidCounter += 1;
    return `${prefix}-${uidCounter}`;
  }

  function applyWashVars(node, wash, wash2) {
    if (!node) return;
    if (wash) node.style.setProperty('--wash', wash);
    if (wash2) node.style.setProperty('--wash2', wash2);
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

  function buildKeySplines(n, spline) {
    const sp = spline || '.42 0 .58 1';
    const arr = [];
    for (let i = 0; i < Math.max(1, n - 1); i += 1) arr.push(sp);
    return arr.join(';');
  }

  /* ------------------------------------------------------------------ *
   * Species registry — mix of rich textured mineral pigments &
   * monochrome woodcut (DE DRACONE), refined medieval proportions.
   * ------------------------------------------------------------------ */
  const CREATURES = {
    'draco-bipes': {
      label: 'Draco bipes apteros',
      attitude: 'imperious',
      head: 'wyvern',
      legs: 2,
      wings: false,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 68,
      maxWidth: 9.4,
      headScale: 0.35,
      motion: 'gait',
      wash: PALETTE.charcoal,
      headWash: PALETTE.cinnabar,
      shade: PALETTE.charcoalDeep,
      highlight: PALETTE.charcoalLight,
      wash2: PALETTE.gold,
      illum: PALETTE.charcoal,
      illum2: PALETTE.cinnabar,
      widthFn(u) {
        if (u < 0.22) return 0.50 + 0.18 * (0.5 - 0.5 * Math.cos((u / 0.22) * Math.PI));
        if (u < 0.44) return 0.68 + 0.32 * (0.5 - 0.5 * Math.cos(((u - 0.22) / 0.22) * Math.PI));
        if (u < 0.66) return 1.00 - 0.48 * (0.5 - 0.5 * Math.cos(((u - 0.44) / 0.22) * Math.PI));
        return 0.52 * Math.pow(Math.max(0, 1 - (u - 0.66) / 0.34), 1.2) + 0.08;
      }
    },
    'draco-alatus': {
      label: 'Draco alatus (Wyvern)',
      attitude: 'regal',
      head: 'wyvern',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 70,
      maxWidth: 9.2,
      headScale: 0.35,
      motion: 'gait',
      wash: PALETTE.lapis,
      headWash: PALETTE.sage,
      shade: PALETTE.lapisDeep,
      highlight: PALETTE.lapisLight,
      wash2: PALETTE.wine,
      illum: PALETTE.lapis,
      illum2: PALETTE.wine,
      widthFn(u) {
        if (u < 0.24) return 0.48 + 0.22 * (0.5 - 0.5 * Math.cos((u / 0.24) * Math.PI));
        if (u < 0.46) return 0.70 + 0.30 * (0.5 - 0.5 * Math.cos(((u - 0.24) / 0.22) * Math.PI));
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.46) / 0.54), 1.15) + 0.08;
      }
    },
    'aspis': {
      label: 'Aspis (Roundel Serpent)',
      attitude: 'wary',
      head: 'serpent',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 80,
      maxWidth: 8.6,
      headScale: 0.34,
      motion: 'undulate',
      wash: PALETTE.wine,
      headWash: PALETTE.ivory,
      shade: PALETTE.wineDeep,
      highlight: PALETTE.wineLight,
      wash2: PALETTE.cinnabar,
      illum: PALETTE.wine,
      illum2: PALETTE.cinnabar,
      widthFn(u) {
        if (u < 0.18) return 0.56 + 0.26 * (u / 0.18);
        if (u < 0.40) return 0.82 + 0.18 * Math.sin(((u - 0.18) / 0.22) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.40) / 0.60), 1.15) + 0.08;
      }
    },
    'spotted-serpent': {
      label: 'Draco Woodcut (De Dracone)',
      attitude: 'watchful',
      head: 'woodcut',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      woodcut: true,
      bodyLen: 72,
      maxWidth: 9.2,
      headScale: 0.35,
      motion: 'gait',
      wash: PALETTE.ivory,
      headWash: PALETTE.ivory,
      shade: PALETTE.charcoal,
      highlight: PALETTE.lead,
      wash2: PALETTE.gold,
      illum: PALETTE.sage,
      illum2: PALETTE.gold,
      widthFn(u) {
        if (u < 0.20) return 0.52 + 0.24 * (u / 0.20);
        if (u < 0.42) return 0.76 + 0.24 * Math.sin(((u - 0.20) / 0.22) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.42) / 0.58), 1.18) + 0.08;
      }
    },
    'basilisk': {
      label: 'Basiliscus coronatus',
      attitude: 'watchful',
      head: 'basilisk',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 72,
      maxWidth: 9.2,
      headScale: 0.35,
      motion: 'gait',
      wash: PALETTE.sage,
      headWash: PALETTE.cinnabar,
      shade: PALETTE.sageDeep,
      highlight: PALETTE.sageLight,
      wash2: PALETTE.gold,
      illum: PALETTE.sage,
      illum2: PALETTE.cinnabar,
      widthFn(u) {
        if (u < 0.20) return 0.52 + 0.24 * (u / 0.20);
        if (u < 0.42) return 0.76 + 0.24 * Math.sin(((u - 0.20) / 0.22) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.42) / 0.58), 1.18) + 0.08;
      }
    },
    'basilisk-6': {
      label: 'Basiliscus hexapus',
      attitude: 'alert',
      head: 'basilisk',
      legs: 6,
      wings: false,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 72,
      maxWidth: 9.4,
      headScale: 0.35,
      motion: 'gait',
      wash: PALETTE.chestnut,
      headWash: PALETTE.cinnabar,
      shade: PALETTE.cinnabarDeep,
      highlight: PALETTE.goldLight,
      wash2: PALETTE.gold,
      illum: PALETTE.cinnabar,
      illum2: PALETTE.gold,
      widthFn(u) {
        if (u < 0.20) return 0.50 + 0.24 * (u / 0.20);
        if (u < 0.46) return 0.74 + 0.26 * Math.sin(((u - 0.20) / 0.26) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.46) / 0.54), 1.15) + 0.08;
      }
    },
    'knotted-pair': {
      label: 'Serpentes Nexi',
      attitude: 'wry',
      head: 'wyvern',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: true,
      woodcut: false,
      bodyLen: 78,
      maxWidth: 8.4,
      headScale: 0.34,
      motion: 'undulate',
      wash: PALETTE.lapis,
      headWash: PALETTE.sage,
      shade: PALETTE.lapisDeep,
      highlight: PALETTE.lapisLight,
      wash2: PALETTE.wine,
      illum: PALETTE.lapis,
      illum2: PALETTE.wine,
      widthFn(u) {
        if (u < 0.22) return 0.56 + 0.44 * Math.sin((u / 0.22) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.22) / 0.78), 1.10) + 0.08;
      }
    },
    'ouroboros': {
      label: 'Ouroboros alatus',
      attitude: 'regal',
      head: 'wyvern',
      legs: 2,
      wings: true,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 74,
      maxWidth: 9.0,
      headScale: 0.35,
      motion: 'concertina',
      wash: PALETTE.sage,
      headWash: PALETTE.cinnabar,
      shade: PALETTE.sageDeep,
      highlight: PALETTE.sageLight,
      wash2: PALETTE.gold,
      illum: PALETTE.sage,
      illum2: PALETTE.gold,
      widthFn(u) {
        if (u < 0.26) return 0.52 + 0.48 * Math.sin((u / 0.26) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.26) / 0.74), 1.10) + 0.08;
      }
    },
    'amphisbaena': {
      label: 'Amphisbaena',
      attitude: 'alert',
      head: 'serpent',
      legs: 0,
      wings: false,
      twoHeaded: true,
      pair: false,
      woodcut: false,
      bodyLen: 74,
      maxWidth: 8.6,
      headScale: 0.34,
      motion: 'concertina',
      wash: PALETTE.lapis,
      headWash: PALETTE.cinnabar,
      shade: PALETTE.lapisDeep,
      highlight: PALETTE.lapisLight,
      wash2: PALETTE.cinnabar,
      illum: PALETTE.lapis,
      illum2: PALETTE.cinnabar,
      widthFn(u) {
        const d = Math.abs(u - 0.5) * 2;
        return 0.54 + 0.46 * Math.cos(d * Math.PI * 0.5);
      }
    },
    'cute': {
      label: 'Wendling (odd one out)',
      attitude: 'playful',
      head: 'cute',
      legs: 0,
      wings: false,
      twoHeaded: false,
      pair: false,
      woodcut: false,
      bodyLen: 50,
      maxWidth: 9.6,
      headScale: 0.44,
      motion: 'undulate',
      wash: PALETTE.vermilion,
      headWash: PALETTE.vermilion,
      shade: PALETTE.cinnabarDeep,
      highlight: PALETTE.goldLight,
      wash2: PALETTE.gold,
      illum: PALETTE.vermilion,
      illum2: PALETTE.gold,
      widthFn(u) {
        if (u < 0.30) return 0.78 + 0.22 * Math.sin((u / 0.30) * Math.PI * 0.5);
        return 1.00 * Math.pow(Math.max(0, 1 - (u - 0.30) / 0.70), 0.85) + 0.16;
      }
    }
  };
  CREATURES.spotted = CREATURES['spotted-serpent'];
  CREATURES.wyvern = CREATURES['draco-alatus'];
  CREATURES.biscione = CREATURES.aspis;

  const REGISTERS = ['illuminated', 'woodcut'];

  /* ------------------------------------------------------------------ *
   * Visibility Observer (Phone performance: run ONLY while on screen!)
   * ------------------------------------------------------------------ */
  const _visObservers = typeof WeakMap !== 'undefined' ? new WeakMap() : null;

  function observeVisibility(svgEl, opts) {
    if (!svgEl || typeof IntersectionObserver === 'undefined') return null;
    opts = opts || {};
    if (_visObservers && _visObservers.has(svgEl)) {
      return _visObservers.get(svgEl);
    }
    const io = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        const target = entry.target;
        if (target.dataset && target.dataset.bestiaryManualPause === '1') {
          if (typeof target.pauseAnimations === 'function') target.pauseAnimations();
          continue;
        }
        if (entry.isIntersecting) {
          if (typeof target.unpauseAnimations === 'function') target.unpauseAnimations();
          if (target.dataset) delete target.dataset.bestiaryOffscreenPaused;
        } else {
          if (typeof target.pauseAnimations === 'function') target.pauseAnimations();
          if (target.dataset) target.dataset.bestiaryOffscreenPaused = '1';
        }
      }
    }, {
      rootMargin: opts.rootMargin || '60px 0px 60px 0px',
      threshold: opts.threshold !== undefined ? opts.threshold : 0.01
    });
    io.observe(svgEl);
    if (_visObservers) _visObservers.set(svgEl, io);
    return io;
  }

  function maybeAutoPause(svgEl, opts) {
    if (opts && opts.autoPause === false) return null;
    if (typeof window === 'undefined' || typeof IntersectionObserver === 'undefined') return null;
    return observeVisibility(svgEl, opts);
  }

  /* ------------------------------------------------------------------ *
   * Shared SVG <defs>: antique burnished gold & subtle mineral gradients
   * (lightweight, zero animated filters on mobile)
   * ------------------------------------------------------------------ */
  function ensureSharedDefs(svgRoot) {
    if (!svgRoot) return null;
    let defs = svgRoot.querySelector('defs[data-bestiary-shared="1"]');
    if (defs) return defs;
    defs = el('defs', { 'data-bestiary-shared': '1' });
    svgRoot.insertBefore(defs, svgRoot.firstChild);

    const goldGrad = el('linearGradient', {
      id: 'bestiary-gold-leaf', x1: '0%', y1: '0%', x2: '100%', y2: '100%'
    }, defs);
    el('stop', { offset: '0%', 'stop-color': PALETTE.goldLight }, goldGrad);
    el('stop', { offset: '42%', 'stop-color': PALETTE.gold }, goldGrad);
    el('stop', { offset: '100%', 'stop-color': PALETTE.goldDeep }, goldGrad);

    // Subtle mineral cinnabar-to-charcoal transition for border-drollery neck/body
    const drollGrad = el('linearGradient', {
      id: 'bestiary-drollery-body', x1: '0%', y1: '0%', x2: '100%', y2: '35%'
    }, defs);
    el('stop', { offset: '0%', 'stop-color': PALETTE.cinnabar }, drollGrad);
    el('stop', { offset: '36%', 'stop-color': '#8c4420' }, drollGrad);
    el('stop', { offset: '55%', 'stop-color': PALETTE.charcoal }, drollGrad);
    el('stop', { offset: '100%', 'stop-color': PALETTE.charcoalDeep }, drollGrad);

    return defs;
  }

  /* ------------------------------------------------------------------ *
   * Delicate 8-ray antique manuscript star (fine iron-gall + burnished gold)
   * ------------------------------------------------------------------ */
  function addManuscriptStar(parent, cx, cy, r, strokeCol) {
    const g = el('g', { transform: `translate(${f1(cx)},${f1(cy)})` }, parent);
    const d0 = r * 0.707;
    el('path', {
      d: `M${f1(-r)},0 L${f1(r)},0 M0,${f1(-r)} L0,${f1(r)} M${f1(-d0)},${f1(-d0)} L${f1(d0)},${f1(d0)} M${f1(-d0)},${f1(d0)} L${f1(d0)},${f1(-d0)}`,
      fill: 'none', stroke: INK, 'stroke-width': '1.6', 'stroke-linecap': 'round'
    }, g);
    el('path', {
      d: `M${f1(-r)},0 L${f1(r)},0 M0,${f1(-r)} L0,${f1(r)} M${f1(-d0)},${f1(-d0)} L${f1(d0)},${f1(d0)} M${f1(-d0)},${f1(d0)} L${f1(d0)},${f1(-d0)}`,
      fill: 'none', stroke: strokeCol || PALETTE.goldLight, 'stroke-width': '0.8', 'stroke-linecap': 'round'
    }, g);
    return g;
  }

  /* ------------------------------------------------------------------ *
   * Finely ribbed Medieval Manuscript / Woodcut Wing
   * (Direct homage to 2-winged-dragon-initial-interlace.jpg,
   *  3-dragon-border-drollery.jpg, and IMG_7329 DE DRACONE!)
   * Uses fine parallel pinion/rib strokes & ivory highlight bands
   * inside a crisp modulated contour — ZERO cartoonish block fills!
   * ------------------------------------------------------------------ */
  function buildTexturedWing(parent, cfg, scale, keyTimesStr, flapAngles, dur, beginAttr) {
    cfg = cfg || {};
    const baseFill = cfg.fill || 'var(--wash2, #ffb800)';
    const covertFill = cfg.covert || 'var(--wash, #ff5a1f)';

    const g = el('g', { transform: `scale(${f2(scale || 1)})` }, parent);
    const wing = el('g', {}, g);
    if (keyTimesStr && flapAngles) {
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: flapAngles, keyTimes: keyTimesStr,
        calcMode: 'spline', keySplines: buildKeySplines(flapAngles.split(';').length),
        dur: `${f1(dur)}s`, begin: beginAttr || '0s', repeatCount: 'indefinite'
      }, wing);
    }

    // Bold, flat, confident Romanesque scalloped pinion (cleaned from traced/components/roundel/wing.svg)
    el('path', {
      d: 'M0,0 C-5,-11 -15,-20 -29,-24 C-25,-16 -24,-10 -27,-6 C-20,-9 -15,-6 -17,-1 C-11,-5 -6,-3 -8,1 C-4,-2 -1,-1 0,0 Z',
      fill: baseFill, stroke: INK, 'stroke-width': '1.6', 'stroke-linejoin': 'round'
    }, wing);

    // Flat shoulder covert band
    el('path', {
      d: 'M0,0 C-4,-8 -10,-14 -16,-16 C-13,-10 -12,-6 -13,-3 C-8,-4 -4,-2 0,0 Z',
      fill: covertFill, stroke: INK, 'stroke-width': '1.35', 'stroke-linejoin': 'round'
    }, wing);

    // Three clean structural feather ribs (no noisy hatching)
    el('path', {
      d: 'M-2,-2 L-26,-21 M-5,-4 L-23,-8 M-4,-2 L-15,-1',
      fill: 'none', stroke: INK, 'stroke-width': '1.1', 'stroke-linecap': 'round'
    }, wing);

    // Lead-white dot rail along leading wing bone
    el('path', {
      d: 'M-3,-4 C-9,-10 -16,-16 -24,-20',
      fill: 'none', stroke: LEAD, 'stroke-width': '1.25', 'stroke-linecap': 'round', 'stroke-dasharray': '0.4 4.2'
    }, wing);

    return g;
  }

  function measureBubble(bubbleEl, opts, defW, defH) {
    opts = opts || {};
    const isDom = bubbleEl && typeof bubbleEl.getBoundingClientRect === 'function';
    let w, h, r;
    if (isDom) {
      const rect = bubbleEl.getBoundingClientRect();
      const cs = window.getComputedStyle ? window.getComputedStyle(bubbleEl) : null;
      w = rect.width || bubbleEl.offsetWidth || defW || 240;
      h = rect.height || bubbleEl.offsetHeight || defH || 68;
      r = cs ? (parseFloat(cs.borderTopLeftRadius) || 14) : 14;
    } else {
      w = (bubbleEl && bubbleEl.width) || opts.width || defW || 240;
      h = (bubbleEl && bubbleEl.height) || opts.height || defH || 68;
      r = (bubbleEl && bubbleEl.radius) || opts.radius || 14;
    }
    return { isDom, w, h, r };
  }

  function attachOverlay(bubbleEl, svg, isDom, opts) {
    if (isDom && opts.attach !== false) {
      if (window.getComputedStyle && window.getComputedStyle(bubbleEl).position === 'static') {
        bubbleEl.style.position = 'relative';
      }
      bubbleEl.appendChild(svg);
    }
    maybeAutoPause(svg, opts);
    return svg;
  }

  /* ------------------------------------------------------------------ *
   * Authentic Medieval Manuscript & Woodcut Head Library
   * (Direct study of 1-coiled-dragon-roundel.jpg, 2-winged-dragon-initial.jpg,
   *  3-dragon-border-drollery.jpg, and IMG_7329 DE DRACONE):
   * - Small, intense, earnest canine/equine medieval hybrid beast head
   * - Slightly blunt upturned snout, neat tiny articulated teeth
   * - Small dark/amber almond eye with radiating fine facial brushwork
   * - Tall flame-like swept-back twin ears/horns
   * - Slim barbed spear-arrow tongue (DE DRACONE) or delicate cinnabar flame-wisp
   * - NO cartoon grins, NO chunky 3px outlines, NO emoji crowns!
   * - ONLY baseType === 'cute' ("yuck") stays compact & endearing!
   * ------------------------------------------------------------------ */
  function addClaudeRuff(parent) {
    const ruff = el('g', { class: 'bestiary-claude-ruff' }, parent);
    const petalCount = 14;
    for (let i = 0; i < petalCount; i += 1) {
      const a = (i / petalCount) * Math.PI * 2 + 0.18;
      const deg = ((a * 180) / Math.PI) % 360;
      if (deg < 22 || deg > 338) continue;
      const len = 8.6 + ((i * 5) % 3) * 1.3;
      const w = 1.75;
      const cos = Math.cos(a);
      const sin = Math.sin(a);
      const nx = -sin;
      const ny = cos;
      const bx = -1.4 + cos * 2.8;
      const by = sin * 2.6;
      const tx = bx + cos * len;
      const ty = by + sin * len;
      const mx = bx + cos * (len * 0.58);
      const my = by + sin * (len * 0.58);
      const d = `M${f2(bx + nx * w * 0.65)},${f2(by + ny * w * 0.65)} ` +
        `Q${f2(mx + nx * w)},${f2(my + ny * w)} ${f2(tx)},${f2(ty)} ` +
        `Q${f2(mx - nx * w)},${f2(my - ny * w)} ${f2(bx - nx * w * 0.65)},${f2(by - ny * w * 0.65)} Z`;
      el('path', {
        d,
        fill: i % 2 === 0 ? PALETTE.claude : PALETTE.cinnabar,
        stroke: INK, 'stroke-width': '1.0', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, ruff);
      // Fine golden-ochre brushwork rib down each petal
      el('path', {
        d: `M${f2(bx + cos * 1.5)},${f2(by + sin * 1.5)} L${f2(bx + cos * (len * 0.72))},${f2(by + sin * (len * 0.72))}`,
        fill: 'none', stroke: PALETTE.goldLight, 'stroke-width': '0.52', 'stroke-linecap': 'round', opacity: '0.8'
      }, ruff);
    }
    // Subtle parchment nape badge with tiny ^ω^ motif
    const badge = el('g', { transform: 'translate(-2.2,0)' }, ruff);
    el('ellipse', { cx: '0', cy: '0', rx: '3.0', ry: '2.3', fill: IVORY, stroke: INK, 'stroke-width': '0.85' }, badge);
    el('path', {
      d: 'M-1.9,-.35 l.6,-.65 .6,.65 M.65,-.35 l.6,-.65 .6,.65 M-.95,.75 q.48,.65 .95,0 q.48,.65 .95,0',
      fill: 'none', stroke: INK, 'stroke-width': '0.48', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
    }, badge);
  }

  // Slender barbed spear-arrow or forked vermilion tongue (DE DRACONE & manuscripts!)
  function addBarbedTongue(parent, dur, tVals, tKeyTimes, tOpVals, isWoodcut) {
    const tongueG = el('g', { class: 'bestiary-tongue' }, parent);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: tVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tongueG);
    el('animate', {
      attributeName: 'opacity', values: tOpVals, keyTimes: tKeyTimes,
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tongueG);
    if (isWoodcut) {
      // Exact DE DRACONE barbed spear-arrow tongue!
      el('path', {
        d: 'M0,0 L12.5,0 M9.0,-1.8 L13.4,0 L9.0,1.8 L10.6,0 Z',
        fill: INK, stroke: INK, 'stroke-width': '0.85', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, tongueG);
    } else {
      el('path', {
        d: 'M0,0 C3.8,-0.25 7.0,0.35 10.4,0.0 M10.2,0.0 L13.8,-1.6 M10.2,0.0 L13.8,1.6',
        fill: 'none', stroke: INK, 'stroke-width': '1.35', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, tongueG);
      el('path', {
        d: 'M0,0 C3.8,-0.25 7.0,0.35 10.4,0.0 M10.2,0.0 L13.8,-1.6 M10.2,0.0 L13.8,1.6',
        fill: 'none', stroke: PALETTE.cinnabar, 'stroke-width': '0.72', 'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, tongueG);
    }
    return tongueG;
  }

  // Intense small medieval almond eye (amber or ivory sclera, dark pupil, fine radiating brow lines!)
  function addManuscriptEye(parent, cx, cy, dur, amberIris) {
    const eyeG = el('g', { class: 'bestiary-eye' }, parent);
    el('path', {
      d: `M${f1(cx - 2.3)},${f1(cy + 0.1)} Q${f1(cx)},${f1(cy - 1.65)} ${f1(cx + 2.3)},${f1(cy - 0.1)} Q${f1(cx)},${f1(cy + 1.2)} ${f1(cx - 2.3)},${f1(cy + 0.1)} Z`,
      fill: amberIris ? PALETTE.goldLight : LEAD, stroke: INK, 'stroke-width': '0.95', 'stroke-linejoin': 'round'
    }, eyeG);
    el('circle', { cx: f1(cx + 0.35), cy: f1(cy - 0.05), r: '0.88', fill: INK }, eyeG);
    el('circle', { cx: f1(cx + 0.1), cy: f1(cy - 0.32), r: '0.24', fill: LEAD }, eyeG);
    // Fine earnest brow fold + subtle cheek line
    el('path', {
      d: `M${f1(cx - 2.7)},${f1(cy - 0.5)} Q${f1(cx)},${f1(cy - 2.3)} ${f1(cx + 2.6)},${f1(cy - 0.7)}`,
      fill: 'none', stroke: INK, 'stroke-width': '0.95', 'stroke-linecap': 'round'
    }, eyeG);
    if (dur && dur > 0) {
      const lid = el('path', {
        d: `M${f1(cx - 2.4)},${f1(cy + 0.1)} Q${f1(cx)},${f1(cy - 1.65)} ${f1(cx + 2.4)},${f1(cy - 0.1)} Q${f1(cx)},${f1(cy + 1.2)} ${f1(cx - 2.4)},${f1(cy + 0.1)} Z`,
        fill: INK, opacity: '0'
      }, eyeG);
      el('animate', {
        attributeName: 'opacity',
        values: '0;0;1;1;0;0',
        keyTimes: '0;.64;.652;.666;.678;1',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lid);
    }
    return eyeG;
  }

  function buildHead(type, opts) {
    opts = opts || {};
    const wash = opts.wash || PALETTE.cinnabar;
    const isWoodcut = !!opts.woodcut || type === 'woodcut';
    const isClaudeRuffed = opts.headVariant === 'claude-ruffed' || type === 'claude-ruffed';
    const baseType = type === 'claude-ruffed' ? (opts.baseHead || 'wyvern') : type;
    const dur = opts.dur || 6.1;
    const flicks = opts.flicks || null;
    const brushCol = isWoodcut ? INK : (opts.brushCol || PALETTE.goldLight);

    const g = el('g', { class: `bestiary-head bestiary-head-${baseType}` });

    if (isClaudeRuffed) {
      addClaudeRuff(g);
    }

    let tKeyTimes = '0;.44;.49;.54;.59;.64;1';
    let tVals = '.04 1;.04 1;1 1;.45 1;1.05 1;.04 1;.04 1';
    let tOpVals = '0;0;1;1;1;0;0';
    let jVals = '3;3;15;7;17;4;3';
    if (flicks && flicks.length) {
      const kt = [0];
      const sc = ['.04 1'];
      const op = ['0'];
      const jw = ['3'];
      const sorted = flicks.slice().sort((a, b) => a.t0 - b.t0);
      for (const fl of sorted) {
        const a = clamp(fl.t0, 0.01, 0.94);
        const b = clamp(fl.t1, a + 0.02, 0.97);
        const m1 = a + (b - a) * 0.25;
        const m2 = a + (b - a) * 0.52;
        const m3 = a + (b - a) * 0.78;
        if (a > kt[kt.length - 1] + 0.005) {
          kt.push(a); sc.push('.04 1'); op.push('0'); jw.push('3');
        }
        kt.push(m1, m2, m3, b);
        sc.push('1 1', '.45 1', '1.05 1', '.04 1');
        op.push('1', '1', '1', '0');
        jw.push('15', '7', '17', '3');
      }
      if (kt[kt.length - 1] < 1) {
        kt.push(1); sc.push('.04 1'); op.push('0'); jw.push('3');
      }
      tKeyTimes = kt.map(f2).join(';');
      tVals = sc.join(';');
      tOpVals = op.join(';');
      jVals = jw.join(';');
    }

    // ONLY 'cute' ("yuck" odd-one-out) stays compact, warm, and endearing
    if (baseType === 'cute') {
      const headW = el('g', {}, g);
      el('path', {
        d: 'M-3.5,-5.4 C2.4,-8.6 11.0,-8.2 15.0,-3.8 C17.6,-0.8 17.2,3.8 13.8,6.0 C9.2,8.6 1.8,8.2 -3.5,5.0 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, headW);
      el('circle', { cx: '9.2', cy: '-1.3', r: '2.7', fill: INK }, headW);
      el('circle', { cx: '8.3', cy: '-2.1', r: '0.95', fill: LEAD }, headW);
      el('circle', { cx: '10.1', cy: '-0.5', r: '0.42', fill: LEAD }, headW);
      el('path', {
        d: 'M11.6,2.2 Q13.5,3.8 15.2,2.0',
        fill: 'none', stroke: INK, 'stroke-width': '1.25', 'stroke-linecap': 'round'
      }, headW);
      return g;
    }

    // ------------------------------------------------------------------
    // BRIEF-5 Cleaned Traced Heads: bold, flat, confident shapes with
    // one continuous dark outline, zero hatching/specks/cracks, and
    // per-joint SMIL jaw, blink & tongue motion.
    // ------------------------------------------------------------------
    const wash2 = opts.wash2 || 'var(--wash2, #ffb800)';
    function addJaw(jawD, teethD, pivX, pivY) {
      const jp = el('g', { transform: `translate(${f1(pivX)},${f1(pivY)})` }, g);
      const j = el('g', {}, jp);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, j);
      el('path', { d: jawD, fill: wash, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round' }, j);
      if (teethD) {
        el('path', { d: teethD, fill: LEAD, stroke: INK, 'stroke-width': '0.6', 'stroke-linejoin': 'round' }, j);
      }
      return j;
    }

    if (baseType === 'romanesque-s-horned') {
      // Cleaned from traced/components/romanesque-s-initial/dragon-head.svg:
      // tall curved twin horns, bold arched brow, gaping jaw + fangs
      el('path', { d: 'M5.5,-0.2 L18.5,-0.8 L17.0,2.4 L5.8,2.2 Z', fill: INK }, g);
      // Twin swept Romanesque horns
      el('path', {
        d: 'M1.4,-4.4 C-1.8,-8.8 -5.4,-11.8 -9.4,-12.4 C-6.4,-9.6 -3.2,-7.0 -0.4,-3.8 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M4.4,-4.6 C1.8,-9.2 -1.6,-12.8 -6.0,-14.0 C-3.0,-10.8 -0.2,-7.6 2.0,-4.0 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-3.0,-0.2 L12.4,0.2 C13.2,0.8 12.6,2.2 10.6,2.6 C6.4,3.4 0.8,3.0 -3.2,1.8 Z',
        'M5.2,-0.1 L6.0,-1.4 L6.8,0.0 M8.4,0.0 L9.2,-1.3 L10.0,0.1',
        5.6, 1.4
      );
      addBarbedTongue(el('g', { transform: 'translate(12.2,0.3)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      // Bold flat skull & hooked snout
      el('path', {
        d: 'M-3.2,-3.6 C0.4,-6.4 5.4,-6.8 10.4,-5.0 C14.0,-4.0 17.6,-3.6 20.6,-2.4 C21.8,-1.8 21.4,0.2 19.6,0.4 C15.6,0.4 10.8,0.2 6.4,1.0 C2.4,1.8 -1.2,2.2 -3.2,1.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.6', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M12.0,0.2 L12.8,1.6 L13.6,0.2 M15.4,0.2 L16.2,1.5 L17.0,0.2', fill: LEAD, stroke: INK, 'stroke-width': '0.6' }, g);
      el('circle', { cx: '19.0', cy: '-1.4', r: '0.7', fill: INK }, g);
      addManuscriptEye(g, 8.8, -2.6, dur, true);
      return g;
    }

    if (baseType === 'roundel-head') {
      // Cleaned from traced/components/roundel/head.svg:
      // blunt upturned Romanesque snout, upright twin pointed ears
      el('path', { d: 'M5.6,-0.2 L18.0,-0.8 L16.6,2.2 L6.0,2.0 Z', fill: INK }, g);
      el('path', {
        d: 'M0.8,-4.2 C-0.8,-8.4 -1.8,-11.4 -3.8,-12.6 C-2.8,-9.2 -1.6,-6.6 -0.4,-3.8 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M3.6,-4.2 C2.8,-8.2 1.6,-11.0 -0.2,-12.2 C0.4,-8.8 1.4,-6.2 2.2,-3.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-3.0,-0.2 L11.8,0.2 C12.4,0.8 12.0,2.0 10.4,2.4 C6.4,3.1 1.0,2.8 -3.2,1.8 Z',
        'M5.0,-0.1 L5.8,-1.3 L6.6,0.0 M8.0,0.0 L8.8,-1.2 L9.6,0.1',
        5.6, 1.3
      );
      addBarbedTongue(el('g', { transform: 'translate(12.0,0.3)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-3.2,-3.4 C0.2,-5.8 4.8,-6.2 9.4,-4.8 C13.2,-4.0 16.8,-4.0 19.4,-3.2 C20.8,-2.6 20.6,-0.4 19.0,0.0 C15.2,0.1 10.6,0.1 6.4,0.9 C2.6,1.6 -1.0,2.0 -3.2,1.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.6', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M11.6,0.0 L12.4,1.3 L13.2,0.0 M14.8,0.0 L15.6,1.2 L16.4,0.0', fill: LEAD, stroke: INK, 'stroke-width': '0.6' }, g);
      el('circle', { cx: '18.2', cy: '-1.8', r: '0.7', fill: INK }, g);
      addManuscriptEye(g, 8.4, -2.4, dur, true);
      return g;
    }

    if (baseType === 'aberdeen-head') {
      // Cleaned from traced/components/aberdeen-serpent-pillar/head.svg:
      // eared open-jawed Aberdeen Bestiary serpent skull
      el('path', { d: 'M5.2,-0.2 L17.6,-0.6 L16.2,2.4 L5.4,2.0 Z', fill: INK }, g);
      // Leaf-shaped swept Aberdeen ears
      el('path', {
        d: 'M1.2,-4.0 C-2.4,-7.0 -6.4,-8.2 -9.4,-7.6 C-6.6,-5.6 -3.4,-4.2 -0.6,-3.0 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M3.8,-4.2 C0.6,-7.6 -3.0,-9.6 -6.6,-9.8 C-3.8,-7.6 -1.0,-5.6 1.4,-3.4 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-2.8,-0.2 L12.2,0.4 C12.8,1.0 12.2,2.2 10.4,2.5 C6.2,3.0 0.8,2.6 -3.0,1.6 Z',
        'M5.6,-0.1 L6.4,-1.2 L7.2,0.1 M8.8,0.1 L9.6,-1.1 L10.4,0.2',
        5.2, 1.3
      );
      addBarbedTongue(el('g', { transform: 'translate(11.6,0.3)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-3.0,-3.2 C0.6,-5.6 5.4,-5.8 10.2,-4.4 C13.8,-3.4 17.2,-2.4 19.6,-1.2 C20.4,-0.6 20.0,0.6 18.4,0.6 C14.6,0.6 10.2,0.4 6.2,1.0 C2.6,1.6 -0.8,1.8 -3.0,1.6 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.55', 'stroke-linejoin': 'round'
      }, g);
      el('circle', { cx: '17.6', cy: '-0.8', r: '0.65', fill: INK }, g);
      addManuscriptEye(g, 8.2, -2.2, dur, true);
      return g;
    }

    if (baseType === 'calendar-head') {
      // Calendar serpent: sleek crested/horned viper head with crisp flat profile
      el('path', { d: 'M5.4,-0.2 L17.4,-0.6 L16.2,2.0 L5.6,1.8 Z', fill: INK }, g);
      el('path', {
        d: 'M2.2,-4.0 C-0.6,-7.8 -4.4,-10.2 -8.2,-10.4 C-5.0,-8.0 -2.2,-5.8 0.2,-3.4 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-2.6,-0.2 L11.4,0.2 C12.0,0.7 11.6,1.8 10.0,2.2 C6.2,2.8 1.0,2.4 -2.8,1.5 Z',
        'M6.0,-0.1 L6.7,-1.1 L7.4,0.0',
        5.4, 1.2
      );
      addBarbedTongue(el('g', { transform: 'translate(11.4,0.2)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-3.0,-3.0 C0.6,-5.4 5.2,-5.6 10.2,-4.2 C14.4,-3.0 18.0,-2.0 20.4,-0.8 C21.0,-0.2 20.4,0.8 18.8,0.8 C14.8,0.7 10.4,0.4 6.2,1.0 C2.6,1.6 -0.8,1.8 -3.0,1.5 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.55', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M13.2,0.5 L14.0,1.8 L14.8,0.5', fill: LEAD, stroke: INK, 'stroke-width': '0.55' }, g);
      el('circle', { cx: '18.4', cy: '-0.6', r: '0.6', fill: INK }, g);
      addManuscriptEye(g, 8.6, -2.0, dur, true);
      return g;
    }

    if (baseType === 'gothic-dragon-a' || baseType === 'gothic-dragon-b') {
      // Cleaned from traced/components/gothic-initial-interlace/green-dragon-head.svg & lower-dragon-head.svg:
      // roaring horned Gothic interlace dragon head
      const isB = baseType === 'gothic-dragon-b';
      el('path', { d: 'M5.6,-0.2 L18.8,-0.8 L17.2,2.6 L5.8,2.2 Z', fill: INK }, g);
      // Gothic notched crest & twin horns
      el('path', {
        d: isB
          ? 'M1.0,-4.2 C-2.2,-8.0 -5.6,-10.8 -9.6,-11.0 C-6.4,-8.4 -3.6,-6.2 -1.0,-3.6 Z'
          : 'M1.4,-4.4 C-1.4,-9.0 -4.8,-12.0 -8.8,-12.6 C-5.8,-9.6 -3.0,-7.0 -0.4,-3.8 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M4.6,-4.4 C2.4,-8.6 -0.6,-11.4 -4.2,-12.4 C-1.8,-9.6 0.4,-6.8 2.2,-3.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-3.0,-0.2 L12.8,0.2 C13.4,0.8 12.8,2.2 10.8,2.6 C6.6,3.4 0.8,3.0 -3.2,1.8 Z',
        'M4.8,-0.1 L5.6,-1.4 L6.4,0.0 M7.8,0.0 L8.6,-1.3 L9.4,0.1 M10.6,0.1 L11.4,-1.2 L12.0,0.2',
        5.6, 1.4
      );
      addBarbedTongue(el('g', { transform: 'translate(12.4,0.3)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-3.2,-3.6 C0.4,-6.2 5.4,-6.6 10.4,-5.0 C14.2,-4.0 17.8,-3.6 20.8,-2.2 C21.8,-1.6 21.4,0.2 19.6,0.4 C15.6,0.4 10.8,0.2 6.4,1.0 C2.6,1.7 -1.0,2.0 -3.2,1.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.6', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M11.6,0.2 L12.4,1.6 L13.2,0.2 M15.0,0.2 L15.8,1.5 L16.6,0.2', fill: LEAD, stroke: INK, 'stroke-width': '0.6' }, g);
      el('circle', { cx: '19.0', cy: '-1.3', r: '0.7', fill: INK }, g);
      addManuscriptEye(g, 8.8, -2.5, dur, true);
      return g;
    }

    if (baseType === 'border-drollery-head') {
      // Border-drollery dragon head: bold flat canine-dragon snout, long swept twin ears, crisp teeth
      el('path', { d: 'M5.8,-0.2 L18.4,-0.8 L16.8,2.2 L6.0,2.0 Z', fill: INK }, g);
      el('path', {
        d: 'M1.8,-4.2 C-1.4,-7.8 -5.4,-10.0 -9.4,-10.2 C-6.2,-8.2 -3.0,-6.0 -0.4,-3.6 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M4.4,-4.4 C1.6,-8.0 -2.0,-10.8 -6.0,-11.8 C-3.2,-9.4 -0.4,-6.8 1.8,-3.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.45', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-3.0,-0.2 L11.8,0.2 C12.4,0.7 12.0,1.9 10.4,2.3 C6.4,3.0 1.0,2.7 -3.2,1.7 Z',
        'M5.2,-0.1 L5.9,-1.2 L6.6,0.0 M8.2,0.0 L8.9,-1.1 L9.6,0.1',
        5.6, 1.3
      );
      addBarbedTongue(el('g', { transform: 'translate(12.0,0.3)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-2.8,-3.6 C0.4,-6.0 5.0,-6.2 9.8,-4.8 C13.2,-3.8 16.8,-3.2 19.8,-2.0 C21.0,-1.4 20.8,0.2 19.2,0.4 C15.2,0.4 10.6,0.2 6.4,1.0 C2.8,1.6 -0.6,2.0 -2.8,1.8 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.55', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M12.0,0.2 L12.8,1.4 L13.6,0.2 M15.2,0.2 L16.0,1.3 L16.8,0.2', fill: LEAD, stroke: INK, 'stroke-width': '0.55' }, g);
      el('circle', { cx: '18.6', cy: '-1.3', r: '0.65', fill: INK }, g);
      addManuscriptEye(g, 9.0, -2.5, dur, true);
      return g;
    }

    if (baseType === 'viper-head') {
      // Sharp diamond-horned viper head for the gold Swarm
      el('path', { d: 'M4.4,-0.2 L15.4,-0.5 L14.4,1.8 L4.6,1.6 Z', fill: INK }, g);
      el('path', {
        d: 'M0.8,-3.6 C-1.8,-6.6 -5.0,-8.0 -7.6,-7.8 C-5.0,-6.0 -2.4,-4.4 -0.4,-2.8 Z',
        fill: wash2, stroke: INK, 'stroke-width': '1.4', 'stroke-linejoin': 'round'
      }, g);
      addJaw(
        'M-2.2,-0.1 L10.2,0.2 C10.8,0.6 10.4,1.6 9.0,2.0 C5.4,2.6 0.8,2.2 -2.4,1.4 Z',
        'M6.2,-0.1 L6.8,-1.1 L7.4,0.0',
        4.6, 1.1
      );
      addBarbedTongue(el('g', { transform: 'translate(10.2,0.2)' }, g), dur, tVals, tKeyTimes, tOpVals, false);
      el('path', {
        d: 'M-2.8,-3.2 C0.4,-5.4 4.8,-5.4 9.6,-3.8 C13.2,-2.6 16.4,-1.4 18.2,-0.4 C18.8,0.1 18.2,0.9 16.8,0.9 C13.2,0.8 9.2,0.4 5.4,1.0 C2.0,1.5 -0.8,1.6 -2.8,1.4 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.5', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M12.2,0.5 L13.0,1.8 L13.8,0.5', fill: LEAD, stroke: INK, 'stroke-width': '0.55' }, g);
      addManuscriptEye(g, 7.6, -1.9, dur, true);
      return g;
    }

    if (baseType === 'basilisk') {
      // Textured mineral cockerel-basilisk (fine ribbed comb, antique ochre beak, intense almond eye)
      el('path', {
        d: 'M7.0,2.0 C6.2,4.6 7.8,6.6 9.6,6.0 C10.8,5.4 11.0,3.6 10.0,2.0 Z',
        fill: PALETTE.cinnabar, stroke: INK, 'stroke-width': '1.15', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M-1.2,-4.2 C-2.2,-7.0 -0.4,-8.2 1.2,-6.4 C2.2,-8.4 4.4,-8.0 4.6,-5.8 C6.0,-7.2 7.6,-6.4 7.0,-4.2 Z',
        fill: PALETTE.cinnabar, stroke: INK, 'stroke-width': '1.15', 'stroke-linejoin': 'round'
      }, g);
      el('path', { d: 'M7.0,0.0 L16.0,-0.4 L15.0,1.9 L7.2,1.7 Z', fill: INK }, g);

      const jawPiv = el('g', { transform: 'translate(6.6,1.3)' }, g);
      const jaw = el('g', {}, jawPiv);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, jaw);
      el('path', {
        d: 'M-2.2,-0.3 L10.2,0.2 C10.6,0.7 10.0,1.8 8.4,2.1 C4.6,2.8 0.4,2.4 -3.0,1.5 Z',
        fill: PALETTE.gold, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
      }, jaw);

      const tPiv = el('g', { transform: 'translate(10.6,0.25)' }, g);
      addBarbedTongue(tPiv, dur, tVals, tKeyTimes, tOpVals, isWoodcut);

      el('path', {
        d: 'M-3.0,-3.6 C-0.2,-6.0 4.4,-6.2 8.8,-4.8 C11.2,-4.0 12.6,-2.2 12.8,-0.5 C11.0,1.1 6.8,2.0 1.8,2.4 C-1.0,2.6 -3.0,2.0 -3.0,-3.6 Z',
        fill: wash, stroke: INK, 'stroke-width': '1.35', 'stroke-linejoin': 'round'
      }, g);
      // Fine radiating cheek brushwork
      el('path', {
        d: 'M-1.2,-2.2 L3.4,-3.6 M-1.0,-0.6 L3.8,-1.4 M-0.6,1.0 L3.4,0.4',
        fill: 'none', stroke: brushCol, 'stroke-width': '0.58', 'stroke-linecap': 'round', opacity: '0.85'
      }, g);
      el('path', {
        d: 'M11.0,-3.8 C14.8,-3.4 18.6,-1.8 20.0,0.2 C20.2,0.9 19.2,1.5 18.2,0.7 C16.8,-0.2 14.4,-0.5 11.0,-0.3 Z',
        fill: PALETTE.gold, stroke: INK, 'stroke-width': '1.3', 'stroke-linejoin': 'round'
      }, g);
      addManuscriptEye(g, 8.2, -2.3, dur, true);
      return g;
    }

    if (baseType === 'serpent' || baseType === 'woodcut') {
      // Study of 1-coiled-dragon-roundel.jpg & IMG_7329 DE DRACONE:
      // Ivory/mineral or woodcut beast-serpent skull with radiating cinnabar/ink facial brushwork,
      // pointed twin ears/horns, neat articulated rows of tiny teeth, barbed spear-arrow tongue!
      const skullFill = isWoodcut ? IVORY : (wash === PALETTE.wine ? IVORY : wash);
      const faceStroke = isWoodcut ? INK : (skullFill === IVORY ? PALETTE.cinnabar : PALETTE.goldLight);

      el('path', { d: 'M5.8,-0.1 L17.8,-0.7 L16.4,2.1 L6.2,1.9 Z', fill: INK }, g);

      // Twin tall alert pointed ears (1-coiled-dragon-roundel.jpg & DE DRACONE!)
      el('path', {
        d: 'M1.2,-4.0 C-0.6,-7.6 -1.6,-10.0 -3.2,-11.0 C-2.4,-8.4 -1.4,-6.2 -0.4,-3.6 Z',
        fill: skullFill, stroke: INK, 'stroke-width': '1.15', 'stroke-linejoin': 'round'
      }, g);
      el('path', {
        d: 'M3.6,-4.0 C2.6,-7.4 1.4,-9.8 -0.2,-10.8 C0.4,-8.2 1.4,-6.0 2.0,-3.6 Z',
        fill: skullFill, stroke: INK, 'stroke-width': '1.15', 'stroke-linejoin': 'round'
      }, g);

      // Lower jaw with neat tiny white teeth
      const jawPiv = el('g', { transform: 'translate(5.6,1.3)' }, g);
      const jaw = el('g', {}, jawPiv);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, jaw);
      el('path', {
        d: 'M-3.2,-0.2 L11.4,0.2 C12.0,0.6 11.6,1.6 10.2,2.0 C6.4,2.7 1.0,2.5 -3.4,1.6 Z',
        fill: skullFill, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
      }, jaw);
      el('path', {
        d: 'M4.8,-0.1 L5.4,-1.1 L6.0,0.0 M7.2,0.0 L7.8,-1.0 L8.4,0.1 M9.4,0.1 L10.0,-0.9 L10.6,0.2',
        fill: LEAD, stroke: INK, 'stroke-width': '0.52', 'stroke-linejoin': 'round'
      }, jaw);

      // Barbed spear-arrow or forked tongue
      const tPiv = el('g', { transform: 'translate(11.8,0.25)' }, g);
      addBarbedTongue(tPiv, dur, tVals, tKeyTimes, tOpVals, isWoodcut);

      // Sculpted slightly blunt upturned snout (exact 1-coiled-dragon-roundel.jpg silhouette!)
      el('path', {
        d: 'M-3.0,-3.2 C0.0,-5.4 4.4,-5.8 9.0,-4.6 C12.4,-3.8 16.0,-3.4 18.8,-2.8 C20.2,-2.4 20.4,-0.8 19.0,-0.2 C15.2,-0.1 10.6,0.0 6.4,0.8 C2.8,1.4 -0.8,1.9 -3.0,1.7 Z',
        fill: skullFill, stroke: INK, 'stroke-width': '1.35', 'stroke-linejoin': 'round'
      }, g);
      // Tiny articulated upper teeth
      el('path', {
        d: 'M11.6,-0.1 L12.2,1.1 L12.8,-0.1 M14.2,-0.1 L14.8,1.0 L15.4,-0.1 M16.8,-0.1 L17.4,0.9 L18.0,-0.1',
        fill: LEAD, stroke: INK, 'stroke-width': '0.52', 'stroke-linejoin': 'round'
      }, g);
      // Radiating fine cinnabar / ink facial brushwork (signature texture of 1-coiled-dragon-roundel.jpg & DE DRACONE!)
      el('path', {
        d: 'M-1.4,-3.0 C1.0,-4.0 3.6,-4.4 6.0,-3.8 M-1.6,-1.2 L4.6,-2.2 M-1.0,0.6 L5.2,-0.2 M10.8,-3.2 L14.8,-2.4 M11.6,-1.8 L15.4,-1.4',
        fill: 'none', stroke: faceStroke, 'stroke-width': '0.68', 'stroke-linecap': 'round', opacity: '0.88'
      }, g);
      // Cheek jaw-scroll accent
      el('path', {
        d: 'M0.4,1.2 C2.6,1.0 4.6,0.2 6.0,-0.8',
        fill: 'none', stroke: INK, 'stroke-width': '0.7', 'stroke-linecap': 'round'
      }, g);
      el('circle', { cx: '18.0', cy: '-1.8', r: '0.55', fill: INK }, g);

      addManuscriptEye(g, 8.6, -2.4, dur, !isWoodcut);
      return g;
    }

    // Default: 'wyvern' — Direct study of 3-dragon-border-drollery.jpg & 2-winged-dragon-initial-interlace.jpg:
    // Earnest canine/equine medieval hybrid head, tall flame-like swept-back twin ears/horns,
    // fine parallel ochre/ink brushwork along cheek & brow, blunt snout, neat tiny white teeth,
    // small intense dark/amber almond eye, zero clownish crown!
    el('path', { d: 'M6.0,-0.1 L18.2,-0.7 L16.6,2.1 L6.4,1.9 Z', fill: INK }, g);

    // Tall flame-like swept-back twin horns/ears (3-dragon-border-drollery.jpg!)
    el('path', {
      d: 'M2.0,-4.2 C-0.8,-7.4 -4.6,-9.4 -8.2,-9.6 C-5.4,-7.8 -2.6,-5.8 -0.2,-3.6 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
    }, g);
    el('path', {
      d: 'M4.4,-4.2 C1.8,-7.6 -1.6,-10.2 -5.4,-11.2 C-2.8,-9.0 -0.2,-6.6 1.8,-3.8 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
    }, g);
    // Fine golden-ochre highlight stroke along swept ear edge
    el('path', {
      d: 'M3.2,-5.0 C1.0,-7.4 -1.6,-9.2 -4.2,-10.0',
      fill: 'none', stroke: brushCol, 'stroke-width': '0.68', 'stroke-linecap': 'round', opacity: '0.88'
    }, g);

    // Subtle medieval ruff / mane wisps at nape (2-winged-dragon-initial.jpg!)
    el('path', {
      d: 'M-1.6,-2.4 L-5.2,-3.6 L-2.8,-1.2 L-5.6,-0.2 L-2.8,1.0 L-4.8,2.4 L-1.4,2.0 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.05', 'stroke-linejoin': 'round'
    }, g);

    // Smaller articulated lower jaw with neat tiny teeth
    const jawPiv = el('g', { transform: 'translate(5.6,1.2)' }, g);
    const jaw = el('g', {}, jawPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: jVals, keyTimes: tKeyTimes, dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, jaw);
    el('path', {
      d: 'M-3.0,-0.2 L11.2,0.2 C11.8,0.6 11.4,1.6 10.0,1.9 C6.2,2.6 1.0,2.4 -3.2,1.5 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
    }, jaw);
    el('path', {
      d: 'M4.8,-0.1 L5.3,-1.1 L5.9,0.0 M7.2,0.0 L7.7,-1.0 L8.3,0.1 M9.4,0.1 L9.9,-0.9 L10.5,0.2',
      fill: LEAD, stroke: INK, 'stroke-width': '0.5', 'stroke-linejoin': 'round'
    }, jaw);

    // Barbed/forked vermilion tongue
    const tPiv = el('g', { transform: 'translate(12.0,0.25)' }, g);
    addBarbedTongue(tPiv, dur, tVals, tKeyTimes, tOpVals, isWoodcut);

    // Earnest canine/equine medieval upper snout (slightly blunt tip, articulated brow)
    el('path', {
      d: 'M-2.6,-3.4 C0.4,-5.6 4.8,-6.0 9.6,-4.6 C12.8,-3.8 16.4,-3.2 19.6,-2.2 C20.8,-1.8 20.8,-0.2 19.2,0.0 C15.4,0.1 10.8,0.0 6.6,0.8 C3.0,1.4 -0.4,1.9 -2.4,1.7 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.35', 'stroke-linejoin': 'round'
    }, g);
    el('path', {
      d: 'M11.6,0.0 L12.2,1.1 L12.8,0.0 M14.2,0.0 L14.8,1.0 L15.4,0.0 M16.8,0.0 L17.4,0.9 L18.0,0.0',
      fill: LEAD, stroke: INK, 'stroke-width': '0.5', 'stroke-linejoin': 'round'
    }, g);
    // Fine parallel ochre/ivory brushwork along snout, brow & cheek (3-dragon-border-drollery.jpg!)
    el('path', {
      d: 'M-1.2,-2.6 C1.8,-4.0 5.0,-4.4 8.0,-3.8 M-0.8,-1.0 L5.4,-1.8 M-0.4,0.6 L6.0,-0.2 M11.6,-3.0 L16.2,-2.0',
      fill: 'none', stroke: brushCol, 'stroke-width': '0.62', 'stroke-linecap': 'round', opacity: '0.86'
    }, g);
    el('circle', { cx: '18.6', cy: '-1.4', r: '0.55', fill: INK }, g);

    addManuscriptEye(g, 9.0, -2.5, dur, true);
    return g;
  }

  /* ------------------------------------------------------------------ *
   * Finely textured medieval legs (smoky olive-charcoal or woodcut ivory
   * with fine parallel muscle-rib lines & 3-toed dark claws)
   * ------------------------------------------------------------------ */
  function buildMiniLeg(wash, ribCol, scale, keyTimesStr, stepAngles, dur) {
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
      d: 'M-2.4,-1.4 C-3.2,2.4 -2.4,6.2 -0.6,8.2 C1.4,8.6 3.0,7.0 3.2,3.8 C3.4,1.2 2.4,-1.4 1.0,-1.8 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.3', 'stroke-linejoin': 'round'
    }, upper);
    // Fine parallel muscle/scale rib strokes
    el('path', {
      d: 'M-1.2,0.8 L1.8,1.8 M-1.4,2.8 L1.6,3.8 M-1.0,4.8 L1.2,5.6',
      fill: 'none', stroke: ribCol || PALETTE.charcoalLight, 'stroke-width': '0.6', 'stroke-linecap': 'round', opacity: '0.85'
    }, upper);
    const knee = el('g', { transform: 'translate(0,7.4)' }, upper);
    const lower = el('g', {}, knee);
    if (keyTimesStr && stepAngles) {
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: stepAngles.knee, keyTimes: keyTimesStr,
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lower);
    }
    el('path', {
      d: 'M-1.4,-0.8 L0.5,6.0 L2.2,5.4 L1.0,-1.0 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
    }, lower);
    el('path', {
      d: 'M0.2,5.6 L5.4,5.8 L2.0,6.8 L6.0,7.6 L1.0,8.0 L-1.0,6.6 Z',
      fill: INK, stroke: INK, 'stroke-width': '0.75', 'stroke-linejoin': 'round'
    }, lower);
    return g;
  }

  /* ------------------------------------------------------------------ *
   * Path measurement, cubic smoothing, spatial lateral undulation, and
   * strict post-hole tight coil extension (0-stray-pixel Burrow Contract).
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
      const env = Math.sin(clamp(u / 0.12, 0, 1) * Math.PI * 0.5) *
                  Math.sin(clamp((1 - u) / 0.14, 0, 1) * Math.PI * 0.5);
      const off = amp * env * Math.sin((pt.s / wavelength) * Math.PI * 2);
      return { x: pt.x + pt.nx * off, y: pt.y + pt.ny * off, s: pt.s };
    });
  }

  function buildBurrowGeometry(pathD, hole, startHole, bodyLen, maxHalfExtent, undAmp, undWave) {
    const raw = measurePath(pathD);
    let visPts = raw.pts;

    const first = visPts[0];
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

    const undPts = applySpatialUndulation(visPts, visPts[visPts.length - 1].s, undAmp, undWave);
    if (hole && undPts.length >= 2) {
      undPts[undPts.length - 1].x = hole.x;
      undPts[undPts.length - 1].y = hole.y;
    }
    if (startHole && undPts.length >= 2) {
      undPts[0].x = startHole.x;
      undPts[0].y = startHole.y;
    }

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

    let prePts = [];
    let startMaskDisc = null;
    let preLen = 0;

    if (startHole) {
      const stx = first.tx;
      const sty = first.ty;
      const snx = -sty;
      const sny = stx;
      const shr = Math.max(1.2, startHole.r || 3);
      const coilR = 1.1;
      const margin = 4.5;
      const rMask = Math.max(shr, maxHalfExtent + coilR + margin);
      const shift = Math.max(0, rMask - shr);
      const smx = startHole.x - stx * shift;
      const smy = startHole.y - sty * shift;

      const needPre = Math.max(bodyLen * 1.35 + shift + 16, 100);
      const revPts = [];
      const leadSteps = Math.max(2, Math.ceil(shift / 4));
      for (let i = 1; i <= leadSteps; i += 1) {
        const frac = i / leadSteps;
        revPts.push({ x: startHole.x - stx * (shift * frac), y: startHole.y - sty * (shift * frac) });
      }
      let acc = shift;
      let theta = 0;
      const dTheta = Math.PI / 4;
      while (acc < needPre) {
        theta += dTheta;
        const px = smx + (-Math.cos(theta) * stx + Math.sin(theta) * snx) * coilR;
        const py = smy + (-Math.cos(theta) * sty + Math.sin(theta) * sny) * coilR;
        const prev = revPts[revPts.length - 1];
        acc += Math.hypot(px - prev.x, py - prev.y);
        revPts.push({ x: px, y: py });
      }
      prePts = revPts.reverse();
      const preD = ptsToCubicD(prePts.concat([cleanVis[0]]), false);
      preLen = measurePath(preD, 32).total;
      startMaskDisc = { cx: smx, cy: smy, r: rMask, holeX: startHole.x, holeY: startHole.y, holeR: shr, shift };
    }

    if (!hole) {
      const combinedPts = prePts.concat(cleanVis);
      const combinedD = ptsToCubicD(combinedPts, false);
      const combinedLen = measurePath(combinedD, 48).total;
      return {
        fullD: combinedD,
        preLen,
        visLen,
        fullLen: combinedLen,
        maskDisc: null,
        startMaskDisc
      };
    }

    const hr = Math.max(1.2, hole.r || 3);
    const coilR = 1.1;
    const margin = 4.5;
    const rMask = Math.max(hr, maxHalfExtent + coilR + margin);
    const shift = Math.max(0, rMask - hr);
    const mx = hole.x + tx * shift;
    const my = hole.y + ty * shift;
    const nx = -ty;
    const ny = tx;

    const needLen = Math.max(bodyLen * 1.85 + shift + 30, 150);
    const extPts = [];
    const leadSteps = Math.max(2, Math.ceil(shift / 4));
    for (let i = 1; i <= leadSteps; i += 1) {
      const frac = i / leadSteps;
      extPts.push({
        x: hole.x + tx * (shift * frac),
        y: hole.y + ty * (shift * frac)
      });
    }
    let accLen = shift;
    const dTheta = Math.PI / 4;
    let theta = 0;
    while (accLen < needLen) {
      theta += dTheta;
      const px = mx + (Math.cos(theta) * tx + Math.sin(theta) * nx) * coilR;
      const py = my + (Math.cos(theta) * ty + Math.sin(theta) * ny) * coilR;
      const prev = extPts[extPts.length - 1];
      accLen += Math.hypot(px - prev.x, py - prev.y);
      extPts.push({ x: px, y: py });
    }

    const fullPts = prePts.concat(cleanVis, extPts);
    const fullD = ptsToCubicD(fullPts, false);
    const fullLen = measurePath(fullD, 64).total;

    return {
      fullD,
      preLen,
      visLen,
      fullLen,
      maskDisc: { cx: mx, cy: my, r: rMask, holeX: hole.x, holeY: hole.y, holeR: hr, shift },
      startMaskDisc
    };
  }

  function buildDefaultStoryboard(motion) {
    if (motion === 'concertina') {
      return [
        { t: 0.00, u: 0.00 },
        { t: 0.16, u: 0.22 },
        { t: 0.24, u: 0.22 },
        { t: 0.44, u: 0.50 },
        { t: 0.54, u: 0.62 },
        { t: 0.63, u: 0.62 },
        { t: 0.74, u: 0.78 },
        { t: 0.88, u: 1.00 },
        { t: 1.00, u: 1.00 }
      ];
    }
    return [
      { t: 0.00, u: 0.00 },
      { t: 0.18, u: 0.24 },
      { t: 0.26, u: 0.24 },
      { t: 0.48, u: 0.56 },
      { t: 0.56, u: 0.64 },
      { t: 0.64, u: 0.64 },
      { t: 0.88, u: 1.00 },
      { t: 1.00, u: 1.00 }
    ];
  }

  function normalizeStoryboard(sb, motion) {
    const raw = (sb && sb.length >= 2 ? sb : buildDefaultStoryboard(motion))
      .map((k) => ({ t: clamp(k.t, 0, 1), u: k.u }))
      .sort((a, b) => a.t - b.t);
    if (raw[0].t > 0) raw.unshift({ t: 0, u: raw[0].u });
    if (raw[raw.length - 1].t < 1) raw.push({ t: 1, u: raw[raw.length - 1].u });
    const out = [raw[0]];
    for (let i = 1; i < raw.length; i += 1) {
      const prev = out[out.length - 1];
      const t = raw[i].t <= prev.t ? Math.min(1, prev.t + 0.002) : raw[i].t;
      out.push({ t, u: raw[i].u });
    }
    out[out.length - 1].t = 1;
    return out;
  }

  /* ------------------------------------------------------------------ *
   * 1. makeCrawler(opts) -> SVGGElement
   * Text-weaving burrower in rich multi-band mineral gouache (1-coiled-
   * dragon-roundel.jpg) OR monochrome woodcut (IMG_7329 DE DRACONE),
   * preserving the exact 0-stray-pixel shifted circular <mask> contract.
   * ------------------------------------------------------------------ */
  function makeCrawler(opts) {
    opts = opts || {};
    const spec = CREATURES[opts.creature] || CREATURES['draco-bipes'];
    const register = opts.register || (spec.woodcut ? 'woodcut' : 'illuminated');
    const isWoodcut = register === 'woodcut' || !!opts.woodcut || (spec.woodcut && !opts.wash);
    const scale = opts.scale !== undefined ? opts.scale : 1;
    const wash = opts.wash || (isWoodcut ? IVORY : spec.wash);
    const headWash = opts.headWash || (isWoodcut ? IVORY : (opts.wash ? opts.wash : (spec.headWash || wash)));
    const shadeCol = opts.shade || spec.shade || PALETTE.charcoalDeep;
    const hiCol = opts.highlight || spec.highlight || IVORY;
    const wash2 = opts.wash2 || spec.wash2;
    const dur = opts.dur || 12.5;
    const motion = opts.motion && opts.motion !== 'auto' ? opts.motion : spec.motion;
    const hole = opts.hole || null;
    const startHole = opts.startHole || null;
    const headType = opts.headVariant === 'claude-ruffed' ? 'claude-ruffed' : (opts.headVariant || spec.head);

    const bodyLen = spec.bodyLen * scale;
    const maxWidth = spec.maxWidth * scale;
    const headScale = spec.headScale * scale;
    const ruffExtra = headType === 'claude-ruffed' ? 16 * headScale : 0;
    const wingExtra = spec.wings ? 28 * scale : 0;
    const legExtra = spec.legs ? 14 * scale : 0;
    const maxHalfExtent = Math.max(
      maxWidth * 0.9 + 2.2 * scale,
      24 * headScale + ruffExtra,
      wingExtra,
      legExtra
    );

    const undAmp = opts.undAmp !== undefined ? opts.undAmp : (motion === 'undulate' ? 2.8 * scale : 1.4 * scale);
    const undWave = 22 * scale;
    const geom = buildBurrowGeometry(opts.path, hole, startHole, bodyLen, maxHalfExtent, undAmp, undWave);
    const sb = normalizeStoryboard(opts.storyboard, motion);

    const sStart = startHole ? Math.max(0, geom.preLen - (geom.startMaskDisc.shift + 4)) : 0;
    const holeEntryPad = hole ? (geom.maskDisc.shift + 12 * scale + 6) : 0;
    const sEnd = hole ? (geom.preLen + geom.visLen + bodyLen + holeEntryPad) : (geom.preLen + geom.visLen);
    const safeFullLen = Math.max(geom.fullLen, sEnd + 4);

    const rootG = el('g', {
      class: `bestiary-crawler bestiary-${register}`,
      'data-creature': opts.creature || 'draco-bipes',
      'data-register': register
    });
    const localDefs = el('defs', {}, rootG);

    const pathId = uid('crawler-path');
    el('path', { id: pathId, d: geom.fullD, fill: 'none' }, localDefs);

    let targetG = rootG;
    if (geom.maskDisc || geom.startMaskDisc) {
      const maskId = uid('crawler-hole-mask');
      const mask = el('mask', {
        id: maskId,
        maskUnits: 'userSpaceOnUse',
        x: '-10000', y: '-10000', width: '20000', height: '20000'
      }, localDefs);
      el('rect', { x: '-10000', y: '-10000', width: '20000', height: '20000', fill: '#ffffff' }, mask);
      if (geom.maskDisc) {
        el('circle', {
          cx: f2(geom.maskDisc.cx), cy: f2(geom.maskDisc.cy), r: f2(geom.maskDisc.r),
          fill: '#000000'
        }, mask);
      }
      if (geom.startMaskDisc) {
        el('circle', {
          cx: f2(geom.startMaskDisc.cx), cy: f2(geom.startMaskDisc.cy), r: f2(geom.startMaskDisc.r),
          fill: '#000000'
        }, mask);
      }
      targetG = el('g', { mask: `url(#${maskId})` }, rootG);
    }

    const keyTimesStr = sb.map((k) => f4(k.t)).join(';');
    const keySplinesStr = buildKeySplines(sb.length);
    const sHeadAt = (k) => clamp(sStart + k.u * (sEnd - sStart), 0, geom.fullLen);

    function bandOffsetAt(kIdx, baseFrac) {
      const baseOff = baseFrac * bodyLen;
      if (motion !== 'concertina') return baseOff;
      const prev = sb[Math.max(0, kIdx - 1)];
      const next = sb[Math.min(sb.length - 1, kIdx + 1)];
      const vel = (next.u - prev.u) / Math.max(0.01, next.t - prev.t);
      const bunch = vel < 0.25 ? 0.88 : 1.04;
      return baseOff * bunch;
    }

    const gap = Math.ceil(geom.fullLen + bodyLen + 400);
    function addDashBand(parent, frac0, frac1, strokeColor, strokeWidth, opacity, dashPattern, lineCap) {
      const segLen = Math.max(1.2, (frac1 - frac0) * bodyLen);
      const period = segLen + gap;
      const vals = sb.map((k, idx) => {
        const sHead = sHeadAt(k);
        const s0 = sHead - bandOffsetAt(idx, frac1);
        return f1(period - s0);
      }).join(';');
      const p = el('path', {
        d: geom.fullD,
        fill: 'none',
        stroke: strokeColor,
        'stroke-width': f2(strokeWidth),
        'stroke-linecap': lineCap || 'round',
        'stroke-linejoin': 'round',
        'stroke-dasharray': dashPattern || `${f1(segLen)} ${gap}`,
        'stroke-dashoffset': vals.split(';')[0],
        opacity: opacity !== undefined ? f2(opacity) : '1'
      }, parent);
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

    // Layered textured body:
    // - Woodcut mode (DE DRACONE): aged parchment ivory core + fine transverse woodcut scute ribs + dark ocelli dots!
    // - Mineral gouache mode (1-coiled-dragon-roundel.jpg): crisp fine contour + pale ivory outer edge
    //   + rich mineral mid-band + deep inner shadow band + fine rib ticks + single crisp row of tiny lead-white seam dots!
    function buildBodyRibbons(parent, colorMain, colorHi, colorShade, phaseShiftFrac, woodcutMode) {
      const bands = 10;
      const bodyG = el('g', { class: 'bestiary-body' }, parent);
      const outlineExtra = 2.2 * scale;

      // Layer 1: Bold continuous dark outline
      for (let i = 0; i < bands; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(1, (i + 1) / bands + 0.03);
        const uMid = (f0 + f1b) * 0.5;
        const w = Math.max(0.9, spec.widthFn(uMid) * maxWidth);
        addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, INK, w + outlineExtra, 1);
      }

      // Layer 2: Lighter belly band + saturated var(--wash) core
      for (let i = 0; i < bands; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(1, (i + 1) / bands + 0.03);
        const uMid = (f0 + f1b) * 0.5;
        const w = Math.max(0.8, spec.widthFn(uMid) * maxWidth);
        addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, BELLY, w, 1);
        addDashBand(bodyG, f0 + phaseShiftFrac, f1b + phaseShiftFrac, colorMain, w * 0.74, 1);
      }

      // Layer 3: Fine brushwork / woodcut scale rib-strokes + crisp tiny lead-white seam dot row!
      for (let i = 0; i < bands - 1; i += 1) {
        const f0 = i / bands;
        const f1b = Math.min(0.92, (i + 1) / bands);
        const uMid = (f0 + f1b) * 0.5;
        const w = spec.widthFn(uMid) * maxWidth;
        if (w < 1.8 * scale) continue;

        const segLen = (f1b - f0) * bodyLen;
        const bPeriod = segLen + gap;

        // Fine transverse rib/scute strokes locked to same timeline (DE DRACONE / border drollery!)
        const ribStep = Math.max(2.6, 3.2 * scale);
        const numR = Math.max(1, Math.floor(segLen / ribStep));
        const rPat = [];
        for (let r = 0; r < numR; r += 1) rPat.push(`0.75 ${f1(ribStep - 0.75)}`);
        rPat.push(`0.01 ${f1(gap + Math.max(0, segLen - numR * ribStep))}`);
        const rVals = sb.map((k, idx) => {
          const sHead = sHeadAt(k);
          const s0 = sHead - bandOffsetAt(idx, f1b + phaseShiftFrac);
          return f1(bPeriod - s0);
        }).join(';');
        const rp = el('path', {
          d: geom.fullD, fill: 'none', stroke: INK,
          'stroke-width': f2(w * (woodcutMode ? 0.84 : 0.72)),
          'stroke-linecap': 'butt',
          'stroke-dasharray': rPat.join(' '),
          'stroke-dashoffset': rVals.split(';')[0],
          opacity: woodcutMode ? '0.68' : '0.32'
        }, bodyG);
        el('animate', {
          attributeName: 'stroke-dashoffset',
          values: rVals, keyTimes: keyTimesStr,
          calcMode: 'spline', keySplines: keySplinesStr,
          dur: `${f1(dur)}s`, repeatCount: 'indefinite'
        }, rp);

        // Single crisp row of tiny dots along the spine seam:
        // dark ocelli ring dots in woodcut mode (DE DRACONE), tiny lead-white pearl dots in mineral mode (1-coiled-dragon-roundel.jpg)!
        const dotGap = Math.max(3.2, 4.4 * scale);
        const numDots = Math.max(1, Math.floor(segLen / dotGap));
        const pat = [];
        for (let d = 0; d < numDots; d += 1) pat.push(`0.35 ${f1(dotGap - 0.35)}`);
        pat.push(`0.01 ${f1(gap + Math.max(0, segLen - numDots * dotGap))}`);
        const dotVals = sb.map((k, idx) => {
          const sHead = sHeadAt(k);
          const s0 = sHead - bandOffsetAt(idx, f1b + phaseShiftFrac);
          return f1(bPeriod - s0);
        }).join(';');
        const dp = el('path', {
          d: geom.fullD, fill: 'none', stroke: woodcutMode ? INK : LEAD,
          'stroke-width': f2(Math.max(0.85, w * 0.19)),
          'stroke-linecap': 'round',
          'stroke-dasharray': pat.join(' '),
          'stroke-dashoffset': dotVals.split(';')[0],
          opacity: woodcutMode ? '0.88' : '0.96'
        }, bodyG);
        el('animate', {
          attributeName: 'stroke-dashoffset',
          values: dotVals, keyTimes: keyTimesStr,
          calcMode: 'spline', keySplines: keySplinesStr,
          dur: `${f1(dur)}s`, repeatCount: 'indefinite'
        }, dp);
      }
    }

    function attachAtBodyOffset(parent, offsetFromHead, childNode, flip180) {
      const holder = el('g', {}, parent);
      const kp = sb.map((k, idx) => {
        const sHead = sHeadAt(k);
        const s = clamp(sHead - bandOffsetAt(idx, offsetFromHead / bodyLen), 0, geom.fullLen);
        return f4(s / geom.fullLen);
      }).join(';');

      if (offsetFromHead > 1) {
        const minVis = startHole ? geom.preLen - 2 : 0.5;
        const visVals = sb.map((k, idx) => (sHeadAt(k) - bandOffsetAt(idx, offsetFromHead / bodyLen) > minVis ? '1' : '0')).join(';');
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

    const washCss = `var(--wash, ${wash})`;
    const wash2Css = `var(--wash2, ${wash2})`;
    const headWashCss = opts.headWash ? opts.headWash : washCss;

    buildBodyRibbons(targetG, washCss, BELLY, shadeCol, 0, false);
    if (spec.pair) {
      buildBodyRibbons(targetG, wash2Css, BELLY, PALETTE.wineDeep, 0.08, false);
    }

    // Calligraphic tapering tail curl flourish right before vanishing into hole
    if (!spec.twoHeaded && opts.tailFlourish !== false) {
      const tailWrap = el('g', {});
      const tailWag = el('g', {}, tailWrap);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;10;-10;8;0;26;-28;20;0;0',
        keyTimes: '0;.18;.34;.50;.66;.74;.79;.84;.88;1',
        calcMode: 'spline', keySplines: buildKeySplines(10),
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, tailWag);
      const tScale = el('g', { transform: `scale(${f2(0.75 * scale)})` }, tailWag);
      el('path', {
        d: 'M0,0 C-4,-1 -8,-5 -6,-9 C-4,-11 -1,-9 -2,-6 C-4,-5 -7,-1 -9,1',
        fill: 'none', stroke: INK, 'stroke-width': '2.6', 'stroke-linecap': 'round'
      }, tScale);
      el('path', {
        d: 'M0,0 C-4,-1 -8,-5 -6,-9 C-4,-11 -1,-9 -2,-6 C-4,-5 -7,-1 -9,1',
        fill: 'none', stroke: washCss, 'stroke-width': '1.3', 'stroke-linecap': 'round'
      }, tScale);
      attachAtBodyOffset(targetG, bodyLen * 0.94, tailWrap, false);
    }

    if (spec.legs > 0) {
      const legFracs = spec.legs === 6 ? [0.24, 0.38, 0.52] : [0.30, 0.46];
      legFracs.forEach((lf, i) => {
        const hips = [];
        const knees = [];
        sb.forEach((k, idx) => {
          const prev = sb[Math.max(0, idx - 1)];
          const moving = idx > 0 && Math.abs(k.u - prev.u) > 0.015;
          if (!moving) {
            hips.push('0'); knees.push('-8');
          } else {
            const phase = (idx + i) % 2 === 0 ? 1 : -1;
            hips.push(String(phase * 19));
            knees.push(phase > 0 ? '-22' : '8');
          }
        });
        const legNode = buildMiniLeg(washCss, BELLY, 0.54 * scale, keyTimesStr, {
          hip: hips.join(';'), knee: knees.join(';')
        }, dur);
        attachAtBodyOffset(targetG, lf * bodyLen, legNode, false);
      });
    }

    if (spec.wings) {
      const flaps = sb.map((k, idx) => {
        const prev = sb[Math.max(0, idx - 1)];
        const moving = idx > 0 && Math.abs(k.u - prev.u) > 0.015;
        if (!moving) return '-8';
        return idx % 2 === 0 ? '-28' : '10';
      }).join(';');
      const wingHolder = el('g', {});
      buildTexturedWing(wingHolder, {
        fill: wash2Css,
        covert: washCss
      }, 0.65 * scale, keyTimesStr, flaps, dur);
      attachAtBodyOffset(targetG, 0.28 * bodyLen, wingHolder, false);
    }

    // Cleaned traced head: alert glance + courtly bow before diving into letter
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
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;0;-14;14;-8;0;0;-10;26;24;0;0',
        keyTimes: '0;.17;.20;.225;.25;.28;.54;.565;.60;.63;.66;1',
        calcMode: 'spline',
        keySplines: buildKeySplines(12),
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, lookG);
    }
    const scaledHead = el('g', { transform: `scale(${f2(headScale)})` }, lookG);
    scaledHead.appendChild(buildHead(headType, {
      wash: headWashCss, wash2: wash2Css, dur,
      woodcut: false,
      baseHead: spec.head,
      headVariant: opts.headVariant,
      flicks: opts.flicks || [{ t0: 0.20, t1: 0.26 }, { t0: 0.56, t1: 0.62 }]
    }));
    attachAtBodyOffset(targetG, 0, headWrap, false);

    if (spec.twoHeaded) {
      const rearWrap = el('g', { transform: `scale(${f2(headScale * 0.92)})` });
      rearWrap.appendChild(buildHead(headType, {
        wash: wash2Css, wash2: washCss, dur,
        woodcut: false,
        baseHead: spec.head,
        headVariant: opts.headVariant,
        flicks: [{ t0: 0.34, t1: 0.42 }]
      }));
      attachAtBodyOffset(targetG, bodyLen * 0.85, rearWrap, true);
    }

    rootG._crawlerMeta = {
      visLen: geom.visLen,
      fullLen: safeFullLen,
      bodyLen,
      maxHalfExtent,
      maskDisc: geom.maskDisc,
      startMaskDisc: geom.startMaskDisc,
      storyboard: sb,
      dur
    };
    return rootG;
  }

  /* ------------------------------------------------------------------ *
   * 2. makeMoonwalker(bubbleEl, opts) -> SVGSVGElement
   * Direct homage to 3-dragon-border-drollery.jpg:
   * - Cinnabar-vermilion head & sweeping S-neck (#c84416) richly stroked
   *   with fine parallel golden-ochre brushwork (#e6c068)
   * - Transitioning cleanly into a smoky olive-charcoal finely ribbed
   *   haunch, clawed legs, small tucked ribbed winglet, and curling whip
   *   tail (#3e453b) with fine pale-ochre rib-lines & woodcut scale ticks
   * - Dignified border-hugging moonwalk circuit (sliding claw, heel roll,
   *   poised corner pause with sideways glance & forked tongue flick)
   * ------------------------------------------------------------------ */
  function makeMoonwalker(bubbleEl, opts) {
    opts = opts || {};
    const spec = CREATURES[opts.creature] || CREATURES['draco-bipes'];
    const scale = opts.scale !== undefined ? opts.scale : 0.94;
    const bodyCol = opts.wash || PALETTE.charcoal;
    const headCol = opts.headWash || PALETTE.cinnabar;
    const ribCol = opts.wash2 || PALETTE.goldLight;
    const dur = opts.dur || 14;
    const headType = opts.headVariant || spec.head;

    const { isDom, w, h, r } = measureBubble(bubbleEl, opts, 240, 68);
    const pad = Math.round(36 * scale);
    const svgW = Math.ceil(w + pad * 2);
    const svgH = Math.ceil(h + pad * 2);
    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-moonwalker bestiary-rig',
      'data-rig': 'moonwalker',
      'data-dur': f1(dur),
      style: `position:absolute;left:${-pad}px;top:${-pad}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    ensureSharedDefs(svg);

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

    const topL = Math.max(1, w - 2 * cr);
    const sideL = Math.max(1, h - 2 * cr);
    const arcL = Math.PI * cr * 0.5;
    const perim = 2 * topL + 2 * sideL + 4 * arcL;
    const pTR = (topL + arcL * 0.35) / perim;
    const pBR = (topL + arcL + sideL + arcL * 0.5) / perim;
    const pBL = (2 * topL + 2 * arcL + sideL + arcL * 0.45) / perim;

    const kt = [0, 0.22, 0.38, 0.52, 0.72, 0.78, 1.0];
    const kp = [0, pTR, pTR, pBR, pBL, pBL, 1.0];

    const walker = el('g', { class: 'bestiary-moonwalker-actor' }, svg);
    const am = el('animateMotion', {
      dur: `${f1(dur)}s`, repeatCount: 'indefinite', rotate: 'auto',
      calcMode: 'spline',
      keyTimes: kt.map(f4).join(';'),
      keyPoints: kp.map(f4).join(';'),
      keySplines: '.38 0 .62 1;0 0 1 1;.35 0 .65 1;.38 0 .62 1;0 0 1 1;.35 0 .65 1'
    }, walker);
    el('mpath', { href: `#${pid}` }, am);

    const stepSamples = 28;
    const legKT = [];
    const bobVals = [];
    const backSlideVals = [], frontSlideVals = [];
    const backHipVals = [], frontHipVals = [];
    const backFootVals = [], frontFootVals = [];

    for (let i = 0; i <= stepSamples; i += 1) {
      const t = i / stepSamples;
      legKT.push(f4(t));
      const inTopPause = t >= 0.22 && t <= 0.38;
      const inBotPause = t >= 0.72 && t <= 0.78;
      if (inTopPause) {
        const pMid = (t - 0.22) / 0.16;
        const lift = Math.sin(pMid * Math.PI);
        bobVals.push(`0 ${f1(-1.5 * lift)}`);
        backSlideVals.push('-2.5 0');
        frontSlideVals.push('2.5 0');
        backHipVals.push('10');
        frontHipVals.push('-14');
        backFootVals.push('-8');
        frontFootVals.push('26');
      } else if (inBotPause) {
        bobVals.push('0 -0.7');
        backSlideVals.push('-1.5 0');
        frontSlideVals.push('1.5 0');
        backHipVals.push('8');
        frontHipVals.push('-8');
        backFootVals.push('-4');
        frontFootVals.push('14');
      } else {
        const ph = i % 4;
        bobVals.push(ph % 2 === 1 ? '0 -1.3' : '0 0');
        if (ph === 0) {
          backSlideVals.push('-4.2 0'); frontSlideVals.push('4.2 0');
          backHipVals.push('20'); frontHipVals.push('-18');
          backFootVals.push('-16'); frontFootVals.push('24');
        } else if (ph === 1) {
          backSlideVals.push('0 0'); frontSlideVals.push('1.0 -1.4');
          backHipVals.push('2'); frontHipVals.push('-6');
          backFootVals.push('0'); frontFootVals.push('32');
        } else if (ph === 2) {
          backSlideVals.push('4.2 0'); frontSlideVals.push('-4.2 0');
          backHipVals.push('-18'); frontHipVals.push('20');
          backFootVals.push('24'); frontFootVals.push('-16');
        } else {
          backSlideVals.push('1.0 -1.4'); frontSlideVals.push('0 0');
          backHipVals.push('-6'); frontHipVals.push('2');
          backFootVals.push('32'); frontFootVals.push('0');
        }
      }
    }
    const legKTStr = legKT.join(';');

    const standG = el('g', { transform: `scale(${f2(scale)})` }, walker);
    const bobG = el('g', {}, standG);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: bobVals.join(';'),
      keyTimes: legKTStr, calcMode: 'linear',
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, bobG);

    function buildMoonLeg(isBack) {
      const lg = el('g', { transform: isBack ? 'translate(-4.5,-11.5)' : 'translate(5.5,-11.5)' });
      const slide = el('g', {}, lg);
      el('animateTransform', {
        attributeName: 'transform', type: 'translate',
        values: (isBack ? backSlideVals : frontSlideVals).join(';'),
        keyTimes: legKTStr, calcMode: 'linear',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, slide);
      const hip = el('g', {}, slide);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: (isBack ? backHipVals : frontHipVals).join(';'),
        keyTimes: legKTStr, calcMode: 'linear',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, hip);
      el('path', {
        d: 'M-2.4,-1.0 C-3.0,2.2 -2.2,5.4 -0.4,7.2 C1.6,7.6 3.0,6.0 3.2,3.2 C3.2,0.8 2.0,-1.2 0,-1.4 Z',
        fill: bodyCol, stroke: INK, 'stroke-width': '1.35', 'stroke-linejoin': 'round'
      }, hip);
      // Fine parallel thigh brushwork (3-dragon-border-drollery.jpg!)
      el('path', {
        d: 'M-1.4,0.8 L1.8,1.8 M-1.6,2.8 L1.6,3.8 M-1.0,4.8 L1.2,5.6',
        fill: 'none', stroke: PALETTE.charcoalLight, 'stroke-width': '0.62', 'stroke-linecap': 'round'
      }, hip);
      const ankle = el('g', { transform: 'translate(0,6.8)' }, hip);
      const foot = el('g', {}, ankle);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: (isBack ? backFootVals : frontFootVals).join(';'),
        keyTimes: legKTStr, calcMode: 'linear',
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, foot);
      el('path', {
        d: 'M-1.2,-0.8 L0.4,4.2 L2.0,3.8 L1.0,-0.8 Z',
        fill: bodyCol, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
      }, foot);
      // Crisp 3-toed gripping claw planted on y=0
      el('path', {
        d: 'M-5.2,4.4 L3.0,4.4 L1.2,2.8 L-2.6,3.0 Z',
        fill: INK, stroke: INK, 'stroke-width': '0.75', 'stroke-linejoin': 'round'
      }, foot);
      return lg;
    }

    bobG.appendChild(buildMoonLeg(true));

    const torso = el('g', {}, bobG);
    const tailPiv = el('g', { transform: 'translate(12,-12)' }, torso);
    const tailWag = el('g', {}, tailPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;9;-7;11;14;-7;5;0',
      keyTimes: '0;.18;.28;.36;.52;.72;.86;1',
      calcMode: 'spline', keySplines: buildKeySplines(8),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tailWag);
    // Knackering-style procedural tail: bold dark outline, belly band, var(--wash) core, dot rail
    el('path', {
      d: 'M0,-2 C8,0 17,-2 24,-8 C27,-11 25,-15 21,-12',
      fill: 'none', stroke: INK, 'stroke-width': '6.4', 'stroke-linecap': 'round'
    }, tailWag);
    el('path', {
      d: 'M0,-2 C8,0 17,-2 24,-8 C27,-11 25,-15 21,-12',
      fill: 'none', stroke: BELLY, 'stroke-width': '4.4', 'stroke-linecap': 'round'
    }, tailWag);
    el('path', {
      d: 'M0,-2 C8,0 17,-2 24,-8 C27,-11 25,-15 21,-12',
      fill: 'none', stroke: bodyCol, 'stroke-width': '3.0', 'stroke-linecap': 'round'
    }, tailWag);
    el('path', {
      d: 'M2,-2 C9,0 16,-2 22,-7',
      fill: 'none', stroke: LEAD, 'stroke-width': '1.05', 'stroke-linecap': 'round', 'stroke-dasharray': '0.4 3.6'
    }, tailWag);

    // Knackering-style torso tube: bold dark outline, lighter belly band, saturated var(--wash) core, rib bands & lead-white dot rail
    el('path', {
      d: 'M-13,-14 C-8,-20 5,-20 13,-14 C16,-11 14,-7 9,-6 C1,-5 -8,-6 -13,-10 Z',
      fill: bodyCol, stroke: INK, 'stroke-width': '1.8', 'stroke-linejoin': 'round'
    }, torso);
    // Lighter belly band along lower torso
    el('path', {
      d: 'M-11,-9.5 C-4,-6.8 5,-6.6 12,-9.5 C8,-6.2 -2,-5.8 -11,-9.5 Z',
      fill: BELLY, stroke: INK, 'stroke-width': '0.85', 'stroke-linejoin': 'round'
    }, torso);
    // Crisp rib bands
    el('path', {
      d: 'M-6,-17 L-5,-8.5 M-2,-18 L-1,-8 M2,-18 L3,-8 M6,-17 L7,-8.5 M10,-15 L10,-9.5',
      fill: 'none', stroke: INK, 'stroke-width': '0.95', 'stroke-linecap': 'round', opacity: '0.42'
    }, torso);
    // Lead-white dot rail along dorsal ridge
    el('path', {
      d: 'M-9,-16.5 C-3,-18.8 4,-18.6 10,-15.2',
      fill: 'none', stroke: LEAD, 'stroke-width': '1.25', 'stroke-linecap': 'round', 'stroke-dasharray': '0.4 3.4'
    }, torso);

    // Articulated traced wing mounted at anatomical shoulder joint (beats on stride & lifts on pause)
    const wingPiv = el('g', { transform: 'translate(-2,-17) scale(-0.78,0.78)' }, torso);
    buildTexturedWing(wingPiv, {
      fill: headCol,
      covert: bodyCol
    }, 1.0, '0;.12;.22;.30;.38;.52;.64;.75;.88;1', '-6;-22;-10;-30;-12;-24;-8;-26;-12;-6', dur);

    // Fore leg
    bobG.appendChild(buildMoonLeg(false));

    // Knackering-style sweeping S-neck (bold dark outline, belly band, saturated core, dot rail)
    const neckG = el('g', { transform: 'translate(-11,-13)' }, bobG);
    el('path', {
      d: 'M0,0 C-6,-2 -8,-8 -5,-14 C-3,-18 -1,-20 -3,-22',
      fill: 'none', stroke: INK, 'stroke-width': '7.4', 'stroke-linecap': 'round'
    }, neckG);
    el('path', {
      d: 'M0,0 C-6,-2 -8,-8 -5,-14 C-3,-18 -1,-20 -3,-22',
      fill: 'none', stroke: BELLY, 'stroke-width': '5.2', 'stroke-linecap': 'round'
    }, neckG);
    el('path', {
      d: 'M0,0 C-6,-2 -8,-8 -5,-14 C-3,-18 -1,-20 -3,-22',
      fill: 'none', stroke: headCol, 'stroke-width': '3.8', 'stroke-linecap': 'round'
    }, neckG);
    el('path', {
      d: 'M-1,-1 C-5.5,-3 -7,-8.5 -4.5,-14 C-3,-17.5 -1.8,-19.2 -2.8,-21',
      fill: 'none', stroke: LEAD, 'stroke-width': '1.1', 'stroke-linecap': 'round', 'stroke-dasharray': '0.35 3.6'
    }, neckG);

    // Cleaned border-drollery head facing -x (glancing sideways at corner pause)
    const headPiv = el('g', { transform: 'translate(-3,-22) scale(-0.64,0.64)' }, neckG);
    const headTurn = el('g', {}, headPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;0;-16;14;-12;10;0;-10;0;-14;10;0;0',
      keyTimes: '0;.21;.245;.28;.315;.35;.39;.52;.62;.73;.76;.80;1',
      calcMode: 'spline', keySplines: buildKeySplines(13),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, headTurn);
    headTurn.appendChild(buildHead(headType, {
      wash: headCol, wash2: bodyCol, dur,
      baseHead: spec.head,
      headVariant: opts.headVariant,
      brushCol: ribCol,
      flicks: [{ t0: 0.25, t1: 0.35 }, { t0: 0.73, t1: 0.78 }]
    }));

    svg._rigMeta = { dur, kind: 'moonwalker', w, h, pad };
    return attachOverlay(bubbleEl, svg, isDom, opts);
  }

  /* ------------------------------------------------------------------ *
   * 3. makeFlightLoop(bubbleEl, opts) -> SVGSVGElement
   * Winged dragons & staggered 3-dragon swarm mixing rich textured
   * mineral pigments (2-winged-dragon-initial-interlace.jpg,
   * 1-coiled-dragon-roundel.jpg) and monochrome woodcut (DE DRACONE):
   * - Finely ribbed medieval pinion wings, multi-band gouache torso
   *   with tiny lead-white seam dots, earnest swept-ear beast heads
   * - Figure-eight swoop launching from bubble corner, apex banking
   *   glance, and controlled touchdown back on the bubble!
   * ------------------------------------------------------------------ */
  function makeFlightLoop(bubbleEl, opts) {
    opts = opts || {};
    const count = Math.max(1, opts.count || 3);
    const scale = opts.scale !== undefined ? opts.scale : 0.88;
    const dur = opts.dur || 10.5;
    const rawWash = opts.wash || PALETTE.gold;
    const rawWash2 = opts.wash2 || '#ff6a00';
    const washCss = `var(--wash, ${rawWash})`;
    const wash2Css = `var(--wash2, ${rawWash2})`;

    const { isDom, w, h } = measureBubble(bubbleEl, opts, 260, 74);
    const padX = Math.round(Math.min(26, Math.max(18, w * 0.09)));
    const padY = Math.round(Math.max(54, 62 * scale));
    const svgW = Math.ceil(w + padX * 2);
    const svgH = Math.ceil(h + padY * 2);

    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-flight bestiary-rig',
      'data-rig': 'viper-swarm',
      'data-dur': f1(dur),
      style: `position:absolute;left:${-padX}px;top:${-padY}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    applyWashVars(svg, rawWash, rawWash2);
    ensureSharedDefs(svg);
    const defs = el('defs', {}, svg);

    const bx0 = padX, by0 = padY, bx1 = padX + w, by1 = padY + h;
    const cx = padX + w * 0.5;
    const cy = padY + h * 0.5;
    const rx = Math.max(10, padX - 6);
    const ryTop = Math.max(36, padY - 14);
    const ryBot = Math.max(32, padY - 18);

    // Orbit + dart path touching the bubble's top-right edge on launch/landing
    const flightPathD = [
      `M${f1(bx1 - 14)},${f1(by0)}`,
      `C${f1(bx1 + rx * 0.85)},${f1(by0 - ryTop * 0.55)} ${f1(cx + w * 0.28)},${f1(by0 - ryTop)} ${f1(cx)},${f1(by0 - ryTop * 0.88)}`,
      `C${f1(cx - w * 0.34)},${f1(by0 - ryTop * 0.80)} ${f1(bx0 - rx * 0.75)},${f1(by0 - 4)} ${f1(bx0 - rx * 0.48)},${f1(cy)}`,
      `C${f1(bx0 - rx * 0.25)},${f1(by1 + ryBot * 0.6)} ${f1(cx - w * 0.24)},${f1(by1 + ryBot)} ${f1(cx + w * 0.06)},${f1(by1 + ryBot * 0.72)}`,
      `C${f1(cx + w * 0.38)},${f1(by1 + ryBot * 0.45)} ${f1(bx1 + rx * 0.8)},${f1(cy + 12)} ${f1(bx1 + rx * 0.5)},${f1(by0 + 10)}`,
      `C${f1(bx1 + rx * 0.25)},${f1(by0 - 14)} ${f1(bx1 - 2)},${f1(by0 - 10)} ${f1(bx1 - 14)},${f1(by0)}`
    ].join(' ');

    const pid = uid('swarm-path');
    el('path', { id: pid, d: flightPathD, fill: 'none' }, defs);

    if (opts.showTrail) {
      el('path', {
        d: flightPathD, fill: 'none', stroke: washCss,
        'stroke-width': '1.25', 'stroke-dasharray': '3 6', opacity: '0.65'
      }, svg);
    }

    function buildSwarmViper(idx) {
      const beginOffset = count > 1 ? -((dur / count) * idx) : 0;
      const beginAttr = beginOffset !== 0 ? `${f2(beginOffset)}s` : '0s';
      const dScale = scale * (idx === 0 ? 0.96 : (idx === 1 ? 0.86 : 0.78));
      const mainCol = idx % 2 === 0 ? washCss : wash2Css;
      const accentCol = idx % 2 === 0 ? wash2Css : washCss;

      // Orbit with quick darts and brief bubble-edge perch
      const kt = [0, 0.06, 0.22, 0.30, 0.52, 0.72, 0.88, 1.0];
      const kp = [0, 0.00, 0.26, 0.30, 0.58, 0.82, 1.00, 1.0];

      const actor = el('g', { class: 'bestiary-flight-actor bestiary-swarm-viper' }, svg);
      const am = el('animateMotion', {
        dur: `${f1(dur)}s`, begin: beginAttr, repeatCount: 'indefinite',
        rotate: 'auto', calcMode: 'spline',
        keyTimes: kt.map(f4).join(';'),
        keyPoints: kp.map(f4).join(';'),
        keySplines: '0 0 1 1;.22 0 .38 1;.42 0 .58 1;.25 0 .45 1;.28 0 .52 1;.30 0 .70 1;0 0 1 1'
      }, actor);
      el('mpath', { href: `#${pid}` }, am);

      // Sinuous dart wiggle perpendicular to flight path
      const dartG = el('g', {}, actor);
      el('animateTransform', {
        attributeName: 'transform', type: 'translate',
        values: '0 -3;0 -3;0 -9;0 -2;0 -10;0 -4;0 -8;0 -3;0 -3',
        keyTimes: '0;.08;.18;.30;.44;.58;.74;.88;1',
        calcMode: 'spline', keySplines: buildKeySplines(9),
        dur: `${f1(dur)}s`, begin: beginAttr, repeatCount: 'indefinite'
      }, dartG);

      const viper = el('g', { transform: `scale(${f2(dScale)})` }, dartG);

      // Knackering-style writhing viper body (bold dark outline, belly band, saturated gold core, rib bands, lead-white dot rail)
      const bodyD = 'M-26,1 C-20,-6 -13,6 -6,-2 C0,-8 6,3 11,-2';
      const bodyWiggle = el('g', {}, viper);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;8;-8;10;-8;6;0',
        keyTimes: '0;.16;.33;.50;.67;.84;1',
        calcMode: 'spline', keySplines: buildKeySplines(7),
        dur: `${f1(dur * 0.45)}s`, begin: beginAttr, repeatCount: 'indefinite'
      }, bodyWiggle);

      el('path', { d: bodyD, fill: 'none', stroke: INK, 'stroke-width': '7.6', 'stroke-linecap': 'round' }, bodyWiggle);
      el('path', { d: bodyD, fill: 'none', stroke: BELLY, 'stroke-width': '5.2', 'stroke-linecap': 'round' }, bodyWiggle);
      el('path', { d: bodyD, fill: 'none', stroke: mainCol, 'stroke-width': '3.8', 'stroke-linecap': 'round' }, bodyWiggle);
      el('path', { d: bodyD, fill: 'none', stroke: INK, 'stroke-width': '3.6', 'stroke-dasharray': '0.9 3.6', opacity: '0.35' }, bodyWiggle);
      el('path', { d: bodyD, fill: 'none', stroke: LEAD, 'stroke-width': '1.15', 'stroke-linecap': 'round', 'stroke-dasharray': '0.4 3.8' }, bodyWiggle);

      // Traced viper head at front joint
      const headPiv = el('g', { transform: 'translate(10,-2) scale(0.56)' }, bodyWiggle);
      const headTurn = el('g', {}, headPiv);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;-10;12;-14;8;-8;0',
        keyTimes: '0;.18;.32;.50;.68;.84;1',
        calcMode: 'spline', keySplines: buildKeySplines(7),
        dur: `${f1(dur)}s`, begin: beginAttr, repeatCount: 'indefinite'
      }, headTurn);
      headTurn.appendChild(buildHead('viper-head', {
        wash: mainCol, wash2: accentCol, dur,
        flicks: [{ t0: 0.24, t1: 0.34 }, { t0: 0.70, t1: 0.78 }]
      }));
    }

    for (let i = 0; i < count; i += 1) {
      buildSwarmViper(i);
    }

    svg._rigMeta = { dur, kind: 'viper-swarm', count, w, h, padX, padY };
    return attachOverlay(bubbleEl, svg, isDom, opts);
  }

  /* ------------------------------------------------------------------ *
   * 4. makeKnottedSlider(bubbleEl, opts) -> SVGSVGElement
   * Direct homage to 2-winged-dragon-initial-interlace.jpg &
   * 1-coiled-dragon-roundel.jpg:
   * - Twin richly modelled interlace ribbons in deep mineral ultramarine
   *   (#244594) and deep Romanesque violet-wine (#5e2250), each painted
   *   with 3-band gouache depth (warm ivory outer highlight + mineral
   *   core + deep underside shadow + tiny lead-white seam dot row!)
   * - Tightens & loosens as both serpents bite at each other's curled
   *   tails, then slithers smoothly along the bubble's bottom edge & back!
   * ------------------------------------------------------------------ */
  function makeKnottedSlider(bubbleEl, opts) {
    opts = opts || {};
    const scale = opts.scale !== undefined ? opts.scale : 0.94;
    const dur = opts.dur || 13;
    const rawWash = opts.wash || PALETTE.lapis;
    const rawWash2 = opts.wash2 || PALETTE.sage;
    const colA = `var(--wash, ${rawWash})`;
    const colB = `var(--wash2, ${rawWash2})`;

    const { isDom, w, h } = measureBubble(bubbleEl, opts, 260, 74);
    const padX = 24;
    const padTop = 12;
    const knotH = Math.round(76 * scale);
    const svgW = Math.ceil(w + padX * 2);
    const svgH = Math.ceil(h + padTop + knotH);

    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-knotted bestiary-rig',
      'data-rig': 'knotted-pair',
      'data-dur': f1(dur),
      style: `position:absolute;left:${-padX}px;top:${-padTop}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    applyWashVars(svg, rawWash, rawWash2);
    ensureSharedDefs(svg);

    const edgeY = padTop + h;
    const leftStopX = padX + Math.min(w * 0.30, 76);
    const rightStopX = padX + Math.max(w * 0.70, w - 76);
    const midX = padX + w * 0.5;

    const slideG = el('g', { class: 'bestiary-knot-slider' }, svg);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: `${f1(leftStopX)} ${f1(edgeY)};${f1(leftStopX)} ${f1(edgeY)};${f1(midX)} ${f1(edgeY + 1.5)};${f1(rightStopX)} ${f1(edgeY)};${f1(rightStopX)} ${f1(edgeY)};${f1(midX)} ${f1(edgeY - 1)};${f1(leftStopX)} ${f1(edgeY)};${f1(leftStopX)} ${f1(edgeY)}`,
      keyTimes: '0;.44;.56;.68;.76;.86;.95;1',
      calcMode: 'spline',
      keySplines: '0 0 1 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1',
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, slideG);

    const cinchG = el('g', {}, slideG);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: `${f2(scale)} ${f2(scale)};${f2(scale * 0.85)} ${f2(scale * 0.88)};${f2(scale * 1.10)} ${f2(scale * 1.05)};${f2(scale * 0.87)} ${f2(scale * 0.89)};${f2(scale * 1.15)} ${f2(scale * 0.92)};${f2(scale * 1.04)} ${f2(scale * 0.96)};${f2(scale * 0.88)} ${f2(scale * 0.91)};${f2(scale * 1.13)} ${f2(scale * 0.92)};${f2(scale)} ${f2(scale)}`,
      keyTimes: '0;.12;.24;.36;.52;.68;.74;.86;1',
      calcMode: 'spline', keySplines: buildKeySplines(9),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, cinchG);

    const swayG = el('g', {}, cinchG);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;-2.5;2.5;-2;4;-3.5;0;-4;3;0',
      keyTimes: '0;.12;.24;.36;.52;.64;.72;.84;.92;1',
      calcMode: 'spline', keySplines: buildKeySplines(10),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, swayG);

    // Knackering-style interlace tube: bold dark outline, lighter belly band, saturated core, rib bands, lead-white dot rail
    function drawTexturedInterlace(parent, d, mainCol) {
      el('path', {
        d, fill: 'none', stroke: INK, 'stroke-width': '11.2',
        'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, parent);
      el('path', {
        d, fill: 'none', stroke: BELLY, 'stroke-width': '8.2',
        'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, parent);
      el('path', {
        d, fill: 'none', stroke: mainCol, 'stroke-width': '6.0',
        'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      }, parent);
      el('path', {
        d, fill: 'none', stroke: INK, 'stroke-width': '5.8',
        'stroke-linecap': 'butt', 'stroke-dasharray': '1.0 4.4', opacity: '0.32'
      }, parent);
      el('path', {
        d, fill: 'none', stroke: LEAD, 'stroke-width': '1.35',
        'stroke-linecap': 'round', 'stroke-dasharray': '0.4 4.2', opacity: '0.96'
      }, parent);
    }

    const underG = el('g', { class: 'knot-under' }, swayG);
    const slideA = el('g', {}, underG);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: '0 0;3 1;-3 -1;2 1;0 0;-2 1;0 0',
      keyTimes: '0;.12;.24;.36;.56;.80;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, slideA);
    drawTexturedInterlace(slideA, 'M-42,10 C-48,26 -30,44 -6,42 C10,40 20,26 24,14', colA);

    const slideB = el('g', {}, underG);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: '0 0;-3 1;3 -1;-2 1;0 0;2 1;0 0',
      keyTimes: '0;.12;.24;.36;.56;.80;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, slideB);
    drawTexturedInterlace(slideB, 'M42,10 C48,26 30,44 6,42 C-10,40 -20,26 -24,14', colB);

    const overG = el('g', { class: 'knot-over' }, swayG);
    drawTexturedInterlace(overG, 'M-26,12 C-16,2 2,2 14,16 C22,26 34,26 42,12', colA);
    drawTexturedInterlace(overG, 'M26,12 C16,2 -2,2 -14,16 C-22,26 -34,26 -42,12', colB);

    const tailLeft = el('g', { transform: 'translate(-44,10)' }, swayG);
    const tailLeftWag = el('g', {}, tailLeft);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;14;-12;16;-8;10;0',
      keyTimes: '0;.14;.28;.44;.64;.82;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tailLeftWag);
    el('path', {
      d: 'M0,0 C-8,-2 -13,-8 -9,-12 C-5,-14 -1,-10 -3,-6',
      fill: 'none', stroke: INK, 'stroke-width': '4.6', 'stroke-linecap': 'round'
    }, tailLeftWag);
    el('path', {
      d: 'M0,0 C-8,-2 -13,-8 -9,-12 C-5,-14 -1,-10 -3,-6',
      fill: 'none', stroke: colA, 'stroke-width': '2.4', 'stroke-linecap': 'round'
    }, tailLeftWag);

    const tailRight = el('g', { transform: 'translate(44,10)' }, swayG);
    const tailRightWag = el('g', {}, tailRight);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '0;-14;12;-16;8;-10;0',
      keyTimes: '0;.14;.28;.44;.64;.82;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, tailRightWag);
    el('path', {
      d: 'M0,0 C8,-2 13,-8 9,-12 C5,-14 1,-10 3,-6',
      fill: 'none', stroke: INK, 'stroke-width': '4.6', 'stroke-linecap': 'round'
    }, tailRightWag);
    el('path', {
      d: 'M0,0 C8,-2 13,-8 9,-12 C5,-14 1,-10 3,-6',
      fill: 'none', stroke: colB, 'stroke-width': '2.4', 'stroke-linecap': 'round'
    }, tailRightWag);

    // Gothic interlace dragon heads (cleaned from traced/components/gothic-initial-interlace/)
    const headRightPiv = el('g', { transform: 'translate(34,8) scale(0.64)' }, swayG);
    const headRightSnap = el('g', {}, headRightPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '-10;8;-14;9;-6;12;-10',
      keyTimes: '0;.12;.24;.36;.60;.74;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, headRightSnap);
    headRightSnap.appendChild(buildHead('gothic-dragon-a', {
      wash: colA, wash2: colB, dur,
      flicks: [{ t0: 0.10, t1: 0.18 }, { t0: 0.34, t1: 0.42 }, { t0: 0.70, t1: 0.77 }]
    }));

    const headLeftPiv = el('g', { transform: 'translate(-34,8) scale(-0.64,0.64)' }, swayG);
    const headLeftSnap = el('g', {}, headLeftPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: '-8;10;-14;6;-12;8;-8',
      keyTimes: '0;.15;.28;.40;.56;.72;1',
      calcMode: 'spline', keySplines: buildKeySplines(7),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, headLeftSnap);
    headLeftSnap.appendChild(buildHead('gothic-dragon-b', {
      wash: colB, wash2: colA, dur,
      flicks: [{ t0: 0.22, t1: 0.30 }, { t0: 0.68, t1: 0.76 }]
    }));

    svg._rigMeta = { dur, kind: 'knotted-pair', w, h, padX, padTop, knotH };
    return attachOverlay(bubbleEl, svg, isDom, opts);
  }

  /* ------------------------------------------------------------------ *
   * 5. makeHeadlineChase(bubbleEl, opts) -> SVGSVGElement
   * Two Claude-ruffed serpents in rich mineral cinnabar-orange (#d95224 /
   * #c84416) with multi-band gouache modelling & tiny lead-white seam dots
   * chasing round the headline bubble, snapping & recoiling at bottom center
   * while leaving the gold caption below 100% uncovered at desktop & 390px!
   * ------------------------------------------------------------------ */
  function makeHeadlineChase(bubbleEl, opts) {
    opts = opts || {};
    const scale = opts.scale !== undefined ? opts.scale : 0.86;
    const dur = opts.dur || 11;
    const rawWash = opts.wash || PALETTE.claude;
    const rawWash2 = opts.wash2 || PALETTE.gold;
    const colA = `var(--wash, ${rawWash})`;
    const colB = `var(--wash2, ${rawWash2})`;

    const { isDom, w, h, r } = measureBubble(bubbleEl, opts, 300, 74);
    const padX = Math.round(24 * scale);
    const padTop = Math.round(24 * scale);
    const padBot = Math.round(16 * scale);
    const svgW = Math.ceil(w + padX * 2);
    const svgH = Math.ceil(h + padTop + padBot);

    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-headline-chase bestiary-rig',
      'data-rig': 'headline-chase',
      'data-dur': f1(dur),
      style: `position:absolute;left:${-padX}px;top:${-padTop}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    applyWashVars(svg, rawWash, rawWash2);
    ensureSharedDefs(svg);

    const cr = Math.min(r, w * 0.42, h * 0.42);
    const x0 = padX, y0 = padTop, x1 = padX + w, y1 = padTop + h;
    const midX = x0 + w * 0.5;
    const meetGap = Math.max(18 * scale, 14);

    const leftTrackD = [
      `M${f1(midX - 8)},${f1(y0)}`,
      `L${f1(x0 + cr)},${f1(y0)}`,
      `A${f1(cr)},${f1(cr)} 0 0 0 ${f1(x0)},${f1(y0 + cr)}`,
      `L${f1(x0)},${f1(y1 - cr)}`,
      `A${f1(cr)},${f1(cr)} 0 0 0 ${f1(x0 + cr)},${f1(y1)}`,
      `L${f1(midX - meetGap)},${f1(y1)}`
    ].join(' ');

    const rightTrackD = [
      `M${f1(midX + 8)},${f1(y0)}`,
      `L${f1(x1 - cr)},${f1(y0)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x1)},${f1(y0 + cr)}`,
      `L${f1(x1)},${f1(y1 - cr)}`,
      `A${f1(cr)},${f1(cr)} 0 0 1 ${f1(x1 - cr)},${f1(y1)}`,
      `L${f1(midX + meetGap)},${f1(y1)}`
    ].join(' ');

    const defs = el('defs', {}, svg);
    const idL = uid('chase-left');
    const idR = uid('chase-right');
    el('path', { id: idL, d: leftTrackD, fill: 'none' }, defs);
    el('path', { id: idR, d: rightTrackD, fill: 'none' }, defs);

    const lenL = measurePath(leftTrackD, 48).total;
    const lenR = measurePath(rightTrackD, 48).total;

    const kt = [0, 0.22, 0.40, 0.52, 0.56, 0.66, 0.75, 0.94, 1.0];
    const uVals = [0.22, 0.56, 0.84, 0.99, 0.99, 0.76, 0.76, 0.22, 0.22];
    const ktStr = kt.map(f4).join(';');
    const ksStr = buildKeySplines(kt.length);

    function buildChaser(pathId, pathD, trackLen, color, accentColor, headKind, isRight) {
      const bodyLen = Math.min(trackLen * 0.44, 72 * scale);
      const gap = Math.ceil(trackLen + bodyLen + 300);
      const g = el('g', { class: isRight ? 'chaser-right' : 'chaser-left' }, svg);

      const segs = 6;
      // 5 passes: bold dark outline -> lighter belly band -> saturated core -> rib bands -> lead-white dot rail
      for (let pass = 0; pass < 5; pass += 1) {
        for (let i = 0; i < segs; i += 1) {
          const f0 = i / segs;
          const f1b = Math.min(1, (i + 1) / segs + 0.04);
          const uMid = (f0 + f1b) * 0.5;
          const baseW = (8.8 * (1 - 0.60 * Math.pow(uMid, 1.15))) * scale;
          const segLen = Math.max(2, (f1b - f0) * bodyLen);
          const period = segLen + gap;
          const vals = uVals.map((u) => {
            const sHead = clamp(u * trackLen, bodyLen * 0.35, trackLen);
            const s0 = Math.max(0, sHead - f1b * bodyLen);
            return f1(period - s0);
          }).join(';');

          let strokeCol = color;
          let strokeW = baseW * 0.74;
          let dash = `${f1(segLen)} ${gap}`;
          let cap = 'round';
          let op = '1';
          if (pass === 0) {
            strokeCol = INK;
            strokeW = baseW + 2.2 * scale;
          } else if (pass === 1) {
            strokeCol = BELLY;
            strokeW = baseW;
          } else if (pass === 3) {
            strokeCol = INK;
            strokeW = baseW * 0.72;
            cap = 'butt';
            op = '0.32';
            const rStep = Math.max(3.0, 3.8 * scale);
            dash = `0.85 ${f1(rStep - 0.85)} `.repeat(Math.max(1, Math.floor(segLen / rStep))) + `0.01 ${gap}`;
          } else if (pass === 4) {
            strokeCol = LEAD;
            strokeW = Math.max(0.95, baseW * 0.20);
            dash = `0.38 ${f1(Math.max(3.6, 4.8 * scale))} `.repeat(Math.max(1, Math.floor(segLen / (4.8 * scale)))) + `0.01 ${gap}`;
          }

          const p = el('path', {
            d: pathD, fill: 'none', stroke: strokeCol,
            'stroke-width': f2(strokeW), 'stroke-linecap': cap,
            'stroke-dasharray': dash,
            'stroke-dashoffset': vals.split(';')[0],
            opacity: op
          }, g);
          el('animate', {
            attributeName: 'stroke-dashoffset',
            values: vals, keyTimes: ktStr,
            calcMode: 'spline', keySplines: ksStr,
            dur: `${f1(dur)}s`, repeatCount: 'indefinite'
          }, p);
        }
      }

      const headHolder = el('g', {}, g);
      const kpStr = uVals.map((u) => f4(clamp(u, 0.05, 1.0))).join(';');
      const am = el('animateMotion', {
        dur: `${f1(dur)}s`, repeatCount: 'indefinite', rotate: 'auto',
        calcMode: 'spline', keyTimes: ktStr, keyPoints: kpStr, keySplines: ksStr
      }, headHolder);
      el('mpath', { href: `#${pathId}` }, am);

      const orientG = el('g', { transform: `scale(${f2(0.64 * scale)},${f2((isRight ? -0.64 : 0.64) * scale)})` }, headHolder);
      const snapTurn = el('g', {}, orientG);
      el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0;0;-8;14;-20;10;-10;0;0',
        keyTimes: '0;.40;.50;.54;.60;.68;.73;.84;1',
        calcMode: 'spline', keySplines: buildKeySplines(9),
        dur: `${f1(dur)}s`, repeatCount: 'indefinite'
      }, snapTurn);
      snapTurn.appendChild(buildHead(headKind, {
        wash: color, wash2: accentColor, dur,
        headVariant: opts.ruff === false ? headKind : 'claude-ruffed',
        baseHead: headKind,
        flicks: [{ t0: 0.49, t1: 0.58 }, { t0: 0.69, t1: 0.74 }]
      }));
    }

    buildChaser(idL, leftTrackD, lenL, colA, colB, opts.leftHead || 'romanesque-s-horned', false);
    buildChaser(idR, rightTrackD, lenR, colA, colB, opts.rightHead || 'roundel-head', true);

    const sparkG = el('g', { transform: `translate(${f1(midX)},${f1(y1)})`, opacity: '0' }, svg);
    el('animate', {
      attributeName: 'opacity',
      values: '0;0;1;1;0;0',
      keyTimes: '0;.50;.52;.57;.60;1',
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, sparkG);
    const sparkScale = el('g', {}, sparkG);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: '0.4;0.4;1.2;0.85;0.4;0.4',
      keyTimes: '0;.50;.53;.58;.61;1',
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, sparkScale);
    addManuscriptStar(sparkScale, 0, 0, 6.4 * scale, PALETTE.goldLight);

    // Antique mineral verdigris manuscript fleurons at top corners
    const fScale = 0.76 * scale;
    [
      { x: x0 + 6, y: y0 - 2, deg: -35 },
      { x: x1 - 6, y: y0 - 2, deg: 215 }
    ].forEach((fl) => {
      const fg = el('g', { transform: `translate(${f1(fl.x)},${f1(fl.y)}) rotate(${fl.deg}) scale(${f2(fScale)})` }, svg);
      el('path', {
        d: 'M-8,0 C-4,-5 2,-5 7,-2 C9,-4 12,-3 12,0 C12,3 9,4 7,2 C2,5 -4,5 -8,0 Z',
        fill: PALETTE.fleuronGreen, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
      }, fg);
      el('path', {
        d: 'M-5,0 C-1,-2 3,-1 7,0',
        fill: 'none', stroke: LEAD, 'stroke-width': '0.65', 'stroke-linecap': 'round', opacity: '0.85'
      }, fg);
      el('circle', { cx: '13', cy: '0', r: '1.6', fill: PALETTE.lapis, stroke: INK, 'stroke-width': '0.85' }, fg);
    });

    svg._rigMeta = { dur, kind: 'headline-chase', w, h, padX, padTop, padBot };
    return attachOverlay(bubbleEl, svg, isDom, opts);
  }

  /* ------------------------------------------------------------------ *
   * 6. makeYuckBounce(bubbleEl, opts) -> SVGSVGElement
   * Deliberate cute odd-one-out: small, round, endearing wind-up creature
   * in warm antique vermilion & parchment bouncing round red blackletter "yuck"!
   * ------------------------------------------------------------------ */
  function makeYuckBounce(bubbleEl, opts) {
    opts = opts || {};
    const scale = opts.scale !== undefined ? opts.scale : 0.90;
    const dur = opts.dur || 8.5;
    const rawWash = opts.wash || PALETTE.vermilion;
    const rawWash2 = opts.wash2 || PALETTE.gold;
    const wash = `var(--wash, ${rawWash})`;
    const wash2 = `var(--wash2, ${rawWash2})`;

    const { isDom, w, h, r } = measureBubble(bubbleEl, opts, 136, 58);
    const pad = Math.round(30 * scale);
    const svgW = Math.ceil(w + pad * 2);
    const svgH = Math.ceil(h + pad * 2);

    const svg = el('svg', {
      xmlns: SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-yuck-bounce bestiary-rig',
      'data-rig': 'yuck-bounce',
      'data-dur': f1(dur),
      style: `position:absolute;left:${-pad}px;top:${-pad}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    applyWashVars(svg, rawWash, rawWash2);
    ensureSharedDefs(svg);

    const cr = Math.min(r, w * 0.46, h * 0.46);
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
    const pid = uid('yuck-loop');
    el('path', { id: pid, d: loopD, fill: 'none' }, defs);

    const hops = 12;
    const hopKT = [];
    const hopKP = [];
    const jumpVals = [];
    const squashVals = [];
    const keyRotVals = [];

    const totalPts = hops * 2 + 4;
    for (let i = 0; i <= totalPts; i += 1) {
      const frac = i / totalPts;
      hopKT.push(f4(frac));
      if (frac <= 0.76) {
        const u = frac / 0.76;
        hopKP.push(f4(u * 0.86));
        const isAir = i % 2 === 1;
        jumpVals.push(isAir ? '0 -11' : '0 0');
        squashVals.push(isAir ? '0.92 1.10' : '1.14 0.86');
        keyRotVals.push(String(Math.round(u * 720)));
      } else if (frac <= 0.90) {
        hopKP.push('0.86');
        jumpVals.push(i % 2 === 0 ? '0 0' : '0 -1.4');
        squashVals.push(i % 2 === 0 ? '1.06 0.94' : '0.96 1.04');
        keyRotVals.push(i % 2 === 0 ? '720' : '695');
      } else {
        const u2 = (frac - 0.90) / 0.10;
        hopKP.push(f4(0.86 + u2 * 0.14));
        const isAir = i % 2 === 1 && frac < 1;
        jumpVals.push(isAir ? '0 -9' : '0 0');
        squashVals.push(isAir ? '0.92 1.10' : '1.10 0.90');
        keyRotVals.push(String(Math.round(720 + u2 * 180)));
      }
    }
    const hopKTStr = hopKT.join(';');

    const actor = el('g', { class: 'bestiary-yuck-actor' }, svg);
    const am = el('animateMotion', {
      dur: `${f1(dur)}s`, repeatCount: 'indefinite', rotate: 'auto',
      calcMode: 'linear',
      keyTimes: hopKTStr,
      keyPoints: hopKP.join(';')
    }, actor);
    el('mpath', { href: `#${pid}` }, am);

    const scaleBase = el('g', { transform: `scale(${f2(scale)})` }, actor);
    const hopG = el('g', {}, scaleBase);
    el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: jumpVals.join(';'), keyTimes: hopKTStr,
      calcMode: 'spline', keySplines: buildKeySplines(hopKT.length),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, hopG);

    const sqG = el('g', {}, hopG);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: squashVals.join(';'), keyTimes: hopKTStr,
      calcMode: 'spline', keySplines: buildKeySplines(hopKT.length),
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, sqG);

    // Wind-up key
    const keyPiv = el('g', { transform: 'translate(-4,-16)' }, sqG);
    const keySpin = el('g', {}, keyPiv);
    el('animateTransform', {
      attributeName: 'transform', type: 'rotate',
      values: keyRotVals.join(';'), keyTimes: hopKTStr,
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, keySpin);
    el('path', {
      d: 'M0,3 L0,-5', fill: 'none', stroke: INK, 'stroke-width': '1.9', 'stroke-linecap': 'round'
    }, keySpin);
    el('circle', { cx: '-3.8', cy: '-7.2', r: '2.7', fill: wash2, stroke: INK, 'stroke-width': '1.3' }, keySpin);
    el('circle', { cx: '3.8', cy: '-7.2', r: '2.7', fill: wash2, stroke: INK, 'stroke-width': '1.3' }, keySpin);
    el('circle', { cx: '-3.8', cy: '-7.2', r: '0.95', fill: LEAD }, keySpin);
    el('circle', { cx: '3.8', cy: '-7.2', r: '0.95', fill: LEAD }, keySpin);

    // Coiled viper spring-tail
    el('path', {
      d: 'M-11,-7 C-16,-7 -18,-12 -14,-14 C-10,-15 -10,-10 -14,-9 C-18,-8 -20,-13 -18,-17',
      fill: 'none', stroke: INK, 'stroke-width': '3.8', 'stroke-linecap': 'round'
    }, sqG);
    el('path', {
      d: 'M-11,-7 C-16,-7 -18,-12 -14,-14 C-10,-15 -10,-10 -14,-9 C-18,-8 -20,-13 -18,-17',
      fill: 'none', stroke: wash2, 'stroke-width': '1.8', 'stroke-linecap': 'round'
    }, sqG);

    // Small, round, cute coiled viper body with bold dark outline, lighter belly band & lead-white dots
    el('path', {
      d: 'M-11,-8 C-11,-15 -4,-18 5,-17 C12,-16 15,-10 14,-5 C13,0 6,0 -2,0 C-8,0 -11,-3 -11,-8 Z',
      fill: wash, stroke: INK, 'stroke-width': '1.7', 'stroke-linejoin': 'round'
    }, sqG);
    // Tiny cute viper horn-tuft
    el('path', {
      d: 'M-2,-17 C-4,-20 -6,-21 -7,-20 C-5,-18 -4,-17 -3,-16 Z',
      fill: wash2, stroke: INK, 'stroke-width': '1.2', 'stroke-linejoin': 'round'
    }, sqG);
    el('ellipse', { cx: '1', cy: '-3.6', rx: '7.4', ry: '3.0', fill: BELLY, stroke: INK, 'stroke-width': '1.0' }, sqG);
    // Lead-white dot rail
    el('path', {
      d: 'M-7,-12 C-3,-15 2,-15 6,-14',
      fill: 'none', stroke: LEAD, 'stroke-width': '1.3', 'stroke-linecap': 'round', 'stroke-dasharray': '0.4 3.4'
    }, sqG);

    // Big glossy cute eye
    el('circle', { cx: '7.6', cy: '-10.2', r: '2.9', fill: INK }, sqG);
    el('circle', { cx: '6.7', cy: '-11.1', r: '1.0', fill: LEAD }, sqG);
    el('circle', { cx: '8.5', cy: '-9.3', r: '0.45', fill: LEAD }, sqG);
    // Cute viper smile + tiny forked blep tongue
    el('path', {
      d: 'M10.2,-6.0 Q12.2,-4.2 14.2,-6.2',
      fill: 'none', stroke: INK, 'stroke-width': '1.25', 'stroke-linecap': 'round'
    }, sqG);
    const blep = el('g', { transform: 'translate(13.2,-5.6)' }, sqG);
    el('path', {
      d: 'M0,0 L4.4,0.8 M3.2,0.5 L4.8,-0.5 M3.2,0.7 L4.6,2.0',
      fill: 'none', stroke: wash2, 'stroke-width': '1.15', 'stroke-linecap': 'round'
    }, blep);
    el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: '.3 1;.3 1;1.1 1;1.1 1;.3 1;.3 1',
      keyTimes: '0;.74;.79;.88;.92;1',
      dur: `${f1(dur)}s`, repeatCount: 'indefinite'
    }, blep);

    svg._rigMeta = { dur, kind: 'yuck-bounce', w, h, pad };
    return attachOverlay(bubbleEl, svg, isDom, opts);
  }

  function drape(bubbleEl, opts) {
    opts = opts || {};
    if (opts.side === 'frame' || opts.headVariant === 'claude-ruffed') {
      return makeHeadlineChase(bubbleEl, opts);
    }
    if (opts.creature === 'cute') {
      return makeYuckBounce(bubbleEl, opts);
    }
    if (opts.creature === 'knotted-pair') {
      return makeKnottedSlider(bubbleEl, opts);
    }
    if (opts.creature === 'draco-alatus' || opts.creature === 'wyvern') {
      return makeFlightLoop(bubbleEl, opts);
    }
    return makeMoonwalker(bubbleEl, opts);
  }

  return {
    PALETTE,
    CREATURES,
    REGISTERS,
    ensureSharedDefs,
    observeVisibility,
    buildHead,
    makeCrawler,
    makeMoonwalker,
    makeFlightLoop,
    makeKnottedSlider,
    makeHeadlineChase,
    makeYuckBounce,
    drape
  };
}));
