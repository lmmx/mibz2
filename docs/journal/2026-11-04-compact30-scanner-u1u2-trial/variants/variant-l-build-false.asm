
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
   47581:	mov    %rsi,0x58(%rsp)
   47586:	mov    0x5530(%rsp),%rax
   4758e:	movq   $0x0,0x10(%r9)
   47596:	movq   $0x0,0x10(%rax)
   4759e:	mov    %rdx,0x8(%rsp)
   475a3:	cmp    $0xeffe2,%rdx
   475aa:	ja     48a8d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x156d>
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
   4762e:	jne    48a8d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x156d>
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
   47752:	jbe    48bec <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16cc>
   47758:	mov    0x5530(%rsp),%rax
   47760:	mov    (%rax),%rax
   47763:	cmp    %rbx,%rax
   47766:	jb     48c41 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1721>
   4776c:	movq   $0x0,0x10(%rdi)
   47774:	lea    0x3f(%r13),%rbx
   47778:	shr    $0x6,%rbx
   4777c:	mov    %rdi,0x70(%rsp)
   47781:	mov    %r13,0x78(%rsp)
   47786:	je     477db <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2bb>
   47788:	cmp    (%rdi),%rbx
   4778b:	ja     48d4a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x182a>
   47791:	mov    %r9,%r13
   47794:	xor    %r15d,%r15d
   47797:	mov    0x8(%rdi),%r12
   4779b:	lea    (%r12,%r15,8),%rdi
   4779f:	cmp    $0x1,%rbx
   477a3:	je     477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   477a5:	lea    -0x8(,%rbx,8),%rdx
   477ad:	xor    %esi,%esi
   477af:	call   *0x105f4b(%rip)        # 14d700 <memset@GLIBC_2.2.5>
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
   477f3:	jbe    48d1f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17ff>
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
   47925:	call   *0x105df5(%rip)        # 14d720 <memcpy@GLIBC_2.14>
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
   47ab1:	jae    48db1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1891>
   47ab7:	mov    $0x1,%eax
   47abc:	mov    %r12d,%ecx
   47abf:	shl    %cl,%rax
   47ac2:	or     %rax,(%r9,%rdi,8)
   47ac6:	mov    %r12,%rcx
   47ac9:	mov    0x8(%rsp),%r11
   47ace:	movabs $0x888888888888889,%rdi
   47ad8:	jmp    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47add:	mov    %rsi,0x20(%rsp)
   47ae2:	mov    %r9,0x48(%rsp)
   47ae7:	call   9c280 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47aec:	pcmpeqd %xmm1,%xmm1
   47af0:	pxor   %xmm0,%xmm0
   47af4:	movabs $0xeeeeeeeeeeeeeeef,%r10
   47afe:	mov    0x48(%rsp),%r9
   47b03:	mov    0x20(%rsp),%rsi
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
   47c02:	jae    48dbd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x189d>
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
   47c94:	mov    %r8,0x20(%rsp)
   47c99:	mov    %r9,0x48(%rsp)
   47c9e:	mov    %r10,0x40(%rsp)
   47ca3:	mov    %rdi,0x68(%rsp)
   47ca8:	call   9c280 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47cad:	pcmpeqd %xmm1,%xmm1
   47cb1:	pxor   %xmm0,%xmm0
   47cb5:	mov    0x68(%rsp),%rdi
   47cba:	mov    0x40(%rsp),%r10
   47cbf:	mov    0x48(%rsp),%r9
   47cc4:	mov    0x20(%rsp),%r8
   47cc9:	mov    0x8(%rsp),%r11
   47cce:	jmp    47b67 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x647>
   47cd3:	test   %r11,%r11
   47cd6:	je     48a2b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x150b>
   47cdc:	movq   $0x0,0x18(%rsp)
   47ce5:	lea    0x28f8(%rsp),%r15
   47ced:	jmp    47d35 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x815>
   47cef:	nop
   47cf0:	lea    0x1(%rax),%rdx
   47cf4:	add    $0xffffffffffffffe3,%rax
   47cf8:	xor    %esi,%esi
   47cfa:	cmp    $0x1e,%rdx
   47cfe:	setae  %sil
   47d02:	cmovb  %rdx,%rax
   47d06:	lea    (%rcx,%rsi,2),%ecx
   47d09:	inc    %ecx
   47d0b:	mov    %rax,0x28e8(%rsp)
   47d13:	mov    %ecx,0x28f0(%rsp)
   47d1a:	mov    0x28(%rsp),%rbp
   47d1f:	cmp    %rdi,0x90(%rsp)
   47d27:	lea    0x28f8(%rsp),%r15
   47d2f:	jae    48a2b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x150b>
   47d35:	mov    0x18(%rsp),%rbx
   47d3a:	mov    $0x400,%edx
   47d3f:	mov    %r15,%rdi
   47d42:	xor    %esi,%esi
   47d44:	call   *0x1059b6(%rip)        # 14d700 <memset@GLIBC_2.2.5>
   47d4a:	lea    0x128(%rsp),%rax
   47d52:	xor    %ecx,%ecx
   47d54:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   47d60:	mov    -0x40(%rax),%rdx
   47d64:	sub    -0x30(%rax),%rdx
   47d68:	shr    $1,%rdx
   47d6b:	add    -0x28(%rax),%edx
   47d6e:	mov    -0x18(%rax),%rsi
   47d72:	sub    -0x8(%rax),%rsi
   47d76:	mov    %edx,0x28f8(%rsp,%rcx,4)
   47d7d:	shr    $1,%rsi
   47d80:	add    (%rax),%esi
   47d82:	mov    %esi,0x28fc(%rsp,%rcx,4)
   47d89:	add    $0x2,%rcx
   47d8d:	add    $0x50,%rax
   47d91:	cmp    $0x100,%rcx
   47d98:	jne    47d60 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x840>
   47d9a:	mov    0xc0(%rsp),%r12
   47da2:	mov    0x10(%r12),%r14
   47da7:	cmp    (%r12),%r14
   47dab:	je     48a1e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14fe>
   47db1:	mov    %r14,%rdi
   47db4:	shl    $0xa,%rdi
   47db8:	add    0x8(%r12),%rdi
   47dbd:	mov    $0x400,%edx
   47dc2:	mov    %r15,%rsi
   47dc5:	call   *0x105955(%rip)        # 14d720 <memcpy@GLIBC_2.14>
   47dcb:	inc    %r14
   47dce:	lea    0xf000(%rbx),%rax
   47dd5:	mov    0x8(%rsp),%rdi
   47dda:	cmp    %rdi,%rax
   47ddd:	mov    %rdi,%rcx
   47de0:	mov    %rax,0x90(%rsp)
   47de8:	cmovb  %rax,%rcx
   47dec:	mov    %r14,0x10(%r12)
   47df1:	mov    %rcx,0x18(%rsp)
   47df6:	mov    %rcx,%r9
   47df9:	sub    %rbx,%r9
   47dfc:	cmp    $0x10,%r9
   47e00:	lea    0xe8(%rsp),%r13
   47e08:	jb     48850 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1330>
   47e0e:	add    0x58(%rsp),%rbx
   47e13:	xor    %edx,%edx
   47e15:	jmp    47e72 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x952>
   47e17:	nopw   0x0(%rax,%rax,1)
   47e20:	mov    0x28e8(%rsp),%rax
   47e28:	lea    0x10(%rax),%rcx
   47e2c:	add    $0xfffffffffffffff2,%rax
   47e30:	xor    %edx,%edx
   47e32:	cmp    $0x1e,%rcx
   47e36:	setae  %dl
   47e39:	cmovb  %rcx,%rax
   47e3d:	mov    0x28f0(%rsp),%ecx
   47e44:	lea    (%rcx,%rdx,2),%ecx
   47e47:	add    $0x10,%ecx
   47e4a:	mov    %rax,0x28e8(%rsp)
   47e52:	mov    %ecx,0x28f0(%rsp)
   47e59:	xor    %edx,%edx
   47e5b:	mov    0x30(%rsp),%rbx
   47e60:	mov    0x38(%rsp),%rcx
   47e65:	mov    %rcx,%r9
   47e68:	cmp    $0xf,%rcx
   47e6c:	jbe    48853 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1333>
   47e72:	mov    0x60(%rsp),%rcx
   47e77:	test   $0x1,%dl
   47e7a:	jne    47e83 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x963>
   47e7c:	mov    (%rbx),%rcx
   47e7f:	mov    0x8(%rbx),%rbp
   47e83:	lea    0x10(%rbx),%r8
   47e87:	lea    -0x10(%r9),%rdx
   47e8b:	xor    %rcx,%rbp
   47e8e:	mov    %rcx,%rax
   47e91:	rol    $0x8,%rax
   47e95:	xor    %rcx,%rax
   47e98:	or     %rbp,%rax
   47e9b:	jne    47ef0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x9d0>
   47e9d:	cmp    $0x10,%rdx
   47ea1:	jb     47f70 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa50>
   47ea7:	nopw   0x0(%rax,%rax,1)
   47eb0:	mov    (%r8),%rsi
   47eb3:	mov    0x8(%r8),%rax
   47eb7:	mov    $0x1,%r10b
   47eba:	cmp    %rcx,%rsi
   47ebd:	jne    4809b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb7b>
   47ec3:	cmp    %rcx,%rax
   47ec6:	jne    4809b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb7b>
   47ecc:	add    $0xfffffffffffffff0,%rdx
   47ed0:	add    $0x10,%r8
   47ed4:	cmp    $0xf,%rdx
   47ed8:	ja     47eb0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x990>
   47eda:	mov    %r8,0x30(%rsp)
   47edf:	xor    %r10d,%r10d
   47ee2:	mov    %rcx,0x60(%rsp)
   47ee7:	jmp    480aa <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb8a>
   47eec:	nopl   0x0(%rax)
   47ef0:	mov    %r9,%rbp
   47ef3:	mov    %r8,0x30(%rsp)
   47ef8:	mov    %rdx,0x38(%rsp)
   47efd:	mov    0x28f0(%rsp),%r14d
   47f05:	mov    $0x1e,%r12d
   47f0b:	sub    0x28e8(%rsp),%r12
   47f13:	cmp    $0x10,%r12
   47f17:	mov    $0x10,%r15d
   47f1d:	cmovb  %r12,%r15
   47f21:	movzbl (%rbx),%esi
   47f24:	lea    (%rsi,%rsi,4),%rcx
   47f28:	mov    %r13,%rdx
   47f2b:	lea    0x0(%r13,%rcx,8),%r13
   47f30:	mov    (%rdx,%rcx,8),%rax
   47f34:	cmp    0x8(%rdx,%rcx,8),%rax
   47f39:	je     48803 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12e3>
   47f3f:	mov    0x18(%rsp),%rbx
   47f44:	sub    %rbp,%rbx
   47f47:	add    0x58(%rsp),%rbx
   47f4c:	mov    %r14w,(%rax)
   47f50:	addq   $0x2,0x0(%r13)
   47f55:	cmp    $0x1,%r12
   47f59:	ja     47f80 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa60>
   47f5b:	lea    0x10(%rbx),%r12
   47f5f:	lea    0xe8(%rsp),%r13
   47f67:	jmp    48036 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb16>
   47f6c:	nopl   0x0(%rax)
   47f70:	mov    %r8,0x30(%rsp)
   47f75:	xor    %r10d,%r10d
   47f78:	jmp    480aa <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb8a>
   47f7d:	nopl   (%rax)
   47f80:	movzbl 0x1(%rbx),%esi
   47f84:	lea    (%rsi,%rsi,4),%rcx
   47f88:	lea    0xe8(%rsp),%rdx
   47f90:	lea    (%rdx,%rcx,8),%r13
   47f94:	mov    (%rdx,%rcx,8),%rax
   47f98:	cmp    0x8(%rdx,%rcx,8),%rax
   47f9d:	je     48826 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1306>
   47fa3:	lea    0x1(%r14),%ecx
   47fa7:	mov    %cx,(%rax)
   47faa:	addq   $0x2,0x0(%r13)
   47faf:	mov    $0x2,%r13d
   47fb5:	cmp    %r13,%r15
   47fb8:	je     48020 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb00>
   47fba:	nopw   0x0(%rax,%rax,1)
   47fc0:	movzbl (%rbx,%r13,1),%esi
   47fc5:	lea    (%rsi,%rsi,4),%rcx
   47fc9:	lea    0xe8(%rsp),%rdx
   47fd1:	lea    (%rdx,%rcx,8),%rbp
   47fd5:	mov    (%rdx,%rcx,8),%rax
   47fd9:	cmp    0x8(%rdx,%rcx,8),%rax
   47fde:	je     47ff6 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xad6>
   47fe0:	lea    (%r14,%r13,1),%rcx
   47fe4:	mov    %cx,(%rax)
   47fe7:	addq   $0x2,0x0(%rbp)
   47fec:	inc    %r13
   47fef:	cmp    %r13,%r15
   47ff2:	jne    47fc0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaa0>
   47ff4:	jmp    48020 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb00>
   47ff6:	lea    0xa8(%rsp),%rdi
   47ffe:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48003:	test   %al,%al
   48005:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   4800b:	mov    0x0(%rbp),%rax
   4800f:	mov    0x8(%rsp),%rdi
   48014:	jmp    47fe0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xac0>
   48016:	cs nopw 0x0(%rax,%rax,1)
   48020:	cmp    $0xf,%r12
   48024:	lea    0xe8(%rsp),%r13
   4802c:	ja     47e20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x900>
   48032:	lea    0x10(%rbx),%r12
   48036:	add    %r15,%rbx
   48039:	lea    (%r14,%r15,1),%ebp
   4803d:	add    $0x2,%ebp
   48040:	movzbl (%rbx),%esi
   48043:	lea    (%rsi,%rsi,4),%rcx
   48047:	lea    0x0(,%rcx,8),%r14
   4804f:	add    %r13,%r14
   48052:	mov    0x0(%r13,%rcx,8),%rax
   48057:	cmp    0x8(%r13,%rcx,8),%rax
   4805c:	je     48074 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb54>
   4805e:	mov    %bp,(%rax)
   48061:	addq   $0x2,(%r14)
   48065:	inc    %rbx
   48068:	inc    %ebp
   4806a:	cmp    %r12,%rbx
   4806d:	jne    48040 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb20>
   4806f:	jmp    47e20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x900>
   48074:	lea    0xa8(%rsp),%rdi
   4807c:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48081:	test   %al,%al
   48083:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   48089:	mov    (%r14),%rax
   4808c:	mov    0x8(%rsp),%rdi
   48091:	lea    0xe8(%rsp),%r13
   48099:	jmp    4805e <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb3e>
   4809b:	mov    %r8,0x30(%rsp)
   480a0:	mov    %rsi,0x60(%rsp)
   480a5:	mov    %rax,0x28(%rsp)
   480aa:	mov    %rdx,0x38(%rsp)
   480af:	sub    %rdx,%r9
   480b2:	movzbl %cl,%eax
   480b5:	mov    %rax,0x98(%rsp)
   480bd:	lea    (%rax,%rax,4),%rdx
   480c1:	mov    0x0(%r13,%rdx,8),%rax
   480c6:	mov    0x1c(%r13,%rdx,8),%esi
   480cb:	mov    %rax,%rdi
   480ce:	sub    0x10(%r13,%rdx,8),%rdi
   480d3:	shr    $1,%rdi
   480d6:	add    0x18(%r13,%rdx,8),%edi
   480db:	add    %r9,%rdi
   480de:	cmp    %rsi,%rdi
   480e1:	ja     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   480e7:	mov    %r9,%r12
   480ea:	mov    %r10d,0x54(%rsp)
   480ef:	lea    0xe8(%rsp),%r13
   480f7:	lea    0x0(,%rdx,8),%r10
   480ff:	add    %r13,%r10
   48102:	movzbl %cl,%r15d
   48106:	mov    0x8(%r10),%rsi
   4810a:	mov    $0x88888889,%r14d
   48110:	lea    -0x2a7d7(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   48117:	mov    %r10,0x40(%rsp)
   4811c:	jmp    48150 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc30>
   4811e:	xchg   %ax,%ax
   48120:	mov    0x3c(%rsi),%edi
   48123:	mov    $0xfffff,%r9d
   48129:	and    %r9d,%edi
   4812c:	sub    %r8d,%ecx
   4812f:	sub    %eax,%r8d
   48132:	mov    %r8d,%eax
   48135:	lea    (%rsi,%rax,2),%rax
   48139:	mov    %rax,0x10(%r10)
   4813d:	mov    %rax,(%r10)
   48140:	lea    (%rax,%rcx,2),%rsi
   48144:	mov    %rsi,0x8(%r10)
   48148:	mov    %edx,0x20(%r10)
   4814c:	mov    %edi,0x24(%r10)
   48150:	sub    %rax,%rsi
   48153:	shr    $1,%rsi
   48156:	cmp    %rsi,%r12
   48159:	cmovb  %r12,%rsi
   4815d:	test   %rsi,%rsi
   48160:	je     4821d <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcfd>
   48166:	mov    0x28e8(%rsp),%rdi
   4816e:	cmp    $0x1d,%rdi
   48172:	ja     48d8f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x186f>
   48178:	cmp    $0x1f,%rsi
   4817c:	jae    48aa1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1581>
   48182:	mov    0x28f0(%rsp),%ecx
   48189:	mov    %rdi,%r11
   4818c:	shl    $0x6,%r11
   48190:	lea    (%rbx,%r11,1),%rdx
   48194:	lea    -0x2(,%rsi,2),%r9
   4819c:	cmp    $0xe,%r9
   481a0:	jb     481b5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc95>
   481a2:	mov    %rax,%r8
   481a5:	sub    %rbx,%r8
   481a8:	sub    %r11,%r8
   481ab:	cmp    $0x10,%r8
   481af:	jae    48608 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10e8>
   481b5:	mov    %rdx,%r8
   481b8:	xor    %r10d,%r10d
   481bb:	lea    (%rdx,%rsi,2),%rdx
   481bf:	lea    (%rax,%r10,2),%r9
   481c3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   481d0:	movzwl (%r8),%r10d
   481d4:	add    %cx,%r10w
   481d8:	add    $0x2,%r8
   481dc:	mov    %r10w,(%r9)
   481e0:	add    $0x2,%r9
   481e4:	cmp    %rdx,%r8
   481e7:	jne    481d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcb0>
   481e9:	lea    (%rax,%rsi,2),%rax
   481ed:	mov    0x40(%rsp),%r10
   481f2:	mov    %rax,(%r10)
   481f5:	add    %rsi,%rdi
   481f8:	lea    0x2(%rsi),%edx
   481fb:	mov    %rdi,%r8
   481fe:	sub    $0x1e,%r8
   48202:	cmovb  %rdi,%r8
   48206:	cmovb  %esi,%edx
   48209:	mov    %r8,0x28e8(%rsp)
   48211:	add    %ecx,%edx
   48213:	mov    %edx,0x28f0(%rsp)
   4821a:	sub    %rsi,%r12
   4821d:	test   %r12,%r12
   48220:	je     487f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12d0>
   48226:	mov    %rax,%r8
   48229:	cmp    0x10(%r10),%rax
   4822d:	je     48250 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xd30>
   4822f:	lea    0xa8(%rsp),%rdi
   48237:	mov    0x98(%rsp),%rsi
   4823f:	call   969d0 <<c30walk::ctor2su::W<false>>::finish>
   48244:	mov    0x40(%rsp),%r10
   48249:	mov    (%r10),%r8
   4824c:	mov    0x10(%r10),%rax
   48250:	sub    %rax,%r8
   48253:	shr    $1,%r8
   48256:	add    0x18(%r10),%r8d
   4825a:	mov    %r8d,0x18(%r10)
   4825e:	pxor   %xmm0,%xmm0
   48262:	movdqu %xmm0,(%r10)
   48267:	movq   $0x0,0x10(%r10)
   4826f:	cmp    $0x1e,%r12
   48273:	jb     48788 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1268>
   48279:	mov    %r8,%rcx
   4827c:	imul   %r14,%rcx
   48280:	shr    $0x24,%rcx
   48284:	mov    %ecx,%eax
   48286:	shl    $0x5,%eax
   48289:	lea    (%rcx,%rcx,1),%edx
   4828c:	sub    %eax,%edx
   4828e:	add    %r8d,%edx
   48291:	jne    48788 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1268>
   48297:	mov    %r12,%rax
   4829a:	movabs $0x8888888888888889,%rdx
   482a4:	mul    %rdx
   482a7:	shr    $0x4,%rdx
   482ab:	mov    %ecx,%r9d
   482ae:	cmpl   $0x0,0x14(%rsp)
   482b3:	mov    %r12,0x68(%rsp)
   482b8:	mov    %rdx,0xa0(%rsp)
   482c0:	je     485c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10a0>
   482c6:	mov    0x28f0(%rsp),%r10d
   482ce:	mov    0xb8(%rsp),%rax
   482d6:	mov    %rax,0x20(%rsp)
   482db:	mov    0xa8(%rsp),%r11
   482e3:	mov    0xb0(%rsp),%rax
   482eb:	mov    %rax,0x48(%rsp)
   482f0:	mov    0xd0(%rsp),%rax
   482f8:	mov    0x28e8(%rsp),%r12
   48300:	mov    %r12,%rdi
   48303:	shl    $0x6,%rdi
   48307:	add    %rbx,%rdi
   4830a:	mov    %r9,%rcx
   4830d:	shl    $0x6,%rcx
   48311:	lea    (%rax,%rcx,1),%rbx
   48315:	add    $0x3c,%rbx
   48319:	mov    %r10d,%r13d
   4831c:	shl    $0x8,%r13d
   48320:	or     %r15d,%r13d
   48323:	xor    %r14d,%r14d
   48326:	jmp    48534 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1014>
   4832b:	nopl   0x0(%rax,%rax,1)
   48330:	mov    0x8(%r11),%rcx
   48334:	shl    $0x7,%eax
   48337:	mov    %r13d,(%rcx,%rax,1)
   4833b:	movzwl 0x2(%rdi),%esi
   4833f:	lea    (%r10,%r14,1),%ebp
   48343:	add    %ebp,%esi
   48345:	shl    $0x8,%esi
   48348:	or     %r15d,%esi
   4834b:	mov    %esi,0x4(%rcx,%rax,1)
   4834f:	movzwl 0x4(%rdi),%esi
   48353:	add    %ebp,%esi
   48355:	shl    $0x8,%esi
   48358:	or     %r15d,%esi
   4835b:	mov    %esi,0x8(%rcx,%rax,1)
   4835f:	movzwl 0x6(%rdi),%esi
   48363:	add    %ebp,%esi
   48365:	shl    $0x8,%esi
   48368:	or     %r15d,%esi
   4836b:	mov    %esi,0xc(%rcx,%rax,1)
   4836f:	movzwl 0x8(%rdi),%esi
   48373:	add    %ebp,%esi
   48375:	shl    $0x8,%esi
   48378:	or     %r15d,%esi
   4837b:	mov    %esi,0x10(%rcx,%rax,1)
   4837f:	movzwl 0xa(%rdi),%esi
   48383:	add    %ebp,%esi
   48385:	shl    $0x8,%esi
   48388:	or     %r15d,%esi
   4838b:	mov    %esi,0x14(%rcx,%rax,1)
   4838f:	movzwl 0xc(%rdi),%esi
   48393:	add    %ebp,%esi
   48395:	shl    $0x8,%esi
   48398:	or     %r15d,%esi
   4839b:	mov    %esi,0x18(%rcx,%rax,1)
   4839f:	movzwl 0xe(%rdi),%esi
   483a3:	add    %ebp,%esi
   483a5:	shl    $0x8,%esi
   483a8:	or     %r15d,%esi
   483ab:	mov    %esi,0x1c(%rcx,%rax,1)
   483af:	movzwl 0x10(%rdi),%esi
   483b3:	add    %ebp,%esi
   483b5:	shl    $0x8,%esi
   483b8:	or     %r15d,%esi
   483bb:	mov    %esi,0x20(%rcx,%rax,1)
   483bf:	movzwl 0x12(%rdi),%esi
   483c3:	add    %ebp,%esi
   483c5:	shl    $0x8,%esi
   483c8:	or     %r15d,%esi
   483cb:	mov    %esi,0x24(%rcx,%rax,1)
   483cf:	movzwl 0x14(%rdi),%esi
   483d3:	add    %ebp,%esi
   483d5:	shl    $0x8,%esi
   483d8:	or     %r15d,%esi
   483db:	mov    %esi,0x28(%rcx,%rax,1)
   483df:	movzwl 0x16(%rdi),%esi
   483e3:	add    %ebp,%esi
   483e5:	shl    $0x8,%esi
   483e8:	or     %r15d,%esi
   483eb:	mov    %esi,0x2c(%rcx,%rax,1)
   483ef:	movzwl 0x18(%rdi),%esi
   483f3:	add    %ebp,%esi
   483f5:	shl    $0x8,%esi
   483f8:	or     %r15d,%esi
   483fb:	mov    %esi,0x30(%rcx,%rax,1)
   483ff:	movzwl 0x1a(%rdi),%esi
   48403:	add    %ebp,%esi
   48405:	shl    $0x8,%esi
   48408:	or     %r15d,%esi
   4840b:	mov    %esi,0x34(%rcx,%rax,1)
   4840f:	movzwl 0x1c(%rdi),%esi
   48413:	add    %ebp,%esi
   48415:	shl    $0x8,%esi
   48418:	or     %r15d,%esi
   4841b:	mov    %esi,0x38(%rcx,%rax,1)
   4841f:	movzwl 0x1e(%rdi),%esi
   48423:	add    %ebp,%esi
   48425:	shl    $0x8,%esi
   48428:	or     %r15d,%esi
   4842b:	mov    %esi,0x3c(%rcx,%rax,1)
   4842f:	movzwl 0x20(%rdi),%esi
   48433:	add    %ebp,%esi
   48435:	shl    $0x8,%esi
   48438:	or     %r15d,%esi
   4843b:	mov    %esi,0x40(%rcx,%rax,1)
   4843f:	movzwl 0x22(%rdi),%esi
   48443:	add    %ebp,%esi
   48445:	shl    $0x8,%esi
   48448:	or     %r15d,%esi
   4844b:	mov    %esi,0x44(%rcx,%rax,1)
   4844f:	movzwl 0x24(%rdi),%esi
   48453:	add    %ebp,%esi
   48455:	shl    $0x8,%esi
   48458:	or     %r15d,%esi
   4845b:	mov    %esi,0x48(%rcx,%rax,1)
   4845f:	movzwl 0x26(%rdi),%esi
   48463:	add    %ebp,%esi
   48465:	shl    $0x8,%esi
   48468:	or     %r15d,%esi
   4846b:	mov    %esi,0x4c(%rcx,%rax,1)
   4846f:	movzwl 0x28(%rdi),%esi
   48473:	add    %ebp,%esi
   48475:	shl    $0x8,%esi
   48478:	or     %r15d,%esi
   4847b:	mov    %esi,0x50(%rcx,%rax,1)
   4847f:	movzwl 0x2a(%rdi),%esi
   48483:	add    %ebp,%esi
   48485:	shl    $0x8,%esi
   48488:	or     %r15d,%esi
   4848b:	mov    %esi,0x54(%rcx,%rax,1)
   4848f:	movzwl 0x2c(%rdi),%esi
   48493:	add    %ebp,%esi
   48495:	shl    $0x8,%esi
   48498:	or     %r15d,%esi
   4849b:	mov    %esi,0x58(%rcx,%rax,1)
   4849f:	movzwl 0x2e(%rdi),%esi
   484a3:	add    %ebp,%esi
   484a5:	shl    $0x8,%esi
   484a8:	or     %r15d,%esi
   484ab:	mov    %esi,0x5c(%rcx,%rax,1)
   484af:	movzwl 0x30(%rdi),%esi
   484b3:	add    %ebp,%esi
   484b5:	shl    $0x8,%esi
   484b8:	or     %r15d,%esi
   484bb:	mov    %esi,0x60(%rcx,%rax,1)
   484bf:	movzwl 0x32(%rdi),%esi
   484c3:	add    %ebp,%esi
   484c5:	shl    $0x8,%esi
   484c8:	or     %r15d,%esi
   484cb:	mov    %esi,0x64(%rcx,%rax,1)
   484cf:	movzwl 0x34(%rdi),%esi
   484d3:	add    %ebp,%esi
   484d5:	shl    $0x8,%esi
   484d8:	or     %r15d,%esi
   484db:	mov    %esi,0x68(%rcx,%rax,1)
   484df:	movzwl 0x36(%rdi),%esi
   484e3:	add    %ebp,%esi
   484e5:	shl    $0x8,%esi
   484e8:	or     %r15d,%esi
   484eb:	mov    %esi,0x6c(%rcx,%rax,1)
   484ef:	movzwl 0x38(%rdi),%esi
   484f3:	add    %ebp,%esi
   484f5:	shl    $0x8,%esi
   484f8:	or     %r15d,%esi
   484fb:	mov    %esi,0x70(%rcx,%rax,1)
   484ff:	movzwl 0x3a(%rdi),%esi
   48503:	add    %ebp,%esi
   48505:	shl    $0x8,%esi
   48508:	or     %r15d,%esi
   4850b:	mov    %esi,0x74(%rcx,%rax,1)
   4850f:	lea    (%r10,%r14,1),%eax
   48513:	add    $0x20,%eax
   48516:	mov    %eax,0x28f0(%rsp)
   4851d:	add    $0x20,%r14
   48521:	inc    %r9
   48524:	add    $0x2000,%r13d
   4852b:	dec    %rdx
   4852e:	je     4873c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x121c>
   48534:	mov    %r9,%rax
   48537:	shr    $0x6,%rax
   4853b:	cmp    0x20(%rsp),%rax
   48540:	jae    48ca5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1785>
   48546:	mov    0x48(%rsp),%rcx
   4854b:	mov    (%rcx,%rax,8),%rax
   4854f:	bt     %r9,%rax
   48553:	jae    48580 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1060>
   48555:	cmp    $0x1e,%r12
   48559:	jae    48ccb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17ab>
   4855f:	mov    (%rbx,%r14,2),%eax
   48563:	and    $0xfffff,%eax
   48568:	mov    0x10(%r11),%rcx
   4856c:	cmp    %rax,%rcx
   4856f:	ja     48330 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xe10>
   48575:	jmp    48cb9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1799>
   4857a:	nopw   0x0(%rax,%rax,1)
   48580:	cmp    $0x1e,%r12
   48584:	jae    48d0b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17eb>
   4858a:	movups 0x2c(%rdi),%xmm0
   4858e:	movups %xmm0,-0x10(%rbx,%r14,2)
   48594:	movdqa (%rdi),%xmm0
   48598:	movdqa 0x10(%rdi),%xmm1
   4859d:	movdqa 0x20(%rdi),%xmm2
   485a2:	movdqa %xmm2,-0x1c(%rbx,%r14,2)
   485a9:	movdqa %xmm1,-0x2c(%rbx,%r14,2)
   485b0:	movdqa %xmm0,-0x3c(%rbx,%r14,2)
   485b7:	mov    %r13d,(%rbx,%r14,2)
   485bb:	jmp    4850f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xfef>
   485c0:	mov    0x28e8(%rsp),%rdi
   485c8:	cmp    $0x1d,%rdi
   485cc:	ja     48da0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1880>
   485d2:	shl    $0x6,%rdi
   485d6:	add    %rbx,%rdi
   485d9:	mov    0x28f0(%rsp),%eax
   485e0:	mov    0xd0(%rsp),%rcx
   485e8:	shl    $0x6,%r9
   485ec:	mov    %eax,%r14d
   485ef:	shl    $0x8,%r14d
   485f3:	or     %r15d,%r14d
   485f6:	lea    -0x1e(%r12),%rsi
   485fb:	cmp    $0x1e,%rsi
   485ff:	jae    4866c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x114c>
   48601:	xor    %esi,%esi
   48603:	jmp    486fa <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11da>
   48608:	shr    $1,%r9
   4860b:	inc    %r9
   4860e:	mov    %r9,%r10
   48611:	and    $0xfffffffffffffff8,%r10
   48615:	lea    (%rdx,%r10,2),%r8
   48619:	movd   %ecx,%xmm0
   4861d:	pshuflw $0x0,%xmm0,%xmm0
   48622:	add    %rbx,%r11
   48625:	add    $0x8,%r11
   48629:	xor    %ebx,%ebx
   4862b:	nopl   0x0(%rax,%rax,1)
   48630:	movq   -0x8(%r11,%rbx,2),%xmm1
   48637:	movq   (%r11,%rbx,2),%xmm2
   4863d:	paddw  %xmm0,%xmm1
   48641:	paddw  %xmm0,%xmm2
   48645:	punpcklqdq %xmm2,%xmm1
   48649:	movdqu %xmm1,(%rax,%rbx,2)
   4864e:	add    $0x8,%rbx
   48652:	cmp    %rbx,%r10
   48655:	jne    48630 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1110>
   48657:	cmp    %r10,%r9
   4865a:	lea    -0x2ad21(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   48661:	jne    481bb <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc9b>
   48667:	jmp    481e9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcc9>
   4866c:	mov    %rdx,%r10
   4866f:	movabs $0xffffffffffffffe,%rsi
   48679:	and    %rsi,%r10
   4867c:	lea    (%rcx,%r9,1),%r11
   48680:	add    $0x7c,%r11
   48684:	xor    %esi,%esi
   48686:	cs nopw 0x0(%rax,%rax,1)
   48690:	movdqu 0x2c(%rdi),%xmm0
   48695:	movdqu %xmm0,-0x50(%r11)
   4869b:	movdqa (%rdi),%xmm1
   4869f:	movdqa 0x10(%rdi),%xmm2
   486a4:	movdqa 0x20(%rdi),%xmm3
   486a9:	movdqa %xmm3,-0x5c(%r11)
   486af:	movdqa %xmm2,-0x6c(%r11)
   486b5:	movdqa %xmm1,-0x7c(%r11)
   486bb:	mov    %r14d,-0x40(%r11)
   486bf:	add    $0x2,%rsi
   486c3:	movdqa %xmm1,-0x3c(%r11)
   486c9:	movdqa %xmm2,-0x2c(%r11)
   486cf:	movdqa %xmm3,-0x1c(%r11)
   486d5:	movdqu %xmm0,-0x10(%r11)
   486db:	lea    0x2000(%r14),%ebx
   486e2:	mov    %ebx,(%r11)
   486e5:	add    $0x4000,%r14d
   486ec:	sub    $0xffffffffffffff80,%r11
   486f0:	cmp    %rsi,%r10
   486f3:	jne    48690 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1170>
   486f5:	test   $0x1,%dl
   486f8:	je     4872e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x120e>
   486fa:	add    %r9,%rcx
   486fd:	shl    $0x6,%rsi
   48701:	movups 0x2c(%rdi),%xmm0
   48705:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4870a:	movdqa (%rdi),%xmm0
   4870e:	movdqa 0x10(%rdi),%xmm1
   48713:	movdqa 0x20(%rdi),%xmm2
   48718:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   4871e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   48724:	movdqa %xmm0,(%rcx,%rsi,1)
   48729:	mov    %r14d,0x3c(%rcx,%rsi,1)
   4872e:	mov    %edx,%ecx
   48730:	shl    $0x5,%ecx
   48733:	add    %ecx,%eax
   48735:	mov    %eax,0x28f0(%rsp)
   4873c:	mov    0xa0(%rsp),%rdx
   48744:	mov    %rdx,%rax
   48747:	shl    $0x5,%rax
   4874b:	lea    (%rdx,%rdx,1),%rcx
   4874f:	sub    %rax,%rcx
   48752:	mov    0x68(%rsp),%r12
   48757:	add    %rcx,%r12
   4875a:	lea    (%rdx,%rdx,1),%eax
   4875d:	shl    $0x5,%edx
   48760:	sub    %eax,%edx
   48762:	add    %edx,%r8d
   48765:	mov    0x40(%rsp),%r10
   4876a:	mov    %r8d,0x18(%r10)
   4876e:	test   %r12,%r12
   48771:	lea    0xe8(%rsp),%r13
   48779:	mov    $0x88888889,%r14d
   4877f:	lea    -0x2ae46(%rip),%rbx        # 1d940 <c30walk::ctor2su::PATTERNS>
   48786:	je     487f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12d0>
   48788:	mov    0x1c(%r10),%ecx
   4878c:	mov    %r8d,%edi
   4878f:	imul   %r14,%rdi
   48793:	mov    %rdi,%rdx
   48796:	shr    $0x24,%rdx
   4879a:	mov    %edx,%eax
   4879c:	shl    $0x5,%eax
   4879f:	sub    %edx,%eax
   487a1:	sub    %edx,%eax
   487a3:	lea    0x1e(%rax),%esi
   487a6:	cmp    %ecx,%esi
   487a8:	cmovb  %esi,%ecx
   487ab:	shr    $0x2a,%rdi
   487af:	mov    0xb8(%rsp),%rsi
   487b7:	cmp    %rsi,%rdi
   487ba:	jae    48d83 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1863>
   487c0:	mov    %rdx,%rsi
   487c3:	shl    $0x6,%rsi
   487c7:	add    0xd0(%rsp),%rsi
   487cf:	mov    0xb0(%rsp),%r9
   487d7:	mov    (%r9,%rdi,8),%rdi
   487db:	bt     %rdx,%rdi
   487df:	jb     48120 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc00>
   487e5:	mov    $0xffffffff,%edi
   487ea:	jmp    4812c <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc0c>
   487ef:	nop
   487f0:	mov    0x8(%rsp),%rdi
   487f5:	mov    0x28(%rsp),%rbp
   487fa:	mov    0x54(%rsp),%edx
   487fe:	jmp    47e5b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93b>
   48803:	lea    0xa8(%rsp),%rdi
   4880b:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48810:	test   %al,%al
   48812:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   48818:	mov    0x0(%r13),%rax
   4881c:	mov    0x8(%rsp),%rdi
   48821:	jmp    47f3f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa1f>
   48826:	lea    0xa8(%rsp),%rdi
   4882e:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48833:	test   %al,%al
   48835:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   4883b:	mov    0x0(%r13),%rax
   4883f:	mov    0x8(%rsp),%rdi
   48844:	jmp    47fa3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa83>
   48849:	nopl   0x0(%rax)
   48850:	mov    %r9,%rcx
   48853:	test   %rcx,%rcx
   48856:	je     47d1f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7ff>
   4885c:	mov    0x18(%rsp),%r12
   48861:	sub    %rcx,%r12
   48864:	mov    0x28e8(%rsp),%rax
   4886c:	mov    $0x1e,%ebx
   48871:	sub    %rax,%rbx
   48874:	cmp    %rcx,%rbx
   48877:	cmovae %rcx,%rbx
   4887b:	mov    %rbx,%r13
   4887e:	add    %r12,%r13
   48881:	jb     48cdf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17bf>
   48887:	cmp    %rdi,%r13
   4888a:	ja     48cdf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17bf>
   48890:	mov    %rbp,0x28(%rsp)
   48895:	mov    %rcx,0x38(%rsp)
   4889a:	mov    0x28f0(%rsp),%r14d
   488a2:	cmp    $0x1e,%rax
   488a6:	jne    4893f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x141f>
   488ac:	cmp    %r13,0x18(%rsp)
   488b1:	jb     48cf4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17d4>
   488b7:	lea    0xe8(%rsp),%rdx
   488bf:	jne    489a9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1489>
   488c5:	mov    0x28e8(%rsp),%rax
   488cd:	mov    0x28f0(%rsp),%ecx
   488d4:	mov    0x38(%rsp),%r8
   488d9:	cmp    $0x1,%r8
   488dd:	je     47cf0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7d0>
   488e3:	mov    %r8d,%edx
   488e6:	and    $0xe,%edx
   488e9:	nopl   0x0(%rax)
   488f0:	lea    0x1(%rax),%rsi
   488f4:	add    $0xffffffffffffffe3,%rax
   488f8:	xor    %edi,%edi
   488fa:	cmp    $0x1e,%rsi
   488fe:	setae  %dil
   48902:	cmovb  %rsi,%rax
   48906:	lea    (%rcx,%rdi,2),%ecx
   48909:	lea    0x1(%rax),%rsi
   4890d:	add    $0xffffffffffffffe3,%rax
   48911:	xor    %edi,%edi
   48913:	cmp    $0x1e,%rsi
   48917:	setae  %dil
   4891b:	cmovb  %rsi,%rax
   4891f:	lea    (%rcx,%rdi,2),%ecx
   48922:	add    $0x2,%ecx
   48925:	add    $0xfffffffffffffffe,%rdx
   48929:	jne    488f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13d0>
   4892b:	test   $0x1,%r8b
   4892f:	mov    0x8(%rsp),%rdi
   48934:	jne    47cf0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7d0>
   4893a:	jmp    47d0b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7eb>
   4893f:	add    0x58(%rsp),%r12
   48944:	xor    %r15d,%r15d
   48947:	nopw   0x0(%rax,%rax,1)
   48950:	movzbl (%r12,%r15,1),%esi
   48955:	lea    (%rsi,%rsi,4),%rcx
   48959:	lea    0xe8(%rsp),%rdx
   48961:	lea    (%rdx,%rcx,8),%rbp
   48965:	mov    (%rdx,%rcx,8),%rax
   48969:	cmp    0x8(%rdx,%rcx,8),%rax
   4896e:	je     48989 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1469>
   48970:	lea    (%r14,%r15,1),%rcx
   48974:	mov    %cx,(%rax)
   48977:	addq   $0x2,0x0(%rbp)
   4897c:	inc    %r15
   4897f:	cmp    %r15,%rbx
   48982:	jne    48950 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1430>
   48984:	jmp    488ac <<c30walk::ctor2su::Ctor2su>::build::<false>+0x138c>
   48989:	lea    0xa8(%rsp),%rdi
   48991:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48996:	test   %al,%al
   48998:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   4899e:	mov    0x0(%rbp),%rax
   489a2:	mov    0x8(%rsp),%rdi
   489a7:	jmp    48970 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1450>
   489a9:	mov    0x58(%rsp),%rax
   489ae:	mov    0x18(%rsp),%rcx
   489b3:	lea    (%rax,%rcx,1),%r15
   489b7:	lea    (%r14,%rbx,1),%ebp
   489bb:	add    $0x2,%ebp
   489be:	sub    0x38(%rsp),%rbx
   489c3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   489d0:	movzbl (%r15,%rbx,1),%esi
   489d5:	lea    (%rsi,%rsi,4),%rcx
   489d9:	lea    (%rdx,%rcx,8),%r14
   489dd:	mov    (%rdx,%rcx,8),%rax
   489e1:	cmp    0x8(%rdx,%rcx,8),%rax
   489e6:	je     489fb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14db>
   489e8:	mov    %bp,(%rax)
   489eb:	addq   $0x2,(%r14)
   489ef:	inc    %ebp
   489f1:	inc    %rbx
   489f4:	jne    489d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14b0>
   489f6:	jmp    488c5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13a5>
   489fb:	lea    0xa8(%rsp),%rdi
   48a03:	call   96fd0 <<c30walk::ctor2su::W<false>>::refill>
   48a08:	test   %al,%al
   48a0a:	je     48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   48a0c:	mov    (%r14),%rax
   48a0f:	mov    0x8(%rsp),%rdi
   48a14:	lea    0xe8(%rsp),%rdx
   48a1c:	jmp    489e8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14c8>
   48a1e:	mov    %r12,%rdi
   48a21:	call   feb80 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   48a26:	jmp    47db1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x891>
   48a2b:	mov    $0x84,%eax
   48a30:	lea    0x50(%rax),%rcx
   48a34:	cmp    $0x28d4,%rcx
   48a3b:	je     48ab4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1594>
   48a3d:	mov    0x64(%rsp,%rax,1),%rdx
   48a42:	sub    0x74(%rsp,%rax,1),%rdx
   48a47:	shr    $1,%rdx
   48a4a:	add    0x7c(%rsp,%rax,1),%edx
   48a4e:	cmp    0x80(%rsp,%rax,1),%edx
   48a55:	jne    48a7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x155d>
   48a57:	mov    0x8c(%rsp,%rax,1),%rdx
   48a5f:	sub    0x9c(%rsp,%rax,1),%rdx
   48a67:	shr    $1,%rdx
   48a6a:	add    0xa4(%rsp,%rax,1),%edx
   48a71:	cmp    0xa8(%rsp,%rax,1),%edx
   48a78:	mov    %rcx,%rax
   48a7b:	je     48a30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1510>
   48a7d:	mov    0x5530(%rsp),%rax
   48a85:	movq   $0x0,0x10(%rax)
   48a8d:	xor    %eax,%eax
   48a8f:	add    $0x54f8,%rsp
   48a96:	pop    %rbx
   48a97:	pop    %r12
   48a99:	pop    %r13
   48a9b:	pop    %r14
   48a9d:	pop    %r15
   48a9f:	pop    %rbp
   48aa0:	ret
   48aa1:	lea    0x100230(%rip),%rcx        # 148cd8 <__frame_dummy_init_array_entry+0x2338>
   48aa8:	mov    $0x1e,%edx
   48aad:	xor    %edi,%edi
   48aaf:	call   f0d60 <core::slice::index::slice_index_fail>
   48ab4:	xor    %r14d,%r14d
   48ab7:	lea    0xa8(%rsp),%r15
   48abf:	mov    0x80(%rsp),%r12
   48ac7:	mov    0x8(%rsp),%rdx
   48acc:	mov    0x78(%rsp),%r13
   48ad1:	lea    0xe8(%rsp),%rbx
   48ad9:	jmp    48b09 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15e9>
   48adb:	nopl   0x0(%rax,%rax,1)
   48ae0:	sub    %rcx,%rax
   48ae3:	inc    %r14
   48ae6:	shr    $1,%rax
   48ae9:	add    %eax,0x18(%rbx)
   48aec:	pxor   %xmm0,%xmm0
   48af0:	movdqu %xmm0,(%rbx)
   48af4:	movq   $0x0,0x10(%rbx)
   48afc:	add    $0x28,%rbx
   48b00:	cmp    $0x100,%r14
   48b07:	je     48b2e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x160e>
   48b09:	mov    (%rbx),%rcx
   48b0c:	mov    %rcx,%rax
   48b0f:	cmp    0x10(%rbx),%rcx
   48b13:	je     48ae0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15c0>
   48b15:	mov    %r15,%rdi
   48b18:	mov    %r14,%rsi
   48b1b:	call   969d0 <<c30walk::ctor2su::W<false>>::finish>
   48b20:	mov    0x8(%rsp),%rdx
   48b25:	mov    (%rbx),%rax
   48b28:	mov    0x10(%rbx),%rcx
   48b2c:	jmp    48ae0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15c0>
   48b2e:	xor    %ecx,%ecx
   48b30:	mov    0x8(%r12),%rbx
   48b35:	test   %r13,%r13
   48b38:	setne  %al
   48b3b:	je     48b7f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x165f>
   48b3d:	mov    %al,%cl
   48b3f:	mov    %r13,%rax
   48b42:	sub    %rcx,%rax
   48b45:	imul   $0xffffffffffffffe2,%rax,%rcx
   48b49:	add    %rdx,%rcx
   48b4c:	cmp    $0x1e,%rcx
   48b50:	jae    48b7f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x165f>
   48b52:	lea    (%rbx,%r13,4),%rcx
   48b56:	lea    (%rcx,%rdx,2),%rdi
   48b5a:	add    $0xfffffffffffffffc,%rdi
   48b5e:	lea    (%rax,%rax,1),%rcx
   48b62:	shl    $0x5,%rax
   48b66:	sub    %rcx,%rax
   48b69:	sub    %rdx,%rax
   48b6c:	lea    0x3c(,%rax,2),%rdx
   48b74:	mov    $0xff,%esi
   48b79:	call   *0x104b81(%rip)        # 14d700 <memset@GLIBC_2.2.5>
   48b7f:	shl    $0x6,%r13
   48b83:	pcmpeqd %xmm0,%xmm0
   48b87:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   48b8e:	movdqa %xmm0,0x20(%rbx,%r13,1)
   48b95:	movdqa %xmm0,0x10(%rbx,%r13,1)
   48b9c:	movdqa %xmm0,(%rbx,%r13,1)
   48ba2:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   48bab:	mov    0x70(%rsp),%rax
   48bb0:	mov    0x8(%rax),%rdx
   48bb4:	mov    0x10(%rax),%rcx
   48bb8:	mov    0x5530(%rsp),%rax
   48bc0:	mov    0x8(%rax),%r8
   48bc4:	mov    0x10(%rax),%r9
   48bc8:	lea    0xa8(%rsp),%rdi
   48bd0:	mov    %rbx,%rsi
   48bd3:	call   a5960 <c30walk::ctor::canonicalise>
   48bd8:	mov    0x88(%rsp),%rax
   48be0:	mov    %rax,0x10(%r12)
   48be5:	mov    $0x1,%al
   48be7:	jmp    48a8f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x156f>
   48bec:	mov    %rdi,%r12
   48bef:	test   %rax,%rax
   48bf2:	je     48c04 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16e4>
   48bf4:	mov    0x8(%r9),%rdi
   48bf8:	mov    %r9,%r15
   48bfb:	call   *0x104b0f(%rip)        # 14d710 <free@GLIBC_2.2.5>
   48c01:	mov    %r15,%r9
   48c04:	movq   $0x0,(%r9)
   48c0b:	movq   $0x40,0x8(%r9)
   48c13:	movq   $0x0,0x10(%r9)
   48c1b:	mov    $0x40,%edx
   48c20:	mov    %r9,%rdi
   48c23:	lea    0x1(%r13),%rsi
   48c27:	mov    %r9,%r15
   48c2a:	call   9b530 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48c2f:	mov    %r15,%r9
   48c32:	cmp    $0xffffffffffffffff,%rax
   48c36:	mov    %r12,%rdi
   48c39:	je     47758 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x238>
   48c3f:	jmp    48c9a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x177a>
   48c41:	mov    %rdi,%r12
   48c44:	mov    %r9,%r15
   48c47:	test   %rax,%rax
   48c4a:	je     48c5e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x173e>
   48c4c:	mov    0x5530(%rsp),%rax
   48c54:	mov    0x8(%rax),%rdi
   48c58:	call   *0x104ab2(%rip)        # 14d710 <free@GLIBC_2.2.5>
   48c5e:	mov    0x5530(%rsp),%rdi
   48c66:	movq   $0x0,(%rdi)
   48c6d:	movq   $0x40,0x8(%rdi)
   48c75:	movq   $0x0,0x10(%rdi)
   48c7d:	mov    $0x80,%edx
   48c82:	mov    %rbx,%rsi
   48c85:	call   9b530 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48c8a:	cmp    $0xffffffffffffffff,%rax
   48c8e:	mov    %r15,%r9
   48c91:	mov    %r12,%rdi
   48c94:	je     4776c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x24c>
   48c9a:	mov    %rax,%rdi
   48c9d:	mov    %rdx,%rsi
   48ca0:	call   ee4eb <alloc::raw_vec::handle_error>
   48ca5:	lea    0xfdf7c(%rip),%rdx        # 146c28 <__frame_dummy_init_array_entry+0x288>
   48cac:	mov    %rax,%rdi
   48caf:	mov    0x20(%rsp),%rsi
   48cb4:	call   eff83 <core::panicking::panic_bounds_check>
   48cb9:	lea    0x1000d8(%rip),%rdx        # 148d98 <__frame_dummy_init_array_entry+0x23f8>
   48cc0:	mov    %rax,%rdi
   48cc3:	mov    %rcx,%rsi
   48cc6:	call   eff83 <core::panicking::panic_bounds_check>
   48ccb:	lea    0x1000ae(%rip),%rdx        # 148d80 <__frame_dummy_init_array_entry+0x23e0>
   48cd2:	mov    $0x1e,%esi
   48cd7:	mov    %r12,%rdi
   48cda:	call   eff83 <core::panicking::panic_bounds_check>
   48cdf:	lea    0x100022(%rip),%rcx        # 148d08 <__frame_dummy_init_array_entry+0x2368>
   48ce6:	mov    %rdi,%rdx
   48ce9:	mov    %r12,%rdi
   48cec:	mov    %r13,%rsi
   48cef:	call   f0d60 <core::slice::index::slice_index_fail>
   48cf4:	lea    0xffff5(%rip),%rcx        # 148cf0 <__frame_dummy_init_array_entry+0x2350>
   48cfb:	mov    %rdi,%rdx
   48cfe:	mov    %r13,%rdi
   48d01:	mov    0x18(%rsp),%rsi
   48d06:	call   f0d60 <core::slice::index::slice_index_fail>
   48d0b:	lea    0x100056(%rip),%rdx        # 148d68 <__frame_dummy_init_array_entry+0x23c8>
   48d12:	mov    $0x1e,%esi
   48d17:	mov    %r12,%rdi
   48d1a:	call   eff83 <core::panicking::panic_bounds_check>
   48d1f:	mov    $0x10,%edx
   48d24:	mov    $0x4,%ecx
   48d29:	mov    $0x400,%r8d
   48d2f:	mov    %rdi,%r15
   48d32:	mov    %r13,%rdi
   48d35:	xor    %esi,%esi
   48d37:	mov    %r9,%rbx
   48d3a:	call   6f000 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48d3f:	mov    %r15,%rdi
   48d42:	mov    %rbx,%r9
   48d45:	jmp    477f9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2d9>
   48d4a:	mov    %r9,%r13
   48d4d:	mov    $0x8,%ecx
   48d52:	mov    $0x8,%r8d
   48d58:	mov    %rdi,%r15
   48d5b:	xor    %esi,%esi
   48d5d:	mov    %rbx,%rdx
   48d60:	call   6f000 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48d65:	mov    %r15,%rdi
   48d68:	mov    0x10(%r15),%r15
   48d6c:	mov    0x8(%rdi),%r12
   48d70:	lea    (%r12,%r15,8),%rdi
   48d74:	cmp    $0x1,%rbx
   48d78:	jne    477a5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x285>
   48d7e:	jmp    477c7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   48d83:	lea    0xfde9e(%rip),%rdx        # 146c28 <__frame_dummy_init_array_entry+0x288>
   48d8a:	call   eff83 <core::panicking::panic_bounds_check>
   48d8f:	lea    0xfff2a(%rip),%rdx        # 148cc0 <__frame_dummy_init_array_entry+0x2320>
   48d96:	mov    $0x1e,%esi
   48d9b:	call   eff83 <core::panicking::panic_bounds_check>
   48da0:	lea    0xfffa9(%rip),%rdx        # 148d50 <__frame_dummy_init_array_entry+0x23b0>
   48da7:	mov    $0x1e,%esi
   48dac:	call   eff83 <core::panicking::panic_bounds_check>
   48db1:	lea    0xffe90(%rip),%rdx        # 148c48 <__frame_dummy_init_array_entry+0x22a8>
   48db8:	call   eff83 <core::panicking::panic_bounds_check>
   48dbd:	lea    0xfde64(%rip),%rdx        # 146c28 <__frame_dummy_init_array_entry+0x288>
   48dc4:	mov    %r13,%rdi
   48dc7:	call   eff83 <core::panicking::panic_bounds_check>
