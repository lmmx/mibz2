
base/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

000000000003ab50 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build>:
   3ab50:	push   rbp
   3ab51:	push   r15
   3ab53:	push   r14
   3ab55:	push   r13
   3ab57:	push   r12
   3ab59:	push   rbx
   3ab5a:	sub    rsp,0x808
   3ab61:	mov    r15,rdx
   3ab64:	mov    rbx,rsi
   3ab67:	mov    r14,rdi
   3ab6a:	mov    QWORD PTR [rcx+0x10],0x0
   3ab72:	cmp    rsi,0xffffff
   3ab79:	ja     3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3ab7f:	pxor   xmm0,xmm0
   3ab83:	xor    eax,eax
   3ab85:	pxor   xmm2,xmm2
   3ab89:	pxor   xmm1,xmm1
   3ab8d:	nop    DWORD PTR [rax]
   3ab90:	movq   xmm3,QWORD PTR [r15+rax*4]
   3ab96:	movq   xmm4,QWORD PTR [r15+rax*4+0x8]
   3ab9d:	punpckldq xmm3,xmm0
   3aba1:	paddq  xmm3,xmm2
   3aba5:	punpckldq xmm4,xmm0
   3aba9:	paddq  xmm4,xmm1
   3abad:	movq   xmm2,QWORD PTR [r15+rax*4+0x10]
   3abb4:	movq   xmm1,QWORD PTR [r15+rax*4+0x18]
   3abbb:	punpckldq xmm2,xmm0
   3abbf:	paddq  xmm2,xmm3
   3abc3:	punpckldq xmm1,xmm0
   3abc7:	paddq  xmm1,xmm4
   3abcb:	add    rax,0x8
   3abcf:	cmp    rax,0x100
   3abd5:	jne    3ab90 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x40>
   3abd7:	paddq  xmm1,xmm2
   3abdb:	pshufd xmm0,xmm1,0xee
   3abe0:	paddq  xmm0,xmm1
   3abe4:	movq   rax,xmm0
   3abe9:	cmp    rax,rbx
   3abec:	jne    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3abf2:	mov    r12,rcx
   3abf5:	lea    rdi,[rsp+0x8]
   3abfa:	xor    ebp,ebp
   3abfc:	mov    r13,QWORD PTR [rip+0x5b475]        # 96078 <memset@GLIBC_2.2.5>
   3ac03:	mov    edx,0x400
   3ac08:	xor    esi,esi
   3ac0a:	call   r13
   3ac0d:	lea    rdi,[rsp+0x408]
   3ac15:	mov    edx,0x400
   3ac1a:	xor    esi,esi
   3ac1c:	call   r13
   3ac1f:	mov    eax,0x1
   3ac24:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3ac30:	mov    DWORD PTR [rsp+rax*4+0x404],ebp
   3ac37:	add    ebp,DWORD PTR [r15+rax*4-0x4]
   3ac3c:	mov    DWORD PTR [rsp+rax*4+0x4],ebp
   3ac40:	mov    DWORD PTR [rsp+rax*4+0x408],ebp
   3ac47:	add    ebp,DWORD PTR [r15+rax*4]
   3ac4b:	mov    DWORD PTR [rsp+rax*4+0x8],ebp
   3ac4f:	add    rax,0x2
   3ac53:	cmp    rax,0x101
   3ac59:	jne    3ac30 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe0>
   3ac5b:	mov    rcx,r12
   3ac5e:	cmp    rbx,QWORD PTR [r12]
   3ac62:	ja     3adb6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x266>
   3ac68:	mov    rdx,QWORD PTR [rcx+0x8]
   3ac6c:	xor    r8d,r8d
   3ac6f:	cmp    rbx,0x10
   3ac73:	jae    3ade2 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x292>
   3ac79:	cmp    r8,rbx
   3ac7c:	jae    3acd7 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x187>
   3ac7e:	mov    eax,r8d
   3ac81:	shl    eax,0x8
   3ac84:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3ac90:	movzx  esi,BYTE PTR [r14+r8*1]
   3ac95:	lea    r9d,[rsi*4+0x0]
   3ac9d:	mov    edi,DWORD PTR [rsp+r9*1+0x408]
   3aca5:	cmp    edi,DWORD PTR [rsp+r9*1+0x8]
   3acaa:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3acac:	cmp    rbx,rdi
   3acaf:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3acb5:	movzx  r9d,BYTE PTR [r14+rdi*1]
   3acba:	add    r9d,eax
   3acbd:	mov    DWORD PTR [rdx+rdi*4],r9d
   3acc1:	inc    edi
   3acc3:	mov    DWORD PTR [rsp+rsi*4+0x408],edi
   3acca:	inc    r8
   3accd:	add    eax,0x100
   3acd2:	cmp    rbx,r8
   3acd5:	jne    3ac90 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x140>
   3acd7:	lea    rdi,[rsp+0x408]
   3acdf:	lea    rsi,[rsp+0x8]
   3ace4:	mov    edx,0x400
   3ace9:	call   QWORD PTR [rip+0x5b429]        # 96118 <bcmp@GLIBC_2.2.5>
   3acef:	mov    rcx,r12
   3acf2:	test   eax,eax
   3acf4:	je     3ad20 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1d0>
   3acf6:	mov    QWORD PTR [rcx+0x10],0x0
   3acfe:	cmp    rbx,QWORD PTR [rcx]
   3ad01:	ja     3ad29 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1d9>
   3ad03:	mov    r13,QWORD PTR [rcx+0x8]
   3ad07:	xor    r12d,r12d
   3ad0a:	test   rbx,rbx
   3ad0d:	je     3b47a <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x92a>
   3ad13:	cmp    rbx,0x8
   3ad17:	jae    3ad57 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x207>
   3ad19:	xor    eax,eax
   3ad1b:	jmp    3b465 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x915>
   3ad20:	mov    QWORD PTR [rcx+0x10],rbx
   3ad24:	jmp    3b453 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x903>
   3ad29:	mov    r12,rcx
   3ad2c:	mov    ecx,0x4
   3ad31:	mov    r8d,0x4
   3ad37:	mov    rdi,r12
   3ad3a:	xor    esi,esi
   3ad3c:	mov    rdx,rbx
   3ad3f:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   3ad44:	mov    rcx,r12
   3ad47:	mov    r13,QWORD PTR [r12+0x8]
   3ad4c:	mov    r12,QWORD PTR [r12+0x10]
   3ad51:	cmp    rbx,0x8
   3ad55:	jb     3ad19 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1c9>
   3ad57:	movabs rax,0x7ffffffffffffff0
   3ad61:	or     rax,0x8
   3ad65:	and    rax,rbx
   3ad68:	lea    rdx,[r12*4+0x10]
   3ad70:	add    rdx,r13
   3ad73:	add    r12,rax
   3ad76:	xor    esi,esi
   3ad78:	pxor   xmm0,xmm0
   3ad7c:	nop    DWORD PTR [rax+0x0]
   3ad80:	movd   xmm1,DWORD PTR [r14+rsi*1]
   3ad86:	movd   xmm2,DWORD PTR [r14+rsi*1+0x4]
   3ad8d:	punpcklbw xmm1,xmm0
   3ad91:	punpcklwd xmm1,xmm0
   3ad95:	punpcklbw xmm2,xmm0
   3ad99:	punpcklwd xmm2,xmm0
   3ad9d:	movdqu XMMWORD PTR [rdx+rsi*4-0x10],xmm1
   3ada3:	movdqu XMMWORD PTR [rdx+rsi*4],xmm2
   3ada8:	add    rsi,0x8
   3adac:	cmp    rax,rsi
   3adaf:	jne    3ad80 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x230>
   3adb1:	jmp    3b475 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x925>
   3adb6:	mov    ecx,0x4
   3adbb:	mov    r8d,0x4
   3adc1:	mov    rdi,r12
   3adc4:	xor    esi,esi
   3adc6:	mov    rdx,rbx
   3adc9:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   3adce:	mov    rcx,r12
   3add1:	mov    rdx,QWORD PTR [rcx+0x8]
   3add5:	xor    r8d,r8d
   3add8:	cmp    rbx,0x10
   3addc:	jb     3ac79 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x129>
   3ade2:	mov    esi,0x8
   3ade7:	pxor   xmm0,xmm0
   3adeb:	movdqa xmm1,XMMWORD PTR [rip+0xfffffffffffccd2d]        # 7b20 <GCC_except_table701+0x58>
   3adf3:	movdqa xmm2,XMMWORD PTR [rip+0xfffffffffffccef5]        # 7cf0 <GCC_except_table701+0x228>
   3adfb:	movdqa xmm3,XMMWORD PTR [rip+0xfffffffffffcd0fd]        # 7f00 <GCC_except_table701+0x438>
   3ae03:	jmp    3ae2b <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x2db>
   3ae05:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   3ae10:	lea    rax,[rsi+0x10]
   3ae14:	add    rsi,0x18
   3ae18:	add    r8d,0x1000
   3ae1f:	cmp    rsi,rbx
   3ae22:	mov    rsi,rax
   3ae25:	ja     3b3e7 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x897>
   3ae2b:	lea    rax,[rsi-0x8]
   3ae2f:	cmp    rsi,rbx
   3ae32:	ja     3b427 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8d7>
   3ae38:	lea    rdi,[rsi+0x7]
   3ae3c:	cmp    rdi,rbx
   3ae3f:	jae    3b3fc <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8ac>
   3ae45:	mov    r9,QWORD PTR [r14+rsi*1-0x8]
   3ae4a:	mov    rax,QWORD PTR [r14+rsi*1]
   3ae4e:	mov    rdi,r9
   3ae51:	xor    rdi,rax
   3ae54:	mov    r10,r9
   3ae57:	rol    r10,0x8
   3ae5b:	xor    r10,r9
   3ae5e:	or     r10,rdi
   3ae61:	jne    3af50 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x400>
   3ae67:	movzx  r9d,r9b
   3ae6b:	lea    r10d,[r9*4+0x0]
   3ae73:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3ae7b:	lea    rax,[rdi+0x10]
   3ae7f:	mov    r10d,DWORD PTR [rsp+r10*1+0x8]
   3ae84:	cmp    rax,r10
   3ae87:	ja     3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3ae8d:	cmp    rax,rbx
   3ae90:	ja     3b406 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8b6>
   3ae96:	movzx  eax,BYTE PTR [r14+rdi*1]
   3ae9b:	add    eax,r8d
   3ae9e:	mov    DWORD PTR [rdx+rdi*4],eax
   3aea1:	movd   xmm4,DWORD PTR [r14+rdi*1+0x1]
   3aea8:	punpcklbw xmm4,xmm0
   3aeac:	punpcklwd xmm4,xmm0
   3aeb0:	movd   xmm5,r8d
   3aeb5:	pshufd xmm5,xmm5,0x0
   3aeba:	por    xmm4,xmm5
   3aebe:	por    xmm4,xmm1
   3aec2:	movdqu XMMWORD PTR [rdx+rdi*4+0x4],xmm4
   3aec8:	movd   xmm4,DWORD PTR [r14+rdi*1+0x5]
   3aecf:	punpcklbw xmm4,xmm0
   3aed3:	punpcklwd xmm4,xmm0
   3aed7:	por    xmm4,xmm5
   3aedb:	por    xmm4,xmm2
   3aedf:	movdqu XMMWORD PTR [rdx+rdi*4+0x14],xmm4
   3aee5:	movd   xmm4,DWORD PTR [r14+rdi*1+0x9]
   3aeec:	punpcklbw xmm4,xmm0
   3aef0:	punpcklwd xmm4,xmm0
   3aef4:	por    xmm4,xmm5
   3aef8:	por    xmm4,xmm3
   3aefc:	movdqu XMMWORD PTR [rdx+rdi*4+0x24],xmm4
   3af02:	movzx  eax,BYTE PTR [r14+rdi*1+0xd]
   3af08:	lea    eax,[r8+rax*1+0xd00]
   3af10:	mov    DWORD PTR [rdx+rdi*4+0x34],eax
   3af14:	movzx  eax,BYTE PTR [r14+rdi*1+0xe]
   3af1a:	lea    eax,[r8+rax*1+0xe00]
   3af22:	mov    DWORD PTR [rdx+rdi*4+0x38],eax
   3af26:	movzx  eax,BYTE PTR [r14+rdi*1+0xf]
   3af2c:	lea    eax,[r8+rax*1+0xf00]
   3af34:	mov    DWORD PTR [rdx+rdi*4+0x3c],eax
   3af38:	lea    eax,[rdi+0x10]
   3af3b:	mov    DWORD PTR [rsp+r9*4+0x408],eax
   3af43:	jmp    3ae10 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x2c0>
   3af48:	nop    DWORD PTR [rax+rax*1+0x0]
   3af50:	movzx  r10d,r9b
   3af54:	lea    r11d,[r10*4+0x0]
   3af5c:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3af64:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3af69:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3af6f:	cmp    rbx,rdi
   3af72:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3af78:	mov    r11,r9
   3af7b:	shr    r11,0x8
   3af7f:	movzx  ebp,BYTE PTR [r14+rdi*1]
   3af84:	add    ebp,r8d
   3af87:	mov    DWORD PTR [rdx+rdi*4],ebp
   3af8a:	inc    edi
   3af8c:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3af94:	movzx  r10d,r11b
   3af98:	lea    r11d,[r10*4+0x0]
   3afa0:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3afa8:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3afad:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3afb3:	cmp    rbx,rdi
   3afb6:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3afbc:	mov    r11,r9
   3afbf:	shr    r11,0x10
   3afc3:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3afc8:	lea    ebp,[r8+r13*1]
   3afcc:	add    ebp,0x100
   3afd2:	mov    DWORD PTR [rdx+rdi*4],ebp
   3afd5:	inc    edi
   3afd7:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3afdf:	movzx  r10d,r11b
   3afe3:	lea    r11d,[r10*4+0x0]
   3afeb:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3aff3:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3aff8:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3affe:	cmp    rbx,rdi
   3b001:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b007:	mov    r11,r9
   3b00a:	shr    r11,0x18
   3b00e:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3b013:	lea    ebp,[r8+r13*1]
   3b017:	add    ebp,0x200
   3b01d:	mov    DWORD PTR [rdx+rdi*4],ebp
   3b020:	inc    edi
   3b022:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3b02a:	movzx  r10d,r11b
   3b02e:	lea    r11d,[r10*4+0x0]
   3b036:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3b03e:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3b043:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b049:	cmp    rbx,rdi
   3b04c:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b052:	mov    r11,r9
   3b055:	shr    r11,0x20
   3b059:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3b05e:	lea    ebp,[r8+r13*1]
   3b062:	add    ebp,0x300
   3b068:	mov    DWORD PTR [rdx+rdi*4],ebp
   3b06b:	inc    edi
   3b06d:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3b075:	movzx  r10d,r11b
   3b079:	lea    r11d,[r10*4+0x0]
   3b081:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3b089:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3b08e:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b094:	cmp    rbx,rdi
   3b097:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b09d:	mov    r11,r9
   3b0a0:	shr    r11,0x28
   3b0a4:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3b0a9:	lea    ebp,[r8+r13*1]
   3b0ad:	add    ebp,0x400
   3b0b3:	mov    DWORD PTR [rdx+rdi*4],ebp
   3b0b6:	inc    edi
   3b0b8:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3b0c0:	movzx  r10d,r11b
   3b0c4:	lea    r11d,[r10*4+0x0]
   3b0cc:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3b0d4:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3b0d9:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b0df:	cmp    rbx,rdi
   3b0e2:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b0e8:	mov    r11,r9
   3b0eb:	shr    r11,0x30
   3b0ef:	movzx  r13d,BYTE PTR [r14+rdi*1]
   3b0f4:	lea    ebp,[r8+r13*1]
   3b0f8:	add    ebp,0x500
   3b0fe:	mov    DWORD PTR [rdx+rdi*4],ebp
   3b101:	inc    edi
   3b103:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3b10b:	movzx  r10d,r11b
   3b10f:	lea    r11d,[r10*4+0x0]
   3b117:	mov    edi,DWORD PTR [rsp+r11*1+0x408]
   3b11f:	cmp    edi,DWORD PTR [rsp+r11*1+0x8]
   3b124:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b12a:	cmp    rbx,rdi
   3b12d:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b133:	shr    r9,0x38
   3b137:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b13c:	add    r11d,r8d
   3b13f:	add    r11d,0x600
   3b146:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b14a:	inc    edi
   3b14c:	mov    DWORD PTR [rsp+r10*4+0x408],edi
   3b154:	lea    r10d,[r9*4+0x0]
   3b15c:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b164:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b169:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b16f:	cmp    rbx,rdi
   3b172:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b178:	movzx  r10d,BYTE PTR [r14+rdi*1]
   3b17d:	add    r10d,r8d
   3b180:	add    r10d,0x700
   3b187:	mov    DWORD PTR [rdx+rdi*4],r10d
   3b18b:	inc    edi
   3b18d:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b195:	movzx  r9d,al
   3b199:	lea    r10d,[r9*4+0x0]
   3b1a1:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b1a9:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b1ae:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b1b4:	cmp    rbx,rdi
   3b1b7:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b1bd:	mov    r10,rax
   3b1c0:	shr    r10,0x8
   3b1c4:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b1c9:	add    r11d,r8d
   3b1cc:	add    r11d,0x800
   3b1d3:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b1d7:	inc    edi
   3b1d9:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b1e1:	movzx  r9d,r10b
   3b1e5:	lea    r10d,[r9*4+0x0]
   3b1ed:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b1f5:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b1fa:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b200:	cmp    rbx,rdi
   3b203:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b209:	mov    r10,rax
   3b20c:	shr    r10,0x10
   3b210:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b215:	add    r11d,r8d
   3b218:	add    r11d,0x900
   3b21f:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b223:	inc    edi
   3b225:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b22d:	movzx  r9d,r10b
   3b231:	lea    r10d,[r9*4+0x0]
   3b239:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b241:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b246:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b24c:	cmp    rbx,rdi
   3b24f:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b255:	mov    r10,rax
   3b258:	shr    r10,0x18
   3b25c:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b261:	add    r11d,r8d
   3b264:	add    r11d,0xa00
   3b26b:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b26f:	inc    edi
   3b271:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b279:	movzx  r9d,r10b
   3b27d:	lea    r10d,[r9*4+0x0]
   3b285:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b28d:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b292:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b298:	cmp    rbx,rdi
   3b29b:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b2a1:	mov    r10,rax
   3b2a4:	shr    r10,0x20
   3b2a8:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b2ad:	add    r11d,r8d
   3b2b0:	add    r11d,0xb00
   3b2b7:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b2bb:	inc    edi
   3b2bd:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b2c5:	movzx  r9d,r10b
   3b2c9:	lea    r10d,[r9*4+0x0]
   3b2d1:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b2d9:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b2de:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b2e4:	cmp    rbx,rdi
   3b2e7:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b2ed:	mov    r10,rax
   3b2f0:	shr    r10,0x28
   3b2f4:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b2f9:	add    r11d,r8d
   3b2fc:	add    r11d,0xc00
   3b303:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b307:	inc    edi
   3b309:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b311:	movzx  r9d,r10b
   3b315:	lea    r10d,[r9*4+0x0]
   3b31d:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b325:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b32a:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b330:	cmp    rbx,rdi
   3b333:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b339:	mov    r10,rax
   3b33c:	shr    r10,0x30
   3b340:	movzx  r11d,BYTE PTR [r14+rdi*1]
   3b345:	add    r11d,r8d
   3b348:	add    r11d,0xd00
   3b34f:	mov    DWORD PTR [rdx+rdi*4],r11d
   3b353:	inc    edi
   3b355:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b35d:	movzx  r9d,r10b
   3b361:	lea    r10d,[r9*4+0x0]
   3b369:	mov    edi,DWORD PTR [rsp+r10*1+0x408]
   3b371:	cmp    edi,DWORD PTR [rsp+r10*1+0x8]
   3b376:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b37c:	cmp    rbx,rdi
   3b37f:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b385:	shr    rax,0x38
   3b389:	movzx  r10d,BYTE PTR [r14+rdi*1]
   3b38e:	add    r10d,r8d
   3b391:	add    r10d,0xe00
   3b398:	mov    DWORD PTR [rdx+rdi*4],r10d
   3b39c:	inc    edi
   3b39e:	mov    DWORD PTR [rsp+r9*4+0x408],edi
   3b3a6:	lea    r9d,[rax*4+0x0]
   3b3ae:	mov    edi,DWORD PTR [rsp+r9*1+0x408]
   3b3b6:	cmp    edi,DWORD PTR [rsp+r9*1+0x8]
   3b3bb:	jae    3acf6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x1a6>
   3b3c1:	cmp    rbx,rdi
   3b3c4:	jbe    3b418 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8c8>
   3b3c6:	movzx  r9d,BYTE PTR [r14+rdi*1]
   3b3cb:	add    r9d,r8d
   3b3ce:	add    r9d,0xf00
   3b3d5:	mov    DWORD PTR [rdx+rdi*4],r9d
   3b3d9:	inc    edi
   3b3db:	mov    DWORD PTR [rsp+rax*4+0x408],edi
   3b3e2:	jmp    3ae10 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x2c0>
   3b3e7:	add    rax,0xfffffffffffffff8
   3b3eb:	mov    r8,rax
   3b3ee:	cmp    r8,rbx
   3b3f1:	jb     3ac7e <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x12e>
   3b3f7:	jmp    3acd7 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x187>
   3b3fc:	mov    edi,ebx
   3b3fe:	and    edi,0xfffff0
   3b404:	jmp    3b439 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8e9>
   3b406:	lea    rcx,[rip+0x58fdb]        # 943e8 <__frame_dummy_init_array_entry+0x1100>
   3b40d:	mov    rsi,rax
   3b410:	mov    rdx,rbx
   3b413:	call   2bce0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3b418:	lea    rdx,[rip+0x58c81]        # 940a0 <__frame_dummy_init_array_entry+0xdb8>
   3b41f:	mov    rsi,rbx
   3b422:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3b427:	lea    rcx,[rip+0x5901a]        # 94448 <__frame_dummy_init_array_entry+0x1160>
   3b42e:	mov    rdi,rax
   3b431:	mov    rdx,rbx
   3b434:	call   2bce0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3b439:	or     rdi,0x8
   3b43d:	add    rax,0x10
   3b441:	lea    rcx,[rip+0x59000]        # 94448 <__frame_dummy_init_array_entry+0x1160>
   3b448:	mov    rsi,rax
   3b44b:	mov    rdx,rbx
   3b44e:	call   2bce0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3b453:	add    rsp,0x808
   3b45a:	pop    rbx
   3b45b:	pop    r12
   3b45d:	pop    r13
   3b45f:	pop    r14
   3b461:	pop    r15
   3b463:	pop    rbp
   3b464:	ret
   3b465:	movzx  edx,BYTE PTR [r14+rax*1]
   3b46a:	mov    DWORD PTR [r13+r12*4+0x0],edx
   3b46f:	inc    r12
   3b472:	inc    rax
   3b475:	cmp    rbx,rax
   3b478:	jne    3b465 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x915>
   3b47a:	mov    QWORD PTR [rcx+0x10],r12
   3b47e:	lea    rdi,[rsp+0x8]
   3b483:	xor    ebp,ebp
   3b485:	mov    edx,0x400
   3b48a:	xor    esi,esi
   3b48c:	call   QWORD PTR [rip+0x5abe6]        # 96078 <memset@GLIBC_2.2.5>
   3b492:	xor    eax,eax
   3b494:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   3b4a0:	mov    DWORD PTR [rsp+rbp*4+0x8],eax
   3b4a4:	add    eax,DWORD PTR [r15+rbp*4]
   3b4a8:	mov    DWORD PTR [rsp+rbp*4+0xc],eax
   3b4ac:	add    eax,DWORD PTR [r15+rbp*4+0x4]
   3b4b1:	mov    DWORD PTR [rsp+rbp*4+0x10],eax
   3b4b5:	add    eax,DWORD PTR [r15+rbp*4+0x8]
   3b4ba:	mov    DWORD PTR [rsp+rbp*4+0x14],eax
   3b4be:	add    eax,DWORD PTR [r15+rbp*4+0xc]
   3b4c3:	add    rbp,0x4
   3b4c7:	cmp    rbp,0x100
   3b4ce:	jne    3b4a0 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x950>
   3b4d0:	xor    ecx,ecx
   3b4d2:	cmp    rbx,0x10
   3b4d6:	jb     3b96a <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe1a>
   3b4dc:	mov    esi,0x8
   3b4e1:	movdqa xmm0,XMMWORD PTR [rip+0xfffffffffffcc997]        # 7e80 <GCC_except_table701+0x3b8>
   3b4e9:	movdqa xmm1,XMMWORD PTR [rip+0xfffffffffffcc62f]        # 7b20 <GCC_except_table701+0x58>
   3b4f1:	movdqa xmm2,XMMWORD PTR [rip+0xfffffffffffcc7f7]        # 7cf0 <GCC_except_table701+0x228>
   3b4f9:	movdqa xmm3,XMMWORD PTR [rip+0xfffffffffffcc9ff]        # 7f00 <GCC_except_table701+0x438>
   3b501:	movq   xmm4,QWORD PTR [rip+0xfffffffffffcc787]        # 7c90 <GCC_except_table701+0x1c8>
   3b509:	movq   xmm5,QWORD PTR [rip+0xfffffffffffcc78f]        # 7ca0 <GCC_except_table701+0x1d8>
   3b511:	jmp    3b5ed <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xa9d>
   3b516:	cs nop WORD PTR [rax+rax*1+0x0]
   3b520:	movzx  edx,dl
   3b523:	mov    eax,edx
   3b525:	mov    edi,DWORD PTR [rsp+rax*4+0x8]
   3b529:	lea    rax,[rdi+0x10]
   3b52d:	cmp    rax,r12
   3b530:	ja     3b9c6 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe76>
   3b536:	movzx  eax,BYTE PTR [r13+rdi*4+0x0]
   3b53c:	add    eax,ecx
   3b53e:	mov    DWORD PTR [r13+rdi*4+0x0],eax
   3b543:	movdqu xmm6,XMMWORD PTR [r13+rdi*4+0x4]
   3b54a:	movdqu xmm7,XMMWORD PTR [r13+rdi*4+0x14]
   3b551:	movdqu xmm8,XMMWORD PTR [r13+rdi*4+0x24]
   3b558:	pand   xmm6,xmm0
   3b55c:	movd   xmm9,ecx
   3b561:	pshufd xmm9,xmm9,0x0
   3b567:	por    xmm6,xmm9
   3b56c:	por    xmm6,xmm1
   3b570:	movdqu XMMWORD PTR [r13+rdi*4+0x4],xmm6
   3b577:	pand   xmm7,xmm0
   3b57b:	por    xmm7,xmm9
   3b580:	por    xmm7,xmm2
   3b584:	movdqu XMMWORD PTR [r13+rdi*4+0x14],xmm7
   3b58b:	pand   xmm8,xmm0
   3b590:	por    xmm8,xmm9
   3b595:	por    xmm8,xmm3
   3b59a:	movdqu XMMWORD PTR [r13+rdi*4+0x24],xmm8
   3b5a1:	movq   xmm6,QWORD PTR [r13+rdi*4+0x34]
   3b5a8:	pand   xmm6,xmm4
   3b5ac:	por    xmm6,xmm9
   3b5b1:	por    xmm6,xmm5
   3b5b5:	movq   QWORD PTR [r13+rdi*4+0x34],xmm6
   3b5bc:	movzx  eax,BYTE PTR [r13+rdi*4+0x3c]
   3b5c2:	add    eax,ecx
   3b5c4:	add    eax,0xf00
   3b5c9:	mov    DWORD PTR [r13+rdi*4+0x3c],eax
   3b5ce:	add    DWORD PTR [rsp+rdx*4+0x8],0x10
   3b5d3:	lea    rax,[rsi+0x10]
   3b5d7:	add    rsi,0x18
   3b5db:	add    ecx,0x1000
   3b5e1:	cmp    rsi,rbx
   3b5e4:	mov    rsi,rax
   3b5e7:	ja     3b963 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe13>
   3b5ed:	lea    rax,[rsi-0x8]
   3b5f1:	cmp    rsi,rbx
   3b5f4:	ja     3b427 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8d7>
   3b5fa:	lea    rdx,[rsi+0x7]
   3b5fe:	cmp    rdx,rbx
   3b601:	jae    3b9b4 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe64>
   3b607:	mov    rdx,QWORD PTR [r14+rsi*1-0x8]
   3b60c:	mov    rax,QWORD PTR [r14+rsi*1]
   3b610:	mov    rdi,rdx
   3b613:	xor    rdi,rax
   3b616:	mov    r8,rdx
   3b619:	rol    r8,0x8
   3b61d:	xor    r8,rdx
   3b620:	or     r8,rdi
   3b623:	je     3b520 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x9d0>
   3b629:	movzx  r8d,dl
   3b62d:	mov    edi,r8d
   3b630:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b634:	cmp    r12,rdi
   3b637:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b63d:	mov    r9,rdx
   3b640:	shr    r9,0x8
   3b644:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b64a:	add    r10d,ecx
   3b64d:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b652:	inc    edi
   3b654:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b659:	movzx  r8d,r9b
   3b65d:	mov    edi,r8d
   3b660:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b664:	cmp    r12,rdi
   3b667:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b66d:	mov    r9,rdx
   3b670:	shr    r9,0x10
   3b674:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b67a:	add    r10d,ecx
   3b67d:	add    r10d,0x100
   3b684:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b689:	inc    edi
   3b68b:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b690:	movzx  r8d,r9b
   3b694:	mov    edi,r8d
   3b697:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b69b:	cmp    r12,rdi
   3b69e:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b6a4:	mov    r9,rdx
   3b6a7:	shr    r9,0x18
   3b6ab:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b6b1:	add    r10d,ecx
   3b6b4:	add    r10d,0x200
   3b6bb:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b6c0:	inc    edi
   3b6c2:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b6c7:	movzx  r8d,r9b
   3b6cb:	mov    edi,r8d
   3b6ce:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b6d2:	cmp    r12,rdi
   3b6d5:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b6db:	mov    r9,rdx
   3b6de:	shr    r9,0x20
   3b6e2:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b6e8:	add    r10d,ecx
   3b6eb:	add    r10d,0x300
   3b6f2:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b6f7:	inc    edi
   3b6f9:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b6fe:	movzx  r8d,r9b
   3b702:	mov    edi,r8d
   3b705:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b709:	cmp    r12,rdi
   3b70c:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b712:	mov    r9,rdx
   3b715:	shr    r9,0x28
   3b719:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b71f:	add    r10d,ecx
   3b722:	add    r10d,0x400
   3b729:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b72e:	inc    edi
   3b730:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b735:	movzx  r8d,r9b
   3b739:	mov    edi,r8d
   3b73c:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b740:	cmp    r12,rdi
   3b743:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b749:	mov    r9,rdx
   3b74c:	shr    r9,0x30
   3b750:	movzx  r10d,BYTE PTR [r13+rdi*4+0x0]
   3b756:	add    r10d,ecx
   3b759:	add    r10d,0x500
   3b760:	mov    DWORD PTR [r13+rdi*4+0x0],r10d
   3b765:	inc    edi
   3b767:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b76c:	movzx  r8d,r9b
   3b770:	mov    edi,r8d
   3b773:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b777:	cmp    r12,rdi
   3b77a:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b780:	shr    rdx,0x38
   3b784:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b78a:	add    r9d,ecx
   3b78d:	add    r9d,0x600
   3b794:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b799:	inc    edi
   3b79b:	mov    DWORD PTR [rsp+r8*4+0x8],edi
   3b7a0:	mov    edi,DWORD PTR [rsp+rdx*4+0x8]
   3b7a4:	cmp    r12,rdi
   3b7a7:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b7ad:	movzx  r8d,BYTE PTR [r13+rdi*4+0x0]
   3b7b3:	add    r8d,ecx
   3b7b6:	add    r8d,0x700
   3b7bd:	mov    DWORD PTR [r13+rdi*4+0x0],r8d
   3b7c2:	inc    edi
   3b7c4:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b7c8:	movzx  edx,al
   3b7cb:	mov    edi,edx
   3b7cd:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b7d1:	cmp    r12,rdi
   3b7d4:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b7da:	mov    r8,rax
   3b7dd:	shr    r8,0x8
   3b7e1:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b7e7:	add    r9d,ecx
   3b7ea:	add    r9d,0x800
   3b7f1:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b7f6:	inc    edi
   3b7f8:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b7fc:	movzx  edx,r8b
   3b800:	mov    edi,edx
   3b802:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b806:	cmp    r12,rdi
   3b809:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b80f:	mov    r8,rax
   3b812:	shr    r8,0x10
   3b816:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b81c:	add    r9d,ecx
   3b81f:	add    r9d,0x900
   3b826:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b82b:	inc    edi
   3b82d:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b831:	movzx  edx,r8b
   3b835:	mov    edi,edx
   3b837:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b83b:	cmp    r12,rdi
   3b83e:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b844:	mov    r8,rax
   3b847:	shr    r8,0x18
   3b84b:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b851:	add    r9d,ecx
   3b854:	add    r9d,0xa00
   3b85b:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b860:	inc    edi
   3b862:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b866:	movzx  edx,r8b
   3b86a:	mov    edi,edx
   3b86c:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b870:	cmp    r12,rdi
   3b873:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b879:	mov    r8,rax
   3b87c:	shr    r8,0x20
   3b880:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b886:	add    r9d,ecx
   3b889:	add    r9d,0xb00
   3b890:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b895:	inc    edi
   3b897:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b89b:	movzx  edx,r8b
   3b89f:	mov    edi,edx
   3b8a1:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b8a5:	cmp    r12,rdi
   3b8a8:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b8ae:	mov    r8,rax
   3b8b1:	shr    r8,0x28
   3b8b5:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b8bb:	add    r9d,ecx
   3b8be:	add    r9d,0xc00
   3b8c5:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b8ca:	inc    edi
   3b8cc:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b8d0:	movzx  edx,r8b
   3b8d4:	mov    edi,edx
   3b8d6:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b8da:	cmp    r12,rdi
   3b8dd:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b8e3:	mov    r8,rax
   3b8e6:	shr    r8,0x30
   3b8ea:	movzx  r9d,BYTE PTR [r13+rdi*4+0x0]
   3b8f0:	add    r9d,ecx
   3b8f3:	add    r9d,0xd00
   3b8fa:	mov    DWORD PTR [r13+rdi*4+0x0],r9d
   3b8ff:	inc    edi
   3b901:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b905:	movzx  edx,r8b
   3b909:	mov    edi,edx
   3b90b:	mov    edi,DWORD PTR [rsp+rdi*4+0x8]
   3b90f:	cmp    r12,rdi
   3b912:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b918:	shr    rax,0x38
   3b91c:	movzx  r8d,BYTE PTR [r13+rdi*4+0x0]
   3b922:	add    r8d,ecx
   3b925:	add    r8d,0xe00
   3b92c:	mov    DWORD PTR [r13+rdi*4+0x0],r8d
   3b931:	inc    edi
   3b933:	mov    DWORD PTR [rsp+rdx*4+0x8],edi
   3b937:	mov    edi,DWORD PTR [rsp+rax*4+0x8]
   3b93b:	cmp    r12,rdi
   3b93e:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b944:	movzx  edx,BYTE PTR [r13+rdi*4+0x0]
   3b94a:	add    edx,ecx
   3b94c:	add    edx,0xf00
   3b952:	mov    DWORD PTR [r13+rdi*4+0x0],edx
   3b957:	lea    edx,[rdi+0x1]
   3b95a:	mov    DWORD PTR [rsp+rax*4+0x8],edx
   3b95e:	jmp    3b5d3 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xa83>
   3b963:	add    rax,0xfffffffffffffff8
   3b967:	mov    rcx,rax
   3b96a:	cmp    rcx,rbx
   3b96d:	jae    3b453 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x903>
   3b973:	mov    eax,ecx
   3b975:	shl    eax,0x8
   3b978:	nop    DWORD PTR [rax+rax*1+0x0]
   3b980:	movzx  edx,BYTE PTR [r14+rcx*1]
   3b985:	mov    edi,DWORD PTR [rsp+rdx*4+0x8]
   3b989:	cmp    r12,rdi
   3b98c:	jbe    3b9d8 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe88>
   3b98e:	inc    rcx
   3b991:	movzx  esi,BYTE PTR [r13+rdi*4+0x0]
   3b997:	add    esi,eax
   3b999:	mov    DWORD PTR [r13+rdi*4+0x0],esi
   3b99e:	lea    esi,[rdi+0x1]
   3b9a1:	mov    DWORD PTR [rsp+rdx*4+0x8],esi
   3b9a5:	add    eax,0x100
   3b9aa:	cmp    rbx,rcx
   3b9ad:	jne    3b980 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0xe30>
   3b9af:	jmp    3b453 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x903>
   3b9b4:	movabs rdi,0x7ffffffffffffff0
   3b9be:	and    rdi,rbx
   3b9c1:	jmp    3b439 <_RNvNtCsdtMXQ4ssD8O_5mibz23bwt5build+0x8e9>
   3b9c6:	lea    rcx,[rip+0x58a33]        # 94400 <__frame_dummy_init_array_entry+0x1118>
   3b9cd:	mov    rsi,rax
   3b9d0:	mov    rdx,r12
   3b9d3:	call   2bce0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   3b9d8:	lea    rdx,[rip+0x58a39]        # 94418 <__frame_dummy_init_array_entry+0x1130>
   3b9df:	mov    rsi,r12
   3b9e2:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
