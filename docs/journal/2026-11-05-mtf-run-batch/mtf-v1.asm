00000000000501b0 <mibz2::mtf::mtf_avx2>:
   501b0:	push   %rbp
   501b1:	mov    %rsp,%rbp
   501b4:	push   %r15
   501b6:	push   %r14
   501b8:	push   %r13
   501ba:	push   %r12
   501bc:	push   %rbx
   501bd:	and    $0xffffffffffffffc0,%rsp
   501c1:	sub    $0x240,%rsp
   501c8:	mov    %rdx,%r14
   501cb:	mov    %rsi,%rbx
   501ce:	mov    %rdi,0x58(%rsp)
   501d3:	mov    0x118(%rsi),%rax
   501da:	movq   $0x0,0x10(%rdx)
   501e2:	mov    %rax,0x10(%rsp)
   501e7:	cmp    (%rdx),%rax
   501ea:	mov    %rcx,0x18(%rsp)
   501ef:	ja     5082a <mibz2::mtf::mtf_avx2+0x67a>
   501f5:	xor    %r15d,%r15d
   501f8:	mov    $0x400,%edx
   501fd:	mov    %rcx,%rdi
   50200:	xor    %esi,%esi
   50202:	call   *0x45938(%rip)        # 95b40 <memset@GLIBC_2.2.5>
   50208:	vxorps %xmm0,%xmm0,%xmm0
   5020c:	vmovaps %ymm0,0x1a0(%rsp)
   50215:	vmovaps %ymm0,0x180(%rsp)
   5021e:	vmovaps %ymm0,0x160(%rsp)
   50227:	vmovaps %ymm0,0x140(%rsp)
   50230:	vmovaps %ymm0,0x120(%rsp)
   50239:	vmovaps %ymm0,0x100(%rsp)
   50242:	vmovaps %ymm0,0xe0(%rsp)
   5024b:	vmovaps %ymm0,0xc0(%rsp)
   50254:	vmovaps %ymm0,0xa0(%rsp)
   5025d:	vmovaps %ymm0,0x80(%rsp)
   50266:	vmovups 0xf8(%rbx),%ymm0
   5026e:	vmovups %ymm0,0x180(%rsp)
   50277:	vmovups 0xd8(%rbx),%ymm0
   5027f:	vmovaps %ymm0,0x160(%rsp)
   50288:	vmovups 0xb8(%rbx),%ymm0
   50290:	vmovups %ymm0,0x140(%rsp)
   50299:	vmovups 0x98(%rbx),%ymm0
   502a1:	vmovaps %ymm0,0x120(%rsp)
   502aa:	vmovdqu 0x18(%rbx),%ymm0
   502af:	vmovdqu 0x38(%rbx),%ymm1
   502b4:	vmovdqu 0x58(%rbx),%ymm2
   502b9:	vmovdqu 0x78(%rbx),%ymm3
   502be:	vmovdqu %ymm3,0x100(%rsp)
   502c7:	vmovdqa %ymm2,0xe0(%rsp)
   502d0:	vmovdqu %ymm1,0xc0(%rsp)
   502d9:	vmovdqa %ymm0,0xa0(%rsp)
   502e2:	mov    0x10(%rbx),%rax
   502e6:	mov    %rax,0x38(%rsp)
   502eb:	test   %rax,%rax
   502ee:	mov    %r14,0x28(%rsp)
   502f3:	je     507d3 <mibz2::mtf::mtf_avx2+0x623>
   502f9:	mov    0x8(%rbx),%rax
   502fd:	mov    %rax,0x40(%rsp)
   50302:	movzwl 0x128(%rbx),%ecx
   50309:	mov    0x8(%r14),%rax
   5030d:	mov    %rax,0x20(%rsp)
   50312:	movabs $0x800000000000000d,%rax
   5031c:	xor    %r13d,%r13d
   5031f:	mov    0x10(%rsp),%rdi
   50324:	mov    %rdi,%rdx
   50327:	sub    $0x40,%rdx
   5032b:	cmovb  %r13,%rdx
   5032f:	mov    %rdx,0x78(%rsp)
   50334:	mov    %ecx,0x4c(%rsp)
   50338:	vmovd  %ecx,%xmm0
   5033c:	vpbroadcastw %xmm0,%ymm0
   50341:	vmovdqa %ymm0,0x1e0(%rsp)
   5034a:	mov    $0x1,%r14b
   5034d:	mov    $0x1,%edx
   50352:	dec    %rax
   50355:	mov    %rax,0x60(%rsp)
   5035a:	xor    %r12d,%r12d
   5035d:	mov    0x18(%rsp),%rsi
   50362:	xor    %eax,%eax
   50364:	mov    %r13,%r15
   50367:	jmp    503f3 <mibz2::mtf::mtf_avx2+0x243>
   5036c:	nopl   0x0(%rax)
   50370:	dec    %rbx
   50373:	lea    0xa1(%rsp),%rdi
   5037b:	lea    0xa0(%rsp),%rsi
   50383:	mov    %edx,%r15d
   50386:	mov    %rbx,%rdx
   50389:	mov    %r8,%rbx
   5038c:	mov    %r13,0x30(%rsp)
   50391:	mov    %r14d,%r13d
   50394:	mov    %r9d,%r14d
   50397:	vzeroupper
   5039a:	call   *0x45760(%rip)        # 95b00 <memmove@GLIBC_2.2.5>
   503a0:	mov    %r14d,%r9d
   503a3:	mov    %r13d,%r14d
   503a6:	mov    0x30(%rsp),%r13
   503ab:	mov    %rbx,%r8
   503ae:	mov    %r15d,%edx
   503b1:	mov    0x10(%rsp),%rdi
   503b6:	mov    0x18(%rsp),%rsi
   503bb:	mov    %r9b,0xa0(%rsp)
   503c3:	test   $0x1,%r14b
   503c7:	jne    5050c <mibz2::mtf::mtf_avx2+0x35c>
   503cd:	movzbl %r9b,%eax
   503d1:	incl   (%rsi,%rax,4)
   503d4:	mov    0x20(%rsp),%rax
   503d9:	mov    %r9b,(%rax,%r13,1)
   503dd:	inc    %r13
   503e0:	xor    %eax,%eax
   503e2:	mov    %r13,%r15
   503e5:	mov    %r8,%r12
   503e8:	cmp    0x38(%rsp),%r8
   503ed:	jae    507d6 <mibz2::mtf::mtf_avx2+0x626>
   503f3:	mov    0x40(%rsp),%rcx
   503f8:	movzwl (%rcx,%r12,2),%ebx
   503fd:	lea    0x1(%r12),%r8
   50402:	cmp    $0x2,%rbx
   50406:	jae    50430 <mibz2::mtf::mtf_avx2+0x280>
   50408:	cmp    $0x1fffff,%edx
   5040e:	ja     50802 <mibz2::mtf::mtf_avx2+0x652>
   50414:	lea    0x1(%rbx),%ecx
   50417:	imul   %edx,%ecx
   5041a:	add    %ecx,%eax
   5041c:	add    %edx,%edx
   5041e:	mov    %r8,%r12
   50421:	cmp    0x38(%rsp),%r8
   50426:	jb     503f3 <mibz2::mtf::mtf_avx2+0x243>
   50428:	jmp    507d6 <mibz2::mtf::mtf_avx2+0x626>
   5042d:	nopl   (%rax)
   50430:	test   %eax,%eax
   50432:	je     504a5 <mibz2::mtf::mtf_avx2+0x2f5>
   50434:	mov    %eax,%edx
   50436:	lea    (%rdx,%r13,1),%r15
   5043a:	cmp    %rdi,%r15
   5043d:	ja     507f1 <mibz2::mtf::mtf_avx2+0x641>
   50443:	movzbl 0xa0(%rsp),%ecx
   5044b:	add    %eax,(%rsi,%rcx,4)
   5044e:	cmp    $0x40,%eax
   50451:	ja     50479 <mibz2::mtf::mtf_avx2+0x2c9>
   50453:	lea    0x40(%r13),%rax
   50457:	cmp    %rdi,%rax
   5045a:	ja     50479 <mibz2::mtf::mtf_avx2+0x2c9>
   5045c:	vmovd  %ecx,%xmm0
   50460:	vpbroadcastb %xmm0,%ymm0
   50465:	mov    0x20(%rsp),%rax
   5046a:	vmovdqu %ymm0,(%rax,%r13,1)
   50470:	vmovdqu %ymm0,0x20(%rax,%r13,1)
   50477:	jmp    5049d <mibz2::mtf::mtf_avx2+0x2ed>
   50479:	add    0x20(%rsp),%r13
   5047e:	movzbl %cl,%esi
   50481:	mov    %r13,%rdi
   50484:	mov    %r8,%r13
   50487:	vzeroupper
   5048a:	call   *0x456b0(%rip)        # 95b40 <memset@GLIBC_2.2.5>
   50490:	mov    %r13,%r8
   50493:	mov    0x10(%rsp),%rdi
   50498:	mov    0x18(%rsp),%rsi
   5049d:	mov    $0x1,%edx
   504a2:	mov    %r15,%r13
   504a5:	cmp    0x4c(%rsp),%bx
   504aa:	je     507d6 <mibz2::mtf::mtf_avx2+0x626>
   504b0:	cmp    %rdi,%r13
   504b3:	jae    507f1 <mibz2::mtf::mtf_avx2+0x641>
   504b9:	movzbl 0x9f(%rsp,%rbx,1),%r9d
   504c2:	cmp    $0x21,%bx
   504c6:	jae    50370 <mibz2::mtf::mtf_avx2+0x1c0>
   504cc:	vmovdqa 0xa0(%rsp),%ymm0
   504d5:	vmovd  %ebx,%xmm1
   504d9:	vpbroadcastb %xmm1,%ymm1
   504de:	vpcmpgtb -0x480e6(%rip),%ymm1,%ymm1        # 8400 <GCC_except_table701+0x920>
   504e6:	vpblendvb %ymm1,0x9f(%rsp),%ymm0,%ymm0
   504f1:	vmovdqa %ymm0,0xa0(%rsp)
   504fa:	mov    %r9b,0xa0(%rsp)
   50502:	test   $0x1,%r14b
   50506:	je     503cd <mibz2::mtf::mtf_avx2+0x21d>
   5050c:	add    $0x41,%r12
   50510:	cmp    0x38(%rsp),%r12
   50515:	mov    %r8,%r12
   50518:	ja     507a8 <mibz2::mtf::mtf_avx2+0x5f8>
   5051e:	mov    %edx,0xc(%rsp)
   50522:	mov    0x40(%rsp),%rax
   50527:	vmovdqu (%rax,%r8,2),%ymm0
   5052d:	vmovdqu 0x20(%rax,%r8,2),%ymm1
   50534:	vmovdqu 0x40(%rax,%r8,2),%ymm2
   5053b:	vmovdqu 0x60(%rax,%r8,2),%ymm3
   50542:	vpbroadcastw -0x480fb(%rip),%ymm6        # 8450 <GCC_except_table701+0x970>
   5054b:	vpminuw %ymm6,%ymm0,%ymm4
   50550:	vpcmpeqw %ymm4,%ymm0,%ymm4
   50554:	vpminuw %ymm6,%ymm1,%ymm5
   50559:	vpcmpeqw %ymm5,%ymm1,%ymm5
   5055d:	vpacksswb %ymm5,%ymm4,%ymm4
   50561:	vpermq $0xd8,%ymm4,%ymm4
   50567:	vpminuw %ymm6,%ymm2,%ymm5
   5056c:	vpcmpeqw %ymm5,%ymm2,%ymm5
   50570:	vpminuw %ymm6,%ymm3,%ymm6
   50575:	vpcmpeqw %ymm6,%ymm3,%ymm6
   50579:	vpacksswb %ymm6,%ymm5,%ymm5
   5057d:	vpermq $0xd8,%ymm5,%ymm5
   50583:	vpmovmskb %ymm4,%eax
   50587:	vpmovmskb %ymm5,%r10d
   5058b:	shl    $0x20,%r10
   5058f:	or     %rax,%r10
   50592:	mov    %r10,%rax
   50595:	shr    $1,%rax
   50598:	and    %r10,%rax
   5059b:	mov    %rax,%rcx
   5059e:	shr    $0x2,%rcx
   505a2:	and    %rax,%rcx
   505a5:	mov    %rcx,%rax
   505a8:	shr    $0x4,%rax
   505ac:	and    %rcx,%rax
   505af:	mov    %rax,%rcx
   505b2:	shr    $0x8,%rcx
   505b6:	and    %rax,%rcx
   505b9:	mov    %rcx,%rdx
   505bc:	shr    $0x6,%rdx
   505c0:	mov    $0x1,%al
   505c2:	test   %rcx,%rdx
   505c5:	jne    505f4 <mibz2::mtf::mtf_avx2+0x444>
   505c7:	vmovdqa 0x1e0(%rsp),%ymm6
   505d0:	vpcmpeqw %ymm6,%ymm3,%ymm4
   505d4:	vpcmpeqw %ymm6,%ymm2,%ymm5
   505d8:	vpor   %ymm4,%ymm5,%ymm4
   505dc:	vpcmpeqw %ymm6,%ymm1,%ymm5
   505e0:	vpcmpeqw %ymm6,%ymm0,%ymm6
   505e4:	vpor   %ymm5,%ymm6,%ymm5
   505e8:	vpor   %ymm4,%ymm5,%ymm4
   505ec:	vptest %ymm4,%ymm4
   505f1:	setne  %al
   505f4:	test   %al,%al
   505f6:	jne    507ce <mibz2::mtf::mtf_avx2+0x61e>
   505fc:	mov    0x40(%rsp),%rax
   50601:	lea    (%rax,%r8,2),%rax
   50605:	mov    %rax,0x68(%rsp)
   5060a:	vpxor  %xmm4,%xmm4,%xmm4
   5060e:	vpcmpeqw %ymm4,%ymm2,%ymm2
   50612:	vpcmpeqw %ymm4,%ymm3,%ymm3
   50616:	vpacksswb %ymm3,%ymm2,%ymm2
   5061a:	vpermq $0xd8,%ymm2,%ymm2
   50620:	vpcmpeqw %ymm4,%ymm0,%ymm0
   50624:	vpcmpeqw %ymm4,%ymm1,%ymm1
   50628:	vpacksswb %ymm1,%ymm0,%ymm0
   5062c:	vpermq $0xd8,%ymm0,%ymm0
   50632:	vpmovmskb %ymm0,%eax
   50636:	vpmovmskb %ymm2,%ecx
   5063a:	shl    $0x20,%rcx
   5063e:	or     %rax,%rcx
   50641:	not    %rcx
   50644:	mov    %rcx,0x70(%rsp)
   50649:	not    %r10
   5064c:	xor    %eax,%eax
   5064e:	mov    %r13,%rdi
   50651:	mov    %r8,0x50(%rsp)
   50656:	jmp    506aa <mibz2::mtf::mtf_avx2+0x4fa>
   50658:	nopl   0x0(%rax,%rax,1)
   50660:	dec    %rdx
   50663:	lea    0xa1(%rsp),%rdi
   5066b:	lea    0xa0(%rsp),%rsi
   50673:	mov    %r9d,%ebx
   50676:	vzeroupper
   50679:	call   *0x45481(%rip)        # 95b00 <memmove@GLIBC_2.2.5>
   5067f:	mov    %ebx,%r9d
   50682:	mov    0x18(%rsp),%rsi
   50687:	mov    0x50(%rsp),%r8
   5068c:	mov    0x30(%rsp),%r10
   50691:	blsr   %r10,%r10
   50696:	mov    %r9b,0xa0(%rsp)
   5069e:	mov    %r12d,%eax
   506a1:	mov    %r13,%rdi
   506a4:	je     5076d <mibz2::mtf::mtf_avx2+0x5bd>
   506aa:	tzcnt  %r10,%r15
   506af:	shrx   %rax,0x70(%rsp),%rcx
   506b6:	lea    0x1(%r15),%r12d
   506ba:	mov    %r12d,%edx
   506bd:	sub    %eax,%edx
   506bf:	bzhi   %rdx,%rcx,%rbx
   506c4:	lea    (%rbx,%rdi,1),%r13
   506c8:	cmp    0x78(%rsp),%r13
   506cd:	ja     50794 <mibz2::mtf::mtf_avx2+0x5e4>
   506d3:	movzbl %r9b,%r14d
   506d7:	cmp    $0x41,%rbx
   506db:	mov    %r10,0x30(%rsp)
   506e0:	jae    50700 <mibz2::mtf::mtf_avx2+0x550>
   506e2:	vmovd  %r9d,%xmm0
   506e7:	vpbroadcastb %xmm0,%ymm0
   506ec:	mov    0x20(%rsp),%rax
   506f1:	vmovdqu %ymm0,(%rax,%rdi,1)
   506f6:	vmovdqu %ymm0,0x20(%rax,%rdi,1)
   506fc:	jmp    50719 <mibz2::mtf::mtf_avx2+0x569>
   506fe:	xchg   %ax,%ax
   50700:	add    0x20(%rsp),%rdi
   50705:	mov    %r14d,%esi
   50708:	mov    %rbx,%rdx
   5070b:	vzeroupper
   5070e:	call   *0x4542c(%rip)        # 95b40 <memset@GLIBC_2.2.5>
   50714:	mov    0x18(%rsp),%rsi
   50719:	add    %ebx,(%rsi,%r14,4)
   5071d:	mov    0x68(%rsp),%rax
   50722:	movzwl (%rax,%r15,2),%edx
   50727:	movzbl 0x9f(%rsp,%rdx,1),%r9d
   50730:	cmp    $0x21,%rdx
   50734:	jae    50660 <mibz2::mtf::mtf_avx2+0x4b0>
   5073a:	vmovdqa 0xa0(%rsp),%ymm0
   50743:	vmovd  %edx,%xmm1
   50747:	vpbroadcastb %xmm1,%ymm1
   5074c:	vpcmpgtb -0x48354(%rip),%ymm1,%ymm1        # 8400 <GCC_except_table701+0x920>
   50754:	vpblendvb %ymm1,0x9f(%rsp),%ymm0,%ymm0
   5075f:	vmovdqa %ymm0,0xa0(%rsp)
   50768:	jmp    50687 <mibz2::mtf::mtf_avx2+0x4d7>
   5076d:	mov    %r12d,%eax
   50770:	lea    (%r8,%rax,1),%r12
   50774:	add    %r8,%rax
   50777:	add    $0x40,%rax
   5077b:	mov    %r12,%r8
   5077e:	cmp    0x38(%rsp),%rax
   50783:	mov    0x10(%rsp),%rdi
   50788:	mov    0xc(%rsp),%edx
   5078c:	jbe    5051e <mibz2::mtf::mtf_avx2+0x36e>
   50792:	jmp    507a8 <mibz2::mtf::mtf_avx2+0x5f8>
   50794:	mov    %eax,%eax
   50796:	add    %rax,%r8
   50799:	mov    %r8,%r12
   5079c:	mov    %rdi,%r13
   5079f:	mov    0x10(%rsp),%rdi
   507a4:	mov    0xc(%rsp),%edx
   507a8:	movzbl %r9b,%eax
   507ac:	incl   (%rsi,%rax,4)
   507af:	mov    0x20(%rsp),%rcx
   507b4:	mov    %al,(%rcx,%r13,1)
   507b8:	inc    %r13
   507bb:	xor    %r14d,%r14d
   507be:	mov    %r13,%r15
   507c1:	cmp    0x38(%rsp),%r12
   507c6:	jb     50362 <mibz2::mtf::mtf_avx2+0x1b2>
   507cc:	jmp    507d6 <mibz2::mtf::mtf_avx2+0x626>
   507ce:	mov    %r8,%r12
   507d1:	jmp    507a4 <mibz2::mtf::mtf_avx2+0x5f4>
   507d3:	xor    %r13d,%r13d
   507d6:	mov    0x28(%rsp),%rax
   507db:	mov    %r13,0x10(%rax)
   507df:	mov    0x58(%rsp),%rax
   507e4:	mov    %r15,0x8(%rax)
   507e8:	mov    $0xffffffffffffffff,%rcx
   507ef:	jmp    50815 <mibz2::mtf::mtf_avx2+0x665>
   507f1:	mov    0x28(%rsp),%rax
   507f6:	movabs $0x800000000000000d,%rcx
   50800:	jmp    5080c <mibz2::mtf::mtf_avx2+0x65c>
   50802:	mov    0x60(%rsp),%rcx
   50807:	mov    0x28(%rsp),%rax
   5080c:	mov    %r13,0x10(%rax)
   50810:	mov    0x58(%rsp),%rax
   50815:	mov    %rcx,(%rax)
   50818:	lea    -0x28(%rbp),%rsp
   5081c:	pop    %rbx
   5081d:	pop    %r12
   5081f:	pop    %r13
   50821:	pop    %r14
   50823:	pop    %r15
   50825:	pop    %rbp
   50826:	vzeroupper
   50829:	ret
   5082a:	mov    $0x1,%ecx
   5082f:	mov    $0x1,%r8d
   50835:	mov    %r14,%rdi
   50838:	xor    %esi,%esi
   5083a:	mov    0x10(%rsp),%rdx
   5083f:	call   35840 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   50844:	mov    0x18(%rsp),%rcx
   50849:	jmp    501f5 <mibz2::mtf::mtf_avx2+0x45>
   5084e:	int3
   5084f:	int3

