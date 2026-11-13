
bin/original-cells-ab:     file format elf64-x86-64


Disassembly of section .text:

00000000001347b0 <mibz2::cells::Kernel::induce_cells>:
  1347b0:	mov    0x78(%rsi),%rcx
  1347b4:	test   %rcx,%rcx
  1347b7:	je     1347c7 <mibz2::cells::Kernel::induce_cells+0x17>
  1347b9:	mov    0x68(%rsi),%r11
  1347bd:	lea    0x10(%r11),%rax
  1347c1:	cmp    %rax,0x10(%rsi)
  1347c5:	jae    1347d0 <mibz2::cells::Kernel::induce_cells+0x20>
  1347c7:	movb   $0x3,0x1(%rdi)
  1347cb:	mov    $0x1,%al
  1347cd:	mov    %al,(%rdi)
  1347cf:	ret
  1347d0:	push   %rbp
  1347d1:	push   %r15
  1347d3:	push   %r14
  1347d5:	push   %r13
  1347d7:	push   %r12
  1347d9:	push   %rbx
  1347da:	mov    %rdi,-0x58(%rsp)
  1347df:	mov    0x50(%rsi),%r9
  1347e3:	mov    0x58(%rsi),%rdi
  1347e7:	mov    0x60(%rsi),%rax
  1347eb:	mov    0x30(%rsi),%r8
  1347ef:	mov    %r8,-0x20(%rsp)
  1347f4:	mov    0x38(%rsi),%r8
  1347f8:	mov    %r8,-0x28(%rsp)
  1347fd:	mov    0x40(%rsi),%r13
  134801:	mov    0x48(%rsi),%r12
  134805:	mov    0x8(%rsi),%rbx
  134809:	add    $0x120,%rax
  13480f:	shl    $0x3,%rcx
  134813:	lea    (%rcx,%rcx,2),%rcx
  134817:	lea    (%rdi,%rcx,1),%r8
  13481b:	mov    %r8,-0x50(%rsp)
  134820:	lea    (%r9,%rcx,1),%r15
  134824:	add    $0xffffffffffffffe8,%r15
  134828:	add    %rcx,%r9
  13482b:	mov    %r9,-0x48(%rsp)
  134830:	lea    (%rdi,%rcx,1),%r8
  134834:	add    $0xffffffffffffffe8,%r8
  134838:	mov    %rdx,-0x18(%rsp)
  13483d:	neg    %rdx
  134840:	mov    %rdx,-0x10(%rsp)
  134845:	movq   $0x0,-0x70(%rsp)
  13484e:	xchg   %ax,%ax
  134850:	mov    -0x110(%rbx,%r15,1),%r9d
  134858:	mov    (%rbx,%r15,1),%ecx
  13485c:	mov    %rcx,-0x60(%rsp)
  134861:	mov    0x4(%rbx,%r15,1),%edi
  134866:	mov    0x8(%rbx,%r15,1),%r10
  13486b:	mov    0x10(%rbx,%r15,1),%rbp
  134870:	mov    (%rbx,%r8,1),%ecx
  134874:	mov    %rcx,-0x68(%rsp)
  134879:	mov    0x4(%rbx,%r8,1),%edx
  13487e:	mov    0x8(%rbx,%r8,1),%rcx
  134883:	mov    0x10(%rbx,%r8,1),%r14
  134888:	prefetcht0 (%rbx,%r9,1)
  13488d:	mov    -0x114(%rbx,%r15,1),%r9d
  134895:	prefetcht0 (%rbx,%r9,1)
  13489a:	mov    -0x110(%rbx,%r8,1),%r9d
  1348a2:	prefetcht0 (%rbx,%r9,1)
  1348a7:	mov    -0x114(%rbx,%r8,1),%r9d
  1348af:	prefetcht0 (%rbx,%r9,1)
  1348b4:	cmp    %r8,%rbp
  1348b7:	jne    1348cd <mibz2::cells::Kernel::induce_cells+0x11d>
  1348b9:	test   %r10d,%r10d
  1348bc:	mov    -0x60(%rsp),%r15
  1348c1:	mov    -0x68(%rsp),%r8
  1348c6:	jns    134850 <mibz2::cells::Kernel::induce_cells+0xa0>
  1348c8:	jmp    134acb <mibz2::cells::Kernel::induce_cells+0x31b>
  1348cd:	mov    %r13,-0x40(%rsp)
  1348d2:	mov    %r12,-0x38(%rsp)
  1348d7:	mov    %rsi,-0x30(%rsp)
  1348dc:	mov    -0x70(%rsp),%r8
  1348e1:	cmp    -0x18(%rsp),%r8
  1348e6:	je     134b19 <mibz2::cells::Kernel::induce_cells+0x369>
  1348ec:	inc    %r8
  1348ef:	mov    %rdi,%r9
  1348f2:	jmp    134988 <mibz2::cells::Kernel::induce_cells+0x1d8>
  1348f7:	nopw   0x0(%rax,%rax,1)
  134900:	mov    %edx,0x4(%rbx,%r14,1)
  134905:	mov    %rsi,0x8(%rbx,%r14,1)
  13490a:	mov    %rbp,0x10(%rbx,%r14,1)
  13490f:	mov    (%rbx,%rbp,1),%ecx
  134912:	mov    -0x50(%rsp),%rdi
  134917:	mov    %ecx,(%rbx,%rdi,1)
  13491a:	mov    %edi,(%rbx,%rbp,1)
  13491d:	mov    %edx,0x4(%rbx,%rbp,1)
  134921:	mov    %rsi,0x8(%rbx,%rbp,1)
  134926:	mov    %r14,0x10(%rbx,%rbp,1)
  13492b:	mov    %rdi,%rbp
  13492e:	lea    0x18(%rdi),%rsi
  134932:	mov    -0x68(%rsp),%rdi
  134937:	mov    -0x110(%rbx,%rdi,1),%ecx
  13493e:	prefetcht0 (%rbx,%rcx,1)
  134942:	mov    -0x114(%rbx,%rdi,1),%ecx
  134949:	prefetcht0 (%rbx,%rcx,1)
  13494d:	mov    0x4(%rbx,%rdi,1),%edx
  134951:	mov    0x8(%rbx,%rdi,1),%rcx
  134956:	mov    0x10(%rbx,%rdi,1),%r14
  13495b:	mov    (%rbx,%rdi,1),%edi
  13495e:	mov    %rdi,-0x68(%rsp)
  134963:	mov    %rsi,-0x50(%rsp)
  134968:	mov    -0x70(%rsp),%r8
  13496d:	mov    -0x10(%rsp),%rsi
  134972:	add    %r8,%rsi
  134975:	inc    %rsi
  134978:	inc    %r8
  13497b:	mov    %r9,%rdi
  13497e:	cmp    $0x1,%rsi
  134982:	je     134b19 <mibz2::cells::Kernel::induce_cells+0x369>
  134988:	mov    %r9,-0x8(%rsp)
  13498d:	mov    %r8,-0x70(%rsp)
  134992:	mov    %r11,%r9
  134995:	mov    %rax,%r12
  134998:	mov    %r10d,%eax
  13499b:	mov    %ecx,%r8d
  13499e:	add    %rax,%r8
  1349a1:	mov    %r10,%rsi
  1349a4:	shr    $0x20,%rsi
  1349a8:	cmp    $0x10,%rax
  1349ac:	mov    $0x10,%r11d
  1349b2:	cmovae %r11,%rax
  1349b6:	lea    (%rbx,%r12,1),%r13
  1349ba:	mov    %rcx,%r15
  1349bd:	shr    $0x20,%r15
  1349c1:	xor    %r11d,%r11d
  1349c4:	cmp    $0x11,%r8
  1349c8:	setae  %r11b
  1349cc:	movups (%rbx,%rsi,1),%xmm0
  1349d0:	movups %xmm0,(%rbx,%r12,1)
  1349d5:	movups (%rbx,%r15,1),%xmm0
  1349da:	movups %xmm0,(%rax,%r13,1)
  1349df:	mov    %r10,(%rbx,%r9,1)
  1349e3:	mov    %rcx,0x8(%rbx,%r9,1)
  1349e8:	mov    %r9,%rsi
  1349eb:	cmovb  %r12,%rsi
  1349ef:	mov    $0x0,%eax
  1349f4:	cmovb  %r8,%rax
  1349f8:	add    %r12,%rax
  1349fb:	shl    $0x4,%r11d
  1349ff:	add    %r9,%r11
  134a02:	mov    -0x8(%rsp),%r9
  134a07:	shl    $0x20,%rsi
  134a0b:	or     %r8,%rsi
  134a0e:	sub    %rdx,%r9
  134a11:	je     134a90 <mibz2::cells::Kernel::induce_cells+0x2e0>
  134a13:	ja     134900 <mibz2::cells::Kernel::induce_cells+0x150>
  134a19:	mov    %edi,0x4(%rbx,%rbp,1)
  134a1d:	mov    %rsi,0x8(%rbx,%rbp,1)
  134a22:	mov    %r14,0x10(%rbx,%rbp,1)
  134a27:	mov    (%rbx,%r14,1),%r8d
  134a2b:	mov    -0x48(%rsp),%r9
  134a30:	mov    %r8d,(%rbx,%r9,1)
  134a34:	mov    %r9d,(%rbx,%r14,1)
  134a38:	mov    %edi,0x4(%rbx,%r14,1)
  134a3d:	mov    %rsi,0x8(%rbx,%r14,1)
  134a42:	mov    %rbp,0x10(%rbx,%r14,1)
  134a47:	mov    %r9,%r14
  134a4a:	lea    0x18(%r9),%rsi
  134a4e:	sub    %rdi,%rdx
  134a51:	mov    -0x60(%rsp),%r8
  134a56:	mov    -0x110(%rbx,%r8,1),%edi
  134a5e:	prefetcht0 (%rbx,%rdi,1)
  134a62:	mov    -0x114(%rbx,%r8,1),%edi
  134a6a:	prefetcht0 (%rbx,%rdi,1)
  134a6e:	mov    0x4(%rbx,%r8,1),%r9d
  134a73:	mov    0x8(%rbx,%r8,1),%r10
  134a78:	mov    0x10(%rbx,%r8,1),%rbp
  134a7d:	mov    (%rbx,%r8,1),%edi
  134a81:	mov    %rdi,-0x60(%rsp)
  134a86:	mov    %rsi,-0x48(%rsp)
  134a8b:	jmp    134968 <mibz2::cells::Kernel::induce_cells+0x1b8>
  134a90:	mov    %edi,0x4(%rbx,%r14,1)
  134a95:	mov    %rsi,0x8(%rbx,%r14,1)
  134a9a:	mov    %rbp,0x10(%rbx,%r14,1)
  134a9f:	mov    %edi,0x4(%rbx,%rbp,1)
  134aa3:	mov    %rsi,0x8(%rbx,%rbp,1)
  134aa8:	mov    %r14,0x10(%rbx,%rbp,1)
  134aad:	mov    -0x30(%rsp),%rsi
  134ab2:	mov    -0x38(%rsp),%r12
  134ab7:	mov    -0x40(%rsp),%r13
  134abc:	mov    -0x60(%rsp),%r15
  134ac1:	mov    -0x68(%rsp),%r8
  134ac6:	jmp    134850 <mibz2::cells::Kernel::induce_cells+0xa0>
  134acb:	mov    -0x48(%rsp),%rcx
  134ad0:	cmp    -0x20(%rsp),%rcx
  134ad5:	ja     134b24 <mibz2::cells::Kernel::induce_cells+0x374>
  134ad7:	mov    -0x50(%rsp),%rcx
  134adc:	cmp    -0x28(%rsp),%rcx
  134ae1:	ja     134b24 <mibz2::cells::Kernel::induce_cells+0x374>
  134ae3:	add    $0x20,%rax
  134ae7:	cmp    %r13,%rax
  134aea:	ja     134b24 <mibz2::cells::Kernel::induce_cells+0x374>
  134aec:	lea    0x10(%r11),%rax
  134af0:	cmp    %r12,%rax
  134af3:	ja     134b24 <mibz2::cells::Kernel::induce_cells+0x374>
  134af5:	mov    %r11,0x70(%rsi)
  134af9:	mov    %r10,%rax
  134afc:	shr    $0x20,%rax
  134b00:	mov    -0x58(%rsp),%rdi
  134b05:	mov    %eax,0x8(%rdi)
  134b08:	mov    %r10d,0xc(%rdi)
  134b0c:	mov    -0x70(%rsp),%rax
  134b11:	mov    %rax,0x10(%rdi)
  134b15:	xor    %eax,%eax
  134b17:	jmp    134b2f <mibz2::cells::Kernel::induce_cells+0x37f>
  134b19:	mov    -0x58(%rsp),%rdi
  134b1e:	movb   $0x2,0x1(%rdi)
  134b22:	jmp    134b2d <mibz2::cells::Kernel::induce_cells+0x37d>
  134b24:	mov    -0x58(%rsp),%rdi
  134b29:	movb   $0x3,0x1(%rdi)
  134b2d:	mov    $0x1,%al
  134b2f:	pop    %rbx
  134b30:	pop    %r12
  134b32:	pop    %r13
  134b34:	pop    %r14
  134b36:	pop    %r15
  134b38:	pop    %rbp
  134b39:	mov    %al,(%rdi)
  134b3b:	ret
