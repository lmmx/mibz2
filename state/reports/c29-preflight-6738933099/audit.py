"""Review-only checks against #288 at 0ce4dca4; no decoder execution.
Run from a checkout of that head with the downloaded preflight artifact directory:
  python audit.py --bench /path/to/bench --artifact /path/to/artifact
"""
import argparse, copy, json, re, sys
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument("--bench",required=True);p.add_argument("--artifact",required=True);a=p.parse_args()
sys.path.insert(0,a.bench)
import c29_identity as identity
import c29_checkpoints as reporter
d=Path(a.artifact);raw=d/"c29-preflight-raw"
before=json.loads((d/"c29-env.json").read_text());after=json.loads((d/"c29-env-after.json").read_text())
cg=before["cgroup"];member=(raw/"proc-self-cgroup.txt").read_text()
assert member==cg["membership"]["text"]
assert member.strip()==cg["membership_line"]
assert "cgroup:[4026531835]" in (raw/"proc-self-ns.txt").read_text()
mounts=[]
for line in (raw/"proc-self-mountinfo.txt").read_text().splitlines():
    f=line.split();s=f.index("-")
    decode=lambda x: re.sub(r"\\([0-7]{3})",lambda m:chr(int(m[1],8)),x)
    mounts.append((int(f[0]),decode(f[3]),decode(f[4]),f[s+1]))
matching=[m for m in mounts if m[0]==cg["mount"]["id"]]
assert len(matching)==1 and matching[0][1:]==("/","/sys/fs/cgroup","cgroup2")
leaf=member.strip().split(":",2)[2];expected=[]
while True:
    expected.append(leaf)
    if leaf=="/":break
    leaf=leaf.rsplit("/",1)[0] or "/"
assert [l["path"] for l in cg["levels"]]==expected
rawfiles=(raw/"levels-files.txt").read_text()
rawlevels=(raw/"levels.txt").read_text().splitlines()
for lv in cg["levels"]:
    target="/sys/fs/cgroup"+("" if lv["path"]=="/" else lv["path"])
    assert target==lv["dir"] and target in rawlevels
    parents=[m for m in mounts if target==m[2] or target.startswith(m[2].rstrip("/")+"/")]
    assert max(parents,key=lambda m:len(m[2]))[0]==cg["mount"]["id"]
    section=rawfiles.split("### "+target+"\n",1)[1].split("### ",1)[0]
    if lv["path"]=="/":
        assert not re.search(r"(?m)^== "+re.escape(target)+r"/(?:cpu.max|cgroup.type)$",section)
        assert not re.search(r"(?m)^-.* (?:cpu.max|cgroup.type)$",section)
    else:
        assert "== "+target+"/cpu.max\nmax 100000\n" in section
facts=identity.environment((d/"c29-env.json").read_text(),(d/"c29-env-after.json").read_text(),(d/"c29-nproc.txt").read_text())
assert reporter.environment_problems(facts,[],preflight=True)==[]
checks={"raw_membership_mount_namespace_ancestor_chain":"pass","artifact_preflight":facts["cpu_quota"],"stability":facts["cgroup_stable"]}
# Move an error through every level: the output must remain unknown even when the
# producer claims complete/unlimited. Preserve every other field from the real runner.
errors=[]
for i in range(3):
    x=copy.deepcopy(before);x["cgroup"]["levels"][i]["files"]["cpu.max"]={"error":"EACCES"}
    q=identity.coverage(x)
    assert q["state"]=="unknown" and q["coverage"]=="incomplete"
    errors.append({"level":i,"state":q["state"]})
checks["raw_error_at_each_level"]=errors
# Change only the raw+parsed finite limit consistently: exercise the exact boundary
# against real-run topology and ensure a smaller ancestor wins.
bounds=[]
for q,p0,want in [(399999,100000,False),(400000,100000,True),(1199999,300000,False),(1200000,300000,True)]:
    x=copy.deepcopy(before);lv=x["cgroup"]["levels"][1]
    lv.update(kind="limit",quota=q,period=p0)
    lv["files"]["cpu.max"]={"text":f"{q} {p0}\n"}
    x["cgroup"]["state"]="finite";x["cpu_quota_state"]="finite";x["cpu_quota"]=round(q/p0,2)
    result=identity.coverage(x);admit=not reporter.quota_problems("cpu_quota",result)
    assert admit==want
    bounds.append({"quota":q,"period":p0,"rounded":x["cpu_quota"],"admitted":admit})
checks["exact_boundary"]=bounds
print(json.dumps(checks,indent=2))
