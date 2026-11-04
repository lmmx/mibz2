# <c30walk::ctor2::Ctor2>::build::<false>
# size 0x18e5

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003e010 <<c30walk::ctor2::Ctor2>::build::<false>>:
   3e010:	push   %rbp
   3e011:	push   %r15
   3e013:	push   %r14
   3e015:	push   %r13
   3e017:	push   %r12
   3e019:	push   %rbx
   3e01a:	sub    $0x1000,%rsp
   3e021:	movq   $0x0,(%rsp)
   3e029:	sub    $0x1000,%rsp
   3e030:	movq   $0x0,(%rsp)
   3e038:	sub    $0x1000,%rsp
   3e03f:	movq   $0x0,(%rsp)
   3e047:	sub    $0x1000,%rsp
   3e04e:	movq   $0x0,(%rsp)
   3e056:	sub    $0x1000,%rsp
   3e05d:	movq   $0x0,(%rsp)
   3e065:	sub    $0x4e8,%rsp
   3e06c:	mov    %r8d,0x1c(%rsp)
   3e071:	mov    0x5520(%rsp),%rax
   3e079:	movq   $0x0,0x10(%r9)
   3e081:	movq   $0x0,0x10(%rax)
   3e089:	cmp    $0xeffe2,%rdx
   3e090:	ja     3f587 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3e096:	mov    %rcx,%r15
   3e099:	mov    %rdx,%r13
   3e09c:	pxor   %xmm0,%xmm0
   3e0a0:	xor    %eax,%eax
   3e0a2:	pxor   %xmm2,%xmm2
   3e0a6:	pxor   %xmm1,%xmm1
   3e0aa:	nopw   0x0(%rax,%rax,1)
   3e0b0:	movq   (%r15,%rax,4),%xmm3
   3e0b6:	movq   0x8(%r15,%rax,4),%xmm4
   3e0bd:	punpckldq %xmm0,%xmm3
   3e0c1:	paddq  %xmm2,%xmm3
   3e0c5:	punpckldq %xmm0,%xmm4
   3e0c9:	paddq  %xmm1,%xmm4
   3e0cd:	movq   0x10(%r15,%rax,4),%xmm2
   3e0d4:	movq   0x18(%r15,%rax,4),%xmm1
   3e0db:	punpckldq %xmm0,%xmm2
   3e0df:	paddq  %xmm3,%xmm2
   3e0e3:	punpckldq %xmm0,%xmm1
   3e0e7:	paddq  %xmm4,%xmm1
   3e0eb:	add    $0x8,%rax
   3e0ef:	cmp    $0x100,%rax
   3e0f5:	jne    3e0b0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   3e0f7:	paddq  %xmm2,%xmm1
   3e0fb:	pshufd $0xee,%xmm1,%xmm0
   3e100:	paddq  %xmm1,%xmm0
   3e104:	movq   %xmm0,%rax
   3e109:	cmp    %r13,%rax
   3e10c:	jne    3f587 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3e112:	pxor   %xmm2,%xmm2
   3e116:	xor    %eax,%eax
   3e118:	movdqa -0x2e960(%rip),%xmm3        # f7c0 <GCC_except_table976+0xae8>
   3e120:	pxor   %xmm1,%xmm1
   3e124:	pxor   %xmm0,%xmm0
   3e128:	nopl   0x0(%rax,%rax,1)
   3e130:	movq   (%r15,%rax,4),%xmm4
   3e136:	movq   0x8(%r15,%rax,4),%xmm5
   3e13d:	pshufd $0xd4,%xmm4,%xmm4
   3e142:	pcmpeqd %xmm2,%xmm4
   3e146:	pandn  %xmm3,%xmm4
   3e14a:	paddq  %xmm1,%xmm4
   3e14e:	pshufd $0xd4,%xmm5,%xmm5
   3e153:	pcmpeqd %xmm2,%xmm5
   3e157:	pandn  %xmm3,%xmm5
   3e15b:	paddq  %xmm0,%xmm5
   3e15f:	movq   0x10(%r15,%rax,4),%xmm0
   3e166:	movq   0x18(%r15,%rax,4),%xmm6
   3e16d:	pshufd $0xd4,%xmm0,%xmm1
   3e172:	pcmpeqd %xmm2,%xmm1
   3e176:	pandn  %xmm3,%xmm1
   3e17a:	paddq  %xmm4,%xmm1
   3e17e:	pshufd $0xd4,%xmm6,%xmm0
   3e183:	pcmpeqd %xmm2,%xmm0
   3e187:	pandn  %xmm3,%xmm0
   3e18b:	paddq  %xmm5,%xmm0
   3e18f:	add    $0x8,%rax
   3e193:	cmp    $0x100,%rax
   3e199:	jne    3e130 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
   3e19b:	mov    $0x88888889,%ebp
   3e1a0:	imul   %r13,%rbp
   3e1a4:	shr    $0x24,%rbp
   3e1a8:	imul   $0xeeeeeeef,%r13d,%eax
   3e1af:	ror    $1,%eax
   3e1b1:	cmp    $0x8888889,%eax
   3e1b6:	sbb    $0xffffffffffffffff,%rbp
   3e1ba:	mov    %rbp,%rbx
   3e1bd:	cmpl   $0x0,0x1c(%rsp)
   3e1c2:	jne    3e218 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3e1c4:	test   %r13,%r13
   3e1c7:	je     3e216 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   3e1c9:	paddq  %xmm1,%xmm0
   3e1cd:	pshufd $0xee,%xmm0,%xmm1
   3e1d2:	paddq  %xmm0,%xmm1
   3e1d6:	movq   %xmm1,%rax
   3e1db:	lea    -0x1(%r13),%ecx
   3e1df:	mov    $0x88888889,%edx
   3e1e4:	imul   %rcx,%rdx
   3e1e8:	shr    $0x23,%rdx
   3e1ec:	and    $0xfffffffe,%edx
   3e1ef:	lea    (%rdx,%r13,1),%ecx
   3e1f3:	dec    %ecx
   3e1f5:	shr    $1,%ecx
   3e1f7:	mov    $0x80010003,%ebx
   3e1fc:	imul   %rcx,%rbx
   3e200:	shr    $0x2e,%rbx
   3e204:	inc    %ebx
   3e206:	imul   %rax,%rbx
   3e20a:	dec    %rbx
   3e20d:	cmp    %rbp,%rbx
   3e210:	cmovae %rbp,%rbx
   3e214:	jmp    3e218 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3e216:	xor    %ebx,%ebx
   3e218:	mov    (%r9),%rax
   3e21b:	lea    0x1(%rbp),%rcx
   3e21f:	mov    %rcx,0x80(%rsp)
   3e227:	cmp    %rbp,%rax
   3e22a:	mov    %rsi,0x28(%rsp)
   3e22f:	jbe    3f6ef <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
   3e235:	mov    0x5520(%rsp),%rax
   3e23d:	mov    (%rax),%rax
   3e240:	cmp    %rbx,%rax
   3e243:	jb     3f744 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
   3e249:	movq   $0x0,0x10(%rdi)
   3e251:	lea    0x3f(%rbp),%rbx
   3e255:	shr    $0x6,%rbx
   3e259:	mov    %r13,0x8(%rsp)
   3e25e:	mov    %rdi,0x58(%rsp)
   3e263:	mov    %rbp,0x60(%rsp)
   3e268:	je     3e2c2 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
   3e26a:	cmp    (%rdi),%rbx
   3e26d:	ja     3f873 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   3e273:	mov    %r9,%r13
   3e276:	xor    %r14d,%r14d
   3e279:	mov    0x8(%rdi),%r12
   3e27d:	lea    (%r12,%r14,8),%rdi
   3e281:	cmp    $0x1,%rbx
   3e285:	je     3e2a9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3e287:	lea    -0x8(,%rbx,8),%rdx
   3e28f:	xor    %esi,%esi
   3e291:	call   *0xf7bb9(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3e297:	lea    (%r14,%rbx,1),%rax
   3e29b:	lea    (%r12,%rax,8),%rdi
   3e29f:	add    $0xfffffffffffffff8,%rdi
   3e2a3:	add    %rbx,%r14
   3e2a6:	dec    %r14
   3e2a9:	mov    %r13,%r9
   3e2ac:	movq   $0x0,(%rdi)
   3e2b3:	inc    %r14
   3e2b6:	mov    0x8(%rsp),%r13
   3e2bb:	mov    0x58(%rsp),%rdi
   3e2c0:	jmp    3e2c5 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
   3e2c2:	xor    %r14d,%r14d
   3e2c5:	lea    0x18(%rdi),%r12
   3e2c9:	mov    %r14,0x10(%rdi)
   3e2cd:	movq   $0x0,0x28(%rdi)
   3e2d5:	cmpq   $0xf,0x18(%rdi)
   3e2da:	jbe    3f848 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   3e2e0:	lea    0x30(%rdi),%rax
   3e2e4:	mov    %rax,(%rsp)
   3e2e8:	xor    %eax,%eax
   3e2ea:	pxor   %xmm0,%xmm0
   3e2ee:	movabs $0xffffffff00000000,%rcx
   3e2f8:	nopl   0x0(%rax,%rax,1)
   3e300:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   3e309:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3e312:	mov    %rcx,0x2d08(%rsp,%rax,1)
   3e31a:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   3e323:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3e32c:	mov    %rcx,0x2d30(%rsp,%rax,1)
   3e334:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3e33d:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   3e346:	mov    %rcx,0x2d58(%rsp,%rax,1)
   3e34e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   3e357:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   3e360:	mov    %rcx,0x2d80(%rsp,%rax,1)
   3e368:	add    $0xa0,%rax
   3e36e:	cmp    $0x2800,%rax
   3e374:	jne    3e300 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   3e376:	mov    %r12,%r14
   3e379:	lea    0xc(%r15),%rax
   3e37d:	xor    %edx,%edx
   3e37f:	mov    $0x25,%ecx
   3e384:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e390:	mov    -0xc(%rax),%esi
   3e393:	add    %edx,%esi
   3e395:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   3e39c:	mov    %esi,0x2c70(%rsp,%rcx,4)
   3e3a3:	mov    -0x8(%rax),%edx
   3e3a6:	add    %esi,%edx
   3e3a8:	mov    %esi,0x2c94(%rsp,%rcx,4)
   3e3af:	mov    %edx,0x2c98(%rsp,%rcx,4)
   3e3b6:	mov    -0x4(%rax),%esi
   3e3b9:	add    %edx,%esi
   3e3bb:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   3e3c2:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   3e3c9:	mov    (%rax),%edx
   3e3cb:	add    %esi,%edx
   3e3cd:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3e3d4:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3e3db:	add    $0x28,%rcx
   3e3df:	add    $0x10,%rax
   3e3e3:	cmp    $0xa25,%rcx
   3e3ea:	jne    3e390 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
   3e3ec:	mov    %r9,0x78(%rsp)
   3e3f1:	mov    0x8(%r9),%rbx
   3e3f5:	mov    0x8(%rdi),%rbp
   3e3f9:	mov    0x10(%rdi),%r12
   3e3fd:	lea    0xd8(%rsp),%rdi
   3e405:	lea    0x2ce8(%rsp),%rsi
   3e40d:	mov    $0x2800,%edx
   3e412:	call   *0xf7a58(%rip)        # 135e70 <memcpy@GLIBC_2.14>
   3e418:	mov    %rbp,%r8
   3e41b:	mov    %r12,%rsi
   3e41e:	mov    %rbx,0xc0(%rsp)
   3e426:	mov    0x60(%rsp),%rax
   3e42b:	mov    %rax,0xc8(%rsp)
   3e433:	mov    %r13,0xd0(%rsp)
   3e43b:	mov    0x5520(%rsp),%rax
   3e443:	mov    %rax,0x98(%rsp)
   3e44b:	mov    %rbp,0xa0(%rsp)
   3e453:	mov    %r12,0xa8(%rsp)
   3e45b:	mov    %r14,0xb0(%rsp)
   3e463:	movl   $0x0,0x28e0(%rsp)
   3e46e:	movq   $0x0,0x28d8(%rsp)
   3e47a:	mov    (%rsp),%rax
   3e47e:	mov    %rax,0xb8(%rsp)
   3e486:	mov    $0xffffffffffffffff,%rcx
   3e48d:	xor    %ebp,%ebp
   3e48f:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3e499:	movabs $0x888888888888889,%r10
   3e4a3:	pxor   %xmm0,%xmm0
   3e4a7:	pcmpeqd %xmm1,%xmm1
   3e4ab:	xor    %r12d,%r12d
   3e4ae:	jmp    3e566 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   3e4b3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e4c0:	mov    0x8(%rdi),%rax
   3e4c4:	mov    %r13,%rcx
   3e4c7:	shl    $0x7,%rcx
   3e4cb:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3e4d1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3e4d7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3e4dd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3e4e3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3e4e9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3e4ef:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3e4f5:	movdqa %xmm0,(%rax,%rcx,1)
   3e4fa:	mov    %r13,%rax
   3e4fd:	inc    %rax
   3e500:	mov    %rax,0x10(%rdi)
   3e504:	mov    %r14,%rax
   3e507:	shl    $0x6,%rax
   3e50b:	or     $0x10000000,%r13d
   3e512:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   3e518:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3e51e:	movdqa %xmm1,0x10(%rbx,%rax,1)
   3e524:	movdqa %xmm1,(%rbx,%rax,1)
   3e529:	mov    %r13d,0x3c(%rbx,%rax,1)
   3e52e:	mov    %r14,%rdi
   3e531:	shr    $0x6,%rdi
   3e535:	cmp    %rsi,%rdi
   3e538:	jae    3f8da <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   3e53e:	mov    $0x1,%eax
   3e543:	mov    %r14d,%ecx
   3e546:	shl    %cl,%rax
   3e549:	or     %rax,(%r8,%rdi,8)
   3e54d:	mov    %r14,%rcx
   3e550:	mov    0x8(%rsp),%r13
   3e555:	add    $0x4,%r12
   3e559:	cmp    $0x400,%r12
   3e560:	je     3e600 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
   3e566:	mov    %rbp,%rax
   3e569:	mov    (%r15,%r12,1),%ebp
   3e56d:	add    %rax,%rbp
   3e570:	cmp    %r13,%rbp
   3e573:	setae  %dil
   3e577:	mov    %rbp,%rax
   3e57a:	movabs $0x8888888888888889,%rdx
   3e584:	mul    %rdx
   3e587:	mov    %rdx,%r14
   3e58a:	shr    $0x4,%r14
   3e58e:	mov    %rbp,%rax
   3e591:	imul   %r9,%rax
   3e595:	ror    $1,%rax
   3e598:	cmp    %r10,%rax
   3e59b:	setb   %al
   3e59e:	cmp    %rcx,%r14
   3e5a1:	sete   %dl
   3e5a4:	or     %dil,%dl
   3e5a7:	or     %al,%dl
   3e5a9:	jne    3e555 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   3e5ab:	mov    0x5520(%rsp),%rdi
   3e5b3:	mov    0x10(%rdi),%r13
   3e5b7:	cmp    (%rdi),%r13
   3e5ba:	jne    3e4c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3e5c0:	mov    %rsi,(%rsp)
   3e5c4:	mov    %r8,0x10(%rsp)
   3e5c9:	call   91cf0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3e5ce:	pcmpeqd %xmm1,%xmm1
   3e5d2:	pxor   %xmm0,%xmm0
   3e5d6:	movabs $0x888888888888889,%r10
   3e5e0:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3e5ea:	mov    0x10(%rsp),%r8
   3e5ef:	mov    (%rsp),%rsi
   3e5f3:	mov    0x5520(%rsp),%rdi
   3e5fb:	jmp    3e4c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3e600:	cmpl   $0x0,0x1c(%rsp)
   3e605:	lea    0xd8(%rsp),%r8
   3e60d:	je     3e7c4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3e613:	mov    0xc8(%rsp),%r11
   3e61b:	test   %r11,%r11
   3e61e:	je     3e7c4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3e624:	mov    0x8(%rsp),%r9
   3e629:	shl    $0x20,%r9
   3e62d:	mov    0x1c(%rsp),%r10d
   3e632:	mov    0x98(%rsp),%rdi
   3e63a:	mov    0xc0(%rsp),%r13
   3e642:	xor    %r12d,%r12d
   3e645:	movabs $0xff51afd7ed558ccd,%rbx
   3e64f:	pxor   %xmm0,%xmm0
   3e653:	pcmpeqd %xmm1,%xmm1
   3e657:	jmp    3e6e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
   3e65c:	mov    0x8(%rdi),%rax
   3e660:	mov    %rbx,%rcx
   3e663:	shl    $0x7,%rcx
   3e667:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3e66d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3e673:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3e679:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3e67f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3e685:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3e68b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3e691:	movdqa %xmm0,(%rax,%rcx,1)
   3e696:	mov    %rbx,%rax
   3e699:	inc    %rax
   3e69c:	mov    %rax,0x10(%rdi)
   3e6a0:	or     $0x10000000,%ebx
   3e6a6:	movdqu %xmm1,0x2c(%r13)
   3e6ac:	movdqa %xmm1,0x20(%r13)
   3e6b2:	movdqa %xmm1,0x10(%r13)
   3e6b8:	movdqa %xmm1,0x0(%r13)
   3e6be:	mov    %ebx,0x3c(%r13)
   3e6c2:	or     %r15,(%r14,%rbp,8)
   3e6c6:	movabs $0xff51afd7ed558ccd,%rbx
   3e6d0:	inc    %r12
   3e6d3:	add    $0x40,%r13
   3e6d7:	cmp    %r12,%r11
   3e6da:	je     3e7c4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3e6e0:	mov    %r12,%rbp
   3e6e3:	shr    $0x6,%rbp
   3e6e7:	mov    0xa8(%rsp),%rsi
   3e6ef:	cmp    %rsi,%rbp
   3e6f2:	jae    3f8e6 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   3e6f8:	mov    0xa0(%rsp),%r14
   3e700:	mov    (%r14,%rbp,8),%rax
   3e704:	mov    $0x1,%r15d
   3e70a:	mov    %r12d,%ecx
   3e70d:	shl    %cl,%r15
   3e710:	bt     %r12,%rax
   3e714:	jb     3e6d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3e716:	mov    %r12,%rax
   3e719:	xor    %r9,%rax
   3e71c:	mov    %rax,%rcx
   3e71f:	shr    $0x21,%rcx
   3e723:	xor    %rax,%rcx
   3e726:	imul   %rbx,%rcx
   3e72a:	mov    %rcx,%rax
   3e72d:	shr    $0x21,%rax
   3e731:	xor    %rcx,%rax
   3e734:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3e73e:	imul   %rcx,%rax
   3e742:	mov    %rax,%rcx
   3e745:	shr    $0x21,%rcx
   3e749:	xor    %rax,%rcx
   3e74c:	mov    %rcx,%rax
   3e74f:	shr    $0x3,%rax
   3e753:	movabs $0x20c49ba5e353f7cf,%rdx
   3e75d:	mul    %rdx
   3e760:	shr    $0x4,%rdx
   3e764:	imul   $0x3e8,%rdx,%rax
   3e76b:	sub    %rax,%rcx
   3e76e:	cmp    %r10,%rcx
   3e771:	jae    3e6d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3e777:	mov    0x10(%rdi),%rbx
   3e77b:	cmp    (%rdi),%rbx
   3e77e:	jne    3e65c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3e784:	mov    %r11,(%rsp)
   3e788:	mov    %r9,0x10(%rsp)
   3e78d:	mov    %r10,0x50(%rsp)
   3e792:	mov    %rdi,0x48(%rsp)
   3e797:	call   91cf0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3e79c:	pcmpeqd %xmm1,%xmm1
   3e7a0:	pxor   %xmm0,%xmm0
   3e7a4:	mov    0x48(%rsp),%rdi
   3e7a9:	mov    0x50(%rsp),%r10
   3e7ae:	mov    0x10(%rsp),%r9
   3e7b3:	mov    (%rsp),%r11
   3e7b7:	lea    0xd8(%rsp),%r8
   3e7bf:	jmp    3e65c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3e7c4:	mov    0x8(%rsp),%r13
   3e7c9:	test   %r13,%r13
   3e7cc:	mov    0x28(%rsp),%r14
   3e7d1:	je     3f51f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3e7d7:	xor    %r11d,%r11d
   3e7da:	jmp    3e7ec <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3e7dc:	nopl   0x0(%rax)
   3e7e0:	mov    %r11,%r10
   3e7e3:	cmp    %r13,%r10
   3e7e6:	jae    3f51f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3e7ec:	mov    %r11,%rbp
   3e7ef:	mov    %r9,%r12
   3e7f2:	mov    $0x400,%edx
   3e7f7:	lea    0x28e8(%rsp),%rdi
   3e7ff:	xor    %esi,%esi
   3e801:	call   *0xf7649(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3e807:	lea    0x118(%rsp),%rax
   3e80f:	xor    %ecx,%ecx
   3e811:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e820:	mov    -0x40(%rax),%rdx
   3e824:	sub    -0x30(%rax),%rdx
   3e828:	shr    $1,%rdx
   3e82b:	add    -0x28(%rax),%edx
   3e82e:	mov    -0x18(%rax),%rsi
   3e832:	sub    -0x8(%rax),%rsi
   3e836:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3e83d:	shr    $1,%rsi
   3e840:	add    (%rax),%esi
   3e842:	mov    %esi,0x28ec(%rsp,%rcx,4)
   3e849:	add    $0x2,%rcx
   3e84d:	add    $0x50,%rax
   3e851:	cmp    $0x100,%rcx
   3e858:	jne    3e820 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
   3e85a:	mov    0xb0(%rsp),%r15
   3e862:	mov    0x10(%r15),%rbx
   3e866:	cmp    (%r15),%rbx
   3e869:	je     3f512 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
   3e86f:	mov    %rbx,%rdi
   3e872:	shl    $0xa,%rdi
   3e876:	add    0x8(%r15),%rdi
   3e87a:	mov    $0x400,%edx
   3e87f:	lea    0x28e8(%rsp),%rsi
   3e887:	call   *0xf75e3(%rip)        # 135e70 <memcpy@GLIBC_2.14>
   3e88d:	inc    %rbx
   3e890:	mov    %rbx,0x10(%r15)
   3e894:	mov    %rbp,%r11
   3e897:	lea    0xf000(%rbp),%r10
   3e89e:	cmp    %r13,%r10
   3e8a1:	cmovae %r13,%r10
   3e8a5:	lea    0x10(%rbp),%rdi
   3e8a9:	cmp    %r10,%rdi
   3e8ac:	mov    %r10,0x20(%rsp)
   3e8b1:	jbe    3e980 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   3e8b7:	lea    0xd8(%rsp),%r8
   3e8bf:	mov    %r12,%r9
   3e8c2:	mov    0x38(%rsp),%rcx
   3e8c7:	mov    %r10,%rdx
   3e8ca:	sub    %r11,%rdx
   3e8cd:	mov    %rcx,0x38(%rsp)
   3e8d2:	jbe    3e7e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   3e8d8:	mov    %r9,%rbp
   3e8db:	mov    %r14,%rax
   3e8de:	mov    0x28e0(%rsp),%r12d
   3e8e6:	mov    $0x1e,%r14d
   3e8ec:	sub    0x28d8(%rsp),%r14
   3e8f4:	cmp    %rdx,%r14
   3e8f7:	cmovae %rdx,%r14
   3e8fb:	lea    (%rax,%r11,1),%r15
   3e8ff:	test   %r14,%r14
   3e902:	je     3f380 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3e908:	xor    %r13d,%r13d
   3e90b:	nopl   0x0(%rax,%rax,1)
   3e910:	movzbl (%r15,%r13,1),%esi
   3e915:	lea    (%rsi,%rsi,4),%rcx
   3e919:	lea    (%r8,%rcx,8),%rbx
   3e91d:	mov    (%r8,%rcx,8),%rax
   3e921:	cmp    0x8(%r8,%rcx,8),%rax
   3e926:	je     3e940 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   3e928:	lea    (%r12,%r13,1),%rcx
   3e92c:	mov    %cx,(%rax)
   3e92f:	addq   $0x2,(%rbx)
   3e933:	inc    %r13
   3e936:	cmp    %r13,%r14
   3e939:	jne    3e910 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   3e93b:	jmp    3f380 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3e940:	mov    %rdx,0x10(%rsp)
   3e945:	mov    %r11,(%rsp)
   3e949:	lea    0x98(%rsp),%rdi
   3e951:	call   8a7e0 <<c30walk::ctor2::W<false>>::refill>
   3e956:	lea    0xd8(%rsp),%r8
   3e95e:	test   %al,%al
   3e960:	je     3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3e966:	mov    (%rbx),%rax
   3e969:	mov    0x20(%rsp),%r10
   3e96e:	mov    (%rsp),%r11
   3e972:	mov    0x10(%rsp),%rdx
   3e977:	jmp    3e928 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
   3e979:	nopl   0x0(%rax)
   3e980:	xor    %edx,%edx
   3e982:	lea    0xd8(%rsp),%r8
   3e98a:	mov    %r12,%r9
   3e98d:	mov    0x38(%rsp),%rcx
   3e992:	jmp    3e9f3 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   3e994:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e9a0:	mov    0x28d8(%rsp),%rax
   3e9a8:	lea    0x10(%rax),%rcx
   3e9ac:	add    $0xfffffffffffffff2,%rax
   3e9b0:	xor    %edx,%edx
   3e9b2:	cmp    $0x1e,%rcx
   3e9b6:	setae  %dl
   3e9b9:	cmovb  %rcx,%rax
   3e9bd:	mov    %rax,0x28d8(%rsp)
   3e9c5:	mov    0x28e0(%rsp),%eax
   3e9cc:	lea    0x10(%rax,%rdx,2),%eax
   3e9d0:	mov    %eax,0x28e0(%rsp)
   3e9d7:	mov    0x30(%rsp),%r11
   3e9dc:	lea    0x10(%r11),%rsi
   3e9e0:	xor    %edx,%edx
   3e9e2:	mov    %rsi,%rdi
   3e9e5:	cmp    %r10,%rsi
   3e9e8:	mov    0x28(%rsp),%r14
   3e9ed:	ja     3e8c7 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
   3e9f3:	test   $0x1,%dl
   3e9f6:	jne    3ea24 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
   3e9f8:	lea    0x8(%r11),%rsi
   3e9fc:	cmp    $0xfffffffffffffff7,%r11
   3ea00:	ja     3f7a8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3ea06:	cmp    %r13,%rsi
   3ea09:	ja     3f7a8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3ea0f:	lea    0x10(%r11),%rax
   3ea13:	cmp    %r13,%rax
   3ea16:	ja     3f7cf <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
   3ea1c:	mov    (%r14,%r11,1),%r9
   3ea20:	mov    (%r14,%rsi,1),%rcx
   3ea24:	mov    %r9,%rax
   3ea27:	xor    %rcx,%rax
   3ea2a:	mov    %r9,%rdx
   3ea2d:	rol    $0x8,%rdx
   3ea31:	xor    %r9,%rdx
   3ea34:	or     %rax,%rdx
   3ea37:	jne    3ea90 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   3ea39:	mov    %rdi,%rax
   3ea3c:	nopl   0x0(%rax)
   3ea40:	mov    %rax,%r12
   3ea43:	add    $0x10,%rax
   3ea47:	cmp    %r10,%rax
   3ea4a:	setbe  %r9b
   3ea4e:	ja     3eaf0 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
   3ea54:	lea    0x8(%r12),%rsi
   3ea59:	cmp    $0xfffffffffffffff7,%r12
   3ea5d:	ja     3f6ca <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3ea63:	cmp    %r13,%rsi
   3ea66:	ja     3f6ca <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3ea6c:	mov    (%r14,%r12,1),%rdx
   3ea70:	mov    0x8(%r14,%r12,1),%rsi
   3ea75:	cmp    %rcx,%rdx
   3ea78:	jne    3ea7f <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   3ea7a:	cmp    %rcx,%rsi
   3ea7d:	je     3ea40 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   3ea7f:	mov    %rsi,0x38(%rsp)
   3ea84:	mov    %rdx,0x68(%rsp)
   3ea89:	lea    0x10(%r12),%r10
   3ea8e:	jmp    3eaf5 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   3ea90:	mov    %rdi,0x30(%rsp)
   3ea95:	mov    0x28d8(%rsp),%rax
   3ea9d:	mov    $0x1e,%r12d
   3eaa3:	sub    %rax,%r12
   3eaa6:	cmp    $0x10,%r12
   3eaaa:	mov    $0x10,%ebx
   3eaaf:	cmovb  %r12,%rbx
   3eab3:	mov    %rbx,%r15
   3eab6:	add    %r11,%r15
   3eab9:	jb     3f7ba <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3eabf:	cmp    %r13,%r15
   3eac2:	ja     3f7ba <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3eac8:	mov    0x28e0(%rsp),%r14d
   3ead0:	cmp    $0x1e,%rax
   3ead4:	jne    3f250 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
   3eada:	cmp    %r15,0x30(%rsp)
   3eadf:	jae    3f2da <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   3eae5:	jmp    3f7e4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3eaea:	nopw   0x0(%rax,%rax,1)
   3eaf0:	lea    0x10(%r12),%r10
   3eaf5:	mov    %r12,0x30(%rsp)
   3eafa:	sub    %r11,%r12
   3eafd:	movzbl %cl,%eax
   3eb00:	mov    %rax,0x90(%rsp)
   3eb08:	lea    (%rax,%rax,4),%rdx
   3eb0c:	mov    (%r8,%rdx,8),%rax
   3eb10:	mov    0x1c(%r8,%rdx,8),%esi
   3eb15:	mov    %rax,%rdi
   3eb18:	sub    0x10(%r8,%rdx,8),%rdi
   3eb1d:	shr    $1,%rdi
   3eb20:	add    0x18(%r8,%rdx,8),%edi
   3eb25:	add    %r12,%rdi
   3eb28:	cmp    %rsi,%rdi
   3eb2b:	ja     3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3eb31:	mov    %r10,0x70(%rsp)
   3eb36:	mov    %r9d,0x44(%rsp)
   3eb3b:	lea    (%r8,%rdx,8),%rbp
   3eb3f:	movzbl %cl,%r13d
   3eb43:	mov    0x8(%rbp),%rsi
   3eb47:	mov    $0x88888889,%r14d
   3eb4d:	lea    -0x25494(%rip),%r15        # 196c0 <c30walk::ctor2::PATTERNS>
   3eb54:	mov    %rbp,0x88(%rsp)
   3eb5c:	jmp    3eb97 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   3eb5e:	xchg   %ax,%ax
   3eb60:	mov    0x3c(%rsi),%edi
   3eb63:	mov    $0xfffff,%r9d
   3eb69:	and    %r9d,%edi
   3eb6c:	sub    %r8d,%ecx
   3eb6f:	sub    %eax,%r8d
   3eb72:	mov    %r8d,%eax
   3eb75:	lea    (%rsi,%rax,2),%rax
   3eb79:	mov    %rax,0x10(%rbp)
   3eb7d:	mov    %rax,0x0(%rbp)
   3eb81:	lea    (%rax,%rcx,2),%rsi
   3eb85:	mov    %rsi,0x8(%rbp)
   3eb89:	mov    %edx,0x20(%rbp)
   3eb8c:	mov    %edi,0x24(%rbp)
   3eb8f:	lea    0xd8(%rsp),%r8
   3eb97:	sub    %rax,%rsi
   3eb9a:	shr    $1,%rsi
   3eb9d:	cmp    %rsi,%r12
   3eba0:	cmovb  %r12,%rsi
   3eba4:	test   %rsi,%rsi
   3eba7:	je     3ec61 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
   3ebad:	mov    0x28d8(%rsp),%rdi
   3ebb5:	cmp    $0x1d,%rdi
   3ebb9:	ja     3f8b8 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   3ebbf:	cmp    $0x1f,%rsi
   3ebc3:	jae    3f6dc <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
   3ebc9:	mov    0x28e0(%rsp),%ecx
   3ebd0:	mov    %rdi,%r11
   3ebd3:	shl    $0x6,%r11
   3ebd7:	lea    (%r15,%r11,1),%rdx
   3ebdb:	lea    -0x2(,%rsi,2),%r9
   3ebe3:	cmp    $0xe,%r9
   3ebe7:	jb     3ebfc <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   3ebe9:	mov    %rax,%r8
   3ebec:	sub    %r15,%r8
   3ebef:	sub    %r11,%r8
   3ebf2:	cmp    $0x10,%r8
   3ebf6:	jae    3f063 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
   3ebfc:	xor    %r10d,%r10d
   3ebff:	mov    %rdx,%r8
   3ec02:	lea    (%rdx,%rsi,2),%rdx
   3ec06:	lea    (%rax,%r10,2),%r9
   3ec0a:	nopw   0x0(%rax,%rax,1)
   3ec10:	movzwl (%r8),%r10d
   3ec14:	add    %cx,%r10w
   3ec18:	add    $0x2,%r8
   3ec1c:	mov    %r10w,(%r9)
   3ec20:	add    $0x2,%r9
   3ec24:	cmp    %rdx,%r8
   3ec27:	jne    3ec10 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
   3ec29:	lea    (%rax,%rsi,2),%rax
   3ec2d:	mov    %rax,0x0(%rbp)
   3ec31:	add    %rsi,%rdi
   3ec34:	lea    0x2(%rsi),%edx
   3ec37:	mov    %rdi,%r8
   3ec3a:	sub    $0x1e,%r8
   3ec3e:	cmovb  %rdi,%r8
   3ec42:	cmovb  %esi,%edx
   3ec45:	mov    %r8,0x28d8(%rsp)
   3ec4d:	add    %ecx,%edx
   3ec4f:	mov    %edx,0x28e0(%rsp)
   3ec56:	sub    %rsi,%r12
   3ec59:	lea    0xd8(%rsp),%r8
   3ec61:	test   %r12,%r12
   3ec64:	je     3f240 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   3ec6a:	mov    %rax,%r8
   3ec6d:	cmp    0x10(%rbp),%rax
   3ec71:	je     3ec90 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
   3ec73:	lea    0x98(%rsp),%rdi
   3ec7b:	mov    0x90(%rsp),%rsi
   3ec83:	call   8a1e0 <<c30walk::ctor2::W<false>>::finish>
   3ec88:	mov    0x0(%rbp),%r8
   3ec8c:	mov    0x10(%rbp),%rax
   3ec90:	sub    %rax,%r8
   3ec93:	shr    $1,%r8
   3ec96:	add    0x18(%rbp),%r8d
   3ec9a:	mov    %r8d,0x18(%rbp)
   3ec9e:	pxor   %xmm0,%xmm0
   3eca2:	movdqu %xmm0,0x0(%rbp)
   3eca7:	movq   $0x0,0x10(%rbp)
   3ecaf:	cmp    $0x1e,%r12
   3ecb3:	jb     3f1d4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3ecb9:	mov    %r8,%rcx
   3ecbc:	imul   %r14,%rcx
   3ecc0:	shr    $0x24,%rcx
   3ecc4:	mov    %ecx,%eax
   3ecc6:	shl    $0x5,%eax
   3ecc9:	lea    (%rcx,%rcx,1),%edx
   3eccc:	sub    %eax,%edx
   3ecce:	add    %r8d,%edx
   3ecd1:	jne    3f1d4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3ecd7:	mov    %r12,%rax
   3ecda:	movabs $0x8888888888888889,%rdx
   3ece4:	mul    %rdx
   3ece7:	shr    $0x4,%rdx
   3eceb:	mov    %ecx,%r9d
   3ecee:	cmpl   $0x0,0x1c(%rsp)
   3ecf3:	mov    %r12,0x50(%rsp)
   3ecf8:	mov    %rdx,0x48(%rsp)
   3ecfd:	je     3f020 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
   3ed03:	mov    0x28e0(%rsp),%r10d
   3ed0b:	mov    0xa8(%rsp),%rax
   3ed13:	mov    %rax,(%rsp)
   3ed17:	mov    0x98(%rsp),%r11
   3ed1f:	mov    0xa0(%rsp),%rax
   3ed27:	mov    %rax,0x10(%rsp)
   3ed2c:	mov    0xc0(%rsp),%rax
   3ed34:	mov    0x28d8(%rsp),%rbx
   3ed3c:	mov    %rbx,%rdi
   3ed3f:	shl    $0x6,%rdi
   3ed43:	add    %r15,%rdi
   3ed46:	mov    %r9,%rcx
   3ed49:	shl    $0x6,%rcx
   3ed4d:	lea    (%rax,%rcx,1),%r12
   3ed51:	add    $0x3c,%r12
   3ed55:	mov    %r10d,%r15d
   3ed58:	shl    $0x8,%r15d
   3ed5c:	or     %r13d,%r15d
   3ed5f:	xor    %ebp,%ebp
   3ed61:	jmp    3ef91 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
   3ed66:	cs nopw 0x0(%rax,%rax,1)
   3ed70:	mov    0x8(%r11),%rcx
   3ed74:	shl    $0x7,%eax
   3ed77:	mov    %r15d,(%rcx,%rax,1)
   3ed7b:	movzwl 0x2(%rdi),%esi
   3ed7f:	lea    (%r10,%rbp,1),%r14d
   3ed83:	add    %r14d,%esi
   3ed86:	shl    $0x8,%esi
   3ed89:	or     %r13d,%esi
   3ed8c:	mov    %esi,0x4(%rcx,%rax,1)
   3ed90:	movzwl 0x4(%rdi),%esi
   3ed94:	add    %r14d,%esi
   3ed97:	shl    $0x8,%esi
   3ed9a:	or     %r13d,%esi
   3ed9d:	mov    %esi,0x8(%rcx,%rax,1)
   3eda1:	movzwl 0x6(%rdi),%esi
   3eda5:	add    %r14d,%esi
   3eda8:	shl    $0x8,%esi
   3edab:	or     %r13d,%esi
   3edae:	mov    %esi,0xc(%rcx,%rax,1)
   3edb2:	movzwl 0x8(%rdi),%esi
   3edb6:	add    %r14d,%esi
   3edb9:	shl    $0x8,%esi
   3edbc:	or     %r13d,%esi
   3edbf:	mov    %esi,0x10(%rcx,%rax,1)
   3edc3:	movzwl 0xa(%rdi),%esi
   3edc7:	add    %r14d,%esi
   3edca:	shl    $0x8,%esi
   3edcd:	or     %r13d,%esi
   3edd0:	mov    %esi,0x14(%rcx,%rax,1)
   3edd4:	movzwl 0xc(%rdi),%esi
   3edd8:	add    %r14d,%esi
   3eddb:	shl    $0x8,%esi
   3edde:	or     %r13d,%esi
   3ede1:	mov    %esi,0x18(%rcx,%rax,1)
   3ede5:	movzwl 0xe(%rdi),%esi
   3ede9:	add    %r14d,%esi
   3edec:	shl    $0x8,%esi
   3edef:	or     %r13d,%esi
   3edf2:	mov    %esi,0x1c(%rcx,%rax,1)
   3edf6:	movzwl 0x10(%rdi),%esi
   3edfa:	add    %r14d,%esi
   3edfd:	shl    $0x8,%esi
   3ee00:	or     %r13d,%esi
   3ee03:	mov    %esi,0x20(%rcx,%rax,1)
   3ee07:	movzwl 0x12(%rdi),%esi
   3ee0b:	add    %r14d,%esi
   3ee0e:	shl    $0x8,%esi
   3ee11:	or     %r13d,%esi
   3ee14:	mov    %esi,0x24(%rcx,%rax,1)
   3ee18:	movzwl 0x14(%rdi),%esi
   3ee1c:	add    %r14d,%esi
   3ee1f:	shl    $0x8,%esi
   3ee22:	or     %r13d,%esi
   3ee25:	mov    %esi,0x28(%rcx,%rax,1)
   3ee29:	movzwl 0x16(%rdi),%esi
   3ee2d:	add    %r14d,%esi
   3ee30:	shl    $0x8,%esi
   3ee33:	or     %r13d,%esi
   3ee36:	mov    %esi,0x2c(%rcx,%rax,1)
   3ee3a:	movzwl 0x18(%rdi),%esi
   3ee3e:	add    %r14d,%esi
   3ee41:	shl    $0x8,%esi
   3ee44:	or     %r13d,%esi
   3ee47:	mov    %esi,0x30(%rcx,%rax,1)
   3ee4b:	movzwl 0x1a(%rdi),%esi
   3ee4f:	add    %r14d,%esi
   3ee52:	shl    $0x8,%esi
   3ee55:	or     %r13d,%esi
   3ee58:	mov    %esi,0x34(%rcx,%rax,1)
   3ee5c:	movzwl 0x1c(%rdi),%esi
   3ee60:	add    %r14d,%esi
   3ee63:	shl    $0x8,%esi
   3ee66:	or     %r13d,%esi
   3ee69:	mov    %esi,0x38(%rcx,%rax,1)
   3ee6d:	movzwl 0x1e(%rdi),%esi
   3ee71:	add    %r14d,%esi
   3ee74:	shl    $0x8,%esi
   3ee77:	or     %r13d,%esi
   3ee7a:	mov    %esi,0x3c(%rcx,%rax,1)
   3ee7e:	movzwl 0x20(%rdi),%esi
   3ee82:	add    %r14d,%esi
   3ee85:	shl    $0x8,%esi
   3ee88:	or     %r13d,%esi
   3ee8b:	mov    %esi,0x40(%rcx,%rax,1)
   3ee8f:	movzwl 0x22(%rdi),%esi
   3ee93:	add    %r14d,%esi
   3ee96:	shl    $0x8,%esi
   3ee99:	or     %r13d,%esi
   3ee9c:	mov    %esi,0x44(%rcx,%rax,1)
   3eea0:	movzwl 0x24(%rdi),%esi
   3eea4:	add    %r14d,%esi
   3eea7:	shl    $0x8,%esi
   3eeaa:	or     %r13d,%esi
   3eead:	mov    %esi,0x48(%rcx,%rax,1)
   3eeb1:	movzwl 0x26(%rdi),%esi
   3eeb5:	add    %r14d,%esi
   3eeb8:	shl    $0x8,%esi
   3eebb:	or     %r13d,%esi
   3eebe:	mov    %esi,0x4c(%rcx,%rax,1)
   3eec2:	movzwl 0x28(%rdi),%esi
   3eec6:	add    %r14d,%esi
   3eec9:	shl    $0x8,%esi
   3eecc:	or     %r13d,%esi
   3eecf:	mov    %esi,0x50(%rcx,%rax,1)
   3eed3:	movzwl 0x2a(%rdi),%esi
   3eed7:	add    %r14d,%esi
   3eeda:	shl    $0x8,%esi
   3eedd:	or     %r13d,%esi
   3eee0:	mov    %esi,0x54(%rcx,%rax,1)
   3eee4:	movzwl 0x2c(%rdi),%esi
   3eee8:	add    %r14d,%esi
   3eeeb:	shl    $0x8,%esi
   3eeee:	or     %r13d,%esi
   3eef1:	mov    %esi,0x58(%rcx,%rax,1)
   3eef5:	movzwl 0x2e(%rdi),%esi
   3eef9:	add    %r14d,%esi
   3eefc:	shl    $0x8,%esi
   3eeff:	or     %r13d,%esi
   3ef02:	mov    %esi,0x5c(%rcx,%rax,1)
   3ef06:	movzwl 0x30(%rdi),%esi
   3ef0a:	add    %r14d,%esi
   3ef0d:	shl    $0x8,%esi
   3ef10:	or     %r13d,%esi
   3ef13:	mov    %esi,0x60(%rcx,%rax,1)
   3ef17:	movzwl 0x32(%rdi),%esi
   3ef1b:	add    %r14d,%esi
   3ef1e:	shl    $0x8,%esi
   3ef21:	or     %r13d,%esi
   3ef24:	mov    %esi,0x64(%rcx,%rax,1)
   3ef28:	movzwl 0x34(%rdi),%esi
   3ef2c:	add    %r14d,%esi
   3ef2f:	shl    $0x8,%esi
   3ef32:	or     %r13d,%esi
   3ef35:	mov    %esi,0x68(%rcx,%rax,1)
   3ef39:	movzwl 0x36(%rdi),%esi
   3ef3d:	add    %r14d,%esi
   3ef40:	shl    $0x8,%esi
   3ef43:	or     %r13d,%esi
   3ef46:	mov    %esi,0x6c(%rcx,%rax,1)
   3ef4a:	movzwl 0x38(%rdi),%esi
   3ef4e:	add    %r14d,%esi
   3ef51:	shl    $0x8,%esi
   3ef54:	or     %r13d,%esi
   3ef57:	mov    %esi,0x70(%rcx,%rax,1)
   3ef5b:	movzwl 0x3a(%rdi),%esi
   3ef5f:	add    %r14d,%esi
   3ef62:	shl    $0x8,%esi
   3ef65:	or     %r13d,%esi
   3ef68:	mov    %esi,0x74(%rcx,%rax,1)
   3ef6c:	lea    (%r10,%rbp,1),%eax
   3ef70:	add    $0x20,%eax
   3ef73:	mov    %eax,0x28e0(%rsp)
   3ef7a:	add    $0x20,%rbp
   3ef7e:	inc    %r9
   3ef81:	add    $0x2000,%r15d
   3ef88:	dec    %rdx
   3ef8b:	je     3f18c <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
   3ef91:	mov    %r9,%rax
   3ef94:	shr    $0x6,%rax
   3ef98:	cmp    (%rsp),%rax
   3ef9c:	jae    3f7fb <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   3efa2:	mov    0x10(%rsp),%rcx
   3efa7:	mov    (%rcx,%rax,8),%rax
   3efab:	bt     %r9,%rax
   3efaf:	jae    3efe0 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   3efb1:	cmp    $0x1e,%rbx
   3efb5:	jae    3f820 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   3efbb:	mov    (%r12,%rbp,2),%eax
   3efbf:	and    $0xfffff,%eax
   3efc4:	mov    0x10(%r11),%rcx
   3efc8:	cmp    %rax,%rcx
   3efcb:	ja     3ed70 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   3efd1:	jmp    3f80e <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
   3efd6:	cs nopw 0x0(%rax,%rax,1)
   3efe0:	cmp    $0x1e,%rbx
   3efe4:	jae    3f834 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
   3efea:	movups 0x2c(%rdi),%xmm0
   3efee:	movups %xmm0,-0x10(%r12,%rbp,2)
   3eff4:	movdqa (%rdi),%xmm0
   3eff8:	movdqa 0x10(%rdi),%xmm1
   3effd:	movdqa 0x20(%rdi),%xmm2
   3f002:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   3f009:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   3f010:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   3f017:	mov    %r15d,(%r12,%rbp,2)
   3f01b:	jmp    3ef6c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
   3f020:	mov    0x28d8(%rsp),%rdi
   3f028:	cmp    $0x1d,%rdi
   3f02c:	ja     3f8c9 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   3f032:	shl    $0x6,%rdi
   3f036:	add    %r15,%rdi
   3f039:	mov    0x28e0(%rsp),%eax
   3f040:	mov    0xc0(%rsp),%rcx
   3f048:	shl    $0x6,%r9
   3f04c:	mov    %eax,%r14d
   3f04f:	shl    $0x8,%r14d
   3f053:	or     %r13d,%r14d
   3f056:	cmp    $0x3c,%r12
   3f05a:	jae    3f0c5 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3f05c:	xor    %esi,%esi
   3f05e:	jmp    3f14a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
   3f063:	shr    $1,%r9
   3f066:	inc    %r9
   3f069:	mov    %r9,%r10
   3f06c:	and    $0xfffffffffffffff8,%r10
   3f070:	lea    (%rdx,%r10,2),%r8
   3f074:	movd   %ecx,%xmm0
   3f078:	pshuflw $0x0,%xmm0,%xmm0
   3f07d:	add    %r15,%r11
   3f080:	add    $0x8,%r11
   3f084:	xor    %ebx,%ebx
   3f086:	cs nopw 0x0(%rax,%rax,1)
   3f090:	movq   -0x8(%r11,%rbx,2),%xmm1
   3f097:	movq   (%r11,%rbx,2),%xmm2
   3f09d:	paddw  %xmm0,%xmm1
   3f0a1:	paddw  %xmm0,%xmm2
   3f0a5:	punpcklqdq %xmm2,%xmm1
   3f0a9:	movdqu %xmm1,(%rax,%rbx,2)
   3f0ae:	add    $0x8,%rbx
   3f0b2:	cmp    %rbx,%r10
   3f0b5:	jne    3f090 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   3f0b7:	cmp    %r10,%r9
   3f0ba:	jne    3ec02 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   3f0c0:	jmp    3ec29 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
   3f0c5:	mov    %rdx,%r10
   3f0c8:	movabs $0xffffffffffffffe,%rsi
   3f0d2:	and    %rsi,%r10
   3f0d5:	lea    (%rcx,%r9,1),%r11
   3f0d9:	add    $0x7c,%r11
   3f0dd:	xor    %esi,%esi
   3f0df:	nop
   3f0e0:	movdqu 0x2c(%rdi),%xmm0
   3f0e5:	movdqu %xmm0,-0x50(%r11)
   3f0eb:	movdqa (%rdi),%xmm1
   3f0ef:	movdqa 0x10(%rdi),%xmm2
   3f0f4:	movdqa 0x20(%rdi),%xmm3
   3f0f9:	movdqa %xmm3,-0x5c(%r11)
   3f0ff:	movdqa %xmm2,-0x6c(%r11)
   3f105:	movdqa %xmm1,-0x7c(%r11)
   3f10b:	mov    %r14d,-0x40(%r11)
   3f10f:	add    $0x2,%rsi
   3f113:	movdqa %xmm1,-0x3c(%r11)
   3f119:	movdqa %xmm2,-0x2c(%r11)
   3f11f:	movdqa %xmm3,-0x1c(%r11)
   3f125:	movdqu %xmm0,-0x10(%r11)
   3f12b:	lea    0x2000(%r14),%ebx
   3f132:	mov    %ebx,(%r11)
   3f135:	add    $0x4000,%r14d
   3f13c:	sub    $0xffffffffffffff80,%r11
   3f140:	cmp    %rsi,%r10
   3f143:	jne    3f0e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   3f145:	test   $0x1,%dl
   3f148:	je     3f17e <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
   3f14a:	add    %r9,%rcx
   3f14d:	shl    $0x6,%rsi
   3f151:	movups 0x2c(%rdi),%xmm0
   3f155:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3f15a:	movdqa (%rdi),%xmm0
   3f15e:	movdqa 0x10(%rdi),%xmm1
   3f163:	movdqa 0x20(%rdi),%xmm2
   3f168:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3f16e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3f174:	movdqa %xmm0,(%rcx,%rsi,1)
   3f179:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3f17e:	mov    %edx,%ecx
   3f180:	shl    $0x5,%ecx
   3f183:	add    %ecx,%eax
   3f185:	mov    %eax,0x28e0(%rsp)
   3f18c:	mov    0x48(%rsp),%rdx
   3f191:	mov    %rdx,%rax
   3f194:	shl    $0x5,%rax
   3f198:	lea    (%rdx,%rdx,1),%rcx
   3f19c:	sub    %rax,%rcx
   3f19f:	mov    0x50(%rsp),%r12
   3f1a4:	add    %rcx,%r12
   3f1a7:	lea    (%rdx,%rdx,1),%eax
   3f1aa:	shl    $0x5,%edx
   3f1ad:	sub    %eax,%edx
   3f1af:	add    %edx,%r8d
   3f1b2:	mov    0x88(%rsp),%rbp
   3f1ba:	mov    %r8d,0x18(%rbp)
   3f1be:	test   %r12,%r12
   3f1c1:	mov    $0x88888889,%r14d
   3f1c7:	lea    -0x25b0e(%rip),%r15        # 196c0 <c30walk::ctor2::PATTERNS>
   3f1ce:	je     3f349 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
   3f1d4:	mov    0x1c(%rbp),%ecx
   3f1d7:	mov    %r8d,%edi
   3f1da:	imul   %r14,%rdi
   3f1de:	mov    %rdi,%rdx
   3f1e1:	shr    $0x24,%rdx
   3f1e5:	mov    %edx,%eax
   3f1e7:	shl    $0x5,%eax
   3f1ea:	sub    %edx,%eax
   3f1ec:	sub    %edx,%eax
   3f1ee:	lea    0x1e(%rax),%esi
   3f1f1:	cmp    %ecx,%esi
   3f1f3:	cmovb  %esi,%ecx
   3f1f6:	shr    $0x2a,%rdi
   3f1fa:	mov    0xa8(%rsp),%rsi
   3f202:	cmp    %rsi,%rdi
   3f205:	jae    3f8ac <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
   3f20b:	mov    %rdx,%rsi
   3f20e:	shl    $0x6,%rsi
   3f212:	add    0xc0(%rsp),%rsi
   3f21a:	mov    0xa0(%rsp),%r9
   3f222:	mov    (%r9,%rdi,8),%rdi
   3f226:	bt     %rdx,%rdi
   3f22a:	jb     3eb60 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   3f230:	mov    $0xffffffff,%edi
   3f235:	jmp    3eb6c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
   3f23a:	nopw   0x0(%rax,%rax,1)
   3f240:	mov    0x8(%rsp),%r13
   3f245:	jmp    3f356 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   3f24a:	nopw   0x0(%rax,%rax,1)
   3f250:	add    0x28(%rsp),%r11
   3f255:	xor    %r13d,%r13d
   3f258:	nopl   0x0(%rax,%rax,1)
   3f260:	movzbl (%r11,%r13,1),%esi
   3f265:	lea    (%rsi,%rsi,4),%rcx
   3f269:	lea    (%r8,%rcx,8),%rbp
   3f26d:	mov    (%r8,%rcx,8),%rax
   3f271:	cmp    0x8(%r8,%rcx,8),%rax
   3f276:	je     3f28e <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   3f278:	lea    (%r14,%r13,1),%rcx
   3f27c:	mov    %cx,(%rax)
   3f27f:	addq   $0x2,0x0(%rbp)
   3f284:	inc    %r13
   3f287:	cmp    %r13,%rbx
   3f28a:	jne    3f260 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3f28c:	jmp    3f2c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   3f28e:	mov    %r11,(%rsp)
   3f292:	lea    0x98(%rsp),%rdi
   3f29a:	call   8a7e0 <<c30walk::ctor2::W<false>>::refill>
   3f29f:	lea    0xd8(%rsp),%r8
   3f2a7:	test   %al,%al
   3f2a9:	je     3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3f2af:	mov    0x0(%rbp),%rax
   3f2b3:	mov    0x20(%rsp),%r10
   3f2b8:	mov    (%rsp),%r11
   3f2bc:	jmp    3f278 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
   3f2be:	xchg   %ax,%ax
   3f2c0:	cmp    %r15,0x30(%rsp)
   3f2c5:	mov    0x8(%rsp),%r13
   3f2ca:	jb     3f7e4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3f2d0:	cmp    $0xf,%r12
   3f2d4:	ja     3e9a0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3f2da:	add    0x28(%rsp),%r15
   3f2df:	mov    %r15,%r12
   3f2e2:	sub    %rbx,%r12
   3f2e5:	add    $0x10,%r12
   3f2e9:	add    %r14d,%ebx
   3f2ec:	add    $0x2,%ebx
   3f2ef:	nop
   3f2f0:	movzbl (%r15),%esi
   3f2f4:	lea    (%rsi,%rsi,4),%rcx
   3f2f8:	lea    (%r8,%rcx,8),%r14
   3f2fc:	mov    (%r8,%rcx,8),%rax
   3f300:	cmp    0x8(%r8,%rcx,8),%rax
   3f305:	je     3f31d <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   3f307:	mov    %bx,(%rax)
   3f30a:	addq   $0x2,(%r14)
   3f30e:	inc    %r15
   3f311:	inc    %ebx
   3f313:	cmp    %r12,%r15
   3f316:	jne    3f2f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   3f318:	jmp    3e9a0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3f31d:	lea    0x98(%rsp),%rdi
   3f325:	call   8a7e0 <<c30walk::ctor2::W<false>>::refill>
   3f32a:	lea    0xd8(%rsp),%r8
   3f332:	test   %al,%al
   3f334:	je     3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3f33a:	mov    (%r14),%rax
   3f33d:	mov    0x8(%rsp),%r13
   3f342:	mov    0x20(%rsp),%r10
   3f347:	jmp    3f307 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
   3f349:	mov    0x8(%rsp),%r13
   3f34e:	lea    0xd8(%rsp),%r8
   3f356:	mov    0x20(%rsp),%r10
   3f35b:	mov    0x68(%rsp),%r9
   3f360:	mov    0x38(%rsp),%rcx
   3f365:	mov    0x30(%rsp),%r11
   3f36a:	mov    0x44(%rsp),%edx
   3f36e:	mov    0x70(%rsp),%rsi
   3f373:	jmp    3e9e2 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
   3f378:	nopl   0x0(%rax,%rax,1)
   3f380:	lea    (%r14,%r11,1),%rax
   3f384:	cmp    %r10,%rax
   3f387:	jne    3f3d1 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
   3f389:	cmp    $0x10,%rdx
   3f38d:	jne    3f447 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
   3f393:	mov    0x28d8(%rsp),%rax
   3f39b:	lea    0x10(%rax),%rcx
   3f39f:	cmp    $0x1d,%rcx
   3f3a3:	mov    0x8(%rsp),%r13
   3f3a8:	mov    0x28(%rsp),%r14
   3f3ad:	mov    %rbp,%r9
   3f3b0:	jbe    3f4eb <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
   3f3b6:	add    $0xfffffffffffffff2,%rax
   3f3ba:	mov    %rax,0x28d8(%rsp)
   3f3c2:	mov    0x28e0(%rsp),%eax
   3f3c9:	add    $0x12,%eax
   3f3cc:	jmp    3f4fd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3f3d1:	add    $0x2,%r12d
   3f3d5:	movzwl %r12w,%r12d
   3f3d9:	nopl   0x0(%rax)
   3f3e0:	movzbl (%r15,%r14,1),%esi
   3f3e5:	lea    (%rsi,%rsi,4),%rcx
   3f3e9:	lea    (%r8,%rcx,8),%rbx
   3f3ed:	mov    (%r8,%rcx,8),%rax
   3f3f1:	cmp    0x8(%r8,%rcx,8),%rax
   3f3f6:	je     3f410 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   3f3f8:	lea    (%r12,%r14,1),%rcx
   3f3fc:	mov    %cx,(%rax)
   3f3ff:	addq   $0x2,(%rbx)
   3f403:	inc    %r14
   3f406:	cmp    %r14,%rdx
   3f409:	jne    3f3e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   3f40b:	jmp    3f389 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
   3f410:	mov    %rdx,0x10(%rsp)
   3f415:	mov    %r11,%r13
   3f418:	lea    0x98(%rsp),%rdi
   3f420:	call   8a7e0 <<c30walk::ctor2::W<false>>::refill>
   3f425:	lea    0xd8(%rsp),%r8
   3f42d:	test   %al,%al
   3f42f:	je     3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3f435:	mov    (%rbx),%rax
   3f438:	mov    0x20(%rsp),%r10
   3f43d:	mov    %r13,%r11
   3f440:	mov    0x10(%rsp),%rdx
   3f445:	jmp    3f3f8 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
   3f447:	mov    0x28e0(%rsp),%eax
   3f44e:	mov    0x28d8(%rsp),%rcx
   3f456:	inc    %r11
   3f459:	cmp    %r11,%r10
   3f45c:	mov    0x8(%rsp),%r13
   3f461:	mov    0x28(%rsp),%r14
   3f466:	mov    %rbp,%r9
   3f469:	je     3f4c6 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   3f46b:	mov    %rdx,%r10
   3f46e:	and    $0xffffe,%edx
   3f474:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3f480:	lea    0x1(%rcx),%rsi
   3f484:	add    $0xffffffffffffffe3,%rcx
   3f488:	xor    %edi,%edi
   3f48a:	cmp    $0x1e,%rsi
   3f48e:	setae  %dil
   3f492:	cmovb  %rsi,%rcx
   3f496:	lea    (%rax,%rdi,2),%eax
   3f499:	lea    0x1(%rcx),%rsi
   3f49d:	add    $0xffffffffffffffe3,%rcx
   3f4a1:	xor    %edi,%edi
   3f4a3:	cmp    $0x1e,%rsi
   3f4a7:	setae  %dil
   3f4ab:	cmovb  %rsi,%rcx
   3f4af:	lea    (%rax,%rdi,2),%eax
   3f4b2:	add    $0x2,%eax
   3f4b5:	add    $0xfffffffffffffffe,%rdx
   3f4b9:	jne    3f480 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   3f4bb:	test   $0x1,%r10b
   3f4bf:	mov    0x20(%rsp),%r10
   3f4c4:	je     3f4e1 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
   3f4c6:	lea    0x1(%rcx),%rdx
   3f4ca:	add    $0xffffffffffffffe3,%rcx
   3f4ce:	xor    %esi,%esi
   3f4d0:	cmp    $0x1e,%rdx
   3f4d4:	setae  %sil
   3f4d8:	cmovb  %rdx,%rcx
   3f4dc:	lea    (%rax,%rsi,2),%eax
   3f4df:	inc    %eax
   3f4e1:	mov    %rcx,0x28d8(%rsp)
   3f4e9:	jmp    3f4fd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3f4eb:	mov    %rcx,0x28d8(%rsp)
   3f4f3:	mov    0x28e0(%rsp),%eax
   3f4fa:	add    $0x10,%eax
   3f4fd:	mov    %eax,0x28e0(%rsp)
   3f504:	mov    %r10,%r11
   3f507:	cmp    %r13,%r10
   3f50a:	jb     3e7ec <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3f510:	jmp    3f51f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3f512:	mov    %r15,%rdi
   3f515:	call   e7d00 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3f51a:	jmp    3e86f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
   3f51f:	mov    $0x84,%eax
   3f524:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3f530:	lea    0x50(%rax),%rcx
   3f534:	cmp    $0x28d4,%rcx
   3f53b:	je     3f59b <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
   3f53d:	mov    0x54(%rsp,%rax,1),%rdx
   3f542:	sub    0x64(%rsp,%rax,1),%rdx
   3f547:	shr    $1,%rdx
   3f54a:	add    0x6c(%rsp,%rax,1),%edx
   3f54e:	cmp    0x70(%rsp,%rax,1),%edx
   3f552:	jne    3f577 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3f554:	mov    0x7c(%rsp,%rax,1),%rdx
   3f559:	sub    0x8c(%rsp,%rax,1),%rdx
   3f561:	shr    $1,%rdx
   3f564:	add    0x94(%rsp,%rax,1),%edx
   3f56b:	cmp    0x98(%rsp,%rax,1),%edx
   3f572:	mov    %rcx,%rax
   3f575:	je     3f530 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
   3f577:	mov    0x5520(%rsp),%rax
   3f57f:	movq   $0x0,0x10(%rax)
   3f587:	xor    %eax,%eax
   3f589:	add    $0x54e8,%rsp
   3f590:	pop    %rbx
   3f591:	pop    %r12
   3f593:	pop    %r13
   3f595:	pop    %r14
   3f597:	pop    %r15
   3f599:	pop    %rbp
   3f59a:	ret
   3f59b:	xor    %r14d,%r14d
   3f59e:	lea    0x98(%rsp),%r15
   3f5a6:	mov    0x78(%rsp),%r12
   3f5ab:	mov    0x8(%rsp),%r13
   3f5b0:	mov    0x60(%rsp),%rbp
   3f5b5:	jmp    3f5eb <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   3f5b7:	nopw   0x0(%rax,%rax,1)
   3f5c0:	sub    %rcx,%rax
   3f5c3:	inc    %r14
   3f5c6:	shr    $1,%rax
   3f5c9:	add    %eax,0x18(%r8)
   3f5cd:	pxor   %xmm0,%xmm0
   3f5d1:	movdqu %xmm0,(%r8)
   3f5d6:	movq   $0x0,0x10(%r8)
   3f5de:	add    $0x28,%r8
   3f5e2:	cmp    $0x100,%r14
   3f5e9:	je     3f611 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
   3f5eb:	mov    (%r8),%rcx
   3f5ee:	mov    %rcx,%rax
   3f5f1:	cmp    0x10(%r8),%rcx
   3f5f5:	je     3f5c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3f5f7:	mov    %r15,%rdi
   3f5fa:	mov    %r14,%rsi
   3f5fd:	mov    %r8,%rbx
   3f600:	call   8a1e0 <<c30walk::ctor2::W<false>>::finish>
   3f605:	mov    %rbx,%r8
   3f608:	mov    (%rbx),%rax
   3f60b:	mov    0x10(%rbx),%rcx
   3f60f:	jmp    3f5c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3f611:	xor    %ecx,%ecx
   3f613:	mov    0x8(%r12),%rbx
   3f618:	test   %rbp,%rbp
   3f61b:	setne  %al
   3f61e:	je     3f662 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3f620:	mov    %al,%cl
   3f622:	mov    %rbp,%rax
   3f625:	sub    %rcx,%rax
   3f628:	imul   $0xffffffffffffffe2,%rax,%rcx
   3f62c:	add    %r13,%rcx
   3f62f:	cmp    $0x1e,%rcx
   3f633:	jae    3f662 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3f635:	lea    (%rbx,%rbp,4),%rcx
   3f639:	lea    (%rcx,%r13,2),%rdi
   3f63d:	add    $0xfffffffffffffffc,%rdi
   3f641:	lea    (%rax,%rax,1),%rcx
   3f645:	shl    $0x5,%rax
   3f649:	sub    %rcx,%rax
   3f64c:	sub    %r13,%rax
   3f64f:	lea    0x3c(,%rax,2),%rdx
   3f657:	mov    $0xff,%esi
   3f65c:	call   *0xf67ee(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3f662:	shl    $0x6,%rbp
   3f666:	pcmpeqd %xmm0,%xmm0
   3f66a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   3f670:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   3f676:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   3f67c:	movdqa %xmm0,(%rbx,%rbp,1)
   3f681:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   3f689:	mov    0x58(%rsp),%rax
   3f68e:	mov    0x8(%rax),%rdx
   3f692:	mov    0x10(%rax),%rcx
   3f696:	mov    0x5520(%rsp),%rax
   3f69e:	mov    0x8(%rax),%r8
   3f6a2:	mov    0x10(%rax),%r9
   3f6a6:	lea    0x98(%rsp),%rdi
   3f6ae:	mov    %rbx,%rsi
   3f6b1:	call   9b7f0 <c30walk::ctor::canonicalise>
   3f6b6:	mov    0x80(%rsp),%rax
   3f6be:	mov    %rax,0x10(%r12)
   3f6c3:	mov    $0x1,%al
   3f6c5:	jmp    3f589 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
   3f6ca:	lea    0xf33c7(%rip),%rcx        # 132a98 <__frame_dummy_init_array_entry+0x2f78>
   3f6d1:	mov    %r12,%rdi
   3f6d4:	mov    %r13,%rdx
   3f6d7:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f6dc:	lea    0xf1fbd(%rip),%rcx        # 1316a0 <__frame_dummy_init_array_entry+0x1b80>
   3f6e3:	mov    $0x1e,%edx
   3f6e8:	xor    %edi,%edi
   3f6ea:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f6ef:	mov    %rdi,%r12
   3f6f2:	test   %rax,%rax
   3f6f5:	je     3f707 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
   3f6f7:	mov    0x8(%r9),%rdi
   3f6fb:	mov    %r9,%r14
   3f6fe:	call   *0xf675c(%rip)        # 135e60 <free@GLIBC_2.2.5>
   3f704:	mov    %r14,%r9
   3f707:	movq   $0x0,(%r9)
   3f70e:	movq   $0x40,0x8(%r9)
   3f716:	movq   $0x0,0x10(%r9)
   3f71e:	mov    $0x40,%edx
   3f723:	mov    %r9,%rdi
   3f726:	lea    0x1(%rbp),%rsi
   3f72a:	mov    %r9,%r14
   3f72d:	call   90bf0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3f732:	mov    %r14,%r9
   3f735:	cmp    $0xffffffffffffffff,%rax
   3f739:	mov    %r12,%rdi
   3f73c:	je     3e235 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   3f742:	jmp    3f79d <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
   3f744:	mov    %rdi,%r12
   3f747:	mov    %r9,%r14
   3f74a:	test   %rax,%rax
   3f74d:	je     3f761 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
   3f74f:	mov    0x5520(%rsp),%rax
   3f757:	mov    0x8(%rax),%rdi
   3f75b:	call   *0xf66ff(%rip)        # 135e60 <free@GLIBC_2.2.5>
   3f761:	mov    0x5520(%rsp),%rdi
   3f769:	movq   $0x0,(%rdi)
   3f770:	movq   $0x40,0x8(%rdi)
   3f778:	movq   $0x0,0x10(%rdi)
   3f780:	mov    $0x80,%edx
   3f785:	mov    %rbx,%rsi
   3f788:	call   90bf0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3f78d:	cmp    $0xffffffffffffffff,%rax
   3f791:	mov    %r14,%r9
   3f794:	mov    %r12,%rdi
   3f797:	je     3e249 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
   3f79d:	mov    %rax,%rdi
   3f7a0:	mov    %rdx,%rsi
   3f7a3:	call   d766b <alloc::raw_vec::handle_error>
   3f7a8:	lea    0xf32e9(%rip),%rcx        # 132a98 <__frame_dummy_init_array_entry+0x2f78>
   3f7af:	mov    %r11,%rdi
   3f7b2:	mov    %r13,%rdx
   3f7b5:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f7ba:	lea    0xf1f0f(%rip),%rcx        # 1316d0 <__frame_dummy_init_array_entry+0x1bb0>
   3f7c1:	mov    %r11,%rdi
   3f7c4:	mov    %r15,%rsi
   3f7c7:	mov    %r13,%rdx
   3f7ca:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f7cf:	lea    0xf32c2(%rip),%rcx        # 132a98 <__frame_dummy_init_array_entry+0x2f78>
   3f7d6:	mov    %rsi,%rdi
   3f7d9:	mov    %rax,%rsi
   3f7dc:	mov    %r13,%rdx
   3f7df:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f7e4:	lea    0xf1ecd(%rip),%rcx        # 1316b8 <__frame_dummy_init_array_entry+0x1b98>
   3f7eb:	mov    %r15,%rdi
   3f7ee:	mov    0x30(%rsp),%rsi
   3f7f3:	mov    %r13,%rdx
   3f7f6:	call   d9ee0 <core::slice::index::slice_index_fail>
   3f7fb:	lea    0xf0576(%rip),%rdx        # 12fd78 <__frame_dummy_init_array_entry+0x258>
   3f802:	mov    %rax,%rdi
   3f805:	mov    (%rsp),%rsi
   3f809:	call   d9103 <core::panicking::panic_bounds_check>
   3f80e:	lea    0xf1f4b(%rip),%rdx        # 131760 <__frame_dummy_init_array_entry+0x1c40>
   3f815:	mov    %rax,%rdi
   3f818:	mov    %rcx,%rsi
   3f81b:	call   d9103 <core::panicking::panic_bounds_check>
   3f820:	lea    0xf1f21(%rip),%rdx        # 131748 <__frame_dummy_init_array_entry+0x1c28>
   3f827:	mov    $0x1e,%esi
   3f82c:	mov    %rbx,%rdi
   3f82f:	call   d9103 <core::panicking::panic_bounds_check>
   3f834:	lea    0xf1ef5(%rip),%rdx        # 131730 <__frame_dummy_init_array_entry+0x1c10>
   3f83b:	mov    $0x1e,%esi
   3f840:	mov    %rbx,%rdi
   3f843:	call   d9103 <core::panicking::panic_bounds_check>
   3f848:	mov    $0x10,%edx
   3f84d:	mov    $0x4,%ecx
   3f852:	mov    $0x400,%r8d
   3f858:	mov    %rdi,%r14
   3f85b:	mov    %r12,%rdi
   3f85e:	xor    %esi,%esi
   3f860:	mov    %r9,%rbx
   3f863:	call   68110 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3f868:	mov    %r14,%rdi
   3f86b:	mov    %rbx,%r9
   3f86e:	jmp    3e2e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
   3f873:	mov    %r9,%r13
   3f876:	mov    $0x8,%ecx
   3f87b:	mov    $0x8,%r8d
   3f881:	mov    %rdi,%r14
   3f884:	xor    %esi,%esi
   3f886:	mov    %rbx,%rdx
   3f889:	call   68110 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3f88e:	mov    %r14,%rdi
   3f891:	mov    0x10(%r14),%r14
   3f895:	mov    0x8(%rdi),%r12
   3f899:	lea    (%r12,%r14,8),%rdi
   3f89d:	cmp    $0x1,%rbx
   3f8a1:	jne    3e287 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   3f8a7:	jmp    3e2a9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3f8ac:	lea    0xf04c5(%rip),%rdx        # 12fd78 <__frame_dummy_init_array_entry+0x258>
   3f8b3:	call   d9103 <core::panicking::panic_bounds_check>
   3f8b8:	lea    0xf1dc9(%rip),%rdx        # 131688 <__frame_dummy_init_array_entry+0x1b68>
   3f8bf:	mov    $0x1e,%esi
   3f8c4:	call   d9103 <core::panicking::panic_bounds_check>
   3f8c9:	lea    0xf1e48(%rip),%rdx        # 131718 <__frame_dummy_init_array_entry+0x1bf8>
   3f8d0:	mov    $0x1e,%esi
   3f8d5:	call   d9103 <core::panicking::panic_bounds_check>
   3f8da:	lea    0xf1d2f(%rip),%rdx        # 131610 <__frame_dummy_init_array_entry+0x1af0>
   3f8e1:	call   d9103 <core::panicking::panic_bounds_check>
   3f8e6:	lea    0xf048b(%rip),%rdx        # 12fd78 <__frame_dummy_init_array_entry+0x258>
   3f8ed:	mov    %rbp,%rdi
   3f8f0:	call   d9103 <core::panicking::panic_bounds_check>
