# <c30walk::ctor2d::W<false>>::finish
# size 0x60a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000092240 <<c30walk::ctor2d::W<false>>::finish>:
   92240:	push   %rbp
   92241:	push   %r15
   92243:	push   %r14
   92245:	push   %r13
   92247:	push   %r12
   92249:	push   %rbx
   9224a:	sub    $0x38,%rsp
   9224e:	mov    0x18(%rdi),%rbp
   92252:	mov    0x10(%rbp),%r8
   92256:	mov    %r8,%r13
   92259:	sub    $0x1,%r13
   9225d:	jb     927da <<c30walk::ctor2d::W<false>>::finish+0x59a>
   92263:	mov    %rdi,%r11
   92266:	lea    (%rsi,%rsi,2),%rax
   9226a:	mov    0x1040(%rdi,%rax,8),%rdi
   92272:	mov    0x1048(%r11,%rax,8),%r12d
   9227a:	mov    0x1050(%r11,%rax,8),%ecx
   92282:	mov    0x1054(%r11,%rax,8),%eax
   9228a:	mov    0x40(%r11,%rsi,8),%r14
   9228f:	sub    %rdi,%r14
   92292:	mov    %r14,0x8(%rsp)
   92297:	shr    $1,%r14
   9229a:	mov    0x8(%rbp),%r10
   9229e:	mov    %r13,%rdx
   922a1:	shl    $0xa,%rdx
   922a5:	add    %r10,%rdx
   922a8:	mov    %r13,%rbx
   922ab:	cmp    %r12d,(%rdx,%rsi,4)
   922af:	jbe    92328 <<c30walk::ctor2d::W<false>>::finish+0xe8>
   922b1:	mov    %r11,%r15
   922b4:	lea    (%r12,%r14,1),%edx
   922b8:	dec    %edx
   922ba:	mov    %r8,%r9
   922bd:	shl    $0xa,%r9
   922c1:	lea    (%r9,%rsi,4),%r9
   922c5:	lea    (%r10,%r9,1),%r11
   922c9:	add    $0xfffffffffffffc00,%r11
   922d0:	mov    %r8,%rbx
   922d3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   922e0:	dec    %rbx
   922e3:	cmp    %r8,%rbx
   922e6:	jae    9278e <<c30walk::ctor2d::W<false>>::finish+0x54e>
   922ec:	cmp    %r12d,(%r11)
   922ef:	lea    -0x400(%r11),%r11
   922f6:	ja     922e0 <<c30walk::ctor2d::W<false>>::finish+0xa0>
   922f8:	add    %r10,%r9
   922fb:	add    $0xfffffffffffffc00,%r9
   92302:	mov    %r8,%r13
   92305:	mov    %r15,%r11
   92308:	nopl   0x0(%rax,%rax,1)
   92310:	dec    %r13
   92313:	cmp    %r8,%r13
   92316:	jae    927a0 <<c30walk::ctor2d::W<false>>::finish+0x560>
   9231c:	cmp    %edx,(%r9)
   9231f:	lea    -0x400(%r9),%r9
   92326:	ja     92310 <<c30walk::ctor2d::W<false>>::finish+0xd0>
   92328:	imul   $0xffffffffffffffe2,%rcx,%r15
   9232c:	add    %r12,%r15
   9232f:	cmp    $0xffffffff,%eax
   92332:	mov    %r14,0x10(%rsp)
   92337:	je     9243b <<c30walk::ctor2d::W<false>>::finish+0x1fb>
   9233d:	mov    $0xffffffff,%r9d
   92343:	cmp    %r13,%rbx
   92346:	jae    92360 <<c30walk::ctor2d::W<false>>::finish+0x120>
   92348:	lea    0x1(%rbx),%rcx
   9234c:	cmp    %r8,%rcx
   9234f:	jae    927fd <<c30walk::ctor2d::W<false>>::finish+0x5bd>
   92355:	shl    $0xa,%rcx
   92359:	add    %rcx,%r10
   9235c:	mov    (%r10,%rsi,4),%r9d
   92360:	mov    (%r11),%rdx
   92363:	mov    0x10(%rdx),%rcx
   92367:	cmp    %rax,%rcx
   9236a:	jbe    927eb <<c30walk::ctor2d::W<false>>::finish+0x5ab>
   92370:	test   %r14,%r14
   92373:	je     9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92379:	shl    $0x7,%rax
   9237d:	add    0x8(%rdx),%rax
   92381:	lea    0x800(,%rsi,4),%rdx
   92389:	xor    %r11d,%r11d
   9238c:	nopl   0x0(%rax)
   92390:	lea    (%r12,%r11,1),%r10d
   92394:	cmp    %r9d,%r10d
   92397:	jae    923d0 <<c30walk::ctor2d::W<false>>::finish+0x190>
   92399:	lea    (%r11,%r15,1),%rcx
   9239d:	cmp    $0x20,%rcx
   923a1:	jae    927b2 <<c30walk::ctor2d::W<false>>::finish+0x572>
   923a7:	movzwl (%rdi,%r11,2),%r8d
   923ac:	inc    %r11
   923af:	mov    %ebx,%r10d
   923b2:	shl    $0x18,%r10d
   923b6:	shl    $0x8,%r8d
   923ba:	or     %r10d,%r8d
   923bd:	or     %esi,%r8d
   923c0:	mov    %r8d,(%rax,%rcx,4)
   923c4:	cmp    0x10(%rsp),%r11
   923c9:	jne    92390 <<c30walk::ctor2d::W<false>>::finish+0x150>
   923cb:	jmp    92419 <<c30walk::ctor2d::W<false>>::finish+0x1d9>
   923cd:	nopl   (%rax)
   923d0:	mov    %rbx,%r14
   923d3:	shl    $0xa,%r14
   923d7:	add    %rdx,%r14
   923da:	mov    %rbx,%rcx
   923dd:	jmp    923ef <<c30walk::ctor2d::W<false>>::finish+0x1af>
   923df:	nop
   923e0:	add    $0x400,%r14
   923e7:	mov    %rbx,%rcx
   923ea:	cmp    %r9d,%r10d
   923ed:	jb     92399 <<c30walk::ctor2d::W<false>>::finish+0x159>
   923ef:	lea    0x1(%rcx),%rbx
   923f3:	mov    $0xffffffff,%r9d
   923f9:	cmp    %r13,%rbx
   923fc:	jae    923e0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   923fe:	add    $0x2,%rcx
   92402:	mov    0x10(%rbp),%r8
   92406:	cmp    %r8,%rcx
   92409:	jae    9277c <<c30walk::ctor2d::W<false>>::finish+0x53c>
   9240f:	mov    0x8(%rbp),%rcx
   92413:	mov    (%rcx,%r14,1),%r9d
   92417:	jmp    923e0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   92419:	mov    0x8(%rsp),%rdx
   9241e:	and    $0xfffffffffffffffe,%rdx
   92422:	mov    $0xff,%esi
   92427:	add    $0x38,%rsp
   9242b:	pop    %rbx
   9242c:	pop    %r12
   9242e:	pop    %r13
   92430:	pop    %r14
   92432:	pop    %r15
   92434:	pop    %rbp
   92435:	jmp    *0xa1d8d(%rip)        # 1341c8 <memset@GLIBC_2.2.5>
   9243b:	movzwl (%rdi),%eax
   9243e:	movzwl -0x2(%rdi,%r14,2),%r8d
   92444:	mov    %ebx,%edx
   92446:	shl    $0x10,%edx
   92449:	or     %eax,%edx
   9244b:	mov    %r13d,%r9d
   9244e:	shl    $0x10,%r9d
   92452:	or     %r8d,%r9d
   92455:	sub    %edx,%r9d
   92458:	cmp    $0xfffe,%r9d
   9245f:	jae    924dd <<c30walk::ctor2d::W<false>>::finish+0x29d>
   92461:	mov    0x28(%r11),%r8
   92465:	shl    $0x6,%rcx
   92469:	shl    $0x8,%edx
   9246c:	or     %edx,%esi
   9246e:	mov    %esi,0x3c(%r8,%rcx,1)
   92473:	test   %r14,%r14
   92476:	je     9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   9247c:	cmp    $0x1e,%r14
   92480:	jne    92676 <<c30walk::ctor2d::W<false>>::finish+0x436>
   92486:	movd   %eax,%xmm0
   9248a:	pshuflw $0x0,%xmm0,%xmm0
   9248f:	pshufd $0x44,%xmm0,%xmm1
   92494:	movdqu (%rdi),%xmm2
   92498:	movdqu 0x10(%rdi),%xmm3
   9249d:	movdqu 0x20(%rdi),%xmm4
   924a2:	psubw  %xmm1,%xmm2
   924a6:	movdqu %xmm2,(%rdi)
   924aa:	psubw  %xmm1,%xmm3
   924ae:	movdqu %xmm3,0x10(%rdi)
   924b3:	psubw  %xmm1,%xmm4
   924b7:	movdqu %xmm4,0x20(%rdi)
   924bc:	movq   0x30(%rdi),%xmm1
   924c1:	psubw  %xmm0,%xmm1
   924c5:	movq   %xmm1,0x30(%rdi)
   924ca:	movd   0x38(%rdi),%xmm1
   924cf:	psubw  %xmm0,%xmm1
   924d3:	movd   %xmm1,0x38(%rdi)
   924d8:	jmp    9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   924dd:	mov    (%r11),%r10
   924e0:	mov    0x10(%r10),%r9
   924e4:	cmp    (%r10),%r9
   924e7:	mov    %r11,0x30(%rsp)
   924ec:	je     9273c <<c30walk::ctor2d::W<false>>::finish+0x4fc>
   924f2:	mov    0x8(%r10),%rax
   924f6:	mov    %r9,%rdx
   924f9:	shl    $0x7,%rdx
   924fd:	pxor   %xmm0,%xmm0
   92501:	movdqa %xmm0,0x70(%rax,%rdx,1)
   92507:	movdqa %xmm0,0x60(%rax,%rdx,1)
   9250d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   92513:	movdqa %xmm0,0x40(%rax,%rdx,1)
   92519:	movdqa %xmm0,0x30(%rax,%rdx,1)
   9251f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   92525:	movdqa %xmm0,0x10(%rax,%rdx,1)
   9252b:	movdqa %xmm0,(%rax,%rdx,1)
   92530:	lea    0x1(%r9),%rax
   92534:	mov    %rax,0x10(%r10)
   92538:	mov    $0xffffffff,%r8d
   9253e:	cmp    %r13,%rbx
   92541:	jae    92560 <<c30walk::ctor2d::W<false>>::finish+0x320>
   92543:	lea    0x1(%rbx),%rax
   92547:	mov    0x10(%rbp),%r8
   9254b:	cmp    %r8,%rax
   9254e:	jae    9281b <<c30walk::ctor2d::W<false>>::finish+0x5db>
   92554:	shl    $0xa,%rax
   92558:	add    0x8(%rbp),%rax
   9255c:	mov    (%rax,%rsi,4),%r8d
   92560:	mov    %r9,0x8(%rsp)
   92565:	test   %r14,%r14
   92568:	je     92615 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   9256e:	add    0x8(%r10),%rdx
   92572:	lea    0x800(,%rsi,4),%r9
   9257a:	xor    %r10d,%r10d
   9257d:	nopl   (%rax)
   92580:	lea    (%r12,%r10,1),%r11d
   92584:	cmp    %r8d,%r11d
   92587:	jae    925c0 <<c30walk::ctor2d::W<false>>::finish+0x380>
   92589:	lea    (%r10,%r15,1),%rax
   9258d:	cmp    $0x20,%rax
   92591:	jae    927c6 <<c30walk::ctor2d::W<false>>::finish+0x586>
   92597:	movzwl (%rdi,%r10,2),%r11d
   9259c:	inc    %r10
   9259f:	mov    %ebx,%r14d
   925a2:	shl    $0x18,%r14d
   925a6:	shl    $0x8,%r11d
   925aa:	or     %r14d,%r11d
   925ad:	or     %esi,%r11d
   925b0:	mov    %r11d,(%rdx,%rax,4)
   925b4:	cmp    0x10(%rsp),%r10
   925b9:	jne    92580 <<c30walk::ctor2d::W<false>>::finish+0x340>
   925bb:	jmp    92615 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   925bd:	nopl   (%rax)
   925c0:	mov    %rbx,%r14
   925c3:	shl    $0xa,%r14
   925c7:	add    %r9,%r14
   925ca:	mov    %rbx,%rax
   925cd:	jmp    925e7 <<c30walk::ctor2d::W<false>>::finish+0x3a7>
   925cf:	nop
   925d0:	mov    0x8(%rbp),%rax
   925d4:	mov    (%rax,%r14,1),%r8d
   925d8:	add    $0x400,%r14
   925df:	mov    %rbx,%rax
   925e2:	cmp    %r8d,%r11d
   925e5:	jb     92589 <<c30walk::ctor2d::W<false>>::finish+0x349>
   925e7:	lea    0x1(%rax),%rbx
   925eb:	mov    $0xffffffff,%r8d
   925f1:	cmp    %r13,%rbx
   925f4:	jae    925d8 <<c30walk::ctor2d::W<false>>::finish+0x398>
   925f6:	add    $0x2,%rax
   925fa:	mov    0x10(%rbp),%r8
   925fe:	cmp    %r8,%rax
   92601:	jb     925d0 <<c30walk::ctor2d::W<false>>::finish+0x390>
   92603:	lea    0x9df36(%rip),%rdx        # 130540 <__frame_dummy_init_array_entry+0x2630>
   9260a:	mov    %rax,%rdi
   9260d:	mov    %r8,%rsi
   92610:	call   d7523 <core::panicking::panic_bounds_check>
   92615:	mov    0x30(%rsp),%r8
   9261a:	mov    0x28(%r8),%rax
   9261e:	mov    %rcx,%rdx
   92621:	shl    $0x6,%rdx
   92625:	mov    0x8(%rsp),%rsi
   9262a:	or     $0x10000000,%esi
   92630:	pcmpeqd %xmm0,%xmm0
   92634:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9263a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   92640:	movdqa %xmm0,0x10(%rax,%rdx,1)
   92646:	movdqa %xmm0,(%rax,%rdx,1)
   9264b:	mov    %esi,0x3c(%rax,%rdx,1)
   9264f:	mov    %ecx,%edi
   92651:	shr    $0x6,%edi
   92654:	mov    0x10(%r8),%rsi
   92658:	cmp    %rsi,%rdi
   9265b:	jae    9280f <<c30walk::ctor2d::W<false>>::finish+0x5cf>
   92661:	mov    $0x1,%eax
   92666:	shl    %cl,%rax
   92669:	mov    0x8(%r8),%rcx
   9266d:	or     %rax,(%rcx,%rdi,8)
   92671:	jmp    9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92676:	mov    0x8(%rsp),%rcx
   9267b:	cmp    $0x8,%rcx
   9267f:	jae    92688 <<c30walk::ctor2d::W<false>>::finish+0x448>
   92681:	xor    %ecx,%ecx
   92683:	jmp    92832 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   92688:	movabs $0x7ffffffffffffff0,%rsi
   92692:	cmp    $0x20,%rcx
   92696:	jae    9269c <<c30walk::ctor2d::W<false>>::finish+0x45c>
   92698:	xor    %ecx,%ecx
   9269a:	jmp    926fb <<c30walk::ctor2d::W<false>>::finish+0x4bb>
   9269c:	mov    %r14,%rcx
   9269f:	and    %rsi,%rcx
   926a2:	movd   %eax,%xmm0
   926a6:	pshuflw $0x0,%xmm0,%xmm0
   926ab:	pshufd $0x44,%xmm0,%xmm0
   926b0:	xor    %edx,%edx
   926b2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   926c0:	movdqu (%rdi,%rdx,2),%xmm1
   926c5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   926cb:	psubw  %xmm0,%xmm1
   926cf:	psubw  %xmm0,%xmm2
   926d3:	movdqu %xmm1,(%rdi,%rdx,2)
   926d8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   926de:	add    $0x10,%rdx
   926e2:	cmp    %rdx,%rcx
   926e5:	jne    926c0 <<c30walk::ctor2d::W<false>>::finish+0x480>
   926e7:	cmp    %rcx,%r14
   926ea:	je     9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   926f0:	testb  $0x18,0x8(%rsp)
   926f5:	je     92832 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   926fb:	mov    %rcx,%rdx
   926fe:	add    $0xc,%rsi
   92702:	mov    %rsi,%rcx
   92705:	and    %r14,%rcx
   92708:	movd   %eax,%xmm0
   9270c:	pshuflw $0x0,%xmm0,%xmm0
   92711:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   92720:	movq   (%rdi,%rdx,2),%xmm1
   92725:	psubw  %xmm0,%xmm1
   92729:	movq   %xmm1,(%rdi,%rdx,2)
   9272e:	add    $0x4,%rdx
   92732:	cmp    %rdx,%rcx
   92735:	jne    92720 <<c30walk::ctor2d::W<false>>::finish+0x4e0>
   92737:	jmp    9282d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9273c:	mov    %rdi,0x8(%rsp)
   92741:	mov    %r10,%rdi
   92744:	mov    %rsi,0x20(%rsp)
   92749:	mov    %rcx,0x28(%rsp)
   9274e:	mov    %r9,%r14
   92751:	mov    %r10,0x18(%rsp)
   92756:	call   90210 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9275b:	mov    0x18(%rsp),%r10
   92760:	mov    %r14,%r9
   92763:	mov    0x28(%rsp),%rcx
   92768:	mov    0x20(%rsp),%rsi
   9276d:	mov    0x10(%rsp),%r14
   92772:	mov    0x8(%rsp),%rdi
   92777:	jmp    924f2 <<c30walk::ctor2d::W<false>>::finish+0x2b2>
   9277c:	lea    0x9ddbd(%rip),%rdx        # 130540 <__frame_dummy_init_array_entry+0x2630>
   92783:	mov    %rcx,%rdi
   92786:	mov    %r8,%rsi
   92789:	call   d7523 <core::panicking::panic_bounds_check>
   9278e:	lea    0x9deb3(%rip),%rdx        # 130648 <__frame_dummy_init_array_entry+0x2738>
   92795:	mov    %rbx,%rdi
   92798:	mov    %r8,%rsi
   9279b:	call   d7523 <core::panicking::panic_bounds_check>
   927a0:	lea    0x9dea1(%rip),%rdx        # 130648 <__frame_dummy_init_array_entry+0x2738>
   927a7:	mov    %r13,%rdi
   927aa:	mov    %r8,%rsi
   927ad:	call   d7523 <core::panicking::panic_bounds_check>
   927b2:	lea    0x9dd6f(%rip),%rdx        # 130528 <__frame_dummy_init_array_entry+0x2618>
   927b9:	mov    $0x20,%esi
   927be:	mov    %rcx,%rdi
   927c1:	call   d7523 <core::panicking::panic_bounds_check>
   927c6:	lea    0x9dd5b(%rip),%rdx        # 130528 <__frame_dummy_init_array_entry+0x2618>
   927cd:	mov    $0x20,%esi
   927d2:	mov    %rax,%rdi
   927d5:	call   d7523 <core::panicking::panic_bounds_check>
   927da:	lea    0x9ddd7(%rip),%rdx        # 1305b8 <__frame_dummy_init_array_entry+0x26a8>
   927e1:	mov    %r13,%rdi
   927e4:	xor    %esi,%esi
   927e6:	call   d7523 <core::panicking::panic_bounds_check>
   927eb:	lea    0x9dd1e(%rip),%rdx        # 130510 <__frame_dummy_init_array_entry+0x2600>
   927f2:	mov    %rax,%rdi
   927f5:	mov    %rcx,%rsi
   927f8:	call   d7523 <core::panicking::panic_bounds_check>
   927fd:	lea    0x9dcf4(%rip),%rdx        # 1304f8 <__frame_dummy_init_array_entry+0x25e8>
   92804:	mov    %rcx,%rdi
   92807:	mov    %r8,%rsi
   9280a:	call   d7523 <core::panicking::panic_bounds_check>
   9280f:	lea    0x9ddba(%rip),%rdx        # 1305d0 <__frame_dummy_init_array_entry+0x26c0>
   92816:	call   d7523 <core::panicking::panic_bounds_check>
   9281b:	lea    0x9dcd6(%rip),%rdx        # 1304f8 <__frame_dummy_init_array_entry+0x25e8>
   92822:	mov    %rax,%rdi
   92825:	mov    %r8,%rsi
   92828:	call   d7523 <core::panicking::panic_bounds_check>
   9282d:	cmp    %rcx,%r14
   92830:	je     9283b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   92832:	sub    %ax,(%rdi,%rcx,2)
   92836:	inc    %rcx
   92839:	jmp    9282d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9283b:	add    $0x38,%rsp
   9283f:	pop    %rbx
   92840:	pop    %r12
   92842:	pop    %r13
   92844:	pop    %r14
   92846:	pop    %r15
   92848:	pop    %rbp
   92849:	ret
