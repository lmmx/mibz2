
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
   47581:	mov    %rsi,0x60(%rsp)
   47586:	mov    0x5530(%rsp),%rax
   4758e:	movq   $0x0,0x10(%r9)
   47596:	movq   $0x0,0x10(%rax)
   4759e:	mov    %rdx,0x8(%rsp)
   475a3:	cmp    $0xeffe2,%rdx
   475aa:	ja     489dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14bd>
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
   4762e:	jne    489dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14bd>
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
   47747:	mov    %rcx,0x88(%rsp)
   4774f:	cmp    %r13,%rax
   47752:	jbe    48b3c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x161c>
   47758:	mov    0x5530(%rsp),%rax
   47760:	mov    (%rax),%rax
   47763:	cmp    %rbx,%rax
   47766:	jb     48b91 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1671>
   4776c:	movq   $0x0,0x10(%rdi)
   47774:	lea    0x3f(%r13),%rbx
   47778:	shr    $0x6,%rbx
   4777c:	mov    %rdi,0x70(%rsp)
   47781:	mov    %r13,0x78(%rsp)
   47786:	je     477db <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2bb>
   47788:	cmp    (%rdi),%rbx
   4778b:	ja     48c9a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x177a>
   47791:	mov    %r9,%r13
   47794:	xor    %r15d,%r15d
   47797:	mov    0x8(%rdi),%r12
   4779b:	lea    (%r12,%r15,8),%rdi
   4779f:	cmp    $0x1,%rbx
   477a3:	je     477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   477a5:	lea    -0x8(,%rbx,8),%rdx
   477ad:	xor    %esi,%esi
   477af:	call   *0x105dfb(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
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
   477f3:	jbe    48c6f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x174f>
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
   478fc:	mov    %r9,0x80(%rsp)
   47904:	mov    0x8(%r9),%rbx
   47908:	mov    0x8(%rdi),%rbp
   4790c:	mov    0x10(%rdi),%r12
   47910:	lea    0xe8(%rsp),%rdi
   47918:	lea    0x2cf8(%rsp),%rsi
   47920:	mov    $0x2800,%edx
   47925:	call   *0x105ca5(%rip)        # 14d5d0 <memcpy@GLIBC_2.14>
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
   479c3:	jmp    479e1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4c1>
   479c5:	data16 cs nopw 0x0(%rax,%rax,1)
   479d0:	add    $0x4,%r15
   479d4:	cmp    $0x400,%r15
   479db:	je     47b15 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x5f5>
   479e1:	mov    %r13,%rax
   479e4:	mov    (%r14,%r15,1),%r13d
   479e8:	add    %rax,%r13
   479eb:	cmp    %r11,%r13
   479ee:	setae  %r8b
   479f2:	mov    %r13,%rax
   479f5:	movabs $0x8888888888888889,%rdx
   479ff:	mul    %rdx
   47a02:	mov    %rdx,%r12
   47a05:	shr    $0x4,%r12
   47a09:	mov    %r13,%rax
   47a0c:	imul   %r10,%rax
   47a10:	ror    $1,%rax
   47a13:	cmp    %rdi,%rax
   47a16:	setb   %al
   47a19:	cmp    %rcx,%r12
   47a1c:	sete   %dl
   47a1f:	or     %r8b,%dl
   47a22:	or     %al,%dl
   47a24:	jne    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47a26:	mov    0x5530(%rsp),%rdi
   47a2e:	mov    0x10(%rdi),%rbp
   47a32:	cmp    (%rdi),%rbp
   47a35:	je     47add <<c30walk::ctor2su::Ctor2su>::build::<false>+0x5bd>
   47a3b:	mov    0x8(%rdi),%rax
   47a3f:	mov    %rbp,%rcx
   47a42:	shl    $0x7,%rcx
   47a46:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47a4c:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47a52:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47a58:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47a5e:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47a64:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47a6a:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47a70:	movdqa %xmm0,(%rax,%rcx,1)
   47a75:	mov    %rbp,%rax
   47a78:	inc    %rax
   47a7b:	mov    %rax,0x10(%rdi)
   47a7f:	mov    %r12,%rax
   47a82:	shl    $0x6,%rax
   47a86:	or     $0x10000000,%ebp
   47a8c:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   47a92:	movdqa %xmm1,0x20(%rbx,%rax,1)
   47a98:	movdqa %xmm1,0x10(%rbx,%rax,1)
   47a9e:	movdqa %xmm1,(%rbx,%rax,1)
   47aa3:	mov    %ebp,0x3c(%rbx,%rax,1)
   47aa7:	mov    %r12,%rdi
   47aaa:	shr    $0x6,%rdi
   47aae:	cmp    %rsi,%rdi
   47ab1:	jae    48d01 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17e1>
   47ab7:	mov    $0x1,%eax
   47abc:	mov    %r12d,%ecx
   47abf:	shl    %cl,%rax
   47ac2:	or     %rax,(%r9,%rdi,8)
   47ac6:	mov    %r12,%rcx
   47ac9:	mov    0x8(%rsp),%r11
   47ace:	movabs $0x888888888888889,%rdi
   47ad8:	jmp    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47add:	mov    %rsi,0x30(%rsp)
   47ae2:	mov    %r9,0x50(%rsp)
   47ae7:	call   9c130 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47aec:	pcmpeqd %xmm1,%xmm1
   47af0:	pxor   %xmm0,%xmm0
   47af4:	movabs $0xeeeeeeeeeeeeeeef,%r10
   47afe:	mov    0x50(%rsp),%r9
   47b03:	mov    0x30(%rsp),%rsi
   47b08:	mov    0x5530(%rsp),%rdi
   47b10:	jmp    47a3b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x51b>
   47b15:	cmpl   $0x0,0x14(%rsp)
   47b1a:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47b20:	mov    0xd8(%rsp),%r8
   47b28:	test   %r8,%r8
   47b2b:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47b31:	mov    %r11,%r9
   47b34:	shl    $0x20,%r9
   47b38:	mov    0x14(%rsp),%r10d
   47b3d:	mov    0xa8(%rsp),%rdi
   47b45:	mov    0xd0(%rsp),%rbp
   47b4d:	xor    %r15d,%r15d
   47b50:	movabs $0xff51afd7ed558ccd,%rbx
   47b5a:	pxor   %xmm0,%xmm0
   47b5e:	pcmpeqd %xmm1,%xmm1
   47b62:	jmp    47bf0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6d0>
   47b67:	mov    0x8(%rdi),%rax
   47b6b:	mov    %rbx,%rcx
   47b6e:	shl    $0x7,%rcx
   47b72:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47b78:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47b7e:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47b84:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47b8a:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47b90:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47b96:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47b9c:	movdqa %xmm0,(%rax,%rcx,1)
   47ba1:	mov    %rbx,%rax
   47ba4:	inc    %rax
   47ba7:	mov    %rax,0x10(%rdi)
   47bab:	or     $0x10000000,%ebx
   47bb1:	movdqu %xmm1,0x2c(%rbp)
   47bb6:	movdqa %xmm1,0x20(%rbp)
   47bbb:	movdqa %xmm1,0x10(%rbp)
   47bc0:	movdqa %xmm1,0x0(%rbp)
   47bc5:	mov    %ebx,0x3c(%rbp)
   47bc8:	or     %r14,(%r12,%r13,8)
   47bcc:	movabs $0xff51afd7ed558ccd,%rbx
   47bd6:	cs nopw 0x0(%rax,%rax,1)
   47be0:	inc    %r15
   47be3:	add    $0x40,%rbp
   47be7:	cmp    %r15,%r8
   47bea:	je     47cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47bf0:	mov    %r15,%r13
   47bf3:	shr    $0x6,%r13
   47bf7:	mov    0xb8(%rsp),%rsi
   47bff:	cmp    %rsi,%r13
   47c02:	jae    48d0d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17ed>
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
   47c8e:	jne    47b67 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x647>
   47c94:	mov    %r8,0x30(%rsp)
   47c99:	mov    %r9,0x50(%rsp)
   47c9e:	mov    %r10,0x48(%rsp)
   47ca3:	mov    %rdi,0x68(%rsp)
   47ca8:	call   9c130 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47cad:	pcmpeqd %xmm1,%xmm1
   47cb1:	pxor   %xmm0,%xmm0
   47cb5:	mov    0x68(%rsp),%rdi
   47cba:	mov    0x48(%rsp),%r10
   47cbf:	mov    0x50(%rsp),%r9
   47cc4:	mov    0x30(%rsp),%r8
   47cc9:	mov    0x8(%rsp),%r11
   47cce:	jmp    47b67 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x647>
   47cd3:	test   %r11,%r11
   47cd6:	je     4897b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x145b>
   47cdc:	movq   $0x0,0x20(%rsp)
   47ce5:	jmp    47d29 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x809>
   47ce7:	mov    0x28(%rsp),%r14
   47cec:	mov    0x18(%rsp),%r15
   47cf1:	lea    0x1(%rax),%rdx
   47cf5:	add    $0xffffffffffffffe3,%rax
   47cf9:	xor    %esi,%esi
   47cfb:	cmp    $0x1e,%rdx
   47cff:	setae  %sil
   47d03:	cmovb  %rdx,%rax
   47d07:	lea    (%rcx,%rsi,2),%ecx
   47d0a:	inc    %ecx
   47d0c:	mov    %rax,0x28e8(%rsp)
   47d14:	mov    %ecx,0x28f0(%rsp)
   47d1b:	cmp    %rdi,0x90(%rsp)
   47d23:	jae    4897b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x145b>
   47d29:	mov    0x20(%rsp),%r12
   47d2e:	mov    $0x400,%edx
   47d33:	lea    0x28f8(%rsp),%rdi
   47d3b:	xor    %esi,%esi
   47d3d:	call   *0x10586d(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
   47d43:	lea    0x128(%rsp),%rax
   47d4b:	xor    %ecx,%ecx
   47d4d:	nopl   (%rax)
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
   47d8a:	mov    0xc0(%rsp),%r13
   47d92:	mov    0x10(%r13),%rbx
   47d96:	cmp    0x0(%r13),%rbx
   47d9a:	je     4896e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x144e>
   47da0:	mov    %rbx,%rdi
   47da3:	shl    $0xa,%rdi
   47da7:	add    0x8(%r13),%rdi
   47dab:	mov    $0x400,%edx
   47db0:	lea    0x28f8(%rsp),%rsi
   47db8:	call   *0x105812(%rip)        # 14d5d0 <memcpy@GLIBC_2.14>
   47dbe:	inc    %rbx
   47dc1:	lea    0xf000(%r12),%rax
   47dc9:	mov    0x8(%rsp),%rdi
   47dce:	cmp    %rdi,%rax
   47dd1:	mov    %rdi,%rcx
   47dd4:	mov    %rax,0x90(%rsp)
   47ddc:	cmovb  %rax,%rcx
   47de0:	mov    %rbx,0x10(%r13)
   47de4:	mov    %r12,%r8
   47de7:	mov    %rcx,0x20(%rsp)
   47dec:	mov    %rcx,%r12
   47def:	sub    %r8,%r12
   47df2:	cmp    $0x10,%r12
   47df6:	jb     48790 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1270>
   47dfc:	add    0x60(%rsp),%r8
   47e01:	xor    %eax,%eax
   47e03:	lea    0xe8(%rsp),%r13
   47e0b:	jmp    47e62 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x942>
   47e0d:	nopl   (%rax)
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
   47e49:	xor    %eax,%eax
   47e4b:	mov    0x40(%rsp),%rcx
   47e50:	mov    %rcx,%r12
   47e53:	cmp    $0xf,%rcx
   47e57:	mov    0x38(%rsp),%r8
   47e5c:	jbe    48793 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1273>
   47e62:	mov    %r15,%rcx
   47e65:	test   $0x1,%al
   47e67:	jne    47e70 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x950>
   47e69:	mov    (%r8),%rcx
   47e6c:	mov    0x8(%r8),%r14
   47e70:	add    $0x10,%r8
   47e74:	lea    -0x10(%r12),%rdx
   47e79:	xor    %rcx,%r14
   47e7c:	mov    %rcx,%rax
   47e7f:	rol    $0x8,%rax
   47e83:	xor    %rcx,%rax
   47e86:	or     %r14,%rax
   47e89:	jne    47ee0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x9c0>
   47e8b:	cmp    $0x10,%rdx
   47e8f:	jb     47f90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa70>
   47e95:	data16 cs nopw 0x0(%rax,%rax,1)
   47ea0:	mov    (%r8),%rsi
   47ea3:	mov    0x8(%r8),%rax
   47ea7:	mov    $0x1,%r9b
   47eaa:	cmp    %rcx,%rsi
   47ead:	jne    4801b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xafb>
   47eb3:	cmp    %rcx,%rax
   47eb6:	jne    4801b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xafb>
   47ebc:	add    $0xfffffffffffffff0,%rdx
   47ec0:	add    $0x10,%r8
   47ec4:	cmp    $0xf,%rdx
   47ec8:	ja     47ea0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x980>
   47eca:	mov    %r8,0x38(%rsp)
   47ecf:	xor    %r9d,%r9d
   47ed2:	mov    %rcx,0x18(%rsp)
   47ed7:	jmp    4802a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb0a>
   47edc:	nopl   0x0(%rax)
   47ee0:	mov    %r8,0x38(%rsp)
   47ee5:	mov    %rdx,0x40(%rsp)
   47eea:	mov    0x20(%rsp),%rbx
   47eef:	sub    %r12,%rbx
   47ef2:	mov    0x28e8(%rsp),%rax
   47efa:	lea    -0x1e(%rax),%r12
   47efe:	neg    %r12
   47f01:	cmp    $0x10,%r12
   47f05:	mov    $0x10,%r14d
   47f0b:	cmovb  %r12,%r14
   47f0f:	add    0x60(%rsp),%rbx
   47f14:	mov    0x28f0(%rsp),%r15d
   47f1c:	cmp    $0x1e,%rax
   47f20:	je     47fb2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa92>
   47f26:	xor    %r13d,%r13d
   47f29:	nopl   0x0(%rax)
   47f30:	movzbl (%rbx,%r13,1),%esi
   47f35:	lea    (%rsi,%rsi,4),%rcx
   47f39:	lea    0xe8(%rsp),%rdx
   47f41:	lea    (%rdx,%rcx,8),%rbp
   47f45:	mov    (%rdx,%rcx,8),%rax
   47f49:	cmp    0x8(%rdx,%rcx,8),%rax
   47f4e:	je     47f66 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa46>
   47f50:	lea    (%r15,%r13,1),%rcx
   47f54:	mov    %cx,(%rax)
   47f57:	addq   $0x2,0x0(%rbp)
   47f5c:	inc    %r13
   47f5f:	cmp    %r13,%r14
   47f62:	jne    47f30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa10>
   47f64:	jmp    47fa0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa80>
   47f66:	lea    0xa8(%rsp),%rdi
   47f6e:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   47f73:	test   %al,%al
   47f75:	je     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   47f7b:	mov    0x0(%rbp),%rax
   47f7f:	mov    0x8(%rsp),%rdi
   47f84:	jmp    47f50 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa30>
   47f86:	cs nopw 0x0(%rax,%rax,1)
   47f90:	mov    %r8,0x38(%rsp)
   47f95:	xor    %r9d,%r9d
   47f98:	jmp    4802a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb0a>
   47f9d:	nopl   (%rax)
   47fa0:	cmp    $0xf,%r12
   47fa4:	lea    0xe8(%rsp),%r13
   47fac:	ja     47e10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   47fb2:	lea    0x10(%rbx),%r12
   47fb6:	add    %r14,%rbx
   47fb9:	lea    (%r15,%r14,1),%ebp
   47fbd:	add    $0x2,%ebp
   47fc0:	movzbl (%rbx),%esi
   47fc3:	lea    (%rsi,%rsi,4),%rcx
   47fc7:	lea    0x0(,%rcx,8),%r14
   47fcf:	add    %r13,%r14
   47fd2:	mov    0x0(%r13,%rcx,8),%rax
   47fd7:	cmp    0x8(%r13,%rcx,8),%rax
   47fdc:	je     47ff4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xad4>
   47fde:	mov    %bp,(%rax)
   47fe1:	addq   $0x2,(%r14)
   47fe5:	inc    %rbx
   47fe8:	inc    %ebp
   47fea:	cmp    %r12,%rbx
   47fed:	jne    47fc0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaa0>
   47fef:	jmp    47e10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   47ff4:	lea    0xa8(%rsp),%rdi
   47ffc:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   48001:	test   %al,%al
   48003:	je     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   48009:	mov    (%r14),%rax
   4800c:	mov    0x8(%rsp),%rdi
   48011:	lea    0xe8(%rsp),%r13
   48019:	jmp    47fde <<c30walk::ctor2su::Ctor2su>::build::<false>+0xabe>
   4801b:	mov    %r8,0x38(%rsp)
   48020:	mov    %rsi,0x18(%rsp)
   48025:	mov    %rax,0x28(%rsp)
   4802a:	mov    %rdx,0x40(%rsp)
   4802f:	sub    %rdx,%r12
   48032:	movzbl %cl,%eax
   48035:	mov    %rax,0x98(%rsp)
   4803d:	lea    (%rax,%rax,4),%rdx
   48041:	mov    0x0(%r13,%rdx,8),%rax
   48046:	mov    0x1c(%r13,%rdx,8),%esi
   4804b:	mov    %rax,%rdi
   4804e:	sub    0x10(%r13,%rdx,8),%rdi
   48053:	shr    $1,%rdi
   48056:	add    0x18(%r13,%rdx,8),%edi
   4805b:	add    %r12,%rdi
   4805e:	cmp    %rsi,%rdi
   48061:	ja     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   48067:	mov    %r9d,0x5c(%rsp)
   4806c:	lea    0xe8(%rsp),%r13
   48074:	lea    0x0(,%rdx,8),%r10
   4807c:	add    %r13,%r10
   4807f:	movzbl %cl,%r15d
   48083:	mov    0x8(%r10),%rsi
   48087:	mov    $0x88888889,%r14d
   4808d:	lea    -0x2a754(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   48094:	mov    %r10,0x48(%rsp)
   48099:	jmp    480d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xbb0>
   4809b:	nopl   0x0(%rax,%rax,1)
   480a0:	mov    0x3c(%rsi),%edi
   480a3:	mov    $0xfffff,%r9d
   480a9:	and    %r9d,%edi
   480ac:	sub    %r8d,%ecx
   480af:	sub    %eax,%r8d
   480b2:	mov    %r8d,%eax
   480b5:	lea    (%rsi,%rax,2),%rax
   480b9:	mov    %rax,0x10(%r10)
   480bd:	mov    %rax,(%r10)
   480c0:	lea    (%rax,%rcx,2),%rsi
   480c4:	mov    %rsi,0x8(%r10)
   480c8:	mov    %edx,0x20(%r10)
   480cc:	mov    %edi,0x24(%r10)
   480d0:	sub    %rax,%rsi
   480d3:	shr    $1,%rsi
   480d6:	cmp    %rsi,%r12
   480d9:	cmovb  %r12,%rsi
   480dd:	test   %rsi,%rsi
   480e0:	je     4819d <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc7d>
   480e6:	mov    0x28e8(%rsp),%rdi
   480ee:	cmp    $0x1d,%rdi
   480f2:	ja     48cdf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17bf>
   480f8:	cmp    $0x1f,%rsi
   480fc:	jae    489f1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14d1>
   48102:	mov    0x28f0(%rsp),%ecx
   48109:	mov    %rdi,%r11
   4810c:	shl    $0x6,%r11
   48110:	lea    (%rbx,%r11,1),%rdx
   48114:	lea    -0x2(,%rsi,2),%r9
   4811c:	cmp    $0xe,%r9
   48120:	jb     48135 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc15>
   48122:	mov    %rax,%r8
   48125:	sub    %rbx,%r8
   48128:	sub    %r11,%r8
   4812b:	cmp    $0x10,%r8
   4812f:	jae    48588 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1068>
   48135:	mov    %rdx,%r8
   48138:	xor    %r10d,%r10d
   4813b:	lea    (%rdx,%rsi,2),%rdx
   4813f:	lea    (%rax,%r10,2),%r9
   48143:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48150:	movzwl (%r8),%r10d
   48154:	add    %cx,%r10w
   48158:	add    $0x2,%r8
   4815c:	mov    %r10w,(%r9)
   48160:	add    $0x2,%r9
   48164:	cmp    %rdx,%r8
   48167:	jne    48150 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc30>
   48169:	lea    (%rax,%rsi,2),%rax
   4816d:	mov    0x48(%rsp),%r10
   48172:	mov    %rax,(%r10)
   48175:	add    %rsi,%rdi
   48178:	lea    0x2(%rsi),%edx
   4817b:	mov    %rdi,%r8
   4817e:	sub    $0x1e,%r8
   48182:	cmovb  %rdi,%r8
   48186:	cmovb  %esi,%edx
   48189:	mov    %r8,0x28e8(%rsp)
   48191:	add    %ecx,%edx
   48193:	mov    %edx,0x28f0(%rsp)
   4819a:	sub    %rsi,%r12
   4819d:	test   %r12,%r12
   481a0:	je     48770 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1250>
   481a6:	mov    %rax,%r8
   481a9:	cmp    0x10(%r10),%rax
   481ad:	je     481d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcb0>
   481af:	lea    0xa8(%rsp),%rdi
   481b7:	mov    0x98(%rsp),%rsi
   481bf:	call   96880 <<c30walk::ctor2su::W<false>>::finish>
   481c4:	mov    0x48(%rsp),%r10
   481c9:	mov    (%r10),%r8
   481cc:	mov    0x10(%r10),%rax
   481d0:	sub    %rax,%r8
   481d3:	shr    $1,%r8
   481d6:	add    0x18(%r10),%r8d
   481da:	mov    %r8d,0x18(%r10)
   481de:	pxor   %xmm0,%xmm0
   481e2:	movdqu %xmm0,(%r10)
   481e7:	movq   $0x0,0x10(%r10)
   481ef:	cmp    $0x1e,%r12
   481f3:	jb     48708 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11e8>
   481f9:	mov    %r8,%rcx
   481fc:	imul   %r14,%rcx
   48200:	shr    $0x24,%rcx
   48204:	mov    %ecx,%eax
   48206:	shl    $0x5,%eax
   48209:	lea    (%rcx,%rcx,1),%edx
   4820c:	sub    %eax,%edx
   4820e:	add    %r8d,%edx
   48211:	jne    48708 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11e8>
   48217:	mov    %r12,%rax
   4821a:	movabs $0x8888888888888889,%rdx
   48224:	mul    %rdx
   48227:	shr    $0x4,%rdx
   4822b:	mov    %ecx,%r9d
   4822e:	cmpl   $0x0,0x14(%rsp)
   48233:	mov    %r12,0x68(%rsp)
   48238:	mov    %rdx,0xa0(%rsp)
   48240:	je     48540 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1020>
   48246:	mov    0x28f0(%rsp),%r10d
   4824e:	mov    0xb8(%rsp),%rax
   48256:	mov    %rax,0x30(%rsp)
   4825b:	mov    0xa8(%rsp),%r11
   48263:	mov    0xb0(%rsp),%rax
   4826b:	mov    %rax,0x50(%rsp)
   48270:	mov    0xd0(%rsp),%rax
   48278:	mov    0x28e8(%rsp),%r12
   48280:	mov    %r12,%rdi
   48283:	shl    $0x6,%rdi
   48287:	add    %rbx,%rdi
   4828a:	mov    %r9,%rcx
   4828d:	shl    $0x6,%rcx
   48291:	lea    (%rax,%rcx,1),%rbx
   48295:	add    $0x3c,%rbx
   48299:	mov    %r10d,%r13d
   4829c:	shl    $0x8,%r13d
   482a0:	or     %r15d,%r13d
   482a3:	xor    %r14d,%r14d
   482a6:	jmp    484b4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf94>
   482ab:	nopl   0x0(%rax,%rax,1)
   482b0:	mov    0x8(%r11),%rcx
   482b4:	shl    $0x7,%eax
   482b7:	mov    %r13d,(%rcx,%rax,1)
   482bb:	movzwl 0x2(%rdi),%esi
   482bf:	lea    (%r10,%r14,1),%ebp
   482c3:	add    %ebp,%esi
   482c5:	shl    $0x8,%esi
   482c8:	or     %r15d,%esi
   482cb:	mov    %esi,0x4(%rcx,%rax,1)
   482cf:	movzwl 0x4(%rdi),%esi
   482d3:	add    %ebp,%esi
   482d5:	shl    $0x8,%esi
   482d8:	or     %r15d,%esi
   482db:	mov    %esi,0x8(%rcx,%rax,1)
   482df:	movzwl 0x6(%rdi),%esi
   482e3:	add    %ebp,%esi
   482e5:	shl    $0x8,%esi
   482e8:	or     %r15d,%esi
   482eb:	mov    %esi,0xc(%rcx,%rax,1)
   482ef:	movzwl 0x8(%rdi),%esi
   482f3:	add    %ebp,%esi
   482f5:	shl    $0x8,%esi
   482f8:	or     %r15d,%esi
   482fb:	mov    %esi,0x10(%rcx,%rax,1)
   482ff:	movzwl 0xa(%rdi),%esi
   48303:	add    %ebp,%esi
   48305:	shl    $0x8,%esi
   48308:	or     %r15d,%esi
   4830b:	mov    %esi,0x14(%rcx,%rax,1)
   4830f:	movzwl 0xc(%rdi),%esi
   48313:	add    %ebp,%esi
   48315:	shl    $0x8,%esi
   48318:	or     %r15d,%esi
   4831b:	mov    %esi,0x18(%rcx,%rax,1)
   4831f:	movzwl 0xe(%rdi),%esi
   48323:	add    %ebp,%esi
   48325:	shl    $0x8,%esi
   48328:	or     %r15d,%esi
   4832b:	mov    %esi,0x1c(%rcx,%rax,1)
   4832f:	movzwl 0x10(%rdi),%esi
   48333:	add    %ebp,%esi
   48335:	shl    $0x8,%esi
   48338:	or     %r15d,%esi
   4833b:	mov    %esi,0x20(%rcx,%rax,1)
   4833f:	movzwl 0x12(%rdi),%esi
   48343:	add    %ebp,%esi
   48345:	shl    $0x8,%esi
   48348:	or     %r15d,%esi
   4834b:	mov    %esi,0x24(%rcx,%rax,1)
   4834f:	movzwl 0x14(%rdi),%esi
   48353:	add    %ebp,%esi
   48355:	shl    $0x8,%esi
   48358:	or     %r15d,%esi
   4835b:	mov    %esi,0x28(%rcx,%rax,1)
   4835f:	movzwl 0x16(%rdi),%esi
   48363:	add    %ebp,%esi
   48365:	shl    $0x8,%esi
   48368:	or     %r15d,%esi
   4836b:	mov    %esi,0x2c(%rcx,%rax,1)
   4836f:	movzwl 0x18(%rdi),%esi
   48373:	add    %ebp,%esi
   48375:	shl    $0x8,%esi
   48378:	or     %r15d,%esi
   4837b:	mov    %esi,0x30(%rcx,%rax,1)
   4837f:	movzwl 0x1a(%rdi),%esi
   48383:	add    %ebp,%esi
   48385:	shl    $0x8,%esi
   48388:	or     %r15d,%esi
   4838b:	mov    %esi,0x34(%rcx,%rax,1)
   4838f:	movzwl 0x1c(%rdi),%esi
   48393:	add    %ebp,%esi
   48395:	shl    $0x8,%esi
   48398:	or     %r15d,%esi
   4839b:	mov    %esi,0x38(%rcx,%rax,1)
   4839f:	movzwl 0x1e(%rdi),%esi
   483a3:	add    %ebp,%esi
   483a5:	shl    $0x8,%esi
   483a8:	or     %r15d,%esi
   483ab:	mov    %esi,0x3c(%rcx,%rax,1)
   483af:	movzwl 0x20(%rdi),%esi
   483b3:	add    %ebp,%esi
   483b5:	shl    $0x8,%esi
   483b8:	or     %r15d,%esi
   483bb:	mov    %esi,0x40(%rcx,%rax,1)
   483bf:	movzwl 0x22(%rdi),%esi
   483c3:	add    %ebp,%esi
   483c5:	shl    $0x8,%esi
   483c8:	or     %r15d,%esi
   483cb:	mov    %esi,0x44(%rcx,%rax,1)
   483cf:	movzwl 0x24(%rdi),%esi
   483d3:	add    %ebp,%esi
   483d5:	shl    $0x8,%esi
   483d8:	or     %r15d,%esi
   483db:	mov    %esi,0x48(%rcx,%rax,1)
   483df:	movzwl 0x26(%rdi),%esi
   483e3:	add    %ebp,%esi
   483e5:	shl    $0x8,%esi
   483e8:	or     %r15d,%esi
   483eb:	mov    %esi,0x4c(%rcx,%rax,1)
   483ef:	movzwl 0x28(%rdi),%esi
   483f3:	add    %ebp,%esi
   483f5:	shl    $0x8,%esi
   483f8:	or     %r15d,%esi
   483fb:	mov    %esi,0x50(%rcx,%rax,1)
   483ff:	movzwl 0x2a(%rdi),%esi
   48403:	add    %ebp,%esi
   48405:	shl    $0x8,%esi
   48408:	or     %r15d,%esi
   4840b:	mov    %esi,0x54(%rcx,%rax,1)
   4840f:	movzwl 0x2c(%rdi),%esi
   48413:	add    %ebp,%esi
   48415:	shl    $0x8,%esi
   48418:	or     %r15d,%esi
   4841b:	mov    %esi,0x58(%rcx,%rax,1)
   4841f:	movzwl 0x2e(%rdi),%esi
   48423:	add    %ebp,%esi
   48425:	shl    $0x8,%esi
   48428:	or     %r15d,%esi
   4842b:	mov    %esi,0x5c(%rcx,%rax,1)
   4842f:	movzwl 0x30(%rdi),%esi
   48433:	add    %ebp,%esi
   48435:	shl    $0x8,%esi
   48438:	or     %r15d,%esi
   4843b:	mov    %esi,0x60(%rcx,%rax,1)
   4843f:	movzwl 0x32(%rdi),%esi
   48443:	add    %ebp,%esi
   48445:	shl    $0x8,%esi
   48448:	or     %r15d,%esi
   4844b:	mov    %esi,0x64(%rcx,%rax,1)
   4844f:	movzwl 0x34(%rdi),%esi
   48453:	add    %ebp,%esi
   48455:	shl    $0x8,%esi
   48458:	or     %r15d,%esi
   4845b:	mov    %esi,0x68(%rcx,%rax,1)
   4845f:	movzwl 0x36(%rdi),%esi
   48463:	add    %ebp,%esi
   48465:	shl    $0x8,%esi
   48468:	or     %r15d,%esi
   4846b:	mov    %esi,0x6c(%rcx,%rax,1)
   4846f:	movzwl 0x38(%rdi),%esi
   48473:	add    %ebp,%esi
   48475:	shl    $0x8,%esi
   48478:	or     %r15d,%esi
   4847b:	mov    %esi,0x70(%rcx,%rax,1)
   4847f:	movzwl 0x3a(%rdi),%esi
   48483:	add    %ebp,%esi
   48485:	shl    $0x8,%esi
   48488:	or     %r15d,%esi
   4848b:	mov    %esi,0x74(%rcx,%rax,1)
   4848f:	lea    (%r10,%r14,1),%eax
   48493:	add    $0x20,%eax
   48496:	mov    %eax,0x28f0(%rsp)
   4849d:	add    $0x20,%r14
   484a1:	inc    %r9
   484a4:	add    $0x2000,%r13d
   484ab:	dec    %rdx
   484ae:	je     486bc <<c30walk::ctor2su::Ctor2su>::build::<false>+0x119c>
   484b4:	mov    %r9,%rax
   484b7:	shr    $0x6,%rax
   484bb:	cmp    0x30(%rsp),%rax
   484c0:	jae    48bf5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16d5>
   484c6:	mov    0x50(%rsp),%rcx
   484cb:	mov    (%rcx,%rax,8),%rax
   484cf:	bt     %r9,%rax
   484d3:	jae    48500 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xfe0>
   484d5:	cmp    $0x1e,%r12
   484d9:	jae    48c1b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16fb>
   484df:	mov    (%rbx,%r14,2),%eax
   484e3:	and    $0xfffff,%eax
   484e8:	mov    0x10(%r11),%rcx
   484ec:	cmp    %rax,%rcx
   484ef:	ja     482b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xd90>
   484f5:	jmp    48c09 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16e9>
   484fa:	nopw   0x0(%rax,%rax,1)
   48500:	cmp    $0x1e,%r12
   48504:	jae    48c5b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x173b>
   4850a:	movups 0x2c(%rdi),%xmm0
   4850e:	movups %xmm0,-0x10(%rbx,%r14,2)
   48514:	movdqa (%rdi),%xmm0
   48518:	movdqa 0x10(%rdi),%xmm1
   4851d:	movdqa 0x20(%rdi),%xmm2
   48522:	movdqa %xmm2,-0x1c(%rbx,%r14,2)
   48529:	movdqa %xmm1,-0x2c(%rbx,%r14,2)
   48530:	movdqa %xmm0,-0x3c(%rbx,%r14,2)
   48537:	mov    %r13d,(%rbx,%r14,2)
   4853b:	jmp    4848f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf6f>
   48540:	mov    0x28e8(%rsp),%rdi
   48548:	cmp    $0x1d,%rdi
   4854c:	ja     48cf0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17d0>
   48552:	shl    $0x6,%rdi
   48556:	add    %rbx,%rdi
   48559:	mov    0x28f0(%rsp),%eax
   48560:	mov    0xd0(%rsp),%rcx
   48568:	shl    $0x6,%r9
   4856c:	mov    %eax,%r14d
   4856f:	shl    $0x8,%r14d
   48573:	or     %r15d,%r14d
   48576:	lea    -0x1e(%r12),%rsi
   4857b:	cmp    $0x1e,%rsi
   4857f:	jae    485ec <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10cc>
   48581:	xor    %esi,%esi
   48583:	jmp    4867a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x115a>
   48588:	shr    $1,%r9
   4858b:	inc    %r9
   4858e:	mov    %r9,%r10
   48591:	and    $0xfffffffffffffff8,%r10
   48595:	lea    (%rdx,%r10,2),%r8
   48599:	movd   %ecx,%xmm0
   4859d:	pshuflw $0x0,%xmm0,%xmm0
   485a2:	add    %rbx,%r11
   485a5:	add    $0x8,%r11
   485a9:	xor    %ebx,%ebx
   485ab:	nopl   0x0(%rax,%rax,1)
   485b0:	movq   -0x8(%r11,%rbx,2),%xmm1
   485b7:	movq   (%r11,%rbx,2),%xmm2
   485bd:	paddw  %xmm0,%xmm1
   485c1:	paddw  %xmm0,%xmm2
   485c5:	punpcklqdq %xmm2,%xmm1
   485c9:	movdqu %xmm1,(%rax,%rbx,2)
   485ce:	add    $0x8,%rbx
   485d2:	cmp    %rbx,%r10
   485d5:	jne    485b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1090>
   485d7:	cmp    %r10,%r9
   485da:	lea    -0x2aca1(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   485e1:	jne    4813b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc1b>
   485e7:	jmp    48169 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc49>
   485ec:	mov    %rdx,%r10
   485ef:	movabs $0xffffffffffffffe,%rsi
   485f9:	and    %rsi,%r10
   485fc:	lea    (%rcx,%r9,1),%r11
   48600:	add    $0x7c,%r11
   48604:	xor    %esi,%esi
   48606:	cs nopw 0x0(%rax,%rax,1)
   48610:	movdqu 0x2c(%rdi),%xmm0
   48615:	movdqu %xmm0,-0x50(%r11)
   4861b:	movdqa (%rdi),%xmm1
   4861f:	movdqa 0x10(%rdi),%xmm2
   48624:	movdqa 0x20(%rdi),%xmm3
   48629:	movdqa %xmm3,-0x5c(%r11)
   4862f:	movdqa %xmm2,-0x6c(%r11)
   48635:	movdqa %xmm1,-0x7c(%r11)
   4863b:	mov    %r14d,-0x40(%r11)
   4863f:	add    $0x2,%rsi
   48643:	movdqa %xmm1,-0x3c(%r11)
   48649:	movdqa %xmm2,-0x2c(%r11)
   4864f:	movdqa %xmm3,-0x1c(%r11)
   48655:	movdqu %xmm0,-0x10(%r11)
   4865b:	lea    0x2000(%r14),%ebx
   48662:	mov    %ebx,(%r11)
   48665:	add    $0x4000,%r14d
   4866c:	sub    $0xffffffffffffff80,%r11
   48670:	cmp    %rsi,%r10
   48673:	jne    48610 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10f0>
   48675:	test   $0x1,%dl
   48678:	je     486ae <<c30walk::ctor2su::Ctor2su>::build::<false>+0x118e>
   4867a:	add    %r9,%rcx
   4867d:	shl    $0x6,%rsi
   48681:	movups 0x2c(%rdi),%xmm0
   48685:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4868a:	movdqa (%rdi),%xmm0
   4868e:	movdqa 0x10(%rdi),%xmm1
   48693:	movdqa 0x20(%rdi),%xmm2
   48698:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   4869e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   486a4:	movdqa %xmm0,(%rcx,%rsi,1)
   486a9:	mov    %r14d,0x3c(%rcx,%rsi,1)
   486ae:	mov    %edx,%ecx
   486b0:	shl    $0x5,%ecx
   486b3:	add    %ecx,%eax
   486b5:	mov    %eax,0x28f0(%rsp)
   486bc:	mov    0xa0(%rsp),%rdx
   486c4:	mov    %rdx,%rax
   486c7:	shl    $0x5,%rax
   486cb:	lea    (%rdx,%rdx,1),%rcx
   486cf:	sub    %rax,%rcx
   486d2:	mov    0x68(%rsp),%r12
   486d7:	add    %rcx,%r12
   486da:	lea    (%rdx,%rdx,1),%eax
   486dd:	shl    $0x5,%edx
   486e0:	sub    %eax,%edx
   486e2:	add    %edx,%r8d
   486e5:	mov    0x48(%rsp),%r10
   486ea:	mov    %r8d,0x18(%r10)
   486ee:	test   %r12,%r12
   486f1:	lea    0xe8(%rsp),%r13
   486f9:	mov    $0x88888889,%r14d
   486ff:	lea    -0x2adc6(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   48706:	je     48770 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1250>
   48708:	mov    0x1c(%r10),%ecx
   4870c:	mov    %r8d,%edi
   4870f:	imul   %r14,%rdi
   48713:	mov    %rdi,%rdx
   48716:	shr    $0x24,%rdx
   4871a:	mov    %edx,%eax
   4871c:	shl    $0x5,%eax
   4871f:	sub    %edx,%eax
   48721:	sub    %edx,%eax
   48723:	lea    0x1e(%rax),%esi
   48726:	cmp    %ecx,%esi
   48728:	cmovb  %esi,%ecx
   4872b:	shr    $0x2a,%rdi
   4872f:	mov    0xb8(%rsp),%rsi
   48737:	cmp    %rsi,%rdi
   4873a:	jae    48cd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17b3>
   48740:	mov    %rdx,%rsi
   48743:	shl    $0x6,%rsi
   48747:	add    0xd0(%rsp),%rsi
   4874f:	mov    0xb0(%rsp),%r9
   48757:	mov    (%r9,%rdi,8),%rdi
   4875b:	bt     %rdx,%rdi
   4875f:	jb     480a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb80>
   48765:	mov    $0xffffffff,%edi
   4876a:	jmp    480ac <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb8c>
   4876f:	nop
   48770:	mov    0x8(%rsp),%rdi
   48775:	mov    0x28(%rsp),%r14
   4877a:	mov    0x18(%rsp),%r15
   4877f:	mov    0x5c(%rsp),%eax
   48783:	jmp    47e4b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x92b>
   48788:	nopl   0x0(%rax,%rax,1)
   48790:	mov    %r12,%rcx
   48793:	test   %rcx,%rcx
   48796:	je     47d1b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7fb>
   4879c:	mov    0x20(%rsp),%r12
   487a1:	sub    %rcx,%r12
   487a4:	mov    0x28e8(%rsp),%rax
   487ac:	mov    $0x1e,%ebx
   487b1:	sub    %rax,%rbx
   487b4:	cmp    %rcx,%rbx
   487b7:	cmovae %rcx,%rbx
   487bb:	mov    %rbx,%r13
   487be:	add    %r12,%r13
   487c1:	jb     48c2f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x170f>
   487c7:	cmp    %rdi,%r13
   487ca:	ja     48c2f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x170f>
   487d0:	mov    %r15,0x18(%rsp)
   487d5:	mov    %r14,0x28(%rsp)
   487da:	mov    %rcx,0x40(%rsp)
   487df:	mov    0x28f0(%rsp),%r14d
   487e7:	cmp    $0x1e,%rax
   487eb:	jne    4888f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x136f>
   487f1:	cmp    %r13,0x20(%rsp)
   487f6:	jb     48c44 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1724>
   487fc:	lea    0xe8(%rsp),%rdx
   48804:	jne    488f9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13d9>
   4880a:	mov    0x28e8(%rsp),%rax
   48812:	mov    0x28f0(%rsp),%ecx
   48819:	mov    0x40(%rsp),%r8
   4881e:	cmp    $0x1,%r8
   48822:	je     47ce7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7c7>
   48828:	mov    %r8d,%edx
   4882b:	and    $0xe,%edx
   4882e:	mov    0x28(%rsp),%r14
   48833:	mov    0x18(%rsp),%r15
   48838:	nopl   0x0(%rax,%rax,1)
   48840:	lea    0x1(%rax),%rsi
   48844:	add    $0xffffffffffffffe3,%rax
   48848:	xor    %edi,%edi
   4884a:	cmp    $0x1e,%rsi
   4884e:	setae  %dil
   48852:	cmovb  %rsi,%rax
   48856:	lea    (%rcx,%rdi,2),%ecx
   48859:	lea    0x1(%rax),%rsi
   4885d:	add    $0xffffffffffffffe3,%rax
   48861:	xor    %edi,%edi
   48863:	cmp    $0x1e,%rsi
   48867:	setae  %dil
   4886b:	cmovb  %rsi,%rax
   4886f:	lea    (%rcx,%rdi,2),%ecx
   48872:	add    $0x2,%ecx
   48875:	add    $0xfffffffffffffffe,%rdx
   48879:	jne    48840 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1320>
   4887b:	test   $0x1,%r8b
   4887f:	mov    0x8(%rsp),%rdi
   48884:	jne    47cf1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7d1>
   4888a:	jmp    47d0c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7ec>
   4888f:	add    0x60(%rsp),%r12
   48894:	xor    %r15d,%r15d
   48897:	nopw   0x0(%rax,%rax,1)
   488a0:	movzbl (%r12,%r15,1),%esi
   488a5:	lea    (%rsi,%rsi,4),%rcx
   488a9:	lea    0xe8(%rsp),%rdx
   488b1:	lea    (%rdx,%rcx,8),%rbp
   488b5:	mov    (%rdx,%rcx,8),%rax
   488b9:	cmp    0x8(%rdx,%rcx,8),%rax
   488be:	je     488d9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13b9>
   488c0:	lea    (%r14,%r15,1),%rcx
   488c4:	mov    %cx,(%rax)
   488c7:	addq   $0x2,0x0(%rbp)
   488cc:	inc    %r15
   488cf:	cmp    %r15,%rbx
   488d2:	jne    488a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1380>
   488d4:	jmp    487f1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12d1>
   488d9:	lea    0xa8(%rsp),%rdi
   488e1:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   488e6:	test   %al,%al
   488e8:	je     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   488ee:	mov    0x0(%rbp),%rax
   488f2:	mov    0x8(%rsp),%rdi
   488f7:	jmp    488c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13a0>
   488f9:	mov    0x60(%rsp),%rax
   488fe:	mov    0x20(%rsp),%rcx
   48903:	lea    (%rax,%rcx,1),%r15
   48907:	lea    (%r14,%rbx,1),%ebp
   4890b:	add    $0x2,%ebp
   4890e:	sub    0x40(%rsp),%rbx
   48913:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48920:	movzbl (%r15,%rbx,1),%esi
   48925:	lea    (%rsi,%rsi,4),%rcx
   48929:	lea    (%rdx,%rcx,8),%r14
   4892d:	mov    (%rdx,%rcx,8),%rax
   48931:	cmp    0x8(%rdx,%rcx,8),%rax
   48936:	je     4894b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x142b>
   48938:	mov    %bp,(%rax)
   4893b:	addq   $0x2,(%r14)
   4893f:	inc    %ebp
   48941:	inc    %rbx
   48944:	jne    48920 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1400>
   48946:	jmp    4880a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12ea>
   4894b:	lea    0xa8(%rsp),%rdi
   48953:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   48958:	test   %al,%al
   4895a:	je     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   4895c:	mov    (%r14),%rax
   4895f:	mov    0x8(%rsp),%rdi
   48964:	lea    0xe8(%rsp),%rdx
   4896c:	jmp    48938 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1418>
   4896e:	mov    %r13,%rdi
   48971:	call   fea30 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   48976:	jmp    47da0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x880>
   4897b:	mov    $0x84,%eax
   48980:	lea    0x50(%rax),%rcx
   48984:	cmp    $0x28d4,%rcx
   4898b:	je     48a04 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14e4>
   4898d:	mov    0x64(%rsp,%rax,1),%rdx
   48992:	sub    0x74(%rsp,%rax,1),%rdx
   48997:	shr    $1,%rdx
   4899a:	add    0x7c(%rsp,%rax,1),%edx
   4899e:	cmp    0x80(%rsp,%rax,1),%edx
   489a5:	jne    489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   489a7:	mov    0x8c(%rsp,%rax,1),%rdx
   489af:	sub    0x9c(%rsp,%rax,1),%rdx
   489b7:	shr    $1,%rdx
   489ba:	add    0xa4(%rsp,%rax,1),%edx
   489c1:	cmp    0xa8(%rsp,%rax,1),%edx
   489c8:	mov    %rcx,%rax
   489cb:	je     48980 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1460>
   489cd:	mov    0x5530(%rsp),%rax
   489d5:	movq   $0x0,0x10(%rax)
   489dd:	xor    %eax,%eax
   489df:	add    $0x54f8,%rsp
   489e6:	pop    %rbx
   489e7:	pop    %r12
   489e9:	pop    %r13
   489eb:	pop    %r14
   489ed:	pop    %r15
   489ef:	pop    %rbp
   489f0:	ret
   489f1:	lea    0x100190(%rip),%rcx        # 148b88 <__frame_dummy_init_array_entry+0x2338>
   489f8:	mov    $0x1e,%edx
   489fd:	xor    %edi,%edi
   489ff:	call   f0c10 <core::slice::index::slice_index_fail>
   48a04:	xor    %r14d,%r14d
   48a07:	lea    0xa8(%rsp),%r15
   48a0f:	mov    0x80(%rsp),%r12
   48a17:	mov    0x8(%rsp),%rdx
   48a1c:	mov    0x78(%rsp),%r13
   48a21:	lea    0xe8(%rsp),%rbx
   48a29:	jmp    48a59 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1539>
   48a2b:	nopl   0x0(%rax,%rax,1)
   48a30:	sub    %rcx,%rax
   48a33:	inc    %r14
   48a36:	shr    $1,%rax
   48a39:	add    %eax,0x18(%rbx)
   48a3c:	pxor   %xmm0,%xmm0
   48a40:	movdqu %xmm0,(%rbx)
   48a44:	movq   $0x0,0x10(%rbx)
   48a4c:	add    $0x28,%rbx
   48a50:	cmp    $0x100,%r14
   48a57:	je     48a7e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155e>
   48a59:	mov    (%rbx),%rcx
   48a5c:	mov    %rcx,%rax
   48a5f:	cmp    0x10(%rbx),%rcx
   48a63:	je     48a30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1510>
   48a65:	mov    %r15,%rdi
   48a68:	mov    %r14,%rsi
   48a6b:	call   96880 <<c30walk::ctor2su::W<false>>::finish>
   48a70:	mov    0x8(%rsp),%rdx
   48a75:	mov    (%rbx),%rax
   48a78:	mov    0x10(%rbx),%rcx
   48a7c:	jmp    48a30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1510>
   48a7e:	xor    %ecx,%ecx
   48a80:	mov    0x8(%r12),%rbx
   48a85:	test   %r13,%r13
   48a88:	setne  %al
   48a8b:	je     48acf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15af>
   48a8d:	mov    %al,%cl
   48a8f:	mov    %r13,%rax
   48a92:	sub    %rcx,%rax
   48a95:	imul   $0xffffffffffffffe2,%rax,%rcx
   48a99:	add    %rdx,%rcx
   48a9c:	cmp    $0x1e,%rcx
   48aa0:	jae    48acf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15af>
   48aa2:	lea    (%rbx,%r13,4),%rcx
   48aa6:	lea    (%rcx,%rdx,2),%rdi
   48aaa:	add    $0xfffffffffffffffc,%rdi
   48aae:	lea    (%rax,%rax,1),%rcx
   48ab2:	shl    $0x5,%rax
   48ab6:	sub    %rcx,%rax
   48ab9:	sub    %rdx,%rax
   48abc:	lea    0x3c(,%rax,2),%rdx
   48ac4:	mov    $0xff,%esi
   48ac9:	call   *0x104ae1(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
   48acf:	shl    $0x6,%r13
   48ad3:	pcmpeqd %xmm0,%xmm0
   48ad7:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   48ade:	movdqa %xmm0,0x20(%rbx,%r13,1)
   48ae5:	movdqa %xmm0,0x10(%rbx,%r13,1)
   48aec:	movdqa %xmm0,(%rbx,%r13,1)
   48af2:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   48afb:	mov    0x70(%rsp),%rax
   48b00:	mov    0x8(%rax),%rdx
   48b04:	mov    0x10(%rax),%rcx
   48b08:	mov    0x5530(%rsp),%rax
   48b10:	mov    0x8(%rax),%r8
   48b14:	mov    0x10(%rax),%r9
   48b18:	lea    0xa8(%rsp),%rdi
   48b20:	mov    %rbx,%rsi
   48b23:	call   a5810 <c30walk::ctor::canonicalise>
   48b28:	mov    0x88(%rsp),%rax
   48b30:	mov    %rax,0x10(%r12)
   48b35:	mov    $0x1,%al
   48b37:	jmp    489df <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14bf>
   48b3c:	mov    %rdi,%r12
   48b3f:	test   %rax,%rax
   48b42:	je     48b54 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1634>
   48b44:	mov    0x8(%r9),%rdi
   48b48:	mov    %r9,%r15
   48b4b:	call   *0x104a6f(%rip)        # 14d5c0 <free@GLIBC_2.2.5>
   48b51:	mov    %r15,%r9
   48b54:	movq   $0x0,(%r9)
   48b5b:	movq   $0x40,0x8(%r9)
   48b63:	movq   $0x0,0x10(%r9)
   48b6b:	mov    $0x40,%edx
   48b70:	mov    %r9,%rdi
   48b73:	lea    0x1(%r13),%rsi
   48b77:	mov    %r9,%r15
   48b7a:	call   9b3e0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48b7f:	mov    %r15,%r9
   48b82:	cmp    $0xffffffffffffffff,%rax
   48b86:	mov    %r12,%rdi
   48b89:	je     47758 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x238>
   48b8f:	jmp    48bea <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16ca>
   48b91:	mov    %rdi,%r12
   48b94:	mov    %r9,%r15
   48b97:	test   %rax,%rax
   48b9a:	je     48bae <<c30walk::ctor2su::Ctor2su>::build::<false>+0x168e>
   48b9c:	mov    0x5530(%rsp),%rax
   48ba4:	mov    0x8(%rax),%rdi
   48ba8:	call   *0x104a12(%rip)        # 14d5c0 <free@GLIBC_2.2.5>
   48bae:	mov    0x5530(%rsp),%rdi
   48bb6:	movq   $0x0,(%rdi)
   48bbd:	movq   $0x40,0x8(%rdi)
   48bc5:	movq   $0x0,0x10(%rdi)
   48bcd:	mov    $0x80,%edx
   48bd2:	mov    %rbx,%rsi
   48bd5:	call   9b3e0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48bda:	cmp    $0xffffffffffffffff,%rax
   48bde:	mov    %r15,%r9
   48be1:	mov    %r12,%rdi
   48be4:	je     4776c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x24c>
   48bea:	mov    %rax,%rdi
   48bed:	mov    %rdx,%rsi
   48bf0:	call   ee39b <alloc::raw_vec::handle_error>
   48bf5:	lea    0xfdedc(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48bfc:	mov    %rax,%rdi
   48bff:	mov    0x30(%rsp),%rsi
   48c04:	call   efe33 <core::panicking::panic_bounds_check>
   48c09:	lea    0x100038(%rip),%rdx        # 148c48 <__frame_dummy_init_array_entry+0x23f8>
   48c10:	mov    %rax,%rdi
   48c13:	mov    %rcx,%rsi
   48c16:	call   efe33 <core::panicking::panic_bounds_check>
   48c1b:	lea    0x10000e(%rip),%rdx        # 148c30 <__frame_dummy_init_array_entry+0x23e0>
   48c22:	mov    $0x1e,%esi
   48c27:	mov    %r12,%rdi
   48c2a:	call   efe33 <core::panicking::panic_bounds_check>
   48c2f:	lea    0xfff82(%rip),%rcx        # 148bb8 <__frame_dummy_init_array_entry+0x2368>
   48c36:	mov    %rdi,%rdx
   48c39:	mov    %r12,%rdi
   48c3c:	mov    %r13,%rsi
   48c3f:	call   f0c10 <core::slice::index::slice_index_fail>
   48c44:	lea    0xfff55(%rip),%rcx        # 148ba0 <__frame_dummy_init_array_entry+0x2350>
   48c4b:	mov    %rdi,%rdx
   48c4e:	mov    %r13,%rdi
   48c51:	mov    0x20(%rsp),%rsi
   48c56:	call   f0c10 <core::slice::index::slice_index_fail>
   48c5b:	lea    0xfffb6(%rip),%rdx        # 148c18 <__frame_dummy_init_array_entry+0x23c8>
   48c62:	mov    $0x1e,%esi
   48c67:	mov    %r12,%rdi
   48c6a:	call   efe33 <core::panicking::panic_bounds_check>
   48c6f:	mov    $0x10,%edx
   48c74:	mov    $0x4,%ecx
   48c79:	mov    $0x400,%r8d
   48c7f:	mov    %rdi,%r15
   48c82:	mov    %r13,%rdi
   48c85:	xor    %esi,%esi
   48c87:	mov    %r9,%rbx
   48c8a:	call   6eeb0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48c8f:	mov    %r15,%rdi
   48c92:	mov    %rbx,%r9
   48c95:	jmp    477f9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2d9>
   48c9a:	mov    %r9,%r13
   48c9d:	mov    $0x8,%ecx
   48ca2:	mov    $0x8,%r8d
   48ca8:	mov    %rdi,%r15
   48cab:	xor    %esi,%esi
   48cad:	mov    %rbx,%rdx
   48cb0:	call   6eeb0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48cb5:	mov    %r15,%rdi
   48cb8:	mov    0x10(%r15),%r15
   48cbc:	mov    0x8(%rdi),%r12
   48cc0:	lea    (%r12,%r15,8),%rdi
   48cc4:	cmp    $0x1,%rbx
   48cc8:	jne    477a5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x285>
   48cce:	jmp    477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   48cd3:	lea    0xfddfe(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48cda:	call   efe33 <core::panicking::panic_bounds_check>
   48cdf:	lea    0xffe8a(%rip),%rdx        # 148b70 <__frame_dummy_init_array_entry+0x2320>
   48ce6:	mov    $0x1e,%esi
   48ceb:	call   efe33 <core::panicking::panic_bounds_check>
   48cf0:	lea    0xfff09(%rip),%rdx        # 148c00 <__frame_dummy_init_array_entry+0x23b0>
   48cf7:	mov    $0x1e,%esi
   48cfc:	call   efe33 <core::panicking::panic_bounds_check>
   48d01:	lea    0xffdf0(%rip),%rdx        # 148af8 <__frame_dummy_init_array_entry+0x22a8>
   48d08:	call   efe33 <core::panicking::panic_bounds_check>
   48d0d:	lea    0xfddc4(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48d14:	mov    %r13,%rdi
   48d17:	call   efe33 <core::panicking::panic_bounds_check>
