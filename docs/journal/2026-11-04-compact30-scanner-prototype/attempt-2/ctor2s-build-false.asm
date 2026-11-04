# <c30walk::ctor2s::Ctor2s>::build::<false>
# size 0x177d

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000040f00 <<c30walk::ctor2s::Ctor2s>::build::<false>>:
   40f00:	push   %rbp
   40f01:	push   %r15
   40f03:	push   %r14
   40f05:	push   %r13
   40f07:	push   %r12
   40f09:	push   %rbx
   40f0a:	sub    $0x1000,%rsp
   40f11:	movq   $0x0,(%rsp)
   40f19:	sub    $0x1000,%rsp
   40f20:	movq   $0x0,(%rsp)
   40f28:	sub    $0x1000,%rsp
   40f2f:	movq   $0x0,(%rsp)
   40f37:	sub    $0x1000,%rsp
   40f3e:	movq   $0x0,(%rsp)
   40f46:	sub    $0x1000,%rsp
   40f4d:	movq   $0x0,(%rsp)
   40f55:	sub    $0x4f8,%rsp
   40f5c:	mov    %r8d,0x14(%rsp)
   40f61:	mov    %rsi,0x90(%rsp)
   40f69:	mov    0x5530(%rsp),%rax
   40f71:	movq   $0x0,0x10(%r9)
   40f79:	movq   $0x0,0x10(%rax)
   40f81:	mov    %rdx,0x8(%rsp)
   40f86:	cmp    $0xeffe2,%rdx
   40f8d:	ja     4237d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x147d>
   40f93:	mov    %rcx,%rbx
   40f96:	pxor   %xmm0,%xmm0
   40f9a:	xor    %eax,%eax
   40f9c:	pxor   %xmm2,%xmm2
   40fa0:	pxor   %xmm1,%xmm1
   40fa4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   40fb0:	movq   (%rbx,%rax,4),%xmm3
   40fb5:	movq   0x8(%rbx,%rax,4),%xmm4
   40fbb:	punpckldq %xmm0,%xmm3
   40fbf:	paddq  %xmm2,%xmm3
   40fc3:	punpckldq %xmm0,%xmm4
   40fc7:	paddq  %xmm1,%xmm4
   40fcb:	movq   0x10(%rbx,%rax,4),%xmm2
   40fd1:	movq   0x18(%rbx,%rax,4),%xmm1
   40fd7:	punpckldq %xmm0,%xmm2
   40fdb:	paddq  %xmm3,%xmm2
   40fdf:	punpckldq %xmm0,%xmm1
   40fe3:	paddq  %xmm4,%xmm1
   40fe7:	add    $0x8,%rax
   40feb:	cmp    $0x100,%rax
   40ff1:	jne    40fb0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb0>
   40ff3:	paddq  %xmm2,%xmm1
   40ff7:	pshufd $0xee,%xmm1,%xmm0
   40ffc:	paddq  %xmm1,%xmm0
   41000:	movq   %xmm0,%rax
   41005:	cmp    0x8(%rsp),%rax
   4100a:	jne    4237d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x147d>
   41010:	pxor   %xmm2,%xmm2
   41014:	xor    %eax,%eax
   41016:	movdqa -0x319fe(%rip),%xmm3        # f620 <GCC_except_table971+0xa28>
   4101e:	pxor   %xmm1,%xmm1
   41022:	pxor   %xmm0,%xmm0
   41026:	cs nopw 0x0(%rax,%rax,1)
   41030:	movq   (%rbx,%rax,4),%xmm4
   41035:	movq   0x8(%rbx,%rax,4),%xmm5
   4103b:	pshufd $0xd4,%xmm4,%xmm4
   41040:	pcmpeqd %xmm2,%xmm4
   41044:	pandn  %xmm3,%xmm4
   41048:	paddq  %xmm1,%xmm4
   4104c:	pshufd $0xd4,%xmm5,%xmm5
   41051:	pcmpeqd %xmm2,%xmm5
   41055:	pandn  %xmm3,%xmm5
   41059:	paddq  %xmm0,%xmm5
   4105d:	movq   0x10(%rbx,%rax,4),%xmm0
   41063:	movq   0x18(%rbx,%rax,4),%xmm6
   41069:	pshufd $0xd4,%xmm0,%xmm1
   4106e:	pcmpeqd %xmm2,%xmm1
   41072:	pandn  %xmm3,%xmm1
   41076:	paddq  %xmm4,%xmm1
   4107a:	pshufd $0xd4,%xmm6,%xmm0
   4107f:	pcmpeqd %xmm2,%xmm0
   41083:	pandn  %xmm3,%xmm0
   41087:	paddq  %xmm5,%xmm0
   4108b:	add    $0x8,%rax
   4108f:	cmp    $0x100,%rax
   41095:	jne    41030 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x130>
   41097:	mov    $0x88888889,%r13d
   4109d:	mov    0x8(%rsp),%rax
   410a2:	imul   %rax,%r13
   410a6:	shr    $0x24,%r13
   410aa:	imul   $0xeeeeeeef,%eax,%eax
   410b0:	ror    $1,%eax
   410b2:	cmp    $0x8888889,%eax
   410b7:	sbb    $0xffffffffffffffff,%r13
   410bb:	mov    %r13,%r14
   410be:	cmpl   $0x0,0x14(%rsp)
   410c3:	jne    4111f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x21f>
   410c5:	mov    0x8(%rsp),%rsi
   410ca:	test   %rsi,%rsi
   410cd:	je     4111c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x21c>
   410cf:	paddq  %xmm1,%xmm0
   410d3:	pshufd $0xee,%xmm0,%xmm1
   410d8:	paddq  %xmm0,%xmm1
   410dc:	movq   %xmm1,%rax
   410e1:	lea    -0x1(%rsi),%ecx
   410e4:	mov    $0x88888889,%edx
   410e9:	imul   %rcx,%rdx
   410ed:	shr    $0x23,%rdx
   410f1:	and    $0xfffffffe,%edx
   410f4:	lea    (%rdx,%rsi,1),%ecx
   410f7:	dec    %ecx
   410f9:	shr    $1,%ecx
   410fb:	mov    $0x80010003,%r14d
   41101:	imul   %rcx,%r14
   41105:	shr    $0x2e,%r14
   41109:	inc    %r14d
   4110c:	imul   %rax,%r14
   41110:	dec    %r14
   41113:	cmp    %r13,%r14
   41116:	cmovae %r13,%r14
   4111a:	jmp    4111f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x21f>
   4111c:	xor    %r14d,%r14d
   4111f:	mov    (%r9),%rax
   41122:	lea    0x1(%r13),%rcx
   41126:	mov    %rcx,0x78(%rsp)
   4112b:	cmp    %r13,%rax
   4112e:	jbe    424d7 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15d7>
   41134:	mov    0x5530(%rsp),%rax
   4113c:	mov    (%rax),%rax
   4113f:	cmp    %r14,%rax
   41142:	jb     4252c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162c>
   41148:	movq   $0x0,0x10(%rdi)
   41150:	lea    0x3f(%r13),%r14
   41154:	shr    $0x6,%r14
   41158:	mov    %rdi,0x50(%rsp)
   4115d:	mov    %r13,0x68(%rsp)
   41162:	je     411bd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2bd>
   41164:	cmp    (%rdi),%r14
   41167:	ja     42609 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1709>
   4116d:	xor    %r12d,%r12d
   41170:	mov    %rdi,%rcx
   41173:	mov    0x8(%rdi),%r15
   41177:	lea    (%r15,%r12,8),%rdi
   4117b:	cmp    $0x1,%r14
   4117f:	je     411ae <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2ae>
   41181:	lea    -0x8(,%r14,8),%rdx
   41189:	xor    %esi,%esi
   4118b:	mov    %r9,%r13
   4118e:	call   *0xf2ffc(%rip)        # 134190 <memset@GLIBC_2.2.5>
   41194:	mov    %r13,%r9
   41197:	mov    0x50(%rsp),%rcx
   4119c:	lea    (%r12,%r14,1),%rax
   411a0:	lea    (%r15,%rax,8),%rdi
   411a4:	add    $0xfffffffffffffff8,%rdi
   411a8:	add    %r14,%r12
   411ab:	dec    %r12
   411ae:	movq   $0x0,(%rdi)
   411b5:	inc    %r12
   411b8:	mov    %rcx,%rdi
   411bb:	jmp    411c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2c0>
   411bd:	xor    %r12d,%r12d
   411c0:	lea    0x18(%rdi),%r15
   411c4:	mov    %r12,0x10(%rdi)
   411c8:	movq   $0x0,0x28(%rdi)
   411d0:	cmpq   $0xf,0x18(%rdi)
   411d5:	jbe    425de <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16de>
   411db:	lea    0x30(%rdi),%r14
   411df:	xor    %eax,%eax
   411e1:	pxor   %xmm0,%xmm0
   411e5:	movabs $0xffffffff00000000,%rcx
   411ef:	nop
   411f0:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   411f9:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   41202:	mov    %rcx,0x2d18(%rsp,%rax,1)
   4120a:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   41213:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   4121c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   41224:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   4122d:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   41236:	mov    %rcx,0x2d68(%rsp,%rax,1)
   4123e:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   41247:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   41250:	mov    %rcx,0x2d90(%rsp,%rax,1)
   41258:	add    $0xa0,%rax
   4125e:	cmp    $0x2800,%rax
   41264:	jne    411f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2f0>
   41266:	lea    0xc(%rbx),%rax
   4126a:	xor    %edx,%edx
   4126c:	mov    $0x25,%ecx
   41271:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41280:	mov    -0xc(%rax),%esi
   41283:	add    %edx,%esi
   41285:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   4128c:	mov    %esi,0x2c80(%rsp,%rcx,4)
   41293:	mov    -0x8(%rax),%edx
   41296:	add    %esi,%edx
   41298:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   4129f:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   412a6:	mov    -0x4(%rax),%esi
   412a9:	add    %edx,%esi
   412ab:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   412b2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   412b9:	mov    (%rax),%edx
   412bb:	add    %esi,%edx
   412bd:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   412c4:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   412cb:	add    $0x28,%rcx
   412cf:	add    $0x10,%rax
   412d3:	cmp    $0xa25,%rcx
   412da:	jne    41280 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x380>
   412dc:	mov    %r9,0x70(%rsp)
   412e1:	mov    0x8(%r9),%r13
   412e5:	mov    0x50(%rsp),%rax
   412ea:	mov    0x8(%rax),%rbp
   412ee:	mov    0x10(%rax),%r12
   412f2:	lea    0xe8(%rsp),%rdi
   412fa:	lea    0x2cf8(%rsp),%rsi
   41302:	mov    $0x2800,%edx
   41307:	call   *0xf2ea3(%rip)        # 1341b0 <memcpy@GLIBC_2.14>
   4130d:	mov    %rbp,%r9
   41310:	mov    %r12,%rsi
   41313:	mov    %r13,0xd0(%rsp)
   4131b:	mov    0x68(%rsp),%rax
   41320:	mov    %rax,0xd8(%rsp)
   41328:	mov    0x8(%rsp),%r11
   4132d:	mov    %r11,0xe0(%rsp)
   41335:	mov    0x5530(%rsp),%rax
   4133d:	mov    %rax,0xa8(%rsp)
   41345:	mov    %rbp,0xb0(%rsp)
   4134d:	mov    %r12,0xb8(%rsp)
   41355:	mov    %r15,0xc0(%rsp)
   4135d:	movl   $0x0,0x28f0(%rsp)
   41368:	movq   $0x0,0x28e8(%rsp)
   41374:	mov    %r14,0xc8(%rsp)
   4137c:	mov    $0xffffffffffffffff,%rcx
   41383:	xor    %r15d,%r15d
   41386:	movabs $0xeeeeeeeeeeeeeeef,%r10
   41390:	movabs $0x888888888888889,%rdi
   4139a:	pxor   %xmm0,%xmm0
   4139e:	pcmpeqd %xmm1,%xmm1
   413a2:	xor    %r14d,%r14d
   413a5:	jmp    413c1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4c1>
   413a7:	nopw   0x0(%rax,%rax,1)
   413b0:	add    $0x4,%r14
   413b4:	cmp    $0x400,%r14
   413bb:	je     414fb <<c30walk::ctor2s::Ctor2s>::build::<false>+0x5fb>
   413c1:	mov    %r15,%rax
   413c4:	mov    (%rbx,%r14,1),%r15d
   413c8:	add    %rax,%r15
   413cb:	cmp    %r11,%r15
   413ce:	setae  %r8b
   413d2:	mov    %r15,%rax
   413d5:	movabs $0x8888888888888889,%rdx
   413df:	mul    %rdx
   413e2:	mov    %rdx,%r12
   413e5:	shr    $0x4,%r12
   413e9:	mov    %r15,%rax
   413ec:	imul   %r10,%rax
   413f0:	ror    $1,%rax
   413f3:	cmp    %rdi,%rax
   413f6:	setb   %al
   413f9:	cmp    %rcx,%r12
   413fc:	sete   %dl
   413ff:	or     %r8b,%dl
   41402:	or     %al,%dl
   41404:	jne    413b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4b0>
   41406:	mov    0x5530(%rsp),%rdi
   4140e:	mov    0x10(%rdi),%rbp
   41412:	cmp    (%rdi),%rbp
   41415:	je     414c3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x5c3>
   4141b:	mov    0x8(%rdi),%rax
   4141f:	mov    %rbp,%rcx
   41422:	shl    $0x7,%rcx
   41426:	movdqa %xmm0,0x70(%rax,%rcx,1)
   4142c:	movdqa %xmm0,0x60(%rax,%rcx,1)
   41432:	movdqa %xmm0,0x50(%rax,%rcx,1)
   41438:	movdqa %xmm0,0x40(%rax,%rcx,1)
   4143e:	movdqa %xmm0,0x30(%rax,%rcx,1)
   41444:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4144a:	movdqa %xmm0,0x10(%rax,%rcx,1)
   41450:	movdqa %xmm0,(%rax,%rcx,1)
   41455:	mov    %rbp,%rax
   41458:	inc    %rax
   4145b:	mov    %rax,0x10(%rdi)
   4145f:	mov    %r12,%rax
   41462:	shl    $0x6,%rax
   41466:	or     $0x10000000,%ebp
   4146c:	movdqu %xmm1,0x2c(%r13,%rax,1)
   41473:	movdqa %xmm1,0x20(%r13,%rax,1)
   4147a:	movdqa %xmm1,0x10(%r13,%rax,1)
   41481:	movdqa %xmm1,0x0(%r13,%rax,1)
   41488:	mov    %ebp,0x3c(%r13,%rax,1)
   4148d:	mov    %r12,%rdi
   41490:	shr    $0x6,%rdi
   41494:	cmp    %rsi,%rdi
   41497:	jae    42662 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1762>
   4149d:	mov    $0x1,%eax
   414a2:	mov    %r12d,%ecx
   414a5:	shl    %cl,%rax
   414a8:	or     %rax,(%r9,%rdi,8)
   414ac:	mov    %r12,%rcx
   414af:	mov    0x8(%rsp),%r11
   414b4:	movabs $0x888888888888889,%rdi
   414be:	jmp    413b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4b0>
   414c3:	mov    %rsi,0x20(%rsp)
   414c8:	mov    %r9,0x40(%rsp)
   414cd:	call   90d20 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   414d2:	pcmpeqd %xmm1,%xmm1
   414d6:	pxor   %xmm0,%xmm0
   414da:	movabs $0xeeeeeeeeeeeeeeef,%r10
   414e4:	mov    0x40(%rsp),%r9
   414e9:	mov    0x20(%rsp),%rsi
   414ee:	mov    0x5530(%rsp),%rdi
   414f6:	jmp    4141b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x51b>
   414fb:	cmpl   $0x0,0x14(%rsp)
   41500:	je     416b3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7b3>
   41506:	mov    0xd8(%rsp),%r8
   4150e:	test   %r8,%r8
   41511:	je     416b3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7b3>
   41517:	mov    %r11,%r9
   4151a:	shl    $0x20,%r9
   4151e:	mov    0x14(%rsp),%r10d
   41523:	mov    0xa8(%rsp),%rdi
   4152b:	mov    0xd0(%rsp),%rbp
   41533:	xor    %r14d,%r14d
   41536:	movabs $0xff51afd7ed558ccd,%r12
   41540:	pxor   %xmm0,%xmm0
   41544:	pcmpeqd %xmm1,%xmm1
   41548:	jmp    415d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6d0>
   4154d:	mov    0x8(%rdi),%rax
   41551:	mov    %r12,%rcx
   41554:	shl    $0x7,%rcx
   41558:	movdqa %xmm0,0x70(%rax,%rcx,1)
   4155e:	movdqa %xmm0,0x60(%rax,%rcx,1)
   41564:	movdqa %xmm0,0x50(%rax,%rcx,1)
   4156a:	movdqa %xmm0,0x40(%rax,%rcx,1)
   41570:	movdqa %xmm0,0x30(%rax,%rcx,1)
   41576:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4157c:	movdqa %xmm0,0x10(%rax,%rcx,1)
   41582:	movdqa %xmm0,(%rax,%rcx,1)
   41587:	mov    %r12,%rax
   4158a:	inc    %rax
   4158d:	mov    %rax,0x10(%rdi)
   41591:	or     $0x10000000,%r12d
   41598:	movdqu %xmm1,0x2c(%rbp)
   4159d:	movdqa %xmm1,0x20(%rbp)
   415a2:	movdqa %xmm1,0x10(%rbp)
   415a7:	movdqa %xmm1,0x0(%rbp)
   415ac:	mov    %r12d,0x3c(%rbp)
   415b0:	or     %rbx,0x0(%r13,%r15,8)
   415b5:	movabs $0xff51afd7ed558ccd,%r12
   415bf:	nop
   415c0:	inc    %r14
   415c3:	add    $0x40,%rbp
   415c7:	cmp    %r14,%r8
   415ca:	je     416b3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7b3>
   415d0:	mov    %r14,%r15
   415d3:	shr    $0x6,%r15
   415d7:	mov    0xb8(%rsp),%rsi
   415df:	cmp    %rsi,%r15
   415e2:	jae    4266e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176e>
   415e8:	mov    0xb0(%rsp),%r13
   415f0:	mov    0x0(%r13,%r15,8),%rax
   415f5:	mov    $0x1,%ebx
   415fa:	mov    %r14d,%ecx
   415fd:	shl    %cl,%rbx
   41600:	bt     %r14,%rax
   41604:	jb     415c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6c0>
   41606:	mov    %r14,%rax
   41609:	xor    %r9,%rax
   4160c:	mov    %rax,%rcx
   4160f:	shr    $0x21,%rcx
   41613:	xor    %rax,%rcx
   41616:	imul   %r12,%rcx
   4161a:	mov    %rcx,%rax
   4161d:	shr    $0x21,%rax
   41621:	xor    %rcx,%rax
   41624:	movabs $0xc4ceb9fe1a85ec53,%rcx
   4162e:	imul   %rcx,%rax
   41632:	mov    %rax,%rcx
   41635:	shr    $0x21,%rcx
   41639:	xor    %rax,%rcx
   4163c:	mov    %rcx,%rax
   4163f:	shr    $0x3,%rax
   41643:	movabs $0x20c49ba5e353f7cf,%rdx
   4164d:	mul    %rdx
   41650:	shr    $0x4,%rdx
   41654:	imul   $0x3e8,%rdx,%rax
   4165b:	sub    %rax,%rcx
   4165e:	cmp    %r10,%rcx
   41661:	jae    415c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6c0>
   41667:	mov    0x10(%rdi),%r12
   4166b:	cmp    (%rdi),%r12
   4166e:	jne    4154d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x64d>
   41674:	mov    %r8,0x20(%rsp)
   41679:	mov    %r9,0x40(%rsp)
   4167e:	mov    %r10,0x38(%rsp)
   41683:	mov    %rdi,0x60(%rsp)
   41688:	call   90d20 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4168d:	pcmpeqd %xmm1,%xmm1
   41691:	pxor   %xmm0,%xmm0
   41695:	mov    0x60(%rsp),%rdi
   4169a:	mov    0x38(%rsp),%r10
   4169f:	mov    0x40(%rsp),%r9
   416a4:	mov    0x20(%rsp),%r8
   416a9:	mov    0x8(%rsp),%r11
   416ae:	jmp    4154d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x64d>
   416b3:	test   %r11,%r11
   416b6:	je     42313 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1413>
   416bc:	xor    %ebx,%ebx
   416be:	lea    0x28f8(%rsp),%r12
   416c6:	jmp    41718 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x818>
   416c8:	nopl   0x0(%rax,%rax,1)
   416d0:	lea    0x1(%rax),%rdx
   416d4:	add    $0xffffffffffffffe3,%rax
   416d8:	xor    %esi,%esi
   416da:	cmp    $0x1e,%rdx
   416de:	setae  %sil
   416e2:	cmovb  %rdx,%rax
   416e6:	lea    (%rcx,%rsi,2),%ecx
   416e9:	inc    %ecx
   416eb:	mov    %rax,0x28e8(%rsp)
   416f3:	mov    %ecx,0x28f0(%rsp)
   416fa:	cmp    %rdi,0x80(%rsp)
   41702:	lea    0x28f8(%rsp),%r12
   4170a:	mov    0x88(%rsp),%rbx
   41712:	jae    42313 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1413>
   41718:	mov    $0x400,%edx
   4171d:	mov    %r12,%rdi
   41720:	xor    %esi,%esi
   41722:	call   *0xf2a68(%rip)        # 134190 <memset@GLIBC_2.2.5>
   41728:	lea    0x128(%rsp),%rax
   41730:	xor    %ecx,%ecx
   41732:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41740:	mov    -0x40(%rax),%rdx
   41744:	sub    -0x30(%rax),%rdx
   41748:	shr    $1,%rdx
   4174b:	add    -0x28(%rax),%edx
   4174e:	mov    -0x18(%rax),%rsi
   41752:	sub    -0x8(%rax),%rsi
   41756:	mov    %edx,0x28f8(%rsp,%rcx,4)
   4175d:	shr    $1,%rsi
   41760:	add    (%rax),%esi
   41762:	mov    %esi,0x28fc(%rsp,%rcx,4)
   41769:	add    $0x2,%rcx
   4176d:	add    $0x50,%rax
   41771:	cmp    $0x100,%rcx
   41778:	jne    41740 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x840>
   4177a:	mov    0xc0(%rsp),%r14
   41782:	mov    0x10(%r14),%r15
   41786:	cmp    (%r14),%r15
   41789:	je     42306 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1406>
   4178f:	mov    %r15,%rdi
   41792:	shl    $0xa,%rdi
   41796:	add    0x8(%r14),%rdi
   4179a:	mov    $0x400,%edx
   4179f:	mov    %r12,%rsi
   417a2:	call   *0xf2a08(%rip)        # 1341b0 <memcpy@GLIBC_2.14>
   417a8:	inc    %r15
   417ab:	mov    %r15,0x10(%r14)
   417af:	lea    0xf000(%rbx),%rcx
   417b6:	mov    0x8(%rsp),%rdi
   417bb:	cmp    %rdi,%rcx
   417be:	mov    %rdi,%r14
   417c1:	mov    %rcx,0x80(%rsp)
   417c9:	cmovb  %rcx,%r14
   417cd:	mov    %r14,0x88(%rsp)
   417d5:	sub    %rbx,%r14
   417d8:	add    0x90(%rsp),%rbx
   417e0:	cmp    $0x10,%r14
   417e4:	lea    0xe8(%rsp),%r13
   417ec:	jb     42160 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1260>
   417f2:	xor    %edx,%edx
   417f4:	mov    0x18(%rsp),%rsi
   417f9:	jmp    41855 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x955>
   417fb:	nopl   0x0(%rax,%rax,1)
   41800:	mov    0x28e8(%rsp),%rax
   41808:	lea    0x10(%rax),%rcx
   4180c:	add    $0xfffffffffffffff2,%rax
   41810:	xor    %edx,%edx
   41812:	cmp    $0x1e,%rcx
   41816:	setae  %dl
   41819:	cmovb  %rcx,%rax
   4181d:	mov    0x28f0(%rsp),%ecx
   41824:	lea    (%rcx,%rdx,2),%ecx
   41827:	add    $0x10,%ecx
   4182a:	mov    %rax,0x28e8(%rsp)
   41832:	mov    %ecx,0x28f0(%rsp)
   41839:	xor    %edx,%edx
   4183b:	mov    0x30(%rsp),%rcx
   41840:	mov    %rcx,%r14
   41843:	mov    0x28(%rsp),%rbp
   41848:	mov    %rbp,%rbx
   4184b:	cmp    $0xf,%rcx
   4184f:	jbe    4216b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x126b>
   41855:	mov    0x58(%rsp),%rcx
   4185a:	test   $0x1,%dl
   4185d:	jne    41866 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x966>
   4185f:	mov    (%rbx),%rcx
   41862:	mov    0x8(%rbx),%rsi
   41866:	lea    0x10(%rbx),%r8
   4186a:	lea    -0x10(%r14),%rdx
   4186e:	xor    %rcx,%rsi
   41871:	mov    %rcx,%rax
   41874:	rol    $0x8,%rax
   41878:	xor    %rcx,%rax
   4187b:	or     %rsi,%rax
   4187e:	jne    418d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x9d0>
   41880:	cmp    $0x10,%rdx
   41884:	jb     41970 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa70>
   4188a:	nopw   0x0(%rax,%rax,1)
   41890:	mov    (%r8),%rsi
   41893:	mov    0x8(%r8),%rax
   41897:	mov    $0x1,%r9b
   4189a:	cmp    %rcx,%rsi
   4189d:	jne    41a02 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb02>
   418a3:	cmp    %rcx,%rax
   418a6:	jne    41a02 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb02>
   418ac:	add    $0xfffffffffffffff0,%rdx
   418b0:	add    $0x10,%r8
   418b4:	cmp    $0xf,%rdx
   418b8:	ja     41890 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x990>
   418ba:	mov    %r8,0x28(%rsp)
   418bf:	xor    %r9d,%r9d
   418c2:	mov    %rcx,0x58(%rsp)
   418c7:	jmp    41a11 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb11>
   418cc:	nopl   0x0(%rax)
   418d0:	mov    %r8,0x28(%rsp)
   418d5:	mov    %rdx,0x30(%rsp)
   418da:	mov    0x28e8(%rsp),%rax
   418e2:	lea    -0x1e(%rax),%r12
   418e6:	neg    %r12
   418e9:	cmp    $0x10,%r12
   418ed:	mov    $0x10,%r14d
   418f3:	cmovb  %r12,%r14
   418f7:	mov    0x28f0(%rsp),%r15d
   418ff:	cmp    $0x1e,%rax
   41903:	je     41992 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa92>
   41909:	xor    %r13d,%r13d
   4190c:	nopl   0x0(%rax)
   41910:	movzbl (%rbx,%r13,1),%esi
   41915:	lea    (%rsi,%rsi,4),%rcx
   41919:	lea    0xe8(%rsp),%rdx
   41921:	lea    (%rdx,%rcx,8),%rbp
   41925:	mov    (%rdx,%rcx,8),%rax
   41929:	cmp    0x8(%rdx,%rcx,8),%rax
   4192e:	je     41946 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa46>
   41930:	lea    (%r15,%r13,1),%rcx
   41934:	mov    %cx,(%rax)
   41937:	addq   $0x2,0x0(%rbp)
   4193c:	inc    %r13
   4193f:	cmp    %r13,%r14
   41942:	jne    41910 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa10>
   41944:	jmp    41980 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa80>
   41946:	lea    0xa8(%rsp),%rdi
   4194e:	call   8bae0 <<c30walk::ctor2s::W<false>>::refill>
   41953:	test   %al,%al
   41955:	je     4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   4195b:	mov    0x0(%rbp),%rax
   4195f:	mov    0x8(%rsp),%rdi
   41964:	jmp    41930 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa30>
   41966:	cs nopw 0x0(%rax,%rax,1)
   41970:	mov    %r8,0x28(%rsp)
   41975:	xor    %r9d,%r9d
   41978:	jmp    41a11 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb11>
   4197d:	nopl   (%rax)
   41980:	cmp    $0xf,%r12
   41984:	lea    0xe8(%rsp),%r13
   4198c:	ja     41800 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x900>
   41992:	add    $0x2,%r15d
   41996:	movzwl %r15w,%r15d
   4199a:	nopw   0x0(%rax,%rax,1)
   419a0:	movzbl (%rbx,%r14,1),%esi
   419a5:	lea    (%rsi,%rsi,4),%rcx
   419a9:	lea    0x0(,%rcx,8),%r12
   419b1:	add    %r13,%r12
   419b4:	mov    0x0(%r13,%rcx,8),%rax
   419b9:	cmp    0x8(%r13,%rcx,8),%rax
   419be:	je     419da <<c30walk::ctor2s::Ctor2s>::build::<false>+0xada>
   419c0:	lea    (%r15,%r14,1),%rcx
   419c4:	inc    %r14
   419c7:	mov    %cx,(%rax)
   419ca:	addq   $0x2,(%r12)
   419cf:	cmp    $0x10,%r14
   419d3:	jne    419a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xaa0>
   419d5:	jmp    41800 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x900>
   419da:	lea    0xa8(%rsp),%rdi
   419e2:	call   8bae0 <<c30walk::ctor2s::W<false>>::refill>
   419e7:	test   %al,%al
   419e9:	je     4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   419ef:	mov    (%r12),%rax
   419f3:	mov    0x8(%rsp),%rdi
   419f8:	lea    0xe8(%rsp),%r13
   41a00:	jmp    419c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xac0>
   41a02:	mov    %r8,0x28(%rsp)
   41a07:	mov    %rsi,0x58(%rsp)
   41a0c:	mov    %rax,0x18(%rsp)
   41a11:	mov    %rdx,0x30(%rsp)
   41a16:	sub    %rdx,%r14
   41a19:	movzbl %cl,%eax
   41a1c:	mov    %rax,0x98(%rsp)
   41a24:	lea    (%rax,%rax,4),%rdx
   41a28:	mov    0x0(%r13,%rdx,8),%rax
   41a2d:	mov    0x1c(%r13,%rdx,8),%esi
   41a32:	mov    %rax,%rdi
   41a35:	sub    0x10(%r13,%rdx,8),%rdi
   41a3a:	shr    $1,%rdi
   41a3d:	add    0x18(%r13,%rdx,8),%edi
   41a42:	add    %r14,%rdi
   41a45:	cmp    %rsi,%rdi
   41a48:	ja     4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   41a4e:	mov    %r9d,0x4c(%rsp)
   41a53:	lea    0xe8(%rsp),%r13
   41a5b:	lea    0x0(,%rdx,8),%r10
   41a63:	add    %r13,%r10
   41a66:	movzbl %cl,%ebx
   41a69:	mov    0x8(%r10),%rsi
   41a6d:	mov    $0x88888889,%r15d
   41a73:	lea    -0x267ba(%rip),%r12        # 1b2c0 <c30walk::ctor2s::PATTERNS>
   41a7a:	mov    %r10,0x38(%rsp)
   41a7f:	jmp    41ac0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xbc0>
   41a81:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41a90:	mov    0x3c(%rsi),%edi
   41a93:	mov    $0xfffff,%r9d
   41a99:	and    %r9d,%edi
   41a9c:	sub    %r8d,%ecx
   41a9f:	sub    %eax,%r8d
   41aa2:	mov    %r8d,%eax
   41aa5:	lea    (%rsi,%rax,2),%rax
   41aa9:	mov    %rax,0x10(%r10)
   41aad:	mov    %rax,(%r10)
   41ab0:	lea    (%rax,%rcx,2),%rsi
   41ab4:	mov    %rsi,0x8(%r10)
   41ab8:	mov    %edx,0x20(%r10)
   41abc:	mov    %edi,0x24(%r10)
   41ac0:	sub    %rax,%rsi
   41ac3:	shr    $1,%rsi
   41ac6:	cmp    %rsi,%r14
   41ac9:	cmovb  %r14,%rsi
   41acd:	test   %rsi,%rsi
   41ad0:	je     41b8d <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc8d>
   41ad6:	mov    0x28e8(%rsp),%rdi
   41ade:	cmp    $0x1d,%rdi
   41ae2:	ja     42640 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1740>
   41ae8:	cmp    $0x1f,%rsi
   41aec:	jae    42391 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1491>
   41af2:	mov    0x28f0(%rsp),%ecx
   41af9:	mov    %rdi,%r11
   41afc:	shl    $0x6,%r11
   41b00:	lea    (%r12,%r11,1),%rdx
   41b04:	lea    -0x2(,%rsi,2),%r9
   41b0c:	cmp    $0xe,%r9
   41b10:	jb     41b25 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc25>
   41b12:	mov    %rax,%r8
   41b15:	sub    %r12,%r8
   41b18:	sub    %r11,%r8
   41b1b:	cmp    $0x10,%r8
   41b1f:	jae    41f58 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1058>
   41b25:	mov    %rdx,%r8
   41b28:	xor    %r10d,%r10d
   41b2b:	lea    (%rdx,%rsi,2),%rdx
   41b2f:	lea    (%rax,%r10,2),%r9
   41b33:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41b40:	movzwl (%r8),%r10d
   41b44:	add    %cx,%r10w
   41b48:	add    $0x2,%r8
   41b4c:	mov    %r10w,(%r9)
   41b50:	add    $0x2,%r9
   41b54:	cmp    %rdx,%r8
   41b57:	jne    41b40 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc40>
   41b59:	lea    (%rax,%rsi,2),%rax
   41b5d:	mov    0x38(%rsp),%r10
   41b62:	mov    %rax,(%r10)
   41b65:	add    %rsi,%rdi
   41b68:	lea    0x2(%rsi),%edx
   41b6b:	mov    %rdi,%r8
   41b6e:	sub    $0x1e,%r8
   41b72:	cmovb  %rdi,%r8
   41b76:	cmovb  %esi,%edx
   41b79:	mov    %r8,0x28e8(%rsp)
   41b81:	add    %ecx,%edx
   41b83:	mov    %edx,0x28f0(%rsp)
   41b8a:	sub    %rsi,%r14
   41b8d:	test   %r14,%r14
   41b90:	je     42140 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1240>
   41b96:	mov    %rax,%r8
   41b99:	cmp    0x10(%r10),%rax
   41b9d:	je     41bc0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcc0>
   41b9f:	lea    0xa8(%rsp),%rdi
   41ba7:	mov    0x98(%rsp),%rsi
   41baf:	call   8b4e0 <<c30walk::ctor2s::W<false>>::finish>
   41bb4:	mov    0x38(%rsp),%r10
   41bb9:	mov    (%r10),%r8
   41bbc:	mov    0x10(%r10),%rax
   41bc0:	sub    %rax,%r8
   41bc3:	shr    $1,%r8
   41bc6:	add    0x18(%r10),%r8d
   41bca:	mov    %r8d,0x18(%r10)
   41bce:	pxor   %xmm0,%xmm0
   41bd2:	movdqu %xmm0,(%r10)
   41bd7:	movq   $0x0,0x10(%r10)
   41bdf:	cmp    $0x1e,%r14
   41be3:	jb     420d8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11d8>
   41be9:	mov    %r8,%rcx
   41bec:	imul   %r15,%rcx
   41bf0:	shr    $0x24,%rcx
   41bf4:	mov    %ecx,%eax
   41bf6:	shl    $0x5,%eax
   41bf9:	lea    (%rcx,%rcx,1),%edx
   41bfc:	sub    %eax,%edx
   41bfe:	add    %r8d,%edx
   41c01:	jne    420d8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11d8>
   41c07:	mov    %r14,%rax
   41c0a:	movabs $0x8888888888888889,%rdx
   41c14:	mul    %rdx
   41c17:	shr    $0x4,%rdx
   41c1b:	mov    %ecx,%r9d
   41c1e:	cmpl   $0x0,0x14(%rsp)
   41c23:	mov    %r14,0x60(%rsp)
   41c28:	mov    %rdx,0xa0(%rsp)
   41c30:	je     41f11 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1011>
   41c36:	mov    0x28f0(%rsp),%r10d
   41c3e:	mov    0xb8(%rsp),%rax
   41c46:	mov    %rax,0x20(%rsp)
   41c4b:	mov    0xa8(%rsp),%r11
   41c53:	mov    0xb0(%rsp),%rax
   41c5b:	mov    %rax,0x40(%rsp)
   41c60:	mov    0xd0(%rsp),%rax
   41c68:	mov    0x28e8(%rsp),%r14
   41c70:	mov    %r14,%rdi
   41c73:	shl    $0x6,%rdi
   41c77:	add    %r12,%rdi
   41c7a:	mov    %r9,%rcx
   41c7d:	shl    $0x6,%rcx
   41c81:	lea    (%rax,%rcx,1),%r13
   41c85:	add    $0x3c,%r13
   41c89:	mov    %r10d,%r15d
   41c8c:	shl    $0x8,%r15d
   41c90:	or     %ebx,%r15d
   41c93:	xor    %r12d,%r12d
   41c96:	jmp    41e87 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xf87>
   41c9b:	nopl   0x0(%rax,%rax,1)
   41ca0:	mov    0x8(%r11),%rcx
   41ca4:	shl    $0x7,%eax
   41ca7:	mov    %r15d,(%rcx,%rax,1)
   41cab:	movzwl 0x2(%rdi),%esi
   41caf:	lea    (%r10,%r12,1),%ebp
   41cb3:	add    %ebp,%esi
   41cb5:	shl    $0x8,%esi
   41cb8:	or     %ebx,%esi
   41cba:	mov    %esi,0x4(%rcx,%rax,1)
   41cbe:	movzwl 0x4(%rdi),%esi
   41cc2:	add    %ebp,%esi
   41cc4:	shl    $0x8,%esi
   41cc7:	or     %ebx,%esi
   41cc9:	mov    %esi,0x8(%rcx,%rax,1)
   41ccd:	movzwl 0x6(%rdi),%esi
   41cd1:	add    %ebp,%esi
   41cd3:	shl    $0x8,%esi
   41cd6:	or     %ebx,%esi
   41cd8:	mov    %esi,0xc(%rcx,%rax,1)
   41cdc:	movzwl 0x8(%rdi),%esi
   41ce0:	add    %ebp,%esi
   41ce2:	shl    $0x8,%esi
   41ce5:	or     %ebx,%esi
   41ce7:	mov    %esi,0x10(%rcx,%rax,1)
   41ceb:	movzwl 0xa(%rdi),%esi
   41cef:	add    %ebp,%esi
   41cf1:	shl    $0x8,%esi
   41cf4:	or     %ebx,%esi
   41cf6:	mov    %esi,0x14(%rcx,%rax,1)
   41cfa:	movzwl 0xc(%rdi),%esi
   41cfe:	add    %ebp,%esi
   41d00:	shl    $0x8,%esi
   41d03:	or     %ebx,%esi
   41d05:	mov    %esi,0x18(%rcx,%rax,1)
   41d09:	movzwl 0xe(%rdi),%esi
   41d0d:	add    %ebp,%esi
   41d0f:	shl    $0x8,%esi
   41d12:	or     %ebx,%esi
   41d14:	mov    %esi,0x1c(%rcx,%rax,1)
   41d18:	movzwl 0x10(%rdi),%esi
   41d1c:	add    %ebp,%esi
   41d1e:	shl    $0x8,%esi
   41d21:	or     %ebx,%esi
   41d23:	mov    %esi,0x20(%rcx,%rax,1)
   41d27:	movzwl 0x12(%rdi),%esi
   41d2b:	add    %ebp,%esi
   41d2d:	shl    $0x8,%esi
   41d30:	or     %ebx,%esi
   41d32:	mov    %esi,0x24(%rcx,%rax,1)
   41d36:	movzwl 0x14(%rdi),%esi
   41d3a:	add    %ebp,%esi
   41d3c:	shl    $0x8,%esi
   41d3f:	or     %ebx,%esi
   41d41:	mov    %esi,0x28(%rcx,%rax,1)
   41d45:	movzwl 0x16(%rdi),%esi
   41d49:	add    %ebp,%esi
   41d4b:	shl    $0x8,%esi
   41d4e:	or     %ebx,%esi
   41d50:	mov    %esi,0x2c(%rcx,%rax,1)
   41d54:	movzwl 0x18(%rdi),%esi
   41d58:	add    %ebp,%esi
   41d5a:	shl    $0x8,%esi
   41d5d:	or     %ebx,%esi
   41d5f:	mov    %esi,0x30(%rcx,%rax,1)
   41d63:	movzwl 0x1a(%rdi),%esi
   41d67:	add    %ebp,%esi
   41d69:	shl    $0x8,%esi
   41d6c:	or     %ebx,%esi
   41d6e:	mov    %esi,0x34(%rcx,%rax,1)
   41d72:	movzwl 0x1c(%rdi),%esi
   41d76:	add    %ebp,%esi
   41d78:	shl    $0x8,%esi
   41d7b:	or     %ebx,%esi
   41d7d:	mov    %esi,0x38(%rcx,%rax,1)
   41d81:	movzwl 0x1e(%rdi),%esi
   41d85:	add    %ebp,%esi
   41d87:	shl    $0x8,%esi
   41d8a:	or     %ebx,%esi
   41d8c:	mov    %esi,0x3c(%rcx,%rax,1)
   41d90:	movzwl 0x20(%rdi),%esi
   41d94:	add    %ebp,%esi
   41d96:	shl    $0x8,%esi
   41d99:	or     %ebx,%esi
   41d9b:	mov    %esi,0x40(%rcx,%rax,1)
   41d9f:	movzwl 0x22(%rdi),%esi
   41da3:	add    %ebp,%esi
   41da5:	shl    $0x8,%esi
   41da8:	or     %ebx,%esi
   41daa:	mov    %esi,0x44(%rcx,%rax,1)
   41dae:	movzwl 0x24(%rdi),%esi
   41db2:	add    %ebp,%esi
   41db4:	shl    $0x8,%esi
   41db7:	or     %ebx,%esi
   41db9:	mov    %esi,0x48(%rcx,%rax,1)
   41dbd:	movzwl 0x26(%rdi),%esi
   41dc1:	add    %ebp,%esi
   41dc3:	shl    $0x8,%esi
   41dc6:	or     %ebx,%esi
   41dc8:	mov    %esi,0x4c(%rcx,%rax,1)
   41dcc:	movzwl 0x28(%rdi),%esi
   41dd0:	add    %ebp,%esi
   41dd2:	shl    $0x8,%esi
   41dd5:	or     %ebx,%esi
   41dd7:	mov    %esi,0x50(%rcx,%rax,1)
   41ddb:	movzwl 0x2a(%rdi),%esi
   41ddf:	add    %ebp,%esi
   41de1:	shl    $0x8,%esi
   41de4:	or     %ebx,%esi
   41de6:	mov    %esi,0x54(%rcx,%rax,1)
   41dea:	movzwl 0x2c(%rdi),%esi
   41dee:	add    %ebp,%esi
   41df0:	shl    $0x8,%esi
   41df3:	or     %ebx,%esi
   41df5:	mov    %esi,0x58(%rcx,%rax,1)
   41df9:	movzwl 0x2e(%rdi),%esi
   41dfd:	add    %ebp,%esi
   41dff:	shl    $0x8,%esi
   41e02:	or     %ebx,%esi
   41e04:	mov    %esi,0x5c(%rcx,%rax,1)
   41e08:	movzwl 0x30(%rdi),%esi
   41e0c:	add    %ebp,%esi
   41e0e:	shl    $0x8,%esi
   41e11:	or     %ebx,%esi
   41e13:	mov    %esi,0x60(%rcx,%rax,1)
   41e17:	movzwl 0x32(%rdi),%esi
   41e1b:	add    %ebp,%esi
   41e1d:	shl    $0x8,%esi
   41e20:	or     %ebx,%esi
   41e22:	mov    %esi,0x64(%rcx,%rax,1)
   41e26:	movzwl 0x34(%rdi),%esi
   41e2a:	add    %ebp,%esi
   41e2c:	shl    $0x8,%esi
   41e2f:	or     %ebx,%esi
   41e31:	mov    %esi,0x68(%rcx,%rax,1)
   41e35:	movzwl 0x36(%rdi),%esi
   41e39:	add    %ebp,%esi
   41e3b:	shl    $0x8,%esi
   41e3e:	or     %ebx,%esi
   41e40:	mov    %esi,0x6c(%rcx,%rax,1)
   41e44:	movzwl 0x38(%rdi),%esi
   41e48:	add    %ebp,%esi
   41e4a:	shl    $0x8,%esi
   41e4d:	or     %ebx,%esi
   41e4f:	mov    %esi,0x70(%rcx,%rax,1)
   41e53:	movzwl 0x3a(%rdi),%esi
   41e57:	add    %ebp,%esi
   41e59:	shl    $0x8,%esi
   41e5c:	or     %ebx,%esi
   41e5e:	mov    %esi,0x74(%rcx,%rax,1)
   41e62:	lea    (%r10,%r12,1),%eax
   41e66:	add    $0x20,%eax
   41e69:	mov    %eax,0x28f0(%rsp)
   41e70:	add    $0x20,%r12
   41e74:	inc    %r9
   41e77:	add    $0x2000,%r15d
   41e7e:	dec    %rdx
   41e81:	je     4208c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x118c>
   41e87:	mov    %r9,%rax
   41e8a:	shr    $0x6,%rax
   41e8e:	cmp    0x20(%rsp),%rax
   41e93:	jae    42590 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1690>
   41e99:	mov    0x40(%rsp),%rcx
   41e9e:	mov    (%rcx,%rax,8),%rax
   41ea2:	bt     %r9,%rax
   41ea6:	jae    41ed0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xfd0>
   41ea8:	cmp    $0x1e,%r14
   41eac:	jae    425b6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16b6>
   41eb2:	mov    0x0(%r13,%r12,2),%eax
   41eb7:	and    $0xfffff,%eax
   41ebc:	mov    0x10(%r11),%rcx
   41ec0:	cmp    %rax,%rcx
   41ec3:	ja     41ca0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xda0>
   41ec9:	jmp    425a4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16a4>
   41ece:	xchg   %ax,%ax
   41ed0:	cmp    $0x1e,%r14
   41ed4:	jae    425ca <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16ca>
   41eda:	movups 0x2c(%rdi),%xmm0
   41ede:	movups %xmm0,-0x10(%r13,%r12,2)
   41ee4:	movdqa (%rdi),%xmm0
   41ee8:	movdqa 0x10(%rdi),%xmm1
   41eed:	movdqa 0x20(%rdi),%xmm2
   41ef2:	movdqa %xmm2,-0x1c(%r13,%r12,2)
   41ef9:	movdqa %xmm1,-0x2c(%r13,%r12,2)
   41f00:	movdqa %xmm0,-0x3c(%r13,%r12,2)
   41f07:	mov    %r15d,0x0(%r13,%r12,2)
   41f0c:	jmp    41e62 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xf62>
   41f11:	mov    0x28e8(%rsp),%rdi
   41f19:	cmp    $0x1d,%rdi
   41f1d:	ja     42651 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1751>
   41f23:	shl    $0x6,%rdi
   41f27:	add    %r12,%rdi
   41f2a:	mov    0x28f0(%rsp),%eax
   41f31:	mov    0xd0(%rsp),%rcx
   41f39:	shl    $0x6,%r9
   41f3d:	mov    %eax,%r15d
   41f40:	shl    $0x8,%r15d
   41f44:	or     %ebx,%r15d
   41f47:	lea    -0x1e(%r14),%rsi
   41f4b:	cmp    $0x1e,%rsi
   41f4f:	jae    41fb9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x10b9>
   41f51:	xor    %esi,%esi
   41f53:	jmp    4204a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x114a>
   41f58:	mov    %r14,%rbp
   41f5b:	shr    $1,%r9
   41f5e:	inc    %r9
   41f61:	mov    %r9,%r10
   41f64:	and    $0xfffffffffffffff8,%r10
   41f68:	lea    (%rdx,%r10,2),%r8
   41f6c:	movd   %ecx,%xmm0
   41f70:	pshuflw $0x0,%xmm0,%xmm0
   41f75:	add    %r12,%r11
   41f78:	add    $0x8,%r11
   41f7c:	xor    %r14d,%r14d
   41f7f:	nop
   41f80:	movq   -0x8(%r11,%r14,2),%xmm1
   41f87:	movq   (%r11,%r14,2),%xmm2
   41f8d:	paddw  %xmm0,%xmm1
   41f91:	paddw  %xmm0,%xmm2
   41f95:	punpcklqdq %xmm2,%xmm1
   41f99:	movdqu %xmm1,(%rax,%r14,2)
   41f9f:	add    $0x8,%r14
   41fa3:	cmp    %r14,%r10
   41fa6:	jne    41f80 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1080>
   41fa8:	cmp    %r10,%r9
   41fab:	mov    %rbp,%r14
   41fae:	jne    41b2b <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc2b>
   41fb4:	jmp    41b59 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc59>
   41fb9:	mov    %rdx,%r10
   41fbc:	movabs $0xffffffffffffffe,%rsi
   41fc6:	and    %rsi,%r10
   41fc9:	lea    (%rcx,%r9,1),%r11
   41fcd:	add    $0x7c,%r11
   41fd1:	xor    %esi,%esi
   41fd3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41fe0:	movdqu 0x2c(%rdi),%xmm0
   41fe5:	movdqu %xmm0,-0x50(%r11)
   41feb:	movdqa (%rdi),%xmm1
   41fef:	movdqa 0x10(%rdi),%xmm2
   41ff4:	movdqa 0x20(%rdi),%xmm3
   41ff9:	movdqa %xmm3,-0x5c(%r11)
   41fff:	movdqa %xmm2,-0x6c(%r11)
   42005:	movdqa %xmm1,-0x7c(%r11)
   4200b:	mov    %r15d,-0x40(%r11)
   4200f:	add    $0x2,%rsi
   42013:	movdqa %xmm1,-0x3c(%r11)
   42019:	movdqa %xmm2,-0x2c(%r11)
   4201f:	movdqa %xmm3,-0x1c(%r11)
   42025:	movdqu %xmm0,-0x10(%r11)
   4202b:	lea    0x2000(%r15),%ebp
   42032:	mov    %ebp,(%r11)
   42035:	add    $0x4000,%r15d
   4203c:	sub    $0xffffffffffffff80,%r11
   42040:	cmp    %rsi,%r10
   42043:	jne    41fe0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x10e0>
   42045:	test   $0x1,%dl
   42048:	je     4207e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x117e>
   4204a:	add    %r9,%rcx
   4204d:	shl    $0x6,%rsi
   42051:	movups 0x2c(%rdi),%xmm0
   42055:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4205a:	movdqa (%rdi),%xmm0
   4205e:	movdqa 0x10(%rdi),%xmm1
   42063:	movdqa 0x20(%rdi),%xmm2
   42068:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   4206e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   42074:	movdqa %xmm0,(%rcx,%rsi,1)
   42079:	mov    %r15d,0x3c(%rcx,%rsi,1)
   4207e:	mov    %edx,%ecx
   42080:	shl    $0x5,%ecx
   42083:	add    %ecx,%eax
   42085:	mov    %eax,0x28f0(%rsp)
   4208c:	mov    0xa0(%rsp),%rdx
   42094:	mov    %rdx,%rax
   42097:	shl    $0x5,%rax
   4209b:	lea    (%rdx,%rdx,1),%rcx
   4209f:	sub    %rax,%rcx
   420a2:	mov    0x60(%rsp),%r14
   420a7:	add    %rcx,%r14
   420aa:	lea    (%rdx,%rdx,1),%eax
   420ad:	shl    $0x5,%edx
   420b0:	sub    %eax,%edx
   420b2:	add    %edx,%r8d
   420b5:	mov    0x38(%rsp),%r10
   420ba:	mov    %r8d,0x18(%r10)
   420be:	test   %r14,%r14
   420c1:	lea    0xe8(%rsp),%r13
   420c9:	mov    $0x88888889,%r15d
   420cf:	lea    -0x26e16(%rip),%r12        # 1b2c0 <c30walk::ctor2s::PATTERNS>
   420d6:	je     42140 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1240>
   420d8:	mov    0x1c(%r10),%ecx
   420dc:	mov    %r8d,%edi
   420df:	imul   %r15,%rdi
   420e3:	mov    %rdi,%rdx
   420e6:	shr    $0x24,%rdx
   420ea:	mov    %edx,%eax
   420ec:	shl    $0x5,%eax
   420ef:	sub    %edx,%eax
   420f1:	sub    %edx,%eax
   420f3:	lea    0x1e(%rax),%esi
   420f6:	cmp    %ecx,%esi
   420f8:	cmovb  %esi,%ecx
   420fb:	shr    $0x2a,%rdi
   420ff:	mov    0xb8(%rsp),%rsi
   42107:	cmp    %rsi,%rdi
   4210a:	jae    42634 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1734>
   42110:	mov    %rdx,%rsi
   42113:	shl    $0x6,%rsi
   42117:	add    0xd0(%rsp),%rsi
   4211f:	mov    0xb0(%rsp),%r9
   42127:	mov    (%r9,%rdi,8),%rdi
   4212b:	bt     %rdx,%rdi
   4212f:	jb     41a90 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb90>
   42135:	mov    $0xffffffff,%edi
   4213a:	jmp    41a9c <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb9c>
   4213f:	nop
   42140:	mov    0x8(%rsp),%rdi
   42145:	mov    0x18(%rsp),%rsi
   4214a:	mov    0x4c(%rsp),%edx
   4214e:	jmp    4183b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x93b>
   42153:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   42160:	mov    %rbx,%rbp
   42163:	mov    %r14,%rcx
   42166:	mov    0x18(%rsp),%rsi
   4216b:	mov    %rsi,0x18(%rsp)
   42170:	test   %rcx,%rcx
   42173:	je     416fa <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7fa>
   42179:	mov    0x28e8(%rsp),%rax
   42181:	lea    -0x1e(%rax),%r15
   42185:	neg    %r15
   42188:	cmp    %rcx,%r15
   4218b:	mov    %rcx,0x30(%rsp)
   42190:	mov    %rcx,%rbx
   42193:	cmovb  %r15,%rbx
   42197:	mov    0x28f0(%rsp),%r14d
   4219f:	cmp    $0x1e,%rax
   421a3:	jne    4229f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x139f>
   421a9:	mov    0x30(%rsp),%r8
   421ae:	cmp    %r15,%r8
   421b1:	lea    0xe8(%rsp),%rdx
   421b9:	jbe    42230 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1330>
   421bb:	mov    %rbp,%r9
   421be:	add    $0x2,%r14d
   421c2:	movzwl %r14w,%r14d
   421c6:	cs nopw 0x0(%rax,%rax,1)
   421d0:	movzbl (%r9,%rbx,1),%esi
   421d5:	lea    (%rsi,%rsi,4),%rcx
   421d9:	lea    (%rdx,%rcx,8),%r15
   421dd:	mov    (%rdx,%rcx,8),%rax
   421e1:	cmp    0x8(%rdx,%rcx,8),%rax
   421e6:	je     421fd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12fd>
   421e8:	lea    (%r14,%rbx,1),%rcx
   421ec:	mov    %cx,(%rax)
   421ef:	addq   $0x2,(%r15)
   421f3:	inc    %rbx
   421f6:	cmp    %rbx,%r8
   421f9:	jne    421d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12d0>
   421fb:	jmp    42230 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1330>
   421fd:	lea    0xa8(%rsp),%rdi
   42205:	call   8bae0 <<c30walk::ctor2s::W<false>>::refill>
   4220a:	test   %al,%al
   4220c:	je     4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   42212:	mov    (%r15),%rax
   42215:	mov    0x8(%rsp),%rdi
   4221a:	lea    0xe8(%rsp),%rdx
   42222:	mov    0x30(%rsp),%r8
   42227:	mov    %rbp,%r9
   4222a:	jmp    421e8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12e8>
   4222c:	nopl   0x0(%rax)
   42230:	mov    0x28e8(%rsp),%rax
   42238:	mov    0x28f0(%rsp),%ecx
   4223f:	cmp    $0x1,%r8
   42243:	je     416d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7d0>
   42249:	mov    %r8d,%edx
   4224c:	and    $0xe,%edx
   4224f:	nop
   42250:	lea    0x1(%rax),%rsi
   42254:	add    $0xffffffffffffffe3,%rax
   42258:	xor    %edi,%edi
   4225a:	cmp    $0x1e,%rsi
   4225e:	setae  %dil
   42262:	cmovb  %rsi,%rax
   42266:	lea    (%rcx,%rdi,2),%ecx
   42269:	lea    0x1(%rax),%rsi
   4226d:	add    $0xffffffffffffffe3,%rax
   42271:	xor    %edi,%edi
   42273:	cmp    $0x1e,%rsi
   42277:	setae  %dil
   4227b:	cmovb  %rsi,%rax
   4227f:	lea    (%rcx,%rdi,2),%ecx
   42282:	add    $0x2,%ecx
   42285:	add    $0xfffffffffffffffe,%rdx
   42289:	jne    42250 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1350>
   4228b:	test   $0x1,%r8b
   4228f:	mov    0x8(%rsp),%rdi
   42294:	jne    416d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7d0>
   4229a:	jmp    416eb <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7eb>
   4229f:	xor    %r12d,%r12d
   422a2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   422b0:	movzbl 0x0(%rbp,%r12,1),%esi
   422b6:	lea    (%rsi,%rsi,4),%rcx
   422ba:	lea    0xe8(%rsp),%rdx
   422c2:	lea    (%rdx,%rcx,8),%r13
   422c6:	mov    (%rdx,%rcx,8),%rax
   422ca:	cmp    0x8(%rdx,%rcx,8),%rax
   422cf:	je     422ea <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13ea>
   422d1:	lea    (%r14,%r12,1),%rcx
   422d5:	mov    %cx,(%rax)
   422d8:	addq   $0x2,0x0(%r13)
   422dd:	inc    %r12
   422e0:	cmp    %r12,%rbx
   422e3:	jne    422b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13b0>
   422e5:	jmp    421a9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12a9>
   422ea:	lea    0xa8(%rsp),%rdi
   422f2:	call   8bae0 <<c30walk::ctor2s::W<false>>::refill>
   422f7:	test   %al,%al
   422f9:	je     4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   422fb:	mov    0x0(%r13),%rax
   422ff:	mov    0x8(%rsp),%rdi
   42304:	jmp    422d1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13d1>
   42306:	mov    %r14,%rdi
   42309:	call   e60b0 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   4230e:	jmp    4178f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x88f>
   42313:	mov    $0x84,%eax
   42318:	nopl   0x0(%rax,%rax,1)
   42320:	lea    0x50(%rax),%rcx
   42324:	cmp    $0x28d4,%rcx
   4232b:	je     423a4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14a4>
   4232d:	mov    0x64(%rsp,%rax,1),%rdx
   42332:	sub    0x74(%rsp,%rax,1),%rdx
   42337:	shr    $1,%rdx
   4233a:	add    0x7c(%rsp,%rax,1),%edx
   4233e:	cmp    0x80(%rsp,%rax,1),%edx
   42345:	jne    4236d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x146d>
   42347:	mov    0x8c(%rsp,%rax,1),%rdx
   4234f:	sub    0x9c(%rsp,%rax,1),%rdx
   42357:	shr    $1,%rdx
   4235a:	add    0xa4(%rsp,%rax,1),%edx
   42361:	cmp    0xa8(%rsp,%rax,1),%edx
   42368:	mov    %rcx,%rax
   4236b:	je     42320 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1420>
   4236d:	mov    0x5530(%rsp),%rax
   42375:	movq   $0x0,0x10(%rax)
   4237d:	xor    %eax,%eax
   4237f:	add    $0x54f8,%rsp
   42386:	pop    %rbx
   42387:	pop    %r12
   42389:	pop    %r13
   4238b:	pop    %r14
   4238d:	pop    %r15
   4238f:	pop    %rbp
   42390:	ret
   42391:	lea    0xedb68(%rip),%rcx        # 12ff00 <__frame_dummy_init_array_entry+0x2060>
   42398:	mov    $0x1e,%edx
   4239d:	xor    %edi,%edi
   4239f:	call   d8290 <core::slice::index::slice_index_fail>
   423a4:	xor    %ebx,%ebx
   423a6:	lea    0xa8(%rsp),%r14
   423ae:	mov    0x70(%rsp),%r15
   423b3:	mov    0x50(%rsp),%r12
   423b8:	mov    0x8(%rsp),%rdx
   423bd:	mov    0x68(%rsp),%r13
   423c2:	lea    0xe8(%rsp),%rbp
   423ca:	jmp    423fa <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14fa>
   423cc:	nopl   0x0(%rax)
   423d0:	sub    %rcx,%rax
   423d3:	inc    %rbx
   423d6:	shr    $1,%rax
   423d9:	add    %eax,0x18(%rbp)
   423dc:	pxor   %xmm0,%xmm0
   423e0:	movdqu %xmm0,0x0(%rbp)
   423e5:	movq   $0x0,0x10(%rbp)
   423ed:	add    $0x28,%rbp
   423f1:	cmp    $0x100,%rbx
   423f8:	je     42421 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1521>
   423fa:	mov    0x0(%rbp),%rcx
   423fe:	mov    %rcx,%rax
   42401:	cmp    0x10(%rbp),%rcx
   42405:	je     423d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d0>
   42407:	mov    %r14,%rdi
   4240a:	mov    %rbx,%rsi
   4240d:	call   8b4e0 <<c30walk::ctor2s::W<false>>::finish>
   42412:	mov    0x8(%rsp),%rdx
   42417:	mov    0x0(%rbp),%rax
   4241b:	mov    0x10(%rbp),%rcx
   4241f:	jmp    423d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d0>
   42421:	xor    %ecx,%ecx
   42423:	mov    0x8(%r15),%rbx
   42427:	test   %r13,%r13
   4242a:	setne  %al
   4242d:	je     42471 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1571>
   4242f:	mov    %al,%cl
   42431:	mov    %r13,%rax
   42434:	sub    %rcx,%rax
   42437:	imul   $0xffffffffffffffe2,%rax,%rcx
   4243b:	add    %rdx,%rcx
   4243e:	cmp    $0x1e,%rcx
   42442:	jae    42471 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1571>
   42444:	lea    (%rbx,%r13,4),%rcx
   42448:	lea    (%rcx,%rdx,2),%rdi
   4244c:	add    $0xfffffffffffffffc,%rdi
   42450:	lea    (%rax,%rax,1),%rcx
   42454:	shl    $0x5,%rax
   42458:	sub    %rcx,%rax
   4245b:	sub    %rdx,%rax
   4245e:	lea    0x3c(,%rax,2),%rdx
   42466:	mov    $0xff,%esi
   4246b:	call   *0xf1d1f(%rip)        # 134190 <memset@GLIBC_2.2.5>
   42471:	shl    $0x6,%r13
   42475:	pcmpeqd %xmm0,%xmm0
   42479:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   42480:	movdqa %xmm0,0x20(%rbx,%r13,1)
   42487:	movdqa %xmm0,0x10(%rbx,%r13,1)
   4248e:	movdqa %xmm0,(%rbx,%r13,1)
   42494:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   4249d:	mov    0x8(%r12),%rdx
   424a2:	mov    0x10(%r12),%rcx
   424a7:	mov    0x5530(%rsp),%rax
   424af:	mov    0x8(%rax),%r8
   424b3:	mov    0x10(%rax),%r9
   424b7:	lea    0xa8(%rsp),%rdi
   424bf:	mov    %rbx,%rsi
   424c2:	call   99d40 <c30walk::ctor::canonicalise>
   424c7:	mov    0x78(%rsp),%rax
   424cc:	mov    %rax,0x10(%r15)
   424d0:	mov    $0x1,%al
   424d2:	jmp    4237f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x147f>
   424d7:	mov    %rdi,%r12
   424da:	test   %rax,%rax
   424dd:	je     424ef <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15ef>
   424df:	mov    0x8(%r9),%rdi
   424e3:	mov    %r9,%r15
   424e6:	call   *0xf1cb4(%rip)        # 1341a0 <free@GLIBC_2.2.5>
   424ec:	mov    %r15,%r9
   424ef:	movq   $0x0,(%r9)
   424f6:	movq   $0x40,0x8(%r9)
   424fe:	movq   $0x0,0x10(%r9)
   42506:	mov    $0x40,%edx
   4250b:	mov    %r9,%rdi
   4250e:	lea    0x1(%r13),%rsi
   42512:	mov    %r9,%r15
   42515:	call   90040 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4251a:	mov    %r15,%r9
   4251d:	cmp    $0xffffffffffffffff,%rax
   42521:	mov    %r12,%rdi
   42524:	je     41134 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x234>
   4252a:	jmp    42585 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1685>
   4252c:	mov    %rdi,%r12
   4252f:	mov    %r9,%r15
   42532:	test   %rax,%rax
   42535:	je     42549 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1649>
   42537:	mov    0x5530(%rsp),%rax
   4253f:	mov    0x8(%rax),%rdi
   42543:	call   *0xf1c57(%rip)        # 1341a0 <free@GLIBC_2.2.5>
   42549:	mov    0x5530(%rsp),%rdi
   42551:	movq   $0x0,(%rdi)
   42558:	movq   $0x40,0x8(%rdi)
   42560:	movq   $0x0,0x10(%rdi)
   42568:	mov    $0x80,%edx
   4256d:	mov    %r14,%rsi
   42570:	call   90040 <<alloc::raw_vec::RawVecInner>::grow_exact>
   42575:	cmp    $0xffffffffffffffff,%rax
   42579:	mov    %r15,%r9
   4257c:	mov    %r12,%rdi
   4257f:	je     41148 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x248>
   42585:	mov    %rax,%rdi
   42588:	mov    %rdx,%rsi
   4258b:	call   d5a1b <alloc::raw_vec::handle_error>
   42590:	lea    0xebb79(%rip),%rdx        # 12e110 <__frame_dummy_init_array_entry+0x270>
   42597:	mov    %rax,%rdi
   4259a:	mov    0x20(%rsp),%rsi
   4259f:	call   d74b3 <core::panicking::panic_bounds_check>
   425a4:	lea    0xed9e5(%rip),%rdx        # 12ff90 <__frame_dummy_init_array_entry+0x20f0>
   425ab:	mov    %rax,%rdi
   425ae:	mov    %rcx,%rsi
   425b1:	call   d74b3 <core::panicking::panic_bounds_check>
   425b6:	lea    0xed9bb(%rip),%rdx        # 12ff78 <__frame_dummy_init_array_entry+0x20d8>
   425bd:	mov    $0x1e,%esi
   425c2:	mov    %r14,%rdi
   425c5:	call   d74b3 <core::panicking::panic_bounds_check>
   425ca:	lea    0xed98f(%rip),%rdx        # 12ff60 <__frame_dummy_init_array_entry+0x20c0>
   425d1:	mov    $0x1e,%esi
   425d6:	mov    %r14,%rdi
   425d9:	call   d74b3 <core::panicking::panic_bounds_check>
   425de:	mov    $0x10,%edx
   425e3:	mov    $0x4,%ecx
   425e8:	mov    $0x400,%r8d
   425ee:	mov    %rdi,%r12
   425f1:	mov    %r15,%rdi
   425f4:	xor    %esi,%esi
   425f6:	mov    %r9,%r14
   425f9:	call   67350 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   425fe:	mov    %r12,%rdi
   42601:	mov    %r14,%r9
   42604:	jmp    411db <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2db>
   42609:	mov    $0x8,%ecx
   4260e:	mov    $0x8,%r8d
   42614:	mov    %rdi,%r12
   42617:	xor    %esi,%esi
   42619:	mov    %r14,%rdx
   4261c:	mov    %r9,%r15
   4261f:	call   67350 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   42624:	mov    %r12,%rdi
   42627:	mov    %r15,%r9
   4262a:	mov    0x10(%r12),%r12
   4262f:	jmp    41170 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x270>
   42634:	lea    0xebad5(%rip),%rdx        # 12e110 <__frame_dummy_init_array_entry+0x270>
   4263b:	call   d74b3 <core::panicking::panic_bounds_check>
   42640:	lea    0xed8a1(%rip),%rdx        # 12fee8 <__frame_dummy_init_array_entry+0x2048>
   42647:	mov    $0x1e,%esi
   4264c:	call   d74b3 <core::panicking::panic_bounds_check>
   42651:	lea    0xed8f0(%rip),%rdx        # 12ff48 <__frame_dummy_init_array_entry+0x20a8>
   42658:	mov    $0x1e,%esi
   4265d:	call   d74b3 <core::panicking::panic_bounds_check>
   42662:	lea    0xed807(%rip),%rdx        # 12fe70 <__frame_dummy_init_array_entry+0x1fd0>
   42669:	call   d74b3 <core::panicking::panic_bounds_check>
   4266e:	lea    0xeba9b(%rip),%rdx        # 12e110 <__frame_dummy_init_array_entry+0x270>
   42675:	mov    %r15,%rdi
   42678:	call   d74b3 <core::panicking::panic_bounds_check>
