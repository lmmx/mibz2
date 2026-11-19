import importlib.util,pathlib,tempfile,os,sys,subprocess,json,contextlib,io
from unittest.mock import patch
ROOT=pathlib.Path.cwd();sources=json.load(open(ROOT/'d115-sources.json'))
s=importlib.util.spec_from_file_location('mutator',ROOT/'d115-mutator.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def run(mode):
    calls=0
    def fake(args,**kw):
        nonlocal calls
        calls+=1
        baseline=calls<=2
        fail=mode=='baseline_build' or (not baseline and mode=='mutant_build')
        if '--no-run' in args:
            return subprocess.CompletedProcess(args,101 if fail else 0,'','error: dependency/build failure' if fail else '')
        if mode=='selection_build' and not baseline:
            return subprocess.CompletedProcess(args,101,'','error: dependency/build failure')
        if mode=='zero_tests':return subprocess.CompletedProcess(args,0,'running 0 tests\ntest result: ok.\n','')
        killed=not baseline and mode=='runtime'
        name='open::b5::every_small_column_at_every_row'
        out=f"running 1 test\ntest {name} ... {'FAILED' if killed else 'ok'}\ntest result: {'FAILED' if killed else 'ok'}.\n"
        return subprocess.CompletedProcess(args,101 if killed else 0,out,'')
    with tempfile.TemporaryDirectory(dir=ROOT) as d:
        os.chdir(d)
        for p,s in sources.items():
            pathlib.Path(p).parent.mkdir(parents=True,exist_ok=True);pathlib.Path(p).write_text(s)
        with patch.object(sys,'argv',['mutate.py','receipts']),patch('subprocess.run',side_effect=fake),contextlib.redirect_stdout(io.StringIO()):code=m.main()
        receipt=json.load(open('receipts/receipts.json'));assert all(pathlib.Path(p).read_text()==s for p,s in sources.items())
        os.chdir(ROOT)
        result={'mode':mode,'exit':code,'calls':calls,'verdicts':[x['verdict'] for x in receipt['mutants']]}
        assert code==(0 if mode=='runtime' else 1),result
        assert all(x!='KILLED' for x in result['verdicts']) if mode!='runtime' else all(x=='KILLED' for x in result['verdicts'])
        return result
print(json.dumps([run(x) for x in ['baseline_build','mutant_build','selection_build','zero_tests','runtime']],indent=2))
