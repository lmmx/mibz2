0000000000050240 <mibz2::mtf::mtf_avx2>:
   50240:	push   %r15
   50242:	push   %r14
   50244:	push   %r13
   50246:	push   %r12
   50248:	push   %rbx
   50249:	mov    0x8(%rsi),%r8
   5024d:	mov    0x10(%rsi),%rax
   50251:	movabs $0x3ffffffffffffff0,%r9
   5025b:	mov    %rax,%r10
   5025e:	and    %r9,%r10
   50261:	mov    %eax,%r11d
   50264:	and    $0xf,%r11d
   50268:	test   %r10,%r10
   5026b:	je     50287 <mibz2::mtf::mtf_avx2+0x47>
   5026d:	lea    -0x10(%r10),%r15
   50271:	mov    %r15d,%ebx
   50274:	not    %ebx
   50276:	test   $0x30,%bl
   50279:	jne    50290 <mibz2::mtf::mtf_avx2+0x50>
   5027b:	vpxor  %xmm0,%xmm0,%xmm0
   5027f:	mov    %r8,%rbx
   50282:	mov    %r10,%r14
   50285:	jmp    502de <mibz2::mtf::mtf_avx2+0x9e>
   50287:	vpxor  %xmm0,%xmm0,%xmm0
   5028b:	jmp    50348 <mibz2::mtf::mtf_avx2+0x108>
   50290:	mov    %r15d,%r12d
   50293:	shr    $0x4,%r12d
   50297:	inc    %r12d
   5029a:	and    $0x3,%r12d
   5029e:	vpxor  %xmm0,%xmm0,%xmm0
   502a2:	vpbroadcastw -0x47df7(%rip),%ymm1        # 84b4 <GCC_except_table703+0x9d4>
   502ab:	mov    %r8,%rbx
   502ae:	mov    %r10,%r14
   502b1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   502c0:	add    $0xfffffffffffffff0,%r14
   502c4:	vmovdqu (%rbx),%ymm2
   502c8:	add    $0x20,%rbx
   502cc:	vpminuw %ymm1,%ymm2,%ymm3
   502d1:	vpcmpeqw %ymm3,%ymm2,%ymm2
   502d5:	vpsubw %ymm2,%ymm0,%ymm0
   502d9:	dec    %r12
   502dc:	jne    502c0 <mibz2::mtf::mtf_avx2+0x80>
   502de:	cmp    $0x30,%r15
   502e2:	jb     50348 <mibz2::mtf::mtf_avx2+0x108>
   502e4:	xor    %r15d,%r15d
   502e7:	vpbroadcastw -0x47e3c(%rip),%ymm1        # 84b4 <GCC_except_table703+0x9d4>
   502f0:	vmovdqu (%rbx,%r15,2),%ymm2
   502f6:	vmovdqu 0x20(%rbx,%r15,2),%ymm3
   502fd:	vmovdqu 0x40(%rbx,%r15,2),%ymm4
   50304:	vmovdqu 0x60(%rbx,%r15,2),%ymm5
   5030b:	vpminuw %ymm1,%ymm2,%ymm6
   50310:	vpcmpeqw %ymm6,%ymm2,%ymm2
   50314:	vpsubw %ymm2,%ymm0,%ymm0
   50318:	vpminuw %ymm1,%ymm3,%ymm2
   5031d:	vpcmpeqw %ymm2,%ymm3,%ymm2
   50321:	vpsubw %ymm2,%ymm0,%ymm0
   50325:	vpminuw %ymm1,%ymm4,%ymm2
   5032a:	vpcmpeqw %ymm2,%ymm4,%ymm2
   5032e:	vpsubw %ymm2,%ymm0,%ymm0
   50332:	vpminuw %ymm1,%ymm5,%ymm2
   50337:	vpcmpeqw %ymm2,%ymm5,%ymm2
   5033b:	vpsubw %ymm2,%ymm0,%ymm0
   5033f:	add    $0x40,%r15
   50343:	cmp    %r15,%r14
   50346:	jne    502f0 <mibz2::mtf::mtf_avx2+0xb0>
   50348:	test   %r11,%r11
   5034b:	je     50360 <mibz2::mtf::mtf_avx2+0x120>
   5034d:	mov    %eax,%ebx
   5034f:	and    $0x3,%ebx
   50352:	cmp    $0x4,%r11d
   50356:	jae    50365 <mibz2::mtf::mtf_avx2+0x125>
   50358:	xor    %r11d,%r11d
   5035b:	xor    %r10d,%r10d
   5035e:	jmp    503c0 <mibz2::mtf::mtf_avx2+0x180>
   50360:	xor    %r10d,%r10d
   50363:	jmp    503e2 <mibz2::mtf::mtf_avx2+0x1a2>
   50365:	lea    (%r8,%r10,2),%r14
   50369:	mov    %eax,%r15d
   5036c:	and    $0xc,%r15d
   50370:	xor    %r11d,%r11d
   50373:	vpbroadcastd -0x47ef0(%rip),%xmm1        # 848c <GCC_except_table703+0x9ac>
   5037c:	movabs $0x4332322132212110,%r12
   50386:	xor    %r10d,%r10d
   50389:	nopl   0x0(%rax)
   50390:	vpmovzxwd (%r14,%r11,2),%xmm2
   50396:	vpcmpgtd %xmm1,%xmm2,%xmm2
   5039a:	vmovmskps %xmm2,%r13d
   5039e:	xor    $0xf,%r13d
   503a2:	shl    $0x2,%r13d
   503a6:	shrx   %r13,%r12,%r13
   503ab:	and    $0x7,%r13d
   503af:	add    %r13,%r10
   503b2:	add    $0x4,%r11
   503b6:	cmp    %r11,%r15
   503b9:	jne    50390 <mibz2::mtf::mtf_avx2+0x150>
   503bb:	test   %rbx,%rbx
   503be:	je     503e2 <mibz2::mtf::mtf_avx2+0x1a2>
   503c0:	and    %rax,%r9
   503c3:	add    %r11,%r11
   503c6:	lea    (%r11,%r9,2),%r9
   503ca:	add    %r9,%r8
   503cd:	xor    %r9d,%r9d
   503d0:	cmpw   $0x2,(%r8,%r9,2)
   503d6:	adc    $0x0,%r10
   503da:	inc    %r9
   503dd:	cmp    %r9,%rbx
   503e0:	jne    503d0 <mibz2::mtf::mtf_avx2+0x190>
   503e2:	vextracti128 $0x1,%ymm0,%xmm1
   503e8:	vpshufd $0xee,%xmm1,%xmm2
   503ed:	vpmovzxwq %xmm2,%ymm2
   503f2:	vpshufd $0xee,%xmm0,%xmm3
   503f7:	vpmovzxwq %xmm3,%ymm3
   503fc:	vpaddq %ymm2,%ymm3,%ymm2
   50400:	vpmovzxwq %xmm1,%ymm1
   50405:	vpmovzxwq %xmm0,%ymm0
   5040a:	vpaddq %ymm1,%ymm0,%ymm0
   5040e:	vpaddq %ymm2,%ymm0,%ymm0
   50412:	vextracti128 $0x1,%ymm0,%xmm1
   50418:	vpaddq %xmm1,%xmm0,%xmm0
   5041c:	vpshufd $0xee,%xmm0,%xmm1
   50421:	vpaddq %xmm1,%xmm0,%xmm0
   50425:	vmovq  %xmm0,%r8
   5042a:	add    %r10,%r8
   5042d:	shl    $0x3,%r8
   50431:	cmp    %rax,%r8
   50434:	jae    50447 <mibz2::mtf::mtf_avx2+0x207>
   50436:	pop    %rbx
   50437:	pop    %r12
   50439:	pop    %r13
   5043b:	pop    %r14
   5043d:	pop    %r15
   5043f:	vzeroupper
   50442:	jmp    50bc0 <mibz2::mtf::mtf_avx2_symbols>
   50447:	pop    %rbx
   50448:	pop    %r12
   5044a:	pop    %r13
   5044c:	pop    %r14
   5044e:	pop    %r15
   50450:	vzeroupper
   50453:	jmp    50460 <mibz2::mtf::mtf_avx2_windows>
   50458:	int3
   50459:	int3
   5045a:	int3
   5045b:	int3
   5045c:	int3
   5045d:	int3
   5045e:	int3
   5045f:	int3

