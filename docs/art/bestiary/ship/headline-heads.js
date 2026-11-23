/**
 * Brief 6 — Start Here: Traced Roundel Heads on the Headline Frame Serpents
 * Keeps the existing headline frame serpent bodies, size, writhing chase motion,
 * bottom snap & recoil, and gold caption clearance unchanged, replacing ONLY
 * the heads with the potrace-traced Romanesque roundel horned hound head
 * (traced/components/roundel/head.svg) recoloured via --wash / --wash2 with
 * separate animated jaw, blink lid, and ear twitch.
 */
(function (root) {
  'use strict';

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const INK = '#19100b';
  const IVORY = '#fff3d6';

  // Exact potrace-extracted paths from traced/components/roundel/head.svg
  const ROUNDEL_HEAD = {
    jawBase: 'M117.5 145.2h3.1c4.3 0 4.3 0 5.1 .8l2.5 .2c5.4 .5 5.4 .5 7.8 2.1 1.3 .6 1.6 .7 3 .5l1-.8 7.5 4.4 3.7 2.5c4 2.2 4.5 5.8 5 10.1l-.2 2c-1.4 .7-2.3 .3-3.8 0l-4.2-.7c-2.2-.5-2.8-.7-3.9-2.7-.4-.7-.9-1.1-1.6-1.6l-5.8-5-1-.3c-1.7-.6-3.3-1.3-4.6-2.6-.8-.8-1.7-1-2.9-1.3l-.7-.4c-1.2-.5-2.5-.8-3.7-1l-4-1.1c-.9-.3-1.6-.4-2.6-.4-1.3-.1-1.7-.2-2.7-.9-.6-1.2-.5-2-.3-3.2 1.2-.5 2.1-.6 3.3-.6z',
    jawHi: 'M117 147c9 .2 17.8 2.4 26 6v1.5l.5-1 2 1-1 1v1h1.5l-.5-1h1l1-2.5c2 .5 2 .5 3 3-4 2.4-5.5 2-9.5 0l-3-2c-5.6-2.8-11.1-4.2-17.3-4.8l-3.7-.7zM147 160.5h2.5l1 6-2-.5z',
    jawInk: 'M135.7 148.1c1.5 .5 3 .8 4.6 1.1l.3-.7 1.2 1.1 1.8 .4 3 1.6c.8 .6 .8 .6 .8 2.1-2.2 .5-2.3 .3-4.1-.7-2.9-1.4-5.9-2.3-9-3.1l-1.5-.4c-4.3-1.1-8.7-1.7-13.2-2l-2.6-.1 .4 1.5h1.1c3.9 .2 7.5 .8 11.3 1.8l1.4 .4c4.1 1.3 7.6 3.3 11.2 5.5 3.8 2.3 7.8 3 12.1 1.6l.8 1.2c-1.8 .9-3.1 .9-5 1l-3.3 .1 .8 2.6c.3 1.1 .3 1.1 0 3l-2.3-.4-1.1-2.2c-1.3-1.4-2.7-2.6-4.2-3.8l-3.3-3-2.6-.7c-1.3-.7-2.3-1.4-3.4-2.3-2.9-1.2-5.7-2-8.7-2.6l-1.3-.3c-2.1-.5-4.1-.9-6.1-1.2l-.8-3.4c6.1-2.4 15.7-.4 21.7 1.9z',
    skullBase: 'M150.8 115.2c1 .4 2 .8 3 1.4l2.2 1.4c1.7 1.1 3.9 1.5 5.9 1.8 1.3 .2 1.3 .2 1.9 .7l2.5 .6c1.9 .3 3.4 0 5.2-.5 .7-.1 .7-.1 2 .2 .7 2 .9 4 1 6.2v1.3c.1 2.5-.1 4.6-.7 6.9-.4 2.6-.5 5.1 .1 7.6s.4 4.8 .4 7.2v1.4c0 2.2 0 4.3-.4 6.4l-.5 4.4c-.4 2.5-.4 2.5-1.6 3.2l-10.1 2c-2.7 .7-5.5 .4-8.2-.2-.3-.7-.3-1.4-.3-2.2-.3-3.3-.8-6.4-3.9-8.2l-6.8-4c-3.7-2-3.7-2-4.7-3v-.6l1.4-.7c-.2-1.5-.5-1.5-1.7-2.3l-.5-.4-.8-.3c-1-.3-1.8-.7-2.7-1.3l-3.3-1.5c-1.2-.4-2.3-.8-3.5-1l-1.9-.7-1.2-.2c-1.7-.4-3.2-1.1-4.8-1.8l-1-.4c-1.1-.5-2-1.2-3-1.8l-1.1-.5c-1.3-.7-1.7-1.6-2.5-2.8l-.5-.9c-.5-1.3-.6-2.2-.6-3.5v-1.3c.1-1 .3-1.4 .9-2.3l.2-.9c.3-1.2 .8-2.1 1.5-3.2l.6-1c1.1-1.4 1.9-1.7 3.7-1.6 3.4 .5 3.4 .5 4.6 1.1 5.1 2.1 13.2 .7 17.9-1.7l.8-.1c.9-.1 .9-.1 1.7-.5 1.3-.5 2.6-.4 4-.4 2.8 .1 2.8 .1 4.2-.4z',
    skullWash2: 'M144.4 130.6c1.7 0 1.7 0 5 3.8 9.2 6.2 9.2 6.2 12.3 6.2l1.1-1.7 2.8 2.8c2.7-.6 2.7-.6 5-5.6 .5 1.7 .5 1.7-.6 4.5l1.7 5c.2 4.7-.2 7.5-1.7 12.2v5.5c-1.6 4.1-3.7 3.4-7.8 3.9 .3-5.9 .3-5.9 2.8-7.8h1.1c1.1-2.7 1.1-2.7 1.1-5.5-1.6 0-1.6 0-2.2 1.1h-1.1c.5-2.2 .5-2.2 1.7-3.9 0-3.9 0-3.9-1.7-6.7-4-1.5-8-1.7-12.2-2.2 .5 1.7 .5 1.7 2.2 2.2 1.3 9.6 1.3 9.6-.6 13.9l-2.7-.5 .5-2.8c1-6.8-2.3-9.9-7.2-13.9l-4.5-2.8v-1.1l5.6 1.7 2.8-1.1-1.7-3.3c-1.7-3.4-1.7-3.4-1.7-3.9z',
    skullHi: 'M145.5 120c1.1 2.1 1 3.2 1 5.5 .5 1.5 .5 1.5 2.5 2.5l3.5 3.5 1.5 1v1.5h-1l-.5 1.5-4.1-4.5c-2.4-2.5-4.7-3.3-7.9-4-1-.5-1-.5-1.4-2.4-.1-2.1-.1-2.1 1.9-4.6 2.5-.5 2.5-.5 4.5 0zM117.3 121.1c1.7 .4 1.7 .4 2.2 .9 5.6 .6 11.3 .6 17 .5v2.5l-7.9-.2c-3.5-.2-6.7-.5-10.1-1.3l-1 2.5-3.5-.5c-.7-2.9 .3-4.5 3.3-4.4zM169.5 121l2 .5c.8 6 .2 11.1-2.5 16.5l-.5 2c-2.3-.6-3.6-1.1-5.5-2.5-.4-3.8 .6-6.9 1.8-10.5l1.7-5.5 2 .5v2.5l-2 .5 1 1c1 1.9-1 3.8-2 5.5h-1l.5 5.5c2.9-.7 3-2.3 4-5 .5-2.5 .6-4.9 .5-7.5zM112.5 126.5c2 .5 2 .5 2.5 2l1.5 1h-1v1h1l1.5 1.5c2.2 1.7 3.9 3 6.5 4l1-.5 1.5 1.5 4.5 1 2 1h2.5c2.5 1.5 2.5 1.5 2.5 2.5l3 .5 5 1.5-1 1.5c.5 2 .5 2 2.3 3.2s1.7 1.3 1.7 3.8h2l-1.5 4.5-3-1 .2-3.1c-.6-5.3-7.6-8.5-12.1-10.5-4-1.6-8-2.6-12.2-3.4-2.6-.5-4.9-1.4-7.4-2.5v-1c2-.5 2-.5 3 0v-1h-2.5l-.5-2-2 .5c-1.5-2-1.5-2-1.5-5z',
    skullInk: 'M153.4 118.2c1.5 .3 1.5 .3 2.2-.1l.8 1.5 .7 .4v-1.1c1.5 .4 1.5 .4 2.3 2.2l3 1.5-.4-3h1.1l1.2 1.9 1.1 .4v-.8h.7c-.5 3-1.2 5.8-2.2 8.7l-.8-.4 .4-2.6-1.1-.4-.5-.9c-.8-1.2-1.4-1.5-2.7-2l-8.2-2.8c-3.8-.9-8-1.9-11.5 .4l-1.1 2.3-1.5-.8h-3.6c-3.4 .1-3.4 .1-5-.3l-2.3 .3c-2.9 .1-4.9-.2-7.5-1.5-3-.3-3-.3-4.3 .8-.6 1.1-.6 1.1-.6 3 1.5 1.3 3.4 1.5 5.3 1.9v.7l-1.1 .4-.8 1.1-1.1-.9c-2.1-1.7-2.1-1.7-3.4-1.7-.5 1-.4 1.8-.4 2.9l-.1 1.2c.2 2.6 1.5 4 3.5 5.6 3 1.3 6.1 2 9.4 2.7 8.4 1.8 16.8 4.8 22.1 12 .4 1.7 .4 1.7 .4 3-2.3 .4-3-.1-5.1-1.1l-1.7-.8c-1.5-.8-1.5-.8-3-1.9v-.7h2.7c-2.4-5-10-6.7-14.8-7.9-5.9-1.5-11.9-3.5-14.9-9-1.5-4.5 .8-8.7 3-12.4 1.2-1.1 2.2-1 3.8-1.1l2.9 .7c6.1 1.5 13.2 1.5 18.8-1.5l2.3-.3c4-1.7 7.9-.6 12 .4zM173.6 120.4c1.1 5.8 1.5 10.4 0 16.2-.4 1.9-.2 3.1 .2 5 1.1 6.2 .9 12.7-.2 18.9l-.2 1.2c-.4 2.8-.4 2.8-1.3 3.7-2 .4-2 .4-3 0l.4-1c.4-1.4 .7-2.6 0-3.9l.8-5.5c.3-1.6 .3-1.6 1.5-2.8 .3-2.7 .4-6.6-1.2-9-1.5 .8-1.5 .8-2.6 1.9l-.8-.3c1.5-1.9 1.5-1.9 2.3-1.9l.1-1.2c.3-1.5 .7-2.8 1.3-4.2 .8-2.3 .9-3.8 .5-6.3l.6-2.5c.3-2.2 0-4.4-.2-6.6-.1-1.7-.1-1.7 1.8-1.7zM169.1 123.4h.4c.1 4.8 0 9-2.6 13.1h-1.5l-.4-2.6 1.9-.4c.6-2.5 1.1-5 1.5-7.5zM139.5 127.1c4.9 1.2 8.4 2.4 11.3 6.8l2.6 2.6-.8 .4c-5.8-4.1-5.8-4.1-6.4-6.4l-2.2-.4v1.1l1.1 .4c1.1 2.6 1.1 2.6 1.1 4.2-1.2-1.2-1.9-2.1-2.7-3.7-1.1-1.7-1.5-1.9-3.3-2.7zM139.5 130.1l1.8 2.9c.5 .9 .5 .9 .5 1.6l-1.8-.4c-2-.3-2-.3-3.5 0l.4-3.3c1.9-.8 1.9-.8 2.6-.8zM163.5 155.2c-3.2 3-7.1 5.4-11.6 4.6 .3-1.2 .3-1.2 1.5-1.5l1.6-.4c2.5-.5 4.6-1.3 6.9-2.6 .9-.4 .9-.4 1.6-.1z',
    earsBase: 'M147.4 99.3c1 .2 1.5 .6 2.2 1.1l2.2 1.8 2.6 2.6c.7 .5 1.1 1.2 1.6 2l.9 .8c.8 .9 1.3 1.6 1.8 2.7l.7 .5c1.1 1.3 1.7 2.9 2.4 4.5 .4 1 .4 1 .9 1.5l.4 1.7c.4 1.7 .4 1.7 .6 2 1.5 .1 1.5 .1 1.8-.2l.1-10.4c.1-1.1 .1-1.1 1.1-1.9 2 0 2.5 1.3 3.5 2.8l1.7 3.2c.6 1.4 .8 2.8 1 4.3 .1 1.2 .4 2.1 .8 3.2v1.3c-3.3 1.9-6.9 1.5-10.5 .5l-.7-.5-4-.7c-2-.4-3.4-1.3-5.1-2.5-1.1-.7-2.2-1.3-3.4-1.8l.2-1c.1-1.8 .1-3.4-.5-5l-.1-1.2c-.5-2.7-1.3-5.3-2.4-7.8-.5-1.1-.6-2.1-.7-3.3 .2-.2 .2-.2 .9-.2z',
    earsHi: 'M148.5 102c3.1 1.2 4.1 2.8 6 5.5v1l1.5 .5c4.7 7.3 4.7 7.3 4 10h-1.5l-2.6-6c-1.9-4.3-1.9-4.3-3.4-5l.7 1.9c.8 2.7 1.1 5.3 1.3 8.1h-3l.5-3-.9-6.5c-.7-2.2-1.6-3.3-3.1-5zM166.5 109.5c4 2 4.9 7.8 5.2 12l-.2 2-1.5 .5c-.5-1-.5-1-.5-4h-2.5z',
    earsInk: 'M146.6 99c2.8 .6 5.1 2.4 6.4 4.9l1.5 .7c4.2 4.7 8 9.9 9.4 16.2 .3 .7 .3 .7 1.5 1.1l.4-10.9h.3c.7 3.5 .9 6.3 .2 9.9l-.5 2.9-3-.4-.5-1.4c-1.8-5.4-4.2-10.4-7.8-14.8l-2.5-3.2c-.9-.9-1.6-1.5-2.8-2-1.1 .4-1.1 .4-1.8 1.5-1.2-2.8-1.2-2.8-.8-4.5zM148.1 104.2c1.6 .6 1.9 1.2 2.7 2.7l.9 5.1c.4 4 .4 4-.6 5-.8-1.2-.9-1.8-1.1-3.3-.3-2.4-.9-4.8-1.6-7.2-.3-1.1-.3-1.1-.3-2.3zM167.2 108c1.2 0 1.2 0 2.3 1.5 1.9 3 3.4 6.1 3.4 9.7l.7 1.2 .4 2.6-1.9 .4-.2-1.1c-.8-4.9-1.8-9.4-4.7-13.5zM153.8 109.9c1.5 1 1.8 2.3 2.6 4l3 7.2 1.5 1.5c-4-.6-6.9-2.1-10.1-4.5 1.3-.6 2.7 .1 4.1 .4l.7-.4 .8 1.5 .7 .4c-1-4.4-1-4.4-2.6-5.2z'
  };

  function el(tag, attrs, parent) {
    const node = document.createElementNS(SVG_NS, tag);
    if (attrs) {
      for (const k in attrs) {
        if (attrs[k] !== undefined && attrs[k] !== null) node.setAttribute(k, String(attrs[k]));
      }
    }
    if (parent) parent.appendChild(node);
    return node;
  }

  /**
   * Build the potrace-traced Romanesque roundel horned hound head, oriented
   * facing +x with origin (0,0) at the anatomical neck joint (166,156),
   * recoloured via --wash / --wash2, with synchronized SMIL jaw snap, blink,
   * and ear twitch.
   */
  function buildTracedRoundelHead(opts) {
    opts = opts || {};
    const dur = opts.dur || 11;
    const wash = opts.wash || 'var(--wash, #ff5a1f)';
    const wash2 = opts.wash2 || 'var(--wash2, #b82d00)';
    const scale = opts.scale !== undefined ? opts.scale : 0.44;
    const durS = `${(Math.round(dur * 10) / 10)}s`;

    // Mirror x so the left-facing roundel head faces +x (travel direction)
    const rootG = el('g', {
      class: 'bestiary-traced-head bestiary-traced-roundel-head',
      transform: `scale(${(-scale).toFixed(3)},${scale.toFixed(3)}) translate(-166,-156)`
    });

    // Separate traced lower jaw pivoting at (149,155) — opens wide at bottom snap (t=0.50..0.58)
    const jawOut = el('g', { transform: 'translate(149,155)' }, rootG);
    const jawAnim = el('g', {}, jawOut);
    el('animateTransform', {
      attributeName: 'transform',
      type: 'rotate',
      values: '0;0;-18;-3;-20;0;0;-12;0;0',
      keyTimes: '0;.46;.51;.54;.57;.62;.68;.72;.77;1',
      calcMode: 'spline',
      keySplines: '0 0 1 1;.3 0 .4 1;.4 0 .6 1;.3 0 .4 1;.4 0 .6 1;0 0 1 1;.3 0 .5 1;.4 0 .6 1;0 0 1 1',
      dur: durS,
      repeatCount: 'indefinite'
    }, jawAnim);
    const jawIn = el('g', { transform: 'translate(-149,-155)' }, jawAnim);
    el('path', { d: ROUNDEL_HEAD.jawBase, fill: wash }, jawIn);
    el('path', { d: ROUNDEL_HEAD.jawHi, fill: IVORY, opacity: '0.88' }, jawIn);
    el('path', { d: ROUNDEL_HEAD.jawInk, fill: INK, 'data-role': 'ink' }, jawIn);

    // Traced horned skull
    const skullG = el('g', {}, rootG);
    el('path', { d: ROUNDEL_HEAD.skullBase, fill: wash }, skullG);
    el('path', { d: ROUNDEL_HEAD.skullWash2, fill: wash2, opacity: '0.94' }, skullG);
    el('path', { d: ROUNDEL_HEAD.skullHi, fill: IVORY, opacity: '0.88' }, skullG);
    el('path', { d: ROUNDEL_HEAD.skullInk, fill: INK, 'data-role': 'ink' }, skullG);

    // Blink lid over the traced eye socket (140,131)
    const lid = el('ellipse', {
      cx: '140.2', cy: '131.0', rx: '4.6', ry: '3.4',
      fill: wash, stroke: INK, 'stroke-width': '1.0', opacity: '0'
    }, skullG);
    el('animate', {
      attributeName: 'opacity',
      values: '0;0;1;1;0;0;1;1;0;0',
      keyTimes: '0;.23;.245;.265;.28;.82;.835;.855;.87;1',
      dur: durS,
      repeatCount: 'indefinite'
    }, lid);

    // Separate traced twin ears twitching alertly around (160,120)
    const earsOut = el('g', { transform: 'translate(160,120)' }, rootG);
    const earsAnim = el('g', {}, earsOut);
    el('animateTransform', {
      attributeName: 'transform',
      type: 'rotate',
      values: '0;0;5;-4;0;0;4;0',
      keyTimes: '0;.44;.51;.58;.64;.76;.82;1',
      calcMode: 'spline',
      keySplines: '0 0 1 1;.3 0 .5 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1;.3 0 .5 1;.4 0 .6 1',
      dur: durS,
      repeatCount: 'indefinite'
    }, earsAnim);
    const earsIn = el('g', { transform: 'translate(-160,-120)' }, earsAnim);
    el('path', { d: ROUNDEL_HEAD.earsBase, fill: wash }, earsIn);
    el('path', { d: ROUNDEL_HEAD.earsHi, fill: IVORY, opacity: '0.88' }, earsIn);
    el('path', { d: ROUNDEL_HEAD.earsInk, fill: INK, 'data-role': 'ink' }, earsIn);

    return rootG;
  }

  /**
   * Mount the headline frame keeping the existing frame serpent bodies & motion
   * intact while swapping ONLY the heads for the traced Romanesque roundel heads
   * recoloured to the frame's neon orange (--wash / --wash2).
   */
  function mountHeadlineHeads(bubbleEl, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      wash: '#ff5a1f',
      wash2: '#b82d00',
      scale: narrow ? 0.72 : 0.86,
      dur: 11,
      autoPause: true
    }, overrides || {});

    // Reuse the existing headline frame bodies, track, snap & recoil, and gold spark
    const svg = root.BestiaryCrawler.makeHeadlineChase(bubbleEl, opts);
    if (!svg) return null;

    svg.style.setProperty('--wash', opts.wash);
    svg.style.setProperty('--wash2', opts.wash2);
    svg.setAttribute('data-rig', 'headline-heads');

    // Swap ONLY the two heads in place, preserving their animateMotion & snap/recoil <g>
    const oldHeads = svg.querySelectorAll('.bestiary-head');
    oldHeads.forEach(function (oldHead) {
      const parent = oldHead.parentNode;
      if (!parent) return;
      const tracedHead = buildTracedRoundelHead({
        dur: opts.dur,
        wash: `var(--wash, ${opts.wash})`,
        wash2: `var(--wash2, ${opts.wash2})`,
        scale: 0.44
      });
      parent.replaceChild(tracedHead, oldHead);
    });

    return svg;
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.buildTracedRoundelHead = buildTracedRoundelHead;
  root.BestiarySlots.mountHeadlineHeads = mountHeadlineHeads;
}(typeof globalThis !== 'undefined' ? globalThis : this));
