	.att_syntax
	.file	"probe.e538d19652c9b163-cgu.0"
	.section	.text.unlikely._RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe,"ax",@progbits
	.prefalign	4, .Lfunc_end0, nop
	.type	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe,@function
_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe:
	.cfi_startproc
	pushq	%r14
	.cfi_def_cfa_offset 16
	pushq	%rbx
	.cfi_def_cfa_offset 24
	subq	$24, %rsp
	.cfi_def_cfa_offset 48
	.cfi_offset %rbx, -24
	.cfi_offset %r14, -16
	addq	%rdx, %rsi
	jb	.LBB0_1
	movq	%rdi, %rbx
	movq	(%rdi), %rax
	leaq	(%rax,%rax), %rcx
	cmpq	%rcx, %rsi
	cmovaq	%rsi, %rcx
	cmpq	$5, %rcx
	movl	$4, %r14d
	cmovaeq	%rcx, %r14
	movq	8(%rdi), %rdx
	movq	%rsp, %rdi
	movq	%rax, %rsi
	movq	%r14, %rcx
	callq	_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe
	cmpl	$1, (%rsp)
	je	.LBB0_3
	movq	8(%rsp), %rax
	movq	%rax, 8(%rbx)
	movq	%r14, (%rbx)
	addq	$24, %rsp
	.cfi_def_cfa_offset 24
	popq	%rbx
	.cfi_def_cfa_offset 16
	popq	%r14
	.cfi_def_cfa_offset 8
	retq
.LBB0_1:
	.cfi_def_cfa_offset 48
	xorl	%edi, %edi
	callq	*_RNvNtCsgFX3jaOk65H_5alloc7raw_vec12handle_error@GOTPCREL(%rip)
.LBB0_3:
	movq	8(%rsp), %rdi
	movq	16(%rsp), %rsi
	callq	*_RNvNtCsgFX3jaOk65H_5alloc7raw_vec12handle_error@GOTPCREL(%rip)
.Lfunc_end0:
	.size	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe, .Lfunc_end0-_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	.cfi_endproc

	.section	.text.unlikely._RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe,"ax",@progbits
	.globl	_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe
	.prefalign	4, .Lfunc_end1, nop
	.type	_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe,@function
_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe:
	.cfi_startproc
	pushq	%r14
	.cfi_def_cfa_offset 16
	pushq	%rbx
	.cfi_def_cfa_offset 24
	subq	$24, %rsp
	.cfi_def_cfa_offset 48
	.cfi_offset %rbx, -24
	.cfi_offset %r14, -16
	movq	%rdi, %rbx
	movq	(%rdi), %rsi
	leaq	(%rsi,%rsi), %rax
	cmpq	$5, %rax
	movl	$4, %r14d
	cmovaeq	%rax, %r14
	movq	8(%rdi), %rdx
	movq	%rsp, %rdi
	movq	%r14, %rcx
	callq	_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe
	cmpl	$1, (%rsp)
	je	.LBB1_2
	movq	8(%rsp), %rax
	movq	%rax, 8(%rbx)
	movq	%r14, (%rbx)
	addq	$24, %rsp
	.cfi_def_cfa_offset 24
	popq	%rbx
	.cfi_def_cfa_offset 16
	popq	%r14
	.cfi_def_cfa_offset 8
	retq
.LBB1_2:
	.cfi_def_cfa_offset 48
	movq	8(%rsp), %rdi
	movq	16(%rsp), %rsi
	callq	*_RNvNtCsgFX3jaOk65H_5alloc7raw_vec12handle_error@GOTPCREL(%rip)
.Lfunc_end1:
	.size	_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe, .Lfunc_end1-_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe
	.cfi_endproc

	.section	.text.unlikely._RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe,"ax",@progbits
	.prefalign	4, .Lfunc_end2, nop
	.type	_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe,@function
_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe:
	.cfi_startproc
	pushq	%r15
	.cfi_def_cfa_offset 16
	pushq	%r14
	.cfi_def_cfa_offset 24
	pushq	%rbx
	.cfi_def_cfa_offset 32
	.cfi_offset %rbx, -32
	.cfi_offset %r14, -24
	.cfi_offset %r15, -16
	movq	%rdi, %rbx
	leaq	(,%rcx,4), %r14
	shrq	$62, %rcx
	setne	%al
	movabsq	$9223372036854775804, %rcx
	cmpq	%rcx, %r14
	seta	%cl
	movl	$1, %r15d
	orb	%al, %cl
	je	.LBB2_2
	movl	$8, %eax
	xorl	%r14d, %r14d
	jmp	.LBB2_10
