# c30walk::ctor::canonicalise
# size 0x305

/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/c30walk-e003a32:     file format elf64-x86-64


Disassembly of section .text:

0000000000051560 <c30walk::ctor::canonicalise>:
   51560:	push   %r15
   51562:	push   %r14
   51564:	push   %r13
   51566:	push   %r12
   51568:	push   %rbx
   51569:	sub    $0x10,%rsp
   5156d:	xor    %r11d,%r11d
   51570:	test   %rcx,%rcx
   51573:	je     51609 <c30walk::ctor::canonicalise+0xa9>
   51579:	lea    (%rdx,%rcx,8),%rcx
   5157d:	xor    %r10d,%r10d
   51580:	jmp    5159c <c30walk::ctor::canonicalise+0x3c>
   51582:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   51590:	add    $0x8,%rdx
   51594:	inc    %r10
   51597:	cmp    %rcx,%rdx
   5159a:	je     51606 <c30walk::ctor::canonicalise+0xa6>
   5159c:	mov    (%rdx),%rbx
   5159f:	test   %rbx,%rbx
   515a2:	je     51590 <c30walk::ctor::canonicalise+0x30>
   515a4:	mov    %r10,%r14
   515a7:	shl    $0xc,%r14
   515ab:	add    %rsi,%r14
   515ae:	mov    %r11d,%r15d
   515b1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   515c0:	tzcnt  %rbx,%r12
   515c5:	shl    $0x6,%r12d
   515c9:	mov    0x3c(%r14,%r12,1),%eax
   515ce:	and    $0xfffff,%eax
   515d3:	cmp    %rax,%r9
   515d6:	jbe    51841 <c30walk::ctor::canonicalise+0x2e1>
   515dc:	add    %r14,%r12
   515df:	lea    -0x1(%rbx),%r13
   515e3:	shl    $0x7,%eax
   515e6:	lea    0x1(%r15),%r11d
   515ea:	mov    %r15d,0x7c(%r8,%rax,1)
   515ef:	mov    %r15d,%eax
   515f2:	or     $0x10000000,%eax
   515f7:	mov    %eax,0x3c(%r12)
   515fc:	mov    %r11d,%r15d
   515ff:	and    %r13,%rbx
   51602:	jne    515c0 <c30walk::ctor::canonicalise+0x60>
   51604:	jmp    51590 <c30walk::ctor::canonicalise+0x30>
   51606:	mov    %r11d,%r11d
   51609:	mov    %r11,(%rsp)
   5160d:	mov    %r9,0x8(%rsp)
   51612:	cmp    %r11,%r9
   51615:	jne    51821 <c30walk::ctor::canonicalise+0x2c1>
   5161b:	lea    -0x80(%r8),%r10
   5161f:	xor    %r11d,%r11d
   51622:	xor    %ecx,%ecx
   51624:	xor    %edx,%edx
   51626:	xor    %esi,%esi
   51628:	mov    %r11,%rax
   5162b:	shl    $0x7,%rax
   5162f:	add    %r10,%rax
   51632:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   51640:	cmp    %r9,%r11
   51643:	jae    5174c <c30walk::ctor::canonicalise+0x1ec>
   51649:	mov    %r11,%rbx
   5164c:	inc    %r11
   5164f:	mov    0xfc(%rax),%r14d
   51656:	sub    $0xffffffffffffff80,%rax
   5165a:	cmp    %r14,%rbx
   5165d:	je     51640 <c30walk::ctor::canonicalise+0xe0>
   5165f:	movaps (%rax),%xmm0
   51662:	movaps 0x10(%rax),%xmm1
   51666:	movaps 0x20(%rax),%xmm2
   5166a:	movaps 0x30(%rax),%xmm3
   5166e:	movaps 0x40(%rax),%xmm4
   51672:	movaps 0x50(%rax),%xmm5
   51676:	movaps 0x60(%rax),%xmm6
   5167a:	movdqa 0x70(%rax),%xmm7
   5167f:	xor    %r14d,%r14d
   51682:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   51690:	pshufd $0xee,%xmm7,%xmm8
   51696:	movq   %xmm8,%rax
   5169b:	shr    $0x20,%rax
   5169f:	cmp    %rax,%r9
   516a2:	jbe    51853 <c30walk::ctor::canonicalise+0x2f3>
   516a8:	mov    %rax,%r15
   516ab:	shl    $0x7,%r15
   516af:	movaps (%r8,%r15,1),%xmm8
   516b4:	movaps 0x10(%r8,%r15,1),%xmm9
   516ba:	movaps 0x20(%r8,%r15,1),%xmm10
   516c0:	movaps 0x30(%r8,%r15,1),%xmm11
   516c6:	movaps %xmm0,(%r8,%r15,1)
   516cb:	movaps %xmm1,0x10(%r8,%r15,1)
   516d1:	movaps %xmm2,0x20(%r8,%r15,1)
   516d7:	movaps %xmm3,0x30(%r8,%r15,1)
   516dd:	movaps 0x40(%r8,%r15,1),%xmm12
   516e3:	movaps %xmm4,0x40(%r8,%r15,1)
   516e9:	movaps 0x50(%r8,%r15,1),%xmm13
   516ef:	movaps %xmm5,0x50(%r8,%r15,1)
   516f5:	movaps 0x60(%r8,%r15,1),%xmm14
   516fb:	movaps %xmm6,0x60(%r8,%r15,1)
   51701:	movaps 0x70(%r8,%r15,1),%xmm15
   51707:	movdqa %xmm7,0x70(%r8,%r15,1)
   5170e:	inc    %r14
   51711:	movaps %xmm8,%xmm0
   51715:	movaps %xmm9,%xmm1
   51719:	movaps %xmm10,%xmm2
   5171d:	movaps %xmm11,%xmm3
   51721:	movaps %xmm12,%xmm4
   51725:	movaps %xmm13,%xmm5
   51729:	movaps %xmm14,%xmm6
   5172d:	movaps %xmm15,%xmm7
   51731:	cmp    %rax,%rbx
   51734:	jne    51690 <c30walk::ctor::canonicalise+0x130>
   5173a:	add    %r14,%rcx
   5173d:	inc    %rdx
   51740:	cmp    %rsi,%r14
   51743:	cmova  %r14,%rsi
   51747:	jmp    51628 <c30walk::ctor::canonicalise+0xc8>
   5174c:	test   %r9,%r9
   5174f:	je     51808 <c30walk::ctor::canonicalise+0x2a8>
   51755:	shl    $0x7,%r9
   51759:	lea    -0x80(%r9),%r10
   5175d:	mov    %r10d,%r11d
   51760:	not    %r11d
   51763:	mov    %r8,%rax
   51766:	test   $0x380,%r11d
   5176d:	je     517a9 <c30walk::ctor::canonicalise+0x249>
   5176f:	mov    %r10d,%r11d
   51772:	shr    $0x7,%r11d
   51776:	inc    %r11d
   51779:	and    $0x7,%r11d
   5177d:	neg    %r11
   51780:	mov    %r8,%rax
   51783:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   51790:	movl   $0x0,0x7c(%rax)
   51797:	sub    $0xffffffffffffff80,%rax
   5179b:	inc    %r11
   5179e:	jne    51790 <c30walk::ctor::canonicalise+0x230>
   517a0:	cmp    $0x380,%r10
   517a7:	jb     51808 <c30walk::ctor::canonicalise+0x2a8>
   517a9:	add    %r9,%r8
   517ac:	nopl   0x0(%rax)
   517b0:	movl   $0x0,0x7c(%rax)
   517b7:	movl   $0x0,0xfc(%rax)
   517c1:	movl   $0x0,0x17c(%rax)
   517cb:	movl   $0x0,0x1fc(%rax)
   517d5:	movl   $0x0,0x27c(%rax)
   517df:	movl   $0x0,0x2fc(%rax)
   517e9:	movl   $0x0,0x37c(%rax)
   517f3:	movl   $0x0,0x3fc(%rax)
   517fd:	add    $0x400,%rax
   51803:	cmp    %r8,%rax
   51806:	jne    517b0 <c30walk::ctor::canonicalise+0x250>
   51808:	mov    %rcx,(%rdi)
   5180b:	mov    %rdx,0x8(%rdi)
   5180f:	mov    %rsi,0x10(%rdi)
   51813:	add    $0x10,%rsp
   51817:	pop    %rbx
   51818:	pop    %r12
   5181a:	pop    %r13
   5181c:	pop    %r14
   5181e:	pop    %r15
   51820:	ret
   51821:	lea    -0x423bf(%rip),%rdx        # f469 <GCC_except_table733+0x641d>
   51828:	lea    0x65971(%rip),%r8        # b71a0 <__frame_dummy_init_array_entry+0x1850>
   5182f:	mov    %rsp,%rdi
   51832:	lea    0x8(%rsp),%rsi
   51837:	mov    $0x33,%ecx
   5183c:	call   62cd2 <core::panicking::assert_failed::<usize, usize>>
   51841:	lea    0x65988(%rip),%rdx        # b71d0 <__frame_dummy_init_array_entry+0x1880>
   51848:	mov    %rax,%rdi
   5184b:	mov    %r9,%rsi
   5184e:	call   6292b <core::panicking::panic_bounds_check>
   51853:	lea    0x6595e(%rip),%rdx        # b71b8 <__frame_dummy_init_array_entry+0x1868>
   5185a:	mov    %rax,%rdi
   5185d:	mov    %r9,%rsi
   51860:	call   6292b <core::panicking::panic_bounds_check>
