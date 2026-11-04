# <c30walk::ctor2su::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000096ec0 <<c30walk::ctor2su::W<false>>::refill>:
   96ec0:	push   %r15
   96ec2:	push   %r14
   96ec4:	push   %rbx
   96ec5:	mov    %rdi,%rbx
   96ec8:	lea    (%rsi,%rsi,4),%rax
   96ecc:	lea    (%rdi,%rax,8),%r15
   96ed0:	add    $0x40,%r15
   96ed4:	mov    0x40(%rdi,%rax,8),%r14
   96ed9:	sub    0x50(%rdi,%rax,8),%r14
   96ede:	je     96ee8 <<c30walk::ctor2su::W<false>>::refill+0x28>
   96ee0:	mov    %rbx,%rdi
   96ee3:	call   968c0 <<c30walk::ctor2su::W<false>>::finish>
   96ee8:	shr    $1,%r14
   96eeb:	add    0x18(%r15),%r14d
   96eef:	mov    %r14d,0x18(%r15)
   96ef3:	xorps  %xmm0,%xmm0
   96ef6:	movups %xmm0,(%r15)
   96efa:	movq   $0x0,0x10(%r15)
   96f02:	mov    0x1c(%r15),%eax
   96f06:	cmp    %eax,%r14d
   96f09:	jae    96f94 <<c30walk::ctor2su::W<false>>::refill+0xd4>
   96f0f:	mov    %r14d,%ecx
   96f12:	mov    $0x88888889,%edi
   96f17:	imul   %rcx,%rdi
   96f1b:	mov    %rdi,%rcx
   96f1e:	shr    $0x24,%rcx
   96f22:	mov    %ecx,%r9d
   96f25:	shl    $0x5,%r9d
   96f29:	sub    %ecx,%r9d
   96f2c:	sub    %ecx,%r9d
   96f2f:	lea    0x1e(%r9),%edx
   96f33:	cmp    %eax,%edx
   96f35:	cmovae %eax,%edx
   96f38:	shr    $0x2a,%rdi
   96f3c:	mov    0x10(%rbx),%rsi
   96f40:	cmp    %rsi,%rdi
   96f43:	jae    96fa0 <<c30walk::ctor2su::W<false>>::refill+0xe0>
   96f45:	mov    %r14d,%esi
   96f48:	mov    %rcx,%r8
   96f4b:	shl    $0x6,%r8
   96f4f:	add    0x28(%rbx),%r8
   96f53:	sub    %r9d,%esi
   96f56:	mov    0x8(%rbx),%r9
   96f5a:	mov    (%r9,%rdi,8),%rdi
   96f5e:	bt     %rcx,%rdi
   96f62:	jae    96f6f <<c30walk::ctor2su::W<false>>::refill+0xaf>
   96f64:	mov    $0xfffff,%edi
   96f69:	and    0x3c(%r8),%edi
   96f6d:	jmp    96f74 <<c30walk::ctor2su::W<false>>::refill+0xb4>
   96f6f:	mov    $0xffffffff,%edi
   96f74:	mov    %esi,%esi
   96f76:	lea    (%r8,%rsi,2),%rsi
   96f7a:	mov    %rsi,0x10(%r15)
   96f7e:	mov    %rsi,(%r15)
   96f81:	sub    %r14d,%edx
   96f84:	lea    (%rsi,%rdx,2),%rdx
   96f88:	mov    %rdx,0x8(%r15)
   96f8c:	mov    %ecx,0x20(%r15)
   96f90:	mov    %edi,0x24(%r15)
   96f94:	cmp    %eax,%r14d
   96f97:	setb   %al
   96f9a:	pop    %rbx
   96f9b:	pop    %r14
   96f9d:	pop    %r15
   96f9f:	ret
   96fa0:	lea    0xafb71(%rip),%rdx        # 146b18 <__frame_dummy_init_array_entry+0x288>
   96fa7:	call   efe73 <core::panicking::panic_bounds_check>