.LBB2_2:
	testq	%rsi, %rsi
	je	.LBB2_4
	shlq	$2, %rsi
	movq	%rdx, %rdi
	movl	$4, %edx
	movq	%r14, %rcx
	callq	*_RNvCs5fGHjzWXRav_7___rustc14___rust_realloc@GOTPCREL(%rip)
	testq	%rax, %rax
	jne	.LBB2_6
	jmp	.LBB2_9
.LBB2_4:
	testq	%r14, %r14
	je	.LBB2_5
	callq	*_RNvCs5fGHjzWXRav_7___rustc35___rust_no_alloc_shim_is_unstable_v2@GOTPCREL(%rip)
	movl	$4, %esi
	movq	%r14, %rdi
	callq	*_RNvCs5fGHjzWXRav_7___rustc12___rust_alloc@GOTPCREL(%rip)
	testq	%rax, %rax
	jne	.LBB2_6
.LBB2_9:
	movq	$4, 8(%rbx)
	movl	$16, %eax
	jmp	.LBB2_10
.LBB2_5:
	movl	$4, %eax
.LBB2_6:
	movq	%rax, 8(%rbx)
	movl	$16, %eax
	xorl	%r15d, %r15d
.LBB2_10:
	movq	%r14, (%rbx,%rax)
	movq	%r15, (%rbx)
	popq	%rbx
	.cfi_def_cfa_offset 24
	popq	%r14
	.cfi_def_cfa_offset 16
	popq	%r15
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end2:
	.size	_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe, .Lfunc_end2-_RNvMs5_NtCsgFX3jaOk65H_5alloc7raw_vecNtB5_11RawVecInner11finish_growCsjG8RT04T8ad_5probe
	.cfi_endproc

	.section	.rodata.cst16,"aM",@progbits,16
	.p2align	4, 0x0
.LCPI3_0:
	.long	0
	.long	1
	.long	2
	.long	3
.LCPI3_1:
	.long	4
	.long	4
	.long	4
	.long	4
.LCPI3_2:
	.long	8
	.long	8
	.long	8
	.long	8
	.section	.text.extend_rows,"ax",@progbits
	.globl	extend_rows
	.prefalign	4, .Lfunc_end3, nop
	.type	extend_rows,@function
extend_rows:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	pushq	%r15
	.cfi_def_cfa_offset 24
	pushq	%r14
	.cfi_def_cfa_offset 32
	pushq	%rbx
	.cfi_def_cfa_offset 40
	pushq	%rax
	.cfi_def_cfa_offset 48
	.cfi_offset %rbx, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	.cfi_offset %rbp, -16
	leal	(%rdx,%rsi), %ebp
	xorl	%eax, %eax
	movl	%ebp, %ecx
	subl	%esi, %ecx
	cmovbl	%eax, %ecx
	movq	(%rdi), %r8
	movq	16(%rdi), %rax
	subq	%rax, %r8
	cmpq	%rcx, %r8
	jb	.LBB3_1
	cmpl	%ebp, %esi
	jae	.LBB3_8
.LBB3_3:
	movq	8(%rdi), %rcx
	decl	%edx
	cmpl	$7, %edx
	jb	.LBB3_7
	incq	%rdx
	movq	%rdx, %r8
	andq	$-8, %r8
	movd	%esi, %xmm0
	addl	%r8d, %esi
	pshufd	$0, %xmm0, %xmm0
	paddd	.LCPI3_0(%rip), %xmm0
	leaq	(%rcx,%rax,4), %r9
	addq	$16, %r9
	addq	%r8, %rax
	xorl	%r10d, %r10d
	movdqa	.LCPI3_1(%rip), %xmm1
	movdqa	.LCPI3_2(%rip), %xmm2
	.p2align	4
.LBB3_5:
	movdqa	%xmm0, %xmm3
	paddd	%xmm1, %xmm3
	movdqu	%xmm0, -16(%r9,%r10,4)
	movdqu	%xmm3, (%r9,%r10,4)
	addq	$8, %r10
	paddd	%xmm2, %xmm0
	cmpq	%r10, %r8
	jne	.LBB3_5
	cmpq	%r8, %rdx
	je	.LBB3_8
	.p2align	4
