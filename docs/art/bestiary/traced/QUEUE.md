# Trace queue

References waiting to be traced, one at a time, with the bestiary pipeline (`PIPELINE.md`):
- write a spec in `specs/`, seeded with `seed_grid.py`;
- run it with `batch_gilded.py specs/<slug>.json` (gilded or painted) or `run_specimen.py` (other presets);
- check `<slug>-sheet.png` and `<slug>-components.png`;
- write the preview with `preview_specimen.py`.

Pencil and pen drawings give outlines, to be filled in. Painted ones give the illuminated detail.

| # | Reference (`refs/louis/`) | Slug | Kind | Status |
| --- | --- | --- | --- | --- |
| 4 | `4-amphisbaena-aberdeen.jpg` | `aberdeen-amphisbaena` | gilded, painted | traced (97.6% silhouette); heads lose their orange |
| 7 | `7-crescent-amphisbaena.jpg` | `crescent-amphisbaena` | painted on parchment | traced (98.3% silhouette, preset `parchment_pale_subject`, tight hand-drawn include polygon): heads now whole with eyes, ears and mouths and the mound is out; weak: faint green wash still leaks as a block behind the left head's ear wisps, belly fur is a detached brown slab, ink linework still chunky |
| 8 | `8-winged-amphisbaena-linear.jpg` | `winged-amphisbaena-linear` | painted on parchment | traced (98.2% silhouette, preset `parchment_pale_subject`, 7 parts): both heads, wing, torso, two legs and tan neck/spiral tail separate, text excluded; weak: right head's wash is mottled, the wing-over-tail overlap line is soft |
| 9 | `9-outline-amphisbaena-roundel.jpg` | `outline-amphisbaena-roundel` | pen outline, roundel | traced (95.0% silhouette, preset `monochrome_woodcut` on a contrast-stretched copy, `derived/contrast_outline.py`, because the raw pen line is too faint to threshold): clean ink outline of coil, both heads, wing and feet with the roundel rings cut off; weak: only wing, heads, torso and feet are filled, the coiled neck tube stays open line, one stray ring arc above the upper head, wing feathering is broken |
| 10 | `10-green-amphisbaena-roundel.jpg` | `green-amphisbaena-roundel` | painted roundel, gold centre | traced (97.2% silhouette, preset `gilded_romanesque`, hand-drawn include polygon hugging the coil to keep out the red field, blue ring, gold and text): 7 parts, both heads blue with ears and mouths, wing, torso and legs separate; weak: red field still leaks as a crimson sliver down the right edge, the pink/red inner bands and the gold sliver inside the coil are kept, ink linework is partial |
| 11 | `11-sketch-amphisbaena.jpg` | `sketch-amphisbaena` | ink sketch | waiting |
| 12 | `12-orange-amphisbaena-gilded.jpg` | `orange-amphisbaena-gilded` | gilded, painted roundel | waiting |
| 13 | `13-striped-amphisbaena-roundel.jpg` | `striped-amphisbaena-roundel` | painted roundel | waiting |
| 14 | `14-striped-amphisbaena-gilded.jpg` | `striped-amphisbaena-gilded` | gilded, painted, with tree and ground | waiting |
| 5 | `5-greek-winged-serpent.png` | `greek-winged-serpent` | painted on parchment (burrower shape) | waiting |
| 6 | `6-horned-serpent-woodcut.jpg` | `horned-serpent-woodcut` | woodcut (scale texture) | waiting |