0000000000050460 <mibz2::mtf::mtf_avx2_windows>:
   50460:	push   %rbp
   50461:	mov    %rsp,%rbp
   50464:	push   %r15
   50466:	push   %r14
   50468:	push   %r13
   5046a:	push   %r12
   5046c:	push   %rbx
   5046d:	and    $0xffffffffffffffc0,%rsp
   50471:	sub    $0x240,%rsp
   50478:	mov    %rdx,%rbx
   5047b:	mov    %rsi,%r13
   5047e:	mov    %rdi,0x68(%rsp)
   50483:	mov    0x118(%rsi),%rax
   5048a:	movq   $0x0,0x10(%rdx)
   50492:	mov    %rax,0x40(%rsp)
   50497:	cmp    (%rdx),%rax
   5049a:	mov    %rcx,0x30(%rsp)
   5049f:	ja     50b91 <mibz2::mtf::mtf_avx2_windows+0x731>
   504a5:	xor    %r12d,%r12d
   504a8:	mov    $0x400,%edx
   504ad:	mov    %rcx,%rdi
   504b0:	xor    %esi,%esi
   504b2:	call   *0x45d88(%rip)        # 96240 <memset@GLIBC_2.2.5>
   504b8:	vxorps %xmm0,%xmm0,%xmm0
   504bc:	vmovaps %ymm0,0x1a0(%rsp)
   504c5:	vmovaps %ymm0,0x180(%rsp)
   504ce:	vmovaps %ymm0,0x160(%rsp)
   504d7:	vmovaps %ymm0,0x140(%rsp)
   504e0:	vmovaps %ymm0,0x120(%rsp)
   504e9:	vmovaps %ymm0,0x100(%rsp)
   504f2:	vmovaps %ymm0,0xe0(%rsp)
   504fb:	vmovaps %ymm0,0xc0(%rsp)
   50504:	vmovaps %ymm0,0xa0(%rsp)
   5050d:	vmovaps %ymm0,0x80(%rsp)
   50516:	vmovups 0xf8(%r13),%ymm0
   5051f:	vmovups %ymm0,0x180(%rsp)
   50528:	vmovups 0xd8(%r13),%ymm0
   50531:	vmovaps %ymm0,0x160(%rsp)
   5053a:	vmovups 0xb8(%r13),%ymm0
   50543:	vmovups %ymm0,0x140(%rsp)
   5054c:	vmovups 0x98(%r13),%ymm0
   50555:	vmovaps %ymm0,0x120(%rsp)
   5055e:	vmovdqu 0x18(%r13),%ymm0
   50564:	vmovdqu 0x38(%r13),%ymm1
   5056a:	vmovdqu 0x58(%r13),%ymm2
   50570:	vmovdqu 0x78(%r13),%ymm3
   50576:	vmovdqu %ymm3,0x100(%rsp)
   5057f:	vmovdqa %ymm2,0xe0(%rsp)
   50588:	vmovdqu %ymm1,0xc0(%rsp)
   50591:	vmovdqa %ymm0,0xa0(%rsp)
   5059a:	mov    0x10(%r13),%r8
   5059e:	test   %r8,%r8
   505a1:	mov    %rbx,0x60(%rsp)
   505a6:	je     50b40 <mibz2::mtf::mtf_avx2_windows+0x6e0>
   505ac:	mov    0x8(%r13),%r9
   505b0:	movzwl 0x128(%r13),%eax
   505b8:	mov    0x8(%rbx),%r13
   505bc:	movabs $0x800000000000000d,%rcx
   505c6:	xor    %edi,%edi
   505c8:	mov    0x40(%rsp),%r11
   505cd:	mov    %r11,%rdx
   505d0:	sub    $0x40,%rdx
   505d4:	cmovb  %rdi,%rdx
   505d8:	mov    %rdx,0x70(%rsp)
   505dd:	mov    %eax,0x5c(%rsp)
   505e1:	vmovd  %eax,%xmm0
   505e5:	vpbroadcastw %xmm0,%ymm0
   505ea:	vmovdqa %ymm0,0x1e0(%rsp)
   505f3:	mov    $0x1,%al
   505f5:	mov    %eax,0x4c(%rsp)
   505f9:	mov    $0x1,%eax
   505fe:	dec    %rcx
   50601:	mov    %rcx,0x78(%rsp)
   50606:	vmovdqa -0x4820e(%rip),%ymm7        # 8400 <GCC_except_table703+0x920>
   5060e:	xor    %r15d,%r15d
   50611:	mov    0x30(%rsp),%r10
   50616:	mov    %r8,0x50(%rsp)
   5061b:	mov    %r9,0x38(%rsp)
   50620:	xor    %ecx,%ecx
   50622:	mov    %rdi,%r12
   50625:	jmp    50662 <mibz2::mtf::mtf_avx2_windows+0x202>
   50627:	nopw   0x0(%rax,%rax,1)
   50630:	mov    %cl,0xa0(%rsp)
   50637:	testb  $0x1,0x4c(%rsp)
   5063c:	jne    5080e <mibz2::mtf::mtf_avx2_windows+0x3ae>
   50642:	movzbl %cl,%edx
   50645:	incl   (%r10,%rdx,4)
   50649:	mov    %cl,0x0(%r13,%rdi,1)
   5064e:	inc    %rdi
   50651:	xor    %ecx,%ecx
   50653:	mov    %rdi,%r12
   50656:	mov    %rbx,%r15
   50659:	cmp    %r8,%rbx
   5065c:	jae    50b42 <mibz2::mtf::mtf_avx2_windows+0x6e2>
   50662:	movzwl (%r9,%r15,2),%r14d
   50667:	lea    0x1(%r15),%rbx
   5066b:	cmp    $0x2,%r14
   5066f:	jae    506a0 <mibz2::mtf::mtf_avx2_windows+0x240>
   50671:	cmp    $0x1fffff,%eax
   50676:	ja     50b69 <mibz2::mtf::mtf_avx2_windows+0x709>
   5067c:	inc    %r14d
   5067f:	imul   %eax,%r14d
   50683:	add    %ecx,%r14d
   50686:	add    %eax,%eax
   50688:	mov    %r14d,%ecx
   5068b:	mov    %rbx,%r15
   5068e:	cmp    %r8,%rbx
   50691:	jb     50662 <mibz2::mtf::mtf_avx2_windows+0x202>
   50693:	jmp    50b42 <mibz2::mtf::mtf_avx2_windows+0x6e2>
   50698:	nopl   0x0(%rax,%rax,1)
   506a0:	test   %ecx,%ecx
   506a2:	je     50719 <mibz2::mtf::mtf_avx2_windows+0x2b9>
   506a4:	mov    %ecx,%edx
   506a6:	lea    (%rdi,%rdx,1),%r12
   506aa:	cmp    %r11,%r12
   506ad:	ja     50b5d <mibz2::mtf::mtf_avx2_windows+0x6fd>
   506b3:	movzbl 0xa0(%rsp),%eax
   506bb:	add    %ecx,(%r10,%rax,4)
   506bf:	cmp    $0x40,%ecx
   506c2:	ja     506e6 <mibz2::mtf::mtf_avx2_windows+0x286>
   506c4:	lea    0x40(%rdi),%rcx
   506c8:	cmp    %r11,%rcx
   506cb:	ja     506e6 <mibz2::mtf::mtf_avx2_windows+0x286>
   506cd:	vmovd  %eax,%xmm0
   506d1:	vpbroadcastb %xmm0,%ymm0
   506d6:	vmovdqu %ymm0,0x0(%r13,%rdi,1)
   506dd:	vmovdqu %ymm0,0x20(%r13,%rdi,1)
   506e4:	jmp    50711 <mibz2::mtf::mtf_avx2_windows+0x2b1>
   506e6:	add    %r13,%rdi
   506e9:	movzbl %al,%esi
   506ec:	vzeroupper
   506ef:	call   *0x45b4b(%rip)        # 96240 <memset@GLIBC_2.2.5>
   506f5:	vmovdqa -0x482fd(%rip),%ymm7        # 8400 <GCC_except_table703+0x920>
   506fd:	mov    0x38(%rsp),%r9
   50702:	mov    0x50(%rsp),%r8
   50707:	mov    0x40(%rsp),%r11
   5070c:	mov    0x30(%rsp),%r10
   50711:	mov    $0x1,%eax
   50716:	mov    %r12,%rdi
   50719:	cmp    0x5c(%rsp),%r14w
   5071f:	je     50b42 <mibz2::mtf::mtf_avx2_windows+0x6e2>
   50725:	cmp    %r11,%rdi
   50728:	jae    50b5d <mibz2::mtf::mtf_avx2_windows+0x6fd>
   5072e:	lea    -0x1(%r14),%rcx
   50732:	mov    %rcx,%rdx
   50735:	and    $0xffffffffffffffe0,%rdx
   50739:	vmovdqa 0xa0(%rsp,%rdx,1),%ymm0
   50742:	and    $0x1f,%cl
   50745:	inc    %cl
   50747:	vmovd  %ecx,%xmm1
   5074b:	vpbroadcastb %xmm1,%ymm1
   50750:	vpcmpgtb %ymm7,%ymm1,%ymm1
   50754:	vpblendvb %ymm1,0x9f(%rsp,%rdx,1),%ymm0,%ymm0
   5075f:	movzbl 0x9f(%rsp,%r14,1),%ecx
   50768:	vmovdqa %ymm0,0xa0(%rsp,%rdx,1)
   50771:	test   %rdx,%rdx
   50774:	je     50630 <mibz2::mtf::mtf_avx2_windows+0x1d0>
   5077a:	add    $0xffffffffffffffdf,%r14
   5077e:	mov    %r14d,%esi
   50781:	not    %esi
   50783:	test   $0x60,%sil
   50787:	je     507b8 <mibz2::mtf::mtf_avx2_windows+0x358>
   50789:	mov    %r14d,%esi
   5078c:	shr    $0x5,%esi
   5078f:	inc    %esi
   50791:	and    $0x3,%esi
   50794:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   507a0:	vmovdqu 0x7f(%rsp,%rdx,1),%ymm0
   507a6:	vmovdqa %ymm0,0x80(%rsp,%rdx,1)
   507af:	add    $0xffffffffffffffe0,%rdx
   507b3:	dec    %rsi
   507b6:	jne    507a0 <mibz2::mtf::mtf_avx2_windows+0x340>
   507b8:	cmp    $0x60,%r14d
   507bc:	jb     50630 <mibz2::mtf::mtf_avx2_windows+0x1d0>
   507c2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   507d0:	vmovdqu 0x1f(%rsp,%rdx,1),%ymm0
   507d6:	vmovdqu 0x3f(%rsp,%rdx,1),%ymm1
   507dc:	vmovdqu 0x5f(%rsp,%rdx,1),%ymm2
   507e2:	vmovdqu 0x7f(%rsp,%rdx,1),%ymm3
   507e8:	vmovdqa %ymm3,0x80(%rsp,%rdx,1)
   507f1:	vmovdqa %ymm2,0x60(%rsp,%rdx,1)
   507f7:	vmovdqa %ymm1,0x40(%rsp,%rdx,1)
   507fd:	vmovdqa %ymm0,0x20(%rsp,%rdx,1)
   50803:	add    $0xffffffffffffff80,%rdx
   50807:	jne    507d0 <mibz2::mtf::mtf_avx2_windows+0x370>
   50809:	jmp    50630 <mibz2::mtf::mtf_avx2_windows+0x1d0>
   5080e:	add    $0x41,%r15
   50812:	cmp    %r8,%r15
   50815:	mov    %rbx,%r15
   50818:	ja     50b16 <mibz2::mtf::mtf_avx2_windows+0x6b6>
   5081e:	vmovdqu (%r9,%rbx,2),%ymm0
   50824:	vmovdqu 0x20(%r9,%rbx,2),%ymm1
   5082b:	vmovdqu 0x40(%r9,%rbx,2),%ymm2
   50832:	vmovdqu 0x60(%r9,%rbx,2),%ymm3
   50839:	vpbroadcastw -0x4838c(%rip),%ymm6        # 84b6 <GCC_except_table703+0x9d6>
   50842:	vpminuw %ymm6,%ymm0,%ymm4
   50847:	vpcmpeqw %ymm4,%ymm0,%ymm4
   5084b:	vpminuw %ymm6,%ymm1,%ymm5
   50850:	vpcmpeqw %ymm5,%ymm1,%ymm5
   50854:	vpacksswb %ymm5,%ymm4,%ymm4
   50858:	vpermq $0xd8,%ymm4,%ymm4
   5085e:	vpminuw %ymm6,%ymm2,%ymm5
   50863:	vpcmpeqw %ymm5,%ymm2,%ymm5
   50867:	vpminuw %ymm6,%ymm3,%ymm6
   5086c:	vpcmpeqw %ymm6,%ymm3,%ymm6
   50870:	vpacksswb %ymm6,%ymm5,%ymm5
   50874:	vpermq $0xd8,%ymm5,%ymm5
   5087a:	vpmovmskb %ymm4,%esi
   5087e:	vpmovmskb %ymm5,%edx
   50882:	shl    $0x20,%rdx
   50886:	or     %rsi,%rdx
   50889:	mov    %rdx,%rsi
   5088c:	shr    $1,%rsi
   5088f:	and    %rdx,%rsi
   50892:	mov    %rsi,%r8
   50895:	shr    $0x2,%r8
   50899:	and    %rsi,%r8
   5089c:	mov    %r8,%rsi
   5089f:	shr    $0x4,%rsi
   508a3:	and    %r8,%rsi
   508a6:	mov    %rsi,%r8
   508a9:	shr    $0x8,%r8
   508ad:	and    %rsi,%r8
   508b0:	mov    %r8,%r9
   508b3:	shr    $0x6,%r9
   508b7:	mov    $0x1,%sil
   508ba:	test   %r8,%r9
   508bd:	jne    508ed <mibz2::mtf::mtf_avx2_windows+0x48d>
   508bf:	vmovdqa 0x1e0(%rsp),%ymm6
   508c8:	vpcmpeqw %ymm6,%ymm3,%ymm4
   508cc:	vpcmpeqw %ymm6,%ymm2,%ymm5
   508d0:	vpor   %ymm4,%ymm5,%ymm4
   508d4:	vpcmpeqw %ymm6,%ymm1,%ymm5
   508d8:	vpcmpeqw %ymm6,%ymm0,%ymm6
   508dc:	vpor   %ymm5,%ymm6,%ymm5
   508e0:	vpor   %ymm4,%ymm5,%ymm4
   508e4:	vptest %ymm4,%ymm4
   508e9:	setne  %sil
   508ed:	test   %sil,%sil
   508f0:	mov    0x70(%rsp),%r12
   508f5:	jne    50b3b <mibz2::mtf::mtf_avx2_windows+0x6db>
   508fb:	mov    0x38(%rsp),%rsi
   50900:	lea    (%rsi,%rbx,2),%rsi
   50904:	vpxor  %xmm4,%xmm4,%xmm4
   50908:	vpcmpeqw %ymm4,%ymm2,%ymm2
   5090c:	vpcmpeqw %ymm4,%ymm3,%ymm3
   50910:	vpacksswb %ymm3,%ymm2,%ymm2
   50914:	vpermq $0xd8,%ymm2,%ymm2
   5091a:	vpcmpeqw %ymm4,%ymm0,%ymm0
   5091e:	vpcmpeqw %ymm4,%ymm1,%ymm1
   50922:	vpacksswb %ymm1,%ymm0,%ymm0
   50926:	vpermq $0xd8,%ymm0,%ymm0
   5092c:	vpmovmskb %ymm0,%r9d
   50930:	vpmovmskb %ymm2,%r8d
   50934:	shl    $0x20,%r8
   50938:	or     %r9,%r8
   5093b:	not    %r8
   5093e:	not    %rdx
   50941:	xor    %r9d,%r9d
   50944:	jmp    50962 <mibz2::mtf::mtf_avx2_windows+0x502>
   50946:	cs nopw 0x0(%rax,%rax,1)
   50950:	blsr   %rdx,%rdx
   50955:	mov    %cl,0xa0(%rsp)
   5095c:	je     50ac6 <mibz2::mtf::mtf_avx2_windows+0x666>
   50962:	mov    %rdi,%r10
   50965:	mov    %r9d,%r15d
   50968:	tzcnt  %rdx,%r11
   5096d:	shrx   %r15,%r8,%rdi
   50972:	lea    0x1(%r11),%r9d
   50976:	mov    %r9d,%r14d
   50979:	sub    %r15d,%r14d
   5097c:	bzhi   %r14,%rdi,%r14
   50981:	lea    (%r14,%r10,1),%rdi
   50985:	cmp    %r12,%rdi
   50988:	ja     50af6 <mibz2::mtf::mtf_avx2_windows+0x696>
   5098e:	vmovd  %ecx,%xmm0
   50992:	vpbroadcastb %xmm0,%ymm0
   50997:	vmovdqu %ymm0,0x0(%r13,%r10,1)
   5099e:	vmovdqu %ymm0,0x20(%r13,%r10,1)
   509a5:	cmp    $0x41,%r14
   509a9:	jb     509cf <mibz2::mtf::mtf_avx2_windows+0x56f>
   509ab:	add    %r13,%r10
   509ae:	mov    $0x40,%r15d
   509b4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   509c0:	vmovdqu %ymm0,(%r10,%r15,1)
   509c6:	add    $0x20,%r15
   509ca:	cmp    %r14,%r15
   509cd:	jb     509c0 <mibz2::mtf::mtf_avx2_windows+0x560>
   509cf:	movzbl %cl,%ecx
   509d2:	mov    0x30(%rsp),%r10
   509d7:	add    %r14d,(%r10,%rcx,4)
   509db:	movzwl (%rsi,%r11,2),%r11d
   509e0:	lea    -0x1(%r11),%rcx
   509e4:	mov    %rcx,%r10
   509e7:	and    $0xffffffffffffffe0,%r10
   509eb:	vmovdqa 0xa0(%rsp,%r10,1),%ymm0
   509f5:	and    $0x1f,%cl
   509f8:	inc    %cl
   509fa:	vmovd  %ecx,%xmm1
   509fe:	vpbroadcastb %xmm1,%ymm1
   50a03:	vpcmpgtb %ymm7,%ymm1,%ymm1
   50a07:	vpblendvb %ymm1,0x9f(%rsp,%r10,1),%ymm0,%ymm0
   50a12:	movzbl 0x9f(%rsp,%r11,1),%ecx
   50a1b:	vmovdqa %ymm0,0xa0(%rsp,%r10,1)
   50a25:	test   %r10,%r10
   50a28:	je     50950 <mibz2::mtf::mtf_avx2_windows+0x4f0>
   50a2e:	add    $0xffffffffffffffdf,%r11
   50a32:	mov    %r11d,%r14d
   50a35:	not    %r14d
   50a38:	test   $0x60,%r14b
   50a3c:	je     50a6a <mibz2::mtf::mtf_avx2_windows+0x60a>
   50a3e:	mov    %r11d,%r14d
   50a41:	shr    $0x5,%r14d
   50a45:	inc    %r14d
   50a48:	and    $0x3,%r14d
   50a4c:	nopl   0x0(%rax)
   50a50:	vmovdqu 0x7f(%rsp,%r10,1),%ymm0
   50a57:	vmovdqa %ymm0,0x80(%rsp,%r10,1)
   50a61:	add    $0xffffffffffffffe0,%r10
   50a65:	dec    %r14
   50a68:	jne    50a50 <mibz2::mtf::mtf_avx2_windows+0x5f0>
   50a6a:	cmp    $0x60,%r11d
   50a6e:	jb     50950 <mibz2::mtf::mtf_avx2_windows+0x4f0>
   50a74:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   50a80:	vmovdqu 0x1f(%rsp,%r10,1),%ymm0
   50a87:	vmovdqu 0x3f(%rsp,%r10,1),%ymm1
   50a8e:	vmovdqu 0x5f(%rsp,%r10,1),%ymm2
   50a95:	vmovdqu 0x7f(%rsp,%r10,1),%ymm3
   50a9c:	vmovdqa %ymm3,0x80(%rsp,%r10,1)
   50aa6:	vmovdqa %ymm2,0x60(%rsp,%r10,1)
   50aad:	vmovdqa %ymm1,0x40(%rsp,%r10,1)
   50ab4:	vmovdqa %ymm0,0x20(%rsp,%r10,1)
   50abb:	add    $0xffffffffffffff80,%r10
   50abf:	jne    50a80 <mibz2::mtf::mtf_avx2_windows+0x620>
   50ac1:	jmp    50950 <mibz2::mtf::mtf_avx2_windows+0x4f0>
   50ac6:	mov    %r9d,%edx
   50ac9:	lea    (%rbx,%rdx,1),%r15
   50acd:	add    %rbx,%rdx
   50ad0:	add    $0x40,%rdx
   50ad4:	mov    %r15,%rbx
   50ad7:	mov    0x50(%rsp),%r8
   50adc:	cmp    %r8,%rdx
   50adf:	mov    0x30(%rsp),%r10
   50ae4:	mov    0x40(%rsp),%r11
   50ae9:	mov    0x38(%rsp),%r9
   50aee:	jbe    5081e <mibz2::mtf::mtf_avx2_windows+0x3be>
   50af4:	jmp    50b16 <mibz2::mtf::mtf_avx2_windows+0x6b6>
   50af6:	mov    %r15d,%edx
   50af9:	add    %rdx,%rbx
   50afc:	mov    %rbx,%r15
   50aff:	mov    %r10,%rdi
   50b02:	mov    0x30(%rsp),%r10
   50b07:	mov    0x40(%rsp),%r11
   50b0c:	mov    0x50(%rsp),%r8
   50b11:	mov    0x38(%rsp),%r9
   50b16:	movzbl %cl,%ecx
   50b19:	incl   (%r10,%rcx,4)
   50b1d:	mov    %cl,0x0(%r13,%rdi,1)
   50b22:	inc    %rdi
   50b25:	movl   $0x0,0x4c(%rsp)
   50b2d:	mov    %rdi,%r12
   50b30:	cmp    %r8,%r15
   50b33:	jb     50620 <mibz2::mtf::mtf_avx2_windows+0x1c0>
   50b39:	jmp    50b42 <mibz2::mtf::mtf_avx2_windows+0x6e2>
   50b3b:	mov    %rbx,%r15
   50b3e:	jmp    50b0c <mibz2::mtf::mtf_avx2_windows+0x6ac>
   50b40:	xor    %edi,%edi
   50b42:	mov    0x60(%rsp),%rax
   50b47:	mov    %rdi,0x10(%rax)
   50b4b:	mov    0x68(%rsp),%rax
   50b50:	mov    %r12,0x8(%rax)
   50b54:	mov    $0xffffffffffffffff,%rcx
   50b5b:	jmp    50b7c <mibz2::mtf::mtf_avx2_windows+0x71c>
   50b5d:	movabs $0x800000000000000d,%rcx
   50b67:	jmp    50b6e <mibz2::mtf::mtf_avx2_windows+0x70e>
   50b69:	mov    0x78(%rsp),%rcx
   50b6e:	mov    0x60(%rsp),%rax
   50b73:	mov    %rdi,0x10(%rax)
   50b77:	mov    0x68(%rsp),%rax
   50b7c:	mov    %rcx,(%rax)
   50b7f:	lea    -0x28(%rbp),%rsp
   50b83:	pop    %rbx
   50b84:	pop    %r12
   50b86:	pop    %r13
   50b88:	pop    %r14
   50b8a:	pop    %r15
   50b8c:	pop    %rbp
   50b8d:	vzeroupper
   50b90:	ret
   50b91:	mov    $0x1,%ecx
   50b96:	mov    $0x1,%r8d
   50b9c:	mov    %rbx,%rdi
   50b9f:	xor    %esi,%esi
   50ba1:	mov    0x40(%rsp),%rdx
   50ba6:	call   358d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   50bab:	mov    0x30(%rsp),%rcx
   50bb0:	jmp    504a5 <mibz2::mtf::mtf_avx2_windows+0x45>
   50bb5:	int3
   50bb6:	int3
   50bb7:	int3
   50bb8:	int3
   50bb9:	int3
   50bba:	int3
   50bbb:	int3
   50bbc:	int3
   50bbd:	int3
   50bbe:	int3
   50bbf:	int3