.LBB3_7:
	movl	%esi, (%rcx,%rax,4)
	incl	%esi
	incq	%rax
	cmpl	%esi, %ebp
	jne	.LBB3_7
.LBB3_8:
	movq	%rax, 16(%rdi)
	addq	$8, %rsp
	.cfi_def_cfa_offset 40
	popq	%rbx
	.cfi_def_cfa_offset 32
	popq	%r14
	.cfi_def_cfa_offset 24
	popq	%r15
	.cfi_def_cfa_offset 16
	popq	%rbp
	.cfi_def_cfa_offset 8
	retq
.LBB3_1:
	.cfi_def_cfa_offset 48
	movq	%rdi, %rbx
	movq	%rsi, %r14
	movq	%rax, %rsi
	movq	%rdx, %r15
	movq	%rcx, %rdx
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movq	%r15, %rdx
	movq	%r14, %rsi
	movq	%rbx, %rdi
	movq	16(%rbx), %rax
	cmpl	%ebp, %esi
	jb	.LBB3_3
	jmp	.LBB3_8
.Lfunc_end3:
	.size	extend_rows, .Lfunc_end3-extend_rows
	.cfi_endproc

	.section	.rodata.cst16,"aM",@progbits,16
	.p2align	4, 0x0
.LCPI4_0:
	.long	0
	.long	1
	.long	2
	.long	3
.LCPI4_1:
	.long	4
	.long	4
	.long	4
	.long	4
.LCPI4_2:
	.long	8
	.long	8
	.long	8
	.long	8
	.section	.text.first_four,"ax",@progbits
	.globl	first_four
	.prefalign	4, .Lfunc_end4, nop
	.type	first_four,@function
first_four:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	pushq	%r15
	.cfi_def_cfa_offset 24
	pushq	%r14
	.cfi_def_cfa_offset 32
	pushq	%r12
	.cfi_def_cfa_offset 40
	pushq	%rbx
	.cfi_def_cfa_offset 48
	subq	$16, %rsp
	.cfi_def_cfa_offset 64
	.cfi_offset %rbx, -48
	.cfi_offset %r12, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	.cfi_offset %rbp, -16
	leal	-1(%rdx), %r14d
	cmpl	$3, %r14d
	ja	.LBB4_10
	leaq	.LJTI4_0(%rip), %rax
	movslq	(%rax,%r14,4), %rcx
	addq	%rax, %rcx
	jmpq	*%rcx
.LBB4_2:
	movq	16(%rdi), %r14
	cmpq	(%rdi), %r14
	je	.LBB4_19
.LBB4_3:
	movq	8(%rdi), %rax
	movl	%esi, (%rax,%r14,4)
	incq	%r14
	movq	%r14, 16(%rdi)
	jmp	.LBB4_18
.LBB4_4:
	leal	1(%rsi), %ebp
	leal	2(%rsi), %r14d
	movq	(%rdi), %rcx
	movq	16(%rdi), %rax
	subq	%rax, %rcx
	cmpq	$2, %rcx
	jbe	.LBB4_20
.LBB4_5:
	movq	8(%rdi), %rcx
	movl	%esi, (%rcx,%rax,4)
	movl	%ebp, 4(%rcx,%rax,4)
	movl	%r14d, 8(%rcx,%rax,4)
	addq	$3, %rax
	jmp	.LBB4_17
.LBB4_6:
	movd	%esi, %xmm0
	pshufd	$0, %xmm0, %xmm0
	paddd	.LCPI4_0(%rip), %xmm0
	movq	(%rdi), %rax
	movq	16(%rdi), %rsi
	subq	%rsi, %rax
	cmpq	$3, %rax
	jbe	.LBB4_21
.LBB4_7:
	movq	8(%rdi), %rax
	movdqu	%xmm0, (%rax,%rsi,4)
	addq	$4, %rsi
	movq	%rsi, 16(%rdi)
	jmp	.LBB4_18
.LBB4_8:
	leal	1(%rsi), %ebp
	movq	(%rdi), %rcx
	movq	16(%rdi), %rax
	subq	%rax, %rcx
	cmpq	$1, %rcx
	jbe	.LBB4_22
.LBB4_9:
	movq	8(%rdi), %rcx
	movl	%esi, (%rcx,%rax,4)
	movl	%ebp, 4(%rcx,%rax,4)
	addq	$2, %rax
	jmp	.LBB4_17
