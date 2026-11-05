# research/mtf-front-reg mtf_avx2 (phases build): one index symbol, nn < 32, no run before
# it: 31775..31a6c of the listing, taken jumps dropped; the four table loads written
# apart from their vpshufb (as folded, llvm-mca would put the load's latency on the
# register operand's path, which the hardware does not)
	xorl	%r8d, %r8d
	movzwl	(%rbx), %r13d
	addq	$2, %rbx
	cmpl	$2, %r13d
	testl	%r8d, %r8d
	movq	%rdi, %r12
	cmpw	%r10w, %r13w
	cmpq	16(%rsp), %r12
	movl	%r13d, %ecx
	decl	%r13d
	movzwl	%r13w, %edx
	cmpw	$33, %cx
	shll	$6, %edx
	vmovdqa	(%rdx,%r11), %xmm10
	vmovdqa	16(%rdx,%r11), %xmm11
	vmovdqa	48(%rdx,%r11), %xmm12
	vmovdqa	32(%rdx,%r11), %xmm13
	vpshufb	%xmm10, %xmm8, %xmm0
	vpshufb	%xmm11, %xmm7, %xmm1
	vpshufb	%xmm12, %xmm8, %xmm2
	vpor	%xmm0, %xmm1, %xmm8
	vpshufb	%xmm13, %xmm7, %xmm1
	vpor	%xmm1, %xmm2, %xmm7
	vmovd	%xmm8, %ecx
	movzbl	%cl, %edx
	movq	32(%rsp), %rsi
	incl	(%rsi,%rdx,4)
	movb	%dl, (%r9,%r12)
	incq	%r12
	movq	%r12, %rdi
	cmpq	%r15, %rbx
