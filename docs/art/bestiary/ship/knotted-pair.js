/**
 * Slot 3: Knotted pair (three places)
 * Two knackering-style bodies with Gothic interlace dragon heads.
 * The over-under knot tightens and loosens, then slithers along the
 * bubble's lower edge. Recolourable via --wash and --wash2.
 */
(function (root) {
  'use strict';

  const KNOTTED_PRESETS = [
    { wash: '#1b5cff', wash2: '#00d97e', dur: 13.0 }, // cobalt + emerald
    { wash: '#ff5a1f', wash2: '#5b2bff', dur: 13.8 }, // neon orange + indigo
    { wash: '#00c8d7', wash2: '#ffb800', dur: 12.6 }  // neon cyan + gold
  ];

  function mountKnottedPair(bubbleEl, presetIndex, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
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
      autoPause: true
    }, overrides || {});
    return root.BestiaryCrawler.makeKnottedSlider(bubbleEl, opts);
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.KNOTTED_PRESETS = KNOTTED_PRESETS;
  root.BestiarySlots.mountKnottedPair = mountKnottedPair;
}(typeof globalThis !== 'undefined' ? globalThis : this));
