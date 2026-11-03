# Ctor2::build::<false>: the one-row append loops (a tile's two parts, before and after its label hole)
   34350:	movzbl (%r11,%r13,1),%esi
   34355:	lea    (%rsi,%rsi,4),%rcx
   34359:	lea    (%r8,%rcx,8),%rbp
   3435d:	mov    (%r8,%rcx,8),%rax
   34361:	cmp    0x8(%r8,%rcx,8),%rax
   34366:	je     3437e <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   34368:	lea    (%r14,%r13,1),%rcx
   3436c:	mov    %cx,(%rax)
   3436f:	addq   $0x2,0x0(%rbp)
   34374:	inc    %r13
   34377:	cmp    %r13,%rbx
   3437a:	jne    34350 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3437c:	jmp    343b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
   3437e:	mov    %r11,(%rsp)
   34382:	lea    0x98(%rsp),%rdi
   3438a:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   3438f:	lea    0xd8(%rsp),%r8
   34397:	test   %al,%al
   34399:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3439f:	mov    0x0(%rbp),%rax
   343a3:	mov    0x20(%rsp),%r10
   343a8:	mov    (%rsp),%r11
   343ac:	jmp    34368 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
23

   343e0:	movzbl (%r15),%esi
   343e4:	lea    (%rsi,%rsi,4),%rcx
   343e8:	lea    (%r8,%rcx,8),%r14
   343ec:	mov    (%r8,%rcx,8),%rax
   343f0:	cmp    0x8(%r8,%rcx,8),%rax
   343f5:	je     3440d <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
   343f7:	mov    %bx,(%rax)
   343fa:	addq   $0x2,(%r14)
   343fe:	inc    %r15
   34401:	inc    %ebx
   34403:	cmp    %r12,%r15
   34406:	jne    343e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   34408:	jmp    33a90 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
   3440d:	lea    0x98(%rsp),%rdi
   34415:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   3441a:	lea    0xd8(%rsp),%r8
   34422:	test   %al,%al
   34424:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
   3442a:	mov    (%r14),%rax
   3442d:	mov    0x8(%rsp),%r13
   34432:	mov    0x20(%rsp),%r10
   34437:	jmp    343f7 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
22

# the bulk emitter (v1's, copied): two complete records an iteration
   341d0:	movdqu 0x2c(%rdi),%xmm0
   341d5:	movdqu %xmm0,-0x50(%r11)
   341db:	movdqa (%rdi),%xmm1
   341df:	movdqa 0x10(%rdi),%xmm2
   341e4:	movdqa 0x20(%rdi),%xmm3
   341e9:	movdqa %xmm3,-0x5c(%r11)
   341ef:	movdqa %xmm2,-0x6c(%r11)
   341f5:	movdqa %xmm1,-0x7c(%r11)
   341fb:	mov    %r14d,-0x40(%r11)
   341ff:	add    $0x2,%rsi
   34203:	movdqa %xmm1,-0x3c(%r11)
   34209:	movdqa %xmm2,-0x2c(%r11)
   3420f:	movdqa %xmm3,-0x1c(%r11)
   34215:	movdqu %xmm0,-0x10(%r11)
   3421b:	lea    0x2000(%r14),%ebx
   34222:	mov    %ebx,(%r11)
   34225:	add    $0x4000,%r14d
   3422c:	sub    $0xffffffffffffff80,%r11
   34230:	cmp    %rsi,%r10
   34233:	jne    341d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
20

# W<false>::finish: the fast ordinary path, entry to the end of the full-group normalisation
   61d30:	push   %rbp
   61d31:	push   %r15
   61d33:	push   %r14
   61d35:	push   %r13
   61d37:	push   %r12
   61d39:	push   %rbx
   61d3a:	sub    $0x38,%rsp
   61d3e:	mov    0x18(%rdi),%rbp
   61d42:	mov    0x10(%rbp),%r8
   61d46:	mov    %r8,%r13
   61d49:	sub    $0x1,%r13
   61d4d:	jb     622ba <<c30walk::ctor2::W<false>>::finish+0x58a>
   61d53:	mov    %rdi,%r11
   61d56:	lea    (%rsi,%rsi,4),%rax
   61d5a:	mov    0x40(%rdi,%rax,8),%r14
   61d5f:	mov    0x50(%rdi,%rax,8),%rdi
   61d64:	mov    0x58(%r11,%rax,8),%r12d
   61d69:	mov    0x60(%r11,%rax,8),%ecx
   61d6e:	mov    0x64(%r11,%rax,8),%eax
   61d73:	sub    %rdi,%r14
   61d76:	mov    %r14,0x8(%rsp)
   61d7b:	shr    $1,%r14
   61d7e:	mov    0x8(%rbp),%r10
   61d82:	mov    %r13,%rdx
   61d85:	shl    $0xa,%rdx
   61d89:	add    %r10,%rdx
   61d8c:	mov    %r13,%rbx
   61d8f:	cmp    %r12d,(%rdx,%rsi,4)
   61d93:	jbe    61e08 <<c30walk::ctor2::W<false>>::finish+0xd8>
29
   61e08:	imul   $0xffffffffffffffe2,%rcx,%r15
   61e0c:	add    %r12,%r15
   61e0f:	cmp    $0xffffffff,%eax
   61e12:	mov    %r14,0x10(%rsp)
   61e17:	je     61f1b <<c30walk::ctor2::W<false>>::finish+0x1eb>
