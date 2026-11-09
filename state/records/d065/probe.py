import os, pathlib, subprocess, tempfile, json
root=pathlib.Path(__file__).resolve().parent
recipe=(root/'recipe.sh').read_text()
# Sandbox lacks /dev/fd: replace only console tee plumbing, not recipe control flow.
recipe=recipe.replace('exec > >(tee -a "$W/console.log") 2>&1', 'exec >> "$W/console.log" 2>&1')
observed=[]
for outcome in ('PASS','FAIL','UNAVAILABLE'):
    with tempfile.TemporaryDirectory(dir=root) as tmp:
        d=pathlib.Path(tmp); (d/'bin').mkdir(); (d/'bench').mkdir(); (d/'target/release/examples').mkdir(parents=True)
        scripts={'git':'#!/bin/sh\ncase "$1" in rev-parse) echo c754acc8180a08b1c92cf21c411bc6504a41c7c2;; esac\n','cargo':'#!/bin/sh\nexit 0\n','lscpu':'#!/bin/sh\necho fixture\n'}
        for name,body in scripts.items():
            p=d/'bin'/name; p.write_text(body); p.chmod(0o755)
        ab=d/'target/release/examples/ab'
        ab.write_text('#!/bin/sh\ncase "$1" in telemetry-check) echo \'{"case":"injected","status":"'+outcome+'","detail":"fixture"}\'; exit '+('1' if outcome=='FAIL' else '0')+';; *) echo "{}";; esac\n'); ab.chmod(0o755)
        (d/'bench/bundle.sh').write_text('#!/bin/sh\ntouch "$1/bundle-called"\n')
        r=subprocess.run(['bash','-c',recipe],cwd=d,env={**os.environ,'PATH':str(d/'bin')+':'+os.environ['PATH']},capture_output=True,text=True,timeout=10)
        w=next((d/'target/telemetry-check').iterdir())
        observed.append({'case':outcome,'recipe_exit':r.returncode,'outcome':(w/'outcome.txt').read_text().strip(),'bundle_called':(w/'bundle-called').exists()})
print(json.dumps(observed,indent=2))
assert observed[1]['recipe_exit']==0 and observed[1]['outcome'].startswith('FAILED')