.LBB4_10:
	addl	%esi, %edx
	xorl	%eax, %eax
	movl	%edx, %ecx
	subl	%esi, %ecx
	cmovbl	%eax, %ecx
	movq	(%rdi), %r8
	movq	16(%rdi), %rax
	subq	%rax, %r8
	cmpq	%rcx, %r8
	jb	.LBB4_23
	cmpl	%edx, %esi
	jae	.LBB4_17
.LBB4_12:
	movq	8(%rdi), %rcx
	cmpl	$7, %r14d
	jb	.LBB4_16
	incq	%r14
	movq	%r14, %r8
	andq	$-8, %r8
	movd	%esi, %xmm0
	addl	%r8d, %esi
	pshufd	$0, %xmm0, %xmm0
	paddd	.LCPI4_0(%rip), %xmm0
	leaq	(%rcx,%rax,4), %r9
	addq	$16, %r9
	addq	%r8, %rax
	xorl	%r10d, %r10d
	movdqa	.LCPI4_1(%rip), %xmm1
	movdqa	.LCPI4_2(%rip), %xmm2
	.p2align	4
.LBB4_14:
	movdqa	%xmm0, %xmm3
	paddd	%xmm1, %xmm3
	movdqu	%xmm0, -16(%r9,%r10,4)
	movdqu	%xmm3, (%r9,%r10,4)
	addq	$8, %r10
	paddd	%xmm2, %xmm0
	cmpq	%r10, %r8
	jne	.LBB4_14
	cmpq	%r8, %r14
	je	.LBB4_17
	.p2align	4
.LBB4_16:
	movl	%esi, (%rcx,%rax,4)
	incl	%esi
	incq	%rax
	cmpl	%esi, %edx
	jne	.LBB4_16
.LBB4_17:
	movq	%rax, 16(%rdi)
.LBB4_18:
	addq	$16, %rsp
	.cfi_def_cfa_offset 48
	popq	%rbx
	.cfi_def_cfa_offset 40
	popq	%r12
	.cfi_def_cfa_offset 32
	popq	%r14
	.cfi_def_cfa_offset 24
	popq	%r15
	.cfi_def_cfa_offset 16
	popq	%rbp
	.cfi_def_cfa_offset 8
	retq
.LBB4_19:
	.cfi_def_cfa_offset 64
	movq	%rdi, %rbx
	movq	%rsi, %r15
	callq	*_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe@GOTPCREL(%rip)
	movq	%r15, %rsi
	movq	%rbx, %rdi
	jmp	.LBB4_3
.LBB4_20:
	movl	$3, %edx
	movq	%rdi, %rbx
	movq	%rsi, %r15
	movq	%rax, %rsi
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movq	%r15, %rsi
	movq	%rbx, %rdi
	movq	16(%rbx), %rax
	jmp	.LBB4_5
.LBB4_21:
	movl	$4, %edx
	movq	%rdi, %rbx
	movdqa	%xmm0, (%rsp)
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movdqa	(%rsp), %xmm0
	movq	%rbx, %rdi
	movq	16(%rbx), %rsi
	jmp	.LBB4_7
.LBB4_22:
	movl	$2, %edx
	movq	%rdi, %rbx
	movq	%rsi, %r14
	movq	%rax, %rsi
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movq	%r14, %rsi
	movq	%rbx, %rdi
	movq	16(%rbx), %rax
	jmp	.LBB4_9
.LBB4_23:
	movq	%rdi, %rbx
	movq	%rsi, %r15
	movq	%rax, %rsi
	movq	%rdx, %r12
	movq	%rcx, %rdx
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movq	%r12, %rdx
	movq	%r15, %rsi
	movq	%rbx, %rdi
	movq	16(%rbx), %rax
	cmpl	%edx, %esi
	jb	.LBB4_12
	jmp	.LBB4_17
.Lfunc_end4:
	.size	first_four, .Lfunc_end4-first_four
	.cfi_endproc
	.section	.rodata.first_four,"a",@progbits
	.p2align	2, 0x0
.LJTI4_0:
	.long	.LBB4_2-.LJTI4_0
	.long	.LBB4_8-.LJTI4_0
	.long	.LBB4_4-.LJTI4_0
	.long	.LBB4_6-.LJTI4_0

	.section	.rodata.cst16,"aM",@progbits,16
	.p2align	4, 0x0
.LCPI5_0:
	.long	0
	.long	1
	.long	2
	.long	3
