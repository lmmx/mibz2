
b16/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

000000000003a5d0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build>:
   3a5d0:	push   rbp
   3a5d1:	push   r15
   3a5d3:	push   r14
   3a5d5:	push   r13
   3a5d7:	push   r12
   3a5d9:	push   rbx
   3a5da:	sub    rsp,0x808
   3a5e1:	mov    r15,rdx
   3a5e4:	mov    rbx,rsi
   3a5e7:	mov    r14,rdi
   3a5ea:	mov    QWORD PTR [rcx+0x10],0x0
   3a5f2:	cmp    rsi,0xffffff
   3a5f9:	ja     3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3a5ff:	pxor   xmm0,xmm0
   3a603:	xor    eax,eax
   3a605:	pxor   xmm2,xmm2
   3a609:	pxor   xmm1,xmm1
   3a60d:	nop    DWORD PTR [rax]
   3a610:	movq   xmm3,QWORD PTR [r15+rax*4]
   3a616:	movq   xmm4,QWORD PTR [r15+rax*4+0x8]
   3a61d:	punpckldq xmm3,xmm0
   3a621:	paddq  xmm3,xmm2
   3a625:	punpckldq xmm4,xmm0
   3a629:	paddq  xmm4,xmm1
   3a62d:	movq   xmm2,QWORD PTR [r15+rax*4+0x10]
   3a634:	movq   xmm1,QWORD PTR [r15+rax*4+0x18]
   3a63b:	punpckldq xmm2,xmm0
   3a63f:	paddq  xmm2,xmm3
   3a643:	punpckldq xmm1,xmm0
   3a647:	paddq  xmm1,xmm4
   3a64b:	add    rax,0x8
   3a64f:	cmp    rax,0x100
   3a655:	jne    3a610 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x40>
   3a657:	paddq  xmm1,xmm2
   3a65b:	pshufd xmm0,xmm1,0xee
   3a660:	paddq  xmm0,xmm1
   3a664:	movq   rax,xmm0
   3a669:	cmp    rax,rbx
   3a66c:	jne    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3a672:	mov    r12,rcx
   3a675:	lea    rdi,[rsp+0x8]
   3a67a:	xor    ebp,ebp
   3a67c:	mov    r13,QWORD PTR [rip+0x5b9ad]        # 96030 <memset@GLIBC_2.2.5>
   3a683:	mov    edx,0x400
   3a688:	xor    esi,esi
   3a68a:	call   r13
   3a68d:	lea    rdi,[rsp+0x408]
   3a695:	mov    edx,0x400
   3a69a:	xor    esi,esi
   3a69c:	call   r13
   3a69f:	mov    eax,0x1
   3a6a4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3a6b0:	mov    DWORD PTR [rsp+rax*4+0x404],ebp
   3a6b7:	add    ebp,DWORD PTR [r15+rax*4-0x4]
   3a6bc:	mov    DWORD PTR [rsp+rax*4+0x4],ebp
   3a6c0:	mov    DWORD PTR [rsp+rax*4+0x408],ebp
   3a6c7:	add    ebp,DWORD PTR [r15+rax*4]
   3a6cb:	mov    DWORD PTR [rsp+rax*4+0x8],ebp
   3a6cf:	add    rax,0x2
   3a6d3:	cmp    rax,0x101
   3a6d9:	jne    3a6b0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe0>
   3a6db:	mov    rcx,r12
   3a6de:	cmp    rbx,QWORD PTR [r12]
   3a6e2:	ja     3a836 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x266>
   3a6e8:	mov    rdx,QWORD PTR [rcx+0x8]
   3a6ec:	xor    r8d,r8d
   3a6ef:	cmp    rbx,0x10
   3a6f3:	jae    3a862 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x292>
   3a6f9:	cmp    r8,rbx
   3a6fc:	jae    3a757 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x187>
   3a6fe:	mov    eax,r8d
   3a701:	shl    eax,0x8
   3a704:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3a710:	movzx  esi,BYTE PTR [r14+r8*1]
   3a715:	lea    r9d,[rsi*4+0x0]
   3a71d:	mov    edi,DWORD PTR [rsp+r9*1+0x408]
   3a725:	cmp    edi,DWORD PTR [rsp+r9*1+0x8]
   3a72a:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3a72c:	cmp    rbx,rdi
   3a72f:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3a735:	movzx  r9d,BYTE PTR [r14+rdi*1]
   3a73a:	add    r9d,eax
   3a73d:	mov    DWORD PTR [rdx+rdi*4],r9d
   3a741:	inc    edi
   3a743:	mov    DWORD PTR [rsp+rsi*4+0x408],edi
   3a74a:	inc    r8
   3a74d:	add    eax,0x100
   3a752:	cmp    rbx,r8
   3a755:	jne    3a710 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x140>
   3a757:	lea    rdi,[rsp+0x408]
   3a75f:	lea    rsi,[rsp+0x8]
   3a764:	mov    edx,0x400
   3a769:	call   QWORD PTR [rip+0x5b959]        # 960c8 <bcmp@GLIBC_2.2.5>
   3a76f:	mov    rcx,r12
   3a772:	test   eax,eax
   3a774:	je     3a7a0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1d0>
   3a776:	mov    QWORD PTR [rcx+0x10],0x0
   3a77e:	cmp    rbx,QWORD PTR [rcx]
   3a781:	ja     3a7a9 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1d9>
   3a783:	mov    r13,QWORD PTR [rcx+0x8]
   3a787:	xor    r12d,r12d
   3a78a:	test   rbx,rbx
   3a78d:	je     3aead <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8dd>
   3a793:	cmp    rbx,0x8
   3a797:	jae    3a7d7 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x207>
   3a799:	xor    eax,eax
   3a79b:	jmp    3ae98 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8c8>
   3a7a0:	mov    QWORD PTR [rcx+0x10],rbx
   3a7a4:	jmp    3ae86 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8b6>
   3a7a9:	mov    r12,rcx
   3a7ac:	mov    ecx,0x4
   3a7b1:	mov    r8d,0x4
   3a7b7:	mov    rdi,r12
   3a7ba:	xor    esi,esi
   3a7bc:	mov    rdx,rbx
   3a7bf:	call   36150 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs9pn3potmX1d_5mibz2>
   3a7c4:	mov    rcx,r12
   3a7c7:	mov    r13,QWORD PTR [r12+0x8]
   3a7cc:	mov    r12,QWORD PTR [r12+0x10]
   3a7d1:	cmp    rbx,0x8
   3a7d5:	jb     3a799 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1c9>
   3a7d7:	movabs rax,0x7ffffffffffffff0
   3a7e1:	or     rax,0x8
   3a7e5:	and    rax,rbx
   3a7e8:	lea    rdx,[r12*4+0x10]
   3a7f0:	add    rdx,r13
   3a7f3:	add    r12,rax
   3a7f6:	xor    esi,esi
   3a7f8:	pxor   xmm0,xmm0
   3a7fc:	nop    DWORD PTR [rax+0x0]
   3a800:	movd   xmm1,DWORD PTR [r14+rsi*1]
   3a806:	movd   xmm2,DWORD PTR [r14+rsi*1+0x4]
   3a80d:	punpcklbw xmm1,xmm0
   3a811:	punpcklwd xmm1,xmm0
   3a815:	punpcklbw xmm2,xmm0
   3a819:	punpcklwd xmm2,xmm0
   3a81d:	movdqu XMMWORD PTR [rdx+rsi*4-0x10],xmm1
   3a823:	movdqu XMMWORD PTR [rdx+rsi*4],xmm2
   3a828:	add    rsi,0x8
   3a82c:	cmp    rax,rsi
   3a82f:	jne    3a800 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x230>
   3a831:	jmp    3aea8 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8d8>
   3a836:	mov    ecx,0x4
   3a83b:	mov    r8d,0x4
   3a841:	mov    rdi,r12
   3a844:	xor    esi,esi
   3a846:	mov    rdx,rbx
   3a849:	call   36150 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs9pn3potmX1d_5mibz2>
   3a84e:	mov    rcx,r12
   3a851:	mov    rdx,QWORD PTR [rcx+0x8]
   3a855:	xor    r8d,r8d
   3a858:	cmp    rbx,0x10
   3a85c:	jb     3a6f9 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x129>
   3a862:	mov    esi,0x8
   3a867:	pxor   xmm0,xmm0
   3a86b:	movdqa xmm1,XMMWORD PTR [rip+0xfffffffffffcd5ad]        # 7e20 <GCC_except_table701+0x370>
   3a873:	movdqa xmm2,XMMWORD PTR [rip+0xfffffffffffcd2c5]        # 7b40 <GCC_except_table701+0x90>
   3a87b:	movdqa xmm3,XMMWORD PTR [rip+0xfffffffffffcd2ad]        # 7b30 <GCC_except_table701+0x80>
   3a883:	movdqa xmm4,XMMWORD PTR [rip+0xfffffffffffcd535]        # 7dc0 <GCC_except_table701+0x310>
   3a88b:	jmp    3a8b2 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x2e2>
   3a88d:	nop    DWORD PTR [rax]
   3a890:	mov    DWORD PTR [rsp+rax*4+0x408],edi
   3a897:	lea    rax,[rsi+0x10]
   3a89b:	add    rsi,0x18
   3a89f:	add    r8d,0x1000
   3a8a6:	cmp    rsi,rbx
   3a8a9:	mov    rsi,rax
   3a8ac:	ja     3ae2c <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x85c>
   3a8b2:	lea    rax,[rsi-0x8]
   3a8b6:	cmp    rsi,rbx
   3a8b9:	ja     3ae5a <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x88a>
   3a8bf:	lea    rdi,[rsi+0x7]
   3a8c3:	cmp    rdi,rbx
   3a8c6:	jae    3ae41 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x871>
   3a8cc:	mov    r9,QWORD PTR [r14+rsi*1-0x8]
   3a8d1:	mov    rax,QWORD PTR [r14+rsi*1]
   3a8d5:	mov    rdi,r9
   3a8d8:	xor    rdi,rax
   3a8db:	mov    r10,r9
   3a8de:	rol    r10,0x8
   3a8e2:	xor    r10,r9
   3a8e5:	or     r10,rdi
   3a8e8:	jne    3a9a0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x3d0>
   3a8ee:	movzx  eax,r9b
   3a8f2:	lea    r9d,[rax*4+0x0]
   3a8fa:	mov    edi,DWORD PTR [rsp+r9*1+0x408]
   3a902:	lea    r10,[rdi+0x10]
   3a906:	mov    r9d,DWORD PTR [rsp+r9*1+0x8]
   3a90b:	cmp    r10,r9
   3a90e:	ja     3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3a914:	movdqu xmm6,XMMWORD PTR [r14+rdi*1]
   3a91a:	movdqa xmm5,xmm6
   3a91e:	punpckhbw xmm5,xmm0
   3a922:	movd   xmm7,r8d
   3a927:	pshufd xmm7,xmm7,0x0
   3a92c:	punpcklbw xmm6,xmm0
   3a930:	movdqa xmm8,xmm6
   3a935:	punpcklwd xmm8,xmm0
   3a93a:	por    xmm8,xmm7
   3a93f:	por    xmm8,xmm1
   3a944:	punpckhwd xmm6,xmm0
   3a948:	por    xmm6,xmm7
   3a94c:	por    xmm6,xmm2
   3a950:	movdqa xmm9,xmm5
   3a955:	punpcklwd xmm9,xmm0
   3a95a:	por    xmm9,xmm7
   3a95f:	por    xmm9,xmm3
   3a964:	punpckhwd xmm5,xmm0
   3a968:	por    xmm5,xmm7
   3a96c:	por    xmm5,xmm4
   3a970:	movdqu XMMWORD PTR [rdx+rdi*4],xmm8
   3a976:	movdqu XMMWORD PTR [rdx+rdi*4+0x10],xmm6
   3a97c:	movdqu XMMWORD PTR [rdx+rdi*4+0x20],xmm9
   3a983:	movdqu XMMWORD PTR [rdx+rdi*4+0x30],xmm5
   3a989:	add    edi,0x10
   3a98c:	jmp    3a890 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x2c0>
   3a991:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3a9a0:	movzx  r10d,r9b
   3a9a4:	lea    r11d,[r10*4+0x0]
   3a9ac:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3a9b4:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3a9b9:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3a9bf:	cmp    rbx,rdi
   3a9c2:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3a9c8:	mov    r11,r9
   3a9cb:	shr    r11,0x8
   3a9cf:	movzx  ebp,BYTE PTR [r14+rdi*1]
   3a9d4:	add    ebp,r8d
   3a9d7:	mov    DWORD PTR [rdx+rdi*4],ebp
   3a9da:	inc    edi
   3a9dc:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3a9e4:	movzx  r10d,r11b
   3a9e8:	lea    r11d,[r10*4+0x0]
   3a9f0:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3a9f8:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3a9fd:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3aa03:	cmp    rbx,rdi
   3aa06:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3aa0c:	mov    r11,r9
   3aa0f:	shr    r11,0x10
   3aa13:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3aa18:	lea    ebp,[r8+r13*1]
   3aa1c:	add    ebp,0x100
   3aa22:	mov    DWORD PTR [rdx+rdi*4],ebp
   3aa25:	inc    edi
   3aa27:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3aa2f:	movzx  r10d,r11b
   3aa33:	lea    r11d,[r10*4+0x0]
   3aa3b:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3aa43:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3aa48:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3aa4e:	cmp    rbx,rdi
   3aa51:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3aa57:	mov    r11,r9
   3aa5a:	shr    r11,0x18
   3aa5e:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3aa63:	lea    ebp,[r8+r13*1]
   3aa67:	add    ebp,0x200
   3aa6d:	mov    DWORD PTR [rdx+rdi*4],ebp
   3aa70:	inc    edi
   3aa72:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3aa7a:	movzx  r10d,r11b
   3aa7e:	lea    r11d,[r10*4+0x0]
   3aa86:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3aa8e:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3aa93:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3aa99:	cmp    rbx,rdi
   3aa9c:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3aaa2:	mov    r11,r9
   3aaa5:	shr    r11,0x20
   3aaa9:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3aaae:	lea    ebp,[r8+r13*1]
   3aab2:	add    ebp,0x300
   3aab8:	mov    DWORD PTR [rdx+rdi*4],ebp
   3aabb:	inc    edi
   3aabd:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3aac5:	movzx  r10d,r11b
   3aac9:	lea    r11d,[r10*4+0x0]
   3aad1:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3aad9:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3aade:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3aae4:	cmp    rbx,rdi
   3aae7:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3aaed:	mov    r11,r9
   3aaf0:	shr    r11,0x28
   3aaf4:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3aaf9:	lea    ebp,[r8+r13*1]
   3aafd:	add    ebp,0x400
   3ab03:	mov    DWORD PTR [rdx+rdi*4],ebp
   3ab06:	inc    edi
   3ab08:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3ab10:	movzx  r10d,r11b
   3ab14:	lea    r11d,[r10*4+0x0]
   3ab1c:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3ab24:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3ab29:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ab2f:	cmp    rbx,rdi
   3ab32:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ab38:	mov    r11,r9
   3ab3b:	shr    r11,0x30
   3ab3f:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3ab44:	lea    ebp,[r8+r13*1]
   3ab48:	add    ebp,0x500
   3ab4e:	mov    DWORD PTR [rdx+rdi*4],ebp
   3ab51:	inc    edi
   3ab53:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3ab5b:	movzx  r10d,r11b
   3ab5f:	lea    r11d,[r10*4+0x0]
   3ab67:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3ab6f:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3ab74:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ab7a:	cmp    rbx,rdi
   3ab7d:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ab83:	shr    r9,0x38
   3ab87:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3ab8c:	add    r11d,r8d
   3ab8f:	add    r11d,0x600
   3ab96:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ab9a:	inc    edi
   3ab9c:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3aba4:	lea    r10d,[r9*4+0x0]
   3abac:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3abb4:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3abb9:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3abbf:	cmp    rbx,rdi
   3abc2:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3abc8:	movzx  r10d,BYTE PTR [r14+rdi*1]
   3abcd:	add    r10d,r8d
   3abd0:	add    r10d,0x700
   3abd7:	mov    DWORD PTR [rdx+rdi*4],r10d
   3abdb:	inc    edi
   3abdd:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3abe5:	movzx  r9d,al
   3abe9:	lea    r10d,[r9*4+0x0]
   3abf1:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3abf9:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3abfe:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ac04:	cmp    rbx,rdi
   3ac07:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ac0d:	mov    r10,rax
   3ac10:	shr    r10,0x8
   3ac14:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3ac19:	add    r11d,r8d
   3ac1c:	add    r11d,0x800
   3ac23:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ac27:	inc    edi
   3ac29:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3ac31:	movzx  r9d,r10b
   3ac35:	lea    r10d,[r9*4+0x0]
   3ac3d:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3ac45:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3ac4a:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ac50:	cmp    rbx,rdi
   3ac53:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ac59:	mov    r10,rax
   3ac5c:	shr    r10,0x10
   3ac60:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3ac65:	add    r11d,r8d
   3ac68:	add    r11d,0x900
   3ac6f:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ac73:	inc    edi
   3ac75:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3ac7d:	movzx  r9d,r10b
   3ac81:	lea    r10d,[r9*4+0x0]
   3ac89:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3ac91:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3ac96:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ac9c:	cmp    rbx,rdi
   3ac9f:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3aca5:	mov    r10,rax
   3aca8:	shr    r10,0x18
   3acac:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3acb1:	add    r11d,r8d
   3acb4:	add    r11d,0xa00
   3acbb:	mov    DWORD PTR [rdx+rdi*4],r11d
   3acbf:	inc    edi
   3acc1:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3acc9:	movzx  r9d,r10b
   3accd:	lea    r10d,[r9*4+0x0]
   3acd5:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3acdd:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3ace2:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ace8:	cmp    rbx,rdi
   3aceb:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3acf1:	mov    r10,rax
   3acf4:	shr    r10,0x20
   3acf8:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3acfd:	add    r11d,r8d
   3ad00:	add    r11d,0xb00
   3ad07:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ad0b:	inc    edi
   3ad0d:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3ad15:	movzx  r9d,r10b
   3ad19:	lea    r10d,[r9*4+0x0]
   3ad21:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3ad29:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3ad2e:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ad34:	cmp    rbx,rdi
   3ad37:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ad3d:	mov    r10,rax
   3ad40:	shr    r10,0x28
   3ad44:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3ad49:	add    r11d,r8d
   3ad4c:	add    r11d,0xc00
   3ad53:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ad57:	inc    edi
   3ad59:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3ad61:	movzx  r9d,r10b
   3ad65:	lea    r10d,[r9*4+0x0]
   3ad6d:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3ad75:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3ad7a:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ad80:	cmp    rbx,rdi
   3ad83:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ad89:	mov    r10,rax
   3ad8c:	shr    r10,0x30
   3ad90:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3ad95:	add    r11d,r8d
   3ad98:	add    r11d,0xd00
   3ad9f:	mov    DWORD PTR [rdx+rdi*4],r11d
   3ada3:	inc    edi
   3ada5:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3adad:	movzx  r9d,r10b
   3adb1:	lea    r10d,[r9*4+0x0]
   3adb9:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3adc1:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3adc6:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3adcc:	cmp    rbx,rdi
   3adcf:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3add1:	shr    rax,0x38
   3add5:	movzx  r10d,BYTE PTR [r14+rdi*1]
   3adda:	add    r10d,r8d
   3addd:	add    r10d,0xe00
   3ade4:	mov    DWORD PTR [rdx+rdi*4],r10d
   3ade8:	inc    edi
   3adea:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3adf2:	lea    r9d,[rax*4+0x0]
   3adfa:	mov    edi,DWORD PTR [rsp+r9*1+0x408]
   3ae02:	cmp    edi,DWORD PTR [rsp+r9*1+0x8]
   3ae07:	jae    3a776 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x1a6>
   3ae0d:	cmp    rbx,rdi
   3ae10:	jbe    3ae4b <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x87b>
   3ae12:	movzx  r9d,BYTE PTR [r14+rdi*1]
   3ae17:	add    r9d,r8d
   3ae1a:	add    r9d,0xf00
   3ae21:	mov    DWORD PTR [rdx+rdi*4],r9d
   3ae25:	inc    edi
   3ae27:	jmp    3a890 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x2c0>
   3ae2c:	add    rax,0xfffffffffffffff8
   3ae30:	mov    r8,rax
   3ae33:	cmp    r8,rbx
   3ae36:	jb     3a6fe <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x12e>
   3ae3c:	jmp    3a757 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x187>
   3ae41:	mov    edi,ebx
   3ae43:	and    edi,0xfffff0
   3ae49:	jmp    3ae6c <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x89c>
   3ae4b:	lea    rdx,[rip+0x5921e]        # 94070 <__frame_dummy_init_array_entry+0xdb8>
   3ae52:	mov    rsi,rbx
   3ae55:	call   2a4ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3ae5a:	lea    rcx,[rip+0x5959f]        # 94400 <__frame_dummy_init_array_entry+0x1148>
   3ae61:	mov    rdi,rax
   3ae64:	mov    rdx,rbx
   3ae67:	call   2b1e0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3ae6c:	or     rdi,0x8
   3ae70:	add    rax,0x10
   3ae74:	lea    rcx,[rip+0x59585]        # 94400 <__frame_dummy_init_array_entry+0x1148>
   3ae7b:	mov    rsi,rax
   3ae7e:	mov    rdx,rbx
   3ae81:	call   2b1e0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3ae86:	add    rsp,0x808
   3ae8d:	pop    rbx
   3ae8e:	pop    r12
   3ae90:	pop    r13
   3ae92:	pop    r14
   3ae94:	pop    r15
   3ae96:	pop    rbp
   3ae97:	ret
   3ae98:	movzx  edx,BYTE PTR [r14+rax*1]
   3ae9d:	mov    DWORD PTR [r13+r12*4+0x0],edx
   3aea2:	inc    r12
   3aea5:	inc    rax
   3aea8:	cmp    rbx,rax
   3aeab:	jne    3ae98 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8c8>
   3aead:	mov    QWORD PTR [rcx+0x10],r12
   3aeb1:	lea    rdi,[rsp+0x8]
   3aeb6:	xor    ebp,ebp
   3aeb8:	mov    edx,0x400
   3aebd:	xor    esi,esi
   3aebf:	call   QWORD PTR [rip+0x5b16b]        # 96030 <memset@GLIBC_2.2.5>
   3aec5:	xor    eax,eax
   3aec7:	nop    WORD PTR [rax+rax*1+0x0]
   3aed0:	mov    DWORD PTR [rsp+rbp*4+0x8],eax
   3aed4:	add    eax,DWORD PTR [r15+rbp*4]
   3aed8:	mov    DWORD PTR [rsp+rbp*4+0xc],eax
   3aedc:	add    eax,DWORD PTR [r15+rbp*4+0x4]
   3aee1:	mov    DWORD PTR [rsp+rbp*4+0x10],eax
   3aee5:	add    eax,DWORD PTR [r15+rbp*4+0x8]
   3aeea:	mov    DWORD PTR [rsp+rbp*4+0x14],eax
   3aeee:	add    eax,DWORD PTR [r15+rbp*4+0xc]
   3aef3:	add    rbp,0x4
   3aef7:	cmp    rbp,0x100
   3aefe:	jne    3aed0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x900>
   3af00:	xor    ecx,ecx
   3af02:	cmp    rbx,0x10
   3af06:	jb     3b39a <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xdca>
   3af0c:	mov    esi,0x8
   3af11:	movdqa xmm0,XMMWORD PTR [rip+0xfffffffffffccf87]        # 7ea0 <GCC_except_table701+0x3f0>
   3af19:	movdqa xmm1,XMMWORD PTR [rip+0xfffffffffffccbdf]        # 7b00 <GCC_except_table701+0x50>
   3af21:	movdqa xmm2,XMMWORD PTR [rip+0xfffffffffffccdb7]        # 7ce0 <GCC_except_table701+0x230>
   3af29:	movdqa xmm3,XMMWORD PTR [rip+0xfffffffffffccfef]        # 7f20 <GCC_except_table701+0x470>
   3af31:	movq   xmm4,QWORD PTR [rip+0xfffffffffffccd47]        # 7c80 <GCC_except_table701+0x1d0>
   3af39:	movq   xmm5,QWORD PTR [rip+0xfffffffffffccd4f]        # 7c90 <GCC_except_table701+0x1e0>
   3af41:	jmp    3b01d <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xa4d>
   3af46:	cs nop WORD PTR [rax+rax*1+0x0]
   3af50:	movzx  edx,dl
   3af53:	mov    eax,edx
   3af55:	mov    edi,DWORD PTR [rsp+rax*4+0x8]
   3af59:	lea    rax,[rdi+0x10]
   3af5d:	cmp    rax,r12
   3af60:	ja     3b3f6 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe26>
   3af66:	movzx  eax,BYTE PTR [r13+rdi*4+0x0]
   3af6c:	add    eax,ecx
   3af6e:	mov    DWORD PTR [r13+rdi*4+0x0],eax
   3af73:	movdqu xmm6,XMMWORD PTR [r13+rdi*4+0x4]
   3af7a:	movdqu xmm7,XMMWORD PTR [r13+rdi*4+0x14]
   3af81:	movdqu xmm8,XMMWORD PTR [r13+rdi*4+0x24]
   3af88:	pand   xmm6,xmm0
   3af8c:	movd   xmm9,ecx
   3af91:	pshufd xmm9,xmm9,0x0
   3af97:	por    xmm6,xmm9
   3af9c:	por    xmm6,xmm1
   3afa0:	movdqu XMMWORD PTR [r13+rdi*4+0x4],xmm6
   3afa7:	pand   xmm7,xmm0
   3afab:	por    xmm7,xmm9
   3afb0:	por    xmm7,xmm2
   3afb4:	movdqu XMMWORD PTR [r13+rdi*4+0x14],xmm7
   3afbb:	pand   xmm8,xmm0
   3afc0:	por    xmm8,xmm9
   3afc5:	por    xmm8,xmm3
   3afca:	movdqu XMMWORD PTR [r13+rdi*4+0x24],xmm8
   3afd1:	movq   xmm6,QWORD PTR [r13+rdi*4+0x34]
   3afd8:	pand   xmm6,xmm4
   3afdc:	por    xmm6,xmm9
   3afe1:	por    xmm6,xmm5
   3afe5:	movq   QWORD PTR [r13+rdi*4+0x34],xmm6
   3afec:	movzx  eax,BYTE PTR [r13+rdi*4+0x3c]
   3aff2:	add    eax,ecx
   3aff4:	add    eax,0xf00
   3aff9:	mov    DWORD PTR [r13+rdi*4+0x3c],eax
   3affe:	add    DWORD PTR [rsp+rdx*4+0x8],0x10
   3b003:	lea    rax,[rsi+0x10]
   3b007:	add    rsi,0x18
   3b00b:	add    ecx,0x1000
   3b011:	cmp    rsi,rbx
   3b014:	mov    rsi,rax
   3b017:	ja     3b393 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xdc3>
   3b01d:	lea    rax,[rsi-0x8]
   3b021:	cmp    rsi,rbx
   3b024:	ja     3ae5a <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x88a>
   3b02a:	lea    rdx,[rsi+0x7]
   3b02e:	cmp    rdx,rbx
   3b031:	jae    3b3e4 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe14>
   3b037:	mov    rdx,QWORD PTR [r14+rsi*1-0x8]
   3b03c:	mov    rax,QWORD PTR [r14+rsi*1]
   3b040:	mov    rdi,rdx
   3b043:	xor    rdi,rax
   3b046:	mov    r8,rdx
   3b049:	rol    r8,0x8
   3b04d:	xor    r8,rdx
   3b050:	or     r8,rdi
   3b053:	je     3af50 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x980>
   3b059:	movzx  r8d,dl
   3b05d:	mov    edi,r8d
   3b060:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b064:	cmp    r12,rdi
   3b067:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b06d:	mov    r9,rdx
   3b070:	shr    r9,0x8
   3b074:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b07a:	add    r10d,ecx
   3b07d:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b082:	inc    edi
   3b084:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b089:	movzx  r8d,r9b
   3b08d:	mov    edi,r8d
   3b090:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b094:	cmp    r12,rdi
   3b097:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b09d:	mov    r9,rdx
   3b0a0:	shr    r9,0x10
   3b0a4:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b0aa:	add    r10d,ecx
   3b0ad:	add    r10d,0x100
   3b0b4:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b0b9:	inc    edi
   3b0bb:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b0c0:	movzx  r8d,r9b
   3b0c4:	mov    edi,r8d
   3b0c7:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b0cb:	cmp    r12,rdi
   3b0ce:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b0d4:	mov    r9,rdx
   3b0d7:	shr    r9,0x18
   3b0db:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b0e1:	add    r10d,ecx
   3b0e4:	add    r10d,0x200
   3b0eb:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b0f0:	inc    edi
   3b0f2:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b0f7:	movzx  r8d,r9b
   3b0fb:	mov    edi,r8d
   3b0fe:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b102:	cmp    r12,rdi
   3b105:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b10b:	mov    r9,rdx
   3b10e:	shr    r9,0x20
   3b112:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b118:	add    r10d,ecx
   3b11b:	add    r10d,0x300
   3b122:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b127:	inc    edi
   3b129:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b12e:	movzx  r8d,r9b
   3b132:	mov    edi,r8d
   3b135:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b139:	cmp    r12,rdi
   3b13c:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b142:	mov    r9,rdx
   3b145:	shr    r9,0x28
   3b149:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b14f:	add    r10d,ecx
   3b152:	add    r10d,0x400
   3b159:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b15e:	inc    edi
   3b160:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b165:	movzx  r8d,r9b
   3b169:	mov    edi,r8d
   3b16c:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b170:	cmp    r12,rdi
   3b173:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b179:	mov    r9,rdx
   3b17c:	shr    r9,0x30
   3b180:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b186:	add    r10d,ecx
   3b189:	add    r10d,0x500
   3b190:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b195:	inc    edi
   3b197:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b19c:	movzx  r8d,r9b
   3b1a0:	mov    edi,r8d
   3b1a3:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b1a7:	cmp    r12,rdi
   3b1aa:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b1b0:	shr    rdx,0x38
   3b1b4:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b1ba:	add    r9d,ecx
   3b1bd:	add    r9d,0x600
   3b1c4:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b1c9:	inc    edi
   3b1cb:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b1d0:	mov    edi,DWORD PTR [rsp+rdx*4+0x8]
   3b1d4:	cmp    r12,rdi
   3b1d7:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b1dd:	movzx  r8d,BYTE PTR [r13+rdi*4+0x0]
   3b1e3:	add    r8d,ecx
   3b1e6:	add    r8d,0x700
   3b1ed:	mov    DWORD PTR [r13+rdi*4+0x0],r8d
   3b1f2:	inc    edi
   3b1f4:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b1f8:	movzx  edx,al
   3b1fb:	mov    edi,edx
   3b1fd:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b201:	cmp    r12,rdi
   3b204:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b20a:	mov    r8,rax
   3b20d:	shr    r8,0x8
   3b211:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b217:	add    r9d,ecx
   3b21a:	add    r9d,0x800
   3b221:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b226:	inc    edi
   3b228:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b22c:	movzx  edx,r8b
   3b230:	mov    edi,edx
   3b232:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b236:	cmp    r12,rdi
   3b239:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b23f:	mov    r8,rax
   3b242:	shr    r8,0x10
   3b246:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b24c:	add    r9d,ecx
   3b24f:	add    r9d,0x900
   3b256:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b25b:	inc    edi
   3b25d:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b261:	movzx  edx,r8b
   3b265:	mov    edi,edx
   3b267:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b26b:	cmp    r12,rdi
   3b26e:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b274:	mov    r8,rax
   3b277:	shr    r8,0x18
   3b27b:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b281:	add    r9d,ecx
   3b284:	add    r9d,0xa00
   3b28b:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b290:	inc    edi
   3b292:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b296:	movzx  edx,r8b
   3b29a:	mov    edi,edx
   3b29c:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b2a0:	cmp    r12,rdi
   3b2a3:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b2a9:	mov    r8,rax
   3b2ac:	shr    r8,0x20
   3b2b0:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b2b6:	add    r9d,ecx
   3b2b9:	add    r9d,0xb00
   3b2c0:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b2c5:	inc    edi
   3b2c7:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b2cb:	movzx  edx,r8b
   3b2cf:	mov    edi,edx
   3b2d1:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b2d5:	cmp    r12,rdi
   3b2d8:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b2de:	mov    r8,rax
   3b2e1:	shr    r8,0x28
   3b2e5:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b2eb:	add    r9d,ecx
   3b2ee:	add    r9d,0xc00
   3b2f5:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b2fa:	inc    edi
   3b2fc:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b300:	movzx  edx,r8b
   3b304:	mov    edi,edx
   3b306:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b30a:	cmp    r12,rdi
   3b30d:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b313:	mov    r8,rax
   3b316:	shr    r8,0x30
   3b31a:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b320:	add    r9d,ecx
   3b323:	add    r9d,0xd00
   3b32a:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b32f:	inc    edi
   3b331:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b335:	movzx  edx,r8b
   3b339:	mov    edi,edx
   3b33b:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b33f:	cmp    r12,rdi
   3b342:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b348:	shr    rax,0x38
   3b34c:	movzx  r8d,BYTE PTR [r13+rdi*4+0x0]
   3b352:	add    r8d,ecx
   3b355:	add    r8d,0xe00
   3b35c:	mov    DWORD PTR [r13+rdi*4+0x0],r8d
   3b361:	inc    edi
   3b363:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b367:	mov    edi,DWORD PTR [rsp+rax*4+0x8]
   3b36b:	cmp    r12,rdi
   3b36e:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b374:	movzx  edx,BYTE PTR [r13+rdi*4+0x0]
   3b37a:	add    edx,ecx
   3b37c:	add    edx,0xf00
   3b382:	mov    DWORD PTR [r13+rdi*4+0x0],edx
   3b387:	lea    edx,[rdi+0x1]
   3b38a:	mov    DWORD PTR [rsp+rax*4+0x8],edx
   3b38e:	jmp    3b003 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xa33>
   3b393:	add    rax,0xfffffffffffffff8
   3b397:	mov    rcx,rax
   3b39a:	cmp    rcx,rbx
   3b39d:	jae    3ae86 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8b6>
   3b3a3:	mov    eax,ecx
   3b3a5:	shl    eax,0x8
   3b3a8:	nop    DWORD PTR [rax+rax*1+0x0]
   3b3b0:	movzx  edx,BYTE PTR [r14+rcx*1]
   3b3b5:	mov    edi,DWORD PTR [rsp+rdx*4+0x8]
   3b3b9:	cmp    r12,rdi
   3b3bc:	jbe    3b408 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xe38>
   3b3be:	inc    rcx
   3b3c1:	movzx  esi,BYTE PTR [r13+rdi*4+0x0]
   3b3c7:	add    esi,eax
   3b3c9:	mov    DWORD PTR [r13+rdi*4+0x0],esi
   3b3ce:	lea    esi,[rdi+0x1]
   3b3d1:	mov    DWORD PTR [rsp+rdx*4+0x8],esi
   3b3d5:	add    eax,0x100
   3b3da:	cmp    rbx,rcx
   3b3dd:	jne    3b3b0 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0xde0>
   3b3df:	jmp    3ae86 <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x8b6>
   3b3e4:	movabs rdi,0x7ffffffffffffff0
   3b3ee:	and    rdi,rbx
   3b3f1:	jmp    3ae6c <_RNvNtCs9pn3potmX1d_5mibz23bwt5build+0x89c>
   3b3f6:	lea    rcx,[rip+0x58fbb]        # 943b8 <__frame_dummy_init_array_entry+0x1100>
   3b3fd:	mov    rsi,rax
   3b400:	mov    rdx,r12
   3b403:	call   2b1e0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3b408:	lea    rdx,[rip+0x58fc1]        # 943d0 <__frame_dummy_init_array_entry+0x1118>
   3b40f:	mov    rsi,r12
   3b412:	call   2a4ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
