/**
 * Slot 5: Swarm (gold, by the "agentic development" line)
 * Small vipers with redrawn knackering-style bodies and traced viper heads
 * orbiting and darting round the bubble in neon gold (--wash / --wash2).
 */
(function (root) {
  'use strict';

  function mountSwarm(bubbleEl, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      count: 3,
      wash: '#ffb800',
      wash2: '#ff6a00',
      scale: narrow ? 0.72 : 0.88,
      dur: 10.5,
      autoPause: true
    }, overrides || {});
    return root.BestiaryCrawler.makeFlightLoop(bubbleEl, opts);
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.mountSwarm = mountSwarm;
}(typeof globalThis !== 'undefined' ? globalThis : this));