.LCPI5_1:
	.long	4
	.long	4
	.long	4
	.long	4
.LCPI5_2:
	.long	8
	.long	8
	.long	8
	.long	8
	.section	.text.first_single,"ax",@progbits
	.globl	first_single
	.prefalign	4, .Lfunc_end5, nop
	.type	first_single,@function
first_single:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	pushq	%r15
	.cfi_def_cfa_offset 24
	pushq	%r14
	.cfi_def_cfa_offset 32
	pushq	%rbx
	.cfi_def_cfa_offset 40
	pushq	%rax
	.cfi_def_cfa_offset 48
	.cfi_offset %rbx, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	.cfi_offset %rbp, -16
	cmpl	$1, %edx
	jne	.LBB5_4
	movq	16(%rdi), %r14
	cmpq	(%rdi), %r14
	je	.LBB5_2
.LBB5_3:
	movq	8(%rdi), %rax
	movl	%esi, (%rax,%r14,4)
	incq	%r14
	movq	%r14, 16(%rdi)
	jmp	.LBB5_13
.LBB5_4:
	leal	(%rdx,%rsi), %ebp
	xorl	%eax, %eax
	movl	%ebp, %ecx
	subl	%esi, %ecx
	cmovbl	%eax, %ecx
	movq	(%rdi), %r8
	movq	16(%rdi), %rax
	subq	%rax, %r8
	cmpq	%rcx, %r8
	jb	.LBB5_5
	cmpl	%ebp, %esi
	jae	.LBB5_12
.LBB5_7:
	movq	8(%rdi), %rcx
	decl	%edx
	cmpl	$7, %edx
	jb	.LBB5_11
	incq	%rdx
	movq	%rdx, %r8
	andq	$-8, %r8
	movd	%esi, %xmm0
	addl	%r8d, %esi
	pshufd	$0, %xmm0, %xmm0
	paddd	.LCPI5_0(%rip), %xmm0
	leaq	(%rcx,%rax,4), %r9
	addq	$16, %r9
	addq	%r8, %rax
	xorl	%r10d, %r10d
	movdqa	.LCPI5_1(%rip), %xmm1
	movdqa	.LCPI5_2(%rip), %xmm2
	.p2align	4
.LBB5_9:
	movdqa	%xmm0, %xmm3
	paddd	%xmm1, %xmm3
	movdqu	%xmm0, -16(%r9,%r10,4)
	movdqu	%xmm3, (%r9,%r10,4)
	addq	$8, %r10
	paddd	%xmm2, %xmm0
	cmpq	%r10, %r8
	jne	.LBB5_9
	cmpq	%r8, %rdx
	je	.LBB5_12
	.p2align	4
.LBB5_11:
	movl	%esi, (%rcx,%rax,4)
	incl	%esi
	incq	%rax
	cmpl	%esi, %ebp
	jne	.LBB5_11
.LBB5_12:
	movq	%rax, 16(%rdi)
.LBB5_13:
	addq	$8, %rsp
	.cfi_def_cfa_offset 40
	popq	%rbx
	.cfi_def_cfa_offset 32
	popq	%r14
	.cfi_def_cfa_offset 24
	popq	%r15
	.cfi_def_cfa_offset 16
	popq	%rbp
	.cfi_def_cfa_offset 8
	retq
.LBB5_2:
	.cfi_def_cfa_offset 48
	movq	%rdi, %rbx
	movq	%rsi, %r15
	callq	*_RNvMs4_NtCsgFX3jaOk65H_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsjG8RT04T8ad_5probe@GOTPCREL(%rip)
	movq	%r15, %rsi
	movq	%rbx, %rdi
	jmp	.LBB5_3
.LBB5_5:
	movq	%rdi, %rbx
	movq	%rsi, %r14
	movq	%rax, %rsi
	movq	%rdx, %r15
	movq	%rcx, %rdx
	callq	_RINvNvMs2_NtCsgFX3jaOk65H_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsjG8RT04T8ad_5probe
	movq	%r15, %rdx
	movq	%r14, %rsi
	movq	%rbx, %rdi
	movq	16(%rbx), %rax
	cmpl	%ebp, %esi
	jb	.LBB5_7
	jmp	.LBB5_12
.Lfunc_end5:
	.size	first_single, .Lfunc_end5-first_single
	.cfi_endproc

	.ident	"rustc version 1.99.1 (58cf8f9c3 2026-10-13)"
	.section	".note.GNU-stack","",@progbits
