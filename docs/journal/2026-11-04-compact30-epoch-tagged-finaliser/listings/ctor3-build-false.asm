# <c30walk::ctor3::Ctor3>::build::<false>
# size 0x1904

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000003aed0 <<c30walk::ctor3::Ctor3>::build::<false>>:
   3aed0:	push   %rbp
   3aed1:	push   %r15
   3aed3:	push   %r14
   3aed5:	push   %r13
   3aed7:	push   %r12
   3aed9:	push   %rbx
   3aeda:	sub    $0x1000,%rsp
   3aee1:	movq   $0x0,(%rsp)
   3aee9:	sub    $0x1000,%rsp
   3aef0:	movq   $0x0,(%rsp)
   3aef8:	sub    $0x1000,%rsp
   3aeff:	movq   $0x0,(%rsp)
   3af07:	sub    $0x1000,%rsp
   3af0e:	movq   $0x0,(%rsp)
   3af16:	sub    $0x1000,%rsp
   3af1d:	movq   $0x0,(%rsp)
   3af25:	sub    $0x4e8,%rsp
   3af2c:	mov    %r8d,0x1c(%rsp)
   3af31:	mov    0x5520(%rsp),%rax
   3af39:	movq   $0x0,0x10(%r9)
   3af41:	movq   $0x0,0x10(%rax)
   3af49:	cmp    $0xeffe2,%rdx
   3af50:	ja     3c467 <<c30walk::ctor3::Ctor3>::build::<false>+0x1597>
   3af56:	mov    %rcx,%r15
   3af59:	mov    %rdx,%r13
   3af5c:	pxor   %xmm0,%xmm0
   3af60:	xor    %eax,%eax
   3af62:	pxor   %xmm2,%xmm2
   3af66:	pxor   %xmm1,%xmm1
   3af6a:	nopw   0x0(%rax,%rax,1)
   3af70:	movq   (%r15,%rax,4),%xmm3
   3af76:	movq   0x8(%r15,%rax,4),%xmm4
   3af7d:	punpckldq %xmm0,%xmm3
   3af81:	paddq  %xmm2,%xmm3
   3af85:	punpckldq %xmm0,%xmm4
   3af89:	paddq  %xmm1,%xmm4
   3af8d:	movq   0x10(%r15,%rax,4),%xmm2
   3af94:	movq   0x18(%r15,%rax,4),%xmm1
   3af9b:	punpckldq %xmm0,%xmm2
   3af9f:	paddq  %xmm3,%xmm2
   3afa3:	punpckldq %xmm0,%xmm1
   3afa7:	paddq  %xmm4,%xmm1
   3afab:	add    $0x8,%rax
   3afaf:	cmp    $0x100,%rax
   3afb5:	jne    3af70 <<c30walk::ctor3::Ctor3>::build::<false>+0xa0>
   3afb7:	paddq  %xmm2,%xmm1
   3afbb:	pshufd $0xee,%xmm1,%xmm0
   3afc0:	paddq  %xmm1,%xmm0
   3afc4:	movq   %xmm0,%rax
   3afc9:	cmp    %r13,%rax
   3afcc:	jne    3c467 <<c30walk::ctor3::Ctor3>::build::<false>+0x1597>
   3afd2:	pxor   %xmm2,%xmm2
   3afd6:	xor    %eax,%eax
   3afd8:	movdqa -0x2cb90(%rip),%xmm3        # e450 <GCC_except_table943+0x9c8>
   3afe0:	pxor   %xmm1,%xmm1
   3afe4:	pxor   %xmm0,%xmm0
   3afe8:	nopl   0x0(%rax,%rax,1)
   3aff0:	movq   (%r15,%rax,4),%xmm4
   3aff6:	movq   0x8(%r15,%rax,4),%xmm5
   3affd:	pshufd $0xd4,%xmm4,%xmm4
   3b002:	pcmpeqd %xmm2,%xmm4
   3b006:	pandn  %xmm3,%xmm4
   3b00a:	paddq  %xmm1,%xmm4
   3b00e:	pshufd $0xd4,%xmm5,%xmm5
   3b013:	pcmpeqd %xmm2,%xmm5
   3b017:	pandn  %xmm3,%xmm5
   3b01b:	paddq  %xmm0,%xmm5
   3b01f:	movq   0x10(%r15,%rax,4),%xmm0
   3b026:	movq   0x18(%r15,%rax,4),%xmm6
   3b02d:	pshufd $0xd4,%xmm0,%xmm1
   3b032:	pcmpeqd %xmm2,%xmm1
   3b036:	pandn  %xmm3,%xmm1
   3b03a:	paddq  %xmm4,%xmm1
   3b03e:	pshufd $0xd4,%xmm6,%xmm0
   3b043:	pcmpeqd %xmm2,%xmm0
   3b047:	pandn  %xmm3,%xmm0
   3b04b:	paddq  %xmm5,%xmm0
   3b04f:	add    $0x8,%rax
   3b053:	cmp    $0x100,%rax
   3b059:	jne    3aff0 <<c30walk::ctor3::Ctor3>::build::<false>+0x120>
   3b05b:	mov    $0x88888889,%ebp
   3b060:	imul   %r13,%rbp
   3b064:	shr    $0x24,%rbp
   3b068:	imul   $0xeeeeeeef,%r13d,%eax
   3b06f:	ror    $1,%eax
   3b071:	cmp    $0x8888889,%eax
   3b076:	sbb    $0xffffffffffffffff,%rbp
   3b07a:	mov    %rbp,%rbx
   3b07d:	cmpl   $0x0,0x1c(%rsp)
   3b082:	jne    3b0d8 <<c30walk::ctor3::Ctor3>::build::<false>+0x208>
   3b084:	test   %r13,%r13
   3b087:	je     3b0d6 <<c30walk::ctor3::Ctor3>::build::<false>+0x206>
   3b089:	paddq  %xmm1,%xmm0
   3b08d:	pshufd $0xee,%xmm0,%xmm1
   3b092:	paddq  %xmm0,%xmm1
   3b096:	movq   %xmm1,%rax
   3b09b:	lea    -0x1(%r13),%ecx
   3b09f:	mov    $0x88888889,%edx
   3b0a4:	imul   %rcx,%rdx
   3b0a8:	shr    $0x23,%rdx
   3b0ac:	and    $0xfffffffe,%edx
   3b0af:	lea    (%rdx,%r13,1),%ecx
   3b0b3:	dec    %ecx
   3b0b5:	shr    $1,%ecx
   3b0b7:	mov    $0x80010003,%ebx
   3b0bc:	imul   %rcx,%rbx
   3b0c0:	shr    $0x2e,%rbx
   3b0c4:	inc    %ebx
   3b0c6:	imul   %rax,%rbx
   3b0ca:	dec    %rbx
   3b0cd:	cmp    %rbp,%rbx
   3b0d0:	cmovae %rbp,%rbx
   3b0d4:	jmp    3b0d8 <<c30walk::ctor3::Ctor3>::build::<false>+0x208>
   3b0d6:	xor    %ebx,%ebx
   3b0d8:	mov    (%r9),%rax
   3b0db:	lea    0x1(%rbp),%rcx
   3b0df:	mov    %rcx,0x80(%rsp)
   3b0e7:	cmp    %rbp,%rax
   3b0ea:	mov    %rsi,0x28(%rsp)
   3b0ef:	jbe    3c5d7 <<c30walk::ctor3::Ctor3>::build::<false>+0x1707>
   3b0f5:	mov    0x5520(%rsp),%rax
   3b0fd:	mov    (%rax),%rax
   3b100:	cmp    %rbx,%rax
   3b103:	jb     3c62c <<c30walk::ctor3::Ctor3>::build::<false>+0x175c>
   3b109:	movq   $0x0,0x10(%rdi)
   3b111:	lea    0x3f(%rbp),%rbx
   3b115:	shr    $0x6,%rbx
   3b119:	mov    %rdi,0x48(%rsp)
   3b11e:	mov    %r9,0x68(%rsp)
   3b123:	mov    %rbp,0x60(%rsp)
   3b128:	je     3b177 <<c30walk::ctor3::Ctor3>::build::<false>+0x2a7>
   3b12a:	cmp    (%rdi),%rbx
   3b12d:	ja     3c755 <<c30walk::ctor3::Ctor3>::build::<false>+0x1885>
   3b133:	xor    %r14d,%r14d
   3b136:	mov    0x8(%rdi),%r12
   3b13a:	lea    (%r12,%r14,8),%rdi
   3b13e:	cmp    $0x1,%rbx
   3b142:	je     3b166 <<c30walk::ctor3::Ctor3>::build::<false>+0x296>
   3b144:	lea    -0x8(,%rbx,8),%rdx
   3b14c:	xor    %esi,%esi
   3b14e:	call   *0xe19c4(%rip)        # 11cb18 <memset@GLIBC_2.2.5>
   3b154:	lea    (%r14,%rbx,1),%rax
   3b158:	lea    (%r12,%rax,8),%rdi
   3b15c:	add    $0xfffffffffffffff8,%rdi
   3b160:	add    %rbx,%r14
   3b163:	dec    %r14
   3b166:	movq   $0x0,(%rdi)
   3b16d:	inc    %r14
   3b170:	mov    0x48(%rsp),%rdi
   3b175:	jmp    3b17a <<c30walk::ctor3::Ctor3>::build::<false>+0x2aa>
   3b177:	xor    %r14d,%r14d
   3b17a:	lea    0x18(%rdi),%r12
   3b17e:	mov    %r14,0x10(%rdi)
   3b182:	movq   $0x0,0x28(%rdi)
   3b18a:	cmpq   $0xf,0x18(%rdi)
   3b18f:	jbe    3c730 <<c30walk::ctor3::Ctor3>::build::<false>+0x1860>
   3b195:	mov    %r12,(%rsp)
   3b199:	lea    0x30(%rdi),%rax
   3b19d:	mov    %rax,0x8(%rsp)
   3b1a2:	xor    %eax,%eax
   3b1a4:	pxor   %xmm0,%xmm0
   3b1a8:	movabs $0xffffffff00000000,%r12
   3b1b2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b1c0:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   3b1c9:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   3b1d2:	mov    %r12,0x2d08(%rsp,%rax,1)
   3b1da:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   3b1e3:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3b1ec:	mov    %r12,0x2d30(%rsp,%rax,1)
   3b1f4:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3b1fd:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   3b206:	mov    %r12,0x2d58(%rsp,%rax,1)
   3b20e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   3b217:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   3b220:	mov    %r12,0x2d80(%rsp,%rax,1)
   3b228:	add    $0xa0,%rax
   3b22e:	cmp    $0x2800,%rax
   3b234:	jne    3b1c0 <<c30walk::ctor3::Ctor3>::build::<false>+0x2f0>
   3b236:	lea    0xc(%r15),%rax
   3b23a:	xor    %edx,%edx
   3b23c:	mov    $0x25,%ecx
   3b241:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b250:	mov    -0xc(%rax),%esi
   3b253:	add    %edx,%esi
   3b255:	mov    %edx,0x2c6c(%rsp,%rcx,4)
   3b25c:	mov    %esi,0x2c70(%rsp,%rcx,4)
   3b263:	mov    -0x8(%rax),%edx
   3b266:	add    %esi,%edx
   3b268:	mov    %esi,0x2c94(%rsp,%rcx,4)
   3b26f:	mov    %edx,0x2c98(%rsp,%rcx,4)
   3b276:	mov    -0x4(%rax),%esi
   3b279:	add    %edx,%esi
   3b27b:	mov    %edx,0x2cbc(%rsp,%rcx,4)
   3b282:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   3b289:	mov    (%rax),%edx
   3b28b:	add    %esi,%edx
   3b28d:	mov    %esi,0x2ce4(%rsp,%rcx,4)
   3b294:	mov    %edx,0x2ce8(%rsp,%rcx,4)
   3b29b:	add    $0x28,%rcx
   3b29f:	add    $0x10,%rax
   3b2a3:	cmp    $0xa25,%rcx
   3b2aa:	jne    3b250 <<c30walk::ctor3::Ctor3>::build::<false>+0x380>
   3b2ac:	mov    0x68(%rsp),%rax
   3b2b1:	mov    0x8(%rax),%rbx
   3b2b5:	mov    0x48(%rsp),%rax
   3b2ba:	mov    0x8(%rax),%r14
   3b2be:	mov    0x10(%rax),%rbp
   3b2c2:	lea    0xd8(%rsp),%rdi
   3b2ca:	lea    0x2ce8(%rsp),%rsi
   3b2d2:	mov    $0x2800,%edx
   3b2d7:	call   *0xe185b(%rip)        # 11cb38 <memcpy@GLIBC_2.14>
   3b2dd:	mov    %r14,%r8
   3b2e0:	mov    %rbp,%rsi
   3b2e3:	mov    %rbx,0xc0(%rsp)
   3b2eb:	mov    0x60(%rsp),%rax
   3b2f0:	mov    %rax,0xc8(%rsp)
   3b2f8:	mov    %r13,0xd0(%rsp)
   3b300:	mov    0x5520(%rsp),%rax
   3b308:	mov    %rax,0x98(%rsp)
   3b310:	mov    %r14,0xa0(%rsp)
   3b318:	mov    %rbp,0xa8(%rsp)
   3b320:	mov    (%rsp),%rax
   3b324:	mov    %rax,0xb0(%rsp)
   3b32c:	mov    %r12,0x28e0(%rsp)
   3b334:	movq   $0x0,0x28d8(%rsp)
   3b340:	mov    0x8(%rsp),%rax
   3b345:	mov    %rax,0xb8(%rsp)
   3b34d:	mov    $0xffffffffffffffff,%rcx
   3b354:	xor    %ebp,%ebp
   3b356:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3b360:	movabs $0x888888888888889,%r10
   3b36a:	pxor   %xmm0,%xmm0
   3b36e:	pcmpeqd %xmm1,%xmm1
   3b372:	xor    %r12d,%r12d
   3b375:	mov    %r13,0x10(%rsp)
   3b37a:	jmp    3b426 <<c30walk::ctor3::Ctor3>::build::<false>+0x556>
   3b37f:	nop
   3b380:	mov    0x8(%rdi),%rax
   3b384:	mov    %r13,%rcx
   3b387:	shl    $0x7,%rcx
   3b38b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3b391:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3b397:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3b39d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3b3a3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3b3a9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3b3af:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3b3b5:	movdqa %xmm0,(%rax,%rcx,1)
   3b3ba:	mov    %r13,%rax
   3b3bd:	inc    %rax
   3b3c0:	mov    %rax,0x10(%rdi)
   3b3c4:	mov    %r14,%rax
   3b3c7:	shl    $0x6,%rax
   3b3cb:	or     $0x10000000,%r13d
   3b3d2:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   3b3d8:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3b3de:	movdqa %xmm1,0x10(%rbx,%rax,1)
   3b3e4:	movdqa %xmm1,(%rbx,%rax,1)
   3b3e9:	mov    %r13d,0x3c(%rbx,%rax,1)
   3b3ee:	mov    %r14,%rdi
   3b3f1:	shr    $0x6,%rdi
   3b3f5:	cmp    %rsi,%rdi
   3b3f8:	jae    3c7b9 <<c30walk::ctor3::Ctor3>::build::<false>+0x18e9>
   3b3fe:	mov    $0x1,%eax
   3b403:	mov    %r14d,%ecx
   3b406:	shl    %cl,%rax
   3b409:	or     %rax,(%r8,%rdi,8)
   3b40d:	mov    %r14,%rcx
   3b410:	mov    0x10(%rsp),%r13
   3b415:	add    $0x4,%r12
   3b419:	cmp    $0x400,%r12
   3b420:	je     3b4c0 <<c30walk::ctor3::Ctor3>::build::<false>+0x5f0>
   3b426:	mov    %rbp,%rax
   3b429:	mov    (%r15,%r12,1),%ebp
   3b42d:	add    %rax,%rbp
   3b430:	cmp    %r13,%rbp
   3b433:	setae  %dil
   3b437:	mov    %rbp,%rax
   3b43a:	movabs $0x8888888888888889,%rdx
   3b444:	mul    %rdx
   3b447:	mov    %rdx,%r14
   3b44a:	shr    $0x4,%r14
   3b44e:	mov    %rbp,%rax
   3b451:	imul   %r9,%rax
   3b455:	ror    $1,%rax
   3b458:	cmp    %r10,%rax
   3b45b:	setb   %al
   3b45e:	cmp    %rcx,%r14
   3b461:	sete   %dl
   3b464:	or     %dil,%dl
   3b467:	or     %al,%dl
   3b469:	jne    3b415 <<c30walk::ctor3::Ctor3>::build::<false>+0x545>
   3b46b:	mov    0x5520(%rsp),%rdi
   3b473:	mov    0x10(%rdi),%r13
   3b477:	cmp    (%rdi),%r13
   3b47a:	jne    3b380 <<c30walk::ctor3::Ctor3>::build::<false>+0x4b0>
   3b480:	mov    %rsi,(%rsp)
   3b484:	mov    %r8,0x8(%rsp)
   3b489:	call   87070 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3b48e:	pcmpeqd %xmm1,%xmm1
   3b492:	pxor   %xmm0,%xmm0
   3b496:	movabs $0x888888888888889,%r10
   3b4a0:	movabs $0xeeeeeeeeeeeeeeef,%r9
   3b4aa:	mov    0x8(%rsp),%r8
   3b4af:	mov    (%rsp),%rsi
   3b4b3:	mov    0x5520(%rsp),%rdi
   3b4bb:	jmp    3b380 <<c30walk::ctor3::Ctor3>::build::<false>+0x4b0>
   3b4c0:	cmpl   $0x0,0x1c(%rsp)
   3b4c5:	lea    0xd8(%rsp),%r8
   3b4cd:	je     3b684 <<c30walk::ctor3::Ctor3>::build::<false>+0x7b4>
   3b4d3:	mov    0xc8(%rsp),%r11
   3b4db:	test   %r11,%r11
   3b4de:	je     3b684 <<c30walk::ctor3::Ctor3>::build::<false>+0x7b4>
   3b4e4:	mov    0x10(%rsp),%r9
   3b4e9:	shl    $0x20,%r9
   3b4ed:	mov    0x1c(%rsp),%r10d
   3b4f2:	mov    0x98(%rsp),%rdi
   3b4fa:	mov    0xc0(%rsp),%r13
   3b502:	xor    %r12d,%r12d
   3b505:	movabs $0xff51afd7ed558ccd,%rbx
   3b50f:	pxor   %xmm0,%xmm0
   3b513:	pcmpeqd %xmm1,%xmm1
   3b517:	jmp    3b5a0 <<c30walk::ctor3::Ctor3>::build::<false>+0x6d0>
   3b51c:	mov    0x8(%rdi),%rax
   3b520:	mov    %rbx,%rcx
   3b523:	shl    $0x7,%rcx
   3b527:	movdqa %xmm0,0x70(%rax,%rcx,1)
   3b52d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   3b533:	movdqa %xmm0,0x50(%rax,%rcx,1)
   3b539:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3b53f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   3b545:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3b54b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   3b551:	movdqa %xmm0,(%rax,%rcx,1)
   3b556:	mov    %rbx,%rax
   3b559:	inc    %rax
   3b55c:	mov    %rax,0x10(%rdi)
   3b560:	or     $0x10000000,%ebx
   3b566:	movdqu %xmm1,0x2c(%r13)
   3b56c:	movdqa %xmm1,0x20(%r13)
   3b572:	movdqa %xmm1,0x10(%r13)
   3b578:	movdqa %xmm1,0x0(%r13)
   3b57e:	mov    %ebx,0x3c(%r13)
   3b582:	or     %r15,(%r14,%rbp,8)
   3b586:	movabs $0xff51afd7ed558ccd,%rbx
   3b590:	inc    %r12
   3b593:	add    $0x40,%r13
   3b597:	cmp    %r12,%r11
   3b59a:	je     3b684 <<c30walk::ctor3::Ctor3>::build::<false>+0x7b4>
   3b5a0:	mov    %r12,%rbp
   3b5a3:	shr    $0x6,%rbp
   3b5a7:	mov    0xa8(%rsp),%rsi
   3b5af:	cmp    %rsi,%rbp
   3b5b2:	jae    3c7c5 <<c30walk::ctor3::Ctor3>::build::<false>+0x18f5>
   3b5b8:	mov    0xa0(%rsp),%r14
   3b5c0:	mov    (%r14,%rbp,8),%rax
   3b5c4:	mov    $0x1,%r15d
   3b5ca:	mov    %r12d,%ecx
   3b5cd:	shl    %cl,%r15
   3b5d0:	bt     %r12,%rax
   3b5d4:	jb     3b590 <<c30walk::ctor3::Ctor3>::build::<false>+0x6c0>
   3b5d6:	mov    %r12,%rax
   3b5d9:	xor    %r9,%rax
   3b5dc:	mov    %rax,%rcx
   3b5df:	shr    $0x21,%rcx
   3b5e3:	xor    %rax,%rcx
   3b5e6:	imul   %rbx,%rcx
   3b5ea:	mov    %rcx,%rax
   3b5ed:	shr    $0x21,%rax
   3b5f1:	xor    %rcx,%rax
   3b5f4:	movabs $0xc4ceb9fe1a85ec53,%rcx
   3b5fe:	imul   %rcx,%rax
   3b602:	mov    %rax,%rcx
   3b605:	shr    $0x21,%rcx
   3b609:	xor    %rax,%rcx
   3b60c:	mov    %rcx,%rax
   3b60f:	shr    $0x3,%rax
   3b613:	movabs $0x20c49ba5e353f7cf,%rdx
   3b61d:	mul    %rdx
   3b620:	shr    $0x4,%rdx
   3b624:	imul   $0x3e8,%rdx,%rax
   3b62b:	sub    %rax,%rcx
   3b62e:	cmp    %r10,%rcx
   3b631:	jae    3b590 <<c30walk::ctor3::Ctor3>::build::<false>+0x6c0>
   3b637:	mov    0x10(%rdi),%rbx
   3b63b:	cmp    (%rdi),%rbx
   3b63e:	jne    3b51c <<c30walk::ctor3::Ctor3>::build::<false>+0x64c>
   3b644:	mov    %r11,(%rsp)
   3b648:	mov    %r9,0x8(%rsp)
   3b64d:	mov    %r10,0x58(%rsp)
   3b652:	mov    %rdi,0x50(%rsp)
   3b657:	call   87070 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3b65c:	pcmpeqd %xmm1,%xmm1
   3b660:	pxor   %xmm0,%xmm0
   3b664:	mov    0x50(%rsp),%rdi
   3b669:	mov    0x58(%rsp),%r10
   3b66e:	mov    0x8(%rsp),%r9
   3b673:	mov    (%rsp),%r11
   3b677:	lea    0xd8(%rsp),%r8
   3b67f:	jmp    3b51c <<c30walk::ctor3::Ctor3>::build::<false>+0x64c>
   3b684:	mov    0x10(%rsp),%r13
   3b689:	test   %r13,%r13
   3b68c:	mov    0x28(%rsp),%r14
   3b691:	je     3c3ff <<c30walk::ctor3::Ctor3>::build::<false>+0x152f>
   3b697:	xor    %r11d,%r11d
   3b69a:	jmp    3b6ac <<c30walk::ctor3::Ctor3>::build::<false>+0x7dc>
   3b69c:	nopl   0x0(%rax)
   3b6a0:	mov    %r11,%r10
   3b6a3:	cmp    %r13,%r10
   3b6a6:	jae    3c3ff <<c30walk::ctor3::Ctor3>::build::<false>+0x152f>
   3b6ac:	mov    %r11,%rbp
   3b6af:	mov    %r9,%r12
   3b6b2:	mov    $0x400,%edx
   3b6b7:	lea    0x28e8(%rsp),%rdi
   3b6bf:	xor    %esi,%esi
   3b6c1:	call   *0xe1451(%rip)        # 11cb18 <memset@GLIBC_2.2.5>
   3b6c7:	lea    0x118(%rsp),%rax
   3b6cf:	xor    %ecx,%ecx
   3b6d1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b6e0:	mov    -0x40(%rax),%rdx
   3b6e4:	sub    -0x30(%rax),%rdx
   3b6e8:	shr    $1,%rdx
   3b6eb:	add    -0x28(%rax),%edx
   3b6ee:	mov    -0x18(%rax),%rsi
   3b6f2:	sub    -0x8(%rax),%rsi
   3b6f6:	mov    %edx,0x28e8(%rsp,%rcx,4)
   3b6fd:	shr    $1,%rsi
   3b700:	add    (%rax),%esi
   3b702:	mov    %esi,0x28ec(%rsp,%rcx,4)
   3b709:	add    $0x2,%rcx
   3b70d:	add    $0x50,%rax
   3b711:	cmp    $0x100,%rcx
   3b718:	jne    3b6e0 <<c30walk::ctor3::Ctor3>::build::<false>+0x810>
   3b71a:	mov    0xb0(%rsp),%r15
   3b722:	mov    0x10(%r15),%rbx
   3b726:	cmp    (%r15),%rbx
   3b729:	je     3c3f2 <<c30walk::ctor3::Ctor3>::build::<false>+0x1522>
   3b72f:	mov    %rbx,%rdi
   3b732:	shl    $0xa,%rdi
   3b736:	add    0x8(%r15),%rdi
   3b73a:	mov    $0x400,%edx
   3b73f:	lea    0x28e8(%rsp),%rsi
   3b747:	call   *0xe13eb(%rip)        # 11cb38 <memcpy@GLIBC_2.14>
   3b74d:	inc    %rbx
   3b750:	mov    %rbx,0x10(%r15)
   3b754:	shl    $0x18,%ebx
   3b757:	add    $0x7f000000,%ebx
   3b75d:	or     $0x80000000,%ebx
   3b763:	mov    %ebx,0x28e4(%rsp)
   3b76a:	mov    %rbp,%r11
   3b76d:	lea    0xf000(%rbp),%r10
   3b774:	cmp    %r13,%r10
   3b777:	cmovae %r13,%r10
   3b77b:	lea    0x10(%rbp),%rdi
   3b77f:	cmp    %r10,%rdi
   3b782:	mov    %r10,0x20(%rsp)
   3b787:	jbe    3b860 <<c30walk::ctor3::Ctor3>::build::<false>+0x990>
   3b78d:	lea    0xd8(%rsp),%r8
   3b795:	mov    %r12,%r9
   3b798:	mov    0x38(%rsp),%rcx
   3b79d:	mov    %r10,%rdx
   3b7a0:	sub    %r11,%rdx
   3b7a3:	mov    %rcx,0x38(%rsp)
   3b7a8:	jbe    3b6a0 <<c30walk::ctor3::Ctor3>::build::<false>+0x7d0>
   3b7ae:	mov    %r9,%rbp
   3b7b1:	mov    %r14,%rax
   3b7b4:	mov    0x28e0(%rsp),%r12d
   3b7bc:	mov    $0x1e,%r14d
   3b7c2:	sub    0x28d8(%rsp),%r14
   3b7ca:	cmp    %rdx,%r14
   3b7cd:	cmovae %rdx,%r14
   3b7d1:	lea    (%rax,%r11,1),%r15
   3b7d5:	test   %r14,%r14
   3b7d8:	je     3c260 <<c30walk::ctor3::Ctor3>::build::<false>+0x1390>
   3b7de:	xor    %r13d,%r13d
   3b7e1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b7f0:	movzbl (%r15,%r13,1),%esi
   3b7f5:	lea    (%rsi,%rsi,4),%rcx
   3b7f9:	lea    (%r8,%rcx,8),%rbx
   3b7fd:	mov    (%r8,%rcx,8),%rax
   3b801:	cmp    0x8(%r8,%rcx,8),%rax
   3b806:	je     3b820 <<c30walk::ctor3::Ctor3>::build::<false>+0x950>
   3b808:	lea    (%r12,%r13,1),%rcx
   3b80c:	mov    %cx,(%rax)
   3b80f:	addq   $0x2,(%rbx)
   3b813:	inc    %r13
   3b816:	cmp    %r13,%r14
   3b819:	jne    3b7f0 <<c30walk::ctor3::Ctor3>::build::<false>+0x920>
   3b81b:	jmp    3c260 <<c30walk::ctor3::Ctor3>::build::<false>+0x1390>
   3b820:	mov    %rdx,0x8(%rsp)
   3b825:	mov    %r11,(%rsp)
   3b829:	lea    0x98(%rsp),%rdi
   3b831:	call   81150 <<c30walk::ctor3::W<false>>::refill>
   3b836:	lea    0xd8(%rsp),%r8
   3b83e:	test   %al,%al
   3b840:	je     3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3b846:	mov    (%rbx),%rax
   3b849:	mov    0x20(%rsp),%r10
   3b84e:	mov    (%rsp),%r11
   3b852:	mov    0x8(%rsp),%rdx
   3b857:	jmp    3b808 <<c30walk::ctor3::Ctor3>::build::<false>+0x938>
   3b859:	nopl   0x0(%rax)
   3b860:	xor    %edx,%edx
   3b862:	lea    0xd8(%rsp),%r8
   3b86a:	mov    %r12,%r9
   3b86d:	mov    0x38(%rsp),%rcx
   3b872:	jmp    3b8d3 <<c30walk::ctor3::Ctor3>::build::<false>+0xa03>
   3b874:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3b880:	mov    0x28d8(%rsp),%rax
   3b888:	lea    0x10(%rax),%rcx
   3b88c:	add    $0xfffffffffffffff2,%rax
   3b890:	xor    %edx,%edx
   3b892:	cmp    $0x1e,%rcx
   3b896:	setae  %dl
   3b899:	cmovb  %rcx,%rax
   3b89d:	mov    %rax,0x28d8(%rsp)
   3b8a5:	mov    0x28e0(%rsp),%eax
   3b8ac:	lea    0x10(%rax,%rdx,2),%eax
   3b8b0:	mov    %eax,0x28e0(%rsp)
   3b8b7:	mov    0x30(%rsp),%r11
   3b8bc:	lea    0x10(%r11),%rsi
   3b8c0:	xor    %edx,%edx
   3b8c2:	mov    %rsi,%rdi
   3b8c5:	cmp    %r10,%rsi
   3b8c8:	mov    0x28(%rsp),%r14
   3b8cd:	ja     3b79d <<c30walk::ctor3::Ctor3>::build::<false>+0x8cd>
   3b8d3:	test   $0x1,%dl
   3b8d6:	jne    3b904 <<c30walk::ctor3::Ctor3>::build::<false>+0xa34>
   3b8d8:	lea    0x8(%r11),%rsi
   3b8dc:	cmp    $0xfffffffffffffff7,%r11
   3b8e0:	ja     3c690 <<c30walk::ctor3::Ctor3>::build::<false>+0x17c0>
   3b8e6:	cmp    %r13,%rsi
   3b8e9:	ja     3c690 <<c30walk::ctor3::Ctor3>::build::<false>+0x17c0>
   3b8ef:	lea    0x10(%r11),%rax
   3b8f3:	cmp    %r13,%rax
   3b8f6:	ja     3c6b7 <<c30walk::ctor3::Ctor3>::build::<false>+0x17e7>
   3b8fc:	mov    (%r14,%r11,1),%r9
   3b900:	mov    (%r14,%rsi,1),%rcx
   3b904:	mov    %r9,%rax
   3b907:	xor    %rcx,%rax
   3b90a:	mov    %r9,%rdx
   3b90d:	rol    $0x8,%rdx
   3b911:	xor    %r9,%rdx
   3b914:	or     %rax,%rdx
   3b917:	jne    3b970 <<c30walk::ctor3::Ctor3>::build::<false>+0xaa0>
   3b919:	mov    %rdi,%rax
   3b91c:	nopl   0x0(%rax)
   3b920:	mov    %rax,%r12
   3b923:	add    $0x10,%rax
   3b927:	cmp    %r10,%rax
   3b92a:	setbe  %r9b
   3b92e:	ja     3b9d0 <<c30walk::ctor3::Ctor3>::build::<false>+0xb00>
   3b934:	lea    0x8(%r12),%rsi
   3b939:	cmp    $0xfffffffffffffff7,%r12
   3b93d:	ja     3c5b2 <<c30walk::ctor3::Ctor3>::build::<false>+0x16e2>
   3b943:	cmp    %r13,%rsi
   3b946:	ja     3c5b2 <<c30walk::ctor3::Ctor3>::build::<false>+0x16e2>
   3b94c:	mov    (%r14,%r12,1),%rdx
   3b950:	mov    0x8(%r14,%r12,1),%rsi
   3b955:	cmp    %rcx,%rdx
   3b958:	jne    3b95f <<c30walk::ctor3::Ctor3>::build::<false>+0xa8f>
   3b95a:	cmp    %rcx,%rsi
   3b95d:	je     3b920 <<c30walk::ctor3::Ctor3>::build::<false>+0xa50>
   3b95f:	mov    %rsi,0x38(%rsp)
   3b964:	mov    %rdx,0x70(%rsp)
   3b969:	lea    0x10(%r12),%r10
   3b96e:	jmp    3b9d5 <<c30walk::ctor3::Ctor3>::build::<false>+0xb05>
   3b970:	mov    %rdi,0x30(%rsp)
   3b975:	mov    0x28d8(%rsp),%rax
   3b97d:	mov    $0x1e,%r12d
   3b983:	sub    %rax,%r12
   3b986:	cmp    $0x10,%r12
   3b98a:	mov    $0x10,%ebx
   3b98f:	cmovb  %r12,%rbx
   3b993:	mov    %rbx,%r15
   3b996:	add    %r11,%r15
   3b999:	jb     3c6a2 <<c30walk::ctor3::Ctor3>::build::<false>+0x17d2>
   3b99f:	cmp    %r13,%r15
   3b9a2:	ja     3c6a2 <<c30walk::ctor3::Ctor3>::build::<false>+0x17d2>
   3b9a8:	mov    0x28e0(%rsp),%r14d
   3b9b0:	cmp    $0x1e,%rax
   3b9b4:	jne    3c130 <<c30walk::ctor3::Ctor3>::build::<false>+0x1260>
   3b9ba:	cmp    %r15,0x30(%rsp)
   3b9bf:	jae    3c1ba <<c30walk::ctor3::Ctor3>::build::<false>+0x12ea>
   3b9c5:	jmp    3c6cc <<c30walk::ctor3::Ctor3>::build::<false>+0x17fc>
   3b9ca:	nopw   0x0(%rax,%rax,1)
   3b9d0:	lea    0x10(%r12),%r10
   3b9d5:	mov    %r12,0x30(%rsp)
   3b9da:	sub    %r11,%r12
   3b9dd:	movzbl %cl,%eax
   3b9e0:	mov    %rax,0x90(%rsp)
   3b9e8:	lea    (%rax,%rax,4),%rdx
   3b9ec:	mov    (%r8,%rdx,8),%rax
   3b9f0:	mov    0x1c(%r8,%rdx,8),%esi
   3b9f5:	mov    %rax,%rdi
   3b9f8:	sub    0x10(%r8,%rdx,8),%rdi
   3b9fd:	shr    $1,%rdi
   3ba00:	add    0x18(%r8,%rdx,8),%edi
   3ba05:	add    %r12,%rdi
   3ba08:	cmp    %rsi,%rdi
   3ba0b:	ja     3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3ba11:	mov    %r10,0x78(%rsp)
   3ba16:	mov    %r9d,0x44(%rsp)
   3ba1b:	lea    (%r8,%rdx,8),%rbp
   3ba1f:	movzbl %cl,%r13d
   3ba23:	mov    0x8(%rbp),%rsi
   3ba27:	mov    $0x88888889,%r14d
   3ba2d:	lea    -0x23174(%rip),%r15        # 188c0 <c30walk::ctor3::PATTERNS>
   3ba34:	mov    %rbp,0x88(%rsp)
   3ba3c:	jmp    3ba77 <<c30walk::ctor3::Ctor3>::build::<false>+0xba7>
   3ba3e:	xchg   %ax,%ax
   3ba40:	mov    0x3c(%rsi),%edi
   3ba43:	mov    $0xfffff,%r9d
   3ba49:	and    %r9d,%edi
   3ba4c:	sub    %r8d,%ecx
   3ba4f:	sub    %eax,%r8d
   3ba52:	mov    %r8d,%eax
   3ba55:	lea    (%rsi,%rax,2),%rax
   3ba59:	mov    %rax,0x10(%rbp)
   3ba5d:	mov    %rax,0x0(%rbp)
   3ba61:	lea    (%rax,%rcx,2),%rsi
   3ba65:	mov    %rsi,0x8(%rbp)
   3ba69:	mov    %edx,0x20(%rbp)
   3ba6c:	mov    %edi,0x24(%rbp)
   3ba6f:	lea    0xd8(%rsp),%r8
   3ba77:	sub    %rax,%rsi
   3ba7a:	shr    $1,%rsi
   3ba7d:	cmp    %rsi,%r12
   3ba80:	cmovb  %r12,%rsi
   3ba84:	test   %rsi,%rsi
   3ba87:	je     3bb41 <<c30walk::ctor3::Ctor3>::build::<false>+0xc71>
   3ba8d:	mov    0x28d8(%rsp),%rdi
   3ba95:	cmp    $0x1d,%rdi
   3ba99:	ja     3c797 <<c30walk::ctor3::Ctor3>::build::<false>+0x18c7>
   3ba9f:	cmp    $0x1f,%rsi
   3baa3:	jae    3c5c4 <<c30walk::ctor3::Ctor3>::build::<false>+0x16f4>
   3baa9:	mov    0x28e0(%rsp),%ecx
   3bab0:	mov    %rdi,%r11
   3bab3:	shl    $0x6,%r11
   3bab7:	lea    (%r15,%r11,1),%rdx
   3babb:	lea    -0x2(,%rsi,2),%r9
   3bac3:	cmp    $0xe,%r9
   3bac7:	jb     3badc <<c30walk::ctor3::Ctor3>::build::<false>+0xc0c>
   3bac9:	mov    %rax,%r8
   3bacc:	sub    %r15,%r8
   3bacf:	sub    %r11,%r8
   3bad2:	cmp    $0x10,%r8
   3bad6:	jae    3bf43 <<c30walk::ctor3::Ctor3>::build::<false>+0x1073>
   3badc:	xor    %r10d,%r10d
   3badf:	mov    %rdx,%r8
   3bae2:	lea    (%rdx,%rsi,2),%rdx
   3bae6:	lea    (%rax,%r10,2),%r9
   3baea:	nopw   0x0(%rax,%rax,1)
   3baf0:	movzwl (%r8),%r10d
   3baf4:	add    %cx,%r10w
   3baf8:	add    $0x2,%r8
   3bafc:	mov    %r10w,(%r9)
   3bb00:	add    $0x2,%r9
   3bb04:	cmp    %rdx,%r8
   3bb07:	jne    3baf0 <<c30walk::ctor3::Ctor3>::build::<false>+0xc20>
   3bb09:	lea    (%rax,%rsi,2),%rax
   3bb0d:	mov    %rax,0x0(%rbp)
   3bb11:	add    %rsi,%rdi
   3bb14:	lea    0x2(%rsi),%edx
   3bb17:	mov    %rdi,%r8
   3bb1a:	sub    $0x1e,%r8
   3bb1e:	cmovb  %rdi,%r8
   3bb22:	cmovb  %esi,%edx
   3bb25:	mov    %r8,0x28d8(%rsp)
   3bb2d:	add    %ecx,%edx
   3bb2f:	mov    %edx,0x28e0(%rsp)
   3bb36:	sub    %rsi,%r12
   3bb39:	lea    0xd8(%rsp),%r8
   3bb41:	test   %r12,%r12
   3bb44:	je     3c120 <<c30walk::ctor3::Ctor3>::build::<false>+0x1250>
   3bb4a:	mov    %rax,%r8
   3bb4d:	cmp    0x10(%rbp),%rax
   3bb51:	je     3bb70 <<c30walk::ctor3::Ctor3>::build::<false>+0xca0>
   3bb53:	lea    0x98(%rsp),%rdi
   3bb5b:	mov    0x90(%rsp),%rsi
   3bb63:	call   81090 <<c30walk::ctor3::W<false>>::finish>
   3bb68:	mov    0x0(%rbp),%r8
   3bb6c:	mov    0x10(%rbp),%rax
   3bb70:	sub    %rax,%r8
   3bb73:	shr    $1,%r8
   3bb76:	add    0x18(%rbp),%r8d
   3bb7a:	mov    %r8d,0x18(%rbp)
   3bb7e:	pxor   %xmm0,%xmm0
   3bb82:	movdqu %xmm0,0x0(%rbp)
   3bb87:	movq   $0x0,0x10(%rbp)
   3bb8f:	cmp    $0x1e,%r12
   3bb93:	jb     3c0b4 <<c30walk::ctor3::Ctor3>::build::<false>+0x11e4>
   3bb99:	mov    %r8,%rcx
   3bb9c:	imul   %r14,%rcx
   3bba0:	shr    $0x24,%rcx
   3bba4:	mov    %ecx,%eax
   3bba6:	shl    $0x5,%eax
   3bba9:	lea    (%rcx,%rcx,1),%edx
   3bbac:	sub    %eax,%edx
   3bbae:	add    %r8d,%edx
   3bbb1:	jne    3c0b4 <<c30walk::ctor3::Ctor3>::build::<false>+0x11e4>
   3bbb7:	mov    %r12,%rax
   3bbba:	movabs $0x8888888888888889,%rdx
   3bbc4:	mul    %rdx
   3bbc7:	shr    $0x4,%rdx
   3bbcb:	mov    %ecx,%r9d
   3bbce:	cmpl   $0x0,0x1c(%rsp)
   3bbd3:	mov    %r12,0x58(%rsp)
   3bbd8:	mov    %rdx,0x50(%rsp)
   3bbdd:	je     3bf00 <<c30walk::ctor3::Ctor3>::build::<false>+0x1030>
   3bbe3:	mov    0x28e0(%rsp),%r10d
   3bbeb:	mov    0xa8(%rsp),%rax
   3bbf3:	mov    %rax,(%rsp)
   3bbf7:	mov    0x98(%rsp),%r11
   3bbff:	mov    0xa0(%rsp),%rax
   3bc07:	mov    %rax,0x8(%rsp)
   3bc0c:	mov    0xc0(%rsp),%rax
   3bc14:	mov    0x28d8(%rsp),%rbx
   3bc1c:	mov    %rbx,%rdi
   3bc1f:	shl    $0x6,%rdi
   3bc23:	add    %r15,%rdi
   3bc26:	mov    %r9,%rcx
   3bc29:	shl    $0x6,%rcx
   3bc2d:	lea    (%rax,%rcx,1),%r12
   3bc31:	add    $0x3c,%r12
   3bc35:	mov    %r10d,%r15d
   3bc38:	shl    $0x8,%r15d
   3bc3c:	or     %r13d,%r15d
   3bc3f:	xor    %ebp,%ebp
   3bc41:	jmp    3be71 <<c30walk::ctor3::Ctor3>::build::<false>+0xfa1>
   3bc46:	cs nopw 0x0(%rax,%rax,1)
   3bc50:	mov    0x8(%r11),%rcx
   3bc54:	shl    $0x7,%eax
   3bc57:	mov    %r15d,(%rcx,%rax,1)
   3bc5b:	movzwl 0x2(%rdi),%esi
   3bc5f:	lea    (%r10,%rbp,1),%r14d
   3bc63:	add    %r14d,%esi
   3bc66:	shl    $0x8,%esi
   3bc69:	or     %r13d,%esi
   3bc6c:	mov    %esi,0x4(%rcx,%rax,1)
   3bc70:	movzwl 0x4(%rdi),%esi
   3bc74:	add    %r14d,%esi
   3bc77:	shl    $0x8,%esi
   3bc7a:	or     %r13d,%esi
   3bc7d:	mov    %esi,0x8(%rcx,%rax,1)
   3bc81:	movzwl 0x6(%rdi),%esi
   3bc85:	add    %r14d,%esi
   3bc88:	shl    $0x8,%esi
   3bc8b:	or     %r13d,%esi
   3bc8e:	mov    %esi,0xc(%rcx,%rax,1)
   3bc92:	movzwl 0x8(%rdi),%esi
   3bc96:	add    %r14d,%esi
   3bc99:	shl    $0x8,%esi
   3bc9c:	or     %r13d,%esi
   3bc9f:	mov    %esi,0x10(%rcx,%rax,1)
   3bca3:	movzwl 0xa(%rdi),%esi
   3bca7:	add    %r14d,%esi
   3bcaa:	shl    $0x8,%esi
   3bcad:	or     %r13d,%esi
   3bcb0:	mov    %esi,0x14(%rcx,%rax,1)
   3bcb4:	movzwl 0xc(%rdi),%esi
   3bcb8:	add    %r14d,%esi
   3bcbb:	shl    $0x8,%esi
   3bcbe:	or     %r13d,%esi
   3bcc1:	mov    %esi,0x18(%rcx,%rax,1)
   3bcc5:	movzwl 0xe(%rdi),%esi
   3bcc9:	add    %r14d,%esi
   3bccc:	shl    $0x8,%esi
   3bccf:	or     %r13d,%esi
   3bcd2:	mov    %esi,0x1c(%rcx,%rax,1)
   3bcd6:	movzwl 0x10(%rdi),%esi
   3bcda:	add    %r14d,%esi
   3bcdd:	shl    $0x8,%esi
   3bce0:	or     %r13d,%esi
   3bce3:	mov    %esi,0x20(%rcx,%rax,1)
   3bce7:	movzwl 0x12(%rdi),%esi
   3bceb:	add    %r14d,%esi
   3bcee:	shl    $0x8,%esi
   3bcf1:	or     %r13d,%esi
   3bcf4:	mov    %esi,0x24(%rcx,%rax,1)
   3bcf8:	movzwl 0x14(%rdi),%esi
   3bcfc:	add    %r14d,%esi
   3bcff:	shl    $0x8,%esi
   3bd02:	or     %r13d,%esi
   3bd05:	mov    %esi,0x28(%rcx,%rax,1)
   3bd09:	movzwl 0x16(%rdi),%esi
   3bd0d:	add    %r14d,%esi
   3bd10:	shl    $0x8,%esi
   3bd13:	or     %r13d,%esi
   3bd16:	mov    %esi,0x2c(%rcx,%rax,1)
   3bd1a:	movzwl 0x18(%rdi),%esi
   3bd1e:	add    %r14d,%esi
   3bd21:	shl    $0x8,%esi
   3bd24:	or     %r13d,%esi
   3bd27:	mov    %esi,0x30(%rcx,%rax,1)
   3bd2b:	movzwl 0x1a(%rdi),%esi
   3bd2f:	add    %r14d,%esi
   3bd32:	shl    $0x8,%esi
   3bd35:	or     %r13d,%esi
   3bd38:	mov    %esi,0x34(%rcx,%rax,1)
   3bd3c:	movzwl 0x1c(%rdi),%esi
   3bd40:	add    %r14d,%esi
   3bd43:	shl    $0x8,%esi
   3bd46:	or     %r13d,%esi
   3bd49:	mov    %esi,0x38(%rcx,%rax,1)
   3bd4d:	movzwl 0x1e(%rdi),%esi
   3bd51:	add    %r14d,%esi
   3bd54:	shl    $0x8,%esi
   3bd57:	or     %r13d,%esi
   3bd5a:	mov    %esi,0x3c(%rcx,%rax,1)
   3bd5e:	movzwl 0x20(%rdi),%esi
   3bd62:	add    %r14d,%esi
   3bd65:	shl    $0x8,%esi
   3bd68:	or     %r13d,%esi
   3bd6b:	mov    %esi,0x40(%rcx,%rax,1)
   3bd6f:	movzwl 0x22(%rdi),%esi
   3bd73:	add    %r14d,%esi
   3bd76:	shl    $0x8,%esi
   3bd79:	or     %r13d,%esi
   3bd7c:	mov    %esi,0x44(%rcx,%rax,1)
   3bd80:	movzwl 0x24(%rdi),%esi
   3bd84:	add    %r14d,%esi
   3bd87:	shl    $0x8,%esi
   3bd8a:	or     %r13d,%esi
   3bd8d:	mov    %esi,0x48(%rcx,%rax,1)
   3bd91:	movzwl 0x26(%rdi),%esi
   3bd95:	add    %r14d,%esi
   3bd98:	shl    $0x8,%esi
   3bd9b:	or     %r13d,%esi
   3bd9e:	mov    %esi,0x4c(%rcx,%rax,1)
   3bda2:	movzwl 0x28(%rdi),%esi
   3bda6:	add    %r14d,%esi
   3bda9:	shl    $0x8,%esi
   3bdac:	or     %r13d,%esi
   3bdaf:	mov    %esi,0x50(%rcx,%rax,1)
   3bdb3:	movzwl 0x2a(%rdi),%esi
   3bdb7:	add    %r14d,%esi
   3bdba:	shl    $0x8,%esi
   3bdbd:	or     %r13d,%esi
   3bdc0:	mov    %esi,0x54(%rcx,%rax,1)
   3bdc4:	movzwl 0x2c(%rdi),%esi
   3bdc8:	add    %r14d,%esi
   3bdcb:	shl    $0x8,%esi
   3bdce:	or     %r13d,%esi
   3bdd1:	mov    %esi,0x58(%rcx,%rax,1)
   3bdd5:	movzwl 0x2e(%rdi),%esi
   3bdd9:	add    %r14d,%esi
   3bddc:	shl    $0x8,%esi
   3bddf:	or     %r13d,%esi
   3bde2:	mov    %esi,0x5c(%rcx,%rax,1)
   3bde6:	movzwl 0x30(%rdi),%esi
   3bdea:	add    %r14d,%esi
   3bded:	shl    $0x8,%esi
   3bdf0:	or     %r13d,%esi
   3bdf3:	mov    %esi,0x60(%rcx,%rax,1)
   3bdf7:	movzwl 0x32(%rdi),%esi
   3bdfb:	add    %r14d,%esi
   3bdfe:	shl    $0x8,%esi
   3be01:	or     %r13d,%esi
   3be04:	mov    %esi,0x64(%rcx,%rax,1)
   3be08:	movzwl 0x34(%rdi),%esi
   3be0c:	add    %r14d,%esi
   3be0f:	shl    $0x8,%esi
   3be12:	or     %r13d,%esi
   3be15:	mov    %esi,0x68(%rcx,%rax,1)
   3be19:	movzwl 0x36(%rdi),%esi
   3be1d:	add    %r14d,%esi
   3be20:	shl    $0x8,%esi
   3be23:	or     %r13d,%esi
   3be26:	mov    %esi,0x6c(%rcx,%rax,1)
   3be2a:	movzwl 0x38(%rdi),%esi
   3be2e:	add    %r14d,%esi
   3be31:	shl    $0x8,%esi
   3be34:	or     %r13d,%esi
   3be37:	mov    %esi,0x70(%rcx,%rax,1)
   3be3b:	movzwl 0x3a(%rdi),%esi
   3be3f:	add    %r14d,%esi
   3be42:	shl    $0x8,%esi
   3be45:	or     %r13d,%esi
   3be48:	mov    %esi,0x74(%rcx,%rax,1)
   3be4c:	lea    (%r10,%rbp,1),%eax
   3be50:	add    $0x20,%eax
   3be53:	mov    %eax,0x28e0(%rsp)
   3be5a:	add    $0x20,%rbp
   3be5e:	inc    %r9
   3be61:	add    $0x2000,%r15d
   3be68:	dec    %rdx
   3be6b:	je     3c06c <<c30walk::ctor3::Ctor3>::build::<false>+0x119c>
   3be71:	mov    %r9,%rax
   3be74:	shr    $0x6,%rax
   3be78:	cmp    (%rsp),%rax
   3be7c:	jae    3c6e3 <<c30walk::ctor3::Ctor3>::build::<false>+0x1813>
   3be82:	mov    0x8(%rsp),%rcx
   3be87:	mov    (%rcx,%rax,8),%rax
   3be8b:	bt     %r9,%rax
   3be8f:	jae    3bec0 <<c30walk::ctor3::Ctor3>::build::<false>+0xff0>
   3be91:	cmp    $0x1e,%rbx
   3be95:	jae    3c708 <<c30walk::ctor3::Ctor3>::build::<false>+0x1838>
   3be9b:	mov    (%r12,%rbp,2),%eax
   3be9f:	and    $0xfffff,%eax
   3bea4:	mov    0x10(%r11),%rcx
   3bea8:	cmp    %rax,%rcx
   3beab:	ja     3bc50 <<c30walk::ctor3::Ctor3>::build::<false>+0xd80>
   3beb1:	jmp    3c6f6 <<c30walk::ctor3::Ctor3>::build::<false>+0x1826>
   3beb6:	cs nopw 0x0(%rax,%rax,1)
   3bec0:	cmp    $0x1e,%rbx
   3bec4:	jae    3c71c <<c30walk::ctor3::Ctor3>::build::<false>+0x184c>
   3beca:	movups 0x2c(%rdi),%xmm0
   3bece:	movups %xmm0,-0x10(%r12,%rbp,2)
   3bed4:	movdqa (%rdi),%xmm0
   3bed8:	movdqa 0x10(%rdi),%xmm1
   3bedd:	movdqa 0x20(%rdi),%xmm2
   3bee2:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   3bee9:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   3bef0:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   3bef7:	mov    %r15d,(%r12,%rbp,2)
   3befb:	jmp    3be4c <<c30walk::ctor3::Ctor3>::build::<false>+0xf7c>
   3bf00:	mov    0x28d8(%rsp),%rdi
   3bf08:	cmp    $0x1d,%rdi
   3bf0c:	ja     3c7a8 <<c30walk::ctor3::Ctor3>::build::<false>+0x18d8>
   3bf12:	shl    $0x6,%rdi
   3bf16:	add    %r15,%rdi
   3bf19:	mov    0x28e0(%rsp),%eax
   3bf20:	mov    0xc0(%rsp),%rcx
   3bf28:	shl    $0x6,%r9
   3bf2c:	mov    %eax,%r14d
   3bf2f:	shl    $0x8,%r14d
   3bf33:	or     %r13d,%r14d
   3bf36:	cmp    $0x3c,%r12
   3bf3a:	jae    3bfa5 <<c30walk::ctor3::Ctor3>::build::<false>+0x10d5>
   3bf3c:	xor    %esi,%esi
   3bf3e:	jmp    3c02a <<c30walk::ctor3::Ctor3>::build::<false>+0x115a>
   3bf43:	shr    $1,%r9
   3bf46:	inc    %r9
   3bf49:	mov    %r9,%r10
   3bf4c:	and    $0xfffffffffffffff8,%r10
   3bf50:	lea    (%rdx,%r10,2),%r8
   3bf54:	movd   %ecx,%xmm0
   3bf58:	pshuflw $0x0,%xmm0,%xmm0
   3bf5d:	add    %r15,%r11
   3bf60:	add    $0x8,%r11
   3bf64:	xor    %ebx,%ebx
   3bf66:	cs nopw 0x0(%rax,%rax,1)
   3bf70:	movq   -0x8(%r11,%rbx,2),%xmm1
   3bf77:	movq   (%r11,%rbx,2),%xmm2
   3bf7d:	paddw  %xmm0,%xmm1
   3bf81:	paddw  %xmm0,%xmm2
   3bf85:	punpcklqdq %xmm2,%xmm1
   3bf89:	movdqu %xmm1,(%rax,%rbx,2)
   3bf8e:	add    $0x8,%rbx
   3bf92:	cmp    %rbx,%r10
   3bf95:	jne    3bf70 <<c30walk::ctor3::Ctor3>::build::<false>+0x10a0>
   3bf97:	cmp    %r10,%r9
   3bf9a:	jne    3bae2 <<c30walk::ctor3::Ctor3>::build::<false>+0xc12>
   3bfa0:	jmp    3bb09 <<c30walk::ctor3::Ctor3>::build::<false>+0xc39>
   3bfa5:	mov    %rdx,%r10
   3bfa8:	movabs $0xffffffffffffffe,%rsi
   3bfb2:	and    %rsi,%r10
   3bfb5:	lea    (%rcx,%r9,1),%r11
   3bfb9:	add    $0x7c,%r11
   3bfbd:	xor    %esi,%esi
   3bfbf:	nop
   3bfc0:	movdqu 0x2c(%rdi),%xmm0
   3bfc5:	movdqu %xmm0,-0x50(%r11)
   3bfcb:	movdqa (%rdi),%xmm1
   3bfcf:	movdqa 0x10(%rdi),%xmm2
   3bfd4:	movdqa 0x20(%rdi),%xmm3
   3bfd9:	movdqa %xmm3,-0x5c(%r11)
   3bfdf:	movdqa %xmm2,-0x6c(%r11)
   3bfe5:	movdqa %xmm1,-0x7c(%r11)
   3bfeb:	mov    %r14d,-0x40(%r11)
   3bfef:	add    $0x2,%rsi
   3bff3:	movdqa %xmm1,-0x3c(%r11)
   3bff9:	movdqa %xmm2,-0x2c(%r11)
   3bfff:	movdqa %xmm3,-0x1c(%r11)
   3c005:	movdqu %xmm0,-0x10(%r11)
   3c00b:	lea    0x2000(%r14),%ebx
   3c012:	mov    %ebx,(%r11)
   3c015:	add    $0x4000,%r14d
   3c01c:	sub    $0xffffffffffffff80,%r11
   3c020:	cmp    %rsi,%r10
   3c023:	jne    3bfc0 <<c30walk::ctor3::Ctor3>::build::<false>+0x10f0>
   3c025:	test   $0x1,%dl
   3c028:	je     3c05e <<c30walk::ctor3::Ctor3>::build::<false>+0x118e>
   3c02a:	add    %r9,%rcx
   3c02d:	shl    $0x6,%rsi
   3c031:	movups 0x2c(%rdi),%xmm0
   3c035:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3c03a:	movdqa (%rdi),%xmm0
   3c03e:	movdqa 0x10(%rdi),%xmm1
   3c043:	movdqa 0x20(%rdi),%xmm2
   3c048:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3c04e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   3c054:	movdqa %xmm0,(%rcx,%rsi,1)
   3c059:	mov    %r14d,0x3c(%rcx,%rsi,1)
   3c05e:	mov    %edx,%ecx
   3c060:	shl    $0x5,%ecx
   3c063:	add    %ecx,%eax
   3c065:	mov    %eax,0x28e0(%rsp)
   3c06c:	mov    0x50(%rsp),%rdx
   3c071:	mov    %rdx,%rax
   3c074:	shl    $0x5,%rax
   3c078:	lea    (%rdx,%rdx,1),%rcx
   3c07c:	sub    %rax,%rcx
   3c07f:	mov    0x58(%rsp),%r12
   3c084:	add    %rcx,%r12
   3c087:	lea    (%rdx,%rdx,1),%eax
   3c08a:	shl    $0x5,%edx
   3c08d:	sub    %eax,%edx
   3c08f:	add    %edx,%r8d
   3c092:	mov    0x88(%rsp),%rbp
   3c09a:	mov    %r8d,0x18(%rbp)
   3c09e:	test   %r12,%r12
   3c0a1:	mov    $0x88888889,%r14d
   3c0a7:	lea    -0x237ee(%rip),%r15        # 188c0 <c30walk::ctor3::PATTERNS>
   3c0ae:	je     3c229 <<c30walk::ctor3::Ctor3>::build::<false>+0x1359>
   3c0b4:	mov    0x1c(%rbp),%ecx
   3c0b7:	mov    %r8d,%edi
   3c0ba:	imul   %r14,%rdi
   3c0be:	mov    %rdi,%rdx
   3c0c1:	shr    $0x24,%rdx
   3c0c5:	mov    %edx,%eax
   3c0c7:	shl    $0x5,%eax
   3c0ca:	sub    %edx,%eax
   3c0cc:	sub    %edx,%eax
   3c0ce:	lea    0x1e(%rax),%esi
   3c0d1:	cmp    %ecx,%esi
   3c0d3:	cmovb  %esi,%ecx
   3c0d6:	shr    $0x2a,%rdi
   3c0da:	mov    0xa8(%rsp),%rsi
   3c0e2:	cmp    %rsi,%rdi
   3c0e5:	jae    3c78b <<c30walk::ctor3::Ctor3>::build::<false>+0x18bb>
   3c0eb:	mov    %rdx,%rsi
   3c0ee:	shl    $0x6,%rsi
   3c0f2:	add    0xc0(%rsp),%rsi
   3c0fa:	mov    0xa0(%rsp),%r9
   3c102:	mov    (%r9,%rdi,8),%rdi
   3c106:	bt     %rdx,%rdi
   3c10a:	jb     3ba40 <<c30walk::ctor3::Ctor3>::build::<false>+0xb70>
   3c110:	mov    0x28e4(%rsp),%edi
   3c117:	jmp    3ba4c <<c30walk::ctor3::Ctor3>::build::<false>+0xb7c>
   3c11c:	nopl   0x0(%rax)
   3c120:	mov    0x10(%rsp),%r13
   3c125:	jmp    3c236 <<c30walk::ctor3::Ctor3>::build::<false>+0x1366>
   3c12a:	nopw   0x0(%rax,%rax,1)
   3c130:	add    0x28(%rsp),%r11
   3c135:	xor    %r13d,%r13d
   3c138:	nopl   0x0(%rax,%rax,1)
   3c140:	movzbl (%r11,%r13,1),%esi
   3c145:	lea    (%rsi,%rsi,4),%rcx
   3c149:	lea    (%r8,%rcx,8),%rbp
   3c14d:	mov    (%r8,%rcx,8),%rax
   3c151:	cmp    0x8(%r8,%rcx,8),%rax
   3c156:	je     3c16e <<c30walk::ctor3::Ctor3>::build::<false>+0x129e>
   3c158:	lea    (%r14,%r13,1),%rcx
   3c15c:	mov    %cx,(%rax)
   3c15f:	addq   $0x2,0x0(%rbp)
   3c164:	inc    %r13
   3c167:	cmp    %r13,%rbx
   3c16a:	jne    3c140 <<c30walk::ctor3::Ctor3>::build::<false>+0x1270>
   3c16c:	jmp    3c1a0 <<c30walk::ctor3::Ctor3>::build::<false>+0x12d0>
   3c16e:	mov    %r11,(%rsp)
   3c172:	lea    0x98(%rsp),%rdi
   3c17a:	call   81150 <<c30walk::ctor3::W<false>>::refill>
   3c17f:	lea    0xd8(%rsp),%r8
   3c187:	test   %al,%al
   3c189:	je     3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3c18f:	mov    0x0(%rbp),%rax
   3c193:	mov    0x20(%rsp),%r10
   3c198:	mov    (%rsp),%r11
   3c19c:	jmp    3c158 <<c30walk::ctor3::Ctor3>::build::<false>+0x1288>
   3c19e:	xchg   %ax,%ax
   3c1a0:	cmp    %r15,0x30(%rsp)
   3c1a5:	mov    0x10(%rsp),%r13
   3c1aa:	jb     3c6cc <<c30walk::ctor3::Ctor3>::build::<false>+0x17fc>
   3c1b0:	cmp    $0xf,%r12
   3c1b4:	ja     3b880 <<c30walk::ctor3::Ctor3>::build::<false>+0x9b0>
   3c1ba:	add    0x28(%rsp),%r15
   3c1bf:	mov    %r15,%r12
   3c1c2:	sub    %rbx,%r12
   3c1c5:	add    $0x10,%r12
   3c1c9:	add    %r14d,%ebx
   3c1cc:	add    $0x2,%ebx
   3c1cf:	nop
   3c1d0:	movzbl (%r15),%esi
   3c1d4:	lea    (%rsi,%rsi,4),%rcx
   3c1d8:	lea    (%r8,%rcx,8),%r14
   3c1dc:	mov    (%r8,%rcx,8),%rax
   3c1e0:	cmp    0x8(%r8,%rcx,8),%rax
   3c1e5:	je     3c1fd <<c30walk::ctor3::Ctor3>::build::<false>+0x132d>
   3c1e7:	mov    %bx,(%rax)
   3c1ea:	addq   $0x2,(%r14)
   3c1ee:	inc    %r15
   3c1f1:	inc    %ebx
   3c1f3:	cmp    %r12,%r15
   3c1f6:	jne    3c1d0 <<c30walk::ctor3::Ctor3>::build::<false>+0x1300>
   3c1f8:	jmp    3b880 <<c30walk::ctor3::Ctor3>::build::<false>+0x9b0>
   3c1fd:	lea    0x98(%rsp),%rdi
   3c205:	call   81150 <<c30walk::ctor3::W<false>>::refill>
   3c20a:	lea    0xd8(%rsp),%r8
   3c212:	test   %al,%al
   3c214:	je     3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3c21a:	mov    (%r14),%rax
   3c21d:	mov    0x10(%rsp),%r13
   3c222:	mov    0x20(%rsp),%r10
   3c227:	jmp    3c1e7 <<c30walk::ctor3::Ctor3>::build::<false>+0x1317>
   3c229:	mov    0x10(%rsp),%r13
   3c22e:	lea    0xd8(%rsp),%r8
   3c236:	mov    0x20(%rsp),%r10
   3c23b:	mov    0x70(%rsp),%r9
   3c240:	mov    0x38(%rsp),%rcx
   3c245:	mov    0x30(%rsp),%r11
   3c24a:	mov    0x44(%rsp),%edx
   3c24e:	mov    0x78(%rsp),%rsi
   3c253:	jmp    3b8c2 <<c30walk::ctor3::Ctor3>::build::<false>+0x9f2>
   3c258:	nopl   0x0(%rax,%rax,1)
   3c260:	lea    (%r14,%r11,1),%rax
   3c264:	cmp    %r10,%rax
   3c267:	jne    3c2b1 <<c30walk::ctor3::Ctor3>::build::<false>+0x13e1>
   3c269:	cmp    $0x10,%rdx
   3c26d:	jne    3c327 <<c30walk::ctor3::Ctor3>::build::<false>+0x1457>
   3c273:	mov    0x28d8(%rsp),%rax
   3c27b:	lea    0x10(%rax),%rcx
   3c27f:	cmp    $0x1d,%rcx
   3c283:	mov    0x10(%rsp),%r13
   3c288:	mov    0x28(%rsp),%r14
   3c28d:	mov    %rbp,%r9
   3c290:	jbe    3c3cb <<c30walk::ctor3::Ctor3>::build::<false>+0x14fb>
   3c296:	add    $0xfffffffffffffff2,%rax
   3c29a:	mov    %rax,0x28d8(%rsp)
   3c2a2:	mov    0x28e0(%rsp),%eax
   3c2a9:	add    $0x12,%eax
   3c2ac:	jmp    3c3dd <<c30walk::ctor3::Ctor3>::build::<false>+0x150d>
   3c2b1:	add    $0x2,%r12d
   3c2b5:	movzwl %r12w,%r12d
   3c2b9:	nopl   0x0(%rax)
   3c2c0:	movzbl (%r15,%r14,1),%esi
   3c2c5:	lea    (%rsi,%rsi,4),%rcx
   3c2c9:	lea    (%r8,%rcx,8),%rbx
   3c2cd:	mov    (%r8,%rcx,8),%rax
   3c2d1:	cmp    0x8(%r8,%rcx,8),%rax
   3c2d6:	je     3c2f0 <<c30walk::ctor3::Ctor3>::build::<false>+0x1420>
   3c2d8:	lea    (%r12,%r14,1),%rcx
   3c2dc:	mov    %cx,(%rax)
   3c2df:	addq   $0x2,(%rbx)
   3c2e3:	inc    %r14
   3c2e6:	cmp    %r14,%rdx
   3c2e9:	jne    3c2c0 <<c30walk::ctor3::Ctor3>::build::<false>+0x13f0>
   3c2eb:	jmp    3c269 <<c30walk::ctor3::Ctor3>::build::<false>+0x1399>
   3c2f0:	mov    %rdx,0x8(%rsp)
   3c2f5:	mov    %r11,%r13
   3c2f8:	lea    0x98(%rsp),%rdi
   3c300:	call   81150 <<c30walk::ctor3::W<false>>::refill>
   3c305:	lea    0xd8(%rsp),%r8
   3c30d:	test   %al,%al
   3c30f:	je     3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3c315:	mov    (%rbx),%rax
   3c318:	mov    0x20(%rsp),%r10
   3c31d:	mov    %r13,%r11
   3c320:	mov    0x8(%rsp),%rdx
   3c325:	jmp    3c2d8 <<c30walk::ctor3::Ctor3>::build::<false>+0x1408>
   3c327:	mov    0x28e0(%rsp),%eax
   3c32e:	mov    0x28d8(%rsp),%rcx
   3c336:	inc    %r11
   3c339:	cmp    %r11,%r10
   3c33c:	mov    0x10(%rsp),%r13
   3c341:	mov    0x28(%rsp),%r14
   3c346:	mov    %rbp,%r9
   3c349:	je     3c3a6 <<c30walk::ctor3::Ctor3>::build::<false>+0x14d6>
   3c34b:	mov    %rdx,%r10
   3c34e:	and    $0xffffe,%edx
   3c354:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c360:	lea    0x1(%rcx),%rsi
   3c364:	add    $0xffffffffffffffe3,%rcx
   3c368:	xor    %edi,%edi
   3c36a:	cmp    $0x1e,%rsi
   3c36e:	setae  %dil
   3c372:	cmovb  %rsi,%rcx
   3c376:	lea    (%rax,%rdi,2),%eax
   3c379:	lea    0x1(%rcx),%rsi
   3c37d:	add    $0xffffffffffffffe3,%rcx
   3c381:	xor    %edi,%edi
   3c383:	cmp    $0x1e,%rsi
   3c387:	setae  %dil
   3c38b:	cmovb  %rsi,%rcx
   3c38f:	lea    (%rax,%rdi,2),%eax
   3c392:	add    $0x2,%eax
   3c395:	add    $0xfffffffffffffffe,%rdx
   3c399:	jne    3c360 <<c30walk::ctor3::Ctor3>::build::<false>+0x1490>
   3c39b:	test   $0x1,%r10b
   3c39f:	mov    0x20(%rsp),%r10
   3c3a4:	je     3c3c1 <<c30walk::ctor3::Ctor3>::build::<false>+0x14f1>
   3c3a6:	lea    0x1(%rcx),%rdx
   3c3aa:	add    $0xffffffffffffffe3,%rcx
   3c3ae:	xor    %esi,%esi
   3c3b0:	cmp    $0x1e,%rdx
   3c3b4:	setae  %sil
   3c3b8:	cmovb  %rdx,%rcx
   3c3bc:	lea    (%rax,%rsi,2),%eax
   3c3bf:	inc    %eax
   3c3c1:	mov    %rcx,0x28d8(%rsp)
   3c3c9:	jmp    3c3dd <<c30walk::ctor3::Ctor3>::build::<false>+0x150d>
   3c3cb:	mov    %rcx,0x28d8(%rsp)
   3c3d3:	mov    0x28e0(%rsp),%eax
   3c3da:	add    $0x10,%eax
   3c3dd:	mov    %eax,0x28e0(%rsp)
   3c3e4:	mov    %r10,%r11
   3c3e7:	cmp    %r13,%r10
   3c3ea:	jb     3b6ac <<c30walk::ctor3::Ctor3>::build::<false>+0x7dc>
   3c3f0:	jmp    3c3ff <<c30walk::ctor3::Ctor3>::build::<false>+0x152f>
   3c3f2:	mov    %r15,%rdi
   3c3f5:	call   d0240 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3c3fa:	jmp    3b72f <<c30walk::ctor3::Ctor3>::build::<false>+0x85f>
   3c3ff:	mov    $0x84,%eax
   3c404:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c410:	lea    0x50(%rax),%rcx
   3c414:	cmp    $0x28d4,%rcx
   3c41b:	je     3c47b <<c30walk::ctor3::Ctor3>::build::<false>+0x15ab>
   3c41d:	mov    0x54(%rsp,%rax,1),%rdx
   3c422:	sub    0x64(%rsp,%rax,1),%rdx
   3c427:	shr    $1,%rdx
   3c42a:	add    0x6c(%rsp,%rax,1),%edx
   3c42e:	cmp    0x70(%rsp,%rax,1),%edx
   3c432:	jne    3c457 <<c30walk::ctor3::Ctor3>::build::<false>+0x1587>
   3c434:	mov    0x7c(%rsp,%rax,1),%rdx
   3c439:	sub    0x8c(%rsp,%rax,1),%rdx
   3c441:	shr    $1,%rdx
   3c444:	add    0x94(%rsp,%rax,1),%edx
   3c44b:	cmp    0x98(%rsp,%rax,1),%edx
   3c452:	mov    %rcx,%rax
   3c455:	je     3c410 <<c30walk::ctor3::Ctor3>::build::<false>+0x1540>
   3c457:	mov    0x5520(%rsp),%rax
   3c45f:	movq   $0x0,0x10(%rax)
   3c467:	xor    %eax,%eax
   3c469:	add    $0x54e8,%rsp
   3c470:	pop    %rbx
   3c471:	pop    %r12
   3c473:	pop    %r13
   3c475:	pop    %r14
   3c477:	pop    %r15
   3c479:	pop    %rbp
   3c47a:	ret
   3c47b:	xor    %r14d,%r14d
   3c47e:	lea    0x98(%rsp),%r15
   3c486:	mov    0x10(%rsp),%r12
   3c48b:	mov    0x60(%rsp),%r13
   3c490:	jmp    3c4cb <<c30walk::ctor3::Ctor3>::build::<false>+0x15fb>
   3c492:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3c4a0:	sub    %rcx,%rax
   3c4a3:	inc    %r14
   3c4a6:	shr    $1,%rax
   3c4a9:	add    %eax,0x18(%r8)
   3c4ad:	pxor   %xmm0,%xmm0
   3c4b1:	movdqu %xmm0,(%r8)
   3c4b6:	movq   $0x0,0x10(%r8)
   3c4be:	add    $0x28,%r8
   3c4c2:	cmp    $0x100,%r14
   3c4c9:	je     3c4f1 <<c30walk::ctor3::Ctor3>::build::<false>+0x1621>
   3c4cb:	mov    (%r8),%rcx
   3c4ce:	mov    %rcx,%rax
   3c4d1:	cmp    0x10(%r8),%rcx
   3c4d5:	je     3c4a0 <<c30walk::ctor3::Ctor3>::build::<false>+0x15d0>
   3c4d7:	mov    %r15,%rdi
   3c4da:	mov    %r14d,%esi
   3c4dd:	mov    %r8,%rbx
   3c4e0:	call   81090 <<c30walk::ctor3::W<false>>::finish>
   3c4e5:	mov    %rbx,%r8
   3c4e8:	mov    (%rbx),%rax
   3c4eb:	mov    0x10(%rbx),%rcx
   3c4ef:	jmp    3c4a0 <<c30walk::ctor3::Ctor3>::build::<false>+0x15d0>
   3c4f1:	xor    %ecx,%ecx
   3c4f3:	mov    0x68(%rsp),%r14
   3c4f8:	mov    0x8(%r14),%rbx
   3c4fc:	test   %r13,%r13
   3c4ff:	setne  %al
   3c502:	je     3c546 <<c30walk::ctor3::Ctor3>::build::<false>+0x1676>
   3c504:	mov    %al,%cl
   3c506:	mov    %r13,%rax
   3c509:	sub    %rcx,%rax
   3c50c:	imul   $0xffffffffffffffe2,%rax,%rcx
   3c510:	add    %r12,%rcx
   3c513:	cmp    $0x1e,%rcx
   3c517:	jae    3c546 <<c30walk::ctor3::Ctor3>::build::<false>+0x1676>
   3c519:	lea    (%rbx,%r13,4),%rcx
   3c51d:	lea    (%rcx,%r12,2),%rdi
   3c521:	add    $0xfffffffffffffffc,%rdi
   3c525:	lea    (%rax,%rax,1),%rcx
   3c529:	shl    $0x5,%rax
   3c52d:	sub    %rcx,%rax
   3c530:	sub    %r12,%rax
   3c533:	lea    0x3c(,%rax,2),%rdx
   3c53b:	mov    $0xff,%esi
   3c540:	call   *0xe05d2(%rip)        # 11cb18 <memset@GLIBC_2.2.5>
   3c546:	shl    $0x6,%r13
   3c54a:	pcmpeqd %xmm0,%xmm0
   3c54e:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   3c555:	movdqa %xmm0,0x20(%rbx,%r13,1)
   3c55c:	movdqa %xmm0,0x10(%rbx,%r13,1)
   3c563:	movdqa %xmm0,(%rbx,%r13,1)
   3c569:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   3c572:	mov    0x48(%rsp),%rax
   3c577:	mov    0x8(%rax),%rdx
   3c57b:	mov    0x10(%rax),%rcx
   3c57f:	mov    0x5520(%rsp),%rax
   3c587:	mov    0x8(%rax),%r8
   3c58b:	mov    0x10(%rax),%r9
   3c58f:	lea    0x98(%rsp),%rdi
   3c597:	mov    %rbx,%rsi
   3c59a:	call   8fa30 <c30walk::ctor::canonicalise>
   3c59f:	mov    0x80(%rsp),%rax
   3c5a7:	mov    %rax,0x10(%r14)
   3c5ab:	mov    $0x1,%al
   3c5ad:	jmp    3c469 <<c30walk::ctor3::Ctor3>::build::<false>+0x1599>
   3c5b2:	lea    0xddb07(%rip),%rcx        # 11a0c0 <__frame_dummy_init_array_entry+0x3090>
   3c5b9:	mov    %r12,%rdi
   3c5bc:	mov    %r13,%rdx
   3c5bf:	call   c2420 <core::slice::index::slice_index_fail>
   3c5c4:	lea    0xdc78d(%rip),%rcx        # 118d58 <__frame_dummy_init_array_entry+0x1d28>
   3c5cb:	mov    $0x1e,%edx
   3c5d0:	xor    %edi,%edi
   3c5d2:	call   c2420 <core::slice::index::slice_index_fail>
   3c5d7:	mov    %rdi,%r12
   3c5da:	test   %rax,%rax
   3c5dd:	je     3c5ef <<c30walk::ctor3::Ctor3>::build::<false>+0x171f>
   3c5df:	mov    0x8(%r9),%rdi
   3c5e3:	mov    %r9,%r14
   3c5e6:	call   *0xe053c(%rip)        # 11cb28 <free@GLIBC_2.2.5>
   3c5ec:	mov    %r14,%r9
   3c5ef:	movq   $0x0,(%r9)
   3c5f6:	movq   $0x40,0x8(%r9)
   3c5fe:	movq   $0x0,0x10(%r9)
   3c606:	mov    $0x40,%edx
   3c60b:	mov    %r9,%rdi
   3c60e:	lea    0x1(%rbp),%rsi
   3c612:	mov    %r9,%r14
   3c615:	call   86400 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c61a:	mov    %r14,%r9
   3c61d:	cmp    $0xffffffffffffffff,%rax
   3c621:	mov    %r12,%rdi
   3c624:	je     3b0f5 <<c30walk::ctor3::Ctor3>::build::<false>+0x225>
   3c62a:	jmp    3c685 <<c30walk::ctor3::Ctor3>::build::<false>+0x17b5>
   3c62c:	mov    %rdi,%r12
   3c62f:	mov    %r9,%r14
   3c632:	test   %rax,%rax
   3c635:	je     3c649 <<c30walk::ctor3::Ctor3>::build::<false>+0x1779>
   3c637:	mov    0x5520(%rsp),%rax
   3c63f:	mov    0x8(%rax),%rdi
   3c643:	call   *0xe04df(%rip)        # 11cb28 <free@GLIBC_2.2.5>
   3c649:	mov    0x5520(%rsp),%rdi
   3c651:	movq   $0x0,(%rdi)
   3c658:	movq   $0x40,0x8(%rdi)
   3c660:	movq   $0x0,0x10(%rdi)
   3c668:	mov    $0x80,%edx
   3c66d:	mov    %rbx,%rsi
   3c670:	call   86400 <<alloc::raw_vec::RawVecInner>::grow_exact>
   3c675:	cmp    $0xffffffffffffffff,%rax
   3c679:	mov    %r14,%r9
   3c67c:	mov    %r12,%rdi
   3c67f:	je     3b109 <<c30walk::ctor3::Ctor3>::build::<false>+0x239>
   3c685:	mov    %rax,%rdi
   3c688:	mov    %rdx,%rsi
   3c68b:	call   bfbab <alloc::raw_vec::handle_error>
   3c690:	lea    0xdda29(%rip),%rcx        # 11a0c0 <__frame_dummy_init_array_entry+0x3090>
   3c697:	mov    %r11,%rdi
   3c69a:	mov    %r13,%rdx
   3c69d:	call   c2420 <core::slice::index::slice_index_fail>
   3c6a2:	lea    0xdc6df(%rip),%rcx        # 118d88 <__frame_dummy_init_array_entry+0x1d58>
   3c6a9:	mov    %r11,%rdi
   3c6ac:	mov    %r15,%rsi
   3c6af:	mov    %r13,%rdx
   3c6b2:	call   c2420 <core::slice::index::slice_index_fail>
   3c6b7:	lea    0xdda02(%rip),%rcx        # 11a0c0 <__frame_dummy_init_array_entry+0x3090>
   3c6be:	mov    %rsi,%rdi
   3c6c1:	mov    %rax,%rsi
   3c6c4:	mov    %r13,%rdx
   3c6c7:	call   c2420 <core::slice::index::slice_index_fail>
   3c6cc:	lea    0xdc69d(%rip),%rcx        # 118d70 <__frame_dummy_init_array_entry+0x1d40>
   3c6d3:	mov    %r15,%rdi
   3c6d6:	mov    0x30(%rsp),%rsi
   3c6db:	mov    %r13,%rdx
   3c6de:	call   c2420 <core::slice::index::slice_index_fail>
   3c6e3:	lea    0xdab9e(%rip),%rdx        # 117288 <__frame_dummy_init_array_entry+0x258>
   3c6ea:	mov    %rax,%rdi
   3c6ed:	mov    (%rsp),%rsi
   3c6f1:	call   c1643 <core::panicking::panic_bounds_check>
   3c6f6:	lea    0xdc6eb(%rip),%rdx        # 118de8 <__frame_dummy_init_array_entry+0x1db8>
   3c6fd:	mov    %rax,%rdi
   3c700:	mov    %rcx,%rsi
   3c703:	call   c1643 <core::panicking::panic_bounds_check>
   3c708:	lea    0xdc6c1(%rip),%rdx        # 118dd0 <__frame_dummy_init_array_entry+0x1da0>
   3c70f:	mov    $0x1e,%esi
   3c714:	mov    %rbx,%rdi
   3c717:	call   c1643 <core::panicking::panic_bounds_check>
   3c71c:	lea    0xdc695(%rip),%rdx        # 118db8 <__frame_dummy_init_array_entry+0x1d88>
   3c723:	mov    $0x1e,%esi
   3c728:	mov    %rbx,%rdi
   3c72b:	call   c1643 <core::panicking::panic_bounds_check>
   3c730:	mov    $0x10,%edx
   3c735:	mov    $0x4,%ecx
   3c73a:	mov    $0x400,%r8d
   3c740:	mov    %rdi,%rbx
   3c743:	mov    %r12,%rdi
   3c746:	xor    %esi,%esi
   3c748:	call   60790 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c74d:	mov    %rbx,%rdi
   3c750:	jmp    3b195 <<c30walk::ctor3::Ctor3>::build::<false>+0x2c5>
   3c755:	mov    $0x8,%ecx
   3c75a:	mov    $0x8,%r8d
   3c760:	mov    %rdi,%r14
   3c763:	xor    %esi,%esi
   3c765:	mov    %rbx,%rdx
   3c768:	call   60790 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3c76d:	mov    %r14,%rdi
   3c770:	mov    0x10(%r14),%r14
   3c774:	mov    0x8(%rdi),%r12
   3c778:	lea    (%r12,%r14,8),%rdi
   3c77c:	cmp    $0x1,%rbx
   3c780:	jne    3b144 <<c30walk::ctor3::Ctor3>::build::<false>+0x274>
   3c786:	jmp    3b166 <<c30walk::ctor3::Ctor3>::build::<false>+0x296>
   3c78b:	lea    0xdaaf6(%rip),%rdx        # 117288 <__frame_dummy_init_array_entry+0x258>
   3c792:	call   c1643 <core::panicking::panic_bounds_check>
   3c797:	lea    0xdc5a2(%rip),%rdx        # 118d40 <__frame_dummy_init_array_entry+0x1d10>
   3c79e:	mov    $0x1e,%esi
   3c7a3:	call   c1643 <core::panicking::panic_bounds_check>
   3c7a8:	lea    0xdc5f1(%rip),%rdx        # 118da0 <__frame_dummy_init_array_entry+0x1d70>
   3c7af:	mov    $0x1e,%esi
   3c7b4:	call   c1643 <core::panicking::panic_bounds_check>
   3c7b9:	lea    0xdc4d8(%rip),%rdx        # 118c98 <__frame_dummy_init_array_entry+0x1c68>
   3c7c0:	call   c1643 <core::panicking::panic_bounds_check>
   3c7c5:	lea    0xdaabc(%rip),%rdx        # 117288 <__frame_dummy_init_array_entry+0x258>
   3c7cc:	mov    %rbp,%rdi
   3c7cf:	call   c1643 <core::panicking::panic_bounds_check>
