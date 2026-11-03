# c30walk::cbuild::c2_build
# size 0x15

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000009cbf0 <c30walk::cbuild::c2_build>:
   9cbf0:	push   %rax
   9cbf1:	mov    %r8,%rax
   9cbf4:	mov    %r9,(%rsp)
   9cbf8:	xor    %r8d,%r8d
   9cbfb:	mov    %rax,%r9
   9cbfe:	call   347f0 <<c30walk::ctor2::Ctor2>::build::<false>>
   9cc03:	pop    %rcx
   9cc04:	ret
