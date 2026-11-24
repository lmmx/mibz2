# Diagrams in an outside model's response: describe them, don't commit them

A response from ChatGPT, or any model outside the swarm, often comes with images: rendered
diagrams, charts, interactive widgets captured as screenshots. They are drafts of figures, not
records, so they are handled as follows (Louis, 24 November 2026).

- **Not committed.** Screenshots of a response's figures don't go in the repository. They are
  large (a typical phone screenshot is 100–700 KB), they don't follow the site's light and dark
  schemes, and any figure we publish is redrawn anyway. A figure small enough to store as text
  (an SVG, a Mermaid or Graphviz source, a data table) may be kept verbatim beside the response
  instead of a description.
- **Described, well enough to redraw.** The verbatim record of the response gets a section of
  figure descriptions, one entry per image, in the order they appeared. Each entry says:
  - where it sat in the response (the sentence before and after it);
  - its title and caption, verbatim;
  - what it shows: layout, every label and value, colours where they carry meaning, and for an
    interactive figure its controls and states;
  - errors found in it: a value that contradicts the text or the mathematics, a mislabelled
    element. These get fixed when the figure is redrawn, not reproduced.
  - The original image's file name, size and the start of its SHA-256, so the copy that the
    person who pasted it keeps can be matched to its description.
- **A starting point, not a spec.** When a figure is redrawn for the site, it follows
  write-blog-post's figure rules (inline SVG in the blog's figure style). The description is
  what the outside model proposed; the redraw should improve on it wherever it can. That
  includes correcting it, simplifying it, or replacing it with a figure that makes the point
  better.
- **Credit.** A redrawn figure that keeps an outside model's idea credits it the way the prose
  does, in the post's Compendium or a gloss.

The first record made this way is §5 of
`docs/journal/2026-11-24-against-mathematics-source.md`.
