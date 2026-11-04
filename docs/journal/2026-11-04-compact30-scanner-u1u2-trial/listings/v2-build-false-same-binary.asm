# <c30walk::ctor2::Ctor2>::build::<false>
# size 0x18e5

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003d7d0 <<c30walk::ctor2::Ctor2>::build::<false>>:
   3d7d0:	push   %rbp
   3d7d1:	push   %r15
   3d7d3:	push   %r14
   3d7d5:	push   %r13
   3d7d7:	push   %r12
   3d7d9:	push   %rbx
   3d7da:	sub    $0x1000,%rsp
   3d7e1:	movq   $0x0,(%rsp)
   3d7e9:	sub    $0x1000,%rsp
   3d7f0:	movq   $0x0,(%rsp)
   3d7f8:	sub    $0x1000,%rsp
   3d7ff:	movq   $0x0,(%rsp)
   3d807:	sub    $0x1000,%rsp
   3d80e:	movq   $0x0,(%rsp)
   3d816:	sub    $0x1000,%rsp
   3d81d:	movq   $0x0,(%rsp)
   3d825:	sub    $0x4e8,%rsp
   3d82c:	mov    %r8d,0x1c(%rsp)
   3d831:	mov    0x5520(%rsp),%rax
   3d839:	movq   $0x0,0x10(%r9)
   3d841:	movq   $0x0,0x10(%rax)
   3d849:	cmp    $0xeffe2,%rdx
   3d850:	ja     3ed47 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3d856:	mov    %rcx,%r15
   3d859:	mov    %rdx,%r13
   3d85c:	pxor   %xmm0,%xmm0
   3d860:	xor    %eax,%eax
   3d862:	pxor   %xmm2,%xmm2
   3d866:	pxor   %xmm1,%xmm1
   3d86a:	nopw   0x0(%rax,%rax,1)
   3d870:	movq   (%r15,%rax,4),%xmm3
   3d876:	movq   0x8(%r15,%rax,4),%xmm4
   3d87d:	punpckldq %xmm0,%xmm3
   3d881:	paddq  %xmm2,%xmm3
   3d885:	punpckldq %xmm0,%xmm4
   3d889:	paddq  %xmm1,%xmm4
   3d88d:	movq   0x10(%r15,%rax,4),%xmm2
   3d894:	movq   0x18(%r15,%rax,4),%xmm1
   3d89b:	punpckldq %xmm0,%xmm2
   3d89f:	paddq  %xmm3,%xmm2
   3d8a3:	punpckldq %xmm0,%xmm1
   3d8a7:	paddq  %xmm4,%xmm1
   3d8ab:	add    $0x8,%rax
   3d8af:	cmp    $0x100,%rax
   3d8b5:	jne    3d870 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   3d8b7:	paddq  %xmm2,%xmm1
   3d8bb:	pshufd $0xee,%xmm1,%xmm0
   3d8c0:	paddq  %xmm1,%xmm0
   3d8c4:	movq   %xmm0,%rax
   3d8c9:	cmp    %r13,%rax
   3d8cc:	jne    3ed47 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3d8d2:	pxor   %xmm2,%xmm2
   3d8d6:	xor    %eax,%eax
   3d8d8:	movdqa -0x2cd80(%rip),%xmm3        # 10b60 <GCC_except_table1006+0xa08>
   3d8e0:	pxor   %xmm1,%xmm1
   3d8e4:	pxor   %xmm0,%xmm0
   3d8e8:	nopl   0x0(%rax,%rax,1)
   3d8f0:	movq   (%r15,%rax,4),%xmm4
   3d8f6:	movq   0x8(%r15,%rax,4),%xmm5
   3d8fd:	pshufd $0xd4,%xmm4,%xmm4
   3d902:	pcmpeqd %xmm2,%xmm4
   3d906:	pandn  %xmm3,%xmm4
   3d90a:	paddq  %xmm1,%xmm4
   3d90e:	pshufd $0xd4,%xmm5,%xmm5
   3d913:	pcmpeqd %xmm2,%xmm5
   3d917:	pandn  %xmm3,%xmm5
   3d91b:	paddq  %xmm0,%xmm5
   3d91f:	movq   0x10(%r15,%rax,4),%xmm0
   3d926:	movq   0x18(%r15,%rax,4),%xmm6
   3d92d:	pshufd $0xd4,%xmm0,%xmm1
   3d932:	pcmpeqd %xmm2,%xmm1
   3d936:	pandn  %xmm3,%xmm1
   3d93a:	paddq  %xmm4,%xmm1
   3d93e:	pshufd $0xd4,%xmm6,%xmm0
   3d943:	pcmpeqd %xmm2,%xmm0
   3d947:	pandn  %xmm3,%xmm0
   3d94b:	paddq  %xmm5,%xmm0
   3d94f:	add    $0x8,%rax
   3d953:	cmp    $0x100,%rax
   3d959:	jne    3d8f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
   3d95b:	mov    $0x88888889,%ebp
   3d960:	imul   %r13,%rbp
   3d964:	shr    $0x24,%rbp
   3d968:	imul   $0xeeeeeeef,%r13d,%eax
   3d96f:	ror    $1,%eax
   3d971:	cmp    $0x8888889,%eax
   3d976:	sbb    $0xffffffffffffffff,%rbp
   3d97a:	mov    %rbp,%rbx
   3d97d:	cmpl   $0x0,0x1c(%rsp)
   3d982:	jne    3d9d8 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3d984:	test   %r13,%r13
   3d987:	je     3d9d6 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   3d989:	paddq  %xmm1,%xmm0
   3d98d:	pshufd $0xee,%xmm0,%xmm1
   3d992:	paddq  %xmm0,%xmm1
   3d996:	movq   %xmm1,%rax
   3d99b:	lea    -0x1(%r13),%ecx
   3d99f:	mov    $0x88888889,%edx
   3d9a4:	imul   %rcx,%rdx
   3d9a8:	shr    $0x23,%rdx
   3d9ac:	and    $0xfffffffe,%edx
   3d9af:	lea    (%rdx,%r13,1),%ecx
   3d9b3:	dec    %ecx
   3d9b5:	shr    $1,%ecx
   3d9b7:	mov    $0x80010003,%ebx
   3d9bc:	imul   %rcx,%rbx
   3d9c0:	shr    $0x2e,%rbx
   3d9c4:	inc    %ebx
   3d9c6:	imul   %rax,%rbx
   3d9ca:	dec    %rbx
   3d9cd:	cmp    %rbp,%rbx
   3d9d0:	cmovae %rbp,%rbx
   3d9d4:	jmp    3d9d8 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3d9d6:	xor    %ebx,%ebx
   3d9d8:	mov    (%r9),%rax
   3d9db:	lea    0x1(%rbp),%rcx
   3d9df:	mov    %rcx,0x80(%rsp)
   3d9e7:	cmp    %rbp,%rax
   3d9ea:	mov    %rsi,0x28(%rsp)
   3d9ef:	jbe    3eeaf <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
   3d9f5:	mov    0x5520(%rsp),%rax
   3d9fd:	mov    (%rax),%rax
   3da00:	cmp    %rbx,%rax
   3da03:	jb     3ef04 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
   3da09:	movq   $0x0,0x10(%rdi)
   3da11:	lea    0x3f(%rbp),%rbx
   3da15:	shr    $0x6,%rbx
   3da19:	mov    %r13,0x8(%rsp)
   3da1e:	mov    %rdi,0x58(%rsp)
   3da23:	mov    %rbp,0x60(%rsp)
   3da28:	je     3da82 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
   3da2a:	cmp    (%rdi),%rbx
   3da2d:	ja     3f033 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   3da33:	mov    %r9,%r13
   3da36:	xor    %r14d,%r14d
   3da39:	mov    0x8(%rdi),%r12
   3da3d:	lea    (%r12,%r14,8),%rdi
   3da41:	cmp    $0x1,%rbx
   3da45:	je     3da69 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3da47:	lea    -0x8(,%rbx,8),%rdx
   3da4f:	xor    %esi,%esi
   3da51:	call   *0x10fb99(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   3da57:	lea    (%r14,%rbx,1),%rax
   3da5b:	lea    (%r12,%rax,8),%rdi
   3da5f:	add    $0xfffffffffffffff8,%rdi
   3da63:	add    %rbx,%r14
   3da66:	dec    %r14
   3da69:	mov    %r13,%r9
   3da6c:	movq   $0x0,(%rdi)
   3da73:	inc    %r14
   3da76:	mov    0x8(%rsp),%r13
   3da7b:	mov    0x58(%rsp),%rdi
   3da80:	jmp    3da85 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
   3da82:	xor    %r14d,%r14d
   3da85:	lea    0x18(%rdi),%r12
   3da89:	mov    %r14,0x10(%rdi)
   3da8d:	movq   $0x0,0x28(%rdi)
   3da95:	cmpq   $0xf,0x18(%rdi)
   3da9a:	jbe    3f008 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   3daa0:	lea    0x30(%rdi),%rax
   3daa4:	mov    %rax,(%rsp)
   3daa8:	xor    %eax,%eax
   3daaa:	pxor   %xmm0,%xmm0
   3daae:	movabs $0xffffffff00000000,%rcx
   3dab8:	nopl   0x0(%rax,%rax,1)
   3dac0:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   3dac9:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3dad2:	mov    %rcx,0x2d08(%rsp,%rax,1)
   3dada:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   3dae3:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3daec:	mov    %rcx,0x2d30(%rsp,%rax,1)
   3daf4:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3dafd:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   3db06:	mov    %rcx,0x2d58(%rsp,%rax,1)
   3db0e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   3db17:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   3db20:	mov    %rcx,0x2d80(%rsp,%rax,1)
   3db28:	add    $0xa0,%rax
   3db2e:	cmp    $0x2800,%rax
   3db34:	jne    3dac0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   3db36:	mov    %r12,%r14
   3db39:	lea    0xc(%r15),%rax
   3db3d:	xor    %edx,%edx
   3db3f:	mov    $0x25,%ecx
   3db44:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3db50:	mov    -0xc(%rax),%esi
   3db53:	add    %edx,%esi
   3db55:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   3db5c:	mov    %esi,0x2c70(%rsp,%rcx,4)
   3db63:	mov    -0x8(%rax),%edx
   3db66:	add    %esi,%edx
   3db68:	mov    %esi,0x2c94(%rsp,%rcx,4)
   3db6f:	mov    %edx,0x2c98(%rsp,%rcx,4)
   3db76:	mov    -0x4(%rax),%esi
   3db79:	add    %edx,%esi
   3db7b:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   3db82:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   3db89:	mov    (%rax),%edx
   3db8b:	add    %esi,%edx
   3db8d:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3db94:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3db9b:	add    $0x28,%rcx
   3db9f:	add    $0x10,%rax
   3dba3:	cmp    $0xa25,%rcx
   3dbaa:	jne    3db50 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
   3dbac:	mov    %r9,0x78(%rsp)
   3dbb1:	mov    0x8(%r9),%rbx
   3dbb5:	mov    0x8(%rdi),%rbp
   3dbb9:	mov    0x10(%rdi),%r12
   3dbbd:	lea    0xd8(%rsp),%rdi
   3dbc5:	lea    0x2ce8(%rsp),%rsi
   3dbcd:	mov    $0x2800,%edx
   3dbd2:	call   *0x10fa38(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   3dbd8:	mov    %rbp,%r8
   3dbdb:	mov    %r12,%rsi
   3dbde:	mov    %rbx,0xc0(%rsp)
   3dbe6:	mov    0x60(%rsp),%rax
   3dbeb:	mov    %rax,0xc8(%rsp)
   3dbf3:	mov    %r13,0xd0(%rsp)
   3dbfb:	mov    0x5520(%rsp),%rax
   3dc03:	mov    %rax,0x98(%rsp)
   3dc0b:	mov    %rbp,0xa0(%rsp)
   3dc13:	mov    %r12,0xa8(%rsp)
   3dc1b:	mov    %r14,0xb0(%rsp)
   3dc23:	movl   $0x0,0x28e0(%rsp)
   3dc2e:	movq   $0x0,0x28d8(%rsp)
   3dc3a:	mov    (%rsp),%rax
   3dc3e:	mov    %rax,0xb8(%rsp)
   3dc46:	mov    $0xffffffffffffffff,%rcx
   3dc4d:	xor    %ebp,%ebp
   3dc4f:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3dc59:	movabs $0x888888888888889,%r10
   3dc63:	pxor   %xmm0,%xmm0
   3dc67:	pcmpeqd %xmm1,%xmm1
   3dc6b:	xor    %r12d,%r12d
   3dc6e:	jmp    3dd26 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   3dc73:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3dc80:	mov    0x8(%rdi),%rax
   3dc84:	mov    %r13,%rcx
   3dc87:	shl    $0x7,%rcx
   3dc8b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3dc91:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3dc97:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3dc9d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3dca3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3dca9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3dcaf:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3dcb5:	movdqa %xmm0,(%rax,%rcx,1)
   3dcba:	mov    %r13,%rax
   3dcbd:	inc    %rax
   3dcc0:	mov    %rax,0x10(%rdi)
   3dcc4:	mov    %r14,%rax
   3dcc7:	shl    $0x6,%rax
   3dccb:	or     $0x10000000,%r13d
   3dcd2:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   3dcd8:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3dcde:	movdqa %xmm1,0x10(%rbx,%rax,1)
   3dce4:	movdqa %xmm1,(%rbx,%rax,1)
   3dce9:	mov    %r13d,0x3c(%rbx,%rax,1)
   3dcee:	mov    %r14,%rdi
   3dcf1:	shr    $0x6,%rdi
   3dcf5:	cmp    %rsi,%rdi
   3dcf8:	jae    3f09a <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   3dcfe:	mov    $0x1,%eax
   3dd03:	mov    %r14d,%ecx
   3dd06:	shl    %cl,%rax
   3dd09:	or     %rax,(%r8,%rdi,8)
   3dd0d:	mov    %r14,%rcx
   3dd10:	mov    0x8(%rsp),%r13
   3dd15:	add    $0x4,%r12
   3dd19:	cmp    $0x400,%r12
   3dd20:	je     3ddc0 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
   3dd26:	mov    %rbp,%rax
   3dd29:	mov    (%r15,%r12,1),%ebp
   3dd2d:	add    %rax,%rbp
   3dd30:	cmp    %r13,%rbp
   3dd33:	setae  %dil
   3dd37:	mov    %rbp,%rax
   3dd3a:	movabs $0x8888888888888889,%rdx
   3dd44:	mul    %rdx
   3dd47:	mov    %rdx,%r14
   3dd4a:	shr    $0x4,%r14
   3dd4e:	mov    %rbp,%rax
   3dd51:	imul   %r9,%rax
   3dd55:	ror    $1,%rax
   3dd58:	cmp    %r10,%rax
   3dd5b:	setb   %al
   3dd5e:	cmp    %rcx,%r14
   3dd61:	sete   %dl
   3dd64:	or     %dil,%dl
   3dd67:	or     %al,%dl
   3dd69:	jne    3dd15 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   3dd6b:	mov    0x5520(%rsp),%rdi
   3dd73:	mov    0x10(%rdi),%r13
   3dd77:	cmp    (%rdi),%r13
   3dd7a:	jne    3dc80 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3dd80:	mov    %rsi,(%rsp)
   3dd84:	mov    %r8,0x10(%rsp)
   3dd89:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3dd8e:	pcmpeqd %xmm1,%xmm1
   3dd92:	pxor   %xmm0,%xmm0
   3dd96:	movabs $0x888888888888889,%r10
   3dda0:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3ddaa:	mov    0x10(%rsp),%r8
   3ddaf:	mov    (%rsp),%rsi
   3ddb3:	mov    0x5520(%rsp),%rdi
   3ddbb:	jmp    3dc80 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3ddc0:	cmpl   $0x0,0x1c(%rsp)
   3ddc5:	lea    0xd8(%rsp),%r8
   3ddcd:	je     3df84 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3ddd3:	mov    0xc8(%rsp),%r11
   3dddb:	test   %r11,%r11
   3ddde:	je     3df84 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3dde4:	mov    0x8(%rsp),%r9
   3dde9:	shl    $0x20,%r9
   3dded:	mov    0x1c(%rsp),%r10d
   3ddf2:	mov    0x98(%rsp),%rdi
   3ddfa:	mov    0xc0(%rsp),%r13
   3de02:	xor    %r12d,%r12d
   3de05:	movabs $0xff51afd7ed558ccd,%rbx
   3de0f:	pxor   %xmm0,%xmm0
   3de13:	pcmpeqd %xmm1,%xmm1
   3de17:	jmp    3dea0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
   3de1c:	mov    0x8(%rdi),%rax
   3de20:	mov    %rbx,%rcx
   3de23:	shl    $0x7,%rcx
   3de27:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3de2d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3de33:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3de39:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3de3f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3de45:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3de4b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3de51:	movdqa %xmm0,(%rax,%rcx,1)
   3de56:	mov    %rbx,%rax
   3de59:	inc    %rax
   3de5c:	mov    %rax,0x10(%rdi)
   3de60:	or     $0x10000000,%ebx
   3de66:	movdqu %xmm1,0x2c(%r13)
   3de6c:	movdqa %xmm1,0x20(%r13)
   3de72:	movdqa %xmm1,0x10(%r13)
   3de78:	movdqa %xmm1,0x0(%r13)
   3de7e:	mov    %ebx,0x3c(%r13)
   3de82:	or     %r15,(%r14,%rbp,8)
   3de86:	movabs $0xff51afd7ed558ccd,%rbx
   3de90:	inc    %r12
   3de93:	add    $0x40,%r13
   3de97:	cmp    %r12,%r11
   3de9a:	je     3df84 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3dea0:	mov    %r12,%rbp
   3dea3:	shr    $0x6,%rbp
   3dea7:	mov    0xa8(%rsp),%rsi
   3deaf:	cmp    %rsi,%rbp
   3deb2:	jae    3f0a6 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   3deb8:	mov    0xa0(%rsp),%r14
   3dec0:	mov    (%r14,%rbp,8),%rax
   3dec4:	mov    $0x1,%r15d
   3deca:	mov    %r12d,%ecx
   3decd:	shl    %cl,%r15
   3ded0:	bt     %r12,%rax
   3ded4:	jb     3de90 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3ded6:	mov    %r12,%rax
   3ded9:	xor    %r9,%rax
   3dedc:	mov    %rax,%rcx
   3dedf:	shr    $0x21,%rcx
   3dee3:	xor    %rax,%rcx
   3dee6:	imul   %rbx,%rcx
   3deea:	mov    %rcx,%rax
   3deed:	shr    $0x21,%rax
   3def1:	xor    %rcx,%rax
   3def4:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3defe:	imul   %rcx,%rax
   3df02:	mov    %rax,%rcx
   3df05:	shr    $0x21,%rcx
   3df09:	xor    %rax,%rcx
   3df0c:	mov    %rcx,%rax
   3df0f:	shr    $0x3,%rax
   3df13:	movabs $0x20c49ba5e353f7cf,%rdx
   3df1d:	mul    %rdx
   3df20:	shr    $0x4,%rdx
   3df24:	imul   $0x3e8,%rdx,%rax
   3df2b:	sub    %rax,%rcx
   3df2e:	cmp    %r10,%rcx
   3df31:	jae    3de90 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3df37:	mov    0x10(%rdi),%rbx
   3df3b:	cmp    (%rdi),%rbx
   3df3e:	jne    3de1c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3df44:	mov    %r11,(%rsp)
   3df48:	mov    %r9,0x10(%rsp)
   3df4d:	mov    %r10,0x50(%rsp)
   3df52:	mov    %rdi,0x48(%rsp)
   3df57:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3df5c:	pcmpeqd %xmm1,%xmm1
   3df60:	pxor   %xmm0,%xmm0
   3df64:	mov    0x48(%rsp),%rdi
   3df69:	mov    0x50(%rsp),%r10
   3df6e:	mov    0x10(%rsp),%r9
   3df73:	mov    (%rsp),%r11
   3df77:	lea    0xd8(%rsp),%r8
   3df7f:	jmp    3de1c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3df84:	mov    0x8(%rsp),%r13
   3df89:	test   %r13,%r13
   3df8c:	mov    0x28(%rsp),%r14
   3df91:	je     3ecdf <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3df97:	xor    %r11d,%r11d
   3df9a:	jmp    3dfac <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3df9c:	nopl   0x0(%rax)
   3dfa0:	mov    %r11,%r10
   3dfa3:	cmp    %r13,%r10
   3dfa6:	jae    3ecdf <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3dfac:	mov    %r11,%rbp
   3dfaf:	mov    %r9,%r12
   3dfb2:	mov    $0x400,%edx
   3dfb7:	lea    0x28e8(%rsp),%rdi
   3dfbf:	xor    %esi,%esi
   3dfc1:	call   *0x10f629(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   3dfc7:	lea    0x118(%rsp),%rax
   3dfcf:	xor    %ecx,%ecx
   3dfd1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3dfe0:	mov    -0x40(%rax),%rdx
   3dfe4:	sub    -0x30(%rax),%rdx
   3dfe8:	shr    $1,%rdx
   3dfeb:	add    -0x28(%rax),%edx
   3dfee:	mov    -0x18(%rax),%rsi
   3dff2:	sub    -0x8(%rax),%rsi
   3dff6:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3dffd:	shr    $1,%rsi
   3e000:	add    (%rax),%esi
   3e002:	mov    %esi,0x28ec(%rsp,%rcx,4)
   3e009:	add    $0x2,%rcx
   3e00d:	add    $0x50,%rax
   3e011:	cmp    $0x100,%rcx
   3e018:	jne    3dfe0 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
   3e01a:	mov    0xb0(%rsp),%r15
   3e022:	mov    0x10(%r15),%rbx
   3e026:	cmp    (%r15),%rbx
   3e029:	je     3ecd2 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
   3e02f:	mov    %rbx,%rdi
   3e032:	shl    $0xa,%rdi
   3e036:	add    0x8(%r15),%rdi
   3e03a:	mov    $0x400,%edx
   3e03f:	lea    0x28e8(%rsp),%rsi
   3e047:	call   *0x10f5c3(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   3e04d:	inc    %rbx
   3e050:	mov    %rbx,0x10(%r15)
   3e054:	mov    %rbp,%r11
   3e057:	lea    0xf000(%rbp),%r10
   3e05e:	cmp    %r13,%r10
   3e061:	cmovae %r13,%r10
   3e065:	lea    0x10(%rbp),%rdi
   3e069:	cmp    %r10,%rdi
   3e06c:	mov    %r10,0x20(%rsp)
   3e071:	jbe    3e140 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   3e077:	lea    0xd8(%rsp),%r8
   3e07f:	mov    %r12,%r9
   3e082:	mov    0x38(%rsp),%rcx
   3e087:	mov    %r10,%rdx
   3e08a:	sub    %r11,%rdx
   3e08d:	mov    %rcx,0x38(%rsp)
   3e092:	jbe    3dfa0 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   3e098:	mov    %r9,%rbp
   3e09b:	mov    %r14,%rax
   3e09e:	mov    0x28e0(%rsp),%r12d
   3e0a6:	mov    $0x1e,%r14d
   3e0ac:	sub    0x28d8(%rsp),%r14
   3e0b4:	cmp    %rdx,%r14
   3e0b7:	cmovae %rdx,%r14
   3e0bb:	lea    (%rax,%r11,1),%r15
   3e0bf:	test   %r14,%r14
   3e0c2:	je     3eb40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3e0c8:	xor    %r13d,%r13d
   3e0cb:	nopl   0x0(%rax,%rax,1)
   3e0d0:	movzbl (%r15,%r13,1),%esi
   3e0d5:	lea    (%rsi,%rsi,4),%rcx
   3e0d9:	lea    (%r8,%rcx,8),%rbx
   3e0dd:	mov    (%r8,%rcx,8),%rax
   3e0e1:	cmp    0x8(%r8,%rcx,8),%rax
   3e0e6:	je     3e100 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   3e0e8:	lea    (%r12,%r13,1),%rcx
   3e0ec:	mov    %cx,(%rax)
   3e0ef:	addq   $0x2,(%rbx)
   3e0f3:	inc    %r13
   3e0f6:	cmp    %r13,%r14
   3e0f9:	jne    3e0d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   3e0fb:	jmp    3eb40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3e100:	mov    %rdx,0x10(%rsp)
   3e105:	mov    %r11,(%rsp)
   3e109:	lea    0x98(%rsp),%rdi
   3e111:	call   93020 <<c30walk::ctor2::W<false>>::refill>
   3e116:	lea    0xd8(%rsp),%r8
   3e11e:	test   %al,%al
   3e120:	je     3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3e126:	mov    (%rbx),%rax
   3e129:	mov    0x20(%rsp),%r10
   3e12e:	mov    (%rsp),%r11
   3e132:	mov    0x10(%rsp),%rdx
   3e137:	jmp    3e0e8 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
   3e139:	nopl   0x0(%rax)
   3e140:	xor    %edx,%edx
   3e142:	lea    0xd8(%rsp),%r8
   3e14a:	mov    %r12,%r9
   3e14d:	mov    0x38(%rsp),%rcx
   3e152:	jmp    3e1b3 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   3e154:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e160:	mov    0x28d8(%rsp),%rax
   3e168:	lea    0x10(%rax),%rcx
   3e16c:	add    $0xfffffffffffffff2,%rax
   3e170:	xor    %edx,%edx
   3e172:	cmp    $0x1e,%rcx
   3e176:	setae  %dl
   3e179:	cmovb  %rcx,%rax
   3e17d:	mov    %rax,0x28d8(%rsp)
   3e185:	mov    0x28e0(%rsp),%eax
   3e18c:	lea    0x10(%rax,%rdx,2),%eax
   3e190:	mov    %eax,0x28e0(%rsp)
   3e197:	mov    0x30(%rsp),%r11
   3e19c:	lea    0x10(%r11),%rsi
   3e1a0:	xor    %edx,%edx
   3e1a2:	mov    %rsi,%rdi
   3e1a5:	cmp    %r10,%rsi
   3e1a8:	mov    0x28(%rsp),%r14
   3e1ad:	ja     3e087 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
   3e1b3:	test   $0x1,%dl
   3e1b6:	jne    3e1e4 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
   3e1b8:	lea    0x8(%r11),%rsi
   3e1bc:	cmp    $0xfffffffffffffff7,%r11
   3e1c0:	ja     3ef68 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3e1c6:	cmp    %r13,%rsi
   3e1c9:	ja     3ef68 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3e1cf:	lea    0x10(%r11),%rax
   3e1d3:	cmp    %r13,%rax
   3e1d6:	ja     3ef8f <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
   3e1dc:	mov    (%r14,%r11,1),%r9
   3e1e0:	mov    (%r14,%rsi,1),%rcx
   3e1e4:	mov    %r9,%rax
   3e1e7:	xor    %rcx,%rax
   3e1ea:	mov    %r9,%rdx
   3e1ed:	rol    $0x8,%rdx
   3e1f1:	xor    %r9,%rdx
   3e1f4:	or     %rax,%rdx
   3e1f7:	jne    3e250 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   3e1f9:	mov    %rdi,%rax
   3e1fc:	nopl   0x0(%rax)
   3e200:	mov    %rax,%r12
   3e203:	add    $0x10,%rax
   3e207:	cmp    %r10,%rax
   3e20a:	setbe  %r9b
   3e20e:	ja     3e2b0 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
   3e214:	lea    0x8(%r12),%rsi
   3e219:	cmp    $0xfffffffffffffff7,%r12
   3e21d:	ja     3ee8a <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3e223:	cmp    %r13,%rsi
   3e226:	ja     3ee8a <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3e22c:	mov    (%r14,%r12,1),%rdx
   3e230:	mov    0x8(%r14,%r12,1),%rsi
   3e235:	cmp    %rcx,%rdx
   3e238:	jne    3e23f <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   3e23a:	cmp    %rcx,%rsi
   3e23d:	je     3e200 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   3e23f:	mov    %rsi,0x38(%rsp)
   3e244:	mov    %rdx,0x68(%rsp)
   3e249:	lea    0x10(%r12),%r10
   3e24e:	jmp    3e2b5 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   3e250:	mov    %rdi,0x30(%rsp)
   3e255:	mov    0x28d8(%rsp),%rax
   3e25d:	mov    $0x1e,%r12d
   3e263:	sub    %rax,%r12
   3e266:	cmp    $0x10,%r12
   3e26a:	mov    $0x10,%ebx
   3e26f:	cmovb  %r12,%rbx
   3e273:	mov    %rbx,%r15
   3e276:	add    %r11,%r15
   3e279:	jb     3ef7a <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3e27f:	cmp    %r13,%r15
   3e282:	ja     3ef7a <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3e288:	mov    0x28e0(%rsp),%r14d
   3e290:	cmp    $0x1e,%rax
   3e294:	jne    3ea10 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
   3e29a:	cmp    %r15,0x30(%rsp)
   3e29f:	jae    3ea9a <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   3e2a5:	jmp    3efa4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3e2aa:	nopw   0x0(%rax,%rax,1)
   3e2b0:	lea    0x10(%r12),%r10
   3e2b5:	mov    %r12,0x30(%rsp)
   3e2ba:	sub    %r11,%r12
   3e2bd:	movzbl %cl,%eax
   3e2c0:	mov    %rax,0x90(%rsp)
   3e2c8:	lea    (%rax,%rax,4),%rdx
   3e2cc:	mov    (%r8,%rdx,8),%rax
   3e2d0:	mov    0x1c(%r8,%rdx,8),%esi
   3e2d5:	mov    %rax,%rdi
   3e2d8:	sub    0x10(%r8,%rdx,8),%rdi
   3e2dd:	shr    $1,%rdi
   3e2e0:	add    0x18(%r8,%rdx,8),%edi
   3e2e5:	add    %r12,%rdi
   3e2e8:	cmp    %rsi,%rdi
   3e2eb:	ja     3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3e2f1:	mov    %r10,0x70(%rsp)
   3e2f6:	mov    %r9d,0x44(%rsp)
   3e2fb:	lea    (%r8,%rdx,8),%rbp
   3e2ff:	movzbl %cl,%r13d
   3e303:	mov    0x8(%rbp),%rsi
   3e307:	mov    $0x88888889,%r14d
   3e30d:	lea    -0x230d4(%rip),%r15        # 1b240 <c30walk::ctor2::PATTERNS>
   3e314:	mov    %rbp,0x88(%rsp)
   3e31c:	jmp    3e357 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   3e31e:	xchg   %ax,%ax
   3e320:	mov    0x3c(%rsi),%edi
   3e323:	mov    $0xfffff,%r9d
   3e329:	and    %r9d,%edi
   3e32c:	sub    %r8d,%ecx
   3e32f:	sub    %eax,%r8d
   3e332:	mov    %r8d,%eax
   3e335:	lea    (%rsi,%rax,2),%rax
   3e339:	mov    %rax,0x10(%rbp)
   3e33d:	mov    %rax,0x0(%rbp)
   3e341:	lea    (%rax,%rcx,2),%rsi
   3e345:	mov    %rsi,0x8(%rbp)
   3e349:	mov    %edx,0x20(%rbp)
   3e34c:	mov    %edi,0x24(%rbp)
   3e34f:	lea    0xd8(%rsp),%r8
   3e357:	sub    %rax,%rsi
   3e35a:	shr    $1,%rsi
   3e35d:	cmp    %rsi,%r12
   3e360:	cmovb  %r12,%rsi
   3e364:	test   %rsi,%rsi
   3e367:	je     3e421 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
   3e36d:	mov    0x28d8(%rsp),%rdi
   3e375:	cmp    $0x1d,%rdi
   3e379:	ja     3f078 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   3e37f:	cmp    $0x1f,%rsi
   3e383:	jae    3ee9c <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
   3e389:	mov    0x28e0(%rsp),%ecx
   3e390:	mov    %rdi,%r11
   3e393:	shl    $0x6,%r11
   3e397:	lea    (%r15,%r11,1),%rdx
   3e39b:	lea    -0x2(,%rsi,2),%r9
   3e3a3:	cmp    $0xe,%r9
   3e3a7:	jb     3e3bc <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   3e3a9:	mov    %rax,%r8
   3e3ac:	sub    %r15,%r8
   3e3af:	sub    %r11,%r8
   3e3b2:	cmp    $0x10,%r8
   3e3b6:	jae    3e823 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
   3e3bc:	xor    %r10d,%r10d
   3e3bf:	mov    %rdx,%r8
   3e3c2:	lea    (%rdx,%rsi,2),%rdx
   3e3c6:	lea    (%rax,%r10,2),%r9
   3e3ca:	nopw   0x0(%rax,%rax,1)
   3e3d0:	movzwl (%r8),%r10d
   3e3d4:	add    %cx,%r10w
   3e3d8:	add    $0x2,%r8
   3e3dc:	mov    %r10w,(%r9)
   3e3e0:	add    $0x2,%r9
   3e3e4:	cmp    %rdx,%r8
   3e3e7:	jne    3e3d0 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
   3e3e9:	lea    (%rax,%rsi,2),%rax
   3e3ed:	mov    %rax,0x0(%rbp)
   3e3f1:	add    %rsi,%rdi
   3e3f4:	lea    0x2(%rsi),%edx
   3e3f7:	mov    %rdi,%r8
   3e3fa:	sub    $0x1e,%r8
   3e3fe:	cmovb  %rdi,%r8
   3e402:	cmovb  %esi,%edx
   3e405:	mov    %r8,0x28d8(%rsp)
   3e40d:	add    %ecx,%edx
   3e40f:	mov    %edx,0x28e0(%rsp)
   3e416:	sub    %rsi,%r12
   3e419:	lea    0xd8(%rsp),%r8
   3e421:	test   %r12,%r12
   3e424:	je     3ea00 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   3e42a:	mov    %rax,%r8
   3e42d:	cmp    0x10(%rbp),%rax
   3e431:	je     3e450 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
   3e433:	lea    0x98(%rsp),%rdi
   3e43b:	mov    0x90(%rsp),%rsi
   3e443:	call   92a20 <<c30walk::ctor2::W<false>>::finish>
   3e448:	mov    0x0(%rbp),%r8
   3e44c:	mov    0x10(%rbp),%rax
   3e450:	sub    %rax,%r8
   3e453:	shr    $1,%r8
   3e456:	add    0x18(%rbp),%r8d
   3e45a:	mov    %r8d,0x18(%rbp)
   3e45e:	pxor   %xmm0,%xmm0
   3e462:	movdqu %xmm0,0x0(%rbp)
   3e467:	movq   $0x0,0x10(%rbp)
   3e46f:	cmp    $0x1e,%r12
   3e473:	jb     3e994 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3e479:	mov    %r8,%rcx
   3e47c:	imul   %r14,%rcx
   3e480:	shr    $0x24,%rcx
   3e484:	mov    %ecx,%eax
   3e486:	shl    $0x5,%eax
   3e489:	lea    (%rcx,%rcx,1),%edx
   3e48c:	sub    %eax,%edx
   3e48e:	add    %r8d,%edx
   3e491:	jne    3e994 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3e497:	mov    %r12,%rax
   3e49a:	movabs $0x8888888888888889,%rdx
   3e4a4:	mul    %rdx
   3e4a7:	shr    $0x4,%rdx
   3e4ab:	mov    %ecx,%r9d
   3e4ae:	cmpl   $0x0,0x1c(%rsp)
   3e4b3:	mov    %r12,0x50(%rsp)
   3e4b8:	mov    %rdx,0x48(%rsp)
   3e4bd:	je     3e7e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
   3e4c3:	mov    0x28e0(%rsp),%r10d
   3e4cb:	mov    0xa8(%rsp),%rax
   3e4d3:	mov    %rax,(%rsp)
   3e4d7:	mov    0x98(%rsp),%r11
   3e4df:	mov    0xa0(%rsp),%rax
   3e4e7:	mov    %rax,0x10(%rsp)
   3e4ec:	mov    0xc0(%rsp),%rax
   3e4f4:	mov    0x28d8(%rsp),%rbx
   3e4fc:	mov    %rbx,%rdi
   3e4ff:	shl    $0x6,%rdi
   3e503:	add    %r15,%rdi
   3e506:	mov    %r9,%rcx
   3e509:	shl    $0x6,%rcx
   3e50d:	lea    (%rax,%rcx,1),%r12
   3e511:	add    $0x3c,%r12
   3e515:	mov    %r10d,%r15d
   3e518:	shl    $0x8,%r15d
   3e51c:	or     %r13d,%r15d
   3e51f:	xor    %ebp,%ebp
   3e521:	jmp    3e751 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
   3e526:	cs nopw 0x0(%rax,%rax,1)
   3e530:	mov    0x8(%r11),%rcx
   3e534:	shl    $0x7,%eax
   3e537:	mov    %r15d,(%rcx,%rax,1)
   3e53b:	movzwl 0x2(%rdi),%esi
   3e53f:	lea    (%r10,%rbp,1),%r14d
   3e543:	add    %r14d,%esi
   3e546:	shl    $0x8,%esi
   3e549:	or     %r13d,%esi
   3e54c:	mov    %esi,0x4(%rcx,%rax,1)
   3e550:	movzwl 0x4(%rdi),%esi
   3e554:	add    %r14d,%esi
   3e557:	shl    $0x8,%esi
   3e55a:	or     %r13d,%esi
   3e55d:	mov    %esi,0x8(%rcx,%rax,1)
   3e561:	movzwl 0x6(%rdi),%esi
   3e565:	add    %r14d,%esi
   3e568:	shl    $0x8,%esi
   3e56b:	or     %r13d,%esi
   3e56e:	mov    %esi,0xc(%rcx,%rax,1)
   3e572:	movzwl 0x8(%rdi),%esi
   3e576:	add    %r14d,%esi
   3e579:	shl    $0x8,%esi
   3e57c:	or     %r13d,%esi
   3e57f:	mov    %esi,0x10(%rcx,%rax,1)
   3e583:	movzwl 0xa(%rdi),%esi
   3e587:	add    %r14d,%esi
   3e58a:	shl    $0x8,%esi
   3e58d:	or     %r13d,%esi
   3e590:	mov    %esi,0x14(%rcx,%rax,1)
   3e594:	movzwl 0xc(%rdi),%esi
   3e598:	add    %r14d,%esi
   3e59b:	shl    $0x8,%esi
   3e59e:	or     %r13d,%esi
   3e5a1:	mov    %esi,0x18(%rcx,%rax,1)
   3e5a5:	movzwl 0xe(%rdi),%esi
   3e5a9:	add    %r14d,%esi
   3e5ac:	shl    $0x8,%esi
   3e5af:	or     %r13d,%esi
   3e5b2:	mov    %esi,0x1c(%rcx,%rax,1)
   3e5b6:	movzwl 0x10(%rdi),%esi
   3e5ba:	add    %r14d,%esi
   3e5bd:	shl    $0x8,%esi
   3e5c0:	or     %r13d,%esi
   3e5c3:	mov    %esi,0x20(%rcx,%rax,1)
   3e5c7:	movzwl 0x12(%rdi),%esi
   3e5cb:	add    %r14d,%esi
   3e5ce:	shl    $0x8,%esi
   3e5d1:	or     %r13d,%esi
   3e5d4:	mov    %esi,0x24(%rcx,%rax,1)
   3e5d8:	movzwl 0x14(%rdi),%esi
   3e5dc:	add    %r14d,%esi
   3e5df:	shl    $0x8,%esi
   3e5e2:	or     %r13d,%esi
   3e5e5:	mov    %esi,0x28(%rcx,%rax,1)
   3e5e9:	movzwl 0x16(%rdi),%esi
   3e5ed:	add    %r14d,%esi
   3e5f0:	shl    $0x8,%esi
   3e5f3:	or     %r13d,%esi
   3e5f6:	mov    %esi,0x2c(%rcx,%rax,1)
   3e5fa:	movzwl 0x18(%rdi),%esi
   3e5fe:	add    %r14d,%esi
   3e601:	shl    $0x8,%esi
   3e604:	or     %r13d,%esi
   3e607:	mov    %esi,0x30(%rcx,%rax,1)
   3e60b:	movzwl 0x1a(%rdi),%esi
   3e60f:	add    %r14d,%esi
   3e612:	shl    $0x8,%esi
   3e615:	or     %r13d,%esi
   3e618:	mov    %esi,0x34(%rcx,%rax,1)
   3e61c:	movzwl 0x1c(%rdi),%esi
   3e620:	add    %r14d,%esi
   3e623:	shl    $0x8,%esi
   3e626:	or     %r13d,%esi
   3e629:	mov    %esi,0x38(%rcx,%rax,1)
   3e62d:	movzwl 0x1e(%rdi),%esi
   3e631:	add    %r14d,%esi
   3e634:	shl    $0x8,%esi
   3e637:	or     %r13d,%esi
   3e63a:	mov    %esi,0x3c(%rcx,%rax,1)
   3e63e:	movzwl 0x20(%rdi),%esi
   3e642:	add    %r14d,%esi
   3e645:	shl    $0x8,%esi
   3e648:	or     %r13d,%esi
   3e64b:	mov    %esi,0x40(%rcx,%rax,1)
   3e64f:	movzwl 0x22(%rdi),%esi
   3e653:	add    %r14d,%esi
   3e656:	shl    $0x8,%esi
   3e659:	or     %r13d,%esi
   3e65c:	mov    %esi,0x44(%rcx,%rax,1)
   3e660:	movzwl 0x24(%rdi),%esi
   3e664:	add    %r14d,%esi
   3e667:	shl    $0x8,%esi
   3e66a:	or     %r13d,%esi
   3e66d:	mov    %esi,0x48(%rcx,%rax,1)
   3e671:	movzwl 0x26(%rdi),%esi
   3e675:	add    %r14d,%esi
   3e678:	shl    $0x8,%esi
   3e67b:	or     %r13d,%esi
   3e67e:	mov    %esi,0x4c(%rcx,%rax,1)
   3e682:	movzwl 0x28(%rdi),%esi
   3e686:	add    %r14d,%esi
   3e689:	shl    $0x8,%esi
   3e68c:	or     %r13d,%esi
   3e68f:	mov    %esi,0x50(%rcx,%rax,1)
   3e693:	movzwl 0x2a(%rdi),%esi
   3e697:	add    %r14d,%esi
   3e69a:	shl    $0x8,%esi
   3e69d:	or     %r13d,%esi
   3e6a0:	mov    %esi,0x54(%rcx,%rax,1)
   3e6a4:	movzwl 0x2c(%rdi),%esi
   3e6a8:	add    %r14d,%esi
   3e6ab:	shl    $0x8,%esi
   3e6ae:	or     %r13d,%esi
   3e6b1:	mov    %esi,0x58(%rcx,%rax,1)
   3e6b5:	movzwl 0x2e(%rdi),%esi
   3e6b9:	add    %r14d,%esi
   3e6bc:	shl    $0x8,%esi
   3e6bf:	or     %r13d,%esi
   3e6c2:	mov    %esi,0x5c(%rcx,%rax,1)
   3e6c6:	movzwl 0x30(%rdi),%esi
   3e6ca:	add    %r14d,%esi
   3e6cd:	shl    $0x8,%esi
   3e6d0:	or     %r13d,%esi
   3e6d3:	mov    %esi,0x60(%rcx,%rax,1)
   3e6d7:	movzwl 0x32(%rdi),%esi
   3e6db:	add    %r14d,%esi
   3e6de:	shl    $0x8,%esi
   3e6e1:	or     %r13d,%esi
   3e6e4:	mov    %esi,0x64(%rcx,%rax,1)
   3e6e8:	movzwl 0x34(%rdi),%esi
   3e6ec:	add    %r14d,%esi
   3e6ef:	shl    $0x8,%esi
   3e6f2:	or     %r13d,%esi
   3e6f5:	mov    %esi,0x68(%rcx,%rax,1)
   3e6f9:	movzwl 0x36(%rdi),%esi
   3e6fd:	add    %r14d,%esi
   3e700:	shl    $0x8,%esi
   3e703:	or     %r13d,%esi
   3e706:	mov    %esi,0x6c(%rcx,%rax,1)
   3e70a:	movzwl 0x38(%rdi),%esi
   3e70e:	add    %r14d,%esi
   3e711:	shl    $0x8,%esi
   3e714:	or     %r13d,%esi
   3e717:	mov    %esi,0x70(%rcx,%rax,1)
   3e71b:	movzwl 0x3a(%rdi),%esi
   3e71f:	add    %r14d,%esi
   3e722:	shl    $0x8,%esi
   3e725:	or     %r13d,%esi
   3e728:	mov    %esi,0x74(%rcx,%rax,1)
   3e72c:	lea    (%r10,%rbp,1),%eax
   3e730:	add    $0x20,%eax
   3e733:	mov    %eax,0x28e0(%rsp)
   3e73a:	add    $0x20,%rbp
   3e73e:	inc    %r9
   3e741:	add    $0x2000,%r15d
   3e748:	dec    %rdx
   3e74b:	je     3e94c <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
   3e751:	mov    %r9,%rax
   3e754:	shr    $0x6,%rax
   3e758:	cmp    (%rsp),%rax
   3e75c:	jae    3efbb <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   3e762:	mov    0x10(%rsp),%rcx
   3e767:	mov    (%rcx,%rax,8),%rax
   3e76b:	bt     %r9,%rax
   3e76f:	jae    3e7a0 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   3e771:	cmp    $0x1e,%rbx
   3e775:	jae    3efe0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   3e77b:	mov    (%r12,%rbp,2),%eax
   3e77f:	and    $0xfffff,%eax
   3e784:	mov    0x10(%r11),%rcx
   3e788:	cmp    %rax,%rcx
   3e78b:	ja     3e530 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   3e791:	jmp    3efce <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
   3e796:	cs nopw 0x0(%rax,%rax,1)
   3e7a0:	cmp    $0x1e,%rbx
   3e7a4:	jae    3eff4 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
   3e7aa:	movups 0x2c(%rdi),%xmm0
   3e7ae:	movups %xmm0,-0x10(%r12,%rbp,2)
   3e7b4:	movdqa (%rdi),%xmm0
   3e7b8:	movdqa 0x10(%rdi),%xmm1
   3e7bd:	movdqa 0x20(%rdi),%xmm2
   3e7c2:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   3e7c9:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   3e7d0:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   3e7d7:	mov    %r15d,(%r12,%rbp,2)
   3e7db:	jmp    3e72c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
   3e7e0:	mov    0x28d8(%rsp),%rdi
   3e7e8:	cmp    $0x1d,%rdi
   3e7ec:	ja     3f089 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   3e7f2:	shl    $0x6,%rdi
   3e7f6:	add    %r15,%rdi
   3e7f9:	mov    0x28e0(%rsp),%eax
   3e800:	mov    0xc0(%rsp),%rcx
   3e808:	shl    $0x6,%r9
   3e80c:	mov    %eax,%r14d
   3e80f:	shl    $0x8,%r14d
   3e813:	or     %r13d,%r14d
   3e816:	cmp    $0x3c,%r12
   3e81a:	jae    3e885 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3e81c:	xor    %esi,%esi
   3e81e:	jmp    3e90a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
   3e823:	shr    $1,%r9
   3e826:	inc    %r9
   3e829:	mov    %r9,%r10
   3e82c:	and    $0xfffffffffffffff8,%r10
   3e830:	lea    (%rdx,%r10,2),%r8
   3e834:	movd   %ecx,%xmm0
   3e838:	pshuflw $0x0,%xmm0,%xmm0
   3e83d:	add    %r15,%r11
   3e840:	add    $0x8,%r11
   3e844:	xor    %ebx,%ebx
   3e846:	cs nopw 0x0(%rax,%rax,1)
   3e850:	movq   -0x8(%r11,%rbx,2),%xmm1
   3e857:	movq   (%r11,%rbx,2),%xmm2
   3e85d:	paddw  %xmm0,%xmm1
   3e861:	paddw  %xmm0,%xmm2
   3e865:	punpcklqdq %xmm2,%xmm1
   3e869:	movdqu %xmm1,(%rax,%rbx,2)
   3e86e:	add    $0x8,%rbx
   3e872:	cmp    %rbx,%r10
   3e875:	jne    3e850 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   3e877:	cmp    %r10,%r9
   3e87a:	jne    3e3c2 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   3e880:	jmp    3e3e9 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
   3e885:	mov    %rdx,%r10
   3e888:	movabs $0xffffffffffffffe,%rsi
   3e892:	and    %rsi,%r10
   3e895:	lea    (%rcx,%r9,1),%r11
   3e899:	add    $0x7c,%r11
   3e89d:	xor    %esi,%esi
   3e89f:	nop
   3e8a0:	movdqu 0x2c(%rdi),%xmm0
   3e8a5:	movdqu %xmm0,-0x50(%r11)
   3e8ab:	movdqa (%rdi),%xmm1
   3e8af:	movdqa 0x10(%rdi),%xmm2
   3e8b4:	movdqa 0x20(%rdi),%xmm3
   3e8b9:	movdqa %xmm3,-0x5c(%r11)
   3e8bf:	movdqa %xmm2,-0x6c(%r11)
   3e8c5:	movdqa %xmm1,-0x7c(%r11)
   3e8cb:	mov    %r14d,-0x40(%r11)
   3e8cf:	add    $0x2,%rsi
   3e8d3:	movdqa %xmm1,-0x3c(%r11)
   3e8d9:	movdqa %xmm2,-0x2c(%r11)
   3e8df:	movdqa %xmm3,-0x1c(%r11)
   3e8e5:	movdqu %xmm0,-0x10(%r11)
   3e8eb:	lea    0x2000(%r14),%ebx
   3e8f2:	mov    %ebx,(%r11)
   3e8f5:	add    $0x4000,%r14d
   3e8fc:	sub    $0xffffffffffffff80,%r11
   3e900:	cmp    %rsi,%r10
   3e903:	jne    3e8a0 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   3e905:	test   $0x1,%dl
   3e908:	je     3e93e <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
   3e90a:	add    %r9,%rcx
   3e90d:	shl    $0x6,%rsi
   3e911:	movups 0x2c(%rdi),%xmm0
   3e915:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3e91a:	movdqa (%rdi),%xmm0
   3e91e:	movdqa 0x10(%rdi),%xmm1
   3e923:	movdqa 0x20(%rdi),%xmm2
   3e928:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3e92e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3e934:	movdqa %xmm0,(%rcx,%rsi,1)
   3e939:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3e93e:	mov    %edx,%ecx
   3e940:	shl    $0x5,%ecx
   3e943:	add    %ecx,%eax
   3e945:	mov    %eax,0x28e0(%rsp)
   3e94c:	mov    0x48(%rsp),%rdx
   3e951:	mov    %rdx,%rax
   3e954:	shl    $0x5,%rax
   3e958:	lea    (%rdx,%rdx,1),%rcx
   3e95c:	sub    %rax,%rcx
   3e95f:	mov    0x50(%rsp),%r12
   3e964:	add    %rcx,%r12
   3e967:	lea    (%rdx,%rdx,1),%eax
   3e96a:	shl    $0x5,%edx
   3e96d:	sub    %eax,%edx
   3e96f:	add    %edx,%r8d
   3e972:	mov    0x88(%rsp),%rbp
   3e97a:	mov    %r8d,0x18(%rbp)
   3e97e:	test   %r12,%r12
   3e981:	mov    $0x88888889,%r14d
   3e987:	lea    -0x2374e(%rip),%r15        # 1b240 <c30walk::ctor2::PATTERNS>
   3e98e:	je     3eb09 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
   3e994:	mov    0x1c(%rbp),%ecx
   3e997:	mov    %r8d,%edi
   3e99a:	imul   %r14,%rdi
   3e99e:	mov    %rdi,%rdx
   3e9a1:	shr    $0x24,%rdx
   3e9a5:	mov    %edx,%eax
   3e9a7:	shl    $0x5,%eax
   3e9aa:	sub    %edx,%eax
   3e9ac:	sub    %edx,%eax
   3e9ae:	lea    0x1e(%rax),%esi
   3e9b1:	cmp    %ecx,%esi
   3e9b3:	cmovb  %esi,%ecx
   3e9b6:	shr    $0x2a,%rdi
   3e9ba:	mov    0xa8(%rsp),%rsi
   3e9c2:	cmp    %rsi,%rdi
   3e9c5:	jae    3f06c <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
   3e9cb:	mov    %rdx,%rsi
   3e9ce:	shl    $0x6,%rsi
   3e9d2:	add    0xc0(%rsp),%rsi
   3e9da:	mov    0xa0(%rsp),%r9
   3e9e2:	mov    (%r9,%rdi,8),%rdi
   3e9e6:	bt     %rdx,%rdi
   3e9ea:	jb     3e320 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   3e9f0:	mov    $0xffffffff,%edi
   3e9f5:	jmp    3e32c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
   3e9fa:	nopw   0x0(%rax,%rax,1)
   3ea00:	mov    0x8(%rsp),%r13
   3ea05:	jmp    3eb16 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   3ea0a:	nopw   0x0(%rax,%rax,1)
   3ea10:	add    0x28(%rsp),%r11
   3ea15:	xor    %r13d,%r13d
   3ea18:	nopl   0x0(%rax,%rax,1)
   3ea20:	movzbl (%r11,%r13,1),%esi
   3ea25:	lea    (%rsi,%rsi,4),%rcx
   3ea29:	lea    (%r8,%rcx,8),%rbp
   3ea2d:	mov    (%r8,%rcx,8),%rax
   3ea31:	cmp    0x8(%r8,%rcx,8),%rax
   3ea36:	je     3ea4e <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   3ea38:	lea    (%r14,%r13,1),%rcx
   3ea3c:	mov    %cx,(%rax)
   3ea3f:	addq   $0x2,0x0(%rbp)
   3ea44:	inc    %r13
   3ea47:	cmp    %r13,%rbx
   3ea4a:	jne    3ea20 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3ea4c:	jmp    3ea80 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   3ea4e:	mov    %r11,(%rsp)
   3ea52:	lea    0x98(%rsp),%rdi
   3ea5a:	call   93020 <<c30walk::ctor2::W<false>>::refill>
   3ea5f:	lea    0xd8(%rsp),%r8
   3ea67:	test   %al,%al
   3ea69:	je     3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3ea6f:	mov    0x0(%rbp),%rax
   3ea73:	mov    0x20(%rsp),%r10
   3ea78:	mov    (%rsp),%r11
   3ea7c:	jmp    3ea38 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
   3ea7e:	xchg   %ax,%ax
   3ea80:	cmp    %r15,0x30(%rsp)
   3ea85:	mov    0x8(%rsp),%r13
   3ea8a:	jb     3efa4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3ea90:	cmp    $0xf,%r12
   3ea94:	ja     3e160 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3ea9a:	add    0x28(%rsp),%r15
   3ea9f:	mov    %r15,%r12
   3eaa2:	sub    %rbx,%r12
   3eaa5:	add    $0x10,%r12
   3eaa9:	add    %r14d,%ebx
   3eaac:	add    $0x2,%ebx
   3eaaf:	nop
   3eab0:	movzbl (%r15),%esi
   3eab4:	lea    (%rsi,%rsi,4),%rcx
   3eab8:	lea    (%r8,%rcx,8),%r14
   3eabc:	mov    (%r8,%rcx,8),%rax
   3eac0:	cmp    0x8(%r8,%rcx,8),%rax
   3eac5:	je     3eadd <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   3eac7:	mov    %bx,(%rax)
   3eaca:	addq   $0x2,(%r14)
   3eace:	inc    %r15
   3ead1:	inc    %ebx
   3ead3:	cmp    %r12,%r15
   3ead6:	jne    3eab0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   3ead8:	jmp    3e160 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3eadd:	lea    0x98(%rsp),%rdi
   3eae5:	call   93020 <<c30walk::ctor2::W<false>>::refill>
   3eaea:	lea    0xd8(%rsp),%r8
   3eaf2:	test   %al,%al
   3eaf4:	je     3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3eafa:	mov    (%r14),%rax
   3eafd:	mov    0x8(%rsp),%r13
   3eb02:	mov    0x20(%rsp),%r10
   3eb07:	jmp    3eac7 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
   3eb09:	mov    0x8(%rsp),%r13
   3eb0e:	lea    0xd8(%rsp),%r8
   3eb16:	mov    0x20(%rsp),%r10
   3eb1b:	mov    0x68(%rsp),%r9
   3eb20:	mov    0x38(%rsp),%rcx
   3eb25:	mov    0x30(%rsp),%r11
   3eb2a:	mov    0x44(%rsp),%edx
   3eb2e:	mov    0x70(%rsp),%rsi
   3eb33:	jmp    3e1a2 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
   3eb38:	nopl   0x0(%rax,%rax,1)
   3eb40:	lea    (%r14,%r11,1),%rax
   3eb44:	cmp    %r10,%rax
   3eb47:	jne    3eb91 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
   3eb49:	cmp    $0x10,%rdx
   3eb4d:	jne    3ec07 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
   3eb53:	mov    0x28d8(%rsp),%rax
   3eb5b:	lea    0x10(%rax),%rcx
   3eb5f:	cmp    $0x1d,%rcx
   3eb63:	mov    0x8(%rsp),%r13
   3eb68:	mov    0x28(%rsp),%r14
   3eb6d:	mov    %rbp,%r9
   3eb70:	jbe    3ecab <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
   3eb76:	add    $0xfffffffffffffff2,%rax
   3eb7a:	mov    %rax,0x28d8(%rsp)
   3eb82:	mov    0x28e0(%rsp),%eax
   3eb89:	add    $0x12,%eax
   3eb8c:	jmp    3ecbd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3eb91:	add    $0x2,%r12d
   3eb95:	movzwl %r12w,%r12d
   3eb99:	nopl   0x0(%rax)
   3eba0:	movzbl (%r15,%r14,1),%esi
   3eba5:	lea    (%rsi,%rsi,4),%rcx
   3eba9:	lea    (%r8,%rcx,8),%rbx
   3ebad:	mov    (%r8,%rcx,8),%rax
   3ebb1:	cmp    0x8(%r8,%rcx,8),%rax
   3ebb6:	je     3ebd0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   3ebb8:	lea    (%r12,%r14,1),%rcx
   3ebbc:	mov    %cx,(%rax)
   3ebbf:	addq   $0x2,(%rbx)
   3ebc3:	inc    %r14
   3ebc6:	cmp    %r14,%rdx
   3ebc9:	jne    3eba0 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   3ebcb:	jmp    3eb49 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
   3ebd0:	mov    %rdx,0x10(%rsp)
   3ebd5:	mov    %r11,%r13
   3ebd8:	lea    0x98(%rsp),%rdi
   3ebe0:	call   93020 <<c30walk::ctor2::W<false>>::refill>
   3ebe5:	lea    0xd8(%rsp),%r8
   3ebed:	test   %al,%al
   3ebef:	je     3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3ebf5:	mov    (%rbx),%rax
   3ebf8:	mov    0x20(%rsp),%r10
   3ebfd:	mov    %r13,%r11
   3ec00:	mov    0x10(%rsp),%rdx
   3ec05:	jmp    3ebb8 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
   3ec07:	mov    0x28e0(%rsp),%eax
   3ec0e:	mov    0x28d8(%rsp),%rcx
   3ec16:	inc    %r11
   3ec19:	cmp    %r11,%r10
   3ec1c:	mov    0x8(%rsp),%r13
   3ec21:	mov    0x28(%rsp),%r14
   3ec26:	mov    %rbp,%r9
   3ec29:	je     3ec86 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   3ec2b:	mov    %rdx,%r10
   3ec2e:	and    $0xffffe,%edx
   3ec34:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ec40:	lea    0x1(%rcx),%rsi
   3ec44:	add    $0xffffffffffffffe3,%rcx
   3ec48:	xor    %edi,%edi
   3ec4a:	cmp    $0x1e,%rsi
   3ec4e:	setae  %dil
   3ec52:	cmovb  %rsi,%rcx
   3ec56:	lea    (%rax,%rdi,2),%eax
   3ec59:	lea    0x1(%rcx),%rsi
   3ec5d:	add    $0xffffffffffffffe3,%rcx
   3ec61:	xor    %edi,%edi
   3ec63:	cmp    $0x1e,%rsi
   3ec67:	setae  %dil
   3ec6b:	cmovb  %rsi,%rcx
   3ec6f:	lea    (%rax,%rdi,2),%eax
   3ec72:	add    $0x2,%eax
   3ec75:	add    $0xfffffffffffffffe,%rdx
   3ec79:	jne    3ec40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   3ec7b:	test   $0x1,%r10b
   3ec7f:	mov    0x20(%rsp),%r10
   3ec84:	je     3eca1 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
   3ec86:	lea    0x1(%rcx),%rdx
   3ec8a:	add    $0xffffffffffffffe3,%rcx
   3ec8e:	xor    %esi,%esi
   3ec90:	cmp    $0x1e,%rdx
   3ec94:	setae  %sil
   3ec98:	cmovb  %rdx,%rcx
   3ec9c:	lea    (%rax,%rsi,2),%eax
   3ec9f:	inc    %eax
   3eca1:	mov    %rcx,0x28d8(%rsp)
   3eca9:	jmp    3ecbd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3ecab:	mov    %rcx,0x28d8(%rsp)
   3ecb3:	mov    0x28e0(%rsp),%eax
   3ecba:	add    $0x10,%eax
   3ecbd:	mov    %eax,0x28e0(%rsp)
   3ecc4:	mov    %r10,%r11
   3ecc7:	cmp    %r13,%r10
   3ecca:	jb     3dfac <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3ecd0:	jmp    3ecdf <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3ecd2:	mov    %r15,%rdi
   3ecd5:	call   fea70 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3ecda:	jmp    3e02f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
   3ecdf:	mov    $0x84,%eax
   3ece4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ecf0:	lea    0x50(%rax),%rcx
   3ecf4:	cmp    $0x28d4,%rcx
   3ecfb:	je     3ed5b <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
   3ecfd:	mov    0x54(%rsp,%rax,1),%rdx
   3ed02:	sub    0x64(%rsp,%rax,1),%rdx
   3ed07:	shr    $1,%rdx
   3ed0a:	add    0x6c(%rsp,%rax,1),%edx
   3ed0e:	cmp    0x70(%rsp,%rax,1),%edx
   3ed12:	jne    3ed37 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3ed14:	mov    0x7c(%rsp,%rax,1),%rdx
   3ed19:	sub    0x8c(%rsp,%rax,1),%rdx
   3ed21:	shr    $1,%rdx
   3ed24:	add    0x94(%rsp,%rax,1),%edx
   3ed2b:	cmp    0x98(%rsp,%rax,1),%edx
   3ed32:	mov    %rcx,%rax
   3ed35:	je     3ecf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
   3ed37:	mov    0x5520(%rsp),%rax
   3ed3f:	movq   $0x0,0x10(%rax)
   3ed47:	xor    %eax,%eax
   3ed49:	add    $0x54e8,%rsp
   3ed50:	pop    %rbx
   3ed51:	pop    %r12
   3ed53:	pop    %r13
   3ed55:	pop    %r14
   3ed57:	pop    %r15
   3ed59:	pop    %rbp
   3ed5a:	ret
   3ed5b:	xor    %r14d,%r14d
   3ed5e:	lea    0x98(%rsp),%r15
   3ed66:	mov    0x78(%rsp),%r12
   3ed6b:	mov    0x8(%rsp),%r13
   3ed70:	mov    0x60(%rsp),%rbp
   3ed75:	jmp    3edab <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   3ed77:	nopw   0x0(%rax,%rax,1)
   3ed80:	sub    %rcx,%rax
   3ed83:	inc    %r14
   3ed86:	shr    $1,%rax
   3ed89:	add    %eax,0x18(%r8)
   3ed8d:	pxor   %xmm0,%xmm0
   3ed91:	movdqu %xmm0,(%r8)
   3ed96:	movq   $0x0,0x10(%r8)
   3ed9e:	add    $0x28,%r8
   3eda2:	cmp    $0x100,%r14
   3eda9:	je     3edd1 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
   3edab:	mov    (%r8),%rcx
   3edae:	mov    %rcx,%rax
   3edb1:	cmp    0x10(%r8),%rcx
   3edb5:	je     3ed80 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3edb7:	mov    %r15,%rdi
   3edba:	mov    %r14,%rsi
   3edbd:	mov    %r8,%rbx
   3edc0:	call   92a20 <<c30walk::ctor2::W<false>>::finish>
   3edc5:	mov    %rbx,%r8
   3edc8:	mov    (%rbx),%rax
   3edcb:	mov    0x10(%rbx),%rcx
   3edcf:	jmp    3ed80 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3edd1:	xor    %ecx,%ecx
   3edd3:	mov    0x8(%r12),%rbx
   3edd8:	test   %rbp,%rbp
   3eddb:	setne  %al
   3edde:	je     3ee22 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3ede0:	mov    %al,%cl
   3ede2:	mov    %rbp,%rax
   3ede5:	sub    %rcx,%rax
   3ede8:	imul   $0xffffffffffffffe2,%rax,%rcx
   3edec:	add    %r13,%rcx
   3edef:	cmp    $0x1e,%rcx
   3edf3:	jae    3ee22 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3edf5:	lea    (%rbx,%rbp,4),%rcx
   3edf9:	lea    (%rcx,%r13,2),%rdi
   3edfd:	add    $0xfffffffffffffffc,%rdi
   3ee01:	lea    (%rax,%rax,1),%rcx
   3ee05:	shl    $0x5,%rax
   3ee09:	sub    %rcx,%rax
   3ee0c:	sub    %r13,%rax
   3ee0f:	lea    0x3c(,%rax,2),%rdx
   3ee17:	mov    $0xff,%esi
   3ee1c:	call   *0x10e7ce(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   3ee22:	shl    $0x6,%rbp
   3ee26:	pcmpeqd %xmm0,%xmm0
   3ee2a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   3ee30:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   3ee36:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   3ee3c:	movdqa %xmm0,(%rbx,%rbp,1)
   3ee41:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   3ee49:	mov    0x58(%rsp),%rax
   3ee4e:	mov    0x8(%rax),%rdx
   3ee52:	mov    0x10(%rax),%rcx
   3ee56:	mov    0x5520(%rsp),%rax
   3ee5e:	mov    0x8(%rax),%r8
   3ee62:	mov    0x10(%rax),%r9
   3ee66:	lea    0x98(%rsp),%rdi
   3ee6e:	mov    %rbx,%rsi
   3ee71:	call   a5850 <c30walk::ctor::canonicalise>
   3ee76:	mov    0x80(%rsp),%rax
   3ee7e:	mov    %rax,0x10(%r12)
   3ee83:	mov    $0x1,%al
   3ee85:	jmp    3ed49 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
   3ee8a:	lea    0x10acc7(%rip),%rcx        # 149b58 <__frame_dummy_init_array_entry+0x32c8>
   3ee91:	mov    %r12,%rdi
   3ee94:	mov    %r13,%rdx
   3ee97:	call   f0c50 <core::slice::index::slice_index_fail>
   3ee9c:	lea    0x1095ed(%rip),%rcx        # 148490 <__frame_dummy_init_array_entry+0x1c00>
   3eea3:	mov    $0x1e,%edx
   3eea8:	xor    %edi,%edi
   3eeaa:	call   f0c50 <core::slice::index::slice_index_fail>
   3eeaf:	mov    %rdi,%r12
   3eeb2:	test   %rax,%rax
   3eeb5:	je     3eec7 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
   3eeb7:	mov    0x8(%r9),%rdi
   3eebb:	mov    %r9,%r14
   3eebe:	call   *0x10e73c(%rip)        # 14d600 <free@GLIBC_2.2.5>
   3eec4:	mov    %r14,%r9
   3eec7:	movq   $0x0,(%r9)
   3eece:	movq   $0x40,0x8(%r9)
   3eed6:	movq   $0x0,0x10(%r9)
   3eede:	mov    $0x40,%edx
   3eee3:	mov    %r9,%rdi
   3eee6:	lea    0x1(%rbp),%rsi
   3eeea:	mov    %r9,%r14
   3eeed:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3eef2:	mov    %r14,%r9
   3eef5:	cmp    $0xffffffffffffffff,%rax
   3eef9:	mov    %r12,%rdi
   3eefc:	je     3d9f5 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   3ef02:	jmp    3ef5d <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
   3ef04:	mov    %rdi,%r12
   3ef07:	mov    %r9,%r14
   3ef0a:	test   %rax,%rax
   3ef0d:	je     3ef21 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
   3ef0f:	mov    0x5520(%rsp),%rax
   3ef17:	mov    0x8(%rax),%rdi
   3ef1b:	call   *0x10e6df(%rip)        # 14d600 <free@GLIBC_2.2.5>
   3ef21:	mov    0x5520(%rsp),%rdi
   3ef29:	movq   $0x0,(%rdi)
   3ef30:	movq   $0x40,0x8(%rdi)
   3ef38:	movq   $0x0,0x10(%rdi)
   3ef40:	mov    $0x80,%edx
   3ef45:	mov    %rbx,%rsi
   3ef48:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3ef4d:	cmp    $0xffffffffffffffff,%rax
   3ef51:	mov    %r14,%r9
   3ef54:	mov    %r12,%rdi
   3ef57:	je     3da09 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
   3ef5d:	mov    %rax,%rdi
   3ef60:	mov    %rdx,%rsi
   3ef63:	call   ee3db <alloc::raw_vec::handle_error>
   3ef68:	lea    0x10abe9(%rip),%rcx        # 149b58 <__frame_dummy_init_array_entry+0x32c8>
   3ef6f:	mov    %r11,%rdi
   3ef72:	mov    %r13,%rdx
   3ef75:	call   f0c50 <core::slice::index::slice_index_fail>
   3ef7a:	lea    0x10953f(%rip),%rcx        # 1484c0 <__frame_dummy_init_array_entry+0x1c30>
   3ef81:	mov    %r11,%rdi
   3ef84:	mov    %r15,%rsi
   3ef87:	mov    %r13,%rdx
   3ef8a:	call   f0c50 <core::slice::index::slice_index_fail>
   3ef8f:	lea    0x10abc2(%rip),%rcx        # 149b58 <__frame_dummy_init_array_entry+0x32c8>
   3ef96:	mov    %rsi,%rdi
   3ef99:	mov    %rax,%rsi
   3ef9c:	mov    %r13,%rdx
   3ef9f:	call   f0c50 <core::slice::index::slice_index_fail>
   3efa4:	lea    0x1094fd(%rip),%rcx        # 1484a8 <__frame_dummy_init_array_entry+0x1c18>
   3efab:	mov    %r15,%rdi
   3efae:	mov    0x30(%rsp),%rsi
   3efb3:	mov    %r13,%rdx
   3efb6:	call   f0c50 <core::slice::index::slice_index_fail>
   3efbb:	lea    0x107b0e(%rip),%rdx        # 146ad0 <__frame_dummy_init_array_entry+0x240>
   3efc2:	mov    %rax,%rdi
   3efc5:	mov    (%rsp),%rsi
   3efc9:	call   efe73 <core::panicking::panic_bounds_check>
   3efce:	lea    0x10957b(%rip),%rdx        # 148550 <__frame_dummy_init_array_entry+0x1cc0>
   3efd5:	mov    %rax,%rdi
   3efd8:	mov    %rcx,%rsi
   3efdb:	call   efe73 <core::panicking::panic_bounds_check>
   3efe0:	lea    0x109551(%rip),%rdx        # 148538 <__frame_dummy_init_array_entry+0x1ca8>
   3efe7:	mov    $0x1e,%esi
   3efec:	mov    %rbx,%rdi
   3efef:	call   efe73 <core::panicking::panic_bounds_check>
   3eff4:	lea    0x109525(%rip),%rdx        # 148520 <__frame_dummy_init_array_entry+0x1c90>
   3effb:	mov    $0x1e,%esi
   3f000:	mov    %rbx,%rdi
   3f003:	call   efe73 <core::panicking::panic_bounds_check>
   3f008:	mov    $0x10,%edx
   3f00d:	mov    $0x4,%ecx
   3f012:	mov    $0x400,%r8d
   3f018:	mov    %rdi,%r14
   3f01b:	mov    %r12,%rdi
   3f01e:	xor    %esi,%esi
   3f020:	mov    %r9,%rbx
   3f023:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3f028:	mov    %r14,%rdi
   3f02b:	mov    %rbx,%r9
   3f02e:	jmp    3daa0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
   3f033:	mov    %r9,%r13
   3f036:	mov    $0x8,%ecx
   3f03b:	mov    $0x8,%r8d
   3f041:	mov    %rdi,%r14
   3f044:	xor    %esi,%esi
   3f046:	mov    %rbx,%rdx
   3f049:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3f04e:	mov    %r14,%rdi
   3f051:	mov    0x10(%r14),%r14
   3f055:	mov    0x8(%rdi),%r12
   3f059:	lea    (%r12,%r14,8),%rdi
   3f05d:	cmp    $0x1,%rbx
   3f061:	jne    3da47 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   3f067:	jmp    3da69 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3f06c:	lea    0x107a5d(%rip),%rdx        # 146ad0 <__frame_dummy_init_array_entry+0x240>
   3f073:	call   efe73 <core::panicking::panic_bounds_check>
   3f078:	lea    0x1093f9(%rip),%rdx        # 148478 <__frame_dummy_init_array_entry+0x1be8>
   3f07f:	mov    $0x1e,%esi
   3f084:	call   efe73 <core::panicking::panic_bounds_check>
   3f089:	lea    0x109478(%rip),%rdx        # 148508 <__frame_dummy_init_array_entry+0x1c78>
   3f090:	mov    $0x1e,%esi
   3f095:	call   efe73 <core::panicking::panic_bounds_check>
   3f09a:	lea    0x10935f(%rip),%rdx        # 148400 <__frame_dummy_init_array_entry+0x1b70>
   3f0a1:	call   efe73 <core::panicking::panic_bounds_check>
   3f0a6:	lea    0x107a23(%rip),%rdx        # 146ad0 <__frame_dummy_init_array_entry+0x240>
   3f0ad:	mov    %rbp,%rdi
   3f0b0:	call   efe73 <core::panicking::panic_bounds_check>
