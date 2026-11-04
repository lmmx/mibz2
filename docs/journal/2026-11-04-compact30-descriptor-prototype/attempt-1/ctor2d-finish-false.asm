# <c30walk::ctor2d::W<false>>::finish
# size 0x60a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000092250 <<c30walk::ctor2d::W<false>>::finish>:
   92250:	push   %rbp
   92251:	push   %r15
   92253:	push   %r14
   92255:	push   %r13
   92257:	push   %r12
   92259:	push   %rbx
   9225a:	sub    $0x38,%rsp
   9225e:	mov    0x18(%rdi),%rbp
   92262:	mov    0x10(%rbp),%r8
   92266:	mov    %r8,%r13
   92269:	sub    $0x1,%r13
   9226d:	jb     927ea <<c30walk::ctor2d::W<false>>::finish+0x59a>
   92273:	mov    %rdi,%r11
   92276:	lea    (%rsi,%rsi,2),%rax
   9227a:	mov    0x1040(%rdi,%rax,8),%rdi
   92282:	mov    0x1048(%r11,%rax,8),%r12d
   9228a:	mov    0x1050(%r11,%rax,8),%ecx
   92292:	mov    0x1054(%r11,%rax,8),%eax
   9229a:	mov    0x40(%r11,%rsi,8),%r14
   9229f:	sub    %rdi,%r14
   922a2:	mov    %r14,0x8(%rsp)
   922a7:	shr    $1,%r14
   922aa:	mov    0x8(%rbp),%r10
   922ae:	mov    %r13,%rdx
   922b1:	shl    $0xa,%rdx
   922b5:	add    %r10,%rdx
   922b8:	mov    %r13,%rbx
   922bb:	cmp    %r12d,(%rdx,%rsi,4)
   922bf:	jbe    92338 <<c30walk::ctor2d::W<false>>::finish+0xe8>
   922c1:	mov    %r11,%r15
   922c4:	lea    (%r12,%r14,1),%edx
   922c8:	dec    %edx
   922ca:	mov    %r8,%r9
   922cd:	shl    $0xa,%r9
   922d1:	lea    (%r9,%rsi,4),%r9
   922d5:	lea    (%r10,%r9,1),%r11
   922d9:	add    $0xfffffffffffffc00,%r11
   922e0:	mov    %r8,%rbx
   922e3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   922f0:	dec    %rbx
   922f3:	cmp    %r8,%rbx
   922f6:	jae    9279e <<c30walk::ctor2d::W<false>>::finish+0x54e>
   922fc:	cmp    %r12d,(%r11)
   922ff:	lea    -0x400(%r11),%r11
   92306:	ja     922f0 <<c30walk::ctor2d::W<false>>::finish+0xa0>
   92308:	add    %r10,%r9
   9230b:	add    $0xfffffffffffffc00,%r9
   92312:	mov    %r8,%r13
   92315:	mov    %r15,%r11
   92318:	nopl   0x0(%rax,%rax,1)
   92320:	dec    %r13
   92323:	cmp    %r8,%r13
   92326:	jae    927b0 <<c30walk::ctor2d::W<false>>::finish+0x560>
   9232c:	cmp    %edx,(%r9)
   9232f:	lea    -0x400(%r9),%r9
   92336:	ja     92320 <<c30walk::ctor2d::W<false>>::finish+0xd0>
   92338:	imul   $0xffffffffffffffe2,%rcx,%r15
   9233c:	add    %r12,%r15
   9233f:	cmp    $0xffffffff,%eax
   92342:	mov    %r14,0x10(%rsp)
   92347:	je     9244b <<c30walk::ctor2d::W<false>>::finish+0x1fb>
   9234d:	mov    $0xffffffff,%r9d
   92353:	cmp    %r13,%rbx
   92356:	jae    92370 <<c30walk::ctor2d::W<false>>::finish+0x120>
   92358:	lea    0x1(%rbx),%rcx
   9235c:	cmp    %r8,%rcx
   9235f:	jae    9280d <<c30walk::ctor2d::W<false>>::finish+0x5bd>
   92365:	shl    $0xa,%rcx
   92369:	add    %rcx,%r10
   9236c:	mov    (%r10,%rsi,4),%r9d
   92370:	mov    (%r11),%rdx
   92373:	mov    0x10(%rdx),%rcx
   92377:	cmp    %rax,%rcx
   9237a:	jbe    927fb <<c30walk::ctor2d::W<false>>::finish+0x5ab>
   92380:	test   %r14,%r14
   92383:	je     9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92389:	shl    $0x7,%rax
   9238d:	add    0x8(%rdx),%rax
   92391:	lea    0x800(,%rsi,4),%rdx
   92399:	xor    %r11d,%r11d
   9239c:	nopl   0x0(%rax)
   923a0:	lea    (%r12,%r11,1),%r10d
   923a4:	cmp    %r9d,%r10d
   923a7:	jae    923e0 <<c30walk::ctor2d::W<false>>::finish+0x190>
   923a9:	lea    (%r11,%r15,1),%rcx
   923ad:	cmp    $0x20,%rcx
   923b1:	jae    927c2 <<c30walk::ctor2d::W<false>>::finish+0x572>
   923b7:	movzwl (%rdi,%r11,2),%r8d
   923bc:	inc    %r11
   923bf:	mov    %ebx,%r10d
   923c2:	shl    $0x18,%r10d
   923c6:	shl    $0x8,%r8d
   923ca:	or     %r10d,%r8d
   923cd:	or     %esi,%r8d
   923d0:	mov    %r8d,(%rax,%rcx,4)
   923d4:	cmp    0x10(%rsp),%r11
   923d9:	jne    923a0 <<c30walk::ctor2d::W<false>>::finish+0x150>
   923db:	jmp    92429 <<c30walk::ctor2d::W<false>>::finish+0x1d9>
   923dd:	nopl   (%rax)
   923e0:	mov    %rbx,%r14
   923e3:	shl    $0xa,%r14
   923e7:	add    %rdx,%r14
   923ea:	mov    %rbx,%rcx
   923ed:	jmp    923ff <<c30walk::ctor2d::W<false>>::finish+0x1af>
   923ef:	nop
   923f0:	add    $0x400,%r14
   923f7:	mov    %rbx,%rcx
   923fa:	cmp    %r9d,%r10d
   923fd:	jb     923a9 <<c30walk::ctor2d::W<false>>::finish+0x159>
   923ff:	lea    0x1(%rcx),%rbx
   92403:	mov    $0xffffffff,%r9d
   92409:	cmp    %r13,%rbx
   9240c:	jae    923f0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   9240e:	add    $0x2,%rcx
   92412:	mov    0x10(%rbp),%r8
   92416:	cmp    %r8,%rcx
   92419:	jae    9278c <<c30walk::ctor2d::W<false>>::finish+0x53c>
   9241f:	mov    0x8(%rbp),%rcx
   92423:	mov    (%rcx,%r14,1),%r9d
   92427:	jmp    923f0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   92429:	mov    0x8(%rsp),%rdx
   9242e:	and    $0xfffffffffffffffe,%rdx
   92432:	mov    $0xff,%esi
   92437:	add    $0x38,%rsp
   9243b:	pop    %rbx
   9243c:	pop    %r12
   9243e:	pop    %r13
   92440:	pop    %r14
   92442:	pop    %r15
   92444:	pop    %rbp
   92445:	jmp    *0xa1d8d(%rip)        # 1341d8 <memset@GLIBC_2.2.5>
   9244b:	movzwl (%rdi),%eax
   9244e:	movzwl -0x2(%rdi,%r14,2),%r8d
   92454:	mov    %ebx,%edx
   92456:	shl    $0x10,%edx
   92459:	or     %eax,%edx
   9245b:	mov    %r13d,%r9d
   9245e:	shl    $0x10,%r9d
   92462:	or     %r8d,%r9d
   92465:	sub    %edx,%r9d
   92468:	cmp    $0xfffe,%r9d
   9246f:	jae    924ed <<c30walk::ctor2d::W<false>>::finish+0x29d>
   92471:	mov    0x28(%r11),%r8
   92475:	shl    $0x6,%rcx
   92479:	shl    $0x8,%edx
   9247c:	or     %edx,%esi
   9247e:	mov    %esi,0x3c(%r8,%rcx,1)
   92483:	test   %r14,%r14
   92486:	je     9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   9248c:	cmp    $0x1e,%r14
   92490:	jne    92686 <<c30walk::ctor2d::W<false>>::finish+0x436>
   92496:	movd   %eax,%xmm0
   9249a:	pshuflw $0x0,%xmm0,%xmm0
   9249f:	pshufd $0x44,%xmm0,%xmm1
   924a4:	movdqu (%rdi),%xmm2
   924a8:	movdqu 0x10(%rdi),%xmm3
   924ad:	movdqu 0x20(%rdi),%xmm4
   924b2:	psubw  %xmm1,%xmm2
   924b6:	movdqu %xmm2,(%rdi)
   924ba:	psubw  %xmm1,%xmm3
   924be:	movdqu %xmm3,0x10(%rdi)
   924c3:	psubw  %xmm1,%xmm4
   924c7:	movdqu %xmm4,0x20(%rdi)
   924cc:	movq   0x30(%rdi),%xmm1
   924d1:	psubw  %xmm0,%xmm1
   924d5:	movq   %xmm1,0x30(%rdi)
   924da:	movd   0x38(%rdi),%xmm1
   924df:	psubw  %xmm0,%xmm1
   924e3:	movd   %xmm1,0x38(%rdi)
   924e8:	jmp    9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   924ed:	mov    (%r11),%r10
   924f0:	mov    0x10(%r10),%r9
   924f4:	cmp    (%r10),%r9
   924f7:	mov    %r11,0x30(%rsp)
   924fc:	je     9274c <<c30walk::ctor2d::W<false>>::finish+0x4fc>
   92502:	mov    0x8(%r10),%rax
   92506:	mov    %r9,%rdx
   92509:	shl    $0x7,%rdx
   9250d:	pxor   %xmm0,%xmm0
   92511:	movdqa %xmm0,0x70(%rax,%rdx,1)
   92517:	movdqa %xmm0,0x60(%rax,%rdx,1)
   9251d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   92523:	movdqa %xmm0,0x40(%rax,%rdx,1)
   92529:	movdqa %xmm0,0x30(%rax,%rdx,1)
   9252f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   92535:	movdqa %xmm0,0x10(%rax,%rdx,1)
   9253b:	movdqa %xmm0,(%rax,%rdx,1)
   92540:	lea    0x1(%r9),%rax
   92544:	mov    %rax,0x10(%r10)
   92548:	mov    $0xffffffff,%r8d
   9254e:	cmp    %r13,%rbx
   92551:	jae    92570 <<c30walk::ctor2d::W<false>>::finish+0x320>
   92553:	lea    0x1(%rbx),%rax
   92557:	mov    0x10(%rbp),%r8
   9255b:	cmp    %r8,%rax
   9255e:	jae    9282b <<c30walk::ctor2d::W<false>>::finish+0x5db>
   92564:	shl    $0xa,%rax
   92568:	add    0x8(%rbp),%rax
   9256c:	mov    (%rax,%rsi,4),%r8d
   92570:	mov    %r9,0x8(%rsp)
   92575:	test   %r14,%r14
   92578:	je     92625 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   9257e:	add    0x8(%r10),%rdx
   92582:	lea    0x800(,%rsi,4),%r9
   9258a:	xor    %r10d,%r10d
   9258d:	nopl   (%rax)
   92590:	lea    (%r12,%r10,1),%r11d
   92594:	cmp    %r8d,%r11d
   92597:	jae    925d0 <<c30walk::ctor2d::W<false>>::finish+0x380>
   92599:	lea    (%r10,%r15,1),%rax
   9259d:	cmp    $0x20,%rax
   925a1:	jae    927d6 <<c30walk::ctor2d::W<false>>::finish+0x586>
   925a7:	movzwl (%rdi,%r10,2),%r11d
   925ac:	inc    %r10
   925af:	mov    %ebx,%r14d
   925b2:	shl    $0x18,%r14d
   925b6:	shl    $0x8,%r11d
   925ba:	or     %r14d,%r11d
   925bd:	or     %esi,%r11d
   925c0:	mov    %r11d,(%rdx,%rax,4)
   925c4:	cmp    0x10(%rsp),%r10
   925c9:	jne    92590 <<c30walk::ctor2d::W<false>>::finish+0x340>
   925cb:	jmp    92625 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   925cd:	nopl   (%rax)
   925d0:	mov    %rbx,%r14
   925d3:	shl    $0xa,%r14
   925d7:	add    %r9,%r14
   925da:	mov    %rbx,%rax
   925dd:	jmp    925f7 <<c30walk::ctor2d::W<false>>::finish+0x3a7>
   925df:	nop
   925e0:	mov    0x8(%rbp),%rax
   925e4:	mov    (%rax,%r14,1),%r8d
   925e8:	add    $0x400,%r14
   925ef:	mov    %rbx,%rax
   925f2:	cmp    %r8d,%r11d
   925f5:	jb     92599 <<c30walk::ctor2d::W<false>>::finish+0x349>
   925f7:	lea    0x1(%rax),%rbx
   925fb:	mov    $0xffffffff,%r8d
   92601:	cmp    %r13,%rbx
   92604:	jae    925e8 <<c30walk::ctor2d::W<false>>::finish+0x398>
   92606:	add    $0x2,%rax
   9260a:	mov    0x10(%rbp),%r8
   9260e:	cmp    %r8,%rax
   92611:	jb     925e0 <<c30walk::ctor2d::W<false>>::finish+0x390>
   92613:	lea    0x9df36(%rip),%rdx        # 130550 <__frame_dummy_init_array_entry+0x2630>
   9261a:	mov    %rax,%rdi
   9261d:	mov    %r8,%rsi
   92620:	call   d7533 <core::panicking::panic_bounds_check>
   92625:	mov    0x30(%rsp),%r8
   9262a:	mov    0x28(%r8),%rax
   9262e:	mov    %rcx,%rdx
   92631:	shl    $0x6,%rdx
   92635:	mov    0x8(%rsp),%rsi
   9263a:	or     $0x10000000,%esi
   92640:	pcmpeqd %xmm0,%xmm0
   92644:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9264a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   92650:	movdqa %xmm0,0x10(%rax,%rdx,1)
   92656:	movdqa %xmm0,(%rax,%rdx,1)
   9265b:	mov    %esi,0x3c(%rax,%rdx,1)
   9265f:	mov    %ecx,%edi
   92661:	shr    $0x6,%edi
   92664:	mov    0x10(%r8),%rsi
   92668:	cmp    %rsi,%rdi
   9266b:	jae    9281f <<c30walk::ctor2d::W<false>>::finish+0x5cf>
   92671:	mov    $0x1,%eax
   92676:	shl    %cl,%rax
   92679:	mov    0x8(%r8),%rcx
   9267d:	or     %rax,(%rcx,%rdi,8)
   92681:	jmp    9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92686:	mov    0x8(%rsp),%rcx
   9268b:	cmp    $0x8,%rcx
   9268f:	jae    92698 <<c30walk::ctor2d::W<false>>::finish+0x448>
   92691:	xor    %ecx,%ecx
   92693:	jmp    92842 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   92698:	movabs $0x7ffffffffffffff0,%rsi
   926a2:	cmp    $0x20,%rcx
   926a6:	jae    926ac <<c30walk::ctor2d::W<false>>::finish+0x45c>
   926a8:	xor    %ecx,%ecx
   926aa:	jmp    9270b <<c30walk::ctor2d::W<false>>::finish+0x4bb>
   926ac:	mov    %r14,%rcx
   926af:	and    %rsi,%rcx
   926b2:	movd   %eax,%xmm0
   926b6:	pshuflw $0x0,%xmm0,%xmm0
   926bb:	pshufd $0x44,%xmm0,%xmm0
   926c0:	xor    %edx,%edx
   926c2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   926d0:	movdqu (%rdi,%rdx,2),%xmm1
   926d5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   926db:	psubw  %xmm0,%xmm1
   926df:	psubw  %xmm0,%xmm2
   926e3:	movdqu %xmm1,(%rdi,%rdx,2)
   926e8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   926ee:	add    $0x10,%rdx
   926f2:	cmp    %rdx,%rcx
   926f5:	jne    926d0 <<c30walk::ctor2d::W<false>>::finish+0x480>
   926f7:	cmp    %rcx,%r14
   926fa:	je     9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92700:	testb  $0x18,0x8(%rsp)
   92705:	je     92842 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   9270b:	mov    %rcx,%rdx
   9270e:	add    $0xc,%rsi
   92712:	mov    %rsi,%rcx
   92715:	and    %r14,%rcx
   92718:	movd   %eax,%xmm0
   9271c:	pshuflw $0x0,%xmm0,%xmm0
   92721:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   92730:	movq   (%rdi,%rdx,2),%xmm1
   92735:	psubw  %xmm0,%xmm1
   92739:	movq   %xmm1,(%rdi,%rdx,2)
   9273e:	add    $0x4,%rdx
   92742:	cmp    %rdx,%rcx
   92745:	jne    92730 <<c30walk::ctor2d::W<false>>::finish+0x4e0>
   92747:	jmp    9283d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9274c:	mov    %rdi,0x8(%rsp)
   92751:	mov    %r10,%rdi
   92754:	mov    %rsi,0x20(%rsp)
   92759:	mov    %rcx,0x28(%rsp)
   9275e:	mov    %r9,%r14
   92761:	mov    %r10,0x18(%rsp)
   92766:	call   90220 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9276b:	mov    0x18(%rsp),%r10
   92770:	mov    %r14,%r9
   92773:	mov    0x28(%rsp),%rcx
   92778:	mov    0x20(%rsp),%rsi
   9277d:	mov    0x10(%rsp),%r14
   92782:	mov    0x8(%rsp),%rdi
   92787:	jmp    92502 <<c30walk::ctor2d::W<false>>::finish+0x2b2>
   9278c:	lea    0x9ddbd(%rip),%rdx        # 130550 <__frame_dummy_init_array_entry+0x2630>
   92793:	mov    %rcx,%rdi
   92796:	mov    %r8,%rsi
   92799:	call   d7533 <core::panicking::panic_bounds_check>
   9279e:	lea    0x9deb3(%rip),%rdx        # 130658 <__frame_dummy_init_array_entry+0x2738>
   927a5:	mov    %rbx,%rdi
   927a8:	mov    %r8,%rsi
   927ab:	call   d7533 <core::panicking::panic_bounds_check>
   927b0:	lea    0x9dea1(%rip),%rdx        # 130658 <__frame_dummy_init_array_entry+0x2738>
   927b7:	mov    %r13,%rdi
   927ba:	mov    %r8,%rsi
   927bd:	call   d7533 <core::panicking::panic_bounds_check>
   927c2:	lea    0x9dd6f(%rip),%rdx        # 130538 <__frame_dummy_init_array_entry+0x2618>
   927c9:	mov    $0x20,%esi
   927ce:	mov    %rcx,%rdi
   927d1:	call   d7533 <core::panicking::panic_bounds_check>
   927d6:	lea    0x9dd5b(%rip),%rdx        # 130538 <__frame_dummy_init_array_entry+0x2618>
   927dd:	mov    $0x20,%esi
   927e2:	mov    %rax,%rdi
   927e5:	call   d7533 <core::panicking::panic_bounds_check>
   927ea:	lea    0x9ddd7(%rip),%rdx        # 1305c8 <__frame_dummy_init_array_entry+0x26a8>
   927f1:	mov    %r13,%rdi
   927f4:	xor    %esi,%esi
   927f6:	call   d7533 <core::panicking::panic_bounds_check>
   927fb:	lea    0x9dd1e(%rip),%rdx        # 130520 <__frame_dummy_init_array_entry+0x2600>
   92802:	mov    %rax,%rdi
   92805:	mov    %rcx,%rsi
   92808:	call   d7533 <core::panicking::panic_bounds_check>
   9280d:	lea    0x9dcf4(%rip),%rdx        # 130508 <__frame_dummy_init_array_entry+0x25e8>
   92814:	mov    %rcx,%rdi
   92817:	mov    %r8,%rsi
   9281a:	call   d7533 <core::panicking::panic_bounds_check>
   9281f:	lea    0x9ddba(%rip),%rdx        # 1305e0 <__frame_dummy_init_array_entry+0x26c0>
   92826:	call   d7533 <core::panicking::panic_bounds_check>
   9282b:	lea    0x9dcd6(%rip),%rdx        # 130508 <__frame_dummy_init_array_entry+0x25e8>
   92832:	mov    %rax,%rdi
   92835:	mov    %r8,%rsi
   92838:	call   d7533 <core::panicking::panic_bounds_check>
   9283d:	cmp    %rcx,%r14
   92840:	je     9284b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92842:	sub    %ax,(%rdi,%rcx,2)
   92846:	inc    %rcx
   92849:	jmp    9283d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9284b:	add    $0x38,%rsp
   9284f:	pop    %rbx
   92850:	pop    %r12
   92852:	pop    %r13
   92854:	pop    %r14
   92856:	pop    %r15
   92858:	pop    %rbp
   92859:	ret
