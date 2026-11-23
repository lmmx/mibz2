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

// generated by make_heads.py from the bestiary SVGs; do not edit by hand
const BESTIARY_HEADS = {
 "amphisbaena-a": "<path d=\"M1.4,-6.4 C-.6,-9.6 2.2,-11.0 4.0,-8.0Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".64\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L16,-1.4 L14.2,2.6 L6.2,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(2)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.9,-.1Z M6.8,0 L7.4,-1.4 L8.1,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M11,.4 L11.5,-.7 L12,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -2,2.9 C-4.6,2.6 -6.2,2.1 -7.4,1.6 C-6.4,.1 -4.4,-.5 -2.4,-.5Z\" fill=\"#d22f1c\"/><path d=\"M-4.8,1.9 C.2,2.7 7,2.8 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".85\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -1.8,2.9 C-4.4,2.6 -5.8,2.1 -7,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.8 19.8,-3.7 C20.6,-3.2 20.4,-1.8 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C4.8,.6 1.8,1.9 -2.4,2.7 C-3.3,.7 -3.5,-1.8 -2.8,-4Z\" fill=\"#d22f1c\"/><path d=\"M-2.4,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.8 19.8,-3.7 C20.6,-3.2 20.4,-1.8 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C5.8,.4 4.6,.8 3.4,1.3\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.5 C3.2,-7.3 6.8,-7.2 9.8,-5.7 M12.8,-4.4 C15.2,-4.4 17.4,-4.2 18.8,-3.6\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".58\" stroke-linecap=\"round\"/><path d=\"M2.2,-6.8 C.2,-10.4 4.2,-11.6 6.0,-7.8 C6.4,-6.9 5.4,-6.2 4.2,-6.4Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M2.6,-8.2 C2.8,-9.4 4.4,-9.4 4.8,-8.0\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".54\" stroke-linecap=\"round\"/><path d=\"M17.8,-3.4 C18.4,-3.9 19.2,-3.5 18.9,-2.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M4.2,-4.8 h.1 M5.8,-5.4 h.1 M5.4,-2.6 h.1\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".7\" stroke-linecap=\"round\"/><g transform=\"translate(9.3,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "amphisbaena-b": "<path d=\"M-1.4,-4.4 L-3.8,-5.8 L-2.6,-3.0 L-4.4,-2.2 L-2.4,-.2 L-4.0,1.6 L-1.8,2.8 L-3.0,4.2 L-.4,3.4Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".62\" stroke-linejoin=\"round\"/><path d=\"M1.4,-6.4 C-.6,-9.6 2.2,-11.0 4.0,-8.0Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".64\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L16,-1.4 L14.2,2.6 L6.2,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(2)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.9,-.1Z M6.8,0 L7.4,-1.4 L8.1,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M11,.4 L11.5,-.7 L12,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -2,2.9 C-4.6,2.6 -6.2,2.1 -7.4,1.6 C-6.4,.1 -4.4,-.5 -2.4,-.5Z\" fill=\"#d22f1c\"/><path d=\"M-4.8,1.9 C.2,2.7 7,2.8 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".85\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -1.8,2.9 C-4.4,2.6 -5.8,2.1 -7,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.8 19.8,-3.7 C20.6,-3.2 20.4,-1.8 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C4.8,.6 1.8,1.9 -2.4,2.7 C-3.3,.7 -3.5,-1.8 -2.8,-4Z\" fill=\"#d22f1c\"/><path d=\"M-2.4,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.8 19.8,-3.7 C20.6,-3.2 20.4,-1.8 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C5.8,.4 4.6,.8 3.4,1.3\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.5 C3.2,-7.3 6.8,-7.2 9.8,-5.7 M12.8,-4.4 C15.2,-4.4 17.4,-4.2 18.8,-3.6\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".58\" stroke-linecap=\"round\"/><path d=\"M2.8,-7.2 L-.2,-11.6 L6.0,-7.8Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M1.8,-9.6 L4.4,-7.9\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".54\" stroke-linecap=\"round\"/><path d=\"M17.8,-3.4 C18.4,-3.9 19.2,-3.5 18.9,-2.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M4.2,-4.8 h.1 M5.8,-5.4 h.1 M5.4,-2.6 h.1\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".7\" stroke-linecap=\"round\"/><g transform=\"translate(9.3,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "aspis": "<!-- Shaggy medieval collar ruff behind cheek --> <path d=\"M-1.2,-4.6 L-3.6,-6.0 L-2.4,-3.2 L-4.4,-2.6 L-2.5,-0.5 L-4.2,1.2 L-2.0,2.4 L-3.4,4.2 L-0.4,3.4 Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\"0.62\" stroke-linejoin=\"round\"/> <!-- Back pointed hound ear --> <path d=\"M1.2,-6.6 C-0.2,-10.0 2.6,-11.2 4.2,-8.4 C4.6,-7.4 3.8,-6.6 2.8,-6.4 Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\"0.65\" stroke-linejoin=\"round\"/> <!-- Dark madder mouth cavity --> <path d=\"M6.4,-0.8 L16.2,-1.5 L14.4,2.8 L6.6,2.0 Z\" fill=\"#6e130f\"/> <!-- Lower jaw (pivots at real jaw hinge (6.2, 1.5)) --> <g transform=\"translate(6.2,1.5)\"> <g class=\"h-jaw\" transform=\"rotate(2)\"> <path d=\"M4.6,-0.2 L5.2,-1.5 L5.9,-0.1 Z M6.8,0.0 L7.4,-1.4 L8.1,0.1 Z M8.9,0.2 L9.6,-1.1 L10.2,0.4 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.22\" stroke-linejoin=\"round\"/> <path d=\"M-2.2,-0.5 C1.0,-0.4 5.2,-0.2 10.6,0.4 C11.4,0.5 11.7,1.4 11.0,2.1 C9.2,3.4 4.2,3.6 -1.6,3.0 C-4.2,2.7 -6.0,2.2 -7.2,1.8 C-6.2,0.2 -4.2,-0.4 -2.2,-0.5 Z\" fill=\"#d22f1c\"/> <path d=\"M-4.6,2.0 C0.2,2.8 6.4,2.9 10.2,1.7\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\"0.85\" stroke-linecap=\"round\"/> <path d=\"M-2.2,-0.5 C1.0,-0.4 5.2,-0.2 10.6,0.4 C11.4,0.5 11.7,1.4 11.0,2.1 C9.2,3.4 4.2,3.6 -1.6,3.0 C-4.2,2.7 -5.8,2.2 -7.0,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> </g> </g> <!-- Forked RED tongue (pivots at mouth throat (10.8, 0.3), fully extended along +x) --> <g transform=\"translate(10.8,0.3)\"> <g class=\"h-tongue\"> <g> <path d=\"M0,0 C3.8,-0.2 7.4,0.5 11.6,0.2 M11.2,0.2 C13.5,-0.3 15.2,-1.5 17.2,-2.9 M11.2,0.2 C13.5,0.8 15.4,2.0 17.4,3.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> <path d=\"M0,0 C3.8,-0.2 7.4,0.5 11.6,0.2 M11.2,0.2 C13.5,-0.3 15.2,-1.5 17.2,-2.9 M11.2,0.2 C13.5,0.8 15.4,2.0 17.4,3.1\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\"0.82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> </g> </g> </g> <!-- Upper jaw teeth --> <path d=\"M11.2,-1.1 L11.8,0.3 L12.5,-1.2 Z M13.4,-1.3 L14.0,0.2 L14.7,-1.4 Z M15.6,-1.6 L16.2,-0.1 L16.8,-1.7 Z M17.5,-1.8 L18.0,-0.4 L18.5,-1.9 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.22\" stroke-linejoin=\"round\"/> <!-- Upper skull and hound snout (open-back join into thick muscular neck) --> <path d=\"M-2.6,-4.0 C-0.8,-7.1 2.6,-8.8 6.4,-8.3 C8.8,-8.0 10.6,-6.7 11.8,-5.4 C13.8,-4.9 16.4,-5.1 18.6,-4.4 C19.8,-4.0 20.2,-2.6 19.3,-1.8 C18.5,-1.3 16.4,-1.4 13.8,-1.2 C11.2,-1.0 8.6,-0.4 6.6,0.3 C4.2,1.1 1.2,2.0 -2.2,2.8 C-3.0,0.8 -3.2,-1.8 -2.6,-4.0 Z\" fill=\"#d22f1c\"/> <path d=\"M-2.4,-4.0 C-0.8,-7.1 2.6,-8.8 6.4,-8.3 C8.8,-8.0 10.6,-6.7 11.8,-5.4 C13.8,-4.9 16.4,-5.1 18.6,-4.4 C19.8,-4.0 20.2,-2.6 19.3,-1.8 C18.5,-1.3 16.4,-1.4 13.8,-1.2 C11.2,-1.0 8.6,-0.4 6.6,0.3 C5.4,0.7 4.2,1.1 3.2,1.5\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> <!-- Lead-white ridge stroke along crown and muzzle bridge --> <path d=\"M0.4,-5.6 C3.0,-7.5 6.8,-7.6 9.6,-6.1 M12.6,-4.4 C14.8,-4.3 16.6,-4.3 18.0,-3.8\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\"0.56\" stroke-linecap=\"round\"/> <!-- Front hound ear tucked at crown --> <path d=\"M3.2,-7.6 C2.2,-10.8 5.4,-11.6 6.4,-8.4 C6.7,-7.4 5.8,-6.7 4.6,-6.8 Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\"0.66\" stroke-linejoin=\"round\"/> <path d=\"M4.2,-8.0 C4.0,-9.3 5.0,-9.6 5.4,-8.3\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\"0.54\" stroke-linecap=\"round\"/> <!-- Curled nostril & lip crease --> <path d=\"M17.4,-3.6 C18.0,-4.1 18.9,-3.7 18.6,-3.0\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.5\" stroke-linecap=\"round\"/> <path d=\"M11.6,-2.5 C14.0,-2.6 16.2,-2.4 18.2,-2.1\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\"0.44\" stroke-linecap=\"round\"/> <!-- Medieval cheek creases --> <path d=\"M2.6,-2.4 C3.8,-1.0 3.7,0.5 2.6,1.6 M0.8,-1.8 C1.8,-0.5 1.7,0.8 0.7,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.48\" stroke-linecap=\"round\"/> <!-- Small lead-white accents on cheek --> <path d=\"M4.2,-4.8 h0.1 M5.8,-5.5 h0.1 M5.6,-2.8 h0.1 M4.0,-2.0 h0.1\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\"0.7\" stroke-linecap=\"round\"/> <!-- Almond eye under heavy iron-gall lid (origin at eye center (9.0, -4.4)) --> <g transform=\"translate(9.0,-4.4)\"> <g class=\"h-eye\"> <path d=\"M-2.2,0.1 C-1.1,-1.1 1.1,-1.1 2.3,0.1 C1.2,1.1 -1.0,1.1 -2.2,0.1 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.42\" stroke-linejoin=\"round\"/> <circle cx=\"0.35\" cy=\"0.05\" r=\"0.68\" fill=\"#1c100a\"/> <g transform=\"translate(0,-1.05)\"> <g class=\"h-lid\" transform=\"scale(1 0.06)\"> <path d=\"M-2.3,0.0 C-1.0,-0.2 1.0,-0.2 2.4,0.0 L2.3,2.0 C1.0,2.3 -1.0,2.3 -2.2,2.0 Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\"0.38\" stroke-linejoin=\"round\"/> <path d=\"M-2.2,1.9 C-0.8,2.2 0.8,2.2 2.2,1.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.55\" stroke-linecap=\"round\"/> </g> </g> <path d=\"M-2.6,-0.1 C-1.2,-1.7 1.2,-1.6 2.7,-0.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.82\" stroke-linecap=\"round\"/> </g> </g>",
 "draco-alatus": "<path d=\"M2.4,-7.0 C-.6,-9.6 -2.6,-12.0 -1.2,-13.0 C.6,-11.4 3.2,-9.6 4.8,-7.4Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".66\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L16,-1.4 L14.2,2.6 L6.2,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(3)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.9,-.1Z M6.8,0 L7.4,-1.4 L8.1,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M11,.4 L11.5,-.7 L12,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -2,2.9 C-4.6,2.6 -6.2,2.1 -7.4,1.6 C-6.4,.1 -4.4,-.5 -2.4,-.5Z\" fill=\"#d22f1c\"/><path d=\"M-4.8,1.9 C.2,2.7 7,2.8 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".85\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -1.8,2.9 C-4.4,2.6 -5.8,2.1 -7,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.6,-4.8 17.2,-5.2 19.6,-4.2 C20.5,-3.7 20.4,-2.1 19.3,-1.5 C16.6,-1.1 12,-.8 7.4,.1 C4.8,.6 1.8,1.9 -2.4,2.7 C-3.3,.7 -3.5,-1.8 -2.8,-4Z\" fill=\"#d22f1c\"/><path d=\"M-2.4,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.6,-4.8 17.2,-5.2 19.6,-4.2 C20.5,-3.7 20.4,-2.1 19.3,-1.5 C16.6,-1.1 12,-.8 7.4,.1 C5.8,.4 4.6,.8 3.4,1.3\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.5 C3.2,-7.3 6.8,-7.2 9.8,-5.7 M12.8,-4.5 C15.2,-4.5 17.2,-4.4 18.6,-3.8\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".58\" stroke-linecap=\"round\"/><path d=\"M4.2,-7.5 C1.4,-10.2 .0,-12.8 1.6,-13.6 C3.4,-11.8 5.6,-10.0 6.8,-7.6Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M1.2,-6.0 C-1.4,-7.4 -.6,-9.2 2.4,-7.6Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".62\" stroke-linejoin=\"round\"/><path d=\"M17.8,-3.5 C18.4,-4.0 19.2,-3.6 18.9,-2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><g transform=\"translate(9.3,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "draco-bipes": "<path d=\"M1.2,-6.4 C-1.4,-9.4 .8,-11.4 3.8,-8.2Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".66\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L16,-1.4 L14.2,2.6 L6.2,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(2)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.9,-.1Z M6.8,0 L7.4,-1.4 L8.1,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M11,.4 L11.5,-.7 L12,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -2,2.9 C-4.6,2.6 -6.2,2.1 -7.4,1.6 C-6.4,.1 -4.4,-.5 -2.4,-.5Z\" fill=\"#d22f1c\"/><path d=\"M-4.8,1.9 C.2,2.7 7,2.8 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".85\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.5 C1.4,-.4 6.6,-.2 12.2,.5 C13,.7 13.1,1.6 12.2,2.1 C9,3.3 3.6,3.5 -1.8,2.9 C-4.4,2.6 -5.8,2.1 -7,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.8,-.1 7.4,.4 11.4,.1 M11,.1 C13.2,-.4 15,-1.5 16.8,-2.7 M11,.1 C13.2,.7 15.1,1.8 17,2.9\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.6 19.8,-3.5 C20.6,-3 20.4,-1.7 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C4.8,.6 1.8,1.9 -2.4,2.7 C-3.3,.7 -3.5,-1.8 -2.8,-4Z\" fill=\"#d22f1c\"/><path d=\"M-2.4,-4 C-.8,-7 2.8,-8.4 6.6,-7.8 C8.8,-7.5 10.4,-6.1 12,-5.3 C14.8,-4.6 17.6,-4.6 19.8,-3.5 C20.6,-3 20.4,-1.7 19.3,-1.4 C16.6,-1.1 12,-.8 7.4,.1 C5.8,.4 4.6,.8 3.4,1.3\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.5 C3.2,-7.3 6.8,-7.2 9.8,-5.7 M12.8,-4.4 C15.2,-4.3 17.4,-4.1 18.8,-3.5\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".58\" stroke-linecap=\"round\"/><path d=\"M2.8,-7.2 C.6,-10.8 4.4,-12 6.2,-8 C6.5,-7.2 5.4,-6.5 4.2,-6.7Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M2.8,-8.8 C3.6,-9.6 4.7,-9.0 5.0,-8.0\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".54\" stroke-linecap=\"round\"/><path d=\"M17.8,-3.3 C18.4,-3.8 19.2,-3.4 18.9,-2.7\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M4.2,-4.8 h.1 M5.8,-5.4 h.1 M5.4,-2.6 h.1\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".7\" stroke-linecap=\"round\"/><g transform=\"translate(9.3,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "knotted-pair-a": "<path d=\"M1.6,-6.2 L-1.6,-10.8 L4.4,-7.8Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".66\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L15.8,-1.3 L14.2,2.5 L6,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(3)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.8,-.1Z M6.8,0 L7.4,-1.4 L8,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M10.9,.4 L11.4,-.8 L11.9,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.4 C1.6,-.3 6.6,-.1 12.2,.6 C12.9,.8 13,1.7 12.2,2.1 C9,3.2 3.6,3.3 -2,2.8 C-4.5,2.5 -6,2.1 -7.2,1.6 C-6.2,.1 -4.2,-.4 -2.4,-.4Z\" fill=\"#d22f1c\"/><path d=\"M-4.6,1.9 C.4,2.6 7,2.7 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".82\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.4 C1.6,-.3 6.6,-.1 12.2,.6 C12.9,.8 13,1.7 12.2,2.1 C9,3.2 3.6,3.3 -1.8,2.8 C-4.4,2.5 -5.8,2.1 -6.8,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.6,-.1 7.2,.4 11,.1 M10.6,.1 C12.8,-.4 14.5,-1.5 16.2,-2.6 M10.6,.1 C12.8,.7 14.6,1.8 16.4,2.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.6,-.1 7.2,.4 11,.1 M10.6,.1 C12.8,-.4 14.5,-1.5 16.2,-2.6 M10.6,.1 C12.8,.7 14.6,1.8 16.4,2.8\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-3.9 C-.8,-6.8 2.8,-8.2 6.4,-7.7 C8.6,-7.4 10.2,-6 11.8,-5.2 C14.6,-4.6 17.6,-4.8 19.6,-4 C20.4,-3.6 20.4,-2.1 19.3,-1.6 C16.6,-1.2 12,-.9 7.4,0 C4.8,.5 1.8,1.7 -2.4,2.6 C-3.2,.6 -3.4,-1.7 -2.8,-3.9Z\" fill=\"#d22f1c\"/><path d=\"M-2.4,-3.9 C-.8,-6.8 2.8,-8.2 6.4,-7.7 C8.6,-7.4 10.2,-6 11.8,-5.2 C14.6,-4.6 17.6,-4.8 19.6,-4 C20.4,-3.6 20.4,-2.1 19.3,-1.6 C16.6,-1.2 12,-.9 7.4,0 C5.8,.3 4.6,.7 3.4,1.2\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.4 C3.2,-7.2 6.8,-7.1 9.8,-5.6 M12.8,-4.3 C15.2,-4.2 17.2,-4.2 18.6,-3.6\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".56\" stroke-linecap=\"round\"/><path d=\"M3.2,-7 L.4,-11.6 L6.2,-7.6Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M2.2,-9.6 L4.6,-7.8\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".54\" stroke-linecap=\"round\"/><path d=\"M17.8,-3.5 C18.4,-4 19.2,-3.6 18.9,-2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><g transform=\"translate(9.2,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "knotted-pair-b": "<path d=\"M1.4,-6.2 C-1.4,-9.6 1.8,-11.4 4.2,-8Z\" fill=\"#6e130f\" stroke=\"#1c100a\" stroke-width=\".66\" stroke-linejoin=\"round\"/><path d=\"M5.6,-.8 L15.8,-1.3 L14.2,2.5 L6,2Z\" fill=\"#6e130f\"/><g transform=\"translate(5.8,1.4)\"><g class=\"h-jaw\" transform=\"rotate(3)\"><path d=\"M4.6,-.2 L5.2,-1.5 L5.8,-.1Z M6.8,0 L7.4,-1.4 L8,.1Z M9,.2 L9.6,-1.1 L10.2,.4Z M10.9,.4 L11.4,-.8 L11.9,.5Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.4,-.4 C1.6,-.3 6.6,-.1 12.2,.6 C12.9,.8 13,1.7 12.2,2.1 C9,3.2 3.6,3.3 -2,2.8 C-4.5,2.5 -6,2.1 -7.2,1.6 C-6.2,.1 -4.2,-.4 -2.4,-.4Z\" fill=\"#a3141c\"/><path d=\"M-4.6,1.9 C.4,2.6 7,2.7 11.4,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".82\" stroke-linecap=\"round\"/><path d=\"M-2.2,-.4 C1.6,-.3 6.6,-.1 12.2,.6 C12.9,.8 13,1.7 12.2,2.1 C9,3.2 3.6,3.3 -1.8,2.8 C-4.4,2.5 -5.8,2.1 -6.8,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g><g transform=\"translate(10.8,.35)\"><g class=\"h-tongue\"><g><path d=\"M0,0 C3.6,-.1 7.2,.4 11,.1 M10.6,.1 C12.8,-.4 14.5,-1.5 16.2,-2.6 M10.6,.1 C12.8,.7 14.6,1.8 16.4,2.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.32\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M0,0 C3.6,-.1 7.2,.4 11,.1 M10.6,.1 C12.8,-.4 14.5,-1.5 16.2,-2.6 M10.6,.1 C12.8,.7 14.6,1.8 16.4,2.8\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\".82\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></g></g></g><path d=\"M10.8,-1 L11.4,.4 L12,-1.1Z M13,-1.2 L13.6,.3 L14.2,-1.3Z M15.2,-1.4 L15.8,.1 L16.4,-1.5Z M17.2,-1.6 L17.7,-.2 L18.3,-1.7Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".2\" stroke-linejoin=\"round\"/><path d=\"M-2.8,-3.9 C-.8,-6.8 2.8,-8.2 6.4,-7.7 C8.6,-7.4 10.2,-6 11.8,-5.2 C14.6,-4.6 17.6,-4.8 19.6,-4 C20.4,-3.6 20.4,-2.1 19.3,-1.6 C16.6,-1.2 12,-.9 7.4,0 C4.8,.5 1.8,1.7 -2.4,2.6 C-3.2,.6 -3.4,-1.7 -2.8,-3.9Z\" fill=\"#a3141c\"/><path d=\"M-2.4,-3.9 C-.8,-6.8 2.8,-8.2 6.4,-7.7 C8.6,-7.4 10.2,-6 11.8,-5.2 C14.6,-4.6 17.6,-4.8 19.6,-4 C20.4,-3.6 20.4,-2.1 19.3,-1.6 C16.6,-1.2 12,-.9 7.4,0 C5.8,.3 4.6,.7 3.4,1.2\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M.4,-5.4 C3.2,-7.2 6.8,-7.1 9.8,-5.6 M12.8,-4.3 C15.2,-4.2 17.2,-4.2 18.6,-3.6\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\".56\" stroke-linecap=\"round\"/><path d=\"M2.8,-6.8 C.2,-10.6 4,-11.8 6,-7.8Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".68\" stroke-linejoin=\"round\"/><path d=\"M2.6,-8.6 C3.4,-9.4 4.5,-8.8 4.8,-7.8\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\".54\" stroke-linecap=\"round\"/><path d=\"M17.8,-3.5 C18.4,-4 19.2,-3.6 18.9,-2.9\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><path d=\"M3.6,-2.2 C4.6,-.8 4.5,.6 3.5,1.6 M1.6,-1.8 C2.6,-.5 2.5,.8 1.5,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".5\" stroke-linecap=\"round\"/><g transform=\"translate(9.2,-4.2)\"><g class=\"h-eye\"><path d=\"M-2.2,.1 C-1.1,-1 1.1,-1 2.3,.1 C1.2,1 -1,1 -2.2,.1Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\".42\" stroke-linejoin=\"round\"/><circle cx=\".45\" cy=\".05\" r=\".68\" fill=\"#1c100a\"/><g transform=\"translate(0,-1)\"><g class=\"h-lid\" transform=\"scale(1 .06)\"><path d=\"M-2.3,0 C-1,-.2 1,-.2 2.4,0 L2.3,1.9 C1,2.2 -1,2.2 -2.2,1.9Z\" fill=\"#a3141c\" stroke=\"#1c100a\" stroke-width=\".38\" stroke-linejoin=\"round\"/><path d=\"M-2.2,1.8 C-.8,2.1 .8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".55\" stroke-linecap=\"round\"/></g></g><path d=\"M-2.5,-.1 C-1.2,-1.6 1.2,-1.5 2.7,-.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\".82\" stroke-linecap=\"round\"/></g></g>",
 "spotted-serpent": "<path d=\"M5.4,-0.8 L16.2,-1.2 L14.4,2.4 L6.0,1.9 Z\" fill=\"#6e130f\"/> <g transform=\"translate(5.8,1.4)\"> <g class=\"h-jaw\" transform=\"rotate(2)\"> <path d=\"M5.2,-0.1 L5.7,-1.3 L6.3,0.0 Z M7.4,0.1 L7.9,-1.2 L8.5,0.2 Z M9.6,0.3 L10.1,-1.0 L10.7,0.5 Z M11.6,0.5 L12.0,-0.6 L12.5,0.6 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.2\" stroke-linejoin=\"round\"/> <path d=\"M-2.2,-0.4 C1.6,-0.3 6.8,-0.1 12.6,0.7 C13.3,0.8 13.4,1.7 12.6,2.0 C9.2,3.0 3.6,3.2 -1.8,2.7 C-4.4,2.4 -6.0,2.0 -7.2,1.6 C-6.2,0.1 -4.2,-0.4 -2.2,-0.4 Z\" fill=\"#d22f1c\"/> <path d=\"M-4.6,1.9 C0.4,2.6 7.4,2.6 11.8,1.6\" fill=\"none\" stroke=\"#6e130f\" stroke-width=\"0.82\" stroke-linecap=\"round\"/> <path d=\"M-2.2,-0.4 C1.6,-0.3 6.8,-0.1 12.6,0.7 C13.3,0.8 13.4,1.7 12.6,2.0 C9.2,3.0 3.6,3.2 -1.8,2.7 C-4.4,2.4 -5.8,2.0 -7.0,1.6\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> </g> </g> <g transform=\"translate(11.2,0.35)\"> <g class=\"h-tongue\"> <g> <path d=\"M0,0 C4.0,0.1 7.8,-0.3 12.0,0.1 M11.6,0.1 C14.0,-0.4 15.8,-1.5 17.6,-2.7 M11.6,0.1 C14.0,0.7 15.9,1.8 17.8,2.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"1.28\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> <path d=\"M0,0 C4.0,0.1 7.8,-0.3 12.0,0.1 M11.6,0.1 C14.0,-0.4 15.8,-1.5 17.6,-2.7 M11.6,0.1 C14.0,0.7 15.9,1.8 17.8,2.8\" fill=\"none\" stroke=\"#d22f1c\" stroke-width=\"0.8\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> </g> </g> </g> <path d=\"M11.0,-0.9 L11.5,0.4 L12.2,-1.0 Z M13.2,-1.1 L13.8,0.2 L14.4,-1.2 Z M15.4,-1.3 L16.0,0.0 L16.6,-1.4 Z M17.4,-1.4 L17.9,-0.2 L18.4,-1.5 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.2\" stroke-linejoin=\"round\"/> <path d=\"M-2.8,-3.8 C-0.8,-6.8 2.8,-8.0 6.4,-7.5 C8.6,-7.2 10.2,-6.0 11.8,-5.2 C14.6,-4.4 17.4,-4.2 19.6,-3.2 C20.4,-2.8 20.4,-1.6 19.4,-1.3 C16.6,-1.1 12.2,-0.8 7.6,0.0 C5.0,0.5 1.8,1.8 -2.4,2.6 C-3.4,0.6 -3.6,-1.6 -2.8,-3.8 Z\" fill=\"#d22f1c\"/> <path d=\"M-2.6,-3.8 C-0.8,-6.8 2.8,-8.0 6.4,-7.5 C8.6,-7.2 10.2,-6.0 11.8,-5.2 C14.6,-4.4 17.4,-4.2 19.6,-3.2 C20.4,-2.8 20.4,-1.6 19.4,-1.3 C16.6,-1.1 12.2,-0.8 7.6,0.0 C6.2,0.3 4.8,0.7 3.6,1.2\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.72\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/> <path d=\"M0.6,-5.4 C3.4,-7.0 6.8,-6.9 9.8,-5.6 M13.0,-4.2 C15.2,-4.0 17.2,-3.8 18.6,-3.3\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\"0.54\" stroke-linecap=\"round\"/> <path d=\"M1.2,-6.4 C-0.6,-9.4 3.4,-11.0 5.4,-8.4 C6.2,-7.3 5.2,-6.2 3.8,-6.1 Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\"0.68\" stroke-linejoin=\"round\"/> <path d=\"M2.0,-7.0 C1.4,-8.4 3.2,-9.2 4.1,-7.9 C4.4,-7.4 3.8,-6.9 3.0,-7.0 Z\" fill=\"#6e130f\" stroke=\"#1c100a\" stroke-width=\"0.35\"/> <path d=\"M17.8,-3.2 C18.4,-3.6 19.2,-3.2 18.9,-2.5\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.5\" stroke-linecap=\"round\"/> <path d=\"M4.2,-2.2 C5.2,-0.8 5.1,0.7 4.0,1.7 M2.4,-2.0 C3.4,-0.6 3.3,0.9 2.2,1.9 M0.6,-1.6 C1.5,-0.3 1.4,1.0 0.4,2.0\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.5\" stroke-linecap=\"round\"/> <path d=\"M3.2,-4.8 h.1 M5.2,-5.4 h.1 M6.6,-3.2 h.1 M13.6,-2.8 h.1 M15.6,-2.7 h.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.72\" stroke-linecap=\"round\"/> <path d=\"M4.4,-3.8 h.1 M6.2,-4.6 h.1 M12.2,-3.2 h.1 M14.6,-3.4 h.1\" fill=\"none\" stroke=\"#f6efe2\" stroke-width=\"0.68\" stroke-linecap=\"round\"/> <g transform=\"translate(9.4,-4.1)\"> <g class=\"h-eye\"> <path d=\"M-2.2,0.1 C-1.1,-1.0 1.1,-1.0 2.3,0.1 C1.2,1.0 -1.0,1.0 -2.2,0.1 Z\" fill=\"#f6efe2\" stroke=\"#1c100a\" stroke-width=\"0.42\" stroke-linejoin=\"round\"/> <circle cx=\"0.45\" cy=\"0.05\" r=\"0.66\" fill=\"#1c100a\"/> <g transform=\"translate(0,-1.0)\"> <g class=\"h-lid\" transform=\"scale(1 0.06)\"> <path d=\"M-2.3,0.0 C-1.0,-0.2 1.0,-0.2 2.4,0.0 L2.3,1.9 C1.0,2.2 -1.0,2.2 -2.2,1.9 Z\" fill=\"#d22f1c\" stroke=\"#1c100a\" stroke-width=\"0.38\" stroke-linejoin=\"round\"/> <path d=\"M-2.2,1.8 C-0.8,2.1 0.8,2.1 2.2,1.8\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.55\" stroke-linecap=\"round\"/> </g> </g> <path d=\"M-2.5,-0.1 C-1.2,-1.6 1.2,-1.5 2.7,-0.1\" fill=\"none\" stroke=\"#1c100a\" stroke-width=\"0.82\" stroke-linecap=\"round\"/> </g> </g>"
};

// The danger mark. Two red snakes, twisted round each other like a rope (after wend's snake W),
// coil round the bubble and meet head to head beneath it. They are gummy and glossy, like the
// heart: a deeper rim, a white sheen along the lit side. They writhe (a slow liquid displacement
// over the whole rope), their forked tongues flick out, their eyes catch the light, and blood beads
// on the underside of the rope and drips. At the top, an engraved lozenge clasps the rope.
(() => {
  const NS = 'http://www.w3.org/2000/svg';
  const BEAT = '1.9s';
  const PULSE = { keyTimes: '0;.07;.14;.22;1', dur: BEAT, repeatCount: 'indefinite' };
  const el = (tag, attrs, parent) => {
    const e = document.createElementNS(NS, tag);
    for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
    if (parent) parent.appendChild(e);
    return e;
  };
  const anim = (parent, attrs, tag = 'animate') => el(tag, attrs, parent);
  const f2 = v => v.toFixed(2);

  function rr(x0, y0, x1, y1, r) {
    return `M${x0 + r},${y0} H${x1 - r} A${r},${r} 0 0 1 ${x1},${y0 + r} V${y1 - r} ` +
           `A${r},${r} 0 0 1 ${x1 - r},${y1} H${x0 + r} A${r},${r} 0 0 1 ${x0},${y1 - r} ` +
           `V${y0 + r} A${r},${r} 0 0 1 ${x0 + r},${y0}Z`;
  }

  // a rounded, gummy snake head along +x, its neck at the origin (neck half-width about 2.7)
  // the head, after wend's: a smooth bean with a domed crown, a blunt rounded snout and a throat
  // that curves back into the neck
  const HEAD = 'M-2,-2.9 C-1,-6.2 3,-8.2 7.6,-8 C12.2,-7.8 15.8,-5.5 16.6,-2.6 C17.2,-.5 16.1,1.3 13.6,1.9 C11,2.5 8,2.7 5.4,2.3 C3,1.9 0,2.7 -2,2.9Z';
  // the tongue: a long crimson stem, forked, the tines black
  const TSTEM = 'M17.4,.1 C19.6,.2 21.6,1 23.8,.8';
  const TFORK = 'M23.6,.8 C25,.5 26.2,-.4 27.4,-1.4 M23.6,.8 C25.1,1.2 26.5,2 27.8,3';
  // a beast's head as a manuscript draws it: a long snout, open jaws with small teeth, a round ear,
  // an almond eye under a heavy lid; flat colour, iron-gall ink outline, a stroke of lead white
  const INK = '#1c100a', LEAD = '#f6efe2';
  // the bestiary's heads (bestiary-heads.js, made from the Antigravity SVGs by make_heads.py):
  // static markup with h-jaw / h-tongue hooks; the vermilion is swapped for carmine where asked
  const PARSER = new DOMParser();
  const PAL = {"spotted-serpent": ["#f0b832", "#c98f1c", "#8f6312"], "aspis": ["#e0a468", "#b9783f", "#7e4d27"], "knotted-pair": ["#e08aa6", "#5b86d8", "#7a3d5a"], "draco-bipes": ["#e0952a", "#b0631a", "#6e3a0c"], "draco-alatus": ["#e0952a", "#b0631a", "#6e3a0c"], "amphisbaena": ["#4fae45", "#2f7f2c", "#1d4f1a"]};
  const palOf = kind => PAL[kind.replace(/-[ab]$/, '')];
  // every head is the perched roundel's hound (headline-heads.js), facing +x from the neck
  const DOG = () => window.BestiarySlots && window.BestiarySlots.buildTracedRoundelHead;
  function dogHead(parent, wash, wash2, scale) {
    const g = el('g', { class: 'bhead dog' }, parent);
    g.appendChild(DOG()({ wash, wash2, scale: scale || .34, dur: 7 + (uidd++ % 5) * .9 }));
    return { tongue: null, jaw: null };
  }
  let uidd = 0;
  function bestiaryHead(parent, kind, carmine, palOverride) {
    if (DOG()) {
      const p = palOverride || palOf(kind) || ['#e0952a', '#b0631a', '#6e3a0c'];
      return dogHead(parent, carmine ? '#a3141c' : p[0], p[1] === p[0] ? p[2] : p[1]);
    }
    let m = (typeof BESTIARY_HEADS !== 'undefined') && BESTIARY_HEADS[kind];
    if (!m) return null;
    const pal = palOverride || palOf(kind);
    if (pal) ['#d22f1c', '#a3141c', '#6e130f'].forEach((h, i) => { m = m.replace(new RegExp(h, 'gi'), pal[i]); });
    if (carmine) m = m.replace(/#d22f1c/gi, '#a3141c');
    const doc = PARSER.parseFromString(`<svg xmlns="${NS}">${m}</svg>`, 'image/svg+xml');
    const g = el('g', { class: 'bhead' }, parent);
    for (const n of [...doc.documentElement.childNodes]) g.appendChild(document.importNode(n, true));
    const tongue = g.querySelector('.h-tongue'), jaw = g.querySelector('.h-jaw');
    if (tongue) tongue.setAttribute('transform', 'scale(0,1)');
    return { tongue, jaw };
  }
  // tongue and jaw from one set of keyframes: k = how far out the tongue is (0..1); the jaw opens with it
  function mouth(H, kf, timing) {
    if (!H) return;
    const kt = kf.map(x => x[0].toFixed(3)).join(';');
    if (H.tongue) anim(H.tongue, { attributeName: 'transform', type: 'scale', values: kf.map(x => `${x[1]} 1`).join(';'), keyTimes: kt, ...timing }, 'animateTransform');
    if (H.jaw) anim(H.jaw, { attributeName: 'transform', type: 'rotate', values: kf.map(x => (2 + 16 * Math.min(1, x[1] * 1.6)).toFixed(1)).join(';'), keyTimes: kt, ...timing }, 'animateTransform');
  }

  // Claude's spark as a ruff: a burst of rounded rays of uneven length, turning slowly and breathing
  function sparkRuff(parent, col, scale = 1) {
    const g = el('g', { transform: `translate(3,-1) scale(${scale})` }, parent);
    const spin = el('g', {}, g);
    anim(spin, { attributeName: 'transform', type: 'rotate', values: '0;360', dur: '24s', repeatCount: 'indefinite' }, 'animateTransform');
    const breathe = el('g', {}, spin);
    anim(breathe, { attributeName: 'transform', type: 'scale', values: '1;1.12;1', dur: '3.2s', repeatCount: 'indefinite',
      calcMode: 'spline', keyTimes: '0;.5;1', keySplines: '.4 0 .6 1;.4 0 .6 1' }, 'animateTransform');
    const LEN = [11, 8.5, 12, 9, 10.5, 7.5, 11.5, 9.5, 12.5, 8, 10, 9];
    LEN.forEach((l, i) => {
      const a = i * 30 + (i % 2) * 6;
      el('path', { d: `M0,-1.25 C${(l * .5).toFixed(1)},-1.7 ${(l - 1).toFixed(1)},-1.3 ${l},0 C${(l - 1).toFixed(1)},1.3 ${(l * .5).toFixed(1)},1.7 0,1.25Z`,
        transform: `rotate(${a})`, style: `fill:${col};stroke:#3a1408`, 'stroke-width': .35, 'stroke-linejoin': 'round' }, breathe);
    });
    el('circle', { r: 2.2, style: `fill:${col};stroke:#3a1408`, 'stroke-width': .35 }, breathe);
  }
  function beastHead(parent, col) {
    if (DOG()) { dogHead(parent, col, '#6e3a0c'); return; }
    const o = { 'stroke-width': .6, 'stroke-linejoin': 'round', style: `fill:${col};stroke:${INK}` };
    el('path', { d: 'M3.6,.4 C6,0 9.5,.1 12.6,.6 L17,1.9 C17.5,2.1 17.4,2.8 16.8,2.9 C12.5,3.3 8,3.6 4.2,3.6 C1.6,3.6 -.2,3.3 -2,2.9Z', ...o }, parent);
    el('path', { d: 'M12,-1.2 L18,-1.9 L17.2,1.9 L12.4,.6Z', fill: '#4a0b08' }, parent);
    el('path', { d: 'M-2,-2.9 C-.6,-6.4 4,-8.2 8.2,-7.6 C12,-7.1 16,-5.4 18.4,-3.2 C19,-2.6 18.7,-1.8 18,-1.9 L12.4,-1.3 C9.5,-1 6,-.4 3.6,.4 C1.6,1.1 -.2,2 -2,2.9Z', ...o }, parent);
    el('path', { d: 'M13.4,-1.4 l.4,1.05 l.4,-1.1Z M15,-1.6 l.4,1.05 l.4,-1.1Z M16.6,-1.8 l.35,.95 l.35,-1Z M14,.75 l.35,-.9 l.35,.95Z M15.7,1.2 l.3,-.85 l.3,.9Z',
      'stroke-width': .15, style: `fill:${LEAD};stroke:${INK}` }, parent);
    el('path', { d: 'M5.8,-7.4 C5.2,-9.6 7.6,-10.4 8.4,-8.6 C8.8,-7.8 8.2,-7.4 7.6,-7.3Z', ...o }, parent);
    el('path', { d: 'M1,-5.2 C4,-7 9,-7.3 13,-5.8', fill: 'none', 'stroke-width': .55, 'stroke-linecap': 'round', opacity: .8, style: `stroke:${LEAD}` }, parent);
    el('path', { d: 'M9.4,-4.9 C10.4,-5.8 12,-5.8 12.8,-4.8 C12,-4 10.4,-4 9.4,-4.9Z', 'stroke-width': .35, style: `fill:${LEAD};stroke:${INK}` }, parent);
    el('circle', { cx: 11.3, cy: -4.85, r: .58, fill: INK }, parent);
    el('path', { d: 'M9.1,-5 C10.4,-6.4 12.3,-6.3 13.1,-4.9', fill: 'none', 'stroke-width': .7, 'stroke-linecap': 'round', style: `stroke:${INK}` }, parent);
    el('path', { d: 'M17.1,-3.2 c.4,-.45 .95,-.2 .8,.3', fill: 'none', 'stroke-width': .4, 'stroke-linecap': 'round', style: `stroke:${INK}` }, parent);
    for (const [x, y] of [[3, -4.6], [5.2, -5.4], [6.4, -3.8], [3.8, -2.8], [7.6, -2.6]])
      el('circle', { cx: x, cy: y, r: .32, opacity: .85, style: `fill:${LEAD}` }, parent);
  }
  const DROP = 'M0,-2.4 C1.3,-.4 1.8,1 1.8,2 C1.8,3.2 1,4 0,4 C-1,4 -1.8,3.2 -1.8,2 C-1.8,1 -1.3,-.4 0,-2.4Z';

  let uidc = 0;
  function draw(b) {
    b.querySelectorAll(':scope > svg.vine.frame').forEach(s => s.remove());  // (includes the glow layer)
    const uid = ++uidc;
    const lv = b.dataset.lv || 'high';
    const gloss = b.dataset.vine === 'gloss';
    const k = gloss ? Math.max(.46, Math.min(.62, b.offsetWidth / 200)) : lv === 'high' ? 1 : lv === 'med' ? .88 : .75;
    const W = b.offsetWidth, H = b.offsetHeight;
    const m = gloss ? 9 : 20, P = 58;
    const w = W + 2 * m, h = H + 2 * m;
    const r = Math.min(24, h / 2);
    const svg = el('svg', { class: 'vine frame', width: w + 2 * P, height: h + 2 * P,
      viewBox: `0 0 ${w + 2 * P} ${h + 2 * P}`, 'aria-hidden': 'true' });
    svg.style.left = `${-(m + P)}px`;
    svg.style.top = `${-(m + P)}px`;
    const x0 = P, y0 = P, x1 = P + w, y1 = P + h;
    const defs = el('defs', {}, svg);
    const colP = b.dataset.col ? [b.dataset.col, '#d24a1e', '#7d2a10'] : null;
    if (colP) { svg.style.setProperty('--ra', colP[0]); svg.style.setProperty('--rb', colP[1]); svg.style.setProperty('--md', colP[2]); }

    // writhing: a slow liquid displacement over the whole rope, surging with the beat
    // the gel: writhe (liquid displacement), then light it as a wet surface (a specular highlight
    // over a blurred height map taken from its own luminance, so the dark seams between the strands
    // read as valleys), then a red glow that swells with the beat
    const wr = el('filter', { id: `wr${uid}`, x: '-15%', y: '-15%', width: '130%', height: '130%', 'color-interpolation-filters': 'sRGB' }, defs);
    el('feOffset', { in: 'SourceGraphic', dx: 0, dy: 0, result: 'w' }, wr);
    el('feColorMatrix', { in: 'w', type: 'luminanceToAlpha', result: 'lum' }, wr);
    const ct = el('feComponentTransfer', { in: 'lum', result: 'lum2' }, wr);
    el('feFuncA', { type: 'linear', slope: 2.4, intercept: 0 }, ct);
    el('feComposite', { in: 'lum2', in2: 'w', operator: 'in', result: 'hmask' }, wr);
    el('feGaussianBlur', { in: 'hmask', stdDeviation: 3.8 * k, result: 'hb' }, wr);
    const sp = el('feSpecularLighting', { in: 'hb', surfaceScale: 3.4, specularConstant: .55, specularExponent: 18,
      'lighting-color': '#ffffff', result: 'spec' }, wr);
    el('feDistantLight', { azimuth: 235, elevation: 48 }, sp);
    el('feComposite', { in: 'spec', in2: 'w', operator: 'in', result: 'specIn' }, wr);
    el('feComposite', { in: 'w', in2: 'specIn', operator: 'arithmetic', k1: 0, k2: 1, k3: .7, k4: 0, result: 'lit' }, wr);
    const co = el('filter', { id: `co${uid}`, x: '-20%', y: '-20%', width: '140%', height: '140%' }, defs);
    el('feGaussianBlur', { stdDeviation: 1.2 * k }, co);
    // the gummy sheen for the heads: white at the top fading to nothing by the middle
    const hs = el('linearGradient', { id: `hs${uid}`, x1: 0, y1: 0, x2: 0, y2: 1 }, defs);
    el('stop', { offset: 0, 'stop-color': '#fff', 'stop-opacity': .9 }, hs);
    el('stop', { offset: .38, 'stop-color': '#fff', 'stop-opacity': .25 }, hs);
    el('stop', { offset: .55, 'stop-color': '#fff', 'stop-opacity': 0 }, hs);
    // engraving for the clasp: fine horizontal rules
    const pat = el('pattern', { id: `en${uid}`, width: 4, height: 1.6, patternUnits: 'userSpaceOnUse' }, defs);
    el('rect', { width: 4, height: .7, style: 'fill:var(--ro)' }, pat);

    const base = el('path', { d: rr(x0, y0, x1, y1, r), fill: 'none', stroke: 'none' }, svg);
    const L0 = base.getTotalLength();
    const arc0 = Math.PI * r / 2, top0 = w - 2 * r, side0 = h - 2 * r;
    const cornerMid = [top0 + arc0 / 2, top0 + arc0 + side0 + arc0 / 2,
                       2 * top0 + 2 * arc0 + side0 + arc0 / 2, 2 * top0 + 3 * arc0 + 2 * side0 + arc0 / 2];
    const sTop0 = top0 / 2, sBot0 = top0 + arc0 + side0 + arc0 + top0 / 2;
    const loops = [...cornerMid.map(s0 => [s0, 12 * k]), [sTop0 - 22 * k, 7 * k], [sTop0 + 22 * k, 7 * k]].sort((a, b) => a[0] - b[0]);
    const nb = Math.max(4, 2 * Math.round(L0 / 140));   // bows along the edges
    const gp = [];
    let cum = 0, last = null, sTop = 0, sBot = 0, li = 0;
    const push = (x, y) => { if (last) cum += Math.hypot(x - last[0], y - last[1]); gp.push([x, y]); last = [x, y]; };
    const step = 1.5;
    for (let sb = 0; sb < L0; sb += step) {
      const p = base.getPointAtLength(sb), q = base.getPointAtLength(Math.min(L0, sb + .5));
      const tx = q.x - p.x, ty = q.y - p.y, tl = Math.hypot(tx, ty) || 1, Tx = tx / tl, Ty = ty / tl;
      const ox = Ty, oy = -Tx;                                    // outward
      const bow = 4.6 * k * Math.sin(2 * Math.PI * nb * sb / L0);
      const x = p.x + ox * bow, y = p.y + oy * bow;
      if (Math.abs(sb - sTop0) < step / 2) sTop = cum;
      if (Math.abs(sb - sBot0) < step / 2) sBot = cum;
      push(x, y);
      while (li < loops.length && sb >= loops[li][0]) {          // throw a curl outward here
        const rho = loops[li][1], n = Math.round(2 * Math.PI * rho / 1.2);
        for (let t = 1; t <= n; t++) {
          const th = 2 * Math.PI * t / n;
          push(x + Tx * rho * Math.sin(th) + ox * rho * (1 - Math.cos(th)), y + Ty * rho * Math.sin(th) + oy * rho * (1 - Math.cos(th)));
        }
        li++;
      }
    }
    base.remove();
    const guide = el('path', { d: 'M' + gp.map(([x, y]) => `${f2(x)},${f2(y)}`).join(' L') + 'Z', fill: 'none', stroke: 'none' }, svg);
    const L = guide.getTotalLength();
    const gap = 72 * k;                         // half the space between the two heads
    const sA = sBot + gap, sB = sBot + L - gap;  // the rope runs from sA round to sB
    const at = s => guide.getPointAtLength(((s % L) + L) % L);
    const N = Math.max(110, Math.round((sB - sA) / 3.4));
    const pts = [];
    for (let i = 0; i <= N; i++) {
      const s = sA + (sB - sA) * i / N, p = at(s), q = at(s + .5);
      const tx = q.x - p.x, ty = q.y - p.y, tl = Math.hypot(tx, ty) || 1;
      pts.push({ s: s - sA, x: p.x, y: p.y, tx: tx / tl, ty: ty / tl, nx: -ty / tl, ny: tx / tl });
    }
    const len = sB - sA;
    const lam = 110 * k, R = 4.2 * k, rad = 6.8 * k;
    const near = s => Math.max(0, Math.min(1, s / 44, (len - s) / 44));  // the strands converge at the heads
    const phi = s => 2 * Math.PI * s / lam;
    // strand 0 (signal red) has its head at the end (sB); strand 1 (blood red) at the start (sA)
    const width = (j, s) => {
      const tail = j === 0 ? s / len : 1 - s / len;                     // 0 at its tail, 1 at its head
      return rad * (tail < .12 ? .35 + .65 * tail / .12 : 1);
    };
    // motion: the two snakes coil round each other (the twist tightens and turns, fading out at the necks so
    // the heads stay put), and slither: their pattern slides steadily along them. Every path is drawn at a
    // run of phases and morphed between them with SMIL.
    const wenv = s => Math.max(0, Math.min(1, s / 70, (len - s) / 70));
    const pos = (j, p, ph = 0) => {
      const tw = .9 * Math.sin(ph);
      const o = (j === 0 ? 1 : -1) * R * near(p.s) * Math.sin(phi(p.s) + tw * wenv(p.s));
      return [p.x + p.nx * o, p.y + p.ny * o];
    };
    const NF = 12;
    const PH = Array.from({ length: NF + 1 }, (_, i) => 2 * Math.PI * i / NF);
    const writhe = (e, dfn) => {
      e.setAttribute('d', dfn(0));
      anim(e, { attributeName: 'd', values: PH.map(dfn).join(';'), dur: '5s', repeatCount: 'indefinite' });
      return e;
    };
    const lit = p => (p.nx * -.35 + p.ny * -1) > 0 ? 1 : -1;            // the side facing the light

    // split the rope where the strands are furthest apart; in between, one strand is in front
    const cuts = [0];
    for (let i = 1; i < N; i++) {
      const a = Math.cos(phi(pts[i - 1].s)), c = Math.cos(phi(pts[i].s));
      if (Math.sign(a) !== Math.sign(c)) cuts.push(i);
    }
    cuts.push(N);
    if (gloss) {                                                       // a soft red glow behind the gel
      const gf = el('filter', { id: `gg${uid}`, x: '-20%', y: '-20%', width: '140%', height: '140%' }, defs);
      el('feGaussianBlur', { stdDeviation: 4 * k }, gf);
      const glow = el('path', { d: [0, 1].map(j => { const o = []; for (let i = 0; i <= N; i++) { const [x, y] = pos(j, pts[i]); o.push(`${f2(x)},${f2(y)}`); } return 'M' + o.join(' L'); }).join(' '),
        fill: 'none', 'stroke-width': 2 * rad + 7, 'stroke-linecap': 'round', filter: `url(#gg${uid})`, opacity: .45, style: 'stroke:#ff1e14' }, svg);
      anim(glow, { attributeName: 'opacity', values: '.35;.7;.45;.65;.35', ...PULSE });
    }
    const rope = el('g', gloss ? { filter: `url(#wr${uid})` } : {}, svg);
    const body = rope;
    const COL = ['var(--ra)', 'var(--rb)'], LIT = ['var(--ral)', 'var(--rbl)'], EDGE = 'var(--re)';
    const seg = (j, i0, i1, shift, ph = 0) => {
      const out = [];
      for (let i = i0; i <= i1; i++) {
        const p = pts[i], [x, y] = pos(j, p, ph), sd = lit(p) * width(j, p.s) * shift;
        out.push(`${f2(x + p.nx * sd)},${f2(y + p.ny * sd)}`);
      }
      return 'M' + out.join(' L');
    };
    // a line along strand j at a lateral offset given per point (in units of the strand's half-width)
    const line = (j, i0, i1, f, ph = 0) => {
      const out = [];
      for (let i = i0; i <= i1; i++) {
        const p = pts[i], [x, y] = pos(j, p, ph), sd = lit(p) * width(j, p.s) * f(p.s);
        out.push(`${f2(x + p.nx * sd)},${f2(y + p.ny * sd)}`);
      }
      return 'M' + out.join(' L');
    };
    const zigP = 13 * k;                                                // the adder's zigzag, along the spine
    const zig = s => { const u = ((s / zigP) % 1 + 1) % 1; return .5 * (u < .5 ? 4 * u - 1 : 3 - 4 * u); };
    // draw in two layers, back then front; within a layer the two strands never overlap, so every chunk of
    // a strand in that layer goes into one path per kind (shadow, body, lit flank, zigzag, blotches)
    const chunks = [[], []];                                           // [layer][...] = [j, i0, i1]
    for (let c = 0; c < cuts.length - 1; c++) {
      const i0 = cuts[c], i1 = cuts[c + 1], mid = pts[Math.round((i0 + i1) / 2)];
      const front = Math.cos(phi(mid.s)) > 0 ? 0 : 1;
      chunks[0].push([1 - front, i0, i1]);
      chunks[1].push([front, i0, i1]);
    }
    const sub = (list, j, mk) => ph => list.filter(c => c[0] === j).map(([, i0, i1]) => mk(i0, i1, ph)).join(' ');
    const blot = (j, i0, i1, f, ph) => {                                  // short dashes on the flank, one per zigzag period
      const out = [];
      const sFirst = Math.ceil((pts[i0].s - (f > 0 ? zigP / 2 : 0)) / zigP) * zigP + (f > 0 ? zigP / 2 : 0);
      for (let sb = sFirst; sb < pts[i1].s - 1; sb += zigP) {
        const ia = Math.max(i0, Math.min(i1 - 1, Math.round(sb / len * N))), ib = Math.min(i1, ia + 2);
        out.push(line(j, ia, ib, () => f, ph));
      }
      return out.join(' ');
    };
    const rw = rad;
    for (const layer of [0, 1]) {
      const front = layer === 1, list = chunks[layer];
      for (const j of [0, 1]) {
        if (!list.some(c => c[0] === j)) continue;
        const cap = { fill: 'none', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' };
        const body = ph => sub(list, j, (i0, i1) => seg(j, i0, i1, 0, ph))(ph);
        const off = f => ph => sub(list, j, (i0, i1) => seg(j, i0, i1, f, ph))(ph);
        const slide = (e, per, dirn) => anim(e, { attributeName: 'stroke-dashoffset', values: `0;${f2(dirn * per)}`, dur: '1.6s', repeatCount: 'indefinite' });
        const sd = j ? 1 : -1;                                          // patterns slide towards each snake's head
        if (gloss) {
          writhe(el('path', { ...cap, 'stroke-width': 2 * rw + 1.8, opacity: front ? .4 : .22, style: `stroke:${EDGE}` }, rope), off(-.25));
          writhe(el('path', { ...cap, 'stroke-width': 2 * rw, style: `stroke:${COL[j]}` }, rope), body);
          writhe(el('path', { fill: 'none', 'stroke-width': .5 * rw, 'stroke-linejoin': 'miter', opacity: .9, style: 'stroke:var(--pz)' }, rope),
            sub(list, j, (i0, i1, ph) => line(j, i0, i1, x => zig(x - 2 * zigP * ph / (2 * Math.PI) * (j ? -1 : 1)), ph)));
          writhe(el('path', { ...cap, 'stroke-width': .3 * rw, opacity: .7, style: 'stroke:var(--pz)' }, rope),
            sub(list, j, (i0, i1, ph) => [-.78, .78].map(f => blot(j, i0, i1, f, ph)).join(' ')));
          continue;
        }
        // iron-gall outline, flat body, a darker belly, a stroke of lead white along the back
        writhe(el('path', { ...cap, 'stroke-width': 2 * rw + 2, style: `stroke:${INK}` }, rope), body);
        writhe(el('path', { ...cap, 'stroke-width': 2 * rw, style: `stroke:${COL[j]}` }, rope), body);
        writhe(el('path', { ...cap, 'stroke-width': .7 * rw, opacity: .8, style: 'stroke:var(--md)' }, rope), off(-.45));
        // dark cross-bands, then rows of lead-white and dark dots, all creeping along the body
        slide(writhe(el('path', { fill: 'none', 'stroke-width': 1.86 * rw, 'stroke-dasharray': `${f2(.9 * k)} ${f2(4.6 * k)}`,
          opacity: .5, style: `stroke:${INK}` }, rope), body), 5.5 * k, sd);
        slide(writhe(el('path', { ...cap, 'stroke-width': .26 * rw, 'stroke-dasharray': `.01 ${f2(3.4 * k)}`, opacity: .9,
          style: `stroke:${LEAD}` }, rope), off(.28)), 3.4 * k, sd);
        slide(writhe(el('path', { ...cap, 'stroke-width': .22 * rw, 'stroke-dasharray': `.01 ${f2(3 * k)}`, opacity: .7,
          style: `stroke:${INK}` }, rope), off(-.12)), 3 * k, sd);
        writhe(el('path', { ...cap, 'stroke-width': .16 * rw, opacity: .85, style: `stroke:${LEAD}` }, rope), off(.56));
      }
    }

    // the heads, facing each other across the gap
    [[0, pts[N], 1], [1, pts[0], -1]].forEach(([j, p, dir]) => {
      const ang = Math.atan2(p.ty * dir, p.tx * dir) * 180 / Math.PI;
      // raise the head a little away from the bubble, as wend's heads rise off their coils
      const g = el('g', { transform: `translate(${f2(p.x)},${f2(p.y)}) rotate(${f2(ang + (Math.cos(ang * Math.PI / 180) < 0 ? 14 : -14))}) scale(${2.6 * k})` }, body);
      const sway = el('g', {}, g);
      anim(sway, { attributeName: 'transform', type: 'rotate', values: '-6;5;-6', dur: `${2.6 + j * .5}s`,
        repeatCount: 'indefinite' }, 'animateTransform');
      const flip = Math.cos(ang * Math.PI / 180) < 0 ? 'scale(1,-1)' : '';  // keep the crown on top for both
      const hg = el('g', flip ? { transform: flip } : {}, sway);
      if (gloss && DOG()) { dogHead(hg, COL[j], EDGE); return; }
      if (gloss) {                                                     // the round glossy head, big-eyed
        el('path', { d: HEAD, transform: 'translate(0,1.4)', opacity: .3, style: `fill:${EDGE}` }, hg);
        el('path', { d: HEAD, style: `fill:${COL[j]}` }, hg);
        el('path', { d: HEAD, transform: 'translate(1.6,-.9) scale(.8)', fill: `url(#hs${uid})` }, hg);
        el('circle', { cx: 9.6, cy: -3.6, r: 2.7, fill: '#0a0103' }, hg);
        el('circle', { cx: 10.5, cy: -4.6, r: .95, fill: '#fff' }, hg);
        el('circle', { cx: 8.7, cy: -2.6, r: .35, fill: '#fff', opacity: .7 }, hg);
        return;
      }
      if (b.dataset.ruff) sparkRuff(hg, j ? colP[1] : colP[0], 1.15);
      const HD = DOG() && colP ? bestiaryHead(hg, 'aspis', false, colP) : bestiaryHead(hg, j ? 'knotted-pair-b' : 'aspis', j === 1, colP ? (j ? [colP[1], colP[1], colP[2]] : colP) : null);
      if (HD) {
        const tdur = `${5.5 + j * 1.6}s`;
        mouth(HD, [[0, 0], [.6, 0], [.64, 1], [.7, .45], [.76, 1], [.82, 0], [1, 0]], { dur: tdur, repeatCount: 'indefinite' });
      } else beastHead(hg, COL[j]);
    });

    // blood beads on the underside of the rope and drips
    const nDrip = gloss ? 0 : lv === 'high' ? 3 : lv === 'med' ? 2 : 1;
    const lower = pts.filter(p => p.y > y0 + h * .6);
    for (let d = 0; d < nDrip && lower.length; d++) {
      const p = lower[Math.round((lower.length - 1) * (d + .5) / nDrip)];
      const bx = p.x, by = p.y + 3 * k;
      const t = { dur: `${3.2 + d * .9}s`, begin: `${d * 1.1}s`, repeatCount: 'indefinite' };
      const drop = el('path', { d: DROP, opacity: 0, style: 'fill:var(--rd)' }, body);
      anim(drop, { attributeName: 'transform', type: 'translate',
        values: `${f2(bx)},${f2(by)};${f2(bx)},${f2(by + 1.5)};${f2(bx)},${f2(by + 14)}`, keyTimes: '0;.55;1', ...t }, 'animateTransform');
      anim(drop, { attributeName: 'opacity', values: '0;1;1;0', keyTimes: '0;.2;.85;1', ...t });
    }

    // the clasp: an engraved lozenge at the top, ruled like a banknote, edged in oxblood
    const tp = at(sTop);
    const cl = el('g', { transform: `translate(${f2(tp.x)},${f2(tp.y)}) scale(${k})` }, svg);
    el('path', { d: 'M0,-8 L5.6,0 L0,8 L-5.6,0Z', 'stroke-width': 1.1, style: 'fill:var(--bg);stroke:var(--ro)' }, cl);
    el('path', { d: 'M0,-5.4 L3.8,0 L0,5.4 L-3.8,0Z', fill: `url(#en${uid})` }, cl);
    const core = el('path', { d: 'M0,-2 L1.4,0 L0,2 L-1.4,0Z', style: 'fill:var(--ra)' }, cl);
    anim(core, { attributeName: 'opacity', values: '.6;1;.7;1;.6', ...PULSE });

    guide.remove();
    b.appendChild(svg);
  }


  // The burrower: one slim snake slides in from the edge of the bubble, slithers along under the
  // flagged phrase, then turns up into the hole of a letter in it and is gone; after a rest, again.
  const HOLES = 'oaedgqpbODQAPRBg';

  // little storyboards for the burrowers: t is the time through the cycle, U how far the snake has come
  // (0 = not yet out, 1 = wholly into the hole); looks are head turns, flicks are tongue flicks
  const STORIES = [
    { name: 'cautious', t: [0, .14, .27, .38, .44, .62, .8, 1], U: [0, .3, .3, .52, .52, 1, 1, 1],
      look: [[.14, 0], [.17, -30], [.21, 24], [.25, -8], [.27, 0]], flick: [[.38, .44]] },
    { name: 'darting', t: [0, .07, .12, .3, .38, .56, .8, 1], U: [0, .4, .36, .36, .7, 1, 1, 1],
      look: [[.12, 0], [.14, 18], [.26, 18], [.3, 0]], flick: [[.16, .2], [.24, .28]] },
    { name: 'lazy', t: [0, .32, .46, .74, .84, 1], U: [0, .46, .46, .9, 1, 1],
      look: [[.32, 0], [.37, -16], [.43, -16], [.46, 0]], flick: [[.36, .4]] },
    { name: 'shedding', t: [0, .18, .31, .47, .56, .72, 1], U: [0, .4, .4, .72, .72, 1, 1],
      look: [[.47, 0], [.5, 22], [.54, -12], [.56, 0]], flick: [[.2, .24]], shed: { at: .4, on: .26, off: .47, gone: .66 } },
    { name: 'draco', t: [0, .22, .34, .56, .64, .8, 1], U: [0, .42, .42, .74, .74, 1, 1], draco: true,
      look: [[.22, 0], [.26, -24], [.31, 18], [.34, 0]], flick: [[.58, .63]] },
  ];
  function holeOf(mark) {
    const tw = document.createTreeWalker(mark, NodeFilter.SHOW_TEXT);
    const cands = [];
    for (let n = tw.nextNode(); n; n = tw.nextNode())
      for (let i = 0; i < n.data.length; i++) if (HOLES.includes(n.data[i])) cands.push([n, i]);
    if (!cands.length) return null;
    // prefer a letter on the bubble's last line of text, so the run beneath it crosses no other line
    const rect = ([n, i]) => { const r = document.createRange(); r.setStart(n, i); r.setEnd(n, i + 1); return r.getBoundingClientRect(); };
    const rs = cands.map(c => rect(c));
    const bb = mark.closest('.b').getBoundingClientRect();
    const lastLine = rs.filter(r => bb.bottom - r.bottom < r.height * .9);
    const pool = lastLine.length ? lastLine : rs;
    return pool[Math.floor(pool.length * .7)];
  }
  function drawBurrow(b) {
    b.querySelectorAll(':scope > svg.vine.burrow').forEach(s => s.remove());
    const mark = b.querySelector('mark.dzx');
    if (!mark) return;
    let hr = holeOf(b.querySelector('.btarget') || mark);
    const up = b.dataset.up ? b.dataset.up.split('|') : null, sc = parseFloat(b.dataset.sc || '1');
    if (up) {                                                          // the named letter of the named word
      const tw = document.createTreeWalker(mark, NodeFilter.SHOW_TEXT);
      for (let n = tw.nextNode(); n; n = tw.nextNode()) {
        const wi = n.data.indexOf(up[0]);
        if (wi < 0) continue;
        const ci = wi + up[0].indexOf(up[1]);
        const r = document.createRange(); r.setStart(n, ci); r.setEnd(n, ci + 1); hr = r.getBoundingClientRect(); break;
      }
    }
    if (!hr) return;
    const br = b.getBoundingClientRect();
    const uid = ++uidc, me = !!b.closest('.me');
    const W = b.offsetWidth, H = b.offsetHeight, P = 30;
    const svg = el('svg', { class: 'vine burrow', width: W + 2 * P, height: H + 2 * P, viewBox: `0 0 ${W + 2 * P} ${H + 2 * P}`, 'aria-hidden': 'true' });
    svg.style.left = `${-P}px`; svg.style.top = `${-P}px`;
    // the hole: the letter's counter, a little below the middle of its box
    const hx = hr.left - br.left + hr.width * .5 + P, hy = hr.top - br.top + hr.height * .6 + P;
    const yb = Math.min(hr.bottom - br.top + 2, br.height - 5) + P;                           // just under the line
    const x0 = me ? W + P + 16 : P - 16;                               // in from the outer edge
    const dir = me ? -1 : 1;
    // a slithering run along the line, then a turn up into the hole
    const pts = [];
    const runEnd = hx - dir * 9, runLen = Math.abs(runEnd - x0), waves = Math.max(2, Math.round(runLen / 34));
    for (let i = 0; i <= 60; i++) {
      const t = i / 60, x = x0 + (runEnd - x0) * t;
      pts.push(`${f2(x)},${f2(yb + 2.6 * Math.sin(2 * Math.PI * waves * t) * Math.min(1, t * 6, (1 - t) * 4))}`);
    }
    let d = 'M' + pts.join(' L') + ` C${f2(hx - dir * 3)},${f2(yb)} ${f2(hx)},${f2(yb - 1)} ${f2(hx)},${f2(hy)} L${f2(hx)},${f2(hy - 34)}`;
    let rimY = Math.min(hy, yb - 2);   // the letter's bottom edge: it burrows in from beneath, never over the glyph
    if (up) {                                                          // up from below the bubble, into the dot
      const dotY = hr.top - br.top + hr.height * .2 + P, y0u = H + P + 26, q = [];
      for (let i = 0; i <= 50; i++) {
        const t = i / 50, y = y0u + (dotY - y0u) * t;
        q.push(`${f2(hx + 5 * Math.sin(2 * Math.PI * 2.2 * t) * (1 - t))},${f2(y)}`);
      }
      d = 'M' + q.join(' L') + ` L${f2(hx)},${f2(dotY - 30)}`;
      rimY = dotY;
    }
    const cw = (b.dataset.ruff ? 22 : 13) * sc, SW = W + 2 * P, SH = H + 2 * P;
    const clip = el('clipPath', { id: `rim${uid}` }, el('defs', {}, svg));
    el('path', { 'clip-rule': 'evenodd', d: `M0,0 H${SW} V${SH} H0Z M${f2(hx - cw)},${f2(rimY - 400)} H${f2(hx + cw)} V${f2(rimY)} H${f2(hx - cw)}Z` }, clip);
    const g = el('g', { 'clip-path': `url(#rim${uid})` }, svg);
    const path = el('path', { d, fill: 'none', stroke: 'none' }, g);
    const L = path.getTotalLength(), B = Math.min(86 * sc, L * .6), wmax = 3.1 * sc;
    const POOL = STORIES.filter(x => !x.shed);                         // shedding only where a line asks for it
    const st = (b.dataset.story && STORIES.find(x => x.name === b.dataset.story)) || POOL[uid % POOL.length];
    const T = 11, cyc = { dur: `${T}s`, begin: `${(uid % 4) * .9}s`, repeatCount: 'indefinite' };
    // refine the storyboard with the moment the head reaches the hole, so the head can stop there exactly
    const hU = L / (L + B), tt = [], UU = [];
    st.t.forEach((t, i) => {
      if (i && (st.U[i - 1] - hU) * (st.U[i] - hU) < 0) {
        const f = (hU - st.U[i - 1]) / (st.U[i] - st.U[i - 1]);
        tt.push(st.t[i - 1] + f * (t - st.t[i - 1])); UU.push(hU);
      }
      tt.push(t); UU.push(st.U[i]);
    });
    const kt = tt.map(v => v.toFixed(4)).join(';');
    const ease = Array(tt.length - 1).fill('.42 0 .58 1').join(';');
    const tGoneAll = Math.min(.995, (tt[UU.findIndex(u => u >= 1)] ?? .9) + .005);
    anim(g, { attributeName: 'opacity', values: '1;1;0;0', keyTimes: `0;${tGoneAll.toFixed(4)};${(tGoneAll + .002).toFixed(4)};1`, ...cyc });
    // the body as bands from neck to tail, each its own travelling dash, so it tapers like a snake's:
    // [from, to] as fractions of the body behind the head, and the half-width there
    const BANDS = [[0, .07, .78], [.06, .5, 1], [.48, .64, .86], [.62, .76, .66], [.74, .87, .46], [.85, .95, .28], [.93, 1, .14]];
    const band = (a0, b0, attrs, dash) => {
      const len = (b0 - a0) * B;
      const p = el('path', { d, fill: 'none', 'stroke-linecap': 'round', 'stroke-linejoin': 'round',
        'stroke-dasharray': dash || `${f2(len)} ${f2(L + B + 40)}`, 'stroke-dashoffset': f2(b0 * B), ...attrs }, g);
      anim(p, { attributeName: 'stroke-dashoffset', values: UU.map(u => f2(b0 * B - u * (L + B))).join(';'),
        keyTimes: kt, calcMode: 'spline', keySplines: ease, ...cyc });
      return p;
    };
    for (const [a0, b0, wf] of BANDS) band(a0, b0, { 'stroke-width': f2(2 * wmax * wf + 1.6), style: `stroke:${INK}` });
    for (const [a0, b0, wf] of BANDS) band(a0, b0, { 'stroke-width': f2(2 * wmax * wf), style: 'stroke:var(--ra)' });
    // the darker belly along the lower flank
    for (const [a0, b0, wf] of BANDS) band(a0, b0, { 'stroke-width': f2(wmax * wf * .75), opacity: .8, transform: `translate(0,${f2(wmax * wf * .5)})`, style: 'stroke:var(--md)' });
    // dark cross-bands, travelling with the body
    const marks = [];
    for (let x = .06 * B; x < .92 * B; x += 4.4) marks.push(1, 3.4);
    marks[marks.length - 1] += L + B + 40;
    band(0, 1, { 'stroke-width': f2(wmax * 1.85), opacity: .5, 'stroke-linecap': 'butt', style: `stroke:${INK}` },
      `0 ${f2(.06 * B)} ${marks.map(f2).join(' ')}`);
    // a row of lead-white dots, and a stroke of lead white along the back
    const dots = [];
    for (let x = .04 * B; x < .85 * B; x += 3.2) dots.push(.01, 3.19);
    dots[dots.length - 1] += L + B + 40;
    band(0, 1, { 'stroke-width': f2(wmax * .5), 'stroke-linecap': 'round', opacity: .9, transform: `translate(0,${f2(-wmax * .3)})`, style: `stroke:${LEAD}` },
      `0 ${f2(.04 * B)} ${dots.map(f2).join(' ')}`);
    band(.02, .8, { 'stroke-width': .55, opacity: .8, transform: `translate(0,${f2(-wmax * .62)})`, style: `stroke:${LEAD}` });
    // shedding: the old skin loosens where it paused, the snake slides on out of it, and the husk drops away
    if (st.shed) {
      const sh = st.shed, off0 = B - sh.at * (L + B);
      const tgt = (b.closest('.stack') && b.closest('.stack').querySelector('.drape')) || b.querySelector('a.xl');
      const lr = tgt ? tgt.getBoundingClientRect() : null;
      const NP = 8, defs2 = el('defs', {}, svg);
      const wfAt = f => { for (const [a0, b0, wf] of BANDS) if (f >= a0 && f <= b0) return wf; return .3; };
      for (let i = 0; i < NP; i++) {
        const a0 = i / NP, b0 = (i + 1) / NP, sMid = sh.at * (L + B) - (a0 + b0) / 2 * B;
        const c = path.getPointAtLength(Math.max(0, Math.min(L, sMid)));
        // where this flake lands: spread across the target, or just below where it was shed
        const k = NP - 1 - i;
        const txp = c.x;                                                   // leaves fall straight down
        const typ = (lr ? lr.top - br.top + lr.height * .5 + P : c.y + 30) + ((i * 7) % 3) * 2;
        const t0 = sh.off + i * .018, t1 = t0 + .11, t2 = t1 + .05;
        const fid = `fk${uid}_${i}`;
        const f = el('filter', { id: fid, x: '-100%', y: '-150%', width: '300%', height: '400%' }, defs2);
        el('feTurbulence', { type: 'fractalNoise', baseFrequency: '1.1', numOctaves: 1, seed: 3 + i, result: 'n' }, f);
        const dm = el('feDisplacementMap', { in: 'SourceGraphic', in2: 'n', scale: 0, xChannelSelector: 'R', yChannelSelector: 'G' }, f);
        anim(dm, { attributeName: 'scale', values: '0;0;4;30;30', keyTimes: `0;${(t1 - .02).toFixed(3)};${t1.toFixed(3)};${t2.toFixed(3)};1`, ...cyc });
        const flake = el('g', { opacity: 0, filter: `url(#${fid})` }, svg);
        anim(flake, { attributeName: 'opacity', values: '0;0;.92;.92;.85;0;0',
          keyTimes: `0;${sh.on};${(sh.on + .03).toFixed(3)};${t0.toFixed(3)};${t1.toFixed(3)};${t2.toFixed(3)};1`, ...cyc });
        const dy = typ - c.y, sw = (i % 2 ? 1 : -1) * (3 + (i % 3));
        anim(flake, { attributeName: 'transform', type: 'translate',
          values: `0 0;0 0;${f2(sw)} ${f2(dy * .25)};${f2(-sw * .8)} ${f2(dy * .5)};${f2(sw * .6)} ${f2(dy * .75)};0 ${f2(dy)};0 ${f2(dy)}`,
          keyTimes: [0, t0, t0 + (t1 - t0) * .25, t0 + (t1 - t0) * .5, t0 + (t1 - t0) * .75, t1, 1].map(v => v.toFixed(3)).join(';'),
          calcMode: 'spline', keySplines: '0 0 1 1;.4 0 .6 1;.4 0 .6 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1', ...cyc }, 'animateTransform');
        const da = `${f2((b0 - a0) * B + .6)} ${f2(L + B + 40)}`, doff = f2(B - sh.at * (L + B) + a0 * B * 0 - a0 * B + B * 0);
        const w = 2 * wmax * wfAt((a0 + b0) / 2);
        const off = f2(-(sh.at * (L + B) - b0 * B));
        el('path', { d, fill: 'none', 'stroke-linecap': 'butt', 'stroke-dasharray': da, 'stroke-dashoffset': off,
          'stroke-width': f2(w + 1.4), opacity: .55, style: 'stroke:#b98a7e' }, flake);
        el('path', { d, fill: 'none', 'stroke-linecap': 'butt', 'stroke-dasharray': da, 'stroke-dashoffset': off,
          'stroke-width': f2(w + .4), style: 'stroke:#f4e3da' }, flake);
        el('path', { d, fill: 'none', 'stroke-dasharray': `1.1 1.3`, 'stroke-dashoffset': off, opacity: .35,
          'stroke-width': f2(w * .6), style: 'stroke:#b98a7e', mask: '' }, flake).setAttribute('stroke-dasharray', da);
        void doff;
      }
    }
    // a draco: a serpent with a body, after the bestiaries. A heavier torso rides the path a little behind the
    // head, on two clawed legs that walk while it moves, with a ribbed bat wing that slowly beats.
    if (st.draco) {
      const lag = .26 * B, kp = UU.map(u => Math.max(0, Math.min(1, (u * (L + B) - lag) / L)).toFixed(4));
      const on = tt.findIndex((t, i) => UU[i] * (L + B) - lag > 0), offI = tt.findIndex((t, i) => UU[i] * (L + B) - lag >= L);
      const tOn = on > 0 ? tt[on - 1] + .02 : .02, tOff = offI > 0 ? tt[offI] - .01 : .9;
      const torso = el('g', { opacity: 0 }, g);
      anim(torso, { attributeName: 'opacity', values: '0;0;1;1;0;0', keyTimes: `0;${tOn.toFixed(3)};${(tOn + .02).toFixed(3)};${(tOff - .02).toFixed(3)};${tOff.toFixed(3)};1`, ...cyc });
      el('animateMotion', { path: d, rotate: 'auto', keyPoints: kp.join(';'), keyTimes: kt, calcMode: 'spline', keySplines: ease, ...cyc }, torso);
      const tg = el('g', { transform: `scale(${f2(wmax / 3.1)})${dir < 0 ? ' scale(1,-1)' : ''}` }, torso);
      const ink = { 'stroke-linecap': 'round', 'stroke-linejoin': 'round' };
      // legs first (behind), each a red limb in an ink outline, three claws; they swing in turn
      const leg = (x, ph) => {
        const lg = el('g', { transform: `translate(${x},2.2)` }, tg);
        anim(lg, { attributeName: 'transform', type: 'rotate', values: ph ? '22;-22;22' : '-22;22;-22', dur: '.7s', repeatCount: 'indefinite', additive: 'sum' }, 'animateTransform');
        const dd = 'M0,0 C1.6,2.4 1.2,4.6 -.4,6.6';
        el('path', { d: dd, fill: 'none', ...ink, 'stroke-width': 3.6, style: `stroke:${INK}` }, lg);
        el('path', { d: dd, fill: 'none', ...ink, 'stroke-width': 2.4, style: 'stroke:var(--ra)' }, lg);
        el('path', { d: 'M-.4,6.6 L2.4,7.4 M-.4,6.6 L.8,8.4 M-.4,6.6 L-1.8,8', fill: 'none', ...ink, 'stroke-width': .7, style: `stroke:${INK}` }, lg);
      };
      leg(5.5, 0); leg(-6, 1);
      // the torso: a heavier swell of the body, belly and all
      el('path', { d: 'M-12,0 C-9,-4.6 8,-5 12.5,-1.2 C13.4,0 13,1.4 12,1.8 C6,4.6 -8,4.4 -12,0Z', 'stroke-width': .8, ...ink, style: `fill:var(--ra);stroke:${INK}` }, tg);
      el('path', { d: 'M-9,1.6 C-3,3.6 5,3.6 10.6,1.9', fill: 'none', 'stroke-width': 1.6, opacity: .8, ...ink, style: 'stroke:var(--md)' }, tg);
      for (let x = -9; x <= 10; x += 2.6) el('path', { d: `M${x},2.4 l.3,1.4`, 'stroke-width': .35, opacity: .7, style: `stroke:${INK}` }, tg);
      for (const [x, y] of [[-7, -1.6], [-4, -2.6], [-1, -3], [2.4, -2.9], [5.6, -2.4], [8.6, -1.4], [-5.4, .2], [-1.6, -.4], [2, -.6], [5.8, -.2]])
        el('circle', { cx: x, cy: y, r: .42, opacity: .9, style: `fill:${LEAD}` }, tg);
      el('path', { d: 'M-10,-1.8 C-4,-4.4 5,-4.6 11,-2', fill: 'none', 'stroke-width': .6, opacity: .8, ...ink, style: `stroke:${LEAD}` }, tg);
      // the wing: a bat's, ribbed, rooted on the back, beating slowly
      const wg = el('g', { transform: 'translate(1,-3.4)' }, tg);
      const wb = el('g', {}, wg);
      anim(wb, { attributeName: 'transform', type: 'scale', values: '1 1;1 .55;1 1', dur: '1.5s', repeatCount: 'indefinite' }, 'animateTransform');
      el('path', { d: 'M-3,0 C-5.5,-6 -1,-13 7,-13.4 C6,-10.6 7,-8.6 9.4,-7.6 C6.6,-7 5.4,-5.4 6.2,-3.6 C3.6,-3.8 1.6,-2.8 1,0Z',
        'stroke-width': .7, ...ink, style: `fill:var(--rb);stroke:${INK}` }, wb);
      el('path', { d: 'M-1,-.4 L7,-13.4 M-.4,-.2 L9.4,-7.6 M0,0 L6.2,-3.6 M-1.6,-.6 C-3,-5 -1,-10 3,-12.4', fill: 'none', 'stroke-width': .5, ...ink, style: `stroke:${INK}` }, wb);
    }
    // the head rides the front of the body and goes into the hole with it
    const tHole = tt[UU.findIndex(u => u >= hU)];
    const tIn = Math.max(.05, tHole - .045);
    const head = el('g', { opacity: 0 }, svg);
    anim(head, { attributeName: 'opacity', values: '0;1;1;0;0', keyTimes: `0;.015;${tIn.toFixed(4)};${(tHole - .004).toFixed(4)};1`, ...cyc });
    el('animateMotion', { path: d, rotate: 'auto', keyPoints: UU.map(u => Math.min(1, u / hU).toFixed(4)).join(';'),
      keyTimes: kt, calcMode: 'spline', keySplines: ease, ...cyc }, head);
    const look = el('g', {}, head);
    const lk = [[0, 0], ...st.look, [1, 0]];
    anim(look, { attributeName: 'transform', type: 'rotate', values: lk.map(x => x[1] * (dir < 0 ? -1 : 1)).join(';'),
      keyTimes: lk.map(x => x[0].toFixed(3)).join(';'), ...cyc }, 'animateTransform');
    const dive = el('g', {}, look);
    anim(dive, { attributeName: 'transform', type: 'scale', values: '1;1;0;0;1', keyTimes: `0;${tIn.toFixed(4)};${tHole.toFixed(4)};.999;1`,
      calcMode: 'spline', keySplines: '0 0 1 1;.5 0 .9 .6;0 0 1 1;0 0 1 1', ...cyc }, 'animateTransform');
    const hs = el('g', { transform: `scale(${f2(wmax * 1.05 / 2.9)})${dir < 0 ? ' scale(1,-1)' : ''}` }, dive);
    const KIND = { cautious: 'spotted-serpent', darting: 'aspis', lazy: 'amphisbaena-a', shedding: 'knotted-pair-a', draco: 'draco-bipes' };
    const kp = palOf(KIND[st.name] || 'aspis');
    if (kp) { svg.style.setProperty('--ra', kp[0]); svg.style.setProperty('--rb', kp[1]); svg.style.setProperty('--md', kp[2]); }
    if (b.dataset.col) { svg.style.setProperty('--ra', b.dataset.col); svg.style.setProperty('--rb', b.dataset.col); svg.style.setProperty('--md', b.dataset.colmd || '#9c4a2f'); }
    if (b.dataset.ruff) sparkRuff(hs, b.dataset.col || '#F2683A', 1.1);
    const HD = bestiaryHead(hs, KIND[st.name] || 'aspis', false, b.dataset.col ? [b.dataset.col, b.dataset.col, b.dataset.colmd || '#9c4a2f'] : null);
    const fk = [[0, 0]];
    for (const [f0, f1] of st.flick) fk.push([f0, 0], [f0 + .01, 1], [(f0 + f1) / 2, .4], [f1 - .01, 1], [f1, 0]);
    fk.push([1, 0]);
    if (HD) mouth(HD, fk, cyc); else beastHead(hs, 'var(--ra)');
    // the whole snake is drawn down into the hole: it shrinks to nothing about the hole's rim
    {
      const outer = el('g', { transform: `translate(${f2(hx)},${f2(rimY)})` });
      svg.insertBefore(outer, g);
      const mid = el('g', {}, outer);
      anim(mid, { attributeName: 'transform', type: 'scale', values: '1;1;0;0;1',
        keyTimes: `0;${tHole.toFixed(4)};${Math.max(tHole + .01, tGoneAll - .002).toFixed(4)};.999;1`, ...cyc }, 'animateTransform');
      const inner = el('g', { transform: `translate(${f2(-hx)},${f2(-rimY)})` }, mid);
      inner.appendChild(g);
      svg.appendChild(head);
    }
    b.appendChild(svg);
  }

  const all = () => document.querySelectorAll('.b.dzb').forEach(b => {
    if (b.dataset.vine) draw(b);
    if (b.dataset.lv !== 'high' && b.dataset.vine !== 'gloss' && !b.dataset.noburrow) drawBurrow(b);
  });
  addEventListener('refit', () => setTimeout(all, 50));
  let tid;
  window.addEventListener('resize', () => { clearTimeout(tid); tid = setTimeout(all, 150); });
  const ready = document.fonts ? document.fonts.ready : Promise.resolve();
  ready.then(all);
  // only animate what is on screen
  if ('IntersectionObserver' in window) {
    const io = new IntersectionObserver(es => es.forEach(e => {
      const s = e.target.querySelector(':scope > svg.vine');
      if (s) (e.isIntersecting ? s.unpauseAnimations() : s.pauseAnimations());
    }), { rootMargin: '200px' });
    ready.then(() => document.querySelectorAll('.b.dzb').forEach(b => io.observe(b)));
  }
})();

(() => { if (!('IntersectionObserver' in window)) return;
  const io = new IntersectionObserver(es => es.forEach(e => { const s = e.target.querySelector('svg');
    if (s && s.pauseAnimations) (e.isIntersecting ? s.unpauseAnimations() : s.pauseAnimations()); }), { rootMargin: '200px' });
  document.querySelectorAll('.beast').forEach(b => io.observe(b)); })();
(() => { const fit = () => document.querySelectorAll('.dz.cap-illum small').forEach(sm => {
  const W = sm.closest('.stack').clientWidth;
  // each line runs to the right edge of the message it captions (its frame), whatever the width
  const fb = sm.closest('.dzs').parentElement.querySelector(':scope > .b');
  const right = (fb || sm.closest('.stack')).getBoundingClientRect().right;
  sm.querySelectorAll('.ilfit').forEach(l => {
    l.style.fontSize = '20px'; l.style.display = 'inline-block';
    const r = l.getBoundingClientRect(); l.style.display = '';
    const avail = right - r.left - (l.classList.contains('il2') ? 10 : 4);
    l.style.fontSize = Math.max(12, 20 * avail / r.width).toFixed(1) + 'px'; });
  // tuck line 1 under the frame's bottom rope: its letters' tops sit just above the rope's centre line
  const d = sm.closest('.dzs'), bub = d.parentElement.querySelector(':scope > .b'), l1 = sm.querySelector('.il1');
  if (!bub) return; d.style.top = '0px'; d.style.marginBottom = '0px';
  const fs = parseFloat(l1.style.fontSize), rope = bub.getBoundingClientRect().bottom + 20;
  // the rope is ~16px thick at any width: on a wide page the big line tucks its cap tops under it; on a
  // narrow one the line is too small to lose any of it, so it clears the rope (and the heads) instead
  const capTop = l1.getBoundingClientRect().top + .32 * fs, wide = false;  // the caption clears the rope and the heads, at every width
  const dy = 0 * Math.round(capTop - (wide ? rope + 8 - .2 * fs : rope + 22));  // no tuck: the caption sits in flow, below the frame
  d.style.top = -dy + 'px'; d.style.marginBottom = -dy + 'px'; });
  const later = () => requestAnimationFrame(() => requestAnimationFrame(fit));
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(later); addEventListener('resize', later); setTimeout(fit, 1200); })();
(() => { const fit = () => document.querySelectorAll('.b[data-fit]').forEach(b => {
  const cs = getComputedStyle(b), avail = Math.min(b.parentElement.clientWidth * .97, (b.closest('article') || document.body).getBoundingClientRect().right - b.getBoundingClientRect().left - 66) - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
  b.style.width = 'auto'; b.style.whiteSpace = 'nowrap'; b.style.fontSize = '20px';
  const r = document.createRange(); r.selectNodeContents(b.querySelector('mark') || b);
  const w = r.getBoundingClientRect().width;
  b.style.fontSize = (20 * avail / w).toFixed(1) + 'px'; b.style.lineHeight = '1.25'; });
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(() => { fit(); dispatchEvent(new Event('refit')); });
  addEventListener('resize', fit); })();
(() => { const fit = () => document.querySelectorAll('.shout .sl, .hl').forEach(sl => {
  const box = sl.closest('.b'), cs = getComputedStyle(box);
  const avail = box.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
  sl.style.fontSize = '20px'; sl.style.display = 'inline-block';
  const w = sl.getBoundingClientRect().width; sl.style.display = '';
  sl.style.fontSize = (20 * avail / w * 0.985).toFixed(1) + 'px'; });
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(fit); addEventListener('resize', fit); })();
(() => { const NS = 'http://www.w3.org/2000/svg';
  const mk = (t, a, p) => { const e = document.createElementNS(NS, t); for (const k in a) e.setAttribute(k, a[k]); if (p) p.appendChild(e); return e; };
  const set = () => document.querySelectorAll('.beast.moonwalk').forEach(m => {
    const b = m.closest('.b'); b.querySelectorAll(':scope > svg.mwalk').forEach(x => x.remove());
    const art = m.querySelector(':scope > svg') || m._art; if (!art) return; m._art = art; m.style.display = 'none';
    const W = b.offsetWidth, H = b.offsetHeight, r = Math.min(18, H / 2), P = 90, w = 64, vb = art.viewBox.baseVal, h = w * vb.height / vb.width;
    const ov = mk('svg', { class: 'mwalk', width: W + 2 * P, height: H + 2 * P, 'aria-hidden': 'true' }, b);
    Object.assign(ov.style, { position: 'absolute', left: -P + 'px', top: -P + 'px', overflow: 'visible', pointerEvents: 'none', zIndex: 3 });
    const d = `M${P + r},${P} H${P + W - r} A${r},${r} 0 0 1 ${P + W},${P + r} V${P + H - r} A${r},${r} 0 0 1 ${P + W - r},${P + H} H${P + r} A${r},${r} 0 0 1 ${P},${P + H - r} V${P + r} A${r},${r} 0 0 1 ${P + r},${P} Z`;
    const g = mk('g', {}, ov);
    mk('animateMotion', { path: d, dur: '16s', repeatCount: 'indefinite', rotate: 'auto' }, g);
    const flip = mk('g', { transform: `translate(${w / 2},${-h + 7}) scale(-1,1)` }, g);
    const a2 = art.cloneNode(true); a2.setAttribute('width', w); a2.setAttribute('height', h); a2.removeAttribute('style'); flip.appendChild(a2); });
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(set); addEventListener('resize', set); })();
(() => { const al = () => document.querySelectorAll('aside.inkgloss, aside.glossa:not(.mine)').forEach(g => {
  const m = g.previousElementSibling; if (!m || !m.classList.contains('m')) return;
  const bs = m.querySelectorAll('.stack > .b'), bub = bs[bs.length - 1]; if (!bub) return;
  g.style.marginLeft = ''; g.style.marginRight = '';
  const pr = g.parentElement.getBoundingClientRect(), br = bub.getBoundingClientRect();
  g.style.marginLeft = Math.max(0, Math.round(br.left - pr.left)) + 'px';
  g.style.marginRight = Math.max(0, Math.round(pr.right - br.right)) + 'px'; });
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(() => requestAnimationFrame(al));
  addEventListener('resize', al); setTimeout(al, 1200); })();

(() => {
  for (const a of (document.querySelector('.convo') || document).querySelectorAll('a[href^="http"]')) { a.target = '_blank'; a.rel = 'noopener'; }
  const stop = ev => { ev.preventDefault(); ev.stopPropagation(); };
  for (const m of document.querySelectorAll('img[data-href]')) {
    m.addEventListener('click', ev => { stop(ev); window.open(m.dataset.href, '_blank', 'noopener'); });
  }
  const close = () => { const o = document.querySelector('.lbx'); if (o) o.remove(); };
  for (const m of document.querySelectorAll('img[data-zoom]')) {
    m.addEventListener('click', ev => {
      stop(ev); close();
      const o = document.createElement('div'); o.className = 'lbx'; o.setAttribute('role', 'dialog');
      o.setAttribute('aria-label', m.alt || 'image, full size');
      const big = document.createElement('img'); big.src = m.currentSrc || m.src; big.alt = m.alt;
      o.appendChild(big); o.addEventListener('click', close);
      (m.closest('.convo') || document.body).appendChild(o);
    });
  }
  addEventListener('keydown', ev => { if (ev.key === 'Escape') close(); });
})();


(() => {
  const NS = 'http://www.w3.org/2000/svg';
  const go = () => {
    const R = window.BestiarySlots && window.BestiarySlots.buildTracedRoundelHead; if (!R) return;
    const put = (g, wash, wash2, s, dur) => { g.replaceChildren(R({ wash, wash2, scale: s, dur })); };
    document.querySelectorAll('.beast:not(.beast-roundel) svg [id$="-head"]').forEach((h, i) => {
      const g = h.querySelector(':scope > g[transform^="scale"]'); if (!g) return;
      const mw = h.closest('.moonwalk');
      put(g, mw ? '#00c8d7' : '#e0952a', mw ? '#ff4518' : '#b0631a', .36, 7.1 + i);
    });
    document.querySelectorAll('svg.swarmfx circle.sglow').forEach((c, i) => {
      const g = c.nextElementSibling; if (g && g.tagName === 'g') { const RP = [['#1f5fff', '#0b2fa8'], ['#acae8e', '#6f7d5c'], ['#865f57', '#6d3935'], ['#b39e92', '#7d6558']]; put(g, RP[i % 4][0], RP[i % 4][1], .36, 6 + (i % 4) * .7); }
    });
  };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', go); else go();
})();

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

/**
 * Brief 6 — Slot: Knotted Pair (three places)
 * Clean cut-out of the two potrace-traced Gothic interlace dragons
 * (traced/components/gothic-initial-interlace/), rigged as 16 overlapping
 * traced segments per dragon coiling round each other and sliding along the
 * bubble's lower edge. Zero procedural tubes/ribbons. Recolourable via --wash / --wash2.
 */
(function (root) {
  'use strict';

  const GOTHIC_ART = {"leftBody":[{"d":"M196.5 22c2.1 1 3.5 3 4.5 5-1.3 .9-2.4 1.2-4 1.2l-1.2-.7-6 .7-.8-.2h-3.1l-1.7 .2c-1.5 .1-2.9 .1-4.4-.2 0 1.2 0 1.2 1 2l1.7 .2 2.7 1.5c1.5 .6 2.2 .6 3.8 .3s3-.3 4.6-.3h2.6c-1.1 1.5-2.2 1.2-4 1.3l-.7 .7c.7 .8 .7 .8 1.7 .8s1 0 1.8 .7h1.2l.3 .5 .8 .3c1.3 .3 1.6 .8 2.2 2l3 .7 1 .8 1 .1c1 .1 1 .1 1.7 .9l1.1 .3c1.1 .4 1.8 .8 2.7 1.4l1.2 .5 .3 .8 1.7 .5c1.3 .4 2.2 .7 3.3 1.5l2.5 .5 2.2 .7c1.7 .6 3 .8 4.7 .6s1.6-.1 2.6 .9c.8 1.5 1.3 2.9 1.7 4.5l.5 1c.4 1.2 .3 2.1 .1 3.3-1.2 1.2-2.5 .8-4 .7l-.3-.2h-1.5c-1.7 0-2.9-.4-4.5-1-.9-.3-1.7-.3-2.7-.4l-3.6-.4c-1.2 .8-1.2 .8-1.3 2.1l.1 1.2c-.8 .2-.8 .2-1.5-.3l-.3-2.5c-.6-3.3-.6-3.3-1-3.7l-1.1-.1c-2.2-.3-4.5-1.1-6.1-2.7l-2.8-.2-.2-.8-4.2-.2-.6-1.5-1.4-.5-1.8-1.8c-1.3-.8-1.9-1-3.5-1l-1-1.2-3.3-.3c-1.2-.2-1.2-.2-2-.7l-.9-.3c-1.2-.4-2.1-1-3.1-1.7-3.6-2.5-7.3-2.8-11.5-3-1.8-.1-3.3-.4-4.9-1h-1c-1 0-1 0-2.2-.4-1.9-.4-3.6-.5-5.5-.5l-1-.1c-2.5 0-2.5 0-3.3 .2h-2.7c-5 .1-5 .1-6.1 .8l-3.3-.1c-1.4 .1-2.7 .3-3.7 1.3l-1.6 .4c-1.6 .4-1.6 .4-2.2 1.6-1.4 .8-2.5 1-4.1 1.2-1.3 .1-1.3 .1-1.9 1.1-2.3 2.5-2.3 2.5-3.2 2.5l.2-1.8-1-.5-.2-.5c-1.5 .5-1.5 .5-2 1.3l-1.5 .5-1 1-2.3 .5-3.3 1.9c-.9 .6-.9 .6-1.4 1.6-1 .8-1.8 1-3 1.2-.3 1.7-.2 1.8 .7 3l-.2 .5-1.3-.2c-2.2 0-4.1 .1-5.4 2l-.3 1c-.5 .5-.5 .5-1.5 .7-1.6 .5-2.2 1.5-3.3 2.8l-.2 1.2c-.4 2-1.1 2.4-3 3l-.8 2.5-5-2.3c-1.6-.7-3-1-4.7-.9v1.2l-1.5 1c.3 1.7 .8 1.9 2.4 2.5 2.2 1.1 2.4 3.7 3 6 .4 1 .4 1 1.4 2v3.1c0 .9 0 .9-.6 2.2l-.1 3.8c.1 1.2 .1 1.2 1.1 2.4v1.5l.6 .3 .5-.7c.7-.6 .7-.6 1.9-.6l-.2 .5h-.5l.3 2.8c-2.2-.4-2.2-.4-2.8-1.3-1.2 .3-1.2 .3-1.8 .8l.1 1.4c-.1 1.5-.3 2.2-.8 3.6v2l-1 .5-.5 1.2-.5 .3c-.5 1.5-.7 2.9-.9 4.5-.1 1-.1 1-.6 2v5h1.2c.4 1.1 .3 2.2 .3 3.5 0 5.4 0 5.4 1.7 7.5 .2 1.8 .2 1.8-.4 2.5 .3 1.9 1.1 3.3 2.4 4.7h2.6l-.8 1.3c.1 1.5 .1 1.6 1.5 2.5 1.3 1.1 1.3 1.1 1.3 1.7l1 .5 .7 .8c2.7 1.1 4.5 1.1 7.3-.3l.2-.7 .8-.8 1.2-2.5 .1-.9c.1-.8 .1-.8 1.4-1.6l1-1.2c1.4-.6 2-.6 3.3 .2l1 .3c1.3 .5 2 1 3 2l-.3 .5-2.7 .2-.3 1c-.3 1-.3 1-1.5 2l-2.5 2.3v1.5l1.5-.3c2.5 0 2.5 0 3.8 .3 2.3 0 2.3 0 3.2-.5 4.1 .6 4.1 .6 5.5 2.7l1.7 .3 .8 .5c1.3 .4 2.4 .7 3.8 .8s2.4 .7 3.7 1.4c1.1 .3 1.9 .5 3 .5l.5-.7 1 .7c-.4 1.7-1.8 2.2-3.3 3l-2.7 1.8-1.3 .3c-1.5 .4-1.5 .4-2.2 .9-1 .7-1.8 .7-3 .8l-1.3-.8c-1.5-.4-3-.6-4.7-.6-2.2-.1-2.2-.1-3.3-.4h-2.7l-.5 1.5-1.8-1.5c-1 .7-1 1.3-1 2.5l1.3 1.3-.3 .7c-1.6 .1-1.9 .1-3.2-1-2-1-4.9-2.3-7-1.1l-.8 .6c-5.3 3.6-5.3 3.6-6.7 4.2-.7 .3-.7 .3-1 1.6h.5l-.2 .5-1-.5c-1.4 .3-1.9 .9-2.3 2.2l.5 1c-1 .3-1 .3-1.8 0-.8 .4-.6 1.2-.7 2l-1.5 .5-.5 1.8c-.9 .9-2.3 .5-3.5 .5l-1.8-.8-1-.1c-1.3-.2-1.7-.9-2.6-1.8-.6-.6-.6-.6-1.6-1.1l-.2-1.2 .7 .7c1.7 0 1.9-.6 2.7-2v-2l2.6-1.5h.4c.5-1.4 .8-2.5 .8-4l2 .5-.5-2.5 1.3-1.7c0-.8 0-.8-.8-1.3l.5-2.7c-1.5-.8-1.5-.8-2.5-.3-.6 1.3-.4 1.5 .2 2.8-1.2 2.1-3.7 2.3-6 2.6-1.3-.5-2.1-1.4-3-2.5-.7-.6-.7-.6-2.2-1.4-1.8 .7-2.1 1.3-2.5 3.1l-1.2 4c-.6 1.5-.7 3-.8 4.7l-.5 .7-.6 4.6c-.2 1.4-.2 1.4 .1 2.7s.3 1.2 0 2.5l-.3 2.7 1 .5 .2 3.5c.1 1.4 .2 3-.4 4.2-.1 2.9-.1 2.9 .8 3.8l.4 4.2-.2 .8c-.1 2.2 .2 3.9 .8 6l.4 3.2c.4 2 1 3.7 1.8 5.5l.5 1.5c.5 1.4 1 2.6 1.7 3.8l.6 1 .7 .7 1 2.3 .8-.1c1.5 .5 2 1.9 2.7 3.3 .9 1.6 1.7 2.9 3.2 3.8 .6 .7 .6 .7 .5 1.6 .1 1.3 .8 1.6 1.8 2.4l.7 .8c.7 .9 1.4 1.2 2.6 1.2l.1 .8c.7 2 2.2 3.9 4.3 4.4 .8 1.7 .8 1.7 .8 2.5 1.2 0 1.2 0 1.8-.5 1.4 .4 2.6 1 3.7 2v2.3h1.3l.6 .6c.6 .6 .6 .6 2.1 .4v.5l3.7-.2c1-.1 1-.1 1.3 .2h3.5v-.8l-2-.5 .2-1c1.7-.3 2.6-.4 3.8 .8 .7 1.1 1.2 2.2 1.5 3.5l2 .7 1 1 4 2.8c1.4 .7 1.9 1.5 2.5 3 1.4 .4 1.9 .5 3.2 .2l.3 1.3 3.6 1.2c1.7 .4 2.4 .3 3.7-1l1.3 .1c2.6-.1 3.4-2.5 4.7-4.6l.4-.7 .2-1c.1-1 .1-1 .6-1.8v-1c2.1 .2 2.1 .2 2.7 1.5l.6 2.7c.5 1.8 1.2 3 2.2 4.6v.5l3.5 1.2-.5 1.5h-1.4c-3.4 0-3.4 0-4.6 1.3l-.5 1.2-1.2 1-.8 1.5-1.2 .5-.6 2.3 2.8 .7 .2 1.8c-1 .5-2.3 .3-3.5 .3h-27.1c-2.1 0-2.1 0-2.4-.3l-.2-2.5 10-.3v-1.2h-.8c-1.2 0-1.6-.2-2.4-1l-.8-2.5c-.9-1.3-2-1.5-3.5-2l-1-1-2.3-.5-5.8-2.5c-1.3-.4-2.3-1.1-3.4-1.8-1.2-.2-1.2-.2-2.2 0l-1.6-2.5v-.5l-.7 .1c-3.1-.2-6.3-2.6-8.3-4.8l-.7-1.8c-.9-1.1-1.3-1.4-2.5-2-2.4-1.3-4.1-2.7-5.2-5.2l-1.3-1.3c-2-2-2-2-2.7-3.2l-2.5-3c-.6-.8-.9-1.6-1.3-2.5-.9-1.5-.9-1.5-1.8-1.8l-.2-.7c-.7-1.4-1.8-3.3-3.2-4l-.8-1.5-1-.9c-1-.9-1.3-1.4-1.8-2.6l-4.2-9v-1l-1.5-.3c-.3-.7-.3-.7 0-1.2 .4-2.2 .6-4.3 .6-6.6l.6-13.7 .5-4 1.3-16.4c.2-3.4 .2-3.4 .5-4.3l.1-2.2c.1-2.3 .3-4.5 .9-6.7 1.5-6.1 1.5-6.1 1.6-9.1 .2-2.6 1.1-5 1.9-7.5l3.7-15.7c.4-1.2 .4-1.9 0-3l-2-1.5c-.7-1.5-.4-2.4 .3-3.9 .5-1.4 .7-2.7 .9-4.2l.4-1.8c.6-1.5 .9-3.1 1.2-4.7l3-13.9-1.2-.5c-1.9-1.1-3-2.2-3.2-4.4-.1-1.4-.1-1.4-.6-1.9 0-.7 0-.7 .7-1.3s.8-.7 1.3-2.2l4.6-4.7c.9-1 1.8-1.6 2.9-2.3 .7-1 .7-1 1-2.2l.8-.1c.9-.2 .9-.2 1.5-.7l.4-1.5 .9-.3c1.5-.6 2.3-1.5 3.4-2.7l1-.7 .7-1.3 .9-.2c.9-.3 .9-.3 1.4-1.8 1.5-.7 1.5-.7 2.4-.8s.9-.2 1.9-1.2l1.2-.5 2.2-1.5 .6-.7c.7-.8 .7-.8 1.8-1.2 1.6-.6 2.8-1.5 4.1-2.5s2.9-1.7 4.5-2.3c2.8-1.2 2.8-1.2 3.2-2.1s.4-.9 .8-1.2l3.3 .3c2.8-.6 2.8-.6 4-1.5l1.3-.2c1.6-.3 3-.7 4.6-1.3l3.6-1c1-.3 1-.3 2-1l1.1 .1c1.2 .2 1.8 0 2.9-.6v-.8l-2.6 .2-4.6 .9-7.2 .6-25.7 .2c-7.9 .2-15.7 .4-23.5-.3-3.7-.2-8.3-.3-11.9 .9-4.7 .6-4.7 .6-5.3 0l-2-.2v-1.8h24.2l2.6 .3 6.5-.3 25.8-.1c1.2-.1 1.2-.1 1.7-.6l.2-2.8c.7-.6 2.9-.2 3.8-.2l88.2-.3zM72 120.9c-.9 .7-1 1.1-1 2.3l.5 .8-1.3 .2-1.2 2-.8 .8v3.2l-1-.2c-.8 .8-.9 1.9-1.2 3l-2.2 5.5v.8c0 .9 0 .9-.6 1.9 .3 1.4 .5 1.7 1.6 2.5 1.4 0 1.9-.2 3.2-.7l1.5-.5 .3-.5c1.2-.4 1.9-.6 3.2-.3l1.2-1.2 1.6-.8c.4-1.2 .4-1.2 .2-2.2l-.1-1.7c-.1-2.2-.6-4.2-1.1-6.3h-.6v-1.5l-.7-1.3c.1-1.2 .2-1.9 .7-3-.4-3.8-.4-3.8-2.2-2.8z","role":"wash"},{"d":"M121.1 24.2l13.1-.1c3.1 .1 3.1 .1 4.3 1.3-13.7 3.3-27 4.3-41 4.4l-20.5 .2c-4.4 0-8.9 0-13.3-.3-3.7-.3-7.2 0-10.9 .3-7 .6-7 .6-9.5 0v-1.7h4.6c25.7 .1 50-4.1 73.2-4.1zM200.2 26.5v1.2c-10.5 1.7-22 1.2-32.3-1.2l-5.2-1.1c3.2-3.3 13.1-1.6 17.4-1.7l13.2-.3c3.4 .2 3.4 .2 6.9 3.1z","role":"wash2"},{"d":"M133.9 35c-2.1 1.4-4.1 1.8-6.5 2.5l-9.1 2.5c3.9-3.9 10.3-5 15.6-5zM61.1 72.2l.6 1.7 3.5 .8c3.7 .9 3.7 .9 5.9 3.1-.1 2.3-.1 2.3-.5 5l-.4 2.6c-.9 3.7-2.4 6.8-4.1 10.2l-1.8 3.8c-2.8 6-2.8 6-4.9 6.7 1-5.8 3.5-11.1 6.3-16.2 1.8-4 2.5-8.4 3.2-12.7l-3-.7c-3.1-.9-3.1-.9-4.8-2.1zM103.3 127.8h2.3v1.1h-1.2l-.5 2.2c-1.1 1.7-1.1 1.7-2.2 1.7v1.6c-2.3 1.9-3.7 2.8-6.7 3.4 1.7-1.7 1.7-1.7 5-3.9 1.4-2 2.6-3.8 3.3-6.1zM75.6 131.7c1.6 .5 1.6 .5 2.1 3.4 .6 3.2 .6 3.2 1.7 6l-.5 1.7c-3.3-7.4-3.3-7.4-3.3-11.1zM110.6 139.4c-4.5 .7-8.9 .6-13.4 .6v-1.1c4.8-1 8.8-1.7 13.4 .5zM92.8 142.8c-1 2.9-4 3.7-6.6 5.2-5 3.1-9.5 7-14 10.9 2.2-5.4 6-7.8 10.6-11.1l3-2.2c4.2-2.8 4.2-2.8 7-2.8zM48.9 184.4h1.1c3 9.3 3 9.3 3.9 13.9-2-1-2.2-3.3-2.8-5.3l-.9-2.7c-.8-2.5-.8-2.5-1.3-5.9zM59.4 209.4c3.1 3.8 5.5 7.8 7.3 12.3h-1.1l-.6-2.3-1.7-.5-3.1-5.9c-.8-1.9-.8-1.9-.8-3.6zM106.1 241.1l2.2 2.2-3.9-1.1c-5.3-1.3-10.2-1-15.7-.6-2.6 .1-2.6 .1-4.3-1l4.1-.3c12.7-1 12.7-1 17.6 .8zM93.3 243.3l2.4 1.5c5.6 3.3 11.4 5.2 17.6 6.9l.6 1.1c-8.6-1.1-16.5-4.6-23.9-8.9 1.1-1.1 1.1-1.1 3.3-.6zM137.2 247.8l1.1 1.1c-5.9 7.5-12.2 9.7-21.6 9.4-3.4-1.1-3.4-1.1-5.6-3.9 3.3 .6 3.3 .6 6.7 2.3 6.7 .6 11.8-.7 16.1-6.1l2.8-1.2z","role":"hi"},{"d":"M164.1 25.8l2.3 .2c10.7 1.4 10.7 1.4 14 3.6l1.8 .6c1.9 .6 1.9 .6 3.8 1.7l4.1 1.1c1.2 .4 1.2 .4 2.3 1.5l3.4 .8c2.8 .9 2.8 .9 3.7 2.6l3 .8 1.1 .7 5.1 2 4.7 2.5 4.4 1.7c3.4 1.2 6.5 1.8 10.2 1.7l.8 2.2c-2.9 .2-4.4 0-6.8-1.5l-4.5-1.1-11.6-4.7c-1.1-.5-1.1-.5-1.5-1.3l-2.8-.6c-2.9-.7-4.6-1.8-7-3.5l-8.2-3.5c-1.8-.8-3.7-1.3-5.6-1.8l-2.3-1.1-2.6-.4c-1.8-.2-3.1-.5-4.9-1.1h-1.4c-1.8 0-3.3-.4-5-.8l-1.1 .4-1.5-.7-7-.5c-1.2 .1-1.2 .1-1.6 .5-2.9 .1-2.9 .1-3.4-.4l-6-.4-.4 1.1-2.2 .8v-1.1c-2 .1-3.4 .4-5.3 1.1l-3.2 .1c-3.2 .2-6.1 .7-9.2 1.4l-3.3 .5c-3 .4-5.8 1.5-8.6 2.5l-3 1c-5.9 1.9-11.3 4.6-16.2 8.5l-6.4 4.2c-8.9 5.6-18.8 12.4-24.7 21.2l-1.5 1.9-1.1-.4c.4-1.1 .4-1.1 1.5-2.3l1.1-1.8c1.9-2.3 3.7-4.3 6.1-6.1 1.4-1.1 1.4-1.1 2.2-2.6l1.8-.7 .8-1.5 1.3-.8c2-1.3 3.4-2.8 5.1-4.5l3.3-2.2 1.9-.4 .4-1.1 2.2-1.1 1.5-1.3c1.6-1.4 3-2 4.9-2.9l1.7-1.2c4.2-2.9 8.5-4.7 13.4-5.9 1.4-.4 1.4-.4 2.5-1.1l3.8-.8 6.7-2.2 6.3-1.3c3.6-1.1 7.3-1.6 11-2.1l1.9-.4h3.8l4.4-.4c5.3-.5 10.3-.1 15.6 .7zM161.6 34.1l.8 .4 3 .4c2.9 .3 2.9 .3 4.1 .7v.8c-3.1 .1-5.9 0-9-.8l-3.4-.7v-.4h2.1c2 0 2 0 2.4-.4zM133.1 36.4c-2.5 1.1-2.5 1.1-3.3 1.1l-.4 1.5c-2.3 .8-2.3 .8-3.8 .9-1.9 .3-2.3 1-3.4 2.5-1.4 1.1-1.4 1.1-2.2 1.1l.4-2.2-1.2-.4v-.7c11.3-4.7 11.3-4.7 13.9-3.8zM174.8 37.1c2.2 .4 3.9 1.1 6 2.1l6 2.5v.7c-4.7 .5-8.4-2.4-12-4.9zM111.8 44.3l-3.8 2.2-1.5 1.5-1.9 .8-.7 2.6c-1.9 0-1.9 0-3-1.1 8.8-6.7 8.8-6.7 10.9-6zM191.2 43.9c4 1.6 7.9 3.3 11.7 5.3v.3c-2.7 0-2.7 0-3.4-.7l-1.9-.2c-1.8-.2-1.8-.2-2.6-1.3l-1.9-1.1zM210.8 52.5l7.2 1.9c3.1 .9 5.9 1.3 9.1 1.4l3.9 .1-.4 1.5c-7.5 .2-7.5 .2-11.2-1.1l-4.2-.4-2.2-.7v4.1l-1.5-.4 .4-3.4-1.1-.7zM97.5 53.7v1.8l-1.2 .5c-1.9 .9-2.8 2-3.3 4l-4.5 4.9-.7-1.1c2.1-3.6 5-6.4 8.2-9 1.1-1.1 1.1-1.1 1.5-1.1zM63 67.9c.4 1.5 .4 1.5 .4 4.1 1.8 .8 1.8 .8 3.5 .6 2.8-.3 3.8 .7 5.5 2.8 1.8 5.6 .4 9.8-1.9 15l-6 15.1-1.5 2.9 .4 1.9-1.5 1.5c-1.3 2.5-2.5 4.9-3.4 7.5-1.9-7.6 5.1-18.7 8.3-25.2 6.8-14.3 6.8-14.3 3.3-19.4-2.2-.8-2.2-.8-4.7-.8s-2.4 0-3.5-1.1c-.2-2.2-.1-2.9 1.1-4.9zM79.9 109.9h.7v1.3c-.1 7.6-.1 7.6 1.9 9.6v2.3h-.7l1.1 4.1c-4.2-4.2-4.3-11.8-3-17.3zM68.6 115.2c1.7 .6 2.1 1.9 3 3.4l1.2 1.5c-.4 1.1-.4 1.1-1.6 2.2l-.7 1.9c-.7 1.9-.7 1.9-1.9 2.6l-.7 3-.8 .8-2.5 6.7c-.5 1.3-.7 2.4-.8 3.8l-.4 1.9 1.1 .3-1.1 .7c-1.6 1.6-1.8 3.1-2.3 5.3l-.6 1.6c-.6 1.7-.7 3.4-.9 5.2l-.7 2.2c-.6 4.4-.9 8.9 .2 13.2 .5 1.9 .5 3.5 .3 5.5-.2 3.7 .2 7.2 .7 10.8l.3 1.9c.5 4.1 1.1 7.7 3 11.4l.4 1.6c1.3 3.5 2.6 6.5 6 8.2l4.1 6.2c.7 .9 .7 .9 1.1 2.8 1 1.1 2 1.9 3.2 2.8 2.8 2 2.8 2 2.8 3.9h1.1c1.1 1 1.9 1.9 2.6 3.1 1.1 1.5 2 1.3 3.8 1.4l.7 1.2 1.2 .7 .4 1.5c2.6 1.6 4.9 1.7 8 1.7h2.4c1.9-.2 1.9-.2 3-.9l-1.4-1.2 .3-.7c1.6 .9 2.2 1.3 2.8 3 .6 1.5 .6 1.5 2.9 2.6l3.7 3 1.5 .8c1.1 .7 1.1 .7 2.2 3l6.1 2.1c1.9 .5 2.9 .4 4.8-.2h1.9l4.5-5.3c-.4 1.6-.9 2.6-1.8 3.9l-.8 1.3-2.7 3.1-1.1 1.8c-2.2 1.2-2.2 1.2-3.7 .8l1.1-.7c1.1-.8 1.1-.8 1.5-1.6-.9-1.8-5.8-2.4-7.7-2.8-2.1-.5-2.1-.5-3.5-1.3l-5.6-1.4c-6.6-2.1-12.6-3-19.6-2.7 .4 1.1 .4 1.1 2.3 1.9l-.4 .7c-2.3-.7-2.3-.7-3.4-2.6l-2.6-.8v-.3l5.6-.2c5.1-.5 9.6-.6 14.5 1.2 1.6 .6 3.3 .9 5 1.2-.7-2.2-.7-2.2-2.6-3h-1.9l-.3-1.5c-5.4-3.2-12.5-2.2-18.3-.7-3.7 .7-6-.2-8.4-3.1l-.7-.7c1.8 .4 1.8 .4 3 1.5 2.5 1.4 4.9 1.7 7.5 .4l.3-1.5-1.4-.8c-2.2-1.1-3.8-2.1-4.6-4.5l-7.3-7.8c-1.9-2-3.4-4.2-5-6.4l-4.6-5.8c-1.4-2.1-1.4-2.1-2.9-3.6-5.1-11.1-6.5-23.1-6.5-35.2v-2.2c0-4.9 .3-9.5 1.5-14.3l1.3-5c.8-3.3 1.6-6.3 2.9-9.4l4.2-12.1c.8-2.4 1.5-4.4 3.3-6.2 .7-2.7 .1-3.9-1.5-6.1zM111 243.5l.4 1.2h1.1l-.4-1.2zM82.9 127.2l1.1 .8h2.6c0 1.1 0 1.1-.7 2.2 1.1 1.5 1.1 1.5 2.6 2.3l1.5 2.2c-3.4-1.4-5.2-3.7-7.1-6.7zM108.4 128.7l1.1 1.1-2.6 .4-.8 2.3-3.3 3-.4 1.5 1.5-.4c2.1-.1 4.2 .1 6.3 .4l.4-.4c4 .1 4 .1 4.9 1.9l-1.4-.1c-5.9-.4-11.8-.3-17.7 .1 1.5-1.5 2.9-2.4 4.7-3.4 1.3-.8 1.3-.8 2-2.6h1.9c.8-2.3 .8-2.3 .8-3.8 .7-.4 .7-.4 2.6 0zM74.2 128.7c2.1 .8 3.8 1.9 5.7 3l2.1 1.3c6.5 4.3 6.5 4.3 6.5 8.5l1.9 .3 1.1-1.8c2.7 1 3.6 1.4 4.1 4.5l1.9-.4 .7 .4-.3 1.8c2.2 1.2 2.2 1.2 4.5 1.5l-.8 1.9 3 1.5v1.1c-2.8 0-3.4-.3-5.6-1.8-4.1-1-5.7 0-9 2.6-3.2 1.9-3.2 1.9-4.1 1.5l6.5-4.5 4-2.3c0-1.5 0-1.5-1.9-3l-.7-2.6c-1.9 0-1.9 0-3 1.1-2.3 .4-2.3 .4-4.6-.7l-.7-3.4c-1.9-3.9-6.2-5.9-9.7-7.9-1.6-1.8-1.6-1.8-1.6-2.6zM100.1 128.7h.8c-.4 3.9-1.7 6.1-5.2 7.9-1.8 .4-2.8 .3-4.6-.4l-.7-1.1 3.4 .7c4.1-.8 4.8-1.9 6-5.9zM113.6 139.6l1.9 .4c-.7 2.6-2.8 3.4-5.3 4.1-1.8-.8-1.8-.8-3-2.3l-3.3 .4-2.3-1.1h-1.5l3 2.7c1.4 1.2 2.1 2.5 3 4h-3l-.7-2.2c-1.7-2.3-3.6-3.4-6-4.9v-.7h6.4l.7 1.5 2.3-1.5c1.8 0 1.8 0 3 1.8 1.8 .5 2.4 .5 4.1-.3zM133.9 238.3c2.2 .3 2.2 .3 2.9 1.5l.4 1.5c.7 2 1.4 3.7 2.3 5.6v.8l1.9 .7 1.5 1.1-.4 1.2h-4.9l.8-2.3c-.8-2.5-.8-2.5-1.9-3l-.4-4.1-2.6 .7zM106.9 251.4l1.9 .5c4.7 1.2 9 2 13.8 1.8-1.2 1.8-2.6 1.5-4.6 1.6h-1.5c-1.4-.1-1.4-.1-2.9-1.3l-4.1-.7c3 3.7 6.9 5.8 11.6 6l1.8 .1c3.2-.2 6.1-.7 8.7-2.7h.8c0 2.2 0 2.2-1.6 3.1-5.3 2.4-13.5 1.8-18.3-1.6l-1.1-1.5-1.9-1.2c-2.6-3.2-2.6-3.2-2.6-4.1z","role":"ink"}],"greenHead":[{"d":"M220.3 17.4h1.2c2.3 0 4.1 .3 6.3 1.1l4.9-.1c4.1-.1 4.1-.1 5.3-.4l2 1c2.1 .1 3.5 0 5-1.5l4.5-.2c1.5 .2 1.5 .2 2.4 .8 1.2 .9 2.1 .6 3.6 .4l4.9-.1c1.4 .1 1.8 .2 2.6 1.3v.5c1.2-.4 1.6-.8 2.5-1.7s.8-.8 2-.5l3.3 3.3c1.4 2.4 .6 5.1-.5 7.4l-1 1c-1 1.3-1 2-1 3.5l-.3 .5c-1.2-.5-1.2-.5-2-1.2l-1.2 .5v1.7c-1 .3-1 .3-2 0-.6 .6-.5 2-.5 2.8l1.2-.3c1.9 0 1.9 0 2.5 .3-.7 1.4-1.3 1.1-2.7 1.2-1.1 .8-.9 1.3-.8 2.5l.3 1 .8-.4c.9-.3 .9-.3 1.9-.1v2l1.5-.5c1.5-.5 1.5-.5 2.3 0 .1 1.8 .1 2.7-.8 4.3-1.7 1-3.4 1.5-5.2 2-.7 .7-.8 1.3-1 2.2l-3.7 .2c-1.1 .1-1.1 .1-1.6 .3v3.6c-.3 1.4-.6 1.7-1.7 2.4l-.9 .6c-1.5 .7-3.2 1-4.9 1.2 .3-1.5 .3-1.5 .8-2.8v-.7l.9-.1c1.1-.2 1.1-.2 1.6-.7-.1-1.1-.3-1.7-.8-2.7-1.9 0-1.9 0-2.5 .2l-.2 1h-.9c-1.1 0-1.1 0-2.5 .5s-2.6 .7-4.1 .8l-2 .5 .5 2c-1.3-.3-1.3-.3-2.3-1.3-1.8-.2-2.5 .1-3.7 1.3-2.7 .9-6.5 0-9-1v-1l2.2-.5c-.2-1.3-.2-1.3-1-2.8l-.2-.8c-.5-1.6-1.5-2.9-2.5-4.2l5.7-1.2 .3-1.3 .7-.5c-.3-1.4-.4-1.6-1.7-2.2l-2.9-.1c-2.8-.1-4.5-.8-6.9-2.2v-.5l-.8-.1c-1.2-.4-2.3-.9-3.4-1.5l-2.3-1c-1.9-.8-3.3-1.7-4.7-3.1-1.2-1.1-1.7-1.4-3.3-1.3-.2-1-.2-1 0-1.7l-2-.8v-1.5c-1-1.9-2.2-3.8-3.9-5.2-1-.8-1.5-1.6-2.1-2.8 .3-1.2 .3-1.2 1.3-1.7l3.2 .3c1 .2 1 .2 1.8 .7h4.4c1.3 0 1.3 0 2.1-.3l6.7-.2c.3-.8 .3-.8 0-1.8 .8-.7 3.3-.3 4.3-.3zM210 27.7l.3 1 2.5 1c1.5 .2 2.1 .1 3.5-.7h2.7l1-.8-.2-1c-8.6-.1-8.6-.1-9.8 .5zM242.3 29.7c-.2 2.6-.2 4.2 2 5.8 1.4 .3 1.7 0 2.8-1 .9-1 .9-1 .9-1.8l-1.5-.2 .5-.9c.5-1.1 .5-1.1 0-2.1l-2.2 .5-1-1c-1 0-1 0-1.5 .7z","role":"wash"},{"d":"M267.2 20c1.1 2.2 1.1 2.2 0 3.9h-1.1v1.1l3.9-1.7c-.7 4.2-1.5 6.2-5.6 7.8-3.6 1.7-6.7 3.8-10 6.1l-2.9 1.8c-2.9 1.8-5.1 3.1-5.9 6.6l1.1 2.7-1.1 .6v-1.7l-1.7-.5c0-4.4 1.4-5.5 5-7.8l-.6-1.7 2.3-1.1v1.7h1.1l-.6-2.2 3.9-.6 1.7-1.7-.6-1.1 3.9-.5 1.1-2.3 5-.5 1.1-1.7-2.8-.5v-1.7l-5 1.7c.6-2.3 .6-2.3 4.5-3.9v1.1c1.7-.6 1.7-.6 2.8-3.3zM211.1 30.6l3.3 .5h4.2c4.2 0 4.2 0 7 1.1-3.1 3.1-10.6 .9-14.5 0zM210.6 34.4l2 .8c9.7 3 9.7 3 14.6 2.6l1.1-1.1c1.1 2.2 1.1 2.2 1.1 3.9l-7.1-1.5c-10.3-1.8-10.3-1.8-11.7-4.7zM265 45.6v.5c-5.3 .1-5.3 .1-7.8-.5-3.3-.2-6.6 .1-10 .5 5.4-4 11.8-1.9 17.8-.5z","role":"hi"},{"d":"M225 17.6l1.5 .6c5 1.5 10.5 1 15.6 .8h1.6c1.2-.2 1.2-.2 1.9-1.7l.8 .7c-1 1.9-1.9 2-3.8 2.6l-.7 .8-1.5-.8v1.2l-2.6-.4-.4 .7h-.8v-.7c-2.1-.2-4.2-.4-6.3-.3-3.4-.1-5.9-1.1-8.3-3.5 .8-.7 2.1-.1 3 0zM250.5 17.3l1.9 1.1c2.2 .9 3.3 .7 5.6 .4 3.6-.5 3.6-.5 4.9 .3l.3 1.2 4.9-1.9c2.7 3.4 4.1 5.5 2.3 10-.8 1.2-.8 1.2-1.5 1.6l-.8 1.5h-1.1v.8l1.1-.4v1.9c-1.1-.4-1.1-.4-1.5-1.9l-1.8 1.1c-.8 .4-.8 .4-1.9 0-.4 2-.5 3.9-.6 6l-.2 3.8h.8v1.5l1.5-.8-.4-1.5 1.5-.3v1.8l3.4-.7c.3 .4 .3 .4 .3 3l-.7 .4v-2.3l-1.4 .6c-4.2 1.2-7.3 .3-11.3-1-2.6-.2-4.4 .3-6.8 1.2-1.9 0-1.9 0-2.2-.4 0-3.4 3.3-4.5 6-6.1 2.3-1.4 4.5-2.8 6.6-4.5 1.9-1.3 2.8-1.7 5-2.2 3.8-2 5.5-3.9 5.6-8.2l-3 1.5c.8-1.5 .8-1.5 1.9-2.3l-.4-2.2c-2.4 0-3.9 .5-5.6 2.2l-3 .4-.8 .7c-5.2-.3-5.2-.3-6.7-3l-.8-1.8-1.5-.8zM253.9 21.4l.3 1.1c4.8 .2 4.8 .2 6-1.1v-.8c-2.4-.5-4-.1-6.3 .8zM258.8 35.6v1.2h-2.3l-1.9 1.8-2.6 1.2-2.2 1.9 .3 1.5c4.1-.7 4.1-.7 5.3-1.9l.7 .7c1.9 .8 1.9 .8 4.1 .4v1.1l1.2 .4 .4-1.5-1.6-.4 .4-6.7zM216.4 18h.7v.8h.8l.3-.8 2.7 2.6-1.1 .4-1.9-1.9 2.1 3.2c.9 1 .9 1 2.8 2.1l.3 .7 1.5-.3 .8 1.1 2.2-.4 1.5 1.5 .4-1.9 3.4 .8v.7l-1.5 .4c.4 .4 .4 .4 1.1 .4-.6 2.2-.6 2.2-2.3 2.6v1.9l-2.2-.8-5.1-.3c-2.4-.1-4.5-.1-6.9 .3-3.5-.3-6-1-8.6-3.3l-6-4.7c-1.2-1-1.2-1-3-1.3l2.8 2.7c1 1.1 1.7 2.2 2.5 3.5l2.5 3.9v.7c-1.4-.3-1.4-.3-2.2-1.8l-1.1-.8-1.5-3c-1.3-1.8-2.6-3.3-4.2-4.9-.7-1.1-.7-1.1-.7-2.6l1.9 .4h17.6zM213 21.8c-2 .5-3.9 .4-6 .4l-4.1-.1c0 2.2 .8 2.3 2.5 3.5l1.4 1c1.3 .8 1.3 .8 3.2 1.2l.4 1.1c2.1 .7 3.2 .8 5.3 .7l5.5-.2c4.5-.1 4.5-.1 5.3-1.6l-1.7-.3c-1.7-.5-1.7-.5-2-1.6l-2.7-2.3-.8-1.2c-1.8-1.7-4.1-1.4-6.3-.6zM213.4 34.1l1.4 .2c2.5 .3 4.9 .3 7.3 0 1.4-.2 1.4-.2 2.9 .2l.8 2.3-1.6 .7-.7-.7 .7-.8h-1.4l.3 1.5c-4.2-.6-4.2-.6-6.3-1.5l-3.8-.7zM214.1 37.5l10.3 1.6c1.7 .3 1.7 .3 3.2 1.4h2.3l1.1 1.9c.8 1.1 .8 1.1 1.9 1.5l-.4 2.6c-1.1 .4-1.1 .4-2.3 0v1.2l.8 .3c-2.1 1.7-2.7 1.8-5.2 1.5-1.9-1.1-1.9-1.1-2.3-1.8l4.6-1.1c1.4-.4 1.4-.4 2.1-1.6-.3-1.5-.3-1.5-1.1-2.2l-4.4-.2c-1.7-.2-3.8-.4-4.6-2.1l-2.8-1.1c-2.8-1-2.8-1-3.2-1.9zM228.8 47.7v1.1h.7v-1.1zM268.9 46.5c-.8 2.2-3.9 2.5-6 3l-.4 2.3h-1.1l-.8-2.3c-1.8 .8-1.8 .8-2.6 1.9-1.1-1.9-1.1-1.9-.8-3.4h.8v.8l1.8-.6c2.2-.5 4.2-.7 6.4-.9 1.9-.8 1.9-.8 2.7-.8zM235.9 49.2l-1.5 5.6-1.5-1.9-1.2-.7c-1.1-.8-1.1-.8-1.5-2.7h1.2l.4 1.5zM248.2 50.3h.8l.4 .7 2.6 .8c-.4 1.1-.4 1.1-2.2 1.5l-.8 .7-2.2-.3c0-1.5 0-1.5 1.1-2.7zM235.5 54h2.6l.4 1.5 .7 .4v.8l-1.7 .1c-2 .2-2 .2-3.5 1.4-1.6 .1-3 .1-4.5-.4l-4.1-.4-.4-1.5 9.4-.4 .4-.7 .7 .4z","role":"ink"}],"leftWing":[{"d":"M60.5 67.7c1.5 .5 1.5 .5 1.8 1.3l.3 2.4c.2 1.3 .2 1.3 .6 1.6l3.2 .2c.8 0 .8 0 1.1 .3 0 2.4-.4 4.4-1.2 6.7l-3 13.4-.9 3.3c-.5 1.3-.6 2.6-.7 3.9-.2 .9-.2 .9-.9 2.2-.3 1.1-.3 2.1-.2 3.3 0 1.8-.4 3.1-1 4.8-.8 2.7-1.3 5.5-1.8 8.2-.5 2.4-1.1 4.7-1.9 7s-1.2 4.5-1.3 7l-2.4 11.6-2.5 29.9-.4 6.5-.3 1.9c-.1 1.9-.1 1.9 .2 2.5l-.5 7.1c-.1 1.4-.2 2.6-.7 3.9l1 .7c0 1.3 0 1.3-.5 2.3-.5 2.5-.5 2.5-.3 3.7l-.2 .5c0 1.5 0 1.5 .5 2.8 .7 3.7 .3 7.8 0 11.6v1c-.1 1.5-.3 2.7-.7 4.1l-.1 1.3c-.3 2.6-1.4 5-3.7 6.5l-1.5 .2 .6-2.7c.2-1.6 0-2.6-.6-4l.1-3.5c.1-2.2 .2-4.3-.2-6.5-.3-1.9-.3-3.8-.3-5.7-.1-1.3-.1-1.3-.6-2.8v-101.8c0-6.2 0-6.2 1.5-8.9l1.9-4.2 1.1-3.1c.6-1.5 1.2-2.7 2.5-3.7l.2-1.2c.3-1.1 .3-1.1 1.5-1.6l.1-.8c.2-.9 .2-.9 .9-1.4 1.5-1 1.9-2.7 2.6-4.3l.7-1 .3-.7 .6-.4c1.5-1.1 2.6-2.4 3.4-4.1z","role":"wash2"},{"d":"M52.2 81.1c1 2.1-.9 4.1-1.9 5.9l-1.4 2.6c-1.1 2.1-1.1 2.1-1.7 2.1 0-3.9 2.2-7.8 5-10.6zM60 103.3h.6c.3 4.5-.9 7.2-2.8 11.1-2.4 5.8-4.2 11.7-6 17.7l-4.6 17.3-1.1 12.4c-2.2 16.1-2.2 16.1 0 20.4l.6 9.2c.2 4.9 .5 9.8 1.1 14.7 .2 8-.2 15.8-2.8 23.3h-1.7l1.1-3.4c1.6-5.1 1.8-9.6 1.8-14.9v-2.3c0-4.7-.4-9.1-1.2-13.8l-1.7-18.3c0-21.1 3.8-40.5 11.7-60l3.3-9.5h.6z","role":"hi"},{"d":"M60.8 67.5c.5 1.8-.5 2.4-1.6 3.8l-.8 .8c-1.2 1.3-2 2.6-2.9 4.1l-2.3 3.2c-3.3 4.3-5.9 9-7.8 14l-3.4 8.7h-.4c-.3-4.2 1.6-7.7 3.4-11.3l1-2.4c.9-2.1 .9-2.1 2.4-4l1.1-2.6c.8-1.3 1.5-2.3 2.6-3.4l.8-1.7c1.2-2.3 2.7-3.9 4.5-5.8zM56.6 78c.6 1.7-.1 2-1.1 3.4-1.8 3.2-3.3 6.5-4.8 9.8-.8 1.5-.8 1.5-2.7 1.9 1.4-5.6 5.2-10.5 8.6-15.1zM56.2 84.4c.7 1.3-.3 2.3-.9 3.5l-4.3 8.9h-.8l-.3 1.1c-1.1-2.1 1.8-6.2 3-8l1.1-1.3zM47.6 93.1c1.2 1.1 .9 2.1 .8 3.7-.8 1.1-.8 1.1-1.5 1.5 .3 1.5 .3 1.5 1.5 1.9l.4-1.1h.7l-2.5 6.7c-2.3 5.6-3.9 11.4-5 17.3-1.4-2.1-.9-3.9 0-6l1.1-4.2h-.7l-.8 1.5c-.5-1.6-.4-3.1-.4-4.8 1.9-.4 1.9-.4 2.7 0v-.8l-1.5-.4c-1.6-2.4 .1-5.4 1.1-7.8l1.9-3zM46.1 100.2c-1.5 3.6-1.5 3.6-1.1 4.9 1.5-.8 1.5-3.4 1.9-4.9zM43.9 106.6l.3 1.1 .8-.4v-.7zM60.8 94.6c.5 1.6-.1 2.3-.8 3.8l-4.8 11.5c-.8 2.3-.8 2.3-1.6 2.7 .4-2.6 1.4-4.8 2.3-7.2l3-7c1.4-3.4 1.4-3.4 1.9-3.8zM49.5 109.9c.5 2-.1 3.2-.7 5l-5.7 23.2-1.5 .4 2.2-11.5c1.3-5.8 3.5-11.5 5.7-17.1zM58.5 113.3h.4c-.6 5.5-2.1 10.9-3.8 16.2l-2.4 7.4c-2.9 8-4.4 16.4-4.3 24.8l-.4 1.9-.2 6.5c-.1 3.5-.1 3.5-.6 4.8l.4 .7c.4 3.6 .5 7.2 .6 10.8 .1 3.4 .1 3.4 .6 4.2 .1 4.2 .1 4.2-.8 6.4h.8l-.4 6.4-.8 .4-.6-7.4c-.3-4.9-.6-9.8-.5-14.7l-.2-6.5c-.2-4.8 0-9.4 .3-14.2l1-12.8h.8l.1-1.3c.8-6.1 2.4-12 4.4-17.8l.7-2.7c.8-3.3 2.1-6.3 3.4-9.3zM52.9 113.7h.7l-3 10.2c-1 3.6-1 3.6-1.8 4.4 0-2.4 .4-4.1 1.2-6.4zM48.4 128.3h.4c-.2 2.6-.6 4.8-1.5 7.2-1 3.4-1.2 7-1.5 10.5h-1.2c.2-3.9 .9-7.5 1.9-11.3zM45.4 147.1c0 2.5-.1 4.9-.6 7.4-.5 3.3-.8 6.5-.9 9.9l-.1 2.4c-.6 13.4-.6 13.4 .1 14.1l.7 6.3c.6 6.2 .9 12.3 .9 18.6v4c0 4.5-.1 8.7-1.3 13.1l-1.4-1.5 1.1-.4 .3-5.4c.2-4.7-.1-9.4-.3-14.1l-.4-9.9c0-2.3-.2-4.5-.5-6.7-.6-4.5-2.8-37.8 2.4-37.8zM41.6 200.8c1.6 1.5 1.2 3.6 1.1 5.7 .1 2.9 .2 5.7 .8 8.5l-1.1 1.1c-.4-2.5-.5-5-.7-7.5l-.5-3c0-3.9 0-3.9 .4-4.8zM47.6 206h.8c1.5 7.8 1.3 18.8-3.8 25.1l-2.2 .4v-.7l2.2-.4 1.7-5.5c1.7-6 1.4-12.7 1.3-18.9z","role":"ink"}],"rightBody":[{"d":"M163.5 36.2l2.5 .2c8.1 .4 8.1 .4 11.5 2.8l2.3 1.5 1.8 .9c1.2 .4 1.2 .4 1.6 .9l1-.2c1-.1 1-.1 2 1.2l-1.2 .2c.5 1.3 .5 1.3 1.5 1.8l.7 1.5 1 1 .8 1.2 3 3c1 .8 1.5 .9 2.8 1l.4 1c1.4 .5 2.1 .5 3.6 .3 .4-.5 .4-.5 .8-1.4 .5-1.1 .8-1.5 2-1.8l2.2-.3 1.4 .5v.5l-1.3 .2c-1.4 .3-1.4 .3-1.9 1.5l-1.8 .5 .6 1.5c.4 1.5-.6 2.7-1.5 3.8s-.8 1.2-.8 2c2.7 0 2.7 0 3.3-.3-.3-1.2-.3-1.2-.8-2.2l2 .5c.8-.8 .8-.8 1-2l-1.2-2.8c.7-.5 .7-.5 2.2-.5l.5-2.2 3.1 .3c1.2 .2 1.2 .2 1.9 .7 0 1.5 0 1.5-.7 2.5l.4 1.5c.2 1.6 .2 2.5-1 3.7l-2 1-1 1-1.4-.2c.2 1.5 .2 1.5 1.4 2l2.3 1 1.2 .1c1.4 .2 1.8 .4 2.8 1.4 1.6 1 2.7 1.3 4.5 1.5l.8 1 1.7 .7 1 1 1.1 .3c1.2 .2 1.2 .2 2.2 1.5l4.2 2.5 .8 .9c1 1.1 1 1.1 1.9 1.6 1.8 1.2 2.9 3.1 3.3 5.2l1.8-.2 .3 .9c.8 1.4 1.7 2.1 3.1 2.6l1.2 2.4c.9 2 2.2 3.7 3.5 5.5s1.3 1.9 1.3 2.8h1.6c.2 .8 .2 .8 0 2.3 .7 1.2 .7 1.2 1.7 2.2 1-.2 1-.2 2-1v-2.2l.3-1c.3-1.6 .5-3-.3-4.5l-.1-2.7c.1-.8 .1-.8 .6-1.6 .1-3-.2-5.5-1.8-8.2v-.9c-.4-1.5-1.7-2-3-2.9l-.4-.5c-2.4-.4-2.4-.4-3.7 .2-.8 1.1-.9 2.2-1.1 3.6l-2.5 .5c-.5-1.9-.7-4.4 .6-6 2.5-2.2 4.7-2.5 7.9-2l1.2 .5 2-.3 .6 1.3 .7 .4c1 .8 1.4 1.7 2 2.8l1.7 3.3h.6c.2-1 .2-1 0-1.5l.1-3.6c.1-.9 .1-.9-.1-1.9l.4-1.3h.6v9.8l-1 .5c.1 1.3 .1 1.9 1 3l.1 79.5c-.1 2.1-.1 2.1-.9 3 0 1.5 0 1.5 .8 2.2v44.7c-.6-.2-.6-.2-.7-1.8l-.4-3.2c-.4-2.4-.3-4.9-.2-7.3v-1.5c0-6.9 0-6.9 .5-7.9 .1-1.8 0-3.3-.5-5-.3-2.5-.2-4.8 0-7.3h-.7c-1 1.7-1.4 3.4-1.8 5.3-.8 2.2-.8 2.2-1.5 3.2l-.1 .9c-.2 .9-.2 .9-1.6 1.6l-.6 1.5-3 6.1c-.7 1.8-1.5 3.4-2.7 4.9l-.1 .9c-.2 .9-.2 .9-1.4 1.9l-.2 1.2-1.3 .8-2.5 3.5-.2 1.2-1 .8c-1 .7-1 .7-1.6 2.2l-.7 1-1.8 6.6c-.4 1.6-.5 2.9-.5 4.4l-.2 1.5 .7 .8c1 1.3 1 1.9 .8 3.5-.8 1.6-1.4 2.3-3 3.2l-2.3 1.8c-.7 .5-.7 .5-2.2 1-.5 .5-.5 .5-.5 2-1.2 1.1-1.2 1.1-2.7 .9-1.7 0-1.7 0-2.5 .9l-.5 .9-1.8 1-3.5 3.5c-1.7 1.7-2.2 2.6-2.3 5l1 .3 1 1.5c-1 .2-1 .2-2-.8-1 0-1.3 .5-2 1.3l-.2 .7 18.2 .3c0 2.2 0 2.2-.4 2.7l-89.1 .1c-2.4 0-2.4 0-3.2-.6v-1.7c2.7-1.4 7.3-.8 10.3-1l2.7-.3c1.7-.1 3.1-.3 4.7 0l-.1-.8c-.1-.9-.1-.9 .1-1.4l1.2-.4c1.7-.8 3-1.8 4.4-3 2-1.6 3.4-1.8 6-1.9 .7-.2 .7-.2 1-1.2 1-1.1 1.6-.9 3.1-.9 2.2 0 2.2 0 3.3-.9h4.3l2.5 .3 .9-.6c1.1-.7 1.1-.7 2.1-.5 1.5 .1 2.5-.3 3.9-.8l3.9-1.6c2.6-1.1 5.9-2.7 8-4.8l1.2-.5 1.2-1.2c.7-.7 1.2-1.2 2-1.7 1-.7 1.4-1.5 2-2.6l1-1 .8-1.2 .7-.7c.5-.6 .5-.6 .8-1.6l1.3-.2 1-1.5 1-1 1.3-2.8c.7-1.2 .7-1.2 1.1-1.5l1.3-2.5c.8-1.2 1-2.2 1-3.7l-.7-.8c.2-1.4 .5-1.6 1.7-2.5l.7-.2-.2-2.5-1.5-.3 .3-3.2 1-.5-.3-3.6c0-1.7 0-1.7-.7-2.4l-.2-5.3c0-2.1 .1-3.4 1.2-5.2 .6-2 .6-4.1 .7-6.2 .2-3.7 .2-3.7 .5-4.3l.3-4.2c-.1-1.3-.1-1.3-.5-2.6-.3-1.7 .2-2.6 1.3-4 1.4-1.8 1.4-1.8 1.7-3.2 .4-1.6 .7-2.5 1.7-3.7l.6-2.5c.1-.8 .1-.8 1.1-1.8l.8-1.5 1-1.2-.5-2.5 .7-.4c.8-.7 1.4-1.3 2-2.1l1.3-.8c1-1.2 1.7-2.2 2.4-3.6s1.3-2.3 2.3-3.4l.8-1.7 .5-.3v-1.5c.4-1.6 .8-2.8 1.7-4.2l.8-2.8c.1-2.5 .1-2.5-.2-3.7l.2-.8c.9-3.1 .7-8.2-.8-11v-3l-.7-.7v-2.3c-.3-1.1-.6-2-1.3-3-.8-1.3-1.2-2.7-1.2-4.2l-.8-.3v-1.2l-.7-1-.5-1.3c-.7-2-1.5-3.3-3-4.7l-2.6-3.7c-.9-1.1-1.8-1.9-2.9-2.6-.7-.5-.7-.5-1.5-1.7l-.8-.5c-.7-.5-.7-.5-1-1.3-.9-1.8-2.9-2.7-4.6-3.6l-2.2-1.5c-1.3-1-2.3-1.2-3.9-1.4l-.3-.7-.8-.1c-.9-.2-.9-.2-1.6-.9l-3.1-1.3c-.9-.2-.9-.2-1.9 .5-2.8 .2-2.8 .2-4-.7l-2-.3c-1.8-.2-1.8-.2-2.3-.5l-3.8 .1c-1.2-.1-1.2-.1-2.7-.8-3.4-.9-9-1.5-11.8 1.2h-3.2c-1.3 .3-2.3 .8-3.5 1.5-1.6 .9-2.9 1.6-4.7 1.8-1.4 .7-1.6 1-2.3 2.5l-1.5 .7-2.7 2.7c-.8 .8-.8 .8-1.3 1.8l-1.2 .5c-1.3 1.8-1.8 3.4-1.3 5.5l1 2 .3 1.8 .4 1 .8 8.4c0 2.8 .6 5 3 6.6l-.5 1-2 .2c0 2.4 0 2.4 .3 3.5l-.1 1.1c.1 1.2 .1 1.2 .5 2.2-.5 .5-1.3 .3-2 .2l-.7-.5h-2.5c-.1 3.7-.1 3.7 1 5l.2 1 2 .8-.7 4 .5 .2v.5l-1.5 .3-2.3-3.5c-.7-1.5-.7-1.5-.7-2l-.7-.3c-1.6-2.5-2.4-4.9-1.3-7.7l.3-4.5h-.6v1.7l-1 .5c-.7-.7-.7-.7-.7-2.2l-.3-1c.6-1 .6-1 1.8-1.3 .6-1.2 .2-2.3-.2-3.5 .1-1.4 .3-2.7 1-4-.1-1.7-.5-2.9-1.3-4.5l-1.6-4.6-1.7-3.6-.4-.5-.8-1.8c-.2-.2-.2-.2-1.5-.2-1.7 0-2.2-1.3-3.3-2.5-.2-1-.2-1 0-2.3l-2.2-1.7-2.2-1.2c-2.4-1.5-4.9-3.1-7.8-3.3l-.8 .5-.5-.6c-1.5-1.4-2.4-1.6-4.5-1.7l.2-.8c.1-.9 .1-.9-.2-1.7l.2-.8c.1-.9 .1-.9-.4-2.2l.2-3 1.6-.2c.7-3.6 .7-3.6-1-5.8l-.8-2.5-.8-1.5c-.1-1.3 0-1.6 .8-2.7v-1c-1.8-.6-1.8-.6-3.2 .2l-1.3 2.3-.7 .7-.2 .8c-.7 2-2.3 3.3-4.4 4l.1 .8c-.5 1.7-2.2 2-3.9 2.5l-4.6 .4c-.6-1-.6-1-.3-1.5l-.7-1.2-2 .2c-.5-4-.5-4 .4-5.7 1.3-.4 2.5-.4 3.8-.5 1.3 1.7 1.3 2.7 1 4.7l-.2 1.8c1.1 1.2 1.4 1.1 3 1l.7-.3-.3-1h-1.7l.3-1 2-.2-.8-1.8 .5-.2c.5-1.5 .5-1.5 .5-3l-1.8 .2c0-1.2 0-1.2 1.3-2.2 1.3 .2 1.3 .2 2.3 1 1.4-.3 1.4-.6 2.2-1.8l.8-.2 .7-2 .7-.8 .8-1.5c1.1-1.3 2.2-2 3.8-2.7l.4-.8 1.8-.5 1-.7h.5l.5-1c1-1.3 1.9-1.5 3.5-1.8l2.3-.5c1-.2 1-.2 1.4-1.5l3.3-.7 .7 1.5 3.3 1 .8-.1c1.3 .1 2.1 .7 3.2 1.4l2.3 1.2 .7 1.7 .7 .8c.4 1 .8 2 .9 3 .3 1.7 .7 3.8 2.1 4.7h1.6l.2-1.2c.2-1.3 .2-1.3 .8-1.8l2-4.3c.6-1.3 1.3-2.7 2.7-3.2l.3-1 2.4-.7 .8-.8 1.5-.7 .7-2c.8-.5 .8-.5 2.3-.3zM166.2 43.2h-1.4l-2.8 1.8c-2 1.2-2 1.2-2 2.2l-1 .5c-1.6 2.9-1.6 2.9-1.5 4.8l-.7 1.5c-.4 3.4 .3 7.4 3 9.7 1 .8 1 1.6 1.2 2.8 2.3 0 3.9-1.5 5.5-3 .3-.8 .3-.8 0-1.5l.7 .5 2.6 .2c.2-1.4 0-1.7-.6-3-.2-1.2-.2-1.2 .6-2.7l2.4-.8v-1.5l1 .3c1.7 .1 2.5 0 3.6-1.3l.2-2c-1-.5-1-.5-2 0v-1.2l-.8-1 1-.3c.6-.5 .2-1.6 0-2.2l-.4-1.5-1.3-.3-.5-1-1.8-.2-.4-.8c-1.7-.5-3-.7-4.6 0zM129.2 46.5l-.4 1.5c-.6 .5-.6 .5-1.4 .3-1.2-.1-1.4 .2-2.2 1.2-.6 1.2-.8 2.1-.7 3.5l.5 .7 .2 1c.3 1 .3 1 1.6 1.5l.4-.2v-.5l1.6 .2 .7-.2v1.2c0 1.3 0 1.3 .7 2 1-.5 1-.5 1.6-1.7l.7-.8 .5-1 1.2 .3 1.3-1 1.3 1.2 .7-.2c.3-1.3 .3-1.3 0-2.5l.5 .2v1.3c.8 .7 .8 .7 1.8 .5l.2-1 1-.5c.5-1 .3-1.8-.2-2.8-1.6-2-1.6-2-2.8-2l-.5-1.5c-2.1-1.2-6.3-2.6-8.3-.7zM182.8 56c-1.3 2.3-1.3 2.3-1.3 3l-1 .2-.3 1.8c1.6 .6 2.7 .5 4.3 .5l4.8 .2c1.5 0 1.5 0 2.9-.7-.3-1.6-.9-1.9-2.2-2.7-1.4-1-2.8-2.1-4-3.4-1.3-1.6-2.4-.1-3.2 1.1zM131.2 59.5l-.2 2.5h-1.5l-1 .7-2.8-.2c-.9 0-.9 0-2.5 .7-.4 1.5-.4 1.5 0 2.3h1v1c1 .2 1 .2 2.3 0l.7 1c1.2 .2 2.1 .2 3.3 .2l1.3 .8 2.4 .2 .3 .3h2l.6-1.1c.7-1.2 .7-1.2 1.4-1.9v-.8l1.3-.5 .7-2c.1-1.3 .1-1.6-.7-2.7-1 .5-1 .5-1.8 2l-1.5 .7-.8 .7c-.7 .6-.7 .6-1.9 .6-1.2-1.6-1.1-2.9-1-4.8-1-.2-1-.2-1.6 .3zM149.5 61.7l-.3 1.8-1.4-.3-.6 .3c0 1.6 0 1.6 .6 2.5 0 1.6-.4 1.7-1.8 2.5l-2 .7c-.8 .8-.4 1.7-.2 2.8l-1.6 .5-1 1 2.7 2.8c2 2.1 2 2.1 3.6 1.9 1-.5 1.7-1.1 2.5-2l1-.5v-1.5c1.2-1 1.2-1 2.5-1.5 1-1.1 1.8-2.1 1.7-3.7-.7-1-.9-1.3-2.2-1.3l-.2-1.1c-.3-1.4-.3-1.4-1.3-2.6l-.5-2c-.9-.9-.9-.9-1.5-.3zM210 241.4l-.2 1-2.5 .3c-1.2 .3-1.3 1.1-1.8 2.2-1 1-1 1-2.5 1.5-.8 .8-.8 .8-.5 2 1.5 1.5 2.6 1.2 4.6 .7 1.1-.4 1.1-.4 2.1-1.4v-.5l1.3-.5c1.3-1.5 1.3-1.5 1-3l.7-1v-1.5c-1-.5-1-.5-2.2 .2zM181.5 255.7v1.2h-1.3l-.2 1c1.3 .3 2.5 .3 3.8 .3l1.4 .1c1-.1 1-.1 1.6-.9-.3-.5-.3-.5-1.6-1h-.7l.3-1.5c-1.4-.9-2.1-.2-3.3 .8zM241 255.7l1.2 1-.7 .2-.3 1.3h-1.4c-.6-1.7-.9-2.5 1.2-2.5zM229 256.9h4l.5 1c-.5 .5-1.7 .3-2.4 .3h-2.1z","role":"wash"},{"d":"M182.7 65.2h1.1c3.5 0 6.8 .5 10.2 1l1.8 .2c2.7 .3 5.2 1.1 7.7 1.8l1.2 .4c1.8 .5 3.4 1.1 5 2.1l8.1 4.3c1.9 1 3.6 2.1 5.2 3.5l1.5 .7 .8 .8c1 1 2 1.7 3.2 2.5 2.2 1.4 3.9 3.3 5.6 5.2l1.6 1.6c2.2 2.2 3.6 4.1 4.8 6.9l1 2.1c1.2 2.3 2.2 4.4 2.7 6.9 .5 1.4 .9 2.8 1.6 4.1 1.1 2.9 .5 5.4-.8 8.2l-.5 1.6c-.8 2.7-1.9 5-3.3 7.4h-1l.1-3.6c0-1.1-.2-1.9-.6-2.9l-.1-2.6c-.2-2.2-.6-4.2-1.4-6.2l-2.6-7.9c-.3-1.2-.8-2.2-1.4-3.3l-.2-.9c-.3-1-.7-1.8-1.3-2.6l-2.7-4.5-2.5-3.3c-1.2-1.6-2.5-2.7-4.2-3.8-1.4-.9-2.4-1.8-3.4-3s-2.4-1.8-3.6-2.6l-2.7-1.7c-1.2-.8-2-1-3.4-1.4l-2.1-1.3c-1.4-.7-2.9-1.2-4.4-1.7l-7-2.1c-2-.5-4-.4-6.1-.4h-3.5l-3.4-.2c-2.6-.2-4.7 .3-7.2 1l-4.4 1c-1.4 .5-2.6 1.2-3.9 2l-3.5 1.2c-2.2 .8-4.1 2.4-5.7 4.1-1.1 1-2.2 1.8-3.5 2.7-1.1 .8-1.8 1.6-2.5 2.7-1.8-.6-1.8-.6-2.5-2-.3-1.9 .3-2.5 1.7-3.7l2.2-2.1c2.5-2.7 5.6-4.7 8.8-6.4l1.3-1c1.2-.8 1.2-.8 2.4-1.1 1.4-.4 2.4-1.1 3.6-1.9 .9-.3 1.6-.3 2.5-.3 1.6-.1 2.8-.5 4.2-1.2 1.9-.3 3.7-.3 5.5-.3zM256.8 141.2v8.1c0 9.1 0 9.1-.8 11.7l-.6 6.6c-.2 .9-.2 .9-.9 2.1l-1.2 5.7-1.6 6-4.7 13.4c-.3 1-.7 1.6-1.2 2.4l-.8 1.8c-1 2.6-2.3 4.8-4.2 6.9l-.8 1.5c-.8 1.4-1.4 2.5-2.5 3.5l-.7 1.3c-1 1.4-2.1 2.5-3.3 3.7l-4.1 6.1c-.6 .9-.6 .9-1.6 1.7l-.3 .7-.7-.2c0-2.9 1.5-4.6 3.4-6.5l.6-1.8 1-1.2 .3-1.3c.6-2.2 1.7-4.1 3.1-6l4.4-8.4c1.5-2.6 2.7-5.1 3.4-8.1l.8-1.2 .2-1.2c.2-1.3 .6-1.8 1.2-2.8l3.1-11.2c.2-1.6 .2-1.6 .9-3.1l.3-1.2c.3-1.5 .3-1.5 .9-2.9 .8-1.9 .9-4 1.1-6.1 .2-2.3 .6-4.4 1.3-6.6 .5-2 .7-3.9 .7-5.9 .1-1.7 1-7.5 3.3-7.5zM216.2 217.9c1 .5 1 1.6 1.1 2.6 .1 2.7-.5 4.7-2 7-.5 .9-.5 .9-.9 2.1-.7 1.8-1.7 2.2-3.4 3.1-.5 .5-.5 .5-.7 1.6-.5 1.7-1.3 2.2-2.9 2.9-1.1 .5-1.6 .9-2.4 1.7s-1.1 1.2-2.1 1.6c-1.3 .6-2.1 1.6-3.1 2.7-1.6 .7-1.6 .7-2.5 .9-1.1 .3-1.1 .3-2 1.3-1.1 1.1-1.8 1.4-3.3 1.8-2.3 .8-2.3 .8-3.2 1.6s-1.9 1-3.1 1.4l-5.5 1.7-1.5 .2c-1.4 .3-2.6 .6-3.9 1.1-2 .6-4.5 .4-6.6 .4l-6.3 .1c-1.4 0-1.4 0-2.7-.8 .4-1.5 1.2-1.8 2.6-2.5 1.7-.2 3.3 0 5 .3l.7-.3 1.9-.1c1.9-.1 3.5-.4 5.3-.8l3.2-.8c3.2-.8 6.3-1.8 9.3-3l1.2-.5c3.5-1.4 7.4-3 9.8-6l1.3-.8 .9-.8c.8-.9 .8-.9 2.1-1.7l2.5-3 1.2-1 5.6-6.5z","role":"wash"},{"d":"M224.4 85.2c4.8 .6 4.8 .6 6 2.4l3 .6v3.6l5.4 6v1.2l1.8 .6c4.5 10.4 6.6 16.9 .6 26.9h-.6l-.6-5.2c-1.9-13.6-6.3-25.4-15.6-35.5z","role":"wash2"},{"d":"M135.6 38.6l3.2 1.1c5.9 2.4 7.3 6.1 8.3 12.2l1.7 .6 1.2 9.2c-.6 1.7-.6 1.7-2.3 2.3l.6 2.9-4.1 2.9-1.1 2.9c-3.5 0-3.5 0-6.4-1.8 .6-2.3 .6-2.3 2.3-4.8 1.8-2.7 1.8-2.7 2.3-5.5l1.2-2.9c-1-5-2.1-10.3-7.5-12.1-8 0-8 0-10.4 4 0 4.6 0 4.6 2.3 7.5l2.9 1.2c.6 1.7 .6 1.7-1.7 4-5.2-.6-5.2-.6-6.9-2.3v-2.3l-1.8-1.8c-.6-6.7 1.5-10.3 5.8-15.5l5.2-1.2c2.3-1.7 2.3-1.7 5.2-.6zM229 230.7c2.9 .6 2.9 .6 3.5 1.2 0 4.3-2.3 4.7-5.8 6.9l-5.2 3.5-.5-1.8c-4.1 2.1-4.7 3.7-7 7.5l-4 4.6-1.2 2.9-1.7 .6-1.1-1.7-1.2 1.1h-2.9l1.2 4.6c-2.3 2.3-10.8 .6-13.9 0 0-3.4 0-3.4 2.3-4.6l.6-1.7h-1.1c0-1.7 0-1.7 2.3-2.9h4.6c-2.3 1.7-2.3 1.7-4.6 1.7v1.2c5.2 0 5.2 0 7.5-1.2v-2.3h5.2c4.3-2.6 5-3.5 6.3-8.6l3.5-2.3 1.4-3.6c1.5-3.4 1.5-3.4 4.9-3.9 2.9 2.9 2.9 2.9 2.9 4 2.3-1.1 2.3-1.1 4-5.2z","role":"wash2"},{"d":"M232.5 148.2c-.6 4.1-.6 4.1-1.2 4.7 0 3.4 0 3.4 .6 4.6l-2.3-.6c0 4 0 4 .6 8.1l-1.7 2.8-2.9-1.1c0 2.3 0 2.3 1.7 3.5l.6 2.8-1.7 .6v4.6l-2.9 .6 1.1 3.5-1.7 .5-.6 1.2-1.7-.6c-.2-8.8-.2-8.8 1.1-12.7l1.8-.5-.6-2.9c2.9-11.3 2.9-11.3 6.3-15.6l1.8-2.9c.5-.6 .5-.6 1.7-.6zM226.2 162.1v1.7l1.1-1.1z","role":"wash2"},{"d":"M172.2 37.2c5.2 .9 8.2 3.1 10.6 7.8-2.2-1.1-2.2-1.1-5-3.9-2.8-2.2-2.8-2.2-5.6-3.3zM147.2 52.2l1.7 .6v2.8h1.7l.5 5.5-1.7-1.1c-1.6 3.9-1.6 3.9-2.7 7.8l-2.3 1.1c-.5-2.2-.5-2.2 1.7-4.5 .9-3.2 1.3-5 1.2-8.3zM119.4 52.8c1.2 1.1 1.2 1.1 2.3 5 2.4 2.4 4 2.9 7.2 3.9-2.2 1.1-2.2 1.1-6.1-.6-2-2.5-3-4.2-3.9-7.2zM187.2 55c1.1 0 1.1 0 3.4 2.2l7.2 3.4c-2.2 1.1-4.5-.4-6.7-1.2-3.9-2.8-3.9-2.8-3.9-4.4zM181.7 56.7l.5 1.6c-3.6 3.6-5.6 3.4-10.5 3.4l-1.7-1.7 2.8 .1c3.9-.1 5.8-1.2 8.9-3.4zM206.1 64.4v1.2c-3.1-.6-5.9-1.3-8.9-2.3l-3.9-.2c-3.9-.3-3.9-.3-6.1-1.4 7-1 12.4 .4 18.9 2.7zM182.2 61.7c-1.1 2.2-5 1.9-7.2 2.2-3.6 .7-6.7 1.8-10 3.3l-2.8 1.1c-4.2 1.8-7.3 4-10.5 7.3h-1.1c0-3.4 6.8-6.8 9.4-8.4l8.5-3.4c4.6-1.5 8.9-2.1 13.7-2.1zM127.8 67.2c10.7 1.6 17.2 9 22.8 17.8l1.5 2.1c3.1 4.6 3.4 9.8 4 15.1h-1.1l-.7-3.5c-3-13.8-11.8-23.9-24.3-29.8l-2.2-1.1zM253.3 79.4c2.5 3.3 1.6 6.1 1.1 10h1.2c-.6 2.8-.6 2.8-1.7 3.4l-.9-8.5c-.2-2.6-.2-2.6 .3-4.9zM256.1 96.1h.6c0 8 0 8-.6 11.1h-1.7l-.5 3.9-1.1 1.1zM256.1 114.4h.6c0 3.9 0 3.9-1.1 7.8l-1.2-.5-1.1 1.1c.6-3.2 1.5-5.5 2.8-8.4zM256.7 127.2c-.5 3.5-1 5-2.6 7.9l-2.4 4.3c-2 3.5-3.8 6.8-5 10.6l-1.1-.6c7.6-22.2 7.6-22.2 11.1-22.2zM255 136.1l1.1 .6c-2.2 5.8-4.4 11.2-7.7 16.4-4.2 6.8-4.2 6.8-5.1 10.2h-1.1v1.1h-1.1c1.1-3.3 2.5-6.1 4.2-9.2l8.1-16.4c1-2.1 1-2.1 1.6-2.7zM250.6 152.2c1.3 5.3-3.6 11.6-6.2 16.1h-1.1l-.5 2.3-2.2 4.4h-1.2l-1.1 3.3-1.6 1.1c1.1-3.2 2.5-5.8 4.2-8.8 2.9-4.9 5.5-9.9 8-15zM246.1 168.3c1.7 5.1-5.8 14.5-8.3 18.9l-1.1 2.2-2.3 .6c1.3-3.1 2.9-5.9 4.5-8.9l1.1-2.2h1.1l.6-2.2c1.2-3 2.7-5.6 4.4-8.4zM240 185.6h1.1c-.7 3.6-2.4 6.1-4.2 9.3l-3.6 6.2c-.9-2.8 1-4.7 2.4-7.1 1.6-2.8 3.1-5.5 4.3-8.4zM181.1 254.4c-4.8 2.1-8.7 2.4-13.9 2.7l-4 .1c-3.4 0-6.5-.3-9.9-1.1 4.2-2.1 9.5-1.1 14.1-1.1 3.6 .1 10.9-2 13.7-.6z","role":"hi"},{"d":"M137.6 46.5l.8 1.5 1.5 .8-.4 .7c-1.9-1.1-1.9-1.1-3-2.6-2.4-1.2-4.2-1.3-6.7-.7l-3.4 2.6h-.8c-.9 2.3-.7 4.3-.4 6.7-1.3-2-1.2-4-.7-6.3 3.4-4.6 8-5.6 13.1-2.7zM185.2 45c1.9 0 2.4 1.8 3.3 3.3 1.7 2.4 3.8 4.3 6.5 5.4l1.1 .7c2.9 .2 2.9 .2 3.8-1.1h.7l-.2 1.2c-.2 1.4-.2 1.4 .2 2.5-1.5-.3-1.5-.3-2.6-1.1l-1.4-.3c-4.8-1.3-8.7-4.8-10.6-9.4zM174.8 45.4c3.1 2.2 4.1 4.8 4.1 8.6l-.8 1.5-1.1-.3 .8-1.5c0-1.9 0-1.9-1.2-3l-1.1-.4-.4 .7-.7-1.1 1.5-1.1zM141 51.8c1.8 1.8 1.7 3.5 1.9 6l-1.5 3.4c-1.2-1.2-1.2-1.2-.4-3.8l-1.5 .4v-.8l1.1-.3zM111 53.7l-.9 4.2c-.8 1.8-1.5 2-3.2 2.5v-.7h.7l.4-5.7c.4-.3 .4-.3 3-.3zM172.1 54.4c1.1 .4 1.1 .4 1.9 1.5l2.2-1.5c0 2-.6 2.2-2.2 3.4-1.9 .7-1.9 .7-3.4 0 .4-1.1 .4-1.1 1.2-1.5zM188.7 70.6h2.2c5.3 .2 5.3 .2 7.5 1.1l1.7 .3c1.6 .4 1.6 .4 2.8 1.5h-.8v.8c-2.6 0-2.6 0-4.5-.8-2.5-.4-4.9-.7-7.5-.7-1.9 0-1.9 0-3.7-.8-4.1-.6-7.3-.3-11.2 .9-1.2 .3-1.2 .3-2.3-.1 5-2.5 10.3-2.4 15.8-2.2zM253.9 78c1.1 1.5 1.1 1.5 1.1 2.7h.8l.1 1.8c.2 1.9 .2 1.9 .6 2.3l.3 6c.1 2-.2 3.9-.7 5.9l-1.1 6.5h-.4c-.1-2.3 .3-4.5 .6-6.8s.2-4.7 .2-7.1l-.8-.4 .4-5.3c0-1.8-.3-2.8-1.1-4.4zM249.4 79.5c1.2 1.3 1.5 2 2.1 3.6l.5 1.4c2.1 7.7 1 16.1-1.5 23.6l-1.9 5.8-1.2 3.1c-1.1 2.1-1.8 4.2-2.5 6.5l1.9 1.8-1.2 2.7-.7-1.5c-.8-1.2-.8-1.2-1.5-1.2l-.8 2.3-3.5 6.5c-4.9 8.7-4.9 8.7-7.3 11.1 .3-2.3 .7-2.8 2.2-4.5l1.1-1.9h.8l-.4-1.5 .9-1.4c1.1-1.7 1.1-2.6 1-4.6l.4-1.1c.9-11.3 .9-11.3-.8-16.5v-1.9l-.8-5.2c3.3 1.6 4.1 13.2 4.2 16.9-.1 2.2-.6 4.2-1.2 6.3l.8 .4 .1-1.1c.3-1.9 1.1-2.9 2.1-4.5 1.6-3.1 2.7-6.1 3.8-9.4l.8-1.8c.8-2.2 1.3-4.3 1.9-6.6l1.1-3.6c1-4.2 1.2-8.4 1.2-12.7v-1.4c0-3.1-.3-5.3-1.6-8zM220.1 83.3c2.2 .9 5.2 2.1 6 4.5l1.5 .8 5.5 8.2c1.7 2.9 2.4 6.1 3.1 9.4-1.6-.6-1.6-1.6-2.2-3.2l-1.1-3.2c-2.2-5.3-5.7-10.6-10.5-13.9-1.8-1.6-1.8-1.6-2.3-2.6zM256.5 106.6h.4c.2 5.4-1.6 9.7-3.8 14.6-.5-2 .1-3.2 .7-5.2zM256.5 120.1h.4c.3 3.8-.1 5.9-2.2 9.2l-6.5 12.5c-.8-2.6 1.8-6.6 3-9 2-4.2 3.7-8.4 5.3-12.7zM256.9 131l-.2 1.5c-.2 1.8-.2 1.8 .2 4.1l-1.9-1.5-2.6 6.3-9.4 18.5c-1.1 2.2-1.1 2.2-2.6 3.7 .6-2 1.4-3.6 2.4-5.4 2.3-4.2 4.2-8.6 6.2-13 6.5-14.2 6.5-14.2 7.9-14.2zM253.5 144.8c.4 1.2 .4 1.2 0 2.3l.1 1.5c-.2 5.2-2.3 9.6-4.7 14-.8 2.1-.5 3.4-.3 5.5l-2.2 5.6c-.8 1.5-.8 1.5-1.9 2.3l2.6-8.6c-1.5 1.5-2.4 3.1-3.4 5.1l-12.9 23.9c-.9 1.7-.9 1.7-1.7 2.1 1.2-3.5 2.9-6.8 4.5-10.1l.8-1.7c1.5-3 3.1-5.7 4.8-8.5l4.6-8.4c3.7-6 6.4-12.6 8.6-19.3-1.2 .3-1.2 .3-2.3 2.2-.6-1.2 .3-2.1 .9-3.3zM229.5 148.2c-.6 1.9-1.2 2.9-2.3 4.5-2.2 4-3.6 7.9-4.7 12.4-.5 1.8-1.2 3.5-2 5.3l-.9 10.2-.4 38.4c-.1 1.7-.2 3.3-.6 5l-.4 9c-.9 3.1-1.9 6.1-4.1 8.7l-1.5-.8h-.7c.7-1.5 .7-1.5 2-2.6 3.8-3.3 3.3-9.2 3.3-13.9v-1.4c0-1.7 0-3.3-.4-5-.8 0-.8 0-1.6 1.5l-2 3.7c-1 1.6-1 1.6-2.1 2.3 3.3-8.1 3.3-8.1 6-10.9v-1.1l-.7-.4-.8 1.5-1.1-.3 2.6-2.7c.5-2.2 .4-3-.3-5.2-.5-1.3-.4-2.4-.4-3.8 0-2.6 0-2.6-.4-3.4-1-7.4-1-7.4 .8-10.8 .5-3.1 .7-6.2 .9-9.4 .4-5.6 1.6-10.5 3.5-15.8l2.3-6.4 1.2-1.5c1.1-1.5 1.1-1.5 1.1-3.7l-.4-.8 1.5-.3 .7-1.5c1.2-.8 1.2-.8 1.9-.8zM249.4 152.7c1.4 2.9-4.5 10.9-6 13.5l-3.6 6.5c-4.5 8.1-4.5 8.1-5.4 8.5 .5-2.4 1.7-4.2 3-6.3l10.3-19.4zM256.5 158h.4v9.4l-.8 .3 .1 1.7c-.3 8.8-3.6 17.5-7.5 25.2l-3.4 7.1c-2.8 6-2.8 6-5.3 8.4l-3 4.5-1.1 .8-.9 2.1c-1 2.4-1 2.4-2 3.6-1.2 1.4-1.2 2.6-1.2 4.4l-1.2-1.5-1.1 1.9-1.1 .4-1.5 2.2c-1.6-1.1-1.4-1.8-1.5-3.7l-.2-2c.6-9.6 5.8-18.1 10.7-26.2l1.9-3.4c2.9-5.2 2.9-5.2 4.1-5.2l-10.7 19.4c-1 2.1-1.8 4.3-2.5 6.6l-.8 2.3c-.9 3-1.5 5.3-1.4 8.5h1.1l.8-1.5 4.7-6.5 3.9-4.8c1.8-2.2 3.1-4.6 4.5-7.1l1.5-1.9 3.3-6.7c1.3-2.8 2.3-5.7 3.3-8.7l.5-1.6c3.2-9.9 3.2-9.9 4.2-14.7 .3-1.7 .3-1.7 .7-2.1zM244.1 176l.8 .7-.7 1.1c-2.8 4.9-2.8 4.9-2 8.3l-.7 .4-.4-1.1c-1.7 .6-1.8 1.9-2.4 3.5-1 2.2-2.3 4.2-3.6 6.2l-1.1 1.9c-.8-1.5 .8-3 1.6-4.3l2.2-5.1c1.8-4 4-7.8 6.3-11.6zM232.9 231.9l.7 .4c0 3.4-.5 4.2-3.4 6l-1.2 .9c-1.4 1-1.4 1-3.6 1.7l-1.9 1.9c-1.1 .7-1.1 .7-2.3 .7 .8-3.1 5-4.8 7.8-6.1s3.7-2.3 3.9-5.5zM225 234.1c1.5 1.2 1.5 1.2 1.5 1.9h-.7l-.8 1.5-6 6 1.5 .8c-.5 1.5-1.3 1.8-2.6 2.6-5.3 4.2-5.3 4.2-5.3 6.8l-1.5 .7-.3 .8-1.2-.4c1.1-3 3.5-7.6 6.8-8.6l.4-1.2c1-2.1 2.7-3.3 4.4-4.8l3.4-3.8zM189.8 243.5l.7 .8c-4.8 2-9.5 3.7-14.6 4.9l-1.5 .3c-2.1 .5-3.8 .8-6 .8l-1.4 .2c-1.2 .2-1.2 .2-2.4-.6 1.8-1.7 2.5-1.7 4.8-1.8l8-.9c1.8-.3 1.8-.3 3.7-1l1.5-.2c1.6-.2 3-.7 4.5-1.4zM202.5 246.9l.4 1.9c1.8 .3 2.9 .3 4.9 0-1.6 1.5-1.6 1.5-3.4 2.6l-.8-.4v-.3c-2.6-.5-2.6-.5-3 0l-11.2 1.1c2-2.1 5-2.2 7.8-2.4 2.2-.3 3.5-2.5 5.3-2.5zM201.4 252.2v.7c-2.8 1.2-5.4 1.4-8.4 1.6-2.8 .5-4.1 1.4-5.1 4v1.2l3 .7 .3-1.2c.4-1.4 .4-1.4 1.6-2.5l.7 .3-.7 1.1c-.8 1.2-.8 1.2-1.2 2.7-4 .1-4 .1-4.8-1.1l.7-3.8-1.9-.7v-1.2l3-1.5v.8l.8 .4-.4 .7 7.8-1.5c3.4-.7 3.4-.7 4.6-.7zM206.2 255.2l1.6 1.5 2.2 .3c-.4 1.2-.4 1.2-1.5 1.4-1.5 .1-1.5 .1-3.7-.6v2.6l-1.6 .4-.3-1.5-.8-.4c.4-1.5 .4-1.5 2.3-2.2zM180.8 255.5c-.4 1.5-.4 1.5-1.2 2.3-6 1.4-24.4 4.1-29.2-.8l3 .4v-.7c2.6 0 2.6 0 4.1 .3l8.6 .1c4.9 0 9.9-1.6 14.7-1.6z","role":"ink"}],"lowerHead":[{"d":"M154.2 190.4l3.9 3c.9 .8 .9 .8 2.1 1.3l.3 1 1 .5c1 .5 1.5 .9 2.3 1.7l1.3 1c1.4 1.3 1.3 2.4 1.2 4.2-.1 .8-.1 .8 .7 1.8l.5 1.3 1.5 .7c1.4 1.4 1.9 3.1 2.2 5l-.9 2.6-.8 1.2-.2 .9c-.3 .8-.3 .8-1.1 1.3-.9 1.8-1.3 3.6-1.7 5.5l-.2 .9c-.3 1.9-.2 2.9 .5 4.6l.5 3.3c.2 1 .2 1 .9 2l.8 2.2 .8 1v2.3c-1.3 0-1.3 0-2-.8v-3.2l-5.2-1.9c-2.4-.8-4.5-1.5-7-1.7l-2.1-.7-5-.7c-1.8 .9-2.6 1.5-3 3.5-.1 2.6-.1 2.6 .7 3.5l-.4 1c-.3 1.8 .5 3.3 1.9 4.6 .8 .9 .9 1.9 1.1 3.1-2.2 1.9-4.3 2.2-7 2-2.8-1.4-4.2-4.5-4.8-7.5l-.3-.8c-.2-1.1-.2-2.1-.2-3.2l-.3-.2c-.5-4.4 .7-7.7 3.3-11.3l1.5-2.7c.5-.8 .5-.8 1.8-1.8 .1-2 .1-2-.3-2.5-.1-1.6-.1-1.6 .3-2.2h.4l.3 2.2h1c0-2.9 0-2.9-.3-3.7h1.3v-1.5l-1.3-.5c-.2-.5-.2-.5 0-1.3h1.3c.1-2.1 .1-2.1-.5-3.2 1.5-.5 1.5-.5 2.2 0 2.1-1.3 2.4-3 2.8-5.3l.2-1.2-.4-1.3 .4-2 .2-2.1c.1-2.2 .2-3.9 1.8-5.6 .6-.3 .6-.3 2-.3z","role":"wash"},{"d":"M153.8 190.4c2.8 1.2 2.8 1.2 3.2 2.8l.2 4.4c.1 1-.2 1.6-.4 2.6l-.3 2.5c-.3 3.6-.8 7.1-3.5 9.7h-.8c-.3-1.6-.3-1.6 .3-2.2 .9-2.5 .8-5.1 .7-7.8 0-1 0-1 .8-2l.1-2.7c-.1-.8-.1-.8-.6-1.5-.2-1.3-.2-2.3-.2-3.6l.1-1.1c.1-.8 .1-.8 .4-1.1zM146.2 226.9c.8 .8 .8 .8 .6 2l-1.8 5.3-.4 1.5c-.7 3.1 .8 6.5 2.9 8.8 .5 .7 .5 .7 .3 2.2l-1.6 1c-2.5-1.3-3.6-2.4-3.8-5.3-.2-1.5-.4-1.9-1.4-3l.2-1c.1-1.2 0-1.8-.4-3-.2-1.4 .1-2.4 .6-3.7l.4-1c1.8-4.1 1.8-4.1 4.4-3.8zM157 229.4l.2 1-1-.2-2.8 .1c-.9 .1-.9 .1-1.9 .6-1.5 0-1.5 0-2-.2 .3-1.4 1-1.6 2.3-2.1 2-.4 3.5-.3 5.2 .8z","role":"wash2"},{"d":"M161.1 222.2c2.8 .6 2.8 .6 3.3 1.1l.2 2.9c.4 3.1 1 4.5 2.6 7.1l-.5 1.7c-2.8-3.2-3.2-5.8-3.4-10-.5-1.1-.5-1.1-2.2-1.7zM138.9 228.9h.5l-.3 3.5c-.3 5.4 .2 9.7 3.1 14.3l2.8 .5v1.1c-3.3 0-3.3 0-5.1-2-2.8-5.3-3.8-11.8-1-17.4z","role":"hi"},{"d":"M160.9 195.5c.2 7.7-3.3 14.2-8.7 19.5l-1.4-.4c.3-2.2 .3-2.2 1.1-2.6v1.1h1.1l.8-1.5 2.5-3.5c1.6-2.7 2.3-5.5 2.9-8.5 1-4.1 1-4.1 1.7-4.1zM165 208.3c1.6 1.6 1.7 3.8 1.1 6-1.3 1.9-3 3.1-4.9 4.5l-.7-.4 1.5-1.2c1.5-1.4 1.5-1.4 1.5-2.9l-.7-.4v-1.5h2.6l-1.2-1.5zM160.9 223.6c1.5 .4 1.5 .4 2.1 2.1l1.6 8.1-1.5 .3-.7-1.1 1.1-.7c-1.7-4.3-1.7-4.3-2.8-5.8-.6-1-.6-1 .2-2.9zM140.2 224c.6 1.8-.4 2.9-1.2 4.4-1.4 2.8-1.5 5.3-1.6 8.4l-.2 4.1c-1.1-1.2-.9-2.5-1.1-4.1l-.3-1.6c-.1-4.3 1.9-7.9 4.4-11.2zM160 226.8c1.1 1.7 1.5 3.4 2 5.5-1.5-.8-1.5-.8-3.4-2.3-3.3-1.7-5.1-2.2-8.6-.7l-.7 1.1-2.3 1.1-1.5 1.5 2.3-7.1 .3 1.5 2.3-.4 1.1 .4v-.8l1.5-.1c5.9-.7 5.9-.7 7 .3zM156 226.3v1.1c2.5 1.1 2.5 1.1 3.8 1.1l-1.2-2.2z","role":"ink"}]};

  // Centerlines in source 300x300 coordinates (head end -> tail end)
  const LEFT_SPINE = [
    [212, 48], [174, 30], [134, 28], [94, 42], [68, 66],
    [76, 98], [76, 124], [62, 154], [56, 192], [72, 232], [112, 252], [148, 248]
  ];
  const RIGHT_SPINE = [
    [198, 254], [218, 240], [224, 214], [228, 184], [236, 150], [246, 116], [252, 82]
  ];

  const KNOTTED_PRESETS = [
    { wash: '#1b5cff', wash2: '#00d97e', dur: 13.0, swirl: true },
    { wash: '#ff5a1f', wash2: '#5b2bff', dur: 13.8 },
    { wash: '#00c8d7', wash2: '#ffb800', dur: 12.6 }
  ];

  function mountKnottedPair(bubbleEl, presetIndex, overrides) {
    const Rig = root.BestiaryTracedRig;
    if (!bubbleEl || !Rig) return null;
    if (typeof presetIndex === 'object' && presetIndex !== null) {
      overrides = presetIndex;
      presetIndex = 0;
    }
    const preset = KNOTTED_PRESETS[(presetIndex || 0) % KNOTTED_PRESETS.length];
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      wash: preset.wash,
      wash2: preset.wash2,
      scale: narrow ? 0.78 : 0.94,
      dur: preset.dur,
      swirl: !!preset.swirl,
      autoPause: true
    }, overrides || {});

    const rect = bubbleEl.getBoundingClientRect ? bubbleEl.getBoundingClientRect() : { width: 280, height: 74 };
    const w = Math.max(120, Math.round(rect.width || 280));
    const h = Math.max(44, Math.round(rect.height || 74));
    const sc = opts.scale;
    const padX = 24;
    const padTop = 10;
    const knotH = Math.round(86 * sc);
    const svgW = Math.ceil(w + padX * 2);
    const svgH = Math.ceil(h + padTop + knotH);

    const svg = Rig.el('svg', {
      xmlns: Rig.SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-knotted bestiary-rig bestiary-traced-rig',
      'data-rig': 'knotted-pair-traced',
      'data-dur': Rig.f1(opts.dur),
      style: `position:absolute;left:${-padX}px;top:${-padTop}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    svg.style.setProperty('--wash', opts.wash);
    svg.style.setProperty('--wash2', opts.wash2);

    const defs = Rig.el('defs', {}, svg);
    const edgeY = padTop + h;
    const leftStopX = padX + Math.min(w * 0.34, 88);
    const rightStopX = padX + Math.max(w * 0.66, w - 88);
    const midX = padX + w * 0.5;
    const durS = `${Rig.f1(opts.dur)}s`;

    // Two interlocking over-under coiling loops (local coordinates centered at (0, 0) on bubble bottom edge)
    const loopAId = Rig.uid('knot-loop-a');
    const loopBId = Rig.uid('knot-loop-b');
    const pathA = 'M-56,8 C-54,36 -26,48 0,24 C26,0 56,8 54,34 C52,52 18,52 -4,26 C-26,0 -58,0 -56,8';
    const pathB = 'M56,8 C54,36 26,48 0,24 C-26,0 -56,8 -54,34 C-52,52 -18,52 4,26 C26,0 58,0 56,8';
    Rig.el('path', { id: loopAId, d: pathA, fill: 'none' }, defs);
    Rig.el('path', { id: loopBId, d: pathB, fill: 'none' }, defs);

    // Whole knotted pair glides along the bubble's lower edge & cinches tight/loose
    const slideG = Rig.el('g', { class: 'bestiary-knot-slider' }, svg);
    Rig.el('animateTransform', {
      attributeName: 'transform', type: 'translate',
      values: `${Rig.f1(leftStopX)} ${Rig.f1(edgeY)};${Rig.f1(leftStopX)} ${Rig.f1(edgeY)};${Rig.f1(midX)} ${Rig.f1(edgeY + 2)};${Rig.f1(rightStopX)} ${Rig.f1(edgeY)};${Rig.f1(rightStopX)} ${Rig.f1(edgeY)};${Rig.f1(midX)} ${Rig.f1(edgeY - 1)};${Rig.f1(leftStopX)} ${Rig.f1(edgeY)};${Rig.f1(leftStopX)} ${Rig.f1(edgeY)}`,
      keyTimes: '0;.42;.55;.68;.76;.86;.95;1',
      calcMode: 'spline',
      keySplines: '0 0 1 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1;.4 0 .6 1;.4 0 .6 1;0 0 1 1',
      dur: durS, repeatCount: 'indefinite'
    }, slideG);

    if (opts.swirl) {
      slideG.replaceChildren();
      const cx = midX, cy = edgeY + 22 * sc, dx = Math.min(w * 0.18, 60);
      Rig.el('animateTransform', {
        attributeName: 'transform', type: 'translate',
        values: `${Rig.f1(cx)} ${Rig.f1(cy)};${Rig.f1(cx - dx)} ${Rig.f1(cy + 4)};${Rig.f1(cx + dx * .4)} ${Rig.f1(cy - 2)};${Rig.f1(cx + dx)} ${Rig.f1(cy + 5)};${Rig.f1(cx)} ${Rig.f1(cy)}`,
        keyTimes: '0;.27;.5;.76;1', calcMode: 'spline', keySplines: Rig.buildKeySplines(5),
        dur: `${Rig.f1(opts.dur * 1.9)}s`, repeatCount: 'indefinite'
      }, slideG);
    }
    const cinchG = Rig.el('g', {}, slideG);
    if (opts.swirl) {
      const wheel = Rig.el('g', {}, slideG);
      Rig.el('animateTransform', {
        attributeName: 'transform', type: 'rotate',
        values: '0 0 24;70 0 24;105 0 24;250 0 24;290 0 24;360 0 24',
        keyTimes: '0;.22;.38;.62;.78;1', calcMode: 'spline',
        keySplines: '.5 0 .3 1;.6 0 .6 1;.5 0 .3 1;.6 0 .6 1;.5 0 .4 1',
        dur: `${Rig.f1(opts.dur * 1.3)}s`, repeatCount: 'indefinite'
      }, wheel);
      Rig.el('animateTransform', {
        attributeName: 'transform', type: 'translate', additive: 'sum',
        values: '0 -24;0 -24', dur: '1s', repeatCount: 'indefinite'
      }, wheel);
      wheel.appendChild(cinchG);
    }
    Rig.el('animateTransform', {
      attributeName: 'transform', type: 'scale',
      values: `${Rig.f2(sc)} ${Rig.f2(sc)};${Rig.f2(sc * 0.88)} ${Rig.f2(sc * 0.90)};${Rig.f2(sc * 1.08)} ${Rig.f2(sc * 1.04)};${Rig.f2(sc * 0.90)} ${Rig.f2(sc * 0.92)};${Rig.f2(sc * 1.12)} ${Rig.f2(sc * 0.94)};${Rig.f2(sc)} ${Rig.f2(sc)}`,
      keyTimes: '0;.18;.36;.56;.78;1',
      calcMode: 'spline', keySplines: Rig.buildKeySplines(6),
      dur: durS, repeatCount: 'indefinite'
    }, cinchG);

    const kt = '0;.18;.38;.56;.76;.92;1';
    const ks = Rig.buildKeySplines(7);
    const uValsA = [0.48, 0.64, 0.78, 0.92, 0.72, 0.56, 0.48];
    const uValsB = [0.52, 0.68, 0.82, 0.96, 0.76, 0.60, 0.52];

    const colA = `var(--wash, ${opts.wash})`;
    const colB = `var(--wash2, ${opts.wash2})`;

    // Dragon B (back/under coil): 16 overlapping traced segments from right-dragon.svg
    const rigB = Rig.mountSegmentedBody({
      defs, parent: cinchG,
      bodyLayers: GOTHIC_ART.rightBody,
      srcSpine: RIGHT_SPINE,
      srcWidth: 38,
      numSegments: 16,
      overlap: 0.60,
      targetPathId: loopBId,
      targetPathLen: 260,
      bodyScale: 0.34,
      bodyLenPx: 92,
      keyTimes: kt, keySplines: ks, headUValues: uValsB,
      dur: opts.dur,
      wash: colB, wash2: colA,
      className: 'gothic-dragon-b-body'
    });

    // Dragon A (front/over coil): 18 overlapping traced segments from left-dragon-body.svg
    const rigA = Rig.mountSegmentedBody({
      defs, parent: cinchG,
      bodyLayers: GOTHIC_ART.leftBody,
      srcSpine: LEFT_SPINE,
      srcWidth: 36,
      numSegments: 18,
      overlap: 0.60,
      targetPathId: loopAId,
      targetPathLen: 260,
      bodyScale: 0.32,
      bodyLenPx: 98,
      keyTimes: kt, keySplines: ks, headUValues: uValsA,
      dur: opts.dur,
      wash: colA, wash2: colB,
      className: 'gothic-dragon-a-body'
    });

    // Traced wing on Dragon A shoulder joint (left-dragon-wing.svg)
    const wingNode = Rig.el('g', { transform: 'scale(0.24) rotate(-68) translate(-58,-80)' });
    Rig.buildTracedGroup(GOTHIC_ART.leftWing, { wash: colA, wash2: colB }, wingNode);
    Rig.mountJointPart({
      parent: cinchG, targetPathId: loopAId,
      spanFrac: rigA.spanFrac, jointFrac: 0.24,
      keyTimes: kt, keySplines: ks, headUValues: uValsA,
      dur: opts.dur, node: wingNode,
      localRotate: { values: '-8;14;-12;16;-8', keyTimes: '0;.25;.50;.75;1', dur: opts.dur * 0.42 }
    });

    // Traced roaring head on Dragon A (green-dragon-head.svg) with biting snap + blink
    const headANode = Rig.el('g', {});
    headANode.appendChild(root.BestiarySlots.buildTracedRoundelHead({ wash: colA, wash2: colB, scale: 0.3, dur: opts.dur }));
        Rig.mountJointPart({
      parent: cinchG, targetPathId: loopAId,
      spanFrac: rigA.spanFrac, jointFrac: 0.0,
      keyTimes: kt, keySplines: ks, headUValues: uValsA,
      dur: opts.dur, node: headANode,
      localRotate: { values: '-8;12;-14;10;-8', keyTimes: '0;.22;.48;.74;1', dur: opts.dur }
    });

    // Traced biting head on Dragon B (lower-dragon-head.svg) with snapping motion + blink
    const headBNode = Rig.el('g', {});
    headBNode.appendChild(root.BestiarySlots.buildTracedRoundelHead({ wash: colB, wash2: colA, scale: 0.3, dur: opts.dur * 1.1 }));
        Rig.mountJointPart({
      parent: cinchG, targetPathId: loopBId,
      spanFrac: rigB.spanFrac, jointFrac: 0.0,
      keyTimes: kt, keySplines: ks, headUValues: uValsB,
      dur: opts.dur, node: headBNode,
      localRotate: { values: '8;-12;14;-10;8', keyTimes: '0;.26;.54;.80;1', dur: opts.dur }
    });

    if (typeof getComputedStyle === 'function' && getComputedStyle(bubbleEl).position === 'static') {
      bubbleEl.style.position = 'relative';
    }
    bubbleEl.appendChild(svg);
    Rig.observeVisibility(svg, opts.autoPause);
    return svg;
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.KNOTTED_PRESETS = KNOTTED_PRESETS;
  root.BestiarySlots.mountKnottedPair = mountKnottedPair;
}(typeof globalThis !== 'undefined' ? globalThis : this));

/**
 * Brief 6 — Slot: Gold Swarm (by "agentic development")
 * Three small potrace-traced bestiary vipers (traced/components/bestiary-vipers/),
 * each rigged as 14 overlapping traced body segments + traced head with separate
 * animated jaw & blink, swooping and darting round the bubble in neon gold (--wash / --wash2).
 */
(function (root) {
  'use strict';

  const VIPER_ART = {"headSkull":[{"d":"M112.0 174.2 l0.0 14.22.5 -0.0 c2.2 -0.0 2.5 -0.1 2.1 -0.5-0.5 -0.6 -0.2 -3.7 0.3 -4.30.6 -0.6 3.5 -0.7 4.1 -0.10.3 0.3 0.7 0.5 0.9 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.20.3 -0.0 0.8 0.4 1.2 1.0 l0.6 1.013.4 -0.0 c10.7 -0.0 13.1 -0.1 12.4 -0.4-1.2 -0.5 -2.2 -3.0 -2.6 -6.2-0.1 -1.2 -0.4 -2.5 -0.6 -2.9-0.4 -0.8 -0.5 -2.2 -0.2 -2.60.1 -0.1 0.2 -0.8 0.1 -1.4-0.1 -0.8 0.1 -1.1 0.4 -1.10.3 -0.0 0.7 -0.5 0.9 -1.10.3 -0.9 0.6 -1.2 1.3 -1.31.1 -0.1 1.3 -1.0 0.5 -1.8-1.8 -1.8 -2.4 -4.4 -1.3 -6.00.3 -0.4 0.6 -1.3 0.7 -2.1 l0.2 -1.4-1.2 -0.0 c-1.9 -0.0 -2.2 0.5 -2.6 3.8-0.3 3.1 -0.8 4.8 -1.4 4.8-0.2 -0.0 -0.3 0.3 -0.3 0.60.0 1.4 -1.6 1.8 -3.9 0.9-1.0 -0.4 -3.7 -0.6 -6.1 -0.5-1.0 0.1 -0.2 1.5 0.9 1.72.1 0.3 3.3 1.0 3.3 1.90.0 1.5 -10.9 0.5 -12.2 -1.1-0.8 -1.0 -1.7 -1.1 -2.0 -0.2-0.8 2.5 -7.8 0.9 -7.8 -1.80.0 -1.3 1.5 -3.8 2.3 -3.80.8 -0.0 1.7 -1.1 1.7 -2.10.0 -0.7 0.2 -0.9 0.8 -0.90.9 -0.0 1.5 -0.4 2.7 -2.1 l0.9 -1.2-8.2 -0.0-8.2 -0.00.0 14.2zM157.0 160.5 c0.0 0.6 0.9 1.5 1.9 1.91.3 0.5 1.4 0.4 1.4 -1.0 l0.0 -1.3-1.7 -0.0 c-1.2 -0.0 -1.7 0.1 -1.7 0.5z","role":"wash"},{"d":"M112.1 160.8 c-0.4 1.5 0.7 1.4 1.6 -0.20.3 -0.5 0.2 -0.6 -0.5 -0.6-0.6 -0.0 -0.9 0.2 -1.1 0.8zM150.0 162.7 c-0.6 1.0 -0.6 1.1 -0.1 2.0 l0.6 0.90.6 -1.3 c1.2 -2.5 0.3 -3.9 -1.1 -1.7zM123.0 178.3 c-0.4 0.3 -0.3 0.3 0.3 0.30.4 -0.1 0.9 0.1 1.0 0.40.1 0.3 0.0 0.4 -0.4 0.3-0.4 -0.1 -0.4 -0.0 -0.3 0.50.1 0.5 0.0 0.6 -0.7 0.6-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.6 -1.0 0.4 -1.2 -0.3-0.4 -1.0 -1.4 -1.5 -1.4 -0.60.0 0.7 1.0 1.5 2.2 1.70.6 0.1 1.4 0.5 1.8 0.81.1 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.7 -1.1-0.3 -0.0 -0.9 -0.4 -1.2 -0.8-0.4 -0.4 -0.9 -0.8 -1.3 -0.9-0.3 -0.1 -0.6 -0.2 -0.5 -0.30.6 -0.7 -1.1 -1.5 -2.0 -1.0zM130.7 180.1 c-0.2 0.2 -0.4 0.9 -0.4 1.40.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.4 0.1 0.7 -1.7 0.3 -1.9-0.2 -0.1 -0.5 -0.0 -0.7 0.2z","role":"hi"},{"d":"M123.1 160.7 c-0.5 0.8 -0.5 1.0 0.0 1.00.2 -0.0 0.7 0.4 1.1 0.90.9 1.2 1.7 0.9 3.1 -1.0 l1.1 -1.5-2.4 -0.0 c-2.1 -0.0 -2.5 0.1 -2.9 0.7zM157.0 160.5 c0.0 0.6 0.9 1.5 1.9 1.91.3 0.5 1.4 0.4 1.4 -1.0 l0.0 -1.3-1.7 -0.0 c-1.2 -0.0 -1.7 0.1 -1.7 0.5zM120.9 163.4 c-0.5 0.7 -1.2 1.3 -1.4 1.3-0.3 -0.0 -0.5 0.1 -0.5 0.30.0 0.1 -0.8 1.1 -1.8 2.2-1.0 1.0 -1.8 2.0 -1.8 2.20.0 0.2 -0.3 0.6 -0.6 0.9-0.6 0.6 -1.1 2.1 -2.1 5.9-0.4 1.8 -0.8 10.2 -0.5 11.40.2 0.7 0.4 0.8 1.6 0.8 l1.5 -0.0-0.3 -1.5 c-0.6 -2.8 -0.3 -4.4 0.9 -4.30.5 0.0 1.1 0.3 1.2 0.50.2 0.3 0.4 0.3 1.1 -0.00.7 -0.4 1.1 -0.4 2.0 -0.00.7 0.2 1.4 0.3 1.6 0.20.3 -0.1 -0.3 -0.5 -2.0 -1.0-3.0 -1.1 -4.4 -9.9 -1.5 -9.90.6 -0.0 2.7 1.8 2.7 2.30.0 0.5 2.9 2.0 3.8 2.00.5 -0.0 2.0 -0.3 3.3 -0.74.1 -1.1 6.7 -0.9 12.7 0.90.4 0.1 0.6 -0.0 0.6 -0.30.0 -0.5 4.1 -2.6 4.9 -2.60.2 -0.0 0.7 -0.4 1.2 -0.90.5 -0.6 1.0 -0.8 1.3 -0.70.3 0.1 0.5 0.0 0.5 -0.30.0 -0.6 -0.9 -0.6 -2.8 -0.1-2.7 0.7 -3.6 1.0 -4.3 1.7-0.4 0.4 -1.0 0.6 -1.5 0.5-0.5 -0.1 -0.9 0.0 -1.0 0.3-0.3 0.8 -3.7 0.1 -5.0 -0.9-0.3 -0.2 -2.6 -0.4 -5.9 -0.4-6.6 -0.1 -8.6 -0.7 -8.8 -2.9-0.1 -1.4 1.4 -4.0 2.3 -4.01.7 -0.0 2.0 -2.1 0.4 -2.8-0.4 -0.2 -0.7 -0.6 -0.8 -0.9-0.1 -0.4 -0.4 -0.2 -1.0 0.7zM148.9 174.3 c-0.4 1.7 -0.3 3.9 0.2 3.30.5 -0.5 0.7 -1.9 0.4 -2.7-0.1 -0.3 -0.2 -0.7 -0.3 -1.0-0.1 -0.3 -0.2 -0.2 -0.4 0.3zM147.5 179.1 c-0.5 0.5 -1.4 0.9 -1.8 0.9 l-0.9 0.00.8 0.6 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.13.2 -1.6 7.2 -0.8 8.7 1.80.7 1.2 0.9 1.4 1.9 1.30.6 -0.0 0.9 -0.1 0.5 -0.2-0.4 -0.1 -1.0 -0.9 -1.7 -2.2-0.7 -1.5 -1.4 -2.3 -2.4 -3.0-1.5 -1.0 -1.9 -2.3 -0.9 -2.80.3 -0.2 0.5 -0.6 0.4 -1.1 l-0.1 -0.8-1.0 0.9zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.3 0.8 0.7 1.4 0.80.6 0.1 1.6 0.3 2.2 0.41.5 0.3 1.1 -0.3 -0.7 -1.2-0.8 -0.4 -1.7 -1.0 -2.1 -1.3-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7z","role":"ink"}],"headJaw":[{"d":"M114.0 193.2 l0.0 13.215.3 -0.015.3 -0.00.9 -1.1 c0.5 -0.6 1.3 -1.9 1.8 -2.90.4 -1.0 1.0 -2.2 1.3 -2.70.3 -0.5 0.5 -1.5 0.5 -2.30.0 -1.8 0.4 -2.3 2.2 -2.31.7 -0.0 2.5 -0.8 2.5 -2.50.0 -0.7 0.1 -2.0 0.2 -2.80.2 -1.4 0.2 -1.5 -0.5 -1.8-1.0 -0.4 -2.1 -3.3 -2.3 -5.9 l-0.1 -1.9-18.5 -0.0-18.5 -0.00.0 13.2z m7.1 -9.7 c0.3 0.3 0.6 0.5 0.8 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.21.5 -0.0 2.1 3.1 0.9 4.1-1.2 1.0 -6.5 0.6 -7.5 -0.6-0.4 -0.5 -0.9 -0.9 -1.0 -0.9-1.5 -0.0 -2.3 -1.9 -1.9 -4.30.3 -1.6 3.4 -2.4 4.6 -1.2zM153.8 201.9 c-0.6 0.9 -0.8 1.7 -0.8 2.8 l0.0 1.61.7 -0.01.7 -0.00.0 -2.8 c0.0 -3.4 -0.9 -4.0 -2.5 -1.6z","role":"wash"},{"d":"M119.8 180.6 c0.3 0.3 1.1 0.7 1.7 0.80.6 0.1 1.5 0.5 1.9 0.81.2 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.6 -1.1-0.3 -0.0 -0.9 -0.3 -1.2 -0.7-0.7 -0.8 -4.5 -0.7 -4.7 0.0-0.2 0.5 -1.1 0.4 -1.3 -0.2-0.1 -0.3 -0.5 -0.5 -0.9 -0.5-0.6 -0.0 -0.6 0.0 -0.1 0.6zM130.6 180.2 c-0.1 0.1 -0.2 0.7 -0.3 1.20.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.2 0.0 0.4 -0.3 0.5 -0.90.1 -0.9 -0.4 -1.2 -1.0 -0.7zM128.0 188.9 c0.0 0.3 1.7 1.4 2.1 1.40.6 -0.0 0.1 -0.9 -0.6 -1.3-1.0 -0.5 -1.5 -0.5 -1.5 -0.1z","role":"hi"},{"d":"M114.0 185.6 c0.0 3.5 0.1 5.7 0.3 5.90.2 0.1 0.3 0.6 0.3 1.1-0.0 0.5 0.3 1.4 0.7 2.10.4 0.6 0.9 1.7 1.1 2.30.4 1.6 3.3 4.7 5.0 5.50.8 0.3 1.4 0.7 1.4 0.90.0 0.9 2.8 1.6 4.6 1.32.2 -0.4 6.4 -1.9 7.0 -2.40.3 -0.3 0.7 -0.2 1.5 0.22.5 1.3 5.0 0.9 6.9 -1.0 l1.1 -1.1-1.2 0.2 c-0.6 0.1 -1.4 0.1 -1.8 -0.1-0.4 -0.2 -0.8 -0.2 -1.3 -0.0-0.9 0.4 -2.9 0.1 -3.6 -0.6-1.5 -1.4 -6.9 -3.5 -9.7 -3.6-1.6 -0.1 -3.1 -0.4 -3.7 -0.7-2.1 -1.1 -3.0 -0.6 -1.6 0.71.6 1.5 3.8 2.2 8.5 2.61.0 0.1 1.6 0.3 1.6 0.60.0 0.8 -5.6 1.0 -7.2 0.3-3.5 -1.5 -4.2 -2.0 -4.5 -2.9-0.2 -0.5 -0.5 -1.0 -0.8 -1.1-0.3 -0.1 -0.5 -0.5 -0.5 -0.90.0 -0.4 -0.5 -1.1 -1.0 -1.6-0.8 -0.7 -1.0 -1.1 -0.9 -1.90.0 -0.6 -0.1 -1.3 -0.3 -1.6-0.5 -0.6 -1.2 -4.8 -1.0 -6.30.1 -1.2 1.7 -1.6 2.3 -0.60.2 0.3 0.4 0.3 1.1 -0.00.6 -0.4 1.1 -0.4 1.9 -0.10.6 0.2 1.3 0.4 1.5 0.40.8 -0.0 0.2 -0.6 -0.9 -0.8-0.5 -0.1 -1.4 -0.7 -2.0 -1.4-0.9 -1.0 -1.2 -1.1 -2.9 -1.1 l-1.8 -0.00.0 5.6zM145.6 180.7 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.03.3 -1.7 7.0 -0.9 8.8 2.01.4 2.2 1.5 2.8 0.3 3.0-0.5 0.1 -1.4 0.6 -1.9 1.0-0.7 0.6 -1.4 0.8 -2.7 0.8-2.5 0.0 -3.5 1.0 -2.6 2.60.2 0.3 0.4 0.8 0.5 1.20.1 0.3 0.6 0.8 1.2 1.21.2 0.7 1.3 1.7 0.3 3.7-0.4 0.7 -0.7 1.5 -0.7 1.70.0 0.2 -0.2 0.5 -0.4 0.8-0.2 0.2 -0.8 1.0 -1.4 1.8 l-0.9 1.30.9 0.1 c1.1 0.2 2.6 -1.4 3.7 -3.90.5 -1.0 1.0 -2.1 1.1 -2.40.2 -0.3 0.3 -1.3 0.3 -2.20.0 -1.9 0.7 -2.7 2.1 -2.71.8 -0.0 2.9 -1.5 3.0 -4.30.1 -2.3 0.0 -2.5 -0.6 -2.6-0.4 -0.1 -1.0 -0.8 -1.7 -2.2-0.7 -1.5 -1.4 -2.3 -2.4 -3.0-1.3 -0.8 -1.4 -1.1 -1.3 -1.90.1 -0.9 0.1 -1.0 -1.4 -1.0 l-1.5 0.00.8 0.6zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.4 0.9 0.7 1.8 0.82.9 0.4 5.3 3.8 2.6 3.6-0.6 -0.0 -3.5 -0.2 -6.3 -0.4-6.3 -0.4 -6.7 -0.3 -6.6 1.1 l0.1 1.04.8 -0.0 c3.6 -0.0 5.3 -0.2 6.8 -0.61.2 -0.4 2.3 -0.5 2.9 -0.41.1 0.2 1.5 -1.0 0.5 -1.4-0.3 -0.1 -0.5 -0.4 -0.5 -0.60.0 -0.7 -2.1 -2.7 -3.3 -3.4-0.6 -0.3 -1.5 -0.8 -1.8 -1.1-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7zM155.5 201.1 c-1.3 1.7 -2.1 3.2 -2.3 4.1 l-0.2 1.11.6 -0.01.7 -0.00.0 -2.8 c0.0 -2.8 -0.2 -3.3 -0.8 -2.4z","role":"ink"}],"body":[{"d":"M196.5 120.7 c0.8 0.8 1.4 1.0 3.4 1.31.3 0.2 2.7 0.5 3.2 0.60.4 0.2 1.8 0.5 3.1 0.73.6 0.5 5.2 1.3 5.2 2.50.0 1.0 0.7 2.5 0.7 1.60.0 -0.5 2.0 -0.5 2.8 0.10.3 0.2 0.6 0.8 0.6 1.20.0 0.6 0.2 0.7 1.6 0.74.3 0.1 5.3 4.1 1.6 6.1-1.3 0.7 -1.9 1.2 -2.0 1.8-0.1 0.5 -0.3 1.2 -0.5 1.6-0.4 1.0 -0.4 2.4 0.0 2.40.5 -0.0 0.4 1.5 -0.2 2.2-0.3 0.4 -0.5 1.4 -0.5 2.30.0 1.7 -0.8 5.0 -1.4 5.6-0.5 0.5 -0.4 1.7 0.2 2.10.3 0.3 0.5 1.6 0.7 4.30.1 2.2 0.4 4.5 0.7 5.30.3 0.7 0.6 2.0 0.8 2.80.2 0.8 0.8 2.4 1.3 3.40.5 1.1 1.0 2.2 1.0 2.40.0 0.2 0.5 0.9 1.0 1.40.6 0.6 1.0 1.1 1.0 1.30.0 0.2 0.4 0.8 1.0 1.41.2 1.4 1.6 2.5 1.0 3.6-0.9 1.7 -4.0 0.7 -4.0 -1.40.0 -0.3 -0.4 -0.8 -0.8 -1.2-0.5 -0.4 -0.8 -0.8 -0.8 -1.00.0 -0.2 -0.4 -0.9 -0.9 -1.5-1.3 -1.4 -2.5 -2.9 -2.9 -3.5-2.3 -3.4 -3.2 -5.3 -3.2 -7.00.0 -0.6 -0.5 -2.1 -1.1 -3.4-0.6 -1.3 -1.3 -3.0 -1.5 -3.9-0.2 -0.8 -0.6 -1.8 -0.9 -2.2-1.0 -1.1 -2.6 -3.6 -2.9 -4.4-0.1 -0.4 -0.5 -0.9 -0.9 -1.1-0.3 -0.2 -0.6 -0.5 -0.6 -0.80.0 -0.2 -0.5 -0.8 -1.0 -1.4-0.5 -0.5 -1.2 -1.4 -1.4 -1.9-0.2 -0.6 -0.7 -1.0 -1.0 -1.0-1.1 -0.0 -1.9 -0.7 -1.9 -1.50.0 -0.6 -0.3 -1.0 -1.5 -1.5-0.8 -0.4 -1.6 -1.0 -1.7 -1.3-0.1 -0.4 -0.5 -0.8 -0.8 -0.9-0.3 -0.1 -0.8 -0.6 -0.9 -1.2-0.2 -0.5 -0.8 -1.2 -1.4 -1.5-0.6 -0.3 -1.0 -0.7 -1.0 -0.90.0 -0.2 -0.3 -0.4 -0.6 -0.5-0.3 -0.1 -1.1 -0.8 -1.8 -1.5-1.2 -1.3 -8.4 -5.3 -9.6 -5.3-0.3 -0.0 -1.1 -0.3 -1.8 -0.7-2.1 -1.1 -2.6 -1.3 -4.9 -1.7-1.2 -0.2 -3.2 -0.7 -4.4 -1.0-3.7 -1.1 -15.5 -1.1 -16.9 0.1-0.8 0.6 -4.1 1.6 -5.6 1.6-0.7 -0.0 -1.9 0.3 -2.6 0.7-0.7 0.4 -1.9 0.8 -2.6 0.9-0.8 0.1 -1.6 0.4 -1.8 0.7-0.3 0.3 -1.3 0.7 -2.2 1.0-1.0 0.3 -1.9 0.6 -2.1 0.8-0.2 0.1 -0.8 0.3 -1.3 0.5-0.5 0.1 -1.6 0.8 -2.3 1.4-0.8 0.6 -1.5 1.1 -1.8 1.1-0.2 -0.0 -0.5 0.3 -0.6 0.6-0.2 0.7 -1.7 1.7 -3.9 2.6-0.8 0.4 -1.7 0.8 -1.9 1.1-0.2 0.2 -0.6 0.4 -0.8 0.4-0.5 -0.0 -4.8 4.1 -6.8 6.4-0.7 0.9 -1.5 1.6 -1.7 1.6-0.7 -0.0 -2.1 1.4 -2.1 2.10.0 0.9 -0.6 1.9 -1.2 1.9-0.3 -0.0 -0.5 0.3 -0.5 0.60.0 0.4 -0.3 0.7 -0.6 0.9-0.3 0.1 -1.0 0.9 -1.4 1.7-0.4 0.8 -1.1 1.6 -1.4 1.7-0.3 0.1 -0.6 0.4 -0.6 0.60.0 0.2 -0.6 1.1 -1.3 1.9-0.8 0.9 -1.3 1.8 -1.3 2.30.0 1.0 -0.6 2.1 -1.1 2.1-0.6 -0.0 -1.7 2.8 -2.9 7.2-0.7 3.0 -0.9 10.4 -0.2 11.20.4 0.5 0.5 1.0 0.4 1.8-0.1 0.8 0.0 1.7 0.5 2.70.5 1.1 0.6 1.9 0.5 2.8-0.2 1.1 -0.1 1.3 0.4 1.50.5 0.1 0.7 0.5 0.9 2.10.3 2.6 0.5 3.1 1.0 3.10.3 -0.0 0.5 0.2 0.5 0.50.0 0.2 0.2 0.6 0.4 0.70.2 0.1 0.8 1.1 1.4 2.20.5 1.1 1.3 2.2 1.6 2.40.3 0.2 0.8 1.1 0.9 1.90.5 2.2 2.0 4.2 3.9 5.21.4 0.7 1.7 1.0 1.9 2.10.2 1.1 1.7 3.1 2.4 3.10.2 -0.0 37.0 -23.1 38.6 -24.10.5 -0.4 0.5 -0.5 -0.1 -2.2-0.4 -1.0 -0.8 -2.8 -0.9 -4.0-0.1 -1.2 -0.4 -2.5 -0.6 -2.9-0.4 -0.8 -0.5 -2.2 -0.2 -2.60.1 -0.1 0.2 -0.8 0.1 -1.4-0.1 -0.8 0.1 -1.1 0.4 -1.10.3 -0.0 0.7 -0.5 0.9 -1.10.3 -0.9 0.6 -1.2 1.3 -1.31.1 -0.1 1.3 -1.0 0.5 -1.8-1.8 -1.8 -2.4 -4.4 -1.3 -6.00.3 -0.4 0.6 -1.3 0.7 -2.10.6 -4.5 2.5 -5.1 3.9 -1.20.6 1.4 1.9 2.6 3.1 2.60.2 -0.0 0.5 0.6 0.8 1.30.3 0.8 0.7 1.5 1.3 1.71.5 0.8 2.5 1.9 3.4 3.80.4 1.0 1.0 1.8 1.2 1.80.6 -0.0 0.5 2.1 -0.1 4.3-0.3 1.0 -0.5 3.0 -0.5 4.40.0 1.8 -0.1 2.7 -0.5 2.9-0.7 0.6 -0.9 2.0 -0.6 3.20.3 1.0 0.3 1.0 1.1 -1.00.4 -1.1 1.1 -2.4 1.5 -2.90.4 -0.5 1.0 -1.6 1.5 -2.40.4 -0.8 0.9 -1.6 1.0 -1.70.2 -0.1 0.3 -1.7 0.3 -3.5 l0.0 -3.32.3 -0.0 c2.5 -0.0 2.8 0.2 5.7 2.7 l1.7 1.50.2 3.4 c0.2 2.4 0.1 3.4 -0.2 3.6-0.2 0.1 -0.4 0.7 -0.4 1.20.0 0.5 -0.7 2.4 -1.7 4.1-0.9 1.7 -1.7 3.3 -1.7 3.60.0 0.3 -0.8 1.6 -1.7 3.0-1.7 2.5 -2.6 4.5 -2.2 4.90.4 0.3 1.0 -0.5 1.5 -2.20.4 -1.2 0.7 -1.6 1.3 -1.71.0 -0.1 2.1 -1.9 2.1 -3.20.0 -1.2 0.3 -1.7 1.2 -2.00.7 -0.2 1.4 -1.5 2.2 -4.00.3 -1.0 0.7 -1.5 1.5 -1.80.6 -0.3 1.2 -0.8 1.3 -1.10.1 -0.4 0.5 -0.6 1.0 -0.61.0 -0.0 1.0 -0.5 -0.0 -0.9-1.2 -0.6 -1.2 -2.1 0.1 -2.11.1 -0.0 1.7 0.5 1.7 1.60.0 0.4 0.2 0.8 0.3 0.80.2 -0.0 0.3 0.4 0.3 0.80.0 0.7 -0.2 0.8 -0.8 0.8-2.2 -0.0 -1.6 3.8 0.6 4.22.1 0.4 3.7 3.4 2.2 4.3-0.2 0.1 -0.4 0.9 -0.6 1.7-0.1 1.0 -0.4 1.5 -0.7 1.5-0.3 -0.0 -0.5 0.3 -0.5 0.6-0.0 0.3 -0.4 1.2 -0.8 1.9-0.5 0.7 -1.0 1.7 -1.1 2.0-0.2 0.4 -0.5 0.8 -0.6 0.8-0.4 0.2 -3.1 4.7 -3.1 5.20.0 0.2 -0.2 0.4 -0.4 0.4-0.2 -0.0 -1.0 0.9 -1.6 2.0-0.6 1.1 -1.6 2.5 -2.1 3.1-0.5 0.6 -0.9 1.2 -0.9 1.50.0 0.3 -0.5 1.0 -1.2 1.6-0.6 0.6 -1.2 1.3 -1.2 1.60.0 0.2 -0.6 1.0 -1.3 1.8-0.7 0.8 -1.3 1.6 -1.3 1.90.0 0.3 -0.3 0.8 -0.7 1.1-0.6 0.5 -0.6 0.8 -0.3 1.30.5 0.9 0.9 0.9 1.6 -0.20.3 -0.5 1.3 -1.8 2.3 -2.80.9 -1.0 2.1 -2.5 2.5 -3.31.0 -2.0 4.7 -7.7 6.0 -9.10.5 -0.6 1.8 -2.8 2.7 -4.71.0 -2.0 2.4 -4.4 3.2 -5.41.7 -2.0 2.3 -3.2 2.7 -4.80.4 -1.8 3.8 -0.2 3.8 1.80.0 0.2 0.5 0.9 1.2 1.51.1 0.9 1.2 1.2 1.0 2.1-0.1 0.6 -0.3 2.4 -0.4 4.0 l-0.2 2.9-1.9 2.1 c-1.7 1.9 -4.5 6.5 -6.1 9.9-0.3 0.6 -1.4 2.1 -2.3 3.2-1.8 2.0 -2.4 2.8 -4.3 5.7-0.6 0.8 -1.4 1.7 -1.9 2.0-1.7 1.0 -3.6 4.1 -3.0 5.00.4 0.8 0.5 0.8 0.8 0.20.2 -0.4 0.9 -1.2 1.6 -1.80.7 -0.7 1.2 -1.4 1.2 -1.60.0 -0.2 0.5 -0.6 1.1 -0.91.2 -0.6 3.9 -3.3 3.9 -3.90.0 -0.2 0.3 -0.8 0.8 -1.30.4 -0.5 1.1 -1.4 1.6 -2.00.5 -0.6 1.4 -1.7 2.1 -2.40.7 -0.7 1.2 -1.5 1.2 -1.70.0 -0.2 0.3 -0.6 0.6 -0.80.4 -0.3 0.7 -1.0 0.9 -2.00.1 -0.9 0.3 -1.7 0.4 -1.80.7 -0.5 1.7 -2.7 1.7 -3.60.0 -0.9 0.9 -2.7 1.7 -3.40.1 -0.1 0.3 -0.4 0.4 -0.80.4 -1.0 2.3 -0.6 2.8 0.50.2 0.5 0.6 0.9 0.7 0.90.2 -0.0 0.4 0.3 0.5 0.70.1 0.4 0.4 1.0 0.7 1.40.3 0.3 0.7 0.9 0.9 1.30.2 0.4 0.7 0.8 1.2 0.90.5 0.1 1.0 0.6 1.3 1.30.3 0.6 1.0 1.3 1.7 1.60.6 0.3 1.5 0.9 1.8 1.31.2 1.3 3.1 2.5 5.6 3.71.3 0.6 3.3 1.6 4.3 2.22.6 1.4 5.4 1.9 6.5 1.30.5 -0.3 2.1 -0.5 3.9 -0.51.7 -0.0 3.4 -0.1 3.8 -0.10.4 -0.0 1.1 -0.6 1.6 -1.50.5 -0.8 1.6 -1.9 2.4 -2.41.0 -0.7 1.6 -1.3 1.8 -2.00.1 -0.5 0.4 -1.1 0.5 -1.20.1 -0.1 0.6 -0.7 1.1 -1.41.4 -2.0 3.8 -4.7 4.3 -4.81.7 -0.5 1.9 -4.1 1.0 -13.8-1.2 -12.7 -1.2 -12.8 -2.1 -13.5-0.4 -0.4 -1.1 -0.9 -1.4 -1.2-0.4 -0.3 -0.9 -1.0 -1.3 -1.7-0.4 -0.6 -0.9 -1.5 -1.2 -2.0-0.3 -0.5 -0.8 -1.2 -1.0 -1.8-0.2 -0.5 -0.5 -0.9 -0.6 -0.9-0.1 -0.0 -0.4 -0.4 -0.5 -0.8-0.1 -0.4 -0.6 -1.3 -1.2 -1.9-0.5 -0.6 -1.2 -1.9 -1.5 -2.9-0.4 -1.0 -0.8 -2.1 -1.0 -2.3-1.5 -2.0 -2.5 -5.4 -2.7 -9.4 l-0.2 -4.11.1 -2.3 c0.7 -1.5 1.4 -2.4 2.1 -2.80.5 -0.3 1.3 -0.8 1.7 -1.12.6 -2.2 9.3 -0.8 11.6 2.50.4 0.6 1.3 1.6 1.9 2.10.6 0.6 1.1 1.3 1.1 1.60.0 0.3 0.2 0.9 0.5 1.51.3 2.6 1.7 4.3 1.7 7.90.0 3.4 -0.6 8.8 -1.0 9.1-0.6 0.4 -1.0 3.5 -0.9 6.70.3 4.2 -0.5 13.7 -1.1 13.9-0.6 0.2 -0.6 1.4 0.2 3.50.3 1.0 0.8 2.4 1.0 3.30.2 0.8 0.5 1.9 0.7 2.50.2 0.5 0.5 1.6 0.7 2.40.2 0.8 0.5 1.4 0.6 1.40.3 -0.0 1.1 2.1 1.7 4.40.3 0.9 0.9 2.2 1.7 3.11.1 1.3 1.4 1.8 1.6 3.70.2 1.2 0.4 2.3 0.6 2.60.2 0.2 0.4 1.0 0.5 1.60.1 0.8 0.5 1.4 0.9 1.61.1 0.6 0.8 4.4 -0.4 4.7-1.2 0.3 -1.1 1.1 0.2 1.31.1 0.2 1.6 1.6 0.9 2.4-0.8 1.0 -2.2 10.9 -1.8 13.20.3 1.9 -0.3 2.4 -2.9 2.4-1.4 0.0 -1.7 0.1 -1.0 0.30.5 0.1 4.8 0.3 9.6 0.3 l8.7 0.00.0 -10.70.0 -10.7-1.1 -0.0-1.1 -0.0-0.4 -2.4 c-0.2 -1.3 -0.6 -3.9 -0.9 -5.7-0.3 -1.8 -0.7 -3.7 -0.8 -4.1-0.6 -1.4 -0.4 -3.7 0.3 -3.80.3 -0.0 0.4 -0.1 0.2 -0.2-0.2 -0.1 -0.4 -0.7 -0.4 -1.40.0 -1.5 -0.5 -2.5 -0.9 -1.6-0.1 0.3 -0.6 0.6 -1.0 0.6-0.8 -0.0 -1.4 -1.7 -1.0 -2.80.1 -0.3 -0.1 -0.9 -0.6 -1.3-0.7 -0.7 -0.8 -1.1 -0.8 -3.3 l-0.0 -2.51.5 -0.0 c1.4 -0.0 1.5 -0.1 1.5 -0.80.0 -0.5 0.2 -0.9 0.3 -1.00.2 -0.1 0.3 -0.6 0.3 -1.00.0 -1.2 0.7 -1.5 2.9 -1.6 l2.1 -0.10.0 -3.9 c0.0 -3.0 -0.1 -3.9 -0.4 -3.9-0.5 -0.0 -1.4 -3.9 -1.1 -4.80.1 -0.3 0.5 -0.6 0.9 -0.60.6 -0.0 0.6 -0.3 0.6 -4.3-0.0 -2.3 -0.1 -4.0 -0.2 -3.7-0.2 0.7 -1.8 0.8 -1.8 0.2-0.0 -0.7 -0.9 1.9 -0.9 2.8-0.0 0.7 -0.2 0.9 -0.8 0.9-1.3 -0.0 -2.3 -3.0 -2.0 -5.60.1 -1.2 0.1 -2.0 -0.1 -2.0-0.7 -0.0 -2.1 0.9 -2.1 1.20.0 0.2 -0.4 0.4 -0.9 0.4 l-0.9 -0.00.2 -5.6 c0.3 -8.4 0.4 -9.3 1.3 -9.50.4 -0.1 0.8 -0.4 0.8 -0.70.1 -0.5 -0.1 -0.5 -1.1 -0.4-1.0 0.2 -1.2 0.1 -1.4 -0.5-0.1 -0.4 -0.3 -0.7 -0.4 -0.7-0.8 -0.0 -2.0 -1.7 -2.2 -3.1-0.2 -1.5 -0.2 -1.6 0.7 -1.60.5 -0.0 0.9 0.2 0.9 0.30.0 0.2 0.5 0.6 1.1 0.81.2 0.5 1.9 -0.1 1.9 -1.90.0 -0.9 0.0 -0.9 2.2 -0.9 l2.2 -0.0-0.0 2.2 c-0.1 3.1 -0.1 3.1 0.6 3.11.1 -0.0 2.3 7.7 1.4 9.0-0.3 0.5 -0.3 0.6 0.2 0.60.3 -0.0 0.8 0.4 1.0 0.90.3 0.7 0.4 -4.5 0.4 -19.7 l0.0 -20.6-2.1 -0.0-2.2 -0.00.2 1.2 c0.2 1.1 0.1 1.2 -0.7 1.2-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.4 0.6 0.5 2.2 0.41.0 -0.1 1.1 0.0 1.3 1.10.3 1.5 0.0 1.9 -1.3 1.8-2.6 -0.2 -3.9 0.3 -3.6 1.50.2 1.3 0.1 1.2 1.6 1.52.0 0.3 2.5 1.8 1.1 3.2-0.7 0.7 -0.8 1.1 -0.8 3.6-0.1 5.0 -0.0 4.9 -1.1 5.3-0.8 0.3 -0.9 0.5 -0.8 1.50.1 1.1 0.1 1.2 -0.7 1.2-1.4 -0.0 -2.3 -1.0 -2.1 -2.60.1 -1.2 0.0 -1.3 -0.8 -1.4-0.5 -0.1 -0.9 -0.3 -0.9 -0.60.0 -0.3 -0.3 -0.5 -0.7 -0.5-0.4 -0.0 -0.9 -0.2 -1.2 -0.5-0.3 -0.3 -0.9 -0.5 -1.5 -0.5-1.5 -0.0 -1.8 -1.6 -0.7 -4.20.7 -1.6 0.7 -2.5 -0.1 -2.5-0.5 -0.0 -1.2 -10.0 -0.7 -10.70.1 -0.2 -10.7 -0.3 -30.6 -0.3 l-30.8 -0.01.0 1.0z m56.3 2.9 c0.1 1.3 0.4 3.0 0.6 4.00.5 2.3 -1.7 4.3 -3.2 2.8-0.2 -0.2 -0.6 -0.4 -0.9 -0.4-0.5 -0.0 -3.4 -1.3 -3.7 -1.7-0.1 -0.1 -0.6 -0.4 -1.2 -0.6-1.8 -0.6 -1.1 -3.8 0.9 -3.80.6 -0.0 1.4 -0.4 2.1 -1.00.6 -0.6 1.4 -1.0 1.7 -1.00.3 -0.0 0.6 -0.2 0.7 -0.30.1 -0.2 0.8 -0.3 1.5 -0.3 l1.3 -0.00.2 2.2z m-30.3 -1.7 c0.6 0.2 0.7 1.8 0.2 1.8-0.2 -0.0 -0.3 0.2 -0.3 0.50.0 1.1 -2.5 0.5 -2.6 -0.6-0.2 -1.6 1.1 -2.3 2.7 -1.7z m-71.4 33.9 c0.9 1.0 0.6 3.5 -0.6 4.4-0.7 0.6 -0.8 1.1 -1.1 3.7-0.3 3.1 -0.8 4.8 -1.4 4.8-0.2 -0.0 -0.3 0.3 -0.3 0.60.0 1.4 -1.6 1.8 -3.9 0.9-1.0 -0.4 -3.7 -0.6 -6.1 -0.5-1.0 0.1 -0.2 1.5 0.9 1.72.1 0.3 3.3 1.0 3.3 1.90.0 1.5 -10.9 0.5 -12.2 -1.1-0.8 -1.0 -1.7 -1.1 -2.0 -0.2-0.8 2.5 -7.8 0.9 -7.8 -1.80.0 -1.3 1.5 -3.8 2.3 -3.80.9 -0.0 1.7 -1.1 1.7 -2.10.0 -0.8 0.1 -0.9 0.9 -0.90.6 -0.0 1.2 -0.4 1.7 -1.12.0 -2.4 4.0 -3.9 5.0 -3.91.5 -0.0 3.3 -0.6 4.3 -1.52.0 -1.7 13.9 -2.6 15.2 -1.1z m114.6 13.6 c0.0 0.8 -0.1 1.0 -0.8 1.0-0.7 -0.0 -0.8 -0.2 -0.8 -1.00.0 -0.8 0.1 -1.0 0.8 -1.00.7 -0.0 0.8 0.2 0.8 1.0z m-0.1 3.4 c0.9 0.9 0.5 1.9 -0.8 1.9-1.1 -0.0 -1.2 -0.1 -1.2 -1.20.0 -1.2 1.0 -1.7 1.9 -0.8z m2.1 3.3 c0.1 0.5 0.2 0.6 0.3 0.40.1 -0.9 1.6 -0.4 2.0 0.60.6 1.5 0.1 6.3 -0.6 6.3-0.4 -0.0 -0.8 0.6 -1.2 1.5-0.4 0.8 -0.7 1.5 -0.9 1.5-0.1 -0.0 -0.4 0.5 -0.5 1.2-0.2 1.1 -0.3 1.2 -1.4 1.2-0.9 -0.0 -1.2 0.1 -1.2 0.60.0 0.7 1.1 2.1 1.6 2.10.2 -0.0 0.4 0.7 0.4 1.7 l0.0 1.7-1.2 -0.1 c-1.1 -0.1 -1.3 -0.2 -1.6 -1.2-0.2 -0.6 -0.4 -1.1 -0.6 -1.1-0.1 -0.0 -0.3 -0.2 -0.3 -0.50.0 -0.3 -0.3 -0.6 -0.6 -0.7-0.4 -0.1 -0.7 -0.4 -0.9 -0.8-0.2 -0.7 0.5 -6.4 0.8 -6.40.1 -0.0 0.3 -0.9 0.5 -2.10.5 -4.8 0.5 -4.9 1.4 -4.90.8 -0.0 0.8 -0.0 0.7 2.8-0.1 2.8 -0.1 2.8 0.6 2.80.4 -0.0 0.7 -0.2 0.7 -0.30.0 -0.2 0.5 -0.3 1.0 -0.31.0 -0.0 1.1 -0.0 0.9 -1.4-0.2 -1.4 -0.6 -2.1 -0.8 -1.4-0.6 1.7 -1.8 0.4 -1.8 -1.9 l0.0 -2.31.1 0.1 c0.8 0.1 1.1 0.3 1.3 0.9z m-146.6 7.5 c0.3 0.3 0.6 0.5 0.8 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.21.5 -0.0 2.1 3.1 0.9 4.1-1.2 1.0 -6.5 0.6 -7.5 -0.6-0.4 -0.5 -0.9 -0.9 -1.0 -0.9-1.5 -0.0 -2.3 -1.9 -1.9 -4.30.3 -1.6 3.4 -2.4 4.6 -1.2z m110.1 7.3 c0.2 2.1 -2.6 3.3 -4.9 2.1-1.7 -0.9 -1.4 -2.2 0.5 -2.20.9 -0.0 1.6 -0.2 1.9 -0.50.8 -0.9 2.4 -0.5 2.5 0.6z m39.3 50.4 c1.3 2.8 1.1 3.2 -1.6 3.2 l-2.4 -0.00.2 -1.6 c0.5 -3.7 2.4 -4.5 3.8 -1.6z","role":"wash"},{"d":"M253.5 120.7 c-0.1 0.6 -0.1 1.3 0.1 1.50.1 0.3 0.2 0.8 0.1 1.1-0.1 0.4 0.0 0.5 0.5 0.40.5 -0.2 0.6 0.1 0.6 2.3-0.0 2.4 -0.7 5.6 -1.2 5.6-0.1 -0.0 -0.1 0.6 0.1 1.40.2 0.9 0.2 1.5 -0.0 1.8-0.4 0.4 -0.1 1.5 0.3 1.50.1 -0.0 0.3 -0.6 0.4 -1.30.1 -0.7 0.4 -2.6 0.7 -4.20.3 -1.7 0.6 -4.8 0.7 -7.1 l0.2 -4.1-1.1 -0.0 c-0.9 -0.0 -1.1 0.1 -1.3 1.0zM151.3 140.6 c0.0 0.1 -0.7 0.4 -1.5 0.7-0.8 0.2 -1.5 0.6 -1.5 0.70.0 0.6 -0.9 0.3 -1.1 -0.4-0.2 -0.9 -1.1 -0.8 -1.8 0.2-0.7 1.0 -1.5 1.1 -3.1 0.1-1.3 -0.8 -2.7 -0.8 -4.9 -0.0-0.7 0.3 -2.0 0.6 -2.9 0.7-0.8 0.1 -1.8 0.3 -2.2 0.5-0.4 0.2 -1.7 0.9 -2.9 1.5-1.2 0.6 -2.4 1.3 -2.6 1.6-0.2 0.3 -1.1 0.9 -1.9 1.3-4.5 2.5 -6.7 4.1 -6.7 5.00.0 0.2 -0.9 1.3 -2.1 2.3-1.2 1.1 -2.5 2.8 -3.1 3.9-0.6 1.1 -1.1 1.9 -1.2 1.9-0.2 -0.0 -0.3 0.2 -0.3 0.40.0 0.2 -0.4 0.8 -1.0 1.2-1.1 1.0 -1.7 2.0 -1.5 2.60.2 0.6 0.7 0.5 0.9 -0.20.1 -0.3 1.0 -1.4 1.9 -2.41.0 -1.0 2.3 -2.5 2.9 -3.30.6 -0.8 1.7 -1.8 2.5 -2.40.7 -0.6 2.5 -2.1 3.9 -3.43.6 -3.5 4.9 -4.2 10.7 -5.50.6 -0.2 1.7 -0.5 2.4 -0.81.1 -0.5 6.0 -1.4 8.9 -1.70.6 -0.0 1.5 -0.2 2.0 -0.40.5 -0.2 1.3 -0.3 1.8 -0.21.0 0.2 5.5 0.3 8.2 0.10.8 -0.0 1.9 -0.0 2.3 0.01.2 0.1 1.2 -0.8 -0.2 -1.8 l-1.2 -0.91.1 -0.2 c1.6 -0.3 0.7 -0.9 -1.6 -1.1-1.1 -0.1 -2.5 -0.2 -3.2 -0.3-0.7 -0.1 -1.3 -0.0 -1.3 0.1zM161.3 142.6 c0.0 0.4 0.2 0.7 0.4 0.80.5 0.2 -0.4 2.3 -1.1 2.3-0.3 -0.0 -0.2 0.2 0.3 0.71.4 1.1 3.6 2.1 4.0 1.90.4 -0.2 1.9 0.6 1.9 1.10.0 0.1 0.1 0.2 0.3 0.10.1 -0.0 0.2 0.1 0.2 0.20.0 0.2 0.3 0.4 0.7 0.50.8 0.2 0.9 -0.1 0.3 -0.7-0.3 -0.3 -0.3 -0.5 0.1 -0.70.8 -0.5 1.0 -0.4 1.3 0.50.5 1.2 0.4 1.5 -0.2 1.5-0.7 -0.0 -0.6 0.6 0.2 1.00.5 0.3 0.7 0.3 0.9 -0.10.3 -0.8 2.1 -0.0 2.1 1.00.0 0.5 0.2 0.9 0.5 1.00.3 0.1 0.5 0.6 0.5 1.00.0 1.1 0.6 1.3 1.4 0.60.8 -0.7 1.7 -0.3 1.9 0.70.2 1.1 0.5 1.2 1.8 0.5 l1.2 -0.6-1.1 -1.1 c-0.7 -0.7 -1.3 -1.0 -1.6 -0.9-0.3 0.1 -0.5 0.1 -0.5 -0.10.0 -0.2 -0.4 -0.3 -0.8 -0.3-0.7 -0.0 -0.8 -0.1 -0.6 -0.90.2 -0.8 0.0 -1.1 -0.9 -1.8-0.6 -0.4 -1.4 -1.2 -1.9 -1.6-2.9 -3.1 -5.2 -4.3 -6.7 -3.8-0.5 0.2 -0.5 0.1 -0.4 -0.40.3 -0.7 -0.3 -1.5 -1.1 -1.5-0.3 -0.0 -0.8 -0.3 -1.0 -0.7-0.5 -0.9 -2.0 -0.9 -2.0 -0.0zM179.8 158.8 c-0.3 0.7 0.6 2.1 1.4 2.30.3 0.1 0.8 0.5 0.9 0.90.2 0.4 1.0 1.1 1.7 1.5 l1.4 0.8-0.2 -1.3 c-0.3 -1.9 -0.7 -2.4 -1.8 -2.4-0.8 -0.0 -1.0 -0.2 -1.2 -1.0-0.1 -0.6 -0.4 -1.0 -0.7 -1.0-0.2 -0.0 -0.7 -0.1 -1.0 -0.2-0.3 -0.1 -0.6 -0.0 -0.7 0.3zM150.0 162.7 c-0.6 1.0 -0.6 1.1 -0.1 2.0 l0.6 0.90.6 -1.3 c1.2 -2.5 0.3 -3.9 -1.1 -1.7zM186.3 165.6 c0.2 0.1 0.4 0.4 0.4 0.60.0 0.3 0.3 0.6 0.6 0.80.4 0.2 0.9 0.7 1.1 1.20.8 1.6 3.3 3.0 3.3 1.80.0 -0.6 -2.4 -3.1 -3.0 -3.1-0.2 -0.0 -0.6 -0.4 -0.8 -0.8-0.3 -0.6 -0.6 -0.8 -1.2 -0.8-0.5 -0.0 -0.7 0.1 -0.4 0.2zM193.5 170.6 c-0.1 0.3 -0.3 0.9 -0.5 1.2-0.4 0.7 -0.0 1.6 0.6 1.60.3 -0.0 0.4 -0.2 0.3 -0.6-0.1 -0.4 0.0 -0.8 0.4 -1.00.6 -0.3 0.5 -1.4 -0.1 -1.7-0.2 -0.1 -0.5 0.2 -0.6 0.5zM195.7 174.7 c0.0 0.4 1.7 2.0 2.1 2.00.4 -0.0 0.3 1.9 -0.1 2.1-0.4 0.1 2.0 3.9 2.5 3.90.4 -0.0 0.2 -1.6 -0.3 -2.1-0.5 -0.5 -0.3 -1.4 0.2 -1.00.2 0.1 0.3 -0.1 0.3 -0.50.0 -0.7 -2.6 -3.7 -3.3 -3.7-0.2 -0.0 -0.5 -0.2 -0.6 -0.5-0.2 -0.6 -0.8 -0.7 -0.8 -0.2zM123.0 178.3 c-0.4 0.3 -0.3 0.3 0.3 0.30.4 -0.1 0.9 0.1 1.0 0.40.1 0.3 0.0 0.4 -0.4 0.3-0.4 -0.1 -0.4 -0.0 -0.3 0.50.1 0.5 0.0 0.6 -0.7 0.6-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.6 -1.0 0.4 -1.2 -0.3-0.4 -1.0 -1.4 -1.5 -1.4 -0.60.0 0.7 1.0 1.5 2.2 1.70.6 0.1 1.4 0.5 1.8 0.81.1 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.7 -1.1-0.3 -0.0 -0.9 -0.4 -1.2 -0.8-0.4 -0.4 -0.9 -0.8 -1.3 -0.9-0.3 -0.1 -0.6 -0.2 -0.5 -0.30.6 -0.7 -1.1 -1.5 -2.0 -1.0zM130.7 180.1 c-0.2 0.2 -0.4 0.9 -0.4 1.40.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.4 0.1 0.7 -1.7 0.3 -1.9-0.2 -0.1 -0.5 -0.0 -0.7 0.2z","role":"hi"},{"d":"M196.5 120.7 c0.8 0.8 1.4 1.0 3.4 1.31.3 0.2 2.7 0.5 3.2 0.60.4 0.2 1.8 0.5 3.1 0.72.7 0.4 5.2 1.3 5.2 2.00.0 0.2 0.3 0.5 0.6 0.61.6 0.6 3.4 2.0 3.4 2.70.0 0.5 0.2 0.8 0.5 0.80.3 -0.0 0.5 -0.2 0.5 -0.50.0 -1.4 2.5 -0.4 3.8 1.51.4 2.1 0.8 3.9 -1.7 5.2-1.3 0.7 -1.9 1.2 -2.0 1.8-0.1 0.5 -0.3 1.2 -0.5 1.6-0.4 1.0 -0.4 2.4 0.0 2.40.5 -0.0 0.4 1.5 -0.2 2.2-0.3 0.4 -0.5 1.4 -0.5 2.30.0 0.9 -0.2 2.4 -0.5 3.3-0.3 0.9 -0.4 2.1 -0.3 2.70.2 0.9 0.1 1.1 -0.4 1.1-0.5 -0.0 -0.5 0.1 -0.2 0.50.2 0.3 0.6 0.5 0.9 0.50.5 -0.0 0.6 0.9 0.1 1.1-0.5 0.1 0.3 7.8 0.8 8.90.2 0.5 0.5 1.5 0.7 2.20.1 0.8 0.7 2.2 1.2 3.30.5 1.0 1.0 2.1 1.0 2.30.0 0.2 0.5 0.9 1.0 1.40.6 0.6 1.0 1.1 1.0 1.30.0 0.2 0.5 0.8 1.0 1.40.6 0.6 1.1 1.6 1.2 2.20.1 0.6 0.6 1.5 1.1 2.12.8 3.4 5.3 6.9 5.5 7.90.1 0.5 0.4 0.9 0.7 0.90.3 -0.0 0.6 0.3 0.8 0.61.5 2.8 -2.2 4.9 -5.0 2.9-0.6 -0.5 -1.6 -1.1 -2.1 -1.4-0.6 -0.4 -0.9 -0.8 -0.8 -1.20.2 -0.8 -0.8 -2.3 -2.4 -3.8-0.9 -0.8 -1.4 -1.5 -1.4 -2.00.0 -0.4 -0.2 -0.9 -0.5 -1.1-0.3 -0.2 -0.6 -0.9 -0.7 -1.4-0.1 -0.5 -0.4 -1.0 -0.5 -1.1-0.6 -0.4 -0.3 -1.5 0.4 -1.50.4 -0.0 0.7 -0.2 0.7 -0.30.0 -0.2 -0.2 -0.3 -0.5 -0.3-0.3 -0.0 -0.5 -0.3 -0.5 -0.70.0 -0.4 -0.2 -0.8 -0.5 -0.9-0.3 -0.1 -0.5 -0.5 -0.5 -0.80.0 -0.3 -0.1 -0.6 -0.3 -0.6-0.2 -0.0 -0.5 -0.5 -0.8 -1.1-0.2 -0.6 -0.8 -1.5 -1.2 -1.9-0.4 -0.5 -0.9 -1.1 -1.1 -1.4-2.4 -3.5 -3.2 -5.3 -3.2 -7.00.0 -0.7 -0.7 -2.4 -1.6 -4.3-0.9 -1.7 -1.7 -3.6 -1.8 -4.2-0.1 -0.6 -0.8 -1.6 -1.4 -2.3-0.6 -0.7 -1.1 -1.4 -1.1 -1.60.0 -0.4 -2.2 -3.7 -3.1 -4.7-0.3 -0.4 -0.8 -1.0 -1.1 -1.5-0.3 -0.5 -1.1 -1.1 -1.8 -1.4-1.0 -0.4 -1.3 -0.7 -1.3 -1.40.0 -0.6 -0.3 -1.0 -1.2 -1.4-1.5 -0.7 -3.8 -2.9 -3.8 -3.60.0 -0.3 -0.5 -0.8 -1.2 -1.2-0.6 -0.4 -1.2 -0.8 -1.2 -1.00.0 -0.2 -0.3 -0.4 -0.6 -0.5-0.3 -0.1 -1.1 -0.8 -1.8 -1.5-1.2 -1.3 -8.4 -5.3 -9.6 -5.3-0.3 -0.0 -1.1 -0.3 -1.8 -0.7-2.1 -1.1 -2.6 -1.3 -4.9 -1.7-1.2 -0.2 -3.2 -0.7 -4.4 -1.0-3.7 -1.1 -15.5 -1.1 -16.9 0.1-0.6 0.5 -4.0 1.6 -5.0 1.6-0.4 -0.0 -1.1 0.4 -1.6 0.8-1.1 1.0 -2.0 1.3 -2.8 0.9-0.7 -0.4 -3.0 0.0 -3.4 0.6-0.1 0.2 -1.0 0.6 -2.0 0.9-1.0 0.3 -1.9 0.6 -2.1 0.8-0.2 0.1 -0.8 0.3 -1.3 0.5-0.5 0.1 -1.6 0.8 -2.3 1.4-0.8 0.6 -1.5 1.1 -1.8 1.1-0.2 -0.0 -0.5 0.3 -0.6 0.6-0.2 0.7 -1.7 1.7 -3.9 2.6-0.8 0.4 -1.7 0.8 -1.9 1.1-0.2 0.2 -0.6 0.4 -0.8 0.4-0.5 -0.0 -4.8 4.1 -6.8 6.4-0.7 0.9 -1.5 1.6 -1.7 1.6-0.7 -0.0 -2.1 1.4 -2.1 2.10.0 0.9 -0.6 1.9 -1.2 1.9-0.3 -0.0 -0.5 0.3 -0.5 0.60.0 0.4 -0.3 0.7 -0.6 0.9-0.3 0.1 -1.0 0.9 -1.4 1.7-0.4 0.8 -1.0 1.6 -1.3 1.7-0.3 0.1 -0.7 0.4 -0.8 0.8-1.0 1.7 -1.7 2.7 -2.1 2.7-0.3 -0.0 -0.4 0.4 -0.4 1.00.0 1.2 -0.5 2.3 -1.1 2.3-0.2 -0.0 -0.7 0.6 -1.0 1.2-1.1 2.8 -1.6 4.2 -1.4 4.50.1 0.2 -0.1 0.9 -0.4 1.6-0.9 1.9 -1.1 10.2 -0.3 11.10.4 0.5 0.5 1.0 0.4 1.8-0.1 0.8 0.0 1.7 0.5 2.70.5 1.1 0.6 1.9 0.5 2.8-0.2 1.1 -0.1 1.3 0.4 1.50.5 0.1 0.7 0.6 0.9 2.10.2 1.4 0.4 2.1 1.0 2.60.4 0.4 0.8 1.0 0.8 1.30.0 0.3 0.1 0.6 0.3 0.60.1 0.0 0.7 1.0 1.2 2.00.5 1.1 1.3 2.2 1.6 2.40.3 0.2 0.8 1.1 0.9 1.90.7 3.2 3.3 5.8 4.5 4.50.6 -0.6 0.2 -1.5 -2.1 -4.4-2.0 -2.6 -5.6 -9.9 -6.4 -13.3-0.7 -2.8 -1.5 -8.7 -1.3 -9.10.1 -0.1 0.3 -2.1 0.5 -4.40.4 -4.5 1.2 -7.7 2.3 -9.60.4 -0.7 0.7 -1.4 0.7 -1.70.0 -0.3 0.1 -0.7 0.3 -1.10.2 -0.3 0.6 -1.3 1.0 -2.31.3 -3.6 6.6 -10.8 7.9 -10.80.1 -0.0 0.4 -0.3 0.8 -0.60.3 -0.3 1.7 -1.5 3.0 -2.72.3 -2.1 3.5 -2.9 7.8 -5.81.2 -0.8 2.3 -1.6 2.4 -1.80.1 -0.2 1.2 -0.8 2.3 -1.31.1 -0.6 2.7 -1.5 3.4 -2.02.2 -1.6 6.1 -3.4 8.3 -3.71.1 -0.2 3.5 -0.9 5.2 -1.61.7 -0.7 3.5 -1.4 4.0 -1.50.5 -0.1 1.7 -0.5 2.8 -0.93.0 -1.1 5.5 -1.4 7.7 -1.01.1 0.2 2.9 0.5 4.0 0.71.2 0.2 2.9 0.8 4.2 1.41.2 0.6 3.0 1.3 4.1 1.52.5 0.4 3.9 0.9 5.7 2.10.8 0.5 2.4 1.5 3.7 2.11.3 0.7 2.8 1.6 3.4 2.10.6 0.4 1.2 0.8 1.4 0.80.2 -0.0 0.4 0.2 0.4 0.50.0 0.3 0.1 0.5 0.3 0.50.6 -0.0 8.2 8.1 9.9 10.50.5 0.8 1.7 2.4 2.6 3.61.0 1.3 2.3 3.7 3.3 6.00.9 2.2 2.0 4.5 2.4 5.20.5 0.7 0.8 1.4 0.8 1.50.0 0.1 0.2 0.5 0.4 0.70.3 0.3 0.7 1.1 0.9 1.70.4 1.1 0.8 1.6 2.3 4.00.4 0.5 1.3 2.3 2.2 4.10.9 1.7 1.8 3.6 2.1 4.12.7 5.1 3.6 6.7 4.7 8.20.8 1.0 1.6 2.2 1.9 2.80.3 0.6 1.0 1.5 1.6 2.00.6 0.5 1.6 1.5 2.2 2.30.6 0.9 1.5 1.7 2.3 2.00.7 0.3 1.4 0.8 1.6 1.20.6 1.1 1.7 0.7 2.5 -0.82.0 -3.9 0.7 -11.0 -2.8 -15.9-0.9 -1.2 -2.3 -3.2 -3.2 -4.5-5.5 -7.8 -7.3 -13.4 -7.6 -24.6-0.1 -3.5 1.3 -6.1 3.0 -5.71.3 0.3 2.1 -0.1 2.1 -1.10.0 -0.7 0.1 -0.8 0.8 -0.70.9 0.2 1.2 -0.5 0.4 -1.0-0.3 -0.1 -0.5 -0.6 -0.6 -1.1-0.1 -1.1 -0.9 -1.3 -0.9 -0.30.0 1.1 -1.9 2.0 -2.6 1.1-1.0 -1.3 -0.1 -2.7 1.4 -2.40.6 0.2 0.8 0.1 0.8 -0.30.0 -0.3 0.3 -0.7 0.6 -0.80.3 -0.2 0.8 -0.8 1.0 -1.40.2 -0.6 0.6 -1.4 0.7 -1.70.2 -0.3 0.3 -0.8 0.3 -1.10.0 -1.1 5.0 -4.5 6.6 -4.50.2 -0.0 0.8 -0.3 1.3 -0.71.0 -0.8 2.0 -0.9 2.4 -0.10.3 0.5 0.6 0.5 2.3 0.33.7 -0.5 5.3 -0.4 6.7 0.60.7 0.5 1.4 0.9 1.6 0.91.3 -0.0 5.2 2.6 5.2 3.50.0 0.2 0.4 0.7 0.8 1.10.8 0.6 3.8 6.1 3.8 6.90.0 0.2 0.2 0.8 0.5 1.40.3 0.5 0.7 1.3 0.8 1.80.2 0.5 0.5 1.1 0.7 1.31.3 2.0 2.0 4.9 2.0 8.80.0 2.2 0.1 4.4 0.1 4.80.1 2.7 -0.1 3.7 -1.0 4.6-1.0 1.0 -1.0 0.9 -1.2 6.7-0.1 2.8 -0.2 3.9 -0.6 4.1-0.3 0.2 -0.5 1.0 -0.5 2.20.0 1.1 -0.1 3.5 -0.1 5.4-0.0 1.9 0.1 4.2 0.3 5.10.3 1.2 0.3 1.8 0.0 2.3-0.6 1.2 0.8 6.8 2.8 10.90.5 0.9 0.8 2.0 0.8 2.30.0 0.6 2.4 4.4 2.9 4.80.2 0.1 0.4 1.0 0.6 1.90.1 1.0 0.4 2.3 0.7 2.90.2 0.6 0.4 1.7 0.4 2.50.0 0.7 0.4 1.9 0.9 2.71.2 2.1 1.2 6.3 -0.1 8.2-1.2 1.8 -2.8 6.7 -2.4 7.40.4 0.7 0.5 3.8 0.2 6.5-0.3 2.4 -1.6 2.6 -2.0 0.3-0.1 -0.8 -0.5 -1.7 -0.8 -2.1-1.2 -1.4 -1.5 -2.7 -0.8 -3.40.4 -0.4 0.8 -1.6 1.1 -3.60.4 -2.6 0.9 -4.1 2.7 -8.00.4 -0.8 -0.2 -3.3 -1.0 -4.2-0.3 -0.3 -0.6 -1.0 -0.8 -1.7-0.4 -1.4 -1.2 -3.8 -2.2 -6.6-0.4 -1.2 -0.8 -2.4 -0.8 -2.70.0 -0.3 -0.4 -1.1 -1.0 -1.7-0.5 -0.6 -1.0 -1.3 -1.0 -1.50.0 -0.2 -0.4 -1.1 -0.8 -2.0-2.1 -4.0 -3.7 -13.5 -3.4 -19.80.1 -2.4 0.3 -6.8 0.3 -9.60.1 -4.9 0.3 -7.0 1.2 -11.30.6 -3.2 -0.4 -11.2 -1.7 -12.5-0.2 -0.2 -0.3 -0.5 -0.3 -0.70.0 -1.5 -5.8 -7.8 -8.2 -8.8-3.5 -1.5 -4.6 -1.6 -7.1 -0.3-2.6 1.3 -5.4 3.5 -5.4 4.10.0 0.3 -0.4 0.8 -0.9 1.1-1.3 1.0 -1.8 6.3 -0.8 9.21.0 3.0 1.7 6.0 1.7 7.10.0 1.0 1.8 4.8 3.1 6.40.4 0.5 1.3 1.8 2.0 3.00.8 1.2 2.0 3.0 2.8 4.03.3 4.4 3.9 6.4 3.7 13.7-0.2 9.5 -0.7 11.3 -4.0 16.5-1.7 2.6 -2.2 3.1 -3.4 3.1-0.3 -0.0 -0.8 0.3 -1.2 0.6-0.4 0.3 -1.5 1.0 -2.5 1.6-1.0 0.5 -2.1 1.1 -2.3 1.3-2.9 2.4 -11.1 1.5 -17.3 -1.8-3.6 -1.9 -3.7 -2.0 -8.6 -6.8-4.2 -4.1 -4.9 -5.3 -3.3 -5.30.5 -0.0 0.8 0.1 0.8 0.30.0 0.7 2.8 2.9 4.5 3.71.1 0.5 2.1 1.0 2.2 1.20.1 0.2 0.3 0.2 0.5 0.10.1 -0.1 0.0 -0.4 -0.3 -0.6-1.3 -0.7 -4.5 -4.2 -6.0 -6.6-0.9 -1.4 -2.2 -3.3 -2.9 -4.3-0.7 -1.0 -1.3 -2.0 -1.3 -2.10.0 -0.4 -2.2 -3.6 -3.2 -4.7-1.8 -2.0 -3.4 -4.1 -4.3 -5.9-0.5 -1.1 -1.7 -2.7 -2.9 -3.8-1.1 -1.1 -2.7 -2.6 -3.6 -3.5-0.8 -0.9 -3.3 -2.9 -5.4 -4.5-4.4 -3.4 -6.7 -5.6 -7.4 -7.0-0.3 -0.5 -1.4 -1.6 -2.5 -2.4-1.1 -0.8 -2.5 -1.8 -3.1 -2.3-2.4 -2.1 -2.2 -0.6 0.4 2.21.4 1.4 3.3 4.4 3.3 5.10.0 0.3 0.2 0.5 0.3 0.51.0 -0.0 0.2 1.0 -1.0 1.2-1.7 0.3 -2.0 0.1 -2.2 -1.5-0.2 -1.4 -1.9 -3.7 -2.7 -3.7-0.2 -0.0 -0.8 -0.6 -1.3 -1.3-1.2 -1.8 -6.2 -6.8 -7.5 -7.3-0.6 -0.2 -2.6 -0.5 -4.5 -0.6-1.9 -0.1 -4.2 -0.4 -5.2 -0.7-2.9 -0.8 -6.3 -0.6 -8.7 0.4-1.1 0.5 -2.3 0.8 -2.7 0.8-1.0 -0.0 -4.5 1.8 -5.9 3.0-0.6 0.5 -1.4 1.0 -1.6 1.0-0.2 -0.0 -0.5 0.4 -0.6 0.8-0.1 0.5 -0.5 0.9 -1.0 1.0-1.2 0.3 -2.7 2.5 -1.7 2.50.2 -0.0 0.7 0.4 1.1 0.90.9 1.1 1.8 0.9 3.0 -0.90.5 -0.7 1.3 -1.6 1.8 -1.80.5 -0.3 1.1 -0.7 1.2 -1.00.2 -0.4 0.9 -0.6 2.1 -0.71.8 -0.1 2.7 -0.5 3.8 -1.50.9 -0.8 3.8 -1.3 9.4 -1.5 l4.7 -0.20.9 0.9 c0.5 0.5 0.8 1.1 0.7 1.4-0.1 0.3 -0.0 0.5 0.1 0.50.2 -0.0 0.3 -0.3 0.3 -0.70.0 -0.5 0.2 -0.7 1.0 -0.70.6 -0.0 1.0 0.2 1.0 0.40.0 0.2 0.4 0.7 1.0 1.10.6 0.4 1.2 1.3 1.5 2.20.5 1.5 1.9 2.6 3.1 2.60.2 -0.0 0.5 0.6 0.8 1.30.3 0.9 0.7 1.4 1.5 1.80.6 0.3 1.5 1.0 2.0 1.60.5 0.6 1.3 1.2 1.8 1.41.7 0.7 1.9 1.7 1.2 5.5-0.3 1.9 -0.8 4.5 -1.0 5.8-0.2 1.3 -0.6 2.6 -0.8 2.9-0.6 0.7 -0.9 2.5 -0.4 2.80.2 0.1 0.5 -0.1 0.6 -0.50.1 -0.4 1.1 -2.5 2.2 -4.51.6 -3.1 1.9 -4.0 2.0 -5.50.0 -1.0 0.1 -2.5 0.3 -3.40.2 -0.9 0.4 -2.3 0.5 -3.10.3 -2.1 2.6 -1.4 4.0 1.30.3 0.5 1.1 1.4 1.9 1.90.8 0.5 1.5 1.0 1.5 1.20.2 0.4 0.8 1.0 2.0 1.90.8 0.6 1.1 1.1 1.1 1.90.0 0.9 0.1 1.1 0.5 0.90.3 -0.1 0.8 -0.0 1.2 0.12.9 1.3 3.4 1.7 4.0 2.80.6 1.2 1.0 1.8 3.1 4.20.8 0.9 1.1 1.6 1.1 2.2-0.1 1.2 -1.3 1.1 -2.2 -0.3-1.8 -2.8 -3.5 -3.5 -3.7 -1.5-0.2 1.9 0.1 2.5 1.7 2.72.0 0.3 2.7 1.1 2.4 2.9-0.2 1.9 -1.0 4.6 -1.3 4.6-0.1 -0.0 -0.6 0.8 -1.1 1.7-1.3 2.5 -1.7 3.3 -2.1 3.6-0.5 0.3 -3.1 4.8 -3.1 5.30.0 0.2 -0.2 0.4 -0.4 0.4-0.2 -0.0 -1.0 0.9 -1.6 2.0-0.6 1.1 -1.6 2.5 -2.1 3.1-0.5 0.6 -0.9 1.2 -0.9 1.50.0 0.3 -0.5 1.0 -1.2 1.6-0.6 0.6 -1.2 1.3 -1.2 1.60.0 0.2 -0.6 1.0 -1.3 1.8-0.7 0.8 -1.3 1.6 -1.3 1.90.0 0.3 -0.3 0.8 -0.7 1.1-0.6 0.6 -0.6 1.7 -0.0 1.70.5 -0.0 4.6 -5.0 6.0 -7.33.3 -5.6 4.1 -6.9 5.3 -8.00.6 -0.6 1.8 -2.5 2.6 -4.21.6 -3.3 3.5 -6.4 4.3 -7.10.9 -0.7 1.9 -3.2 1.9 -4.80.0 -1.0 0.2 -1.6 0.5 -1.70.6 -0.3 3.8 2.4 4.1 3.50.2 0.9 1.0 1.8 2.0 2.30.6 0.3 2.8 4.0 2.8 4.70.0 0.2 -0.4 0.4 -0.8 0.5-0.6 0.2 -0.8 0.4 -0.8 1.00.0 0.4 -0.3 1.1 -0.6 1.6-0.9 1.1 -1.4 2.3 -1.1 2.30.2 -0.0 0.4 -0.3 0.6 -0.70.6 -1.3 2.8 -0.8 2.8 0.60.0 0.2 0.2 0.5 0.5 0.60.3 0.1 0.6 0.5 0.7 0.90.1 0.4 0.4 1.0 0.7 1.30.3 0.3 0.7 0.9 0.9 1.30.2 0.4 0.7 0.8 1.2 0.90.5 0.1 1.0 0.6 1.3 1.30.3 0.6 1.0 1.3 1.7 1.60.6 0.3 1.5 0.9 1.8 1.31.2 1.3 3.1 2.5 5.6 3.71.3 0.6 3.3 1.6 4.3 2.22.5 1.3 5.4 1.9 6.4 1.30.4 -0.3 2.1 -0.5 4.5 -0.54.1 -0.1 4.7 -0.4 4.7 -1.90.0 -0.3 0.2 -0.6 0.3 -0.60.2 -0.0 0.5 -0.2 0.8 -0.30.2 -0.2 1.0 -0.8 1.8 -1.30.8 -0.6 1.4 -1.3 1.4 -1.60.0 -0.4 0.2 -0.9 0.4 -1.10.2 -0.2 0.8 -1.0 1.2 -1.61.4 -2.0 3.8 -4.7 4.4 -4.81.5 -0.5 2.1 -8.4 0.9 -13.0-0.2 -0.7 -0.4 -2.2 -0.4 -3.2-0.3 -5.3 -1.6 -10.9 -2.9 -12.2-0.4 -0.4 -1.0 -1.2 -1.4 -1.8-0.4 -0.6 -1.0 -1.5 -1.3 -2.0-0.3 -0.5 -0.8 -1.3 -1.0 -1.8-0.2 -0.5 -0.5 -0.9 -0.6 -0.9-0.1 -0.0 -0.4 -0.4 -0.5 -0.8-0.1 -0.4 -0.6 -1.3 -1.2 -1.9-0.5 -0.6 -1.2 -1.9 -1.5 -2.9-0.4 -1.0 -0.8 -2.1 -1.0 -2.3-3.8 -5.1 -3.7 -16.5 0.2 -18.60.5 -0.3 1.3 -0.8 1.7 -1.11.0 -0.9 3.0 -1.2 6.4 -0.91.2 0.1 4.2 2.1 5.2 3.40.5 0.7 1.3 1.6 1.8 2.00.5 0.4 0.9 1.0 1.0 1.40.1 0.4 0.4 1.2 0.7 1.71.4 3.1 1.6 4.4 1.6 8.00.0 3.4 -0.6 8.8 -1.0 9.1-0.6 0.4 -1.0 3.5 -0.9 6.70.3 4.2 -0.5 13.7 -1.1 13.9-0.6 0.2 -0.6 1.4 0.2 3.50.3 1.0 0.8 2.4 1.0 3.30.2 0.8 0.5 1.9 0.7 2.50.2 0.5 0.5 1.6 0.7 2.40.2 0.8 0.5 1.4 0.6 1.40.3 -0.0 1.1 2.1 1.7 4.40.3 0.9 0.9 2.2 1.7 3.11.1 1.3 1.4 1.8 1.6 3.70.2 1.2 0.4 2.3 0.6 2.60.2 0.2 0.4 1.0 0.5 1.60.1 0.7 0.5 1.4 0.8 1.60.6 0.3 1.6 5.1 1.2 5.8-0.1 0.2 -0.2 0.8 -0.1 1.30.0 0.5 -0.1 1.1 -0.2 1.3-0.9 1.2 -2.3 11.5 -1.7 12.90.7 1.8 -1.0 3.2 -3.6 2.8-1.0 -0.2 -1.1 -0.1 -0.4 0.10.5 0.2 4.8 0.3 9.6 0.3 l8.7 0.0-0.0 -8.8 c0.0 -6.4 -0.1 -8.5 -0.4 -7.9-1.2 2.8 -1.4 3.4 -1.3 3.90.0 0.3 0.1 2.1 0.1 3.90.0 1.9 0.2 4.2 0.5 5.30.8 3.0 0.9 2.9 -2.8 2.9-1.8 -0.0 -3.4 -0.1 -3.5 -0.3-0.2 -0.3 0.6 -4.6 1.2 -6.50.2 -0.7 0.2 -1.7 -0.2 -3.1-0.6 -2.3 -0.4 -3.1 0.7 -4.42.0 -2.4 2.6 -6.5 1.5 -11.3-0.2 -0.8 -0.5 -2.4 -0.7 -3.4-0.7 -3.8 -2.7 -8.8 -6.9 -17.4-0.6 -1.3 -1.1 -2.5 -1.1 -2.80.0 -0.2 -0.3 -1.0 -0.6 -1.7-1.4 -2.8 -1.5 -4.5 -0.6 -6.51.6 -3.4 2.7 -14.0 2.3 -22.7-0.0 -0.9 0.1 -1.8 0.2 -2.01.9 -2.5 1.1 -8.8 -1.9 -14.9-0.9 -1.8 -1.7 -3.5 -1.7 -3.70.0 -0.2 -0.2 -0.9 -0.5 -1.6-1.1 -2.5 -1.7 -3.9 -1.9 -4.5-0.2 -0.4 -0.6 -0.6 -1.4 -0.6-1.4 -0.0 -1.6 -0.2 -1.8 -2.9-0.2 -2.2 -1.4 -3.8 -2.8 -3.8-0.3 -0.0 -0.7 -0.2 -0.9 -0.5-0.2 -0.3 -0.9 -0.6 -1.6 -0.7-0.7 -0.1 -1.5 -0.4 -1.7 -0.7-0.3 -0.2 -0.9 -0.5 -1.4 -0.6-1.0 -0.2 -1.4 -0.9 -1.1 -1.60.8 -1.7 1.5 -2.4 2.3 -2.50.7 -0.1 0.9 -0.3 0.9 -0.90.0 -0.7 0.1 -0.8 0.9 -0.81.0 0.1 1.7 -0.1 3.2 -0.90.5 -0.3 1.2 -0.4 1.5 -0.30.6 0.2 1.1 -0.3 1.1 -1.10.0 -0.3 -0.8 -0.4 -2.7 -0.4 l-2.7 -0.0-1.6 1.4 c-4.9 4.4 -5.6 4.9 -7.4 4.9-0.5 -0.0 -0.9 -0.2 -0.9 -0.40.0 -0.6 2.3 -4.6 2.8 -4.81.3 -0.5 0.5 -1.2 -1.3 -1.2 l-1.7 -0.0-0.6 1.6 c-1.3 3.6 -2.3 4.3 -4.4 3.4-1.5 -0.7 -2.4 -0.0 -2.2 1.7 l0.1 1.4-1.7 -0.0 c-0.9 -0.0 -1.8 0.1 -1.9 0.3-0.2 0.4 -2.2 0.5 -2.2 0.10.0 -0.1 -1.0 -0.5 -2.2 -0.9-3.4 -1.0 -5.4 -2.6 -5.4 -4.40.0 -0.3 -0.3 -0.6 -0.6 -0.8-0.4 -0.2 -0.7 -0.8 -0.9 -1.3 l-0.2 -1.0-9.7 -0.0-9.7 -0.01.0 1.0zM272.9 122.6 c0.2 3.1 0.4 3.3 0.4 0.50.0 -1.3 -0.1 -2.5 -0.3 -2.6-0.2 -0.1 -0.2 0.8 -0.1 2.1zM272.8 133.4 c-0.4 2.9 -0.2 18.9 0.2 21.80.2 1.4 0.3 -2.1 0.3 -11.10.0 -7.2 -0.0 -13.1 -0.1 -13.1-0.1 -0.0 -0.3 1.1 -0.4 2.4zM120.9 163.4 c-0.5 0.7 -1.2 1.3 -1.4 1.3-0.3 -0.0 -0.5 0.1 -0.5 0.30.0 0.1 -0.8 1.1 -1.8 2.2-1.0 1.0 -1.8 2.0 -1.8 2.20.0 0.2 -0.3 0.6 -0.6 0.9-0.7 0.7 -1.5 3.1 -1.8 5.4-0.1 0.8 -0.4 1.8 -0.7 2.3-0.3 0.5 -0.6 1.7 -0.7 2.6-0.1 1.0 -0.3 1.7 -0.5 1.7-0.6 -0.0 -0.6 1.1 0.0 2.30.4 0.7 0.7 1.7 0.7 2.20.0 0.5 0.1 0.8 0.3 0.80.2 -0.0 0.4 0.3 0.5 0.70.1 0.4 0.7 1.4 1.2 2.20.6 0.8 0.9 1.8 0.8 2.1-0.1 0.3 0.2 1.2 0.6 1.90.5 0.7 1.0 1.8 1.1 2.50.4 1.5 3.1 4.5 4.8 5.40.7 0.4 1.5 0.9 1.6 1.11.0 1.7 7.4 -0.8 11.2 -4.30.8 -0.8 -4.6 -2.9 -7.8 -3.0-1.6 -0.1 -3.1 -0.4 -3.7 -0.7-2.1 -1.1 -3.0 -0.6 -1.6 0.71.6 1.5 3.8 2.2 8.5 2.61.0 0.1 1.6 0.3 1.6 0.60.0 0.8 -5.6 1.0 -7.2 0.3-3.5 -1.5 -4.2 -2.0 -4.5 -2.9-0.2 -0.5 -0.5 -1.0 -0.8 -1.1-0.3 -0.1 -0.5 -0.5 -0.5 -0.90.0 -0.4 -0.5 -1.1 -1.0 -1.6-0.8 -0.7 -1.0 -1.1 -0.9 -1.90.0 -0.6 -0.1 -1.3 -0.3 -1.6-0.5 -0.6 -1.2 -4.8 -1.0 -6.30.1 -1.2 1.7 -1.6 2.3 -0.60.2 0.3 0.4 0.3 1.1 -0.00.7 -0.4 1.1 -0.4 2.0 -0.00.7 0.2 1.4 0.3 1.6 0.20.3 -0.1 -0.3 -0.5 -2.0 -1.0-3.0 -1.1 -4.4 -9.9 -1.5 -9.90.6 -0.0 2.7 1.8 2.7 2.30.0 1.5 3.8 2.2 7.1 1.34.1 -1.1 6.7 -0.9 12.7 0.90.4 0.1 0.6 -0.0 0.6 -0.30.0 -0.5 4.1 -2.6 4.9 -2.60.2 -0.0 0.7 -0.4 1.2 -0.90.5 -0.6 1.0 -0.8 1.3 -0.70.3 0.1 0.5 0.0 0.5 -0.30.0 -0.6 -0.9 -0.6 -2.8 -0.1-2.7 0.7 -3.6 1.0 -4.3 1.7-0.4 0.4 -1.0 0.6 -1.5 0.5-0.5 -0.1 -0.9 0.0 -1.0 0.3-0.3 0.8 -3.7 0.1 -5.0 -0.9-0.3 -0.2 -2.6 -0.4 -5.9 -0.4-6.6 -0.1 -8.6 -0.7 -8.8 -2.9-0.1 -1.4 1.4 -4.0 2.3 -4.01.7 -0.0 2.0 -2.1 0.4 -2.8-0.4 -0.2 -0.7 -0.6 -0.8 -0.9-0.1 -0.4 -0.4 -0.2 -1.0 0.7zM148.9 174.3 c-0.4 1.7 -0.3 3.9 0.2 3.30.5 -0.5 0.7 -1.9 0.4 -2.7-0.1 -0.3 -0.2 -0.7 -0.3 -1.0-0.1 -0.3 -0.2 -0.2 -0.4 0.3zM147.5 179.1 c-0.5 0.5 -1.4 0.9 -1.8 0.9 l-0.9 0.00.8 0.6 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.03.1 -1.6 7.0 -0.9 8.6 1.70.8 1.2 1.0 1.4 1.3 1.00.6 -0.7 -1.2 -3.8 -2.9 -5.0-1.5 -1.0 -1.9 -2.3 -0.9 -2.80.3 -0.2 0.5 -0.6 0.4 -1.1 l-0.1 -0.8-1.0 0.9zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM181.5 181.8 c-0.3 0.2 -0.9 1.5 -1.4 2.8-0.4 1.3 -1.2 2.9 -1.7 3.6-0.9 1.4 -1.1 2.2 -0.4 2.20.8 -0.0 1.4 -0.9 2.0 -2.61.0 -2.9 1.2 -3.1 2.1 -3.51.1 -0.4 2.0 -2.2 1.3 -2.7-0.6 -0.4 -1.4 -0.3 -2.1 0.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.4 0.9 0.7 1.8 0.82.9 0.4 5.3 3.8 2.6 3.6-0.6 -0.0 -3.5 -0.2 -6.3 -0.4-6.3 -0.4 -6.7 -0.3 -6.6 1.1 l0.1 1.04.8 -0.0 c3.6 -0.0 5.3 -0.2 6.8 -0.61.2 -0.4 2.3 -0.5 2.9 -0.41.1 0.2 1.5 -1.0 0.5 -1.4-0.3 -0.1 -0.5 -0.4 -0.5 -0.60.0 -0.7 -2.1 -2.7 -3.3 -3.4-0.6 -0.3 -1.5 -0.8 -1.8 -1.1-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7zM177.0 192.1 c0.0 0.3 -0.3 0.8 -0.6 1.0-0.9 0.7 -1.6 2.3 -0.8 2.20.8 -0.1 1.8 -2.1 1.7 -3.0-0.1 -0.5 -0.2 -0.5 -0.2 -0.2zM195.9 206.0 c-1.0 0.6 -4.6 6.3 -4.6 7.30.0 0.2 -0.4 0.7 -0.8 1.1-0.6 0.6 -0.8 1.2 -0.8 2.20.0 0.7 -0.1 1.3 -0.3 1.3-0.2 -0.0 -0.3 0.2 -0.3 0.50.0 0.3 -0.3 0.5 -0.6 0.5-0.6 -0.0 -3.4 2.5 -3.4 3.00.0 0.1 -0.5 0.9 -1.2 1.6-0.7 0.9 -1.2 1.8 -1.2 2.30.0 0.7 -0.2 1.0 -0.9 1.1-1.3 0.3 -3.8 2.9 -3.8 3.90.0 0.2 -0.2 0.4 -0.4 0.4-0.3 0.1 -0.3 0.4 -0.1 1.0 l0.4 0.80.5 -1.1 c0.3 -0.6 1.0 -1.4 1.6 -1.80.5 -0.4 1.0 -0.9 1.0 -1.20.0 -0.3 0.5 -0.7 1.1 -1.01.2 -0.6 3.9 -3.3 3.9 -3.90.0 -0.2 0.3 -0.8 0.8 -1.30.4 -0.5 1.1 -1.4 1.6 -2.00.5 -0.6 1.4 -1.7 2.1 -2.40.7 -0.7 1.2 -1.5 1.2 -1.70.0 -0.2 0.3 -0.6 0.7 -0.90.5 -0.4 0.7 -0.9 0.7 -2.00.0 -0.8 0.2 -1.5 0.4 -1.60.8 -0.5 1.9 -2.7 1.9 -3.90.0 -0.7 0.1 -1.3 0.3 -1.40.3 -0.2 1.1 -1.5 0.9 -1.5-0.1 -0.0 -0.4 0.2 -0.7 0.4zM111.7 206.6 c0.0 0.1 0.4 0.7 0.9 1.2 l0.8 0.90.7 -0.8 c0.5 -0.6 0.6 -0.8 0.2 -1.0-0.8 -0.4 -2.6 -0.7 -2.6 -0.4zM210.7 207.6 c0.0 0.3 0.6 0.9 1.4 1.10.7 0.3 0.7 -0.2 0.0 -0.9-0.6 -0.6 -1.5 -0.7 -1.5 -0.2z","role":"ink"}]};

  // Source centerline along the arched viper body (neck -> arch -> tail)
  const VIPER_SPINE = [
    [116, 186], [104, 166], [116, 144], [146, 134],
    [182, 142], [208, 174], [222, 198], [238, 152], [260, 188], [266, 232]
  ];

  function mountSwarm(bubbleEl, overrides) {
    const Rig = root.BestiaryTracedRig;
    if (!bubbleEl || !Rig) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      count: 3,
      wash: '#1f5fff',
      wash2: '#0b2fa8',
      scale: narrow ? 0.72 : 0.88,
      dur: 10.5,
      autoPause: true
    }, overrides || {});

    const rect = bubbleEl.getBoundingClientRect ? bubbleEl.getBoundingClientRect() : { width: 280, height: 74 };
    const w = Math.max(120, Math.round(rect.width || 280));
    const h = Math.max(44, Math.round(rect.height || 74));
    const sc = opts.scale;
    const padX = Math.round(Math.min(26, Math.max(18, w * 0.09)));
    const padY = Math.round(Math.max(54, 62 * sc));
    const svgW = Math.ceil(w + padX * 2);
    const svgH = Math.ceil(h + padY * 2);
    const durS = `${Rig.f1(opts.dur)}s`;

    const svg = Rig.el('svg', {
      xmlns: Rig.SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-flight bestiary-rig bestiary-traced-rig',
      'data-rig': 'swarm-traced',
      'data-dur': Rig.f1(opts.dur),
      style: `position:absolute;left:${-padX}px;top:${-padY}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    svg.style.setProperty('--wash', opts.wash);
    svg.style.setProperty('--wash2', opts.wash2);

    const defs = Rig.el('defs', {}, svg);
    const bx0 = padX, by0 = padY, bx1 = padX + w, by1 = padY + h;
    const cx = padX + w * 0.5, cy = padY + h * 0.5;
    const rx = Math.max(10, padX - 6);
    const ryTop = Math.max(36, padY - 14);
    const ryBot = Math.max(32, padY - 18);

    // Swooping orbit touching bubble's top-right edge
    const pathD = [
      `M${Rig.f1(bx1 - 14)},${Rig.f1(by0)}`,
      `C${Rig.f1(bx1 + rx * 0.85)},${Rig.f1(by0 - ryTop * 0.55)} ${Rig.f1(cx + w * 0.28)},${Rig.f1(by0 - ryTop)} ${Rig.f1(cx)},${Rig.f1(by0 - ryTop * 0.88)}`,
      `C${Rig.f1(cx - w * 0.34)},${Rig.f1(by0 - ryTop * 0.80)} ${Rig.f1(bx0 - rx * 0.75)},${Rig.f1(by0 - 4)} ${Rig.f1(bx0 - rx * 0.48)},${Rig.f1(cy)}`,
      `C${Rig.f1(bx0 - rx * 0.25)},${Rig.f1(by1 + ryBot * 0.6)} ${Rig.f1(cx - w * 0.24)},${Rig.f1(by1 + ryBot)} ${Rig.f1(cx + w * 0.06)},${Rig.f1(by1 + ryBot * 0.72)}`,
      `C${Rig.f1(cx + w * 0.38)},${Rig.f1(by1 + ryBot * 0.45)} ${Rig.f1(bx1 + rx * 0.8)},${Rig.f1(cy + 12)} ${Rig.f1(bx1 + rx * 0.5)},${Rig.f1(by0 + 10)}`,
      `C${Rig.f1(bx1 + rx * 0.25)},${Rig.f1(by0 - 14)} ${Rig.f1(bx1 - 2)},${Rig.f1(by0 - 10)} ${Rig.f1(bx1 - 14)},${Rig.f1(by0)}`
    ].join(' ');
    const pathId = Rig.uid('swarm-spine');
    Rig.el('path', { id: pathId, d: pathD, fill: 'none' }, defs);

    const approxLen = 2 * w + 2 * h + 2 * (ryTop + ryBot);
    const kt = '0;.06;.22;.30;.52;.72;.88;1';
    const ks = '0 0 1 1;.22 0 .38 1;.42 0 .58 1;.25 0 .45 1;.28 0 .52 1;.30 0 .70 1;0 0 1 1';
    const headU = [0.08, 0.08, 0.30, 0.34, 0.62, 0.84, 0.99, 0.99];

    const colA = `var(--wash, ${opts.wash})`;
    const colB = `var(--wash2, ${opts.wash2})`;

    for (let i = 0; i < opts.count; i += 1) {
      const beginAttr = i === 0 ? '0s' : `${Rig.f2(-((opts.dur / opts.count) * i))}s`;
      const vScale = sc * (i === 0 ? 0.96 : (i === 1 ? 0.86 : 0.78));
      // the roundel's palette, one pair per viper: lapis, sage, claret, taupe
      const RP = [['#1f5fff', '#0b2fa8'], ['#acae8e', '#6f7d5c'], ['#865f57', '#6d3935'], ['#b39e92', '#7d6558']];
      const mainCol = RP[i % RP.length][0];
      const altCol = RP[i % RP.length][1];

      const rig = Rig.mountSegmentedBody({
        defs, parent: svg,
        bodyLayers: VIPER_ART.body,
        srcSpine: VIPER_SPINE,
        srcWidth: 32,
        numSegments: 14,
        overlap: 0.62,
        targetPathId: pathId,
        targetPathLen: approxLen,
        bodyScale: 0.22 * vScale,
        bodyLenPx: 44 * vScale,
        keyTimes: kt, keySplines: ks, headUValues: headU,
        dur: opts.dur, begin: beginAttr,
        wash: mainCol, wash2: altCol,
        className: 'swarm-viper-body'
      });

      // Traced viper head + separate animated jaw & blink
      const headNode = Rig.el('g', {});
    headNode.appendChild(root.BestiarySlots.buildTracedRoundelHead({ wash: mainCol, wash2: altCol, scale: 0.266*vScale, dur: opts.dur }));

      Rig.mountJointPart({
        parent: svg, targetPathId: pathId,
        spanFrac: rig.spanFrac, jointFrac: 0.0,
        keyTimes: kt, keySplines: ks, headUValues: headU,
        dur: opts.dur, begin: beginAttr, node: headNode,
        localRotate: { values: '0;-10;12;-12;8;-8;0', keyTimes: '0;.18;.32;.50;.68;.84;1', dur: opts.dur }
      });
    }

    if (typeof getComputedStyle === 'function' && getComputedStyle(bubbleEl).position === 'static') {
      bubbleEl.style.position = 'relative';
    }
    bubbleEl.appendChild(svg);
    Rig.observeVisibility(svg, opts.autoPause);
    return svg;
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.VIPER_ART = VIPER_ART;
  root.BestiarySlots.mountSwarm = mountSwarm;
}(typeof globalThis !== 'undefined' ? globalThis : this));

/**
 * Brief 6 — Slot: "yuck"
 * Small potrace-traced bestiary viper rigged as 14 overlapping traced segments
 * smoothly coiling and slithering round the red blackletter "yuck" bubble.
 * No bounce, no wind-up, no procedural body. Recolourable via --wash / --wash2.
 */
(function (root) {
  'use strict';

  const YUCK_ART = {"headSkull":[{"d":"M112.0 174.2 l0.0 14.22.5 -0.0 c2.2 -0.0 2.5 -0.1 2.1 -0.5-0.5 -0.6 -0.2 -3.7 0.3 -4.30.6 -0.6 3.5 -0.7 4.1 -0.10.3 0.3 0.7 0.5 0.9 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.20.3 -0.0 0.8 0.4 1.2 1.0 l0.6 1.013.4 -0.0 c10.7 -0.0 13.1 -0.1 12.4 -0.4-1.2 -0.5 -2.2 -3.0 -2.6 -6.2-0.1 -1.2 -0.4 -2.5 -0.6 -2.9-0.4 -0.8 -0.5 -2.2 -0.2 -2.60.1 -0.1 0.2 -0.8 0.1 -1.4-0.1 -0.8 0.1 -1.1 0.4 -1.10.3 -0.0 0.7 -0.5 0.9 -1.10.3 -0.9 0.6 -1.2 1.3 -1.31.1 -0.1 1.3 -1.0 0.5 -1.8-1.8 -1.8 -2.4 -4.4 -1.3 -6.00.3 -0.4 0.6 -1.3 0.7 -2.1 l0.2 -1.4-1.2 -0.0 c-1.9 -0.0 -2.2 0.5 -2.6 3.8-0.3 3.1 -0.8 4.8 -1.4 4.8-0.2 -0.0 -0.3 0.3 -0.3 0.60.0 1.4 -1.6 1.8 -3.9 0.9-1.0 -0.4 -3.7 -0.6 -6.1 -0.5-1.0 0.1 -0.2 1.5 0.9 1.72.1 0.3 3.3 1.0 3.3 1.90.0 1.5 -10.9 0.5 -12.2 -1.1-0.8 -1.0 -1.7 -1.1 -2.0 -0.2-0.8 2.5 -7.8 0.9 -7.8 -1.80.0 -1.3 1.5 -3.8 2.3 -3.80.8 -0.0 1.7 -1.1 1.7 -2.10.0 -0.7 0.2 -0.9 0.8 -0.90.9 -0.0 1.5 -0.4 2.7 -2.1 l0.9 -1.2-8.2 -0.0-8.2 -0.00.0 14.2zM157.0 160.5 c0.0 0.6 0.9 1.5 1.9 1.91.3 0.5 1.4 0.4 1.4 -1.0 l0.0 -1.3-1.7 -0.0 c-1.2 -0.0 -1.7 0.1 -1.7 0.5z","role":"wash"},{"d":"M112.1 160.8 c-0.4 1.5 0.7 1.4 1.6 -0.20.3 -0.5 0.2 -0.6 -0.5 -0.6-0.6 -0.0 -0.9 0.2 -1.1 0.8zM150.0 162.7 c-0.6 1.0 -0.6 1.1 -0.1 2.0 l0.6 0.90.6 -1.3 c1.2 -2.5 0.3 -3.9 -1.1 -1.7zM123.0 178.3 c-0.4 0.3 -0.3 0.3 0.3 0.30.4 -0.1 0.9 0.1 1.0 0.40.1 0.3 0.0 0.4 -0.4 0.3-0.4 -0.1 -0.4 -0.0 -0.3 0.50.1 0.5 0.0 0.6 -0.7 0.6-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.6 -1.0 0.4 -1.2 -0.3-0.4 -1.0 -1.4 -1.5 -1.4 -0.60.0 0.7 1.0 1.5 2.2 1.70.6 0.1 1.4 0.5 1.8 0.81.1 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.7 -1.1-0.3 -0.0 -0.9 -0.4 -1.2 -0.8-0.4 -0.4 -0.9 -0.8 -1.3 -0.9-0.3 -0.1 -0.6 -0.2 -0.5 -0.30.6 -0.7 -1.1 -1.5 -2.0 -1.0zM130.7 180.1 c-0.2 0.2 -0.4 0.9 -0.4 1.40.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.4 0.1 0.7 -1.7 0.3 -1.9-0.2 -0.1 -0.5 -0.0 -0.7 0.2z","role":"hi"},{"d":"M123.1 160.7 c-0.5 0.8 -0.5 1.0 0.0 1.00.2 -0.0 0.7 0.4 1.1 0.90.9 1.2 1.7 0.9 3.1 -1.0 l1.1 -1.5-2.4 -0.0 c-2.1 -0.0 -2.5 0.1 -2.9 0.7zM157.0 160.5 c0.0 0.6 0.9 1.5 1.9 1.91.3 0.5 1.4 0.4 1.4 -1.0 l0.0 -1.3-1.7 -0.0 c-1.2 -0.0 -1.7 0.1 -1.7 0.5zM120.9 163.4 c-0.5 0.7 -1.2 1.3 -1.4 1.3-0.3 -0.0 -0.5 0.1 -0.5 0.30.0 0.1 -0.8 1.1 -1.8 2.2-1.0 1.0 -1.8 2.0 -1.8 2.20.0 0.2 -0.3 0.6 -0.6 0.9-0.6 0.6 -1.1 2.1 -2.1 5.9-0.4 1.8 -0.8 10.2 -0.5 11.40.2 0.7 0.4 0.8 1.6 0.8 l1.5 -0.0-0.3 -1.5 c-0.6 -2.8 -0.3 -4.4 0.9 -4.30.5 0.0 1.1 0.3 1.2 0.50.2 0.3 0.4 0.3 1.1 -0.00.7 -0.4 1.1 -0.4 2.0 -0.00.7 0.2 1.4 0.3 1.6 0.20.3 -0.1 -0.3 -0.5 -2.0 -1.0-3.0 -1.1 -4.4 -9.9 -1.5 -9.90.6 -0.0 2.7 1.8 2.7 2.30.0 0.5 2.9 2.0 3.8 2.00.5 -0.0 2.0 -0.3 3.3 -0.74.1 -1.1 6.7 -0.9 12.7 0.90.4 0.1 0.6 -0.0 0.6 -0.30.0 -0.5 4.1 -2.6 4.9 -2.60.2 -0.0 0.7 -0.4 1.2 -0.90.5 -0.6 1.0 -0.8 1.3 -0.70.3 0.1 0.5 0.0 0.5 -0.30.0 -0.6 -0.9 -0.6 -2.8 -0.1-2.7 0.7 -3.6 1.0 -4.3 1.7-0.4 0.4 -1.0 0.6 -1.5 0.5-0.5 -0.1 -0.9 0.0 -1.0 0.3-0.3 0.8 -3.7 0.1 -5.0 -0.9-0.3 -0.2 -2.6 -0.4 -5.9 -0.4-6.6 -0.1 -8.6 -0.7 -8.8 -2.9-0.1 -1.4 1.4 -4.0 2.3 -4.01.7 -0.0 2.0 -2.1 0.4 -2.8-0.4 -0.2 -0.7 -0.6 -0.8 -0.9-0.1 -0.4 -0.4 -0.2 -1.0 0.7zM148.9 174.3 c-0.4 1.7 -0.3 3.9 0.2 3.30.5 -0.5 0.7 -1.9 0.4 -2.7-0.1 -0.3 -0.2 -0.7 -0.3 -1.0-0.1 -0.3 -0.2 -0.2 -0.4 0.3zM147.5 179.1 c-0.5 0.5 -1.4 0.9 -1.8 0.9 l-0.9 0.00.8 0.6 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.13.2 -1.6 7.2 -0.8 8.7 1.80.7 1.2 0.9 1.4 1.9 1.30.6 -0.0 0.9 -0.1 0.5 -0.2-0.4 -0.1 -1.0 -0.9 -1.7 -2.2-0.7 -1.5 -1.4 -2.3 -2.4 -3.0-1.5 -1.0 -1.9 -2.3 -0.9 -2.80.3 -0.2 0.5 -0.6 0.4 -1.1 l-0.1 -0.8-1.0 0.9zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.3 0.8 0.7 1.4 0.80.6 0.1 1.6 0.3 2.2 0.41.5 0.3 1.1 -0.3 -0.7 -1.2-0.8 -0.4 -1.7 -1.0 -2.1 -1.3-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7z","role":"ink"}],"headJaw":[{"d":"M114.0 193.2 l0.0 13.215.3 -0.015.3 -0.00.9 -1.1 c0.5 -0.6 1.3 -1.9 1.8 -2.90.4 -1.0 1.0 -2.2 1.3 -2.70.3 -0.5 0.5 -1.5 0.5 -2.30.0 -1.8 0.4 -2.3 2.2 -2.31.7 -0.0 2.5 -0.8 2.5 -2.50.0 -0.7 0.1 -2.0 0.2 -2.80.2 -1.4 0.2 -1.5 -0.5 -1.8-1.0 -0.4 -2.1 -3.3 -2.3 -5.9 l-0.1 -1.9-18.5 -0.0-18.5 -0.00.0 13.2z m7.1 -9.7 c0.3 0.3 0.6 0.5 0.8 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.21.5 -0.0 2.1 3.1 0.9 4.1-1.2 1.0 -6.5 0.6 -7.5 -0.6-0.4 -0.5 -0.9 -0.9 -1.0 -0.9-1.5 -0.0 -2.3 -1.9 -1.9 -4.30.3 -1.6 3.4 -2.4 4.6 -1.2zM153.8 201.9 c-0.6 0.9 -0.8 1.7 -0.8 2.8 l0.0 1.61.7 -0.01.7 -0.00.0 -2.8 c0.0 -3.4 -0.9 -4.0 -2.5 -1.6z","role":"wash"},{"d":"M119.8 180.6 c0.3 0.3 1.1 0.7 1.7 0.80.6 0.1 1.5 0.5 1.9 0.81.2 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.6 -1.1-0.3 -0.0 -0.9 -0.3 -1.2 -0.7-0.7 -0.8 -4.5 -0.7 -4.7 0.0-0.2 0.5 -1.1 0.4 -1.3 -0.2-0.1 -0.3 -0.5 -0.5 -0.9 -0.5-0.6 -0.0 -0.6 0.0 -0.1 0.6zM130.6 180.2 c-0.1 0.1 -0.2 0.7 -0.3 1.20.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.2 0.0 0.4 -0.3 0.5 -0.90.1 -0.9 -0.4 -1.2 -1.0 -0.7zM128.0 188.9 c0.0 0.3 1.7 1.4 2.1 1.40.6 -0.0 0.1 -0.9 -0.6 -1.3-1.0 -0.5 -1.5 -0.5 -1.5 -0.1z","role":"hi"},{"d":"M114.0 185.6 c0.0 3.5 0.1 5.7 0.3 5.90.2 0.1 0.3 0.6 0.3 1.1-0.0 0.5 0.3 1.4 0.7 2.10.4 0.6 0.9 1.7 1.1 2.30.4 1.6 3.3 4.7 5.0 5.50.8 0.3 1.4 0.7 1.4 0.90.0 0.9 2.8 1.6 4.6 1.32.2 -0.4 6.4 -1.9 7.0 -2.40.3 -0.3 0.7 -0.2 1.5 0.22.5 1.3 5.0 0.9 6.9 -1.0 l1.1 -1.1-1.2 0.2 c-0.6 0.1 -1.4 0.1 -1.8 -0.1-0.4 -0.2 -0.8 -0.2 -1.3 -0.0-0.9 0.4 -2.9 0.1 -3.6 -0.6-1.5 -1.4 -6.9 -3.5 -9.7 -3.6-1.6 -0.1 -3.1 -0.4 -3.7 -0.7-2.1 -1.1 -3.0 -0.6 -1.6 0.71.6 1.5 3.8 2.2 8.5 2.61.0 0.1 1.6 0.3 1.6 0.60.0 0.8 -5.6 1.0 -7.2 0.3-3.5 -1.5 -4.2 -2.0 -4.5 -2.9-0.2 -0.5 -0.5 -1.0 -0.8 -1.1-0.3 -0.1 -0.5 -0.5 -0.5 -0.90.0 -0.4 -0.5 -1.1 -1.0 -1.6-0.8 -0.7 -1.0 -1.1 -0.9 -1.90.0 -0.6 -0.1 -1.3 -0.3 -1.6-0.5 -0.6 -1.2 -4.8 -1.0 -6.30.1 -1.2 1.7 -1.6 2.3 -0.60.2 0.3 0.4 0.3 1.1 -0.00.6 -0.4 1.1 -0.4 1.9 -0.10.6 0.2 1.3 0.4 1.5 0.40.8 -0.0 0.2 -0.6 -0.9 -0.8-0.5 -0.1 -1.4 -0.7 -2.0 -1.4-0.9 -1.0 -1.2 -1.1 -2.9 -1.1 l-1.8 -0.00.0 5.6zM145.6 180.7 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.03.3 -1.7 7.0 -0.9 8.8 2.01.4 2.2 1.5 2.8 0.3 3.0-0.5 0.1 -1.4 0.6 -1.9 1.0-0.7 0.6 -1.4 0.8 -2.7 0.8-2.5 0.0 -3.5 1.0 -2.6 2.60.2 0.3 0.4 0.8 0.5 1.20.1 0.3 0.6 0.8 1.2 1.21.2 0.7 1.3 1.7 0.3 3.7-0.4 0.7 -0.7 1.5 -0.7 1.70.0 0.2 -0.2 0.5 -0.4 0.8-0.2 0.2 -0.8 1.0 -1.4 1.8 l-0.9 1.30.9 0.1 c1.1 0.2 2.6 -1.4 3.7 -3.90.5 -1.0 1.0 -2.1 1.1 -2.40.2 -0.3 0.3 -1.3 0.3 -2.20.0 -1.9 0.7 -2.7 2.1 -2.71.8 -0.0 2.9 -1.5 3.0 -4.30.1 -2.3 0.0 -2.5 -0.6 -2.6-0.4 -0.1 -1.0 -0.8 -1.7 -2.2-0.7 -1.5 -1.4 -2.3 -2.4 -3.0-1.3 -0.8 -1.4 -1.1 -1.3 -1.90.1 -0.9 0.1 -1.0 -1.4 -1.0 l-1.5 0.00.8 0.6zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.4 0.9 0.7 1.8 0.82.9 0.4 5.3 3.8 2.6 3.6-0.6 -0.0 -3.5 -0.2 -6.3 -0.4-6.3 -0.4 -6.7 -0.3 -6.6 1.1 l0.1 1.04.8 -0.0 c3.6 -0.0 5.3 -0.2 6.8 -0.61.2 -0.4 2.3 -0.5 2.9 -0.41.1 0.2 1.5 -1.0 0.5 -1.4-0.3 -0.1 -0.5 -0.4 -0.5 -0.60.0 -0.7 -2.1 -2.7 -3.3 -3.4-0.6 -0.3 -1.5 -0.8 -1.8 -1.1-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7zM155.5 201.1 c-1.3 1.7 -2.1 3.2 -2.3 4.1 l-0.2 1.11.6 -0.01.7 -0.00.0 -2.8 c0.0 -2.8 -0.2 -3.3 -0.8 -2.4z","role":"ink"}],"body":[{"d":"M196.5 120.7 c0.8 0.8 1.4 1.0 3.4 1.31.3 0.2 2.7 0.5 3.2 0.60.4 0.2 1.8 0.5 3.1 0.73.6 0.5 5.2 1.3 5.2 2.50.0 1.0 0.7 2.5 0.7 1.60.0 -0.5 2.0 -0.5 2.8 0.10.3 0.2 0.6 0.8 0.6 1.20.0 0.6 0.2 0.7 1.6 0.74.3 0.1 5.3 4.1 1.6 6.1-1.3 0.7 -1.9 1.2 -2.0 1.8-0.1 0.5 -0.3 1.2 -0.5 1.6-0.4 1.0 -0.4 2.4 0.0 2.40.5 -0.0 0.4 1.5 -0.2 2.2-0.3 0.4 -0.5 1.4 -0.5 2.30.0 1.7 -0.8 5.0 -1.4 5.6-0.5 0.5 -0.4 1.7 0.2 2.10.3 0.3 0.5 1.6 0.7 4.30.1 2.2 0.4 4.5 0.7 5.30.3 0.7 0.6 2.0 0.8 2.80.2 0.8 0.8 2.4 1.3 3.40.5 1.1 1.0 2.2 1.0 2.40.0 0.2 0.5 0.9 1.0 1.40.6 0.6 1.0 1.1 1.0 1.30.0 0.2 0.4 0.8 1.0 1.41.2 1.4 1.6 2.5 1.0 3.6-0.9 1.7 -4.0 0.7 -4.0 -1.40.0 -0.3 -0.4 -0.8 -0.8 -1.2-0.5 -0.4 -0.8 -0.8 -0.8 -1.00.0 -0.2 -0.4 -0.9 -0.9 -1.5-1.3 -1.4 -2.5 -2.9 -2.9 -3.5-2.3 -3.4 -3.2 -5.3 -3.2 -7.00.0 -0.6 -0.5 -2.1 -1.1 -3.4-0.6 -1.3 -1.3 -3.0 -1.5 -3.9-0.2 -0.8 -0.6 -1.8 -0.9 -2.2-1.0 -1.1 -2.6 -3.6 -2.9 -4.4-0.1 -0.4 -0.5 -0.9 -0.9 -1.1-0.3 -0.2 -0.6 -0.5 -0.6 -0.80.0 -0.2 -0.5 -0.8 -1.0 -1.4-0.5 -0.5 -1.2 -1.4 -1.4 -1.9-0.2 -0.6 -0.7 -1.0 -1.0 -1.0-1.1 -0.0 -1.9 -0.7 -1.9 -1.50.0 -0.6 -0.3 -1.0 -1.5 -1.5-0.8 -0.4 -1.6 -1.0 -1.7 -1.3-0.1 -0.4 -0.5 -0.8 -0.8 -0.9-0.3 -0.1 -0.8 -0.6 -0.9 -1.2-0.2 -0.5 -0.8 -1.2 -1.4 -1.5-0.6 -0.3 -1.0 -0.7 -1.0 -0.90.0 -0.2 -0.3 -0.4 -0.6 -0.5-0.3 -0.1 -1.1 -0.8 -1.8 -1.5-1.2 -1.3 -8.4 -5.3 -9.6 -5.3-0.3 -0.0 -1.1 -0.3 -1.8 -0.7-2.1 -1.1 -2.6 -1.3 -4.9 -1.7-1.2 -0.2 -3.2 -0.7 -4.4 -1.0-3.7 -1.1 -15.5 -1.1 -16.9 0.1-0.8 0.6 -4.1 1.6 -5.6 1.6-0.7 -0.0 -1.9 0.3 -2.6 0.7-0.7 0.4 -1.9 0.8 -2.6 0.9-0.8 0.1 -1.6 0.4 -1.8 0.7-0.3 0.3 -1.3 0.7 -2.2 1.0-1.0 0.3 -1.9 0.6 -2.1 0.8-0.2 0.1 -0.8 0.3 -1.3 0.5-0.5 0.1 -1.6 0.8 -2.3 1.4-0.8 0.6 -1.5 1.1 -1.8 1.1-0.2 -0.0 -0.5 0.3 -0.6 0.6-0.2 0.7 -1.7 1.7 -3.9 2.6-0.8 0.4 -1.7 0.8 -1.9 1.1-0.2 0.2 -0.6 0.4 -0.8 0.4-0.5 -0.0 -4.8 4.1 -6.8 6.4-0.7 0.9 -1.5 1.6 -1.7 1.6-0.7 -0.0 -2.1 1.4 -2.1 2.10.0 0.9 -0.6 1.9 -1.2 1.9-0.3 -0.0 -0.5 0.3 -0.5 0.60.0 0.4 -0.3 0.7 -0.6 0.9-0.3 0.1 -1.0 0.9 -1.4 1.7-0.4 0.8 -1.1 1.6 -1.4 1.7-0.3 0.1 -0.6 0.4 -0.6 0.60.0 0.2 -0.6 1.1 -1.3 1.9-0.8 0.9 -1.3 1.8 -1.3 2.30.0 1.0 -0.6 2.1 -1.1 2.1-0.6 -0.0 -1.7 2.8 -2.9 7.2-0.7 3.0 -0.9 10.4 -0.2 11.20.4 0.5 0.5 1.0 0.4 1.8-0.1 0.8 0.0 1.7 0.5 2.70.5 1.1 0.6 1.9 0.5 2.8-0.2 1.1 -0.1 1.3 0.4 1.50.5 0.1 0.7 0.5 0.9 2.10.3 2.6 0.5 3.1 1.0 3.10.3 -0.0 0.5 0.2 0.5 0.50.0 0.2 0.2 0.6 0.4 0.70.2 0.1 0.8 1.1 1.4 2.20.5 1.1 1.3 2.2 1.6 2.40.3 0.2 0.8 1.1 0.9 1.90.5 2.2 2.0 4.2 3.9 5.21.4 0.7 1.7 1.0 1.9 2.10.2 1.1 1.7 3.1 2.4 3.10.2 -0.0 37.0 -23.1 38.6 -24.10.5 -0.4 0.5 -0.5 -0.1 -2.2-0.4 -1.0 -0.8 -2.8 -0.9 -4.0-0.1 -1.2 -0.4 -2.5 -0.6 -2.9-0.4 -0.8 -0.5 -2.2 -0.2 -2.60.1 -0.1 0.2 -0.8 0.1 -1.4-0.1 -0.8 0.1 -1.1 0.4 -1.10.3 -0.0 0.7 -0.5 0.9 -1.10.3 -0.9 0.6 -1.2 1.3 -1.31.1 -0.1 1.3 -1.0 0.5 -1.8-1.8 -1.8 -2.4 -4.4 -1.3 -6.00.3 -0.4 0.6 -1.3 0.7 -2.10.6 -4.5 2.5 -5.1 3.9 -1.20.6 1.4 1.9 2.6 3.1 2.60.2 -0.0 0.5 0.6 0.8 1.30.3 0.8 0.7 1.5 1.3 1.71.5 0.8 2.5 1.9 3.4 3.80.4 1.0 1.0 1.8 1.2 1.80.6 -0.0 0.5 2.1 -0.1 4.3-0.3 1.0 -0.5 3.0 -0.5 4.40.0 1.8 -0.1 2.7 -0.5 2.9-0.7 0.6 -0.9 2.0 -0.6 3.20.3 1.0 0.3 1.0 1.1 -1.00.4 -1.1 1.1 -2.4 1.5 -2.90.4 -0.5 1.0 -1.6 1.5 -2.40.4 -0.8 0.9 -1.6 1.0 -1.70.2 -0.1 0.3 -1.7 0.3 -3.5 l0.0 -3.32.3 -0.0 c2.5 -0.0 2.8 0.2 5.7 2.7 l1.7 1.50.2 3.4 c0.2 2.4 0.1 3.4 -0.2 3.6-0.2 0.1 -0.4 0.7 -0.4 1.20.0 0.5 -0.7 2.4 -1.7 4.1-0.9 1.7 -1.7 3.3 -1.7 3.60.0 0.3 -0.8 1.6 -1.7 3.0-1.7 2.5 -2.6 4.5 -2.2 4.90.4 0.3 1.0 -0.5 1.5 -2.20.4 -1.2 0.7 -1.6 1.3 -1.71.0 -0.1 2.1 -1.9 2.1 -3.20.0 -1.2 0.3 -1.7 1.2 -2.00.7 -0.2 1.4 -1.5 2.2 -4.00.3 -1.0 0.7 -1.5 1.5 -1.80.6 -0.3 1.2 -0.8 1.3 -1.10.1 -0.4 0.5 -0.6 1.0 -0.61.0 -0.0 1.0 -0.5 -0.0 -0.9-1.2 -0.6 -1.2 -2.1 0.1 -2.11.1 -0.0 1.7 0.5 1.7 1.60.0 0.4 0.2 0.8 0.3 0.80.2 -0.0 0.3 0.4 0.3 0.80.0 0.7 -0.2 0.8 -0.8 0.8-2.2 -0.0 -1.6 3.8 0.6 4.22.1 0.4 3.7 3.4 2.2 4.3-0.2 0.1 -0.4 0.9 -0.6 1.7-0.1 1.0 -0.4 1.5 -0.7 1.5-0.3 -0.0 -0.5 0.3 -0.5 0.6-0.0 0.3 -0.4 1.2 -0.8 1.9-0.5 0.7 -1.0 1.7 -1.1 2.0-0.2 0.4 -0.5 0.8 -0.6 0.8-0.4 0.2 -3.1 4.7 -3.1 5.20.0 0.2 -0.2 0.4 -0.4 0.4-0.2 -0.0 -1.0 0.9 -1.6 2.0-0.6 1.1 -1.6 2.5 -2.1 3.1-0.5 0.6 -0.9 1.2 -0.9 1.50.0 0.3 -0.5 1.0 -1.2 1.6-0.6 0.6 -1.2 1.3 -1.2 1.60.0 0.2 -0.6 1.0 -1.3 1.8-0.7 0.8 -1.3 1.6 -1.3 1.90.0 0.3 -0.3 0.8 -0.7 1.1-0.6 0.5 -0.6 0.8 -0.3 1.30.5 0.9 0.9 0.9 1.6 -0.20.3 -0.5 1.3 -1.8 2.3 -2.80.9 -1.0 2.1 -2.5 2.5 -3.31.0 -2.0 4.7 -7.7 6.0 -9.10.5 -0.6 1.8 -2.8 2.7 -4.71.0 -2.0 2.4 -4.4 3.2 -5.41.7 -2.0 2.3 -3.2 2.7 -4.80.4 -1.8 3.8 -0.2 3.8 1.80.0 0.2 0.5 0.9 1.2 1.51.1 0.9 1.2 1.2 1.0 2.1-0.1 0.6 -0.3 2.4 -0.4 4.0 l-0.2 2.9-1.9 2.1 c-1.7 1.9 -4.5 6.5 -6.1 9.9-0.3 0.6 -1.4 2.1 -2.3 3.2-1.8 2.0 -2.4 2.8 -4.3 5.7-0.6 0.8 -1.4 1.7 -1.9 2.0-1.7 1.0 -3.6 4.1 -3.0 5.00.4 0.8 0.5 0.8 0.8 0.20.2 -0.4 0.9 -1.2 1.6 -1.80.7 -0.7 1.2 -1.4 1.2 -1.60.0 -0.2 0.5 -0.6 1.1 -0.91.2 -0.6 3.9 -3.3 3.9 -3.90.0 -0.2 0.3 -0.8 0.8 -1.30.4 -0.5 1.1 -1.4 1.6 -2.00.5 -0.6 1.4 -1.7 2.1 -2.40.7 -0.7 1.2 -1.5 1.2 -1.70.0 -0.2 0.3 -0.6 0.6 -0.80.4 -0.3 0.7 -1.0 0.9 -2.00.1 -0.9 0.3 -1.7 0.4 -1.80.7 -0.5 1.7 -2.7 1.7 -3.60.0 -0.9 0.9 -2.7 1.7 -3.40.1 -0.1 0.3 -0.4 0.4 -0.80.4 -1.0 2.3 -0.6 2.8 0.50.2 0.5 0.6 0.9 0.7 0.90.2 -0.0 0.4 0.3 0.5 0.70.1 0.4 0.4 1.0 0.7 1.40.3 0.3 0.7 0.9 0.9 1.30.2 0.4 0.7 0.8 1.2 0.90.5 0.1 1.0 0.6 1.3 1.30.3 0.6 1.0 1.3 1.7 1.60.6 0.3 1.5 0.9 1.8 1.31.2 1.3 3.1 2.5 5.6 3.71.3 0.6 3.3 1.6 4.3 2.22.6 1.4 5.4 1.9 6.5 1.30.5 -0.3 2.1 -0.5 3.9 -0.51.7 -0.0 3.4 -0.1 3.8 -0.10.4 -0.0 1.1 -0.6 1.6 -1.50.5 -0.8 1.6 -1.9 2.4 -2.41.0 -0.7 1.6 -1.3 1.8 -2.00.1 -0.5 0.4 -1.1 0.5 -1.20.1 -0.1 0.6 -0.7 1.1 -1.41.4 -2.0 3.8 -4.7 4.3 -4.81.7 -0.5 1.9 -4.1 1.0 -13.8-1.2 -12.7 -1.2 -12.8 -2.1 -13.5-0.4 -0.4 -1.1 -0.9 -1.4 -1.2-0.4 -0.3 -0.9 -1.0 -1.3 -1.7-0.4 -0.6 -0.9 -1.5 -1.2 -2.0-0.3 -0.5 -0.8 -1.2 -1.0 -1.8-0.2 -0.5 -0.5 -0.9 -0.6 -0.9-0.1 -0.0 -0.4 -0.4 -0.5 -0.8-0.1 -0.4 -0.6 -1.3 -1.2 -1.9-0.5 -0.6 -1.2 -1.9 -1.5 -2.9-0.4 -1.0 -0.8 -2.1 -1.0 -2.3-1.5 -2.0 -2.5 -5.4 -2.7 -9.4 l-0.2 -4.11.1 -2.3 c0.7 -1.5 1.4 -2.4 2.1 -2.80.5 -0.3 1.3 -0.8 1.7 -1.12.6 -2.2 9.3 -0.8 11.6 2.50.4 0.6 1.3 1.6 1.9 2.10.6 0.6 1.1 1.3 1.1 1.60.0 0.3 0.2 0.9 0.5 1.51.3 2.6 1.7 4.3 1.7 7.90.0 3.4 -0.6 8.8 -1.0 9.1-0.6 0.4 -1.0 3.5 -0.9 6.70.3 4.2 -0.5 13.7 -1.1 13.9-0.6 0.2 -0.6 1.4 0.2 3.50.3 1.0 0.8 2.4 1.0 3.30.2 0.8 0.5 1.9 0.7 2.50.2 0.5 0.5 1.6 0.7 2.40.2 0.8 0.5 1.4 0.6 1.40.3 -0.0 1.1 2.1 1.7 4.40.3 0.9 0.9 2.2 1.7 3.11.1 1.3 1.4 1.8 1.6 3.70.2 1.2 0.4 2.3 0.6 2.60.2 0.2 0.4 1.0 0.5 1.60.1 0.8 0.5 1.4 0.9 1.61.1 0.6 0.8 4.4 -0.4 4.7-1.2 0.3 -1.1 1.1 0.2 1.31.1 0.2 1.6 1.6 0.9 2.4-0.8 1.0 -2.2 10.9 -1.8 13.20.3 1.9 -0.3 2.4 -2.9 2.4-1.4 0.0 -1.7 0.1 -1.0 0.30.5 0.1 4.8 0.3 9.6 0.3 l8.7 0.00.0 -10.70.0 -10.7-1.1 -0.0-1.1 -0.0-0.4 -2.4 c-0.2 -1.3 -0.6 -3.9 -0.9 -5.7-0.3 -1.8 -0.7 -3.7 -0.8 -4.1-0.6 -1.4 -0.4 -3.7 0.3 -3.80.3 -0.0 0.4 -0.1 0.2 -0.2-0.2 -0.1 -0.4 -0.7 -0.4 -1.40.0 -1.5 -0.5 -2.5 -0.9 -1.6-0.1 0.3 -0.6 0.6 -1.0 0.6-0.8 -0.0 -1.4 -1.7 -1.0 -2.80.1 -0.3 -0.1 -0.9 -0.6 -1.3-0.7 -0.7 -0.8 -1.1 -0.8 -3.3 l-0.0 -2.51.5 -0.0 c1.4 -0.0 1.5 -0.1 1.5 -0.80.0 -0.5 0.2 -0.9 0.3 -1.00.2 -0.1 0.3 -0.6 0.3 -1.00.0 -1.2 0.7 -1.5 2.9 -1.6 l2.1 -0.10.0 -3.9 c0.0 -3.0 -0.1 -3.9 -0.4 -3.9-0.5 -0.0 -1.4 -3.9 -1.1 -4.80.1 -0.3 0.5 -0.6 0.9 -0.60.6 -0.0 0.6 -0.3 0.6 -4.3-0.0 -2.3 -0.1 -4.0 -0.2 -3.7-0.2 0.7 -1.8 0.8 -1.8 0.2-0.0 -0.7 -0.9 1.9 -0.9 2.8-0.0 0.7 -0.2 0.9 -0.8 0.9-1.3 -0.0 -2.3 -3.0 -2.0 -5.60.1 -1.2 0.1 -2.0 -0.1 -2.0-0.7 -0.0 -2.1 0.9 -2.1 1.20.0 0.2 -0.4 0.4 -0.9 0.4 l-0.9 -0.00.2 -5.6 c0.3 -8.4 0.4 -9.3 1.3 -9.50.4 -0.1 0.8 -0.4 0.8 -0.70.1 -0.5 -0.1 -0.5 -1.1 -0.4-1.0 0.2 -1.2 0.1 -1.4 -0.5-0.1 -0.4 -0.3 -0.7 -0.4 -0.7-0.8 -0.0 -2.0 -1.7 -2.2 -3.1-0.2 -1.5 -0.2 -1.6 0.7 -1.60.5 -0.0 0.9 0.2 0.9 0.30.0 0.2 0.5 0.6 1.1 0.81.2 0.5 1.9 -0.1 1.9 -1.90.0 -0.9 0.0 -0.9 2.2 -0.9 l2.2 -0.0-0.0 2.2 c-0.1 3.1 -0.1 3.1 0.6 3.11.1 -0.0 2.3 7.7 1.4 9.0-0.3 0.5 -0.3 0.6 0.2 0.60.3 -0.0 0.8 0.4 1.0 0.90.3 0.7 0.4 -4.5 0.4 -19.7 l0.0 -20.6-2.1 -0.0-2.2 -0.00.2 1.2 c0.2 1.1 0.1 1.2 -0.7 1.2-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.4 0.6 0.5 2.2 0.41.0 -0.1 1.1 0.0 1.3 1.10.3 1.5 0.0 1.9 -1.3 1.8-2.6 -0.2 -3.9 0.3 -3.6 1.50.2 1.3 0.1 1.2 1.6 1.52.0 0.3 2.5 1.8 1.1 3.2-0.7 0.7 -0.8 1.1 -0.8 3.6-0.1 5.0 -0.0 4.9 -1.1 5.3-0.8 0.3 -0.9 0.5 -0.8 1.50.1 1.1 0.1 1.2 -0.7 1.2-1.4 -0.0 -2.3 -1.0 -2.1 -2.60.1 -1.2 0.0 -1.3 -0.8 -1.4-0.5 -0.1 -0.9 -0.3 -0.9 -0.60.0 -0.3 -0.3 -0.5 -0.7 -0.5-0.4 -0.0 -0.9 -0.2 -1.2 -0.5-0.3 -0.3 -0.9 -0.5 -1.5 -0.5-1.5 -0.0 -1.8 -1.6 -0.7 -4.20.7 -1.6 0.7 -2.5 -0.1 -2.5-0.5 -0.0 -1.2 -10.0 -0.7 -10.70.1 -0.2 -10.7 -0.3 -30.6 -0.3 l-30.8 -0.01.0 1.0z m56.3 2.9 c0.1 1.3 0.4 3.0 0.6 4.00.5 2.3 -1.7 4.3 -3.2 2.8-0.2 -0.2 -0.6 -0.4 -0.9 -0.4-0.5 -0.0 -3.4 -1.3 -3.7 -1.7-0.1 -0.1 -0.6 -0.4 -1.2 -0.6-1.8 -0.6 -1.1 -3.8 0.9 -3.80.6 -0.0 1.4 -0.4 2.1 -1.00.6 -0.6 1.4 -1.0 1.7 -1.00.3 -0.0 0.6 -0.2 0.7 -0.30.1 -0.2 0.8 -0.3 1.5 -0.3 l1.3 -0.00.2 2.2z m-30.3 -1.7 c0.6 0.2 0.7 1.8 0.2 1.8-0.2 -0.0 -0.3 0.2 -0.3 0.50.0 1.1 -2.5 0.5 -2.6 -0.6-0.2 -1.6 1.1 -2.3 2.7 -1.7z m-71.4 33.9 c0.9 1.0 0.6 3.5 -0.6 4.4-0.7 0.6 -0.8 1.1 -1.1 3.7-0.3 3.1 -0.8 4.8 -1.4 4.8-0.2 -0.0 -0.3 0.3 -0.3 0.60.0 1.4 -1.6 1.8 -3.9 0.9-1.0 -0.4 -3.7 -0.6 -6.1 -0.5-1.0 0.1 -0.2 1.5 0.9 1.72.1 0.3 3.3 1.0 3.3 1.90.0 1.5 -10.9 0.5 -12.2 -1.1-0.8 -1.0 -1.7 -1.1 -2.0 -0.2-0.8 2.5 -7.8 0.9 -7.8 -1.80.0 -1.3 1.5 -3.8 2.3 -3.80.9 -0.0 1.7 -1.1 1.7 -2.10.0 -0.8 0.1 -0.9 0.9 -0.90.6 -0.0 1.2 -0.4 1.7 -1.12.0 -2.4 4.0 -3.9 5.0 -3.91.5 -0.0 3.3 -0.6 4.3 -1.52.0 -1.7 13.9 -2.6 15.2 -1.1z m114.6 13.6 c0.0 0.8 -0.1 1.0 -0.8 1.0-0.7 -0.0 -0.8 -0.2 -0.8 -1.00.0 -0.8 0.1 -1.0 0.8 -1.00.7 -0.0 0.8 0.2 0.8 1.0z m-0.1 3.4 c0.9 0.9 0.5 1.9 -0.8 1.9-1.1 -0.0 -1.2 -0.1 -1.2 -1.20.0 -1.2 1.0 -1.7 1.9 -0.8z m2.1 3.3 c0.1 0.5 0.2 0.6 0.3 0.40.1 -0.9 1.6 -0.4 2.0 0.60.6 1.5 0.1 6.3 -0.6 6.3-0.4 -0.0 -0.8 0.6 -1.2 1.5-0.4 0.8 -0.7 1.5 -0.9 1.5-0.1 -0.0 -0.4 0.5 -0.5 1.2-0.2 1.1 -0.3 1.2 -1.4 1.2-0.9 -0.0 -1.2 0.1 -1.2 0.60.0 0.7 1.1 2.1 1.6 2.10.2 -0.0 0.4 0.7 0.4 1.7 l0.0 1.7-1.2 -0.1 c-1.1 -0.1 -1.3 -0.2 -1.6 -1.2-0.2 -0.6 -0.4 -1.1 -0.6 -1.1-0.1 -0.0 -0.3 -0.2 -0.3 -0.50.0 -0.3 -0.3 -0.6 -0.6 -0.7-0.4 -0.1 -0.7 -0.4 -0.9 -0.8-0.2 -0.7 0.5 -6.4 0.8 -6.40.1 -0.0 0.3 -0.9 0.5 -2.10.5 -4.8 0.5 -4.9 1.4 -4.90.8 -0.0 0.8 -0.0 0.7 2.8-0.1 2.8 -0.1 2.8 0.6 2.80.4 -0.0 0.7 -0.2 0.7 -0.30.0 -0.2 0.5 -0.3 1.0 -0.31.0 -0.0 1.1 -0.0 0.9 -1.4-0.2 -1.4 -0.6 -2.1 -0.8 -1.4-0.6 1.7 -1.8 0.4 -1.8 -1.9 l0.0 -2.31.1 0.1 c0.8 0.1 1.1 0.3 1.3 0.9z m-146.6 7.5 c0.3 0.3 0.6 0.5 0.8 0.50.1 -0.0 1.0 0.5 1.9 1.20.9 0.6 1.9 1.2 2.2 1.21.5 -0.0 2.1 3.1 0.9 4.1-1.2 1.0 -6.5 0.6 -7.5 -0.6-0.4 -0.5 -0.9 -0.9 -1.0 -0.9-1.5 -0.0 -2.3 -1.9 -1.9 -4.30.3 -1.6 3.4 -2.4 4.6 -1.2z m110.1 7.3 c0.2 2.1 -2.6 3.3 -4.9 2.1-1.7 -0.9 -1.4 -2.2 0.5 -2.20.9 -0.0 1.6 -0.2 1.9 -0.50.8 -0.9 2.4 -0.5 2.5 0.6z m39.3 50.4 c1.3 2.8 1.1 3.2 -1.6 3.2 l-2.4 -0.00.2 -1.6 c0.5 -3.7 2.4 -4.5 3.8 -1.6z","role":"wash"},{"d":"M253.5 120.7 c-0.1 0.6 -0.1 1.3 0.1 1.50.1 0.3 0.2 0.8 0.1 1.1-0.1 0.4 0.0 0.5 0.5 0.40.5 -0.2 0.6 0.1 0.6 2.3-0.0 2.4 -0.7 5.6 -1.2 5.6-0.1 -0.0 -0.1 0.6 0.1 1.40.2 0.9 0.2 1.5 -0.0 1.8-0.4 0.4 -0.1 1.5 0.3 1.50.1 -0.0 0.3 -0.6 0.4 -1.30.1 -0.7 0.4 -2.6 0.7 -4.20.3 -1.7 0.6 -4.8 0.7 -7.1 l0.2 -4.1-1.1 -0.0 c-0.9 -0.0 -1.1 0.1 -1.3 1.0zM151.3 140.6 c0.0 0.1 -0.7 0.4 -1.5 0.7-0.8 0.2 -1.5 0.6 -1.5 0.70.0 0.6 -0.9 0.3 -1.1 -0.4-0.2 -0.9 -1.1 -0.8 -1.8 0.2-0.7 1.0 -1.5 1.1 -3.1 0.1-1.3 -0.8 -2.7 -0.8 -4.9 -0.0-0.7 0.3 -2.0 0.6 -2.9 0.7-0.8 0.1 -1.8 0.3 -2.2 0.5-0.4 0.2 -1.7 0.9 -2.9 1.5-1.2 0.6 -2.4 1.3 -2.6 1.6-0.2 0.3 -1.1 0.9 -1.9 1.3-4.5 2.5 -6.7 4.1 -6.7 5.00.0 0.2 -0.9 1.3 -2.1 2.3-1.2 1.1 -2.5 2.8 -3.1 3.9-0.6 1.1 -1.1 1.9 -1.2 1.9-0.2 -0.0 -0.3 0.2 -0.3 0.40.0 0.2 -0.4 0.8 -1.0 1.2-1.1 1.0 -1.7 2.0 -1.5 2.60.2 0.6 0.7 0.5 0.9 -0.20.1 -0.3 1.0 -1.4 1.9 -2.41.0 -1.0 2.3 -2.5 2.9 -3.30.6 -0.8 1.7 -1.8 2.5 -2.40.7 -0.6 2.5 -2.1 3.9 -3.43.6 -3.5 4.9 -4.2 10.7 -5.50.6 -0.2 1.7 -0.5 2.4 -0.81.1 -0.5 6.0 -1.4 8.9 -1.70.6 -0.0 1.5 -0.2 2.0 -0.40.5 -0.2 1.3 -0.3 1.8 -0.21.0 0.2 5.5 0.3 8.2 0.10.8 -0.0 1.9 -0.0 2.3 0.01.2 0.1 1.2 -0.8 -0.2 -1.8 l-1.2 -0.91.1 -0.2 c1.6 -0.3 0.7 -0.9 -1.6 -1.1-1.1 -0.1 -2.5 -0.2 -3.2 -0.3-0.7 -0.1 -1.3 -0.0 -1.3 0.1zM161.3 142.6 c0.0 0.4 0.2 0.7 0.4 0.80.5 0.2 -0.4 2.3 -1.1 2.3-0.3 -0.0 -0.2 0.2 0.3 0.71.4 1.1 3.6 2.1 4.0 1.90.4 -0.2 1.9 0.6 1.9 1.10.0 0.1 0.1 0.2 0.3 0.10.1 -0.0 0.2 0.1 0.2 0.20.0 0.2 0.3 0.4 0.7 0.50.8 0.2 0.9 -0.1 0.3 -0.7-0.3 -0.3 -0.3 -0.5 0.1 -0.70.8 -0.5 1.0 -0.4 1.3 0.50.5 1.2 0.4 1.5 -0.2 1.5-0.7 -0.0 -0.6 0.6 0.2 1.00.5 0.3 0.7 0.3 0.9 -0.10.3 -0.8 2.1 -0.0 2.1 1.00.0 0.5 0.2 0.9 0.5 1.00.3 0.1 0.5 0.6 0.5 1.00.0 1.1 0.6 1.3 1.4 0.60.8 -0.7 1.7 -0.3 1.9 0.70.2 1.1 0.5 1.2 1.8 0.5 l1.2 -0.6-1.1 -1.1 c-0.7 -0.7 -1.3 -1.0 -1.6 -0.9-0.3 0.1 -0.5 0.1 -0.5 -0.10.0 -0.2 -0.4 -0.3 -0.8 -0.3-0.7 -0.0 -0.8 -0.1 -0.6 -0.90.2 -0.8 0.0 -1.1 -0.9 -1.8-0.6 -0.4 -1.4 -1.2 -1.9 -1.6-2.9 -3.1 -5.2 -4.3 -6.7 -3.8-0.5 0.2 -0.5 0.1 -0.4 -0.40.3 -0.7 -0.3 -1.5 -1.1 -1.5-0.3 -0.0 -0.8 -0.3 -1.0 -0.7-0.5 -0.9 -2.0 -0.9 -2.0 -0.0zM179.8 158.8 c-0.3 0.7 0.6 2.1 1.4 2.30.3 0.1 0.8 0.5 0.9 0.90.2 0.4 1.0 1.1 1.7 1.5 l1.4 0.8-0.2 -1.3 c-0.3 -1.9 -0.7 -2.4 -1.8 -2.4-0.8 -0.0 -1.0 -0.2 -1.2 -1.0-0.1 -0.6 -0.4 -1.0 -0.7 -1.0-0.2 -0.0 -0.7 -0.1 -1.0 -0.2-0.3 -0.1 -0.6 -0.0 -0.7 0.3zM150.0 162.7 c-0.6 1.0 -0.6 1.1 -0.1 2.0 l0.6 0.90.6 -1.3 c1.2 -2.5 0.3 -3.9 -1.1 -1.7zM186.3 165.6 c0.2 0.1 0.4 0.4 0.4 0.60.0 0.3 0.3 0.6 0.6 0.80.4 0.2 0.9 0.7 1.1 1.20.8 1.6 3.3 3.0 3.3 1.80.0 -0.6 -2.4 -3.1 -3.0 -3.1-0.2 -0.0 -0.6 -0.4 -0.8 -0.8-0.3 -0.6 -0.6 -0.8 -1.2 -0.8-0.5 -0.0 -0.7 0.1 -0.4 0.2zM193.5 170.6 c-0.1 0.3 -0.3 0.9 -0.5 1.2-0.4 0.7 -0.0 1.6 0.6 1.60.3 -0.0 0.4 -0.2 0.3 -0.6-0.1 -0.4 0.0 -0.8 0.4 -1.00.6 -0.3 0.5 -1.4 -0.1 -1.7-0.2 -0.1 -0.5 0.2 -0.6 0.5zM195.7 174.7 c0.0 0.4 1.7 2.0 2.1 2.00.4 -0.0 0.3 1.9 -0.1 2.1-0.4 0.1 2.0 3.9 2.5 3.90.4 -0.0 0.2 -1.6 -0.3 -2.1-0.5 -0.5 -0.3 -1.4 0.2 -1.00.2 0.1 0.3 -0.1 0.3 -0.50.0 -0.7 -2.6 -3.7 -3.3 -3.7-0.2 -0.0 -0.5 -0.2 -0.6 -0.5-0.2 -0.6 -0.8 -0.7 -0.8 -0.2zM123.0 178.3 c-0.4 0.3 -0.3 0.3 0.3 0.30.4 -0.1 0.9 0.1 1.0 0.40.1 0.3 0.0 0.4 -0.4 0.3-0.4 -0.1 -0.4 -0.0 -0.3 0.50.1 0.5 0.0 0.6 -0.7 0.6-0.5 -0.0 -0.9 0.1 -0.9 0.30.0 0.6 -1.0 0.4 -1.2 -0.3-0.4 -1.0 -1.4 -1.5 -1.4 -0.60.0 0.7 1.0 1.5 2.2 1.70.6 0.1 1.4 0.5 1.8 0.81.1 0.9 2.3 1.2 2.3 0.60.0 -0.9 2.1 -1.3 2.4 -0.50.1 0.3 0.2 0.3 0.6 0.00.5 -0.5 0.2 -1.1 -0.7 -1.1-0.3 -0.0 -0.9 -0.4 -1.2 -0.8-0.4 -0.4 -0.9 -0.8 -1.3 -0.9-0.3 -0.1 -0.6 -0.2 -0.5 -0.30.6 -0.7 -1.1 -1.5 -2.0 -1.0zM130.7 180.1 c-0.2 0.2 -0.4 0.9 -0.4 1.40.0 0.6 -0.1 0.8 -0.2 0.6-0.3 -0.7 -0.7 -0.4 -0.7 0.40.0 1.1 0.3 1.1 0.9 -0.00.2 -0.4 0.6 -0.8 0.8 -0.70.4 0.1 0.7 -1.7 0.3 -1.9-0.2 -0.1 -0.5 -0.0 -0.7 0.2z","role":"hi"},{"d":"M196.5 120.7 c0.8 0.8 1.4 1.0 3.4 1.31.3 0.2 2.7 0.5 3.2 0.60.4 0.2 1.8 0.5 3.1 0.72.7 0.4 5.2 1.3 5.2 2.00.0 0.2 0.3 0.5 0.6 0.61.6 0.6 3.4 2.0 3.4 2.70.0 0.5 0.2 0.8 0.5 0.80.3 -0.0 0.5 -0.2 0.5 -0.50.0 -1.4 2.5 -0.4 3.8 1.51.4 2.1 0.8 3.9 -1.7 5.2-1.3 0.7 -1.9 1.2 -2.0 1.8-0.1 0.5 -0.3 1.2 -0.5 1.6-0.4 1.0 -0.4 2.4 0.0 2.40.5 -0.0 0.4 1.5 -0.2 2.2-0.3 0.4 -0.5 1.4 -0.5 2.30.0 0.9 -0.2 2.4 -0.5 3.3-0.3 0.9 -0.4 2.1 -0.3 2.70.2 0.9 0.1 1.1 -0.4 1.1-0.5 -0.0 -0.5 0.1 -0.2 0.50.2 0.3 0.6 0.5 0.9 0.50.5 -0.0 0.6 0.9 0.1 1.1-0.5 0.1 0.3 7.8 0.8 8.90.2 0.5 0.5 1.5 0.7 2.20.1 0.8 0.7 2.2 1.2 3.30.5 1.0 1.0 2.1 1.0 2.30.0 0.2 0.5 0.9 1.0 1.40.6 0.6 1.0 1.1 1.0 1.30.0 0.2 0.5 0.8 1.0 1.40.6 0.6 1.1 1.6 1.2 2.20.1 0.6 0.6 1.5 1.1 2.12.8 3.4 5.3 6.9 5.5 7.90.1 0.5 0.4 0.9 0.7 0.90.3 -0.0 0.6 0.3 0.8 0.61.5 2.8 -2.2 4.9 -5.0 2.9-0.6 -0.5 -1.6 -1.1 -2.1 -1.4-0.6 -0.4 -0.9 -0.8 -0.8 -1.20.2 -0.8 -0.8 -2.3 -2.4 -3.8-0.9 -0.8 -1.4 -1.5 -1.4 -2.00.0 -0.4 -0.2 -0.9 -0.5 -1.1-0.3 -0.2 -0.6 -0.9 -0.7 -1.4-0.1 -0.5 -0.4 -1.0 -0.5 -1.1-0.6 -0.4 -0.3 -1.5 0.4 -1.50.4 -0.0 0.7 -0.2 0.7 -0.30.0 -0.2 -0.2 -0.3 -0.5 -0.3-0.3 -0.0 -0.5 -0.3 -0.5 -0.70.0 -0.4 -0.2 -0.8 -0.5 -0.9-0.3 -0.1 -0.5 -0.5 -0.5 -0.80.0 -0.3 -0.1 -0.6 -0.3 -0.6-0.2 -0.0 -0.5 -0.5 -0.8 -1.1-0.2 -0.6 -0.8 -1.5 -1.2 -1.9-0.4 -0.5 -0.9 -1.1 -1.1 -1.4-2.4 -3.5 -3.2 -5.3 -3.2 -7.00.0 -0.7 -0.7 -2.4 -1.6 -4.3-0.9 -1.7 -1.7 -3.6 -1.8 -4.2-0.1 -0.6 -0.8 -1.6 -1.4 -2.3-0.6 -0.7 -1.1 -1.4 -1.1 -1.60.0 -0.4 -2.2 -3.7 -3.1 -4.7-0.3 -0.4 -0.8 -1.0 -1.1 -1.5-0.3 -0.5 -1.1 -1.1 -1.8 -1.4-1.0 -0.4 -1.3 -0.7 -1.3 -1.40.0 -0.6 -0.3 -1.0 -1.2 -1.4-1.5 -0.7 -3.8 -2.9 -3.8 -3.60.0 -0.3 -0.5 -0.8 -1.2 -1.2-0.6 -0.4 -1.2 -0.8 -1.2 -1.00.0 -0.2 -0.3 -0.4 -0.6 -0.5-0.3 -0.1 -1.1 -0.8 -1.8 -1.5-1.2 -1.3 -8.4 -5.3 -9.6 -5.3-0.3 -0.0 -1.1 -0.3 -1.8 -0.7-2.1 -1.1 -2.6 -1.3 -4.9 -1.7-1.2 -0.2 -3.2 -0.7 -4.4 -1.0-3.7 -1.1 -15.5 -1.1 -16.9 0.1-0.6 0.5 -4.0 1.6 -5.0 1.6-0.4 -0.0 -1.1 0.4 -1.6 0.8-1.1 1.0 -2.0 1.3 -2.8 0.9-0.7 -0.4 -3.0 0.0 -3.4 0.6-0.1 0.2 -1.0 0.6 -2.0 0.9-1.0 0.3 -1.9 0.6 -2.1 0.8-0.2 0.1 -0.8 0.3 -1.3 0.5-0.5 0.1 -1.6 0.8 -2.3 1.4-0.8 0.6 -1.5 1.1 -1.8 1.1-0.2 -0.0 -0.5 0.3 -0.6 0.6-0.2 0.7 -1.7 1.7 -3.9 2.6-0.8 0.4 -1.7 0.8 -1.9 1.1-0.2 0.2 -0.6 0.4 -0.8 0.4-0.5 -0.0 -4.8 4.1 -6.8 6.4-0.7 0.9 -1.5 1.6 -1.7 1.6-0.7 -0.0 -2.1 1.4 -2.1 2.10.0 0.9 -0.6 1.9 -1.2 1.9-0.3 -0.0 -0.5 0.3 -0.5 0.60.0 0.4 -0.3 0.7 -0.6 0.9-0.3 0.1 -1.0 0.9 -1.4 1.7-0.4 0.8 -1.0 1.6 -1.3 1.7-0.3 0.1 -0.7 0.4 -0.8 0.8-1.0 1.7 -1.7 2.7 -2.1 2.7-0.3 -0.0 -0.4 0.4 -0.4 1.00.0 1.2 -0.5 2.3 -1.1 2.3-0.2 -0.0 -0.7 0.6 -1.0 1.2-1.1 2.8 -1.6 4.2 -1.4 4.50.1 0.2 -0.1 0.9 -0.4 1.6-0.9 1.9 -1.1 10.2 -0.3 11.10.4 0.5 0.5 1.0 0.4 1.8-0.1 0.8 0.0 1.7 0.5 2.70.5 1.1 0.6 1.9 0.5 2.8-0.2 1.1 -0.1 1.3 0.4 1.50.5 0.1 0.7 0.6 0.9 2.10.2 1.4 0.4 2.1 1.0 2.60.4 0.4 0.8 1.0 0.8 1.30.0 0.3 0.1 0.6 0.3 0.60.1 0.0 0.7 1.0 1.2 2.00.5 1.1 1.3 2.2 1.6 2.40.3 0.2 0.8 1.1 0.9 1.90.7 3.2 3.3 5.8 4.5 4.50.6 -0.6 0.2 -1.5 -2.1 -4.4-2.0 -2.6 -5.6 -9.9 -6.4 -13.3-0.7 -2.8 -1.5 -8.7 -1.3 -9.10.1 -0.1 0.3 -2.1 0.5 -4.40.4 -4.5 1.2 -7.7 2.3 -9.60.4 -0.7 0.7 -1.4 0.7 -1.70.0 -0.3 0.1 -0.7 0.3 -1.10.2 -0.3 0.6 -1.3 1.0 -2.31.3 -3.6 6.6 -10.8 7.9 -10.80.1 -0.0 0.4 -0.3 0.8 -0.60.3 -0.3 1.7 -1.5 3.0 -2.72.3 -2.1 3.5 -2.9 7.8 -5.81.2 -0.8 2.3 -1.6 2.4 -1.80.1 -0.2 1.2 -0.8 2.3 -1.31.1 -0.6 2.7 -1.5 3.4 -2.02.2 -1.6 6.1 -3.4 8.3 -3.71.1 -0.2 3.5 -0.9 5.2 -1.61.7 -0.7 3.5 -1.4 4.0 -1.50.5 -0.1 1.7 -0.5 2.8 -0.93.0 -1.1 5.5 -1.4 7.7 -1.01.1 0.2 2.9 0.5 4.0 0.71.2 0.2 2.9 0.8 4.2 1.41.2 0.6 3.0 1.3 4.1 1.52.5 0.4 3.9 0.9 5.7 2.10.8 0.5 2.4 1.5 3.7 2.11.3 0.7 2.8 1.6 3.4 2.10.6 0.4 1.2 0.8 1.4 0.80.2 -0.0 0.4 0.2 0.4 0.50.0 0.3 0.1 0.5 0.3 0.50.6 -0.0 8.2 8.1 9.9 10.50.5 0.8 1.7 2.4 2.6 3.61.0 1.3 2.3 3.7 3.3 6.00.9 2.2 2.0 4.5 2.4 5.20.5 0.7 0.8 1.4 0.8 1.50.0 0.1 0.2 0.5 0.4 0.70.3 0.3 0.7 1.1 0.9 1.70.4 1.1 0.8 1.6 2.3 4.00.4 0.5 1.3 2.3 2.2 4.10.9 1.7 1.8 3.6 2.1 4.12.7 5.1 3.6 6.7 4.7 8.20.8 1.0 1.6 2.2 1.9 2.80.3 0.6 1.0 1.5 1.6 2.00.6 0.5 1.6 1.5 2.2 2.30.6 0.9 1.5 1.7 2.3 2.00.7 0.3 1.4 0.8 1.6 1.20.6 1.1 1.7 0.7 2.5 -0.82.0 -3.9 0.7 -11.0 -2.8 -15.9-0.9 -1.2 -2.3 -3.2 -3.2 -4.5-5.5 -7.8 -7.3 -13.4 -7.6 -24.6-0.1 -3.5 1.3 -6.1 3.0 -5.71.3 0.3 2.1 -0.1 2.1 -1.10.0 -0.7 0.1 -0.8 0.8 -0.70.9 0.2 1.2 -0.5 0.4 -1.0-0.3 -0.1 -0.5 -0.6 -0.6 -1.1-0.1 -1.1 -0.9 -1.3 -0.9 -0.30.0 1.1 -1.9 2.0 -2.6 1.1-1.0 -1.3 -0.1 -2.7 1.4 -2.40.6 0.2 0.8 0.1 0.8 -0.30.0 -0.3 0.3 -0.7 0.6 -0.80.3 -0.2 0.8 -0.8 1.0 -1.40.2 -0.6 0.6 -1.4 0.7 -1.70.2 -0.3 0.3 -0.8 0.3 -1.10.0 -1.1 5.0 -4.5 6.6 -4.50.2 -0.0 0.8 -0.3 1.3 -0.71.0 -0.8 2.0 -0.9 2.4 -0.10.3 0.5 0.6 0.5 2.3 0.33.7 -0.5 5.3 -0.4 6.7 0.60.7 0.5 1.4 0.9 1.6 0.91.3 -0.0 5.2 2.6 5.2 3.50.0 0.2 0.4 0.7 0.8 1.10.8 0.6 3.8 6.1 3.8 6.90.0 0.2 0.2 0.8 0.5 1.40.3 0.5 0.7 1.3 0.8 1.80.2 0.5 0.5 1.1 0.7 1.31.3 2.0 2.0 4.9 2.0 8.80.0 2.2 0.1 4.4 0.1 4.80.1 2.7 -0.1 3.7 -1.0 4.6-1.0 1.0 -1.0 0.9 -1.2 6.7-0.1 2.8 -0.2 3.9 -0.6 4.1-0.3 0.2 -0.5 1.0 -0.5 2.20.0 1.1 -0.1 3.5 -0.1 5.4-0.0 1.9 0.1 4.2 0.3 5.10.3 1.2 0.3 1.8 0.0 2.3-0.6 1.2 0.8 6.8 2.8 10.90.5 0.9 0.8 2.0 0.8 2.30.0 0.6 2.4 4.4 2.9 4.80.2 0.1 0.4 1.0 0.6 1.90.1 1.0 0.4 2.3 0.7 2.90.2 0.6 0.4 1.7 0.4 2.50.0 0.7 0.4 1.9 0.9 2.71.2 2.1 1.2 6.3 -0.1 8.2-1.2 1.8 -2.8 6.7 -2.4 7.40.4 0.7 0.5 3.8 0.2 6.5-0.3 2.4 -1.6 2.6 -2.0 0.3-0.1 -0.8 -0.5 -1.7 -0.8 -2.1-1.2 -1.4 -1.5 -2.7 -0.8 -3.40.4 -0.4 0.8 -1.6 1.1 -3.60.4 -2.6 0.9 -4.1 2.7 -8.00.4 -0.8 -0.2 -3.3 -1.0 -4.2-0.3 -0.3 -0.6 -1.0 -0.8 -1.7-0.4 -1.4 -1.2 -3.8 -2.2 -6.6-0.4 -1.2 -0.8 -2.4 -0.8 -2.70.0 -0.3 -0.4 -1.1 -1.0 -1.7-0.5 -0.6 -1.0 -1.3 -1.0 -1.50.0 -0.2 -0.4 -1.1 -0.8 -2.0-2.1 -4.0 -3.7 -13.5 -3.4 -19.80.1 -2.4 0.3 -6.8 0.3 -9.60.1 -4.9 0.3 -7.0 1.2 -11.30.6 -3.2 -0.4 -11.2 -1.7 -12.5-0.2 -0.2 -0.3 -0.5 -0.3 -0.70.0 -1.5 -5.8 -7.8 -8.2 -8.8-3.5 -1.5 -4.6 -1.6 -7.1 -0.3-2.6 1.3 -5.4 3.5 -5.4 4.10.0 0.3 -0.4 0.8 -0.9 1.1-1.3 1.0 -1.8 6.3 -0.8 9.21.0 3.0 1.7 6.0 1.7 7.10.0 1.0 1.8 4.8 3.1 6.40.4 0.5 1.3 1.8 2.0 3.00.8 1.2 2.0 3.0 2.8 4.03.3 4.4 3.9 6.4 3.7 13.7-0.2 9.5 -0.7 11.3 -4.0 16.5-1.7 2.6 -2.2 3.1 -3.4 3.1-0.3 -0.0 -0.8 0.3 -1.2 0.6-0.4 0.3 -1.5 1.0 -2.5 1.6-1.0 0.5 -2.1 1.1 -2.3 1.3-2.9 2.4 -11.1 1.5 -17.3 -1.8-3.6 -1.9 -3.7 -2.0 -8.6 -6.8-4.2 -4.1 -4.9 -5.3 -3.3 -5.30.5 -0.0 0.8 0.1 0.8 0.30.0 0.7 2.8 2.9 4.5 3.71.1 0.5 2.1 1.0 2.2 1.20.1 0.2 0.3 0.2 0.5 0.10.1 -0.1 0.0 -0.4 -0.3 -0.6-1.3 -0.7 -4.5 -4.2 -6.0 -6.6-0.9 -1.4 -2.2 -3.3 -2.9 -4.3-0.7 -1.0 -1.3 -2.0 -1.3 -2.10.0 -0.4 -2.2 -3.6 -3.2 -4.7-1.8 -2.0 -3.4 -4.1 -4.3 -5.9-0.5 -1.1 -1.7 -2.7 -2.9 -3.8-1.1 -1.1 -2.7 -2.6 -3.6 -3.5-0.8 -0.9 -3.3 -2.9 -5.4 -4.5-4.4 -3.4 -6.7 -5.6 -7.4 -7.0-0.3 -0.5 -1.4 -1.6 -2.5 -2.4-1.1 -0.8 -2.5 -1.8 -3.1 -2.3-2.4 -2.1 -2.2 -0.6 0.4 2.21.4 1.4 3.3 4.4 3.3 5.10.0 0.3 0.2 0.5 0.3 0.51.0 -0.0 0.2 1.0 -1.0 1.2-1.7 0.3 -2.0 0.1 -2.2 -1.5-0.2 -1.4 -1.9 -3.7 -2.7 -3.7-0.2 -0.0 -0.8 -0.6 -1.3 -1.3-1.2 -1.8 -6.2 -6.8 -7.5 -7.3-0.6 -0.2 -2.6 -0.5 -4.5 -0.6-1.9 -0.1 -4.2 -0.4 -5.2 -0.7-2.9 -0.8 -6.3 -0.6 -8.7 0.4-1.1 0.5 -2.3 0.8 -2.7 0.8-1.0 -0.0 -4.5 1.8 -5.9 3.0-0.6 0.5 -1.4 1.0 -1.6 1.0-0.2 -0.0 -0.5 0.4 -0.6 0.8-0.1 0.5 -0.5 0.9 -1.0 1.0-1.2 0.3 -2.7 2.5 -1.7 2.50.2 -0.0 0.7 0.4 1.1 0.90.9 1.1 1.8 0.9 3.0 -0.90.5 -0.7 1.3 -1.6 1.8 -1.80.5 -0.3 1.1 -0.7 1.2 -1.00.2 -0.4 0.9 -0.6 2.1 -0.71.8 -0.1 2.7 -0.5 3.8 -1.50.9 -0.8 3.8 -1.3 9.4 -1.5 l4.7 -0.20.9 0.9 c0.5 0.5 0.8 1.1 0.7 1.4-0.1 0.3 -0.0 0.5 0.1 0.50.2 -0.0 0.3 -0.3 0.3 -0.70.0 -0.5 0.2 -0.7 1.0 -0.70.6 -0.0 1.0 0.2 1.0 0.40.0 0.2 0.4 0.7 1.0 1.10.6 0.4 1.2 1.3 1.5 2.20.5 1.5 1.9 2.6 3.1 2.60.2 -0.0 0.5 0.6 0.8 1.30.3 0.9 0.7 1.4 1.5 1.80.6 0.3 1.5 1.0 2.0 1.60.5 0.6 1.3 1.2 1.8 1.41.7 0.7 1.9 1.7 1.2 5.5-0.3 1.9 -0.8 4.5 -1.0 5.8-0.2 1.3 -0.6 2.6 -0.8 2.9-0.6 0.7 -0.9 2.5 -0.4 2.80.2 0.1 0.5 -0.1 0.6 -0.50.1 -0.4 1.1 -2.5 2.2 -4.51.6 -3.1 1.9 -4.0 2.0 -5.50.0 -1.0 0.1 -2.5 0.3 -3.40.2 -0.9 0.4 -2.3 0.5 -3.10.3 -2.1 2.6 -1.4 4.0 1.30.3 0.5 1.1 1.4 1.9 1.90.8 0.5 1.5 1.0 1.5 1.20.2 0.4 0.8 1.0 2.0 1.90.8 0.6 1.1 1.1 1.1 1.90.0 0.9 0.1 1.1 0.5 0.90.3 -0.1 0.8 -0.0 1.2 0.12.9 1.3 3.4 1.7 4.0 2.80.6 1.2 1.0 1.8 3.1 4.20.8 0.9 1.1 1.6 1.1 2.2-0.1 1.2 -1.3 1.1 -2.2 -0.3-1.8 -2.8 -3.5 -3.5 -3.7 -1.5-0.2 1.9 0.1 2.5 1.7 2.72.0 0.3 2.7 1.1 2.4 2.9-0.2 1.9 -1.0 4.6 -1.3 4.6-0.1 -0.0 -0.6 0.8 -1.1 1.7-1.3 2.5 -1.7 3.3 -2.1 3.6-0.5 0.3 -3.1 4.8 -3.1 5.30.0 0.2 -0.2 0.4 -0.4 0.4-0.2 -0.0 -1.0 0.9 -1.6 2.0-0.6 1.1 -1.6 2.5 -2.1 3.1-0.5 0.6 -0.9 1.2 -0.9 1.50.0 0.3 -0.5 1.0 -1.2 1.6-0.6 0.6 -1.2 1.3 -1.2 1.60.0 0.2 -0.6 1.0 -1.3 1.8-0.7 0.8 -1.3 1.6 -1.3 1.90.0 0.3 -0.3 0.8 -0.7 1.1-0.6 0.6 -0.6 1.7 -0.0 1.70.5 -0.0 4.6 -5.0 6.0 -7.33.3 -5.6 4.1 -6.9 5.3 -8.00.6 -0.6 1.8 -2.5 2.6 -4.21.6 -3.3 3.5 -6.4 4.3 -7.10.9 -0.7 1.9 -3.2 1.9 -4.80.0 -1.0 0.2 -1.6 0.5 -1.70.6 -0.3 3.8 2.4 4.1 3.50.2 0.9 1.0 1.8 2.0 2.30.6 0.3 2.8 4.0 2.8 4.70.0 0.2 -0.4 0.4 -0.8 0.5-0.6 0.2 -0.8 0.4 -0.8 1.00.0 0.4 -0.3 1.1 -0.6 1.6-0.9 1.1 -1.4 2.3 -1.1 2.30.2 -0.0 0.4 -0.3 0.6 -0.70.6 -1.3 2.8 -0.8 2.8 0.60.0 0.2 0.2 0.5 0.5 0.60.3 0.1 0.6 0.5 0.7 0.90.1 0.4 0.4 1.0 0.7 1.30.3 0.3 0.7 0.9 0.9 1.30.2 0.4 0.7 0.8 1.2 0.90.5 0.1 1.0 0.6 1.3 1.30.3 0.6 1.0 1.3 1.7 1.60.6 0.3 1.5 0.9 1.8 1.31.2 1.3 3.1 2.5 5.6 3.71.3 0.6 3.3 1.6 4.3 2.22.5 1.3 5.4 1.9 6.4 1.30.4 -0.3 2.1 -0.5 4.5 -0.54.1 -0.1 4.7 -0.4 4.7 -1.90.0 -0.3 0.2 -0.6 0.3 -0.60.2 -0.0 0.5 -0.2 0.8 -0.30.2 -0.2 1.0 -0.8 1.8 -1.30.8 -0.6 1.4 -1.3 1.4 -1.60.0 -0.4 0.2 -0.9 0.4 -1.10.2 -0.2 0.8 -1.0 1.2 -1.61.4 -2.0 3.8 -4.7 4.4 -4.81.5 -0.5 2.1 -8.4 0.9 -13.0-0.2 -0.7 -0.4 -2.2 -0.4 -3.2-0.3 -5.3 -1.6 -10.9 -2.9 -12.2-0.4 -0.4 -1.0 -1.2 -1.4 -1.8-0.4 -0.6 -1.0 -1.5 -1.3 -2.0-0.3 -0.5 -0.8 -1.3 -1.0 -1.8-0.2 -0.5 -0.5 -0.9 -0.6 -0.9-0.1 -0.0 -0.4 -0.4 -0.5 -0.8-0.1 -0.4 -0.6 -1.3 -1.2 -1.9-0.5 -0.6 -1.2 -1.9 -1.5 -2.9-0.4 -1.0 -0.8 -2.1 -1.0 -2.3-3.8 -5.1 -3.7 -16.5 0.2 -18.60.5 -0.3 1.3 -0.8 1.7 -1.11.0 -0.9 3.0 -1.2 6.4 -0.91.2 0.1 4.2 2.1 5.2 3.40.5 0.7 1.3 1.6 1.8 2.00.5 0.4 0.9 1.0 1.0 1.40.1 0.4 0.4 1.2 0.7 1.71.4 3.1 1.6 4.4 1.6 8.00.0 3.4 -0.6 8.8 -1.0 9.1-0.6 0.4 -1.0 3.5 -0.9 6.70.3 4.2 -0.5 13.7 -1.1 13.9-0.6 0.2 -0.6 1.4 0.2 3.50.3 1.0 0.8 2.4 1.0 3.30.2 0.8 0.5 1.9 0.7 2.50.2 0.5 0.5 1.6 0.7 2.40.2 0.8 0.5 1.4 0.6 1.40.3 -0.0 1.1 2.1 1.7 4.40.3 0.9 0.9 2.2 1.7 3.11.1 1.3 1.4 1.8 1.6 3.70.2 1.2 0.4 2.3 0.6 2.60.2 0.2 0.4 1.0 0.5 1.60.1 0.7 0.5 1.4 0.8 1.60.6 0.3 1.6 5.1 1.2 5.8-0.1 0.2 -0.2 0.8 -0.1 1.30.0 0.5 -0.1 1.1 -0.2 1.3-0.9 1.2 -2.3 11.5 -1.7 12.90.7 1.8 -1.0 3.2 -3.6 2.8-1.0 -0.2 -1.1 -0.1 -0.4 0.10.5 0.2 4.8 0.3 9.6 0.3 l8.7 0.0-0.0 -8.8 c0.0 -6.4 -0.1 -8.5 -0.4 -7.9-1.2 2.8 -1.4 3.4 -1.3 3.90.0 0.3 0.1 2.1 0.1 3.90.0 1.9 0.2 4.2 0.5 5.30.8 3.0 0.9 2.9 -2.8 2.9-1.8 -0.0 -3.4 -0.1 -3.5 -0.3-0.2 -0.3 0.6 -4.6 1.2 -6.50.2 -0.7 0.2 -1.7 -0.2 -3.1-0.6 -2.3 -0.4 -3.1 0.7 -4.42.0 -2.4 2.6 -6.5 1.5 -11.3-0.2 -0.8 -0.5 -2.4 -0.7 -3.4-0.7 -3.8 -2.7 -8.8 -6.9 -17.4-0.6 -1.3 -1.1 -2.5 -1.1 -2.80.0 -0.2 -0.3 -1.0 -0.6 -1.7-1.4 -2.8 -1.5 -4.5 -0.6 -6.51.6 -3.4 2.7 -14.0 2.3 -22.7-0.0 -0.9 0.1 -1.8 0.2 -2.01.9 -2.5 1.1 -8.8 -1.9 -14.9-0.9 -1.8 -1.7 -3.5 -1.7 -3.70.0 -0.2 -0.2 -0.9 -0.5 -1.6-1.1 -2.5 -1.7 -3.9 -1.9 -4.5-0.2 -0.4 -0.6 -0.6 -1.4 -0.6-1.4 -0.0 -1.6 -0.2 -1.8 -2.9-0.2 -2.2 -1.4 -3.8 -2.8 -3.8-0.3 -0.0 -0.7 -0.2 -0.9 -0.5-0.2 -0.3 -0.9 -0.6 -1.6 -0.7-0.7 -0.1 -1.5 -0.4 -1.7 -0.7-0.3 -0.2 -0.9 -0.5 -1.4 -0.6-1.0 -0.2 -1.4 -0.9 -1.1 -1.60.8 -1.7 1.5 -2.4 2.3 -2.50.7 -0.1 0.9 -0.3 0.9 -0.90.0 -0.7 0.1 -0.8 0.9 -0.81.0 0.1 1.7 -0.1 3.2 -0.90.5 -0.3 1.2 -0.4 1.5 -0.30.6 0.2 1.1 -0.3 1.1 -1.10.0 -0.3 -0.8 -0.4 -2.7 -0.4 l-2.7 -0.0-1.6 1.4 c-4.9 4.4 -5.6 4.9 -7.4 4.9-0.5 -0.0 -0.9 -0.2 -0.9 -0.40.0 -0.6 2.3 -4.6 2.8 -4.81.3 -0.5 0.5 -1.2 -1.3 -1.2 l-1.7 -0.0-0.6 1.6 c-1.3 3.6 -2.3 4.3 -4.4 3.4-1.5 -0.7 -2.4 -0.0 -2.2 1.7 l0.1 1.4-1.7 -0.0 c-0.9 -0.0 -1.8 0.1 -1.9 0.3-0.2 0.4 -2.2 0.5 -2.2 0.10.0 -0.1 -1.0 -0.5 -2.2 -0.9-3.4 -1.0 -5.4 -2.6 -5.4 -4.40.0 -0.3 -0.3 -0.6 -0.6 -0.8-0.4 -0.2 -0.7 -0.8 -0.9 -1.3 l-0.2 -1.0-9.7 -0.0-9.7 -0.01.0 1.0zM272.9 122.6 c0.2 3.1 0.4 3.3 0.4 0.50.0 -1.3 -0.1 -2.5 -0.3 -2.6-0.2 -0.1 -0.2 0.8 -0.1 2.1zM272.8 133.4 c-0.4 2.9 -0.2 18.9 0.2 21.80.2 1.4 0.3 -2.1 0.3 -11.10.0 -7.2 -0.0 -13.1 -0.1 -13.1-0.1 -0.0 -0.3 1.1 -0.4 2.4zM120.9 163.4 c-0.5 0.7 -1.2 1.3 -1.4 1.3-0.3 -0.0 -0.5 0.1 -0.5 0.30.0 0.1 -0.8 1.1 -1.8 2.2-1.0 1.0 -1.8 2.0 -1.8 2.20.0 0.2 -0.3 0.6 -0.6 0.9-0.7 0.7 -1.5 3.1 -1.8 5.4-0.1 0.8 -0.4 1.8 -0.7 2.3-0.3 0.5 -0.6 1.7 -0.7 2.6-0.1 1.0 -0.3 1.7 -0.5 1.7-0.6 -0.0 -0.6 1.1 0.0 2.30.4 0.7 0.7 1.7 0.7 2.20.0 0.5 0.1 0.8 0.3 0.80.2 -0.0 0.4 0.3 0.5 0.70.1 0.4 0.7 1.4 1.2 2.20.6 0.8 0.9 1.8 0.8 2.1-0.1 0.3 0.2 1.2 0.6 1.90.5 0.7 1.0 1.8 1.1 2.50.4 1.5 3.1 4.5 4.8 5.40.7 0.4 1.5 0.9 1.6 1.11.0 1.7 7.4 -0.8 11.2 -4.30.8 -0.8 -4.6 -2.9 -7.8 -3.0-1.6 -0.1 -3.1 -0.4 -3.7 -0.7-2.1 -1.1 -3.0 -0.6 -1.6 0.71.6 1.5 3.8 2.2 8.5 2.61.0 0.1 1.6 0.3 1.6 0.60.0 0.8 -5.6 1.0 -7.2 0.3-3.5 -1.5 -4.2 -2.0 -4.5 -2.9-0.2 -0.5 -0.5 -1.0 -0.8 -1.1-0.3 -0.1 -0.5 -0.5 -0.5 -0.90.0 -0.4 -0.5 -1.1 -1.0 -1.6-0.8 -0.7 -1.0 -1.1 -0.9 -1.90.0 -0.6 -0.1 -1.3 -0.3 -1.6-0.5 -0.6 -1.2 -4.8 -1.0 -6.30.1 -1.2 1.7 -1.6 2.3 -0.60.2 0.3 0.4 0.3 1.1 -0.00.7 -0.4 1.1 -0.4 2.0 -0.00.7 0.2 1.4 0.3 1.6 0.20.3 -0.1 -0.3 -0.5 -2.0 -1.0-3.0 -1.1 -4.4 -9.9 -1.5 -9.90.6 -0.0 2.7 1.8 2.7 2.30.0 1.5 3.8 2.2 7.1 1.34.1 -1.1 6.7 -0.9 12.7 0.90.4 0.1 0.6 -0.0 0.6 -0.30.0 -0.5 4.1 -2.6 4.9 -2.60.2 -0.0 0.7 -0.4 1.2 -0.90.5 -0.6 1.0 -0.8 1.3 -0.70.3 0.1 0.5 0.0 0.5 -0.30.0 -0.6 -0.9 -0.6 -2.8 -0.1-2.7 0.7 -3.6 1.0 -4.3 1.7-0.4 0.4 -1.0 0.6 -1.5 0.5-0.5 -0.1 -0.9 0.0 -1.0 0.3-0.3 0.8 -3.7 0.1 -5.0 -0.9-0.3 -0.2 -2.6 -0.4 -5.9 -0.4-6.6 -0.1 -8.6 -0.7 -8.8 -2.9-0.1 -1.4 1.4 -4.0 2.3 -4.01.7 -0.0 2.0 -2.1 0.4 -2.8-0.4 -0.2 -0.7 -0.6 -0.8 -0.9-0.1 -0.4 -0.4 -0.2 -1.0 0.7zM148.9 174.3 c-0.4 1.7 -0.3 3.9 0.2 3.30.5 -0.5 0.7 -1.9 0.4 -2.7-0.1 -0.3 -0.2 -0.7 -0.3 -1.0-0.1 -0.3 -0.2 -0.2 -0.4 0.3zM147.5 179.1 c-0.5 0.5 -1.4 0.9 -1.8 0.9 l-0.9 0.00.8 0.6 c1.0 0.9 0.7 1.5 -1.1 2.0-1.6 0.4 -3.6 1.8 -3.6 2.60.0 0.4 0.2 0.4 1.0 -0.03.1 -1.6 7.0 -0.9 8.6 1.70.8 1.2 1.0 1.4 1.3 1.00.6 -0.7 -1.2 -3.8 -2.9 -5.0-1.5 -1.0 -1.9 -2.3 -0.9 -2.80.3 -0.2 0.5 -0.6 0.4 -1.1 l-0.1 -0.8-1.0 0.9zM134.8 182.2 c-0.7 1.6 -0.2 2.2 2.6 2.82.9 0.5 2.8 0.3 -1.2 -3.2 l-0.8 -0.8-0.6 1.2zM181.5 181.8 c-0.3 0.2 -0.9 1.5 -1.4 2.8-0.4 1.3 -1.2 2.9 -1.7 3.6-0.9 1.4 -1.1 2.2 -0.4 2.20.8 -0.0 1.4 -0.9 2.0 -2.61.0 -2.9 1.2 -3.1 2.1 -3.51.1 -0.4 2.0 -2.2 1.3 -2.7-0.6 -0.4 -1.4 -0.3 -2.1 0.2zM123.8 184.4 c0.3 0.9 1.4 1.9 2.2 1.90.3 -0.0 0.8 0.3 0.9 0.60.2 0.4 0.9 0.7 1.8 0.82.9 0.4 5.3 3.8 2.6 3.6-0.6 -0.0 -3.5 -0.2 -6.3 -0.4-6.3 -0.4 -6.7 -0.3 -6.6 1.1 l0.1 1.04.8 -0.0 c3.6 -0.0 5.3 -0.2 6.8 -0.61.2 -0.4 2.3 -0.5 2.9 -0.41.1 0.2 1.5 -1.0 0.5 -1.4-0.3 -0.1 -0.5 -0.4 -0.5 -0.60.0 -0.7 -2.1 -2.7 -3.3 -3.4-0.6 -0.3 -1.5 -0.8 -1.8 -1.1-0.4 -0.3 -1.5 -0.9 -2.4 -1.3 l-1.8 -0.70.2 0.7zM177.0 192.1 c0.0 0.3 -0.3 0.8 -0.6 1.0-0.9 0.7 -1.6 2.3 -0.8 2.20.8 -0.1 1.8 -2.1 1.7 -3.0-0.1 -0.5 -0.2 -0.5 -0.2 -0.2zM195.9 206.0 c-1.0 0.6 -4.6 6.3 -4.6 7.30.0 0.2 -0.4 0.7 -0.8 1.1-0.6 0.6 -0.8 1.2 -0.8 2.20.0 0.7 -0.1 1.3 -0.3 1.3-0.2 -0.0 -0.3 0.2 -0.3 0.50.0 0.3 -0.3 0.5 -0.6 0.5-0.6 -0.0 -3.4 2.5 -3.4 3.00.0 0.1 -0.5 0.9 -1.2 1.6-0.7 0.9 -1.2 1.8 -1.2 2.30.0 0.7 -0.2 1.0 -0.9 1.1-1.3 0.3 -3.8 2.9 -3.8 3.90.0 0.2 -0.2 0.4 -0.4 0.4-0.3 0.1 -0.3 0.4 -0.1 1.0 l0.4 0.80.5 -1.1 c0.3 -0.6 1.0 -1.4 1.6 -1.80.5 -0.4 1.0 -0.9 1.0 -1.20.0 -0.3 0.5 -0.7 1.1 -1.01.2 -0.6 3.9 -3.3 3.9 -3.90.0 -0.2 0.3 -0.8 0.8 -1.30.4 -0.5 1.1 -1.4 1.6 -2.00.5 -0.6 1.4 -1.7 2.1 -2.40.7 -0.7 1.2 -1.5 1.2 -1.70.0 -0.2 0.3 -0.6 0.7 -0.90.5 -0.4 0.7 -0.9 0.7 -2.00.0 -0.8 0.2 -1.5 0.4 -1.60.8 -0.5 1.9 -2.7 1.9 -3.90.0 -0.7 0.1 -1.3 0.3 -1.40.3 -0.2 1.1 -1.5 0.9 -1.5-0.1 -0.0 -0.4 0.2 -0.7 0.4zM111.7 206.6 c0.0 0.1 0.4 0.7 0.9 1.2 l0.8 0.90.7 -0.8 c0.5 -0.6 0.6 -0.8 0.2 -1.0-0.8 -0.4 -2.6 -0.7 -2.6 -0.4zM210.7 207.6 c0.0 0.3 0.6 0.9 1.4 1.10.7 0.3 0.7 -0.2 0.0 -0.9-0.6 -0.6 -1.5 -0.7 -1.5 -0.2z","role":"ink"}]};
  const VIPER_SPINE = [
    [116, 186], [104, 166], [116, 144], [146, 134],
    [182, 142], [208, 174], [222, 198], [238, 152], [260, 188], [266, 232]
  ];

  function mountYuck(bubbleEl, overrides) {
    const Rig = root.BestiaryTracedRig;
    if (!bubbleEl || !Rig) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      wash: '#ff2e12',
      wash2: '#ffb800',
      scale: narrow ? 0.80 : 0.92,
      dur: 11.5,
      autoPause: true
    }, overrides || {});

    const rect = bubbleEl.getBoundingClientRect ? bubbleEl.getBoundingClientRect() : { width: 136, height: 58 };
    const w = Math.max(90, Math.round(rect.width || 136));
    const h = Math.max(44, Math.round(rect.height || 58));
    const sc = opts.scale;
    const r = Math.min(18, w * 0.44, h * 0.44);
    const pad = Math.round(28 * sc);
    const svgW = Math.ceil(w + pad * 2);
    const svgH = Math.ceil(h + pad * 2);
    const durS = `${Rig.f1(opts.dur)}s`;

    const svg = Rig.el('svg', {
      xmlns: Rig.SVG_NS,
      viewBox: `0 0 ${svgW} ${svgH}`,
      width: svgW,
      height: svgH,
      class: 'bestiary-yuck-coil bestiary-rig bestiary-traced-rig',
      'data-rig': 'yuck-traced',
      'data-dur': Rig.f1(opts.dur),
      style: `position:absolute;left:${-pad}px;top:${-pad}px;width:${svgW}px;height:${svgH}px;pointer-events:none;overflow:visible;`
    });
    svg.style.setProperty('--wash', opts.wash);
    svg.style.setProperty('--wash2', opts.wash2);

    const defs = Rig.el('defs', {}, svg);
    const x0 = pad, y0 = pad, x1 = pad + w, y1 = pad + h;
    const loopD = [
      `M${Rig.f1(x0 + r)},${Rig.f1(y0)}`,
      `L${Rig.f1(x1 - r)},${Rig.f1(y0)}`,
      `A${Rig.f1(r)},${Rig.f1(r)} 0 0 1 ${Rig.f1(x1)},${Rig.f1(y0 + r)}`,
      `L${Rig.f1(x1)},${Rig.f1(y1 - r)}`,
      `A${Rig.f1(r)},${Rig.f1(r)} 0 0 1 ${Rig.f1(x1 - r)},${Rig.f1(y1)}`,
      `L${Rig.f1(x0 + r)},${Rig.f1(y1)}`,
      `A${Rig.f1(r)},${Rig.f1(r)} 0 0 1 ${Rig.f1(x0)},${Rig.f1(y1 - r)}`,
      `L${Rig.f1(x0)},${Rig.f1(y0 + r)}`,
      `A${Rig.f1(r)},${Rig.f1(r)} 0 0 1 ${Rig.f1(x0 + r)},${Rig.f1(y0)} Z`
    ].join(' ');
    const loopId = Rig.uid('yuck-coil-spine');
    Rig.el('path', { id: loopId, d: loopD, fill: 'none' }, defs);

    const perim = 2 * Math.max(1, w - 2 * r) + 2 * Math.max(1, h - 2 * r) + 2 * Math.PI * r;
    // Smooth, gentle coiling glide round the bubble with a curious pause at top-right (zero bounce!)
    const kt = '0;.28;.44;.72;.86;1';
    const ks = '.4 0 .6 1;0 0 1 1;.4 0 .6 1;0 0 1 1;.4 0 .6 1';
    const headU = [0.16, 0.42, 0.42, 0.84, 0.84, 0.99];

    const colA = `var(--wash, ${opts.wash})`;
    const colB = `var(--wash2, ${opts.wash2})`;

    const rig = Rig.mountSegmentedBody({
      defs, parent: svg,
      bodyLayers: YUCK_ART.body,
      srcSpine: VIPER_SPINE,
      srcWidth: 32,
      numSegments: 14,
      overlap: 0.62,
      targetPathId: loopId,
      targetPathLen: perim,
      bodyScale: 0.22 * sc,
      bodyLenPx: Math.min(perim * 0.42, 62 * sc),
      keyTimes: kt, keySplines: ks, headUValues: headU,
      dur: opts.dur, wash: colA, wash2: colB,
      className: 'yuck-viper-coil-body'
    });

    // Cute traced viper head with inquisitive tilt, gentle jaw snap, and blink
    const headNode = Rig.el('g', {});
    headNode.appendChild(root.BestiarySlots.buildTracedRoundelHead({ wash: colA, wash2: colB, scale: 0.304*sc, dur: opts.dur }));

    Rig.mountJointPart({
      parent: svg, targetPathId: loopId,
      spanFrac: rig.spanFrac, jointFrac: 0.0,
      keyTimes: kt, keySplines: ks, headUValues: headU,
      dur: opts.dur, node: headNode,
      localRotate: { values: '0;0;-14;12;-8;0;0;-10;8;0', keyTimes: '0;.28;.33;.38;.42;.46;.74;.79;.84;1', dur: opts.dur }
    });

    if (typeof getComputedStyle === 'function' && getComputedStyle(bubbleEl).position === 'static') {
      bubbleEl.style.position = 'relative';
    }
    bubbleEl.appendChild(svg);
    Rig.observeVisibility(svg, opts.autoPause);
    return svg;
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.mountYuck = mountYuck;
}(typeof globalThis !== 'undefined' ? globalThis : this));


(() => {
  const go = () => {
    const S = window.BestiarySlots; if (!S) return;
    const bubbleOf = sp => sp.closest('.b') || sp.parentElement;
    let k = 0;
    document.querySelectorAll('span.beast-knotted-pair').forEach(sp => {
      const b = bubbleOf(sp); sp.classList.add('b6-off'); b.classList.add('b6'); const pi = k++ % 3; S.mountKnottedPair(b, pi, { scale: pi === 0 ? (innerWidth <= 420 ? 0.72 : 0.85) : (innerWidth <= 420 ? 0.9 : 1.15) });
    });
    const sw = document.querySelector('span.beast-draco-alatus.swarm');
    if (sw) { const b = bubbleOf(sw); b.classList.add('b6'); S.mountSwarm(b); }
    const y = [...document.querySelectorAll('.b.dzb')].find(b => b.textContent.trim() === 'yuck');
    if (y) { y.classList.add('b6'); y.querySelectorAll(':scope > svg.vine').forEach(v => v.classList.add('b6-off')); S.mountYuck(y);
      new MutationObserver(() => y.querySelectorAll(':scope > svg.vine').forEach(v => v.classList.add('b6-off'))).observe(y, { childList: true }); }
    document.querySelectorAll('.b.b6').forEach(b => {
      const m = b.closest('.m'); m.style.position = 'relative'; m.style.zIndex = '6';
      const rig = b.querySelector('svg.bestiary-traced-rig'); if (!rig) return;
      if (b.querySelector('.b6-off.beast-knotted-pair')) {   // tuck the pair just under its bubble
        if (rig.querySelector('animateTransform[type=rotate][values^="0 0 24"]')) { m.style.marginBottom = '96px'; return; }
        rig.style.translate = '0 -14px';
      }
      const under = rig.getBoundingClientRect().bottom - b.getBoundingClientRect().bottom;
      m.style.marginBottom = Math.max(0, Math.round(under) - 18) + 'px';
    });
  };
  (document.fonts ? document.fonts.ready : Promise.resolve()).then(() => setTimeout(go, 400));
})();

