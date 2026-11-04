# <c30walk::ctor2::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008e2f0 <<c30walk::ctor2::W<false>>::finish>:
   8e2f0:	push   %rbp
   8e2f1:	push   %r15
   8e2f3:	push   %r14
   8e2f5:	push   %r13
   8e2f7:	push   %r12
   8e2f9:	push   %rbx
   8e2fa:	sub    $0x38,%rsp
   8e2fe:	mov    0x18(%rdi),%rbp
   8e302:	mov    0x10(%rbp),%r8
   8e306:	mov    %r8,%r13
   8e309:	sub    $0x1,%r13
   8e30d:	jb     8e87a <<c30walk::ctor2::W<false>>::finish+0x58a>
   8e313:	mov    %rdi,%r11
   8e316:	lea    (%rsi,%rsi,4),%rax
   8e31a:	mov    0x40(%rdi,%rax,8),%r14
   8e31f:	mov    0x50(%rdi,%rax,8),%rdi
   8e324:	mov    0x58(%r11,%rax,8),%r12d
   8e329:	mov    0x60(%r11,%rax,8),%ecx
   8e32e:	mov    0x64(%r11,%rax,8),%eax
   8e333:	sub    %rdi,%r14
   8e336:	mov    %r14,0x8(%rsp)
   8e33b:	shr    $1,%r14
   8e33e:	mov    0x8(%rbp),%r10
   8e342:	mov    %r13,%rdx
   8e345:	shl    $0xa,%rdx
   8e349:	add    %r10,%rdx
   8e34c:	mov    %r13,%rbx
   8e34f:	cmp    %r12d,(%rdx,%rsi,4)
   8e353:	jbe    8e3c8 <<c30walk::ctor2::W<false>>::finish+0xd8>
   8e355:	mov    %r11,%r15
   8e358:	lea    (%r12,%r14,1),%edx
   8e35c:	dec    %edx
   8e35e:	mov    %r8,%r9
   8e361:	shl    $0xa,%r9
   8e365:	lea    (%r9,%rsi,4),%r9
   8e369:	lea    (%r10,%r9,1),%r11
   8e36d:	add    $0xfffffffffffffc00,%r11
   8e374:	mov    %r8,%rbx
   8e377:	nopw   0x0(%rax,%rax,1)
   8e380:	dec    %rbx
   8e383:	cmp    %r8,%rbx
   8e386:	jae    8e82e <<c30walk::ctor2::W<false>>::finish+0x53e>
   8e38c:	cmp    %r12d,(%r11)
   8e38f:	lea    -0x400(%r11),%r11
   8e396:	ja     8e380 <<c30walk::ctor2::W<false>>::finish+0x90>
   8e398:	add    %r10,%r9
   8e39b:	add    $0xfffffffffffffc00,%r9
   8e3a2:	mov    %r8,%r13
   8e3a5:	mov    %r15,%r11
   8e3a8:	nopl   0x0(%rax,%rax,1)
   8e3b0:	dec    %r13
   8e3b3:	cmp    %r8,%r13
   8e3b6:	jae    8e840 <<c30walk::ctor2::W<false>>::finish+0x550>
   8e3bc:	cmp    %edx,(%r9)
   8e3bf:	lea    -0x400(%r9),%r9
   8e3c6:	ja     8e3b0 <<c30walk::ctor2::W<false>>::finish+0xc0>
   8e3c8:	imul   $0xffffffffffffffe2,%rcx,%r15
   8e3cc:	add    %r12,%r15
   8e3cf:	cmp    $0xffffffff,%eax
   8e3d2:	mov    %r14,0x10(%rsp)
   8e3d7:	je     8e4db <<c30walk::ctor2::W<false>>::finish+0x1eb>
   8e3dd:	mov    $0xffffffff,%r9d
   8e3e3:	cmp    %r13,%rbx
   8e3e6:	jae    8e400 <<c30walk::ctor2::W<false>>::finish+0x110>
   8e3e8:	lea    0x1(%rbx),%rcx
   8e3ec:	cmp    %r8,%rcx
   8e3ef:	jae    8e89d <<c30walk::ctor2::W<false>>::finish+0x5ad>
   8e3f5:	shl    $0xa,%rcx
   8e3f9:	add    %rcx,%r10
   8e3fc:	mov    (%r10,%rsi,4),%r9d
   8e400:	mov    (%r11),%rdx
   8e403:	mov    0x10(%rdx),%rcx
   8e407:	cmp    %rax,%rcx
   8e40a:	jbe    8e88b <<c30walk::ctor2::W<false>>::finish+0x59b>
   8e410:	test   %r14,%r14
   8e413:	je     8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e419:	shl    $0x7,%rax
   8e41d:	add    0x8(%rdx),%rax
   8e421:	lea    0x800(,%rsi,4),%rdx
   8e429:	xor    %r11d,%r11d
   8e42c:	nopl   0x0(%rax)
   8e430:	lea    (%r12,%r11,1),%r10d
   8e434:	cmp    %r9d,%r10d
   8e437:	jae    8e470 <<c30walk::ctor2::W<false>>::finish+0x180>
   8e439:	lea    (%r11,%r15,1),%rcx
   8e43d:	cmp    $0x20,%rcx
   8e441:	jae    8e852 <<c30walk::ctor2::W<false>>::finish+0x562>
   8e447:	movzwl (%rdi,%r11,2),%r8d
   8e44c:	inc    %r11
   8e44f:	mov    %ebx,%r10d
   8e452:	shl    $0x18,%r10d
   8e456:	shl    $0x8,%r8d
   8e45a:	or     %r10d,%r8d
   8e45d:	or     %esi,%r8d
   8e460:	mov    %r8d,(%rax,%rcx,4)
   8e464:	cmp    0x10(%rsp),%r11
   8e469:	jne    8e430 <<c30walk::ctor2::W<false>>::finish+0x140>
   8e46b:	jmp    8e4b9 <<c30walk::ctor2::W<false>>::finish+0x1c9>
   8e46d:	nopl   (%rax)
   8e470:	mov    %rbx,%r14
   8e473:	shl    $0xa,%r14
   8e477:	add    %rdx,%r14
   8e47a:	mov    %rbx,%rcx
   8e47d:	jmp    8e48f <<c30walk::ctor2::W<false>>::finish+0x19f>
   8e47f:	nop
   8e480:	add    $0x400,%r14
   8e487:	mov    %rbx,%rcx
   8e48a:	cmp    %r9d,%r10d
   8e48d:	jb     8e439 <<c30walk::ctor2::W<false>>::finish+0x149>
   8e48f:	lea    0x1(%rcx),%rbx
   8e493:	mov    $0xffffffff,%r9d
   8e499:	cmp    %r13,%rbx
   8e49c:	jae    8e480 <<c30walk::ctor2::W<false>>::finish+0x190>
   8e49e:	add    $0x2,%rcx
   8e4a2:	mov    0x10(%rbp),%r8
   8e4a6:	cmp    %r8,%rcx
   8e4a9:	jae    8e81c <<c30walk::ctor2::W<false>>::finish+0x52c>
   8e4af:	mov    0x8(%rbp),%rcx
   8e4b3:	mov    (%rcx,%r14,1),%r9d
   8e4b7:	jmp    8e480 <<c30walk::ctor2::W<false>>::finish+0x190>
   8e4b9:	mov    0x8(%rsp),%rdx
   8e4be:	and    $0xfffffffffffffffe,%rdx
   8e4c2:	mov    $0xff,%esi
   8e4c7:	add    $0x38,%rsp
   8e4cb:	pop    %rbx
   8e4cc:	pop    %r12
   8e4ce:	pop    %r13
   8e4d0:	pop    %r14
   8e4d2:	pop    %r15
   8e4d4:	pop    %rbp
   8e4d5:	jmp    *0xb5a65(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   8e4db:	movzwl (%rdi),%eax
   8e4de:	movzwl -0x2(%rdi,%r14,2),%r8d
   8e4e4:	mov    %ebx,%edx
   8e4e6:	shl    $0x10,%edx
   8e4e9:	or     %eax,%edx
   8e4eb:	mov    %r13d,%r9d
   8e4ee:	shl    $0x10,%r9d
   8e4f2:	or     %r8d,%r9d
   8e4f5:	sub    %edx,%r9d
   8e4f8:	cmp    $0xfffe,%r9d
   8e4ff:	jae    8e57d <<c30walk::ctor2::W<false>>::finish+0x28d>
   8e501:	mov    0x28(%r11),%r8
   8e505:	shl    $0x6,%rcx
   8e509:	shl    $0x8,%edx
   8e50c:	or     %edx,%esi
   8e50e:	mov    %esi,0x3c(%r8,%rcx,1)
   8e513:	test   %r14,%r14
   8e516:	je     8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e51c:	cmp    $0x1e,%r14
   8e520:	jne    8e716 <<c30walk::ctor2::W<false>>::finish+0x426>
   8e526:	movd   %eax,%xmm0
   8e52a:	pshuflw $0x0,%xmm0,%xmm0
   8e52f:	pshufd $0x44,%xmm0,%xmm1
   8e534:	movdqu (%rdi),%xmm2
   8e538:	movdqu 0x10(%rdi),%xmm3
   8e53d:	movdqu 0x20(%rdi),%xmm4
   8e542:	psubw  %xmm1,%xmm2
   8e546:	movdqu %xmm2,(%rdi)
   8e54a:	psubw  %xmm1,%xmm3
   8e54e:	movdqu %xmm3,0x10(%rdi)
   8e553:	psubw  %xmm1,%xmm4
   8e557:	movdqu %xmm4,0x20(%rdi)
   8e55c:	movq   0x30(%rdi),%xmm1
   8e561:	psubw  %xmm0,%xmm1
   8e565:	movq   %xmm1,0x30(%rdi)
   8e56a:	movd   0x38(%rdi),%xmm1
   8e56f:	psubw  %xmm0,%xmm1
   8e573:	movd   %xmm1,0x38(%rdi)
   8e578:	jmp    8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e57d:	mov    (%r11),%r10
   8e580:	mov    0x10(%r10),%r9
   8e584:	cmp    (%r10),%r9
   8e587:	mov    %r11,0x30(%rsp)
   8e58c:	je     8e7dc <<c30walk::ctor2::W<false>>::finish+0x4ec>
   8e592:	mov    0x8(%r10),%rax
   8e596:	mov    %r9,%rdx
   8e599:	shl    $0x7,%rdx
   8e59d:	pxor   %xmm0,%xmm0
   8e5a1:	movdqa %xmm0,0x70(%rax,%rdx,1)
   8e5a7:	movdqa %xmm0,0x60(%rax,%rdx,1)
   8e5ad:	movdqa %xmm0,0x50(%rax,%rdx,1)
   8e5b3:	movdqa %xmm0,0x40(%rax,%rdx,1)
   8e5b9:	movdqa %xmm0,0x30(%rax,%rdx,1)
   8e5bf:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8e5c5:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8e5cb:	movdqa %xmm0,(%rax,%rdx,1)
   8e5d0:	lea    0x1(%r9),%rax
   8e5d4:	mov    %rax,0x10(%r10)
   8e5d8:	mov    $0xffffffff,%r8d
   8e5de:	cmp    %r13,%rbx
   8e5e1:	jae    8e600 <<c30walk::ctor2::W<false>>::finish+0x310>
   8e5e3:	lea    0x1(%rbx),%rax
   8e5e7:	mov    0x10(%rbp),%r8
   8e5eb:	cmp    %r8,%rax
   8e5ee:	jae    8e8bb <<c30walk::ctor2::W<false>>::finish+0x5cb>
   8e5f4:	shl    $0xa,%rax
   8e5f8:	add    0x8(%rbp),%rax
   8e5fc:	mov    (%rax,%rsi,4),%r8d
   8e600:	mov    %r9,0x8(%rsp)
   8e605:	test   %r14,%r14
   8e608:	je     8e6b5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
   8e60e:	add    0x8(%r10),%rdx
   8e612:	lea    0x800(,%rsi,4),%r9
   8e61a:	xor    %r10d,%r10d
   8e61d:	nopl   (%rax)
   8e620:	lea    (%r12,%r10,1),%r11d
   8e624:	cmp    %r8d,%r11d
   8e627:	jae    8e660 <<c30walk::ctor2::W<false>>::finish+0x370>
   8e629:	lea    (%r10,%r15,1),%rax
   8e62d:	cmp    $0x20,%rax
   8e631:	jae    8e866 <<c30walk::ctor2::W<false>>::finish+0x576>
   8e637:	movzwl (%rdi,%r10,2),%r11d
   8e63c:	inc    %r10
   8e63f:	mov    %ebx,%r14d
   8e642:	shl    $0x18,%r14d
   8e646:	shl    $0x8,%r11d
   8e64a:	or     %r14d,%r11d
   8e64d:	or     %esi,%r11d
   8e650:	mov    %r11d,(%rdx,%rax,4)
   8e654:	cmp    0x10(%rsp),%r10
   8e659:	jne    8e620 <<c30walk::ctor2::W<false>>::finish+0x330>
   8e65b:	jmp    8e6b5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
   8e65d:	nopl   (%rax)
   8e660:	mov    %rbx,%r14
   8e663:	shl    $0xa,%r14
   8e667:	add    %r9,%r14
   8e66a:	mov    %rbx,%rax
   8e66d:	jmp    8e687 <<c30walk::ctor2::W<false>>::finish+0x397>
   8e66f:	nop
   8e670:	mov    0x8(%rbp),%rax
   8e674:	mov    (%rax,%r14,1),%r8d
   8e678:	add    $0x400,%r14
   8e67f:	mov    %rbx,%rax
   8e682:	cmp    %r8d,%r11d
   8e685:	jb     8e629 <<c30walk::ctor2::W<false>>::finish+0x339>
   8e687:	lea    0x1(%rax),%rbx
   8e68b:	mov    $0xffffffff,%r8d
   8e691:	cmp    %r13,%rbx
   8e694:	jae    8e678 <<c30walk::ctor2::W<false>>::finish+0x388>
   8e696:	add    $0x2,%rax
   8e69a:	mov    0x10(%rbp),%r8
   8e69e:	cmp    %r8,%rax
   8e6a1:	jb     8e670 <<c30walk::ctor2::W<false>>::finish+0x380>
   8e6a3:	lea    0xb0ce6(%rip),%rdx        # 13f390 <__frame_dummy_init_array_entry+0x1eb0>
   8e6aa:	mov    %rax,%rdi
   8e6ad:	mov    %r8,%rsi
   8e6b0:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e6b5:	mov    0x30(%rsp),%r8
   8e6ba:	mov    0x28(%r8),%rax
   8e6be:	mov    %rcx,%rdx
   8e6c1:	shl    $0x6,%rdx
   8e6c5:	mov    0x8(%rsp),%rsi
   8e6ca:	or     $0x10000000,%esi
   8e6d0:	pcmpeqd %xmm0,%xmm0
   8e6d4:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   8e6da:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8e6e0:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8e6e6:	movdqa %xmm0,(%rax,%rdx,1)
   8e6eb:	mov    %esi,0x3c(%rax,%rdx,1)
   8e6ef:	mov    %ecx,%edi
   8e6f1:	shr    $0x6,%edi
   8e6f4:	mov    0x10(%r8),%rsi
   8e6f8:	cmp    %rsi,%rdi
   8e6fb:	jae    8e8af <<c30walk::ctor2::W<false>>::finish+0x5bf>
   8e701:	mov    $0x1,%eax
   8e706:	shl    %cl,%rax
   8e709:	mov    0x8(%r8),%rcx
   8e70d:	or     %rax,(%rcx,%rdi,8)
   8e711:	jmp    8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e716:	mov    0x8(%rsp),%rcx
   8e71b:	cmp    $0x8,%rcx
   8e71f:	jae    8e728 <<c30walk::ctor2::W<false>>::finish+0x438>
   8e721:	xor    %ecx,%ecx
   8e723:	jmp    8e8d2 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   8e728:	movabs $0x7ffffffffffffff0,%rsi
   8e732:	cmp    $0x20,%rcx
   8e736:	jae    8e73c <<c30walk::ctor2::W<false>>::finish+0x44c>
   8e738:	xor    %ecx,%ecx
   8e73a:	jmp    8e79b <<c30walk::ctor2::W<false>>::finish+0x4ab>
   8e73c:	mov    %r14,%rcx
   8e73f:	and    %rsi,%rcx
   8e742:	movd   %eax,%xmm0
   8e746:	pshuflw $0x0,%xmm0,%xmm0
   8e74b:	pshufd $0x44,%xmm0,%xmm0
   8e750:	xor    %edx,%edx
   8e752:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8e760:	movdqu (%rdi,%rdx,2),%xmm1
   8e765:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   8e76b:	psubw  %xmm0,%xmm1
   8e76f:	psubw  %xmm0,%xmm2
   8e773:	movdqu %xmm1,(%rdi,%rdx,2)
   8e778:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   8e77e:	add    $0x10,%rdx
   8e782:	cmp    %rdx,%rcx
   8e785:	jne    8e760 <<c30walk::ctor2::W<false>>::finish+0x470>
   8e787:	cmp    %rcx,%r14
   8e78a:	je     8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e790:	testb  $0x18,0x8(%rsp)
   8e795:	je     8e8d2 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   8e79b:	mov    %rcx,%rdx
   8e79e:	add    $0xc,%rsi
   8e7a2:	mov    %rsi,%rcx
   8e7a5:	and    %r14,%rcx
   8e7a8:	movd   %eax,%xmm0
   8e7ac:	pshuflw $0x0,%xmm0,%xmm0
   8e7b1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8e7c0:	movq   (%rdi,%rdx,2),%xmm1
   8e7c5:	psubw  %xmm0,%xmm1
   8e7c9:	movq   %xmm1,(%rdi,%rdx,2)
   8e7ce:	add    $0x4,%rdx
   8e7d2:	cmp    %rdx,%rcx
   8e7d5:	jne    8e7c0 <<c30walk::ctor2::W<false>>::finish+0x4d0>
   8e7d7:	jmp    8e8cd <<c30walk::ctor2::W<false>>::finish+0x5dd>
   8e7dc:	mov    %rdi,0x8(%rsp)
   8e7e1:	mov    %r10,%rdi
   8e7e4:	mov    %rsi,0x20(%rsp)
   8e7e9:	mov    %rcx,0x28(%rsp)
   8e7ee:	mov    %r9,%r14
   8e7f1:	mov    %r10,0x18(%rsp)
   8e7f6:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   8e7fb:	mov    0x18(%rsp),%r10
   8e800:	mov    %r14,%r9
   8e803:	mov    0x28(%rsp),%rcx
   8e808:	mov    0x20(%rsp),%rsi
   8e80d:	mov    0x10(%rsp),%r14
   8e812:	mov    0x8(%rsp),%rdi
   8e817:	jmp    8e592 <<c30walk::ctor2::W<false>>::finish+0x2a2>
   8e81c:	lea    0xb0b6d(%rip),%rdx        # 13f390 <__frame_dummy_init_array_entry+0x1eb0>
   8e823:	mov    %rcx,%rdi
   8e826:	mov    %r8,%rsi
   8e829:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e82e:	lea    0xb0c63(%rip),%rdx        # 13f498 <__frame_dummy_init_array_entry+0x1fb8>
   8e835:	mov    %rbx,%rdi
   8e838:	mov    %r8,%rsi
   8e83b:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e840:	lea    0xb0c51(%rip),%rdx        # 13f498 <__frame_dummy_init_array_entry+0x1fb8>
   8e847:	mov    %r13,%rdi
   8e84a:	mov    %r8,%rsi
   8e84d:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e852:	lea    0xb0b1f(%rip),%rdx        # 13f378 <__frame_dummy_init_array_entry+0x1e98>
   8e859:	mov    $0x20,%esi
   8e85e:	mov    %rcx,%rdi
   8e861:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e866:	lea    0xb0b0b(%rip),%rdx        # 13f378 <__frame_dummy_init_array_entry+0x1e98>
   8e86d:	mov    $0x20,%esi
   8e872:	mov    %rax,%rdi
   8e875:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e87a:	lea    0xb0b87(%rip),%rdx        # 13f408 <__frame_dummy_init_array_entry+0x1f28>
   8e881:	mov    %r13,%rdi
   8e884:	xor    %esi,%esi
   8e886:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e88b:	lea    0xb0ace(%rip),%rdx        # 13f360 <__frame_dummy_init_array_entry+0x1e80>
   8e892:	mov    %rax,%rdi
   8e895:	mov    %rcx,%rsi
   8e898:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e89d:	lea    0xb0aa4(%rip),%rdx        # 13f348 <__frame_dummy_init_array_entry+0x1e68>
   8e8a4:	mov    %rcx,%rdi
   8e8a7:	mov    %r8,%rsi
   8e8aa:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e8af:	lea    0xb0b6a(%rip),%rdx        # 13f420 <__frame_dummy_init_array_entry+0x1f40>
   8e8b6:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e8bb:	lea    0xb0a86(%rip),%rdx        # 13f348 <__frame_dummy_init_array_entry+0x1e68>
   8e8c2:	mov    %rax,%rdi
   8e8c5:	mov    %r8,%rsi
   8e8c8:	call   e6ac3 <core::panicking::panic_bounds_check>
   8e8cd:	cmp    %rcx,%r14
   8e8d0:	je     8e8db <<c30walk::ctor2::W<false>>::finish+0x5eb>
   8e8d2:	sub    %ax,(%rdi,%rcx,2)
   8e8d6:	inc    %rcx
   8e8d9:	jmp    8e8cd <<c30walk::ctor2::W<false>>::finish+0x5dd>
   8e8db:	add    $0x38,%rsp
   8e8df:	pop    %rbx
   8e8e0:	pop    %r12
   8e8e2:	pop    %r13
   8e8e4:	pop    %r14
   8e8e6:	pop    %r15
   8e8e8:	pop    %rbp
   8e8e9:	ret
