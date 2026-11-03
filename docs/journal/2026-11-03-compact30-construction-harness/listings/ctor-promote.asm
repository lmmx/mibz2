# <c30walk::ctor::W<false>>::promote
# size 0x21e

/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/c30walk-e003a32:     file format elf64-x86-64


Disassembly of section .text:

000000000004dcb0 <<c30walk::ctor::W<false>>::promote>:
   4dcb0:	push   %rbp
   4dcb1:	push   %r15
   4dcb3:	push   %r14
   4dcb5:	push   %r13
   4dcb7:	push   %r12
   4dcb9:	push   %rbx
   4dcba:	sub    $0x18,%rsp
   4dcbe:	mov    (%rdi),%rbx
   4dcc1:	mov    0x20(%rdi),%r14
   4dcc5:	mov    %rsi,%r15
   4dcc8:	shl    $0x6,%r15
   4dccc:	mov    0x10(%rbx),%rax
   4dcd0:	cmp    (%rbx),%rax
   4dcd3:	je     4de8b <<c30walk::ctor::W<false>>::promote+0x1db>
   4dcd9:	lea    (%r14,%r15,1),%r8
   4dcdd:	mov    0x8(%rbx),%r9
   4dce1:	mov    %rax,%r10
   4dce4:	shl    $0x7,%r10
   4dce8:	pxor   %xmm0,%xmm0
   4dcec:	movdqa %xmm0,0x70(%r9,%r10,1)
   4dcf3:	movdqa %xmm0,0x60(%r9,%r10,1)
   4dcfa:	movdqa %xmm0,0x50(%r9,%r10,1)
   4dd01:	movdqa %xmm0,0x40(%r9,%r10,1)
   4dd08:	movdqa %xmm0,0x30(%r9,%r10,1)
   4dd0f:	movdqa %xmm0,0x20(%r9,%r10,1)
   4dd16:	movdqa %xmm0,0x10(%r9,%r10,1)
   4dd1d:	movdqa %xmm0,(%r9,%r10,1)
   4dd23:	mov    %rax,%r9
   4dd26:	inc    %r9
   4dd29:	mov    %r9,0x10(%rbx)
   4dd2d:	mov    %eax,%r12d
   4dd30:	mov    0x8(%rbx),%rbx
   4dd34:	shl    $0x7,%r12
   4dd38:	lea    (%rbx,%r12,1),%r9
   4dd3c:	lea    (%r9,%rdx,4),%r10
   4dd40:	lea    -0x4(,%rdx,4),%r11
   4dd48:	cmp    $0x3c,%r11
   4dd4c:	jb     4dd79 <<c30walk::ctor::W<false>>::promote+0xc9>
   4dd4e:	shl    $0x2,%rdx
   4dd52:	add    $0xfffffffffffffffc,%rdx
   4dd56:	shr    $1,%rdx
   4dd59:	add    %r14,%rdx
   4dd5c:	add    %r15,%rdx
   4dd5f:	add    $0x2,%rdx
   4dd63:	cmp    %rdx,%r9
   4dd66:	setb   %dl
   4dd69:	cmp    %r10,%r8
   4dd6c:	setb   %bpl
   4dd70:	test   %bpl,%dl
   4dd73:	je     4de05 <<c30walk::ctor::W<false>>::promote+0x155>
   4dd79:	xor    %edx,%edx
   4dd7b:	lea    (%r15,%rdx,2),%rdx
   4dd7f:	add    %rdx,%r14
   4dd82:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   4dd90:	movzwl (%r14),%edx
   4dd94:	shl    $0x8,%edx
   4dd97:	add    %ecx,%edx
   4dd99:	mov    %edx,(%r9)
   4dd9c:	add    $0x4,%r9
   4dda0:	add    $0x2,%r14
   4dda4:	cmp    %r10,%r9
   4dda7:	jne    4dd90 <<c30walk::ctor::W<false>>::promote+0xe0>
   4dda9:	mov    %eax,%ecx
   4ddab:	or     $0x10000000,%ecx
   4ddb1:	pcmpeqd %xmm0,%xmm0
   4ddb5:	movdqu %xmm0,0x2c(%r8)
   4ddbb:	movdqa %xmm0,0x20(%r8)
   4ddc1:	movdqa %xmm0,0x10(%r8)
   4ddc7:	movdqa %xmm0,(%r8)
   4ddcc:	mov    %ecx,0x3c(%r8)
   4ddd0:	mov    %rsi,%r8
   4ddd3:	shr    $0x6,%r8
   4ddd7:	mov    0x10(%rdi),%rcx
   4dddb:	cmp    %rcx,%r8
   4ddde:	jae    4debc <<c30walk::ctor::W<false>>::promote+0x20c>
   4dde4:	mov    $0x1,%edx
   4dde9:	mov    %esi,%ecx
   4ddeb:	shl    %cl,%rdx
   4ddee:	mov    0x8(%rdi),%rcx
   4ddf2:	or     %rdx,(%rcx,%r8,8)
   4ddf6:	add    $0x18,%rsp
   4ddfa:	pop    %rbx
   4ddfb:	pop    %r12
   4ddfd:	pop    %r13
   4ddff:	pop    %r14
   4de01:	pop    %r15
   4de03:	pop    %rbp
   4de04:	ret
   4de05:	shr    $0x2,%r11
   4de09:	inc    %r11
   4de0c:	mov    %r11,%rdx
   4de0f:	and    $0xfffffffffffffff8,%rdx
   4de13:	lea    (%r9,%rdx,4),%r9
   4de17:	movd   %ecx,%xmm0
   4de1b:	pshufd $0x0,%xmm0,%xmm0
   4de20:	add    %r12,%rbx
   4de23:	add    $0x10,%rbx
   4de27:	lea    (%r15,%r14,1),%r12
   4de2b:	add    $0x8,%r12
   4de2f:	xor    %r13d,%r13d
   4de32:	pxor   %xmm1,%xmm1
   4de36:	cs nopw 0x0(%rax,%rax,1)
   4de40:	movq   -0x8(%r12,%r13,2),%xmm2
   4de47:	movq   (%r12,%r13,2),%xmm3
   4de4d:	punpcklwd %xmm1,%xmm2
   4de51:	punpcklwd %xmm1,%xmm3
   4de55:	pslld  $0x8,%xmm2
   4de5a:	pslld  $0x8,%xmm3
   4de5f:	paddd  %xmm0,%xmm2
   4de63:	paddd  %xmm0,%xmm3
   4de67:	movdqu %xmm2,-0x10(%rbx,%r13,4)
   4de6e:	movdqu %xmm3,(%rbx,%r13,4)
   4de74:	add    $0x8,%r13
   4de78:	cmp    %r13,%rdx
   4de7b:	jne    4de40 <<c30walk::ctor::W<false>>::promote+0x190>
   4de7d:	cmp    %rdx,%r11
   4de80:	je     4dda9 <<c30walk::ctor::W<false>>::promote+0xf9>
   4de86:	jmp    4dd7b <<c30walk::ctor::W<false>>::promote+0xcb>
   4de8b:	mov    %rdi,0x10(%rsp)
   4de90:	mov    %rbx,%rdi
   4de93:	mov    %rsi,%r13
   4de96:	mov    %rax,%r12
   4de99:	mov    %ecx,0xc(%rsp)
   4de9d:	mov    %rdx,%rbp
   4dea0:	call   4d300 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4dea5:	mov    %rbp,%rdx
   4dea8:	mov    0xc(%rsp),%ecx
   4deac:	mov    %r12,%rax
   4deaf:	mov    %r13,%rsi
   4deb2:	mov    0x10(%rsp),%rdi
   4deb7:	jmp    4dcd9 <<c30walk::ctor::W<false>>::promote+0x29>
   4debc:	lea    0x68db5(%rip),%rdx        # b6c78 <__frame_dummy_init_array_entry+0x1328>
   4dec3:	mov    %r8,%rdi
   4dec6:	mov    %rcx,%rsi
   4dec9:	call   6292b <core::panicking::panic_bounds_check>
