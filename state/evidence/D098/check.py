import hashlib, json, pathlib, re, difflib
root=pathlib.Path(__file__).parent
rows=[]; outcomes={}
for seed in (1,2):
    text=(root/f'fuzz-teams-seed{seed}.log').read_text()
    summary=re.search(r'^(\d+) cases in (\d+) s .*?, (\d+) failures',text)
    assert summary and int(summary[3])==0
    parsed=[]
    for line in text.splitlines():
        if 'records pooled' not in line or '; blocks' not in line: continue
        m=re.fullmatch(r'\s*(\d+)  (.*): records pooled (\d+), records alone (\d+), column pooled (\d+), column alone (\d+), errors (\d+); blocks (\d+) / (\d+) / (\d+) / (\d+); front errors (\d+) / (\d+)',line)
        assert m,line
        parsed.append({'seed':seed,'name':m[2],'cases':int(m[1]),'routes':[int(m[i]) for i in range(3,7)],'errors':int(m[7]),'blocks':[int(m[i]) for i in range(8,12)],'front_errors':[int(m[i]) for i in range(12,14)]})
    assert len(parsed)==5 and sum(x['cases'] for x in parsed)==int(summary[1])
    for n,kind in re.findall(r'^\s*(\d+)  teams: both fail \((\w+)\), as the plain decoder$',text,re.M):outcomes[kind]=outcomes.get(kind,0)+int(n)
    rows.extend(parsed)
result={'cases':sum(x['cases'] for x in rows),'errors':sum(x['errors'] for x in rows),'error_kinds':outcomes,'route_cases':[sum(x['routes'][i] for x in rows) for i in range(4)],'route_blocks':[sum(x['blocks'][i] for x in rows) for i in range(4)],'front_errors':[sum(x['front_errors'][i] for x in rows) for i in range(2)],'per_candidate':rows}
assert result['cases']==3755 and result['errors']==3273==sum(outcomes.values()) and len(outcomes)==18
assert result['route_cases']==[1759,1160,973,1078]
assert result['route_blocks']==[3932,2744,2611,4363]
assert result['front_errors']==[111,69]
result['patch_sha256']=hashlib.sha256((root/'patch.diff').read_bytes()).hexdigest()
result['patch_application']=json.loads((root/'source-check.json').read_text())
result['limits']='Re-tabulation and patch digest only. source-check.json records the separately reproduced patch-to-source comparison; no Rust/Miri/fuzz execution.'
(root/'recount.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
