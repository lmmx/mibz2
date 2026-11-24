---
name: link-and-gloss
description: Use on every blog post before it ships, and on any published post a reader found hard to enter. It fixes the blog's recurring in-medias-res failure in two ways - linking each term, result and earlier piece of work to the post that explains it, at first use, and adding fleuron glosses where something needs explaining as an aside without stopping the prose.
---

# Link and gloss: no reader starts in the middle

Our posts keep dropping the reader into the middle of things. A post leans on the walk, Cells,
Open, G1, a census or a lanes experiment as if the reader had followed the project for a month.
Fresh-reader passes find it every time (the "Halving the single-row steps" pass, 24 November
2026, listed over thirty terms used before or without explanation). Louis, the same day: "I think
this piece is under-linked and this underlies a general failure mode of in medias res that we have
repeatedly come up against."

There are two remedies, and a post usually needs both:

1. **Link to what already explains it.** Most of what a post assumes was explained once, in an
   earlier post. The fix is a link at first use, not a new explanation.
2. **Gloss what needs explaining but would derail the prose.** Where the reader needs a few
   sentences of context that the argument doesn't, put them in a fleuron gloss beside the line,
   not in the paragraph.

This skill sits between write-blog-post (the text) and publish-page (the render). Its markup is
write-blog-post's "Ornament" section; this skill decides where that markup goes.

## 1. Run the audit

```sh
python3 docs/scripts/link_audit.py docs/site/blog/<post>.md
```

It prints two worklists:
- **Used before it is explained:** terms the post defines in bold or in a heading, but uses
  earlier. Each needs a forward pointer, a link, a gloss at first use, or the definition moved up.
- **Explained in another post, not linked at first use here:** terms that other posts define (in
  bold, or as a title), with the posts that explain them.

The script finds candidates, and it misses things: internal names used in passing, results quoted
without a source, numbers without a referent. So after it, **read the post as a stranger** and add
to the list every place where you'd ask "what's that?", "says who?" or "which one?".

## 2. Decide, for each item: link, gloss, inline, or cut

Take each item in order of first appearance, and choose one of four:

| The reader needs… | Do this | Example |
|---|---|---|
| a term an earlier post explains, and the argument only needs its name | **link at first use** to that post | "Open ([Open Returns](2026-11-19-open-returns.md))" |
| a term no post explains, in one clause | **define it inline**, in the sentence that uses it | "a valid column, one that is the BWT of some text" |
| context the argument doesn't use: history, an analogy's source, what a named project is, why a number is what it is | **a gloss** beside the line | the Mario 64 A-button challenge; batched backward search |
| something the post neither needs nor can explain briefly | **cut it** | an internal shorthand from a journal |

Rules for choosing:
- **Link before you explain.** If an earlier post explains a term, link it, even if you also gloss
  or define it. The link is where a curious reader goes; a gloss is what a passing reader needs.
- **The first use gets the link,** not the most convenient one. If a term first appears in a
  glossary or the opening list, the link goes there.
- **One link per term per post,** at first use, unless the term returns after a long gap in a
  section a reader might jump to directly.
- **Link the post that explains it best,** not the newest one that mentions it. When two posts
  share a term, prefer the one whose title is about it.
- **Results and numbers need a source too.** A claim such as "lanes bought 5–14%" links to the
  post or record that measured it.
- **Internal names stay out of a gloss's term** unless the post uses them. Gloss the idea, and
  name it the way the post does.

## 3. Writing a gloss

A gloss is a fleuron aside: a short paragraph in a second voice, beside the line it serves.

```markdown
<aside class="il-gloss" markdown>**Backward search** is how a compressed index finds a pattern:
it reads the pattern from its last letter to its first, narrowing an interval of rows with one
LF step per letter. Run for many patterns at once, it moves their intervals together.</aside>
```

- **It belongs to one word or one moment.** It goes straight after the paragraph containing the
  term, and its bold opening names that term as the post does. If you can't say which word it
  serves, it doesn't belong.
- **It is skippable.** The paragraph before it must read correctly without it. If the argument
  needs the content, it goes in the prose instead.
- **Two to five sentences.** A longer gloss is a section, or a link to the post that explains it.
- **The editor's voice may be a little old-fashioned** ("Herein…", "the reader may take…"), but
  every sentence states something plain and true. No metaphor stands in for the fact, and every
  fact in a gloss has a source, like any other.
- **Link inside the gloss** to the post or source that explains more: `**[Open](…)**` keeps the
  gloss's colour.
- **The page needs the stylesheet.** Glosses render from `illuminated.css`. It loads site-wide,
  but check the post in both themes and at phone width, since a gloss beside a figure can crowd it.
- **No gloss on every term.** Two to six glosses suit a long post. Too many, and the asides become
  the text.

## 4. Check

- [ ] `link_audit.py` has been run, and every line is resolved: linked, glossed, defined, or
      knowingly left (a false match).
- [ ] Every term in the opening, the glossary and the dek is linked or defined where it first
      appears.
- [ ] Every project name, kernel, theorem label, census and experiment links to the post or record
      that explains it, at first use.
- [ ] Every quoted number links to where it was measured.
- [ ] Each gloss can be pointed at: which word, and why there.
- [ ] The prose reads correctly with every gloss skipped.
- [ ] The strict build, `check_render.py`, `check_site.py` and `look_pages.py` pass (publish-page).

## 5. For a published post

The fix ships as a docs commit to master (coordinate-blog-post §4). If other commits to master are
deploying, wait until the page serves them: each push cancels the Pages deploy in flight.
