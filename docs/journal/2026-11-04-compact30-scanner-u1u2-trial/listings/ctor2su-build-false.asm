# <c30walk::ctor2su::Ctor2su>::build::<false>
# size 0x1834

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000047520 <<c30walk::ctor2su::Ctor2su>::build::<false>>:
   47520:	push   %rbp
   47521:	push   %r15
   47523:	push   %r14
   47525:	push   %r13
   47527:	push   %r12
   47529:	push   %rbx
   4752a:	sub    $0x1000,%rsp
   47531:	movq   $0x0,(%rsp)
   47539:	sub    $0x1000,%rsp
   47540:	movq   $0x0,(%rsp)
   47548:	sub    $0x1000,%rsp
   4754f:	movq   $0x0,(%rsp)
   47557:	sub    $0x1000,%rsp
   4755e:	movq   $0x0,(%rsp)
   47566:	sub    $0x1000,%rsp
   4756d:	movq   $0x0,(%rsp)
   47575:	sub    $0x4f8,%rsp
   4757c:	mov    %r8d,0x14(%rsp)
   47581:	mov    %rsi,0x50(%rsp)
   47586:	mov    0x5530(%rsp),%rax
   4758e:	movq   $0x0,0x10(%r9)
   47596:	movq   $0x0,0x10(%rax)
   4759e:	mov    %rdx,0x8(%rsp)
   475a3:	cmp    $0xeffe2,%rdx
   475aa:	ja     48a0d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ed>
   475b0:	mov    %rcx,%r14
   475b3:	pxor   %xmm0,%xmm0
   475b7:	xor    %eax,%eax
   475b9:	pxor   %xmm2,%xmm2
   475bd:	pxor   %xmm1,%xmm1
   475c1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   475d0:	movq   (%r14,%rax,4),%xmm3
   475d6:	movq   0x8(%r14,%rax,4),%xmm4
   475dd:	punpckldq %xmm0,%xmm3
   475e1:	paddq  %xmm2,%xmm3
   475e5:	punpckldq %xmm0,%xmm4
   475e9:	paddq  %xmm1,%xmm4
   475ed:	movq   0x10(%r14,%rax,4),%xmm2
   475f4:	movq   0x18(%r14,%rax,4),%xmm1
   475fb:	punpckldq %xmm0,%xmm2
   475ff:	paddq  %xmm3,%xmm2
   47603:	punpckldq %xmm0,%xmm1
   47607:	paddq  %xmm4,%xmm1
   4760b:	add    $0x8,%rax
   4760f:	cmp    $0x100,%rax
   47615:	jne    475d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb0>
   47617:	paddq  %xmm2,%xmm1
   4761b:	pshufd $0xee,%xmm1,%xmm0
   47620:	paddq  %xmm1,%xmm0
   47624:	movq   %xmm0,%rax
   47629:	cmp    0x8(%rsp),%rax
   4762e:	jne    48a0d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ed>
   47634:	pxor   %xmm2,%xmm2
   47638:	xor    %eax,%eax
   4763a:	movdqa -0x36ae2(%rip),%xmm3        # 10b60 <GCC_except_table1006+0xa08>
   47642:	pxor   %xmm1,%xmm1
   47646:	pxor   %xmm0,%xmm0
   4764a:	nopw   0x0(%rax,%rax,1)
   47650:	movq   (%r14,%rax,4),%xmm4
   47656:	movq   0x8(%r14,%rax,4),%xmm5
   4765d:	pshufd $0xd4,%xmm4,%xmm4
   47662:	pcmpeqd %xmm2,%xmm4
   47666:	pandn  %xmm3,%xmm4
   4766a:	paddq  %xmm1,%xmm4
   4766e:	pshufd $0xd4,%xmm5,%xmm5
   47673:	pcmpeqd %xmm2,%xmm5
   47677:	pandn  %xmm3,%xmm5
   4767b:	paddq  %xmm0,%xmm5
   4767f:	movq   0x10(%r14,%rax,4),%xmm0
   47686:	movq   0x18(%r14,%rax,4),%xmm6
   4768d:	pshufd $0xd4,%xmm0,%xmm1
   47692:	pcmpeqd %xmm2,%xmm1
   47696:	pandn  %xmm3,%xmm1
   4769a:	paddq  %xmm4,%xmm1
   4769e:	pshufd $0xd4,%xmm6,%xmm0
   476a3:	pcmpeqd %xmm2,%xmm0
   476a7:	pandn  %xmm3,%xmm0
   476ab:	paddq  %xmm5,%xmm0
   476af:	add    $0x8,%rax
   476b3:	cmp    $0x100,%rax
   476b9:	jne    47650 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x130>
   476bb:	mov    $0x88888889,%r13d
   476c1:	mov    0x8(%rsp),%rax
   476c6:	imul   %rax,%r13
   476ca:	shr    $0x24,%r13
   476ce:	imul   $0xeeeeeeef,%eax,%eax
   476d4:	ror    $1,%eax
   476d6:	cmp    $0x8888889,%eax
   476db:	sbb    $0xffffffffffffffff,%r13
   476df:	mov    %r13,%rbx
   476e2:	cmpl   $0x0,0x14(%rsp)
   476e7:	jne    47740 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x220>
   476e9:	mov    0x8(%rsp),%rsi
   476ee:	test   %rsi,%rsi
   476f1:	je     4773e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x21e>
   476f3:	paddq  %xmm1,%xmm0
   476f7:	pshufd $0xee,%xmm0,%xmm1
   476fc:	paddq  %xmm0,%xmm1
   47700:	movq   %xmm1,%rax
   47705:	lea    -0x1(%rsi),%ecx
   47708:	mov    $0x88888889,%edx
   4770d:	imul   %rcx,%rdx
   47711:	shr    $0x23,%rdx
   47715:	and    $0xfffffffe,%edx
   47718:	lea    (%rdx,%rsi,1),%ecx
   4771b:	dec    %ecx
   4771d:	shr    $1,%ecx
   4771f:	mov    $0x80010003,%ebx
   47724:	imul   %rcx,%rbx
   47728:	shr    $0x2e,%rbx
   4772c:	inc    %ebx
   4772e:	imul   %rax,%rbx
   47732:	dec    %rbx
   47735:	cmp    %r13,%rbx
   47738:	cmovae %r13,%rbx
   4773c:	jmp    47740 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x220>
   4773e:	xor    %ebx,%ebx
   47740:	mov    (%r9),%rax
   47743:	lea    0x1(%r13),%rcx
   47747:	mov    %rcx,0x90(%rsp)
   4774f:	cmp    %r13,%rax
   47752:	jbe    48b74 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1654>
   47758:	mov    0x5530(%rsp),%rax
   47760:	mov    (%rax),%rax
   47763:	cmp    %rbx,%rax
   47766:	jb     48bc9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16a9>
   4776c:	movq   $0x0,0x10(%rdi)
   47774:	lea    0x3f(%r13),%rbx
   47778:	shr    $0x6,%rbx
   4777c:	mov    %rdi,0x70(%rsp)
   47781:	mov    %r13,0x78(%rsp)
   47786:	je     477db <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2bb>
   47788:	cmp    (%rdi),%rbx
   4778b:	ja     48cd2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17b2>
   47791:	mov    %r9,%r13
   47794:	xor    %r15d,%r15d
   47797:	mov    0x8(%rdi),%r12
   4779b:	lea    (%r12,%r15,8),%rdi
   4779f:	cmp    $0x1,%rbx
   477a3:	je     477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   477a5:	lea    -0x8(,%rbx,8),%rdx
   477ad:	xor    %esi,%esi
   477af:	call   *0x105e3b(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   477b5:	lea    (%r15,%rbx,1),%rax
   477b9:	lea    (%r12,%rax,8),%rdi
   477bd:	add    $0xfffffffffffffff8,%rdi
   477c1:	add    %rbx,%r15
   477c4:	dec    %r15
   477c7:	mov    %r13,%r9
   477ca:	movq   $0x0,(%rdi)
   477d1:	inc    %r15
   477d4:	mov    0x70(%rsp),%rdi
   477d9:	jmp    477de <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2be>
   477db:	xor    %r15d,%r15d
   477de:	lea    0x18(%rdi),%r13
   477e2:	mov    %r15,0x10(%rdi)
   477e6:	movq   $0x0,0x28(%rdi)
   477ee:	cmpq   $0xf,0x18(%rdi)
   477f3:	jbe    48ca7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1787>
   477f9:	lea    0x30(%rdi),%r15
   477fd:	xor    %eax,%eax
   477ff:	pxor   %xmm0,%xmm0
   47803:	movabs $0xffffffff00000000,%rcx
   4780d:	nopl   (%rax)
   47810:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   47819:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   47822:	mov    %rcx,0x2d18(%rsp,%rax,1)
   4782a:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   47833:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   4783c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   47844:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   4784d:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   47856:	mov    %rcx,0x2d68(%rsp,%rax,1)
   4785e:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   47867:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   47870:	mov    %rcx,0x2d90(%rsp,%rax,1)
   47878:	add    $0xa0,%rax
   4787e:	cmp    $0x2800,%rax
   47884:	jne    47810 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2f0>
   47886:	lea    0xc(%r14),%rax
   4788a:	xor    %edx,%edx
   4788c:	mov    $0x25,%ecx
   47891:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   478a0:	mov    -0xc(%rax),%esi
   478a3:	add    %edx,%esi
   478a5:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   478ac:	mov    %esi,0x2c80(%rsp,%rcx,4)
   478b3:	mov    -0x8(%rax),%edx
   478b6:	add    %esi,%edx
   478b8:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   478bf:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   478c6:	mov    -0x4(%rax),%esi
   478c9:	add    %edx,%esi
   478cb:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   478d2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   478d9:	mov    (%rax),%edx
   478db:	add    %esi,%edx
   478dd:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   478e4:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   478eb:	add    $0x28,%rcx
   478ef:	add    $0x10,%rax
   478f3:	cmp    $0xa25,%rcx
   478fa:	jne    478a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x380>
   478fc:	mov    %r9,0x88(%rsp)
   47904:	mov    0x8(%r9),%rbx
   47908:	mov    0x8(%rdi),%rbp
   4790c:	mov    0x10(%rdi),%r12
   47910:	lea    0xe8(%rsp),%rdi
   47918:	lea    0x2cf8(%rsp),%rsi
   47920:	mov    $0x2800,%edx
   47925:	call   *0x105ce5(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   4792b:	mov    %rbp,%r9
   4792e:	mov    %r12,%rsi
   47931:	mov    %rbx,0xd0(%rsp)
   47939:	mov    0x78(%rsp),%rax
   4793e:	mov    %rax,0xd8(%rsp)
   47946:	mov    0x8(%rsp),%r11
   4794b:	mov    %r11,0xe0(%rsp)
   47953:	mov    0x5530(%rsp),%rax
   4795b:	mov    %rax,0xa8(%rsp)
   47963:	mov    %rbp,0xb0(%rsp)
   4796b:	mov    %r12,0xb8(%rsp)
   47973:	mov    %r13,0xc0(%rsp)
   4797b:	movl   $0x0,0x28f0(%rsp)
   47986:	movq   $0x0,0x28e8(%rsp)
   47992:	mov    %r15,0xc8(%rsp)
   4799a:	mov    $0xffffffffffffffff,%rcx
   479a1:	xor    %r13d,%r13d
   479a4:	movabs $0xeeeeeeeeeeeeeeef,%r10
   479ae:	movabs $0x888888888888889,%rdi
   479b8:	pxor   %xmm0,%xmm0
   479bc:	pcmpeqd %xmm1,%xmm1
   479c0:	xor    %r15d,%r15d
   479c3:	jmp    47a7e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x55e>
   479c8:	nopl   0x0(%rax,%rax,1)
   479d0:	mov    0x8(%rdi),%rax
   479d4:	mov    %rbp,%rcx
   479d7:	shl    $0x7,%rcx
   479db:	movdqa %xmm0,0x70(%rax,%rcx,1)
   479e1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   479e7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   479ed:	movdqa %xmm0,0x40(%rax,%rcx,1)
   479f3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   479f9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   479ff:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47a05:	movdqa %xmm0,(%rax,%rcx,1)
   47a0a:	mov    %rbp,%rax
   47a0d:	inc    %rax
   47a10:	mov    %rax,0x10(%rdi)
   47a14:	mov    %r12,%rax
   47a17:	shl    $0x6,%rax
   47a1b:	or     $0x10000000,%ebp
   47a21:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   47a27:	movdqa %xmm1,0x20(%rbx,%rax,1)
   47a2d:	movdqa %xmm1,0x10(%rbx,%rax,1)
   47a33:	movdqa %xmm1,(%rbx,%rax,1)
   47a38:	mov    %ebp,0x3c(%rbx,%rax,1)
   47a3c:	mov    %r12,%rdi
   47a3f:	shr    $0x6,%rdi
   47a43:	cmp    %rsi,%rdi
   47a46:	jae    48d39 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1819>
   47a4c:	mov    $0x1,%eax
   47a51:	mov    %r12d,%ecx
   47a54:	shl    %cl,%rax
   47a57:	or     %rax,(%r9,%rdi,8)
   47a5b:	mov    %r12,%rcx
   47a5e:	mov    0x8(%rsp),%r11
   47a63:	movabs $0x888888888888889,%rdi
   47a6d:	add    $0x4,%r15
   47a71:	cmp    $0x400,%r15
   47a78:	je     47b10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x5f0>
   47a7e:	mov    %r13,%rax
   47a81:	mov    (%r14,%r15,1),%r13d
   47a85:	add    %rax,%r13
   47a88:	cmp    %r11,%r13
   47a8b:	setae  %r8b
   47a8f:	mov    %r13,%rax
   47a92:	movabs $0x8888888888888889,%rdx
   47a9c:	mul    %rdx
   47a9f:	mov    %rdx,%r12
   47aa2:	shr    $0x4,%r12
   47aa6:	mov    %r13,%rax
   47aa9:	imul   %r10,%rax
   47aad:	ror    $1,%rax
   47ab0:	cmp    %rdi,%rax
   47ab3:	setb   %al
   47ab6:	cmp    %rcx,%r12
   47ab9:	sete   %dl
   47abc:	or     %r8b,%dl
   47abf:	or     %al,%dl
   47ac1:	jne    47a6d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x54d>
   47ac3:	mov    0x5530(%rsp),%rdi
   47acb:	mov    0x10(%rdi),%rbp
   47acf:	cmp    (%rdi),%rbp
   47ad2:	jne    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47ad8:	mov    %rsi,0x20(%rsp)
   47add:	mov    %r9,0x30(%rsp)
   47ae2:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47ae7:	pcmpeqd %xmm1,%xmm1
   47aeb:	pxor   %xmm0,%xmm0
   47aef:	movabs $0xeeeeeeeeeeeeeeef,%r10
   47af9:	mov    0x30(%rsp),%r9
   47afe:	mov    0x20(%rsp),%rsi
   47b03:	mov    0x5530(%rsp),%rdi
   47b0b:	jmp    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47b10:	cmpl   $0x0,0x14(%rsp)
   47b15:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47b1b:	mov    0xd8(%rsp),%r8
   47b23:	test   %r8,%r8
   47b26:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47b2c:	mov    %r11,%r9
   47b2f:	shl    $0x20,%r9
   47b33:	mov    0x14(%rsp),%r10d
   47b38:	mov    0xa8(%rsp),%rdi
   47b40:	mov    0xd0(%rsp),%rbp
   47b48:	xor    %r15d,%r15d
   47b4b:	movabs $0xff51afd7ed558ccd,%rbx
   47b55:	pxor   %xmm0,%xmm0
   47b59:	pcmpeqd %xmm1,%xmm1
   47b5d:	jmp    47bf0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6d0>
   47b62:	mov    0x8(%rdi),%rax
   47b66:	mov    %rbx,%rcx
   47b69:	shl    $0x7,%rcx
   47b6d:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47b73:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47b79:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47b7f:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47b85:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47b8b:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47b91:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47b97:	movdqa %xmm0,(%rax,%rcx,1)
   47b9c:	mov    %rbx,%rax
   47b9f:	inc    %rax
   47ba2:	mov    %rax,0x10(%rdi)
   47ba6:	or     $0x10000000,%ebx
   47bac:	movdqu %xmm1,0x2c(%rbp)
   47bb1:	movdqa %xmm1,0x20(%rbp)
   47bb6:	movdqa %xmm1,0x10(%rbp)
   47bbb:	movdqa %xmm1,0x0(%rbp)
   47bc0:	mov    %ebx,0x3c(%rbp)
   47bc3:	or     %r14,(%r12,%r13,8)
   47bc7:	movabs $0xff51afd7ed558ccd,%rbx
   47bd1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47be0:	inc    %r15
   47be3:	add    $0x40,%rbp
   47be7:	cmp    %r15,%r8
   47bea:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47bf0:	mov    %r15,%r13
   47bf3:	shr    $0x6,%r13
   47bf7:	mov    0xb8(%rsp),%rsi
   47bff:	cmp    %rsi,%r13
   47c02:	jae    48d45 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1825>
   47c08:	mov    0xb0(%rsp),%r12
   47c10:	mov    (%r12,%r13,8),%rax
   47c14:	mov    $0x1,%r14d
   47c1a:	mov    %r15d,%ecx
   47c1d:	shl    %cl,%r14
   47c20:	bt     %r15,%rax
   47c24:	jb     47be0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6c0>
   47c26:	mov    %r15,%rax
   47c29:	xor    %r9,%rax
   47c2c:	mov    %rax,%rcx
   47c2f:	shr    $0x21,%rcx
   47c33:	xor    %rax,%rcx
   47c36:	imul   %rbx,%rcx
   47c3a:	mov    %rcx,%rax
   47c3d:	shr    $0x21,%rax
   47c41:	xor    %rcx,%rax
   47c44:	movabs $0xc4ceb9fe1a85ec53,%rcx
   47c4e:	imul   %rcx,%rax
   47c52:	mov    %rax,%rcx
   47c55:	shr    $0x21,%rcx
   47c59:	xor    %rax,%rcx
   47c5c:	mov    %rcx,%rax
   47c5f:	shr    $0x3,%rax
   47c63:	movabs $0x20c49ba5e353f7cf,%rdx
   47c6d:	mul    %rdx
   47c70:	shr    $0x4,%rdx
   47c74:	imul   $0x3e8,%rdx,%rax
   47c7b:	sub    %rax,%rcx
   47c7e:	cmp    %r10,%rcx
   47c81:	jae    47be0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6c0>
   47c87:	mov    0x10(%rdi),%rbx
   47c8b:	cmp    (%rdi),%rbx
   47c8e:	jne    47b62 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x642>
   47c94:	mov    %r8,0x20(%rsp)
   47c99:	mov    %r9,0x30(%rsp)
   47c9e:	mov    %r10,0x68(%rsp)
   47ca3:	mov    %rdi,0x60(%rsp)
   47ca8:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47cad:	pcmpeqd %xmm1,%xmm1
   47cb1:	pxor   %xmm0,%xmm0
   47cb5:	mov    0x60(%rsp),%rdi
   47cba:	mov    0x68(%rsp),%r10
   47cbf:	mov    0x30(%rsp),%r9
   47cc4:	mov    0x20(%rsp),%r8
   47cc9:	mov    0x8(%rsp),%r11
   47cce:	jmp    47b62 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x642>
   47cd3:	test   %r11,%r11
   47cd6:	je     489a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1480>
   47cdc:	movq   $0x0,0x28(%rsp)
   47ce5:	jmp    47d24 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x804>
   47ce7:	mov    0x38(%rsp),%r15
   47cec:	lea    0x1(%rax),%rdx
   47cf0:	add    $0xffffffffffffffe3,%rax
   47cf4:	xor    %esi,%esi
   47cf6:	cmp    $0x1e,%rdx
   47cfa:	setae  %sil
   47cfe:	cmovb  %rdx,%rax
   47d02:	lea    (%rcx,%rsi,2),%ecx
   47d05:	inc    %ecx
   47d07:	mov    %rax,0x28e8(%rsp)
   47d0f:	mov    %ecx,0x28f0(%rsp)
   47d16:	cmp    %rdi,0x98(%rsp)
   47d1e:	jae    489a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1480>
   47d24:	mov    0x28(%rsp),%rbx
   47d29:	mov    $0x400,%edx
   47d2e:	lea    0x28f8(%rsp),%rdi
   47d36:	xor    %esi,%esi
   47d38:	call   *0x1058b2(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   47d3e:	lea    0x128(%rsp),%rax
   47d46:	xor    %ecx,%ecx
   47d48:	nopl   0x0(%rax,%rax,1)
   47d50:	mov    -0x40(%rax),%rdx
   47d54:	sub    -0x30(%rax),%rdx
   47d58:	shr    $1,%rdx
   47d5b:	add    -0x28(%rax),%edx
   47d5e:	mov    -0x18(%rax),%rsi
   47d62:	sub    -0x8(%rax),%rsi
   47d66:	mov    %edx,0x28f8(%rsp,%rcx,4)
   47d6d:	shr    $1,%rsi
   47d70:	add    (%rax),%esi
   47d72:	mov    %esi,0x28fc(%rsp,%rcx,4)
   47d79:	add    $0x2,%rcx
   47d7d:	add    $0x50,%rax
   47d81:	cmp    $0x100,%rcx
   47d88:	jne    47d50 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x830>
   47d8a:	mov    0xc0(%rsp),%r12
   47d92:	mov    0x10(%r12),%r14
   47d97:	cmp    (%r12),%r14
   47d9b:	je     48993 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1473>
   47da1:	mov    %r14,%rdi
   47da4:	shl    $0xa,%rdi
   47da8:	add    0x8(%r12),%rdi
   47dad:	mov    $0x400,%edx
   47db2:	lea    0x28f8(%rsp),%rsi
   47dba:	call   *0x105850(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   47dc0:	inc    %r14
   47dc3:	lea    0xf000(%rbx),%rax
   47dca:	mov    0x8(%rsp),%rdi
   47dcf:	cmp    %rdi,%rax
   47dd2:	mov    %rdi,%rcx
   47dd5:	mov    %rax,0x98(%rsp)
   47ddd:	cmovb  %rax,%rcx
   47de1:	mov    %r14,0x10(%r12)
   47de6:	mov    %rcx,0x28(%rsp)
   47deb:	mov    %rcx,%r12
   47dee:	sub    %rbx,%r12
   47df1:	cmp    $0x10,%r12
   47df5:	jb     487c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12a0>
   47dfb:	add    0x50(%rsp),%rbx
   47e00:	xor    %edx,%edx
   47e02:	lea    0xe8(%rsp),%r8
   47e0a:	jmp    47e5d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93d>
   47e0c:	nopl   0x0(%rax)
   47e10:	mov    0x28e8(%rsp),%rax
   47e18:	lea    0x10(%rax),%rcx
   47e1c:	add    $0xfffffffffffffff2,%rax
   47e20:	xor    %edx,%edx
   47e22:	cmp    $0x1e,%rcx
   47e26:	setae  %dl
   47e29:	cmovb  %rcx,%rax
   47e2d:	mov    0x28f0(%rsp),%ecx
   47e34:	lea    (%rcx,%rdx,2),%ecx
   47e37:	add    $0x10,%ecx
   47e3a:	mov    %rax,0x28e8(%rsp)
   47e42:	mov    %ecx,0x28f0(%rsp)
   47e49:	xor    %edx,%edx
   47e4b:	mov    0x40(%rsp),%rbx
   47e50:	mov    %r9,%r12
   47e53:	cmp    $0xf,%r9
   47e57:	jbe    487cb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12ab>
   47e5d:	mov    0x58(%rsp),%rcx
   47e62:	test   $0x1,%dl
   47e65:	jne    47e6e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x94e>
   47e67:	mov    (%rbx),%rcx
   47e6a:	mov    0x8(%rbx),%r15
   47e6e:	lea    0x10(%rbx),%r9
   47e72:	lea    -0x10(%r12),%rdx
   47e77:	xor    %rcx,%r15
   47e7a:	mov    %rcx,%rax
   47e7d:	rol    $0x8,%rax
   47e81:	xor    %rcx,%rax
   47e84:	or     %r15,%rax
   47e87:	jne    47ee0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x9c0>
   47e89:	cmp    $0x10,%rdx
   47e8d:	jb     48030 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb10>
   47e93:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47ea0:	mov    (%r9),%rsi
   47ea3:	mov    0x8(%r9),%rax
   47ea7:	mov    $0x1,%r10b
   47eaa:	cmp    %rcx,%rsi
   47ead:	jne    4803a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb1a>
   47eb3:	cmp    %rcx,%rax
   47eb6:	jne    4803a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb1a>
   47ebc:	add    $0xfffffffffffffff0,%rdx
   47ec0:	add    $0x10,%r9
   47ec4:	cmp    $0xf,%rdx
   47ec8:	ja     47ea0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x980>
   47eca:	mov    %r9,0x40(%rsp)
   47ecf:	xor    %r10d,%r10d
   47ed2:	mov    %rcx,0x58(%rsp)
   47ed7:	jmp    48049 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb29>
   47edc:	nopl   0x0(%rax)
   47ee0:	mov    %r9,0x40(%rsp)
   47ee5:	mov    %rdx,0x18(%rsp)
   47eea:	mov    0x28(%rsp),%rdx
   47eef:	sub    %r12,%rdx
   47ef2:	mov    0x28f0(%rsp),%r12d
   47efa:	mov    $0x1e,%r9d
   47f00:	sub    0x28e8(%rsp),%r9
   47f08:	cmp    $0x10,%r9
   47f0c:	mov    $0x10,%r15d
   47f12:	cmovb  %r9,%r15
   47f16:	add    0x50(%rsp),%rdx
   47f1b:	cmp    $0x1,%r15
   47f1f:	mov    %r15,%rbp
   47f22:	adc    $0x0,%rbp
   47f26:	xor    %r13d,%r13d
   47f29:	nopl   0x0(%rax)
   47f30:	movzbl (%rbx,%r13,1),%esi
   47f35:	lea    (%rsi,%rsi,4),%rcx
   47f39:	lea    (%r8,%rcx,8),%r14
   47f3d:	mov    (%r8,%rcx,8),%rax
   47f41:	cmp    0x8(%r8,%rcx,8),%rax
   47f46:	je     47f5d <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa3d>
   47f48:	lea    (%r12,%r13,1),%rcx
   47f4c:	mov    %cx,(%rax)
   47f4f:	addq   $0x2,(%r14)
   47f53:	inc    %r13
   47f56:	cmp    %r13,%rbp
   47f59:	jne    47f30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa10>
   47f5b:	jmp    47fa0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa80>
   47f5d:	mov    %r9,0x30(%rsp)
   47f62:	mov    %rdx,0x20(%rsp)
   47f67:	lea    0xa8(%rsp),%rdi
   47f6f:	call   96ec0 <<c30walk::ctor2su::W<false>>::refill>
   47f74:	lea    0xe8(%rsp),%r8
   47f7c:	test   %al,%al
   47f7e:	je     489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   47f84:	mov    (%r14),%rax
   47f87:	mov    0x8(%rsp),%rdi
   47f8c:	mov    0x20(%rsp),%rdx
   47f91:	mov    0x30(%rsp),%r9
   47f96:	jmp    47f48 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa28>
   47f98:	nopl   0x0(%rax,%rax,1)
   47fa0:	cmp    $0xf,%r9
   47fa4:	mov    0x18(%rsp),%r9
   47fa9:	ja     47e10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   47faf:	add    $0x2,%r12d
   47fb3:	movzwl %r12w,%ebx
   47fb7:	nopw   0x0(%rax,%rax,1)
   47fc0:	movzbl (%rdx,%r15,1),%esi
   47fc5:	lea    (%rsi,%rsi,4),%rcx
   47fc9:	lea    (%r8,%rcx,8),%r14
   47fcd:	mov    (%r8,%rcx,8),%rax
   47fd1:	cmp    0x8(%r8,%rcx,8),%rax
   47fd6:	je     47ff1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xad1>
   47fd8:	lea    (%rbx,%r15,1),%rcx
   47fdc:	mov    %cx,(%rax)
   47fdf:	addq   $0x2,(%r14)
   47fe3:	inc    %r15
   47fe6:	cmp    $0x10,%r15
   47fea:	jne    47fc0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaa0>
   47fec:	jmp    47e10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   47ff1:	mov    %rdx,%r12
   47ff4:	lea    0xa8(%rsp),%rdi
   47ffc:	call   96ec0 <<c30walk::ctor2su::W<false>>::refill>
   48001:	lea    0xe8(%rsp),%r8
   48009:	test   %al,%al
   4800b:	je     489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48011:	mov    (%r14),%rax
   48014:	mov    0x8(%rsp),%rdi
   48019:	mov    0x18(%rsp),%r9
   4801e:	mov    %r12,%rdx
   48021:	jmp    47fd8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xab8>
   48023:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48030:	mov    %r9,0x40(%rsp)
   48035:	xor    %r10d,%r10d
   48038:	jmp    48049 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb29>
   4803a:	mov    %r9,0x40(%rsp)
   4803f:	mov    %rsi,0x58(%rsp)
   48044:	mov    %rax,0x38(%rsp)
   48049:	mov    %rdx,0x18(%rsp)
   4804e:	sub    %rdx,%r12
   48051:	movzbl %cl,%eax
   48054:	mov    %rax,0xa0(%rsp)
   4805c:	lea    (%rax,%rax,4),%rdx
   48060:	mov    (%r8,%rdx,8),%rax
   48064:	mov    0x1c(%r8,%rdx,8),%esi
   48069:	mov    %rax,%rdi
   4806c:	sub    0x10(%r8,%rdx,8),%rdi
   48071:	shr    $1,%rdi
   48074:	add    0x18(%r8,%rdx,8),%edi
   48079:	add    %r12,%rdi
   4807c:	cmp    %rsi,%rdi
   4807f:	ja     489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48085:	mov    %r10d,0x4c(%rsp)
   4808a:	lea    (%r8,%rdx,8),%rbx
   4808e:	movzbl %cl,%r15d
   48092:	mov    0x8(%rbx),%rsi
   48096:	mov    $0x88888889,%r14d
   4809c:	lea    -0x2a763(%rip),%r13        # 1d940 <c30walk::ctor2su::PATTERNS>
   480a3:	mov    %rbx,0x80(%rsp)
   480ab:	jmp    480e6 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xbc6>
   480ad:	nopl   (%rax)
   480b0:	mov    0x3c(%rsi),%edi
   480b3:	mov    $0xfffff,%r9d
   480b9:	and    %r9d,%edi
   480bc:	sub    %r8d,%ecx
   480bf:	sub    %eax,%r8d
   480c2:	mov    %r8d,%eax
   480c5:	lea    (%rsi,%rax,2),%rax
   480c9:	mov    %rax,0x10(%rbx)
   480cd:	mov    %rax,(%rbx)
   480d0:	lea    (%rax,%rcx,2),%rsi
   480d4:	mov    %rsi,0x8(%rbx)
   480d8:	mov    %edx,0x20(%rbx)
   480db:	mov    %edi,0x24(%rbx)
   480de:	lea    0xe8(%rsp),%r8
   480e6:	sub    %rax,%rsi
   480e9:	shr    $1,%rsi
   480ec:	cmp    %rsi,%r12
   480ef:	cmovb  %r12,%rsi
   480f3:	test   %rsi,%rsi
   480f6:	je     481b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc90>
   480fc:	mov    0x28e8(%rsp),%rdi
   48104:	cmp    $0x1d,%rdi
   48108:	ja     48d17 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17f7>
   4810e:	cmp    $0x1f,%rsi
   48112:	jae    48a21 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1501>
   48118:	mov    0x28f0(%rsp),%ecx
   4811f:	mov    %rdi,%r11
   48122:	shl    $0x6,%r11
   48126:	lea    (%r11,%r13,1),%rdx
   4812a:	lea    -0x2(,%rsi,2),%r9
   48132:	cmp    $0xe,%r9
   48136:	jb     4814b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc2b>
   48138:	mov    %rax,%r8
   4813b:	sub    %r13,%r8
   4813e:	sub    %r11,%r8
   48141:	cmp    $0x10,%r8
   48145:	jae    48598 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1078>
   4814b:	mov    %rdx,%r8
   4814e:	xor    %r10d,%r10d
   48151:	lea    (%rdx,%rsi,2),%rdx
   48155:	lea    (%rax,%r10,2),%r9
   48159:	nopl   0x0(%rax)
   48160:	movzwl (%r8),%r10d
   48164:	add    %cx,%r10w
   48168:	add    $0x2,%r8
   4816c:	mov    %r10w,(%r9)
   48170:	add    $0x2,%r9
   48174:	cmp    %rdx,%r8
   48177:	jne    48160 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc40>
   48179:	lea    (%rax,%rsi,2),%rax
   4817d:	mov    %rax,(%rbx)
   48180:	add    %rsi,%rdi
   48183:	lea    0x2(%rsi),%edx
   48186:	mov    %rdi,%r8
   48189:	sub    $0x1e,%r8
   4818d:	cmovb  %rdi,%r8
   48191:	cmovb  %esi,%edx
   48194:	mov    %r8,0x28e8(%rsp)
   4819c:	add    %ecx,%edx
   4819e:	mov    %edx,0x28f0(%rsp)
   481a5:	sub    %rsi,%r12
   481a8:	lea    0xe8(%rsp),%r8
   481b0:	test   %r12,%r12
   481b3:	je     48780 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1260>
   481b9:	mov    %rax,%r8
   481bc:	cmp    0x10(%rbx),%rax
   481c0:	je     481de <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcbe>
   481c2:	lea    0xa8(%rsp),%rdi
   481ca:	mov    0xa0(%rsp),%rsi
   481d2:	call   968c0 <<c30walk::ctor2su::W<false>>::finish>
   481d7:	mov    (%rbx),%r8
   481da:	mov    0x10(%rbx),%rax
   481de:	sub    %rax,%r8
   481e1:	shr    $1,%r8
   481e4:	add    0x18(%rbx),%r8d
   481e8:	mov    %r8d,0x18(%rbx)
   481ec:	pxor   %xmm0,%xmm0
   481f0:	movdqu %xmm0,(%rbx)
   481f4:	movq   $0x0,0x10(%rbx)
   481fc:	cmp    $0x1e,%r12
   48200:	jb     48710 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11f0>
   48206:	mov    %r8,%rcx
   48209:	imul   %r14,%rcx
   4820d:	shr    $0x24,%rcx
   48211:	mov    %ecx,%eax
   48213:	shl    $0x5,%eax
   48216:	lea    (%rcx,%rcx,1),%edx
   48219:	sub    %eax,%edx
   4821b:	add    %r8d,%edx
   4821e:	jne    48710 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11f0>
   48224:	mov    %r12,%rax
   48227:	movabs $0x8888888888888889,%rdx
   48231:	mul    %rdx
   48234:	shr    $0x4,%rdx
   48238:	mov    %ecx,%r9d
   4823b:	cmpl   $0x0,0x14(%rsp)
   48240:	mov    %r12,0x68(%rsp)
   48245:	mov    %rdx,0x60(%rsp)
   4824a:	je     48550 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1030>
   48250:	mov    0x28f0(%rsp),%r10d
   48258:	mov    0xb8(%rsp),%rax
   48260:	mov    %rax,0x20(%rsp)
   48265:	mov    0xa8(%rsp),%r11
   4826d:	mov    0xb0(%rsp),%rax
   48275:	mov    %rax,0x30(%rsp)
   4827a:	mov    0xd0(%rsp),%rax
   48282:	mov    0x28e8(%rsp),%r12
   4828a:	mov    %r12,%rdi
   4828d:	shl    $0x6,%rdi
   48291:	add    %r13,%rdi
   48294:	mov    %r9,%rcx
   48297:	shl    $0x6,%rcx
   4829b:	lea    (%rax,%rcx,1),%rbx
   4829f:	add    $0x3c,%rbx
   482a3:	mov    %r10d,%r13d
   482a6:	shl    $0x8,%r13d
   482aa:	or     %r15d,%r13d
   482ad:	xor    %r14d,%r14d
   482b0:	jmp    484c4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xfa4>
   482b5:	data16 cs nopw 0x0(%rax,%rax,1)
   482c0:	mov    0x8(%r11),%rcx
   482c4:	shl    $0x7,%eax
   482c7:	mov    %r13d,(%rcx,%rax,1)
   482cb:	movzwl 0x2(%rdi),%esi
   482cf:	lea    (%r10,%r14,1),%ebp
   482d3:	add    %ebp,%esi
   482d5:	shl    $0x8,%esi
   482d8:	or     %r15d,%esi
   482db:	mov    %esi,0x4(%rcx,%rax,1)
   482df:	movzwl 0x4(%rdi),%esi
   482e3:	add    %ebp,%esi
   482e5:	shl    $0x8,%esi
   482e8:	or     %r15d,%esi
   482eb:	mov    %esi,0x8(%rcx,%rax,1)
   482ef:	movzwl 0x6(%rdi),%esi
   482f3:	add    %ebp,%esi
   482f5:	shl    $0x8,%esi
   482f8:	or     %r15d,%esi
   482fb:	mov    %esi,0xc(%rcx,%rax,1)
   482ff:	movzwl 0x8(%rdi),%esi
   48303:	add    %ebp,%esi
   48305:	shl    $0x8,%esi
   48308:	or     %r15d,%esi
   4830b:	mov    %esi,0x10(%rcx,%rax,1)
   4830f:	movzwl 0xa(%rdi),%esi
   48313:	add    %ebp,%esi
   48315:	shl    $0x8,%esi
   48318:	or     %r15d,%esi
   4831b:	mov    %esi,0x14(%rcx,%rax,1)
   4831f:	movzwl 0xc(%rdi),%esi
   48323:	add    %ebp,%esi
   48325:	shl    $0x8,%esi
   48328:	or     %r15d,%esi
   4832b:	mov    %esi,0x18(%rcx,%rax,1)
   4832f:	movzwl 0xe(%rdi),%esi
   48333:	add    %ebp,%esi
   48335:	shl    $0x8,%esi
   48338:	or     %r15d,%esi
   4833b:	mov    %esi,0x1c(%rcx,%rax,1)
   4833f:	movzwl 0x10(%rdi),%esi
   48343:	add    %ebp,%esi
   48345:	shl    $0x8,%esi
   48348:	or     %r15d,%esi
   4834b:	mov    %esi,0x20(%rcx,%rax,1)
   4834f:	movzwl 0x12(%rdi),%esi
   48353:	add    %ebp,%esi
   48355:	shl    $0x8,%esi
   48358:	or     %r15d,%esi
   4835b:	mov    %esi,0x24(%rcx,%rax,1)
   4835f:	movzwl 0x14(%rdi),%esi
   48363:	add    %ebp,%esi
   48365:	shl    $0x8,%esi
   48368:	or     %r15d,%esi
   4836b:	mov    %esi,0x28(%rcx,%rax,1)
   4836f:	movzwl 0x16(%rdi),%esi
   48373:	add    %ebp,%esi
   48375:	shl    $0x8,%esi
   48378:	or     %r15d,%esi
   4837b:	mov    %esi,0x2c(%rcx,%rax,1)
   4837f:	movzwl 0x18(%rdi),%esi
   48383:	add    %ebp,%esi
   48385:	shl    $0x8,%esi
   48388:	or     %r15d,%esi
   4838b:	mov    %esi,0x30(%rcx,%rax,1)
   4838f:	movzwl 0x1a(%rdi),%esi
   48393:	add    %ebp,%esi
   48395:	shl    $0x8,%esi
   48398:	or     %r15d,%esi
   4839b:	mov    %esi,0x34(%rcx,%rax,1)
   4839f:	movzwl 0x1c(%rdi),%esi
   483a3:	add    %ebp,%esi
   483a5:	shl    $0x8,%esi
   483a8:	or     %r15d,%esi
   483ab:	mov    %esi,0x38(%rcx,%rax,1)
   483af:	movzwl 0x1e(%rdi),%esi
   483b3:	add    %ebp,%esi
   483b5:	shl    $0x8,%esi
   483b8:	or     %r15d,%esi
   483bb:	mov    %esi,0x3c(%rcx,%rax,1)
   483bf:	movzwl 0x20(%rdi),%esi
   483c3:	add    %ebp,%esi
   483c5:	shl    $0x8,%esi
   483c8:	or     %r15d,%esi
   483cb:	mov    %esi,0x40(%rcx,%rax,1)
   483cf:	movzwl 0x22(%rdi),%esi
   483d3:	add    %ebp,%esi
   483d5:	shl    $0x8,%esi
   483d8:	or     %r15d,%esi
   483db:	mov    %esi,0x44(%rcx,%rax,1)
   483df:	movzwl 0x24(%rdi),%esi
   483e3:	add    %ebp,%esi
   483e5:	shl    $0x8,%esi
   483e8:	or     %r15d,%esi
   483eb:	mov    %esi,0x48(%rcx,%rax,1)
   483ef:	movzwl 0x26(%rdi),%esi
   483f3:	add    %ebp,%esi
   483f5:	shl    $0x8,%esi
   483f8:	or     %r15d,%esi
   483fb:	mov    %esi,0x4c(%rcx,%rax,1)
   483ff:	movzwl 0x28(%rdi),%esi
   48403:	add    %ebp,%esi
   48405:	shl    $0x8,%esi
   48408:	or     %r15d,%esi
   4840b:	mov    %esi,0x50(%rcx,%rax,1)
   4840f:	movzwl 0x2a(%rdi),%esi
   48413:	add    %ebp,%esi
   48415:	shl    $0x8,%esi
   48418:	or     %r15d,%esi
   4841b:	mov    %esi,0x54(%rcx,%rax,1)
   4841f:	movzwl 0x2c(%rdi),%esi
   48423:	add    %ebp,%esi
   48425:	shl    $0x8,%esi
   48428:	or     %r15d,%esi
   4842b:	mov    %esi,0x58(%rcx,%rax,1)
   4842f:	movzwl 0x2e(%rdi),%esi
   48433:	add    %ebp,%esi
   48435:	shl    $0x8,%esi
   48438:	or     %r15d,%esi
   4843b:	mov    %esi,0x5c(%rcx,%rax,1)
   4843f:	movzwl 0x30(%rdi),%esi
   48443:	add    %ebp,%esi
   48445:	shl    $0x8,%esi
   48448:	or     %r15d,%esi
   4844b:	mov    %esi,0x60(%rcx,%rax,1)
   4844f:	movzwl 0x32(%rdi),%esi
   48453:	add    %ebp,%esi
   48455:	shl    $0x8,%esi
   48458:	or     %r15d,%esi
   4845b:	mov    %esi,0x64(%rcx,%rax,1)
   4845f:	movzwl 0x34(%rdi),%esi
   48463:	add    %ebp,%esi
   48465:	shl    $0x8,%esi
   48468:	or     %r15d,%esi
   4846b:	mov    %esi,0x68(%rcx,%rax,1)
   4846f:	movzwl 0x36(%rdi),%esi
   48473:	add    %ebp,%esi
   48475:	shl    $0x8,%esi
   48478:	or     %r15d,%esi
   4847b:	mov    %esi,0x6c(%rcx,%rax,1)
   4847f:	movzwl 0x38(%rdi),%esi
   48483:	add    %ebp,%esi
   48485:	shl    $0x8,%esi
   48488:	or     %r15d,%esi
   4848b:	mov    %esi,0x70(%rcx,%rax,1)
   4848f:	movzwl 0x3a(%rdi),%esi
   48493:	add    %ebp,%esi
   48495:	shl    $0x8,%esi
   48498:	or     %r15d,%esi
   4849b:	mov    %esi,0x74(%rcx,%rax,1)
   4849f:	lea    (%r10,%r14,1),%eax
   484a3:	add    $0x20,%eax
   484a6:	mov    %eax,0x28f0(%rsp)
   484ad:	add    $0x20,%r14
   484b1:	inc    %r9
   484b4:	add    $0x2000,%r13d
   484bb:	dec    %rdx
   484be:	je     486cc <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11ac>
   484c4:	mov    %r9,%rax
   484c7:	shr    $0x6,%rax
   484cb:	cmp    0x20(%rsp),%rax
   484d0:	jae    48c2d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x170d>
   484d6:	mov    0x30(%rsp),%rcx
   484db:	mov    (%rcx,%rax,8),%rax
   484df:	bt     %r9,%rax
   484e3:	jae    48510 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xff0>
   484e5:	cmp    $0x1e,%r12
   484e9:	jae    48c53 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1733>
   484ef:	mov    (%rbx,%r14,2),%eax
   484f3:	and    $0xfffff,%eax
   484f8:	mov    0x10(%r11),%rcx
   484fc:	cmp    %rax,%rcx
   484ff:	ja     482c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xda0>
   48505:	jmp    48c41 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1721>
   4850a:	nopw   0x0(%rax,%rax,1)
   48510:	cmp    $0x1e,%r12
   48514:	jae    48c93 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1773>
   4851a:	movups 0x2c(%rdi),%xmm0
   4851e:	movups %xmm0,-0x10(%rbx,%r14,2)
   48524:	movdqa (%rdi),%xmm0
   48528:	movdqa 0x10(%rdi),%xmm1
   4852d:	movdqa 0x20(%rdi),%xmm2
   48532:	movdqa %xmm2,-0x1c(%rbx,%r14,2)
   48539:	movdqa %xmm1,-0x2c(%rbx,%r14,2)
   48540:	movdqa %xmm0,-0x3c(%rbx,%r14,2)
   48547:	mov    %r13d,(%rbx,%r14,2)
   4854b:	jmp    4849f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf7f>
   48550:	mov    0x28e8(%rsp),%rdi
   48558:	cmp    $0x1d,%rdi
   4855c:	ja     48d28 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1808>
   48562:	shl    $0x6,%rdi
   48566:	add    %r13,%rdi
   48569:	mov    0x28f0(%rsp),%eax
   48570:	mov    0xd0(%rsp),%rcx
   48578:	shl    $0x6,%r9
   4857c:	mov    %eax,%r14d
   4857f:	shl    $0x8,%r14d
   48583:	or     %r15d,%r14d
   48586:	lea    -0x1e(%r12),%rsi
   4858b:	cmp    $0x1e,%rsi
   4858f:	jae    485fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10dd>
   48591:	xor    %esi,%esi
   48593:	jmp    4868a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x116a>
   48598:	shr    $1,%r9
   4859b:	inc    %r9
   4859e:	mov    %r9,%r10
   485a1:	and    $0xfffffffffffffff8,%r10
   485a5:	lea    (%rdx,%r10,2),%r8
   485a9:	movd   %ecx,%xmm0
   485ad:	pshuflw $0x0,%xmm0,%xmm0
   485b2:	add    %r13,%r11
   485b5:	add    $0x8,%r11
   485b9:	xor    %ebx,%ebx
   485bb:	nopl   0x0(%rax,%rax,1)
   485c0:	movq   -0x8(%r11,%rbx,2),%xmm1
   485c7:	movq   (%r11,%rbx,2),%xmm2
   485cd:	paddw  %xmm0,%xmm1
   485d1:	paddw  %xmm0,%xmm2
   485d5:	punpcklqdq %xmm2,%xmm1
   485d9:	movdqu %xmm1,(%rax,%rbx,2)
   485de:	add    $0x8,%rbx
   485e2:	cmp    %rbx,%r10
   485e5:	jne    485c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10a0>
   485e7:	cmp    %r10,%r9
   485ea:	mov    0x80(%rsp),%rbx
   485f2:	jne    48151 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc31>
   485f8:	jmp    48179 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc59>
   485fd:	mov    %rdx,%r10
   48600:	movabs $0xffffffffffffffe,%rsi
   4860a:	and    %rsi,%r10
   4860d:	lea    (%rcx,%r9,1),%r11
   48611:	add    $0x7c,%r11
   48615:	xor    %esi,%esi
   48617:	nopw   0x0(%rax,%rax,1)
   48620:	movdqu 0x2c(%rdi),%xmm0
   48625:	movdqu %xmm0,-0x50(%r11)
   4862b:	movdqa (%rdi),%xmm1
   4862f:	movdqa 0x10(%rdi),%xmm2
   48634:	movdqa 0x20(%rdi),%xmm3
   48639:	movdqa %xmm3,-0x5c(%r11)
   4863f:	movdqa %xmm2,-0x6c(%r11)
   48645:	movdqa %xmm1,-0x7c(%r11)
   4864b:	mov    %r14d,-0x40(%r11)
   4864f:	add    $0x2,%rsi
   48653:	movdqa %xmm1,-0x3c(%r11)
   48659:	movdqa %xmm2,-0x2c(%r11)
   4865f:	movdqa %xmm3,-0x1c(%r11)
   48665:	movdqu %xmm0,-0x10(%r11)
   4866b:	lea    0x2000(%r14),%ebx
   48672:	mov    %ebx,(%r11)
   48675:	add    $0x4000,%r14d
   4867c:	sub    $0xffffffffffffff80,%r11
   48680:	cmp    %rsi,%r10
   48683:	jne    48620 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1100>
   48685:	test   $0x1,%dl
   48688:	je     486be <<c30walk::ctor2su::Ctor2su>::build::<false>+0x119e>
   4868a:	add    %r9,%rcx
   4868d:	shl    $0x6,%rsi
   48691:	movups 0x2c(%rdi),%xmm0
   48695:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4869a:	movdqa (%rdi),%xmm0
   4869e:	movdqa 0x10(%rdi),%xmm1
   486a3:	movdqa 0x20(%rdi),%xmm2
   486a8:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   486ae:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   486b4:	movdqa %xmm0,(%rcx,%rsi,1)
   486b9:	mov    %r14d,0x3c(%rcx,%rsi,1)
   486be:	mov    %edx,%ecx
   486c0:	shl    $0x5,%ecx
   486c3:	add    %ecx,%eax
   486c5:	mov    %eax,0x28f0(%rsp)
   486cc:	mov    0x60(%rsp),%rdx
   486d1:	mov    %rdx,%rax
   486d4:	shl    $0x5,%rax
   486d8:	lea    (%rdx,%rdx,1),%rcx
   486dc:	sub    %rax,%rcx
   486df:	mov    0x68(%rsp),%r12
   486e4:	add    %rcx,%r12
   486e7:	lea    (%rdx,%rdx,1),%eax
   486ea:	shl    $0x5,%edx
   486ed:	sub    %eax,%edx
   486ef:	add    %edx,%r8d
   486f2:	mov    0x80(%rsp),%rbx
   486fa:	mov    %r8d,0x18(%rbx)
   486fe:	test   %r12,%r12
   48701:	mov    $0x88888889,%r14d
   48707:	lea    -0x2adce(%rip),%r13        # 1d940 <c30walk::ctor2su::PATTERNS>
   4870e:	je     48787 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1267>
   48710:	mov    0x1c(%rbx),%ecx
   48713:	mov    %r8d,%edi
   48716:	imul   %r14,%rdi
   4871a:	mov    %rdi,%rdx
   4871d:	shr    $0x24,%rdx
   48721:	mov    %edx,%eax
   48723:	shl    $0x5,%eax
   48726:	sub    %edx,%eax
   48728:	sub    %edx,%eax
   4872a:	lea    0x1e(%rax),%esi
   4872d:	cmp    %ecx,%esi
   4872f:	cmovb  %esi,%ecx
   48732:	shr    $0x2a,%rdi
   48736:	mov    0xb8(%rsp),%rsi
   4873e:	cmp    %rsi,%rdi
   48741:	jae    48d0b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17eb>
   48747:	mov    %rdx,%rsi
   4874a:	shl    $0x6,%rsi
   4874e:	add    0xd0(%rsp),%rsi
   48756:	mov    0xb0(%rsp),%r9
   4875e:	mov    (%r9,%rdi,8),%rdi
   48762:	bt     %rdx,%rdi
   48766:	jb     480b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb90>
   4876c:	mov    $0xffffffff,%edi
   48771:	jmp    480bc <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb9c>
   48776:	cs nopw 0x0(%rax,%rax,1)
   48780:	mov    0x8(%rsp),%rdi
   48785:	jmp    48794 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1274>
   48787:	mov    0x8(%rsp),%rdi
   4878c:	lea    0xe8(%rsp),%r8
   48794:	mov    0x18(%rsp),%r9
   48799:	mov    0x38(%rsp),%r15
   4879e:	mov    0x4c(%rsp),%edx
   487a2:	mov    0x40(%rsp),%rbx
   487a7:	mov    %r9,%r12
   487aa:	cmp    $0xf,%r9
   487ae:	ja     47e5d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93d>
   487b4:	jmp    487cb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12ab>
   487b6:	cs nopw 0x0(%rax,%rax,1)
   487c0:	mov    %r12,%r9
   487c3:	lea    0xe8(%rsp),%r8
   487cb:	test   %r9,%r9
   487ce:	je     47d16 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7f6>
   487d4:	mov    0x28(%rsp),%r12
   487d9:	sub    %r9,%r12
   487dc:	mov    0x28e8(%rsp),%rax
   487e4:	mov    $0x1e,%ebx
   487e9:	sub    %rax,%rbx
   487ec:	cmp    %r9,%rbx
   487ef:	mov    %r9,0x18(%rsp)
   487f4:	cmovae %r9,%rbx
   487f8:	mov    %rbx,%r13
   487fb:	add    %r12,%r13
   487fe:	jb     48c67 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1747>
   48804:	cmp    %rdi,%r13
   48807:	ja     48c67 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1747>
   4880d:	mov    %r15,0x38(%rsp)
   48812:	mov    0x28f0(%rsp),%r14d
   4881a:	cmp    $0x1e,%rax
   4881e:	jne    488af <<c30walk::ctor2su::Ctor2su>::build::<false>+0x138f>
   48824:	cmp    %r13,0x28(%rsp)
   48829:	jb     48c7c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x175c>
   4882f:	mov    0x18(%rsp),%r9
   48834:	jne    48919 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13f9>
   4883a:	mov    0x28e8(%rsp),%rax
   48842:	mov    0x28f0(%rsp),%ecx
   48849:	cmp    $0x1,%r9
   4884d:	je     47ce7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7c7>
   48853:	mov    %r9d,%edx
   48856:	and    $0xe,%edx
   48859:	mov    0x38(%rsp),%r15
   4885e:	xchg   %ax,%ax
   48860:	lea    0x1(%rax),%rsi
   48864:	add    $0xffffffffffffffe3,%rax
   48868:	xor    %edi,%edi
   4886a:	cmp    $0x1e,%rsi
   4886e:	setae  %dil
   48872:	cmovb  %rsi,%rax
   48876:	lea    (%rcx,%rdi,2),%ecx
   48879:	lea    0x1(%rax),%rsi
   4887d:	add    $0xffffffffffffffe3,%rax
   48881:	xor    %edi,%edi
   48883:	cmp    $0x1e,%rsi
   48887:	setae  %dil
   4888b:	cmovb  %rsi,%rax
   4888f:	lea    (%rcx,%rdi,2),%ecx
   48892:	add    $0x2,%ecx
   48895:	add    $0xfffffffffffffffe,%rdx
   48899:	jne    48860 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1340>
   4889b:	test   $0x1,%r9b
   4889f:	mov    0x8(%rsp),%rdi
   488a4:	jne    47cec <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7cc>
   488aa:	jmp    47d07 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7e7>
   488af:	add    0x50(%rsp),%r12
   488b4:	xor    %r15d,%r15d
   488b7:	nopw   0x0(%rax,%rax,1)
   488c0:	movzbl (%r12,%r15,1),%esi
   488c5:	lea    (%rsi,%rsi,4),%rcx
   488c9:	lea    (%r8,%rcx,8),%rbp
   488cd:	mov    (%r8,%rcx,8),%rax
   488d1:	cmp    0x8(%r8,%rcx,8),%rax
   488d6:	je     488f1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13d1>
   488d8:	lea    (%r14,%r15,1),%rcx
   488dc:	mov    %cx,(%rax)
   488df:	addq   $0x2,0x0(%rbp)
   488e4:	inc    %r15
   488e7:	cmp    %r15,%rbx
   488ea:	jne    488c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13a0>
   488ec:	jmp    48824 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1304>
   488f1:	lea    0xa8(%rsp),%rdi
   488f9:	call   96ec0 <<c30walk::ctor2su::W<false>>::refill>
   488fe:	lea    0xe8(%rsp),%r8
   48906:	test   %al,%al
   48908:	je     489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   4890e:	mov    0x0(%rbp),%rax
   48912:	mov    0x8(%rsp),%rdi
   48917:	jmp    488d8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13b8>
   48919:	mov    0x50(%rsp),%rax
   4891e:	mov    0x28(%rsp),%rcx
   48923:	lea    (%rax,%rcx,1),%r15
   48927:	lea    (%r14,%rbx,1),%ebp
   4892b:	add    $0x2,%ebp
   4892e:	sub    %r9,%rbx
   48931:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48940:	movzbl (%r15,%rbx,1),%esi
   48945:	lea    (%rsi,%rsi,4),%rcx
   48949:	lea    (%r8,%rcx,8),%r14
   4894d:	mov    (%r8,%rcx,8),%rax
   48951:	cmp    0x8(%r8,%rcx,8),%rax
   48956:	je     4896b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x144b>
   48958:	mov    %bp,(%rax)
   4895b:	addq   $0x2,(%r14)
   4895f:	inc    %ebp
   48961:	inc    %rbx
   48964:	jne    48940 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1420>
   48966:	jmp    4883a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x131a>
   4896b:	lea    0xa8(%rsp),%rdi
   48973:	call   96ec0 <<c30walk::ctor2su::W<false>>::refill>
   48978:	lea    0xe8(%rsp),%r8
   48980:	test   %al,%al
   48982:	je     489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48984:	mov    (%r14),%rax
   48987:	mov    0x8(%rsp),%rdi
   4898c:	mov    0x18(%rsp),%r9
   48991:	jmp    48958 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1438>
   48993:	mov    %r12,%rdi
   48996:	call   fea70 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   4899b:	jmp    47da1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x881>
   489a0:	mov    $0x84,%eax
   489a5:	lea    0xe8(%rsp),%r8
   489ad:	nopl   (%rax)
   489b0:	lea    0x50(%rax),%rcx
   489b4:	cmp    $0x28d4,%rcx
   489bb:	je     48a34 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1514>
   489bd:	mov    0x64(%rsp,%rax,1),%rdx
   489c2:	sub    0x74(%rsp,%rax,1),%rdx
   489c7:	shr    $1,%rdx
   489ca:	add    0x7c(%rsp,%rax,1),%edx
   489ce:	cmp    0x80(%rsp,%rax,1),%edx
   489d5:	jne    489fd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   489d7:	mov    0x8c(%rsp,%rax,1),%rdx
   489df:	sub    0x9c(%rsp,%rax,1),%rdx
   489e7:	shr    $1,%rdx
   489ea:	add    0xa4(%rsp,%rax,1),%edx
   489f1:	cmp    0xa8(%rsp,%rax,1),%edx
   489f8:	mov    %rcx,%rax
   489fb:	je     489b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1490>
   489fd:	mov    0x5530(%rsp),%rax
   48a05:	movq   $0x0,0x10(%rax)
   48a0d:	xor    %eax,%eax
   48a0f:	add    $0x54f8,%rsp
   48a16:	pop    %rbx
   48a17:	pop    %r12
   48a19:	pop    %r13
   48a1b:	pop    %r14
   48a1d:	pop    %r15
   48a1f:	pop    %rbp
   48a20:	ret
   48a21:	lea    0x1001a0(%rip),%rcx        # 148bc8 <__frame_dummy_init_array_entry+0x2338>
   48a28:	mov    $0x1e,%edx
   48a2d:	xor    %edi,%edi
   48a2f:	call   f0c50 <core::slice::index::slice_index_fail>
   48a34:	xor    %r14d,%r14d
   48a37:	lea    0xa8(%rsp),%r15
   48a3f:	mov    0x88(%rsp),%r12
   48a47:	mov    0x8(%rsp),%rdx
   48a4c:	mov    0x78(%rsp),%r13
   48a51:	jmp    48a8b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x156b>
   48a53:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48a60:	sub    %rcx,%rax
   48a63:	inc    %r14
   48a66:	shr    $1,%rax
   48a69:	add    %eax,0x18(%r8)
   48a6d:	pxor   %xmm0,%xmm0
   48a71:	movdqu %xmm0,(%r8)
   48a76:	movq   $0x0,0x10(%r8)
   48a7e:	add    $0x28,%r8
   48a82:	cmp    $0x100,%r14
   48a89:	je     48ab6 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1596>
   48a8b:	mov    (%r8),%rcx
   48a8e:	mov    %rcx,%rax
   48a91:	cmp    0x10(%r8),%rcx
   48a95:	je     48a60 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1540>
   48a97:	mov    %r15,%rdi
   48a9a:	mov    %r14,%rsi
   48a9d:	mov    %r8,%rbx
   48aa0:	call   968c0 <<c30walk::ctor2su::W<false>>::finish>
   48aa5:	mov    %rbx,%r8
   48aa8:	mov    0x8(%rsp),%rdx
   48aad:	mov    (%rbx),%rax
   48ab0:	mov    0x10(%rbx),%rcx
   48ab4:	jmp    48a60 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1540>
   48ab6:	xor    %ecx,%ecx
   48ab8:	mov    0x8(%r12),%rbx
   48abd:	test   %r13,%r13
   48ac0:	setne  %al
   48ac3:	je     48b07 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15e7>
   48ac5:	mov    %al,%cl
   48ac7:	mov    %r13,%rax
   48aca:	sub    %rcx,%rax
   48acd:	imul   $0xffffffffffffffe2,%rax,%rcx
   48ad1:	add    %rdx,%rcx
   48ad4:	cmp    $0x1e,%rcx
   48ad8:	jae    48b07 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15e7>
   48ada:	lea    (%rbx,%r13,4),%rcx
   48ade:	lea    (%rcx,%rdx,2),%rdi
   48ae2:	add    $0xfffffffffffffffc,%rdi
   48ae6:	lea    (%rax,%rax,1),%rcx
   48aea:	shl    $0x5,%rax
   48aee:	sub    %rcx,%rax
   48af1:	sub    %rdx,%rax
   48af4:	lea    0x3c(,%rax,2),%rdx
   48afc:	mov    $0xff,%esi
   48b01:	call   *0x104ae9(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   48b07:	shl    $0x6,%r13
   48b0b:	pcmpeqd %xmm0,%xmm0
   48b0f:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   48b16:	movdqa %xmm0,0x20(%rbx,%r13,1)
   48b1d:	movdqa %xmm0,0x10(%rbx,%r13,1)
   48b24:	movdqa %xmm0,(%rbx,%r13,1)
   48b2a:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   48b33:	mov    0x70(%rsp),%rax
   48b38:	mov    0x8(%rax),%rdx
   48b3c:	mov    0x10(%rax),%rcx
   48b40:	mov    0x5530(%rsp),%rax
   48b48:	mov    0x8(%rax),%r8
   48b4c:	mov    0x10(%rax),%r9
   48b50:	lea    0xa8(%rsp),%rdi
   48b58:	mov    %rbx,%rsi
   48b5b:	call   a5850 <c30walk::ctor::canonicalise>
   48b60:	mov    0x90(%rsp),%rax
   48b68:	mov    %rax,0x10(%r12)
   48b6d:	mov    $0x1,%al
   48b6f:	jmp    48a0f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ef>
   48b74:	mov    %rdi,%r12
   48b77:	test   %rax,%rax
   48b7a:	je     48b8c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x166c>
   48b7c:	mov    0x8(%r9),%rdi
   48b80:	mov    %r9,%r15
   48b83:	call   *0x104a77(%rip)        # 14d600 <free@GLIBC_2.2.5>
   48b89:	mov    %r15,%r9
   48b8c:	movq   $0x0,(%r9)
   48b93:	movq   $0x40,0x8(%r9)
   48b9b:	movq   $0x0,0x10(%r9)
   48ba3:	mov    $0x40,%edx
   48ba8:	mov    %r9,%rdi
   48bab:	lea    0x1(%r13),%rsi
   48baf:	mov    %r9,%r15
   48bb2:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48bb7:	mov    %r15,%r9
   48bba:	cmp    $0xffffffffffffffff,%rax
   48bbe:	mov    %r12,%rdi
   48bc1:	je     47758 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x238>
   48bc7:	jmp    48c22 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1702>
   48bc9:	mov    %rdi,%r12
   48bcc:	mov    %r9,%r15
   48bcf:	test   %rax,%rax
   48bd2:	je     48be6 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16c6>
   48bd4:	mov    0x5530(%rsp),%rax
   48bdc:	mov    0x8(%rax),%rdi
   48be0:	call   *0x104a1a(%rip)        # 14d600 <free@GLIBC_2.2.5>
   48be6:	mov    0x5530(%rsp),%rdi
   48bee:	movq   $0x0,(%rdi)
   48bf5:	movq   $0x40,0x8(%rdi)
   48bfd:	movq   $0x0,0x10(%rdi)
   48c05:	mov    $0x80,%edx
   48c0a:	mov    %rbx,%rsi
   48c0d:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48c12:	cmp    $0xffffffffffffffff,%rax
   48c16:	mov    %r15,%r9
   48c19:	mov    %r12,%rdi
   48c1c:	je     4776c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x24c>
   48c22:	mov    %rax,%rdi
   48c25:	mov    %rdx,%rsi
   48c28:	call   ee3db <alloc::raw_vec::handle_error>
   48c2d:	lea    0xfdee4(%rip),%rdx        # 146b18 <__frame_dummy_init_array_entry+0x288>
   48c34:	mov    %rax,%rdi
   48c37:	mov    0x20(%rsp),%rsi
   48c3c:	call   efe73 <core::panicking::panic_bounds_check>
   48c41:	lea    0x100040(%rip),%rdx        # 148c88 <__frame_dummy_init_array_entry+0x23f8>
   48c48:	mov    %rax,%rdi
   48c4b:	mov    %rcx,%rsi
   48c4e:	call   efe73 <core::panicking::panic_bounds_check>
   48c53:	lea    0x100016(%rip),%rdx        # 148c70 <__frame_dummy_init_array_entry+0x23e0>
   48c5a:	mov    $0x1e,%esi
   48c5f:	mov    %r12,%rdi
   48c62:	call   efe73 <core::panicking::panic_bounds_check>
   48c67:	lea    0xfff8a(%rip),%rcx        # 148bf8 <__frame_dummy_init_array_entry+0x2368>
   48c6e:	mov    %rdi,%rdx
   48c71:	mov    %r12,%rdi
   48c74:	mov    %r13,%rsi
   48c77:	call   f0c50 <core::slice::index::slice_index_fail>
   48c7c:	lea    0xfff5d(%rip),%rcx        # 148be0 <__frame_dummy_init_array_entry+0x2350>
   48c83:	mov    %rdi,%rdx
   48c86:	mov    %r13,%rdi
   48c89:	mov    0x28(%rsp),%rsi
   48c8e:	call   f0c50 <core::slice::index::slice_index_fail>
   48c93:	lea    0xfffbe(%rip),%rdx        # 148c58 <__frame_dummy_init_array_entry+0x23c8>
   48c9a:	mov    $0x1e,%esi
   48c9f:	mov    %r12,%rdi
   48ca2:	call   efe73 <core::panicking::panic_bounds_check>
   48ca7:	mov    $0x10,%edx
   48cac:	mov    $0x4,%ecx
   48cb1:	mov    $0x400,%r8d
   48cb7:	mov    %rdi,%r15
   48cba:	mov    %r13,%rdi
   48cbd:	xor    %esi,%esi
   48cbf:	mov    %r9,%rbx
   48cc2:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48cc7:	mov    %r15,%rdi
   48cca:	mov    %rbx,%r9
   48ccd:	jmp    477f9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2d9>
   48cd2:	mov    %r9,%r13
   48cd5:	mov    $0x8,%ecx
   48cda:	mov    $0x8,%r8d
   48ce0:	mov    %rdi,%r15
   48ce3:	xor    %esi,%esi
   48ce5:	mov    %rbx,%rdx
   48ce8:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48ced:	mov    %r15,%rdi
   48cf0:	mov    0x10(%r15),%r15
   48cf4:	mov    0x8(%rdi),%r12
   48cf8:	lea    (%r12,%r15,8),%rdi
   48cfc:	cmp    $0x1,%rbx
   48d00:	jne    477a5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x285>
   48d06:	jmp    477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   48d0b:	lea    0xfde06(%rip),%rdx        # 146b18 <__frame_dummy_init_array_entry+0x288>
   48d12:	call   efe73 <core::panicking::panic_bounds_check>
   48d17:	lea    0xffe92(%rip),%rdx        # 148bb0 <__frame_dummy_init_array_entry+0x2320>
   48d1e:	mov    $0x1e,%esi
   48d23:	call   efe73 <core::panicking::panic_bounds_check>
   48d28:	lea    0xfff11(%rip),%rdx        # 148c40 <__frame_dummy_init_array_entry+0x23b0>
   48d2f:	mov    $0x1e,%esi
   48d34:	call   efe73 <core::panicking::panic_bounds_check>
   48d39:	lea    0xffdf8(%rip),%rdx        # 148b38 <__frame_dummy_init_array_entry+0x22a8>
   48d40:	call   efe73 <core::panicking::panic_bounds_check>
   48d45:	lea    0xfddcc(%rip),%rdx        # 146b18 <__frame_dummy_init_array_entry+0x288>
   48d4c:	mov    %r13,%rdi
   48d4f:	call   efe73 <core::panicking::panic_bounds_check>
