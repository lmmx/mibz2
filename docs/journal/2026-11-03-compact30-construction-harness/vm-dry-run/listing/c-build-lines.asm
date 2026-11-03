
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000006fd70 <c30walk::cbuild::c_build>:
c30walk::cbuild::c_build:
./examples/c30walk/cbuild.rs:111
   6fd70:	push   %rax
   6fd71:	mov    %r8,%rax
./examples/c30walk/cbuild.rs:118
   6fd74:	mov    %r9,(%rsp)
   6fd78:	xor    %r8d,%r8d
   6fd7b:	mov    %rax,%r9
   6fd7e:	call   2a370 <<c30walk::ctor::Ctor>::build::<false>>
./examples/c30walk/cbuild.rs:119
   6fd83:	pop    %rcx
   6fd84:	ret
