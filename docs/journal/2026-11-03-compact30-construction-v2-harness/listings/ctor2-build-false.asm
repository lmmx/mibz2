# <c30walk::ctor2::Ctor2>::build::<false>
# size 0x18e5

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000347f0 <<c30walk::ctor2::Ctor2>::build::<false>>:
   347f0:	push   %rbp
   347f1:	push   %r15
   347f3:	push   %r14
   347f5:	push   %r13
   347f7:	push   %r12
   347f9:	push   %rbx
   347fa:	sub    $0x1000,%rsp
   34801:	movq   $0x0,(%rsp)
   34809:	sub    $0x1000,%rsp
   34810:	movq   $0x0,(%rsp)
   34818:	sub    $0x1000,%rsp
   3481f:	movq   $0x0,(%rsp)
   34827:	sub    $0x1000,%rsp
   3482e:	movq   $0x0,(%rsp)
   34836:	sub    $0x1000,%rsp
   3483d:	movq   $0x0,(%rsp)
   34845:	sub    $0x4e8,%rsp
   3484c:	mov    %r8d,0x1c(%rsp)
   34851:	mov    0x5520(%rsp),%rax
   34859:	movq   $0x0,0x10(%r9)
   34861:	movq   $0x0,0x10(%rax)
   34869:	cmp    $0xeffe2,%rdx
   34870:	ja     35d67 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   34876:	mov    %rcx,%r15
   34879:	mov    %rdx,%r13
   3487c:	pxor   %xmm0,%xmm0
   34880:	xor    %eax,%eax
   34882:	pxor   %xmm2,%xmm2
   34886:	pxor   %xmm1,%xmm1
   3488a:	nopw   0x0(%rax,%rax,1)
   34890:	movq   (%r15,%rax,4),%xmm3
   34896:	movq   0x8(%r15,%rax,4),%xmm4
   3489d:	punpckldq %xmm0,%xmm3
   348a1:	paddq  %xmm2,%xmm3
   348a5:	punpckldq %xmm0,%xmm4
   348a9:	paddq  %xmm1,%xmm4
   348ad:	movq   0x10(%r15,%rax,4),%xmm2
   348b4:	movq   0x18(%r15,%rax,4),%xmm1
   348bb:	punpckldq %xmm0,%xmm2
   348bf:	paddq  %xmm3,%xmm2
   348c3:	punpckldq %xmm0,%xmm1
   348c7:	paddq  %xmm4,%xmm1
   348cb:	add    $0x8,%rax
   348cf:	cmp    $0x100,%rax
   348d5:	jne    34890 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   348d7:	paddq  %xmm2,%xmm1
   348db:	pshufd $0xee,%xmm1,%xmm0
   348e0:	paddq  %xmm1,%xmm0
   348e4:	movq   %xmm0,%rax
   348e9:	cmp    %r13,%rax
   348ec:	jne    35d67 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   348f2:	pxor   %xmm2,%xmm2
   348f6:	xor    %eax,%eax
   348f8:	movdqa -0x27710(%rip),%xmm3        # d1f0 <GCC_except_table910+0x930>
   34900:	pxor   %xmm1,%xmm1
   34904:	pxor   %xmm0,%xmm0
   34908:	nopl   0x0(%rax,%rax,1)
   34910:	movq   (%r15,%rax,4),%xmm4
   34916:	movq   0x8(%r15,%rax,4),%xmm5
   3491d:	pshufd $0xd4,%xmm4,%xmm4
   34922:	pcmpeqd %xmm2,%xmm4
   34926:	pandn  %xmm3,%xmm4
   3492a:	paddq  %xmm1,%xmm4
   3492e:	pshufd $0xd4,%xmm5,%xmm5
   34933:	pcmpeqd %xmm2,%xmm5
   34937:	pandn  %xmm3,%xmm5
   3493b:	paddq  %xmm0,%xmm5
   3493f:	movq   0x10(%r15,%rax,4),%xmm0
   34946:	movq   0x18(%r15,%rax,4),%xmm6
   3494d:	pshufd $0xd4,%xmm0,%xmm1
   34952:	pcmpeqd %xmm2,%xmm1
   34956:	pandn  %xmm3,%xmm1
   3495a:	paddq  %xmm4,%xmm1
   3495e:	pshufd $0xd4,%xmm6,%xmm0
   34963:	pcmpeqd %xmm2,%xmm0
   34967:	pandn  %xmm3,%xmm0
   3496b:	paddq  %xmm5,%xmm0
   3496f:	add    $0x8,%rax
   34973:	cmp    $0x100,%rax
   34979:	jne    34910 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
   3497b:	mov    $0x88888889,%ebp
   34980:	imul   %r13,%rbp
   34984:	shr    $0x24,%rbp
   34988:	imul   $0xeeeeeeef,%r13d,%eax
   3498f:	ror    $1,%eax
   34991:	cmp    $0x8888889,%eax
   34996:	sbb    $0xffffffffffffffff,%rbp
   3499a:	mov    %rbp,%rbx
   3499d:	cmpl   $0x0,0x1c(%rsp)
   349a2:	jne    349f8 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   349a4:	test   %r13,%r13
   349a7:	je     349f6 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   349a9:	paddq  %xmm1,%xmm0
   349ad:	pshufd $0xee,%xmm0,%xmm1
   349b2:	paddq  %xmm0,%xmm1
   349b6:	movq   %xmm1,%rax
   349bb:	lea    -0x1(%r13),%ecx
   349bf:	mov    $0x88888889,%edx
   349c4:	imul   %rcx,%rdx
   349c8:	shr    $0x23,%rdx
   349cc:	and    $0xfffffffe,%edx
   349cf:	lea    (%rdx,%r13,1),%ecx
   349d3:	dec    %ecx
   349d5:	shr    $1,%ecx
   349d7:	mov    $0x80010003,%ebx
   349dc:	imul   %rcx,%rbx
   349e0:	shr    $0x2e,%rbx
   349e4:	inc    %ebx
   349e6:	imul   %rax,%rbx
   349ea:	dec    %rbx
   349ed:	cmp    %rbp,%rbx
   349f0:	cmovae %rbp,%rbx
   349f4:	jmp    349f8 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   349f6:	xor    %ebx,%ebx
   349f8:	mov    (%r9),%rax
   349fb:	lea    0x1(%rbp),%rcx
   349ff:	mov    %rcx,0x80(%rsp)
   34a07:	cmp    %rbp,%rax
   34a0a:	mov    %rsi,0x28(%rsp)
   34a0f:	jbe    35ecf <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
   34a15:	mov    0x5520(%rsp),%rax
   34a1d:	mov    (%rax),%rax
   34a20:	cmp    %rbx,%rax
   34a23:	jb     35f24 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
   34a29:	movq   $0x0,0x10(%rdi)
   34a31:	lea    0x3f(%rbp),%rbx
   34a35:	shr    $0x6,%rbx
   34a39:	mov    %r13,0x8(%rsp)
   34a3e:	mov    %rdi,0x58(%rsp)
   34a43:	mov    %rbp,0x60(%rsp)
   34a48:	je     34aa2 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
   34a4a:	cmp    (%rdi),%rbx
   34a4d:	ja     36053 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   34a53:	mov    %r9,%r13
   34a56:	xor    %r14d,%r14d
   34a59:	mov    0x8(%rdi),%r12
   34a5d:	lea    (%r12,%r14,8),%rdi
   34a61:	cmp    $0x1,%rbx
   34a65:	je     34a89 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   34a67:	lea    -0x8(,%rbx,8),%rdx
   34a6f:	xor    %esi,%esi
   34a71:	call   *0xd11c9(%rip)        # 105c40 <memset@GLIBC_2.2.5>
   34a77:	lea    (%r14,%rbx,1),%rax
   34a7b:	lea    (%r12,%rax,8),%rdi
   34a7f:	add    $0xfffffffffffffff8,%rdi
   34a83:	add    %rbx,%r14
   34a86:	dec    %r14
   34a89:	mov    %r13,%r9
   34a8c:	movq   $0x0,(%rdi)
   34a93:	inc    %r14
   34a96:	mov    0x8(%rsp),%r13
   34a9b:	mov    0x58(%rsp),%rdi
   34aa0:	jmp    34aa5 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
   34aa2:	xor    %r14d,%r14d
   34aa5:	lea    0x18(%rdi),%r12
   34aa9:	mov    %r14,0x10(%rdi)
   34aad:	movq   $0x0,0x28(%rdi)
   34ab5:	cmpq   $0xf,0x18(%rdi)
   34aba:	jbe    36028 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   34ac0:	lea    0x30(%rdi),%rax
   34ac4:	mov    %rax,(%rsp)
   34ac8:	xor    %eax,%eax
   34aca:	pxor   %xmm0,%xmm0
   34ace:	movabs $0xffffffff00000000,%rcx
   34ad8:	nopl   0x0(%rax,%rax,1)
   34ae0:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   34ae9:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   34af2:	mov    %rcx,0x2d08(%rsp,%rax,1)
   34afa:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   34b03:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   34b0c:	mov    %rcx,0x2d30(%rsp,%rax,1)
   34b14:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   34b1d:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   34b26:	mov    %rcx,0x2d58(%rsp,%rax,1)
   34b2e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   34b37:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   34b40:	mov    %rcx,0x2d80(%rsp,%rax,1)
   34b48:	add    $0xa0,%rax
   34b4e:	cmp    $0x2800,%rax
   34b54:	jne    34ae0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   34b56:	mov    %r12,%r14
   34b59:	lea    0xc(%r15),%rax
   34b5d:	xor    %edx,%edx
   34b5f:	mov    $0x25,%ecx
   34b64:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   34b70:	mov    -0xc(%rax),%esi
   34b73:	add    %edx,%esi
   34b75:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   34b7c:	mov    %esi,0x2c70(%rsp,%rcx,4)
   34b83:	mov    -0x8(%rax),%edx
   34b86:	add    %esi,%edx
   34b88:	mov    %esi,0x2c94(%rsp,%rcx,4)
   34b8f:	mov    %edx,0x2c98(%rsp,%rcx,4)
   34b96:	mov    -0x4(%rax),%esi
   34b99:	add    %edx,%esi
   34b9b:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   34ba2:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   34ba9:	mov    (%rax),%edx
   34bab:	add    %esi,%edx
   34bad:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   34bb4:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   34bbb:	add    $0x28,%rcx
   34bbf:	add    $0x10,%rax
   34bc3:	cmp    $0xa25,%rcx
   34bca:	jne    34b70 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
   34bcc:	mov    %r9,0x78(%rsp)
   34bd1:	mov    0x8(%r9),%rbx
   34bd5:	mov    0x8(%rdi),%rbp
   34bd9:	mov    0x10(%rdi),%r12
   34bdd:	lea    0xd8(%rsp),%rdi
   34be5:	lea    0x2ce8(%rsp),%rsi
   34bed:	mov    $0x2800,%edx
   34bf2:	call   *0xd1068(%rip)        # 105c60 <memcpy@GLIBC_2.14>
   34bf8:	mov    %rbp,%r8
   34bfb:	mov    %r12,%rsi
   34bfe:	mov    %rbx,0xc0(%rsp)
   34c06:	mov    0x60(%rsp),%rax
   34c0b:	mov    %rax,0xc8(%rsp)
   34c13:	mov    %r13,0xd0(%rsp)
   34c1b:	mov    0x5520(%rsp),%rax
   34c23:	mov    %rax,0x98(%rsp)
   34c2b:	mov    %rbp,0xa0(%rsp)
   34c33:	mov    %r12,0xa8(%rsp)
   34c3b:	mov    %r14,0xb0(%rsp)
   34c43:	movl   $0x0,0x28e0(%rsp)
   34c4e:	movq   $0x0,0x28d8(%rsp)
   34c5a:	mov    (%rsp),%rax
   34c5e:	mov    %rax,0xb8(%rsp)
   34c66:	mov    $0xffffffffffffffff,%rcx
   34c6d:	xor    %ebp,%ebp
   34c6f:	movabs $0xeeeeeeeeeeeeeeef,%r9
   34c79:	movabs $0x888888888888889,%r10
   34c83:	pxor   %xmm0,%xmm0
   34c87:	pcmpeqd %xmm1,%xmm1
   34c8b:	xor    %r12d,%r12d
   34c8e:	jmp    34d46 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   34c93:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   34ca0:	mov    0x8(%rdi),%rax
   34ca4:	mov    %r13,%rcx
   34ca7:	shl    $0x7,%rcx
   34cab:	movdqa %xmm0,0x70(%rax,%rcx,1)
   34cb1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   34cb7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   34cbd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   34cc3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   34cc9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   34ccf:	movdqa %xmm0,0x10(%rax,%rcx,1)
   34cd5:	movdqa %xmm0,(%rax,%rcx,1)
   34cda:	mov    %r13,%rax
   34cdd:	inc    %rax
   34ce0:	mov    %rax,0x10(%rdi)
   34ce4:	mov    %r14,%rax
   34ce7:	shl    $0x6,%rax
   34ceb:	or     $0x10000000,%r13d
   34cf2:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   34cf8:	movdqa %xmm1,0x20(%rbx,%rax,1)
   34cfe:	movdqa %xmm1,0x10(%rbx,%rax,1)
   34d04:	movdqa %xmm1,(%rbx,%rax,1)
   34d09:	mov    %r13d,0x3c(%rbx,%rax,1)
   34d0e:	mov    %r14,%rdi
   34d11:	shr    $0x6,%rdi
   34d15:	cmp    %rsi,%rdi
   34d18:	jae    360ba <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   34d1e:	mov    $0x1,%eax
   34d23:	mov    %r14d,%ecx
   34d26:	shl    %cl,%rax
   34d29:	or     %rax,(%r8,%rdi,8)
   34d2d:	mov    %r14,%rcx
   34d30:	mov    0x8(%rsp),%r13
   34d35:	add    $0x4,%r12
   34d39:	cmp    $0x400,%r12
   34d40:	je     34de0 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
   34d46:	mov    %rbp,%rax
   34d49:	mov    (%r15,%r12,1),%ebp
   34d4d:	add    %rax,%rbp
   34d50:	cmp    %r13,%rbp
   34d53:	setae  %dil
   34d57:	mov    %rbp,%rax
   34d5a:	movabs $0x8888888888888889,%rdx
   34d64:	mul    %rdx
   34d67:	mov    %rdx,%r14
   34d6a:	shr    $0x4,%r14
   34d6e:	mov    %rbp,%rax
   34d71:	imul   %r9,%rax
   34d75:	ror    $1,%rax
   34d78:	cmp    %r10,%rax
   34d7b:	setb   %al
   34d7e:	cmp    %rcx,%r14
   34d81:	sete   %dl
   34d84:	or     %dil,%dl
   34d87:	or     %al,%dl
   34d89:	jne    34d35 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   34d8b:	mov    0x5520(%rsp),%rdi
   34d93:	mov    0x10(%rdi),%r13
   34d97:	cmp    (%rdi),%r13
   34d9a:	jne    34ca0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   34da0:	mov    %rsi,(%rsp)
   34da4:	mov    %r8,0x10(%rsp)
   34da9:	call   7c690 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   34dae:	pcmpeqd %xmm1,%xmm1
   34db2:	pxor   %xmm0,%xmm0
   34db6:	movabs $0x888888888888889,%r10
   34dc0:	movabs $0xeeeeeeeeeeeeeeef,%r9
   34dca:	mov    0x10(%rsp),%r8
   34dcf:	mov    (%rsp),%rsi
   34dd3:	mov    0x5520(%rsp),%rdi
   34ddb:	jmp    34ca0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   34de0:	cmpl   $0x0,0x1c(%rsp)
   34de5:	lea    0xd8(%rsp),%r8
   34ded:	je     34fa4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   34df3:	mov    0xc8(%rsp),%r11
   34dfb:	test   %r11,%r11
   34dfe:	je     34fa4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   34e04:	mov    0x8(%rsp),%r9
   34e09:	shl    $0x20,%r9
   34e0d:	mov    0x1c(%rsp),%r10d
   34e12:	mov    0x98(%rsp),%rdi
   34e1a:	mov    0xc0(%rsp),%r13
   34e22:	xor    %r12d,%r12d
   34e25:	movabs $0xff51afd7ed558ccd,%rbx
   34e2f:	pxor   %xmm0,%xmm0
   34e33:	pcmpeqd %xmm1,%xmm1
   34e37:	jmp    34ec0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
   34e3c:	mov    0x8(%rdi),%rax
   34e40:	mov    %rbx,%rcx
   34e43:	shl    $0x7,%rcx
   34e47:	movdqa %xmm0,0x70(%rax,%rcx,1)
   34e4d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   34e53:	movdqa %xmm0,0x50(%rax,%rcx,1)
   34e59:	movdqa %xmm0,0x40(%rax,%rcx,1)
   34e5f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   34e65:	movdqa %xmm0,0x20(%rax,%rcx,1)
   34e6b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   34e71:	movdqa %xmm0,(%rax,%rcx,1)
   34e76:	mov    %rbx,%rax
   34e79:	inc    %rax
   34e7c:	mov    %rax,0x10(%rdi)
   34e80:	or     $0x10000000,%ebx
   34e86:	movdqu %xmm1,0x2c(%r13)
   34e8c:	movdqa %xmm1,0x20(%r13)
   34e92:	movdqa %xmm1,0x10(%r13)
   34e98:	movdqa %xmm1,0x0(%r13)
   34e9e:	mov    %ebx,0x3c(%r13)
   34ea2:	or     %r15,(%r14,%rbp,8)
   34ea6:	movabs $0xff51afd7ed558ccd,%rbx
   34eb0:	inc    %r12
   34eb3:	add    $0x40,%r13
   34eb7:	cmp    %r12,%r11
   34eba:	je     34fa4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   34ec0:	mov    %r12,%rbp
   34ec3:	shr    $0x6,%rbp
   34ec7:	mov    0xa8(%rsp),%rsi
   34ecf:	cmp    %rsi,%rbp
   34ed2:	jae    360c6 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   34ed8:	mov    0xa0(%rsp),%r14
   34ee0:	mov    (%r14,%rbp,8),%rax
   34ee4:	mov    $0x1,%r15d
   34eea:	mov    %r12d,%ecx
   34eed:	shl    %cl,%r15
   34ef0:	bt     %r12,%rax
   34ef4:	jb     34eb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   34ef6:	mov    %r12,%rax
   34ef9:	xor    %r9,%rax
   34efc:	mov    %rax,%rcx
   34eff:	shr    $0x21,%rcx
   34f03:	xor    %rax,%rcx
   34f06:	imul   %rbx,%rcx
   34f0a:	mov    %rcx,%rax
   34f0d:	shr    $0x21,%rax
   34f11:	xor    %rcx,%rax
   34f14:	movabs $0xc4ceb9fe1a85ec53,%rcx
   34f1e:	imul   %rcx,%rax
   34f22:	mov    %rax,%rcx
   34f25:	shr    $0x21,%rcx
   34f29:	xor    %rax,%rcx
   34f2c:	mov    %rcx,%rax
   34f2f:	shr    $0x3,%rax
   34f33:	movabs $0x20c49ba5e353f7cf,%rdx
   34f3d:	mul    %rdx
   34f40:	shr    $0x4,%rdx
   34f44:	imul   $0x3e8,%rdx,%rax
   34f4b:	sub    %rax,%rcx
   34f4e:	cmp    %r10,%rcx
   34f51:	jae    34eb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   34f57:	mov    0x10(%rdi),%rbx
   34f5b:	cmp    (%rdi),%rbx
   34f5e:	jne    34e3c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   34f64:	mov    %r11,(%rsp)
   34f68:	mov    %r9,0x10(%rsp)
   34f6d:	mov    %r10,0x50(%rsp)
   34f72:	mov    %rdi,0x48(%rsp)
   34f77:	call   7c690 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   34f7c:	pcmpeqd %xmm1,%xmm1
   34f80:	pxor   %xmm0,%xmm0
   34f84:	mov    0x48(%rsp),%rdi
   34f89:	mov    0x50(%rsp),%r10
   34f8e:	mov    0x10(%rsp),%r9
   34f93:	mov    (%rsp),%r11
   34f97:	lea    0xd8(%rsp),%r8
   34f9f:	jmp    34e3c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   34fa4:	mov    0x8(%rsp),%r13
   34fa9:	test   %r13,%r13
   34fac:	mov    0x28(%rsp),%r14
   34fb1:	je     35cff <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   34fb7:	xor    %r11d,%r11d
   34fba:	jmp    34fcc <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   34fbc:	nopl   0x0(%rax)
   34fc0:	mov    %r11,%r10
   34fc3:	cmp    %r13,%r10
   34fc6:	jae    35cff <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   34fcc:	mov    %r11,%rbp
   34fcf:	mov    %r9,%r12
   34fd2:	mov    $0x400,%edx
   34fd7:	lea    0x28e8(%rsp),%rdi
   34fdf:	xor    %esi,%esi
   34fe1:	call   *0xd0c59(%rip)        # 105c40 <memset@GLIBC_2.2.5>
   34fe7:	lea    0x118(%rsp),%rax
   34fef:	xor    %ecx,%ecx
   34ff1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   35000:	mov    -0x40(%rax),%rdx
   35004:	sub    -0x30(%rax),%rdx
   35008:	shr    $1,%rdx
   3500b:	add    -0x28(%rax),%edx
   3500e:	mov    -0x18(%rax),%rsi
   35012:	sub    -0x8(%rax),%rsi
   35016:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3501d:	shr    $1,%rsi
   35020:	add    (%rax),%esi
   35022:	mov    %esi,0x28ec(%rsp,%rcx,4)
   35029:	add    $0x2,%rcx
   3502d:	add    $0x50,%rax
   35031:	cmp    $0x100,%rcx
   35038:	jne    35000 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
   3503a:	mov    0xb0(%rsp),%r15
   35042:	mov    0x10(%r15),%rbx
   35046:	cmp    (%r15),%rbx
   35049:	je     35cf2 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
   3504f:	mov    %rbx,%rdi
   35052:	shl    $0xa,%rdi
   35056:	add    0x8(%r15),%rdi
   3505a:	mov    $0x400,%edx
   3505f:	lea    0x28e8(%rsp),%rsi
   35067:	call   *0xd0bf3(%rip)        # 105c60 <memcpy@GLIBC_2.14>
   3506d:	inc    %rbx
   35070:	mov    %rbx,0x10(%r15)
   35074:	mov    %rbp,%r11
   35077:	lea    0xf000(%rbp),%r10
   3507e:	cmp    %r13,%r10
   35081:	cmovae %r13,%r10
   35085:	lea    0x10(%rbp),%rdi
   35089:	cmp    %r10,%rdi
   3508c:	mov    %r10,0x20(%rsp)
   35091:	jbe    35160 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   35097:	lea    0xd8(%rsp),%r8
   3509f:	mov    %r12,%r9
   350a2:	mov    0x38(%rsp),%rcx
   350a7:	mov    %r10,%rdx
   350aa:	sub    %r11,%rdx
   350ad:	mov    %rcx,0x38(%rsp)
   350b2:	jbe    34fc0 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   350b8:	mov    %r9,%rbp
   350bb:	mov    %r14,%rax
   350be:	mov    0x28e0(%rsp),%r12d
   350c6:	mov    $0x1e,%r14d
   350cc:	sub    0x28d8(%rsp),%r14
   350d4:	cmp    %rdx,%r14
   350d7:	cmovae %rdx,%r14
   350db:	lea    (%rax,%r11,1),%r15
   350df:	test   %r14,%r14
   350e2:	je     35b60 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   350e8:	xor    %r13d,%r13d
   350eb:	nopl   0x0(%rax,%rax,1)
   350f0:	movzbl (%r15,%r13,1),%esi
   350f5:	lea    (%rsi,%rsi,4),%rcx
   350f9:	lea    (%r8,%rcx,8),%rbx
   350fd:	mov    (%r8,%rcx,8),%rax
   35101:	cmp    0x8(%r8,%rcx,8),%rax
   35106:	je     35120 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   35108:	lea    (%r12,%r13,1),%rcx
   3510c:	mov    %cx,(%rax)
   3510f:	addq   $0x2,(%rbx)
   35113:	inc    %r13
   35116:	cmp    %r13,%r14
   35119:	jne    350f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   3511b:	jmp    35b60 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   35120:	mov    %rdx,0x10(%rsp)
   35125:	mov    %r11,(%rsp)
   35129:	lea    0x98(%rsp),%rdi
   35131:	call   76af0 <<c30walk::ctor2::W<false>>::refill>
   35136:	lea    0xd8(%rsp),%r8
   3513e:	test   %al,%al
   35140:	je     35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35146:	mov    (%rbx),%rax
   35149:	mov    0x20(%rsp),%r10
   3514e:	mov    (%rsp),%r11
   35152:	mov    0x10(%rsp),%rdx
   35157:	jmp    35108 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
   35159:	nopl   0x0(%rax)
   35160:	xor    %edx,%edx
   35162:	lea    0xd8(%rsp),%r8
   3516a:	mov    %r12,%r9
   3516d:	mov    0x38(%rsp),%rcx
   35172:	jmp    351d3 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   35174:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   35180:	mov    0x28d8(%rsp),%rax
   35188:	lea    0x10(%rax),%rcx
   3518c:	add    $0xfffffffffffffff2,%rax
   35190:	xor    %edx,%edx
   35192:	cmp    $0x1e,%rcx
   35196:	setae  %dl
   35199:	cmovb  %rcx,%rax
   3519d:	mov    %rax,0x28d8(%rsp)
   351a5:	mov    0x28e0(%rsp),%eax
   351ac:	lea    0x10(%rax,%rdx,2),%eax
   351b0:	mov    %eax,0x28e0(%rsp)
   351b7:	mov    0x30(%rsp),%r11
   351bc:	lea    0x10(%r11),%rsi
   351c0:	xor    %edx,%edx
   351c2:	mov    %rsi,%rdi
   351c5:	cmp    %r10,%rsi
   351c8:	mov    0x28(%rsp),%r14
   351cd:	ja     350a7 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
   351d3:	test   $0x1,%dl
   351d6:	jne    35204 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
   351d8:	lea    0x8(%r11),%rsi
   351dc:	cmp    $0xfffffffffffffff7,%r11
   351e0:	ja     35f88 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   351e6:	cmp    %r13,%rsi
   351e9:	ja     35f88 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   351ef:	lea    0x10(%r11),%rax
   351f3:	cmp    %r13,%rax
   351f6:	ja     35faf <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
   351fc:	mov    (%r14,%r11,1),%r9
   35200:	mov    (%r14,%rsi,1),%rcx
   35204:	mov    %r9,%rax
   35207:	xor    %rcx,%rax
   3520a:	mov    %r9,%rdx
   3520d:	rol    $0x8,%rdx
   35211:	xor    %r9,%rdx
   35214:	or     %rax,%rdx
   35217:	jne    35270 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   35219:	mov    %rdi,%rax
   3521c:	nopl   0x0(%rax)
   35220:	mov    %rax,%r12
   35223:	add    $0x10,%rax
   35227:	cmp    %r10,%rax
   3522a:	setbe  %r9b
   3522e:	ja     352d0 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
   35234:	lea    0x8(%r12),%rsi
   35239:	cmp    $0xfffffffffffffff7,%r12
   3523d:	ja     35eaa <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   35243:	cmp    %r13,%rsi
   35246:	ja     35eaa <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3524c:	mov    (%r14,%r12,1),%rdx
   35250:	mov    0x8(%r14,%r12,1),%rsi
   35255:	cmp    %rcx,%rdx
   35258:	jne    3525f <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   3525a:	cmp    %rcx,%rsi
   3525d:	je     35220 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   3525f:	mov    %rsi,0x38(%rsp)
   35264:	mov    %rdx,0x68(%rsp)
   35269:	lea    0x10(%r12),%r10
   3526e:	jmp    352d5 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   35270:	mov    %rdi,0x30(%rsp)
   35275:	mov    0x28d8(%rsp),%rax
   3527d:	mov    $0x1e,%r12d
   35283:	sub    %rax,%r12
   35286:	cmp    $0x10,%r12
   3528a:	mov    $0x10,%ebx
   3528f:	cmovb  %r12,%rbx
   35293:	mov    %rbx,%r15
   35296:	add    %r11,%r15
   35299:	jb     35f9a <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3529f:	cmp    %r13,%r15
   352a2:	ja     35f9a <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   352a8:	mov    0x28e0(%rsp),%r14d
   352b0:	cmp    $0x1e,%rax
   352b4:	jne    35a30 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
   352ba:	cmp    %r15,0x30(%rsp)
   352bf:	jae    35aba <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   352c5:	jmp    35fc4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   352ca:	nopw   0x0(%rax,%rax,1)
   352d0:	lea    0x10(%r12),%r10
   352d5:	mov    %r12,0x30(%rsp)
   352da:	sub    %r11,%r12
   352dd:	movzbl %cl,%eax
   352e0:	mov    %rax,0x90(%rsp)
   352e8:	lea    (%rax,%rax,4),%rdx
   352ec:	mov    (%r8,%rdx,8),%rax
   352f0:	mov    0x1c(%r8,%rdx,8),%esi
   352f5:	mov    %rax,%rdi
   352f8:	sub    0x10(%r8,%rdx,8),%rdi
   352fd:	shr    $1,%rdi
   35300:	add    0x18(%r8,%rdx,8),%edi
   35305:	add    %r12,%rdi
   35308:	cmp    %rsi,%rdi
   3530b:	ja     35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35311:	mov    %r10,0x70(%rsp)
   35316:	mov    %r9d,0x44(%rsp)
   3531b:	lea    (%r8,%rdx,8),%rbp
   3531f:	movzbl %cl,%r13d
   35323:	mov    0x8(%rbp),%rsi
   35327:	mov    $0x88888889,%r14d
   3532d:	lea    -0x1f634(%rip),%r15        # 15d00 <c30walk::ctor2::PATTERNS>
   35334:	mov    %rbp,0x88(%rsp)
   3533c:	jmp    35377 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   3533e:	xchg   %ax,%ax
   35340:	mov    0x3c(%rsi),%edi
   35343:	mov    $0xfffff,%r9d
   35349:	and    %r9d,%edi
   3534c:	sub    %r8d,%ecx
   3534f:	sub    %eax,%r8d
   35352:	mov    %r8d,%eax
   35355:	lea    (%rsi,%rax,2),%rax
   35359:	mov    %rax,0x10(%rbp)
   3535d:	mov    %rax,0x0(%rbp)
   35361:	lea    (%rax,%rcx,2),%rsi
   35365:	mov    %rsi,0x8(%rbp)
   35369:	mov    %edx,0x20(%rbp)
   3536c:	mov    %edi,0x24(%rbp)
   3536f:	lea    0xd8(%rsp),%r8
   35377:	sub    %rax,%rsi
   3537a:	shr    $1,%rsi
   3537d:	cmp    %rsi,%r12
   35380:	cmovb  %r12,%rsi
   35384:	test   %rsi,%rsi
   35387:	je     35441 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
   3538d:	mov    0x28d8(%rsp),%rdi
   35395:	cmp    $0x1d,%rdi
   35399:	ja     36098 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   3539f:	cmp    $0x1f,%rsi
   353a3:	jae    35ebc <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
   353a9:	mov    0x28e0(%rsp),%ecx
   353b0:	mov    %rdi,%r11
   353b3:	shl    $0x6,%r11
   353b7:	lea    (%r15,%r11,1),%rdx
   353bb:	lea    -0x2(,%rsi,2),%r9
   353c3:	cmp    $0xe,%r9
   353c7:	jb     353dc <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   353c9:	mov    %rax,%r8
   353cc:	sub    %r15,%r8
   353cf:	sub    %r11,%r8
   353d2:	cmp    $0x10,%r8
   353d6:	jae    35843 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
   353dc:	xor    %r10d,%r10d
   353df:	mov    %rdx,%r8
   353e2:	lea    (%rdx,%rsi,2),%rdx
   353e6:	lea    (%rax,%r10,2),%r9
   353ea:	nopw   0x0(%rax,%rax,1)
   353f0:	movzwl (%r8),%r10d
   353f4:	add    %cx,%r10w
   353f8:	add    $0x2,%r8
   353fc:	mov    %r10w,(%r9)
   35400:	add    $0x2,%r9
   35404:	cmp    %rdx,%r8
   35407:	jne    353f0 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
   35409:	lea    (%rax,%rsi,2),%rax
   3540d:	mov    %rax,0x0(%rbp)
   35411:	add    %rsi,%rdi
   35414:	lea    0x2(%rsi),%edx
   35417:	mov    %rdi,%r8
   3541a:	sub    $0x1e,%r8
   3541e:	cmovb  %rdi,%r8
   35422:	cmovb  %esi,%edx
   35425:	mov    %r8,0x28d8(%rsp)
   3542d:	add    %ecx,%edx
   3542f:	mov    %edx,0x28e0(%rsp)
   35436:	sub    %rsi,%r12
   35439:	lea    0xd8(%rsp),%r8
   35441:	test   %r12,%r12
   35444:	je     35a20 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   3544a:	mov    %rax,%r8
   3544d:	cmp    0x10(%rbp),%rax
   35451:	je     35470 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
   35453:	lea    0x98(%rsp),%rdi
   3545b:	mov    0x90(%rsp),%rsi
   35463:	call   764f0 <<c30walk::ctor2::W<false>>::finish>
   35468:	mov    0x0(%rbp),%r8
   3546c:	mov    0x10(%rbp),%rax
   35470:	sub    %rax,%r8
   35473:	shr    $1,%r8
   35476:	add    0x18(%rbp),%r8d
   3547a:	mov    %r8d,0x18(%rbp)
   3547e:	pxor   %xmm0,%xmm0
   35482:	movdqu %xmm0,0x0(%rbp)
   35487:	movq   $0x0,0x10(%rbp)
   3548f:	cmp    $0x1e,%r12
   35493:	jb     359b4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   35499:	mov    %r8,%rcx
   3549c:	imul   %r14,%rcx
   354a0:	shr    $0x24,%rcx
   354a4:	mov    %ecx,%eax
   354a6:	shl    $0x5,%eax
   354a9:	lea    (%rcx,%rcx,1),%edx
   354ac:	sub    %eax,%edx
   354ae:	add    %r8d,%edx
   354b1:	jne    359b4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   354b7:	mov    %r12,%rax
   354ba:	movabs $0x8888888888888889,%rdx
   354c4:	mul    %rdx
   354c7:	shr    $0x4,%rdx
   354cb:	mov    %ecx,%r9d
   354ce:	cmpl   $0x0,0x1c(%rsp)
   354d3:	mov    %r12,0x50(%rsp)
   354d8:	mov    %rdx,0x48(%rsp)
   354dd:	je     35800 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
   354e3:	mov    0x28e0(%rsp),%r10d
   354eb:	mov    0xa8(%rsp),%rax
   354f3:	mov    %rax,(%rsp)
   354f7:	mov    0x98(%rsp),%r11
   354ff:	mov    0xa0(%rsp),%rax
   35507:	mov    %rax,0x10(%rsp)
   3550c:	mov    0xc0(%rsp),%rax
   35514:	mov    0x28d8(%rsp),%rbx
   3551c:	mov    %rbx,%rdi
   3551f:	shl    $0x6,%rdi
   35523:	add    %r15,%rdi
   35526:	mov    %r9,%rcx
   35529:	shl    $0x6,%rcx
   3552d:	lea    (%rax,%rcx,1),%r12
   35531:	add    $0x3c,%r12
   35535:	mov    %r10d,%r15d
   35538:	shl    $0x8,%r15d
   3553c:	or     %r13d,%r15d
   3553f:	xor    %ebp,%ebp
   35541:	jmp    35771 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
   35546:	cs nopw 0x0(%rax,%rax,1)
   35550:	mov    0x8(%r11),%rcx
   35554:	shl    $0x7,%eax
   35557:	mov    %r15d,(%rcx,%rax,1)
   3555b:	movzwl 0x2(%rdi),%esi
   3555f:	lea    (%r10,%rbp,1),%r14d
   35563:	add    %r14d,%esi
   35566:	shl    $0x8,%esi
   35569:	or     %r13d,%esi
   3556c:	mov    %esi,0x4(%rcx,%rax,1)
   35570:	movzwl 0x4(%rdi),%esi
   35574:	add    %r14d,%esi
   35577:	shl    $0x8,%esi
   3557a:	or     %r13d,%esi
   3557d:	mov    %esi,0x8(%rcx,%rax,1)
   35581:	movzwl 0x6(%rdi),%esi
   35585:	add    %r14d,%esi
   35588:	shl    $0x8,%esi
   3558b:	or     %r13d,%esi
   3558e:	mov    %esi,0xc(%rcx,%rax,1)
   35592:	movzwl 0x8(%rdi),%esi
   35596:	add    %r14d,%esi
   35599:	shl    $0x8,%esi
   3559c:	or     %r13d,%esi
   3559f:	mov    %esi,0x10(%rcx,%rax,1)
   355a3:	movzwl 0xa(%rdi),%esi
   355a7:	add    %r14d,%esi
   355aa:	shl    $0x8,%esi
   355ad:	or     %r13d,%esi
   355b0:	mov    %esi,0x14(%rcx,%rax,1)
   355b4:	movzwl 0xc(%rdi),%esi
   355b8:	add    %r14d,%esi
   355bb:	shl    $0x8,%esi
   355be:	or     %r13d,%esi
   355c1:	mov    %esi,0x18(%rcx,%rax,1)
   355c5:	movzwl 0xe(%rdi),%esi
   355c9:	add    %r14d,%esi
   355cc:	shl    $0x8,%esi
   355cf:	or     %r13d,%esi
   355d2:	mov    %esi,0x1c(%rcx,%rax,1)
   355d6:	movzwl 0x10(%rdi),%esi
   355da:	add    %r14d,%esi
   355dd:	shl    $0x8,%esi
   355e0:	or     %r13d,%esi
   355e3:	mov    %esi,0x20(%rcx,%rax,1)
   355e7:	movzwl 0x12(%rdi),%esi
   355eb:	add    %r14d,%esi
   355ee:	shl    $0x8,%esi
   355f1:	or     %r13d,%esi
   355f4:	mov    %esi,0x24(%rcx,%rax,1)
   355f8:	movzwl 0x14(%rdi),%esi
   355fc:	add    %r14d,%esi
   355ff:	shl    $0x8,%esi
   35602:	or     %r13d,%esi
   35605:	mov    %esi,0x28(%rcx,%rax,1)
   35609:	movzwl 0x16(%rdi),%esi
   3560d:	add    %r14d,%esi
   35610:	shl    $0x8,%esi
   35613:	or     %r13d,%esi
   35616:	mov    %esi,0x2c(%rcx,%rax,1)
   3561a:	movzwl 0x18(%rdi),%esi
   3561e:	add    %r14d,%esi
   35621:	shl    $0x8,%esi
   35624:	or     %r13d,%esi
   35627:	mov    %esi,0x30(%rcx,%rax,1)
   3562b:	movzwl 0x1a(%rdi),%esi
   3562f:	add    %r14d,%esi
   35632:	shl    $0x8,%esi
   35635:	or     %r13d,%esi
   35638:	mov    %esi,0x34(%rcx,%rax,1)
   3563c:	movzwl 0x1c(%rdi),%esi
   35640:	add    %r14d,%esi
   35643:	shl    $0x8,%esi
   35646:	or     %r13d,%esi
   35649:	mov    %esi,0x38(%rcx,%rax,1)
   3564d:	movzwl 0x1e(%rdi),%esi
   35651:	add    %r14d,%esi
   35654:	shl    $0x8,%esi
   35657:	or     %r13d,%esi
   3565a:	mov    %esi,0x3c(%rcx,%rax,1)
   3565e:	movzwl 0x20(%rdi),%esi
   35662:	add    %r14d,%esi
   35665:	shl    $0x8,%esi
   35668:	or     %r13d,%esi
   3566b:	mov    %esi,0x40(%rcx,%rax,1)
   3566f:	movzwl 0x22(%rdi),%esi
   35673:	add    %r14d,%esi
   35676:	shl    $0x8,%esi
   35679:	or     %r13d,%esi
   3567c:	mov    %esi,0x44(%rcx,%rax,1)
   35680:	movzwl 0x24(%rdi),%esi
   35684:	add    %r14d,%esi
   35687:	shl    $0x8,%esi
   3568a:	or     %r13d,%esi
   3568d:	mov    %esi,0x48(%rcx,%rax,1)
   35691:	movzwl 0x26(%rdi),%esi
   35695:	add    %r14d,%esi
   35698:	shl    $0x8,%esi
   3569b:	or     %r13d,%esi
   3569e:	mov    %esi,0x4c(%rcx,%rax,1)
   356a2:	movzwl 0x28(%rdi),%esi
   356a6:	add    %r14d,%esi
   356a9:	shl    $0x8,%esi
   356ac:	or     %r13d,%esi
   356af:	mov    %esi,0x50(%rcx,%rax,1)
   356b3:	movzwl 0x2a(%rdi),%esi
   356b7:	add    %r14d,%esi
   356ba:	shl    $0x8,%esi
   356bd:	or     %r13d,%esi
   356c0:	mov    %esi,0x54(%rcx,%rax,1)
   356c4:	movzwl 0x2c(%rdi),%esi
   356c8:	add    %r14d,%esi
   356cb:	shl    $0x8,%esi
   356ce:	or     %r13d,%esi
   356d1:	mov    %esi,0x58(%rcx,%rax,1)
   356d5:	movzwl 0x2e(%rdi),%esi
   356d9:	add    %r14d,%esi
   356dc:	shl    $0x8,%esi
   356df:	or     %r13d,%esi
   356e2:	mov    %esi,0x5c(%rcx,%rax,1)
   356e6:	movzwl 0x30(%rdi),%esi
   356ea:	add    %r14d,%esi
   356ed:	shl    $0x8,%esi
   356f0:	or     %r13d,%esi
   356f3:	mov    %esi,0x60(%rcx,%rax,1)
   356f7:	movzwl 0x32(%rdi),%esi
   356fb:	add    %r14d,%esi
   356fe:	shl    $0x8,%esi
   35701:	or     %r13d,%esi
   35704:	mov    %esi,0x64(%rcx,%rax,1)
   35708:	movzwl 0x34(%rdi),%esi
   3570c:	add    %r14d,%esi
   3570f:	shl    $0x8,%esi
   35712:	or     %r13d,%esi
   35715:	mov    %esi,0x68(%rcx,%rax,1)
   35719:	movzwl 0x36(%rdi),%esi
   3571d:	add    %r14d,%esi
   35720:	shl    $0x8,%esi
   35723:	or     %r13d,%esi
   35726:	mov    %esi,0x6c(%rcx,%rax,1)
   3572a:	movzwl 0x38(%rdi),%esi
   3572e:	add    %r14d,%esi
   35731:	shl    $0x8,%esi
   35734:	or     %r13d,%esi
   35737:	mov    %esi,0x70(%rcx,%rax,1)
   3573b:	movzwl 0x3a(%rdi),%esi
   3573f:	add    %r14d,%esi
   35742:	shl    $0x8,%esi
   35745:	or     %r13d,%esi
   35748:	mov    %esi,0x74(%rcx,%rax,1)
   3574c:	lea    (%r10,%rbp,1),%eax
   35750:	add    $0x20,%eax
   35753:	mov    %eax,0x28e0(%rsp)
   3575a:	add    $0x20,%rbp
   3575e:	inc    %r9
   35761:	add    $0x2000,%r15d
   35768:	dec    %rdx
   3576b:	je     3596c <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
   35771:	mov    %r9,%rax
   35774:	shr    $0x6,%rax
   35778:	cmp    (%rsp),%rax
   3577c:	jae    35fdb <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   35782:	mov    0x10(%rsp),%rcx
   35787:	mov    (%rcx,%rax,8),%rax
   3578b:	bt     %r9,%rax
   3578f:	jae    357c0 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   35791:	cmp    $0x1e,%rbx
   35795:	jae    36000 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   3579b:	mov    (%r12,%rbp,2),%eax
   3579f:	and    $0xfffff,%eax
   357a4:	mov    0x10(%r11),%rcx
   357a8:	cmp    %rax,%rcx
   357ab:	ja     35550 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   357b1:	jmp    35fee <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
   357b6:	cs nopw 0x0(%rax,%rax,1)
   357c0:	cmp    $0x1e,%rbx
   357c4:	jae    36014 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
   357ca:	movups 0x2c(%rdi),%xmm0
   357ce:	movups %xmm0,-0x10(%r12,%rbp,2)
   357d4:	movdqa (%rdi),%xmm0
   357d8:	movdqa 0x10(%rdi),%xmm1
   357dd:	movdqa 0x20(%rdi),%xmm2
   357e2:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   357e9:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   357f0:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   357f7:	mov    %r15d,(%r12,%rbp,2)
   357fb:	jmp    3574c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
   35800:	mov    0x28d8(%rsp),%rdi
   35808:	cmp    $0x1d,%rdi
   3580c:	ja     360a9 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   35812:	shl    $0x6,%rdi
   35816:	add    %r15,%rdi
   35819:	mov    0x28e0(%rsp),%eax
   35820:	mov    0xc0(%rsp),%rcx
   35828:	shl    $0x6,%r9
   3582c:	mov    %eax,%r14d
   3582f:	shl    $0x8,%r14d
   35833:	or     %r13d,%r14d
   35836:	cmp    $0x3c,%r12
   3583a:	jae    358a5 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3583c:	xor    %esi,%esi
   3583e:	jmp    3592a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
   35843:	shr    $1,%r9
   35846:	inc    %r9
   35849:	mov    %r9,%r10
   3584c:	and    $0xfffffffffffffff8,%r10
   35850:	lea    (%rdx,%r10,2),%r8
   35854:	movd   %ecx,%xmm0
   35858:	pshuflw $0x0,%xmm0,%xmm0
   3585d:	add    %r15,%r11
   35860:	add    $0x8,%r11
   35864:	xor    %ebx,%ebx
   35866:	cs nopw 0x0(%rax,%rax,1)
   35870:	movq   -0x8(%r11,%rbx,2),%xmm1
   35877:	movq   (%r11,%rbx,2),%xmm2
   3587d:	paddw  %xmm0,%xmm1
   35881:	paddw  %xmm0,%xmm2
   35885:	punpcklqdq %xmm2,%xmm1
   35889:	movdqu %xmm1,(%rax,%rbx,2)
   3588e:	add    $0x8,%rbx
   35892:	cmp    %rbx,%r10
   35895:	jne    35870 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   35897:	cmp    %r10,%r9
   3589a:	jne    353e2 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   358a0:	jmp    35409 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
   358a5:	mov    %rdx,%r10
   358a8:	movabs $0xffffffffffffffe,%rsi
   358b2:	and    %rsi,%r10
   358b5:	lea    (%rcx,%r9,1),%r11
   358b9:	add    $0x7c,%r11
   358bd:	xor    %esi,%esi
   358bf:	nop
   358c0:	movdqu 0x2c(%rdi),%xmm0
   358c5:	movdqu %xmm0,-0x50(%r11)
   358cb:	movdqa (%rdi),%xmm1
   358cf:	movdqa 0x10(%rdi),%xmm2
   358d4:	movdqa 0x20(%rdi),%xmm3
   358d9:	movdqa %xmm3,-0x5c(%r11)
   358df:	movdqa %xmm2,-0x6c(%r11)
   358e5:	movdqa %xmm1,-0x7c(%r11)
   358eb:	mov    %r14d,-0x40(%r11)
   358ef:	add    $0x2,%rsi
   358f3:	movdqa %xmm1,-0x3c(%r11)
   358f9:	movdqa %xmm2,-0x2c(%r11)
   358ff:	movdqa %xmm3,-0x1c(%r11)
   35905:	movdqu %xmm0,-0x10(%r11)
   3590b:	lea    0x2000(%r14),%ebx
   35912:	mov    %ebx,(%r11)
   35915:	add    $0x4000,%r14d
   3591c:	sub    $0xffffffffffffff80,%r11
   35920:	cmp    %rsi,%r10
   35923:	jne    358c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   35925:	test   $0x1,%dl
   35928:	je     3595e <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
   3592a:	add    %r9,%rcx
   3592d:	shl    $0x6,%rsi
   35931:	movups 0x2c(%rdi),%xmm0
   35935:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3593a:	movdqa (%rdi),%xmm0
   3593e:	movdqa 0x10(%rdi),%xmm1
   35943:	movdqa 0x20(%rdi),%xmm2
   35948:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3594e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   35954:	movdqa %xmm0,(%rcx,%rsi,1)
   35959:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3595e:	mov    %edx,%ecx
   35960:	shl    $0x5,%ecx
   35963:	add    %ecx,%eax
   35965:	mov    %eax,0x28e0(%rsp)
   3596c:	mov    0x48(%rsp),%rdx
   35971:	mov    %rdx,%rax
   35974:	shl    $0x5,%rax
   35978:	lea    (%rdx,%rdx,1),%rcx
   3597c:	sub    %rax,%rcx
   3597f:	mov    0x50(%rsp),%r12
   35984:	add    %rcx,%r12
   35987:	lea    (%rdx,%rdx,1),%eax
   3598a:	shl    $0x5,%edx
   3598d:	sub    %eax,%edx
   3598f:	add    %edx,%r8d
   35992:	mov    0x88(%rsp),%rbp
   3599a:	mov    %r8d,0x18(%rbp)
   3599e:	test   %r12,%r12
   359a1:	mov    $0x88888889,%r14d
   359a7:	lea    -0x1fcae(%rip),%r15        # 15d00 <c30walk::ctor2::PATTERNS>
   359ae:	je     35b29 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
   359b4:	mov    0x1c(%rbp),%ecx
   359b7:	mov    %r8d,%edi
   359ba:	imul   %r14,%rdi
   359be:	mov    %rdi,%rdx
   359c1:	shr    $0x24,%rdx
   359c5:	mov    %edx,%eax
   359c7:	shl    $0x5,%eax
   359ca:	sub    %edx,%eax
   359cc:	sub    %edx,%eax
   359ce:	lea    0x1e(%rax),%esi
   359d1:	cmp    %ecx,%esi
   359d3:	cmovb  %esi,%ecx
   359d6:	shr    $0x2a,%rdi
   359da:	mov    0xa8(%rsp),%rsi
   359e2:	cmp    %rsi,%rdi
   359e5:	jae    3608c <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
   359eb:	mov    %rdx,%rsi
   359ee:	shl    $0x6,%rsi
   359f2:	add    0xc0(%rsp),%rsi
   359fa:	mov    0xa0(%rsp),%r9
   35a02:	mov    (%r9,%rdi,8),%rdi
   35a06:	bt     %rdx,%rdi
   35a0a:	jb     35340 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   35a10:	mov    $0xffffffff,%edi
   35a15:	jmp    3534c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
   35a1a:	nopw   0x0(%rax,%rax,1)
   35a20:	mov    0x8(%rsp),%r13
   35a25:	jmp    35b36 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   35a2a:	nopw   0x0(%rax,%rax,1)
   35a30:	add    0x28(%rsp),%r11
   35a35:	xor    %r13d,%r13d
   35a38:	nopl   0x0(%rax,%rax,1)
   35a40:	movzbl (%r11,%r13,1),%esi
   35a45:	lea    (%rsi,%rsi,4),%rcx
   35a49:	lea    (%r8,%rcx,8),%rbp
   35a4d:	mov    (%r8,%rcx,8),%rax
   35a51:	cmp    0x8(%r8,%rcx,8),%rax
   35a56:	je     35a6e <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   35a58:	lea    (%r14,%r13,1),%rcx
   35a5c:	mov    %cx,(%rax)
   35a5f:	addq   $0x2,0x0(%rbp)
   35a64:	inc    %r13
   35a67:	cmp    %r13,%rbx
   35a6a:	jne    35a40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   35a6c:	jmp    35aa0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   35a6e:	mov    %r11,(%rsp)
   35a72:	lea    0x98(%rsp),%rdi
   35a7a:	call   76af0 <<c30walk::ctor2::W<false>>::refill>
   35a7f:	lea    0xd8(%rsp),%r8
   35a87:	test   %al,%al
   35a89:	je     35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35a8f:	mov    0x0(%rbp),%rax
   35a93:	mov    0x20(%rsp),%r10
   35a98:	mov    (%rsp),%r11
   35a9c:	jmp    35a58 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
   35a9e:	xchg   %ax,%ax
   35aa0:	cmp    %r15,0x30(%rsp)
   35aa5:	mov    0x8(%rsp),%r13
   35aaa:	jb     35fc4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   35ab0:	cmp    $0xf,%r12
   35ab4:	ja     35180 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   35aba:	add    0x28(%rsp),%r15
   35abf:	mov    %r15,%r12
   35ac2:	sub    %rbx,%r12
   35ac5:	add    $0x10,%r12
   35ac9:	add    %r14d,%ebx
   35acc:	add    $0x2,%ebx
   35acf:	nop
   35ad0:	movzbl (%r15),%esi
   35ad4:	lea    (%rsi,%rsi,4),%rcx
   35ad8:	lea    (%r8,%rcx,8),%r14
   35adc:	mov    (%r8,%rcx,8),%rax
   35ae0:	cmp    0x8(%r8,%rcx,8),%rax
   35ae5:	je     35afd <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   35ae7:	mov    %bx,(%rax)
   35aea:	addq   $0x2,(%r14)
   35aee:	inc    %r15
   35af1:	inc    %ebx
   35af3:	cmp    %r12,%r15
   35af6:	jne    35ad0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   35af8:	jmp    35180 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   35afd:	lea    0x98(%rsp),%rdi
   35b05:	call   76af0 <<c30walk::ctor2::W<false>>::refill>
   35b0a:	lea    0xd8(%rsp),%r8
   35b12:	test   %al,%al
   35b14:	je     35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35b1a:	mov    (%r14),%rax
   35b1d:	mov    0x8(%rsp),%r13
   35b22:	mov    0x20(%rsp),%r10
   35b27:	jmp    35ae7 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
   35b29:	mov    0x8(%rsp),%r13
   35b2e:	lea    0xd8(%rsp),%r8
   35b36:	mov    0x20(%rsp),%r10
   35b3b:	mov    0x68(%rsp),%r9
   35b40:	mov    0x38(%rsp),%rcx
   35b45:	mov    0x30(%rsp),%r11
   35b4a:	mov    0x44(%rsp),%edx
   35b4e:	mov    0x70(%rsp),%rsi
   35b53:	jmp    351c2 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
   35b58:	nopl   0x0(%rax,%rax,1)
   35b60:	lea    (%r14,%r11,1),%rax
   35b64:	cmp    %r10,%rax
   35b67:	jne    35bb1 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
   35b69:	cmp    $0x10,%rdx
   35b6d:	jne    35c27 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
   35b73:	mov    0x28d8(%rsp),%rax
   35b7b:	lea    0x10(%rax),%rcx
   35b7f:	cmp    $0x1d,%rcx
   35b83:	mov    0x8(%rsp),%r13
   35b88:	mov    0x28(%rsp),%r14
   35b8d:	mov    %rbp,%r9
   35b90:	jbe    35ccb <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
   35b96:	add    $0xfffffffffffffff2,%rax
   35b9a:	mov    %rax,0x28d8(%rsp)
   35ba2:	mov    0x28e0(%rsp),%eax
   35ba9:	add    $0x12,%eax
   35bac:	jmp    35cdd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   35bb1:	add    $0x2,%r12d
   35bb5:	movzwl %r12w,%r12d
   35bb9:	nopl   0x0(%rax)
   35bc0:	movzbl (%r15,%r14,1),%esi
   35bc5:	lea    (%rsi,%rsi,4),%rcx
   35bc9:	lea    (%r8,%rcx,8),%rbx
   35bcd:	mov    (%r8,%rcx,8),%rax
   35bd1:	cmp    0x8(%r8,%rcx,8),%rax
   35bd6:	je     35bf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   35bd8:	lea    (%r12,%r14,1),%rcx
   35bdc:	mov    %cx,(%rax)
   35bdf:	addq   $0x2,(%rbx)
   35be3:	inc    %r14
   35be6:	cmp    %r14,%rdx
   35be9:	jne    35bc0 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   35beb:	jmp    35b69 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
   35bf0:	mov    %rdx,0x10(%rsp)
   35bf5:	mov    %r11,%r13
   35bf8:	lea    0x98(%rsp),%rdi
   35c00:	call   76af0 <<c30walk::ctor2::W<false>>::refill>
   35c05:	lea    0xd8(%rsp),%r8
   35c0d:	test   %al,%al
   35c0f:	je     35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35c15:	mov    (%rbx),%rax
   35c18:	mov    0x20(%rsp),%r10
   35c1d:	mov    %r13,%r11
   35c20:	mov    0x10(%rsp),%rdx
   35c25:	jmp    35bd8 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
   35c27:	mov    0x28e0(%rsp),%eax
   35c2e:	mov    0x28d8(%rsp),%rcx
   35c36:	inc    %r11
   35c39:	cmp    %r11,%r10
   35c3c:	mov    0x8(%rsp),%r13
   35c41:	mov    0x28(%rsp),%r14
   35c46:	mov    %rbp,%r9
   35c49:	je     35ca6 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   35c4b:	mov    %rdx,%r10
   35c4e:	and    $0xffffe,%edx
   35c54:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   35c60:	lea    0x1(%rcx),%rsi
   35c64:	add    $0xffffffffffffffe3,%rcx
   35c68:	xor    %edi,%edi
   35c6a:	cmp    $0x1e,%rsi
   35c6e:	setae  %dil
   35c72:	cmovb  %rsi,%rcx
   35c76:	lea    (%rax,%rdi,2),%eax
   35c79:	lea    0x1(%rcx),%rsi
   35c7d:	add    $0xffffffffffffffe3,%rcx
   35c81:	xor    %edi,%edi
   35c83:	cmp    $0x1e,%rsi
   35c87:	setae  %dil
   35c8b:	cmovb  %rsi,%rcx
   35c8f:	lea    (%rax,%rdi,2),%eax
   35c92:	add    $0x2,%eax
   35c95:	add    $0xfffffffffffffffe,%rdx
   35c99:	jne    35c60 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   35c9b:	test   $0x1,%r10b
   35c9f:	mov    0x20(%rsp),%r10
   35ca4:	je     35cc1 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
   35ca6:	lea    0x1(%rcx),%rdx
   35caa:	add    $0xffffffffffffffe3,%rcx
   35cae:	xor    %esi,%esi
   35cb0:	cmp    $0x1e,%rdx
   35cb4:	setae  %sil
   35cb8:	cmovb  %rdx,%rcx
   35cbc:	lea    (%rax,%rsi,2),%eax
   35cbf:	inc    %eax
   35cc1:	mov    %rcx,0x28d8(%rsp)
   35cc9:	jmp    35cdd <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   35ccb:	mov    %rcx,0x28d8(%rsp)
   35cd3:	mov    0x28e0(%rsp),%eax
   35cda:	add    $0x10,%eax
   35cdd:	mov    %eax,0x28e0(%rsp)
   35ce4:	mov    %r10,%r11
   35ce7:	cmp    %r13,%r10
   35cea:	jb     34fcc <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   35cf0:	jmp    35cff <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   35cf2:	mov    %r15,%rdi
   35cf5:	call   b8b20 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   35cfa:	jmp    3504f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
   35cff:	mov    $0x84,%eax
   35d04:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   35d10:	lea    0x50(%rax),%rcx
   35d14:	cmp    $0x28d4,%rcx
   35d1b:	je     35d7b <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
   35d1d:	mov    0x54(%rsp,%rax,1),%rdx
   35d22:	sub    0x64(%rsp,%rax,1),%rdx
   35d27:	shr    $1,%rdx
   35d2a:	add    0x6c(%rsp,%rax,1),%edx
   35d2e:	cmp    0x70(%rsp,%rax,1),%edx
   35d32:	jne    35d57 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   35d34:	mov    0x7c(%rsp,%rax,1),%rdx
   35d39:	sub    0x8c(%rsp,%rax,1),%rdx
   35d41:	shr    $1,%rdx
   35d44:	add    0x94(%rsp,%rax,1),%edx
   35d4b:	cmp    0x98(%rsp,%rax,1),%edx
   35d52:	mov    %rcx,%rax
   35d55:	je     35d10 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
   35d57:	mov    0x5520(%rsp),%rax
   35d5f:	movq   $0x0,0x10(%rax)
   35d67:	xor    %eax,%eax
   35d69:	add    $0x54e8,%rsp
   35d70:	pop    %rbx
   35d71:	pop    %r12
   35d73:	pop    %r13
   35d75:	pop    %r14
   35d77:	pop    %r15
   35d79:	pop    %rbp
   35d7a:	ret
   35d7b:	xor    %r14d,%r14d
   35d7e:	lea    0x98(%rsp),%r15
   35d86:	mov    0x78(%rsp),%r12
   35d8b:	mov    0x8(%rsp),%r13
   35d90:	mov    0x60(%rsp),%rbp
   35d95:	jmp    35dcb <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   35d97:	nopw   0x0(%rax,%rax,1)
   35da0:	sub    %rcx,%rax
   35da3:	inc    %r14
   35da6:	shr    $1,%rax
   35da9:	add    %eax,0x18(%r8)
   35dad:	pxor   %xmm0,%xmm0
   35db1:	movdqu %xmm0,(%r8)
   35db6:	movq   $0x0,0x10(%r8)
   35dbe:	add    $0x28,%r8
   35dc2:	cmp    $0x100,%r14
   35dc9:	je     35df1 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
   35dcb:	mov    (%r8),%rcx
   35dce:	mov    %rcx,%rax
   35dd1:	cmp    0x10(%r8),%rcx
   35dd5:	je     35da0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   35dd7:	mov    %r15,%rdi
   35dda:	mov    %r14,%rsi
   35ddd:	mov    %r8,%rbx
   35de0:	call   764f0 <<c30walk::ctor2::W<false>>::finish>
   35de5:	mov    %rbx,%r8
   35de8:	mov    (%rbx),%rax
   35deb:	mov    0x10(%rbx),%rcx
   35def:	jmp    35da0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   35df1:	xor    %ecx,%ecx
   35df3:	mov    0x8(%r12),%rbx
   35df8:	test   %rbp,%rbp
   35dfb:	setne  %al
   35dfe:	je     35e42 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   35e00:	mov    %al,%cl
   35e02:	mov    %rbp,%rax
   35e05:	sub    %rcx,%rax
   35e08:	imul   $0xffffffffffffffe2,%rax,%rcx
   35e0c:	add    %r13,%rcx
   35e0f:	cmp    $0x1e,%rcx
   35e13:	jae    35e42 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   35e15:	lea    (%rbx,%rbp,4),%rcx
   35e19:	lea    (%rcx,%r13,2),%rdi
   35e1d:	add    $0xfffffffffffffffc,%rdi
   35e21:	lea    (%rax,%rax,1),%rcx
   35e25:	shl    $0x5,%rax
   35e29:	sub    %rcx,%rax
   35e2c:	sub    %r13,%rax
   35e2f:	lea    0x3c(,%rax,2),%rdx
   35e37:	mov    $0xff,%esi
   35e3c:	call   *0xcfdfe(%rip)        # 105c40 <memset@GLIBC_2.2.5>
   35e42:	shl    $0x6,%rbp
   35e46:	pcmpeqd %xmm0,%xmm0
   35e4a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   35e50:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   35e56:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   35e5c:	movdqa %xmm0,(%rbx,%rbp,1)
   35e61:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   35e69:	mov    0x58(%rsp),%rax
   35e6e:	mov    0x8(%rax),%rdx
   35e72:	mov    0x10(%rax),%rcx
   35e76:	mov    0x5520(%rsp),%rax
   35e7e:	mov    0x8(%rax),%r8
   35e82:	mov    0x10(%rax),%r9
   35e86:	lea    0x98(%rsp),%rdi
   35e8e:	mov    %rbx,%rsi
   35e91:	call   84b60 <c30walk::ctor::canonicalise>
   35e96:	mov    0x80(%rsp),%rax
   35e9e:	mov    %rax,0x10(%r12)
   35ea3:	mov    $0x1,%al
   35ea5:	jmp    35d69 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
   35eaa:	lea    0xcd37f(%rip),%rcx        # 103230 <__frame_dummy_init_array_entry+0x2890>
   35eb1:	mov    %r12,%rdi
   35eb4:	mov    %r13,%rdx
   35eb7:	call   aad00 <core::slice::index::slice_index_fail>
   35ebc:	lea    0xcc515(%rip),%rcx        # 1023d8 <__frame_dummy_init_array_entry+0x1a38>
   35ec3:	mov    $0x1e,%edx
   35ec8:	xor    %edi,%edi
   35eca:	call   aad00 <core::slice::index::slice_index_fail>
   35ecf:	mov    %rdi,%r12
   35ed2:	test   %rax,%rax
   35ed5:	je     35ee7 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
   35ed7:	mov    0x8(%r9),%rdi
   35edb:	mov    %r9,%r14
   35ede:	call   *0xcfd6c(%rip)        # 105c50 <free@GLIBC_2.2.5>
   35ee4:	mov    %r14,%r9
   35ee7:	movq   $0x0,(%r9)
   35eee:	movq   $0x40,0x8(%r9)
   35ef6:	movq   $0x0,0x10(%r9)
   35efe:	mov    $0x40,%edx
   35f03:	mov    %r9,%rdi
   35f06:	lea    0x1(%rbp),%rsi
   35f0a:	mov    %r9,%r14
   35f0d:	call   7ba90 <<alloc::raw_vec::RawVecInner>::grow_exact>
   35f12:	mov    %r14,%r9
   35f15:	cmp    $0xffffffffffffffff,%rax
   35f19:	mov    %r12,%rdi
   35f1c:	je     34a15 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   35f22:	jmp    35f7d <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
   35f24:	mov    %rdi,%r12
   35f27:	mov    %r9,%r14
   35f2a:	test   %rax,%rax
   35f2d:	je     35f41 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
   35f2f:	mov    0x5520(%rsp),%rax
   35f37:	mov    0x8(%rax),%rdi
   35f3b:	call   *0xcfd0f(%rip)        # 105c50 <free@GLIBC_2.2.5>
   35f41:	mov    0x5520(%rsp),%rdi
   35f49:	movq   $0x0,(%rdi)
   35f50:	movq   $0x40,0x8(%rdi)
   35f58:	movq   $0x0,0x10(%rdi)
   35f60:	mov    $0x80,%edx
   35f65:	mov    %rbx,%rsi
   35f68:	call   7ba90 <<alloc::raw_vec::RawVecInner>::grow_exact>
   35f6d:	cmp    $0xffffffffffffffff,%rax
   35f71:	mov    %r14,%r9
   35f74:	mov    %r12,%rdi
   35f77:	je     34a29 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
   35f7d:	mov    %rax,%rdi
   35f80:	mov    %rdx,%rsi
   35f83:	call   a848b <alloc::raw_vec::handle_error>
   35f88:	lea    0xcd2a1(%rip),%rcx        # 103230 <__frame_dummy_init_array_entry+0x2890>
   35f8f:	mov    %r11,%rdi
   35f92:	mov    %r13,%rdx
   35f95:	call   aad00 <core::slice::index::slice_index_fail>
   35f9a:	lea    0xcc467(%rip),%rcx        # 102408 <__frame_dummy_init_array_entry+0x1a68>
   35fa1:	mov    %r11,%rdi
   35fa4:	mov    %r15,%rsi
   35fa7:	mov    %r13,%rdx
   35faa:	call   aad00 <core::slice::index::slice_index_fail>
   35faf:	lea    0xcd27a(%rip),%rcx        # 103230 <__frame_dummy_init_array_entry+0x2890>
   35fb6:	mov    %rsi,%rdi
   35fb9:	mov    %rax,%rsi
   35fbc:	mov    %r13,%rdx
   35fbf:	call   aad00 <core::slice::index::slice_index_fail>
   35fc4:	lea    0xcc425(%rip),%rcx        # 1023f0 <__frame_dummy_init_array_entry+0x1a50>
   35fcb:	mov    %r15,%rdi
   35fce:	mov    0x30(%rsp),%rsi
   35fd3:	mov    %r13,%rdx
   35fd6:	call   aad00 <core::slice::index::slice_index_fail>
   35fdb:	lea    0xcabfe(%rip),%rdx        # 100be0 <__frame_dummy_init_array_entry+0x240>
   35fe2:	mov    %rax,%rdi
   35fe5:	mov    (%rsp),%rsi
   35fe9:	call   a9f23 <core::panicking::panic_bounds_check>
   35fee:	lea    0xcc4a3(%rip),%rdx        # 102498 <__frame_dummy_init_array_entry+0x1af8>
   35ff5:	mov    %rax,%rdi
   35ff8:	mov    %rcx,%rsi
   35ffb:	call   a9f23 <core::panicking::panic_bounds_check>
   36000:	lea    0xcc479(%rip),%rdx        # 102480 <__frame_dummy_init_array_entry+0x1ae0>
   36007:	mov    $0x1e,%esi
   3600c:	mov    %rbx,%rdi
   3600f:	call   a9f23 <core::panicking::panic_bounds_check>
   36014:	lea    0xcc44d(%rip),%rdx        # 102468 <__frame_dummy_init_array_entry+0x1ac8>
   3601b:	mov    $0x1e,%esi
   36020:	mov    %rbx,%rdi
   36023:	call   a9f23 <core::panicking::panic_bounds_check>
   36028:	mov    $0x10,%edx
   3602d:	mov    $0x4,%ecx
   36032:	mov    $0x400,%r8d
   36038:	mov    %rdi,%r14
   3603b:	mov    %r12,%rdi
   3603e:	xor    %esi,%esi
   36040:	mov    %r9,%rbx
   36043:	call   58c70 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   36048:	mov    %r14,%rdi
   3604b:	mov    %rbx,%r9
   3604e:	jmp    34ac0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
   36053:	mov    %r9,%r13
   36056:	mov    $0x8,%ecx
   3605b:	mov    $0x8,%r8d
   36061:	mov    %rdi,%r14
   36064:	xor    %esi,%esi
   36066:	mov    %rbx,%rdx
   36069:	call   58c70 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3606e:	mov    %r14,%rdi
   36071:	mov    0x10(%r14),%r14
   36075:	mov    0x8(%rdi),%r12
   36079:	lea    (%r12,%r14,8),%rdi
   3607d:	cmp    $0x1,%rbx
   36081:	jne    34a67 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   36087:	jmp    34a89 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3608c:	lea    0xcab4d(%rip),%rdx        # 100be0 <__frame_dummy_init_array_entry+0x240>
   36093:	call   a9f23 <core::panicking::panic_bounds_check>
   36098:	lea    0xcc321(%rip),%rdx        # 1023c0 <__frame_dummy_init_array_entry+0x1a20>
   3609f:	mov    $0x1e,%esi
   360a4:	call   a9f23 <core::panicking::panic_bounds_check>
   360a9:	lea    0xcc3a0(%rip),%rdx        # 102450 <__frame_dummy_init_array_entry+0x1ab0>
   360b0:	mov    $0x1e,%esi
   360b5:	call   a9f23 <core::panicking::panic_bounds_check>
   360ba:	lea    0xcc287(%rip),%rdx        # 102348 <__frame_dummy_init_array_entry+0x19a8>
   360c1:	call   a9f23 <core::panicking::panic_bounds_check>
   360c6:	lea    0xcab13(%rip),%rdx        # 100be0 <__frame_dummy_init_array_entry+0x240>
   360cd:	mov    %rbp,%rdi
   360d0:	call   a9f23 <core::panicking::panic_bounds_check>
