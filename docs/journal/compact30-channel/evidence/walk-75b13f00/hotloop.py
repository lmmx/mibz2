import re, subprocess, collections, sys

path, binary, pat = sys.argv[1], sys.argv[2], sys.argv[3]
f = open(path).read().splitlines()
cur = ""
cnt = collections.Counter()
names = {}
addr = 0
line_ = 0
skip = False


def upd(tok, prev):
    if tok == "*":
        return prev
    if tok.startswith("+"):
        return prev + int(tok[1:], 0)
    if tok.startswith("-"):
        return prev - int(tok[1:], 0)
    return int(tok, 0)


for l in f:
    m = re.match(r"^fn=\((\d+)\)(?: (.*))?$", l)
    if m:
        if m.group(2):
            names[m.group(1)] = m.group(2)
        cur = names.get(m.group(1), "")
        continue
    m2 = re.match(r"^cfn=\((\d+)\)(?: (.*))?$", l)
    if m2 and m2.group(2):
        names[m2.group(1)] = m2.group(2)
    if l.startswith("calls="):
        skip = True
        continue
    if re.match(r"^[a-z]+=", l):
        continue
    if not l or not (l[0] in "0123456789+-*"):
        continue
    parts = l.split()
    if len(parts) < 3:
        continue
    a = upd(parts[0], addr)
    ln = upd(parts[1], line_)
    addr, line_ = a, ln
    if skip:
        skip = False
        continue  # cost line of a call: inclusive, not self
    if pat in cur:
        cnt[a] += int(parts[2])
tot = sum(cnt.values())
hot = sorted(a for a, c in cnt.items() if c >= int(sys.argv[4]))
print("self", tot, "hot instrs", len(hot), "hot sum", sum(cnt[a] for a in hot))
lo, hi = min(hot), max(hot)
dis = subprocess.run(
    [
        "objdump",
        "-d",
        "--no-show-raw-insn",
        "-M",
        "intel",
        f"--start-address={lo}",
        f"--stop-address={hi + 16}",
        binary,
    ],
    capture_output=True,
    text=True,
).stdout
for l in dis.splitlines():
    m = re.match(r"\s*([0-9a-f]+):\s*(.*)", l)
    if m:
        a = int(m.group(1), 16)
        if cnt.get(a, 0) >= int(sys.argv[4]):
            print(f"{cnt[a]:>10,} {a:x} {m.group(2)}")
