# <c30walk::ctor2su::Ctor2su>::build::<false>
# size 0x1834

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000478a0 <<c30walk::ctor2su::Ctor2su>::build::<false>>:
   478a0:	push   %rbp
   478a1:	push   %r15
   478a3:	push   %r14
   478a5:	push   %r13
   478a7:	push   %r12
   478a9:	push   %rbx
   478aa:	sub    $0x1000,%rsp
   478b1:	movq   $0x0,(%rsp)
   478b9:	sub    $0x1000,%rsp
   478c0:	movq   $0x0,(%rsp)
   478c8:	sub    $0x1000,%rsp
   478cf:	movq   $0x0,(%rsp)
   478d7:	sub    $0x1000,%rsp
   478de:	movq   $0x0,(%rsp)
   478e6:	sub    $0x1000,%rsp
   478ed:	movq   $0x0,(%rsp)
   478f5:	sub    $0x4f8,%rsp
   478fc:	mov    %r8d,0x14(%rsp)
   47901:	mov    %rsi,0x50(%rsp)
   47906:	mov    0x5530(%rsp),%rax
   4790e:	movq   $0x0,0x10(%r9)
   47916:	movq   $0x0,0x10(%rax)
   4791e:	mov    %rdx,0x8(%rsp)
   47923:	cmp    $0xeffe2,%rdx
   4792a:	ja     48d8d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ed>
   47930:	mov    %rcx,%r14
   47933:	pxor   %xmm0,%xmm0
   47937:	xor    %eax,%eax
   47939:	pxor   %xmm2,%xmm2
   4793d:	pxor   %xmm1,%xmm1
   47941:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47950:	movq   (%r14,%rax,4),%xmm3
   47956:	movq   0x8(%r14,%rax,4),%xmm4
   4795d:	punpckldq %xmm0,%xmm3
   47961:	paddq  %xmm2,%xmm3
   47965:	punpckldq %xmm0,%xmm4
   47969:	paddq  %xmm1,%xmm4
   4796d:	movq   0x10(%r14,%rax,4),%xmm2
   47974:	movq   0x18(%r14,%rax,4),%xmm1
   4797b:	punpckldq %xmm0,%xmm2
   4797f:	paddq  %xmm3,%xmm2
   47983:	punpckldq %xmm0,%xmm1
   47987:	paddq  %xmm4,%xmm1
   4798b:	add    $0x8,%rax
   4798f:	cmp    $0x100,%rax
   47995:	jne    47950 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb0>
   47997:	paddq  %xmm2,%xmm1
   4799b:	pshufd $0xee,%xmm1,%xmm0
   479a0:	paddq  %xmm1,%xmm0
   479a4:	movq   %xmm0,%rax
   479a9:	cmp    0x8(%rsp),%rax
   479ae:	jne    48d8d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ed>
   479b4:	pxor   %xmm2,%xmm2
   479b8:	xor    %eax,%eax
   479ba:	movdqa -0x36d62(%rip),%xmm3        # 10c60 <GCC_except_table1006+0xa18>
   479c2:	pxor   %xmm1,%xmm1
   479c6:	pxor   %xmm0,%xmm0
   479ca:	nopw   0x0(%rax,%rax,1)
   479d0:	movq   (%r14,%rax,4),%xmm4
   479d6:	movq   0x8(%r14,%rax,4),%xmm5
   479dd:	pshufd $0xd4,%xmm4,%xmm4
   479e2:	pcmpeqd %xmm2,%xmm4
   479e6:	pandn  %xmm3,%xmm4
   479ea:	paddq  %xmm1,%xmm4
   479ee:	pshufd $0xd4,%xmm5,%xmm5
   479f3:	pcmpeqd %xmm2,%xmm5
   479f7:	pandn  %xmm3,%xmm5
   479fb:	paddq  %xmm0,%xmm5
   479ff:	movq   0x10(%r14,%rax,4),%xmm0
   47a06:	movq   0x18(%r14,%rax,4),%xmm6
   47a0d:	pshufd $0xd4,%xmm0,%xmm1
   47a12:	pcmpeqd %xmm2,%xmm1
   47a16:	pandn  %xmm3,%xmm1
   47a1a:	paddq  %xmm4,%xmm1
   47a1e:	pshufd $0xd4,%xmm6,%xmm0
   47a23:	pcmpeqd %xmm2,%xmm0
   47a27:	pandn  %xmm3,%xmm0
   47a2b:	paddq  %xmm5,%xmm0
   47a2f:	add    $0x8,%rax
   47a33:	cmp    $0x100,%rax
   47a39:	jne    479d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x130>
   47a3b:	mov    $0x88888889,%r13d
   47a41:	mov    0x8(%rsp),%rax
   47a46:	imul   %rax,%r13
   47a4a:	shr    $0x24,%r13
   47a4e:	imul   $0xeeeeeeef,%eax,%eax
   47a54:	ror    $1,%eax
   47a56:	cmp    $0x8888889,%eax
   47a5b:	sbb    $0xffffffffffffffff,%r13
   47a5f:	mov    %r13,%rbx
   47a62:	cmpl   $0x0,0x14(%rsp)
   47a67:	jne    47ac0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x220>
   47a69:	mov    0x8(%rsp),%rsi
   47a6e:	test   %rsi,%rsi
   47a71:	je     47abe <<c30walk::ctor2su::Ctor2su>::build::<false>+0x21e>
   47a73:	paddq  %xmm1,%xmm0
   47a77:	pshufd $0xee,%xmm0,%xmm1
   47a7c:	paddq  %xmm0,%xmm1
   47a80:	movq   %xmm1,%rax
   47a85:	lea    -0x1(%rsi),%ecx
   47a88:	mov    $0x88888889,%edx
   47a8d:	imul   %rcx,%rdx
   47a91:	shr    $0x23,%rdx
   47a95:	and    $0xfffffffe,%edx
   47a98:	lea    (%rdx,%rsi,1),%ecx
   47a9b:	dec    %ecx
   47a9d:	shr    $1,%ecx
   47a9f:	mov    $0x80010003,%ebx
   47aa4:	imul   %rcx,%rbx
   47aa8:	shr    $0x2e,%rbx
   47aac:	inc    %ebx
   47aae:	imul   %rax,%rbx
   47ab2:	dec    %rbx
   47ab5:	cmp    %r13,%rbx
   47ab8:	cmovae %r13,%rbx
   47abc:	jmp    47ac0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x220>
   47abe:	xor    %ebx,%ebx
   47ac0:	mov    (%r9),%rax
   47ac3:	lea    0x1(%r13),%rcx
   47ac7:	mov    %rcx,0x90(%rsp)
   47acf:	cmp    %r13,%rax
   47ad2:	jbe    48ef4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1654>
   47ad8:	mov    0x5530(%rsp),%rax
   47ae0:	mov    (%rax),%rax
   47ae3:	cmp    %rbx,%rax
   47ae6:	jb     48f49 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16a9>
   47aec:	movq   $0x0,0x10(%rdi)
   47af4:	lea    0x3f(%r13),%rbx
   47af8:	shr    $0x6,%rbx
   47afc:	mov    %rdi,0x70(%rsp)
   47b01:	mov    %r13,0x78(%rsp)
   47b06:	je     47b5b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2bb>
   47b08:	cmp    (%rdi),%rbx
   47b0b:	ja     49052 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17b2>
   47b11:	mov    %r9,%r13
   47b14:	xor    %r15d,%r15d
   47b17:	mov    0x8(%rdi),%r12
   47b1b:	lea    (%r12,%r15,8),%rdi
   47b1f:	cmp    $0x1,%rbx
   47b23:	je     47b47 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   47b25:	lea    -0x8(,%rbx,8),%rdx
   47b2d:	xor    %esi,%esi
   47b2f:	call   *0x10653b(%rip)        # 14e070 <memset@GLIBC_2.2.5>
   47b35:	lea    (%r15,%rbx,1),%rax
   47b39:	lea    (%r12,%rax,8),%rdi
   47b3d:	add    $0xfffffffffffffff8,%rdi
   47b41:	add    %rbx,%r15
   47b44:	dec    %r15
   47b47:	mov    %r13,%r9
   47b4a:	movq   $0x0,(%rdi)
   47b51:	inc    %r15
   47b54:	mov    0x70(%rsp),%rdi
   47b59:	jmp    47b5e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2be>
   47b5b:	xor    %r15d,%r15d
   47b5e:	lea    0x18(%rdi),%r13
   47b62:	mov    %r15,0x10(%rdi)
   47b66:	movq   $0x0,0x28(%rdi)
   47b6e:	cmpq   $0xf,0x18(%rdi)
   47b73:	jbe    49027 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1787>
   47b79:	lea    0x30(%rdi),%r15
   47b7d:	xor    %eax,%eax
   47b7f:	pxor   %xmm0,%xmm0
   47b83:	movabs $0xffffffff00000000,%rcx
   47b8d:	nopl   (%rax)
   47b90:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   47b99:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   47ba2:	mov    %rcx,0x2d18(%rsp,%rax,1)
   47baa:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   47bb3:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   47bbc:	mov    %rcx,0x2d40(%rsp,%rax,1)
   47bc4:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   47bcd:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   47bd6:	mov    %rcx,0x2d68(%rsp,%rax,1)
   47bde:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   47be7:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   47bf0:	mov    %rcx,0x2d90(%rsp,%rax,1)
   47bf8:	add    $0xa0,%rax
   47bfe:	cmp    $0x2800,%rax
   47c04:	jne    47b90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2f0>
   47c06:	lea    0xc(%r14),%rax
   47c0a:	xor    %edx,%edx
   47c0c:	mov    $0x25,%ecx
   47c11:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47c20:	mov    -0xc(%rax),%esi
   47c23:	add    %edx,%esi
   47c25:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   47c2c:	mov    %esi,0x2c80(%rsp,%rcx,4)
   47c33:	mov    -0x8(%rax),%edx
   47c36:	add    %esi,%edx
   47c38:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   47c3f:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   47c46:	mov    -0x4(%rax),%esi
   47c49:	add    %edx,%esi
   47c4b:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   47c52:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   47c59:	mov    (%rax),%edx
   47c5b:	add    %esi,%edx
   47c5d:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   47c64:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   47c6b:	add    $0x28,%rcx
   47c6f:	add    $0x10,%rax
   47c73:	cmp    $0xa25,%rcx
   47c7a:	jne    47c20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x380>
   47c7c:	mov    %r9,0x88(%rsp)
   47c84:	mov    0x8(%r9),%rbx
   47c88:	mov    0x8(%rdi),%rbp
   47c8c:	mov    0x10(%rdi),%r12
   47c90:	lea    0xe8(%rsp),%rdi
   47c98:	lea    0x2cf8(%rsp),%rsi
   47ca0:	mov    $0x2800,%edx
   47ca5:	call   *0x1063e5(%rip)        # 14e090 <memcpy@GLIBC_2.14>
   47cab:	mov    %rbp,%r9
   47cae:	mov    %r12,%rsi
   47cb1:	mov    %rbx,0xd0(%rsp)
   47cb9:	mov    0x78(%rsp),%rax
   47cbe:	mov    %rax,0xd8(%rsp)
   47cc6:	mov    0x8(%rsp),%r11
   47ccb:	mov    %r11,0xe0(%rsp)
   47cd3:	mov    0x5530(%rsp),%rax
   47cdb:	mov    %rax,0xa8(%rsp)
   47ce3:	mov    %rbp,0xb0(%rsp)
   47ceb:	mov    %r12,0xb8(%rsp)
   47cf3:	mov    %r13,0xc0(%rsp)
   47cfb:	movl   $0x0,0x28f0(%rsp)
   47d06:	movq   $0x0,0x28e8(%rsp)
   47d12:	mov    %r15,0xc8(%rsp)
   47d1a:	mov    $0xffffffffffffffff,%rcx
   47d21:	xor    %r13d,%r13d
   47d24:	movabs $0xeeeeeeeeeeeeeeef,%r10
   47d2e:	movabs $0x888888888888889,%rdi
   47d38:	pxor   %xmm0,%xmm0
   47d3c:	pcmpeqd %xmm1,%xmm1
   47d40:	xor    %r15d,%r15d
   47d43:	jmp    47dfe <<c30walk::ctor2su::Ctor2su>::build::<false>+0x55e>
   47d48:	nopl   0x0(%rax,%rax,1)
   47d50:	mov    0x8(%rdi),%rax
   47d54:	mov    %rbp,%rcx
   47d57:	shl    $0x7,%rcx
   47d5b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47d61:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47d67:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47d6d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47d73:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47d79:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47d7f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47d85:	movdqa %xmm0,(%rax,%rcx,1)
   47d8a:	mov    %rbp,%rax
   47d8d:	inc    %rax
   47d90:	mov    %rax,0x10(%rdi)
   47d94:	mov    %r12,%rax
   47d97:	shl    $0x6,%rax
   47d9b:	or     $0x10000000,%ebp
   47da1:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   47da7:	movdqa %xmm1,0x20(%rbx,%rax,1)
   47dad:	movdqa %xmm1,0x10(%rbx,%rax,1)
   47db3:	movdqa %xmm1,(%rbx,%rax,1)
   47db8:	mov    %ebp,0x3c(%rbx,%rax,1)
   47dbc:	mov    %r12,%rdi
   47dbf:	shr    $0x6,%rdi
   47dc3:	cmp    %rsi,%rdi
   47dc6:	jae    490b9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1819>
   47dcc:	mov    $0x1,%eax
   47dd1:	mov    %r12d,%ecx
   47dd4:	shl    %cl,%rax
   47dd7:	or     %rax,(%r9,%rdi,8)
   47ddb:	mov    %r12,%rcx
   47dde:	mov    0x8(%rsp),%r11
   47de3:	movabs $0x888888888888889,%rdi
   47ded:	add    $0x4,%r15
   47df1:	cmp    $0x400,%r15
   47df8:	je     47e90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x5f0>
   47dfe:	mov    %r13,%rax
   47e01:	mov    (%r14,%r15,1),%r13d
   47e05:	add    %rax,%r13
   47e08:	cmp    %r11,%r13
   47e0b:	setae  %r8b
   47e0f:	mov    %r13,%rax
   47e12:	movabs $0x8888888888888889,%rdx
   47e1c:	mul    %rdx
   47e1f:	mov    %rdx,%r12
   47e22:	shr    $0x4,%r12
   47e26:	mov    %r13,%rax
   47e29:	imul   %r10,%rax
   47e2d:	ror    $1,%rax
   47e30:	cmp    %rdi,%rax
   47e33:	setb   %al
   47e36:	cmp    %rcx,%r12
   47e39:	sete   %dl
   47e3c:	or     %r8b,%dl
   47e3f:	or     %al,%dl
   47e41:	jne    47ded <<c30walk::ctor2su::Ctor2su>::build::<false>+0x54d>
   47e43:	mov    0x5530(%rsp),%rdi
   47e4b:	mov    0x10(%rdi),%rbp
   47e4f:	cmp    (%rdi),%rbp
   47e52:	jne    47d50 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47e58:	mov    %rsi,0x20(%rsp)
   47e5d:	mov    %r9,0x30(%rsp)
   47e62:	call   9c910 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   47e67:	pcmpeqd %xmm1,%xmm1
   47e6b:	pxor   %xmm0,%xmm0
   47e6f:	movabs $0xeeeeeeeeeeeeeeef,%r10
   47e79:	mov    0x30(%rsp),%r9
   47e7e:	mov    0x20(%rsp),%rsi
   47e83:	mov    0x5530(%rsp),%rdi
   47e8b:	jmp    47d50 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x4b0>
   47e90:	cmpl   $0x0,0x14(%rsp)
   47e95:	je     48053 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47e9b:	mov    0xd8(%rsp),%r8
   47ea3:	test   %r8,%r8
   47ea6:	je     48053 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47eac:	mov    %r11,%r9
   47eaf:	shl    $0x20,%r9
   47eb3:	mov    0x14(%rsp),%r10d
   47eb8:	mov    0xa8(%rsp),%rdi
   47ec0:	mov    0xd0(%rsp),%rbp
   47ec8:	xor    %r15d,%r15d
   47ecb:	movabs $0xff51afd7ed558ccd,%rbx
   47ed5:	pxor   %xmm0,%xmm0
   47ed9:	pcmpeqd %xmm1,%xmm1
   47edd:	jmp    47f70 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6d0>
   47ee2:	mov    0x8(%rdi),%rax
   47ee6:	mov    %rbx,%rcx
   47ee9:	shl    $0x7,%rcx
   47eed:	movdqa %xmm0,0x70(%rax,%rcx,1)
   47ef3:	movdqa %xmm0,0x60(%rax,%rcx,1)
   47ef9:	movdqa %xmm0,0x50(%rax,%rcx,1)
   47eff:	movdqa %xmm0,0x40(%rax,%rcx,1)
   47f05:	movdqa %xmm0,0x30(%rax,%rcx,1)
   47f0b:	movdqa %xmm0,0x20(%rax,%rcx,1)
   47f11:	movdqa %xmm0,0x10(%rax,%rcx,1)
   47f17:	movdqa %xmm0,(%rax,%rcx,1)
   47f1c:	mov    %rbx,%rax
   47f1f:	inc    %rax
   47f22:	mov    %rax,0x10(%rdi)
   47f26:	or     $0x10000000,%ebx
   47f2c:	movdqu %xmm1,0x2c(%rbp)
   47f31:	movdqa %xmm1,0x20(%rbp)
   47f36:	movdqa %xmm1,0x10(%rbp)
   47f3b:	movdqa %xmm1,0x0(%rbp)
   47f40:	mov    %ebx,0x3c(%rbp)
   47f43:	or     %r14,(%r12,%r13,8)
   47f47:	movabs $0xff51afd7ed558ccd,%rbx
   47f51:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   47f60:	inc    %r15
   47f63:	add    $0x40,%rbp
   47f67:	cmp    %r15,%r8
   47f6a:	je     48053 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7b3>
   47f70:	mov    %r15,%r13
   47f73:	shr    $0x6,%r13
   47f77:	mov    0xb8(%rsp),%rsi
   47f7f:	cmp    %rsi,%r13
   47f82:	jae    490c5 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1825>
   47f88:	mov    0xb0(%rsp),%r12
   47f90:	mov    (%r12,%r13,8),%rax
   47f94:	mov    $0x1,%r14d
   47f9a:	mov    %r15d,%ecx
   47f9d:	shl    %cl,%r14
   47fa0:	bt     %r15,%rax
   47fa4:	jb     47f60 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6c0>
   47fa6:	mov    %r15,%rax
   47fa9:	xor    %r9,%rax
   47fac:	mov    %rax,%rcx
   47faf:	shr    $0x21,%rcx
   47fb3:	xor    %rax,%rcx
   47fb6:	imul   %rbx,%rcx
   47fba:	mov    %rcx,%rax
   47fbd:	shr    $0x21,%rax
   47fc1:	xor    %rcx,%rax
   47fc4:	movabs $0xc4ceb9fe1a85ec53,%rcx
   47fce:	imul   %rcx,%rax
   47fd2:	mov    %rax,%rcx
   47fd5:	shr    $0x21,%rcx
   47fd9:	xor    %rax,%rcx
   47fdc:	mov    %rcx,%rax
   47fdf:	shr    $0x3,%rax
   47fe3:	movabs $0x20c49ba5e353f7cf,%rdx
   47fed:	mul    %rdx
   47ff0:	shr    $0x4,%rdx
   47ff4:	imul   $0x3e8,%rdx,%rax
   47ffb:	sub    %rax,%rcx
   47ffe:	cmp    %r10,%rcx
   48001:	jae    47f60 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x6c0>
   48007:	mov    0x10(%rdi),%rbx
   4800b:	cmp    (%rdi),%rbx
   4800e:	jne    47ee2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x642>
   48014:	mov    %r8,0x20(%rsp)
   48019:	mov    %r9,0x30(%rsp)
   4801e:	mov    %r10,0x68(%rsp)
   48023:	mov    %rdi,0x60(%rsp)
   48028:	call   9c910 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4802d:	pcmpeqd %xmm1,%xmm1
   48031:	pxor   %xmm0,%xmm0
   48035:	mov    0x60(%rsp),%rdi
   4803a:	mov    0x68(%rsp),%r10
   4803f:	mov    0x30(%rsp),%r9
   48044:	mov    0x20(%rsp),%r8
   48049:	mov    0x8(%rsp),%r11
   4804e:	jmp    47ee2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x642>
   48053:	test   %r11,%r11
   48056:	je     48d20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1480>
   4805c:	movq   $0x0,0x28(%rsp)
   48065:	jmp    480a4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x804>
   48067:	mov    0x38(%rsp),%r15
   4806c:	lea    0x1(%rax),%rdx
   48070:	add    $0xffffffffffffffe3,%rax
   48074:	xor    %esi,%esi
   48076:	cmp    $0x1e,%rdx
   4807a:	setae  %sil
   4807e:	cmovb  %rdx,%rax
   48082:	lea    (%rcx,%rsi,2),%ecx
   48085:	inc    %ecx
   48087:	mov    %rax,0x28e8(%rsp)
   4808f:	mov    %ecx,0x28f0(%rsp)
   48096:	cmp    %rdi,0x98(%rsp)
   4809e:	jae    48d20 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1480>
   480a4:	mov    0x28(%rsp),%rbx
   480a9:	mov    $0x400,%edx
   480ae:	lea    0x28f8(%rsp),%rdi
   480b6:	xor    %esi,%esi
   480b8:	call   *0x105fb2(%rip)        # 14e070 <memset@GLIBC_2.2.5>
   480be:	lea    0x128(%rsp),%rax
   480c6:	xor    %ecx,%ecx
   480c8:	nopl   0x0(%rax,%rax,1)
   480d0:	mov    -0x40(%rax),%rdx
   480d4:	sub    -0x30(%rax),%rdx
   480d8:	shr    $1,%rdx
   480db:	add    -0x28(%rax),%edx
   480de:	mov    -0x18(%rax),%rsi
   480e2:	sub    -0x8(%rax),%rsi
   480e6:	mov    %edx,0x28f8(%rsp,%rcx,4)
   480ed:	shr    $1,%rsi
   480f0:	add    (%rax),%esi
   480f2:	mov    %esi,0x28fc(%rsp,%rcx,4)
   480f9:	add    $0x2,%rcx
   480fd:	add    $0x50,%rax
   48101:	cmp    $0x100,%rcx
   48108:	jne    480d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x830>
   4810a:	mov    0xc0(%rsp),%r12
   48112:	mov    0x10(%r12),%r14
   48117:	cmp    (%r12),%r14
   4811b:	je     48d13 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1473>
   48121:	mov    %r14,%rdi
   48124:	shl    $0xa,%rdi
   48128:	add    0x8(%r12),%rdi
   4812d:	mov    $0x400,%edx
   48132:	lea    0x28f8(%rsp),%rsi
   4813a:	call   *0x105f50(%rip)        # 14e090 <memcpy@GLIBC_2.14>
   48140:	inc    %r14
   48143:	lea    0xf000(%rbx),%rax
   4814a:	mov    0x8(%rsp),%rdi
   4814f:	cmp    %rdi,%rax
   48152:	mov    %rdi,%rcx
   48155:	mov    %rax,0x98(%rsp)
   4815d:	cmovb  %rax,%rcx
   48161:	mov    %r14,0x10(%r12)
   48166:	mov    %rcx,0x28(%rsp)
   4816b:	mov    %rcx,%r12
   4816e:	sub    %rbx,%r12
   48171:	cmp    $0x10,%r12
   48175:	jb     48b40 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12a0>
   4817b:	add    0x50(%rsp),%rbx
   48180:	xor    %edx,%edx
   48182:	lea    0xe8(%rsp),%r8
   4818a:	jmp    481dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93d>
   4818c:	nopl   0x0(%rax)
   48190:	mov    0x28e8(%rsp),%rax
   48198:	lea    0x10(%rax),%rcx
   4819c:	add    $0xfffffffffffffff2,%rax
   481a0:	xor    %edx,%edx
   481a2:	cmp    $0x1e,%rcx
   481a6:	setae  %dl
   481a9:	cmovb  %rcx,%rax
   481ad:	mov    0x28f0(%rsp),%ecx
   481b4:	lea    (%rcx,%rdx,2),%ecx
   481b7:	add    $0x10,%ecx
   481ba:	mov    %rax,0x28e8(%rsp)
   481c2:	mov    %ecx,0x28f0(%rsp)
   481c9:	xor    %edx,%edx
   481cb:	mov    0x40(%rsp),%rbx
   481d0:	mov    %r9,%r12
   481d3:	cmp    $0xf,%r9
   481d7:	jbe    48b4b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12ab>
   481dd:	mov    0x58(%rsp),%rcx
   481e2:	test   $0x1,%dl
   481e5:	jne    481ee <<c30walk::ctor2su::Ctor2su>::build::<false>+0x94e>
   481e7:	mov    (%rbx),%rcx
   481ea:	mov    0x8(%rbx),%r15
   481ee:	lea    0x10(%rbx),%r9
   481f2:	lea    -0x10(%r12),%rdx
   481f7:	xor    %rcx,%r15
   481fa:	mov    %rcx,%rax
   481fd:	rol    $0x8,%rax
   48201:	xor    %rcx,%rax
   48204:	or     %r15,%rax
   48207:	jne    48260 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x9c0>
   48209:	cmp    $0x10,%rdx
   4820d:	jb     483b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb10>
   48213:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48220:	mov    (%r9),%rsi
   48223:	mov    0x8(%r9),%rax
   48227:	mov    $0x1,%r10b
   4822a:	cmp    %rcx,%rsi
   4822d:	jne    483ba <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb1a>
   48233:	cmp    %rcx,%rax
   48236:	jne    483ba <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb1a>
   4823c:	add    $0xfffffffffffffff0,%rdx
   48240:	add    $0x10,%r9
   48244:	cmp    $0xf,%rdx
   48248:	ja     48220 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x980>
   4824a:	mov    %r9,0x40(%rsp)
   4824f:	xor    %r10d,%r10d
   48252:	mov    %rcx,0x58(%rsp)
   48257:	jmp    483c9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb29>
   4825c:	nopl   0x0(%rax)
   48260:	mov    %r9,0x40(%rsp)
   48265:	mov    %rdx,0x18(%rsp)
   4826a:	mov    0x28(%rsp),%rdx
   4826f:	sub    %r12,%rdx
   48272:	mov    0x28f0(%rsp),%r12d
   4827a:	mov    $0x1e,%r9d
   48280:	sub    0x28e8(%rsp),%r9
   48288:	cmp    $0x10,%r9
   4828c:	mov    $0x10,%r15d
   48292:	cmovb  %r9,%r15
   48296:	add    0x50(%rsp),%rdx
   4829b:	cmp    $0x1,%r15
   4829f:	mov    %r15,%rbp
   482a2:	adc    $0x0,%rbp
   482a6:	xor    %r13d,%r13d
   482a9:	nopl   0x0(%rax)
   482b0:	movzbl (%rbx,%r13,1),%esi
   482b5:	lea    (%rsi,%rsi,4),%rcx
   482b9:	lea    (%r8,%rcx,8),%r14
   482bd:	mov    (%r8,%rcx,8),%rax
   482c1:	cmp    0x8(%r8,%rcx,8),%rax
   482c6:	je     482dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa3d>
   482c8:	lea    (%r12,%r13,1),%rcx
   482cc:	mov    %cx,(%rax)
   482cf:	addq   $0x2,(%r14)
   482d3:	inc    %r13
   482d6:	cmp    %r13,%rbp
   482d9:	jne    482b0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa10>
   482db:	jmp    48320 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa80>
   482dd:	mov    %r9,0x30(%rsp)
   482e2:	mov    %rdx,0x20(%rsp)
   482e7:	lea    0xa8(%rsp),%rdi
   482ef:	call   97660 <<c30walk::ctor2su::W<false>>::refill>
   482f4:	lea    0xe8(%rsp),%r8
   482fc:	test   %al,%al
   482fe:	je     48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48304:	mov    (%r14),%rax
   48307:	mov    0x8(%rsp),%rdi
   4830c:	mov    0x20(%rsp),%rdx
   48311:	mov    0x30(%rsp),%r9
   48316:	jmp    482c8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xa28>
   48318:	nopl   0x0(%rax,%rax,1)
   48320:	cmp    $0xf,%r9
   48324:	mov    0x18(%rsp),%r9
   48329:	ja     48190 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   4832f:	add    $0x2,%r12d
   48333:	movzwl %r12w,%ebx
   48337:	nopw   0x0(%rax,%rax,1)
   48340:	movzbl (%rdx,%r15,1),%esi
   48345:	lea    (%rsi,%rsi,4),%rcx
   48349:	lea    (%r8,%rcx,8),%r14
   4834d:	mov    (%r8,%rcx,8),%rax
   48351:	cmp    0x8(%r8,%rcx,8),%rax
   48356:	je     48371 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xad1>
   48358:	lea    (%rbx,%r15,1),%rcx
   4835c:	mov    %cx,(%rax)
   4835f:	addq   $0x2,(%r14)
   48363:	inc    %r15
   48366:	cmp    $0x10,%r15
   4836a:	jne    48340 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xaa0>
   4836c:	jmp    48190 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x8f0>
   48371:	mov    %rdx,%r12
   48374:	lea    0xa8(%rsp),%rdi
   4837c:	call   97660 <<c30walk::ctor2su::W<false>>::refill>
   48381:	lea    0xe8(%rsp),%r8
   48389:	test   %al,%al
   4838b:	je     48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48391:	mov    (%r14),%rax
   48394:	mov    0x8(%rsp),%rdi
   48399:	mov    0x18(%rsp),%r9
   4839e:	mov    %r12,%rdx
   483a1:	jmp    48358 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xab8>
   483a3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   483b0:	mov    %r9,0x40(%rsp)
   483b5:	xor    %r10d,%r10d
   483b8:	jmp    483c9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb29>
   483ba:	mov    %r9,0x40(%rsp)
   483bf:	mov    %rsi,0x58(%rsp)
   483c4:	mov    %rax,0x38(%rsp)
   483c9:	mov    %rdx,0x18(%rsp)
   483ce:	sub    %rdx,%r12
   483d1:	movzbl %cl,%eax
   483d4:	mov    %rax,0xa0(%rsp)
   483dc:	lea    (%rax,%rax,4),%rdx
   483e0:	mov    (%r8,%rdx,8),%rax
   483e4:	mov    0x1c(%r8,%rdx,8),%esi
   483e9:	mov    %rax,%rdi
   483ec:	sub    0x10(%r8,%rdx,8),%rdi
   483f1:	shr    $1,%rdi
   483f4:	add    0x18(%r8,%rdx,8),%edi
   483f9:	add    %r12,%rdi
   483fc:	cmp    %rsi,%rdi
   483ff:	ja     48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48405:	mov    %r10d,0x4c(%rsp)
   4840a:	lea    (%r8,%rdx,8),%rbx
   4840e:	movzbl %cl,%r15d
   48412:	mov    0x8(%rbx),%rsi
   48416:	mov    $0x88888889,%r14d
   4841c:	lea    -0x2a763(%rip),%r13        # 1dcc0 <c30walk::ctor2su::PATTERNS>
   48423:	mov    %rbx,0x80(%rsp)
   4842b:	jmp    48466 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xbc6>
   4842d:	nopl   (%rax)
   48430:	mov    0x3c(%rsi),%edi
   48433:	mov    $0xfffff,%r9d
   48439:	and    %r9d,%edi
   4843c:	sub    %r8d,%ecx
   4843f:	sub    %eax,%r8d
   48442:	mov    %r8d,%eax
   48445:	lea    (%rsi,%rax,2),%rax
   48449:	mov    %rax,0x10(%rbx)
   4844d:	mov    %rax,(%rbx)
   48450:	lea    (%rax,%rcx,2),%rsi
   48454:	mov    %rsi,0x8(%rbx)
   48458:	mov    %edx,0x20(%rbx)
   4845b:	mov    %edi,0x24(%rbx)
   4845e:	lea    0xe8(%rsp),%r8
   48466:	sub    %rax,%rsi
   48469:	shr    $1,%rsi
   4846c:	cmp    %rsi,%r12
   4846f:	cmovb  %r12,%rsi
   48473:	test   %rsi,%rsi
   48476:	je     48530 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc90>
   4847c:	mov    0x28e8(%rsp),%rdi
   48484:	cmp    $0x1d,%rdi
   48488:	ja     49097 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17f7>
   4848e:	cmp    $0x1f,%rsi
   48492:	jae    48da1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1501>
   48498:	mov    0x28f0(%rsp),%ecx
   4849f:	mov    %rdi,%r11
   484a2:	shl    $0x6,%r11
   484a6:	lea    (%r11,%r13,1),%rdx
   484aa:	lea    -0x2(,%rsi,2),%r9
   484b2:	cmp    $0xe,%r9
   484b6:	jb     484cb <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc2b>
   484b8:	mov    %rax,%r8
   484bb:	sub    %r13,%r8
   484be:	sub    %r11,%r8
   484c1:	cmp    $0x10,%r8
   484c5:	jae    48918 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1078>
   484cb:	mov    %rdx,%r8
   484ce:	xor    %r10d,%r10d
   484d1:	lea    (%rdx,%rsi,2),%rdx
   484d5:	lea    (%rax,%r10,2),%r9
   484d9:	nopl   0x0(%rax)
   484e0:	movzwl (%r8),%r10d
   484e4:	add    %cx,%r10w
   484e8:	add    $0x2,%r8
   484ec:	mov    %r10w,(%r9)
   484f0:	add    $0x2,%r9
   484f4:	cmp    %rdx,%r8
   484f7:	jne    484e0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc40>
   484f9:	lea    (%rax,%rsi,2),%rax
   484fd:	mov    %rax,(%rbx)
   48500:	add    %rsi,%rdi
   48503:	lea    0x2(%rsi),%edx
   48506:	mov    %rdi,%r8
   48509:	sub    $0x1e,%r8
   4850d:	cmovb  %rdi,%r8
   48511:	cmovb  %esi,%edx
   48514:	mov    %r8,0x28e8(%rsp)
   4851c:	add    %ecx,%edx
   4851e:	mov    %edx,0x28f0(%rsp)
   48525:	sub    %rsi,%r12
   48528:	lea    0xe8(%rsp),%r8
   48530:	test   %r12,%r12
   48533:	je     48b00 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1260>
   48539:	mov    %rax,%r8
   4853c:	cmp    0x10(%rbx),%rax
   48540:	je     4855e <<c30walk::ctor2su::Ctor2su>::build::<false>+0xcbe>
   48542:	lea    0xa8(%rsp),%rdi
   4854a:	mov    0xa0(%rsp),%rsi
   48552:	call   97060 <<c30walk::ctor2su::W<false>>::finish>
   48557:	mov    (%rbx),%r8
   4855a:	mov    0x10(%rbx),%rax
   4855e:	sub    %rax,%r8
   48561:	shr    $1,%r8
   48564:	add    0x18(%rbx),%r8d
   48568:	mov    %r8d,0x18(%rbx)
   4856c:	pxor   %xmm0,%xmm0
   48570:	movdqu %xmm0,(%rbx)
   48574:	movq   $0x0,0x10(%rbx)
   4857c:	cmp    $0x1e,%r12
   48580:	jb     48a90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11f0>
   48586:	mov    %r8,%rcx
   48589:	imul   %r14,%rcx
   4858d:	shr    $0x24,%rcx
   48591:	mov    %ecx,%eax
   48593:	shl    $0x5,%eax
   48596:	lea    (%rcx,%rcx,1),%edx
   48599:	sub    %eax,%edx
   4859b:	add    %r8d,%edx
   4859e:	jne    48a90 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11f0>
   485a4:	mov    %r12,%rax
   485a7:	movabs $0x8888888888888889,%rdx
   485b1:	mul    %rdx
   485b4:	shr    $0x4,%rdx
   485b8:	mov    %ecx,%r9d
   485bb:	cmpl   $0x0,0x14(%rsp)
   485c0:	mov    %r12,0x68(%rsp)
   485c5:	mov    %rdx,0x60(%rsp)
   485ca:	je     488d0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1030>
   485d0:	mov    0x28f0(%rsp),%r10d
   485d8:	mov    0xb8(%rsp),%rax
   485e0:	mov    %rax,0x20(%rsp)
   485e5:	mov    0xa8(%rsp),%r11
   485ed:	mov    0xb0(%rsp),%rax
   485f5:	mov    %rax,0x30(%rsp)
   485fa:	mov    0xd0(%rsp),%rax
   48602:	mov    0x28e8(%rsp),%r12
   4860a:	mov    %r12,%rdi
   4860d:	shl    $0x6,%rdi
   48611:	add    %r13,%rdi
   48614:	mov    %r9,%rcx
   48617:	shl    $0x6,%rcx
   4861b:	lea    (%rax,%rcx,1),%rbx
   4861f:	add    $0x3c,%rbx
   48623:	mov    %r10d,%r13d
   48626:	shl    $0x8,%r13d
   4862a:	or     %r15d,%r13d
   4862d:	xor    %r14d,%r14d
   48630:	jmp    48844 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xfa4>
   48635:	data16 cs nopw 0x0(%rax,%rax,1)
   48640:	mov    0x8(%r11),%rcx
   48644:	shl    $0x7,%eax
   48647:	mov    %r13d,(%rcx,%rax,1)
   4864b:	movzwl 0x2(%rdi),%esi
   4864f:	lea    (%r10,%r14,1),%ebp
   48653:	add    %ebp,%esi
   48655:	shl    $0x8,%esi
   48658:	or     %r15d,%esi
   4865b:	mov    %esi,0x4(%rcx,%rax,1)
   4865f:	movzwl 0x4(%rdi),%esi
   48663:	add    %ebp,%esi
   48665:	shl    $0x8,%esi
   48668:	or     %r15d,%esi
   4866b:	mov    %esi,0x8(%rcx,%rax,1)
   4866f:	movzwl 0x6(%rdi),%esi
   48673:	add    %ebp,%esi
   48675:	shl    $0x8,%esi
   48678:	or     %r15d,%esi
   4867b:	mov    %esi,0xc(%rcx,%rax,1)
   4867f:	movzwl 0x8(%rdi),%esi
   48683:	add    %ebp,%esi
   48685:	shl    $0x8,%esi
   48688:	or     %r15d,%esi
   4868b:	mov    %esi,0x10(%rcx,%rax,1)
   4868f:	movzwl 0xa(%rdi),%esi
   48693:	add    %ebp,%esi
   48695:	shl    $0x8,%esi
   48698:	or     %r15d,%esi
   4869b:	mov    %esi,0x14(%rcx,%rax,1)
   4869f:	movzwl 0xc(%rdi),%esi
   486a3:	add    %ebp,%esi
   486a5:	shl    $0x8,%esi
   486a8:	or     %r15d,%esi
   486ab:	mov    %esi,0x18(%rcx,%rax,1)
   486af:	movzwl 0xe(%rdi),%esi
   486b3:	add    %ebp,%esi
   486b5:	shl    $0x8,%esi
   486b8:	or     %r15d,%esi
   486bb:	mov    %esi,0x1c(%rcx,%rax,1)
   486bf:	movzwl 0x10(%rdi),%esi
   486c3:	add    %ebp,%esi
   486c5:	shl    $0x8,%esi
   486c8:	or     %r15d,%esi
   486cb:	mov    %esi,0x20(%rcx,%rax,1)
   486cf:	movzwl 0x12(%rdi),%esi
   486d3:	add    %ebp,%esi
   486d5:	shl    $0x8,%esi
   486d8:	or     %r15d,%esi
   486db:	mov    %esi,0x24(%rcx,%rax,1)
   486df:	movzwl 0x14(%rdi),%esi
   486e3:	add    %ebp,%esi
   486e5:	shl    $0x8,%esi
   486e8:	or     %r15d,%esi
   486eb:	mov    %esi,0x28(%rcx,%rax,1)
   486ef:	movzwl 0x16(%rdi),%esi
   486f3:	add    %ebp,%esi
   486f5:	shl    $0x8,%esi
   486f8:	or     %r15d,%esi
   486fb:	mov    %esi,0x2c(%rcx,%rax,1)
   486ff:	movzwl 0x18(%rdi),%esi
   48703:	add    %ebp,%esi
   48705:	shl    $0x8,%esi
   48708:	or     %r15d,%esi
   4870b:	mov    %esi,0x30(%rcx,%rax,1)
   4870f:	movzwl 0x1a(%rdi),%esi
   48713:	add    %ebp,%esi
   48715:	shl    $0x8,%esi
   48718:	or     %r15d,%esi
   4871b:	mov    %esi,0x34(%rcx,%rax,1)
   4871f:	movzwl 0x1c(%rdi),%esi
   48723:	add    %ebp,%esi
   48725:	shl    $0x8,%esi
   48728:	or     %r15d,%esi
   4872b:	mov    %esi,0x38(%rcx,%rax,1)
   4872f:	movzwl 0x1e(%rdi),%esi
   48733:	add    %ebp,%esi
   48735:	shl    $0x8,%esi
   48738:	or     %r15d,%esi
   4873b:	mov    %esi,0x3c(%rcx,%rax,1)
   4873f:	movzwl 0x20(%rdi),%esi
   48743:	add    %ebp,%esi
   48745:	shl    $0x8,%esi
   48748:	or     %r15d,%esi
   4874b:	mov    %esi,0x40(%rcx,%rax,1)
   4874f:	movzwl 0x22(%rdi),%esi
   48753:	add    %ebp,%esi
   48755:	shl    $0x8,%esi
   48758:	or     %r15d,%esi
   4875b:	mov    %esi,0x44(%rcx,%rax,1)
   4875f:	movzwl 0x24(%rdi),%esi
   48763:	add    %ebp,%esi
   48765:	shl    $0x8,%esi
   48768:	or     %r15d,%esi
   4876b:	mov    %esi,0x48(%rcx,%rax,1)
   4876f:	movzwl 0x26(%rdi),%esi
   48773:	add    %ebp,%esi
   48775:	shl    $0x8,%esi
   48778:	or     %r15d,%esi
   4877b:	mov    %esi,0x4c(%rcx,%rax,1)
   4877f:	movzwl 0x28(%rdi),%esi
   48783:	add    %ebp,%esi
   48785:	shl    $0x8,%esi
   48788:	or     %r15d,%esi
   4878b:	mov    %esi,0x50(%rcx,%rax,1)
   4878f:	movzwl 0x2a(%rdi),%esi
   48793:	add    %ebp,%esi
   48795:	shl    $0x8,%esi
   48798:	or     %r15d,%esi
   4879b:	mov    %esi,0x54(%rcx,%rax,1)
   4879f:	movzwl 0x2c(%rdi),%esi
   487a3:	add    %ebp,%esi
   487a5:	shl    $0x8,%esi
   487a8:	or     %r15d,%esi
   487ab:	mov    %esi,0x58(%rcx,%rax,1)
   487af:	movzwl 0x2e(%rdi),%esi
   487b3:	add    %ebp,%esi
   487b5:	shl    $0x8,%esi
   487b8:	or     %r15d,%esi
   487bb:	mov    %esi,0x5c(%rcx,%rax,1)
   487bf:	movzwl 0x30(%rdi),%esi
   487c3:	add    %ebp,%esi
   487c5:	shl    $0x8,%esi
   487c8:	or     %r15d,%esi
   487cb:	mov    %esi,0x60(%rcx,%rax,1)
   487cf:	movzwl 0x32(%rdi),%esi
   487d3:	add    %ebp,%esi
   487d5:	shl    $0x8,%esi
   487d8:	or     %r15d,%esi
   487db:	mov    %esi,0x64(%rcx,%rax,1)
   487df:	movzwl 0x34(%rdi),%esi
   487e3:	add    %ebp,%esi
   487e5:	shl    $0x8,%esi
   487e8:	or     %r15d,%esi
   487eb:	mov    %esi,0x68(%rcx,%rax,1)
   487ef:	movzwl 0x36(%rdi),%esi
   487f3:	add    %ebp,%esi
   487f5:	shl    $0x8,%esi
   487f8:	or     %r15d,%esi
   487fb:	mov    %esi,0x6c(%rcx,%rax,1)
   487ff:	movzwl 0x38(%rdi),%esi
   48803:	add    %ebp,%esi
   48805:	shl    $0x8,%esi
   48808:	or     %r15d,%esi
   4880b:	mov    %esi,0x70(%rcx,%rax,1)
   4880f:	movzwl 0x3a(%rdi),%esi
   48813:	add    %ebp,%esi
   48815:	shl    $0x8,%esi
   48818:	or     %r15d,%esi
   4881b:	mov    %esi,0x74(%rcx,%rax,1)
   4881f:	lea    (%r10,%r14,1),%eax
   48823:	add    $0x20,%eax
   48826:	mov    %eax,0x28f0(%rsp)
   4882d:	add    $0x20,%r14
   48831:	inc    %r9
   48834:	add    $0x2000,%r13d
   4883b:	dec    %rdx
   4883e:	je     48a4c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x11ac>
   48844:	mov    %r9,%rax
   48847:	shr    $0x6,%rax
   4884b:	cmp    0x20(%rsp),%rax
   48850:	jae    48fad <<c30walk::ctor2su::Ctor2su>::build::<false>+0x170d>
   48856:	mov    0x30(%rsp),%rcx
   4885b:	mov    (%rcx,%rax,8),%rax
   4885f:	bt     %r9,%rax
   48863:	jae    48890 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xff0>
   48865:	cmp    $0x1e,%r12
   48869:	jae    48fd3 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1733>
   4886f:	mov    (%rbx,%r14,2),%eax
   48873:	and    $0xfffff,%eax
   48878:	mov    0x10(%r11),%rcx
   4887c:	cmp    %rax,%rcx
   4887f:	ja     48640 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xda0>
   48885:	jmp    48fc1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1721>
   4888a:	nopw   0x0(%rax,%rax,1)
   48890:	cmp    $0x1e,%r12
   48894:	jae    49013 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1773>
   4889a:	movups 0x2c(%rdi),%xmm0
   4889e:	movups %xmm0,-0x10(%rbx,%r14,2)
   488a4:	movdqa (%rdi),%xmm0
   488a8:	movdqa 0x10(%rdi),%xmm1
   488ad:	movdqa 0x20(%rdi),%xmm2
   488b2:	movdqa %xmm2,-0x1c(%rbx,%r14,2)
   488b9:	movdqa %xmm1,-0x2c(%rbx,%r14,2)
   488c0:	movdqa %xmm0,-0x3c(%rbx,%r14,2)
   488c7:	mov    %r13d,(%rbx,%r14,2)
   488cb:	jmp    4881f <<c30walk::ctor2su::Ctor2su>::build::<false>+0xf7f>
   488d0:	mov    0x28e8(%rsp),%rdi
   488d8:	cmp    $0x1d,%rdi
   488dc:	ja     490a8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1808>
   488e2:	shl    $0x6,%rdi
   488e6:	add    %r13,%rdi
   488e9:	mov    0x28f0(%rsp),%eax
   488f0:	mov    0xd0(%rsp),%rcx
   488f8:	shl    $0x6,%r9
   488fc:	mov    %eax,%r14d
   488ff:	shl    $0x8,%r14d
   48903:	or     %r15d,%r14d
   48906:	lea    -0x1e(%r12),%rsi
   4890b:	cmp    $0x1e,%rsi
   4890f:	jae    4897d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10dd>
   48911:	xor    %esi,%esi
   48913:	jmp    48a0a <<c30walk::ctor2su::Ctor2su>::build::<false>+0x116a>
   48918:	shr    $1,%r9
   4891b:	inc    %r9
   4891e:	mov    %r9,%r10
   48921:	and    $0xfffffffffffffff8,%r10
   48925:	lea    (%rdx,%r10,2),%r8
   48929:	movd   %ecx,%xmm0
   4892d:	pshuflw $0x0,%xmm0,%xmm0
   48932:	add    %r13,%r11
   48935:	add    $0x8,%r11
   48939:	xor    %ebx,%ebx
   4893b:	nopl   0x0(%rax,%rax,1)
   48940:	movq   -0x8(%r11,%rbx,2),%xmm1
   48947:	movq   (%r11,%rbx,2),%xmm2
   4894d:	paddw  %xmm0,%xmm1
   48951:	paddw  %xmm0,%xmm2
   48955:	punpcklqdq %xmm2,%xmm1
   48959:	movdqu %xmm1,(%rax,%rbx,2)
   4895e:	add    $0x8,%rbx
   48962:	cmp    %rbx,%r10
   48965:	jne    48940 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x10a0>
   48967:	cmp    %r10,%r9
   4896a:	mov    0x80(%rsp),%rbx
   48972:	jne    484d1 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc31>
   48978:	jmp    484f9 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xc59>
   4897d:	mov    %rdx,%r10
   48980:	movabs $0xffffffffffffffe,%rsi
   4898a:	and    %rsi,%r10
   4898d:	lea    (%rcx,%r9,1),%r11
   48991:	add    $0x7c,%r11
   48995:	xor    %esi,%esi
   48997:	nopw   0x0(%rax,%rax,1)
   489a0:	movdqu 0x2c(%rdi),%xmm0
   489a5:	movdqu %xmm0,-0x50(%r11)
   489ab:	movdqa (%rdi),%xmm1
   489af:	movdqa 0x10(%rdi),%xmm2
   489b4:	movdqa 0x20(%rdi),%xmm3
   489b9:	movdqa %xmm3,-0x5c(%r11)
   489bf:	movdqa %xmm2,-0x6c(%r11)
   489c5:	movdqa %xmm1,-0x7c(%r11)
   489cb:	mov    %r14d,-0x40(%r11)
   489cf:	add    $0x2,%rsi
   489d3:	movdqa %xmm1,-0x3c(%r11)
   489d9:	movdqa %xmm2,-0x2c(%r11)
   489df:	movdqa %xmm3,-0x1c(%r11)
   489e5:	movdqu %xmm0,-0x10(%r11)
   489eb:	lea    0x2000(%r14),%ebx
   489f2:	mov    %ebx,(%r11)
   489f5:	add    $0x4000,%r14d
   489fc:	sub    $0xffffffffffffff80,%r11
   48a00:	cmp    %rsi,%r10
   48a03:	jne    489a0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1100>
   48a05:	test   $0x1,%dl
   48a08:	je     48a3e <<c30walk::ctor2su::Ctor2su>::build::<false>+0x119e>
   48a0a:	add    %r9,%rcx
   48a0d:	shl    $0x6,%rsi
   48a11:	movups 0x2c(%rdi),%xmm0
   48a15:	movups %xmm0,0x2c(%rcx,%rsi,1)
   48a1a:	movdqa (%rdi),%xmm0
   48a1e:	movdqa 0x10(%rdi),%xmm1
   48a23:	movdqa 0x20(%rdi),%xmm2
   48a28:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   48a2e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   48a34:	movdqa %xmm0,(%rcx,%rsi,1)
   48a39:	mov    %r14d,0x3c(%rcx,%rsi,1)
   48a3e:	mov    %edx,%ecx
   48a40:	shl    $0x5,%ecx
   48a43:	add    %ecx,%eax
   48a45:	mov    %eax,0x28f0(%rsp)
   48a4c:	mov    0x60(%rsp),%rdx
   48a51:	mov    %rdx,%rax
   48a54:	shl    $0x5,%rax
   48a58:	lea    (%rdx,%rdx,1),%rcx
   48a5c:	sub    %rax,%rcx
   48a5f:	mov    0x68(%rsp),%r12
   48a64:	add    %rcx,%r12
   48a67:	lea    (%rdx,%rdx,1),%eax
   48a6a:	shl    $0x5,%edx
   48a6d:	sub    %eax,%edx
   48a6f:	add    %edx,%r8d
   48a72:	mov    0x80(%rsp),%rbx
   48a7a:	mov    %r8d,0x18(%rbx)
   48a7e:	test   %r12,%r12
   48a81:	mov    $0x88888889,%r14d
   48a87:	lea    -0x2adce(%rip),%r13        # 1dcc0 <c30walk::ctor2su::PATTERNS>
   48a8e:	je     48b07 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1267>
   48a90:	mov    0x1c(%rbx),%ecx
   48a93:	mov    %r8d,%edi
   48a96:	imul   %r14,%rdi
   48a9a:	mov    %rdi,%rdx
   48a9d:	shr    $0x24,%rdx
   48aa1:	mov    %edx,%eax
   48aa3:	shl    $0x5,%eax
   48aa6:	sub    %edx,%eax
   48aa8:	sub    %edx,%eax
   48aaa:	lea    0x1e(%rax),%esi
   48aad:	cmp    %ecx,%esi
   48aaf:	cmovb  %esi,%ecx
   48ab2:	shr    $0x2a,%rdi
   48ab6:	mov    0xb8(%rsp),%rsi
   48abe:	cmp    %rsi,%rdi
   48ac1:	jae    4908b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x17eb>
   48ac7:	mov    %rdx,%rsi
   48aca:	shl    $0x6,%rsi
   48ace:	add    0xd0(%rsp),%rsi
   48ad6:	mov    0xb0(%rsp),%r9
   48ade:	mov    (%r9,%rdi,8),%rdi
   48ae2:	bt     %rdx,%rdi
   48ae6:	jb     48430 <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb90>
   48aec:	mov    $0xffffffff,%edi
   48af1:	jmp    4843c <<c30walk::ctor2su::Ctor2su>::build::<false>+0xb9c>
   48af6:	cs nopw 0x0(%rax,%rax,1)
   48b00:	mov    0x8(%rsp),%rdi
   48b05:	jmp    48b14 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1274>
   48b07:	mov    0x8(%rsp),%rdi
   48b0c:	lea    0xe8(%rsp),%r8
   48b14:	mov    0x18(%rsp),%r9
   48b19:	mov    0x38(%rsp),%r15
   48b1e:	mov    0x4c(%rsp),%edx
   48b22:	mov    0x40(%rsp),%rbx
   48b27:	mov    %r9,%r12
   48b2a:	cmp    $0xf,%r9
   48b2e:	ja     481dd <<c30walk::ctor2su::Ctor2su>::build::<false>+0x93d>
   48b34:	jmp    48b4b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x12ab>
   48b36:	cs nopw 0x0(%rax,%rax,1)
   48b40:	mov    %r12,%r9
   48b43:	lea    0xe8(%rsp),%r8
   48b4b:	test   %r9,%r9
   48b4e:	je     48096 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7f6>
   48b54:	mov    0x28(%rsp),%r12
   48b59:	sub    %r9,%r12
   48b5c:	mov    0x28e8(%rsp),%rax
   48b64:	mov    $0x1e,%ebx
   48b69:	sub    %rax,%rbx
   48b6c:	cmp    %r9,%rbx
   48b6f:	mov    %r9,0x18(%rsp)
   48b74:	cmovae %r9,%rbx
   48b78:	mov    %rbx,%r13
   48b7b:	add    %r12,%r13
   48b7e:	jb     48fe7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1747>
   48b84:	cmp    %rdi,%r13
   48b87:	ja     48fe7 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1747>
   48b8d:	mov    %r15,0x38(%rsp)
   48b92:	mov    0x28f0(%rsp),%r14d
   48b9a:	cmp    $0x1e,%rax
   48b9e:	jne    48c2f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x138f>
   48ba4:	cmp    %r13,0x28(%rsp)
   48ba9:	jb     48ffc <<c30walk::ctor2su::Ctor2su>::build::<false>+0x175c>
   48baf:	mov    0x18(%rsp),%r9
   48bb4:	jne    48c99 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13f9>
   48bba:	mov    0x28e8(%rsp),%rax
   48bc2:	mov    0x28f0(%rsp),%ecx
   48bc9:	cmp    $0x1,%r9
   48bcd:	je     48067 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7c7>
   48bd3:	mov    %r9d,%edx
   48bd6:	and    $0xe,%edx
   48bd9:	mov    0x38(%rsp),%r15
   48bde:	xchg   %ax,%ax
   48be0:	lea    0x1(%rax),%rsi
   48be4:	add    $0xffffffffffffffe3,%rax
   48be8:	xor    %edi,%edi
   48bea:	cmp    $0x1e,%rsi
   48bee:	setae  %dil
   48bf2:	cmovb  %rsi,%rax
   48bf6:	lea    (%rcx,%rdi,2),%ecx
   48bf9:	lea    0x1(%rax),%rsi
   48bfd:	add    $0xffffffffffffffe3,%rax
   48c01:	xor    %edi,%edi
   48c03:	cmp    $0x1e,%rsi
   48c07:	setae  %dil
   48c0b:	cmovb  %rsi,%rax
   48c0f:	lea    (%rcx,%rdi,2),%ecx
   48c12:	add    $0x2,%ecx
   48c15:	add    $0xfffffffffffffffe,%rdx
   48c19:	jne    48be0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1340>
   48c1b:	test   $0x1,%r9b
   48c1f:	mov    0x8(%rsp),%rdi
   48c24:	jne    4806c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7cc>
   48c2a:	jmp    48087 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x7e7>
   48c2f:	add    0x50(%rsp),%r12
   48c34:	xor    %r15d,%r15d
   48c37:	nopw   0x0(%rax,%rax,1)
   48c40:	movzbl (%r12,%r15,1),%esi
   48c45:	lea    (%rsi,%rsi,4),%rcx
   48c49:	lea    (%r8,%rcx,8),%rbp
   48c4d:	mov    (%r8,%rcx,8),%rax
   48c51:	cmp    0x8(%r8,%rcx,8),%rax
   48c56:	je     48c71 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13d1>
   48c58:	lea    (%r14,%r15,1),%rcx
   48c5c:	mov    %cx,(%rax)
   48c5f:	addq   $0x2,0x0(%rbp)
   48c64:	inc    %r15
   48c67:	cmp    %r15,%rbx
   48c6a:	jne    48c40 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13a0>
   48c6c:	jmp    48ba4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1304>
   48c71:	lea    0xa8(%rsp),%rdi
   48c79:	call   97660 <<c30walk::ctor2su::W<false>>::refill>
   48c7e:	lea    0xe8(%rsp),%r8
   48c86:	test   %al,%al
   48c88:	je     48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48c8e:	mov    0x0(%rbp),%rax
   48c92:	mov    0x8(%rsp),%rdi
   48c97:	jmp    48c58 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x13b8>
   48c99:	mov    0x50(%rsp),%rax
   48c9e:	mov    0x28(%rsp),%rcx
   48ca3:	lea    (%rax,%rcx,1),%r15
   48ca7:	lea    (%r14,%rbx,1),%ebp
   48cab:	add    $0x2,%ebp
   48cae:	sub    %r9,%rbx
   48cb1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48cc0:	movzbl (%r15,%rbx,1),%esi
   48cc5:	lea    (%rsi,%rsi,4),%rcx
   48cc9:	lea    (%r8,%rcx,8),%r14
   48ccd:	mov    (%r8,%rcx,8),%rax
   48cd1:	cmp    0x8(%r8,%rcx,8),%rax
   48cd6:	je     48ceb <<c30walk::ctor2su::Ctor2su>::build::<false>+0x144b>
   48cd8:	mov    %bp,(%rax)
   48cdb:	addq   $0x2,(%r14)
   48cdf:	inc    %ebp
   48ce1:	inc    %rbx
   48ce4:	jne    48cc0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1420>
   48ce6:	jmp    48bba <<c30walk::ctor2su::Ctor2su>::build::<false>+0x131a>
   48ceb:	lea    0xa8(%rsp),%rdi
   48cf3:	call   97660 <<c30walk::ctor2su::W<false>>::refill>
   48cf8:	lea    0xe8(%rsp),%r8
   48d00:	test   %al,%al
   48d02:	je     48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48d04:	mov    (%r14),%rax
   48d07:	mov    0x8(%rsp),%rdi
   48d0c:	mov    0x18(%rsp),%r9
   48d11:	jmp    48cd8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1438>
   48d13:	mov    %r12,%rdi
   48d16:	call   ff400 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   48d1b:	jmp    48121 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x881>
   48d20:	mov    $0x84,%eax
   48d25:	lea    0xe8(%rsp),%r8
   48d2d:	nopl   (%rax)
   48d30:	lea    0x50(%rax),%rcx
   48d34:	cmp    $0x28d4,%rcx
   48d3b:	je     48db4 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1514>
   48d3d:	mov    0x64(%rsp,%rax,1),%rdx
   48d42:	sub    0x74(%rsp,%rax,1),%rdx
   48d47:	shr    $1,%rdx
   48d4a:	add    0x7c(%rsp,%rax,1),%edx
   48d4e:	cmp    0x80(%rsp,%rax,1),%edx
   48d55:	jne    48d7d <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14dd>
   48d57:	mov    0x8c(%rsp,%rax,1),%rdx
   48d5f:	sub    0x9c(%rsp,%rax,1),%rdx
   48d67:	shr    $1,%rdx
   48d6a:	add    0xa4(%rsp,%rax,1),%edx
   48d71:	cmp    0xa8(%rsp,%rax,1),%edx
   48d78:	mov    %rcx,%rax
   48d7b:	je     48d30 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1490>
   48d7d:	mov    0x5530(%rsp),%rax
   48d85:	movq   $0x0,0x10(%rax)
   48d8d:	xor    %eax,%eax
   48d8f:	add    $0x54f8,%rsp
   48d96:	pop    %rbx
   48d97:	pop    %r12
   48d99:	pop    %r13
   48d9b:	pop    %r14
   48d9d:	pop    %r15
   48d9f:	pop    %rbp
   48da0:	ret
   48da1:	lea    0x1007b0(%rip),%rcx        # 149558 <__frame_dummy_init_array_entry+0x2338>
   48da8:	mov    $0x1e,%edx
   48dad:	xor    %edi,%edi
   48daf:	call   f15e0 <core::slice::index::slice_index_fail>
   48db4:	xor    %r14d,%r14d
   48db7:	lea    0xa8(%rsp),%r15
   48dbf:	mov    0x88(%rsp),%r12
   48dc7:	mov    0x8(%rsp),%rdx
   48dcc:	mov    0x78(%rsp),%r13
   48dd1:	jmp    48e0b <<c30walk::ctor2su::Ctor2su>::build::<false>+0x156b>
   48dd3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   48de0:	sub    %rcx,%rax
   48de3:	inc    %r14
   48de6:	shr    $1,%rax
   48de9:	add    %eax,0x18(%r8)
   48ded:	pxor   %xmm0,%xmm0
   48df1:	movdqu %xmm0,(%r8)
   48df6:	movq   $0x0,0x10(%r8)
   48dfe:	add    $0x28,%r8
   48e02:	cmp    $0x100,%r14
   48e09:	je     48e36 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1596>
   48e0b:	mov    (%r8),%rcx
   48e0e:	mov    %rcx,%rax
   48e11:	cmp    0x10(%r8),%rcx
   48e15:	je     48de0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1540>
   48e17:	mov    %r15,%rdi
   48e1a:	mov    %r14,%rsi
   48e1d:	mov    %r8,%rbx
   48e20:	call   97060 <<c30walk::ctor2su::W<false>>::finish>
   48e25:	mov    %rbx,%r8
   48e28:	mov    0x8(%rsp),%rdx
   48e2d:	mov    (%rbx),%rax
   48e30:	mov    0x10(%rbx),%rcx
   48e34:	jmp    48de0 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1540>
   48e36:	xor    %ecx,%ecx
   48e38:	mov    0x8(%r12),%rbx
   48e3d:	test   %r13,%r13
   48e40:	setne  %al
   48e43:	je     48e87 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15e7>
   48e45:	mov    %al,%cl
   48e47:	mov    %r13,%rax
   48e4a:	sub    %rcx,%rax
   48e4d:	imul   $0xffffffffffffffe2,%rax,%rcx
   48e51:	add    %rdx,%rcx
   48e54:	cmp    $0x1e,%rcx
   48e58:	jae    48e87 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x15e7>
   48e5a:	lea    (%rbx,%r13,4),%rcx
   48e5e:	lea    (%rcx,%rdx,2),%rdi
   48e62:	add    $0xfffffffffffffffc,%rdi
   48e66:	lea    (%rax,%rax,1),%rcx
   48e6a:	shl    $0x5,%rax
   48e6e:	sub    %rcx,%rax
   48e71:	sub    %rdx,%rax
   48e74:	lea    0x3c(,%rax,2),%rdx
   48e7c:	mov    $0xff,%esi
   48e81:	call   *0x1051e9(%rip)        # 14e070 <memset@GLIBC_2.2.5>
   48e87:	shl    $0x6,%r13
   48e8b:	pcmpeqd %xmm0,%xmm0
   48e8f:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   48e96:	movdqa %xmm0,0x20(%rbx,%r13,1)
   48e9d:	movdqa %xmm0,0x10(%rbx,%r13,1)
   48ea4:	movdqa %xmm0,(%rbx,%r13,1)
   48eaa:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   48eb3:	mov    0x70(%rsp),%rax
   48eb8:	mov    0x8(%rax),%rdx
   48ebc:	mov    0x10(%rax),%rcx
   48ec0:	mov    0x5530(%rsp),%rax
   48ec8:	mov    0x8(%rax),%r8
   48ecc:	mov    0x10(%rax),%r9
   48ed0:	lea    0xa8(%rsp),%rdi
   48ed8:	mov    %rbx,%rsi
   48edb:	call   a5ff0 <c30walk::ctor::canonicalise>
   48ee0:	mov    0x90(%rsp),%rax
   48ee8:	mov    %rax,0x10(%r12)
   48eed:	mov    $0x1,%al
   48eef:	jmp    48d8f <<c30walk::ctor2su::Ctor2su>::build::<false>+0x14ef>
   48ef4:	mov    %rdi,%r12
   48ef7:	test   %rax,%rax
   48efa:	je     48f0c <<c30walk::ctor2su::Ctor2su>::build::<false>+0x166c>
   48efc:	mov    0x8(%r9),%rdi
   48f00:	mov    %r9,%r15
   48f03:	call   *0x105177(%rip)        # 14e080 <free@GLIBC_2.2.5>
   48f09:	mov    %r15,%r9
   48f0c:	movq   $0x0,(%r9)
   48f13:	movq   $0x40,0x8(%r9)
   48f1b:	movq   $0x0,0x10(%r9)
   48f23:	mov    $0x40,%edx
   48f28:	mov    %r9,%rdi
   48f2b:	lea    0x1(%r13),%rsi
   48f2f:	mov    %r9,%r15
   48f32:	call   9bbc0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48f37:	mov    %r15,%r9
   48f3a:	cmp    $0xffffffffffffffff,%rax
   48f3e:	mov    %r12,%rdi
   48f41:	je     47ad8 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x238>
   48f47:	jmp    48fa2 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x1702>
   48f49:	mov    %rdi,%r12
   48f4c:	mov    %r9,%r15
   48f4f:	test   %rax,%rax
   48f52:	je     48f66 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x16c6>
   48f54:	mov    0x5530(%rsp),%rax
   48f5c:	mov    0x8(%rax),%rdi
   48f60:	call   *0x10511a(%rip)        # 14e080 <free@GLIBC_2.2.5>
   48f66:	mov    0x5530(%rsp),%rdi
   48f6e:	movq   $0x0,(%rdi)
   48f75:	movq   $0x40,0x8(%rdi)
   48f7d:	movq   $0x0,0x10(%rdi)
   48f85:	mov    $0x80,%edx
   48f8a:	mov    %rbx,%rsi
   48f8d:	call   9bbc0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   48f92:	cmp    $0xffffffffffffffff,%rax
   48f96:	mov    %r15,%r9
   48f99:	mov    %r12,%rdi
   48f9c:	je     47aec <<c30walk::ctor2su::Ctor2su>::build::<false>+0x24c>
   48fa2:	mov    %rax,%rdi
   48fa5:	mov    %rdx,%rsi
   48fa8:	call   eed6b <alloc::raw_vec::handle_error>
   48fad:	lea    0xfe4f4(%rip),%rdx        # 1474a8 <__frame_dummy_init_array_entry+0x288>
   48fb4:	mov    %rax,%rdi
   48fb7:	mov    0x20(%rsp),%rsi
   48fbc:	call   f0803 <core::panicking::panic_bounds_check>
   48fc1:	lea    0x100650(%rip),%rdx        # 149618 <__frame_dummy_init_array_entry+0x23f8>
   48fc8:	mov    %rax,%rdi
   48fcb:	mov    %rcx,%rsi
   48fce:	call   f0803 <core::panicking::panic_bounds_check>
   48fd3:	lea    0x100626(%rip),%rdx        # 149600 <__frame_dummy_init_array_entry+0x23e0>
   48fda:	mov    $0x1e,%esi
   48fdf:	mov    %r12,%rdi
   48fe2:	call   f0803 <core::panicking::panic_bounds_check>
   48fe7:	lea    0x10059a(%rip),%rcx        # 149588 <__frame_dummy_init_array_entry+0x2368>
   48fee:	mov    %rdi,%rdx
   48ff1:	mov    %r12,%rdi
   48ff4:	mov    %r13,%rsi
   48ff7:	call   f15e0 <core::slice::index::slice_index_fail>
   48ffc:	lea    0x10056d(%rip),%rcx        # 149570 <__frame_dummy_init_array_entry+0x2350>
   49003:	mov    %rdi,%rdx
   49006:	mov    %r13,%rdi
   49009:	mov    0x28(%rsp),%rsi
   4900e:	call   f15e0 <core::slice::index::slice_index_fail>
   49013:	lea    0x1005ce(%rip),%rdx        # 1495e8 <__frame_dummy_init_array_entry+0x23c8>
   4901a:	mov    $0x1e,%esi
   4901f:	mov    %r12,%rdi
   49022:	call   f0803 <core::panicking::panic_bounds_check>
   49027:	mov    $0x10,%edx
   4902c:	mov    $0x4,%ecx
   49031:	mov    $0x400,%r8d
   49037:	mov    %rdi,%r15
   4903a:	mov    %r13,%rdi
   4903d:	xor    %esi,%esi
   4903f:	mov    %r9,%rbx
   49042:	call   6f3e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   49047:	mov    %r15,%rdi
   4904a:	mov    %rbx,%r9
   4904d:	jmp    47b79 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2d9>
   49052:	mov    %r9,%r13
   49055:	mov    $0x8,%ecx
   4905a:	mov    $0x8,%r8d
   49060:	mov    %rdi,%r15
   49063:	xor    %esi,%esi
   49065:	mov    %rbx,%rdx
   49068:	call   6f3e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   4906d:	mov    %r15,%rdi
   49070:	mov    0x10(%r15),%r15
   49074:	mov    0x8(%rdi),%r12
   49078:	lea    (%r12,%r15,8),%rdi
   4907c:	cmp    $0x1,%rbx
   49080:	jne    47b25 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x285>
   49086:	jmp    47b47 <<c30walk::ctor2su::Ctor2su>::build::<false>+0x2a7>
   4908b:	lea    0xfe416(%rip),%rdx        # 1474a8 <__frame_dummy_init_array_entry+0x288>
   49092:	call   f0803 <core::panicking::panic_bounds_check>
   49097:	lea    0x1004a2(%rip),%rdx        # 149540 <__frame_dummy_init_array_entry+0x2320>
   4909e:	mov    $0x1e,%esi
   490a3:	call   f0803 <core::panicking::panic_bounds_check>
   490a8:	lea    0x100521(%rip),%rdx        # 1495d0 <__frame_dummy_init_array_entry+0x23b0>
   490af:	mov    $0x1e,%esi
   490b4:	call   f0803 <core::panicking::panic_bounds_check>
   490b9:	lea    0x100408(%rip),%rdx        # 1494c8 <__frame_dummy_init_array_entry+0x22a8>
   490c0:	call   f0803 <core::panicking::panic_bounds_check>
   490c5:	lea    0xfe3dc(%rip),%rdx        # 1474a8 <__frame_dummy_init_array_entry+0x288>
   490cc:	mov    %r13,%rdi
   490cf:	call   f0803 <core::panicking::panic_bounds_check>
