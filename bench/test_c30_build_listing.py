"""bench/c30_build_listing.py's normalised comparison, on synthetic listings.
Run: python3 bench/test_c30_build_listing.py"""

import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import c30_build_listing as L  # noqa: E402


def listing(base, body):
    """An objdump -d -C -l style listing at `base`: a label, source lines and instructions"""
    out = ["", "x:     file format elf64-x86-64", "", f"{base:016x} <<f>::g>:", "<f>::g:"]
    a = base
    for ins, n in body:
        out.append("/some/file.rs:12")
        out.append(f"  {a:x}:\t{ins.format(a=a, base=base)}")
        a += n
    return "\n".join(out) + "\n"


BODY = [
    ("push   %rax", 1),
    ("mov    0x10(%rsp),%rdi", 5),
    ("lea    -0x14f7e(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>", 7),
    ("jae    {t1:x} <<f>::g+0x10>", 6),
    ("call   2a370 <<c30walk::ctor::Ctor>::build::<false>>", 5),
    ("call   *0x95e61(%rip)        # b93b0 <memcpy@GLIBC_2.14>", 6),
    ("ret", 1),
]
SIZE = sum(n for _, n in BODY)


def make(base, body=BODY, callee=0x2A370):
    out = []
    for ins, n in body:
        ins = ins.replace("{t1:x}", f"{base + 0x10:x}").replace("2a370", f"{callee:x}")
        out.append((ins, n))
    return listing(base, out)


def test_identical_after_relocation():
    a, b = make(0x23410), make(0x6FD70, callee=0x88570)
    assert L.compare(a, SIZE, b, SIZE) == []


def test_changed_instruction_differs():
    body = list(BODY)
    body[1] = ("mov    0x18(%rsp),%rdi", 5)
    assert L.compare(make(0x1000, body), SIZE, make(0x2000), SIZE)


def test_changed_size_differs():
    assert L.compare(make(0x1000), SIZE + 1, make(0x2000), SIZE)


def test_internal_branch_target_differs():
    body = list(BODY)
    body[3] = ("jae    {t1:x} <<f>::g+0x10>", 6)
    a = make(0x1000, body).replace(f"{0x1010:x} <<f>::g+0x10>", f"{0x1012:x} <<f>::g+0x12>")
    assert L.compare(a, SIZE, make(0x2000), SIZE)


def test_other_callee_differs():
    body = list(BODY)
    body[4] = ("call   2a370 <<c30walk::ctor::W<false>>::promote>", 5)
    assert L.compare(make(0x1000, body), SIZE, make(0x2000), SIZE)


def test_missing_instruction_differs():
    body = BODY[:2] + BODY[3:]
    assert L.compare(make(0x1000, body), SIZE, make(0x2000), SIZE)


if __name__ == "__main__":
    n = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            n += 1
    print(f"{n} tests passed")
