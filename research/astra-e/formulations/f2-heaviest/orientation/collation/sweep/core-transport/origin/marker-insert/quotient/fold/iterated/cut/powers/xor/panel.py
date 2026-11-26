"""Use the frozen stable constructor and an independent XOR selector for marked checks."""
import argparse
from contextlib import ExitStack
from pathlib import Path
import sys
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
import gate_panel
import solve
from chooser import select


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('arm',choices=('greedy','exact','gaps'))
    p.add_argument('tapes',type=Path);p.add_argument('corpus',type=Path);p.add_argument('scratch',type=Path)
    args=p.parse_args()
    with ExitStack() as patches:
        patches.enter_context(patch.object(solve,'choose',lambda edges,a:select(edges,a,args.arm=='exact')[0]))
        if args.arm=='gaps':
            import gap
            patches.enter_context(patch.object(solve,'split',gap.split))
            patches.enter_context(patch.object(solve,'check_step',gap.check))
        gate_panel.run(args)
