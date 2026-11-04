# <c30walk::ctor2c::Ctor2c>::build::<false>
# size 0x20bf

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000436f0 <<c30walk::ctor2c::Ctor2c>::build::<false>>:
   436f0:	push   %rbp
   436f1:	push   %r15
   436f3:	push   %r14
   436f5:	push   %r13
   436f7:	push   %r12
   436f9:	push   %rbx
   436fa:	sub    $0x1000,%rsp
   43701:	movq   $0x0,(%rsp)
   43709:	sub    $0x1000,%rsp
   43710:	movq   $0x0,(%rsp)
   43718:	sub    $0x1000,%rsp
   4371f:	movq   $0x0,(%rsp)
   43727:	sub    $0x1000,%rsp
   4372e:	movq   $0x0,(%rsp)
   43736:	sub    $0x4f8,%rsp
   4373d:	mov    %r8d,0x2c(%rsp)
   43742:	mov    %rsi,0x60(%rsp)
   43747:	mov    %rdi,%rbp
   4374a:	mov    0x4530(%rsp),%rdi
   43752:	movq   $0x0,0x10(%r9)
   4375a:	movq   $0x0,0x10(%rdi)
   43762:	cmp    $0xeffe2,%rdx
   43769:	ja     4542d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d3d>
   4376f:	mov    %rcx,%rbx
   43772:	pxor   %xmm0,%xmm0
   43776:	xor    %eax,%eax
   43778:	pxor   %xmm2,%xmm2
   4377c:	pxor   %xmm1,%xmm1
   43780:	movq   (%rbx,%rax,4),%xmm3
   43785:	movq   0x8(%rbx,%rax,4),%xmm4
   4378b:	punpckldq %xmm0,%xmm3
   4378f:	paddq  %xmm2,%xmm3
   43793:	punpckldq %xmm0,%xmm4
   43797:	paddq  %xmm1,%xmm4
   4379b:	movq   0x10(%rbx,%rax,4),%xmm2
   437a1:	movq   0x18(%rbx,%rax,4),%xmm1
   437a7:	punpckldq %xmm0,%xmm2
   437ab:	paddq  %xmm3,%xmm2
   437af:	punpckldq %xmm0,%xmm1
   437b3:	paddq  %xmm4,%xmm1
   437b7:	add    $0x8,%rax
   437bb:	cmp    $0x100,%rax
   437c1:	jne    43780 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x90>
   437c3:	paddq  %xmm2,%xmm1
   437c7:	pshufd $0xee,%xmm1,%xmm0
   437cc:	paddq  %xmm1,%xmm0
   437d0:	movq   %xmm0,%rax
   437d5:	cmp    %rdx,%rax
   437d8:	jne    4542d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d3d>
   437de:	pxor   %xmm2,%xmm2
   437e2:	xor    %eax,%eax
   437e4:	movdqa -0x32dfc(%rip),%xmm3        # 109f0 <GCC_except_table992+0xb2c>
   437ec:	pxor   %xmm1,%xmm1
   437f0:	pxor   %xmm0,%xmm0
   437f4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   43800:	movq   (%rbx,%rax,4),%xmm4
   43805:	movq   0x8(%rbx,%rax,4),%xmm5
   4380b:	pshufd $0xd4,%xmm4,%xmm4
   43810:	pcmpeqd %xmm2,%xmm4
   43814:	pandn  %xmm3,%xmm4
   43818:	paddq  %xmm1,%xmm4
   4381c:	pshufd $0xd4,%xmm5,%xmm5
   43821:	pcmpeqd %xmm2,%xmm5
   43825:	pandn  %xmm3,%xmm5
   43829:	paddq  %xmm0,%xmm5
   4382d:	movq   0x10(%rbx,%rax,4),%xmm0
   43833:	movq   0x18(%rbx,%rax,4),%xmm6
   43839:	pshufd $0xd4,%xmm0,%xmm1
   4383e:	pcmpeqd %xmm2,%xmm1
   43842:	pandn  %xmm3,%xmm1
   43846:	paddq  %xmm4,%xmm1
   4384a:	pshufd $0xd4,%xmm6,%xmm0
   4384f:	pcmpeqd %xmm2,%xmm0
   43853:	pandn  %xmm3,%xmm0
   43857:	paddq  %xmm5,%xmm0
   4385b:	add    $0x8,%rax
   4385f:	cmp    $0x100,%rax
   43865:	jne    43800 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x110>
   43867:	mov    $0x88888889,%r8d
   4386d:	imul   %rdx,%r8
   43871:	shr    $0x24,%r8
   43875:	imul   $0xeeeeeeef,%edx,%eax
   4387b:	ror    $1,%eax
   4387d:	cmp    $0x8888889,%eax
   43882:	sbb    $0xffffffffffffffff,%r8
   43886:	mov    %r8,%r14
   43889:	cmpl   $0x0,0x2c(%rsp)
   4388e:	jne    438e5 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f5>
   43890:	test   %rdx,%rdx
   43893:	je     438e2 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f2>
   43895:	paddq  %xmm1,%xmm0
   43899:	pshufd $0xee,%xmm0,%xmm1
   4389e:	paddq  %xmm0,%xmm1
   438a2:	movq   %xmm1,%rax
   438a7:	lea    -0x1(%rdx),%ecx
   438aa:	mov    $0x88888889,%esi
   438af:	imul   %rcx,%rsi
   438b3:	shr    $0x23,%rsi
   438b7:	and    $0xfffffffe,%esi
   438ba:	lea    (%rsi,%rdx,1),%ecx
   438bd:	dec    %ecx
   438bf:	shr    $1,%ecx
   438c1:	mov    $0x80010003,%r14d
   438c7:	imul   %rcx,%r14
   438cb:	shr    $0x2e,%r14
   438cf:	inc    %r14d
   438d2:	imul   %rax,%r14
   438d6:	dec    %r14
   438d9:	cmp    %r8,%r14
   438dc:	cmovae %r8,%r14
   438e0:	jmp    438e5 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f5>
   438e2:	xor    %r14d,%r14d
   438e5:	mov    (%r9),%rax
   438e8:	lea    0x1(%r8),%rcx
   438ec:	mov    %rcx,0x78(%rsp)
   438f1:	cmp    %r8,%rax
   438f4:	mov    %rdx,0x20(%rsp)
   438f9:	mov    %r8,0x58(%rsp)
   438fe:	jbe    455cd <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1edd>
   43904:	mov    (%rdi),%rax
   43907:	cmp    %r14,%rax
   4390a:	jb     4562a <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f3a>
   43910:	movq   $0x0,0x10(%rbp)
   43918:	lea    0x3f(%r8),%r14
   4391c:	shr    $0x6,%r14
   43920:	je     43971 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x281>
   43922:	cmp    0x0(%rbp),%r14
   43926:	ja     45728 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x2038>
   4392c:	xor    %r15d,%r15d
   4392f:	mov    0x8(%rbp),%r12
   43933:	lea    (%r12,%r15,8),%rdi
   43937:	cmp    $0x1,%r14
   4393b:	je     43965 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x275>
   4393d:	lea    -0x8(,%r14,8),%rdx
   43945:	xor    %esi,%esi
   43947:	mov    %r9,%r13
   4394a:	call   *0x1005f0(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   43950:	mov    %r13,%r9
   43953:	lea    (%r15,%r14,1),%rax
   43957:	lea    (%r12,%rax,8),%rdi
   4395b:	add    $0xfffffffffffffff8,%rdi
   4395f:	add    %r14,%r15
   43962:	dec    %r15
   43965:	movq   $0x0,(%rdi)
   4396c:	inc    %r15
   4396f:	jmp    43974 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x284>
   43971:	xor    %r15d,%r15d
   43974:	lea    0x18(%rbp),%r12
   43978:	mov    %r15,0x10(%rbp)
   4397c:	movq   $0x0,0x28(%rbp)
   43984:	cmpq   $0xf,0x18(%rbp)
   43989:	jbe    45703 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x2013>
   4398f:	lea    0x30(%rbp),%rax
   43993:	mov    %rax,0x48(%rsp)
   43998:	xor    %eax,%eax
   4399a:	pxor   %xmm0,%xmm0
   4399e:	movabs $0xffffffff00000000,%r13
   439a8:	nopl   0x0(%rax,%rax,1)
   439b0:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   439b9:	mov    %r13,0x2d08(%rsp,%rax,1)
   439c1:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   439ca:	mov    %r13,0x2d20(%rsp,%rax,1)
   439d2:	movdqu %xmm0,0x2d28(%rsp,%rax,1)
   439db:	mov    %r13,0x2d38(%rsp,%rax,1)
   439e3:	movdqu %xmm0,0x2d40(%rsp,%rax,1)
   439ec:	mov    %r13,0x2d50(%rsp,%rax,1)
   439f4:	add    $0x60,%rax
   439f8:	cmp    $0x1800,%rax
   439fe:	jne    439b0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x2c0>
   43a00:	mov    %r12,0x8(%rsp)
   43a05:	lea    0xc(%rbx),%rax
   43a09:	xor    %edx,%edx
   43a0b:	mov    $0x15,%ecx
   43a10:	mov    -0xc(%rax),%esi
   43a13:	add    %edx,%esi
   43a15:	mov    %edx,0x2cac(%rsp,%rcx,4)
   43a1c:	mov    %esi,0x2cb0(%rsp,%rcx,4)
   43a23:	mov    -0x8(%rax),%edx
   43a26:	add    %esi,%edx
   43a28:	mov    %esi,0x2cc4(%rsp,%rcx,4)
   43a2f:	mov    %edx,0x2cc8(%rsp,%rcx,4)
   43a36:	mov    -0x4(%rax),%esi
   43a39:	add    %edx,%esi
   43a3b:	mov    %edx,0x2cdc(%rsp,%rcx,4)
   43a42:	mov    %esi,0x2ce0(%rsp,%rcx,4)
   43a49:	mov    (%rax),%edx
   43a4b:	add    %esi,%edx
   43a4d:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   43a54:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   43a5b:	add    $0x18,%rcx
   43a5f:	add    $0x10,%rax
   43a63:	cmp    $0x615,%rcx
   43a6a:	jne    43a10 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x320>
   43a6c:	mov    %r9,0x90(%rsp)
   43a74:	mov    0x8(%r9),%r12
   43a78:	mov    0x8(%rbp),%r14
   43a7c:	mov    %rbp,0x88(%rsp)
   43a84:	mov    0x10(%rbp),%rbp
   43a88:	lea    0x8e8(%rsp),%rdi
   43a90:	xor    %r15d,%r15d
   43a93:	mov    $0x800,%edx
   43a98:	xor    %esi,%esi
   43a9a:	call   *0x1004a0(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   43aa0:	lea    0x10e8(%rsp),%rdi
   43aa8:	lea    0x2cf8(%rsp),%rsi
   43ab0:	mov    $0x1800,%edx
   43ab5:	call   *0x1004a5(%rip)        # 143f60 <memcpy@GLIBC_2.14>
   43abb:	mov    %r12,0xd0(%rsp)
   43ac3:	mov    0x58(%rsp),%rax
   43ac8:	mov    %rax,0xd8(%rsp)
   43ad0:	mov    0x20(%rsp),%rax
   43ad5:	mov    %rax,0xe0(%rsp)
   43add:	mov    0x4530(%rsp),%rax
   43ae5:	mov    %rax,0xa8(%rsp)
   43aed:	mov    %r14,0x18(%rsp)
   43af2:	mov    %r14,0xb0(%rsp)
   43afa:	mov    %rbp,0x10(%rsp)
   43aff:	mov    %rbp,0xb8(%rsp)
   43b07:	mov    0x8(%rsp),%rax
   43b0c:	mov    %rax,0xc0(%rsp)
   43b14:	lea    0xe8(%rsp),%rdi
   43b1c:	mov    $0x800,%edx
   43b21:	xor    %esi,%esi
   43b23:	call   *0x100417(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   43b29:	mov    %r13,0x28f0(%rsp)
   43b31:	movq   $0x0,0x28e8(%rsp)
   43b3d:	mov    0x48(%rsp),%rax
   43b42:	mov    %rax,0xc8(%rsp)
   43b4a:	mov    $0xffffffffffffffff,%rcx
   43b51:	movabs $0xeeeeeeeeeeeeeeef,%r8
   43b5b:	movabs $0x888888888888889,%r14
   43b65:	pxor   %xmm0,%xmm0
   43b69:	pcmpeqd %xmm1,%xmm1
   43b6d:	xor    %ebp,%ebp
   43b6f:	jmp    43c2b <<c30walk::ctor2c::Ctor2c>::build::<false>+0x53b>
   43b74:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   43b80:	mov    0x8(%rdi),%rax
   43b84:	mov    %r13,%rcx
   43b87:	shl    $0x7,%rcx
   43b8b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   43b91:	movdqa %xmm0,0x60(%rax,%rcx,1)
   43b97:	movdqa %xmm0,0x50(%rax,%rcx,1)
   43b9d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   43ba3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   43ba9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   43baf:	movdqa %xmm0,0x10(%rax,%rcx,1)
   43bb5:	movdqa %xmm0,(%rax,%rcx,1)
   43bba:	mov    %r13,%rax
   43bbd:	inc    %rax
   43bc0:	mov    %rax,0x10(%rdi)
   43bc4:	mov    %rdx,%rax
   43bc7:	shl    $0x6,%rax
   43bcb:	or     $0x10000000,%r13d
   43bd2:	movdqu %xmm1,0x2c(%r12,%rax,1)
   43bd9:	movdqa %xmm1,0x20(%r12,%rax,1)
   43be0:	movdqa %xmm1,0x10(%r12,%rax,1)
   43be7:	movdqa %xmm1,(%r12,%rax,1)
   43bed:	mov    %r13d,0x3c(%r12,%rax,1)
   43bf2:	mov    %rdx,%rdi
   43bf5:	shr    $0x6,%rdi
   43bf9:	cmp    0x10(%rsp),%rdi
   43bfe:	jae    4578f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x209f>
   43c04:	mov    $0x1,%eax
   43c09:	mov    %edx,%ecx
   43c0b:	shl    %cl,%rax
   43c0e:	mov    0x18(%rsp),%rcx
   43c13:	or     %rax,(%rcx,%rdi,8)
   43c17:	mov    %rdx,%rcx
   43c1a:	add    $0x4,%rbp
   43c1e:	cmp    $0x400,%rbp
   43c25:	je     43cb4 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x5c4>
   43c2b:	mov    %r15,%rax
   43c2e:	mov    (%rbx,%rbp,1),%r15d
   43c32:	add    %rax,%r15
   43c35:	cmp    0x20(%rsp),%r15
   43c3a:	setae  %sil
   43c3e:	mov    %r15,%rax
   43c41:	movabs $0x8888888888888889,%rdx
   43c4b:	mul    %rdx
   43c4e:	shr    $0x4,%rdx
   43c52:	mov    %r15,%rax
   43c55:	imul   %r8,%rax
   43c59:	ror    $1,%rax
   43c5c:	cmp    %r14,%rax
   43c5f:	setb   %al
   43c62:	cmp    %rcx,%rdx
   43c65:	sete   %dil
   43c69:	or     %sil,%dil
   43c6c:	or     %al,%dil
   43c6f:	jne    43c1a <<c30walk::ctor2c::Ctor2c>::build::<false>+0x52a>
   43c71:	mov    0x4530(%rsp),%rdi
   43c79:	mov    0x10(%rdi),%r13
   43c7d:	cmp    (%rdi),%r13
   43c80:	jne    43b80 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x490>
   43c86:	mov    %rdx,0x8(%rsp)
   43c8b:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   43c90:	pcmpeqd %xmm1,%xmm1
   43c94:	pxor   %xmm0,%xmm0
   43c98:	movabs $0xeeeeeeeeeeeeeeef,%r8
   43ca2:	mov    0x4530(%rsp),%rdi
   43caa:	mov    0x8(%rsp),%rdx
   43caf:	jmp    43b80 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x490>
   43cb4:	cmpl   $0x0,0x2c(%rsp)
   43cb9:	je     43e7e <<c30walk::ctor2c::Ctor2c>::build::<false>+0x78e>
   43cbf:	mov    0xd8(%rsp),%r10
   43cc7:	test   %r10,%r10
   43cca:	je     43e7e <<c30walk::ctor2c::Ctor2c>::build::<false>+0x78e>
   43cd0:	mov    0x20(%rsp),%r8
   43cd5:	shl    $0x20,%r8
   43cd9:	mov    0x2c(%rsp),%r9d
   43cde:	mov    0xa8(%rsp),%rdi
   43ce6:	mov    0xd0(%rsp),%rbp
   43cee:	xor    %r14d,%r14d
   43cf1:	movabs $0xff51afd7ed558ccd,%r11
   43cfb:	movabs $0xc4ceb9fe1a85ec53,%r12
   43d05:	pxor   %xmm0,%xmm0
   43d09:	pcmpeqd %xmm1,%xmm1
   43d0d:	jmp    43da0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x6b0>
   43d12:	mov    0x8(%rdi),%rax
   43d16:	mov    %r12,%rcx
   43d19:	shl    $0x7,%rcx
   43d1d:	movdqa %xmm0,0x70(%rax,%rcx,1)
   43d23:	movdqa %xmm0,0x60(%rax,%rcx,1)
   43d29:	movdqa %xmm0,0x50(%rax,%rcx,1)
   43d2f:	movdqa %xmm0,0x40(%rax,%rcx,1)
   43d35:	movdqa %xmm0,0x30(%rax,%rcx,1)
   43d3b:	movdqa %xmm0,0x20(%rax,%rcx,1)
   43d41:	movdqa %xmm0,0x10(%rax,%rcx,1)
   43d47:	movdqa %xmm0,(%rax,%rcx,1)
   43d4c:	mov    %r12,%rax
   43d4f:	inc    %rax
   43d52:	mov    %rax,0x10(%rdi)
   43d56:	or     $0x10000000,%r12d
   43d5d:	movdqu %xmm1,0x2c(%rbp)
   43d62:	movdqa %xmm1,0x20(%rbp)
   43d67:	movdqa %xmm1,0x10(%rbp)
   43d6c:	movdqa %xmm1,0x0(%rbp)
   43d71:	mov    %r12d,0x3c(%rbp)
   43d75:	or     %rbx,0x0(%r13,%r15,8)
   43d7a:	movabs $0xc4ceb9fe1a85ec53,%r12
   43d84:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   43d90:	inc    %r14
   43d93:	add    $0x40,%rbp
   43d97:	cmp    %r14,%r10
   43d9a:	je     43e7e <<c30walk::ctor2c::Ctor2c>::build::<false>+0x78e>
   43da0:	mov    %r14,%r15
   43da3:	shr    $0x6,%r15
   43da7:	mov    0xb8(%rsp),%rsi
   43daf:	cmp    %rsi,%r15
   43db2:	jae    457a0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x20b0>
   43db8:	mov    0xb0(%rsp),%r13
   43dc0:	mov    0x0(%r13,%r15,8),%rax
   43dc5:	mov    $0x1,%ebx
   43dca:	mov    %r14d,%ecx
   43dcd:	shl    %cl,%rbx
   43dd0:	bt     %r14,%rax
   43dd4:	jb     43d90 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x6a0>
   43dd6:	mov    %r14,%rax
   43dd9:	xor    %r8,%rax
   43ddc:	mov    %rax,%rcx
   43ddf:	shr    $0x21,%rcx
   43de3:	xor    %rax,%rcx
   43de6:	imul   %r11,%rcx
   43dea:	mov    %rcx,%rax
   43ded:	shr    $0x21,%rax
   43df1:	xor    %rcx,%rax
   43df4:	imul   %r12,%rax
   43df8:	mov    %rax,%rcx
   43dfb:	shr    $0x21,%rcx
   43dff:	xor    %rax,%rcx
   43e02:	mov    %rcx,%rax
   43e05:	shr    $0x3,%rax
   43e09:	movabs $0x20c49ba5e353f7cf,%rdx
   43e13:	mul    %rdx
   43e16:	shr    $0x4,%rdx
   43e1a:	imul   $0x3e8,%rdx,%rax
   43e21:	sub    %rax,%rcx
   43e24:	cmp    %r9,%rcx
   43e27:	jae    43d90 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x6a0>
   43e2d:	mov    0x10(%rdi),%r12
   43e31:	cmp    (%rdi),%r12
   43e34:	jne    43d12 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x622>
   43e3a:	mov    %r10,0x10(%rsp)
   43e3f:	mov    %r8,0x18(%rsp)
   43e44:	mov    %r9,0x8(%rsp)
   43e49:	mov    %rdi,0x48(%rsp)
   43e4e:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   43e53:	pcmpeqd %xmm1,%xmm1
   43e57:	pxor   %xmm0,%xmm0
   43e5b:	movabs $0xff51afd7ed558ccd,%r11
   43e65:	mov    0x48(%rsp),%rdi
   43e6a:	mov    0x8(%rsp),%r9
   43e6f:	mov    0x18(%rsp),%r8
   43e74:	mov    0x10(%rsp),%r10
   43e79:	jmp    43d12 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x622>
   43e7e:	cmpq   $0x0,0x20(%rsp)
   43e84:	je     453d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1ce0>
   43e8a:	movq   $0x0,0x30(%rsp)
   43e93:	jmp    43ed7 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x7e7>
   43e95:	mov    0x38(%rsp),%r12
   43e9a:	lea    0x1(%rax),%rdx
   43e9e:	add    $0xffffffffffffffe3,%rax
   43ea2:	xor    %esi,%esi
   43ea4:	cmp    $0x1e,%rdx
   43ea8:	setae  %sil
   43eac:	cmovb  %rdx,%rax
   43eb0:	lea    (%rcx,%rsi,2),%ecx
   43eb3:	inc    %ecx
   43eb5:	mov    %rax,0x28e8(%rsp)
   43ebd:	mov    %ecx,0x28f0(%rsp)
   43ec4:	mov    0x98(%rsp),%rax
   43ecc:	cmp    0x20(%rsp),%rax
   43ed1:	jae    453d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1ce0>
   43ed7:	mov    0x30(%rsp),%r15
   43edc:	mov    $0x400,%edx
   43ee1:	lea    0x28f8(%rsp),%rdi
   43ee9:	xor    %esi,%esi
   43eeb:	call   *0x10004f(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   43ef1:	mov    $0x848,%eax
   43ef6:	mov    $0xa,%ecx
   43efb:	nopl   0x0(%rax,%rax,1)
   43f00:	movd   0x78(%rsp,%rax,2),%xmm0
   43f06:	movd   0x60(%rsp,%rax,2),%xmm1
   43f0c:	punpckldq %xmm0,%xmm1
   43f10:	movd   0xa8(%rsp,%rax,2),%xmm0
   43f19:	movd   0x90(%rsp,%rax,2),%xmm2
   43f22:	punpckldq %xmm0,%xmm2
   43f26:	punpcklqdq %xmm2,%xmm1
   43f2a:	movdqu 0x98(%rsp,%rcx,8),%xmm0
   43f33:	movdqu 0xa8(%rsp,%rcx,8),%xmm2
   43f3c:	movq   0x70(%rsp,%rax,2),%xmm3
   43f42:	movq   0x58(%rsp,%rax,2),%xmm4
   43f48:	punpcklqdq %xmm3,%xmm4
   43f4c:	psubq  %xmm4,%xmm0
   43f50:	movq   0xa0(%rsp,%rax,2),%xmm3
   43f59:	movq   0x88(%rsp,%rax,2),%xmm4
   43f62:	punpcklqdq %xmm3,%xmm4
   43f66:	psubq  %xmm4,%xmm2
   43f6a:	psrlq  $0x1,%xmm0
   43f6f:	psrlq  $0x1,%xmm2
   43f74:	pshufd $0xe8,%xmm0,%xmm0
   43f79:	pshufd $0xe8,%xmm2,%xmm2
   43f7e:	punpcklqdq %xmm2,%xmm0
   43f82:	paddd  %xmm1,%xmm0
   43f86:	movdqu %xmm0,0x28d0(%rsp,%rcx,4)
   43f8f:	add    $0x30,%rax
   43f93:	add    $0x4,%rcx
   43f97:	cmp    $0x106,%rcx
   43f9e:	jne    43f00 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x810>
   43fa4:	movups 0x2890(%rsp),%xmm0
   43fac:	movdqu 0x8d8(%rsp),%xmm1
   43fb5:	movdqu 0x8c8(%rsp),%xmm2
   43fbe:	movq   0x28d0(%rsp),%xmm3
   43fc7:	movq   0x28b8(%rsp),%xmm4
   43fd0:	punpcklqdq %xmm3,%xmm4
   43fd4:	psubq  %xmm4,%xmm1
   43fd8:	movq   0x28a0(%rsp),%xmm3
   43fe1:	movq   0x2888(%rsp),%xmm4
   43fea:	punpcklqdq %xmm3,%xmm4
   43fee:	psubq  %xmm4,%xmm2
   43ff2:	psrlq  $0x1,%xmm1
   43ff7:	psrlq  $0x1,%xmm2
   43ffc:	shufps $0x88,%xmm1,%xmm2
   44000:	movss  0x28a8(%rsp),%xmm1
   44009:	movlhps %xmm0,%xmm1
   4400c:	movss  0x28c0(%rsp),%xmm0
   44015:	movss  0x28d8(%rsp),%xmm3
   4401e:	movlhps %xmm0,%xmm3
   44021:	shufps $0x22,%xmm3,%xmm1
   44025:	paddd  %xmm2,%xmm1
   44029:	movdqu %xmm1,0x2ce8(%rsp)
   44032:	mov    0xc0(%rsp),%rbx
   4403a:	mov    0x10(%rbx),%r14
   4403e:	cmp    (%rbx),%r14
   44041:	je     453c3 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1cd3>
   44047:	mov    %r14,%rdi
   4404a:	shl    $0xa,%rdi
   4404e:	add    0x8(%rbx),%rdi
   44052:	mov    $0x400,%edx
   44057:	lea    0x28f8(%rsp),%rsi
   4405f:	call   *0xffefb(%rip)        # 143f60 <memcpy@GLIBC_2.14>
   44065:	inc    %r14
   44068:	mov    %r14,0x10(%rbx)
   4406c:	shl    $0x18,%r14d
   44070:	add    $0x7f000000,%r14d
   44077:	or     $0x80000000,%r14d
   4407e:	lea    0xf000(%r15),%rdx
   44085:	mov    0x20(%rsp),%r13
   4408a:	cmp    %r13,%rdx
   4408d:	mov    %rdx,0x98(%rsp)
   44095:	cmovb  %rdx,%r13
   44099:	mov    %r14d,0x28f4(%rsp)
   440a1:	mov    %r13,0x30(%rsp)
   440a6:	sub    %r15,%r13
   440a9:	cmp    $0x10,%r13
   440ad:	jb     451f0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1b00>
   440b3:	add    0x60(%rsp),%r15
   440b8:	xor    %edx,%edx
   440ba:	jmp    4410b <<c30walk::ctor2c::Ctor2c>::build::<false>+0xa1b>
   440bc:	nopl   0x0(%rax)
   440c0:	mov    0x28e8(%rsp),%rax
   440c8:	lea    0x10(%rax),%rcx
   440cc:	add    $0xfffffffffffffff2,%rax
   440d0:	xor    %edx,%edx
   440d2:	cmp    $0x1e,%rcx
   440d6:	setae  %dl
   440d9:	cmovb  %rcx,%rax
   440dd:	mov    0x28f0(%rsp),%ecx
   440e4:	lea    (%rcx,%rdx,2),%ecx
   440e7:	add    $0x10,%ecx
   440ea:	mov    %rax,0x28e8(%rsp)
   440f2:	mov    %ecx,0x28f0(%rsp)
   440f9:	xor    %edx,%edx
   440fb:	mov    %rsi,%r15
   440fe:	mov    %rdi,%r13
   44101:	cmp    $0xf,%rdi
   44105:	jbe    451f3 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1b03>
   4410b:	mov    0x70(%rsp),%rcx
   44110:	test   $0x1,%dl
   44113:	jne    4411c <<c30walk::ctor2c::Ctor2c>::build::<false>+0xa2c>
   44115:	mov    (%r15),%rcx
   44118:	mov    0x8(%r15),%r12
   4411c:	lea    0x10(%r15),%rsi
   44120:	lea    -0x10(%r13),%rdi
   44124:	xor    %rcx,%r12
   44127:	mov    %rcx,%rax
   4412a:	rol    $0x8,%rax
   4412e:	xor    %rcx,%rax
   44131:	or     %r12,%rax
   44134:	jne    44190 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xaa0>
   44136:	cmp    $0x10,%rdi
   4413a:	lea    0x10e8(%rsp),%r8
   44142:	jb     44540 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe50>
   44148:	nopl   0x0(%rax,%rax,1)
   44150:	mov    (%rsi),%rdx
   44153:	mov    0x8(%rsi),%rax
   44157:	mov    $0x1,%r9b
   4415a:	cmp    %rcx,%rdx
   4415d:	jne    4454a <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe5a>
   44163:	cmp    %rcx,%rax
   44166:	jne    4454a <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe5a>
   4416c:	add    $0xfffffffffffffff0,%rdi
   44170:	add    $0x10,%rsi
   44174:	cmp    $0xf,%rdi
   44178:	ja     44150 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xa60>
   4417a:	mov    %rsi,0x40(%rsp)
   4417f:	xor    %r9d,%r9d
   44182:	mov    %rcx,0x70(%rsp)
   44187:	jmp    44559 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe69>
   4418c:	nopl   0x0(%rax)
   44190:	movzwl 0x28f0(%rsp),%ebp
   44198:	mov    0x28e8(%rsp),%r9
   441a0:	mov    $0x1e,%ebx
   441a5:	sub    %r9,%rbx
   441a8:	cmp    $0x10,%rbx
   441ac:	mov    $0x10,%edx
   441b1:	cmovb  %rbx,%rdx
   441b5:	movzbl (%r15),%r14d
   441b9:	mov    0xe8(%rsp,%r14,8),%rax
   441c1:	cmp    0x8e8(%rsp,%r14,8),%rax
   441c9:	je     44cd8 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x15e8>
   441cf:	mov    0x30(%rsp),%r12
   441d4:	sub    %r13,%r12
   441d7:	add    0x60(%rsp),%r12
   441dc:	mov    %bp,(%rax)
   441df:	addq   $0x2,0xe8(%rsp,%r14,8)
   441e8:	cmp    $0x2,%rbx
   441ec:	jb     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   441f2:	movzbl 0x1(%r15),%r14d
   441f7:	mov    0xe8(%rsp,%r14,8),%rax
   441ff:	cmp    0x8e8(%rsp,%r14,8),%rax
   44207:	je     44d15 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1625>
   4420d:	lea    0x1(%rbp),%ecx
   44210:	mov    %cx,(%rax)
   44213:	addq   $0x2,0xe8(%rsp,%r14,8)
   4421c:	cmp    $0x1c,%r9
   44220:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   44226:	movzbl 0x2(%r15),%r14d
   4422b:	mov    0xe8(%rsp,%r14,8),%rax
   44233:	cmp    0x8e8(%rsp,%r14,8),%rax
   4423b:	je     44d64 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1674>
   44241:	lea    0x2(%rbp),%ecx
   44244:	mov    %cx,(%rax)
   44247:	addq   $0x2,0xe8(%rsp,%r14,8)
   44250:	cmp    $0x1b,%r9
   44254:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4425a:	movzbl 0x3(%r15),%r14d
   4425f:	mov    0xe8(%rsp,%r14,8),%rax
   44267:	cmp    0x8e8(%rsp,%r14,8),%rax
   4426f:	je     44db3 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x16c3>
   44275:	lea    0x3(%rbp),%ecx
   44278:	mov    %cx,(%rax)
   4427b:	addq   $0x2,0xe8(%rsp,%r14,8)
   44284:	cmp    $0x1a,%r9
   44288:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4428e:	movzbl 0x4(%r15),%r14d
   44293:	mov    0xe8(%rsp,%r14,8),%rax
   4429b:	cmp    0x8e8(%rsp,%r14,8),%rax
   442a3:	je     44e02 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1712>
   442a9:	lea    0x4(%rbp),%ecx
   442ac:	mov    %cx,(%rax)
   442af:	addq   $0x2,0xe8(%rsp,%r14,8)
   442b8:	cmp    $0x19,%r9
   442bc:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   442c2:	movzbl 0x5(%r15),%r14d
   442c7:	mov    0xe8(%rsp,%r14,8),%rax
   442cf:	cmp    0x8e8(%rsp,%r14,8),%rax
   442d7:	je     44e95 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x17a5>
   442dd:	lea    0x5(%rbp),%ecx
   442e0:	mov    %cx,(%rax)
   442e3:	addq   $0x2,0xe8(%rsp,%r14,8)
   442ec:	cmp    $0x18,%r9
   442f0:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   442f6:	movzbl 0x6(%r15),%r14d
   442fb:	mov    0xe8(%rsp,%r14,8),%rax
   44303:	cmp    0x8e8(%rsp,%r14,8),%rax
   4430b:	je     44ee4 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x17f4>
   44311:	lea    0x6(%rbp),%ecx
   44314:	mov    %cx,(%rax)
   44317:	addq   $0x2,0xe8(%rsp,%r14,8)
   44320:	cmp    $0x17,%r9
   44324:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4432a:	movzbl 0x7(%r15),%r14d
   4432f:	mov    0xe8(%rsp,%r14,8),%rax
   44337:	cmp    0x8e8(%rsp,%r14,8),%rax
   4433f:	je     44f33 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1843>
   44345:	lea    0x7(%rbp),%ecx
   44348:	mov    %cx,(%rax)
   4434b:	addq   $0x2,0xe8(%rsp,%r14,8)
   44354:	cmp    $0x16,%r9
   44358:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4435e:	movzbl 0x8(%r15),%r14d
   44363:	mov    0xe8(%rsp,%r14,8),%rax
   4436b:	cmp    0x8e8(%rsp,%r14,8),%rax
   44373:	je     44f82 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1892>
   44379:	lea    0x8(%rbp),%ecx
   4437c:	mov    %cx,(%rax)
   4437f:	addq   $0x2,0xe8(%rsp,%r14,8)
   44388:	cmp    $0x15,%r9
   4438c:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   44392:	movzbl 0x9(%r15),%r14d
   44397:	mov    0xe8(%rsp,%r14,8),%rax
   4439f:	cmp    0x8e8(%rsp,%r14,8),%rax
   443a7:	je     44fd1 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x18e1>
   443ad:	lea    0x9(%rbp),%ecx
   443b0:	mov    %cx,(%rax)
   443b3:	addq   $0x2,0xe8(%rsp,%r14,8)
   443bc:	cmp    $0x14,%r9
   443c0:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   443c6:	movzbl 0xa(%r15),%r14d
   443cb:	mov    0xe8(%rsp,%r14,8),%rax
   443d3:	cmp    0x8e8(%rsp,%r14,8),%rax
   443db:	je     45020 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1930>
   443e1:	lea    0xa(%rbp),%ecx
   443e4:	mov    %cx,(%rax)
   443e7:	addq   $0x2,0xe8(%rsp,%r14,8)
   443f0:	cmp    $0x13,%r9
   443f4:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   443fa:	movzbl 0xb(%r15),%r14d
   443ff:	mov    0xe8(%rsp,%r14,8),%rax
   44407:	cmp    0x8e8(%rsp,%r14,8),%rax
   4440f:	je     4506f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x197f>
   44415:	lea    0xb(%rbp),%ecx
   44418:	mov    %cx,(%rax)
   4441b:	addq   $0x2,0xe8(%rsp,%r14,8)
   44424:	cmp    $0x12,%r9
   44428:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4442e:	movzbl 0xc(%r15),%r14d
   44433:	mov    0xe8(%rsp,%r14,8),%rax
   4443b:	cmp    0x8e8(%rsp,%r14,8),%rax
   44443:	je     450be <<c30walk::ctor2c::Ctor2c>::build::<false>+0x19ce>
   44449:	lea    0xc(%rbp),%ecx
   4444c:	mov    %cx,(%rax)
   4444f:	addq   $0x2,0xe8(%rsp,%r14,8)
   44458:	cmp    $0x11,%r9
   4445c:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4445e:	movzbl 0xd(%r15),%r14d
   44463:	mov    0xe8(%rsp,%r14,8),%rax
   4446b:	cmp    0x8e8(%rsp,%r14,8),%rax
   44473:	je     4510d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1a1d>
   44479:	lea    0xd(%rbp),%ecx
   4447c:	mov    %cx,(%rax)
   4447f:	addq   $0x2,0xe8(%rsp,%r14,8)
   44488:	cmp    $0x10,%r9
   4448c:	je     444d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde0>
   4448e:	movzbl 0xe(%r15),%r14d
   44493:	mov    0xe8(%rsp,%r14,8),%rax
   4449b:	cmp    0x8e8(%rsp,%r14,8),%rax
   444a3:	je     4515c <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1a6c>
   444a9:	lea    0xe(%rbp),%ecx
   444ac:	mov    %cx,(%rax)
   444af:	addq   $0x2,0xe8(%rsp,%r14,8)
   444b8:	cmp    $0xf,%r9
   444bc:	jne    44e51 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1761>
   444c2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   444d0:	lea    0x10(%r12),%rbx
   444d5:	add    %rdx,%r12
   444d8:	add    %edx,%ebp
   444da:	add    $0x2,%ebp
   444dd:	nopl   (%rax)
   444e0:	movzbl (%r12),%r14d
   444e5:	mov    0xe8(%rsp,%r14,8),%rax
   444ed:	cmp    0x8e8(%rsp,%r14,8),%rax
   444f5:	je     44512 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe22>
   444f7:	mov    %bp,(%rax)
   444fa:	addq   $0x2,0xe8(%rsp,%r14,8)
   44503:	inc    %r12
   44506:	inc    %ebp
   44508:	cmp    %rbx,%r12
   4450b:	jne    444e0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xdf0>
   4450d:	jmp    440c0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x9d0>
   44512:	mov    %rsi,%r13
   44515:	mov    %rdi,%r15
   44518:	lea    0xa8(%rsp),%rdi
   44520:	mov    %r14,%rsi
   44523:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44528:	test   %al,%al
   4452a:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44530:	mov    0xe8(%rsp,%r14,8),%rax
   44538:	mov    %r15,%rdi
   4453b:	mov    %r13,%rsi
   4453e:	jmp    444f7 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe07>
   44540:	mov    %rsi,0x40(%rsp)
   44545:	xor    %r9d,%r9d
   44548:	jmp    44559 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xe69>
   4454a:	mov    %rdx,0x70(%rsp)
   4454f:	mov    %rax,0x38(%rsp)
   44554:	mov    %rsi,0x40(%rsp)
   44559:	movzbl %cl,%ebx
   4455c:	lea    (%rbx,%rbx,2),%rdx
   44560:	mov    %ebx,%eax
   44562:	mov    0xe8(%rsp,%rax,8),%rax
   4456a:	mov    %rax,%rsi
   4456d:	sub    (%r8,%rdx,8),%rsi
   44571:	shr    $1,%rsi
   44574:	add    0x8(%r8,%rdx,8),%esi
   44579:	mov    %rdi,0x68(%rsp)
   4457e:	sub    %rdi,%r13
   44581:	add    %r13,%rsi
   44584:	mov    0xc(%r8,%rdx,8),%edi
   44589:	cmp    %rdi,%rsi
   4458c:	ja     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44592:	mov    %r9d,0x54(%rsp)
   44597:	lea    (%r8,%rdx,8),%r10
   4459b:	movzbl %cl,%r12d
   4459f:	mov    0x8e8(%rsp,%rbx,8),%rsi
   445a7:	mov    $0x88888889,%r14d
   445ad:	lea    -0x27274(%rip),%r15        # 1d340 <c30walk::ctor2c::PATTERNS>
   445b4:	mov    %rbx,0x80(%rsp)
   445bc:	mov    %r10,0x8(%rsp)
   445c1:	jmp    44608 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xf18>
   445c3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   445d0:	mov    0x3c(%rsi),%edi
   445d3:	mov    $0xfffff,%r9d
   445d9:	and    %r9d,%edi
   445dc:	sub    %r8d,%ecx
   445df:	sub    %eax,%r8d
   445e2:	mov    %r8d,%eax
   445e5:	lea    (%rsi,%rax,2),%rax
   445e9:	mov    %rax,0xe8(%rsp,%rbx,8)
   445f1:	lea    (%rax,%rcx,2),%rsi
   445f5:	mov    %rsi,0x8e8(%rsp,%rbx,8)
   445fd:	mov    %rax,(%r10)
   44600:	mov    %edx,0x10(%r10)
   44604:	mov    %edi,0x14(%r10)
   44608:	sub    %rax,%rsi
   4460b:	shr    $1,%rsi
   4460e:	cmp    %rsi,%r13
   44611:	cmovb  %r13,%rsi
   44615:	test   %rsi,%rsi
   44618:	je     446d2 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xfe2>
   4461e:	mov    0x28e8(%rsp),%rdi
   44626:	cmp    $0x1d,%rdi
   4462a:	ja     4576d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x207d>
   44630:	cmp    $0x1f,%rsi
   44634:	jae    45441 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d51>
   4463a:	mov    0x28f0(%rsp),%ecx
   44641:	mov    %rdi,%r11
   44644:	shl    $0x6,%r11
   44648:	lea    (%r15,%r11,1),%rdx
   4464c:	lea    -0x2(,%rsi,2),%r9
   44654:	cmp    $0xe,%r9
   44658:	jb     4466d <<c30walk::ctor2c::Ctor2c>::build::<false>+0xf7d>
   4465a:	mov    %rax,%r8
   4465d:	sub    %r15,%r8
   44660:	sub    %r11,%r8
   44663:	cmp    $0x10,%r8
   44667:	jae    44ac7 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x13d7>
   4466d:	mov    %rdx,%r8
   44670:	xor    %r10d,%r10d
   44673:	lea    (%rdx,%rsi,2),%rdx
   44677:	lea    (%rax,%r10,2),%r9
   4467b:	nopl   0x0(%rax,%rax,1)
   44680:	movzwl (%r8),%r10d
   44684:	add    %cx,%r10w
   44688:	add    $0x2,%r8
   4468c:	mov    %r10w,(%r9)
   44690:	add    $0x2,%r9
   44694:	cmp    %rdx,%r8
   44697:	jne    44680 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xf90>
   44699:	lea    (%rax,%rsi,2),%rax
   4469d:	mov    %rax,0xe8(%rsp,%rbx,8)
   446a5:	add    %rsi,%rdi
   446a8:	lea    0x2(%rsi),%edx
   446ab:	mov    %rdi,%r8
   446ae:	sub    $0x1e,%r8
   446b2:	cmovb  %rdi,%r8
   446b6:	cmovb  %esi,%edx
   446b9:	mov    %r8,0x28e8(%rsp)
   446c1:	add    %ecx,%edx
   446c3:	mov    %edx,0x28f0(%rsp)
   446ca:	sub    %rsi,%r13
   446cd:	mov    0x8(%rsp),%r10
   446d2:	test   %r13,%r13
   446d5:	je     44cb0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x15c0>
   446db:	mov    %rax,%r8
   446de:	cmp    (%r10),%rax
   446e1:	je     44702 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1012>
   446e3:	lea    0xa8(%rsp),%rdi
   446eb:	mov    %ebx,%esi
   446ed:	call   8adc0 <<c30walk::ctor2c::W<false>>::finish>
   446f2:	mov    0x8(%rsp),%r10
   446f7:	mov    0xe8(%rsp,%rbx,8),%r8
   446ff:	mov    (%r10),%rax
   44702:	sub    %rax,%r8
   44705:	shr    $1,%r8
   44708:	add    0x8(%r10),%r8d
   4470c:	mov    %r8d,0x8(%r10)
   44710:	movq   $0x0,(%r10)
   44717:	movq   $0x0,0xe8(%rsp,%rbx,8)
   44723:	movq   $0x0,0x8e8(%rsp,%rbx,8)
   4472f:	cmp    $0x1e,%r13
   44733:	jb     44c45 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1555>
   44739:	mov    %r8,%rcx
   4473c:	imul   %r14,%rcx
   44740:	shr    $0x24,%rcx
   44744:	mov    %ecx,%eax
   44746:	shl    $0x5,%eax
   44749:	lea    (%rcx,%rcx,1),%edx
   4474c:	sub    %eax,%edx
   4474e:	add    %r8d,%edx
   44751:	jne    44c45 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1555>
   44757:	mov    %r13,%rax
   4475a:	movabs $0x8888888888888889,%rdx
   44764:	mul    %rdx
   44767:	shr    $0x4,%rdx
   4476b:	mov    %ecx,%r9d
   4476e:	cmpl   $0x0,0x2c(%rsp)
   44773:	mov    %r13,0x48(%rsp)
   44778:	mov    %rdx,0xa0(%rsp)
   44780:	je     44a80 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1390>
   44786:	mov    0x28f0(%rsp),%r10d
   4478e:	mov    0xb8(%rsp),%rax
   44796:	mov    %rax,0x10(%rsp)
   4479b:	mov    0xa8(%rsp),%r11
   447a3:	mov    0xb0(%rsp),%rax
   447ab:	mov    %rax,0x18(%rsp)
   447b0:	mov    0xd0(%rsp),%rax
   447b8:	mov    0x28e8(%rsp),%rbx
   447c0:	mov    %rbx,%rdi
   447c3:	shl    $0x6,%rdi
   447c7:	add    %r15,%rdi
   447ca:	mov    %r9,%rcx
   447cd:	shl    $0x6,%rcx
   447d1:	lea    (%rax,%rcx,1),%r14
   447d5:	add    $0x3c,%r14
   447d9:	mov    %r10d,%r13d
   447dc:	shl    $0x8,%r13d
   447e0:	or     %r12d,%r13d
   447e3:	xor    %r15d,%r15d
   447e6:	jmp    449f4 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1304>
   447eb:	nopl   0x0(%rax,%rax,1)
   447f0:	mov    0x8(%r11),%rcx
   447f4:	shl    $0x7,%eax
   447f7:	mov    %r13d,(%rcx,%rax,1)
   447fb:	movzwl 0x2(%rdi),%esi
   447ff:	lea    (%r10,%r15,1),%ebp
   44803:	add    %ebp,%esi
   44805:	shl    $0x8,%esi
   44808:	or     %r12d,%esi
   4480b:	mov    %esi,0x4(%rcx,%rax,1)
   4480f:	movzwl 0x4(%rdi),%esi
   44813:	add    %ebp,%esi
   44815:	shl    $0x8,%esi
   44818:	or     %r12d,%esi
   4481b:	mov    %esi,0x8(%rcx,%rax,1)
   4481f:	movzwl 0x6(%rdi),%esi
   44823:	add    %ebp,%esi
   44825:	shl    $0x8,%esi
   44828:	or     %r12d,%esi
   4482b:	mov    %esi,0xc(%rcx,%rax,1)
   4482f:	movzwl 0x8(%rdi),%esi
   44833:	add    %ebp,%esi
   44835:	shl    $0x8,%esi
   44838:	or     %r12d,%esi
   4483b:	mov    %esi,0x10(%rcx,%rax,1)
   4483f:	movzwl 0xa(%rdi),%esi
   44843:	add    %ebp,%esi
   44845:	shl    $0x8,%esi
   44848:	or     %r12d,%esi
   4484b:	mov    %esi,0x14(%rcx,%rax,1)
   4484f:	movzwl 0xc(%rdi),%esi
   44853:	add    %ebp,%esi
   44855:	shl    $0x8,%esi
   44858:	or     %r12d,%esi
   4485b:	mov    %esi,0x18(%rcx,%rax,1)
   4485f:	movzwl 0xe(%rdi),%esi
   44863:	add    %ebp,%esi
   44865:	shl    $0x8,%esi
   44868:	or     %r12d,%esi
   4486b:	mov    %esi,0x1c(%rcx,%rax,1)
   4486f:	movzwl 0x10(%rdi),%esi
   44873:	add    %ebp,%esi
   44875:	shl    $0x8,%esi
   44878:	or     %r12d,%esi
   4487b:	mov    %esi,0x20(%rcx,%rax,1)
   4487f:	movzwl 0x12(%rdi),%esi
   44883:	add    %ebp,%esi
   44885:	shl    $0x8,%esi
   44888:	or     %r12d,%esi
   4488b:	mov    %esi,0x24(%rcx,%rax,1)
   4488f:	movzwl 0x14(%rdi),%esi
   44893:	add    %ebp,%esi
   44895:	shl    $0x8,%esi
   44898:	or     %r12d,%esi
   4489b:	mov    %esi,0x28(%rcx,%rax,1)
   4489f:	movzwl 0x16(%rdi),%esi
   448a3:	add    %ebp,%esi
   448a5:	shl    $0x8,%esi
   448a8:	or     %r12d,%esi
   448ab:	mov    %esi,0x2c(%rcx,%rax,1)
   448af:	movzwl 0x18(%rdi),%esi
   448b3:	add    %ebp,%esi
   448b5:	shl    $0x8,%esi
   448b8:	or     %r12d,%esi
   448bb:	mov    %esi,0x30(%rcx,%rax,1)
   448bf:	movzwl 0x1a(%rdi),%esi
   448c3:	add    %ebp,%esi
   448c5:	shl    $0x8,%esi
   448c8:	or     %r12d,%esi
   448cb:	mov    %esi,0x34(%rcx,%rax,1)
   448cf:	movzwl 0x1c(%rdi),%esi
   448d3:	add    %ebp,%esi
   448d5:	shl    $0x8,%esi
   448d8:	or     %r12d,%esi
   448db:	mov    %esi,0x38(%rcx,%rax,1)
   448df:	movzwl 0x1e(%rdi),%esi
   448e3:	add    %ebp,%esi
   448e5:	shl    $0x8,%esi
   448e8:	or     %r12d,%esi
   448eb:	mov    %esi,0x3c(%rcx,%rax,1)
   448ef:	movzwl 0x20(%rdi),%esi
   448f3:	add    %ebp,%esi
   448f5:	shl    $0x8,%esi
   448f8:	or     %r12d,%esi
   448fb:	mov    %esi,0x40(%rcx,%rax,1)
   448ff:	movzwl 0x22(%rdi),%esi
   44903:	add    %ebp,%esi
   44905:	shl    $0x8,%esi
   44908:	or     %r12d,%esi
   4490b:	mov    %esi,0x44(%rcx,%rax,1)
   4490f:	movzwl 0x24(%rdi),%esi
   44913:	add    %ebp,%esi
   44915:	shl    $0x8,%esi
   44918:	or     %r12d,%esi
   4491b:	mov    %esi,0x48(%rcx,%rax,1)
   4491f:	movzwl 0x26(%rdi),%esi
   44923:	add    %ebp,%esi
   44925:	shl    $0x8,%esi
   44928:	or     %r12d,%esi
   4492b:	mov    %esi,0x4c(%rcx,%rax,1)
   4492f:	movzwl 0x28(%rdi),%esi
   44933:	add    %ebp,%esi
   44935:	shl    $0x8,%esi
   44938:	or     %r12d,%esi
   4493b:	mov    %esi,0x50(%rcx,%rax,1)
   4493f:	movzwl 0x2a(%rdi),%esi
   44943:	add    %ebp,%esi
   44945:	shl    $0x8,%esi
   44948:	or     %r12d,%esi
   4494b:	mov    %esi,0x54(%rcx,%rax,1)
   4494f:	movzwl 0x2c(%rdi),%esi
   44953:	add    %ebp,%esi
   44955:	shl    $0x8,%esi
   44958:	or     %r12d,%esi
   4495b:	mov    %esi,0x58(%rcx,%rax,1)
   4495f:	movzwl 0x2e(%rdi),%esi
   44963:	add    %ebp,%esi
   44965:	shl    $0x8,%esi
   44968:	or     %r12d,%esi
   4496b:	mov    %esi,0x5c(%rcx,%rax,1)
   4496f:	movzwl 0x30(%rdi),%esi
   44973:	add    %ebp,%esi
   44975:	shl    $0x8,%esi
   44978:	or     %r12d,%esi
   4497b:	mov    %esi,0x60(%rcx,%rax,1)
   4497f:	movzwl 0x32(%rdi),%esi
   44983:	add    %ebp,%esi
   44985:	shl    $0x8,%esi
   44988:	or     %r12d,%esi
   4498b:	mov    %esi,0x64(%rcx,%rax,1)
   4498f:	movzwl 0x34(%rdi),%esi
   44993:	add    %ebp,%esi
   44995:	shl    $0x8,%esi
   44998:	or     %r12d,%esi
   4499b:	mov    %esi,0x68(%rcx,%rax,1)
   4499f:	movzwl 0x36(%rdi),%esi
   449a3:	add    %ebp,%esi
   449a5:	shl    $0x8,%esi
   449a8:	or     %r12d,%esi
   449ab:	mov    %esi,0x6c(%rcx,%rax,1)
   449af:	movzwl 0x38(%rdi),%esi
   449b3:	add    %ebp,%esi
   449b5:	shl    $0x8,%esi
   449b8:	or     %r12d,%esi
   449bb:	mov    %esi,0x70(%rcx,%rax,1)
   449bf:	movzwl 0x3a(%rdi),%esi
   449c3:	add    %ebp,%esi
   449c5:	shl    $0x8,%esi
   449c8:	or     %r12d,%esi
   449cb:	mov    %esi,0x74(%rcx,%rax,1)
   449cf:	lea    (%r10,%r15,1),%eax
   449d3:	add    $0x20,%eax
   449d6:	mov    %eax,0x28f0(%rsp)
   449dd:	add    $0x20,%r15
   449e1:	inc    %r9
   449e4:	add    $0x2000,%r13d
   449eb:	dec    %rdx
   449ee:	je     44bf9 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1509>
   449f4:	mov    %r9,%rax
   449f7:	shr    $0x6,%rax
   449fb:	cmp    0x10(%rsp),%rax
   44a00:	jae    45685 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f95>
   44a06:	mov    0x18(%rsp),%rcx
   44a0b:	mov    (%rcx,%rax,8),%rax
   44a0f:	bt     %r9,%rax
   44a13:	jae    44a40 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1350>
   44a15:	cmp    $0x1e,%rbx
   44a19:	jae    456ab <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fbb>
   44a1f:	mov    (%r14,%r15,2),%eax
   44a23:	and    $0xfffff,%eax
   44a28:	mov    0x10(%r11),%rcx
   44a2c:	cmp    %rax,%rcx
   44a2f:	ja     447f0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1100>
   44a35:	jmp    45699 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fa9>
   44a3a:	nopw   0x0(%rax,%rax,1)
   44a40:	cmp    $0x1e,%rbx
   44a44:	jae    456ef <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fff>
   44a4a:	movups 0x2c(%rdi),%xmm0
   44a4e:	movups %xmm0,-0x10(%r14,%r15,2)
   44a54:	movdqa (%rdi),%xmm0
   44a58:	movdqa 0x10(%rdi),%xmm1
   44a5d:	movdqa 0x20(%rdi),%xmm2
   44a62:	movdqa %xmm2,-0x1c(%r14,%r15,2)
   44a69:	movdqa %xmm1,-0x2c(%r14,%r15,2)
   44a70:	movdqa %xmm0,-0x3c(%r14,%r15,2)
   44a77:	mov    %r13d,(%r14,%r15,2)
   44a7b:	jmp    449cf <<c30walk::ctor2c::Ctor2c>::build::<false>+0x12df>
   44a80:	mov    0x28e8(%rsp),%rdi
   44a88:	cmp    $0x1d,%rdi
   44a8c:	ja     4577e <<c30walk::ctor2c::Ctor2c>::build::<false>+0x208e>
   44a92:	shl    $0x6,%rdi
   44a96:	add    %r15,%rdi
   44a99:	mov    0x28f0(%rsp),%eax
   44aa0:	mov    0xd0(%rsp),%rcx
   44aa8:	shl    $0x6,%r9
   44aac:	mov    %eax,%r14d
   44aaf:	shl    $0x8,%r14d
   44ab3:	or     %r12d,%r14d
   44ab6:	lea    -0x1e(%r13),%rsi
   44aba:	cmp    $0x1e,%rsi
   44abe:	jae    44b2d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x143d>
   44ac0:	xor    %esi,%esi
   44ac2:	jmp    44bb7 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x14c7>
   44ac7:	shr    $1,%r9
   44aca:	inc    %r9
   44acd:	mov    %r9,%r10
   44ad0:	and    $0xfffffffffffffff8,%r10
   44ad4:	lea    (%rdx,%r10,2),%r8
   44ad8:	movd   %ecx,%xmm0
   44adc:	pshuflw $0x0,%xmm0,%xmm0
   44ae1:	add    %r15,%r11
   44ae4:	add    $0x8,%r11
   44ae8:	xor    %ebx,%ebx
   44aea:	nopw   0x0(%rax,%rax,1)
   44af0:	movq   -0x8(%r11,%rbx,2),%xmm1
   44af7:	movq   (%r11,%rbx,2),%xmm2
   44afd:	paddw  %xmm0,%xmm1
   44b01:	paddw  %xmm0,%xmm2
   44b05:	punpcklqdq %xmm2,%xmm1
   44b09:	movdqu %xmm1,(%rax,%rbx,2)
   44b0e:	add    $0x8,%rbx
   44b12:	cmp    %rbx,%r10
   44b15:	jne    44af0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1400>
   44b17:	cmp    %r10,%r9
   44b1a:	mov    0x80(%rsp),%rbx
   44b22:	jne    44673 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xf83>
   44b28:	jmp    44699 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xfa9>
   44b2d:	mov    %rdx,%r10
   44b30:	movabs $0xffffffffffffffe,%rsi
   44b3a:	and    %rsi,%r10
   44b3d:	lea    (%rcx,%r9,1),%r11
   44b41:	add    $0x7c,%r11
   44b45:	xor    %esi,%esi
   44b47:	nopw   0x0(%rax,%rax,1)
   44b50:	movdqu 0x2c(%rdi),%xmm0
   44b55:	movdqu %xmm0,-0x50(%r11)
   44b5b:	movdqa (%rdi),%xmm1
   44b5f:	movdqa 0x10(%rdi),%xmm2
   44b64:	movaps 0x20(%rdi),%xmm3
   44b68:	movaps %xmm3,-0x5c(%r11)
   44b6d:	movdqa %xmm2,-0x6c(%r11)
   44b73:	movdqa %xmm1,-0x7c(%r11)
   44b79:	mov    %r14d,-0x40(%r11)
   44b7d:	add    $0x2,%rsi
   44b81:	movdqa %xmm1,-0x3c(%r11)
   44b87:	movdqa %xmm2,-0x2c(%r11)
   44b8d:	movaps %xmm3,-0x1c(%r11)
   44b92:	movdqu %xmm0,-0x10(%r11)
   44b98:	lea    0x2000(%r14),%ebx
   44b9f:	mov    %ebx,(%r11)
   44ba2:	add    $0x4000,%r14d
   44ba9:	sub    $0xffffffffffffff80,%r11
   44bad:	cmp    %rsi,%r10
   44bb0:	jne    44b50 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1460>
   44bb2:	test   $0x1,%dl
   44bb5:	je     44beb <<c30walk::ctor2c::Ctor2c>::build::<false>+0x14fb>
   44bb7:	add    %r9,%rcx
   44bba:	shl    $0x6,%rsi
   44bbe:	movups 0x2c(%rdi),%xmm0
   44bc2:	movups %xmm0,0x2c(%rcx,%rsi,1)
   44bc7:	movdqa (%rdi),%xmm0
   44bcb:	movdqa 0x10(%rdi),%xmm1
   44bd0:	movdqa 0x20(%rdi),%xmm2
   44bd5:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   44bdb:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   44be1:	movdqa %xmm0,(%rcx,%rsi,1)
   44be6:	mov    %r14d,0x3c(%rcx,%rsi,1)
   44beb:	mov    %edx,%ecx
   44bed:	shl    $0x5,%ecx
   44bf0:	add    %ecx,%eax
   44bf2:	mov    %eax,0x28f0(%rsp)
   44bf9:	mov    0xa0(%rsp),%rdx
   44c01:	mov    %rdx,%rax
   44c04:	shl    $0x5,%rax
   44c08:	lea    (%rdx,%rdx,1),%rcx
   44c0c:	sub    %rax,%rcx
   44c0f:	mov    0x48(%rsp),%r13
   44c14:	add    %rcx,%r13
   44c17:	lea    (%rdx,%rdx,1),%eax
   44c1a:	shl    $0x5,%edx
   44c1d:	sub    %eax,%edx
   44c1f:	add    %edx,%r8d
   44c22:	mov    0x8(%rsp),%r10
   44c27:	mov    %r8d,0x8(%r10)
   44c2b:	test   %r13,%r13
   44c2e:	mov    $0x88888889,%r14d
   44c34:	lea    -0x278fb(%rip),%r15        # 1d340 <c30walk::ctor2c::PATTERNS>
   44c3b:	mov    0x80(%rsp),%rbx
   44c43:	je     44cb0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x15c0>
   44c45:	mov    0xc(%r10),%ecx
   44c49:	mov    %r8d,%edi
   44c4c:	imul   %r14,%rdi
   44c50:	mov    %rdi,%rdx
   44c53:	shr    $0x24,%rdx
   44c57:	mov    %edx,%eax
   44c59:	shl    $0x5,%eax
   44c5c:	sub    %edx,%eax
   44c5e:	sub    %edx,%eax
   44c60:	lea    0x1e(%rax),%esi
   44c63:	cmp    %ecx,%esi
   44c65:	cmovb  %esi,%ecx
   44c68:	shr    $0x2a,%rdi
   44c6c:	mov    0xb8(%rsp),%rsi
   44c74:	cmp    %rsi,%rdi
   44c77:	jae    45761 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x2071>
   44c7d:	mov    %rdx,%rsi
   44c80:	shl    $0x6,%rsi
   44c84:	add    0xd0(%rsp),%rsi
   44c8c:	mov    0xb0(%rsp),%r9
   44c94:	mov    (%r9,%rdi,8),%rdi
   44c98:	bt     %rdx,%rdi
   44c9c:	jb     445d0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xee0>
   44ca2:	mov    0x28f4(%rsp),%edi
   44ca9:	jmp    445dc <<c30walk::ctor2c::Ctor2c>::build::<false>+0xeec>
   44cae:	xchg   %ax,%ax
   44cb0:	mov    0x68(%rsp),%rdi
   44cb5:	mov    0x40(%rsp),%rsi
   44cba:	mov    0x38(%rsp),%r12
   44cbf:	mov    0x54(%rsp),%edx
   44cc3:	mov    %rsi,%r15
   44cc6:	mov    %rdi,%r13
   44cc9:	cmp    $0xf,%rdi
   44ccd:	ja     4410b <<c30walk::ctor2c::Ctor2c>::build::<false>+0xa1b>
   44cd3:	jmp    451f3 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1b03>
   44cd8:	mov    %r9,0x10(%rsp)
   44cdd:	mov    %rdx,%r12
   44ce0:	lea    0xa8(%rsp),%rdi
   44ce8:	mov    %r14,%rsi
   44ceb:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44cf0:	test   %al,%al
   44cf2:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44cf8:	mov    0xe8(%rsp,%r14,8),%rax
   44d00:	lea    -0x10(%r13),%rdi
   44d04:	lea    0x10(%r15),%rsi
   44d08:	mov    %r12,%rdx
   44d0b:	mov    0x10(%rsp),%r9
   44d10:	jmp    441cf <<c30walk::ctor2c::Ctor2c>::build::<false>+0xadf>
   44d15:	mov    %r9,0x10(%rsp)
   44d1a:	mov    %rbx,0x18(%rsp)
   44d1f:	mov    %rdx,0x8(%rsp)
   44d24:	mov    %rsi,%r13
   44d27:	mov    %rdi,%rbx
   44d2a:	lea    0xa8(%rsp),%rdi
   44d32:	mov    %r14,%rsi
   44d35:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44d3a:	test   %al,%al
   44d3c:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44d42:	mov    0xe8(%rsp,%r14,8),%rax
   44d4a:	mov    %rbx,%rdi
   44d4d:	mov    %r13,%rsi
   44d50:	mov    0x8(%rsp),%rdx
   44d55:	mov    0x18(%rsp),%rbx
   44d5a:	mov    0x10(%rsp),%r9
   44d5f:	jmp    4420d <<c30walk::ctor2c::Ctor2c>::build::<false>+0xb1d>
   44d64:	mov    %r9,0x10(%rsp)
   44d69:	mov    %rbx,0x18(%rsp)
   44d6e:	mov    %rdx,0x8(%rsp)
   44d73:	mov    %rsi,%r13
   44d76:	mov    %rdi,%rbx
   44d79:	lea    0xa8(%rsp),%rdi
   44d81:	mov    %r14,%rsi
   44d84:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44d89:	test   %al,%al
   44d8b:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44d91:	mov    0xe8(%rsp,%r14,8),%rax
   44d99:	mov    %rbx,%rdi
   44d9c:	mov    %r13,%rsi
   44d9f:	mov    0x8(%rsp),%rdx
   44da4:	mov    0x18(%rsp),%rbx
   44da9:	mov    0x10(%rsp),%r9
   44dae:	jmp    44241 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xb51>
   44db3:	mov    %r9,0x10(%rsp)
   44db8:	mov    %rbx,0x18(%rsp)
   44dbd:	mov    %rdx,0x8(%rsp)
   44dc2:	mov    %rsi,%r13
   44dc5:	mov    %rdi,%rbx
   44dc8:	lea    0xa8(%rsp),%rdi
   44dd0:	mov    %r14,%rsi
   44dd3:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44dd8:	test   %al,%al
   44dda:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44de0:	mov    0xe8(%rsp,%r14,8),%rax
   44de8:	mov    %rbx,%rdi
   44deb:	mov    %r13,%rsi
   44dee:	mov    0x8(%rsp),%rdx
   44df3:	mov    0x18(%rsp),%rbx
   44df8:	mov    0x10(%rsp),%r9
   44dfd:	jmp    44275 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xb85>
   44e02:	mov    %r9,0x10(%rsp)
   44e07:	mov    %rbx,0x18(%rsp)
   44e0c:	mov    %rdx,0x8(%rsp)
   44e11:	mov    %rsi,%r13
   44e14:	mov    %rdi,%rbx
   44e17:	lea    0xa8(%rsp),%rdi
   44e1f:	mov    %r14,%rsi
   44e22:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44e27:	test   %al,%al
   44e29:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44e2f:	mov    0xe8(%rsp,%r14,8),%rax
   44e37:	mov    %rbx,%rdi
   44e3a:	mov    %r13,%rsi
   44e3d:	mov    0x8(%rsp),%rdx
   44e42:	mov    0x18(%rsp),%rbx
   44e47:	mov    0x10(%rsp),%r9
   44e4c:	jmp    442a9 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xbb9>
   44e51:	mov    %rdx,%r13
   44e54:	movzbl 0xf(%r15),%r14d
   44e59:	mov    0xe8(%rsp,%r14,8),%rax
   44e61:	cmp    0x8e8(%rsp,%r14,8),%rax
   44e69:	je     451ab <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1abb>
   44e6f:	lea    0xf(%rbp),%ecx
   44e72:	mov    %cx,(%rax)
   44e75:	addq   $0x2,0xe8(%rsp,%r14,8)
   44e7e:	cmp    $0xf,%rbx
   44e82:	ja     440c0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x9d0>
   44e88:	lea    0x10(%r12),%rbx
   44e8d:	mov    %r13,%rdx
   44e90:	jmp    444d5 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xde5>
   44e95:	mov    %r9,0x10(%rsp)
   44e9a:	mov    %rbx,0x18(%rsp)
   44e9f:	mov    %rdx,0x8(%rsp)
   44ea4:	mov    %rsi,%r13
   44ea7:	mov    %rdi,%rbx
   44eaa:	lea    0xa8(%rsp),%rdi
   44eb2:	mov    %r14,%rsi
   44eb5:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44eba:	test   %al,%al
   44ebc:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44ec2:	mov    0xe8(%rsp,%r14,8),%rax
   44eca:	mov    %rbx,%rdi
   44ecd:	mov    %r13,%rsi
   44ed0:	mov    0x8(%rsp),%rdx
   44ed5:	mov    0x18(%rsp),%rbx
   44eda:	mov    0x10(%rsp),%r9
   44edf:	jmp    442dd <<c30walk::ctor2c::Ctor2c>::build::<false>+0xbed>
   44ee4:	mov    %r9,0x10(%rsp)
   44ee9:	mov    %rbx,0x18(%rsp)
   44eee:	mov    %rdx,0x8(%rsp)
   44ef3:	mov    %rsi,%r13
   44ef6:	mov    %rdi,%rbx
   44ef9:	lea    0xa8(%rsp),%rdi
   44f01:	mov    %r14,%rsi
   44f04:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44f09:	test   %al,%al
   44f0b:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44f11:	mov    0xe8(%rsp,%r14,8),%rax
   44f19:	mov    %rbx,%rdi
   44f1c:	mov    %r13,%rsi
   44f1f:	mov    0x8(%rsp),%rdx
   44f24:	mov    0x18(%rsp),%rbx
   44f29:	mov    0x10(%rsp),%r9
   44f2e:	jmp    44311 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xc21>
   44f33:	mov    %r9,0x10(%rsp)
   44f38:	mov    %rbx,0x18(%rsp)
   44f3d:	mov    %rdx,0x8(%rsp)
   44f42:	mov    %rsi,%r13
   44f45:	mov    %rdi,%rbx
   44f48:	lea    0xa8(%rsp),%rdi
   44f50:	mov    %r14,%rsi
   44f53:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44f58:	test   %al,%al
   44f5a:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44f60:	mov    0xe8(%rsp,%r14,8),%rax
   44f68:	mov    %rbx,%rdi
   44f6b:	mov    %r13,%rsi
   44f6e:	mov    0x8(%rsp),%rdx
   44f73:	mov    0x18(%rsp),%rbx
   44f78:	mov    0x10(%rsp),%r9
   44f7d:	jmp    44345 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xc55>
   44f82:	mov    %r9,0x10(%rsp)
   44f87:	mov    %rbx,0x18(%rsp)
   44f8c:	mov    %rdx,0x8(%rsp)
   44f91:	mov    %rsi,%r13
   44f94:	mov    %rdi,%rbx
   44f97:	lea    0xa8(%rsp),%rdi
   44f9f:	mov    %r14,%rsi
   44fa2:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44fa7:	test   %al,%al
   44fa9:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44faf:	mov    0xe8(%rsp,%r14,8),%rax
   44fb7:	mov    %rbx,%rdi
   44fba:	mov    %r13,%rsi
   44fbd:	mov    0x8(%rsp),%rdx
   44fc2:	mov    0x18(%rsp),%rbx
   44fc7:	mov    0x10(%rsp),%r9
   44fcc:	jmp    44379 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xc89>
   44fd1:	mov    %r9,0x10(%rsp)
   44fd6:	mov    %rbx,0x18(%rsp)
   44fdb:	mov    %rdx,0x8(%rsp)
   44fe0:	mov    %rsi,%r13
   44fe3:	mov    %rdi,%rbx
   44fe6:	lea    0xa8(%rsp),%rdi
   44fee:	mov    %r14,%rsi
   44ff1:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   44ff6:	test   %al,%al
   44ff8:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   44ffe:	mov    0xe8(%rsp,%r14,8),%rax
   45006:	mov    %rbx,%rdi
   45009:	mov    %r13,%rsi
   4500c:	mov    0x8(%rsp),%rdx
   45011:	mov    0x18(%rsp),%rbx
   45016:	mov    0x10(%rsp),%r9
   4501b:	jmp    443ad <<c30walk::ctor2c::Ctor2c>::build::<false>+0xcbd>
   45020:	mov    %r9,0x10(%rsp)
   45025:	mov    %rbx,0x18(%rsp)
   4502a:	mov    %rdx,0x8(%rsp)
   4502f:	mov    %rsi,%r13
   45032:	mov    %rdi,%rbx
   45035:	lea    0xa8(%rsp),%rdi
   4503d:	mov    %r14,%rsi
   45040:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   45045:	test   %al,%al
   45047:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   4504d:	mov    0xe8(%rsp,%r14,8),%rax
   45055:	mov    %rbx,%rdi
   45058:	mov    %r13,%rsi
   4505b:	mov    0x8(%rsp),%rdx
   45060:	mov    0x18(%rsp),%rbx
   45065:	mov    0x10(%rsp),%r9
   4506a:	jmp    443e1 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xcf1>
   4506f:	mov    %r9,0x10(%rsp)
   45074:	mov    %rbx,0x18(%rsp)
   45079:	mov    %rdx,0x8(%rsp)
   4507e:	mov    %rsi,%r13
   45081:	mov    %rdi,%rbx
   45084:	lea    0xa8(%rsp),%rdi
   4508c:	mov    %r14,%rsi
   4508f:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   45094:	test   %al,%al
   45096:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   4509c:	mov    0xe8(%rsp,%r14,8),%rax
   450a4:	mov    %rbx,%rdi
   450a7:	mov    %r13,%rsi
   450aa:	mov    0x8(%rsp),%rdx
   450af:	mov    0x18(%rsp),%rbx
   450b4:	mov    0x10(%rsp),%r9
   450b9:	jmp    44415 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xd25>
   450be:	mov    %r9,0x10(%rsp)
   450c3:	mov    %rbx,0x18(%rsp)
   450c8:	mov    %rdx,0x8(%rsp)
   450cd:	mov    %rsi,%r13
   450d0:	mov    %rdi,%rbx
   450d3:	lea    0xa8(%rsp),%rdi
   450db:	mov    %r14,%rsi
   450de:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   450e3:	test   %al,%al
   450e5:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   450eb:	mov    0xe8(%rsp,%r14,8),%rax
   450f3:	mov    %rbx,%rdi
   450f6:	mov    %r13,%rsi
   450f9:	mov    0x8(%rsp),%rdx
   450fe:	mov    0x18(%rsp),%rbx
   45103:	mov    0x10(%rsp),%r9
   45108:	jmp    44449 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xd59>
   4510d:	mov    %r9,0x10(%rsp)
   45112:	mov    %rbx,0x18(%rsp)
   45117:	mov    %rdx,0x8(%rsp)
   4511c:	mov    %rsi,%r13
   4511f:	mov    %rdi,%rbx
   45122:	lea    0xa8(%rsp),%rdi
   4512a:	mov    %r14,%rsi
   4512d:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   45132:	test   %al,%al
   45134:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   4513a:	mov    0xe8(%rsp,%r14,8),%rax
   45142:	mov    %rbx,%rdi
   45145:	mov    %r13,%rsi
   45148:	mov    0x8(%rsp),%rdx
   4514d:	mov    0x18(%rsp),%rbx
   45152:	mov    0x10(%rsp),%r9
   45157:	jmp    44479 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xd89>
   4515c:	mov    %r9,0x10(%rsp)
   45161:	mov    %rbx,0x18(%rsp)
   45166:	mov    %rdx,0x8(%rsp)
   4516b:	mov    %rsi,%r13
   4516e:	mov    %rdi,%rbx
   45171:	lea    0xa8(%rsp),%rdi
   45179:	mov    %r14,%rsi
   4517c:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   45181:	test   %al,%al
   45183:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   45189:	mov    0xe8(%rsp,%r14,8),%rax
   45191:	mov    %rbx,%rdi
   45194:	mov    %r13,%rsi
   45197:	mov    0x8(%rsp),%rdx
   4519c:	mov    0x18(%rsp),%rbx
   451a1:	mov    0x10(%rsp),%r9
   451a6:	jmp    444a9 <<c30walk::ctor2c::Ctor2c>::build::<false>+0xdb9>
   451ab:	mov    %rbx,0x18(%rsp)
   451b0:	mov    %rsi,%r15
   451b3:	mov    %rdi,%rbx
   451b6:	lea    0xa8(%rsp),%rdi
   451be:	mov    %r14,%rsi
   451c1:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   451c6:	test   %al,%al
   451c8:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   451ce:	mov    0xe8(%rsp,%r14,8),%rax
   451d6:	mov    %rbx,%rdi
   451d9:	mov    %r15,%rsi
   451dc:	mov    0x18(%rsp),%rbx
   451e1:	jmp    44e6f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x177f>
   451e6:	cs nopw 0x0(%rax,%rax,1)
   451f0:	mov    %r13,%rdi
   451f3:	test   %rdi,%rdi
   451f6:	je     43ec4 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x7d4>
   451fc:	mov    0x30(%rsp),%r13
   45201:	sub    %rdi,%r13
   45204:	mov    0x28e8(%rsp),%rax
   4520c:	mov    $0x1e,%r15d
   45212:	sub    %rax,%r15
   45215:	cmp    %rdi,%r15
   45218:	cmovae %rdi,%r15
   4521c:	mov    %r15,%r14
   4521f:	add    %r13,%r14
   45222:	jb     456bf <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fcf>
   45228:	cmp    0x20(%rsp),%r14
   4522d:	ja     456bf <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fcf>
   45233:	mov    %r12,0x38(%rsp)
   45238:	mov    0x28f0(%rsp),%r12d
   45240:	cmp    $0x1e,%rax
   45244:	jne    452da <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1bea>
   4524a:	cmp    %r14,0x30(%rsp)
   4524f:	jb     456d6 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1fe6>
   45255:	jne    45351 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c61>
   4525b:	mov    0x28e8(%rsp),%rax
   45263:	mov    0x28f0(%rsp),%ecx
   4526a:	cmp    $0x1,%rdi
   4526e:	je     43e95 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x7a5>
   45274:	mov    %rdi,%r8
   45277:	mov    %edi,%edx
   45279:	and    $0xe,%edx
   4527c:	mov    0x38(%rsp),%r12
   45281:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   45290:	lea    0x1(%rax),%rsi
   45294:	add    $0xffffffffffffffe3,%rax
   45298:	xor    %edi,%edi
   4529a:	cmp    $0x1e,%rsi
   4529e:	setae  %dil
   452a2:	cmovb  %rsi,%rax
   452a6:	lea    (%rcx,%rdi,2),%ecx
   452a9:	lea    0x1(%rax),%rsi
   452ad:	add    $0xffffffffffffffe3,%rax
   452b1:	xor    %edi,%edi
   452b3:	cmp    $0x1e,%rsi
   452b7:	setae  %dil
   452bb:	cmovb  %rsi,%rax
   452bf:	lea    (%rcx,%rdi,2),%ecx
   452c2:	add    $0x2,%ecx
   452c5:	add    $0xfffffffffffffffe,%rdx
   452c9:	jne    45290 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1ba0>
   452cb:	test   $0x1,%r8b
   452cf:	jne    43e9a <<c30walk::ctor2c::Ctor2c>::build::<false>+0x7aa>
   452d5:	jmp    43eb5 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x7c5>
   452da:	add    0x60(%rsp),%r13
   452df:	xor    %ebp,%ebp
   452e1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   452f0:	movzbl 0x0(%r13,%rbp,1),%ebx
   452f6:	mov    0xe8(%rsp,%rbx,8),%rax
   452fe:	cmp    0x8e8(%rsp,%rbx,8),%rax
   45306:	je     45325 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c35>
   45308:	lea    (%r12,%rbp,1),%rcx
   4530c:	mov    %cx,(%rax)
   4530f:	addq   $0x2,0xe8(%rsp,%rbx,8)
   45318:	inc    %rbp
   4531b:	cmp    %rbp,%r15
   4531e:	jne    452f0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c00>
   45320:	jmp    4524a <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1b5a>
   45325:	mov    %rdi,0x68(%rsp)
   4532a:	lea    0xa8(%rsp),%rdi
   45332:	mov    %rbx,%rsi
   45335:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   4533a:	test   %al,%al
   4533c:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   45342:	mov    0xe8(%rsp,%rbx,8),%rax
   4534a:	mov    0x68(%rsp),%rdi
   4534f:	jmp    45308 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c18>
   45351:	mov    0x60(%rsp),%rax
   45356:	mov    0x30(%rsp),%rcx
   4535b:	lea    (%rax,%rcx,1),%r14
   4535f:	lea    (%r12,%r15,1),%ebp
   45363:	add    $0x2,%ebp
   45366:	sub    %rdi,%r15
   45369:	nopl   0x0(%rax)
   45370:	movzbl (%r14,%r15,1),%ebx
   45375:	mov    0xe8(%rsp,%rbx,8),%rax
   4537d:	cmp    0x8e8(%rsp,%rbx,8),%rax
   45385:	je     4539f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1caf>
   45387:	mov    %bp,(%rax)
   4538a:	addq   $0x2,0xe8(%rsp,%rbx,8)
   45393:	inc    %ebp
   45395:	inc    %r15
   45398:	jne    45370 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c80>
   4539a:	jmp    4525b <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1b6b>
   4539f:	mov    %rdi,%r12
   453a2:	lea    0xa8(%rsp),%rdi
   453aa:	mov    %rbx,%rsi
   453ad:	call   8ae90 <<c30walk::ctor2c::W<false>>::refill>
   453b2:	test   %al,%al
   453b4:	je     4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   453b6:	mov    0xe8(%rsp,%rbx,8),%rax
   453be:	mov    %r12,%rdi
   453c1:	jmp    45387 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1c97>
   453c3:	mov    %rbx,%rdi
   453c6:	call   f56c0 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   453cb:	jmp    44047 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x957>
   453d0:	mov    $0x9,%eax
   453d5:	lea    0x110c(%rsp),%rcx
   453dd:	nopl   (%rax)
   453e0:	cmp    $0x109,%rax
   453e6:	je     45454 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d64>
   453e8:	mov    0xa0(%rsp,%rax,8),%rdx
   453f0:	sub    -0x24(%rcx),%rdx
   453f4:	shr    $1,%rdx
   453f7:	add    -0x1c(%rcx),%edx
   453fa:	cmp    -0x18(%rcx),%edx
   453fd:	jne    4541d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d2d>
   453ff:	mov    0xa8(%rsp,%rax,8),%rdx
   45407:	sub    -0xc(%rcx),%rdx
   4540b:	shr    $1,%rdx
   4540e:	add    -0x4(%rcx),%edx
   45411:	add    $0x2,%rax
   45415:	cmp    (%rcx),%edx
   45417:	lea    0x30(%rcx),%rcx
   4541b:	je     453e0 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1cf0>
   4541d:	mov    0x4530(%rsp),%rax
   45425:	movq   $0x0,0x10(%rax)
   4542d:	xor    %eax,%eax
   4542f:	add    $0x44f8,%rsp
   45436:	pop    %rbx
   45437:	pop    %r12
   45439:	pop    %r13
   4543b:	pop    %r14
   4543d:	pop    %r15
   4543f:	pop    %rbp
   45440:	ret
   45441:	lea    0xf98a0(%rip),%rcx        # 13ece8 <__frame_dummy_init_array_entry+0x1808>
   45448:	mov    $0x1e,%edx
   4544d:	xor    %edi,%edi
   4544f:	call   e78a0 <core::slice::index::slice_index_fail>
   45454:	mov    $0x8,%r14d
   4545a:	mov    $0x1040,%r15d
   45460:	lea    0xa8(%rsp),%rbx
   45468:	mov    0x90(%rsp),%r12
   45470:	mov    0x4530(%rsp),%rsi
   45478:	mov    0x88(%rsp),%r13
   45480:	jmp    454d2 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1de2>
   45482:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   45490:	sub    %rcx,%rax
   45493:	shr    $1,%rax
   45496:	add    %eax,0xb0(%rsp,%r15,1)
   4549e:	movq   $0x0,0xa8(%rsp,%r15,1)
   454aa:	movq   $0x0,0xa8(%rsp,%r14,8)
   454b6:	movq   $0x0,0x8a8(%rsp,%r14,8)
   454c2:	inc    %r14
   454c5:	add    $0x18,%r15
   454c9:	cmp    $0x108,%r14
   454d0:	je     4550d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1e1d>
   454d2:	mov    0xa8(%rsp,%r14,8),%rcx
   454da:	mov    %rcx,%rax
   454dd:	cmp    0xa8(%rsp,%r15,1),%rcx
   454e5:	je     45490 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1da0>
   454e7:	lea    -0x8(%r14),%esi
   454eb:	mov    %rbx,%rdi
   454ee:	call   8adc0 <<c30walk::ctor2c::W<false>>::finish>
   454f3:	mov    0x4530(%rsp),%rsi
   454fb:	mov    0xa8(%rsp,%r14,8),%rax
   45503:	mov    0xa8(%rsp,%r15,1),%rcx
   4550b:	jmp    45490 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1da0>
   4550d:	xor    %ecx,%ecx
   4550f:	mov    0x8(%r12),%rbx
   45514:	mov    0x58(%rsp),%r14
   45519:	test   %r14,%r14
   4551c:	setne  %al
   4551f:	mov    0x20(%rsp),%rdx
   45524:	je     45570 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1e80>
   45526:	mov    %al,%cl
   45528:	mov    %r14,%rax
   4552b:	sub    %rcx,%rax
   4552e:	imul   $0xffffffffffffffe2,%rax,%rcx
   45532:	add    %rdx,%rcx
   45535:	cmp    $0x1e,%rcx
   45539:	jae    45570 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1e80>
   4553b:	lea    (%rbx,%r14,4),%rcx
   4553f:	lea    (%rcx,%rdx,2),%rdi
   45543:	add    $0xfffffffffffffffc,%rdi
   45547:	lea    (%rax,%rax,1),%rcx
   4554b:	shl    $0x5,%rax
   4554f:	sub    %rcx,%rax
   45552:	sub    %rdx,%rax
   45555:	lea    0x3c(,%rax,2),%rdx
   4555d:	mov    $0xff,%esi
   45562:	call   *0xfe9d8(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   45568:	mov    0x4530(%rsp),%rsi
   45570:	shl    $0x6,%r14
   45574:	pcmpeqd %xmm0,%xmm0
   45578:	movdqu %xmm0,0x2c(%rbx,%r14,1)
   4557f:	movdqa %xmm0,0x20(%rbx,%r14,1)
   45586:	movdqa %xmm0,0x10(%rbx,%r14,1)
   4558d:	movdqa %xmm0,(%rbx,%r14,1)
   45593:	movl   $0x100fffff,0x3c(%rbx,%r14,1)
   4559c:	mov    0x8(%r13),%rdx
   455a0:	mov    0x10(%r13),%rcx
   455a4:	mov    0x8(%rsi),%r8
   455a8:	mov    0x10(%rsi),%r9
   455ac:	lea    0xa8(%rsp),%rdi
   455b4:	mov    %rbx,%rsi
   455b7:	call   a2470 <c30walk::ctor::canonicalise>
   455bc:	mov    0x78(%rsp),%rax
   455c1:	mov    %rax,0x10(%r12)
   455c6:	mov    $0x1,%al
   455c8:	jmp    4542f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1d3f>
   455cd:	test   %rax,%rax
   455d0:	je     455e2 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1ef2>
   455d2:	mov    0x8(%r9),%rdi
   455d6:	mov    %r9,%r15
   455d9:	call   *0xfe971(%rip)        # 143f50 <free@GLIBC_2.2.5>
   455df:	mov    %r15,%r9
   455e2:	movq   $0x0,(%r9)
   455e9:	movq   $0x40,0x8(%r9)
   455f1:	movq   $0x0,0x10(%r9)
   455f9:	mov    $0x40,%edx
   455fe:	mov    %r9,%rdi
   45601:	mov    0x78(%rsp),%rsi
   45606:	mov    %r9,%r15
   45609:	call   95360 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4560e:	mov    %r15,%r9
   45611:	cmp    $0xffffffffffffffff,%rax
   45615:	mov    0x4530(%rsp),%rdi
   4561d:	mov    0x58(%rsp),%r8
   45622:	je     43904 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x214>
   45628:	jmp    4567a <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f8a>
   4562a:	mov    %r9,%r15
   4562d:	test   %rax,%rax
   45630:	je     45644 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x1f54>
   45632:	mov    0x8(%rdi),%rdi
   45636:	call   *0xfe914(%rip)        # 143f50 <free@GLIBC_2.2.5>
   4563c:	mov    0x4530(%rsp),%rdi
   45644:	movq   $0x0,(%rdi)
   4564b:	movq   $0x40,0x8(%rdi)
   45653:	movq   $0x0,0x10(%rdi)
   4565b:	mov    $0x80,%edx
   45660:	mov    %r14,%rsi
   45663:	call   95360 <<alloc::raw_vec::RawVecInner>::grow_exact>
   45668:	cmp    $0xffffffffffffffff,%rax
   4566c:	mov    %r15,%r9
   4566f:	mov    0x58(%rsp),%r8
   45674:	je     43910 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x220>
   4567a:	mov    %rax,%rdi
   4567d:	mov    %rdx,%rsi
   45680:	call   e502b <alloc::raw_vec::handle_error>
   45685:	lea    0xf80e4(%rip),%rdx        # 13d770 <__frame_dummy_init_array_entry+0x290>
   4568c:	mov    %rax,%rdi
   4568f:	mov    0x10(%rsp),%rsi
   45694:	call   e6ac3 <core::panicking::panic_bounds_check>
   45699:	lea    0xf96d8(%rip),%rdx        # 13ed78 <__frame_dummy_init_array_entry+0x1898>
   456a0:	mov    %rax,%rdi
   456a3:	mov    %rcx,%rsi
   456a6:	call   e6ac3 <core::panicking::panic_bounds_check>
   456ab:	lea    0xf96ae(%rip),%rdx        # 13ed60 <__frame_dummy_init_array_entry+0x1880>
   456b2:	mov    $0x1e,%esi
   456b7:	mov    %rbx,%rdi
   456ba:	call   e6ac3 <core::panicking::panic_bounds_check>
   456bf:	lea    0xf9652(%rip),%rcx        # 13ed18 <__frame_dummy_init_array_entry+0x1838>
   456c6:	mov    %r13,%rdi
   456c9:	mov    %r14,%rsi
   456cc:	mov    0x20(%rsp),%rdx
   456d1:	call   e78a0 <core::slice::index::slice_index_fail>
   456d6:	lea    0xf9623(%rip),%rcx        # 13ed00 <__frame_dummy_init_array_entry+0x1820>
   456dd:	mov    %r14,%rdi
   456e0:	mov    0x30(%rsp),%rsi
   456e5:	mov    0x20(%rsp),%rdx
   456ea:	call   e78a0 <core::slice::index::slice_index_fail>
   456ef:	lea    0xf9652(%rip),%rdx        # 13ed48 <__frame_dummy_init_array_entry+0x1868>
   456f6:	mov    $0x1e,%esi
   456fb:	mov    %rbx,%rdi
   456fe:	call   e6ac3 <core::panicking::panic_bounds_check>
   45703:	mov    $0x10,%edx
   45708:	mov    $0x4,%ecx
   4570d:	mov    $0x400,%r8d
   45713:	mov    %r12,%rdi
   45716:	xor    %esi,%esi
   45718:	mov    %r9,%r14
   4571b:	call   6aea0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   45720:	mov    %r14,%r9
   45723:	jmp    4398f <<c30walk::ctor2c::Ctor2c>::build::<false>+0x29f>
   45728:	mov    $0x8,%ecx
   4572d:	mov    $0x8,%r8d
   45733:	mov    %rbp,%rdi
   45736:	xor    %esi,%esi
   45738:	mov    %r14,%rdx
   4573b:	mov    %r9,%r15
   4573e:	call   6aea0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   45743:	mov    %r15,%r9
   45746:	mov    0x10(%rbp),%r15
   4574a:	mov    0x8(%rbp),%r12
   4574e:	lea    (%r12,%r15,8),%rdi
   45752:	cmp    $0x1,%r14
   45756:	jne    4393d <<c30walk::ctor2c::Ctor2c>::build::<false>+0x24d>
   4575c:	jmp    43965 <<c30walk::ctor2c::Ctor2c>::build::<false>+0x275>
   45761:	lea    0xf8008(%rip),%rdx        # 13d770 <__frame_dummy_init_array_entry+0x290>
   45768:	call   e6ac3 <core::panicking::panic_bounds_check>
   4576d:	lea    0xf955c(%rip),%rdx        # 13ecd0 <__frame_dummy_init_array_entry+0x17f0>
   45774:	mov    $0x1e,%esi
   45779:	call   e6ac3 <core::panicking::panic_bounds_check>
   4577e:	lea    0xf95ab(%rip),%rdx        # 13ed30 <__frame_dummy_init_array_entry+0x1850>
   45785:	mov    $0x1e,%esi
   4578a:	call   e6ac3 <core::panicking::panic_bounds_check>
   4578f:	lea    0xf9492(%rip),%rdx        # 13ec28 <__frame_dummy_init_array_entry+0x1748>
   45796:	mov    0x10(%rsp),%rsi
   4579b:	call   e6ac3 <core::panicking::panic_bounds_check>
   457a0:	lea    0xf7fc9(%rip),%rdx        # 13d770 <__frame_dummy_init_array_entry+0x290>
   457a7:	mov    %r15,%rdi
   457aa:	call   e6ac3 <core::panicking::panic_bounds_check>
