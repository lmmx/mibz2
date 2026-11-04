# <c30walk::ctor2d::Ctor2d>::build::<false>
# size 0x1a1a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003a2a0 <<c30walk::ctor2d::Ctor2d>::build::<false>>:
   3a2a0:	push   %rbp
   3a2a1:	push   %r15
   3a2a3:	push   %r14
   3a2a5:	push   %r13
   3a2a7:	push   %r12
   3a2a9:	push   %rbx
   3a2aa:	sub    $0x1000,%rsp
   3a2b1:	movq   $0x0,(%rsp)
   3a2b9:	sub    $0x1000,%rsp
   3a2c0:	movq   $0x0,(%rsp)
   3a2c8:	sub    $0x1000,%rsp
   3a2cf:	movq   $0x0,(%rsp)
   3a2d7:	sub    $0x1000,%rsp
   3a2de:	movq   $0x0,(%rsp)
   3a2e6:	sub    $0x4e8,%rsp
   3a2ed:	mov    %r8d,0x1c(%rsp)
   3a2f2:	mov    0x4520(%rsp),%rax
   3a2fa:	movq   $0x0,0x10(%r9)
   3a302:	movq   $0x0,0x10(%rax)
   3a30a:	cmp    $0xeffe2,%rdx
   3a311:	ja     3b91d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x167d>
   3a317:	mov    %rcx,%rbx
   3a31a:	mov    %rsi,%r12
   3a31d:	mov    %rdi,%rbp
   3a320:	mov    %rdx,%rsi
   3a323:	pxor   %xmm0,%xmm0
   3a327:	xor    %eax,%eax
   3a329:	pxor   %xmm2,%xmm2
   3a32d:	pxor   %xmm1,%xmm1
   3a331:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3a340:	movq   (%rbx,%rax,4),%xmm3
   3a345:	movq   0x8(%rbx,%rax,4),%xmm4
   3a34b:	punpckldq %xmm0,%xmm3
   3a34f:	paddq  %xmm2,%xmm3
   3a353:	punpckldq %xmm0,%xmm4
   3a357:	paddq  %xmm1,%xmm4
   3a35b:	movq   0x10(%rbx,%rax,4),%xmm2
   3a361:	movq   0x18(%rbx,%rax,4),%xmm1
   3a367:	punpckldq %xmm0,%xmm2
   3a36b:	paddq  %xmm3,%xmm2
   3a36f:	punpckldq %xmm0,%xmm1
   3a373:	paddq  %xmm4,%xmm1
   3a377:	add    $0x8,%rax
   3a37b:	cmp    $0x100,%rax
   3a381:	jne    3a340 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa0>
   3a383:	paddq  %xmm2,%xmm1
   3a387:	pshufd $0xee,%xmm1,%xmm0
   3a38c:	paddq  %xmm1,%xmm0
   3a390:	movq   %xmm0,%rax
   3a395:	cmp    %rsi,%rax
   3a398:	jne    3b91d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x167d>
   3a39e:	pxor   %xmm2,%xmm2
   3a3a2:	xor    %eax,%eax
   3a3a4:	movdqa -0x2ad2c(%rip),%xmm3        # f680 <GCC_except_table965+0xaf0>
   3a3ac:	pxor   %xmm1,%xmm1
   3a3b0:	pxor   %xmm0,%xmm0
   3a3b4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3a3c0:	movq   (%rbx,%rax,4),%xmm4
   3a3c5:	movq   0x8(%rbx,%rax,4),%xmm5
   3a3cb:	pshufd $0xd4,%xmm4,%xmm4
   3a3d0:	pcmpeqd %xmm2,%xmm4
   3a3d4:	pandn  %xmm3,%xmm4
   3a3d8:	paddq  %xmm1,%xmm4
   3a3dc:	pshufd $0xd4,%xmm5,%xmm5
   3a3e1:	pcmpeqd %xmm2,%xmm5
   3a3e5:	pandn  %xmm3,%xmm5
   3a3e9:	paddq  %xmm0,%xmm5
   3a3ed:	movq   0x10(%rbx,%rax,4),%xmm0
   3a3f3:	movq   0x18(%rbx,%rax,4),%xmm6
   3a3f9:	pshufd $0xd4,%xmm0,%xmm1
   3a3fe:	pcmpeqd %xmm2,%xmm1
   3a402:	pandn  %xmm3,%xmm1
   3a406:	paddq  %xmm4,%xmm1
   3a40a:	pshufd $0xd4,%xmm6,%xmm0
   3a40f:	pcmpeqd %xmm2,%xmm0
   3a413:	pandn  %xmm3,%xmm0
   3a417:	paddq  %xmm5,%xmm0
   3a41b:	add    $0x8,%rax
   3a41f:	cmp    $0x100,%rax
   3a425:	jne    3a3c0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x120>
   3a427:	mov    $0x88888889,%r8d
   3a42d:	imul   %rsi,%r8
   3a431:	shr    $0x24,%r8
   3a435:	imul   $0xeeeeeeef,%esi,%eax
   3a43b:	ror    $1,%eax
   3a43d:	cmp    $0x8888889,%eax
   3a442:	sbb    $0xffffffffffffffff,%r8
   3a446:	mov    %r8,%r15
   3a449:	cmpl   $0x0,0x1c(%rsp)
   3a44e:	jne    3a4a5 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3a450:	test   %rsi,%rsi
   3a453:	je     3a4a2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x202>
   3a455:	paddq  %xmm1,%xmm0
   3a459:	pshufd $0xee,%xmm0,%xmm1
   3a45e:	paddq  %xmm0,%xmm1
   3a462:	movq   %xmm1,%rax
   3a467:	lea    -0x1(%rsi),%ecx
   3a46a:	mov    $0x88888889,%edx
   3a46f:	imul   %rcx,%rdx
   3a473:	shr    $0x23,%rdx
   3a477:	and    $0xfffffffe,%edx
   3a47a:	lea    (%rdx,%rsi,1),%ecx
   3a47d:	dec    %ecx
   3a47f:	shr    $1,%ecx
   3a481:	mov    $0x80010003,%r15d
   3a487:	imul   %rcx,%r15
   3a48b:	shr    $0x2e,%r15
   3a48f:	inc    %r15d
   3a492:	imul   %rax,%r15
   3a496:	dec    %r15
   3a499:	cmp    %r8,%r15
   3a49c:	cmovae %r8,%r15
   3a4a0:	jmp    3a4a5 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3a4a2:	xor    %r15d,%r15d
   3a4a5:	mov    (%r9),%rax
   3a4a8:	lea    0x1(%r8),%rcx
   3a4ac:	mov    %rcx,0x68(%rsp)
   3a4b1:	cmp    %r8,%rax
   3a4b4:	mov    %rsi,(%rsp)
   3a4b8:	mov    %r8,0x58(%rsp)
   3a4bd:	jbe    3bad2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1832>
   3a4c3:	mov    0x4520(%rsp),%rdi
   3a4cb:	mov    (%rdi),%rax
   3a4ce:	cmp    %r15,%rax
   3a4d1:	jb     3bb27 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1887>
   3a4d7:	movq   $0x0,0x10(%rbp)
   3a4df:	lea    0x3f(%r8),%r15
   3a4e3:	shr    $0x6,%r15
   3a4e7:	mov    %r12,0x40(%rsp)
   3a4ec:	je     3a53d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x29d>
   3a4ee:	cmp    0x0(%rbp),%r15
   3a4f2:	ja     3bc33 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1993>
   3a4f8:	xor    %r14d,%r14d
   3a4fb:	mov    0x8(%rbp),%r12
   3a4ff:	lea    (%r12,%r14,8),%rdi
   3a503:	cmp    $0x1,%r15
   3a507:	je     3a531 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3a509:	lea    -0x8(,%r15,8),%rdx
   3a511:	xor    %esi,%esi
   3a513:	mov    %r9,%r13
   3a516:	call   *0xf9cbc(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   3a51c:	mov    %r13,%r9
   3a51f:	lea    (%r14,%r15,1),%rax
   3a523:	lea    (%r12,%rax,8),%rdi
   3a527:	add    $0xfffffffffffffff8,%rdi
   3a52b:	add    %r15,%r14
   3a52e:	dec    %r14
   3a531:	movq   $0x0,(%rdi)
   3a538:	inc    %r14
   3a53b:	jmp    3a540 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2a0>
   3a53d:	xor    %r14d,%r14d
   3a540:	lea    0x18(%rbp),%r15
   3a544:	mov    %r14,0x10(%rbp)
   3a548:	movq   $0x0,0x28(%rbp)
   3a550:	cmpq   $0xf,0x18(%rbp)
   3a555:	jbe    3bc0e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x196e>
   3a55b:	lea    0x30(%rbp),%rax
   3a55f:	mov    %rax,0x48(%rsp)
   3a564:	xor    %eax,%eax
   3a566:	pxor   %xmm0,%xmm0
   3a56a:	movabs $0xffffffff00000000,%rcx
   3a574:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3a580:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3a589:	mov    %rcx,0x2cf8(%rsp,%rax,1)
   3a591:	movdqu %xmm0,0x2d00(%rsp,%rax,1)
   3a59a:	mov    %rcx,0x2d10(%rsp,%rax,1)
   3a5a2:	movdqu %xmm0,0x2d18(%rsp,%rax,1)
   3a5ab:	mov    %rcx,0x2d28(%rsp,%rax,1)
   3a5b3:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   3a5bc:	mov    %rcx,0x2d40(%rsp,%rax,1)
   3a5c4:	add    $0x60,%rax
   3a5c8:	cmp    $0x1800,%rax
   3a5ce:	jne    3a580 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2e0>
   3a5d0:	mov    %r15,0x28(%rsp)
   3a5d5:	lea    0xc(%rbx),%rax
   3a5d9:	xor    %edx,%edx
   3a5db:	mov    $0x15,%ecx
   3a5e0:	mov    -0xc(%rax),%esi
   3a5e3:	add    %edx,%esi
   3a5e5:	mov    %edx,0x2c9c(%rsp,%rcx,4)
   3a5ec:	mov    %esi,0x2ca0(%rsp,%rcx,4)
   3a5f3:	mov    -0x8(%rax),%edx
   3a5f6:	add    %esi,%edx
   3a5f8:	mov    %esi,0x2cb4(%rsp,%rcx,4)
   3a5ff:	mov    %edx,0x2cb8(%rsp,%rcx,4)
   3a606:	mov    -0x4(%rax),%esi
   3a609:	add    %edx,%esi
   3a60b:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   3a612:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   3a619:	mov    (%rax),%edx
   3a61b:	add    %esi,%edx
   3a61d:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3a624:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3a62b:	add    $0x18,%rcx
   3a62f:	add    $0x10,%rax
   3a633:	cmp    $0x615,%rcx
   3a63a:	jne    3a5e0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x340>
   3a63c:	mov    %r9,0x88(%rsp)
   3a644:	mov    0x8(%r9),%r14
   3a648:	mov    0x8(%rbp),%r13
   3a64c:	mov    %rbp,0x80(%rsp)
   3a654:	mov    0x10(%rbp),%rbp
   3a658:	lea    0x8d8(%rsp),%rdi
   3a660:	xor    %r12d,%r12d
   3a663:	mov    $0x800,%edx
   3a668:	xor    %esi,%esi
   3a66a:	call   *0xf9b68(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   3a670:	lea    0x10d8(%rsp),%rdi
   3a678:	lea    0x2ce8(%rsp),%rsi
   3a680:	mov    $0x1800,%edx
   3a685:	call   *0xf9b6d(%rip)        # 1341f8 <memcpy@GLIBC_2.14>
   3a68b:	mov    %r14,0xc0(%rsp)
   3a693:	mov    0x58(%rsp),%rax
   3a698:	mov    %rax,0xc8(%rsp)
   3a6a0:	mov    (%rsp),%r15
   3a6a4:	mov    %r15,0xd0(%rsp)
   3a6ac:	mov    0x4520(%rsp),%rax
   3a6b4:	mov    %rax,0x98(%rsp)
   3a6bc:	mov    %r13,0x10(%rsp)
   3a6c1:	mov    %r13,0xa0(%rsp)
   3a6c9:	mov    %rbp,0x8(%rsp)
   3a6ce:	mov    %rbp,0xa8(%rsp)
   3a6d6:	mov    0x28(%rsp),%rax
   3a6db:	mov    %rax,0xb0(%rsp)
   3a6e3:	lea    0xd8(%rsp),%rdi
   3a6eb:	mov    $0x800,%edx
   3a6f0:	xor    %esi,%esi
   3a6f2:	call   *0xf9ae0(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   3a6f8:	mov    %r15,%r11
   3a6fb:	movl   $0x0,0x28e0(%rsp)
   3a706:	movq   $0x0,0x28d8(%rsp)
   3a712:	mov    0x48(%rsp),%rax
   3a717:	mov    %rax,0xb8(%rsp)
   3a71f:	mov    $0xffffffffffffffff,%rcx
   3a726:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3a730:	movabs $0x888888888888889,%r15
   3a73a:	pxor   %xmm0,%xmm0
   3a73e:	pcmpeqd %xmm1,%xmm1
   3a742:	xor    %r13d,%r13d
   3a745:	jmp    3a765 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4c5>
   3a747:	nopw   0x0(%rax,%rax,1)
   3a750:	add    $0x4,%r13
   3a754:	cmp    $0x400,%r13
   3a75b:	mov    (%rsp),%r11
   3a75f:	je     3a88a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5ea>
   3a765:	mov    %r12,%rax
   3a768:	mov    (%rbx,%r13,1),%r12d
   3a76c:	add    %rax,%r12
   3a76f:	cmp    %r11,%r12
   3a772:	setae  %sil
   3a776:	mov    %r12,%rax
   3a779:	movabs $0x8888888888888889,%rdx
   3a783:	mul    %rdx
   3a786:	shr    $0x4,%rdx
   3a78a:	mov    %r12,%rax
   3a78d:	imul   %r8,%rax
   3a791:	ror    $1,%rax
   3a794:	cmp    %r15,%rax
   3a797:	setb   %al
   3a79a:	cmp    %rcx,%rdx
   3a79d:	sete   %dil
   3a7a1:	or     %sil,%dil
   3a7a4:	or     %al,%dil
   3a7a7:	jne    3a750 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3a7a9:	mov    0x4520(%rsp),%rdi
   3a7b1:	mov    0x10(%rdi),%rbp
   3a7b5:	cmp    (%rdi),%rbp
   3a7b8:	je     3a85c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5bc>
   3a7be:	mov    0x8(%rdi),%rax
   3a7c2:	mov    %rbp,%rcx
   3a7c5:	shl    $0x7,%rcx
   3a7c9:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3a7cf:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3a7d5:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3a7db:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3a7e1:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3a7e7:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3a7ed:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3a7f3:	movdqa %xmm0,(%rax,%rcx,1)
   3a7f8:	mov    %rbp,%rax
   3a7fb:	inc    %rax
   3a7fe:	mov    %rax,0x10(%rdi)
   3a802:	mov    %rdx,%rax
   3a805:	shl    $0x6,%rax
   3a809:	or     $0x10000000,%ebp
   3a80f:	movdqu %xmm1,0x2c(%r14,%rax,1)
   3a816:	movdqa %xmm1,0x20(%r14,%rax,1)
   3a81d:	movdqa %xmm1,0x10(%r14,%rax,1)
   3a824:	movdqa %xmm1,(%r14,%rax,1)
   3a82a:	mov    %ebp,0x3c(%r14,%rax,1)
   3a82f:	mov    %rdx,%rdi
   3a832:	shr    $0x6,%rdi
   3a836:	cmp    0x8(%rsp),%rdi
   3a83b:	jae    3bc9a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19fa>
   3a841:	mov    $0x1,%eax
   3a846:	mov    %edx,%ecx
   3a848:	shl    %cl,%rax
   3a84b:	mov    0x10(%rsp),%rcx
   3a850:	or     %rax,(%rcx,%rdi,8)
   3a854:	mov    %rdx,%rcx
   3a857:	jmp    3a750 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3a85c:	mov    %rdx,0x28(%rsp)
   3a861:	call   90220 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3a866:	pcmpeqd %xmm1,%xmm1
   3a86a:	pxor   %xmm0,%xmm0
   3a86e:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3a878:	mov    0x4520(%rsp),%rdi
   3a880:	mov    0x28(%rsp),%rdx
   3a885:	jmp    3a7be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x51e>
   3a88a:	cmpl   $0x0,0x1c(%rsp)
   3a88f:	je     3aa42 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3a895:	mov    0xc8(%rsp),%r8
   3a89d:	test   %r8,%r8
   3a8a0:	je     3aa42 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3a8a6:	mov    %r11,%r9
   3a8a9:	shl    $0x20,%r9
   3a8ad:	mov    0x1c(%rsp),%r10d
   3a8b2:	mov    0x98(%rsp),%rdi
   3a8ba:	mov    0xc0(%rsp),%rbp
   3a8c2:	xor    %r15d,%r15d
   3a8c5:	movabs $0xff51afd7ed558ccd,%r14
   3a8cf:	pxor   %xmm0,%xmm0
   3a8d3:	pcmpeqd %xmm1,%xmm1
   3a8d7:	jmp    3a960 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6c0>
   3a8dc:	mov    0x8(%rdi),%rax
   3a8e0:	mov    %r14,%rcx
   3a8e3:	shl    $0x7,%rcx
   3a8e7:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3a8ed:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3a8f3:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3a8f9:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3a8ff:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3a905:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3a90b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3a911:	movdqa %xmm0,(%rax,%rcx,1)
   3a916:	mov    %r14,%rax
   3a919:	inc    %rax
   3a91c:	mov    %rax,0x10(%rdi)
   3a920:	or     $0x10000000,%r14d
   3a927:	movdqu %xmm1,0x2c(%rbp)
   3a92c:	movdqa %xmm1,0x20(%rbp)
   3a931:	movdqa %xmm1,0x10(%rbp)
   3a936:	movdqa %xmm1,0x0(%rbp)
   3a93b:	mov    %r14d,0x3c(%rbp)
   3a93f:	or     %rbx,0x0(%r13,%r12,8)
   3a944:	movabs $0xff51afd7ed558ccd,%r14
   3a94e:	xchg   %ax,%ax
   3a950:	inc    %r15
   3a953:	add    $0x40,%rbp
   3a957:	cmp    %r15,%r8
   3a95a:	je     3aa42 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3a960:	mov    %r15,%r12
   3a963:	shr    $0x6,%r12
   3a967:	mov    0xa8(%rsp),%rsi
   3a96f:	cmp    %rsi,%r12
   3a972:	jae    3bcab <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1a0b>
   3a978:	mov    0xa0(%rsp),%r13
   3a980:	mov    0x0(%r13,%r12,8),%rax
   3a985:	mov    $0x1,%ebx
   3a98a:	mov    %r15d,%ecx
   3a98d:	shl    %cl,%rbx
   3a990:	bt     %r15,%rax
   3a994:	jb     3a950 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3a996:	mov    %r15,%rax
   3a999:	xor    %r9,%rax
   3a99c:	mov    %rax,%rcx
   3a99f:	shr    $0x21,%rcx
   3a9a3:	xor    %rax,%rcx
   3a9a6:	imul   %r14,%rcx
   3a9aa:	mov    %rcx,%rax
   3a9ad:	shr    $0x21,%rax
   3a9b1:	xor    %rcx,%rax
   3a9b4:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3a9be:	imul   %rcx,%rax
   3a9c2:	mov    %rax,%rcx
   3a9c5:	shr    $0x21,%rcx
   3a9c9:	xor    %rax,%rcx
   3a9cc:	mov    %rcx,%rax
   3a9cf:	shr    $0x3,%rax
   3a9d3:	movabs $0x20c49ba5e353f7cf,%rdx
   3a9dd:	mul    %rdx
   3a9e0:	shr    $0x4,%rdx
   3a9e4:	imul   $0x3e8,%rdx,%rax
   3a9eb:	sub    %rax,%rcx
   3a9ee:	cmp    %r10,%rcx
   3a9f1:	jae    3a950 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3a9f7:	mov    0x10(%rdi),%r14
   3a9fb:	cmp    (%rdi),%r14
   3a9fe:	jne    3a8dc <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3aa04:	mov    %r8,0x8(%rsp)
   3aa09:	mov    %r9,0x10(%rsp)
   3aa0e:	mov    %r10,0x28(%rsp)
   3aa13:	mov    %rdi,0x48(%rsp)
   3aa18:	call   90220 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3aa1d:	pcmpeqd %xmm1,%xmm1
   3aa21:	pxor   %xmm0,%xmm0
   3aa25:	mov    0x48(%rsp),%rdi
   3aa2a:	mov    0x28(%rsp),%r10
   3aa2f:	mov    0x10(%rsp),%r9
   3aa34:	mov    0x8(%rsp),%r8
   3aa39:	mov    (%rsp),%r11
   3aa3d:	jmp    3a8dc <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3aa42:	test   %r11,%r11
   3aa45:	mov    0x40(%rsp),%rbp
   3aa4a:	je     3b8be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x161e>
   3aa50:	xor    %edi,%edi
   3aa52:	jmp    3aa6c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3aa54:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3aa60:	mov    %rdi,%r8
   3aa63:	cmp    %r10,%r8
   3aa66:	jae    3b8be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x161e>
   3aa6c:	mov    %rdi,%r13
   3aa6f:	mov    $0x400,%edx
   3aa74:	lea    0x28e8(%rsp),%rdi
   3aa7c:	xor    %esi,%esi
   3aa7e:	call   *0xf9754(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   3aa84:	mov    $0x848,%eax
   3aa89:	mov    $0xa,%ecx
   3aa8e:	xchg   %ax,%ax
   3aa90:	movd   0x68(%rsp,%rax,2),%xmm0
   3aa96:	movd   0x50(%rsp,%rax,2),%xmm1
   3aa9c:	punpckldq %xmm0,%xmm1
   3aaa0:	movd   0x98(%rsp,%rax,2),%xmm0
   3aaa9:	movd   0x80(%rsp,%rax,2),%xmm2
   3aab2:	punpckldq %xmm0,%xmm2
   3aab6:	punpcklqdq %xmm2,%xmm1
   3aaba:	movdqu 0x88(%rsp,%rcx,8),%xmm0
   3aac3:	movdqu 0x98(%rsp,%rcx,8),%xmm2
   3aacc:	movq   0x60(%rsp,%rax,2),%xmm3
   3aad2:	movq   0x48(%rsp,%rax,2),%xmm4
   3aad8:	punpcklqdq %xmm3,%xmm4
   3aadc:	psubq  %xmm4,%xmm0
   3aae0:	movq   0x90(%rsp,%rax,2),%xmm3
   3aae9:	movq   0x78(%rsp,%rax,2),%xmm4
   3aaef:	punpcklqdq %xmm3,%xmm4
   3aaf3:	psubq  %xmm4,%xmm2
   3aaf7:	psrlq  $0x1,%xmm0
   3aafc:	psrlq  $0x1,%xmm2
   3ab01:	pshufd $0xe8,%xmm0,%xmm0
   3ab06:	pshufd $0xe8,%xmm2,%xmm2
   3ab0b:	punpcklqdq %xmm2,%xmm0
   3ab0f:	paddd  %xmm1,%xmm0
   3ab13:	movdqu %xmm0,0x28c0(%rsp,%rcx,4)
   3ab1c:	add    $0x30,%rax
   3ab20:	add    $0x4,%rcx
   3ab24:	cmp    $0x106,%rcx
   3ab2b:	jne    3aa90 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7f0>
   3ab31:	movups 0x2880(%rsp),%xmm0
   3ab39:	movdqu 0x8c8(%rsp),%xmm1
   3ab42:	movdqu 0x8b8(%rsp),%xmm2
   3ab4b:	movq   0x28c0(%rsp),%xmm3
   3ab54:	movq   0x28a8(%rsp),%xmm4
   3ab5d:	punpcklqdq %xmm3,%xmm4
   3ab61:	psubq  %xmm4,%xmm1
   3ab65:	movq   0x2890(%rsp),%xmm3
   3ab6e:	movq   0x2878(%rsp),%xmm4
   3ab77:	punpcklqdq %xmm3,%xmm4
   3ab7b:	psubq  %xmm4,%xmm2
   3ab7f:	psrlq  $0x1,%xmm1
   3ab84:	psrlq  $0x1,%xmm2
   3ab89:	shufps $0x88,%xmm1,%xmm2
   3ab8d:	movss  0x2898(%rsp),%xmm1
   3ab96:	movlhps %xmm0,%xmm1
   3ab99:	movss  0x28b0(%rsp),%xmm0
   3aba2:	movss  0x28c8(%rsp),%xmm3
   3abab:	movlhps %xmm0,%xmm3
   3abae:	shufps $0x22,%xmm3,%xmm1
   3abb2:	paddd  %xmm2,%xmm1
   3abb6:	movdqu %xmm1,0x2cd8(%rsp)
   3abbf:	mov    0xb0(%rsp),%rbx
   3abc7:	mov    0x10(%rbx),%r14
   3abcb:	cmp    (%rbx),%r14
   3abce:	je     3b8b1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1611>
   3abd4:	mov    %r14,%rdi
   3abd7:	shl    $0xa,%rdi
   3abdb:	add    0x8(%rbx),%rdi
   3abdf:	mov    $0x400,%edx
   3abe4:	lea    0x28e8(%rsp),%rsi
   3abec:	call   *0xf9606(%rip)        # 1341f8 <memcpy@GLIBC_2.14>
   3abf2:	inc    %r14
   3abf5:	mov    %r14,0x10(%rbx)
   3abf9:	mov    %r13,%rdi
   3abfc:	lea    0xf000(%r13),%r9
   3ac03:	mov    (%rsp),%r10
   3ac07:	cmp    %r10,%r9
   3ac0a:	cmovae %r10,%r9
   3ac0e:	lea    0x10(%r13),%r11
   3ac12:	cmp    %r9,%r11
   3ac15:	mov    %r9,0x60(%rsp)
   3ac1a:	jbe    3ace0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa40>
   3ac20:	sub    %rdi,%r9
   3ac23:	jbe    3aa60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7c0>
   3ac29:	mov    %r12,0x30(%rsp)
   3ac2e:	mov    %r15,0x38(%rsp)
   3ac33:	mov    0x28e0(%rsp),%r13d
   3ac3b:	mov    $0x1e,%r15d
   3ac41:	sub    0x28d8(%rsp),%r15
   3ac49:	cmp    %r9,%r15
   3ac4c:	cmovae %r9,%r15
   3ac50:	lea    (%rdi,%rbp,1),%r12
   3ac54:	test   %r15,%r15
   3ac57:	je     3b710 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3ac5d:	xor    %r14d,%r14d
   3ac60:	movzbl (%r12,%r14,1),%ebx
   3ac65:	mov    0xd8(%rsp,%rbx,8),%rax
   3ac6d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3ac75:	je     3ac97 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9f7>
   3ac77:	lea    (%r14,%r13,1),%rcx
   3ac7b:	mov    %cx,(%rax)
   3ac7e:	add    $0x2,%rax
   3ac82:	mov    %rax,0xd8(%rsp,%rbx,8)
   3ac8a:	inc    %r14
   3ac8d:	cmp    %r14,%r15
   3ac90:	jne    3ac60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9c0>
   3ac92:	jmp    3b710 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3ac97:	mov    %r9,0x10(%rsp)
   3ac9c:	mov    %rdi,0x8(%rsp)
   3aca1:	lea    0x98(%rsp),%rdi
   3aca9:	mov    %rbx,%rsi
   3acac:	call   92860 <<c30walk::ctor2d::W<false>>::refill>
   3acb1:	test   %al,%al
   3acb3:	je     3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3acb9:	mov    0xd8(%rsp,%rbx,8),%rax
   3acc1:	mov    (%rsp),%r10
   3acc5:	mov    0x40(%rsp),%rbp
   3acca:	mov    0x8(%rsp),%rdi
   3accf:	mov    0x10(%rsp),%r9
   3acd4:	jmp    3ac77 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9d7>
   3acd6:	cs nopw 0x0(%rax,%rax,1)
   3ace0:	xor    %edx,%edx
   3ace2:	lea    0x10d8(%rsp),%r8
   3acea:	jmp    3ad53 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xab3>
   3acec:	nopl   0x0(%rax)
   3acf0:	mov    0x28d8(%rsp),%rax
   3acf8:	lea    0x10(%rax),%rcx
   3acfc:	add    $0xfffffffffffffff2,%rax
   3ad00:	xor    %edx,%edx
   3ad02:	cmp    $0x1e,%rcx
   3ad06:	setae  %dl
   3ad09:	cmovb  %rcx,%rax
   3ad0d:	mov    %rax,0x28d8(%rsp)
   3ad15:	mov    0x28e0(%rsp),%eax
   3ad1c:	lea    0x10(%rax,%rdx,2),%eax
   3ad20:	mov    %eax,0x28e0(%rsp)
   3ad27:	mov    0x20(%rsp),%rax
   3ad2c:	lea    0x10(%rax),%rcx
   3ad30:	xor    %edx,%edx
   3ad32:	mov    %rax,%rdi
   3ad35:	mov    %rcx,%r11
   3ad38:	mov    0x60(%rsp),%r9
   3ad3d:	cmp    %r9,%rcx
   3ad40:	mov    0x40(%rsp),%rbp
   3ad45:	lea    0x10d8(%rsp),%r8
   3ad4d:	ja     3ac20 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x980>
   3ad53:	mov    %r12,%rcx
   3ad56:	test   $0x1,%dl
   3ad59:	jne    3ad89 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xae9>
   3ad5b:	lea    0x8(%rdi),%rsi
   3ad5f:	cmp    $0xfffffffffffffff7,%rdi
   3ad63:	ja     3b931 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1691>
   3ad69:	cmp    %r10,%rsi
   3ad6c:	ja     3b931 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1691>
   3ad72:	lea    0x10(%rdi),%rax
   3ad76:	cmp    %r10,%rax
   3ad79:	ja     3bb94 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18f4>
   3ad7f:	mov    0x0(%rbp,%rdi,1),%r15
   3ad84:	mov    0x0(%rbp,%rsi,1),%rcx
   3ad89:	mov    %r15,%rax
   3ad8c:	xor    %rcx,%rax
   3ad8f:	mov    %r15,%rdx
   3ad92:	rol    $0x8,%rdx
   3ad96:	xor    %r15,%rdx
   3ad99:	or     %rax,%rdx
   3ad9c:	jne    3ae00 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb60>
   3ad9e:	mov    %rdi,%rbx
   3ada1:	mov    %r11,%rax
   3ada4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3adb0:	mov    %rax,%rdi
   3adb3:	add    $0x10,%rax
   3adb7:	cmp    %r9,%rax
   3adba:	setbe  %r11b
   3adbe:	ja     3ae60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc0>
   3adc4:	lea    0x8(%rdi),%rsi
   3adc8:	cmp    $0xfffffffffffffff7,%rdi
   3adcc:	ja     3b931 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1691>
   3add2:	cmp    %r10,%rsi
   3add5:	ja     3b931 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1691>
   3addb:	mov    0x0(%rbp,%rdi,1),%rdx
   3ade0:	mov    0x8(%rbp,%rdi,1),%rsi
   3ade5:	cmp    %rcx,%rdx
   3ade8:	jne    3adef <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb4f>
   3adea:	cmp    %rcx,%rsi
   3aded:	je     3adb0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb10>
   3adef:	mov    %rsi,0x30(%rsp)
   3adf4:	mov    %rdx,0x38(%rsp)
   3adf9:	lea    0x10(%rdi),%r9
   3adfd:	jmp    3ae64 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc4>
   3adff:	nop
   3ae00:	mov    %r11,0x20(%rsp)
   3ae05:	mov    0x28d8(%rsp),%rax
   3ae0d:	mov    $0x1e,%r13d
   3ae13:	sub    %rax,%r13
   3ae16:	cmp    $0x10,%r13
   3ae1a:	mov    $0x10,%r14d
   3ae20:	cmovb  %r13,%r14
   3ae24:	mov    %r14,%rbx
   3ae27:	add    %rdi,%rbx
   3ae2a:	jb     3bb82 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18e2>
   3ae30:	cmp    %r10,%rbx
   3ae33:	ja     3bb82 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18e2>
   3ae39:	mov    0x28e0(%rsp),%r12d
   3ae41:	cmp    $0x1e,%rax
   3ae45:	jne    3b600 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1360>
   3ae4b:	cmp    %rbx,0x20(%rsp)
   3ae50:	jb     3bba9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1909>
   3ae56:	add    %rbp,%rbx
   3ae59:	jmp    3b69a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13fa>
   3ae5e:	xchg   %ax,%ax
   3ae60:	lea    0x10(%rdi),%r9
   3ae64:	movzbl %cl,%r12d
   3ae68:	lea    (%r12,%r12,2),%rdx
   3ae6c:	mov    %r12d,%eax
   3ae6f:	mov    0xd8(%rsp,%rax,8),%rax
   3ae77:	mov    %rax,%rsi
   3ae7a:	sub    (%r8,%rdx,8),%rsi
   3ae7e:	mov    %rdi,0x20(%rsp)
   3ae83:	mov    %rdi,%rbp
   3ae86:	shr    $1,%rsi
   3ae89:	add    0x8(%r8,%rdx,8),%esi
   3ae8e:	sub    %rbx,%rbp
   3ae91:	add    %rbp,%rsi
   3ae94:	mov    0xc(%r8,%rdx,8),%edi
   3ae99:	cmp    %rdi,%rsi
   3ae9c:	ja     3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3aea2:	mov    %r9,0x70(%rsp)
   3aea7:	mov    %r11d,0x54(%rsp)
   3aeac:	lea    (%r8,%rdx,8),%rbx
   3aeb0:	movzbl %cl,%r14d
   3aeb4:	mov    0x8d8(%rsp,%r12,8),%rsi
   3aebc:	mov    $0x88888889,%r15d
   3aec2:	lea    -0x1fec9(%rip),%r13        # 1b000 <c30walk::ctor2d::PATTERNS>
   3aec9:	mov    %r12,0x90(%rsp)
   3aed1:	mov    %rbx,0x78(%rsp)
   3aed6:	jmp    3af16 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc76>
   3aed8:	nopl   0x0(%rax,%rax,1)
   3aee0:	mov    0x3c(%rsi),%edi
   3aee3:	mov    $0xfffff,%r9d
   3aee9:	and    %r9d,%edi
   3aeec:	sub    %r8d,%ecx
   3aeef:	sub    %eax,%r8d
   3aef2:	mov    %r8d,%eax
   3aef5:	lea    (%rsi,%rax,2),%rax
   3aef9:	mov    %rax,0xd8(%rsp,%r12,8)
   3af01:	lea    (%rax,%rcx,2),%rsi
   3af05:	mov    %rsi,0x8d8(%rsp,%r12,8)
   3af0d:	mov    %rax,(%rbx)
   3af10:	mov    %edx,0x10(%rbx)
   3af13:	mov    %edi,0x14(%rbx)
   3af16:	sub    %rax,%rsi
   3af19:	shr    $1,%rsi
   3af1c:	cmp    %rsi,%rbp
   3af1f:	cmovb  %rbp,%rsi
   3af23:	test   %rsi,%rsi
   3af26:	je     3afdd <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd3d>
   3af2c:	mov    0x28d8(%rsp),%rdi
   3af34:	cmp    $0x1d,%rdi
   3af38:	ja     3bc78 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19d8>
   3af3e:	cmp    $0x1f,%rsi
   3af42:	jae    3babf <<c30walk::ctor2d::Ctor2d>::build::<false>+0x181f>
   3af48:	mov    0x28e0(%rsp),%ecx
   3af4f:	mov    %rdi,%r11
   3af52:	shl    $0x6,%r11
   3af56:	lea    (%r11,%r13,1),%rdx
   3af5a:	lea    -0x2(,%rsi,2),%r9
   3af62:	cmp    $0xe,%r9
   3af66:	jb     3af7b <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcdb>
   3af68:	mov    %rax,%r8
   3af6b:	sub    %r13,%r8
   3af6e:	sub    %r11,%r8
   3af71:	cmp    $0x10,%r8
   3af75:	jae    3b3e3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1143>
   3af7b:	xor    %r10d,%r10d
   3af7e:	mov    %rdx,%r8
   3af81:	lea    (%rdx,%rsi,2),%rdx
   3af85:	lea    (%rax,%r10,2),%r9
   3af89:	nopl   0x0(%rax)
   3af90:	movzwl (%r8),%r10d
   3af94:	add    %cx,%r10w
   3af98:	add    $0x2,%r8
   3af9c:	mov    %r10w,(%r9)
   3afa0:	add    $0x2,%r9
   3afa4:	cmp    %rdx,%r8
   3afa7:	jne    3af90 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcf0>
   3afa9:	lea    (%rax,%rsi,2),%rax
   3afad:	mov    %rax,0xd8(%rsp,%r12,8)
   3afb5:	add    %rsi,%rdi
   3afb8:	lea    0x2(%rsi),%edx
   3afbb:	mov    %rdi,%r8
   3afbe:	sub    $0x1e,%r8
   3afc2:	cmovb  %rdi,%r8
   3afc6:	cmovb  %esi,%edx
   3afc9:	mov    %r8,0x28d8(%rsp)
   3afd1:	add    %ecx,%edx
   3afd3:	mov    %edx,0x28e0(%rsp)
   3afda:	sub    %rsi,%rbp
   3afdd:	test   %rbp,%rbp
   3afe0:	je     3b5d0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3afe6:	mov    %rax,%r8
   3afe9:	cmp    (%rbx),%rax
   3afec:	je     3b009 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd69>
   3afee:	lea    0x98(%rsp),%rdi
   3aff6:	mov    %r12,%rsi
   3aff9:	call   92250 <<c30walk::ctor2d::W<false>>::finish>
   3affe:	mov    0xd8(%rsp,%r12,8),%r8
   3b006:	mov    (%rbx),%rax
   3b009:	sub    %rax,%r8
   3b00c:	shr    $1,%r8
   3b00f:	add    0x8(%rbx),%r8d
   3b013:	mov    %r8d,0x8(%rbx)
   3b017:	movq   $0x0,(%rbx)
   3b01e:	movq   $0x0,0xd8(%rsp,%r12,8)
   3b02a:	movq   $0x0,0x8d8(%rsp,%r12,8)
   3b036:	cmp    $0x1e,%rbp
   3b03a:	jb     3b562 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3b040:	mov    %r8,%rcx
   3b043:	imul   %r15,%rcx
   3b047:	shr    $0x24,%rcx
   3b04b:	mov    %ecx,%eax
   3b04d:	shl    $0x5,%eax
   3b050:	lea    (%rcx,%rcx,1),%edx
   3b053:	sub    %eax,%edx
   3b055:	add    %r8d,%edx
   3b058:	jne    3b562 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3b05e:	mov    %rbp,%rax
   3b061:	movabs $0x8888888888888889,%rdx
   3b06b:	mul    %rdx
   3b06e:	shr    $0x4,%rdx
   3b072:	mov    %ecx,%r9d
   3b075:	cmpl   $0x0,0x1c(%rsp)
   3b07a:	mov    %rbp,0x28(%rsp)
   3b07f:	mov    %rdx,0x48(%rsp)
   3b084:	je     3b3a0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1100>
   3b08a:	mov    0x28e0(%rsp),%r10d
   3b092:	mov    0xa8(%rsp),%rax
   3b09a:	mov    %rax,0x8(%rsp)
   3b09f:	mov    0x98(%rsp),%r11
   3b0a7:	mov    0xa0(%rsp),%rax
   3b0af:	mov    %rax,0x10(%rsp)
   3b0b4:	mov    0xc0(%rsp),%rax
   3b0bc:	mov    0x28d8(%rsp),%r12
   3b0c4:	mov    %r12,%rdi
   3b0c7:	shl    $0x6,%rdi
   3b0cb:	add    %r13,%rdi
   3b0ce:	mov    %r9,%rcx
   3b0d1:	shl    $0x6,%rcx
   3b0d5:	lea    (%rax,%rcx,1),%r15
   3b0d9:	add    $0x3c,%r15
   3b0dd:	mov    %r10d,%ebp
   3b0e0:	shl    $0x8,%ebp
   3b0e3:	or     %r14d,%ebp
   3b0e6:	xor    %ebx,%ebx
   3b0e8:	jmp    3b30f <<c30walk::ctor2d::Ctor2d>::build::<false>+0x106f>
   3b0ed:	nopl   (%rax)
   3b0f0:	mov    0x8(%r11),%rcx
   3b0f4:	shl    $0x7,%eax
   3b0f7:	mov    %ebp,(%rcx,%rax,1)
   3b0fa:	movzwl 0x2(%rdi),%esi
   3b0fe:	lea    (%r10,%rbx,1),%r13d
   3b102:	add    %r13d,%esi
   3b105:	shl    $0x8,%esi
   3b108:	or     %r14d,%esi
   3b10b:	mov    %esi,0x4(%rcx,%rax,1)
   3b10f:	movzwl 0x4(%rdi),%esi
   3b113:	add    %r13d,%esi
   3b116:	shl    $0x8,%esi
   3b119:	or     %r14d,%esi
   3b11c:	mov    %esi,0x8(%rcx,%rax,1)
   3b120:	movzwl 0x6(%rdi),%esi
   3b124:	add    %r13d,%esi
   3b127:	shl    $0x8,%esi
   3b12a:	or     %r14d,%esi
   3b12d:	mov    %esi,0xc(%rcx,%rax,1)
   3b131:	movzwl 0x8(%rdi),%esi
   3b135:	add    %r13d,%esi
   3b138:	shl    $0x8,%esi
   3b13b:	or     %r14d,%esi
   3b13e:	mov    %esi,0x10(%rcx,%rax,1)
   3b142:	movzwl 0xa(%rdi),%esi
   3b146:	add    %r13d,%esi
   3b149:	shl    $0x8,%esi
   3b14c:	or     %r14d,%esi
   3b14f:	mov    %esi,0x14(%rcx,%rax,1)
   3b153:	movzwl 0xc(%rdi),%esi
   3b157:	add    %r13d,%esi
   3b15a:	shl    $0x8,%esi
   3b15d:	or     %r14d,%esi
   3b160:	mov    %esi,0x18(%rcx,%rax,1)
   3b164:	movzwl 0xe(%rdi),%esi
   3b168:	add    %r13d,%esi
   3b16b:	shl    $0x8,%esi
   3b16e:	or     %r14d,%esi
   3b171:	mov    %esi,0x1c(%rcx,%rax,1)
   3b175:	movzwl 0x10(%rdi),%esi
   3b179:	add    %r13d,%esi
   3b17c:	shl    $0x8,%esi
   3b17f:	or     %r14d,%esi
   3b182:	mov    %esi,0x20(%rcx,%rax,1)
   3b186:	movzwl 0x12(%rdi),%esi
   3b18a:	add    %r13d,%esi
   3b18d:	shl    $0x8,%esi
   3b190:	or     %r14d,%esi
   3b193:	mov    %esi,0x24(%rcx,%rax,1)
   3b197:	movzwl 0x14(%rdi),%esi
   3b19b:	add    %r13d,%esi
   3b19e:	shl    $0x8,%esi
   3b1a1:	or     %r14d,%esi
   3b1a4:	mov    %esi,0x28(%rcx,%rax,1)
   3b1a8:	movzwl 0x16(%rdi),%esi
   3b1ac:	add    %r13d,%esi
   3b1af:	shl    $0x8,%esi
   3b1b2:	or     %r14d,%esi
   3b1b5:	mov    %esi,0x2c(%rcx,%rax,1)
   3b1b9:	movzwl 0x18(%rdi),%esi
   3b1bd:	add    %r13d,%esi
   3b1c0:	shl    $0x8,%esi
   3b1c3:	or     %r14d,%esi
   3b1c6:	mov    %esi,0x30(%rcx,%rax,1)
   3b1ca:	movzwl 0x1a(%rdi),%esi
   3b1ce:	add    %r13d,%esi
   3b1d1:	shl    $0x8,%esi
   3b1d4:	or     %r14d,%esi
   3b1d7:	mov    %esi,0x34(%rcx,%rax,1)
   3b1db:	movzwl 0x1c(%rdi),%esi
   3b1df:	add    %r13d,%esi
   3b1e2:	shl    $0x8,%esi
   3b1e5:	or     %r14d,%esi
   3b1e8:	mov    %esi,0x38(%rcx,%rax,1)
   3b1ec:	movzwl 0x1e(%rdi),%esi
   3b1f0:	add    %r13d,%esi
   3b1f3:	shl    $0x8,%esi
   3b1f6:	or     %r14d,%esi
   3b1f9:	mov    %esi,0x3c(%rcx,%rax,1)
   3b1fd:	movzwl 0x20(%rdi),%esi
   3b201:	add    %r13d,%esi
   3b204:	shl    $0x8,%esi
   3b207:	or     %r14d,%esi
   3b20a:	mov    %esi,0x40(%rcx,%rax,1)
   3b20e:	movzwl 0x22(%rdi),%esi
   3b212:	add    %r13d,%esi
   3b215:	shl    $0x8,%esi
   3b218:	or     %r14d,%esi
   3b21b:	mov    %esi,0x44(%rcx,%rax,1)
   3b21f:	movzwl 0x24(%rdi),%esi
   3b223:	add    %r13d,%esi
   3b226:	shl    $0x8,%esi
   3b229:	or     %r14d,%esi
   3b22c:	mov    %esi,0x48(%rcx,%rax,1)
   3b230:	movzwl 0x26(%rdi),%esi
   3b234:	add    %r13d,%esi
   3b237:	shl    $0x8,%esi
   3b23a:	or     %r14d,%esi
   3b23d:	mov    %esi,0x4c(%rcx,%rax,1)
   3b241:	movzwl 0x28(%rdi),%esi
   3b245:	add    %r13d,%esi
   3b248:	shl    $0x8,%esi
   3b24b:	or     %r14d,%esi
   3b24e:	mov    %esi,0x50(%rcx,%rax,1)
   3b252:	movzwl 0x2a(%rdi),%esi
   3b256:	add    %r13d,%esi
   3b259:	shl    $0x8,%esi
   3b25c:	or     %r14d,%esi
   3b25f:	mov    %esi,0x54(%rcx,%rax,1)
   3b263:	movzwl 0x2c(%rdi),%esi
   3b267:	add    %r13d,%esi
   3b26a:	shl    $0x8,%esi
   3b26d:	or     %r14d,%esi
   3b270:	mov    %esi,0x58(%rcx,%rax,1)
   3b274:	movzwl 0x2e(%rdi),%esi
   3b278:	add    %r13d,%esi
   3b27b:	shl    $0x8,%esi
   3b27e:	or     %r14d,%esi
   3b281:	mov    %esi,0x5c(%rcx,%rax,1)
   3b285:	movzwl 0x30(%rdi),%esi
   3b289:	add    %r13d,%esi
   3b28c:	shl    $0x8,%esi
   3b28f:	or     %r14d,%esi
   3b292:	mov    %esi,0x60(%rcx,%rax,1)
   3b296:	movzwl 0x32(%rdi),%esi
   3b29a:	add    %r13d,%esi
   3b29d:	shl    $0x8,%esi
   3b2a0:	or     %r14d,%esi
   3b2a3:	mov    %esi,0x64(%rcx,%rax,1)
   3b2a7:	movzwl 0x34(%rdi),%esi
   3b2ab:	add    %r13d,%esi
   3b2ae:	shl    $0x8,%esi
   3b2b1:	or     %r14d,%esi
   3b2b4:	mov    %esi,0x68(%rcx,%rax,1)
   3b2b8:	movzwl 0x36(%rdi),%esi
   3b2bc:	add    %r13d,%esi
   3b2bf:	shl    $0x8,%esi
   3b2c2:	or     %r14d,%esi
   3b2c5:	mov    %esi,0x6c(%rcx,%rax,1)
   3b2c9:	movzwl 0x38(%rdi),%esi
   3b2cd:	add    %r13d,%esi
   3b2d0:	shl    $0x8,%esi
   3b2d3:	or     %r14d,%esi
   3b2d6:	mov    %esi,0x70(%rcx,%rax,1)
   3b2da:	movzwl 0x3a(%rdi),%esi
   3b2de:	add    %r13d,%esi
   3b2e1:	shl    $0x8,%esi
   3b2e4:	or     %r14d,%esi
   3b2e7:	mov    %esi,0x74(%rcx,%rax,1)
   3b2eb:	lea    (%r10,%rbx,1),%eax
   3b2ef:	add    $0x20,%eax
   3b2f2:	mov    %eax,0x28e0(%rsp)
   3b2f9:	add    $0x20,%rbx
   3b2fd:	inc    %r9
   3b300:	add    $0x2000,%ebp
   3b306:	dec    %rdx
   3b309:	je     3b519 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1279>
   3b30f:	mov    %r9,%rax
   3b312:	shr    $0x6,%rax
   3b316:	cmp    0x8(%rsp),%rax
   3b31b:	jae    3bbc0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1920>
   3b321:	mov    0x10(%rsp),%rcx
   3b326:	mov    (%rcx,%rax,8),%rax
   3b32a:	bt     %r9,%rax
   3b32e:	jae    3b360 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x10c0>
   3b330:	cmp    $0x1e,%r12
   3b334:	jae    3bbe6 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1946>
   3b33a:	mov    (%r15,%rbx,2),%eax
   3b33e:	and    $0xfffff,%eax
   3b343:	mov    0x10(%r11),%rcx
   3b347:	cmp    %rax,%rcx
   3b34a:	ja     3b0f0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xe50>
   3b350:	jmp    3bbd4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1934>
   3b355:	data16 cs nopw 0x0(%rax,%rax,1)
   3b360:	cmp    $0x1e,%r12
   3b364:	jae    3bbfa <<c30walk::ctor2d::Ctor2d>::build::<false>+0x195a>
   3b36a:	movups 0x2c(%rdi),%xmm0
   3b36e:	movups %xmm0,-0x10(%r15,%rbx,2)
   3b374:	movdqa (%rdi),%xmm0
   3b378:	movdqa 0x10(%rdi),%xmm1
   3b37d:	movdqa 0x20(%rdi),%xmm2
   3b382:	movdqa %xmm2,-0x1c(%r15,%rbx,2)
   3b389:	movdqa %xmm1,-0x2c(%r15,%rbx,2)
   3b390:	movdqa %xmm0,-0x3c(%r15,%rbx,2)
   3b397:	mov    %ebp,(%r15,%rbx,2)
   3b39b:	jmp    3b2eb <<c30walk::ctor2d::Ctor2d>::build::<false>+0x104b>
   3b3a0:	mov    0x28d8(%rsp),%rdi
   3b3a8:	cmp    $0x1d,%rdi
   3b3ac:	ja     3bc89 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19e9>
   3b3b2:	shl    $0x6,%rdi
   3b3b6:	add    %r13,%rdi
   3b3b9:	mov    0x28e0(%rsp),%eax
   3b3c0:	mov    0xc0(%rsp),%rcx
   3b3c8:	shl    $0x6,%r9
   3b3cc:	mov    %eax,%r15d
   3b3cf:	shl    $0x8,%r15d
   3b3d3:	or     %r14d,%r15d
   3b3d6:	cmp    $0x3c,%rbp
   3b3da:	jae    3b44a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11aa>
   3b3dc:	xor    %esi,%esi
   3b3de:	jmp    3b4d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1237>
   3b3e3:	shr    $1,%r9
   3b3e6:	inc    %r9
   3b3e9:	mov    %r9,%r10
   3b3ec:	and    $0xfffffffffffffff8,%r10
   3b3f0:	lea    (%rdx,%r10,2),%r8
   3b3f4:	movd   %ecx,%xmm0
   3b3f8:	pshuflw $0x0,%xmm0,%xmm0
   3b3fd:	add    %r13,%r11
   3b400:	add    $0x8,%r11
   3b404:	xor    %ebx,%ebx
   3b406:	cs nopw 0x0(%rax,%rax,1)
   3b410:	movq   -0x8(%r11,%rbx,2),%xmm1
   3b417:	movq   (%r11,%rbx,2),%xmm2
   3b41d:	paddw  %xmm0,%xmm1
   3b421:	paddw  %xmm0,%xmm2
   3b425:	punpcklqdq %xmm2,%xmm1
   3b429:	movdqu %xmm1,(%rax,%rbx,2)
   3b42e:	add    $0x8,%rbx
   3b432:	cmp    %rbx,%r10
   3b435:	jne    3b410 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1170>
   3b437:	cmp    %r10,%r9
   3b43a:	mov    0x78(%rsp),%rbx
   3b43f:	jne    3af81 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xce1>
   3b445:	jmp    3afa9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd09>
   3b44a:	mov    %rdx,%r10
   3b44d:	movabs $0xffffffffffffffe,%rsi
   3b457:	and    %rsi,%r10
   3b45a:	lea    (%rcx,%r9,1),%r11
   3b45e:	add    $0x7c,%r11
   3b462:	xor    %esi,%esi
   3b464:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b470:	movdqu 0x2c(%rdi),%xmm0
   3b475:	movdqu %xmm0,-0x50(%r11)
   3b47b:	movdqa (%rdi),%xmm1
   3b47f:	movdqa 0x10(%rdi),%xmm2
   3b484:	movaps 0x20(%rdi),%xmm3
   3b488:	movaps %xmm3,-0x5c(%r11)
   3b48d:	movdqa %xmm2,-0x6c(%r11)
   3b493:	movdqa %xmm1,-0x7c(%r11)
   3b499:	mov    %r15d,-0x40(%r11)
   3b49d:	add    $0x2,%rsi
   3b4a1:	movdqa %xmm1,-0x3c(%r11)
   3b4a7:	movdqa %xmm2,-0x2c(%r11)
   3b4ad:	movaps %xmm3,-0x1c(%r11)
   3b4b2:	movdqu %xmm0,-0x10(%r11)
   3b4b8:	lea    0x2000(%r15),%ebx
   3b4bf:	mov    %ebx,(%r11)
   3b4c2:	add    $0x4000,%r15d
   3b4c9:	sub    $0xffffffffffffff80,%r11
   3b4cd:	cmp    %rsi,%r10
   3b4d0:	jne    3b470 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11d0>
   3b4d2:	test   $0x1,%dl
   3b4d5:	je     3b50b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x126b>
   3b4d7:	add    %r9,%rcx
   3b4da:	shl    $0x6,%rsi
   3b4de:	movups 0x2c(%rdi),%xmm0
   3b4e2:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3b4e7:	movdqa (%rdi),%xmm0
   3b4eb:	movdqa 0x10(%rdi),%xmm1
   3b4f0:	movdqa 0x20(%rdi),%xmm2
   3b4f5:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3b4fb:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3b501:	movdqa %xmm0,(%rcx,%rsi,1)
   3b506:	mov    %r15d,0x3c(%rcx,%rsi,1)
   3b50b:	mov    %edx,%ecx
   3b50d:	shl    $0x5,%ecx
   3b510:	add    %ecx,%eax
   3b512:	mov    %eax,0x28e0(%rsp)
   3b519:	mov    0x48(%rsp),%rdx
   3b51e:	mov    %rdx,%rax
   3b521:	shl    $0x5,%rax
   3b525:	lea    (%rdx,%rdx,1),%rcx
   3b529:	sub    %rax,%rcx
   3b52c:	mov    0x28(%rsp),%rbp
   3b531:	add    %rcx,%rbp
   3b534:	lea    (%rdx,%rdx,1),%eax
   3b537:	shl    $0x5,%edx
   3b53a:	sub    %eax,%edx
   3b53c:	add    %edx,%r8d
   3b53f:	mov    0x78(%rsp),%rbx
   3b544:	mov    %r8d,0x8(%rbx)
   3b548:	test   %rbp,%rbp
   3b54b:	mov    $0x88888889,%r15d
   3b551:	lea    -0x20558(%rip),%r13        # 1b000 <c30walk::ctor2d::PATTERNS>
   3b558:	mov    0x90(%rsp),%r12
   3b560:	je     3b5d0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3b562:	mov    0xc(%rbx),%ecx
   3b565:	mov    %r8d,%edi
   3b568:	imul   %r15,%rdi
   3b56c:	mov    %rdi,%rdx
   3b56f:	shr    $0x24,%rdx
   3b573:	mov    %edx,%eax
   3b575:	shl    $0x5,%eax
   3b578:	sub    %edx,%eax
   3b57a:	sub    %edx,%eax
   3b57c:	lea    0x1e(%rax),%esi
   3b57f:	cmp    %ecx,%esi
   3b581:	cmovb  %esi,%ecx
   3b584:	shr    $0x2a,%rdi
   3b588:	mov    0xa8(%rsp),%rsi
   3b590:	cmp    %rsi,%rdi
   3b593:	jae    3bc6c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19cc>
   3b599:	mov    %rdx,%rsi
   3b59c:	shl    $0x6,%rsi
   3b5a0:	add    0xc0(%rsp),%rsi
   3b5a8:	mov    0xa0(%rsp),%r9
   3b5b0:	mov    (%r9,%rdi,8),%rdi
   3b5b4:	bt     %rdx,%rdi
   3b5b8:	jb     3aee0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc40>
   3b5be:	mov    $0xffffffff,%edi
   3b5c3:	jmp    3aeec <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc4c>
   3b5c8:	nopl   0x0(%rax,%rax,1)
   3b5d0:	mov    (%rsp),%r10
   3b5d4:	mov    0x38(%rsp),%r15
   3b5d9:	mov    0x30(%rsp),%r12
   3b5de:	mov    0x20(%rsp),%rax
   3b5e3:	mov    0x54(%rsp),%edx
   3b5e7:	mov    0x70(%rsp),%rcx
   3b5ec:	jmp    3ad32 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa92>
   3b5f1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b600:	add    %rbp,%rdi
   3b603:	xor    %ebp,%ebp
   3b605:	data16 cs nopw 0x0(%rax,%rax,1)
   3b610:	movzbl (%rdi,%rbp,1),%r15d
   3b615:	mov    0xd8(%rsp,%r15,8),%rax
   3b61d:	cmp    0x8d8(%rsp,%r15,8),%rax
   3b625:	je     3b644 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13a4>
   3b627:	lea    (%r12,%rbp,1),%rcx
   3b62b:	mov    %cx,(%rax)
   3b62e:	add    $0x2,%rax
   3b632:	mov    %rax,0xd8(%rsp,%r15,8)
   3b63a:	inc    %rbp
   3b63d:	cmp    %rbp,%r14
   3b640:	jne    3b610 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1370>
   3b642:	jmp    3b680 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13e0>
   3b644:	mov    %rdi,0x8(%rsp)
   3b649:	lea    0x98(%rsp),%rdi
   3b651:	mov    %r15,%rsi
   3b654:	call   92860 <<c30walk::ctor2d::W<false>>::refill>
   3b659:	test   %al,%al
   3b65b:	je     3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3b661:	mov    0xd8(%rsp,%r15,8),%rax
   3b669:	mov    (%rsp),%r10
   3b66d:	mov    0x8(%rsp),%rdi
   3b672:	jmp    3b627 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1387>
   3b674:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b680:	cmp    %rbx,0x20(%rsp)
   3b685:	jb     3bba9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1909>
   3b68b:	cmp    $0xf,%r13
   3b68f:	ja     3acf0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3b695:	add    0x40(%rsp),%rbx
   3b69a:	mov    %rbx,%r13
   3b69d:	sub    %r14,%r13
   3b6a0:	add    $0x10,%r13
   3b6a4:	lea    (%r12,%r14,1),%ebp
   3b6a8:	add    $0x2,%ebp
   3b6ab:	nopl   0x0(%rax,%rax,1)
   3b6b0:	movzbl (%rbx),%r15d
   3b6b4:	mov    0xd8(%rsp,%r15,8),%rax
   3b6bc:	cmp    0x8d8(%rsp,%r15,8),%rax
   3b6c4:	je     3b6e4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1444>
   3b6c6:	inc    %rbx
   3b6c9:	mov    %bp,(%rax)
   3b6cc:	add    $0x2,%rax
   3b6d0:	mov    %rax,0xd8(%rsp,%r15,8)
   3b6d8:	inc    %ebp
   3b6da:	cmp    %r13,%rbx
   3b6dd:	jne    3b6b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1410>
   3b6df:	jmp    3acf0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3b6e4:	lea    0x98(%rsp),%rdi
   3b6ec:	mov    %r15,%rsi
   3b6ef:	call   92860 <<c30walk::ctor2d::W<false>>::refill>
   3b6f4:	test   %al,%al
   3b6f6:	je     3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3b6fc:	mov    0xd8(%rsp,%r15,8),%rax
   3b704:	mov    (%rsp),%r10
   3b708:	jmp    3b6c6 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1426>
   3b70a:	nopw   0x0(%rax,%rax,1)
   3b710:	lea    (%r15,%rdi,1),%rax
   3b714:	mov    0x60(%rsp),%r8
   3b719:	cmp    %r8,%rax
   3b71c:	jne    3b763 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14c3>
   3b71e:	cmp    $0x10,%r9
   3b722:	jne    3b7e7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1547>
   3b728:	mov    0x28d8(%rsp),%rax
   3b730:	lea    0x10(%rax),%rcx
   3b734:	cmp    $0x1d,%rcx
   3b738:	mov    0x38(%rsp),%r15
   3b73d:	mov    0x30(%rsp),%r12
   3b742:	jbe    3b88a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15ea>
   3b748:	add    $0xfffffffffffffff2,%rax
   3b74c:	mov    %rax,0x28d8(%rsp)
   3b754:	mov    0x28e0(%rsp),%eax
   3b75b:	add    $0x12,%eax
   3b75e:	jmp    3b89c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15fc>
   3b763:	add    $0x2,%r13d
   3b767:	movzwl %r13w,%r13d
   3b76b:	nopl   0x0(%rax,%rax,1)
   3b770:	movzbl (%r12,%r15,1),%ebx
   3b775:	mov    0xd8(%rsp,%rbx,8),%rax
   3b77d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3b785:	je     3b7a7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1507>
   3b787:	lea    (%r15,%r13,1),%rcx
   3b78b:	mov    %cx,(%rax)
   3b78e:	add    $0x2,%rax
   3b792:	mov    %rax,0xd8(%rsp,%rbx,8)
   3b79a:	inc    %r15
   3b79d:	cmp    %r15,%r9
   3b7a0:	jne    3b770 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14d0>
   3b7a2:	jmp    3b71e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x147e>
   3b7a7:	mov    %r9,0x10(%rsp)
   3b7ac:	mov    %rdi,%r14
   3b7af:	lea    0x98(%rsp),%rdi
   3b7b7:	mov    %rbx,%rsi
   3b7ba:	call   92860 <<c30walk::ctor2d::W<false>>::refill>
   3b7bf:	test   %al,%al
   3b7c1:	je     3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3b7c7:	mov    0xd8(%rsp,%rbx,8),%rax
   3b7cf:	mov    (%rsp),%r10
   3b7d3:	mov    0x40(%rsp),%rbp
   3b7d8:	mov    0x60(%rsp),%r8
   3b7dd:	mov    %r14,%rdi
   3b7e0:	mov    0x10(%rsp),%r9
   3b7e5:	jmp    3b787 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14e7>
   3b7e7:	mov    0x28e0(%rsp),%eax
   3b7ee:	mov    0x28d8(%rsp),%rcx
   3b7f6:	inc    %rdi
   3b7f9:	cmp    %rdi,%r8
   3b7fc:	mov    0x38(%rsp),%r15
   3b801:	mov    0x30(%rsp),%r12
   3b806:	je     3b865 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15c5>
   3b808:	mov    %r9d,%edx
   3b80b:	and    $0xffffe,%edx
   3b811:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b820:	lea    0x1(%rcx),%rsi
   3b824:	add    $0xffffffffffffffe3,%rcx
   3b828:	xor    %edi,%edi
   3b82a:	cmp    $0x1e,%rsi
   3b82e:	setae  %dil
   3b832:	cmovb  %rsi,%rcx
   3b836:	lea    (%rax,%rdi,2),%eax
   3b839:	lea    0x1(%rcx),%rsi
   3b83d:	add    $0xffffffffffffffe3,%rcx
   3b841:	xor    %edi,%edi
   3b843:	cmp    $0x1e,%rsi
   3b847:	setae  %dil
   3b84b:	cmovb  %rsi,%rcx
   3b84f:	lea    (%rax,%rdi,2),%eax
   3b852:	add    $0x2,%eax
   3b855:	add    $0xfffffffffffffffe,%rdx
   3b859:	jne    3b820 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1580>
   3b85b:	test   $0x1,%r9b
   3b85f:	mov    (%rsp),%r10
   3b863:	je     3b880 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15e0>
   3b865:	lea    0x1(%rcx),%rdx
   3b869:	add    $0xffffffffffffffe3,%rcx
   3b86d:	xor    %esi,%esi
   3b86f:	cmp    $0x1e,%rdx
   3b873:	setae  %sil
   3b877:	cmovb  %rdx,%rcx
   3b87b:	lea    (%rax,%rsi,2),%eax
   3b87e:	inc    %eax
   3b880:	mov    %rcx,0x28d8(%rsp)
   3b888:	jmp    3b89c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15fc>
   3b88a:	mov    %rcx,0x28d8(%rsp)
   3b892:	mov    0x28e0(%rsp),%eax
   3b899:	add    $0x10,%eax
   3b89c:	mov    %eax,0x28e0(%rsp)
   3b8a3:	mov    %r8,%rdi
   3b8a6:	cmp    %r10,%r8
   3b8a9:	jb     3aa6c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3b8af:	jmp    3b8be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x161e>
   3b8b1:	mov    %rbx,%rdi
   3b8b4:	call   e6130 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3b8b9:	jmp    3abd4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x934>
   3b8be:	mov    $0x9,%eax
   3b8c3:	lea    0x10fc(%rsp),%rcx
   3b8cb:	nopl   0x0(%rax,%rax,1)
   3b8d0:	cmp    $0x109,%rax
   3b8d6:	je     3b940 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16a0>
   3b8d8:	mov    0x90(%rsp,%rax,8),%rdx
   3b8e0:	sub    -0x24(%rcx),%rdx
   3b8e4:	shr    $1,%rdx
   3b8e7:	add    -0x1c(%rcx),%edx
   3b8ea:	cmp    -0x18(%rcx),%edx
   3b8ed:	jne    3b90d <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3b8ef:	mov    0x98(%rsp,%rax,8),%rdx
   3b8f7:	sub    -0xc(%rcx),%rdx
   3b8fb:	shr    $1,%rdx
   3b8fe:	add    -0x4(%rcx),%edx
   3b901:	add    $0x2,%rax
   3b905:	cmp    (%rcx),%edx
   3b907:	lea    0x30(%rcx),%rcx
   3b90b:	je     3b8d0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1630>
   3b90d:	mov    0x4520(%rsp),%rax
   3b915:	movq   $0x0,0x10(%rax)
   3b91d:	xor    %eax,%eax
   3b91f:	add    $0x44e8,%rsp
   3b926:	pop    %rbx
   3b927:	pop    %r12
   3b929:	pop    %r13
   3b92b:	pop    %r14
   3b92d:	pop    %r15
   3b92f:	pop    %rbp
   3b930:	ret
   3b931:	lea    0xf5f38(%rip),%rcx        # 131870 <__frame_dummy_init_array_entry+0x3950>
   3b938:	mov    %r10,%rdx
   3b93b:	call   d8310 <core::slice::index::slice_index_fail>
   3b940:	mov    $0x8,%r14d
   3b946:	mov    $0x1040,%r15d
   3b94c:	lea    0x98(%rsp),%rbx
   3b954:	mov    0x88(%rsp),%r12
   3b95c:	mov    (%rsp),%rdx
   3b960:	mov    0x4520(%rsp),%rsi
   3b968:	mov    0x80(%rsp),%r13
   3b970:	jmp    3b9c2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1722>
   3b972:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b980:	sub    %rcx,%rax
   3b983:	shr    $1,%rax
   3b986:	add    %eax,0xa0(%rsp,%r15,1)
   3b98e:	movq   $0x0,0x98(%rsp,%r15,1)
   3b99a:	movq   $0x0,0x98(%rsp,%r14,8)
   3b9a6:	movq   $0x0,0x898(%rsp,%r14,8)
   3b9b2:	inc    %r14
   3b9b5:	add    $0x18,%r15
   3b9b9:	cmp    $0x108,%r14
   3b9c0:	je     3ba04 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1764>
   3b9c2:	mov    0x98(%rsp,%r14,8),%rcx
   3b9ca:	mov    %rcx,%rax
   3b9cd:	cmp    0x98(%rsp,%r15,1),%rcx
   3b9d5:	je     3b980 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16e0>
   3b9d7:	lea    -0x8(%r14),%rsi
   3b9db:	mov    %rbx,%rdi
   3b9de:	call   92250 <<c30walk::ctor2d::W<false>>::finish>
   3b9e3:	mov    0x4520(%rsp),%rsi
   3b9eb:	mov    (%rsp),%rdx
   3b9ef:	mov    0x98(%rsp,%r14,8),%rax
   3b9f7:	mov    0x98(%rsp,%r15,1),%rcx
   3b9ff:	jmp    3b980 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16e0>
   3ba04:	xor    %ecx,%ecx
   3ba06:	mov    0x8(%r12),%rbx
   3ba0b:	mov    0x58(%rsp),%r14
   3ba10:	test   %r14,%r14
   3ba13:	setne  %al
   3ba16:	je     3ba62 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17c2>
   3ba18:	mov    %al,%cl
   3ba1a:	mov    %r14,%rax
   3ba1d:	sub    %rcx,%rax
   3ba20:	imul   $0xffffffffffffffe2,%rax,%rcx
   3ba24:	add    %rdx,%rcx
   3ba27:	cmp    $0x1e,%rcx
   3ba2b:	jae    3ba62 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17c2>
   3ba2d:	lea    (%rbx,%r14,4),%rcx
   3ba31:	lea    (%rcx,%rdx,2),%rdi
   3ba35:	add    $0xfffffffffffffffc,%rdi
   3ba39:	lea    (%rax,%rax,1),%rcx
   3ba3d:	shl    $0x5,%rax
   3ba41:	sub    %rcx,%rax
   3ba44:	sub    %rdx,%rax
   3ba47:	lea    0x3c(,%rax,2),%rdx
   3ba4f:	mov    $0xff,%esi
   3ba54:	call   *0xf877e(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   3ba5a:	mov    0x4520(%rsp),%rsi
   3ba62:	shl    $0x6,%r14
   3ba66:	pcmpeqd %xmm0,%xmm0
   3ba6a:	movdqu %xmm0,0x2c(%rbx,%r14,1)
   3ba71:	movdqa %xmm0,0x20(%rbx,%r14,1)
   3ba78:	movdqa %xmm0,0x10(%rbx,%r14,1)
   3ba7f:	movdqa %xmm0,(%rbx,%r14,1)
   3ba85:	movl   $0x100fffff,0x3c(%rbx,%r14,1)
   3ba8e:	mov    0x8(%r13),%rdx
   3ba92:	mov    0x10(%r13),%rcx
   3ba96:	mov    0x8(%rsi),%r8
   3ba9a:	mov    0x10(%rsi),%r9
   3ba9e:	lea    0x98(%rsp),%rdi
   3baa6:	mov    %rbx,%rsi
   3baa9:	call   99c90 <c30walk::ctor::canonicalise>
   3baae:	mov    0x68(%rsp),%rax
   3bab3:	mov    %rax,0x10(%r12)
   3bab8:	mov    $0x1,%al
   3baba:	jmp    3b91f <<c30walk::ctor2d::Ctor2d>::build::<false>+0x167f>
   3babf:	lea    0xf4aba(%rip),%rcx        # 130580 <__frame_dummy_init_array_entry+0x2660>
   3bac6:	mov    $0x1e,%edx
   3bacb:	xor    %edi,%edi
   3bacd:	call   d8310 <core::slice::index::slice_index_fail>
   3bad2:	test   %rax,%rax
   3bad5:	je     3bae7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1847>
   3bad7:	mov    0x8(%r9),%rdi
   3badb:	mov    %r9,%r14
   3bade:	call   *0xf8704(%rip)        # 1341e8 <free@GLIBC_2.2.5>
   3bae4:	mov    %r14,%r9
   3bae7:	movq   $0x0,(%r9)
   3baee:	movq   $0x40,0x8(%r9)
   3baf6:	movq   $0x0,0x10(%r9)
   3bafe:	mov    $0x40,%edx
   3bb03:	mov    %r9,%rdi
   3bb06:	mov    0x68(%rsp),%rsi
   3bb0b:	mov    %r9,%r14
   3bb0e:	call   8f120 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3bb13:	mov    %r14,%r9
   3bb16:	cmp    $0xffffffffffffffff,%rax
   3bb1a:	mov    0x58(%rsp),%r8
   3bb1f:	je     3a4c3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x223>
   3bb25:	jmp    3bb77 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18d7>
   3bb27:	mov    %r9,%r14
   3bb2a:	test   %rax,%rax
   3bb2d:	je     3bb41 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18a1>
   3bb2f:	mov    0x8(%rdi),%rdi
   3bb33:	call   *0xf86af(%rip)        # 1341e8 <free@GLIBC_2.2.5>
   3bb39:	mov    0x4520(%rsp),%rdi
   3bb41:	movq   $0x0,(%rdi)
   3bb48:	movq   $0x40,0x8(%rdi)
   3bb50:	movq   $0x0,0x10(%rdi)
   3bb58:	mov    $0x80,%edx
   3bb5d:	mov    %r15,%rsi
   3bb60:	call   8f120 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3bb65:	cmp    $0xffffffffffffffff,%rax
   3bb69:	mov    %r14,%r9
   3bb6c:	mov    0x58(%rsp),%r8
   3bb71:	je     3a4d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x237>
   3bb77:	mov    %rax,%rdi
   3bb7a:	mov    %rdx,%rsi
   3bb7d:	call   d5a9b <alloc::raw_vec::handle_error>
   3bb82:	lea    0xf4a27(%rip),%rcx        # 1305b0 <__frame_dummy_init_array_entry+0x2690>
   3bb89:	mov    %r10,%rdx
   3bb8c:	mov    %rbx,%rsi
   3bb8f:	call   d8310 <core::slice::index::slice_index_fail>
   3bb94:	lea    0xf5cd5(%rip),%rcx        # 131870 <__frame_dummy_init_array_entry+0x3950>
   3bb9b:	mov    %r10,%rdx
   3bb9e:	mov    %rsi,%rdi
   3bba1:	mov    %rax,%rsi
   3bba4:	call   d8310 <core::slice::index::slice_index_fail>
   3bba9:	lea    0xf49e8(%rip),%rcx        # 130598 <__frame_dummy_init_array_entry+0x2678>
   3bbb0:	mov    %r10,%rdx
   3bbb3:	mov    %rbx,%rdi
   3bbb6:	mov    0x20(%rsp),%rsi
   3bbbb:	call   d8310 <core::slice::index::slice_index_fail>
   3bbc0:	lea    0xf2571(%rip),%rdx        # 12e138 <__frame_dummy_init_array_entry+0x218>
   3bbc7:	mov    %rax,%rdi
   3bbca:	mov    0x8(%rsp),%rsi
   3bbcf:	call   d7533 <core::panicking::panic_bounds_check>
   3bbd4:	lea    0xf4a65(%rip),%rdx        # 130640 <__frame_dummy_init_array_entry+0x2720>
   3bbdb:	mov    %rax,%rdi
   3bbde:	mov    %rcx,%rsi
   3bbe1:	call   d7533 <core::panicking::panic_bounds_check>
   3bbe6:	lea    0xf4a3b(%rip),%rdx        # 130628 <__frame_dummy_init_array_entry+0x2708>
   3bbed:	mov    $0x1e,%esi
   3bbf2:	mov    %r12,%rdi
   3bbf5:	call   d7533 <core::panicking::panic_bounds_check>
   3bbfa:	lea    0xf4a0f(%rip),%rdx        # 130610 <__frame_dummy_init_array_entry+0x26f0>
   3bc01:	mov    $0x1e,%esi
   3bc06:	mov    %r12,%rdi
   3bc09:	call   d7533 <core::panicking::panic_bounds_check>
   3bc0e:	mov    $0x10,%edx
   3bc13:	mov    $0x4,%ecx
   3bc18:	mov    $0x400,%r8d
   3bc1e:	mov    %r15,%rdi
   3bc21:	xor    %esi,%esi
   3bc23:	mov    %r9,%r14
   3bc26:	call   675f0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3bc2b:	mov    %r14,%r9
   3bc2e:	jmp    3a55b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2bb>
   3bc33:	mov    $0x8,%ecx
   3bc38:	mov    $0x8,%r8d
   3bc3e:	mov    %rbp,%rdi
   3bc41:	xor    %esi,%esi
   3bc43:	mov    %r15,%rdx
   3bc46:	mov    %r9,%r14
   3bc49:	call   675f0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3bc4e:	mov    %r14,%r9
   3bc51:	mov    0x10(%rbp),%r14
   3bc55:	mov    0x8(%rbp),%r12
   3bc59:	lea    (%r12,%r14,8),%rdi
   3bc5d:	cmp    $0x1,%r15
   3bc61:	jne    3a509 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x269>
   3bc67:	jmp    3a531 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3bc6c:	lea    0xf24c5(%rip),%rdx        # 12e138 <__frame_dummy_init_array_entry+0x218>
   3bc73:	call   d7533 <core::panicking::panic_bounds_check>
   3bc78:	lea    0xf48e9(%rip),%rdx        # 130568 <__frame_dummy_init_array_entry+0x2648>
   3bc7f:	mov    $0x1e,%esi
   3bc84:	call   d7533 <core::panicking::panic_bounds_check>
   3bc89:	lea    0xf4968(%rip),%rdx        # 1305f8 <__frame_dummy_init_array_entry+0x26d8>
   3bc90:	mov    $0x1e,%esi
   3bc95:	call   d7533 <core::panicking::panic_bounds_check>
   3bc9a:	lea    0xf484f(%rip),%rdx        # 1304f0 <__frame_dummy_init_array_entry+0x25d0>
   3bca1:	mov    0x8(%rsp),%rsi
   3bca6:	call   d7533 <core::panicking::panic_bounds_check>
   3bcab:	lea    0xf2486(%rip),%rdx        # 12e138 <__frame_dummy_init_array_entry+0x218>
   3bcb2:	mov    %r12,%rdi
   3bcb5:	call   d7533 <core::panicking::panic_bounds_check>
