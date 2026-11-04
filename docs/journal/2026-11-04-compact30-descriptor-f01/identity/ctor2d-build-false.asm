# <c30walk::ctor2d::Ctor2d>::build::<false>
# size 0x1a0a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003aea0 <<c30walk::ctor2d::Ctor2d>::build::<false>>:
   3aea0:	push   %rbp
   3aea1:	push   %r15
   3aea3:	push   %r14
   3aea5:	push   %r13
   3aea7:	push   %r12
   3aea9:	push   %rbx
   3aeaa:	sub    $0x1000,%rsp
   3aeb1:	movq   $0x0,(%rsp)
   3aeb9:	sub    $0x1000,%rsp
   3aec0:	movq   $0x0,(%rsp)
   3aec8:	sub    $0x1000,%rsp
   3aecf:	movq   $0x0,(%rsp)
   3aed7:	sub    $0x1000,%rsp
   3aede:	movq   $0x0,(%rsp)
   3aee6:	sub    $0x4e8,%rsp
   3aeed:	mov    %r8d,0x1c(%rsp)
   3aef2:	mov    0x4520(%rsp),%rax
   3aefa:	movq   $0x0,0x10(%r9)
   3af02:	movq   $0x0,0x10(%rax)
   3af0a:	cmp    $0xeffe2,%rdx
   3af11:	ja     3c50d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3af17:	mov    %rcx,%rbx
   3af1a:	mov    %rsi,%r12
   3af1d:	mov    %rdi,%rbp
   3af20:	mov    %rdx,%rsi
   3af23:	pxor   %xmm0,%xmm0
   3af27:	xor    %eax,%eax
   3af29:	pxor   %xmm2,%xmm2
   3af2d:	pxor   %xmm1,%xmm1
   3af31:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3af40:	movq   (%rbx,%rax,4),%xmm3
   3af45:	movq   0x8(%rbx,%rax,4),%xmm4
   3af4b:	punpckldq %xmm0,%xmm3
   3af4f:	paddq  %xmm2,%xmm3
   3af53:	punpckldq %xmm0,%xmm4
   3af57:	paddq  %xmm1,%xmm4
   3af5b:	movq   0x10(%rbx,%rax,4),%xmm2
   3af61:	movq   0x18(%rbx,%rax,4),%xmm1
   3af67:	punpckldq %xmm0,%xmm2
   3af6b:	paddq  %xmm3,%xmm2
   3af6f:	punpckldq %xmm0,%xmm1
   3af73:	paddq  %xmm4,%xmm1
   3af77:	add    $0x8,%rax
   3af7b:	cmp    $0x100,%rax
   3af81:	jne    3af40 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa0>
   3af83:	paddq  %xmm2,%xmm1
   3af87:	pshufd $0xee,%xmm1,%xmm0
   3af8c:	paddq  %xmm1,%xmm0
   3af90:	movq   %xmm0,%rax
   3af95:	cmp    %rsi,%rax
   3af98:	jne    3c50d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3af9e:	pxor   %xmm2,%xmm2
   3afa2:	xor    %eax,%eax
   3afa4:	movdqa -0x2b62c(%rip),%xmm3        # f980 <GCC_except_table976+0xaec>
   3afac:	pxor   %xmm1,%xmm1
   3afb0:	pxor   %xmm0,%xmm0
   3afb4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3afc0:	movq   (%rbx,%rax,4),%xmm4
   3afc5:	movq   0x8(%rbx,%rax,4),%xmm5
   3afcb:	pshufd $0xd4,%xmm4,%xmm4
   3afd0:	pcmpeqd %xmm2,%xmm4
   3afd4:	pandn  %xmm3,%xmm4
   3afd8:	paddq  %xmm1,%xmm4
   3afdc:	pshufd $0xd4,%xmm5,%xmm5
   3afe1:	pcmpeqd %xmm2,%xmm5
   3afe5:	pandn  %xmm3,%xmm5
   3afe9:	paddq  %xmm0,%xmm5
   3afed:	movq   0x10(%rbx,%rax,4),%xmm0
   3aff3:	movq   0x18(%rbx,%rax,4),%xmm6
   3aff9:	pshufd $0xd4,%xmm0,%xmm1
   3affe:	pcmpeqd %xmm2,%xmm1
   3b002:	pandn  %xmm3,%xmm1
   3b006:	paddq  %xmm4,%xmm1
   3b00a:	pshufd $0xd4,%xmm6,%xmm0
   3b00f:	pcmpeqd %xmm2,%xmm0
   3b013:	pandn  %xmm3,%xmm0
   3b017:	paddq  %xmm5,%xmm0
   3b01b:	add    $0x8,%rax
   3b01f:	cmp    $0x100,%rax
   3b025:	jne    3afc0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x120>
   3b027:	mov    $0x88888889,%r8d
   3b02d:	imul   %rsi,%r8
   3b031:	shr    $0x24,%r8
   3b035:	imul   $0xeeeeeeef,%esi,%eax
   3b03b:	ror    $1,%eax
   3b03d:	cmp    $0x8888889,%eax
   3b042:	sbb    $0xffffffffffffffff,%r8
   3b046:	mov    %r8,%r15
   3b049:	cmpl   $0x0,0x1c(%rsp)
   3b04e:	jne    3b0a5 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3b050:	test   %rsi,%rsi
   3b053:	je     3b0a2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x202>
   3b055:	paddq  %xmm1,%xmm0
   3b059:	pshufd $0xee,%xmm0,%xmm1
   3b05e:	paddq  %xmm0,%xmm1
   3b062:	movq   %xmm1,%rax
   3b067:	lea    -0x1(%rsi),%ecx
   3b06a:	mov    $0x88888889,%edx
   3b06f:	imul   %rcx,%rdx
   3b073:	shr    $0x23,%rdx
   3b077:	and    $0xfffffffe,%edx
   3b07a:	lea    (%rdx,%rsi,1),%ecx
   3b07d:	dec    %ecx
   3b07f:	shr    $1,%ecx
   3b081:	mov    $0x80010003,%r15d
   3b087:	imul   %rcx,%r15
   3b08b:	shr    $0x2e,%r15
   3b08f:	inc    %r15d
   3b092:	imul   %rax,%r15
   3b096:	dec    %r15
   3b099:	cmp    %r8,%r15
   3b09c:	cmovae %r8,%r15
   3b0a0:	jmp    3b0a5 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3b0a2:	xor    %r15d,%r15d
   3b0a5:	mov    (%r9),%rax
   3b0a8:	lea    0x1(%r8),%rcx
   3b0ac:	mov    %rcx,0x68(%rsp)
   3b0b1:	cmp    %r8,%rax
   3b0b4:	mov    %rsi,(%rsp)
   3b0b8:	mov    %r8,0x58(%rsp)
   3b0bd:	jbe    3c6c2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1822>
   3b0c3:	mov    0x4520(%rsp),%rdi
   3b0cb:	mov    (%rdi),%rax
   3b0ce:	cmp    %r15,%rax
   3b0d1:	jb     3c717 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1877>
   3b0d7:	movq   $0x0,0x10(%rbp)
   3b0df:	lea    0x3f(%r8),%r15
   3b0e3:	shr    $0x6,%r15
   3b0e7:	mov    %r12,0x40(%rsp)
   3b0ec:	je     3b13d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x29d>
   3b0ee:	cmp    0x0(%rbp),%r15
   3b0f2:	ja     3c823 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1983>
   3b0f8:	xor    %r14d,%r14d
   3b0fb:	mov    0x8(%rbp),%r12
   3b0ff:	lea    (%r12,%r14,8),%rdi
   3b103:	cmp    $0x1,%r15
   3b107:	je     3b131 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3b109:	lea    -0x8(,%r15,8),%rdx
   3b111:	xor    %esi,%esi
   3b113:	mov    %r9,%r13
   3b116:	call   *0xfbe24(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   3b11c:	mov    %r13,%r9
   3b11f:	lea    (%r14,%r15,1),%rax
   3b123:	lea    (%r12,%rax,8),%rdi
   3b127:	add    $0xfffffffffffffff8,%rdi
   3b12b:	add    %r15,%r14
   3b12e:	dec    %r14
   3b131:	movq   $0x0,(%rdi)
   3b138:	inc    %r14
   3b13b:	jmp    3b140 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2a0>
   3b13d:	xor    %r14d,%r14d
   3b140:	lea    0x18(%rbp),%r15
   3b144:	mov    %r14,0x10(%rbp)
   3b148:	movq   $0x0,0x28(%rbp)
   3b150:	cmpq   $0xf,0x18(%rbp)
   3b155:	jbe    3c7fe <<c30walk::ctor2d::Ctor2d>::build::<false>+0x195e>
   3b15b:	lea    0x30(%rbp),%rax
   3b15f:	mov    %rax,0x48(%rsp)
   3b164:	xor    %eax,%eax
   3b166:	pxor   %xmm0,%xmm0
   3b16a:	movabs $0xffffffff00000000,%rcx
   3b174:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b180:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3b189:	mov    %rcx,0x2cf8(%rsp,%rax,1)
   3b191:	movdqu %xmm0,0x2d00(%rsp,%rax,1)
   3b19a:	mov    %rcx,0x2d10(%rsp,%rax,1)
   3b1a2:	movdqu %xmm0,0x2d18(%rsp,%rax,1)
   3b1ab:	mov    %rcx,0x2d28(%rsp,%rax,1)
   3b1b3:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   3b1bc:	mov    %rcx,0x2d40(%rsp,%rax,1)
   3b1c4:	add    $0x60,%rax
   3b1c8:	cmp    $0x1800,%rax
   3b1ce:	jne    3b180 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2e0>
   3b1d0:	mov    %r15,0x28(%rsp)
   3b1d5:	lea    0xc(%rbx),%rax
   3b1d9:	xor    %edx,%edx
   3b1db:	mov    $0x15,%ecx
   3b1e0:	mov    -0xc(%rax),%esi
   3b1e3:	add    %edx,%esi
   3b1e5:	mov    %edx,0x2c9c(%rsp,%rcx,4)
   3b1ec:	mov    %esi,0x2ca0(%rsp,%rcx,4)
   3b1f3:	mov    -0x8(%rax),%edx
   3b1f6:	add    %esi,%edx
   3b1f8:	mov    %esi,0x2cb4(%rsp,%rcx,4)
   3b1ff:	mov    %edx,0x2cb8(%rsp,%rcx,4)
   3b206:	mov    -0x4(%rax),%esi
   3b209:	add    %edx,%esi
   3b20b:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   3b212:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   3b219:	mov    (%rax),%edx
   3b21b:	add    %esi,%edx
   3b21d:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3b224:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3b22b:	add    $0x18,%rcx
   3b22f:	add    $0x10,%rax
   3b233:	cmp    $0x615,%rcx
   3b23a:	jne    3b1e0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x340>
   3b23c:	mov    %r9,0x88(%rsp)
   3b244:	mov    0x8(%r9),%r14
   3b248:	mov    0x8(%rbp),%r13
   3b24c:	mov    %rbp,0x80(%rsp)
   3b254:	mov    0x10(%rbp),%rbp
   3b258:	lea    0x8d8(%rsp),%rdi
   3b260:	xor    %r12d,%r12d
   3b263:	mov    $0x800,%edx
   3b268:	xor    %esi,%esi
   3b26a:	call   *0xfbcd0(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   3b270:	lea    0x10d8(%rsp),%rdi
   3b278:	lea    0x2ce8(%rsp),%rsi
   3b280:	mov    $0x1800,%edx
   3b285:	call   *0xfbcd5(%rip)        # 136f60 <memcpy@GLIBC_2.14>
   3b28b:	mov    %r14,0xc0(%rsp)
   3b293:	mov    0x58(%rsp),%rax
   3b298:	mov    %rax,0xc8(%rsp)
   3b2a0:	mov    (%rsp),%r15
   3b2a4:	mov    %r15,0xd0(%rsp)
   3b2ac:	mov    0x4520(%rsp),%rax
   3b2b4:	mov    %rax,0x98(%rsp)
   3b2bc:	mov    %r13,0x10(%rsp)
   3b2c1:	mov    %r13,0xa0(%rsp)
   3b2c9:	mov    %rbp,0x8(%rsp)
   3b2ce:	mov    %rbp,0xa8(%rsp)
   3b2d6:	mov    0x28(%rsp),%rax
   3b2db:	mov    %rax,0xb0(%rsp)
   3b2e3:	lea    0xd8(%rsp),%rdi
   3b2eb:	mov    $0x800,%edx
   3b2f0:	xor    %esi,%esi
   3b2f2:	call   *0xfbc48(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   3b2f8:	mov    %r15,%r11
   3b2fb:	movl   $0x0,0x28e0(%rsp)
   3b306:	movq   $0x0,0x28d8(%rsp)
   3b312:	mov    0x48(%rsp),%rax
   3b317:	mov    %rax,0xb8(%rsp)
   3b31f:	mov    $0xffffffffffffffff,%rcx
   3b326:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3b330:	movabs $0x888888888888889,%r15
   3b33a:	pxor   %xmm0,%xmm0
   3b33e:	pcmpeqd %xmm1,%xmm1
   3b342:	xor    %r13d,%r13d
   3b345:	jmp    3b365 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4c5>
   3b347:	nopw   0x0(%rax,%rax,1)
   3b350:	add    $0x4,%r13
   3b354:	cmp    $0x400,%r13
   3b35b:	mov    (%rsp),%r11
   3b35f:	je     3b48a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5ea>
   3b365:	mov    %r12,%rax
   3b368:	mov    (%rbx,%r13,1),%r12d
   3b36c:	add    %rax,%r12
   3b36f:	cmp    %r11,%r12
   3b372:	setae  %sil
   3b376:	mov    %r12,%rax
   3b379:	movabs $0x8888888888888889,%rdx
   3b383:	mul    %rdx
   3b386:	shr    $0x4,%rdx
   3b38a:	mov    %r12,%rax
   3b38d:	imul   %r8,%rax
   3b391:	ror    $1,%rax
   3b394:	cmp    %r15,%rax
   3b397:	setb   %al
   3b39a:	cmp    %rcx,%rdx
   3b39d:	sete   %dil
   3b3a1:	or     %sil,%dil
   3b3a4:	or     %al,%dil
   3b3a7:	jne    3b350 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3b3a9:	mov    0x4520(%rsp),%rdi
   3b3b1:	mov    0x10(%rdi),%rbp
   3b3b5:	cmp    (%rdi),%rbp
   3b3b8:	je     3b45c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5bc>
   3b3be:	mov    0x8(%rdi),%rax
   3b3c2:	mov    %rbp,%rcx
   3b3c5:	shl    $0x7,%rcx
   3b3c9:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3b3cf:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3b3d5:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3b3db:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3b3e1:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3b3e7:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3b3ed:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3b3f3:	movdqa %xmm0,(%rax,%rcx,1)
   3b3f8:	mov    %rbp,%rax
   3b3fb:	inc    %rax
   3b3fe:	mov    %rax,0x10(%rdi)
   3b402:	mov    %rdx,%rax
   3b405:	shl    $0x6,%rax
   3b409:	or     $0x10000000,%ebp
   3b40f:	movdqu %xmm1,0x2c(%r14,%rax,1)
   3b416:	movdqa %xmm1,0x20(%r14,%rax,1)
   3b41d:	movdqa %xmm1,0x10(%r14,%rax,1)
   3b424:	movdqa %xmm1,(%r14,%rax,1)
   3b42a:	mov    %ebp,0x3c(%r14,%rax,1)
   3b42f:	mov    %rdx,%rdi
   3b432:	shr    $0x6,%rdi
   3b436:	cmp    0x8(%rsp),%rdi
   3b43b:	jae    3c88a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19ea>
   3b441:	mov    $0x1,%eax
   3b446:	mov    %edx,%ecx
   3b448:	shl    %cl,%rax
   3b44b:	mov    0x10(%rsp),%rcx
   3b450:	or     %rax,(%rcx,%rdi,8)
   3b454:	mov    %rdx,%rcx
   3b457:	jmp    3b350 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3b45c:	mov    %rdx,0x28(%rsp)
   3b461:	call   92c50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3b466:	pcmpeqd %xmm1,%xmm1
   3b46a:	pxor   %xmm0,%xmm0
   3b46e:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3b478:	mov    0x4520(%rsp),%rdi
   3b480:	mov    0x28(%rsp),%rdx
   3b485:	jmp    3b3be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x51e>
   3b48a:	cmpl   $0x0,0x1c(%rsp)
   3b48f:	je     3b642 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3b495:	mov    0xc8(%rsp),%r8
   3b49d:	test   %r8,%r8
   3b4a0:	je     3b642 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3b4a6:	mov    %r11,%r9
   3b4a9:	shl    $0x20,%r9
   3b4ad:	mov    0x1c(%rsp),%r10d
   3b4b2:	mov    0x98(%rsp),%rdi
   3b4ba:	mov    0xc0(%rsp),%rbp
   3b4c2:	xor    %r15d,%r15d
   3b4c5:	movabs $0xff51afd7ed558ccd,%r14
   3b4cf:	pxor   %xmm0,%xmm0
   3b4d3:	pcmpeqd %xmm1,%xmm1
   3b4d7:	jmp    3b560 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6c0>
   3b4dc:	mov    0x8(%rdi),%rax
   3b4e0:	mov    %r14,%rcx
   3b4e3:	shl    $0x7,%rcx
   3b4e7:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3b4ed:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3b4f3:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3b4f9:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3b4ff:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3b505:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3b50b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3b511:	movdqa %xmm0,(%rax,%rcx,1)
   3b516:	mov    %r14,%rax
   3b519:	inc    %rax
   3b51c:	mov    %rax,0x10(%rdi)
   3b520:	or     $0x10000000,%r14d
   3b527:	movdqu %xmm1,0x2c(%rbp)
   3b52c:	movdqa %xmm1,0x20(%rbp)
   3b531:	movdqa %xmm1,0x10(%rbp)
   3b536:	movdqa %xmm1,0x0(%rbp)
   3b53b:	mov    %r14d,0x3c(%rbp)
   3b53f:	or     %rbx,0x0(%r13,%r12,8)
   3b544:	movabs $0xff51afd7ed558ccd,%r14
   3b54e:	xchg   %ax,%ax
   3b550:	inc    %r15
   3b553:	add    $0x40,%rbp
   3b557:	cmp    %r15,%r8
   3b55a:	je     3b642 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3b560:	mov    %r15,%r12
   3b563:	shr    $0x6,%r12
   3b567:	mov    0xa8(%rsp),%rsi
   3b56f:	cmp    %rsi,%r12
   3b572:	jae    3c89b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19fb>
   3b578:	mov    0xa0(%rsp),%r13
   3b580:	mov    0x0(%r13,%r12,8),%rax
   3b585:	mov    $0x1,%ebx
   3b58a:	mov    %r15d,%ecx
   3b58d:	shl    %cl,%rbx
   3b590:	bt     %r15,%rax
   3b594:	jb     3b550 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3b596:	mov    %r15,%rax
   3b599:	xor    %r9,%rax
   3b59c:	mov    %rax,%rcx
   3b59f:	shr    $0x21,%rcx
   3b5a3:	xor    %rax,%rcx
   3b5a6:	imul   %r14,%rcx
   3b5aa:	mov    %rcx,%rax
   3b5ad:	shr    $0x21,%rax
   3b5b1:	xor    %rcx,%rax
   3b5b4:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3b5be:	imul   %rcx,%rax
   3b5c2:	mov    %rax,%rcx
   3b5c5:	shr    $0x21,%rcx
   3b5c9:	xor    %rax,%rcx
   3b5cc:	mov    %rcx,%rax
   3b5cf:	shr    $0x3,%rax
   3b5d3:	movabs $0x20c49ba5e353f7cf,%rdx
   3b5dd:	mul    %rdx
   3b5e0:	shr    $0x4,%rdx
   3b5e4:	imul   $0x3e8,%rdx,%rax
   3b5eb:	sub    %rax,%rcx
   3b5ee:	cmp    %r10,%rcx
   3b5f1:	jae    3b550 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3b5f7:	mov    0x10(%rdi),%r14
   3b5fb:	cmp    (%rdi),%r14
   3b5fe:	jne    3b4dc <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3b604:	mov    %r8,0x8(%rsp)
   3b609:	mov    %r9,0x10(%rsp)
   3b60e:	mov    %r10,0x28(%rsp)
   3b613:	mov    %rdi,0x48(%rsp)
   3b618:	call   92c50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3b61d:	pcmpeqd %xmm1,%xmm1
   3b621:	pxor   %xmm0,%xmm0
   3b625:	mov    0x48(%rsp),%rdi
   3b62a:	mov    0x28(%rsp),%r10
   3b62f:	mov    0x10(%rsp),%r9
   3b634:	mov    0x8(%rsp),%r8
   3b639:	mov    (%rsp),%r11
   3b63d:	jmp    3b4dc <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3b642:	test   %r11,%r11
   3b645:	mov    0x40(%rsp),%rbp
   3b64a:	je     3c4ae <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3b650:	xor    %edi,%edi
   3b652:	jmp    3b66c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3b654:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b660:	mov    %rdi,%r8
   3b663:	cmp    %r10,%r8
   3b666:	jae    3c4ae <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3b66c:	mov    %rdi,%r13
   3b66f:	mov    $0x400,%edx
   3b674:	lea    0x28e8(%rsp),%rdi
   3b67c:	xor    %esi,%esi
   3b67e:	call   *0xfb8bc(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   3b684:	mov    $0x848,%eax
   3b689:	mov    $0xa,%ecx
   3b68e:	xchg   %ax,%ax
   3b690:	movd   0x68(%rsp,%rax,2),%xmm0
   3b696:	movd   0x50(%rsp,%rax,2),%xmm1
   3b69c:	punpckldq %xmm0,%xmm1
   3b6a0:	movd   0x98(%rsp,%rax,2),%xmm0
   3b6a9:	movd   0x80(%rsp,%rax,2),%xmm2
   3b6b2:	punpckldq %xmm0,%xmm2
   3b6b6:	punpcklqdq %xmm2,%xmm1
   3b6ba:	movdqu 0x88(%rsp,%rcx,8),%xmm0
   3b6c3:	movdqu 0x98(%rsp,%rcx,8),%xmm2
   3b6cc:	movq   0x60(%rsp,%rax,2),%xmm3
   3b6d2:	movq   0x48(%rsp,%rax,2),%xmm4
   3b6d8:	punpcklqdq %xmm3,%xmm4
   3b6dc:	psubq  %xmm4,%xmm0
   3b6e0:	movq   0x90(%rsp,%rax,2),%xmm3
   3b6e9:	movq   0x78(%rsp,%rax,2),%xmm4
   3b6ef:	punpcklqdq %xmm3,%xmm4
   3b6f3:	psubq  %xmm4,%xmm2
   3b6f7:	psrlq  $0x1,%xmm0
   3b6fc:	psrlq  $0x1,%xmm2
   3b701:	pshufd $0xe8,%xmm0,%xmm0
   3b706:	pshufd $0xe8,%xmm2,%xmm2
   3b70b:	punpcklqdq %xmm2,%xmm0
   3b70f:	paddd  %xmm1,%xmm0
   3b713:	movdqu %xmm0,0x28c0(%rsp,%rcx,4)
   3b71c:	add    $0x30,%rax
   3b720:	add    $0x4,%rcx
   3b724:	cmp    $0x106,%rcx
   3b72b:	jne    3b690 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7f0>
   3b731:	movups 0x2880(%rsp),%xmm0
   3b739:	movdqu 0x8c8(%rsp),%xmm1
   3b742:	movdqu 0x8b8(%rsp),%xmm2
   3b74b:	movq   0x28c0(%rsp),%xmm3
   3b754:	movq   0x28a8(%rsp),%xmm4
   3b75d:	punpcklqdq %xmm3,%xmm4
   3b761:	psubq  %xmm4,%xmm1
   3b765:	movq   0x2890(%rsp),%xmm3
   3b76e:	movq   0x2878(%rsp),%xmm4
   3b777:	punpcklqdq %xmm3,%xmm4
   3b77b:	psubq  %xmm4,%xmm2
   3b77f:	psrlq  $0x1,%xmm1
   3b784:	psrlq  $0x1,%xmm2
   3b789:	shufps $0x88,%xmm1,%xmm2
   3b78d:	movss  0x2898(%rsp),%xmm1
   3b796:	movlhps %xmm0,%xmm1
   3b799:	movss  0x28b0(%rsp),%xmm0
   3b7a2:	movss  0x28c8(%rsp),%xmm3
   3b7ab:	movlhps %xmm0,%xmm3
   3b7ae:	shufps $0x22,%xmm3,%xmm1
   3b7b2:	paddd  %xmm2,%xmm1
   3b7b6:	movdqu %xmm1,0x2cd8(%rsp)
   3b7bf:	mov    0xb0(%rsp),%rbx
   3b7c7:	mov    0x10(%rbx),%r14
   3b7cb:	cmp    (%rbx),%r14
   3b7ce:	je     3c4a1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1601>
   3b7d4:	mov    %r14,%rdi
   3b7d7:	shl    $0xa,%rdi
   3b7db:	add    0x8(%rbx),%rdi
   3b7df:	mov    $0x400,%edx
   3b7e4:	lea    0x28e8(%rsp),%rsi
   3b7ec:	call   *0xfb76e(%rip)        # 136f60 <memcpy@GLIBC_2.14>
   3b7f2:	inc    %r14
   3b7f5:	mov    %r14,0x10(%rbx)
   3b7f9:	mov    %r13,%rdi
   3b7fc:	lea    0xf000(%r13),%r9
   3b803:	mov    (%rsp),%r10
   3b807:	cmp    %r10,%r9
   3b80a:	cmovae %r10,%r9
   3b80e:	lea    0x10(%r13),%r11
   3b812:	cmp    %r9,%r11
   3b815:	mov    %r9,0x60(%rsp)
   3b81a:	jbe    3b8e0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa40>
   3b820:	sub    %rdi,%r9
   3b823:	jbe    3b660 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7c0>
   3b829:	mov    %r12,0x30(%rsp)
   3b82e:	mov    %r15,0x38(%rsp)
   3b833:	mov    0x28e0(%rsp),%r13d
   3b83b:	mov    $0x1e,%r15d
   3b841:	sub    0x28d8(%rsp),%r15
   3b849:	cmp    %r9,%r15
   3b84c:	cmovae %r9,%r15
   3b850:	lea    (%rdi,%rbp,1),%r12
   3b854:	test   %r15,%r15
   3b857:	je     3c310 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3b85d:	xor    %r14d,%r14d
   3b860:	movzbl (%r12,%r14,1),%ebx
   3b865:	mov    0xd8(%rsp,%rbx,8),%rax
   3b86d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3b875:	je     3b894 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9f4>
   3b877:	lea    (%r14,%r13,1),%rcx
   3b87b:	mov    %cx,(%rax)
   3b87e:	addq   $0x2,0xd8(%rsp,%rbx,8)
   3b887:	inc    %r14
   3b88a:	cmp    %r14,%r15
   3b88d:	jne    3b860 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9c0>
   3b88f:	jmp    3c310 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3b894:	mov    %r9,0x10(%rsp)
   3b899:	mov    %rdi,0x8(%rsp)
   3b89e:	lea    0x98(%rsp),%rdi
   3b8a6:	mov    %rbx,%rsi
   3b8a9:	call   95290 <<c30walk::ctor2d::W<false>>::refill>
   3b8ae:	test   %al,%al
   3b8b0:	je     3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3b8b6:	mov    0xd8(%rsp,%rbx,8),%rax
   3b8be:	mov    (%rsp),%r10
   3b8c2:	mov    0x40(%rsp),%rbp
   3b8c7:	mov    0x8(%rsp),%rdi
   3b8cc:	mov    0x10(%rsp),%r9
   3b8d1:	jmp    3b877 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9d7>
   3b8d3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b8e0:	xor    %edx,%edx
   3b8e2:	lea    0x10d8(%rsp),%r8
   3b8ea:	jmp    3b953 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xab3>
   3b8ec:	nopl   0x0(%rax)
   3b8f0:	mov    0x28d8(%rsp),%rax
   3b8f8:	lea    0x10(%rax),%rcx
   3b8fc:	add    $0xfffffffffffffff2,%rax
   3b900:	xor    %edx,%edx
   3b902:	cmp    $0x1e,%rcx
   3b906:	setae  %dl
   3b909:	cmovb  %rcx,%rax
   3b90d:	mov    %rax,0x28d8(%rsp)
   3b915:	mov    0x28e0(%rsp),%eax
   3b91c:	lea    0x10(%rax,%rdx,2),%eax
   3b920:	mov    %eax,0x28e0(%rsp)
   3b927:	mov    0x20(%rsp),%rax
   3b92c:	lea    0x10(%rax),%rcx
   3b930:	xor    %edx,%edx
   3b932:	mov    %rax,%rdi
   3b935:	mov    %rcx,%r11
   3b938:	mov    0x60(%rsp),%r9
   3b93d:	cmp    %r9,%rcx
   3b940:	mov    0x40(%rsp),%rbp
   3b945:	lea    0x10d8(%rsp),%r8
   3b94d:	ja     3b820 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x980>
   3b953:	mov    %r12,%rcx
   3b956:	test   $0x1,%dl
   3b959:	jne    3b989 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xae9>
   3b95b:	lea    0x8(%rdi),%rsi
   3b95f:	cmp    $0xfffffffffffffff7,%rdi
   3b963:	ja     3c521 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b969:	cmp    %r10,%rsi
   3b96c:	ja     3c521 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b972:	lea    0x10(%rdi),%rax
   3b976:	cmp    %r10,%rax
   3b979:	ja     3c784 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18e4>
   3b97f:	mov    0x0(%rbp,%rdi,1),%r15
   3b984:	mov    0x0(%rbp,%rsi,1),%rcx
   3b989:	mov    %r15,%rax
   3b98c:	xor    %rcx,%rax
   3b98f:	mov    %r15,%rdx
   3b992:	rol    $0x8,%rdx
   3b996:	xor    %r15,%rdx
   3b999:	or     %rax,%rdx
   3b99c:	jne    3ba00 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb60>
   3b99e:	mov    %rdi,%rbx
   3b9a1:	mov    %r11,%rax
   3b9a4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b9b0:	mov    %rax,%rdi
   3b9b3:	add    $0x10,%rax
   3b9b7:	cmp    %r9,%rax
   3b9ba:	setbe  %r11b
   3b9be:	ja     3ba60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc0>
   3b9c4:	lea    0x8(%rdi),%rsi
   3b9c8:	cmp    $0xfffffffffffffff7,%rdi
   3b9cc:	ja     3c521 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b9d2:	cmp    %r10,%rsi
   3b9d5:	ja     3c521 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b9db:	mov    0x0(%rbp,%rdi,1),%rdx
   3b9e0:	mov    0x8(%rbp,%rdi,1),%rsi
   3b9e5:	cmp    %rcx,%rdx
   3b9e8:	jne    3b9ef <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb4f>
   3b9ea:	cmp    %rcx,%rsi
   3b9ed:	je     3b9b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb10>
   3b9ef:	mov    %rsi,0x30(%rsp)
   3b9f4:	mov    %rdx,0x38(%rsp)
   3b9f9:	lea    0x10(%rdi),%r9
   3b9fd:	jmp    3ba64 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc4>
   3b9ff:	nop
   3ba00:	mov    %r11,0x20(%rsp)
   3ba05:	mov    0x28d8(%rsp),%rax
   3ba0d:	mov    $0x1e,%r13d
   3ba13:	sub    %rax,%r13
   3ba16:	cmp    $0x10,%r13
   3ba1a:	mov    $0x10,%r14d
   3ba20:	cmovb  %r13,%r14
   3ba24:	mov    %r14,%rbx
   3ba27:	add    %rdi,%rbx
   3ba2a:	jb     3c772 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18d2>
   3ba30:	cmp    %r10,%rbx
   3ba33:	ja     3c772 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18d2>
   3ba39:	mov    0x28e0(%rsp),%r12d
   3ba41:	cmp    $0x1e,%rax
   3ba45:	jne    3c200 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1360>
   3ba4b:	cmp    %rbx,0x20(%rsp)
   3ba50:	jb     3c799 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18f9>
   3ba56:	add    %rbp,%rbx
   3ba59:	jmp    3c29a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13fa>
   3ba5e:	xchg   %ax,%ax
   3ba60:	lea    0x10(%rdi),%r9
   3ba64:	movzbl %cl,%r12d
   3ba68:	lea    (%r12,%r12,2),%rdx
   3ba6c:	mov    %r12d,%eax
   3ba6f:	mov    0xd8(%rsp,%rax,8),%rax
   3ba77:	mov    %rax,%rsi
   3ba7a:	sub    (%r8,%rdx,8),%rsi
   3ba7e:	mov    %rdi,0x20(%rsp)
   3ba83:	mov    %rdi,%rbp
   3ba86:	shr    $1,%rsi
   3ba89:	add    0x8(%r8,%rdx,8),%esi
   3ba8e:	sub    %rbx,%rbp
   3ba91:	add    %rbp,%rsi
   3ba94:	mov    0xc(%r8,%rdx,8),%edi
   3ba99:	cmp    %rdi,%rsi
   3ba9c:	ja     3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3baa2:	mov    %r9,0x70(%rsp)
   3baa7:	mov    %r11d,0x54(%rsp)
   3baac:	lea    (%r8,%rdx,8),%rbx
   3bab0:	movzbl %cl,%r14d
   3bab4:	mov    0x8d8(%rsp,%r12,8),%rsi
   3babc:	mov    $0x88888889,%r15d
   3bac2:	lea    -0x20149(%rip),%r13        # 1b980 <c30walk::ctor2d::PATTERNS>
   3bac9:	mov    %r12,0x90(%rsp)
   3bad1:	mov    %rbx,0x78(%rsp)
   3bad6:	jmp    3bb16 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc76>
   3bad8:	nopl   0x0(%rax,%rax,1)
   3bae0:	mov    0x3c(%rsi),%edi
   3bae3:	mov    $0xfffff,%r9d
   3bae9:	and    %r9d,%edi
   3baec:	sub    %r8d,%ecx
   3baef:	sub    %eax,%r8d
   3baf2:	mov    %r8d,%eax
   3baf5:	lea    (%rsi,%rax,2),%rax
   3baf9:	mov    %rax,0xd8(%rsp,%r12,8)
   3bb01:	lea    (%rax,%rcx,2),%rsi
   3bb05:	mov    %rsi,0x8d8(%rsp,%r12,8)
   3bb0d:	mov    %rax,(%rbx)
   3bb10:	mov    %edx,0x10(%rbx)
   3bb13:	mov    %edi,0x14(%rbx)
   3bb16:	sub    %rax,%rsi
   3bb19:	shr    $1,%rsi
   3bb1c:	cmp    %rsi,%rbp
   3bb1f:	cmovb  %rbp,%rsi
   3bb23:	test   %rsi,%rsi
   3bb26:	je     3bbdd <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd3d>
   3bb2c:	mov    0x28d8(%rsp),%rdi
   3bb34:	cmp    $0x1d,%rdi
   3bb38:	ja     3c868 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19c8>
   3bb3e:	cmp    $0x1f,%rsi
   3bb42:	jae    3c6af <<c30walk::ctor2d::Ctor2d>::build::<false>+0x180f>
   3bb48:	mov    0x28e0(%rsp),%ecx
   3bb4f:	mov    %rdi,%r11
   3bb52:	shl    $0x6,%r11
   3bb56:	lea    (%r11,%r13,1),%rdx
   3bb5a:	lea    -0x2(,%rsi,2),%r9
   3bb62:	cmp    $0xe,%r9
   3bb66:	jb     3bb7b <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcdb>
   3bb68:	mov    %rax,%r8
   3bb6b:	sub    %r13,%r8
   3bb6e:	sub    %r11,%r8
   3bb71:	cmp    $0x10,%r8
   3bb75:	jae    3bfe3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1143>
   3bb7b:	xor    %r10d,%r10d
   3bb7e:	mov    %rdx,%r8
   3bb81:	lea    (%rdx,%rsi,2),%rdx
   3bb85:	lea    (%rax,%r10,2),%r9
   3bb89:	nopl   0x0(%rax)
   3bb90:	movzwl (%r8),%r10d
   3bb94:	add    %cx,%r10w
   3bb98:	add    $0x2,%r8
   3bb9c:	mov    %r10w,(%r9)
   3bba0:	add    $0x2,%r9
   3bba4:	cmp    %rdx,%r8
   3bba7:	jne    3bb90 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcf0>
   3bba9:	lea    (%rax,%rsi,2),%rax
   3bbad:	mov    %rax,0xd8(%rsp,%r12,8)
   3bbb5:	add    %rsi,%rdi
   3bbb8:	lea    0x2(%rsi),%edx
   3bbbb:	mov    %rdi,%r8
   3bbbe:	sub    $0x1e,%r8
   3bbc2:	cmovb  %rdi,%r8
   3bbc6:	cmovb  %esi,%edx
   3bbc9:	mov    %r8,0x28d8(%rsp)
   3bbd1:	add    %ecx,%edx
   3bbd3:	mov    %edx,0x28e0(%rsp)
   3bbda:	sub    %rsi,%rbp
   3bbdd:	test   %rbp,%rbp
   3bbe0:	je     3c1d0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3bbe6:	mov    %rax,%r8
   3bbe9:	cmp    (%rbx),%rax
   3bbec:	je     3bc09 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd69>
   3bbee:	lea    0x98(%rsp),%rdi
   3bbf6:	mov    %r12,%rsi
   3bbf9:	call   94c80 <<c30walk::ctor2d::W<false>>::finish>
   3bbfe:	mov    0xd8(%rsp,%r12,8),%r8
   3bc06:	mov    (%rbx),%rax
   3bc09:	sub    %rax,%r8
   3bc0c:	shr    $1,%r8
   3bc0f:	add    0x8(%rbx),%r8d
   3bc13:	mov    %r8d,0x8(%rbx)
   3bc17:	movq   $0x0,(%rbx)
   3bc1e:	movq   $0x0,0xd8(%rsp,%r12,8)
   3bc2a:	movq   $0x0,0x8d8(%rsp,%r12,8)
   3bc36:	cmp    $0x1e,%rbp
   3bc3a:	jb     3c162 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3bc40:	mov    %r8,%rcx
   3bc43:	imul   %r15,%rcx
   3bc47:	shr    $0x24,%rcx
   3bc4b:	mov    %ecx,%eax
   3bc4d:	shl    $0x5,%eax
   3bc50:	lea    (%rcx,%rcx,1),%edx
   3bc53:	sub    %eax,%edx
   3bc55:	add    %r8d,%edx
   3bc58:	jne    3c162 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3bc5e:	mov    %rbp,%rax
   3bc61:	movabs $0x8888888888888889,%rdx
   3bc6b:	mul    %rdx
   3bc6e:	shr    $0x4,%rdx
   3bc72:	mov    %ecx,%r9d
   3bc75:	cmpl   $0x0,0x1c(%rsp)
   3bc7a:	mov    %rbp,0x28(%rsp)
   3bc7f:	mov    %rdx,0x48(%rsp)
   3bc84:	je     3bfa0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1100>
   3bc8a:	mov    0x28e0(%rsp),%r10d
   3bc92:	mov    0xa8(%rsp),%rax
   3bc9a:	mov    %rax,0x8(%rsp)
   3bc9f:	mov    0x98(%rsp),%r11
   3bca7:	mov    0xa0(%rsp),%rax
   3bcaf:	mov    %rax,0x10(%rsp)
   3bcb4:	mov    0xc0(%rsp),%rax
   3bcbc:	mov    0x28d8(%rsp),%r12
   3bcc4:	mov    %r12,%rdi
   3bcc7:	shl    $0x6,%rdi
   3bccb:	add    %r13,%rdi
   3bcce:	mov    %r9,%rcx
   3bcd1:	shl    $0x6,%rcx
   3bcd5:	lea    (%rax,%rcx,1),%r15
   3bcd9:	add    $0x3c,%r15
   3bcdd:	mov    %r10d,%ebp
   3bce0:	shl    $0x8,%ebp
   3bce3:	or     %r14d,%ebp
   3bce6:	xor    %ebx,%ebx
   3bce8:	jmp    3bf0f <<c30walk::ctor2d::Ctor2d>::build::<false>+0x106f>
   3bced:	nopl   (%rax)
   3bcf0:	mov    0x8(%r11),%rcx
   3bcf4:	shl    $0x7,%eax
   3bcf7:	mov    %ebp,(%rcx,%rax,1)
   3bcfa:	movzwl 0x2(%rdi),%esi
   3bcfe:	lea    (%r10,%rbx,1),%r13d
   3bd02:	add    %r13d,%esi
   3bd05:	shl    $0x8,%esi
   3bd08:	or     %r14d,%esi
   3bd0b:	mov    %esi,0x4(%rcx,%rax,1)
   3bd0f:	movzwl 0x4(%rdi),%esi
   3bd13:	add    %r13d,%esi
   3bd16:	shl    $0x8,%esi
   3bd19:	or     %r14d,%esi
   3bd1c:	mov    %esi,0x8(%rcx,%rax,1)
   3bd20:	movzwl 0x6(%rdi),%esi
   3bd24:	add    %r13d,%esi
   3bd27:	shl    $0x8,%esi
   3bd2a:	or     %r14d,%esi
   3bd2d:	mov    %esi,0xc(%rcx,%rax,1)
   3bd31:	movzwl 0x8(%rdi),%esi
   3bd35:	add    %r13d,%esi
   3bd38:	shl    $0x8,%esi
   3bd3b:	or     %r14d,%esi
   3bd3e:	mov    %esi,0x10(%rcx,%rax,1)
   3bd42:	movzwl 0xa(%rdi),%esi
   3bd46:	add    %r13d,%esi
   3bd49:	shl    $0x8,%esi
   3bd4c:	or     %r14d,%esi
   3bd4f:	mov    %esi,0x14(%rcx,%rax,1)
   3bd53:	movzwl 0xc(%rdi),%esi
   3bd57:	add    %r13d,%esi
   3bd5a:	shl    $0x8,%esi
   3bd5d:	or     %r14d,%esi
   3bd60:	mov    %esi,0x18(%rcx,%rax,1)
   3bd64:	movzwl 0xe(%rdi),%esi
   3bd68:	add    %r13d,%esi
   3bd6b:	shl    $0x8,%esi
   3bd6e:	or     %r14d,%esi
   3bd71:	mov    %esi,0x1c(%rcx,%rax,1)
   3bd75:	movzwl 0x10(%rdi),%esi
   3bd79:	add    %r13d,%esi
   3bd7c:	shl    $0x8,%esi
   3bd7f:	or     %r14d,%esi
   3bd82:	mov    %esi,0x20(%rcx,%rax,1)
   3bd86:	movzwl 0x12(%rdi),%esi
   3bd8a:	add    %r13d,%esi
   3bd8d:	shl    $0x8,%esi
   3bd90:	or     %r14d,%esi
   3bd93:	mov    %esi,0x24(%rcx,%rax,1)
   3bd97:	movzwl 0x14(%rdi),%esi
   3bd9b:	add    %r13d,%esi
   3bd9e:	shl    $0x8,%esi
   3bda1:	or     %r14d,%esi
   3bda4:	mov    %esi,0x28(%rcx,%rax,1)
   3bda8:	movzwl 0x16(%rdi),%esi
   3bdac:	add    %r13d,%esi
   3bdaf:	shl    $0x8,%esi
   3bdb2:	or     %r14d,%esi
   3bdb5:	mov    %esi,0x2c(%rcx,%rax,1)
   3bdb9:	movzwl 0x18(%rdi),%esi
   3bdbd:	add    %r13d,%esi
   3bdc0:	shl    $0x8,%esi
   3bdc3:	or     %r14d,%esi
   3bdc6:	mov    %esi,0x30(%rcx,%rax,1)
   3bdca:	movzwl 0x1a(%rdi),%esi
   3bdce:	add    %r13d,%esi
   3bdd1:	shl    $0x8,%esi
   3bdd4:	or     %r14d,%esi
   3bdd7:	mov    %esi,0x34(%rcx,%rax,1)
   3bddb:	movzwl 0x1c(%rdi),%esi
   3bddf:	add    %r13d,%esi
   3bde2:	shl    $0x8,%esi
   3bde5:	or     %r14d,%esi
   3bde8:	mov    %esi,0x38(%rcx,%rax,1)
   3bdec:	movzwl 0x1e(%rdi),%esi
   3bdf0:	add    %r13d,%esi
   3bdf3:	shl    $0x8,%esi
   3bdf6:	or     %r14d,%esi
   3bdf9:	mov    %esi,0x3c(%rcx,%rax,1)
   3bdfd:	movzwl 0x20(%rdi),%esi
   3be01:	add    %r13d,%esi
   3be04:	shl    $0x8,%esi
   3be07:	or     %r14d,%esi
   3be0a:	mov    %esi,0x40(%rcx,%rax,1)
   3be0e:	movzwl 0x22(%rdi),%esi
   3be12:	add    %r13d,%esi
   3be15:	shl    $0x8,%esi
   3be18:	or     %r14d,%esi
   3be1b:	mov    %esi,0x44(%rcx,%rax,1)
   3be1f:	movzwl 0x24(%rdi),%esi
   3be23:	add    %r13d,%esi
   3be26:	shl    $0x8,%esi
   3be29:	or     %r14d,%esi
   3be2c:	mov    %esi,0x48(%rcx,%rax,1)
   3be30:	movzwl 0x26(%rdi),%esi
   3be34:	add    %r13d,%esi
   3be37:	shl    $0x8,%esi
   3be3a:	or     %r14d,%esi
   3be3d:	mov    %esi,0x4c(%rcx,%rax,1)
   3be41:	movzwl 0x28(%rdi),%esi
   3be45:	add    %r13d,%esi
   3be48:	shl    $0x8,%esi
   3be4b:	or     %r14d,%esi
   3be4e:	mov    %esi,0x50(%rcx,%rax,1)
   3be52:	movzwl 0x2a(%rdi),%esi
   3be56:	add    %r13d,%esi
   3be59:	shl    $0x8,%esi
   3be5c:	or     %r14d,%esi
   3be5f:	mov    %esi,0x54(%rcx,%rax,1)
   3be63:	movzwl 0x2c(%rdi),%esi
   3be67:	add    %r13d,%esi
   3be6a:	shl    $0x8,%esi
   3be6d:	or     %r14d,%esi
   3be70:	mov    %esi,0x58(%rcx,%rax,1)
   3be74:	movzwl 0x2e(%rdi),%esi
   3be78:	add    %r13d,%esi
   3be7b:	shl    $0x8,%esi
   3be7e:	or     %r14d,%esi
   3be81:	mov    %esi,0x5c(%rcx,%rax,1)
   3be85:	movzwl 0x30(%rdi),%esi
   3be89:	add    %r13d,%esi
   3be8c:	shl    $0x8,%esi
   3be8f:	or     %r14d,%esi
   3be92:	mov    %esi,0x60(%rcx,%rax,1)
   3be96:	movzwl 0x32(%rdi),%esi
   3be9a:	add    %r13d,%esi
   3be9d:	shl    $0x8,%esi
   3bea0:	or     %r14d,%esi
   3bea3:	mov    %esi,0x64(%rcx,%rax,1)
   3bea7:	movzwl 0x34(%rdi),%esi
   3beab:	add    %r13d,%esi
   3beae:	shl    $0x8,%esi
   3beb1:	or     %r14d,%esi
   3beb4:	mov    %esi,0x68(%rcx,%rax,1)
   3beb8:	movzwl 0x36(%rdi),%esi
   3bebc:	add    %r13d,%esi
   3bebf:	shl    $0x8,%esi
   3bec2:	or     %r14d,%esi
   3bec5:	mov    %esi,0x6c(%rcx,%rax,1)
   3bec9:	movzwl 0x38(%rdi),%esi
   3becd:	add    %r13d,%esi
   3bed0:	shl    $0x8,%esi
   3bed3:	or     %r14d,%esi
   3bed6:	mov    %esi,0x70(%rcx,%rax,1)
   3beda:	movzwl 0x3a(%rdi),%esi
   3bede:	add    %r13d,%esi
   3bee1:	shl    $0x8,%esi
   3bee4:	or     %r14d,%esi
   3bee7:	mov    %esi,0x74(%rcx,%rax,1)
   3beeb:	lea    (%r10,%rbx,1),%eax
   3beef:	add    $0x20,%eax
   3bef2:	mov    %eax,0x28e0(%rsp)
   3bef9:	add    $0x20,%rbx
   3befd:	inc    %r9
   3bf00:	add    $0x2000,%ebp
   3bf06:	dec    %rdx
   3bf09:	je     3c119 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1279>
   3bf0f:	mov    %r9,%rax
   3bf12:	shr    $0x6,%rax
   3bf16:	cmp    0x8(%rsp),%rax
   3bf1b:	jae    3c7b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1910>
   3bf21:	mov    0x10(%rsp),%rcx
   3bf26:	mov    (%rcx,%rax,8),%rax
   3bf2a:	bt     %r9,%rax
   3bf2e:	jae    3bf60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x10c0>
   3bf30:	cmp    $0x1e,%r12
   3bf34:	jae    3c7d6 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1936>
   3bf3a:	mov    (%r15,%rbx,2),%eax
   3bf3e:	and    $0xfffff,%eax
   3bf43:	mov    0x10(%r11),%rcx
   3bf47:	cmp    %rax,%rcx
   3bf4a:	ja     3bcf0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xe50>
   3bf50:	jmp    3c7c4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1924>
   3bf55:	data16 cs nopw 0x0(%rax,%rax,1)
   3bf60:	cmp    $0x1e,%r12
   3bf64:	jae    3c7ea <<c30walk::ctor2d::Ctor2d>::build::<false>+0x194a>
   3bf6a:	movups 0x2c(%rdi),%xmm0
   3bf6e:	movups %xmm0,-0x10(%r15,%rbx,2)
   3bf74:	movdqa (%rdi),%xmm0
   3bf78:	movdqa 0x10(%rdi),%xmm1
   3bf7d:	movdqa 0x20(%rdi),%xmm2
   3bf82:	movdqa %xmm2,-0x1c(%r15,%rbx,2)
   3bf89:	movdqa %xmm1,-0x2c(%r15,%rbx,2)
   3bf90:	movdqa %xmm0,-0x3c(%r15,%rbx,2)
   3bf97:	mov    %ebp,(%r15,%rbx,2)
   3bf9b:	jmp    3beeb <<c30walk::ctor2d::Ctor2d>::build::<false>+0x104b>
   3bfa0:	mov    0x28d8(%rsp),%rdi
   3bfa8:	cmp    $0x1d,%rdi
   3bfac:	ja     3c879 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19d9>
   3bfb2:	shl    $0x6,%rdi
   3bfb6:	add    %r13,%rdi
   3bfb9:	mov    0x28e0(%rsp),%eax
   3bfc0:	mov    0xc0(%rsp),%rcx
   3bfc8:	shl    $0x6,%r9
   3bfcc:	mov    %eax,%r15d
   3bfcf:	shl    $0x8,%r15d
   3bfd3:	or     %r14d,%r15d
   3bfd6:	cmp    $0x3c,%rbp
   3bfda:	jae    3c04a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11aa>
   3bfdc:	xor    %esi,%esi
   3bfde:	jmp    3c0d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1237>
   3bfe3:	shr    $1,%r9
   3bfe6:	inc    %r9
   3bfe9:	mov    %r9,%r10
   3bfec:	and    $0xfffffffffffffff8,%r10
   3bff0:	lea    (%rdx,%r10,2),%r8
   3bff4:	movd   %ecx,%xmm0
   3bff8:	pshuflw $0x0,%xmm0,%xmm0
   3bffd:	add    %r13,%r11
   3c000:	add    $0x8,%r11
   3c004:	xor    %ebx,%ebx
   3c006:	cs nopw 0x0(%rax,%rax,1)
   3c010:	movq   -0x8(%r11,%rbx,2),%xmm1
   3c017:	movq   (%r11,%rbx,2),%xmm2
   3c01d:	paddw  %xmm0,%xmm1
   3c021:	paddw  %xmm0,%xmm2
   3c025:	punpcklqdq %xmm2,%xmm1
   3c029:	movdqu %xmm1,(%rax,%rbx,2)
   3c02e:	add    $0x8,%rbx
   3c032:	cmp    %rbx,%r10
   3c035:	jne    3c010 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1170>
   3c037:	cmp    %r10,%r9
   3c03a:	mov    0x78(%rsp),%rbx
   3c03f:	jne    3bb81 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xce1>
   3c045:	jmp    3bba9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd09>
   3c04a:	mov    %rdx,%r10
   3c04d:	movabs $0xffffffffffffffe,%rsi
   3c057:	and    %rsi,%r10
   3c05a:	lea    (%rcx,%r9,1),%r11
   3c05e:	add    $0x7c,%r11
   3c062:	xor    %esi,%esi
   3c064:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c070:	movdqu 0x2c(%rdi),%xmm0
   3c075:	movdqu %xmm0,-0x50(%r11)
   3c07b:	movdqa (%rdi),%xmm1
   3c07f:	movdqa 0x10(%rdi),%xmm2
   3c084:	movaps 0x20(%rdi),%xmm3
   3c088:	movaps %xmm3,-0x5c(%r11)
   3c08d:	movdqa %xmm2,-0x6c(%r11)
   3c093:	movdqa %xmm1,-0x7c(%r11)
   3c099:	mov    %r15d,-0x40(%r11)
   3c09d:	add    $0x2,%rsi
   3c0a1:	movdqa %xmm1,-0x3c(%r11)
   3c0a7:	movdqa %xmm2,-0x2c(%r11)
   3c0ad:	movaps %xmm3,-0x1c(%r11)
   3c0b2:	movdqu %xmm0,-0x10(%r11)
   3c0b8:	lea    0x2000(%r15),%ebx
   3c0bf:	mov    %ebx,(%r11)
   3c0c2:	add    $0x4000,%r15d
   3c0c9:	sub    $0xffffffffffffff80,%r11
   3c0cd:	cmp    %rsi,%r10
   3c0d0:	jne    3c070 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11d0>
   3c0d2:	test   $0x1,%dl
   3c0d5:	je     3c10b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x126b>
   3c0d7:	add    %r9,%rcx
   3c0da:	shl    $0x6,%rsi
   3c0de:	movups 0x2c(%rdi),%xmm0
   3c0e2:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3c0e7:	movdqa (%rdi),%xmm0
   3c0eb:	movdqa 0x10(%rdi),%xmm1
   3c0f0:	movdqa 0x20(%rdi),%xmm2
   3c0f5:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3c0fb:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3c101:	movdqa %xmm0,(%rcx,%rsi,1)
   3c106:	mov    %r15d,0x3c(%rcx,%rsi,1)
   3c10b:	mov    %edx,%ecx
   3c10d:	shl    $0x5,%ecx
   3c110:	add    %ecx,%eax
   3c112:	mov    %eax,0x28e0(%rsp)
   3c119:	mov    0x48(%rsp),%rdx
   3c11e:	mov    %rdx,%rax
   3c121:	shl    $0x5,%rax
   3c125:	lea    (%rdx,%rdx,1),%rcx
   3c129:	sub    %rax,%rcx
   3c12c:	mov    0x28(%rsp),%rbp
   3c131:	add    %rcx,%rbp
   3c134:	lea    (%rdx,%rdx,1),%eax
   3c137:	shl    $0x5,%edx
   3c13a:	sub    %eax,%edx
   3c13c:	add    %edx,%r8d
   3c13f:	mov    0x78(%rsp),%rbx
   3c144:	mov    %r8d,0x8(%rbx)
   3c148:	test   %rbp,%rbp
   3c14b:	mov    $0x88888889,%r15d
   3c151:	lea    -0x207d8(%rip),%r13        # 1b980 <c30walk::ctor2d::PATTERNS>
   3c158:	mov    0x90(%rsp),%r12
   3c160:	je     3c1d0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3c162:	mov    0xc(%rbx),%ecx
   3c165:	mov    %r8d,%edi
   3c168:	imul   %r15,%rdi
   3c16c:	mov    %rdi,%rdx
   3c16f:	shr    $0x24,%rdx
   3c173:	mov    %edx,%eax
   3c175:	shl    $0x5,%eax
   3c178:	sub    %edx,%eax
   3c17a:	sub    %edx,%eax
   3c17c:	lea    0x1e(%rax),%esi
   3c17f:	cmp    %ecx,%esi
   3c181:	cmovb  %esi,%ecx
   3c184:	shr    $0x2a,%rdi
   3c188:	mov    0xa8(%rsp),%rsi
   3c190:	cmp    %rsi,%rdi
   3c193:	jae    3c85c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19bc>
   3c199:	mov    %rdx,%rsi
   3c19c:	shl    $0x6,%rsi
   3c1a0:	add    0xc0(%rsp),%rsi
   3c1a8:	mov    0xa0(%rsp),%r9
   3c1b0:	mov    (%r9,%rdi,8),%rdi
   3c1b4:	bt     %rdx,%rdi
   3c1b8:	jb     3bae0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc40>
   3c1be:	mov    $0xffffffff,%edi
   3c1c3:	jmp    3baec <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc4c>
   3c1c8:	nopl   0x0(%rax,%rax,1)
   3c1d0:	mov    (%rsp),%r10
   3c1d4:	mov    0x38(%rsp),%r15
   3c1d9:	mov    0x30(%rsp),%r12
   3c1de:	mov    0x20(%rsp),%rax
   3c1e3:	mov    0x54(%rsp),%edx
   3c1e7:	mov    0x70(%rsp),%rcx
   3c1ec:	jmp    3b932 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa92>
   3c1f1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c200:	add    %rbp,%rdi
   3c203:	xor    %ebp,%ebp
   3c205:	data16 cs nopw 0x0(%rax,%rax,1)
   3c210:	movzbl (%rdi,%rbp,1),%r15d
   3c215:	mov    0xd8(%rsp,%r15,8),%rax
   3c21d:	cmp    0x8d8(%rsp,%r15,8),%rax
   3c225:	je     3c241 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13a1>
   3c227:	lea    (%r12,%rbp,1),%rcx
   3c22b:	mov    %cx,(%rax)
   3c22e:	addq   $0x2,0xd8(%rsp,%r15,8)
   3c237:	inc    %rbp
   3c23a:	cmp    %rbp,%r14
   3c23d:	jne    3c210 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1370>
   3c23f:	jmp    3c280 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13e0>
   3c241:	mov    %rdi,0x8(%rsp)
   3c246:	lea    0x98(%rsp),%rdi
   3c24e:	mov    %r15,%rsi
   3c251:	call   95290 <<c30walk::ctor2d::W<false>>::refill>
   3c256:	test   %al,%al
   3c258:	je     3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3c25e:	mov    0xd8(%rsp,%r15,8),%rax
   3c266:	mov    (%rsp),%r10
   3c26a:	mov    0x8(%rsp),%rdi
   3c26f:	jmp    3c227 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1387>
   3c271:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c280:	cmp    %rbx,0x20(%rsp)
   3c285:	jb     3c799 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18f9>
   3c28b:	cmp    $0xf,%r13
   3c28f:	ja     3b8f0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3c295:	add    0x40(%rsp),%rbx
   3c29a:	mov    %rbx,%r13
   3c29d:	sub    %r14,%r13
   3c2a0:	add    $0x10,%r13
   3c2a4:	lea    (%r12,%r14,1),%ebp
   3c2a8:	add    $0x2,%ebp
   3c2ab:	nopl   0x0(%rax,%rax,1)
   3c2b0:	movzbl (%rbx),%r15d
   3c2b4:	mov    0xd8(%rsp,%r15,8),%rax
   3c2bc:	cmp    0x8d8(%rsp,%r15,8),%rax
   3c2c4:	je     3c2e1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1441>
   3c2c6:	mov    %bp,(%rax)
   3c2c9:	addq   $0x2,0xd8(%rsp,%r15,8)
   3c2d2:	inc    %rbx
   3c2d5:	inc    %ebp
   3c2d7:	cmp    %r13,%rbx
   3c2da:	jne    3c2b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1410>
   3c2dc:	jmp    3b8f0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3c2e1:	lea    0x98(%rsp),%rdi
   3c2e9:	mov    %r15,%rsi
   3c2ec:	call   95290 <<c30walk::ctor2d::W<false>>::refill>
   3c2f1:	test   %al,%al
   3c2f3:	je     3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3c2f9:	mov    0xd8(%rsp,%r15,8),%rax
   3c301:	mov    (%rsp),%r10
   3c305:	jmp    3c2c6 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1426>
   3c307:	nopw   0x0(%rax,%rax,1)
   3c310:	lea    (%r15,%rdi,1),%rax
   3c314:	mov    0x60(%rsp),%r8
   3c319:	cmp    %r8,%rax
   3c31c:	jne    3c363 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14c3>
   3c31e:	cmp    $0x10,%r9
   3c322:	jne    3c3e4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1544>
   3c328:	mov    0x28d8(%rsp),%rax
   3c330:	lea    0x10(%rax),%rcx
   3c334:	cmp    $0x1d,%rcx
   3c338:	mov    0x38(%rsp),%r15
   3c33d:	mov    0x30(%rsp),%r12
   3c342:	jbe    3c47a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15da>
   3c348:	add    $0xfffffffffffffff2,%rax
   3c34c:	mov    %rax,0x28d8(%rsp)
   3c354:	mov    0x28e0(%rsp),%eax
   3c35b:	add    $0x12,%eax
   3c35e:	jmp    3c48c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15ec>
   3c363:	add    $0x2,%r13d
   3c367:	movzwl %r13w,%r13d
   3c36b:	nopl   0x0(%rax,%rax,1)
   3c370:	movzbl (%r12,%r15,1),%ebx
   3c375:	mov    0xd8(%rsp,%rbx,8),%rax
   3c37d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3c385:	je     3c3a4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1504>
   3c387:	lea    (%r15,%r13,1),%rcx
   3c38b:	mov    %cx,(%rax)
   3c38e:	addq   $0x2,0xd8(%rsp,%rbx,8)
   3c397:	inc    %r15
   3c39a:	cmp    %r15,%r9
   3c39d:	jne    3c370 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14d0>
   3c39f:	jmp    3c31e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x147e>
   3c3a4:	mov    %r9,0x10(%rsp)
   3c3a9:	mov    %rdi,%r14
   3c3ac:	lea    0x98(%rsp),%rdi
   3c3b4:	mov    %rbx,%rsi
   3c3b7:	call   95290 <<c30walk::ctor2d::W<false>>::refill>
   3c3bc:	test   %al,%al
   3c3be:	je     3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3c3c4:	mov    0xd8(%rsp,%rbx,8),%rax
   3c3cc:	mov    (%rsp),%r10
   3c3d0:	mov    0x40(%rsp),%rbp
   3c3d5:	mov    0x60(%rsp),%r8
   3c3da:	mov    %r14,%rdi
   3c3dd:	mov    0x10(%rsp),%r9
   3c3e2:	jmp    3c387 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14e7>
   3c3e4:	mov    0x28e0(%rsp),%eax
   3c3eb:	mov    0x28d8(%rsp),%rcx
   3c3f3:	inc    %rdi
   3c3f6:	cmp    %rdi,%r8
   3c3f9:	mov    0x38(%rsp),%r15
   3c3fe:	mov    0x30(%rsp),%r12
   3c403:	je     3c455 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15b5>
   3c405:	mov    %r9d,%edx
   3c408:	and    $0xffffe,%edx
   3c40e:	xchg   %ax,%ax
   3c410:	lea    0x1(%rcx),%rsi
   3c414:	add    $0xffffffffffffffe3,%rcx
   3c418:	xor    %edi,%edi
   3c41a:	cmp    $0x1e,%rsi
   3c41e:	setae  %dil
   3c422:	cmovb  %rsi,%rcx
   3c426:	lea    (%rax,%rdi,2),%eax
   3c429:	lea    0x1(%rcx),%rsi
   3c42d:	add    $0xffffffffffffffe3,%rcx
   3c431:	xor    %edi,%edi
   3c433:	cmp    $0x1e,%rsi
   3c437:	setae  %dil
   3c43b:	cmovb  %rsi,%rcx
   3c43f:	lea    (%rax,%rdi,2),%eax
   3c442:	add    $0x2,%eax
   3c445:	add    $0xfffffffffffffffe,%rdx
   3c449:	jne    3c410 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1570>
   3c44b:	test   $0x1,%r9b
   3c44f:	mov    (%rsp),%r10
   3c453:	je     3c470 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15d0>
   3c455:	lea    0x1(%rcx),%rdx
   3c459:	add    $0xffffffffffffffe3,%rcx
   3c45d:	xor    %esi,%esi
   3c45f:	cmp    $0x1e,%rdx
   3c463:	setae  %sil
   3c467:	cmovb  %rdx,%rcx
   3c46b:	lea    (%rax,%rsi,2),%eax
   3c46e:	inc    %eax
   3c470:	mov    %rcx,0x28d8(%rsp)
   3c478:	jmp    3c48c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15ec>
   3c47a:	mov    %rcx,0x28d8(%rsp)
   3c482:	mov    0x28e0(%rsp),%eax
   3c489:	add    $0x10,%eax
   3c48c:	mov    %eax,0x28e0(%rsp)
   3c493:	mov    %r8,%rdi
   3c496:	cmp    %r10,%r8
   3c499:	jb     3b66c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3c49f:	jmp    3c4ae <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3c4a1:	mov    %rbx,%rdi
   3c4a4:	call   e8ca0 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3c4a9:	jmp    3b7d4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x934>
   3c4ae:	mov    $0x9,%eax
   3c4b3:	lea    0x10fc(%rsp),%rcx
   3c4bb:	nopl   0x0(%rax,%rax,1)
   3c4c0:	cmp    $0x109,%rax
   3c4c6:	je     3c530 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1690>
   3c4c8:	mov    0x90(%rsp,%rax,8),%rdx
   3c4d0:	sub    -0x24(%rcx),%rdx
   3c4d4:	shr    $1,%rdx
   3c4d7:	add    -0x1c(%rcx),%edx
   3c4da:	cmp    -0x18(%rcx),%edx
   3c4dd:	jne    3c4fd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3c4df:	mov    0x98(%rsp,%rax,8),%rdx
   3c4e7:	sub    -0xc(%rcx),%rdx
   3c4eb:	shr    $1,%rdx
   3c4ee:	add    -0x4(%rcx),%edx
   3c4f1:	add    $0x2,%rax
   3c4f5:	cmp    (%rcx),%edx
   3c4f7:	lea    0x30(%rcx),%rcx
   3c4fb:	je     3c4c0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1620>
   3c4fd:	mov    0x4520(%rsp),%rax
   3c505:	movq   $0x0,0x10(%rax)
   3c50d:	xor    %eax,%eax
   3c50f:	add    $0x44e8,%rsp
   3c516:	pop    %rbx
   3c517:	pop    %r12
   3c519:	pop    %r13
   3c51b:	pop    %r14
   3c51d:	pop    %r15
   3c51f:	pop    %rbp
   3c520:	ret
   3c521:	lea    0xf7ff0(%rip),%rcx        # 134518 <__frame_dummy_init_array_entry+0x3a58>
   3c528:	mov    %r10,%rdx
   3c52b:	call   dae80 <core::slice::index::slice_index_fail>
   3c530:	mov    $0x8,%r14d
   3c536:	mov    $0x1040,%r15d
   3c53c:	lea    0x98(%rsp),%rbx
   3c544:	mov    0x88(%rsp),%r12
   3c54c:	mov    (%rsp),%rdx
   3c550:	mov    0x4520(%rsp),%rsi
   3c558:	mov    0x80(%rsp),%r13
   3c560:	jmp    3c5b2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1712>
   3c562:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c570:	sub    %rcx,%rax
   3c573:	shr    $1,%rax
   3c576:	add    %eax,0xa0(%rsp,%r15,1)
   3c57e:	movq   $0x0,0x98(%rsp,%r15,1)
   3c58a:	movq   $0x0,0x98(%rsp,%r14,8)
   3c596:	movq   $0x0,0x898(%rsp,%r14,8)
   3c5a2:	inc    %r14
   3c5a5:	add    $0x18,%r15
   3c5a9:	cmp    $0x108,%r14
   3c5b0:	je     3c5f4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1754>
   3c5b2:	mov    0x98(%rsp,%r14,8),%rcx
   3c5ba:	mov    %rcx,%rax
   3c5bd:	cmp    0x98(%rsp,%r15,1),%rcx
   3c5c5:	je     3c570 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16d0>
   3c5c7:	lea    -0x8(%r14),%rsi
   3c5cb:	mov    %rbx,%rdi
   3c5ce:	call   94c80 <<c30walk::ctor2d::W<false>>::finish>
   3c5d3:	mov    0x4520(%rsp),%rsi
   3c5db:	mov    (%rsp),%rdx
   3c5df:	mov    0x98(%rsp,%r14,8),%rax
   3c5e7:	mov    0x98(%rsp,%r15,1),%rcx
   3c5ef:	jmp    3c570 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16d0>
   3c5f4:	xor    %ecx,%ecx
   3c5f6:	mov    0x8(%r12),%rbx
   3c5fb:	mov    0x58(%rsp),%r14
   3c600:	test   %r14,%r14
   3c603:	setne  %al
   3c606:	je     3c652 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17b2>
   3c608:	mov    %al,%cl
   3c60a:	mov    %r14,%rax
   3c60d:	sub    %rcx,%rax
   3c610:	imul   $0xffffffffffffffe2,%rax,%rcx
   3c614:	add    %rdx,%rcx
   3c617:	cmp    $0x1e,%rcx
   3c61b:	jae    3c652 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17b2>
   3c61d:	lea    (%rbx,%r14,4),%rcx
   3c621:	lea    (%rcx,%rdx,2),%rdi
   3c625:	add    $0xfffffffffffffffc,%rdi
   3c629:	lea    (%rax,%rax,1),%rcx
   3c62d:	shl    $0x5,%rax
   3c631:	sub    %rcx,%rax
   3c634:	sub    %rdx,%rax
   3c637:	lea    0x3c(,%rax,2),%rdx
   3c63f:	mov    $0xff,%esi
   3c644:	call   *0xfa8f6(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   3c64a:	mov    0x4520(%rsp),%rsi
   3c652:	shl    $0x6,%r14
   3c656:	pcmpeqd %xmm0,%xmm0
   3c65a:	movdqu %xmm0,0x2c(%rbx,%r14,1)
   3c661:	movdqa %xmm0,0x20(%rbx,%r14,1)
   3c668:	movdqa %xmm0,0x10(%rbx,%r14,1)
   3c66f:	movdqa %xmm0,(%rbx,%r14,1)
   3c675:	movl   $0x100fffff,0x3c(%rbx,%r14,1)
   3c67e:	mov    0x8(%r13),%rdx
   3c682:	mov    0x10(%r13),%rcx
   3c686:	mov    0x8(%rsi),%r8
   3c68a:	mov    0x10(%rsi),%r9
   3c68e:	lea    0x98(%rsp),%rdi
   3c696:	mov    %rbx,%rsi
   3c699:	call   9c750 <c30walk::ctor::canonicalise>
   3c69e:	mov    0x68(%rsp),%rax
   3c6a3:	mov    %rax,0x10(%r12)
   3c6a8:	mov    $0x1,%al
   3c6aa:	jmp    3c50f <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166f>
   3c6af:	lea    0xf6a6a(%rip),%rcx        # 133120 <__frame_dummy_init_array_entry+0x2660>
   3c6b6:	mov    $0x1e,%edx
   3c6bb:	xor    %edi,%edi
   3c6bd:	call   dae80 <core::slice::index::slice_index_fail>
   3c6c2:	test   %rax,%rax
   3c6c5:	je     3c6d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1837>
   3c6c7:	mov    0x8(%r9),%rdi
   3c6cb:	mov    %r9,%r14
   3c6ce:	call   *0xfa87c(%rip)        # 136f50 <free@GLIBC_2.2.5>
   3c6d4:	mov    %r14,%r9
   3c6d7:	movq   $0x0,(%r9)
   3c6de:	movq   $0x40,0x8(%r9)
   3c6e6:	movq   $0x0,0x10(%r9)
   3c6ee:	mov    $0x40,%edx
   3c6f3:	mov    %r9,%rdi
   3c6f6:	mov    0x68(%rsp),%rsi
   3c6fb:	mov    %r9,%r14
   3c6fe:	call   91b50 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c703:	mov    %r14,%r9
   3c706:	cmp    $0xffffffffffffffff,%rax
   3c70a:	mov    0x58(%rsp),%r8
   3c70f:	je     3b0c3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x223>
   3c715:	jmp    3c767 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18c7>
   3c717:	mov    %r9,%r14
   3c71a:	test   %rax,%rax
   3c71d:	je     3c731 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1891>
   3c71f:	mov    0x8(%rdi),%rdi
   3c723:	call   *0xfa827(%rip)        # 136f50 <free@GLIBC_2.2.5>
   3c729:	mov    0x4520(%rsp),%rdi
   3c731:	movq   $0x0,(%rdi)
   3c738:	movq   $0x40,0x8(%rdi)
   3c740:	movq   $0x0,0x10(%rdi)
   3c748:	mov    $0x80,%edx
   3c74d:	mov    %r15,%rsi
   3c750:	call   91b50 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c755:	cmp    $0xffffffffffffffff,%rax
   3c759:	mov    %r14,%r9
   3c75c:	mov    0x58(%rsp),%r8
   3c761:	je     3b0d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x237>
   3c767:	mov    %rax,%rdi
   3c76a:	mov    %rdx,%rsi
   3c76d:	call   d860b <alloc::raw_vec::handle_error>
   3c772:	lea    0xf69d7(%rip),%rcx        # 133150 <__frame_dummy_init_array_entry+0x2690>
   3c779:	mov    %r10,%rdx
   3c77c:	mov    %rbx,%rsi
   3c77f:	call   dae80 <core::slice::index::slice_index_fail>
   3c784:	lea    0xf7d8d(%rip),%rcx        # 134518 <__frame_dummy_init_array_entry+0x3a58>
   3c78b:	mov    %r10,%rdx
   3c78e:	mov    %rsi,%rdi
   3c791:	mov    %rax,%rsi
   3c794:	call   dae80 <core::slice::index::slice_index_fail>
   3c799:	lea    0xf6998(%rip),%rcx        # 133138 <__frame_dummy_init_array_entry+0x2678>
   3c7a0:	mov    %r10,%rdx
   3c7a3:	mov    %rbx,%rdi
   3c7a6:	mov    0x20(%rsp),%rsi
   3c7ab:	call   dae80 <core::slice::index::slice_index_fail>
   3c7b0:	lea    0xf4521(%rip),%rdx        # 130cd8 <__frame_dummy_init_array_entry+0x218>
   3c7b7:	mov    %rax,%rdi
   3c7ba:	mov    0x8(%rsp),%rsi
   3c7bf:	call   da0a3 <core::panicking::panic_bounds_check>
   3c7c4:	lea    0xf6a15(%rip),%rdx        # 1331e0 <__frame_dummy_init_array_entry+0x2720>
   3c7cb:	mov    %rax,%rdi
   3c7ce:	mov    %rcx,%rsi
   3c7d1:	call   da0a3 <core::panicking::panic_bounds_check>
   3c7d6:	lea    0xf69eb(%rip),%rdx        # 1331c8 <__frame_dummy_init_array_entry+0x2708>
   3c7dd:	mov    $0x1e,%esi
   3c7e2:	mov    %r12,%rdi
   3c7e5:	call   da0a3 <core::panicking::panic_bounds_check>
   3c7ea:	lea    0xf69bf(%rip),%rdx        # 1331b0 <__frame_dummy_init_array_entry+0x26f0>
   3c7f1:	mov    $0x1e,%esi
   3c7f6:	mov    %r12,%rdi
   3c7f9:	call   da0a3 <core::panicking::panic_bounds_check>
   3c7fe:	mov    $0x10,%edx
   3c803:	mov    $0x4,%ecx
   3c808:	mov    $0x400,%r8d
   3c80e:	mov    %r15,%rdi
   3c811:	xor    %esi,%esi
   3c813:	mov    %r9,%r14
   3c816:	call   68910 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c81b:	mov    %r14,%r9
   3c81e:	jmp    3b15b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2bb>
   3c823:	mov    $0x8,%ecx
   3c828:	mov    $0x8,%r8d
   3c82e:	mov    %rbp,%rdi
   3c831:	xor    %esi,%esi
   3c833:	mov    %r15,%rdx
   3c836:	mov    %r9,%r14
   3c839:	call   68910 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c83e:	mov    %r14,%r9
   3c841:	mov    0x10(%rbp),%r14
   3c845:	mov    0x8(%rbp),%r12
   3c849:	lea    (%r12,%r14,8),%rdi
   3c84d:	cmp    $0x1,%r15
   3c851:	jne    3b109 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x269>
   3c857:	jmp    3b131 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3c85c:	lea    0xf4475(%rip),%rdx        # 130cd8 <__frame_dummy_init_array_entry+0x218>
   3c863:	call   da0a3 <core::panicking::panic_bounds_check>
   3c868:	lea    0xf6899(%rip),%rdx        # 133108 <__frame_dummy_init_array_entry+0x2648>
   3c86f:	mov    $0x1e,%esi
   3c874:	call   da0a3 <core::panicking::panic_bounds_check>
   3c879:	lea    0xf6918(%rip),%rdx        # 133198 <__frame_dummy_init_array_entry+0x26d8>
   3c880:	mov    $0x1e,%esi
   3c885:	call   da0a3 <core::panicking::panic_bounds_check>
   3c88a:	lea    0xf67ff(%rip),%rdx        # 133090 <__frame_dummy_init_array_entry+0x25d0>
   3c891:	mov    0x8(%rsp),%rsi
   3c896:	call   da0a3 <core::panicking::panic_bounds_check>
   3c89b:	lea    0xf4436(%rip),%rdx        # 130cd8 <__frame_dummy_init_array_entry+0x218>
   3c8a2:	mov    %r12,%rdi
   3c8a5:	call   da0a3 <core::panicking::panic_bounds_check>
