# master mtf_avx2: one index symbol (nn < 32, no run before it), the straight path
# 51107..51299 of the listing, the taken jumps dropped
	xorl	%eax, %eax
	movzwl	(%rbx), %r14d
	addq	$2, %rbx
	cmpl	$2, %r14d
	testl	%eax, %eax
	movq	%rdi, %r12
	cmpw	%r10w, %r14w
	cmpq	32(%rsp), %r12
	movl	%r14d, %eax
	decl	%r14d
	movzwl	%r14w, %edx
	movzbl	96(%rsp,%rdx), %r14d
	cmpw	$33, %ax
	vmovdqa	96(%rsp), %ymm0
	incb	%dl
	vmovd	%edx, %xmm1
	vpbroadcastb	%xmm1, %ymm1
	vpcmpgtb	%ymm2, %ymm1, %ymm1
	vpblendvb	%ymm1, 95(%rsp), %ymm0, %ymm0
	vmovdqa	%ymm0, 96(%rsp)
	movzbl	%r14b, %eax
	movb	%al, 96(%rsp)
	movq	48(%rsp), %rcx
	incl	(%rcx,%rax,4)
	movb	%al, (%r9,%r12)
	incq	%r12
	movq	%r12, %rdi
	cmpq	%r15, %rbx
	movq	24(%rsp), %r14
