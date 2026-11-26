from pathlib import Path
import sys
d=Path(__file__).resolve().parent
source=(d.parent/"count-only/price.py").read_text()
source=source.replace('fields.append("edits")', 'fields.extend(["edits","edit_start"])')
exec(compile(source,__file__,"exec"),{"__name__":"__main__","__file__":__file__})
