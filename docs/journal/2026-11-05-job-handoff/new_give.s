   24760:	push   %rbp
   24761:	push   %r15
   24763:	push   %r14
   24765:	push   %r13
   24767:	push   %r12
   24769:	push   %rbx
   2476a:	push   %rax
   2476b:	mov    %rsi,%r15
   2476e:	mov    %rdi,%r14
   24771:	lea    0x10(%rdi),%rbx
   24775:	mov    $0x1,%ecx
   2477a:	xor    %eax,%eax
   2477c:	lock cmpxchg %ecx,0x10(%rdi)
   24781:	jne    24880 <handoff::new_give+0x120>
   24787:	mov    0x4c562(%rip),%r13        # 70cf0 <_DYNAMIC+0x268>
   2478e:	mov    0x0(%r13),%rax
   24792:	shl    $1,%rax
   24795:	test   %rax,%rax
   24798:	jne    2488e <handoff::new_give+0x12e>
   2479e:	xor    %ebp,%ebp
   247a0:	movabs $0x7fffffffffffffff,%rcx
   247aa:	movzbl 0x14(%r14),%eax
   247af:	mov    0x40(%r14),%r12
   247b3:	test   %r12,%r12
   247b6:	je     24854 <handoff::new_give+0xf4>
   247bc:	mov    0x18(%r14),%rcx
   247c0:	mov    0x30(%r14),%rax
   247c4:	cmp    %rcx,%rax
   247c7:	jne    247da <handoff::new_give+0x7a>
   247c9:	lea    0x18(%r14),%rdi
   247cd:	call   27d30 <<alloc::collections::vec_deque::VecDeque<(u64, [u64; 38])>>::grow>
   247d2:	mov    0x18(%r14),%rcx
   247d6:	mov    0x30(%r14),%rax
   247da:	lea    0x1(%rax),%rdx
   247de:	mov    %rdx,0x30(%r14)
   247e2:	add    0x28(%r14),%rax
   247e6:	xor    %r13d,%r13d
   247e9:	cmp    %rcx,%rax
   247ec:	mov    $0x0,%edx
   247f1:	cmovae %rcx,%rdx
   247f5:	sub    %rdx,%rax
   247f8:	imul   $0x138,%rax,%rdi
   247ff:	add    0x20(%r14),%rdi
   24803:	mov    $0x138,%edx
   24808:	mov    %r15,%rsi
   2480b:	call   *0x4c4c7(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   24811:	mov    0x38(%r14),%r15
   24815:	test   %bpl,%bpl
   24818:	jne    24837 <handoff::new_give+0xd7>
   2481a:	mov    0x4c4cf(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   24821:	mov    (%rax),%rax
   24824:	movabs $0x7fffffffffffffff,%rcx
   2482e:	test   %rcx,%rax
   24831:	jne    248ba <handoff::new_give+0x15a>
   24837:	xchg   %r13d,(%rbx)
   2483a:	cmp    $0x2,%r13d
   2483e:	je     2489f <handoff::new_give+0x13f>
   24840:	test   %r15,%r15
   24843:	je     2486b <handoff::new_give+0x10b>
   24845:	add    $0x50,%r14
   24849:	mov    %r14,%rdi
   2484c:	call   *0x4c62e(%rip)        # 70e80 <_DYNAMIC+0x3f8>
   24852:	jmp    2486b <handoff::new_give+0x10b>
   24854:	test   %bpl,%bpl
   24857:	jne    24862 <handoff::new_give+0x102>
   24859:	mov    0x0(%r13),%rax
   2485d:	test   %rcx,%rax
   24860:	jne    248d2 <handoff::new_give+0x172>
   24862:	xor    %eax,%eax
   24864:	xchg   %eax,(%rbx)
   24866:	cmp    $0x2,%eax
   24869:	je     248af <handoff::new_give+0x14f>
   2486b:	test   %r12,%r12
   2486e:	setne  %al
   24871:	add    $0x8,%rsp
   24875:	pop    %rbx
   24876:	pop    %r12
   24878:	pop    %r13
   2487a:	pop    %r14
   2487c:	pop    %r15
   2487e:	pop    %rbp
   2487f:	ret
   24880:	mov    %rbx,%rdi
   24883:	call   *0x4c46f(%rip)        # 70cf8 <_DYNAMIC+0x270>
   24889:	jmp    24787 <handoff::new_give+0x27>
   2488e:	call   *0x4c46c(%rip)        # 70d00 <_DYNAMIC+0x278>
   24894:	mov    %eax,%ebp
   24896:	xor    $0x1,%bpl
   2489a:	jmp    247a0 <handoff::new_give+0x40>
   2489f:	mov    %rbx,%rdi
   248a2:	call   *0x4c460(%rip)        # 70d08 <_DYNAMIC+0x280>
   248a8:	test   %r15,%r15
   248ab:	jne    24845 <handoff::new_give+0xe5>
   248ad:	jmp    2486b <handoff::new_give+0x10b>
   248af:	mov    %rbx,%rdi
   248b2:	call   *0x4c450(%rip)        # 70d08 <_DYNAMIC+0x280>
   248b8:	jmp    2486b <handoff::new_give+0x10b>
   248ba:	call   *0x4c440(%rip)        # 70d00 <_DYNAMIC+0x278>
   248c0:	test   %al,%al
   248c2:	jne    24837 <handoff::new_give+0xd7>
   248c8:	movb   $0x1,0x14(%r14)
   248cd:	jmp    24837 <handoff::new_give+0xd7>
   248d2:	call   *0x4c428(%rip)        # 70d00 <_DYNAMIC+0x278>
   248d8:	test   %al,%al
   248da:	jne    24862 <handoff::new_give+0x102>
   248dc:	movb   $0x1,0x14(%r14)
   248e1:	jmp    24862 <handoff::new_give+0x102>
   248e6:	mov    %rax,%r14
   248e9:	movzbl %bpl,%esi
   248ed:	mov    %rbx,%rdi
   248f0:	call   1c960 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<handoff::Queued<(u64, [u64; 38])>>>>
   248f5:	mov    %r14,%rdi
   248f8:	call   6cec0 <_Unwind_Resume@plt>
   248fd:	call   *0x4c415(%rip)        # 70d18 <_DYNAMIC+0x290>
   24903:	int3
   24904:	int3
   24905:	int3
   24906:	int3
   24907:	int3
   24908:	int3
   24909:	int3
   2490a:	int3
   2490b:	int3
   2490c:	int3
   2490d:	int3
   2490e:	int3
   2490f:	int3

