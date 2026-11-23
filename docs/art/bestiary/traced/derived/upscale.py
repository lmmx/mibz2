#!/usr/bin/env python3
"""Upscale a small reference with Lanczos so fine detail layers have pixels to trace.

    upscale.py SRC OUT.png FACTOR
"""
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("RGB")
f = float(sys.argv[3])
im.resize((round(im.width * f), round(im.height * f)), Image.Resampling.LANCZOS).save(sys.argv[2])
