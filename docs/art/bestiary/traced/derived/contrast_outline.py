#!/usr/bin/env python3
"""Make a high-contrast copy of a faint pen drawing so monochrome_woodcut can threshold its ink.

    contrast_outline.py SRC OUT.png [lo hi [thicken_px]]

Divides out uneven vellum illumination, then stretches flat values lo..hi (default 0.70..0.93)
onto black..white. Linework only is changed; nothing is drawn or erased.
"""
import sys
import numpy as np
from PIL import Image
from scipy import ndimage

src, out = sys.argv[1], sys.argv[2]
lo, hi = (float(sys.argv[3]), float(sys.argv[4])) if len(sys.argv) > 4 else (0.70, 0.93)
g = np.asarray(Image.open(src).convert("L"), dtype=np.float32) / 255.0
flat = np.clip(g / np.maximum(ndimage.gaussian_filter(g, 25), 0.15), 0, 1)
v = np.clip((flat - lo) / (hi - lo), 0, 1)
t = int(sys.argv[5]) if len(sys.argv) > 5 else 0
if t:  # thicken dark strokes so broken sketch lines join into closed contours
    v = ndimage.minimum_filter(v, size=t)
Image.fromarray((v * 255).astype(np.uint8)).save(out)
