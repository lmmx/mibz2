0000000000016000 <mibz2::bwt::room_segments_unrolled::<32, 1>>:
   16000:	push   rbp
   16001:	push   r15
   16003:	push   r14
   16005:	push   r13
   16007:	push   r12
   16009:	push   rbx
   1600a:	sub    rsp,0x668
   16011:	mov    r14,rsi
   16014:	mov    r15,rdi
   16017:	mov    edi,edx
   16019:	cmp    rsi,rdi
   1601c:	jbe    1d2b9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72b9>
   16022:	mov    r9d,DWORD PTR [r15+rdi*4]
   16026:	dec    r8
   16029:	mov    QWORD PTR [rcx+0x10],0x0
   16031:	mov    rax,QWORD PTR [rcx]
   16034:	cmp    r8,rax
   16037:	mov    rdi,rcx
   1603a:	mov    QWORD PTR [rsp],rcx
   1603e:	ja     1d048 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7048>
   16044:	xor    r12d,r12d
   16047:	test   r8,r8
   1604a:	je     16099 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x99>
   1604c:	movabs rcx,0x2545f4914f6cdd1d
   16056:	xor    rcx,r14
   16059:	mov    rsi,QWORD PTR [rdi+0x8]
   1605d:	nop    DWORD PTR [rax]
   16060:	mov    rax,rcx
   16063:	shl    rax,0xd
   16067:	xor    rax,rcx
   1606a:	mov    rdx,rax
   1606d:	shr    rdx,0x7
   16071:	xor    rdx,rax
   16074:	mov    rcx,rdx
   16077:	shl    rcx,0x11
   1607b:	xor    rcx,rdx
   1607e:	mov    rax,rcx
   16081:	xor    edx,edx
   16083:	div    r14
   16086:	mov    DWORD PTR [rsi+r12*4],edx
   1608a:	inc    r12
   1608d:	dec    r8
   16090:	jne    16060 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x60>
   16092:	mov    rdi,QWORD PTR [rsp]
   16096:	mov    rax,QWORD PTR [rdi]
   16099:	shr    r9d,0x8
   1609d:	mov    QWORD PTR [rdi+0x10],r12
   160a1:	cmp    r12,rax
   160a4:	mov    DWORD PTR [rsp+0x50c],r9d
   160ac:	je     1a036 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4036>
   160b2:	mov    rax,QWORD PTR [rdi+0x8]
   160b6:	mov    DWORD PTR [rax+r12*4],r9d
   160ba:	inc    r12
   160bd:	mov    QWORD PTR [rdi+0x10],r12
   160c1:	mov    rdi,QWORD PTR [rdi+0x8]
   160c5:	cmp    r12,0x2
   160c9:	jae    1d076 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7076>
   160cf:	test   r12,r12
   160d2:	je     1d2c8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72c8>
   160d8:	lea    rcx,[rdi+r12*4]
   160dc:	lea    rsi,[rsp+0x4d0]
   160e4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   160f0:	mov    eax,DWORD PTR [rdi]
   160f2:	cmp    r14,rax
   160f5:	jbe    1d1f9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x71f9>
   160fb:	add    rdi,0x4
   160ff:	or     BYTE PTR [r15+rax*4+0x3],0x80
   16105:	cmp    rdi,rcx
   16108:	jne    160f0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xf0>
   1610a:	lea    rax,[r14+r14*1]
   1610e:	mov    rcx,rax
   16111:	or     rcx,r12
   16114:	shr    rcx,0x20
   16118:	je     16124 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x124>
   1611a:	xor    edx,edx
   1611c:	div    r12
   1611f:	mov    r9,rax
   16122:	jmp    1612c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x12c>
   16124:	xor    edx,edx
   16126:	div    r12d
   16129:	mov    r9d,eax
   1612c:	mov    rbx,QWORD PTR [rsp]
   16130:	mov    r13,QWORD PTR [rbx+0x40]
   16134:	cmp    r13,r12
   16137:	mov    rbp,r12
   1613a:	cmova  rbp,r13
   1613e:	jae    1a011 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4011>
   16144:	sub    rbp,r13
   16147:	mov    rax,QWORD PTR [rbx+0x30]
   1614b:	sub    rax,r13
   1614e:	mov    rdx,r13
   16151:	cmp    rbp,rax
   16154:	ja     1d11b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x711b>
   1615a:	mov    rax,QWORD PTR [rbx+0x38]
   1615e:	inc    r13
   16161:	test   bpl,0x1
   16165:	jne    1d162 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7162>
   1616b:	cmp    r12,r13
   1616e:	jne    1d18f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x718f>
   16174:	mov    rcx,QWORD PTR [rsp]
   16178:	mov    QWORD PTR [rcx+0x40],rdx
   1617c:	cmp    r12,rdx
   1617f:	ja     1a025 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4025>
   16185:	lea    r13,[r9+0x40]
   16189:	mov    rbp,QWORD PTR [rcx+0x38]
   1618d:	lea    rax,[r12*8+0x0]
   16195:	lea    rbx,[rax+rax*2]
   16199:	mov    rdx,rcx
   1619c:	nop    DWORD PTR [rax+0x0]
   161a0:	mov    QWORD PTR [rbp+0x10],0x0
   161a8:	cmp    r13,QWORD PTR [rbp+0x0]
   161ac:	ja     161ba <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1ba>
   161ae:	add    rbp,0x18
   161b2:	add    rbx,0xffffffffffffffe8
   161b6:	jne    161a0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1a0>
   161b8:	jmp    161f8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1f8>
   161ba:	mov    ecx,0x1
   161bf:	mov    r8d,0x1
   161c5:	mov    rdi,rbp
   161c8:	xor    esi,esi
   161ca:	mov    rdx,r13
   161cd:	mov    QWORD PTR [rsp+0x510],r9
   161d5:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   161da:	mov    r9,QWORD PTR [rsp+0x510]
   161e2:	lea    rsi,[rsp+0x4d0]
   161ea:	mov    rdx,QWORD PTR [rsp]
   161ee:	add    rbp,0x18
   161f2:	add    rbx,0xffffffffffffffe8
   161f6:	jne    161a0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1a0>
   161f8:	mov    QWORD PTR [rdx+0x58],0x0
   16200:	cmp    r12,QWORD PTR [rdx+0x48]
   16204:	mov    QWORD PTR [rsp+0x648],r14
   1620c:	ja     1d0d7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x70d7>
   16212:	xor    ebx,ebx
   16214:	mov    r14,QWORD PTR [rdx+0x50]
   16218:	lea    rdi,[r14+rbx*8]
   1621c:	cmp    r12,0x1
   16220:	je     16257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x257>
   16222:	lea    rdx,[r12*8-0x8]
   1622a:	mov    r13,rsi
   1622d:	mov    esi,0xff
   16232:	mov    rbp,r9
   16235:	call   QWORD PTR [rip+0x5b4d5]        # 71710 <memset@GLIBC_2.2.5>
   1623b:	mov    r9,rbp
   1623e:	mov    rsi,r13
   16241:	mov    rdx,QWORD PTR [rsp]
   16245:	lea    rax,[rbx+r12*1]
   16249:	lea    rdi,[r14+rax*8]
   1624d:	add    rdi,0xfffffffffffffff8
   16251:	add    rbx,r12
   16254:	dec    rbx
   16257:	mov    QWORD PTR [rdi],0xffffffffffffffff
   1625e:	inc    rbx
   16261:	mov    QWORD PTR [rdx+0x58],rbx
   16265:	xorps  xmm0,xmm0
   16268:	movups XMMWORD PTR [rsp+0x110],xmm0
   16270:	mov    DWORD PTR [rsp+0x120],0x0
   1627b:	movups XMMWORD PTR [rsp+0x130],xmm0
   16283:	mov    DWORD PTR [rsp+0x140],0x0
   1628e:	movups XMMWORD PTR [rsp+0x150],xmm0
   16296:	mov    DWORD PTR [rsp+0x160],0x0
   162a1:	movups XMMWORD PTR [rsp+0x170],xmm0
   162a9:	mov    DWORD PTR [rsp+0x180],0x0
   162b4:	movups XMMWORD PTR [rsp+0x190],xmm0
   162bc:	mov    DWORD PTR [rsp+0x1a0],0x0
   162c7:	movups XMMWORD PTR [rsp+0x1b0],xmm0
   162cf:	mov    DWORD PTR [rsp+0x1c0],0x0
   162da:	movups XMMWORD PTR [rsp+0x1d0],xmm0
   162e2:	mov    DWORD PTR [rsp+0x1e0],0x0
   162ed:	movups XMMWORD PTR [rsp+0x1f0],xmm0
   162f5:	mov    DWORD PTR [rsp+0x200],0x0
   16300:	movups XMMWORD PTR [rsp+0x210],xmm0
   16308:	mov    DWORD PTR [rsp+0x220],0x0
   16313:	movups XMMWORD PTR [rsp+0x230],xmm0
   1631b:	mov    DWORD PTR [rsp+0x240],0x0
   16326:	movups XMMWORD PTR [rsp+0x250],xmm0
   1632e:	mov    DWORD PTR [rsp+0x260],0x0
   16339:	movups XMMWORD PTR [rsp+0x270],xmm0
   16341:	mov    DWORD PTR [rsp+0x280],0x0
   1634c:	movups XMMWORD PTR [rsp+0x290],xmm0
   16354:	mov    DWORD PTR [rsp+0x2a0],0x0
   1635f:	movups XMMWORD PTR [rsp+0x2b0],xmm0
   16367:	mov    DWORD PTR [rsp+0x2c0],0x0
   16372:	movups XMMWORD PTR [rsp+0x2d0],xmm0
   1637a:	mov    DWORD PTR [rsp+0x2e0],0x0
   16385:	movups XMMWORD PTR [rsp+0x2f0],xmm0
   1638d:	mov    DWORD PTR [rsp+0x300],0x0
   16398:	movups XMMWORD PTR [rsp+0x310],xmm0
   163a0:	mov    DWORD PTR [rsp+0x320],0x0
   163ab:	movups XMMWORD PTR [rsp+0x330],xmm0
   163b3:	mov    DWORD PTR [rsp+0x340],0x0
   163be:	movups XMMWORD PTR [rsp+0x350],xmm0
   163c6:	mov    DWORD PTR [rsp+0x360],0x0
   163d1:	movups XMMWORD PTR [rsp+0x370],xmm0
   163d9:	mov    DWORD PTR [rsp+0x380],0x0
   163e4:	movups XMMWORD PTR [rsp+0x390],xmm0
   163ec:	mov    DWORD PTR [rsp+0x3a0],0x0
   163f7:	movups XMMWORD PTR [rsp+0x3b0],xmm0
   163ff:	mov    DWORD PTR [rsp+0x3c0],0x0
   1640a:	movups XMMWORD PTR [rsp+0x3d0],xmm0
   16412:	mov    DWORD PTR [rsp+0x3e0],0x0
   1641d:	movups XMMWORD PTR [rsp+0x3f0],xmm0
   16425:	mov    DWORD PTR [rsp+0x400],0x0
   16430:	movups XMMWORD PTR [rsp+0x410],xmm0
   16438:	mov    DWORD PTR [rsp+0x420],0x0
   16443:	movups XMMWORD PTR [rsp+0x430],xmm0
   1644b:	mov    DWORD PTR [rsp+0x440],0x0
   16456:	movups XMMWORD PTR [rsp+0x450],xmm0
   1645e:	mov    DWORD PTR [rsp+0x460],0x0
   16469:	movups XMMWORD PTR [rsp+0x470],xmm0
   16471:	mov    DWORD PTR [rsp+0x480],0x0
   1647c:	movups XMMWORD PTR [rsp+0x490],xmm0
   16484:	mov    DWORD PTR [rsp+0x4a0],0x0
   1648f:	movups XMMWORD PTR [rsp+0x4b0],xmm0
   16497:	mov    DWORD PTR [rsp+0x4c0],0x0
   164a2:	movups XMMWORD PTR [rsi],xmm0
   164a5:	mov    DWORD PTR [rsi+0x10],0x0
   164ac:	lea    rax,[rsp+0x4f0]
   164b4:	movups XMMWORD PTR [rax],xmm0
   164b7:	mov    DWORD PTR [rax+0x10],0x0
   164be:	mov    QWORD PTR [rsp+0x108],0xffffffffffffffff
   164ca:	mov    QWORD PTR [rsp+0x128],0xffffffffffffffff
   164d6:	mov    QWORD PTR [rsp+0x148],0xffffffffffffffff
   164e2:	mov    QWORD PTR [rsp+0x168],0xffffffffffffffff
   164ee:	mov    QWORD PTR [rsp+0x188],0xffffffffffffffff
   164fa:	mov    QWORD PTR [rsp+0x1a8],0xffffffffffffffff
   16506:	mov    QWORD PTR [rsp+0x1c8],0xffffffffffffffff
   16512:	mov    QWORD PTR [rsp+0x1e8],0xffffffffffffffff
   1651e:	mov    QWORD PTR [rsp+0x208],0xffffffffffffffff
   1652a:	mov    QWORD PTR [rsp+0x228],0xffffffffffffffff
   16536:	mov    QWORD PTR [rsp+0x248],0xffffffffffffffff
   16542:	mov    QWORD PTR [rsp+0x268],0xffffffffffffffff
   1654e:	mov    QWORD PTR [rsp+0x288],0xffffffffffffffff
   1655a:	mov    QWORD PTR [rsp+0x2a8],0xffffffffffffffff
   16566:	mov    QWORD PTR [rsp+0x2c8],0xffffffffffffffff
   16572:	mov    QWORD PTR [rsp+0x2e8],0xffffffffffffffff
   1657e:	mov    QWORD PTR [rsp+0x308],0xffffffffffffffff
   1658a:	mov    QWORD PTR [rsp+0x328],0xffffffffffffffff
   16596:	mov    QWORD PTR [rsp+0x348],0xffffffffffffffff
   165a2:	mov    QWORD PTR [rsp+0x368],0xffffffffffffffff
   165ae:	mov    QWORD PTR [rsp+0x388],0xffffffffffffffff
   165ba:	mov    QWORD PTR [rsp+0x3a8],0xffffffffffffffff
   165c6:	mov    QWORD PTR [rsp+0x3c8],0xffffffffffffffff
   165d2:	mov    QWORD PTR [rsp+0x3e8],0xffffffffffffffff
   165de:	mov    QWORD PTR [rsp+0x408],0xffffffffffffffff
   165ea:	mov    QWORD PTR [rsp+0x428],0xffffffffffffffff
   165f6:	mov    QWORD PTR [rsp+0x448],0xffffffffffffffff
   16602:	mov    QWORD PTR [rsp+0x468],0xffffffffffffffff
   1660e:	mov    QWORD PTR [rsp+0x488],0xffffffffffffffff
   1661a:	mov    QWORD PTR [rsp+0x4a8],0xffffffffffffffff
   16626:	mov    QWORD PTR [rsp+0x4c8],0xffffffffffffffff
   16632:	mov    QWORD PTR [rsp+0x4e8],0xffffffffffffffff
   1663e:	mov    eax,0x2
   16643:	xor    edi,edi
   16645:	xor    ebp,ebp
   16647:	nop    WORD PTR [rax+rax*1+0x0]
   16650:	mov    rsi,QWORD PTR [rdx+0x40]
   16654:	cmp    rdi,rsi
   16657:	jae    1d217 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7217>
   1665d:	mov    rsi,QWORD PTR [rdx+0x10]
   16661:	cmp    rdi,rsi
   16664:	jae    1d20b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x720b>
   1666a:	mov    rcx,QWORD PTR [rdx+0x38]
   1666e:	mov    rdx,QWORD PTR [rsp]
   16672:	mov    rdx,QWORD PTR [rdx+0x8]
   16676:	mov    rsi,QWORD PTR [rcx+rax*4]
   1667a:	mov    edx,DWORD PTR [rdx+rdi*4]
   1667d:	mov    edx,DWORD PTR [r15+rdx*4]
   16681:	mov    BYTE PTR [rsi],dl
   16683:	shr    edx,0x8
   16686:	and    edx,0x7fffff
   1668c:	lea    r8,[rsi+0x1]
   16690:	add    rsi,QWORD PTR [rcx+rax*4-0x8]
   16695:	mov    QWORD PTR [rsp+rbp*1+0x108],rdi
   1669d:	mov    QWORD PTR [rsp+rbp*1+0x110],r8
   166a5:	mov    QWORD PTR [rsp+rbp*1+0x118],rsi
   166ad:	mov    DWORD PTR [rsp+rbp*1+0x120],edx
   166b4:	prefetcht0 BYTE PTR [r15+rdx*4]
   166b9:	mov    rdx,QWORD PTR [rsp]
   166bd:	lea    r8,[rdi+0x1]
   166c1:	add    rbp,0x20
   166c5:	cmp    rdi,0x1e
   166c9:	ja     166db <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6db>
   166cb:	add    rax,0x6
   166cf:	mov    rdi,r8
   166d2:	cmp    r8,r12
   166d5:	jb     16650 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x650>
   166db:	cmp    r8,0x20
   166df:	jne    1ac21 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c21>
   166e5:	mov    QWORD PTR [rsp+0x100],r8
   166ed:	add    r9,0x3f
   166f1:	mov    ebx,0x20
   166f6:	mov    r14d,0x10
   166fc:	mov    r8,r9
   166ff:	mov    QWORD PTR [rsp+0x510],r9
   16707:	mov    QWORD PTR [rsp+0x640],r12
   1670f:	mov    r13,QWORD PTR [rsp+r14*1+0x100]
   16717:	mov    rdx,QWORD PTR [rsp+r14*1+0x108]
   1671f:	cmp    r13,rdx
   16722:	je     1675c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x75c>
   16724:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   16730:	sub    rdx,r13
   16733:	cmp    rdx,r8
   16736:	cmovb  r8,rdx
   1673a:	add    r14,0x20
   1673e:	cmp    r14,0x410
   16745:	je     167b8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7b8>
   16747:	mov    r13,QWORD PTR [rsp+r14*1+0x100]
   1674f:	mov    rdx,QWORD PTR [rsp+r14*1+0x108]
   16757:	cmp    r13,rdx
   1675a:	jne    16730 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x730>
   1675c:	mov    rax,QWORD PTR [rsp]
   16760:	mov    rsi,QWORD PTR [rax+0x40]
   16764:	mov    rdi,QWORD PTR [rsp+r14*1+0xf8]
   1676c:	cmp    rdi,rsi
   1676f:	jae    1d29c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x729c>
   16775:	mov    rcx,QWORD PTR [rax+0x38]
   16779:	lea    rsi,[rdi+rdi*2]
   1677d:	mov    rdx,QWORD PTR [rcx+rsi*8]
   16781:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   16786:	sub    r13,rax
   16789:	mov    QWORD PTR [rcx+rsi*8+0x10],r13
   1678e:	mov    rdi,rdx
   16791:	sub    rdi,r13
   16794:	cmp    rdx,rdi
   16797:	ja     19fd4 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3fd4>
   1679d:	add    r13,rax
   167a0:	mov    QWORD PTR [rsp+r14*1+0x100],r13
   167a8:	add    rdx,rax
   167ab:	mov    QWORD PTR [rsp+r14*1+0x108],rdx
   167b3:	jmp    16730 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x730>
   167b8:	mov    QWORD PTR [rsp+0x8],rbx
   167bd:	mov    rax,QWORD PTR [rsp+0x110]
   167c5:	mov    rcx,QWORD PTR [rsp+0x130]
   167cd:	mov    r12,QWORD PTR [rsp+0x150]
   167d5:	mov    rbx,QWORD PTR [rsp+0x170]
   167dd:	mov    r13,QWORD PTR [rsp+0x190]
   167e5:	mov    r14,QWORD PTR [rsp+0x1b0]
   167ed:	mov    r11,QWORD PTR [rsp+0x1d0]
   167f5:	mov    r10,QWORD PTR [rsp+0x1f0]
   167fd:	mov    r9,QWORD PTR [rsp+0x210]
   16805:	mov    QWORD PTR [rsp+0x518],r8
   1680d:	mov    r8,QWORD PTR [rsp+0x230]
   16815:	mov    rdi,QWORD PTR [rsp+0x250]
   1681d:	mov    rsi,QWORD PTR [rsp+0x270]
   16825:	mov    rdx,QWORD PTR [rsp+0x290]
   1682d:	mov    QWORD PTR [rsp+0x40],rdx
   16832:	mov    rdx,QWORD PTR [rsp+0x2b0]
   1683a:	mov    QWORD PTR [rsp+0x38],rdx
   1683f:	mov    rdx,QWORD PTR [rsp+0x2d0]
   16847:	mov    QWORD PTR [rsp+0x30],rdx
   1684c:	mov    QWORD PTR [rsp+0x10],rax
   16851:	mov    QWORD PTR [rsp+0x540],rax
   16859:	mov    QWORD PTR [rsp+0xf8],rcx
   16861:	mov    QWORD PTR [rsp+0x548],rcx
   16869:	mov    QWORD PTR [rsp+0xf0],r12
   16871:	mov    QWORD PTR [rsp+0x550],r12
   16879:	mov    QWORD PTR [rsp+0xe8],rbx
   16881:	mov    QWORD PTR [rsp+0x558],rbx
   16889:	mov    QWORD PTR [rsp+0xe0],r13
   16891:	mov    QWORD PTR [rsp+0x560],r13
   16899:	mov    QWORD PTR [rsp+0xd8],r14
   168a1:	mov    QWORD PTR [rsp+0x568],r14
   168a9:	mov    QWORD PTR [rsp+0xd0],r11
   168b1:	mov    QWORD PTR [rsp+0x570],r11
   168b9:	mov    QWORD PTR [rsp+0xc0],r10
   168c1:	mov    QWORD PTR [rsp+0x578],r10
   168c9:	mov    QWORD PTR [rsp+0xb0],r9
   168d1:	mov    QWORD PTR [rsp+0x580],r9
   168d9:	mov    QWORD PTR [rsp+0xa0],r8
   168e1:	mov    QWORD PTR [rsp+0x588],r8
   168e9:	mov    QWORD PTR [rsp+0x98],rdi
   168f1:	mov    QWORD PTR [rsp+0x590],rdi
   168f9:	mov    QWORD PTR [rsp+0x90],rsi
   16901:	mov    QWORD PTR [rsp+0x598],rsi
   16909:	mov    rax,QWORD PTR [rsp+0x40]
   1690e:	mov    QWORD PTR [rsp+0x5a0],rax
   16916:	mov    rax,QWORD PTR [rsp+0x38]
   1691b:	mov    QWORD PTR [rsp+0x5a8],rax
   16923:	mov    rax,QWORD PTR [rsp+0x30]
   16928:	mov    QWORD PTR [rsp+0x5b0],rax
   16930:	mov    rax,QWORD PTR [rsp+0x2f0]
   16938:	mov    QWORD PTR [rsp+0x88],rax
   16940:	mov    QWORD PTR [rsp+0x5b8],rax
   16948:	mov    rax,QWORD PTR [rsp+0x310]
   16950:	mov    QWORD PTR [rsp+0x80],rax
   16958:	mov    QWORD PTR [rsp+0x5c0],rax
   16960:	mov    rax,QWORD PTR [rsp+0x330]
   16968:	mov    QWORD PTR [rsp+0x78],rax
   1696d:	mov    QWORD PTR [rsp+0x5c8],rax
   16975:	mov    rax,QWORD PTR [rsp+0x350]
   1697d:	mov    QWORD PTR [rsp+0x70],rax
   16982:	mov    QWORD PTR [rsp+0x5d0],rax
   1698a:	mov    rax,QWORD PTR [rsp+0x370]
   16992:	mov    QWORD PTR [rsp+0x68],rax
   16997:	mov    QWORD PTR [rsp+0x5d8],rax
   1699f:	mov    rax,QWORD PTR [rsp+0x390]
   169a7:	mov    QWORD PTR [rsp+0x60],rax
   169ac:	mov    QWORD PTR [rsp+0x5e0],rax
   169b4:	mov    rax,QWORD PTR [rsp+0x3b0]
   169bc:	mov    QWORD PTR [rsp+0x58],rax
   169c1:	mov    QWORD PTR [rsp+0x5e8],rax
   169c9:	mov    rax,QWORD PTR [rsp+0x3d0]
   169d1:	mov    QWORD PTR [rsp+0x50],rax
   169d6:	mov    QWORD PTR [rsp+0x5f0],rax
   169de:	mov    rax,QWORD PTR [rsp+0x3f0]
   169e6:	mov    QWORD PTR [rsp+0x48],rax
   169eb:	mov    QWORD PTR [rsp+0x5f8],rax
   169f3:	mov    r11,QWORD PTR [rsp+0x410]
   169fb:	mov    QWORD PTR [rsp+0x600],r11
   16a03:	mov    r10,QWORD PTR [rsp+0x430]
   16a0b:	mov    QWORD PTR [rsp+0x608],r10
   16a13:	mov    rax,QWORD PTR [rsp+0x450]
   16a1b:	mov    QWORD PTR [rsp+0xc8],rax
   16a23:	mov    QWORD PTR [rsp+0x610],rax
   16a2b:	mov    rax,QWORD PTR [rsp+0x470]
   16a33:	mov    QWORD PTR [rsp+0xb8],rax
   16a3b:	mov    QWORD PTR [rsp+0x618],rax
   16a43:	mov    rax,QWORD PTR [rsp+0x490]
   16a4b:	mov    QWORD PTR [rsp+0xa8],rax
   16a53:	mov    QWORD PTR [rsp+0x620],rax
   16a5b:	mov    r9,QWORD PTR [rsp+0x4b0]
   16a63:	mov    QWORD PTR [rsp+0x628],r9
   16a6b:	mov    rax,QWORD PTR [rsp+0x4d0]
   16a73:	mov    QWORD PTR [rsp+0x28],rax
   16a78:	mov    QWORD PTR [rsp+0x630],rax
   16a80:	mov    rax,QWORD PTR [rsp+0x4f0]
   16a88:	mov    QWORD PTR [rsp+0x20],rax
   16a8d:	mov    QWORD PTR [rsp+0x638],rax
   16a95:	cmp    QWORD PTR [rsp+0x518],0x0
   16a9e:	je     19d7a <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3d7a>
   16aa4:	xor    r8d,r8d
   16aa7:	mov    rdx,QWORD PTR [rsp]
   16aab:	mov    rbx,QWORD PTR [rsp+0x8]
   16ab0:	mov    r12,QWORD PTR [rsp+0x640]
   16ab8:	mov    r14d,DWORD PTR [rsp+0x120]
   16ac0:	mov    eax,DWORD PTR [r15+r14*4]
   16ac4:	test   eax,eax
   16ac6:	mov    QWORD PTR [rsp+0x8],r11
   16acb:	mov    QWORD PTR [rsp+0x18],r10
   16ad0:	js     16b00 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xb00>
   16ad2:	mov    ecx,eax
   16ad4:	shr    ecx,0x8
   16ad7:	mov    DWORD PTR [rsp+0x120],ecx
   16ade:	prefetcht0 BYTE PTR [r15+rcx*4]
   16ae3:	mov    rcx,QWORD PTR [rsp+0x10]
   16ae8:	mov    BYTE PTR [rcx+r8*1],al
   16aec:	jmp    16c64 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xc64>
   16af1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   16b00:	mov    rax,r8
   16b03:	mov    r8,r9
   16b06:	mov    r9,QWORD PTR [rsp+0x10]
   16b0b:	add    r9,rax
   16b0e:	mov    QWORD PTR [rsp+0x110],r9
   16b16:	mov    rsi,QWORD PTR [rdx+0x40]
   16b1a:	mov    rdi,QWORD PTR [rsp+0x108]
   16b22:	cmp    rdi,rsi
   16b25:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   16b2b:	mov    r13,rax
   16b2e:	mov    rax,QWORD PTR [rdx+0x38]
   16b32:	mov    rcx,QWORD PTR [rdx+0x50]
   16b36:	mov    QWORD PTR [rsp+0x10],rcx
   16b3b:	lea    rcx,[rdi+rdi*2]
   16b3f:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   16b44:	mov    rsi,QWORD PTR [rdx+0x58]
   16b48:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   16b4d:	mov    rax,rdx
   16b50:	mov    rdx,QWORD PTR [rdx+0x10]
   16b54:	test   rdx,rdx
   16b57:	je     16b6e <mibz2::bwt::room_segments_unrolled::<32, 1>+0xb6e>
   16b59:	mov    r11,QWORD PTR [rax+0x8]
   16b5d:	cmp    rdx,0x1
   16b61:	mov    r9,r8
   16b64:	jne    16b7d <mibz2::bwt::room_segments_unrolled::<32, 1>+0xb7d>
   16b66:	xor    r10d,r10d
   16b69:	mov    r8,r13
   16b6c:	jmp    16bad <mibz2::bwt::room_segments_unrolled::<32, 1>+0xbad>
   16b6e:	mov    r10,0xffffffffffffffff
   16b75:	mov    r9,r8
   16b78:	mov    r8,r13
   16b7b:	jmp    16bc1 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xbc1>
   16b7d:	xor    r10d,r10d
   16b80:	mov    r8,r13
   16b83:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   16b90:	mov    rax,r10
   16b93:	mov    rcx,rdx
   16b96:	shr    rcx,1
   16b99:	add    r10,rcx
   16b9c:	cmp    DWORD PTR [r11+r10*4],r14d
   16ba0:	cmova  r10,rax
   16ba4:	sub    rdx,rcx
   16ba7:	cmp    rdx,0x1
   16bab:	ja     16b90 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xb90>
   16bad:	cmp    DWORD PTR [r11+r10*4],r14d
   16bb1:	mov    rax,0xffffffffffffffff
   16bb8:	cmovne r10,rax
   16bbc:	mov    r11,QWORD PTR [rsp+0x8]
   16bc1:	cmp    rdi,rsi
   16bc4:	mov    rdx,QWORD PTR [rsp]
   16bc8:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   16bce:	mov    rax,QWORD PTR [rsp+0x10]
   16bd3:	mov    QWORD PTR [rax+rdi*8],r10
   16bd7:	cmp    rbx,r12
   16bda:	je     1a04c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x404c>
   16be0:	mov    rsi,QWORD PTR [rdx+0x40]
   16be4:	cmp    rbx,rsi
   16be7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   16bed:	mov    rsi,QWORD PTR [rdx+0x10]
   16bf1:	cmp    rbx,rsi
   16bf4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   16bfa:	mov    rax,QWORD PTR [rdx+0x38]
   16bfe:	mov    rcx,QWORD PTR [rdx+0x8]
   16c02:	lea    rdx,[rbx+rbx*2]
   16c06:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   16c0b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   16c0e:	mov    ecx,DWORD PTR [r15+rcx*4]
   16c12:	mov    BYTE PTR [rsi],cl
   16c14:	shr    ecx,0x8
   16c17:	and    ecx,0x7fffff
   16c1d:	lea    rdi,[rsi+0x1]
   16c21:	mov    rax,QWORD PTR [rax+rdx*8]
   16c25:	add    rax,rsi
   16c28:	mov    QWORD PTR [rsp+0x108],rbx
   16c30:	mov    QWORD PTR [rsp+0x110],rdi
   16c38:	mov    QWORD PTR [rsp+0x118],rax
   16c40:	mov    DWORD PTR [rsp+0x120],ecx
   16c47:	prefetcht0 BYTE PTR [r15+rcx*4]
   16c4c:	inc    rbx
   16c4f:	mov    rax,r8
   16c52:	not    rax
   16c55:	lea    rcx,[rsi+rax*1]
   16c59:	inc    rcx
   16c5c:	mov    QWORD PTR [rsp+0x540],rcx
   16c64:	mov    QWORD PTR [rsp+0x10],rcx
   16c69:	lea    rax,[r8+0x1]
   16c6d:	mov    QWORD PTR [rsp+0x520],rax
   16c75:	mov    r14d,DWORD PTR [rsp+0x140]
   16c7d:	mov    eax,DWORD PTR [r15+r14*4]
   16c81:	test   eax,eax
   16c83:	mov    r10,r9
   16c86:	js     170f0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x10f0>
   16c8c:	mov    ecx,eax
   16c8e:	shr    ecx,0x8
   16c91:	mov    DWORD PTR [rsp+0x140],ecx
   16c98:	prefetcht0 BYTE PTR [r15+rcx*4]
   16c9d:	mov    rcx,QWORD PTR [rsp+0xf8]
   16ca5:	mov    BYTE PTR [rcx+r8*1],al
   16ca9:	mov    r14d,DWORD PTR [rsp+0x160]
   16cb1:	mov    eax,DWORD PTR [r15+r14*4]
   16cb5:	test   eax,eax
   16cb7:	js     1725e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x125e>
   16cbd:	mov    ecx,eax
   16cbf:	shr    ecx,0x8
   16cc2:	mov    DWORD PTR [rsp+0x160],ecx
   16cc9:	prefetcht0 BYTE PTR [r15+rcx*4]
   16cce:	mov    rcx,QWORD PTR [rsp+0xf0]
   16cd6:	mov    BYTE PTR [rcx+r8*1],al
   16cda:	mov    r14d,DWORD PTR [rsp+0x180]
   16ce2:	mov    eax,DWORD PTR [r15+r14*4]
   16ce6:	test   eax,eax
   16ce8:	js     173ce <mibz2::bwt::room_segments_unrolled::<32, 1>+0x13ce>
   16cee:	mov    ecx,eax
   16cf0:	shr    ecx,0x8
   16cf3:	mov    DWORD PTR [rsp+0x180],ecx
   16cfa:	prefetcht0 BYTE PTR [r15+rcx*4]
   16cff:	mov    rcx,QWORD PTR [rsp+0xe8]
   16d07:	mov    BYTE PTR [rcx+r8*1],al
   16d0b:	mov    r14d,DWORD PTR [rsp+0x1a0]
   16d13:	mov    eax,DWORD PTR [r15+r14*4]
   16d17:	test   eax,eax
   16d19:	js     1753e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x153e>
   16d1f:	mov    ecx,eax
   16d21:	shr    ecx,0x8
   16d24:	mov    DWORD PTR [rsp+0x1a0],ecx
   16d2b:	prefetcht0 BYTE PTR [r15+rcx*4]
   16d30:	mov    rcx,QWORD PTR [rsp+0xe0]
   16d38:	mov    BYTE PTR [rcx+r8*1],al
   16d3c:	mov    r14d,DWORD PTR [rsp+0x1c0]
   16d44:	mov    eax,DWORD PTR [r15+r14*4]
   16d48:	test   eax,eax
   16d4a:	js     176a4 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x16a4>
   16d50:	mov    ecx,eax
   16d52:	shr    ecx,0x8
   16d55:	mov    DWORD PTR [rsp+0x1c0],ecx
   16d5c:	prefetcht0 BYTE PTR [r15+rcx*4]
   16d61:	mov    rcx,QWORD PTR [rsp+0xd8]
   16d69:	mov    BYTE PTR [rcx+r8*1],al
   16d6d:	mov    r14d,DWORD PTR [rsp+0x1e0]
   16d75:	mov    eax,DWORD PTR [r15+r14*4]
   16d79:	test   eax,eax
   16d7b:	js     17809 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1809>
   16d81:	mov    ecx,eax
   16d83:	shr    ecx,0x8
   16d86:	mov    DWORD PTR [rsp+0x1e0],ecx
   16d8d:	prefetcht0 BYTE PTR [r15+rcx*4]
   16d92:	mov    rcx,QWORD PTR [rsp+0xd0]
   16d9a:	mov    BYTE PTR [rcx+r8*1],al
   16d9e:	mov    r14d,DWORD PTR [rsp+0x200]
   16da6:	mov    eax,DWORD PTR [r15+r14*4]
   16daa:	test   eax,eax
   16dac:	js     17979 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1979>
   16db2:	mov    ecx,eax
   16db4:	shr    ecx,0x8
   16db7:	mov    DWORD PTR [rsp+0x200],ecx
   16dbe:	prefetcht0 BYTE PTR [r15+rcx*4]
   16dc3:	mov    rcx,QWORD PTR [rsp+0xc0]
   16dcb:	mov    BYTE PTR [rcx+r8*1],al
   16dcf:	mov    r14d,DWORD PTR [rsp+0x220]
   16dd7:	mov    eax,DWORD PTR [r15+r14*4]
   16ddb:	test   eax,eax
   16ddd:	js     17ae9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1ae9>
   16de3:	mov    ecx,eax
   16de5:	shr    ecx,0x8
   16de8:	mov    DWORD PTR [rsp+0x220],ecx
   16def:	prefetcht0 BYTE PTR [r15+rcx*4]
   16df4:	mov    rcx,QWORD PTR [rsp+0xb0]
   16dfc:	mov    BYTE PTR [rcx+r8*1],al
   16e00:	mov    r14d,DWORD PTR [rsp+0x240]
   16e08:	mov    eax,DWORD PTR [r15+r14*4]
   16e0c:	test   eax,eax
   16e0e:	js     17c59 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1c59>
   16e14:	mov    ecx,eax
   16e16:	shr    ecx,0x8
   16e19:	mov    DWORD PTR [rsp+0x240],ecx
   16e20:	prefetcht0 BYTE PTR [r15+rcx*4]
   16e25:	mov    rcx,QWORD PTR [rsp+0xa0]
   16e2d:	mov    BYTE PTR [rcx+r8*1],al
   16e31:	mov    r14d,DWORD PTR [rsp+0x260]
   16e39:	mov    eax,DWORD PTR [r15+r14*4]
   16e3d:	test   eax,eax
   16e3f:	js     17dc9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1dc9>
   16e45:	mov    ecx,eax
   16e47:	shr    ecx,0x8
   16e4a:	mov    DWORD PTR [rsp+0x260],ecx
   16e51:	prefetcht0 BYTE PTR [r15+rcx*4]
   16e56:	mov    rcx,QWORD PTR [rsp+0x98]
   16e5e:	mov    BYTE PTR [rcx+r8*1],al
   16e62:	mov    r14d,DWORD PTR [rsp+0x280]
   16e6a:	mov    eax,DWORD PTR [r15+r14*4]
   16e6e:	test   eax,eax
   16e70:	js     17f39 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1f39>
   16e76:	mov    ecx,eax
   16e78:	shr    ecx,0x8
   16e7b:	mov    DWORD PTR [rsp+0x280],ecx
   16e82:	prefetcht0 BYTE PTR [r15+rcx*4]
   16e87:	mov    rcx,QWORD PTR [rsp+0x90]
   16e8f:	mov    BYTE PTR [rcx+r8*1],al
   16e93:	mov    r14d,DWORD PTR [rsp+0x2a0]
   16e9b:	mov    eax,DWORD PTR [r15+r14*4]
   16e9f:	test   eax,eax
   16ea1:	js     180a9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x20a9>
   16ea7:	mov    ecx,eax
   16ea9:	shr    ecx,0x8
   16eac:	mov    DWORD PTR [rsp+0x2a0],ecx
   16eb3:	prefetcht0 BYTE PTR [r15+rcx*4]
   16eb8:	mov    rcx,QWORD PTR [rsp+0x40]
   16ebd:	mov    BYTE PTR [rcx+r8*1],al
   16ec1:	mov    r14d,DWORD PTR [rsp+0x2c0]
   16ec9:	mov    eax,DWORD PTR [r15+r14*4]
   16ecd:	test   eax,eax
   16ecf:	js     18216 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2216>
   16ed5:	mov    ecx,eax
   16ed7:	shr    ecx,0x8
   16eda:	mov    DWORD PTR [rsp+0x2c0],ecx
   16ee1:	prefetcht0 BYTE PTR [r15+rcx*4]
   16ee6:	mov    rcx,QWORD PTR [rsp+0x38]
   16eeb:	mov    BYTE PTR [rcx+r8*1],al
   16eef:	mov    r14d,DWORD PTR [rsp+0x2e0]
   16ef7:	mov    eax,DWORD PTR [r15+r14*4]
   16efb:	test   eax,eax
   16efd:	js     1836c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x236c>
   16f03:	mov    ecx,eax
   16f05:	shr    ecx,0x8
   16f08:	mov    DWORD PTR [rsp+0x2e0],ecx
   16f0f:	prefetcht0 BYTE PTR [r15+rcx*4]
   16f14:	mov    rcx,QWORD PTR [rsp+0x30]
   16f19:	mov    BYTE PTR [rcx+r8*1],al
   16f1d:	mov    r14d,DWORD PTR [rsp+0x300]
   16f25:	mov    eax,DWORD PTR [r15+r14*4]
   16f29:	test   eax,eax
   16f2b:	js     184cc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x24cc>
   16f31:	mov    ecx,eax
   16f33:	shr    ecx,0x8
   16f36:	mov    DWORD PTR [rsp+0x300],ecx
   16f3d:	prefetcht0 BYTE PTR [r15+rcx*4]
   16f42:	mov    rcx,QWORD PTR [rsp+0x88]
   16f4a:	mov    BYTE PTR [rcx+r8*1],al
   16f4e:	mov    r14d,DWORD PTR [rsp+0x320]
   16f56:	mov    eax,DWORD PTR [r15+r14*4]
   16f5a:	test   eax,eax
   16f5c:	js     1862f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x262f>
   16f62:	mov    ecx,eax
   16f64:	shr    ecx,0x8
   16f67:	mov    DWORD PTR [rsp+0x320],ecx
   16f6e:	prefetcht0 BYTE PTR [r15+rcx*4]
   16f73:	mov    rcx,QWORD PTR [rsp+0x80]
   16f7b:	mov    BYTE PTR [rcx+r8*1],al
   16f7f:	mov    r14d,DWORD PTR [rsp+0x340]
   16f87:	mov    eax,DWORD PTR [r15+r14*4]
   16f8b:	test   eax,eax
   16f8d:	js     1878f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x278f>
   16f93:	mov    ecx,eax
   16f95:	shr    ecx,0x8
   16f98:	mov    DWORD PTR [rsp+0x340],ecx
   16f9f:	prefetcht0 BYTE PTR [r15+rcx*4]
   16fa4:	mov    rcx,QWORD PTR [rsp+0x78]
   16fa9:	mov    BYTE PTR [rcx+r8*1],al
   16fad:	mov    r14d,DWORD PTR [rsp+0x360]
   16fb5:	mov    eax,DWORD PTR [r15+r14*4]
   16fb9:	test   eax,eax
   16fbb:	js     188ec <mibz2::bwt::room_segments_unrolled::<32, 1>+0x28ec>
   16fc1:	mov    ecx,eax
   16fc3:	shr    ecx,0x8
   16fc6:	mov    DWORD PTR [rsp+0x360],ecx
   16fcd:	prefetcht0 BYTE PTR [r15+rcx*4]
   16fd2:	mov    rcx,QWORD PTR [rsp+0x70]
   16fd7:	mov    BYTE PTR [rcx+r8*1],al
   16fdb:	mov    r14d,DWORD PTR [rsp+0x380]
   16fe3:	mov    eax,DWORD PTR [r15+r14*4]
   16fe7:	test   eax,eax
   16fe9:	js     18a4c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2a4c>
   16fef:	mov    ecx,eax
   16ff1:	shr    ecx,0x8
   16ff4:	mov    DWORD PTR [rsp+0x380],ecx
   16ffb:	prefetcht0 BYTE PTR [r15+rcx*4]
   17000:	mov    rcx,QWORD PTR [rsp+0x68]
   17005:	mov    BYTE PTR [rcx+r8*1],al
   17009:	mov    r14d,DWORD PTR [rsp+0x3a0]
   17011:	mov    eax,DWORD PTR [r15+r14*4]
   17015:	test   eax,eax
   17017:	js     18bac <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2bac>
   1701d:	mov    ecx,eax
   1701f:	shr    ecx,0x8
   17022:	mov    DWORD PTR [rsp+0x3a0],ecx
   17029:	prefetcht0 BYTE PTR [r15+rcx*4]
   1702e:	mov    rcx,QWORD PTR [rsp+0x60]
   17033:	mov    BYTE PTR [rcx+r8*1],al
   17037:	mov    r14d,DWORD PTR [rsp+0x3c0]
   1703f:	mov    eax,DWORD PTR [r15+r14*4]
   17043:	test   eax,eax
   17045:	js     18d0c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2d0c>
   1704b:	mov    ecx,eax
   1704d:	shr    ecx,0x8
   17050:	mov    DWORD PTR [rsp+0x3c0],ecx
   17057:	prefetcht0 BYTE PTR [r15+rcx*4]
   1705c:	mov    rcx,QWORD PTR [rsp+0x58]
   17061:	mov    BYTE PTR [rcx+r8*1],al
   17065:	mov    r14d,DWORD PTR [rsp+0x3e0]
   1706d:	mov    eax,DWORD PTR [r15+r14*4]
   17071:	test   eax,eax
   17073:	js     18e6c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2e6c>
   17079:	mov    ecx,eax
   1707b:	shr    ecx,0x8
   1707e:	mov    DWORD PTR [rsp+0x3e0],ecx
   17085:	prefetcht0 BYTE PTR [r15+rcx*4]
   1708a:	mov    rcx,QWORD PTR [rsp+0x50]
   1708f:	mov    BYTE PTR [rcx+r8*1],al
   17093:	mov    r14d,DWORD PTR [rsp+0x400]
   1709b:	mov    eax,DWORD PTR [r15+r14*4]
   1709f:	test   eax,eax
   170a1:	js     18fcc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2fcc>
   170a7:	mov    ecx,eax
   170a9:	shr    ecx,0x8
   170ac:	mov    DWORD PTR [rsp+0x400],ecx
   170b3:	prefetcht0 BYTE PTR [r15+rcx*4]
   170b8:	mov    rcx,QWORD PTR [rsp+0x48]
   170bd:	mov    BYTE PTR [rcx+r8*1],al
   170c1:	mov    r14d,DWORD PTR [rsp+0x420]
   170c9:	mov    eax,DWORD PTR [r15+r14*4]
   170cd:	test   eax,eax
   170cf:	js     1912c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x312c>
   170d5:	mov    ecx,eax
   170d7:	shr    ecx,0x8
   170da:	mov    DWORD PTR [rsp+0x420],ecx
   170e1:	prefetcht0 BYTE PTR [r15+rcx*4]
   170e6:	mov    BYTE PTR [r11+r8*1],al
   170ea:	jmp    1925f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x325f>
   170ef:	nop
   170f0:	mov    r9,QWORD PTR [rsp+0xf8]
   170f8:	add    r9,r8
   170fb:	mov    QWORD PTR [rsp+0x130],r9
   17103:	mov    rdx,QWORD PTR [rsp]
   17107:	mov    rsi,QWORD PTR [rdx+0x40]
   1710b:	mov    rdi,QWORD PTR [rsp+0x128]
   17113:	cmp    rdi,rsi
   17116:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1711c:	mov    rax,QWORD PTR [rdx+0x38]
   17120:	mov    r13,QWORD PTR [rdx+0x50]
   17124:	lea    rcx,[rdi+rdi*2]
   17128:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   1712d:	mov    rsi,QWORD PTR [rdx+0x58]
   17131:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   17136:	mov    rcx,QWORD PTR [rdx+0x10]
   1713a:	test   rcx,rcx
   1713d:	je     17194 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1194>
   1713f:	mov    r11,QWORD PTR [rdx+0x8]
   17143:	xor    r9d,r9d
   17146:	cmp    rcx,0x1
   1714a:	je     1716d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x116d>
   1714c:	nop    DWORD PTR [rax+0x0]
   17150:	mov    rax,r9
   17153:	mov    rdx,rcx
   17156:	shr    rdx,1
   17159:	add    r9,rdx
   1715c:	cmp    DWORD PTR [r11+r9*4],r14d
   17160:	cmova  r9,rax
   17164:	sub    rcx,rdx
   17167:	cmp    rcx,0x1
   1716b:	ja     17150 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1150>
   1716d:	cmp    DWORD PTR [r11+r9*4],r14d
   17171:	mov    rax,0xffffffffffffffff
   17178:	cmovne r9,rax
   1717c:	mov    rdx,QWORD PTR [rsp]
   17180:	cmp    rdi,rsi
   17183:	mov    r11,QWORD PTR [rsp+0x8]
   17188:	mov    rcx,QWORD PTR [rsp+0x10]
   1718d:	jb     171ae <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11ae>
   1718f:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17194:	mov    r9,0xffffffffffffffff
   1719b:	cmp    rdi,rsi
   1719e:	mov    r11,QWORD PTR [rsp+0x8]
   171a3:	mov    rcx,QWORD PTR [rsp+0x10]
   171a8:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   171ae:	mov    QWORD PTR [r13+rdi*8+0x0],r9
   171b3:	cmp    rbx,r12
   171b6:	je     1a247 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4247>
   171bc:	mov    rsi,QWORD PTR [rdx+0x40]
   171c0:	cmp    rbx,rsi
   171c3:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   171c9:	mov    rsi,QWORD PTR [rdx+0x10]
   171cd:	cmp    rbx,rsi
   171d0:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   171d6:	mov    r9,r10
   171d9:	mov    rax,QWORD PTR [rdx+0x38]
   171dd:	mov    rcx,QWORD PTR [rdx+0x8]
   171e1:	lea    rdx,[rbx+rbx*2]
   171e5:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   171ea:	mov    ecx,DWORD PTR [rcx+rbx*4]
   171ed:	mov    ecx,DWORD PTR [r15+rcx*4]
   171f1:	mov    BYTE PTR [rsi],cl
   171f3:	shr    ecx,0x8
   171f6:	and    ecx,0x7fffff
   171fc:	lea    rdi,[rsi+0x1]
   17200:	mov    rax,QWORD PTR [rax+rdx*8]
   17204:	add    rax,rsi
   17207:	mov    QWORD PTR [rsp+0x128],rbx
   1720f:	mov    QWORD PTR [rsp+0x130],rdi
   17217:	mov    QWORD PTR [rsp+0x138],rax
   1721f:	mov    DWORD PTR [rsp+0x140],ecx
   17226:	prefetcht0 BYTE PTR [r15+rcx*4]
   1722b:	inc    rbx
   1722e:	mov    rax,r8
   17231:	not    rax
   17234:	add    rax,rsi
   17237:	inc    rax
   1723a:	mov    QWORD PTR [rsp+0xf8],rax
   17242:	mov    QWORD PTR [rsp+0x548],rax
   1724a:	mov    r14d,DWORD PTR [rsp+0x160]
   17252:	mov    eax,DWORD PTR [r15+r14*4]
   17256:	test   eax,eax
   17258:	jns    16cbd <mibz2::bwt::room_segments_unrolled::<32, 1>+0xcbd>
   1725e:	mov    r9,QWORD PTR [rsp+0xf0]
   17266:	add    r9,r8
   17269:	mov    QWORD PTR [rsp+0x150],r9
   17271:	mov    rdx,QWORD PTR [rsp]
   17275:	mov    rsi,QWORD PTR [rdx+0x40]
   17279:	mov    rdi,QWORD PTR [rsp+0x148]
   17281:	cmp    rdi,rsi
   17284:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1728a:	mov    rax,QWORD PTR [rdx+0x38]
   1728e:	mov    r13,QWORD PTR [rdx+0x50]
   17292:	lea    rcx,[rdi+rdi*2]
   17296:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   1729b:	mov    rsi,QWORD PTR [rdx+0x58]
   1729f:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   172a4:	mov    r9,QWORD PTR [rdx+0x10]
   172a8:	test   r9,r9
   172ab:	je     17300 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1300>
   172ad:	mov    r11,QWORD PTR [rdx+0x8]
   172b1:	xor    edx,edx
   172b3:	cmp    r9,0x1
   172b7:	je     172dd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x12dd>
   172b9:	nop    DWORD PTR [rax+0x0]
   172c0:	mov    rax,rdx
   172c3:	mov    rcx,r9
   172c6:	shr    rcx,1
   172c9:	add    rdx,rcx
   172cc:	cmp    DWORD PTR [r11+rdx*4],r14d
   172d0:	cmova  rdx,rax
   172d4:	sub    r9,rcx
   172d7:	cmp    r9,0x1
   172db:	ja     172c0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x12c0>
   172dd:	cmp    DWORD PTR [r11+rdx*4],r14d
   172e1:	mov    rax,0xffffffffffffffff
   172e8:	cmovne rdx,rax
   172ec:	cmp    rdi,rsi
   172ef:	mov    r11,QWORD PTR [rsp+0x8]
   172f4:	mov    rcx,QWORD PTR [rsp+0x10]
   172f9:	jb     1731a <mibz2::bwt::room_segments_unrolled::<32, 1>+0x131a>
   172fb:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17300:	mov    rdx,0xffffffffffffffff
   17307:	cmp    rdi,rsi
   1730a:	mov    r11,QWORD PTR [rsp+0x8]
   1730f:	mov    rcx,QWORD PTR [rsp+0x10]
   17314:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1731a:	mov    r9,r10
   1731d:	mov    QWORD PTR [r13+rdi*8+0x0],rdx
   17322:	cmp    rbx,r12
   17325:	je     1a466 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4466>
   1732b:	mov    rcx,QWORD PTR [rsp]
   1732f:	mov    rsi,QWORD PTR [rcx+0x40]
   17333:	cmp    rbx,rsi
   17336:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1733c:	mov    rsi,QWORD PTR [rcx+0x10]
   17340:	cmp    rbx,rsi
   17343:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17349:	mov    rax,QWORD PTR [rcx+0x38]
   1734d:	mov    rcx,QWORD PTR [rcx+0x8]
   17351:	lea    rdx,[rbx+rbx*2]
   17355:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1735a:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1735d:	mov    ecx,DWORD PTR [r15+rcx*4]
   17361:	mov    BYTE PTR [rsi],cl
   17363:	shr    ecx,0x8
   17366:	and    ecx,0x7fffff
   1736c:	lea    rdi,[rsi+0x1]
   17370:	mov    rax,QWORD PTR [rax+rdx*8]
   17374:	add    rax,rsi
   17377:	mov    QWORD PTR [rsp+0x148],rbx
   1737f:	mov    QWORD PTR [rsp+0x150],rdi
   17387:	mov    QWORD PTR [rsp+0x158],rax
   1738f:	mov    DWORD PTR [rsp+0x160],ecx
   17396:	prefetcht0 BYTE PTR [r15+rcx*4]
   1739b:	inc    rbx
   1739e:	mov    rax,r8
   173a1:	not    rax
   173a4:	add    rax,rsi
   173a7:	inc    rax
   173aa:	mov    QWORD PTR [rsp+0xf0],rax
   173b2:	mov    QWORD PTR [rsp+0x550],rax
   173ba:	mov    r14d,DWORD PTR [rsp+0x180]
   173c2:	mov    eax,DWORD PTR [r15+r14*4]
   173c6:	test   eax,eax
   173c8:	jns    16cee <mibz2::bwt::room_segments_unrolled::<32, 1>+0xcee>
   173ce:	mov    r11,QWORD PTR [rsp+0xe8]
   173d6:	add    r11,r8
   173d9:	mov    QWORD PTR [rsp+0x170],r11
   173e1:	mov    rdx,QWORD PTR [rsp]
   173e5:	mov    rsi,QWORD PTR [rdx+0x40]
   173e9:	mov    rdi,QWORD PTR [rsp+0x168]
   173f1:	cmp    rdi,rsi
   173f4:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   173fa:	mov    rax,QWORD PTR [rdx+0x38]
   173fe:	mov    r13,QWORD PTR [rdx+0x50]
   17402:	lea    rcx,[rdi+rdi*2]
   17406:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   1740b:	mov    rsi,QWORD PTR [rdx+0x58]
   1740f:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17414:	mov    rcx,QWORD PTR [rdx+0x10]
   17418:	test   rcx,rcx
   1741b:	je     17477 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1477>
   1741d:	mov    r11,QWORD PTR [rdx+0x8]
   17421:	xor    eax,eax
   17423:	cmp    rcx,0x1
   17427:	je     1744d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x144d>
   17429:	nop    DWORD PTR [rax+0x0]
   17430:	mov    rdx,rax
   17433:	mov    r9,rcx
   17436:	shr    r9,1
   17439:	add    rax,r9
   1743c:	cmp    DWORD PTR [r11+rax*4],r14d
   17440:	cmova  rax,rdx
   17444:	sub    rcx,r9
   17447:	cmp    rcx,0x1
   1744b:	ja     17430 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1430>
   1744d:	cmp    DWORD PTR [r11+rax*4],r14d
   17451:	mov    rcx,0xffffffffffffffff
   17458:	cmovne rax,rcx
   1745c:	mov    rdx,QWORD PTR [rsp]
   17460:	mov    r9,r10
   17463:	cmp    rdi,rsi
   17466:	mov    r11,QWORD PTR [rsp+0x8]
   1746b:	mov    rcx,QWORD PTR [rsp+0x10]
   17470:	jb     17491 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1491>
   17472:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17477:	mov    rax,0xffffffffffffffff
   1747e:	cmp    rdi,rsi
   17481:	mov    r11,QWORD PTR [rsp+0x8]
   17486:	mov    rcx,QWORD PTR [rsp+0x10]
   1748b:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17491:	mov    QWORD PTR [r13+rdi*8+0x0],rax
   17496:	cmp    rbx,r12
   17499:	je     1a681 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4681>
   1749f:	mov    rsi,QWORD PTR [rdx+0x40]
   174a3:	cmp    rbx,rsi
   174a6:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   174ac:	mov    rsi,QWORD PTR [rdx+0x10]
   174b0:	cmp    rbx,rsi
   174b3:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   174b9:	mov    rax,QWORD PTR [rdx+0x38]
   174bd:	mov    rcx,QWORD PTR [rdx+0x8]
   174c1:	lea    rdx,[rbx+rbx*2]
   174c5:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   174ca:	mov    ecx,DWORD PTR [rcx+rbx*4]
   174cd:	mov    ecx,DWORD PTR [r15+rcx*4]
   174d1:	mov    BYTE PTR [rsi],cl
   174d3:	shr    ecx,0x8
   174d6:	and    ecx,0x7fffff
   174dc:	lea    rdi,[rsi+0x1]
   174e0:	mov    rax,QWORD PTR [rax+rdx*8]
   174e4:	add    rax,rsi
   174e7:	mov    QWORD PTR [rsp+0x168],rbx
   174ef:	mov    QWORD PTR [rsp+0x170],rdi
   174f7:	mov    QWORD PTR [rsp+0x178],rax
   174ff:	mov    DWORD PTR [rsp+0x180],ecx
   17506:	prefetcht0 BYTE PTR [r15+rcx*4]
   1750b:	inc    rbx
   1750e:	mov    rax,r8
   17511:	not    rax
   17514:	add    rax,rsi
   17517:	inc    rax
   1751a:	mov    QWORD PTR [rsp+0xe8],rax
   17522:	mov    QWORD PTR [rsp+0x558],rax
   1752a:	mov    r14d,DWORD PTR [rsp+0x1a0]
   17532:	mov    eax,DWORD PTR [r15+r14*4]
   17536:	test   eax,eax
   17538:	jns    16d1f <mibz2::bwt::room_segments_unrolled::<32, 1>+0xd1f>
   1753e:	mov    r11,QWORD PTR [rsp+0xe0]
   17546:	add    r11,r8
   17549:	mov    QWORD PTR [rsp+0x190],r11
   17551:	mov    rdx,QWORD PTR [rsp]
   17555:	mov    rsi,QWORD PTR [rdx+0x40]
   17559:	mov    rdi,QWORD PTR [rsp+0x188]
   17561:	cmp    rdi,rsi
   17564:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1756a:	mov    rax,QWORD PTR [rdx+0x38]
   1756e:	mov    r13,QWORD PTR [rdx+0x50]
   17572:	lea    rcx,[rdi+rdi*2]
   17576:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   1757b:	mov    rsi,QWORD PTR [rdx+0x58]
   1757f:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17584:	mov    rax,QWORD PTR [rdx+0x10]
   17588:	test   rax,rax
   1758b:	je     175e2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x15e2>
   1758d:	mov    r11,QWORD PTR [rdx+0x8]
   17591:	xor    ecx,ecx
   17593:	cmp    rax,0x1
   17597:	je     175bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x15bd>
   17599:	nop    DWORD PTR [rax+0x0]
   175a0:	mov    rdx,rcx
   175a3:	mov    r9,rax
   175a6:	shr    r9,1
   175a9:	add    rcx,r9
   175ac:	cmp    DWORD PTR [r11+rcx*4],r14d
   175b0:	cmova  rcx,rdx
   175b4:	sub    rax,r9
   175b7:	cmp    rax,0x1
   175bb:	ja     175a0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x15a0>
   175bd:	cmp    DWORD PTR [r11+rcx*4],r14d
   175c1:	mov    rax,0xffffffffffffffff
   175c8:	cmovne rcx,rax
   175cc:	mov    rdx,QWORD PTR [rsp]
   175d0:	mov    r9,r10
   175d3:	cmp    rdi,rsi
   175d6:	mov    r11,QWORD PTR [rsp+0x8]
   175db:	jb     175f7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x15f7>
   175dd:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   175e2:	mov    rcx,0xffffffffffffffff
   175e9:	cmp    rdi,rsi
   175ec:	mov    r11,QWORD PTR [rsp+0x8]
   175f1:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   175f7:	mov    QWORD PTR [r13+rdi*8+0x0],rcx
   175fc:	cmp    rbx,r12
   175ff:	je     1a6cb <mibz2::bwt::room_segments_unrolled::<32, 1>+0x46cb>
   17605:	mov    rsi,QWORD PTR [rdx+0x40]
   17609:	cmp    rbx,rsi
   1760c:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17612:	mov    rsi,QWORD PTR [rdx+0x10]
   17616:	cmp    rbx,rsi
   17619:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1761f:	mov    rax,QWORD PTR [rdx+0x38]
   17623:	mov    rcx,QWORD PTR [rdx+0x8]
   17627:	lea    rdx,[rbx+rbx*2]
   1762b:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17630:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17633:	mov    ecx,DWORD PTR [r15+rcx*4]
   17637:	mov    BYTE PTR [rsi],cl
   17639:	shr    ecx,0x8
   1763c:	and    ecx,0x7fffff
   17642:	lea    rdi,[rsi+0x1]
   17646:	mov    rax,QWORD PTR [rax+rdx*8]
   1764a:	add    rax,rsi
   1764d:	mov    QWORD PTR [rsp+0x188],rbx
   17655:	mov    QWORD PTR [rsp+0x190],rdi
   1765d:	mov    QWORD PTR [rsp+0x198],rax
   17665:	mov    DWORD PTR [rsp+0x1a0],ecx
   1766c:	prefetcht0 BYTE PTR [r15+rcx*4]
   17671:	inc    rbx
   17674:	mov    rax,r8
   17677:	not    rax
   1767a:	add    rax,rsi
   1767d:	inc    rax
   17680:	mov    QWORD PTR [rsp+0xe0],rax
   17688:	mov    QWORD PTR [rsp+0x560],rax
   17690:	mov    r14d,DWORD PTR [rsp+0x1c0]
   17698:	mov    eax,DWORD PTR [r15+r14*4]
   1769c:	test   eax,eax
   1769e:	jns    16d50 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xd50>
   176a4:	mov    r11,QWORD PTR [rsp+0xd8]
   176ac:	add    r11,r8
   176af:	mov    QWORD PTR [rsp+0x1b0],r11
   176b7:	mov    rdx,QWORD PTR [rsp]
   176bb:	mov    rsi,QWORD PTR [rdx+0x40]
   176bf:	mov    rdi,QWORD PTR [rsp+0x1a8]
   176c7:	cmp    rdi,rsi
   176ca:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   176d0:	mov    rax,QWORD PTR [rdx+0x38]
   176d4:	mov    r13,QWORD PTR [rdx+0x50]
   176d8:	lea    rcx,[rdi+rdi*2]
   176dc:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   176e1:	mov    rsi,QWORD PTR [rdx+0x58]
   176e5:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   176ea:	mov    rax,QWORD PTR [rdx+0x10]
   176ee:	test   rax,rax
   176f1:	je     17742 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1742>
   176f3:	mov    rcx,QWORD PTR [rdx+0x8]
   176f7:	xor    r11d,r11d
   176fa:	cmp    rax,0x1
   176fe:	je     1771d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x171d>
   17700:	mov    rdx,r11
   17703:	mov    r9,rax
   17706:	shr    r9,1
   17709:	add    r11,r9
   1770c:	cmp    DWORD PTR [rcx+r11*4],r14d
   17710:	cmova  r11,rdx
   17714:	sub    rax,r9
   17717:	cmp    rax,0x1
   1771b:	ja     17700 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1700>
   1771d:	cmp    DWORD PTR [rcx+r11*4],r14d
   17721:	mov    rax,0xffffffffffffffff
   17728:	cmovne r11,rax
   1772c:	mov    rdx,QWORD PTR [rsp]
   17730:	mov    r9,r10
   17733:	cmp    rdi,rsi
   17736:	mov    rcx,QWORD PTR [rsp+0x10]
   1773b:	jb     17757 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1757>
   1773d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17742:	mov    r11,0xffffffffffffffff
   17749:	cmp    rdi,rsi
   1774c:	mov    rcx,QWORD PTR [rsp+0x10]
   17751:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17757:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   1775c:	cmp    rbx,r12
   1775f:	je     1a873 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4873>
   17765:	mov    rsi,QWORD PTR [rdx+0x40]
   17769:	cmp    rbx,rsi
   1776c:	mov    r11,QWORD PTR [rsp+0x8]
   17771:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17777:	mov    rsi,QWORD PTR [rdx+0x10]
   1777b:	cmp    rbx,rsi
   1777e:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17784:	mov    rax,QWORD PTR [rdx+0x38]
   17788:	mov    rcx,QWORD PTR [rdx+0x8]
   1778c:	lea    rdx,[rbx+rbx*2]
   17790:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17795:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17798:	mov    ecx,DWORD PTR [r15+rcx*4]
   1779c:	mov    BYTE PTR [rsi],cl
   1779e:	shr    ecx,0x8
   177a1:	and    ecx,0x7fffff
   177a7:	lea    rdi,[rsi+0x1]
   177ab:	mov    rax,QWORD PTR [rax+rdx*8]
   177af:	add    rax,rsi
   177b2:	mov    QWORD PTR [rsp+0x1a8],rbx
   177ba:	mov    QWORD PTR [rsp+0x1b0],rdi
   177c2:	mov    QWORD PTR [rsp+0x1b8],rax
   177ca:	mov    DWORD PTR [rsp+0x1c0],ecx
   177d1:	prefetcht0 BYTE PTR [r15+rcx*4]
   177d6:	inc    rbx
   177d9:	mov    rax,r8
   177dc:	not    rax
   177df:	add    rax,rsi
   177e2:	inc    rax
   177e5:	mov    QWORD PTR [rsp+0xd8],rax
   177ed:	mov    QWORD PTR [rsp+0x568],rax
   177f5:	mov    r14d,DWORD PTR [rsp+0x1e0]
   177fd:	mov    eax,DWORD PTR [r15+r14*4]
   17801:	test   eax,eax
   17803:	jns    16d81 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xd81>
   17809:	mov    r11,QWORD PTR [rsp+0xd0]
   17811:	add    r11,r8
   17814:	mov    QWORD PTR [rsp+0x1d0],r11
   1781c:	mov    rdx,QWORD PTR [rsp]
   17820:	mov    rsi,QWORD PTR [rdx+0x40]
   17824:	mov    rdi,QWORD PTR [rsp+0x1c8]
   1782c:	cmp    rdi,rsi
   1782f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   17835:	mov    rax,QWORD PTR [rdx+0x38]
   17839:	mov    r13,QWORD PTR [rdx+0x50]
   1783d:	lea    rcx,[rdi+rdi*2]
   17841:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   17846:	mov    rsi,QWORD PTR [rdx+0x58]
   1784a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   1784f:	mov    rax,QWORD PTR [rdx+0x10]
   17853:	test   rax,rax
   17856:	je     178b2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x18b2>
   17858:	mov    rcx,QWORD PTR [rdx+0x8]
   1785c:	xor    r11d,r11d
   1785f:	cmp    rax,0x1
   17863:	je     1788d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x188d>
   17865:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17870:	mov    rdx,r11
   17873:	mov    r9,rax
   17876:	shr    r9,1
   17879:	add    r11,r9
   1787c:	cmp    DWORD PTR [rcx+r11*4],r14d
   17880:	cmova  r11,rdx
   17884:	sub    rax,r9
   17887:	cmp    rax,0x1
   1788b:	ja     17870 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1870>
   1788d:	cmp    DWORD PTR [rcx+r11*4],r14d
   17891:	mov    rax,0xffffffffffffffff
   17898:	cmovne r11,rax
   1789c:	mov    rdx,QWORD PTR [rsp]
   178a0:	mov    r9,r10
   178a3:	cmp    rdi,rsi
   178a6:	mov    rcx,QWORD PTR [rsp+0x10]
   178ab:	jb     178c7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x18c7>
   178ad:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   178b2:	mov    r11,0xffffffffffffffff
   178b9:	cmp    rdi,rsi
   178bc:	mov    rcx,QWORD PTR [rsp+0x10]
   178c1:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   178c7:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   178cc:	cmp    rbx,r12
   178cf:	je     1a8e3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x48e3>
   178d5:	mov    rsi,QWORD PTR [rdx+0x40]
   178d9:	cmp    rbx,rsi
   178dc:	mov    r11,QWORD PTR [rsp+0x8]
   178e1:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   178e7:	mov    rsi,QWORD PTR [rdx+0x10]
   178eb:	cmp    rbx,rsi
   178ee:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   178f4:	mov    rax,QWORD PTR [rdx+0x38]
   178f8:	mov    rcx,QWORD PTR [rdx+0x8]
   178fc:	lea    rdx,[rbx+rbx*2]
   17900:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17905:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17908:	mov    ecx,DWORD PTR [r15+rcx*4]
   1790c:	mov    BYTE PTR [rsi],cl
   1790e:	shr    ecx,0x8
   17911:	and    ecx,0x7fffff
   17917:	lea    rdi,[rsi+0x1]
   1791b:	mov    rax,QWORD PTR [rax+rdx*8]
   1791f:	add    rax,rsi
   17922:	mov    QWORD PTR [rsp+0x1c8],rbx
   1792a:	mov    QWORD PTR [rsp+0x1d0],rdi
   17932:	mov    QWORD PTR [rsp+0x1d8],rax
   1793a:	mov    DWORD PTR [rsp+0x1e0],ecx
   17941:	prefetcht0 BYTE PTR [r15+rcx*4]
   17946:	inc    rbx
   17949:	mov    rax,r8
   1794c:	not    rax
   1794f:	add    rax,rsi
   17952:	inc    rax
   17955:	mov    QWORD PTR [rsp+0xd0],rax
   1795d:	mov    QWORD PTR [rsp+0x570],rax
   17965:	mov    r14d,DWORD PTR [rsp+0x200]
   1796d:	mov    eax,DWORD PTR [r15+r14*4]
   17971:	test   eax,eax
   17973:	jns    16db2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xdb2>
   17979:	mov    r11,QWORD PTR [rsp+0xc0]
   17981:	add    r11,r8
   17984:	mov    QWORD PTR [rsp+0x1f0],r11
   1798c:	mov    rdx,QWORD PTR [rsp]
   17990:	mov    rsi,QWORD PTR [rdx+0x40]
   17994:	mov    rdi,QWORD PTR [rsp+0x1e8]
   1799c:	cmp    rdi,rsi
   1799f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   179a5:	mov    rax,QWORD PTR [rdx+0x38]
   179a9:	mov    r13,QWORD PTR [rdx+0x50]
   179ad:	lea    rcx,[rdi+rdi*2]
   179b1:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   179b6:	mov    rsi,QWORD PTR [rdx+0x58]
   179ba:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   179bf:	mov    rax,QWORD PTR [rdx+0x10]
   179c3:	test   rax,rax
   179c6:	je     17a22 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1a22>
   179c8:	mov    rcx,QWORD PTR [rdx+0x8]
   179cc:	xor    r11d,r11d
   179cf:	cmp    rax,0x1
   179d3:	je     179fd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x19fd>
   179d5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   179e0:	mov    rdx,r11
   179e3:	mov    r9,rax
   179e6:	shr    r9,1
   179e9:	add    r11,r9
   179ec:	cmp    DWORD PTR [rcx+r11*4],r14d
   179f0:	cmova  r11,rdx
   179f4:	sub    rax,r9
   179f7:	cmp    rax,0x1
   179fb:	ja     179e0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x19e0>
   179fd:	cmp    DWORD PTR [rcx+r11*4],r14d
   17a01:	mov    rax,0xffffffffffffffff
   17a08:	cmovne r11,rax
   17a0c:	mov    rdx,QWORD PTR [rsp]
   17a10:	mov    r9,r10
   17a13:	cmp    rdi,rsi
   17a16:	mov    rcx,QWORD PTR [rsp+0x10]
   17a1b:	jb     17a37 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1a37>
   17a1d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17a22:	mov    r11,0xffffffffffffffff
   17a29:	cmp    rdi,rsi
   17a2c:	mov    rcx,QWORD PTR [rsp+0x10]
   17a31:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17a37:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   17a3c:	cmp    rbx,r12
   17a3f:	je     1a969 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4969>
   17a45:	mov    rsi,QWORD PTR [rdx+0x40]
   17a49:	cmp    rbx,rsi
   17a4c:	mov    r11,QWORD PTR [rsp+0x8]
   17a51:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17a57:	mov    rsi,QWORD PTR [rdx+0x10]
   17a5b:	cmp    rbx,rsi
   17a5e:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17a64:	mov    rax,QWORD PTR [rdx+0x38]
   17a68:	mov    rcx,QWORD PTR [rdx+0x8]
   17a6c:	lea    rdx,[rbx+rbx*2]
   17a70:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17a75:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17a78:	mov    ecx,DWORD PTR [r15+rcx*4]
   17a7c:	mov    BYTE PTR [rsi],cl
   17a7e:	shr    ecx,0x8
   17a81:	and    ecx,0x7fffff
   17a87:	lea    rdi,[rsi+0x1]
   17a8b:	mov    rax,QWORD PTR [rax+rdx*8]
   17a8f:	add    rax,rsi
   17a92:	mov    QWORD PTR [rsp+0x1e8],rbx
   17a9a:	mov    QWORD PTR [rsp+0x1f0],rdi
   17aa2:	mov    QWORD PTR [rsp+0x1f8],rax
   17aaa:	mov    DWORD PTR [rsp+0x200],ecx
   17ab1:	prefetcht0 BYTE PTR [r15+rcx*4]
   17ab6:	inc    rbx
   17ab9:	mov    rax,r8
   17abc:	not    rax
   17abf:	add    rax,rsi
   17ac2:	inc    rax
   17ac5:	mov    QWORD PTR [rsp+0xc0],rax
   17acd:	mov    QWORD PTR [rsp+0x578],rax
   17ad5:	mov    r14d,DWORD PTR [rsp+0x220]
   17add:	mov    eax,DWORD PTR [r15+r14*4]
   17ae1:	test   eax,eax
   17ae3:	jns    16de3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xde3>
   17ae9:	mov    r11,QWORD PTR [rsp+0xb0]
   17af1:	add    r11,r8
   17af4:	mov    QWORD PTR [rsp+0x210],r11
   17afc:	mov    rdx,QWORD PTR [rsp]
   17b00:	mov    rsi,QWORD PTR [rdx+0x40]
   17b04:	mov    rdi,QWORD PTR [rsp+0x208]
   17b0c:	cmp    rdi,rsi
   17b0f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   17b15:	mov    rax,QWORD PTR [rdx+0x38]
   17b19:	mov    r13,QWORD PTR [rdx+0x50]
   17b1d:	lea    rcx,[rdi+rdi*2]
   17b21:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   17b26:	mov    rsi,QWORD PTR [rdx+0x58]
   17b2a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17b2f:	mov    rax,QWORD PTR [rdx+0x10]
   17b33:	test   rax,rax
   17b36:	je     17b92 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1b92>
   17b38:	mov    rcx,QWORD PTR [rdx+0x8]
   17b3c:	xor    r11d,r11d
   17b3f:	cmp    rax,0x1
   17b43:	je     17b6d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1b6d>
   17b45:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17b50:	mov    rdx,r11
   17b53:	mov    r9,rax
   17b56:	shr    r9,1
   17b59:	add    r11,r9
   17b5c:	cmp    DWORD PTR [rcx+r11*4],r14d
   17b60:	cmova  r11,rdx
   17b64:	sub    rax,r9
   17b67:	cmp    rax,0x1
   17b6b:	ja     17b50 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1b50>
   17b6d:	cmp    DWORD PTR [rcx+r11*4],r14d
   17b71:	mov    rax,0xffffffffffffffff
   17b78:	cmovne r11,rax
   17b7c:	mov    rdx,QWORD PTR [rsp]
   17b80:	mov    r9,r10
   17b83:	cmp    rdi,rsi
   17b86:	mov    rcx,QWORD PTR [rsp+0x10]
   17b8b:	jb     17ba7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1ba7>
   17b8d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17b92:	mov    r11,0xffffffffffffffff
   17b99:	cmp    rdi,rsi
   17b9c:	mov    rcx,QWORD PTR [rsp+0x10]
   17ba1:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17ba7:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   17bac:	cmp    rbx,r12
   17baf:	je     1aa02 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4a02>
   17bb5:	mov    rsi,QWORD PTR [rdx+0x40]
   17bb9:	cmp    rbx,rsi
   17bbc:	mov    r11,QWORD PTR [rsp+0x8]
   17bc1:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17bc7:	mov    rsi,QWORD PTR [rdx+0x10]
   17bcb:	cmp    rbx,rsi
   17bce:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17bd4:	mov    rax,QWORD PTR [rdx+0x38]
   17bd8:	mov    rcx,QWORD PTR [rdx+0x8]
   17bdc:	lea    rdx,[rbx+rbx*2]
   17be0:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17be5:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17be8:	mov    ecx,DWORD PTR [r15+rcx*4]
   17bec:	mov    BYTE PTR [rsi],cl
   17bee:	shr    ecx,0x8
   17bf1:	and    ecx,0x7fffff
   17bf7:	lea    rdi,[rsi+0x1]
   17bfb:	mov    rax,QWORD PTR [rax+rdx*8]
   17bff:	add    rax,rsi
   17c02:	mov    QWORD PTR [rsp+0x208],rbx
   17c0a:	mov    QWORD PTR [rsp+0x210],rdi
   17c12:	mov    QWORD PTR [rsp+0x218],rax
   17c1a:	mov    DWORD PTR [rsp+0x220],ecx
   17c21:	prefetcht0 BYTE PTR [r15+rcx*4]
   17c26:	inc    rbx
   17c29:	mov    rax,r8
   17c2c:	not    rax
   17c2f:	add    rax,rsi
   17c32:	inc    rax
   17c35:	mov    QWORD PTR [rsp+0xb0],rax
   17c3d:	mov    QWORD PTR [rsp+0x580],rax
   17c45:	mov    r14d,DWORD PTR [rsp+0x240]
   17c4d:	mov    eax,DWORD PTR [r15+r14*4]
   17c51:	test   eax,eax
   17c53:	jns    16e14 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xe14>
   17c59:	mov    r11,QWORD PTR [rsp+0xa0]
   17c61:	add    r11,r8
   17c64:	mov    QWORD PTR [rsp+0x230],r11
   17c6c:	mov    rdx,QWORD PTR [rsp]
   17c70:	mov    rsi,QWORD PTR [rdx+0x40]
   17c74:	mov    rdi,QWORD PTR [rsp+0x228]
   17c7c:	cmp    rdi,rsi
   17c7f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   17c85:	mov    rax,QWORD PTR [rdx+0x38]
   17c89:	mov    r13,QWORD PTR [rdx+0x50]
   17c8d:	lea    rcx,[rdi+rdi*2]
   17c91:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   17c96:	mov    rsi,QWORD PTR [rdx+0x58]
   17c9a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17c9f:	mov    rax,QWORD PTR [rdx+0x10]
   17ca3:	test   rax,rax
   17ca6:	je     17d02 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1d02>
   17ca8:	mov    rcx,QWORD PTR [rdx+0x8]
   17cac:	xor    r11d,r11d
   17caf:	cmp    rax,0x1
   17cb3:	je     17cdd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1cdd>
   17cb5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17cc0:	mov    rdx,r11
   17cc3:	mov    r9,rax
   17cc6:	shr    r9,1
   17cc9:	add    r11,r9
   17ccc:	cmp    DWORD PTR [rcx+r11*4],r14d
   17cd0:	cmova  r11,rdx
   17cd4:	sub    rax,r9
   17cd7:	cmp    rax,0x1
   17cdb:	ja     17cc0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1cc0>
   17cdd:	cmp    DWORD PTR [rcx+r11*4],r14d
   17ce1:	mov    rax,0xffffffffffffffff
   17ce8:	cmovne r11,rax
   17cec:	mov    rdx,QWORD PTR [rsp]
   17cf0:	mov    r9,r10
   17cf3:	cmp    rdi,rsi
   17cf6:	mov    rcx,QWORD PTR [rsp+0x10]
   17cfb:	jb     17d17 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1d17>
   17cfd:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17d02:	mov    r11,0xffffffffffffffff
   17d09:	cmp    rdi,rsi
   17d0c:	mov    rcx,QWORD PTR [rsp+0x10]
   17d11:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17d17:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   17d1c:	cmp    rbx,r12
   17d1f:	je     1addf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4ddf>
   17d25:	mov    rsi,QWORD PTR [rdx+0x40]
   17d29:	cmp    rbx,rsi
   17d2c:	mov    r11,QWORD PTR [rsp+0x8]
   17d31:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17d37:	mov    rsi,QWORD PTR [rdx+0x10]
   17d3b:	cmp    rbx,rsi
   17d3e:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17d44:	mov    rax,QWORD PTR [rdx+0x38]
   17d48:	mov    rcx,QWORD PTR [rdx+0x8]
   17d4c:	lea    rdx,[rbx+rbx*2]
   17d50:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17d55:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17d58:	mov    ecx,DWORD PTR [r15+rcx*4]
   17d5c:	mov    BYTE PTR [rsi],cl
   17d5e:	shr    ecx,0x8
   17d61:	and    ecx,0x7fffff
   17d67:	lea    rdi,[rsi+0x1]
   17d6b:	mov    rax,QWORD PTR [rax+rdx*8]
   17d6f:	add    rax,rsi
   17d72:	mov    QWORD PTR [rsp+0x228],rbx
   17d7a:	mov    QWORD PTR [rsp+0x230],rdi
   17d82:	mov    QWORD PTR [rsp+0x238],rax
   17d8a:	mov    DWORD PTR [rsp+0x240],ecx
   17d91:	prefetcht0 BYTE PTR [r15+rcx*4]
   17d96:	inc    rbx
   17d99:	mov    rax,r8
   17d9c:	not    rax
   17d9f:	add    rax,rsi
   17da2:	inc    rax
   17da5:	mov    QWORD PTR [rsp+0xa0],rax
   17dad:	mov    QWORD PTR [rsp+0x588],rax
   17db5:	mov    r14d,DWORD PTR [rsp+0x260]
   17dbd:	mov    eax,DWORD PTR [r15+r14*4]
   17dc1:	test   eax,eax
   17dc3:	jns    16e45 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xe45>
   17dc9:	mov    r11,QWORD PTR [rsp+0x98]
   17dd1:	add    r11,r8
   17dd4:	mov    QWORD PTR [rsp+0x250],r11
   17ddc:	mov    rdx,QWORD PTR [rsp]
   17de0:	mov    rsi,QWORD PTR [rdx+0x40]
   17de4:	mov    rdi,QWORD PTR [rsp+0x248]
   17dec:	cmp    rdi,rsi
   17def:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   17df5:	mov    rax,QWORD PTR [rdx+0x38]
   17df9:	mov    r13,QWORD PTR [rdx+0x50]
   17dfd:	lea    rcx,[rdi+rdi*2]
   17e01:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   17e06:	mov    rsi,QWORD PTR [rdx+0x58]
   17e0a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17e0f:	mov    rax,QWORD PTR [rdx+0x10]
   17e13:	test   rax,rax
   17e16:	je     17e72 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1e72>
   17e18:	mov    rcx,QWORD PTR [rdx+0x8]
   17e1c:	xor    r11d,r11d
   17e1f:	cmp    rax,0x1
   17e23:	je     17e4d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1e4d>
   17e25:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17e30:	mov    rdx,r11
   17e33:	mov    r9,rax
   17e36:	shr    r9,1
   17e39:	add    r11,r9
   17e3c:	cmp    DWORD PTR [rcx+r11*4],r14d
   17e40:	cmova  r11,rdx
   17e44:	sub    rax,r9
   17e47:	cmp    rax,0x1
   17e4b:	ja     17e30 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1e30>
   17e4d:	cmp    DWORD PTR [rcx+r11*4],r14d
   17e51:	mov    rax,0xffffffffffffffff
   17e58:	cmovne r11,rax
   17e5c:	mov    rdx,QWORD PTR [rsp]
   17e60:	mov    r9,r10
   17e63:	cmp    rdi,rsi
   17e66:	mov    rcx,QWORD PTR [rsp+0x10]
   17e6b:	jb     17e87 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1e87>
   17e6d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17e72:	mov    r11,0xffffffffffffffff
   17e79:	cmp    rdi,rsi
   17e7c:	mov    rcx,QWORD PTR [rsp+0x10]
   17e81:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17e87:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   17e8c:	cmp    rbx,r12
   17e8f:	je     1ae8b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4e8b>
   17e95:	mov    rsi,QWORD PTR [rdx+0x40]
   17e99:	cmp    rbx,rsi
   17e9c:	mov    r11,QWORD PTR [rsp+0x8]
   17ea1:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   17ea7:	mov    rsi,QWORD PTR [rdx+0x10]
   17eab:	cmp    rbx,rsi
   17eae:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   17eb4:	mov    rax,QWORD PTR [rdx+0x38]
   17eb8:	mov    rcx,QWORD PTR [rdx+0x8]
   17ebc:	lea    rdx,[rbx+rbx*2]
   17ec0:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   17ec5:	mov    ecx,DWORD PTR [rcx+rbx*4]
   17ec8:	mov    ecx,DWORD PTR [r15+rcx*4]
   17ecc:	mov    BYTE PTR [rsi],cl
   17ece:	shr    ecx,0x8
   17ed1:	and    ecx,0x7fffff
   17ed7:	lea    rdi,[rsi+0x1]
   17edb:	mov    rax,QWORD PTR [rax+rdx*8]
   17edf:	add    rax,rsi
   17ee2:	mov    QWORD PTR [rsp+0x248],rbx
   17eea:	mov    QWORD PTR [rsp+0x250],rdi
   17ef2:	mov    QWORD PTR [rsp+0x258],rax
   17efa:	mov    DWORD PTR [rsp+0x260],ecx
   17f01:	prefetcht0 BYTE PTR [r15+rcx*4]
   17f06:	inc    rbx
   17f09:	mov    rax,r8
   17f0c:	not    rax
   17f0f:	add    rax,rsi
   17f12:	inc    rax
   17f15:	mov    QWORD PTR [rsp+0x98],rax
   17f1d:	mov    QWORD PTR [rsp+0x590],rax
   17f25:	mov    r14d,DWORD PTR [rsp+0x280]
   17f2d:	mov    eax,DWORD PTR [r15+r14*4]
   17f31:	test   eax,eax
   17f33:	jns    16e76 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xe76>
   17f39:	mov    r11,QWORD PTR [rsp+0x90]
   17f41:	add    r11,r8
   17f44:	mov    QWORD PTR [rsp+0x270],r11
   17f4c:	mov    rdx,QWORD PTR [rsp]
   17f50:	mov    rsi,QWORD PTR [rdx+0x40]
   17f54:	mov    rdi,QWORD PTR [rsp+0x268]
   17f5c:	cmp    rdi,rsi
   17f5f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   17f65:	mov    rax,QWORD PTR [rdx+0x38]
   17f69:	mov    r13,QWORD PTR [rdx+0x50]
   17f6d:	lea    rcx,[rdi+rdi*2]
   17f71:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   17f76:	mov    rsi,QWORD PTR [rdx+0x58]
   17f7a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   17f7f:	mov    rax,QWORD PTR [rdx+0x10]
   17f83:	test   rax,rax
   17f86:	je     17fe2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1fe2>
   17f88:	mov    rcx,QWORD PTR [rdx+0x8]
   17f8c:	xor    r11d,r11d
   17f8f:	cmp    rax,0x1
   17f93:	je     17fbd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1fbd>
   17f95:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17fa0:	mov    rdx,r11
   17fa3:	mov    r9,rax
   17fa6:	shr    r9,1
   17fa9:	add    r11,r9
   17fac:	cmp    DWORD PTR [rcx+r11*4],r14d
   17fb0:	cmova  r11,rdx
   17fb4:	sub    rax,r9
   17fb7:	cmp    rax,0x1
   17fbb:	ja     17fa0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1fa0>
   17fbd:	cmp    DWORD PTR [rcx+r11*4],r14d
   17fc1:	mov    rax,0xffffffffffffffff
   17fc8:	cmovne r11,rax
   17fcc:	mov    rdx,QWORD PTR [rsp]
   17fd0:	mov    r9,r10
   17fd3:	cmp    rdi,rsi
   17fd6:	mov    rcx,QWORD PTR [rsp+0x10]
   17fdb:	jb     17ff7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1ff7>
   17fdd:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17fe2:	mov    r11,0xffffffffffffffff
   17fe9:	cmp    rdi,rsi
   17fec:	mov    rcx,QWORD PTR [rsp+0x10]
   17ff1:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   17ff7:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   17ffc:	cmp    rbx,r12
   17fff:	je     1af4a <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4f4a>
   18005:	mov    rsi,QWORD PTR [rdx+0x40]
   18009:	cmp    rbx,rsi
   1800c:	mov    r11,QWORD PTR [rsp+0x8]
   18011:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18017:	mov    rsi,QWORD PTR [rdx+0x10]
   1801b:	cmp    rbx,rsi
   1801e:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18024:	mov    rax,QWORD PTR [rdx+0x38]
   18028:	mov    rcx,QWORD PTR [rdx+0x8]
   1802c:	lea    rdx,[rbx+rbx*2]
   18030:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   18035:	mov    ecx,DWORD PTR [rcx+rbx*4]
   18038:	mov    ecx,DWORD PTR [r15+rcx*4]
   1803c:	mov    BYTE PTR [rsi],cl
   1803e:	shr    ecx,0x8
   18041:	and    ecx,0x7fffff
   18047:	lea    rdi,[rsi+0x1]
   1804b:	mov    rax,QWORD PTR [rax+rdx*8]
   1804f:	add    rax,rsi
   18052:	mov    QWORD PTR [rsp+0x268],rbx
   1805a:	mov    QWORD PTR [rsp+0x270],rdi
   18062:	mov    QWORD PTR [rsp+0x278],rax
   1806a:	mov    DWORD PTR [rsp+0x280],ecx
   18071:	prefetcht0 BYTE PTR [r15+rcx*4]
   18076:	inc    rbx
   18079:	mov    rax,r8
   1807c:	not    rax
   1807f:	add    rax,rsi
   18082:	inc    rax
   18085:	mov    QWORD PTR [rsp+0x90],rax
   1808d:	mov    QWORD PTR [rsp+0x598],rax
   18095:	mov    r14d,DWORD PTR [rsp+0x2a0]
   1809d:	mov    eax,DWORD PTR [r15+r14*4]
   180a1:	test   eax,eax
   180a3:	jns    16ea7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xea7>
   180a9:	mov    r11,QWORD PTR [rsp+0x40]
   180ae:	add    r11,r8
   180b1:	mov    QWORD PTR [rsp+0x290],r11
   180b9:	mov    rdx,QWORD PTR [rsp]
   180bd:	mov    rsi,QWORD PTR [rdx+0x40]
   180c1:	mov    rdi,QWORD PTR [rsp+0x288]
   180c9:	cmp    rdi,rsi
   180cc:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   180d2:	mov    rax,QWORD PTR [rdx+0x38]
   180d6:	mov    r13,QWORD PTR [rdx+0x50]
   180da:	lea    rcx,[rdi+rdi*2]
   180de:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   180e3:	mov    rsi,QWORD PTR [rdx+0x58]
   180e7:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   180ec:	mov    rax,QWORD PTR [rdx+0x10]
   180f0:	test   rax,rax
   180f3:	je     18152 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2152>
   180f5:	mov    rcx,QWORD PTR [rdx+0x8]
   180f9:	xor    r11d,r11d
   180fc:	cmp    rax,0x1
   18100:	je     1812d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x212d>
   18102:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   18110:	mov    rdx,r11
   18113:	mov    r9,rax
   18116:	shr    r9,1
   18119:	add    r11,r9
   1811c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18120:	cmova  r11,rdx
   18124:	sub    rax,r9
   18127:	cmp    rax,0x1
   1812b:	ja     18110 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2110>
   1812d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18131:	mov    rax,0xffffffffffffffff
   18138:	cmovne r11,rax
   1813c:	mov    rdx,QWORD PTR [rsp]
   18140:	mov    r9,r10
   18143:	cmp    rdi,rsi
   18146:	mov    rcx,QWORD PTR [rsp+0x10]
   1814b:	jb     18167 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2167>
   1814d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18152:	mov    r11,0xffffffffffffffff
   18159:	cmp    rdi,rsi
   1815c:	mov    rcx,QWORD PTR [rsp+0x10]
   18161:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18167:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   1816c:	cmp    rbx,r12
   1816f:	je     1b01c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x501c>
   18175:	mov    rsi,QWORD PTR [rdx+0x40]
   18179:	cmp    rbx,rsi
   1817c:	mov    r11,QWORD PTR [rsp+0x8]
   18181:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18187:	mov    rsi,QWORD PTR [rdx+0x10]
   1818b:	cmp    rbx,rsi
   1818e:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18194:	mov    rax,QWORD PTR [rdx+0x38]
   18198:	mov    rcx,QWORD PTR [rdx+0x8]
   1819c:	lea    rdx,[rbx+rbx*2]
   181a0:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   181a5:	mov    ecx,DWORD PTR [rcx+rbx*4]
   181a8:	mov    ecx,DWORD PTR [r15+rcx*4]
   181ac:	mov    BYTE PTR [rsi],cl
   181ae:	shr    ecx,0x8
   181b1:	and    ecx,0x7fffff
   181b7:	lea    rdi,[rsi+0x1]
   181bb:	mov    rax,QWORD PTR [rax+rdx*8]
   181bf:	add    rax,rsi
   181c2:	mov    QWORD PTR [rsp+0x288],rbx
   181ca:	mov    QWORD PTR [rsp+0x290],rdi
   181d2:	mov    QWORD PTR [rsp+0x298],rax
   181da:	mov    DWORD PTR [rsp+0x2a0],ecx
   181e1:	prefetcht0 BYTE PTR [r15+rcx*4]
   181e6:	inc    rbx
   181e9:	mov    rax,r8
   181ec:	not    rax
   181ef:	add    rax,rsi
   181f2:	inc    rax
   181f5:	mov    QWORD PTR [rsp+0x40],rax
   181fa:	mov    QWORD PTR [rsp+0x5a0],rax
   18202:	mov    r14d,DWORD PTR [rsp+0x2c0]
   1820a:	mov    eax,DWORD PTR [r15+r14*4]
   1820e:	test   eax,eax
   18210:	jns    16ed5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xed5>
   18216:	mov    r11,QWORD PTR [rsp+0x38]
   1821b:	add    r11,r8
   1821e:	mov    QWORD PTR [rsp+0x2b0],r11
   18226:	mov    rdx,QWORD PTR [rsp]
   1822a:	mov    rsi,QWORD PTR [rdx+0x40]
   1822e:	mov    rdi,QWORD PTR [rsp+0x2a8]
   18236:	cmp    rdi,rsi
   18239:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1823f:	mov    rax,QWORD PTR [rdx+0x38]
   18243:	mov    r13,QWORD PTR [rdx+0x50]
   18247:	lea    rcx,[rdi+rdi*2]
   1824b:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18250:	mov    rsi,QWORD PTR [rdx+0x58]
   18254:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18259:	mov    rax,QWORD PTR [rdx+0x10]
   1825d:	test   rax,rax
   18260:	je     182ad <mibz2::bwt::room_segments_unrolled::<32, 1>+0x22ad>
   18262:	mov    rcx,QWORD PTR [rdx+0x8]
   18266:	xor    r11d,r11d
   18269:	cmp    rax,0x1
   1826d:	je     1828d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x228d>
   1826f:	nop
   18270:	mov    rdx,r11
   18273:	mov    r9,rax
   18276:	shr    r9,1
   18279:	add    r11,r9
   1827c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18280:	cmova  r11,rdx
   18284:	sub    rax,r9
   18287:	cmp    rax,0x1
   1828b:	ja     18270 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2270>
   1828d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18291:	mov    rax,0xffffffffffffffff
   18298:	cmovne r11,rax
   1829c:	mov    rdx,QWORD PTR [rsp]
   182a0:	mov    r9,r10
   182a3:	cmp    rdi,rsi
   182a6:	jb     182bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x22bd>
   182a8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   182ad:	mov    r11,0xffffffffffffffff
   182b4:	cmp    rdi,rsi
   182b7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   182bd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   182c2:	cmp    rbx,r12
   182c5:	je     1b101 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5101>
   182cb:	mov    rsi,QWORD PTR [rdx+0x40]
   182cf:	cmp    rbx,rsi
   182d2:	mov    r11,QWORD PTR [rsp+0x8]
   182d7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   182dd:	mov    rsi,QWORD PTR [rdx+0x10]
   182e1:	cmp    rbx,rsi
   182e4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   182ea:	mov    rax,QWORD PTR [rdx+0x38]
   182ee:	mov    rcx,QWORD PTR [rdx+0x8]
   182f2:	lea    rdx,[rbx+rbx*2]
   182f6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   182fb:	mov    ecx,DWORD PTR [rcx+rbx*4]
   182fe:	mov    ecx,DWORD PTR [r15+rcx*4]
   18302:	mov    BYTE PTR [rsi],cl
   18304:	shr    ecx,0x8
   18307:	and    ecx,0x7fffff
   1830d:	lea    rdi,[rsi+0x1]
   18311:	mov    rax,QWORD PTR [rax+rdx*8]
   18315:	add    rax,rsi
   18318:	mov    QWORD PTR [rsp+0x2a8],rbx
   18320:	mov    QWORD PTR [rsp+0x2b0],rdi
   18328:	mov    QWORD PTR [rsp+0x2b8],rax
   18330:	mov    DWORD PTR [rsp+0x2c0],ecx
   18337:	prefetcht0 BYTE PTR [r15+rcx*4]
   1833c:	inc    rbx
   1833f:	mov    rax,r8
   18342:	not    rax
   18345:	add    rax,rsi
   18348:	inc    rax
   1834b:	mov    QWORD PTR [rsp+0x38],rax
   18350:	mov    QWORD PTR [rsp+0x5a8],rax
   18358:	mov    r14d,DWORD PTR [rsp+0x2e0]
   18360:	mov    eax,DWORD PTR [r15+r14*4]
   18364:	test   eax,eax
   18366:	jns    16f03 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xf03>
   1836c:	mov    r11,QWORD PTR [rsp+0x30]
   18371:	add    r11,r8
   18374:	mov    QWORD PTR [rsp+0x2d0],r11
   1837c:	mov    rdx,QWORD PTR [rsp]
   18380:	mov    rsi,QWORD PTR [rdx+0x40]
   18384:	mov    rdi,QWORD PTR [rsp+0x2c8]
   1838c:	cmp    rdi,rsi
   1838f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18395:	mov    rax,QWORD PTR [rdx+0x38]
   18399:	mov    r13,QWORD PTR [rdx+0x50]
   1839d:	lea    rcx,[rdi+rdi*2]
   183a1:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   183a6:	mov    rsi,QWORD PTR [rdx+0x58]
   183aa:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   183af:	mov    rax,QWORD PTR [rdx+0x10]
   183b3:	test   rax,rax
   183b6:	je     1840d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x240d>
   183b8:	mov    rcx,QWORD PTR [rdx+0x8]
   183bc:	xor    r11d,r11d
   183bf:	cmp    rax,0x1
   183c3:	je     183ed <mibz2::bwt::room_segments_unrolled::<32, 1>+0x23ed>
   183c5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   183d0:	mov    rdx,r11
   183d3:	mov    r9,rax
   183d6:	shr    r9,1
   183d9:	add    r11,r9
   183dc:	cmp    DWORD PTR [rcx+r11*4],r14d
   183e0:	cmova  r11,rdx
   183e4:	sub    rax,r9
   183e7:	cmp    rax,0x1
   183eb:	ja     183d0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x23d0>
   183ed:	cmp    DWORD PTR [rcx+r11*4],r14d
   183f1:	mov    rax,0xffffffffffffffff
   183f8:	cmovne r11,rax
   183fc:	mov    rdx,QWORD PTR [rsp]
   18400:	mov    r9,r10
   18403:	cmp    rdi,rsi
   18406:	jb     1841d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x241d>
   18408:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1840d:	mov    r11,0xffffffffffffffff
   18414:	cmp    rdi,rsi
   18417:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1841d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18422:	cmp    rbx,r12
   18425:	je     1b1fb <mibz2::bwt::room_segments_unrolled::<32, 1>+0x51fb>
   1842b:	mov    rsi,QWORD PTR [rdx+0x40]
   1842f:	cmp    rbx,rsi
   18432:	mov    r11,QWORD PTR [rsp+0x8]
   18437:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1843d:	mov    rsi,QWORD PTR [rdx+0x10]
   18441:	cmp    rbx,rsi
   18444:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1844a:	mov    rax,QWORD PTR [rdx+0x38]
   1844e:	mov    rcx,QWORD PTR [rdx+0x8]
   18452:	lea    rdx,[rbx+rbx*2]
   18456:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1845b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1845e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18462:	mov    BYTE PTR [rsi],cl
   18464:	shr    ecx,0x8
   18467:	and    ecx,0x7fffff
   1846d:	lea    rdi,[rsi+0x1]
   18471:	mov    rax,QWORD PTR [rax+rdx*8]
   18475:	add    rax,rsi
   18478:	mov    QWORD PTR [rsp+0x2c8],rbx
   18480:	mov    QWORD PTR [rsp+0x2d0],rdi
   18488:	mov    QWORD PTR [rsp+0x2d8],rax
   18490:	mov    DWORD PTR [rsp+0x2e0],ecx
   18497:	prefetcht0 BYTE PTR [r15+rcx*4]
   1849c:	inc    rbx
   1849f:	mov    rax,r8
   184a2:	not    rax
   184a5:	add    rax,rsi
   184a8:	inc    rax
   184ab:	mov    QWORD PTR [rsp+0x30],rax
   184b0:	mov    QWORD PTR [rsp+0x5b0],rax
   184b8:	mov    r14d,DWORD PTR [rsp+0x300]
   184c0:	mov    eax,DWORD PTR [r15+r14*4]
   184c4:	test   eax,eax
   184c6:	jns    16f31 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xf31>
   184cc:	mov    r11,QWORD PTR [rsp+0x88]
   184d4:	add    r11,r8
   184d7:	mov    QWORD PTR [rsp+0x2f0],r11
   184df:	mov    rdx,QWORD PTR [rsp]
   184e3:	mov    rsi,QWORD PTR [rdx+0x40]
   184e7:	mov    rdi,QWORD PTR [rsp+0x2e8]
   184ef:	cmp    rdi,rsi
   184f2:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   184f8:	mov    rax,QWORD PTR [rdx+0x38]
   184fc:	mov    r13,QWORD PTR [rdx+0x50]
   18500:	lea    rcx,[rdi+rdi*2]
   18504:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18509:	mov    rsi,QWORD PTR [rdx+0x58]
   1850d:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18512:	mov    rax,QWORD PTR [rdx+0x10]
   18516:	test   rax,rax
   18519:	je     1856d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x256d>
   1851b:	mov    rcx,QWORD PTR [rdx+0x8]
   1851f:	xor    r11d,r11d
   18522:	cmp    rax,0x1
   18526:	je     1854d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x254d>
   18528:	nop    DWORD PTR [rax+rax*1+0x0]
   18530:	mov    rdx,r11
   18533:	mov    r9,rax
   18536:	shr    r9,1
   18539:	add    r11,r9
   1853c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18540:	cmova  r11,rdx
   18544:	sub    rax,r9
   18547:	cmp    rax,0x1
   1854b:	ja     18530 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2530>
   1854d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18551:	mov    rax,0xffffffffffffffff
   18558:	cmovne r11,rax
   1855c:	mov    rdx,QWORD PTR [rsp]
   18560:	mov    r9,r10
   18563:	cmp    rdi,rsi
   18566:	jb     1857d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x257d>
   18568:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1856d:	mov    r11,0xffffffffffffffff
   18574:	cmp    rdi,rsi
   18577:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1857d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18582:	cmp    rbx,r12
   18585:	je     1b305 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5305>
   1858b:	mov    rsi,QWORD PTR [rdx+0x40]
   1858f:	cmp    rbx,rsi
   18592:	mov    r11,QWORD PTR [rsp+0x8]
   18597:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1859d:	mov    rsi,QWORD PTR [rdx+0x10]
   185a1:	cmp    rbx,rsi
   185a4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   185aa:	mov    rax,QWORD PTR [rdx+0x38]
   185ae:	mov    rcx,QWORD PTR [rdx+0x8]
   185b2:	lea    rdx,[rbx+rbx*2]
   185b6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   185bb:	mov    ecx,DWORD PTR [rcx+rbx*4]
   185be:	mov    ecx,DWORD PTR [r15+rcx*4]
   185c2:	mov    BYTE PTR [rsi],cl
   185c4:	shr    ecx,0x8
   185c7:	and    ecx,0x7fffff
   185cd:	lea    rdi,[rsi+0x1]
   185d1:	mov    rax,QWORD PTR [rax+rdx*8]
   185d5:	add    rax,rsi
   185d8:	mov    QWORD PTR [rsp+0x2e8],rbx
   185e0:	mov    QWORD PTR [rsp+0x2f0],rdi
   185e8:	mov    QWORD PTR [rsp+0x2f8],rax
   185f0:	mov    DWORD PTR [rsp+0x300],ecx
   185f7:	prefetcht0 BYTE PTR [r15+rcx*4]
   185fc:	inc    rbx
   185ff:	mov    rax,r8
   18602:	not    rax
   18605:	add    rax,rsi
   18608:	inc    rax
   1860b:	mov    QWORD PTR [rsp+0x88],rax
   18613:	mov    QWORD PTR [rsp+0x5b8],rax
   1861b:	mov    r14d,DWORD PTR [rsp+0x320]
   18623:	mov    eax,DWORD PTR [r15+r14*4]
   18627:	test   eax,eax
   18629:	jns    16f62 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xf62>
   1862f:	mov    r11,QWORD PTR [rsp+0x80]
   18637:	add    r11,r8
   1863a:	mov    QWORD PTR [rsp+0x310],r11
   18642:	mov    rdx,QWORD PTR [rsp]
   18646:	mov    rsi,QWORD PTR [rdx+0x40]
   1864a:	mov    rdi,QWORD PTR [rsp+0x308]
   18652:	cmp    rdi,rsi
   18655:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1865b:	mov    rax,QWORD PTR [rdx+0x38]
   1865f:	mov    r13,QWORD PTR [rdx+0x50]
   18663:	lea    rcx,[rdi+rdi*2]
   18667:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   1866c:	mov    rsi,QWORD PTR [rdx+0x58]
   18670:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18675:	mov    rax,QWORD PTR [rdx+0x10]
   18679:	test   rax,rax
   1867c:	je     186cd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x26cd>
   1867e:	mov    rcx,QWORD PTR [rdx+0x8]
   18682:	xor    r11d,r11d
   18685:	cmp    rax,0x1
   18689:	je     186ad <mibz2::bwt::room_segments_unrolled::<32, 1>+0x26ad>
   1868b:	nop    DWORD PTR [rax+rax*1+0x0]
   18690:	mov    rdx,r11
   18693:	mov    r9,rax
   18696:	shr    r9,1
   18699:	add    r11,r9
   1869c:	cmp    DWORD PTR [rcx+r11*4],r14d
   186a0:	cmova  r11,rdx
   186a4:	sub    rax,r9
   186a7:	cmp    rax,0x1
   186ab:	ja     18690 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2690>
   186ad:	cmp    DWORD PTR [rcx+r11*4],r14d
   186b1:	mov    rax,0xffffffffffffffff
   186b8:	cmovne r11,rax
   186bc:	mov    rdx,QWORD PTR [rsp]
   186c0:	mov    r9,r10
   186c3:	cmp    rdi,rsi
   186c6:	jb     186dd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x26dd>
   186c8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   186cd:	mov    r11,0xffffffffffffffff
   186d4:	cmp    rdi,rsi
   186d7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   186dd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   186e2:	cmp    rbx,r12
   186e5:	je     1b41f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x541f>
   186eb:	mov    rsi,QWORD PTR [rdx+0x40]
   186ef:	cmp    rbx,rsi
   186f2:	mov    r11,QWORD PTR [rsp+0x8]
   186f7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   186fd:	mov    rsi,QWORD PTR [rdx+0x10]
   18701:	cmp    rbx,rsi
   18704:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1870a:	mov    rax,QWORD PTR [rdx+0x38]
   1870e:	mov    rcx,QWORD PTR [rdx+0x8]
   18712:	lea    rdx,[rbx+rbx*2]
   18716:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1871b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1871e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18722:	mov    BYTE PTR [rsi],cl
   18724:	shr    ecx,0x8
   18727:	and    ecx,0x7fffff
   1872d:	lea    rdi,[rsi+0x1]
   18731:	mov    rax,QWORD PTR [rax+rdx*8]
   18735:	add    rax,rsi
   18738:	mov    QWORD PTR [rsp+0x308],rbx
   18740:	mov    QWORD PTR [rsp+0x310],rdi
   18748:	mov    QWORD PTR [rsp+0x318],rax
   18750:	mov    DWORD PTR [rsp+0x320],ecx
   18757:	prefetcht0 BYTE PTR [r15+rcx*4]
   1875c:	inc    rbx
   1875f:	mov    rax,r8
   18762:	not    rax
   18765:	add    rax,rsi
   18768:	inc    rax
   1876b:	mov    QWORD PTR [rsp+0x80],rax
   18773:	mov    QWORD PTR [rsp+0x5c0],rax
   1877b:	mov    r14d,DWORD PTR [rsp+0x340]
   18783:	mov    eax,DWORD PTR [r15+r14*4]
   18787:	test   eax,eax
   18789:	jns    16f93 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xf93>
   1878f:	mov    r11,QWORD PTR [rsp+0x78]
   18794:	add    r11,r8
   18797:	mov    QWORD PTR [rsp+0x330],r11
   1879f:	mov    rdx,QWORD PTR [rsp]
   187a3:	mov    rsi,QWORD PTR [rdx+0x40]
   187a7:	mov    rdi,QWORD PTR [rsp+0x328]
   187af:	cmp    rdi,rsi
   187b2:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   187b8:	mov    rax,QWORD PTR [rdx+0x38]
   187bc:	mov    r13,QWORD PTR [rdx+0x50]
   187c0:	lea    rcx,[rdi+rdi*2]
   187c4:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   187c9:	mov    rsi,QWORD PTR [rdx+0x58]
   187cd:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   187d2:	mov    rax,QWORD PTR [rdx+0x10]
   187d6:	test   rax,rax
   187d9:	je     1882d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x282d>
   187db:	mov    rcx,QWORD PTR [rdx+0x8]
   187df:	xor    r11d,r11d
   187e2:	cmp    rax,0x1
   187e6:	je     1880d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x280d>
   187e8:	nop    DWORD PTR [rax+rax*1+0x0]
   187f0:	mov    rdx,r11
   187f3:	mov    r9,rax
   187f6:	shr    r9,1
   187f9:	add    r11,r9
   187fc:	cmp    DWORD PTR [rcx+r11*4],r14d
   18800:	cmova  r11,rdx
   18804:	sub    rax,r9
   18807:	cmp    rax,0x1
   1880b:	ja     187f0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x27f0>
   1880d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18811:	mov    rax,0xffffffffffffffff
   18818:	cmovne r11,rax
   1881c:	mov    rdx,QWORD PTR [rsp]
   18820:	mov    r9,r10
   18823:	cmp    rdi,rsi
   18826:	jb     1883d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x283d>
   18828:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1882d:	mov    r11,0xffffffffffffffff
   18834:	cmp    rdi,rsi
   18837:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1883d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18842:	cmp    rbx,r12
   18845:	je     1b54c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x554c>
   1884b:	mov    rsi,QWORD PTR [rdx+0x40]
   1884f:	cmp    rbx,rsi
   18852:	mov    r11,QWORD PTR [rsp+0x8]
   18857:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1885d:	mov    rsi,QWORD PTR [rdx+0x10]
   18861:	cmp    rbx,rsi
   18864:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1886a:	mov    rax,QWORD PTR [rdx+0x38]
   1886e:	mov    rcx,QWORD PTR [rdx+0x8]
   18872:	lea    rdx,[rbx+rbx*2]
   18876:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1887b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1887e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18882:	mov    BYTE PTR [rsi],cl
   18884:	shr    ecx,0x8
   18887:	and    ecx,0x7fffff
   1888d:	lea    rdi,[rsi+0x1]
   18891:	mov    rax,QWORD PTR [rax+rdx*8]
   18895:	add    rax,rsi
   18898:	mov    QWORD PTR [rsp+0x328],rbx
   188a0:	mov    QWORD PTR [rsp+0x330],rdi
   188a8:	mov    QWORD PTR [rsp+0x338],rax
   188b0:	mov    DWORD PTR [rsp+0x340],ecx
   188b7:	prefetcht0 BYTE PTR [r15+rcx*4]
   188bc:	inc    rbx
   188bf:	mov    rax,r8
   188c2:	not    rax
   188c5:	add    rax,rsi
   188c8:	inc    rax
   188cb:	mov    QWORD PTR [rsp+0x78],rax
   188d0:	mov    QWORD PTR [rsp+0x5c8],rax
   188d8:	mov    r14d,DWORD PTR [rsp+0x360]
   188e0:	mov    eax,DWORD PTR [r15+r14*4]
   188e4:	test   eax,eax
   188e6:	jns    16fc1 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xfc1>
   188ec:	mov    r11,QWORD PTR [rsp+0x70]
   188f1:	add    r11,r8
   188f4:	mov    QWORD PTR [rsp+0x350],r11
   188fc:	mov    rdx,QWORD PTR [rsp]
   18900:	mov    rsi,QWORD PTR [rdx+0x40]
   18904:	mov    rdi,QWORD PTR [rsp+0x348]
   1890c:	cmp    rdi,rsi
   1890f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18915:	mov    rax,QWORD PTR [rdx+0x38]
   18919:	mov    r13,QWORD PTR [rdx+0x50]
   1891d:	lea    rcx,[rdi+rdi*2]
   18921:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18926:	mov    rsi,QWORD PTR [rdx+0x58]
   1892a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   1892f:	mov    rax,QWORD PTR [rdx+0x10]
   18933:	test   rax,rax
   18936:	je     1898d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x298d>
   18938:	mov    rcx,QWORD PTR [rdx+0x8]
   1893c:	xor    r11d,r11d
   1893f:	cmp    rax,0x1
   18943:	je     1896d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x296d>
   18945:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   18950:	mov    rdx,r11
   18953:	mov    r9,rax
   18956:	shr    r9,1
   18959:	add    r11,r9
   1895c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18960:	cmova  r11,rdx
   18964:	sub    rax,r9
   18967:	cmp    rax,0x1
   1896b:	ja     18950 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2950>
   1896d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18971:	mov    rax,0xffffffffffffffff
   18978:	cmovne r11,rax
   1897c:	mov    rdx,QWORD PTR [rsp]
   18980:	mov    r9,r10
   18983:	cmp    rdi,rsi
   18986:	jb     1899d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x299d>
   18988:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1898d:	mov    r11,0xffffffffffffffff
   18994:	cmp    rdi,rsi
   18997:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1899d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   189a2:	cmp    rbx,r12
   189a5:	je     1b68c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x568c>
   189ab:	mov    rsi,QWORD PTR [rdx+0x40]
   189af:	cmp    rbx,rsi
   189b2:	mov    r11,QWORD PTR [rsp+0x8]
   189b7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   189bd:	mov    rsi,QWORD PTR [rdx+0x10]
   189c1:	cmp    rbx,rsi
   189c4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   189ca:	mov    rax,QWORD PTR [rdx+0x38]
   189ce:	mov    rcx,QWORD PTR [rdx+0x8]
   189d2:	lea    rdx,[rbx+rbx*2]
   189d6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   189db:	mov    ecx,DWORD PTR [rcx+rbx*4]
   189de:	mov    ecx,DWORD PTR [r15+rcx*4]
   189e2:	mov    BYTE PTR [rsi],cl
   189e4:	shr    ecx,0x8
   189e7:	and    ecx,0x7fffff
   189ed:	lea    rdi,[rsi+0x1]
   189f1:	mov    rax,QWORD PTR [rax+rdx*8]
   189f5:	add    rax,rsi
   189f8:	mov    QWORD PTR [rsp+0x348],rbx
   18a00:	mov    QWORD PTR [rsp+0x350],rdi
   18a08:	mov    QWORD PTR [rsp+0x358],rax
   18a10:	mov    DWORD PTR [rsp+0x360],ecx
   18a17:	prefetcht0 BYTE PTR [r15+rcx*4]
   18a1c:	inc    rbx
   18a1f:	mov    rax,r8
   18a22:	not    rax
   18a25:	add    rax,rsi
   18a28:	inc    rax
   18a2b:	mov    QWORD PTR [rsp+0x70],rax
   18a30:	mov    QWORD PTR [rsp+0x5d0],rax
   18a38:	mov    r14d,DWORD PTR [rsp+0x380]
   18a40:	mov    eax,DWORD PTR [r15+r14*4]
   18a44:	test   eax,eax
   18a46:	jns    16fef <mibz2::bwt::room_segments_unrolled::<32, 1>+0xfef>
   18a4c:	mov    r11,QWORD PTR [rsp+0x68]
   18a51:	add    r11,r8
   18a54:	mov    QWORD PTR [rsp+0x370],r11
   18a5c:	mov    rdx,QWORD PTR [rsp]
   18a60:	mov    rsi,QWORD PTR [rdx+0x40]
   18a64:	mov    rdi,QWORD PTR [rsp+0x368]
   18a6c:	cmp    rdi,rsi
   18a6f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18a75:	mov    rax,QWORD PTR [rdx+0x38]
   18a79:	mov    r13,QWORD PTR [rdx+0x50]
   18a7d:	lea    rcx,[rdi+rdi*2]
   18a81:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18a86:	mov    rsi,QWORD PTR [rdx+0x58]
   18a8a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18a8f:	mov    rax,QWORD PTR [rdx+0x10]
   18a93:	test   rax,rax
   18a96:	je     18aed <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2aed>
   18a98:	mov    rcx,QWORD PTR [rdx+0x8]
   18a9c:	xor    r11d,r11d
   18a9f:	cmp    rax,0x1
   18aa3:	je     18acd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2acd>
   18aa5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   18ab0:	mov    rdx,r11
   18ab3:	mov    r9,rax
   18ab6:	shr    r9,1
   18ab9:	add    r11,r9
   18abc:	cmp    DWORD PTR [rcx+r11*4],r14d
   18ac0:	cmova  r11,rdx
   18ac4:	sub    rax,r9
   18ac7:	cmp    rax,0x1
   18acb:	ja     18ab0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2ab0>
   18acd:	cmp    DWORD PTR [rcx+r11*4],r14d
   18ad1:	mov    rax,0xffffffffffffffff
   18ad8:	cmovne r11,rax
   18adc:	mov    rdx,QWORD PTR [rsp]
   18ae0:	mov    r9,r10
   18ae3:	cmp    rdi,rsi
   18ae6:	jb     18afd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2afd>
   18ae8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18aed:	mov    r11,0xffffffffffffffff
   18af4:	cmp    rdi,rsi
   18af7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18afd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18b02:	cmp    rbx,r12
   18b05:	je     1b7dc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x57dc>
   18b0b:	mov    rsi,QWORD PTR [rdx+0x40]
   18b0f:	cmp    rbx,rsi
   18b12:	mov    r11,QWORD PTR [rsp+0x8]
   18b17:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18b1d:	mov    rsi,QWORD PTR [rdx+0x10]
   18b21:	cmp    rbx,rsi
   18b24:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18b2a:	mov    rax,QWORD PTR [rdx+0x38]
   18b2e:	mov    rcx,QWORD PTR [rdx+0x8]
   18b32:	lea    rdx,[rbx+rbx*2]
   18b36:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   18b3b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   18b3e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18b42:	mov    BYTE PTR [rsi],cl
   18b44:	shr    ecx,0x8
   18b47:	and    ecx,0x7fffff
   18b4d:	lea    rdi,[rsi+0x1]
   18b51:	mov    rax,QWORD PTR [rax+rdx*8]
   18b55:	add    rax,rsi
   18b58:	mov    QWORD PTR [rsp+0x368],rbx
   18b60:	mov    QWORD PTR [rsp+0x370],rdi
   18b68:	mov    QWORD PTR [rsp+0x378],rax
   18b70:	mov    DWORD PTR [rsp+0x380],ecx
   18b77:	prefetcht0 BYTE PTR [r15+rcx*4]
   18b7c:	inc    rbx
   18b7f:	mov    rax,r8
   18b82:	not    rax
   18b85:	add    rax,rsi
   18b88:	inc    rax
   18b8b:	mov    QWORD PTR [rsp+0x68],rax
   18b90:	mov    QWORD PTR [rsp+0x5d8],rax
   18b98:	mov    r14d,DWORD PTR [rsp+0x3a0]
   18ba0:	mov    eax,DWORD PTR [r15+r14*4]
   18ba4:	test   eax,eax
   18ba6:	jns    1701d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x101d>
   18bac:	mov    r11,QWORD PTR [rsp+0x60]
   18bb1:	add    r11,r8
   18bb4:	mov    QWORD PTR [rsp+0x390],r11
   18bbc:	mov    rdx,QWORD PTR [rsp]
   18bc0:	mov    rsi,QWORD PTR [rdx+0x40]
   18bc4:	mov    rdi,QWORD PTR [rsp+0x388]
   18bcc:	cmp    rdi,rsi
   18bcf:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18bd5:	mov    rax,QWORD PTR [rdx+0x38]
   18bd9:	mov    r13,QWORD PTR [rdx+0x50]
   18bdd:	lea    rcx,[rdi+rdi*2]
   18be1:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18be6:	mov    rsi,QWORD PTR [rdx+0x58]
   18bea:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18bef:	mov    rax,QWORD PTR [rdx+0x10]
   18bf3:	test   rax,rax
   18bf6:	je     18c4d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2c4d>
   18bf8:	mov    rcx,QWORD PTR [rdx+0x8]
   18bfc:	xor    r11d,r11d
   18bff:	cmp    rax,0x1
   18c03:	je     18c2d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2c2d>
   18c05:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   18c10:	mov    rdx,r11
   18c13:	mov    r9,rax
   18c16:	shr    r9,1
   18c19:	add    r11,r9
   18c1c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18c20:	cmova  r11,rdx
   18c24:	sub    rax,r9
   18c27:	cmp    rax,0x1
   18c2b:	ja     18c10 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2c10>
   18c2d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18c31:	mov    rax,0xffffffffffffffff
   18c38:	cmovne r11,rax
   18c3c:	mov    rdx,QWORD PTR [rsp]
   18c40:	mov    r9,r10
   18c43:	cmp    rdi,rsi
   18c46:	jb     18c5d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2c5d>
   18c48:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18c4d:	mov    r11,0xffffffffffffffff
   18c54:	cmp    rdi,rsi
   18c57:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18c5d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18c62:	cmp    rbx,r12
   18c65:	je     1b93c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x593c>
   18c6b:	mov    rsi,QWORD PTR [rdx+0x40]
   18c6f:	cmp    rbx,rsi
   18c72:	mov    r11,QWORD PTR [rsp+0x8]
   18c77:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18c7d:	mov    rsi,QWORD PTR [rdx+0x10]
   18c81:	cmp    rbx,rsi
   18c84:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18c8a:	mov    rax,QWORD PTR [rdx+0x38]
   18c8e:	mov    rcx,QWORD PTR [rdx+0x8]
   18c92:	lea    rdx,[rbx+rbx*2]
   18c96:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   18c9b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   18c9e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18ca2:	mov    BYTE PTR [rsi],cl
   18ca4:	shr    ecx,0x8
   18ca7:	and    ecx,0x7fffff
   18cad:	lea    rdi,[rsi+0x1]
   18cb1:	mov    rax,QWORD PTR [rax+rdx*8]
   18cb5:	add    rax,rsi
   18cb8:	mov    QWORD PTR [rsp+0x388],rbx
   18cc0:	mov    QWORD PTR [rsp+0x390],rdi
   18cc8:	mov    QWORD PTR [rsp+0x398],rax
   18cd0:	mov    DWORD PTR [rsp+0x3a0],ecx
   18cd7:	prefetcht0 BYTE PTR [r15+rcx*4]
   18cdc:	inc    rbx
   18cdf:	mov    rax,r8
   18ce2:	not    rax
   18ce5:	add    rax,rsi
   18ce8:	inc    rax
   18ceb:	mov    QWORD PTR [rsp+0x60],rax
   18cf0:	mov    QWORD PTR [rsp+0x5e0],rax
   18cf8:	mov    r14d,DWORD PTR [rsp+0x3c0]
   18d00:	mov    eax,DWORD PTR [r15+r14*4]
   18d04:	test   eax,eax
   18d06:	jns    1704b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x104b>
   18d0c:	mov    r11,QWORD PTR [rsp+0x58]
   18d11:	add    r11,r8
   18d14:	mov    QWORD PTR [rsp+0x3b0],r11
   18d1c:	mov    rdx,QWORD PTR [rsp]
   18d20:	mov    rsi,QWORD PTR [rdx+0x40]
   18d24:	mov    rdi,QWORD PTR [rsp+0x3a8]
   18d2c:	cmp    rdi,rsi
   18d2f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18d35:	mov    rax,QWORD PTR [rdx+0x38]
   18d39:	mov    r13,QWORD PTR [rdx+0x50]
   18d3d:	lea    rcx,[rdi+rdi*2]
   18d41:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18d46:	mov    rsi,QWORD PTR [rdx+0x58]
   18d4a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18d4f:	mov    rax,QWORD PTR [rdx+0x10]
   18d53:	test   rax,rax
   18d56:	je     18dad <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2dad>
   18d58:	mov    rcx,QWORD PTR [rdx+0x8]
   18d5c:	xor    r11d,r11d
   18d5f:	cmp    rax,0x1
   18d63:	je     18d8d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2d8d>
   18d65:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   18d70:	mov    rdx,r11
   18d73:	mov    r9,rax
   18d76:	shr    r9,1
   18d79:	add    r11,r9
   18d7c:	cmp    DWORD PTR [rcx+r11*4],r14d
   18d80:	cmova  r11,rdx
   18d84:	sub    rax,r9
   18d87:	cmp    rax,0x1
   18d8b:	ja     18d70 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2d70>
   18d8d:	cmp    DWORD PTR [rcx+r11*4],r14d
   18d91:	mov    rax,0xffffffffffffffff
   18d98:	cmovne r11,rax
   18d9c:	mov    rdx,QWORD PTR [rsp]
   18da0:	mov    r9,r10
   18da3:	cmp    rdi,rsi
   18da6:	jb     18dbd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2dbd>
   18da8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18dad:	mov    r11,0xffffffffffffffff
   18db4:	cmp    rdi,rsi
   18db7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18dbd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18dc2:	cmp    rbx,r12
   18dc5:	je     1baac <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5aac>
   18dcb:	mov    rsi,QWORD PTR [rdx+0x40]
   18dcf:	cmp    rbx,rsi
   18dd2:	mov    r11,QWORD PTR [rsp+0x8]
   18dd7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18ddd:	mov    rsi,QWORD PTR [rdx+0x10]
   18de1:	cmp    rbx,rsi
   18de4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18dea:	mov    rax,QWORD PTR [rdx+0x38]
   18dee:	mov    rcx,QWORD PTR [rdx+0x8]
   18df2:	lea    rdx,[rbx+rbx*2]
   18df6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   18dfb:	mov    ecx,DWORD PTR [rcx+rbx*4]
   18dfe:	mov    ecx,DWORD PTR [r15+rcx*4]
   18e02:	mov    BYTE PTR [rsi],cl
   18e04:	shr    ecx,0x8
   18e07:	and    ecx,0x7fffff
   18e0d:	lea    rdi,[rsi+0x1]
   18e11:	mov    rax,QWORD PTR [rax+rdx*8]
   18e15:	add    rax,rsi
   18e18:	mov    QWORD PTR [rsp+0x3a8],rbx
   18e20:	mov    QWORD PTR [rsp+0x3b0],rdi
   18e28:	mov    QWORD PTR [rsp+0x3b8],rax
   18e30:	mov    DWORD PTR [rsp+0x3c0],ecx
   18e37:	prefetcht0 BYTE PTR [r15+rcx*4]
   18e3c:	inc    rbx
   18e3f:	mov    rax,r8
   18e42:	not    rax
   18e45:	add    rax,rsi
   18e48:	inc    rax
   18e4b:	mov    QWORD PTR [rsp+0x58],rax
   18e50:	mov    QWORD PTR [rsp+0x5e8],rax
   18e58:	mov    r14d,DWORD PTR [rsp+0x3e0]
   18e60:	mov    eax,DWORD PTR [r15+r14*4]
   18e64:	test   eax,eax
   18e66:	jns    17079 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1079>
   18e6c:	mov    r11,QWORD PTR [rsp+0x50]
   18e71:	add    r11,r8
   18e74:	mov    QWORD PTR [rsp+0x3d0],r11
   18e7c:	mov    rdx,QWORD PTR [rsp]
   18e80:	mov    rsi,QWORD PTR [rdx+0x40]
   18e84:	mov    rdi,QWORD PTR [rsp+0x3c8]
   18e8c:	cmp    rdi,rsi
   18e8f:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18e95:	mov    rax,QWORD PTR [rdx+0x38]
   18e99:	mov    r13,QWORD PTR [rdx+0x50]
   18e9d:	lea    rcx,[rdi+rdi*2]
   18ea1:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   18ea6:	mov    rsi,QWORD PTR [rdx+0x58]
   18eaa:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   18eaf:	mov    rax,QWORD PTR [rdx+0x10]
   18eb3:	test   rax,rax
   18eb6:	je     18f0d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2f0d>
   18eb8:	mov    rcx,QWORD PTR [rdx+0x8]
   18ebc:	xor    r11d,r11d
   18ebf:	cmp    rax,0x1
   18ec3:	je     18eed <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2eed>
   18ec5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   18ed0:	mov    rdx,r11
   18ed3:	mov    r9,rax
   18ed6:	shr    r9,1
   18ed9:	add    r11,r9
   18edc:	cmp    DWORD PTR [rcx+r11*4],r14d
   18ee0:	cmova  r11,rdx
   18ee4:	sub    rax,r9
   18ee7:	cmp    rax,0x1
   18eeb:	ja     18ed0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2ed0>
   18eed:	cmp    DWORD PTR [rcx+r11*4],r14d
   18ef1:	mov    rax,0xffffffffffffffff
   18ef8:	cmovne r11,rax
   18efc:	mov    rdx,QWORD PTR [rsp]
   18f00:	mov    r9,r10
   18f03:	cmp    rdi,rsi
   18f06:	jb     18f1d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x2f1d>
   18f08:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18f0d:	mov    r11,0xffffffffffffffff
   18f14:	cmp    rdi,rsi
   18f17:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   18f1d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   18f22:	cmp    rbx,r12
   18f25:	je     1bc2c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5c2c>
   18f2b:	mov    rsi,QWORD PTR [rdx+0x40]
   18f2f:	cmp    rbx,rsi
   18f32:	mov    r11,QWORD PTR [rsp+0x8]
   18f37:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   18f3d:	mov    rsi,QWORD PTR [rdx+0x10]
   18f41:	cmp    rbx,rsi
   18f44:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   18f4a:	mov    rax,QWORD PTR [rdx+0x38]
   18f4e:	mov    rcx,QWORD PTR [rdx+0x8]
   18f52:	lea    rdx,[rbx+rbx*2]
   18f56:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   18f5b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   18f5e:	mov    ecx,DWORD PTR [r15+rcx*4]
   18f62:	mov    BYTE PTR [rsi],cl
   18f64:	shr    ecx,0x8
   18f67:	and    ecx,0x7fffff
   18f6d:	lea    rdi,[rsi+0x1]
   18f71:	mov    rax,QWORD PTR [rax+rdx*8]
   18f75:	add    rax,rsi
   18f78:	mov    QWORD PTR [rsp+0x3c8],rbx
   18f80:	mov    QWORD PTR [rsp+0x3d0],rdi
   18f88:	mov    QWORD PTR [rsp+0x3d8],rax
   18f90:	mov    DWORD PTR [rsp+0x3e0],ecx
   18f97:	prefetcht0 BYTE PTR [r15+rcx*4]
   18f9c:	inc    rbx
   18f9f:	mov    rax,r8
   18fa2:	not    rax
   18fa5:	add    rax,rsi
   18fa8:	inc    rax
   18fab:	mov    QWORD PTR [rsp+0x50],rax
   18fb0:	mov    QWORD PTR [rsp+0x5f0],rax
   18fb8:	mov    r14d,DWORD PTR [rsp+0x400]
   18fc0:	mov    eax,DWORD PTR [r15+r14*4]
   18fc4:	test   eax,eax
   18fc6:	jns    170a7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x10a7>
   18fcc:	mov    r11,QWORD PTR [rsp+0x48]
   18fd1:	add    r11,r8
   18fd4:	mov    QWORD PTR [rsp+0x3f0],r11
   18fdc:	mov    rdx,QWORD PTR [rsp]
   18fe0:	mov    rsi,QWORD PTR [rdx+0x40]
   18fe4:	mov    rdi,QWORD PTR [rsp+0x3e8]
   18fec:	cmp    rdi,rsi
   18fef:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   18ff5:	mov    rax,QWORD PTR [rdx+0x38]
   18ff9:	mov    r13,QWORD PTR [rdx+0x50]
   18ffd:	lea    rcx,[rdi+rdi*2]
   19001:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   19006:	mov    rsi,QWORD PTR [rdx+0x58]
   1900a:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   1900f:	mov    rax,QWORD PTR [rdx+0x10]
   19013:	test   rax,rax
   19016:	je     1906d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x306d>
   19018:	mov    rcx,QWORD PTR [rdx+0x8]
   1901c:	xor    r11d,r11d
   1901f:	cmp    rax,0x1
   19023:	je     1904d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x304d>
   19025:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   19030:	mov    rdx,r11
   19033:	mov    r9,rax
   19036:	shr    r9,1
   19039:	add    r11,r9
   1903c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19040:	cmova  r11,rdx
   19044:	sub    rax,r9
   19047:	cmp    rax,0x1
   1904b:	ja     19030 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3030>
   1904d:	cmp    DWORD PTR [rcx+r11*4],r14d
   19051:	mov    rax,0xffffffffffffffff
   19058:	cmovne r11,rax
   1905c:	mov    rdx,QWORD PTR [rsp]
   19060:	mov    r9,r10
   19063:	cmp    rdi,rsi
   19066:	jb     1907d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x307d>
   19068:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1906d:	mov    r11,0xffffffffffffffff
   19074:	cmp    rdi,rsi
   19077:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1907d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   19082:	cmp    rbx,r12
   19085:	je     1bdbc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5dbc>
   1908b:	mov    rsi,QWORD PTR [rdx+0x40]
   1908f:	cmp    rbx,rsi
   19092:	mov    r11,QWORD PTR [rsp+0x8]
   19097:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1909d:	mov    rsi,QWORD PTR [rdx+0x10]
   190a1:	cmp    rbx,rsi
   190a4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   190aa:	mov    rax,QWORD PTR [rdx+0x38]
   190ae:	mov    rcx,QWORD PTR [rdx+0x8]
   190b2:	lea    rdx,[rbx+rbx*2]
   190b6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   190bb:	mov    ecx,DWORD PTR [rcx+rbx*4]
   190be:	mov    ecx,DWORD PTR [r15+rcx*4]
   190c2:	mov    BYTE PTR [rsi],cl
   190c4:	shr    ecx,0x8
   190c7:	and    ecx,0x7fffff
   190cd:	lea    rdi,[rsi+0x1]
   190d1:	mov    rax,QWORD PTR [rax+rdx*8]
   190d5:	add    rax,rsi
   190d8:	mov    QWORD PTR [rsp+0x3e8],rbx
   190e0:	mov    QWORD PTR [rsp+0x3f0],rdi
   190e8:	mov    QWORD PTR [rsp+0x3f8],rax
   190f0:	mov    DWORD PTR [rsp+0x400],ecx
   190f7:	prefetcht0 BYTE PTR [r15+rcx*4]
   190fc:	inc    rbx
   190ff:	mov    rax,r8
   19102:	not    rax
   19105:	add    rax,rsi
   19108:	inc    rax
   1910b:	mov    QWORD PTR [rsp+0x48],rax
   19110:	mov    QWORD PTR [rsp+0x5f8],rax
   19118:	mov    r14d,DWORD PTR [rsp+0x420]
   19120:	mov    eax,DWORD PTR [r15+r14*4]
   19124:	test   eax,eax
   19126:	jns    170d5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x10d5>
   1912c:	add    r11,r8
   1912f:	mov    QWORD PTR [rsp+0x410],r11
   19137:	mov    rdx,QWORD PTR [rsp]
   1913b:	mov    rsi,QWORD PTR [rdx+0x40]
   1913f:	mov    rdi,QWORD PTR [rsp+0x408]
   19147:	cmp    rdi,rsi
   1914a:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   19150:	mov    rax,QWORD PTR [rdx+0x38]
   19154:	mov    r13,QWORD PTR [rdx+0x50]
   19158:	lea    rcx,[rdi+rdi*2]
   1915c:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   19161:	mov    rsi,QWORD PTR [rdx+0x58]
   19165:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   1916a:	mov    rax,QWORD PTR [rdx+0x10]
   1916e:	test   rax,rax
   19171:	je     191bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x31bd>
   19173:	mov    rcx,QWORD PTR [rdx+0x8]
   19177:	xor    r11d,r11d
   1917a:	cmp    rax,0x1
   1917e:	je     1919d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x319d>
   19180:	mov    rdx,r11
   19183:	mov    r9,rax
   19186:	shr    r9,1
   19189:	add    r11,r9
   1918c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19190:	cmova  r11,rdx
   19194:	sub    rax,r9
   19197:	cmp    rax,0x1
   1919b:	ja     19180 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3180>
   1919d:	cmp    DWORD PTR [rcx+r11*4],r14d
   191a1:	mov    rax,0xffffffffffffffff
   191a8:	cmovne r11,rax
   191ac:	mov    rdx,QWORD PTR [rsp]
   191b0:	mov    r9,r10
   191b3:	cmp    rdi,rsi
   191b6:	jb     191cd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x31cd>
   191b8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   191bd:	mov    r11,0xffffffffffffffff
   191c4:	cmp    rdi,rsi
   191c7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   191cd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   191d2:	cmp    rbx,r12
   191d5:	je     1bf5c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x5f5c>
   191db:	mov    rsi,QWORD PTR [rdx+0x40]
   191df:	cmp    rbx,rsi
   191e2:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   191e8:	mov    rsi,QWORD PTR [rdx+0x10]
   191ec:	cmp    rbx,rsi
   191ef:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   191f5:	mov    rax,QWORD PTR [rdx+0x38]
   191f9:	mov    rcx,QWORD PTR [rdx+0x8]
   191fd:	lea    rdx,[rbx+rbx*2]
   19201:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   19206:	mov    ecx,DWORD PTR [rcx+rbx*4]
   19209:	mov    ecx,DWORD PTR [r15+rcx*4]
   1920d:	mov    BYTE PTR [rsi],cl
   1920f:	shr    ecx,0x8
   19212:	and    ecx,0x7fffff
   19218:	lea    rdi,[rsi+0x1]
   1921c:	mov    rax,QWORD PTR [rax+rdx*8]
   19220:	add    rax,rsi
   19223:	mov    QWORD PTR [rsp+0x408],rbx
   1922b:	mov    QWORD PTR [rsp+0x410],rdi
   19233:	mov    QWORD PTR [rsp+0x418],rax
   1923b:	mov    DWORD PTR [rsp+0x420],ecx
   19242:	prefetcht0 BYTE PTR [r15+rcx*4]
   19247:	inc    rbx
   1924a:	mov    rax,r8
   1924d:	not    rax
   19250:	lea    r11,[rsi+rax*1]
   19254:	inc    r11
   19257:	mov    QWORD PTR [rsp+0x600],r11
   1925f:	mov    r14d,DWORD PTR [rsp+0x440]
   19267:	mov    eax,DWORD PTR [r15+r14*4]
   1926b:	test   eax,eax
   1926d:	mov    QWORD PTR [rsp+0x8],r11
   19272:	js     193c0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x33c0>
   19278:	mov    ecx,eax
   1927a:	shr    ecx,0x8
   1927d:	mov    DWORD PTR [rsp+0x440],ecx
   19284:	prefetcht0 BYTE PTR [r15+rcx*4]
   19289:	mov    rcx,QWORD PTR [rsp+0x18]
   1928e:	mov    BYTE PTR [rcx+r8*1],al
   19292:	mov    r14d,DWORD PTR [rsp+0x460]
   1929a:	mov    eax,DWORD PTR [r15+r14*4]
   1929e:	test   eax,eax
   192a0:	js     1951c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x351c>
   192a6:	mov    ecx,eax
   192a8:	shr    ecx,0x8
   192ab:	mov    DWORD PTR [rsp+0x460],ecx
   192b2:	prefetcht0 BYTE PTR [r15+rcx*4]
   192b7:	mov    rcx,QWORD PTR [rsp+0xc8]
   192bf:	mov    BYTE PTR [rcx+r8*1],al
   192c3:	mov    r14d,DWORD PTR [rsp+0x480]
   192cb:	mov    eax,DWORD PTR [r15+r14*4]
   192cf:	test   eax,eax
   192d1:	js     1967f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x367f>
   192d7:	mov    ecx,eax
   192d9:	shr    ecx,0x8
   192dc:	mov    DWORD PTR [rsp+0x480],ecx
   192e3:	prefetcht0 BYTE PTR [r15+rcx*4]
   192e8:	mov    rcx,QWORD PTR [rsp+0xb8]
   192f0:	mov    BYTE PTR [rcx+r8*1],al
   192f4:	mov    r14d,DWORD PTR [rsp+0x4a0]
   192fc:	mov    eax,DWORD PTR [r15+r14*4]
   19300:	test   eax,eax
   19302:	js     197df <mibz2::bwt::room_segments_unrolled::<32, 1>+0x37df>
   19308:	mov    ecx,eax
   1930a:	shr    ecx,0x8
   1930d:	mov    DWORD PTR [rsp+0x4a0],ecx
   19314:	prefetcht0 BYTE PTR [r15+rcx*4]
   19319:	mov    rcx,QWORD PTR [rsp+0xa8]
   19321:	mov    BYTE PTR [rcx+r8*1],al
   19325:	mov    r10,QWORD PTR [rsp+0x18]
   1932a:	mov    r14d,DWORD PTR [rsp+0x4c0]
   19332:	mov    eax,DWORD PTR [r15+r14*4]
   19336:	test   eax,eax
   19338:	js     19949 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3949>
   1933e:	mov    ecx,eax
   19340:	shr    ecx,0x8
   19343:	mov    DWORD PTR [rsp+0x4c0],ecx
   1934a:	prefetcht0 BYTE PTR [r15+rcx*4]
   1934f:	mov    BYTE PTR [r9+r8*1],al
   19353:	mov    r14d,DWORD PTR [rsp+0x4e0]
   1935b:	mov    eax,DWORD PTR [r15+r14*4]
   1935f:	test   eax,eax
   19361:	js     19aa5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3aa5>
   19367:	mov    ecx,eax
   19369:	shr    ecx,0x8
   1936c:	mov    DWORD PTR [rsp+0x4e0],ecx
   19373:	prefetcht0 BYTE PTR [r15+rcx*4]
   19378:	mov    rcx,QWORD PTR [rsp+0x28]
   1937d:	mov    BYTE PTR [rcx+r8*1],al
   19381:	mov    r14d,DWORD PTR [rsp+0x500]
   19389:	mov    eax,DWORD PTR [r15+r14*4]
   1938d:	test   eax,eax
   1938f:	js     19c11 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3c11>
   19395:	mov    ecx,eax
   19397:	shr    ecx,0x8
   1939a:	mov    DWORD PTR [rsp+0x500],ecx
   193a1:	prefetcht0 BYTE PTR [r15+rcx*4]
   193a6:	mov    rcx,QWORD PTR [rsp+0x20]
   193ab:	mov    BYTE PTR [rcx+r8*1],al
   193af:	jmp    19d5b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3d5b>
   193b4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   193c0:	mov    r11,QWORD PTR [rsp+0x18]
   193c5:	add    r11,r8
   193c8:	mov    QWORD PTR [rsp+0x430],r11
   193d0:	mov    rdx,QWORD PTR [rsp]
   193d4:	mov    rsi,QWORD PTR [rdx+0x40]
   193d8:	mov    rdi,QWORD PTR [rsp+0x428]
   193e0:	cmp    rdi,rsi
   193e3:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   193e9:	mov    rax,QWORD PTR [rdx+0x38]
   193ed:	mov    r13,QWORD PTR [rdx+0x50]
   193f1:	lea    rcx,[rdi+rdi*2]
   193f5:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   193fa:	mov    rsi,QWORD PTR [rdx+0x58]
   193fe:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   19403:	mov    rax,QWORD PTR [rdx+0x10]
   19407:	test   rax,rax
   1940a:	je     1945d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x345d>
   1940c:	mov    rcx,QWORD PTR [rdx+0x8]
   19410:	xor    r11d,r11d
   19413:	cmp    rax,0x1
   19417:	je     1943d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x343d>
   19419:	nop    DWORD PTR [rax+0x0]
   19420:	mov    rdx,r11
   19423:	mov    r9,rax
   19426:	shr    r9,1
   19429:	add    r11,r9
   1942c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19430:	cmova  r11,rdx
   19434:	sub    rax,r9
   19437:	cmp    rax,0x1
   1943b:	ja     19420 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3420>
   1943d:	cmp    DWORD PTR [rcx+r11*4],r14d
   19441:	mov    rax,0xffffffffffffffff
   19448:	cmovne r11,rax
   1944c:	mov    rdx,QWORD PTR [rsp]
   19450:	mov    r9,r10
   19453:	cmp    rdi,rsi
   19456:	jb     1946d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x346d>
   19458:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1945d:	mov    r11,0xffffffffffffffff
   19464:	cmp    rdi,rsi
   19467:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1946d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   19472:	cmp    rbx,r12
   19475:	je     1c10c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x610c>
   1947b:	mov    rsi,QWORD PTR [rdx+0x40]
   1947f:	cmp    rbx,rsi
   19482:	mov    r11,QWORD PTR [rsp+0x8]
   19487:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1948d:	mov    rsi,QWORD PTR [rdx+0x10]
   19491:	cmp    rbx,rsi
   19494:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1949a:	mov    rax,QWORD PTR [rdx+0x38]
   1949e:	mov    rcx,QWORD PTR [rdx+0x8]
   194a2:	lea    rdx,[rbx+rbx*2]
   194a6:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   194ab:	mov    ecx,DWORD PTR [rcx+rbx*4]
   194ae:	mov    ecx,DWORD PTR [r15+rcx*4]
   194b2:	mov    BYTE PTR [rsi],cl
   194b4:	shr    ecx,0x8
   194b7:	and    ecx,0x7fffff
   194bd:	lea    rdi,[rsi+0x1]
   194c1:	mov    rax,QWORD PTR [rax+rdx*8]
   194c5:	add    rax,rsi
   194c8:	mov    QWORD PTR [rsp+0x428],rbx
   194d0:	mov    QWORD PTR [rsp+0x430],rdi
   194d8:	mov    QWORD PTR [rsp+0x438],rax
   194e0:	mov    DWORD PTR [rsp+0x440],ecx
   194e7:	prefetcht0 BYTE PTR [r15+rcx*4]
   194ec:	inc    rbx
   194ef:	mov    rax,r8
   194f2:	not    rax
   194f5:	add    rax,rsi
   194f8:	inc    rax
   194fb:	mov    QWORD PTR [rsp+0x18],rax
   19500:	mov    QWORD PTR [rsp+0x608],rax
   19508:	mov    r14d,DWORD PTR [rsp+0x460]
   19510:	mov    eax,DWORD PTR [r15+r14*4]
   19514:	test   eax,eax
   19516:	jns    192a6 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x32a6>
   1951c:	mov    r11,QWORD PTR [rsp+0xc8]
   19524:	add    r11,r8
   19527:	mov    QWORD PTR [rsp+0x450],r11
   1952f:	mov    rdx,QWORD PTR [rsp]
   19533:	mov    rsi,QWORD PTR [rdx+0x40]
   19537:	mov    rdi,QWORD PTR [rsp+0x448]
   1953f:	cmp    rdi,rsi
   19542:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   19548:	mov    rax,QWORD PTR [rdx+0x38]
   1954c:	mov    r13,QWORD PTR [rdx+0x50]
   19550:	lea    rcx,[rdi+rdi*2]
   19554:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   19559:	mov    rsi,QWORD PTR [rdx+0x58]
   1955d:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   19562:	mov    rax,QWORD PTR [rdx+0x10]
   19566:	test   rax,rax
   19569:	je     195bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x35bd>
   1956b:	mov    rcx,QWORD PTR [rdx+0x8]
   1956f:	xor    r11d,r11d
   19572:	cmp    rax,0x1
   19576:	je     1959d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x359d>
   19578:	nop    DWORD PTR [rax+rax*1+0x0]
   19580:	mov    rdx,r11
   19583:	mov    r9,rax
   19586:	shr    r9,1
   19589:	add    r11,r9
   1958c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19590:	cmova  r11,rdx
   19594:	sub    rax,r9
   19597:	cmp    rax,0x1
   1959b:	ja     19580 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3580>
   1959d:	cmp    DWORD PTR [rcx+r11*4],r14d
   195a1:	mov    rax,0xffffffffffffffff
   195a8:	cmovne r11,rax
   195ac:	mov    rdx,QWORD PTR [rsp]
   195b0:	mov    r9,r10
   195b3:	cmp    rdi,rsi
   195b6:	jb     195cd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x35cd>
   195b8:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   195bd:	mov    r11,0xffffffffffffffff
   195c4:	cmp    rdi,rsi
   195c7:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   195cd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   195d2:	cmp    rbx,r12
   195d5:	je     1c2cc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x62cc>
   195db:	mov    rsi,QWORD PTR [rdx+0x40]
   195df:	cmp    rbx,rsi
   195e2:	mov    r11,QWORD PTR [rsp+0x8]
   195e7:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   195ed:	mov    rsi,QWORD PTR [rdx+0x10]
   195f1:	cmp    rbx,rsi
   195f4:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   195fa:	mov    rax,QWORD PTR [rdx+0x38]
   195fe:	mov    rcx,QWORD PTR [rdx+0x8]
   19602:	lea    rdx,[rbx+rbx*2]
   19606:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1960b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1960e:	mov    ecx,DWORD PTR [r15+rcx*4]
   19612:	mov    BYTE PTR [rsi],cl
   19614:	shr    ecx,0x8
   19617:	and    ecx,0x7fffff
   1961d:	lea    rdi,[rsi+0x1]
   19621:	mov    rax,QWORD PTR [rax+rdx*8]
   19625:	add    rax,rsi
   19628:	mov    QWORD PTR [rsp+0x448],rbx
   19630:	mov    QWORD PTR [rsp+0x450],rdi
   19638:	mov    QWORD PTR [rsp+0x458],rax
   19640:	mov    DWORD PTR [rsp+0x460],ecx
   19647:	prefetcht0 BYTE PTR [r15+rcx*4]
   1964c:	inc    rbx
   1964f:	mov    rax,r8
   19652:	not    rax
   19655:	add    rax,rsi
   19658:	inc    rax
   1965b:	mov    QWORD PTR [rsp+0xc8],rax
   19663:	mov    QWORD PTR [rsp+0x610],rax
   1966b:	mov    r14d,DWORD PTR [rsp+0x480]
   19673:	mov    eax,DWORD PTR [r15+r14*4]
   19677:	test   eax,eax
   19679:	jns    192d7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x32d7>
   1967f:	mov    r11,QWORD PTR [rsp+0xb8]
   19687:	add    r11,r8
   1968a:	mov    QWORD PTR [rsp+0x470],r11
   19692:	mov    rdx,QWORD PTR [rsp]
   19696:	mov    rsi,QWORD PTR [rdx+0x40]
   1969a:	mov    rdi,QWORD PTR [rsp+0x468]
   196a2:	cmp    rdi,rsi
   196a5:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   196ab:	mov    rax,QWORD PTR [rdx+0x38]
   196af:	mov    r13,QWORD PTR [rdx+0x50]
   196b3:	lea    rcx,[rdi+rdi*2]
   196b7:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   196bc:	mov    rsi,QWORD PTR [rdx+0x58]
   196c0:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   196c5:	mov    rax,QWORD PTR [rdx+0x10]
   196c9:	test   rax,rax
   196cc:	je     1971d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x371d>
   196ce:	mov    rcx,QWORD PTR [rdx+0x8]
   196d2:	xor    r11d,r11d
   196d5:	cmp    rax,0x1
   196d9:	je     196fd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x36fd>
   196db:	nop    DWORD PTR [rax+rax*1+0x0]
   196e0:	mov    rdx,r11
   196e3:	mov    r9,rax
   196e6:	shr    r9,1
   196e9:	add    r11,r9
   196ec:	cmp    DWORD PTR [rcx+r11*4],r14d
   196f0:	cmova  r11,rdx
   196f4:	sub    rax,r9
   196f7:	cmp    rax,0x1
   196fb:	ja     196e0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x36e0>
   196fd:	cmp    DWORD PTR [rcx+r11*4],r14d
   19701:	mov    rax,0xffffffffffffffff
   19708:	cmovne r11,rax
   1970c:	mov    rdx,QWORD PTR [rsp]
   19710:	mov    r9,r10
   19713:	cmp    rdi,rsi
   19716:	jb     1972d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x372d>
   19718:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1971d:	mov    r11,0xffffffffffffffff
   19724:	cmp    rdi,rsi
   19727:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   1972d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   19732:	cmp    rbx,r12
   19735:	je     1c49c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x649c>
   1973b:	mov    rsi,QWORD PTR [rdx+0x40]
   1973f:	cmp    rbx,rsi
   19742:	mov    r11,QWORD PTR [rsp+0x8]
   19747:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   1974d:	mov    rsi,QWORD PTR [rdx+0x10]
   19751:	cmp    rbx,rsi
   19754:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   1975a:	mov    rax,QWORD PTR [rdx+0x38]
   1975e:	mov    rcx,QWORD PTR [rdx+0x8]
   19762:	lea    rdx,[rbx+rbx*2]
   19766:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   1976b:	mov    ecx,DWORD PTR [rcx+rbx*4]
   1976e:	mov    ecx,DWORD PTR [r15+rcx*4]
   19772:	mov    BYTE PTR [rsi],cl
   19774:	shr    ecx,0x8
   19777:	and    ecx,0x7fffff
   1977d:	lea    rdi,[rsi+0x1]
   19781:	mov    rax,QWORD PTR [rax+rdx*8]
   19785:	add    rax,rsi
   19788:	mov    QWORD PTR [rsp+0x468],rbx
   19790:	mov    QWORD PTR [rsp+0x470],rdi
   19798:	mov    QWORD PTR [rsp+0x478],rax
   197a0:	mov    DWORD PTR [rsp+0x480],ecx
   197a7:	prefetcht0 BYTE PTR [r15+rcx*4]
   197ac:	inc    rbx
   197af:	mov    rax,r8
   197b2:	not    rax
   197b5:	add    rax,rsi
   197b8:	inc    rax
   197bb:	mov    QWORD PTR [rsp+0xb8],rax
   197c3:	mov    QWORD PTR [rsp+0x618],rax
   197cb:	mov    r14d,DWORD PTR [rsp+0x4a0]
   197d3:	mov    eax,DWORD PTR [r15+r14*4]
   197d7:	test   eax,eax
   197d9:	jns    19308 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3308>
   197df:	mov    r11,QWORD PTR [rsp+0xa8]
   197e7:	add    r11,r8
   197ea:	mov    QWORD PTR [rsp+0x490],r11
   197f2:	mov    rdx,QWORD PTR [rsp]
   197f6:	mov    rsi,QWORD PTR [rdx+0x40]
   197fa:	mov    rdi,QWORD PTR [rsp+0x488]
   19802:	cmp    rdi,rsi
   19805:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1980b:	mov    rax,QWORD PTR [rdx+0x38]
   1980f:	mov    r13,QWORD PTR [rdx+0x50]
   19813:	lea    rcx,[rdi+rdi*2]
   19817:	sub    r11,QWORD PTR [rax+rcx*8+0x8]
   1981c:	mov    rsi,QWORD PTR [rdx+0x58]
   19820:	mov    QWORD PTR [rax+rcx*8+0x10],r11
   19825:	mov    rax,QWORD PTR [rdx+0x10]
   19829:	test   rax,rax
   1982c:	je     19882 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3882>
   1982e:	mov    rcx,QWORD PTR [rdx+0x8]
   19832:	xor    r11d,r11d
   19835:	cmp    rax,0x1
   19839:	je     1985d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x385d>
   1983b:	nop    DWORD PTR [rax+rax*1+0x0]
   19840:	mov    rdx,r11
   19843:	mov    r9,rax
   19846:	shr    r9,1
   19849:	add    r11,r9
   1984c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19850:	cmova  r11,rdx
   19854:	sub    rax,r9
   19857:	cmp    rax,0x1
   1985b:	ja     19840 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3840>
   1985d:	cmp    DWORD PTR [rcx+r11*4],r14d
   19861:	mov    rax,0xffffffffffffffff
   19868:	cmovne r11,rax
   1986c:	mov    rdx,QWORD PTR [rsp]
   19870:	mov    r9,r10
   19873:	cmp    rdi,rsi
   19876:	mov    r10,QWORD PTR [rsp+0x18]
   1987b:	jb     19897 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3897>
   1987d:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19882:	mov    r11,0xffffffffffffffff
   19889:	cmp    rdi,rsi
   1988c:	mov    r10,QWORD PTR [rsp+0x18]
   19891:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19897:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   1989c:	cmp    rbx,r12
   1989f:	je     1c67f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x667f>
   198a5:	mov    rsi,QWORD PTR [rdx+0x40]
   198a9:	cmp    rbx,rsi
   198ac:	mov    r11,QWORD PTR [rsp+0x8]
   198b1:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   198b7:	mov    rsi,QWORD PTR [rdx+0x10]
   198bb:	cmp    rbx,rsi
   198be:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   198c4:	mov    rax,QWORD PTR [rdx+0x38]
   198c8:	mov    rcx,QWORD PTR [rdx+0x8]
   198cc:	lea    rdx,[rbx+rbx*2]
   198d0:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   198d5:	mov    ecx,DWORD PTR [rcx+rbx*4]
   198d8:	mov    ecx,DWORD PTR [r15+rcx*4]
   198dc:	mov    BYTE PTR [rsi],cl
   198de:	shr    ecx,0x8
   198e1:	and    ecx,0x7fffff
   198e7:	lea    rdi,[rsi+0x1]
   198eb:	mov    rax,QWORD PTR [rax+rdx*8]
   198ef:	add    rax,rsi
   198f2:	mov    QWORD PTR [rsp+0x488],rbx
   198fa:	mov    QWORD PTR [rsp+0x490],rdi
   19902:	mov    QWORD PTR [rsp+0x498],rax
   1990a:	mov    DWORD PTR [rsp+0x4a0],ecx
   19911:	prefetcht0 BYTE PTR [r15+rcx*4]
   19916:	inc    rbx
   19919:	mov    rax,r8
   1991c:	not    rax
   1991f:	add    rax,rsi
   19922:	inc    rax
   19925:	mov    QWORD PTR [rsp+0xa8],rax
   1992d:	mov    QWORD PTR [rsp+0x620],rax
   19935:	mov    r14d,DWORD PTR [rsp+0x4c0]
   1993d:	mov    eax,DWORD PTR [r15+r14*4]
   19941:	test   eax,eax
   19943:	jns    1933e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x333e>
   19949:	add    r9,r8
   1994c:	mov    QWORD PTR [rsp+0x4b0],r9
   19954:	mov    rdx,QWORD PTR [rsp]
   19958:	mov    rsi,QWORD PTR [rdx+0x40]
   1995c:	mov    rdi,QWORD PTR [rsp+0x4a8]
   19964:	cmp    rdi,rsi
   19967:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   1996d:	mov    rax,QWORD PTR [rdx+0x38]
   19971:	mov    r13,QWORD PTR [rdx+0x50]
   19975:	lea    rcx,[rdi+rdi*2]
   19979:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   1997e:	mov    rsi,QWORD PTR [rdx+0x58]
   19982:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   19987:	mov    rax,QWORD PTR [rdx+0x10]
   1998b:	test   rax,rax
   1998e:	je     199d2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x39d2>
   19990:	mov    rcx,QWORD PTR [rdx+0x8]
   19994:	xor    r11d,r11d
   19997:	cmp    rax,0x1
   1999b:	je     199bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x39bd>
   1999d:	nop    DWORD PTR [rax]
   199a0:	mov    rdx,r11
   199a3:	mov    r9,rax
   199a6:	shr    r9,1
   199a9:	add    r11,r9
   199ac:	cmp    DWORD PTR [rcx+r11*4],r14d
   199b0:	cmova  r11,rdx
   199b4:	sub    rax,r9
   199b7:	cmp    rax,0x1
   199bb:	ja     199a0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x39a0>
   199bd:	cmp    DWORD PTR [rcx+r11*4],r14d
   199c1:	mov    rax,0xffffffffffffffff
   199c8:	cmovne r11,rax
   199cc:	mov    rdx,QWORD PTR [rsp]
   199d0:	jmp    199d9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x39d9>
   199d2:	mov    r11,0xffffffffffffffff
   199d9:	cmp    rdi,rsi
   199dc:	mov    rax,QWORD PTR [rsp+0xa8]
   199e4:	mov    rcx,QWORD PTR [rsp+0xb8]
   199ec:	mov    r9,QWORD PTR [rsp+0xc8]
   199f4:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   199fa:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   199ff:	cmp    rbx,r12
   19a02:	je     1c870 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6870>
   19a08:	mov    rsi,QWORD PTR [rdx+0x40]
   19a0c:	cmp    rbx,rsi
   19a0f:	mov    r11,QWORD PTR [rsp+0x8]
   19a14:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   19a1a:	mov    rsi,QWORD PTR [rdx+0x10]
   19a1e:	cmp    rbx,rsi
   19a21:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   19a27:	mov    rax,QWORD PTR [rdx+0x38]
   19a2b:	mov    rcx,QWORD PTR [rdx+0x8]
   19a2f:	lea    rdx,[rbx+rbx*2]
   19a33:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   19a38:	mov    ecx,DWORD PTR [rcx+rbx*4]
   19a3b:	mov    ecx,DWORD PTR [r15+rcx*4]
   19a3f:	mov    BYTE PTR [rsi],cl
   19a41:	shr    ecx,0x8
   19a44:	and    ecx,0x7fffff
   19a4a:	lea    rdi,[rsi+0x1]
   19a4e:	mov    rax,QWORD PTR [rax+rdx*8]
   19a52:	add    rax,rsi
   19a55:	mov    QWORD PTR [rsp+0x4a8],rbx
   19a5d:	mov    QWORD PTR [rsp+0x4b0],rdi
   19a65:	mov    QWORD PTR [rsp+0x4b8],rax
   19a6d:	mov    DWORD PTR [rsp+0x4c0],ecx
   19a74:	prefetcht0 BYTE PTR [r15+rcx*4]
   19a79:	inc    rbx
   19a7c:	mov    rax,r8
   19a7f:	not    rax
   19a82:	lea    r9,[rsi+rax*1]
   19a86:	inc    r9
   19a89:	mov    QWORD PTR [rsp+0x628],r9
   19a91:	mov    r14d,DWORD PTR [rsp+0x4e0]
   19a99:	mov    eax,DWORD PTR [r15+r14*4]
   19a9d:	test   eax,eax
   19a9f:	jns    19367 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3367>
   19aa5:	mov    r10,r9
   19aa8:	mov    r9,QWORD PTR [rsp+0x28]
   19aad:	add    r9,r8
   19ab0:	mov    QWORD PTR [rsp+0x4d0],r9
   19ab8:	mov    rdx,QWORD PTR [rsp]
   19abc:	mov    rsi,QWORD PTR [rdx+0x40]
   19ac0:	mov    rdi,QWORD PTR [rsp+0x4c8]
   19ac8:	cmp    rdi,rsi
   19acb:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   19ad1:	mov    rax,QWORD PTR [rdx+0x38]
   19ad5:	mov    r13,QWORD PTR [rdx+0x50]
   19ad9:	lea    rcx,[rdi+rdi*2]
   19add:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   19ae2:	mov    rsi,QWORD PTR [rdx+0x58]
   19ae6:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   19aeb:	mov    rax,QWORD PTR [rdx+0x10]
   19aef:	test   rax,rax
   19af2:	je     19b4a <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3b4a>
   19af4:	mov    rcx,QWORD PTR [rdx+0x8]
   19af8:	xor    r11d,r11d
   19afb:	cmp    rax,0x1
   19aff:	je     19b2d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3b2d>
   19b01:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   19b10:	mov    rdx,r11
   19b13:	mov    r9,rax
   19b16:	shr    r9,1
   19b19:	add    r11,r9
   19b1c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19b20:	cmova  r11,rdx
   19b24:	sub    rax,r9
   19b27:	cmp    rax,0x1
   19b2b:	ja     19b10 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3b10>
   19b2d:	cmp    DWORD PTR [rcx+r11*4],r14d
   19b31:	mov    rax,0xffffffffffffffff
   19b38:	cmovne r11,rax
   19b3c:	mov    rdx,QWORD PTR [rsp]
   19b40:	cmp    rdi,rsi
   19b43:	jb     19b5a <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3b5a>
   19b45:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19b4a:	mov    r11,0xffffffffffffffff
   19b51:	cmp    rdi,rsi
   19b54:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19b5a:	mov    r9,r10
   19b5d:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   19b62:	cmp    rbx,r12
   19b65:	mov    r10,QWORD PTR [rsp+0x18]
   19b6a:	je     1ca5c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6a5c>
   19b70:	mov    rsi,QWORD PTR [rdx+0x40]
   19b74:	cmp    rbx,rsi
   19b77:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   19b7d:	mov    rsi,QWORD PTR [rdx+0x10]
   19b81:	cmp    rbx,rsi
   19b84:	mov    r11,QWORD PTR [rsp+0x8]
   19b89:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   19b8f:	mov    rax,QWORD PTR [rdx+0x38]
   19b93:	mov    rcx,QWORD PTR [rdx+0x8]
   19b97:	lea    rdx,[rbx+rbx*2]
   19b9b:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   19ba0:	mov    ecx,DWORD PTR [rcx+rbx*4]
   19ba3:	mov    ecx,DWORD PTR [r15+rcx*4]
   19ba7:	mov    BYTE PTR [rsi],cl
   19ba9:	shr    ecx,0x8
   19bac:	and    ecx,0x7fffff
   19bb2:	lea    rdi,[rsi+0x1]
   19bb6:	mov    rax,QWORD PTR [rax+rdx*8]
   19bba:	add    rax,rsi
   19bbd:	mov    QWORD PTR [rsp+0x4c8],rbx
   19bc5:	mov    QWORD PTR [rsp+0x4d0],rdi
   19bcd:	mov    QWORD PTR [rsp+0x4d8],rax
   19bd5:	mov    DWORD PTR [rsp+0x4e0],ecx
   19bdc:	prefetcht0 BYTE PTR [r15+rcx*4]
   19be1:	inc    rbx
   19be4:	mov    rax,r8
   19be7:	not    rax
   19bea:	add    rax,rsi
   19bed:	inc    rax
   19bf0:	mov    QWORD PTR [rsp+0x28],rax
   19bf5:	mov    QWORD PTR [rsp+0x630],rax
   19bfd:	mov    r14d,DWORD PTR [rsp+0x500]
   19c05:	mov    eax,DWORD PTR [r15+r14*4]
   19c09:	test   eax,eax
   19c0b:	jns    19395 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3395>
   19c11:	mov    r10,r9
   19c14:	mov    r9,QWORD PTR [rsp+0x20]
   19c19:	add    r9,r8
   19c1c:	mov    QWORD PTR [rsp+0x4f0],r9
   19c24:	mov    rdx,QWORD PTR [rsp]
   19c28:	mov    rsi,QWORD PTR [rdx+0x40]
   19c2c:	mov    rdi,QWORD PTR [rsp+0x4e8]
   19c34:	cmp    rdi,rsi
   19c37:	jae    1d263 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7263>
   19c3d:	mov    rax,QWORD PTR [rdx+0x38]
   19c41:	mov    r13,QWORD PTR [rdx+0x50]
   19c45:	lea    rcx,[rdi+rdi*2]
   19c49:	sub    r9,QWORD PTR [rax+rcx*8+0x8]
   19c4e:	mov    rsi,QWORD PTR [rdx+0x58]
   19c52:	mov    QWORD PTR [rax+rcx*8+0x10],r9
   19c57:	mov    rax,QWORD PTR [rdx+0x10]
   19c5b:	test   rax,rax
   19c5e:	je     19caa <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3caa>
   19c60:	mov    rcx,QWORD PTR [rdx+0x8]
   19c64:	xor    r11d,r11d
   19c67:	cmp    rax,0x1
   19c6b:	je     19c8d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3c8d>
   19c6d:	nop    DWORD PTR [rax]
   19c70:	mov    rdx,r11
   19c73:	mov    r9,rax
   19c76:	shr    r9,1
   19c79:	add    r11,r9
   19c7c:	cmp    DWORD PTR [rcx+r11*4],r14d
   19c80:	cmova  r11,rdx
   19c84:	sub    rax,r9
   19c87:	cmp    rax,0x1
   19c8b:	ja     19c70 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3c70>
   19c8d:	cmp    DWORD PTR [rcx+r11*4],r14d
   19c91:	mov    rax,0xffffffffffffffff
   19c98:	cmovne r11,rax
   19c9c:	mov    rdx,QWORD PTR [rsp]
   19ca0:	cmp    rdi,rsi
   19ca3:	jb     19cba <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3cba>
   19ca5:	jmp    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19caa:	mov    r11,0xffffffffffffffff
   19cb1:	cmp    rdi,rsi
   19cb4:	jae    1d257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7257>
   19cba:	mov    r9,r10
   19cbd:	mov    QWORD PTR [r13+rdi*8+0x0],r11
   19cc2:	cmp    rbx,r12
   19cc5:	mov    r10,QWORD PTR [rsp+0x18]
   19cca:	je     1caf0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6af0>
   19cd0:	mov    rsi,QWORD PTR [rdx+0x40]
   19cd4:	cmp    rbx,rsi
   19cd7:	mov    r11,QWORD PTR [rsp+0x8]
   19cdc:	jae    1d26f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x726f>
   19ce2:	mov    rsi,QWORD PTR [rdx+0x10]
   19ce6:	cmp    rbx,rsi
   19ce9:	jae    1d27e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x727e>
   19cef:	mov    rax,QWORD PTR [rdx+0x38]
   19cf3:	mov    rcx,QWORD PTR [rdx+0x8]
   19cf7:	lea    rdx,[rbx+rbx*2]
   19cfb:	mov    rsi,QWORD PTR [rax+rdx*8+0x8]
   19d00:	mov    ecx,DWORD PTR [rcx+rbx*4]
   19d03:	mov    ecx,DWORD PTR [r15+rcx*4]
   19d07:	mov    BYTE PTR [rsi],cl
   19d09:	shr    ecx,0x8
   19d0c:	and    ecx,0x7fffff
   19d12:	lea    rdi,[rsi+0x1]
   19d16:	mov    rax,QWORD PTR [rax+rdx*8]
   19d1a:	add    rax,rsi
   19d1d:	mov    QWORD PTR [rsp+0x4e8],rbx
   19d25:	mov    QWORD PTR [rsp+0x4f0],rdi
   19d2d:	mov    QWORD PTR [rsp+0x4f8],rax
   19d35:	mov    DWORD PTR [rsp+0x500],ecx
   19d3c:	prefetcht0 BYTE PTR [r15+rcx*4]
   19d41:	inc    rbx
   19d44:	not    r8
   19d47:	lea    rax,[rsi+r8*1]
   19d4b:	inc    rax
   19d4e:	mov    QWORD PTR [rsp+0x20],rax
   19d53:	mov    QWORD PTR [rsp+0x638],rax
   19d5b:	mov    rax,QWORD PTR [rsp+0x520]
   19d63:	mov    r8,rax
   19d66:	cmp    rax,QWORD PTR [rsp+0x518]
   19d6e:	mov    rdx,QWORD PTR [rsp]
   19d72:	jne    16ab8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xab8>
   19d78:	jmp    19d7f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x3d7f>
   19d7a:	mov    rbx,QWORD PTR [rsp+0x8]
   19d7f:	mov    rcx,QWORD PTR [rsp+0x518]
   19d87:	mov    rax,QWORD PTR [rsp+0x10]
   19d8c:	add    rax,rcx
   19d8f:	mov    QWORD PTR [rsp+0x110],rax
   19d97:	mov    rax,QWORD PTR [rsp+0xf8]
   19d9f:	add    rax,rcx
   19da2:	mov    QWORD PTR [rsp+0x130],rax
   19daa:	mov    rax,QWORD PTR [rsp+0xf0]
   19db2:	add    rax,rcx
   19db5:	mov    QWORD PTR [rsp+0x150],rax
   19dbd:	mov    rax,QWORD PTR [rsp+0xe8]
   19dc5:	add    rax,rcx
   19dc8:	mov    QWORD PTR [rsp+0x170],rax
   19dd0:	mov    rax,QWORD PTR [rsp+0xe0]
   19dd8:	add    rax,rcx
   19ddb:	mov    QWORD PTR [rsp+0x190],rax
   19de3:	mov    rax,QWORD PTR [rsp+0xd8]
   19deb:	add    rax,rcx
   19dee:	mov    QWORD PTR [rsp+0x1b0],rax
   19df6:	mov    rax,QWORD PTR [rsp+0xd0]
   19dfe:	add    rax,rcx
   19e01:	mov    QWORD PTR [rsp+0x1d0],rax
   19e09:	mov    rax,QWORD PTR [rsp+0xc0]
   19e11:	add    rax,rcx
   19e14:	mov    QWORD PTR [rsp+0x1f0],rax
   19e1c:	mov    rax,QWORD PTR [rsp+0xb0]
   19e24:	add    rax,rcx
   19e27:	mov    QWORD PTR [rsp+0x210],rax
   19e2f:	mov    rax,QWORD PTR [rsp+0xa0]
   19e37:	add    rax,rcx
   19e3a:	mov    QWORD PTR [rsp+0x230],rax
   19e42:	mov    rax,QWORD PTR [rsp+0x98]
   19e4a:	add    rax,rcx
   19e4d:	mov    QWORD PTR [rsp+0x250],rax
   19e55:	mov    rax,QWORD PTR [rsp+0x90]
   19e5d:	add    rax,rcx
   19e60:	mov    QWORD PTR [rsp+0x270],rax
   19e68:	mov    rax,QWORD PTR [rsp+0x40]
   19e6d:	add    rax,rcx
   19e70:	mov    QWORD PTR [rsp+0x290],rax
   19e78:	mov    rax,QWORD PTR [rsp+0x38]
   19e7d:	add    rax,rcx
   19e80:	mov    QWORD PTR [rsp+0x2b0],rax
   19e88:	mov    rax,QWORD PTR [rsp+0x30]
   19e8d:	add    rax,rcx
   19e90:	mov    QWORD PTR [rsp+0x2d0],rax
   19e98:	mov    rax,QWORD PTR [rsp+0x88]
   19ea0:	add    rax,rcx
   19ea3:	mov    QWORD PTR [rsp+0x2f0],rax
   19eab:	mov    rax,QWORD PTR [rsp+0x80]
   19eb3:	add    rax,rcx
   19eb6:	mov    QWORD PTR [rsp+0x310],rax
   19ebe:	mov    rax,QWORD PTR [rsp+0x78]
   19ec3:	add    rax,rcx
   19ec6:	mov    QWORD PTR [rsp+0x330],rax
   19ece:	mov    rax,QWORD PTR [rsp+0x70]
   19ed3:	add    rax,rcx
   19ed6:	mov    QWORD PTR [rsp+0x350],rax
   19ede:	mov    rax,QWORD PTR [rsp+0x68]
   19ee3:	add    rax,rcx
   19ee6:	mov    QWORD PTR [rsp+0x370],rax
   19eee:	mov    rax,QWORD PTR [rsp+0x60]
   19ef3:	add    rax,rcx
   19ef6:	mov    QWORD PTR [rsp+0x390],rax
   19efe:	mov    rax,QWORD PTR [rsp+0x58]
   19f03:	add    rax,rcx
   19f06:	mov    QWORD PTR [rsp+0x3b0],rax
   19f0e:	mov    rax,QWORD PTR [rsp+0x50]
   19f13:	add    rax,rcx
   19f16:	mov    QWORD PTR [rsp+0x3d0],rax
   19f1e:	mov    rax,QWORD PTR [rsp+0x48]
   19f23:	add    rax,rcx
   19f26:	mov    QWORD PTR [rsp+0x3f0],rax
   19f2e:	add    r11,rcx
   19f31:	mov    QWORD PTR [rsp+0x410],r11
   19f39:	add    r10,rcx
   19f3c:	mov    QWORD PTR [rsp+0x430],r10
   19f44:	mov    rax,QWORD PTR [rsp+0xc8]
   19f4c:	add    rax,rcx
   19f4f:	mov    QWORD PTR [rsp+0x450],rax
   19f57:	mov    rax,QWORD PTR [rsp+0xb8]
   19f5f:	add    rax,rcx
   19f62:	mov    QWORD PTR [rsp+0x470],rax
   19f6a:	mov    rax,QWORD PTR [rsp+0xa8]
   19f72:	add    rax,rcx
   19f75:	mov    QWORD PTR [rsp+0x490],rax
   19f7d:	add    r9,rcx
   19f80:	mov    QWORD PTR [rsp+0x4b0],r9
   19f88:	mov    rax,QWORD PTR [rsp+0x28]
   19f8d:	add    rax,rcx
   19f90:	mov    QWORD PTR [rsp+0x4d0],rax
   19f98:	mov    rax,QWORD PTR [rsp+0x20]
   19f9d:	add    rax,rcx
   19fa0:	mov    QWORD PTR [rsp+0x4f0],rax
   19fa8:	mov    r14d,0x10
   19fae:	mov    r8,QWORD PTR [rsp+0x510]
   19fb6:	mov    r13,QWORD PTR [rsp+r14*1+0x100]
   19fbe:	mov    rdx,QWORD PTR [rsp+r14*1+0x108]
   19fc6:	cmp    r13,rdx
   19fc9:	jne    16730 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x730>
   19fcf:	jmp    1675c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x75c>
   19fd4:	lea    rdi,[rcx+rsi*8]
   19fd8:	mov    QWORD PTR [rsp+0x8],rdi
   19fdd:	mov    ecx,0x1
   19fe2:	mov    QWORD PTR [rsp+0x518],r8
   19fea:	mov    r8d,0x1
   19ff0:	mov    rsi,r13
   19ff3:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   19ff8:	mov    r8,QWORD PTR [rsp+0x518]
   1a000:	mov    rax,QWORD PTR [rsp+0x8]
   1a005:	mov    rdx,QWORD PTR [rax]
   1a008:	mov    rax,QWORD PTR [rax+0x8]
   1a00c:	jmp    1679d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x79d>
   1a011:	mov    rdx,rbp
   1a014:	mov    rcx,QWORD PTR [rsp]
   1a018:	mov    QWORD PTR [rcx+0x40],rdx
   1a01c:	cmp    r12,rdx
   1a01f:	jbe    16185 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x185>
   1a025:	lea    rcx,[rip+0x5536c]        # 6f398 <__frame_dummy_init_array_entry+0x68>
   1a02c:	xor    edi,edi
   1a02e:	mov    rsi,r12
   1a031:	call   2a9a0 <core::slice::index::slice_index_fail>
   1a036:	call   35230 <<alloc::raw_vec::RawVec<u32>>::grow_one>
   1a03b:	mov    r9d,DWORD PTR [rsp+0x50c]
   1a043:	mov    rdi,QWORD PTR [rsp]
   1a047:	jmp    160b2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xb2>
   1a04c:	mov    rax,QWORD PTR [rsp+0xf8]
   1a054:	add    rax,r8
   1a057:	mov    QWORD PTR [rsp+0x130],rax
   1a05f:	mov    rax,QWORD PTR [rsp+0xf0]
   1a067:	add    rax,r8
   1a06a:	mov    QWORD PTR [rsp+0x150],rax
   1a072:	mov    rax,QWORD PTR [rsp+0xe8]
   1a07a:	add    rax,r8
   1a07d:	mov    QWORD PTR [rsp+0x170],rax
   1a085:	mov    rax,QWORD PTR [rsp+0xe0]
   1a08d:	add    rax,r8
   1a090:	mov    QWORD PTR [rsp+0x190],rax
   1a098:	mov    rax,QWORD PTR [rsp+0xd8]
   1a0a0:	add    rax,r8
   1a0a3:	mov    QWORD PTR [rsp+0x1b0],rax
   1a0ab:	mov    rax,QWORD PTR [rsp+0xd0]
   1a0b3:	add    rax,r8
   1a0b6:	mov    QWORD PTR [rsp+0x1d0],rax
   1a0be:	mov    rax,QWORD PTR [rsp+0xc0]
   1a0c6:	add    rax,r8
   1a0c9:	mov    QWORD PTR [rsp+0x1f0],rax
   1a0d1:	mov    rax,QWORD PTR [rsp+0xb0]
   1a0d9:	add    rax,r8
   1a0dc:	mov    QWORD PTR [rsp+0x210],rax
   1a0e4:	mov    rax,QWORD PTR [rsp+0xa0]
   1a0ec:	add    rax,r8
   1a0ef:	mov    QWORD PTR [rsp+0x230],rax
   1a0f7:	mov    rax,QWORD PTR [rsp+0x98]
   1a0ff:	add    rax,r8
   1a102:	mov    QWORD PTR [rsp+0x250],rax
   1a10a:	mov    rax,QWORD PTR [rsp+0x90]
   1a112:	add    rax,r8
   1a115:	mov    QWORD PTR [rsp+0x270],rax
   1a11d:	mov    rax,QWORD PTR [rsp+0x40]
   1a122:	add    rax,r8
   1a125:	mov    QWORD PTR [rsp+0x290],rax
   1a12d:	mov    rax,QWORD PTR [rsp+0x38]
   1a132:	add    rax,r8
   1a135:	mov    QWORD PTR [rsp+0x2b0],rax
   1a13d:	mov    rax,QWORD PTR [rsp+0x30]
   1a142:	add    rax,r8
   1a145:	mov    QWORD PTR [rsp+0x2d0],rax
   1a14d:	mov    rax,QWORD PTR [rsp+0x88]
   1a155:	add    rax,r8
   1a158:	mov    QWORD PTR [rsp+0x2f0],rax
   1a160:	mov    rax,QWORD PTR [rsp+0x80]
   1a168:	add    rax,r8
   1a16b:	mov    QWORD PTR [rsp+0x310],rax
   1a173:	mov    rax,QWORD PTR [rsp+0x78]
   1a178:	add    rax,r8
   1a17b:	mov    QWORD PTR [rsp+0x330],rax
   1a183:	mov    rax,QWORD PTR [rsp+0x70]
   1a188:	add    rax,r8
   1a18b:	mov    QWORD PTR [rsp+0x350],rax
   1a193:	mov    rax,QWORD PTR [rsp+0x68]
   1a198:	add    rax,r8
   1a19b:	mov    QWORD PTR [rsp+0x370],rax
   1a1a3:	mov    rax,QWORD PTR [rsp+0x60]
   1a1a8:	add    rax,r8
   1a1ab:	mov    QWORD PTR [rsp+0x390],rax
   1a1b3:	mov    rax,QWORD PTR [rsp+0x58]
   1a1b8:	add    rax,r8
   1a1bb:	mov    QWORD PTR [rsp+0x3b0],rax
   1a1c3:	mov    rax,QWORD PTR [rsp+0x50]
   1a1c8:	add    rax,r8
   1a1cb:	mov    QWORD PTR [rsp+0x3d0],rax
   1a1d3:	mov    rax,QWORD PTR [rsp+0x48]
   1a1d8:	add    rax,r8
   1a1db:	mov    QWORD PTR [rsp+0x3f0],rax
   1a1e3:	add    r11,r8
   1a1e6:	mov    QWORD PTR [rsp+0x410],r11
   1a1ee:	mov    rax,QWORD PTR [rsp+0x18]
   1a1f3:	add    rax,r8
   1a1f6:	mov    QWORD PTR [rsp+0x430],rax
   1a1fe:	mov    rax,QWORD PTR [rsp+0xc8]
   1a206:	add    rax,r8
   1a209:	mov    QWORD PTR [rsp+0x450],rax
   1a211:	mov    rax,QWORD PTR [rsp+0xb8]
   1a219:	add    rax,r8
   1a21c:	mov    QWORD PTR [rsp+0x470],rax
   1a224:	mov    rax,QWORD PTR [rsp+0xa8]
   1a22c:	add    rax,r8
   1a22f:	mov    QWORD PTR [rsp+0x490],rax
   1a237:	add    r9,r8
   1a23a:	mov    QWORD PTR [rsp+0x4b0],r9
   1a242:	jmp    1a439 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4439>
   1a247:	lea    rax,[r8+0x1]
   1a24b:	add    rcx,rax
   1a24e:	mov    QWORD PTR [rsp+0x110],rcx
   1a256:	mov    rax,QWORD PTR [rsp+0xf0]
   1a25e:	add    rax,r8
   1a261:	mov    QWORD PTR [rsp+0x150],rax
   1a269:	mov    rax,QWORD PTR [rsp+0xe8]
   1a271:	add    rax,r8
   1a274:	mov    QWORD PTR [rsp+0x170],rax
   1a27c:	mov    rax,QWORD PTR [rsp+0xe0]
   1a284:	add    rax,r8
   1a287:	mov    QWORD PTR [rsp+0x190],rax
   1a28f:	mov    rax,QWORD PTR [rsp+0xd8]
   1a297:	add    rax,r8
   1a29a:	mov    QWORD PTR [rsp+0x1b0],rax
   1a2a2:	mov    rax,QWORD PTR [rsp+0xd0]
   1a2aa:	add    rax,r8
   1a2ad:	mov    QWORD PTR [rsp+0x1d0],rax
   1a2b5:	mov    rax,QWORD PTR [rsp+0xc0]
   1a2bd:	add    rax,r8
   1a2c0:	mov    QWORD PTR [rsp+0x1f0],rax
   1a2c8:	mov    rax,QWORD PTR [rsp+0xb0]
   1a2d0:	add    rax,r8
   1a2d3:	mov    QWORD PTR [rsp+0x210],rax
   1a2db:	mov    rax,QWORD PTR [rsp+0xa0]
   1a2e3:	add    rax,r8
   1a2e6:	mov    QWORD PTR [rsp+0x230],rax
   1a2ee:	mov    rax,QWORD PTR [rsp+0x98]
   1a2f6:	add    rax,r8
   1a2f9:	mov    QWORD PTR [rsp+0x250],rax
   1a301:	mov    rax,QWORD PTR [rsp+0x90]
   1a309:	add    rax,r8
   1a30c:	mov    QWORD PTR [rsp+0x270],rax
   1a314:	mov    rax,QWORD PTR [rsp+0x40]
   1a319:	add    rax,r8
   1a31c:	mov    QWORD PTR [rsp+0x290],rax
   1a324:	mov    rax,QWORD PTR [rsp+0x38]
   1a329:	add    rax,r8
   1a32c:	mov    QWORD PTR [rsp+0x2b0],rax
   1a334:	mov    rax,QWORD PTR [rsp+0x30]
   1a339:	add    rax,r8
   1a33c:	mov    QWORD PTR [rsp+0x2d0],rax
   1a344:	mov    rax,QWORD PTR [rsp+0x88]
   1a34c:	add    rax,r8
   1a34f:	mov    QWORD PTR [rsp+0x2f0],rax
   1a357:	mov    rax,QWORD PTR [rsp+0x80]
   1a35f:	add    rax,r8
   1a362:	mov    QWORD PTR [rsp+0x310],rax
   1a36a:	mov    rax,QWORD PTR [rsp+0x78]
   1a36f:	add    rax,r8
   1a372:	mov    QWORD PTR [rsp+0x330],rax
   1a37a:	mov    rax,QWORD PTR [rsp+0x70]
   1a37f:	add    rax,r8
   1a382:	mov    QWORD PTR [rsp+0x350],rax
   1a38a:	mov    rax,QWORD PTR [rsp+0x68]
   1a38f:	add    rax,r8
   1a392:	mov    QWORD PTR [rsp+0x370],rax
   1a39a:	mov    rax,QWORD PTR [rsp+0x60]
   1a39f:	add    rax,r8
   1a3a2:	mov    QWORD PTR [rsp+0x390],rax
   1a3aa:	mov    rax,QWORD PTR [rsp+0x58]
   1a3af:	add    rax,r8
   1a3b2:	mov    QWORD PTR [rsp+0x3b0],rax
   1a3ba:	mov    rax,QWORD PTR [rsp+0x50]
   1a3bf:	add    rax,r8
   1a3c2:	mov    QWORD PTR [rsp+0x3d0],rax
   1a3ca:	mov    rax,QWORD PTR [rsp+0x48]
   1a3cf:	add    rax,r8
   1a3d2:	mov    QWORD PTR [rsp+0x3f0],rax
   1a3da:	add    r11,r8
   1a3dd:	mov    QWORD PTR [rsp+0x410],r11
   1a3e5:	mov    rax,QWORD PTR [rsp+0x18]
   1a3ea:	add    rax,r8
   1a3ed:	mov    QWORD PTR [rsp+0x430],rax
   1a3f5:	mov    rax,QWORD PTR [rsp+0xc8]
   1a3fd:	add    rax,r8
   1a400:	mov    QWORD PTR [rsp+0x450],rax
   1a408:	mov    rax,QWORD PTR [rsp+0xb8]
   1a410:	add    rax,r8
   1a413:	mov    QWORD PTR [rsp+0x470],rax
   1a41b:	mov    rax,QWORD PTR [rsp+0xa8]
   1a423:	add    rax,r8
   1a426:	mov    QWORD PTR [rsp+0x490],rax
   1a42e:	add    r10,r8
   1a431:	mov    QWORD PTR [rsp+0x4b0],r10
   1a439:	mov    rax,QWORD PTR [rsp+0x28]
   1a43e:	add    rax,r8
   1a441:	mov    QWORD PTR [rsp+0x4d0],rax
   1a449:	mov    rax,QWORD PTR [rsp+0x20]
   1a44e:	add    rax,r8
   1a451:	lea    rcx,[rsp+0x4f0]
   1a459:	mov    r8,QWORD PTR [rsp+0x100]
   1a461:	jmp    1cd16 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6d16>
   1a466:	lea    rax,[r8+0x1]
   1a46a:	add    rcx,rax
   1a46d:	mov    QWORD PTR [rsp+0x110],rcx
   1a475:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a47d:	add    rcx,rax
   1a480:	mov    QWORD PTR [rsp+0x130],rcx
   1a488:	mov    rax,QWORD PTR [rsp+0xe8]
   1a490:	add    rax,r8
   1a493:	mov    QWORD PTR [rsp+0x170],rax
   1a49b:	mov    rax,QWORD PTR [rsp+0xe0]
   1a4a3:	add    rax,r8
   1a4a6:	mov    QWORD PTR [rsp+0x190],rax
   1a4ae:	mov    rax,QWORD PTR [rsp+0xd8]
   1a4b6:	add    rax,r8
   1a4b9:	mov    QWORD PTR [rsp+0x1b0],rax
   1a4c1:	mov    rax,QWORD PTR [rsp+0xd0]
   1a4c9:	add    rax,r8
   1a4cc:	mov    QWORD PTR [rsp+0x1d0],rax
   1a4d4:	mov    rax,QWORD PTR [rsp+0xc0]
   1a4dc:	add    rax,r8
   1a4df:	mov    QWORD PTR [rsp+0x1f0],rax
   1a4e7:	mov    rax,QWORD PTR [rsp+0xb0]
   1a4ef:	add    rax,r8
   1a4f2:	mov    QWORD PTR [rsp+0x210],rax
   1a4fa:	mov    rax,QWORD PTR [rsp+0xa0]
   1a502:	add    rax,r8
   1a505:	mov    QWORD PTR [rsp+0x230],rax
   1a50d:	mov    rax,QWORD PTR [rsp+0x98]
   1a515:	add    rax,r8
   1a518:	mov    QWORD PTR [rsp+0x250],rax
   1a520:	mov    rax,QWORD PTR [rsp+0x90]
   1a528:	add    rax,r8
   1a52b:	mov    QWORD PTR [rsp+0x270],rax
   1a533:	mov    rax,QWORD PTR [rsp+0x40]
   1a538:	add    rax,r8
   1a53b:	mov    QWORD PTR [rsp+0x290],rax
   1a543:	mov    rax,QWORD PTR [rsp+0x38]
   1a548:	add    rax,r8
   1a54b:	mov    QWORD PTR [rsp+0x2b0],rax
   1a553:	mov    rax,QWORD PTR [rsp+0x30]
   1a558:	add    rax,r8
   1a55b:	mov    QWORD PTR [rsp+0x2d0],rax
   1a563:	mov    rax,QWORD PTR [rsp+0x88]
   1a56b:	add    rax,r8
   1a56e:	mov    QWORD PTR [rsp+0x2f0],rax
   1a576:	mov    rax,QWORD PTR [rsp+0x80]
   1a57e:	add    rax,r8
   1a581:	mov    QWORD PTR [rsp+0x310],rax
   1a589:	mov    rax,QWORD PTR [rsp+0x78]
   1a58e:	add    rax,r8
   1a591:	mov    QWORD PTR [rsp+0x330],rax
   1a599:	mov    rax,QWORD PTR [rsp+0x70]
   1a59e:	add    rax,r8
   1a5a1:	mov    QWORD PTR [rsp+0x350],rax
   1a5a9:	mov    rax,QWORD PTR [rsp+0x68]
   1a5ae:	add    rax,r8
   1a5b1:	mov    QWORD PTR [rsp+0x370],rax
   1a5b9:	mov    rax,QWORD PTR [rsp+0x60]
   1a5be:	add    rax,r8
   1a5c1:	mov    QWORD PTR [rsp+0x390],rax
   1a5c9:	mov    rax,QWORD PTR [rsp+0x58]
   1a5ce:	add    rax,r8
   1a5d1:	mov    QWORD PTR [rsp+0x3b0],rax
   1a5d9:	mov    rax,QWORD PTR [rsp+0x50]
   1a5de:	add    rax,r8
   1a5e1:	mov    QWORD PTR [rsp+0x3d0],rax
   1a5e9:	mov    rax,QWORD PTR [rsp+0x48]
   1a5ee:	add    rax,r8
   1a5f1:	mov    QWORD PTR [rsp+0x3f0],rax
   1a5f9:	add    r11,r8
   1a5fc:	mov    QWORD PTR [rsp+0x410],r11
   1a604:	mov    rax,QWORD PTR [rsp+0x18]
   1a609:	add    rax,r8
   1a60c:	mov    QWORD PTR [rsp+0x430],rax
   1a614:	mov    rax,QWORD PTR [rsp+0xc8]
   1a61c:	add    rax,r8
   1a61f:	mov    QWORD PTR [rsp+0x450],rax
   1a627:	mov    rax,QWORD PTR [rsp+0xb8]
   1a62f:	add    rax,r8
   1a632:	mov    QWORD PTR [rsp+0x470],rax
   1a63a:	mov    rax,QWORD PTR [rsp+0xa8]
   1a642:	add    rax,r8
   1a645:	mov    QWORD PTR [rsp+0x490],rax
   1a64d:	add    r9,r8
   1a650:	mov    QWORD PTR [rsp+0x4b0],r9
   1a658:	mov    rax,QWORD PTR [rsp+0x28]
   1a65d:	add    rax,r8
   1a660:	mov    QWORD PTR [rsp+0x4d0],rax
   1a668:	mov    rax,QWORD PTR [rsp+0x20]
   1a66d:	add    rax,r8
   1a670:	mov    QWORD PTR [rsp+0x4f0],rax
   1a678:	mov    rdx,QWORD PTR [rsp]
   1a67c:	jmp    1ac19 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c19>
   1a681:	lea    rax,[r8+0x1]
   1a685:	add    rcx,rax
   1a688:	mov    QWORD PTR [rsp+0x110],rcx
   1a690:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a698:	add    rcx,rax
   1a69b:	mov    QWORD PTR [rsp+0x130],rcx
   1a6a3:	mov    rcx,QWORD PTR [rsp+0xf0]
   1a6ab:	add    rcx,rax
   1a6ae:	mov    QWORD PTR [rsp+0x150],rcx
   1a6b6:	mov    rax,QWORD PTR [rsp+0xe0]
   1a6be:	add    rax,r8
   1a6c1:	mov    QWORD PTR [rsp+0x190],rax
   1a6c9:	jmp    1a718 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4718>
   1a6cb:	lea    rax,[r8+0x1]
   1a6cf:	mov    rcx,QWORD PTR [rsp+0x10]
   1a6d4:	add    rcx,rax
   1a6d7:	mov    QWORD PTR [rsp+0x110],rcx
   1a6df:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a6e7:	add    rcx,rax
   1a6ea:	mov    QWORD PTR [rsp+0x130],rcx
   1a6f2:	mov    rcx,QWORD PTR [rsp+0xf0]
   1a6fa:	add    rcx,rax
   1a6fd:	mov    QWORD PTR [rsp+0x150],rcx
   1a705:	mov    rcx,QWORD PTR [rsp+0xe8]
   1a70d:	add    rcx,rax
   1a710:	mov    QWORD PTR [rsp+0x170],rcx
   1a718:	mov    rax,QWORD PTR [rsp+0xd8]
   1a720:	add    rax,r8
   1a723:	mov    QWORD PTR [rsp+0x1b0],rax
   1a72b:	mov    rax,QWORD PTR [rsp+0xd0]
   1a733:	add    rax,r8
   1a736:	mov    QWORD PTR [rsp+0x1d0],rax
   1a73e:	mov    rax,QWORD PTR [rsp+0xc0]
   1a746:	add    rax,r8
   1a749:	mov    QWORD PTR [rsp+0x1f0],rax
   1a751:	mov    rax,QWORD PTR [rsp+0xb0]
   1a759:	add    rax,r8
   1a75c:	mov    QWORD PTR [rsp+0x210],rax
   1a764:	mov    rax,QWORD PTR [rsp+0xa0]
   1a76c:	add    rax,r8
   1a76f:	mov    QWORD PTR [rsp+0x230],rax
   1a777:	mov    rax,QWORD PTR [rsp+0x98]
   1a77f:	add    rax,r8
   1a782:	mov    QWORD PTR [rsp+0x250],rax
   1a78a:	mov    rax,QWORD PTR [rsp+0x90]
   1a792:	add    rax,r8
   1a795:	mov    QWORD PTR [rsp+0x270],rax
   1a79d:	mov    rax,QWORD PTR [rsp+0x40]
   1a7a2:	add    rax,r8
   1a7a5:	mov    QWORD PTR [rsp+0x290],rax
   1a7ad:	mov    rax,QWORD PTR [rsp+0x38]
   1a7b2:	add    rax,r8
   1a7b5:	mov    QWORD PTR [rsp+0x2b0],rax
   1a7bd:	mov    rax,QWORD PTR [rsp+0x30]
   1a7c2:	add    rax,r8
   1a7c5:	mov    QWORD PTR [rsp+0x2d0],rax
   1a7cd:	mov    rax,QWORD PTR [rsp+0x88]
   1a7d5:	add    rax,r8
   1a7d8:	mov    QWORD PTR [rsp+0x2f0],rax
   1a7e0:	mov    rax,QWORD PTR [rsp+0x80]
   1a7e8:	add    rax,r8
   1a7eb:	mov    QWORD PTR [rsp+0x310],rax
   1a7f3:	mov    rax,QWORD PTR [rsp+0x78]
   1a7f8:	add    rax,r8
   1a7fb:	mov    QWORD PTR [rsp+0x330],rax
   1a803:	mov    rax,QWORD PTR [rsp+0x70]
   1a808:	add    rax,r8
   1a80b:	mov    QWORD PTR [rsp+0x350],rax
   1a813:	mov    rax,QWORD PTR [rsp+0x68]
   1a818:	add    rax,r8
   1a81b:	mov    QWORD PTR [rsp+0x370],rax
   1a823:	mov    rax,QWORD PTR [rsp+0x60]
   1a828:	add    rax,r8
   1a82b:	mov    QWORD PTR [rsp+0x390],rax
   1a833:	mov    rax,QWORD PTR [rsp+0x58]
   1a838:	add    rax,r8
   1a83b:	mov    QWORD PTR [rsp+0x3b0],rax
   1a843:	mov    rax,QWORD PTR [rsp+0x50]
   1a848:	add    rax,r8
   1a84b:	mov    QWORD PTR [rsp+0x3d0],rax
   1a853:	mov    rax,QWORD PTR [rsp+0x48]
   1a858:	add    rax,r8
   1a85b:	mov    QWORD PTR [rsp+0x3f0],rax
   1a863:	add    r11,r8
   1a866:	mov    QWORD PTR [rsp+0x410],r11
   1a86e:	jmp    1aba5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4ba5>
   1a873:	lea    rax,[r8+0x1]
   1a877:	add    rcx,rax
   1a87a:	mov    QWORD PTR [rsp+0x110],rcx
   1a882:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a88a:	add    rcx,rax
   1a88d:	mov    QWORD PTR [rsp+0x130],rcx
   1a895:	mov    rcx,QWORD PTR [rsp+0xf0]
   1a89d:	add    rcx,rax
   1a8a0:	mov    QWORD PTR [rsp+0x150],rcx
   1a8a8:	mov    rcx,QWORD PTR [rsp+0xe8]
   1a8b0:	add    rcx,rax
   1a8b3:	mov    QWORD PTR [rsp+0x170],rcx
   1a8bb:	mov    rcx,QWORD PTR [rsp+0xe0]
   1a8c3:	add    rcx,rax
   1a8c6:	mov    QWORD PTR [rsp+0x190],rcx
   1a8ce:	mov    rax,QWORD PTR [rsp+0xd0]
   1a8d6:	add    rax,r8
   1a8d9:	mov    QWORD PTR [rsp+0x1d0],rax
   1a8e1:	jmp    1a951 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4951>
   1a8e3:	lea    rax,[r8+0x1]
   1a8e7:	add    rcx,rax
   1a8ea:	mov    QWORD PTR [rsp+0x110],rcx
   1a8f2:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a8fa:	add    rcx,rax
   1a8fd:	mov    QWORD PTR [rsp+0x130],rcx
   1a905:	mov    rcx,QWORD PTR [rsp+0xf0]
   1a90d:	add    rcx,rax
   1a910:	mov    QWORD PTR [rsp+0x150],rcx
   1a918:	mov    rcx,QWORD PTR [rsp+0xe8]
   1a920:	add    rcx,rax
   1a923:	mov    QWORD PTR [rsp+0x170],rcx
   1a92b:	mov    rcx,QWORD PTR [rsp+0xe0]
   1a933:	add    rcx,rax
   1a936:	mov    QWORD PTR [rsp+0x190],rcx
   1a93e:	mov    rcx,QWORD PTR [rsp+0xd8]
   1a946:	add    rcx,rax
   1a949:	mov    QWORD PTR [rsp+0x1b0],rcx
   1a951:	mov    rax,QWORD PTR [rsp+0xc0]
   1a959:	add    rax,r8
   1a95c:	mov    QWORD PTR [rsp+0x1f0],rax
   1a964:	jmp    1a9ea <mibz2::bwt::room_segments_unrolled::<32, 1>+0x49ea>
   1a969:	lea    rax,[r8+0x1]
   1a96d:	add    rcx,rax
   1a970:	mov    QWORD PTR [rsp+0x110],rcx
   1a978:	mov    rcx,QWORD PTR [rsp+0xf8]
   1a980:	add    rcx,rax
   1a983:	mov    QWORD PTR [rsp+0x130],rcx
   1a98b:	mov    rcx,QWORD PTR [rsp+0xf0]
   1a993:	add    rcx,rax
   1a996:	mov    QWORD PTR [rsp+0x150],rcx
   1a99e:	mov    rcx,QWORD PTR [rsp+0xe8]
   1a9a6:	add    rcx,rax
   1a9a9:	mov    QWORD PTR [rsp+0x170],rcx
   1a9b1:	mov    rcx,QWORD PTR [rsp+0xe0]
   1a9b9:	add    rcx,rax
   1a9bc:	mov    QWORD PTR [rsp+0x190],rcx
   1a9c4:	mov    rcx,QWORD PTR [rsp+0xd8]
   1a9cc:	add    rcx,rax
   1a9cf:	mov    QWORD PTR [rsp+0x1b0],rcx
   1a9d7:	mov    rcx,QWORD PTR [rsp+0xd0]
   1a9df:	add    rcx,rax
   1a9e2:	mov    QWORD PTR [rsp+0x1d0],rcx
   1a9ea:	mov    rax,QWORD PTR [rsp+0xb0]
   1a9f2:	add    rax,r8
   1a9f5:	mov    QWORD PTR [rsp+0x210],rax
   1a9fd:	jmp    1aa96 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4a96>
   1aa02:	lea    rax,[r8+0x1]
   1aa06:	add    rcx,rax
   1aa09:	mov    QWORD PTR [rsp+0x110],rcx
   1aa11:	mov    rcx,QWORD PTR [rsp+0xf8]
   1aa19:	add    rcx,rax
   1aa1c:	mov    QWORD PTR [rsp+0x130],rcx
   1aa24:	mov    rcx,QWORD PTR [rsp+0xf0]
   1aa2c:	add    rcx,rax
   1aa2f:	mov    QWORD PTR [rsp+0x150],rcx
   1aa37:	mov    rcx,QWORD PTR [rsp+0xe8]
   1aa3f:	add    rcx,rax
   1aa42:	mov    QWORD PTR [rsp+0x170],rcx
   1aa4a:	mov    rcx,QWORD PTR [rsp+0xe0]
   1aa52:	add    rcx,rax
   1aa55:	mov    QWORD PTR [rsp+0x190],rcx
   1aa5d:	mov    rcx,QWORD PTR [rsp+0xd8]
   1aa65:	add    rcx,rax
   1aa68:	mov    QWORD PTR [rsp+0x1b0],rcx
   1aa70:	mov    rcx,QWORD PTR [rsp+0xd0]
   1aa78:	add    rcx,rax
   1aa7b:	mov    QWORD PTR [rsp+0x1d0],rcx
   1aa83:	mov    rcx,QWORD PTR [rsp+0xc0]
   1aa8b:	add    rcx,rax
   1aa8e:	mov    QWORD PTR [rsp+0x1f0],rcx
   1aa96:	mov    rax,QWORD PTR [rsp+0xa0]
   1aa9e:	add    rax,r8
   1aaa1:	mov    QWORD PTR [rsp+0x230],rax
   1aaa9:	mov    rax,QWORD PTR [rsp+0x98]
   1aab1:	add    rax,r8
   1aab4:	mov    QWORD PTR [rsp+0x250],rax
   1aabc:	mov    rax,QWORD PTR [rsp+0x90]
   1aac4:	add    rax,r8
   1aac7:	mov    QWORD PTR [rsp+0x270],rax
   1aacf:	mov    rax,QWORD PTR [rsp+0x40]
   1aad4:	add    rax,r8
   1aad7:	mov    QWORD PTR [rsp+0x290],rax
   1aadf:	mov    rax,QWORD PTR [rsp+0x38]
   1aae4:	add    rax,r8
   1aae7:	mov    QWORD PTR [rsp+0x2b0],rax
   1aaef:	mov    rax,QWORD PTR [rsp+0x30]
   1aaf4:	add    rax,r8
   1aaf7:	mov    QWORD PTR [rsp+0x2d0],rax
   1aaff:	mov    rax,QWORD PTR [rsp+0x88]
   1ab07:	add    rax,r8
   1ab0a:	mov    QWORD PTR [rsp+0x2f0],rax
   1ab12:	mov    rax,QWORD PTR [rsp+0x80]
   1ab1a:	add    rax,r8
   1ab1d:	mov    QWORD PTR [rsp+0x310],rax
   1ab25:	mov    rax,QWORD PTR [rsp+0x78]
   1ab2a:	add    rax,r8
   1ab2d:	mov    QWORD PTR [rsp+0x330],rax
   1ab35:	mov    rax,QWORD PTR [rsp+0x70]
   1ab3a:	add    rax,r8
   1ab3d:	mov    QWORD PTR [rsp+0x350],rax
   1ab45:	mov    rax,QWORD PTR [rsp+0x68]
   1ab4a:	add    rax,r8
   1ab4d:	mov    QWORD PTR [rsp+0x370],rax
   1ab55:	mov    rax,QWORD PTR [rsp+0x60]
   1ab5a:	add    rax,r8
   1ab5d:	mov    QWORD PTR [rsp+0x390],rax
   1ab65:	mov    rax,QWORD PTR [rsp+0x58]
   1ab6a:	add    rax,r8
   1ab6d:	mov    QWORD PTR [rsp+0x3b0],rax
   1ab75:	mov    rax,QWORD PTR [rsp+0x50]
   1ab7a:	add    rax,r8
   1ab7d:	mov    QWORD PTR [rsp+0x3d0],rax
   1ab85:	mov    rax,QWORD PTR [rsp+0x48]
   1ab8a:	add    rax,r8
   1ab8d:	mov    QWORD PTR [rsp+0x3f0],rax
   1ab95:	mov    rax,QWORD PTR [rsp+0x8]
   1ab9a:	add    rax,r8
   1ab9d:	mov    QWORD PTR [rsp+0x410],rax
   1aba5:	mov    rax,QWORD PTR [rsp+0x18]
   1abaa:	add    rax,r8
   1abad:	mov    QWORD PTR [rsp+0x430],rax
   1abb5:	mov    rax,QWORD PTR [rsp+0xc8]
   1abbd:	add    rax,r8
   1abc0:	mov    QWORD PTR [rsp+0x450],rax
   1abc8:	mov    rax,QWORD PTR [rsp+0xb8]
   1abd0:	add    rax,r8
   1abd3:	mov    QWORD PTR [rsp+0x470],rax
   1abdb:	mov    rax,QWORD PTR [rsp+0xa8]
   1abe3:	add    rax,r8
   1abe6:	mov    QWORD PTR [rsp+0x490],rax
   1abee:	add    r9,r8
   1abf1:	mov    QWORD PTR [rsp+0x4b0],r9
   1abf9:	mov    rax,QWORD PTR [rsp+0x28]
   1abfe:	add    rax,r8
   1ac01:	mov    QWORD PTR [rsp+0x4d0],rax
   1ac09:	mov    rax,QWORD PTR [rsp+0x20]
   1ac0e:	add    rax,r8
   1ac11:	mov    QWORD PTR [rsp+0x4f0],rax
   1ac19:	mov    r8,QWORD PTR [rsp+0x100]
   1ac21:	shl    r8,0x5
   1ac25:	lea    r14,[rsp+r8*1]
   1ac29:	add    r14,0x108
   1ac30:	mov    rax,QWORD PTR [rdx+0x50]
   1ac34:	mov    rsi,QWORD PTR [rdx+0x58]
   1ac38:	lea    r13,[rsp+0x108]
   1ac40:	jmp    1ac5d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c5d>
   1ac42:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   1ac50:	mov    r13,rbx
   1ac53:	add    rbp,0xffffffffffffffe0
   1ac57:	je     1ad9d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4d9d>
   1ac5d:	lea    rbx,[r13+0x20]
   1ac61:	mov    r12,QWORD PTR [r13+0x0]
   1ac65:	cmp    r12,0xffffffffffffffff
   1ac69:	je     1ac50 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c50>
   1ac6b:	cmp    r12,rsi
   1ac6e:	jae    1d28d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x728d>
   1ac74:	cmp    QWORD PTR [rax+r12*8],0xffffffffffffffff
   1ac79:	jne    1ac50 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c50>
   1ac7b:	mov    edi,0x80
   1ac80:	call   QWORD PTR [rip+0x56a92]        # 71718 <malloc@GLIBC_2.2.5>
   1ac86:	test   rax,rax
   1ac89:	je     1d2d4 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72d4>
   1ac8f:	mov    rcx,QWORD PTR [r13+0x18]
   1ac93:	mov    QWORD PTR [rax+0x18],rcx
   1ac97:	movups xmm0,XMMWORD PTR [r13+0x8]
   1ac9c:	movups XMMWORD PTR [rax+0x8],xmm0
   1aca0:	mov    QWORD PTR [rax],r12
   1aca3:	mov    QWORD PTR [rsp+0x528],0x4
   1acaf:	mov    QWORD PTR [rsp+0x530],rax
   1acb7:	mov    QWORD PTR [rsp+0x538],0x1
   1acc3:	cmp    rbx,r14
   1acc6:	mov    rdx,QWORD PTR [rsp]
   1acca:	mov    r13d,0x1
   1acd0:	je     1adaf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4daf>
   1acd6:	lea    r12,[rsp+0x528]
   1acde:	mov    rcx,QWORD PTR [rdx+0x50]
   1ace2:	mov    rsi,QWORD PTR [rdx+0x58]
   1ace6:	mov    rdx,rbx
   1ace9:	jmp    1acfc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4cfc>
   1aceb:	nop    DWORD PTR [rax+rax*1+0x0]
   1acf0:	mov    rdx,rbx
   1acf3:	cmp    rbx,r14
   1acf6:	je     1adab <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4dab>
   1acfc:	lea    rbx,[rdx+0x20]
   1ad00:	mov    rbp,QWORD PTR [rdx]
   1ad03:	cmp    rbp,0xffffffffffffffff
   1ad07:	je     1acf0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4cf0>
   1ad09:	cmp    rbp,rsi
   1ad0c:	jae    1d2a8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72a8>
   1ad12:	cmp    QWORD PTR [rcx+rbp*8],0xffffffffffffffff
   1ad17:	jne    1acf0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4cf0>
   1ad19:	mov    rcx,QWORD PTR [rdx+0x18]
   1ad1d:	mov    QWORD PTR [rsp+0x660],rcx
   1ad25:	movups xmm0,XMMWORD PTR [rdx+0x8]
   1ad29:	movaps XMMWORD PTR [rsp+0x650],xmm0
   1ad31:	cmp    r13,QWORD PTR [rsp+0x528]
   1ad39:	jne    1ad5e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4d5e>
   1ad3b:	mov    edx,0x1
   1ad40:	mov    ecx,0x8
   1ad45:	mov    r8d,0x20
   1ad4b:	mov    rdi,r12
   1ad4e:	mov    rsi,r13
   1ad51:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1ad56:	mov    rax,QWORD PTR [rsp+0x530]
   1ad5e:	mov    rcx,r13
   1ad61:	shl    rcx,0x5
   1ad65:	mov    QWORD PTR [rax+rcx*1],rbp
   1ad69:	movaps xmm0,XMMWORD PTR [rsp+0x650]
   1ad71:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   1ad76:	mov    rdx,QWORD PTR [rsp+0x660]
   1ad7e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   1ad83:	inc    r13
   1ad86:	mov    QWORD PTR [rsp+0x538],r13
   1ad8e:	cmp    rbx,r14
   1ad91:	mov    rdx,QWORD PTR [rsp]
   1ad95:	jne    1acde <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4cde>
   1ad9b:	jmp    1adaf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4daf>
   1ad9d:	mov    r12d,0x8
   1ada3:	xor    r14d,r14d
   1ada6:	jmp    1cf77 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6f77>
   1adab:	mov    rdx,QWORD PTR [rsp]
   1adaf:	mov    rax,QWORD PTR [rsp+0x528]
   1adb7:	mov    QWORD PTR [rsp+0x8],rax
   1adbc:	mov    r12,QWORD PTR [rsp+0x530]
   1adc4:	mov    rbx,r13
   1adc7:	shl    rbx,0x5
   1adcb:	xor    r14d,r14d
   1adce:	lea    rax,[rip+0x5460b]        # 6f3e0 <__frame_dummy_init_array_entry+0xb0>
   1add5:	mov    QWORD PTR [rsp+0x10],rax
   1adda:	jmp    1cd4d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6d4d>
   1addf:	lea    rax,[r8+0x1]
   1ade3:	add    rcx,rax
   1ade6:	mov    QWORD PTR [rsp+0x110],rcx
   1adee:	mov    rcx,QWORD PTR [rsp+0xf8]
   1adf6:	add    rcx,rax
   1adf9:	mov    QWORD PTR [rsp+0x130],rcx
   1ae01:	mov    rcx,QWORD PTR [rsp+0xf0]
   1ae09:	add    rcx,rax
   1ae0c:	mov    QWORD PTR [rsp+0x150],rcx
   1ae14:	mov    rcx,QWORD PTR [rsp+0xe8]
   1ae1c:	add    rcx,rax
   1ae1f:	mov    QWORD PTR [rsp+0x170],rcx
   1ae27:	mov    rcx,QWORD PTR [rsp+0xe0]
   1ae2f:	add    rcx,rax
   1ae32:	mov    QWORD PTR [rsp+0x190],rcx
   1ae3a:	mov    rcx,QWORD PTR [rsp+0xd8]
   1ae42:	add    rcx,rax
   1ae45:	mov    QWORD PTR [rsp+0x1b0],rcx
   1ae4d:	mov    rcx,QWORD PTR [rsp+0xd0]
   1ae55:	add    rcx,rax
   1ae58:	mov    QWORD PTR [rsp+0x1d0],rcx
   1ae60:	mov    rcx,QWORD PTR [rsp+0xc0]
   1ae68:	add    rcx,rax
   1ae6b:	mov    QWORD PTR [rsp+0x1f0],rcx
   1ae73:	mov    rcx,QWORD PTR [rsp+0xb0]
   1ae7b:	add    rcx,rax
   1ae7e:	mov    QWORD PTR [rsp+0x210],rcx
   1ae86:	jmp    1aaa9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4aa9>
   1ae8b:	lea    rax,[r8+0x1]
   1ae8f:	add    rcx,rax
   1ae92:	mov    QWORD PTR [rsp+0x110],rcx
   1ae9a:	mov    rcx,QWORD PTR [rsp+0xf8]
   1aea2:	add    rcx,rax
   1aea5:	mov    QWORD PTR [rsp+0x130],rcx
   1aead:	mov    rcx,QWORD PTR [rsp+0xf0]
   1aeb5:	add    rcx,rax
   1aeb8:	mov    QWORD PTR [rsp+0x150],rcx
   1aec0:	mov    rcx,QWORD PTR [rsp+0xe8]
   1aec8:	add    rcx,rax
   1aecb:	mov    QWORD PTR [rsp+0x170],rcx
   1aed3:	mov    rcx,QWORD PTR [rsp+0xe0]
   1aedb:	add    rcx,rax
   1aede:	mov    QWORD PTR [rsp+0x190],rcx
   1aee6:	mov    rcx,QWORD PTR [rsp+0xd8]
   1aeee:	add    rcx,rax
   1aef1:	mov    QWORD PTR [rsp+0x1b0],rcx
   1aef9:	mov    rcx,QWORD PTR [rsp+0xd0]
   1af01:	add    rcx,rax
   1af04:	mov    QWORD PTR [rsp+0x1d0],rcx
   1af0c:	mov    rcx,QWORD PTR [rsp+0xc0]
   1af14:	add    rcx,rax
   1af17:	mov    QWORD PTR [rsp+0x1f0],rcx
   1af1f:	mov    rcx,QWORD PTR [rsp+0xb0]
   1af27:	add    rcx,rax
   1af2a:	mov    QWORD PTR [rsp+0x210],rcx
   1af32:	mov    rcx,QWORD PTR [rsp+0xa0]
   1af3a:	add    rcx,rax
   1af3d:	mov    QWORD PTR [rsp+0x230],rcx
   1af45:	jmp    1aabc <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4abc>
   1af4a:	lea    rax,[r8+0x1]
   1af4e:	add    rcx,rax
   1af51:	mov    QWORD PTR [rsp+0x110],rcx
   1af59:	mov    rcx,QWORD PTR [rsp+0xf8]
   1af61:	add    rcx,rax
   1af64:	mov    QWORD PTR [rsp+0x130],rcx
   1af6c:	mov    rcx,QWORD PTR [rsp+0xf0]
   1af74:	add    rcx,rax
   1af77:	mov    QWORD PTR [rsp+0x150],rcx
   1af7f:	mov    rcx,QWORD PTR [rsp+0xe8]
   1af87:	add    rcx,rax
   1af8a:	mov    QWORD PTR [rsp+0x170],rcx
   1af92:	mov    rcx,QWORD PTR [rsp+0xe0]
   1af9a:	add    rcx,rax
   1af9d:	mov    QWORD PTR [rsp+0x190],rcx
   1afa5:	mov    rcx,QWORD PTR [rsp+0xd8]
   1afad:	add    rcx,rax
   1afb0:	mov    QWORD PTR [rsp+0x1b0],rcx
   1afb8:	mov    rcx,QWORD PTR [rsp+0xd0]
   1afc0:	add    rcx,rax
   1afc3:	mov    QWORD PTR [rsp+0x1d0],rcx
   1afcb:	mov    rcx,QWORD PTR [rsp+0xc0]
   1afd3:	add    rcx,rax
   1afd6:	mov    QWORD PTR [rsp+0x1f0],rcx
   1afde:	mov    rcx,QWORD PTR [rsp+0xb0]
   1afe6:	add    rcx,rax
   1afe9:	mov    QWORD PTR [rsp+0x210],rcx
   1aff1:	mov    rcx,QWORD PTR [rsp+0xa0]
   1aff9:	add    rcx,rax
   1affc:	mov    QWORD PTR [rsp+0x230],rcx
   1b004:	mov    rcx,QWORD PTR [rsp+0x98]
   1b00c:	add    rcx,rax
   1b00f:	mov    QWORD PTR [rsp+0x250],rcx
   1b017:	jmp    1aacf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4acf>
   1b01c:	lea    rax,[r8+0x1]
   1b020:	add    rcx,rax
   1b023:	mov    QWORD PTR [rsp+0x110],rcx
   1b02b:	mov    rcx,QWORD PTR [rsp+0xf8]
   1b033:	add    rcx,rax
   1b036:	mov    QWORD PTR [rsp+0x130],rcx
   1b03e:	mov    rcx,QWORD PTR [rsp+0xf0]
   1b046:	add    rcx,rax
   1b049:	mov    QWORD PTR [rsp+0x150],rcx
   1b051:	mov    rcx,QWORD PTR [rsp+0xe8]
   1b059:	add    rcx,rax
   1b05c:	mov    QWORD PTR [rsp+0x170],rcx
   1b064:	mov    rcx,QWORD PTR [rsp+0xe0]
   1b06c:	add    rcx,rax
   1b06f:	mov    QWORD PTR [rsp+0x190],rcx
   1b077:	mov    rcx,QWORD PTR [rsp+0xd8]
   1b07f:	add    rcx,rax
   1b082:	mov    QWORD PTR [rsp+0x1b0],rcx
   1b08a:	mov    rcx,QWORD PTR [rsp+0xd0]
   1b092:	add    rcx,rax
   1b095:	mov    QWORD PTR [rsp+0x1d0],rcx
   1b09d:	mov    rcx,QWORD PTR [rsp+0xc0]
   1b0a5:	add    rcx,rax
   1b0a8:	mov    QWORD PTR [rsp+0x1f0],rcx
   1b0b0:	mov    rcx,QWORD PTR [rsp+0xb0]
   1b0b8:	add    rcx,rax
   1b0bb:	mov    QWORD PTR [rsp+0x210],rcx
   1b0c3:	mov    rcx,QWORD PTR [rsp+0xa0]
   1b0cb:	add    rcx,rax
   1b0ce:	mov    QWORD PTR [rsp+0x230],rcx
   1b0d6:	mov    rcx,QWORD PTR [rsp+0x98]
   1b0de:	add    rcx,rax
   1b0e1:	mov    QWORD PTR [rsp+0x250],rcx
   1b0e9:	mov    rcx,QWORD PTR [rsp+0x90]
   1b0f1:	add    rcx,rax
   1b0f4:	mov    QWORD PTR [rsp+0x270],rcx
   1b0fc:	jmp    1aadf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4adf>
   1b101:	lea    rcx,[r8+0x1]
   1b105:	mov    rax,QWORD PTR [rsp+0x10]
   1b10a:	add    rax,rcx
   1b10d:	mov    QWORD PTR [rsp+0x110],rax
   1b115:	mov    rax,QWORD PTR [rsp+0xf8]
   1b11d:	add    rax,rcx
   1b120:	mov    QWORD PTR [rsp+0x130],rax
   1b128:	mov    rax,QWORD PTR [rsp+0xf0]
   1b130:	add    rax,rcx
   1b133:	mov    QWORD PTR [rsp+0x150],rax
   1b13b:	mov    rax,QWORD PTR [rsp+0xe8]
   1b143:	add    rax,rcx
   1b146:	mov    QWORD PTR [rsp+0x170],rax
   1b14e:	mov    rax,QWORD PTR [rsp+0xe0]
   1b156:	add    rax,rcx
   1b159:	mov    QWORD PTR [rsp+0x190],rax
   1b161:	mov    rax,QWORD PTR [rsp+0xd8]
   1b169:	add    rax,rcx
   1b16c:	mov    QWORD PTR [rsp+0x1b0],rax
   1b174:	mov    rax,QWORD PTR [rsp+0xd0]
   1b17c:	add    rax,rcx
   1b17f:	mov    QWORD PTR [rsp+0x1d0],rax
   1b187:	mov    rax,QWORD PTR [rsp+0xc0]
   1b18f:	add    rax,rcx
   1b192:	mov    QWORD PTR [rsp+0x1f0],rax
   1b19a:	mov    rax,QWORD PTR [rsp+0xb0]
   1b1a2:	add    rax,rcx
   1b1a5:	mov    QWORD PTR [rsp+0x210],rax
   1b1ad:	mov    rax,QWORD PTR [rsp+0xa0]
   1b1b5:	add    rax,rcx
   1b1b8:	mov    QWORD PTR [rsp+0x230],rax
   1b1c0:	mov    rax,QWORD PTR [rsp+0x98]
   1b1c8:	add    rax,rcx
   1b1cb:	mov    QWORD PTR [rsp+0x250],rax
   1b1d3:	mov    rax,QWORD PTR [rsp+0x90]
   1b1db:	add    rax,rcx
   1b1de:	mov    QWORD PTR [rsp+0x270],rax
   1b1e6:	mov    rax,QWORD PTR [rsp+0x40]
   1b1eb:	add    rax,rcx
   1b1ee:	mov    QWORD PTR [rsp+0x290],rax
   1b1f6:	jmp    1aaef <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4aef>
   1b1fb:	lea    rcx,[r8+0x1]
   1b1ff:	mov    rax,QWORD PTR [rsp+0x10]
   1b204:	add    rax,rcx
   1b207:	mov    QWORD PTR [rsp+0x110],rax
   1b20f:	mov    rax,QWORD PTR [rsp+0xf8]
   1b217:	add    rax,rcx
   1b21a:	mov    QWORD PTR [rsp+0x130],rax
   1b222:	mov    rax,QWORD PTR [rsp+0xf0]
   1b22a:	add    rax,rcx
   1b22d:	mov    QWORD PTR [rsp+0x150],rax
   1b235:	mov    rax,QWORD PTR [rsp+0xe8]
   1b23d:	add    rax,rcx
   1b240:	mov    QWORD PTR [rsp+0x170],rax
   1b248:	mov    rax,QWORD PTR [rsp+0xe0]
   1b250:	add    rax,rcx
   1b253:	mov    QWORD PTR [rsp+0x190],rax
   1b25b:	mov    rax,QWORD PTR [rsp+0xd8]
   1b263:	add    rax,rcx
   1b266:	mov    QWORD PTR [rsp+0x1b0],rax
   1b26e:	mov    rax,QWORD PTR [rsp+0xd0]
   1b276:	add    rax,rcx
   1b279:	mov    QWORD PTR [rsp+0x1d0],rax
   1b281:	mov    rax,QWORD PTR [rsp+0xc0]
   1b289:	add    rax,rcx
   1b28c:	mov    QWORD PTR [rsp+0x1f0],rax
   1b294:	mov    rax,QWORD PTR [rsp+0xb0]
   1b29c:	add    rax,rcx
   1b29f:	mov    QWORD PTR [rsp+0x210],rax
   1b2a7:	mov    rax,QWORD PTR [rsp+0xa0]
   1b2af:	add    rax,rcx
   1b2b2:	mov    QWORD PTR [rsp+0x230],rax
   1b2ba:	mov    rax,QWORD PTR [rsp+0x98]
   1b2c2:	add    rax,rcx
   1b2c5:	mov    QWORD PTR [rsp+0x250],rax
   1b2cd:	mov    rax,QWORD PTR [rsp+0x90]
   1b2d5:	add    rax,rcx
   1b2d8:	mov    QWORD PTR [rsp+0x270],rax
   1b2e0:	mov    rax,QWORD PTR [rsp+0x40]
   1b2e5:	add    rax,rcx
   1b2e8:	mov    QWORD PTR [rsp+0x290],rax
   1b2f0:	mov    rax,QWORD PTR [rsp+0x38]
   1b2f5:	add    rax,rcx
   1b2f8:	mov    QWORD PTR [rsp+0x2b0],rax
   1b300:	jmp    1aaff <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4aff>
   1b305:	lea    rcx,[r8+0x1]
   1b309:	mov    rax,QWORD PTR [rsp+0x10]
   1b30e:	add    rax,rcx
   1b311:	mov    QWORD PTR [rsp+0x110],rax
   1b319:	mov    rax,QWORD PTR [rsp+0xf8]
   1b321:	add    rax,rcx
   1b324:	mov    QWORD PTR [rsp+0x130],rax
   1b32c:	mov    rax,QWORD PTR [rsp+0xf0]
   1b334:	add    rax,rcx
   1b337:	mov    QWORD PTR [rsp+0x150],rax
   1b33f:	mov    rax,QWORD PTR [rsp+0xe8]
   1b347:	add    rax,rcx
   1b34a:	mov    QWORD PTR [rsp+0x170],rax
   1b352:	mov    rax,QWORD PTR [rsp+0xe0]
   1b35a:	add    rax,rcx
   1b35d:	mov    QWORD PTR [rsp+0x190],rax
   1b365:	mov    rax,QWORD PTR [rsp+0xd8]
   1b36d:	add    rax,rcx
   1b370:	mov    QWORD PTR [rsp+0x1b0],rax
   1b378:	mov    rax,QWORD PTR [rsp+0xd0]
   1b380:	add    rax,rcx
   1b383:	mov    QWORD PTR [rsp+0x1d0],rax
   1b38b:	mov    rax,QWORD PTR [rsp+0xc0]
   1b393:	add    rax,rcx
   1b396:	mov    QWORD PTR [rsp+0x1f0],rax
   1b39e:	mov    rax,QWORD PTR [rsp+0xb0]
   1b3a6:	add    rax,rcx
   1b3a9:	mov    QWORD PTR [rsp+0x210],rax
   1b3b1:	mov    rax,QWORD PTR [rsp+0xa0]
   1b3b9:	add    rax,rcx
   1b3bc:	mov    QWORD PTR [rsp+0x230],rax
   1b3c4:	mov    rax,QWORD PTR [rsp+0x98]
   1b3cc:	add    rax,rcx
   1b3cf:	mov    QWORD PTR [rsp+0x250],rax
   1b3d7:	mov    rax,QWORD PTR [rsp+0x90]
   1b3df:	add    rax,rcx
   1b3e2:	mov    QWORD PTR [rsp+0x270],rax
   1b3ea:	mov    rax,QWORD PTR [rsp+0x40]
   1b3ef:	add    rax,rcx
   1b3f2:	mov    QWORD PTR [rsp+0x290],rax
   1b3fa:	mov    rax,QWORD PTR [rsp+0x38]
   1b3ff:	add    rax,rcx
   1b402:	mov    QWORD PTR [rsp+0x2b0],rax
   1b40a:	mov    rax,QWORD PTR [rsp+0x30]
   1b40f:	add    rax,rcx
   1b412:	mov    QWORD PTR [rsp+0x2d0],rax
   1b41a:	jmp    1ab12 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b12>
   1b41f:	lea    rcx,[r8+0x1]
   1b423:	mov    rax,QWORD PTR [rsp+0x10]
   1b428:	add    rax,rcx
   1b42b:	mov    QWORD PTR [rsp+0x110],rax
   1b433:	mov    rax,QWORD PTR [rsp+0xf8]
   1b43b:	add    rax,rcx
   1b43e:	mov    QWORD PTR [rsp+0x130],rax
   1b446:	mov    rax,QWORD PTR [rsp+0xf0]
   1b44e:	add    rax,rcx
   1b451:	mov    QWORD PTR [rsp+0x150],rax
   1b459:	mov    rax,QWORD PTR [rsp+0xe8]
   1b461:	add    rax,rcx
   1b464:	mov    QWORD PTR [rsp+0x170],rax
   1b46c:	mov    rax,QWORD PTR [rsp+0xe0]
   1b474:	add    rax,rcx
   1b477:	mov    QWORD PTR [rsp+0x190],rax
   1b47f:	mov    rax,QWORD PTR [rsp+0xd8]
   1b487:	add    rax,rcx
   1b48a:	mov    QWORD PTR [rsp+0x1b0],rax
   1b492:	mov    rax,QWORD PTR [rsp+0xd0]
   1b49a:	add    rax,rcx
   1b49d:	mov    QWORD PTR [rsp+0x1d0],rax
   1b4a5:	mov    rax,QWORD PTR [rsp+0xc0]
   1b4ad:	add    rax,rcx
   1b4b0:	mov    QWORD PTR [rsp+0x1f0],rax
   1b4b8:	mov    rax,QWORD PTR [rsp+0xb0]
   1b4c0:	add    rax,rcx
   1b4c3:	mov    QWORD PTR [rsp+0x210],rax
   1b4cb:	mov    rax,QWORD PTR [rsp+0xa0]
   1b4d3:	add    rax,rcx
   1b4d6:	mov    QWORD PTR [rsp+0x230],rax
   1b4de:	mov    rax,QWORD PTR [rsp+0x98]
   1b4e6:	add    rax,rcx
   1b4e9:	mov    QWORD PTR [rsp+0x250],rax
   1b4f1:	mov    rax,QWORD PTR [rsp+0x90]
   1b4f9:	add    rax,rcx
   1b4fc:	mov    QWORD PTR [rsp+0x270],rax
   1b504:	mov    rax,QWORD PTR [rsp+0x40]
   1b509:	add    rax,rcx
   1b50c:	mov    QWORD PTR [rsp+0x290],rax
   1b514:	mov    rax,QWORD PTR [rsp+0x38]
   1b519:	add    rax,rcx
   1b51c:	mov    QWORD PTR [rsp+0x2b0],rax
   1b524:	mov    rax,QWORD PTR [rsp+0x30]
   1b529:	add    rax,rcx
   1b52c:	mov    QWORD PTR [rsp+0x2d0],rax
   1b534:	mov    rax,QWORD PTR [rsp+0x88]
   1b53c:	add    rax,rcx
   1b53f:	mov    QWORD PTR [rsp+0x2f0],rax
   1b547:	jmp    1ab25 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b25>
   1b54c:	lea    rcx,[r8+0x1]
   1b550:	mov    rax,QWORD PTR [rsp+0x10]
   1b555:	add    rax,rcx
   1b558:	mov    QWORD PTR [rsp+0x110],rax
   1b560:	mov    rax,QWORD PTR [rsp+0xf8]
   1b568:	add    rax,rcx
   1b56b:	mov    QWORD PTR [rsp+0x130],rax
   1b573:	mov    rax,QWORD PTR [rsp+0xf0]
   1b57b:	add    rax,rcx
   1b57e:	mov    QWORD PTR [rsp+0x150],rax
   1b586:	mov    rax,QWORD PTR [rsp+0xe8]
   1b58e:	add    rax,rcx
   1b591:	mov    QWORD PTR [rsp+0x170],rax
   1b599:	mov    rax,QWORD PTR [rsp+0xe0]
   1b5a1:	add    rax,rcx
   1b5a4:	mov    QWORD PTR [rsp+0x190],rax
   1b5ac:	mov    rax,QWORD PTR [rsp+0xd8]
   1b5b4:	add    rax,rcx
   1b5b7:	mov    QWORD PTR [rsp+0x1b0],rax
   1b5bf:	mov    rax,QWORD PTR [rsp+0xd0]
   1b5c7:	add    rax,rcx
   1b5ca:	mov    QWORD PTR [rsp+0x1d0],rax
   1b5d2:	mov    rax,QWORD PTR [rsp+0xc0]
   1b5da:	add    rax,rcx
   1b5dd:	mov    QWORD PTR [rsp+0x1f0],rax
   1b5e5:	mov    rax,QWORD PTR [rsp+0xb0]
   1b5ed:	add    rax,rcx
   1b5f0:	mov    QWORD PTR [rsp+0x210],rax
   1b5f8:	mov    rax,QWORD PTR [rsp+0xa0]
   1b600:	add    rax,rcx
   1b603:	mov    QWORD PTR [rsp+0x230],rax
   1b60b:	mov    rax,QWORD PTR [rsp+0x98]
   1b613:	add    rax,rcx
   1b616:	mov    QWORD PTR [rsp+0x250],rax
   1b61e:	mov    rax,QWORD PTR [rsp+0x90]
   1b626:	add    rax,rcx
   1b629:	mov    QWORD PTR [rsp+0x270],rax
   1b631:	mov    rax,QWORD PTR [rsp+0x40]
   1b636:	add    rax,rcx
   1b639:	mov    QWORD PTR [rsp+0x290],rax
   1b641:	mov    rax,QWORD PTR [rsp+0x38]
   1b646:	add    rax,rcx
   1b649:	mov    QWORD PTR [rsp+0x2b0],rax
   1b651:	mov    rax,QWORD PTR [rsp+0x30]
   1b656:	add    rax,rcx
   1b659:	mov    QWORD PTR [rsp+0x2d0],rax
   1b661:	mov    rax,QWORD PTR [rsp+0x88]
   1b669:	add    rax,rcx
   1b66c:	mov    QWORD PTR [rsp+0x2f0],rax
   1b674:	mov    rax,QWORD PTR [rsp+0x80]
   1b67c:	add    rax,rcx
   1b67f:	mov    QWORD PTR [rsp+0x310],rax
   1b687:	jmp    1ab35 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b35>
   1b68c:	lea    rcx,[r8+0x1]
   1b690:	mov    rax,QWORD PTR [rsp+0x10]
   1b695:	add    rax,rcx
   1b698:	mov    QWORD PTR [rsp+0x110],rax
   1b6a0:	mov    rax,QWORD PTR [rsp+0xf8]
   1b6a8:	add    rax,rcx
   1b6ab:	mov    QWORD PTR [rsp+0x130],rax
   1b6b3:	mov    rax,QWORD PTR [rsp+0xf0]
   1b6bb:	add    rax,rcx
   1b6be:	mov    QWORD PTR [rsp+0x150],rax
   1b6c6:	mov    rax,QWORD PTR [rsp+0xe8]
   1b6ce:	add    rax,rcx
   1b6d1:	mov    QWORD PTR [rsp+0x170],rax
   1b6d9:	mov    rax,QWORD PTR [rsp+0xe0]
   1b6e1:	add    rax,rcx
   1b6e4:	mov    QWORD PTR [rsp+0x190],rax
   1b6ec:	mov    rax,QWORD PTR [rsp+0xd8]
   1b6f4:	add    rax,rcx
   1b6f7:	mov    QWORD PTR [rsp+0x1b0],rax
   1b6ff:	mov    rax,QWORD PTR [rsp+0xd0]
   1b707:	add    rax,rcx
   1b70a:	mov    QWORD PTR [rsp+0x1d0],rax
   1b712:	mov    rax,QWORD PTR [rsp+0xc0]
   1b71a:	add    rax,rcx
   1b71d:	mov    QWORD PTR [rsp+0x1f0],rax
   1b725:	mov    rax,QWORD PTR [rsp+0xb0]
   1b72d:	add    rax,rcx
   1b730:	mov    QWORD PTR [rsp+0x210],rax
   1b738:	mov    rax,QWORD PTR [rsp+0xa0]
   1b740:	add    rax,rcx
   1b743:	mov    QWORD PTR [rsp+0x230],rax
   1b74b:	mov    rax,QWORD PTR [rsp+0x98]
   1b753:	add    rax,rcx
   1b756:	mov    QWORD PTR [rsp+0x250],rax
   1b75e:	mov    rax,QWORD PTR [rsp+0x90]
   1b766:	add    rax,rcx
   1b769:	mov    QWORD PTR [rsp+0x270],rax
   1b771:	mov    rax,QWORD PTR [rsp+0x40]
   1b776:	add    rax,rcx
   1b779:	mov    QWORD PTR [rsp+0x290],rax
   1b781:	mov    rax,QWORD PTR [rsp+0x38]
   1b786:	add    rax,rcx
   1b789:	mov    QWORD PTR [rsp+0x2b0],rax
   1b791:	mov    rax,QWORD PTR [rsp+0x30]
   1b796:	add    rax,rcx
   1b799:	mov    QWORD PTR [rsp+0x2d0],rax
   1b7a1:	mov    rax,QWORD PTR [rsp+0x88]
   1b7a9:	add    rax,rcx
   1b7ac:	mov    QWORD PTR [rsp+0x2f0],rax
   1b7b4:	mov    rax,QWORD PTR [rsp+0x80]
   1b7bc:	add    rax,rcx
   1b7bf:	mov    QWORD PTR [rsp+0x310],rax
   1b7c7:	mov    rax,QWORD PTR [rsp+0x78]
   1b7cc:	add    rax,rcx
   1b7cf:	mov    QWORD PTR [rsp+0x330],rax
   1b7d7:	jmp    1ab45 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b45>
   1b7dc:	lea    rcx,[r8+0x1]
   1b7e0:	mov    rax,QWORD PTR [rsp+0x10]
   1b7e5:	add    rax,rcx
   1b7e8:	mov    QWORD PTR [rsp+0x110],rax
   1b7f0:	mov    rax,QWORD PTR [rsp+0xf8]
   1b7f8:	add    rax,rcx
   1b7fb:	mov    QWORD PTR [rsp+0x130],rax
   1b803:	mov    rax,QWORD PTR [rsp+0xf0]
   1b80b:	add    rax,rcx
   1b80e:	mov    QWORD PTR [rsp+0x150],rax
   1b816:	mov    rax,QWORD PTR [rsp+0xe8]
   1b81e:	add    rax,rcx
   1b821:	mov    QWORD PTR [rsp+0x170],rax
   1b829:	mov    rax,QWORD PTR [rsp+0xe0]
   1b831:	add    rax,rcx
   1b834:	mov    QWORD PTR [rsp+0x190],rax
   1b83c:	mov    rax,QWORD PTR [rsp+0xd8]
   1b844:	add    rax,rcx
   1b847:	mov    QWORD PTR [rsp+0x1b0],rax
   1b84f:	mov    rax,QWORD PTR [rsp+0xd0]
   1b857:	add    rax,rcx
   1b85a:	mov    QWORD PTR [rsp+0x1d0],rax
   1b862:	mov    rax,QWORD PTR [rsp+0xc0]
   1b86a:	add    rax,rcx
   1b86d:	mov    QWORD PTR [rsp+0x1f0],rax
   1b875:	mov    rax,QWORD PTR [rsp+0xb0]
   1b87d:	add    rax,rcx
   1b880:	mov    QWORD PTR [rsp+0x210],rax
   1b888:	mov    rax,QWORD PTR [rsp+0xa0]
   1b890:	add    rax,rcx
   1b893:	mov    QWORD PTR [rsp+0x230],rax
   1b89b:	mov    rax,QWORD PTR [rsp+0x98]
   1b8a3:	add    rax,rcx
   1b8a6:	mov    QWORD PTR [rsp+0x250],rax
   1b8ae:	mov    rax,QWORD PTR [rsp+0x90]
   1b8b6:	add    rax,rcx
   1b8b9:	mov    QWORD PTR [rsp+0x270],rax
   1b8c1:	mov    rax,QWORD PTR [rsp+0x40]
   1b8c6:	add    rax,rcx
   1b8c9:	mov    QWORD PTR [rsp+0x290],rax
   1b8d1:	mov    rax,QWORD PTR [rsp+0x38]
   1b8d6:	add    rax,rcx
   1b8d9:	mov    QWORD PTR [rsp+0x2b0],rax
   1b8e1:	mov    rax,QWORD PTR [rsp+0x30]
   1b8e6:	add    rax,rcx
   1b8e9:	mov    QWORD PTR [rsp+0x2d0],rax
   1b8f1:	mov    rax,QWORD PTR [rsp+0x88]
   1b8f9:	add    rax,rcx
   1b8fc:	mov    QWORD PTR [rsp+0x2f0],rax
   1b904:	mov    rax,QWORD PTR [rsp+0x80]
   1b90c:	add    rax,rcx
   1b90f:	mov    QWORD PTR [rsp+0x310],rax
   1b917:	mov    rax,QWORD PTR [rsp+0x78]
   1b91c:	add    rax,rcx
   1b91f:	mov    QWORD PTR [rsp+0x330],rax
   1b927:	mov    rax,QWORD PTR [rsp+0x70]
   1b92c:	add    rax,rcx
   1b92f:	mov    QWORD PTR [rsp+0x350],rax
   1b937:	jmp    1ab55 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b55>
   1b93c:	lea    rcx,[r8+0x1]
   1b940:	mov    rax,QWORD PTR [rsp+0x10]
   1b945:	add    rax,rcx
   1b948:	mov    QWORD PTR [rsp+0x110],rax
   1b950:	mov    rax,QWORD PTR [rsp+0xf8]
   1b958:	add    rax,rcx
   1b95b:	mov    QWORD PTR [rsp+0x130],rax
   1b963:	mov    rax,QWORD PTR [rsp+0xf0]
   1b96b:	add    rax,rcx
   1b96e:	mov    QWORD PTR [rsp+0x150],rax
   1b976:	mov    rax,QWORD PTR [rsp+0xe8]
   1b97e:	add    rax,rcx
   1b981:	mov    QWORD PTR [rsp+0x170],rax
   1b989:	mov    rax,QWORD PTR [rsp+0xe0]
   1b991:	add    rax,rcx
   1b994:	mov    QWORD PTR [rsp+0x190],rax
   1b99c:	mov    rax,QWORD PTR [rsp+0xd8]
   1b9a4:	add    rax,rcx
   1b9a7:	mov    QWORD PTR [rsp+0x1b0],rax
   1b9af:	mov    rax,QWORD PTR [rsp+0xd0]
   1b9b7:	add    rax,rcx
   1b9ba:	mov    QWORD PTR [rsp+0x1d0],rax
   1b9c2:	mov    rax,QWORD PTR [rsp+0xc0]
   1b9ca:	add    rax,rcx
   1b9cd:	mov    QWORD PTR [rsp+0x1f0],rax
   1b9d5:	mov    rax,QWORD PTR [rsp+0xb0]
   1b9dd:	add    rax,rcx
   1b9e0:	mov    QWORD PTR [rsp+0x210],rax
   1b9e8:	mov    rax,QWORD PTR [rsp+0xa0]
   1b9f0:	add    rax,rcx
   1b9f3:	mov    QWORD PTR [rsp+0x230],rax
   1b9fb:	mov    rax,QWORD PTR [rsp+0x98]
   1ba03:	add    rax,rcx
   1ba06:	mov    QWORD PTR [rsp+0x250],rax
   1ba0e:	mov    rax,QWORD PTR [rsp+0x90]
   1ba16:	add    rax,rcx
   1ba19:	mov    QWORD PTR [rsp+0x270],rax
   1ba21:	mov    rax,QWORD PTR [rsp+0x40]
   1ba26:	add    rax,rcx
   1ba29:	mov    QWORD PTR [rsp+0x290],rax
   1ba31:	mov    rax,QWORD PTR [rsp+0x38]
   1ba36:	add    rax,rcx
   1ba39:	mov    QWORD PTR [rsp+0x2b0],rax
   1ba41:	mov    rax,QWORD PTR [rsp+0x30]
   1ba46:	add    rax,rcx
   1ba49:	mov    QWORD PTR [rsp+0x2d0],rax
   1ba51:	mov    rax,QWORD PTR [rsp+0x88]
   1ba59:	add    rax,rcx
   1ba5c:	mov    QWORD PTR [rsp+0x2f0],rax
   1ba64:	mov    rax,QWORD PTR [rsp+0x80]
   1ba6c:	add    rax,rcx
   1ba6f:	mov    QWORD PTR [rsp+0x310],rax
   1ba77:	mov    rax,QWORD PTR [rsp+0x78]
   1ba7c:	add    rax,rcx
   1ba7f:	mov    QWORD PTR [rsp+0x330],rax
   1ba87:	mov    rax,QWORD PTR [rsp+0x70]
   1ba8c:	add    rax,rcx
   1ba8f:	mov    QWORD PTR [rsp+0x350],rax
   1ba97:	mov    rax,QWORD PTR [rsp+0x68]
   1ba9c:	add    rax,rcx
   1ba9f:	mov    QWORD PTR [rsp+0x370],rax
   1baa7:	jmp    1ab65 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b65>
   1baac:	lea    rcx,[r8+0x1]
   1bab0:	mov    rax,QWORD PTR [rsp+0x10]
   1bab5:	add    rax,rcx
   1bab8:	mov    QWORD PTR [rsp+0x110],rax
   1bac0:	mov    rax,QWORD PTR [rsp+0xf8]
   1bac8:	add    rax,rcx
   1bacb:	mov    QWORD PTR [rsp+0x130],rax
   1bad3:	mov    rax,QWORD PTR [rsp+0xf0]
   1badb:	add    rax,rcx
   1bade:	mov    QWORD PTR [rsp+0x150],rax
   1bae6:	mov    rax,QWORD PTR [rsp+0xe8]
   1baee:	add    rax,rcx
   1baf1:	mov    QWORD PTR [rsp+0x170],rax
   1baf9:	mov    rax,QWORD PTR [rsp+0xe0]
   1bb01:	add    rax,rcx
   1bb04:	mov    QWORD PTR [rsp+0x190],rax
   1bb0c:	mov    rax,QWORD PTR [rsp+0xd8]
   1bb14:	add    rax,rcx
   1bb17:	mov    QWORD PTR [rsp+0x1b0],rax
   1bb1f:	mov    rax,QWORD PTR [rsp+0xd0]
   1bb27:	add    rax,rcx
   1bb2a:	mov    QWORD PTR [rsp+0x1d0],rax
   1bb32:	mov    rax,QWORD PTR [rsp+0xc0]
   1bb3a:	add    rax,rcx
   1bb3d:	mov    QWORD PTR [rsp+0x1f0],rax
   1bb45:	mov    rax,QWORD PTR [rsp+0xb0]
   1bb4d:	add    rax,rcx
   1bb50:	mov    QWORD PTR [rsp+0x210],rax
   1bb58:	mov    rax,QWORD PTR [rsp+0xa0]
   1bb60:	add    rax,rcx
   1bb63:	mov    QWORD PTR [rsp+0x230],rax
   1bb6b:	mov    rax,QWORD PTR [rsp+0x98]
   1bb73:	add    rax,rcx
   1bb76:	mov    QWORD PTR [rsp+0x250],rax
   1bb7e:	mov    rax,QWORD PTR [rsp+0x90]
   1bb86:	add    rax,rcx
   1bb89:	mov    QWORD PTR [rsp+0x270],rax
   1bb91:	mov    rax,QWORD PTR [rsp+0x40]
   1bb96:	add    rax,rcx
   1bb99:	mov    QWORD PTR [rsp+0x290],rax
   1bba1:	mov    rax,QWORD PTR [rsp+0x38]
   1bba6:	add    rax,rcx
   1bba9:	mov    QWORD PTR [rsp+0x2b0],rax
   1bbb1:	mov    rax,QWORD PTR [rsp+0x30]
   1bbb6:	add    rax,rcx
   1bbb9:	mov    QWORD PTR [rsp+0x2d0],rax
   1bbc1:	mov    rax,QWORD PTR [rsp+0x88]
   1bbc9:	add    rax,rcx
   1bbcc:	mov    QWORD PTR [rsp+0x2f0],rax
   1bbd4:	mov    rax,QWORD PTR [rsp+0x80]
   1bbdc:	add    rax,rcx
   1bbdf:	mov    QWORD PTR [rsp+0x310],rax
   1bbe7:	mov    rax,QWORD PTR [rsp+0x78]
   1bbec:	add    rax,rcx
   1bbef:	mov    QWORD PTR [rsp+0x330],rax
   1bbf7:	mov    rax,QWORD PTR [rsp+0x70]
   1bbfc:	add    rax,rcx
   1bbff:	mov    QWORD PTR [rsp+0x350],rax
   1bc07:	mov    rax,QWORD PTR [rsp+0x68]
   1bc0c:	add    rax,rcx
   1bc0f:	mov    QWORD PTR [rsp+0x370],rax
   1bc17:	mov    rax,QWORD PTR [rsp+0x60]
   1bc1c:	add    rax,rcx
   1bc1f:	mov    QWORD PTR [rsp+0x390],rax
   1bc27:	jmp    1ab75 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b75>
   1bc2c:	lea    rcx,[r8+0x1]
   1bc30:	mov    rax,QWORD PTR [rsp+0x10]
   1bc35:	add    rax,rcx
   1bc38:	mov    QWORD PTR [rsp+0x110],rax
   1bc40:	mov    rax,QWORD PTR [rsp+0xf8]
   1bc48:	add    rax,rcx
   1bc4b:	mov    QWORD PTR [rsp+0x130],rax
   1bc53:	mov    rax,QWORD PTR [rsp+0xf0]
   1bc5b:	add    rax,rcx
   1bc5e:	mov    QWORD PTR [rsp+0x150],rax
   1bc66:	mov    rax,QWORD PTR [rsp+0xe8]
   1bc6e:	add    rax,rcx
   1bc71:	mov    QWORD PTR [rsp+0x170],rax
   1bc79:	mov    rax,QWORD PTR [rsp+0xe0]
   1bc81:	add    rax,rcx
   1bc84:	mov    QWORD PTR [rsp+0x190],rax
   1bc8c:	mov    rax,QWORD PTR [rsp+0xd8]
   1bc94:	add    rax,rcx
   1bc97:	mov    QWORD PTR [rsp+0x1b0],rax
   1bc9f:	mov    rax,QWORD PTR [rsp+0xd0]
   1bca7:	add    rax,rcx
   1bcaa:	mov    QWORD PTR [rsp+0x1d0],rax
   1bcb2:	mov    rax,QWORD PTR [rsp+0xc0]
   1bcba:	add    rax,rcx
   1bcbd:	mov    QWORD PTR [rsp+0x1f0],rax
   1bcc5:	mov    rax,QWORD PTR [rsp+0xb0]
   1bccd:	add    rax,rcx
   1bcd0:	mov    QWORD PTR [rsp+0x210],rax
   1bcd8:	mov    rax,QWORD PTR [rsp+0xa0]
   1bce0:	add    rax,rcx
   1bce3:	mov    QWORD PTR [rsp+0x230],rax
   1bceb:	mov    rax,QWORD PTR [rsp+0x98]
   1bcf3:	add    rax,rcx
   1bcf6:	mov    QWORD PTR [rsp+0x250],rax
   1bcfe:	mov    rax,QWORD PTR [rsp+0x90]
   1bd06:	add    rax,rcx
   1bd09:	mov    QWORD PTR [rsp+0x270],rax
   1bd11:	mov    rax,QWORD PTR [rsp+0x40]
   1bd16:	add    rax,rcx
   1bd19:	mov    QWORD PTR [rsp+0x290],rax
   1bd21:	mov    rax,QWORD PTR [rsp+0x38]
   1bd26:	add    rax,rcx
   1bd29:	mov    QWORD PTR [rsp+0x2b0],rax
   1bd31:	mov    rax,QWORD PTR [rsp+0x30]
   1bd36:	add    rax,rcx
   1bd39:	mov    QWORD PTR [rsp+0x2d0],rax
   1bd41:	mov    rax,QWORD PTR [rsp+0x88]
   1bd49:	add    rax,rcx
   1bd4c:	mov    QWORD PTR [rsp+0x2f0],rax
   1bd54:	mov    rax,QWORD PTR [rsp+0x80]
   1bd5c:	add    rax,rcx
   1bd5f:	mov    QWORD PTR [rsp+0x310],rax
   1bd67:	mov    rax,QWORD PTR [rsp+0x78]
   1bd6c:	add    rax,rcx
   1bd6f:	mov    QWORD PTR [rsp+0x330],rax
   1bd77:	mov    rax,QWORD PTR [rsp+0x70]
   1bd7c:	add    rax,rcx
   1bd7f:	mov    QWORD PTR [rsp+0x350],rax
   1bd87:	mov    rax,QWORD PTR [rsp+0x68]
   1bd8c:	add    rax,rcx
   1bd8f:	mov    QWORD PTR [rsp+0x370],rax
   1bd97:	mov    rax,QWORD PTR [rsp+0x60]
   1bd9c:	add    rax,rcx
   1bd9f:	mov    QWORD PTR [rsp+0x390],rax
   1bda7:	mov    rax,QWORD PTR [rsp+0x58]
   1bdac:	add    rax,rcx
   1bdaf:	mov    QWORD PTR [rsp+0x3b0],rax
   1bdb7:	jmp    1ab85 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b85>
   1bdbc:	lea    rcx,[r8+0x1]
   1bdc0:	mov    rax,QWORD PTR [rsp+0x10]
   1bdc5:	add    rax,rcx
   1bdc8:	mov    QWORD PTR [rsp+0x110],rax
   1bdd0:	mov    rax,QWORD PTR [rsp+0xf8]
   1bdd8:	add    rax,rcx
   1bddb:	mov    QWORD PTR [rsp+0x130],rax
   1bde3:	mov    rax,QWORD PTR [rsp+0xf0]
   1bdeb:	add    rax,rcx
   1bdee:	mov    QWORD PTR [rsp+0x150],rax
   1bdf6:	mov    rax,QWORD PTR [rsp+0xe8]
   1bdfe:	add    rax,rcx
   1be01:	mov    QWORD PTR [rsp+0x170],rax
   1be09:	mov    rax,QWORD PTR [rsp+0xe0]
   1be11:	add    rax,rcx
   1be14:	mov    QWORD PTR [rsp+0x190],rax
   1be1c:	mov    rax,QWORD PTR [rsp+0xd8]
   1be24:	add    rax,rcx
   1be27:	mov    QWORD PTR [rsp+0x1b0],rax
   1be2f:	mov    rax,QWORD PTR [rsp+0xd0]
   1be37:	add    rax,rcx
   1be3a:	mov    QWORD PTR [rsp+0x1d0],rax
   1be42:	mov    rax,QWORD PTR [rsp+0xc0]
   1be4a:	add    rax,rcx
   1be4d:	mov    QWORD PTR [rsp+0x1f0],rax
   1be55:	mov    rax,QWORD PTR [rsp+0xb0]
   1be5d:	add    rax,rcx
   1be60:	mov    QWORD PTR [rsp+0x210],rax
   1be68:	mov    rax,QWORD PTR [rsp+0xa0]
   1be70:	add    rax,rcx
   1be73:	mov    QWORD PTR [rsp+0x230],rax
   1be7b:	mov    rax,QWORD PTR [rsp+0x98]
   1be83:	add    rax,rcx
   1be86:	mov    QWORD PTR [rsp+0x250],rax
   1be8e:	mov    rax,QWORD PTR [rsp+0x90]
   1be96:	add    rax,rcx
   1be99:	mov    QWORD PTR [rsp+0x270],rax
   1bea1:	mov    rax,QWORD PTR [rsp+0x40]
   1bea6:	add    rax,rcx
   1bea9:	mov    QWORD PTR [rsp+0x290],rax
   1beb1:	mov    rax,QWORD PTR [rsp+0x38]
   1beb6:	add    rax,rcx
   1beb9:	mov    QWORD PTR [rsp+0x2b0],rax
   1bec1:	mov    rax,QWORD PTR [rsp+0x30]
   1bec6:	add    rax,rcx
   1bec9:	mov    QWORD PTR [rsp+0x2d0],rax
   1bed1:	mov    rax,QWORD PTR [rsp+0x88]
   1bed9:	add    rax,rcx
   1bedc:	mov    QWORD PTR [rsp+0x2f0],rax
   1bee4:	mov    rax,QWORD PTR [rsp+0x80]
   1beec:	add    rax,rcx
   1beef:	mov    QWORD PTR [rsp+0x310],rax
   1bef7:	mov    rax,QWORD PTR [rsp+0x78]
   1befc:	add    rax,rcx
   1beff:	mov    QWORD PTR [rsp+0x330],rax
   1bf07:	mov    rax,QWORD PTR [rsp+0x70]
   1bf0c:	add    rax,rcx
   1bf0f:	mov    QWORD PTR [rsp+0x350],rax
   1bf17:	mov    rax,QWORD PTR [rsp+0x68]
   1bf1c:	add    rax,rcx
   1bf1f:	mov    QWORD PTR [rsp+0x370],rax
   1bf27:	mov    rax,QWORD PTR [rsp+0x60]
   1bf2c:	add    rax,rcx
   1bf2f:	mov    QWORD PTR [rsp+0x390],rax
   1bf37:	mov    rax,QWORD PTR [rsp+0x58]
   1bf3c:	add    rax,rcx
   1bf3f:	mov    QWORD PTR [rsp+0x3b0],rax
   1bf47:	mov    rax,QWORD PTR [rsp+0x50]
   1bf4c:	add    rax,rcx
   1bf4f:	mov    QWORD PTR [rsp+0x3d0],rax
   1bf57:	jmp    1ab95 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4b95>
   1bf5c:	lea    rcx,[r8+0x1]
   1bf60:	mov    rax,QWORD PTR [rsp+0x10]
   1bf65:	add    rax,rcx
   1bf68:	mov    QWORD PTR [rsp+0x110],rax
   1bf70:	mov    rax,QWORD PTR [rsp+0xf8]
   1bf78:	add    rax,rcx
   1bf7b:	mov    QWORD PTR [rsp+0x130],rax
   1bf83:	mov    rax,QWORD PTR [rsp+0xf0]
   1bf8b:	add    rax,rcx
   1bf8e:	mov    QWORD PTR [rsp+0x150],rax
   1bf96:	mov    rax,QWORD PTR [rsp+0xe8]
   1bf9e:	add    rax,rcx
   1bfa1:	mov    QWORD PTR [rsp+0x170],rax
   1bfa9:	mov    rax,QWORD PTR [rsp+0xe0]
   1bfb1:	add    rax,rcx
   1bfb4:	mov    QWORD PTR [rsp+0x190],rax
   1bfbc:	mov    rax,QWORD PTR [rsp+0xd8]
   1bfc4:	add    rax,rcx
   1bfc7:	mov    QWORD PTR [rsp+0x1b0],rax
   1bfcf:	mov    rax,QWORD PTR [rsp+0xd0]
   1bfd7:	add    rax,rcx
   1bfda:	mov    QWORD PTR [rsp+0x1d0],rax
   1bfe2:	mov    rax,QWORD PTR [rsp+0xc0]
   1bfea:	add    rax,rcx
   1bfed:	mov    QWORD PTR [rsp+0x1f0],rax
   1bff5:	mov    rax,QWORD PTR [rsp+0xb0]
   1bffd:	add    rax,rcx
   1c000:	mov    QWORD PTR [rsp+0x210],rax
   1c008:	mov    rax,QWORD PTR [rsp+0xa0]
   1c010:	add    rax,rcx
   1c013:	mov    QWORD PTR [rsp+0x230],rax
   1c01b:	mov    rax,QWORD PTR [rsp+0x98]
   1c023:	add    rax,rcx
   1c026:	mov    QWORD PTR [rsp+0x250],rax
   1c02e:	mov    rax,QWORD PTR [rsp+0x90]
   1c036:	add    rax,rcx
   1c039:	mov    QWORD PTR [rsp+0x270],rax
   1c041:	mov    rax,QWORD PTR [rsp+0x40]
   1c046:	add    rax,rcx
   1c049:	mov    QWORD PTR [rsp+0x290],rax
   1c051:	mov    rax,QWORD PTR [rsp+0x38]
   1c056:	add    rax,rcx
   1c059:	mov    QWORD PTR [rsp+0x2b0],rax
   1c061:	mov    rax,QWORD PTR [rsp+0x30]
   1c066:	add    rax,rcx
   1c069:	mov    QWORD PTR [rsp+0x2d0],rax
   1c071:	mov    rax,QWORD PTR [rsp+0x88]
   1c079:	add    rax,rcx
   1c07c:	mov    QWORD PTR [rsp+0x2f0],rax
   1c084:	mov    rax,QWORD PTR [rsp+0x80]
   1c08c:	add    rax,rcx
   1c08f:	mov    QWORD PTR [rsp+0x310],rax
   1c097:	mov    rax,QWORD PTR [rsp+0x78]
   1c09c:	add    rax,rcx
   1c09f:	mov    QWORD PTR [rsp+0x330],rax
   1c0a7:	mov    rax,QWORD PTR [rsp+0x70]
   1c0ac:	add    rax,rcx
   1c0af:	mov    QWORD PTR [rsp+0x350],rax
   1c0b7:	mov    rax,QWORD PTR [rsp+0x68]
   1c0bc:	add    rax,rcx
   1c0bf:	mov    QWORD PTR [rsp+0x370],rax
   1c0c7:	mov    rax,QWORD PTR [rsp+0x60]
   1c0cc:	add    rax,rcx
   1c0cf:	mov    QWORD PTR [rsp+0x390],rax
   1c0d7:	mov    rax,QWORD PTR [rsp+0x58]
   1c0dc:	add    rax,rcx
   1c0df:	mov    QWORD PTR [rsp+0x3b0],rax
   1c0e7:	mov    rax,QWORD PTR [rsp+0x50]
   1c0ec:	add    rax,rcx
   1c0ef:	mov    QWORD PTR [rsp+0x3d0],rax
   1c0f7:	mov    rax,QWORD PTR [rsp+0x48]
   1c0fc:	add    rax,rcx
   1c0ff:	mov    QWORD PTR [rsp+0x3f0],rax
   1c107:	jmp    1aba5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4ba5>
   1c10c:	lea    rcx,[r8+0x1]
   1c110:	mov    rax,QWORD PTR [rsp+0x10]
   1c115:	add    rax,rcx
   1c118:	mov    QWORD PTR [rsp+0x110],rax
   1c120:	mov    rax,QWORD PTR [rsp+0xf8]
   1c128:	add    rax,rcx
   1c12b:	mov    QWORD PTR [rsp+0x130],rax
   1c133:	mov    rax,QWORD PTR [rsp+0xf0]
   1c13b:	add    rax,rcx
   1c13e:	mov    QWORD PTR [rsp+0x150],rax
   1c146:	mov    rax,QWORD PTR [rsp+0xe8]
   1c14e:	add    rax,rcx
   1c151:	mov    QWORD PTR [rsp+0x170],rax
   1c159:	mov    rax,QWORD PTR [rsp+0xe0]
   1c161:	add    rax,rcx
   1c164:	mov    QWORD PTR [rsp+0x190],rax
   1c16c:	mov    rax,QWORD PTR [rsp+0xd8]
   1c174:	add    rax,rcx
   1c177:	mov    QWORD PTR [rsp+0x1b0],rax
   1c17f:	mov    rax,QWORD PTR [rsp+0xd0]
   1c187:	add    rax,rcx
   1c18a:	mov    QWORD PTR [rsp+0x1d0],rax
   1c192:	mov    rax,QWORD PTR [rsp+0xc0]
   1c19a:	add    rax,rcx
   1c19d:	mov    QWORD PTR [rsp+0x1f0],rax
   1c1a5:	mov    rax,QWORD PTR [rsp+0xb0]
   1c1ad:	add    rax,rcx
   1c1b0:	mov    QWORD PTR [rsp+0x210],rax
   1c1b8:	mov    rax,QWORD PTR [rsp+0xa0]
   1c1c0:	add    rax,rcx
   1c1c3:	mov    QWORD PTR [rsp+0x230],rax
   1c1cb:	mov    rax,QWORD PTR [rsp+0x98]
   1c1d3:	add    rax,rcx
   1c1d6:	mov    QWORD PTR [rsp+0x250],rax
   1c1de:	mov    rax,QWORD PTR [rsp+0x90]
   1c1e6:	add    rax,rcx
   1c1e9:	mov    QWORD PTR [rsp+0x270],rax
   1c1f1:	mov    rax,QWORD PTR [rsp+0x40]
   1c1f6:	add    rax,rcx
   1c1f9:	mov    QWORD PTR [rsp+0x290],rax
   1c201:	mov    rax,QWORD PTR [rsp+0x38]
   1c206:	add    rax,rcx
   1c209:	mov    QWORD PTR [rsp+0x2b0],rax
   1c211:	mov    rax,QWORD PTR [rsp+0x30]
   1c216:	add    rax,rcx
   1c219:	mov    QWORD PTR [rsp+0x2d0],rax
   1c221:	mov    rax,QWORD PTR [rsp+0x88]
   1c229:	add    rax,rcx
   1c22c:	mov    QWORD PTR [rsp+0x2f0],rax
   1c234:	mov    rax,QWORD PTR [rsp+0x80]
   1c23c:	add    rax,rcx
   1c23f:	mov    QWORD PTR [rsp+0x310],rax
   1c247:	mov    rax,QWORD PTR [rsp+0x78]
   1c24c:	add    rax,rcx
   1c24f:	mov    QWORD PTR [rsp+0x330],rax
   1c257:	mov    rax,QWORD PTR [rsp+0x70]
   1c25c:	add    rax,rcx
   1c25f:	mov    QWORD PTR [rsp+0x350],rax
   1c267:	mov    rax,QWORD PTR [rsp+0x68]
   1c26c:	add    rax,rcx
   1c26f:	mov    QWORD PTR [rsp+0x370],rax
   1c277:	mov    rax,QWORD PTR [rsp+0x60]
   1c27c:	add    rax,rcx
   1c27f:	mov    QWORD PTR [rsp+0x390],rax
   1c287:	mov    rax,QWORD PTR [rsp+0x58]
   1c28c:	add    rax,rcx
   1c28f:	mov    QWORD PTR [rsp+0x3b0],rax
   1c297:	mov    rax,QWORD PTR [rsp+0x50]
   1c29c:	add    rax,rcx
   1c29f:	mov    QWORD PTR [rsp+0x3d0],rax
   1c2a7:	mov    rax,QWORD PTR [rsp+0x48]
   1c2ac:	add    rax,rcx
   1c2af:	mov    QWORD PTR [rsp+0x3f0],rax
   1c2b7:	mov    rax,QWORD PTR [rsp+0x8]
   1c2bc:	add    rax,rcx
   1c2bf:	mov    QWORD PTR [rsp+0x410],rax
   1c2c7:	jmp    1abb5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4bb5>
   1c2cc:	lea    rcx,[r8+0x1]
   1c2d0:	mov    rax,QWORD PTR [rsp+0x10]
   1c2d5:	add    rax,rcx
   1c2d8:	mov    QWORD PTR [rsp+0x110],rax
   1c2e0:	mov    rax,QWORD PTR [rsp+0xf8]
   1c2e8:	add    rax,rcx
   1c2eb:	mov    QWORD PTR [rsp+0x130],rax
   1c2f3:	mov    rax,QWORD PTR [rsp+0xf0]
   1c2fb:	add    rax,rcx
   1c2fe:	mov    QWORD PTR [rsp+0x150],rax
   1c306:	mov    rax,QWORD PTR [rsp+0xe8]
   1c30e:	add    rax,rcx
   1c311:	mov    QWORD PTR [rsp+0x170],rax
   1c319:	mov    rax,QWORD PTR [rsp+0xe0]
   1c321:	add    rax,rcx
   1c324:	mov    QWORD PTR [rsp+0x190],rax
   1c32c:	mov    rax,QWORD PTR [rsp+0xd8]
   1c334:	add    rax,rcx
   1c337:	mov    QWORD PTR [rsp+0x1b0],rax
   1c33f:	mov    rax,QWORD PTR [rsp+0xd0]
   1c347:	add    rax,rcx
   1c34a:	mov    QWORD PTR [rsp+0x1d0],rax
   1c352:	mov    rax,QWORD PTR [rsp+0xc0]
   1c35a:	add    rax,rcx
   1c35d:	mov    QWORD PTR [rsp+0x1f0],rax
   1c365:	mov    rax,QWORD PTR [rsp+0xb0]
   1c36d:	add    rax,rcx
   1c370:	mov    QWORD PTR [rsp+0x210],rax
   1c378:	mov    rax,QWORD PTR [rsp+0xa0]
   1c380:	add    rax,rcx
   1c383:	mov    QWORD PTR [rsp+0x230],rax
   1c38b:	mov    rax,QWORD PTR [rsp+0x98]
   1c393:	add    rax,rcx
   1c396:	mov    QWORD PTR [rsp+0x250],rax
   1c39e:	mov    rax,QWORD PTR [rsp+0x90]
   1c3a6:	add    rax,rcx
   1c3a9:	mov    QWORD PTR [rsp+0x270],rax
   1c3b1:	mov    rax,QWORD PTR [rsp+0x40]
   1c3b6:	add    rax,rcx
   1c3b9:	mov    QWORD PTR [rsp+0x290],rax
   1c3c1:	mov    rax,QWORD PTR [rsp+0x38]
   1c3c6:	add    rax,rcx
   1c3c9:	mov    QWORD PTR [rsp+0x2b0],rax
   1c3d1:	mov    rax,QWORD PTR [rsp+0x30]
   1c3d6:	add    rax,rcx
   1c3d9:	mov    QWORD PTR [rsp+0x2d0],rax
   1c3e1:	mov    rax,QWORD PTR [rsp+0x88]
   1c3e9:	add    rax,rcx
   1c3ec:	mov    QWORD PTR [rsp+0x2f0],rax
   1c3f4:	mov    rax,QWORD PTR [rsp+0x80]
   1c3fc:	add    rax,rcx
   1c3ff:	mov    QWORD PTR [rsp+0x310],rax
   1c407:	mov    rax,QWORD PTR [rsp+0x78]
   1c40c:	add    rax,rcx
   1c40f:	mov    QWORD PTR [rsp+0x330],rax
   1c417:	mov    rax,QWORD PTR [rsp+0x70]
   1c41c:	add    rax,rcx
   1c41f:	mov    QWORD PTR [rsp+0x350],rax
   1c427:	mov    rax,QWORD PTR [rsp+0x68]
   1c42c:	add    rax,rcx
   1c42f:	mov    QWORD PTR [rsp+0x370],rax
   1c437:	mov    rax,QWORD PTR [rsp+0x60]
   1c43c:	add    rax,rcx
   1c43f:	mov    QWORD PTR [rsp+0x390],rax
   1c447:	mov    rax,QWORD PTR [rsp+0x58]
   1c44c:	add    rax,rcx
   1c44f:	mov    QWORD PTR [rsp+0x3b0],rax
   1c457:	mov    rax,QWORD PTR [rsp+0x50]
   1c45c:	add    rax,rcx
   1c45f:	mov    QWORD PTR [rsp+0x3d0],rax
   1c467:	mov    rax,QWORD PTR [rsp+0x48]
   1c46c:	add    rax,rcx
   1c46f:	mov    QWORD PTR [rsp+0x3f0],rax
   1c477:	mov    rax,QWORD PTR [rsp+0x8]
   1c47c:	add    rax,rcx
   1c47f:	mov    QWORD PTR [rsp+0x410],rax
   1c487:	mov    rax,QWORD PTR [rsp+0x18]
   1c48c:	add    rax,rcx
   1c48f:	mov    QWORD PTR [rsp+0x430],rax
   1c497:	jmp    1abc8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4bc8>
   1c49c:	lea    rcx,[r8+0x1]
   1c4a0:	mov    rax,QWORD PTR [rsp+0x10]
   1c4a5:	add    rax,rcx
   1c4a8:	mov    QWORD PTR [rsp+0x110],rax
   1c4b0:	mov    rax,QWORD PTR [rsp+0xf8]
   1c4b8:	add    rax,rcx
   1c4bb:	mov    QWORD PTR [rsp+0x130],rax
   1c4c3:	mov    rax,QWORD PTR [rsp+0xf0]
   1c4cb:	add    rax,rcx
   1c4ce:	mov    QWORD PTR [rsp+0x150],rax
   1c4d6:	mov    rax,QWORD PTR [rsp+0xe8]
   1c4de:	add    rax,rcx
   1c4e1:	mov    QWORD PTR [rsp+0x170],rax
   1c4e9:	mov    rax,QWORD PTR [rsp+0xe0]
   1c4f1:	add    rax,rcx
   1c4f4:	mov    QWORD PTR [rsp+0x190],rax
   1c4fc:	mov    rax,QWORD PTR [rsp+0xd8]
   1c504:	add    rax,rcx
   1c507:	mov    QWORD PTR [rsp+0x1b0],rax
   1c50f:	mov    rax,QWORD PTR [rsp+0xd0]
   1c517:	add    rax,rcx
   1c51a:	mov    QWORD PTR [rsp+0x1d0],rax
   1c522:	mov    rax,QWORD PTR [rsp+0xc0]
   1c52a:	add    rax,rcx
   1c52d:	mov    QWORD PTR [rsp+0x1f0],rax
   1c535:	mov    rax,QWORD PTR [rsp+0xb0]
   1c53d:	add    rax,rcx
   1c540:	mov    QWORD PTR [rsp+0x210],rax
   1c548:	mov    rax,QWORD PTR [rsp+0xa0]
   1c550:	add    rax,rcx
   1c553:	mov    QWORD PTR [rsp+0x230],rax
   1c55b:	mov    rax,QWORD PTR [rsp+0x98]
   1c563:	add    rax,rcx
   1c566:	mov    QWORD PTR [rsp+0x250],rax
   1c56e:	mov    rax,QWORD PTR [rsp+0x90]
   1c576:	add    rax,rcx
   1c579:	mov    QWORD PTR [rsp+0x270],rax
   1c581:	mov    rax,QWORD PTR [rsp+0x40]
   1c586:	add    rax,rcx
   1c589:	mov    QWORD PTR [rsp+0x290],rax
   1c591:	mov    rax,QWORD PTR [rsp+0x38]
   1c596:	add    rax,rcx
   1c599:	mov    QWORD PTR [rsp+0x2b0],rax
   1c5a1:	mov    rax,QWORD PTR [rsp+0x30]
   1c5a6:	add    rax,rcx
   1c5a9:	mov    QWORD PTR [rsp+0x2d0],rax
   1c5b1:	mov    rax,QWORD PTR [rsp+0x88]
   1c5b9:	add    rax,rcx
   1c5bc:	mov    QWORD PTR [rsp+0x2f0],rax
   1c5c4:	mov    rax,QWORD PTR [rsp+0x80]
   1c5cc:	add    rax,rcx
   1c5cf:	mov    QWORD PTR [rsp+0x310],rax
   1c5d7:	mov    rax,QWORD PTR [rsp+0x78]
   1c5dc:	add    rax,rcx
   1c5df:	mov    QWORD PTR [rsp+0x330],rax
   1c5e7:	mov    rax,QWORD PTR [rsp+0x70]
   1c5ec:	add    rax,rcx
   1c5ef:	mov    QWORD PTR [rsp+0x350],rax
   1c5f7:	mov    rax,QWORD PTR [rsp+0x68]
   1c5fc:	add    rax,rcx
   1c5ff:	mov    QWORD PTR [rsp+0x370],rax
   1c607:	mov    rax,QWORD PTR [rsp+0x60]
   1c60c:	add    rax,rcx
   1c60f:	mov    QWORD PTR [rsp+0x390],rax
   1c617:	mov    rax,QWORD PTR [rsp+0x58]
   1c61c:	add    rax,rcx
   1c61f:	mov    QWORD PTR [rsp+0x3b0],rax
   1c627:	mov    rax,QWORD PTR [rsp+0x50]
   1c62c:	add    rax,rcx
   1c62f:	mov    QWORD PTR [rsp+0x3d0],rax
   1c637:	mov    rax,QWORD PTR [rsp+0x48]
   1c63c:	add    rax,rcx
   1c63f:	mov    QWORD PTR [rsp+0x3f0],rax
   1c647:	mov    rax,QWORD PTR [rsp+0x8]
   1c64c:	add    rax,rcx
   1c64f:	mov    QWORD PTR [rsp+0x410],rax
   1c657:	mov    rax,QWORD PTR [rsp+0x18]
   1c65c:	add    rax,rcx
   1c65f:	mov    QWORD PTR [rsp+0x430],rax
   1c667:	mov    rax,QWORD PTR [rsp+0xc8]
   1c66f:	add    rax,rcx
   1c672:	mov    QWORD PTR [rsp+0x450],rax
   1c67a:	jmp    1abdb <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4bdb>
   1c67f:	lea    rcx,[r8+0x1]
   1c683:	mov    rax,QWORD PTR [rsp+0x10]
   1c688:	add    rax,rcx
   1c68b:	mov    QWORD PTR [rsp+0x110],rax
   1c693:	mov    rax,QWORD PTR [rsp+0xf8]
   1c69b:	add    rax,rcx
   1c69e:	mov    QWORD PTR [rsp+0x130],rax
   1c6a6:	mov    rax,QWORD PTR [rsp+0xf0]
   1c6ae:	add    rax,rcx
   1c6b1:	mov    QWORD PTR [rsp+0x150],rax
   1c6b9:	mov    rax,QWORD PTR [rsp+0xe8]
   1c6c1:	add    rax,rcx
   1c6c4:	mov    QWORD PTR [rsp+0x170],rax
   1c6cc:	mov    rax,QWORD PTR [rsp+0xe0]
   1c6d4:	add    rax,rcx
   1c6d7:	mov    QWORD PTR [rsp+0x190],rax
   1c6df:	mov    rax,QWORD PTR [rsp+0xd8]
   1c6e7:	add    rax,rcx
   1c6ea:	mov    QWORD PTR [rsp+0x1b0],rax
   1c6f2:	mov    rax,QWORD PTR [rsp+0xd0]
   1c6fa:	add    rax,rcx
   1c6fd:	mov    QWORD PTR [rsp+0x1d0],rax
   1c705:	mov    rax,QWORD PTR [rsp+0xc0]
   1c70d:	add    rax,rcx
   1c710:	mov    QWORD PTR [rsp+0x1f0],rax
   1c718:	mov    rax,QWORD PTR [rsp+0xb0]
   1c720:	add    rax,rcx
   1c723:	mov    QWORD PTR [rsp+0x210],rax
   1c72b:	mov    rax,QWORD PTR [rsp+0xa0]
   1c733:	add    rax,rcx
   1c736:	mov    QWORD PTR [rsp+0x230],rax
   1c73e:	mov    rax,QWORD PTR [rsp+0x98]
   1c746:	add    rax,rcx
   1c749:	mov    QWORD PTR [rsp+0x250],rax
   1c751:	mov    rax,QWORD PTR [rsp+0x90]
   1c759:	add    rax,rcx
   1c75c:	mov    QWORD PTR [rsp+0x270],rax
   1c764:	mov    rax,QWORD PTR [rsp+0x40]
   1c769:	add    rax,rcx
   1c76c:	mov    QWORD PTR [rsp+0x290],rax
   1c774:	mov    rax,QWORD PTR [rsp+0x38]
   1c779:	add    rax,rcx
   1c77c:	mov    QWORD PTR [rsp+0x2b0],rax
   1c784:	mov    rax,QWORD PTR [rsp+0x30]
   1c789:	add    rax,rcx
   1c78c:	mov    QWORD PTR [rsp+0x2d0],rax
   1c794:	mov    rax,QWORD PTR [rsp+0x88]
   1c79c:	add    rax,rcx
   1c79f:	mov    QWORD PTR [rsp+0x2f0],rax
   1c7a7:	mov    rax,QWORD PTR [rsp+0x80]
   1c7af:	add    rax,rcx
   1c7b2:	mov    QWORD PTR [rsp+0x310],rax
   1c7ba:	mov    rax,QWORD PTR [rsp+0x78]
   1c7bf:	add    rax,rcx
   1c7c2:	mov    QWORD PTR [rsp+0x330],rax
   1c7ca:	mov    rax,QWORD PTR [rsp+0x70]
   1c7cf:	add    rax,rcx
   1c7d2:	mov    QWORD PTR [rsp+0x350],rax
   1c7da:	mov    rax,QWORD PTR [rsp+0x68]
   1c7df:	add    rax,rcx
   1c7e2:	mov    QWORD PTR [rsp+0x370],rax
   1c7ea:	mov    rax,QWORD PTR [rsp+0x60]
   1c7ef:	add    rax,rcx
   1c7f2:	mov    QWORD PTR [rsp+0x390],rax
   1c7fa:	mov    rax,QWORD PTR [rsp+0x58]
   1c7ff:	add    rax,rcx
   1c802:	mov    QWORD PTR [rsp+0x3b0],rax
   1c80a:	mov    rax,QWORD PTR [rsp+0x50]
   1c80f:	add    rax,rcx
   1c812:	mov    QWORD PTR [rsp+0x3d0],rax
   1c81a:	mov    rax,QWORD PTR [rsp+0x48]
   1c81f:	add    rax,rcx
   1c822:	mov    QWORD PTR [rsp+0x3f0],rax
   1c82a:	mov    rax,QWORD PTR [rsp+0x8]
   1c82f:	add    rax,rcx
   1c832:	mov    QWORD PTR [rsp+0x410],rax
   1c83a:	add    r10,rcx
   1c83d:	mov    QWORD PTR [rsp+0x430],r10
   1c845:	mov    rax,QWORD PTR [rsp+0xc8]
   1c84d:	add    rax,rcx
   1c850:	mov    QWORD PTR [rsp+0x450],rax
   1c858:	mov    rax,QWORD PTR [rsp+0xb8]
   1c860:	add    rax,rcx
   1c863:	mov    QWORD PTR [rsp+0x470],rax
   1c86b:	jmp    1abee <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4bee>
   1c870:	lea    rdi,[r8+0x1]
   1c874:	mov    rsi,QWORD PTR [rsp+0x10]
   1c879:	add    rsi,rdi
   1c87c:	mov    QWORD PTR [rsp+0x110],rsi
   1c884:	mov    rsi,QWORD PTR [rsp+0xf8]
   1c88c:	add    rsi,rdi
   1c88f:	mov    QWORD PTR [rsp+0x130],rsi
   1c897:	mov    rsi,QWORD PTR [rsp+0xf0]
   1c89f:	add    rsi,rdi
   1c8a2:	mov    QWORD PTR [rsp+0x150],rsi
   1c8aa:	mov    rsi,QWORD PTR [rsp+0xe8]
   1c8b2:	add    rsi,rdi
   1c8b5:	mov    QWORD PTR [rsp+0x170],rsi
   1c8bd:	mov    rsi,QWORD PTR [rsp+0xe0]
   1c8c5:	add    rsi,rdi
   1c8c8:	mov    QWORD PTR [rsp+0x190],rsi
   1c8d0:	mov    rsi,QWORD PTR [rsp+0xd8]
   1c8d8:	add    rsi,rdi
   1c8db:	mov    QWORD PTR [rsp+0x1b0],rsi
   1c8e3:	mov    rsi,QWORD PTR [rsp+0xd0]
   1c8eb:	add    rsi,rdi
   1c8ee:	mov    QWORD PTR [rsp+0x1d0],rsi
   1c8f6:	mov    rsi,QWORD PTR [rsp+0xc0]
   1c8fe:	add    rsi,rdi
   1c901:	mov    QWORD PTR [rsp+0x1f0],rsi
   1c909:	mov    rsi,QWORD PTR [rsp+0xb0]
   1c911:	add    rsi,rdi
   1c914:	mov    QWORD PTR [rsp+0x210],rsi
   1c91c:	mov    rsi,QWORD PTR [rsp+0xa0]
   1c924:	add    rsi,rdi
   1c927:	mov    QWORD PTR [rsp+0x230],rsi
   1c92f:	mov    rsi,QWORD PTR [rsp+0x98]
   1c937:	add    rsi,rdi
   1c93a:	mov    QWORD PTR [rsp+0x250],rsi
   1c942:	mov    rsi,QWORD PTR [rsp+0x90]
   1c94a:	add    rsi,rdi
   1c94d:	mov    QWORD PTR [rsp+0x270],rsi
   1c955:	mov    rsi,QWORD PTR [rsp+0x40]
   1c95a:	add    rsi,rdi
   1c95d:	mov    QWORD PTR [rsp+0x290],rsi
   1c965:	mov    rsi,QWORD PTR [rsp+0x38]
   1c96a:	add    rsi,rdi
   1c96d:	mov    QWORD PTR [rsp+0x2b0],rsi
   1c975:	mov    rsi,QWORD PTR [rsp+0x30]
   1c97a:	add    rsi,rdi
   1c97d:	mov    QWORD PTR [rsp+0x2d0],rsi
   1c985:	mov    rsi,QWORD PTR [rsp+0x88]
   1c98d:	add    rsi,rdi
   1c990:	mov    QWORD PTR [rsp+0x2f0],rsi
   1c998:	mov    rsi,QWORD PTR [rsp+0x80]
   1c9a0:	add    rsi,rdi
   1c9a3:	mov    QWORD PTR [rsp+0x310],rsi
   1c9ab:	mov    rsi,QWORD PTR [rsp+0x78]
   1c9b0:	add    rsi,rdi
   1c9b3:	mov    QWORD PTR [rsp+0x330],rsi
   1c9bb:	mov    rsi,QWORD PTR [rsp+0x70]
   1c9c0:	add    rsi,rdi
   1c9c3:	mov    QWORD PTR [rsp+0x350],rsi
   1c9cb:	mov    rsi,QWORD PTR [rsp+0x68]
   1c9d0:	add    rsi,rdi
   1c9d3:	mov    QWORD PTR [rsp+0x370],rsi
   1c9db:	mov    rsi,QWORD PTR [rsp+0x60]
   1c9e0:	add    rsi,rdi
   1c9e3:	mov    QWORD PTR [rsp+0x390],rsi
   1c9eb:	mov    rsi,QWORD PTR [rsp+0x58]
   1c9f0:	add    rsi,rdi
   1c9f3:	mov    QWORD PTR [rsp+0x3b0],rsi
   1c9fb:	mov    rsi,QWORD PTR [rsp+0x50]
   1ca00:	add    rsi,rdi
   1ca03:	mov    QWORD PTR [rsp+0x3d0],rsi
   1ca0b:	mov    rsi,QWORD PTR [rsp+0x48]
   1ca10:	add    rsi,rdi
   1ca13:	mov    QWORD PTR [rsp+0x3f0],rsi
   1ca1b:	mov    rsi,QWORD PTR [rsp+0x8]
   1ca20:	add    rsi,rdi
   1ca23:	mov    QWORD PTR [rsp+0x410],rsi
   1ca2b:	add    r10,rdi
   1ca2e:	mov    QWORD PTR [rsp+0x430],r10
   1ca36:	add    r9,rdi
   1ca39:	mov    QWORD PTR [rsp+0x450],r9
   1ca41:	add    rcx,rdi
   1ca44:	mov    QWORD PTR [rsp+0x470],rcx
   1ca4c:	add    rax,rdi
   1ca4f:	mov    QWORD PTR [rsp+0x490],rax
   1ca57:	jmp    1abf9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4bf9>
   1ca5c:	mov    rdi,QWORD PTR [rsp+0x20]
   1ca61:	add    rdi,r8
   1ca64:	xor    eax,eax
   1ca66:	xor    ecx,ecx
   1ca68:	mov    r8,QWORD PTR [rsp+0x100]
   1ca70:	mov    r9,QWORD PTR [rsp+0x520]
   1ca78:	jmp    1cac5 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6ac5>
   1ca7a:	nop    WORD PTR [rax+rax*1+0x0]
   1ca80:	mov    rsi,QWORD PTR [rsp+rcx*8+0x540]
   1ca88:	add    rsi,r9
   1ca8b:	lea    rdx,[rsp+rax*1]
   1ca8f:	add    rdx,0x108
   1ca96:	mov    QWORD PTR [rsp+rax*1+0x110],rsi
   1ca9e:	mov    rsi,QWORD PTR [rsp+rcx*8+0x548]
   1caa6:	add    rsi,r9
   1caa9:	mov    QWORD PTR [rdx+0x28],rsi
   1caad:	add    rcx,0x2
   1cab1:	add    rax,0x40
   1cab5:	cmp    rax,0x400
   1cabb:	mov    rdx,QWORD PTR [rsp]
   1cabf:	je     1ac21 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c21>
   1cac5:	cmp    rcx,0x1e
   1cac9:	jb     1ca80 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6a80>
   1cacb:	lea    rdx,[rsp+rax*1]
   1cacf:	add    rdx,0x108
   1cad6:	cmp    rax,0x3c0
   1cadc:	jne    1cae3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6ae3>
   1cade:	mov    rsi,rdi
   1cae1:	jmp    1caa9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6aa9>
   1cae3:	mov    QWORD PTR [rsp+rax*1+0x110],rdi
   1caeb:	mov    rsi,rdi
   1caee:	jmp    1caa9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6aa9>
   1caf0:	mov    rcx,QWORD PTR [rsp+0x520]
   1caf8:	mov    rax,QWORD PTR [rsp+0x10]
   1cafd:	add    rax,rcx
   1cb00:	mov    QWORD PTR [rsp+0x110],rax
   1cb08:	mov    rax,QWORD PTR [rsp+0xf8]
   1cb10:	add    rax,rcx
   1cb13:	mov    QWORD PTR [rsp+0x130],rax
   1cb1b:	mov    rax,QWORD PTR [rsp+0xf0]
   1cb23:	add    rax,rcx
   1cb26:	mov    QWORD PTR [rsp+0x150],rax
   1cb2e:	mov    rax,QWORD PTR [rsp+0xe8]
   1cb36:	add    rax,rcx
   1cb39:	mov    QWORD PTR [rsp+0x170],rax
   1cb41:	mov    rax,QWORD PTR [rsp+0xe0]
   1cb49:	add    rax,rcx
   1cb4c:	mov    QWORD PTR [rsp+0x190],rax
   1cb54:	mov    rax,QWORD PTR [rsp+0xd8]
   1cb5c:	add    rax,rcx
   1cb5f:	mov    QWORD PTR [rsp+0x1b0],rax
   1cb67:	mov    rax,QWORD PTR [rsp+0xd0]
   1cb6f:	add    rax,rcx
   1cb72:	mov    QWORD PTR [rsp+0x1d0],rax
   1cb7a:	mov    rax,QWORD PTR [rsp+0xc0]
   1cb82:	add    rax,rcx
   1cb85:	mov    QWORD PTR [rsp+0x1f0],rax
   1cb8d:	mov    rax,QWORD PTR [rsp+0xb0]
   1cb95:	add    rax,rcx
   1cb98:	mov    QWORD PTR [rsp+0x210],rax
   1cba0:	mov    rax,QWORD PTR [rsp+0xa0]
   1cba8:	add    rax,rcx
   1cbab:	mov    QWORD PTR [rsp+0x230],rax
   1cbb3:	mov    rax,QWORD PTR [rsp+0x98]
   1cbbb:	add    rax,rcx
   1cbbe:	mov    QWORD PTR [rsp+0x250],rax
   1cbc6:	mov    rax,QWORD PTR [rsp+0x90]
   1cbce:	add    rax,rcx
   1cbd1:	mov    QWORD PTR [rsp+0x270],rax
   1cbd9:	mov    rax,QWORD PTR [rsp+0x40]
   1cbde:	add    rax,rcx
   1cbe1:	mov    QWORD PTR [rsp+0x290],rax
   1cbe9:	mov    rax,QWORD PTR [rsp+0x38]
   1cbee:	add    rax,rcx
   1cbf1:	mov    QWORD PTR [rsp+0x2b0],rax
   1cbf9:	mov    rax,QWORD PTR [rsp+0x30]
   1cbfe:	add    rax,rcx
   1cc01:	mov    QWORD PTR [rsp+0x2d0],rax
   1cc09:	mov    rax,QWORD PTR [rsp+0x88]
   1cc11:	add    rax,rcx
   1cc14:	mov    QWORD PTR [rsp+0x2f0],rax
   1cc1c:	mov    rax,QWORD PTR [rsp+0x80]
   1cc24:	add    rax,rcx
   1cc27:	mov    QWORD PTR [rsp+0x310],rax
   1cc2f:	mov    rax,QWORD PTR [rsp+0x78]
   1cc34:	add    rax,rcx
   1cc37:	mov    QWORD PTR [rsp+0x330],rax
   1cc3f:	mov    rax,QWORD PTR [rsp+0x70]
   1cc44:	add    rax,rcx
   1cc47:	mov    QWORD PTR [rsp+0x350],rax
   1cc4f:	mov    rax,QWORD PTR [rsp+0x68]
   1cc54:	add    rax,rcx
   1cc57:	mov    QWORD PTR [rsp+0x370],rax
   1cc5f:	mov    rax,QWORD PTR [rsp+0x60]
   1cc64:	add    rax,rcx
   1cc67:	mov    QWORD PTR [rsp+0x390],rax
   1cc6f:	mov    rax,QWORD PTR [rsp+0x58]
   1cc74:	add    rax,rcx
   1cc77:	mov    QWORD PTR [rsp+0x3b0],rax
   1cc7f:	mov    rax,QWORD PTR [rsp+0x50]
   1cc84:	add    rax,rcx
   1cc87:	mov    QWORD PTR [rsp+0x3d0],rax
   1cc8f:	mov    rax,QWORD PTR [rsp+0x48]
   1cc94:	add    rax,rcx
   1cc97:	mov    QWORD PTR [rsp+0x3f0],rax
   1cc9f:	mov    rax,QWORD PTR [rsp+0x8]
   1cca4:	add    rax,rcx
   1cca7:	mov    QWORD PTR [rsp+0x410],rax
   1ccaf:	add    r10,rcx
   1ccb2:	mov    QWORD PTR [rsp+0x430],r10
   1ccba:	mov    rax,QWORD PTR [rsp+0xc8]
   1ccc2:	add    rax,rcx
   1ccc5:	mov    QWORD PTR [rsp+0x450],rax
   1cccd:	mov    rax,QWORD PTR [rsp+0xb8]
   1ccd5:	add    rax,rcx
   1ccd8:	mov    QWORD PTR [rsp+0x470],rax
   1cce0:	mov    rax,QWORD PTR [rsp+0xa8]
   1cce8:	add    rax,rcx
   1cceb:	mov    QWORD PTR [rsp+0x490],rax
   1ccf3:	add    r9,rcx
   1ccf6:	mov    QWORD PTR [rsp+0x4b0],r9
   1ccfe:	mov    rax,QWORD PTR [rsp+0x28]
   1cd03:	add    rax,rcx
   1cd06:	mov    r8,QWORD PTR [rsp+0x100]
   1cd0e:	lea    rcx,[rsp+0x4d0]
   1cd16:	mov    QWORD PTR [rcx],rax
   1cd19:	jmp    1ac21 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c21>
   1cd1e:	mov    rax,QWORD PTR [rsp+0x18]
   1cd23:	mov    rdx,QWORD PTR [rax]
   1cd26:	mov    rax,QWORD PTR [rax+0x8]
   1cd2a:	nop    WORD PTR [rax+rax*1+0x0]
   1cd30:	add    rbp,rax
   1cd33:	mov    QWORD PTR [r12+r14*1+0x8],rbp
   1cd38:	add    rdx,rax
   1cd3b:	mov    QWORD PTR [r12+r14*1+0x10],rdx
   1cd40:	mov    rdx,QWORD PTR [rsp]
   1cd44:	add    r14,0x20
   1cd48:	cmp    rbx,r14
   1cd4b:	je     1cdaf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6daf>
   1cd4d:	mov    rbp,QWORD PTR [r12+r14*1+0x8]
   1cd52:	cmp    rbp,QWORD PTR [r12+r14*1+0x10]
   1cd57:	jne    1cd44 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6d44>
   1cd59:	mov    rsi,QWORD PTR [rdx+0x40]
   1cd5d:	mov    rdi,QWORD PTR [r12+r14*1]
   1cd61:	cmp    rdi,rsi
   1cd64:	jae    1d238 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7238>
   1cd6a:	mov    rcx,QWORD PTR [rdx+0x38]
   1cd6e:	lea    rsi,[rdi+rdi*2]
   1cd72:	mov    rdx,QWORD PTR [rcx+rsi*8]
   1cd76:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   1cd7b:	sub    rbp,rax
   1cd7e:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   1cd83:	mov    rdi,rdx
   1cd86:	sub    rdi,rbp
   1cd89:	cmp    rdx,rdi
   1cd8c:	jbe    1cd30 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6d30>
   1cd8e:	lea    rdi,[rcx+rsi*8]
   1cd92:	mov    ecx,0x1
   1cd97:	mov    r8d,0x1
   1cd9d:	mov    QWORD PTR [rsp+0x18],rdi
   1cda2:	mov    rsi,rbp
   1cda5:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1cdaa:	jmp    1cd1e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6d1e>
   1cdaf:	xor    r14d,r14d
   1cdb2:	jmp    1cdc3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dc3>
   1cdb4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   1cdc0:	xor    r14d,r14d
   1cdc3:	mov    rbx,r14
   1cdc6:	shl    rbx,0x5
   1cdca:	add    rbx,r12
   1cdcd:	jmp    1cdfe <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dfe>
   1cdcf:	mov    rax,QWORD PTR [rsp+0x18]
   1cdd4:	mov    rdx,QWORD PTR [rax]
   1cdd7:	mov    rax,QWORD PTR [rax+0x8]
   1cddb:	nop    DWORD PTR [rax+rax*1+0x0]
   1cde0:	add    rbp,rax
   1cde3:	mov    QWORD PTR [rbx+0x8],rbp
   1cde7:	add    rdx,rax
   1cdea:	mov    QWORD PTR [rbx+0x10],rdx
   1cdee:	mov    rdx,QWORD PTR [rsp]
   1cdf2:	inc    r14
   1cdf5:	add    rbx,0x20
   1cdf9:	cmp    r14,r13
   1cdfc:	jae    1cdc0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dc0>
   1cdfe:	mov    eax,DWORD PTR [rbx+0x18]
   1ce01:	mov    ecx,DWORD PTR [r15+rax*4]
   1ce05:	test   ecx,ecx
   1ce07:	js     1ce90 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6e90>
   1ce0d:	mov    eax,ecx
   1ce0f:	shr    eax,0x8
   1ce12:	mov    DWORD PTR [rbx+0x18],eax
   1ce15:	prefetcht0 BYTE PTR [r15+rax*4]
   1ce1a:	mov    rax,QWORD PTR [rbx+0x8]
   1ce1e:	mov    BYTE PTR [rax],cl
   1ce20:	mov    rbp,QWORD PTR [rbx+0x8]
   1ce24:	inc    rbp
   1ce27:	mov    QWORD PTR [rbx+0x8],rbp
   1ce2b:	cmp    rbp,QWORD PTR [rbx+0x10]
   1ce2f:	jne    1cdf2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6df2>
   1ce31:	mov    rsi,QWORD PTR [rdx+0x40]
   1ce35:	mov    rdi,QWORD PTR [rbx]
   1ce38:	cmp    rdi,rsi
   1ce3b:	jae    1d238 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7238>
   1ce41:	mov    rcx,QWORD PTR [rdx+0x38]
   1ce45:	lea    rsi,[rdi+rdi*2]
   1ce49:	mov    rdx,QWORD PTR [rcx+rsi*8]
   1ce4d:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   1ce52:	sub    rbp,rax
   1ce55:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   1ce5a:	mov    rdi,rdx
   1ce5d:	sub    rdi,rbp
   1ce60:	cmp    rdx,rdi
   1ce63:	jbe    1cde0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6de0>
   1ce69:	lea    rdi,[rcx+rsi*8]
   1ce6d:	mov    ecx,0x1
   1ce72:	mov    r8d,0x1
   1ce78:	mov    QWORD PTR [rsp+0x18],rdi
   1ce7d:	mov    rsi,rbp
   1ce80:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1ce85:	jmp    1cdcf <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dcf>
   1ce8a:	nop    WORD PTR [rax+rax*1+0x0]
   1ce90:	mov    rsi,QWORD PTR [rdx+0x40]
   1ce94:	mov    rdi,QWORD PTR [rbx]
   1ce97:	cmp    rdi,rsi
   1ce9a:	jae    1d223 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7223>
   1cea0:	mov    rcx,QWORD PTR [rsp]
   1cea4:	mov    rdx,QWORD PTR [rcx+0x38]
   1cea8:	mov    rcx,QWORD PTR [rsp]
   1ceac:	mov    rcx,QWORD PTR [rcx+0x50]
   1ceb0:	lea    r8,[rdi+rdi*2]
   1ceb4:	mov    r9,QWORD PTR [rbx+0x8]
   1ceb8:	sub    r9,QWORD PTR [rdx+r8*8+0x8]
   1cebd:	mov    rsi,QWORD PTR [rsp]
   1cec1:	mov    rsi,QWORD PTR [rsi+0x58]
   1cec5:	mov    QWORD PTR [rdx+r8*8+0x10],r9
   1ceca:	mov    rdx,QWORD PTR [rsp]
   1cece:	mov    r9,QWORD PTR [rdx+0x10]
   1ced2:	test   r9,r9
   1ced5:	je     1cf26 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6f26>
   1ced7:	mov    r8,QWORD PTR [rdx+0x8]
   1cedb:	xor    edx,edx
   1cedd:	cmp    r9,0x1
   1cee1:	je     1cf0d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6f0d>
   1cee3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   1cef0:	mov    r10,rdx
   1cef3:	mov    r11,r9
   1cef6:	shr    r11,1
   1cef9:	add    rdx,r11
   1cefc:	cmp    DWORD PTR [r8+rdx*4],eax
   1cf00:	cmova  rdx,r10
   1cf04:	sub    r9,r11
   1cf07:	cmp    r9,0x1
   1cf0b:	ja     1cef0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6ef0>
   1cf0d:	cmp    DWORD PTR [r8+rdx*4],eax
   1cf11:	mov    rax,0xffffffffffffffff
   1cf18:	cmovne rdx,rax
   1cf1c:	cmp    rdi,rsi
   1cf1f:	jb     1cf36 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6f36>
   1cf21:	jmp    1d22c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x722c>
   1cf26:	mov    rdx,0xffffffffffffffff
   1cf2d:	cmp    rdi,rsi
   1cf30:	jae    1d22c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x722c>
   1cf36:	mov    QWORD PTR [rcx+rdi*8],rdx
   1cf3a:	dec    r13
   1cf3d:	mov    rax,r13
   1cf40:	shl    rax,0x5
   1cf44:	movups xmm0,XMMWORD PTR [r12+rax*1]
   1cf49:	movups xmm1,XMMWORD PTR [r12+rax*1+0x10]
   1cf4f:	movups XMMWORD PTR [rbx+0x10],xmm1
   1cf53:	movups XMMWORD PTR [rbx],xmm0
   1cf56:	cmp    r14,r13
   1cf59:	mov    rdx,QWORD PTR [rsp]
   1cf5d:	jb     1cdc3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dc3>
   1cf63:	mov    r14d,0x0
   1cf69:	test   r13,r13
   1cf6c:	jne    1cdc3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6dc3>
   1cf72:	mov    r14,QWORD PTR [rsp+0x8]
   1cf77:	mov    rcx,QWORD PTR [rdx+0x10]
   1cf7b:	mov    rax,rdx
   1cf7e:	mov    rdx,0xffffffffffffffff
   1cf85:	test   rcx,rcx
   1cf88:	mov    rsi,QWORD PTR [rsp+0x648]
   1cf90:	je     1d01c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x701c>
   1cf96:	mov    rax,QWORD PTR [rax+0x8]
   1cf9a:	lea    r9,[rcx*4+0x0]
   1cfa2:	xor    r8d,r8d
   1cfa5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   1cfb0:	mov    edi,DWORD PTR [rax+r8*1]
   1cfb4:	cmp    rsi,rdi
   1cfb7:	jbe    1d249 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7249>
   1cfbd:	and    BYTE PTR [r15+rdi*4+0x3],0x7f
   1cfc3:	add    r8,0x4
   1cfc7:	cmp    r9,r8
   1cfca:	jne    1cfb0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6fb0>
   1cfcc:	test   rcx,rcx
   1cfcf:	je     1d01c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x701c>
   1cfd1:	xor    r8d,r8d
   1cfd4:	cmp    rcx,0x1
   1cfd8:	jne    1cfe4 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6fe4>
   1cfda:	mov    r9d,DWORD PTR [rsp+0x50c]
   1cfe2:	jmp    1d00d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x700d>
   1cfe4:	mov    r9d,DWORD PTR [rsp+0x50c]
   1cfec:	nop    DWORD PTR [rax+0x0]
   1cff0:	mov    rdx,r8
   1cff3:	mov    rdi,rcx
   1cff6:	shr    rdi,1
   1cff9:	add    r8,rdi
   1cffc:	cmp    DWORD PTR [rax+r8*4],r9d
   1d000:	cmova  r8,rdx
   1d004:	sub    rcx,rdi
   1d007:	cmp    rcx,0x1
   1d00b:	ja     1cff0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x6ff0>
   1d00d:	cmp    DWORD PTR [rax+r8*4],r9d
   1d011:	mov    rdx,0xffffffffffffffff
   1d018:	cmove  rdx,r8
   1d01c:	mov    rax,QWORD PTR [rsp]
   1d020:	mov    QWORD PTR [rax+0x68],rdx
   1d024:	mov    QWORD PTR [rax+0x60],rsi
   1d028:	test   r14,r14
   1d02b:	je     1d036 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7036>
   1d02d:	mov    rdi,r12
   1d030:	call   QWORD PTR [rip+0x546ea]        # 71720 <free@GLIBC_2.2.5>
   1d036:	add    rsp,0x668
   1d03d:	pop    rbx
   1d03e:	pop    r12
   1d040:	pop    r13
   1d042:	pop    r14
   1d044:	pop    r15
   1d046:	pop    rbp
   1d047:	ret
   1d048:	mov    ecx,0x4
   1d04d:	mov    rbx,r8
   1d050:	mov    r8d,0x4
   1d056:	xor    esi,esi
   1d058:	mov    rdx,rbx
   1d05b:	mov    ebp,r9d
   1d05e:	call   351b0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1d063:	mov    r9d,ebp
   1d066:	mov    rdi,QWORD PTR [rsp]
   1d06a:	mov    r8,rbx
   1d06d:	mov    r12,QWORD PTR [rdi+0x10]
   1d071:	jmp    1604c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4c>
   1d076:	cmp    r12,0x15
   1d07a:	jae    1d2e3 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72e3>
   1d080:	lea    rax,[rdi+r12*4]
   1d084:	lea    r8,[rdi+0x4]
   1d088:	mov    ecx,0x4
   1d08d:	mov    rdx,rdi
   1d090:	jmp    1d0a9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x70a9>
   1d092:	mov    r8,rdi
   1d095:	mov    DWORD PTR [r8],esi
   1d098:	lea    r8,[rdx+0x4]
   1d09c:	add    rcx,0x4
   1d0a0:	cmp    r8,rax
   1d0a3:	je     1d2eb <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72eb>
   1d0a9:	mov    r9d,DWORD PTR [rdx]
   1d0ac:	mov    esi,DWORD PTR [rdx+0x4]
   1d0af:	mov    rdx,r8
   1d0b2:	cmp    esi,r9d
   1d0b5:	jae    1d098 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7098>
   1d0b7:	mov    r8,rcx
   1d0ba:	mov    DWORD PTR [rdi+r8*1],r9d
   1d0be:	cmp    r8,0x4
   1d0c2:	je     1d092 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7092>
   1d0c4:	mov    r9d,DWORD PTR [rdi+r8*1-0x8]
   1d0c9:	add    r8,0xfffffffffffffffc
   1d0cd:	cmp    esi,r9d
   1d0d0:	jb     1d0ba <mibz2::bwt::room_segments_unrolled::<32, 1>+0x70ba>
   1d0d2:	add    r8,rdi
   1d0d5:	jmp    1d095 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7095>
   1d0d7:	lea    rdi,[rdx+0x48]
   1d0db:	mov    ecx,0x8
   1d0e0:	mov    r8d,0x8
   1d0e6:	mov    rbx,rsi
   1d0e9:	xor    esi,esi
   1d0eb:	mov    rdx,r12
   1d0ee:	mov    r14,r9
   1d0f1:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1d0f6:	mov    r9,r14
   1d0f9:	mov    rsi,rbx
   1d0fc:	mov    rdx,QWORD PTR [rsp]
   1d100:	mov    rbx,QWORD PTR [rdx+0x58]
   1d104:	mov    r14,QWORD PTR [rdx+0x50]
   1d108:	lea    rdi,[r14+rbx*8]
   1d10c:	cmp    r12,0x1
   1d110:	jne    16222 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x222>
   1d116:	jmp    16257 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x257>
   1d11b:	lea    rdi,[rbx+0x30]
   1d11f:	mov    ecx,0x8
   1d124:	mov    r8d,0x18
   1d12a:	mov    rsi,r13
   1d12d:	mov    rdx,rbp
   1d130:	mov    QWORD PTR [rsp+0x510],r9
   1d138:	call   1f090 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   1d13d:	mov    r9,QWORD PTR [rsp+0x510]
   1d145:	lea    rsi,[rsp+0x4d0]
   1d14d:	mov    rdx,QWORD PTR [rbx+0x40]
   1d151:	mov    rax,QWORD PTR [rbx+0x38]
   1d155:	inc    r13
   1d158:	test   bpl,0x1
   1d15c:	je     1616b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x16b>
   1d162:	dec    rbp
   1d165:	lea    rcx,[rdx+rdx*2]
   1d169:	mov    QWORD PTR [rax+rcx*8],0x0
   1d171:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   1d17a:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   1d183:	inc    rdx
   1d186:	cmp    r12,r13
   1d189:	je     16174 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x174>
   1d18f:	lea    rcx,[rdx+rdx*2]
   1d193:	lea    rax,[rax+rcx*8]
   1d197:	add    rax,0x28
   1d19b:	neg    rbp
   1d19e:	xor    ecx,ecx
   1d1a0:	xorps  xmm0,xmm0
   1d1a3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   1d1b0:	mov    QWORD PTR [rax-0x28],0x0
   1d1b8:	mov    QWORD PTR [rax-0x20],0x1
   1d1c0:	movups XMMWORD PTR [rax-0x18],xmm0
   1d1c4:	mov    QWORD PTR [rax-0x8],0x1
   1d1cc:	mov    QWORD PTR [rax],0x0
   1d1d3:	add    rax,0x30
   1d1d7:	add    rcx,0xfffffffffffffffe
   1d1db:	cmp    rbp,rcx
   1d1de:	jne    1d1b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x71b0>
   1d1e0:	sub    rdx,rcx
   1d1e3:	mov    rcx,QWORD PTR [rsp]
   1d1e7:	mov    QWORD PTR [rcx+0x40],rdx
   1d1eb:	cmp    r12,rdx
   1d1ee:	jbe    16185 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x185>
   1d1f4:	jmp    1a025 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x4025>
   1d1f9:	lea    rdx,[rip+0x52150]        # 6f350 <__frame_dummy_init_array_entry+0x20>
   1d200:	mov    rdi,rax
   1d203:	mov    rsi,r14
   1d206:	call   29c6b <core::panicking::panic_bounds_check>
   1d20b:	lea    rdx,[rip+0x521fe]        # 6f410 <__frame_dummy_init_array_entry+0xe0>
   1d212:	call   29c6b <core::panicking::panic_bounds_check>
   1d217:	lea    rdx,[rip+0x521da]        # 6f3f8 <__frame_dummy_init_array_entry+0xc8>
   1d21e:	call   29c6b <core::panicking::panic_bounds_check>
   1d223:	lea    rax,[rip+0x52186]        # 6f3b0 <__frame_dummy_init_array_entry+0x80>
   1d22a:	jmp    1d233 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7233>
   1d22c:	lea    rax,[rip+0x52195]        # 6f3c8 <__frame_dummy_init_array_entry+0x98>
   1d233:	mov    QWORD PTR [rsp+0x10],rax
   1d238:	mov    rdx,QWORD PTR [rsp+0x10]
   1d23d:	mov    r14,QWORD PTR [rsp+0x8]
   1d242:	call   29c6b <core::panicking::panic_bounds_check>
   1d247:	jmp    1d2b7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72b7>
   1d249:	lea    rdx,[rip+0x52130]        # 6f380 <__frame_dummy_init_array_entry+0x50>
   1d250:	call   29c6b <core::panicking::panic_bounds_check>
   1d255:	jmp    1d2b7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x72b7>
   1d257:	lea    rdx,[rip+0x5216a]        # 6f3c8 <__frame_dummy_init_array_entry+0x98>
   1d25e:	call   29c6b <core::panicking::panic_bounds_check>
   1d263:	lea    rdx,[rip+0x52146]        # 6f3b0 <__frame_dummy_init_array_entry+0x80>
   1d26a:	call   29c6b <core::panicking::panic_bounds_check>
   1d26f:	lea    rdx,[rip+0x52182]        # 6f3f8 <__frame_dummy_init_array_entry+0xc8>
   1d276:	mov    rdi,rbx
   1d279:	call   29c6b <core::panicking::panic_bounds_check>
   1d27e:	lea    rdx,[rip+0x5218b]        # 6f410 <__frame_dummy_init_array_entry+0xe0>
   1d285:	mov    rdi,rbx
   1d288:	call   29c6b <core::panicking::panic_bounds_check>
   1d28d:	lea    rdx,[rip+0x52a0c]        # 6fca0 <__frame_dummy_init_array_entry+0x970>
   1d294:	mov    rdi,r12
   1d297:	call   29c6b <core::panicking::panic_bounds_check>
   1d29c:	lea    rdx,[rip+0x5213d]        # 6f3e0 <__frame_dummy_init_array_entry+0xb0>
   1d2a3:	call   29c6b <core::panicking::panic_bounds_check>
   1d2a8:	lea    rdx,[rip+0x529f1]        # 6fca0 <__frame_dummy_init_array_entry+0x970>
   1d2af:	mov    rdi,rbp
   1d2b2:	call   29c6b <core::panicking::panic_bounds_check>
   1d2b7:	ud2
   1d2b9:	lea    rdx,[rip+0x52078]        # 6f338 <__frame_dummy_init_array_entry+0x8>
   1d2c0:	mov    rsi,r14
   1d2c3:	call   29c6b <core::panicking::panic_bounds_check>
   1d2c8:	lea    rdi,[rip+0x52099]        # 6f368 <__frame_dummy_init_array_entry+0x38>
   1d2cf:	call   2c8f0 <core::panicking::panic_const::panic_const_div_by_zero>
   1d2d4:	mov    edi,0x8
   1d2d9:	mov    esi,0x80
   1d2de:	call   285f1 <alloc::raw_vec::handle_error>
   1d2e3:	mov    rsi,r12
   1d2e6:	call   352a0 <core::slice::sort::unstable::ipnsort::<u32, <u32 as core::cmp::PartialOrd>::lt>>
   1d2eb:	mov    rax,QWORD PTR [rsp]
   1d2ef:	mov    rdi,QWORD PTR [rax+0x8]
   1d2f3:	mov    r12,QWORD PTR [rax+0x10]
   1d2f7:	cmp    r12,0x2
   1d2fb:	jb     160cf <mibz2::bwt::room_segments_unrolled::<32, 1>+0xcf>
   1d301:	mov    ecx,0x1
   1d306:	mov    edx,DWORD PTR [rdi+rcx*4]
   1d309:	lea    rax,[rcx+0x1]
   1d30d:	cmp    edx,DWORD PTR [rdi+rcx*4-0x4]
   1d311:	je     1d320 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7320>
   1d313:	mov    rcx,rax
   1d316:	cmp    r12,rax
   1d319:	jne    1d306 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7306>
   1d31b:	jmp    160d8 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xd8>
   1d320:	cmp    rax,r12
   1d323:	jae    1d36b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x736b>
   1d325:	lea    rsi,[rcx-0x1]
   1d329:	mov    edx,r12d
   1d32c:	sub    edx,esi
   1d32e:	mov    r9,r12
   1d331:	lea    r8,[r12-0x3]
   1d336:	test   dl,0x1
   1d339:	jne    1d33d <mibz2::bwt::room_segments_unrolled::<32, 1>+0x733d>
   1d33b:	jmp    1d35b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x735b>
   1d33d:	mov    eax,DWORD PTR [rdi+rcx*4+0x4]
   1d341:	mov    rdx,rcx
   1d344:	cmp    eax,DWORD PTR [rdi+rcx*4-0x4]
   1d348:	je     1d351 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7351>
   1d34a:	mov    DWORD PTR [rdi+rcx*4],eax
   1d34d:	lea    rdx,[rcx+0x1]
   1d351:	add    rcx,0x2
   1d355:	mov    rax,rcx
   1d358:	mov    rcx,rdx
   1d35b:	cmp    r8,rsi
   1d35e:	jne    1d37b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x737b>
   1d360:	mov    rax,QWORD PTR [rsp]
   1d364:	mov    rdi,QWORD PTR [rax+0x8]
   1d368:	mov    rcx,rdx
   1d36b:	mov    rax,QWORD PTR [rsp]
   1d36f:	mov    QWORD PTR [rax+0x10],rcx
   1d373:	mov    r12,rcx
   1d376:	jmp    160cf <mibz2::bwt::room_segments_unrolled::<32, 1>+0xcf>
   1d37b:	mov    rsi,r9
   1d37e:	mov    rdx,rcx
   1d381:	jmp    1d38c <mibz2::bwt::room_segments_unrolled::<32, 1>+0x738c>
   1d383:	add    rax,0x2
   1d387:	cmp    rsi,rax
   1d38a:	je     1d360 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7360>
   1d38c:	mov    ecx,DWORD PTR [rdi+rax*4]
   1d38f:	cmp    ecx,DWORD PTR [rdi+rdx*4-0x4]
   1d393:	jne    1d3a1 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73a1>
   1d395:	mov    ecx,DWORD PTR [rdi+rax*4+0x4]
   1d399:	cmp    ecx,DWORD PTR [rdi+rdx*4-0x4]
   1d39d:	je     1d383 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7383>
   1d39f:	jmp    1d3b1 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73b1>
   1d3a1:	mov    DWORD PTR [rdi+rdx*4],ecx
   1d3a4:	inc    rdx
   1d3a7:	mov    ecx,DWORD PTR [rdi+rax*4+0x4]
   1d3ab:	cmp    ecx,DWORD PTR [rdi+rdx*4-0x4]
   1d3af:	je     1d383 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7383>
   1d3b1:	mov    DWORD PTR [rdi+rdx*4],ecx
   1d3b4:	inc    rdx
   1d3b7:	jmp    1d383 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x7383>
   1d3b9:	jmp    1d3bd <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73bd>
   1d3bb:	jmp    1d3c7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73c7>
   1d3bd:	mov    rbx,rax
   1d3c0:	mov    r14,QWORD PTR [rsp+0x8]
   1d3c5:	jmp    1d3e2 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73e2>
   1d3c7:	mov    rbx,rax
   1d3ca:	cmp    QWORD PTR [rsp+0x528],0x0
   1d3d3:	je     1d3f0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73f0>
   1d3d5:	mov    r12,QWORD PTR [rsp+0x530]
   1d3dd:	jmp    1d3e7 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73e7>
   1d3df:	mov    rbx,rax
   1d3e2:	test   r14,r14
   1d3e5:	je     1d3f0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x73f0>
   1d3e7:	mov    rdi,r12
   1d3ea:	call   QWORD PTR [rip+0x54330]        # 71720 <free@GLIBC_2.2.5>
   1d3f0:	mov    rdi,rbx
   1d3f3:	call   6e310 <_Unwind_Resume@plt>
   1d3f8:	int3
   1d3f9:	int3
   1d3fa:	int3
   1d3fb:	int3
   1d3fc:	int3
   1d3fd:	int3
   1d3fe:	int3
   1d3ff:	int3
