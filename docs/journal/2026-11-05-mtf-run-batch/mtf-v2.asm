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
   501c8:	mov    %rdx,%rbx
   501cb:	mov    %rsi,%r13
   501ce:	mov    %rdi,0x68(%rsp)
   501d3:	mov    0x118(%rsi),%rax
   501da:	movq   $0x0,0x10(%rdx)
   501e2:	mov    %rax,0x40(%rsp)
   501e7:	cmp    (%rdx),%rax
   501ea:	mov    %rcx,0x30(%rsp)
   501ef:	ja     508e1 <mibz2::mtf::mtf_avx2+0x731>
   501f5:	xor    %r12d,%r12d
   501f8:	mov    $0x400,%edx
   501fd:	mov    %rcx,%rdi
   50200:	xor    %esi,%esi
   50202:	call   *0x459f8(%rip)        # 95c00 <memset@GLIBC_2.2.5>
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
   50266:	vmovups 0xf8(%r13),%ymm0
   5026f:	vmovups %ymm0,0x180(%rsp)
   50278:	vmovups 0xd8(%r13),%ymm0
   50281:	vmovaps %ymm0,0x160(%rsp)
   5028a:	vmovups 0xb8(%r13),%ymm0
   50293:	vmovups %ymm0,0x140(%rsp)
   5029c:	vmovups 0x98(%r13),%ymm0
   502a5:	vmovaps %ymm0,0x120(%rsp)
   502ae:	vmovdqu 0x18(%r13),%ymm0
   502b4:	vmovdqu 0x38(%r13),%ymm1
   502ba:	vmovdqu 0x58(%r13),%ymm2
   502c0:	vmovdqu 0x78(%r13),%ymm3
   502c6:	vmovdqu %ymm3,0x100(%rsp)
   502cf:	vmovdqa %ymm2,0xe0(%rsp)
   502d8:	vmovdqu %ymm1,0xc0(%rsp)
   502e1:	vmovdqa %ymm0,0xa0(%rsp)
   502ea:	mov    0x10(%r13),%r8
   502ee:	test   %r8,%r8
   502f1:	mov    %rbx,0x60(%rsp)
   502f6:	je     50890 <mibz2::mtf::mtf_avx2+0x6e0>
   502fc:	mov    0x8(%r13),%r9
   50300:	movzwl 0x128(%r13),%eax
   50308:	mov    0x8(%rbx),%r13
   5030c:	movabs $0x800000000000000d,%rcx
   50316:	xor    %edi,%edi
   50318:	mov    0x40(%rsp),%r11
   5031d:	mov    %r11,%rdx
   50320:	sub    $0x40,%rdx
   50324:	cmovb  %rdi,%rdx
   50328:	mov    %rdx,0x70(%rsp)
   5032d:	mov    %eax,0x5c(%rsp)
   50331:	vmovd  %eax,%xmm0
   50335:	vpbroadcastw %xmm0,%ymm0
   5033a:	vmovdqa %ymm0,0x1e0(%rsp)
   50343:	mov    $0x1,%al
   50345:	mov    %eax,0x4c(%rsp)
   50349:	mov    $0x1,%eax
   5034e:	dec    %rcx
   50351:	mov    %rcx,0x78(%rsp)
   50356:	vmovdqa -0x47f5e(%rip),%ymm7        # 8400 <GCC_except_table701+0x920>
   5035e:	xor    %r15d,%r15d
   50361:	mov    0x30(%rsp),%r10
   50366:	mov    %r8,0x50(%rsp)
   5036b:	mov    %r9,0x38(%rsp)
   50370:	xor    %ecx,%ecx
   50372:	mov    %rdi,%r12
   50375:	jmp    503b2 <mibz2::mtf::mtf_avx2+0x202>
   50377:	nopw   0x0(%rax,%rax,1)
   50380:	mov    %cl,0xa0(%rsp)
   50387:	testb  $0x1,0x4c(%rsp)
   5038c:	jne    5055e <mibz2::mtf::mtf_avx2+0x3ae>
   50392:	movzbl %cl,%edx
   50395:	incl   (%r10,%rdx,4)
   50399:	mov    %cl,0x0(%r13,%rdi,1)
   5039e:	inc    %rdi
   503a1:	xor    %ecx,%ecx
   503a3:	mov    %rdi,%r12
   503a6:	mov    %rbx,%r15
   503a9:	cmp    %r8,%rbx
   503ac:	jae    50892 <mibz2::mtf::mtf_avx2+0x6e2>
   503b2:	movzwl (%r9,%r15,2),%r14d
   503b7:	lea    0x1(%r15),%rbx
   503bb:	cmp    $0x2,%r14
   503bf:	jae    503f0 <mibz2::mtf::mtf_avx2+0x240>
   503c1:	cmp    $0x1fffff,%eax
   503c6:	ja     508b9 <mibz2::mtf::mtf_avx2+0x709>
   503cc:	inc    %r14d
   503cf:	imul   %eax,%r14d
   503d3:	add    %ecx,%r14d
   503d6:	add    %eax,%eax
   503d8:	mov    %r14d,%ecx
   503db:	mov    %rbx,%r15
   503de:	cmp    %r8,%rbx
   503e1:	jb     503b2 <mibz2::mtf::mtf_avx2+0x202>
   503e3:	jmp    50892 <mibz2::mtf::mtf_avx2+0x6e2>
   503e8:	nopl   0x0(%rax,%rax,1)
   503f0:	test   %ecx,%ecx
   503f2:	je     50469 <mibz2::mtf::mtf_avx2+0x2b9>
   503f4:	mov    %ecx,%edx
   503f6:	lea    (%rdi,%rdx,1),%r12
   503fa:	cmp    %r11,%r12
   503fd:	ja     508ad <mibz2::mtf::mtf_avx2+0x6fd>
   50403:	movzbl 0xa0(%rsp),%eax
   5040b:	add    %ecx,(%r10,%rax,4)
   5040f:	cmp    $0x40,%ecx
   50412:	ja     50436 <mibz2::mtf::mtf_avx2+0x286>
   50414:	lea    0x40(%rdi),%rcx
   50418:	cmp    %r11,%rcx
   5041b:	ja     50436 <mibz2::mtf::mtf_avx2+0x286>
   5041d:	vmovd  %eax,%xmm0
   50421:	vpbroadcastb %xmm0,%ymm0
   50426:	vmovdqu %ymm0,0x0(%r13,%rdi,1)
   5042d:	vmovdqu %ymm0,0x20(%r13,%rdi,1)
   50434:	jmp    50461 <mibz2::mtf::mtf_avx2+0x2b1>
   50436:	add    %r13,%rdi
   50439:	movzbl %al,%esi
   5043c:	vzeroupper
   5043f:	call   *0x457bb(%rip)        # 95c00 <memset@GLIBC_2.2.5>
   50445:	vmovdqa -0x4804d(%rip),%ymm7        # 8400 <GCC_except_table701+0x920>
   5044d:	mov    0x38(%rsp),%r9
   50452:	mov    0x50(%rsp),%r8
   50457:	mov    0x40(%rsp),%r11
   5045c:	mov    0x30(%rsp),%r10
   50461:	mov    $0x1,%eax
   50466:	mov    %r12,%rdi
   50469:	cmp    0x5c(%rsp),%r14w
   5046f:	je     50892 <mibz2::mtf::mtf_avx2+0x6e2>
   50475:	cmp    %r11,%rdi
   50478:	jae    508ad <mibz2::mtf::mtf_avx2+0x6fd>
   5047e:	lea    -0x1(%r14),%rcx
   50482:	mov    %rcx,%rdx
   50485:	and    $0xffffffffffffffe0,%rdx
   50489:	vmovdqa 0xa0(%rsp,%rdx,1),%ymm0
   50492:	and    $0x1f,%cl
   50495:	inc    %cl
   50497:	vmovd  %ecx,%xmm1
   5049b:	vpbroadcastb %xmm1,%ymm1
   504a0:	vpcmpgtb %ymm7,%ymm1,%ymm1
   504a4:	vpblendvb %ymm1,0x9f(%rsp,%rdx,1),%ymm0,%ymm0
   504af:	movzbl 0x9f(%rsp,%r14,1),%ecx
   504b8:	vmovdqa %ymm0,0xa0(%rsp,%rdx,1)
   504c1:	test   %rdx,%rdx
   504c4:	je     50380 <mibz2::mtf::mtf_avx2+0x1d0>
   504ca:	add    $0xffffffffffffffdf,%r14
   504ce:	mov    %r14d,%esi
   504d1:	not    %esi
   504d3:	test   $0x60,%sil
   504d7:	je     50508 <mibz2::mtf::mtf_avx2+0x358>
   504d9:	mov    %r14d,%esi
   504dc:	shr    $0x5,%esi
   504df:	inc    %esi
   504e1:	and    $0x3,%esi
   504e4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   504f0:	vmovdqu 0x7f(%rsp,%rdx,1),%ymm0
   504f6:	vmovdqa %ymm0,0x80(%rsp,%rdx,1)
   504ff:	add    $0xffffffffffffffe0,%rdx
   50503:	dec    %rsi
   50506:	jne    504f0 <mibz2::mtf::mtf_avx2+0x340>
   50508:	cmp    $0x60,%r14d
   5050c:	jb     50380 <mibz2::mtf::mtf_avx2+0x1d0>
   50512:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   50520:	vmovdqu 0x1f(%rsp,%rdx,1),%ymm0
   50526:	vmovdqu 0x3f(%rsp,%rdx,1),%ymm1
   5052c:	vmovdqu 0x5f(%rsp,%rdx,1),%ymm2
   50532:	vmovdqu 0x7f(%rsp,%rdx,1),%ymm3
   50538:	vmovdqa %ymm3,0x80(%rsp,%rdx,1)
   50541:	vmovdqa %ymm2,0x60(%rsp,%rdx,1)
   50547:	vmovdqa %ymm1,0x40(%rsp,%rdx,1)
   5054d:	vmovdqa %ymm0,0x20(%rsp,%rdx,1)
   50553:	add    $0xffffffffffffff80,%rdx
   50557:	jne    50520 <mibz2::mtf::mtf_avx2+0x370>
   50559:	jmp    50380 <mibz2::mtf::mtf_avx2+0x1d0>
   5055e:	add    $0x41,%r15
   50562:	cmp    %r8,%r15
   50565:	mov    %rbx,%r15
   50568:	ja     50866 <mibz2::mtf::mtf_avx2+0x6b6>
   5056e:	vmovdqu (%r9,%rbx,2),%ymm0
   50574:	vmovdqu 0x20(%r9,%rbx,2),%ymm1
   5057b:	vmovdqu 0x40(%r9,%rbx,2),%ymm2
   50582:	vmovdqu 0x60(%r9,%rbx,2),%ymm3
   50589:	vpbroadcastw -0x48142(%rip),%ymm6        # 8450 <GCC_except_table701+0x970>
   50592:	vpminuw %ymm6,%ymm0,%ymm4
   50597:	vpcmpeqw %ymm4,%ymm0,%ymm4
   5059b:	vpminuw %ymm6,%ymm1,%ymm5
   505a0:	vpcmpeqw %ymm5,%ymm1,%ymm5
   505a4:	vpacksswb %ymm5,%ymm4,%ymm4
   505a8:	vpermq $0xd8,%ymm4,%ymm4
   505ae:	vpminuw %ymm6,%ymm2,%ymm5
   505b3:	vpcmpeqw %ymm5,%ymm2,%ymm5
   505b7:	vpminuw %ymm6,%ymm3,%ymm6
   505bc:	vpcmpeqw %ymm6,%ymm3,%ymm6
   505c0:	vpacksswb %ymm6,%ymm5,%ymm5
   505c4:	vpermq $0xd8,%ymm5,%ymm5
   505ca:	vpmovmskb %ymm4,%esi
   505ce:	vpmovmskb %ymm5,%edx
   505d2:	shl    $0x20,%rdx
   505d6:	or     %rsi,%rdx
   505d9:	mov    %rdx,%rsi
   505dc:	shr    $1,%rsi
   505df:	and    %rdx,%rsi
   505e2:	mov    %rsi,%r8
   505e5:	shr    $0x2,%r8
   505e9:	and    %rsi,%r8
   505ec:	mov    %r8,%rsi
   505ef:	shr    $0x4,%rsi
   505f3:	and    %r8,%rsi
   505f6:	mov    %rsi,%r8
   505f9:	shr    $0x8,%r8
   505fd:	and    %rsi,%r8
   50600:	mov    %r8,%r9
   50603:	shr    $0x6,%r9
   50607:	mov    $0x1,%sil
   5060a:	test   %r8,%r9
   5060d:	jne    5063d <mibz2::mtf::mtf_avx2+0x48d>
   5060f:	vmovdqa 0x1e0(%rsp),%ymm6
   50618:	vpcmpeqw %ymm6,%ymm3,%ymm4
   5061c:	vpcmpeqw %ymm6,%ymm2,%ymm5
   50620:	vpor   %ymm4,%ymm5,%ymm4
   50624:	vpcmpeqw %ymm6,%ymm1,%ymm5
   50628:	vpcmpeqw %ymm6,%ymm0,%ymm6
   5062c:	vpor   %ymm5,%ymm6,%ymm5
   50630:	vpor   %ymm4,%ymm5,%ymm4
   50634:	vptest %ymm4,%ymm4
   50639:	setne  %sil
   5063d:	test   %sil,%sil
   50640:	mov    0x70(%rsp),%r12
   50645:	jne    5088b <mibz2::mtf::mtf_avx2+0x6db>
   5064b:	mov    0x38(%rsp),%rsi
   50650:	lea    (%rsi,%rbx,2),%rsi
   50654:	vpxor  %xmm4,%xmm4,%xmm4
   50658:	vpcmpeqw %ymm4,%ymm2,%ymm2
   5065c:	vpcmpeqw %ymm4,%ymm3,%ymm3
   50660:	vpacksswb %ymm3,%ymm2,%ymm2
   50664:	vpermq $0xd8,%ymm2,%ymm2
   5066a:	vpcmpeqw %ymm4,%ymm0,%ymm0
   5066e:	vpcmpeqw %ymm4,%ymm1,%ymm1
   50672:	vpacksswb %ymm1,%ymm0,%ymm0
   50676:	vpermq $0xd8,%ymm0,%ymm0
   5067c:	vpmovmskb %ymm0,%r9d
   50680:	vpmovmskb %ymm2,%r8d
   50684:	shl    $0x20,%r8
   50688:	or     %r9,%r8
   5068b:	not    %r8
   5068e:	not    %rdx
   50691:	xor    %r9d,%r9d
   50694:	jmp    506b2 <mibz2::mtf::mtf_avx2+0x502>
   50696:	cs nopw 0x0(%rax,%rax,1)
   506a0:	blsr   %rdx,%rdx
   506a5:	mov    %cl,0xa0(%rsp)
   506ac:	je     50816 <mibz2::mtf::mtf_avx2+0x666>
   506b2:	mov    %rdi,%r10
   506b5:	mov    %r9d,%r15d
   506b8:	tzcnt  %rdx,%r11
   506bd:	shrx   %r15,%r8,%rdi
   506c2:	lea    0x1(%r11),%r9d
   506c6:	mov    %r9d,%r14d
   506c9:	sub    %r15d,%r14d
   506cc:	bzhi   %r14,%rdi,%r14
   506d1:	lea    (%r14,%r10,1),%rdi
   506d5:	cmp    %r12,%rdi
   506d8:	ja     50846 <mibz2::mtf::mtf_avx2+0x696>
   506de:	vmovd  %ecx,%xmm0
   506e2:	vpbroadcastb %xmm0,%ymm0
   506e7:	vmovdqu %ymm0,0x0(%r13,%r10,1)
   506ee:	vmovdqu %ymm0,0x20(%r13,%r10,1)
   506f5:	cmp    $0x41,%r14
   506f9:	jb     5071f <mibz2::mtf::mtf_avx2+0x56f>
   506fb:	add    %r13,%r10
   506fe:	mov    $0x40,%r15d
   50704:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   50710:	vmovdqu %ymm0,(%r10,%r15,1)
   50716:	add    $0x20,%r15
   5071a:	cmp    %r14,%r15
   5071d:	jb     50710 <mibz2::mtf::mtf_avx2+0x560>
   5071f:	movzbl %cl,%ecx
   50722:	mov    0x30(%rsp),%r10
   50727:	add    %r14d,(%r10,%rcx,4)
   5072b:	movzwl (%rsi,%r11,2),%r11d
   50730:	lea    -0x1(%r11),%rcx
   50734:	mov    %rcx,%r10
   50737:	and    $0xffffffffffffffe0,%r10
   5073b:	vmovdqa 0xa0(%rsp,%r10,1),%ymm0
   50745:	and    $0x1f,%cl
   50748:	inc    %cl
   5074a:	vmovd  %ecx,%xmm1
   5074e:	vpbroadcastb %xmm1,%ymm1
   50753:	vpcmpgtb %ymm7,%ymm1,%ymm1
   50757:	vpblendvb %ymm1,0x9f(%rsp,%r10,1),%ymm0,%ymm0
   50762:	movzbl 0x9f(%rsp,%r11,1),%ecx
   5076b:	vmovdqa %ymm0,0xa0(%rsp,%r10,1)
   50775:	test   %r10,%r10
   50778:	je     506a0 <mibz2::mtf::mtf_avx2+0x4f0>
   5077e:	add    $0xffffffffffffffdf,%r11
   50782:	mov    %r11d,%r14d
   50785:	not    %r14d
   50788:	test   $0x60,%r14b
   5078c:	je     507ba <mibz2::mtf::mtf_avx2+0x60a>
   5078e:	mov    %r11d,%r14d
   50791:	shr    $0x5,%r14d
   50795:	inc    %r14d
   50798:	and    $0x3,%r14d
   5079c:	nopl   0x0(%rax)
   507a0:	vmovdqu 0x7f(%rsp,%r10,1),%ymm0
   507a7:	vmovdqa %ymm0,0x80(%rsp,%r10,1)
   507b1:	add    $0xffffffffffffffe0,%r10
   507b5:	dec    %r14
   507b8:	jne    507a0 <mibz2::mtf::mtf_avx2+0x5f0>
   507ba:	cmp    $0x60,%r11d
   507be:	jb     506a0 <mibz2::mtf::mtf_avx2+0x4f0>
   507c4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   507d0:	vmovdqu 0x1f(%rsp,%r10,1),%ymm0
   507d7:	vmovdqu 0x3f(%rsp,%r10,1),%ymm1
   507de:	vmovdqu 0x5f(%rsp,%r10,1),%ymm2
   507e5:	vmovdqu 0x7f(%rsp,%r10,1),%ymm3
   507ec:	vmovdqa %ymm3,0x80(%rsp,%r10,1)
   507f6:	vmovdqa %ymm2,0x60(%rsp,%r10,1)
   507fd:	vmovdqa %ymm1,0x40(%rsp,%r10,1)
   50804:	vmovdqa %ymm0,0x20(%rsp,%r10,1)
   5080b:	add    $0xffffffffffffff80,%r10
   5080f:	jne    507d0 <mibz2::mtf::mtf_avx2+0x620>
   50811:	jmp    506a0 <mibz2::mtf::mtf_avx2+0x4f0>
   50816:	mov    %r9d,%edx
   50819:	lea    (%rbx,%rdx,1),%r15
   5081d:	add    %rbx,%rdx
   50820:	add    $0x40,%rdx
   50824:	mov    %r15,%rbx
   50827:	mov    0x50(%rsp),%r8
   5082c:	cmp    %r8,%rdx
   5082f:	mov    0x30(%rsp),%r10
   50834:	mov    0x40(%rsp),%r11
   50839:	mov    0x38(%rsp),%r9
   5083e:	jbe    5056e <mibz2::mtf::mtf_avx2+0x3be>
   50844:	jmp    50866 <mibz2::mtf::mtf_avx2+0x6b6>
   50846:	mov    %r15d,%edx
   50849:	add    %rdx,%rbx
   5084c:	mov    %rbx,%r15
   5084f:	mov    %r10,%rdi
   50852:	mov    0x30(%rsp),%r10
   50857:	mov    0x40(%rsp),%r11
   5085c:	mov    0x50(%rsp),%r8
   50861:	mov    0x38(%rsp),%r9
   50866:	movzbl %cl,%ecx
   50869:	incl   (%r10,%rcx,4)
   5086d:	mov    %cl,0x0(%r13,%rdi,1)
   50872:	inc    %rdi
   50875:	movl   $0x0,0x4c(%rsp)
   5087d:	mov    %rdi,%r12
   50880:	cmp    %r8,%r15
   50883:	jb     50370 <mibz2::mtf::mtf_avx2+0x1c0>
   50889:	jmp    50892 <mibz2::mtf::mtf_avx2+0x6e2>
   5088b:	mov    %rbx,%r15
   5088e:	jmp    5085c <mibz2::mtf::mtf_avx2+0x6ac>
   50890:	xor    %edi,%edi
   50892:	mov    0x60(%rsp),%rax
   50897:	mov    %rdi,0x10(%rax)
   5089b:	mov    0x68(%rsp),%rax
   508a0:	mov    %r12,0x8(%rax)
   508a4:	mov    $0xffffffffffffffff,%rcx
   508ab:	jmp    508cc <mibz2::mtf::mtf_avx2+0x71c>
   508ad:	movabs $0x800000000000000d,%rcx
   508b7:	jmp    508be <mibz2::mtf::mtf_avx2+0x70e>
   508b9:	mov    0x78(%rsp),%rcx
   508be:	mov    0x60(%rsp),%rax
   508c3:	mov    %rdi,0x10(%rax)
   508c7:	mov    0x68(%rsp),%rax
   508cc:	mov    %rcx,(%rax)
   508cf:	lea    -0x28(%rbp),%rsp
   508d3:	pop    %rbx
   508d4:	pop    %r12
   508d6:	pop    %r13
   508d8:	pop    %r14
   508da:	pop    %r15
   508dc:	pop    %rbp
   508dd:	vzeroupper
   508e0:	ret
   508e1:	mov    $0x1,%ecx
   508e6:	mov    $0x1,%r8d
   508ec:	mov    %rbx,%rdi
   508ef:	xor    %esi,%esi
   508f1:	mov    0x40(%rsp),%rdx
   508f6:	call   35840 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   508fb:	mov    0x30(%rsp),%rcx
   50900:	jmp    501f5 <mibz2::mtf::mtf_avx2+0x45>
   50905:	int3
   50906:	int3
   50907:	int3
   50908:	int3
   50909:	int3
   5090a:	int3
   5090b:	int3
   5090c:	int3
   5090d:	int3
   5090e:	int3
   5090f:	int3

