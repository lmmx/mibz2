# <c30walk::ctor2::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000764f0 <<c30walk::ctor2::W<false>>::finish>:
   764f0:	push   %rbp
   764f1:	push   %r15
   764f3:	push   %r14
   764f5:	push   %r13
   764f7:	push   %r12
   764f9:	push   %rbx
   764fa:	sub    $0x38,%rsp
   764fe:	mov    0x18(%rdi),%rbp
   76502:	mov    0x10(%rbp),%r8
   76506:	mov    %r8,%r13
   76509:	sub    $0x1,%r13
   7650d:	jb     76a7a <<c30walk::ctor2::W<false>>::finish+0x58a>
   76513:	mov    %rdi,%r11
   76516:	lea    (%rsi,%rsi,4),%rax
   7651a:	mov    0x40(%rdi,%rax,8),%r14
   7651f:	mov    0x50(%rdi,%rax,8),%rdi
   76524:	mov    0x58(%r11,%rax,8),%r12d
   76529:	mov    0x60(%r11,%rax,8),%ecx
   7652e:	mov    0x64(%r11,%rax,8),%eax
   76533:	sub    %rdi,%r14
   76536:	mov    %r14,0x8(%rsp)
   7653b:	shr    $1,%r14
   7653e:	mov    0x8(%rbp),%r10
   76542:	mov    %r13,%rdx
   76545:	shl    $0xa,%rdx
   76549:	add    %r10,%rdx
   7654c:	mov    %r13,%rbx
   7654f:	cmp    %r12d,(%rdx,%rsi,4)
   76553:	jbe    765c8 <<c30walk::ctor2::W<false>>::finish+0xd8>
   76555:	mov    %r11,%r15
   76558:	lea    (%r12,%r14,1),%edx
   7655c:	dec    %edx
   7655e:	mov    %r8,%r9
   76561:	shl    $0xa,%r9
   76565:	lea    (%r9,%rsi,4),%r9
   76569:	lea    (%r10,%r9,1),%r11
   7656d:	add    $0xfffffffffffffc00,%r11
   76574:	mov    %r8,%rbx
   76577:	nopw   0x0(%rax,%rax,1)
   76580:	dec    %rbx
   76583:	cmp    %r8,%rbx
   76586:	jae    76a2e <<c30walk::ctor2::W<false>>::finish+0x53e>
   7658c:	cmp    %r12d,(%r11)
   7658f:	lea    -0x400(%r11),%r11
   76596:	ja     76580 <<c30walk::ctor2::W<false>>::finish+0x90>
   76598:	add    %r10,%r9
   7659b:	add    $0xfffffffffffffc00,%r9
   765a2:	mov    %r8,%r13
   765a5:	mov    %r15,%r11
   765a8:	nopl   0x0(%rax,%rax,1)
   765b0:	dec    %r13
   765b3:	cmp    %r8,%r13
   765b6:	jae    76a40 <<c30walk::ctor2::W<false>>::finish+0x550>
   765bc:	cmp    %edx,(%r9)
   765bf:	lea    -0x400(%r9),%r9
   765c6:	ja     765b0 <<c30walk::ctor2::W<false>>::finish+0xc0>
   765c8:	imul   $0xffffffffffffffe2,%rcx,%r15
   765cc:	add    %r12,%r15
   765cf:	cmp    $0xffffffff,%eax
   765d2:	mov    %r14,0x10(%rsp)
   765d7:	je     766db <<c30walk::ctor2::W<false>>::finish+0x1eb>
   765dd:	mov    $0xffffffff,%r9d
   765e3:	cmp    %r13,%rbx
   765e6:	jae    76600 <<c30walk::ctor2::W<false>>::finish+0x110>
   765e8:	lea    0x1(%rbx),%rcx
   765ec:	cmp    %r8,%rcx
   765ef:	jae    76a9d <<c30walk::ctor2::W<false>>::finish+0x5ad>
   765f5:	shl    $0xa,%rcx
   765f9:	add    %rcx,%r10
   765fc:	mov    (%r10,%rsi,4),%r9d
   76600:	mov    (%r11),%rdx
   76603:	mov    0x10(%rdx),%rcx
   76607:	cmp    %rax,%rcx
   7660a:	jbe    76a8b <<c30walk::ctor2::W<false>>::finish+0x59b>
   76610:	test   %r14,%r14
   76613:	je     76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   76619:	shl    $0x7,%rax
   7661d:	add    0x8(%rdx),%rax
   76621:	lea    0x800(,%rsi,4),%rdx
   76629:	xor    %r11d,%r11d
   7662c:	nopl   0x0(%rax)
   76630:	lea    (%r12,%r11,1),%r10d
   76634:	cmp    %r9d,%r10d
   76637:	jae    76670 <<c30walk::ctor2::W<false>>::finish+0x180>
   76639:	lea    (%r11,%r15,1),%rcx
   7663d:	cmp    $0x20,%rcx
   76641:	jae    76a52 <<c30walk::ctor2::W<false>>::finish+0x562>
   76647:	movzwl (%rdi,%r11,2),%r8d
   7664c:	inc    %r11
   7664f:	mov    %ebx,%r10d
   76652:	shl    $0x18,%r10d
   76656:	shl    $0x8,%r8d
   7665a:	or     %r10d,%r8d
   7665d:	or     %esi,%r8d
   76660:	mov    %r8d,(%rax,%rcx,4)
   76664:	cmp    0x10(%rsp),%r11
   76669:	jne    76630 <<c30walk::ctor2::W<false>>::finish+0x140>
   7666b:	jmp    766b9 <<c30walk::ctor2::W<false>>::finish+0x1c9>
   7666d:	nopl   (%rax)
   76670:	mov    %rbx,%r14
   76673:	shl    $0xa,%r14
   76677:	add    %rdx,%r14
   7667a:	mov    %rbx,%rcx
   7667d:	jmp    7668f <<c30walk::ctor2::W<false>>::finish+0x19f>
   7667f:	nop
   76680:	add    $0x400,%r14
   76687:	mov    %rbx,%rcx
   7668a:	cmp    %r9d,%r10d
   7668d:	jb     76639 <<c30walk::ctor2::W<false>>::finish+0x149>
   7668f:	lea    0x1(%rcx),%rbx
   76693:	mov    $0xffffffff,%r9d
   76699:	cmp    %r13,%rbx
   7669c:	jae    76680 <<c30walk::ctor2::W<false>>::finish+0x190>
   7669e:	add    $0x2,%rcx
   766a2:	mov    0x10(%rbp),%r8
   766a6:	cmp    %r8,%rcx
   766a9:	jae    76a1c <<c30walk::ctor2::W<false>>::finish+0x52c>
   766af:	mov    0x8(%rbp),%rcx
   766b3:	mov    (%rcx,%r14,1),%r9d
   766b7:	jmp    76680 <<c30walk::ctor2::W<false>>::finish+0x190>
   766b9:	mov    0x8(%rsp),%rdx
   766be:	and    $0xfffffffffffffffe,%rdx
   766c2:	mov    $0xff,%esi
   766c7:	add    $0x38,%rsp
   766cb:	pop    %rbx
   766cc:	pop    %r12
   766ce:	pop    %r13
   766d0:	pop    %r14
   766d2:	pop    %r15
   766d4:	pop    %rbp
   766d5:	jmp    *0x8f565(%rip)        # 105c40 <memset@GLIBC_2.2.5>
   766db:	movzwl (%rdi),%eax
   766de:	movzwl -0x2(%rdi,%r14,2),%r8d
   766e4:	mov    %ebx,%edx
   766e6:	shl    $0x10,%edx
   766e9:	or     %eax,%edx
   766eb:	mov    %r13d,%r9d
   766ee:	shl    $0x10,%r9d
   766f2:	or     %r8d,%r9d
   766f5:	sub    %edx,%r9d
   766f8:	cmp    $0xfffe,%r9d
   766ff:	jae    7677d <<c30walk::ctor2::W<false>>::finish+0x28d>
   76701:	mov    0x28(%r11),%r8
   76705:	shl    $0x6,%rcx
   76709:	shl    $0x8,%edx
   7670c:	or     %edx,%esi
   7670e:	mov    %esi,0x3c(%r8,%rcx,1)
   76713:	test   %r14,%r14
   76716:	je     76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   7671c:	cmp    $0x1e,%r14
   76720:	jne    76916 <<c30walk::ctor2::W<false>>::finish+0x426>
   76726:	movd   %eax,%xmm0
   7672a:	pshuflw $0x0,%xmm0,%xmm0
   7672f:	pshufd $0x44,%xmm0,%xmm1
   76734:	movdqu (%rdi),%xmm2
   76738:	movdqu 0x10(%rdi),%xmm3
   7673d:	movdqu 0x20(%rdi),%xmm4
   76742:	psubw  %xmm1,%xmm2
   76746:	movdqu %xmm2,(%rdi)
   7674a:	psubw  %xmm1,%xmm3
   7674e:	movdqu %xmm3,0x10(%rdi)
   76753:	psubw  %xmm1,%xmm4
   76757:	movdqu %xmm4,0x20(%rdi)
   7675c:	movq   0x30(%rdi),%xmm1
   76761:	psubw  %xmm0,%xmm1
   76765:	movq   %xmm1,0x30(%rdi)
   7676a:	movd   0x38(%rdi),%xmm1
   7676f:	psubw  %xmm0,%xmm1
   76773:	movd   %xmm1,0x38(%rdi)
   76778:	jmp    76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   7677d:	mov    (%r11),%r10
   76780:	mov    0x10(%r10),%r9
   76784:	cmp    (%r10),%r9
   76787:	mov    %r11,0x30(%rsp)
   7678c:	je     769dc <<c30walk::ctor2::W<false>>::finish+0x4ec>
   76792:	mov    0x8(%r10),%rax
   76796:	mov    %r9,%rdx
   76799:	shl    $0x7,%rdx
   7679d:	pxor   %xmm0,%xmm0
   767a1:	movdqa %xmm0,0x70(%rax,%rdx,1)
   767a7:	movdqa %xmm0,0x60(%rax,%rdx,1)
   767ad:	movdqa %xmm0,0x50(%rax,%rdx,1)
   767b3:	movdqa %xmm0,0x40(%rax,%rdx,1)
   767b9:	movdqa %xmm0,0x30(%rax,%rdx,1)
   767bf:	movdqa %xmm0,0x20(%rax,%rdx,1)
   767c5:	movdqa %xmm0,0x10(%rax,%rdx,1)
   767cb:	movdqa %xmm0,(%rax,%rdx,1)
   767d0:	lea    0x1(%r9),%rax
   767d4:	mov    %rax,0x10(%r10)
   767d8:	mov    $0xffffffff,%r8d
   767de:	cmp    %r13,%rbx
   767e1:	jae    76800 <<c30walk::ctor2::W<false>>::finish+0x310>
   767e3:	lea    0x1(%rbx),%rax
   767e7:	mov    0x10(%rbp),%r8
   767eb:	cmp    %r8,%rax
   767ee:	jae    76abb <<c30walk::ctor2::W<false>>::finish+0x5cb>
   767f4:	shl    $0xa,%rax
   767f8:	add    0x8(%rbp),%rax
   767fc:	mov    (%rax,%rsi,4),%r8d
   76800:	mov    %r9,0x8(%rsp)
   76805:	test   %r14,%r14
   76808:	je     768b5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
   7680e:	add    0x8(%r10),%rdx
   76812:	lea    0x800(,%rsi,4),%r9
   7681a:	xor    %r10d,%r10d
   7681d:	nopl   (%rax)
   76820:	lea    (%r12,%r10,1),%r11d
   76824:	cmp    %r8d,%r11d
   76827:	jae    76860 <<c30walk::ctor2::W<false>>::finish+0x370>
   76829:	lea    (%r10,%r15,1),%rax
   7682d:	cmp    $0x20,%rax
   76831:	jae    76a66 <<c30walk::ctor2::W<false>>::finish+0x576>
   76837:	movzwl (%rdi,%r10,2),%r11d
   7683c:	inc    %r10
   7683f:	mov    %ebx,%r14d
   76842:	shl    $0x18,%r14d
   76846:	shl    $0x8,%r11d
   7684a:	or     %r14d,%r11d
   7684d:	or     %esi,%r11d
   76850:	mov    %r11d,(%rdx,%rax,4)
   76854:	cmp    0x10(%rsp),%r10
   76859:	jne    76820 <<c30walk::ctor2::W<false>>::finish+0x330>
   7685b:	jmp    768b5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
   7685d:	nopl   (%rax)
   76860:	mov    %rbx,%r14
   76863:	shl    $0xa,%r14
   76867:	add    %r9,%r14
   7686a:	mov    %rbx,%rax
   7686d:	jmp    76887 <<c30walk::ctor2::W<false>>::finish+0x397>
   7686f:	nop
   76870:	mov    0x8(%rbp),%rax
   76874:	mov    (%rax,%r14,1),%r8d
   76878:	add    $0x400,%r14
   7687f:	mov    %rbx,%rax
   76882:	cmp    %r8d,%r11d
   76885:	jb     76829 <<c30walk::ctor2::W<false>>::finish+0x339>
   76887:	lea    0x1(%rax),%rbx
   7688b:	mov    $0xffffffff,%r8d
   76891:	cmp    %r13,%rbx
   76894:	jae    76878 <<c30walk::ctor2::W<false>>::finish+0x388>
   76896:	add    $0x2,%rax
   7689a:	mov    0x10(%rbp),%r8
   7689e:	cmp    %r8,%rax
   768a1:	jb     76870 <<c30walk::ctor2::W<false>>::finish+0x380>
   768a3:	lea    0x8bafe(%rip),%rdx        # 1023a8 <__frame_dummy_init_array_entry+0x1a08>
   768aa:	mov    %rax,%rdi
   768ad:	mov    %r8,%rsi
   768b0:	call   a9f23 <core::panicking::panic_bounds_check>
   768b5:	mov    0x30(%rsp),%r8
   768ba:	mov    0x28(%r8),%rax
   768be:	mov    %rcx,%rdx
   768c1:	shl    $0x6,%rdx
   768c5:	mov    0x8(%rsp),%rsi
   768ca:	or     $0x10000000,%esi
   768d0:	pcmpeqd %xmm0,%xmm0
   768d4:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   768da:	movdqa %xmm0,0x20(%rax,%rdx,1)
   768e0:	movdqa %xmm0,0x10(%rax,%rdx,1)
   768e6:	movdqa %xmm0,(%rax,%rdx,1)
   768eb:	mov    %esi,0x3c(%rax,%rdx,1)
   768ef:	mov    %ecx,%edi
   768f1:	shr    $0x6,%edi
   768f4:	mov    0x10(%r8),%rsi
   768f8:	cmp    %rsi,%rdi
   768fb:	jae    76aaf <<c30walk::ctor2::W<false>>::finish+0x5bf>
   76901:	mov    $0x1,%eax
   76906:	shl    %cl,%rax
   76909:	mov    0x8(%r8),%rcx
   7690d:	or     %rax,(%rcx,%rdi,8)
   76911:	jmp    76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   76916:	mov    0x8(%rsp),%rcx
   7691b:	cmp    $0x8,%rcx
   7691f:	jae    76928 <<c30walk::ctor2::W<false>>::finish+0x438>
   76921:	xor    %ecx,%ecx
   76923:	jmp    76ad2 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   76928:	movabs $0x7ffffffffffffff0,%rsi
   76932:	cmp    $0x20,%rcx
   76936:	jae    7693c <<c30walk::ctor2::W<false>>::finish+0x44c>
   76938:	xor    %ecx,%ecx
   7693a:	jmp    7699b <<c30walk::ctor2::W<false>>::finish+0x4ab>
   7693c:	mov    %r14,%rcx
   7693f:	and    %rsi,%rcx
   76942:	movd   %eax,%xmm0
   76946:	pshuflw $0x0,%xmm0,%xmm0
   7694b:	pshufd $0x44,%xmm0,%xmm0
   76950:	xor    %edx,%edx
   76952:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   76960:	movdqu (%rdi,%rdx,2),%xmm1
   76965:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   7696b:	psubw  %xmm0,%xmm1
   7696f:	psubw  %xmm0,%xmm2
   76973:	movdqu %xmm1,(%rdi,%rdx,2)
   76978:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   7697e:	add    $0x10,%rdx
   76982:	cmp    %rdx,%rcx
   76985:	jne    76960 <<c30walk::ctor2::W<false>>::finish+0x470>
   76987:	cmp    %rcx,%r14
   7698a:	je     76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   76990:	testb  $0x18,0x8(%rsp)
   76995:	je     76ad2 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   7699b:	mov    %rcx,%rdx
   7699e:	add    $0xc,%rsi
   769a2:	mov    %rsi,%rcx
   769a5:	and    %r14,%rcx
   769a8:	movd   %eax,%xmm0
   769ac:	pshuflw $0x0,%xmm0,%xmm0
   769b1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   769c0:	movq   (%rdi,%rdx,2),%xmm1
   769c5:	psubw  %xmm0,%xmm1
   769c9:	movq   %xmm1,(%rdi,%rdx,2)
   769ce:	add    $0x4,%rdx
   769d2:	cmp    %rdx,%rcx
   769d5:	jne    769c0 <<c30walk::ctor2::W<false>>::finish+0x4d0>
   769d7:	jmp    76acd <<c30walk::ctor2::W<false>>::finish+0x5dd>
   769dc:	mov    %rdi,0x8(%rsp)
   769e1:	mov    %r10,%rdi
   769e4:	mov    %rsi,0x20(%rsp)
   769e9:	mov    %rcx,0x28(%rsp)
   769ee:	mov    %r9,%r14
   769f1:	mov    %r10,0x18(%rsp)
   769f6:	call   7c690 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   769fb:	mov    0x18(%rsp),%r10
   76a00:	mov    %r14,%r9
   76a03:	mov    0x28(%rsp),%rcx
   76a08:	mov    0x20(%rsp),%rsi
   76a0d:	mov    0x10(%rsp),%r14
   76a12:	mov    0x8(%rsp),%rdi
   76a17:	jmp    76792 <<c30walk::ctor2::W<false>>::finish+0x2a2>
   76a1c:	lea    0x8b985(%rip),%rdx        # 1023a8 <__frame_dummy_init_array_entry+0x1a08>
   76a23:	mov    %rcx,%rdi
   76a26:	mov    %r8,%rsi
   76a29:	call   a9f23 <core::panicking::panic_bounds_check>
   76a2e:	lea    0x8ba7b(%rip),%rdx        # 1024b0 <__frame_dummy_init_array_entry+0x1b10>
   76a35:	mov    %rbx,%rdi
   76a38:	mov    %r8,%rsi
   76a3b:	call   a9f23 <core::panicking::panic_bounds_check>
   76a40:	lea    0x8ba69(%rip),%rdx        # 1024b0 <__frame_dummy_init_array_entry+0x1b10>
   76a47:	mov    %r13,%rdi
   76a4a:	mov    %r8,%rsi
   76a4d:	call   a9f23 <core::panicking::panic_bounds_check>
   76a52:	lea    0x8b937(%rip),%rdx        # 102390 <__frame_dummy_init_array_entry+0x19f0>
   76a59:	mov    $0x20,%esi
   76a5e:	mov    %rcx,%rdi
   76a61:	call   a9f23 <core::panicking::panic_bounds_check>
   76a66:	lea    0x8b923(%rip),%rdx        # 102390 <__frame_dummy_init_array_entry+0x19f0>
   76a6d:	mov    $0x20,%esi
   76a72:	mov    %rax,%rdi
   76a75:	call   a9f23 <core::panicking::panic_bounds_check>
   76a7a:	lea    0x8b99f(%rip),%rdx        # 102420 <__frame_dummy_init_array_entry+0x1a80>
   76a81:	mov    %r13,%rdi
   76a84:	xor    %esi,%esi
   76a86:	call   a9f23 <core::panicking::panic_bounds_check>
   76a8b:	lea    0x8b8e6(%rip),%rdx        # 102378 <__frame_dummy_init_array_entry+0x19d8>
   76a92:	mov    %rax,%rdi
   76a95:	mov    %rcx,%rsi
   76a98:	call   a9f23 <core::panicking::panic_bounds_check>
   76a9d:	lea    0x8b8bc(%rip),%rdx        # 102360 <__frame_dummy_init_array_entry+0x19c0>
   76aa4:	mov    %rcx,%rdi
   76aa7:	mov    %r8,%rsi
   76aaa:	call   a9f23 <core::panicking::panic_bounds_check>
   76aaf:	lea    0x8b982(%rip),%rdx        # 102438 <__frame_dummy_init_array_entry+0x1a98>
   76ab6:	call   a9f23 <core::panicking::panic_bounds_check>
   76abb:	lea    0x8b89e(%rip),%rdx        # 102360 <__frame_dummy_init_array_entry+0x19c0>
   76ac2:	mov    %rax,%rdi
   76ac5:	mov    %r8,%rsi
   76ac8:	call   a9f23 <core::panicking::panic_bounds_check>
   76acd:	cmp    %rcx,%r14
   76ad0:	je     76adb <<c30walk::ctor2::W<false>>::finish+0x5eb>
   76ad2:	sub    %ax,(%rdi,%rcx,2)
   76ad6:	inc    %rcx
   76ad9:	jmp    76acd <<c30walk::ctor2::W<false>>::finish+0x5dd>
   76adb:	add    $0x38,%rsp
   76adf:	pop    %rbx
   76ae0:	pop    %r12
   76ae2:	pop    %r13
   76ae4:	pop    %r14
   76ae6:	pop    %r15
   76ae8:	pop    %rbp
   76ae9:	ret
