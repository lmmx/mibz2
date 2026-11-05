
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/target/release/pwsim:     file format elf64-x86-64


Disassembly of section .text:

0000000000022cd0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   22cd0:	push   rbp
   22cd1:	push   r15
   22cd3:	push   r14
   22cd5:	push   r13
   22cd7:	push   r12
   22cd9:	push   rbx
   22cda:	sub    rsp,0x1000
   22ce1:	mov    QWORD PTR [rsp],0x0
   22ce9:	sub    rsp,0x1000
   22cf0:	mov    QWORD PTR [rsp],0x0
   22cf8:	sub    rsp,0x638
   22cff:	mov    QWORD PTR [rsp+0xf0],rsi
   22d07:	cmp    rsi,0x800001
   22d0e:	jae    2423e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x156e>
   22d14:	mov    r15,rdi
   22d17:	mov    edi,edx
   22d19:	cmp    rsi,rdi
   22d1c:	jbe    244ee <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181e>
   22d22:	mov    QWORD PTR [rsp+0x50],rsi
   22d27:	mov    esi,DWORD PTR [r15+rdi*4]
   22d2b:	mov    QWORD PTR [rcx+0x10],0x0
   22d33:	movabs r14,0x2545f4914f6cdd1d
   22d3d:	cmp    QWORD PTR [rcx],0x1fe
   22d44:	jbe    24278 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15a8>
   22d4a:	xor    r9d,r9d
   22d4d:	shr    esi,0x8
   22d50:	mov    rdi,QWORD PTR [rsp+0x50]
   22d55:	xor    r14,rdi
   22d58:	mov    QWORD PTR [rsp+0x20],rcx
   22d5d:	mov    rbx,QWORD PTR [rcx+0x8]
   22d61:	mov    ecx,0x1ff
   22d66:	mov    r12,r9
   22d69:	nop    DWORD PTR [rax+0x0]
   22d70:	mov    rbp,r12
   22d73:	mov    rax,r14
   22d76:	shl    rax,0xd
   22d7a:	xor    rax,r14
   22d7d:	mov    rdx,rax
   22d80:	shr    rdx,0x7
   22d84:	xor    rdx,rax
   22d87:	mov    r14,rdx
   22d8a:	shl    r14,0x11
   22d8e:	xor    r14,rdx
   22d91:	mov    rax,r14
   22d94:	xor    edx,edx
   22d96:	div    rdi
   22d99:	mov    DWORD PTR [rbx+r12*4],edx
   22d9d:	inc    r12
   22da0:	dec    rcx
   22da3:	jne    22d70 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xa0>
   22da5:	mov    r14,QWORD PTR [rsp+0x20]
   22daa:	mov    QWORD PTR [r14+0x10],r12
   22dae:	cmp    r12,QWORD PTR [r14]
   22db1:	mov    DWORD PTR [rsp+0x4c],esi
   22db5:	je     23eec <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x121c>
   22dbb:	mov    DWORD PTR [rbx+r12*4],esi
   22dbf:	lea    rax,[rbp+0x2]
   22dc3:	mov    r13,rax
   22dc6:	mov    QWORD PTR [r14+0x10],rax
   22dca:	cmp    rbp,0xfffffffffffffffd
   22dce:	jbe    242a5 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15d5>
   22dd4:	mov    rbp,QWORD PTR [rsp+0x20]
   22dd9:	mov    rsi,QWORD PTR [rsp+0x50]
   22dde:	test   r13,r13
   22de1:	je     24597 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18c7>
   22de7:	lea    rax,[r13*4+0x0]
   22def:	xor    ecx,ecx
   22df1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   22e00:	mov    edi,DWORD PTR [rbx+rcx*1]
   22e03:	cmp    rsi,rdi
   22e06:	jbe    24401 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1731>
   22e0c:	or     BYTE PTR [r15+rdi*4+0x3],0x80
   22e12:	add    rcx,0x4
   22e16:	cmp    rax,rcx
   22e19:	jne    22e00 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x130>
   22e1b:	lea    rax,[rsi+rsi*1]
   22e1f:	mov    rcx,rax
   22e22:	or     rcx,r13
   22e25:	shr    rcx,0x20
   22e29:	je     22e32 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x162>
   22e2b:	xor    edx,edx
   22e2d:	div    r13
   22e30:	jmp    22e37 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x167>
   22e32:	xor    edx,edx
   22e34:	div    r13d
   22e37:	mov    QWORD PTR [rsp+0x58],rax
   22e3c:	mov    r14,QWORD PTR [rbp+0x40]
   22e40:	cmp    r14,r13
   22e43:	mov    rbx,r13
   22e46:	cmova  rbx,r14
   22e4a:	jae    23c5f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf8f>
   22e50:	sub    rbx,r14
   22e53:	mov    rax,QWORD PTR [rbp+0x30]
   22e57:	sub    rax,r14
   22e5a:	mov    rdx,r14
   22e5d:	cmp    rbx,rax
   22e60:	ja     2433b <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x166b>
   22e66:	mov    rax,QWORD PTR [rbp+0x38]
   22e6a:	inc    r14
   22e6d:	test   bl,0x1
   22e70:	jne    24369 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1699>
   22e76:	cmp    r13,r14
   22e79:	jne    24396 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16c6>
   22e7f:	mov    QWORD PTR [rbp+0x40],rdx
   22e83:	cmp    r13,rdx
   22e86:	ja     23c6f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf9f>
   22e8c:	mov    rax,QWORD PTR [rsp+0x58]
   22e91:	lea    rbx,[rax+0x40]
   22e95:	mov    r14,QWORD PTR [rbp+0x38]
   22e99:	lea    rax,[r13*8+0x0]
   22ea1:	lea    r12,[rax+rax*2]
   22ea5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   22eb0:	mov    QWORD PTR [r14+0x10],0x0
   22eb8:	cmp    rbx,QWORD PTR [r14]
   22ebb:	ja     22ec9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f9>
   22ebd:	add    r14,0x18
   22ec1:	add    r12,0xffffffffffffffe8
   22ec5:	jne    22eb0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e0>
   22ec7:	jmp    22eeb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21b>
   22ec9:	mov    ecx,0x1
   22ece:	mov    r8d,0x1
   22ed4:	mov    rdi,r14
   22ed7:	xor    esi,esi
   22ed9:	mov    rdx,rbx
   22edc:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   22ee1:	add    r14,0x18
   22ee5:	add    r12,0xffffffffffffffe8
   22ee9:	jne    22eb0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e0>
   22eeb:	mov    QWORD PTR [rbp+0x58],0x0
   22ef3:	cmp    r13,QWORD PTR [rbp+0x48]
   22ef7:	ja     24307 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1637>
   22efd:	xor    ebx,ebx
   22eff:	mov    r14,QWORD PTR [rbp+0x50]
   22f03:	lea    rdi,[r14+rbx*8]
   22f07:	cmp    r13,0x1
   22f0b:	je     22f32 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x262>
   22f0d:	lea    rdx,[r13*8-0x8]
   22f15:	mov    esi,0xff
   22f1a:	call   QWORD PTR [rip+0x3b1f0]        # 5e110 <memset@GLIBC_2.2.5>
   22f20:	lea    rax,[rbx+r13*1]
   22f24:	lea    rdi,[r14+rax*8]
   22f28:	add    rdi,0xfffffffffffffff8
   22f2c:	add    rbx,r13
   22f2f:	dec    rbx
   22f32:	mov    QWORD PTR [rdi],0xffffffffffffffff
   22f39:	inc    rbx
   22f3c:	mov    QWORD PTR [rbp+0x58],rbx
   22f40:	xorps  xmm0,xmm0
   22f43:	movups XMMWORD PTR [rsp+0x138],xmm0
   22f4b:	mov    DWORD PTR [rsp+0x148],0x0
   22f56:	movups XMMWORD PTR [rsp+0x158],xmm0
   22f5e:	mov    DWORD PTR [rsp+0x168],0x0
   22f69:	movups XMMWORD PTR [rsp+0x178],xmm0
   22f71:	mov    DWORD PTR [rsp+0x188],0x0
   22f7c:	movups XMMWORD PTR [rsp+0x198],xmm0
   22f84:	mov    DWORD PTR [rsp+0x1a8],0x0
   22f8f:	movups XMMWORD PTR [rsp+0x1b8],xmm0
   22f97:	mov    DWORD PTR [rsp+0x1c8],0x0
   22fa2:	movups XMMWORD PTR [rsp+0x1d8],xmm0
   22faa:	mov    DWORD PTR [rsp+0x1e8],0x0
   22fb5:	movups XMMWORD PTR [rsp+0x1f8],xmm0
   22fbd:	mov    DWORD PTR [rsp+0x208],0x0
   22fc8:	movups XMMWORD PTR [rsp+0x218],xmm0
   22fd0:	mov    DWORD PTR [rsp+0x228],0x0
   22fdb:	movups XMMWORD PTR [rsp+0x238],xmm0
   22fe3:	mov    DWORD PTR [rsp+0x248],0x0
   22fee:	movups XMMWORD PTR [rsp+0x258],xmm0
   22ff6:	mov    DWORD PTR [rsp+0x268],0x0
   23001:	movups XMMWORD PTR [rsp+0x278],xmm0
   23009:	mov    DWORD PTR [rsp+0x288],0x0
   23014:	movups XMMWORD PTR [rsp+0x298],xmm0
   2301c:	mov    DWORD PTR [rsp+0x2a8],0x0
   23027:	movups XMMWORD PTR [rsp+0x2b8],xmm0
   2302f:	mov    DWORD PTR [rsp+0x2c8],0x0
   2303a:	movups XMMWORD PTR [rsp+0x2d8],xmm0
   23042:	mov    DWORD PTR [rsp+0x2e8],0x0
   2304d:	movups XMMWORD PTR [rsp+0x2f8],xmm0
   23055:	mov    DWORD PTR [rsp+0x308],0x0
   23060:	movups XMMWORD PTR [rsp+0x318],xmm0
   23068:	mov    DWORD PTR [rsp+0x328],0x0
   23073:	movups XMMWORD PTR [rsp+0x338],xmm0
   2307b:	mov    DWORD PTR [rsp+0x348],0x0
   23086:	movups XMMWORD PTR [rsp+0x358],xmm0
   2308e:	mov    DWORD PTR [rsp+0x368],0x0
   23099:	movups XMMWORD PTR [rsp+0x378],xmm0
   230a1:	mov    DWORD PTR [rsp+0x388],0x0
   230ac:	movups XMMWORD PTR [rsp+0x398],xmm0
   230b4:	mov    DWORD PTR [rsp+0x3a8],0x0
   230bf:	movups XMMWORD PTR [rsp+0x3b8],xmm0
   230c7:	mov    DWORD PTR [rsp+0x3c8],0x0
   230d2:	movups XMMWORD PTR [rsp+0x3d8],xmm0
   230da:	mov    DWORD PTR [rsp+0x3e8],0x0
   230e5:	movups XMMWORD PTR [rsp+0x3f8],xmm0
   230ed:	mov    DWORD PTR [rsp+0x408],0x0
   230f8:	movups XMMWORD PTR [rsp+0x418],xmm0
   23100:	mov    DWORD PTR [rsp+0x428],0x0
   2310b:	movups XMMWORD PTR [rsp+0x438],xmm0
   23113:	mov    DWORD PTR [rsp+0x448],0x0
   2311e:	movups XMMWORD PTR [rsp+0x458],xmm0
   23126:	mov    DWORD PTR [rsp+0x468],0x0
   23131:	movups XMMWORD PTR [rsp+0x478],xmm0
   23139:	mov    DWORD PTR [rsp+0x488],0x0
   23144:	movups XMMWORD PTR [rsp+0x498],xmm0
   2314c:	mov    DWORD PTR [rsp+0x4a8],0x0
   23157:	movups XMMWORD PTR [rsp+0x4b8],xmm0
   2315f:	mov    DWORD PTR [rsp+0x4c8],0x0
   2316a:	movups XMMWORD PTR [rsp+0x4d8],xmm0
   23172:	mov    DWORD PTR [rsp+0x4e8],0x0
   2317d:	movups XMMWORD PTR [rsp+0x4f8],xmm0
   23185:	mov    DWORD PTR [rsp+0x508],0x0
   23190:	movups XMMWORD PTR [rsp+0x518],xmm0
   23198:	mov    DWORD PTR [rsp+0x528],0x0
   231a3:	mov    QWORD PTR [rsp+0x130],0xffffffffffffffff
   231af:	mov    QWORD PTR [rsp+0x150],0xffffffffffffffff
   231bb:	mov    QWORD PTR [rsp+0x170],0xffffffffffffffff
   231c7:	mov    QWORD PTR [rsp+0x190],0xffffffffffffffff
   231d3:	mov    QWORD PTR [rsp+0x1b0],0xffffffffffffffff
   231df:	mov    QWORD PTR [rsp+0x1d0],0xffffffffffffffff
   231eb:	mov    QWORD PTR [rsp+0x1f0],0xffffffffffffffff
   231f7:	mov    QWORD PTR [rsp+0x210],0xffffffffffffffff
   23203:	mov    QWORD PTR [rsp+0x230],0xffffffffffffffff
   2320f:	mov    QWORD PTR [rsp+0x250],0xffffffffffffffff
   2321b:	mov    QWORD PTR [rsp+0x270],0xffffffffffffffff
   23227:	mov    QWORD PTR [rsp+0x290],0xffffffffffffffff
   23233:	mov    QWORD PTR [rsp+0x2b0],0xffffffffffffffff
   2323f:	mov    QWORD PTR [rsp+0x2d0],0xffffffffffffffff
   2324b:	mov    QWORD PTR [rsp+0x2f0],0xffffffffffffffff
   23257:	mov    QWORD PTR [rsp+0x310],0xffffffffffffffff
   23263:	mov    QWORD PTR [rsp+0x330],0xffffffffffffffff
   2326f:	mov    QWORD PTR [rsp+0x350],0xffffffffffffffff
   2327b:	mov    QWORD PTR [rsp+0x370],0xffffffffffffffff
   23287:	mov    QWORD PTR [rsp+0x390],0xffffffffffffffff
   23293:	mov    QWORD PTR [rsp+0x3b0],0xffffffffffffffff
   2329f:	mov    QWORD PTR [rsp+0x3d0],0xffffffffffffffff
   232ab:	mov    QWORD PTR [rsp+0x3f0],0xffffffffffffffff
   232b7:	mov    QWORD PTR [rsp+0x410],0xffffffffffffffff
   232c3:	mov    QWORD PTR [rsp+0x430],0xffffffffffffffff
   232cf:	mov    QWORD PTR [rsp+0x450],0xffffffffffffffff
   232db:	mov    QWORD PTR [rsp+0x470],0xffffffffffffffff
   232e7:	mov    QWORD PTR [rsp+0x490],0xffffffffffffffff
   232f3:	mov    QWORD PTR [rsp+0x4b0],0xffffffffffffffff
   232ff:	mov    QWORD PTR [rsp+0x4d0],0xffffffffffffffff
   2330b:	mov    QWORD PTR [rsp+0x4f0],0xffffffffffffffff
   23317:	mov    QWORD PTR [rsp+0x510],0xffffffffffffffff
   23323:	mov    rax,QWORD PTR [rbp+0x38]
   23327:	mov    rdi,QWORD PTR [rbp+0x40]
   2332b:	mov    r9,QWORD PTR [rbp+0x8]
   2332f:	mov    r10,QWORD PTR [rbp+0x10]
   23333:	add    rax,0x8
   23337:	xor    r12d,r12d
   2333a:	xor    ecx,ecx
   2333c:	nop    DWORD PTR [rax+0x0]
   23340:	cmp    rdi,rcx
   23343:	je     2440d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x173d>
   23349:	cmp    r10,rcx
   2334c:	je     2441c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x174c>
   23352:	mov    rdx,QWORD PTR [rax]
   23355:	mov    esi,DWORD PTR [r9+rcx*4]
   23359:	mov    esi,DWORD PTR [r15+rsi*4]
   2335d:	mov    BYTE PTR [rdx],sil
   23360:	shr    esi,0x8
   23363:	and    esi,0x7fffff
   23369:	lea    r8,[rdx+0x1]
   2336d:	add    rdx,QWORD PTR [rax-0x8]
   23371:	mov    QWORD PTR [rsp+r12*1+0x130],rcx
   23379:	mov    QWORD PTR [rsp+r12*1+0x138],r8
   23381:	mov    QWORD PTR [rsp+r12*1+0x140],rdx
   23389:	mov    DWORD PTR [rsp+r12*1+0x148],esi
   23391:	prefetcht0 BYTE PTR [r15+rsi*4]
   23396:	lea    rdx,[rcx+0x1]
   2339a:	add    r12,0x20
   2339e:	cmp    rcx,0x1e
   233a2:	ja     233b0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6e0>
   233a4:	add    rax,0x18
   233a8:	mov    rcx,rdx
   233ab:	cmp    rdx,r13
   233ae:	jb     23340 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x670>
   233b0:	cmp    rdx,0x20
   233b4:	jne    23c80 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfb0>
   233ba:	mov    QWORD PTR [rsp+0xf8],rdx
   233c2:	mov    QWORD PTR [rsp+0x128],r13
   233ca:	mov    eax,DWORD PTR [rsp+0x148]
   233d1:	mov    ecx,DWORD PTR [rsp+0x168]
   233d8:	mov    DWORD PTR [rsp+0x68],eax
   233dc:	mov    DWORD PTR [rsp+0x6c],ecx
   233e0:	mov    eax,DWORD PTR [rsp+0x188]
   233e7:	mov    DWORD PTR [rsp+0x70],eax
   233eb:	mov    eax,DWORD PTR [rsp+0x1a8]
   233f2:	mov    DWORD PTR [rsp+0x74],eax
   233f6:	mov    eax,DWORD PTR [rsp+0x1c8]
   233fd:	mov    DWORD PTR [rsp+0x78],eax
   23401:	mov    eax,DWORD PTR [rsp+0x1e8]
   23408:	mov    DWORD PTR [rsp+0x7c],eax
   2340c:	mov    eax,DWORD PTR [rsp+0x208]
   23413:	mov    DWORD PTR [rsp+0x80],eax
   2341a:	mov    eax,DWORD PTR [rsp+0x228]
   23421:	mov    DWORD PTR [rsp+0x84],eax
   23428:	mov    eax,DWORD PTR [rsp+0x248]
   2342f:	mov    DWORD PTR [rsp+0x88],eax
   23436:	mov    eax,DWORD PTR [rsp+0x268]
   2343d:	mov    ecx,DWORD PTR [rsp+0x288]
   23444:	mov    DWORD PTR [rsp+0x8c],eax
   2344b:	mov    DWORD PTR [rsp+0x90],ecx
   23452:	mov    eax,DWORD PTR [rsp+0x2a8]
   23459:	mov    DWORD PTR [rsp+0x94],eax
   23460:	mov    eax,DWORD PTR [rsp+0x2c8]
   23467:	mov    DWORD PTR [rsp+0x98],eax
   2346e:	mov    eax,DWORD PTR [rsp+0x2e8]
   23475:	mov    DWORD PTR [rsp+0x9c],eax
   2347c:	mov    eax,DWORD PTR [rsp+0x308]
   23483:	mov    DWORD PTR [rsp+0xa0],eax
   2348a:	mov    eax,DWORD PTR [rsp+0x328]
   23491:	mov    DWORD PTR [rsp+0xa4],eax
   23498:	mov    eax,DWORD PTR [rsp+0x348]
   2349f:	mov    DWORD PTR [rsp+0xa8],eax
   234a6:	mov    eax,DWORD PTR [rsp+0x368]
   234ad:	mov    DWORD PTR [rsp+0xac],eax
   234b4:	mov    eax,DWORD PTR [rsp+0x388]
   234bb:	mov    DWORD PTR [rsp+0xb0],eax
   234c2:	mov    eax,DWORD PTR [rsp+0x3a8]
   234c9:	mov    DWORD PTR [rsp+0xb4],eax
   234d0:	mov    eax,DWORD PTR [rsp+0x3c8]
   234d7:	mov    DWORD PTR [rsp+0xb8],eax
   234de:	mov    eax,DWORD PTR [rsp+0x3e8]
   234e5:	mov    DWORD PTR [rsp+0xbc],eax
   234ec:	mov    eax,DWORD PTR [rsp+0x408]
   234f3:	mov    DWORD PTR [rsp+0xc0],eax
   234fa:	mov    eax,DWORD PTR [rsp+0x428]
   23501:	mov    DWORD PTR [rsp+0xc4],eax
   23508:	mov    eax,DWORD PTR [rsp+0x448]
   2350f:	mov    DWORD PTR [rsp+0xc8],eax
   23516:	mov    eax,DWORD PTR [rsp+0x468]
   2351d:	mov    DWORD PTR [rsp+0xcc],eax
   23524:	mov    eax,DWORD PTR [rsp+0x488]
   2352b:	mov    DWORD PTR [rsp+0xd0],eax
   23532:	mov    eax,DWORD PTR [rsp+0x4a8]
   23539:	mov    DWORD PTR [rsp+0xd4],eax
   23540:	mov    eax,DWORD PTR [rsp+0x4c8]
   23547:	mov    DWORD PTR [rsp+0xd8],eax
   2354e:	mov    eax,DWORD PTR [rsp+0x4e8]
   23555:	mov    DWORD PTR [rsp+0xdc],eax
   2355c:	mov    eax,DWORD PTR [rsp+0x508]
   23563:	mov    DWORD PTR [rsp+0xe0],eax
   2356a:	mov    eax,DWORD PTR [rsp+0x528]
   23571:	mov    DWORD PTR [rsp+0xe4],eax
   23578:	lea    rdi,[rsp+0x630]
   23580:	mov    edx,0x2000
   23585:	xor    esi,esi
   23587:	call   QWORD PTR [rip+0x3ab83]        # 5e110 <memset@GLIBC_2.2.5>
   2358d:	add    QWORD PTR [rsp+0x58],0x3f
   23593:	mov    rbx,QWORD PTR [rbp+0x38]
   23597:	mov    rsi,QWORD PTR [rbp+0x40]
   2359b:	mov    rax,QWORD PTR [rbp+0x50]
   2359f:	mov    QWORD PTR [rsp+0x60],rax
   235a4:	mov    rax,QWORD PTR [rbp+0x58]
   235a8:	mov    QWORD PTR [rsp+0x28],rax
   235ad:	mov    rax,QWORD PTR [rbp+0x8]
   235b1:	mov    QWORD PTR [rsp+0x38],rax
   235b6:	mov    rax,QWORD PTR [rbp+0x10]
   235ba:	mov    QWORD PTR [rsp+0x40],rax
   235bf:	mov    r9d,0x20
   235c5:	mov    QWORD PTR [rsp+0x100],r12
   235cd:	mov    QWORD PTR [rsp+0x18],rsi
   235d2:	mov    QWORD PTR [rsp+0xe8],rbx
   235da:	mov    QWORD PTR [rsp+0x108],r15
   235e2:	jmp    235f9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x929>
   235e4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   235f0:	test   r13,r13
   235f3:	jne    236ba <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x9ea>
   235f9:	xor    ebp,ebp
   235fb:	mov    r13,QWORD PTR [rsp+0x58]
   23600:	jmp    2363d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x96d>
   23602:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   23610:	add    r14,rax
   23613:	mov    QWORD PTR [rsp+rbp*1+0x138],r14
   2361b:	add    rdx,rax
   2361e:	mov    QWORD PTR [rsp+rbp*1+0x140],rdx
   23626:	sub    rdx,r14
   23629:	add    rbp,0x20
   2362d:	cmp    rdx,r13
   23630:	cmovb  r13,rdx
   23634:	cmp    rbp,0x400
   2363b:	je     235f0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x920>
   2363d:	mov    r14,QWORD PTR [rsp+rbp*1+0x138]
   23645:	mov    rdx,QWORD PTR [rsp+rbp*1+0x140]
   2364d:	cmp    r14,rdx
   23650:	jne    23626 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x956>
   23652:	mov    rdi,QWORD PTR [rsp+rbp*1+0x130]
   2365a:	cmp    rdi,rsi
   2365d:	jae    243f5 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1725>
   23663:	lea    rcx,[rdi+rdi*2]
   23667:	mov    rdx,QWORD PTR [rbx+rcx*8]
   2366b:	mov    rax,QWORD PTR [rbx+rcx*8+0x8]
   23670:	sub    r14,rax
   23673:	mov    QWORD PTR [rbx+rcx*8+0x10],r14
   23678:	mov    rdi,rdx
   2367b:	sub    rdi,r14
   2367e:	cmp    rdx,rdi
   23681:	jbe    23610 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   23683:	lea    r12,[rbx+rcx*8]
   23687:	mov    ecx,0x1
   2368c:	mov    r8d,0x1
   23692:	mov    rdi,r12
   23695:	mov    rsi,r14
   23698:	mov    QWORD PTR [rsp+0x8],r9
   2369d:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   236a2:	mov    r9,QWORD PTR [rsp+0x8]
   236a7:	mov    rsi,QWORD PTR [rsp+0x18]
   236ac:	mov    rdx,QWORD PTR [r12]
   236b0:	mov    rax,QWORD PTR [r12+0x8]
   236b5:	jmp    23610 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   236ba:	xor    ecx,ecx
   236bc:	mov    rsi,r13
   236bf:	sub    rsi,rcx
   236c2:	cmp    rsi,0x100
   236c9:	mov    eax,0x100
   236ce:	cmovae rsi,rax
   236d2:	xorps  xmm0,xmm0
   236d5:	movaps XMMWORD PTR [rsp+0x620],xmm0
   236dd:	movaps XMMWORD PTR [rsp+0x610],xmm0
   236e5:	movaps XMMWORD PTR [rsp+0x600],xmm0
   236ed:	movaps XMMWORD PTR [rsp+0x5f0],xmm0
   236f5:	movaps XMMWORD PTR [rsp+0x5e0],xmm0
   236fd:	movaps XMMWORD PTR [rsp+0x5d0],xmm0
   23705:	movaps XMMWORD PTR [rsp+0x5c0],xmm0
   2370d:	movaps XMMWORD PTR [rsp+0x5b0],xmm0
   23715:	movaps XMMWORD PTR [rsp+0x5a0],xmm0
   2371d:	movaps XMMWORD PTR [rsp+0x590],xmm0
   23725:	movaps XMMWORD PTR [rsp+0x580],xmm0
   2372d:	movaps XMMWORD PTR [rsp+0x570],xmm0
   23735:	movaps XMMWORD PTR [rsp+0x560],xmm0
   2373d:	movaps XMMWORD PTR [rsp+0x550],xmm0
   23745:	movaps XMMWORD PTR [rsp+0x540],xmm0
   2374d:	movaps XMMWORD PTR [rsp+0x530],xmm0
   23755:	cmp    r13,rcx
   23758:	mov    QWORD PTR [rsp+0x110],rcx
   23760:	mov    QWORD PTR [rsp+0x30],rsi
   23765:	jne    23840 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb70>
   2376b:	mov    QWORD PTR [rsp+0x8],r9
   23770:	lea    r12,[rsp+0x538]
   23778:	lea    rbp,[rsp+0x630]
   23780:	xor    ebx,ebx
   23782:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   23790:	mov    rsi,QWORD PTR [r12-0x8]
   23795:	mov    r15,QWORD PTR [rsp+0x30]
   2379a:	mov    r14,r15
   2379d:	sub    r14,rsi
   237a0:	add    rsi,rbp
   237a3:	mov    rdi,QWORD PTR [rsp+rbx*1+0x138]
   237ab:	mov    rdx,r14
   237ae:	call   QWORD PTR [rip+0x3a92c]        # 5e0e0 <memcpy@GLIBC_2.14>
   237b4:	add    QWORD PTR [rsp+rbx*1+0x138],r14
   237bc:	lea    rax,[rbx+0x40]
   237c0:	mov    QWORD PTR [rsp+0x10],rax
   237c5:	mov    rax,QWORD PTR [r12]
   237c9:	sub    r15,rax
   237cc:	lea    rsi,[rax+rbp*1]
   237d0:	add    rsi,0x100
   237d7:	mov    rdi,QWORD PTR [rsp+rbx*1+0x158]
   237df:	mov    rdx,r15
   237e2:	call   QWORD PTR [rip+0x3a8f8]        # 5e0e0 <memcpy@GLIBC_2.14>
   237e8:	add    QWORD PTR [rsp+rbx*1+0x158],r15
   237f0:	add    rbp,0x200
   237f7:	add    r12,0x10
   237fb:	mov    rax,QWORD PTR [rsp+0x10]
   23800:	mov    rbx,rax
   23803:	cmp    rax,0x400
   23809:	jne    23790 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   2380b:	mov    rcx,QWORD PTR [rsp+0x110]
   23813:	add    rcx,QWORD PTR [rsp+0x30]
   23818:	cmp    rcx,r13
   2381b:	mov    rsi,QWORD PTR [rsp+0x18]
   23820:	mov    rbx,QWORD PTR [rsp+0xe8]
   23828:	mov    r9,QWORD PTR [rsp+0x8]
   2382d:	mov    r15,QWORD PTR [rsp+0x108]
   23835:	jb     236bc <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x9ec>
   2383b:	jmp    235f9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x929>
   23840:	lea    rcx,[rsp+0xd30]
   23848:	xor    eax,eax
   2384a:	jmp    23869 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb99>
   2384c:	nop    DWORD PTR [rax+0x0]
   23850:	mov    rcx,QWORD PTR [rsp+0x118]
   23858:	inc    rcx
   2385b:	mov    rsi,QWORD PTR [rsp+0x30]
   23860:	cmp    rax,rsi
   23863:	je     2376b <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xa9b>
   23869:	mov    QWORD PTR [rsp+0x10],rax
   2386e:	mov    QWORD PTR [rsp+0x118],rcx
   23876:	mov    rax,rcx
   23879:	xor    ecx,ecx
   2387b:	xor    ebp,ebp
   2387d:	jmp    2389f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbcf>
   2387f:	nop
   23880:	mov    edx,0x80
   23885:	shl    rdx,cl
   23888:	or     rbp,rdx
   2388b:	add    rcx,0x8
   2388f:	add    rax,0x800
   23895:	cmp    rcx,0x19
   23899:	jae    23a90 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdc0>
   2389f:	mov    edx,DWORD PTR [rsp+rcx*4+0x68]
   238a3:	mov    edx,DWORD PTR [r15+rdx*4]
   238a7:	test   edx,edx
   238a9:	js     239b0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xce0>
   238af:	mov    BYTE PTR [rax-0x700],dl
   238b5:	shr    edx,0x8
   238b8:	mov    DWORD PTR [rsp+rcx*4+0x68],edx
   238bc:	prefetcht0 BYTE PTR [r15+rdx*4]
   238c1:	mov    edx,DWORD PTR [rsp+rcx*4+0x6c]
   238c5:	mov    edx,DWORD PTR [r15+rdx*4]
   238c9:	test   edx,edx
   238cb:	js     239c4 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcf4>
   238d1:	mov    BYTE PTR [rax-0x600],dl
   238d7:	shr    edx,0x8
   238da:	mov    DWORD PTR [rsp+rcx*4+0x6c],edx
   238de:	prefetcht0 BYTE PTR [r15+rdx*4]
   238e3:	mov    edx,DWORD PTR [rsp+rcx*4+0x70]
   238e7:	mov    edx,DWORD PTR [r15+rdx*4]
   238eb:	test   edx,edx
   238ed:	js     239df <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd0f>
   238f3:	mov    BYTE PTR [rax-0x500],dl
   238f9:	shr    edx,0x8
   238fc:	mov    DWORD PTR [rsp+rcx*4+0x70],edx
   23900:	prefetcht0 BYTE PTR [r15+rdx*4]
   23905:	mov    edx,DWORD PTR [rsp+rcx*4+0x74]
   23909:	mov    edx,DWORD PTR [r15+rdx*4]
   2390d:	test   edx,edx
   2390f:	js     239fa <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd2a>
   23915:	mov    BYTE PTR [rax-0x400],dl
   2391b:	shr    edx,0x8
   2391e:	mov    DWORD PTR [rsp+rcx*4+0x74],edx
   23922:	prefetcht0 BYTE PTR [r15+rdx*4]
   23927:	mov    edx,DWORD PTR [rsp+rcx*4+0x78]
   2392b:	mov    edx,DWORD PTR [r15+rdx*4]
   2392f:	test   edx,edx
   23931:	js     23a15 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd45>
   23937:	mov    BYTE PTR [rax-0x300],dl
   2393d:	shr    edx,0x8
   23940:	mov    DWORD PTR [rsp+rcx*4+0x78],edx
   23944:	prefetcht0 BYTE PTR [r15+rdx*4]
   23949:	mov    edx,DWORD PTR [rsp+rcx*4+0x7c]
   2394d:	mov    edx,DWORD PTR [r15+rdx*4]
   23951:	test   edx,edx
   23953:	js     23a30 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd60>
   23959:	mov    BYTE PTR [rax-0x200],dl
   2395f:	shr    edx,0x8
   23962:	mov    DWORD PTR [rsp+rcx*4+0x7c],edx
   23966:	prefetcht0 BYTE PTR [r15+rdx*4]
   2396b:	mov    edx,DWORD PTR [rsp+rcx*4+0x80]
   23972:	mov    edx,DWORD PTR [r15+rdx*4]
   23976:	test   edx,edx
   23978:	js     23a4e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7e>
   2397e:	mov    BYTE PTR [rax-0x100],dl
   23984:	shr    edx,0x8
   23987:	mov    DWORD PTR [rsp+rcx*4+0x80],edx
   2398e:	prefetcht0 BYTE PTR [r15+rdx*4]
   23993:	mov    edx,DWORD PTR [rsp+rcx*4+0x84]
   2399a:	mov    edx,DWORD PTR [r15+rdx*4]
   2399e:	test   edx,edx
   239a0:	jns    23a6c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd9c>
   239a6:	jmp    23880 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbb0>
   239ab:	nop    DWORD PTR [rax+rax*1+0x0]
   239b0:	bts    rbp,rcx
   239b4:	mov    edx,DWORD PTR [rsp+rcx*4+0x6c]
   239b8:	mov    edx,DWORD PTR [r15+rdx*4]
   239bc:	test   edx,edx
   239be:	jns    238d1 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc01>
   239c4:	mov    edx,0x2
   239c9:	shl    rdx,cl
   239cc:	or     rbp,rdx
   239cf:	mov    edx,DWORD PTR [rsp+rcx*4+0x70]
   239d3:	mov    edx,DWORD PTR [r15+rdx*4]
   239d7:	test   edx,edx
   239d9:	jns    238f3 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc23>
   239df:	mov    edx,0x4
   239e4:	shl    rdx,cl
   239e7:	or     rbp,rdx
   239ea:	mov    edx,DWORD PTR [rsp+rcx*4+0x74]
   239ee:	mov    edx,DWORD PTR [r15+rdx*4]
   239f2:	test   edx,edx
   239f4:	jns    23915 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc45>
   239fa:	mov    edx,0x8
   239ff:	shl    rdx,cl
   23a02:	or     rbp,rdx
   23a05:	mov    edx,DWORD PTR [rsp+rcx*4+0x78]
   23a09:	mov    edx,DWORD PTR [r15+rdx*4]
   23a0d:	test   edx,edx
   23a0f:	jns    23937 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc67>
   23a15:	mov    edx,0x10
   23a1a:	shl    rdx,cl
   23a1d:	or     rbp,rdx
   23a20:	mov    edx,DWORD PTR [rsp+rcx*4+0x7c]
   23a24:	mov    edx,DWORD PTR [r15+rdx*4]
   23a28:	test   edx,edx
   23a2a:	jns    23959 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc89>
   23a30:	mov    edx,0x20
   23a35:	shl    rdx,cl
   23a38:	or     rbp,rdx
   23a3b:	mov    edx,DWORD PTR [rsp+rcx*4+0x80]
   23a42:	mov    edx,DWORD PTR [r15+rdx*4]
   23a46:	test   edx,edx
   23a48:	jns    2397e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcae>
   23a4e:	mov    edx,0x40
   23a53:	shl    rdx,cl
   23a56:	or     rbp,rdx
   23a59:	mov    edx,DWORD PTR [rsp+rcx*4+0x84]
   23a60:	mov    edx,DWORD PTR [r15+rdx*4]
   23a64:	test   edx,edx
   23a66:	js     23880 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbb0>
   23a6c:	mov    BYTE PTR [rax],dl
   23a6e:	shr    edx,0x8
   23a71:	mov    DWORD PTR [rsp+rcx*4+0x84],edx
   23a78:	prefetcht0 BYTE PTR [r15+rdx*4]
   23a7d:	jmp    2388b <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbbb>
   23a82:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   23a90:	mov    rax,QWORD PTR [rsp+0x10]
   23a95:	inc    rax
   23a98:	test   rbp,rbp
   23a9b:	je     23850 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb80>
   23aa1:	mov    QWORD PTR [rsp+0x120],rax
   23aa9:	nop    DWORD PTR [rax+0x0]
   23ab0:	mov    QWORD PTR [rsp+0x8],r9
   23ab5:	tzcnt  r14,rbp
   23aba:	cmp    r14,0x1f
   23abe:	ja     24494 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17c4>
   23ac4:	mov    ebx,r14d
   23ac7:	shl    ebx,0x5
   23aca:	mov    eax,DWORD PTR [rsp+r14*4+0x68]
   23acf:	mov    DWORD PTR [rsp+rbx*1+0x148],eax
   23ad6:	mov    eax,r14d
   23ad9:	shl    eax,0x8
   23adc:	lea    rsi,[rsp+rax*1]
   23ae0:	add    rsi,0x630
   23ae7:	mov    rax,QWORD PTR [rsp+r14*8+0x530]
   23aef:	mov    r12,QWORD PTR [rsp+0x10]
   23af4:	sub    r12,rax
   23af7:	add    rsi,rax
   23afa:	mov    rdi,QWORD PTR [rsp+rbx*1+0x138]
   23b02:	mov    rdx,r12
   23b05:	call   QWORD PTR [rip+0x3a5d5]        # 5e0e0 <memcpy@GLIBC_2.14>
   23b0b:	add    r12,QWORD PTR [rsp+rbx*1+0x138]
   23b13:	mov    QWORD PTR [rsp+rbx*1+0x138],r12
   23b1b:	mov    rdi,QWORD PTR [rsp+rbx*1+0x130]
   23b23:	mov    r8,QWORD PTR [rsp+0x18]
   23b28:	cmp    rdi,r8
   23b2b:	jae    24485 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17b5>
   23b31:	lea    rax,[rsp+rbx*1]
   23b35:	add    rax,0x130
   23b3b:	lea    rcx,[rdi+rdi*2]
   23b3f:	mov    rbx,QWORD PTR [rsp+0xe8]
   23b47:	sub    r12,QWORD PTR [rbx+rcx*8+0x8]
   23b4c:	mov    QWORD PTR [rbx+rcx*8+0x10],r12
   23b51:	mov    rsi,QWORD PTR [rsp+0x40]
   23b56:	test   rsi,rsi
   23b59:	je     23b70 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xea0>
   23b5b:	mov    edx,DWORD PTR [rax+0x18]
   23b5e:	cmp    rsi,0x1
   23b62:	mov    r10,QWORD PTR [rsp+0x38]
   23b67:	jne    23b80 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xeb0>
   23b69:	xor    ecx,ecx
   23b6b:	jmp    23bad <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xedd>
   23b6d:	nop    DWORD PTR [rax]
   23b70:	mov    rcx,0xffffffffffffffff
   23b77:	mov    r10,QWORD PTR [rsp+0x38]
   23b7c:	jmp    23bc1 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xef1>
   23b7e:	xchg   ax,ax
   23b80:	mov    r11,rsi
   23b83:	xor    ecx,ecx
   23b85:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   23b90:	mov    r8,rcx
   23b93:	mov    r9,r11
   23b96:	shr    r9,1
   23b99:	add    rcx,r9
   23b9c:	cmp    DWORD PTR [r10+rcx*4],edx
   23ba0:	cmova  rcx,r8
   23ba4:	sub    r11,r9
   23ba7:	cmp    r11,0x1
   23bab:	ja     23b90 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xec0>
   23bad:	cmp    DWORD PTR [r10+rcx*4],edx
   23bb1:	mov    rdx,0xffffffffffffffff
   23bb8:	cmovne rcx,rdx
   23bbc:	mov    r8,QWORD PTR [rsp+0x18]
   23bc1:	cmp    rdi,QWORD PTR [rsp+0x28]
   23bc6:	mov    r9,QWORD PTR [rsp+0x8]
   23bcb:	jae    24474 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17a4>
   23bd1:	lea    rdx,[rbp-0x1]
   23bd5:	and    rbp,rdx
   23bd8:	mov    rdx,QWORD PTR [rsp+0x60]
   23bdd:	mov    QWORD PTR [rdx+rdi*8],rcx
   23be1:	cmp    r9,QWORD PTR [rsp+0x128]
   23be9:	je     23c97 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfc7>
   23bef:	cmp    r9,r8
   23bf2:	jae    244b7 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e7>
   23bf8:	cmp    r9,rsi
   23bfb:	jae    244a8 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17d8>
   23c01:	lea    rcx,[r9+r9*2]
   23c05:	mov    rdx,QWORD PTR [rbx+rcx*8+0x8]
   23c0a:	mov    esi,DWORD PTR [r10+r9*4]
   23c0e:	mov    esi,DWORD PTR [r15+rsi*4]
   23c12:	mov    BYTE PTR [rdx],sil
   23c15:	shr    esi,0x8
   23c18:	and    esi,0x7fffff
   23c1e:	lea    rdi,[rdx+0x1]
   23c22:	add    rdx,QWORD PTR [rbx+rcx*8]
   23c26:	mov    QWORD PTR [rax],r9
   23c29:	mov    QWORD PTR [rax+0x8],rdi
   23c2d:	mov    QWORD PTR [rax+0x10],rdx
   23c31:	mov    DWORD PTR [rax+0x18],esi
   23c34:	mov    DWORD PTR [rsp+r14*4+0x68],esi
   23c39:	prefetcht0 BYTE PTR [r15+rsi*4]
   23c3e:	inc    r9
   23c41:	mov    rax,QWORD PTR [rsp+0x120]
   23c49:	mov    QWORD PTR [rsp+r14*8+0x530],rax
   23c51:	test   rbp,rbp
   23c54:	jne    23ab0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xde0>
   23c5a:	jmp    23850 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb80>
   23c5f:	mov    rdx,rbx
   23c62:	mov    QWORD PTR [rbp+0x40],rdx
   23c66:	cmp    r13,rdx
   23c69:	jbe    22e8c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc>
   23c6f:	lea    rcx,[rip+0x38c0a]        # 5c880 <__frame_dummy_init_array_entry+0xb10>
   23c76:	xor    edi,edi
   23c78:	mov    rsi,r13
   23c7b:	call   1b680 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   23c80:	mov    rax,QWORD PTR [rbp+0x50]
   23c84:	mov    QWORD PTR [rsp+0x60],rax
   23c89:	mov    rax,QWORD PTR [rbp+0x58]
   23c8d:	mov    QWORD PTR [rsp+0x28],rax
   23c92:	jmp    23d2c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x105c>
   23c97:	not    rbp
   23c9a:	xor    r12d,r12d
   23c9d:	xor    r13d,r13d
   23ca0:	jmp    23cc0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xff0>
   23ca2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   23cb0:	add    r13,0x20
   23cb4:	inc    r12
   23cb7:	cmp    r13,0x400
   23cbe:	je     23d12 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1042>
   23cc0:	mov    eax,DWORD PTR [rsp+r12*4+0x68]
   23cc5:	mov    DWORD PTR [rsp+r13*1+0x148],eax
   23ccd:	cmp    r14,r12
   23cd0:	je     23cb0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfe0>
   23cd2:	mov    rax,QWORD PTR [rsp+r12*8+0x530]
   23cda:	bt     rbp,r12
   23cde:	mov    rbx,QWORD PTR [rsp+0x10]
   23ce3:	adc    rbx,0x0
   23ce7:	sub    rbx,rax
   23cea:	add    rax,rsp
   23ced:	add    rax,0x630
   23cf3:	lea    rsi,[rax+r13*8]
   23cf7:	mov    rdi,QWORD PTR [rsp+r13*1+0x138]
   23cff:	mov    rdx,rbx
   23d02:	call   QWORD PTR [rip+0x3a3d8]        # 5e0e0 <memcpy@GLIBC_2.14>
   23d08:	add    QWORD PTR [rsp+r13*1+0x138],rbx
   23d10:	jmp    23cb0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xfe0>
   23d12:	mov    r9,QWORD PTR [rsp+0x38]
   23d17:	mov    r10,QWORD PTR [rsp+0x40]
   23d1c:	mov    r12,QWORD PTR [rsp+0x100]
   23d24:	mov    rdx,QWORD PTR [rsp+0xf8]
   23d2c:	shl    rdx,0x5
   23d30:	lea    r13,[rsp+rdx*1]
   23d34:	add    r13,0x130
   23d3b:	lea    r14,[rsp+0x130]
   23d43:	mov    rax,QWORD PTR [rsp+0x60]
   23d48:	jmp    23d5d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x108d>
   23d4a:	nop    WORD PTR [rax+rax*1+0x0]
   23d50:	mov    r14,rbp
   23d53:	add    r12,0xffffffffffffffe0
   23d57:	je     23edb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x120b>
   23d5d:	lea    rbp,[r14+0x20]
   23d61:	mov    rbx,QWORD PTR [r14]
   23d64:	cmp    rbx,0xffffffffffffffff
   23d68:	je     23d50 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1080>
   23d6a:	cmp    rbx,QWORD PTR [rsp+0x28]
   23d6f:	jae    244c9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17f9>
   23d75:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   23d7a:	jne    23d50 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1080>
   23d7c:	mov    edi,0x80
   23d81:	call   QWORD PTR [rip+0x3a351]        # 5e0d8 <malloc@GLIBC_2.2.5>
   23d87:	test   rax,rax
   23d8a:	je     244fa <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x182a>
   23d90:	mov    rcx,QWORD PTR [r14+0x18]
   23d94:	mov    QWORD PTR [rax+0x18],rcx
   23d98:	movups xmm0,XMMWORD PTR [r14+0x8]
   23d9d:	movups XMMWORD PTR [rax+0x8],xmm0
   23da1:	mov    QWORD PTR [rax],rbx
   23da4:	mov    QWORD PTR [rsp+0x530],0x4
   23db0:	mov    QWORD PTR [rsp+0x538],rax
   23db8:	mov    QWORD PTR [rsp+0x540],0x1
   23dc4:	mov    r12d,0x1
   23dca:	cmp    rbp,r13
   23dcd:	je     23e97 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11c7>
   23dd3:	lea    rbx,[rsp+0x530]
   23ddb:	mov    rdx,QWORD PTR [rsp+0x20]
   23de0:	mov    rcx,QWORD PTR [rdx+0x50]
   23de4:	mov    rsi,QWORD PTR [rdx+0x58]
   23de8:	mov    rdx,rbp
   23deb:	jmp    23dfc <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x112c>
   23ded:	nop    DWORD PTR [rax]
   23df0:	mov    rdx,rbp
   23df3:	cmp    rbp,r13
   23df6:	je     23e97 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11c7>
   23dfc:	lea    rbp,[rdx+0x20]
   23e00:	mov    r14,QWORD PTR [rdx]
   23e03:	cmp    r14,0xffffffffffffffff
   23e07:	je     23df0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1120>
   23e09:	cmp    r14,rsi
   23e0c:	jae    244dd <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x180d>
   23e12:	cmp    QWORD PTR [rcx+r14*8],0xffffffffffffffff
   23e17:	jne    23df0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1120>
   23e19:	mov    rcx,QWORD PTR [rdx+0x18]
   23e1d:	mov    QWORD PTR [rsp+0x640],rcx
   23e25:	movups xmm0,XMMWORD PTR [rdx+0x8]
   23e29:	movaps XMMWORD PTR [rsp+0x630],xmm0
   23e31:	cmp    r12,QWORD PTR [rsp+0x530]
   23e39:	jne    23e5e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x118e>
   23e3b:	mov    edx,0x1
   23e40:	mov    ecx,0x8
   23e45:	mov    r8d,0x20
   23e4b:	mov    rdi,rbx
   23e4e:	mov    rsi,r12
   23e51:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   23e56:	mov    rax,QWORD PTR [rsp+0x538]
   23e5e:	mov    rcx,r12
   23e61:	shl    rcx,0x5
   23e65:	mov    QWORD PTR [rax+rcx*1],r14
   23e69:	movaps xmm0,XMMWORD PTR [rsp+0x630]
   23e71:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   23e76:	mov    rdx,QWORD PTR [rsp+0x640]
   23e7e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   23e83:	inc    r12
   23e86:	mov    QWORD PTR [rsp+0x540],r12
   23e8e:	cmp    rbp,r13
   23e91:	jne    23ddb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x110b>
   23e97:	mov    rax,QWORD PTR [rsp+0x530]
   23e9f:	mov    QWORD PTR [rsp+0x18],rax
   23ea4:	mov    rax,QWORD PTR [rsp+0x538]
   23eac:	mov    QWORD PTR [rsp+0x8],rax
   23eb1:	mov    rbx,r12
   23eb4:	shl    rbx,0x5
   23eb8:	mov    rax,QWORD PTR [rsp+0x20]
   23ebd:	mov    rbp,QWORD PTR [rax+0x38]
   23ec1:	mov    rax,QWORD PTR [rax+0x40]
   23ec5:	mov    QWORD PTR [rsp+0x10],rax
   23eca:	xor    r13d,r13d
   23ecd:	lea    rax,[rip+0x389f4]        # 5c8c8 <__frame_dummy_init_array_entry+0xb58>
   23ed4:	mov    QWORD PTR [rsp+0x30],rax
   23ed9:	jmp    23f3e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x126e>
   23edb:	mov    eax,0x8
   23ee0:	mov    QWORD PTR [rsp+0x8],rax
   23ee5:	xor    ebx,ebx
   23ee7:	jmp    24176 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14a6>
   23eec:	mov    rdi,r14
   23eef:	mov    rbx,r9
   23ef2:	call   246b0 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsdLEVLgM5ZIZ_5mibz2>
   23ef7:	mov    r9,rbx
   23efa:	mov    esi,DWORD PTR [rsp+0x4c]
   23efe:	mov    rbx,QWORD PTR [r14+0x8]
   23f02:	jmp    22dbb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xeb>
   23f07:	mov    rax,QWORD PTR [rsp+0x40]
   23f0c:	mov    rdx,QWORD PTR [rax]
   23f0f:	mov    rax,QWORD PTR [rax+0x8]
   23f13:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   23f20:	add    r14,rax
   23f23:	mov    rcx,QWORD PTR [rsp+0x8]
   23f28:	mov    QWORD PTR [rcx+r13*1+0x8],r14
   23f2d:	add    rdx,rax
   23f30:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   23f35:	add    r13,0x20
   23f39:	cmp    rbx,r13
   23f3c:	je     23fac <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12dc>
   23f3e:	mov    rax,QWORD PTR [rsp+0x8]
   23f43:	mov    r14,QWORD PTR [rax+r13*1+0x8]
   23f48:	cmp    r14,QWORD PTR [rax+r13*1+0x10]
   23f4d:	jne    23f35 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1265>
   23f4f:	mov    rax,QWORD PTR [rsp+0x8]
   23f54:	mov    rdi,QWORD PTR [rax+r13*1]
   23f58:	cmp    rdi,QWORD PTR [rsp+0x10]
   23f5d:	jae    2444d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x177d>
   23f63:	lea    rcx,[rdi+rdi*2]
   23f67:	mov    rdx,QWORD PTR [rbp+rcx*8+0x0]
   23f6c:	mov    rax,QWORD PTR [rbp+rcx*8+0x8]
   23f71:	sub    r14,rax
   23f74:	mov    QWORD PTR [rbp+rcx*8+0x10],r14
   23f79:	mov    rsi,rdx
   23f7c:	sub    rsi,r14
   23f7f:	cmp    rdx,rsi
   23f82:	jbe    23f20 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1250>
   23f84:	lea    rdi,[rcx*8+0x0]
   23f8c:	add    rdi,rbp
   23f8f:	mov    ecx,0x1
   23f94:	mov    r8d,0x1
   23f9a:	mov    QWORD PTR [rsp+0x40],rdi
   23f9f:	mov    rsi,r14
   23fa2:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   23fa7:	jmp    23f07 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1237>
   23fac:	mov    rax,QWORD PTR [rsp+0x20]
   23fb1:	mov    rcx,QWORD PTR [rax+0x8]
   23fb5:	mov    QWORD PTR [rsp+0x38],rcx
   23fba:	mov    rcx,QWORD PTR [rax+0x10]
   23fbe:	mov    QWORD PTR [rsp+0x40],rcx
   23fc3:	mov    rcx,QWORD PTR [rax+0x50]
   23fc7:	mov    QWORD PTR [rsp+0x60],rcx
   23fcc:	mov    rax,QWORD PTR [rax+0x58]
   23fd0:	mov    QWORD PTR [rsp+0x28],rax
   23fd5:	xor    r13d,r13d
   23fd8:	jmp    23fe3 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   23fda:	nop    WORD PTR [rax+rax*1+0x0]
   23fe0:	xor    r13d,r13d
   23fe3:	mov    rbx,r13
   23fe6:	shl    rbx,0x5
   23fea:	add    rbx,QWORD PTR [rsp+0x8]
   23fef:	jmp    2401a <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x134a>
   23ff1:	mov    rax,QWORD PTR [rsp+0x58]
   23ff6:	mov    rdx,QWORD PTR [rax]
   23ff9:	mov    rax,QWORD PTR [rax+0x8]
   23ffd:	nop    DWORD PTR [rax]
   24000:	add    r14,rax
   24003:	mov    QWORD PTR [rbx+0x8],r14
   24007:	add    rdx,rax
   2400a:	mov    QWORD PTR [rbx+0x10],rdx
   2400e:	inc    r13
   24011:	add    rbx,0x20
   24015:	cmp    r13,r12
   24018:	jae    23fe0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1310>
   2401a:	mov    eax,DWORD PTR [rbx+0x18]
   2401d:	mov    ecx,DWORD PTR [r15+rax*4]
   24021:	test   ecx,ecx
   24023:	js     240a0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13d0>
   24025:	mov    eax,ecx
   24027:	shr    eax,0x8
   2402a:	mov    DWORD PTR [rbx+0x18],eax
   2402d:	prefetcht0 BYTE PTR [r15+rax*4]
   24032:	mov    rax,QWORD PTR [rbx+0x8]
   24036:	mov    BYTE PTR [rax],cl
   24038:	mov    r14,QWORD PTR [rbx+0x8]
   2403c:	inc    r14
   2403f:	mov    QWORD PTR [rbx+0x8],r14
   24043:	cmp    r14,QWORD PTR [rbx+0x10]
   24047:	jne    2400e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x133e>
   24049:	mov    rdi,QWORD PTR [rbx]
   2404c:	cmp    rdi,QWORD PTR [rsp+0x10]
   24051:	jae    2444d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x177d>
   24057:	lea    rcx,[rdi+rdi*2]
   2405b:	mov    rdx,QWORD PTR [rbp+rcx*8+0x0]
   24060:	mov    rax,QWORD PTR [rbp+rcx*8+0x8]
   24065:	sub    r14,rax
   24068:	mov    QWORD PTR [rbp+rcx*8+0x10],r14
   2406d:	mov    rsi,rdx
   24070:	sub    rsi,r14
   24073:	cmp    rdx,rsi
   24076:	jbe    24000 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1330>
   24078:	lea    rdi,[rcx*8+0x0]
   24080:	add    rdi,rbp
   24083:	mov    ecx,0x1
   24088:	mov    r8d,0x1
   2408e:	mov    QWORD PTR [rsp+0x58],rdi
   24093:	mov    rsi,r14
   24096:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   2409b:	jmp    23ff1 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1321>
   240a0:	mov    rdi,QWORD PTR [rbx]
   240a3:	cmp    rdi,QWORD PTR [rsp+0x10]
   240a8:	jae    2442e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x175e>
   240ae:	lea    rcx,[rdi+rdi*2]
   240b2:	mov    rdx,QWORD PTR [rbx+0x8]
   240b6:	sub    rdx,QWORD PTR [rbp+rcx*8+0x8]
   240bb:	mov    QWORD PTR [rbp+rcx*8+0x10],rdx
   240c0:	mov    rdx,QWORD PTR [rsp+0x40]
   240c5:	test   rdx,rdx
   240c8:	je     240d9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1409>
   240ca:	xor    ecx,ecx
   240cc:	cmp    rdx,0x1
   240d0:	jne    240e2 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1412>
   240d2:	mov    r9,QWORD PTR [rsp+0x38]
   240d7:	jmp    2410d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x143d>
   240d9:	mov    rcx,0xffffffffffffffff
   240e0:	jmp    2411c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x144c>
   240e2:	mov    r9,QWORD PTR [rsp+0x38]
   240e7:	nop    WORD PTR [rax+rax*1+0x0]
   240f0:	mov    rsi,rcx
   240f3:	mov    r8,rdx
   240f6:	shr    r8,1
   240f9:	add    rcx,r8
   240fc:	cmp    DWORD PTR [r9+rcx*4],eax
   24100:	cmova  rcx,rsi
   24104:	sub    rdx,r8
   24107:	cmp    rdx,0x1
   2410b:	ja     240f0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1420>
   2410d:	cmp    DWORD PTR [r9+rcx*4],eax
   24111:	mov    rax,0xffffffffffffffff
   24118:	cmovne rcx,rax
   2411c:	cmp    rdi,QWORD PTR [rsp+0x28]
   24121:	jae    24437 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1767>
   24127:	mov    rax,QWORD PTR [rsp+0x60]
   2412c:	mov    QWORD PTR [rax+rdi*8],rcx
   24130:	dec    r12
   24133:	mov    rax,r12
   24136:	shl    rax,0x5
   2413a:	mov    rcx,QWORD PTR [rsp+0x8]
   2413f:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   24143:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   24148:	movups XMMWORD PTR [rbx+0x10],xmm1
   2414c:	movups XMMWORD PTR [rbx],xmm0
   2414f:	cmp    r13,r12
   24152:	jb     23fe3 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   24158:	mov    r13d,0x0
   2415e:	test   r12,r12
   24161:	jne    23fe3 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1313>
   24167:	mov    r9,QWORD PTR [rsp+0x38]
   2416c:	mov    r10,QWORD PTR [rsp+0x40]
   24171:	mov    rbx,QWORD PTR [rsp+0x18]
   24176:	mov    rax,0xffffffffffffffff
   2417d:	test   r10,r10
   24180:	mov    r8,QWORD PTR [rsp+0x20]
   24185:	mov    rsi,QWORD PTR [rsp+0x50]
   2418a:	je     2420c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x153c>
   24190:	lea    rcx,[r10*4+0x0]
   24198:	xor    edx,edx
   2419a:	nop    WORD PTR [rax+rax*1+0x0]
   241a0:	mov    edi,DWORD PTR [r9+rdx*1]
   241a4:	cmp    rsi,rdi
   241a7:	jbe    24466 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1796>
   241ad:	and    BYTE PTR [r15+rdi*4+0x3],0x7f
   241b3:	add    rdx,0x4
   241b7:	cmp    rcx,rdx
   241ba:	jne    241a0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14d0>
   241bc:	test   r10,r10
   241bf:	je     2420c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x153c>
   241c1:	xor    ecx,ecx
   241c3:	cmp    r10,0x1
   241c7:	jne    241cf <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14ff>
   241c9:	mov    edi,DWORD PTR [rsp+0x4c]
   241cd:	jmp    241fd <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x152d>
   241cf:	mov    edi,DWORD PTR [rsp+0x4c]
   241d3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   241e0:	mov    rax,rcx
   241e3:	mov    rdx,r10
   241e6:	shr    rdx,1
   241e9:	add    rcx,rdx
   241ec:	cmp    DWORD PTR [r9+rcx*4],edi
   241f0:	cmova  rcx,rax
   241f4:	sub    r10,rdx
   241f7:	cmp    r10,0x1
   241fb:	ja     241e0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1510>
   241fd:	cmp    DWORD PTR [r9+rcx*4],edi
   24201:	mov    rax,0xffffffffffffffff
   24208:	cmove  rax,rcx
   2420c:	mov    QWORD PTR [r8+0x68],rax
   24210:	mov    rax,QWORD PTR [rsp+0xf0]
   24218:	mov    QWORD PTR [r8+0x60],rax
   2421c:	test   rbx,rbx
   2421f:	je     2422c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x155c>
   24221:	mov    rdi,QWORD PTR [rsp+0x8]
   24226:	call   QWORD PTR [rip+0x39e9c]        # 5e0c8 <free@GLIBC_2.2.5>
   2422c:	add    rsp,0x2638
   24233:	pop    rbx
   24234:	pop    r12
   24236:	pop    r13
   24238:	pop    r14
   2423a:	pop    r15
   2423c:	pop    rbp
   2423d:	ret
   2423e:	lea    rax,[rsp+0xf0]
   24246:	mov    QWORD PTR [rsp+0x630],rax
   2424e:	lea    rax,[rip+0xffffffffffffd81b]        # 21a70 <_RNvXsd_NtNtNtCscI6d9CVNmLh_4core3fmt3num3impyNtB9_7Display3fmt>
   24255:	mov    QWORD PTR [rsp+0x638],rax
   2425d:	lea    rdi,[rip+0xfffffffffffe3810]        # 7a74 <GCC_except_table453+0x3020>
   24264:	lea    rdx,[rip+0x38585]        # 5c7f0 <__frame_dummy_init_array_entry+0xa80>
   2426b:	lea    rsi,[rsp+0x630]
   24273:	call   19bc0 <_RNvNtCscI6d9CVNmLh_4core9panicking9panic_fmt>
   24278:	mov    edx,0x1ff
   2427d:	mov    rbx,rcx
   24280:	mov    ecx,0x4
   24285:	mov    r8d,0x4
   2428b:	mov    ebp,esi
   2428d:	mov    rdi,rbx
   24290:	xor    esi,esi
   24292:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   24297:	mov    esi,ebp
   24299:	mov    rcx,rbx
   2429c:	mov    r9,QWORD PTR [rbx+0x10]
   242a0:	jmp    22d4d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x7d>
   242a5:	mov    rsi,r13
   242a8:	cmp    r13,0x15
   242ac:	jae    24509 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1839>
   242b2:	lea    rax,[rbx+rsi*4]
   242b6:	lea    rcx,[rbx+0x4]
   242ba:	mov    edx,0x4
   242bf:	mov    rbp,QWORD PTR [rsp+0x20]
   242c4:	jmp    242dc <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x160c>
   242c6:	mov    rdi,rbx
   242c9:	mov    DWORD PTR [rdi],esi
   242cb:	add    rcx,0x4
   242cf:	add    rdx,0x4
   242d3:	cmp    rcx,rax
   242d6:	je     2451c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x184c>
   242dc:	mov    r8d,DWORD PTR [rcx-0x4]
   242e0:	mov    esi,DWORD PTR [rcx]
   242e2:	cmp    esi,r8d
   242e5:	jae    242cb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15fb>
   242e7:	mov    rdi,rdx
   242ea:	mov    DWORD PTR [rbx+rdi*1],r8d
   242ee:	cmp    rdi,0x4
   242f2:	je     242c6 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15f6>
   242f4:	mov    r8d,DWORD PTR [rbx+rdi*1-0x8]
   242f9:	add    rdi,0xfffffffffffffffc
   242fd:	cmp    esi,r8d
   24300:	jb     242ea <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x161a>
   24302:	add    rdi,rbx
   24305:	jmp    242c9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15f9>
   24307:	lea    rdi,[rbp+0x48]
   2430b:	mov    ecx,0x8
   24310:	mov    r8d,0x8
   24316:	xor    esi,esi
   24318:	mov    rdx,r13
   2431b:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   24320:	mov    rbx,QWORD PTR [rbp+0x58]
   24324:	mov    r14,QWORD PTR [rbp+0x50]
   24328:	lea    rdi,[r14+rbx*8]
   2432c:	cmp    r13,0x1
   24330:	jne    22f0d <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x23d>
   24336:	jmp    22f32 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x262>
   2433b:	lea    rdi,[rbp+0x30]
   2433f:	mov    ecx,0x8
   24344:	mov    r8d,0x18
   2434a:	mov    rsi,r14
   2434d:	mov    rdx,rbx
   24350:	call   24630 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdLEVLgM5ZIZ_5mibz2>
   24355:	mov    rdx,QWORD PTR [rbp+0x40]
   24359:	mov    rax,QWORD PTR [rbp+0x38]
   2435d:	inc    r14
   24360:	test   bl,0x1
   24363:	je     22e76 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a6>
   24369:	dec    rbx
   2436c:	lea    rcx,[rdx+rdx*2]
   24370:	mov    QWORD PTR [rax+rcx*8],0x0
   24378:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   24381:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   2438a:	inc    rdx
   2438d:	cmp    r13,r14
   24390:	je     22e7f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1af>
   24396:	lea    rcx,[rdx+rdx*2]
   2439a:	lea    rax,[rax+rcx*8]
   2439e:	add    rax,0x28
   243a2:	neg    rbx
   243a5:	xor    ecx,ecx
   243a7:	xorps  xmm0,xmm0
   243aa:	nop    WORD PTR [rax+rax*1+0x0]
   243b0:	mov    QWORD PTR [rax-0x28],0x0
   243b8:	mov    QWORD PTR [rax-0x20],0x1
   243c0:	movups XMMWORD PTR [rax-0x18],xmm0
   243c4:	mov    QWORD PTR [rax-0x8],0x1
   243cc:	mov    QWORD PTR [rax],0x0
   243d3:	add    rax,0x30
   243d7:	add    rcx,0xfffffffffffffffe
   243db:	cmp    rbx,rcx
   243de:	jne    243b0 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16e0>
   243e0:	sub    rdx,rcx
   243e3:	mov    QWORD PTR [rbp+0x40],rdx
   243e7:	cmp    r13,rdx
   243ea:	jbe    22e8c <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc>
   243f0:	jmp    23c6f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf9f>
   243f5:	lea    rdx,[rip+0x384cc]        # 5c8c8 <__frame_dummy_init_array_entry+0xb58>
   243fc:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   24401:	lea    rdx,[rip+0x38418]        # 5c820 <__frame_dummy_init_array_entry+0xab0>
   24408:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2440d:	lea    rdx,[rip+0x384e4]        # 5c8f8 <__frame_dummy_init_array_entry+0xb88>
   24414:	mov    rsi,rdi
   24417:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2441c:	lea    rdx,[rip+0x384ed]        # 5c910 <__frame_dummy_init_array_entry+0xba0>
   24423:	mov    rdi,r10
   24426:	mov    rsi,r10
   24429:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   2442e:	lea    rax,[rip+0x38463]        # 5c898 <__frame_dummy_init_array_entry+0xb28>
   24435:	jmp    24448 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1778>
   24437:	mov    rax,QWORD PTR [rsp+0x28]
   2443c:	mov    QWORD PTR [rsp+0x10],rax
   24441:	lea    rax,[rip+0x38468]        # 5c8b0 <__frame_dummy_init_array_entry+0xb40>
   24448:	mov    QWORD PTR [rsp+0x30],rax
   2444d:	mov    rsi,QWORD PTR [rsp+0x10]
   24452:	mov    rdx,QWORD PTR [rsp+0x30]
   24457:	mov    rbx,QWORD PTR [rsp+0x18]
   2445c:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   24461:	jmp    244ec <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181c>
   24466:	lea    rdx,[rip+0x383fb]        # 5c868 <__frame_dummy_init_array_entry+0xaf8>
   2446d:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   24472:	jmp    244ec <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181c>
   24474:	lea    rdx,[rip+0x38435]        # 5c8b0 <__frame_dummy_init_array_entry+0xb40>
   2447b:	mov    rsi,QWORD PTR [rsp+0x28]
   24480:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   24485:	lea    rdx,[rip+0x3840c]        # 5c898 <__frame_dummy_init_array_entry+0xb28>
   2448c:	mov    rsi,r8
   2448f:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   24494:	lea    rdx,[rip+0x383b5]        # 5c850 <__frame_dummy_init_array_entry+0xae0>
   2449b:	mov    esi,0x20
   244a0:	mov    rdi,r14
   244a3:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244a8:	lea    rdx,[rip+0x38461]        # 5c910 <__frame_dummy_init_array_entry+0xba0>
   244af:	mov    rdi,r9
   244b2:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244b7:	lea    rdx,[rip+0x3843a]        # 5c8f8 <__frame_dummy_init_array_entry+0xb88>
   244be:	mov    rdi,r9
   244c1:	mov    rsi,r8
   244c4:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244c9:	lea    rdx,[rip+0x38410]        # 5c8e0 <__frame_dummy_init_array_entry+0xb70>
   244d0:	mov    rdi,rbx
   244d3:	mov    rsi,QWORD PTR [rsp+0x28]
   244d8:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244dd:	lea    rdx,[rip+0x383fc]        # 5c8e0 <__frame_dummy_init_array_entry+0xb70>
   244e4:	mov    rdi,r14
   244e7:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244ec:	ud2
   244ee:	lea    rdx,[rip+0x38313]        # 5c808 <__frame_dummy_init_array_entry+0xa98>
   244f5:	call   1a91b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   244fa:	mov    edi,0x8
   244ff:	mov    esi,0x80
   24504:	call   19561 <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   24509:	mov    rdi,rbx
   2450c:	mov    r14,r9
   2450f:	call   24720 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECsdLEVLgM5ZIZ_5mibz2>
   24514:	mov    r9,r14
   24517:	mov    rbp,QWORD PTR [rsp+0x20]
   2451c:	xor    ecx,ecx
   2451e:	mov    eax,DWORD PTR [rbx+rcx*4+0x4]
   24522:	cmp    eax,DWORD PTR [rbx+rcx*4]
   24525:	je     24534 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1864>
   24527:	inc    rcx
   2452a:	cmp    r12,rcx
   2452d:	jne    2451e <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x184e>
   2452f:	jmp    22dd9 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x109>
   24534:	lea    rdx,[rcx+0x1]
   24538:	lea    rax,[rcx+0x2]
   2453c:	cmp    rax,r13
   2453f:	jae    24555 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1885>
   24541:	mov    esi,r9d
   24544:	sub    esi,ecx
   24546:	add    r9,0x1fd
   2454d:	test   sil,0x1
   24551:	jne    2455f <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x188f>
   24553:	jmp    24580 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18b0>
   24555:	mov    r13,rdx
   24558:	mov    rsi,QWORD PTR [rsp+0x50]
   2455d:	jmp    2458a <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ba>
   2455f:	mov    eax,DWORD PTR [rbx+rcx*4+0x8]
   24563:	mov    rsi,rdx
   24566:	cmp    eax,DWORD PTR [rbx+rcx*4]
   24569:	je     24573 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18a3>
   2456b:	mov    DWORD PTR [rbx+rcx*4+0x4],eax
   2456f:	lea    rsi,[rdx+0x1]
   24573:	add    rdx,0x2
   24577:	mov    rax,rdx
   2457a:	mov    r13,rsi
   2457d:	mov    rdx,rsi
   24580:	cmp    r9,rcx
   24583:	mov    rsi,QWORD PTR [rsp+0x50]
   24588:	jne    245a3 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d3>
   2458a:	mov    QWORD PTR [rbp+0x10],r13
   2458e:	test   r13,r13
   24591:	jne    22de7 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x117>
   24597:	lea    rdi,[rip+0x3829a]        # 5c838 <__frame_dummy_init_array_entry+0xac8>
   2459e:	call   1d0c0 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   245a3:	inc    r12
   245a6:	mov    r13,rdx
   245a9:	jmp    245b4 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18e4>
   245ab:	add    rax,0x2
   245af:	cmp    r12,rax
   245b2:	je     2458a <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ba>
   245b4:	mov    ecx,DWORD PTR [rbx+rax*4]
   245b7:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   245bc:	jne    245cb <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18fb>
   245be:	mov    ecx,DWORD PTR [rbx+rax*4+0x4]
   245c2:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   245c7:	je     245ab <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   245c9:	jmp    245dd <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x190d>
   245cb:	mov    DWORD PTR [rbx+r13*4],ecx
   245cf:	inc    r13
   245d2:	mov    ecx,DWORD PTR [rbx+rax*4+0x4]
   245d6:	cmp    ecx,DWORD PTR [rbx+r13*4-0x4]
   245db:	je     245ab <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   245dd:	mov    DWORD PTR [rbx+r13*4],ecx
   245e1:	inc    r13
   245e4:	jmp    245ab <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18db>
   245e6:	jmp    245ea <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191a>
   245e8:	jmp    245f4 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1924>
   245ea:	mov    r14,rax
   245ed:	mov    rbx,QWORD PTR [rsp+0x18]
   245f2:	jmp    24614 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1944>
   245f4:	mov    r14,rax
   245f7:	cmp    QWORD PTR [rsp+0x530],0x0
   24600:	je     24624 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1954>
   24602:	mov    rax,QWORD PTR [rsp+0x538]
   2460a:	mov    QWORD PTR [rsp+0x8],rax
   2460f:	jmp    24619 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1949>
   24611:	mov    r14,rax
   24614:	test   rbx,rbx
   24617:	je     24624 <_RINvNtCsdLEVLgM5ZIZ_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1954>
   24619:	mov    rdi,QWORD PTR [rsp+0x8]
   2461e:	call   QWORD PTR [rip+0x39aa4]        # 5e0c8 <free@GLIBC_2.2.5>
   24624:	mov    rdi,r14
   24627:	call   5ad50 <_Unwind_Resume@plt>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
