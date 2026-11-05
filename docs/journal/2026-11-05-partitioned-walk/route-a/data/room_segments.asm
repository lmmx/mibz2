
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/target/release/examples/phases:     file format elf64-x86-64


Disassembly of section .text:

00000000000299f0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   299f0:	push   rbp
   299f1:	push   r15
   299f3:	push   r14
   299f5:	push   r13
   299f7:	push   r12
   299f9:	push   rbx
   299fa:	sub    rsp,0x1000
   29a01:	mov    QWORD PTR [rsp],0x0
   29a09:	sub    rsp,0x1000
   29a10:	mov    QWORD PTR [rsp],0x0
   29a18:	sub    rsp,0x638
   29a1f:	mov    QWORD PTR [rsp+0xf0],rsi
   29a27:	cmp    rsi,0x800001
   29a2e:	jae    2af5e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x156e>
   29a34:	mov    r15,rdi
   29a37:	mov    edi,edx
   29a39:	cmp    rsi,rdi
   29a3c:	jbe    2b20e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181e>
   29a42:	mov    QWORD PTR [rsp+0x50],rsi
   29a47:	mov    esi,DWORD PTR [r15+rdi*4]
   29a4b:	mov    QWORD PTR [rcx+0x10],0x0
   29a53:	movabs r14,0x2545f4914f6cdd1d
   29a5d:	cmp    QWORD PTR [rcx],0x1fe
   29a64:	jbe    2af98 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15a8>
   29a6a:	xor    r9d,r9d
   29a6d:	shr    esi,0x8
   29a70:	mov    rdi,QWORD PTR [rsp+0x50]
   29a75:	xor    r14,rdi
   29a78:	mov    QWORD PTR [rsp+0x20],rcx
   29a7d:	mov    rbx,QWORD PTR [rcx+0x8]
   29a81:	mov    ecx,0x1ff
   29a86:	mov    r12,r9
   29a89:	nop    DWORD PTR [rax+0x0]
   29a90:	mov    rbp,r12
   29a93:	mov    rax,r14
   29a96:	shl    rax,0xd
   29a9a:	xor    rax,r14
   29a9d:	mov    rdx,rax
   29aa0:	shr    rdx,0x7
   29aa4:	xor    rdx,rax
   29aa7:	mov    r14,rdx
   29aaa:	shl    r14,0x11
   29aae:	xor    r14,rdx
   29ab1:	mov    rax,r14
   29ab4:	xor    edx,edx
   29ab6:	div    rdi
   29ab9:	mov    DWORD PTR [rbx+r12*4],edx
   29abd:	inc    r12
   29ac0:	dec    rcx
   29ac3:	jne    29a90 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xa0>
   29ac5:	mov    r14,QWORD PTR [rsp+0x20]
   29aca:	mov    QWORD PTR [r14+0x10],r12
   29ace:	cmp    r12,QWORD PTR [r14]
   29ad1:	mov    DWORD PTR [rsp+0x4c],esi
   29ad5:	je     2ac0c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x121c>
   29adb:	mov    DWORD PTR [rbx+r12*4],esi
   29adf:	lea    rax,[rbp+0x2]
   29ae3:	mov    r13,rax
   29ae6:	mov    QWORD PTR [r14+0x10],rax
   29aea:	cmp    rbp,0xfffffffffffffffd
   29aee:	jbe    2afc5 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15d5>
   29af4:	mov    rbp,QWORD PTR [rsp+0x20]
   29af9:	mov    rsi,QWORD PTR [rsp+0x50]
   29afe:	test   r13,r13
   29b01:	je     2b2b7 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18c7>
   29b07:	lea    rax,[r13*4+0x0]
   29b0f:	xor    ecx,ecx
   29b11:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   29b20:	mov    edi,DWORD PTR [rbx+rcx*1]
   29b23:	cmp    rsi,rdi
   29b26:	jbe    2b121 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1731>
   29b2c:	or     BYTE PTR [r15+rdi*4+0x3],0x80
   29b32:	add    rcx,0x4
   29b36:	cmp    rax,rcx
   29b39:	jne    29b20 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x130>
   29b3b:	lea    rax,[rsi+rsi*1]
   29b3f:	mov    rcx,rax
   29b42:	or     rcx,r13
   29b45:	shr    rcx,0x20
   29b49:	je     29b52 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x162>
   29b4b:	xor    edx,edx
   29b4d:	div    r13
   29b50:	jmp    29b57 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x167>
   29b52:	xor    edx,edx
   29b54:	div    r13d
   29b57:	mov    QWORD PTR [rsp+0x58],rax
   29b5c:	mov    r14,QWORD PTR [rbp+0x40]
   29b60:	cmp    r14,r13
   29b63:	mov    rbx,r13
   29b66:	cmova  rbx,r14
   29b6a:	jae    2a97f <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf8f>
   29b70:	sub    rbx,r14
   29b73:	mov    rax,QWORD PTR [rbp+0x30]
   29b77:	sub    rax,r14
   29b7a:	mov    rdx,r14
   29b7d:	cmp    rbx,rax
   29b80:	ja     2b05b <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x166b>
   29b86:	mov    rax,QWORD PTR [rbp+0x38]
   29b8a:	inc    r14
   29b8d:	test   bl,0x1
   29b90:	jne    2b089 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1699>
   29b96:	cmp    r13,r14
   29b99:	jne    2b0b6 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16c6>
   29b9f:	mov    QWORD PTR [rbp+0x40],rdx
   29ba3:	cmp    r13,rdx
   29ba6:	ja     2a98f <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf9f>
   29bac:	mov    rax,QWORD PTR [rsp+0x58]
   29bb1:	lea    rbx,[rax+0x40]
   29bb5:	mov    r14,QWORD PTR [rbp+0x38]
   29bb9:	lea    rax,[r13*8+0x0]
   29bc1:	lea    r12,[rax+rax*2]
   29bc5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   29bd0:	mov    QWORD PTR [r14+0x10],0x0
   29bd8:	cmp    rbx,QWORD PTR [r14]
   29bdb:	ja     29be9 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f9>
   29bdd:	add    r14,0x18
   29be1:	add    r12,0xffffffffffffffe8
   29be5:	jne    29bd0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e0>
   29be7:	jmp    29c0b <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21b>
   29be9:	mov    ecx,0x1
   29bee:	mov    r8d,0x1
   29bf4:	mov    rdi,r14
   29bf7:	xor    esi,esi
   29bf9:	mov    rdx,rbx
   29bfc:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   29c01:	add    r14,0x18
   29c05:	add    r12,0xffffffffffffffe8
   29c09:	jne    29bd0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e0>
   29c0b:	mov    QWORD PTR [rbp+0x58],0x0
   29c13:	cmp    r13,QWORD PTR [rbp+0x48]
   29c17:	ja     2b027 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1637>
   29c1d:	xor    ebx,ebx
   29c1f:	mov    r14,QWORD PTR [rbp+0x50]
   29c23:	lea    rdi,[r14+rbx*8]
   29c27:	cmp    r13,0x1
   29c2b:	je     29c52 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x262>
   29c2d:	lea    rdx,[r13*8-0x8]
   29c35:	mov    esi,0xff
   29c3a:	call   QWORD PTR [rip+0x42258]        # 6be98 <memset@GLIBC_2.2.5>
   29c40:	lea    rax,[rbx+r13*1]
   29c44:	lea    rdi,[r14+rax*8]
   29c48:	add    rdi,0xfffffffffffffff8
   29c4c:	add    rbx,r13
   29c4f:	dec    rbx
   29c52:	mov    QWORD PTR [rdi],0xffffffffffffffff
   29c59:	inc    rbx
   29c5c:	mov    QWORD PTR [rbp+0x58],rbx
   29c60:	xorps  xmm0,xmm0
   29c63:	movups XMMWORD PTR [rsp+0x138],xmm0
   29c6b:	mov    DWORD PTR [rsp+0x148],0x0
   29c76:	movups XMMWORD PTR [rsp+0x158],xmm0
   29c7e:	mov    DWORD PTR [rsp+0x168],0x0
   29c89:	movups XMMWORD PTR [rsp+0x178],xmm0
   29c91:	mov    DWORD PTR [rsp+0x188],0x0
   29c9c:	movups XMMWORD PTR [rsp+0x198],xmm0
   29ca4:	mov    DWORD PTR [rsp+0x1a8],0x0
   29caf:	movups XMMWORD PTR [rsp+0x1b8],xmm0
   29cb7:	mov    DWORD PTR [rsp+0x1c8],0x0
   29cc2:	movups XMMWORD PTR [rsp+0x1d8],xmm0
   29cca:	mov    DWORD PTR [rsp+0x1e8],0x0
   29cd5:	movups XMMWORD PTR [rsp+0x1f8],xmm0
   29cdd:	mov    DWORD PTR [rsp+0x208],0x0
   29ce8:	movups XMMWORD PTR [rsp+0x218],xmm0
   29cf0:	mov    DWORD PTR [rsp+0x228],0x0
   29cfb:	movups XMMWORD PTR [rsp+0x238],xmm0
   29d03:	mov    DWORD PTR [rsp+0x248],0x0
   29d0e:	movups XMMWORD PTR [rsp+0x258],xmm0
   29d16:	mov    DWORD PTR [rsp+0x268],0x0
   29d21:	movups XMMWORD PTR [rsp+0x278],xmm0
   29d29:	mov    DWORD PTR [rsp+0x288],0x0
   29d34:	movups XMMWORD PTR [rsp+0x298],xmm0
   29d3c:	mov    DWORD PTR [rsp+0x2a8],0x0
   29d47:	movups XMMWORD PTR [rsp+0x2b8],xmm0
   29d4f:	mov    DWORD PTR [rsp+0x2c8],0x0
   29d5a:	movups XMMWORD PTR [rsp+0x2d8],xmm0
   29d62:	mov    DWORD PTR [rsp+0x2e8],0x0
   29d6d:	movups XMMWORD PTR [rsp+0x2f8],xmm0
   29d75:	mov    DWORD PTR [rsp+0x308],0x0
   29d80:	movups XMMWORD PTR [rsp+0x318],xmm0
   29d88:	mov    DWORD PTR [rsp+0x328],0x0
   29d93:	movups XMMWORD PTR [rsp+0x338],xmm0
   29d9b:	mov    DWORD PTR [rsp+0x348],0x0
   29da6:	movups XMMWORD PTR [rsp+0x358],xmm0
   29dae:	mov    DWORD PTR [rsp+0x368],0x0
   29db9:	movups XMMWORD PTR [rsp+0x378],xmm0
   29dc1:	mov    DWORD PTR [rsp+0x388],0x0
   29dcc:	movups XMMWORD PTR [rsp+0x398],xmm0
   29dd4:	mov    DWORD PTR [rsp+0x3a8],0x0
   29ddf:	movups XMMWORD PTR [rsp+0x3b8],xmm0
   29de7:	mov    DWORD PTR [rsp+0x3c8],0x0
   29df2:	movups XMMWORD PTR [rsp+0x3d8],xmm0
   29dfa:	mov    DWORD PTR [rsp+0x3e8],0x0
   29e05:	movups XMMWORD PTR [rsp+0x3f8],xmm0
   29e0d:	mov    DWORD PTR [rsp+0x408],0x0
   29e18:	movups XMMWORD PTR [rsp+0x418],xmm0
   29e20:	mov    DWORD PTR [rsp+0x428],0x0
   29e2b:	movups XMMWORD PTR [rsp+0x438],xmm0
   29e33:	mov    DWORD PTR [rsp+0x448],0x0
   29e3e:	movups XMMWORD PTR [rsp+0x458],xmm0
   29e46:	mov    DWORD PTR [rsp+0x468],0x0
   29e51:	movups XMMWORD PTR [rsp+0x478],xmm0
   29e59:	mov    DWORD PTR [rsp+0x488],0x0
   29e64:	movups XMMWORD PTR [rsp+0x498],xmm0
   29e6c:	mov    DWORD PTR [rsp+0x4a8],0x0
   29e77:	movups XMMWORD PTR [rsp+0x4b8],xmm0
   29e7f:	mov    DWORD PTR [rsp+0x4c8],0x0
   29e8a:	movups XMMWORD PTR [rsp+0x4d8],xmm0
   29e92:	mov    DWORD PTR [rsp+0x4e8],0x0
   29e9d:	movups XMMWORD PTR [rsp+0x4f8],xmm0
   29ea5:	mov    DWORD PTR [rsp+0x508],0x0
   29eb0:	movups XMMWORD PTR [rsp+0x518],xmm0
   29eb8:	mov    DWORD PTR [rsp+0x528],0x0
   29ec3:	mov    QWORD PTR [rsp+0x130],0xffffffffffffffff
   29ecf:	mov    QWORD PTR [rsp+0x150],0xffffffffffffffff
   29edb:	mov    QWORD PTR [rsp+0x170],0xffffffffffffffff
   29ee7:	mov    QWORD PTR [rsp+0x190],0xffffffffffffffff
   29ef3:	mov    QWORD PTR [rsp+0x1b0],0xffffffffffffffff
   29eff:	mov    QWORD PTR [rsp+0x1d0],0xffffffffffffffff
   29f0b:	mov    QWORD PTR [rsp+0x1f0],0xffffffffffffffff
   29f17:	mov    QWORD PTR [rsp+0x210],0xffffffffffffffff
   29f23:	mov    QWORD PTR [rsp+0x230],0xffffffffffffffff
   29f2f:	mov    QWORD PTR [rsp+0x250],0xffffffffffffffff
   29f3b:	mov    QWORD PTR [rsp+0x270],0xffffffffffffffff
   29f47:	mov    QWORD PTR [rsp+0x290],0xffffffffffffffff
   29f53:	mov    QWORD PTR [rsp+0x2b0],0xffffffffffffffff
   29f5f:	mov    QWORD PTR [rsp+0x2d0],0xffffffffffffffff
   29f6b:	mov    QWORD PTR [rsp+0x2f0],0xffffffffffffffff
   29f77:	mov    QWORD PTR [rsp+0x310],0xffffffffffffffff
   29f83:	mov    QWORD PTR [rsp+0x330],0xffffffffffffffff
   29f8f:	mov    QWORD PTR [rsp+0x350],0xffffffffffffffff
   29f9b:	mov    QWORD PTR [rsp+0x370],0xffffffffffffffff
   29fa7:	mov    QWORD PTR [rsp+0x390],0xffffffffffffffff
   29fb3:	mov    QWORD PTR [rsp+0x3b0],0xffffffffffffffff
   29fbf:	mov    QWORD PTR [rsp+0x3d0],0xffffffffffffffff
   29fcb:	mov    QWORD PTR [rsp+0x3f0],0xffffffffffffffff
   29fd7:	mov    QWORD PTR [rsp+0x410],0xffffffffffffffff
   29fe3:	mov    QWORD PTR [rsp+0x430],0xffffffffffffffff
   29fef:	mov    QWORD PTR [rsp+0x450],0xffffffffffffffff
   29ffb:	mov    QWORD PTR [rsp+0x470],0xffffffffffffffff
   2a007:	mov    QWORD PTR [rsp+0x490],0xffffffffffffffff
   2a013:	mov    QWORD PTR [rsp+0x4b0],0xffffffffffffffff
   2a01f:	mov    QWORD PTR [rsp+0x4d0],0xffffffffffffffff
   2a02b:	mov    QWORD PTR [rsp+0x4f0],0xffffffffffffffff
   2a037:	mov    QWORD PTR [rsp+0x510],0xffffffffffffffff
   2a043:	mov    rax,QWORD PTR [rbp+0x38]
   2a047:	mov    rdi,QWORD PTR [rbp+0x40]
   2a04b:	mov    r9,QWORD PTR [rbp+0x8]
   2a04f:	mov    r10,QWORD PTR [rbp+0x10]
   2a053:	add    rax,0x8
   2a057:	xor    r12d,r12d
   2a05a:	xor    ecx,ecx
   2a05c:	nop    DWORD PTR [rax+0x0]
   2a060:	cmp    rdi,rcx
   2a063:	je     2b12d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x173d>
   2a069:	cmp    r10,rcx
   2a06c:	je     2b13c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x174c>
   2a072:	mov    rdx,QWORD PTR [rax]
   2a075:	mov    esi,DWORD PTR [r9+rcx*4]
   2a079:	mov    esi,DWORD PTR [r15+rsi*4]
   2a07d:	mov    BYTE PTR [rdx],sil
   2a080:	shr    esi,0x8
   2a083:	and    esi,0x7fffff
   2a089:	lea    r8,[rdx+0x1]
   2a08d:	add    rdx,QWORD PTR [rax-0x8]
   2a091:	mov    QWORD PTR [rsp+r12*1+0x130],rcx
   2a099:	mov    QWORD PTR [rsp+r12*1+0x138],r8
   2a0a1:	mov    QWORD PTR [rsp+r12*1+0x140],rdx
   2a0a9:	mov    DWORD PTR [rsp+r12*1+0x148],esi
   2a0b1:	prefetcht0 BYTE PTR [r15+rsi*4]
   2a0b6:	lea    rdx,[rcx+0x1]
   2a0ba:	add    r12,0x20
   2a0be:	cmp    rcx,0x1e
   2a0c2:	ja     2a0d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6e0>
   2a0c4:	add    rax,0x18
   2a0c8:	mov    rcx,rdx
   2a0cb:	cmp    rdx,r13
   2a0ce:	jb     2a060 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x670>
   2a0d0:	cmp    rdx,0x20
   2a0d4:	jne    2a9a0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfb0>
   2a0da:	mov    QWORD PTR [rsp+0xf8],rdx
   2a0e2:	mov    QWORD PTR [rsp+0x128],r13
   2a0ea:	mov    eax,DWORD PTR [rsp+0x148]
   2a0f1:	mov    ecx,DWORD PTR [rsp+0x168]
   2a0f8:	mov    DWORD PTR [rsp+0x68],eax
   2a0fc:	mov    DWORD PTR [rsp+0x6c],ecx
   2a100:	mov    eax,DWORD PTR [rsp+0x188]
   2a107:	mov    DWORD PTR [rsp+0x70],eax
   2a10b:	mov    eax,DWORD PTR [rsp+0x1a8]
   2a112:	mov    DWORD PTR [rsp+0x74],eax
   2a116:	mov    eax,DWORD PTR [rsp+0x1c8]
   2a11d:	mov    DWORD PTR [rsp+0x78],eax
   2a121:	mov    eax,DWORD PTR [rsp+0x1e8]
   2a128:	mov    DWORD PTR [rsp+0x7c],eax
   2a12c:	mov    eax,DWORD PTR [rsp+0x208]
   2a133:	mov    DWORD PTR [rsp+0x80],eax
   2a13a:	mov    eax,DWORD PTR [rsp+0x228]
   2a141:	mov    DWORD PTR [rsp+0x84],eax
   2a148:	mov    eax,DWORD PTR [rsp+0x248]
   2a14f:	mov    DWORD PTR [rsp+0x88],eax
   2a156:	mov    eax,DWORD PTR [rsp+0x268]
   2a15d:	mov    ecx,DWORD PTR [rsp+0x288]
   2a164:	mov    DWORD PTR [rsp+0x8c],eax
   2a16b:	mov    DWORD PTR [rsp+0x90],ecx
   2a172:	mov    eax,DWORD PTR [rsp+0x2a8]
   2a179:	mov    DWORD PTR [rsp+0x94],eax
   2a180:	mov    eax,DWORD PTR [rsp+0x2c8]
   2a187:	mov    DWORD PTR [rsp+0x98],eax
   2a18e:	mov    eax,DWORD PTR [rsp+0x2e8]
   2a195:	mov    DWORD PTR [rsp+0x9c],eax
   2a19c:	mov    eax,DWORD PTR [rsp+0x308]
   2a1a3:	mov    DWORD PTR [rsp+0xa0],eax
   2a1aa:	mov    eax,DWORD PTR [rsp+0x328]
   2a1b1:	mov    DWORD PTR [rsp+0xa4],eax
   2a1b8:	mov    eax,DWORD PTR [rsp+0x348]
   2a1bf:	mov    DWORD PTR [rsp+0xa8],eax
   2a1c6:	mov    eax,DWORD PTR [rsp+0x368]
   2a1cd:	mov    DWORD PTR [rsp+0xac],eax
   2a1d4:	mov    eax,DWORD PTR [rsp+0x388]
   2a1db:	mov    DWORD PTR [rsp+0xb0],eax
   2a1e2:	mov    eax,DWORD PTR [rsp+0x3a8]
   2a1e9:	mov    DWORD PTR [rsp+0xb4],eax
   2a1f0:	mov    eax,DWORD PTR [rsp+0x3c8]
   2a1f7:	mov    DWORD PTR [rsp+0xb8],eax
   2a1fe:	mov    eax,DWORD PTR [rsp+0x3e8]
   2a205:	mov    DWORD PTR [rsp+0xbc],eax
   2a20c:	mov    eax,DWORD PTR [rsp+0x408]
   2a213:	mov    DWORD PTR [rsp+0xc0],eax
   2a21a:	mov    eax,DWORD PTR [rsp+0x428]
   2a221:	mov    DWORD PTR [rsp+0xc4],eax
   2a228:	mov    eax,DWORD PTR [rsp+0x448]
   2a22f:	mov    DWORD PTR [rsp+0xc8],eax
   2a236:	mov    eax,DWORD PTR [rsp+0x468]
   2a23d:	mov    DWORD PTR [rsp+0xcc],eax
   2a244:	mov    eax,DWORD PTR [rsp+0x488]
   2a24b:	mov    DWORD PTR [rsp+0xd0],eax
   2a252:	mov    eax,DWORD PTR [rsp+0x4a8]
   2a259:	mov    DWORD PTR [rsp+0xd4],eax
   2a260:	mov    eax,DWORD PTR [rsp+0x4c8]
   2a267:	mov    DWORD PTR [rsp+0xd8],eax
   2a26e:	mov    eax,DWORD PTR [rsp+0x4e8]
   2a275:	mov    DWORD PTR [rsp+0xdc],eax
   2a27c:	mov    eax,DWORD PTR [rsp+0x508]
   2a283:	mov    DWORD PTR [rsp+0xe0],eax
   2a28a:	mov    eax,DWORD PTR [rsp+0x528]
   2a291:	mov    DWORD PTR [rsp+0xe4],eax
   2a298:	lea    rdi,[rsp+0x630]
   2a2a0:	mov    edx,0x2000
   2a2a5:	xor    esi,esi
   2a2a7:	call   QWORD PTR [rip+0x41beb]        # 6be98 <memset@GLIBC_2.2.5>
   2a2ad:	add    QWORD PTR [rsp+0x58],0x3f
   2a2b3:	mov    rbx,QWORD PTR [rbp+0x38]
   2a2b7:	mov    rsi,QWORD PTR [rbp+0x40]
   2a2bb:	mov    rax,QWORD PTR [rbp+0x50]
   2a2bf:	mov    QWORD PTR [rsp+0x60],rax
   2a2c4:	mov    rax,QWORD PTR [rbp+0x58]
   2a2c8:	mov    QWORD PTR [rsp+0x28],rax
   2a2cd:	mov    rax,QWORD PTR [rbp+0x8]
   2a2d1:	mov    QWORD PTR [rsp+0x38],rax
   2a2d6:	mov    rax,QWORD PTR [rbp+0x10]
   2a2da:	mov    QWORD PTR [rsp+0x40],rax
   2a2df:	mov    r9d,0x20
   2a2e5:	mov    QWORD PTR [rsp+0x100],r12
   2a2ed:	mov    QWORD PTR [rsp+0x18],rsi
   2a2f2:	mov    QWORD PTR [rsp+0xe8],rbx
   2a2fa:	mov    QWORD PTR [rsp+0x108],r15
   2a302:	jmp    2a319 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x929>
   2a304:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a310:	test   r13,r13
   2a313:	jne    2a3da <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x9ea>
   2a319:	xor    ebp,ebp
   2a31b:	mov    r13,QWORD PTR [rsp+0x58]
   2a320:	jmp    2a35d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x96d>
   2a322:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a330:	add    r14,rax
   2a333:	mov    QWORD PTR [rsp+rbp*1+0x138],r14
   2a33b:	add    rdx,rax
   2a33e:	mov    QWORD PTR [rsp+rbp*1+0x140],rdx
   2a346:	sub    rdx,r14
   2a349:	add    rbp,0x20
   2a34d:	cmp    rdx,r13
   2a350:	cmovb  r13,rdx
   2a354:	cmp    rbp,0x400
   2a35b:	je     2a310 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x920>
   2a35d:	mov    r14,QWORD PTR [rsp+rbp*1+0x138]
   2a365:	mov    rdx,QWORD PTR [rsp+rbp*1+0x140]
   2a36d:	cmp    r14,rdx
   2a370:	jne    2a346 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x956>
   2a372:	mov    rdi,QWORD PTR [rsp+rbp*1+0x130]
   2a37a:	cmp    rdi,rsi
   2a37d:	jae    2b115 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1725>
   2a383:	lea    rcx,[rdi+rdi*2]
   2a387:	mov    rdx,QWORD PTR [rbx+rcx*8]
   2a38b:	mov    rax,QWORD PTR [rbx+rcx*8+0x8]
   2a390:	sub    r14,rax
   2a393:	mov    QWORD PTR [rbx+rcx*8+0x10],r14
   2a398:	mov    rdi,rdx
   2a39b:	sub    rdi,r14
   2a39e:	cmp    rdx,rdi
   2a3a1:	jbe    2a330 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   2a3a3:	lea    r12,[rbx+rcx*8]
   2a3a7:	mov    ecx,0x1
   2a3ac:	mov    r8d,0x1
   2a3b2:	mov    rdi,r12
   2a3b5:	mov    rsi,r14
   2a3b8:	mov    QWORD PTR [rsp+0x8],r9
   2a3bd:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2a3c2:	mov    r9,QWORD PTR [rsp+0x8]
   2a3c7:	mov    rsi,QWORD PTR [rsp+0x18]
   2a3cc:	mov    rdx,QWORD PTR [r12]
   2a3d0:	mov    rax,QWORD PTR [r12+0x8]
   2a3d5:	jmp    2a330 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   2a3da:	xor    ecx,ecx
   2a3dc:	mov    rsi,r13
   2a3df:	sub    rsi,rcx
   2a3e2:	cmp    rsi,0x100
   2a3e9:	mov    eax,0x100
   2a3ee:	cmovae rsi,rax
   2a3f2:	xorps  xmm0,xmm0
   2a3f5:	movaps XMMWORD PTR [rsp+0x620],xmm0
   2a3fd:	movaps XMMWORD PTR [rsp+0x610],xmm0
   2a405:	movaps XMMWORD PTR [rsp+0x600],xmm0
   2a40d:	movaps XMMWORD PTR [rsp+0x5f0],xmm0
   2a415:	movaps XMMWORD PTR [rsp+0x5e0],xmm0
   2a41d:	movaps XMMWORD PTR [rsp+0x5d0],xmm0
   2a425:	movaps XMMWORD PTR [rsp+0x5c0],xmm0
   2a42d:	movaps XMMWORD PTR [rsp+0x5b0],xmm0
   2a435:	movaps XMMWORD PTR [rsp+0x5a0],xmm0
   2a43d:	movaps XMMWORD PTR [rsp+0x590],xmm0
   2a445:	movaps XMMWORD PTR [rsp+0x580],xmm0
   2a44d:	movaps XMMWORD PTR [rsp+0x570],xmm0
   2a455:	movaps XMMWORD PTR [rsp+0x560],xmm0
   2a45d:	movaps XMMWORD PTR [rsp+0x550],xmm0
   2a465:	movaps XMMWORD PTR [rsp+0x540],xmm0
   2a46d:	movaps XMMWORD PTR [rsp+0x530],xmm0
   2a475:	cmp    r13,rcx
   2a478:	mov    QWORD PTR [rsp+0x110],rcx
   2a480:	mov    QWORD PTR [rsp+0x30],rsi
   2a485:	jne    2a560 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb70>
   2a48b:	mov    QWORD PTR [rsp+0x8],r9
   2a490:	lea    r12,[rsp+0x538]
   2a498:	lea    rbp,[rsp+0x630]
   2a4a0:	xor    ebx,ebx
   2a4a2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a4b0:	mov    rsi,QWORD PTR [r12-0x8]
   2a4b5:	mov    r15,QWORD PTR [rsp+0x30]
   2a4ba:	mov    r14,r15
   2a4bd:	sub    r14,rsi
   2a4c0:	add    rsi,rbp
   2a4c3:	mov    rdi,QWORD PTR [rsp+rbx*1+0x138]
   2a4cb:	mov    rdx,r14
   2a4ce:	call   QWORD PTR [rip+0x4199c]        # 6be70 <memcpy@GLIBC_2.14>
   2a4d4:	add    QWORD PTR [rsp+rbx*1+0x138],r14
   2a4dc:	lea    rax,[rbx+0x40]
   2a4e0:	mov    QWORD PTR [rsp+0x10],rax
   2a4e5:	mov    rax,QWORD PTR [r12]
   2a4e9:	sub    r15,rax
   2a4ec:	lea    rsi,[rax+rbp*1]
   2a4f0:	add    rsi,0x100
   2a4f7:	mov    rdi,QWORD PTR [rsp+rbx*1+0x158]
   2a4ff:	mov    rdx,r15
   2a502:	call   QWORD PTR [rip+0x41968]        # 6be70 <memcpy@GLIBC_2.14>
   2a508:	add    QWORD PTR [rsp+rbx*1+0x158],r15
   2a510:	add    rbp,0x200
   2a517:	add    r12,0x10
   2a51b:	mov    rax,QWORD PTR [rsp+0x10]
   2a520:	mov    rbx,rax
   2a523:	cmp    rax,0x400
   2a529:	jne    2a4b0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   2a52b:	mov    rcx,QWORD PTR [rsp+0x110]
   2a533:	add    rcx,QWORD PTR [rsp+0x30]
   2a538:	cmp    rcx,r13
   2a53b:	mov    rsi,QWORD PTR [rsp+0x18]
   2a540:	mov    rbx,QWORD PTR [rsp+0xe8]
   2a548:	mov    r9,QWORD PTR [rsp+0x8]
   2a54d:	mov    r15,QWORD PTR [rsp+0x108]
   2a555:	jb     2a3dc <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x9ec>
   2a55b:	jmp    2a319 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x929>
   2a560:	lea    rcx,[rsp+0xd30]
   2a568:	xor    eax,eax
   2a56a:	jmp    2a589 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb99>
   2a56c:	nop    DWORD PTR [rax+0x0]
   2a570:	mov    rcx,QWORD PTR [rsp+0x118]
   2a578:	inc    rcx
   2a57b:	mov    rsi,QWORD PTR [rsp+0x30]
   2a580:	cmp    rax,rsi
   2a583:	je     2a48b <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xa9b>
   2a589:	mov    QWORD PTR [rsp+0x10],rax
   2a58e:	mov    QWORD PTR [rsp+0x118],rcx
   2a596:	mov    rax,rcx
   2a599:	xor    ecx,ecx
   2a59b:	xor    ebp,ebp
   2a59d:	jmp    2a5bf <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbcf>
   2a59f:	nop
   2a5a0:	mov    edx,0x80
   2a5a5:	shl    rdx,cl
   2a5a8:	or     rbp,rdx
   2a5ab:	add    rcx,0x8
   2a5af:	add    rax,0x800
   2a5b5:	cmp    rcx,0x19
   2a5b9:	jae    2a7b0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdc0>
   2a5bf:	mov    edx,DWORD PTR [rsp+rcx*4+0x68]
   2a5c3:	mov    edx,DWORD PTR [r15+rdx*4]
   2a5c7:	test   edx,edx
   2a5c9:	js     2a6d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xce0>
   2a5cf:	mov    BYTE PTR [rax-0x700],dl
   2a5d5:	shr    edx,0x8
   2a5d8:	mov    DWORD PTR [rsp+rcx*4+0x68],edx
   2a5dc:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a5e1:	mov    edx,DWORD PTR [rsp+rcx*4+0x6c]
   2a5e5:	mov    edx,DWORD PTR [r15+rdx*4]
   2a5e9:	test   edx,edx
   2a5eb:	js     2a6e4 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcf4>
   2a5f1:	mov    BYTE PTR [rax-0x600],dl
   2a5f7:	shr    edx,0x8
   2a5fa:	mov    DWORD PTR [rsp+rcx*4+0x6c],edx
   2a5fe:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a603:	mov    edx,DWORD PTR [rsp+rcx*4+0x70]
   2a607:	mov    edx,DWORD PTR [r15+rdx*4]
   2a60b:	test   edx,edx
   2a60d:	js     2a6ff <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd0f>
   2a613:	mov    BYTE PTR [rax-0x500],dl
   2a619:	shr    edx,0x8
   2a61c:	mov    DWORD PTR [rsp+rcx*4+0x70],edx
   2a620:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a625:	mov    edx,DWORD PTR [rsp+rcx*4+0x74]
   2a629:	mov    edx,DWORD PTR [r15+rdx*4]
   2a62d:	test   edx,edx
   2a62f:	js     2a71a <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd2a>
   2a635:	mov    BYTE PTR [rax-0x400],dl
   2a63b:	shr    edx,0x8
   2a63e:	mov    DWORD PTR [rsp+rcx*4+0x74],edx
   2a642:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a647:	mov    edx,DWORD PTR [rsp+rcx*4+0x78]
   2a64b:	mov    edx,DWORD PTR [r15+rdx*4]
   2a64f:	test   edx,edx
   2a651:	js     2a735 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd45>
   2a657:	mov    BYTE PTR [rax-0x300],dl
   2a65d:	shr    edx,0x8
   2a660:	mov    DWORD PTR [rsp+rcx*4+0x78],edx
   2a664:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a669:	mov    edx,DWORD PTR [rsp+rcx*4+0x7c]
   2a66d:	mov    edx,DWORD PTR [r15+rdx*4]
   2a671:	test   edx,edx
   2a673:	js     2a750 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd60>
   2a679:	mov    BYTE PTR [rax-0x200],dl
   2a67f:	shr    edx,0x8
   2a682:	mov    DWORD PTR [rsp+rcx*4+0x7c],edx
   2a686:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a68b:	mov    edx,DWORD PTR [rsp+rcx*4+0x80]
   2a692:	mov    edx,DWORD PTR [r15+rdx*4]
   2a696:	test   edx,edx
   2a698:	js     2a76e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7e>
   2a69e:	mov    BYTE PTR [rax-0x100],dl
   2a6a4:	shr    edx,0x8
   2a6a7:	mov    DWORD PTR [rsp+rcx*4+0x80],edx
   2a6ae:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a6b3:	mov    edx,DWORD PTR [rsp+rcx*4+0x84]
   2a6ba:	mov    edx,DWORD PTR [r15+rdx*4]
   2a6be:	test   edx,edx
   2a6c0:	jns    2a78c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd9c>
   2a6c6:	jmp    2a5a0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbb0>
   2a6cb:	nop    DWORD PTR [rax+rax*1+0x0]
   2a6d0:	bts    rbp,rcx
   2a6d4:	mov    edx,DWORD PTR [rsp+rcx*4+0x6c]
   2a6d8:	mov    edx,DWORD PTR [r15+rdx*4]
   2a6dc:	test   edx,edx
   2a6de:	jns    2a5f1 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc01>
   2a6e4:	mov    edx,0x2
   2a6e9:	shl    rdx,cl
   2a6ec:	or     rbp,rdx
   2a6ef:	mov    edx,DWORD PTR [rsp+rcx*4+0x70]
   2a6f3:	mov    edx,DWORD PTR [r15+rdx*4]
   2a6f7:	test   edx,edx
   2a6f9:	jns    2a613 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc23>
   2a6ff:	mov    edx,0x4
   2a704:	shl    rdx,cl
   2a707:	or     rbp,rdx
   2a70a:	mov    edx,DWORD PTR [rsp+rcx*4+0x74]
   2a70e:	mov    edx,DWORD PTR [r15+rdx*4]
   2a712:	test   edx,edx
   2a714:	jns    2a635 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc45>
   2a71a:	mov    edx,0x8
   2a71f:	shl    rdx,cl
   2a722:	or     rbp,rdx
   2a725:	mov    edx,DWORD PTR [rsp+rcx*4+0x78]
   2a729:	mov    edx,DWORD PTR [r15+rdx*4]
   2a72d:	test   edx,edx
   2a72f:	jns    2a657 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc67>
   2a735:	mov    edx,0x10
   2a73a:	shl    rdx,cl
   2a73d:	or     rbp,rdx
   2a740:	mov    edx,DWORD PTR [rsp+rcx*4+0x7c]
   2a744:	mov    edx,DWORD PTR [r15+rdx*4]
   2a748:	test   edx,edx
   2a74a:	jns    2a679 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc89>
   2a750:	mov    edx,0x20
   2a755:	shl    rdx,cl
   2a758:	or     rbp,rdx
   2a75b:	mov    edx,DWORD PTR [rsp+rcx*4+0x80]
   2a762:	mov    edx,DWORD PTR [r15+rdx*4]
   2a766:	test   edx,edx
   2a768:	jns    2a69e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcae>
   2a76e:	mov    edx,0x40
   2a773:	shl    rdx,cl
   2a776:	or     rbp,rdx
   2a779:	mov    edx,DWORD PTR [rsp+rcx*4+0x84]
   2a780:	mov    edx,DWORD PTR [r15+rdx*4]
   2a784:	test   edx,edx
   2a786:	js     2a5a0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbb0>
   2a78c:	mov    BYTE PTR [rax],dl
   2a78e:	shr    edx,0x8
   2a791:	mov    DWORD PTR [rsp+rcx*4+0x84],edx
   2a798:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a79d:	jmp    2a5ab <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbbb>
   2a7a2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a7b0:	mov    rax,QWORD PTR [rsp+0x10]
   2a7b5:	inc    rax
   2a7b8:	test   rbp,rbp
   2a7bb:	je     2a570 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb80>
   2a7c1:	mov    QWORD PTR [rsp+0x120],rax
   2a7c9:	nop    DWORD PTR [rax+0x0]
   2a7d0:	mov    QWORD PTR [rsp+0x8],r9
   2a7d5:	tzcnt  r14,rbp
   2a7da:	cmp    r14,0x1f
   2a7de:	ja     2b1b4 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17c4>
   2a7e4:	mov    ebx,r14d
   2a7e7:	shl    ebx,0x5
   2a7ea:	mov    eax,DWORD PTR [rsp+r14*4+0x68]
   2a7ef:	mov    DWORD PTR [rsp+rbx*1+0x148],eax
   2a7f6:	mov    eax,r14d
   2a7f9:	shl    eax,0x8
   2a7fc:	lea    rsi,[rsp+rax*1]
   2a800:	add    rsi,0x630
   2a807:	mov    rax,QWORD PTR [rsp+r14*8+0x530]
   2a80f:	mov    r12,QWORD PTR [rsp+0x10]
   2a814:	sub    r12,rax
   2a817:	add    rsi,rax
   2a81a:	mov    rdi,QWORD PTR [rsp+rbx*1+0x138]
   2a822:	mov    rdx,r12
   2a825:	call   QWORD PTR [rip+0x41645]        # 6be70 <memcpy@GLIBC_2.14>
   2a82b:	add    r12,QWORD PTR [rsp+rbx*1+0x138]
   2a833:	mov    QWORD PTR [rsp+rbx*1+0x138],r12
   2a83b:	mov    rdi,QWORD PTR [rsp+rbx*1+0x130]
   2a843:	mov    r8,QWORD PTR [rsp+0x18]
   2a848:	cmp    rdi,r8
   2a84b:	jae    2b1a5 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17b5>
   2a851:	lea    rax,[rsp+rbx*1]
   2a855:	add    rax,0x130
   2a85b:	lea    rcx,[rdi+rdi*2]
   2a85f:	mov    rbx,QWORD PTR [rsp+0xe8]
   2a867:	sub    r12,QWORD PTR [rbx+rcx*8+0x8]
   2a86c:	mov    QWORD PTR [rbx+rcx*8+0x10],r12
   2a871:	mov    rsi,QWORD PTR [rsp+0x40]
   2a876:	test   rsi,rsi
   2a879:	je     2a890 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xea0>
   2a87b:	mov    edx,DWORD PTR [rax+0x18]
   2a87e:	cmp    rsi,0x1
   2a882:	mov    r10,QWORD PTR [rsp+0x38]
   2a887:	jne    2a8a0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xeb0>
   2a889:	xor    ecx,ecx
   2a88b:	jmp    2a8cd <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xedd>
   2a88d:	nop    DWORD PTR [rax]
   2a890:	mov    rcx,0xffffffffffffffff
   2a897:	mov    r10,QWORD PTR [rsp+0x38]
   2a89c:	jmp    2a8e1 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xef1>
   2a89e:	xchg   ax,ax
   2a8a0:	mov    r11,rsi
   2a8a3:	xor    ecx,ecx
   2a8a5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a8b0:	mov    r8,rcx
   2a8b3:	mov    r9,r11
   2a8b6:	shr    r9,1
   2a8b9:	add    rcx,r9
   2a8bc:	cmp    DWORD PTR [r10+rcx*4],edx
   2a8c0:	cmova  rcx,r8
   2a8c4:	sub    r11,r9
   2a8c7:	cmp    r11,0x1
   2a8cb:	ja     2a8b0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xec0>
   2a8cd:	cmp    DWORD PTR [r10+rcx*4],edx
   2a8d1:	mov    rdx,0xffffffffffffffff
   2a8d8:	cmovne rcx,rdx
   2a8dc:	mov    r8,QWORD PTR [rsp+0x18]
   2a8e1:	cmp    rdi,QWORD PTR [rsp+0x28]
   2a8e6:	mov    r9,QWORD PTR [rsp+0x8]
   2a8eb:	jae    2b194 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17a4>
   2a8f1:	lea    rdx,[rbp-0x1]
   2a8f5:	and    rbp,rdx
   2a8f8:	mov    rdx,QWORD PTR [rsp+0x60]
   2a8fd:	mov    QWORD PTR [rdx+rdi*8],rcx
   2a901:	cmp    r9,QWORD PTR [rsp+0x128]
   2a909:	je     2a9b7 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfc7>
   2a90f:	cmp    r9,r8
   2a912:	jae    2b1d7 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e7>
   2a918:	cmp    r9,rsi
   2a91b:	jae    2b1c8 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17d8>
   2a921:	lea    rcx,[r9+r9*2]
   2a925:	mov    rdx,QWORD PTR [rbx+rcx*8+0x8]
   2a92a:	mov    esi,DWORD PTR [r10+r9*4]
   2a92e:	mov    esi,DWORD PTR [r15+rsi*4]
   2a932:	mov    BYTE PTR [rdx],sil
   2a935:	shr    esi,0x8
   2a938:	and    esi,0x7fffff
   2a93e:	lea    rdi,[rdx+0x1]
   2a942:	add    rdx,QWORD PTR [rbx+rcx*8]
   2a946:	mov    QWORD PTR [rax],r9
   2a949:	mov    QWORD PTR [rax+0x8],rdi
   2a94d:	mov    QWORD PTR [rax+0x10],rdx
   2a951:	mov    DWORD PTR [rax+0x18],esi
   2a954:	mov    DWORD PTR [rsp+r14*4+0x68],esi
   2a959:	prefetcht0 BYTE PTR [r15+rsi*4]
   2a95e:	inc    r9
   2a961:	mov    rax,QWORD PTR [rsp+0x120]
   2a969:	mov    QWORD PTR [rsp+r14*8+0x530],rax
   2a971:	test   rbp,rbp
   2a974:	jne    2a7d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xde0>
   2a97a:	jmp    2a570 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb80>
   2a97f:	mov    rdx,rbx
   2a982:	mov    QWORD PTR [rbp+0x40],rdx
   2a986:	cmp    r13,rdx
   2a989:	jbe    29bac <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc>
   2a98f:	lea    rcx,[rip+0x3f7d2]        # 6a168 <__frame_dummy_init_array_entry+0x7e8>
   2a996:	xor    edi,edi
   2a998:	mov    rsi,r13
   2a99b:	call   21140 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   2a9a0:	mov    rax,QWORD PTR [rbp+0x50]
   2a9a4:	mov    QWORD PTR [rsp+0x60],rax
   2a9a9:	mov    rax,QWORD PTR [rbp+0x58]
   2a9ad:	mov    QWORD PTR [rsp+0x28],rax
   2a9b2:	jmp    2aa4c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x105c>
   2a9b7:	not    rbp
   2a9ba:	xor    r12d,r12d
   2a9bd:	xor    r13d,r13d
   2a9c0:	jmp    2a9e0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xff0>
   2a9c2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a9d0:	add    r13,0x20
   2a9d4:	inc    r12
   2a9d7:	cmp    r13,0x400
   2a9de:	je     2aa32 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1042>
   2a9e0:	mov    eax,DWORD PTR [rsp+r12*4+0x68]
   2a9e5:	mov    DWORD PTR [rsp+r13*1+0x148],eax
   2a9ed:	cmp    r14,r12
   2a9f0:	je     2a9d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfe0>
   2a9f2:	mov    rax,QWORD PTR [rsp+r12*8+0x530]
   2a9fa:	bt     rbp,r12
   2a9fe:	mov    rbx,QWORD PTR [rsp+0x10]
   2aa03:	adc    rbx,0x0
   2aa07:	sub    rbx,rax
   2aa0a:	add    rax,rsp
   2aa0d:	add    rax,0x630
   2aa13:	lea    rsi,[rax+r13*8]
   2aa17:	mov    rdi,QWORD PTR [rsp+r13*1+0x138]
   2aa1f:	mov    rdx,rbx
   2aa22:	call   QWORD PTR [rip+0x41448]        # 6be70 <memcpy@GLIBC_2.14>
   2aa28:	add    QWORD PTR [rsp+r13*1+0x138],rbx
   2aa30:	jmp    2a9d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfe0>
   2aa32:	mov    r9,QWORD PTR [rsp+0x38]
   2aa37:	mov    r10,QWORD PTR [rsp+0x40]
   2aa3c:	mov    r12,QWORD PTR [rsp+0x100]
   2aa44:	mov    rdx,QWORD PTR [rsp+0xf8]
   2aa4c:	shl    rdx,0x5
   2aa50:	lea    r13,[rsp+rdx*1]
   2aa54:	add    r13,0x130
   2aa5b:	lea    r14,[rsp+0x130]
   2aa63:	mov    rax,QWORD PTR [rsp+0x60]
   2aa68:	jmp    2aa7d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x108d>
   2aa6a:	nop    WORD PTR [rax+rax*1+0x0]
   2aa70:	mov    r14,rbp
   2aa73:	add    r12,0xffffffffffffffe0
   2aa77:	je     2abfb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x120b>
   2aa7d:	lea    rbp,[r14+0x20]
   2aa81:	mov    rbx,QWORD PTR [r14]
   2aa84:	cmp    rbx,0xffffffffffffffff
   2aa88:	je     2aa70 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1080>
   2aa8a:	cmp    rbx,QWORD PTR [rsp+0x28]
   2aa8f:	jae    2b1e9 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17f9>
   2aa95:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   2aa9a:	jne    2aa70 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1080>
   2aa9c:	mov    edi,0x80
   2aaa1:	call   QWORD PTR [rip+0x413b9]        # 6be60 <malloc@GLIBC_2.2.5>
   2aaa7:	test   rax,rax
   2aaaa:	je     2b21a <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x182a>
   2aab0:	mov    rcx,QWORD PTR [r14+0x18]
   2aab4:	mov    QWORD PTR [rax+0x18],rcx
   2aab8:	movups xmm0,XMMWORD PTR [r14+0x8]
   2aabd:	movups XMMWORD PTR [rax+0x8],xmm0
   2aac1:	mov    QWORD PTR [rax],rbx
   2aac4:	mov    QWORD PTR [rsp+0x530],0x4
   2aad0:	mov    QWORD PTR [rsp+0x538],rax
   2aad8:	mov    QWORD PTR [rsp+0x540],0x1
   2aae4:	mov    r12d,0x1
   2aaea:	cmp    rbp,r13
   2aaed:	je     2abb7 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11c7>
   2aaf3:	lea    rbx,[rsp+0x530]
   2aafb:	mov    rdx,QWORD PTR [rsp+0x20]
   2ab00:	mov    rcx,QWORD PTR [rdx+0x50]
   2ab04:	mov    rsi,QWORD PTR [rdx+0x58]
   2ab08:	mov    rdx,rbp
   2ab0b:	jmp    2ab1c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x112c>
   2ab0d:	nop    DWORD PTR [rax]
   2ab10:	mov    rdx,rbp
   2ab13:	cmp    rbp,r13
   2ab16:	je     2abb7 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11c7>
   2ab1c:	lea    rbp,[rdx+0x20]
   2ab20:	mov    r14,QWORD PTR [rdx]
   2ab23:	cmp    r14,0xffffffffffffffff
   2ab27:	je     2ab10 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1120>
   2ab29:	cmp    r14,rsi
   2ab2c:	jae    2b1fd <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x180d>
   2ab32:	cmp    QWORD PTR [rcx+r14*8],0xffffffffffffffff
   2ab37:	jne    2ab10 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1120>
   2ab39:	mov    rcx,QWORD PTR [rdx+0x18]
   2ab3d:	mov    QWORD PTR [rsp+0x640],rcx
   2ab45:	movups xmm0,XMMWORD PTR [rdx+0x8]
   2ab49:	movaps XMMWORD PTR [rsp+0x630],xmm0
   2ab51:	cmp    r12,QWORD PTR [rsp+0x530]
   2ab59:	jne    2ab7e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x118e>
   2ab5b:	mov    edx,0x1
   2ab60:	mov    ecx,0x8
   2ab65:	mov    r8d,0x20
   2ab6b:	mov    rdi,rbx
   2ab6e:	mov    rsi,r12
   2ab71:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2ab76:	mov    rax,QWORD PTR [rsp+0x538]
   2ab7e:	mov    rcx,r12
   2ab81:	shl    rcx,0x5
   2ab85:	mov    QWORD PTR [rax+rcx*1],r14
   2ab89:	movaps xmm0,XMMWORD PTR [rsp+0x630]
   2ab91:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   2ab96:	mov    rdx,QWORD PTR [rsp+0x640]
   2ab9e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   2aba3:	inc    r12
   2aba6:	mov    QWORD PTR [rsp+0x540],r12
   2abae:	cmp    rbp,r13
   2abb1:	jne    2aafb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x110b>
   2abb7:	mov    rax,QWORD PTR [rsp+0x530]
   2abbf:	mov    QWORD PTR [rsp+0x18],rax
   2abc4:	mov    rax,QWORD PTR [rsp+0x538]
   2abcc:	mov    QWORD PTR [rsp+0x8],rax
   2abd1:	mov    rbx,r12
   2abd4:	shl    rbx,0x5
   2abd8:	mov    rax,QWORD PTR [rsp+0x20]
   2abdd:	mov    rbp,QWORD PTR [rax+0x38]
   2abe1:	mov    rax,QWORD PTR [rax+0x40]
   2abe5:	mov    QWORD PTR [rsp+0x10],rax
   2abea:	xor    r13d,r13d
   2abed:	lea    rax,[rip+0x3f664]        # 6a258 <__frame_dummy_init_array_entry+0x8d8>
   2abf4:	mov    QWORD PTR [rsp+0x30],rax
   2abf9:	jmp    2ac5e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x126e>
   2abfb:	mov    eax,0x8
   2ac00:	mov    QWORD PTR [rsp+0x8],rax
   2ac05:	xor    ebx,ebx
   2ac07:	jmp    2ae96 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14a6>
   2ac0c:	mov    rdi,r14
   2ac0f:	mov    rbx,r9
   2ac12:	call   2b3d0 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCs7ivwnIpGHJL_5mibz2>
   2ac17:	mov    r9,rbx
   2ac1a:	mov    esi,DWORD PTR [rsp+0x4c]
   2ac1e:	mov    rbx,QWORD PTR [r14+0x8]
   2ac22:	jmp    29adb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xeb>
   2ac27:	mov    rax,QWORD PTR [rsp+0x40]
   2ac2c:	mov    rdx,QWORD PTR [rax]
   2ac2f:	mov    rax,QWORD PTR [rax+0x8]
   2ac33:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2ac40:	add    r14,rax
   2ac43:	mov    rcx,QWORD PTR [rsp+0x8]
   2ac48:	mov    QWORD PTR [rcx+r13*1+0x8],r14
   2ac4d:	add    rdx,rax
   2ac50:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   2ac55:	add    r13,0x20
   2ac59:	cmp    rbx,r13
   2ac5c:	je     2accc <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12dc>
   2ac5e:	mov    rax,QWORD PTR [rsp+0x8]
   2ac63:	mov    r14,QWORD PTR [rax+r13*1+0x8]
   2ac68:	cmp    r14,QWORD PTR [rax+r13*1+0x10]
   2ac6d:	jne    2ac55 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1265>
   2ac6f:	mov    rax,QWORD PTR [rsp+0x8]
   2ac74:	mov    rdi,QWORD PTR [rax+r13*1]
   2ac78:	cmp    rdi,QWORD PTR [rsp+0x10]
   2ac7d:	jae    2b16d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x177d>
   2ac83:	lea    rcx,[rdi+rdi*2]
   2ac87:	mov    rdx,QWORD PTR [rbp+rcx*8+0x0]
   2ac8c:	mov    rax,QWORD PTR [rbp+rcx*8+0x8]
   2ac91:	sub    r14,rax
   2ac94:	mov    QWORD PTR [rbp+rcx*8+0x10],r14
   2ac99:	mov    rsi,rdx
   2ac9c:	sub    rsi,r14
   2ac9f:	cmp    rdx,rsi
   2aca2:	jbe    2ac40 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1250>
   2aca4:	lea    rdi,[rcx*8+0x0]
   2acac:	add    rdi,rbp
   2acaf:	mov    ecx,0x1
   2acb4:	mov    r8d,0x1
   2acba:	mov    QWORD PTR [rsp+0x40],rdi
   2acbf:	mov    rsi,r14
   2acc2:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2acc7:	jmp    2ac27 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1237>
   2accc:	mov    rax,QWORD PTR [rsp+0x20]
   2acd1:	mov    rcx,QWORD PTR [rax+0x8]
   2acd5:	mov    QWORD PTR [rsp+0x38],rcx
   2acda:	mov    rcx,QWORD PTR [rax+0x10]
   2acde:	mov    QWORD PTR [rsp+0x40],rcx
   2ace3:	mov    rcx,QWORD PTR [rax+0x50]
   2ace7:	mov    QWORD PTR [rsp+0x60],rcx
   2acec:	mov    rax,QWORD PTR [rax+0x58]
   2acf0:	mov    QWORD PTR [rsp+0x28],rax
   2acf5:	xor    r13d,r13d
   2acf8:	jmp    2ad03 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   2acfa:	nop    WORD PTR [rax+rax*1+0x0]
   2ad00:	xor    r13d,r13d
   2ad03:	mov    rbx,r13
   2ad06:	shl    rbx,0x5
   2ad0a:	add    rbx,QWORD PTR [rsp+0x8]
   2ad0f:	jmp    2ad3a <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x134a>
   2ad11:	mov    rax,QWORD PTR [rsp+0x58]
   2ad16:	mov    rdx,QWORD PTR [rax]
   2ad19:	mov    rax,QWORD PTR [rax+0x8]
   2ad1d:	nop    DWORD PTR [rax]
   2ad20:	add    r14,rax
   2ad23:	mov    QWORD PTR [rbx+0x8],r14
   2ad27:	add    rdx,rax
   2ad2a:	mov    QWORD PTR [rbx+0x10],rdx
   2ad2e:	inc    r13
   2ad31:	add    rbx,0x20
   2ad35:	cmp    r13,r12
   2ad38:	jae    2ad00 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1310>
   2ad3a:	mov    eax,DWORD PTR [rbx+0x18]
   2ad3d:	mov    ecx,DWORD PTR [r15+rax*4]
   2ad41:	test   ecx,ecx
   2ad43:	js     2adc0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13d0>
   2ad45:	mov    eax,ecx
   2ad47:	shr    eax,0x8
   2ad4a:	mov    DWORD PTR [rbx+0x18],eax
   2ad4d:	prefetcht0 BYTE PTR [r15+rax*4]
   2ad52:	mov    rax,QWORD PTR [rbx+0x8]
   2ad56:	mov    BYTE PTR [rax],cl
   2ad58:	mov    r14,QWORD PTR [rbx+0x8]
   2ad5c:	inc    r14
   2ad5f:	mov    QWORD PTR [rbx+0x8],r14
   2ad63:	cmp    r14,QWORD PTR [rbx+0x10]
   2ad67:	jne    2ad2e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x133e>
   2ad69:	mov    rdi,QWORD PTR [rbx]
   2ad6c:	cmp    rdi,QWORD PTR [rsp+0x10]
   2ad71:	jae    2b16d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x177d>
   2ad77:	lea    rcx,[rdi+rdi*2]
   2ad7b:	mov    rdx,QWORD PTR [rbp+rcx*8+0x0]
   2ad80:	mov    rax,QWORD PTR [rbp+rcx*8+0x8]
   2ad85:	sub    r14,rax
   2ad88:	mov    QWORD PTR [rbp+rcx*8+0x10],r14
   2ad8d:	mov    rsi,rdx
   2ad90:	sub    rsi,r14
   2ad93:	cmp    rdx,rsi
   2ad96:	jbe    2ad20 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1330>
   2ad98:	lea    rdi,[rcx*8+0x0]
   2ada0:	add    rdi,rbp
   2ada3:	mov    ecx,0x1
   2ada8:	mov    r8d,0x1
   2adae:	mov    QWORD PTR [rsp+0x58],rdi
   2adb3:	mov    rsi,r14
   2adb6:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2adbb:	jmp    2ad11 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1321>
   2adc0:	mov    rdi,QWORD PTR [rbx]
   2adc3:	cmp    rdi,QWORD PTR [rsp+0x10]
   2adc8:	jae    2b14e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x175e>
   2adce:	lea    rcx,[rdi+rdi*2]
   2add2:	mov    rdx,QWORD PTR [rbx+0x8]
   2add6:	sub    rdx,QWORD PTR [rbp+rcx*8+0x8]
   2addb:	mov    QWORD PTR [rbp+rcx*8+0x10],rdx
   2ade0:	mov    rdx,QWORD PTR [rsp+0x40]
   2ade5:	test   rdx,rdx
   2ade8:	je     2adf9 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1409>
   2adea:	xor    ecx,ecx
   2adec:	cmp    rdx,0x1
   2adf0:	jne    2ae02 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1412>
   2adf2:	mov    r9,QWORD PTR [rsp+0x38]
   2adf7:	jmp    2ae2d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x143d>
   2adf9:	mov    rcx,0xffffffffffffffff
   2ae00:	jmp    2ae3c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x144c>
   2ae02:	mov    r9,QWORD PTR [rsp+0x38]
   2ae07:	nop    WORD PTR [rax+rax*1+0x0]
   2ae10:	mov    rsi,rcx
   2ae13:	mov    r8,rdx
   2ae16:	shr    r8,1
   2ae19:	add    rcx,r8
   2ae1c:	cmp    DWORD PTR [r9+rcx*4],eax
   2ae20:	cmova  rcx,rsi
   2ae24:	sub    rdx,r8
   2ae27:	cmp    rdx,0x1
   2ae2b:	ja     2ae10 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1420>
   2ae2d:	cmp    DWORD PTR [r9+rcx*4],eax
   2ae31:	mov    rax,0xffffffffffffffff
   2ae38:	cmovne rcx,rax
   2ae3c:	cmp    rdi,QWORD PTR [rsp+0x28]
   2ae41:	jae    2b157 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1767>
   2ae47:	mov    rax,QWORD PTR [rsp+0x60]
   2ae4c:	mov    QWORD PTR [rax+rdi*8],rcx
   2ae50:	dec    r12
   2ae53:	mov    rax,r12
   2ae56:	shl    rax,0x5
   2ae5a:	mov    rcx,QWORD PTR [rsp+0x8]
   2ae5f:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   2ae63:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   2ae68:	movups XMMWORD PTR [rbx+0x10],xmm1
   2ae6c:	movups XMMWORD PTR [rbx],xmm0
   2ae6f:	cmp    r13,r12
   2ae72:	jb     2ad03 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   2ae78:	mov    r13d,0x0
   2ae7e:	test   r12,r12
   2ae81:	jne    2ad03 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   2ae87:	mov    r9,QWORD PTR [rsp+0x38]
   2ae8c:	mov    r10,QWORD PTR [rsp+0x40]
   2ae91:	mov    rbx,QWORD PTR [rsp+0x18]
   2ae96:	mov    rax,0xffffffffffffffff
   2ae9d:	test   r10,r10
   2aea0:	mov    r8,QWORD PTR [rsp+0x20]
   2aea5:	mov    rsi,QWORD PTR [rsp+0x50]
   2aeaa:	je     2af2c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x153c>
   2aeb0:	lea    rcx,[r10*4+0x0]
   2aeb8:	xor    edx,edx
   2aeba:	nop    WORD PTR [rax+rax*1+0x0]
   2aec0:	mov    edi,DWORD PTR [r9+rdx*1]
   2aec4:	cmp    rsi,rdi
   2aec7:	jbe    2b186 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1796>
   2aecd:	and    BYTE PTR [r15+rdi*4+0x3],0x7f
   2aed3:	add    rdx,0x4
   2aed7:	cmp    rcx,rdx
   2aeda:	jne    2aec0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14d0>
   2aedc:	test   r10,r10
   2aedf:	je     2af2c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x153c>
   2aee1:	xor    ecx,ecx
   2aee3:	cmp    r10,0x1
   2aee7:	jne    2aeef <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14ff>
   2aee9:	mov    edi,DWORD PTR [rsp+0x4c]
   2aeed:	jmp    2af1d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x152d>
   2aeef:	mov    edi,DWORD PTR [rsp+0x4c]
   2aef3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2af00:	mov    rax,rcx
   2af03:	mov    rdx,r10
   2af06:	shr    rdx,1
   2af09:	add    rcx,rdx
   2af0c:	cmp    DWORD PTR [r9+rcx*4],edi
   2af10:	cmova  rcx,rax
   2af14:	sub    r10,rdx
   2af17:	cmp    r10,0x1
   2af1b:	ja     2af00 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1510>
   2af1d:	cmp    DWORD PTR [r9+rcx*4],edi
   2af21:	mov    rax,0xffffffffffffffff
   2af28:	cmove  rax,rcx
   2af2c:	mov    QWORD PTR [r8+0x68],rax
   2af30:	mov    rax,QWORD PTR [rsp+0xf0]
   2af38:	mov    QWORD PTR [r8+0x60],rax
   2af3c:	test   rbx,rbx
   2af3f:	je     2af4c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x155c>
   2af41:	mov    rdi,QWORD PTR [rsp+0x8]
   2af46:	call   QWORD PTR [rip+0x40f0c]        # 6be58 <free@GLIBC_2.2.5>
   2af4c:	add    rsp,0x2638
   2af53:	pop    rbx
   2af54:	pop    r12
   2af56:	pop    r13
   2af58:	pop    r14
   2af5a:	pop    r15
   2af5c:	pop    rbp
   2af5d:	ret
   2af5e:	lea    rax,[rsp+0xf0]
   2af66:	mov    QWORD PTR [rsp+0x630],rax
   2af6e:	lea    rax,[rip+0xffffffffffffd74b]        # 286c0 <_RNvXsd_NtNtNtCscI6d9CVNmLh_4core3fmt3num3impyNtB9_7Display3fmt>
   2af75:	mov    QWORD PTR [rsp+0x638],rax
   2af7d:	lea    rdi,[rip+0xfffffffffffdd126]        # 80aa <GCC_except_table475+0x33b2>
   2af84:	lea    rdx,[rip+0x3f14d]        # 6a0d8 <__frame_dummy_init_array_entry+0x758>
   2af8b:	lea    rsi,[rsp+0x630]
   2af93:	call   1f6b0 <_RNvNtCscI6d9CVNmLh_4core9panicking9panic_fmt>
   2af98:	mov    edx,0x1ff
   2af9d:	mov    rbx,rcx
   2afa0:	mov    ecx,0x4
   2afa5:	mov    r8d,0x4
   2afab:	mov    ebp,esi
   2afad:	mov    rdi,rbx
   2afb0:	xor    esi,esi
   2afb2:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2afb7:	mov    esi,ebp
   2afb9:	mov    rcx,rbx
   2afbc:	mov    r9,QWORD PTR [rbx+0x10]
   2afc0:	jmp    29a6d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x7d>
   2afc5:	mov    rsi,r13
   2afc8:	cmp    r13,0x15
   2afcc:	jae    2b229 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1839>
   2afd2:	lea    rax,[rbx+rsi*4]
   2afd6:	lea    rcx,[rbx+0x4]
   2afda:	mov    edx,0x4
   2afdf:	mov    rbp,QWORD PTR [rsp+0x20]
   2afe4:	jmp    2affc <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x160c>
   2afe6:	mov    rdi,rbx
   2afe9:	mov    DWORD PTR [rdi],esi
   2afeb:	add    rcx,0x4
   2afef:	add    rdx,0x4
   2aff3:	cmp    rcx,rax
   2aff6:	je     2b23c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x184c>
   2affc:	mov    r8d,DWORD PTR [rcx-0x4]
   2b000:	mov    esi,DWORD PTR [rcx]
   2b002:	cmp    esi,r8d
   2b005:	jae    2afeb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15fb>
   2b007:	mov    rdi,rdx
   2b00a:	mov    DWORD PTR [rbx+rdi*1],r8d
   2b00e:	cmp    rdi,0x4
   2b012:	je     2afe6 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15f6>
   2b014:	mov    r8d,DWORD PTR [rbx+rdi*1-0x8]
   2b019:	add    rdi,0xfffffffffffffffc
   2b01d:	cmp    esi,r8d
   2b020:	jb     2b00a <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x161a>
   2b022:	add    rdi,rbx
   2b025:	jmp    2afe9 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15f9>
   2b027:	lea    rdi,[rbp+0x48]
   2b02b:	mov    ecx,0x8
   2b030:	mov    r8d,0x8
   2b036:	xor    esi,esi
   2b038:	mov    rdx,r13
   2b03b:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2b040:	mov    rbx,QWORD PTR [rbp+0x58]
   2b044:	mov    r14,QWORD PTR [rbp+0x50]
   2b048:	lea    rdi,[r14+rbx*8]
   2b04c:	cmp    r13,0x1
   2b050:	jne    29c2d <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x23d>
   2b056:	jmp    29c52 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x262>
   2b05b:	lea    rdi,[rbp+0x30]
   2b05f:	mov    ecx,0x8
   2b064:	mov    r8d,0x18
   2b06a:	mov    rsi,r14
   2b06d:	mov    rdx,rbx
   2b070:	call   2b350 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs7ivwnIpGHJL_5mibz2>
   2b075:	mov    rdx,QWORD PTR [rbp+0x40]
   2b079:	mov    rax,QWORD PTR [rbp+0x38]
   2b07d:	inc    r14
   2b080:	test   bl,0x1
   2b083:	je     29b96 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a6>
   2b089:	dec    rbx
   2b08c:	lea    rcx,[rdx+rdx*2]
   2b090:	mov    QWORD PTR [rax+rcx*8],0x0
   2b098:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   2b0a1:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   2b0aa:	inc    rdx
   2b0ad:	cmp    r13,r14
   2b0b0:	je     29b9f <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1af>
   2b0b6:	lea    rcx,[rdx+rdx*2]
   2b0ba:	lea    rax,[rax+rcx*8]
   2b0be:	add    rax,0x28
   2b0c2:	neg    rbx
   2b0c5:	xor    ecx,ecx
   2b0c7:	xorps  xmm0,xmm0
   2b0ca:	nop    WORD PTR [rax+rax*1+0x0]
   2b0d0:	mov    QWORD PTR [rax-0x28],0x0
   2b0d8:	mov    QWORD PTR [rax-0x20],0x1
   2b0e0:	movups XMMWORD PTR [rax-0x18],xmm0
   2b0e4:	mov    QWORD PTR [rax-0x8],0x1
   2b0ec:	mov    QWORD PTR [rax],0x0
   2b0f3:	add    rax,0x30
   2b0f7:	add    rcx,0xfffffffffffffffe
   2b0fb:	cmp    rbx,rcx
   2b0fe:	jne    2b0d0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16e0>
   2b100:	sub    rdx,rcx
   2b103:	mov    QWORD PTR [rbp+0x40],rdx
   2b107:	cmp    r13,rdx
   2b10a:	jbe    29bac <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc>
   2b110:	jmp    2a98f <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf9f>
   2b115:	lea    rdx,[rip+0x3f13c]        # 6a258 <__frame_dummy_init_array_entry+0x8d8>
   2b11c:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b121:	lea    rdx,[rip+0x3efe0]        # 6a108 <__frame_dummy_init_array_entry+0x788>
   2b128:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b12d:	lea    rdx,[rip+0x3f154]        # 6a288 <__frame_dummy_init_array_entry+0x908>
   2b134:	mov    rsi,rdi
   2b137:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b13c:	lea    rdx,[rip+0x3f15d]        # 6a2a0 <__frame_dummy_init_array_entry+0x920>
   2b143:	mov    rdi,r10
   2b146:	mov    rsi,r10
   2b149:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b14e:	lea    rax,[rip+0x3f0d3]        # 6a228 <__frame_dummy_init_array_entry+0x8a8>
   2b155:	jmp    2b168 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1778>
   2b157:	mov    rax,QWORD PTR [rsp+0x28]
   2b15c:	mov    QWORD PTR [rsp+0x10],rax
   2b161:	lea    rax,[rip+0x3f0d8]        # 6a240 <__frame_dummy_init_array_entry+0x8c0>
   2b168:	mov    QWORD PTR [rsp+0x30],rax
   2b16d:	mov    rsi,QWORD PTR [rsp+0x10]
   2b172:	mov    rdx,QWORD PTR [rsp+0x30]
   2b177:	mov    rbx,QWORD PTR [rsp+0x18]
   2b17c:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b181:	jmp    2b20c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181c>
   2b186:	lea    rdx,[rip+0x3efc3]        # 6a150 <__frame_dummy_init_array_entry+0x7d0>
   2b18d:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b192:	jmp    2b20c <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181c>
   2b194:	lea    rdx,[rip+0x3f0a5]        # 6a240 <__frame_dummy_init_array_entry+0x8c0>
   2b19b:	mov    rsi,QWORD PTR [rsp+0x28]
   2b1a0:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1a5:	lea    rdx,[rip+0x3f07c]        # 6a228 <__frame_dummy_init_array_entry+0x8a8>
   2b1ac:	mov    rsi,r8
   2b1af:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1b4:	lea    rdx,[rip+0x3ef7d]        # 6a138 <__frame_dummy_init_array_entry+0x7b8>
   2b1bb:	mov    esi,0x20
   2b1c0:	mov    rdi,r14
   2b1c3:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1c8:	lea    rdx,[rip+0x3f0d1]        # 6a2a0 <__frame_dummy_init_array_entry+0x920>
   2b1cf:	mov    rdi,r9
   2b1d2:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1d7:	lea    rdx,[rip+0x3f0aa]        # 6a288 <__frame_dummy_init_array_entry+0x908>
   2b1de:	mov    rdi,r9
   2b1e1:	mov    rsi,r8
   2b1e4:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1e9:	lea    rdx,[rip+0x3f080]        # 6a270 <__frame_dummy_init_array_entry+0x8f0>
   2b1f0:	mov    rdi,rbx
   2b1f3:	mov    rsi,QWORD PTR [rsp+0x28]
   2b1f8:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b1fd:	lea    rdx,[rip+0x3f06c]        # 6a270 <__frame_dummy_init_array_entry+0x8f0>
   2b204:	mov    rdi,r14
   2b207:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b20c:	ud2
   2b20e:	lea    rdx,[rip+0x3eedb]        # 6a0f0 <__frame_dummy_init_array_entry+0x770>
   2b215:	call   2040b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2b21a:	mov    edi,0x8
   2b21f:	mov    esi,0x80
   2b224:	call   1f054 <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   2b229:	mov    rdi,rbx
   2b22c:	mov    r14,r9
   2b22f:	call   2b440 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECs7ivwnIpGHJL_5mibz2>
   2b234:	mov    r9,r14
   2b237:	mov    rbp,QWORD PTR [rsp+0x20]
   2b23c:	xor    ecx,ecx
   2b23e:	mov    eax,DWORD PTR [rbx+rcx*4+0x4]
   2b242:	cmp    eax,DWORD PTR [rbx+rcx*4]
   2b245:	je     2b254 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1864>
   2b247:	inc    rcx
   2b24a:	cmp    r12,rcx
   2b24d:	jne    2b23e <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x184e>
   2b24f:	jmp    29af9 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x109>
   2b254:	lea    rdx,[rcx+0x1]
   2b258:	lea    rax,[rcx+0x2]
   2b25c:	cmp    rax,r13
   2b25f:	jae    2b275 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1885>
   2b261:	mov    esi,r9d
   2b264:	sub    esi,ecx
   2b266:	add    r9,0x1fd
   2b26d:	test   sil,0x1
   2b271:	jne    2b27f <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x188f>
   2b273:	jmp    2b2a0 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18b0>
   2b275:	mov    r13,rdx
   2b278:	mov    rsi,QWORD PTR [rsp+0x50]
   2b27d:	jmp    2b2aa <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ba>
   2b27f:	mov    eax,DWORD PTR [rbx+rcx*4+0x8]
   2b283:	mov    rsi,rdx
   2b286:	cmp    eax,DWORD PTR [rbx+rcx*4]
   2b289:	je     2b293 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18a3>
   2b28b:	mov    DWORD PTR [rbx+rcx*4+0x4],eax
   2b28f:	lea    rsi,[rdx+0x1]
   2b293:	add    rdx,0x2
   2b297:	mov    rax,rdx
   2b29a:	mov    r13,rsi
   2b29d:	mov    rdx,rsi
   2b2a0:	cmp    r9,rcx
   2b2a3:	mov    rsi,QWORD PTR [rsp+0x50]
   2b2a8:	jne    2b2c3 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d3>
   2b2aa:	mov    QWORD PTR [rbp+0x10],r13
   2b2ae:	test   r13,r13
   2b2b1:	jne    29b07 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x117>
   2b2b7:	lea    rdi,[rip+0x3ee62]        # 6a120 <__frame_dummy_init_array_entry+0x7a0>
   2b2be:	call   22f80 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   2b2c3:	inc    r12
   2b2c6:	mov    r13,rdx
   2b2c9:	jmp    2b2d4 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18e4>
   2b2cb:	add    rax,0x2
   2b2cf:	cmp    r12,rax
   2b2d2:	je     2b2aa <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ba>
   2b2d4:	mov    ecx,DWORD PTR [rbx+rax*4]
   2b2d7:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   2b2dc:	jne    2b2eb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18fb>
   2b2de:	mov    ecx,DWORD PTR [rbx+rax*4+0x4]
   2b2e2:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   2b2e7:	je     2b2cb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   2b2e9:	jmp    2b2fd <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x190d>
   2b2eb:	mov    DWORD PTR [rbx+r13*4],ecx
   2b2ef:	inc    r13
   2b2f2:	mov    ecx,DWORD PTR [rbx+rax*4+0x4]
   2b2f6:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   2b2fb:	je     2b2cb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   2b2fd:	mov    DWORD PTR [rbx+r13*4],ecx
   2b301:	inc    r13
   2b304:	jmp    2b2cb <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   2b306:	jmp    2b30a <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191a>
   2b308:	jmp    2b314 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1924>
   2b30a:	mov    r14,rax
   2b30d:	mov    rbx,QWORD PTR [rsp+0x18]
   2b312:	jmp    2b334 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1944>
   2b314:	mov    r14,rax
   2b317:	cmp    QWORD PTR [rsp+0x530],0x0
   2b320:	je     2b344 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1954>
   2b322:	mov    rax,QWORD PTR [rsp+0x538]
   2b32a:	mov    QWORD PTR [rsp+0x8],rax
   2b32f:	jmp    2b339 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1949>
   2b331:	mov    r14,rax
   2b334:	test   rbx,rbx
   2b337:	je     2b344 <_RINvNtCs7ivwnIpGHJL_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1954>
   2b339:	mov    rdi,QWORD PTR [rsp+0x8]
   2b33e:	call   QWORD PTR [rip+0x40b14]        # 6be58 <free@GLIBC_2.2.5>
   2b344:	mov    rdi,r14
   2b347:	call   68960 <_Unwind_Resume@plt>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
