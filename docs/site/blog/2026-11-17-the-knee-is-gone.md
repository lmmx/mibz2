---
title: "The knee is gone: how mibz2 stopped losing speed to more threads"
image: assets/social/blog-2026-11-17-the-knee-is-gone.png
description: "Four releases that run fewer workers than threads have smoothed mibz2's slide past eight threads into a level plateau near 2 GB of output a second."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The knee is gone: how mibz2 stopped losing speed to more threads

Four releases that run fewer workers than threads have smoothed mibz2's slide past eight threads into a level plateau near 2 GB of output a second.
{: .post-dek }

17 November 2026 · 8 min read · [Research result](#5-what-still-holds-the-plateau-down)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Asking mibz2 for more threads no longer makes it slower. mibz2 is our bzip2 decoder, written
in Rust. A bzip2 file is a series of blocks that decode independently, so mibz2 decodes several
blocks at once, on as many threads as it is given. On 7 November,
[Why mibz2 stops scaling at eight threads](2026-11-07-why-mibz2-stops-scaling.md) reported
that on our test machine the decoder got faster up to eight threads and no faster after. That
point, where a scaling curve bends from rising to flat, is its *knee*. The post traced part of
the cause to the blocks in progress competing for the processor's shared cache.

By mibz2 0.5.0, released on 16 November, the knee had become a fall. On the test machine, a
ten-core Intel desktop (an i9-10900K, with 20 logical CPUs and a 20 MiB level-3 cache shared by
all ten cores), 0.5.0 took 16–18% longer to decode the same file with 12, 16 or 20 threads than
with 8. Four releases since, 0.5.1 to 0.5.4, have removed that fall. From 8 to 20 threads, 0.5.4
decodes between 1.95 and 2.06 GB of output a second, fastest at ten threads.

*Draft in progress: the rest of the post follows in further commits.*
