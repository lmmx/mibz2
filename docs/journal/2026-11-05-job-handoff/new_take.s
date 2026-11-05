   24910:	push   %rbp
   24911:	push   %r15
   24913:	push   %r14
   24915:	push   %r13
   24917:	push   %r12
   24919:	push   %rbx
   2491a:	push   %rax
   2491b:	mov    %rsi,%r14
   2491e:	mov    %rdi,%r15
   24921:	lea    0x10(%rsi),%rbx
   24925:	mov    $0x1,%ecx
   2492a:	xor    %eax,%eax
   2492c:	lock cmpxchg %ecx,0x10(%rsi)
   24931:	jne    24a2a <handoff::new_take+0x11a>
   24937:	mov    0x4c3b2(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   2493e:	mov    (%rax),%rax
   24941:	shl    $1,%rax
   24944:	test   %rax,%rax
   24947:	jne    24a38 <handoff::new_take+0x128>
   2494d:	movl   $0x0,0x4(%rsp)
   24955:	movzbl 0x14(%r14),%eax
   2495a:	mov    0x30(%r14),%rax
   2495e:	test   %rax,%rax
   24961:	jne    249a5 <handoff::new_take+0x95>
   24963:	lea    0x50(%r14),%r12
   24967:	xor    %r13d,%r13d
   2496a:	mov    0x4c447(%rip),%rbp        # 70db8 <_DYNAMIC+0x330>
   24971:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   24980:	cmpb   $0x1,0x48(%r14)
   24985:	je     249ef <handoff::new_take+0xdf>
   24987:	incq   0x38(%r14)
   2498b:	mov    %r12,%rdi
   2498e:	mov    %rbx,%rsi
   24991:	call   *%rbp
   24993:	movzbl 0x14(%r14),%eax
   24998:	decq   0x38(%r14)
   2499c:	mov    0x30(%r14),%rax
   249a0:	test   %rax,%rax
   249a3:	je     24980 <handoff::new_take+0x70>
   249a5:	mov    0x18(%r14),%rcx
   249a9:	mov    0x28(%r14),%rdx
   249ad:	lea    0x1(%rdx),%rsi
   249b1:	xor    %edi,%edi
   249b3:	cmp    %rcx,%rsi
   249b6:	cmovae %rcx,%rdi
   249ba:	neg    %rdi
   249bd:	lea    (%rdx,%rdi,1),%rcx
   249c1:	inc    %rcx
   249c4:	mov    %rcx,0x28(%r14)
   249c8:	dec    %rax
   249cb:	mov    %rax,0x30(%r14)
   249cf:	imul   $0x138,%rdx,%rsi
   249d6:	add    0x20(%r14),%rsi
   249da:	lea    0x8(%r15),%rdi
   249de:	mov    $0x138,%edx
   249e3:	call   *0x4c2ef(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   249e9:	mov    $0x1,%r13d
   249ef:	mov    %r13,(%r15)
   249f2:	cmpb   $0x0,0x4(%rsp)
   249f7:	jne    24a12 <handoff::new_take+0x102>
   249f9:	movabs $0x7fffffffffffffff,%rax
   24a03:	mov    0x4c2e6(%rip),%rcx        # 70cf0 <_DYNAMIC+0x268>
   24a0a:	mov    (%rcx),%rcx
   24a0d:	test   %rax,%rcx
   24a10:	jne    24a72 <handoff::new_take+0x162>
   24a12:	xor    %eax,%eax
   24a14:	xchg   %eax,(%rbx)
   24a16:	cmp    $0x2,%eax
   24a19:	je     24a5b <handoff::new_take+0x14b>
   24a1b:	add    $0x8,%rsp
   24a1f:	pop    %rbx
   24a20:	pop    %r12
   24a22:	pop    %r13
   24a24:	pop    %r14
   24a26:	pop    %r15
   24a28:	pop    %rbp
   24a29:	ret
   24a2a:	mov    %rbx,%rdi
   24a2d:	call   *0x4c2c5(%rip)        # 70cf8 <_DYNAMIC+0x270>
   24a33:	jmp    24937 <handoff::new_take+0x27>
   24a38:	call   *0x4c2c2(%rip)        # 70d00 <_DYNAMIC+0x278>
   24a3e:	xor    $0x1,%al
   24a40:	mov    %eax,0x4(%rsp)
   24a44:	movzbl 0x14(%r14),%eax
   24a49:	mov    0x30(%r14),%rax
   24a4d:	test   %rax,%rax
   24a50:	jne    249a5 <handoff::new_take+0x95>
   24a56:	jmp    24963 <handoff::new_take+0x53>
   24a5b:	mov    %rbx,%rdi
   24a5e:	add    $0x8,%rsp
   24a62:	pop    %rbx
   24a63:	pop    %r12
   24a65:	pop    %r13
   24a67:	pop    %r14
   24a69:	pop    %r15
   24a6b:	pop    %rbp
   24a6c:	jmp    *0x4c296(%rip)        # 70d08 <_DYNAMIC+0x280>
   24a72:	call   *0x4c288(%rip)        # 70d00 <_DYNAMIC+0x278>
   24a78:	test   %al,%al
   24a7a:	jne    24a12 <handoff::new_take+0x102>
   24a7c:	movb   $0x1,0x14(%r14)
   24a81:	jmp    24a12 <handoff::new_take+0x102>
   24a83:	mov    %rax,%r14
   24a86:	movq   $0x0,(%r15)
   24a8d:	movzbl 0x4(%rsp),%esi
   24a92:	mov    %rbx,%rdi
   24a95:	call   1c960 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<handoff::Queued<(u64, [u64; 38])>>>>
   24a9a:	mov    %r14,%rdi
   24a9d:	call   6cec0 <_Unwind_Resume@plt>
   24aa2:	call   *0x4c270(%rip)        # 70d18 <_DYNAMIC+0x290>
   24aa8:	int3
   24aa9:	int3
   24aaa:	int3
   24aab:	int3
   24aac:	int3
   24aad:	int3
   24aae:	int3
   24aaf:	int3

