# <c30walk::ctor2s::Ctor2s>::build::<false>
# size 0x18a6

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000440c0 <<c30walk::ctor2s::Ctor2s>::build::<false>>:
   440c0:	push   %rbp
   440c1:	push   %r15
   440c3:	push   %r14
   440c5:	push   %r13
   440c7:	push   %r12
   440c9:	push   %rbx
   440ca:	sub    $0x1000,%rsp
   440d1:	movq   $0x0,(%rsp)
   440d9:	sub    $0x1000,%rsp
   440e0:	movq   $0x0,(%rsp)
   440e8:	sub    $0x1000,%rsp
   440ef:	movq   $0x0,(%rsp)
   440f7:	sub    $0x1000,%rsp
   440fe:	movq   $0x0,(%rsp)
   44106:	sub    $0x1000,%rsp
   4410d:	movq   $0x0,(%rsp)
   44115:	sub    $0x4f8,%rsp
   4411c:	mov    %r8d,0x14(%rsp)
   44121:	mov    %rsi,0x48(%rsp)
   44126:	mov    0x5530(%rsp),%rax
   4412e:	movq   $0x0,0x10(%r9)
   44136:	movq   $0x0,0x10(%rax)
   4413e:	cmp    $0xeffe2,%rdx
   44145:	ja     455fd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   4414b:	mov    %rcx,%r15
   4414e:	mov    %rdx,%rsi
   44151:	pxor   %xmm0,%xmm0
   44155:	xor    %eax,%eax
   44157:	pxor   %xmm2,%xmm2
   4415b:	pxor   %xmm1,%xmm1
   4415f:	nop
   44160:	movq   (%r15,%rax,4),%xmm3
   44166:	movq   0x8(%r15,%rax,4),%xmm4
   4416d:	punpckldq %xmm0,%xmm3
   44171:	paddq  %xmm2,%xmm3
   44175:	punpckldq %xmm0,%xmm4
   44179:	paddq  %xmm1,%xmm4
   4417d:	movq   0x10(%r15,%rax,4),%xmm2
   44184:	movq   0x18(%r15,%rax,4),%xmm1
   4418b:	punpckldq %xmm0,%xmm2
   4418f:	paddq  %xmm3,%xmm2
   44193:	punpckldq %xmm0,%xmm1
   44197:	paddq  %xmm4,%xmm1
   4419b:	add    $0x8,%rax
   4419f:	cmp    $0x100,%rax
   441a5:	jne    44160 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa0>
   441a7:	paddq  %xmm2,%xmm1
   441ab:	pshufd $0xee,%xmm1,%xmm0
   441b0:	paddq  %xmm1,%xmm0
   441b4:	movq   %xmm0,%rax
   441b9:	cmp    %rsi,%rax
   441bc:	jne    455fd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   441c2:	pxor   %xmm2,%xmm2
   441c6:	xor    %eax,%eax
   441c8:	movdqa -0x33670(%rip),%xmm3        # 10b60 <GCC_except_table1006+0xa08>
   441d0:	pxor   %xmm1,%xmm1
   441d4:	pxor   %xmm0,%xmm0
   441d8:	nopl   0x0(%rax,%rax,1)
   441e0:	movq   (%r15,%rax,4),%xmm4
   441e6:	movq   0x8(%r15,%rax,4),%xmm5
   441ed:	pshufd $0xd4,%xmm4,%xmm4
   441f2:	pcmpeqd %xmm2,%xmm4
   441f6:	pandn  %xmm3,%xmm4
   441fa:	paddq  %xmm1,%xmm4
   441fe:	pshufd $0xd4,%xmm5,%xmm5
   44203:	pcmpeqd %xmm2,%xmm5
   44207:	pandn  %xmm3,%xmm5
   4420b:	paddq  %xmm0,%xmm5
   4420f:	movq   0x10(%r15,%rax,4),%xmm0
   44216:	movq   0x18(%r15,%rax,4),%xmm6
   4421d:	pshufd $0xd4,%xmm0,%xmm1
   44222:	pcmpeqd %xmm2,%xmm1
   44226:	pandn  %xmm3,%xmm1
   4422a:	paddq  %xmm4,%xmm1
   4422e:	pshufd $0xd4,%xmm6,%xmm0
   44233:	pcmpeqd %xmm2,%xmm0
   44237:	pandn  %xmm3,%xmm0
   4423b:	paddq  %xmm5,%xmm0
   4423f:	add    $0x8,%rax
   44243:	cmp    $0x100,%rax
   44249:	jne    441e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120>
   4424b:	mov    $0x88888889,%r13d
   44251:	imul   %rsi,%r13
   44255:	shr    $0x24,%r13
   44259:	imul   $0xeeeeeeef,%esi,%eax
   4425f:	ror    $1,%eax
   44261:	cmp    $0x8888889,%eax
   44266:	sbb    $0xffffffffffffffff,%r13
   4426a:	mov    %r13,%rbx
   4426d:	cmpl   $0x0,0x14(%rsp)
   44272:	jne    442c6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   44274:	test   %rsi,%rsi
   44277:	je     442c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x204>
   44279:	paddq  %xmm1,%xmm0
   4427d:	pshufd $0xee,%xmm0,%xmm1
   44282:	paddq  %xmm0,%xmm1
   44286:	movq   %xmm1,%rax
   4428b:	lea    -0x1(%rsi),%ecx
   4428e:	mov    $0x88888889,%edx
   44293:	imul   %rcx,%rdx
   44297:	shr    $0x23,%rdx
   4429b:	and    $0xfffffffe,%edx
   4429e:	lea    (%rdx,%rsi,1),%ecx
   442a1:	dec    %ecx
   442a3:	shr    $1,%ecx
   442a5:	mov    $0x80010003,%ebx
   442aa:	imul   %rcx,%rbx
   442ae:	shr    $0x2e,%rbx
   442b2:	inc    %ebx
   442b4:	imul   %rax,%rbx
   442b8:	dec    %rbx
   442bb:	cmp    %r13,%rbx
   442be:	cmovae %r13,%rbx
   442c2:	jmp    442c6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   442c4:	xor    %ebx,%ebx
   442c6:	mov    (%r9),%rax
   442c9:	lea    0x1(%r13),%rcx
   442cd:	mov    %rcx,0x90(%rsp)
   442d5:	cmp    %r13,%rax
   442d8:	mov    %rsi,0x8(%rsp)
   442dd:	jbe    4575c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x169c>
   442e3:	mov    0x5530(%rsp),%rax
   442eb:	mov    (%rax),%rax
   442ee:	cmp    %rbx,%rax
   442f1:	jb     457b1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16f1>
   442f7:	movq   $0x0,0x10(%rdi)
   442ff:	lea    0x3f(%r13),%rbx
   44303:	shr    $0x6,%rbx
   44307:	mov    %rdi,0x70(%rsp)
   4430c:	mov    %r13,0x78(%rsp)
   44311:	je     44366 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a6>
   44313:	cmp    (%rdi),%rbx
   44316:	ja     458e4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1824>
   4431c:	mov    %r9,%r13
   4431f:	xor    %r14d,%r14d
   44322:	mov    0x8(%rdi),%r12
   44326:	lea    (%r12,%r14,8),%rdi
   4432a:	cmp    $0x1,%rbx
   4432e:	je     44352 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   44330:	lea    -0x8(,%rbx,8),%rdx
   44338:	xor    %esi,%esi
   4433a:	call   *0x1092b0(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   44340:	lea    (%r14,%rbx,1),%rax
   44344:	lea    (%r12,%rax,8),%rdi
   44348:	add    $0xfffffffffffffff8,%rdi
   4434c:	add    %rbx,%r14
   4434f:	dec    %r14
   44352:	mov    %r13,%r9
   44355:	movq   $0x0,(%rdi)
   4435c:	inc    %r14
   4435f:	mov    0x70(%rsp),%rdi
   44364:	jmp    44369 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a9>
   44366:	xor    %r14d,%r14d
   44369:	lea    0x18(%rdi),%r13
   4436d:	mov    %r14,0x10(%rdi)
   44371:	movq   $0x0,0x28(%rdi)
   44379:	cmpq   $0xf,0x18(%rdi)
   4437e:	jbe    458b9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17f9>
   44384:	lea    0x30(%rdi),%r14
   44388:	xor    %eax,%eax
   4438a:	pxor   %xmm0,%xmm0
   4438e:	movabs $0xffffffff00000000,%rcx
   44398:	nopl   0x0(%rax,%rax,1)
   443a0:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   443a9:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   443b2:	mov    %rcx,0x2d18(%rsp,%rax,1)
   443ba:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   443c3:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   443cc:	mov    %rcx,0x2d40(%rsp,%rax,1)
   443d4:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   443dd:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   443e6:	mov    %rcx,0x2d68(%rsp,%rax,1)
   443ee:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   443f7:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   44400:	mov    %rcx,0x2d90(%rsp,%rax,1)
   44408:	add    $0xa0,%rax
   4440e:	cmp    $0x2800,%rax
   44414:	jne    443a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2e0>
   44416:	lea    0xc(%r15),%rax
   4441a:	xor    %edx,%edx
   4441c:	mov    $0x25,%ecx
   44421:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   44430:	mov    -0xc(%rax),%esi
   44433:	add    %edx,%esi
   44435:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   4443c:	mov    %esi,0x2c80(%rsp,%rcx,4)
   44443:	mov    -0x8(%rax),%edx
   44446:	add    %esi,%edx
   44448:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   4444f:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   44456:	mov    -0x4(%rax),%esi
   44459:	add    %edx,%esi
   4445b:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   44462:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   44469:	mov    (%rax),%edx
   4446b:	add    %esi,%edx
   4446d:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   44474:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   4447b:	add    $0x28,%rcx
   4447f:	add    $0x10,%rax
   44483:	cmp    $0xa25,%rcx
   4448a:	jne    44430 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x370>
   4448c:	mov    %r9,0x88(%rsp)
   44494:	mov    0x8(%r9),%rbx
   44498:	mov    0x8(%rdi),%rbp
   4449c:	mov    0x10(%rdi),%r12
   444a0:	lea    0xe8(%rsp),%rdi
   444a8:	lea    0x2cf8(%rsp),%rsi
   444b0:	mov    $0x2800,%edx
   444b5:	call   *0x109155(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   444bb:	mov    %rbp,%r9
   444be:	mov    %r12,%rsi
   444c1:	mov    %rbx,0xd0(%rsp)
   444c9:	mov    0x78(%rsp),%rax
   444ce:	mov    %rax,0xd8(%rsp)
   444d6:	mov    0x8(%rsp),%r11
   444db:	mov    %r11,0xe0(%rsp)
   444e3:	mov    0x5530(%rsp),%rax
   444eb:	mov    %rax,0xa8(%rsp)
   444f3:	mov    %rbp,0xb0(%rsp)
   444fb:	mov    %r12,0xb8(%rsp)
   44503:	mov    %r13,0xc0(%rsp)
   4450b:	movl   $0x0,0x28f0(%rsp)
   44516:	movq   $0x0,0x28e8(%rsp)
   44522:	mov    %r14,0xc8(%rsp)
   4452a:	mov    $0xffffffffffffffff,%rcx
   44531:	xor    %r13d,%r13d
   44534:	movabs $0xeeeeeeeeeeeeeeef,%r10
   4453e:	movabs $0x888888888888889,%r12
   44548:	pxor   %xmm0,%xmm0
   4454c:	pcmpeqd %xmm1,%xmm1
   44550:	xor    %r14d,%r14d
   44553:	jmp    44603 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x543>
   44558:	nopl   0x0(%rax,%rax,1)
   44560:	mov    0x8(%rdi),%rax
   44564:	mov    %rbp,%rcx
   44567:	shl    $0x7,%rcx
   4456b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   44571:	movdqa %xmm0,0x60(%rax,%rcx,1)
   44577:	movdqa %xmm0,0x50(%rax,%rcx,1)
   4457d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   44583:	movdqa %xmm0,0x30(%rax,%rcx,1)
   44589:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4458f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   44595:	movdqa %xmm0,(%rax,%rcx,1)
   4459a:	mov    %rbp,%rax
   4459d:	inc    %rax
   445a0:	mov    %rax,0x10(%rdi)
   445a4:	mov    %rdx,%rax
   445a7:	shl    $0x6,%rax
   445ab:	or     $0x10000000,%ebp
   445b1:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   445b7:	movdqa %xmm1,0x20(%rbx,%rax,1)
   445bd:	movdqa %xmm1,0x10(%rbx,%rax,1)
   445c3:	movdqa %xmm1,(%rbx,%rax,1)
   445c8:	mov    %ebp,0x3c(%rbx,%rax,1)
   445cc:	mov    %rdx,%rdi
   445cf:	shr    $0x6,%rdi
   445d3:	cmp    %rsi,%rdi
   445d6:	jae    4594b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x188b>
   445dc:	mov    $0x1,%eax
   445e1:	mov    %edx,%ecx
   445e3:	shl    %cl,%rax
   445e6:	or     %rax,(%r9,%rdi,8)
   445ea:	mov    %rdx,%rcx
   445ed:	add    $0x4,%r14
   445f1:	cmp    $0x400,%r14
   445f8:	mov    0x8(%rsp),%r11
   445fd:	je     4469e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x5de>
   44603:	mov    %r13,%rax
   44606:	mov    (%r15,%r14,1),%r13d
   4460a:	add    %rax,%r13
   4460d:	cmp    %r11,%r13
   44610:	setae  %r8b
   44614:	mov    %r13,%rax
   44617:	movabs $0x8888888888888889,%rdx
   44621:	mul    %rdx
   44624:	shr    $0x4,%rdx
   44628:	mov    %r13,%rax
   4462b:	imul   %r10,%rax
   4462f:	ror    $1,%rax
   44632:	cmp    %r12,%rax
   44635:	setb   %al
   44638:	cmp    %rcx,%rdx
   4463b:	sete   %dil
   4463f:	or     %r8b,%dil
   44642:	or     %al,%dil
   44645:	jne    445ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x52d>
   44647:	mov    0x5530(%rsp),%rdi
   4464f:	mov    0x10(%rdi),%rbp
   44653:	cmp    (%rdi),%rbp
   44656:	jne    44560 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   4465c:	mov    %rdx,0x18(%rsp)
   44661:	mov    %rsi,0x30(%rsp)
   44666:	mov    %r9,0x58(%rsp)
   4466b:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   44670:	pcmpeqd %xmm1,%xmm1
   44674:	pxor   %xmm0,%xmm0
   44678:	movabs $0xeeeeeeeeeeeeeeef,%r10
   44682:	mov    0x58(%rsp),%r9
   44687:	mov    0x30(%rsp),%rsi
   4468c:	mov    0x5530(%rsp),%rdi
   44694:	mov    0x18(%rsp),%rdx
   44699:	jmp    44560 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   4469e:	cmpl   $0x0,0x14(%rsp)
   446a3:	je     44854 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   446a9:	mov    0xd8(%rsp),%r8
   446b1:	test   %r8,%r8
   446b4:	je     44854 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   446ba:	mov    %r11,%r9
   446bd:	shl    $0x20,%r9
   446c1:	mov    0x14(%rsp),%r10d
   446c6:	mov    0xa8(%rsp),%rdi
   446ce:	mov    0xd0(%rsp),%rbp
   446d6:	xor    %r12d,%r12d
   446d9:	movabs $0xff51afd7ed558ccd,%rbx
   446e3:	pxor   %xmm0,%xmm0
   446e7:	pcmpeqd %xmm1,%xmm1
   446eb:	jmp    44770 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6b0>
   446f0:	mov    0x8(%rdi),%rax
   446f4:	mov    %rbx,%rcx
   446f7:	shl    $0x7,%rcx
   446fb:	movdqa %xmm0,0x70(%rax,%rcx,1)
   44701:	movdqa %xmm0,0x60(%rax,%rcx,1)
   44707:	movdqa %xmm0,0x50(%rax,%rcx,1)
   4470d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   44713:	movdqa %xmm0,0x30(%rax,%rcx,1)
   44719:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4471f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   44725:	movdqa %xmm0,(%rax,%rcx,1)
   4472a:	mov    %rbx,%rax
   4472d:	inc    %rax
   44730:	mov    %rax,0x10(%rdi)
   44734:	or     $0x10000000,%ebx
   4473a:	movdqu %xmm1,0x2c(%rbp)
   4473f:	movdqa %xmm1,0x20(%rbp)
   44744:	movdqa %xmm1,0x10(%rbp)
   44749:	movdqa %xmm1,0x0(%rbp)
   4474e:	mov    %ebx,0x3c(%rbp)
   44751:	or     %r15,0x0(%r13,%r14,8)
   44756:	movabs $0xff51afd7ed558ccd,%rbx
   44760:	inc    %r12
   44763:	add    $0x40,%rbp
   44767:	cmp    %r12,%r8
   4476a:	je     44854 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   44770:	mov    %r12,%r14
   44773:	shr    $0x6,%r14
   44777:	mov    0xb8(%rsp),%rsi
   4477f:	cmp    %rsi,%r14
   44782:	jae    45957 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1897>
   44788:	mov    0xb0(%rsp),%r13
   44790:	mov    0x0(%r13,%r14,8),%rax
   44795:	mov    $0x1,%r15d
   4479b:	mov    %r12d,%ecx
   4479e:	shl    %cl,%r15
   447a1:	bt     %r12,%rax
   447a5:	jb     44760 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   447a7:	mov    %r12,%rax
   447aa:	xor    %r9,%rax
   447ad:	mov    %rax,%rcx
   447b0:	shr    $0x21,%rcx
   447b4:	xor    %rax,%rcx
   447b7:	imul   %rbx,%rcx
   447bb:	mov    %rcx,%rax
   447be:	shr    $0x21,%rax
   447c2:	xor    %rcx,%rax
   447c5:	movabs $0xc4ceb9fe1a85ec53,%rcx
   447cf:	imul   %rcx,%rax
   447d3:	mov    %rax,%rcx
   447d6:	shr    $0x21,%rcx
   447da:	xor    %rax,%rcx
   447dd:	mov    %rcx,%rax
   447e0:	shr    $0x3,%rax
   447e4:	movabs $0x20c49ba5e353f7cf,%rdx
   447ee:	mul    %rdx
   447f1:	shr    $0x4,%rdx
   447f5:	imul   $0x3e8,%rdx,%rax
   447fc:	sub    %rax,%rcx
   447ff:	cmp    %r10,%rcx
   44802:	jae    44760 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   44808:	mov    0x10(%rdi),%rbx
   4480c:	cmp    (%rdi),%rbx
   4480f:	jne    446f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   44815:	mov    %r8,0x18(%rsp)
   4481a:	mov    %r9,0x30(%rsp)
   4481f:	mov    %r10,0x58(%rsp)
   44824:	mov    %rdi,0x68(%rsp)
   44829:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4482e:	pcmpeqd %xmm1,%xmm1
   44832:	pxor   %xmm0,%xmm0
   44836:	mov    0x68(%rsp),%rdi
   4483b:	mov    0x58(%rsp),%r10
   44840:	mov    0x30(%rsp),%r9
   44845:	mov    0x18(%rsp),%r8
   4484a:	mov    0x8(%rsp),%r11
   4484f:	jmp    446f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   44854:	test   %r11,%r11
   44857:	je     45591 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   4485d:	movq   $0x0,0x20(%rsp)
   44866:	jmp    448ad <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ed>
   44868:	mov    0x38(%rsp),%r14
   4486d:	lea    0x1(%rax),%rdx
   44871:	add    $0xffffffffffffffe3,%rax
   44875:	xor    %esi,%esi
   44877:	cmp    $0x1e,%rdx
   4487b:	setae  %sil
   4487f:	cmovb  %rdx,%rax
   44883:	lea    (%rcx,%rsi,2),%ecx
   44886:	inc    %ecx
   44888:	mov    %rax,0x28e8(%rsp)
   44890:	mov    %ecx,0x28f0(%rsp)
   44897:	mov    0x40(%rsp),%rcx
   4489c:	mov    %rcx,%r12
   4489f:	cmp    %rdi,0x98(%rsp)
   448a7:	jae    45591 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   448ad:	mov    0x20(%rsp),%r13
   448b2:	mov    $0x400,%edx
   448b7:	lea    0x28f8(%rsp),%rdi
   448bf:	xor    %esi,%esi
   448c1:	call   *0x108d29(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   448c7:	lea    0x128(%rsp),%rax
   448cf:	xor    %ecx,%ecx
   448d1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   448e0:	mov    -0x40(%rax),%rdx
   448e4:	sub    -0x30(%rax),%rdx
   448e8:	shr    $1,%rdx
   448eb:	add    -0x28(%rax),%edx
   448ee:	mov    -0x18(%rax),%rsi
   448f2:	sub    -0x8(%rax),%rsi
   448f6:	mov    %edx,0x28f8(%rsp,%rcx,4)
   448fd:	shr    $1,%rsi
   44900:	add    (%rax),%esi
   44902:	mov    %esi,0x28fc(%rsp,%rcx,4)
   44909:	add    $0x2,%rcx
   4490d:	add    $0x50,%rax
   44911:	cmp    $0x100,%rcx
   44918:	jne    448e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x820>
   4491a:	mov    0xc0(%rsp),%r15
   44922:	mov    0x10(%r15),%rbx
   44926:	cmp    (%r15),%rbx
   44929:	je     45584 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14c4>
   4492f:	mov    %rbx,%rdi
   44932:	shl    $0xa,%rdi
   44936:	add    0x8(%r15),%rdi
   4493a:	mov    $0x400,%edx
   4493f:	lea    0x28f8(%rsp),%rsi
   44947:	call   *0x108cc3(%rip)        # 14d610 <memcpy@GLIBC_2.14>
   4494d:	inc    %rbx
   44950:	lea    0xf000(%r13),%rax
   44957:	mov    0x8(%rsp),%rdi
   4495c:	cmp    %rdi,%rax
   4495f:	mov    %rdi,%rcx
   44962:	mov    %rax,0x98(%rsp)
   4496a:	cmovb  %rax,%rcx
   4496e:	mov    %rbx,0x10(%r15)
   44972:	mov    %rcx,0x20(%rsp)
   44977:	mov    %rcx,%r8
   4497a:	sub    %r13,%r8
   4497d:	cmp    $0x10,%r8
   44981:	jb     453b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f0>
   44987:	add    0x48(%rsp),%r13
   4498c:	xor    %eax,%eax
   4498e:	mov    %r12,%rcx
   44991:	mov    %r8,%r12
   44994:	jmp    449e8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   44996:	cs nopw 0x0(%rax,%rax,1)
   449a0:	mov    0x28e8(%rsp),%rax
   449a8:	lea    0x10(%rax),%rcx
   449ac:	add    $0xfffffffffffffff2,%rax
   449b0:	xor    %edx,%edx
   449b2:	cmp    $0x1e,%rcx
   449b6:	setae  %dl
   449b9:	cmovb  %rcx,%rax
   449bd:	mov    %rax,0x28e8(%rsp)
   449c5:	mov    0x28f0(%rsp),%eax
   449cc:	lea    (%rax,%rdx,2),%eax
   449cf:	add    $0x10,%eax
   449d2:	mov    %eax,0x28f0(%rsp)
   449d9:	xor    %eax,%eax
   449db:	mov    %r8,%r12
   449de:	cmp    $0xf,%r8
   449e2:	jbe    453b3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   449e8:	test   $0x1,%al
   449ea:	jne    449f4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x934>
   449ec:	mov    0x0(%r13),%rcx
   449f0:	mov    0x8(%r13),%r14
   449f4:	add    $0x10,%r13
   449f8:	lea    -0x10(%r12),%r8
   449fd:	xor    %rcx,%r14
   44a00:	mov    %rcx,%rax
   44a03:	rol    $0x8,%rax
   44a07:	xor    %rcx,%rax
   44a0a:	or     %r14,%rax
   44a0d:	lea    0xe8(%rsp),%rdx
   44a15:	jne    44a70 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x9b0>
   44a17:	cmp    $0x10,%r8
   44a1b:	jb     44ae0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa20>
   44a21:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   44a30:	mov    0x0(%r13),%rdx
   44a34:	mov    0x8(%r13),%rax
   44a38:	mov    $0x1,%r9b
   44a3b:	cmp    %rcx,%rdx
   44a3e:	jne    44c32 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   44a44:	cmp    %rcx,%rax
   44a47:	jne    44c32 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   44a4d:	add    $0xfffffffffffffff0,%r8
   44a51:	add    $0x10,%r13
   44a55:	cmp    $0xf,%r8
   44a59:	ja     44a30 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x970>
   44a5b:	mov    %r13,0x28(%rsp)
   44a60:	xor    %r9d,%r9d
   44a63:	mov    %rcx,0x40(%rsp)
   44a68:	jmp    44c41 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   44a6d:	nopl   (%rax)
   44a70:	mov    0x20(%rsp),%r14
   44a75:	sub    %r12,%r14
   44a78:	mov    0x28e8(%rsp),%rax
   44a80:	mov    $0x1e,%r9d
   44a86:	sub    %rax,%r9
   44a89:	cmp    $0x10,%r9
   44a8d:	mov    $0x10,%ebx
   44a92:	cmovb  %r9,%rbx
   44a96:	mov    %rbx,%r15
   44a99:	add    %r14,%r15
   44a9c:	jb     45815 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   44aa2:	cmp    %rdi,%r15
   44aa5:	ja     45815 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   44aab:	mov    0x28f0(%rsp),%r12d
   44ab3:	cmp    $0x1e,%rax
   44ab7:	jne    44af0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa30>
   44ab9:	add    $0x10,%r14
   44abd:	cmp    %rdi,%r14
   44ac0:	ja     4582a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   44ac6:	cmp    %r15,%r14
   44ac9:	jae    44bb5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xaf5>
   44acf:	jmp    4582a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   44ad4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   44ae0:	mov    %r13,0x28(%rsp)
   44ae5:	xor    %r9d,%r9d
   44ae8:	jmp    44c41 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   44aed:	nopl   (%rax)
   44af0:	mov    %r13,0x28(%rsp)
   44af5:	mov    0x48(%rsp),%rax
   44afa:	lea    (%rax,%r14,1),%r10
   44afe:	xor    %r13d,%r13d
   44b01:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   44b10:	movzbl (%r10,%r13,1),%esi
   44b15:	lea    (%rsi,%rsi,4),%rcx
   44b19:	lea    (%rdx,%rcx,8),%rbp
   44b1d:	mov    (%rdx,%rcx,8),%rax
   44b21:	cmp    0x8(%rdx,%rcx,8),%rax
   44b26:	je     44b3e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa7e>
   44b28:	lea    (%r12,%r13,1),%rcx
   44b2c:	mov    %cx,(%rax)
   44b2f:	addq   $0x2,0x0(%rbp)
   44b34:	inc    %r13
   44b37:	cmp    %r13,%rbx
   44b3a:	jne    44b10 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa50>
   44b3c:	jmp    44b90 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xad0>
   44b3e:	mov    %r10,0x30(%rsp)
   44b43:	mov    %r9,0x18(%rsp)
   44b48:	mov    %r8,0x50(%rsp)
   44b4d:	lea    0xa8(%rsp),%rdi
   44b55:	call   95e90 <<c30walk::ctor2s::W<false>>::refill>
   44b5a:	test   %al,%al
   44b5c:	je     455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   44b62:	mov    0x0(%rbp),%rax
   44b66:	mov    0x8(%rsp),%rdi
   44b6b:	lea    0xe8(%rsp),%rdx
   44b73:	mov    0x50(%rsp),%r8
   44b78:	mov    0x18(%rsp),%r9
   44b7d:	mov    0x30(%rsp),%r10
   44b82:	jmp    44b28 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa68>
   44b84:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   44b90:	add    $0x10,%r14
   44b94:	cmp    %rdi,%r14
   44b97:	ja     4582a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   44b9d:	cmp    %r15,%r14
   44ba0:	mov    0x28(%rsp),%r13
   44ba5:	jb     4582a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   44bab:	cmp    $0xf,%r9
   44baf:	ja     449a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   44bb5:	add    0x48(%rsp),%r15
   44bba:	mov    %r15,%r14
   44bbd:	sub    %rbx,%r14
   44bc0:	add    $0x10,%r14
   44bc4:	add    %r12d,%ebx
   44bc7:	add    $0x2,%ebx
   44bca:	nopw   0x0(%rax,%rax,1)
   44bd0:	movzbl (%r15),%esi
   44bd4:	lea    (%rsi,%rsi,4),%rcx
   44bd8:	lea    (%rdx,%rcx,8),%r12
   44bdc:	mov    (%rdx,%rcx,8),%rax
   44be0:	cmp    0x8(%rdx,%rcx,8),%rax
   44be5:	je     44bfe <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb3e>
   44be7:	mov    %bx,(%rax)
   44bea:	addq   $0x2,(%r12)
   44bef:	inc    %r15
   44bf2:	inc    %ebx
   44bf4:	cmp    %r14,%r15
   44bf7:	jne    44bd0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb10>
   44bf9:	jmp    449a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   44bfe:	mov    %r13,%rbp
   44c01:	mov    %r8,%r13
   44c04:	lea    0xa8(%rsp),%rdi
   44c0c:	call   95e90 <<c30walk::ctor2s::W<false>>::refill>
   44c11:	test   %al,%al
   44c13:	je     455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   44c19:	mov    (%r12),%rax
   44c1d:	mov    0x8(%rsp),%rdi
   44c22:	lea    0xe8(%rsp),%rdx
   44c2a:	mov    %r13,%r8
   44c2d:	mov    %rbp,%r13
   44c30:	jmp    44be7 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb27>
   44c32:	mov    %rdx,0x40(%rsp)
   44c37:	mov    %r13,0x28(%rsp)
   44c3c:	mov    %rax,0x38(%rsp)
   44c41:	mov    %r8,0x50(%rsp)
   44c46:	sub    %r8,%r12
   44c49:	movzbl %cl,%eax
   44c4c:	mov    %rax,0xa0(%rsp)
   44c54:	lea    (%rax,%rax,4),%rdx
   44c58:	lea    0xe8(%rsp),%r8
   44c60:	mov    (%r8,%rdx,8),%rax
   44c64:	mov    0x1c(%r8,%rdx,8),%esi
   44c69:	mov    %rax,%rdi
   44c6c:	sub    0x10(%r8,%rdx,8),%rdi
   44c71:	shr    $1,%rdi
   44c74:	add    0x18(%r8,%rdx,8),%edi
   44c79:	add    %r12,%rdi
   44c7c:	cmp    %rsi,%rdi
   44c7f:	ja     455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   44c85:	mov    %r9d,0x64(%rsp)
   44c8a:	lea    0xe8(%rsp),%rsi
   44c92:	lea    (%rsi,%rdx,8),%rbx
   44c96:	movzbl %cl,%r14d
   44c9a:	mov    0x8(%rbx),%rsi
   44c9e:	mov    $0x88888889,%r15d
   44ca4:	lea    -0x27c6b(%rip),%r13        # 1d040 <c30walk::ctor2s::PATTERNS>
   44cab:	mov    %rbx,0x80(%rsp)
   44cb3:	jmp    44cee <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc2e>
   44cb5:	data16 cs nopw 0x0(%rax,%rax,1)
   44cc0:	mov    0x3c(%rsi),%edi
   44cc3:	mov    $0xfffff,%r9d
   44cc9:	and    %r9d,%edi
   44ccc:	sub    %r8d,%ecx
   44ccf:	sub    %eax,%r8d
   44cd2:	mov    %r8d,%eax
   44cd5:	lea    (%rsi,%rax,2),%rax
   44cd9:	mov    %rax,0x10(%rbx)
   44cdd:	mov    %rax,(%rbx)
   44ce0:	lea    (%rax,%rcx,2),%rsi
   44ce4:	mov    %rsi,0x8(%rbx)
   44ce8:	mov    %edx,0x20(%rbx)
   44ceb:	mov    %edi,0x24(%rbx)
   44cee:	sub    %rax,%rsi
   44cf1:	shr    $1,%rsi
   44cf4:	cmp    %rsi,%r12
   44cf7:	cmovb  %r12,%rsi
   44cfb:	test   %rsi,%rsi
   44cfe:	je     44db8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcf8>
   44d04:	mov    0x28e8(%rsp),%rdi
   44d0c:	cmp    $0x1d,%rdi
   44d10:	ja     45929 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1869>
   44d16:	cmp    $0x1f,%rsi
   44d1a:	jae    45611 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1551>
   44d20:	mov    0x28f0(%rsp),%ecx
   44d27:	mov    %rdi,%r11
   44d2a:	shl    $0x6,%r11
   44d2e:	lea    (%r11,%r13,1),%rdx
   44d32:	lea    -0x2(,%rsi,2),%r9
   44d3a:	cmp    $0xe,%r9
   44d3e:	jb     44d53 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc93>
   44d40:	mov    %rax,%r8
   44d43:	sub    %r13,%r8
   44d46:	sub    %r11,%r8
   44d49:	cmp    $0x10,%r8
   44d4d:	jae    45198 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x10d8>
   44d53:	mov    %rdx,%r8
   44d56:	xor    %r10d,%r10d
   44d59:	lea    (%rdx,%rsi,2),%rdx
   44d5d:	lea    (%rax,%r10,2),%r9
   44d61:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   44d70:	movzwl (%r8),%r10d
   44d74:	add    %cx,%r10w
   44d78:	add    $0x2,%r8
   44d7c:	mov    %r10w,(%r9)
   44d80:	add    $0x2,%r9
   44d84:	cmp    %rdx,%r8
   44d87:	jne    44d70 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcb0>
   44d89:	lea    (%rax,%rsi,2),%rax
   44d8d:	mov    %rax,(%rbx)
   44d90:	add    %rsi,%rdi
   44d93:	lea    0x2(%rsi),%edx
   44d96:	mov    %rdi,%r8
   44d99:	sub    $0x1e,%r8
   44d9d:	cmovb  %rdi,%r8
   44da1:	cmovb  %esi,%edx
   44da4:	mov    %r8,0x28e8(%rsp)
   44dac:	add    %ecx,%edx
   44dae:	mov    %edx,0x28f0(%rsp)
   44db5:	sub    %rsi,%r12
   44db8:	test   %r12,%r12
   44dbb:	je     45380 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   44dc1:	mov    %rax,%r8
   44dc4:	cmp    0x10(%rbx),%rax
   44dc8:	je     44de6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xd26>
   44dca:	lea    0xa8(%rsp),%rdi
   44dd2:	mov    0xa0(%rsp),%rsi
   44dda:	call   95890 <<c30walk::ctor2s::W<false>>::finish>
   44ddf:	mov    (%rbx),%r8
   44de2:	mov    0x10(%rbx),%rax
   44de6:	sub    %rax,%r8
   44de9:	shr    $1,%r8
   44dec:	add    0x18(%rbx),%r8d
   44df0:	mov    %r8d,0x18(%rbx)
   44df4:	pxor   %xmm0,%xmm0
   44df8:	movdqu %xmm0,(%rbx)
   44dfc:	movq   $0x0,0x10(%rbx)
   44e04:	cmp    $0x1e,%r12
   44e08:	jb     45310 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   44e0e:	mov    %r8,%rcx
   44e11:	imul   %r15,%rcx
   44e15:	shr    $0x24,%rcx
   44e19:	mov    %ecx,%eax
   44e1b:	shl    $0x5,%eax
   44e1e:	lea    (%rcx,%rcx,1),%edx
   44e21:	sub    %eax,%edx
   44e23:	add    %r8d,%edx
   44e26:	jne    45310 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   44e2c:	mov    %r12,%rax
   44e2f:	movabs $0x8888888888888889,%rdx
   44e39:	mul    %rdx
   44e3c:	shr    $0x4,%rdx
   44e40:	mov    %ecx,%r9d
   44e43:	cmpl   $0x0,0x14(%rsp)
   44e48:	mov    %r12,0x58(%rsp)
   44e4d:	mov    %rdx,0x68(%rsp)
   44e52:	je     45150 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1090>
   44e58:	mov    0x28f0(%rsp),%r10d
   44e60:	mov    0xb8(%rsp),%rax
   44e68:	mov    %rax,0x18(%rsp)
   44e6d:	mov    0xa8(%rsp),%r11
   44e75:	mov    0xb0(%rsp),%rax
   44e7d:	mov    %rax,0x30(%rsp)
   44e82:	mov    0xd0(%rsp),%rax
   44e8a:	mov    0x28e8(%rsp),%r12
   44e92:	mov    %r12,%rdi
   44e95:	shl    $0x6,%rdi
   44e99:	add    %r13,%rdi
   44e9c:	mov    %r9,%rcx
   44e9f:	shl    $0x6,%rcx
   44ea3:	lea    (%rax,%rcx,1),%rbx
   44ea7:	add    $0x3c,%rbx
   44eab:	mov    %r10d,%r13d
   44eae:	shl    $0x8,%r13d
   44eb2:	or     %r14d,%r13d
   44eb5:	xor    %r15d,%r15d
   44eb8:	jmp    450c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1004>
   44ebd:	nopl   (%rax)
   44ec0:	mov    0x8(%r11),%rcx
   44ec4:	shl    $0x7,%eax
   44ec7:	mov    %r13d,(%rcx,%rax,1)
   44ecb:	movzwl 0x2(%rdi),%esi
   44ecf:	lea    (%r10,%r15,1),%ebp
   44ed3:	add    %ebp,%esi
   44ed5:	shl    $0x8,%esi
   44ed8:	or     %r14d,%esi
   44edb:	mov    %esi,0x4(%rcx,%rax,1)
   44edf:	movzwl 0x4(%rdi),%esi
   44ee3:	add    %ebp,%esi
   44ee5:	shl    $0x8,%esi
   44ee8:	or     %r14d,%esi
   44eeb:	mov    %esi,0x8(%rcx,%rax,1)
   44eef:	movzwl 0x6(%rdi),%esi
   44ef3:	add    %ebp,%esi
   44ef5:	shl    $0x8,%esi
   44ef8:	or     %r14d,%esi
   44efb:	mov    %esi,0xc(%rcx,%rax,1)
   44eff:	movzwl 0x8(%rdi),%esi
   44f03:	add    %ebp,%esi
   44f05:	shl    $0x8,%esi
   44f08:	or     %r14d,%esi
   44f0b:	mov    %esi,0x10(%rcx,%rax,1)
   44f0f:	movzwl 0xa(%rdi),%esi
   44f13:	add    %ebp,%esi
   44f15:	shl    $0x8,%esi
   44f18:	or     %r14d,%esi
   44f1b:	mov    %esi,0x14(%rcx,%rax,1)
   44f1f:	movzwl 0xc(%rdi),%esi
   44f23:	add    %ebp,%esi
   44f25:	shl    $0x8,%esi
   44f28:	or     %r14d,%esi
   44f2b:	mov    %esi,0x18(%rcx,%rax,1)
   44f2f:	movzwl 0xe(%rdi),%esi
   44f33:	add    %ebp,%esi
   44f35:	shl    $0x8,%esi
   44f38:	or     %r14d,%esi
   44f3b:	mov    %esi,0x1c(%rcx,%rax,1)
   44f3f:	movzwl 0x10(%rdi),%esi
   44f43:	add    %ebp,%esi
   44f45:	shl    $0x8,%esi
   44f48:	or     %r14d,%esi
   44f4b:	mov    %esi,0x20(%rcx,%rax,1)
   44f4f:	movzwl 0x12(%rdi),%esi
   44f53:	add    %ebp,%esi
   44f55:	shl    $0x8,%esi
   44f58:	or     %r14d,%esi
   44f5b:	mov    %esi,0x24(%rcx,%rax,1)
   44f5f:	movzwl 0x14(%rdi),%esi
   44f63:	add    %ebp,%esi
   44f65:	shl    $0x8,%esi
   44f68:	or     %r14d,%esi
   44f6b:	mov    %esi,0x28(%rcx,%rax,1)
   44f6f:	movzwl 0x16(%rdi),%esi
   44f73:	add    %ebp,%esi
   44f75:	shl    $0x8,%esi
   44f78:	or     %r14d,%esi
   44f7b:	mov    %esi,0x2c(%rcx,%rax,1)
   44f7f:	movzwl 0x18(%rdi),%esi
   44f83:	add    %ebp,%esi
   44f85:	shl    $0x8,%esi
   44f88:	or     %r14d,%esi
   44f8b:	mov    %esi,0x30(%rcx,%rax,1)
   44f8f:	movzwl 0x1a(%rdi),%esi
   44f93:	add    %ebp,%esi
   44f95:	shl    $0x8,%esi
   44f98:	or     %r14d,%esi
   44f9b:	mov    %esi,0x34(%rcx,%rax,1)
   44f9f:	movzwl 0x1c(%rdi),%esi
   44fa3:	add    %ebp,%esi
   44fa5:	shl    $0x8,%esi
   44fa8:	or     %r14d,%esi
   44fab:	mov    %esi,0x38(%rcx,%rax,1)
   44faf:	movzwl 0x1e(%rdi),%esi
   44fb3:	add    %ebp,%esi
   44fb5:	shl    $0x8,%esi
   44fb8:	or     %r14d,%esi
   44fbb:	mov    %esi,0x3c(%rcx,%rax,1)
   44fbf:	movzwl 0x20(%rdi),%esi
   44fc3:	add    %ebp,%esi
   44fc5:	shl    $0x8,%esi
   44fc8:	or     %r14d,%esi
   44fcb:	mov    %esi,0x40(%rcx,%rax,1)
   44fcf:	movzwl 0x22(%rdi),%esi
   44fd3:	add    %ebp,%esi
   44fd5:	shl    $0x8,%esi
   44fd8:	or     %r14d,%esi
   44fdb:	mov    %esi,0x44(%rcx,%rax,1)
   44fdf:	movzwl 0x24(%rdi),%esi
   44fe3:	add    %ebp,%esi
   44fe5:	shl    $0x8,%esi
   44fe8:	or     %r14d,%esi
   44feb:	mov    %esi,0x48(%rcx,%rax,1)
   44fef:	movzwl 0x26(%rdi),%esi
   44ff3:	add    %ebp,%esi
   44ff5:	shl    $0x8,%esi
   44ff8:	or     %r14d,%esi
   44ffb:	mov    %esi,0x4c(%rcx,%rax,1)
   44fff:	movzwl 0x28(%rdi),%esi
   45003:	add    %ebp,%esi
   45005:	shl    $0x8,%esi
   45008:	or     %r14d,%esi
   4500b:	mov    %esi,0x50(%rcx,%rax,1)
   4500f:	movzwl 0x2a(%rdi),%esi
   45013:	add    %ebp,%esi
   45015:	shl    $0x8,%esi
   45018:	or     %r14d,%esi
   4501b:	mov    %esi,0x54(%rcx,%rax,1)
   4501f:	movzwl 0x2c(%rdi),%esi
   45023:	add    %ebp,%esi
   45025:	shl    $0x8,%esi
   45028:	or     %r14d,%esi
   4502b:	mov    %esi,0x58(%rcx,%rax,1)
   4502f:	movzwl 0x2e(%rdi),%esi
   45033:	add    %ebp,%esi
   45035:	shl    $0x8,%esi
   45038:	or     %r14d,%esi
   4503b:	mov    %esi,0x5c(%rcx,%rax,1)
   4503f:	movzwl 0x30(%rdi),%esi
   45043:	add    %ebp,%esi
   45045:	shl    $0x8,%esi
   45048:	or     %r14d,%esi
   4504b:	mov    %esi,0x60(%rcx,%rax,1)
   4504f:	movzwl 0x32(%rdi),%esi
   45053:	add    %ebp,%esi
   45055:	shl    $0x8,%esi
   45058:	or     %r14d,%esi
   4505b:	mov    %esi,0x64(%rcx,%rax,1)
   4505f:	movzwl 0x34(%rdi),%esi
   45063:	add    %ebp,%esi
   45065:	shl    $0x8,%esi
   45068:	or     %r14d,%esi
   4506b:	mov    %esi,0x68(%rcx,%rax,1)
   4506f:	movzwl 0x36(%rdi),%esi
   45073:	add    %ebp,%esi
   45075:	shl    $0x8,%esi
   45078:	or     %r14d,%esi
   4507b:	mov    %esi,0x6c(%rcx,%rax,1)
   4507f:	movzwl 0x38(%rdi),%esi
   45083:	add    %ebp,%esi
   45085:	shl    $0x8,%esi
   45088:	or     %r14d,%esi
   4508b:	mov    %esi,0x70(%rcx,%rax,1)
   4508f:	movzwl 0x3a(%rdi),%esi
   45093:	add    %ebp,%esi
   45095:	shl    $0x8,%esi
   45098:	or     %r14d,%esi
   4509b:	mov    %esi,0x74(%rcx,%rax,1)
   4509f:	lea    (%r10,%r15,1),%eax
   450a3:	add    $0x20,%eax
   450a6:	mov    %eax,0x28f0(%rsp)
   450ad:	add    $0x20,%r15
   450b1:	inc    %r9
   450b4:	add    $0x2000,%r13d
   450bb:	dec    %rdx
   450be:	je     452cc <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120c>
   450c4:	mov    %r9,%rax
   450c7:	shr    $0x6,%rax
   450cb:	cmp    0x18(%rsp),%rax
   450d0:	jae    4583f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x177f>
   450d6:	mov    0x30(%rsp),%rcx
   450db:	mov    (%rcx,%rax,8),%rax
   450df:	bt     %r9,%rax
   450e3:	jae    45110 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1050>
   450e5:	cmp    $0x1e,%r12
   450e9:	jae    45865 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17a5>
   450ef:	mov    (%rbx,%r15,2),%eax
   450f3:	and    $0xfffff,%eax
   450f8:	mov    0x10(%r11),%rcx
   450fc:	cmp    %rax,%rcx
   450ff:	ja     44ec0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xe00>
   45105:	jmp    45853 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1793>
   4510a:	nopw   0x0(%rax,%rax,1)
   45110:	cmp    $0x1e,%r12
   45114:	jae    458a5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17e5>
   4511a:	movups 0x2c(%rdi),%xmm0
   4511e:	movups %xmm0,-0x10(%rbx,%r15,2)
   45124:	movdqa (%rdi),%xmm0
   45128:	movdqa 0x10(%rdi),%xmm1
   4512d:	movdqa 0x20(%rdi),%xmm2
   45132:	movdqa %xmm2,-0x1c(%rbx,%r15,2)
   45139:	movdqa %xmm1,-0x2c(%rbx,%r15,2)
   45140:	movdqa %xmm0,-0x3c(%rbx,%r15,2)
   45147:	mov    %r13d,(%rbx,%r15,2)
   4514b:	jmp    4509f <<c30walk::ctor2s::Ctor2s>::build::<false>+0xfdf>
   45150:	mov    0x28e8(%rsp),%rdi
   45158:	cmp    $0x1d,%rdi
   4515c:	ja     4593a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x187a>
   45162:	shl    $0x6,%rdi
   45166:	add    %r13,%rdi
   45169:	mov    0x28f0(%rsp),%eax
   45170:	mov    0xd0(%rsp),%rcx
   45178:	shl    $0x6,%r9
   4517c:	mov    %eax,%r15d
   4517f:	shl    $0x8,%r15d
   45183:	or     %r14d,%r15d
   45186:	lea    -0x1e(%r12),%rsi
   4518b:	cmp    $0x1e,%rsi
   4518f:	jae    451fd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x113d>
   45191:	xor    %esi,%esi
   45193:	jmp    4528a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11ca>
   45198:	shr    $1,%r9
   4519b:	inc    %r9
   4519e:	mov    %r9,%r10
   451a1:	and    $0xfffffffffffffff8,%r10
   451a5:	lea    (%rdx,%r10,2),%r8
   451a9:	movd   %ecx,%xmm0
   451ad:	pshuflw $0x0,%xmm0,%xmm0
   451b2:	add    %r13,%r11
   451b5:	add    $0x8,%r11
   451b9:	xor    %ebx,%ebx
   451bb:	nopl   0x0(%rax,%rax,1)
   451c0:	movq   -0x8(%r11,%rbx,2),%xmm1
   451c7:	movq   (%r11,%rbx,2),%xmm2
   451cd:	paddw  %xmm0,%xmm1
   451d1:	paddw  %xmm0,%xmm2
   451d5:	punpcklqdq %xmm2,%xmm1
   451d9:	movdqu %xmm1,(%rax,%rbx,2)
   451de:	add    $0x8,%rbx
   451e2:	cmp    %rbx,%r10
   451e5:	jne    451c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1100>
   451e7:	cmp    %r10,%r9
   451ea:	mov    0x80(%rsp),%rbx
   451f2:	jne    44d59 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc99>
   451f8:	jmp    44d89 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcc9>
   451fd:	mov    %rdx,%r10
   45200:	movabs $0xffffffffffffffe,%rsi
   4520a:	and    %rsi,%r10
   4520d:	lea    (%rcx,%r9,1),%r11
   45211:	add    $0x7c,%r11
   45215:	xor    %esi,%esi
   45217:	nopw   0x0(%rax,%rax,1)
   45220:	movdqu 0x2c(%rdi),%xmm0
   45225:	movdqu %xmm0,-0x50(%r11)
   4522b:	movdqa (%rdi),%xmm1
   4522f:	movdqa 0x10(%rdi),%xmm2
   45234:	movdqa 0x20(%rdi),%xmm3
   45239:	movdqa %xmm3,-0x5c(%r11)
   4523f:	movdqa %xmm2,-0x6c(%r11)
   45245:	movdqa %xmm1,-0x7c(%r11)
   4524b:	mov    %r15d,-0x40(%r11)
   4524f:	add    $0x2,%rsi
   45253:	movdqa %xmm1,-0x3c(%r11)
   45259:	movdqa %xmm2,-0x2c(%r11)
   4525f:	movdqa %xmm3,-0x1c(%r11)
   45265:	movdqu %xmm0,-0x10(%r11)
   4526b:	lea    0x2000(%r15),%ebx
   45272:	mov    %ebx,(%r11)
   45275:	add    $0x4000,%r15d
   4527c:	sub    $0xffffffffffffff80,%r11
   45280:	cmp    %rsi,%r10
   45283:	jne    45220 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1160>
   45285:	test   $0x1,%dl
   45288:	je     452be <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11fe>
   4528a:	add    %r9,%rcx
   4528d:	shl    $0x6,%rsi
   45291:	movups 0x2c(%rdi),%xmm0
   45295:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4529a:	movdqa (%rdi),%xmm0
   4529e:	movdqa 0x10(%rdi),%xmm1
   452a3:	movdqa 0x20(%rdi),%xmm2
   452a8:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   452ae:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   452b4:	movdqa %xmm0,(%rcx,%rsi,1)
   452b9:	mov    %r15d,0x3c(%rcx,%rsi,1)
   452be:	mov    %edx,%ecx
   452c0:	shl    $0x5,%ecx
   452c3:	add    %ecx,%eax
   452c5:	mov    %eax,0x28f0(%rsp)
   452cc:	mov    0x68(%rsp),%rdx
   452d1:	mov    %rdx,%rax
   452d4:	shl    $0x5,%rax
   452d8:	lea    (%rdx,%rdx,1),%rcx
   452dc:	sub    %rax,%rcx
   452df:	mov    0x58(%rsp),%r12
   452e4:	add    %rcx,%r12
   452e7:	lea    (%rdx,%rdx,1),%eax
   452ea:	shl    $0x5,%edx
   452ed:	sub    %eax,%edx
   452ef:	add    %edx,%r8d
   452f2:	mov    0x80(%rsp),%rbx
   452fa:	mov    %r8d,0x18(%rbx)
   452fe:	test   %r12,%r12
   45301:	mov    $0x88888889,%r15d
   45307:	lea    -0x282ce(%rip),%r13        # 1d040 <c30walk::ctor2s::PATTERNS>
   4530e:	je     45380 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   45310:	mov    0x1c(%rbx),%ecx
   45313:	mov    %r8d,%edi
   45316:	imul   %r15,%rdi
   4531a:	mov    %rdi,%rdx
   4531d:	shr    $0x24,%rdx
   45321:	mov    %edx,%eax
   45323:	shl    $0x5,%eax
   45326:	sub    %edx,%eax
   45328:	sub    %edx,%eax
   4532a:	lea    0x1e(%rax),%esi
   4532d:	cmp    %ecx,%esi
   4532f:	cmovb  %esi,%ecx
   45332:	shr    $0x2a,%rdi
   45336:	mov    0xb8(%rsp),%rsi
   4533e:	cmp    %rsi,%rdi
   45341:	jae    4591d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x185d>
   45347:	mov    %rdx,%rsi
   4534a:	shl    $0x6,%rsi
   4534e:	add    0xd0(%rsp),%rsi
   45356:	mov    0xb0(%rsp),%r9
   4535e:	mov    (%r9,%rdi,8),%rdi
   45362:	bt     %rdx,%rdi
   45366:	jb     44cc0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc00>
   4536c:	mov    $0xffffffff,%edi
   45371:	jmp    44ccc <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc0c>
   45376:	cs nopw 0x0(%rax,%rax,1)
   45380:	mov    0x8(%rsp),%rdi
   45385:	mov    0x50(%rsp),%r8
   4538a:	mov    0x38(%rsp),%r14
   4538f:	mov    0x28(%rsp),%r13
   45394:	mov    0x40(%rsp),%rcx
   45399:	mov    0x64(%rsp),%eax
   4539d:	mov    %r8,%r12
   453a0:	cmp    $0xf,%r8
   453a4:	ja     449e8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   453aa:	jmp    453b3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   453ac:	nopl   0x0(%rax)
   453b0:	mov    %r12,%rcx
   453b3:	test   %r8,%r8
   453b6:	je     4489c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7dc>
   453bc:	mov    0x20(%rsp),%r15
   453c1:	sub    %r8,%r15
   453c4:	mov    0x28e8(%rsp),%rax
   453cc:	mov    $0x1e,%ebx
   453d1:	sub    %rax,%rbx
   453d4:	cmp    %r8,%rbx
   453d7:	cmovae %r8,%rbx
   453db:	mov    %rbx,%r12
   453de:	add    %r15,%r12
   453e1:	lea    0xe8(%rsp),%rdx
   453e9:	jb     45879 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   453ef:	cmp    %rdi,%r12
   453f2:	ja     45879 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   453f8:	mov    %rcx,0x40(%rsp)
   453fd:	mov    %r14,0x38(%rsp)
   45402:	mov    0x28f0(%rsp),%r14d
   4540a:	cmp    $0x1e,%rax
   4540e:	jne    4549f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13df>
   45414:	cmp    %r12,0x20(%rsp)
   45419:	jb     4588e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17ce>
   4541f:	jne    45513 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1453>
   45425:	mov    0x28e8(%rsp),%rax
   4542d:	mov    0x28f0(%rsp),%ecx
   45434:	cmp    $0x1,%r8
   45438:	je     44868 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7a8>
   4543e:	mov    %r8d,%edx
   45441:	and    $0xe,%edx
   45444:	mov    0x38(%rsp),%r14
   45449:	nopl   0x0(%rax)
   45450:	lea    0x1(%rax),%rsi
   45454:	add    $0xffffffffffffffe3,%rax
   45458:	xor    %edi,%edi
   4545a:	cmp    $0x1e,%rsi
   4545e:	setae  %dil
   45462:	cmovb  %rsi,%rax
   45466:	lea    (%rcx,%rdi,2),%ecx
   45469:	lea    0x1(%rax),%rsi
   4546d:	add    $0xffffffffffffffe3,%rax
   45471:	xor    %edi,%edi
   45473:	cmp    $0x1e,%rsi
   45477:	setae  %dil
   4547b:	cmovb  %rsi,%rax
   4547f:	lea    (%rcx,%rdi,2),%ecx
   45482:	add    $0x2,%ecx
   45485:	add    $0xfffffffffffffffe,%rdx
   45489:	jne    45450 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1390>
   4548b:	test   $0x1,%r8b
   4548f:	mov    0x8(%rsp),%rdi
   45494:	jne    4486d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ad>
   4549a:	jmp    44888 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7c8>
   4549f:	add    0x48(%rsp),%r15
   454a4:	xor    %r13d,%r13d
   454a7:	nopw   0x0(%rax,%rax,1)
   454b0:	movzbl (%r15,%r13,1),%esi
   454b5:	lea    (%rsi,%rsi,4),%rcx
   454b9:	lea    (%rdx,%rcx,8),%rbp
   454bd:	mov    (%rdx,%rcx,8),%rax
   454c1:	cmp    0x8(%rdx,%rcx,8),%rax
   454c6:	je     454e1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1421>
   454c8:	lea    (%r14,%r13,1),%rcx
   454cc:	mov    %cx,(%rax)
   454cf:	addq   $0x2,0x0(%rbp)
   454d4:	inc    %r13
   454d7:	cmp    %r13,%rbx
   454da:	jne    454b0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13f0>
   454dc:	jmp    45414 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1354>
   454e1:	mov    %r8,0x50(%rsp)
   454e6:	lea    0xa8(%rsp),%rdi
   454ee:	call   95e90 <<c30walk::ctor2s::W<false>>::refill>
   454f3:	test   %al,%al
   454f5:	je     455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   454fb:	mov    0x0(%rbp),%rax
   454ff:	mov    0x8(%rsp),%rdi
   45504:	lea    0xe8(%rsp),%rdx
   4550c:	mov    0x50(%rsp),%r8
   45511:	jmp    454c8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1408>
   45513:	mov    0x48(%rsp),%rax
   45518:	mov    0x20(%rsp),%rcx
   4551d:	lea    (%rax,%rcx,1),%r15
   45521:	lea    (%r14,%rbx,1),%ebp
   45525:	add    $0x2,%ebp
   45528:	sub    %r8,%rbx
   4552b:	nopl   0x0(%rax,%rax,1)
   45530:	movzbl (%r15,%rbx,1),%esi
   45535:	lea    (%rsi,%rsi,4),%rcx
   45539:	lea    (%rdx,%rcx,8),%r14
   4553d:	mov    (%rdx,%rcx,8),%rax
   45541:	cmp    0x8(%rdx,%rcx,8),%rax
   45546:	je     4555b <<c30walk::ctor2s::Ctor2s>::build::<false>+0x149b>
   45548:	mov    %bp,(%rax)
   4554b:	addq   $0x2,(%r14)
   4554f:	inc    %ebp
   45551:	inc    %rbx
   45554:	jne    45530 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1470>
   45556:	jmp    45425 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1365>
   4555b:	mov    %r8,%r12
   4555e:	lea    0xa8(%rsp),%rdi
   45566:	call   95e90 <<c30walk::ctor2s::W<false>>::refill>
   4556b:	test   %al,%al
   4556d:	je     455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   4556f:	mov    (%r14),%rax
   45572:	mov    0x8(%rsp),%rdi
   45577:	lea    0xe8(%rsp),%rdx
   4557f:	mov    %r12,%r8
   45582:	jmp    45548 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1488>
   45584:	mov    %r15,%rdi
   45587:	call   fea70 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   4558c:	jmp    4492f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x86f>
   45591:	mov    $0x84,%eax
   45596:	cs nopw 0x0(%rax,%rax,1)
   455a0:	lea    0x50(%rax),%rcx
   455a4:	cmp    $0x28d4,%rcx
   455ab:	je     45624 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1564>
   455ad:	mov    0x64(%rsp,%rax,1),%rdx
   455b2:	sub    0x74(%rsp,%rax,1),%rdx
   455b7:	shr    $1,%rdx
   455ba:	add    0x7c(%rsp,%rax,1),%edx
   455be:	cmp    0x80(%rsp,%rax,1),%edx
   455c5:	jne    455ed <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   455c7:	mov    0x8c(%rsp,%rax,1),%rdx
   455cf:	sub    0x9c(%rsp,%rax,1),%rdx
   455d7:	shr    $1,%rdx
   455da:	add    0xa4(%rsp,%rax,1),%edx
   455e1:	cmp    0xa8(%rsp,%rax,1),%edx
   455e8:	mov    %rcx,%rax
   455eb:	je     455a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14e0>
   455ed:	mov    0x5530(%rsp),%rax
   455f5:	movq   $0x0,0x10(%rax)
   455fd:	xor    %eax,%eax
   455ff:	add    $0x54f8,%rsp
   45606:	pop    %rbx
   45607:	pop    %r12
   45609:	pop    %r13
   4560b:	pop    %r14
   4560d:	pop    %r15
   4560f:	pop    %rbp
   45610:	ret
   45611:	lea    0x103388(%rip),%rcx        # 1489a0 <__frame_dummy_init_array_entry+0x2110>
   45618:	mov    $0x1e,%edx
   4561d:	xor    %edi,%edi
   4561f:	call   f0c50 <core::slice::index::slice_index_fail>
   45624:	xor    %r14d,%r14d
   45627:	lea    0xa8(%rsp),%r15
   4562f:	mov    0x88(%rsp),%r12
   45637:	mov    0x8(%rsp),%rdx
   4563c:	mov    0x78(%rsp),%r13
   45641:	lea    0xe8(%rsp),%rbx
   45649:	jmp    45679 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15b9>
   4564b:	nopl   0x0(%rax,%rax,1)
   45650:	sub    %rcx,%rax
   45653:	inc    %r14
   45656:	shr    $1,%rax
   45659:	add    %eax,0x18(%rbx)
   4565c:	pxor   %xmm0,%xmm0
   45660:	movdqu %xmm0,(%rbx)
   45664:	movq   $0x0,0x10(%rbx)
   4566c:	add    $0x28,%rbx
   45670:	cmp    $0x100,%r14
   45677:	je     4569e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15de>
   45679:	mov    (%rbx),%rcx
   4567c:	mov    %rcx,%rax
   4567f:	cmp    0x10(%rbx),%rcx
   45683:	je     45650 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   45685:	mov    %r15,%rdi
   45688:	mov    %r14,%rsi
   4568b:	call   95890 <<c30walk::ctor2s::W<false>>::finish>
   45690:	mov    0x8(%rsp),%rdx
   45695:	mov    (%rbx),%rax
   45698:	mov    0x10(%rbx),%rcx
   4569c:	jmp    45650 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   4569e:	xor    %ecx,%ecx
   456a0:	mov    0x8(%r12),%rbx
   456a5:	test   %r13,%r13
   456a8:	setne  %al
   456ab:	je     456ef <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   456ad:	mov    %al,%cl
   456af:	mov    %r13,%rax
   456b2:	sub    %rcx,%rax
   456b5:	imul   $0xffffffffffffffe2,%rax,%rcx
   456b9:	add    %rdx,%rcx
   456bc:	cmp    $0x1e,%rcx
   456c0:	jae    456ef <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   456c2:	lea    (%rbx,%r13,4),%rcx
   456c6:	lea    (%rcx,%rdx,2),%rdi
   456ca:	add    $0xfffffffffffffffc,%rdi
   456ce:	lea    (%rax,%rax,1),%rcx
   456d2:	shl    $0x5,%rax
   456d6:	sub    %rcx,%rax
   456d9:	sub    %rdx,%rax
   456dc:	lea    0x3c(,%rax,2),%rdx
   456e4:	mov    $0xff,%esi
   456e9:	call   *0x107f01(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   456ef:	shl    $0x6,%r13
   456f3:	pcmpeqd %xmm0,%xmm0
   456f7:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   456fe:	movdqa %xmm0,0x20(%rbx,%r13,1)
   45705:	movdqa %xmm0,0x10(%rbx,%r13,1)
   4570c:	movdqa %xmm0,(%rbx,%r13,1)
   45712:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   4571b:	mov    0x70(%rsp),%rax
   45720:	mov    0x8(%rax),%rdx
   45724:	mov    0x10(%rax),%rcx
   45728:	mov    0x5530(%rsp),%rax
   45730:	mov    0x8(%rax),%r8
   45734:	mov    0x10(%rax),%r9
   45738:	lea    0xa8(%rsp),%rdi
   45740:	mov    %rbx,%rsi
   45743:	call   a5850 <c30walk::ctor::canonicalise>
   45748:	mov    0x90(%rsp),%rax
   45750:	mov    %rax,0x10(%r12)
   45755:	mov    $0x1,%al
   45757:	jmp    455ff <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153f>
   4575c:	mov    %rdi,%r12
   4575f:	test   %rax,%rax
   45762:	je     45774 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16b4>
   45764:	mov    0x8(%r9),%rdi
   45768:	mov    %r9,%r14
   4576b:	call   *0x107e8f(%rip)        # 14d600 <free@GLIBC_2.2.5>
   45771:	mov    %r14,%r9
   45774:	movq   $0x0,(%r9)
   4577b:	movq   $0x40,0x8(%r9)
   45783:	movq   $0x0,0x10(%r9)
   4578b:	mov    $0x40,%edx
   45790:	mov    %r9,%rdi
   45793:	lea    0x1(%r13),%rsi
   45797:	mov    %r9,%r14
   4579a:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4579f:	mov    %r14,%r9
   457a2:	cmp    $0xffffffffffffffff,%rax
   457a6:	mov    %r12,%rdi
   457a9:	je     442e3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x223>
   457af:	jmp    4580a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x174a>
   457b1:	mov    %rdi,%r12
   457b4:	mov    %r9,%r14
   457b7:	test   %rax,%rax
   457ba:	je     457ce <<c30walk::ctor2s::Ctor2s>::build::<false>+0x170e>
   457bc:	mov    0x5530(%rsp),%rax
   457c4:	mov    0x8(%rax),%rdi
   457c8:	call   *0x107e32(%rip)        # 14d600 <free@GLIBC_2.2.5>
   457ce:	mov    0x5530(%rsp),%rdi
   457d6:	movq   $0x0,(%rdi)
   457dd:	movq   $0x40,0x8(%rdi)
   457e5:	movq   $0x0,0x10(%rdi)
   457ed:	mov    $0x80,%edx
   457f2:	mov    %rbx,%rsi
   457f5:	call   9b420 <<alloc::raw_vec::RawVecInner>::grow_exact>
   457fa:	cmp    $0xffffffffffffffff,%rax
   457fe:	mov    %r14,%r9
   45801:	mov    %r12,%rdi
   45804:	je     442f7 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x237>
   4580a:	mov    %rax,%rdi
   4580d:	mov    %rdx,%rsi
   45810:	call   ee3db <alloc::raw_vec::handle_error>
   45815:	lea    0x1031b4(%rip),%rcx        # 1489d0 <__frame_dummy_init_array_entry+0x2140>
   4581c:	mov    %rdi,%rdx
   4581f:	mov    %r14,%rdi
   45822:	mov    %r15,%rsi
   45825:	call   f0c50 <core::slice::index::slice_index_fail>
   4582a:	lea    0x103187(%rip),%rcx        # 1489b8 <__frame_dummy_init_array_entry+0x2128>
   45831:	mov    %rdi,%rdx
   45834:	mov    %r15,%rdi
   45837:	mov    %r14,%rsi
   4583a:	call   f0c50 <core::slice::index::slice_index_fail>
   4583f:	lea    0x1012ba(%rip),%rdx        # 146b00 <__frame_dummy_init_array_entry+0x270>
   45846:	mov    %rax,%rdi
   45849:	mov    0x18(%rsp),%rsi
   4584e:	call   efe73 <core::panicking::panic_bounds_check>
   45853:	lea    0x103206(%rip),%rdx        # 148a60 <__frame_dummy_init_array_entry+0x21d0>
   4585a:	mov    %rax,%rdi
   4585d:	mov    %rcx,%rsi
   45860:	call   efe73 <core::panicking::panic_bounds_check>
   45865:	lea    0x1031dc(%rip),%rdx        # 148a48 <__frame_dummy_init_array_entry+0x21b8>
   4586c:	mov    $0x1e,%esi
   45871:	mov    %r12,%rdi
   45874:	call   efe73 <core::panicking::panic_bounds_check>
   45879:	lea    0x103150(%rip),%rcx        # 1489d0 <__frame_dummy_init_array_entry+0x2140>
   45880:	mov    %rdi,%rdx
   45883:	mov    %r15,%rdi
   45886:	mov    %r12,%rsi
   45889:	call   f0c50 <core::slice::index::slice_index_fail>
   4588e:	lea    0x103123(%rip),%rcx        # 1489b8 <__frame_dummy_init_array_entry+0x2128>
   45895:	mov    %rdi,%rdx
   45898:	mov    %r12,%rdi
   4589b:	mov    0x20(%rsp),%rsi
   458a0:	call   f0c50 <core::slice::index::slice_index_fail>
   458a5:	lea    0x103184(%rip),%rdx        # 148a30 <__frame_dummy_init_array_entry+0x21a0>
   458ac:	mov    $0x1e,%esi
   458b1:	mov    %r12,%rdi
   458b4:	call   efe73 <core::panicking::panic_bounds_check>
   458b9:	mov    $0x10,%edx
   458be:	mov    $0x4,%ecx
   458c3:	mov    $0x400,%r8d
   458c9:	mov    %rdi,%r14
   458cc:	mov    %r13,%rdi
   458cf:	xor    %esi,%esi
   458d1:	mov    %r9,%rbx
   458d4:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   458d9:	mov    %r14,%rdi
   458dc:	mov    %rbx,%r9
   458df:	jmp    44384 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2c4>
   458e4:	mov    %r9,%r13
   458e7:	mov    $0x8,%ecx
   458ec:	mov    $0x8,%r8d
   458f2:	mov    %rdi,%r14
   458f5:	xor    %esi,%esi
   458f7:	mov    %rbx,%rdx
   458fa:	call   6eef0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   458ff:	mov    %r14,%rdi
   45902:	mov    0x10(%r14),%r14
   45906:	mov    0x8(%rdi),%r12
   4590a:	lea    (%r12,%r14,8),%rdi
   4590e:	cmp    $0x1,%rbx
   45912:	jne    44330 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x270>
   45918:	jmp    44352 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   4591d:	lea    0x1011dc(%rip),%rdx        # 146b00 <__frame_dummy_init_array_entry+0x270>
   45924:	call   efe73 <core::panicking::panic_bounds_check>
   45929:	lea    0x103058(%rip),%rdx        # 148988 <__frame_dummy_init_array_entry+0x20f8>
   45930:	mov    $0x1e,%esi
   45935:	call   efe73 <core::panicking::panic_bounds_check>
   4593a:	lea    0x1030d7(%rip),%rdx        # 148a18 <__frame_dummy_init_array_entry+0x2188>
   45941:	mov    $0x1e,%esi
   45946:	call   efe73 <core::panicking::panic_bounds_check>
   4594b:	lea    0x102fbe(%rip),%rdx        # 148910 <__frame_dummy_init_array_entry+0x2080>
   45952:	call   efe73 <core::panicking::panic_bounds_check>
   45957:	lea    0x1011a2(%rip),%rdx        # 146b00 <__frame_dummy_init_array_entry+0x270>
   4595e:	mov    %r14,%rdi
   45961:	call   efe73 <core::panicking::panic_bounds_check>
