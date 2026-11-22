"""Optional medium-family plugin loader (loaded lazily by lib.presets)."""
from __future__ import annotations

import importlib

for _mod in ("illuminated", "prints_heraldry"):
    try:
        importlib.import_module(f".{_mod}", __name__)
    except ModuleNotFoundError as exc:
        if exc.name != f"{__name__}.{_mod}":
            raise
