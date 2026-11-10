import copy,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import team_screen as t
import test_team_screen as f
m,d=f.manifest(),f.delta()
base=f.good_records(m,d)
out={'baseline':t.outputs_check(m,d,base)}
for name,change in [
 ('wrong_build_label',lambda r:r[0].update(build='unrelated')),
 ('different_executable_each_fixture',lambda r:[w.update(exe_sha256=f'{i:064x}') for i,w in enumerate(r)]),
 ('valid_inputs_report_error_kind',lambda r:[w['record'].update(err_kind='BlockCrc',err_block=99) for w in r if not w['record']['err']]),
 ('error_exit_124',lambda r:[w.update(exit_status=124) for w in r if w['record']['err']]),
]:
 r=copy.deepcopy(base);change(r);out[name]=t.outputs_check(m,d,r)
assert all(v==[] for v in out.values()),out
print(json.dumps(out,indent=2))
