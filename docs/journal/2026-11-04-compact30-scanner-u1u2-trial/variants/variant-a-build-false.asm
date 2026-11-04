
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
   4759e:	cmp    $0xeffe2,%rdx
   475a5:	ja     489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   475ab:	mov    %rcx,%r14
   475ae:	pxor   %xmm0,%xmm0
   475b2:	xor    %eax,%eax
   475b4:	pxor   %xmm2,%xmm2
   475b8:	pxor   %xmm1,%xmm1
   475bc:	nopl   0x0(%rax)
   475c0:	movq   (%r14,%rax,4),%xmm3
   475c6:	movq   0x8(%r14,%rax,4),%xmm4
   475cd:	punpckldq %xmm0,%xmm3
   475d1:	paddq  %xmm2,%xmm3
   475d5:	punpckldq %xmm0,%xmm4
   475d9:	paddq  %xmm1,%xmm4
   475dd:	movq   0x10(%r14,%rax,4),%xmm2
   475e4:	movq   0x18(%r14,%rax,4),%xmm1
   475eb:	punpckldq %xmm0,%xmm2
   475ef:	paddq  %xmm3,%xmm2
   475f3:	punpckldq %xmm0,%xmm1
   475f7:	paddq  %xmm4,%xmm1
   475fb:	add    $0x8,%rax
   475ff:	cmp    $0x100,%rax
   47605:	jne    475c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa0>
   47607:	paddq  %xmm2,%xmm1
   4760b:	pshufd $0xee,%xmm1,%xmm0
   47610:	paddq  %xmm1,%xmm0
   47614:	movq   %xmm0,%rax
   47619:	cmp    %rdx,%rax
   4761c:	jne    489cd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ad>
   47622:	pxor   %xmm2,%xmm2
   47626:	xor    %eax,%eax
   47628:	movdqa -0x36ad0(%rip),%xmm3        # 10b60 <GCC_except_table1006+0xa08>
   47630:	pxor   %xmm1,%xmm1
   47634:	pxor   %xmm0,%xmm0
   47638:	nopl   0x0(%rax,%rax,1)
   47640:	movq   (%r14,%rax,4),%xmm4
   47646:	movq   0x8(%r14,%rax,4),%xmm5
   4764d:	pshufd $0xd4,%xmm4,%xmm4
   47652:	pcmpeqd %xmm2,%xmm4
   47656:	pandn  %xmm3,%xmm4
   4765a:	paddq  %xmm1,%xmm4
   4765e:	pshufd $0xd4,%xmm5,%xmm5
   47663:	pcmpeqd %xmm2,%xmm5
   47667:	pandn  %xmm3,%xmm5
   4766b:	paddq  %xmm0,%xmm5
   4766f:	movq   0x10(%r14,%rax,4),%xmm0
   47676:	movq   0x18(%r14,%rax,4),%xmm6
   4767d:	pshufd $0xd4,%xmm0,%xmm1
   47682:	pcmpeqd %xmm2,%xmm1
   47686:	pandn  %xmm3,%xmm1
   4768a:	paddq  %xmm4,%xmm1
   4768e:	pshufd $0xd4,%xmm6,%xmm0
   47693:	pcmpeqd %xmm2,%xmm0
   47697:	pandn  %xmm3,%xmm0
   4769b:	paddq  %xmm5,%xmm0
   4769f:	add    $0x8,%rax
   476a3:	cmp    $0x100,%rax
   476a9:	jne    47640 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x120>
   476ab:	mov    $0x88888889,%r13d
   476b1:	imul   %rdx,%r13
   476b5:	shr    $0x24,%r13
   476b9:	mov    %rdx,0x8(%rsp)
   476be:	imul   $0xeeeeeeef,%edx,%eax
   476c4:	ror    $1,%eax
   476c6:	cmp    $0x8888889,%eax
   476cb:	sbb    $0xffffffffffffffff,%r13
   476cf:	mov    %r13,%rbx
   476d2:	cmpl   $0x0,0x14(%rsp)
   476d7:	jne    47730 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x210>
   476d9:	mov    0x8(%rsp),%rsi
   476de:	test   %rsi,%rsi
   476e1:	je     4772e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x20e>
   476e3:	paddq  %xmm1,%xmm0
   476e7:	pshufd $0xee,%xmm0,%xmm1
   476ec:	paddq  %xmm0,%xmm1
   476f0:	movq   %xmm1,%rax
   476f5:	lea    -0x1(%rsi),%ecx
   476f8:	mov    $0x88888889,%edx
   476fd:	imul   %rcx,%rdx
   47701:	shr    $0x23,%rdx
   47705:	and    $0xfffffffe,%edx
   47708:	lea    (%rdx,%rsi,1),%ecx
   4770b:	dec    %ecx
   4770d:	shr    $1,%ecx
   4770f:	mov    $0x80010003,%ebx
   47714:	imul   %rcx,%rbx
   47718:	shr    $0x2e,%rbx
   4771c:	inc    %ebx
   4771e:	imul   %rax,%rbx
   47722:	dec    %rbx
   47725:	cmp    %r13,%rbx
   47728:	cmovae %r13,%rbx
   4772c:	jmp    47730 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x210>
   4772e:	xor    %ebx,%ebx
   47730:	mov    (%r9),%rax
   47733:	lea    0x1(%r13),%rcx
   47737:	mov    %rcx,0x90(%rsp)
   4773f:	cmp    %r13,%rax
   47742:	jbe    48b26 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1606>
   47748:	mov    0x5530(%rsp),%rax
   47750:	mov    (%rax),%rax
   47753:	cmp    %rbx,%rax
   47756:	jb     48b7b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x165b>
   4775c:	movq   $0x0,0x10(%rdi)
   47764:	lea    0x3f(%r13),%rbx
   47768:	shr    $0x6,%rbx
   4776c:	mov    %rdi,0x70(%rsp)
   47771:	mov    %r13,0x78(%rsp)
   47776:	je     477cb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2ab>
   47778:	cmp    (%rdi),%rbx
   4777b:	ja     48c88 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1768>
   47781:	mov    %r9,%r13
   47784:	xor    %r15d,%r15d
   47787:	mov    0x8(%rdi),%r12
   4778b:	lea    (%r12,%r15,8),%rdi
   4778f:	cmp    $0x1,%rbx
   47793:	je     477b7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x297>
   47795:	lea    -0x8(,%rbx,8),%rdx
   4779d:	xor    %esi,%esi
   4779f:	call   *0x105e0b(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
   477a5:	lea    (%r15,%rbx,1),%rax
   477a9:	lea    (%r12,%rax,8),%rdi
   477ad:	add    $0xfffffffffffffff8,%rdi
   477b1:	add    %rbx,%r15
   477b4:	dec    %r15
   477b7:	mov    %r13,%r9
   477ba:	movq   $0x0,(%rdi)
   477c1:	inc    %r15
   477c4:	mov    0x70(%rsp),%rdi
   477c9:	jmp    477ce <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2ae>
   477cb:	xor    %r15d,%r15d
   477ce:	lea    0x18(%rdi),%r13
   477d2:	mov    %r15,0x10(%rdi)
   477d6:	movq   $0x0,0x28(%rdi)
   477de:	cmpq   $0xf,0x18(%rdi)
   477e3:	jbe    48c5d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x173d>
   477e9:	lea    0x30(%rdi),%r15
   477ed:	xor    %eax,%eax
   477ef:	pxor   %xmm0,%xmm0
   477f3:	movabs $0xffffffff00000000,%rcx
   477fd:	nopl   (%rax)
   47800:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   47809:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   47812:	mov    %rcx,0x2d18(%rsp,%rax,1)
   4781a:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   47823:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   4782c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   47834:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   4783d:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   47846:	mov    %rcx,0x2d68(%rsp,%rax,1)
   4784e:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   47857:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   47860:	mov    %rcx,0x2d90(%rsp,%rax,1)
   47868:	add    $0xa0,%rax
   4786e:	cmp    $0x2800,%rax
   47874:	jne    47800 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2e0>
   47876:	lea    0xc(%r14),%rax
   4787a:	xor    %edx,%edx
   4787c:	mov    $0x25,%ecx
   47881:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47890:	mov    -0xc(%rax),%esi
   47893:	add    %edx,%esi
   47895:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   4789c:	mov    %esi,0x2c80(%rsp,%rcx,4)
   478a3:	mov    -0x8(%rax),%edx
   478a6:	add    %esi,%edx
   478a8:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   478af:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   478b6:	mov    -0x4(%rax),%esi
   478b9:	add    %edx,%esi
   478bb:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   478c2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   478c9:	mov    (%rax),%edx
   478cb:	add    %esi,%edx
   478cd:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   478d4:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   478db:	add    $0x28,%rcx
   478df:	add    $0x10,%rax
   478e3:	cmp    $0xa25,%rcx
   478ea:	jne    47890 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x370>
   478ec:	mov    %r9,0x88(%rsp)
   478f4:	mov    0x8(%r9),%rbx
   478f8:	mov    0x8(%rdi),%rbp
   478fc:	mov    0x10(%rdi),%r12
   47900:	lea    0xe8(%rsp),%rdi
   47908:	lea    0x2cf8(%rsp),%rsi
   47910:	mov    $0x2800,%edx
   47915:	call   *0x105cb5(%rip)        # 14d5d0 <memcpy@GLIBC_2.14>
   4791b:	mov    %rbp,%r8
   4791e:	mov    %r12,%rsi
   47921:	mov    %rbx,0xd0(%rsp)
   47929:	mov    0x78(%rsp),%rax
   4792e:	mov    %rax,0xd8(%rsp)
   47936:	mov    0x8(%rsp),%rax
   4793b:	mov    %rax,0xe0(%rsp)
   47943:	mov    0x5530(%rsp),%rax
   4794b:	mov    %rax,0xa8(%rsp)
   47953:	mov    %rbp,0xb0(%rsp)
   4795b:	mov    %r12,0xb8(%rsp)
   47963:	mov    %r13,0xc0(%rsp)
   4796b:	movl   $0x0,0x28f0(%rsp)
   47976:	movq   $0x0,0x28e8(%rsp)
   47982:	mov    %r15,0xc8(%rsp)
   4798a:	mov    $0xffffffffffffffff,%rcx
   47991:	xor    %r13d,%r13d
   47994:	movabs $0xeeeeeeeeeeeeeeef,%r9
   4799e:	movabs $0x888888888888889,%r10
   479a8:	pxor   %xmm0,%xmm0
   479ac:	pcmpeqd %xmm1,%xmm1
   479b0:	xor    %r15d,%r15d
   479b3:	jmp    47a5f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x53f>
   479b8:	nopl   0x0(%rax,%rax,1)
   479c0:	mov    0x8(%rdi),%rax
   479c4:	mov    %rbp,%rcx
   479c7:	shl    $0x7,%rcx
   479cb:	movdqa %xmm0,0x70(%rax,%rcx,1)
   479d1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   479d7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   479dd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   479e3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   479e9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   479ef:	movdqa %xmm0,0x10(%rax,%rcx,1)
   479f5:	movdqa %xmm0,(%rax,%rcx,1)
   479fa:	mov    %rbp,%rax
   479fd:	inc    %rax
   47a00:	mov    %rax,0x10(%rdi)
   47a04:	mov    %r12,%rax
   47a07:	shl    $0x6,%rax
   47a0b:	or     $0x10000000,%ebp
   47a11:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   47a17:	movdqa %xmm1,0x20(%rbx,%rax,1)
   47a1d:	movdqa %xmm1,0x10(%rbx,%rax,1)
   47a23:	movdqa %xmm1,(%rbx,%rax,1)
   47a28:	mov    %ebp,0x3c(%rbx,%rax,1)
   47a2c:	mov    %r12,%rdi
   47a2f:	shr    $0x6,%rdi
   47a33:	cmp    %rsi,%rdi
   47a36:	jae    48cef <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17cf>
   47a3c:	mov    $0x1,%eax
   47a41:	mov    %r12d,%ecx
   47a44:	shl    %cl,%rax
   47a47:	or     %rax,(%r8,%rdi,8)
   47a4b:	mov    %r12,%rcx
   47a4e:	add    $0x4,%r15
   47a52:	cmp    $0x400,%r15
   47a59:	je     47afd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x5dd>
   47a5f:	mov    %r13,%rax
   47a62:	mov    (%r14,%r15,1),%r13d
   47a66:	add    %rax,%r13
   47a69:	cmp    0x8(%rsp),%r13
   47a6e:	setae  %dil
   47a72:	mov    %r13,%rax
   47a75:	movabs $0x8888888888888889,%rdx
   47a7f:	mul    %rdx
   47a82:	mov    %rdx,%r12
   47a85:	shr    $0x4,%r12
   47a89:	mov    %r13,%rax
   47a8c:	imul   %r9,%rax
   47a90:	ror    $1,%rax
   47a93:	cmp    %r10,%rax
   47a96:	setb   %al
   47a99:	cmp    %rcx,%r12
   47a9c:	sete   %dl
   47a9f:	or     %dil,%dl
   47aa2:	or     %al,%dl
   47aa4:	jne    47a4e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x52e>
   47aa6:	mov    0x5530(%rsp),%rdi
   47aae:	mov    0x10(%rdi),%rbp
   47ab2:	cmp    (%rdi),%rbp
   47ab5:	jne    479c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4a0>
   47abb:	mov    %rsi,0x18(%rsp)
   47ac0:	mov    %r8,0x28(%rsp)
   47ac5:	call   9c130 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47aca:	pcmpeqd %xmm1,%xmm1
   47ace:	pxor   %xmm0,%xmm0
   47ad2:	movabs $0x888888888888889,%r10
   47adc:	movabs $0xeeeeeeeeeeeeeeef,%r9
   47ae6:	mov    0x28(%rsp),%r8
   47aeb:	mov    0x18(%rsp),%rsi
   47af0:	mov    0x5530(%rsp),%rdi
   47af8:	jmp    479c0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4a0>
   47afd:	cmpl   $0x0,0x14(%rsp)
   47b02:	mov    0x8(%rsp),%r11
   47b07:	je     47cbe <<c30walk::ctor2su::Ctor2su>::build::<false>+0x79e>
   47b0d:	mov    0xd8(%rsp),%r8
   47b15:	test   %r8,%r8
   47b18:	je     47cbe <<c30walk::ctor2su::Ctor2su>::build::<false>+0x79e>
   47b1e:	mov    %r11,%r9
   47b21:	shl    $0x20,%r9
   47b25:	mov    0x14(%rsp),%r10d
   47b2a:	mov    0xa8(%rsp),%rdi
   47b32:	mov    0xd0(%rsp),%rbp
   47b3a:	xor    %r15d,%r15d
   47b3d:	movabs $0xff51afd7ed558ccd,%rbx
   47b47:	pxor   %xmm0,%xmm0
   47b4b:	pcmpeqd %xmm1,%xmm1
   47b4f:	jmp    47be0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6c0>
   47b54:	mov    0x8(%rdi),%rax
   47b58:	mov    %rbx,%rcx
   47b5b:	shl    $0x7,%rcx
   47b5f:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47b65:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47b6b:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47b71:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47b77:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47b7d:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47b83:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47b89:	movdqa %xmm0,(%rax,%rcx,1)
   47b8e:	mov    %rbx,%rax
   47b91:	inc    %rax
   47b94:	mov    %rax,0x10(%rdi)
   47b98:	or     $0x10000000,%ebx
   47b9e:	movdqu %xmm1,0x2c(%rbp)
   47ba3:	movdqa %xmm1,0x20(%rbp)
   47ba8:	movdqa %xmm1,0x10(%rbp)
   47bad:	movdqa %xmm1,0x0(%rbp)
   47bb2:	mov    %ebx,0x3c(%rbp)
   47bb5:	or     %r14,(%r12,%r13,8)
   47bb9:	mov    0x8(%rsp),%r11
   47bbe:	movabs $0xff51afd7ed558ccd,%rbx
   47bc8:	nopl   0x0(%rax,%rax,1)
   47bd0:	inc    %r15
   47bd3:	add    $0x40,%rbp
   47bd7:	cmp    %r15,%r8
   47bda:	je     47cbe <<c30walk::ctor2su::Ctor2su>::build::<false>+0x79e>
   47be0:	mov    %r15,%r13
   47be3:	shr    $0x6,%r13
   47be7:	mov    0xb8(%rsp),%rsi
   47bef:	cmp    %rsi,%r13
   47bf2:	jae    48cfb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17db>
   47bf8:	mov    0xb0(%rsp),%r12
   47c00:	mov    (%r12,%r13,8),%rax
   47c04:	mov    $0x1,%r14d
   47c0a:	mov    %r15d,%ecx
   47c0d:	shl    %cl,%r14
   47c10:	bt     %r15,%rax
   47c14:	jb     47bd0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6b0>
   47c16:	mov    %r15,%rax
   47c19:	xor    %r9,%rax
   47c1c:	mov    %rax,%rcx
   47c1f:	shr    $0x21,%rcx
   47c23:	xor    %rax,%rcx
   47c26:	imul   %rbx,%rcx
   47c2a:	mov    %rcx,%rax
   47c2d:	shr    $0x21,%rax
   47c31:	xor    %rcx,%rax
   47c34:	movabs $0xc4ceb9fe1a85ec53,%rcx
   47c3e:	imul   %rcx,%rax
   47c42:	mov    %rax,%rcx
   47c45:	shr    $0x21,%rcx
   47c49:	xor    %rax,%rcx
   47c4c:	mov    %rcx,%rax
   47c4f:	shr    $0x3,%rax
   47c53:	movabs $0x20c49ba5e353f7cf,%rdx
   47c5d:	mul    %rdx
   47c60:	shr    $0x4,%rdx
   47c64:	imul   $0x3e8,%rdx,%rax
   47c6b:	sub    %rax,%rcx
   47c6e:	cmp    %r10,%rcx
   47c71:	jae    47bd0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6b0>
   47c77:	mov    0x10(%rdi),%rbx
   47c7b:	cmp    (%rdi),%rbx
   47c7e:	jne    47b54 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x634>
   47c84:	mov    %r8,0x18(%rsp)
   47c89:	mov    %r9,0x28(%rsp)
   47c8e:	mov    %r10,0x68(%rsp)
   47c93:	mov    %rdi,0x60(%rsp)
   47c98:	call   9c130 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47c9d:	pcmpeqd %xmm1,%xmm1
   47ca1:	pxor   %xmm0,%xmm0
   47ca5:	mov    0x60(%rsp),%rdi
   47caa:	mov    0x68(%rsp),%r10
   47caf:	mov    0x28(%rsp),%r9
   47cb4:	mov    0x18(%rsp),%r8
   47cb9:	jmp    47b54 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x634>
   47cbe:	test   %r11,%r11
   47cc1:	je     4895c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x143c>
   47cc7:	movq   $0x0,0x20(%rsp)
   47cd0:	jmp    47d14 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7f4>
   47cd2:	mov    0x30(%rsp),%r15
   47cd7:	lea    0x1(%rax),%rdx
   47cdb:	add    $0xffffffffffffffe3,%rax
   47cdf:	xor    %esi,%esi
   47ce1:	cmp    $0x1e,%rdx
   47ce5:	setae  %sil
   47ce9:	cmovb  %rdx,%rax
   47ced:	lea    (%rcx,%rsi,2),%ecx
   47cf0:	inc    %ecx
   47cf2:	mov    %rax,0x28e8(%rsp)
   47cfa:	mov    %ecx,0x28f0(%rsp)
   47d01:	mov    0x98(%rsp),%rax
   47d09:	cmp    0x8(%rsp),%rax
   47d0e:	jae    4895c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x143c>
   47d14:	mov    0x20(%rsp),%rbx
   47d19:	mov    $0x400,%edx
   47d1e:	lea    0x28f8(%rsp),%rdi
   47d26:	xor    %esi,%esi
   47d28:	call   *0x105882(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
   47d2e:	lea    0x128(%rsp),%rax
   47d36:	xor    %ecx,%ecx
   47d38:	nopl   0x0(%rax,%rax,1)
   47d40:	mov    -0x40(%rax),%rdx
   47d44:	sub    -0x30(%rax),%rdx
   47d48:	shr    $1,%rdx
   47d4b:	add    -0x28(%rax),%edx
   47d4e:	mov    -0x18(%rax),%rsi
   47d52:	sub    -0x8(%rax),%rsi
   47d56:	mov    %edx,0x28f8(%rsp,%rcx,4)
   47d5d:	shr    $1,%rsi
   47d60:	add    (%rax),%esi
   47d62:	mov    %esi,0x28fc(%rsp,%rcx,4)
   47d69:	add    $0x2,%rcx
   47d6d:	add    $0x50,%rax
   47d71:	cmp    $0x100,%rcx
   47d78:	jne    47d40 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x820>
   47d7a:	mov    0xc0(%rsp),%r12
   47d82:	mov    0x10(%r12),%r14
   47d87:	cmp    (%r12),%r14
   47d8b:	je     4894f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x142f>
   47d91:	mov    %r14,%rdi
   47d94:	shl    $0xa,%rdi
   47d98:	add    0x8(%r12),%rdi
   47d9d:	mov    $0x400,%edx
   47da2:	lea    0x28f8(%rsp),%rsi
   47daa:	call   *0x105820(%rip)        # 14d5d0 <memcpy@GLIBC_2.14>
   47db0:	inc    %r14
   47db3:	lea    0xf000(%rbx),%rdx
   47dba:	mov    0x8(%rsp),%rcx
   47dbf:	cmp    %rcx,%rdx
   47dc2:	mov    %rdx,0x98(%rsp)
   47dca:	cmovb  %rdx,%rcx
   47dce:	mov    %r14,0x10(%r12)
   47dd3:	mov    %rcx,0x20(%rsp)
   47dd8:	mov    %rcx,%r12
   47ddb:	sub    %rbx,%r12
   47dde:	cmp    $0x10,%r12
   47de2:	jb     48780 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1260>
   47de8:	add    0x50(%rsp),%rbx
   47ded:	xor    %edx,%edx
   47def:	lea    0xe8(%rsp),%r8
   47df7:	jmp    47e4d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x92d>
   47df9:	nopl   0x0(%rax)
   47e00:	mov    0x28e8(%rsp),%rax
   47e08:	lea    0x10(%rax),%rcx
   47e0c:	add    $0xfffffffffffffff2,%rax
   47e10:	xor    %edx,%edx
   47e12:	cmp    $0x1e,%rcx
   47e16:	setae  %dl
   47e19:	cmovb  %rcx,%rax
   47e1d:	mov    0x28f0(%rsp),%ecx
   47e24:	lea    (%rcx,%rdx,2),%ecx
   47e27:	add    $0x10,%ecx
   47e2a:	mov    %rax,0x28e8(%rsp)
   47e32:	mov    %ecx,0x28f0(%rsp)
   47e39:	xor    %edx,%edx
   47e3b:	mov    0x40(%rsp),%rbx
   47e40:	mov    %rdi,%r12
   47e43:	cmp    $0xf,%rdi
   47e47:	jbe    4878b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x126b>
   47e4d:	mov    0x58(%rsp),%rcx
   47e52:	test   $0x1,%dl
   47e55:	jne    47e5e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93e>
   47e57:	mov    (%rbx),%rcx
   47e5a:	mov    0x8(%rbx),%r15
   47e5e:	lea    0x10(%rbx),%rsi
   47e62:	lea    -0x10(%r12),%rdi
   47e67:	xor    %rcx,%r15
   47e6a:	mov    %rcx,%rax
   47e6d:	rol    $0x8,%rax
   47e71:	xor    %rcx,%rax
   47e74:	or     %r15,%rax
   47e77:	jne    47ed0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x9b0>
   47e79:	cmp    $0x10,%rdi
   47e7d:	jb     48010 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaf0>
   47e83:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47e90:	mov    (%rsi),%rdx
   47e93:	mov    0x8(%rsi),%rax
   47e97:	mov    $0x1,%r9b
   47e9a:	cmp    %rcx,%rdx
   47e9d:	jne    4801a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xafa>
   47ea3:	cmp    %rcx,%rax
   47ea6:	jne    4801a <<c30walk::ctor2su::Ctor2su>::build::<false>+0xafa>
   47eac:	add    $0xfffffffffffffff0,%rdi
   47eb0:	add    $0x10,%rsi
   47eb4:	cmp    $0xf,%rdi
   47eb8:	ja     47e90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x970>
   47eba:	mov    %rsi,0x40(%rsp)
   47ebf:	xor    %r9d,%r9d
   47ec2:	mov    %rcx,0x58(%rsp)
   47ec7:	jmp    48029 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb09>
   47ecc:	nopl   0x0(%rax)
   47ed0:	mov    %rsi,0x40(%rsp)
   47ed5:	mov    0x20(%rsp),%r14
   47eda:	sub    %r12,%r14
   47edd:	mov    $0x1e,%r9d
   47ee3:	sub    0x28e8(%rsp),%r9
   47eeb:	cmp    $0x10,%r9
   47eef:	mov    $0x10,%r15d
   47ef5:	cmovb  %r9,%r15
   47ef9:	movzwl 0x28f0(%rsp),%edx
   47f01:	add    0x50(%rsp),%r14
   47f06:	cmp    $0x1,%r15
   47f0a:	mov    %r15,%r13
   47f0d:	adc    $0xffffffffffffffff,%r13
   47f11:	mov    %edx,%ebp
   47f13:	xor    %r12d,%r12d
   47f16:	cs nopw 0x0(%rax,%rax,1)
   47f20:	movzbl (%rbx),%esi
   47f23:	lea    (%rsi,%rsi,4),%rcx
   47f27:	lea    (%r8,%rcx,8),%rbx
   47f2b:	mov    (%r8,%rcx,8),%rax
   47f2f:	cmp    0x8(%r8,%rcx,8),%rax
   47f34:	je     47f51 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa31>
   47f36:	mov    %bp,(%rax)
   47f39:	addq   $0x2,(%rbx)
   47f3d:	sub    $0x1,%r13
   47f41:	jb     47fa0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa80>
   47f43:	lea    (%r14,%r12,1),%rbx
   47f47:	inc    %rbx
   47f4a:	inc    %r12
   47f4d:	inc    %ebp
   47f4f:	jmp    47f20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa00>
   47f51:	mov    %r9,0x28(%rsp)
   47f56:	mov    %edx,0x18(%rsp)
   47f5a:	mov    %rdi,0x38(%rsp)
   47f5f:	lea    0xa8(%rsp),%rdi
   47f67:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   47f6c:	lea    0xe8(%rsp),%r8
   47f74:	test   %al,%al
   47f76:	je     489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   47f7c:	mov    (%rbx),%rax
   47f7f:	mov    0x38(%rsp),%rdi
   47f84:	mov    0x18(%rsp),%edx
   47f88:	mov    0x28(%rsp),%r9
   47f8d:	mov    %bp,(%rax)
   47f90:	addq   $0x2,(%rbx)
   47f94:	sub    $0x1,%r13
   47f98:	jae    47f43 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa23>
   47f9a:	nopw   0x0(%rax,%rax,1)
   47fa0:	cmp    $0xf,%r9
   47fa4:	ja     47e00 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8e0>
   47faa:	add    $0x2,%edx
   47fad:	movzwl %dx,%ebx
   47fb0:	movzbl (%r14,%r15,1),%esi
   47fb5:	lea    (%rsi,%rsi,4),%rcx
   47fb9:	lea    (%r8,%rcx,8),%r12
   47fbd:	mov    (%r8,%rcx,8),%rax
   47fc1:	cmp    0x8(%r8,%rcx,8),%rax
   47fc6:	je     47fe2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xac2>
   47fc8:	lea    (%rbx,%r15,1),%rcx
   47fcc:	mov    %cx,(%rax)
   47fcf:	addq   $0x2,(%r12)
   47fd4:	inc    %r15
   47fd7:	cmp    $0x10,%r15
   47fdb:	jne    47fb0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa90>
   47fdd:	jmp    47e00 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8e0>
   47fe2:	mov    %rdi,%r13
   47fe5:	lea    0xa8(%rsp),%rdi
   47fed:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   47ff2:	lea    0xe8(%rsp),%r8
   47ffa:	test   %al,%al
   47ffc:	je     489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   48002:	mov    (%r12),%rax
   48006:	mov    %r13,%rdi
   48009:	jmp    47fc8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaa8>
   4800b:	nopl   0x0(%rax,%rax,1)
   48010:	mov    %rsi,0x40(%rsp)
   48015:	xor    %r9d,%r9d
   48018:	jmp    48029 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb09>
   4801a:	mov    %rsi,0x40(%rsp)
   4801f:	mov    %rdx,0x58(%rsp)
   48024:	mov    %rax,0x30(%rsp)
   48029:	mov    %rdi,0x38(%rsp)
   4802e:	sub    %rdi,%r12
   48031:	movzbl %cl,%eax
   48034:	mov    %rax,0xa0(%rsp)
   4803c:	lea    (%rax,%rax,4),%rdx
   48040:	mov    (%r8,%rdx,8),%rax
   48044:	mov    0x1c(%r8,%rdx,8),%esi
   48049:	mov    %rax,%rdi
   4804c:	sub    0x10(%r8,%rdx,8),%rdi
   48051:	shr    $1,%rdi
   48054:	add    0x18(%r8,%rdx,8),%edi
   48059:	add    %r12,%rdi
   4805c:	cmp    %rsi,%rdi
   4805f:	ja     489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   48065:	mov    %r9d,0x4c(%rsp)
   4806a:	lea    (%r8,%rdx,8),%rbx
   4806e:	movzbl %cl,%r15d
   48072:	mov    0x8(%rbx),%rsi
   48076:	mov    $0x88888889,%r14d
   4807c:	lea    -0x2a743(%rip),%r13        # 1d940 <c30walk::ctor2su::PATTERNS>
   48083:	mov    %rbx,0x80(%rsp)
   4808b:	jmp    480c6 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xba6>
   4808d:	nopl   (%rax)
   48090:	mov    0x3c(%rsi),%edi
   48093:	mov    $0xfffff,%r9d
   48099:	and    %r9d,%edi
   4809c:	sub    %r8d,%ecx
   4809f:	sub    %eax,%r8d
   480a2:	mov    %r8d,%eax
   480a5:	lea    (%rsi,%rax,2),%rax
   480a9:	mov    %rax,0x10(%rbx)
   480ad:	mov    %rax,(%rbx)
   480b0:	lea    (%rax,%rcx,2),%rsi
   480b4:	mov    %rsi,0x8(%rbx)
   480b8:	mov    %edx,0x20(%rbx)
   480bb:	mov    %edi,0x24(%rbx)
   480be:	lea    0xe8(%rsp),%r8
   480c6:	sub    %rax,%rsi
   480c9:	shr    $1,%rsi
   480cc:	cmp    %rsi,%r12
   480cf:	cmovb  %r12,%rsi
   480d3:	test   %rsi,%rsi
   480d6:	je     48190 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc70>
   480dc:	mov    0x28e8(%rsp),%rdi
   480e4:	cmp    $0x1d,%rdi
   480e8:	ja     48ccd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17ad>
   480ee:	cmp    $0x1f,%rsi
   480f2:	jae    489e1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14c1>
   480f8:	mov    0x28f0(%rsp),%ecx
   480ff:	mov    %rdi,%r11
   48102:	shl    $0x6,%r11
   48106:	lea    (%r11,%r13,1),%rdx
   4810a:	lea    -0x2(,%rsi,2),%r9
   48112:	cmp    $0xe,%r9
   48116:	jb     4812b <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc0b>
   48118:	mov    %rax,%r8
   4811b:	sub    %r13,%r8
   4811e:	sub    %r11,%r8
   48121:	cmp    $0x10,%r8
   48125:	jae    48578 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1058>
   4812b:	mov    %rdx,%r8
   4812e:	xor    %r10d,%r10d
   48131:	lea    (%rdx,%rsi,2),%rdx
   48135:	lea    (%rax,%r10,2),%r9
   48139:	nopl   0x0(%rax)
   48140:	movzwl (%r8),%r10d
   48144:	add    %cx,%r10w
   48148:	add    $0x2,%r8
   4814c:	mov    %r10w,(%r9)
   48150:	add    $0x2,%r9
   48154:	cmp    %rdx,%r8
   48157:	jne    48140 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc20>
   48159:	lea    (%rax,%rsi,2),%rax
   4815d:	mov    %rax,(%rbx)
   48160:	add    %rsi,%rdi
   48163:	lea    0x2(%rsi),%edx
   48166:	mov    %rdi,%r8
   48169:	sub    $0x1e,%r8
   4816d:	cmovb  %rdi,%r8
   48171:	cmovb  %esi,%edx
   48174:	mov    %r8,0x28e8(%rsp)
   4817c:	add    %ecx,%edx
   4817e:	mov    %edx,0x28f0(%rsp)
   48185:	sub    %rsi,%r12
   48188:	lea    0xe8(%rsp),%r8
   48190:	test   %r12,%r12
   48193:	je     4875e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x123e>
   48199:	mov    %rax,%r8
   4819c:	cmp    0x10(%rbx),%rax
   481a0:	je     481be <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc9e>
   481a2:	lea    0xa8(%rsp),%rdi
   481aa:	mov    0xa0(%rsp),%rsi
   481b2:	call   96880 <<c30walk::ctor2su::W<false>>::finish>
   481b7:	mov    (%rbx),%r8
   481ba:	mov    0x10(%rbx),%rax
   481be:	sub    %rax,%r8
   481c1:	shr    $1,%r8
   481c4:	add    0x18(%rbx),%r8d
   481c8:	mov    %r8d,0x18(%rbx)
   481cc:	pxor   %xmm0,%xmm0
   481d0:	movdqu %xmm0,(%rbx)
   481d4:	movq   $0x0,0x10(%rbx)
   481dc:	cmp    $0x1e,%r12
   481e0:	jb     486f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11d0>
   481e6:	mov    %r8,%rcx
   481e9:	imul   %r14,%rcx
   481ed:	shr    $0x24,%rcx
   481f1:	mov    %ecx,%eax
   481f3:	shl    $0x5,%eax
   481f6:	lea    (%rcx,%rcx,1),%edx
   481f9:	sub    %eax,%edx
   481fb:	add    %r8d,%edx
   481fe:	jne    486f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11d0>
   48204:	mov    %r12,%rax
   48207:	movabs $0x8888888888888889,%rdx
   48211:	mul    %rdx
   48214:	shr    $0x4,%rdx
   48218:	mov    %ecx,%r9d
   4821b:	cmpl   $0x0,0x14(%rsp)
   48220:	mov    %r12,0x68(%rsp)
   48225:	mov    %rdx,0x60(%rsp)
   4822a:	je     48530 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1010>
   48230:	mov    0x28f0(%rsp),%r10d
   48238:	mov    0xb8(%rsp),%rax
   48240:	mov    %rax,0x18(%rsp)
   48245:	mov    0xa8(%rsp),%r11
   4824d:	mov    0xb0(%rsp),%rax
   48255:	mov    %rax,0x28(%rsp)
   4825a:	mov    0xd0(%rsp),%rax
   48262:	mov    0x28e8(%rsp),%r12
   4826a:	mov    %r12,%rdi
   4826d:	shl    $0x6,%rdi
   48271:	add    %r13,%rdi
   48274:	mov    %r9,%rcx
   48277:	shl    $0x6,%rcx
   4827b:	lea    (%rax,%rcx,1),%rbx
   4827f:	add    $0x3c,%rbx
   48283:	mov    %r10d,%r13d
   48286:	shl    $0x8,%r13d
   4828a:	or     %r15d,%r13d
   4828d:	xor    %r14d,%r14d
   48290:	jmp    484a4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf84>
   48295:	data16 cs nopw 0x0(%rax,%rax,1)
   482a0:	mov    0x8(%r11),%rcx
   482a4:	shl    $0x7,%eax
   482a7:	mov    %r13d,(%rcx,%rax,1)
   482ab:	movzwl 0x2(%rdi),%esi
   482af:	lea    (%r10,%r14,1),%ebp
   482b3:	add    %ebp,%esi
   482b5:	shl    $0x8,%esi
   482b8:	or     %r15d,%esi
   482bb:	mov    %esi,0x4(%rcx,%rax,1)
   482bf:	movzwl 0x4(%rdi),%esi
   482c3:	add    %ebp,%esi
   482c5:	shl    $0x8,%esi
   482c8:	or     %r15d,%esi
   482cb:	mov    %esi,0x8(%rcx,%rax,1)
   482cf:	movzwl 0x6(%rdi),%esi
   482d3:	add    %ebp,%esi
   482d5:	shl    $0x8,%esi
   482d8:	or     %r15d,%esi
   482db:	mov    %esi,0xc(%rcx,%rax,1)
   482df:	movzwl 0x8(%rdi),%esi
   482e3:	add    %ebp,%esi
   482e5:	shl    $0x8,%esi
   482e8:	or     %r15d,%esi
   482eb:	mov    %esi,0x10(%rcx,%rax,1)
   482ef:	movzwl 0xa(%rdi),%esi
   482f3:	add    %ebp,%esi
   482f5:	shl    $0x8,%esi
   482f8:	or     %r15d,%esi
   482fb:	mov    %esi,0x14(%rcx,%rax,1)
   482ff:	movzwl 0xc(%rdi),%esi
   48303:	add    %ebp,%esi
   48305:	shl    $0x8,%esi
   48308:	or     %r15d,%esi
   4830b:	mov    %esi,0x18(%rcx,%rax,1)
   4830f:	movzwl 0xe(%rdi),%esi
   48313:	add    %ebp,%esi
   48315:	shl    $0x8,%esi
   48318:	or     %r15d,%esi
   4831b:	mov    %esi,0x1c(%rcx,%rax,1)
   4831f:	movzwl 0x10(%rdi),%esi
   48323:	add    %ebp,%esi
   48325:	shl    $0x8,%esi
   48328:	or     %r15d,%esi
   4832b:	mov    %esi,0x20(%rcx,%rax,1)
   4832f:	movzwl 0x12(%rdi),%esi
   48333:	add    %ebp,%esi
   48335:	shl    $0x8,%esi
   48338:	or     %r15d,%esi
   4833b:	mov    %esi,0x24(%rcx,%rax,1)
   4833f:	movzwl 0x14(%rdi),%esi
   48343:	add    %ebp,%esi
   48345:	shl    $0x8,%esi
   48348:	or     %r15d,%esi
   4834b:	mov    %esi,0x28(%rcx,%rax,1)
   4834f:	movzwl 0x16(%rdi),%esi
   48353:	add    %ebp,%esi
   48355:	shl    $0x8,%esi
   48358:	or     %r15d,%esi
   4835b:	mov    %esi,0x2c(%rcx,%rax,1)
   4835f:	movzwl 0x18(%rdi),%esi
   48363:	add    %ebp,%esi
   48365:	shl    $0x8,%esi
   48368:	or     %r15d,%esi
   4836b:	mov    %esi,0x30(%rcx,%rax,1)
   4836f:	movzwl 0x1a(%rdi),%esi
   48373:	add    %ebp,%esi
   48375:	shl    $0x8,%esi
   48378:	or     %r15d,%esi
   4837b:	mov    %esi,0x34(%rcx,%rax,1)
   4837f:	movzwl 0x1c(%rdi),%esi
   48383:	add    %ebp,%esi
   48385:	shl    $0x8,%esi
   48388:	or     %r15d,%esi
   4838b:	mov    %esi,0x38(%rcx,%rax,1)
   4838f:	movzwl 0x1e(%rdi),%esi
   48393:	add    %ebp,%esi
   48395:	shl    $0x8,%esi
   48398:	or     %r15d,%esi
   4839b:	mov    %esi,0x3c(%rcx,%rax,1)
   4839f:	movzwl 0x20(%rdi),%esi
   483a3:	add    %ebp,%esi
   483a5:	shl    $0x8,%esi
   483a8:	or     %r15d,%esi
   483ab:	mov    %esi,0x40(%rcx,%rax,1)
   483af:	movzwl 0x22(%rdi),%esi
   483b3:	add    %ebp,%esi
   483b5:	shl    $0x8,%esi
   483b8:	or     %r15d,%esi
   483bb:	mov    %esi,0x44(%rcx,%rax,1)
   483bf:	movzwl 0x24(%rdi),%esi
   483c3:	add    %ebp,%esi
   483c5:	shl    $0x8,%esi
   483c8:	or     %r15d,%esi
   483cb:	mov    %esi,0x48(%rcx,%rax,1)
   483cf:	movzwl 0x26(%rdi),%esi
   483d3:	add    %ebp,%esi
   483d5:	shl    $0x8,%esi
   483d8:	or     %r15d,%esi
   483db:	mov    %esi,0x4c(%rcx,%rax,1)
   483df:	movzwl 0x28(%rdi),%esi
   483e3:	add    %ebp,%esi
   483e5:	shl    $0x8,%esi
   483e8:	or     %r15d,%esi
   483eb:	mov    %esi,0x50(%rcx,%rax,1)
   483ef:	movzwl 0x2a(%rdi),%esi
   483f3:	add    %ebp,%esi
   483f5:	shl    $0x8,%esi
   483f8:	or     %r15d,%esi
   483fb:	mov    %esi,0x54(%rcx,%rax,1)
   483ff:	movzwl 0x2c(%rdi),%esi
   48403:	add    %ebp,%esi
   48405:	shl    $0x8,%esi
   48408:	or     %r15d,%esi
   4840b:	mov    %esi,0x58(%rcx,%rax,1)
   4840f:	movzwl 0x2e(%rdi),%esi
   48413:	add    %ebp,%esi
   48415:	shl    $0x8,%esi
   48418:	or     %r15d,%esi
   4841b:	mov    %esi,0x5c(%rcx,%rax,1)
   4841f:	movzwl 0x30(%rdi),%esi
   48423:	add    %ebp,%esi
   48425:	shl    $0x8,%esi
   48428:	or     %r15d,%esi
   4842b:	mov    %esi,0x60(%rcx,%rax,1)
   4842f:	movzwl 0x32(%rdi),%esi
   48433:	add    %ebp,%esi
   48435:	shl    $0x8,%esi
   48438:	or     %r15d,%esi
   4843b:	mov    %esi,0x64(%rcx,%rax,1)
   4843f:	movzwl 0x34(%rdi),%esi
   48443:	add    %ebp,%esi
   48445:	shl    $0x8,%esi
   48448:	or     %r15d,%esi
   4844b:	mov    %esi,0x68(%rcx,%rax,1)
   4844f:	movzwl 0x36(%rdi),%esi
   48453:	add    %ebp,%esi
   48455:	shl    $0x8,%esi
   48458:	or     %r15d,%esi
   4845b:	mov    %esi,0x6c(%rcx,%rax,1)
   4845f:	movzwl 0x38(%rdi),%esi
   48463:	add    %ebp,%esi
   48465:	shl    $0x8,%esi
   48468:	or     %r15d,%esi
   4846b:	mov    %esi,0x70(%rcx,%rax,1)
   4846f:	movzwl 0x3a(%rdi),%esi
   48473:	add    %ebp,%esi
   48475:	shl    $0x8,%esi
   48478:	or     %r15d,%esi
   4847b:	mov    %esi,0x74(%rcx,%rax,1)
   4847f:	lea    (%r10,%r14,1),%eax
   48483:	add    $0x20,%eax
   48486:	mov    %eax,0x28f0(%rsp)
   4848d:	add    $0x20,%r14
   48491:	inc    %r9
   48494:	add    $0x2000,%r13d
   4849b:	dec    %rdx
   4849e:	je     486ac <<c30walk::ctor2su::Ctor2su>::build::<false>+0x118c>
   484a4:	mov    %r9,%rax
   484a7:	shr    $0x6,%rax
   484ab:	cmp    0x18(%rsp),%rax
   484b0:	jae    48bdf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16bf>
   484b6:	mov    0x28(%rsp),%rcx
   484bb:	mov    (%rcx,%rax,8),%rax
   484bf:	bt     %r9,%rax
   484c3:	jae    484f0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xfd0>
   484c5:	cmp    $0x1e,%r12
   484c9:	jae    48c05 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16e5>
   484cf:	mov    (%rbx,%r14,2),%eax
   484d3:	and    $0xfffff,%eax
   484d8:	mov    0x10(%r11),%rcx
   484dc:	cmp    %rax,%rcx
   484df:	ja     482a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xd80>
   484e5:	jmp    48bf3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16d3>
   484ea:	nopw   0x0(%rax,%rax,1)
   484f0:	cmp    $0x1e,%r12
   484f4:	jae    48c49 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1729>
   484fa:	movups 0x2c(%rdi),%xmm0
   484fe:	movups %xmm0,-0x10(%rbx,%r14,2)
   48504:	movdqa (%rdi),%xmm0
   48508:	movdqa 0x10(%rdi),%xmm1
   4850d:	movdqa 0x20(%rdi),%xmm2
   48512:	movdqa %xmm2,-0x1c(%rbx,%r14,2)
   48519:	movdqa %xmm1,-0x2c(%rbx,%r14,2)
   48520:	movdqa %xmm0,-0x3c(%rbx,%r14,2)
   48527:	mov    %r13d,(%rbx,%r14,2)
   4852b:	jmp    4847f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf5f>
   48530:	mov    0x28e8(%rsp),%rdi
   48538:	cmp    $0x1d,%rdi
   4853c:	ja     48cde <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17be>
   48542:	shl    $0x6,%rdi
   48546:	add    %r13,%rdi
   48549:	mov    0x28f0(%rsp),%eax
   48550:	mov    0xd0(%rsp),%rcx
   48558:	shl    $0x6,%r9
   4855c:	mov    %eax,%r14d
   4855f:	shl    $0x8,%r14d
   48563:	or     %r15d,%r14d
   48566:	lea    -0x1e(%r12),%rsi
   4856b:	cmp    $0x1e,%rsi
   4856f:	jae    485dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10bd>
   48571:	xor    %esi,%esi
   48573:	jmp    4866a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x114a>
   48578:	shr    $1,%r9
   4857b:	inc    %r9
   4857e:	mov    %r9,%r10
   48581:	and    $0xfffffffffffffff8,%r10
   48585:	lea    (%rdx,%r10,2),%r8
   48589:	movd   %ecx,%xmm0
   4858d:	pshuflw $0x0,%xmm0,%xmm0
   48592:	add    %r13,%r11
   48595:	add    $0x8,%r11
   48599:	xor    %ebx,%ebx
   4859b:	nopl   0x0(%rax,%rax,1)
   485a0:	movq   -0x8(%r11,%rbx,2),%xmm1
   485a7:	movq   (%r11,%rbx,2),%xmm2
   485ad:	paddw  %xmm0,%xmm1
   485b1:	paddw  %xmm0,%xmm2
   485b5:	punpcklqdq %xmm2,%xmm1
   485b9:	movdqu %xmm1,(%rax,%rbx,2)
   485be:	add    $0x8,%rbx
   485c2:	cmp    %rbx,%r10
   485c5:	jne    485a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1080>
   485c7:	cmp    %r10,%r9
   485ca:	mov    0x80(%rsp),%rbx
   485d2:	jne    48131 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc11>
   485d8:	jmp    48159 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc39>
   485dd:	mov    %rdx,%r10
   485e0:	movabs $0xffffffffffffffe,%rsi
   485ea:	and    %rsi,%r10
   485ed:	lea    (%rcx,%r9,1),%r11
   485f1:	add    $0x7c,%r11
   485f5:	xor    %esi,%esi
   485f7:	nopw   0x0(%rax,%rax,1)
   48600:	movdqu 0x2c(%rdi),%xmm0
   48605:	movdqu %xmm0,-0x50(%r11)
   4860b:	movdqa (%rdi),%xmm1
   4860f:	movdqa 0x10(%rdi),%xmm2
   48614:	movdqa 0x20(%rdi),%xmm3
   48619:	movdqa %xmm3,-0x5c(%r11)
   4861f:	movdqa %xmm2,-0x6c(%r11)
   48625:	movdqa %xmm1,-0x7c(%r11)
   4862b:	mov    %r14d,-0x40(%r11)
   4862f:	add    $0x2,%rsi
   48633:	movdqa %xmm1,-0x3c(%r11)
   48639:	movdqa %xmm2,-0x2c(%r11)
   4863f:	movdqa %xmm3,-0x1c(%r11)
   48645:	movdqu %xmm0,-0x10(%r11)
   4864b:	lea    0x2000(%r14),%ebx
   48652:	mov    %ebx,(%r11)
   48655:	add    $0x4000,%r14d
   4865c:	sub    $0xffffffffffffff80,%r11
   48660:	cmp    %rsi,%r10
   48663:	jne    48600 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10e0>
   48665:	test   $0x1,%dl
   48668:	je     4869e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x117e>
   4866a:	add    %r9,%rcx
   4866d:	shl    $0x6,%rsi
   48671:	movups 0x2c(%rdi),%xmm0
   48675:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4867a:	movdqa (%rdi),%xmm0
   4867e:	movdqa 0x10(%rdi),%xmm1
   48683:	movdqa 0x20(%rdi),%xmm2
   48688:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   4868e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   48694:	movdqa %xmm0,(%rcx,%rsi,1)
   48699:	mov    %r14d,0x3c(%rcx,%rsi,1)
   4869e:	mov    %edx,%ecx
   486a0:	shl    $0x5,%ecx
   486a3:	add    %ecx,%eax
   486a5:	mov    %eax,0x28f0(%rsp)
   486ac:	mov    0x60(%rsp),%rdx
   486b1:	mov    %rdx,%rax
   486b4:	shl    $0x5,%rax
   486b8:	lea    (%rdx,%rdx,1),%rcx
   486bc:	sub    %rax,%rcx
   486bf:	mov    0x68(%rsp),%r12
   486c4:	add    %rcx,%r12
   486c7:	lea    (%rdx,%rdx,1),%eax
   486ca:	shl    $0x5,%edx
   486cd:	sub    %eax,%edx
   486cf:	add    %edx,%r8d
   486d2:	mov    0x80(%rsp),%rbx
   486da:	mov    %r8d,0x18(%rbx)
   486de:	test   %r12,%r12
   486e1:	mov    $0x88888889,%r14d
   486e7:	lea    -0x2adae(%rip),%r13        # 1d940 <c30walk::ctor2su::PATTERNS>
   486ee:	je     48756 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1236>
   486f0:	mov    0x1c(%rbx),%ecx
   486f3:	mov    %r8d,%edi
   486f6:	imul   %r14,%rdi
   486fa:	mov    %rdi,%rdx
   486fd:	shr    $0x24,%rdx
   48701:	mov    %edx,%eax
   48703:	shl    $0x5,%eax
   48706:	sub    %edx,%eax
   48708:	sub    %edx,%eax
   4870a:	lea    0x1e(%rax),%esi
   4870d:	cmp    %ecx,%esi
   4870f:	cmovb  %esi,%ecx
   48712:	shr    $0x2a,%rdi
   48716:	mov    0xb8(%rsp),%rsi
   4871e:	cmp    %rsi,%rdi
   48721:	jae    48cc1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17a1>
   48727:	mov    %rdx,%rsi
   4872a:	shl    $0x6,%rsi
   4872e:	add    0xd0(%rsp),%rsi
   48736:	mov    0xb0(%rsp),%r9
   4873e:	mov    (%r9,%rdi,8),%rdi
   48742:	bt     %rdx,%rdi
   48746:	jb     48090 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb70>
   4874c:	mov    $0xffffffff,%edi
   48751:	jmp    4809c <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb7c>
   48756:	lea    0xe8(%rsp),%r8
   4875e:	mov    0x38(%rsp),%rdi
   48763:	mov    0x30(%rsp),%r15
   48768:	mov    0x4c(%rsp),%edx
   4876c:	mov    0x40(%rsp),%rbx
   48771:	mov    %rdi,%r12
   48774:	cmp    $0xf,%rdi
   48778:	ja     47e4d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x92d>
   4877e:	jmp    4878b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x126b>
   48780:	mov    %r12,%rdi
   48783:	lea    0xe8(%rsp),%r8
   4878b:	test   %rdi,%rdi
   4878e:	je     47d01 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7e1>
   48794:	mov    0x20(%rsp),%r12
   48799:	sub    %rdi,%r12
   4879c:	mov    0x28e8(%rsp),%rax
   487a4:	mov    $0x1e,%ebx
   487a9:	sub    %rax,%rbx
   487ac:	cmp    %rdi,%rbx
   487af:	cmovae %rdi,%rbx
   487b3:	mov    %rbx,%r13
   487b6:	add    %r12,%r13
   487b9:	jb     48c19 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16f9>
   487bf:	cmp    0x8(%rsp),%r13
   487c4:	ja     48c19 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16f9>
   487ca:	mov    %r15,0x30(%rsp)
   487cf:	mov    0x28f0(%rsp),%r14d
   487d7:	cmp    $0x1e,%rax
   487db:	jne    4886a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x134a>
   487e1:	cmp    %r13,0x20(%rsp)
   487e6:	jb     48c30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1710>
   487ec:	jne    488de <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13be>
   487f2:	mov    0x28e8(%rsp),%rax
   487fa:	mov    0x28f0(%rsp),%ecx
   48801:	cmp    $0x1,%rdi
   48805:	je     47cd2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b2>
   4880b:	mov    %rdi,%r8
   4880e:	mov    %edi,%edx
   48810:	and    $0xe,%edx
   48813:	mov    0x30(%rsp),%r15
   48818:	nopl   0x0(%rax,%rax,1)
   48820:	lea    0x1(%rax),%rsi
   48824:	add    $0xffffffffffffffe3,%rax
   48828:	xor    %edi,%edi
   4882a:	cmp    $0x1e,%rsi
   4882e:	setae  %dil
   48832:	cmovb  %rsi,%rax
   48836:	lea    (%rcx,%rdi,2),%ecx
   48839:	lea    0x1(%rax),%rsi
   4883d:	add    $0xffffffffffffffe3,%rax
   48841:	xor    %edi,%edi
   48843:	cmp    $0x1e,%rsi
   48847:	setae  %dil
   4884b:	cmovb  %rsi,%rax
   4884f:	lea    (%rcx,%rdi,2),%ecx
   48852:	add    $0x2,%ecx
   48855:	add    $0xfffffffffffffffe,%rdx
   48859:	jne    48820 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1300>
   4885b:	test   $0x1,%r8b
   4885f:	jne    47cd7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b7>
   48865:	jmp    47cf2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7d2>
   4886a:	add    0x50(%rsp),%r12
   4886f:	xor    %r15d,%r15d
   48872:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48880:	movzbl (%r12,%r15,1),%esi
   48885:	lea    (%rsi,%rsi,4),%rcx
   48889:	lea    (%r8,%rcx,8),%rbp
   4888d:	mov    (%r8,%rcx,8),%rax
   48891:	cmp    0x8(%r8,%rcx,8),%rax
   48896:	je     488b1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1391>
   48898:	lea    (%r14,%r15,1),%rcx
   4889c:	mov    %cx,(%rax)
   4889f:	addq   $0x2,0x0(%rbp)
   488a4:	inc    %r15
   488a7:	cmp    %r15,%rbx
   488aa:	jne    48880 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1360>
   488ac:	jmp    487e1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12c1>
   488b1:	mov    %rdi,0x38(%rsp)
   488b6:	lea    0xa8(%rsp),%rdi
   488be:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   488c3:	lea    0xe8(%rsp),%r8
   488cb:	test   %al,%al
   488cd:	je     489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   488d3:	mov    0x0(%rbp),%rax
   488d7:	mov    0x38(%rsp),%rdi
   488dc:	jmp    48898 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1378>
   488de:	mov    0x50(%rsp),%rax
   488e3:	mov    0x20(%rsp),%rcx
   488e8:	lea    (%rax,%rcx,1),%r15
   488ec:	lea    (%r14,%rbx,1),%ebp
   488f0:	add    $0x2,%ebp
   488f3:	sub    %rdi,%rbx
   488f6:	cs nopw 0x0(%rax,%rax,1)
   48900:	movzbl (%r15,%rbx,1),%esi
   48905:	lea    (%rsi,%rsi,4),%rcx
   48909:	lea    (%r8,%rcx,8),%r14
   4890d:	mov    (%r8,%rcx,8),%rax
   48911:	cmp    0x8(%r8,%rcx,8),%rax
   48916:	je     4892b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x140b>
   48918:	mov    %bp,(%rax)
   4891b:	addq   $0x2,(%r14)
   4891f:	inc    %ebp
   48921:	inc    %rbx
   48924:	jne    48900 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13e0>
   48926:	jmp    487f2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12d2>
   4892b:	mov    %rdi,%r12
   4892e:	lea    0xa8(%rsp),%rdi
   48936:	call   96e80 <<c30walk::ctor2su::W<false>>::refill>
   4893b:	lea    0xe8(%rsp),%r8
   48943:	test   %al,%al
   48945:	je     489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   48947:	mov    (%r14),%rax
   4894a:	mov    %r12,%rdi
   4894d:	jmp    48918 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13f8>
   4894f:	mov    %r12,%rdi
   48952:	call   fea30 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   48957:	jmp    47d91 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x871>
   4895c:	mov    $0x84,%eax
   48961:	lea    0xe8(%rsp),%r8
   48969:	nopl   0x0(%rax)
   48970:	lea    0x50(%rax),%rcx
   48974:	cmp    $0x28d4,%rcx
   4897b:	je     489f4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14d4>
   4897d:	mov    0x64(%rsp,%rax,1),%rdx
   48982:	sub    0x74(%rsp,%rax,1),%rdx
   48987:	shr    $1,%rdx
   4898a:	add    0x7c(%rsp,%rax,1),%edx
   4898e:	cmp    0x80(%rsp,%rax,1),%edx
   48995:	jne    489bd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x149d>
   48997:	mov    0x8c(%rsp,%rax,1),%rdx
   4899f:	sub    0x9c(%rsp,%rax,1),%rdx
   489a7:	shr    $1,%rdx
   489aa:	add    0xa4(%rsp,%rax,1),%edx
   489b1:	cmp    0xa8(%rsp,%rax,1),%edx
   489b8:	mov    %rcx,%rax
   489bb:	je     48970 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1450>
   489bd:	mov    0x5530(%rsp),%rax
   489c5:	movq   $0x0,0x10(%rax)
   489cd:	xor    %eax,%eax
   489cf:	add    $0x54f8,%rsp
   489d6:	pop    %rbx
   489d7:	pop    %r12
   489d9:	pop    %r13
   489db:	pop    %r14
   489dd:	pop    %r15
   489df:	pop    %rbp
   489e0:	ret
   489e1:	lea    0x1001a0(%rip),%rcx        # 148b88 <__frame_dummy_init_array_entry+0x2338>
   489e8:	mov    $0x1e,%edx
   489ed:	xor    %edi,%edi
   489ef:	call   f0c10 <core::slice::index::slice_index_fail>
   489f4:	xor    %r14d,%r14d
   489f7:	lea    0xa8(%rsp),%r15
   489ff:	mov    0x88(%rsp),%r12
   48a07:	mov    0x78(%rsp),%r13
   48a0c:	jmp    48a3b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x151b>
   48a0e:	xchg   %ax,%ax
   48a10:	sub    %rcx,%rax
   48a13:	inc    %r14
   48a16:	shr    $1,%rax
   48a19:	add    %eax,0x18(%r8)
   48a1d:	pxor   %xmm0,%xmm0
   48a21:	movdqu %xmm0,(%r8)
   48a26:	movq   $0x0,0x10(%r8)
   48a2e:	add    $0x28,%r8
   48a32:	cmp    $0x100,%r14
   48a39:	je     48a61 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1541>
   48a3b:	mov    (%r8),%rcx
   48a3e:	mov    %rcx,%rax
   48a41:	cmp    0x10(%r8),%rcx
   48a45:	je     48a10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14f0>
   48a47:	mov    %r15,%rdi
   48a4a:	mov    %r14,%rsi
   48a4d:	mov    %r8,%rbx
   48a50:	call   96880 <<c30walk::ctor2su::W<false>>::finish>
   48a55:	mov    %rbx,%r8
   48a58:	mov    (%rbx),%rax
   48a5b:	mov    0x10(%rbx),%rcx
   48a5f:	jmp    48a10 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14f0>
   48a61:	xor    %ecx,%ecx
   48a63:	mov    0x8(%r12),%rbx
   48a68:	test   %r13,%r13
   48a6b:	setne  %al
   48a6e:	je     48ab9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1599>
   48a70:	mov    %al,%cl
   48a72:	mov    %r13,%rax
   48a75:	sub    %rcx,%rax
   48a78:	imul   $0xffffffffffffffe2,%rax,%rcx
   48a7c:	add    0x8(%rsp),%rcx
   48a81:	cmp    $0x1e,%rcx
   48a85:	jae    48ab9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1599>
   48a87:	lea    (%rbx,%r13,4),%rcx
   48a8b:	mov    0x8(%rsp),%rdx
   48a90:	lea    (%rcx,%rdx,2),%rdi
   48a94:	add    $0xfffffffffffffffc,%rdi
   48a98:	lea    (%rax,%rax,1),%rcx
   48a9c:	shl    $0x5,%rax
   48aa0:	sub    %rcx,%rax
   48aa3:	sub    %rdx,%rax
   48aa6:	lea    0x3c(,%rax,2),%rdx
   48aae:	mov    $0xff,%esi
   48ab3:	call   *0x104af7(%rip)        # 14d5b0 <memset@GLIBC_2.2.5>
   48ab9:	shl    $0x6,%r13
   48abd:	pcmpeqd %xmm0,%xmm0
   48ac1:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   48ac8:	movdqa %xmm0,0x20(%rbx,%r13,1)
   48acf:	movdqa %xmm0,0x10(%rbx,%r13,1)
   48ad6:	movdqa %xmm0,(%rbx,%r13,1)
   48adc:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   48ae5:	mov    0x70(%rsp),%rax
   48aea:	mov    0x8(%rax),%rdx
   48aee:	mov    0x10(%rax),%rcx
   48af2:	mov    0x5530(%rsp),%rax
   48afa:	mov    0x8(%rax),%r8
   48afe:	mov    0x10(%rax),%r9
   48b02:	lea    0xa8(%rsp),%rdi
   48b0a:	mov    %rbx,%rsi
   48b0d:	call   a5810 <c30walk::ctor::canonicalise>
   48b12:	mov    0x90(%rsp),%rax
   48b1a:	mov    %rax,0x10(%r12)
   48b1f:	mov    $0x1,%al
   48b21:	jmp    489cf <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14af>
   48b26:	mov    %rdi,%r12
   48b29:	test   %rax,%rax
   48b2c:	je     48b3e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x161e>
   48b2e:	mov    0x8(%r9),%rdi
   48b32:	mov    %r9,%r15
   48b35:	call   *0x104a85(%rip)        # 14d5c0 <free@GLIBC_2.2.5>
   48b3b:	mov    %r15,%r9
   48b3e:	movq   $0x0,(%r9)
   48b45:	movq   $0x40,0x8(%r9)
   48b4d:	movq   $0x0,0x10(%r9)
   48b55:	mov    $0x40,%edx
   48b5a:	mov    %r9,%rdi
   48b5d:	lea    0x1(%r13),%rsi
   48b61:	mov    %r9,%r15
   48b64:	call   9b3e0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48b69:	mov    %r15,%r9
   48b6c:	cmp    $0xffffffffffffffff,%rax
   48b70:	mov    %r12,%rdi
   48b73:	je     47748 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x228>
   48b79:	jmp    48bd4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16b4>
   48b7b:	mov    %rdi,%r12
   48b7e:	mov    %r9,%r15
   48b81:	test   %rax,%rax
   48b84:	je     48b98 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1678>
   48b86:	mov    0x5530(%rsp),%rax
   48b8e:	mov    0x8(%rax),%rdi
   48b92:	call   *0x104a28(%rip)        # 14d5c0 <free@GLIBC_2.2.5>
   48b98:	mov    0x5530(%rsp),%rdi
   48ba0:	movq   $0x0,(%rdi)
   48ba7:	movq   $0x40,0x8(%rdi)
   48baf:	movq   $0x0,0x10(%rdi)
   48bb7:	mov    $0x80,%edx
   48bbc:	mov    %rbx,%rsi
   48bbf:	call   9b3e0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48bc4:	cmp    $0xffffffffffffffff,%rax
   48bc8:	mov    %r15,%r9
   48bcb:	mov    %r12,%rdi
   48bce:	je     4775c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x23c>
   48bd4:	mov    %rax,%rdi
   48bd7:	mov    %rdx,%rsi
   48bda:	call   ee39b <alloc::raw_vec::handle_error>
   48bdf:	lea    0xfdef2(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48be6:	mov    %rax,%rdi
   48be9:	mov    0x18(%rsp),%rsi
   48bee:	call   efe33 <core::panicking::panic_bounds_check>
   48bf3:	lea    0x10004e(%rip),%rdx        # 148c48 <__frame_dummy_init_array_entry+0x23f8>
   48bfa:	mov    %rax,%rdi
   48bfd:	mov    %rcx,%rsi
   48c00:	call   efe33 <core::panicking::panic_bounds_check>
   48c05:	lea    0x100024(%rip),%rdx        # 148c30 <__frame_dummy_init_array_entry+0x23e0>
   48c0c:	mov    $0x1e,%esi
   48c11:	mov    %r12,%rdi
   48c14:	call   efe33 <core::panicking::panic_bounds_check>
   48c19:	lea    0xfff98(%rip),%rcx        # 148bb8 <__frame_dummy_init_array_entry+0x2368>
   48c20:	mov    %r12,%rdi
   48c23:	mov    %r13,%rsi
   48c26:	mov    0x8(%rsp),%rdx
   48c2b:	call   f0c10 <core::slice::index::slice_index_fail>
   48c30:	lea    0xfff69(%rip),%rcx        # 148ba0 <__frame_dummy_init_array_entry+0x2350>
   48c37:	mov    %r13,%rdi
   48c3a:	mov    0x20(%rsp),%rsi
   48c3f:	mov    0x8(%rsp),%rdx
   48c44:	call   f0c10 <core::slice::index::slice_index_fail>
   48c49:	lea    0xfffc8(%rip),%rdx        # 148c18 <__frame_dummy_init_array_entry+0x23c8>
   48c50:	mov    $0x1e,%esi
   48c55:	mov    %r12,%rdi
   48c58:	call   efe33 <core::panicking::panic_bounds_check>
   48c5d:	mov    $0x10,%edx
   48c62:	mov    $0x4,%ecx
   48c67:	mov    $0x400,%r8d
   48c6d:	mov    %rdi,%r15
   48c70:	mov    %r13,%rdi
   48c73:	xor    %esi,%esi
   48c75:	mov    %r9,%rbx
   48c78:	call   6eeb0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48c7d:	mov    %r15,%rdi
   48c80:	mov    %rbx,%r9
   48c83:	jmp    477e9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2c9>
   48c88:	mov    %r9,%r13
   48c8b:	mov    $0x8,%ecx
   48c90:	mov    $0x8,%r8d
   48c96:	mov    %rdi,%r15
   48c99:	xor    %esi,%esi
   48c9b:	mov    %rbx,%rdx
   48c9e:	call   6eeb0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   48ca3:	mov    %r15,%rdi
   48ca6:	mov    0x10(%r15),%r15
   48caa:	mov    0x8(%rdi),%r12
   48cae:	lea    (%r12,%r15,8),%rdi
   48cb2:	cmp    $0x1,%rbx
   48cb6:	jne    47795 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x275>
   48cbc:	jmp    477b7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x297>
   48cc1:	lea    0xfde10(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48cc8:	call   efe33 <core::panicking::panic_bounds_check>
   48ccd:	lea    0xffe9c(%rip),%rdx        # 148b70 <__frame_dummy_init_array_entry+0x2320>
   48cd4:	mov    $0x1e,%esi
   48cd9:	call   efe33 <core::panicking::panic_bounds_check>
   48cde:	lea    0xfff1b(%rip),%rdx        # 148c00 <__frame_dummy_init_array_entry+0x23b0>
   48ce5:	mov    $0x1e,%esi
   48cea:	call   efe33 <core::panicking::panic_bounds_check>
   48cef:	lea    0xffe02(%rip),%rdx        # 148af8 <__frame_dummy_init_array_entry+0x22a8>
   48cf6:	call   efe33 <core::panicking::panic_bounds_check>
   48cfb:	lea    0xfddd6(%rip),%rdx        # 146ad8 <__frame_dummy_init_array_entry+0x288>
   48d02:	mov    %r13,%rdi
   48d05:	call   efe33 <core::panicking::panic_bounds_check>