5
   61f1b:	movzwl (%rdi),%eax
   61f1e:	movzwl -0x2(%rdi,%r14,2),%r8d
   61f24:	mov    %ebx,%edx
   61f26:	shl    $0x10,%edx
   61f29:	or     %eax,%edx
   61f2b:	mov    %r13d,%r9d
   61f2e:	shl    $0x10,%r9d
   61f32:	or     %r8d,%r9d
   61f35:	sub    %edx,%r9d
   61f38:	cmp    $0xfffe,%r9d
   61f3f:	jae    61fbd <<c30walk::ctor2::W<false>>::finish+0x28d>
   61f41:	mov    0x28(%r11),%r8
   61f45:	shl    $0x6,%rcx
   61f49:	shl    $0x8,%edx
   61f4c:	or     %edx,%esi
   61f4e:	mov    %esi,0x3c(%r8,%rcx,1)
   61f53:	test   %r14,%r14
   61f56:	je     6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
   61f5c:	cmp    $0x1e,%r14
   61f60:	jne    62156 <<c30walk::ctor2::W<false>>::finish+0x426>
   61f66:	movd   %eax,%xmm0
   61f6a:	pshuflw $0x0,%xmm0,%xmm0
   61f6f:	pshufd $0x44,%xmm0,%xmm1
   61f74:	movdqu (%rdi),%xmm2
   61f78:	movdqu 0x10(%rdi),%xmm3
   61f7d:	movdqu 0x20(%rdi),%xmm4
   61f82:	psubw  %xmm1,%xmm2
   61f86:	movdqu %xmm2,(%rdi)
   61f8a:	psubw  %xmm1,%xmm3
   61f8e:	movdqu %xmm3,0x10(%rdi)
   61f93:	psubw  %xmm1,%xmm4
   61f97:	movdqu %xmm4,0x20(%rdi)
   61f9c:	movq   0x30(%rdi),%xmm1
   61fa1:	psubw  %xmm0,%xmm1
   61fa5:	movq   %xmm1,0x30(%rdi)
   61faa:	movd   0x38(%rdi),%xmm1
   61faf:	psubw  %xmm0,%xmm1
   61fb3:	movd   %xmm1,0x38(%rdi)
   61fb8:	jmp    6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
39
   6231b:	add    $0x38,%rsp
   6231f:	pop    %rbx
   62320:	pop    %r12
   62322:	pop    %r13
   62324:	pop    %r14
   62326:	pop    %r15
   62328:	pop    %rbp
   62329:	ret
8

# W<false>::refill (the append's slow edge): close, finish call, open
   62330:	push   %r15
   62332:	push   %r14
   62334:	push   %rbx
   62335:	mov    %rdi,%rbx
   62338:	lea    (%rsi,%rsi,4),%rax
   6233c:	lea    (%rdi,%rax,8),%r15
   62340:	add    $0x40,%r15
   62344:	mov    0x40(%rdi,%rax,8),%r14
   62349:	sub    0x50(%rdi,%rax,8),%r14
   6234e:	je     62358 <<c30walk::ctor2::W<false>>::refill+0x28>
   62350:	mov    %rbx,%rdi
   62353:	call   61d30 <<c30walk::ctor2::W<false>>::finish>
   62358:	shr    $1,%r14
   6235b:	add    0x18(%r15),%r14d
   6235f:	mov    %r14d,0x18(%r15)
   62363:	xorps  %xmm0,%xmm0
   62366:	movups %xmm0,(%r15)
   6236a:	movq   $0x0,0x10(%r15)
   62372:	mov    0x1c(%r15),%eax
   62376:	cmp    %eax,%r14d
   62379:	jae    62404 <<c30walk::ctor2::W<false>>::refill+0xd4>
   6237f:	mov    %r14d,%ecx
   62382:	mov    $0x88888889,%edi
   62387:	imul   %rcx,%rdi
   6238b:	mov    %rdi,%rcx
   6238e:	shr    $0x24,%rcx
   62392:	mov    %ecx,%r9d
   62395:	shl    $0x5,%r9d
   62399:	sub    %ecx,%r9d
   6239c:	sub    %ecx,%r9d
   6239f:	lea    0x1e(%r9),%edx
   623a3:	cmp    %eax,%edx
   623a5:	cmovae %eax,%edx
   623a8:	shr    $0x2a,%rdi
   623ac:	mov    0x10(%rbx),%rsi
   623b0:	cmp    %rsi,%rdi
   623b3:	jae    62410 <<c30walk::ctor2::W<false>>::refill+0xe0>
   623b5:	mov    %r14d,%esi
   623b8:	mov    %rcx,%r8
   623bb:	shl    $0x6,%r8
   623bf:	add    0x28(%rbx),%r8
   623c3:	sub    %r9d,%esi
   623c6:	mov    0x8(%rbx),%r9
   623ca:	mov    (%r9,%rdi,8),%rdi
   623ce:	bt     %rcx,%rdi
   623d2:	jae    623df <<c30walk::ctor2::W<false>>::refill+0xaf>
   623d4:	mov    $0xfffff,%edi
   623d9:	and    0x3c(%r8),%edi
   623dd:	jmp    623e4 <<c30walk::ctor2::W<false>>::refill+0xb4>
   623df:	mov    $0xffffffff,%edi
   623e4:	mov    %esi,%esi
   623e6:	lea    (%r8,%rsi,2),%rsi
   623ea:	mov    %rsi,0x10(%r15)
   623ee:	mov    %rsi,(%r15)
   623f1:	sub    %r14d,%edx
   623f4:	lea    (%rsi,%rdx,2),%rdx
   623f8:	mov    %rdx,0x8(%r15)
   623fc:	mov    %ecx,0x20(%r15)
   62400:	mov    %edi,0x24(%r15)
   62404:	cmp    %eax,%r14d
   62407:	setb   %al
   6240a:	pop    %rbx
   6240b:	pop    %r14
   6240d:	pop    %r15
   6240f:	ret
65
