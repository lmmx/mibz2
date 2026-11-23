/**
 * Slot 1: Headline frame — "managing Claudes all day is knackering"
 * Two knackering-style serpents with traced Romanesque S-initial horned head (left)
 * and roundel head (right) chasing round the headline bubble, snapping and recoiling
 * at the bottom while leaving the gold caption below uncovered.
 */
(function (root) {
  'use strict';

  function mountHeadlineFrame(bubbleEl, overrides) {
    if (!bubbleEl || !root.BestiaryCrawler) return null;
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 420;
    const opts = Object.assign({
      wash: '#ff5a1f',
      wash2: '#ffb800',
      leftHead: 'romanesque-s-horned',
      rightHead: 'roundel-head',
      ruff: true,
      scale: narrow ? 0.72 : 0.86,
      dur: 11,
      autoPause: true
    }, overrides || {});
    return root.BestiaryCrawler.makeHeadlineChase(bubbleEl, opts);
  }

  root.BestiarySlots = root.BestiarySlots || {};
  root.BestiarySlots.mountHeadlineFrame = mountHeadlineFrame;
}(typeof globalThis !== 'undefined' ? globalThis : this));
