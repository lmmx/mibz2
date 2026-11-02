"""bench/asm_listing.py's G3 comparison on recorded listings (no compiler, no timing).
Run: python3 bench/test_asm_listing.py

The i9's second attempt (report 182) is the regression: its room::<false> is the reviewed
function, instruction for instruction, followed by one multi-byte `nop` of linker fill where
the reference has six `int3`. The old boundary (the next symbol) compared the fill and
failed G3; the function's own extent (its symbol size) passes it. Every other test changes
something real and must fail: an instruction of the 21-instruction ordinary path, a branch
target, the last call, one of the function's own alignment NOPs, the size, or the fill.

The i9 binary's symbol size was not archived. Its fill starts at 0xcbd0 + 0x20aa = 0xec7a,
the reviewed size from the function's start, as on the VM's build; on the workstation
asm_listing.py reads the size from the binary's own symbol table."""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(__file__))
import asm_listing as g3  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
JOURNAL = os.path.join(HERE, "..", "docs", "journal")
REFERENCE = open(os.path.join(JOURNAL, "2026-11-02-compact30-walker", "room-false.asm")).read()
I9 = os.path.join(JOURNAL, "2026-11-02-compact30-traversal-protocol", "i9-attempt-2")
I9_ASM = open(os.path.join(I9, "room-false.asm")).read()
I9_START = 0xCBD0
SIZE = g3.REFERENCE_SIZE

# the ordinary step, as offsets from the function's start (the walker journal's 21
# instructions: 13 at 15f30-15f59 and 8 at 15f5b-15f78 in the reviewed binary, which
# starts at 14ed0)
ORDINARY = [
    0x1060, 0x1062, 0x1065, 0x1067, 0x106A, 0x106C, 0x1071, 0x1076, 0x107A, 0x107E, 0x1082,
    0x1085, 0x1089, 0x108B, 0x108F, 0x1092, 0x1097, 0x109A, 0x109D, 0x10A2, 0x10A8,
]  # fmt: skip


def replace(text, address, new):
    """The listing with the instruction at `address` replaced by `new`."""
    pat = re.compile(r"^( *%x:\t).*$" % address, re.M)
    out, n = pat.subn(lambda m: m.group(1) + new, text)
    assert n == 1, f"no instruction at 0x{address:x}"
    return out


def instruction(text, address):
    return next(t for a, t in g3.instructions(text) if a == address)


def passes(text, size=SIZE):
    diff, _ = g3.compare(text, size, REFERENCE)
    return not diff


def test_reference_passes_against_itself():
    diff, bounds = g3.compare(REFERENCE, SIZE, REFERENCE)
    assert diff == [], diff
    assert "1552 instructions compared" in bounds[0]
    assert bounds[1].endswith("not compared: int3 x6")


def test_i9_attempt_2_passes():
    diff, bounds = g3.compare(I9_ASM, SIZE, REFERENCE)
    assert diff == [], diff
    assert bounds[0] == "this binary: function 0xcbd0 + 0x20aa = 0xec7a, 1552 instructions compared"
    assert (
        bounds[1]
        == "this binary: after the function, not compared: nop WORD PTR [rax+rax*1+0x0] x1"
    )
    assert bounds[3] == "reference: after the function, not compared: int3 x6"


def test_i9_attempt_2_failed_at_the_next_symbol():
    # the old boundary: everything to the next symbol (0xec80 on the i9, 0x16f80 on the
    # reference, both 0x20b0 from the start) is the function; this is the archived diff
    diff, _ = g3.compare(I9_ASM, 0x20B0, REFERENCE, 0x20B0)
    archived = open(os.path.join(I9, "room-false.diff")).read().splitlines()
    assert diff == archived, diff


def test_i9_attempt_2_ordinary_path_is_the_reviewed_one():
    ref = [(0x14ED0 + o, instruction(REFERENCE, 0x14ED0 + o)) for o in ORDINARY]
    got = [(I9_START + o, instruction(I9_ASM, I9_START + o)) for o in ORDINARY]
    assert [t for _, t in ref[:3]] == ["mov    ecx,esi", "shr    ecx,0x8", "add    ecx,edi"]
    want, have = g3.normalise(ref, 0x14ED0, SIZE), g3.normalise(got, I9_START, SIZE)
    assert len(want) == 21 and want == have
    assert want[12] == "jae    T+4144" and want[20] == "jb     T+4192"


