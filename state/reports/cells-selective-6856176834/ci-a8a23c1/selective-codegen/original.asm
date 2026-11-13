
bin/original-cells-ab:     file format elf64-x86-64


Disassembly of section .text:

0000000000150880 <<mibz2::cells::Kernel>::induce_cells>:
  150880:	push   %rbp
  150881:	push   %r15
  150883:	push   %r14
  150885:	push   %r13
  150887:	push   %r12
  150889:	push   %rbx
  15088a:	mov    0x78(%rsi),%rax
  15088e:	test   %rax,%rax
  150891:	je     1508a1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  150893:	mov    0x68(%rsi),%r15
  150897:	lea    0x10(%r15),%rcx
  15089b:	cmp    %rcx,0x10(%rsi)
  15089f:	jae    1508b4 <<mibz2::cells::Kernel>::induce_cells+0x34>
  1508a1:	movb   $0x3,0x1(%rdi)
  1508a5:	mov    $0x1,%al
  1508a7:	mov    %al,(%rdi)
  1508a9:	pop    %rbx
  1508aa:	pop    %r12
  1508ac:	pop    %r13
  1508ae:	pop    %r14
  1508b0:	pop    %r15
  1508b2:	pop    %rbp
  1508b3:	ret
  1508b4:	mov    0x50(%rsi),%r10
  1508b8:	mov    0x58(%rsi),%rcx
  1508bc:	mov    0x60(%rsi),%r8
  1508c0:	mov    %rdi,%r11
  1508c3:	mov    0x30(%rsi),%rdi
  1508c7:	mov    %rdi,-0x18(%rsp)
  1508cc:	mov    0x38(%rsi),%rdi
  1508d0:	mov    %rdi,-0x20(%rsp)
  1508d5:	mov    0x40(%rsi),%r9
  1508d9:	mov    %r9,-0x28(%rsp)
  1508de:	mov    0x48(%rsi),%rdi
  1508e2:	mov    %rdi,-0x30(%rsp)
  1508e7:	mov    0x8(%rsi),%rbx
  1508eb:	add    $0x120,%r8
  1508f2:	shl    $0x3,%rax
  1508f6:	lea    (%rax,%rax,2),%rax
  1508fa:	lea    (%rcx,%rax,1),%rdi
  1508fe:	mov    %rdi,-0x58(%rsp)
  150903:	mov    %r11,%rdi
  150906:	lea    (%r10,%rax,1),%r11
  15090a:	add    $0xffffffffffffffe8,%r11
  15090e:	add    %rax,%r10
  150911:	mov    %r10,-0x50(%rsp)
  150916:	add    %rax,%rcx
  150919:	add    $0xffffffffffffffe8,%rcx
  15091d:	mov    %rdx,-0x10(%rsp)
  150922:	neg    %rdx
  150925:	mov    %rdx,-0x8(%rsp)
  15092a:	movq   $0x0,-0x68(%rsp)
  150933:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  150940:	mov    %r11,%rax
  150943:	mov    -0x110(%rbx,%r11,1),%edx
  15094b:	mov    0x4(%rbx,%r11,1),%r11d
  150950:	mov    0x8(%rbx,%rax,1),%r9
  150955:	mov    0x10(%rbx,%rax,1),%r13
  15095a:	mov    -0x114(%rbx,%rax,1),%r10d
  150962:	mov    (%rbx,%rax,1),%eax
  150965:	mov    %rax,-0x60(%rsp)
  15096a:	mov    0x4(%rbx,%rcx,1),%ebp
  15096e:	mov    0x8(%rbx,%rcx,1),%r12
  150973:	mov    0x10(%rbx,%rcx,1),%r14
  150978:	prefetcht0 (%rbx,%rdx,1)
  15097c:	prefetcht0 (%rbx,%r10,1)
  150981:	mov    -0x110(%rbx,%rcx,1),%eax
  150988:	prefetcht0 (%rbx,%rax,1)
  15098c:	mov    -0x114(%rbx,%rcx,1),%eax
  150993:	prefetcht0 (%rbx,%rax,1)
  150997:	cmp    %rcx,%r13
  15099a:	mov    (%rbx,%rcx,1),%ecx
  15099d:	jne    1509ae <<mibz2::cells::Kernel>::induce_cells+0x12e>
  15099f:	test   %r9d,%r9d
  1509a2:	mov    -0x60(%rsp),%r11
  1509a7:	jns    150940 <<mibz2::cells::Kernel>::induce_cells+0xc0>
  1509a9:	jmp    150b9a <<mibz2::cells::Kernel>::induce_cells+0x31a>
  1509ae:	mov    %rbp,-0x48(%rsp)
  1509b3:	mov    %rsi,-0x38(%rsp)
  1509b8:	mov    %rdi,-0x40(%rsp)
  1509bd:	mov    -0x68(%rsp),%rdx
  1509c2:	cmp    -0x10(%rsp),%rdx
  1509c7:	je     150bfa <<mibz2::cells::Kernel>::induce_cells+0x37a>
  1509cd:	inc    %rdx
  1509d0:	mov    %r11,%rdi
  1509d3:	jmp    150a66 <<mibz2::cells::Kernel>::induce_cells+0x1e6>
  1509d8:	nopl   0x0(%rax,%rax,1)
  1509e0:	mov    %r10d,0x4(%rbx,%r14,1)
  1509e5:	mov    %rsi,0x8(%rbx,%r14,1)
  1509ea:	mov    %r13,0x10(%rbx,%r14,1)
  1509ef:	mov    (%rbx,%r13,1),%eax
  1509f3:	mov    -0x58(%rsp),%rdx
  1509f8:	mov    %eax,(%rbx,%rdx,1)
  1509fb:	mov    %edx,(%rbx,%r13,1)
  1509ff:	mov    %r10d,0x4(%rbx,%r13,1)
  150a04:	mov    %rsi,0x8(%rbx,%r13,1)
  150a09:	mov    %r14,0x10(%rbx,%r13,1)
  150a0e:	mov    %rdx,%r13
  150a11:	lea    0x18(%rdx),%rax
  150a15:	mov    -0x110(%rbx,%rcx,1),%edx
  150a1c:	prefetcht0 (%rbx,%rdx,1)
  150a20:	mov    -0x114(%rbx,%rcx,1),%edx
  150a27:	prefetcht0 (%rbx,%rdx,1)
  150a2b:	mov    0x4(%rbx,%rcx,1),%edx
  150a2f:	mov    %rdx,-0x48(%rsp)
  150a34:	mov    0x8(%rbx,%rcx,1),%r12
  150a39:	mov    0x10(%rbx,%rcx,1),%r14
  150a3e:	mov    (%rbx,%rcx,1),%ecx
  150a41:	mov    %rax,-0x58(%rsp)
  150a46:	mov    -0x68(%rsp),%rdx
  150a4b:	mov    -0x8(%rsp),%rax
  150a50:	add    %rdx,%rax
  150a53:	inc    %rax
  150a56:	inc    %rdx
  150a59:	mov    %rdi,%r11
  150a5c:	cmp    $0x1,%rax
  150a60:	je     150bfa <<mibz2::cells::Kernel>::induce_cells+0x37a>
  150a66:	mov    %rdx,-0x68(%rsp)
  150a6b:	mov    %r15,%r10
  150a6e:	mov    %r8,%rbp
  150a71:	mov    %r9d,%esi
  150a74:	mov    %r12d,%eax
  150a77:	add    %rsi,%rax
  150a7a:	mov    %r9,%r8
  150a7d:	shr    $0x20,%r8
  150a81:	cmp    $0x10,%rsi
  150a85:	mov    $0x10,%edx
  150a8a:	cmovae %rdx,%rsi
  150a8e:	mov    %r12,%rdx
  150a91:	shr    $0x20,%rdx
  150a95:	xor    %r15d,%r15d
  150a98:	cmp    $0x11,%rax
  150a9c:	setae  %r15b
  150aa0:	movups (%rbx,%r8,1),%xmm0
  150aa5:	movups %xmm0,(%rbx,%rbp,1)
  150aa9:	movups (%rbx,%rdx,1),%xmm0
  150aad:	lea    (%rbx,%rbp,1),%rdx
  150ab1:	movups %xmm0,(%rsi,%rdx,1)
  150ab5:	mov    %r9,(%rbx,%r10,1)
  150ab9:	mov    %r12,0x8(%rbx,%r10,1)
  150abe:	mov    %r10,%rsi
  150ac1:	cmovb  %rbp,%rsi
  150ac5:	mov    $0x0,%r8d
  150acb:	cmovb  %rax,%r8
  150acf:	shl    $0x4,%r15d
  150ad3:	add    %rbp,%r8
  150ad6:	add    %r10,%r15
  150ad9:	shl    $0x20,%rsi
  150add:	or     %rax,%rsi
  150ae0:	mov    -0x48(%rsp),%r10
  150ae5:	sub    %r10,%rdi
  150ae8:	je     150b68 <<mibz2::cells::Kernel>::induce_cells+0x2e8>
  150aea:	ja     1509e0 <<mibz2::cells::Kernel>::induce_cells+0x160>
  150af0:	mov    %r11d,0x4(%rbx,%r13,1)
  150af5:	mov    %rsi,0x8(%rbx,%r13,1)
  150afa:	mov    %r14,0x10(%rbx,%r13,1)
  150aff:	mov    (%rbx,%r14,1),%eax
  150b03:	mov    -0x50(%rsp),%rdx
  150b08:	mov    %eax,(%rbx,%rdx,1)
  150b0b:	mov    %edx,(%rbx,%r14,1)
  150b0f:	mov    %r11d,0x4(%rbx,%r14,1)
  150b14:	mov    %rsi,0x8(%rbx,%r14,1)
  150b19:	mov    %r13,0x10(%rbx,%r14,1)
  150b1e:	mov    %rdx,%r14
  150b21:	lea    0x18(%rdx),%rax
  150b25:	sub    %r11,%r10
  150b28:	mov    %r10,-0x48(%rsp)
  150b2d:	mov    -0x60(%rsp),%rsi
  150b32:	mov    -0x110(%rbx,%rsi,1),%edx
  150b39:	prefetcht0 (%rbx,%rdx,1)
  150b3d:	mov    -0x114(%rbx,%rsi,1),%edx
  150b44:	prefetcht0 (%rbx,%rdx,1)
  150b48:	mov    0x4(%rbx,%rsi,1),%edi
  150b4c:	mov    0x8(%rbx,%rsi,1),%r9
  150b51:	mov    0x10(%rbx,%rsi,1),%r13
  150b56:	mov    (%rbx,%rsi,1),%edx
  150b59:	mov    %rdx,-0x60(%rsp)
  150b5e:	mov    %rax,-0x50(%rsp)
  150b63:	jmp    150a46 <<mibz2::cells::Kernel>::induce_cells+0x1c6>
  150b68:	mov    %r11d,0x4(%rbx,%r14,1)
  150b6d:	mov    %rsi,0x8(%rbx,%r14,1)
  150b72:	mov    %r13,0x10(%rbx,%r14,1)
  150b77:	mov    %r11d,0x4(%rbx,%r13,1)
  150b7c:	mov    %rsi,0x8(%rbx,%r13,1)
  150b81:	mov    %r14,0x10(%rbx,%r13,1)
  150b86:	mov    -0x40(%rsp),%rdi
  150b8b:	mov    -0x38(%rsp),%rsi
  150b90:	mov    -0x60(%rsp),%r11
  150b95:	jmp    150940 <<mibz2::cells::Kernel>::induce_cells+0xc0>
  150b9a:	mov    -0x50(%rsp),%rax
  150b9f:	cmp    -0x18(%rsp),%rax
  150ba4:	ja     1508a1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  150baa:	mov    -0x58(%rsp),%rax
  150baf:	cmp    -0x20(%rsp),%rax
  150bb4:	ja     1508a1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  150bba:	add    $0x20,%r8
  150bbe:	cmp    -0x28(%rsp),%r8
  150bc3:	ja     1508a1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  150bc9:	lea    0x10(%r15),%rax
  150bcd:	cmp    -0x30(%rsp),%rax
  150bd2:	ja     1508a1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  150bd8:	mov    %r15,0x70(%rsi)
  150bdc:	mov    %r9,%rax
  150bdf:	shr    $0x20,%rax
  150be3:	mov    %eax,0x8(%rdi)
  150be6:	mov    %r9d,0xc(%rdi)
  150bea:	mov    -0x68(%rsp),%rax
  150bef:	mov    %rax,0x10(%rdi)
  150bf3:	xor    %eax,%eax
  150bf5:	jmp    1508a7 <<mibz2::cells::Kernel>::induce_cells+0x27>
  150bfa:	mov    -0x40(%rsp),%rdi
  150bff:	movb   $0x2,0x1(%rdi)
  150c03:	jmp    1508a5 <<mibz2::cells::Kernel>::induce_cells+0x25>
