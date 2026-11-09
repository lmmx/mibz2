import json,hashlib,re,bz2,pathlib
d=json.load(open("d050/input.json"))
sha=lambda s:hashlib.sha256(s.encode()).hexdigest()
p=d["parallel"];t=d["tests"];mut=p.replace("for b in held.into_iter().flatten() {","for b in held.into_iter().flatten().take(0) {")
assert p.count("for b in held.into_iter().flatten() {")==1
assert sha(p).startswith("60236bf519d08a62")
assert sha(t).startswith("bb74157037d816bf")
assert sha(mut).startswith("0ff4914e59050036")
assert d["patch"]==d["oldpatch"]
a=t.split("const TINY: [u8; 539] = [",1)[1].split("];",1)[0]
z=bytes(int(x,16) for x in re.findall(r"0x([0-9a-f]{2})",a))
want=b"".join(bytes(b"abcdefghij"[(j*7+i*3+j//11+j*j//97)%10] for j in range(240)) for i in range(4))
assert bz2.decompress(z)==want and len(z)==539
for name,pattern in [("baseline.log","8 passed; 0 failed"),("miri.log","3 passed; 0 failed"),("m12-mutant.log","1 passed; 7 failed"),("m12-mutant-miri.log","0 passed; 3 failed"),("restored.log","8 passed; 0 failed")]:assert pattern in d["logs"][name]
assert 'places [0, 1, 3]' in d["logs"]["m12-mutant.log"]
ns={"__name__":"import_only"};exec(d["coord"],ns)
ns["comments"]=lambda n:d["channels"][str(n)]
audit=ns["audit"](ns["CUTOVER"])
assert all(x["resolved"] for x in audit)
out={"parallel_sha256":sha(p),"tests_sha256":sha(t),"mutant_sha256":sha(mut),"patch_byte_identical":True,"tiny_bz2_bytes":len(z),"tiny_decoded_bytes":len(want),"log_expectations_pass":True,"routing":ns["route"]("coordinator","REVIEW"),"audit":audit,"scope":"Python hash/mutation reconstruction, fixture decompression, log consistency and routing only; no Rust execution"}
pathlib.Path("d050/results.json").write_text(json.dumps(out,indent=2)+"\n")
print(json.dumps(out,indent=2))
