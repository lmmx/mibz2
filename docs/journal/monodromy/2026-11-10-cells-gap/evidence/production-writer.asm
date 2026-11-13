
bench/cells-gap/target/release/mibz2-cells-gap:     file format elf64-x86-64


Disassembly of section .text:

00000000000382b0 <<mibz2::cells::Kernel>::write_cells>:
   382b0:	55                   	push   %rbp
   382b1:	41 57                	push   %r15
   382b3:	41 56                	push   %r14
   382b5:	41 55                	push   %r13
   382b7:	41 54                	push   %r12
   382b9:	53                   	push   %rbx
   382ba:	48 83 ec 68          	sub    $0x68,%rsp
   382be:	4d 89 c4             	mov    %r8,%r12
   382c1:	89 d5                	mov    %edx,%ebp
   382c3:	48 89 fb             	mov    %rdi,%rbx
   382c6:	48 89 4c 24 50       	mov    %rcx,0x50(%rsp)
   382cb:	48 83 c1 20          	add    $0x20,%rcx
   382cf:	4d 8b 68 10          	mov    0x10(%r8),%r13
   382d3:	49 89 ce             	mov    %rcx,%r14
   382d6:	4d 29 ee             	sub    %r13,%r14
   382d9:	89 74 24 08          	mov    %esi,0x8(%rsp)
   382dd:	76 45                	jbe    38324 <<mibz2::cells::Kernel>::write_cells+0x74>
   382df:	49 8b 04 24          	mov    (%r12),%rax
   382e3:	4c 29 e8             	sub    %r13,%rax
   382e6:	49 39 c6             	cmp    %rax,%r14
   382e9:	0f 87 68 01 00 00    	ja     38457 <<mibz2::cells::Kernel>::write_cells+0x1a7>
   382ef:	4d 8b 7c 24 08       	mov    0x8(%r12),%r15
   382f4:	4b 8d 3c 2f          	lea    (%r15,%r13,1),%rdi
   382f8:	49 83 fe 02          	cmp    $0x2,%r14
   382fc:	4c 89 7c 24 18       	mov    %r15,0x18(%rsp)
   38301:	72 16                	jb     38319 <<mibz2::cells::Kernel>::write_cells+0x69>
   38303:	49 8d 56 ff          	lea    -0x1(%r14),%rdx
   38307:	31 f6                	xor    %esi,%esi
   38309:	ff 15 21 e0 03 00    	call   *0x3e021(%rip)        # 76330 <memset@GLIBC_2.2.5>
   3830f:	4d 01 f5             	add    %r14,%r13
   38312:	49 ff cd             	dec    %r13
   38315:	4b 8d 3c 2f          	lea    (%r15,%r13,1),%rdi
   38319:	c6 07 00             	movb   $0x0,(%rdi)
   3831c:	49 ff c5             	inc    %r13
   3831f:	4c 89 e9             	mov    %r13,%rcx
   38322:	eb 0a                	jmp    3832e <<mibz2::cells::Kernel>::write_cells+0x7e>
   38324:	49 8b 44 24 08       	mov    0x8(%r12),%rax
   38329:	48 89 44 24 18       	mov    %rax,0x18(%rsp)
   3832e:	48 89 4c 24 40       	mov    %rcx,0x40(%rsp)
   38333:	49 89 4c 24 10       	mov    %rcx,0x10(%r12)
   38338:	48 8b 43 08          	mov    0x8(%rbx),%rax
   3833c:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
   38341:	48 8b 43 18          	mov    0x18(%rbx),%rax
   38345:	48 8b 4b 68          	mov    0x68(%rbx),%rcx
   38349:	48 89 4c 24 10       	mov    %rcx,0x10(%rsp)
   3834e:	4c 8b 7b 70          	mov    0x70(%rbx),%r15
   38352:	49 29 cf             	sub    %rcx,%r15
   38355:	4d 89 fe             	mov    %r15,%r14
   38358:	49 c1 ee 04          	shr    $0x4,%r14
   3835c:	4c 39 f0             	cmp    %r14,%rax
   3835f:	4c 89 74 24 28       	mov    %r14,0x28(%rsp)
   38364:	4c 89 64 24 58       	mov    %r12,0x58(%rsp)
   38369:	0f 83 8f 00 00 00    	jae    383fe <<mibz2::cells::Kernel>::write_cells+0x14e>
   3836f:	48 85 c0             	test   %rax,%rax
   38372:	74 0a                	je     3837e <<mibz2::cells::Kernel>::write_cells+0xce>
   38374:	48 8b 7b 20          	mov    0x20(%rbx),%rdi
   38378:	ff 15 c2 df 03 00    	call   *0x3dfc2(%rip)        # 76340 <free@GLIBC_2.2.5>
   3837e:	48 c7 43 18 00 00 00 00 	movq   $0x0,0x18(%rbx)
   38386:	48 c7 43 20 04 00 00 00 	movq   $0x4,0x20(%rbx)
   3838e:	48 c7 43 28 00 00 00 00 	movq   $0x0,0x28(%rbx)
   38396:	4e 8d 34 b5 00 00 00 00 	lea    0x0(,%r14,4),%r14
   3839e:	4c 89 f7             	mov    %r14,%rdi
   383a1:	ff 15 81 df 03 00    	call   *0x3df81(%rip)        # 76328 <malloc@GLIBC_2.2.5>
   383a7:	48 85 c0             	test   %rax,%rax
   383aa:	0f 84 f2 05 00 00    	je     389a2 <<mibz2::cells::Kernel>::write_cells+0x6f2>
   383b0:	49 89 c5             	mov    %rax,%r13
   383b3:	49 83 ff 20          	cmp    $0x20,%r15
   383b7:	72 1a                	jb     383d3 <<mibz2::cells::Kernel>::write_cells+0x123>
   383b9:	49 8d 56 fc          	lea    -0x4(%r14),%rdx
   383bd:	4c 89 ef             	mov    %r13,%rdi
   383c0:	be ff 00 00 00       	mov    $0xff,%esi
   383c5:	ff 15 65 df 03 00    	call   *0x3df65(%rip)        # 76330 <memset@GLIBC_2.2.5>
   383cb:	4b 8d 04 2e          	lea    (%r14,%r13,1),%rax
   383cf:	48 83 c0 fc          	add    $0xfffffffffffffffc,%rax
   383d3:	c7 00 ff ff ff ff    	movl   $0xffffffff,(%rax)
   383d9:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
   383de:	48 89 43 18          	mov    %rax,0x18(%rbx)
   383e2:	4c 89 6b 20          	mov    %r13,0x20(%rbx)
   383e6:	48 89 43 28          	mov    %rax,0x28(%rbx)
   383ea:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   383ef:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   383f4:	44 8b 7c 24 08       	mov    0x8(%rsp),%r15d
   383f9:	e9 e6 00 00 00       	jmp    384e4 <<mibz2::cells::Kernel>::write_cells+0x234>
   383fe:	4c 8b 6b 28          	mov    0x28(%rbx),%r13
   38402:	4c 89 f1             	mov    %r14,%rcx
   38405:	4d 29 ee             	sub    %r13,%r14
   38408:	76 44                	jbe    3844e <<mibz2::cells::Kernel>::write_cells+0x19e>
   3840a:	4c 29 e8             	sub    %r13,%rax
   3840d:	49 39 c6             	cmp    %rax,%r14
   38410:	77 68                	ja     3847a <<mibz2::cells::Kernel>::write_cells+0x1ca>
   38412:	4c 8b 7b 20          	mov    0x20(%rbx),%r15
   38416:	4b 8d 3c af          	lea    (%r15,%r13,4),%rdi
   3841a:	49 83 fe 02          	cmp    $0x2,%r14
   3841e:	0f 82 86 00 00 00    	jb     384aa <<mibz2::cells::Kernel>::write_cells+0x1fa>
   38424:	4a 8d 14 b5 fc ff ff ff 	lea    -0x4(,%r14,4),%rdx
   3842c:	be ff 00 00 00       	mov    $0xff,%esi
   38431:	ff 15 f9 de 03 00    	call   *0x3def9(%rip)        # 76330 <memset@GLIBC_2.2.5>
   38437:	4b 8d 04 2e          	lea    (%r14,%r13,1),%rax
   3843b:	4c 89 f9             	mov    %r15,%rcx
   3843e:	49 8d 3c 87          	lea    (%r15,%rax,4),%rdi
   38442:	48 83 c7 fc          	add    $0xfffffffffffffffc,%rdi
   38446:	4d 01 f5             	add    %r14,%r13
   38449:	49 ff cd             	dec    %r13
   3844c:	eb 5f                	jmp    384ad <<mibz2::cells::Kernel>::write_cells+0x1fd>
   3844e:	4c 8b 73 20          	mov    0x20(%rbx),%r14
   38452:	49 89 cd             	mov    %rcx,%r13
   38455:	eb 62                	jmp    384b9 <<mibz2::cells::Kernel>::write_cells+0x209>
   38457:	b9 01 00 00 00       	mov    $0x1,%ecx
   3845c:	41 b8 01 00 00 00    	mov    $0x1,%r8d
   38462:	4c 89 e7             	mov    %r12,%rdi
   38465:	4c 89 ee             	mov    %r13,%rsi
   38468:	4c 89 f2             	mov    %r14,%rdx
   3846b:	e8 20 8f ff ff       	call   31390 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   38470:	4d 8b 6c 24 10       	mov    0x10(%r12),%r13
   38475:	e9 75 fe ff ff       	jmp    382ef <<mibz2::cells::Kernel>::write_cells+0x3f>
   3847a:	48 8d 7b 18          	lea    0x18(%rbx),%rdi
   3847e:	b9 04 00 00 00       	mov    $0x4,%ecx
   38483:	41 b8 04 00 00 00    	mov    $0x4,%r8d
   38489:	4c 89 ee             	mov    %r13,%rsi
   3848c:	4c 89 f2             	mov    %r14,%rdx
   3848f:	e8 fc 8e ff ff       	call   31390 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   38494:	4c 8b 6b 28          	mov    0x28(%rbx),%r13
   38498:	4c 8b 7b 20          	mov    0x20(%rbx),%r15
   3849c:	4b 8d 3c af          	lea    (%r15,%r13,4),%rdi
   384a0:	49 83 fe 02          	cmp    $0x2,%r14
   384a4:	0f 83 7a ff ff ff    	jae    38424 <<mibz2::cells::Kernel>::write_cells+0x174>
   384aa:	4c 89 f9             	mov    %r15,%rcx
   384ad:	c7 07 ff ff ff ff    	movl   $0xffffffff,(%rdi)
   384b3:	49 ff c5             	inc    %r13
   384b6:	49 89 ce             	mov    %rcx,%r14
   384b9:	44 8b 7c 24 08       	mov    0x8(%rsp),%r15d
   384be:	4c 89 6b 28          	mov    %r13,0x28(%rbx)
   384c2:	49 c1 e5 02          	shl    $0x2,%r13
   384c6:	4c 89 f7             	mov    %r14,%rdi
   384c9:	be ff 00 00 00       	mov    $0xff,%esi
   384ce:	4c 89 ea             	mov    %r13,%rdx
   384d1:	ff 15 59 de 03 00    	call   *0x3de59(%rip)        # 76330 <memset@GLIBC_2.2.5>
   384d7:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   384dc:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   384e1:	4d 89 f5             	mov    %r14,%r13
   384e4:	81 e5 ff ff ff 7f    	and    $0x7fffffff,%ebp
   384ea:	45 89 fc             	mov    %r15d,%r12d
   384ed:	4d 39 e3             	cmp    %r12,%r11
   384f0:	4c 89 6c 24 08       	mov    %r13,0x8(%rsp)
   384f5:	48 89 6c 24 30       	mov    %rbp,0x30(%rsp)
   384fa:	76 22                	jbe    3851e <<mibz2::cells::Kernel>::write_cells+0x26e>
   384fc:	48 85 ed             	test   %rbp,%rbp
   384ff:	0f 84 91 04 00 00    	je     38996 <<mibz2::cells::Kernel>::write_cells+0x6e6>
   38505:	31 c0                	xor    %eax,%eax
   38507:	48 83 fd 01          	cmp    $0x1,%rbp
   3850b:	0f 95 c0             	setne  %al
   3850e:	49 89 ef             	mov    %rbp,%r15
   38511:	49 29 c7             	sub    %rax,%r15
   38514:	31 db                	xor    %ebx,%ebx
   38516:	4d 89 e6             	mov    %r12,%r14
   38519:	e9 2e 01 00 00       	jmp    3864c <<mibz2::cells::Kernel>::write_cells+0x39c>
   3851e:	31 c0                	xor    %eax,%eax
   38520:	4c 89 e2             	mov    %r12,%rdx
   38523:	eb 17                	jmp    3853c <<mibz2::cells::Kernel>::write_cells+0x28c>
   38525:	66 66 2e 0f 1f 84 00 00 00 00 00 	data16 cs nopw 0x0(%rax,%rax,1)
   38530:	48 c1 e9 20          	shr    $0x20,%rcx
   38534:	48 89 ca             	mov    %rcx,%rdx
   38537:	4c 39 d9             	cmp    %r11,%rcx
   3853a:	72 1b                	jb     38557 <<mibz2::cells::Kernel>::write_cells+0x2a7>
   3853c:	49 8b 0c 12          	mov    (%r10,%rdx,1),%rcx
   38540:	85 c9                	test   %ecx,%ecx
   38542:	78 ec                	js     38530 <<mibz2::cells::Kernel>::write_cells+0x280>
   38544:	49 8b 54 12 08       	mov    0x8(%r10,%rdx,1),%rdx
   38549:	81 e1 ff ff ff 7f    	and    $0x7fffffff,%ecx
   3854f:	48 01 c8             	add    %rcx,%rax
   38552:	48 89 d1             	mov    %rdx,%rcx
   38555:	eb d9                	jmp    38530 <<mibz2::cells::Kernel>::write_cells+0x280>
   38557:	48 85 ed             	test   %rbp,%rbp
   3855a:	0f 84 36 04 00 00    	je     38996 <<mibz2::cells::Kernel>::write_cells+0x6e6>
   38560:	48 ff c0             	inc    %rax
   38563:	48 89 c1             	mov    %rax,%rcx
   38566:	48 c1 e9 20          	shr    $0x20,%rcx
   3856a:	74 07                	je     38573 <<mibz2::cells::Kernel>::write_cells+0x2c3>
   3856c:	31 d2                	xor    %edx,%edx
   3856e:	48 f7 f5             	div    %rbp
   38571:	eb 04                	jmp    38577 <<mibz2::cells::Kernel>::write_cells+0x2c7>
   38573:	31 d2                	xor    %edx,%edx
   38575:	f7 f5                	div    %ebp
   38577:	49 89 ef             	mov    %rbp,%r15
   3857a:	49 29 d7             	sub    %rdx,%r15
   3857d:	31 db                	xor    %ebx,%ebx
   3857f:	4d 89 e6             	mov    %r12,%r14
   38582:	eb 42                	jmp    385c6 <<mibz2::cells::Kernel>::write_cells+0x316>
   38584:	66 66 66 2e 0f 1f 84 00 00 00 00 00 	data16 data16 cs nopw 0x0(%rax,%rax,1)
   38590:	48 03 54 24 18       	add    0x18(%rsp),%rdx
   38595:	48 c1 ee 20          	shr    $0x20,%rsi
   38599:	4c 01 d6             	add    %r10,%rsi
   3859c:	48 89 d7             	mov    %rdx,%rdi
   3859f:	48 89 c2             	mov    %rax,%rdx
   385a2:	ff 15 90 dd 03 00    	call   *0x3dd90(%rip)        # 76338 <memcpy@GLIBC_2.14>
   385a8:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   385ad:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   385b2:	45 85 ed             	test   %r13d,%r13d
   385b5:	48 0f 49 dd          	cmovns %rbp,%rbx
   385b9:	49 c1 ee 20          	shr    $0x20,%r14
   385bd:	4d 39 de             	cmp    %r11,%r14
   385c0:	0f 82 7c 00 00 00    	jb     38642 <<mibz2::cells::Kernel>::write_cells+0x392>
   385c6:	4f 8b 2c 32          	mov    (%r10,%r14,1),%r13
   385ca:	4b 8b 74 32 08       	mov    0x8(%r10,%r14,1),%rsi
   385cf:	44 89 ed             	mov    %r13d,%ebp
   385d2:	81 e5 ff ff ff 7f    	and    $0x7fffffff,%ebp
   385d8:	48 01 dd             	add    %rbx,%rbp
   385db:	45 85 ed             	test   %r13d,%r13d
   385de:	4d 89 ee             	mov    %r13,%r14
   385e1:	4c 0f 49 f6          	cmovns %rsi,%r14
   385e5:	49 0f 49 f5          	cmovns %r13,%rsi
   385e9:	48 89 e8             	mov    %rbp,%rax
   385ec:	48 0f 49 c3          	cmovns %rbx,%rax
   385f0:	4c 01 f8             	add    %r15,%rax
   385f3:	48 89 c1             	mov    %rax,%rcx
   385f6:	48 c1 e9 20          	shr    $0x20,%rcx
   385fa:	74 14                	je     38610 <<mibz2::cells::Kernel>::write_cells+0x360>
   385fc:	31 d2                	xor    %edx,%edx
   385fe:	48 f7 74 24 30       	divq   0x30(%rsp)
   38603:	89 f0                	mov    %esi,%eax
   38605:	48 83 f8 11          	cmp    $0x11,%rax
   38609:	72 85                	jb     38590 <<mibz2::cells::Kernel>::write_cells+0x2e0>
   3860b:	eb 15                	jmp    38622 <<mibz2::cells::Kernel>::write_cells+0x372>
   3860d:	0f 1f 00             	nopl   (%rax)
   38610:	31 d2                	xor    %edx,%edx
   38612:	f7 74 24 30          	divl   0x30(%rsp)
   38616:	89 f0                	mov    %esi,%eax
   38618:	48 83 f8 11          	cmp    $0x11,%rax
   3861c:	0f 82 6e ff ff ff    	jb     38590 <<mibz2::cells::Kernel>::write_cells+0x2e0>
   38622:	48 c1 ee 20          	shr    $0x20,%rsi
   38626:	4c 29 de             	sub    %r11,%rsi
   38629:	48 c1 ee 04          	shr    $0x4,%rsi
   3862d:	48 8b 4c 24 08       	mov    0x8(%rsp),%rcx
   38632:	8b 04 b1             	mov    (%rcx,%rsi,4),%eax
   38635:	39 c2                	cmp    %eax,%edx
   38637:	0f 42 c2             	cmovb  %edx,%eax
   3863a:	89 04 b1             	mov    %eax,(%rcx,%rsi,4)
   3863d:	e9 70 ff ff ff       	jmp    385b2 <<mibz2::cells::Kernel>::write_cells+0x302>
   38642:	48 8b 6c 24 30       	mov    0x30(%rsp),%rbp
   38647:	4c 8b 6c 24 08       	mov    0x8(%rsp),%r13
   3864c:	4c 01 fb             	add    %r15,%rbx
   3864f:	43 0f b6 0c 32       	movzbl (%r10,%r14,1),%ecx
   38654:	48 89 d8             	mov    %rbx,%rax
   38657:	48 c1 e8 20          	shr    $0x20,%rax
   3865b:	4c 89 7c 24 48       	mov    %r15,0x48(%rsp)
   38660:	74 0a                	je     3866c <<mibz2::cells::Kernel>::write_cells+0x3bc>
   38662:	48 89 d8             	mov    %rbx,%rax
   38665:	31 d2                	xor    %edx,%edx
   38667:	48 f7 f5             	div    %rbp
   3866a:	eb 06                	jmp    38672 <<mibz2::cells::Kernel>::write_cells+0x3c2>
   3866c:	89 d8                	mov    %ebx,%eax
   3866e:	31 d2                	xor    %edx,%edx
   38670:	f7 f5                	div    %ebp
   38672:	4c 8b 74 24 28       	mov    0x28(%rsp),%r14
   38677:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
   3867c:	88 0c 10             	mov    %cl,(%rax,%rdx,1)
   3867f:	4d 85 f6             	test   %r14,%r14
   38682:	0f 84 9d 01 00 00    	je     38825 <<mibz2::cells::Kernel>::write_cells+0x575>
   38688:	4a 8d 04 b5 00 00 00 00 	lea    0x0(,%r14,4),%rax
   38690:	4b 8d 0c 13          	lea    (%r11,%r10,1),%rcx
   38694:	48 83 c1 f8          	add    $0xfffffffffffffff8,%rcx
   38698:	eb 0c                	jmp    386a6 <<mibz2::cells::Kernel>::write_cells+0x3f6>
   3869a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
   386a0:	48 83 c0 fc          	add    $0xfffffffffffffffc,%rax
   386a4:	74 5c                	je     38702 <<mibz2::cells::Kernel>::write_cells+0x452>
   386a6:	41 8b 54 05 fc       	mov    -0x4(%r13,%rax,1),%edx
   386ab:	83 fa ff             	cmp    $0xffffffff,%edx
   386ae:	74 f0                	je     386a0 <<mibz2::cells::Kernel>::write_cells+0x3f0>
   386b0:	48 8b 7c 81 f8       	mov    -0x8(%rcx,%rax,4),%rdi
   386b5:	48 8b 34 81          	mov    (%rcx,%rax,4),%rsi
   386b9:	83 ff 10             	cmp    $0x10,%edi
   386bc:	76 1f                	jbe    386dd <<mibz2::cells::Kernel>::write_cells+0x42d>
   386be:	49 89 f8             	mov    %rdi,%r8
   386c1:	49 c1 e8 20          	shr    $0x20,%r8
   386c5:	4d 29 d8             	sub    %r11,%r8
   386c8:	49 c1 e8 04          	shr    $0x4,%r8
   386cc:	47 8b 4c 85 00       	mov    0x0(%r13,%r8,4),%r9d
   386d1:	44 39 ca             	cmp    %r9d,%edx
   386d4:	44 0f 42 ca          	cmovb  %edx,%r9d
   386d8:	47 89 4c 85 00       	mov    %r9d,0x0(%r13,%r8,4)
   386dd:	83 fe 10             	cmp    $0x10,%esi
   386e0:	76 be                	jbe    386a0 <<mibz2::cells::Kernel>::write_cells+0x3f0>
   386e2:	48 c1 ee 20          	shr    $0x20,%rsi
   386e6:	4c 29 de             	sub    %r11,%rsi
   386e9:	48 c1 ee 04          	shr    $0x4,%rsi
   386ed:	45 8b 44 b5 00       	mov    0x0(%r13,%rsi,4),%r8d
   386f2:	01 fa                	add    %edi,%edx
   386f4:	44 39 c2             	cmp    %r8d,%edx
   386f7:	41 0f 43 d0          	cmovae %r8d,%edx
   386fb:	41 89 54 b5 00       	mov    %edx,0x0(%r13,%rsi,4)
   38700:	eb 9e                	jmp    386a0 <<mibz2::cells::Kernel>::write_cells+0x3f0>
   38702:	4b 8d 04 13          	lea    (%r11,%r10,1),%rax
   38706:	48 83 c0 08          	add    $0x8,%rax
   3870a:	48 89 44 24 60       	mov    %rax,0x60(%rsp)
   3870f:	49 c1 e6 02          	shl    $0x2,%r14
   38713:	31 ed                	xor    %ebp,%ebp
   38715:	4c 89 74 24 28       	mov    %r14,0x28(%rsp)
   3871a:	eb 37                	jmp    38753 <<mibz2::cells::Kernel>::write_cells+0x4a3>
   3871c:	0f 1f 40 00          	nopl   0x0(%rax)
   38720:	4c 8b 74 24 20       	mov    0x20(%rsp),%r14
   38725:	4d 01 f5             	add    %r14,%r13
   38728:	48 89 df             	mov    %rbx,%rdi
   3872b:	4c 89 ee             	mov    %r13,%rsi
   3872e:	ff 15 04 dc 03 00    	call   *0x3dc04(%rip)        # 76338 <memcpy@GLIBC_2.14>
   38734:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   38739:	4d 89 f2             	mov    %r14,%r10
   3873c:	4c 8b 6c 24 08       	mov    0x8(%rsp),%r13
   38741:	4c 8b 74 24 28       	mov    0x28(%rsp),%r14
   38746:	48 83 c5 04          	add    $0x4,%rbp
   3874a:	49 39 ee             	cmp    %rbp,%r14
   3874d:	0f 84 d2 00 00 00    	je     38825 <<mibz2::cells::Kernel>::write_cells+0x575>
   38753:	45 8b 7c 2d 00       	mov    0x0(%r13,%rbp,1),%r15d
   38758:	b8 ff ff ff ff       	mov    $0xffffffff,%eax
   3875d:	49 39 c7             	cmp    %rax,%r15
   38760:	74 e4                	je     38746 <<mibz2::cells::Kernel>::write_cells+0x496>
   38762:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
   38767:	48 8b 74 a8 f8       	mov    -0x8(%rax,%rbp,4),%rsi
   3876c:	4c 8b 2c a8          	mov    (%rax,%rbp,4),%r13
   38770:	89 74 24 3c          	mov    %esi,0x3c(%rsp)
   38774:	41 89 f6             	mov    %esi,%r14d
   38777:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
   3877c:	4a 8d 1c 38          	lea    (%rax,%r15,1),%rbx
   38780:	48 c1 ee 20          	shr    $0x20,%rsi
   38784:	49 83 fe 11          	cmp    $0x11,%r14
   38788:	73 16                	jae    387a0 <<mibz2::cells::Kernel>::write_cells+0x4f0>
   3878a:	4c 01 d6             	add    %r10,%rsi
   3878d:	48 89 df             	mov    %rbx,%rdi
   38790:	4c 89 f2             	mov    %r14,%rdx
   38793:	ff 15 9f db 03 00    	call   *0x3db9f(%rip)        # 76338 <memcpy@GLIBC_2.14>
   38799:	eb 2a                	jmp    387c5 <<mibz2::cells::Kernel>::write_cells+0x515>
   3879b:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
   387a0:	4c 29 de             	sub    %r11,%rsi
   387a3:	48 c1 ee 04          	shr    $0x4,%rsi
   387a7:	48 8b 44 24 08       	mov    0x8(%rsp),%rax
   387ac:	8b 34 b0             	mov    (%rax,%rsi,4),%esi
   387af:	44 39 fe             	cmp    %r15d,%esi
   387b2:	74 16                	je     387ca <<mibz2::cells::Kernel>::write_cells+0x51a>
   387b4:	48 03 74 24 18       	add    0x18(%rsp),%rsi
   387b9:	48 89 df             	mov    %rbx,%rdi
   387bc:	4c 89 f2             	mov    %r14,%rdx
   387bf:	ff 15 c3 db 03 00    	call   *0x3dbc3(%rip)        # 76388 <memmove@GLIBC_2.2.5>
   387c5:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   387ca:	44 89 ea             	mov    %r13d,%edx
   387cd:	4c 01 f3             	add    %r14,%rbx
   387d0:	49 c1 ed 20          	shr    $0x20,%r13
   387d4:	83 fa 11             	cmp    $0x11,%edx
   387d7:	0f 82 43 ff ff ff    	jb     38720 <<mibz2::cells::Kernel>::write_cells+0x470>
   387dd:	4d 29 dd             	sub    %r11,%r13
   387e0:	49 c1 ed 04          	shr    $0x4,%r13
   387e4:	48 8b 44 24 08       	mov    0x8(%rsp),%rax
   387e9:	42 8b 34 a8          	mov    (%rax,%r13,4),%esi
   387ed:	49 89 c5             	mov    %rax,%r13
   387f0:	44 03 7c 24 3c       	add    0x3c(%rsp),%r15d
   387f5:	44 39 fe             	cmp    %r15d,%esi
   387f8:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   387fd:	4c 8b 74 24 28       	mov    0x28(%rsp),%r14
   38802:	0f 84 3e ff ff ff    	je     38746 <<mibz2::cells::Kernel>::write_cells+0x496>
   38808:	48 03 74 24 18       	add    0x18(%rsp),%rsi
   3880d:	48 89 df             	mov    %rbx,%rdi
   38810:	ff 15 72 db 03 00    	call   *0x3db72(%rip)        # 76388 <memmove@GLIBC_2.2.5>
   38816:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   3881b:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   38820:	e9 21 ff ff ff       	jmp    38746 <<mibz2::cells::Kernel>::write_cells+0x496>
   38825:	4d 39 e3             	cmp    %r12,%r11
   38828:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
   3882d:	76 79                	jbe    388a8 <<mibz2::cells::Kernel>::write_cells+0x5f8>
   3882f:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
   38834:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
   38839:	4d 39 ec             	cmp    %r13,%r12
   3883c:	4c 8b 7c 24 58       	mov    0x58(%rsp),%r15
   38841:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
   38846:	76 46                	jbe    3888e <<mibz2::cells::Kernel>::write_cells+0x5de>
   38848:	4c 8b 35 39 db 03 00 	mov    0x3db39(%rip),%r14        # 76388 <memmove@GLIBC_2.2.5>
   3884f:	90                   	nop
   38850:	4c 89 e3             	mov    %r12,%rbx
   38853:	4c 29 eb             	sub    %r13,%rbx
   38856:	4c 39 eb             	cmp    %r13,%rbx
   38859:	49 0f 43 dd          	cmovae %r13,%rbx
   3885d:	48 8b 54 24 40       	mov    0x40(%rsp),%rdx
   38862:	48 89 d0             	mov    %rdx,%rax
   38865:	48 29 d8             	sub    %rbx,%rax
   38868:	0f 82 ff 00 00 00    	jb     3896d <<mibz2::cells::Kernel>::write_cells+0x6bd>
   3886e:	49 39 c5             	cmp    %rax,%r13
   38871:	0f 87 07 01 00 00    	ja     3897e <<mibz2::cells::Kernel>::write_cells+0x6ce>
   38877:	48 89 ef             	mov    %rbp,%rdi
   3887a:	4c 01 ef             	add    %r13,%rdi
   3887d:	48 89 ee             	mov    %rbp,%rsi
   38880:	48 89 da             	mov    %rbx,%rdx
   38883:	41 ff d6             	call   *%r14
   38886:	49 01 dd             	add    %rbx,%r13
   38889:	4d 39 e5             	cmp    %r12,%r13
   3888c:	72 c2                	jb     38850 <<mibz2::cells::Kernel>::write_cells+0x5a0>
   3888e:	4c 3b 64 24 40       	cmp    0x40(%rsp),%r12
   38893:	77 04                	ja     38899 <<mibz2::cells::Kernel>::write_cells+0x5e9>
   38895:	4d 89 67 10          	mov    %r12,0x10(%r15)
   38899:	48 83 c4 68          	add    $0x68,%rsp
   3889d:	5b                   	pop    %rbx
   3889e:	41 5c                	pop    %r12
   388a0:	41 5d                	pop    %r13
   388a2:	41 5e                	pop    %r14
   388a4:	41 5f                	pop    %r15
   388a6:	5d                   	pop    %rbp
   388a7:	c3                   	ret
   388a8:	31 db                	xor    %ebx,%ebx
   388aa:	4c 8b 35 d7 da 03 00 	mov    0x3dad7(%rip),%r14        # 76388 <memmove@GLIBC_2.2.5>
   388b1:	eb 21                	jmp    388d4 <<mibz2::cells::Kernel>::write_cells+0x624>
   388b3:	66 66 66 66 2e 0f 1f 84 00 00 00 00 00 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   388c0:	45 85 ed             	test   %r13d,%r13d
   388c3:	48 0f 49 dd          	cmovns %rbp,%rbx
   388c7:	49 c1 ec 20          	shr    $0x20,%r12
   388cb:	4d 39 dc             	cmp    %r11,%r12
   388ce:	0f 82 5b ff ff ff    	jb     3882f <<mibz2::cells::Kernel>::write_cells+0x57f>
   388d4:	4f 8b 2c 22          	mov    (%r10,%r12,1),%r13
   388d8:	4b 8b 4c 22 08       	mov    0x8(%r10,%r12,1),%rcx
   388dd:	44 89 ed             	mov    %r13d,%ebp
   388e0:	81 e5 ff ff ff 7f    	and    $0x7fffffff,%ebp
   388e6:	48 01 dd             	add    %rbx,%rbp
   388e9:	45 85 ed             	test   %r13d,%r13d
   388ec:	48 89 e8             	mov    %rbp,%rax
   388ef:	48 0f 49 c3          	cmovns %rbx,%rax
   388f3:	4d 89 ec             	mov    %r13,%r12
   388f6:	4c 0f 49 e1          	cmovns %rcx,%r12
   388fa:	49 0f 49 cd          	cmovns %r13,%rcx
   388fe:	4c 01 f8             	add    %r15,%rax
   38901:	48 89 c2             	mov    %rax,%rdx
   38904:	48 c1 ea 20          	shr    $0x20,%rdx
   38908:	74 16                	je     38920 <<mibz2::cells::Kernel>::write_cells+0x670>
   3890a:	31 d2                	xor    %edx,%edx
   3890c:	48 f7 74 24 30       	divq   0x30(%rsp)
   38911:	89 c8                	mov    %ecx,%eax
   38913:	48 83 f8 10          	cmp    $0x10,%rax
   38917:	76 a7                	jbe    388c0 <<mibz2::cells::Kernel>::write_cells+0x610>
   38919:	eb 13                	jmp    3892e <<mibz2::cells::Kernel>::write_cells+0x67e>
   3891b:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
   38920:	31 d2                	xor    %edx,%edx
   38922:	f7 74 24 30          	divl   0x30(%rsp)
   38926:	89 c8                	mov    %ecx,%eax
   38928:	48 83 f8 10          	cmp    $0x10,%rax
   3892c:	76 92                	jbe    388c0 <<mibz2::cells::Kernel>::write_cells+0x610>
   3892e:	48 c1 e9 20          	shr    $0x20,%rcx
   38932:	4c 29 d9             	sub    %r11,%rcx
   38935:	48 c1 e9 04          	shr    $0x4,%rcx
   38939:	48 8b 74 24 08       	mov    0x8(%rsp),%rsi
   3893e:	8b 34 8e             	mov    (%rsi,%rcx,4),%esi
   38941:	48 39 f2             	cmp    %rsi,%rdx
   38944:	0f 84 76 ff ff ff    	je     388c0 <<mibz2::cells::Kernel>::write_cells+0x610>
   3894a:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
   3894f:	48 01 ca             	add    %rcx,%rdx
   38952:	48 01 ce             	add    %rcx,%rsi
   38955:	48 89 d7             	mov    %rdx,%rdi
   38958:	48 89 c2             	mov    %rax,%rdx
   3895b:	41 ff d6             	call   *%r14
   3895e:	4c 8b 5c 24 10       	mov    0x10(%rsp),%r11
   38963:	4c 8b 54 24 20       	mov    0x20(%rsp),%r10
   38968:	e9 53 ff ff ff       	jmp    388c0 <<mibz2::cells::Kernel>::write_cells+0x610>
   3896d:	48 8d 0d 84 c6 03 00 	lea    0x3c684(%rip),%rcx        # 74ff8 <__frame_dummy_init_array_entry+0xf38>
   38974:	31 ff                	xor    %edi,%edi
   38976:	48 89 de             	mov    %rbx,%rsi
   38979:	e8 32 31 ff ff       	call   2bab0 <core::slice::index::slice_index_fail>
   3897e:	48 8d 3d bb 34 fd ff 	lea    -0x2cb45(%rip),%rdi        # be40 <mibz2::runs::STEPS+0x800>
   38985:	48 8d 15 f4 bf 03 00 	lea    0x3bff4(%rip),%rdx        # 74980 <__frame_dummy_init_array_entry+0x8c0>
   3898c:	be 2b 00 00 00       	mov    $0x2b,%esi
   38991:	e8 2a 15 ff ff       	call   29ec0 <core::panicking::panic_fmt>
   38996:	48 8d 3d cb bf 03 00 	lea    0x3bfcb(%rip),%rdi        # 74968 <__frame_dummy_init_array_entry+0x8a8>
   3899d:	e8 2e 44 ff ff       	call   2cdd0 <core::panicking::panic_const::panic_const_rem_by_zero>
   389a2:	bf 04 00 00 00       	mov    $0x4,%edi
   389a7:	4c 89 f6             	mov    %r14,%rsi
   389aa:	e8 1c 0b ff ff       	call   294cb <alloc::raw_vec::handle_error>
