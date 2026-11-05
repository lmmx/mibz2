
c30cg/base/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

0000000000033e60 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   33e60:	push   rbp
   33e61:	push   r15
   33e63:	push   r14
   33e65:	push   r13
   33e67:	push   r12
   33e69:	push   rbx
   33e6a:	sub    rsp,0x648
   33e71:	mov    r15,rdi
   33e74:	mov    edi,edx
   33e76:	cmp    rsi,rdi
   33e79:	jbe    35c5e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dfe>
   33e7f:	mov    edi,DWORD PTR [r15+rdi*4]
   33e83:	mov    QWORD PTR [rcx+0x10],0x0
   33e8b:	movabs r12,0x2545f4914f6cdd1d
   33e95:	cmp    QWORD PTR [rcx],0x1fe
   33e9c:	jbe    359d6 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b76>
   33ea2:	xor    r13d,r13d
   33ea5:	shr    edi,0x8
   33ea8:	xor    r12,rsi
   33eab:	mov    QWORD PTR [rsp+0x30],rcx
   33eb0:	mov    r14,QWORD PTR [rcx+0x8]
   33eb4:	mov    ecx,0x1ff
   33eb9:	mov    rbx,r13
   33ebc:	nop    DWORD PTR [rax+0x0]
   33ec0:	mov    rbp,rbx
   33ec3:	mov    rax,r12
   33ec6:	shl    rax,0xd
   33eca:	xor    rax,r12
   33ecd:	mov    rdx,rax
   33ed0:	shr    rdx,0x7
   33ed4:	xor    rdx,rax
   33ed7:	mov    r12,rdx
   33eda:	shl    r12,0x11
   33ede:	xor    r12,rdx
   33ee1:	mov    rax,r12
   33ee4:	xor    edx,edx
   33ee6:	div    rsi
   33ee9:	mov    DWORD PTR [r14+rbx*4],edx
   33eed:	inc    rbx
   33ef0:	dec    rcx
   33ef3:	jne    33ec0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x60>
   33ef5:	mov    QWORD PTR [rsp+0x170],rsi
   33efd:	mov    r12,QWORD PTR [rsp+0x30]
   33f02:	mov    QWORD PTR [r12+0x10],rbx
   33f07:	cmp    rbx,QWORD PTR [r12]
   33f0b:	mov    DWORD PTR [rsp+0x4c],edi
   33f0f:	je     35682 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1822>
   33f15:	mov    DWORD PTR [r14+rbx*4],edi
   33f19:	lea    r9,[rbp+0x2]
   33f1d:	mov    QWORD PTR [r12+0x10],r9
   33f22:	cmp    rbp,0xfffffffffffffffd
   33f26:	jbe    35a09 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ba9>
   33f2c:	mov    r12,QWORD PTR [rsp+0x170]
   33f34:	mov    rbp,QWORD PTR [rsp+0x30]
   33f39:	test   r9,r9
   33f3c:	je     35c6a <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e0a>
   33f42:	lea    rax,[r9*4+0x0]
   33f4a:	xor    ecx,ecx
   33f4c:	nop    DWORD PTR [rax+0x0]
   33f50:	mov    edi,DWORD PTR [r14+rcx*1]
   33f54:	cmp    r12,rdi
   33f57:	jbe    35b75 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d15>
   33f5d:	or     BYTE PTR [r15+rdi*4+0x3],0x80
   33f63:	add    rcx,0x4
   33f67:	cmp    rax,rcx
   33f6a:	jne    33f50 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf0>
   33f6c:	lea    rax,[r12+r12*1]
   33f70:	mov    rcx,rax
   33f73:	or     rcx,r9
   33f76:	shr    rcx,0x20
   33f7a:	je     33f83 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x123>
   33f7c:	xor    edx,edx
   33f7e:	div    r9
   33f81:	jmp    33f88 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x128>
   33f83:	xor    edx,edx
   33f85:	div    r9d
   33f88:	mov    rbx,QWORD PTR [rbp+0x40]
   33f8c:	cmp    rbx,r9
   33f8f:	mov    r14,r9
   33f92:	cmova  r14,rbx
   33f96:	mov    QWORD PTR [rsp+0x58],r9
   33f9b:	jae    34ea4 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1044>
   33fa1:	sub    r14,rbx
   33fa4:	mov    rcx,QWORD PTR [rbp+0x30]
   33fa8:	sub    rcx,rbx
   33fab:	mov    rdx,rbx
   33fae:	cmp    r14,rcx
   33fb1:	ja     35aaf <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c4f>
   33fb7:	mov    rsi,QWORD PTR [rbp+0x38]
   33fbb:	inc    rbx
   33fbe:	test   r14b,0x1
   33fc2:	jne    35ae9 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c89>
   33fc8:	cmp    r9,rbx
   33fcb:	jne    35b16 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb6>
   33fd1:	mov    QWORD PTR [rbp+0x40],rdx
   33fd5:	cmp    r9,rdx
   33fd8:	ja     34eb4 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1054>
   33fde:	lea    rbx,[rax+0x40]
   33fe2:	mov    r14,QWORD PTR [rbp+0x38]
   33fe6:	lea    rcx,[r9*8+0x0]
   33fee:	lea    r12,[rcx+rcx*2]
   33ff2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34000:	mov    QWORD PTR [r14+0x10],0x0
   34008:	cmp    rbx,QWORD PTR [r14]
   3400b:	ja     34019 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b9>
   3400d:	add    r14,0x18
   34011:	add    r12,0xffffffffffffffe8
   34015:	jne    34000 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a0>
   34017:	jmp    34046 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e6>
   34019:	mov    ecx,0x1
   3401e:	mov    r8d,0x1
   34024:	mov    rdi,r14
   34027:	xor    esi,esi
   34029:	mov    rdx,rbx
   3402c:	mov    r13,rax
   3402f:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   34034:	mov    rax,r13
   34037:	mov    r9,QWORD PTR [rsp+0x58]
   3403c:	add    r14,0x18
   34040:	add    r12,0xffffffffffffffe8
   34044:	jne    34000 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a0>
   34046:	mov    QWORD PTR [rbp+0x58],0x0
   3404e:	cmp    r9,QWORD PTR [rbp+0x48]
   34052:	ja     35a70 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c10>
   34058:	xor    ebx,ebx
   3405a:	mov    r14,QWORD PTR [rbp+0x50]
   3405e:	lea    rdi,[r14+rbx*8]
   34062:	cmp    r9,0x1
   34066:	je     34098 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x238>
   34068:	lea    rdx,[r9*8-0x8]
   34070:	mov    esi,0xff
   34075:	mov    r12,rax
   34078:	call   QWORD PTR [rip+0x61c42]        # 95cc0 <memset@GLIBC_2.2.5>
   3407e:	mov    rax,r12
   34081:	mov    r9,QWORD PTR [rsp+0x58]
   34086:	lea    rcx,[rbx+r9*1]
   3408a:	lea    rdi,[r14+rcx*8]
   3408e:	add    rdi,0xfffffffffffffff8
   34092:	add    rbx,r9
   34095:	dec    rbx
   34098:	mov    QWORD PTR [rdi],0xffffffffffffffff
   3409f:	inc    rbx
   340a2:	mov    QWORD PTR [rbp+0x58],rbx
   340a6:	xorps  xmm0,xmm0
   340a9:	movups XMMWORD PTR [rsp+0x238],xmm0
   340b1:	mov    DWORD PTR [rsp+0x248],0x0
   340bc:	movups XMMWORD PTR [rsp+0x258],xmm0
   340c4:	mov    DWORD PTR [rsp+0x268],0x0
   340cf:	movups XMMWORD PTR [rsp+0x278],xmm0
   340d7:	mov    DWORD PTR [rsp+0x288],0x0
   340e2:	movups XMMWORD PTR [rsp+0x298],xmm0
   340ea:	mov    DWORD PTR [rsp+0x2a8],0x0
   340f5:	movups XMMWORD PTR [rsp+0x2b8],xmm0
   340fd:	mov    DWORD PTR [rsp+0x2c8],0x0
   34108:	movups XMMWORD PTR [rsp+0x2d8],xmm0
   34110:	mov    DWORD PTR [rsp+0x2e8],0x0
   3411b:	movups XMMWORD PTR [rsp+0x2f8],xmm0
   34123:	mov    DWORD PTR [rsp+0x308],0x0
   3412e:	movups XMMWORD PTR [rsp+0x318],xmm0
   34136:	mov    DWORD PTR [rsp+0x328],0x0
   34141:	movups XMMWORD PTR [rsp+0x338],xmm0
   34149:	mov    DWORD PTR [rsp+0x348],0x0
   34154:	movups XMMWORD PTR [rsp+0x358],xmm0
   3415c:	mov    DWORD PTR [rsp+0x368],0x0
   34167:	movups XMMWORD PTR [rsp+0x378],xmm0
   3416f:	mov    DWORD PTR [rsp+0x388],0x0
   3417a:	movups XMMWORD PTR [rsp+0x398],xmm0
   34182:	mov    DWORD PTR [rsp+0x3a8],0x0
   3418d:	movups XMMWORD PTR [rsp+0x3b8],xmm0
   34195:	mov    DWORD PTR [rsp+0x3c8],0x0
   341a0:	movups XMMWORD PTR [rsp+0x3d8],xmm0
   341a8:	mov    DWORD PTR [rsp+0x3e8],0x0
   341b3:	movups XMMWORD PTR [rsp+0x3f8],xmm0
   341bb:	mov    DWORD PTR [rsp+0x408],0x0
   341c6:	movups XMMWORD PTR [rsp+0x418],xmm0
   341ce:	mov    DWORD PTR [rsp+0x428],0x0
   341d9:	movups XMMWORD PTR [rsp+0x438],xmm0
   341e1:	mov    DWORD PTR [rsp+0x448],0x0
   341ec:	movups XMMWORD PTR [rsp+0x458],xmm0
   341f4:	mov    DWORD PTR [rsp+0x468],0x0
   341ff:	movups XMMWORD PTR [rsp+0x478],xmm0
   34207:	mov    DWORD PTR [rsp+0x488],0x0
   34212:	movups XMMWORD PTR [rsp+0x498],xmm0
   3421a:	mov    DWORD PTR [rsp+0x4a8],0x0
   34225:	movups XMMWORD PTR [rsp+0x4b8],xmm0
   3422d:	mov    DWORD PTR [rsp+0x4c8],0x0
   34238:	movups XMMWORD PTR [rsp+0x4d8],xmm0
   34240:	mov    DWORD PTR [rsp+0x4e8],0x0
   3424b:	movups XMMWORD PTR [rsp+0x4f8],xmm0
   34253:	mov    DWORD PTR [rsp+0x508],0x0
   3425e:	movups XMMWORD PTR [rsp+0x518],xmm0
   34266:	mov    DWORD PTR [rsp+0x528],0x0
   34271:	movups XMMWORD PTR [rsp+0x538],xmm0
   34279:	mov    DWORD PTR [rsp+0x548],0x0
   34284:	movups XMMWORD PTR [rsp+0x558],xmm0
   3428c:	mov    DWORD PTR [rsp+0x568],0x0
   34297:	movups XMMWORD PTR [rsp+0x578],xmm0
   3429f:	mov    DWORD PTR [rsp+0x588],0x0
   342aa:	movups XMMWORD PTR [rsp+0x598],xmm0
   342b2:	mov    DWORD PTR [rsp+0x5a8],0x0
   342bd:	movups XMMWORD PTR [rsp+0x5b8],xmm0
   342c5:	mov    DWORD PTR [rsp+0x5c8],0x0
   342d0:	movups XMMWORD PTR [rsp+0x5d8],xmm0
   342d8:	mov    DWORD PTR [rsp+0x5e8],0x0
   342e3:	movups XMMWORD PTR [rsp+0x5f8],xmm0
   342eb:	mov    DWORD PTR [rsp+0x608],0x0
   342f6:	movups XMMWORD PTR [rsp+0x618],xmm0
   342fe:	mov    DWORD PTR [rsp+0x628],0x0
   34309:	mov    QWORD PTR [rsp+0x230],0xffffffffffffffff
   34315:	mov    QWORD PTR [rsp+0x250],0xffffffffffffffff
   34321:	mov    QWORD PTR [rsp+0x270],0xffffffffffffffff
   3432d:	mov    QWORD PTR [rsp+0x290],0xffffffffffffffff
   34339:	mov    QWORD PTR [rsp+0x2b0],0xffffffffffffffff
   34345:	mov    QWORD PTR [rsp+0x2d0],0xffffffffffffffff
   34351:	mov    QWORD PTR [rsp+0x2f0],0xffffffffffffffff
   3435d:	mov    QWORD PTR [rsp+0x310],0xffffffffffffffff
   34369:	mov    QWORD PTR [rsp+0x330],0xffffffffffffffff
   34375:	mov    QWORD PTR [rsp+0x350],0xffffffffffffffff
   34381:	mov    QWORD PTR [rsp+0x370],0xffffffffffffffff
   3438d:	mov    QWORD PTR [rsp+0x390],0xffffffffffffffff
   34399:	mov    QWORD PTR [rsp+0x3b0],0xffffffffffffffff
   343a5:	mov    QWORD PTR [rsp+0x3d0],0xffffffffffffffff
   343b1:	mov    QWORD PTR [rsp+0x3f0],0xffffffffffffffff
   343bd:	mov    QWORD PTR [rsp+0x410],0xffffffffffffffff
   343c9:	mov    QWORD PTR [rsp+0x430],0xffffffffffffffff
   343d5:	mov    QWORD PTR [rsp+0x450],0xffffffffffffffff
   343e1:	mov    QWORD PTR [rsp+0x470],0xffffffffffffffff
   343ed:	mov    QWORD PTR [rsp+0x490],0xffffffffffffffff
   343f9:	mov    QWORD PTR [rsp+0x4b0],0xffffffffffffffff
   34405:	mov    QWORD PTR [rsp+0x4d0],0xffffffffffffffff
   34411:	mov    QWORD PTR [rsp+0x4f0],0xffffffffffffffff
   3441d:	mov    QWORD PTR [rsp+0x510],0xffffffffffffffff
   34429:	mov    QWORD PTR [rsp+0x530],0xffffffffffffffff
   34435:	mov    QWORD PTR [rsp+0x550],0xffffffffffffffff
   34441:	mov    QWORD PTR [rsp+0x570],0xffffffffffffffff
   3444d:	mov    QWORD PTR [rsp+0x590],0xffffffffffffffff
   34459:	mov    QWORD PTR [rsp+0x5b0],0xffffffffffffffff
   34465:	mov    QWORD PTR [rsp+0x5d0],0xffffffffffffffff
   34471:	mov    QWORD PTR [rsp+0x5f0],0xffffffffffffffff
   3447d:	mov    QWORD PTR [rsp+0x610],0xffffffffffffffff
   34489:	mov    rcx,QWORD PTR [rbp+0x38]
   3448d:	mov    rsi,QWORD PTR [rbp+0x40]
   34491:	mov    r8,QWORD PTR [rbp+0x8]
   34495:	mov    r14,QWORD PTR [rbp+0x10]
   34499:	mov    QWORD PTR [rsp+0x28],rcx
   3449e:	lea    r10,[rcx+0x8]
   344a2:	xor    edi,edi
   344a4:	xor    r13d,r13d
   344a7:	nop    WORD PTR [rax+rax*1+0x0]
   344b0:	cmp    rsi,rdi
   344b3:	je     35b84 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d24>
   344b9:	cmp    r14,rdi
   344bc:	je     35b90 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d30>
   344c2:	mov    rcx,QWORD PTR [r10]
   344c5:	mov    edx,DWORD PTR [r8+rdi*4]
   344c9:	mov    edx,DWORD PTR [r15+rdx*4]
   344cd:	mov    BYTE PTR [rcx],dl
   344cf:	shr    edx,0x8
   344d2:	and    edx,0x7fffff
   344d8:	lea    r11,[rcx+0x1]
   344dc:	add    rcx,QWORD PTR [r10-0x8]
   344e0:	mov    QWORD PTR [rsp+r13*1+0x230],rdi
   344e8:	mov    QWORD PTR [rsp+r13*1+0x238],r11
   344f0:	mov    QWORD PTR [rsp+r13*1+0x240],rcx
   344f8:	mov    DWORD PTR [rsp+r13*1+0x248],edx
   34500:	prefetcht0 BYTE PTR [r15+rdx*4]
   34505:	lea    rdx,[rdi+0x1]
   34509:	add    r13,0x20
   3450d:	cmp    rdi,0x1e
   34511:	ja     3451f <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6bf>
   34513:	add    r10,0x18
   34517:	mov    rdi,rdx
   3451a:	cmp    rdx,r9
   3451d:	jb     344b0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x650>
   3451f:	cmp    rdx,0x20
   34523:	mov    QWORD PTR [rsp+0x20],r8
   34528:	mov    QWORD PTR [rsp+0x68],r14
   3452d:	jne    34ec5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1065>
   34533:	mov    QWORD PTR [rsp+0x220],rdx
   3453b:	add    rax,0x3f
   3453f:	mov    ecx,0x20
   34544:	mov    QWORD PTR [rsp+0x60],rcx
   34549:	mov    QWORD PTR [rsp+0x8],rsi
   3454e:	mov    QWORD PTR [rsp+0x228],rax
   34556:	xor    r12d,r12d
   34559:	mov    rbp,rax
   3455c:	jmp    34591 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x731>
   3455e:	xchg   ax,ax
   34560:	add    rbx,rdi
   34563:	mov    QWORD PTR [rsp+r12*1+0x238],rbx
   3456b:	add    rdx,rdi
   3456e:	mov    QWORD PTR [rsp+r12*1+0x240],rdx
   34576:	sub    rdx,rbx
   34579:	add    r12,0x20
   3457d:	cmp    rdx,rbp
   34580:	cmovb  rbp,rdx
   34584:	cmp    r12,0x400
   3458b:	je     34620 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x7c0>
   34591:	mov    rbx,QWORD PTR [rsp+r12*1+0x238]
   34599:	mov    rdx,QWORD PTR [rsp+r12*1+0x240]
   345a1:	cmp    rbx,rdx
   345a4:	jne    34576 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x716>
   345a6:	mov    rdi,QWORD PTR [rsp+r12*1+0x230]
   345ae:	cmp    rdi,rsi
   345b1:	jae    35be5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d85>
   345b7:	lea    rcx,[rdi+rdi*2]
   345bb:	mov    r8,QWORD PTR [rsp+0x28]
   345c0:	mov    rdx,QWORD PTR [r8+rcx*8]
   345c4:	mov    rdi,QWORD PTR [r8+rcx*8+0x8]
   345c9:	sub    rbx,rdi
   345cc:	mov    QWORD PTR [r8+rcx*8+0x10],rbx
   345d1:	mov    r8,rdx
   345d4:	sub    r8,rbx
   345d7:	cmp    rdx,r8
   345da:	jbe    34560 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x700>
   345dc:	mov    rsi,QWORD PTR [rsp+0x28]
   345e1:	lea    rdi,[rsi+rcx*8]
   345e5:	mov    QWORD PTR [rsp+0x40],rdi
   345ea:	mov    ecx,0x1
   345ef:	mov    r8d,0x1
   345f5:	mov    rsi,rbx
   345f8:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   345fd:	mov    rsi,QWORD PTR [rsp+0x8]
   34602:	mov    rcx,QWORD PTR [rsp+0x40]
   34607:	mov    rdx,QWORD PTR [rcx]
   3460a:	mov    rdi,QWORD PTR [rcx+0x8]
   3460e:	jmp    34560 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x700>
   34613:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34620:	mov    rax,QWORD PTR [rsp+0x238]
   34628:	mov    rcx,QWORD PTR [rsp+0x258]
   34630:	mov    rdx,QWORD PTR [rsp+0x278]
   34638:	mov    QWORD PTR [rsp+0x50],rdx
   3463d:	mov    rdx,QWORD PTR [rsp+0x298]
   34645:	mov    QWORD PTR [rsp+0x10],rdx
   3464a:	mov    r12,QWORD PTR [rsp+0x2b8]
   34652:	mov    rbx,QWORD PTR [rsp+0x2d8]
   3465a:	mov    r11,QWORD PTR [rsp+0x2f8]
   34662:	mov    r10,QWORD PTR [rsp+0x318]
   3466a:	mov    r9,QWORD PTR [rsp+0x338]
   34672:	mov    r8,QWORD PTR [rsp+0x358]
   3467a:	mov    rdi,QWORD PTR [rsp+0x378]
   34682:	mov    rsi,QWORD PTR [rsp+0x398]
   3468a:	mov    rdx,QWORD PTR [rsp+0x3b8]
   34692:	mov    QWORD PTR [rsp+0x38],rdx
   34697:	mov    rdx,QWORD PTR [rsp+0x3d8]
   3469f:	mov    QWORD PTR [rsp+0x18],rdx
   346a4:	mov    rdx,QWORD PTR [rsp+0x3f8]
   346ac:	mov    QWORD PTR [rsp+0x40],rdx
   346b1:	mov    QWORD PTR [rsp+0x1d0],rax
   346b9:	mov    QWORD PTR [rsp+0x70],rax
   346be:	mov    QWORD PTR [rsp+0x1d8],rcx
   346c6:	mov    QWORD PTR [rsp+0x78],rcx
   346cb:	mov    rax,QWORD PTR [rsp+0x50]
   346d0:	mov    QWORD PTR [rsp+0x80],rax
   346d8:	mov    rax,QWORD PTR [rsp+0x10]
   346dd:	mov    QWORD PTR [rsp+0x88],rax
   346e5:	mov    QWORD PTR [rsp+0x1e0],r12
   346ed:	mov    QWORD PTR [rsp+0x90],r12
   346f5:	mov    QWORD PTR [rsp+0x1e8],rbx
   346fd:	mov    QWORD PTR [rsp+0x98],rbx
   34705:	mov    QWORD PTR [rsp+0x1f0],r11
   3470d:	mov    QWORD PTR [rsp+0xa0],r11
   34715:	mov    QWORD PTR [rsp+0x1f8],r10
   3471d:	mov    QWORD PTR [rsp+0xa8],r10
   34725:	mov    QWORD PTR [rsp+0x200],r9
   3472d:	mov    QWORD PTR [rsp+0xb0],r9
   34735:	mov    QWORD PTR [rsp+0x208],r8
   3473d:	mov    QWORD PTR [rsp+0xb8],r8
   34745:	mov    QWORD PTR [rsp+0x210],rdi
   3474d:	mov    QWORD PTR [rsp+0xc0],rdi
   34755:	mov    QWORD PTR [rsp+0x218],rsi
   3475d:	mov    QWORD PTR [rsp+0xc8],rsi
   34765:	mov    rax,QWORD PTR [rsp+0x38]
   3476a:	mov    QWORD PTR [rsp+0xd0],rax
   34772:	mov    rax,QWORD PTR [rsp+0x18]
   34777:	mov    QWORD PTR [rsp+0xd8],rax
   3477f:	mov    rax,QWORD PTR [rsp+0x40]
   34784:	mov    QWORD PTR [rsp+0xe0],rax
   3478c:	mov    rax,QWORD PTR [rsp+0x418]
   34794:	mov    QWORD PTR [rsp+0x1a0],rax
   3479c:	mov    QWORD PTR [rsp+0xe8],rax
   347a4:	mov    rax,QWORD PTR [rsp+0x438]
   347ac:	mov    QWORD PTR [rsp+0x1a8],rax
   347b4:	mov    QWORD PTR [rsp+0xf0],rax
   347bc:	mov    rax,QWORD PTR [rsp+0x458]
   347c4:	mov    QWORD PTR [rsp+0x1b0],rax
   347cc:	mov    QWORD PTR [rsp+0xf8],rax
   347d4:	mov    rax,QWORD PTR [rsp+0x478]
   347dc:	mov    QWORD PTR [rsp+0x1b8],rax
   347e4:	mov    QWORD PTR [rsp+0x100],rax
   347ec:	mov    rax,QWORD PTR [rsp+0x498]
   347f4:	mov    QWORD PTR [rsp+0x1c0],rax
   347fc:	mov    QWORD PTR [rsp+0x108],rax
   34804:	mov    rax,QWORD PTR [rsp+0x4b8]
   3480c:	mov    QWORD PTR [rsp+0x1c8],rax
   34814:	mov    QWORD PTR [rsp+0x110],rax
   3481c:	mov    r12,QWORD PTR [rsp+0x4d8]
   34824:	mov    QWORD PTR [rsp+0x118],r12
   3482c:	mov    rdi,QWORD PTR [rsp+0x4f8]
   34834:	mov    QWORD PTR [rsp+0x120],rdi
   3483c:	mov    rsi,QWORD PTR [rsp+0x518]
   34844:	mov    QWORD PTR [rsp+0x128],rsi
   3484c:	mov    rdx,QWORD PTR [rsp+0x538]
   34854:	mov    QWORD PTR [rsp+0x130],rdx
   3485c:	mov    rcx,QWORD PTR [rsp+0x558]
   34864:	mov    QWORD PTR [rsp+0x138],rcx
   3486c:	mov    rax,QWORD PTR [rsp+0x578]
   34874:	mov    QWORD PTR [rsp+0x140],rax
   3487c:	mov    rbx,QWORD PTR [rsp+0x598]
   34884:	mov    QWORD PTR [rsp+0x148],rbx
   3488c:	mov    r11,QWORD PTR [rsp+0x5b8]
   34894:	mov    QWORD PTR [rsp+0x150],r11
   3489c:	mov    r10,QWORD PTR [rsp+0x5d8]
   348a4:	mov    QWORD PTR [rsp+0x158],r10
   348ac:	mov    r9,QWORD PTR [rsp+0x5f8]
   348b4:	mov    QWORD PTR [rsp+0x160],r9
   348bc:	mov    r8,QWORD PTR [rsp+0x618]
   348c4:	mov    QWORD PTR [rsp+0x168],r8
   348cc:	test   rbp,rbp
   348cf:	je     34ad0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc70>
   348d5:	mov    rax,QWORD PTR [rsp+0x30]
   348da:	mov    rcx,QWORD PTR [rax+0x50]
   348de:	mov    QWORD PTR [rsp+0x50],rcx
   348e3:	mov    rax,QWORD PTR [rax+0x58]
   348e7:	mov    QWORD PTR [rsp+0x10],rax
   348ec:	xor    r9d,r9d
   348ef:	jmp    34909 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xaa9>
   348f1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34900:	cmp    r9,rbp
   34903:	je     34af0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc90>
   34909:	mov    rcx,r9
   3490c:	inc    r9
   3490f:	mov    rax,rcx
   34912:	not    rax
   34915:	mov    QWORD PTR [rsp+0x40],rax
   3491a:	xor    r10d,r10d
   3491d:	mov    r11,r10
   34920:	shl    r11,0x5
   34924:	lea    rax,[rsp+0x248]
   3492c:	add    r11,rax
   3492f:	mov    r8,r10
   34932:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34940:	lea    r10,[r8+0x1]
   34944:	mov    ebx,DWORD PTR [r11]
   34947:	mov    eax,DWORD PTR [r15+rbx*4]
   3494b:	test   eax,eax
   3494d:	js     34980 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb20>
   3494f:	mov    edi,eax
   34951:	shr    edi,0x8
   34954:	mov    DWORD PTR [r11],edi
   34957:	prefetcht0 BYTE PTR [r15+rdi*4]
   3495c:	mov    rdi,QWORD PTR [rsp+r8*8+0x70]
   34961:	mov    BYTE PTR [rdi+rcx*1],al
   34964:	lea    rax,[r10-0x1]
   34968:	add    r11,0x20
   3496c:	mov    r8,r10
   3496f:	cmp    rax,0x1f
   34973:	jb     34940 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xae0>
   34975:	jmp    34900 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xaa0>
   34977:	nop    WORD PTR [rax+rax*1+0x0]
   34980:	mov    rax,QWORD PTR [rsp+r8*8+0x70]
   34985:	add    rax,rcx
   34988:	mov    QWORD PTR [r11-0x10],rax
   3498c:	mov    rdi,QWORD PTR [r11-0x18]
   34990:	cmp    rdi,QWORD PTR [rsp+0x8]
   34995:	jae    35bf1 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d91>
   3499b:	lea    r12,[rdi+rdi*2]
   3499f:	mov    rdx,QWORD PTR [rsp+0x28]
   349a4:	sub    rax,QWORD PTR [rdx+r12*8+0x8]
   349a9:	mov    QWORD PTR [rdx+r12*8+0x10],rax
   349ae:	test   r14,r14
   349b1:	je     349c3 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb63>
   349b3:	cmp    r14,0x1
   349b7:	jne    349cc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb6c>
   349b9:	xor    r12d,r12d
   349bc:	mov    r14,QWORD PTR [rsp+0x20]
   349c1:	jmp    349fd <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb9d>
   349c3:	mov    r12,0xffffffffffffffff
   349ca:	jmp    34a0c <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbac>
   349cc:	mov    rax,r14
   349cf:	xor    r12d,r12d
   349d2:	mov    r14,QWORD PTR [rsp+0x20]
   349d7:	nop    WORD PTR [rax+rax*1+0x0]
   349e0:	mov    rdx,r12
   349e3:	mov    rsi,rax
   349e6:	shr    rsi,1
   349e9:	add    r12,rsi
   349ec:	cmp    DWORD PTR [r14+r12*4],ebx
   349f0:	cmova  r12,rdx
   349f4:	sub    rax,rsi
   349f7:	cmp    rax,0x1
   349fb:	ja     349e0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb80>
   349fd:	cmp    DWORD PTR [r14+r12*4],ebx
   34a01:	mov    rax,0xffffffffffffffff
   34a08:	cmovne r12,rax
   34a0c:	cmp    rdi,QWORD PTR [rsp+0x10]
   34a11:	mov    rbx,QWORD PTR [rsp+0x60]
   34a16:	jae    35c02 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1da2>
   34a1c:	mov    rax,QWORD PTR [rsp+0x50]
   34a21:	mov    QWORD PTR [rax+rdi*8],r12
   34a25:	cmp    rbx,QWORD PTR [rsp+0x58]
   34a2a:	mov    rax,QWORD PTR [rsp+0x68]
   34a2f:	je     34edc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x107c>
   34a35:	cmp    rbx,QWORD PTR [rsp+0x8]
   34a3a:	jae    35c25 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dc5>
   34a40:	mov    r14,rax
   34a43:	cmp    rbx,rax
   34a46:	jae    35c13 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1db3>
   34a4c:	lea    rax,[rbx+rbx*2]
   34a50:	mov    QWORD PTR [rsp+0x18],rax
   34a55:	mov    r12,QWORD PTR [rsp+0x28]
   34a5a:	mov    rdx,QWORD PTR [r12+rax*8+0x8]
   34a5f:	mov    rsi,QWORD PTR [rsp+0x20]
   34a64:	mov    esi,DWORD PTR [rsi+rbx*4]
   34a67:	mov    esi,DWORD PTR [r15+rsi*4]
   34a6b:	mov    BYTE PTR [rdx],sil
   34a6e:	shr    esi,0x8
   34a71:	and    esi,0x7fffff
   34a77:	lea    rdi,[rdx+0x1]
   34a7b:	mov    rax,QWORD PTR [rsp+0x40]
   34a80:	add    rax,rdx
   34a83:	inc    rax
   34a86:	mov    QWORD PTR [rsp+0x38],rax
   34a8b:	mov    rax,QWORD PTR [rsp+0x18]
   34a90:	add    rdx,QWORD PTR [r12+rax*8]
   34a94:	mov    QWORD PTR [r11-0x18],rbx
   34a98:	mov    QWORD PTR [r11-0x10],rdi
   34a9c:	mov    QWORD PTR [r11-0x8],rdx
   34aa0:	mov    DWORD PTR [r11],esi
   34aa3:	prefetcht0 BYTE PTR [r15+rsi*4]
   34aa8:	inc    rbx
   34aab:	mov    QWORD PTR [rsp+0x60],rbx
   34ab0:	mov    rax,QWORD PTR [rsp+0x38]
   34ab5:	mov    QWORD PTR [rsp+r8*8+0x70],rax
   34aba:	cmp    r8,0x1f
   34abe:	jb     3491d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xabd>
   34ac4:	jmp    34900 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xaa0>
   34ac9:	nop    DWORD PTR [rax+0x0]
   34ad0:	mov    QWORD PTR [rsp+0x190],r12
   34ad8:	mov    r12,QWORD PTR [rsp+0x50]
   34add:	mov    QWORD PTR [rsp+0x198],rax
   34ae5:	jmp    34c8e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe2e>
   34aea:	nop    WORD PTR [rax+rax*1+0x0]
   34af0:	mov    rax,QWORD PTR [rsp+0x70]
   34af5:	mov    QWORD PTR [rsp+0x1d0],rax
   34afd:	mov    rax,QWORD PTR [rsp+0x78]
   34b02:	mov    QWORD PTR [rsp+0x1d8],rax
   34b0a:	mov    r12,QWORD PTR [rsp+0x80]
   34b12:	mov    rax,QWORD PTR [rsp+0x88]
   34b1a:	mov    QWORD PTR [rsp+0x10],rax
   34b1f:	mov    rax,QWORD PTR [rsp+0x90]
   34b27:	mov    QWORD PTR [rsp+0x1e0],rax
   34b2f:	mov    rax,QWORD PTR [rsp+0x98]
   34b37:	mov    QWORD PTR [rsp+0x1e8],rax
   34b3f:	mov    rax,QWORD PTR [rsp+0xa0]
   34b47:	mov    QWORD PTR [rsp+0x1f0],rax
   34b4f:	mov    rax,QWORD PTR [rsp+0xa8]
   34b57:	mov    QWORD PTR [rsp+0x1f8],rax
   34b5f:	mov    rax,QWORD PTR [rsp+0xb0]
   34b67:	mov    QWORD PTR [rsp+0x200],rax
   34b6f:	mov    rax,QWORD PTR [rsp+0xb8]
   34b77:	mov    QWORD PTR [rsp+0x208],rax
   34b7f:	mov    rax,QWORD PTR [rsp+0xc0]
   34b87:	mov    QWORD PTR [rsp+0x210],rax
   34b8f:	mov    rax,QWORD PTR [rsp+0xc8]
   34b97:	mov    QWORD PTR [rsp+0x218],rax
   34b9f:	mov    rax,QWORD PTR [rsp+0xd0]
   34ba7:	mov    QWORD PTR [rsp+0x38],rax
   34bac:	mov    rax,QWORD PTR [rsp+0xd8]
   34bb4:	mov    QWORD PTR [rsp+0x18],rax
   34bb9:	mov    rax,QWORD PTR [rsp+0xe0]
   34bc1:	mov    QWORD PTR [rsp+0x40],rax
   34bc6:	mov    rax,QWORD PTR [rsp+0xe8]
   34bce:	mov    QWORD PTR [rsp+0x1a0],rax
   34bd6:	mov    rax,QWORD PTR [rsp+0xf0]
   34bde:	mov    QWORD PTR [rsp+0x1a8],rax
   34be6:	mov    rax,QWORD PTR [rsp+0xf8]
   34bee:	mov    QWORD PTR [rsp+0x1b0],rax
   34bf6:	mov    rax,QWORD PTR [rsp+0x100]
   34bfe:	mov    QWORD PTR [rsp+0x1b8],rax
   34c06:	mov    rax,QWORD PTR [rsp+0x108]
   34c0e:	mov    QWORD PTR [rsp+0x1c0],rax
   34c16:	mov    rax,QWORD PTR [rsp+0x110]
   34c1e:	mov    QWORD PTR [rsp+0x1c8],rax
   34c26:	mov    rax,QWORD PTR [rsp+0x118]
   34c2e:	mov    QWORD PTR [rsp+0x190],rax
   34c36:	mov    rdi,QWORD PTR [rsp+0x120]
   34c3e:	mov    rsi,QWORD PTR [rsp+0x128]
   34c46:	mov    rdx,QWORD PTR [rsp+0x130]
   34c4e:	mov    rcx,QWORD PTR [rsp+0x138]
   34c56:	mov    rax,QWORD PTR [rsp+0x140]
   34c5e:	mov    QWORD PTR [rsp+0x198],rax
   34c66:	mov    rbx,QWORD PTR [rsp+0x148]
   34c6e:	mov    r11,QWORD PTR [rsp+0x150]
   34c76:	mov    r10,QWORD PTR [rsp+0x158]
   34c7e:	mov    r9,QWORD PTR [rsp+0x160]
   34c86:	mov    r8,QWORD PTR [rsp+0x168]
   34c8e:	mov    rax,QWORD PTR [rsp+0x1d0]
   34c96:	add    rax,rbp
   34c99:	mov    QWORD PTR [rsp+0x238],rax
   34ca1:	mov    rax,QWORD PTR [rsp+0x1d8]
   34ca9:	add    rax,rbp
   34cac:	mov    QWORD PTR [rsp+0x258],rax
   34cb4:	add    r12,rbp
   34cb7:	mov    QWORD PTR [rsp+0x278],r12
   34cbf:	mov    r12,QWORD PTR [rsp+0x10]
   34cc4:	add    r12,rbp
   34cc7:	mov    QWORD PTR [rsp+0x298],r12
   34ccf:	mov    r12,QWORD PTR [rsp+0x1e0]
   34cd7:	add    r12,rbp
   34cda:	mov    QWORD PTR [rsp+0x2b8],r12
   34ce2:	mov    r12,QWORD PTR [rsp+0x1e8]
   34cea:	add    r12,rbp
   34ced:	mov    QWORD PTR [rsp+0x2d8],r12
   34cf5:	mov    r12,QWORD PTR [rsp+0x1f0]
   34cfd:	add    r12,rbp
   34d00:	mov    QWORD PTR [rsp+0x2f8],r12
   34d08:	mov    r12,QWORD PTR [rsp+0x1f8]
   34d10:	add    r12,rbp
   34d13:	mov    QWORD PTR [rsp+0x318],r12
   34d1b:	mov    r12,QWORD PTR [rsp+0x200]
   34d23:	add    r12,rbp
   34d26:	mov    QWORD PTR [rsp+0x338],r12
   34d2e:	mov    r12,QWORD PTR [rsp+0x208]
   34d36:	add    r12,rbp
   34d39:	mov    QWORD PTR [rsp+0x358],r12
   34d41:	mov    r12,QWORD PTR [rsp+0x210]
   34d49:	add    r12,rbp
   34d4c:	mov    QWORD PTR [rsp+0x378],r12
   34d54:	mov    r12,QWORD PTR [rsp+0x218]
   34d5c:	add    r12,rbp
   34d5f:	mov    QWORD PTR [rsp+0x398],r12
   34d67:	mov    r12,QWORD PTR [rsp+0x38]
   34d6c:	add    r12,rbp
   34d6f:	mov    QWORD PTR [rsp+0x3b8],r12
   34d77:	mov    r12,QWORD PTR [rsp+0x18]
   34d7c:	add    r12,rbp
   34d7f:	mov    QWORD PTR [rsp+0x3d8],r12
   34d87:	mov    r12,QWORD PTR [rsp+0x40]
   34d8c:	add    r12,rbp
   34d8f:	mov    QWORD PTR [rsp+0x3f8],r12
   34d97:	mov    rax,QWORD PTR [rsp+0x1a0]
   34d9f:	add    rax,rbp
   34da2:	mov    QWORD PTR [rsp+0x418],rax
   34daa:	mov    rax,QWORD PTR [rsp+0x1a8]
   34db2:	add    rax,rbp
   34db5:	mov    QWORD PTR [rsp+0x438],rax
   34dbd:	mov    rax,QWORD PTR [rsp+0x1b0]
   34dc5:	add    rax,rbp
   34dc8:	mov    QWORD PTR [rsp+0x458],rax
   34dd0:	mov    rax,QWORD PTR [rsp+0x1b8]
   34dd8:	add    rax,rbp
   34ddb:	mov    QWORD PTR [rsp+0x478],rax
   34de3:	mov    rax,QWORD PTR [rsp+0x1c0]
   34deb:	add    rax,rbp
   34dee:	mov    QWORD PTR [rsp+0x498],rax
   34df6:	mov    rax,QWORD PTR [rsp+0x1c8]
   34dfe:	add    rax,rbp
   34e01:	mov    QWORD PTR [rsp+0x4b8],rax
   34e09:	mov    rax,QWORD PTR [rsp+0x190]
   34e11:	add    rax,rbp
   34e14:	mov    QWORD PTR [rsp+0x4d8],rax
   34e1c:	add    rdi,rbp
   34e1f:	mov    QWORD PTR [rsp+0x4f8],rdi
   34e27:	add    rsi,rbp
   34e2a:	mov    QWORD PTR [rsp+0x518],rsi
   34e32:	add    rdx,rbp
   34e35:	mov    QWORD PTR [rsp+0x538],rdx
   34e3d:	add    rcx,rbp
   34e40:	mov    QWORD PTR [rsp+0x558],rcx
   34e48:	mov    rax,QWORD PTR [rsp+0x198]
   34e50:	add    rax,rbp
   34e53:	mov    QWORD PTR [rsp+0x578],rax
   34e5b:	add    rbx,rbp
   34e5e:	mov    QWORD PTR [rsp+0x598],rbx
   34e66:	add    r11,rbp
   34e69:	mov    QWORD PTR [rsp+0x5b8],r11
   34e71:	add    r10,rbp
   34e74:	mov    QWORD PTR [rsp+0x5d8],r10
   34e7c:	add    r9,rbp
   34e7f:	mov    QWORD PTR [rsp+0x5f8],r9
   34e87:	add    r8,rbp
   34e8a:	mov    QWORD PTR [rsp+0x618],r8
   34e92:	mov    rax,QWORD PTR [rsp+0x228]
   34e9a:	mov    rsi,QWORD PTR [rsp+0x8]
   34e9f:	jmp    34556 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6f6>
   34ea4:	mov    rdx,r14
   34ea7:	mov    QWORD PTR [rbp+0x40],rdx
   34eab:	cmp    r9,rdx
   34eae:	jbe    33fde <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e>
   34eb4:	lea    rcx,[rip+0x5e995]        # 93850 <__frame_dummy_init_array_entry+0x908>
   34ebb:	xor    edi,edi
   34ebd:	mov    rsi,r9
   34ec0:	call   2b1c0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   34ec5:	mov    rax,QWORD PTR [rbp+0x50]
   34ec9:	mov    QWORD PTR [rsp+0x50],rax
   34ece:	mov    rax,QWORD PTR [rbp+0x58]
   34ed2:	mov    QWORD PTR [rsp+0x10],rax
   34ed7:	jmp    354bc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x165c>
   34edc:	test   r8,r8
   34edf:	mov    rdx,QWORD PTR [rsp+0x220]
   34ee7:	je     35275 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1415>
   34eed:	mov    rax,QWORD PTR [rsp+0x70]
   34ef2:	add    rax,r9
   34ef5:	mov    QWORD PTR [rsp+0x238],rax
   34efd:	cmp    r8,0x2
   34f01:	jb     35285 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1425>
   34f07:	mov    rax,QWORD PTR [rsp+0x78]
   34f0c:	add    rax,r9
   34f0f:	mov    QWORD PTR [rsp+0x258],rax
   34f17:	cmp    r8,0x2
   34f1b:	jbe    35298 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1438>
   34f21:	mov    rax,QWORD PTR [rsp+0x80]
   34f29:	add    rax,r9
   34f2c:	mov    QWORD PTR [rsp+0x278],rax
   34f34:	cmp    r8,0x3
   34f38:	jbe    352ab <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x144b>
   34f3e:	mov    rax,QWORD PTR [rsp+0x88]
   34f46:	add    rax,r9
   34f49:	mov    QWORD PTR [rsp+0x298],rax
   34f51:	cmp    r8,0x4
   34f55:	jbe    352be <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x145e>
   34f5b:	mov    rax,QWORD PTR [rsp+0x90]
   34f63:	add    rax,r9
   34f66:	mov    QWORD PTR [rsp+0x2b8],rax
   34f6e:	cmp    r8,0x5
   34f72:	jbe    352d1 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1471>
   34f78:	mov    rax,QWORD PTR [rsp+0x98]
   34f80:	add    rax,r9
   34f83:	mov    QWORD PTR [rsp+0x2d8],rax
   34f8b:	cmp    r8,0x6
   34f8f:	jbe    352e4 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1484>
   34f95:	mov    rax,QWORD PTR [rsp+0xa0]
   34f9d:	add    rax,r9
   34fa0:	mov    QWORD PTR [rsp+0x2f8],rax
   34fa8:	cmp    r8,0x7
   34fac:	jbe    352f7 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1497>
   34fb2:	mov    rax,QWORD PTR [rsp+0xa8]
   34fba:	add    rax,r9
   34fbd:	mov    QWORD PTR [rsp+0x318],rax
   34fc5:	cmp    r8,0x8
   34fc9:	jbe    3530a <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14aa>
   34fcf:	mov    rax,QWORD PTR [rsp+0xb0]
   34fd7:	add    rax,r9
   34fda:	mov    QWORD PTR [rsp+0x338],rax
   34fe2:	cmp    r8,0x9
   34fe6:	jbe    3531d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14bd>
   34fec:	mov    rax,QWORD PTR [rsp+0xb8]
   34ff4:	add    rax,r9
   34ff7:	mov    QWORD PTR [rsp+0x358],rax
   34fff:	cmp    r8,0xa
   35003:	jbe    35330 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14d0>
   35009:	mov    rax,QWORD PTR [rsp+0xc0]
   35011:	add    rax,r9
   35014:	mov    QWORD PTR [rsp+0x378],rax
   3501c:	cmp    r8,0xb
   35020:	jbe    35343 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14e3>
   35026:	mov    rax,QWORD PTR [rsp+0xc8]
   3502e:	add    rax,r9
   35031:	mov    QWORD PTR [rsp+0x398],rax
   35039:	cmp    r8,0xc
   3503d:	jbe    35356 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14f6>
   35043:	mov    rax,QWORD PTR [rsp+0xd0]
   3504b:	add    rax,r9
   3504e:	mov    QWORD PTR [rsp+0x3b8],rax
   35056:	cmp    r8,0xd
   3505a:	jbe    35369 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1509>
   35060:	mov    rsi,r9
   35063:	mov    rax,QWORD PTR [rsp+0xd8]
   3506b:	add    rax,r9
   3506e:	mov    QWORD PTR [rsp+0x3d8],rax
   35076:	cmp    r8,0xe
   3507a:	jbe    3537c <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x151c>
   35080:	mov    rax,QWORD PTR [rsp+0xe0]
   35088:	add    rax,rsi
   3508b:	mov    QWORD PTR [rsp+0x3f8],rax
   35093:	cmp    r8,0xf
   35097:	jbe    3538f <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x152f>
   3509d:	mov    rax,QWORD PTR [rsp+0xe8]
   350a5:	add    rax,rsi
   350a8:	mov    QWORD PTR [rsp+0x418],rax
   350b0:	cmp    r8,0x10
   350b4:	jbe    353a2 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1542>
   350ba:	mov    rax,QWORD PTR [rsp+0xf0]
   350c2:	add    rax,rsi
   350c5:	mov    QWORD PTR [rsp+0x438],rax
   350cd:	cmp    r8,0x11
   350d1:	jbe    353b5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1555>
   350d7:	mov    rax,QWORD PTR [rsp+0xf8]
   350df:	add    rax,rsi
   350e2:	mov    QWORD PTR [rsp+0x458],rax
   350ea:	cmp    r8,0x12
   350ee:	jbe    353c8 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1568>
   350f4:	mov    rax,QWORD PTR [rsp+0x100]
   350fc:	add    rax,rsi
   350ff:	mov    QWORD PTR [rsp+0x478],rax
   35107:	cmp    r8,0x13
   3510b:	jbe    353db <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x157b>
   35111:	mov    rax,QWORD PTR [rsp+0x108]
   35119:	add    rax,rsi
   3511c:	mov    QWORD PTR [rsp+0x498],rax
   35124:	cmp    r8,0x14
   35128:	jbe    353ee <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x158e>
   3512e:	mov    rax,QWORD PTR [rsp+0x110]
   35136:	add    rax,rsi
   35139:	mov    QWORD PTR [rsp+0x4b8],rax
   35141:	cmp    r8,0x15
   35145:	jbe    35401 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15a1>
   3514b:	mov    rax,QWORD PTR [rsp+0x118]
   35153:	add    rax,rsi
   35156:	mov    QWORD PTR [rsp+0x4d8],rax
   3515e:	cmp    r8,0x16
   35162:	jbe    35414 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15b4>
   35168:	mov    rax,QWORD PTR [rsp+0x120]
   35170:	add    rax,rsi
   35173:	mov    QWORD PTR [rsp+0x4f8],rax
   3517b:	cmp    r8,0x17
   3517f:	jbe    35427 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15c7>
   35185:	mov    rax,QWORD PTR [rsp+0x128]
   3518d:	add    rax,rsi
   35190:	mov    QWORD PTR [rsp+0x518],rax
   35198:	cmp    r8,0x18
   3519c:	jbe    3543a <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15da>
   351a2:	mov    rax,QWORD PTR [rsp+0x130]
   351aa:	add    rax,rsi
   351ad:	mov    QWORD PTR [rsp+0x538],rax
   351b5:	cmp    r8,0x19
   351b9:	jbe    3544d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x15ed>
   351bf:	mov    rax,QWORD PTR [rsp+0x138]
   351c7:	add    rax,rsi
   351ca:	mov    QWORD PTR [rsp+0x558],rax
   351d2:	cmp    r8,0x1a
   351d6:	jbe    35460 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1600>
   351dc:	mov    rax,QWORD PTR [rsp+0x140]
   351e4:	add    rax,rsi
   351e7:	mov    QWORD PTR [rsp+0x578],rax
   351ef:	cmp    r8,0x1b
   351f3:	jbe    35473 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1613>
   351f9:	mov    rax,QWORD PTR [rsp+0x148]
   35201:	add    rax,rsi
   35204:	mov    QWORD PTR [rsp+0x598],rax
   3520c:	cmp    r8,0x1c
   35210:	jbe    35486 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1626>
   35216:	mov    rax,QWORD PTR [rsp+0x150]
   3521e:	add    rax,rsi
   35221:	mov    QWORD PTR [rsp+0x5b8],rax
   35229:	cmp    r8,0x1d
   3522d:	jbe    35499 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1639>
   35233:	mov    rax,QWORD PTR [rsp+0x158]
   3523b:	add    rax,rsi
   3523e:	mov    QWORD PTR [rsp+0x5d8],rax
   35246:	cmp    r8,0x1e
   3524a:	jbe    354ac <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x164c>
   35250:	mov    rax,QWORD PTR [rsp+0x160]
   35258:	mov    rcx,rsi
   3525b:	add    rax,rsi
   3525e:	mov    QWORD PTR [rsp+0x5f8],rax
   35266:	cmp    r8,0x20
   3526a:	jae    354ac <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x164c>
   35270:	jmp    354bc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x165c>
   35275:	mov    rax,QWORD PTR [rsp+0x78]
   3527a:	add    rax,rcx
   3527d:	mov    QWORD PTR [rsp+0x258],rax
   35285:	mov    rax,QWORD PTR [rsp+0x80]
   3528d:	add    rax,rcx
   35290:	mov    QWORD PTR [rsp+0x278],rax
   35298:	mov    rax,QWORD PTR [rsp+0x88]
   352a0:	add    rax,rcx
   352a3:	mov    QWORD PTR [rsp+0x298],rax
   352ab:	mov    rax,QWORD PTR [rsp+0x90]
   352b3:	add    rax,rcx
   352b6:	mov    QWORD PTR [rsp+0x2b8],rax
   352be:	mov    rax,QWORD PTR [rsp+0x98]
   352c6:	add    rax,rcx
   352c9:	mov    QWORD PTR [rsp+0x2d8],rax
   352d1:	mov    rax,QWORD PTR [rsp+0xa0]
   352d9:	add    rax,rcx
   352dc:	mov    QWORD PTR [rsp+0x2f8],rax
   352e4:	mov    rax,QWORD PTR [rsp+0xa8]
   352ec:	add    rax,rcx
   352ef:	mov    QWORD PTR [rsp+0x318],rax
   352f7:	mov    rax,QWORD PTR [rsp+0xb0]
   352ff:	add    rax,rcx
   35302:	mov    QWORD PTR [rsp+0x338],rax
   3530a:	mov    rax,QWORD PTR [rsp+0xb8]
   35312:	add    rax,rcx
   35315:	mov    QWORD PTR [rsp+0x358],rax
   3531d:	mov    rax,QWORD PTR [rsp+0xc0]
   35325:	add    rax,rcx
   35328:	mov    QWORD PTR [rsp+0x378],rax
   35330:	mov    rax,QWORD PTR [rsp+0xc8]
   35338:	add    rax,rcx
   3533b:	mov    QWORD PTR [rsp+0x398],rax
   35343:	mov    rax,QWORD PTR [rsp+0xd0]
   3534b:	add    rax,rcx
   3534e:	mov    QWORD PTR [rsp+0x3b8],rax
   35356:	mov    rax,QWORD PTR [rsp+0xd8]
   3535e:	add    rax,rcx
   35361:	mov    QWORD PTR [rsp+0x3d8],rax
   35369:	mov    rax,QWORD PTR [rsp+0xe0]
   35371:	add    rax,rcx
   35374:	mov    QWORD PTR [rsp+0x3f8],rax
   3537c:	mov    rax,QWORD PTR [rsp+0xe8]
   35384:	add    rax,rcx
   35387:	mov    QWORD PTR [rsp+0x418],rax
   3538f:	mov    rax,QWORD PTR [rsp+0xf0]
   35397:	add    rax,rcx
   3539a:	mov    QWORD PTR [rsp+0x438],rax
   353a2:	mov    rax,QWORD PTR [rsp+0xf8]
   353aa:	add    rax,rcx
   353ad:	mov    QWORD PTR [rsp+0x458],rax
   353b5:	mov    rax,QWORD PTR [rsp+0x100]
   353bd:	add    rax,rcx
   353c0:	mov    QWORD PTR [rsp+0x478],rax
   353c8:	mov    rax,QWORD PTR [rsp+0x108]
   353d0:	add    rax,rcx
   353d3:	mov    QWORD PTR [rsp+0x498],rax
   353db:	mov    rax,QWORD PTR [rsp+0x110]
   353e3:	add    rax,rcx
   353e6:	mov    QWORD PTR [rsp+0x4b8],rax
   353ee:	mov    rax,QWORD PTR [rsp+0x118]
   353f6:	add    rax,rcx
   353f9:	mov    QWORD PTR [rsp+0x4d8],rax
   35401:	mov    rax,QWORD PTR [rsp+0x120]
   35409:	add    rax,rcx
   3540c:	mov    QWORD PTR [rsp+0x4f8],rax
   35414:	mov    rax,QWORD PTR [rsp+0x128]
   3541c:	add    rax,rcx
   3541f:	mov    QWORD PTR [rsp+0x518],rax
   35427:	mov    rax,QWORD PTR [rsp+0x130]
   3542f:	add    rax,rcx
   35432:	mov    QWORD PTR [rsp+0x538],rax
   3543a:	mov    rax,QWORD PTR [rsp+0x138]
   35442:	add    rax,rcx
   35445:	mov    QWORD PTR [rsp+0x558],rax
   3544d:	mov    rax,QWORD PTR [rsp+0x140]
   35455:	add    rax,rcx
   35458:	mov    QWORD PTR [rsp+0x578],rax
   35460:	mov    rax,QWORD PTR [rsp+0x148]
   35468:	add    rax,rcx
   3546b:	mov    QWORD PTR [rsp+0x598],rax
   35473:	mov    rax,QWORD PTR [rsp+0x150]
   3547b:	add    rax,rcx
   3547e:	mov    QWORD PTR [rsp+0x5b8],rax
   35486:	mov    rax,QWORD PTR [rsp+0x158]
   3548e:	add    rax,rcx
   35491:	mov    QWORD PTR [rsp+0x5d8],rax
   35499:	mov    rax,QWORD PTR [rsp+0x160]
   354a1:	add    rax,rcx
   354a4:	mov    QWORD PTR [rsp+0x5f8],rax
   354ac:	add    rcx,QWORD PTR [rsp+0x168]
   354b4:	mov    QWORD PTR [rsp+0x618],rcx
   354bc:	shl    rdx,0x5
   354c0:	lea    r12,[rsp+rdx*1]
   354c4:	add    r12,0x230
   354cb:	lea    r14,[rsp+0x230]
   354d3:	mov    rax,QWORD PTR [rsp+0x50]
   354d8:	jmp    354ed <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x168d>
   354da:	nop    WORD PTR [rax+rax*1+0x0]
   354e0:	mov    r14,rbp
   354e3:	add    r13,0xffffffffffffffe0
   354e7:	je     3566b <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x180b>
   354ed:	lea    rbp,[r14+0x20]
   354f1:	mov    rbx,QWORD PTR [r14]
   354f4:	cmp    rbx,0xffffffffffffffff
   354f8:	je     354e0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1680>
   354fa:	cmp    rbx,QWORD PTR [rsp+0x10]
   354ff:	jae    35c39 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dd9>
   35505:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   3550a:	jne    354e0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1680>
   3550c:	mov    edi,0x80
   35511:	call   QWORD PTR [rip+0x60749]        # 95c60 <malloc@GLIBC_2.2.5>
   35517:	test   rax,rax
   3551a:	je     35c76 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e16>
   35520:	mov    rcx,QWORD PTR [r14+0x18]
   35524:	mov    QWORD PTR [rax+0x18],rcx
   35528:	movups xmm0,XMMWORD PTR [r14+0x8]
   3552d:	movups XMMWORD PTR [rax+0x8],xmm0
   35531:	mov    QWORD PTR [rax],rbx
   35534:	mov    QWORD PTR [rsp+0x178],0x4
   35540:	mov    QWORD PTR [rsp+0x180],rax
   35548:	mov    QWORD PTR [rsp+0x188],0x1
   35554:	mov    r13d,0x1
   3555a:	cmp    rbp,r12
   3555d:	je     35627 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17c7>
   35563:	lea    r14,[rsp+0x178]
   3556b:	mov    rdx,QWORD PTR [rsp+0x30]
   35570:	mov    rcx,QWORD PTR [rdx+0x50]
   35574:	mov    rsi,QWORD PTR [rdx+0x58]
   35578:	mov    rdx,rbp
   3557b:	jmp    3558c <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x172c>
   3557d:	nop    DWORD PTR [rax]
   35580:	mov    rdx,rbp
   35583:	cmp    rbp,r12
   35586:	je     35627 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17c7>
   3558c:	lea    rbp,[rdx+0x20]
   35590:	mov    rbx,QWORD PTR [rdx]
   35593:	cmp    rbx,0xffffffffffffffff
   35597:	je     35580 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1720>
   35599:	cmp    rbx,rsi
   3559c:	jae    35c4d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ded>
   355a2:	cmp    QWORD PTR [rcx+rbx*8],0xffffffffffffffff
   355a7:	jne    35580 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1720>
   355a9:	mov    rcx,QWORD PTR [rdx+0x18]
   355ad:	mov    QWORD PTR [rsp+0x640],rcx
   355b5:	movups xmm0,XMMWORD PTR [rdx+0x8]
   355b9:	movaps XMMWORD PTR [rsp+0x630],xmm0
   355c1:	cmp    r13,QWORD PTR [rsp+0x178]
   355c9:	jne    355ee <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x178e>
   355cb:	mov    edx,0x1
   355d0:	mov    ecx,0x8
   355d5:	mov    r8d,0x20
   355db:	mov    rdi,r14
   355de:	mov    rsi,r13
   355e1:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   355e6:	mov    rax,QWORD PTR [rsp+0x180]
   355ee:	mov    rcx,r13
   355f1:	shl    rcx,0x5
   355f5:	mov    QWORD PTR [rax+rcx*1],rbx
   355f9:	movaps xmm0,XMMWORD PTR [rsp+0x630]
   35601:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   35606:	mov    rdx,QWORD PTR [rsp+0x640]
   3560e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   35613:	inc    r13
   35616:	mov    QWORD PTR [rsp+0x188],r13
   3561e:	cmp    rbp,r12
   35621:	jne    3556b <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x170b>
   35627:	mov    rax,QWORD PTR [rsp+0x178]
   3562f:	mov    QWORD PTR [rsp+0x18],rax
   35634:	mov    rax,QWORD PTR [rsp+0x180]
   3563c:	mov    QWORD PTR [rsp+0x8],rax
   35641:	mov    rbp,r13
   35644:	shl    rbp,0x5
   35648:	mov    rax,QWORD PTR [rsp+0x30]
   3564d:	mov    r14,QWORD PTR [rax+0x38]
   35651:	mov    rax,QWORD PTR [rax+0x40]
   35655:	mov    QWORD PTR [rsp+0x28],rax
   3565a:	xor    r12d,r12d
   3565d:	lea    rax,[rip+0x5e414]        # 93a78 <__frame_dummy_init_array_entry+0xb30>
   35664:	mov    QWORD PTR [rsp+0x38],rax
   35669:	jmp    356ce <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x186e>
   3566b:	mov    eax,0x8
   35670:	mov    QWORD PTR [rsp+0x8],rax
   35675:	xor    r14d,r14d
   35678:	mov    r9,QWORD PTR [rsp+0x20]
   3567d:	jmp    35914 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ab4>
   35682:	mov    rdi,r12
   35685:	call   35e20 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCs5Rf2Z7g0iFY_5mibz2>
   3568a:	mov    edi,DWORD PTR [rsp+0x4c]
   3568e:	mov    r14,QWORD PTR [r12+0x8]
   35693:	jmp    33f15 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb5>
   35698:	mov    rax,QWORD PTR [rsp+0x20]
   3569d:	mov    rdx,QWORD PTR [rax]
   356a0:	mov    rax,QWORD PTR [rax+0x8]
   356a4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   356b0:	add    rbx,rax
   356b3:	mov    rcx,QWORD PTR [rsp+0x8]
   356b8:	mov    QWORD PTR [rcx+r12*1+0x8],rbx
   356bd:	add    rdx,rax
   356c0:	mov    QWORD PTR [rcx+r12*1+0x10],rdx
   356c5:	add    r12,0x20
   356c9:	cmp    rbp,r12
   356cc:	je     35734 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d4>
   356ce:	mov    rax,QWORD PTR [rsp+0x8]
   356d3:	mov    rbx,QWORD PTR [rax+r12*1+0x8]
   356d8:	cmp    rbx,QWORD PTR [rax+r12*1+0x10]
   356dd:	jne    356c5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1865>
   356df:	mov    rax,QWORD PTR [rsp+0x8]
   356e4:	mov    rdi,QWORD PTR [rax+r12*1]
   356e8:	cmp    rdi,QWORD PTR [rsp+0x28]
   356ed:	jae    35bbe <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d5e>
   356f3:	lea    rcx,[rdi+rdi*2]
   356f7:	mov    rdx,QWORD PTR [r14+rcx*8]
   356fb:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   35700:	sub    rbx,rax
   35703:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   35708:	mov    rsi,rdx
   3570b:	sub    rsi,rbx
   3570e:	cmp    rdx,rsi
   35711:	jbe    356b0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1850>
   35713:	lea    rdi,[r14+rcx*8]
   35717:	mov    ecx,0x1
   3571c:	mov    r8d,0x1
   35722:	mov    QWORD PTR [rsp+0x20],rdi
   35727:	mov    rsi,rbx
   3572a:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   3572f:	jmp    35698 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1838>
   35734:	mov    rax,QWORD PTR [rsp+0x30]
   35739:	mov    rcx,QWORD PTR [rax+0x50]
   3573d:	mov    QWORD PTR [rsp+0x40],rcx
   35742:	mov    rcx,QWORD PTR [rax+0x58]
   35746:	mov    QWORD PTR [rsp+0x60],rcx
   3574b:	mov    rcx,QWORD PTR [rax+0x8]
   3574f:	mov    QWORD PTR [rsp+0x20],rcx
   35754:	mov    rax,QWORD PTR [rax+0x10]
   35758:	mov    QWORD PTR [rsp+0x68],rax
   3575d:	xor    ebp,ebp
   3575f:	jmp    35772 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1912>
   35761:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35770:	xor    ebp,ebp
   35772:	mov    r12,rbp
   35775:	shl    r12,0x5
   35779:	add    r12,QWORD PTR [rsp+0x8]
   3577e:	jmp    357ac <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x194c>
   35780:	mov    rax,QWORD PTR [rsp+0x10]
   35785:	mov    rdx,QWORD PTR [rax]
   35788:	mov    rax,QWORD PTR [rax+0x8]
   3578c:	nop    DWORD PTR [rax+0x0]
   35790:	add    rbx,rax
   35793:	mov    QWORD PTR [r12+0x8],rbx
   35798:	add    rdx,rax
   3579b:	mov    QWORD PTR [r12+0x10],rdx
   357a0:	inc    rbp
   357a3:	add    r12,0x20
   357a7:	cmp    rbp,r13
   357aa:	jae    35770 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1910>
   357ac:	mov    eax,DWORD PTR [r12+0x18]
   357b1:	mov    ecx,DWORD PTR [r15+rax*4]
   357b5:	test   ecx,ecx
   357b7:	js     35840 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19e0>
   357bd:	mov    eax,ecx
   357bf:	shr    eax,0x8
   357c2:	mov    DWORD PTR [r12+0x18],eax
   357c7:	prefetcht0 BYTE PTR [r15+rax*4]
   357cc:	mov    rax,QWORD PTR [r12+0x8]
   357d1:	mov    BYTE PTR [rax],cl
   357d3:	mov    rbx,QWORD PTR [r12+0x8]
   357d8:	inc    rbx
   357db:	mov    QWORD PTR [r12+0x8],rbx
   357e0:	cmp    rbx,QWORD PTR [r12+0x10]
   357e5:	jne    357a0 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1940>
   357e7:	mov    rdi,QWORD PTR [r12]
   357eb:	cmp    rdi,QWORD PTR [rsp+0x28]
   357f0:	jae    35bbe <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d5e>
   357f6:	lea    rcx,[rdi+rdi*2]
   357fa:	mov    rdx,QWORD PTR [r14+rcx*8]
   357fe:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   35803:	sub    rbx,rax
   35806:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   3580b:	mov    rsi,rdx
   3580e:	sub    rsi,rbx
   35811:	cmp    rdx,rsi
   35814:	jbe    35790 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1930>
   3581a:	lea    rdi,[r14+rcx*8]
   3581e:	mov    ecx,0x1
   35823:	mov    r8d,0x1
   35829:	mov    QWORD PTR [rsp+0x10],rdi
   3582e:	mov    rsi,rbx
   35831:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   35836:	jmp    35780 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1920>
   3583b:	nop    DWORD PTR [rax+rax*1+0x0]
   35840:	mov    rdi,QWORD PTR [r12]
   35844:	cmp    rdi,QWORD PTR [rsp+0x28]
   35849:	jae    35b9f <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d3f>
   3584f:	lea    rcx,[rdi+rdi*2]
   35853:	mov    rdx,QWORD PTR [r12+0x8]
   35858:	sub    rdx,QWORD PTR [r14+rcx*8+0x8]
   3585d:	mov    QWORD PTR [r14+rcx*8+0x10],rdx
   35862:	mov    rdx,QWORD PTR [rsp+0x68]
   35867:	test   rdx,rdx
   3586a:	je     3587b <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a1b>
   3586c:	xor    ecx,ecx
   3586e:	cmp    rdx,0x1
   35872:	jne    35884 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a24>
   35874:	mov    r9,QWORD PTR [rsp+0x20]
   35879:	jmp    358ad <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a4d>
   3587b:	mov    rcx,0xffffffffffffffff
   35882:	jmp    358bc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a5c>
   35884:	mov    r9,QWORD PTR [rsp+0x20]
   35889:	nop    DWORD PTR [rax+0x0]
   35890:	mov    rsi,rcx
   35893:	mov    r8,rdx
   35896:	shr    r8,1
   35899:	add    rcx,r8
   3589c:	cmp    DWORD PTR [r9+rcx*4],eax
   358a0:	cmova  rcx,rsi
   358a4:	sub    rdx,r8
   358a7:	cmp    rdx,0x1
   358ab:	ja     35890 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a30>
   358ad:	cmp    DWORD PTR [r9+rcx*4],eax
   358b1:	mov    rax,0xffffffffffffffff
   358b8:	cmovne rcx,rax
   358bc:	cmp    rdi,QWORD PTR [rsp+0x60]
   358c1:	jae    35ba8 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d48>
   358c7:	mov    rax,QWORD PTR [rsp+0x40]
   358cc:	mov    QWORD PTR [rax+rdi*8],rcx
   358d0:	dec    r13
   358d3:	mov    rax,r13
   358d6:	shl    rax,0x5
   358da:	mov    rcx,QWORD PTR [rsp+0x8]
   358df:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   358e3:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   358e8:	movups XMMWORD PTR [r12+0x10],xmm1
   358ee:	movups XMMWORD PTR [r12],xmm0
   358f3:	cmp    rbp,r13
   358f6:	jb     35772 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1912>
   358fc:	mov    ebp,0x0
   35901:	test   r13,r13
   35904:	jne    35772 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1912>
   3590a:	mov    r9,QWORD PTR [rsp+0x20]
   3590f:	mov    r14,QWORD PTR [rsp+0x18]
   35914:	mov    rax,0xffffffffffffffff
   3591b:	mov    r10,QWORD PTR [rsp+0x68]
   35920:	test   r10,r10
   35923:	mov    rsi,QWORD PTR [rsp+0x170]
   3592b:	mov    r8,QWORD PTR [rsp+0x30]
   35930:	je     359ac <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b4c>
   35932:	lea    rcx,[r10*4+0x0]
   3593a:	xor    edx,edx
   3593c:	nop    DWORD PTR [rax+0x0]
   35940:	mov    edi,DWORD PTR [r9+rdx*1]
   35944:	cmp    rsi,rdi
   35947:	jbe    35bd7 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d77>
   3594d:	and    BYTE PTR [r15+rdi*4+0x3],0x7f
   35953:	add    rdx,0x4
   35957:	cmp    rcx,rdx
   3595a:	jne    35940 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ae0>
   3595c:	test   r10,r10
   3595f:	je     359ac <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b4c>
   35961:	xor    ecx,ecx
   35963:	cmp    r10,0x1
   35967:	jne    3596f <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0f>
   35969:	mov    edi,DWORD PTR [rsp+0x4c]
   3596d:	jmp    3599d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b3d>
   3596f:	mov    edi,DWORD PTR [rsp+0x4c]
   35973:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35980:	mov    rax,rcx
   35983:	mov    rdx,r10
   35986:	shr    rdx,1
   35989:	add    rcx,rdx
   3598c:	cmp    DWORD PTR [r9+rcx*4],edi
   35990:	cmova  rcx,rax
   35994:	sub    r10,rdx
   35997:	cmp    r10,0x1
   3599b:	ja     35980 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b20>
   3599d:	cmp    DWORD PTR [r9+rcx*4],edi
   359a1:	mov    rax,0xffffffffffffffff
   359a8:	cmove  rax,rcx
   359ac:	mov    QWORD PTR [r8+0x68],rax
   359b0:	mov    QWORD PTR [r8+0x60],rsi
   359b4:	test   r14,r14
   359b7:	je     359c4 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b64>
   359b9:	mov    rdi,QWORD PTR [rsp+0x8]
   359be:	call   QWORD PTR [rip+0x602a4]        # 95c68 <free@GLIBC_2.2.5>
   359c4:	add    rsp,0x648
   359cb:	pop    rbx
   359cc:	pop    r12
   359ce:	pop    r13
   359d0:	pop    r14
   359d2:	pop    r15
   359d4:	pop    rbp
   359d5:	ret
   359d6:	mov    edx,0x1ff
   359db:	mov    r14,rcx
   359de:	mov    ecx,0x4
   359e3:	mov    r8d,0x4
   359e9:	mov    ebp,edi
   359eb:	mov    rdi,r14
   359ee:	mov    rbx,rsi
   359f1:	xor    esi,esi
   359f3:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   359f8:	mov    edi,ebp
   359fa:	mov    rcx,r14
   359fd:	mov    rsi,rbx
   35a00:	mov    r13,QWORD PTR [r14+0x10]
   35a04:	jmp    33ea5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x45>
   35a09:	cmp    r9,0x15
   35a0d:	mov    r12,QWORD PTR [rsp+0x170]
   35a15:	jae    35c85 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e25>
   35a1b:	lea    rax,[r14+r9*4]
   35a1f:	lea    rcx,[r14+0x4]
   35a23:	mov    edx,0x4
   35a28:	mov    rbp,QWORD PTR [rsp+0x30]
   35a2d:	jmp    35a45 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1be5>
   35a2f:	mov    rdi,r14
   35a32:	mov    DWORD PTR [rdi],esi
   35a34:	add    rcx,0x4
   35a38:	add    rdx,0x4
   35a3c:	cmp    rcx,rax
   35a3f:	je     35c9b <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e3b>
   35a45:	mov    r8d,DWORD PTR [rcx-0x4]
   35a49:	mov    esi,DWORD PTR [rcx]
   35a4b:	cmp    esi,r8d
   35a4e:	jae    35a34 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bd4>
   35a50:	mov    rdi,rdx
   35a53:	mov    DWORD PTR [r14+rdi*1],r8d
   35a57:	cmp    rdi,0x4
   35a5b:	je     35a2f <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bcf>
   35a5d:	mov    r8d,DWORD PTR [r14+rdi*1-0x8]
   35a62:	add    rdi,0xfffffffffffffffc
   35a66:	cmp    esi,r8d
   35a69:	jb     35a53 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bf3>
   35a6b:	add    rdi,r14
   35a6e:	jmp    35a32 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bd2>
   35a70:	lea    rdi,[rbp+0x48]
   35a74:	mov    ecx,0x8
   35a79:	mov    r8d,0x8
   35a7f:	xor    esi,esi
   35a81:	mov    rdx,r9
   35a84:	mov    rbx,rax
   35a87:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   35a8c:	mov    rax,rbx
   35a8f:	mov    r9,QWORD PTR [rsp+0x58]
   35a94:	mov    rbx,QWORD PTR [rbp+0x58]
   35a98:	mov    r14,QWORD PTR [rbp+0x50]
   35a9c:	lea    rdi,[r14+rbx*8]
   35aa0:	cmp    r9,0x1
   35aa4:	jne    34068 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x208>
   35aaa:	jmp    34098 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x238>
   35aaf:	lea    rdi,[rbp+0x30]
   35ab3:	mov    ecx,0x8
   35ab8:	mov    r8d,0x18
   35abe:	mov    rsi,rbx
   35ac1:	mov    rdx,r14
   35ac4:	mov    r12,rax
   35ac7:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs5Rf2Z7g0iFY_5mibz2>
   35acc:	mov    rax,r12
   35acf:	mov    r9,QWORD PTR [rsp+0x58]
   35ad4:	mov    rdx,QWORD PTR [rbp+0x40]
   35ad8:	mov    rsi,QWORD PTR [rbp+0x38]
   35adc:	inc    rbx
   35adf:	test   r14b,0x1
   35ae3:	je     33fc8 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x168>
   35ae9:	dec    r14
   35aec:	lea    rcx,[rdx+rdx*2]
   35af0:	mov    QWORD PTR [rsi+rcx*8],0x0
   35af8:	mov    QWORD PTR [rsi+rcx*8+0x8],0x1
   35b01:	mov    QWORD PTR [rsi+rcx*8+0x10],0x0
   35b0a:	inc    rdx
   35b0d:	cmp    r9,rbx
   35b10:	je     33fd1 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x171>
   35b16:	lea    rcx,[rdx+rdx*2]
   35b1a:	lea    rsi,[rsi+rcx*8]
   35b1e:	add    rsi,0x28
   35b22:	neg    r14
   35b25:	xor    ecx,ecx
   35b27:	xorps  xmm0,xmm0
   35b2a:	nop    WORD PTR [rax+rax*1+0x0]
   35b30:	mov    QWORD PTR [rsi-0x28],0x0
   35b38:	mov    QWORD PTR [rsi-0x20],0x1
   35b40:	movups XMMWORD PTR [rsi-0x18],xmm0
   35b44:	mov    QWORD PTR [rsi-0x8],0x1
   35b4c:	mov    QWORD PTR [rsi],0x0
   35b53:	add    rsi,0x30
   35b57:	add    rcx,0xfffffffffffffffe
   35b5b:	cmp    r14,rcx
   35b5e:	jne    35b30 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cd0>
   35b60:	sub    rdx,rcx
   35b63:	mov    QWORD PTR [rbp+0x40],rdx
   35b67:	cmp    r9,rdx
   35b6a:	jbe    33fde <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e>
   35b70:	jmp    34eb4 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1054>
   35b75:	lea    rdx,[rip+0x5dc8c]        # 93808 <__frame_dummy_init_array_entry+0x8c0>
   35b7c:	mov    rsi,r12
   35b7f:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35b84:	lea    rdx,[rip+0x5df1d]        # 93aa8 <__frame_dummy_init_array_entry+0xb60>
   35b8b:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35b90:	lea    rdx,[rip+0x5df29]        # 93ac0 <__frame_dummy_init_array_entry+0xb78>
   35b97:	mov    rsi,r14
   35b9a:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35b9f:	lea    rax,[rip+0x5dea2]        # 93a48 <__frame_dummy_init_array_entry+0xb00>
   35ba6:	jmp    35bb9 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d59>
   35ba8:	mov    rax,QWORD PTR [rsp+0x60]
   35bad:	mov    QWORD PTR [rsp+0x28],rax
   35bb2:	lea    rax,[rip+0x5dea7]        # 93a60 <__frame_dummy_init_array_entry+0xb18>
   35bb9:	mov    QWORD PTR [rsp+0x38],rax
   35bbe:	mov    rsi,QWORD PTR [rsp+0x28]
   35bc3:	mov    rdx,QWORD PTR [rsp+0x38]
   35bc8:	mov    r14,QWORD PTR [rsp+0x18]
   35bcd:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35bd2:	jmp    35c5c <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dfc>
   35bd7:	lea    rdx,[rip+0x5dc5a]        # 93838 <__frame_dummy_init_array_entry+0x8f0>
   35bde:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35be3:	jmp    35c5c <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dfc>
   35be5:	lea    rdx,[rip+0x5de8c]        # 93a78 <__frame_dummy_init_array_entry+0xb30>
   35bec:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35bf1:	lea    rdx,[rip+0x5de50]        # 93a48 <__frame_dummy_init_array_entry+0xb00>
   35bf8:	mov    rsi,QWORD PTR [rsp+0x8]
   35bfd:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c02:	lea    rdx,[rip+0x5de57]        # 93a60 <__frame_dummy_init_array_entry+0xb18>
   35c09:	mov    rsi,QWORD PTR [rsp+0x10]
   35c0e:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c13:	lea    rdx,[rip+0x5dea6]        # 93ac0 <__frame_dummy_init_array_entry+0xb78>
   35c1a:	mov    rdi,rbx
   35c1d:	mov    rsi,r14
   35c20:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c25:	lea    rdx,[rip+0x5de7c]        # 93aa8 <__frame_dummy_init_array_entry+0xb60>
   35c2c:	mov    rdi,rbx
   35c2f:	mov    rsi,QWORD PTR [rsp+0x8]
   35c34:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c39:	lea    rdx,[rip+0x5de50]        # 93a90 <__frame_dummy_init_array_entry+0xb48>
   35c40:	mov    rdi,rbx
   35c43:	mov    rsi,QWORD PTR [rsp+0x10]
   35c48:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c4d:	lea    rdx,[rip+0x5de3c]        # 93a90 <__frame_dummy_init_array_entry+0xb48>
   35c54:	mov    rdi,rbx
   35c57:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c5c:	ud2
   35c5e:	lea    rdx,[rip+0x5db8b]        # 937f0 <__frame_dummy_init_array_entry+0x8a8>
   35c65:	call   2a48b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   35c6a:	lea    rdi,[rip+0x5dbaf]        # 93820 <__frame_dummy_init_array_entry+0x8d8>
   35c71:	call   2ce80 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   35c76:	mov    edi,0x8
   35c7b:	mov    esi,0x80
   35c80:	call   28ceb <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   35c85:	mov    rdi,r14
   35c88:	mov    rsi,r9
   35c8b:	mov    rbp,r9
   35c8e:	call   35e90 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECs5Rf2Z7g0iFY_5mibz2>
   35c93:	mov    r9,rbp
   35c96:	mov    rbp,QWORD PTR [rsp+0x30]
   35c9b:	xor    ecx,ecx
   35c9d:	mov    eax,DWORD PTR [r14+rcx*4+0x4]
   35ca2:	cmp    eax,DWORD PTR [r14+rcx*4]
   35ca6:	je     35cb5 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e55>
   35ca8:	inc    rcx
   35cab:	cmp    rbx,rcx
   35cae:	jne    35c9d <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e3d>
   35cb0:	jmp    33f39 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd9>
   35cb5:	lea    rdx,[rcx+0x1]
   35cb9:	lea    rax,[rcx+0x2]
   35cbd:	cmp    rax,r9
   35cc0:	jae    35cd6 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e76>
   35cc2:	mov    esi,r13d
   35cc5:	sub    esi,ecx
   35cc7:	add    r13,0x1fd
   35cce:	test   sil,0x1
   35cd2:	jne    35cdb <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e7b>
   35cd4:	jmp    35cfc <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e9c>
   35cd6:	mov    r9,rdx
   35cd9:	jmp    35d01 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ea1>
   35cdb:	mov    eax,DWORD PTR [r14+rcx*4+0x8]
   35ce0:	mov    r9,rdx
   35ce3:	cmp    eax,DWORD PTR [r14+rcx*4]
   35ce7:	je     35cf2 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e92>
   35ce9:	mov    DWORD PTR [r14+rcx*4+0x4],eax
   35cee:	lea    r9,[rdx+0x1]
   35cf2:	add    rdx,0x2
   35cf6:	mov    rax,rdx
   35cf9:	mov    rdx,r9
   35cfc:	cmp    r13,rcx
   35cff:	jne    35d0a <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eaa>
   35d01:	mov    QWORD PTR [rbp+0x10],r9
   35d05:	jmp    33f39 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd9>
   35d0a:	inc    rbx
   35d0d:	mov    r9,rdx
   35d10:	jmp    35d1b <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ebb>
   35d12:	add    rax,0x2
   35d16:	cmp    rbx,rax
   35d19:	je     35d01 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ea1>
   35d1b:	mov    ecx,DWORD PTR [r14+rax*4]
   35d1f:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   35d24:	jne    35d34 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ed4>
   35d26:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   35d2b:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   35d30:	je     35d12 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eb2>
   35d32:	jmp    35d47 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ee7>
   35d34:	mov    DWORD PTR [r14+r9*4],ecx
   35d38:	inc    r9
   35d3b:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   35d40:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   35d45:	je     35d12 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eb2>
   35d47:	mov    DWORD PTR [r14+r9*4],ecx
   35d4b:	inc    r9
   35d4e:	jmp    35d12 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eb2>
   35d50:	jmp    35d54 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ef4>
   35d52:	jmp    35d5e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1efe>
   35d54:	mov    rbx,rax
   35d57:	mov    r14,QWORD PTR [rsp+0x18]
   35d5c:	jmp    35d7e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f1e>
   35d5e:	mov    rbx,rax
   35d61:	cmp    QWORD PTR [rsp+0x178],0x0
   35d6a:	je     35d8e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f2e>
   35d6c:	mov    rax,QWORD PTR [rsp+0x180]
   35d74:	mov    QWORD PTR [rsp+0x8],rax
   35d79:	jmp    35d83 <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f23>
   35d7b:	mov    rbx,rax
   35d7e:	test   r14,r14
   35d81:	je     35d8e <_RINvNtCs5Rf2Z7g0iFY_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f2e>
   35d83:	mov    rdi,QWORD PTR [rsp+0x8]
   35d88:	call   QWORD PTR [rip+0x5feda]        # 95c68 <free@GLIBC_2.2.5>
   35d8e:	mov    rdi,rbx
   35d91:	call   91f00 <_Unwind_Resume@plt>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
