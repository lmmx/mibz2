import json
import service_guarded as sg
from test_service_guarded import grid, OK

results = {}
def case(name, mutate, pf=OK):
    rows=grid(); mutate(rows)
    v=sg.judge(rows,pf)
    results[name]={'valid':v['valid'],'p1':v['p1'],'problems':v['problems']}
case('positive',lambda r:None)
case('nonzero_exit_empty_err',lambda r:r[0].update(exit=17))
case('boolean_round_zero',lambda r:r[0].update(round=False))
case('preflight_ok_without_evidence',lambda r:None, {'ok':True})
case('contradictory_preflight',lambda r:None, {'ok':True,'machine':{'quota':'Unknown','target':'None'},'at':[]})
case('unbound_base_path',lambda rs:[r.update(arm='mibz2@/unregistered/bin/ab-base') for r in rs if r['arm'].endswith('/ab-base')])
case('bad_output_positive_control',lambda r:r[0].update(hash='different'))
case('wrong_registered_size',lambda rs:[r.update(out=1) for r in rs])
case('nonhex_output_digest',lambda rs:[r.update(hash='not-a-digest') for r in rs])
print(json.dumps(results,indent=2))