def test_each_ordinary_instruction_changed_fails():
    for o in ORDINARY:
        a = I9_START + o
        old = instruction(I9_ASM, a)
        for new in ("nop", "mov    ecx,edx" if old != "mov    ecx,edx" else "mov    ecx,esi"):
            assert not passes(replace(I9_ASM, a, new)), f"+0x{o:x}: {old!r} -> {new!r} passed"


def test_changed_branch_target_fails():
    # `jb dc30` closes the ordinary step; aim it two bytes later, still inside the function
    a = I9_START + 0x10A8
    old = instruction(I9_ASM, a)
    new = old.replace("dc30 <", "dc32 <").replace("+0x1060>", "+0x1062>")
    assert new != old
    assert not passes(replace(I9_ASM, a, new))


def test_changed_last_call_fails():
    # the last executable instruction before the fill
    old = instruction(I9_ASM, 0xEC75)
    assert old.startswith("call") and "_Unwind_Resume" in old
    assert not passes(replace(I9_ASM, 0xEC75, "call   5050 <abort@plt>"))
    assert not passes(replace(I9_ASM, 0xEC75, "nop    DWORD PTR [rax+rax*1+0x0]"))


def test_function_alignment_nops_are_compared():
    # the function's own loop alignment is part of it: changed, dropped or swapped, it fails
    a = I9_START + 0x7A
    assert instruction(I9_ASM, a) == "nop    WORD PTR [rax+rax*1+0x0]"
    assert not passes(replace(I9_ASM, a, "int3"))
    assert not passes(replace(I9_ASM, a, "nop    DWORD PTR [rax+rax*1+0x0]"))
    assert not passes(replace(I9_ASM, a, "xchg   ax,ax"))
    dropped = re.sub(r"^ *%x:\t.*\n" % a, "", I9_ASM, flags=re.M)
    assert dropped != I9_ASM and not passes(dropped)


def test_real_instruction_in_the_fill_fails():
    for new in ("ret", "jmp    cbd0 <x>", "push   rbp", "ud2", "int    0x3", "mov    rdi,rbx"):
        diff, _ = g3.compare(replace(I9_ASM, 0xEC7A, new), SIZE, REFERENCE)
        assert any("not alignment fill" in d for d in diff), (new, diff)


def test_elided_bytes_in_the_fill_fail():
    assert not passes(I9_ASM.rstrip("\n") + "\n\t...\n")


def test_unrecognised_line_inside_the_function_fails():
    text = I9_ASM.replace("    dc30:\t", "\t...\n    dc30:\t", 1)
    diff, _ = g3.compare(text, SIZE, REFERENCE)
    assert any("inside the function" in d for d in diff), diff


def test_other_sizes_fail():
    # the symbol table's size is the function: a different one fails, whatever it cuts
    for size in (0x20A5, 0x20A9, 0x20AB, 0x20B0, 0x2000):
        assert not passes(I9_ASM, size), hex(size)


def test_end_inside_an_instruction_fails():
    # with both sizes moved into the last call, no instruction starts at the end
    diff, _ = g3.compare(REFERENCE, 0x20A8, REFERENCE, 0x20A8)
    assert any("no instruction starts at the function's end" in d for d in diff), diff


def test_a_wrong_reference_size_fails():
    # the reference's tail is held to the same rule: a size that leaves the call outside
    # the function puts a real instruction in its "fill"
    diff, _ = g3.compare(I9_ASM, 0x20A5, REFERENCE, 0x20A5)
    assert any(d.startswith("reference: not alignment fill") for d in diff), diff


def test_fill_forms():
    ok = [
        "int3",
        "nop",
        "xchg   ax,ax",
        "nop    DWORD PTR [rax]",
        "nop    DWORD PTR [rax+0x0]",
        "nop    WORD PTR [rax+rax*1+0x0]",
        "cs nop WORD PTR [rax+rax*1+0x0]",
        "data16 cs nop WORD PTR [rax+rax*1+0x0]",
        "data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]",
    ]
    bad = ["ret", "int    0x3", "xchg   eax,eax", "nopx", "ud2", "lea    rsi,[rsi+0x0]", "..."]
    for t in ok:
        assert g3.FILL.match(" ".join(t.split())), t
    for t in bad:
        assert not g3.FILL.match(" ".join(t.split())), t


if __name__ == "__main__":
    n = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            n += 1
    print(f"{n} tests passed")
