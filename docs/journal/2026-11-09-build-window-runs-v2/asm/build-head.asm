tgt-head/release/c30cg:     file format elf64-x86-64
Disassembly of section .text:
000000000003e160 <mibz2::bwt::build>:
   3e160:	push   %rbp
   3e161:	push   %r15
   3e163:	push   %r14
   3e165:	push   %r13
   3e167:	push   %r12
   3e169:	push   %rbx
   3e16a:	sub    $0x818,%rsp
   3e171:	mov    %rdx,%r15
   3e174:	mov    %rsi,%rbx
   3e177:	mov    %rdi,%r14
   3e17a:	movq   $0x0,0x10(%rcx)
   3e182:	cmp    $0xffffff,%rsi
   3e189:	ja     3e308 <mibz2::bwt::build+0x1a8>
   3e18f:	pxor   %xmm0,%xmm0
   3e193:	xor    %eax,%eax
   3e195:	pxor   %xmm2,%xmm2
   3e199:	pxor   %xmm1,%xmm1
   3e19d:	nopl   (%rax)
   3e1a0:	movq   (%r15,%rax,4),%xmm3
   3e1a6:	movq   0x8(%r15,%rax,4),%xmm4
   3e1ad:	punpckldq %xmm0,%xmm3
   3e1b1:	movdqa %xmm2,%xmm5
   3e1b5:	paddq  %xmm3,%xmm5
   3e1b9:	punpckldq %xmm0,%xmm4
   3e1bd:	movdqa %xmm1,%xmm3
   3e1c1:	paddq  %xmm4,%xmm3
   3e1c5:	movq   0x10(%r15,%rax,4),%xmm2
   3e1cc:	movq   0x18(%r15,%rax,4),%xmm1
   3e1d3:	punpckldq %xmm0,%xmm2
   3e1d7:	paddq  %xmm5,%xmm2
   3e1db:	punpckldq %xmm0,%xmm1
   3e1df:	paddq  %xmm3,%xmm1
   3e1e3:	add    $0x8,%rax
   3e1e7:	cmp    $0x100,%rax
   3e1ed:	jne    3e1a0 <mibz2::bwt::build+0x40>
   3e1ef:	paddq  %xmm2,%xmm1
   3e1f3:	pshufd $0xee,%xmm1,%xmm0
   3e1f8:	paddq  %xmm1,%xmm0
   3e1fc:	movq   %xmm0,%rax
   3e201:	cmp    %rbx,%rax
   3e204:	jne    3e308 <mibz2::bwt::build+0x1a8>
   3e20a:	mov    %rcx,(%rsp)
   3e20e:	lea    0x18(%rsp),%rdi
   3e213:	xor    %ebp,%ebp
   3e215:	mov    0x5fadc(%rip),%r12        # 9dcf8 <memset@GLIBC_2.2.5>
   3e21c:	mov    $0x400,%edx
   3e221:	xor    %esi,%esi
   3e223:	call   *%r12
   3e226:	lea    0x418(%rsp),%rdi
   3e22e:	mov    $0x400,%edx
   3e233:	xor    %esi,%esi
   3e235:	call   *%r12
   3e238:	mov    $0x1,%eax
   3e23d:	nopl   (%rax)
   3e240:	mov    %ebp,0x414(%rsp,%rax,4)
   3e247:	add    -0x4(%r15,%rax,4),%ebp
   3e24c:	mov    %ebp,0x14(%rsp,%rax,4)
   3e250:	mov    %ebp,0x418(%rsp,%rax,4)
   3e257:	add    (%r15,%rax,4),%ebp
   3e25b:	mov    %ebp,0x18(%rsp,%rax,4)
   3e25f:	add    $0x2,%rax
   3e263:	cmp    $0x101,%rax
   3e269:	jne    3e240 <mibz2::bwt::build+0xe0>
   3e26b:	mov    (%rsp),%rax
   3e26f:	cmp    (%rax),%rbx
   3e272:	ja     3e3d6 <mibz2::bwt::build+0x276>
   3e278:	mov    0x8(%rax),%rcx
   3e27c:	cmp    $0x10,%rbx
   3e280:	jae    3e401 <mibz2::bwt::build+0x2a1>
   3e286:	xor    %edx,%edx
   3e288:	cmp    %rbx,%rdx
   3e28b:	jae    3e2e8 <mibz2::bwt::build+0x188>
   3e28d:	mov    %edx,%eax
   3e28f:	shl    $0x8,%eax
   3e292:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e2a0:	movzbl (%r14,%rdx,1),%r8d
   3e2a5:	lea    0x0(,%r8,4),%r9d
   3e2ad:	mov    0x418(%rsp,%r9,1),%edi
   3e2b5:	cmp    0x18(%rsp,%r9,1),%edi
   3e2ba:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e2bc:	cmp    %rdi,%rbx
   3e2bf:	jbe    3eb88 <mibz2::bwt::build+0xa28>
   3e2c5:	movzbl (%r14,%rdi,1),%r9d
   3e2ca:	add    %eax,%r9d
   3e2cd:	mov    %r9d,(%rcx,%rdi,4)
   3e2d1:	inc    %edi
   3e2d3:	mov    %edi,0x418(%rsp,%r8,4)
   3e2db:	inc    %rdx
   3e2de:	add    $0x100,%eax
   3e2e3:	cmp    %rdx,%rbx
   3e2e6:	jne    3e2a0 <mibz2::bwt::build+0x140>
   3e2e8:	lea    0x418(%rsp),%rdi
   3e2f0:	lea    0x18(%rsp),%rsi
   3e2f5:	mov    $0x400,%edx
   3e2fa:	call   *0x5fa00(%rip)        # 9dd00 <bcmp@GLIBC_2.2.5>
   3e300:	test   %eax,%eax
   3e302:	je     3e332 <mibz2::bwt::build+0x1d2>
   3e304:	mov    (%rsp),%rcx
   3e308:	movq   $0x0,0x10(%rcx)
   3e310:	cmp    (%rcx),%rbx
   3e313:	ja     3e33f <mibz2::bwt::build+0x1df>
   3e315:	mov    0x8(%rcx),%r13
   3e319:	xor    %r12d,%r12d
   3e31c:	test   %rbx,%rbx
   3e31f:	je     3ebe2 <mibz2::bwt::build+0xa82>
   3e325:	cmp    $0x8,%rbx
   3e329:	jae    3e36d <mibz2::bwt::build+0x20d>
   3e32b:	xor    %eax,%eax
   3e32d:	jmp    3ebcd <mibz2::bwt::build+0xa6d>
   3e332:	mov    (%rsp),%rax
   3e336:	mov    %rbx,0x10(%rax)
   3e33a:	jmp    3ebbb <mibz2::bwt::build+0xa5b>
   3e33f:	mov    %rcx,%r12
   3e342:	mov    $0x4,%ecx
   3e347:	mov    $0x4,%r8d
   3e34d:	mov    %r12,%rdi
   3e350:	xor    %esi,%esi
   3e352:	mov    %rbx,%rdx
   3e355:	call   3a0d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e35a:	mov    %r12,%rcx
   3e35d:	mov    0x8(%r12),%r13
   3e362:	mov    0x10(%r12),%r12
   3e367:	cmp    $0x8,%rbx
   3e36b:	jb     3e32b <mibz2::bwt::build+0x1cb>
   3e36d:	movabs $0x7ffffffffffffff0,%rax
   3e377:	or     $0x8,%rax
   3e37b:	and    %rbx,%rax
   3e37e:	lea    0x10(,%r12,4),%rdx
   3e386:	add    %r13,%rdx
   3e389:	add    %rax,%r12
   3e38c:	xor    %esi,%esi
   3e38e:	pxor   %xmm0,%xmm0
   3e392:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e3a0:	movd   (%r14,%rsi,1),%xmm1
   3e3a6:	movd   0x4(%r14,%rsi,1),%xmm2
   3e3ad:	punpcklbw %xmm0,%xmm1
   3e3b1:	punpcklwd %xmm0,%xmm1
   3e3b5:	punpcklbw %xmm0,%xmm2
   3e3b9:	punpcklwd %xmm0,%xmm2
   3e3bd:	movdqu %xmm1,-0x10(%rdx,%rsi,4)
   3e3c3:	movdqu %xmm2,(%rdx,%rsi,4)
   3e3c8:	add    $0x8,%rsi
   3e3cc:	cmp    %rsi,%rax
   3e3cf:	jne    3e3a0 <mibz2::bwt::build+0x240>
   3e3d1:	jmp    3ebdd <mibz2::bwt::build+0xa7d>
   3e3d6:	mov    $0x4,%ecx
   3e3db:	mov    $0x4,%r8d
   3e3e1:	mov    (%rsp),%rdi
   3e3e5:	xor    %esi,%esi
   3e3e7:	mov    %rbx,%rdx
   3e3ea:	call   3a0d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e3ef:	mov    (%rsp),%rax
   3e3f3:	mov    0x8(%rax),%rcx
   3e3f7:	cmp    $0x10,%rbx
   3e3fb:	jb     3e286 <mibz2::bwt::build+0x126>
   3e401:	mov    %ebx,%eax
   3e403:	and    $0xfffff0,%eax
   3e408:	mov    %rax,0x8(%rsp)
   3e40d:	mov    $0x10,%r9d
   3e413:	xor    %esi,%esi
   3e415:	pxor   %xmm0,%xmm0
   3e419:	movdqa -0x35ee1(%rip),%xmm1        # 8540 <GCC_except_table759+0x37c>
   3e421:	movdqa -0x361d9(%rip),%xmm2        # 8250 <GCC_except_table759+0x8c>
   3e429:	movdqa -0x36201(%rip),%xmm3        # 8230 <GCC_except_table759+0x6c>
   3e431:	movdqa -0x35f79(%rip),%xmm4        # 84c0 <GCC_except_table759+0x2fc>
   3e439:	mov    %rbx,%r11
   3e43c:	mov    %r14,%rdx
   3e43f:	movdqu (%r14,%rsi,1),%xmm5
   3e445:	movdqa %xmm5,%xmm6
   3e449:	pslldq $0x1,%xmm6
   3e44e:	pcmpeqb %xmm5,%xmm6
   3e452:	pmovmskb %xmm6,%ebp
   3e456:	and    $0xfffe,%ebp
   3e45c:	mov    %r9,0x10(%rsp)
   3e461:	je     3e6bc <mibz2::bwt::build+0x55c>
   3e467:	cmp    $0xfffe,%ebp
   3e46d:	jne    3e52b <mibz2::bwt::build+0x3cb>
   3e473:	cmp    %rbx,%rsi
   3e476:	jae    3eba9 <mibz2::bwt::build+0xa49>
   3e47c:	movzbl (%r14,%rsi,1),%eax
   3e481:	lea    0x0(,%rax,4),%r12d
   3e489:	mov    0x418(%rsp,%r12,1),%r8d
   3e491:	lea    0x10(%r8),%r9
   3e495:	mov    0x18(%rsp,%r12,1),%r12d
   3e49a:	cmp    %r12,%r9
   3e49d:	ja     3e304 <mibz2::bwt::build+0x1a4>
   3e4a3:	movdqu (%r14,%r8,1),%xmm6
   3e4a9:	movdqa %xmm6,%xmm5
   3e4ad:	punpckhbw %xmm0,%xmm5
   3e4b1:	movd   %esi,%xmm7
   3e4b5:	pshufd $0x0,%xmm7,%xmm7
   3e4ba:	pslld  $0x8,%xmm7
   3e4bf:	punpcklbw %xmm0,%xmm6
   3e4c3:	movdqa %xmm6,%xmm8
   3e4c8:	punpcklwd %xmm0,%xmm8
   3e4cd:	por    %xmm7,%xmm8
   3e4d2:	punpckhwd %xmm0,%xmm6
   3e4d6:	por    %xmm7,%xmm6
   3e4da:	movdqa %xmm5,%xmm9
   3e4df:	punpcklwd %xmm0,%xmm9
   3e4e4:	por    %xmm7,%xmm9
   3e4e9:	punpckhwd %xmm0,%xmm5
   3e4ed:	por    %xmm7,%xmm5
   3e4f1:	por    %xmm1,%xmm8
   3e4f6:	movdqu %xmm8,(%rcx,%r8,4)
   3e4fc:	por    %xmm2,%xmm6
   3e500:	movdqu %xmm6,0x10(%rcx,%r8,4)
   3e507:	por    %xmm3,%xmm9
   3e50c:	movdqu %xmm9,0x20(%rcx,%r8,4)
   3e513:	por    %xmm4,%xmm5
   3e517:	movdqu %xmm5,0x30(%rcx,%r8,4)
   3e51e:	mov    %r9d,0x418(%rsp,%rax,4)
   3e526:	jmp    3eb33 <mibz2::bwt::build+0x9d3>
   3e52b:	mov    %ebp,%eax
   3e52d:	shr    $0x8,%eax
   3e530:	mov    %ebp,%r8d
   3e533:	and    $0xfe,%r8d
   3e53a:	lea    -0x306b0(%rip),%r10        # de91 <mibz2::bwt::POP8>
   3e541:	movzbl (%rax,%r10,1),%eax
   3e546:	add    (%r8,%r10,1),%al
   3e54a:	cmp    $0xa,%al
   3e54c:	jb     3e6bc <mibz2::bwt::build+0x55c>
   3e552:	xor    $0xffff,%ebp
   3e558:	jmp    3e5f1 <mibz2::bwt::build+0x491>
   3e55d:	sub    %r9d,%r13d
   3e560:	movdqu (%r14,%r8,1),%xmm6
   3e566:	movdqa %xmm6,%xmm5
   3e56a:	punpckhbw %xmm0,%xmm5
   3e56e:	movd   %eax,%xmm7
   3e572:	pshufd $0x0,%xmm7,%xmm7
   3e577:	pslld  $0x8,%xmm7
   3e57c:	punpcklbw %xmm0,%xmm6
   3e580:	movdqa %xmm6,%xmm8
   3e585:	punpcklwd %xmm0,%xmm8
   3e58a:	por    %xmm7,%xmm8
   3e58f:	punpckhwd %xmm0,%xmm6
   3e593:	por    %xmm7,%xmm6
   3e597:	movdqa %xmm5,%xmm9
   3e59c:	punpcklwd %xmm0,%xmm9
   3e5a1:	por    %xmm7,%xmm9
   3e5a6:	punpckhwd %xmm0,%xmm5
   3e5aa:	por    %xmm7,%xmm5
   3e5ae:	paddd  %xmm1,%xmm8
   3e5b3:	movdqu %xmm8,(%rcx,%r8,4)
   3e5b9:	paddd  %xmm2,%xmm6
   3e5bd:	movdqu %xmm6,0x10(%rcx,%r8,4)
   3e5c4:	paddd  %xmm3,%xmm9
   3e5c9:	movdqu %xmm9,0x20(%rcx,%r8,4)
   3e5d0:	paddd  %xmm4,%xmm5
   3e5d4:	movdqu %xmm5,0x30(%rcx,%r8,4)
   3e5db:	lea    (%r8,%r13,1),%eax
   3e5df:	mov    %eax,0x418(%rsp,%rsi,4)
   3e5e6:	mov    %rdi,%rsi
   3e5e9:	test   %ebp,%ebp
   3e5eb:	je     3eb33 <mibz2::bwt::build+0x9d3>
   3e5f1:	tzcnt  %ebp,%r9d
   3e5f6:	lea    (%rsi,%r9,1),%rax
   3e5fa:	cmp    %rbx,%rax
   3e5fd:	jae    3eb97 <mibz2::bwt::build+0xa37>
   3e603:	mov    %ebp,%r8d
   3e606:	mov    %rsi,%rdi
   3e609:	lea    -0x1(%r8),%ebp
   3e60d:	and    %r8d,%ebp
   3e610:	mov    %ebp,%r8d
   3e613:	or     $0x10000,%r8d
   3e61a:	tzcnt  %r8d,%r13d
   3e61f:	movzbl (%r14,%rax,1),%esi
   3e624:	lea    0x0(,%rsi,4),%r10d
   3e62c:	mov    0x418(%rsp,%r10,1),%r8d
   3e634:	lea    0x10(%r8),%r12
   3e638:	mov    0x18(%rsp,%r10,1),%r10d
   3e63d:	cmp    %r10,%r12
   3e640:	jbe    3e55d <mibz2::bwt::build+0x3fd>
   3e646:	cmp    %r13d,%r9d
   3e649:	mov    %rdi,%rsi
   3e64c:	jae    3e5e9 <mibz2::bwt::build+0x489>
   3e64e:	lea    (%rsi,%r9,1),%r8d
   3e652:	shl    $0x8,%r8d
   3e656:	cs nopw 0x0(%rax,%rax,1)
   3e660:	cmp    %r9,%r11
   3e663:	je     3eb76 <mibz2::bwt::build+0xa16>
   3e669:	movzbl (%rdx,%r9,1),%r12d
   3e66e:	lea    0x0(,%r12,4),%r10d
   3e676:	mov    0x418(%rsp,%r10,1),%eax
   3e67e:	cmp    0x18(%rsp,%r10,1),%eax
   3e683:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e689:	cmp    %rax,%rbx
   3e68c:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e692:	movzbl (%r14,%rax,1),%r10d
   3e697:	add    %r8d,%r10d
   3e69a:	mov    %r10d,(%rcx,%rax,4)
   3e69e:	inc    %eax
   3e6a0:	mov    %eax,0x418(%rsp,%r12,4)
   3e6a8:	inc    %r9
   3e6ab:	add    $0x100,%r8d
   3e6b2:	cmp    %r9,%r13
   3e6b5:	jne    3e660 <mibz2::bwt::build+0x500>
   3e6b7:	jmp    3e5e9 <mibz2::bwt::build+0x489>
   3e6bc:	movzbl (%r14,%rsi,1),%r8d
   3e6c1:	lea    0x0(,%r8,4),%r9d
   3e6c9:	mov    0x418(%rsp,%r9,1),%eax
   3e6d1:	cmp    0x18(%rsp,%r9,1),%eax
   3e6d6:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e6dc:	cmp    %rax,%rbx
   3e6df:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e6e5:	movzbl (%r14,%rax,1),%r9d
   3e6ea:	mov    %esi,%ebp
   3e6ec:	shl    $0x8,%ebp
   3e6ef:	or     %r9d,%ebp
   3e6f2:	mov    %ebp,(%rcx,%rax,4)
   3e6f5:	inc    %eax
   3e6f7:	mov    %eax,0x418(%rsp,%r8,4)
   3e6ff:	movzbl 0x1(%r14,%rsi,1),%r8d
   3e705:	lea    0x0(,%r8,4),%r9d
   3e70d:	mov    0x418(%rsp,%r9,1),%eax
   3e715:	cmp    0x18(%rsp,%r9,1),%eax
   3e71a:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e720:	cmp    %rax,%rbx
   3e723:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e729:	lea    0x1(%rsi),%r9
   3e72d:	movzbl (%r14,%rax,1),%ebp
   3e732:	shl    $0x8,%r9d
   3e736:	or     %ebp,%r9d
   3e739:	mov    %r9d,(%rcx,%rax,4)
   3e73d:	inc    %eax
   3e73f:	mov    %eax,0x418(%rsp,%r8,4)
   3e747:	movzbl 0x2(%r14,%rsi,1),%r8d
   3e74d:	lea    0x0(,%r8,4),%r9d
   3e755:	mov    0x418(%rsp,%r9,1),%eax
   3e75d:	cmp    0x18(%rsp,%r9,1),%eax
   3e762:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e768:	cmp    %rax,%rbx
   3e76b:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e771:	lea    0x2(%rsi),%r9
   3e775:	movzbl (%r14,%rax,1),%ebp
   3e77a:	shl    $0x8,%r9d
   3e77e:	or     %ebp,%r9d
   3e781:	mov    %r9d,(%rcx,%rax,4)
   3e785:	inc    %eax
   3e787:	mov    %eax,0x418(%rsp,%r8,4)
   3e78f:	movzbl 0x3(%r14,%rsi,1),%r8d
   3e795:	lea    0x0(,%r8,4),%r9d
   3e79d:	mov    0x418(%rsp,%r9,1),%eax
   3e7a5:	cmp    0x18(%rsp,%r9,1),%eax
   3e7aa:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e7b0:	cmp    %rax,%rbx
   3e7b3:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e7b9:	lea    0x3(%rsi),%r9
   3e7bd:	movzbl (%r14,%rax,1),%ebp
   3e7c2:	shl    $0x8,%r9d
   3e7c6:	or     %ebp,%r9d
   3e7c9:	mov    %r9d,(%rcx,%rax,4)
   3e7cd:	inc    %eax
   3e7cf:	mov    %eax,0x418(%rsp,%r8,4)
   3e7d7:	movzbl 0x4(%r14,%rsi,1),%r8d
   3e7dd:	lea    0x0(,%r8,4),%r9d
   3e7e5:	mov    0x418(%rsp,%r9,1),%eax
   3e7ed:	cmp    0x18(%rsp,%r9,1),%eax
   3e7f2:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e7f8:	cmp    %rax,%rbx
   3e7fb:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e801:	lea    0x4(%rsi),%r9
   3e805:	movzbl (%r14,%rax,1),%ebp
   3e80a:	shl    $0x8,%r9d
   3e80e:	or     %ebp,%r9d
   3e811:	mov    %r9d,(%rcx,%rax,4)
   3e815:	inc    %eax
   3e817:	mov    %eax,0x418(%rsp,%r8,4)
   3e81f:	movzbl 0x5(%r14,%rsi,1),%r8d
   3e825:	lea    0x0(,%r8,4),%r9d
   3e82d:	mov    0x418(%rsp,%r9,1),%eax
   3e835:	cmp    0x18(%rsp,%r9,1),%eax
   3e83a:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e840:	cmp    %rax,%rbx
   3e843:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e849:	lea    0x5(%rsi),%r9
   3e84d:	movzbl (%r14,%rax,1),%ebp
   3e852:	shl    $0x8,%r9d
   3e856:	or     %ebp,%r9d
   3e859:	mov    %r9d,(%rcx,%rax,4)
   3e85d:	inc    %eax
   3e85f:	mov    %eax,0x418(%rsp,%r8,4)
   3e867:	movzbl 0x6(%r14,%rsi,1),%r8d
   3e86d:	lea    0x0(,%r8,4),%r9d
   3e875:	mov    0x418(%rsp,%r9,1),%eax
   3e87d:	cmp    0x18(%rsp,%r9,1),%eax
   3e882:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e888:	cmp    %rax,%rbx
   3e88b:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e891:	lea    0x6(%rsi),%r9
   3e895:	movzbl (%r14,%rax,1),%ebp
   3e89a:	shl    $0x8,%r9d
   3e89e:	or     %ebp,%r9d
   3e8a1:	mov    %r9d,(%rcx,%rax,4)
   3e8a5:	inc    %eax
   3e8a7:	mov    %eax,0x418(%rsp,%r8,4)
   3e8af:	movzbl 0x7(%r14,%rsi,1),%r8d
   3e8b5:	lea    0x0(,%r8,4),%r9d
   3e8bd:	mov    0x418(%rsp,%r9,1),%eax
   3e8c5:	cmp    0x18(%rsp,%r9,1),%eax
   3e8ca:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e8d0:	cmp    %rax,%rbx
   3e8d3:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e8d9:	lea    0x7(%rsi),%r9
   3e8dd:	movzbl (%r14,%rax,1),%ebp
   3e8e2:	shl    $0x8,%r9d
   3e8e6:	or     %ebp,%r9d
   3e8e9:	mov    %r9d,(%rcx,%rax,4)
   3e8ed:	inc    %eax
   3e8ef:	mov    %eax,0x418(%rsp,%r8,4)
   3e8f7:	movzbl 0x8(%r14,%rsi,1),%r8d
   3e8fd:	lea    0x0(,%r8,4),%r9d
   3e905:	mov    0x418(%rsp,%r9,1),%eax
   3e90d:	cmp    0x18(%rsp,%r9,1),%eax
   3e912:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e918:	cmp    %rax,%rbx
   3e91b:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e921:	lea    0x8(%rsi),%r9
   3e925:	movzbl (%r14,%rax,1),%ebp
   3e92a:	shl    $0x8,%r9d
   3e92e:	or     %ebp,%r9d
   3e931:	mov    %r9d,(%rcx,%rax,4)
   3e935:	inc    %eax
   3e937:	mov    %eax,0x418(%rsp,%r8,4)
   3e93f:	movzbl 0x9(%r14,%rsi,1),%r8d
   3e945:	lea    0x0(,%r8,4),%r9d
   3e94d:	mov    0x418(%rsp,%r9,1),%eax
   3e955:	cmp    0x18(%rsp,%r9,1),%eax
   3e95a:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e960:	cmp    %rax,%rbx
   3e963:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e969:	lea    0x9(%rsi),%r9
   3e96d:	movzbl (%r14,%rax,1),%ebp
   3e972:	shl    $0x8,%r9d
   3e976:	or     %ebp,%r9d
   3e979:	mov    %r9d,(%rcx,%rax,4)
   3e97d:	inc    %eax
   3e97f:	mov    %eax,0x418(%rsp,%r8,4)
   3e987:	movzbl 0xa(%r14,%rsi,1),%r8d
   3e98d:	lea    0x0(,%r8,4),%r9d
   3e995:	mov    0x418(%rsp,%r9,1),%eax
   3e99d:	cmp    0x18(%rsp,%r9,1),%eax
   3e9a2:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e9a8:	cmp    %rax,%rbx
   3e9ab:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e9b1:	lea    0xa(%rsi),%r9
   3e9b5:	movzbl (%r14,%rax,1),%ebp
   3e9ba:	shl    $0x8,%r9d
   3e9be:	or     %ebp,%r9d
   3e9c1:	mov    %r9d,(%rcx,%rax,4)
   3e9c5:	inc    %eax
   3e9c7:	mov    %eax,0x418(%rsp,%r8,4)
   3e9cf:	movzbl 0xb(%r14,%rsi,1),%r8d
   3e9d5:	lea    0x0(,%r8,4),%r9d
   3e9dd:	mov    0x418(%rsp,%r9,1),%eax
   3e9e5:	cmp    0x18(%rsp,%r9,1),%eax
   3e9ea:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3e9f0:	cmp    %rax,%rbx
   3e9f3:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3e9f9:	lea    0xb(%rsi),%r9
   3e9fd:	movzbl (%r14,%rax,1),%ebp
   3ea02:	shl    $0x8,%r9d
   3ea06:	or     %ebp,%r9d
   3ea09:	mov    %r9d,(%rcx,%rax,4)
   3ea0d:	inc    %eax
   3ea0f:	mov    %eax,0x418(%rsp,%r8,4)
   3ea17:	movzbl 0xc(%r14,%rsi,1),%r8d
   3ea1d:	lea    0x0(,%r8,4),%r9d
   3ea25:	mov    0x418(%rsp,%r9,1),%eax
   3ea2d:	cmp    0x18(%rsp,%r9,1),%eax
   3ea32:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3ea38:	cmp    %rax,%rbx
   3ea3b:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3ea41:	lea    0xc(%rsi),%r9
   3ea45:	movzbl (%r14,%rax,1),%ebp
   3ea4a:	shl    $0x8,%r9d
   3ea4e:	or     %ebp,%r9d
   3ea51:	mov    %r9d,(%rcx,%rax,4)
   3ea55:	inc    %eax
   3ea57:	mov    %eax,0x418(%rsp,%r8,4)
   3ea5f:	movzbl 0xd(%r14,%rsi,1),%r8d
   3ea65:	lea    0x0(,%r8,4),%r9d
   3ea6d:	mov    0x418(%rsp,%r9,1),%eax
   3ea75:	cmp    0x18(%rsp,%r9,1),%eax
   3ea7a:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3ea80:	cmp    %rax,%rbx
   3ea83:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3ea89:	lea    0xd(%rsi),%r9
   3ea8d:	movzbl (%r14,%rax,1),%ebp
   3ea92:	shl    $0x8,%r9d
   3ea96:	or     %ebp,%r9d
   3ea99:	mov    %r9d,(%rcx,%rax,4)
   3ea9d:	inc    %eax
   3ea9f:	mov    %eax,0x418(%rsp,%r8,4)
   3eaa7:	movzbl 0xe(%r14,%rsi,1),%r8d
   3eaad:	lea    0x0(,%r8,4),%r9d
   3eab5:	mov    0x418(%rsp,%r9,1),%eax
   3eabd:	cmp    0x18(%rsp,%r9,1),%eax
   3eac2:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3eac8:	cmp    %rax,%rbx
   3eacb:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3ead1:	lea    0xe(%rsi),%r9
   3ead5:	movzbl (%r14,%rax,1),%ebp
   3eada:	shl    $0x8,%r9d
   3eade:	or     %ebp,%r9d
   3eae1:	mov    %r9d,(%rcx,%rax,4)
   3eae5:	inc    %eax
   3eae7:	mov    %eax,0x418(%rsp,%r8,4)
   3eaef:	movzbl 0xf(%r14,%rsi,1),%r8d
   3eaf5:	lea    0x0(,%r8,4),%r9d
   3eafd:	mov    0x418(%rsp,%r9,1),%eax
   3eb05:	cmp    0x18(%rsp,%r9,1),%eax
   3eb0a:	jae    3e304 <mibz2::bwt::build+0x1a4>
   3eb10:	cmp    %rax,%rbx
   3eb13:	jbe    3eb64 <mibz2::bwt::build+0xa04>
   3eb15:	lea    0xf(%rsi),%r9
   3eb19:	movzbl (%r14,%rax,1),%ebp
   3eb1e:	shl    $0x8,%r9d
   3eb22:	or     %ebp,%r9d
   3eb25:	mov    %r9d,(%rcx,%rax,4)
   3eb29:	inc    %eax
   3eb2b:	mov    %eax,0x418(%rsp,%r8,4)
   3eb33:	mov    0x10(%rsp),%r9
   3eb38:	add    $0x10,%r9
   3eb3c:	add    $0x10,%rsi
   3eb40:	add    $0x10,%rdx
   3eb44:	add    $0xfffffffffffffff0,%r11
   3eb48:	cmp    %rbx,%r9
   3eb4b:	jbe    3e43f <mibz2::bwt::build+0x2df>
   3eb51:	mov    0x8(%rsp),%rdx
   3eb56:	cmp    %rbx,%rdx
   3eb59:	jb     3e28d <mibz2::bwt::build+0x12d>
   3eb5f:	jmp    3e2e8 <mibz2::bwt::build+0x188>
   3eb64:	lea    0x5d19d(%rip),%rdx        # 9bd08 <__frame_dummy_init_array_entry+0xed0>
   3eb6b:	mov    %rax,%rdi
   3eb6e:	mov    %rbx,%rsi
   3eb71:	call   2f8d1 <core::panicking::panic_bounds_check>
   3eb76:	lea    0x5d173(%rip),%rdx        # 9bcf0 <__frame_dummy_init_array_entry+0xeb8>
   3eb7d:	mov    %rbx,%rdi
   3eb80:	mov    %rbx,%rsi
   3eb83:	call   2f8d1 <core::panicking::panic_bounds_check>
   3eb88:	lea    0x5d179(%rip),%rdx        # 9bd08 <__frame_dummy_init_array_entry+0xed0>
   3eb8f:	mov    %rbx,%rsi
   3eb92:	call   2f8d1 <core::panicking::panic_bounds_check>
   3eb97:	lea    0x5d152(%rip),%rdx        # 9bcf0 <__frame_dummy_init_array_entry+0xeb8>
   3eb9e:	mov    %rax,%rdi
   3eba1:	mov    %rbx,%rsi
   3eba4:	call   2f8d1 <core::panicking::panic_bounds_check>
   3eba9:	lea    0x5d140(%rip),%rdx        # 9bcf0 <__frame_dummy_init_array_entry+0xeb8>
   3ebb0:	mov    %rsi,%rdi
   3ebb3:	mov    %rbx,%rsi
   3ebb6:	call   2f8d1 <core::panicking::panic_bounds_check>
   3ebbb:	add    $0x818,%rsp
   3ebc2:	pop    %rbx
   3ebc3:	pop    %r12
   3ebc5:	pop    %r13
   3ebc7:	pop    %r14
   3ebc9:	pop    %r15
   3ebcb:	pop    %rbp
   3ebcc:	ret
   3ebcd:	movzbl (%r14,%rax,1),%edx
   3ebd2:	mov    %edx,0x0(%r13,%r12,4)
   3ebd7:	inc    %r12
   3ebda:	inc    %rax
   3ebdd:	cmp    %rax,%rbx
   3ebe0:	jne    3ebcd <mibz2::bwt::build+0xa6d>
   3ebe2:	mov    %r12,0x10(%rcx)
   3ebe6:	lea    0x18(%rsp),%rdi
   3ebeb:	xor    %ebp,%ebp
   3ebed:	mov    $0x400,%edx
   3ebf2:	xor    %esi,%esi
   3ebf4:	call   *0x5f0fe(%rip)        # 9dcf8 <memset@GLIBC_2.2.5>
   3ebfa:	xor    %eax,%eax
   3ebfc:	nopl   0x0(%rax)
   3ec00:	mov    %eax,0x18(%rsp,%rbp,4)
   3ec04:	add    (%r15,%rbp,4),%eax
   3ec08:	mov    %eax,0x1c(%rsp,%rbp,4)
   3ec0c:	add    0x4(%r15,%rbp,4),%eax
   3ec11:	mov    %eax,0x20(%rsp,%rbp,4)
   3ec15:	add    0x8(%r15,%rbp,4),%eax
   3ec1a:	mov    %eax,0x24(%rsp,%rbp,4)
   3ec1e:	add    0xc(%r15,%rbp,4),%eax
   3ec23:	add    $0x4,%rbp
   3ec27:	cmp    $0x100,%rbp
   3ec2e:	jne    3ec00 <mibz2::bwt::build+0xaa0>
   3ec30:	xor    %ecx,%ecx
   3ec32:	cmp    $0x10,%rbx
   3ec36:	jae    3ec89 <mibz2::bwt::build+0xb29>
   3ec38:	xor    %eax,%eax
   3ec3a:	cmp    %rbx,%rax
   3ec3d:	jae    3ebbb <mibz2::bwt::build+0xa5b>
   3ec43:	mov    %eax,%ecx
   3ec45:	shl    $0x8,%ecx
   3ec48:	nopl   0x0(%rax,%rax,1)
   3ec50:	movzbl (%r14,%rax,1),%edx
   3ec55:	mov    0x18(%rsp,%rdx,4),%edi
   3ec59:	cmp    %rdi,%r12
   3ec5c:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ec62:	inc    %rax
   3ec65:	movzbl 0x0(%r13,%rdi,4),%esi
   3ec6b:	add    %ecx,%esi
   3ec6d:	mov    %esi,0x0(%r13,%rdi,4)
   3ec72:	lea    0x1(%rdi),%esi
   3ec75:	mov    %esi,0x18(%rsp,%rdx,4)
   3ec79:	add    $0x100,%ecx
   3ec7f:	cmp    %rax,%rbx
   3ec82:	jne    3ec50 <mibz2::bwt::build+0xaf0>
   3ec84:	jmp    3ebbb <mibz2::bwt::build+0xa5b>
   3ec89:	movdqa -0x366d1(%rip),%xmm0        # 85c0 <GCC_except_table759+0x3fc>
   3ec91:	movaps -0x36758(%rip),%xmm1        # 8540 <GCC_except_table759+0x37c>
   3ec98:	movdqa -0x36a50(%rip),%xmm2        # 8250 <GCC_except_table759+0x8c>
   3eca0:	movdqa -0x36a78(%rip),%xmm3        # 8230 <GCC_except_table759+0x6c>
   3eca8:	movdqa -0x367f0(%rip),%xmm4        # 84c0 <GCC_except_table759+0x2fc>
   3ecb0:	xor    %esi,%esi
   3ecb2:	jmp    3ecf4 <mibz2::bwt::build+0xb94>
   3ecb4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ecc0:	movzbl 0x0(%r13,%rdi,4),%edx
   3ecc6:	add    %ecx,%edx
   3ecc8:	add    $0xf00,%edx
   3ecce:	mov    %edx,0x0(%r13,%rdi,4)
   3ecd3:	lea    0x1(%rdi),%edx
   3ecd6:	mov    %edx,0x18(%rsp,%rax,4)
   3ecda:	lea    0x10(%rsi),%rax
   3ecde:	add    $0x20,%rsi
   3ece2:	add    $0x1000,%ecx
   3ece8:	cmp    %rbx,%rsi
   3eceb:	mov    %rax,%rsi
   3ecee:	ja     3ec3a <mibz2::bwt::build+0xada>
   3ecf4:	lea    0xf(%rsi),%rax
   3ecf8:	cmp    %rbx,%rax
   3ecfb:	jae    3f0f2 <mibz2::bwt::build+0xf92>
   3ed01:	mov    (%r14,%rsi,1),%rdx
   3ed05:	mov    0x8(%r14,%rsi,1),%rax
   3ed0a:	mov    %rdx,%rdi
   3ed0d:	xor    %rax,%rdi
   3ed10:	mov    %rdx,%r8
   3ed13:	rol    $0x8,%r8
   3ed17:	xor    %rdx,%r8
   3ed1a:	or     %rdi,%r8
   3ed1d:	jne    3edd0 <mibz2::bwt::build+0xc70>
   3ed23:	movzbl %dl,%edx
   3ed26:	mov    %edx,%eax
   3ed28:	mov    0x18(%rsp,%rax,4),%edi
   3ed2c:	lea    0x10(%rdi),%rax
   3ed30:	cmp    %r12,%rax
   3ed33:	ja     3f116 <mibz2::bwt::build+0xfb6>
   3ed39:	movdqu 0x0(%r13,%rdi,4),%xmm6
   3ed40:	movdqu 0x10(%r13,%rdi,4),%xmm7
   3ed47:	movdqu 0x20(%r13,%rdi,4),%xmm8
   3ed4e:	movdqu 0x30(%r13,%rdi,4),%xmm5
   3ed55:	pand   %xmm0,%xmm6
   3ed59:	movd   %ecx,%xmm9
   3ed5e:	movaps %xmm1,%xmm10
   3ed62:	movss  %xmm9,%xmm10
   3ed67:	movd   %ecx,%xmm9
   3ed6c:	pshufd $0x1,%xmm9,%xmm11
   3ed72:	por    %xmm10,%xmm11
   3ed77:	por    %xmm6,%xmm11
   3ed7c:	movdqu %xmm11,0x0(%r13,%rdi,4)
   3ed83:	pand   %xmm0,%xmm7
   3ed87:	pshufd $0x0,%xmm9,%xmm6
   3ed8d:	por    %xmm6,%xmm7
   3ed91:	por    %xmm2,%xmm7
   3ed95:	movdqu %xmm7,0x10(%r13,%rdi,4)
   3ed9c:	pand   %xmm0,%xmm8
   3eda1:	por    %xmm6,%xmm8
   3eda6:	por    %xmm3,%xmm8
   3edab:	movdqu %xmm8,0x20(%r13,%rdi,4)
   3edb2:	pand   %xmm0,%xmm5
   3edb6:	por    %xmm6,%xmm5
   3edba:	por    %xmm4,%xmm5
   3edbe:	movdqu %xmm5,0x30(%r13,%rdi,4)
   3edc5:	addl   $0x10,0x18(%rsp,%rdx,4)
   3edca:	jmp    3ecda <mibz2::bwt::build+0xb7a>
   3edcf:	nop
   3edd0:	movzbl %dl,%r8d
   3edd4:	mov    %r8d,%edi
   3edd7:	mov    0x18(%rsp,%rdi,4),%edi
   3eddb:	cmp    %rdi,%r12
   3edde:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ede4:	mov    %rdx,%r9
   3ede7:	shr    $0x8,%r9
   3edeb:	movzbl 0x0(%r13,%rdi,4),%r10d
   3edf1:	add    %ecx,%r10d
   3edf4:	mov    %r10d,0x0(%r13,%rdi,4)
   3edf9:	inc    %edi
   3edfb:	mov    %edi,0x18(%rsp,%r8,4)
   3ee00:	movzbl %r9b,%r8d
   3ee04:	mov    %r8d,%edi
   3ee07:	mov    0x18(%rsp,%rdi,4),%edi
   3ee0b:	cmp    %rdi,%r12
   3ee0e:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ee14:	mov    %rdx,%r9
   3ee17:	shr    $0x10,%r9
   3ee1b:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee21:	add    %ecx,%r10d
   3ee24:	add    $0x100,%r10d
   3ee2b:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee30:	inc    %edi
   3ee32:	mov    %edi,0x18(%rsp,%r8,4)
   3ee37:	movzbl %r9b,%r8d
   3ee3b:	mov    %r8d,%edi
   3ee3e:	mov    0x18(%rsp,%rdi,4),%edi
   3ee42:	cmp    %rdi,%r12
   3ee45:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ee4b:	mov    %rdx,%r9
   3ee4e:	shr    $0x18,%r9
   3ee52:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee58:	add    %ecx,%r10d
   3ee5b:	add    $0x200,%r10d
   3ee62:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee67:	inc    %edi
   3ee69:	mov    %edi,0x18(%rsp,%r8,4)
   3ee6e:	movzbl %r9b,%r8d
   3ee72:	mov    %r8d,%edi
   3ee75:	mov    0x18(%rsp,%rdi,4),%edi
   3ee79:	cmp    %rdi,%r12
   3ee7c:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ee82:	mov    %rdx,%r9
   3ee85:	shr    $0x20,%r9
   3ee89:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee8f:	add    %ecx,%r10d
   3ee92:	add    $0x300,%r10d
   3ee99:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee9e:	inc    %edi
   3eea0:	mov    %edi,0x18(%rsp,%r8,4)
   3eea5:	movzbl %r9b,%r8d
   3eea9:	mov    %r8d,%edi
   3eeac:	mov    0x18(%rsp,%rdi,4),%edi
   3eeb0:	cmp    %rdi,%r12
   3eeb3:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3eeb9:	mov    %rdx,%r9
   3eebc:	shr    $0x28,%r9
   3eec0:	movzbl 0x0(%r13,%rdi,4),%r10d
   3eec6:	add    %ecx,%r10d
   3eec9:	add    $0x400,%r10d
   3eed0:	mov    %r10d,0x0(%r13,%rdi,4)
   3eed5:	inc    %edi
   3eed7:	mov    %edi,0x18(%rsp,%r8,4)
   3eedc:	movzbl %r9b,%r8d
   3eee0:	mov    %r8d,%edi
   3eee3:	mov    0x18(%rsp,%rdi,4),%edi
   3eee7:	cmp    %rdi,%r12
   3eeea:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3eef0:	mov    %rdx,%r9
   3eef3:	shr    $0x30,%r9
   3eef7:	movzbl 0x0(%r13,%rdi,4),%r10d
   3eefd:	add    %ecx,%r10d
   3ef00:	add    $0x500,%r10d
   3ef07:	mov    %r10d,0x0(%r13,%rdi,4)
   3ef0c:	inc    %edi
   3ef0e:	mov    %edi,0x18(%rsp,%r8,4)
   3ef13:	movzbl %r9b,%r8d
   3ef17:	mov    %r8d,%edi
   3ef1a:	mov    0x18(%rsp,%rdi,4),%edi
   3ef1e:	cmp    %rdi,%r12
   3ef21:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ef27:	shr    $0x38,%rdx
   3ef2b:	movzbl 0x0(%r13,%rdi,4),%r9d
   3ef31:	add    %ecx,%r9d
   3ef34:	add    $0x600,%r9d
   3ef3b:	mov    %r9d,0x0(%r13,%rdi,4)
   3ef40:	inc    %edi
   3ef42:	mov    %edi,0x18(%rsp,%r8,4)
   3ef47:	mov    0x18(%rsp,%rdx,4),%edi
   3ef4b:	cmp    %rdi,%r12
   3ef4e:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ef54:	movzbl 0x0(%r13,%rdi,4),%r8d
   3ef5a:	add    %ecx,%r8d
   3ef5d:	add    $0x700,%r8d
   3ef64:	mov    %r8d,0x0(%r13,%rdi,4)
   3ef69:	inc    %edi
   3ef6b:	mov    %edi,0x18(%rsp,%rdx,4)
   3ef6f:	movzbl %al,%edx
   3ef72:	mov    %edx,%edi
   3ef74:	mov    0x18(%rsp,%rdi,4),%edi
   3ef78:	cmp    %rdi,%r12
   3ef7b:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3ef81:	mov    %rax,%r8
   3ef84:	shr    $0x8,%r8
   3ef88:	movzbl 0x0(%r13,%rdi,4),%r9d
   3ef8e:	add    %ecx,%r9d
   3ef91:	add    $0x800,%r9d
   3ef98:	mov    %r9d,0x0(%r13,%rdi,4)
   3ef9d:	inc    %edi
   3ef9f:	mov    %edi,0x18(%rsp,%rdx,4)
   3efa3:	movzbl %r8b,%edx
   3efa7:	mov    %edx,%edi
   3efa9:	mov    0x18(%rsp,%rdi,4),%edi
   3efad:	cmp    %rdi,%r12
   3efb0:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3efb6:	mov    %rax,%r8
   3efb9:	shr    $0x10,%r8
   3efbd:	movzbl 0x0(%r13,%rdi,4),%r9d
   3efc3:	add    %ecx,%r9d
   3efc6:	add    $0x900,%r9d
   3efcd:	mov    %r9d,0x0(%r13,%rdi,4)
   3efd2:	inc    %edi
   3efd4:	mov    %edi,0x18(%rsp,%rdx,4)
   3efd8:	movzbl %r8b,%edx
   3efdc:	mov    %edx,%edi
   3efde:	mov    0x18(%rsp,%rdi,4),%edi
   3efe2:	cmp    %rdi,%r12
   3efe5:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3efeb:	mov    %rax,%r8
   3efee:	shr    $0x18,%r8
   3eff2:	movzbl 0x0(%r13,%rdi,4),%r9d
   3eff8:	add    %ecx,%r9d
   3effb:	add    $0xa00,%r9d
   3f002:	mov    %r9d,0x0(%r13,%rdi,4)
   3f007:	inc    %edi
   3f009:	mov    %edi,0x18(%rsp,%rdx,4)
   3f00d:	movzbl %r8b,%edx
   3f011:	mov    %edx,%edi
   3f013:	mov    0x18(%rsp,%rdi,4),%edi
   3f017:	cmp    %rdi,%r12
   3f01a:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3f020:	mov    %rax,%r8
   3f023:	shr    $0x20,%r8
   3f027:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f02d:	add    %ecx,%r9d
   3f030:	add    $0xb00,%r9d
   3f037:	mov    %r9d,0x0(%r13,%rdi,4)
   3f03c:	inc    %edi
   3f03e:	mov    %edi,0x18(%rsp,%rdx,4)
   3f042:	movzbl %r8b,%edx
   3f046:	mov    %edx,%edi
   3f048:	mov    0x18(%rsp,%rdi,4),%edi
   3f04c:	cmp    %rdi,%r12
   3f04f:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3f055:	mov    %rax,%r8
   3f058:	shr    $0x28,%r8
   3f05c:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f062:	add    %ecx,%r9d
   3f065:	add    $0xc00,%r9d
   3f06c:	mov    %r9d,0x0(%r13,%rdi,4)
   3f071:	inc    %edi
   3f073:	mov    %edi,0x18(%rsp,%rdx,4)
   3f077:	movzbl %r8b,%edx
   3f07b:	mov    %edx,%edi
   3f07d:	mov    0x18(%rsp,%rdi,4),%edi
   3f081:	cmp    %rdi,%r12
   3f084:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3f086:	mov    %rax,%r8
   3f089:	shr    $0x30,%r8
   3f08d:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f093:	add    %ecx,%r9d
   3f096:	add    $0xd00,%r9d
   3f09d:	mov    %r9d,0x0(%r13,%rdi,4)
   3f0a2:	inc    %edi
   3f0a4:	mov    %edi,0x18(%rsp,%rdx,4)
   3f0a8:	movzbl %r8b,%edx
   3f0ac:	mov    %edx,%edi
   3f0ae:	mov    0x18(%rsp,%rdi,4),%edi
   3f0b2:	cmp    %rdi,%r12
   3f0b5:	jbe    3f0e3 <mibz2::bwt::build+0xf83>
   3f0b7:	shr    $0x38,%rax
   3f0bb:	movzbl 0x0(%r13,%rdi,4),%r8d
   3f0c1:	add    %ecx,%r8d
   3f0c4:	add    $0xe00,%r8d
   3f0cb:	mov    %r8d,0x0(%r13,%rdi,4)
   3f0d0:	inc    %edi
   3f0d2:	mov    %edi,0x18(%rsp,%rdx,4)
   3f0d6:	mov    0x18(%rsp,%rax,4),%edi
   3f0da:	cmp    %rdi,%r12
   3f0dd:	ja     3ecc0 <mibz2::bwt::build+0xb60>
   3f0e3:	lea    0x5cfde(%rip),%rdx        # 9c0c8 <__frame_dummy_init_array_entry+0x1290>
   3f0ea:	mov    %r12,%rsi
   3f0ed:	call   2f8d1 <core::panicking::panic_bounds_check>
   3f0f2:	movabs $0x7ffffffffffffff0,%rdi
   3f0fc:	and    %rbx,%rdi
   3f0ff:	or     $0x8,%rdi
   3f103:	add    $0x10,%rsi
   3f107:	lea    0x5cfea(%rip),%rcx        # 9c0f8 <__frame_dummy_init_array_entry+0x12c0>
   3f10e:	mov    %rbx,%rdx
   3f111:	call   2f7f0 <core::slice::index::slice_index_fail>
   3f116:	lea    0x5cf93(%rip),%rcx        # 9c0b0 <__frame_dummy_init_array_entry+0x1278>
   3f11d:	mov    %rax,%rsi
   3f120:	mov    %r12,%rdx
   3f123:	call   2f7f0 <core::slice::index::slice_index_fail>
Disassembly of section .init:
Disassembly of section .fini:
Disassembly of section .plt:
