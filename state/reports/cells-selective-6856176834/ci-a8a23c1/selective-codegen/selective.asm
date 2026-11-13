
bin/cells-ab:     file format elf64-x86-64


Disassembly of section .text:

0000000000150880 <<mibz2::cells::Kernel>::induce_cells>:
  150880:	mov    0x78(%rsi),%rcx
  150884:	test   %rcx,%rcx
  150887:	je     150897 <<mibz2::cells::Kernel>::induce_cells+0x17>
  150889:	mov    0x68(%rsi),%rax
  15088d:	lea    0x10(%rax),%r8
  150891:	cmp    %r8,0x10(%rsi)
  150895:	jae    1508a0 <<mibz2::cells::Kernel>::induce_cells+0x20>
  150897:	movb   $0x3,0x1(%rdi)
  15089b:	mov    $0x1,%al
  15089d:	mov    %al,(%rdi)
  15089f:	ret
  1508a0:	push   %rbp
  1508a1:	push   %r15
  1508a3:	push   %r14
  1508a5:	push   %r13
  1508a7:	push   %r12
  1508a9:	push   %rbx
  1508aa:	mov    0x50(%rsi),%r11
  1508ae:	mov    0x58(%rsi),%r8
  1508b2:	mov    0x60(%rsi),%r10
  1508b6:	mov    0x30(%rsi),%r9
  1508ba:	mov    %r9,-0x20(%rsp)
  1508bf:	mov    0x38(%rsi),%r9
  1508c3:	mov    %r9,-0x28(%rsp)
  1508c8:	mov    0x40(%rsi),%r9
  1508cc:	mov    %r9,-0x30(%rsp)
  1508d1:	mov    0x48(%rsi),%r9
  1508d5:	mov    %r9,-0x38(%rsp)
  1508da:	mov    %r10,%r9
  1508dd:	mov    %rsi,-0x40(%rsp)
  1508e2:	mov    0x8(%rsi),%r14
  1508e6:	add    $0x120,%r9
  1508ed:	shl    $0x3,%rcx
  1508f1:	lea    (%rcx,%rcx,2),%rcx
  1508f5:	lea    (%r8,%rcx,1),%rsi
  1508f9:	mov    %rsi,-0x58(%rsp)
  1508fe:	lea    (%r11,%rcx,1),%rbp
  150902:	add    $0xffffffffffffffe8,%rbp
  150906:	add    %rcx,%r11
  150909:	mov    %r11,-0x50(%rsp)
  15090e:	add    %r8,%rcx
  150911:	add    $0xffffffffffffffe8,%rcx
  150915:	mov    %rdx,-0x18(%rsp)
  15091a:	neg    %rdx
  15091d:	mov    %rdx,-0x8(%rsp)
  150922:	xor    %r10d,%r10d
  150925:	data16 cs nopw 0x0(%rax,%rax,1)
  150930:	mov    -0x110(%r14,%rbp,1),%esi
  150938:	mov    0x4(%r14,%rbp,1),%ebx
  15093d:	mov    0x8(%r14,%rbp,1),%rdx
  150942:	mov    0x10(%r14,%rbp,1),%r13
  150947:	mov    -0x114(%r14,%rbp,1),%r8d
  15094f:	mov    (%r14,%rbp,1),%ebp
  150953:	mov    0x4(%r14,%rcx,1),%r11d
  150958:	mov    0x8(%r14,%rcx,1),%r12
  15095d:	mov    0x10(%r14,%rcx,1),%r15
  150962:	prefetcht0 (%r14,%rsi,1)
  150967:	prefetcht0 (%r14,%r8,1)
  15096c:	mov    -0x110(%r14,%rcx,1),%esi
  150974:	prefetcht0 (%r14,%rsi,1)
  150979:	mov    -0x114(%r14,%rcx,1),%esi
  150981:	prefetcht0 (%r14,%rsi,1)
  150986:	cmp    %rcx,%r13
  150989:	mov    (%r14,%rcx,1),%ecx
  15098d:	jne    150998 <<mibz2::cells::Kernel>::induce_cells+0x118>
  15098f:	test   %edx,%edx
  150991:	jns    150930 <<mibz2::cells::Kernel>::induce_cells+0xb0>
  150993:	jmp    150b51 <<mibz2::cells::Kernel>::induce_cells+0x2d1>
  150998:	mov    %rdi,-0x48(%rsp)
  15099d:	cmp    -0x18(%rsp),%r10
  1509a2:	je     150b9d <<mibz2::cells::Kernel>::induce_cells+0x31d>
  1509a8:	inc    %r10
  1509ab:	jmp    150a33 <<mibz2::cells::Kernel>::induce_cells+0x1b3>
  1509b0:	mov    %r11d,0x4(%r14,%r15,1)
  1509b5:	mov    %r8,0x8(%r14,%r15,1)
  1509ba:	mov    %r13,0x10(%r14,%r15,1)
  1509bf:	mov    (%r14,%r13,1),%esi
  1509c3:	mov    -0x58(%rsp),%rbx
  1509c8:	mov    %esi,(%r14,%rbx,1)
  1509cc:	mov    %ebx,(%r14,%r13,1)
  1509d0:	mov    %r11d,0x4(%r14,%r13,1)
  1509d5:	mov    %r8,0x8(%r14,%r13,1)
  1509da:	mov    %r15,0x10(%r14,%r13,1)
  1509df:	mov    %rbx,%r13
  1509e2:	lea    0x18(%rbx),%rsi
  1509e6:	mov    -0x110(%r14,%rcx,1),%r8d
  1509ee:	prefetcht0 (%r14,%r8,1)
  1509f3:	mov    -0x114(%r14,%rcx,1),%r8d
  1509fb:	prefetcht0 (%r14,%r8,1)
  150a00:	mov    0x4(%r14,%rcx,1),%r11d
  150a05:	mov    0x8(%r14,%rcx,1),%r12
  150a0a:	mov    0x10(%r14,%rcx,1),%r15
  150a0f:	mov    (%r14,%rcx,1),%ecx
  150a13:	mov    %rsi,-0x58(%rsp)
  150a18:	mov    -0x8(%rsp),%rsi
  150a1d:	add    %r10,%rsi
  150a20:	inc    %rsi
  150a23:	inc    %r10
  150a26:	mov    %rdi,%rbx
  150a29:	cmp    $0x1,%rsi
  150a2d:	je     150b9d <<mibz2::cells::Kernel>::induce_cells+0x31d>
  150a33:	mov    %edx,%r8d
  150a36:	mov    %r12d,%edi
  150a39:	add    %r8,%rdi
  150a3c:	cmp    $0x11,%rdi
  150a40:	jae    150a90 <<mibz2::cells::Kernel>::induce_cells+0x210>
  150a42:	lea    (%r14,%r9,1),%rsi
  150a46:	mov    %rsi,-0x10(%rsp)
  150a4b:	mov    %rax,%rsi
  150a4e:	mov    %r10,%rax
  150a51:	mov    %r9,%r10
  150a54:	mov    %rdx,%r9
  150a57:	shr    $0x20,%r9
  150a5b:	movups (%r14,%r9,1),%xmm0
  150a60:	movups %xmm0,(%r14,%r10,1)
  150a65:	mov    %r12,%r9
  150a68:	shr    $0x20,%r9
  150a6c:	movups (%r14,%r9,1),%xmm0
  150a71:	mov    -0x10(%rsp),%r9
  150a76:	movups %xmm0,(%r8,%r9,1)
  150a7b:	mov    %r10,%r8
  150a7e:	lea    (%r10,%rdi,1),%r9
  150a82:	mov    %rax,%r10
  150a85:	mov    %rsi,%rax
  150a88:	xor    %esi,%esi
  150a8a:	jmp    150aa1 <<mibz2::cells::Kernel>::induce_cells+0x221>
  150a8c:	nopl   0x0(%rax)
  150a90:	mov    %rdx,(%r14,%rax,1)
  150a94:	mov    %r12,0x8(%r14,%rax,1)
  150a99:	mov    $0x10,%esi
  150a9e:	mov    %rax,%r8
  150aa1:	add    %rsi,%rax
  150aa4:	shl    $0x20,%r8
  150aa8:	or     %rdi,%r8
  150aab:	mov    %rbx,%rdi
  150aae:	sub    %r11,%rdi
  150ab1:	je     150b29 <<mibz2::cells::Kernel>::induce_cells+0x2a9>
  150ab3:	ja     1509b0 <<mibz2::cells::Kernel>::induce_cells+0x130>
  150ab9:	mov    %ebx,0x4(%r14,%r13,1)
  150abe:	mov    %r8,0x8(%r14,%r13,1)
  150ac3:	mov    %r15,0x10(%r14,%r13,1)
  150ac8:	mov    (%r14,%r15,1),%edx
  150acc:	mov    -0x50(%rsp),%rsi
  150ad1:	mov    %edx,(%r14,%rsi,1)
  150ad5:	mov    %esi,(%r14,%r15,1)
  150ad9:	mov    %ebx,0x4(%r14,%r15,1)
  150ade:	mov    %r8,0x8(%r14,%r15,1)
  150ae3:	mov    %r13,0x10(%r14,%r15,1)
  150ae8:	mov    %rsi,%r15
  150aeb:	add    $0x18,%rsi
  150aef:	sub    %rbx,%r11
  150af2:	mov    -0x110(%r14,%rbp,1),%edx
  150afa:	prefetcht0 (%r14,%rdx,1)
  150aff:	mov    -0x114(%r14,%rbp,1),%edx
  150b07:	prefetcht0 (%r14,%rdx,1)
  150b0c:	mov    0x4(%r14,%rbp,1),%edi
  150b11:	mov    0x8(%r14,%rbp,1),%rdx
  150b16:	mov    0x10(%r14,%rbp,1),%r13
  150b1b:	mov    (%r14,%rbp,1),%ebp
  150b1f:	mov    %rsi,-0x50(%rsp)
  150b24:	jmp    150a18 <<mibz2::cells::Kernel>::induce_cells+0x198>
  150b29:	mov    %ebx,0x4(%r14,%r15,1)
  150b2e:	mov    %r8,0x8(%r14,%r15,1)
  150b33:	mov    %r13,0x10(%r14,%r15,1)
  150b38:	mov    %ebx,0x4(%r14,%r13,1)
  150b3d:	mov    %r8,0x8(%r14,%r13,1)
  150b42:	mov    %r15,0x10(%r14,%r13,1)
  150b47:	mov    -0x48(%rsp),%rdi
  150b4c:	jmp    150930 <<mibz2::cells::Kernel>::induce_cells+0xb0>
  150b51:	mov    -0x50(%rsp),%rcx
  150b56:	cmp    -0x20(%rsp),%rcx
  150b5b:	ja     150ba8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150b5d:	mov    -0x58(%rsp),%rcx
  150b62:	cmp    -0x28(%rsp),%rcx
  150b67:	ja     150ba8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150b69:	add    $0x20,%r9
  150b6d:	cmp    -0x30(%rsp),%r9
  150b72:	ja     150ba8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150b74:	lea    0x10(%rax),%rcx
  150b78:	cmp    -0x38(%rsp),%rcx
  150b7d:	ja     150ba8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150b7f:	mov    -0x40(%rsp),%rcx
  150b84:	mov    %rax,0x70(%rcx)
  150b88:	mov    %rdx,%rax
  150b8b:	shr    $0x20,%rax
  150b8f:	mov    %eax,0x8(%rdi)
  150b92:	mov    %edx,0xc(%rdi)
  150b95:	mov    %r10,0x10(%rdi)
  150b99:	xor    %eax,%eax
  150b9b:	jmp    150bae <<mibz2::cells::Kernel>::induce_cells+0x32e>
  150b9d:	mov    -0x48(%rsp),%rdi
  150ba2:	movb   $0x2,0x1(%rdi)
  150ba6:	jmp    150bac <<mibz2::cells::Kernel>::induce_cells+0x32c>
  150ba8:	movb   $0x3,0x1(%rdi)
  150bac:	mov    $0x1,%al
  150bae:	pop    %rbx
  150baf:	pop    %r12
  150bb1:	pop    %r13
  150bb3:	pop    %r14
  150bb5:	pop    %r15
  150bb7:	pop    %rbp
  150bb8:	mov    %al,(%rdi)
  150bba:	ret
