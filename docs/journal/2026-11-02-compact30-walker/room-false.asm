
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000014ed0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_>:
   14ed0:	push   rbp
   14ed1:	push   r15
   14ed3:	push   r14
   14ed5:	push   r13
   14ed7:	push   r12
   14ed9:	push   rbx
   14eda:	sub    rsp,0x638
   14ee1:	mov    DWORD PTR [rsp+0x154],esi
   14ee8:	mov    rax,QWORD PTR [rdi+0xb8]
   14eef:	mov    QWORD PTR [rsp+0x178],rax
   14ef7:	mov    eax,DWORD PTR [rdi+0x10]
   14efa:	mov    r12,QWORD PTR [rdi+0x40]
   14efe:	mov    QWORD PTR [rdi+0xb0],0x0
   14f09:	mov    QWORD PTR [rdi+0x98],0x0
   14f14:	test   r12,r12
   14f17:	je     16ee1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2011>
   14f1d:	mov    rbx,rdi
   14f20:	shl    eax,0x5
   14f23:	add    eax,0xffffffe0
   14f26:	mov    DWORD PTR [rsp+0x24],eax
   14f2a:	xor    edi,edi
   14f2c:	lea    rax,[rbx+0xa0]
   14f33:	mov    QWORD PTR [rsp],rax
   14f37:	lea    rax,[rbx+0x88]
   14f3e:	mov    QWORD PTR [rsp+0x8],rax
   14f43:	mov    QWORD PTR [rsp+0x28],r12
   14f48:	jmp    14f79 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xa9>
   14f4a:	nop    WORD PTR [rax+rax*1+0x0]
   14f50:	inc    rdi
   14f53:	mov    rax,QWORD PTR [r14+0x90]
   14f5a:	mov    WORD PTR [rax+rbx*2],bp
   14f5e:	inc    rbx
   14f61:	mov    QWORD PTR [r14+0x98],rbx
   14f68:	mov    r12,QWORD PTR [rsp+0x28]
   14f6d:	cmp    r12,rdi
   14f70:	mov    rbx,r14
   14f73:	je     150b0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e0>
   14f79:	mov    rsi,QWORD PTR [rbx+0x40]
   14f7d:	cmp    rdi,rsi
   14f80:	jae    16dd0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f00>
   14f86:	mov    rsi,QWORD PTR [rbx+0x10]
   14f8a:	mov    rax,QWORD PTR [rbx+0x38]
   14f8e:	mov    ebp,DWORD PTR [rax+rdi*4]
   14f91:	mov    r12d,ebp
   14f94:	shr    r12d,0x5
   14f98:	cmp    rsi,r12
   14f9b:	jbe    16dc1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ef1>
   14fa1:	and    ebp,0x1f
   14fa4:	cmp    ebp,0x1e
   14fa7:	jae    16dad <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1edd>
   14fad:	mov    rcx,QWORD PTR [rbx+0x8]
   14fb1:	mov    rdx,r12
   14fb4:	shl    rdx,0x6
   14fb8:	lea    rax,[rcx+rdx*1]
   14fbc:	movzx  eax,WORD PTR [rax+rbp*2]
   14fc0:	mov    r13d,DWORD PTR [rcx+rdx*1+0x3c]
   14fc5:	cmp    eax,0xfffe
   14fca:	jae    15050 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x180>
   14fd0:	mov    r14,rbx
   14fd3:	mov    ebx,r13d
   14fd6:	shr    ebx,0x8
   14fd9:	add    ebx,eax
   14fdb:	mov    r15,QWORD PTR [r14+0xb0]
   14fe2:	cmp    r15,QWORD PTR [r14+0xa0]
   14fe9:	je     1508e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1be>
   14fef:	shl    ebx,0x8
   14ff2:	movzx  eax,r13b
   14ff6:	or     eax,ebx
   14ff8:	mov    rcx,QWORD PTR [r14+0xa8]
   14fff:	mov    DWORD PTR [rcx+r15*4],eax
   15003:	inc    r15
   15006:	mov    QWORD PTR [r14+0xb0],r15
   1500d:	mov    rsi,QWORD PTR [r14+0x10]
   15011:	cmp    rsi,r12
   15014:	jbe    16d9e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ece>
   1501a:	add    rdx,QWORD PTR [r14+0x8]
   1501e:	movzx  ebp,WORD PTR [rdx+rbp*2]
   15022:	mov    rbx,QWORD PTR [r14+0x98]
   15029:	cmp    rbx,QWORD PTR [r14+0x88]
   15030:	jne    14f50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
   15036:	mov    r15,rdi
   15039:	mov    rdi,QWORD PTR [rsp+0x8]
   1503e:	call   295a0 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVectE8grow_oneCs3AlRknvjqqN_7c30walk>
   15043:	mov    rdi,r15
   15046:	jmp    14f50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
   1504b:	nop    DWORD PTR [rax+rax*1+0x0]
   15050:	and    r13d,0xfffff
   15057:	mov    rsi,QWORD PTR [rbx+0x28]
   1505b:	cmp    rsi,r13
   1505e:	jbe    16e56 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f86>
   15064:	shl    r13d,0x7
   15068:	add    r13,QWORD PTR [rbx+0x20]
   1506c:	mov    r13d,DWORD PTR [r13+rbp*4+0x0]
   15071:	mov    r14,rbx
   15074:	mov    ebx,r13d
   15077:	shr    ebx,0x8
   1507a:	mov    r15,QWORD PTR [r14+0xb0]
   15081:	cmp    r15,QWORD PTR [r14+0xa0]
   15088:	jne    14fef <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
   1508e:	mov    QWORD PTR [rsp+0x38],rdi
   15093:	mov    rdi,QWORD PTR [rsp]
   15097:	mov    QWORD PTR [rsp+0x18],rdx
   1509c:	call   33750 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCs7ivwnIpGHJL_5mibz2>
   150a1:	mov    rdx,QWORD PTR [rsp+0x18]
   150a6:	mov    rdi,QWORD PTR [rsp+0x38]
   150ab:	jmp    14fef <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
   150b0:	xor    edi,edi
   150b2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   150c0:	mov    rsi,QWORD PTR [rbx+0x40]
   150c4:	cmp    rdi,rsi
   150c7:	jae    16dff <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f2f>
   150cd:	mov    rsi,QWORD PTR [rbx+0x10]
   150d1:	mov    rax,QWORD PTR [rbx+0x38]
   150d5:	mov    eax,DWORD PTR [rax+rdi*4]
   150d8:	mov    ecx,eax
   150da:	shr    ecx,0x5
   150dd:	cmp    rsi,rcx
   150e0:	jbe    16df0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f20>
   150e6:	and    eax,0x1f
   150e9:	cmp    eax,0x1e
   150ec:	jae    16ddc <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0c>
   150f2:	inc    rdi
   150f5:	shl    rcx,0x6
   150f9:	add    rcx,QWORD PTR [rbx+0x8]
   150fd:	mov    WORD PTR [rcx+rax*2],0xfffe
   15103:	cmp    r12,rdi
   15106:	jne    150c0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0>
   15108:	mov    r14,QWORD PTR [rbx+0x8]
   1510c:	mov    rax,QWORD PTR [rbx+0x20]
   15110:	mov    QWORD PTR [rsp+0x38],rax
   15115:	mov    rax,QWORD PTR [rsp+0x178]
   1511d:	add    rax,rax
   15120:	mov    rcx,rax
   15123:	or     rcx,r12
   15126:	shr    rcx,0x20
   1512a:	je     15182 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2b2>
   1512c:	xor    edx,edx
   1512e:	div    r12
   15131:	mov    rbp,rax
   15134:	mov    r13,QWORD PTR [rbx+0x58]
   15138:	cmp    r13,r12
   1513b:	cmova  r12,r13
   1513f:	jae    15196 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2c6>
   15141:	sub    r12,r13
   15144:	mov    rax,QWORD PTR [rbx+0x48]
   15148:	sub    rax,r13
   1514b:	mov    rdx,r13
   1514e:	cmp    r12,rax
   15151:	ja     151a4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2d4>
   15153:	mov    rax,QWORD PTR [rbx+0x50]
   15157:	inc    r13
   1515a:	test   r12b,0x1
   1515e:	jne    151cf <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2ff>
   15160:	cmp    QWORD PTR [rsp+0x28],r13
   15165:	jne    151fe <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x32e>
   1516b:	mov    QWORD PTR [rbx+0x58],rdx
   1516f:	mov    rsi,QWORD PTR [rsp+0x28]
   15174:	cmp    rsi,rdx
   15177:	jbe    15265 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
   1517d:	jmp    16d6e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
   15182:	xor    edx,edx
   15184:	div    r12d
   15187:	mov    ebp,eax
   15189:	mov    r13,QWORD PTR [rbx+0x58]
   1518d:	cmp    r13,r12
   15190:	cmova  r12,r13
   15194:	jb     15141 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x271>
   15196:	mov    QWORD PTR [rbx+0x58],r12
   1519a:	mov    rsi,QWORD PTR [rsp+0x28]
   1519f:	jmp    15265 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
   151a4:	lea    rdi,[rbx+0x48]
   151a8:	mov    ecx,0x8
   151ad:	mov    r8d,0x18
   151b3:	mov    rsi,r13
   151b6:	mov    rdx,r12
   151b9:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   151be:	mov    rdx,QWORD PTR [rbx+0x58]
   151c2:	mov    rax,QWORD PTR [rbx+0x50]
   151c6:	inc    r13
   151c9:	test   r12b,0x1
   151cd:	je     15160 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x290>
   151cf:	dec    r12
   151d2:	lea    rcx,[rdx+rdx*2]
   151d6:	mov    QWORD PTR [rax+rcx*8],0x0
   151de:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   151e7:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   151f0:	inc    rdx
   151f3:	cmp    QWORD PTR [rsp+0x28],r13
   151f8:	je     1516b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x29b>
   151fe:	lea    rcx,[rdx+rdx*2]
   15202:	lea    rax,[rax+rcx*8]
   15206:	add    rax,0x28
   1520a:	neg    r12
   1520d:	xor    ecx,ecx
   1520f:	xorps  xmm0,xmm0
   15212:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   15220:	mov    QWORD PTR [rax-0x28],0x0
   15228:	mov    QWORD PTR [rax-0x20],0x1
   15230:	movups XMMWORD PTR [rax-0x18],xmm0
   15234:	mov    QWORD PTR [rax-0x8],0x1
   1523c:	mov    QWORD PTR [rax],0x0
   15243:	add    rax,0x30
   15247:	add    rcx,0xfffffffffffffffe
   1524b:	cmp    r12,rcx
   1524e:	jne    15220 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x350>
   15250:	sub    rdx,rcx
   15253:	mov    QWORD PTR [rbx+0x58],rdx
   15257:	mov    rsi,QWORD PTR [rsp+0x28]
   1525c:	cmp    rsi,rdx
   1525f:	ja     16d6e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
   15265:	lea    r12,[rbp+0x40]
   15269:	mov    r13,QWORD PTR [rbx+0x50]
   1526d:	lea    rax,[rsi*8+0x0]
   15275:	lea    r15,[rax+rax*2]
   15279:	nop    DWORD PTR [rax+0x0]
   15280:	mov    QWORD PTR [r13+0x10],0x0
   15288:	cmp    r12,QWORD PTR [r13+0x0]
   1528c:	ja     1529a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ca>
   1528e:	add    r13,0x18
   15292:	add    r15,0xffffffffffffffe8
   15296:	jne    15280 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
   15298:	jmp    152bc <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ec>
   1529a:	mov    ecx,0x1
   1529f:	mov    r8d,0x1
   152a5:	mov    rdi,r13
   152a8:	xor    esi,esi
   152aa:	mov    rdx,r12
   152ad:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   152b2:	add    r13,0x18
   152b6:	add    r15,0xffffffffffffffe8
   152ba:	jne    15280 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
   152bc:	mov    QWORD PTR [rbx+0x70],0x0
   152c4:	mov    r12,QWORD PTR [rsp+0x28]
   152c9:	cmp    r12,QWORD PTR [rbx+0x60]
   152cd:	ja     16d7c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1eac>
   152d3:	xor    r15d,r15d
   152d6:	mov    QWORD PTR [rsp+0x18],rbx
   152db:	mov    rbx,QWORD PTR [rbx+0x68]
   152df:	lea    rdi,[rbx+r15*8]
   152e3:	cmp    r12,0x1
   152e7:	je     1530e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x43e>
   152e9:	lea    rdx,[r12*8-0x8]
   152f1:	mov    esi,0xff
   152f6:	call   QWORD PTR [rip+0x65ec4]        # 7b1c0 <memset@GLIBC_2.2.5>
   152fc:	lea    rax,[r15+r12*1]
   15300:	lea    rdi,[rbx+rax*8]
   15304:	add    rdi,0xfffffffffffffff8
   15308:	add    r15,r12
   1530b:	dec    r15
   1530e:	mov    QWORD PTR [rdi],0xffffffffffffffff
   15315:	inc    r15
   15318:	mov    rbx,QWORD PTR [rsp+0x18]
   1531d:	mov    QWORD PTR [rbx+0x70],r15
   15321:	xorps  xmm0,xmm0
   15324:	movups XMMWORD PTR [rsp+0x240],xmm0
   1532c:	mov    QWORD PTR [rsp+0x250],0x0
   15338:	movups XMMWORD PTR [rsp+0x260],xmm0
   15340:	mov    QWORD PTR [rsp+0x270],0x0
   1534c:	movups XMMWORD PTR [rsp+0x280],xmm0
   15354:	mov    QWORD PTR [rsp+0x290],0x0
   15360:	movups XMMWORD PTR [rsp+0x2a0],xmm0
   15368:	mov    QWORD PTR [rsp+0x2b0],0x0
   15374:	movups XMMWORD PTR [rsp+0x2c0],xmm0
   1537c:	mov    QWORD PTR [rsp+0x2d0],0x0
   15388:	movups XMMWORD PTR [rsp+0x2e0],xmm0
   15390:	mov    QWORD PTR [rsp+0x2f0],0x0
   1539c:	movups XMMWORD PTR [rsp+0x300],xmm0
   153a4:	mov    QWORD PTR [rsp+0x310],0x0
   153b0:	movups XMMWORD PTR [rsp+0x320],xmm0
   153b8:	mov    QWORD PTR [rsp+0x330],0x0
   153c4:	movups XMMWORD PTR [rsp+0x340],xmm0
   153cc:	mov    QWORD PTR [rsp+0x350],0x0
   153d8:	movups XMMWORD PTR [rsp+0x360],xmm0
   153e0:	mov    QWORD PTR [rsp+0x370],0x0
   153ec:	movups XMMWORD PTR [rsp+0x380],xmm0
   153f4:	mov    QWORD PTR [rsp+0x390],0x0
   15400:	movups XMMWORD PTR [rsp+0x3a0],xmm0
   15408:	mov    QWORD PTR [rsp+0x3b0],0x0
   15414:	movups XMMWORD PTR [rsp+0x3c0],xmm0
   1541c:	mov    QWORD PTR [rsp+0x3d0],0x0
   15428:	movups XMMWORD PTR [rsp+0x3e0],xmm0
   15430:	mov    QWORD PTR [rsp+0x3f0],0x0
   1543c:	movups XMMWORD PTR [rsp+0x400],xmm0
   15444:	mov    QWORD PTR [rsp+0x410],0x0
   15450:	movups XMMWORD PTR [rsp+0x420],xmm0
   15458:	mov    QWORD PTR [rsp+0x430],0x0
   15464:	movups XMMWORD PTR [rsp+0x440],xmm0
   1546c:	mov    QWORD PTR [rsp+0x450],0x0
   15478:	movups XMMWORD PTR [rsp+0x460],xmm0
   15480:	mov    QWORD PTR [rsp+0x470],0x0
   1548c:	movups XMMWORD PTR [rsp+0x480],xmm0
   15494:	mov    QWORD PTR [rsp+0x490],0x0
   154a0:	movups XMMWORD PTR [rsp+0x4a0],xmm0
   154a8:	mov    QWORD PTR [rsp+0x4b0],0x0
   154b4:	movups XMMWORD PTR [rsp+0x4c0],xmm0
   154bc:	mov    QWORD PTR [rsp+0x4d0],0x0
   154c8:	movups XMMWORD PTR [rsp+0x4e0],xmm0
   154d0:	mov    QWORD PTR [rsp+0x4f0],0x0
   154dc:	movups XMMWORD PTR [rsp+0x500],xmm0
   154e4:	mov    QWORD PTR [rsp+0x510],0x0
   154f0:	movups XMMWORD PTR [rsp+0x520],xmm0
   154f8:	mov    QWORD PTR [rsp+0x530],0x0
   15504:	movups XMMWORD PTR [rsp+0x540],xmm0
   1550c:	mov    QWORD PTR [rsp+0x550],0x0
   15518:	movups XMMWORD PTR [rsp+0x560],xmm0
   15520:	mov    QWORD PTR [rsp+0x570],0x0
   1552c:	movups XMMWORD PTR [rsp+0x580],xmm0
   15534:	mov    QWORD PTR [rsp+0x590],0x0
   15540:	movups XMMWORD PTR [rsp+0x5a0],xmm0
   15548:	mov    QWORD PTR [rsp+0x5b0],0x0
   15554:	movups XMMWORD PTR [rsp+0x5c0],xmm0
   1555c:	mov    QWORD PTR [rsp+0x5d0],0x0
   15568:	movups XMMWORD PTR [rsp+0x5e0],xmm0
   15570:	mov    QWORD PTR [rsp+0x5f0],0x0
   1557c:	movups XMMWORD PTR [rsp+0x600],xmm0
   15584:	mov    QWORD PTR [rsp+0x610],0x0
   15590:	movups XMMWORD PTR [rsp+0x620],xmm0
   15598:	mov    QWORD PTR [rsp+0x630],0x0
   155a4:	mov    QWORD PTR [rsp+0x238],0xffffffffffffffff
   155b0:	mov    QWORD PTR [rsp+0x258],0xffffffffffffffff
   155bc:	mov    QWORD PTR [rsp+0x278],0xffffffffffffffff
   155c8:	mov    QWORD PTR [rsp+0x298],0xffffffffffffffff
   155d4:	mov    QWORD PTR [rsp+0x2b8],0xffffffffffffffff
   155e0:	mov    QWORD PTR [rsp+0x2d8],0xffffffffffffffff
   155ec:	mov    QWORD PTR [rsp+0x2f8],0xffffffffffffffff
   155f8:	mov    QWORD PTR [rsp+0x318],0xffffffffffffffff
   15604:	mov    QWORD PTR [rsp+0x338],0xffffffffffffffff
   15610:	mov    QWORD PTR [rsp+0x358],0xffffffffffffffff
   1561c:	mov    QWORD PTR [rsp+0x378],0xffffffffffffffff
   15628:	mov    QWORD PTR [rsp+0x398],0xffffffffffffffff
   15634:	mov    QWORD PTR [rsp+0x3b8],0xffffffffffffffff
   15640:	mov    QWORD PTR [rsp+0x3d8],0xffffffffffffffff
   1564c:	mov    QWORD PTR [rsp+0x3f8],0xffffffffffffffff
   15658:	mov    QWORD PTR [rsp+0x418],0xffffffffffffffff
   15664:	mov    QWORD PTR [rsp+0x438],0xffffffffffffffff
   15670:	mov    QWORD PTR [rsp+0x458],0xffffffffffffffff
   1567c:	mov    QWORD PTR [rsp+0x478],0xffffffffffffffff
   15688:	mov    QWORD PTR [rsp+0x498],0xffffffffffffffff
   15694:	mov    QWORD PTR [rsp+0x4b8],0xffffffffffffffff
   156a0:	mov    QWORD PTR [rsp+0x4d8],0xffffffffffffffff
   156ac:	mov    QWORD PTR [rsp+0x4f8],0xffffffffffffffff
   156b8:	mov    QWORD PTR [rsp+0x518],0xffffffffffffffff
   156c4:	mov    QWORD PTR [rsp+0x538],0xffffffffffffffff
   156d0:	mov    QWORD PTR [rsp+0x558],0xffffffffffffffff
   156dc:	mov    QWORD PTR [rsp+0x578],0xffffffffffffffff
   156e8:	mov    QWORD PTR [rsp+0x598],0xffffffffffffffff
   156f4:	mov    QWORD PTR [rsp+0x5b8],0xffffffffffffffff
   15700:	mov    QWORD PTR [rsp+0x5d8],0xffffffffffffffff
   1570c:	mov    QWORD PTR [rsp+0x5f8],0xffffffffffffffff
   15718:	mov    QWORD PTR [rsp+0x618],0xffffffffffffffff
   15724:	mov    eax,0x2
   15729:	xor    edi,edi
   1572b:	xor    r9d,r9d
   1572e:	xchg   ax,ax
   15730:	mov    rsi,QWORD PTR [rbx+0x58]
   15734:	cmp    rdi,rsi
   15737:	jae    16e17 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f47>
   1573d:	mov    rsi,QWORD PTR [rbx+0xb0]
   15744:	cmp    rdi,rsi
   15747:	jae    16e0b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f3b>
   1574d:	mov    rcx,QWORD PTR [rbx+0x50]
   15751:	mov    rdx,QWORD PTR [rbx+0xa8]
   15758:	mov    rsi,QWORD PTR [rcx+rax*4]
   1575c:	movzx  r8d,BYTE PTR [rdx+rdi*4]
   15761:	mov    BYTE PTR [rsi],r8b
   15764:	mov    edx,DWORD PTR [rdx+rdi*4]
   15767:	shr    edx,0x8
   1576a:	lea    r8,[rsi+0x1]
   1576e:	add    rsi,QWORD PTR [rcx+rax*4-0x8]
   15773:	mov    QWORD PTR [rsp+r9*1+0x238],rdi
   1577b:	mov    QWORD PTR [rsp+r9*1+0x240],r8
   15783:	mov    QWORD PTR [rsp+r9*1+0x248],rsi
   1578b:	mov    DWORD PTR [rsp+r9*1+0x250],edx
   15793:	mov    DWORD PTR [rsp+r9*1+0x254],0x0
   1579f:	prefetcht0 BYTE PTR [r14+rdx*2]
   157a4:	lea    rsi,[rdi+0x1]
   157a8:	add    r9,0x20
   157ac:	cmp    rdi,0x1e
   157b0:	ja     157c2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x8f2>
   157b2:	add    rax,0x6
   157b6:	mov    rdi,rsi
   157b9:	cmp    rsi,r12
   157bc:	jb     15730 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x860>
   157c2:	cmp    rsi,0x20
   157c6:	jne    167a2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
   157cc:	mov    QWORD PTR [rsp+0x170],rsi
   157d4:	add    rbp,0x3f
   157d8:	mov    eax,0x20
   157dd:	mov    QWORD PTR [rsp+0x8],rax
   157e2:	mov    QWORD PTR [rsp+0x230],rbp
   157ea:	jmp    15b7d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcad>
   157ef:	nop
   157f0:	mov    rbp,QWORD PTR [rsp+0x48]
   157f5:	mov    r12,QWORD PTR [rsp+0x50]
   157fa:	mov    rax,QWORD PTR [rsp+0x58]
   157ff:	mov    QWORD PTR [rsp+0x10],rax
   15804:	mov    rax,QWORD PTR [rsp+0x60]
   15809:	mov    QWORD PTR [rsp+0x1b8],rax
   15811:	mov    rax,QWORD PTR [rsp+0x68]
   15816:	mov    QWORD PTR [rsp+0x1c0],rax
   1581e:	mov    rax,QWORD PTR [rsp+0x70]
   15823:	mov    QWORD PTR [rsp+0x1c8],rax
   1582b:	mov    rax,QWORD PTR [rsp+0x78]
   15830:	mov    QWORD PTR [rsp+0x1d0],rax
   15838:	mov    rax,QWORD PTR [rsp+0x80]
   15840:	mov    QWORD PTR [rsp+0x1d8],rax
   15848:	mov    rax,QWORD PTR [rsp+0x88]
   15850:	mov    QWORD PTR [rsp+0x1e0],rax
   15858:	mov    rax,QWORD PTR [rsp+0x90]
   15860:	mov    QWORD PTR [rsp+0x1e8],rax
   15868:	mov    rax,QWORD PTR [rsp+0x98]
   15870:	mov    QWORD PTR [rsp+0x1f0],rax
   15878:	mov    rax,QWORD PTR [rsp+0xa0]
   15880:	mov    QWORD PTR [rsp+0x1f8],rax
   15888:	mov    rax,QWORD PTR [rsp+0xa8]
   15890:	mov    QWORD PTR [rsp+0x200],rax
   15898:	mov    rax,QWORD PTR [rsp+0xb0]
   158a0:	mov    QWORD PTR [rsp+0x208],rax
   158a8:	mov    rax,QWORD PTR [rsp+0xb8]
   158b0:	mov    QWORD PTR [rsp],rax
   158b4:	mov    rax,QWORD PTR [rsp+0xc0]
   158bc:	mov    QWORD PTR [rsp+0x190],rax
   158c4:	mov    rax,QWORD PTR [rsp+0xc8]
   158cc:	mov    QWORD PTR [rsp+0x198],rax
   158d4:	mov    rax,QWORD PTR [rsp+0xd0]
   158dc:	mov    QWORD PTR [rsp+0x1a0],rax
   158e4:	mov    rax,QWORD PTR [rsp+0xd8]
   158ec:	mov    QWORD PTR [rsp+0x1a8],rax
   158f4:	mov    rax,QWORD PTR [rsp+0xe0]
   158fc:	mov    QWORD PTR [rsp+0x1b0],rax
   15904:	mov    rax,QWORD PTR [rsp+0xe8]
   1590c:	mov    QWORD PTR [rsp+0x180],rax
   15914:	mov    rax,QWORD PTR [rsp+0xf0]
   1591c:	mov    QWORD PTR [rsp+0x188],rax
   15924:	mov    rdi,QWORD PTR [rsp+0xf8]
   1592c:	mov    rsi,QWORD PTR [rsp+0x100]
   15934:	mov    rdx,QWORD PTR [rsp+0x108]
   1593c:	mov    rcx,QWORD PTR [rsp+0x110]
   15944:	mov    rax,QWORD PTR [rsp+0x118]
   1594c:	mov    r13,QWORD PTR [rsp+0x120]
   15954:	mov    r11,QWORD PTR [rsp+0x128]
   1595c:	mov    r10,QWORD PTR [rsp+0x130]
   15964:	mov    r9,QWORD PTR [rsp+0x138]
   1596c:	mov    r8,QWORD PTR [rsp+0x140]
   15974:	add    rbp,r15
   15977:	mov    QWORD PTR [rsp+0x240],rbp
   1597f:	add    r12,r15
   15982:	mov    QWORD PTR [rsp+0x260],r12
   1598a:	mov    r12,QWORD PTR [rsp+0x10]
   1598f:	add    r12,r15
   15992:	mov    QWORD PTR [rsp+0x280],r12
   1599a:	mov    r12,QWORD PTR [rsp+0x1b8]
   159a2:	add    r12,r15
   159a5:	mov    QWORD PTR [rsp+0x2a0],r12
   159ad:	mov    r12,QWORD PTR [rsp+0x1c0]
   159b5:	add    r12,r15
   159b8:	mov    QWORD PTR [rsp+0x2c0],r12
   159c0:	mov    r12,QWORD PTR [rsp+0x1c8]
   159c8:	add    r12,r15
   159cb:	mov    QWORD PTR [rsp+0x2e0],r12
   159d3:	mov    r12,QWORD PTR [rsp+0x1d0]
   159db:	add    r12,r15
   159de:	mov    QWORD PTR [rsp+0x300],r12
   159e6:	mov    r12,QWORD PTR [rsp+0x1d8]
   159ee:	add    r12,r15
   159f1:	mov    QWORD PTR [rsp+0x320],r12
   159f9:	mov    r12,QWORD PTR [rsp+0x1e0]
   15a01:	add    r12,r15
   15a04:	mov    QWORD PTR [rsp+0x340],r12
   15a0c:	mov    r12,QWORD PTR [rsp+0x1e8]
   15a14:	add    r12,r15
   15a17:	mov    QWORD PTR [rsp+0x360],r12
   15a1f:	mov    r12,QWORD PTR [rsp+0x1f0]
   15a27:	add    r12,r15
   15a2a:	mov    QWORD PTR [rsp+0x380],r12
   15a32:	mov    r12,QWORD PTR [rsp+0x1f8]
   15a3a:	add    r12,r15
   15a3d:	mov    QWORD PTR [rsp+0x3a0],r12
   15a45:	mov    r12,QWORD PTR [rsp+0x200]
   15a4d:	add    r12,r15
   15a50:	mov    QWORD PTR [rsp+0x3c0],r12
   15a58:	mov    r12,QWORD PTR [rsp+0x208]
   15a60:	add    r12,r15
   15a63:	mov    QWORD PTR [rsp+0x3e0],r12
   15a6b:	mov    r12,QWORD PTR [rsp]
   15a6f:	add    r12,r15
   15a72:	mov    QWORD PTR [rsp+0x400],r12
   15a7a:	mov    r12,QWORD PTR [rsp+0x190]
   15a82:	add    r12,r15
   15a85:	mov    QWORD PTR [rsp+0x420],r12
   15a8d:	mov    r12,QWORD PTR [rsp+0x198]
   15a95:	add    r12,r15
   15a98:	mov    QWORD PTR [rsp+0x440],r12
   15aa0:	mov    r12,QWORD PTR [rsp+0x1a0]
   15aa8:	add    r12,r15
   15aab:	mov    QWORD PTR [rsp+0x460],r12
   15ab3:	mov    r12,QWORD PTR [rsp+0x1a8]
   15abb:	add    r12,r15
   15abe:	mov    QWORD PTR [rsp+0x480],r12
   15ac6:	mov    r12,QWORD PTR [rsp+0x1b0]
   15ace:	add    r12,r15
   15ad1:	mov    QWORD PTR [rsp+0x4a0],r12
   15ad9:	mov    r12,QWORD PTR [rsp+0x180]
   15ae1:	add    r12,r15
   15ae4:	mov    QWORD PTR [rsp+0x4c0],r12
   15aec:	mov    r12,QWORD PTR [rsp+0x188]
   15af4:	add    r12,r15
   15af7:	mov    QWORD PTR [rsp+0x4e0],r12
   15aff:	add    rdi,r15
   15b02:	mov    QWORD PTR [rsp+0x500],rdi
   15b0a:	add    rsi,r15
   15b0d:	mov    QWORD PTR [rsp+0x520],rsi
   15b15:	add    rdx,r15
   15b18:	mov    QWORD PTR [rsp+0x540],rdx
   15b20:	add    rcx,r15
   15b23:	mov    QWORD PTR [rsp+0x560],rcx
   15b2b:	add    rax,r15
   15b2e:	mov    QWORD PTR [rsp+0x580],rax
   15b36:	add    r13,r15
   15b39:	mov    QWORD PTR [rsp+0x5a0],r13
   15b41:	add    r11,r15
   15b44:	mov    QWORD PTR [rsp+0x5c0],r11
   15b4c:	add    r10,r15
   15b4f:	mov    QWORD PTR [rsp+0x5e0],r10
   15b57:	add    r9,r15
   15b5a:	mov    QWORD PTR [rsp+0x600],r9
   15b62:	add    r8,r15
   15b65:	mov    QWORD PTR [rsp+0x620],r8
   15b6d:	mov    r9,QWORD PTR [rsp+0x148]
   15b75:	mov    rbp,QWORD PTR [rsp+0x230]
   15b7d:	mov    r15,rbp
   15b80:	xor    r12d,r12d
   15b83:	jmp    15bc1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcf1>
   15b85:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   15b90:	add    r13,rax
   15b93:	mov    QWORD PTR [rsp+r12*1+0x240],r13
   15b9b:	add    rdx,rax
   15b9e:	mov    QWORD PTR [rsp+r12*1+0x248],rdx
   15ba6:	sub    rdx,r13
   15ba9:	add    r12,0x20
   15bad:	cmp    rdx,r15
   15bb0:	cmovb  r15,rdx
   15bb4:	cmp    r12,0x400
   15bbb:	je     15c40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xd70>
   15bc1:	mov    r13,QWORD PTR [rsp+r12*1+0x240]
   15bc9:	mov    rdx,QWORD PTR [rsp+r12*1+0x248]
   15bd1:	cmp    r13,rdx
   15bd4:	jne    15ba6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcd6>
   15bd6:	mov    rsi,QWORD PTR [rbx+0x58]
   15bda:	mov    rdi,QWORD PTR [rsp+r12*1+0x238]
   15be2:	cmp    rdi,rsi
   15be5:	jae    16e4a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f7a>
   15beb:	mov    rcx,QWORD PTR [rbx+0x50]
   15bef:	lea    rsi,[rdi+rdi*2]
   15bf3:	mov    rdx,QWORD PTR [rcx+rsi*8]
   15bf7:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   15bfc:	sub    r13,rax
   15bff:	mov    QWORD PTR [rcx+rsi*8+0x10],r13
   15c04:	mov    rdi,rdx
   15c07:	sub    rdi,r13
   15c0a:	cmp    rdx,rdi
   15c0d:	jbe    15b90 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
   15c0f:	lea    rdi,[rcx+rsi*8]
   15c13:	mov    QWORD PTR [rsp],rdi
   15c17:	mov    ecx,0x1
   15c1c:	mov    r8d,0x1
   15c22:	mov    rsi,r13
   15c25:	mov    rbp,r9
   15c28:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   15c2d:	mov    r9,rbp
   15c30:	mov    rax,QWORD PTR [rsp]
   15c34:	mov    rdx,QWORD PTR [rax]
   15c37:	mov    rax,QWORD PTR [rax+0x8]
   15c3b:	jmp    15b90 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
   15c40:	mov    QWORD PTR [rsp+0x148],r9
   15c48:	mov    rax,QWORD PTR [rsp+0x240]
   15c50:	mov    QWORD PTR [rsp+0x30],rax
   15c55:	mov    rax,QWORD PTR [rsp+0x260]
   15c5d:	mov    QWORD PTR [rsp+0x40],rax
   15c62:	mov    rcx,QWORD PTR [rsp+0x280]
   15c6a:	mov    QWORD PTR [rsp+0x10],rcx
   15c6f:	mov    r12,QWORD PTR [rsp+0x2a0]
   15c77:	mov    rbp,QWORD PTR [rsp+0x2c0]
   15c7f:	mov    r13,QWORD PTR [rsp+0x2e0]
   15c87:	mov    r11,QWORD PTR [rsp+0x300]
   15c8f:	mov    r10,QWORD PTR [rsp+0x320]
   15c97:	mov    r9,QWORD PTR [rsp+0x340]
   15c9f:	mov    r8,QWORD PTR [rsp+0x360]
   15ca7:	mov    rdi,QWORD PTR [rsp+0x380]
   15caf:	mov    rsi,QWORD PTR [rsp+0x3a0]
   15cb7:	mov    rdx,QWORD PTR [rsp+0x3c0]
   15cbf:	mov    rcx,QWORD PTR [rsp+0x3e0]
   15cc7:	mov    rax,QWORD PTR [rsp+0x400]
   15ccf:	mov    QWORD PTR [rsp],rax
   15cd3:	mov    rax,QWORD PTR [rsp+0x30]
   15cd8:	mov    QWORD PTR [rsp+0x48],rax
   15cdd:	mov    rax,QWORD PTR [rsp+0x40]
   15ce2:	mov    QWORD PTR [rsp+0x50],rax
   15ce7:	mov    rax,QWORD PTR [rsp+0x10]
   15cec:	mov    QWORD PTR [rsp+0x58],rax
   15cf1:	mov    QWORD PTR [rsp+0x1b8],r12
   15cf9:	mov    QWORD PTR [rsp+0x60],r12
   15cfe:	mov    QWORD PTR [rsp+0x1c0],rbp
   15d06:	mov    QWORD PTR [rsp+0x68],rbp
   15d0b:	mov    QWORD PTR [rsp+0x1c8],r13
   15d13:	mov    QWORD PTR [rsp+0x70],r13
   15d18:	mov    QWORD PTR [rsp+0x1d0],r11
   15d20:	mov    QWORD PTR [rsp+0x78],r11
   15d25:	mov    QWORD PTR [rsp+0x1d8],r10
   15d2d:	mov    QWORD PTR [rsp+0x80],r10
   15d35:	mov    QWORD PTR [rsp+0x1e0],r9
   15d3d:	mov    QWORD PTR [rsp+0x88],r9
   15d45:	mov    QWORD PTR [rsp+0x1e8],r8
   15d4d:	mov    QWORD PTR [rsp+0x90],r8
   15d55:	mov    QWORD PTR [rsp+0x1f0],rdi
   15d5d:	mov    QWORD PTR [rsp+0x98],rdi
   15d65:	mov    QWORD PTR [rsp+0x1f8],rsi
   15d6d:	mov    QWORD PTR [rsp+0xa0],rsi
   15d75:	mov    QWORD PTR [rsp+0x200],rdx
   15d7d:	mov    QWORD PTR [rsp+0xa8],rdx
   15d85:	mov    QWORD PTR [rsp+0x208],rcx
   15d8d:	mov    QWORD PTR [rsp+0xb0],rcx
   15d95:	mov    rax,QWORD PTR [rsp]
   15d99:	mov    QWORD PTR [rsp+0xb8],rax
   15da1:	mov    rax,QWORD PTR [rsp+0x420]
   15da9:	mov    QWORD PTR [rsp+0x190],rax
   15db1:	mov    QWORD PTR [rsp+0xc0],rax
   15db9:	mov    rax,QWORD PTR [rsp+0x440]
   15dc1:	mov    QWORD PTR [rsp+0x198],rax
   15dc9:	mov    QWORD PTR [rsp+0xc8],rax
   15dd1:	mov    rax,QWORD PTR [rsp+0x460]
   15dd9:	mov    QWORD PTR [rsp+0x1a0],rax
   15de1:	mov    QWORD PTR [rsp+0xd0],rax
   15de9:	mov    rax,QWORD PTR [rsp+0x480]
   15df1:	mov    QWORD PTR [rsp+0x1a8],rax
   15df9:	mov    QWORD PTR [rsp+0xd8],rax
   15e01:	mov    rax,QWORD PTR [rsp+0x4a0]
   15e09:	mov    QWORD PTR [rsp+0x1b0],rax
   15e11:	mov    QWORD PTR [rsp+0xe0],rax
   15e19:	mov    rbp,QWORD PTR [rsp+0x4c0]
   15e21:	mov    QWORD PTR [rsp+0xe8],rbp
   15e29:	mov    r12,QWORD PTR [rsp+0x4e0]
   15e31:	mov    QWORD PTR [rsp+0xf0],r12
   15e39:	mov    rdi,QWORD PTR [rsp+0x500]
   15e41:	mov    QWORD PTR [rsp+0xf8],rdi
   15e49:	mov    rsi,QWORD PTR [rsp+0x520]
   15e51:	mov    QWORD PTR [rsp+0x100],rsi
   15e59:	mov    rdx,QWORD PTR [rsp+0x540]
   15e61:	mov    QWORD PTR [rsp+0x108],rdx
   15e69:	mov    rcx,QWORD PTR [rsp+0x560]
   15e71:	mov    QWORD PTR [rsp+0x110],rcx
   15e79:	mov    rax,QWORD PTR [rsp+0x580]
   15e81:	mov    QWORD PTR [rsp+0x118],rax
   15e89:	mov    r13,QWORD PTR [rsp+0x5a0]
   15e91:	mov    QWORD PTR [rsp+0x120],r13
   15e99:	mov    r11,QWORD PTR [rsp+0x5c0]
   15ea1:	mov    QWORD PTR [rsp+0x128],r11
   15ea9:	mov    r10,QWORD PTR [rsp+0x5e0]
   15eb1:	mov    QWORD PTR [rsp+0x130],r10
   15eb9:	mov    r9,QWORD PTR [rsp+0x600]
   15ec1:	mov    QWORD PTR [rsp+0x138],r9
   15ec9:	mov    r8,QWORD PTR [rsp+0x620]
   15ed1:	mov    QWORD PTR [rsp+0x140],r8
   15ed9:	test   r15,r15
   15edc:	je     16160 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1290>
   15ee2:	xor    eax,eax
   15ee4:	mov    ebp,DWORD PTR [rsp+0x24]
   15ee8:	mov    r13,QWORD PTR [rsp+0x38]
   15eed:	lea    r12,[rsp+0x250]
   15ef5:	jmp    15f0d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x103d>
   15ef7:	nop    WORD PTR [rax+rax*1+0x0]
   15f00:	mov    rax,QWORD PTR [rsp]
   15f04:	cmp    rax,r15
   15f07:	je     157f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x920>
   15f0d:	lea    rcx,[rax+0x1]
   15f11:	mov    QWORD PTR [rsp],rcx
   15f15:	mov    r8,rax
   15f18:	not    r8
   15f1b:	xor    r9d,r9d
   15f1e:	mov    r10,r9
   15f21:	shl    r10,0x5
   15f25:	add    r10,r12
   15f28:	mov    rdx,r9
   15f2b:	jmp    15f5b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x108b>
   15f2d:	nop    DWORD PTR [rax]
   15f30:	mov    ecx,esi
   15f32:	shr    ecx,0x8
   15f35:	add    ecx,edi
   15f37:	mov    DWORD PTR [r10],ecx
   15f3a:	mov    ecx,ecx
   15f3c:	prefetcht0 BYTE PTR [r14+rcx*2]
   15f41:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
   15f46:	mov    BYTE PTR [rcx+rax*1],sil
   15f4a:	lea    rcx,[r9-0x1]
   15f4e:	add    r10,0x20
   15f52:	mov    rdx,r9
   15f55:	cmp    rcx,0x1f
   15f59:	jae    15f00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   15f5b:	lea    r9,[rdx+0x1]
   15f5f:	mov    r11d,DWORD PTR [r10]
   15f62:	movzx  edi,WORD PTR [r14+r11*2]
   15f67:	mov    ecx,r11d
   15f6a:	and    ecx,0xffffffe0
   15f6d:	mov    esi,DWORD PTR [r14+rcx*2+0x3c]
   15f72:	cmp    edi,0xfffe
   15f78:	jb     15f30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1060>
   15f7a:	je     15fa0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x10d0>
   15f7c:	cmp    r11d,ebp
   15f7f:	jne    16060 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1190>
   15f85:	mov    ecx,DWORD PTR [r10+0x4]
   15f89:	mov    esi,DWORD PTR [r13+rcx*4+0x0]
   15f8e:	mov    ecx,esi
   15f90:	shr    ecx,0x8
   15f93:	jmp    15f37 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1067>
   15f95:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   15fa0:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
   15fa5:	add    rcx,rax
   15fa8:	mov    QWORD PTR [r10-0x10],rcx
   15fac:	mov    rsi,QWORD PTR [rbx+0x58]
   15fb0:	mov    rdi,QWORD PTR [r10-0x18]
   15fb4:	cmp    rdi,rsi
   15fb7:	jae    16e71 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fa1>
   15fbd:	mov    rsi,QWORD PTR [rsp+0x18]
   15fc2:	mov    rbx,QWORD PTR [rsi+0x50]
   15fc6:	mov    rsi,QWORD PTR [rsp+0x18]
   15fcb:	mov    r13,QWORD PTR [rsi+0x68]
   15fcf:	lea    r12,[rdi+rdi*2]
   15fd3:	sub    rcx,QWORD PTR [rbx+r12*8+0x8]
   15fd8:	mov    rsi,QWORD PTR [rsp+0x18]
   15fdd:	mov    rsi,QWORD PTR [rsi+0x70]
   15fe1:	mov    QWORD PTR [rbx+r12*8+0x10],rcx
   15fe6:	mov    rbx,QWORD PTR [rsp+0x18]
   15feb:	mov    rbp,QWORD PTR [rbx+0x40]
   15fef:	test   rbp,rbp
   15ff2:	mov    QWORD PTR [rsp+0x10],r13
   15ff7:	je     16091 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11c1>
   15ffd:	mov    rcx,QWORD PTR [rbx+0x38]
   16001:	xor    r13d,r13d
   16004:	cmp    rbp,0x1
   16008:	je     1602d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x115d>
   1600a:	nop    WORD PTR [rax+rax*1+0x0]
   16010:	mov    rbx,r13
   16013:	mov    r12,rbp
   16016:	shr    r12,1
   16019:	add    r13,r12
   1601c:	cmp    DWORD PTR [rcx+r13*4],r11d
   16020:	cmova  r13,rbx
   16024:	sub    rbp,r12
   16027:	cmp    rbp,0x1
   1602b:	ja     16010 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1140>
   1602d:	cmp    DWORD PTR [rcx+r13*4],r11d
   16031:	mov    rcx,0xffffffffffffffff
   16038:	cmovne r13,rcx
   1603c:	mov    rbx,QWORD PTR [rsp+0x18]
   16041:	cmp    rdi,rsi
   16044:	mov    rcx,QWORD PTR [rsp+0x28]
   16049:	mov    rbp,QWORD PTR [rsp+0x8]
   1604e:	jb     160ab <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11db>
   16050:	jmp    16e65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
   16055:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   16060:	shl    esi,0x5
   16063:	and    esi,0x1ffffe0
   16069:	and    r11d,0x1f
   1606d:	or     r11d,esi
   16070:	mov    DWORD PTR [r10+0x4],r11d
   16074:	prefetcht0 BYTE PTR [r13+r11*4+0x0]
   1607a:	mov    DWORD PTR [r10],ebp
   1607d:	dec    QWORD PTR [rsp+rdx*8+0x48]
   16082:	cmp    rdx,0x1f
   16086:	jb     15f1e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
   1608c:	jmp    15f00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   16091:	mov    r13,0xffffffffffffffff
   16098:	cmp    rdi,rsi
   1609b:	mov    rcx,QWORD PTR [rsp+0x28]
   160a0:	mov    rbp,QWORD PTR [rsp+0x8]
   160a5:	jae    16e65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
   160ab:	mov    rsi,QWORD PTR [rsp+0x10]
   160b0:	mov    QWORD PTR [rsi+rdi*8],r13
   160b4:	cmp    rbp,rcx
   160b7:	je     1617f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x12af>
   160bd:	mov    rsi,QWORD PTR [rbx+0x58]
   160c1:	cmp    rbp,rsi
   160c4:	mov    r13,QWORD PTR [rsp+0x38]
   160c9:	jae    16e8c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fbc>
   160cf:	mov    rsi,QWORD PTR [rbx+0xb0]
   160d6:	cmp    rbp,rsi
   160d9:	jae    16e7d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fad>
   160df:	mov    rcx,QWORD PTR [rbx+0x50]
   160e3:	mov    rsi,QWORD PTR [rbx+0xa8]
   160ea:	lea    rdi,[rbp*2+0x0]
   160f2:	add    rdi,rbp
   160f5:	mov    r11,QWORD PTR [rcx+rdi*8+0x8]
   160fa:	movzx  ebx,BYTE PTR [rsi+rbp*4]
   160fe:	mov    BYTE PTR [r11],bl
   16101:	mov    esi,DWORD PTR [rsi+rbp*4]
   16104:	shr    esi,0x8
   16107:	lea    rbx,[r11+0x1]
   1610b:	lea    r12,[r11+r8*1]
   1610f:	inc    r12
   16112:	add    r11,QWORD PTR [rcx+rdi*8]
   16116:	mov    QWORD PTR [r10-0x18],rbp
   1611a:	mov    QWORD PTR [r10-0x10],rbx
   1611e:	mov    rbx,QWORD PTR [rsp+0x18]
   16123:	mov    QWORD PTR [r10-0x8],r11
   16127:	mov    DWORD PTR [r10],esi
   1612a:	mov    DWORD PTR [r10+0x4],0x0
   16132:	prefetcht0 BYTE PTR [r14+rsi*2]
   16137:	inc    rbp
   1613a:	mov    QWORD PTR [rsp+0x8],rbp
   1613f:	mov    QWORD PTR [rsp+rdx*8+0x48],r12
   16144:	mov    ebp,DWORD PTR [rsp+0x24]
   16148:	lea    r12,[rsp+0x250]
   16150:	cmp    rdx,0x1f
   16154:	jb     15f1e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
   1615a:	jmp    15f00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   1615f:	nop
   16160:	mov    QWORD PTR [rsp+0x188],r12
   16168:	mov    r12,QWORD PTR [rsp+0x40]
   1616d:	mov    QWORD PTR [rsp+0x180],rbp
   16175:	mov    rbp,QWORD PTR [rsp+0x30]
   1617a:	jmp    15974 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xaa4>
   1617f:	test   rdx,rdx
   16182:	je     1655a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x168a>
   16188:	mov    rcx,QWORD PTR [rsp+0x48]
   1618d:	lea    rdi,[rax+0x1]
   16191:	add    rcx,rdi
   16194:	mov    QWORD PTR [rsp+0x240],rcx
   1619c:	cmp    rdx,0x2
   161a0:	mov    r9,QWORD PTR [rsp+0x148]
   161a8:	mov    rsi,QWORD PTR [rsp+0x170]
   161b0:	jb     1657a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16aa>
   161b6:	mov    rcx,QWORD PTR [rsp+0x50]
   161bb:	add    rcx,rdi
   161be:	mov    QWORD PTR [rsp+0x260],rcx
   161c6:	cmp    rdx,0x2
   161ca:	jbe    1658a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ba>
   161d0:	mov    rcx,QWORD PTR [rsp+0x58]
   161d5:	add    rcx,rdi
   161d8:	mov    QWORD PTR [rsp+0x280],rcx
   161e0:	cmp    rdx,0x3
   161e4:	jbe    1659a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ca>
   161ea:	mov    rcx,QWORD PTR [rsp+0x60]
   161ef:	add    rcx,rdi
   161f2:	mov    QWORD PTR [rsp+0x2a0],rcx
   161fa:	cmp    rdx,0x4
   161fe:	jbe    165aa <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16da>
   16204:	mov    rcx,QWORD PTR [rsp+0x68]
   16209:	add    rcx,rdi
   1620c:	mov    QWORD PTR [rsp+0x2c0],rcx
   16214:	cmp    rdx,0x5
   16218:	jbe    165ba <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ea>
   1621e:	mov    rcx,QWORD PTR [rsp+0x70]
   16223:	add    rcx,rdi
   16226:	mov    QWORD PTR [rsp+0x2e0],rcx
   1622e:	cmp    rdx,0x6
   16232:	jbe    165ca <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16fa>
   16238:	mov    rcx,QWORD PTR [rsp+0x78]
   1623d:	add    rcx,rdi
   16240:	mov    QWORD PTR [rsp+0x300],rcx
   16248:	cmp    rdx,0x7
   1624c:	jbe    165dd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x170d>
   16252:	mov    rcx,QWORD PTR [rsp+0x80]
   1625a:	add    rcx,rdi
   1625d:	mov    QWORD PTR [rsp+0x320],rcx
   16265:	cmp    rdx,0x8
   16269:	jbe    165f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1720>
   1626f:	mov    rcx,QWORD PTR [rsp+0x88]
   16277:	add    rcx,rdi
   1627a:	mov    QWORD PTR [rsp+0x340],rcx
   16282:	cmp    rdx,0x9
   16286:	jbe    16603 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1733>
   1628c:	mov    rcx,QWORD PTR [rsp+0x90]
   16294:	add    rcx,rdi
   16297:	mov    QWORD PTR [rsp+0x360],rcx
   1629f:	cmp    rdx,0xa
   162a3:	jbe    16616 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1746>
   162a9:	mov    rcx,QWORD PTR [rsp+0x98]
   162b1:	add    rcx,rdi
   162b4:	mov    QWORD PTR [rsp+0x380],rcx
   162bc:	cmp    rdx,0xb
   162c0:	jbe    16629 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1759>
   162c6:	mov    rcx,QWORD PTR [rsp+0xa0]
   162ce:	add    rcx,rdi
   162d1:	mov    QWORD PTR [rsp+0x3a0],rcx
   162d9:	cmp    rdx,0xc
   162dd:	jbe    1663c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x176c>
   162e3:	mov    rcx,QWORD PTR [rsp+0xa8]
   162eb:	add    rcx,rdi
   162ee:	mov    QWORD PTR [rsp+0x3c0],rcx
   162f6:	cmp    rdx,0xd
   162fa:	jbe    1664f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x177f>
   16300:	mov    rcx,QWORD PTR [rsp+0xb0]
   16308:	lea    rdi,[rax+0x1]
   1630c:	add    rcx,rdi
   1630f:	mov    QWORD PTR [rsp+0x3e0],rcx
   16317:	cmp    rdx,0xe
   1631b:	jbe    16662 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1792>
   16321:	mov    rcx,QWORD PTR [rsp+0xb8]
   16329:	lea    rdi,[rax+0x1]
   1632d:	add    rcx,rdi
   16330:	mov    QWORD PTR [rsp+0x400],rcx
   16338:	cmp    rdx,0xf
   1633c:	jbe    16675 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17a5>
   16342:	mov    rcx,QWORD PTR [rsp+0xc0]
   1634a:	lea    rdi,[rax+0x1]
   1634e:	add    rcx,rdi
   16351:	mov    QWORD PTR [rsp+0x420],rcx
   16359:	cmp    rdx,0x10
   1635d:	jbe    16688 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17b8>
   16363:	mov    rcx,QWORD PTR [rsp+0xc8]
   1636b:	lea    rdi,[rax+0x1]
   1636f:	add    rcx,rdi
   16372:	mov    QWORD PTR [rsp+0x440],rcx
   1637a:	cmp    rdx,0x11
   1637e:	jbe    1669b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17cb>
   16384:	mov    rcx,QWORD PTR [rsp+0xd0]
   1638c:	lea    rdi,[rax+0x1]
   16390:	add    rcx,rdi
   16393:	mov    QWORD PTR [rsp+0x460],rcx
   1639b:	cmp    rdx,0x12
   1639f:	jbe    166ae <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17de>
   163a5:	mov    rcx,QWORD PTR [rsp+0xd8]
   163ad:	lea    rdi,[rax+0x1]
   163b1:	add    rcx,rdi
   163b4:	mov    QWORD PTR [rsp+0x480],rcx
   163bc:	cmp    rdx,0x13
   163c0:	jbe    166c1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17f1>
   163c6:	mov    rcx,QWORD PTR [rsp+0xe0]
   163ce:	lea    rdi,[rax+0x1]
   163d2:	add    rcx,rdi
   163d5:	mov    QWORD PTR [rsp+0x4a0],rcx
   163dd:	cmp    rdx,0x14
   163e1:	jbe    166d4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1804>
   163e7:	mov    rcx,QWORD PTR [rsp+0xe8]
   163ef:	lea    rdi,[rax+0x1]
   163f3:	add    rcx,rdi
   163f6:	mov    QWORD PTR [rsp+0x4c0],rcx
   163fe:	cmp    rdx,0x15
   16402:	jbe    166e7 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1817>
   16408:	mov    rcx,QWORD PTR [rsp+0xf0]
   16410:	lea    rdi,[rax+0x1]
   16414:	add    rcx,rdi
   16417:	mov    QWORD PTR [rsp+0x4e0],rcx
   1641f:	cmp    rdx,0x16
   16423:	jbe    166fa <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x182a>
   16429:	mov    rcx,QWORD PTR [rsp+0xf8]
   16431:	lea    rdi,[rax+0x1]
   16435:	add    rcx,rdi
   16438:	mov    QWORD PTR [rsp+0x500],rcx
   16440:	cmp    rdx,0x17
   16444:	jbe    1670d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x183d>
   1644a:	mov    rcx,QWORD PTR [rsp+0x100]
   16452:	lea    rdi,[rax+0x1]
   16456:	add    rcx,rdi
   16459:	mov    QWORD PTR [rsp+0x520],rcx
   16461:	cmp    rdx,0x18
   16465:	jbe    16720 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1850>
   1646b:	mov    rcx,QWORD PTR [rsp+0x108]
   16473:	lea    rdi,[rax+0x1]
   16477:	add    rcx,rdi
   1647a:	mov    QWORD PTR [rsp+0x540],rcx
   16482:	cmp    rdx,0x19
   16486:	jbe    16733 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1863>
   1648c:	mov    rcx,QWORD PTR [rsp+0x110]
   16494:	lea    rdi,[rax+0x1]
   16498:	add    rcx,rdi
   1649b:	mov    QWORD PTR [rsp+0x560],rcx
   164a3:	cmp    rdx,0x1a
   164a7:	jbe    16746 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1876>
   164ad:	mov    rcx,QWORD PTR [rsp+0x118]
   164b5:	lea    rdi,[rax+0x1]
   164b9:	add    rcx,rdi
   164bc:	mov    QWORD PTR [rsp+0x580],rcx
   164c4:	cmp    rdx,0x1b
   164c8:	jbe    16759 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1889>
   164ce:	mov    rcx,QWORD PTR [rsp+0x120]
   164d6:	lea    rdi,[rax+0x1]
   164da:	add    rcx,rdi
   164dd:	mov    QWORD PTR [rsp+0x5a0],rcx
   164e5:	cmp    rdx,0x1c
   164e9:	jbe    1676c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x189c>
   164ef:	mov    rcx,QWORD PTR [rsp+0x128]
   164f7:	lea    rdi,[rax+0x1]
   164fb:	add    rcx,rdi
   164fe:	mov    QWORD PTR [rsp+0x5c0],rcx
   16506:	cmp    rdx,0x1d
   1650a:	jbe    1677f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18af>
   16510:	mov    rcx,QWORD PTR [rsp+0x130]
   16518:	lea    rdi,[rax+0x1]
   1651c:	add    rcx,rdi
   1651f:	mov    QWORD PTR [rsp+0x5e0],rcx
   16527:	cmp    rdx,0x1e
   1652b:	jbe    16792 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
   16531:	mov    rax,QWORD PTR [rsp+0x138]
   16539:	mov    rcx,QWORD PTR [rsp]
   1653d:	add    rax,rcx
   16540:	mov    QWORD PTR [rsp+0x600],rax
   16548:	mov    rax,rcx
   1654b:	cmp    rdx,0x20
   1654f:	jae    16792 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
   16555:	jmp    167a2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
   1655a:	mov    rcx,QWORD PTR [rsp+0x50]
   1655f:	add    rcx,rax
   16562:	mov    QWORD PTR [rsp+0x260],rcx
   1656a:	mov    r9,QWORD PTR [rsp+0x148]
   16572:	mov    rsi,QWORD PTR [rsp+0x170]
   1657a:	mov    rcx,QWORD PTR [rsp+0x58]
   1657f:	add    rcx,rax
   16582:	mov    QWORD PTR [rsp+0x280],rcx
   1658a:	mov    rcx,QWORD PTR [rsp+0x60]
   1658f:	add    rcx,rax
   16592:	mov    QWORD PTR [rsp+0x2a0],rcx
   1659a:	mov    rcx,QWORD PTR [rsp+0x68]
   1659f:	add    rcx,rax
   165a2:	mov    QWORD PTR [rsp+0x2c0],rcx
   165aa:	mov    rcx,QWORD PTR [rsp+0x70]
   165af:	add    rcx,rax
   165b2:	mov    QWORD PTR [rsp+0x2e0],rcx
   165ba:	mov    rcx,QWORD PTR [rsp+0x78]
   165bf:	add    rcx,rax
   165c2:	mov    QWORD PTR [rsp+0x300],rcx
   165ca:	mov    rcx,QWORD PTR [rsp+0x80]
   165d2:	add    rcx,rax
   165d5:	mov    QWORD PTR [rsp+0x320],rcx
   165dd:	mov    rcx,QWORD PTR [rsp+0x88]
   165e5:	add    rcx,rax
   165e8:	mov    QWORD PTR [rsp+0x340],rcx
   165f0:	mov    rcx,QWORD PTR [rsp+0x90]
   165f8:	add    rcx,rax
   165fb:	mov    QWORD PTR [rsp+0x360],rcx
   16603:	mov    rcx,QWORD PTR [rsp+0x98]
   1660b:	add    rcx,rax
   1660e:	mov    QWORD PTR [rsp+0x380],rcx
   16616:	mov    rcx,QWORD PTR [rsp+0xa0]
   1661e:	add    rcx,rax
   16621:	mov    QWORD PTR [rsp+0x3a0],rcx
   16629:	mov    rcx,QWORD PTR [rsp+0xa8]
   16631:	add    rcx,rax
   16634:	mov    QWORD PTR [rsp+0x3c0],rcx
   1663c:	mov    rcx,QWORD PTR [rsp+0xb0]
   16644:	add    rcx,rax
   16647:	mov    QWORD PTR [rsp+0x3e0],rcx
   1664f:	mov    rcx,QWORD PTR [rsp+0xb8]
   16657:	add    rcx,rax
   1665a:	mov    QWORD PTR [rsp+0x400],rcx
   16662:	mov    rcx,QWORD PTR [rsp+0xc0]
   1666a:	add    rcx,rax
   1666d:	mov    QWORD PTR [rsp+0x420],rcx
   16675:	mov    rcx,QWORD PTR [rsp+0xc8]
   1667d:	add    rcx,rax
   16680:	mov    QWORD PTR [rsp+0x440],rcx
   16688:	mov    rcx,QWORD PTR [rsp+0xd0]
   16690:	add    rcx,rax
   16693:	mov    QWORD PTR [rsp+0x460],rcx
   1669b:	mov    rcx,QWORD PTR [rsp+0xd8]
   166a3:	add    rcx,rax
   166a6:	mov    QWORD PTR [rsp+0x480],rcx
   166ae:	mov    rcx,QWORD PTR [rsp+0xe0]
   166b6:	add    rcx,rax
   166b9:	mov    QWORD PTR [rsp+0x4a0],rcx
   166c1:	mov    rcx,QWORD PTR [rsp+0xe8]
   166c9:	add    rcx,rax
   166cc:	mov    QWORD PTR [rsp+0x4c0],rcx
   166d4:	mov    rcx,QWORD PTR [rsp+0xf0]
   166dc:	add    rcx,rax
   166df:	mov    QWORD PTR [rsp+0x4e0],rcx
   166e7:	mov    rcx,QWORD PTR [rsp+0xf8]
   166ef:	add    rcx,rax
   166f2:	mov    QWORD PTR [rsp+0x500],rcx
   166fa:	mov    rcx,QWORD PTR [rsp+0x100]
   16702:	add    rcx,rax
   16705:	mov    QWORD PTR [rsp+0x520],rcx
   1670d:	mov    rcx,QWORD PTR [rsp+0x108]
   16715:	add    rcx,rax
   16718:	mov    QWORD PTR [rsp+0x540],rcx
   16720:	mov    rcx,QWORD PTR [rsp+0x110]
   16728:	add    rcx,rax
   1672b:	mov    QWORD PTR [rsp+0x560],rcx
   16733:	mov    rcx,QWORD PTR [rsp+0x118]
   1673b:	add    rcx,rax
   1673e:	mov    QWORD PTR [rsp+0x580],rcx
   16746:	mov    rcx,QWORD PTR [rsp+0x120]
   1674e:	add    rcx,rax
   16751:	mov    QWORD PTR [rsp+0x5a0],rcx
   16759:	mov    rcx,QWORD PTR [rsp+0x128]
   16761:	add    rcx,rax
   16764:	mov    QWORD PTR [rsp+0x5c0],rcx
   1676c:	mov    rcx,QWORD PTR [rsp+0x130]
   16774:	add    rcx,rax
   16777:	mov    QWORD PTR [rsp+0x5e0],rcx
   1677f:	mov    rcx,QWORD PTR [rsp+0x138]
   16787:	add    rcx,rax
   1678a:	mov    QWORD PTR [rsp+0x600],rcx
   16792:	add    rax,QWORD PTR [rsp+0x140]
   1679a:	mov    QWORD PTR [rsp+0x620],rax
   167a2:	shl    rsi,0x5
   167a6:	lea    rax,[rsp+rsi*1]
   167aa:	add    rax,0x238
   167b0:	mov    QWORD PTR [rsp],rax
   167b4:	mov    rax,QWORD PTR [rbx+0x68]
   167b8:	mov    rsi,QWORD PTR [rbx+0x70]
   167bc:	lea    r12,[rsp+0x238]
   167c4:	jmp    167dd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x190d>
   167c6:	cs nop WORD PTR [rax+rax*1+0x0]
   167d0:	mov    r12,r13
   167d3:	add    r9,0xffffffffffffffe0
   167d7:	je     1691b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a4b>
   167dd:	lea    r13,[r12+0x20]
   167e2:	mov    r15,QWORD PTR [r12]
   167e6:	cmp    r15,0xffffffffffffffff
   167ea:	je     167d0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
   167ec:	cmp    r15,rsi
   167ef:	jae    16ec1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ff1>
   167f5:	cmp    QWORD PTR [rax+r15*8],0xffffffffffffffff
   167fa:	jne    167d0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
   167fc:	mov    edi,0x80
   16801:	call   QWORD PTR [rip+0x649c1]        # 7b1c8 <malloc@GLIBC_2.2.5>
   16807:	test   rax,rax
   1680a:	je     16f25 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2055>
   16810:	mov    rcx,QWORD PTR [r12+0x18]
   16815:	mov    QWORD PTR [rax+0x18],rcx
   16819:	movups xmm0,XMMWORD PTR [r12+0x8]
   1681f:	movups XMMWORD PTR [rax+0x8],xmm0
   16823:	mov    QWORD PTR [rax],r15
   16826:	mov    QWORD PTR [rsp+0x158],0x4
   16832:	mov    QWORD PTR [rsp+0x160],rax
   1683a:	mov    QWORD PTR [rsp+0x168],0x1
   16846:	cmp    r13,QWORD PTR [rsp]
   1684a:	je     1692d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a5d>
   16850:	mov    r12d,0x1
   16856:	lea    r15,[rsp+0x158]
   1685e:	mov    rcx,QWORD PTR [rbx+0x68]
   16862:	mov    rsi,QWORD PTR [rbx+0x70]
   16866:	mov    rdx,r13
   16869:	jmp    1687d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19ad>
   1686b:	nop    DWORD PTR [rax+rax*1+0x0]
   16870:	mov    rdx,r13
   16873:	cmp    r13,QWORD PTR [rsp]
   16877:	je     16933 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
   1687d:	lea    r13,[rdx+0x20]
   16881:	mov    rbp,QWORD PTR [rdx]
   16884:	cmp    rbp,0xffffffffffffffff
   16888:	je     16870 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
   1688a:	cmp    rbp,rsi
   1688d:	jae    16ed0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2000>
   16893:	cmp    QWORD PTR [rcx+rbp*8],0xffffffffffffffff
   16898:	jne    16870 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
   1689a:	mov    rcx,QWORD PTR [rdx+0x18]
   1689e:	mov    QWORD PTR [rsp+0x220],rcx
   168a6:	movups xmm0,XMMWORD PTR [rdx+0x8]
   168aa:	movaps XMMWORD PTR [rsp+0x210],xmm0
   168b2:	cmp    r12,QWORD PTR [rsp+0x158]
   168ba:	jne    168df <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a0f>
   168bc:	mov    edx,0x1
   168c1:	mov    ecx,0x8
   168c6:	mov    r8d,0x20
   168cc:	mov    rdi,r15
   168cf:	mov    rsi,r12
   168d2:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   168d7:	mov    rax,QWORD PTR [rsp+0x160]
   168df:	mov    rcx,r12
   168e2:	shl    rcx,0x5
   168e6:	mov    QWORD PTR [rax+rcx*1],rbp
   168ea:	movaps xmm0,XMMWORD PTR [rsp+0x210]
   168f2:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   168f7:	mov    rdx,QWORD PTR [rsp+0x220]
   168ff:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   16904:	inc    r12
   16907:	mov    QWORD PTR [rsp+0x168],r12
   1690f:	cmp    r13,QWORD PTR [rsp]
   16913:	jne    1685e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x198e>
   16919:	jmp    16933 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
   1691b:	mov    eax,0x8
   16920:	mov    QWORD PTR [rsp+0x8],rax
   16925:	xor    r14d,r14d
   16928:	jmp    16c41 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d71>
   1692d:	mov    r12d,0x1
   16933:	mov    rax,QWORD PTR [rsp+0x158]
   1693b:	mov    QWORD PTR [rsp+0x10],rax
   16940:	mov    rax,QWORD PTR [rsp+0x160]
   16948:	mov    QWORD PTR [rsp+0x8],rax
   1694d:	mov    r15,r12
   16950:	shl    r15,0x5
   16954:	xor    r13d,r13d
   16957:	lea    rax,[rip+0x62322]        # 78c80 <__frame_dummy_init_array_entry+0x270>
   1695e:	mov    QWORD PTR [rsp+0x30],rax
   16963:	jmp    1698e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1abe>
   16965:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   16970:	add    rbp,rax
   16973:	mov    rcx,QWORD PTR [rsp+0x8]
   16978:	mov    QWORD PTR [rcx+r13*1+0x8],rbp
   1697d:	add    rdx,rax
   16980:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   16985:	add    r13,0x20
   16989:	cmp    r15,r13
   1698c:	je     16a04 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b34>
   1698e:	mov    rax,QWORD PTR [rsp+0x8]
   16993:	mov    rbp,QWORD PTR [rax+r13*1+0x8]
   16998:	cmp    rbp,QWORD PTR [rax+r13*1+0x10]
   1699d:	jne    16985 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ab5>
   1699f:	mov    rsi,QWORD PTR [rbx+0x58]
   169a3:	mov    rax,QWORD PTR [rsp+0x8]
   169a8:	mov    rdi,QWORD PTR [rax+r13*1]
   169ac:	cmp    rdi,rsi
   169af:	jae    16eb0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
   169b5:	mov    rcx,QWORD PTR [rbx+0x50]
   169b9:	lea    rsi,[rdi+rdi*2]
   169bd:	mov    rdx,QWORD PTR [rcx+rsi*8]
   169c1:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   169c6:	sub    rbp,rax
   169c9:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   169ce:	mov    rdi,rdx
   169d1:	sub    rdi,rbp
   169d4:	cmp    rdx,rdi
   169d7:	jbe    16970 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
   169d9:	lea    rdi,[rcx+rsi*8]
   169dd:	mov    ecx,0x1
   169e2:	mov    r8d,0x1
   169e8:	mov    QWORD PTR [rsp],rdi
   169ec:	mov    rsi,rbp
   169ef:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   169f4:	mov    rax,QWORD PTR [rsp]
   169f8:	mov    rdx,QWORD PTR [rax]
   169fb:	mov    rax,QWORD PTR [rax+0x8]
   169ff:	jmp    16970 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
   16a04:	mov    rax,QWORD PTR [rsp+0x8]
   16a09:	add    rax,0x18
   16a0d:	mov    QWORD PTR [rsp],rax
   16a11:	xor    r15d,r15d
   16a14:	jmp    16a23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   16a16:	cs nop WORD PTR [rax+rax*1+0x0]
   16a20:	xor    r15d,r15d
   16a23:	mov    r13,r15
   16a26:	shl    r13,0x5
   16a2a:	add    r13,QWORD PTR [rsp]
   16a2e:	jmp    16a4a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b7a>
   16a30:	add    rbp,rax
   16a33:	mov    QWORD PTR [r13-0x10],rbp
   16a37:	add    rdx,rax
   16a3a:	mov    QWORD PTR [r13-0x8],rdx
   16a3e:	inc    r15
   16a41:	add    r13,0x20
   16a45:	cmp    r15,r12
   16a48:	jae    16a20 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b50>
   16a4a:	lea    rax,[r13-0x18]
   16a4e:	mov    ecx,DWORD PTR [r13+0x0]
   16a52:	movzx  esi,WORD PTR [r14+rcx*2]
   16a57:	mov    edx,ecx
   16a59:	and    edx,0xffffffe0
   16a5c:	mov    edx,DWORD PTR [r14+rdx*2+0x3c]
   16a61:	cmp    esi,0xfffe
   16a67:	jae    16a80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bb0>
   16a69:	mov    ecx,edx
   16a6b:	shr    ecx,0x8
   16a6e:	add    ecx,esi
   16a70:	jmp    16aa1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bd1>
   16a72:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   16a80:	je     16b30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1c60>
   16a86:	cmp    ecx,DWORD PTR [rsp+0x24]
   16a8a:	jne    16bc0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cf0>
   16a90:	mov    ecx,DWORD PTR [r13+0x4]
   16a94:	mov    rdx,QWORD PTR [rsp+0x38]
   16a99:	mov    edx,DWORD PTR [rdx+rcx*4]
   16a9c:	mov    ecx,edx
   16a9e:	shr    ecx,0x8
   16aa1:	mov    DWORD PTR [r13+0x0],ecx
   16aa5:	mov    ecx,ecx
   16aa7:	prefetcht0 BYTE PTR [r14+rcx*2]
   16aac:	mov    rcx,QWORD PTR [r13-0x10]
   16ab0:	mov    BYTE PTR [rcx],dl
   16ab2:	mov    rbp,QWORD PTR [r13-0x10]
   16ab6:	inc    rbp
   16ab9:	mov    QWORD PTR [r13-0x10],rbp
   16abd:	cmp    rbp,QWORD PTR [r13-0x8]
   16ac1:	jne    16a3e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b6e>
   16ac7:	mov    rsi,QWORD PTR [rbx+0x58]
   16acb:	mov    rdi,QWORD PTR [rax]
   16ace:	cmp    rdi,rsi
   16ad1:	jae    16eb0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
   16ad7:	mov    rcx,QWORD PTR [rbx+0x50]
   16adb:	lea    rsi,[rdi+rdi*2]
   16adf:	mov    rdx,QWORD PTR [rcx+rsi*8]
   16ae3:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   16ae8:	sub    rbp,rax
   16aeb:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   16af0:	mov    rdi,rdx
   16af3:	sub    rdi,rbp
   16af6:	cmp    rdx,rdi
   16af9:	jbe    16a30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
   16aff:	lea    rdi,[rcx+rsi*8]
   16b03:	mov    ecx,0x1
   16b08:	mov    r8d,0x1
   16b0e:	mov    QWORD PTR [rsp+0x40],rdi
   16b13:	mov    rsi,rbp
   16b16:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   16b1b:	mov    rax,QWORD PTR [rsp+0x40]
   16b20:	mov    rdx,QWORD PTR [rax]
   16b23:	mov    rax,QWORD PTR [rax+0x8]
   16b27:	jmp    16a30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
   16b2c:	nop    DWORD PTR [rax+0x0]
   16b30:	mov    rsi,QWORD PTR [rbx+0x58]
   16b34:	mov    rdi,QWORD PTR [rax]
   16b37:	cmp    rdi,rsi
   16b3a:	jae    16ea4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fd4>
   16b40:	mov    r8,QWORD PTR [rbx+0x50]
   16b44:	mov    rdx,QWORD PTR [rbx+0x68]
   16b48:	lea    r9,[rdi+rdi*2]
   16b4c:	mov    r10,QWORD PTR [rax+0x8]
   16b50:	sub    r10,QWORD PTR [r8+r9*8+0x8]
   16b55:	mov    rsi,QWORD PTR [rbx+0x70]
   16b59:	mov    QWORD PTR [r8+r9*8+0x10],r10
   16b5e:	mov    r10,QWORD PTR [rbx+0x40]
   16b62:	test   r10,r10
   16b65:	je     16bf1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d21>
   16b6b:	mov    r9,QWORD PTR [rbx+0x38]
   16b6f:	xor    r8d,r8d
   16b72:	cmp    r10,0x1
   16b76:	je     16b9d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ccd>
   16b78:	nop    DWORD PTR [rax+rax*1+0x0]
   16b80:	mov    r11,r8
   16b83:	mov    rbx,r10
   16b86:	shr    rbx,1
   16b89:	add    r8,rbx
   16b8c:	cmp    DWORD PTR [r9+r8*4],ecx
   16b90:	cmova  r8,r11
   16b94:	sub    r10,rbx
   16b97:	cmp    r10,0x1
   16b9b:	ja     16b80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cb0>
   16b9d:	cmp    DWORD PTR [r9+r8*4],ecx
   16ba1:	mov    rcx,0xffffffffffffffff
   16ba8:	cmovne r8,rcx
   16bac:	mov    rbx,QWORD PTR [rsp+0x18]
   16bb1:	cmp    rdi,rsi
   16bb4:	jb     16c01 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d31>
   16bb6:	jmp    16e9b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
   16bbb:	nop    DWORD PTR [rax+rax*1+0x0]
   16bc0:	shl    edx,0x5
   16bc3:	and    edx,0x1ffffe0
   16bc9:	and    ecx,0x1f
   16bcc:	or     ecx,edx
   16bce:	mov    DWORD PTR [r13+0x4],ecx
   16bd2:	mov    rax,QWORD PTR [rsp+0x38]
   16bd7:	prefetcht0 BYTE PTR [rax+rcx*4]
   16bdb:	mov    eax,DWORD PTR [rsp+0x24]
   16bdf:	mov    DWORD PTR [r13+0x0],eax
   16be3:	inc    r15
   16be6:	cmp    r15,r12
   16be9:	jb     16a23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   16bef:	jmp    16c2d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d5d>
   16bf1:	mov    r8,0xffffffffffffffff
   16bf8:	cmp    rdi,rsi
   16bfb:	jae    16e9b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
   16c01:	mov    QWORD PTR [rdx+rdi*8],r8
   16c05:	dec    r12
   16c08:	mov    rcx,r12
   16c0b:	shl    rcx,0x5
   16c0f:	mov    rdx,QWORD PTR [rsp+0x8]
   16c14:	movups xmm0,XMMWORD PTR [rdx+rcx*1]
   16c18:	movups xmm1,XMMWORD PTR [rdx+rcx*1+0x10]
   16c1d:	movups XMMWORD PTR [rax+0x10],xmm1
   16c21:	movups XMMWORD PTR [rax],xmm0
   16c24:	cmp    r15,r12
   16c27:	jb     16a23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   16c2d:	mov    r15d,0x0
   16c33:	test   r12,r12
   16c36:	jne    16a23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   16c3c:	mov    r14,QWORD PTR [rsp+0x10]
   16c41:	xor    eax,eax
   16c43:	lea    rcx,[rip+0x61dfe]        # 78a48 <__frame_dummy_init_array_entry+0x38>
   16c4a:	lea    r8,[rip+0x61e0f]        # 78a60 <__frame_dummy_init_array_entry+0x50>
   16c51:	lea    r9,[rip+0x61e20]        # 78a78 <__frame_dummy_init_array_entry+0x68>
   16c58:	lea    rdx,[rip+0x61e31]        # 78a90 <__frame_dummy_init_array_entry+0x80>
   16c5f:	mov    r11,QWORD PTR [rsp+0x28]
   16c64:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   16c70:	mov    rsi,QWORD PTR [rbx+0x40]
   16c74:	cmp    rax,rsi
   16c77:	jae    16e3a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f6a>
   16c7d:	mov    rsi,QWORD PTR [rbx+0x98]
   16c84:	cmp    rax,rsi
   16c87:	jae    16e32 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f62>
   16c8d:	mov    rsi,QWORD PTR [rbx+0x38]
   16c91:	mov    edi,DWORD PTR [rsi+rax*4]
   16c94:	mov    rsi,QWORD PTR [rbx+0x10]
   16c98:	mov    r10d,edi
   16c9b:	shr    r10d,0x5
   16c9f:	cmp    rsi,r10
   16ca2:	jbe    16e2a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f5a>
   16ca8:	and    edi,0x1f
   16cab:	cmp    edi,0x1d
   16cae:	ja     16e23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f53>
   16cb4:	mov    rsi,QWORD PTR [rbx+0x90]
   16cbb:	movzx  esi,WORD PTR [rsi+rax*2]
   16cbf:	inc    rax
   16cc2:	shl    r10,0x6
   16cc6:	add    r10,QWORD PTR [rbx+0x8]
   16cca:	mov    WORD PTR [r10+rdi*2],si
   16ccf:	cmp    r11,rax
   16cd2:	jne    16c70 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1da0>
   16cd4:	mov    rax,QWORD PTR [rbx+0x38]
   16cd8:	mov    rcx,QWORD PTR [rbx+0x40]
   16cdc:	mov    edi,DWORD PTR [rsp+0x154]
   16ce3:	mov    esi,edi
   16ce5:	mov    edx,0x88888889
   16cea:	imul   rdx,rsi
   16cee:	shr    rdx,0x23
   16cf2:	and    edx,0xfffffffe
   16cf5:	add    edx,edi
   16cf7:	cmp    rcx,0x1
   16cfb:	je     16d2e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e5e>
   16cfd:	test   rcx,rcx
   16d00:	je     16ef4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2024>
   16d06:	xor    esi,esi
   16d08:	nop    DWORD PTR [rax+rax*1+0x0]
   16d10:	mov    rdi,rsi
   16d13:	mov    r8,rcx
   16d16:	shr    r8,1
   16d19:	add    rsi,r8
   16d1c:	cmp    DWORD PTR [rax+rsi*4],edx
   16d1f:	cmova  rsi,rdi
   16d23:	sub    rcx,r8
   16d26:	cmp    rcx,0x1
   16d2a:	ja     16d10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e40>
   16d2c:	jmp    16d30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e60>
   16d2e:	xor    esi,esi
   16d30:	cmp    DWORD PTR [rax+rsi*4],edx
   16d33:	jne    16eed <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x201d>
   16d39:	mov    QWORD PTR [rbx+0x80],rsi
   16d40:	mov    rax,QWORD PTR [rsp+0x178]
   16d48:	mov    QWORD PTR [rbx+0x78],rax
   16d4c:	test   r14,r14
   16d4f:	je     16d5c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e8c>
   16d51:	mov    rdi,QWORD PTR [rsp+0x8]
   16d56:	call   QWORD PTR [rip+0x64474]        # 7b1d0 <free@GLIBC_2.2.5>
   16d5c:	add    rsp,0x638
   16d63:	pop    rbx
   16d64:	pop    r12
   16d66:	pop    r13
   16d68:	pop    r14
   16d6a:	pop    r15
   16d6c:	pop    rbp
   16d6d:	ret
   16d6e:	lea    rcx,[rip+0x61d33]        # 78aa8 <__frame_dummy_init_array_entry+0x98>
   16d75:	xor    edi,edi
   16d77:	call   2ddc0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   16d7c:	lea    rdi,[rbx+0x60]
   16d80:	mov    ecx,0x8
   16d85:	mov    r8d,0x8
   16d8b:	xor    esi,esi
   16d8d:	mov    rdx,r12
   16d90:	call   1bbb0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   16d95:	mov    r15,QWORD PTR [rbx+0x70]
   16d99:	jmp    152d6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x406>
   16d9e:	lea    rdx,[rip+0x61d7b]        # 78b20 <__frame_dummy_init_array_entry+0x110>
   16da5:	mov    rdi,r12
   16da8:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16dad:	lea    rdx,[rip+0x6219c]        # 78f50 <__frame_dummy_init_array_entry+0x540>
   16db4:	mov    esi,0x1e
   16db9:	mov    rdi,rbp
   16dbc:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16dc1:	lea    rdx,[rip+0x62170]        # 78f38 <__frame_dummy_init_array_entry+0x528>
   16dc8:	mov    rdi,r12
   16dcb:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16dd0:	lea    rdx,[rip+0x61d31]        # 78b08 <__frame_dummy_init_array_entry+0xf8>
   16dd7:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16ddc:	lea    rdx,[rip+0x61d0d]        # 78af0 <__frame_dummy_init_array_entry+0xe0>
   16de3:	mov    esi,0x1e
   16de8:	mov    rdi,rax
   16deb:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16df0:	lea    rdx,[rip+0x61ce1]        # 78ad8 <__frame_dummy_init_array_entry+0xc8>
   16df7:	mov    rdi,rcx
   16dfa:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16dff:	lea    rdx,[rip+0x61cba]        # 78ac0 <__frame_dummy_init_array_entry+0xb0>
   16e06:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e0b:	lea    rdx,[rip+0x61e26]        # 78c38 <__frame_dummy_init_array_entry+0x228>
   16e12:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e17:	lea    rdx,[rip+0x61e02]        # 78c20 <__frame_dummy_init_array_entry+0x210>
   16e1e:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e23:	mov    esi,0x1e
   16e28:	jmp    16e40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   16e2a:	mov    rdi,r10
   16e2d:	mov    rdx,r9
   16e30:	jmp    16e40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   16e32:	mov    rdi,rax
   16e35:	mov    rdx,r8
   16e38:	jmp    16e40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   16e3a:	mov    rdi,rax
   16e3d:	mov    rdx,rcx
   16e40:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e45:	jmp    16f23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   16e4a:	lea    rdx,[rip+0x61e2f]        # 78c80 <__frame_dummy_init_array_entry+0x270>
   16e51:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e56:	lea    rdx,[rip+0x6210b]        # 78f68 <__frame_dummy_init_array_entry+0x558>
   16e5d:	mov    rdi,r13
   16e60:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e65:	lea    rdx,[rip+0x61dfc]        # 78c68 <__frame_dummy_init_array_entry+0x258>
   16e6c:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e71:	lea    rdx,[rip+0x61dd8]        # 78c50 <__frame_dummy_init_array_entry+0x240>
   16e78:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e7d:	lea    rdx,[rip+0x61db4]        # 78c38 <__frame_dummy_init_array_entry+0x228>
   16e84:	mov    rdi,rbp
   16e87:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e8c:	lea    rdx,[rip+0x61d8d]        # 78c20 <__frame_dummy_init_array_entry+0x210>
   16e93:	mov    rdi,rbp
   16e96:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16e9b:	lea    rax,[rip+0x61dc6]        # 78c68 <__frame_dummy_init_array_entry+0x258>
   16ea2:	jmp    16eab <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fdb>
   16ea4:	lea    rax,[rip+0x61da5]        # 78c50 <__frame_dummy_init_array_entry+0x240>
   16eab:	mov    QWORD PTR [rsp+0x30],rax
   16eb0:	mov    rdx,QWORD PTR [rsp+0x30]
   16eb5:	mov    r14,QWORD PTR [rsp+0x10]
   16eba:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16ebf:	jmp    16f23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   16ec1:	lea    rdx,[rip+0x61dd0]        # 78c98 <__frame_dummy_init_array_entry+0x288>
   16ec8:	mov    rdi,r15
   16ecb:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16ed0:	lea    rdx,[rip+0x61dc1]        # 78c98 <__frame_dummy_init_array_entry+0x288>
   16ed7:	mov    rdi,rbp
   16eda:	call   2d2a3 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   16edf:	jmp    16f23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   16ee1:	lea    rdi,[rip+0x61b30]        # 78a18 <__frame_dummy_init_array_entry+0x8>
   16ee8:	call   2f4c0 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   16eed:	adc    rsi,0x0
   16ef1:	mov    rcx,rsi
   16ef4:	mov    QWORD PTR [rsp+0x210],rcx
   16efc:	lea    rdi,[rip+0xffffffffffff2ce5]        # 9be8 <GCC_except_table549+0x3e68>
   16f03:	lea    rcx,[rip+0x6200e]        # 78f18 <__frame_dummy_init_array_entry+0x508>
   16f0a:	lea    r8,[rip+0x61b1f]        # 78a30 <__frame_dummy_init_array_entry+0x20>
   16f11:	lea    rdx,[rsp+0x210]
   16f19:	mov    esi,0x19
   16f1e:	call   2f2e0 <_RNvNtCscI6d9CVNmLh_4core6result13unwrap_failed>
   16f23:	ud2
   16f25:	mov    edi,0x8
   16f2a:	mov    esi,0x80
   16f2f:	call   2b69b <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   16f34:	jmp    16f38 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2068>
   16f36:	jmp    16f42 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2072>
   16f38:	mov    rbx,rax
   16f3b:	mov    r14,QWORD PTR [rsp+0x10]
   16f40:	jmp    16f62 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2092>
   16f42:	mov    rbx,rax
   16f45:	cmp    QWORD PTR [rsp+0x158],0x0
   16f4e:	je     16f72 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
   16f50:	mov    rax,QWORD PTR [rsp+0x160]
   16f58:	mov    QWORD PTR [rsp+0x8],rax
   16f5d:	jmp    16f67 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2097>
   16f5f:	mov    rbx,rax
   16f62:	test   r14,r14
   16f65:	je     16f72 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
   16f67:	mov    rdi,QWORD PTR [rsp+0x8]
   16f6c:	call   QWORD PTR [rip+0x6425e]        # 7b1d0 <free@GLIBC_2.2.5>
   16f72:	mov    rdi,rbx
   16f75:	call   779c0 <_Unwind_Resume@plt>
   16f7a:	int3
   16f7b:	int3
   16f7c:	int3
   16f7d:	int3
   16f7e:	int3
   16f7f:	int3
