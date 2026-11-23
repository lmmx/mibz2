# Bestiary Ship Handoff (Brief 6 — Traced Creatures, Rigged)

Every creature in this handover uses **potrace-traced medieval manuscript artwork** in the style of [`traced/romanesque-roundel-animated.svg`](../traced/romanesque-roundel-animated.svg), recolourable via `--wash` and `--wash2` (neon, saturated, zero pink), rigged onto `crawler.js` timelines with `IntersectionObserver` on-screen-only pausing.

---

## Shipped Slots

| Slot | File | Size (raw / gz) | Proof page | Target message | Exact call |
|---|---|---|---|---|---|
| **Shared rig** | `docs/art/bestiary/ship/traced-rig.js` | `8.5 KB` / `3.59 KB` | — | Overlapping potrace-traced body segment spine rig + `IntersectionObserver` on-screen-only pausing | `<script src="art/bestiary/ship/traced-rig.js"></script>` |
| **Headline frame heads** | `docs/art/bestiary/ship/headline-heads.js` | `11.0 KB` / `4.86 KB` | [`proof-headline-heads.html`](./proof-headline-heads.html) | *"managing Claudes all day is knackering"* (bodies unchanged; potrace-traced roundel horned heads with animated jaw & blink, gold caption uncovered) | `BestiarySlots.mountHeadlineHeads(headlineBubbleEl)` |
| **Knotted pair (×3)** | `docs/art/bestiary/ship/knotted-pair.js` | `46.8 KB` / `17.9 KB` | [`proof-knotted-pair.html`](./proof-knotted-pair.html) | *"this angle got a lot of attention"* + 2 more bubbles (clean cut-out of both potrace-traced Gothic interlace dragons, 18 + 16 overlapping segments coiling & sliding along lower edge) | `BestiarySlots.mountKnottedPair(bubbleEl, 0)` (presets `0`, `1`, `2`) |
| **Perched roundel** | `docs/art/bestiary/traced/romanesque-roundel-animated.svg` | `55.1 KB` / `20.5 KB` | `../proofs/proof-traced-roundel.html` | *"(i.e. you might be able to design a compression format…)"* (merged, no change) | `<object data="art/bestiary/traced/romanesque-roundel-animated.svg" type="image/svg+xml"></object>` |
