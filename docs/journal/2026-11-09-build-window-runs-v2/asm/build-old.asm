tgt-old/release/c30cg:     file format elf64-x86-64
Disassembly of section .text:
000000000003e0e0 <mibz2::bwt::build>:
   3e0e0:	push   %rbp
   3e0e1:	push   %r15
   3e0e3:	push   %r14
   3e0e5:	push   %r13
   3e0e7:	push   %r12
   3e0e9:	push   %rbx
   3e0ea:	sub    $0x838,%rsp
   3e0f1:	mov    %rdx,%r15
   3e0f4:	mov    %rsi,%rbx
   3e0f7:	mov    %rdi,%r14
   3e0fa:	movq   $0x0,0x10(%rcx)
   3e102:	cmp    $0xffffff,%rsi
   3e109:	ja     3e289 <mibz2::bwt::build+0x1a9>
   3e10f:	pxor   %xmm0,%xmm0
   3e113:	xor    %eax,%eax
   3e115:	pxor   %xmm2,%xmm2
   3e119:	pxor   %xmm1,%xmm1
   3e11d:	nopl   (%rax)
   3e120:	movq   (%r15,%rax,4),%xmm3
   3e126:	movq   0x8(%r15,%rax,4),%xmm4
   3e12d:	punpckldq %xmm0,%xmm3
   3e131:	movdqa %xmm2,%xmm5
   3e135:	paddq  %xmm3,%xmm5
   3e139:	punpckldq %xmm0,%xmm4
   3e13d:	movdqa %xmm1,%xmm3
   3e141:	paddq  %xmm4,%xmm3
   3e145:	movq   0x10(%r15,%rax,4),%xmm2
   3e14c:	movq   0x18(%r15,%rax,4),%xmm1
   3e153:	punpckldq %xmm0,%xmm2
   3e157:	paddq  %xmm5,%xmm2
   3e15b:	punpckldq %xmm0,%xmm1
   3e15f:	paddq  %xmm3,%xmm1
   3e163:	add    $0x8,%rax
   3e167:	cmp    $0x100,%rax
   3e16d:	jne    3e120 <mibz2::bwt::build+0x40>
   3e16f:	paddq  %xmm2,%xmm1
   3e173:	pshufd $0xee,%xmm1,%xmm0
   3e178:	paddq  %xmm1,%xmm0
   3e17c:	movq   %xmm0,%rax
   3e181:	cmp    %rbx,%rax
   3e184:	jne    3e289 <mibz2::bwt::build+0x1a9>
   3e18a:	mov    %rcx,0x20(%rsp)
   3e18f:	lea    0x38(%rsp),%rdi
   3e194:	xor    %ebp,%ebp
   3e196:	mov    0x5fb9b(%rip),%r12        # 9dd38 <memset@GLIBC_2.2.5>
   3e19d:	mov    $0x400,%edx
   3e1a2:	xor    %esi,%esi
   3e1a4:	call   *%r12
   3e1a7:	lea    0x438(%rsp),%rdi
   3e1af:	mov    $0x400,%edx
   3e1b4:	xor    %esi,%esi
   3e1b6:	call   *%r12
   3e1b9:	mov    $0x1,%eax
   3e1be:	xchg   %ax,%ax
   3e1c0:	mov    %ebp,0x434(%rsp,%rax,4)
   3e1c7:	add    -0x4(%r15,%rax,4),%ebp
   3e1cc:	mov    %ebp,0x34(%rsp,%rax,4)
   3e1d0:	mov    %ebp,0x438(%rsp,%rax,4)
   3e1d7:	add    (%r15,%rax,4),%ebp
   3e1db:	mov    %ebp,0x38(%rsp,%rax,4)
   3e1df:	add    $0x2,%rax
   3e1e3:	cmp    $0x101,%rax
   3e1e9:	jne    3e1c0 <mibz2::bwt::build+0xe0>
   3e1eb:	mov    0x20(%rsp),%rax
   3e1f0:	cmp    (%rax),%rbx
   3e1f3:	ja     3e356 <mibz2::bwt::build+0x276>
   3e1f9:	mov    0x8(%rax),%rcx
   3e1fd:	cmp    $0x10,%rbx
   3e201:	jae    3e383 <mibz2::bwt::build+0x2a3>
   3e207:	xor    %esi,%esi
   3e209:	cmp    %rbx,%rsi
   3e20c:	jae    3e268 <mibz2::bwt::build+0x188>
   3e20e:	mov    %esi,%eax
   3e210:	shl    $0x8,%eax
   3e213:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e220:	movzbl (%r14,%rsi,1),%r8d
   3e225:	lea    0x0(,%r8,4),%r9d
   3e22d:	mov    0x438(%rsp,%r9,1),%edi
   3e235:	cmp    0x38(%rsp,%r9,1),%edi
   3e23a:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e23c:	cmp    %rdi,%rbx
   3e23f:	jbe    3eb93 <mibz2::bwt::build+0xab3>
   3e245:	movzbl (%r14,%rdi,1),%r9d
   3e24a:	add    %eax,%r9d
   3e24d:	mov    %r9d,(%rcx,%rdi,4)
   3e251:	inc    %edi
   3e253:	mov    %edi,0x438(%rsp,%r8,4)
   3e25b:	inc    %rsi
   3e25e:	add    $0x100,%eax
   3e263:	cmp    %rsi,%rbx
   3e266:	jne    3e220 <mibz2::bwt::build+0x140>
   3e268:	lea    0x438(%rsp),%rdi
   3e270:	lea    0x38(%rsp),%rsi
   3e275:	mov    $0x400,%edx
   3e27a:	call   *0x5fac0(%rip)        # 9dd40 <bcmp@GLIBC_2.2.5>
   3e280:	test   %eax,%eax
   3e282:	je     3e2b3 <mibz2::bwt::build+0x1d3>
   3e284:	mov    0x20(%rsp),%rcx
   3e289:	movq   $0x0,0x10(%rcx)
   3e291:	cmp    (%rcx),%rbx
   3e294:	ja     3e2c1 <mibz2::bwt::build+0x1e1>
   3e296:	mov    0x8(%rcx),%r13
   3e29a:	xor    %r12d,%r12d
   3e29d:	test   %rbx,%rbx
   3e2a0:	je     3ebed <mibz2::bwt::build+0xb0d>
   3e2a6:	cmp    $0x8,%rbx
   3e2aa:	jae    3e2ef <mibz2::bwt::build+0x20f>
   3e2ac:	xor    %eax,%eax
   3e2ae:	jmp    3ebd8 <mibz2::bwt::build+0xaf8>
   3e2b3:	mov    0x20(%rsp),%rax
   3e2b8:	mov    %rbx,0x10(%rax)
   3e2bc:	jmp    3ebc6 <mibz2::bwt::build+0xae6>
   3e2c1:	mov    %rcx,%r12
   3e2c4:	mov    $0x4,%ecx
   3e2c9:	mov    $0x4,%r8d
   3e2cf:	mov    %r12,%rdi
   3e2d2:	xor    %esi,%esi
   3e2d4:	mov    %rbx,%rdx
   3e2d7:	call   3a050 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e2dc:	mov    %r12,%rcx
   3e2df:	mov    0x8(%r12),%r13
   3e2e4:	mov    0x10(%r12),%r12
   3e2e9:	cmp    $0x8,%rbx
   3e2ed:	jb     3e2ac <mibz2::bwt::build+0x1cc>
   3e2ef:	movabs $0x7ffffffffffffff0,%rax
   3e2f9:	or     $0x8,%rax
   3e2fd:	and    %rbx,%rax
   3e300:	lea    0x10(,%r12,4),%rdx
   3e308:	add    %r13,%rdx
   3e30b:	add    %rax,%r12
   3e30e:	xor    %esi,%esi
   3e310:	pxor   %xmm0,%xmm0
   3e314:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e320:	movd   (%r14,%rsi,1),%xmm1
   3e326:	movd   0x4(%r14,%rsi,1),%xmm2
   3e32d:	punpcklbw %xmm0,%xmm1
   3e331:	punpcklwd %xmm0,%xmm1
   3e335:	punpcklbw %xmm0,%xmm2
   3e339:	punpcklwd %xmm0,%xmm2
   3e33d:	movdqu %xmm1,-0x10(%rdx,%rsi,4)
   3e343:	movdqu %xmm2,(%rdx,%rsi,4)
   3e348:	add    $0x8,%rsi
   3e34c:	cmp    %rsi,%rax
   3e34f:	jne    3e320 <mibz2::bwt::build+0x240>
   3e351:	jmp    3ebe8 <mibz2::bwt::build+0xb08>
   3e356:	mov    $0x4,%ecx
   3e35b:	mov    $0x4,%r8d
   3e361:	mov    0x20(%rsp),%rdi
   3e366:	xor    %esi,%esi
   3e368:	mov    %rbx,%rdx
   3e36b:	call   3a050 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e370:	mov    0x20(%rsp),%rax
   3e375:	mov    0x8(%rax),%rcx
   3e379:	cmp    $0x10,%rbx
   3e37d:	jb     3e207 <mibz2::bwt::build+0x127>
   3e383:	mov    %ebx,%esi
   3e385:	and    $0xfffff0,%esi
   3e38b:	mov    $0x10,%edi
   3e390:	xor    %r9d,%r9d
   3e393:	pxor   %xmm0,%xmm0
   3e397:	movdqa -0x35e1f(%rip),%xmm1        # 8580 <GCC_except_table759+0x38c>
   3e39f:	movdqa -0x36117(%rip),%xmm2        # 8290 <GCC_except_table759+0x9c>
   3e3a7:	movdqa -0x3613f(%rip),%xmm3        # 8270 <GCC_except_table759+0x7c>
   3e3af:	movdqa -0x35eb7(%rip),%xmm4        # 8500 <GCC_except_table759+0x30c>
   3e3b7:	mov    %rbx,%r10
   3e3ba:	mov    %r14,%rdx
   3e3bd:	movdqu (%r14,%r9,1),%xmm5
   3e3c3:	movdqa %xmm5,%xmm6
   3e3c7:	pslldq $0x1,%xmm6
   3e3cc:	pcmpeqb %xmm5,%xmm6
   3e3d0:	pmovmskb %xmm6,%r8d
   3e3d5:	movdqa %xmm5,0x10(%rsp)
   3e3db:	movzbl 0x10(%rsp),%eax
   3e3e0:	cmp    $0xfffe,%r8w
   3e3e5:	mov    %rdi,0x30(%rsp)
   3e3ea:	jb     3e4a7 <mibz2::bwt::build+0x3c7>
   3e3f0:	cmp    %rbx,%r9
   3e3f3:	jae    3ebb4 <mibz2::bwt::build+0xad4>
   3e3f9:	movzbl %al,%eax
   3e3fc:	lea    0x0(,%rax,4),%r12d
   3e404:	mov    0x438(%rsp,%r12,1),%r8d
   3e40c:	lea    0x10(%r8),%r11
   3e410:	mov    0x38(%rsp,%r12,1),%r12d
   3e415:	cmp    %r12,%r11
   3e418:	ja     3e284 <mibz2::bwt::build+0x1a4>
   3e41e:	movdqu (%r14,%r8,1),%xmm6
   3e424:	movdqa %xmm6,%xmm5
   3e428:	punpckhbw %xmm0,%xmm5
   3e42c:	movd   %r9d,%xmm7
   3e431:	pshufd $0x0,%xmm7,%xmm7
   3e436:	pslld  $0x8,%xmm7
   3e43b:	punpcklbw %xmm0,%xmm6
   3e43f:	movdqa %xmm6,%xmm8
   3e444:	punpcklwd %xmm0,%xmm8
   3e449:	por    %xmm7,%xmm8
   3e44e:	punpckhwd %xmm0,%xmm6
   3e452:	por    %xmm7,%xmm6
   3e456:	movdqa %xmm5,%xmm9
   3e45b:	punpcklwd %xmm0,%xmm9
   3e460:	por    %xmm7,%xmm9
   3e465:	punpckhwd %xmm0,%xmm5
   3e469:	por    %xmm7,%xmm5
   3e46d:	por    %xmm1,%xmm8
   3e472:	movdqu %xmm8,(%rcx,%r8,4)
   3e478:	por    %xmm2,%xmm6
   3e47c:	movdqu %xmm6,0x10(%rcx,%r8,4)
   3e483:	por    %xmm3,%xmm9
   3e488:	movdqu %xmm9,0x20(%rcx,%r8,4)
   3e48f:	por    %xmm4,%xmm5
   3e493:	movdqu %xmm5,0x30(%rcx,%r8,4)
   3e49a:	mov    %r11d,0x438(%rsp,%rax,4)
   3e4a2:	jmp    3e9e0 <mibz2::bwt::build+0x900>
   3e4a7:	not    %r8d
   3e4aa:	mov    %r8d,%r11d
   3e4ad:	or     $0x1,%r11d
   3e4b1:	movzwl %r11w,%r11d
   3e4b5:	shr    $1,%r8d
   3e4b8:	and    $0x5555,%r8d
   3e4bf:	mov    %r11d,%ebp
   3e4c2:	sub    %r8d,%ebp
   3e4c5:	mov    %ebp,%r8d
   3e4c8:	and    $0x33333333,%r8d
   3e4cf:	shr    $0x2,%ebp
   3e4d2:	and    $0x33333333,%ebp
   3e4d8:	add    %r8d,%ebp
   3e4db:	mov    %ebp,%r8d
   3e4de:	shr    $0x4,%r8d
   3e4e2:	add    %ebp,%r8d
   3e4e5:	and    $0xf0f0f0f,%r8d
   3e4ec:	imul   $0x1010101,%r8d,%r8d
   3e4f3:	shr    $0x18,%r8d
   3e4f7:	cmp    $0x6,%r8d
   3e4fb:	jbe    3eaab <mibz2::bwt::build+0x9cb>
   3e501:	movzbl %al,%r13d
   3e505:	lea    0x0(,%r13,4),%r8d
   3e50d:	mov    0x438(%rsp,%r8,1),%eax
   3e515:	cmp    0x38(%rsp,%r8,1),%eax
   3e51a:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e520:	cmp    %rax,%rbx
   3e523:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e529:	movzbl 0x11(%rsp),%edi
   3e52e:	mov    %dil,0x28(%rsp)
   3e533:	movzbl 0x12(%rsp),%r8d
   3e539:	movzbl 0x13(%rsp),%r12d
   3e53f:	movzbl 0x14(%rsp),%edi
   3e544:	mov    %dil,0xf(%rsp)
   3e549:	movzbl 0x15(%rsp),%edi
   3e54e:	mov    %dil,0xe(%rsp)
   3e553:	movzbl 0x16(%rsp),%edi
   3e558:	mov    %dil,0xd(%rsp)
   3e55d:	movzbl 0x17(%rsp),%edi
   3e562:	mov    %dil,0xc(%rsp)
   3e567:	movzbl 0x18(%rsp),%edi
   3e56c:	mov    %dil,0xb(%rsp)
   3e571:	movzbl 0x19(%rsp),%edi
   3e576:	mov    %dil,0xa(%rsp)
   3e57b:	movzbl 0x1a(%rsp),%edi
   3e580:	mov    %dil,0x9(%rsp)
   3e585:	movzbl 0x1b(%rsp),%edi
   3e58a:	mov    %dil,0x8(%rsp)
   3e58f:	movzbl 0x1c(%rsp),%edi
   3e594:	mov    %dil,0x7(%rsp)
   3e599:	movzbl 0x1d(%rsp),%edi
   3e59e:	mov    %dil,0x6(%rsp)
   3e5a3:	movzbl 0x1e(%rsp),%edi
   3e5a8:	mov    %dil,0x5(%rsp)
   3e5ad:	movzbl 0x1f(%rsp),%edi
   3e5b2:	movzbl (%r14,%rax,1),%ebp
   3e5b7:	mov    %r9d,%r11d
   3e5ba:	shl    $0x8,%r11d
   3e5be:	or     %r11d,%ebp
   3e5c1:	mov    %ebp,(%rcx,%rax,4)
   3e5c4:	inc    %eax
   3e5c6:	mov    %eax,0x438(%rsp,%r13,4)
   3e5ce:	movzbl 0x28(%rsp),%r13d
   3e5d4:	lea    0x0(,%r13,4),%ebp
   3e5dc:	mov    0x438(%rsp,%rbp,1),%eax
   3e5e3:	cmp    0x38(%rsp,%rbp,1),%eax
   3e5e7:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e5ed:	cmp    %rax,%rbx
   3e5f0:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e5f6:	movzbl (%r14,%rax,1),%ebp
   3e5fb:	add    %r11d,%ebp
   3e5fe:	add    $0x100,%ebp
   3e604:	mov    %ebp,(%rcx,%rax,4)
   3e607:	inc    %eax
   3e609:	mov    %eax,0x438(%rsp,%r13,4)
   3e611:	movzbl %r8b,%r8d
   3e615:	lea    0x0(,%r8,4),%r13d
   3e61d:	mov    0x438(%rsp,%r13,1),%eax
   3e625:	cmp    0x38(%rsp,%r13,1),%eax
   3e62a:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e630:	cmp    %rax,%rbx
   3e633:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e639:	movzbl (%r14,%rax,1),%r13d
   3e63e:	lea    (%r11,%r13,1),%ebp
   3e642:	add    $0x200,%ebp
   3e648:	mov    %ebp,(%rcx,%rax,4)
   3e64b:	inc    %eax
   3e64d:	mov    %eax,0x438(%rsp,%r8,4)
   3e655:	movzbl %r12b,%r8d
   3e659:	lea    0x0(,%r8,4),%r12d
   3e661:	mov    0x438(%rsp,%r12,1),%eax
   3e669:	cmp    0x38(%rsp,%r12,1),%eax
   3e66e:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e674:	cmp    %rax,%rbx
   3e677:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e67d:	movzbl (%r14,%rax,1),%r12d
   3e682:	lea    (%r11,%r12,1),%ebp
   3e686:	add    $0x300,%ebp
   3e68c:	mov    %ebp,(%rcx,%rax,4)
   3e68f:	inc    %eax
   3e691:	mov    %eax,0x438(%rsp,%r8,4)
   3e699:	movzbl 0xf(%rsp),%r8d
   3e69f:	lea    0x0(,%r8,4),%r12d
   3e6a7:	mov    0x438(%rsp,%r12,1),%eax
   3e6af:	cmp    0x38(%rsp,%r12,1),%eax
   3e6b4:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e6ba:	cmp    %rax,%rbx
   3e6bd:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e6c3:	movzbl (%r14,%rax,1),%r12d
   3e6c8:	lea    (%r11,%r12,1),%ebp
   3e6cc:	add    $0x400,%ebp
   3e6d2:	mov    %ebp,(%rcx,%rax,4)
   3e6d5:	inc    %eax
   3e6d7:	mov    %eax,0x438(%rsp,%r8,4)
   3e6df:	movzbl 0xe(%rsp),%r8d
   3e6e5:	lea    0x0(,%r8,4),%r12d
   3e6ed:	mov    0x438(%rsp,%r12,1),%eax
   3e6f5:	cmp    0x38(%rsp,%r12,1),%eax
   3e6fa:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e700:	cmp    %rax,%rbx
   3e703:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e709:	movzbl (%r14,%rax,1),%r12d
   3e70e:	lea    (%r11,%r12,1),%ebp
   3e712:	add    $0x500,%ebp
   3e718:	mov    %ebp,(%rcx,%rax,4)
   3e71b:	inc    %eax
   3e71d:	mov    %eax,0x438(%rsp,%r8,4)
   3e725:	movzbl 0xd(%rsp),%r8d
   3e72b:	lea    0x0(,%r8,4),%r12d
   3e733:	mov    0x438(%rsp,%r12,1),%eax
   3e73b:	cmp    0x38(%rsp,%r12,1),%eax
   3e740:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e746:	cmp    %rax,%rbx
   3e749:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e74f:	movzbl (%r14,%rax,1),%r12d
   3e754:	lea    (%r11,%r12,1),%ebp
   3e758:	add    $0x600,%ebp
   3e75e:	mov    %ebp,(%rcx,%rax,4)
   3e761:	inc    %eax
   3e763:	mov    %eax,0x438(%rsp,%r8,4)
   3e76b:	movzbl 0xc(%rsp),%r8d
   3e771:	lea    0x0(,%r8,4),%r12d
   3e779:	mov    0x438(%rsp,%r12,1),%eax
   3e781:	cmp    0x38(%rsp,%r12,1),%eax
   3e786:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e78c:	cmp    %rax,%rbx
   3e78f:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e795:	movzbl (%r14,%rax,1),%r12d
   3e79a:	lea    (%r11,%r12,1),%ebp
   3e79e:	add    $0x700,%ebp
   3e7a4:	mov    %ebp,(%rcx,%rax,4)
   3e7a7:	inc    %eax
   3e7a9:	mov    %eax,0x438(%rsp,%r8,4)
   3e7b1:	movzbl 0xb(%rsp),%r8d
   3e7b7:	lea    0x0(,%r8,4),%r12d
   3e7bf:	mov    0x438(%rsp,%r12,1),%eax
   3e7c7:	cmp    0x38(%rsp,%r12,1),%eax
   3e7cc:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e7d2:	cmp    %rax,%rbx
   3e7d5:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e7db:	movzbl (%r14,%rax,1),%r12d
   3e7e0:	lea    (%r11,%r12,1),%ebp
   3e7e4:	add    $0x800,%ebp
   3e7ea:	mov    %ebp,(%rcx,%rax,4)
   3e7ed:	inc    %eax
   3e7ef:	mov    %eax,0x438(%rsp,%r8,4)
   3e7f7:	movzbl 0xa(%rsp),%r8d
   3e7fd:	lea    0x0(,%r8,4),%r12d
   3e805:	mov    0x438(%rsp,%r12,1),%eax
   3e80d:	cmp    0x38(%rsp,%r12,1),%eax
   3e812:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e818:	cmp    %rax,%rbx
   3e81b:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e821:	movzbl (%r14,%rax,1),%r12d
   3e826:	lea    (%r11,%r12,1),%ebp
   3e82a:	add    $0x900,%ebp
   3e830:	mov    %ebp,(%rcx,%rax,4)
   3e833:	inc    %eax
   3e835:	mov    %eax,0x438(%rsp,%r8,4)
   3e83d:	movzbl 0x9(%rsp),%r8d
   3e843:	lea    0x0(,%r8,4),%r12d
   3e84b:	mov    0x438(%rsp,%r12,1),%eax
   3e853:	cmp    0x38(%rsp,%r12,1),%eax
   3e858:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e85e:	cmp    %rax,%rbx
   3e861:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e867:	movzbl (%r14,%rax,1),%r12d
   3e86c:	lea    (%r11,%r12,1),%ebp
   3e870:	add    $0xa00,%ebp
   3e876:	mov    %ebp,(%rcx,%rax,4)
   3e879:	inc    %eax
   3e87b:	mov    %eax,0x438(%rsp,%r8,4)
   3e883:	movzbl 0x8(%rsp),%r8d
   3e889:	lea    0x0(,%r8,4),%r12d
   3e891:	mov    0x438(%rsp,%r12,1),%eax
   3e899:	cmp    0x38(%rsp,%r12,1),%eax
   3e89e:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e8a4:	cmp    %rax,%rbx
   3e8a7:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e8ad:	movzbl (%r14,%rax,1),%r12d
   3e8b2:	lea    (%r11,%r12,1),%ebp
   3e8b6:	add    $0xb00,%ebp
   3e8bc:	mov    %ebp,(%rcx,%rax,4)
   3e8bf:	inc    %eax
   3e8c1:	mov    %eax,0x438(%rsp,%r8,4)
   3e8c9:	movzbl 0x7(%rsp),%r8d
   3e8cf:	lea    0x0(,%r8,4),%r12d
   3e8d7:	mov    0x438(%rsp,%r12,1),%eax
   3e8df:	cmp    0x38(%rsp,%r12,1),%eax
   3e8e4:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e8ea:	cmp    %rax,%rbx
   3e8ed:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e8f3:	movzbl (%r14,%rax,1),%r12d
   3e8f8:	lea    (%r11,%r12,1),%ebp
   3e8fc:	add    $0xc00,%ebp
   3e902:	mov    %ebp,(%rcx,%rax,4)
   3e905:	inc    %eax
   3e907:	mov    %eax,0x438(%rsp,%r8,4)
   3e90f:	movzbl 0x6(%rsp),%r8d
   3e915:	lea    0x0(,%r8,4),%r12d
   3e91d:	mov    0x438(%rsp,%r12,1),%eax
   3e925:	cmp    0x38(%rsp,%r12,1),%eax
   3e92a:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e930:	cmp    %rax,%rbx
   3e933:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e939:	movzbl (%r14,%rax,1),%r12d
   3e93e:	lea    (%r11,%r12,1),%ebp
   3e942:	add    $0xd00,%ebp
   3e948:	mov    %ebp,(%rcx,%rax,4)
   3e94b:	inc    %eax
   3e94d:	mov    %eax,0x438(%rsp,%r8,4)
   3e955:	movzbl 0x5(%rsp),%r8d
   3e95b:	lea    0x0(,%r8,4),%r12d
   3e963:	mov    0x438(%rsp,%r12,1),%eax
   3e96b:	cmp    0x38(%rsp,%r12,1),%eax
   3e970:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e976:	cmp    %rax,%rbx
   3e979:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e97f:	movzbl (%r14,%rax,1),%r12d
   3e984:	lea    (%r11,%r12,1),%ebp
   3e988:	add    $0xe00,%ebp
   3e98e:	mov    %ebp,(%rcx,%rax,4)
   3e991:	inc    %eax
   3e993:	mov    %eax,0x438(%rsp,%r8,4)
   3e99b:	movzbl %dil,%r8d
   3e99f:	lea    0x0(,%r8,4),%r12d
   3e9a7:	mov    0x438(%rsp,%r12,1),%eax
   3e9af:	cmp    0x38(%rsp,%r12,1),%eax
   3e9b4:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3e9ba:	cmp    %rax,%rbx
   3e9bd:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3e9c3:	movzbl (%r14,%rax,1),%r12d
   3e9c8:	add    %r12d,%r11d
   3e9cb:	add    $0xf00,%r11d
   3e9d2:	mov    %r11d,(%rcx,%rax,4)
   3e9d6:	inc    %eax
   3e9d8:	mov    %eax,0x438(%rsp,%r8,4)
   3e9e0:	mov    0x30(%rsp),%rdi
   3e9e5:	add    $0x10,%rdi
   3e9e9:	add    $0x10,%r9
   3e9ed:	add    $0x10,%rdx
   3e9f1:	add    $0xfffffffffffffff0,%r10
   3e9f5:	cmp    %rbx,%rdi
   3e9f8:	jbe    3e3bd <mibz2::bwt::build+0x2dd>
   3e9fe:	jmp    3e209 <mibz2::bwt::build+0x129>
   3ea03:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ea10:	sub    %r13d,%r12d
   3ea13:	movdqu (%r14,%r8,1),%xmm6
   3ea19:	movdqa %xmm6,%xmm5
   3ea1d:	punpckhbw %xmm0,%xmm5
   3ea21:	movd   %eax,%xmm7
   3ea25:	pshufd $0x0,%xmm7,%xmm7
   3ea2a:	pslld  $0x8,%xmm7
   3ea2f:	punpcklbw %xmm0,%xmm6
   3ea33:	movdqa %xmm6,%xmm8
   3ea38:	punpcklwd %xmm0,%xmm8
   3ea3d:	por    %xmm7,%xmm8
   3ea42:	punpckhwd %xmm0,%xmm6
   3ea46:	por    %xmm7,%xmm6
   3ea4a:	movdqa %xmm5,%xmm9
   3ea4f:	punpcklwd %xmm0,%xmm9
   3ea54:	por    %xmm7,%xmm9
   3ea59:	punpckhwd %xmm0,%xmm5
   3ea5d:	por    %xmm7,%xmm5
   3ea61:	paddd  %xmm1,%xmm8
   3ea66:	movdqu %xmm8,(%rcx,%r8,4)
   3ea6c:	paddd  %xmm2,%xmm6
   3ea70:	movdqu %xmm6,0x10(%rcx,%r8,4)
   3ea77:	paddd  %xmm3,%xmm9
   3ea7c:	movdqu %xmm9,0x20(%rcx,%r8,4)
   3ea83:	paddd  %xmm4,%xmm5
   3ea87:	movdqu %xmm5,0x30(%rcx,%r8,4)
   3ea8e:	lea    (%r8,%r12,1),%eax
   3ea92:	mov    0x28(%rsp),%r8
   3ea97:	mov    %eax,0x438(%rsp,%r8,4)
   3ea9f:	mov    %rdi,%r9
   3eaa2:	test   %r11d,%r11d
   3eaa5:	je     3e9e0 <mibz2::bwt::build+0x900>
   3eaab:	tzcnt  %r11d,%r13d
   3eab0:	lea    (%r9,%r13,1),%rax
   3eab4:	cmp    %rbx,%rax
   3eab7:	jae    3eba2 <mibz2::bwt::build+0xac2>
   3eabd:	mov    %r11d,%r8d
   3eac0:	mov    %r9,%rdi
   3eac3:	lea    -0x1(%r8),%r11d
   3eac7:	and    %r8d,%r11d
   3eaca:	mov    %r11d,%r8d
   3eacd:	or     $0x10000,%r8d
   3ead4:	tzcnt  %r8d,%r12d
   3ead9:	movzbl (%r14,%rax,1),%r8d
   3eade:	mov    %r8,0x28(%rsp)
   3eae3:	lea    0x0(,%r8,4),%r9d
   3eaeb:	mov    0x438(%rsp,%r9,1),%r8d
   3eaf3:	lea    0x10(%r8),%rbp
   3eaf7:	mov    0x38(%rsp,%r9,1),%r9d
   3eafc:	cmp    %r9,%rbp
   3eaff:	jbe    3ea10 <mibz2::bwt::build+0x930>
   3eb05:	cmp    %r12d,%r13d
   3eb08:	mov    %rdi,%r9
   3eb0b:	jae    3eaa2 <mibz2::bwt::build+0x9c2>
   3eb0d:	lea    (%r9,%r13,1),%r8d
   3eb11:	shl    $0x8,%r8d
   3eb15:	data16 cs nopw 0x0(%rax,%rax,1)
   3eb20:	cmp    %r13,%r10
   3eb23:	je     3eb81 <mibz2::bwt::build+0xaa1>
   3eb25:	movzbl (%rdx,%r13,1),%ebp
   3eb2a:	lea    0x0(,%rbp,4),%edi
   3eb31:	mov    0x438(%rsp,%rdi,1),%eax
   3eb38:	cmp    0x38(%rsp,%rdi,1),%eax
   3eb3c:	jae    3e284 <mibz2::bwt::build+0x1a4>
   3eb42:	cmp    %rax,%rbx
   3eb45:	jbe    3eb6f <mibz2::bwt::build+0xa8f>
   3eb47:	movzbl (%r14,%rax,1),%edi
   3eb4c:	add    %r8d,%edi
   3eb4f:	mov    %edi,(%rcx,%rax,4)
   3eb52:	inc    %eax
   3eb54:	mov    %eax,0x438(%rsp,%rbp,4)
   3eb5b:	inc    %r13
   3eb5e:	add    $0x100,%r8d
   3eb65:	cmp    %r13,%r12
   3eb68:	jne    3eb20 <mibz2::bwt::build+0xa40>
   3eb6a:	jmp    3eaa2 <mibz2::bwt::build+0x9c2>
   3eb6f:	lea    0x5d1a2(%rip),%rdx        # 9bd18 <__frame_dummy_init_array_entry+0xed0>
   3eb76:	mov    %rax,%rdi
   3eb79:	mov    %rbx,%rsi
   3eb7c:	call   2f851 <core::panicking::panic_bounds_check>
   3eb81:	lea    0x5d178(%rip),%rdx        # 9bd00 <__frame_dummy_init_array_entry+0xeb8>
   3eb88:	mov    %rbx,%rdi
   3eb8b:	mov    %rbx,%rsi
   3eb8e:	call   2f851 <core::panicking::panic_bounds_check>
   3eb93:	lea    0x5d17e(%rip),%rdx        # 9bd18 <__frame_dummy_init_array_entry+0xed0>
   3eb9a:	mov    %rbx,%rsi
   3eb9d:	call   2f851 <core::panicking::panic_bounds_check>
   3eba2:	lea    0x5d52f(%rip),%rdx        # 9c0d8 <__frame_dummy_init_array_entry+0x1290>
   3eba9:	mov    %rax,%rdi
   3ebac:	mov    %rbx,%rsi
   3ebaf:	call   2f851 <core::panicking::panic_bounds_check>
   3ebb4:	lea    0x5d505(%rip),%rdx        # 9c0c0 <__frame_dummy_init_array_entry+0x1278>
   3ebbb:	mov    %r9,%rdi
   3ebbe:	mov    %rbx,%rsi
   3ebc1:	call   2f851 <core::panicking::panic_bounds_check>
   3ebc6:	add    $0x838,%rsp
   3ebcd:	pop    %rbx
   3ebce:	pop    %r12
   3ebd0:	pop    %r13
   3ebd2:	pop    %r14
   3ebd4:	pop    %r15
   3ebd6:	pop    %rbp
   3ebd7:	ret
   3ebd8:	movzbl (%r14,%rax,1),%edx
   3ebdd:	mov    %edx,0x0(%r13,%r12,4)
   3ebe2:	inc    %r12
   3ebe5:	inc    %rax
   3ebe8:	cmp    %rax,%rbx
   3ebeb:	jne    3ebd8 <mibz2::bwt::build+0xaf8>
   3ebed:	mov    %r12,0x10(%rcx)
   3ebf1:	lea    0x38(%rsp),%rdi
   3ebf6:	xor    %ebp,%ebp
   3ebf8:	mov    $0x400,%edx
   3ebfd:	xor    %esi,%esi
   3ebff:	call   *0x5f133(%rip)        # 9dd38 <memset@GLIBC_2.2.5>
   3ec05:	xor    %eax,%eax
   3ec07:	nopw   0x0(%rax,%rax,1)
   3ec10:	mov    %eax,0x38(%rsp,%rbp,4)
   3ec14:	add    (%r15,%rbp,4),%eax
   3ec18:	mov    %eax,0x3c(%rsp,%rbp,4)
   3ec1c:	add    0x4(%r15,%rbp,4),%eax
   3ec21:	mov    %eax,0x40(%rsp,%rbp,4)
   3ec25:	add    0x8(%r15,%rbp,4),%eax
   3ec2a:	mov    %eax,0x44(%rsp,%rbp,4)
   3ec2e:	add    0xc(%r15,%rbp,4),%eax
   3ec33:	add    $0x4,%rbp
   3ec37:	cmp    $0x100,%rbp
   3ec3e:	jne    3ec10 <mibz2::bwt::build+0xb30>
   3ec40:	xor    %ecx,%ecx
   3ec42:	cmp    $0x10,%rbx
   3ec46:	jae    3ec99 <mibz2::bwt::build+0xbb9>
   3ec48:	xor    %eax,%eax
   3ec4a:	cmp    %rbx,%rax
   3ec4d:	jae    3ebc6 <mibz2::bwt::build+0xae6>
   3ec53:	mov    %eax,%ecx
   3ec55:	shl    $0x8,%ecx
   3ec58:	nopl   0x0(%rax,%rax,1)
   3ec60:	movzbl (%r14,%rax,1),%edx
   3ec65:	mov    0x38(%rsp,%rdx,4),%edi
   3ec69:	cmp    %rdi,%r12
   3ec6c:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ec72:	inc    %rax
   3ec75:	movzbl 0x0(%r13,%rdi,4),%esi
   3ec7b:	add    %ecx,%esi
   3ec7d:	mov    %esi,0x0(%r13,%rdi,4)
   3ec82:	lea    0x1(%rdi),%esi
   3ec85:	mov    %esi,0x38(%rsp,%rdx,4)
   3ec89:	add    $0x100,%ecx
   3ec8f:	cmp    %rax,%rbx
   3ec92:	jne    3ec60 <mibz2::bwt::build+0xb80>
   3ec94:	jmp    3ebc6 <mibz2::bwt::build+0xae6>
   3ec99:	movdqa -0x366a1(%rip),%xmm0        # 8600 <GCC_except_table759+0x40c>
   3eca1:	movaps -0x36728(%rip),%xmm1        # 8580 <GCC_except_table759+0x38c>
   3eca8:	movdqa -0x36a20(%rip),%xmm2        # 8290 <GCC_except_table759+0x9c>
   3ecb0:	movdqa -0x36a48(%rip),%xmm3        # 8270 <GCC_except_table759+0x7c>
   3ecb8:	movdqa -0x367c0(%rip),%xmm4        # 8500 <GCC_except_table759+0x30c>
   3ecc0:	xor    %esi,%esi
   3ecc2:	jmp    3ed04 <mibz2::bwt::build+0xc24>
   3ecc4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ecd0:	movzbl 0x0(%r13,%rdi,4),%edx
   3ecd6:	add    %ecx,%edx
   3ecd8:	add    $0xf00,%edx
   3ecde:	mov    %edx,0x0(%r13,%rdi,4)
   3ece3:	lea    0x1(%rdi),%edx
   3ece6:	mov    %edx,0x38(%rsp,%rax,4)
   3ecea:	lea    0x10(%rsi),%rax
   3ecee:	add    $0x20,%rsi
   3ecf2:	add    $0x1000,%ecx
   3ecf8:	cmp    %rbx,%rsi
   3ecfb:	mov    %rax,%rsi
   3ecfe:	ja     3ec4a <mibz2::bwt::build+0xb6a>
   3ed04:	lea    0xf(%rsi),%rax
   3ed08:	cmp    %rbx,%rax
   3ed0b:	jae    3f102 <mibz2::bwt::build+0x1022>
   3ed11:	mov    (%r14,%rsi,1),%rdx
   3ed15:	mov    0x8(%r14,%rsi,1),%rax
   3ed1a:	mov    %rdx,%rdi
   3ed1d:	xor    %rax,%rdi
   3ed20:	mov    %rdx,%r8
   3ed23:	rol    $0x8,%r8
   3ed27:	xor    %rdx,%r8
   3ed2a:	or     %rdi,%r8
   3ed2d:	jne    3ede0 <mibz2::bwt::build+0xd00>
   3ed33:	movzbl %dl,%edx
   3ed36:	mov    %edx,%eax
   3ed38:	mov    0x38(%rsp,%rax,4),%edi
   3ed3c:	lea    0x10(%rdi),%rax
   3ed40:	cmp    %r12,%rax
   3ed43:	ja     3f126 <mibz2::bwt::build+0x1046>
   3ed49:	movdqu 0x0(%r13,%rdi,4),%xmm6
   3ed50:	movdqu 0x10(%r13,%rdi,4),%xmm7
   3ed57:	movdqu 0x20(%r13,%rdi,4),%xmm8
   3ed5e:	movdqu 0x30(%r13,%rdi,4),%xmm5
   3ed65:	pand   %xmm0,%xmm6
   3ed69:	movd   %ecx,%xmm9
   3ed6e:	movaps %xmm1,%xmm10
   3ed72:	movss  %xmm9,%xmm10
   3ed77:	movd   %ecx,%xmm9
   3ed7c:	pshufd $0x1,%xmm9,%xmm11
   3ed82:	por    %xmm10,%xmm11
   3ed87:	por    %xmm6,%xmm11
   3ed8c:	movdqu %xmm11,0x0(%r13,%rdi,4)
   3ed93:	pand   %xmm0,%xmm7
   3ed97:	pshufd $0x0,%xmm9,%xmm6
   3ed9d:	por    %xmm6,%xmm7
   3eda1:	por    %xmm2,%xmm7
   3eda5:	movdqu %xmm7,0x10(%r13,%rdi,4)
   3edac:	pand   %xmm0,%xmm8
   3edb1:	por    %xmm6,%xmm8
   3edb6:	por    %xmm3,%xmm8
   3edbb:	movdqu %xmm8,0x20(%r13,%rdi,4)
   3edc2:	pand   %xmm0,%xmm5
   3edc6:	por    %xmm6,%xmm5
   3edca:	por    %xmm4,%xmm5
   3edce:	movdqu %xmm5,0x30(%r13,%rdi,4)
   3edd5:	addl   $0x10,0x38(%rsp,%rdx,4)
   3edda:	jmp    3ecea <mibz2::bwt::build+0xc0a>
   3eddf:	nop
   3ede0:	movzbl %dl,%r8d
   3ede4:	mov    %r8d,%edi
   3ede7:	mov    0x38(%rsp,%rdi,4),%edi
   3edeb:	cmp    %rdi,%r12
   3edee:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3edf4:	mov    %rdx,%r9
   3edf7:	shr    $0x8,%r9
   3edfb:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee01:	add    %ecx,%r10d
   3ee04:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee09:	inc    %edi
   3ee0b:	mov    %edi,0x38(%rsp,%r8,4)
   3ee10:	movzbl %r9b,%r8d
   3ee14:	mov    %r8d,%edi
   3ee17:	mov    0x38(%rsp,%rdi,4),%edi
   3ee1b:	cmp    %rdi,%r12
   3ee1e:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ee24:	mov    %rdx,%r9
   3ee27:	shr    $0x10,%r9
   3ee2b:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee31:	add    %ecx,%r10d
   3ee34:	add    $0x100,%r10d
   3ee3b:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee40:	inc    %edi
   3ee42:	mov    %edi,0x38(%rsp,%r8,4)
   3ee47:	movzbl %r9b,%r8d
   3ee4b:	mov    %r8d,%edi
   3ee4e:	mov    0x38(%rsp,%rdi,4),%edi
   3ee52:	cmp    %rdi,%r12
   3ee55:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ee5b:	mov    %rdx,%r9
   3ee5e:	shr    $0x18,%r9
   3ee62:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee68:	add    %ecx,%r10d
   3ee6b:	add    $0x200,%r10d
   3ee72:	mov    %r10d,0x0(%r13,%rdi,4)
   3ee77:	inc    %edi
   3ee79:	mov    %edi,0x38(%rsp,%r8,4)
   3ee7e:	movzbl %r9b,%r8d
   3ee82:	mov    %r8d,%edi
   3ee85:	mov    0x38(%rsp,%rdi,4),%edi
   3ee89:	cmp    %rdi,%r12
   3ee8c:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ee92:	mov    %rdx,%r9
   3ee95:	shr    $0x20,%r9
   3ee99:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ee9f:	add    %ecx,%r10d
   3eea2:	add    $0x300,%r10d
   3eea9:	mov    %r10d,0x0(%r13,%rdi,4)
   3eeae:	inc    %edi
   3eeb0:	mov    %edi,0x38(%rsp,%r8,4)
   3eeb5:	movzbl %r9b,%r8d
   3eeb9:	mov    %r8d,%edi
   3eebc:	mov    0x38(%rsp,%rdi,4),%edi
   3eec0:	cmp    %rdi,%r12
   3eec3:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3eec9:	mov    %rdx,%r9
   3eecc:	shr    $0x28,%r9
   3eed0:	movzbl 0x0(%r13,%rdi,4),%r10d
   3eed6:	add    %ecx,%r10d
   3eed9:	add    $0x400,%r10d
   3eee0:	mov    %r10d,0x0(%r13,%rdi,4)
   3eee5:	inc    %edi
   3eee7:	mov    %edi,0x38(%rsp,%r8,4)
   3eeec:	movzbl %r9b,%r8d
   3eef0:	mov    %r8d,%edi
   3eef3:	mov    0x38(%rsp,%rdi,4),%edi
   3eef7:	cmp    %rdi,%r12
   3eefa:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ef00:	mov    %rdx,%r9
   3ef03:	shr    $0x30,%r9
   3ef07:	movzbl 0x0(%r13,%rdi,4),%r10d
   3ef0d:	add    %ecx,%r10d
   3ef10:	add    $0x500,%r10d
   3ef17:	mov    %r10d,0x0(%r13,%rdi,4)
   3ef1c:	inc    %edi
   3ef1e:	mov    %edi,0x38(%rsp,%r8,4)
   3ef23:	movzbl %r9b,%r8d
   3ef27:	mov    %r8d,%edi
   3ef2a:	mov    0x38(%rsp,%rdi,4),%edi
   3ef2e:	cmp    %rdi,%r12
   3ef31:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ef37:	shr    $0x38,%rdx
   3ef3b:	movzbl 0x0(%r13,%rdi,4),%r9d
   3ef41:	add    %ecx,%r9d
   3ef44:	add    $0x600,%r9d
   3ef4b:	mov    %r9d,0x0(%r13,%rdi,4)
   3ef50:	inc    %edi
   3ef52:	mov    %edi,0x38(%rsp,%r8,4)
   3ef57:	mov    0x38(%rsp,%rdx,4),%edi
   3ef5b:	cmp    %rdi,%r12
   3ef5e:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ef64:	movzbl 0x0(%r13,%rdi,4),%r8d
   3ef6a:	add    %ecx,%r8d
   3ef6d:	add    $0x700,%r8d
   3ef74:	mov    %r8d,0x0(%r13,%rdi,4)
   3ef79:	inc    %edi
   3ef7b:	mov    %edi,0x38(%rsp,%rdx,4)
   3ef7f:	movzbl %al,%edx
   3ef82:	mov    %edx,%edi
   3ef84:	mov    0x38(%rsp,%rdi,4),%edi
   3ef88:	cmp    %rdi,%r12
   3ef8b:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3ef91:	mov    %rax,%r8
   3ef94:	shr    $0x8,%r8
   3ef98:	movzbl 0x0(%r13,%rdi,4),%r9d
   3ef9e:	add    %ecx,%r9d
   3efa1:	add    $0x800,%r9d
   3efa8:	mov    %r9d,0x0(%r13,%rdi,4)
   3efad:	inc    %edi
   3efaf:	mov    %edi,0x38(%rsp,%rdx,4)
   3efb3:	movzbl %r8b,%edx
   3efb7:	mov    %edx,%edi
   3efb9:	mov    0x38(%rsp,%rdi,4),%edi
   3efbd:	cmp    %rdi,%r12
   3efc0:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3efc6:	mov    %rax,%r8
   3efc9:	shr    $0x10,%r8
   3efcd:	movzbl 0x0(%r13,%rdi,4),%r9d
   3efd3:	add    %ecx,%r9d
   3efd6:	add    $0x900,%r9d
   3efdd:	mov    %r9d,0x0(%r13,%rdi,4)
   3efe2:	inc    %edi
   3efe4:	mov    %edi,0x38(%rsp,%rdx,4)
   3efe8:	movzbl %r8b,%edx
   3efec:	mov    %edx,%edi
   3efee:	mov    0x38(%rsp,%rdi,4),%edi
   3eff2:	cmp    %rdi,%r12
   3eff5:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3effb:	mov    %rax,%r8
   3effe:	shr    $0x18,%r8
   3f002:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f008:	add    %ecx,%r9d
   3f00b:	add    $0xa00,%r9d
   3f012:	mov    %r9d,0x0(%r13,%rdi,4)
   3f017:	inc    %edi
   3f019:	mov    %edi,0x38(%rsp,%rdx,4)
   3f01d:	movzbl %r8b,%edx
   3f021:	mov    %edx,%edi
   3f023:	mov    0x38(%rsp,%rdi,4),%edi
   3f027:	cmp    %rdi,%r12
   3f02a:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3f030:	mov    %rax,%r8
   3f033:	shr    $0x20,%r8
   3f037:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f03d:	add    %ecx,%r9d
   3f040:	add    $0xb00,%r9d
   3f047:	mov    %r9d,0x0(%r13,%rdi,4)
   3f04c:	inc    %edi
   3f04e:	mov    %edi,0x38(%rsp,%rdx,4)
   3f052:	movzbl %r8b,%edx
   3f056:	mov    %edx,%edi
   3f058:	mov    0x38(%rsp,%rdi,4),%edi
   3f05c:	cmp    %rdi,%r12
   3f05f:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3f065:	mov    %rax,%r8
   3f068:	shr    $0x28,%r8
   3f06c:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f072:	add    %ecx,%r9d
   3f075:	add    $0xc00,%r9d
   3f07c:	mov    %r9d,0x0(%r13,%rdi,4)
   3f081:	inc    %edi
   3f083:	mov    %edi,0x38(%rsp,%rdx,4)
   3f087:	movzbl %r8b,%edx
   3f08b:	mov    %edx,%edi
   3f08d:	mov    0x38(%rsp,%rdi,4),%edi
   3f091:	cmp    %rdi,%r12
   3f094:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3f096:	mov    %rax,%r8
   3f099:	shr    $0x30,%r8
   3f09d:	movzbl 0x0(%r13,%rdi,4),%r9d
   3f0a3:	add    %ecx,%r9d
   3f0a6:	add    $0xd00,%r9d
   3f0ad:	mov    %r9d,0x0(%r13,%rdi,4)
   3f0b2:	inc    %edi
   3f0b4:	mov    %edi,0x38(%rsp,%rdx,4)
   3f0b8:	movzbl %r8b,%edx
   3f0bc:	mov    %edx,%edi
   3f0be:	mov    0x38(%rsp,%rdi,4),%edi
   3f0c2:	cmp    %rdi,%r12
   3f0c5:	jbe    3f0f3 <mibz2::bwt::build+0x1013>
   3f0c7:	shr    $0x38,%rax
   3f0cb:	movzbl 0x0(%r13,%rdi,4),%r8d
   3f0d1:	add    %ecx,%r8d
   3f0d4:	add    $0xe00,%r8d
   3f0db:	mov    %r8d,0x0(%r13,%rdi,4)
   3f0e0:	inc    %edi
   3f0e2:	mov    %edi,0x38(%rsp,%rdx,4)
   3f0e6:	mov    0x38(%rsp,%rax,4),%edi
   3f0ea:	cmp    %rdi,%r12
   3f0ed:	ja     3ecd0 <mibz2::bwt::build+0xbf0>
   3f0f3:	lea    0x5d00e(%rip),%rdx        # 9c108 <__frame_dummy_init_array_entry+0x12c0>
   3f0fa:	mov    %r12,%rsi
   3f0fd:	call   2f851 <core::panicking::panic_bounds_check>
   3f102:	movabs $0x7ffffffffffffff0,%rdi
   3f10c:	and    %rbx,%rdi
   3f10f:	or     $0x8,%rdi
   3f113:	add    $0x10,%rsi
   3f117:	lea    0x5d01a(%rip),%rcx        # 9c138 <__frame_dummy_init_array_entry+0x12f0>
   3f11e:	mov    %rbx,%rdx
   3f121:	call   2f770 <core::slice::index::slice_index_fail>
   3f126:	lea    0x5cfc3(%rip),%rcx        # 9c0f0 <__frame_dummy_init_array_entry+0x12a8>
   3f12d:	mov    %rax,%rsi
   3f130:	mov    %r12,%rdx
   3f133:	call   2f770 <core::slice::index::slice_index_fail>
Disassembly of section .init:
Disassembly of section .fini:
Disassembly of section .plt:
