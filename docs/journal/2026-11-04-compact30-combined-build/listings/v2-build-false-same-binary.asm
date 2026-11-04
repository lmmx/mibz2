# <c30walk::ctor2::Ctor2>::build::<false>
# size 0x18e5

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003cc30 <<c30walk::ctor2::Ctor2>::build::<false>>:
   3cc30:	push   %rbp
   3cc31:	push   %r15
   3cc33:	push   %r14
   3cc35:	push   %r13
   3cc37:	push   %r12
   3cc39:	push   %rbx
   3cc3a:	sub    $0x1000,%rsp
   3cc41:	movq   $0x0,(%rsp)
   3cc49:	sub    $0x1000,%rsp
   3cc50:	movq   $0x0,(%rsp)
   3cc58:	sub    $0x1000,%rsp
   3cc5f:	movq   $0x0,(%rsp)
   3cc67:	sub    $0x1000,%rsp
   3cc6e:	movq   $0x0,(%rsp)
   3cc76:	sub    $0x1000,%rsp
   3cc7d:	movq   $0x0,(%rsp)
   3cc85:	sub    $0x4e8,%rsp
   3cc8c:	mov    %r8d,0x1c(%rsp)
   3cc91:	mov    0x5520(%rsp),%rax
   3cc99:	movq   $0x0,0x10(%r9)
   3cca1:	movq   $0x0,0x10(%rax)
   3cca9:	cmp    $0xeffe2,%rdx
   3ccb0:	ja     3e1a7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3ccb6:	mov    %rcx,%r15
   3ccb9:	mov    %rdx,%r13
   3ccbc:	pxor   %xmm0,%xmm0
   3ccc0:	xor    %eax,%eax
   3ccc2:	pxor   %xmm2,%xmm2
   3ccc6:	pxor   %xmm1,%xmm1
   3ccca:	nopw   0x0(%rax,%rax,1)
   3ccd0:	movq   (%r15,%rax,4),%xmm3
   3ccd6:	movq   0x8(%r15,%rax,4),%xmm4
   3ccdd:	punpckldq %xmm0,%xmm3
   3cce1:	paddq  %xmm2,%xmm3
   3cce5:	punpckldq %xmm0,%xmm4
   3cce9:	paddq  %xmm1,%xmm4
   3cced:	movq   0x10(%r15,%rax,4),%xmm2
   3ccf4:	movq   0x18(%r15,%rax,4),%xmm1
   3ccfb:	punpckldq %xmm0,%xmm2
   3ccff:	paddq  %xmm3,%xmm2
   3cd03:	punpckldq %xmm0,%xmm1
   3cd07:	paddq  %xmm4,%xmm1
   3cd0b:	add    $0x8,%rax
   3cd0f:	cmp    $0x100,%rax
   3cd15:	jne    3ccd0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   3cd17:	paddq  %xmm2,%xmm1
   3cd1b:	pshufd $0xee,%xmm1,%xmm0
   3cd20:	paddq  %xmm1,%xmm0
   3cd24:	movq   %xmm0,%rax
   3cd29:	cmp    %r13,%rax
   3cd2c:	jne    3e1a7 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   3cd32:	pxor   %xmm2,%xmm2
   3cd36:	xor    %eax,%eax
   3cd38:	movdqa -0x2c350(%rip),%xmm3        # 109f0 <GCC_except_table992+0xb2c>
   3cd40:	pxor   %xmm1,%xmm1
   3cd44:	pxor   %xmm0,%xmm0
   3cd48:	nopl   0x0(%rax,%rax,1)
   3cd50:	movq   (%r15,%rax,4),%xmm4
   3cd56:	movq   0x8(%r15,%rax,4),%xmm5
   3cd5d:	pshufd $0xd4,%xmm4,%xmm4
   3cd62:	pcmpeqd %xmm2,%xmm4
   3cd66:	pandn  %xmm3,%xmm4
   3cd6a:	paddq  %xmm1,%xmm4
   3cd6e:	pshufd $0xd4,%xmm5,%xmm5
   3cd73:	pcmpeqd %xmm2,%xmm5
   3cd77:	pandn  %xmm3,%xmm5
   3cd7b:	paddq  %xmm0,%xmm5
   3cd7f:	movq   0x10(%r15,%rax,4),%xmm0
   3cd86:	movq   0x18(%r15,%rax,4),%xmm6
   3cd8d:	pshufd $0xd4,%xmm0,%xmm1
   3cd92:	pcmpeqd %xmm2,%xmm1
   3cd96:	pandn  %xmm3,%xmm1
   3cd9a:	paddq  %xmm4,%xmm1
   3cd9e:	pshufd $0xd4,%xmm6,%xmm0
   3cda3:	pcmpeqd %xmm2,%xmm0
   3cda7:	pandn  %xmm3,%xmm0
   3cdab:	paddq  %xmm5,%xmm0
   3cdaf:	add    $0x8,%rax
   3cdb3:	cmp    $0x100,%rax
   3cdb9:	jne    3cd50 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
   3cdbb:	mov    $0x88888889,%ebp
   3cdc0:	imul   %r13,%rbp
   3cdc4:	shr    $0x24,%rbp
   3cdc8:	imul   $0xeeeeeeef,%r13d,%eax
   3cdcf:	ror    $1,%eax
   3cdd1:	cmp    $0x8888889,%eax
   3cdd6:	sbb    $0xffffffffffffffff,%rbp
   3cdda:	mov    %rbp,%rbx
   3cddd:	cmpl   $0x0,0x1c(%rsp)
   3cde2:	jne    3ce38 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3cde4:	test   %r13,%r13
   3cde7:	je     3ce36 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   3cde9:	paddq  %xmm1,%xmm0
   3cded:	pshufd $0xee,%xmm0,%xmm1
   3cdf2:	paddq  %xmm0,%xmm1
   3cdf6:	movq   %xmm1,%rax
   3cdfb:	lea    -0x1(%r13),%ecx
   3cdff:	mov    $0x88888889,%edx
   3ce04:	imul   %rcx,%rdx
   3ce08:	shr    $0x23,%rdx
   3ce0c:	and    $0xfffffffe,%edx
   3ce0f:	lea    (%rdx,%r13,1),%ecx
   3ce13:	dec    %ecx
   3ce15:	shr    $1,%ecx
   3ce17:	mov    $0x80010003,%ebx
   3ce1c:	imul   %rcx,%rbx
   3ce20:	shr    $0x2e,%rbx
   3ce24:	inc    %ebx
   3ce26:	imul   %rax,%rbx
   3ce2a:	dec    %rbx
   3ce2d:	cmp    %rbp,%rbx
   3ce30:	cmovae %rbp,%rbx
   3ce34:	jmp    3ce38 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
   3ce36:	xor    %ebx,%ebx
   3ce38:	mov    (%r9),%rax
   3ce3b:	lea    0x1(%rbp),%rcx
   3ce3f:	mov    %rcx,0x80(%rsp)
   3ce47:	cmp    %rbp,%rax
   3ce4a:	mov    %rsi,0x28(%rsp)
   3ce4f:	jbe    3e30f <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
   3ce55:	mov    0x5520(%rsp),%rax
   3ce5d:	mov    (%rax),%rax
   3ce60:	cmp    %rbx,%rax
   3ce63:	jb     3e364 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
   3ce69:	movq   $0x0,0x10(%rdi)
   3ce71:	lea    0x3f(%rbp),%rbx
   3ce75:	shr    $0x6,%rbx
   3ce79:	mov    %r13,0x8(%rsp)
   3ce7e:	mov    %rdi,0x58(%rsp)
   3ce83:	mov    %rbp,0x60(%rsp)
   3ce88:	je     3cee2 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
   3ce8a:	cmp    (%rdi),%rbx
   3ce8d:	ja     3e493 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   3ce93:	mov    %r9,%r13
   3ce96:	xor    %r14d,%r14d
   3ce99:	mov    0x8(%rdi),%r12
   3ce9d:	lea    (%r12,%r14,8),%rdi
   3cea1:	cmp    $0x1,%rbx
   3cea5:	je     3cec9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3cea7:	lea    -0x8(,%rbx,8),%rdx
   3ceaf:	xor    %esi,%esi
   3ceb1:	call   *0x107089(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   3ceb7:	lea    (%r14,%rbx,1),%rax
   3cebb:	lea    (%r12,%rax,8),%rdi
   3cebf:	add    $0xfffffffffffffff8,%rdi
   3cec3:	add    %rbx,%r14
   3cec6:	dec    %r14
   3cec9:	mov    %r13,%r9
   3cecc:	movq   $0x0,(%rdi)
   3ced3:	inc    %r14
   3ced6:	mov    0x8(%rsp),%r13
   3cedb:	mov    0x58(%rsp),%rdi
   3cee0:	jmp    3cee5 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
   3cee2:	xor    %r14d,%r14d
   3cee5:	lea    0x18(%rdi),%r12
   3cee9:	mov    %r14,0x10(%rdi)
   3ceed:	movq   $0x0,0x28(%rdi)
   3cef5:	cmpq   $0xf,0x18(%rdi)
   3cefa:	jbe    3e468 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   3cf00:	lea    0x30(%rdi),%rax
   3cf04:	mov    %rax,(%rsp)
   3cf08:	xor    %eax,%eax
   3cf0a:	pxor   %xmm0,%xmm0
   3cf0e:	movabs $0xffffffff00000000,%rcx
   3cf18:	nopl   0x0(%rax,%rax,1)
   3cf20:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   3cf29:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3cf32:	mov    %rcx,0x2d08(%rsp,%rax,1)
   3cf3a:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   3cf43:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3cf4c:	mov    %rcx,0x2d30(%rsp,%rax,1)
   3cf54:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3cf5d:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   3cf66:	mov    %rcx,0x2d58(%rsp,%rax,1)
   3cf6e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   3cf77:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   3cf80:	mov    %rcx,0x2d80(%rsp,%rax,1)
   3cf88:	add    $0xa0,%rax
   3cf8e:	cmp    $0x2800,%rax
   3cf94:	jne    3cf20 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   3cf96:	mov    %r12,%r14
   3cf99:	lea    0xc(%r15),%rax
   3cf9d:	xor    %edx,%edx
   3cf9f:	mov    $0x25,%ecx
   3cfa4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3cfb0:	mov    -0xc(%rax),%esi
   3cfb3:	add    %edx,%esi
   3cfb5:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   3cfbc:	mov    %esi,0x2c70(%rsp,%rcx,4)
   3cfc3:	mov    -0x8(%rax),%edx
   3cfc6:	add    %esi,%edx
   3cfc8:	mov    %esi,0x2c94(%rsp,%rcx,4)
   3cfcf:	mov    %edx,0x2c98(%rsp,%rcx,4)
   3cfd6:	mov    -0x4(%rax),%esi
   3cfd9:	add    %edx,%esi
   3cfdb:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   3cfe2:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   3cfe9:	mov    (%rax),%edx
   3cfeb:	add    %esi,%edx
   3cfed:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3cff4:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3cffb:	add    $0x28,%rcx
   3cfff:	add    $0x10,%rax
   3d003:	cmp    $0xa25,%rcx
   3d00a:	jne    3cfb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
   3d00c:	mov    %r9,0x78(%rsp)
   3d011:	mov    0x8(%r9),%rbx
   3d015:	mov    0x8(%rdi),%rbp
   3d019:	mov    0x10(%rdi),%r12
   3d01d:	lea    0xd8(%rsp),%rdi
   3d025:	lea    0x2ce8(%rsp),%rsi
   3d02d:	mov    $0x2800,%edx
   3d032:	call   *0x106f28(%rip)        # 143f60 <memcpy@GLIBC_2.14>
   3d038:	mov    %rbp,%r8
   3d03b:	mov    %r12,%rsi
   3d03e:	mov    %rbx,0xc0(%rsp)
   3d046:	mov    0x60(%rsp),%rax
   3d04b:	mov    %rax,0xc8(%rsp)
   3d053:	mov    %r13,0xd0(%rsp)
   3d05b:	mov    0x5520(%rsp),%rax
   3d063:	mov    %rax,0x98(%rsp)
   3d06b:	mov    %rbp,0xa0(%rsp)
   3d073:	mov    %r12,0xa8(%rsp)
   3d07b:	mov    %r14,0xb0(%rsp)
   3d083:	movl   $0x0,0x28e0(%rsp)
   3d08e:	movq   $0x0,0x28d8(%rsp)
   3d09a:	mov    (%rsp),%rax
   3d09e:	mov    %rax,0xb8(%rsp)
   3d0a6:	mov    $0xffffffffffffffff,%rcx
   3d0ad:	xor    %ebp,%ebp
   3d0af:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3d0b9:	movabs $0x888888888888889,%r10
   3d0c3:	pxor   %xmm0,%xmm0
   3d0c7:	pcmpeqd %xmm1,%xmm1
   3d0cb:	xor    %r12d,%r12d
   3d0ce:	jmp    3d186 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   3d0d3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3d0e0:	mov    0x8(%rdi),%rax
   3d0e4:	mov    %r13,%rcx
   3d0e7:	shl    $0x7,%rcx
   3d0eb:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3d0f1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3d0f7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3d0fd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3d103:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3d109:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3d10f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3d115:	movdqa %xmm0,(%rax,%rcx,1)
   3d11a:	mov    %r13,%rax
   3d11d:	inc    %rax
   3d120:	mov    %rax,0x10(%rdi)
   3d124:	mov    %r14,%rax
   3d127:	shl    $0x6,%rax
   3d12b:	or     $0x10000000,%r13d
   3d132:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   3d138:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3d13e:	movdqa %xmm1,0x10(%rbx,%rax,1)
   3d144:	movdqa %xmm1,(%rbx,%rax,1)
   3d149:	mov    %r13d,0x3c(%rbx,%rax,1)
   3d14e:	mov    %r14,%rdi
   3d151:	shr    $0x6,%rdi
   3d155:	cmp    %rsi,%rdi
   3d158:	jae    3e4fa <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   3d15e:	mov    $0x1,%eax
   3d163:	mov    %r14d,%ecx
   3d166:	shl    %cl,%rax
   3d169:	or     %rax,(%r8,%rdi,8)
   3d16d:	mov    %r14,%rcx
   3d170:	mov    0x8(%rsp),%r13
   3d175:	add    $0x4,%r12
   3d179:	cmp    $0x400,%r12
   3d180:	je     3d220 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
   3d186:	mov    %rbp,%rax
   3d189:	mov    (%r15,%r12,1),%ebp
   3d18d:	add    %rax,%rbp
   3d190:	cmp    %r13,%rbp
   3d193:	setae  %dil
   3d197:	mov    %rbp,%rax
   3d19a:	movabs $0x8888888888888889,%rdx
   3d1a4:	mul    %rdx
   3d1a7:	mov    %rdx,%r14
   3d1aa:	shr    $0x4,%r14
   3d1ae:	mov    %rbp,%rax
   3d1b1:	imul   %r9,%rax
   3d1b5:	ror    $1,%rax
   3d1b8:	cmp    %r10,%rax
   3d1bb:	setb   %al
   3d1be:	cmp    %rcx,%r14
   3d1c1:	sete   %dl
   3d1c4:	or     %dil,%dl
   3d1c7:	or     %al,%dl
   3d1c9:	jne    3d175 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   3d1cb:	mov    0x5520(%rsp),%rdi
   3d1d3:	mov    0x10(%rdi),%r13
   3d1d7:	cmp    (%rdi),%r13
   3d1da:	jne    3d0e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3d1e0:	mov    %rsi,(%rsp)
   3d1e4:	mov    %r8,0x10(%rsp)
   3d1e9:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3d1ee:	pcmpeqd %xmm1,%xmm1
   3d1f2:	pxor   %xmm0,%xmm0
   3d1f6:	movabs $0x888888888888889,%r10
   3d200:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3d20a:	mov    0x10(%rsp),%r8
   3d20f:	mov    (%rsp),%rsi
   3d213:	mov    0x5520(%rsp),%rdi
   3d21b:	jmp    3d0e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   3d220:	cmpl   $0x0,0x1c(%rsp)
   3d225:	lea    0xd8(%rsp),%r8
   3d22d:	je     3d3e4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3d233:	mov    0xc8(%rsp),%r11
   3d23b:	test   %r11,%r11
   3d23e:	je     3d3e4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3d244:	mov    0x8(%rsp),%r9
   3d249:	shl    $0x20,%r9
   3d24d:	mov    0x1c(%rsp),%r10d
   3d252:	mov    0x98(%rsp),%rdi
   3d25a:	mov    0xc0(%rsp),%r13
   3d262:	xor    %r12d,%r12d
   3d265:	movabs $0xff51afd7ed558ccd,%rbx
   3d26f:	pxor   %xmm0,%xmm0
   3d273:	pcmpeqd %xmm1,%xmm1
   3d277:	jmp    3d300 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
   3d27c:	mov    0x8(%rdi),%rax
   3d280:	mov    %rbx,%rcx
   3d283:	shl    $0x7,%rcx
   3d287:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3d28d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3d293:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3d299:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3d29f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3d2a5:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3d2ab:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3d2b1:	movdqa %xmm0,(%rax,%rcx,1)
   3d2b6:	mov    %rbx,%rax
   3d2b9:	inc    %rax
   3d2bc:	mov    %rax,0x10(%rdi)
   3d2c0:	or     $0x10000000,%ebx
   3d2c6:	movdqu %xmm1,0x2c(%r13)
   3d2cc:	movdqa %xmm1,0x20(%r13)
   3d2d2:	movdqa %xmm1,0x10(%r13)
   3d2d8:	movdqa %xmm1,0x0(%r13)
   3d2de:	mov    %ebx,0x3c(%r13)
   3d2e2:	or     %r15,(%r14,%rbp,8)
   3d2e6:	movabs $0xff51afd7ed558ccd,%rbx
   3d2f0:	inc    %r12
   3d2f3:	add    $0x40,%r13
   3d2f7:	cmp    %r12,%r11
   3d2fa:	je     3d3e4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
   3d300:	mov    %r12,%rbp
   3d303:	shr    $0x6,%rbp
   3d307:	mov    0xa8(%rsp),%rsi
   3d30f:	cmp    %rsi,%rbp
   3d312:	jae    3e506 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   3d318:	mov    0xa0(%rsp),%r14
   3d320:	mov    (%r14,%rbp,8),%rax
   3d324:	mov    $0x1,%r15d
   3d32a:	mov    %r12d,%ecx
   3d32d:	shl    %cl,%r15
   3d330:	bt     %r12,%rax
   3d334:	jb     3d2f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3d336:	mov    %r12,%rax
   3d339:	xor    %r9,%rax
   3d33c:	mov    %rax,%rcx
   3d33f:	shr    $0x21,%rcx
   3d343:	xor    %rax,%rcx
   3d346:	imul   %rbx,%rcx
   3d34a:	mov    %rcx,%rax
   3d34d:	shr    $0x21,%rax
   3d351:	xor    %rcx,%rax
   3d354:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3d35e:	imul   %rcx,%rax
   3d362:	mov    %rax,%rcx
   3d365:	shr    $0x21,%rcx
   3d369:	xor    %rax,%rcx
   3d36c:	mov    %rcx,%rax
   3d36f:	shr    $0x3,%rax
   3d373:	movabs $0x20c49ba5e353f7cf,%rdx
   3d37d:	mul    %rdx
   3d380:	shr    $0x4,%rdx
   3d384:	imul   $0x3e8,%rdx,%rax
   3d38b:	sub    %rax,%rcx
   3d38e:	cmp    %r10,%rcx
   3d391:	jae    3d2f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
   3d397:	mov    0x10(%rdi),%rbx
   3d39b:	cmp    (%rdi),%rbx
   3d39e:	jne    3d27c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3d3a4:	mov    %r11,(%rsp)
   3d3a8:	mov    %r9,0x10(%rsp)
   3d3ad:	mov    %r10,0x50(%rsp)
   3d3b2:	mov    %rdi,0x48(%rsp)
   3d3b7:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3d3bc:	pcmpeqd %xmm1,%xmm1
   3d3c0:	pxor   %xmm0,%xmm0
   3d3c4:	mov    0x48(%rsp),%rdi
   3d3c9:	mov    0x50(%rsp),%r10
   3d3ce:	mov    0x10(%rsp),%r9
   3d3d3:	mov    (%rsp),%r11
   3d3d7:	lea    0xd8(%rsp),%r8
   3d3df:	jmp    3d27c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   3d3e4:	mov    0x8(%rsp),%r13
   3d3e9:	test   %r13,%r13
   3d3ec:	mov    0x28(%rsp),%r14
   3d3f1:	je     3e13f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3d3f7:	xor    %r11d,%r11d
   3d3fa:	jmp    3d40c <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3d3fc:	nopl   0x0(%rax)
   3d400:	mov    %r11,%r10
   3d403:	cmp    %r13,%r10
   3d406:	jae    3e13f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3d40c:	mov    %r11,%rbp
   3d40f:	mov    %r9,%r12
   3d412:	mov    $0x400,%edx
   3d417:	lea    0x28e8(%rsp),%rdi
   3d41f:	xor    %esi,%esi
   3d421:	call   *0x106b19(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   3d427:	lea    0x118(%rsp),%rax
   3d42f:	xor    %ecx,%ecx
   3d431:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3d440:	mov    -0x40(%rax),%rdx
   3d444:	sub    -0x30(%rax),%rdx
   3d448:	shr    $1,%rdx
   3d44b:	add    -0x28(%rax),%edx
   3d44e:	mov    -0x18(%rax),%rsi
   3d452:	sub    -0x8(%rax),%rsi
   3d456:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3d45d:	shr    $1,%rsi
   3d460:	add    (%rax),%esi
   3d462:	mov    %esi,0x28ec(%rsp,%rcx,4)
   3d469:	add    $0x2,%rcx
   3d46d:	add    $0x50,%rax
   3d471:	cmp    $0x100,%rcx
   3d478:	jne    3d440 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
   3d47a:	mov    0xb0(%rsp),%r15
   3d482:	mov    0x10(%r15),%rbx
   3d486:	cmp    (%r15),%rbx
   3d489:	je     3e132 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
   3d48f:	mov    %rbx,%rdi
   3d492:	shl    $0xa,%rdi
   3d496:	add    0x8(%r15),%rdi
   3d49a:	mov    $0x400,%edx
   3d49f:	lea    0x28e8(%rsp),%rsi
   3d4a7:	call   *0x106ab3(%rip)        # 143f60 <memcpy@GLIBC_2.14>
   3d4ad:	inc    %rbx
   3d4b0:	mov    %rbx,0x10(%r15)
   3d4b4:	mov    %rbp,%r11
   3d4b7:	lea    0xf000(%rbp),%r10
   3d4be:	cmp    %r13,%r10
   3d4c1:	cmovae %r13,%r10
   3d4c5:	lea    0x10(%rbp),%rdi
   3d4c9:	cmp    %r10,%rdi
   3d4cc:	mov    %r10,0x20(%rsp)
   3d4d1:	jbe    3d5a0 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   3d4d7:	lea    0xd8(%rsp),%r8
   3d4df:	mov    %r12,%r9
   3d4e2:	mov    0x38(%rsp),%rcx
   3d4e7:	mov    %r10,%rdx
   3d4ea:	sub    %r11,%rdx
   3d4ed:	mov    %rcx,0x38(%rsp)
   3d4f2:	jbe    3d400 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   3d4f8:	mov    %r9,%rbp
   3d4fb:	mov    %r14,%rax
   3d4fe:	mov    0x28e0(%rsp),%r12d
   3d506:	mov    $0x1e,%r14d
   3d50c:	sub    0x28d8(%rsp),%r14
   3d514:	cmp    %rdx,%r14
   3d517:	cmovae %rdx,%r14
   3d51b:	lea    (%rax,%r11,1),%r15
   3d51f:	test   %r14,%r14
   3d522:	je     3dfa0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3d528:	xor    %r13d,%r13d
   3d52b:	nopl   0x0(%rax,%rax,1)
   3d530:	movzbl (%r15,%r13,1),%esi
   3d535:	lea    (%rsi,%rsi,4),%rcx
   3d539:	lea    (%r8,%rcx,8),%rbx
   3d53d:	mov    (%r8,%rcx,8),%rax
   3d541:	cmp    0x8(%r8,%rcx,8),%rax
   3d546:	je     3d560 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   3d548:	lea    (%r12,%r13,1),%rcx
   3d54c:	mov    %cx,(%rax)
   3d54f:	addq   $0x2,(%rbx)
   3d553:	inc    %r13
   3d556:	cmp    %r13,%r14
   3d559:	jne    3d530 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   3d55b:	jmp    3dfa0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
   3d560:	mov    %rdx,0x10(%rsp)
   3d565:	mov    %r11,(%rsp)
   3d569:	lea    0x98(%rsp),%rdi
   3d571:	call   8e8f0 <<c30walk::ctor2::W<false>>::refill>
   3d576:	lea    0xd8(%rsp),%r8
   3d57e:	test   %al,%al
   3d580:	je     3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3d586:	mov    (%rbx),%rax
   3d589:	mov    0x20(%rsp),%r10
   3d58e:	mov    (%rsp),%r11
   3d592:	mov    0x10(%rsp),%rdx
   3d597:	jmp    3d548 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
   3d599:	nopl   0x0(%rax)
   3d5a0:	xor    %edx,%edx
   3d5a2:	lea    0xd8(%rsp),%r8
   3d5aa:	mov    %r12,%r9
   3d5ad:	mov    0x38(%rsp),%rcx
   3d5b2:	jmp    3d613 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   3d5b4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3d5c0:	mov    0x28d8(%rsp),%rax
   3d5c8:	lea    0x10(%rax),%rcx
   3d5cc:	add    $0xfffffffffffffff2,%rax
   3d5d0:	xor    %edx,%edx
   3d5d2:	cmp    $0x1e,%rcx
   3d5d6:	setae  %dl
   3d5d9:	cmovb  %rcx,%rax
   3d5dd:	mov    %rax,0x28d8(%rsp)
   3d5e5:	mov    0x28e0(%rsp),%eax
   3d5ec:	lea    0x10(%rax,%rdx,2),%eax
   3d5f0:	mov    %eax,0x28e0(%rsp)
   3d5f7:	mov    0x30(%rsp),%r11
   3d5fc:	lea    0x10(%r11),%rsi
   3d600:	xor    %edx,%edx
   3d602:	mov    %rsi,%rdi
   3d605:	cmp    %r10,%rsi
   3d608:	mov    0x28(%rsp),%r14
   3d60d:	ja     3d4e7 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
   3d613:	test   $0x1,%dl
   3d616:	jne    3d644 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
   3d618:	lea    0x8(%r11),%rsi
   3d61c:	cmp    $0xfffffffffffffff7,%r11
   3d620:	ja     3e3c8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3d626:	cmp    %r13,%rsi
   3d629:	ja     3e3c8 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   3d62f:	lea    0x10(%r11),%rax
   3d633:	cmp    %r13,%rax
   3d636:	ja     3e3ef <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
   3d63c:	mov    (%r14,%r11,1),%r9
   3d640:	mov    (%r14,%rsi,1),%rcx
   3d644:	mov    %r9,%rax
   3d647:	xor    %rcx,%rax
   3d64a:	mov    %r9,%rdx
   3d64d:	rol    $0x8,%rdx
   3d651:	xor    %r9,%rdx
   3d654:	or     %rax,%rdx
   3d657:	jne    3d6b0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   3d659:	mov    %rdi,%rax
   3d65c:	nopl   0x0(%rax)
   3d660:	mov    %rax,%r12
   3d663:	add    $0x10,%rax
   3d667:	cmp    %r10,%rax
   3d66a:	setbe  %r9b
   3d66e:	ja     3d710 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
   3d674:	lea    0x8(%r12),%rsi
   3d679:	cmp    $0xfffffffffffffff7,%r12
   3d67d:	ja     3e2ea <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3d683:	cmp    %r13,%rsi
   3d686:	ja     3e2ea <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   3d68c:	mov    (%r14,%r12,1),%rdx
   3d690:	mov    0x8(%r14,%r12,1),%rsi
   3d695:	cmp    %rcx,%rdx
   3d698:	jne    3d69f <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   3d69a:	cmp    %rcx,%rsi
   3d69d:	je     3d660 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   3d69f:	mov    %rsi,0x38(%rsp)
   3d6a4:	mov    %rdx,0x68(%rsp)
   3d6a9:	lea    0x10(%r12),%r10
   3d6ae:	jmp    3d715 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   3d6b0:	mov    %rdi,0x30(%rsp)
   3d6b5:	mov    0x28d8(%rsp),%rax
   3d6bd:	mov    $0x1e,%r12d
   3d6c3:	sub    %rax,%r12
   3d6c6:	cmp    $0x10,%r12
   3d6ca:	mov    $0x10,%ebx
   3d6cf:	cmovb  %r12,%rbx
   3d6d3:	mov    %rbx,%r15
   3d6d6:	add    %r11,%r15
   3d6d9:	jb     3e3da <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3d6df:	cmp    %r13,%r15
   3d6e2:	ja     3e3da <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
   3d6e8:	mov    0x28e0(%rsp),%r14d
   3d6f0:	cmp    $0x1e,%rax
   3d6f4:	jne    3de70 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
   3d6fa:	cmp    %r15,0x30(%rsp)
   3d6ff:	jae    3defa <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   3d705:	jmp    3e404 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3d70a:	nopw   0x0(%rax,%rax,1)
   3d710:	lea    0x10(%r12),%r10
   3d715:	mov    %r12,0x30(%rsp)
   3d71a:	sub    %r11,%r12
   3d71d:	movzbl %cl,%eax
   3d720:	mov    %rax,0x90(%rsp)
   3d728:	lea    (%rax,%rax,4),%rdx
   3d72c:	mov    (%r8,%rdx,8),%rax
   3d730:	mov    0x1c(%r8,%rdx,8),%esi
   3d735:	mov    %rax,%rdi
   3d738:	sub    0x10(%r8,%rdx,8),%rdi
   3d73d:	shr    $1,%rdi
   3d740:	add    0x18(%r8,%rdx,8),%edi
   3d745:	add    %r12,%rdi
   3d748:	cmp    %rsi,%rdi
   3d74b:	ja     3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3d751:	mov    %r10,0x70(%rsp)
   3d756:	mov    %r9d,0x44(%rsp)
   3d75b:	lea    (%r8,%rdx,8),%rbp
   3d75f:	movzbl %cl,%r13d
   3d763:	mov    0x8(%rbp),%rsi
   3d767:	mov    $0x88888889,%r14d
   3d76d:	lea    -0x224b4(%rip),%r15        # 1b2c0 <c30walk::ctor2::PATTERNS>
   3d774:	mov    %rbp,0x88(%rsp)
   3d77c:	jmp    3d7b7 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   3d77e:	xchg   %ax,%ax
   3d780:	mov    0x3c(%rsi),%edi
   3d783:	mov    $0xfffff,%r9d
   3d789:	and    %r9d,%edi
   3d78c:	sub    %r8d,%ecx
   3d78f:	sub    %eax,%r8d
   3d792:	mov    %r8d,%eax
   3d795:	lea    (%rsi,%rax,2),%rax
   3d799:	mov    %rax,0x10(%rbp)
   3d79d:	mov    %rax,0x0(%rbp)
   3d7a1:	lea    (%rax,%rcx,2),%rsi
   3d7a5:	mov    %rsi,0x8(%rbp)
   3d7a9:	mov    %edx,0x20(%rbp)
   3d7ac:	mov    %edi,0x24(%rbp)
   3d7af:	lea    0xd8(%rsp),%r8
   3d7b7:	sub    %rax,%rsi
   3d7ba:	shr    $1,%rsi
   3d7bd:	cmp    %rsi,%r12
   3d7c0:	cmovb  %r12,%rsi
   3d7c4:	test   %rsi,%rsi
   3d7c7:	je     3d881 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
   3d7cd:	mov    0x28d8(%rsp),%rdi
   3d7d5:	cmp    $0x1d,%rdi
   3d7d9:	ja     3e4d8 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   3d7df:	cmp    $0x1f,%rsi
   3d7e3:	jae    3e2fc <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
   3d7e9:	mov    0x28e0(%rsp),%ecx
   3d7f0:	mov    %rdi,%r11
   3d7f3:	shl    $0x6,%r11
   3d7f7:	lea    (%r15,%r11,1),%rdx
   3d7fb:	lea    -0x2(,%rsi,2),%r9
   3d803:	cmp    $0xe,%r9
   3d807:	jb     3d81c <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   3d809:	mov    %rax,%r8
   3d80c:	sub    %r15,%r8
   3d80f:	sub    %r11,%r8
   3d812:	cmp    $0x10,%r8
   3d816:	jae    3dc83 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
   3d81c:	xor    %r10d,%r10d
   3d81f:	mov    %rdx,%r8
   3d822:	lea    (%rdx,%rsi,2),%rdx
   3d826:	lea    (%rax,%r10,2),%r9
   3d82a:	nopw   0x0(%rax,%rax,1)
   3d830:	movzwl (%r8),%r10d
   3d834:	add    %cx,%r10w
   3d838:	add    $0x2,%r8
   3d83c:	mov    %r10w,(%r9)
   3d840:	add    $0x2,%r9
   3d844:	cmp    %rdx,%r8
   3d847:	jne    3d830 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
   3d849:	lea    (%rax,%rsi,2),%rax
   3d84d:	mov    %rax,0x0(%rbp)
   3d851:	add    %rsi,%rdi
   3d854:	lea    0x2(%rsi),%edx
   3d857:	mov    %rdi,%r8
   3d85a:	sub    $0x1e,%r8
   3d85e:	cmovb  %rdi,%r8
   3d862:	cmovb  %esi,%edx
   3d865:	mov    %r8,0x28d8(%rsp)
   3d86d:	add    %ecx,%edx
   3d86f:	mov    %edx,0x28e0(%rsp)
   3d876:	sub    %rsi,%r12
   3d879:	lea    0xd8(%rsp),%r8
   3d881:	test   %r12,%r12
   3d884:	je     3de60 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   3d88a:	mov    %rax,%r8
   3d88d:	cmp    0x10(%rbp),%rax
   3d891:	je     3d8b0 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
   3d893:	lea    0x98(%rsp),%rdi
   3d89b:	mov    0x90(%rsp),%rsi
   3d8a3:	call   8e2f0 <<c30walk::ctor2::W<false>>::finish>
   3d8a8:	mov    0x0(%rbp),%r8
   3d8ac:	mov    0x10(%rbp),%rax
   3d8b0:	sub    %rax,%r8
   3d8b3:	shr    $1,%r8
   3d8b6:	add    0x18(%rbp),%r8d
   3d8ba:	mov    %r8d,0x18(%rbp)
   3d8be:	pxor   %xmm0,%xmm0
   3d8c2:	movdqu %xmm0,0x0(%rbp)
   3d8c7:	movq   $0x0,0x10(%rbp)
   3d8cf:	cmp    $0x1e,%r12
   3d8d3:	jb     3ddf4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3d8d9:	mov    %r8,%rcx
   3d8dc:	imul   %r14,%rcx
   3d8e0:	shr    $0x24,%rcx
   3d8e4:	mov    %ecx,%eax
   3d8e6:	shl    $0x5,%eax
   3d8e9:	lea    (%rcx,%rcx,1),%edx
   3d8ec:	sub    %eax,%edx
   3d8ee:	add    %r8d,%edx
   3d8f1:	jne    3ddf4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   3d8f7:	mov    %r12,%rax
   3d8fa:	movabs $0x8888888888888889,%rdx
   3d904:	mul    %rdx
   3d907:	shr    $0x4,%rdx
   3d90b:	mov    %ecx,%r9d
   3d90e:	cmpl   $0x0,0x1c(%rsp)
   3d913:	mov    %r12,0x50(%rsp)
   3d918:	mov    %rdx,0x48(%rsp)
   3d91d:	je     3dc40 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
   3d923:	mov    0x28e0(%rsp),%r10d
   3d92b:	mov    0xa8(%rsp),%rax
   3d933:	mov    %rax,(%rsp)
   3d937:	mov    0x98(%rsp),%r11
   3d93f:	mov    0xa0(%rsp),%rax
   3d947:	mov    %rax,0x10(%rsp)
   3d94c:	mov    0xc0(%rsp),%rax
   3d954:	mov    0x28d8(%rsp),%rbx
   3d95c:	mov    %rbx,%rdi
   3d95f:	shl    $0x6,%rdi
   3d963:	add    %r15,%rdi
   3d966:	mov    %r9,%rcx
   3d969:	shl    $0x6,%rcx
   3d96d:	lea    (%rax,%rcx,1),%r12
   3d971:	add    $0x3c,%r12
   3d975:	mov    %r10d,%r15d
   3d978:	shl    $0x8,%r15d
   3d97c:	or     %r13d,%r15d
   3d97f:	xor    %ebp,%ebp
   3d981:	jmp    3dbb1 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
   3d986:	cs nopw 0x0(%rax,%rax,1)
   3d990:	mov    0x8(%r11),%rcx
   3d994:	shl    $0x7,%eax
   3d997:	mov    %r15d,(%rcx,%rax,1)
   3d99b:	movzwl 0x2(%rdi),%esi
   3d99f:	lea    (%r10,%rbp,1),%r14d
   3d9a3:	add    %r14d,%esi
   3d9a6:	shl    $0x8,%esi
   3d9a9:	or     %r13d,%esi
   3d9ac:	mov    %esi,0x4(%rcx,%rax,1)
   3d9b0:	movzwl 0x4(%rdi),%esi
   3d9b4:	add    %r14d,%esi
   3d9b7:	shl    $0x8,%esi
   3d9ba:	or     %r13d,%esi
   3d9bd:	mov    %esi,0x8(%rcx,%rax,1)
   3d9c1:	movzwl 0x6(%rdi),%esi
   3d9c5:	add    %r14d,%esi
   3d9c8:	shl    $0x8,%esi
   3d9cb:	or     %r13d,%esi
   3d9ce:	mov    %esi,0xc(%rcx,%rax,1)
   3d9d2:	movzwl 0x8(%rdi),%esi
   3d9d6:	add    %r14d,%esi
   3d9d9:	shl    $0x8,%esi
   3d9dc:	or     %r13d,%esi
   3d9df:	mov    %esi,0x10(%rcx,%rax,1)
   3d9e3:	movzwl 0xa(%rdi),%esi
   3d9e7:	add    %r14d,%esi
   3d9ea:	shl    $0x8,%esi
   3d9ed:	or     %r13d,%esi
   3d9f0:	mov    %esi,0x14(%rcx,%rax,1)
   3d9f4:	movzwl 0xc(%rdi),%esi
   3d9f8:	add    %r14d,%esi
   3d9fb:	shl    $0x8,%esi
   3d9fe:	or     %r13d,%esi
   3da01:	mov    %esi,0x18(%rcx,%rax,1)
   3da05:	movzwl 0xe(%rdi),%esi
   3da09:	add    %r14d,%esi
   3da0c:	shl    $0x8,%esi
   3da0f:	or     %r13d,%esi
   3da12:	mov    %esi,0x1c(%rcx,%rax,1)
   3da16:	movzwl 0x10(%rdi),%esi
   3da1a:	add    %r14d,%esi
   3da1d:	shl    $0x8,%esi
   3da20:	or     %r13d,%esi
   3da23:	mov    %esi,0x20(%rcx,%rax,1)
   3da27:	movzwl 0x12(%rdi),%esi
   3da2b:	add    %r14d,%esi
   3da2e:	shl    $0x8,%esi
   3da31:	or     %r13d,%esi
   3da34:	mov    %esi,0x24(%rcx,%rax,1)
   3da38:	movzwl 0x14(%rdi),%esi
   3da3c:	add    %r14d,%esi
   3da3f:	shl    $0x8,%esi
   3da42:	or     %r13d,%esi
   3da45:	mov    %esi,0x28(%rcx,%rax,1)
   3da49:	movzwl 0x16(%rdi),%esi
   3da4d:	add    %r14d,%esi
   3da50:	shl    $0x8,%esi
   3da53:	or     %r13d,%esi
   3da56:	mov    %esi,0x2c(%rcx,%rax,1)
   3da5a:	movzwl 0x18(%rdi),%esi
   3da5e:	add    %r14d,%esi
   3da61:	shl    $0x8,%esi
   3da64:	or     %r13d,%esi
   3da67:	mov    %esi,0x30(%rcx,%rax,1)
   3da6b:	movzwl 0x1a(%rdi),%esi
   3da6f:	add    %r14d,%esi
   3da72:	shl    $0x8,%esi
   3da75:	or     %r13d,%esi
   3da78:	mov    %esi,0x34(%rcx,%rax,1)
   3da7c:	movzwl 0x1c(%rdi),%esi
   3da80:	add    %r14d,%esi
   3da83:	shl    $0x8,%esi
   3da86:	or     %r13d,%esi
   3da89:	mov    %esi,0x38(%rcx,%rax,1)
   3da8d:	movzwl 0x1e(%rdi),%esi
   3da91:	add    %r14d,%esi
   3da94:	shl    $0x8,%esi
   3da97:	or     %r13d,%esi
   3da9a:	mov    %esi,0x3c(%rcx,%rax,1)
   3da9e:	movzwl 0x20(%rdi),%esi
   3daa2:	add    %r14d,%esi
   3daa5:	shl    $0x8,%esi
   3daa8:	or     %r13d,%esi
   3daab:	mov    %esi,0x40(%rcx,%rax,1)
   3daaf:	movzwl 0x22(%rdi),%esi
   3dab3:	add    %r14d,%esi
   3dab6:	shl    $0x8,%esi
   3dab9:	or     %r13d,%esi
   3dabc:	mov    %esi,0x44(%rcx,%rax,1)
   3dac0:	movzwl 0x24(%rdi),%esi
   3dac4:	add    %r14d,%esi
   3dac7:	shl    $0x8,%esi
   3daca:	or     %r13d,%esi
   3dacd:	mov    %esi,0x48(%rcx,%rax,1)
   3dad1:	movzwl 0x26(%rdi),%esi
   3dad5:	add    %r14d,%esi
   3dad8:	shl    $0x8,%esi
   3dadb:	or     %r13d,%esi
   3dade:	mov    %esi,0x4c(%rcx,%rax,1)
   3dae2:	movzwl 0x28(%rdi),%esi
   3dae6:	add    %r14d,%esi
   3dae9:	shl    $0x8,%esi
   3daec:	or     %r13d,%esi
   3daef:	mov    %esi,0x50(%rcx,%rax,1)
   3daf3:	movzwl 0x2a(%rdi),%esi
   3daf7:	add    %r14d,%esi
   3dafa:	shl    $0x8,%esi
   3dafd:	or     %r13d,%esi
   3db00:	mov    %esi,0x54(%rcx,%rax,1)
   3db04:	movzwl 0x2c(%rdi),%esi
   3db08:	add    %r14d,%esi
   3db0b:	shl    $0x8,%esi
   3db0e:	or     %r13d,%esi
   3db11:	mov    %esi,0x58(%rcx,%rax,1)
   3db15:	movzwl 0x2e(%rdi),%esi
   3db19:	add    %r14d,%esi
   3db1c:	shl    $0x8,%esi
   3db1f:	or     %r13d,%esi
   3db22:	mov    %esi,0x5c(%rcx,%rax,1)
   3db26:	movzwl 0x30(%rdi),%esi
   3db2a:	add    %r14d,%esi
   3db2d:	shl    $0x8,%esi
   3db30:	or     %r13d,%esi
   3db33:	mov    %esi,0x60(%rcx,%rax,1)
   3db37:	movzwl 0x32(%rdi),%esi
   3db3b:	add    %r14d,%esi
   3db3e:	shl    $0x8,%esi
   3db41:	or     %r13d,%esi
   3db44:	mov    %esi,0x64(%rcx,%rax,1)
   3db48:	movzwl 0x34(%rdi),%esi
   3db4c:	add    %r14d,%esi
   3db4f:	shl    $0x8,%esi
   3db52:	or     %r13d,%esi
   3db55:	mov    %esi,0x68(%rcx,%rax,1)
   3db59:	movzwl 0x36(%rdi),%esi
   3db5d:	add    %r14d,%esi
   3db60:	shl    $0x8,%esi
   3db63:	or     %r13d,%esi
   3db66:	mov    %esi,0x6c(%rcx,%rax,1)
   3db6a:	movzwl 0x38(%rdi),%esi
   3db6e:	add    %r14d,%esi
   3db71:	shl    $0x8,%esi
   3db74:	or     %r13d,%esi
   3db77:	mov    %esi,0x70(%rcx,%rax,1)
   3db7b:	movzwl 0x3a(%rdi),%esi
   3db7f:	add    %r14d,%esi
   3db82:	shl    $0x8,%esi
   3db85:	or     %r13d,%esi
   3db88:	mov    %esi,0x74(%rcx,%rax,1)
   3db8c:	lea    (%r10,%rbp,1),%eax
   3db90:	add    $0x20,%eax
   3db93:	mov    %eax,0x28e0(%rsp)
   3db9a:	add    $0x20,%rbp
   3db9e:	inc    %r9
   3dba1:	add    $0x2000,%r15d
   3dba8:	dec    %rdx
   3dbab:	je     3ddac <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
   3dbb1:	mov    %r9,%rax
   3dbb4:	shr    $0x6,%rax
   3dbb8:	cmp    (%rsp),%rax
   3dbbc:	jae    3e41b <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   3dbc2:	mov    0x10(%rsp),%rcx
   3dbc7:	mov    (%rcx,%rax,8),%rax
   3dbcb:	bt     %r9,%rax
   3dbcf:	jae    3dc00 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   3dbd1:	cmp    $0x1e,%rbx
   3dbd5:	jae    3e440 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   3dbdb:	mov    (%r12,%rbp,2),%eax
   3dbdf:	and    $0xfffff,%eax
   3dbe4:	mov    0x10(%r11),%rcx
   3dbe8:	cmp    %rax,%rcx
   3dbeb:	ja     3d990 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   3dbf1:	jmp    3e42e <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
   3dbf6:	cs nopw 0x0(%rax,%rax,1)
   3dc00:	cmp    $0x1e,%rbx
   3dc04:	jae    3e454 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
   3dc0a:	movups 0x2c(%rdi),%xmm0
   3dc0e:	movups %xmm0,-0x10(%r12,%rbp,2)
   3dc14:	movdqa (%rdi),%xmm0
   3dc18:	movdqa 0x10(%rdi),%xmm1
   3dc1d:	movdqa 0x20(%rdi),%xmm2
   3dc22:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   3dc29:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   3dc30:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   3dc37:	mov    %r15d,(%r12,%rbp,2)
   3dc3b:	jmp    3db8c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
   3dc40:	mov    0x28d8(%rsp),%rdi
   3dc48:	cmp    $0x1d,%rdi
   3dc4c:	ja     3e4e9 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   3dc52:	shl    $0x6,%rdi
   3dc56:	add    %r15,%rdi
   3dc59:	mov    0x28e0(%rsp),%eax
   3dc60:	mov    0xc0(%rsp),%rcx
   3dc68:	shl    $0x6,%r9
   3dc6c:	mov    %eax,%r14d
   3dc6f:	shl    $0x8,%r14d
   3dc73:	or     %r13d,%r14d
   3dc76:	cmp    $0x3c,%r12
   3dc7a:	jae    3dce5 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3dc7c:	xor    %esi,%esi
   3dc7e:	jmp    3dd6a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
   3dc83:	shr    $1,%r9
   3dc86:	inc    %r9
   3dc89:	mov    %r9,%r10
   3dc8c:	and    $0xfffffffffffffff8,%r10
   3dc90:	lea    (%rdx,%r10,2),%r8
   3dc94:	movd   %ecx,%xmm0
   3dc98:	pshuflw $0x0,%xmm0,%xmm0
   3dc9d:	add    %r15,%r11
   3dca0:	add    $0x8,%r11
   3dca4:	xor    %ebx,%ebx
   3dca6:	cs nopw 0x0(%rax,%rax,1)
   3dcb0:	movq   -0x8(%r11,%rbx,2),%xmm1
   3dcb7:	movq   (%r11,%rbx,2),%xmm2
   3dcbd:	paddw  %xmm0,%xmm1
   3dcc1:	paddw  %xmm0,%xmm2
   3dcc5:	punpcklqdq %xmm2,%xmm1
   3dcc9:	movdqu %xmm1,(%rax,%rbx,2)
   3dcce:	add    $0x8,%rbx
   3dcd2:	cmp    %rbx,%r10
   3dcd5:	jne    3dcb0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   3dcd7:	cmp    %r10,%r9
   3dcda:	jne    3d822 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   3dce0:	jmp    3d849 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
   3dce5:	mov    %rdx,%r10
   3dce8:	movabs $0xffffffffffffffe,%rsi
   3dcf2:	and    %rsi,%r10
   3dcf5:	lea    (%rcx,%r9,1),%r11
   3dcf9:	add    $0x7c,%r11
   3dcfd:	xor    %esi,%esi
   3dcff:	nop
   3dd00:	movdqu 0x2c(%rdi),%xmm0
   3dd05:	movdqu %xmm0,-0x50(%r11)
   3dd0b:	movdqa (%rdi),%xmm1
   3dd0f:	movdqa 0x10(%rdi),%xmm2
   3dd14:	movdqa 0x20(%rdi),%xmm3
   3dd19:	movdqa %xmm3,-0x5c(%r11)
   3dd1f:	movdqa %xmm2,-0x6c(%r11)
   3dd25:	movdqa %xmm1,-0x7c(%r11)
   3dd2b:	mov    %r14d,-0x40(%r11)
   3dd2f:	add    $0x2,%rsi
   3dd33:	movdqa %xmm1,-0x3c(%r11)
   3dd39:	movdqa %xmm2,-0x2c(%r11)
   3dd3f:	movdqa %xmm3,-0x1c(%r11)
   3dd45:	movdqu %xmm0,-0x10(%r11)
   3dd4b:	lea    0x2000(%r14),%ebx
   3dd52:	mov    %ebx,(%r11)
   3dd55:	add    $0x4000,%r14d
   3dd5c:	sub    $0xffffffffffffff80,%r11
   3dd60:	cmp    %rsi,%r10
   3dd63:	jne    3dd00 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   3dd65:	test   $0x1,%dl
   3dd68:	je     3dd9e <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
   3dd6a:	add    %r9,%rcx
   3dd6d:	shl    $0x6,%rsi
   3dd71:	movups 0x2c(%rdi),%xmm0
   3dd75:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3dd7a:	movdqa (%rdi),%xmm0
   3dd7e:	movdqa 0x10(%rdi),%xmm1
   3dd83:	movdqa 0x20(%rdi),%xmm2
   3dd88:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3dd8e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3dd94:	movdqa %xmm0,(%rcx,%rsi,1)
   3dd99:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3dd9e:	mov    %edx,%ecx
   3dda0:	shl    $0x5,%ecx
   3dda3:	add    %ecx,%eax
   3dda5:	mov    %eax,0x28e0(%rsp)
   3ddac:	mov    0x48(%rsp),%rdx
   3ddb1:	mov    %rdx,%rax
   3ddb4:	shl    $0x5,%rax
   3ddb8:	lea    (%rdx,%rdx,1),%rcx
   3ddbc:	sub    %rax,%rcx
   3ddbf:	mov    0x50(%rsp),%r12
   3ddc4:	add    %rcx,%r12
   3ddc7:	lea    (%rdx,%rdx,1),%eax
   3ddca:	shl    $0x5,%edx
   3ddcd:	sub    %eax,%edx
   3ddcf:	add    %edx,%r8d
   3ddd2:	mov    0x88(%rsp),%rbp
   3ddda:	mov    %r8d,0x18(%rbp)
   3ddde:	test   %r12,%r12
   3dde1:	mov    $0x88888889,%r14d
   3dde7:	lea    -0x22b2e(%rip),%r15        # 1b2c0 <c30walk::ctor2::PATTERNS>
   3ddee:	je     3df69 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
   3ddf4:	mov    0x1c(%rbp),%ecx
   3ddf7:	mov    %r8d,%edi
   3ddfa:	imul   %r14,%rdi
   3ddfe:	mov    %rdi,%rdx
   3de01:	shr    $0x24,%rdx
   3de05:	mov    %edx,%eax
   3de07:	shl    $0x5,%eax
   3de0a:	sub    %edx,%eax
   3de0c:	sub    %edx,%eax
   3de0e:	lea    0x1e(%rax),%esi
   3de11:	cmp    %ecx,%esi
   3de13:	cmovb  %esi,%ecx
   3de16:	shr    $0x2a,%rdi
   3de1a:	mov    0xa8(%rsp),%rsi
   3de22:	cmp    %rsi,%rdi
   3de25:	jae    3e4cc <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
   3de2b:	mov    %rdx,%rsi
   3de2e:	shl    $0x6,%rsi
   3de32:	add    0xc0(%rsp),%rsi
   3de3a:	mov    0xa0(%rsp),%r9
   3de42:	mov    (%r9,%rdi,8),%rdi
   3de46:	bt     %rdx,%rdi
   3de4a:	jb     3d780 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   3de50:	mov    $0xffffffff,%edi
   3de55:	jmp    3d78c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
   3de5a:	nopw   0x0(%rax,%rax,1)
   3de60:	mov    0x8(%rsp),%r13
   3de65:	jmp    3df76 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   3de6a:	nopw   0x0(%rax,%rax,1)
   3de70:	add    0x28(%rsp),%r11
   3de75:	xor    %r13d,%r13d
   3de78:	nopl   0x0(%rax,%rax,1)
   3de80:	movzbl (%r11,%r13,1),%esi
   3de85:	lea    (%rsi,%rsi,4),%rcx
   3de89:	lea    (%r8,%rcx,8),%rbp
   3de8d:	mov    (%r8,%rcx,8),%rax
   3de91:	cmp    0x8(%r8,%rcx,8),%rax
   3de96:	je     3deae <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   3de98:	lea    (%r14,%r13,1),%rcx
   3de9c:	mov    %cx,(%rax)
   3de9f:	addq   $0x2,0x0(%rbp)
   3dea4:	inc    %r13
   3dea7:	cmp    %r13,%rbx
   3deaa:	jne    3de80 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3deac:	jmp    3dee0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   3deae:	mov    %r11,(%rsp)
   3deb2:	lea    0x98(%rsp),%rdi
   3deba:	call   8e8f0 <<c30walk::ctor2::W<false>>::refill>
   3debf:	lea    0xd8(%rsp),%r8
   3dec7:	test   %al,%al
   3dec9:	je     3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3decf:	mov    0x0(%rbp),%rax
   3ded3:	mov    0x20(%rsp),%r10
   3ded8:	mov    (%rsp),%r11
   3dedc:	jmp    3de98 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
   3dede:	xchg   %ax,%ax
   3dee0:	cmp    %r15,0x30(%rsp)
   3dee5:	mov    0x8(%rsp),%r13
   3deea:	jb     3e404 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
   3def0:	cmp    $0xf,%r12
   3def4:	ja     3d5c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3defa:	add    0x28(%rsp),%r15
   3deff:	mov    %r15,%r12
   3df02:	sub    %rbx,%r12
   3df05:	add    $0x10,%r12
   3df09:	add    %r14d,%ebx
   3df0c:	add    $0x2,%ebx
   3df0f:	nop
   3df10:	movzbl (%r15),%esi
   3df14:	lea    (%rsi,%rsi,4),%rcx
   3df18:	lea    (%r8,%rcx,8),%r14
   3df1c:	mov    (%r8,%rcx,8),%rax
   3df20:	cmp    0x8(%r8,%rcx,8),%rax
   3df25:	je     3df3d <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   3df27:	mov    %bx,(%rax)
   3df2a:	addq   $0x2,(%r14)
   3df2e:	inc    %r15
   3df31:	inc    %ebx
   3df33:	cmp    %r12,%r15
   3df36:	jne    3df10 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   3df38:	jmp    3d5c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3df3d:	lea    0x98(%rsp),%rdi
   3df45:	call   8e8f0 <<c30walk::ctor2::W<false>>::refill>
   3df4a:	lea    0xd8(%rsp),%r8
   3df52:	test   %al,%al
   3df54:	je     3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3df5a:	mov    (%r14),%rax
   3df5d:	mov    0x8(%rsp),%r13
   3df62:	mov    0x20(%rsp),%r10
   3df67:	jmp    3df27 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
   3df69:	mov    0x8(%rsp),%r13
   3df6e:	lea    0xd8(%rsp),%r8
   3df76:	mov    0x20(%rsp),%r10
   3df7b:	mov    0x68(%rsp),%r9
   3df80:	mov    0x38(%rsp),%rcx
   3df85:	mov    0x30(%rsp),%r11
   3df8a:	mov    0x44(%rsp),%edx
   3df8e:	mov    0x70(%rsp),%rsi
   3df93:	jmp    3d602 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
   3df98:	nopl   0x0(%rax,%rax,1)
   3dfa0:	lea    (%r14,%r11,1),%rax
   3dfa4:	cmp    %r10,%rax
   3dfa7:	jne    3dff1 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
   3dfa9:	cmp    $0x10,%rdx
   3dfad:	jne    3e067 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
   3dfb3:	mov    0x28d8(%rsp),%rax
   3dfbb:	lea    0x10(%rax),%rcx
   3dfbf:	cmp    $0x1d,%rcx
   3dfc3:	mov    0x8(%rsp),%r13
   3dfc8:	mov    0x28(%rsp),%r14
   3dfcd:	mov    %rbp,%r9
   3dfd0:	jbe    3e10b <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
   3dfd6:	add    $0xfffffffffffffff2,%rax
   3dfda:	mov    %rax,0x28d8(%rsp)
   3dfe2:	mov    0x28e0(%rsp),%eax
   3dfe9:	add    $0x12,%eax
   3dfec:	jmp    3e11d <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3dff1:	add    $0x2,%r12d
   3dff5:	movzwl %r12w,%r12d
   3dff9:	nopl   0x0(%rax)
   3e000:	movzbl (%r15,%r14,1),%esi
   3e005:	lea    (%rsi,%rsi,4),%rcx
   3e009:	lea    (%r8,%rcx,8),%rbx
   3e00d:	mov    (%r8,%rcx,8),%rax
   3e011:	cmp    0x8(%r8,%rcx,8),%rax
   3e016:	je     3e030 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   3e018:	lea    (%r12,%r14,1),%rcx
   3e01c:	mov    %cx,(%rax)
   3e01f:	addq   $0x2,(%rbx)
   3e023:	inc    %r14
   3e026:	cmp    %r14,%rdx
   3e029:	jne    3e000 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   3e02b:	jmp    3dfa9 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
   3e030:	mov    %rdx,0x10(%rsp)
   3e035:	mov    %r11,%r13
   3e038:	lea    0x98(%rsp),%rdi
   3e040:	call   8e8f0 <<c30walk::ctor2::W<false>>::refill>
   3e045:	lea    0xd8(%rsp),%r8
   3e04d:	test   %al,%al
   3e04f:	je     3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3e055:	mov    (%rbx),%rax
   3e058:	mov    0x20(%rsp),%r10
   3e05d:	mov    %r13,%r11
   3e060:	mov    0x10(%rsp),%rdx
   3e065:	jmp    3e018 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
   3e067:	mov    0x28e0(%rsp),%eax
   3e06e:	mov    0x28d8(%rsp),%rcx
   3e076:	inc    %r11
   3e079:	cmp    %r11,%r10
   3e07c:	mov    0x8(%rsp),%r13
   3e081:	mov    0x28(%rsp),%r14
   3e086:	mov    %rbp,%r9
   3e089:	je     3e0e6 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   3e08b:	mov    %rdx,%r10
   3e08e:	and    $0xffffe,%edx
   3e094:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e0a0:	lea    0x1(%rcx),%rsi
   3e0a4:	add    $0xffffffffffffffe3,%rcx
   3e0a8:	xor    %edi,%edi
   3e0aa:	cmp    $0x1e,%rsi
   3e0ae:	setae  %dil
   3e0b2:	cmovb  %rsi,%rcx
   3e0b6:	lea    (%rax,%rdi,2),%eax
   3e0b9:	lea    0x1(%rcx),%rsi
   3e0bd:	add    $0xffffffffffffffe3,%rcx
   3e0c1:	xor    %edi,%edi
   3e0c3:	cmp    $0x1e,%rsi
   3e0c7:	setae  %dil
   3e0cb:	cmovb  %rsi,%rcx
   3e0cf:	lea    (%rax,%rdi,2),%eax
   3e0d2:	add    $0x2,%eax
   3e0d5:	add    $0xfffffffffffffffe,%rdx
   3e0d9:	jne    3e0a0 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   3e0db:	test   $0x1,%r10b
   3e0df:	mov    0x20(%rsp),%r10
   3e0e4:	je     3e101 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
   3e0e6:	lea    0x1(%rcx),%rdx
   3e0ea:	add    $0xffffffffffffffe3,%rcx
   3e0ee:	xor    %esi,%esi
   3e0f0:	cmp    $0x1e,%rdx
   3e0f4:	setae  %sil
   3e0f8:	cmovb  %rdx,%rcx
   3e0fc:	lea    (%rax,%rsi,2),%eax
   3e0ff:	inc    %eax
   3e101:	mov    %rcx,0x28d8(%rsp)
   3e109:	jmp    3e11d <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
   3e10b:	mov    %rcx,0x28d8(%rsp)
   3e113:	mov    0x28e0(%rsp),%eax
   3e11a:	add    $0x10,%eax
   3e11d:	mov    %eax,0x28e0(%rsp)
   3e124:	mov    %r10,%r11
   3e127:	cmp    %r13,%r10
   3e12a:	jb     3d40c <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   3e130:	jmp    3e13f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   3e132:	mov    %r15,%rdi
   3e135:	call   f56c0 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3e13a:	jmp    3d48f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
   3e13f:	mov    $0x84,%eax
   3e144:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3e150:	lea    0x50(%rax),%rcx
   3e154:	cmp    $0x28d4,%rcx
   3e15b:	je     3e1bb <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
   3e15d:	mov    0x54(%rsp,%rax,1),%rdx
   3e162:	sub    0x64(%rsp,%rax,1),%rdx
   3e167:	shr    $1,%rdx
   3e16a:	add    0x6c(%rsp,%rax,1),%edx
   3e16e:	cmp    0x70(%rsp,%rax,1),%edx
   3e172:	jne    3e197 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3e174:	mov    0x7c(%rsp,%rax,1),%rdx
   3e179:	sub    0x8c(%rsp,%rax,1),%rdx
   3e181:	shr    $1,%rdx
   3e184:	add    0x94(%rsp,%rax,1),%edx
   3e18b:	cmp    0x98(%rsp,%rax,1),%edx
   3e192:	mov    %rcx,%rax
   3e195:	je     3e150 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
   3e197:	mov    0x5520(%rsp),%rax
   3e19f:	movq   $0x0,0x10(%rax)
   3e1a7:	xor    %eax,%eax
   3e1a9:	add    $0x54e8,%rsp
   3e1b0:	pop    %rbx
   3e1b1:	pop    %r12
   3e1b3:	pop    %r13
   3e1b5:	pop    %r14
   3e1b7:	pop    %r15
   3e1b9:	pop    %rbp
   3e1ba:	ret
   3e1bb:	xor    %r14d,%r14d
   3e1be:	lea    0x98(%rsp),%r15
   3e1c6:	mov    0x78(%rsp),%r12
   3e1cb:	mov    0x8(%rsp),%r13
   3e1d0:	mov    0x60(%rsp),%rbp
   3e1d5:	jmp    3e20b <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   3e1d7:	nopw   0x0(%rax,%rax,1)
   3e1e0:	sub    %rcx,%rax
   3e1e3:	inc    %r14
   3e1e6:	shr    $1,%rax
   3e1e9:	add    %eax,0x18(%r8)
   3e1ed:	pxor   %xmm0,%xmm0
   3e1f1:	movdqu %xmm0,(%r8)
   3e1f6:	movq   $0x0,0x10(%r8)
   3e1fe:	add    $0x28,%r8
   3e202:	cmp    $0x100,%r14
   3e209:	je     3e231 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
   3e20b:	mov    (%r8),%rcx
   3e20e:	mov    %rcx,%rax
   3e211:	cmp    0x10(%r8),%rcx
   3e215:	je     3e1e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3e217:	mov    %r15,%rdi
   3e21a:	mov    %r14,%rsi
   3e21d:	mov    %r8,%rbx
   3e220:	call   8e2f0 <<c30walk::ctor2::W<false>>::finish>
   3e225:	mov    %rbx,%r8
   3e228:	mov    (%rbx),%rax
   3e22b:	mov    0x10(%rbx),%rcx
   3e22f:	jmp    3e1e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
   3e231:	xor    %ecx,%ecx
   3e233:	mov    0x8(%r12),%rbx
   3e238:	test   %rbp,%rbp
   3e23b:	setne  %al
   3e23e:	je     3e282 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3e240:	mov    %al,%cl
   3e242:	mov    %rbp,%rax
   3e245:	sub    %rcx,%rax
   3e248:	imul   $0xffffffffffffffe2,%rax,%rcx
   3e24c:	add    %r13,%rcx
   3e24f:	cmp    $0x1e,%rcx
   3e253:	jae    3e282 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   3e255:	lea    (%rbx,%rbp,4),%rcx
   3e259:	lea    (%rcx,%r13,2),%rdi
   3e25d:	add    $0xfffffffffffffffc,%rdi
   3e261:	lea    (%rax,%rax,1),%rcx
   3e265:	shl    $0x5,%rax
   3e269:	sub    %rcx,%rax
   3e26c:	sub    %r13,%rax
   3e26f:	lea    0x3c(,%rax,2),%rdx
   3e277:	mov    $0xff,%esi
   3e27c:	call   *0x105cbe(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   3e282:	shl    $0x6,%rbp
   3e286:	pcmpeqd %xmm0,%xmm0
   3e28a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   3e290:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   3e296:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   3e29c:	movdqa %xmm0,(%rbx,%rbp,1)
   3e2a1:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   3e2a9:	mov    0x58(%rsp),%rax
   3e2ae:	mov    0x8(%rax),%rdx
   3e2b2:	mov    0x10(%rax),%rcx
   3e2b6:	mov    0x5520(%rsp),%rax
   3e2be:	mov    0x8(%rax),%r8
   3e2c2:	mov    0x10(%rax),%r9
   3e2c6:	lea    0x98(%rsp),%rdi
   3e2ce:	mov    %rbx,%rsi
   3e2d1:	call   a2470 <c30walk::ctor::canonicalise>
   3e2d6:	mov    0x80(%rsp),%rax
   3e2de:	mov    %rax,0x10(%r12)
   3e2e3:	mov    $0x1,%al
   3e2e5:	jmp    3e1a9 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
   3e2ea:	lea    0x1022b7(%rip),%rcx        # 1405a8 <__frame_dummy_init_array_entry+0x30c8>
   3e2f1:	mov    %r12,%rdi
   3e2f4:	mov    %r13,%rdx
   3e2f7:	call   e78a0 <core::slice::index::slice_index_fail>
   3e2fc:	lea    0x1010bd(%rip),%rcx        # 13f3c0 <__frame_dummy_init_array_entry+0x1ee0>
   3e303:	mov    $0x1e,%edx
   3e308:	xor    %edi,%edi
   3e30a:	call   e78a0 <core::slice::index::slice_index_fail>
   3e30f:	mov    %rdi,%r12
   3e312:	test   %rax,%rax
   3e315:	je     3e327 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
   3e317:	mov    0x8(%r9),%rdi
   3e31b:	mov    %r9,%r14
   3e31e:	call   *0x105c2c(%rip)        # 143f50 <free@GLIBC_2.2.5>
   3e324:	mov    %r14,%r9
   3e327:	movq   $0x0,(%r9)
   3e32e:	movq   $0x40,0x8(%r9)
   3e336:	movq   $0x0,0x10(%r9)
   3e33e:	mov    $0x40,%edx
   3e343:	mov    %r9,%rdi
   3e346:	lea    0x1(%rbp),%rsi
   3e34a:	mov    %r9,%r14
   3e34d:	call   95360 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3e352:	mov    %r14,%r9
   3e355:	cmp    $0xffffffffffffffff,%rax
   3e359:	mov    %r12,%rdi
   3e35c:	je     3ce55 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   3e362:	jmp    3e3bd <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
   3e364:	mov    %rdi,%r12
   3e367:	mov    %r9,%r14
   3e36a:	test   %rax,%rax
   3e36d:	je     3e381 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
   3e36f:	mov    0x5520(%rsp),%rax
   3e377:	mov    0x8(%rax),%rdi
   3e37b:	call   *0x105bcf(%rip)        # 143f50 <free@GLIBC_2.2.5>
   3e381:	mov    0x5520(%rsp),%rdi
   3e389:	movq   $0x0,(%rdi)
   3e390:	movq   $0x40,0x8(%rdi)
   3e398:	movq   $0x0,0x10(%rdi)
   3e3a0:	mov    $0x80,%edx
   3e3a5:	mov    %rbx,%rsi
   3e3a8:	call   95360 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3e3ad:	cmp    $0xffffffffffffffff,%rax
   3e3b1:	mov    %r14,%r9
   3e3b4:	mov    %r12,%rdi
   3e3b7:	je     3ce69 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
   3e3bd:	mov    %rax,%rdi
   3e3c0:	mov    %rdx,%rsi
   3e3c3:	call   e502b <alloc::raw_vec::handle_error>
   3e3c8:	lea    0x1021d9(%rip),%rcx        # 1405a8 <__frame_dummy_init_array_entry+0x30c8>
   3e3cf:	mov    %r11,%rdi
   3e3d2:	mov    %r13,%rdx
   3e3d5:	call   e78a0 <core::slice::index::slice_index_fail>
   3e3da:	lea    0x10100f(%rip),%rcx        # 13f3f0 <__frame_dummy_init_array_entry+0x1f10>
   3e3e1:	mov    %r11,%rdi
   3e3e4:	mov    %r15,%rsi
   3e3e7:	mov    %r13,%rdx
   3e3ea:	call   e78a0 <core::slice::index::slice_index_fail>
   3e3ef:	lea    0x1021b2(%rip),%rcx        # 1405a8 <__frame_dummy_init_array_entry+0x30c8>
   3e3f6:	mov    %rsi,%rdi
   3e3f9:	mov    %rax,%rsi
   3e3fc:	mov    %r13,%rdx
   3e3ff:	call   e78a0 <core::slice::index::slice_index_fail>
   3e404:	lea    0x100fcd(%rip),%rcx        # 13f3d8 <__frame_dummy_init_array_entry+0x1ef8>
   3e40b:	mov    %r15,%rdi
   3e40e:	mov    0x30(%rsp),%rsi
   3e413:	mov    %r13,%rdx
   3e416:	call   e78a0 <core::slice::index::slice_index_fail>
   3e41b:	lea    0xff2fe(%rip),%rdx        # 13d720 <__frame_dummy_init_array_entry+0x240>
   3e422:	mov    %rax,%rdi
   3e425:	mov    (%rsp),%rsi
   3e429:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e42e:	lea    0x10104b(%rip),%rdx        # 13f480 <__frame_dummy_init_array_entry+0x1fa0>
   3e435:	mov    %rax,%rdi
   3e438:	mov    %rcx,%rsi
   3e43b:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e440:	lea    0x101021(%rip),%rdx        # 13f468 <__frame_dummy_init_array_entry+0x1f88>
   3e447:	mov    $0x1e,%esi
   3e44c:	mov    %rbx,%rdi
   3e44f:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e454:	lea    0x100ff5(%rip),%rdx        # 13f450 <__frame_dummy_init_array_entry+0x1f70>
   3e45b:	mov    $0x1e,%esi
   3e460:	mov    %rbx,%rdi
   3e463:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e468:	mov    $0x10,%edx
   3e46d:	mov    $0x4,%ecx
   3e472:	mov    $0x400,%r8d
   3e478:	mov    %rdi,%r14
   3e47b:	mov    %r12,%rdi
   3e47e:	xor    %esi,%esi
   3e480:	mov    %r9,%rbx
   3e483:	call   6aea0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e488:	mov    %r14,%rdi
   3e48b:	mov    %rbx,%r9
   3e48e:	jmp    3cf00 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
   3e493:	mov    %r9,%r13
   3e496:	mov    $0x8,%ecx
   3e49b:	mov    $0x8,%r8d
   3e4a1:	mov    %rdi,%r14
   3e4a4:	xor    %esi,%esi
   3e4a6:	mov    %rbx,%rdx
   3e4a9:	call   6aea0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3e4ae:	mov    %r14,%rdi
   3e4b1:	mov    0x10(%r14),%r14
   3e4b5:	mov    0x8(%rdi),%r12
   3e4b9:	lea    (%r12,%r14,8),%rdi
   3e4bd:	cmp    $0x1,%rbx
   3e4c1:	jne    3cea7 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   3e4c7:	jmp    3cec9 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
   3e4cc:	lea    0xff24d(%rip),%rdx        # 13d720 <__frame_dummy_init_array_entry+0x240>
   3e4d3:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e4d8:	lea    0x100ec9(%rip),%rdx        # 13f3a8 <__frame_dummy_init_array_entry+0x1ec8>
   3e4df:	mov    $0x1e,%esi
   3e4e4:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e4e9:	lea    0x100f48(%rip),%rdx        # 13f438 <__frame_dummy_init_array_entry+0x1f58>
   3e4f0:	mov    $0x1e,%esi
   3e4f5:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e4fa:	lea    0x100e2f(%rip),%rdx        # 13f330 <__frame_dummy_init_array_entry+0x1e50>
   3e501:	call   e6ac3 <core::panicking::panic_bounds_check>
   3e506:	lea    0xff213(%rip),%rdx        # 13d720 <__frame_dummy_init_array_entry+0x240>
   3e50d:	mov    %rbp,%rdi
   3e510:	call   e6ac3 <core::panicking::panic_bounds_check>
