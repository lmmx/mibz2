/**
 * Slot 4: Moonwalker — border-drollery dragon
 * Traced head and beating wing mounted on a redrawn knackering-style body
 * and articulated legs, completing a full clockwise circuit of its bubble
 * via BestiaryCrawler.makeMoonwalker. Recolourable via --wash and --wash2.
 */
(function (root) {
  'use strict';

  function mountMoonwalker(bubbleEl, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      creature: 'draco-bipes',
      headVariant: 'border-drollery-head',
      wash: '#00c8d7',
      wash2: '#ff4518',
      scale: narrow ? 0.78 : 0.94,
      dur: 14,
      autoPause: true
    }, overrides || {});
    return root.BestiaryCrawler.makeMoonwalker(bubbleEl, opts);
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.mountMoonwalker = mountMoonwalker;
}(typeof globalThis !== 'undefined' ? globalThis : this));
