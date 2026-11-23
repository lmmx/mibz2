# Bestiary Ship Handoff (Brief 6 — Traced Creatures, Rigged)

Every creature in this handover uses **potrace-traced medieval manuscript artwork** in the style of [`traced/romanesque-roundel-animated.svg`](../traced/romanesque-roundel-animated.svg), recolourable via `--wash` and `--wash2` (neon, saturated, zero pink), rigged onto `crawler.js` timelines with `IntersectionObserver` on-screen-only pausing.

---

## Shipped Slots

| Slot | File | Size (raw / gz) | Proof page | Target message | Exact call |
|---|---|---|---|---|---|
| **Headline frame heads** | `docs/art/bestiary/ship/headline-heads.js` | `11.0 KB` / `4.86 KB` | [`proof-headline-heads.html`](./proof-headline-heads.html) | *"managing Claudes all day is knackering"* (bodies unchanged; potrace-traced roundel horned heads with animated jaw & blink, gold caption uncovered) | `BestiarySlots.mountHeadlineHeads(headlineBubbleEl)` |
| **Perched roundel** | `docs/art/bestiary/traced/romanesque-roundel-animated.svg` | `55.1 KB` / `20.5 KB` | `../proofs/proof-traced-roundel.html` | *"(i.e. you might be able to design a compression format…)"* (merged, no change) | `<object data="art/bestiary/traced/romanesque-roundel-animated.svg" type="image/svg+xml"></object>` |
