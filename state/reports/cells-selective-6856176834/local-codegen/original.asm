
bin/original-cells-ab:     file format elf64-x86-64


Disassembly of section .text:

000000000014ff80 <<mibz2::cells::Kernel>::induce_cells>:
  14ff80:	push   %rbp
  14ff81:	push   %r15
  14ff83:	push   %r14
  14ff85:	push   %r13
  14ff87:	push   %r12
  14ff89:	push   %rbx
  14ff8a:	mov    0x78(%rsi),%rax
  14ff8e:	test   %rax,%rax
  14ff91:	je     14ffa1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  14ff93:	mov    0x68(%rsi),%r15
  14ff97:	lea    0x10(%r15),%rcx
  14ff9b:	cmp    %rcx,0x10(%rsi)
  14ff9f:	jae    14ffb4 <<mibz2::cells::Kernel>::induce_cells+0x34>
  14ffa1:	movb   $0x3,0x1(%rdi)
  14ffa5:	mov    $0x1,%al
  14ffa7:	mov    %al,(%rdi)
  14ffa9:	pop    %rbx
  14ffaa:	pop    %r12
  14ffac:	pop    %r13
  14ffae:	pop    %r14
  14ffb0:	pop    %r15
  14ffb2:	pop    %rbp
  14ffb3:	ret
  14ffb4:	mov    0x50(%rsi),%r10
  14ffb8:	mov    0x58(%rsi),%rcx
  14ffbc:	mov    0x60(%rsi),%r8
  14ffc0:	mov    %rdi,%r11
  14ffc3:	mov    0x30(%rsi),%rdi
  14ffc7:	mov    %rdi,-0x18(%rsp)
  14ffcc:	mov    0x38(%rsi),%rdi
  14ffd0:	mov    %rdi,-0x20(%rsp)
  14ffd5:	mov    0x40(%rsi),%r9
  14ffd9:	mov    %r9,-0x28(%rsp)
  14ffde:	mov    0x48(%rsi),%rdi
  14ffe2:	mov    %rdi,-0x30(%rsp)
  14ffe7:	mov    0x8(%rsi),%rbx
  14ffeb:	add    $0x120,%r8
  14fff2:	shl    $0x3,%rax
  14fff6:	lea    (%rax,%rax,2),%rax
  14fffa:	lea    (%rcx,%rax,1),%rdi
  14fffe:	mov    %rdi,-0x58(%rsp)
  150003:	mov    %r11,%rdi
  150006:	lea    (%r10,%rax,1),%r11
  15000a:	add    $0xffffffffffffffe8,%r11
  15000e:	add    %rax,%r10
  150011:	mov    %r10,-0x50(%rsp)
  150016:	add    %rax,%rcx
  150019:	add    $0xffffffffffffffe8,%rcx
  15001d:	mov    %rdx,-0x10(%rsp)
  150022:	neg    %rdx
  150025:	mov    %rdx,-0x8(%rsp)
  15002a:	movq   $0x0,-0x68(%rsp)
  150033:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  150040:	mov    %r11,%rax
  150043:	mov    -0x110(%rbx,%r11,1),%edx
  15004b:	mov    0x4(%rbx,%r11,1),%r11d
  150050:	mov    0x8(%rbx,%rax,1),%r9
  150055:	mov    0x10(%rbx,%rax,1),%r13
  15005a:	mov    -0x114(%rbx,%rax,1),%r10d
  150062:	mov    (%rbx,%rax,1),%eax
  150065:	mov    %rax,-0x60(%rsp)
  15006a:	mov    0x4(%rbx,%rcx,1),%ebp
  15006e:	mov    0x8(%rbx,%rcx,1),%r12
  150073:	mov    0x10(%rbx,%rcx,1),%r14
  150078:	prefetcht0 (%rbx,%rdx,1)
  15007c:	prefetcht0 (%rbx,%r10,1)
  150081:	mov    -0x110(%rbx,%rcx,1),%eax
  150088:	prefetcht0 (%rbx,%rax,1)
  15008c:	mov    -0x114(%rbx,%rcx,1),%eax
  150093:	prefetcht0 (%rbx,%rax,1)
  150097:	cmp    %rcx,%r13
  15009a:	mov    (%rbx,%rcx,1),%ecx
  15009d:	jne    1500ae <<mibz2::cells::Kernel>::induce_cells+0x12e>
  15009f:	test   %r9d,%r9d
  1500a2:	mov    -0x60(%rsp),%r11
  1500a7:	jns    150040 <<mibz2::cells::Kernel>::induce_cells+0xc0>
  1500a9:	jmp    15029a <<mibz2::cells::Kernel>::induce_cells+0x31a>
  1500ae:	mov    %rbp,-0x48(%rsp)
  1500b3:	mov    %rsi,-0x38(%rsp)
  1500b8:	mov    %rdi,-0x40(%rsp)
  1500bd:	mov    -0x68(%rsp),%rdx
  1500c2:	cmp    -0x10(%rsp),%rdx
  1500c7:	je     1502fa <<mibz2::cells::Kernel>::induce_cells+0x37a>
  1500cd:	inc    %rdx
  1500d0:	mov    %r11,%rdi
  1500d3:	jmp    150166 <<mibz2::cells::Kernel>::induce_cells+0x1e6>
  1500d8:	nopl   0x0(%rax,%rax,1)
  1500e0:	mov    %r10d,0x4(%rbx,%r14,1)
  1500e5:	mov    %rsi,0x8(%rbx,%r14,1)
  1500ea:	mov    %r13,0x10(%rbx,%r14,1)
  1500ef:	mov    (%rbx,%r13,1),%eax
  1500f3:	mov    -0x58(%rsp),%rdx
  1500f8:	mov    %eax,(%rbx,%rdx,1)
  1500fb:	mov    %edx,(%rbx,%r13,1)
  1500ff:	mov    %r10d,0x4(%rbx,%r13,1)
  150104:	mov    %rsi,0x8(%rbx,%r13,1)
  150109:	mov    %r14,0x10(%rbx,%r13,1)
  15010e:	mov    %rdx,%r13
  150111:	lea    0x18(%rdx),%rax
  150115:	mov    -0x110(%rbx,%rcx,1),%edx
  15011c:	prefetcht0 (%rbx,%rdx,1)
  150120:	mov    -0x114(%rbx,%rcx,1),%edx
  150127:	prefetcht0 (%rbx,%rdx,1)
  15012b:	mov    0x4(%rbx,%rcx,1),%edx
  15012f:	mov    %rdx,-0x48(%rsp)
  150134:	mov    0x8(%rbx,%rcx,1),%r12
  150139:	mov    0x10(%rbx,%rcx,1),%r14
  15013e:	mov    (%rbx,%rcx,1),%ecx
  150141:	mov    %rax,-0x58(%rsp)
  150146:	mov    -0x68(%rsp),%rdx
  15014b:	mov    -0x8(%rsp),%rax
  150150:	add    %rdx,%rax
  150153:	inc    %rax
  150156:	inc    %rdx
  150159:	mov    %rdi,%r11
  15015c:	cmp    $0x1,%rax
  150160:	je     1502fa <<mibz2::cells::Kernel>::induce_cells+0x37a>
  150166:	mov    %rdx,-0x68(%rsp)
  15016b:	mov    %r15,%r10
  15016e:	mov    %r8,%rbp
  150171:	mov    %r9d,%esi
  150174:	mov    %r12d,%eax
  150177:	add    %rsi,%rax
  15017a:	mov    %r9,%r8
  15017d:	shr    $0x20,%r8
  150181:	cmp    $0x10,%rsi
  150185:	mov    $0x10,%edx
  15018a:	cmovae %rdx,%rsi
  15018e:	mov    %r12,%rdx
  150191:	shr    $0x20,%rdx
  150195:	xor    %r15d,%r15d
  150198:	cmp    $0x11,%rax
  15019c:	setae  %r15b
  1501a0:	movups (%rbx,%r8,1),%xmm0
  1501a5:	movups %xmm0,(%rbx,%rbp,1)
  1501a9:	movups (%rbx,%rdx,1),%xmm0
  1501ad:	lea    (%rbx,%rbp,1),%rdx
  1501b1:	movups %xmm0,(%rsi,%rdx,1)
  1501b5:	mov    %r9,(%rbx,%r10,1)
  1501b9:	mov    %r12,0x8(%rbx,%r10,1)
  1501be:	mov    %r10,%rsi
  1501c1:	cmovb  %rbp,%rsi
  1501c5:	mov    $0x0,%r8d
  1501cb:	cmovb  %rax,%r8
  1501cf:	shl    $0x4,%r15d
  1501d3:	add    %rbp,%r8
  1501d6:	add    %r10,%r15
  1501d9:	shl    $0x20,%rsi
  1501dd:	or     %rax,%rsi
  1501e0:	mov    -0x48(%rsp),%r10
  1501e5:	sub    %r10,%rdi
  1501e8:	je     150268 <<mibz2::cells::Kernel>::induce_cells+0x2e8>
  1501ea:	ja     1500e0 <<mibz2::cells::Kernel>::induce_cells+0x160>
  1501f0:	mov    %r11d,0x4(%rbx,%r13,1)
  1501f5:	mov    %rsi,0x8(%rbx,%r13,1)
  1501fa:	mov    %r14,0x10(%rbx,%r13,1)
  1501ff:	mov    (%rbx,%r14,1),%eax
  150203:	mov    -0x50(%rsp),%rdx
  150208:	mov    %eax,(%rbx,%rdx,1)
  15020b:	mov    %edx,(%rbx,%r14,1)
  15020f:	mov    %r11d,0x4(%rbx,%r14,1)
  150214:	mov    %rsi,0x8(%rbx,%r14,1)
  150219:	mov    %r13,0x10(%rbx,%r14,1)
  15021e:	mov    %rdx,%r14
  150221:	lea    0x18(%rdx),%rax
  150225:	sub    %r11,%r10
  150228:	mov    %r10,-0x48(%rsp)
  15022d:	mov    -0x60(%rsp),%rsi
  150232:	mov    -0x110(%rbx,%rsi,1),%edx
  150239:	prefetcht0 (%rbx,%rdx,1)
  15023d:	mov    -0x114(%rbx,%rsi,1),%edx
  150244:	prefetcht0 (%rbx,%rdx,1)
  150248:	mov    0x4(%rbx,%rsi,1),%edi
  15024c:	mov    0x8(%rbx,%rsi,1),%r9
  150251:	mov    0x10(%rbx,%rsi,1),%r13
  150256:	mov    (%rbx,%rsi,1),%edx
  150259:	mov    %rdx,-0x60(%rsp)
  15025e:	mov    %rax,-0x50(%rsp)
  150263:	jmp    150146 <<mibz2::cells::Kernel>::induce_cells+0x1c6>
  150268:	mov    %r11d,0x4(%rbx,%r14,1)
  15026d:	mov    %rsi,0x8(%rbx,%r14,1)
  150272:	mov    %r13,0x10(%rbx,%r14,1)
  150277:	mov    %r11d,0x4(%rbx,%r13,1)
  15027c:	mov    %rsi,0x8(%rbx,%r13,1)
  150281:	mov    %r14,0x10(%rbx,%r13,1)
  150286:	mov    -0x40(%rsp),%rdi
  15028b:	mov    -0x38(%rsp),%rsi
  150290:	mov    -0x60(%rsp),%r11
  150295:	jmp    150040 <<mibz2::cells::Kernel>::induce_cells+0xc0>
  15029a:	mov    -0x50(%rsp),%rax
  15029f:	cmp    -0x18(%rsp),%rax
  1502a4:	ja     14ffa1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  1502aa:	mov    -0x58(%rsp),%rax
  1502af:	cmp    -0x20(%rsp),%rax
  1502b4:	ja     14ffa1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  1502ba:	add    $0x20,%r8
  1502be:	cmp    -0x28(%rsp),%r8
  1502c3:	ja     14ffa1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  1502c9:	lea    0x10(%r15),%rax
  1502cd:	cmp    -0x30(%rsp),%rax
  1502d2:	ja     14ffa1 <<mibz2::cells::Kernel>::induce_cells+0x21>
  1502d8:	mov    %r15,0x70(%rsi)
  1502dc:	mov    %r9,%rax
  1502df:	shr    $0x20,%rax
  1502e3:	mov    %eax,0x8(%rdi)
  1502e6:	mov    %r9d,0xc(%rdi)
  1502ea:	mov    -0x68(%rsp),%rax
  1502ef:	mov    %rax,0x10(%rdi)
  1502f3:	xor    %eax,%eax
  1502f5:	jmp    14ffa7 <<mibz2::cells::Kernel>::induce_cells+0x27>
  1502fa:	mov    -0x40(%rsp),%rdi
  1502ff:	movb   $0x2,0x1(%rdi)
  150303:	jmp    14ffa5 <<mibz2::cells::Kernel>::induce_cells+0x25>
