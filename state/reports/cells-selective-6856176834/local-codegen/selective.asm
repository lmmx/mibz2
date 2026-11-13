
bin/cells-ab:     file format elf64-x86-64


Disassembly of section .text:

000000000014ff80 <<mibz2::cells::Kernel>::induce_cells>:
  14ff80:	mov    0x78(%rsi),%rcx
  14ff84:	test   %rcx,%rcx
  14ff87:	je     14ff97 <<mibz2::cells::Kernel>::induce_cells+0x17>
  14ff89:	mov    0x68(%rsi),%rax
  14ff8d:	lea    0x10(%rax),%r8
  14ff91:	cmp    %r8,0x10(%rsi)
  14ff95:	jae    14ffa0 <<mibz2::cells::Kernel>::induce_cells+0x20>
  14ff97:	movb   $0x3,0x1(%rdi)
  14ff9b:	mov    $0x1,%al
  14ff9d:	mov    %al,(%rdi)
  14ff9f:	ret
  14ffa0:	push   %rbp
  14ffa1:	push   %r15
  14ffa3:	push   %r14
  14ffa5:	push   %r13
  14ffa7:	push   %r12
  14ffa9:	push   %rbx
  14ffaa:	mov    0x50(%rsi),%r11
  14ffae:	mov    0x58(%rsi),%r8
  14ffb2:	mov    0x60(%rsi),%r10
  14ffb6:	mov    0x30(%rsi),%r9
  14ffba:	mov    %r9,-0x20(%rsp)
  14ffbf:	mov    0x38(%rsi),%r9
  14ffc3:	mov    %r9,-0x28(%rsp)
  14ffc8:	mov    0x40(%rsi),%r9
  14ffcc:	mov    %r9,-0x30(%rsp)
  14ffd1:	mov    0x48(%rsi),%r9
  14ffd5:	mov    %r9,-0x38(%rsp)
  14ffda:	mov    %r10,%r9
  14ffdd:	mov    %rsi,-0x40(%rsp)
  14ffe2:	mov    0x8(%rsi),%r14
  14ffe6:	add    $0x120,%r9
  14ffed:	shl    $0x3,%rcx
  14fff1:	lea    (%rcx,%rcx,2),%rcx
  14fff5:	lea    (%r8,%rcx,1),%rsi
  14fff9:	mov    %rsi,-0x58(%rsp)
  14fffe:	lea    (%r11,%rcx,1),%rbp
  150002:	add    $0xffffffffffffffe8,%rbp
  150006:	add    %rcx,%r11
  150009:	mov    %r11,-0x50(%rsp)
  15000e:	add    %r8,%rcx
  150011:	add    $0xffffffffffffffe8,%rcx
  150015:	mov    %rdx,-0x18(%rsp)
  15001a:	neg    %rdx
  15001d:	mov    %rdx,-0x8(%rsp)
  150022:	xor    %r10d,%r10d
  150025:	data16 cs nopw 0x0(%rax,%rax,1)
  150030:	mov    -0x110(%r14,%rbp,1),%esi
  150038:	mov    0x4(%r14,%rbp,1),%ebx
  15003d:	mov    0x8(%r14,%rbp,1),%rdx
  150042:	mov    0x10(%r14,%rbp,1),%r13
  150047:	mov    -0x114(%r14,%rbp,1),%r8d
  15004f:	mov    (%r14,%rbp,1),%ebp
  150053:	mov    0x4(%r14,%rcx,1),%r11d
  150058:	mov    0x8(%r14,%rcx,1),%r12
  15005d:	mov    0x10(%r14,%rcx,1),%r15
  150062:	prefetcht0 (%r14,%rsi,1)
  150067:	prefetcht0 (%r14,%r8,1)
  15006c:	mov    -0x110(%r14,%rcx,1),%esi
  150074:	prefetcht0 (%r14,%rsi,1)
  150079:	mov    -0x114(%r14,%rcx,1),%esi
  150081:	prefetcht0 (%r14,%rsi,1)
  150086:	cmp    %rcx,%r13
  150089:	mov    (%r14,%rcx,1),%ecx
  15008d:	jne    150098 <<mibz2::cells::Kernel>::induce_cells+0x118>
  15008f:	test   %edx,%edx
  150091:	jns    150030 <<mibz2::cells::Kernel>::induce_cells+0xb0>
  150093:	jmp    150251 <<mibz2::cells::Kernel>::induce_cells+0x2d1>
  150098:	mov    %rdi,-0x48(%rsp)
  15009d:	cmp    -0x18(%rsp),%r10
  1500a2:	je     15029d <<mibz2::cells::Kernel>::induce_cells+0x31d>
  1500a8:	inc    %r10
  1500ab:	jmp    150133 <<mibz2::cells::Kernel>::induce_cells+0x1b3>
  1500b0:	mov    %r11d,0x4(%r14,%r15,1)
  1500b5:	mov    %r8,0x8(%r14,%r15,1)
  1500ba:	mov    %r13,0x10(%r14,%r15,1)
  1500bf:	mov    (%r14,%r13,1),%esi
  1500c3:	mov    -0x58(%rsp),%rbx
  1500c8:	mov    %esi,(%r14,%rbx,1)
  1500cc:	mov    %ebx,(%r14,%r13,1)
  1500d0:	mov    %r11d,0x4(%r14,%r13,1)
  1500d5:	mov    %r8,0x8(%r14,%r13,1)
  1500da:	mov    %r15,0x10(%r14,%r13,1)
  1500df:	mov    %rbx,%r13
  1500e2:	lea    0x18(%rbx),%rsi
  1500e6:	mov    -0x110(%r14,%rcx,1),%r8d
  1500ee:	prefetcht0 (%r14,%r8,1)
  1500f3:	mov    -0x114(%r14,%rcx,1),%r8d
  1500fb:	prefetcht0 (%r14,%r8,1)
  150100:	mov    0x4(%r14,%rcx,1),%r11d
  150105:	mov    0x8(%r14,%rcx,1),%r12
  15010a:	mov    0x10(%r14,%rcx,1),%r15
  15010f:	mov    (%r14,%rcx,1),%ecx
  150113:	mov    %rsi,-0x58(%rsp)
  150118:	mov    -0x8(%rsp),%rsi
  15011d:	add    %r10,%rsi
  150120:	inc    %rsi
  150123:	inc    %r10
  150126:	mov    %rdi,%rbx
  150129:	cmp    $0x1,%rsi
  15012d:	je     15029d <<mibz2::cells::Kernel>::induce_cells+0x31d>
  150133:	mov    %edx,%r8d
  150136:	mov    %r12d,%edi
  150139:	add    %r8,%rdi
  15013c:	cmp    $0x11,%rdi
  150140:	jae    150190 <<mibz2::cells::Kernel>::induce_cells+0x210>
  150142:	lea    (%r14,%r9,1),%rsi
  150146:	mov    %rsi,-0x10(%rsp)
  15014b:	mov    %rax,%rsi
  15014e:	mov    %r10,%rax
  150151:	mov    %r9,%r10
  150154:	mov    %rdx,%r9
  150157:	shr    $0x20,%r9
  15015b:	movups (%r14,%r9,1),%xmm0
  150160:	movups %xmm0,(%r14,%r10,1)
  150165:	mov    %r12,%r9
  150168:	shr    $0x20,%r9
  15016c:	movups (%r14,%r9,1),%xmm0
  150171:	mov    -0x10(%rsp),%r9
  150176:	movups %xmm0,(%r8,%r9,1)
  15017b:	mov    %r10,%r8
  15017e:	lea    (%r10,%rdi,1),%r9
  150182:	mov    %rax,%r10
  150185:	mov    %rsi,%rax
  150188:	xor    %esi,%esi
  15018a:	jmp    1501a1 <<mibz2::cells::Kernel>::induce_cells+0x221>
  15018c:	nopl   0x0(%rax)
  150190:	mov    %rdx,(%r14,%rax,1)
  150194:	mov    %r12,0x8(%r14,%rax,1)
  150199:	mov    $0x10,%esi
  15019e:	mov    %rax,%r8
  1501a1:	add    %rsi,%rax
  1501a4:	shl    $0x20,%r8
  1501a8:	or     %rdi,%r8
  1501ab:	mov    %rbx,%rdi
  1501ae:	sub    %r11,%rdi
  1501b1:	je     150229 <<mibz2::cells::Kernel>::induce_cells+0x2a9>
  1501b3:	ja     1500b0 <<mibz2::cells::Kernel>::induce_cells+0x130>
  1501b9:	mov    %ebx,0x4(%r14,%r13,1)
  1501be:	mov    %r8,0x8(%r14,%r13,1)
  1501c3:	mov    %r15,0x10(%r14,%r13,1)
  1501c8:	mov    (%r14,%r15,1),%edx
  1501cc:	mov    -0x50(%rsp),%rsi
  1501d1:	mov    %edx,(%r14,%rsi,1)
  1501d5:	mov    %esi,(%r14,%r15,1)
  1501d9:	mov    %ebx,0x4(%r14,%r15,1)
  1501de:	mov    %r8,0x8(%r14,%r15,1)
  1501e3:	mov    %r13,0x10(%r14,%r15,1)
  1501e8:	mov    %rsi,%r15
  1501eb:	add    $0x18,%rsi
  1501ef:	sub    %rbx,%r11
  1501f2:	mov    -0x110(%r14,%rbp,1),%edx
  1501fa:	prefetcht0 (%r14,%rdx,1)
  1501ff:	mov    -0x114(%r14,%rbp,1),%edx
  150207:	prefetcht0 (%r14,%rdx,1)
  15020c:	mov    0x4(%r14,%rbp,1),%edi
  150211:	mov    0x8(%r14,%rbp,1),%rdx
  150216:	mov    0x10(%r14,%rbp,1),%r13
  15021b:	mov    (%r14,%rbp,1),%ebp
  15021f:	mov    %rsi,-0x50(%rsp)
  150224:	jmp    150118 <<mibz2::cells::Kernel>::induce_cells+0x198>
  150229:	mov    %ebx,0x4(%r14,%r15,1)
  15022e:	mov    %r8,0x8(%r14,%r15,1)
  150233:	mov    %r13,0x10(%r14,%r15,1)
  150238:	mov    %ebx,0x4(%r14,%r13,1)
  15023d:	mov    %r8,0x8(%r14,%r13,1)
  150242:	mov    %r15,0x10(%r14,%r13,1)
  150247:	mov    -0x48(%rsp),%rdi
  15024c:	jmp    150030 <<mibz2::cells::Kernel>::induce_cells+0xb0>
  150251:	mov    -0x50(%rsp),%rcx
  150256:	cmp    -0x20(%rsp),%rcx
  15025b:	ja     1502a8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  15025d:	mov    -0x58(%rsp),%rcx
  150262:	cmp    -0x28(%rsp),%rcx
  150267:	ja     1502a8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150269:	add    $0x20,%r9
  15026d:	cmp    -0x30(%rsp),%r9
  150272:	ja     1502a8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  150274:	lea    0x10(%rax),%rcx
  150278:	cmp    -0x38(%rsp),%rcx
  15027d:	ja     1502a8 <<mibz2::cells::Kernel>::induce_cells+0x328>
  15027f:	mov    -0x40(%rsp),%rcx
  150284:	mov    %rax,0x70(%rcx)
  150288:	mov    %rdx,%rax
  15028b:	shr    $0x20,%rax
  15028f:	mov    %eax,0x8(%rdi)
  150292:	mov    %edx,0xc(%rdi)
  150295:	mov    %r10,0x10(%rdi)
  150299:	xor    %eax,%eax
  15029b:	jmp    1502ae <<mibz2::cells::Kernel>::induce_cells+0x32e>
  15029d:	mov    -0x48(%rsp),%rdi
  1502a2:	movb   $0x2,0x1(%rdi)
  1502a6:	jmp    1502ac <<mibz2::cells::Kernel>::induce_cells+0x32c>
  1502a8:	movb   $0x3,0x1(%rdi)
  1502ac:	mov    $0x1,%al
  1502ae:	pop    %rbx
  1502af:	pop    %r12
  1502b1:	pop    %r13
  1502b3:	pop    %r14
  1502b5:	pop    %r15
  1502b7:	pop    %rbp
  1502b8:	mov    %al,(%rdi)
  1502ba:	ret
