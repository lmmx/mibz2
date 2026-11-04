# <c30walk::ctor2::Ctor2>::build::<false>
# size 0x18e5

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003a840 <<c30walk::ctor2::Ctor2>::build::<false>>:
   3a840:	push   %rbp
   3a841:	push   %r15
   3a843:	push   %r14
   3a845:	push   %r13
   3a847:	push   %r12
   3a849:	push   %rbx
   3a84a:	sub    $0x1000,%rsp
   3a851:	movq   $0x0,(%rsp)
   3a859:	sub    $0x1000,%rsp
   3a860:	movq   $0x0,(%rsp)
   3a868:	sub    $0x1000,%rsp
   3a86f:	movq   $0x0,(%rsp)
   3a877:	sub    $0x1000,%rsp
   3a87e:	movq   $0x0,(%rsp)
   3a886:	sub    $0x1000,%rsp
   3a88d:	movq   $0x0,(%rsp)
   3a895:	sub    $0x4e8,%rsp
   3a89c:	mov    %r8d,0x1c(%rsp)
   3a8a1:	mov    0x5520(%rsp),%rax
   3a8a9:	movq   $0x0,0x10(%r9)
   3a8b1:	movq   $0x0,0x10(%rax)
   3a8b9:	cmp    $0xeffe2,%rdx
   3a8c0:	ja     3bdb7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3a8c6:	mov    %rcx,%r15
   3a8c9:	mov    %rdx,%r13
   3a8cc:	pxor   %xmm0,%xmm0
   3a8d0:	xor    %eax,%eax
   3a8d2:	pxor   %xmm2,%xmm2
   3a8d6:	pxor   %xmm1,%xmm1
   3a8da:	nopw   0x0(%rax,%rax,1)
   3a8e0:	movq   (%r15,%rax,4),%xmm3
   3a8e6:	movq   0x8(%r15,%rax,4),%xmm4
   3a8ed:	punpckldq %xmm0,%xmm3
   3a8f1:	paddq  %xmm2,%xmm3
   3a8f5:	punpckldq %xmm0,%xmm4
   3a8f9:	paddq  %xmm1,%xmm4
   3a8fd:	movq   0x10(%r15,%rax,4),%xmm2
   3a904:	movq   0x18(%r15,%rax,4),%xmm1
   3a90b:	punpckldq %xmm0,%xmm2
   3a90f:	paddq  %xmm3,%xmm2
   3a913:	punpckldq %xmm0,%xmm1
   3a917:	paddq  %xmm4,%xmm1
   3a91b:	add    $0x8,%rax
   3a91f:	cmp    $0x100,%rax
   3a925:	jne    3a8e0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   3a927:	paddq  %xmm2,%xmm1
   3a92b:	pshufd $0xee,%xmm1,%xmm0
   3a930:	paddq  %xmm1,%xmm0
   3a934:	movq   %xmm0,%rax
   3a939:	cmp    %r13,%rax
   3a93c:	jne    3bdb7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3a942:	pxor   %xmm2,%xmm2
   3a946:	xor    %eax,%eax
   3a948:	movdqa -0x2b270(%rip),%xmm3        # f6e0 <GCC_except_table972+0xa0c>
   3a950:	pxor   %xmm1,%xmm1
   3a954:	pxor   %xmm0,%xmm0
   3a958:	nopl   0x0(%rax,%rax,1)
   3a960:	movq   (%r15,%rax,4),%xmm4
   3a966:	movq   0x8(%r15,%rax,4),%xmm5
   3a96d:	pshufd $0xd4,%xmm4,%xmm4
   3a972:	pcmpeqd %xmm2,%xmm4
   3a976:	pandn  %xmm3,%xmm4
   3a97a:	paddq  %xmm1,%xmm4
   3a97e:	pshufd $0xd4,%xmm5,%xmm5
   3a983:	pcmpeqd %xmm2,%xmm5
   3a987:	pandn  %xmm3,%xmm5
   3a98b:	paddq  %xmm0,%xmm5
   3a98f:	movq   0x10(%r15,%rax,4),%xmm0
   3a996:	movq   0x18(%r15,%rax,4),%xmm6
   3a99d:	pshufd $0xd4,%xmm0,%xmm1
   3a9a2:	pcmpeqd %xmm2,%xmm1
   3a9a6:	pandn  %xmm3,%xmm1
   3a9aa:	paddq  %xmm4,%xmm1
   3a9ae:	pshufd $0xd4,%xmm6,%xmm0
   3a9b3:	pcmpeqd %xmm2,%xmm0
   3a9b7:	pandn  %xmm3,%xmm0
   3a9bb:	paddq  %xmm5,%xmm0
   3a9bf:	add    $0x8,%rax
   3a9c3:	cmp    $0x100,%rax
   3a9c9:	jne    3a960 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
   3a9cb:	mov    $0x88888889,%ebp
   3a9d0:	imul   %r13,%rbp
   3a9d4:	shr    $0x24,%rbp
   3a9d8:	imul   $0xeeeeeeef,%r13d,%eax
   3a9df:	ror    $1,%eax
   3a9e1:	cmp    $0x8888889,%eax
   3a9e6:	sbb    $0xffffffffffffffff,%rbp
   3a9ea:	mov    %rbp,%rbx
   3a9ed:	cmpl   $0x0,0x1c(%rsp)
   3a9f2:	jne    3aa48 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3a9f4:	test   %r13,%r13
   3a9f7:	je     3aa46 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   3a9f9:	paddq  %xmm1,%xmm0
   3a9fd:	pshufd $0xee,%xmm0,%xmm1
   3aa02:	paddq  %xmm0,%xmm1
   3aa06:	movq   %xmm1,%rax
   3aa0b:	lea    -0x1(%r13),%ecx
   3aa0f:	mov    $0x88888889,%edx
   3aa14:	imul   %rcx,%rdx
   3aa18:	shr    $0x23,%rdx
   3aa1c:	and    $0xfffffffe,%edx
   3aa1f:	lea    (%rdx,%r13,1),%ecx
   3aa23:	dec    %ecx
   3aa25:	shr    $1,%ecx
   3aa27:	mov    $0x80010003,%ebx
   3aa2c:	imul   %rcx,%rbx
   3aa30:	shr    $0x2e,%rbx
   3aa34:	inc    %ebx
   3aa36:	imul   %rax,%rbx
   3aa3a:	dec    %rbx
   3aa3d:	cmp    %rbp,%rbx
   3aa40:	cmovae %rbp,%rbx
   3aa44:	jmp    3aa48 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3aa46:	xor    %ebx,%ebx
   3aa48:	mov    (%r9),%rax
   3aa4b:	lea    0x1(%rbp),%rcx
   3aa4f:	mov    %rcx,0x80(%rsp)
   3aa57:	cmp    %rbp,%rax
   3aa5a:	mov    %rsi,0x28(%rsp)
   3aa5f:	jbe    3bf1f <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
   3aa65:	mov    0x5520(%rsp),%rax
   3aa6d:	mov    (%rax),%rax
   3aa70:	cmp    %rbx,%rax
   3aa73:	jb     3bf74 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
   3aa79:	movq   $0x0,0x10(%rdi)
   3aa81:	lea    0x3f(%rbp),%rbx
   3aa85:	shr    $0x6,%rbx
   3aa89:	mov    %r13,0x8(%rsp)
   3aa8e:	mov    %rdi,0x58(%rsp)
   3aa93:	mov    %rbp,0x60(%rsp)
   3aa98:	je     3aaf2 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
   3aa9a:	cmp    (%rdi),%rbx
   3aa9d:	ja     3c0a3 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   3aaa3:	mov    %r9,%r13
   3aaa6:	xor    %r14d,%r14d
   3aaa9:	mov    0x8(%rdi),%r12
   3aaad:	lea    (%r12,%r14,8),%rdi
   3aab1:	cmp    $0x1,%rbx
   3aab5:	je     3aad9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3aab7:	lea    -0x8(,%rbx,8),%rdx
   3aabf:	xor    %esi,%esi
   3aac1:	call   *0xfa3c9(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   3aac7:	lea    (%r14,%rbx,1),%rax
   3aacb:	lea    (%r12,%rax,8),%rdi
   3aacf:	add    $0xfffffffffffffff8,%rdi
   3aad3:	add    %rbx,%r14
   3aad6:	dec    %r14
   3aad9:	mov    %r13,%r9
   3aadc:	movq   $0x0,(%rdi)
   3aae3:	inc    %r14
   3aae6:	mov    0x8(%rsp),%r13
   3aaeb:	mov    0x58(%rsp),%rdi
   3aaf0:	jmp    3aaf5 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
   3aaf2:	xor    %r14d,%r14d
   3aaf5:	lea    0x18(%rdi),%r12
   3aaf9:	mov    %r14,0x10(%rdi)
   3aafd:	movq   $0x0,0x28(%rdi)
   3ab05:	cmpq   $0xf,0x18(%rdi)
   3ab0a:	jbe    3c078 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   3ab10:	lea    0x30(%rdi),%rax
   3ab14:	mov    %rax,(%rsp)
   3ab18:	xor    %eax,%eax
   3ab1a:	pxor   %xmm0,%xmm0
   3ab1e:	movabs $0xffffffff00000000,%rcx
   3ab28:	nopl   0x0(%rax,%rax,1)
   3ab30:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   3ab39:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3ab42:	mov    %rcx,0x2d08(%rsp,%rax,1)
   3ab4a:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   3ab53:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3ab5c:	mov    %rcx,0x2d30(%rsp,%rax,1)
   3ab64:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3ab6d:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   3ab76:	mov    %rcx,0x2d58(%rsp,%rax,1)
   3ab7e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   3ab87:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   3ab90:	mov    %rcx,0x2d80(%rsp,%rax,1)
   3ab98:	add    $0xa0,%rax
   3ab9e:	cmp    $0x2800,%rax
   3aba4:	jne    3ab30 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   3aba6:	mov    %r12,%r14
   3aba9:	lea    0xc(%r15),%rax
   3abad:	xor    %edx,%edx
   3abaf:	mov    $0x25,%ecx
   3abb4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3abc0:	mov    -0xc(%rax),%esi
   3abc3:	add    %edx,%esi
   3abc5:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   3abcc:	mov    %esi,0x2c70(%rsp,%rcx,4)
   3abd3:	mov    -0x8(%rax),%edx
   3abd6:	add    %esi,%edx
   3abd8:	mov    %esi,0x2c94(%rsp,%rcx,4)
   3abdf:	mov    %edx,0x2c98(%rsp,%rcx,4)
   3abe6:	mov    -0x4(%rax),%esi
   3abe9:	add    %edx,%esi
   3abeb:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   3abf2:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   3abf9:	mov    (%rax),%edx
   3abfb:	add    %esi,%edx
   3abfd:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3ac04:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3ac0b:	add    $0x28,%rcx
   3ac0f:	add    $0x10,%rax
   3ac13:	cmp    $0xa25,%rcx
   3ac1a:	jne    3abc0 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
   3ac1c:	mov    %r9,0x78(%rsp)
   3ac21:	mov    0x8(%r9),%rbx
   3ac25:	mov    0x8(%rdi),%rbp
   3ac29:	mov    0x10(%rdi),%r12
   3ac2d:	lea    0xd8(%rsp),%rdi
   3ac35:	lea    0x2ce8(%rsp),%rsi
   3ac3d:	mov    $0x2800,%edx
   3ac42:	call   *0xfa268(%rip)        # 134eb0 <memcpy@GLIBC_2.14>
   3ac48:	mov    %rbp,%r8
   3ac4b:	mov    %r12,%rsi
   3ac4e:	mov    %rbx,0xc0(%rsp)
   3ac56:	mov    0x60(%rsp),%rax
   3ac5b:	mov    %rax,0xc8(%rsp)
   3ac63:	mov    %r13,0xd0(%rsp)
   3ac6b:	mov    0x5520(%rsp),%rax
   3ac73:	mov    %rax,0x98(%rsp)
   3ac7b:	mov    %rbp,0xa0(%rsp)
   3ac83:	mov    %r12,0xa8(%rsp)
   3ac8b:	mov    %r14,0xb0(%rsp)
   3ac93:	movl   $0x0,0x28e0(%rsp)
   3ac9e:	movq   $0x0,0x28d8(%rsp)
   3acaa:	mov    (%rsp),%rax
   3acae:	mov    %rax,0xb8(%rsp)
   3acb6:	mov    $0xffffffffffffffff,%rcx
   3acbd:	xor    %ebp,%ebp
   3acbf:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3acc9:	movabs $0x888888888888889,%r10
   3acd3:	pxor   %xmm0,%xmm0
   3acd7:	pcmpeqd %xmm1,%xmm1
   3acdb:	xor    %r12d,%r12d
   3acde:	jmp    3ad96 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   3ace3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3acf0:	mov    0x8(%rdi),%rax
   3acf4:	mov    %r13,%rcx
   3acf7:	shl    $0x7,%rcx
   3acfb:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3ad01:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3ad07:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3ad0d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3ad13:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3ad19:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3ad1f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3ad25:	movdqa %xmm0,(%rax,%rcx,1)
   3ad2a:	mov    %r13,%rax
   3ad2d:	inc    %rax
   3ad30:	mov    %rax,0x10(%rdi)
   3ad34:	mov    %r14,%rax
   3ad37:	shl    $0x6,%rax
   3ad3b:	or     $0x10000000,%r13d
   3ad42:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   3ad48:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3ad4e:	movdqa %xmm1,0x10(%rbx,%rax,1)
   3ad54:	movdqa %xmm1,(%rbx,%rax,1)
   3ad59:	mov    %r13d,0x3c(%rbx,%rax,1)
   3ad5e:	mov    %r14,%rdi
   3ad61:	shr    $0x6,%rdi
   3ad65:	cmp    %rsi,%rdi
   3ad68:	jae    3c10a <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   3ad6e:	mov    $0x1,%eax
   3ad73:	mov    %r14d,%ecx
   3ad76:	shl    %cl,%rax
   3ad79:	or     %rax,(%r8,%rdi,8)
   3ad7d:	mov    %r14,%rcx
   3ad80:	mov    0x8(%rsp),%r13
   3ad85:	add    $0x4,%r12
   3ad89:	cmp    $0x400,%r12
   3ad90:	je     3ae30 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
   3ad96:	mov    %rbp,%rax
   3ad99:	mov    (%r15,%r12,1),%ebp
   3ad9d:	add    %rax,%rbp
   3ada0:	cmp    %r13,%rbp
   3ada3:	setae  %dil
   3ada7:	mov    %rbp,%rax
   3adaa:	movabs $0x8888888888888889,%rdx
   3adb4:	mul    %rdx
   3adb7:	mov    %rdx,%r14
   3adba:	shr    $0x4,%r14
   3adbe:	mov    %rbp,%rax
   3adc1:	imul   %r9,%rax
   3adc5:	ror    $1,%rax
   3adc8:	cmp    %r10,%rax
   3adcb:	setb   %al
   3adce:	cmp    %rcx,%r14
   3add1:	sete   %dl
   3add4:	or     %dil,%dl
   3add7:	or     %al,%dl
   3add9:	jne    3ad85 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   3addb:	mov    0x5520(%rsp),%rdi
   3ade3:	mov    0x10(%rdi),%r13
   3ade7:	cmp    (%rdi),%r13
   3adea:	jne    3acf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3adf0:	mov    %rsi,(%rsp)
   3adf4:	mov    %r8,0x10(%rsp)
   3adf9:	call   91340 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3adfe:	pcmpeqd %xmm1,%xmm1
   3ae02:	pxor   %xmm0,%xmm0
   3ae06:	movabs $0x888888888888889,%r10
   3ae10:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3ae1a:	mov    0x10(%rsp),%r8
   3ae1f:	mov    (%rsp),%rsi
   3ae23:	mov    0x5520(%rsp),%rdi
   3ae2b:	jmp    3acf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3ae30:	cmpl   $0x0,0x1c(%rsp)
   3ae35:	lea    0xd8(%rsp),%r8
   3ae3d:	je     3aff4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3ae43:	mov    0xc8(%rsp),%r11
   3ae4b:	test   %r11,%r11
   3ae4e:	je     3aff4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3ae54:	mov    0x8(%rsp),%r9
   3ae59:	shl    $0x20,%r9
   3ae5d:	mov    0x1c(%rsp),%r10d
   3ae62:	mov    0x98(%rsp),%rdi
   3ae6a:	mov    0xc0(%rsp),%r13
   3ae72:	xor    %r12d,%r12d
   3ae75:	movabs $0xff51afd7ed558ccd,%rbx
   3ae7f:	pxor   %xmm0,%xmm0
   3ae83:	pcmpeqd %xmm1,%xmm1
   3ae87:	jmp    3af10 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
   3ae8c:	mov    0x8(%rdi),%rax
   3ae90:	mov    %rbx,%rcx
   3ae93:	shl    $0x7,%rcx
   3ae97:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3ae9d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3aea3:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3aea9:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3aeaf:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3aeb5:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3aebb:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3aec1:	movdqa %xmm0,(%rax,%rcx,1)
   3aec6:	mov    %rbx,%rax
   3aec9:	inc    %rax
   3aecc:	mov    %rax,0x10(%rdi)
   3aed0:	or     $0x10000000,%ebx
   3aed6:	movdqu %xmm1,0x2c(%r13)
   3aedc:	movdqa %xmm1,0x20(%r13)
   3aee2:	movdqa %xmm1,0x10(%r13)
   3aee8:	movdqa %xmm1,0x0(%r13)
   3aeee:	mov    %ebx,0x3c(%r13)
   3aef2:	or     %r15,(%r14,%rbp,8)
   3aef6:	movabs $0xff51afd7ed558ccd,%rbx
   3af00:	inc    %r12
   3af03:	add    $0x40,%r13
   3af07:	cmp    %r12,%r11
   3af0a:	je     3aff4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3af10:	mov    %r12,%rbp
   3af13:	shr    $0x6,%rbp
   3af17:	mov    0xa8(%rsp),%rsi
   3af1f:	cmp    %rsi,%rbp
   3af22:	jae    3c116 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   3af28:	mov    0xa0(%rsp),%r14
   3af30:	mov    (%r14,%rbp,8),%rax
   3af34:	mov    $0x1,%r15d
   3af3a:	mov    %r12d,%ecx
   3af3d:	shl    %cl,%r15
   3af40:	bt     %r12,%rax
   3af44:	jb     3af00 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3af46:	mov    %r12,%rax
   3af49:	xor    %r9,%rax
   3af4c:	mov    %rax,%rcx
   3af4f:	shr    $0x21,%rcx
   3af53:	xor    %rax,%rcx
   3af56:	imul   %rbx,%rcx
   3af5a:	mov    %rcx,%rax
   3af5d:	shr    $0x21,%rax
   3af61:	xor    %rcx,%rax
   3af64:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3af6e:	imul   %rcx,%rax
   3af72:	mov    %rax,%rcx
   3af75:	shr    $0x21,%rcx
   3af79:	xor    %rax,%rcx
   3af7c:	mov    %rcx,%rax
   3af7f:	shr    $0x3,%rax
   3af83:	movabs $0x20c49ba5e353f7cf,%rdx
   3af8d:	mul    %rdx
   3af90:	shr    $0x4,%rdx
   3af94:	imul   $0x3e8,%rdx,%rax
   3af9b:	sub    %rax,%rcx
   3af9e:	cmp    %r10,%rcx
   3afa1:	jae    3af00 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3afa7:	mov    0x10(%rdi),%rbx
   3afab:	cmp    (%rdi),%rbx
   3afae:	jne    3ae8c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3afb4:	mov    %r11,(%rsp)
   3afb8:	mov    %r9,0x10(%rsp)
   3afbd:	mov    %r10,0x50(%rsp)
   3afc2:	mov    %rdi,0x48(%rsp)
   3afc7:	call   91340 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3afcc:	pcmpeqd %xmm1,%xmm1
   3afd0:	pxor   %xmm0,%xmm0
   3afd4:	mov    0x48(%rsp),%rdi
   3afd9:	mov    0x50(%rsp),%r10
   3afde:	mov    0x10(%rsp),%r9
   3afe3:	mov    (%rsp),%r11
   3afe7:	lea    0xd8(%rsp),%r8
   3afef:	jmp    3ae8c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3aff4:	mov    0x8(%rsp),%r13
   3aff9:	test   %r13,%r13
   3affc:	mov    0x28(%rsp),%r14
   3b001:	je     3bd4f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3b007:	xor    %r11d,%r11d
   3b00a:	jmp    3b01c <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3b00c:	nopl   0x0(%rax)
   3b010:	mov    %r11,%r10
   3b013:	cmp    %r13,%r10
   3b016:	jae    3bd4f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3b01c:	mov    %r11,%rbp
   3b01f:	mov    %r9,%r12
   3b022:	mov    $0x400,%edx
   3b027:	lea    0x28e8(%rsp),%rdi
   3b02f:	xor    %esi,%esi
   3b031:	call   *0xf9e59(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   3b037:	lea    0x118(%rsp),%rax
   3b03f:	xor    %ecx,%ecx
   3b041:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b050:	mov    -0x40(%rax),%rdx
   3b054:	sub    -0x30(%rax),%rdx
   3b058:	shr    $1,%rdx
   3b05b:	add    -0x28(%rax),%edx
   3b05e:	mov    -0x18(%rax),%rsi
   3b062:	sub    -0x8(%rax),%rsi
   3b066:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3b06d:	shr    $1,%rsi
   3b070:	add    (%rax),%esi
   3b072:	mov    %esi,0x28ec(%rsp,%rcx,4)
   3b079:	add    $0x2,%rcx
   3b07d:	add    $0x50,%rax
   3b081:	cmp    $0x100,%rcx
   3b088:	jne    3b050 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
   3b08a:	mov    0xb0(%rsp),%r15
   3b092:	mov    0x10(%r15),%rbx
   3b096:	cmp    (%r15),%rbx
   3b099:	je     3bd42 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
   3b09f:	mov    %rbx,%rdi
   3b0a2:	shl    $0xa,%rdi
   3b0a6:	add    0x8(%r15),%rdi
   3b0aa:	mov    $0x400,%edx
   3b0af:	lea    0x28e8(%rsp),%rsi
   3b0b7:	call   *0xf9df3(%rip)        # 134eb0 <memcpy@GLIBC_2.14>
   3b0bd:	inc    %rbx
   3b0c0:	mov    %rbx,0x10(%r15)
   3b0c4:	mov    %rbp,%r11
   3b0c7:	lea    0xf000(%rbp),%r10
   3b0ce:	cmp    %r13,%r10
   3b0d1:	cmovae %r13,%r10
   3b0d5:	lea    0x10(%rbp),%rdi
   3b0d9:	cmp    %r10,%rdi
   3b0dc:	mov    %r10,0x20(%rsp)
   3b0e1:	jbe    3b1b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   3b0e7:	lea    0xd8(%rsp),%r8
   3b0ef:	mov    %r12,%r9
   3b0f2:	mov    0x38(%rsp),%rcx
   3b0f7:	mov    %r10,%rdx
   3b0fa:	sub    %r11,%rdx
   3b0fd:	mov    %rcx,0x38(%rsp)
   3b102:	jbe    3b010 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   3b108:	mov    %r9,%rbp
   3b10b:	mov    %r14,%rax
   3b10e:	mov    0x28e0(%rsp),%r12d
   3b116:	mov    $0x1e,%r14d
   3b11c:	sub    0x28d8(%rsp),%r14
   3b124:	cmp    %rdx,%r14
   3b127:	cmovae %rdx,%r14
   3b12b:	lea    (%rax,%r11,1),%r15
   3b12f:	test   %r14,%r14
   3b132:	je     3bbb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3b138:	xor    %r13d,%r13d
   3b13b:	nopl   0x0(%rax,%rax,1)
   3b140:	movzbl (%r15,%r13,1),%esi
   3b145:	lea    (%rsi,%rsi,4),%rcx
   3b149:	lea    (%r8,%rcx,8),%rbx
   3b14d:	mov    (%r8,%rcx,8),%rax
   3b151:	cmp    0x8(%r8,%rcx,8),%rax
   3b156:	je     3b170 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   3b158:	lea    (%r12,%r13,1),%rcx
   3b15c:	mov    %cx,(%rax)
   3b15f:	addq   $0x2,(%rbx)
   3b163:	inc    %r13
   3b166:	cmp    %r13,%r14
   3b169:	jne    3b140 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   3b16b:	jmp    3bbb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3b170:	mov    %rdx,0x10(%rsp)
   3b175:	mov    %r11,(%rsp)
   3b179:	lea    0x98(%rsp),%rdi
   3b181:	call   89290 <<c30walk::ctor2::W<false>>::refill>
   3b186:	lea    0xd8(%rsp),%r8
   3b18e:	test   %al,%al
   3b190:	je     3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3b196:	mov    (%rbx),%rax
   3b199:	mov    0x20(%rsp),%r10
   3b19e:	mov    (%rsp),%r11
   3b1a2:	mov    0x10(%rsp),%rdx
   3b1a7:	jmp    3b158 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
   3b1a9:	nopl   0x0(%rax)
   3b1b0:	xor    %edx,%edx
   3b1b2:	lea    0xd8(%rsp),%r8
   3b1ba:	mov    %r12,%r9
   3b1bd:	mov    0x38(%rsp),%rcx
   3b1c2:	jmp    3b223 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   3b1c4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b1d0:	mov    0x28d8(%rsp),%rax
   3b1d8:	lea    0x10(%rax),%rcx
   3b1dc:	add    $0xfffffffffffffff2,%rax
   3b1e0:	xor    %edx,%edx
   3b1e2:	cmp    $0x1e,%rcx
   3b1e6:	setae  %dl
   3b1e9:	cmovb  %rcx,%rax
   3b1ed:	mov    %rax,0x28d8(%rsp)
   3b1f5:	mov    0x28e0(%rsp),%eax
   3b1fc:	lea    0x10(%rax,%rdx,2),%eax
   3b200:	mov    %eax,0x28e0(%rsp)
   3b207:	mov    0x30(%rsp),%r11
   3b20c:	lea    0x10(%r11),%rsi
   3b210:	xor    %edx,%edx
   3b212:	mov    %rsi,%rdi
   3b215:	cmp    %r10,%rsi
   3b218:	mov    0x28(%rsp),%r14
   3b21d:	ja     3b0f7 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
   3b223:	test   $0x1,%dl
   3b226:	jne    3b254 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
   3b228:	lea    0x8(%r11),%rsi
   3b22c:	cmp    $0xfffffffffffffff7,%r11
   3b230:	ja     3bfd8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3b236:	cmp    %r13,%rsi
   3b239:	ja     3bfd8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3b23f:	lea    0x10(%r11),%rax
   3b243:	cmp    %r13,%rax
   3b246:	ja     3bfff <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
   3b24c:	mov    (%r14,%r11,1),%r9
   3b250:	mov    (%r14,%rsi,1),%rcx
   3b254:	mov    %r9,%rax
   3b257:	xor    %rcx,%rax
   3b25a:	mov    %r9,%rdx
   3b25d:	rol    $0x8,%rdx
   3b261:	xor    %r9,%rdx
   3b264:	or     %rax,%rdx
   3b267:	jne    3b2c0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   3b269:	mov    %rdi,%rax
   3b26c:	nopl   0x0(%rax)
   3b270:	mov    %rax,%r12
   3b273:	add    $0x10,%rax
   3b277:	cmp    %r10,%rax
   3b27a:	setbe  %r9b
   3b27e:	ja     3b320 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
   3b284:	lea    0x8(%r12),%rsi
   3b289:	cmp    $0xfffffffffffffff7,%r12
   3b28d:	ja     3befa <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3b293:	cmp    %r13,%rsi
   3b296:	ja     3befa <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3b29c:	mov    (%r14,%r12,1),%rdx
   3b2a0:	mov    0x8(%r14,%r12,1),%rsi
   3b2a5:	cmp    %rcx,%rdx
   3b2a8:	jne    3b2af <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   3b2aa:	cmp    %rcx,%rsi
   3b2ad:	je     3b270 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   3b2af:	mov    %rsi,0x38(%rsp)
   3b2b4:	mov    %rdx,0x68(%rsp)
   3b2b9:	lea    0x10(%r12),%r10
   3b2be:	jmp    3b325 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   3b2c0:	mov    %rdi,0x30(%rsp)
   3b2c5:	mov    0x28d8(%rsp),%rax
   3b2cd:	mov    $0x1e,%r12d
   3b2d3:	sub    %rax,%r12
   3b2d6:	cmp    $0x10,%r12
   3b2da:	mov    $0x10,%ebx
   3b2df:	cmovb  %r12,%rbx
   3b2e3:	mov    %rbx,%r15
   3b2e6:	add    %r11,%r15
   3b2e9:	jb     3bfea <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3b2ef:	cmp    %r13,%r15
   3b2f2:	ja     3bfea <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3b2f8:	mov    0x28e0(%rsp),%r14d
   3b300:	cmp    $0x1e,%rax
   3b304:	jne    3ba80 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
   3b30a:	cmp    %r15,0x30(%rsp)
   3b30f:	jae    3bb0a <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   3b315:	jmp    3c014 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3b31a:	nopw   0x0(%rax,%rax,1)
   3b320:	lea    0x10(%r12),%r10
   3b325:	mov    %r12,0x30(%rsp)
   3b32a:	sub    %r11,%r12
   3b32d:	movzbl %cl,%eax
   3b330:	mov    %rax,0x90(%rsp)
   3b338:	lea    (%rax,%rax,4),%rdx
   3b33c:	mov    (%r8,%rdx,8),%rax
   3b340:	mov    0x1c(%r8,%rdx,8),%esi
   3b345:	mov    %rax,%rdi
   3b348:	sub    0x10(%r8,%rdx,8),%rdi
   3b34d:	shr    $1,%rdi
   3b350:	add    0x18(%r8,%rdx,8),%edi
   3b355:	add    %r12,%rdi
   3b358:	cmp    %rsi,%rdi
   3b35b:	ja     3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3b361:	mov    %r10,0x70(%rsp)
   3b366:	mov    %r9d,0x44(%rsp)
   3b36b:	lea    (%r8,%rdx,8),%rbp
   3b36f:	movzbl %cl,%r13d
   3b373:	mov    0x8(%rbp),%rsi
   3b377:	mov    $0x88888889,%r14d
   3b37d:	lea    -0x21d04(%rip),%r15        # 19680 <c30walk::ctor2::PATTERNS>
   3b384:	mov    %rbp,0x88(%rsp)
   3b38c:	jmp    3b3c7 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   3b38e:	xchg   %ax,%ax
   3b390:	mov    0x3c(%rsi),%edi
   3b393:	mov    $0xfffff,%r9d
   3b399:	and    %r9d,%edi
   3b39c:	sub    %r8d,%ecx
   3b39f:	sub    %eax,%r8d
   3b3a2:	mov    %r8d,%eax
   3b3a5:	lea    (%rsi,%rax,2),%rax
   3b3a9:	mov    %rax,0x10(%rbp)
   3b3ad:	mov    %rax,0x0(%rbp)
   3b3b1:	lea    (%rax,%rcx,2),%rsi
   3b3b5:	mov    %rsi,0x8(%rbp)
   3b3b9:	mov    %edx,0x20(%rbp)
   3b3bc:	mov    %edi,0x24(%rbp)
   3b3bf:	lea    0xd8(%rsp),%r8
   3b3c7:	sub    %rax,%rsi
   3b3ca:	shr    $1,%rsi
   3b3cd:	cmp    %rsi,%r12
   3b3d0:	cmovb  %r12,%rsi
   3b3d4:	test   %rsi,%rsi
   3b3d7:	je     3b491 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
   3b3dd:	mov    0x28d8(%rsp),%rdi
   3b3e5:	cmp    $0x1d,%rdi
   3b3e9:	ja     3c0e8 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   3b3ef:	cmp    $0x1f,%rsi
   3b3f3:	jae    3bf0c <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
   3b3f9:	mov    0x28e0(%rsp),%ecx
   3b400:	mov    %rdi,%r11
   3b403:	shl    $0x6,%r11
   3b407:	lea    (%r15,%r11,1),%rdx
   3b40b:	lea    -0x2(,%rsi,2),%r9
   3b413:	cmp    $0xe,%r9
   3b417:	jb     3b42c <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   3b419:	mov    %rax,%r8
   3b41c:	sub    %r15,%r8
   3b41f:	sub    %r11,%r8
   3b422:	cmp    $0x10,%r8
   3b426:	jae    3b893 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
   3b42c:	xor    %r10d,%r10d
   3b42f:	mov    %rdx,%r8
   3b432:	lea    (%rdx,%rsi,2),%rdx
   3b436:	lea    (%rax,%r10,2),%r9
   3b43a:	nopw   0x0(%rax,%rax,1)
   3b440:	movzwl (%r8),%r10d
   3b444:	add    %cx,%r10w
   3b448:	add    $0x2,%r8
   3b44c:	mov    %r10w,(%r9)
   3b450:	add    $0x2,%r9
   3b454:	cmp    %rdx,%r8
   3b457:	jne    3b440 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
   3b459:	lea    (%rax,%rsi,2),%rax
   3b45d:	mov    %rax,0x0(%rbp)
   3b461:	add    %rsi,%rdi
   3b464:	lea    0x2(%rsi),%edx
   3b467:	mov    %rdi,%r8
   3b46a:	sub    $0x1e,%r8
   3b46e:	cmovb  %rdi,%r8
   3b472:	cmovb  %esi,%edx
   3b475:	mov    %r8,0x28d8(%rsp)
   3b47d:	add    %ecx,%edx
   3b47f:	mov    %edx,0x28e0(%rsp)
   3b486:	sub    %rsi,%r12
   3b489:	lea    0xd8(%rsp),%r8
   3b491:	test   %r12,%r12
   3b494:	je     3ba70 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   3b49a:	mov    %rax,%r8
   3b49d:	cmp    0x10(%rbp),%rax
   3b4a1:	je     3b4c0 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
   3b4a3:	lea    0x98(%rsp),%rdi
   3b4ab:	mov    0x90(%rsp),%rsi
   3b4b3:	call   88c90 <<c30walk::ctor2::W<false>>::finish>
   3b4b8:	mov    0x0(%rbp),%r8
   3b4bc:	mov    0x10(%rbp),%rax
   3b4c0:	sub    %rax,%r8
   3b4c3:	shr    $1,%r8
   3b4c6:	add    0x18(%rbp),%r8d
   3b4ca:	mov    %r8d,0x18(%rbp)
   3b4ce:	pxor   %xmm0,%xmm0
   3b4d2:	movdqu %xmm0,0x0(%rbp)
   3b4d7:	movq   $0x0,0x10(%rbp)
   3b4df:	cmp    $0x1e,%r12
   3b4e3:	jb     3ba04 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3b4e9:	mov    %r8,%rcx
   3b4ec:	imul   %r14,%rcx
   3b4f0:	shr    $0x24,%rcx
   3b4f4:	mov    %ecx,%eax
   3b4f6:	shl    $0x5,%eax
   3b4f9:	lea    (%rcx,%rcx,1),%edx
   3b4fc:	sub    %eax,%edx
   3b4fe:	add    %r8d,%edx
   3b501:	jne    3ba04 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3b507:	mov    %r12,%rax
   3b50a:	movabs $0x8888888888888889,%rdx
   3b514:	mul    %rdx
   3b517:	shr    $0x4,%rdx
   3b51b:	mov    %ecx,%r9d
   3b51e:	cmpl   $0x0,0x1c(%rsp)
   3b523:	mov    %r12,0x50(%rsp)
   3b528:	mov    %rdx,0x48(%rsp)
   3b52d:	je     3b850 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
   3b533:	mov    0x28e0(%rsp),%r10d
   3b53b:	mov    0xa8(%rsp),%rax
   3b543:	mov    %rax,(%rsp)
   3b547:	mov    0x98(%rsp),%r11
   3b54f:	mov    0xa0(%rsp),%rax
   3b557:	mov    %rax,0x10(%rsp)
   3b55c:	mov    0xc0(%rsp),%rax
   3b564:	mov    0x28d8(%rsp),%rbx
   3b56c:	mov    %rbx,%rdi
   3b56f:	shl    $0x6,%rdi
   3b573:	add    %r15,%rdi
   3b576:	mov    %r9,%rcx
   3b579:	shl    $0x6,%rcx
   3b57d:	lea    (%rax,%rcx,1),%r12
   3b581:	add    $0x3c,%r12
   3b585:	mov    %r10d,%r15d
   3b588:	shl    $0x8,%r15d
   3b58c:	or     %r13d,%r15d
   3b58f:	xor    %ebp,%ebp
   3b591:	jmp    3b7c1 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
   3b596:	cs nopw 0x0(%rax,%rax,1)
   3b5a0:	mov    0x8(%r11),%rcx
   3b5a4:	shl    $0x7,%eax
   3b5a7:	mov    %r15d,(%rcx,%rax,1)
   3b5ab:	movzwl 0x2(%rdi),%esi
   3b5af:	lea    (%r10,%rbp,1),%r14d
   3b5b3:	add    %r14d,%esi
   3b5b6:	shl    $0x8,%esi
   3b5b9:	or     %r13d,%esi
   3b5bc:	mov    %esi,0x4(%rcx,%rax,1)
   3b5c0:	movzwl 0x4(%rdi),%esi
   3b5c4:	add    %r14d,%esi
   3b5c7:	shl    $0x8,%esi
   3b5ca:	or     %r13d,%esi
   3b5cd:	mov    %esi,0x8(%rcx,%rax,1)
   3b5d1:	movzwl 0x6(%rdi),%esi
   3b5d5:	add    %r14d,%esi
   3b5d8:	shl    $0x8,%esi
   3b5db:	or     %r13d,%esi
   3b5de:	mov    %esi,0xc(%rcx,%rax,1)
   3b5e2:	movzwl 0x8(%rdi),%esi
   3b5e6:	add    %r14d,%esi
   3b5e9:	shl    $0x8,%esi
   3b5ec:	or     %r13d,%esi
   3b5ef:	mov    %esi,0x10(%rcx,%rax,1)
   3b5f3:	movzwl 0xa(%rdi),%esi
   3b5f7:	add    %r14d,%esi
   3b5fa:	shl    $0x8,%esi
   3b5fd:	or     %r13d,%esi
   3b600:	mov    %esi,0x14(%rcx,%rax,1)
   3b604:	movzwl 0xc(%rdi),%esi
   3b608:	add    %r14d,%esi
   3b60b:	shl    $0x8,%esi
   3b60e:	or     %r13d,%esi
   3b611:	mov    %esi,0x18(%rcx,%rax,1)
   3b615:	movzwl 0xe(%rdi),%esi
   3b619:	add    %r14d,%esi
   3b61c:	shl    $0x8,%esi
   3b61f:	or     %r13d,%esi
   3b622:	mov    %esi,0x1c(%rcx,%rax,1)
   3b626:	movzwl 0x10(%rdi),%esi
   3b62a:	add    %r14d,%esi
   3b62d:	shl    $0x8,%esi
   3b630:	or     %r13d,%esi
   3b633:	mov    %esi,0x20(%rcx,%rax,1)
   3b637:	movzwl 0x12(%rdi),%esi
   3b63b:	add    %r14d,%esi
   3b63e:	shl    $0x8,%esi
   3b641:	or     %r13d,%esi
   3b644:	mov    %esi,0x24(%rcx,%rax,1)
   3b648:	movzwl 0x14(%rdi),%esi
   3b64c:	add    %r14d,%esi
   3b64f:	shl    $0x8,%esi
   3b652:	or     %r13d,%esi
   3b655:	mov    %esi,0x28(%rcx,%rax,1)
   3b659:	movzwl 0x16(%rdi),%esi
   3b65d:	add    %r14d,%esi
   3b660:	shl    $0x8,%esi
   3b663:	or     %r13d,%esi
   3b666:	mov    %esi,0x2c(%rcx,%rax,1)
   3b66a:	movzwl 0x18(%rdi),%esi
   3b66e:	add    %r14d,%esi
   3b671:	shl    $0x8,%esi
   3b674:	or     %r13d,%esi
   3b677:	mov    %esi,0x30(%rcx,%rax,1)
   3b67b:	movzwl 0x1a(%rdi),%esi
   3b67f:	add    %r14d,%esi
   3b682:	shl    $0x8,%esi
   3b685:	or     %r13d,%esi
   3b688:	mov    %esi,0x34(%rcx,%rax,1)
   3b68c:	movzwl 0x1c(%rdi),%esi
   3b690:	add    %r14d,%esi
   3b693:	shl    $0x8,%esi
   3b696:	or     %r13d,%esi
   3b699:	mov    %esi,0x38(%rcx,%rax,1)
   3b69d:	movzwl 0x1e(%rdi),%esi
   3b6a1:	add    %r14d,%esi
   3b6a4:	shl    $0x8,%esi
   3b6a7:	or     %r13d,%esi
   3b6aa:	mov    %esi,0x3c(%rcx,%rax,1)
   3b6ae:	movzwl 0x20(%rdi),%esi
   3b6b2:	add    %r14d,%esi
   3b6b5:	shl    $0x8,%esi
   3b6b8:	or     %r13d,%esi
   3b6bb:	mov    %esi,0x40(%rcx,%rax,1)
   3b6bf:	movzwl 0x22(%rdi),%esi
   3b6c3:	add    %r14d,%esi
   3b6c6:	shl    $0x8,%esi
   3b6c9:	or     %r13d,%esi
   3b6cc:	mov    %esi,0x44(%rcx,%rax,1)
   3b6d0:	movzwl 0x24(%rdi),%esi
   3b6d4:	add    %r14d,%esi
   3b6d7:	shl    $0x8,%esi
   3b6da:	or     %r13d,%esi
   3b6dd:	mov    %esi,0x48(%rcx,%rax,1)
   3b6e1:	movzwl 0x26(%rdi),%esi
   3b6e5:	add    %r14d,%esi
   3b6e8:	shl    $0x8,%esi
   3b6eb:	or     %r13d,%esi
   3b6ee:	mov    %esi,0x4c(%rcx,%rax,1)
   3b6f2:	movzwl 0x28(%rdi),%esi
   3b6f6:	add    %r14d,%esi
   3b6f9:	shl    $0x8,%esi
   3b6fc:	or     %r13d,%esi
   3b6ff:	mov    %esi,0x50(%rcx,%rax,1)
   3b703:	movzwl 0x2a(%rdi),%esi
   3b707:	add    %r14d,%esi
   3b70a:	shl    $0x8,%esi
   3b70d:	or     %r13d,%esi
   3b710:	mov    %esi,0x54(%rcx,%rax,1)
   3b714:	movzwl 0x2c(%rdi),%esi
   3b718:	add    %r14d,%esi
   3b71b:	shl    $0x8,%esi
   3b71e:	or     %r13d,%esi
   3b721:	mov    %esi,0x58(%rcx,%rax,1)
   3b725:	movzwl 0x2e(%rdi),%esi
   3b729:	add    %r14d,%esi
   3b72c:	shl    $0x8,%esi
   3b72f:	or     %r13d,%esi
   3b732:	mov    %esi,0x5c(%rcx,%rax,1)
   3b736:	movzwl 0x30(%rdi),%esi
   3b73a:	add    %r14d,%esi
   3b73d:	shl    $0x8,%esi
   3b740:	or     %r13d,%esi
   3b743:	mov    %esi,0x60(%rcx,%rax,1)
   3b747:	movzwl 0x32(%rdi),%esi
   3b74b:	add    %r14d,%esi
   3b74e:	shl    $0x8,%esi
   3b751:	or     %r13d,%esi
   3b754:	mov    %esi,0x64(%rcx,%rax,1)
   3b758:	movzwl 0x34(%rdi),%esi
   3b75c:	add    %r14d,%esi
   3b75f:	shl    $0x8,%esi
   3b762:	or     %r13d,%esi
   3b765:	mov    %esi,0x68(%rcx,%rax,1)
   3b769:	movzwl 0x36(%rdi),%esi
   3b76d:	add    %r14d,%esi
   3b770:	shl    $0x8,%esi
   3b773:	or     %r13d,%esi
   3b776:	mov    %esi,0x6c(%rcx,%rax,1)
   3b77a:	movzwl 0x38(%rdi),%esi
   3b77e:	add    %r14d,%esi
   3b781:	shl    $0x8,%esi
   3b784:	or     %r13d,%esi
   3b787:	mov    %esi,0x70(%rcx,%rax,1)
   3b78b:	movzwl 0x3a(%rdi),%esi
   3b78f:	add    %r14d,%esi
   3b792:	shl    $0x8,%esi
   3b795:	or     %r13d,%esi
   3b798:	mov    %esi,0x74(%rcx,%rax,1)
   3b79c:	lea    (%r10,%rbp,1),%eax
   3b7a0:	add    $0x20,%eax
   3b7a3:	mov    %eax,0x28e0(%rsp)
   3b7aa:	add    $0x20,%rbp
   3b7ae:	inc    %r9
   3b7b1:	add    $0x2000,%r15d
   3b7b8:	dec    %rdx
   3b7bb:	je     3b9bc <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
   3b7c1:	mov    %r9,%rax
   3b7c4:	shr    $0x6,%rax
   3b7c8:	cmp    (%rsp),%rax
   3b7cc:	jae    3c02b <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   3b7d2:	mov    0x10(%rsp),%rcx
   3b7d7:	mov    (%rcx,%rax,8),%rax
   3b7db:	bt     %r9,%rax
   3b7df:	jae    3b810 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   3b7e1:	cmp    $0x1e,%rbx
   3b7e5:	jae    3c050 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   3b7eb:	mov    (%r12,%rbp,2),%eax
   3b7ef:	and    $0xfffff,%eax
   3b7f4:	mov    0x10(%r11),%rcx
   3b7f8:	cmp    %rax,%rcx
   3b7fb:	ja     3b5a0 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   3b801:	jmp    3c03e <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
   3b806:	cs nopw 0x0(%rax,%rax,1)
   3b810:	cmp    $0x1e,%rbx
   3b814:	jae    3c064 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
   3b81a:	movups 0x2c(%rdi),%xmm0
   3b81e:	movups %xmm0,-0x10(%r12,%rbp,2)
   3b824:	movdqa (%rdi),%xmm0
   3b828:	movdqa 0x10(%rdi),%xmm1
   3b82d:	movdqa 0x20(%rdi),%xmm2
   3b832:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   3b839:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   3b840:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   3b847:	mov    %r15d,(%r12,%rbp,2)
   3b84b:	jmp    3b79c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
   3b850:	mov    0x28d8(%rsp),%rdi
   3b858:	cmp    $0x1d,%rdi
   3b85c:	ja     3c0f9 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   3b862:	shl    $0x6,%rdi
   3b866:	add    %r15,%rdi
   3b869:	mov    0x28e0(%rsp),%eax
   3b870:	mov    0xc0(%rsp),%rcx
   3b878:	shl    $0x6,%r9
   3b87c:	mov    %eax,%r14d
   3b87f:	shl    $0x8,%r14d
   3b883:	or     %r13d,%r14d
   3b886:	cmp    $0x3c,%r12
   3b88a:	jae    3b8f5 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3b88c:	xor    %esi,%esi
   3b88e:	jmp    3b97a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
   3b893:	shr    $1,%r9
   3b896:	inc    %r9
   3b899:	mov    %r9,%r10
   3b89c:	and    $0xfffffffffffffff8,%r10
   3b8a0:	lea    (%rdx,%r10,2),%r8
   3b8a4:	movd   %ecx,%xmm0
   3b8a8:	pshuflw $0x0,%xmm0,%xmm0
   3b8ad:	add    %r15,%r11
   3b8b0:	add    $0x8,%r11
   3b8b4:	xor    %ebx,%ebx
   3b8b6:	cs nopw 0x0(%rax,%rax,1)
   3b8c0:	movq   -0x8(%r11,%rbx,2),%xmm1
   3b8c7:	movq   (%r11,%rbx,2),%xmm2
   3b8cd:	paddw  %xmm0,%xmm1
   3b8d1:	paddw  %xmm0,%xmm2
   3b8d5:	punpcklqdq %xmm2,%xmm1
   3b8d9:	movdqu %xmm1,(%rax,%rbx,2)
   3b8de:	add    $0x8,%rbx
   3b8e2:	cmp    %rbx,%r10
   3b8e5:	jne    3b8c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   3b8e7:	cmp    %r10,%r9
   3b8ea:	jne    3b432 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   3b8f0:	jmp    3b459 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
   3b8f5:	mov    %rdx,%r10
   3b8f8:	movabs $0xffffffffffffffe,%rsi
   3b902:	and    %rsi,%r10
   3b905:	lea    (%rcx,%r9,1),%r11
   3b909:	add    $0x7c,%r11
   3b90d:	xor    %esi,%esi
   3b90f:	nop
   3b910:	movdqu 0x2c(%rdi),%xmm0
   3b915:	movdqu %xmm0,-0x50(%r11)
   3b91b:	movdqa (%rdi),%xmm1
   3b91f:	movdqa 0x10(%rdi),%xmm2
   3b924:	movdqa 0x20(%rdi),%xmm3
   3b929:	movdqa %xmm3,-0x5c(%r11)
   3b92f:	movdqa %xmm2,-0x6c(%r11)
   3b935:	movdqa %xmm1,-0x7c(%r11)
   3b93b:	mov    %r14d,-0x40(%r11)
   3b93f:	add    $0x2,%rsi
   3b943:	movdqa %xmm1,-0x3c(%r11)
   3b949:	movdqa %xmm2,-0x2c(%r11)
   3b94f:	movdqa %xmm3,-0x1c(%r11)
   3b955:	movdqu %xmm0,-0x10(%r11)
   3b95b:	lea    0x2000(%r14),%ebx
   3b962:	mov    %ebx,(%r11)
   3b965:	add    $0x4000,%r14d
   3b96c:	sub    $0xffffffffffffff80,%r11
   3b970:	cmp    %rsi,%r10
   3b973:	jne    3b910 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   3b975:	test   $0x1,%dl
   3b978:	je     3b9ae <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
   3b97a:	add    %r9,%rcx
   3b97d:	shl    $0x6,%rsi
   3b981:	movups 0x2c(%rdi),%xmm0
   3b985:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3b98a:	movdqa (%rdi),%xmm0
   3b98e:	movdqa 0x10(%rdi),%xmm1
   3b993:	movdqa 0x20(%rdi),%xmm2
   3b998:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3b99e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3b9a4:	movdqa %xmm0,(%rcx,%rsi,1)
   3b9a9:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3b9ae:	mov    %edx,%ecx
   3b9b0:	shl    $0x5,%ecx
   3b9b3:	add    %ecx,%eax
   3b9b5:	mov    %eax,0x28e0(%rsp)
   3b9bc:	mov    0x48(%rsp),%rdx
   3b9c1:	mov    %rdx,%rax
   3b9c4:	shl    $0x5,%rax
   3b9c8:	lea    (%rdx,%rdx,1),%rcx
   3b9cc:	sub    %rax,%rcx
   3b9cf:	mov    0x50(%rsp),%r12
   3b9d4:	add    %rcx,%r12
   3b9d7:	lea    (%rdx,%rdx,1),%eax
   3b9da:	shl    $0x5,%edx
   3b9dd:	sub    %eax,%edx
   3b9df:	add    %edx,%r8d
   3b9e2:	mov    0x88(%rsp),%rbp
   3b9ea:	mov    %r8d,0x18(%rbp)
   3b9ee:	test   %r12,%r12
   3b9f1:	mov    $0x88888889,%r14d
   3b9f7:	lea    -0x2237e(%rip),%r15        # 19680 <c30walk::ctor2::PATTERNS>
   3b9fe:	je     3bb79 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
   3ba04:	mov    0x1c(%rbp),%ecx
   3ba07:	mov    %r8d,%edi
   3ba0a:	imul   %r14,%rdi
   3ba0e:	mov    %rdi,%rdx
   3ba11:	shr    $0x24,%rdx
   3ba15:	mov    %edx,%eax
   3ba17:	shl    $0x5,%eax
   3ba1a:	sub    %edx,%eax
   3ba1c:	sub    %edx,%eax
   3ba1e:	lea    0x1e(%rax),%esi
   3ba21:	cmp    %ecx,%esi
   3ba23:	cmovb  %esi,%ecx
   3ba26:	shr    $0x2a,%rdi
   3ba2a:	mov    0xa8(%rsp),%rsi
   3ba32:	cmp    %rsi,%rdi
   3ba35:	jae    3c0dc <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
   3ba3b:	mov    %rdx,%rsi
   3ba3e:	shl    $0x6,%rsi
   3ba42:	add    0xc0(%rsp),%rsi
   3ba4a:	mov    0xa0(%rsp),%r9
   3ba52:	mov    (%r9,%rdi,8),%rdi
   3ba56:	bt     %rdx,%rdi
   3ba5a:	jb     3b390 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   3ba60:	mov    $0xffffffff,%edi
   3ba65:	jmp    3b39c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
   3ba6a:	nopw   0x0(%rax,%rax,1)
   3ba70:	mov    0x8(%rsp),%r13
   3ba75:	jmp    3bb86 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   3ba7a:	nopw   0x0(%rax,%rax,1)
   3ba80:	add    0x28(%rsp),%r11
   3ba85:	xor    %r13d,%r13d
   3ba88:	nopl   0x0(%rax,%rax,1)
   3ba90:	movzbl (%r11,%r13,1),%esi
   3ba95:	lea    (%rsi,%rsi,4),%rcx
   3ba99:	lea    (%r8,%rcx,8),%rbp
   3ba9d:	mov    (%r8,%rcx,8),%rax
   3baa1:	cmp    0x8(%r8,%rcx,8),%rax
   3baa6:	je     3babe <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   3baa8:	lea    (%r14,%r13,1),%rcx
   3baac:	mov    %cx,(%rax)
   3baaf:	addq   $0x2,0x0(%rbp)
   3bab4:	inc    %r13
   3bab7:	cmp    %r13,%rbx
   3baba:	jne    3ba90 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3babc:	jmp    3baf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   3babe:	mov    %r11,(%rsp)
   3bac2:	lea    0x98(%rsp),%rdi
   3baca:	call   89290 <<c30walk::ctor2::W<false>>::refill>
   3bacf:	lea    0xd8(%rsp),%r8
   3bad7:	test   %al,%al
   3bad9:	je     3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3badf:	mov    0x0(%rbp),%rax
   3bae3:	mov    0x20(%rsp),%r10
   3bae8:	mov    (%rsp),%r11
   3baec:	jmp    3baa8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
   3baee:	xchg   %ax,%ax
   3baf0:	cmp    %r15,0x30(%rsp)
   3baf5:	mov    0x8(%rsp),%r13
   3bafa:	jb     3c014 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3bb00:	cmp    $0xf,%r12
   3bb04:	ja     3b1d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3bb0a:	add    0x28(%rsp),%r15
   3bb0f:	mov    %r15,%r12
   3bb12:	sub    %rbx,%r12
   3bb15:	add    $0x10,%r12
   3bb19:	add    %r14d,%ebx
   3bb1c:	add    $0x2,%ebx
   3bb1f:	nop
   3bb20:	movzbl (%r15),%esi
   3bb24:	lea    (%rsi,%rsi,4),%rcx
   3bb28:	lea    (%r8,%rcx,8),%r14
   3bb2c:	mov    (%r8,%rcx,8),%rax
   3bb30:	cmp    0x8(%r8,%rcx,8),%rax
   3bb35:	je     3bb4d <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   3bb37:	mov    %bx,(%rax)
   3bb3a:	addq   $0x2,(%r14)
   3bb3e:	inc    %r15
   3bb41:	inc    %ebx
   3bb43:	cmp    %r12,%r15
   3bb46:	jne    3bb20 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   3bb48:	jmp    3b1d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3bb4d:	lea    0x98(%rsp),%rdi
   3bb55:	call   89290 <<c30walk::ctor2::W<false>>::refill>
   3bb5a:	lea    0xd8(%rsp),%r8
   3bb62:	test   %al,%al
   3bb64:	je     3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3bb6a:	mov    (%r14),%rax
   3bb6d:	mov    0x8(%rsp),%r13
   3bb72:	mov    0x20(%rsp),%r10
   3bb77:	jmp    3bb37 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
   3bb79:	mov    0x8(%rsp),%r13
   3bb7e:	lea    0xd8(%rsp),%r8
   3bb86:	mov    0x20(%rsp),%r10
   3bb8b:	mov    0x68(%rsp),%r9
   3bb90:	mov    0x38(%rsp),%rcx
   3bb95:	mov    0x30(%rsp),%r11
   3bb9a:	mov    0x44(%rsp),%edx
   3bb9e:	mov    0x70(%rsp),%rsi
   3bba3:	jmp    3b212 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
   3bba8:	nopl   0x0(%rax,%rax,1)
   3bbb0:	lea    (%r14,%r11,1),%rax
   3bbb4:	cmp    %r10,%rax
   3bbb7:	jne    3bc01 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
   3bbb9:	cmp    $0x10,%rdx
   3bbbd:	jne    3bc77 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
   3bbc3:	mov    0x28d8(%rsp),%rax
   3bbcb:	lea    0x10(%rax),%rcx
   3bbcf:	cmp    $0x1d,%rcx
   3bbd3:	mov    0x8(%rsp),%r13
   3bbd8:	mov    0x28(%rsp),%r14
   3bbdd:	mov    %rbp,%r9
   3bbe0:	jbe    3bd1b <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
   3bbe6:	add    $0xfffffffffffffff2,%rax
   3bbea:	mov    %rax,0x28d8(%rsp)
   3bbf2:	mov    0x28e0(%rsp),%eax
   3bbf9:	add    $0x12,%eax
   3bbfc:	jmp    3bd2d <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3bc01:	add    $0x2,%r12d
   3bc05:	movzwl %r12w,%r12d
   3bc09:	nopl   0x0(%rax)
   3bc10:	movzbl (%r15,%r14,1),%esi
   3bc15:	lea    (%rsi,%rsi,4),%rcx
   3bc19:	lea    (%r8,%rcx,8),%rbx
   3bc1d:	mov    (%r8,%rcx,8),%rax
   3bc21:	cmp    0x8(%r8,%rcx,8),%rax
   3bc26:	je     3bc40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   3bc28:	lea    (%r12,%r14,1),%rcx
   3bc2c:	mov    %cx,(%rax)
   3bc2f:	addq   $0x2,(%rbx)
   3bc33:	inc    %r14
   3bc36:	cmp    %r14,%rdx
   3bc39:	jne    3bc10 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   3bc3b:	jmp    3bbb9 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
   3bc40:	mov    %rdx,0x10(%rsp)
   3bc45:	mov    %r11,%r13
   3bc48:	lea    0x98(%rsp),%rdi
   3bc50:	call   89290 <<c30walk::ctor2::W<false>>::refill>
   3bc55:	lea    0xd8(%rsp),%r8
   3bc5d:	test   %al,%al
   3bc5f:	je     3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3bc65:	mov    (%rbx),%rax
   3bc68:	mov    0x20(%rsp),%r10
   3bc6d:	mov    %r13,%r11
   3bc70:	mov    0x10(%rsp),%rdx
   3bc75:	jmp    3bc28 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
   3bc77:	mov    0x28e0(%rsp),%eax
   3bc7e:	mov    0x28d8(%rsp),%rcx
   3bc86:	inc    %r11
   3bc89:	cmp    %r11,%r10
   3bc8c:	mov    0x8(%rsp),%r13
   3bc91:	mov    0x28(%rsp),%r14
   3bc96:	mov    %rbp,%r9
   3bc99:	je     3bcf6 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   3bc9b:	mov    %rdx,%r10
   3bc9e:	and    $0xffffe,%edx
   3bca4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3bcb0:	lea    0x1(%rcx),%rsi
   3bcb4:	add    $0xffffffffffffffe3,%rcx
   3bcb8:	xor    %edi,%edi
   3bcba:	cmp    $0x1e,%rsi
   3bcbe:	setae  %dil
   3bcc2:	cmovb  %rsi,%rcx
   3bcc6:	lea    (%rax,%rdi,2),%eax
   3bcc9:	lea    0x1(%rcx),%rsi
   3bccd:	add    $0xffffffffffffffe3,%rcx
   3bcd1:	xor    %edi,%edi
   3bcd3:	cmp    $0x1e,%rsi
   3bcd7:	setae  %dil
   3bcdb:	cmovb  %rsi,%rcx
   3bcdf:	lea    (%rax,%rdi,2),%eax
   3bce2:	add    $0x2,%eax
   3bce5:	add    $0xfffffffffffffffe,%rdx
   3bce9:	jne    3bcb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   3bceb:	test   $0x1,%r10b
   3bcef:	mov    0x20(%rsp),%r10
   3bcf4:	je     3bd11 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
   3bcf6:	lea    0x1(%rcx),%rdx
   3bcfa:	add    $0xffffffffffffffe3,%rcx
   3bcfe:	xor    %esi,%esi
   3bd00:	cmp    $0x1e,%rdx
   3bd04:	setae  %sil
   3bd08:	cmovb  %rdx,%rcx
   3bd0c:	lea    (%rax,%rsi,2),%eax
   3bd0f:	inc    %eax
   3bd11:	mov    %rcx,0x28d8(%rsp)
   3bd19:	jmp    3bd2d <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3bd1b:	mov    %rcx,0x28d8(%rsp)
   3bd23:	mov    0x28e0(%rsp),%eax
   3bd2a:	add    $0x10,%eax
   3bd2d:	mov    %eax,0x28e0(%rsp)
   3bd34:	mov    %r10,%r11
   3bd37:	cmp    %r13,%r10
   3bd3a:	jb     3b01c <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3bd40:	jmp    3bd4f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3bd42:	mov    %r15,%rdi
   3bd45:	call   e6d20 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3bd4a:	jmp    3b09f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
   3bd4f:	mov    $0x84,%eax
   3bd54:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3bd60:	lea    0x50(%rax),%rcx
   3bd64:	cmp    $0x28d4,%rcx
   3bd6b:	je     3bdcb <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
   3bd6d:	mov    0x54(%rsp,%rax,1),%rdx
   3bd72:	sub    0x64(%rsp,%rax,1),%rdx
   3bd77:	shr    $1,%rdx
   3bd7a:	add    0x6c(%rsp,%rax,1),%edx
   3bd7e:	cmp    0x70(%rsp,%rax,1),%edx
   3bd82:	jne    3bda7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3bd84:	mov    0x7c(%rsp,%rax,1),%rdx
   3bd89:	sub    0x8c(%rsp,%rax,1),%rdx
   3bd91:	shr    $1,%rdx
   3bd94:	add    0x94(%rsp,%rax,1),%edx
   3bd9b:	cmp    0x98(%rsp,%rax,1),%edx
   3bda2:	mov    %rcx,%rax
   3bda5:	je     3bd60 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
   3bda7:	mov    0x5520(%rsp),%rax
   3bdaf:	movq   $0x0,0x10(%rax)
   3bdb7:	xor    %eax,%eax
   3bdb9:	add    $0x54e8,%rsp
   3bdc0:	pop    %rbx
   3bdc1:	pop    %r12
   3bdc3:	pop    %r13
   3bdc5:	pop    %r14
   3bdc7:	pop    %r15
   3bdc9:	pop    %rbp
   3bdca:	ret
   3bdcb:	xor    %r14d,%r14d
   3bdce:	lea    0x98(%rsp),%r15
   3bdd6:	mov    0x78(%rsp),%r12
   3bddb:	mov    0x8(%rsp),%r13
   3bde0:	mov    0x60(%rsp),%rbp
   3bde5:	jmp    3be1b <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   3bde7:	nopw   0x0(%rax,%rax,1)
   3bdf0:	sub    %rcx,%rax
   3bdf3:	inc    %r14
   3bdf6:	shr    $1,%rax
   3bdf9:	add    %eax,0x18(%r8)
   3bdfd:	pxor   %xmm0,%xmm0
   3be01:	movdqu %xmm0,(%r8)
   3be06:	movq   $0x0,0x10(%r8)
   3be0e:	add    $0x28,%r8
   3be12:	cmp    $0x100,%r14
   3be19:	je     3be41 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
   3be1b:	mov    (%r8),%rcx
   3be1e:	mov    %rcx,%rax
   3be21:	cmp    0x10(%r8),%rcx
   3be25:	je     3bdf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3be27:	mov    %r15,%rdi
   3be2a:	mov    %r14,%rsi
   3be2d:	mov    %r8,%rbx
   3be30:	call   88c90 <<c30walk::ctor2::W<false>>::finish>
   3be35:	mov    %rbx,%r8
   3be38:	mov    (%rbx),%rax
   3be3b:	mov    0x10(%rbx),%rcx
   3be3f:	jmp    3bdf0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3be41:	xor    %ecx,%ecx
   3be43:	mov    0x8(%r12),%rbx
   3be48:	test   %rbp,%rbp
   3be4b:	setne  %al
   3be4e:	je     3be92 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3be50:	mov    %al,%cl
   3be52:	mov    %rbp,%rax
   3be55:	sub    %rcx,%rax
   3be58:	imul   $0xffffffffffffffe2,%rax,%rcx
   3be5c:	add    %r13,%rcx
   3be5f:	cmp    $0x1e,%rcx
   3be63:	jae    3be92 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3be65:	lea    (%rbx,%rbp,4),%rcx
   3be69:	lea    (%rcx,%r13,2),%rdi
   3be6d:	add    $0xfffffffffffffffc,%rdi
   3be71:	lea    (%rax,%rax,1),%rcx
   3be75:	shl    $0x5,%rax
   3be79:	sub    %rcx,%rax
   3be7c:	sub    %r13,%rax
   3be7f:	lea    0x3c(,%rax,2),%rdx
   3be87:	mov    $0xff,%esi
   3be8c:	call   *0xf8ffe(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   3be92:	shl    $0x6,%rbp
   3be96:	pcmpeqd %xmm0,%xmm0
   3be9a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   3bea0:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   3bea6:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   3beac:	movdqa %xmm0,(%rbx,%rbp,1)
   3beb1:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   3beb9:	mov    0x58(%rsp),%rax
   3bebe:	mov    0x8(%rax),%rdx
   3bec2:	mov    0x10(%rax),%rcx
   3bec6:	mov    0x5520(%rsp),%rax
   3bece:	mov    0x8(%rax),%r8
   3bed2:	mov    0x10(%rax),%r9
   3bed6:	lea    0x98(%rsp),%rdi
   3bede:	mov    %rbx,%rsi
   3bee1:	call   9a360 <c30walk::ctor::canonicalise>
   3bee6:	mov    0x80(%rsp),%rax
   3beee:	mov    %rax,0x10(%r12)
   3bef3:	mov    $0x1,%al
   3bef5:	jmp    3bdb9 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
   3befa:	lea    0xf5b6f(%rip),%rcx        # 131a70 <__frame_dummy_init_array_entry+0x2f60>
   3bf01:	mov    %r12,%rdi
   3bf04:	mov    %r13,%rdx
   3bf07:	call   d8f00 <core::slice::index::slice_index_fail>
   3bf0c:	lea    0xf474d(%rip),%rcx        # 130660 <__frame_dummy_init_array_entry+0x1b50>
   3bf13:	mov    $0x1e,%edx
   3bf18:	xor    %edi,%edi
   3bf1a:	call   d8f00 <core::slice::index::slice_index_fail>
   3bf1f:	mov    %rdi,%r12
   3bf22:	test   %rax,%rax
   3bf25:	je     3bf37 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
   3bf27:	mov    0x8(%r9),%rdi
   3bf2b:	mov    %r9,%r14
   3bf2e:	call   *0xf8f6c(%rip)        # 134ea0 <free@GLIBC_2.2.5>
   3bf34:	mov    %r14,%r9
   3bf37:	movq   $0x0,(%r9)
   3bf3e:	movq   $0x40,0x8(%r9)
   3bf46:	movq   $0x0,0x10(%r9)
   3bf4e:	mov    $0x40,%edx
   3bf53:	mov    %r9,%rdi
   3bf56:	lea    0x1(%rbp),%rsi
   3bf5a:	mov    %r9,%r14
   3bf5d:	call   90660 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3bf62:	mov    %r14,%r9
   3bf65:	cmp    $0xffffffffffffffff,%rax
   3bf69:	mov    %r12,%rdi
   3bf6c:	je     3aa65 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   3bf72:	jmp    3bfcd <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
   3bf74:	mov    %rdi,%r12
   3bf77:	mov    %r9,%r14
   3bf7a:	test   %rax,%rax
   3bf7d:	je     3bf91 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
   3bf7f:	mov    0x5520(%rsp),%rax
   3bf87:	mov    0x8(%rax),%rdi
   3bf8b:	call   *0xf8f0f(%rip)        # 134ea0 <free@GLIBC_2.2.5>
   3bf91:	mov    0x5520(%rsp),%rdi
   3bf99:	movq   $0x0,(%rdi)
   3bfa0:	movq   $0x40,0x8(%rdi)
   3bfa8:	movq   $0x0,0x10(%rdi)
   3bfb0:	mov    $0x80,%edx
   3bfb5:	mov    %rbx,%rsi
   3bfb8:	call   90660 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3bfbd:	cmp    $0xffffffffffffffff,%rax
   3bfc1:	mov    %r14,%r9
   3bfc4:	mov    %r12,%rdi
   3bfc7:	je     3aa79 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
   3bfcd:	mov    %rax,%rdi
   3bfd0:	mov    %rdx,%rsi
   3bfd3:	call   d668b <alloc::raw_vec::handle_error>
   3bfd8:	lea    0xf5a91(%rip),%rcx        # 131a70 <__frame_dummy_init_array_entry+0x2f60>
   3bfdf:	mov    %r11,%rdi
   3bfe2:	mov    %r13,%rdx
   3bfe5:	call   d8f00 <core::slice::index::slice_index_fail>
   3bfea:	lea    0xf469f(%rip),%rcx        # 130690 <__frame_dummy_init_array_entry+0x1b80>
   3bff1:	mov    %r11,%rdi
   3bff4:	mov    %r15,%rsi
   3bff7:	mov    %r13,%rdx
   3bffa:	call   d8f00 <core::slice::index::slice_index_fail>
   3bfff:	lea    0xf5a6a(%rip),%rcx        # 131a70 <__frame_dummy_init_array_entry+0x2f60>
   3c006:	mov    %rsi,%rdi
   3c009:	mov    %rax,%rsi
   3c00c:	mov    %r13,%rdx
   3c00f:	call   d8f00 <core::slice::index::slice_index_fail>
   3c014:	lea    0xf465d(%rip),%rcx        # 130678 <__frame_dummy_init_array_entry+0x1b68>
   3c01b:	mov    %r15,%rdi
   3c01e:	mov    0x30(%rsp),%rsi
   3c023:	mov    %r13,%rdx
   3c026:	call   d8f00 <core::slice::index::slice_index_fail>
   3c02b:	lea    0xf2d1e(%rip),%rdx        # 12ed50 <__frame_dummy_init_array_entry+0x240>
   3c032:	mov    %rax,%rdi
   3c035:	mov    (%rsp),%rsi
   3c039:	call   d8123 <core::panicking::panic_bounds_check>
   3c03e:	lea    0xf46db(%rip),%rdx        # 130720 <__frame_dummy_init_array_entry+0x1c10>
   3c045:	mov    %rax,%rdi
   3c048:	mov    %rcx,%rsi
   3c04b:	call   d8123 <core::panicking::panic_bounds_check>
   3c050:	lea    0xf46b1(%rip),%rdx        # 130708 <__frame_dummy_init_array_entry+0x1bf8>
   3c057:	mov    $0x1e,%esi
   3c05c:	mov    %rbx,%rdi
   3c05f:	call   d8123 <core::panicking::panic_bounds_check>
   3c064:	lea    0xf4685(%rip),%rdx        # 1306f0 <__frame_dummy_init_array_entry+0x1be0>
   3c06b:	mov    $0x1e,%esi
   3c070:	mov    %rbx,%rdi
   3c073:	call   d8123 <core::panicking::panic_bounds_check>
   3c078:	mov    $0x10,%edx
   3c07d:	mov    $0x4,%ecx
   3c082:	mov    $0x400,%r8d
   3c088:	mov    %rdi,%r14
   3c08b:	mov    %r12,%rdi
   3c08e:	xor    %esi,%esi
   3c090:	mov    %r9,%rbx
   3c093:	call   677e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c098:	mov    %r14,%rdi
   3c09b:	mov    %rbx,%r9
   3c09e:	jmp    3ab10 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
   3c0a3:	mov    %r9,%r13
   3c0a6:	mov    $0x8,%ecx
   3c0ab:	mov    $0x8,%r8d
   3c0b1:	mov    %rdi,%r14
   3c0b4:	xor    %esi,%esi
   3c0b6:	mov    %rbx,%rdx
   3c0b9:	call   677e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c0be:	mov    %r14,%rdi
   3c0c1:	mov    0x10(%r14),%r14
   3c0c5:	mov    0x8(%rdi),%r12
   3c0c9:	lea    (%r12,%r14,8),%rdi
   3c0cd:	cmp    $0x1,%rbx
   3c0d1:	jne    3aab7 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   3c0d7:	jmp    3aad9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3c0dc:	lea    0xf2c6d(%rip),%rdx        # 12ed50 <__frame_dummy_init_array_entry+0x240>
   3c0e3:	call   d8123 <core::panicking::panic_bounds_check>
   3c0e8:	lea    0xf4559(%rip),%rdx        # 130648 <__frame_dummy_init_array_entry+0x1b38>
   3c0ef:	mov    $0x1e,%esi
   3c0f4:	call   d8123 <core::panicking::panic_bounds_check>
   3c0f9:	lea    0xf45d8(%rip),%rdx        # 1306d8 <__frame_dummy_init_array_entry+0x1bc8>
   3c100:	mov    $0x1e,%esi
   3c105:	call   d8123 <core::panicking::panic_bounds_check>
   3c10a:	lea    0xf44bf(%rip),%rdx        # 1305d0 <__frame_dummy_init_array_entry+0x1ac0>
   3c111:	call   d8123 <core::panicking::panic_bounds_check>
   3c116:	lea    0xf2c33(%rip),%rdx        # 12ed50 <__frame_dummy_init_array_entry+0x240>
   3c11d:	mov    %rbp,%rdi
   3c120:	call   d8123 <core::panicking::panic_bounds_check>
