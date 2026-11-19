import contextlib,io,math,shutil,statistics
from types import SimpleNamespace
import open_port3 as o3
import open_port as old
import test_open_port3 as t

def run(label, **kw):
    d=t.build3(**kw)
    try:
        with contextlib.redirect_stdout(buf:=io.StringIO()):
            ret = o3.judge(SimpleNamespace(bundle=str(d),port=t.PORT_SHA,part_a=str(d/'part-a.json')))
        print(label,'status',ret)
        for l in buf.getvalue().splitlines():
            if l.startswith(('VERDICT','  Holm','B4, sustained','  pooled:')) or 'ho-wikidata-l9 20t:' in l or 'ho-prose-l1 10t:' in l:
                print(l)
    finally:
        shutil.rmtree(d)

# Choose the final Holm-ranked H comparison to have a one-sided p in (.025,.05).
v=[-.02,.02]*6
mu=-1.95*statistics.stdev(v)/math.sqrt(12)
run('benefit CI crosses 1, last Holm test passes',step={(old.HELDOUT,20):math.exp(mu)},vectors={(old.HELDOUT,20):v})
run('one admitted guard +5% with uncertain local estimate',step={('ho-prose-l1',10):1.05},vectors={('ho-prose-l1',10):[-.075,.075]*8})
run('sustained +3% with uncertain local estimate',step={o3.B4:1.03},vectors={o3.B4:[-.06,.06]*4})
run('Part A empty summary',part_a={'voids':[],'triggers':[]})