0000000000050bc0 <mibz2::mtf::mtf_avx2_symbols>:
   50bc0:	push   %rbp
   50bc1:	mov    %rsp,%rbp
   50bc4:	push   %r15
   50bc6:	push   %r14
   50bc8:	push   %r13
   50bca:	push   %r12
   50bcc:	push   %rbx
   50bcd:	and    $0xffffffffffffffc0,%rsp
   50bd1:	sub    $0x1c0,%rsp
   50bd8:	mov    %rcx,%r15
   50bdb:	mov    %rdx,%rbx
   50bde:	mov    %rsi,%r13
   50be1:	mov    %rdi,%r14
   50be4:	mov    0x118(%rsi),%rax
   50beb:	movq   $0x0,0x10(%rdx)
   50bf3:	mov    %rax,0x20(%rsp)
   50bf8:	cmp    (%rdx),%rax
   50bfb:	ja     50f2e <mibz2::mtf::mtf_avx2_symbols+0x36e>
   50c01:	xor    %r12d,%r12d
   50c04:	mov    $0x400,%edx
   50c09:	mov    %r15,%rdi
   50c0c:	xor    %esi,%esi
   50c0e:	call   *0x4562c(%rip)        # 96240 <memset@GLIBC_2.2.5>
   50c14:	vxorps %xmm0,%xmm0,%xmm0
   50c18:	vmovaps %ymm0,0x160(%rsp)
   50c21:	vmovaps %ymm0,0x140(%rsp)
   50c2a:	vmovaps %ymm0,0x120(%rsp)
   50c33:	vmovaps %ymm0,0x100(%rsp)
   50c3c:	vmovaps %ymm0,0xe0(%rsp)
   50c45:	vmovaps %ymm0,0xc0(%rsp)
   50c4e:	vmovaps %ymm0,0xa0(%rsp)
   50c57:	vmovaps %ymm0,0x80(%rsp)
   50c60:	vmovaps %ymm0,0x60(%rsp)
   50c66:	vmovaps %ymm0,0x40(%rsp)
   50c6c:	vmovups 0xf8(%r13),%ymm0
   50c75:	vmovups %ymm0,0x140(%rsp)
   50c7e:	vmovups 0xd8(%r13),%ymm0
   50c87:	vmovaps %ymm0,0x120(%rsp)
   50c90:	vmovups 0xb8(%r13),%ymm0
   50c99:	vmovups %ymm0,0x100(%rsp)
   50ca2:	vmovups 0x98(%r13),%ymm0
   50cab:	vmovaps %ymm0,0xe0(%rsp)
   50cb4:	vmovdqu 0x18(%r13),%ymm0
   50cba:	vmovdqu 0x38(%r13),%ymm1
   50cc0:	vmovdqu 0x58(%r13),%ymm2
   50cc6:	vmovups 0x78(%r13),%ymm3
   50ccc:	vmovups %ymm3,0xc0(%rsp)
   50cd5:	vmovdqa %ymm2,0xa0(%rsp)
   50cde:	vmovdqu %ymm1,0x80(%rsp)
   50ce7:	vmovdqa %ymm0,0x60(%rsp)
   50ced:	mov    0x10(%r13),%rax
   50cf1:	test   %rax,%rax
   50cf4:	mov    %rbx,0x28(%rsp)
   50cf9:	je     50ef3 <mibz2::mtf::mtf_avx2_symbols+0x333>
   50cff:	mov    %r15,0x30(%rsp)
   50d04:	lea    0x60(%rsp),%r8
   50d09:	mov    0x8(%rbx),%r9
   50d0d:	mov    0x8(%r13),%rbx
   50d11:	lea    (%rbx,%rax,2),%r15
   50d15:	movzwl 0x128(%r13),%r10d
   50d1d:	mov    $0x1,%r13d
   50d23:	lea    0x61(%rsp),%r11
   50d28:	xor    %edi,%edi
   50d2a:	vmovdqa -0x48932(%rip),%ymm2        # 8400 <GCC_except_table703+0x920>
   50d32:	mov    %r14,0x18(%rsp)
   50d37:	xor    %eax,%eax
   50d39:	nopl   0x0(%rax)
   50d40:	movzwl (%rbx),%r14d
   50d44:	add    $0x2,%rbx
   50d48:	cmp    $0x2,%r14d
   50d4c:	jae    50d80 <mibz2::mtf::mtf_avx2_symbols+0x1c0>
   50d4e:	cmp    $0x1fffff,%r13d
   50d55:	ja     50ed1 <mibz2::mtf::mtf_avx2_symbols+0x311>
   50d5b:	inc    %r14d
   50d5e:	imul   %r13d,%r14d
   50d62:	add    %r14d,%eax
   50d65:	add    %r13d,%r13d
   50d68:	cmp    %r15,%rbx
   50d6b:	jne    50d40 <mibz2::mtf::mtf_avx2_symbols+0x180>
   50d6d:	jmp    50eeb <mibz2::mtf::mtf_avx2_symbols+0x32b>
   50d72:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   50d80:	test   %eax,%eax
   50d82:	je     50dd9 <mibz2::mtf::mtf_avx2_symbols+0x219>
   50d84:	mov    %eax,%edx
   50d86:	lea    (%rdi,%rdx,1),%r12
   50d8a:	cmp    0x20(%rsp),%r12
   50d8f:	ja     50f1f <mibz2::mtf::mtf_avx2_symbols+0x35f>
   50d95:	movzbl 0x60(%rsp),%ecx
   50d9a:	mov    0x30(%rsp),%rsi
   50d9f:	add    %eax,(%rsi,%rcx,4)
   50da2:	cmp    $0x40,%eax
   50da5:	ja     50de7 <mibz2::mtf::mtf_avx2_symbols+0x227>
   50da7:	lea    0x40(%rdi),%rax
   50dab:	cmp    0x20(%rsp),%rax
   50db0:	ja     50de7 <mibz2::mtf::mtf_avx2_symbols+0x227>
   50db2:	vmovd  %ecx,%xmm0
   50db6:	vpbroadcastb %xmm0,%ymm0
   50dbb:	vmovdqu %ymm0,(%r9,%rdi,1)
   50dc1:	vmovdqu %ymm0,0x20(%r9,%rdi,1)
   50dc8:	mov    $0x1,%r13d
   50dce:	cmp    %r10w,%r14w
   50dd2:	jne    50e2a <mibz2::mtf::mtf_avx2_symbols+0x26a>
   50dd4:	jmp    50eee <mibz2::mtf::mtf_avx2_symbols+0x32e>
   50dd9:	mov    %rdi,%r12
   50ddc:	cmp    %r10w,%r14w
   50de0:	jne    50e2a <mibz2::mtf::mtf_avx2_symbols+0x26a>
   50de2:	jmp    50eee <mibz2::mtf::mtf_avx2_symbols+0x32e>
   50de7:	add    %r9,%rdi
   50dea:	movzbl %cl,%esi
   50ded:	mov    %r9,%r13
   50df0:	mov    %r10w,0x16(%rsp)
   50df6:	vzeroupper
   50df9:	call   *0x45441(%rip)        # 96240 <memset@GLIBC_2.2.5>
   50dff:	vmovdqa -0x48a07(%rip),%ymm2        # 8400 <GCC_except_table703+0x920>
   50e07:	lea    0x61(%rsp),%r11
   50e0c:	movzwl 0x16(%rsp),%r10d
   50e12:	mov    %r13,%r9
   50e15:	lea    0x60(%rsp),%r8
   50e1a:	mov    $0x1,%r13d
   50e20:	cmp    %r10w,%r14w
   50e24:	je     50eee <mibz2::mtf::mtf_avx2_symbols+0x32e>
   50e2a:	cmp    0x20(%rsp),%r12
   50e2f:	jae    50f1c <mibz2::mtf::mtf_avx2_symbols+0x35c>
   50e35:	mov    %r14d,%eax
   50e38:	dec    %r14d
   50e3b:	movzwl %r14w,%edx
   50e3f:	movzbl 0x60(%rsp,%rdx,1),%r14d
   50e45:	cmp    $0x21,%ax
   50e49:	jae    50e70 <mibz2::mtf::mtf_avx2_symbols+0x2b0>
   50e4b:	vmovdqa 0x60(%rsp),%ymm0
   50e51:	inc    %dl
   50e53:	vmovd  %edx,%xmm1
   50e57:	vpbroadcastb %xmm1,%ymm1
   50e5c:	vpcmpgtb %ymm2,%ymm1,%ymm1
   50e60:	vpblendvb %ymm1,0x5f(%rsp),%ymm0,%ymm0
   50e68:	vmovdqa %ymm0,0x60(%rsp)
   50e6e:	jmp    50ea7 <mibz2::mtf::mtf_avx2_symbols+0x2e7>
   50e70:	mov    %r11,%rdi
   50e73:	mov    %r8,%rsi
   50e76:	mov    %r9,0x38(%rsp)
   50e7b:	mov    %r10w,0x16(%rsp)
   50e81:	vzeroupper
   50e84:	call   *0x45376(%rip)        # 96200 <memmove@GLIBC_2.2.5>
   50e8a:	vmovdqa -0x48a92(%rip),%ymm2        # 8400 <GCC_except_table703+0x920>
   50e92:	lea    0x61(%rsp),%r11
   50e97:	movzwl 0x16(%rsp),%r10d
   50e9d:	mov    0x38(%rsp),%r9
   50ea2:	lea    0x60(%rsp),%r8
   50ea7:	movzbl %r14b,%eax
   50eab:	mov    %al,0x60(%rsp)
   50eaf:	mov    0x30(%rsp),%rcx
   50eb4:	incl   (%rcx,%rax,4)
   50eb7:	mov    %al,(%r9,%r12,1)
   50ebb:	inc    %r12
   50ebe:	mov    %r12,%rdi
   50ec1:	cmp    %r15,%rbx
   50ec4:	mov    0x18(%rsp),%r14
   50ec9:	jne    50d37 <mibz2::mtf::mtf_avx2_symbols+0x177>
   50ecf:	jmp    50ef3 <mibz2::mtf::mtf_avx2_symbols+0x333>
   50ed1:	movabs $0x800000000000000c,%rax
   50edb:	mov    0x28(%rsp),%rcx
   50ee0:	mov    %rdi,0x10(%rcx)
   50ee4:	mov    0x18(%rsp),%r14
   50ee9:	jmp    50f07 <mibz2::mtf::mtf_avx2_symbols+0x347>
   50eeb:	mov    %rdi,%r12
   50eee:	mov    0x18(%rsp),%r14
   50ef3:	mov    0x28(%rsp),%rax
   50ef8:	mov    %r12,0x10(%rax)
   50efc:	mov    %r12,0x8(%r14)
   50f00:	mov    $0xffffffffffffffff,%rax
   50f07:	mov    %rax,(%r14)
   50f0a:	lea    -0x28(%rbp),%rsp
   50f0e:	pop    %rbx
   50f0f:	pop    %r12
   50f11:	pop    %r13
   50f13:	pop    %r14
   50f15:	pop    %r15
   50f17:	pop    %rbp
   50f18:	vzeroupper
   50f1b:	ret
   50f1c:	mov    %r12,%rdi
   50f1f:	movabs $0x800000000000000c,%rax
   50f29:	inc    %rax
   50f2c:	jmp    50edb <mibz2::mtf::mtf_avx2_symbols+0x31b>
   50f2e:	mov    $0x1,%ecx
   50f33:	mov    $0x1,%r8d
   50f39:	mov    %rbx,%rdi
   50f3c:	xor    %esi,%esi
   50f3e:	mov    0x20(%rsp),%rdx
   50f43:	call   358d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   50f48:	jmp    50c01 <mibz2::mtf::mtf_avx2_symbols+0x41>
   50f4d:	int3
   50f4e:	int3
   50f4f:	int3

