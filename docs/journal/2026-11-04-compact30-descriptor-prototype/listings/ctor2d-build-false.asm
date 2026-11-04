# <c30walk::ctor2d::Ctor2d>::build::<false>
# size 0x1a0a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003a960 <<c30walk::ctor2d::Ctor2d>::build::<false>>:
   3a960:	push   %rbp
   3a961:	push   %r15
   3a963:	push   %r14
   3a965:	push   %r13
   3a967:	push   %r12
   3a969:	push   %rbx
   3a96a:	sub    $0x1000,%rsp
   3a971:	movq   $0x0,(%rsp)
   3a979:	sub    $0x1000,%rsp
   3a980:	movq   $0x0,(%rsp)
   3a988:	sub    $0x1000,%rsp
   3a98f:	movq   $0x0,(%rsp)
   3a997:	sub    $0x1000,%rsp
   3a99e:	movq   $0x0,(%rsp)
   3a9a6:	sub    $0x4e8,%rsp
   3a9ad:	mov    %r8d,0x1c(%rsp)
   3a9b2:	mov    0x4520(%rsp),%rax
   3a9ba:	movq   $0x0,0x10(%r9)
   3a9c2:	movq   $0x0,0x10(%rax)
   3a9ca:	cmp    $0xeffe2,%rdx
   3a9d1:	ja     3bfcd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3a9d7:	mov    %rcx,%rbx
   3a9da:	mov    %rsi,%r12
   3a9dd:	mov    %rdi,%rbp
   3a9e0:	mov    %rdx,%rsi
   3a9e3:	pxor   %xmm0,%xmm0
   3a9e7:	xor    %eax,%eax
   3a9e9:	pxor   %xmm2,%xmm2
   3a9ed:	pxor   %xmm1,%xmm1
   3a9f1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3aa00:	movq   (%rbx,%rax,4),%xmm3
   3aa05:	movq   0x8(%rbx,%rax,4),%xmm4
   3aa0b:	punpckldq %xmm0,%xmm3
   3aa0f:	paddq  %xmm2,%xmm3
   3aa13:	punpckldq %xmm0,%xmm4
   3aa17:	paddq  %xmm1,%xmm4
   3aa1b:	movq   0x10(%rbx,%rax,4),%xmm2
   3aa21:	movq   0x18(%rbx,%rax,4),%xmm1
   3aa27:	punpckldq %xmm0,%xmm2
   3aa2b:	paddq  %xmm3,%xmm2
   3aa2f:	punpckldq %xmm0,%xmm1
   3aa33:	paddq  %xmm4,%xmm1
   3aa37:	add    $0x8,%rax
   3aa3b:	cmp    $0x100,%rax
   3aa41:	jne    3aa00 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa0>
   3aa43:	paddq  %xmm2,%xmm1
   3aa47:	pshufd $0xee,%xmm1,%xmm0
   3aa4c:	paddq  %xmm1,%xmm0
   3aa50:	movq   %xmm0,%rax
   3aa55:	cmp    %rsi,%rax
   3aa58:	jne    3bfcd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166d>
   3aa5e:	pxor   %xmm2,%xmm2
   3aa62:	xor    %eax,%eax
   3aa64:	movdqa -0x2b2ac(%rip),%xmm3        # f7c0 <GCC_except_table976+0xae8>
   3aa6c:	pxor   %xmm1,%xmm1
   3aa70:	pxor   %xmm0,%xmm0
   3aa74:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3aa80:	movq   (%rbx,%rax,4),%xmm4
   3aa85:	movq   0x8(%rbx,%rax,4),%xmm5
   3aa8b:	pshufd $0xd4,%xmm4,%xmm4
   3aa90:	pcmpeqd %xmm2,%xmm4
   3aa94:	pandn  %xmm3,%xmm4
   3aa98:	paddq  %xmm1,%xmm4
   3aa9c:	pshufd $0xd4,%xmm5,%xmm5
   3aaa1:	pcmpeqd %xmm2,%xmm5
   3aaa5:	pandn  %xmm3,%xmm5
   3aaa9:	paddq  %xmm0,%xmm5
   3aaad:	movq   0x10(%rbx,%rax,4),%xmm0
   3aab3:	movq   0x18(%rbx,%rax,4),%xmm6
   3aab9:	pshufd $0xd4,%xmm0,%xmm1
   3aabe:	pcmpeqd %xmm2,%xmm1
   3aac2:	pandn  %xmm3,%xmm1
   3aac6:	paddq  %xmm4,%xmm1
   3aaca:	pshufd $0xd4,%xmm6,%xmm0
   3aacf:	pcmpeqd %xmm2,%xmm0
   3aad3:	pandn  %xmm3,%xmm0
   3aad7:	paddq  %xmm5,%xmm0
   3aadb:	add    $0x8,%rax
   3aadf:	cmp    $0x100,%rax
   3aae5:	jne    3aa80 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x120>
   3aae7:	mov    $0x88888889,%r8d
   3aaed:	imul   %rsi,%r8
   3aaf1:	shr    $0x24,%r8
   3aaf5:	imul   $0xeeeeeeef,%esi,%eax
   3aafb:	ror    $1,%eax
   3aafd:	cmp    $0x8888889,%eax
   3ab02:	sbb    $0xffffffffffffffff,%r8
   3ab06:	mov    %r8,%r15
   3ab09:	cmpl   $0x0,0x1c(%rsp)
   3ab0e:	jne    3ab65 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3ab10:	test   %rsi,%rsi
   3ab13:	je     3ab62 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x202>
   3ab15:	paddq  %xmm1,%xmm0
   3ab19:	pshufd $0xee,%xmm0,%xmm1
   3ab1e:	paddq  %xmm0,%xmm1
   3ab22:	movq   %xmm1,%rax
   3ab27:	lea    -0x1(%rsi),%ecx
   3ab2a:	mov    $0x88888889,%edx
   3ab2f:	imul   %rcx,%rdx
   3ab33:	shr    $0x23,%rdx
   3ab37:	and    $0xfffffffe,%edx
   3ab3a:	lea    (%rdx,%rsi,1),%ecx
   3ab3d:	dec    %ecx
   3ab3f:	shr    $1,%ecx
   3ab41:	mov    $0x80010003,%r15d
   3ab47:	imul   %rcx,%r15
   3ab4b:	shr    $0x2e,%r15
   3ab4f:	inc    %r15d
   3ab52:	imul   %rax,%r15
   3ab56:	dec    %r15
   3ab59:	cmp    %r8,%r15
   3ab5c:	cmovae %r8,%r15
   3ab60:	jmp    3ab65 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x205>
   3ab62:	xor    %r15d,%r15d
   3ab65:	mov    (%r9),%rax
   3ab68:	lea    0x1(%r8),%rcx
   3ab6c:	mov    %rcx,0x68(%rsp)
   3ab71:	cmp    %r8,%rax
   3ab74:	mov    %rsi,(%rsp)
   3ab78:	mov    %r8,0x58(%rsp)
   3ab7d:	jbe    3c182 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1822>
   3ab83:	mov    0x4520(%rsp),%rdi
   3ab8b:	mov    (%rdi),%rax
   3ab8e:	cmp    %r15,%rax
   3ab91:	jb     3c1d7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1877>
   3ab97:	movq   $0x0,0x10(%rbp)
   3ab9f:	lea    0x3f(%r8),%r15
   3aba3:	shr    $0x6,%r15
   3aba7:	mov    %r12,0x40(%rsp)
   3abac:	je     3abfd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x29d>
   3abae:	cmp    0x0(%rbp),%r15
   3abb2:	ja     3c2e3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1983>
   3abb8:	xor    %r14d,%r14d
   3abbb:	mov    0x8(%rbp),%r12
   3abbf:	lea    (%r12,%r14,8),%rdi
   3abc3:	cmp    $0x1,%r15
   3abc7:	je     3abf1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3abc9:	lea    -0x8(,%r15,8),%rdx
   3abd1:	xor    %esi,%esi
   3abd3:	mov    %r9,%r13
   3abd6:	call   *0xfb274(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3abdc:	mov    %r13,%r9
   3abdf:	lea    (%r14,%r15,1),%rax
   3abe3:	lea    (%r12,%rax,8),%rdi
   3abe7:	add    $0xfffffffffffffff8,%rdi
   3abeb:	add    %r15,%r14
   3abee:	dec    %r14
   3abf1:	movq   $0x0,(%rdi)
   3abf8:	inc    %r14
   3abfb:	jmp    3ac00 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2a0>
   3abfd:	xor    %r14d,%r14d
   3ac00:	lea    0x18(%rbp),%r15
   3ac04:	mov    %r14,0x10(%rbp)
   3ac08:	movq   $0x0,0x28(%rbp)
   3ac10:	cmpq   $0xf,0x18(%rbp)
   3ac15:	jbe    3c2be <<c30walk::ctor2d::Ctor2d>::build::<false>+0x195e>
   3ac1b:	lea    0x30(%rbp),%rax
   3ac1f:	mov    %rax,0x48(%rsp)
   3ac24:	xor    %eax,%eax
   3ac26:	pxor   %xmm0,%xmm0
   3ac2a:	movabs $0xffffffff00000000,%rcx
   3ac34:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3ac40:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3ac49:	mov    %rcx,0x2cf8(%rsp,%rax,1)
   3ac51:	movdqu %xmm0,0x2d00(%rsp,%rax,1)
   3ac5a:	mov    %rcx,0x2d10(%rsp,%rax,1)
   3ac62:	movdqu %xmm0,0x2d18(%rsp,%rax,1)
   3ac6b:	mov    %rcx,0x2d28(%rsp,%rax,1)
   3ac73:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   3ac7c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   3ac84:	add    $0x60,%rax
   3ac88:	cmp    $0x1800,%rax
   3ac8e:	jne    3ac40 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2e0>
   3ac90:	mov    %r15,0x28(%rsp)
   3ac95:	lea    0xc(%rbx),%rax
   3ac99:	xor    %edx,%edx
   3ac9b:	mov    $0x15,%ecx
   3aca0:	mov    -0xc(%rax),%esi
   3aca3:	add    %edx,%esi
   3aca5:	mov    %edx,0x2c9c(%rsp,%rcx,4)
   3acac:	mov    %esi,0x2ca0(%rsp,%rcx,4)
   3acb3:	mov    -0x8(%rax),%edx
   3acb6:	add    %esi,%edx
   3acb8:	mov    %esi,0x2cb4(%rsp,%rcx,4)
   3acbf:	mov    %edx,0x2cb8(%rsp,%rcx,4)
   3acc6:	mov    -0x4(%rax),%esi
   3acc9:	add    %edx,%esi
   3accb:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   3acd2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   3acd9:	mov    (%rax),%edx
   3acdb:	add    %esi,%edx
   3acdd:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3ace4:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3aceb:	add    $0x18,%rcx
   3acef:	add    $0x10,%rax
   3acf3:	cmp    $0x615,%rcx
   3acfa:	jne    3aca0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x340>
   3acfc:	mov    %r9,0x88(%rsp)
   3ad04:	mov    0x8(%r9),%r14
   3ad08:	mov    0x8(%rbp),%r13
   3ad0c:	mov    %rbp,0x80(%rsp)
   3ad14:	mov    0x10(%rbp),%rbp
   3ad18:	lea    0x8d8(%rsp),%rdi
   3ad20:	xor    %r12d,%r12d
   3ad23:	mov    $0x800,%edx
   3ad28:	xor    %esi,%esi
   3ad2a:	call   *0xfb120(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3ad30:	lea    0x10d8(%rsp),%rdi
   3ad38:	lea    0x2ce8(%rsp),%rsi
   3ad40:	mov    $0x1800,%edx
   3ad45:	call   *0xfb125(%rip)        # 135e70 <memcpy@GLIBC_2.14>
   3ad4b:	mov    %r14,0xc0(%rsp)
   3ad53:	mov    0x58(%rsp),%rax
   3ad58:	mov    %rax,0xc8(%rsp)
   3ad60:	mov    (%rsp),%r15
   3ad64:	mov    %r15,0xd0(%rsp)
   3ad6c:	mov    0x4520(%rsp),%rax
   3ad74:	mov    %rax,0x98(%rsp)
   3ad7c:	mov    %r13,0x10(%rsp)
   3ad81:	mov    %r13,0xa0(%rsp)
   3ad89:	mov    %rbp,0x8(%rsp)
   3ad8e:	mov    %rbp,0xa8(%rsp)
   3ad96:	mov    0x28(%rsp),%rax
   3ad9b:	mov    %rax,0xb0(%rsp)
   3ada3:	lea    0xd8(%rsp),%rdi
   3adab:	mov    $0x800,%edx
   3adb0:	xor    %esi,%esi
   3adb2:	call   *0xfb098(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3adb8:	mov    %r15,%r11
   3adbb:	movl   $0x0,0x28e0(%rsp)
   3adc6:	movq   $0x0,0x28d8(%rsp)
   3add2:	mov    0x48(%rsp),%rax
   3add7:	mov    %rax,0xb8(%rsp)
   3addf:	mov    $0xffffffffffffffff,%rcx
   3ade6:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3adf0:	movabs $0x888888888888889,%r15
   3adfa:	pxor   %xmm0,%xmm0
   3adfe:	pcmpeqd %xmm1,%xmm1
   3ae02:	xor    %r13d,%r13d
   3ae05:	jmp    3ae25 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4c5>
   3ae07:	nopw   0x0(%rax,%rax,1)
   3ae10:	add    $0x4,%r13
   3ae14:	cmp    $0x400,%r13
   3ae1b:	mov    (%rsp),%r11
   3ae1f:	je     3af4a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5ea>
   3ae25:	mov    %r12,%rax
   3ae28:	mov    (%rbx,%r13,1),%r12d
   3ae2c:	add    %rax,%r12
   3ae2f:	cmp    %r11,%r12
   3ae32:	setae  %sil
   3ae36:	mov    %r12,%rax
   3ae39:	movabs $0x8888888888888889,%rdx
   3ae43:	mul    %rdx
   3ae46:	shr    $0x4,%rdx
   3ae4a:	mov    %r12,%rax
   3ae4d:	imul   %r8,%rax
   3ae51:	ror    $1,%rax
   3ae54:	cmp    %r15,%rax
   3ae57:	setb   %al
   3ae5a:	cmp    %rcx,%rdx
   3ae5d:	sete   %dil
   3ae61:	or     %sil,%dil
   3ae64:	or     %al,%dil
   3ae67:	jne    3ae10 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3ae69:	mov    0x4520(%rsp),%rdi
   3ae71:	mov    0x10(%rdi),%rbp
   3ae75:	cmp    (%rdi),%rbp
   3ae78:	je     3af1c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x5bc>
   3ae7e:	mov    0x8(%rdi),%rax
   3ae82:	mov    %rbp,%rcx
   3ae85:	shl    $0x7,%rcx
   3ae89:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3ae8f:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3ae95:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3ae9b:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3aea1:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3aea7:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3aead:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3aeb3:	movdqa %xmm0,(%rax,%rcx,1)
   3aeb8:	mov    %rbp,%rax
   3aebb:	inc    %rax
   3aebe:	mov    %rax,0x10(%rdi)
   3aec2:	mov    %rdx,%rax
   3aec5:	shl    $0x6,%rax
   3aec9:	or     $0x10000000,%ebp
   3aecf:	movdqu %xmm1,0x2c(%r14,%rax,1)
   3aed6:	movdqa %xmm1,0x20(%r14,%rax,1)
   3aedd:	movdqa %xmm1,0x10(%r14,%rax,1)
   3aee4:	movdqa %xmm1,(%r14,%rax,1)
   3aeea:	mov    %ebp,0x3c(%r14,%rax,1)
   3aeef:	mov    %rdx,%rdi
   3aef2:	shr    $0x6,%rdi
   3aef6:	cmp    0x8(%rsp),%rdi
   3aefb:	jae    3c34a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19ea>
   3af01:	mov    $0x1,%eax
   3af06:	mov    %edx,%ecx
   3af08:	shl    %cl,%rax
   3af0b:	mov    0x10(%rsp),%rcx
   3af10:	or     %rax,(%rcx,%rdi,8)
   3af14:	mov    %rdx,%rcx
   3af17:	jmp    3ae10 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x4b0>
   3af1c:	mov    %rdx,0x28(%rsp)
   3af21:	call   91cf0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3af26:	pcmpeqd %xmm1,%xmm1
   3af2a:	pxor   %xmm0,%xmm0
   3af2e:	movabs $0xeeeeeeeeeeeeeeef,%r8
   3af38:	mov    0x4520(%rsp),%rdi
   3af40:	mov    0x28(%rsp),%rdx
   3af45:	jmp    3ae7e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x51e>
   3af4a:	cmpl   $0x0,0x1c(%rsp)
   3af4f:	je     3b102 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3af55:	mov    0xc8(%rsp),%r8
   3af5d:	test   %r8,%r8
   3af60:	je     3b102 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3af66:	mov    %r11,%r9
   3af69:	shl    $0x20,%r9
   3af6d:	mov    0x1c(%rsp),%r10d
   3af72:	mov    0x98(%rsp),%rdi
   3af7a:	mov    0xc0(%rsp),%rbp
   3af82:	xor    %r15d,%r15d
   3af85:	movabs $0xff51afd7ed558ccd,%r14
   3af8f:	pxor   %xmm0,%xmm0
   3af93:	pcmpeqd %xmm1,%xmm1
   3af97:	jmp    3b020 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6c0>
   3af9c:	mov    0x8(%rdi),%rax
   3afa0:	mov    %r14,%rcx
   3afa3:	shl    $0x7,%rcx
   3afa7:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3afad:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3afb3:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3afb9:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3afbf:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3afc5:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3afcb:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3afd1:	movdqa %xmm0,(%rax,%rcx,1)
   3afd6:	mov    %r14,%rax
   3afd9:	inc    %rax
   3afdc:	mov    %rax,0x10(%rdi)
   3afe0:	or     $0x10000000,%r14d
   3afe7:	movdqu %xmm1,0x2c(%rbp)
   3afec:	movdqa %xmm1,0x20(%rbp)
   3aff1:	movdqa %xmm1,0x10(%rbp)
   3aff6:	movdqa %xmm1,0x0(%rbp)
   3affb:	mov    %r14d,0x3c(%rbp)
   3afff:	or     %rbx,0x0(%r13,%r12,8)
   3b004:	movabs $0xff51afd7ed558ccd,%r14
   3b00e:	xchg   %ax,%ax
   3b010:	inc    %r15
   3b013:	add    $0x40,%rbp
   3b017:	cmp    %r15,%r8
   3b01a:	je     3b102 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7a2>
   3b020:	mov    %r15,%r12
   3b023:	shr    $0x6,%r12
   3b027:	mov    0xa8(%rsp),%rsi
   3b02f:	cmp    %rsi,%r12
   3b032:	jae    3c35b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19fb>
   3b038:	mov    0xa0(%rsp),%r13
   3b040:	mov    0x0(%r13,%r12,8),%rax
   3b045:	mov    $0x1,%ebx
   3b04a:	mov    %r15d,%ecx
   3b04d:	shl    %cl,%rbx
   3b050:	bt     %r15,%rax
   3b054:	jb     3b010 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3b056:	mov    %r15,%rax
   3b059:	xor    %r9,%rax
   3b05c:	mov    %rax,%rcx
   3b05f:	shr    $0x21,%rcx
   3b063:	xor    %rax,%rcx
   3b066:	imul   %r14,%rcx
   3b06a:	mov    %rcx,%rax
   3b06d:	shr    $0x21,%rax
   3b071:	xor    %rcx,%rax
   3b074:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3b07e:	imul   %rcx,%rax
   3b082:	mov    %rax,%rcx
   3b085:	shr    $0x21,%rcx
   3b089:	xor    %rax,%rcx
   3b08c:	mov    %rcx,%rax
   3b08f:	shr    $0x3,%rax
   3b093:	movabs $0x20c49ba5e353f7cf,%rdx
   3b09d:	mul    %rdx
   3b0a0:	shr    $0x4,%rdx
   3b0a4:	imul   $0x3e8,%rdx,%rax
   3b0ab:	sub    %rax,%rcx
   3b0ae:	cmp    %r10,%rcx
   3b0b1:	jae    3b010 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x6b0>
   3b0b7:	mov    0x10(%rdi),%r14
   3b0bb:	cmp    (%rdi),%r14
   3b0be:	jne    3af9c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3b0c4:	mov    %r8,0x8(%rsp)
   3b0c9:	mov    %r9,0x10(%rsp)
   3b0ce:	mov    %r10,0x28(%rsp)
   3b0d3:	mov    %rdi,0x48(%rsp)
   3b0d8:	call   91cf0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3b0dd:	pcmpeqd %xmm1,%xmm1
   3b0e1:	pxor   %xmm0,%xmm0
   3b0e5:	mov    0x48(%rsp),%rdi
   3b0ea:	mov    0x28(%rsp),%r10
   3b0ef:	mov    0x10(%rsp),%r9
   3b0f4:	mov    0x8(%rsp),%r8
   3b0f9:	mov    (%rsp),%r11
   3b0fd:	jmp    3af9c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x63c>
   3b102:	test   %r11,%r11
   3b105:	mov    0x40(%rsp),%rbp
   3b10a:	je     3bf6e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3b110:	xor    %edi,%edi
   3b112:	jmp    3b12c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3b114:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b120:	mov    %rdi,%r8
   3b123:	cmp    %r10,%r8
   3b126:	jae    3bf6e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3b12c:	mov    %rdi,%r13
   3b12f:	mov    $0x400,%edx
   3b134:	lea    0x28e8(%rsp),%rdi
   3b13c:	xor    %esi,%esi
   3b13e:	call   *0xfad0c(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3b144:	mov    $0x848,%eax
   3b149:	mov    $0xa,%ecx
   3b14e:	xchg   %ax,%ax
   3b150:	movd   0x68(%rsp,%rax,2),%xmm0
   3b156:	movd   0x50(%rsp,%rax,2),%xmm1
   3b15c:	punpckldq %xmm0,%xmm1
   3b160:	movd   0x98(%rsp,%rax,2),%xmm0
   3b169:	movd   0x80(%rsp,%rax,2),%xmm2
   3b172:	punpckldq %xmm0,%xmm2
   3b176:	punpcklqdq %xmm2,%xmm1
   3b17a:	movdqu 0x88(%rsp,%rcx,8),%xmm0
   3b183:	movdqu 0x98(%rsp,%rcx,8),%xmm2
   3b18c:	movq   0x60(%rsp,%rax,2),%xmm3
   3b192:	movq   0x48(%rsp,%rax,2),%xmm4
   3b198:	punpcklqdq %xmm3,%xmm4
   3b19c:	psubq  %xmm4,%xmm0
   3b1a0:	movq   0x90(%rsp,%rax,2),%xmm3
   3b1a9:	movq   0x78(%rsp,%rax,2),%xmm4
   3b1af:	punpcklqdq %xmm3,%xmm4
   3b1b3:	psubq  %xmm4,%xmm2
   3b1b7:	psrlq  $0x1,%xmm0
   3b1bc:	psrlq  $0x1,%xmm2
   3b1c1:	pshufd $0xe8,%xmm0,%xmm0
   3b1c6:	pshufd $0xe8,%xmm2,%xmm2
   3b1cb:	punpcklqdq %xmm2,%xmm0
   3b1cf:	paddd  %xmm1,%xmm0
   3b1d3:	movdqu %xmm0,0x28c0(%rsp,%rcx,4)
   3b1dc:	add    $0x30,%rax
   3b1e0:	add    $0x4,%rcx
   3b1e4:	cmp    $0x106,%rcx
   3b1eb:	jne    3b150 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7f0>
   3b1f1:	movups 0x2880(%rsp),%xmm0
   3b1f9:	movdqu 0x8c8(%rsp),%xmm1
   3b202:	movdqu 0x8b8(%rsp),%xmm2
   3b20b:	movq   0x28c0(%rsp),%xmm3
   3b214:	movq   0x28a8(%rsp),%xmm4
   3b21d:	punpcklqdq %xmm3,%xmm4
   3b221:	psubq  %xmm4,%xmm1
   3b225:	movq   0x2890(%rsp),%xmm3
   3b22e:	movq   0x2878(%rsp),%xmm4
   3b237:	punpcklqdq %xmm3,%xmm4
   3b23b:	psubq  %xmm4,%xmm2
   3b23f:	psrlq  $0x1,%xmm1
   3b244:	psrlq  $0x1,%xmm2
   3b249:	shufps $0x88,%xmm1,%xmm2
   3b24d:	movss  0x2898(%rsp),%xmm1
   3b256:	movlhps %xmm0,%xmm1
   3b259:	movss  0x28b0(%rsp),%xmm0
   3b262:	movss  0x28c8(%rsp),%xmm3
   3b26b:	movlhps %xmm0,%xmm3
   3b26e:	shufps $0x22,%xmm3,%xmm1
   3b272:	paddd  %xmm2,%xmm1
   3b276:	movdqu %xmm1,0x2cd8(%rsp)
   3b27f:	mov    0xb0(%rsp),%rbx
   3b287:	mov    0x10(%rbx),%r14
   3b28b:	cmp    (%rbx),%r14
   3b28e:	je     3bf61 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1601>
   3b294:	mov    %r14,%rdi
   3b297:	shl    $0xa,%rdi
   3b29b:	add    0x8(%rbx),%rdi
   3b29f:	mov    $0x400,%edx
   3b2a4:	lea    0x28e8(%rsp),%rsi
   3b2ac:	call   *0xfabbe(%rip)        # 135e70 <memcpy@GLIBC_2.14>
   3b2b2:	inc    %r14
   3b2b5:	mov    %r14,0x10(%rbx)
   3b2b9:	mov    %r13,%rdi
   3b2bc:	lea    0xf000(%r13),%r9
   3b2c3:	mov    (%rsp),%r10
   3b2c7:	cmp    %r10,%r9
   3b2ca:	cmovae %r10,%r9
   3b2ce:	lea    0x10(%r13),%r11
   3b2d2:	cmp    %r9,%r11
   3b2d5:	mov    %r9,0x60(%rsp)
   3b2da:	jbe    3b3a0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa40>
   3b2e0:	sub    %rdi,%r9
   3b2e3:	jbe    3b120 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7c0>
   3b2e9:	mov    %r12,0x30(%rsp)
   3b2ee:	mov    %r15,0x38(%rsp)
   3b2f3:	mov    0x28e0(%rsp),%r13d
   3b2fb:	mov    $0x1e,%r15d
   3b301:	sub    0x28d8(%rsp),%r15
   3b309:	cmp    %r9,%r15
   3b30c:	cmovae %r9,%r15
   3b310:	lea    (%rdi,%rbp,1),%r12
   3b314:	test   %r15,%r15
   3b317:	je     3bdd0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3b31d:	xor    %r14d,%r14d
   3b320:	movzbl (%r12,%r14,1),%ebx
   3b325:	mov    0xd8(%rsp,%rbx,8),%rax
   3b32d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3b335:	je     3b354 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9f4>
   3b337:	lea    (%r14,%r13,1),%rcx
   3b33b:	mov    %cx,(%rax)
   3b33e:	addq   $0x2,0xd8(%rsp,%rbx,8)
   3b347:	inc    %r14
   3b34a:	cmp    %r14,%r15
   3b34d:	jne    3b320 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9c0>
   3b34f:	jmp    3bdd0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1470>
   3b354:	mov    %r9,0x10(%rsp)
   3b359:	mov    %rdi,0x8(%rsp)
   3b35e:	lea    0x98(%rsp),%rdi
   3b366:	mov    %rbx,%rsi
   3b369:	call   94330 <<c30walk::ctor2d::W<false>>::refill>
   3b36e:	test   %al,%al
   3b370:	je     3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3b376:	mov    0xd8(%rsp,%rbx,8),%rax
   3b37e:	mov    (%rsp),%r10
   3b382:	mov    0x40(%rsp),%rbp
   3b387:	mov    0x8(%rsp),%rdi
   3b38c:	mov    0x10(%rsp),%r9
   3b391:	jmp    3b337 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x9d7>
   3b393:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b3a0:	xor    %edx,%edx
   3b3a2:	lea    0x10d8(%rsp),%r8
   3b3aa:	jmp    3b413 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xab3>
   3b3ac:	nopl   0x0(%rax)
   3b3b0:	mov    0x28d8(%rsp),%rax
   3b3b8:	lea    0x10(%rax),%rcx
   3b3bc:	add    $0xfffffffffffffff2,%rax
   3b3c0:	xor    %edx,%edx
   3b3c2:	cmp    $0x1e,%rcx
   3b3c6:	setae  %dl
   3b3c9:	cmovb  %rcx,%rax
   3b3cd:	mov    %rax,0x28d8(%rsp)
   3b3d5:	mov    0x28e0(%rsp),%eax
   3b3dc:	lea    0x10(%rax,%rdx,2),%eax
   3b3e0:	mov    %eax,0x28e0(%rsp)
   3b3e7:	mov    0x20(%rsp),%rax
   3b3ec:	lea    0x10(%rax),%rcx
   3b3f0:	xor    %edx,%edx
   3b3f2:	mov    %rax,%rdi
   3b3f5:	mov    %rcx,%r11
   3b3f8:	mov    0x60(%rsp),%r9
   3b3fd:	cmp    %r9,%rcx
   3b400:	mov    0x40(%rsp),%rbp
   3b405:	lea    0x10d8(%rsp),%r8
   3b40d:	ja     3b2e0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x980>
   3b413:	mov    %r12,%rcx
   3b416:	test   $0x1,%dl
   3b419:	jne    3b449 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xae9>
   3b41b:	lea    0x8(%rdi),%rsi
   3b41f:	cmp    $0xfffffffffffffff7,%rdi
   3b423:	ja     3bfe1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b429:	cmp    %r10,%rsi
   3b42c:	ja     3bfe1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b432:	lea    0x10(%rdi),%rax
   3b436:	cmp    %r10,%rax
   3b439:	ja     3c244 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18e4>
   3b43f:	mov    0x0(%rbp,%rdi,1),%r15
   3b444:	mov    0x0(%rbp,%rsi,1),%rcx
   3b449:	mov    %r15,%rax
   3b44c:	xor    %rcx,%rax
   3b44f:	mov    %r15,%rdx
   3b452:	rol    $0x8,%rdx
   3b456:	xor    %r15,%rdx
   3b459:	or     %rax,%rdx
   3b45c:	jne    3b4c0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb60>
   3b45e:	mov    %rdi,%rbx
   3b461:	mov    %r11,%rax
   3b464:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b470:	mov    %rax,%rdi
   3b473:	add    $0x10,%rax
   3b477:	cmp    %r9,%rax
   3b47a:	setbe  %r11b
   3b47e:	ja     3b520 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc0>
   3b484:	lea    0x8(%rdi),%rsi
   3b488:	cmp    $0xfffffffffffffff7,%rdi
   3b48c:	ja     3bfe1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b492:	cmp    %r10,%rsi
   3b495:	ja     3bfe1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1681>
   3b49b:	mov    0x0(%rbp,%rdi,1),%rdx
   3b4a0:	mov    0x8(%rbp,%rdi,1),%rsi
   3b4a5:	cmp    %rcx,%rdx
   3b4a8:	jne    3b4af <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb4f>
   3b4aa:	cmp    %rcx,%rsi
   3b4ad:	je     3b470 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xb10>
   3b4af:	mov    %rsi,0x30(%rsp)
   3b4b4:	mov    %rdx,0x38(%rsp)
   3b4b9:	lea    0x10(%rdi),%r9
   3b4bd:	jmp    3b524 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xbc4>
   3b4bf:	nop
   3b4c0:	mov    %r11,0x20(%rsp)
   3b4c5:	mov    0x28d8(%rsp),%rax
   3b4cd:	mov    $0x1e,%r13d
   3b4d3:	sub    %rax,%r13
   3b4d6:	cmp    $0x10,%r13
   3b4da:	mov    $0x10,%r14d
   3b4e0:	cmovb  %r13,%r14
   3b4e4:	mov    %r14,%rbx
   3b4e7:	add    %rdi,%rbx
   3b4ea:	jb     3c232 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18d2>
   3b4f0:	cmp    %r10,%rbx
   3b4f3:	ja     3c232 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18d2>
   3b4f9:	mov    0x28e0(%rsp),%r12d
   3b501:	cmp    $0x1e,%rax
   3b505:	jne    3bcc0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1360>
   3b50b:	cmp    %rbx,0x20(%rsp)
   3b510:	jb     3c259 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18f9>
   3b516:	add    %rbp,%rbx
   3b519:	jmp    3bd5a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13fa>
   3b51e:	xchg   %ax,%ax
   3b520:	lea    0x10(%rdi),%r9
   3b524:	movzbl %cl,%r12d
   3b528:	lea    (%r12,%r12,2),%rdx
   3b52c:	mov    %r12d,%eax
   3b52f:	mov    0xd8(%rsp,%rax,8),%rax
   3b537:	mov    %rax,%rsi
   3b53a:	sub    (%r8,%rdx,8),%rsi
   3b53e:	mov    %rdi,0x20(%rsp)
   3b543:	mov    %rdi,%rbp
   3b546:	shr    $1,%rsi
   3b549:	add    0x8(%r8,%rdx,8),%esi
   3b54e:	sub    %rbx,%rbp
   3b551:	add    %rbp,%rsi
   3b554:	mov    0xc(%r8,%rdx,8),%edi
   3b559:	cmp    %rdi,%rsi
   3b55c:	ja     3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3b562:	mov    %r9,0x70(%rsp)
   3b567:	mov    %r11d,0x54(%rsp)
   3b56c:	lea    (%r8,%rdx,8),%rbx
   3b570:	movzbl %cl,%r14d
   3b574:	mov    0x8d8(%rsp,%r12,8),%rsi
   3b57c:	mov    $0x88888889,%r15d
   3b582:	lea    -0x20149(%rip),%r13        # 1b440 <c30walk::ctor2d::PATTERNS>
   3b589:	mov    %r12,0x90(%rsp)
   3b591:	mov    %rbx,0x78(%rsp)
   3b596:	jmp    3b5d6 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc76>
   3b598:	nopl   0x0(%rax,%rax,1)
   3b5a0:	mov    0x3c(%rsi),%edi
   3b5a3:	mov    $0xfffff,%r9d
   3b5a9:	and    %r9d,%edi
   3b5ac:	sub    %r8d,%ecx
   3b5af:	sub    %eax,%r8d
   3b5b2:	mov    %r8d,%eax
   3b5b5:	lea    (%rsi,%rax,2),%rax
   3b5b9:	mov    %rax,0xd8(%rsp,%r12,8)
   3b5c1:	lea    (%rax,%rcx,2),%rsi
   3b5c5:	mov    %rsi,0x8d8(%rsp,%r12,8)
   3b5cd:	mov    %rax,(%rbx)
   3b5d0:	mov    %edx,0x10(%rbx)
   3b5d3:	mov    %edi,0x14(%rbx)
   3b5d6:	sub    %rax,%rsi
   3b5d9:	shr    $1,%rsi
   3b5dc:	cmp    %rsi,%rbp
   3b5df:	cmovb  %rbp,%rsi
   3b5e3:	test   %rsi,%rsi
   3b5e6:	je     3b69d <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd3d>
   3b5ec:	mov    0x28d8(%rsp),%rdi
   3b5f4:	cmp    $0x1d,%rdi
   3b5f8:	ja     3c328 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19c8>
   3b5fe:	cmp    $0x1f,%rsi
   3b602:	jae    3c16f <<c30walk::ctor2d::Ctor2d>::build::<false>+0x180f>
   3b608:	mov    0x28e0(%rsp),%ecx
   3b60f:	mov    %rdi,%r11
   3b612:	shl    $0x6,%r11
   3b616:	lea    (%r11,%r13,1),%rdx
   3b61a:	lea    -0x2(,%rsi,2),%r9
   3b622:	cmp    $0xe,%r9
   3b626:	jb     3b63b <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcdb>
   3b628:	mov    %rax,%r8
   3b62b:	sub    %r13,%r8
   3b62e:	sub    %r11,%r8
   3b631:	cmp    $0x10,%r8
   3b635:	jae    3baa3 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1143>
   3b63b:	xor    %r10d,%r10d
   3b63e:	mov    %rdx,%r8
   3b641:	lea    (%rdx,%rsi,2),%rdx
   3b645:	lea    (%rax,%r10,2),%r9
   3b649:	nopl   0x0(%rax)
   3b650:	movzwl (%r8),%r10d
   3b654:	add    %cx,%r10w
   3b658:	add    $0x2,%r8
   3b65c:	mov    %r10w,(%r9)
   3b660:	add    $0x2,%r9
   3b664:	cmp    %rdx,%r8
   3b667:	jne    3b650 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xcf0>
   3b669:	lea    (%rax,%rsi,2),%rax
   3b66d:	mov    %rax,0xd8(%rsp,%r12,8)
   3b675:	add    %rsi,%rdi
   3b678:	lea    0x2(%rsi),%edx
   3b67b:	mov    %rdi,%r8
   3b67e:	sub    $0x1e,%r8
   3b682:	cmovb  %rdi,%r8
   3b686:	cmovb  %esi,%edx
   3b689:	mov    %r8,0x28d8(%rsp)
   3b691:	add    %ecx,%edx
   3b693:	mov    %edx,0x28e0(%rsp)
   3b69a:	sub    %rsi,%rbp
   3b69d:	test   %rbp,%rbp
   3b6a0:	je     3bc90 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3b6a6:	mov    %rax,%r8
   3b6a9:	cmp    (%rbx),%rax
   3b6ac:	je     3b6c9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd69>
   3b6ae:	lea    0x98(%rsp),%rdi
   3b6b6:	mov    %r12,%rsi
   3b6b9:	call   93d20 <<c30walk::ctor2d::W<false>>::finish>
   3b6be:	mov    0xd8(%rsp,%r12,8),%r8
   3b6c6:	mov    (%rbx),%rax
   3b6c9:	sub    %rax,%r8
   3b6cc:	shr    $1,%r8
   3b6cf:	add    0x8(%rbx),%r8d
   3b6d3:	mov    %r8d,0x8(%rbx)
   3b6d7:	movq   $0x0,(%rbx)
   3b6de:	movq   $0x0,0xd8(%rsp,%r12,8)
   3b6ea:	movq   $0x0,0x8d8(%rsp,%r12,8)
   3b6f6:	cmp    $0x1e,%rbp
   3b6fa:	jb     3bc22 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3b700:	mov    %r8,%rcx
   3b703:	imul   %r15,%rcx
   3b707:	shr    $0x24,%rcx
   3b70b:	mov    %ecx,%eax
   3b70d:	shl    $0x5,%eax
   3b710:	lea    (%rcx,%rcx,1),%edx
   3b713:	sub    %eax,%edx
   3b715:	add    %r8d,%edx
   3b718:	jne    3bc22 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x12c2>
   3b71e:	mov    %rbp,%rax
   3b721:	movabs $0x8888888888888889,%rdx
   3b72b:	mul    %rdx
   3b72e:	shr    $0x4,%rdx
   3b732:	mov    %ecx,%r9d
   3b735:	cmpl   $0x0,0x1c(%rsp)
   3b73a:	mov    %rbp,0x28(%rsp)
   3b73f:	mov    %rdx,0x48(%rsp)
   3b744:	je     3ba60 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1100>
   3b74a:	mov    0x28e0(%rsp),%r10d
   3b752:	mov    0xa8(%rsp),%rax
   3b75a:	mov    %rax,0x8(%rsp)
   3b75f:	mov    0x98(%rsp),%r11
   3b767:	mov    0xa0(%rsp),%rax
   3b76f:	mov    %rax,0x10(%rsp)
   3b774:	mov    0xc0(%rsp),%rax
   3b77c:	mov    0x28d8(%rsp),%r12
   3b784:	mov    %r12,%rdi
   3b787:	shl    $0x6,%rdi
   3b78b:	add    %r13,%rdi
   3b78e:	mov    %r9,%rcx
   3b791:	shl    $0x6,%rcx
   3b795:	lea    (%rax,%rcx,1),%r15
   3b799:	add    $0x3c,%r15
   3b79d:	mov    %r10d,%ebp
   3b7a0:	shl    $0x8,%ebp
   3b7a3:	or     %r14d,%ebp
   3b7a6:	xor    %ebx,%ebx
   3b7a8:	jmp    3b9cf <<c30walk::ctor2d::Ctor2d>::build::<false>+0x106f>
   3b7ad:	nopl   (%rax)
   3b7b0:	mov    0x8(%r11),%rcx
   3b7b4:	shl    $0x7,%eax
   3b7b7:	mov    %ebp,(%rcx,%rax,1)
   3b7ba:	movzwl 0x2(%rdi),%esi
   3b7be:	lea    (%r10,%rbx,1),%r13d
   3b7c2:	add    %r13d,%esi
   3b7c5:	shl    $0x8,%esi
   3b7c8:	or     %r14d,%esi
   3b7cb:	mov    %esi,0x4(%rcx,%rax,1)
   3b7cf:	movzwl 0x4(%rdi),%esi
   3b7d3:	add    %r13d,%esi
   3b7d6:	shl    $0x8,%esi
   3b7d9:	or     %r14d,%esi
   3b7dc:	mov    %esi,0x8(%rcx,%rax,1)
   3b7e0:	movzwl 0x6(%rdi),%esi
   3b7e4:	add    %r13d,%esi
   3b7e7:	shl    $0x8,%esi
   3b7ea:	or     %r14d,%esi
   3b7ed:	mov    %esi,0xc(%rcx,%rax,1)
   3b7f1:	movzwl 0x8(%rdi),%esi
   3b7f5:	add    %r13d,%esi
   3b7f8:	shl    $0x8,%esi
   3b7fb:	or     %r14d,%esi
   3b7fe:	mov    %esi,0x10(%rcx,%rax,1)
   3b802:	movzwl 0xa(%rdi),%esi
   3b806:	add    %r13d,%esi
   3b809:	shl    $0x8,%esi
   3b80c:	or     %r14d,%esi
   3b80f:	mov    %esi,0x14(%rcx,%rax,1)
   3b813:	movzwl 0xc(%rdi),%esi
   3b817:	add    %r13d,%esi
   3b81a:	shl    $0x8,%esi
   3b81d:	or     %r14d,%esi
   3b820:	mov    %esi,0x18(%rcx,%rax,1)
   3b824:	movzwl 0xe(%rdi),%esi
   3b828:	add    %r13d,%esi
   3b82b:	shl    $0x8,%esi
   3b82e:	or     %r14d,%esi
   3b831:	mov    %esi,0x1c(%rcx,%rax,1)
   3b835:	movzwl 0x10(%rdi),%esi
   3b839:	add    %r13d,%esi
   3b83c:	shl    $0x8,%esi
   3b83f:	or     %r14d,%esi
   3b842:	mov    %esi,0x20(%rcx,%rax,1)
   3b846:	movzwl 0x12(%rdi),%esi
   3b84a:	add    %r13d,%esi
   3b84d:	shl    $0x8,%esi
   3b850:	or     %r14d,%esi
   3b853:	mov    %esi,0x24(%rcx,%rax,1)
   3b857:	movzwl 0x14(%rdi),%esi
   3b85b:	add    %r13d,%esi
   3b85e:	shl    $0x8,%esi
   3b861:	or     %r14d,%esi
   3b864:	mov    %esi,0x28(%rcx,%rax,1)
   3b868:	movzwl 0x16(%rdi),%esi
   3b86c:	add    %r13d,%esi
   3b86f:	shl    $0x8,%esi
   3b872:	or     %r14d,%esi
   3b875:	mov    %esi,0x2c(%rcx,%rax,1)
   3b879:	movzwl 0x18(%rdi),%esi
   3b87d:	add    %r13d,%esi
   3b880:	shl    $0x8,%esi
   3b883:	or     %r14d,%esi
   3b886:	mov    %esi,0x30(%rcx,%rax,1)
   3b88a:	movzwl 0x1a(%rdi),%esi
   3b88e:	add    %r13d,%esi
   3b891:	shl    $0x8,%esi
   3b894:	or     %r14d,%esi
   3b897:	mov    %esi,0x34(%rcx,%rax,1)
   3b89b:	movzwl 0x1c(%rdi),%esi
   3b89f:	add    %r13d,%esi
   3b8a2:	shl    $0x8,%esi
   3b8a5:	or     %r14d,%esi
   3b8a8:	mov    %esi,0x38(%rcx,%rax,1)
   3b8ac:	movzwl 0x1e(%rdi),%esi
   3b8b0:	add    %r13d,%esi
   3b8b3:	shl    $0x8,%esi
   3b8b6:	or     %r14d,%esi
   3b8b9:	mov    %esi,0x3c(%rcx,%rax,1)
   3b8bd:	movzwl 0x20(%rdi),%esi
   3b8c1:	add    %r13d,%esi
   3b8c4:	shl    $0x8,%esi
   3b8c7:	or     %r14d,%esi
   3b8ca:	mov    %esi,0x40(%rcx,%rax,1)
   3b8ce:	movzwl 0x22(%rdi),%esi
   3b8d2:	add    %r13d,%esi
   3b8d5:	shl    $0x8,%esi
   3b8d8:	or     %r14d,%esi
   3b8db:	mov    %esi,0x44(%rcx,%rax,1)
   3b8df:	movzwl 0x24(%rdi),%esi
   3b8e3:	add    %r13d,%esi
   3b8e6:	shl    $0x8,%esi
   3b8e9:	or     %r14d,%esi
   3b8ec:	mov    %esi,0x48(%rcx,%rax,1)
   3b8f0:	movzwl 0x26(%rdi),%esi
   3b8f4:	add    %r13d,%esi
   3b8f7:	shl    $0x8,%esi
   3b8fa:	or     %r14d,%esi
   3b8fd:	mov    %esi,0x4c(%rcx,%rax,1)
   3b901:	movzwl 0x28(%rdi),%esi
   3b905:	add    %r13d,%esi
   3b908:	shl    $0x8,%esi
   3b90b:	or     %r14d,%esi
   3b90e:	mov    %esi,0x50(%rcx,%rax,1)
   3b912:	movzwl 0x2a(%rdi),%esi
   3b916:	add    %r13d,%esi
   3b919:	shl    $0x8,%esi
   3b91c:	or     %r14d,%esi
   3b91f:	mov    %esi,0x54(%rcx,%rax,1)
   3b923:	movzwl 0x2c(%rdi),%esi
   3b927:	add    %r13d,%esi
   3b92a:	shl    $0x8,%esi
   3b92d:	or     %r14d,%esi
   3b930:	mov    %esi,0x58(%rcx,%rax,1)
   3b934:	movzwl 0x2e(%rdi),%esi
   3b938:	add    %r13d,%esi
   3b93b:	shl    $0x8,%esi
   3b93e:	or     %r14d,%esi
   3b941:	mov    %esi,0x5c(%rcx,%rax,1)
   3b945:	movzwl 0x30(%rdi),%esi
   3b949:	add    %r13d,%esi
   3b94c:	shl    $0x8,%esi
   3b94f:	or     %r14d,%esi
   3b952:	mov    %esi,0x60(%rcx,%rax,1)
   3b956:	movzwl 0x32(%rdi),%esi
   3b95a:	add    %r13d,%esi
   3b95d:	shl    $0x8,%esi
   3b960:	or     %r14d,%esi
   3b963:	mov    %esi,0x64(%rcx,%rax,1)
   3b967:	movzwl 0x34(%rdi),%esi
   3b96b:	add    %r13d,%esi
   3b96e:	shl    $0x8,%esi
   3b971:	or     %r14d,%esi
   3b974:	mov    %esi,0x68(%rcx,%rax,1)
   3b978:	movzwl 0x36(%rdi),%esi
   3b97c:	add    %r13d,%esi
   3b97f:	shl    $0x8,%esi
   3b982:	or     %r14d,%esi
   3b985:	mov    %esi,0x6c(%rcx,%rax,1)
   3b989:	movzwl 0x38(%rdi),%esi
   3b98d:	add    %r13d,%esi
   3b990:	shl    $0x8,%esi
   3b993:	or     %r14d,%esi
   3b996:	mov    %esi,0x70(%rcx,%rax,1)
   3b99a:	movzwl 0x3a(%rdi),%esi
   3b99e:	add    %r13d,%esi
   3b9a1:	shl    $0x8,%esi
   3b9a4:	or     %r14d,%esi
   3b9a7:	mov    %esi,0x74(%rcx,%rax,1)
   3b9ab:	lea    (%r10,%rbx,1),%eax
   3b9af:	add    $0x20,%eax
   3b9b2:	mov    %eax,0x28e0(%rsp)
   3b9b9:	add    $0x20,%rbx
   3b9bd:	inc    %r9
   3b9c0:	add    $0x2000,%ebp
   3b9c6:	dec    %rdx
   3b9c9:	je     3bbd9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1279>
   3b9cf:	mov    %r9,%rax
   3b9d2:	shr    $0x6,%rax
   3b9d6:	cmp    0x8(%rsp),%rax
   3b9db:	jae    3c270 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1910>
   3b9e1:	mov    0x10(%rsp),%rcx
   3b9e6:	mov    (%rcx,%rax,8),%rax
   3b9ea:	bt     %r9,%rax
   3b9ee:	jae    3ba20 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x10c0>
   3b9f0:	cmp    $0x1e,%r12
   3b9f4:	jae    3c296 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1936>
   3b9fa:	mov    (%r15,%rbx,2),%eax
   3b9fe:	and    $0xfffff,%eax
   3ba03:	mov    0x10(%r11),%rcx
   3ba07:	cmp    %rax,%rcx
   3ba0a:	ja     3b7b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xe50>
   3ba10:	jmp    3c284 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1924>
   3ba15:	data16 cs nopw 0x0(%rax,%rax,1)
   3ba20:	cmp    $0x1e,%r12
   3ba24:	jae    3c2aa <<c30walk::ctor2d::Ctor2d>::build::<false>+0x194a>
   3ba2a:	movups 0x2c(%rdi),%xmm0
   3ba2e:	movups %xmm0,-0x10(%r15,%rbx,2)
   3ba34:	movdqa (%rdi),%xmm0
   3ba38:	movdqa 0x10(%rdi),%xmm1
   3ba3d:	movdqa 0x20(%rdi),%xmm2
   3ba42:	movdqa %xmm2,-0x1c(%r15,%rbx,2)
   3ba49:	movdqa %xmm1,-0x2c(%r15,%rbx,2)
   3ba50:	movdqa %xmm0,-0x3c(%r15,%rbx,2)
   3ba57:	mov    %ebp,(%r15,%rbx,2)
   3ba5b:	jmp    3b9ab <<c30walk::ctor2d::Ctor2d>::build::<false>+0x104b>
   3ba60:	mov    0x28d8(%rsp),%rdi
   3ba68:	cmp    $0x1d,%rdi
   3ba6c:	ja     3c339 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19d9>
   3ba72:	shl    $0x6,%rdi
   3ba76:	add    %r13,%rdi
   3ba79:	mov    0x28e0(%rsp),%eax
   3ba80:	mov    0xc0(%rsp),%rcx
   3ba88:	shl    $0x6,%r9
   3ba8c:	mov    %eax,%r15d
   3ba8f:	shl    $0x8,%r15d
   3ba93:	or     %r14d,%r15d
   3ba96:	cmp    $0x3c,%rbp
   3ba9a:	jae    3bb0a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11aa>
   3ba9c:	xor    %esi,%esi
   3ba9e:	jmp    3bb97 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1237>
   3baa3:	shr    $1,%r9
   3baa6:	inc    %r9
   3baa9:	mov    %r9,%r10
   3baac:	and    $0xfffffffffffffff8,%r10
   3bab0:	lea    (%rdx,%r10,2),%r8
   3bab4:	movd   %ecx,%xmm0
   3bab8:	pshuflw $0x0,%xmm0,%xmm0
   3babd:	add    %r13,%r11
   3bac0:	add    $0x8,%r11
   3bac4:	xor    %ebx,%ebx
   3bac6:	cs nopw 0x0(%rax,%rax,1)
   3bad0:	movq   -0x8(%r11,%rbx,2),%xmm1
   3bad7:	movq   (%r11,%rbx,2),%xmm2
   3badd:	paddw  %xmm0,%xmm1
   3bae1:	paddw  %xmm0,%xmm2
   3bae5:	punpcklqdq %xmm2,%xmm1
   3bae9:	movdqu %xmm1,(%rax,%rbx,2)
   3baee:	add    $0x8,%rbx
   3baf2:	cmp    %rbx,%r10
   3baf5:	jne    3bad0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1170>
   3baf7:	cmp    %r10,%r9
   3bafa:	mov    0x78(%rsp),%rbx
   3baff:	jne    3b641 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xce1>
   3bb05:	jmp    3b669 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xd09>
   3bb0a:	mov    %rdx,%r10
   3bb0d:	movabs $0xffffffffffffffe,%rsi
   3bb17:	and    %rsi,%r10
   3bb1a:	lea    (%rcx,%r9,1),%r11
   3bb1e:	add    $0x7c,%r11
   3bb22:	xor    %esi,%esi
   3bb24:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3bb30:	movdqu 0x2c(%rdi),%xmm0
   3bb35:	movdqu %xmm0,-0x50(%r11)
   3bb3b:	movdqa (%rdi),%xmm1
   3bb3f:	movdqa 0x10(%rdi),%xmm2
   3bb44:	movaps 0x20(%rdi),%xmm3
   3bb48:	movaps %xmm3,-0x5c(%r11)
   3bb4d:	movdqa %xmm2,-0x6c(%r11)
   3bb53:	movdqa %xmm1,-0x7c(%r11)
   3bb59:	mov    %r15d,-0x40(%r11)
   3bb5d:	add    $0x2,%rsi
   3bb61:	movdqa %xmm1,-0x3c(%r11)
   3bb67:	movdqa %xmm2,-0x2c(%r11)
   3bb6d:	movaps %xmm3,-0x1c(%r11)
   3bb72:	movdqu %xmm0,-0x10(%r11)
   3bb78:	lea    0x2000(%r15),%ebx
   3bb7f:	mov    %ebx,(%r11)
   3bb82:	add    $0x4000,%r15d
   3bb89:	sub    $0xffffffffffffff80,%r11
   3bb8d:	cmp    %rsi,%r10
   3bb90:	jne    3bb30 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x11d0>
   3bb92:	test   $0x1,%dl
   3bb95:	je     3bbcb <<c30walk::ctor2d::Ctor2d>::build::<false>+0x126b>
   3bb97:	add    %r9,%rcx
   3bb9a:	shl    $0x6,%rsi
   3bb9e:	movups 0x2c(%rdi),%xmm0
   3bba2:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3bba7:	movdqa (%rdi),%xmm0
   3bbab:	movdqa 0x10(%rdi),%xmm1
   3bbb0:	movdqa 0x20(%rdi),%xmm2
   3bbb5:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3bbbb:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3bbc1:	movdqa %xmm0,(%rcx,%rsi,1)
   3bbc6:	mov    %r15d,0x3c(%rcx,%rsi,1)
   3bbcb:	mov    %edx,%ecx
   3bbcd:	shl    $0x5,%ecx
   3bbd0:	add    %ecx,%eax
   3bbd2:	mov    %eax,0x28e0(%rsp)
   3bbd9:	mov    0x48(%rsp),%rdx
   3bbde:	mov    %rdx,%rax
   3bbe1:	shl    $0x5,%rax
   3bbe5:	lea    (%rdx,%rdx,1),%rcx
   3bbe9:	sub    %rax,%rcx
   3bbec:	mov    0x28(%rsp),%rbp
   3bbf1:	add    %rcx,%rbp
   3bbf4:	lea    (%rdx,%rdx,1),%eax
   3bbf7:	shl    $0x5,%edx
   3bbfa:	sub    %eax,%edx
   3bbfc:	add    %edx,%r8d
   3bbff:	mov    0x78(%rsp),%rbx
   3bc04:	mov    %r8d,0x8(%rbx)
   3bc08:	test   %rbp,%rbp
   3bc0b:	mov    $0x88888889,%r15d
   3bc11:	lea    -0x207d8(%rip),%r13        # 1b440 <c30walk::ctor2d::PATTERNS>
   3bc18:	mov    0x90(%rsp),%r12
   3bc20:	je     3bc90 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1330>
   3bc22:	mov    0xc(%rbx),%ecx
   3bc25:	mov    %r8d,%edi
   3bc28:	imul   %r15,%rdi
   3bc2c:	mov    %rdi,%rdx
   3bc2f:	shr    $0x24,%rdx
   3bc33:	mov    %edx,%eax
   3bc35:	shl    $0x5,%eax
   3bc38:	sub    %edx,%eax
   3bc3a:	sub    %edx,%eax
   3bc3c:	lea    0x1e(%rax),%esi
   3bc3f:	cmp    %ecx,%esi
   3bc41:	cmovb  %esi,%ecx
   3bc44:	shr    $0x2a,%rdi
   3bc48:	mov    0xa8(%rsp),%rsi
   3bc50:	cmp    %rsi,%rdi
   3bc53:	jae    3c31c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x19bc>
   3bc59:	mov    %rdx,%rsi
   3bc5c:	shl    $0x6,%rsi
   3bc60:	add    0xc0(%rsp),%rsi
   3bc68:	mov    0xa0(%rsp),%r9
   3bc70:	mov    (%r9,%rdi,8),%rdi
   3bc74:	bt     %rdx,%rdi
   3bc78:	jb     3b5a0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc40>
   3bc7e:	mov    $0xffffffff,%edi
   3bc83:	jmp    3b5ac <<c30walk::ctor2d::Ctor2d>::build::<false>+0xc4c>
   3bc88:	nopl   0x0(%rax,%rax,1)
   3bc90:	mov    (%rsp),%r10
   3bc94:	mov    0x38(%rsp),%r15
   3bc99:	mov    0x30(%rsp),%r12
   3bc9e:	mov    0x20(%rsp),%rax
   3bca3:	mov    0x54(%rsp),%edx
   3bca7:	mov    0x70(%rsp),%rcx
   3bcac:	jmp    3b3f2 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa92>
   3bcb1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3bcc0:	add    %rbp,%rdi
   3bcc3:	xor    %ebp,%ebp
   3bcc5:	data16 cs nopw 0x0(%rax,%rax,1)
   3bcd0:	movzbl (%rdi,%rbp,1),%r15d
   3bcd5:	mov    0xd8(%rsp,%r15,8),%rax
   3bcdd:	cmp    0x8d8(%rsp,%r15,8),%rax
   3bce5:	je     3bd01 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13a1>
   3bce7:	lea    (%r12,%rbp,1),%rcx
   3bceb:	mov    %cx,(%rax)
   3bcee:	addq   $0x2,0xd8(%rsp,%r15,8)
   3bcf7:	inc    %rbp
   3bcfa:	cmp    %rbp,%r14
   3bcfd:	jne    3bcd0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1370>
   3bcff:	jmp    3bd40 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x13e0>
   3bd01:	mov    %rdi,0x8(%rsp)
   3bd06:	lea    0x98(%rsp),%rdi
   3bd0e:	mov    %r15,%rsi
   3bd11:	call   94330 <<c30walk::ctor2d::W<false>>::refill>
   3bd16:	test   %al,%al
   3bd18:	je     3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3bd1e:	mov    0xd8(%rsp,%r15,8),%rax
   3bd26:	mov    (%rsp),%r10
   3bd2a:	mov    0x8(%rsp),%rdi
   3bd2f:	jmp    3bce7 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1387>
   3bd31:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3bd40:	cmp    %rbx,0x20(%rsp)
   3bd45:	jb     3c259 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18f9>
   3bd4b:	cmp    $0xf,%r13
   3bd4f:	ja     3b3b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3bd55:	add    0x40(%rsp),%rbx
   3bd5a:	mov    %rbx,%r13
   3bd5d:	sub    %r14,%r13
   3bd60:	add    $0x10,%r13
   3bd64:	lea    (%r12,%r14,1),%ebp
   3bd68:	add    $0x2,%ebp
   3bd6b:	nopl   0x0(%rax,%rax,1)
   3bd70:	movzbl (%rbx),%r15d
   3bd74:	mov    0xd8(%rsp,%r15,8),%rax
   3bd7c:	cmp    0x8d8(%rsp,%r15,8),%rax
   3bd84:	je     3bda1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1441>
   3bd86:	mov    %bp,(%rax)
   3bd89:	addq   $0x2,0xd8(%rsp,%r15,8)
   3bd92:	inc    %rbx
   3bd95:	inc    %ebp
   3bd97:	cmp    %r13,%rbx
   3bd9a:	jne    3bd70 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1410>
   3bd9c:	jmp    3b3b0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0xa50>
   3bda1:	lea    0x98(%rsp),%rdi
   3bda9:	mov    %r15,%rsi
   3bdac:	call   94330 <<c30walk::ctor2d::W<false>>::refill>
   3bdb1:	test   %al,%al
   3bdb3:	je     3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3bdb9:	mov    0xd8(%rsp,%r15,8),%rax
   3bdc1:	mov    (%rsp),%r10
   3bdc5:	jmp    3bd86 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1426>
   3bdc7:	nopw   0x0(%rax,%rax,1)
   3bdd0:	lea    (%r15,%rdi,1),%rax
   3bdd4:	mov    0x60(%rsp),%r8
   3bdd9:	cmp    %r8,%rax
   3bddc:	jne    3be23 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14c3>
   3bdde:	cmp    $0x10,%r9
   3bde2:	jne    3bea4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1544>
   3bde8:	mov    0x28d8(%rsp),%rax
   3bdf0:	lea    0x10(%rax),%rcx
   3bdf4:	cmp    $0x1d,%rcx
   3bdf8:	mov    0x38(%rsp),%r15
   3bdfd:	mov    0x30(%rsp),%r12
   3be02:	jbe    3bf3a <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15da>
   3be08:	add    $0xfffffffffffffff2,%rax
   3be0c:	mov    %rax,0x28d8(%rsp)
   3be14:	mov    0x28e0(%rsp),%eax
   3be1b:	add    $0x12,%eax
   3be1e:	jmp    3bf4c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15ec>
   3be23:	add    $0x2,%r13d
   3be27:	movzwl %r13w,%r13d
   3be2b:	nopl   0x0(%rax,%rax,1)
   3be30:	movzbl (%r12,%r15,1),%ebx
   3be35:	mov    0xd8(%rsp,%rbx,8),%rax
   3be3d:	cmp    0x8d8(%rsp,%rbx,8),%rax
   3be45:	je     3be64 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1504>
   3be47:	lea    (%r15,%r13,1),%rcx
   3be4b:	mov    %cx,(%rax)
   3be4e:	addq   $0x2,0xd8(%rsp,%rbx,8)
   3be57:	inc    %r15
   3be5a:	cmp    %r15,%r9
   3be5d:	jne    3be30 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14d0>
   3be5f:	jmp    3bdde <<c30walk::ctor2d::Ctor2d>::build::<false>+0x147e>
   3be64:	mov    %r9,0x10(%rsp)
   3be69:	mov    %rdi,%r14
   3be6c:	lea    0x98(%rsp),%rdi
   3be74:	mov    %rbx,%rsi
   3be77:	call   94330 <<c30walk::ctor2d::W<false>>::refill>
   3be7c:	test   %al,%al
   3be7e:	je     3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3be84:	mov    0xd8(%rsp,%rbx,8),%rax
   3be8c:	mov    (%rsp),%r10
   3be90:	mov    0x40(%rsp),%rbp
   3be95:	mov    0x60(%rsp),%r8
   3be9a:	mov    %r14,%rdi
   3be9d:	mov    0x10(%rsp),%r9
   3bea2:	jmp    3be47 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x14e7>
   3bea4:	mov    0x28e0(%rsp),%eax
   3beab:	mov    0x28d8(%rsp),%rcx
   3beb3:	inc    %rdi
   3beb6:	cmp    %rdi,%r8
   3beb9:	mov    0x38(%rsp),%r15
   3bebe:	mov    0x30(%rsp),%r12
   3bec3:	je     3bf15 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15b5>
   3bec5:	mov    %r9d,%edx
   3bec8:	and    $0xffffe,%edx
   3bece:	xchg   %ax,%ax
   3bed0:	lea    0x1(%rcx),%rsi
   3bed4:	add    $0xffffffffffffffe3,%rcx
   3bed8:	xor    %edi,%edi
   3beda:	cmp    $0x1e,%rsi
   3bede:	setae  %dil
   3bee2:	cmovb  %rsi,%rcx
   3bee6:	lea    (%rax,%rdi,2),%eax
   3bee9:	lea    0x1(%rcx),%rsi
   3beed:	add    $0xffffffffffffffe3,%rcx
   3bef1:	xor    %edi,%edi
   3bef3:	cmp    $0x1e,%rsi
   3bef7:	setae  %dil
   3befb:	cmovb  %rsi,%rcx
   3beff:	lea    (%rax,%rdi,2),%eax
   3bf02:	add    $0x2,%eax
   3bf05:	add    $0xfffffffffffffffe,%rdx
   3bf09:	jne    3bed0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1570>
   3bf0b:	test   $0x1,%r9b
   3bf0f:	mov    (%rsp),%r10
   3bf13:	je     3bf30 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15d0>
   3bf15:	lea    0x1(%rcx),%rdx
   3bf19:	add    $0xffffffffffffffe3,%rcx
   3bf1d:	xor    %esi,%esi
   3bf1f:	cmp    $0x1e,%rdx
   3bf23:	setae  %sil
   3bf27:	cmovb  %rdx,%rcx
   3bf2b:	lea    (%rax,%rsi,2),%eax
   3bf2e:	inc    %eax
   3bf30:	mov    %rcx,0x28d8(%rsp)
   3bf38:	jmp    3bf4c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x15ec>
   3bf3a:	mov    %rcx,0x28d8(%rsp)
   3bf42:	mov    0x28e0(%rsp),%eax
   3bf49:	add    $0x10,%eax
   3bf4c:	mov    %eax,0x28e0(%rsp)
   3bf53:	mov    %r8,%rdi
   3bf56:	cmp    %r10,%r8
   3bf59:	jb     3b12c <<c30walk::ctor2d::Ctor2d>::build::<false>+0x7cc>
   3bf5f:	jmp    3bf6e <<c30walk::ctor2d::Ctor2d>::build::<false>+0x160e>
   3bf61:	mov    %rbx,%rdi
   3bf64:	call   e7d00 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3bf69:	jmp    3b294 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x934>
   3bf6e:	mov    $0x9,%eax
   3bf73:	lea    0x10fc(%rsp),%rcx
   3bf7b:	nopl   0x0(%rax,%rax,1)
   3bf80:	cmp    $0x109,%rax
   3bf86:	je     3bff0 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1690>
   3bf88:	mov    0x90(%rsp,%rax,8),%rdx
   3bf90:	sub    -0x24(%rcx),%rdx
   3bf94:	shr    $1,%rdx
   3bf97:	add    -0x1c(%rcx),%edx
   3bf9a:	cmp    -0x18(%rcx),%edx
   3bf9d:	jne    3bfbd <<c30walk::ctor2d::Ctor2d>::build::<false>+0x165d>
   3bf9f:	mov    0x98(%rsp,%rax,8),%rdx
   3bfa7:	sub    -0xc(%rcx),%rdx
   3bfab:	shr    $1,%rdx
   3bfae:	add    -0x4(%rcx),%edx
   3bfb1:	add    $0x2,%rax
   3bfb5:	cmp    (%rcx),%edx
   3bfb7:	lea    0x30(%rcx),%rcx
   3bfbb:	je     3bf80 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1620>
   3bfbd:	mov    0x4520(%rsp),%rax
   3bfc5:	movq   $0x0,0x10(%rax)
   3bfcd:	xor    %eax,%eax
   3bfcf:	add    $0x44e8,%rsp
   3bfd6:	pop    %rbx
   3bfd7:	pop    %r12
   3bfd9:	pop    %r13
   3bfdb:	pop    %r14
   3bfdd:	pop    %r15
   3bfdf:	pop    %rbp
   3bfe0:	ret
   3bfe1:	lea    0xf7488(%rip),%rcx        # 133470 <__frame_dummy_init_array_entry+0x3950>
   3bfe8:	mov    %r10,%rdx
   3bfeb:	call   d9ee0 <core::slice::index::slice_index_fail>
   3bff0:	mov    $0x8,%r14d
   3bff6:	mov    $0x1040,%r15d
   3bffc:	lea    0x98(%rsp),%rbx
   3c004:	mov    0x88(%rsp),%r12
   3c00c:	mov    (%rsp),%rdx
   3c010:	mov    0x4520(%rsp),%rsi
   3c018:	mov    0x80(%rsp),%r13
   3c020:	jmp    3c072 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1712>
   3c022:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c030:	sub    %rcx,%rax
   3c033:	shr    $1,%rax
   3c036:	add    %eax,0xa0(%rsp,%r15,1)
   3c03e:	movq   $0x0,0x98(%rsp,%r15,1)
   3c04a:	movq   $0x0,0x98(%rsp,%r14,8)
   3c056:	movq   $0x0,0x898(%rsp,%r14,8)
   3c062:	inc    %r14
   3c065:	add    $0x18,%r15
   3c069:	cmp    $0x108,%r14
   3c070:	je     3c0b4 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1754>
   3c072:	mov    0x98(%rsp,%r14,8),%rcx
   3c07a:	mov    %rcx,%rax
   3c07d:	cmp    0x98(%rsp,%r15,1),%rcx
   3c085:	je     3c030 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16d0>
   3c087:	lea    -0x8(%r14),%rsi
   3c08b:	mov    %rbx,%rdi
   3c08e:	call   93d20 <<c30walk::ctor2d::W<false>>::finish>
   3c093:	mov    0x4520(%rsp),%rsi
   3c09b:	mov    (%rsp),%rdx
   3c09f:	mov    0x98(%rsp,%r14,8),%rax
   3c0a7:	mov    0x98(%rsp,%r15,1),%rcx
   3c0af:	jmp    3c030 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x16d0>
   3c0b4:	xor    %ecx,%ecx
   3c0b6:	mov    0x8(%r12),%rbx
   3c0bb:	mov    0x58(%rsp),%r14
   3c0c0:	test   %r14,%r14
   3c0c3:	setne  %al
   3c0c6:	je     3c112 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17b2>
   3c0c8:	mov    %al,%cl
   3c0ca:	mov    %r14,%rax
   3c0cd:	sub    %rcx,%rax
   3c0d0:	imul   $0xffffffffffffffe2,%rax,%rcx
   3c0d4:	add    %rdx,%rcx
   3c0d7:	cmp    $0x1e,%rcx
   3c0db:	jae    3c112 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x17b2>
   3c0dd:	lea    (%rbx,%r14,4),%rcx
   3c0e1:	lea    (%rcx,%rdx,2),%rdi
   3c0e5:	add    $0xfffffffffffffffc,%rdi
   3c0e9:	lea    (%rax,%rax,1),%rcx
   3c0ed:	shl    $0x5,%rax
   3c0f1:	sub    %rcx,%rax
   3c0f4:	sub    %rdx,%rax
   3c0f7:	lea    0x3c(,%rax,2),%rdx
   3c0ff:	mov    $0xff,%esi
   3c104:	call   *0xf9d46(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   3c10a:	mov    0x4520(%rsp),%rsi
   3c112:	shl    $0x6,%r14
   3c116:	pcmpeqd %xmm0,%xmm0
   3c11a:	movdqu %xmm0,0x2c(%rbx,%r14,1)
   3c121:	movdqa %xmm0,0x20(%rbx,%r14,1)
   3c128:	movdqa %xmm0,0x10(%rbx,%r14,1)
   3c12f:	movdqa %xmm0,(%rbx,%r14,1)
   3c135:	movl   $0x100fffff,0x3c(%rbx,%r14,1)
   3c13e:	mov    0x8(%r13),%rdx
   3c142:	mov    0x10(%r13),%rcx
   3c146:	mov    0x8(%rsi),%r8
   3c14a:	mov    0x10(%rsi),%r9
   3c14e:	lea    0x98(%rsp),%rdi
   3c156:	mov    %rbx,%rsi
   3c159:	call   9b7f0 <c30walk::ctor::canonicalise>
   3c15e:	mov    0x68(%rsp),%rax
   3c163:	mov    %rax,0x10(%r12)
   3c168:	mov    $0x1,%al
   3c16a:	jmp    3bfcf <<c30walk::ctor2d::Ctor2d>::build::<false>+0x166f>
   3c16f:	lea    0xf600a(%rip),%rcx        # 132180 <__frame_dummy_init_array_entry+0x2660>
   3c176:	mov    $0x1e,%edx
   3c17b:	xor    %edi,%edi
   3c17d:	call   d9ee0 <core::slice::index::slice_index_fail>
   3c182:	test   %rax,%rax
   3c185:	je     3c197 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1837>
   3c187:	mov    0x8(%r9),%rdi
   3c18b:	mov    %r9,%r14
   3c18e:	call   *0xf9ccc(%rip)        # 135e60 <free@GLIBC_2.2.5>
   3c194:	mov    %r14,%r9
   3c197:	movq   $0x0,(%r9)
   3c19e:	movq   $0x40,0x8(%r9)
   3c1a6:	movq   $0x0,0x10(%r9)
   3c1ae:	mov    $0x40,%edx
   3c1b3:	mov    %r9,%rdi
   3c1b6:	mov    0x68(%rsp),%rsi
   3c1bb:	mov    %r9,%r14
   3c1be:	call   90bf0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c1c3:	mov    %r14,%r9
   3c1c6:	cmp    $0xffffffffffffffff,%rax
   3c1ca:	mov    0x58(%rsp),%r8
   3c1cf:	je     3ab83 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x223>
   3c1d5:	jmp    3c227 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x18c7>
   3c1d7:	mov    %r9,%r14
   3c1da:	test   %rax,%rax
   3c1dd:	je     3c1f1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x1891>
   3c1df:	mov    0x8(%rdi),%rdi
   3c1e3:	call   *0xf9c77(%rip)        # 135e60 <free@GLIBC_2.2.5>
   3c1e9:	mov    0x4520(%rsp),%rdi
   3c1f1:	movq   $0x0,(%rdi)
   3c1f8:	movq   $0x40,0x8(%rdi)
   3c200:	movq   $0x0,0x10(%rdi)
   3c208:	mov    $0x80,%edx
   3c20d:	mov    %r15,%rsi
   3c210:	call   90bf0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c215:	cmp    $0xffffffffffffffff,%rax
   3c219:	mov    %r14,%r9
   3c21c:	mov    0x58(%rsp),%r8
   3c221:	je     3ab97 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x237>
   3c227:	mov    %rax,%rdi
   3c22a:	mov    %rdx,%rsi
   3c22d:	call   d766b <alloc::raw_vec::handle_error>
   3c232:	lea    0xf5f77(%rip),%rcx        # 1321b0 <__frame_dummy_init_array_entry+0x2690>
   3c239:	mov    %r10,%rdx
   3c23c:	mov    %rbx,%rsi
   3c23f:	call   d9ee0 <core::slice::index::slice_index_fail>
   3c244:	lea    0xf7225(%rip),%rcx        # 133470 <__frame_dummy_init_array_entry+0x3950>
   3c24b:	mov    %r10,%rdx
   3c24e:	mov    %rsi,%rdi
   3c251:	mov    %rax,%rsi
   3c254:	call   d9ee0 <core::slice::index::slice_index_fail>
   3c259:	lea    0xf5f38(%rip),%rcx        # 132198 <__frame_dummy_init_array_entry+0x2678>
   3c260:	mov    %r10,%rdx
   3c263:	mov    %rbx,%rdi
   3c266:	mov    0x20(%rsp),%rsi
   3c26b:	call   d9ee0 <core::slice::index::slice_index_fail>
   3c270:	lea    0xf3ac1(%rip),%rdx        # 12fd38 <__frame_dummy_init_array_entry+0x218>
   3c277:	mov    %rax,%rdi
   3c27a:	mov    0x8(%rsp),%rsi
   3c27f:	call   d9103 <core::panicking::panic_bounds_check>
   3c284:	lea    0xf5fb5(%rip),%rdx        # 132240 <__frame_dummy_init_array_entry+0x2720>
   3c28b:	mov    %rax,%rdi
   3c28e:	mov    %rcx,%rsi
   3c291:	call   d9103 <core::panicking::panic_bounds_check>
   3c296:	lea    0xf5f8b(%rip),%rdx        # 132228 <__frame_dummy_init_array_entry+0x2708>
   3c29d:	mov    $0x1e,%esi
   3c2a2:	mov    %r12,%rdi
   3c2a5:	call   d9103 <core::panicking::panic_bounds_check>
   3c2aa:	lea    0xf5f5f(%rip),%rdx        # 132210 <__frame_dummy_init_array_entry+0x26f0>
   3c2b1:	mov    $0x1e,%esi
   3c2b6:	mov    %r12,%rdi
   3c2b9:	call   d9103 <core::panicking::panic_bounds_check>
   3c2be:	mov    $0x10,%edx
   3c2c3:	mov    $0x4,%ecx
   3c2c8:	mov    $0x400,%r8d
   3c2ce:	mov    %r15,%rdi
   3c2d1:	xor    %esi,%esi
   3c2d3:	mov    %r9,%r14
   3c2d6:	call   68110 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c2db:	mov    %r14,%r9
   3c2de:	jmp    3ac1b <<c30walk::ctor2d::Ctor2d>::build::<false>+0x2bb>
   3c2e3:	mov    $0x8,%ecx
   3c2e8:	mov    $0x8,%r8d
   3c2ee:	mov    %rbp,%rdi
   3c2f1:	xor    %esi,%esi
   3c2f3:	mov    %r15,%rdx
   3c2f6:	mov    %r9,%r14
   3c2f9:	call   68110 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c2fe:	mov    %r14,%r9
   3c301:	mov    0x10(%rbp),%r14
   3c305:	mov    0x8(%rbp),%r12
   3c309:	lea    (%r12,%r14,8),%rdi
   3c30d:	cmp    $0x1,%r15
   3c311:	jne    3abc9 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x269>
   3c317:	jmp    3abf1 <<c30walk::ctor2d::Ctor2d>::build::<false>+0x291>
   3c31c:	lea    0xf3a15(%rip),%rdx        # 12fd38 <__frame_dummy_init_array_entry+0x218>
   3c323:	call   d9103 <core::panicking::panic_bounds_check>
   3c328:	lea    0xf5e39(%rip),%rdx        # 132168 <__frame_dummy_init_array_entry+0x2648>
   3c32f:	mov    $0x1e,%esi
   3c334:	call   d9103 <core::panicking::panic_bounds_check>
   3c339:	lea    0xf5eb8(%rip),%rdx        # 1321f8 <__frame_dummy_init_array_entry+0x26d8>
   3c340:	mov    $0x1e,%esi
   3c345:	call   d9103 <core::panicking::panic_bounds_check>
   3c34a:	lea    0xf5d9f(%rip),%rdx        # 1320f0 <__frame_dummy_init_array_entry+0x25d0>
   3c351:	mov    0x8(%rsp),%rsi
   3c356:	call   d9103 <core::panicking::panic_bounds_check>
   3c35b:	lea    0xf39d6(%rip),%rdx        # 12fd38 <__frame_dummy_init_array_entry+0x218>
   3c362:	mov    %r12,%rdi
   3c365:	call   d9103 <core::panicking::panic_bounds_check>
