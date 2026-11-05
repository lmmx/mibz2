0000000000050210 <mibz2::mtf::mtf_avx2>:
   50210:	push   %rbp
   50211:	mov    %rsp,%rbp
   50214:	push   %r15
   50216:	push   %r14
   50218:	push   %r13
   5021a:	push   %r12
   5021c:	push   %rbx
   5021d:	and    $0xffffffffffffffc0,%rsp
   50221:	sub    $0x1c0,%rsp
   50228:	mov    %rcx,%r15
   5022b:	mov    %rdx,%rbx
   5022e:	mov    %rsi,%r13
   50231:	mov    %rdi,%r14
   50234:	mov    0x118(%rsi),%rax
   5023b:	movq   $0x0,0x10(%rdx)
   50243:	mov    %rax,0x20(%rsp)
   50248:	cmp    (%rdx),%rax
   5024b:	ja     5057e <mibz2::mtf::mtf_avx2+0x36e>
   50251:	xor    %r12d,%r12d
   50254:	mov    $0x400,%edx
   50259:	mov    %r15,%rdi
   5025c:	xor    %esi,%esi
   5025e:	call   *0x455fc(%rip)        # 95860 <memset@GLIBC_2.2.5>
   50264:	vxorps %xmm0,%xmm0,%xmm0
   50268:	vmovaps %ymm0,0x160(%rsp)
   50271:	vmovaps %ymm0,0x140(%rsp)
   5027a:	vmovaps %ymm0,0x120(%rsp)
   50283:	vmovaps %ymm0,0x100(%rsp)
   5028c:	vmovaps %ymm0,0xe0(%rsp)
   50295:	vmovaps %ymm0,0xc0(%rsp)
   5029e:	vmovaps %ymm0,0xa0(%rsp)
   502a7:	vmovaps %ymm0,0x80(%rsp)
   502b0:	vmovaps %ymm0,0x60(%rsp)
   502b6:	vmovaps %ymm0,0x40(%rsp)
   502bc:	vmovups 0xf8(%r13),%ymm0
   502c5:	vmovups %ymm0,0x140(%rsp)
   502ce:	vmovups 0xd8(%r13),%ymm0
   502d7:	vmovaps %ymm0,0x120(%rsp)
   502e0:	vmovups 0xb8(%r13),%ymm0
   502e9:	vmovups %ymm0,0x100(%rsp)
   502f2:	vmovups 0x98(%r13),%ymm0
   502fb:	vmovaps %ymm0,0xe0(%rsp)
   50304:	vmovdqu 0x18(%r13),%ymm0
   5030a:	vmovdqu 0x38(%r13),%ymm1
   50310:	vmovdqu 0x58(%r13),%ymm2
   50316:	vmovups 0x78(%r13),%ymm3
   5031c:	vmovups %ymm3,0xc0(%rsp)
   50325:	vmovdqa %ymm2,0xa0(%rsp)
   5032e:	vmovdqu %ymm1,0x80(%rsp)
   50337:	vmovdqa %ymm0,0x60(%rsp)
   5033d:	mov    0x10(%r13),%rax
   50341:	test   %rax,%rax
   50344:	mov    %rbx,0x28(%rsp)
   50349:	je     50543 <mibz2::mtf::mtf_avx2+0x333>
   5034f:	mov    %r15,0x30(%rsp)
   50354:	lea    0x60(%rsp),%r8
   50359:	mov    0x8(%rbx),%r9
   5035d:	mov    0x8(%r13),%rbx
   50361:	lea    (%rbx,%rax,2),%r15
   50365:	movzwl 0x128(%r13),%r10d
   5036d:	mov    $0x1,%r13d
   50373:	lea    0x61(%rsp),%r11
   50378:	xor    %edi,%edi
   5037a:	vmovdqa -0x47f82(%rip),%ymm2        # 8400 <GCC_except_table701+0x920>
   50382:	mov    %r14,0x18(%rsp)
   50387:	xor    %eax,%eax
   50389:	nopl   0x0(%rax)
   50390:	movzwl (%rbx),%r14d
   50394:	add    $0x2,%rbx
   50398:	cmp    $0x2,%r14d
   5039c:	jae    503d0 <mibz2::mtf::mtf_avx2+0x1c0>
   5039e:	cmp    $0x1fffff,%r13d
   503a5:	ja     50521 <mibz2::mtf::mtf_avx2+0x311>
   503ab:	inc    %r14d
   503ae:	imul   %r13d,%r14d
   503b2:	add    %r14d,%eax
   503b5:	add    %r13d,%r13d
   503b8:	cmp    %r15,%rbx
   503bb:	jne    50390 <mibz2::mtf::mtf_avx2+0x180>
   503bd:	jmp    5053b <mibz2::mtf::mtf_avx2+0x32b>
   503c2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   503d0:	test   %eax,%eax
   503d2:	je     50429 <mibz2::mtf::mtf_avx2+0x219>
   503d4:	mov    %eax,%edx
   503d6:	lea    (%rdi,%rdx,1),%r12
   503da:	cmp    0x20(%rsp),%r12
   503df:	ja     5056f <mibz2::mtf::mtf_avx2+0x35f>
   503e5:	movzbl 0x60(%rsp),%ecx
   503ea:	mov    0x30(%rsp),%rsi
   503ef:	add    %eax,(%rsi,%rcx,4)
   503f2:	cmp    $0x40,%eax
   503f5:	ja     50437 <mibz2::mtf::mtf_avx2+0x227>
   503f7:	lea    0x40(%rdi),%rax
   503fb:	cmp    0x20(%rsp),%rax
   50400:	ja     50437 <mibz2::mtf::mtf_avx2+0x227>
   50402:	vmovd  %ecx,%xmm0
   50406:	vpbroadcastb %xmm0,%ymm0
   5040b:	vmovdqu %ymm0,(%r9,%rdi,1)
   50411:	vmovdqu %ymm0,0x20(%r9,%rdi,1)
   50418:	mov    $0x1,%r13d
   5041e:	cmp    %r10w,%r14w
   50422:	jne    5047a <mibz2::mtf::mtf_avx2+0x26a>
   50424:	jmp    5053e <mibz2::mtf::mtf_avx2+0x32e>
   50429:	mov    %rdi,%r12
   5042c:	cmp    %r10w,%r14w
   50430:	jne    5047a <mibz2::mtf::mtf_avx2+0x26a>
   50432:	jmp    5053e <mibz2::mtf::mtf_avx2+0x32e>
   50437:	add    %r9,%rdi
   5043a:	movzbl %cl,%esi
   5043d:	mov    %r9,%r13
   50440:	mov    %r10w,0x16(%rsp)
   50446:	vzeroupper
   50449:	call   *0x45411(%rip)        # 95860 <memset@GLIBC_2.2.5>
   5044f:	vmovdqa -0x48057(%rip),%ymm2        # 8400 <GCC_except_table701+0x920>
   50457:	lea    0x61(%rsp),%r11
   5045c:	movzwl 0x16(%rsp),%r10d
   50462:	mov    %r13,%r9
   50465:	lea    0x60(%rsp),%r8
   5046a:	mov    $0x1,%r13d
   50470:	cmp    %r10w,%r14w
   50474:	je     5053e <mibz2::mtf::mtf_avx2+0x32e>
   5047a:	cmp    0x20(%rsp),%r12
   5047f:	jae    5056c <mibz2::mtf::mtf_avx2+0x35c>
   50485:	mov    %r14d,%eax
   50488:	dec    %r14d
   5048b:	movzwl %r14w,%edx
   5048f:	movzbl 0x60(%rsp,%rdx,1),%r14d
   50495:	cmp    $0x21,%ax
   50499:	jae    504c0 <mibz2::mtf::mtf_avx2+0x2b0>
   5049b:	vmovdqa 0x60(%rsp),%ymm0
   504a1:	inc    %dl
   504a3:	vmovd  %edx,%xmm1
   504a7:	vpbroadcastb %xmm1,%ymm1
   504ac:	vpcmpgtb %ymm2,%ymm1,%ymm1
   504b0:	vpblendvb %ymm1,0x5f(%rsp),%ymm0,%ymm0
   504b8:	vmovdqa %ymm0,0x60(%rsp)
   504be:	jmp    504f7 <mibz2::mtf::mtf_avx2+0x2e7>
   504c0:	mov    %r11,%rdi
   504c3:	mov    %r8,%rsi
   504c6:	mov    %r9,0x38(%rsp)
   504cb:	mov    %r10w,0x16(%rsp)
   504d1:	vzeroupper
   504d4:	call   *0x45346(%rip)        # 95820 <memmove@GLIBC_2.2.5>
   504da:	vmovdqa -0x480e2(%rip),%ymm2        # 8400 <GCC_except_table701+0x920>
   504e2:	lea    0x61(%rsp),%r11
   504e7:	movzwl 0x16(%rsp),%r10d
   504ed:	mov    0x38(%rsp),%r9
   504f2:	lea    0x60(%rsp),%r8
   504f7:	movzbl %r14b,%eax
   504fb:	mov    %al,0x60(%rsp)
   504ff:	mov    0x30(%rsp),%rcx
   50504:	incl   (%rcx,%rax,4)
   50507:	mov    %al,(%r9,%r12,1)
   5050b:	inc    %r12
   5050e:	mov    %r12,%rdi
   50511:	cmp    %r15,%rbx
   50514:	mov    0x18(%rsp),%r14
   50519:	jne    50387 <mibz2::mtf::mtf_avx2+0x177>
   5051f:	jmp    50543 <mibz2::mtf::mtf_avx2+0x333>
   50521:	movabs $0x800000000000000c,%rax
   5052b:	mov    0x28(%rsp),%rcx
   50530:	mov    %rdi,0x10(%rcx)
   50534:	mov    0x18(%rsp),%r14
   50539:	jmp    50557 <mibz2::mtf::mtf_avx2+0x347>
   5053b:	mov    %rdi,%r12
   5053e:	mov    0x18(%rsp),%r14
   50543:	mov    0x28(%rsp),%rax
   50548:	mov    %r12,0x10(%rax)
   5054c:	mov    %r12,0x8(%r14)
   50550:	mov    $0xffffffffffffffff,%rax
   50557:	mov    %rax,(%r14)
   5055a:	lea    -0x28(%rbp),%rsp
   5055e:	pop    %rbx
   5055f:	pop    %r12
   50561:	pop    %r13
   50563:	pop    %r14
   50565:	pop    %r15
   50567:	pop    %rbp
   50568:	vzeroupper
   5056b:	ret
   5056c:	mov    %r12,%rdi
   5056f:	movabs $0x800000000000000c,%rax
   50579:	inc    %rax
   5057c:	jmp    5052b <mibz2::mtf::mtf_avx2+0x31b>
   5057e:	mov    $0x1,%ecx
   50583:	mov    $0x1,%r8d
   50589:	mov    %rbx,%rdi
   5058c:	xor    %esi,%esi
   5058e:	mov    0x20(%rsp),%rdx
   50593:	call   358a0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   50598:	jmp    50251 <mibz2::mtf::mtf_avx2+0x41>
   5059d:	int3
   5059e:	int3
   5059f:	int3

