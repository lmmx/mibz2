/**
 * Slot 6: "yuck"
 * One small, round, cute viper bouncing round the red blackletter "yuck"
 * bubble like a wind-up toy. Recolourable via --wash and --wash2.
 */
(function (root) {
  'use strict';

  function mountYuck(bubbleEl, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      wash: '#ff2e12',
      wash2: '#ffb800',
      scale: narrow ? 0.80 : 0.90,
      dur: 8.5,
      autoPause: true
    }, overrides || {});
    return root.BestiaryCrawler.makeYuckBounce(bubbleEl, opts);
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.mountYuck = mountYuck;
}(typeof globalThis !== 'undefined' ? globalThis : this));
