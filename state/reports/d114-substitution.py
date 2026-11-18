import importlib.util,pathlib,tempfile,os,sys,subprocess,json,contextlib,io
from unittest.mock import patch
ROOT=pathlib.Path.cwd();source=(ROOT/'cells_owned.rs').read_text()
s=importlib.util.spec_from_file_location('mutator',ROOT/'d114-mutator.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def run(mode):
    calls=0
    def fake(args,**kw):
        nonlocal calls
        calls+=1
        baseline=calls<=3
        fail=mode=='baseline_build' or (not baseline and mode=='mutant_build')
        if '--no-run' in args:
            return subprocess.CompletedProcess(args,101 if fail else 0,'','error: dependency/build failure' if fail else '')
        if mode=='selection_build' and not baseline:
            return subprocess.CompletedProcess(args,101,'','error: dependency/build failure')
        if mode=='zero_tests':return subprocess.CompletedProcess(args,0,'running 0 tests\ntest result: ok.\n','')
        killed=not baseline and mode=='runtime'
        name='cells::tests::owned_writer_nested_copies'
        out=f"running 1 test\ntest {name} ... {'FAILED' if killed else 'ok'}\ntest result: {'FAILED' if killed else 'ok'}.\n"
        return subprocess.CompletedProcess(args,101 if killed else 0,out,'')
    with tempfile.TemporaryDirectory(dir=ROOT) as d:
        os.chdir(d);pathlib.Path('src').mkdir();pathlib.Path(m.SRC).write_text(source)
        with patch.object(sys,'argv',['mutate.py','receipts']),patch('subprocess.run',side_effect=fake),contextlib.redirect_stdout(io.StringIO()):code=m.main()
        receipt=json.load(open('receipts/receipts.json'));assert pathlib.Path(m.SRC).read_text()==source
        os.chdir(ROOT)
        result={'mode':mode,'exit':code,'calls':calls,'verdicts':[x['verdict'] for x in receipt['mutants']]}
        assert code==(0 if mode=='runtime' else 1),result
        assert all(x!='KILLED' for x in result['verdicts']) if mode!='runtime' else all(x=='KILLED' for x in result['verdicts'])
        return result
print(json.dumps([run(x) for x in ['baseline_build','mutant_build','selection_build','zero_tests','runtime']],indent=2))
