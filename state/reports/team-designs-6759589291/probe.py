import importlib.util,sys,types,tempfile,json
from pathlib import Path
from types import SimpleNamespace as N
sys.modules['team_gap_count']=types.ModuleType('team_gap_count')
spec=importlib.util.spec_from_file_location('screen',Path(__file__).with_name('team_screen.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
results={}
with tempfile.TemporaryDirectory() as d:
    p=Path(d)/'empty.json';p.write_text('{}')
    results['four_empty_records']=m.outputs(N(records=[str(p)]*4))
    results['one_record']=m.outputs(N(records=[str(p)]))
    p.write_text(json.dumps({'out':0,'hash':'bad','err':'same failure'}))
    results['four_matching_failures_on_unspecified_input']=m.outputs(N(records=[str(p)]*4))
# Satisfy A2 minimum-thread counts, but make those threads leaders with no share calls.
# Real profiler replaced here solely to test the checker's membership predicate.
profs=[N(thread=i,total=1,calls={('caller','coop::traverse_staged'):1}) for i in (1,2)]
m.tg.load=lambda _:profs;m.tg.disasm=lambda *a:{};m.tg.coop_lines=lambda _:{}
m.tg.stages=lambda p:({k:0 for k in ('MTF','build','walk','coordination')},None)
m.tg.walker_counts=lambda *a:{'staged':{'calls':1,'claims':2,'fragments':0,'flush_ir':0,'walked':{'take':1,'full':0,'tail':0}}}
results['walker_threads_with_no_helpers']=m.check(N(prefix='fake',binary='fake',coop_src='fake',require=['coop::traverse_staged=2'],forbid=[],coop_blocks=233,walked=2,table=None))
Path(__file__).with_name('probe-results.json').write_text(json.dumps(results,indent=2)+'\n')
print(results)
