---
title: "Decompression is the harder problem: a conversation with Thomas Murphy"
image: assets/social/blog-2026-11-22-decompression-is-the-harder-problem.png
description: "Thomas Murphy of Tropik and I talk about agent swarms and message boards, and why a fast decompressor is a stranger problem than a proof."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Decompression is the harder problem: a conversation with Thomas Murphy

Thomas Murphy of Tropik and I talk about agent swarms and message boards, and why a fast decompressor is a stranger problem than a proof.
{: .post-dek }

22 November 2026 · 15 min read · Conversation
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

This post is a conversation: a Twitter DM between me and [Thomas Murphy](https://x.com/thomasmurphy__),
who organises [Tropik](https://tropik.space), a research collective, journal and podcast on
metaphysics, mathematics, poetics and software. It is published with his agreement, lightly
edited, and set as an illuminated manuscript.

## What it covers

- **Managing agents is its own kind of labour.** Thomas finds managing Claudes all day
  knackering, and more boring than writing code, but there is no going back.
- **A team becomes a swarm once it shares a message board.** I describe how mibz2's agents work:
  a coordinator session watches the repository, and the other agents talk through a pull request
  used as a message board, with addressed, structured messages and attached evidence.
- **Repositories carry work losslessly; chat compresses it.** Messages passed between agents
  come back summarised. Files on disk come back whole.
- **Real problems are not proof-shaped.** A proof like the Riemann hypothesis's is one long route
  to correctness. A fast decompressor has to take its data as it comes, on the computer it has, so
  its answer is always "it depends". That is why I think decompression is the harder problem, and
  why the Hutter Prize's framing, "compression is intelligence", looks backwards to me.
- **Compressing proofs could have computational effects.** If a proof about a decompressor were
  formalised and then compressed, the smaller proof might be easier to mutate and evolve.

## How to read it

The conversation below keeps the DM's layout: my messages on the right in blue, Thomas's on the left. Around
them:

- **Fleurons (❧ ☙)** mark terms with a link. Each carries a numbered footnote, and the notes at
  the end, the *Compendium of Links*, say in a short paragraph what each one is.
- **Notes hung beneath a message** explain a term in passing, such as the BEAM or Bondy.
- **Captions** in blackletter and gold mark a message's tone, and a few words were softened into
  archaic ones in blackletter.
- **Serpents and dragons** crawl through the text, after the medieval bestiaries. They are
  decoration, and they are still being redrawn.

The edits are light: a few words were changed for tone, and some messages after the end of the
conversation were cut.

## The conversation

<div class="convo-embed">
<iframe id="convo" src="conversations/2026-11-22-thomas-murphy.html" title="The conversation between Louis Maddox and Thomas Murphy, set as an illuminated manuscript" scrolling="no" loading="eager"></iframe>
</div>

<script>
(function () {
  var f = document.getElementById("convo");
  if (!f) return;
  function theme(doc) {
    var dark = document.body.getAttribute("data-md-color-scheme") === "slate";
    doc.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
  }
  function fit() {
    try { f.style.height = f.contentDocument.documentElement.scrollHeight + "px"; } catch (e) {}
  }
  f.addEventListener("load", function () {
    var doc;
    try { doc = f.contentDocument; } catch (e) { return; }
    if (!doc) return;
    doc.documentElement.style.overflow = "hidden";
    theme(doc);
    new MutationObserver(function () { theme(doc); })
      .observe(document.body, { attributes: true, attributeFilter: ["data-md-color-scheme"] });
    doc.addEventListener("click", function (ev) {
      var a = ev.target.closest && ev.target.closest("a[href]");
      if (!a) return;
      var h = a.getAttribute("href");
      if (h.charAt(0) === "#") {
        var el = doc.getElementById(h.slice(1));
        if (el) {
          ev.preventDefault();
          var y = f.getBoundingClientRect().top + el.getBoundingClientRect().top + window.scrollY - 80;
          window.scrollTo({ top: y, behavior: "smooth" });
        }
      } else {
        a.setAttribute("target", "_top");
      }
    });
    fit();
    if (window.ResizeObserver) new ResizeObserver(fit).observe(doc.body);
    setTimeout(fit, 1500);
  });
})();
</script>
