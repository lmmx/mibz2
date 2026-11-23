/**
 * Bestiary Traced-Segment Spine Rig (Brief 6)
 * Cuts any potrace-traced medieval manuscript body into N overlapping segments
 * (12-30) pinned at their joints along a crawler.js / SMIL spine path, so the
 * actual traced artwork bends, coils, writhes, and burrows with zero visible
 * seams. Recolourable via --wash and --wash2 (neon, saturated, zero pink),
 * with automatic IntersectionObserver on-screen-only pausing.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) {
    module.exports = factory();
  } else {
    root.BestiaryTracedRig = factory();
  }
}(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const INK = '#19100b';
  const IVORY = '#fff3d6';

  let uidN = 0;
  function uid(prefix) {
    uidN += 1;
    return `${prefix}-${uidN}`;
  }

  function f1(n) { return (Math.round(n * 10) / 10).toString(); }
  function f2(n) { return (Math.round(n * 100) / 100).toString(); }
  function f4(n) { return (Math.round(n * 10000) / 10000).toString(); }
  function clamp(v, a, b) { return v < a ? a : (v > b ? b : v); }

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

  function buildKeySplines(n, sp) {
    const s = sp || '.42 0 .58 1';
    const out = [];
    for (let i = 0; i < Math.max(1, n - 1); i += 1) out.push(s);
    return out.join(';');
  }

  /**
   * Build a <g> of potrace-traced layers recoloured via --wash / --wash2.
   * layers: [{ d, role: 'wash'|'wash2'|'hi'|'ink', opacity? }]
   */
  function buildTracedGroup(layers, opts, parent) {
    opts = opts || {};
    const wash = opts.wash || 'var(--wash, #1b5cff)';
    const wash2 = opts.wash2 || 'var(--wash2, #00d97e)';
    const g = el('g', opts.attrs || {}, parent);
    for (let i = 0; i < layers.length; i += 1) {
      const L = layers[i];
      let fill = INK;
      if (L.role === 'wash') fill = wash;
      else if (L.role === 'wash2') fill = wash2;
      else if (L.role === 'hi') fill = IVORY;
      else if (L.fill) fill = L.fill;
      const attrs = { d: L.d, fill };
      if (L.opacity !== undefined) attrs.opacity = String(L.opacity);
      if (L.role === 'ink') attrs['data-role'] = 'ink';
      el('path', attrs, g);
    }
    return g;
  }

  /**
   * Resample a polyline [[x,y], ...] into N evenly spaced samples with
   * position (x, y) and tangent angle in degrees (pointing from head->tail or tail->head).
   */
  function samplePolyline(pts, count) {
    const cum = [0];
    for (let i = 1; i < pts.length; i += 1) {
      const dx = pts[i][0] - pts[i - 1][0];
      const dy = pts[i][1] - pts[i - 1][1];
      cum.push(cum[i - 1] + Math.hypot(dx, dy));
    }
    const total = cum[cum.length - 1] || 1;
    const out = [];
    for (let k = 0; k < count; k += 1) {
      const target = ((k + 0.5) / count) * total;
      let idx = 0;
      while (idx < cum.length - 2 && cum[idx + 1] < target) idx += 1;
      const segLen = Math.max(0.0001, cum[idx + 1] - cum[idx]);
      const t = clamp((target - cum[idx]) / segLen, 0, 1);
      const x = pts[idx][0] + (pts[idx + 1][0] - pts[idx][0]) * t;
      const y = pts[idx][1] + (pts[idx + 1][1] - pts[idx][1]) * t;
      // Tangent pointing toward head (from tail toward head = direction of travel +x!)
      const i0 = Math.max(0, idx);
      const i1 = Math.min(pts.length - 1, idx + 1);
      const dx = pts[i0][0] - pts[i1][0];
      const dy = pts[i0][1] - pts[i1][1];
      const deg = (Math.atan2(dy, dx) * 180) / Math.PI;
      out.push({ x, y, deg, frac: (k + 0.5) / count });
    }
    return { samples: out, total };
  }

  /**
   * Rig a potrace-traced body as N overlapping segments pinned along a SMIL spine.
   * Each segment clips a generous overlapping pill of the real traced artwork and
   * rides targetPathId via synchronized <animateMotion rotate="auto">.
   */
  function mountSegmentedBody(opts) {
    const defs = opts.defs;
    const parent = opts.parent;
    const numSegs = clamp(opts.numSegments || 16, 12, 30);
    const overlap = opts.overlap !== undefined ? opts.overlap : 0.55;
    const bodyScale = opts.bodyScale || 0.4;
    const srcWidth = opts.srcWidth || 44;
    const dur = opts.dur || 12;
    const begin = opts.begin || '0s';
    const flipY = !!opts.flipY;

    // Store the potrace-traced body ONCE in <defs> so all N segments share it
    const artId = uid('traced-body');
    buildTracedGroup(opts.bodyLayers, {
      wash: opts.wash,
      wash2: opts.wash2,
      attrs: { id: artId }
    }, defs);

    // Sample the traced body's natural centerline (index 0 = neck/head end, last = tail end)
    const { samples, total } = samplePolyline(opts.srcSpine, numSegs);
    const segStep = total / numSegs;
    const pillW = segStep * (1 + 2 * overlap);
    const pillH = srcWidth * 1.9;
    const pillRx = Math.min(pillW * 0.42, pillH * 0.42);

    const bodyLenTarget = opts.bodyLenPx !== undefined ? opts.bodyLenPx : total * bodyScale;
    const spanFrac = clamp(bodyLenTarget / Math.max(1, opts.targetPathLen || 200), 0.05, 0.96);

    const rootG = el('g', { class: opts.className || 'bestiary-traced-segmented-body' }, parent);

    // Build tail-to-head so anterior segments overlap naturally over posterior segments
    for (let i = numSegs - 1; i >= 0; i -= 1) {
      const s = samples[i];
      const clipId = uid('tseg-clip');
      const cp = el('clipPath', { id: clipId, clipPathUnits: 'userSpaceOnUse' }, defs);
      el('rect', {
        x: f1(-pillW * 0.5),
        y: f1(-pillH * 0.5),
        width: f1(pillW),
        height: f1(pillH),
        rx: f1(pillRx),
        ry: f1(pillRx),
        transform: `translate(${f1(s.x)},${f1(s.y)}) rotate(${f1(s.deg)})`
      }, cp);

      const segHolder = el('g', { class: 'traced-seg' }, rootG);
      const kp = opts.headUValues.map((uHead) => {
        const u = uHead - s.frac * spanFrac;
        return f4(clamp(u, 0.001, 0.999));
      }).join(';');

      if (opts.minVisU !== undefined && opts.minVisU !== null) {
        const visVals = opts.headUValues.map((uHead) => ((uHead - s.frac * spanFrac) > opts.minVisU ? '1' : '0')).join(';');
        el('animate', {
          attributeName: 'opacity',
          values: visVals,
          keyTimes: opts.keyTimes,
          calcMode: 'discrete',
          dur: `${f1(dur)}s`,
          begin,
          repeatCount: 'indefinite'
        }, segHolder);
      }

      const am = el('animateMotion', {
        dur: `${f1(dur)}s`,
        begin,
        repeatCount: 'indefinite',
        rotate: 'auto',
        calcMode: 'spline',
        keyTimes: opts.keyTimes,
        keyPoints: kp,
        keySplines: opts.keySplines
      }, segHolder);
      el('mpath', { href: `#${opts.targetPathId}` }, am);

      const sy = flipY ? -bodyScale : bodyScale;
      // Align segment tangent to +x, origin at (s.x, s.y), clip in source coordinates
      const aligned = el('g', {
        transform: `scale(${f4(bodyScale)},${f4(sy)}) rotate(${f1(-s.deg)}) translate(${f1(-s.x)},${f1(-s.y)})`,
        'clip-path': `url(#${clipId})`
      }, segHolder);
      el('use', { href: `#${artId}` }, aligned);
    }

    return { rootG, spanFrac, bodyLenTarget, srcTotalLen: total };
  }

  /**
   * Pin a traced anatomical part (head, wing, leg/foot, tail tip) onto the same
   * spine at jointFrac (0 = head, 1 = tail), with optional local rotation animation.
   */
  function mountJointPart(opts) {
    const holder = el('g', { class: opts.className || 'traced-joint-part' }, opts.parent);
    const spanFrac = opts.spanFrac || 0.4;
    const jointFrac = opts.jointFrac || 0;
    const dur = opts.dur || 12;
    const begin = opts.begin || '0s';

    const kp = opts.headUValues.map((uHead) => f4(clamp(uHead - jointFrac * spanFrac, 0.001, 0.999))).join(';');
    if (opts.minVisU !== undefined && opts.minVisU !== null) {
      const visVals = opts.headUValues.map((uHead) => ((uHead - jointFrac * spanFrac) > opts.minVisU ? '1' : '0')).join(';');
      el('animate', {
        attributeName: 'opacity',
        values: visVals,
        keyTimes: opts.keyTimes,
        calcMode: 'discrete',
        dur: `${f1(dur)}s`,
        begin,
        repeatCount: 'indefinite'
      }, holder);
    }

    const am = el('animateMotion', {
      dur: `${f1(dur)}s`,
      begin,
      repeatCount: 'indefinite',
      rotate: opts.rotateMode || 'auto',
      calcMode: 'spline',
      keyTimes: opts.keyTimes,
      keyPoints: kp,
      keySplines: opts.keySplines
    }, holder);
    el('mpath', { href: `#${opts.targetPathId}` }, am);

    let target = holder;
    if (opts.localRotate) {
      const rotG = el('g', {}, holder);
      el('animateTransform', {
        attributeName: 'transform',
        type: 'rotate',
        values: opts.localRotate.values,
        keyTimes: opts.localRotate.keyTimes,
        calcMode: 'spline',
        keySplines: opts.localRotate.keySplines || buildKeySplines(opts.localRotate.values.split(';').length),
        dur: `${f1(opts.localRotate.dur || dur)}s`,
        begin,
        repeatCount: 'indefinite'
      }, rotG);
      target = rotG;
    }

    if (opts.node) target.appendChild(opts.node);
    return holder;
  }

  /**
   * Attach IntersectionObserver so SMIL runs only while on screen.
   */
  function observeVisibility(svgEl, autoPause) {
    if (autoPause === false) return;
    const bc = typeof globalThis !== 'undefined' ? globalThis.BestiaryCrawler : null;
    if (bc && typeof bc.observeVisibility === 'function') {
      bc.observeVisibility(svgEl);
      return;
    }
    if (typeof IntersectionObserver === 'undefined' || !svgEl || typeof svgEl.pauseAnimations !== 'function') return;
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (svgEl.dataset.bestiaryManualPause === '1') continue;
        if (e.isIntersecting) svgEl.unpauseAnimations();
        else svgEl.pauseAnimations();
      }
    }, { threshold: 0.01 });
    io.observe(svgEl);
  }

  return {
    SVG_NS,
    INK,
    IVORY,
    uid,
    el,
    f1,
    f2,
    f4,
    clamp,
    buildKeySplines,
    buildTracedGroup,
    samplePolyline,
    mountSegmentedBody,
    mountJointPart,
    observeVisibility
  };
}));
