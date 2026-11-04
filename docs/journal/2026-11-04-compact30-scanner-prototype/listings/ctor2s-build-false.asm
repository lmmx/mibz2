# <c30walk::ctor2s::Ctor2s>::build::<false>
# size 0x18a6

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000041130 <<c30walk::ctor2s::Ctor2s>::build::<false>>:
   41130:	push   %rbp
   41131:	push   %r15
   41133:	push   %r14
   41135:	push   %r13
   41137:	push   %r12
   41139:	push   %rbx
   4113a:	sub    $0x1000,%rsp
   41141:	movq   $0x0,(%rsp)
   41149:	sub    $0x1000,%rsp
   41150:	movq   $0x0,(%rsp)
   41158:	sub    $0x1000,%rsp
   4115f:	movq   $0x0,(%rsp)
   41167:	sub    $0x1000,%rsp
   4116e:	movq   $0x0,(%rsp)
   41176:	sub    $0x1000,%rsp
   4117d:	movq   $0x0,(%rsp)
   41185:	sub    $0x4f8,%rsp
   4118c:	mov    %r8d,0x14(%rsp)
   41191:	mov    %rsi,0x48(%rsp)
   41196:	mov    0x5530(%rsp),%rax
   4119e:	movq   $0x0,0x10(%r9)
   411a6:	movq   $0x0,0x10(%rax)
   411ae:	cmp    $0xeffe2,%rdx
   411b5:	ja     4266d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   411bb:	mov    %rcx,%r15
   411be:	mov    %rdx,%rsi
   411c1:	pxor   %xmm0,%xmm0
   411c5:	xor    %eax,%eax
   411c7:	pxor   %xmm2,%xmm2
   411cb:	pxor   %xmm1,%xmm1
   411cf:	nop
   411d0:	movq   (%r15,%rax,4),%xmm3
   411d6:	movq   0x8(%r15,%rax,4),%xmm4
   411dd:	punpckldq %xmm0,%xmm3
   411e1:	paddq  %xmm2,%xmm3
   411e5:	punpckldq %xmm0,%xmm4
   411e9:	paddq  %xmm1,%xmm4
   411ed:	movq   0x10(%r15,%rax,4),%xmm2
   411f4:	movq   0x18(%r15,%rax,4),%xmm1
   411fb:	punpckldq %xmm0,%xmm2
   411ff:	paddq  %xmm3,%xmm2
   41203:	punpckldq %xmm0,%xmm1
   41207:	paddq  %xmm4,%xmm1
   4120b:	add    $0x8,%rax
   4120f:	cmp    $0x100,%rax
   41215:	jne    411d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa0>
   41217:	paddq  %xmm2,%xmm1
   4121b:	pshufd $0xee,%xmm1,%xmm0
   41220:	paddq  %xmm1,%xmm0
   41224:	movq   %xmm0,%rax
   41229:	cmp    %rsi,%rax
   4122c:	jne    4266d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153d>
   41232:	pxor   %xmm2,%xmm2
   41236:	xor    %eax,%eax
   41238:	movdqa -0x31b60(%rip),%xmm3        # f6e0 <GCC_except_table972+0xa0c>
   41240:	pxor   %xmm1,%xmm1
   41244:	pxor   %xmm0,%xmm0
   41248:	nopl   0x0(%rax,%rax,1)
   41250:	movq   (%r15,%rax,4),%xmm4
   41256:	movq   0x8(%r15,%rax,4),%xmm5
   4125d:	pshufd $0xd4,%xmm4,%xmm4
   41262:	pcmpeqd %xmm2,%xmm4
   41266:	pandn  %xmm3,%xmm4
   4126a:	paddq  %xmm1,%xmm4
   4126e:	pshufd $0xd4,%xmm5,%xmm5
   41273:	pcmpeqd %xmm2,%xmm5
   41277:	pandn  %xmm3,%xmm5
   4127b:	paddq  %xmm0,%xmm5
   4127f:	movq   0x10(%r15,%rax,4),%xmm0
   41286:	movq   0x18(%r15,%rax,4),%xmm6
   4128d:	pshufd $0xd4,%xmm0,%xmm1
   41292:	pcmpeqd %xmm2,%xmm1
   41296:	pandn  %xmm3,%xmm1
   4129a:	paddq  %xmm4,%xmm1
   4129e:	pshufd $0xd4,%xmm6,%xmm0
   412a3:	pcmpeqd %xmm2,%xmm0
   412a7:	pandn  %xmm3,%xmm0
   412ab:	paddq  %xmm5,%xmm0
   412af:	add    $0x8,%rax
   412b3:	cmp    $0x100,%rax
   412b9:	jne    41250 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120>
   412bb:	mov    $0x88888889,%r13d
   412c1:	imul   %rsi,%r13
   412c5:	shr    $0x24,%r13
   412c9:	imul   $0xeeeeeeef,%esi,%eax
   412cf:	ror    $1,%eax
   412d1:	cmp    $0x8888889,%eax
   412d6:	sbb    $0xffffffffffffffff,%r13
   412da:	mov    %r13,%rbx
   412dd:	cmpl   $0x0,0x14(%rsp)
   412e2:	jne    41336 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   412e4:	test   %rsi,%rsi
   412e7:	je     41334 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x204>
   412e9:	paddq  %xmm1,%xmm0
   412ed:	pshufd $0xee,%xmm0,%xmm1
   412f2:	paddq  %xmm0,%xmm1
   412f6:	movq   %xmm1,%rax
   412fb:	lea    -0x1(%rsi),%ecx
   412fe:	mov    $0x88888889,%edx
   41303:	imul   %rcx,%rdx
   41307:	shr    $0x23,%rdx
   4130b:	and    $0xfffffffe,%edx
   4130e:	lea    (%rdx,%rsi,1),%ecx
   41311:	dec    %ecx
   41313:	shr    $1,%ecx
   41315:	mov    $0x80010003,%ebx
   4131a:	imul   %rcx,%rbx
   4131e:	shr    $0x2e,%rbx
   41322:	inc    %ebx
   41324:	imul   %rax,%rbx
   41328:	dec    %rbx
   4132b:	cmp    %r13,%rbx
   4132e:	cmovae %r13,%rbx
   41332:	jmp    41336 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x206>
   41334:	xor    %ebx,%ebx
   41336:	mov    (%r9),%rax
   41339:	lea    0x1(%r13),%rcx
   4133d:	mov    %rcx,0x90(%rsp)
   41345:	cmp    %r13,%rax
   41348:	mov    %rsi,0x8(%rsp)
   4134d:	jbe    427cc <<c30walk::ctor2s::Ctor2s>::build::<false>+0x169c>
   41353:	mov    0x5530(%rsp),%rax
   4135b:	mov    (%rax),%rax
   4135e:	cmp    %rbx,%rax
   41361:	jb     42821 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16f1>
   41367:	movq   $0x0,0x10(%rdi)
   4136f:	lea    0x3f(%r13),%rbx
   41373:	shr    $0x6,%rbx
   41377:	mov    %rdi,0x70(%rsp)
   4137c:	mov    %r13,0x78(%rsp)
   41381:	je     413d6 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a6>
   41383:	cmp    (%rdi),%rbx
   41386:	ja     42954 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1824>
   4138c:	mov    %r9,%r13
   4138f:	xor    %r14d,%r14d
   41392:	mov    0x8(%rdi),%r12
   41396:	lea    (%r12,%r14,8),%rdi
   4139a:	cmp    $0x1,%rbx
   4139e:	je     413c2 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   413a0:	lea    -0x8(,%rbx,8),%rdx
   413a8:	xor    %esi,%esi
   413aa:	call   *0xf3ae0(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   413b0:	lea    (%r14,%rbx,1),%rax
   413b4:	lea    (%r12,%rax,8),%rdi
   413b8:	add    $0xfffffffffffffff8,%rdi
   413bc:	add    %rbx,%r14
   413bf:	dec    %r14
   413c2:	mov    %r13,%r9
   413c5:	movq   $0x0,(%rdi)
   413cc:	inc    %r14
   413cf:	mov    0x70(%rsp),%rdi
   413d4:	jmp    413d9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2a9>
   413d6:	xor    %r14d,%r14d
   413d9:	lea    0x18(%rdi),%r13
   413dd:	mov    %r14,0x10(%rdi)
   413e1:	movq   $0x0,0x28(%rdi)
   413e9:	cmpq   $0xf,0x18(%rdi)
   413ee:	jbe    42929 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17f9>
   413f4:	lea    0x30(%rdi),%r14
   413f8:	xor    %eax,%eax
   413fa:	pxor   %xmm0,%xmm0
   413fe:	movabs $0xffffffff00000000,%rcx
   41408:	nopl   0x0(%rax,%rax,1)
   41410:	movdqu %xmm0,0x2d08(%rsp,%rax,1)
   41419:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   41422:	mov    %rcx,0x2d18(%rsp,%rax,1)
   4142a:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   41433:	movdqu %xmm0,0x2d30(%rsp,%rax,1)
   4143c:	mov    %rcx,0x2d40(%rsp,%rax,1)
   41444:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   4144d:	movdqu %xmm0,0x2d58(%rsp,%rax,1)
   41456:	mov    %rcx,0x2d68(%rsp,%rax,1)
   4145e:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   41467:	movdqu %xmm0,0x2d80(%rsp,%rax,1)
   41470:	mov    %rcx,0x2d90(%rsp,%rax,1)
   41478:	add    $0xa0,%rax
   4147e:	cmp    $0x2800,%rax
   41484:	jne    41410 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2e0>
   41486:	lea    0xc(%r15),%rax
   4148a:	xor    %edx,%edx
   4148c:	mov    $0x25,%ecx
   41491:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   414a0:	mov    -0xc(%rax),%esi
   414a3:	add    %edx,%esi
   414a5:	mov    %edx,0x2c7c(%rsp,%rcx,4)
   414ac:	mov    %esi,0x2c80(%rsp,%rcx,4)
   414b3:	mov    -0x8(%rax),%edx
   414b6:	add    %esi,%edx
   414b8:	mov    %esi,0x2ca4(%rsp,%rcx,4)
   414bf:	mov    %edx,0x2ca8(%rsp,%rcx,4)
   414c6:	mov    -0x4(%rax),%esi
   414c9:	add    %edx,%esi
   414cb:	mov    %edx,0x2ccc(%rsp,%rcx,4)
   414d2:	mov    %esi,0x2cd0(%rsp,%rcx,4)
   414d9:	mov    (%rax),%edx
   414db:	add    %esi,%edx
   414dd:	mov    %esi,0x2cf4(%rsp,%rcx,4)
   414e4:	mov    %edx,0x2cf8(%rsp,%rcx,4)
   414eb:	add    $0x28,%rcx
   414ef:	add    $0x10,%rax
   414f3:	cmp    $0xa25,%rcx
   414fa:	jne    414a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x370>
   414fc:	mov    %r9,0x88(%rsp)
   41504:	mov    0x8(%r9),%rbx
   41508:	mov    0x8(%rdi),%rbp
   4150c:	mov    0x10(%rdi),%r12
   41510:	lea    0xe8(%rsp),%rdi
   41518:	lea    0x2cf8(%rsp),%rsi
   41520:	mov    $0x2800,%edx
   41525:	call   *0xf3985(%rip)        # 134eb0 <memcpy@GLIBC_2.14>
   4152b:	mov    %rbp,%r9
   4152e:	mov    %r12,%rsi
   41531:	mov    %rbx,0xd0(%rsp)
   41539:	mov    0x78(%rsp),%rax
   4153e:	mov    %rax,0xd8(%rsp)
   41546:	mov    0x8(%rsp),%r11
   4154b:	mov    %r11,0xe0(%rsp)
   41553:	mov    0x5530(%rsp),%rax
   4155b:	mov    %rax,0xa8(%rsp)
   41563:	mov    %rbp,0xb0(%rsp)
   4156b:	mov    %r12,0xb8(%rsp)
   41573:	mov    %r13,0xc0(%rsp)
   4157b:	movl   $0x0,0x28f0(%rsp)
   41586:	movq   $0x0,0x28e8(%rsp)
   41592:	mov    %r14,0xc8(%rsp)
   4159a:	mov    $0xffffffffffffffff,%rcx
   415a1:	xor    %r13d,%r13d
   415a4:	movabs $0xeeeeeeeeeeeeeeef,%r10
   415ae:	movabs $0x888888888888889,%r12
   415b8:	pxor   %xmm0,%xmm0
   415bc:	pcmpeqd %xmm1,%xmm1
   415c0:	xor    %r14d,%r14d
   415c3:	jmp    41673 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x543>
   415c8:	nopl   0x0(%rax,%rax,1)
   415d0:	mov    0x8(%rdi),%rax
   415d4:	mov    %rbp,%rcx
   415d7:	shl    $0x7,%rcx
   415db:	movdqa %xmm0,0x70(%rax,%rcx,1)
   415e1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   415e7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   415ed:	movdqa %xmm0,0x40(%rax,%rcx,1)
   415f3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   415f9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   415ff:	movdqa %xmm0,0x10(%rax,%rcx,1)
   41605:	movdqa %xmm0,(%rax,%rcx,1)
   4160a:	mov    %rbp,%rax
   4160d:	inc    %rax
   41610:	mov    %rax,0x10(%rdi)
   41614:	mov    %rdx,%rax
   41617:	shl    $0x6,%rax
   4161b:	or     $0x10000000,%ebp
   41621:	movdqu %xmm1,0x2c(%rbx,%rax,1)
   41627:	movdqa %xmm1,0x20(%rbx,%rax,1)
   4162d:	movdqa %xmm1,0x10(%rbx,%rax,1)
   41633:	movdqa %xmm1,(%rbx,%rax,1)
   41638:	mov    %ebp,0x3c(%rbx,%rax,1)
   4163c:	mov    %rdx,%rdi
   4163f:	shr    $0x6,%rdi
   41643:	cmp    %rsi,%rdi
   41646:	jae    429bb <<c30walk::ctor2s::Ctor2s>::build::<false>+0x188b>
   4164c:	mov    $0x1,%eax
   41651:	mov    %edx,%ecx
   41653:	shl    %cl,%rax
   41656:	or     %rax,(%r9,%rdi,8)
   4165a:	mov    %rdx,%rcx
   4165d:	add    $0x4,%r14
   41661:	cmp    $0x400,%r14
   41668:	mov    0x8(%rsp),%r11
   4166d:	je     4170e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x5de>
   41673:	mov    %r13,%rax
   41676:	mov    (%r15,%r14,1),%r13d
   4167a:	add    %rax,%r13
   4167d:	cmp    %r11,%r13
   41680:	setae  %r8b
   41684:	mov    %r13,%rax
   41687:	movabs $0x8888888888888889,%rdx
   41691:	mul    %rdx
   41694:	shr    $0x4,%rdx
   41698:	mov    %r13,%rax
   4169b:	imul   %r10,%rax
   4169f:	ror    $1,%rax
   416a2:	cmp    %r12,%rax
   416a5:	setb   %al
   416a8:	cmp    %rcx,%rdx
   416ab:	sete   %dil
   416af:	or     %r8b,%dil
   416b2:	or     %al,%dil
   416b5:	jne    4165d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x52d>
   416b7:	mov    0x5530(%rsp),%rdi
   416bf:	mov    0x10(%rdi),%rbp
   416c3:	cmp    (%rdi),%rbp
   416c6:	jne    415d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   416cc:	mov    %rdx,0x18(%rsp)
   416d1:	mov    %rsi,0x30(%rsp)
   416d6:	mov    %r9,0x58(%rsp)
   416db:	call   91340 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   416e0:	pcmpeqd %xmm1,%xmm1
   416e4:	pxor   %xmm0,%xmm0
   416e8:	movabs $0xeeeeeeeeeeeeeeef,%r10
   416f2:	mov    0x58(%rsp),%r9
   416f7:	mov    0x30(%rsp),%rsi
   416fc:	mov    0x5530(%rsp),%rdi
   41704:	mov    0x18(%rsp),%rdx
   41709:	jmp    415d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x4a0>
   4170e:	cmpl   $0x0,0x14(%rsp)
   41713:	je     418c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   41719:	mov    0xd8(%rsp),%r8
   41721:	test   %r8,%r8
   41724:	je     418c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   4172a:	mov    %r11,%r9
   4172d:	shl    $0x20,%r9
   41731:	mov    0x14(%rsp),%r10d
   41736:	mov    0xa8(%rsp),%rdi
   4173e:	mov    0xd0(%rsp),%rbp
   41746:	xor    %r12d,%r12d
   41749:	movabs $0xff51afd7ed558ccd,%rbx
   41753:	pxor   %xmm0,%xmm0
   41757:	pcmpeqd %xmm1,%xmm1
   4175b:	jmp    417e0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6b0>
   41760:	mov    0x8(%rdi),%rax
   41764:	mov    %rbx,%rcx
   41767:	shl    $0x7,%rcx
   4176b:	movdqa %xmm0,0x70(%rax,%rcx,1)
   41771:	movdqa %xmm0,0x60(%rax,%rcx,1)
   41777:	movdqa %xmm0,0x50(%rax,%rcx,1)
   4177d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   41783:	movdqa %xmm0,0x30(%rax,%rcx,1)
   41789:	movdqa %xmm0,0x20(%rax,%rcx,1)
   4178f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   41795:	movdqa %xmm0,(%rax,%rcx,1)
   4179a:	mov    %rbx,%rax
   4179d:	inc    %rax
   417a0:	mov    %rax,0x10(%rdi)
   417a4:	or     $0x10000000,%ebx
   417aa:	movdqu %xmm1,0x2c(%rbp)
   417af:	movdqa %xmm1,0x20(%rbp)
   417b4:	movdqa %xmm1,0x10(%rbp)
   417b9:	movdqa %xmm1,0x0(%rbp)
   417be:	mov    %ebx,0x3c(%rbp)
   417c1:	or     %r15,0x0(%r13,%r14,8)
   417c6:	movabs $0xff51afd7ed558ccd,%rbx
   417d0:	inc    %r12
   417d3:	add    $0x40,%rbp
   417d7:	cmp    %r12,%r8
   417da:	je     418c4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x794>
   417e0:	mov    %r12,%r14
   417e3:	shr    $0x6,%r14
   417e7:	mov    0xb8(%rsp),%rsi
   417ef:	cmp    %rsi,%r14
   417f2:	jae    429c7 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1897>
   417f8:	mov    0xb0(%rsp),%r13
   41800:	mov    0x0(%r13,%r14,8),%rax
   41805:	mov    $0x1,%r15d
   4180b:	mov    %r12d,%ecx
   4180e:	shl    %cl,%r15
   41811:	bt     %r12,%rax
   41815:	jb     417d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   41817:	mov    %r12,%rax
   4181a:	xor    %r9,%rax
   4181d:	mov    %rax,%rcx
   41820:	shr    $0x21,%rcx
   41824:	xor    %rax,%rcx
   41827:	imul   %rbx,%rcx
   4182b:	mov    %rcx,%rax
   4182e:	shr    $0x21,%rax
   41832:	xor    %rcx,%rax
   41835:	movabs $0xc4ceb9fe1a85ec53,%rcx
   4183f:	imul   %rcx,%rax
   41843:	mov    %rax,%rcx
   41846:	shr    $0x21,%rcx
   4184a:	xor    %rax,%rcx
   4184d:	mov    %rcx,%rax
   41850:	shr    $0x3,%rax
   41854:	movabs $0x20c49ba5e353f7cf,%rdx
   4185e:	mul    %rdx
   41861:	shr    $0x4,%rdx
   41865:	imul   $0x3e8,%rdx,%rax
   4186c:	sub    %rax,%rcx
   4186f:	cmp    %r10,%rcx
   41872:	jae    417d0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x6a0>
   41878:	mov    0x10(%rdi),%rbx
   4187c:	cmp    (%rdi),%rbx
   4187f:	jne    41760 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   41885:	mov    %r8,0x18(%rsp)
   4188a:	mov    %r9,0x30(%rsp)
   4188f:	mov    %r10,0x58(%rsp)
   41894:	mov    %rdi,0x68(%rsp)
   41899:	call   91340 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   4189e:	pcmpeqd %xmm1,%xmm1
   418a2:	pxor   %xmm0,%xmm0
   418a6:	mov    0x68(%rsp),%rdi
   418ab:	mov    0x58(%rsp),%r10
   418b0:	mov    0x30(%rsp),%r9
   418b5:	mov    0x18(%rsp),%r8
   418ba:	mov    0x8(%rsp),%r11
   418bf:	jmp    41760 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x630>
   418c4:	test   %r11,%r11
   418c7:	je     42601 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   418cd:	movq   $0x0,0x20(%rsp)
   418d6:	jmp    4191d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ed>
   418d8:	mov    0x38(%rsp),%r14
   418dd:	lea    0x1(%rax),%rdx
   418e1:	add    $0xffffffffffffffe3,%rax
   418e5:	xor    %esi,%esi
   418e7:	cmp    $0x1e,%rdx
   418eb:	setae  %sil
   418ef:	cmovb  %rdx,%rax
   418f3:	lea    (%rcx,%rsi,2),%ecx
   418f6:	inc    %ecx
   418f8:	mov    %rax,0x28e8(%rsp)
   41900:	mov    %ecx,0x28f0(%rsp)
   41907:	mov    0x40(%rsp),%rcx
   4190c:	mov    %rcx,%r12
   4190f:	cmp    %rdi,0x98(%rsp)
   41917:	jae    42601 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14d1>
   4191d:	mov    0x20(%rsp),%r13
   41922:	mov    $0x400,%edx
   41927:	lea    0x28f8(%rsp),%rdi
   4192f:	xor    %esi,%esi
   41931:	call   *0xf3559(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   41937:	lea    0x128(%rsp),%rax
   4193f:	xor    %ecx,%ecx
   41941:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41950:	mov    -0x40(%rax),%rdx
   41954:	sub    -0x30(%rax),%rdx
   41958:	shr    $1,%rdx
   4195b:	add    -0x28(%rax),%edx
   4195e:	mov    -0x18(%rax),%rsi
   41962:	sub    -0x8(%rax),%rsi
   41966:	mov    %edx,0x28f8(%rsp,%rcx,4)
   4196d:	shr    $1,%rsi
   41970:	add    (%rax),%esi
   41972:	mov    %esi,0x28fc(%rsp,%rcx,4)
   41979:	add    $0x2,%rcx
   4197d:	add    $0x50,%rax
   41981:	cmp    $0x100,%rcx
   41988:	jne    41950 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x820>
   4198a:	mov    0xc0(%rsp),%r15
   41992:	mov    0x10(%r15),%rbx
   41996:	cmp    (%r15),%rbx
   41999:	je     425f4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14c4>
   4199f:	mov    %rbx,%rdi
   419a2:	shl    $0xa,%rdi
   419a6:	add    0x8(%r15),%rdi
   419aa:	mov    $0x400,%edx
   419af:	lea    0x28f8(%rsp),%rsi
   419b7:	call   *0xf34f3(%rip)        # 134eb0 <memcpy@GLIBC_2.14>
   419bd:	inc    %rbx
   419c0:	lea    0xf000(%r13),%rax
   419c7:	mov    0x8(%rsp),%rdi
   419cc:	cmp    %rdi,%rax
   419cf:	mov    %rdi,%rcx
   419d2:	mov    %rax,0x98(%rsp)
   419da:	cmovb  %rax,%rcx
   419de:	mov    %rbx,0x10(%r15)
   419e2:	mov    %rcx,0x20(%rsp)
   419e7:	mov    %rcx,%r8
   419ea:	sub    %r13,%r8
   419ed:	cmp    $0x10,%r8
   419f1:	jb     42420 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f0>
   419f7:	add    0x48(%rsp),%r13
   419fc:	xor    %eax,%eax
   419fe:	mov    %r12,%rcx
   41a01:	mov    %r8,%r12
   41a04:	jmp    41a58 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   41a06:	cs nopw 0x0(%rax,%rax,1)
   41a10:	mov    0x28e8(%rsp),%rax
   41a18:	lea    0x10(%rax),%rcx
   41a1c:	add    $0xfffffffffffffff2,%rax
   41a20:	xor    %edx,%edx
   41a22:	cmp    $0x1e,%rcx
   41a26:	setae  %dl
   41a29:	cmovb  %rcx,%rax
   41a2d:	mov    %rax,0x28e8(%rsp)
   41a35:	mov    0x28f0(%rsp),%eax
   41a3c:	lea    (%rax,%rdx,2),%eax
   41a3f:	add    $0x10,%eax
   41a42:	mov    %eax,0x28f0(%rsp)
   41a49:	xor    %eax,%eax
   41a4b:	mov    %r8,%r12
   41a4e:	cmp    $0xf,%r8
   41a52:	jbe    42423 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   41a58:	test   $0x1,%al
   41a5a:	jne    41a64 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x934>
   41a5c:	mov    0x0(%r13),%rcx
   41a60:	mov    0x8(%r13),%r14
   41a64:	add    $0x10,%r13
   41a68:	lea    -0x10(%r12),%r8
   41a6d:	xor    %rcx,%r14
   41a70:	mov    %rcx,%rax
   41a73:	rol    $0x8,%rax
   41a77:	xor    %rcx,%rax
   41a7a:	or     %r14,%rax
   41a7d:	lea    0xe8(%rsp),%rdx
   41a85:	jne    41ae0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x9b0>
   41a87:	cmp    $0x10,%r8
   41a8b:	jb     41b50 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa20>
   41a91:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41aa0:	mov    0x0(%r13),%rdx
   41aa4:	mov    0x8(%r13),%rax
   41aa8:	mov    $0x1,%r9b
   41aab:	cmp    %rcx,%rdx
   41aae:	jne    41ca2 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   41ab4:	cmp    %rcx,%rax
   41ab7:	jne    41ca2 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb72>
   41abd:	add    $0xfffffffffffffff0,%r8
   41ac1:	add    $0x10,%r13
   41ac5:	cmp    $0xf,%r8
   41ac9:	ja     41aa0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x970>
   41acb:	mov    %r13,0x28(%rsp)
   41ad0:	xor    %r9d,%r9d
   41ad3:	mov    %rcx,0x40(%rsp)
   41ad8:	jmp    41cb1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   41add:	nopl   (%rax)
   41ae0:	mov    0x20(%rsp),%r14
   41ae5:	sub    %r12,%r14
   41ae8:	mov    0x28e8(%rsp),%rax
   41af0:	mov    $0x1e,%r9d
   41af6:	sub    %rax,%r9
   41af9:	cmp    $0x10,%r9
   41afd:	mov    $0x10,%ebx
   41b02:	cmovb  %r9,%rbx
   41b06:	mov    %rbx,%r15
   41b09:	add    %r14,%r15
   41b0c:	jb     42885 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   41b12:	cmp    %rdi,%r15
   41b15:	ja     42885 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1755>
   41b1b:	mov    0x28f0(%rsp),%r12d
   41b23:	cmp    $0x1e,%rax
   41b27:	jne    41b60 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa30>
   41b29:	add    $0x10,%r14
   41b2d:	cmp    %rdi,%r14
   41b30:	ja     4289a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41b36:	cmp    %r15,%r14
   41b39:	jae    41c25 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xaf5>
   41b3f:	jmp    4289a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41b44:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   41b50:	mov    %r13,0x28(%rsp)
   41b55:	xor    %r9d,%r9d
   41b58:	jmp    41cb1 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb81>
   41b5d:	nopl   (%rax)
   41b60:	mov    %r13,0x28(%rsp)
   41b65:	mov    0x48(%rsp),%rax
   41b6a:	lea    (%rax,%r14,1),%r10
   41b6e:	xor    %r13d,%r13d
   41b71:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41b80:	movzbl (%r10,%r13,1),%esi
   41b85:	lea    (%rsi,%rsi,4),%rcx
   41b89:	lea    (%rdx,%rcx,8),%rbp
   41b8d:	mov    (%rdx,%rcx,8),%rax
   41b91:	cmp    0x8(%rdx,%rcx,8),%rax
   41b96:	je     41bae <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa7e>
   41b98:	lea    (%r12,%r13,1),%rcx
   41b9c:	mov    %cx,(%rax)
   41b9f:	addq   $0x2,0x0(%rbp)
   41ba4:	inc    %r13
   41ba7:	cmp    %r13,%rbx
   41baa:	jne    41b80 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa50>
   41bac:	jmp    41c00 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xad0>
   41bae:	mov    %r10,0x30(%rsp)
   41bb3:	mov    %r9,0x18(%rsp)
   41bb8:	mov    %r8,0x50(%rsp)
   41bbd:	lea    0xa8(%rsp),%rdi
   41bc5:	call   8c100 <<c30walk::ctor2s::W<false>>::refill>
   41bca:	test   %al,%al
   41bcc:	je     4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   41bd2:	mov    0x0(%rbp),%rax
   41bd6:	mov    0x8(%rsp),%rdi
   41bdb:	lea    0xe8(%rsp),%rdx
   41be3:	mov    0x50(%rsp),%r8
   41be8:	mov    0x18(%rsp),%r9
   41bed:	mov    0x30(%rsp),%r10
   41bf2:	jmp    41b98 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xa68>
   41bf4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   41c00:	add    $0x10,%r14
   41c04:	cmp    %rdi,%r14
   41c07:	ja     4289a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41c0d:	cmp    %r15,%r14
   41c10:	mov    0x28(%rsp),%r13
   41c15:	jb     4289a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x176a>
   41c1b:	cmp    $0xf,%r9
   41c1f:	ja     41a10 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   41c25:	add    0x48(%rsp),%r15
   41c2a:	mov    %r15,%r14
   41c2d:	sub    %rbx,%r14
   41c30:	add    $0x10,%r14
   41c34:	add    %r12d,%ebx
   41c37:	add    $0x2,%ebx
   41c3a:	nopw   0x0(%rax,%rax,1)
   41c40:	movzbl (%r15),%esi
   41c44:	lea    (%rsi,%rsi,4),%rcx
   41c48:	lea    (%rdx,%rcx,8),%r12
   41c4c:	mov    (%rdx,%rcx,8),%rax
   41c50:	cmp    0x8(%rdx,%rcx,8),%rax
   41c55:	je     41c6e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb3e>
   41c57:	mov    %bx,(%rax)
   41c5a:	addq   $0x2,(%r12)
   41c5f:	inc    %r15
   41c62:	inc    %ebx
   41c64:	cmp    %r14,%r15
   41c67:	jne    41c40 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb10>
   41c69:	jmp    41a10 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x8e0>
   41c6e:	mov    %r13,%rbp
   41c71:	mov    %r8,%r13
   41c74:	lea    0xa8(%rsp),%rdi
   41c7c:	call   8c100 <<c30walk::ctor2s::W<false>>::refill>
   41c81:	test   %al,%al
   41c83:	je     4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   41c89:	mov    (%r12),%rax
   41c8d:	mov    0x8(%rsp),%rdi
   41c92:	lea    0xe8(%rsp),%rdx
   41c9a:	mov    %r13,%r8
   41c9d:	mov    %rbp,%r13
   41ca0:	jmp    41c57 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xb27>
   41ca2:	mov    %rdx,0x40(%rsp)
   41ca7:	mov    %r13,0x28(%rsp)
   41cac:	mov    %rax,0x38(%rsp)
   41cb1:	mov    %r8,0x50(%rsp)
   41cb6:	sub    %r8,%r12
   41cb9:	movzbl %cl,%eax
   41cbc:	mov    %rax,0xa0(%rsp)
   41cc4:	lea    (%rax,%rax,4),%rdx
   41cc8:	lea    0xe8(%rsp),%r8
   41cd0:	mov    (%r8,%rdx,8),%rax
   41cd4:	mov    0x1c(%r8,%rdx,8),%esi
   41cd9:	mov    %rax,%rdi
   41cdc:	sub    0x10(%r8,%rdx,8),%rdi
   41ce1:	shr    $1,%rdi
   41ce4:	add    0x18(%r8,%rdx,8),%edi
   41ce9:	add    %r12,%rdi
   41cec:	cmp    %rsi,%rdi
   41cef:	ja     4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   41cf5:	mov    %r9d,0x64(%rsp)
   41cfa:	lea    0xe8(%rsp),%rsi
   41d02:	lea    (%rsi,%rdx,8),%rbx
   41d06:	movzbl %cl,%r14d
   41d0a:	mov    0x8(%rbx),%rsi
   41d0e:	mov    $0x88888889,%r15d
   41d14:	lea    -0x2689b(%rip),%r13        # 1b480 <c30walk::ctor2s::PATTERNS>
   41d1b:	mov    %rbx,0x80(%rsp)
   41d23:	jmp    41d5e <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc2e>
   41d25:	data16 cs nopw 0x0(%rax,%rax,1)
   41d30:	mov    0x3c(%rsi),%edi
   41d33:	mov    $0xfffff,%r9d
   41d39:	and    %r9d,%edi
   41d3c:	sub    %r8d,%ecx
   41d3f:	sub    %eax,%r8d
   41d42:	mov    %r8d,%eax
   41d45:	lea    (%rsi,%rax,2),%rax
   41d49:	mov    %rax,0x10(%rbx)
   41d4d:	mov    %rax,(%rbx)
   41d50:	lea    (%rax,%rcx,2),%rsi
   41d54:	mov    %rsi,0x8(%rbx)
   41d58:	mov    %edx,0x20(%rbx)
   41d5b:	mov    %edi,0x24(%rbx)
   41d5e:	sub    %rax,%rsi
   41d61:	shr    $1,%rsi
   41d64:	cmp    %rsi,%r12
   41d67:	cmovb  %r12,%rsi
   41d6b:	test   %rsi,%rsi
   41d6e:	je     41e28 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcf8>
   41d74:	mov    0x28e8(%rsp),%rdi
   41d7c:	cmp    $0x1d,%rdi
   41d80:	ja     42999 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1869>
   41d86:	cmp    $0x1f,%rsi
   41d8a:	jae    42681 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1551>
   41d90:	mov    0x28f0(%rsp),%ecx
   41d97:	mov    %rdi,%r11
   41d9a:	shl    $0x6,%r11
   41d9e:	lea    (%r11,%r13,1),%rdx
   41da2:	lea    -0x2(,%rsi,2),%r9
   41daa:	cmp    $0xe,%r9
   41dae:	jb     41dc3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc93>
   41db0:	mov    %rax,%r8
   41db3:	sub    %r13,%r8
   41db6:	sub    %r11,%r8
   41db9:	cmp    $0x10,%r8
   41dbd:	jae    42208 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x10d8>
   41dc3:	mov    %rdx,%r8
   41dc6:	xor    %r10d,%r10d
   41dc9:	lea    (%rdx,%rsi,2),%rdx
   41dcd:	lea    (%rax,%r10,2),%r9
   41dd1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   41de0:	movzwl (%r8),%r10d
   41de4:	add    %cx,%r10w
   41de8:	add    $0x2,%r8
   41dec:	mov    %r10w,(%r9)
   41df0:	add    $0x2,%r9
   41df4:	cmp    %rdx,%r8
   41df7:	jne    41de0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcb0>
   41df9:	lea    (%rax,%rsi,2),%rax
   41dfd:	mov    %rax,(%rbx)
   41e00:	add    %rsi,%rdi
   41e03:	lea    0x2(%rsi),%edx
   41e06:	mov    %rdi,%r8
   41e09:	sub    $0x1e,%r8
   41e0d:	cmovb  %rdi,%r8
   41e11:	cmovb  %esi,%edx
   41e14:	mov    %r8,0x28e8(%rsp)
   41e1c:	add    %ecx,%edx
   41e1e:	mov    %edx,0x28f0(%rsp)
   41e25:	sub    %rsi,%r12
   41e28:	test   %r12,%r12
   41e2b:	je     423f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   41e31:	mov    %rax,%r8
   41e34:	cmp    0x10(%rbx),%rax
   41e38:	je     41e56 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xd26>
   41e3a:	lea    0xa8(%rsp),%rdi
   41e42:	mov    0xa0(%rsp),%rsi
   41e4a:	call   8bb00 <<c30walk::ctor2s::W<false>>::finish>
   41e4f:	mov    (%rbx),%r8
   41e52:	mov    0x10(%rbx),%rax
   41e56:	sub    %rax,%r8
   41e59:	shr    $1,%r8
   41e5c:	add    0x18(%rbx),%r8d
   41e60:	mov    %r8d,0x18(%rbx)
   41e64:	pxor   %xmm0,%xmm0
   41e68:	movdqu %xmm0,(%rbx)
   41e6c:	movq   $0x0,0x10(%rbx)
   41e74:	cmp    $0x1e,%r12
   41e78:	jb     42380 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   41e7e:	mov    %r8,%rcx
   41e81:	imul   %r15,%rcx
   41e85:	shr    $0x24,%rcx
   41e89:	mov    %ecx,%eax
   41e8b:	shl    $0x5,%eax
   41e8e:	lea    (%rcx,%rcx,1),%edx
   41e91:	sub    %eax,%edx
   41e93:	add    %r8d,%edx
   41e96:	jne    42380 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1250>
   41e9c:	mov    %r12,%rax
   41e9f:	movabs $0x8888888888888889,%rdx
   41ea9:	mul    %rdx
   41eac:	shr    $0x4,%rdx
   41eb0:	mov    %ecx,%r9d
   41eb3:	cmpl   $0x0,0x14(%rsp)
   41eb8:	mov    %r12,0x58(%rsp)
   41ebd:	mov    %rdx,0x68(%rsp)
   41ec2:	je     421c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1090>
   41ec8:	mov    0x28f0(%rsp),%r10d
   41ed0:	mov    0xb8(%rsp),%rax
   41ed8:	mov    %rax,0x18(%rsp)
   41edd:	mov    0xa8(%rsp),%r11
   41ee5:	mov    0xb0(%rsp),%rax
   41eed:	mov    %rax,0x30(%rsp)
   41ef2:	mov    0xd0(%rsp),%rax
   41efa:	mov    0x28e8(%rsp),%r12
   41f02:	mov    %r12,%rdi
   41f05:	shl    $0x6,%rdi
   41f09:	add    %r13,%rdi
   41f0c:	mov    %r9,%rcx
   41f0f:	shl    $0x6,%rcx
   41f13:	lea    (%rax,%rcx,1),%rbx
   41f17:	add    $0x3c,%rbx
   41f1b:	mov    %r10d,%r13d
   41f1e:	shl    $0x8,%r13d
   41f22:	or     %r14d,%r13d
   41f25:	xor    %r15d,%r15d
   41f28:	jmp    42134 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1004>
   41f2d:	nopl   (%rax)
   41f30:	mov    0x8(%r11),%rcx
   41f34:	shl    $0x7,%eax
   41f37:	mov    %r13d,(%rcx,%rax,1)
   41f3b:	movzwl 0x2(%rdi),%esi
   41f3f:	lea    (%r10,%r15,1),%ebp
   41f43:	add    %ebp,%esi
   41f45:	shl    $0x8,%esi
   41f48:	or     %r14d,%esi
   41f4b:	mov    %esi,0x4(%rcx,%rax,1)
   41f4f:	movzwl 0x4(%rdi),%esi
   41f53:	add    %ebp,%esi
   41f55:	shl    $0x8,%esi
   41f58:	or     %r14d,%esi
   41f5b:	mov    %esi,0x8(%rcx,%rax,1)
   41f5f:	movzwl 0x6(%rdi),%esi
   41f63:	add    %ebp,%esi
   41f65:	shl    $0x8,%esi
   41f68:	or     %r14d,%esi
   41f6b:	mov    %esi,0xc(%rcx,%rax,1)
   41f6f:	movzwl 0x8(%rdi),%esi
   41f73:	add    %ebp,%esi
   41f75:	shl    $0x8,%esi
   41f78:	or     %r14d,%esi
   41f7b:	mov    %esi,0x10(%rcx,%rax,1)
   41f7f:	movzwl 0xa(%rdi),%esi
   41f83:	add    %ebp,%esi
   41f85:	shl    $0x8,%esi
   41f88:	or     %r14d,%esi
   41f8b:	mov    %esi,0x14(%rcx,%rax,1)
   41f8f:	movzwl 0xc(%rdi),%esi
   41f93:	add    %ebp,%esi
   41f95:	shl    $0x8,%esi
   41f98:	or     %r14d,%esi
   41f9b:	mov    %esi,0x18(%rcx,%rax,1)
   41f9f:	movzwl 0xe(%rdi),%esi
   41fa3:	add    %ebp,%esi
   41fa5:	shl    $0x8,%esi
   41fa8:	or     %r14d,%esi
   41fab:	mov    %esi,0x1c(%rcx,%rax,1)
   41faf:	movzwl 0x10(%rdi),%esi
   41fb3:	add    %ebp,%esi
   41fb5:	shl    $0x8,%esi
   41fb8:	or     %r14d,%esi
   41fbb:	mov    %esi,0x20(%rcx,%rax,1)
   41fbf:	movzwl 0x12(%rdi),%esi
   41fc3:	add    %ebp,%esi
   41fc5:	shl    $0x8,%esi
   41fc8:	or     %r14d,%esi
   41fcb:	mov    %esi,0x24(%rcx,%rax,1)
   41fcf:	movzwl 0x14(%rdi),%esi
   41fd3:	add    %ebp,%esi
   41fd5:	shl    $0x8,%esi
   41fd8:	or     %r14d,%esi
   41fdb:	mov    %esi,0x28(%rcx,%rax,1)
   41fdf:	movzwl 0x16(%rdi),%esi
   41fe3:	add    %ebp,%esi
   41fe5:	shl    $0x8,%esi
   41fe8:	or     %r14d,%esi
   41feb:	mov    %esi,0x2c(%rcx,%rax,1)
   41fef:	movzwl 0x18(%rdi),%esi
   41ff3:	add    %ebp,%esi
   41ff5:	shl    $0x8,%esi
   41ff8:	or     %r14d,%esi
   41ffb:	mov    %esi,0x30(%rcx,%rax,1)
   41fff:	movzwl 0x1a(%rdi),%esi
   42003:	add    %ebp,%esi
   42005:	shl    $0x8,%esi
   42008:	or     %r14d,%esi
   4200b:	mov    %esi,0x34(%rcx,%rax,1)
   4200f:	movzwl 0x1c(%rdi),%esi
   42013:	add    %ebp,%esi
   42015:	shl    $0x8,%esi
   42018:	or     %r14d,%esi
   4201b:	mov    %esi,0x38(%rcx,%rax,1)
   4201f:	movzwl 0x1e(%rdi),%esi
   42023:	add    %ebp,%esi
   42025:	shl    $0x8,%esi
   42028:	or     %r14d,%esi
   4202b:	mov    %esi,0x3c(%rcx,%rax,1)
   4202f:	movzwl 0x20(%rdi),%esi
   42033:	add    %ebp,%esi
   42035:	shl    $0x8,%esi
   42038:	or     %r14d,%esi
   4203b:	mov    %esi,0x40(%rcx,%rax,1)
   4203f:	movzwl 0x22(%rdi),%esi
   42043:	add    %ebp,%esi
   42045:	shl    $0x8,%esi
   42048:	or     %r14d,%esi
   4204b:	mov    %esi,0x44(%rcx,%rax,1)
   4204f:	movzwl 0x24(%rdi),%esi
   42053:	add    %ebp,%esi
   42055:	shl    $0x8,%esi
   42058:	or     %r14d,%esi
   4205b:	mov    %esi,0x48(%rcx,%rax,1)
   4205f:	movzwl 0x26(%rdi),%esi
   42063:	add    %ebp,%esi
   42065:	shl    $0x8,%esi
   42068:	or     %r14d,%esi
   4206b:	mov    %esi,0x4c(%rcx,%rax,1)
   4206f:	movzwl 0x28(%rdi),%esi
   42073:	add    %ebp,%esi
   42075:	shl    $0x8,%esi
   42078:	or     %r14d,%esi
   4207b:	mov    %esi,0x50(%rcx,%rax,1)
   4207f:	movzwl 0x2a(%rdi),%esi
   42083:	add    %ebp,%esi
   42085:	shl    $0x8,%esi
   42088:	or     %r14d,%esi
   4208b:	mov    %esi,0x54(%rcx,%rax,1)
   4208f:	movzwl 0x2c(%rdi),%esi
   42093:	add    %ebp,%esi
   42095:	shl    $0x8,%esi
   42098:	or     %r14d,%esi
   4209b:	mov    %esi,0x58(%rcx,%rax,1)
   4209f:	movzwl 0x2e(%rdi),%esi
   420a3:	add    %ebp,%esi
   420a5:	shl    $0x8,%esi
   420a8:	or     %r14d,%esi
   420ab:	mov    %esi,0x5c(%rcx,%rax,1)
   420af:	movzwl 0x30(%rdi),%esi
   420b3:	add    %ebp,%esi
   420b5:	shl    $0x8,%esi
   420b8:	or     %r14d,%esi
   420bb:	mov    %esi,0x60(%rcx,%rax,1)
   420bf:	movzwl 0x32(%rdi),%esi
   420c3:	add    %ebp,%esi
   420c5:	shl    $0x8,%esi
   420c8:	or     %r14d,%esi
   420cb:	mov    %esi,0x64(%rcx,%rax,1)
   420cf:	movzwl 0x34(%rdi),%esi
   420d3:	add    %ebp,%esi
   420d5:	shl    $0x8,%esi
   420d8:	or     %r14d,%esi
   420db:	mov    %esi,0x68(%rcx,%rax,1)
   420df:	movzwl 0x36(%rdi),%esi
   420e3:	add    %ebp,%esi
   420e5:	shl    $0x8,%esi
   420e8:	or     %r14d,%esi
   420eb:	mov    %esi,0x6c(%rcx,%rax,1)
   420ef:	movzwl 0x38(%rdi),%esi
   420f3:	add    %ebp,%esi
   420f5:	shl    $0x8,%esi
   420f8:	or     %r14d,%esi
   420fb:	mov    %esi,0x70(%rcx,%rax,1)
   420ff:	movzwl 0x3a(%rdi),%esi
   42103:	add    %ebp,%esi
   42105:	shl    $0x8,%esi
   42108:	or     %r14d,%esi
   4210b:	mov    %esi,0x74(%rcx,%rax,1)
   4210f:	lea    (%r10,%r15,1),%eax
   42113:	add    $0x20,%eax
   42116:	mov    %eax,0x28f0(%rsp)
   4211d:	add    $0x20,%r15
   42121:	inc    %r9
   42124:	add    $0x2000,%r13d
   4212b:	dec    %rdx
   4212e:	je     4233c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x120c>
   42134:	mov    %r9,%rax
   42137:	shr    $0x6,%rax
   4213b:	cmp    0x18(%rsp),%rax
   42140:	jae    428af <<c30walk::ctor2s::Ctor2s>::build::<false>+0x177f>
   42146:	mov    0x30(%rsp),%rcx
   4214b:	mov    (%rcx,%rax,8),%rax
   4214f:	bt     %r9,%rax
   42153:	jae    42180 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1050>
   42155:	cmp    $0x1e,%r12
   42159:	jae    428d5 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17a5>
   4215f:	mov    (%rbx,%r15,2),%eax
   42163:	and    $0xfffff,%eax
   42168:	mov    0x10(%r11),%rcx
   4216c:	cmp    %rax,%rcx
   4216f:	ja     41f30 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xe00>
   42175:	jmp    428c3 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1793>
   4217a:	nopw   0x0(%rax,%rax,1)
   42180:	cmp    $0x1e,%r12
   42184:	jae    42915 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17e5>
   4218a:	movups 0x2c(%rdi),%xmm0
   4218e:	movups %xmm0,-0x10(%rbx,%r15,2)
   42194:	movdqa (%rdi),%xmm0
   42198:	movdqa 0x10(%rdi),%xmm1
   4219d:	movdqa 0x20(%rdi),%xmm2
   421a2:	movdqa %xmm2,-0x1c(%rbx,%r15,2)
   421a9:	movdqa %xmm1,-0x2c(%rbx,%r15,2)
   421b0:	movdqa %xmm0,-0x3c(%rbx,%r15,2)
   421b7:	mov    %r13d,(%rbx,%r15,2)
   421bb:	jmp    4210f <<c30walk::ctor2s::Ctor2s>::build::<false>+0xfdf>
   421c0:	mov    0x28e8(%rsp),%rdi
   421c8:	cmp    $0x1d,%rdi
   421cc:	ja     429aa <<c30walk::ctor2s::Ctor2s>::build::<false>+0x187a>
   421d2:	shl    $0x6,%rdi
   421d6:	add    %r13,%rdi
   421d9:	mov    0x28f0(%rsp),%eax
   421e0:	mov    0xd0(%rsp),%rcx
   421e8:	shl    $0x6,%r9
   421ec:	mov    %eax,%r15d
   421ef:	shl    $0x8,%r15d
   421f3:	or     %r14d,%r15d
   421f6:	lea    -0x1e(%r12),%rsi
   421fb:	cmp    $0x1e,%rsi
   421ff:	jae    4226d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x113d>
   42201:	xor    %esi,%esi
   42203:	jmp    422fa <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11ca>
   42208:	shr    $1,%r9
   4220b:	inc    %r9
   4220e:	mov    %r9,%r10
   42211:	and    $0xfffffffffffffff8,%r10
   42215:	lea    (%rdx,%r10,2),%r8
   42219:	movd   %ecx,%xmm0
   4221d:	pshuflw $0x0,%xmm0,%xmm0
   42222:	add    %r13,%r11
   42225:	add    $0x8,%r11
   42229:	xor    %ebx,%ebx
   4222b:	nopl   0x0(%rax,%rax,1)
   42230:	movq   -0x8(%r11,%rbx,2),%xmm1
   42237:	movq   (%r11,%rbx,2),%xmm2
   4223d:	paddw  %xmm0,%xmm1
   42241:	paddw  %xmm0,%xmm2
   42245:	punpcklqdq %xmm2,%xmm1
   42249:	movdqu %xmm1,(%rax,%rbx,2)
   4224e:	add    $0x8,%rbx
   42252:	cmp    %rbx,%r10
   42255:	jne    42230 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1100>
   42257:	cmp    %r10,%r9
   4225a:	mov    0x80(%rsp),%rbx
   42262:	jne    41dc9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc99>
   42268:	jmp    41df9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xcc9>
   4226d:	mov    %rdx,%r10
   42270:	movabs $0xffffffffffffffe,%rsi
   4227a:	and    %rsi,%r10
   4227d:	lea    (%rcx,%r9,1),%r11
   42281:	add    $0x7c,%r11
   42285:	xor    %esi,%esi
   42287:	nopw   0x0(%rax,%rax,1)
   42290:	movdqu 0x2c(%rdi),%xmm0
   42295:	movdqu %xmm0,-0x50(%r11)
   4229b:	movdqa (%rdi),%xmm1
   4229f:	movdqa 0x10(%rdi),%xmm2
   422a4:	movdqa 0x20(%rdi),%xmm3
   422a9:	movdqa %xmm3,-0x5c(%r11)
   422af:	movdqa %xmm2,-0x6c(%r11)
   422b5:	movdqa %xmm1,-0x7c(%r11)
   422bb:	mov    %r15d,-0x40(%r11)
   422bf:	add    $0x2,%rsi
   422c3:	movdqa %xmm1,-0x3c(%r11)
   422c9:	movdqa %xmm2,-0x2c(%r11)
   422cf:	movdqa %xmm3,-0x1c(%r11)
   422d5:	movdqu %xmm0,-0x10(%r11)
   422db:	lea    0x2000(%r15),%ebx
   422e2:	mov    %ebx,(%r11)
   422e5:	add    $0x4000,%r15d
   422ec:	sub    $0xffffffffffffff80,%r11
   422f0:	cmp    %rsi,%r10
   422f3:	jne    42290 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1160>
   422f5:	test   $0x1,%dl
   422f8:	je     4232e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x11fe>
   422fa:	add    %r9,%rcx
   422fd:	shl    $0x6,%rsi
   42301:	movups 0x2c(%rdi),%xmm0
   42305:	movups %xmm0,0x2c(%rcx,%rsi,1)
   4230a:	movdqa (%rdi),%xmm0
   4230e:	movdqa 0x10(%rdi),%xmm1
   42313:	movdqa 0x20(%rdi),%xmm2
   42318:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   4231e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   42324:	movdqa %xmm0,(%rcx,%rsi,1)
   42329:	mov    %r15d,0x3c(%rcx,%rsi,1)
   4232e:	mov    %edx,%ecx
   42330:	shl    $0x5,%ecx
   42333:	add    %ecx,%eax
   42335:	mov    %eax,0x28f0(%rsp)
   4233c:	mov    0x68(%rsp),%rdx
   42341:	mov    %rdx,%rax
   42344:	shl    $0x5,%rax
   42348:	lea    (%rdx,%rdx,1),%rcx
   4234c:	sub    %rax,%rcx
   4234f:	mov    0x58(%rsp),%r12
   42354:	add    %rcx,%r12
   42357:	lea    (%rdx,%rdx,1),%eax
   4235a:	shl    $0x5,%edx
   4235d:	sub    %eax,%edx
   4235f:	add    %edx,%r8d
   42362:	mov    0x80(%rsp),%rbx
   4236a:	mov    %r8d,0x18(%rbx)
   4236e:	test   %r12,%r12
   42371:	mov    $0x88888889,%r15d
   42377:	lea    -0x26efe(%rip),%r13        # 1b480 <c30walk::ctor2s::PATTERNS>
   4237e:	je     423f0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12c0>
   42380:	mov    0x1c(%rbx),%ecx
   42383:	mov    %r8d,%edi
   42386:	imul   %r15,%rdi
   4238a:	mov    %rdi,%rdx
   4238d:	shr    $0x24,%rdx
   42391:	mov    %edx,%eax
   42393:	shl    $0x5,%eax
   42396:	sub    %edx,%eax
   42398:	sub    %edx,%eax
   4239a:	lea    0x1e(%rax),%esi
   4239d:	cmp    %ecx,%esi
   4239f:	cmovb  %esi,%ecx
   423a2:	shr    $0x2a,%rdi
   423a6:	mov    0xb8(%rsp),%rsi
   423ae:	cmp    %rsi,%rdi
   423b1:	jae    4298d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x185d>
   423b7:	mov    %rdx,%rsi
   423ba:	shl    $0x6,%rsi
   423be:	add    0xd0(%rsp),%rsi
   423c6:	mov    0xb0(%rsp),%r9
   423ce:	mov    (%r9,%rdi,8),%rdi
   423d2:	bt     %rdx,%rdi
   423d6:	jb     41d30 <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc00>
   423dc:	mov    $0xffffffff,%edi
   423e1:	jmp    41d3c <<c30walk::ctor2s::Ctor2s>::build::<false>+0xc0c>
   423e6:	cs nopw 0x0(%rax,%rax,1)
   423f0:	mov    0x8(%rsp),%rdi
   423f5:	mov    0x50(%rsp),%r8
   423fa:	mov    0x38(%rsp),%r14
   423ff:	mov    0x28(%rsp),%r13
   42404:	mov    0x40(%rsp),%rcx
   42409:	mov    0x64(%rsp),%eax
   4240d:	mov    %r8,%r12
   42410:	cmp    $0xf,%r8
   42414:	ja     41a58 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x928>
   4241a:	jmp    42423 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x12f3>
   4241c:	nopl   0x0(%rax)
   42420:	mov    %r12,%rcx
   42423:	test   %r8,%r8
   42426:	je     4190c <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7dc>
   4242c:	mov    0x20(%rsp),%r15
   42431:	sub    %r8,%r15
   42434:	mov    0x28e8(%rsp),%rax
   4243c:	mov    $0x1e,%ebx
   42441:	sub    %rax,%rbx
   42444:	cmp    %r8,%rbx
   42447:	cmovae %r8,%rbx
   4244b:	mov    %rbx,%r12
   4244e:	add    %r15,%r12
   42451:	lea    0xe8(%rsp),%rdx
   42459:	jb     428e9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   4245f:	cmp    %rdi,%r12
   42462:	ja     428e9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17b9>
   42468:	mov    %rcx,0x40(%rsp)
   4246d:	mov    %r14,0x38(%rsp)
   42472:	mov    0x28f0(%rsp),%r14d
   4247a:	cmp    $0x1e,%rax
   4247e:	jne    4250f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13df>
   42484:	cmp    %r12,0x20(%rsp)
   42489:	jb     428fe <<c30walk::ctor2s::Ctor2s>::build::<false>+0x17ce>
   4248f:	jne    42583 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1453>
   42495:	mov    0x28e8(%rsp),%rax
   4249d:	mov    0x28f0(%rsp),%ecx
   424a4:	cmp    $0x1,%r8
   424a8:	je     418d8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7a8>
   424ae:	mov    %r8d,%edx
   424b1:	and    $0xe,%edx
   424b4:	mov    0x38(%rsp),%r14
   424b9:	nopl   0x0(%rax)
   424c0:	lea    0x1(%rax),%rsi
   424c4:	add    $0xffffffffffffffe3,%rax
   424c8:	xor    %edi,%edi
   424ca:	cmp    $0x1e,%rsi
   424ce:	setae  %dil
   424d2:	cmovb  %rsi,%rax
   424d6:	lea    (%rcx,%rdi,2),%ecx
   424d9:	lea    0x1(%rax),%rsi
   424dd:	add    $0xffffffffffffffe3,%rax
   424e1:	xor    %edi,%edi
   424e3:	cmp    $0x1e,%rsi
   424e7:	setae  %dil
   424eb:	cmovb  %rsi,%rax
   424ef:	lea    (%rcx,%rdi,2),%ecx
   424f2:	add    $0x2,%ecx
   424f5:	add    $0xfffffffffffffffe,%rdx
   424f9:	jne    424c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1390>
   424fb:	test   $0x1,%r8b
   424ff:	mov    0x8(%rsp),%rdi
   42504:	jne    418dd <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7ad>
   4250a:	jmp    418f8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x7c8>
   4250f:	add    0x48(%rsp),%r15
   42514:	xor    %r13d,%r13d
   42517:	nopw   0x0(%rax,%rax,1)
   42520:	movzbl (%r15,%r13,1),%esi
   42525:	lea    (%rsi,%rsi,4),%rcx
   42529:	lea    (%rdx,%rcx,8),%rbp
   4252d:	mov    (%rdx,%rcx,8),%rax
   42531:	cmp    0x8(%rdx,%rcx,8),%rax
   42536:	je     42551 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1421>
   42538:	lea    (%r14,%r13,1),%rcx
   4253c:	mov    %cx,(%rax)
   4253f:	addq   $0x2,0x0(%rbp)
   42544:	inc    %r13
   42547:	cmp    %r13,%rbx
   4254a:	jne    42520 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x13f0>
   4254c:	jmp    42484 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1354>
   42551:	mov    %r8,0x50(%rsp)
   42556:	lea    0xa8(%rsp),%rdi
   4255e:	call   8c100 <<c30walk::ctor2s::W<false>>::refill>
   42563:	test   %al,%al
   42565:	je     4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   4256b:	mov    0x0(%rbp),%rax
   4256f:	mov    0x8(%rsp),%rdi
   42574:	lea    0xe8(%rsp),%rdx
   4257c:	mov    0x50(%rsp),%r8
   42581:	jmp    42538 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1408>
   42583:	mov    0x48(%rsp),%rax
   42588:	mov    0x20(%rsp),%rcx
   4258d:	lea    (%rax,%rcx,1),%r15
   42591:	lea    (%r14,%rbx,1),%ebp
   42595:	add    $0x2,%ebp
   42598:	sub    %r8,%rbx
   4259b:	nopl   0x0(%rax,%rax,1)
   425a0:	movzbl (%r15,%rbx,1),%esi
   425a5:	lea    (%rsi,%rsi,4),%rcx
   425a9:	lea    (%rdx,%rcx,8),%r14
   425ad:	mov    (%rdx,%rcx,8),%rax
   425b1:	cmp    0x8(%rdx,%rcx,8),%rax
   425b6:	je     425cb <<c30walk::ctor2s::Ctor2s>::build::<false>+0x149b>
   425b8:	mov    %bp,(%rax)
   425bb:	addq   $0x2,(%r14)
   425bf:	inc    %ebp
   425c1:	inc    %rbx
   425c4:	jne    425a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1470>
   425c6:	jmp    42495 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1365>
   425cb:	mov    %r8,%r12
   425ce:	lea    0xa8(%rsp),%rdi
   425d6:	call   8c100 <<c30walk::ctor2s::W<false>>::refill>
   425db:	test   %al,%al
   425dd:	je     4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   425df:	mov    (%r14),%rax
   425e2:	mov    0x8(%rsp),%rdi
   425e7:	lea    0xe8(%rsp),%rdx
   425ef:	mov    %r12,%r8
   425f2:	jmp    425b8 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1488>
   425f4:	mov    %r15,%rdi
   425f7:	call   e6d20 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   425fc:	jmp    4199f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x86f>
   42601:	mov    $0x84,%eax
   42606:	cs nopw 0x0(%rax,%rax,1)
   42610:	lea    0x50(%rax),%rcx
   42614:	cmp    $0x28d4,%rcx
   4261b:	je     42694 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1564>
   4261d:	mov    0x64(%rsp,%rax,1),%rdx
   42622:	sub    0x74(%rsp,%rax,1),%rdx
   42627:	shr    $1,%rdx
   4262a:	add    0x7c(%rsp,%rax,1),%edx
   4262e:	cmp    0x80(%rsp,%rax,1),%edx
   42635:	jne    4265d <<c30walk::ctor2s::Ctor2s>::build::<false>+0x152d>
   42637:	mov    0x8c(%rsp,%rax,1),%rdx
   4263f:	sub    0x9c(%rsp,%rax,1),%rdx
   42647:	shr    $1,%rdx
   4264a:	add    0xa4(%rsp,%rax,1),%edx
   42651:	cmp    0xa8(%rsp,%rax,1),%edx
   42658:	mov    %rcx,%rax
   4265b:	je     42610 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x14e0>
   4265d:	mov    0x5530(%rsp),%rax
   42665:	movq   $0x0,0x10(%rax)
   4266d:	xor    %eax,%eax
   4266f:	add    $0x54f8,%rsp
   42676:	pop    %rbx
   42677:	pop    %r12
   42679:	pop    %r13
   4267b:	pop    %r14
   4267d:	pop    %r15
   4267f:	pop    %rbp
   42680:	ret
   42681:	lea    0xee4e8(%rip),%rcx        # 130b70 <__frame_dummy_init_array_entry+0x2060>
   42688:	mov    $0x1e,%edx
   4268d:	xor    %edi,%edi
   4268f:	call   d8f00 <core::slice::index::slice_index_fail>
   42694:	xor    %r14d,%r14d
   42697:	lea    0xa8(%rsp),%r15
   4269f:	mov    0x88(%rsp),%r12
   426a7:	mov    0x8(%rsp),%rdx
   426ac:	mov    0x78(%rsp),%r13
   426b1:	lea    0xe8(%rsp),%rbx
   426b9:	jmp    426e9 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15b9>
   426bb:	nopl   0x0(%rax,%rax,1)
   426c0:	sub    %rcx,%rax
   426c3:	inc    %r14
   426c6:	shr    $1,%rax
   426c9:	add    %eax,0x18(%rbx)
   426cc:	pxor   %xmm0,%xmm0
   426d0:	movdqu %xmm0,(%rbx)
   426d4:	movq   $0x0,0x10(%rbx)
   426dc:	add    $0x28,%rbx
   426e0:	cmp    $0x100,%r14
   426e7:	je     4270e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x15de>
   426e9:	mov    (%rbx),%rcx
   426ec:	mov    %rcx,%rax
   426ef:	cmp    0x10(%rbx),%rcx
   426f3:	je     426c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   426f5:	mov    %r15,%rdi
   426f8:	mov    %r14,%rsi
   426fb:	call   8bb00 <<c30walk::ctor2s::W<false>>::finish>
   42700:	mov    0x8(%rsp),%rdx
   42705:	mov    (%rbx),%rax
   42708:	mov    0x10(%rbx),%rcx
   4270c:	jmp    426c0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x1590>
   4270e:	xor    %ecx,%ecx
   42710:	mov    0x8(%r12),%rbx
   42715:	test   %r13,%r13
   42718:	setne  %al
   4271b:	je     4275f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   4271d:	mov    %al,%cl
   4271f:	mov    %r13,%rax
   42722:	sub    %rcx,%rax
   42725:	imul   $0xffffffffffffffe2,%rax,%rcx
   42729:	add    %rdx,%rcx
   4272c:	cmp    $0x1e,%rcx
   42730:	jae    4275f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x162f>
   42732:	lea    (%rbx,%r13,4),%rcx
   42736:	lea    (%rcx,%rdx,2),%rdi
   4273a:	add    $0xfffffffffffffffc,%rdi
   4273e:	lea    (%rax,%rax,1),%rcx
   42742:	shl    $0x5,%rax
   42746:	sub    %rcx,%rax
   42749:	sub    %rdx,%rax
   4274c:	lea    0x3c(,%rax,2),%rdx
   42754:	mov    $0xff,%esi
   42759:	call   *0xf2731(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   4275f:	shl    $0x6,%r13
   42763:	pcmpeqd %xmm0,%xmm0
   42767:	movdqu %xmm0,0x2c(%rbx,%r13,1)
   4276e:	movdqa %xmm0,0x20(%rbx,%r13,1)
   42775:	movdqa %xmm0,0x10(%rbx,%r13,1)
   4277c:	movdqa %xmm0,(%rbx,%r13,1)
   42782:	movl   $0x100fffff,0x3c(%rbx,%r13,1)
   4278b:	mov    0x70(%rsp),%rax
   42790:	mov    0x8(%rax),%rdx
   42794:	mov    0x10(%rax),%rcx
   42798:	mov    0x5530(%rsp),%rax
   427a0:	mov    0x8(%rax),%r8
   427a4:	mov    0x10(%rax),%r9
   427a8:	lea    0xa8(%rsp),%rdi
   427b0:	mov    %rbx,%rsi
   427b3:	call   9a360 <c30walk::ctor::canonicalise>
   427b8:	mov    0x90(%rsp),%rax
   427c0:	mov    %rax,0x10(%r12)
   427c5:	mov    $0x1,%al
   427c7:	jmp    4266f <<c30walk::ctor2s::Ctor2s>::build::<false>+0x153f>
   427cc:	mov    %rdi,%r12
   427cf:	test   %rax,%rax
   427d2:	je     427e4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x16b4>
   427d4:	mov    0x8(%r9),%rdi
   427d8:	mov    %r9,%r14
   427db:	call   *0xf26bf(%rip)        # 134ea0 <free@GLIBC_2.2.5>
   427e1:	mov    %r14,%r9
   427e4:	movq   $0x0,(%r9)
   427eb:	movq   $0x40,0x8(%r9)
   427f3:	movq   $0x0,0x10(%r9)
   427fb:	mov    $0x40,%edx
   42800:	mov    %r9,%rdi
   42803:	lea    0x1(%r13),%rsi
   42807:	mov    %r9,%r14
   4280a:	call   90660 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4280f:	mov    %r14,%r9
   42812:	cmp    $0xffffffffffffffff,%rax
   42816:	mov    %r12,%rdi
   42819:	je     41353 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x223>
   4281f:	jmp    4287a <<c30walk::ctor2s::Ctor2s>::build::<false>+0x174a>
   42821:	mov    %rdi,%r12
   42824:	mov    %r9,%r14
   42827:	test   %rax,%rax
   4282a:	je     4283e <<c30walk::ctor2s::Ctor2s>::build::<false>+0x170e>
   4282c:	mov    0x5530(%rsp),%rax
   42834:	mov    0x8(%rax),%rdi
   42838:	call   *0xf2662(%rip)        # 134ea0 <free@GLIBC_2.2.5>
   4283e:	mov    0x5530(%rsp),%rdi
   42846:	movq   $0x0,(%rdi)
   4284d:	movq   $0x40,0x8(%rdi)
   42855:	movq   $0x0,0x10(%rdi)
   4285d:	mov    $0x80,%edx
   42862:	mov    %rbx,%rsi
   42865:	call   90660 <<alloc::raw_vec::RawVecInner>::grow_exact>
   4286a:	cmp    $0xffffffffffffffff,%rax
   4286e:	mov    %r14,%r9
   42871:	mov    %r12,%rdi
   42874:	je     41367 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x237>
   4287a:	mov    %rax,%rdi
   4287d:	mov    %rdx,%rsi
   42880:	call   d668b <alloc::raw_vec::handle_error>
   42885:	lea    0xee314(%rip),%rcx        # 130ba0 <__frame_dummy_init_array_entry+0x2090>
   4288c:	mov    %rdi,%rdx
   4288f:	mov    %r14,%rdi
   42892:	mov    %r15,%rsi
   42895:	call   d8f00 <core::slice::index::slice_index_fail>
   4289a:	lea    0xee2e7(%rip),%rcx        # 130b88 <__frame_dummy_init_array_entry+0x2078>
   428a1:	mov    %rdi,%rdx
   428a4:	mov    %r15,%rdi
   428a7:	mov    %r14,%rsi
   428aa:	call   d8f00 <core::slice::index::slice_index_fail>
   428af:	lea    0xec4ca(%rip),%rdx        # 12ed80 <__frame_dummy_init_array_entry+0x270>
   428b6:	mov    %rax,%rdi
   428b9:	mov    0x18(%rsp),%rsi
   428be:	call   d8123 <core::panicking::panic_bounds_check>
   428c3:	lea    0xee366(%rip),%rdx        # 130c30 <__frame_dummy_init_array_entry+0x2120>
   428ca:	mov    %rax,%rdi
   428cd:	mov    %rcx,%rsi
   428d0:	call   d8123 <core::panicking::panic_bounds_check>
   428d5:	lea    0xee33c(%rip),%rdx        # 130c18 <__frame_dummy_init_array_entry+0x2108>
   428dc:	mov    $0x1e,%esi
   428e1:	mov    %r12,%rdi
   428e4:	call   d8123 <core::panicking::panic_bounds_check>
   428e9:	lea    0xee2b0(%rip),%rcx        # 130ba0 <__frame_dummy_init_array_entry+0x2090>
   428f0:	mov    %rdi,%rdx
   428f3:	mov    %r15,%rdi
   428f6:	mov    %r12,%rsi
   428f9:	call   d8f00 <core::slice::index::slice_index_fail>
   428fe:	lea    0xee283(%rip),%rcx        # 130b88 <__frame_dummy_init_array_entry+0x2078>
   42905:	mov    %rdi,%rdx
   42908:	mov    %r12,%rdi
   4290b:	mov    0x20(%rsp),%rsi
   42910:	call   d8f00 <core::slice::index::slice_index_fail>
   42915:	lea    0xee2e4(%rip),%rdx        # 130c00 <__frame_dummy_init_array_entry+0x20f0>
   4291c:	mov    $0x1e,%esi
   42921:	mov    %r12,%rdi
   42924:	call   d8123 <core::panicking::panic_bounds_check>
   42929:	mov    $0x10,%edx
   4292e:	mov    $0x4,%ecx
   42933:	mov    $0x400,%r8d
   42939:	mov    %rdi,%r14
   4293c:	mov    %r13,%rdi
   4293f:	xor    %esi,%esi
   42941:	mov    %r9,%rbx
   42944:	call   677e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   42949:	mov    %r14,%rdi
   4294c:	mov    %rbx,%r9
   4294f:	jmp    413f4 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x2c4>
   42954:	mov    %r9,%r13
   42957:	mov    $0x8,%ecx
   4295c:	mov    $0x8,%r8d
   42962:	mov    %rdi,%r14
   42965:	xor    %esi,%esi
   42967:	mov    %rbx,%rdx
   4296a:	call   677e0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   4296f:	mov    %r14,%rdi
   42972:	mov    0x10(%r14),%r14
   42976:	mov    0x8(%rdi),%r12
   4297a:	lea    (%r12,%r14,8),%rdi
   4297e:	cmp    $0x1,%rbx
   42982:	jne    413a0 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x270>
   42988:	jmp    413c2 <<c30walk::ctor2s::Ctor2s>::build::<false>+0x292>
   4298d:	lea    0xec3ec(%rip),%rdx        # 12ed80 <__frame_dummy_init_array_entry+0x270>
   42994:	call   d8123 <core::panicking::panic_bounds_check>
   42999:	lea    0xee1b8(%rip),%rdx        # 130b58 <__frame_dummy_init_array_entry+0x2048>
   429a0:	mov    $0x1e,%esi
   429a5:	call   d8123 <core::panicking::panic_bounds_check>
   429aa:	lea    0xee237(%rip),%rdx        # 130be8 <__frame_dummy_init_array_entry+0x20d8>
   429b1:	mov    $0x1e,%esi
   429b6:	call   d8123 <core::panicking::panic_bounds_check>
   429bb:	lea    0xee11e(%rip),%rdx        # 130ae0 <__frame_dummy_init_array_entry+0x1fd0>
   429c2:	call   d8123 <core::panicking::panic_bounds_check>
   429c7:	lea    0xec3b2(%rip),%rdx        # 12ed80 <__frame_dummy_init_array_entry+0x270>
   429ce:	mov    %r14,%rdi
   429d1:	call   d8123 <core::panicking::panic_bounds_check>
