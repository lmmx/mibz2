# <c30walk::ctor2s::Ctor2s>::build::<false>
# size 0x18a6

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
   40f61:	mov    %rsi,0x48(%rsp)
   40f66:	mov    0x5530(%rsp),%rax
   40f6e:	movq   $0x0,0x10(%r9)
   40f76:	movq   $0x0,0x10(%rax)
   40f7e:	cmp    $0xeffe2,%rdx
   40f85:	ja     4243d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   40f8b:	mov    %rcx,%r15
   40f8e:	mov    %rdx,%rsi
   40f91:	pxor   %xmm0,%xmm0
   40f95:	xor    %eax,%eax
   40f97:	pxor   %xmm2,%xmm2
   40f9b:	pxor   %xmm1,%xmm1
   40f9f:	nop
   40fa0:	movq   (%r15,%rax,4),%xmm3
   40fa6:	movq   0x8(%r15,%rax,4),%xmm4
   40fad:	punpckldq %xmm0,%xmm3
   40fb1:	paddq  %xmm2,%xmm3
   40fb5:	punpckldq %xmm0,%xmm4
   40fb9:	paddq  %xmm1,%xmm4
   40fbd:	movq   0x10(%r15,%rax,4),%xmm2
   40fc4:	movq   0x18(%r15,%rax,4),%xmm1
   40fcb:	punpckldq %xmm0,%xmm2
   40fcf:	paddq  %xmm3,%xmm2
   40fd3:	punpckldq %xmm0,%xmm1
   40fd7:	paddq  %xmm4,%xmm1
   40fdb:	add    $0x8,%rax
   40fdf:	cmp    $0x100,%rax
   40fe5:	jne    40fa0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa0>
   40fe7:	paddq  %xmm2,%xmm1
   40feb:	pshufd $0xee,%xmm1,%xmm0
   40ff0:	paddq  %xmm1,%xmm0
   40ff4:	movq   %xmm0,%rax
   40ff9:	cmp    %rsi,%rax
   40ffc:	jne    4243d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   41002:	pxor   %xmm2,%xmm2
   41006:	xor    %eax,%eax
   41008:	movdqa -0x319f0(%rip),%xmm3        # f620 <GCC_except_table971+0x9f8>
   41010:	pxor   %xmm1,%xmm1
   41014:	pxor   %xmm0,%xmm0
   41018:	nopl   0x0(%rax,%rax,1)
   41020:	movq   (%r15,%rax,4),%xmm4
   41026:	movq   0x8(%r15,%rax,4),%xmm5
   4102d:	pshufd $0xd4,%xmm4,%xmm4
   41032:	pcmpeqd %xmm2,%xmm4
   41036:	pandn  %xmm3,%xmm4
   4103a:	paddq  %xmm1,%xmm4
   4103e:	pshufd $0xd4,%xmm5,%xmm5
   41043:	pcmpeqd %xmm2,%xmm5
   41047:	pandn  %xmm3,%xmm5
   4104b:	paddq  %xmm0,%xmm5
   4104f:	movq   0x10(%r15,%rax,4),%xmm0
   41056:	movq   0x18(%r15,%rax,4),%xmm6
   4105d:	pshufd $0xd4,%xmm0,%xmm1
   41062:	pcmpeqd %xmm2,%xmm1
   41066:	pandn  %xmm3,%xmm1
   4106a:	paddq  %xmm4,%xmm1
   4106e:	pshufd $0xd4,%xmm6,%xmm0
   41073:	pcmpeqd %xmm2,%xmm0
   41077:	pandn  %xmm3,%xmm0
   4107b:	paddq  %xmm5,%xmm0
   4107f:	add    $0x8,%rax
   41083:	cmp    $0x100,%rax
   41089:	jne    41020 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120>
   4108b:	mov    $0x88888889,%r13d
   41091:	imul   %rsi,%r13
   41095:	shr    $0x24,%r13
   41099:	imul   $0xeeeeeeef,%esi,%eax
   4109f:	ror    $1,%eax
   410a1:	cmp    $0x8888889,%eax
   410a6:	sbb    $0xffffffffffffffff,%r13
   410aa:	mov    %r13,%rbx
   410ad:	cmpl   $0x0,0x14(%rsp)
   410b2:	jne    41106 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   410b4:	test   %rsi,%rsi
   410b7:	je     41104 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x204>
   410b9:	paddq  %xmm1,%xmm0
   410bd:	pshufd $0xee,%xmm0,%xmm1
   410c2:	paddq  %xmm0,%xmm1
   410c6:	movq   %xmm1,%rax
   410cb:	lea    -0x1(%rsi),%ecx
   410ce:	mov    $0x88888889,%edx
   410d3:	imul   %rcx,%rdx
   410d7:	shr    $0x23,%rdx
   410db:	and    $0xfffffffe,%edx
   410de:	lea    (%rdx,%rsi,1),%ecx
   410e1:	dec    %ecx
   410e3:	shr    $1,%ecx
   410e5:	mov    $0x80010003,%ebx
   410ea:	imul   %rcx,%rbx
   410ee:	shr    $0x2e,%rbx
   410f2:	inc    %ebx
   410f4:	imul   %rax,%rbx
   410f8:	dec    %rbx
   410fb:	cmp    %r13,%rbx
   410fe:	cmovae %r13,%rbx
   41102:	jmp    41106 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   41104:	xor    %ebx,%ebx
   41106:	mov    (%r9),%rax
   41109:	lea    0x1(%r13),%rcx
   4110d:	mov    %rcx,0x90(%rsp)
   41115:	cmp    %r13,%rax
   41118:	mov    %rsi,0x8(%rsp)
   4111d:	jbe    4259c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x169c>
   41123:	mov    0x5530(%rsp),%rax
   4112b:	mov    (%rax),%rax
   4112e:	cmp    %rbx,%rax
   41131:	jb     425f1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16f1>
   41137:	movq   $0x0,0x10(%rdi)
   4113f:	lea    0x3f(%r13),%rbx
   41143:	shr    $0x6,%rbx
   41147:	mov    %rdi,0x70(%rsp)
   4114c:	mov    %r13,0x78(%rsp)
   41151:	je     411a6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a6>
   41153:	cmp    (%rdi),%rbx
   41156:	ja     42724 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1824>
   4115c:	mov    %r9,%r13
   4115f:	xor    %r14d,%r14d
   41162:	mov    0x8(%rdi),%r12
   41166:	lea    (%r12,%r14,8),%rdi
   4116a:	cmp    $0x1,%rbx
   4116e:	je     41192 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   41170:	lea    -0x8(,%rbx,8),%rdx
   41178:	xor    %esi,%esi
   4117a:	call   *0xf32a0(%rip)        # 134420 <memset@GLIBC_2.2.5>
   41180:	lea    (%r14,%rbx,1),%rax
   41184:	lea    (%r12,%rax,8),%rdi
   41188:	add    $0xfffffffffffffff8,%rdi
   4118c:	add    %rbx,%r14
   4118f:	dec    %r14
   41192:	mov    %r13,%r9
   41195:	movq   $0x0,(%rdi)
   4119c:	inc    %r14
   4119f:	mov    0x70(%rsp),%rdi
   411a4:	jmp    411a9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a9>
   411a6:	xor    %r14d,%r14d
   411a9:	lea    0x18(%rdi),%r13
   411ad:	mov    %r14,0x10(%rdi)
   411b1:	movq   $0x0,0x28(%rdi)
   411b9:	cmpq   $0xf,0x18(%rdi)
   411be:	jbe    426f9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17f9>
   411c4:	lea    0x30(%rdi),%r14
   411c8:	xor    %eax,%eax
   411ca:	pxor   %xmm0,%xmm0
   411ce:	movabs $0xffffffff00000000,%rcx
   411d8:	nopl   0x0(%rax,%rax,1)
   411e0:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   411e9:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   411f2:	mov    %rcx,0x2d18(%rsp,%rax,1)
   411fa:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   41203:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   4120c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   41214:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   4121d:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   41226:	mov    %rcx,0x2d68(%rsp,%rax,1)
   4122e:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   41237:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   41240:	mov    %rcx,0x2d90(%rsp,%rax,1)
   41248:	add    $0xa0,%rax
   4124e:	cmp    $0x2800,%rax
   41254:	jne    411e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2e0>
   41256:	lea    0xc(%r15),%rax
   4125a:	xor    %edx,%edx
   4125c:	mov    $0x25,%ecx
   41261:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41270:	mov    -0xc(%rax),%esi
   41273:	add    %edx,%esi
   41275:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   4127c:	mov    %esi,0x2c80(%rsp,%rcx,4)
   41283:	mov    -0x8(%rax),%edx
   41286:	add    %esi,%edx
   41288:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   4128f:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   41296:	mov    -0x4(%rax),%esi
   41299:	add    %edx,%esi
   4129b:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   412a2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   412a9:	mov    (%rax),%edx
   412ab:	add    %esi,%edx
   412ad:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   412b4:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   412bb:	add    $0x28,%rcx
   412bf:	add    $0x10,%rax
   412c3:	cmp    $0xa25,%rcx
   412ca:	jne    41270 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x370>
   412cc:	mov    %r9,0x88(%rsp)
   412d4:	mov    0x8(%r9),%rbx
   412d8:	mov    0x8(%rdi),%rbp
   412dc:	mov    0x10(%rdi),%r12
   412e0:	lea    0xe8(%rsp),%rdi
   412e8:	lea    0x2cf8(%rsp),%rsi
   412f0:	mov    $0x2800,%edx
   412f5:	call   *0xf3145(%rip)        # 134440 <memcpy@GLIBC_2.14>
   412fb:	mov    %rbp,%r9
   412fe:	mov    %r12,%rsi
   41301:	mov    %rbx,0xd0(%rsp)
   41309:	mov    0x78(%rsp),%rax
   4130e:	mov    %rax,0xd8(%rsp)
   41316:	mov    0x8(%rsp),%r11
   4131b:	mov    %r11,0xe0(%rsp)
   41323:	mov    0x5530(%rsp),%rax
   4132b:	mov    %rax,0xa8(%rsp)
   41333:	mov    %rbp,0xb0(%rsp)
   4133b:	mov    %r12,0xb8(%rsp)
   41343:	mov    %r13,0xc0(%rsp)
   4134b:	movl   $0x0,0x28f0(%rsp)
   41356:	movq   $0x0,0x28e8(%rsp)
   41362:	mov    %r14,0xc8(%rsp)
   4136a:	mov    $0xffffffffffffffff,%rcx
   41371:	xor    %r13d,%r13d
   41374:	movabs $0xeeeeeeeeeeeeeeef,%r10
   4137e:	movabs $0x888888888888889,%r12
   41388:	pxor   %xmm0,%xmm0
   4138c:	pcmpeqd %xmm1,%xmm1
   41390:	xor    %r14d,%r14d
   41393:	jmp    41443 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x543>
   41398:	nopl   0x0(%rax,%rax,1)
   413a0:	mov    0x8(%rdi),%rax
   413a4:	mov    %rbp,%rcx
   413a7:	shl    $0x7,%rcx
   413ab:	movdqa %xmm0,0x70(%rax,%rcx,1)
   413b1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   413b7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   413bd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   413c3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   413c9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   413cf:	movdqa %xmm0,0x10(%rax,%rcx,1)
   413d5:	movdqa %xmm0,(%rax,%rcx,1)
   413da:	mov    %rbp,%rax
   413dd:	inc    %rax
   413e0:	mov    %rax,0x10(%rdi)
   413e4:	mov    %rdx,%rax
   413e7:	shl    $0x6,%rax
   413eb:	or     $0x10000000,%ebp
   413f1:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   413f7:	movdqa %xmm1,0x20(%rbx,%rax,1)
   413fd:	movdqa %xmm1,0x10(%rbx,%rax,1)
   41403:	movdqa %xmm1,(%rbx,%rax,1)
   41408:	mov    %ebp,0x3c(%rbx,%rax,1)
   4140c:	mov    %rdx,%rdi
   4140f:	shr    $0x6,%rdi
   41413:	cmp    %rsi,%rdi
   41416:	jae    4278b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x188b>
   4141c:	mov    $0x1,%eax
   41421:	mov    %edx,%ecx
   41423:	shl    %cl,%rax
   41426:	or     %rax,(%r9,%rdi,8)
   4142a:	mov    %rdx,%rcx
   4142d:	add    $0x4,%r14
   41431:	cmp    $0x400,%r14
   41438:	mov    0x8(%rsp),%r11
   4143d:	je     414de <<c30walk::ctor2s::Ctor2s>::build::<false>+0x5de>
   41443:	mov    %r13,%rax
   41446:	mov    (%r15,%r14,1),%r13d
   4144a:	add    %rax,%r13
   4144d:	cmp    %r11,%r13
   41450:	setae  %r8b
   41454:	mov    %r13,%rax
   41457:	movabs $0x8888888888888889,%rdx
   41461:	mul    %rdx
   41464:	shr    $0x4,%rdx
   41468:	mov    %r13,%rax
   4146b:	imul   %r10,%rax
   4146f:	ror    $1,%rax
   41472:	cmp    %r12,%rax
   41475:	setb   %al
   41478:	cmp    %rcx,%rdx
   4147b:	sete   %dil
   4147f:	or     %r8b,%dil
   41482:	or     %al,%dil
   41485:	jne    4142d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x52d>
   41487:	mov    0x5530(%rsp),%rdi
   4148f:	mov    0x10(%rdi),%rbp
   41493:	cmp    (%rdi),%rbp
   41496:	jne    413a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   4149c:	mov    %rdx,0x18(%rsp)
   414a1:	mov    %rsi,0x30(%rsp)
   414a6:	mov    %r9,0x58(%rsp)
   414ab:	call   90f80 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   414b0:	pcmpeqd %xmm1,%xmm1
   414b4:	pxor   %xmm0,%xmm0
   414b8:	movabs $0xeeeeeeeeeeeeeeef,%r10
   414c2:	mov    0x58(%rsp),%r9
   414c7:	mov    0x30(%rsp),%rsi
   414cc:	mov    0x5530(%rsp),%rdi
   414d4:	mov    0x18(%rsp),%rdx
   414d9:	jmp    413a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   414de:	cmpl   $0x0,0x14(%rsp)
   414e3:	je     41694 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   414e9:	mov    0xd8(%rsp),%r8
   414f1:	test   %r8,%r8
   414f4:	je     41694 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   414fa:	mov    %r11,%r9
   414fd:	shl    $0x20,%r9
   41501:	mov    0x14(%rsp),%r10d
   41506:	mov    0xa8(%rsp),%rdi
   4150e:	mov    0xd0(%rsp),%rbp
   41516:	xor    %r12d,%r12d
   41519:	movabs $0xff51afd7ed558ccd,%rbx
   41523:	pxor   %xmm0,%xmm0
   41527:	pcmpeqd %xmm1,%xmm1
   4152b:	jmp    415b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6b0>
   41530:	mov    0x8(%rdi),%rax
   41534:	mov    %rbx,%rcx
   41537:	shl    $0x7,%rcx
   4153b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   41541:	movdqa %xmm0,0x60(%rax,%rcx,1)
   41547:	movdqa %xmm0,0x50(%rax,%rcx,1)
   4154d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   41553:	movdqa %xmm0,0x30(%rax,%rcx,1)
   41559:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4155f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   41565:	movdqa %xmm0,(%rax,%rcx,1)
   4156a:	mov    %rbx,%rax
   4156d:	inc    %rax
   41570:	mov    %rax,0x10(%rdi)
   41574:	or     $0x10000000,%ebx
   4157a:	movdqu %xmm1,0x2c(%rbp)
   4157f:	movdqa %xmm1,0x20(%rbp)
   41584:	movdqa %xmm1,0x10(%rbp)
   41589:	movdqa %xmm1,0x0(%rbp)
   4158e:	mov    %ebx,0x3c(%rbp)
   41591:	or     %r15,0x0(%r13,%r14,8)
   41596:	movabs $0xff51afd7ed558ccd,%rbx
   415a0:	inc    %r12
   415a3:	add    $0x40,%rbp
   415a7:	cmp    %r12,%r8
   415aa:	je     41694 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   415b0:	mov    %r12,%r14
   415b3:	shr    $0x6,%r14
   415b7:	mov    0xb8(%rsp),%rsi
   415bf:	cmp    %rsi,%r14
   415c2:	jae    42797 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1897>
   415c8:	mov    0xb0(%rsp),%r13
   415d0:	mov    0x0(%r13,%r14,8),%rax
   415d5:	mov    $0x1,%r15d
   415db:	mov    %r12d,%ecx
   415de:	shl    %cl,%r15
   415e1:	bt     %r12,%rax
   415e5:	jb     415a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   415e7:	mov    %r12,%rax
   415ea:	xor    %r9,%rax
   415ed:	mov    %rax,%rcx
   415f0:	shr    $0x21,%rcx
   415f4:	xor    %rax,%rcx
   415f7:	imul   %rbx,%rcx
   415fb:	mov    %rcx,%rax
   415fe:	shr    $0x21,%rax
   41602:	xor    %rcx,%rax
   41605:	movabs $0xc4ceb9fe1a85ec53,%rcx
   4160f:	imul   %rcx,%rax
   41613:	mov    %rax,%rcx
   41616:	shr    $0x21,%rcx
   4161a:	xor    %rax,%rcx
   4161d:	mov    %rcx,%rax
   41620:	shr    $0x3,%rax
   41624:	movabs $0x20c49ba5e353f7cf,%rdx
   4162e:	mul    %rdx
   41631:	shr    $0x4,%rdx
   41635:	imul   $0x3e8,%rdx,%rax
   4163c:	sub    %rax,%rcx
   4163f:	cmp    %r10,%rcx
   41642:	jae    415a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   41648:	mov    0x10(%rdi),%rbx
   4164c:	cmp    (%rdi),%rbx
   4164f:	jne    41530 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   41655:	mov    %r8,0x18(%rsp)
   4165a:	mov    %r9,0x30(%rsp)
   4165f:	mov    %r10,0x58(%rsp)
   41664:	mov    %rdi,0x68(%rsp)
   41669:	call   90f80 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4166e:	pcmpeqd %xmm1,%xmm1
   41672:	pxor   %xmm0,%xmm0
   41676:	mov    0x68(%rsp),%rdi
   4167b:	mov    0x58(%rsp),%r10
   41680:	mov    0x30(%rsp),%r9
   41685:	mov    0x18(%rsp),%r8
   4168a:	mov    0x8(%rsp),%r11
   4168f:	jmp    41530 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   41694:	test   %r11,%r11
   41697:	je     423d1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   4169d:	movq   $0x0,0x20(%rsp)
   416a6:	jmp    416ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ed>
   416a8:	mov    0x38(%rsp),%r14
   416ad:	lea    0x1(%rax),%rdx
   416b1:	add    $0xffffffffffffffe3,%rax
   416b5:	xor    %esi,%esi
   416b7:	cmp    $0x1e,%rdx
   416bb:	setae  %sil
   416bf:	cmovb  %rdx,%rax
   416c3:	lea    (%rcx,%rsi,2),%ecx
   416c6:	inc    %ecx
   416c8:	mov    %rax,0x28e8(%rsp)
   416d0:	mov    %ecx,0x28f0(%rsp)
   416d7:	mov    0x40(%rsp),%rcx
   416dc:	mov    %rcx,%r12
   416df:	cmp    %rdi,0x98(%rsp)
   416e7:	jae    423d1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   416ed:	mov    0x20(%rsp),%r13
   416f2:	mov    $0x400,%edx
   416f7:	lea    0x28f8(%rsp),%rdi
   416ff:	xor    %esi,%esi
   41701:	call   *0xf2d19(%rip)        # 134420 <memset@GLIBC_2.2.5>
   41707:	lea    0x128(%rsp),%rax
   4170f:	xor    %ecx,%ecx
   41711:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41720:	mov    -0x40(%rax),%rdx
   41724:	sub    -0x30(%rax),%rdx
   41728:	shr    $1,%rdx
   4172b:	add    -0x28(%rax),%edx
   4172e:	mov    -0x18(%rax),%rsi
   41732:	sub    -0x8(%rax),%rsi
   41736:	mov    %edx,0x28f8(%rsp,%rcx,4)
   4173d:	shr    $1,%rsi
   41740:	add    (%rax),%esi
   41742:	mov    %esi,0x28fc(%rsp,%rcx,4)
   41749:	add    $0x2,%rcx
   4174d:	add    $0x50,%rax
   41751:	cmp    $0x100,%rcx
   41758:	jne    41720 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x820>
   4175a:	mov    0xc0(%rsp),%r15
   41762:	mov    0x10(%r15),%rbx
   41766:	cmp    (%r15),%rbx
   41769:	je     423c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14c4>
   4176f:	mov    %rbx,%rdi
   41772:	shl    $0xa,%rdi
   41776:	add    0x8(%r15),%rdi
   4177a:	mov    $0x400,%edx
   4177f:	lea    0x28f8(%rsp),%rsi
   41787:	call   *0xf2cb3(%rip)        # 134440 <memcpy@GLIBC_2.14>
   4178d:	inc    %rbx
   41790:	lea    0xf000(%r13),%rax
   41797:	mov    0x8(%rsp),%rdi
   4179c:	cmp    %rdi,%rax
   4179f:	mov    %rdi,%rcx
   417a2:	mov    %rax,0x98(%rsp)
   417aa:	cmovb  %rax,%rcx
   417ae:	mov    %rbx,0x10(%r15)
   417b2:	mov    %rcx,0x20(%rsp)
   417b7:	mov    %rcx,%r8
   417ba:	sub    %r13,%r8
   417bd:	cmp    $0x10,%r8
   417c1:	jb     421f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f0>
   417c7:	add    0x48(%rsp),%r13
   417cc:	xor    %eax,%eax
   417ce:	mov    %r12,%rcx
   417d1:	mov    %r8,%r12
   417d4:	jmp    41828 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   417d6:	cs nopw 0x0(%rax,%rax,1)
   417e0:	mov    0x28e8(%rsp),%rax
   417e8:	lea    0x10(%rax),%rcx
   417ec:	add    $0xfffffffffffffff2,%rax
   417f0:	xor    %edx,%edx
   417f2:	cmp    $0x1e,%rcx
   417f6:	setae  %dl
   417f9:	cmovb  %rcx,%rax
   417fd:	mov    %rax,0x28e8(%rsp)
   41805:	mov    0x28f0(%rsp),%eax
   4180c:	lea    (%rax,%rdx,2),%eax
   4180f:	add    $0x10,%eax
   41812:	mov    %eax,0x28f0(%rsp)
   41819:	xor    %eax,%eax
   4181b:	mov    %r8,%r12
   4181e:	cmp    $0xf,%r8
   41822:	jbe    421f3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   41828:	test   $0x1,%al
   4182a:	jne    41834 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x934>
   4182c:	mov    0x0(%r13),%rcx
   41830:	mov    0x8(%r13),%r14
   41834:	add    $0x10,%r13
   41838:	lea    -0x10(%r12),%r8
   4183d:	xor    %rcx,%r14
   41840:	mov    %rcx,%rax
   41843:	rol    $0x8,%rax
   41847:	xor    %rcx,%rax
   4184a:	or     %r14,%rax
   4184d:	lea    0xe8(%rsp),%rdx
   41855:	jne    418b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x9b0>
   41857:	cmp    $0x10,%r8
   4185b:	jb     41920 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa20>
   41861:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41870:	mov    0x0(%r13),%rdx
   41874:	mov    0x8(%r13),%rax
   41878:	mov    $0x1,%r9b
   4187b:	cmp    %rcx,%rdx
   4187e:	jne    41a72 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   41884:	cmp    %rcx,%rax
   41887:	jne    41a72 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   4188d:	add    $0xfffffffffffffff0,%r8
   41891:	add    $0x10,%r13
   41895:	cmp    $0xf,%r8
   41899:	ja     41870 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x970>
   4189b:	mov    %r13,0x28(%rsp)
   418a0:	xor    %r9d,%r9d
   418a3:	mov    %rcx,0x40(%rsp)
   418a8:	jmp    41a81 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   418ad:	nopl   (%rax)
   418b0:	mov    0x20(%rsp),%r14
   418b5:	sub    %r12,%r14
   418b8:	mov    0x28e8(%rsp),%rax
   418c0:	mov    $0x1e,%r9d
   418c6:	sub    %rax,%r9
   418c9:	cmp    $0x10,%r9
   418cd:	mov    $0x10,%ebx
   418d2:	cmovb  %r9,%rbx
   418d6:	mov    %rbx,%r15
   418d9:	add    %r14,%r15
   418dc:	jb     42655 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   418e2:	cmp    %rdi,%r15
   418e5:	ja     42655 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   418eb:	mov    0x28f0(%rsp),%r12d
   418f3:	cmp    $0x1e,%rax
   418f7:	jne    41930 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa30>
   418f9:	add    $0x10,%r14
   418fd:	cmp    %rdi,%r14
   41900:	ja     4266a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41906:	cmp    %r15,%r14
   41909:	jae    419f5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xaf5>
   4190f:	jmp    4266a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41914:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   41920:	mov    %r13,0x28(%rsp)
   41925:	xor    %r9d,%r9d
   41928:	jmp    41a81 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   4192d:	nopl   (%rax)
   41930:	mov    %r13,0x28(%rsp)
   41935:	mov    0x48(%rsp),%rax
   4193a:	lea    (%rax,%r14,1),%r10
   4193e:	xor    %r13d,%r13d
   41941:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41950:	movzbl (%r10,%r13,1),%esi
   41955:	lea    (%rsi,%rsi,4),%rcx
   41959:	lea    (%rdx,%rcx,8),%rbp
   4195d:	mov    (%rdx,%rcx,8),%rax
   41961:	cmp    0x8(%rdx,%rcx,8),%rax
   41966:	je     4197e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa7e>
   41968:	lea    (%r12,%r13,1),%rcx
   4196c:	mov    %cx,(%rax)
   4196f:	addq   $0x2,0x0(%rbp)
   41974:	inc    %r13
   41977:	cmp    %r13,%rbx
   4197a:	jne    41950 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa50>
   4197c:	jmp    419d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xad0>
   4197e:	mov    %r10,0x30(%rsp)
   41983:	mov    %r9,0x18(%rsp)
   41988:	mov    %r8,0x50(%rsp)
   4198d:	lea    0xa8(%rsp),%rdi
   41995:	call   8bd40 <<c30walk::ctor2s::W<false>>::refill>
   4199a:	test   %al,%al
   4199c:	je     4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   419a2:	mov    0x0(%rbp),%rax
   419a6:	mov    0x8(%rsp),%rdi
   419ab:	lea    0xe8(%rsp),%rdx
   419b3:	mov    0x50(%rsp),%r8
   419b8:	mov    0x18(%rsp),%r9
   419bd:	mov    0x30(%rsp),%r10
   419c2:	jmp    41968 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa68>
   419c4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   419d0:	add    $0x10,%r14
   419d4:	cmp    %rdi,%r14
   419d7:	ja     4266a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   419dd:	cmp    %r15,%r14
   419e0:	mov    0x28(%rsp),%r13
   419e5:	jb     4266a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   419eb:	cmp    $0xf,%r9
   419ef:	ja     417e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   419f5:	add    0x48(%rsp),%r15
   419fa:	mov    %r15,%r14
   419fd:	sub    %rbx,%r14
   41a00:	add    $0x10,%r14
   41a04:	add    %r12d,%ebx
   41a07:	add    $0x2,%ebx
   41a0a:	nopw   0x0(%rax,%rax,1)
   41a10:	movzbl (%r15),%esi
   41a14:	lea    (%rsi,%rsi,4),%rcx
   41a18:	lea    (%rdx,%rcx,8),%r12
   41a1c:	mov    (%rdx,%rcx,8),%rax
   41a20:	cmp    0x8(%rdx,%rcx,8),%rax
   41a25:	je     41a3e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb3e>
   41a27:	mov    %bx,(%rax)
   41a2a:	addq   $0x2,(%r12)
   41a2f:	inc    %r15
   41a32:	inc    %ebx
   41a34:	cmp    %r14,%r15
   41a37:	jne    41a10 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb10>
   41a39:	jmp    417e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   41a3e:	mov    %r13,%rbp
   41a41:	mov    %r8,%r13
   41a44:	lea    0xa8(%rsp),%rdi
   41a4c:	call   8bd40 <<c30walk::ctor2s::W<false>>::refill>
   41a51:	test   %al,%al
   41a53:	je     4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   41a59:	mov    (%r12),%rax
   41a5d:	mov    0x8(%rsp),%rdi
   41a62:	lea    0xe8(%rsp),%rdx
   41a6a:	mov    %r13,%r8
   41a6d:	mov    %rbp,%r13
   41a70:	jmp    41a27 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb27>
   41a72:	mov    %rdx,0x40(%rsp)
   41a77:	mov    %r13,0x28(%rsp)
   41a7c:	mov    %rax,0x38(%rsp)
   41a81:	mov    %r8,0x50(%rsp)
   41a86:	sub    %r8,%r12
   41a89:	movzbl %cl,%eax
   41a8c:	mov    %rax,0xa0(%rsp)
   41a94:	lea    (%rax,%rax,4),%rdx
   41a98:	lea    0xe8(%rsp),%r8
   41aa0:	mov    (%r8,%rdx,8),%rax
   41aa4:	mov    0x1c(%r8,%rdx,8),%esi
   41aa9:	mov    %rax,%rdi
   41aac:	sub    0x10(%r8,%rdx,8),%rdi
   41ab1:	shr    $1,%rdi
   41ab4:	add    0x18(%r8,%rdx,8),%edi
   41ab9:	add    %r12,%rdi
   41abc:	cmp    %rsi,%rdi
   41abf:	ja     4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   41ac5:	mov    %r9d,0x64(%rsp)
   41aca:	lea    0xe8(%rsp),%rsi
   41ad2:	lea    (%rsi,%rdx,8),%rbx
   41ad6:	movzbl %cl,%r14d
   41ada:	mov    0x8(%rbx),%rsi
   41ade:	mov    $0x88888889,%r15d
   41ae4:	lea    -0x2682b(%rip),%r13        # 1b2c0 <c30walk::ctor2s::PATTERNS>
   41aeb:	mov    %rbx,0x80(%rsp)
   41af3:	jmp    41b2e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc2e>
   41af5:	data16 cs nopw 0x0(%rax,%rax,1)
   41b00:	mov    0x3c(%rsi),%edi
   41b03:	mov    $0xfffff,%r9d
   41b09:	and    %r9d,%edi
   41b0c:	sub    %r8d,%ecx
   41b0f:	sub    %eax,%r8d
   41b12:	mov    %r8d,%eax
   41b15:	lea    (%rsi,%rax,2),%rax
   41b19:	mov    %rax,0x10(%rbx)
   41b1d:	mov    %rax,(%rbx)
   41b20:	lea    (%rax,%rcx,2),%rsi
   41b24:	mov    %rsi,0x8(%rbx)
   41b28:	mov    %edx,0x20(%rbx)
   41b2b:	mov    %edi,0x24(%rbx)
   41b2e:	sub    %rax,%rsi
   41b31:	shr    $1,%rsi
   41b34:	cmp    %rsi,%r12
   41b37:	cmovb  %r12,%rsi
   41b3b:	test   %rsi,%rsi
   41b3e:	je     41bf8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcf8>
   41b44:	mov    0x28e8(%rsp),%rdi
   41b4c:	cmp    $0x1d,%rdi
   41b50:	ja     42769 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1869>
   41b56:	cmp    $0x1f,%rsi
   41b5a:	jae    42451 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1551>
   41b60:	mov    0x28f0(%rsp),%ecx
   41b67:	mov    %rdi,%r11
   41b6a:	shl    $0x6,%r11
   41b6e:	lea    (%r11,%r13,1),%rdx
   41b72:	lea    -0x2(,%rsi,2),%r9
   41b7a:	cmp    $0xe,%r9
   41b7e:	jb     41b93 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc93>
   41b80:	mov    %rax,%r8
   41b83:	sub    %r13,%r8
   41b86:	sub    %r11,%r8
   41b89:	cmp    $0x10,%r8
   41b8d:	jae    41fd8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x10d8>
   41b93:	mov    %rdx,%r8
   41b96:	xor    %r10d,%r10d
   41b99:	lea    (%rdx,%rsi,2),%rdx
   41b9d:	lea    (%rax,%r10,2),%r9
   41ba1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41bb0:	movzwl (%r8),%r10d
   41bb4:	add    %cx,%r10w
   41bb8:	add    $0x2,%r8
   41bbc:	mov    %r10w,(%r9)
   41bc0:	add    $0x2,%r9
   41bc4:	cmp    %rdx,%r8
   41bc7:	jne    41bb0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcb0>
   41bc9:	lea    (%rax,%rsi,2),%rax
   41bcd:	mov    %rax,(%rbx)
   41bd0:	add    %rsi,%rdi
   41bd3:	lea    0x2(%rsi),%edx
   41bd6:	mov    %rdi,%r8
   41bd9:	sub    $0x1e,%r8
   41bdd:	cmovb  %rdi,%r8
   41be1:	cmovb  %esi,%edx
   41be4:	mov    %r8,0x28e8(%rsp)
   41bec:	add    %ecx,%edx
   41bee:	mov    %edx,0x28f0(%rsp)
   41bf5:	sub    %rsi,%r12
   41bf8:	test   %r12,%r12
   41bfb:	je     421c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   41c01:	mov    %rax,%r8
   41c04:	cmp    0x10(%rbx),%rax
   41c08:	je     41c26 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xd26>
   41c0a:	lea    0xa8(%rsp),%rdi
   41c12:	mov    0xa0(%rsp),%rsi
   41c1a:	call   8b740 <<c30walk::ctor2s::W<false>>::finish>
   41c1f:	mov    (%rbx),%r8
   41c22:	mov    0x10(%rbx),%rax
   41c26:	sub    %rax,%r8
   41c29:	shr    $1,%r8
   41c2c:	add    0x18(%rbx),%r8d
   41c30:	mov    %r8d,0x18(%rbx)
   41c34:	pxor   %xmm0,%xmm0
   41c38:	movdqu %xmm0,(%rbx)
   41c3c:	movq   $0x0,0x10(%rbx)
   41c44:	cmp    $0x1e,%r12
   41c48:	jb     42150 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   41c4e:	mov    %r8,%rcx
   41c51:	imul   %r15,%rcx
   41c55:	shr    $0x24,%rcx
   41c59:	mov    %ecx,%eax
   41c5b:	shl    $0x5,%eax
   41c5e:	lea    (%rcx,%rcx,1),%edx
   41c61:	sub    %eax,%edx
   41c63:	add    %r8d,%edx
   41c66:	jne    42150 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   41c6c:	mov    %r12,%rax
   41c6f:	movabs $0x8888888888888889,%rdx
   41c79:	mul    %rdx
   41c7c:	shr    $0x4,%rdx
   41c80:	mov    %ecx,%r9d
   41c83:	cmpl   $0x0,0x14(%rsp)
   41c88:	mov    %r12,0x58(%rsp)
   41c8d:	mov    %rdx,0x68(%rsp)
   41c92:	je     41f90 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1090>
   41c98:	mov    0x28f0(%rsp),%r10d
   41ca0:	mov    0xb8(%rsp),%rax
   41ca8:	mov    %rax,0x18(%rsp)
   41cad:	mov    0xa8(%rsp),%r11
   41cb5:	mov    0xb0(%rsp),%rax
   41cbd:	mov    %rax,0x30(%rsp)
   41cc2:	mov    0xd0(%rsp),%rax
   41cca:	mov    0x28e8(%rsp),%r12
   41cd2:	mov    %r12,%rdi
   41cd5:	shl    $0x6,%rdi
   41cd9:	add    %r13,%rdi
   41cdc:	mov    %r9,%rcx
   41cdf:	shl    $0x6,%rcx
   41ce3:	lea    (%rax,%rcx,1),%rbx
   41ce7:	add    $0x3c,%rbx
   41ceb:	mov    %r10d,%r13d
   41cee:	shl    $0x8,%r13d
   41cf2:	or     %r14d,%r13d
   41cf5:	xor    %r15d,%r15d
   41cf8:	jmp    41f04 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1004>
   41cfd:	nopl   (%rax)
   41d00:	mov    0x8(%r11),%rcx
   41d04:	shl    $0x7,%eax
   41d07:	mov    %r13d,(%rcx,%rax,1)
   41d0b:	movzwl 0x2(%rdi),%esi
   41d0f:	lea    (%r10,%r15,1),%ebp
   41d13:	add    %ebp,%esi
   41d15:	shl    $0x8,%esi
   41d18:	or     %r14d,%esi
   41d1b:	mov    %esi,0x4(%rcx,%rax,1)
   41d1f:	movzwl 0x4(%rdi),%esi
   41d23:	add    %ebp,%esi
   41d25:	shl    $0x8,%esi
   41d28:	or     %r14d,%esi
   41d2b:	mov    %esi,0x8(%rcx,%rax,1)
   41d2f:	movzwl 0x6(%rdi),%esi
   41d33:	add    %ebp,%esi
   41d35:	shl    $0x8,%esi
   41d38:	or     %r14d,%esi
   41d3b:	mov    %esi,0xc(%rcx,%rax,1)
   41d3f:	movzwl 0x8(%rdi),%esi
   41d43:	add    %ebp,%esi
   41d45:	shl    $0x8,%esi
   41d48:	or     %r14d,%esi
   41d4b:	mov    %esi,0x10(%rcx,%rax,1)
   41d4f:	movzwl 0xa(%rdi),%esi
   41d53:	add    %ebp,%esi
   41d55:	shl    $0x8,%esi
   41d58:	or     %r14d,%esi
   41d5b:	mov    %esi,0x14(%rcx,%rax,1)
   41d5f:	movzwl 0xc(%rdi),%esi
   41d63:	add    %ebp,%esi
   41d65:	shl    $0x8,%esi
   41d68:	or     %r14d,%esi
   41d6b:	mov    %esi,0x18(%rcx,%rax,1)
   41d6f:	movzwl 0xe(%rdi),%esi
   41d73:	add    %ebp,%esi
   41d75:	shl    $0x8,%esi
   41d78:	or     %r14d,%esi
   41d7b:	mov    %esi,0x1c(%rcx,%rax,1)
   41d7f:	movzwl 0x10(%rdi),%esi
   41d83:	add    %ebp,%esi
   41d85:	shl    $0x8,%esi
   41d88:	or     %r14d,%esi
   41d8b:	mov    %esi,0x20(%rcx,%rax,1)
   41d8f:	movzwl 0x12(%rdi),%esi
   41d93:	add    %ebp,%esi
   41d95:	shl    $0x8,%esi
   41d98:	or     %r14d,%esi
   41d9b:	mov    %esi,0x24(%rcx,%rax,1)
   41d9f:	movzwl 0x14(%rdi),%esi
   41da3:	add    %ebp,%esi
   41da5:	shl    $0x8,%esi
   41da8:	or     %r14d,%esi
   41dab:	mov    %esi,0x28(%rcx,%rax,1)
   41daf:	movzwl 0x16(%rdi),%esi
   41db3:	add    %ebp,%esi
   41db5:	shl    $0x8,%esi
   41db8:	or     %r14d,%esi
   41dbb:	mov    %esi,0x2c(%rcx,%rax,1)
   41dbf:	movzwl 0x18(%rdi),%esi
   41dc3:	add    %ebp,%esi
   41dc5:	shl    $0x8,%esi
   41dc8:	or     %r14d,%esi
   41dcb:	mov    %esi,0x30(%rcx,%rax,1)
   41dcf:	movzwl 0x1a(%rdi),%esi
   41dd3:	add    %ebp,%esi
   41dd5:	shl    $0x8,%esi
   41dd8:	or     %r14d,%esi
   41ddb:	mov    %esi,0x34(%rcx,%rax,1)
   41ddf:	movzwl 0x1c(%rdi),%esi
   41de3:	add    %ebp,%esi
   41de5:	shl    $0x8,%esi
   41de8:	or     %r14d,%esi
   41deb:	mov    %esi,0x38(%rcx,%rax,1)
   41def:	movzwl 0x1e(%rdi),%esi
   41df3:	add    %ebp,%esi
   41df5:	shl    $0x8,%esi
   41df8:	or     %r14d,%esi
   41dfb:	mov    %esi,0x3c(%rcx,%rax,1)
   41dff:	movzwl 0x20(%rdi),%esi
   41e03:	add    %ebp,%esi
   41e05:	shl    $0x8,%esi
   41e08:	or     %r14d,%esi
   41e0b:	mov    %esi,0x40(%rcx,%rax,1)
   41e0f:	movzwl 0x22(%rdi),%esi
   41e13:	add    %ebp,%esi
   41e15:	shl    $0x8,%esi
   41e18:	or     %r14d,%esi
   41e1b:	mov    %esi,0x44(%rcx,%rax,1)
   41e1f:	movzwl 0x24(%rdi),%esi
   41e23:	add    %ebp,%esi
   41e25:	shl    $0x8,%esi
   41e28:	or     %r14d,%esi
   41e2b:	mov    %esi,0x48(%rcx,%rax,1)
   41e2f:	movzwl 0x26(%rdi),%esi
   41e33:	add    %ebp,%esi
   41e35:	shl    $0x8,%esi
   41e38:	or     %r14d,%esi
   41e3b:	mov    %esi,0x4c(%rcx,%rax,1)
   41e3f:	movzwl 0x28(%rdi),%esi
   41e43:	add    %ebp,%esi
   41e45:	shl    $0x8,%esi
   41e48:	or     %r14d,%esi
   41e4b:	mov    %esi,0x50(%rcx,%rax,1)
   41e4f:	movzwl 0x2a(%rdi),%esi
   41e53:	add    %ebp,%esi
   41e55:	shl    $0x8,%esi
   41e58:	or     %r14d,%esi
   41e5b:	mov    %esi,0x54(%rcx,%rax,1)
   41e5f:	movzwl 0x2c(%rdi),%esi
   41e63:	add    %ebp,%esi
   41e65:	shl    $0x8,%esi
   41e68:	or     %r14d,%esi
   41e6b:	mov    %esi,0x58(%rcx,%rax,1)
   41e6f:	movzwl 0x2e(%rdi),%esi
   41e73:	add    %ebp,%esi
   41e75:	shl    $0x8,%esi
   41e78:	or     %r14d,%esi
   41e7b:	mov    %esi,0x5c(%rcx,%rax,1)
   41e7f:	movzwl 0x30(%rdi),%esi
   41e83:	add    %ebp,%esi
   41e85:	shl    $0x8,%esi
   41e88:	or     %r14d,%esi
   41e8b:	mov    %esi,0x60(%rcx,%rax,1)
   41e8f:	movzwl 0x32(%rdi),%esi
   41e93:	add    %ebp,%esi
   41e95:	shl    $0x8,%esi
   41e98:	or     %r14d,%esi
   41e9b:	mov    %esi,0x64(%rcx,%rax,1)
   41e9f:	movzwl 0x34(%rdi),%esi
   41ea3:	add    %ebp,%esi
   41ea5:	shl    $0x8,%esi
   41ea8:	or     %r14d,%esi
   41eab:	mov    %esi,0x68(%rcx,%rax,1)
   41eaf:	movzwl 0x36(%rdi),%esi
   41eb3:	add    %ebp,%esi
   41eb5:	shl    $0x8,%esi
   41eb8:	or     %r14d,%esi
   41ebb:	mov    %esi,0x6c(%rcx,%rax,1)
   41ebf:	movzwl 0x38(%rdi),%esi
   41ec3:	add    %ebp,%esi
   41ec5:	shl    $0x8,%esi
   41ec8:	or     %r14d,%esi
   41ecb:	mov    %esi,0x70(%rcx,%rax,1)
   41ecf:	movzwl 0x3a(%rdi),%esi
   41ed3:	add    %ebp,%esi
   41ed5:	shl    $0x8,%esi
   41ed8:	or     %r14d,%esi
   41edb:	mov    %esi,0x74(%rcx,%rax,1)
   41edf:	lea    (%r10,%r15,1),%eax
   41ee3:	add    $0x20,%eax
   41ee6:	mov    %eax,0x28f0(%rsp)
   41eed:	add    $0x20,%r15
   41ef1:	inc    %r9
   41ef4:	add    $0x2000,%r13d
   41efb:	dec    %rdx
   41efe:	je     4210c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120c>
   41f04:	mov    %r9,%rax
   41f07:	shr    $0x6,%rax
   41f0b:	cmp    0x18(%rsp),%rax
   41f10:	jae    4267f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x177f>
   41f16:	mov    0x30(%rsp),%rcx
   41f1b:	mov    (%rcx,%rax,8),%rax
   41f1f:	bt     %r9,%rax
   41f23:	jae    41f50 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1050>
   41f25:	cmp    $0x1e,%r12
   41f29:	jae    426a5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17a5>
   41f2f:	mov    (%rbx,%r15,2),%eax
   41f33:	and    $0xfffff,%eax
   41f38:	mov    0x10(%r11),%rcx
   41f3c:	cmp    %rax,%rcx
   41f3f:	ja     41d00 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xe00>
   41f45:	jmp    42693 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1793>
   41f4a:	nopw   0x0(%rax,%rax,1)
   41f50:	cmp    $0x1e,%r12
   41f54:	jae    426e5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17e5>
   41f5a:	movups 0x2c(%rdi),%xmm0
   41f5e:	movups %xmm0,-0x10(%rbx,%r15,2)
   41f64:	movdqa (%rdi),%xmm0
   41f68:	movdqa 0x10(%rdi),%xmm1
   41f6d:	movdqa 0x20(%rdi),%xmm2
   41f72:	movdqa %xmm2,-0x1c(%rbx,%r15,2)
   41f79:	movdqa %xmm1,-0x2c(%rbx,%r15,2)
   41f80:	movdqa %xmm0,-0x3c(%rbx,%r15,2)
   41f87:	mov    %r13d,(%rbx,%r15,2)
   41f8b:	jmp    41edf <<c30walk::ctor2s::Ctor2s>::build::<false>+0xfdf>
   41f90:	mov    0x28e8(%rsp),%rdi
   41f98:	cmp    $0x1d,%rdi
   41f9c:	ja     4277a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x187a>
   41fa2:	shl    $0x6,%rdi
   41fa6:	add    %r13,%rdi
   41fa9:	mov    0x28f0(%rsp),%eax
   41fb0:	mov    0xd0(%rsp),%rcx
   41fb8:	shl    $0x6,%r9
   41fbc:	mov    %eax,%r15d
   41fbf:	shl    $0x8,%r15d
   41fc3:	or     %r14d,%r15d
   41fc6:	lea    -0x1e(%r12),%rsi
   41fcb:	cmp    $0x1e,%rsi
   41fcf:	jae    4203d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x113d>
   41fd1:	xor    %esi,%esi
   41fd3:	jmp    420ca <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11ca>
   41fd8:	shr    $1,%r9
   41fdb:	inc    %r9
   41fde:	mov    %r9,%r10
   41fe1:	and    $0xfffffffffffffff8,%r10
   41fe5:	lea    (%rdx,%r10,2),%r8
   41fe9:	movd   %ecx,%xmm0
   41fed:	pshuflw $0x0,%xmm0,%xmm0
   41ff2:	add    %r13,%r11
   41ff5:	add    $0x8,%r11
   41ff9:	xor    %ebx,%ebx
   41ffb:	nopl   0x0(%rax,%rax,1)
   42000:	movq   -0x8(%r11,%rbx,2),%xmm1
   42007:	movq   (%r11,%rbx,2),%xmm2
   4200d:	paddw  %xmm0,%xmm1
   42011:	paddw  %xmm0,%xmm2
   42015:	punpcklqdq %xmm2,%xmm1
   42019:	movdqu %xmm1,(%rax,%rbx,2)
   4201e:	add    $0x8,%rbx
   42022:	cmp    %rbx,%r10
   42025:	jne    42000 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1100>
   42027:	cmp    %r10,%r9
   4202a:	mov    0x80(%rsp),%rbx
   42032:	jne    41b99 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc99>
   42038:	jmp    41bc9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcc9>
   4203d:	mov    %rdx,%r10
   42040:	movabs $0xffffffffffffffe,%rsi
   4204a:	and    %rsi,%r10
   4204d:	lea    (%rcx,%r9,1),%r11
   42051:	add    $0x7c,%r11
   42055:	xor    %esi,%esi
   42057:	nopw   0x0(%rax,%rax,1)
   42060:	movdqu 0x2c(%rdi),%xmm0
   42065:	movdqu %xmm0,-0x50(%r11)
   4206b:	movdqa (%rdi),%xmm1
   4206f:	movdqa 0x10(%rdi),%xmm2
   42074:	movdqa 0x20(%rdi),%xmm3
   42079:	movdqa %xmm3,-0x5c(%r11)
   4207f:	movdqa %xmm2,-0x6c(%r11)
   42085:	movdqa %xmm1,-0x7c(%r11)
   4208b:	mov    %r15d,-0x40(%r11)
   4208f:	add    $0x2,%rsi
   42093:	movdqa %xmm1,-0x3c(%r11)
   42099:	movdqa %xmm2,-0x2c(%r11)
   4209f:	movdqa %xmm3,-0x1c(%r11)
   420a5:	movdqu %xmm0,-0x10(%r11)
   420ab:	lea    0x2000(%r15),%ebx
   420b2:	mov    %ebx,(%r11)
   420b5:	add    $0x4000,%r15d
   420bc:	sub    $0xffffffffffffff80,%r11
   420c0:	cmp    %rsi,%r10
   420c3:	jne    42060 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1160>
   420c5:	test   $0x1,%dl
   420c8:	je     420fe <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11fe>
   420ca:	add    %r9,%rcx
   420cd:	shl    $0x6,%rsi
   420d1:	movups 0x2c(%rdi),%xmm0
   420d5:	movups %xmm0,0x2c(%rcx,%rsi,1)
   420da:	movdqa (%rdi),%xmm0
   420de:	movdqa 0x10(%rdi),%xmm1
   420e3:	movdqa 0x20(%rdi),%xmm2
   420e8:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   420ee:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   420f4:	movdqa %xmm0,(%rcx,%rsi,1)
   420f9:	mov    %r15d,0x3c(%rcx,%rsi,1)
   420fe:	mov    %edx,%ecx
   42100:	shl    $0x5,%ecx
   42103:	add    %ecx,%eax
   42105:	mov    %eax,0x28f0(%rsp)
   4210c:	mov    0x68(%rsp),%rdx
   42111:	mov    %rdx,%rax
   42114:	shl    $0x5,%rax
   42118:	lea    (%rdx,%rdx,1),%rcx
   4211c:	sub    %rax,%rcx
   4211f:	mov    0x58(%rsp),%r12
   42124:	add    %rcx,%r12
   42127:	lea    (%rdx,%rdx,1),%eax
   4212a:	shl    $0x5,%edx
   4212d:	sub    %eax,%edx
   4212f:	add    %edx,%r8d
   42132:	mov    0x80(%rsp),%rbx
   4213a:	mov    %r8d,0x18(%rbx)
   4213e:	test   %r12,%r12
   42141:	mov    $0x88888889,%r15d
   42147:	lea    -0x26e8e(%rip),%r13        # 1b2c0 <c30walk::ctor2s::PATTERNS>
   4214e:	je     421c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   42150:	mov    0x1c(%rbx),%ecx
   42153:	mov    %r8d,%edi
   42156:	imul   %r15,%rdi
   4215a:	mov    %rdi,%rdx
   4215d:	shr    $0x24,%rdx
   42161:	mov    %edx,%eax
   42163:	shl    $0x5,%eax
   42166:	sub    %edx,%eax
   42168:	sub    %edx,%eax
   4216a:	lea    0x1e(%rax),%esi
   4216d:	cmp    %ecx,%esi
   4216f:	cmovb  %esi,%ecx
   42172:	shr    $0x2a,%rdi
   42176:	mov    0xb8(%rsp),%rsi
   4217e:	cmp    %rsi,%rdi
   42181:	jae    4275d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x185d>
   42187:	mov    %rdx,%rsi
   4218a:	shl    $0x6,%rsi
   4218e:	add    0xd0(%rsp),%rsi
   42196:	mov    0xb0(%rsp),%r9
   4219e:	mov    (%r9,%rdi,8),%rdi
   421a2:	bt     %rdx,%rdi
   421a6:	jb     41b00 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc00>
   421ac:	mov    $0xffffffff,%edi
   421b1:	jmp    41b0c <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc0c>
   421b6:	cs nopw 0x0(%rax,%rax,1)
   421c0:	mov    0x8(%rsp),%rdi
   421c5:	mov    0x50(%rsp),%r8
   421ca:	mov    0x38(%rsp),%r14
   421cf:	mov    0x28(%rsp),%r13
   421d4:	mov    0x40(%rsp),%rcx
   421d9:	mov    0x64(%rsp),%eax
   421dd:	mov    %r8,%r12
   421e0:	cmp    $0xf,%r8
   421e4:	ja     41828 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   421ea:	jmp    421f3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   421ec:	nopl   0x0(%rax)
   421f0:	mov    %r12,%rcx
   421f3:	test   %r8,%r8
   421f6:	je     416dc <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7dc>
   421fc:	mov    0x20(%rsp),%r15
   42201:	sub    %r8,%r15
   42204:	mov    0x28e8(%rsp),%rax
   4220c:	mov    $0x1e,%ebx
   42211:	sub    %rax,%rbx
   42214:	cmp    %r8,%rbx
   42217:	cmovae %r8,%rbx
   4221b:	mov    %rbx,%r12
   4221e:	add    %r15,%r12
   42221:	lea    0xe8(%rsp),%rdx
   42229:	jb     426b9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   4222f:	cmp    %rdi,%r12
   42232:	ja     426b9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   42238:	mov    %rcx,0x40(%rsp)
   4223d:	mov    %r14,0x38(%rsp)
   42242:	mov    0x28f0(%rsp),%r14d
   4224a:	cmp    $0x1e,%rax
   4224e:	jne    422df <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13df>
   42254:	cmp    %r12,0x20(%rsp)
   42259:	jb     426ce <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17ce>
   4225f:	jne    42353 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1453>
   42265:	mov    0x28e8(%rsp),%rax
   4226d:	mov    0x28f0(%rsp),%ecx
   42274:	cmp    $0x1,%r8
   42278:	je     416a8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7a8>
   4227e:	mov    %r8d,%edx
   42281:	and    $0xe,%edx
   42284:	mov    0x38(%rsp),%r14
   42289:	nopl   0x0(%rax)
   42290:	lea    0x1(%rax),%rsi
   42294:	add    $0xffffffffffffffe3,%rax
   42298:	xor    %edi,%edi
   4229a:	cmp    $0x1e,%rsi
   4229e:	setae  %dil
   422a2:	cmovb  %rsi,%rax
   422a6:	lea    (%rcx,%rdi,2),%ecx
   422a9:	lea    0x1(%rax),%rsi
   422ad:	add    $0xffffffffffffffe3,%rax
   422b1:	xor    %edi,%edi
   422b3:	cmp    $0x1e,%rsi
   422b7:	setae  %dil
   422bb:	cmovb  %rsi,%rax
   422bf:	lea    (%rcx,%rdi,2),%ecx
   422c2:	add    $0x2,%ecx
   422c5:	add    $0xfffffffffffffffe,%rdx
   422c9:	jne    42290 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1390>
   422cb:	test   $0x1,%r8b
   422cf:	mov    0x8(%rsp),%rdi
   422d4:	jne    416ad <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ad>
   422da:	jmp    416c8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7c8>
   422df:	add    0x48(%rsp),%r15
   422e4:	xor    %r13d,%r13d
   422e7:	nopw   0x0(%rax,%rax,1)
   422f0:	movzbl (%r15,%r13,1),%esi
   422f5:	lea    (%rsi,%rsi,4),%rcx
   422f9:	lea    (%rdx,%rcx,8),%rbp
   422fd:	mov    (%rdx,%rcx,8),%rax
   42301:	cmp    0x8(%rdx,%rcx,8),%rax
   42306:	je     42321 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1421>
   42308:	lea    (%r14,%r13,1),%rcx
   4230c:	mov    %cx,(%rax)
   4230f:	addq   $0x2,0x0(%rbp)
   42314:	inc    %r13
   42317:	cmp    %r13,%rbx
   4231a:	jne    422f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13f0>
   4231c:	jmp    42254 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1354>
   42321:	mov    %r8,0x50(%rsp)
   42326:	lea    0xa8(%rsp),%rdi
   4232e:	call   8bd40 <<c30walk::ctor2s::W<false>>::refill>
   42333:	test   %al,%al
   42335:	je     4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   4233b:	mov    0x0(%rbp),%rax
   4233f:	mov    0x8(%rsp),%rdi
   42344:	lea    0xe8(%rsp),%rdx
   4234c:	mov    0x50(%rsp),%r8
   42351:	jmp    42308 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1408>
   42353:	mov    0x48(%rsp),%rax
   42358:	mov    0x20(%rsp),%rcx
   4235d:	lea    (%rax,%rcx,1),%r15
   42361:	lea    (%r14,%rbx,1),%ebp
   42365:	add    $0x2,%ebp
   42368:	sub    %r8,%rbx
   4236b:	nopl   0x0(%rax,%rax,1)
   42370:	movzbl (%r15,%rbx,1),%esi
   42375:	lea    (%rsi,%rsi,4),%rcx
   42379:	lea    (%rdx,%rcx,8),%r14
   4237d:	mov    (%rdx,%rcx,8),%rax
   42381:	cmp    0x8(%rdx,%rcx,8),%rax
   42386:	je     4239b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x149b>
   42388:	mov    %bp,(%rax)
   4238b:	addq   $0x2,(%r14)
   4238f:	inc    %ebp
   42391:	inc    %rbx
   42394:	jne    42370 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1470>
   42396:	jmp    42265 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1365>
   4239b:	mov    %r8,%r12
   4239e:	lea    0xa8(%rsp),%rdi
   423a6:	call   8bd40 <<c30walk::ctor2s::W<false>>::refill>
   423ab:	test   %al,%al
   423ad:	je     4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   423af:	mov    (%r14),%rax
   423b2:	mov    0x8(%rsp),%rdi
   423b7:	lea    0xe8(%rsp),%rdx
   423bf:	mov    %r12,%r8
   423c2:	jmp    42388 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1488>
   423c4:	mov    %r15,%rdi
   423c7:	call   e6310 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   423cc:	jmp    4176f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x86f>
   423d1:	mov    $0x84,%eax
   423d6:	cs nopw 0x0(%rax,%rax,1)
   423e0:	lea    0x50(%rax),%rcx
   423e4:	cmp    $0x28d4,%rcx
   423eb:	je     42464 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1564>
   423ed:	mov    0x64(%rsp,%rax,1),%rdx
   423f2:	sub    0x74(%rsp,%rax,1),%rdx
   423f7:	shr    $1,%rdx
   423fa:	add    0x7c(%rsp,%rax,1),%edx
   423fe:	cmp    0x80(%rsp,%rax,1),%edx
   42405:	jne    4242d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   42407:	mov    0x8c(%rsp,%rax,1),%rdx
   4240f:	sub    0x9c(%rsp,%rax,1),%rdx
   42417:	shr    $1,%rdx
   4241a:	add    0xa4(%rsp,%rax,1),%edx
   42421:	cmp    0xa8(%rsp,%rax,1),%edx
   42428:	mov    %rcx,%rax
   4242b:	je     423e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14e0>
   4242d:	mov    0x5530(%rsp),%rax
   42435:	movq   $0x0,0x10(%rax)
   4243d:	xor    %eax,%eax
   4243f:	add    $0x54f8,%rsp
   42446:	pop    %rbx
   42447:	pop    %r12
   42449:	pop    %r13
   4244b:	pop    %r14
   4244d:	pop    %r15
   4244f:	pop    %rbp
   42450:	ret
   42451:	lea    0xedd08(%rip),%rcx        # 130160 <__frame_dummy_init_array_entry+0x2060>
   42458:	mov    $0x1e,%edx
   4245d:	xor    %edi,%edi
   4245f:	call   d84f0 <core::slice::index::slice_index_fail>
   42464:	xor    %r14d,%r14d
   42467:	lea    0xa8(%rsp),%r15
   4246f:	mov    0x88(%rsp),%r12
   42477:	mov    0x8(%rsp),%rdx
   4247c:	mov    0x78(%rsp),%r13
   42481:	lea    0xe8(%rsp),%rbx
   42489:	jmp    424b9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15b9>
   4248b:	nopl   0x0(%rax,%rax,1)
   42490:	sub    %rcx,%rax
   42493:	inc    %r14
   42496:	shr    $1,%rax
   42499:	add    %eax,0x18(%rbx)
   4249c:	pxor   %xmm0,%xmm0
   424a0:	movdqu %xmm0,(%rbx)
   424a4:	movq   $0x0,0x10(%rbx)
   424ac:	add    $0x28,%rbx
   424b0:	cmp    $0x100,%r14
   424b7:	je     424de <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15de>
   424b9:	mov    (%rbx),%rcx
   424bc:	mov    %rcx,%rax
   424bf:	cmp    0x10(%rbx),%rcx
   424c3:	je     42490 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   424c5:	mov    %r15,%rdi
   424c8:	mov    %r14,%rsi
   424cb:	call   8b740 <<c30walk::ctor2s::W<false>>::finish>
   424d0:	mov    0x8(%rsp),%rdx
   424d5:	mov    (%rbx),%rax
   424d8:	mov    0x10(%rbx),%rcx
   424dc:	jmp    42490 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   424de:	xor    %ecx,%ecx
   424e0:	mov    0x8(%r12),%rbx
   424e5:	test   %r13,%r13
   424e8:	setne  %al
   424eb:	je     4252f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   424ed:	mov    %al,%cl
   424ef:	mov    %r13,%rax
   424f2:	sub    %rcx,%rax
   424f5:	imul   $0xffffffffffffffe2,%rax,%rcx
   424f9:	add    %rdx,%rcx
   424fc:	cmp    $0x1e,%rcx
   42500:	jae    4252f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   42502:	lea    (%rbx,%r13,4),%rcx
   42506:	lea    (%rcx,%rdx,2),%rdi
   4250a:	add    $0xfffffffffffffffc,%rdi
   4250e:	lea    (%rax,%rax,1),%rcx
   42512:	shl    $0x5,%rax
   42516:	sub    %rcx,%rax
   42519:	sub    %rdx,%rax
   4251c:	lea    0x3c(,%rax,2),%rdx
   42524:	mov    $0xff,%esi
   42529:	call   *0xf1ef1(%rip)        # 134420 <memset@GLIBC_2.2.5>
   4252f:	shl    $0x6,%r13
   42533:	pcmpeqd %xmm0,%xmm0
   42537:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   4253e:	movdqa %xmm0,0x20(%rbx,%r13,1)
   42545:	movdqa %xmm0,0x10(%rbx,%r13,1)
   4254c:	movdqa %xmm0,(%rbx,%r13,1)
   42552:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   4255b:	mov    0x70(%rsp),%rax
   42560:	mov    0x8(%rax),%rdx
   42564:	mov    0x10(%rax),%rcx
   42568:	mov    0x5530(%rsp),%rax
   42570:	mov    0x8(%rax),%r8
   42574:	mov    0x10(%rax),%r9
   42578:	lea    0xa8(%rsp),%rdi
   42580:	mov    %rbx,%rsi
   42583:	call   99fa0 <c30walk::ctor::canonicalise>
   42588:	mov    0x90(%rsp),%rax
   42590:	mov    %rax,0x10(%r12)
   42595:	mov    $0x1,%al
   42597:	jmp    4243f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153f>
   4259c:	mov    %rdi,%r12
   4259f:	test   %rax,%rax
   425a2:	je     425b4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16b4>
   425a4:	mov    0x8(%r9),%rdi
   425a8:	mov    %r9,%r14
   425ab:	call   *0xf1e7f(%rip)        # 134430 <free@GLIBC_2.2.5>
   425b1:	mov    %r14,%r9
   425b4:	movq   $0x0,(%r9)
   425bb:	movq   $0x40,0x8(%r9)
   425c3:	movq   $0x0,0x10(%r9)
   425cb:	mov    $0x40,%edx
   425d0:	mov    %r9,%rdi
   425d3:	lea    0x1(%r13),%rsi
   425d7:	mov    %r9,%r14
   425da:	call   902a0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   425df:	mov    %r14,%r9
   425e2:	cmp    $0xffffffffffffffff,%rax
   425e6:	mov    %r12,%rdi
   425e9:	je     41123 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x223>
   425ef:	jmp    4264a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x174a>
   425f1:	mov    %rdi,%r12
   425f4:	mov    %r9,%r14
   425f7:	test   %rax,%rax
   425fa:	je     4260e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x170e>
   425fc:	mov    0x5530(%rsp),%rax
   42604:	mov    0x8(%rax),%rdi
   42608:	call   *0xf1e22(%rip)        # 134430 <free@GLIBC_2.2.5>
   4260e:	mov    0x5530(%rsp),%rdi
   42616:	movq   $0x0,(%rdi)
   4261d:	movq   $0x40,0x8(%rdi)
   42625:	movq   $0x0,0x10(%rdi)
   4262d:	mov    $0x80,%edx
   42632:	mov    %rbx,%rsi
   42635:	call   902a0 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4263a:	cmp    $0xffffffffffffffff,%rax
   4263e:	mov    %r14,%r9
   42641:	mov    %r12,%rdi
   42644:	je     41137 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x237>
   4264a:	mov    %rax,%rdi
   4264d:	mov    %rdx,%rsi
   42650:	call   d5c7b <alloc::raw_vec::handle_error>
   42655:	lea    0xedb34(%rip),%rcx        # 130190 <__frame_dummy_init_array_entry+0x2090>
   4265c:	mov    %rdi,%rdx
   4265f:	mov    %r14,%rdi
   42662:	mov    %r15,%rsi
   42665:	call   d84f0 <core::slice::index::slice_index_fail>
   4266a:	lea    0xedb07(%rip),%rcx        # 130178 <__frame_dummy_init_array_entry+0x2078>
   42671:	mov    %rdi,%rdx
   42674:	mov    %r15,%rdi
   42677:	mov    %r14,%rsi
   4267a:	call   d84f0 <core::slice::index::slice_index_fail>
   4267f:	lea    0xebcea(%rip),%rdx        # 12e370 <__frame_dummy_init_array_entry+0x270>
   42686:	mov    %rax,%rdi
   42689:	mov    0x18(%rsp),%rsi
   4268e:	call   d7713 <core::panicking::panic_bounds_check>
   42693:	lea    0xedb86(%rip),%rdx        # 130220 <__frame_dummy_init_array_entry+0x2120>
   4269a:	mov    %rax,%rdi
   4269d:	mov    %rcx,%rsi
   426a0:	call   d7713 <core::panicking::panic_bounds_check>
   426a5:	lea    0xedb5c(%rip),%rdx        # 130208 <__frame_dummy_init_array_entry+0x2108>
   426ac:	mov    $0x1e,%esi
   426b1:	mov    %r12,%rdi
   426b4:	call   d7713 <core::panicking::panic_bounds_check>
   426b9:	lea    0xedad0(%rip),%rcx        # 130190 <__frame_dummy_init_array_entry+0x2090>
   426c0:	mov    %rdi,%rdx
   426c3:	mov    %r15,%rdi
   426c6:	mov    %r12,%rsi
   426c9:	call   d84f0 <core::slice::index::slice_index_fail>
   426ce:	lea    0xedaa3(%rip),%rcx        # 130178 <__frame_dummy_init_array_entry+0x2078>
   426d5:	mov    %rdi,%rdx
   426d8:	mov    %r12,%rdi
   426db:	mov    0x20(%rsp),%rsi
   426e0:	call   d84f0 <core::slice::index::slice_index_fail>
   426e5:	lea    0xedb04(%rip),%rdx        # 1301f0 <__frame_dummy_init_array_entry+0x20f0>
   426ec:	mov    $0x1e,%esi
   426f1:	mov    %r12,%rdi
   426f4:	call   d7713 <core::panicking::panic_bounds_check>
   426f9:	mov    $0x10,%edx
   426fe:	mov    $0x4,%ecx
   42703:	mov    $0x400,%r8d
   42709:	mov    %rdi,%r14
   4270c:	mov    %r13,%rdi
   4270f:	xor    %esi,%esi
   42711:	mov    %r9,%rbx
   42714:	call   675b0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   42719:	mov    %r14,%rdi
   4271c:	mov    %rbx,%r9
   4271f:	jmp    411c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2c4>
   42724:	mov    %r9,%r13
   42727:	mov    $0x8,%ecx
   4272c:	mov    $0x8,%r8d
   42732:	mov    %rdi,%r14
   42735:	xor    %esi,%esi
   42737:	mov    %rbx,%rdx
   4273a:	call   675b0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   4273f:	mov    %r14,%rdi
   42742:	mov    0x10(%r14),%r14
   42746:	mov    0x8(%rdi),%r12
   4274a:	lea    (%r12,%r14,8),%rdi
   4274e:	cmp    $0x1,%rbx
   42752:	jne    41170 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x270>
   42758:	jmp    41192 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   4275d:	lea    0xebc0c(%rip),%rdx        # 12e370 <__frame_dummy_init_array_entry+0x270>
   42764:	call   d7713 <core::panicking::panic_bounds_check>
   42769:	lea    0xed9d8(%rip),%rdx        # 130148 <__frame_dummy_init_array_entry+0x2048>
   42770:	mov    $0x1e,%esi
   42775:	call   d7713 <core::panicking::panic_bounds_check>
   4277a:	lea    0xeda57(%rip),%rdx        # 1301d8 <__frame_dummy_init_array_entry+0x20d8>
   42781:	mov    $0x1e,%esi
   42786:	call   d7713 <core::panicking::panic_bounds_check>
   4278b:	lea    0xed93e(%rip),%rdx        # 1300d0 <__frame_dummy_init_array_entry+0x1fd0>
   42792:	call   d7713 <core::panicking::panic_bounds_check>
   42797:	lea    0xebbd2(%rip),%rdx        # 12e370 <__frame_dummy_init_array_entry+0x270>
   4279e:	mov    %r14,%rdi
   427a1:	call   d7713 <core::panicking::panic_bounds_check>
