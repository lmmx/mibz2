
base/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

0000000000034400 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   34400:	push   rbp
   34401:	push   r15
   34403:	push   r14
   34405:	push   r13
   34407:	push   r12
   34409:	push   rbx
   3440a:	sub    rsp,0x678
   34411:	mov    rbp,rsi
   34414:	mov    rax,rdi
   34417:	mov    edi,edx
   34419:	cmp    rsi,rdi
   3441c:	jbe    36599 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2199>
   34422:	mov    QWORD PTR [rsp+0x38],rax
   34427:	mov    esi,DWORD PTR [rax+rdi*4]
   3442a:	mov    QWORD PTR [rcx+0x10],0x0
   34432:	movabs r12,0x2545f4914f6cdd1d
   3443c:	mov    QWORD PTR [rsp+0x30],rcx
   34441:	cmp    QWORD PTR [rcx],0x1fe
   34448:	jbe    36306 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f06>
   3444e:	xor    r9d,r9d
   34451:	shr    esi,0x8
   34454:	xor    r12,rbp
   34457:	mov    rax,QWORD PTR [rsp+0x30]
   3445c:	mov    r14,QWORD PTR [rax+0x8]
   34460:	mov    ecx,0x1ff
   34465:	mov    rbx,r9
   34468:	nop    DWORD PTR [rax+rax*1+0x0]
   34470:	mov    r15,rbx
   34473:	mov    rax,r12
   34476:	shl    rax,0xd
   3447a:	xor    rax,r12
   3447d:	mov    rdx,rax
   34480:	shr    rdx,0x7
   34484:	xor    rdx,rax
   34487:	mov    r12,rdx
   3448a:	shl    r12,0x11
   3448e:	xor    r12,rdx
   34491:	mov    rax,r12
   34494:	xor    edx,edx
   34496:	div    rbp
   34499:	mov    DWORD PTR [r14+rbx*4],edx
   3449d:	inc    rbx
   344a0:	dec    rcx
   344a3:	jne    34470 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x70>
   344a5:	mov    r12,QWORD PTR [rsp+0x30]
   344aa:	mov    QWORD PTR [r12+0x10],rbx
   344af:	cmp    rbx,QWORD PTR [r12]
   344b3:	mov    DWORD PTR [rsp+0x4c],esi
   344b7:	je     35fc7 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc7>
   344bd:	mov    DWORD PTR [r14+rbx*4],esi
   344c1:	lea    rax,[r15+0x2]
   344c5:	mov    r13,rax
   344c8:	mov    QWORD PTR [r12+0x10],rax
   344cd:	cmp    r15,0xfffffffffffffffd
   344d1:	jbe    36334 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f34>
   344d7:	test   r13,r13
   344da:	mov    rdx,QWORD PTR [rsp+0x38]
   344df:	je     365a8 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21a8>
   344e5:	lea    rax,[r13*4+0x0]
   344ed:	xor    ecx,ecx
   344ef:	nop
   344f0:	mov    edi,DWORD PTR [r14+rcx*1]
   344f4:	cmp    rbp,rdi
   344f7:	jbe    36496 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2096>
   344fd:	or     BYTE PTR [rdx+rdi*4+0x3],0x80
   34502:	add    rcx,0x4
   34506:	cmp    rax,rcx
   34509:	jne    344f0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf0>
   3450b:	lea    rax,[rbp*2+0x0]
   34513:	mov    rcx,rax
   34516:	or     rcx,r13
   34519:	shr    rcx,0x20
   3451d:	je     34526 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x126>
   3451f:	xor    edx,edx
   34521:	div    r13
   34524:	jmp    3452b <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12b>
   34526:	xor    edx,edx
   34528:	div    r13d
   3452b:	mov    r14,r13
   3452e:	mov    r13,QWORD PTR [r12+0x40]
   34533:	cmp    r13,r14
   34536:	mov    QWORD PTR [rsp+0x40],r14
   3453b:	cmova  r14,r13
   3453f:	jae    356b9 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12b9>
   34545:	sub    r14,r13
   34548:	mov    rcx,QWORD PTR [r12+0x30]
   3454d:	sub    rcx,r13
   34550:	mov    rdx,r13
   34553:	cmp    r14,rcx
   34556:	ja     363ce <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1fce>
   3455c:	mov    rsi,QWORD PTR [r12+0x38]
   34561:	inc    r13
   34564:	test   r14b,0x1
   34568:	jne    36406 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2006>
   3456e:	mov    rcx,QWORD PTR [rsp+0x40]
   34573:	cmp    rcx,r13
   34576:	mov    r13,rcx
   34579:	jne    3643b <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x203b>
   3457f:	mov    QWORD PTR [r12+0x40],rdx
   34584:	cmp    r13,rdx
   34587:	ja     356cf <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12cf>
   3458d:	lea    rbx,[rax+0x40]
   34591:	mov    r14,QWORD PTR [r12+0x38]
   34596:	lea    rcx,[r13*8+0x0]
   3459e:	lea    r15,[rcx+rcx*2]
   345a2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   345b0:	mov    QWORD PTR [r14+0x10],0x0
   345b8:	cmp    rbx,QWORD PTR [r14]
   345bb:	ja     345c9 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c9>
   345bd:	add    r14,0x18
   345c1:	add    r15,0xffffffffffffffe8
   345c5:	jne    345b0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   345c7:	jmp    345fb <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1fb>
   345c9:	mov    ecx,0x1
   345ce:	mov    r8d,0x1
   345d4:	mov    rdi,r14
   345d7:	xor    esi,esi
   345d9:	mov    rdx,rbx
   345dc:	mov    QWORD PTR [rsp+0x160],rax
   345e4:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   345e9:	mov    rax,QWORD PTR [rsp+0x160]
   345f1:	add    r14,0x18
   345f5:	add    r15,0xffffffffffffffe8
   345f9:	jne    345b0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   345fb:	mov    QWORD PTR [r12+0x58],0x0
   34604:	cmp    r13,QWORD PTR [r12+0x48]
   34609:	ja     36391 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f91>
   3460f:	xor    ebx,ebx
   34611:	mov    r14,QWORD PTR [r12+0x50]
   34616:	lea    rdi,[r14+rbx*8]
   3461a:	cmp    r13,0x1
   3461e:	je     3464b <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x24b>
   34620:	lea    rdx,[r13*8-0x8]
   34628:	mov    esi,0xff
   3462d:	mov    r15,rax
   34630:	call   QWORD PTR [rip+0x61a42]        # 96078 <memset@GLIBC_2.2.5>
   34636:	mov    rax,r15
   34639:	lea    rcx,[rbx+r13*1]
   3463d:	lea    rdi,[r14+rcx*8]
   34641:	add    rdi,0xfffffffffffffff8
   34645:	add    rbx,r13
   34648:	dec    rbx
   3464b:	mov    QWORD PTR [rdi],0xffffffffffffffff
   34652:	inc    rbx
   34655:	mov    QWORD PTR [r12+0x58],rbx
   3465a:	xorps  xmm0,xmm0
   3465d:	movups XMMWORD PTR [rsp+0x268],xmm0
   34665:	mov    DWORD PTR [rsp+0x278],0x0
   34670:	movups XMMWORD PTR [rsp+0x288],xmm0
   34678:	mov    DWORD PTR [rsp+0x298],0x0
   34683:	movups XMMWORD PTR [rsp+0x2a8],xmm0
   3468b:	mov    DWORD PTR [rsp+0x2b8],0x0
   34696:	movups XMMWORD PTR [rsp+0x2c8],xmm0
   3469e:	mov    DWORD PTR [rsp+0x2d8],0x0
   346a9:	movups XMMWORD PTR [rsp+0x2e8],xmm0
   346b1:	mov    DWORD PTR [rsp+0x2f8],0x0
   346bc:	movups XMMWORD PTR [rsp+0x308],xmm0
   346c4:	mov    DWORD PTR [rsp+0x318],0x0
   346cf:	movups XMMWORD PTR [rsp+0x328],xmm0
   346d7:	mov    DWORD PTR [rsp+0x338],0x0
   346e2:	movups XMMWORD PTR [rsp+0x348],xmm0
   346ea:	mov    DWORD PTR [rsp+0x358],0x0
   346f5:	movups XMMWORD PTR [rsp+0x368],xmm0
   346fd:	mov    DWORD PTR [rsp+0x378],0x0
   34708:	movups XMMWORD PTR [rsp+0x388],xmm0
   34710:	mov    DWORD PTR [rsp+0x398],0x0
   3471b:	movups XMMWORD PTR [rsp+0x3a8],xmm0
   34723:	mov    DWORD PTR [rsp+0x3b8],0x0
   3472e:	movups XMMWORD PTR [rsp+0x3c8],xmm0
   34736:	mov    DWORD PTR [rsp+0x3d8],0x0
   34741:	movups XMMWORD PTR [rsp+0x3e8],xmm0
   34749:	mov    DWORD PTR [rsp+0x3f8],0x0
   34754:	movups XMMWORD PTR [rsp+0x408],xmm0
   3475c:	mov    DWORD PTR [rsp+0x418],0x0
   34767:	movups XMMWORD PTR [rsp+0x428],xmm0
   3476f:	mov    DWORD PTR [rsp+0x438],0x0
   3477a:	movups XMMWORD PTR [rsp+0x448],xmm0
   34782:	mov    DWORD PTR [rsp+0x458],0x0
   3478d:	movups XMMWORD PTR [rsp+0x468],xmm0
   34795:	mov    DWORD PTR [rsp+0x478],0x0
   347a0:	movups XMMWORD PTR [rsp+0x488],xmm0
   347a8:	mov    DWORD PTR [rsp+0x498],0x0
   347b3:	movups XMMWORD PTR [rsp+0x4a8],xmm0
   347bb:	mov    DWORD PTR [rsp+0x4b8],0x0
   347c6:	movups XMMWORD PTR [rsp+0x4c8],xmm0
   347ce:	mov    DWORD PTR [rsp+0x4d8],0x0
   347d9:	movups XMMWORD PTR [rsp+0x4e8],xmm0
   347e1:	mov    DWORD PTR [rsp+0x4f8],0x0
   347ec:	movups XMMWORD PTR [rsp+0x508],xmm0
   347f4:	mov    DWORD PTR [rsp+0x518],0x0
   347ff:	movups XMMWORD PTR [rsp+0x528],xmm0
   34807:	mov    DWORD PTR [rsp+0x538],0x0
   34812:	movups XMMWORD PTR [rsp+0x548],xmm0
   3481a:	mov    DWORD PTR [rsp+0x558],0x0
   34825:	movups XMMWORD PTR [rsp+0x568],xmm0
   3482d:	mov    DWORD PTR [rsp+0x578],0x0
   34838:	movups XMMWORD PTR [rsp+0x588],xmm0
   34840:	mov    DWORD PTR [rsp+0x598],0x0
   3484b:	movups XMMWORD PTR [rsp+0x5a8],xmm0
   34853:	mov    DWORD PTR [rsp+0x5b8],0x0
   3485e:	movups XMMWORD PTR [rsp+0x5c8],xmm0
   34866:	mov    DWORD PTR [rsp+0x5d8],0x0
   34871:	movups XMMWORD PTR [rsp+0x5e8],xmm0
   34879:	mov    DWORD PTR [rsp+0x5f8],0x0
   34884:	movups XMMWORD PTR [rsp+0x608],xmm0
   3488c:	mov    DWORD PTR [rsp+0x618],0x0
   34897:	movups XMMWORD PTR [rsp+0x628],xmm0
   3489f:	mov    DWORD PTR [rsp+0x638],0x0
   348aa:	movups XMMWORD PTR [rsp+0x648],xmm0
   348b2:	mov    DWORD PTR [rsp+0x658],0x0
   348bd:	mov    QWORD PTR [rsp+0x260],0xffffffffffffffff
   348c9:	mov    QWORD PTR [rsp+0x280],0xffffffffffffffff
   348d5:	mov    QWORD PTR [rsp+0x2a0],0xffffffffffffffff
   348e1:	mov    QWORD PTR [rsp+0x2c0],0xffffffffffffffff
   348ed:	mov    QWORD PTR [rsp+0x2e0],0xffffffffffffffff
   348f9:	mov    QWORD PTR [rsp+0x300],0xffffffffffffffff
   34905:	mov    QWORD PTR [rsp+0x320],0xffffffffffffffff
   34911:	mov    QWORD PTR [rsp+0x340],0xffffffffffffffff
   3491d:	mov    QWORD PTR [rsp+0x360],0xffffffffffffffff
   34929:	mov    QWORD PTR [rsp+0x380],0xffffffffffffffff
   34935:	mov    QWORD PTR [rsp+0x3a0],0xffffffffffffffff
   34941:	mov    QWORD PTR [rsp+0x3c0],0xffffffffffffffff
   3494d:	mov    QWORD PTR [rsp+0x3e0],0xffffffffffffffff
   34959:	mov    QWORD PTR [rsp+0x400],0xffffffffffffffff
   34965:	mov    QWORD PTR [rsp+0x420],0xffffffffffffffff
   34971:	mov    QWORD PTR [rsp+0x440],0xffffffffffffffff
   3497d:	mov    QWORD PTR [rsp+0x460],0xffffffffffffffff
   34989:	mov    QWORD PTR [rsp+0x480],0xffffffffffffffff
   34995:	mov    QWORD PTR [rsp+0x4a0],0xffffffffffffffff
   349a1:	mov    QWORD PTR [rsp+0x4c0],0xffffffffffffffff
   349ad:	mov    QWORD PTR [rsp+0x4e0],0xffffffffffffffff
   349b9:	mov    QWORD PTR [rsp+0x500],0xffffffffffffffff
   349c5:	mov    QWORD PTR [rsp+0x520],0xffffffffffffffff
   349d1:	mov    QWORD PTR [rsp+0x540],0xffffffffffffffff
   349dd:	mov    QWORD PTR [rsp+0x560],0xffffffffffffffff
   349e9:	mov    QWORD PTR [rsp+0x580],0xffffffffffffffff
   349f5:	mov    QWORD PTR [rsp+0x5a0],0xffffffffffffffff
   34a01:	mov    QWORD PTR [rsp+0x5c0],0xffffffffffffffff
   34a0d:	mov    QWORD PTR [rsp+0x5e0],0xffffffffffffffff
   34a19:	mov    QWORD PTR [rsp+0x600],0xffffffffffffffff
   34a25:	mov    QWORD PTR [rsp+0x620],0xffffffffffffffff
   34a31:	mov    QWORD PTR [rsp+0x640],0xffffffffffffffff
   34a3d:	mov    rcx,QWORD PTR [r12+0x38]
   34a42:	mov    rbx,QWORD PTR [r12+0x40]
   34a47:	mov    r9,QWORD PTR [r12+0x8]
   34a4c:	mov    rsi,QWORD PTR [r12+0x10]
   34a51:	mov    QWORD PTR [rsp+0x8],rcx
   34a56:	lea    r11,[rcx+0x8]
   34a5a:	xor    edi,edi
   34a5c:	xor    r12d,r12d
   34a5f:	mov    r8,QWORD PTR [rsp+0x38]
   34a64:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34a70:	cmp    rbx,rdi
   34a73:	je     364a5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20a5>
   34a79:	cmp    rsi,rdi
   34a7c:	je     364b4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20b4>
   34a82:	mov    rcx,QWORD PTR [r11]
   34a85:	mov    edx,DWORD PTR [r9+rdi*4]
   34a89:	mov    edx,DWORD PTR [r8+rdx*4]
   34a8d:	mov    BYTE PTR [rcx],dl
   34a8f:	shr    edx,0x8
   34a92:	and    edx,0x7fffff
   34a98:	lea    r10,[rcx+0x1]
   34a9c:	add    rcx,QWORD PTR [r11-0x8]
   34aa0:	mov    QWORD PTR [rsp+r12*1+0x260],rdi
   34aa8:	mov    QWORD PTR [rsp+r12*1+0x268],r10
   34ab0:	mov    QWORD PTR [rsp+r12*1+0x270],rcx
   34ab8:	mov    DWORD PTR [rsp+r12*1+0x278],edx
   34ac0:	prefetcht0 BYTE PTR [r8+rdx*4]
   34ac5:	lea    rcx,[rdi+0x1]
   34ac9:	add    r12,0x20
   34acd:	cmp    rdi,0x1e
   34ad1:	ja     34adf <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6df>
   34ad3:	add    r11,0x18
   34ad7:	mov    rdi,rcx
   34ada:	cmp    rcx,r13
   34add:	jb     34a70 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x670>
   34adf:	cmp    rcx,0x20
   34ae3:	jne    356e0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12e0>
   34ae9:	mov    QWORD PTR [rsp+0x1d0],rcx
   34af1:	mov    QWORD PTR [rsp+0x58],rbp
   34af6:	add    rax,0x3f
   34afa:	mov    ebp,0x20
   34aff:	mov    QWORD PTR [rsp+0x160],rax
   34b07:	xor    r15d,r15d
   34b0a:	mov    r14,rax
   34b0d:	jmp    34b41 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x741>
   34b0f:	nop
   34b10:	add    r13,rdi
   34b13:	mov    QWORD PTR [rsp+r15*1+0x268],r13
   34b1b:	add    rdx,rdi
   34b1e:	mov    QWORD PTR [rsp+r15*1+0x270],rdx
   34b26:	sub    rdx,r13
   34b29:	add    r15,0x20
   34b2d:	cmp    rdx,r14
   34b30:	cmovb  r14,rdx
   34b34:	cmp    r15,0x400
   34b3b:	je     34bd0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x7d0>
   34b41:	mov    r13,QWORD PTR [rsp+r15*1+0x268]
   34b49:	mov    rdx,QWORD PTR [rsp+r15*1+0x270]
   34b51:	cmp    r13,rdx
   34b54:	jne    34b26 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x726>
   34b56:	mov    rdi,QWORD PTR [rsp+r15*1+0x260]
   34b5e:	cmp    rdi,rbx
   34b61:	jae    36579 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2179>
   34b67:	lea    rcx,[rdi+rdi*2]
   34b6b:	mov    rsi,QWORD PTR [rsp+0x8]
   34b70:	mov    rdx,QWORD PTR [rsi+rcx*8]
   34b74:	mov    rdi,QWORD PTR [rsi+rcx*8+0x8]
   34b79:	sub    r13,rdi
   34b7c:	mov    QWORD PTR [rsi+rcx*8+0x10],r13
   34b81:	mov    rsi,rdx
   34b84:	sub    rsi,r13
   34b87:	cmp    rdx,rsi
   34b8a:	jbe    34b10 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x710>
   34b8c:	mov    rsi,QWORD PTR [rsp+0x8]
   34b91:	lea    rdi,[rsi+rcx*8]
   34b95:	mov    QWORD PTR [rsp],rdi
   34b99:	mov    ecx,0x1
   34b9e:	mov    r8d,0x1
   34ba4:	mov    rsi,r13
   34ba7:	mov    QWORD PTR [rsp+0x28],rbx
   34bac:	mov    rbx,rax
   34baf:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   34bb4:	mov    rax,rbx
   34bb7:	mov    rbx,QWORD PTR [rsp+0x28]
   34bbc:	mov    rcx,QWORD PTR [rsp]
   34bc0:	mov    rdx,QWORD PTR [rcx]
   34bc3:	mov    rdi,QWORD PTR [rcx+0x8]
   34bc7:	jmp    34b10 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x710>
   34bcc:	nop    DWORD PTR [rax+0x0]
   34bd0:	mov    rax,QWORD PTR [rsp+0x268]
   34bd8:	mov    rcx,QWORD PTR [rsp+0x288]
   34be0:	mov    QWORD PTR [rsp+0x20],rcx
   34be5:	mov    rcx,QWORD PTR [rsp+0x2a8]
   34bed:	mov    QWORD PTR [rsp],rcx
   34bf1:	mov    rcx,QWORD PTR [rsp+0x2c8]
   34bf9:	mov    QWORD PTR [rsp+0x18],rcx
   34bfe:	mov    rdx,QWORD PTR [rsp+0x2e8]
   34c06:	mov    rsi,QWORD PTR [rsp+0x308]
   34c0e:	mov    rdi,QWORD PTR [rsp+0x328]
   34c16:	mov    r8,QWORD PTR [rsp+0x348]
   34c1e:	mov    r9,QWORD PTR [rsp+0x368]
   34c26:	mov    r10,QWORD PTR [rsp+0x388]
   34c2e:	mov    r11,QWORD PTR [rsp+0x3a8]
   34c36:	mov    rbx,QWORD PTR [rsp+0x3c8]
   34c3e:	mov    r15,QWORD PTR [rsp+0x3e8]
   34c46:	mov    r13,QWORD PTR [rsp+0x408]
   34c4e:	mov    rcx,QWORD PTR [rsp+0x428]
   34c56:	mov    QWORD PTR [rsp+0x10],rcx
   34c5b:	mov    QWORD PTR [rsp+0x50],rax
   34c60:	mov    QWORD PTR [rsp+0x60],rax
   34c65:	mov    rax,QWORD PTR [rsp+0x20]
   34c6a:	mov    QWORD PTR [rsp+0x68],rax
   34c6f:	mov    rax,QWORD PTR [rsp]
   34c73:	mov    QWORD PTR [rsp+0x70],rax
   34c78:	mov    rax,QWORD PTR [rsp+0x18]
   34c7d:	mov    QWORD PTR [rsp+0x78],rax
   34c82:	mov    QWORD PTR [rsp+0x258],rdx
   34c8a:	mov    QWORD PTR [rsp+0x80],rdx
   34c92:	mov    QWORD PTR [rsp+0x250],rsi
   34c9a:	mov    QWORD PTR [rsp+0x88],rsi
   34ca2:	mov    QWORD PTR [rsp+0x248],rdi
   34caa:	mov    QWORD PTR [rsp+0x90],rdi
   34cb2:	mov    QWORD PTR [rsp+0x240],r8
   34cba:	mov    QWORD PTR [rsp+0x98],r8
   34cc2:	mov    QWORD PTR [rsp+0x238],r9
   34cca:	mov    QWORD PTR [rsp+0xa0],r9
   34cd2:	mov    QWORD PTR [rsp+0x230],r10
   34cda:	mov    QWORD PTR [rsp+0xa8],r10
   34ce2:	mov    QWORD PTR [rsp+0x228],r11
   34cea:	mov    QWORD PTR [rsp+0xb0],r11
   34cf2:	mov    QWORD PTR [rsp+0x220],rbx
   34cfa:	mov    QWORD PTR [rsp+0xb8],rbx
   34d02:	mov    QWORD PTR [rsp+0x218],r15
   34d0a:	mov    QWORD PTR [rsp+0xc0],r15
   34d12:	mov    QWORD PTR [rsp+0x1d8],r13
   34d1a:	mov    QWORD PTR [rsp+0xc8],r13
   34d22:	mov    rax,QWORD PTR [rsp+0x10]
   34d27:	mov    QWORD PTR [rsp+0xd0],rax
   34d2f:	mov    rax,QWORD PTR [rsp+0x448]
   34d37:	mov    QWORD PTR [rsp+0x1e0],rax
   34d3f:	mov    QWORD PTR [rsp+0xd8],rax
   34d47:	mov    rax,QWORD PTR [rsp+0x468]
   34d4f:	mov    QWORD PTR [rsp+0x1e8],rax
   34d57:	mov    QWORD PTR [rsp+0xe0],rax
   34d5f:	mov    rax,QWORD PTR [rsp+0x488]
   34d67:	mov    QWORD PTR [rsp+0x1f0],rax
   34d6f:	mov    QWORD PTR [rsp+0xe8],rax
   34d77:	mov    rax,QWORD PTR [rsp+0x4a8]
   34d7f:	mov    QWORD PTR [rsp+0x1f8],rax
   34d87:	mov    QWORD PTR [rsp+0xf0],rax
   34d8f:	mov    rax,QWORD PTR [rsp+0x4c8]
   34d97:	mov    QWORD PTR [rsp+0x200],rax
   34d9f:	mov    QWORD PTR [rsp+0xf8],rax
   34da7:	mov    rax,QWORD PTR [rsp+0x4e8]
   34daf:	mov    QWORD PTR [rsp+0x208],rax
   34db7:	mov    QWORD PTR [rsp+0x100],rax
   34dbf:	mov    rax,QWORD PTR [rsp+0x508]
   34dc7:	mov    QWORD PTR [rsp+0x210],rax
   34dcf:	mov    QWORD PTR [rsp+0x108],rax
   34dd7:	mov    rcx,QWORD PTR [rsp+0x528]
   34ddf:	mov    QWORD PTR [rsp+0x110],rcx
   34de7:	mov    rdx,QWORD PTR [rsp+0x548]
   34def:	mov    QWORD PTR [rsp+0x118],rdx
   34df7:	mov    r10,QWORD PTR [rsp+0x568]
   34dff:	mov    QWORD PTR [rsp+0x120],r10
   34e07:	mov    r13,QWORD PTR [rsp+0x588]
   34e0f:	mov    QWORD PTR [rsp+0x128],r13
   34e17:	mov    rax,QWORD PTR [rsp+0x5a8]
   34e1f:	mov    QWORD PTR [rsp+0x130],rax
   34e27:	mov    r11,QWORD PTR [rsp+0x5c8]
   34e2f:	mov    QWORD PTR [rsp+0x138],r11
   34e37:	mov    r8,QWORD PTR [rsp+0x5e8]
   34e3f:	mov    QWORD PTR [rsp+0x140],r8
   34e47:	mov    r9,QWORD PTR [rsp+0x608]
   34e4f:	mov    QWORD PTR [rsp+0x148],r9
   34e57:	mov    rsi,QWORD PTR [rsp+0x628]
   34e5f:	mov    QWORD PTR [rsp+0x150],rsi
   34e67:	mov    rdi,QWORD PTR [rsp+0x648]
   34e6f:	mov    QWORD PTR [rsp+0x158],rdi
   34e77:	mov    rbx,QWORD PTR [rsp+0x30]
   34e7c:	mov    r15,QWORD PTR [rbx+0x40]
   34e80:	mov    QWORD PTR [rsp+0x28],r15
   34e85:	test   r14,r14
   34e88:	je     35270 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe70>
   34e8e:	mov    rax,QWORD PTR [rbx+0x50]
   34e92:	mov    QWORD PTR [rsp+0x18],rax
   34e97:	mov    rax,QWORD PTR [rbx+0x58]
   34e9b:	mov    QWORD PTR [rsp+0x20],rax
   34ea0:	mov    r10,QWORD PTR [rbx+0x8]
   34ea4:	mov    rsi,QWORD PTR [rbx+0x10]
   34ea8:	xor    r8d,r8d
   34eab:	mov    rbx,QWORD PTR [rsp+0x38]
   34eb0:	mov    QWORD PTR [rsp+0x10],r10
   34eb5:	mov    QWORD PTR [rsp],rsi
   34eb9:	jmp    34ecc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xacc>
   34ebb:	nop    DWORD PTR [rax+rax*1+0x0]
   34ec0:	inc    r8
   34ec3:	cmp    r8,r14
   34ec6:	je     35522 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1122>
   34ecc:	lea    rax,[rsp+0x358]
   34ed4:	xor    ecx,ecx
   34ed6:	xor    r9d,r9d
   34ed9:	jmp    34f09 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb09>
   34edb:	nop    DWORD PTR [rax+rax*1+0x0]
   34ee0:	mov    rdx,QWORD PTR [rsp+rcx*8+0x98]
   34ee8:	mov    BYTE PTR [rdx+r8*1],dil
   34eec:	shr    edi,0x8
   34eef:	mov    DWORD PTR [rax],edi
   34ef1:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34ef5:	add    rcx,0x8
   34ef9:	add    rax,0x100
   34eff:	cmp    rcx,0x19
   34f03:	jae    350e0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xce0>
   34f09:	mov    edx,DWORD PTR [rax-0xe0]
   34f0f:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f12:	test   edi,edi
   34f14:	js     35020 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc20>
   34f1a:	mov    rdx,QWORD PTR [rsp+rcx*8+0x60]
   34f1f:	mov    BYTE PTR [rdx+r8*1],dil
   34f23:	shr    edi,0x8
   34f26:	mov    DWORD PTR [rax-0xe0],edi
   34f2c:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f30:	mov    edx,DWORD PTR [rax-0xc0]
   34f36:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f39:	test   edi,edi
   34f3b:	js     35035 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc35>
   34f41:	mov    rdx,QWORD PTR [rsp+rcx*8+0x68]
   34f46:	mov    BYTE PTR [rdx+r8*1],dil
   34f4a:	shr    edi,0x8
   34f4d:	mov    DWORD PTR [rax-0xc0],edi
   34f53:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f57:	mov    edx,DWORD PTR [rax-0xa0]
   34f5d:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f60:	test   edi,edi
   34f62:	js     35051 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc51>
   34f68:	mov    rdx,QWORD PTR [rsp+rcx*8+0x70]
   34f6d:	mov    BYTE PTR [rdx+r8*1],dil
   34f71:	shr    edi,0x8
   34f74:	mov    DWORD PTR [rax-0xa0],edi
   34f7a:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f7e:	mov    edx,DWORD PTR [rax-0x80]
   34f81:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f84:	test   edi,edi
   34f86:	js     3506a <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc6a>
   34f8c:	mov    rdx,QWORD PTR [rsp+rcx*8+0x78]
   34f91:	mov    BYTE PTR [rdx+r8*1],dil
   34f95:	shr    edi,0x8
   34f98:	mov    DWORD PTR [rax-0x80],edi
   34f9b:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f9f:	mov    edx,DWORD PTR [rax-0x60]
   34fa2:	mov    edi,DWORD PTR [rbx+rdx*4]
   34fa5:	test   edi,edi
   34fa7:	js     35083 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc83>
   34fad:	mov    rdx,QWORD PTR [rsp+rcx*8+0x80]
   34fb5:	mov    BYTE PTR [rdx+r8*1],dil
   34fb9:	shr    edi,0x8
   34fbc:	mov    DWORD PTR [rax-0x60],edi
   34fbf:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34fc3:	mov    edx,DWORD PTR [rax-0x40]
   34fc6:	mov    edi,DWORD PTR [rbx+rdx*4]
   34fc9:	test   edi,edi
   34fcb:	js     3509c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc9c>
   34fd1:	mov    rdx,QWORD PTR [rsp+rcx*8+0x88]
   34fd9:	mov    BYTE PTR [rdx+r8*1],dil
   34fdd:	shr    edi,0x8
   34fe0:	mov    DWORD PTR [rax-0x40],edi
   34fe3:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34fe7:	mov    edx,DWORD PTR [rax-0x20]
   34fea:	mov    edi,DWORD PTR [rbx+rdx*4]
   34fed:	test   edi,edi
   34fef:	js     350b5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcb5>
   34ff5:	mov    rdx,QWORD PTR [rsp+rcx*8+0x90]
   34ffd:	mov    BYTE PTR [rdx+r8*1],dil
   35001:	shr    edi,0x8
   35004:	mov    DWORD PTR [rax-0x20],edi
   35007:	prefetcht0 BYTE PTR [rbx+rdi*4]
   3500b:	mov    edx,DWORD PTR [rax]
   3500d:	mov    edi,DWORD PTR [rbx+rdx*4]
   35010:	test   edi,edi
   35012:	jns    34ee0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xae0>
   35018:	jmp    350cd <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xccd>
   3501d:	nop    DWORD PTR [rax]
   35020:	bts    r9,rcx
   35024:	mov    edx,DWORD PTR [rax-0xc0]
   3502a:	mov    edi,DWORD PTR [rbx+rdx*4]
   3502d:	test   edi,edi
   3502f:	jns    34f41 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb41>
   35035:	mov    edx,0x2
   3503a:	shl    rdx,cl
   3503d:	or     r9,rdx
   35040:	mov    edx,DWORD PTR [rax-0xa0]
   35046:	mov    edi,DWORD PTR [rbx+rdx*4]
   35049:	test   edi,edi
   3504b:	jns    34f68 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb68>
   35051:	mov    edx,0x4
   35056:	shl    rdx,cl
   35059:	or     r9,rdx
   3505c:	mov    edx,DWORD PTR [rax-0x80]
   3505f:	mov    edi,DWORD PTR [rbx+rdx*4]
   35062:	test   edi,edi
   35064:	jns    34f8c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb8c>
   3506a:	mov    edx,0x8
   3506f:	shl    rdx,cl
   35072:	or     r9,rdx
   35075:	mov    edx,DWORD PTR [rax-0x60]
   35078:	mov    edi,DWORD PTR [rbx+rdx*4]
   3507b:	test   edi,edi
   3507d:	jns    34fad <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbad>
   35083:	mov    edx,0x10
   35088:	shl    rdx,cl
   3508b:	or     r9,rdx
   3508e:	mov    edx,DWORD PTR [rax-0x40]
   35091:	mov    edi,DWORD PTR [rbx+rdx*4]
   35094:	test   edi,edi
   35096:	jns    34fd1 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbd1>
   3509c:	mov    edx,0x20
   350a1:	shl    rdx,cl
   350a4:	or     r9,rdx
   350a7:	mov    edx,DWORD PTR [rax-0x20]
   350aa:	mov    edi,DWORD PTR [rbx+rdx*4]
   350ad:	test   edi,edi
   350af:	jns    34ff5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbf5>
   350b5:	mov    edx,0x40
   350ba:	shl    rdx,cl
   350bd:	or     r9,rdx
   350c0:	mov    edx,DWORD PTR [rax]
   350c2:	mov    edi,DWORD PTR [rbx+rdx*4]
   350c5:	test   edi,edi
   350c7:	jns    34ee0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xae0>
   350cd:	mov    edx,0x80
   350d2:	shl    rdx,cl
   350d5:	or     r9,rdx
   350d8:	jmp    34ef5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xaf5>
   350dd:	nop    DWORD PTR [rax]
   350e0:	test   r9,r9
   350e3:	je     34ec0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   350e9:	mov    rsi,r8
   350ec:	not    rsi
   350ef:	nop
   350f0:	tzcnt  rdi,r9
   350f5:	cmp    rdi,0x1f
   350f9:	ja     3652d <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x212d>
   350ff:	mov    r10d,edi
   35102:	shl    r10d,0x5
   35106:	mov    r11,QWORD PTR [rsp+rdi*8+0x60]
   3510b:	add    r11,r8
   3510e:	mov    QWORD PTR [rsp+r10*1+0x268],r11
   35116:	mov    rax,QWORD PTR [rsp+r10*1+0x260]
   3511e:	cmp    rax,QWORD PTR [rsp+0x28]
   35123:	jae    36519 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2119>
   35129:	add    r10,rsp
   3512c:	add    r10,0x260
   35133:	lea    r15,[rax+rax*2]
   35137:	mov    rcx,QWORD PTR [rsp+0x8]
   3513c:	sub    r11,QWORD PTR [rcx+r15*8+0x8]
   35141:	mov    QWORD PTR [rcx+r15*8+0x10],r11
   35146:	mov    r15,QWORD PTR [rsp]
   3514a:	test   r15,r15
   3514d:	je     35170 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd70>
   3514f:	mov    r13,r15
   35152:	mov    r15d,DWORD PTR [r10+0x18]
   35156:	xor    r11d,r11d
   35159:	cmp    r13,0x1
   3515d:	jne    35180 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd80>
   3515f:	mov    rcx,QWORD PTR [rsp+0x10]
   35164:	jmp    351ad <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdad>
   35166:	cs nop WORD PTR [rax+rax*1+0x0]
   35170:	mov    r11,0xffffffffffffffff
   35177:	mov    rcx,QWORD PTR [rsp+0x10]
   3517c:	jmp    351c5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdc5>
   3517e:	xchg   ax,ax
   35180:	mov    rcx,QWORD PTR [rsp+0x10]
   35185:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   35190:	mov    rdx,r11
   35193:	mov    rbx,r13
   35196:	shr    rbx,1
   35199:	add    r11,rbx
   3519c:	cmp    DWORD PTR [rcx+r11*4],r15d
   351a0:	cmova  r11,rdx
   351a4:	sub    r13,rbx
   351a7:	cmp    r13,0x1
   351ab:	ja     35190 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd90>
   351ad:	cmp    DWORD PTR [rcx+r11*4],r15d
   351b1:	mov    rdx,0xffffffffffffffff
   351b8:	cmovne r11,rdx
   351bc:	mov    rbx,QWORD PTR [rsp+0x38]
   351c1:	mov    r15,QWORD PTR [rsp]
   351c5:	cmp    rax,QWORD PTR [rsp+0x20]
   351ca:	jae    36505 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2105>
   351d0:	lea    rdx,[r9-0x1]
   351d4:	and    r9,rdx
   351d7:	mov    rdx,QWORD PTR [rsp+0x18]
   351dc:	mov    QWORD PTR [rdx+rax*8],r11
   351e0:	cmp    rbp,QWORD PTR [rsp+0x40]
   351e5:	je     35705 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1305>
   351eb:	cmp    rbp,QWORD PTR [rsp+0x28]
   351f0:	jae    36551 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2151>
   351f6:	cmp    rbp,r15
   351f9:	jae    3653e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x213e>
   351ff:	lea    rax,[rbp*2+0x0]
   35207:	add    rax,rbp
   3520a:	mov    r13,QWORD PTR [rsp+0x8]
   3520f:	mov    rdx,QWORD PTR [r13+rax*8+0x8]
   35214:	mov    r11d,DWORD PTR [rcx+rbp*4]
   35218:	mov    r11d,DWORD PTR [rbx+r11*4]
   3521c:	mov    BYTE PTR [rdx],r11b
   3521f:	shr    r11d,0x8
   35223:	and    r11d,0x7fffff
   3522a:	lea    rbx,[rdx+0x1]
   3522e:	lea    r15,[rdx+rsi*1]
   35232:	inc    r15
   35235:	add    rdx,QWORD PTR [r13+rax*8+0x0]
   3523a:	mov    QWORD PTR [r10],rbp
   3523d:	mov    QWORD PTR [r10+0x8],rbx
   35241:	mov    rbx,QWORD PTR [rsp+0x38]
   35246:	mov    QWORD PTR [r10+0x10],rdx
   3524a:	mov    DWORD PTR [r10+0x18],r11d
   3524e:	prefetcht0 BYTE PTR [rbx+r11*4]
   35253:	inc    rbp
   35256:	mov    QWORD PTR [rsp+rdi*8+0x60],r15
   3525b:	test   r9,r9
   3525e:	jne    350f0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcf0>
   35264:	jmp    34ec0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   35269:	nop    DWORD PTR [rax+0x0]
   35270:	mov    QWORD PTR [rsp+0x180],r10
   35278:	mov    QWORD PTR [rsp+0x188],r13
   35280:	mov    QWORD PTR [rsp+0x190],rax
   35288:	mov    QWORD PTR [rsp+0x198],rcx
   35290:	mov    QWORD PTR [rsp+0x1a0],rdx
   35298:	mov    QWORD PTR [rsp+0x1a8],rsi
   352a0:	mov    QWORD PTR [rsp+0x1b0],rdi
   352a8:	mov    QWORD PTR [rsp+0x1b8],r8
   352b0:	mov    QWORD PTR [rsp+0x1c0],r9
   352b8:	mov    QWORD PTR [rsp+0x1c8],r11
   352c0:	mov    rcx,QWORD PTR [rsp+0x218]
   352c8:	mov    rdx,QWORD PTR [rsp+0x220]
   352d0:	mov    rsi,QWORD PTR [rsp+0x228]
   352d8:	mov    rdi,QWORD PTR [rsp+0x230]
   352e0:	mov    r8,QWORD PTR [rsp+0x238]
   352e8:	mov    r9,QWORD PTR [rsp+0x240]
   352f0:	mov    r11,QWORD PTR [rsp+0x248]
   352f8:	mov    r13,QWORD PTR [rsp+0x250]
   35300:	mov    r10,QWORD PTR [rsp+0x258]
   35308:	mov    r15,QWORD PTR [rsp+0x18]
   3530d:	mov    rbx,QWORD PTR [rsp+0x20]
   35312:	mov    rax,QWORD PTR [rsp+0x50]
   35317:	add    rax,r14
   3531a:	mov    QWORD PTR [rsp+0x268],rax
   35322:	add    rbx,r14
   35325:	mov    QWORD PTR [rsp+0x288],rbx
   3532d:	mov    rbx,QWORD PTR [rsp]
   35331:	add    rbx,r14
   35334:	mov    QWORD PTR [rsp+0x2a8],rbx
   3533c:	add    r15,r14
   3533f:	mov    QWORD PTR [rsp+0x2c8],r15
   35347:	add    r10,r14
   3534a:	mov    QWORD PTR [rsp+0x2e8],r10
   35352:	add    r13,r14
   35355:	mov    QWORD PTR [rsp+0x308],r13
   3535d:	add    r11,r14
   35360:	mov    QWORD PTR [rsp+0x328],r11
   35368:	add    r9,r14
   3536b:	mov    QWORD PTR [rsp+0x348],r9
   35373:	add    r8,r14
   35376:	mov    QWORD PTR [rsp+0x368],r8
   3537e:	add    rdi,r14
   35381:	mov    QWORD PTR [rsp+0x388],rdi
   35389:	add    rsi,r14
   3538c:	mov    QWORD PTR [rsp+0x3a8],rsi
   35394:	add    rdx,r14
   35397:	mov    QWORD PTR [rsp+0x3c8],rdx
   3539f:	add    rcx,r14
   353a2:	mov    QWORD PTR [rsp+0x3e8],rcx
   353aa:	mov    rax,QWORD PTR [rsp+0x1d8]
   353b2:	add    rax,r14
   353b5:	mov    QWORD PTR [rsp+0x408],rax
   353bd:	mov    rax,QWORD PTR [rsp+0x10]
   353c2:	add    rax,r14
   353c5:	mov    QWORD PTR [rsp+0x428],rax
   353cd:	mov    rax,QWORD PTR [rsp+0x1e0]
   353d5:	add    rax,r14
   353d8:	mov    QWORD PTR [rsp+0x448],rax
   353e0:	mov    rax,QWORD PTR [rsp+0x1e8]
   353e8:	add    rax,r14
   353eb:	mov    QWORD PTR [rsp+0x468],rax
   353f3:	mov    rax,QWORD PTR [rsp+0x1f0]
   353fb:	add    rax,r14
   353fe:	mov    QWORD PTR [rsp+0x488],rax
   35406:	mov    rax,QWORD PTR [rsp+0x1f8]
   3540e:	add    rax,r14
   35411:	mov    QWORD PTR [rsp+0x4a8],rax
   35419:	mov    rax,QWORD PTR [rsp+0x200]
   35421:	add    rax,r14
   35424:	mov    QWORD PTR [rsp+0x4c8],rax
   3542c:	mov    rax,QWORD PTR [rsp+0x208]
   35434:	add    rax,r14
   35437:	mov    QWORD PTR [rsp+0x4e8],rax
   3543f:	mov    rax,QWORD PTR [rsp+0x210]
   35447:	add    rax,r14
   3544a:	mov    QWORD PTR [rsp+0x508],rax
   35452:	mov    rax,QWORD PTR [rsp+0x198]
   3545a:	add    rax,r14
   3545d:	mov    QWORD PTR [rsp+0x528],rax
   35465:	mov    rax,QWORD PTR [rsp+0x1a0]
   3546d:	add    rax,r14
   35470:	mov    QWORD PTR [rsp+0x548],rax
   35478:	mov    rax,QWORD PTR [rsp+0x180]
   35480:	add    rax,r14
   35483:	mov    QWORD PTR [rsp+0x568],rax
   3548b:	mov    rax,QWORD PTR [rsp+0x188]
   35493:	add    rax,r14
   35496:	mov    QWORD PTR [rsp+0x588],rax
   3549e:	mov    rax,QWORD PTR [rsp+0x190]
   354a6:	add    rax,r14
   354a9:	mov    QWORD PTR [rsp+0x5a8],rax
   354b1:	mov    rax,QWORD PTR [rsp+0x1c8]
   354b9:	add    rax,r14
   354bc:	mov    QWORD PTR [rsp+0x5c8],rax
   354c4:	mov    rax,QWORD PTR [rsp+0x1b8]
   354cc:	add    rax,r14
   354cf:	mov    QWORD PTR [rsp+0x5e8],rax
   354d7:	mov    rax,QWORD PTR [rsp+0x1c0]
   354df:	add    rax,r14
   354e2:	mov    QWORD PTR [rsp+0x608],rax
   354ea:	mov    rax,QWORD PTR [rsp+0x1a8]
   354f2:	add    rax,r14
   354f5:	mov    QWORD PTR [rsp+0x628],rax
   354fd:	mov    rax,QWORD PTR [rsp+0x1b0]
   35505:	add    rax,r14
   35508:	mov    QWORD PTR [rsp+0x648],rax
   35510:	mov    rax,QWORD PTR [rsp+0x160]
   35518:	mov    rbx,QWORD PTR [rsp+0x28]
   3551d:	jmp    34b07 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x707>
   35522:	mov    rax,QWORD PTR [rsp+0x60]
   35527:	mov    QWORD PTR [rsp+0x50],rax
   3552c:	mov    rbx,QWORD PTR [rsp+0x68]
   35531:	mov    rax,QWORD PTR [rsp+0x70]
   35536:	mov    QWORD PTR [rsp],rax
   3553a:	mov    r15,QWORD PTR [rsp+0x78]
   3553f:	mov    r10,QWORD PTR [rsp+0x80]
   35547:	mov    r13,QWORD PTR [rsp+0x88]
   3554f:	mov    r11,QWORD PTR [rsp+0x90]
   35557:	mov    r9,QWORD PTR [rsp+0x98]
   3555f:	mov    r8,QWORD PTR [rsp+0xa0]
   35567:	mov    rdi,QWORD PTR [rsp+0xa8]
   3556f:	mov    rsi,QWORD PTR [rsp+0xb0]
   35577:	mov    rdx,QWORD PTR [rsp+0xb8]
   3557f:	mov    rcx,QWORD PTR [rsp+0xc0]
   35587:	mov    rax,QWORD PTR [rsp+0xc8]
   3558f:	mov    QWORD PTR [rsp+0x1d8],rax
   35597:	mov    rax,QWORD PTR [rsp+0xd0]
   3559f:	mov    QWORD PTR [rsp+0x10],rax
   355a4:	mov    rax,QWORD PTR [rsp+0xd8]
   355ac:	mov    QWORD PTR [rsp+0x1e0],rax
   355b4:	mov    rax,QWORD PTR [rsp+0xe0]
   355bc:	mov    QWORD PTR [rsp+0x1e8],rax
   355c4:	mov    rax,QWORD PTR [rsp+0xe8]
   355cc:	mov    QWORD PTR [rsp+0x1f0],rax
   355d4:	mov    rax,QWORD PTR [rsp+0xf0]
   355dc:	mov    QWORD PTR [rsp+0x1f8],rax
   355e4:	mov    rax,QWORD PTR [rsp+0xf8]
   355ec:	mov    QWORD PTR [rsp+0x200],rax
   355f4:	mov    rax,QWORD PTR [rsp+0x100]
   355fc:	mov    QWORD PTR [rsp+0x208],rax
   35604:	mov    rax,QWORD PTR [rsp+0x108]
   3560c:	mov    QWORD PTR [rsp+0x210],rax
   35614:	mov    rax,QWORD PTR [rsp+0x110]
   3561c:	mov    QWORD PTR [rsp+0x198],rax
   35624:	mov    rax,QWORD PTR [rsp+0x118]
   3562c:	mov    QWORD PTR [rsp+0x1a0],rax
   35634:	mov    rax,QWORD PTR [rsp+0x120]
   3563c:	mov    QWORD PTR [rsp+0x180],rax
   35644:	mov    rax,QWORD PTR [rsp+0x128]
   3564c:	mov    QWORD PTR [rsp+0x188],rax
   35654:	mov    rax,QWORD PTR [rsp+0x130]
   3565c:	mov    QWORD PTR [rsp+0x190],rax
   35664:	mov    rax,QWORD PTR [rsp+0x138]
   3566c:	mov    QWORD PTR [rsp+0x1c8],rax
   35674:	mov    rax,QWORD PTR [rsp+0x140]
   3567c:	mov    QWORD PTR [rsp+0x1b8],rax
   35684:	mov    rax,QWORD PTR [rsp+0x148]
   3568c:	mov    QWORD PTR [rsp+0x1c0],rax
   35694:	mov    rax,QWORD PTR [rsp+0x150]
   3569c:	mov    QWORD PTR [rsp+0x1a8],rax
   356a4:	mov    rax,QWORD PTR [rsp+0x158]
   356ac:	mov    QWORD PTR [rsp+0x1b0],rax
   356b4:	jmp    35312 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf12>
   356b9:	mov    rdx,r14
   356bc:	mov    r13,QWORD PTR [rsp+0x40]
   356c1:	mov    QWORD PTR [r12+0x40],rdx
   356c6:	cmp    r13,rdx
   356c9:	jbe    3458d <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d>
   356cf:	lea    rcx,[rip+0x5e532]        # 93c08 <__frame_dummy_init_array_entry+0x920>
   356d6:	xor    edi,edi
   356d8:	mov    rsi,r13
   356db:	call   2bce0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   356e0:	mov    QWORD PTR [rsp],rsi
   356e4:	mov    QWORD PTR [rsp+0x10],r9
   356e9:	mov    rax,QWORD PTR [rsp+0x30]
   356ee:	mov    rdx,QWORD PTR [rax+0x50]
   356f2:	mov    QWORD PTR [rsp+0x18],rdx
   356f7:	mov    rax,QWORD PTR [rax+0x58]
   356fb:	mov    QWORD PTR [rsp+0x20],rax
   35700:	jmp    35df3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19f3>
   35705:	not    r9
   35708:	test   rdi,rdi
   3570b:	je     35b08 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1708>
   35711:	mov    rax,QWORD PTR [rsp+0x60]
   35716:	add    rax,r8
   35719:	inc    rax
   3571c:	mov    QWORD PTR [rsp+0x268],rax
   35724:	cmp    edi,0x1
   35727:	mov    rbp,QWORD PTR [rsp+0x58]
   3572c:	mov    rcx,QWORD PTR [rsp+0x1d0]
   35734:	je     35b2a <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x172a>
   3573a:	bt     r9d,0x1
   3573f:	mov    rax,QWORD PTR [rsp+0x68]
   35744:	adc    rax,r8
   35747:	mov    QWORD PTR [rsp+0x288],rax
   3574f:	cmp    edi,0x2
   35752:	je     35b3f <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x173f>
   35758:	bt     r9d,0x2
   3575d:	mov    rax,QWORD PTR [rsp+0x70]
   35762:	adc    rax,r8
   35765:	mov    QWORD PTR [rsp+0x2a8],rax
   3576d:	cmp    edi,0x3
   35770:	je     35b54 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1754>
   35776:	bt     r9d,0x3
   3577b:	mov    rax,QWORD PTR [rsp+0x78]
   35780:	adc    rax,r8
   35783:	mov    QWORD PTR [rsp+0x2c8],rax
   3578b:	cmp    edi,0x4
   3578e:	je     35b6c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x176c>
   35794:	bt     r9d,0x4
   35799:	mov    rax,QWORD PTR [rsp+0x80]
   357a1:	adc    rax,r8
   357a4:	mov    QWORD PTR [rsp+0x2e8],rax
   357ac:	cmp    edi,0x5
   357af:	je     35b84 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1784>
   357b5:	bt     r9d,0x5
   357ba:	mov    rax,QWORD PTR [rsp+0x88]
   357c2:	adc    rax,r8
   357c5:	mov    QWORD PTR [rsp+0x308],rax
   357cd:	cmp    edi,0x6
   357d0:	je     35b9c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x179c>
   357d6:	bt     r9d,0x6
   357db:	mov    rax,QWORD PTR [rsp+0x90]
   357e3:	adc    rax,r8
   357e6:	mov    QWORD PTR [rsp+0x328],rax
   357ee:	cmp    edi,0x7
   357f1:	je     35bb4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17b4>
   357f7:	bt     r9d,0x7
   357fc:	mov    rax,QWORD PTR [rsp+0x98]
   35804:	adc    rax,r8
   35807:	mov    QWORD PTR [rsp+0x348],rax
   3580f:	cmp    edi,0x8
   35812:	je     35bcc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17cc>
   35818:	bt     r9d,0x8
   3581d:	mov    rax,QWORD PTR [rsp+0xa0]
   35825:	adc    rax,r8
   35828:	mov    QWORD PTR [rsp+0x368],rax
   35830:	cmp    edi,0x9
   35833:	je     35be4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e4>
   35839:	bt     r9d,0x9
   3583e:	mov    rax,QWORD PTR [rsp+0xa8]
   35846:	adc    rax,r8
   35849:	mov    QWORD PTR [rsp+0x388],rax
   35851:	cmp    edi,0xa
   35854:	je     35bfc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17fc>
   3585a:	bt     r9d,0xa
   3585f:	mov    rax,QWORD PTR [rsp+0xb0]
   35867:	adc    rax,r8
   3586a:	mov    QWORD PTR [rsp+0x3a8],rax
   35872:	cmp    edi,0xb
   35875:	je     35c14 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1814>
   3587b:	bt     r9d,0xb
   35880:	mov    rax,QWORD PTR [rsp+0xb8]
   35888:	adc    rax,r8
   3588b:	mov    QWORD PTR [rsp+0x3c8],rax
   35893:	cmp    edi,0xc
   35896:	je     35c2c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x182c>
   3589c:	bt     r9d,0xc
   358a1:	mov    rax,QWORD PTR [rsp+0xc0]
   358a9:	adc    rax,r8
   358ac:	mov    QWORD PTR [rsp+0x3e8],rax
   358b4:	cmp    edi,0xd
   358b7:	je     35c44 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1844>
   358bd:	bt     r9d,0xd
   358c2:	mov    rax,QWORD PTR [rsp+0xc8]
   358ca:	adc    rax,r8
   358cd:	mov    QWORD PTR [rsp+0x408],rax
   358d5:	cmp    edi,0xe
   358d8:	je     35c5c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x185c>
   358de:	bt     r9d,0xe
   358e3:	mov    rax,QWORD PTR [rsp+0xd0]
   358eb:	adc    rax,r8
   358ee:	mov    QWORD PTR [rsp+0x428],rax
   358f6:	cmp    edi,0xf
   358f9:	je     35c74 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1874>
   358ff:	bt     r9d,0xf
   35904:	mov    rax,QWORD PTR [rsp+0xd8]
   3590c:	adc    rax,r8
   3590f:	mov    QWORD PTR [rsp+0x448],rax
   35917:	cmp    edi,0x10
   3591a:	je     35c8c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x188c>
   35920:	bt     r9d,0x10
   35925:	mov    rax,QWORD PTR [rsp+0xe0]
   3592d:	adc    rax,r8
   35930:	mov    QWORD PTR [rsp+0x468],rax
   35938:	cmp    edi,0x11
   3593b:	je     35ca4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18a4>
   35941:	bt     r9d,0x11
   35946:	mov    rax,QWORD PTR [rsp+0xe8]
   3594e:	adc    rax,r8
   35951:	mov    QWORD PTR [rsp+0x488],rax
   35959:	cmp    edi,0x12
   3595c:	je     35cbc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18bc>
   35962:	bt     r9d,0x12
   35967:	mov    rax,QWORD PTR [rsp+0xf0]
   3596f:	adc    rax,r8
   35972:	mov    QWORD PTR [rsp+0x4a8],rax
   3597a:	cmp    edi,0x13
   3597d:	je     35cd4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d4>
   35983:	bt     r9d,0x13
   35988:	mov    rax,QWORD PTR [rsp+0xf8]
   35990:	adc    rax,r8
   35993:	mov    QWORD PTR [rsp+0x4c8],rax
   3599b:	cmp    edi,0x14
   3599e:	je     35cec <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ec>
   359a4:	bt     r9d,0x14
   359a9:	mov    rax,QWORD PTR [rsp+0x100]
   359b1:	adc    rax,r8
   359b4:	mov    QWORD PTR [rsp+0x4e8],rax
   359bc:	cmp    edi,0x15
   359bf:	je     35d04 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1904>
   359c5:	bt     r9d,0x15
   359ca:	mov    rax,QWORD PTR [rsp+0x108]
   359d2:	adc    rax,r8
   359d5:	mov    QWORD PTR [rsp+0x508],rax
   359dd:	cmp    edi,0x16
   359e0:	je     35d1c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191c>
   359e6:	bt     r9d,0x16
   359eb:	mov    rax,QWORD PTR [rsp+0x110]
   359f3:	adc    rax,r8
   359f6:	mov    QWORD PTR [rsp+0x528],rax
   359fe:	cmp    edi,0x17
   35a01:	je     35d34 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1934>
   35a07:	bt     r9d,0x17
   35a0c:	mov    rax,QWORD PTR [rsp+0x118]
   35a14:	adc    rax,r8
   35a17:	mov    QWORD PTR [rsp+0x548],rax
   35a1f:	cmp    edi,0x18
   35a22:	je     35d4c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x194c>
   35a28:	bt     r9d,0x18
   35a2d:	mov    rax,QWORD PTR [rsp+0x120]
   35a35:	adc    rax,r8
   35a38:	mov    QWORD PTR [rsp+0x568],rax
   35a40:	cmp    edi,0x19
   35a43:	je     35d64 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1964>
   35a49:	bt     r9d,0x19
   35a4e:	mov    rax,QWORD PTR [rsp+0x128]
   35a56:	adc    rax,r8
   35a59:	mov    QWORD PTR [rsp+0x588],rax
   35a61:	cmp    edi,0x1a
   35a64:	je     35d7c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x197c>
   35a6a:	bt     r9d,0x1a
   35a6f:	mov    rax,QWORD PTR [rsp+0x130]
   35a77:	adc    rax,r8
   35a7a:	mov    QWORD PTR [rsp+0x5a8],rax
   35a82:	cmp    edi,0x1b
   35a85:	je     35d94 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1994>
   35a8b:	bt     r9d,0x1b
   35a90:	mov    rax,QWORD PTR [rsp+0x138]
   35a98:	adc    rax,r8
   35a9b:	mov    QWORD PTR [rsp+0x5c8],rax
   35aa3:	cmp    edi,0x1c
   35aa6:	je     35dac <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19ac>
   35aac:	bt     r9d,0x1c
   35ab1:	mov    rax,QWORD PTR [rsp+0x140]
   35ab9:	adc    rax,r8
   35abc:	mov    QWORD PTR [rsp+0x5e8],rax
   35ac4:	cmp    edi,0x1d
   35ac7:	je     35dc4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19c4>
   35acd:	bt     r9d,0x1d
   35ad2:	mov    rax,QWORD PTR [rsp+0x148]
   35ada:	adc    rax,r8
   35add:	mov    QWORD PTR [rsp+0x608],rax
   35ae5:	cmp    edi,0x1e
   35ae8:	je     35ddc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19dc>
   35aee:	bt     r9d,0x1e
   35af3:	adc    r8,QWORD PTR [rsp+0x150]
   35afb:	mov    QWORD PTR [rsp+0x628],r8
   35b03:	jmp    35df3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19f3>
   35b08:	bt     r9d,0x1
   35b0d:	mov    rax,QWORD PTR [rsp+0x68]
   35b12:	adc    rax,r8
   35b15:	mov    QWORD PTR [rsp+0x288],rax
   35b1d:	mov    rbp,QWORD PTR [rsp+0x58]
   35b22:	mov    rcx,QWORD PTR [rsp+0x1d0]
   35b2a:	bt     r9d,0x2
   35b2f:	mov    rax,QWORD PTR [rsp+0x70]
   35b34:	adc    rax,r8
   35b37:	mov    QWORD PTR [rsp+0x2a8],rax
   35b3f:	bt     r9d,0x3
   35b44:	mov    rax,QWORD PTR [rsp+0x78]
   35b49:	adc    rax,r8
   35b4c:	mov    QWORD PTR [rsp+0x2c8],rax
   35b54:	bt     r9d,0x4
   35b59:	mov    rax,QWORD PTR [rsp+0x80]
   35b61:	adc    rax,r8
   35b64:	mov    QWORD PTR [rsp+0x2e8],rax
   35b6c:	bt     r9d,0x5
   35b71:	mov    rax,QWORD PTR [rsp+0x88]
   35b79:	adc    rax,r8
   35b7c:	mov    QWORD PTR [rsp+0x308],rax
   35b84:	bt     r9d,0x6
   35b89:	mov    rax,QWORD PTR [rsp+0x90]
   35b91:	adc    rax,r8
   35b94:	mov    QWORD PTR [rsp+0x328],rax
   35b9c:	bt     r9d,0x7
   35ba1:	mov    rax,QWORD PTR [rsp+0x98]
   35ba9:	adc    rax,r8
   35bac:	mov    QWORD PTR [rsp+0x348],rax
   35bb4:	bt     r9d,0x8
   35bb9:	mov    rax,QWORD PTR [rsp+0xa0]
   35bc1:	adc    rax,r8
   35bc4:	mov    QWORD PTR [rsp+0x368],rax
   35bcc:	bt     r9d,0x9
   35bd1:	mov    rax,QWORD PTR [rsp+0xa8]
   35bd9:	adc    rax,r8
   35bdc:	mov    QWORD PTR [rsp+0x388],rax
   35be4:	bt     r9d,0xa
   35be9:	mov    rax,QWORD PTR [rsp+0xb0]
   35bf1:	adc    rax,r8
   35bf4:	mov    QWORD PTR [rsp+0x3a8],rax
   35bfc:	bt     r9d,0xb
   35c01:	mov    rax,QWORD PTR [rsp+0xb8]
   35c09:	adc    rax,r8
   35c0c:	mov    QWORD PTR [rsp+0x3c8],rax
   35c14:	bt     r9d,0xc
   35c19:	mov    rax,QWORD PTR [rsp+0xc0]
   35c21:	adc    rax,r8
   35c24:	mov    QWORD PTR [rsp+0x3e8],rax
   35c2c:	bt     r9d,0xd
   35c31:	mov    rax,QWORD PTR [rsp+0xc8]
   35c39:	adc    rax,r8
   35c3c:	mov    QWORD PTR [rsp+0x408],rax
   35c44:	bt     r9d,0xe
   35c49:	mov    rax,QWORD PTR [rsp+0xd0]
   35c51:	adc    rax,r8
   35c54:	mov    QWORD PTR [rsp+0x428],rax
   35c5c:	bt     r9d,0xf
   35c61:	mov    rax,QWORD PTR [rsp+0xd8]
   35c69:	adc    rax,r8
   35c6c:	mov    QWORD PTR [rsp+0x448],rax
   35c74:	bt     r9d,0x10
   35c79:	mov    rax,QWORD PTR [rsp+0xe0]
   35c81:	adc    rax,r8
   35c84:	mov    QWORD PTR [rsp+0x468],rax
   35c8c:	bt     r9d,0x11
   35c91:	mov    rax,QWORD PTR [rsp+0xe8]
   35c99:	adc    rax,r8
   35c9c:	mov    QWORD PTR [rsp+0x488],rax
   35ca4:	bt     r9d,0x12
   35ca9:	mov    rax,QWORD PTR [rsp+0xf0]
   35cb1:	adc    rax,r8
   35cb4:	mov    QWORD PTR [rsp+0x4a8],rax
   35cbc:	bt     r9d,0x13
   35cc1:	mov    rax,QWORD PTR [rsp+0xf8]
   35cc9:	adc    rax,r8
   35ccc:	mov    QWORD PTR [rsp+0x4c8],rax
   35cd4:	bt     r9d,0x14
   35cd9:	mov    rax,QWORD PTR [rsp+0x100]
   35ce1:	adc    rax,r8
   35ce4:	mov    QWORD PTR [rsp+0x4e8],rax
   35cec:	bt     r9d,0x15
   35cf1:	mov    rax,QWORD PTR [rsp+0x108]
   35cf9:	adc    rax,r8
   35cfc:	mov    QWORD PTR [rsp+0x508],rax
   35d04:	bt     r9d,0x16
   35d09:	mov    rax,QWORD PTR [rsp+0x110]
   35d11:	adc    rax,r8
   35d14:	mov    QWORD PTR [rsp+0x528],rax
   35d1c:	bt     r9d,0x17
   35d21:	mov    rax,QWORD PTR [rsp+0x118]
   35d29:	adc    rax,r8
   35d2c:	mov    QWORD PTR [rsp+0x548],rax
   35d34:	bt     r9d,0x18
   35d39:	mov    rax,QWORD PTR [rsp+0x120]
   35d41:	adc    rax,r8
   35d44:	mov    QWORD PTR [rsp+0x568],rax
   35d4c:	bt     r9d,0x19
   35d51:	mov    rax,QWORD PTR [rsp+0x128]
   35d59:	adc    rax,r8
   35d5c:	mov    QWORD PTR [rsp+0x588],rax
   35d64:	bt     r9d,0x1a
   35d69:	mov    rax,QWORD PTR [rsp+0x130]
   35d71:	adc    rax,r8
   35d74:	mov    QWORD PTR [rsp+0x5a8],rax
   35d7c:	bt     r9d,0x1b
   35d81:	mov    rax,QWORD PTR [rsp+0x138]
   35d89:	adc    rax,r8
   35d8c:	mov    QWORD PTR [rsp+0x5c8],rax
   35d94:	bt     r9d,0x1c
   35d99:	mov    rax,QWORD PTR [rsp+0x140]
   35da1:	adc    rax,r8
   35da4:	mov    QWORD PTR [rsp+0x5e8],rax
   35dac:	bt     r9d,0x1d
   35db1:	mov    rax,QWORD PTR [rsp+0x148]
   35db9:	adc    rax,r8
   35dbc:	mov    QWORD PTR [rsp+0x608],rax
   35dc4:	bt     r9d,0x1e
   35dc9:	mov    rax,QWORD PTR [rsp+0x150]
   35dd1:	adc    rax,r8
   35dd4:	mov    QWORD PTR [rsp+0x628],rax
   35ddc:	shr    r9d,0x1f
   35de0:	add    r8,QWORD PTR [rsp+0x158]
   35de8:	add    r8,r9
   35deb:	mov    QWORD PTR [rsp+0x648],r8
   35df3:	shl    rcx,0x5
   35df7:	lea    r15,[rsp+rcx*1]
   35dfb:	add    r15,0x260
   35e02:	lea    r14,[rsp+0x260]
   35e0a:	mov    rax,QWORD PTR [rsp+0x18]
   35e0f:	jmp    35e2d <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a2d>
   35e11:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35e20:	mov    r14,r13
   35e23:	add    r12,0xffffffffffffffe0
   35e27:	je     35fab <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bab>
   35e2d:	lea    r13,[r14+0x20]
   35e31:	mov    rbx,QWORD PTR [r14]
   35e34:	cmp    rbx,0xffffffffffffffff
   35e38:	je     35e20 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a20>
   35e3a:	cmp    rbx,QWORD PTR [rsp+0x20]
   35e3f:	jae    36565 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2165>
   35e45:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   35e4a:	jne    35e20 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a20>
   35e4c:	mov    edi,0x80
   35e51:	call   QWORD PTR [rip+0x601c1]        # 96018 <malloc@GLIBC_2.2.5>
   35e57:	test   rax,rax
   35e5a:	je     365b4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21b4>
   35e60:	mov    rcx,QWORD PTR [r14+0x18]
   35e64:	mov    QWORD PTR [rax+0x18],rcx
   35e68:	movups xmm0,XMMWORD PTR [r14+0x8]
   35e6d:	movups XMMWORD PTR [rax+0x8],xmm0
   35e71:	mov    QWORD PTR [rax],rbx
   35e74:	mov    QWORD PTR [rsp+0x168],0x4
   35e80:	mov    QWORD PTR [rsp+0x170],rax
   35e88:	mov    QWORD PTR [rsp+0x178],0x1
   35e94:	mov    r12d,0x1
   35e9a:	cmp    r13,r15
   35e9d:	je     35f67 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b67>
   35ea3:	lea    r14,[rsp+0x168]
   35eab:	mov    rdx,QWORD PTR [rsp+0x30]
   35eb0:	mov    rcx,QWORD PTR [rdx+0x50]
   35eb4:	mov    rsi,QWORD PTR [rdx+0x58]
   35eb8:	mov    rdx,r13
   35ebb:	jmp    35ecc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1acc>
   35ebd:	nop    DWORD PTR [rax]
   35ec0:	mov    rdx,r13
   35ec3:	cmp    r13,r15
   35ec6:	je     35f67 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b67>
   35ecc:	lea    r13,[rdx+0x20]
   35ed0:	mov    rbx,QWORD PTR [rdx]
   35ed3:	cmp    rbx,0xffffffffffffffff
   35ed7:	je     35ec0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ac0>
   35ed9:	cmp    rbx,rsi
   35edc:	jae    36588 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2188>
   35ee2:	cmp    QWORD PTR [rcx+rbx*8],0xffffffffffffffff
   35ee7:	jne    35ec0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ac0>
   35ee9:	mov    rcx,QWORD PTR [rdx+0x18]
   35eed:	mov    QWORD PTR [rsp+0x670],rcx
   35ef5:	movups xmm0,XMMWORD PTR [rdx+0x8]
   35ef9:	movaps XMMWORD PTR [rsp+0x660],xmm0
   35f01:	cmp    r12,QWORD PTR [rsp+0x168]
   35f09:	jne    35f2e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b2e>
   35f0b:	mov    edx,0x1
   35f10:	mov    ecx,0x8
   35f15:	mov    r8d,0x20
   35f1b:	mov    rdi,r14
   35f1e:	mov    rsi,r12
   35f21:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   35f26:	mov    rax,QWORD PTR [rsp+0x170]
   35f2e:	mov    rcx,r12
   35f31:	shl    rcx,0x5
   35f35:	mov    QWORD PTR [rax+rcx*1],rbx
   35f39:	movaps xmm0,XMMWORD PTR [rsp+0x660]
   35f41:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   35f46:	mov    rdx,QWORD PTR [rsp+0x670]
   35f4e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   35f53:	inc    r12
   35f56:	mov    QWORD PTR [rsp+0x178],r12
   35f5e:	cmp    r13,r15
   35f61:	jne    35eab <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1aab>
   35f67:	mov    QWORD PTR [rsp+0x58],rbp
   35f6c:	mov    rax,QWORD PTR [rsp+0x168]
   35f74:	mov    QWORD PTR [rsp+0x18],rax
   35f79:	mov    rax,QWORD PTR [rsp+0x170]
   35f81:	mov    QWORD PTR [rsp+0x8],rax
   35f86:	mov    r15,r12
   35f89:	shl    r15,0x5
   35f8d:	mov    rax,QWORD PTR [rsp+0x30]
   35f92:	mov    r14,QWORD PTR [rax+0x38]
   35f96:	mov    rbp,QWORD PTR [rax+0x40]
   35f9a:	xor    r13d,r13d
   35f9d:	lea    rax,[rip+0x5de8c]        # 93e30 <__frame_dummy_init_array_entry+0xb48>
   35fa4:	mov    QWORD PTR [rsp+0x40],rax
   35fa9:	jmp    3600e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c0e>
   35fab:	mov    eax,0x8
   35fb0:	mov    QWORD PTR [rsp+0x8],rax
   35fb5:	xor    r14d,r14d
   35fb8:	mov    r9,QWORD PTR [rsp+0x38]
   35fbd:	mov    r8,QWORD PTR [rsp+0x10]
   35fc2:	jmp    36249 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e49>
   35fc7:	mov    rdi,r12
   35fca:	mov    r14,r9
   35fcd:	call   36750 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsdtMXQ4ssD8O_5mibz2>
   35fd2:	mov    r9,r14
   35fd5:	mov    esi,DWORD PTR [rsp+0x4c]
   35fd9:	mov    r14,QWORD PTR [r12+0x8]
   35fde:	jmp    344bd <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbd>
   35fe3:	mov    rax,QWORD PTR [rsp]
   35fe7:	mov    rdx,QWORD PTR [rax]
   35fea:	mov    rax,QWORD PTR [rax+0x8]
   35fee:	xchg   ax,ax
   35ff0:	add    rbx,rax
   35ff3:	mov    rcx,QWORD PTR [rsp+0x8]
   35ff8:	mov    QWORD PTR [rcx+r13*1+0x8],rbx
   35ffd:	add    rdx,rax
   36000:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   36005:	add    r13,0x20
   36009:	cmp    r15,r13
   3600c:	je     36071 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c71>
   3600e:	mov    rax,QWORD PTR [rsp+0x8]
   36013:	mov    rbx,QWORD PTR [rax+r13*1+0x8]
   36018:	cmp    rbx,QWORD PTR [rax+r13*1+0x10]
   3601d:	jne    36005 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c05>
   3601f:	mov    rax,QWORD PTR [rsp+0x8]
   36024:	mov    rdi,QWORD PTR [rax+r13*1]
   36028:	cmp    rdi,rbp
   3602b:	jae    364da <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20da>
   36031:	lea    rcx,[rdi+rdi*2]
   36035:	mov    rdx,QWORD PTR [r14+rcx*8]
   36039:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   3603e:	sub    rbx,rax
   36041:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   36046:	mov    rsi,rdx
   36049:	sub    rsi,rbx
   3604c:	cmp    rdx,rsi
   3604f:	jbe    35ff0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bf0>
   36051:	lea    rdi,[r14+rcx*8]
   36055:	mov    ecx,0x1
   3605a:	mov    r8d,0x1
   36060:	mov    QWORD PTR [rsp],rdi
   36064:	mov    rsi,rbx
   36067:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   3606c:	jmp    35fe3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1be3>
   36071:	mov    rax,QWORD PTR [rsp+0x30]
   36076:	mov    rcx,QWORD PTR [rax+0x50]
   3607a:	mov    QWORD PTR [rsp+0x20],rcx
   3607f:	mov    rcx,QWORD PTR [rax+0x58]
   36083:	mov    QWORD PTR [rsp+0x28],rcx
   36088:	mov    rcx,QWORD PTR [rax+0x8]
   3608c:	mov    QWORD PTR [rsp+0x10],rcx
   36091:	mov    rax,QWORD PTR [rax+0x10]
   36095:	mov    QWORD PTR [rsp],rax
   36099:	xor    r15d,r15d
   3609c:	mov    r9,QWORD PTR [rsp+0x38]
   360a1:	jmp    360b3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   360a3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   360b0:	xor    r15d,r15d
   360b3:	mov    r13,r15
   360b6:	shl    r13,0x5
   360ba:	add    r13,QWORD PTR [rsp+0x8]
   360bf:	jmp    360ef <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cef>
   360c1:	mov    rax,QWORD PTR [rsp+0x50]
   360c6:	mov    rdx,QWORD PTR [rax]
   360c9:	mov    rax,QWORD PTR [rax+0x8]
   360cd:	nop    DWORD PTR [rax]
   360d0:	add    rbx,rax
   360d3:	mov    QWORD PTR [r13+0x8],rbx
   360d7:	add    rdx,rax
   360da:	mov    QWORD PTR [r13+0x10],rdx
   360de:	inc    r15
   360e1:	add    r13,0x20
   360e5:	cmp    r15,r12
   360e8:	mov    r9,QWORD PTR [rsp+0x38]
   360ed:	jae    360b0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb0>
   360ef:	mov    eax,DWORD PTR [r13+0x18]
   360f3:	mov    ecx,DWORD PTR [r9+rax*4]
   360f7:	test   ecx,ecx
   360f9:	js     36170 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d70>
   360fb:	mov    eax,ecx
   360fd:	shr    eax,0x8
   36100:	mov    DWORD PTR [r13+0x18],eax
   36104:	prefetcht0 BYTE PTR [r9+rax*4]
   36109:	mov    rax,QWORD PTR [r13+0x8]
   3610d:	mov    BYTE PTR [rax],cl
   3610f:	mov    rbx,QWORD PTR [r13+0x8]
   36113:	inc    rbx
   36116:	mov    QWORD PTR [r13+0x8],rbx
   3611a:	cmp    rbx,QWORD PTR [r13+0x10]
   3611e:	jne    360de <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cde>
   36120:	mov    rdi,QWORD PTR [r13+0x0]
   36124:	cmp    rdi,rbp
   36127:	jae    364da <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20da>
   3612d:	lea    rcx,[rdi+rdi*2]
   36131:	mov    rdx,QWORD PTR [r14+rcx*8]
   36135:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   3613a:	sub    rbx,rax
   3613d:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   36142:	mov    rsi,rdx
   36145:	sub    rsi,rbx
   36148:	cmp    rdx,rsi
   3614b:	jbe    360d0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cd0>
   3614d:	lea    rdi,[r14+rcx*8]
   36151:	mov    ecx,0x1
   36156:	mov    r8d,0x1
   3615c:	mov    QWORD PTR [rsp+0x50],rdi
   36161:	mov    rsi,rbx
   36164:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   36169:	jmp    360c1 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cc1>
   3616e:	xchg   ax,ax
   36170:	mov    rdi,QWORD PTR [r13+0x0]
   36174:	cmp    rdi,rbp
   36177:	jae    364c0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20c0>
   3617d:	lea    rcx,[rdi+rdi*2]
   36181:	mov    rdx,QWORD PTR [r13+0x8]
   36185:	sub    rdx,QWORD PTR [r14+rcx*8+0x8]
   3618a:	mov    QWORD PTR [r14+rcx*8+0x10],rdx
   3618f:	mov    rdx,QWORD PTR [rsp]
   36193:	test   rdx,rdx
   36196:	je     361a7 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1da7>
   36198:	xor    ecx,ecx
   3619a:	cmp    rdx,0x1
   3619e:	jne    361b0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1db0>
   361a0:	mov    r10,QWORD PTR [rsp+0x10]
   361a5:	jmp    361dd <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ddd>
   361a7:	mov    rcx,0xffffffffffffffff
   361ae:	jmp    361ec <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dec>
   361b0:	mov    r10,QWORD PTR [rsp+0x10]
   361b5:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   361c0:	mov    rsi,rcx
   361c3:	mov    r8,rdx
   361c6:	shr    r8,1
   361c9:	add    rcx,r8
   361cc:	cmp    DWORD PTR [r10+rcx*4],eax
   361d0:	cmova  rcx,rsi
   361d4:	sub    rdx,r8
   361d7:	cmp    rdx,0x1
   361db:	ja     361c0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dc0>
   361dd:	cmp    DWORD PTR [r10+rcx*4],eax
   361e1:	mov    rax,0xffffffffffffffff
   361e8:	cmovne rcx,rax
   361ec:	cmp    rdi,QWORD PTR [rsp+0x28]
   361f1:	jae    364c9 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20c9>
   361f7:	mov    rax,QWORD PTR [rsp+0x20]
   361fc:	mov    QWORD PTR [rax+rdi*8],rcx
   36200:	dec    r12
   36203:	mov    rax,r12
   36206:	shl    rax,0x5
   3620a:	mov    rcx,QWORD PTR [rsp+0x8]
   3620f:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   36213:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   36218:	movups XMMWORD PTR [r13+0x10],xmm1
   3621d:	movups XMMWORD PTR [r13+0x0],xmm0
   36222:	cmp    r15,r12
   36225:	jb     360b3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   3622b:	mov    r15d,0x0
   36231:	test   r12,r12
   36234:	jne    360b3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   3623a:	mov    rbp,QWORD PTR [rsp+0x58]
   3623f:	mov    r8,QWORD PTR [rsp+0x10]
   36244:	mov    r14,QWORD PTR [rsp+0x18]
   36249:	mov    rax,0xffffffffffffffff
   36250:	mov    r10,QWORD PTR [rsp]
   36254:	test   r10,r10
   36257:	mov    rsi,QWORD PTR [rsp+0x30]
   3625c:	je     362dc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1edc>
   3625e:	lea    rcx,[r10*4+0x0]
   36266:	xor    edx,edx
   36268:	nop    DWORD PTR [rax+rax*1+0x0]
   36270:	mov    edi,DWORD PTR [r8+rdx*1]
   36274:	cmp    rbp,rdi
   36277:	jbe    364f1 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20f1>
   3627d:	and    BYTE PTR [r9+rdi*4+0x3],0x7f
   36283:	add    rdx,0x4
   36287:	cmp    rcx,rdx
   3628a:	jne    36270 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e70>
   3628c:	test   r10,r10
   3628f:	je     362dc <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1edc>
   36291:	xor    ecx,ecx
   36293:	cmp    r10,0x1
   36297:	jne    3629f <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e9f>
   36299:	mov    edi,DWORD PTR [rsp+0x4c]
   3629d:	jmp    362cd <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ecd>
   3629f:	mov    edi,DWORD PTR [rsp+0x4c]
   362a3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   362b0:	mov    rax,rcx
   362b3:	mov    rdx,r10
   362b6:	shr    rdx,1
   362b9:	add    rcx,rdx
   362bc:	cmp    DWORD PTR [r8+rcx*4],edi
   362c0:	cmova  rcx,rax
   362c4:	sub    r10,rdx
   362c7:	cmp    r10,0x1
   362cb:	ja     362b0 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eb0>
   362cd:	cmp    DWORD PTR [r8+rcx*4],edi
   362d1:	mov    rax,0xffffffffffffffff
   362d8:	cmove  rax,rcx
   362dc:	mov    QWORD PTR [rsi+0x68],rax
   362e0:	mov    QWORD PTR [rsi+0x60],rbp
   362e4:	test   r14,r14
   362e7:	je     362f4 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ef4>
   362e9:	mov    rdi,QWORD PTR [rsp+0x8]
   362ee:	call   QWORD PTR [rip+0x5fd2c]        # 96020 <free@GLIBC_2.2.5>
   362f4:	add    rsp,0x678
   362fb:	pop    rbx
   362fc:	pop    r12
   362fe:	pop    r13
   36300:	pop    r14
   36302:	pop    r15
   36304:	pop    rbp
   36305:	ret
   36306:	mov    edx,0x1ff
   3630b:	mov    ecx,0x4
   36310:	mov    r8d,0x4
   36316:	mov    rbx,QWORD PTR [rsp+0x30]
   3631b:	mov    r14d,esi
   3631e:	mov    rdi,rbx
   36321:	xor    esi,esi
   36323:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   36328:	mov    esi,r14d
   3632b:	mov    r9,QWORD PTR [rbx+0x10]
   3632f:	jmp    34451 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x51>
   36334:	mov    rsi,r13
   36337:	cmp    r13,0x15
   3633b:	jae    365c3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21c3>
   36341:	lea    rax,[r14+rsi*4]
   36345:	lea    rcx,[r14+0x4]
   36349:	mov    edx,0x4
   3634e:	jmp    36366 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f66>
   36350:	mov    rdi,r14
   36353:	mov    DWORD PTR [rdi],esi
   36355:	add    rcx,0x4
   36359:	add    rdx,0x4
   3635d:	cmp    rcx,rax
   36360:	je     365d1 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21d1>
   36366:	mov    r8d,DWORD PTR [rcx-0x4]
   3636a:	mov    esi,DWORD PTR [rcx]
   3636c:	cmp    esi,r8d
   3636f:	jae    36355 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f55>
   36371:	mov    rdi,rdx
   36374:	mov    DWORD PTR [r14+rdi*1],r8d
   36378:	cmp    rdi,0x4
   3637c:	je     36350 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f50>
   3637e:	mov    r8d,DWORD PTR [r14+rdi*1-0x8]
   36383:	add    rdi,0xfffffffffffffffc
   36387:	cmp    esi,r8d
   3638a:	jb     36374 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f74>
   3638c:	add    rdi,r14
   3638f:	jmp    36353 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f53>
   36391:	lea    rdi,[r12+0x48]
   36396:	mov    ecx,0x8
   3639b:	mov    r8d,0x8
   363a1:	xor    esi,esi
   363a3:	mov    rdx,r13
   363a6:	mov    rbx,rax
   363a9:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   363ae:	mov    rax,rbx
   363b1:	mov    rbx,QWORD PTR [r12+0x58]
   363b6:	mov    r14,QWORD PTR [r12+0x50]
   363bb:	lea    rdi,[r14+rbx*8]
   363bf:	cmp    r13,0x1
   363c3:	jne    34620 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x220>
   363c9:	jmp    3464b <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x24b>
   363ce:	lea    rdi,[r12+0x30]
   363d3:	mov    ecx,0x8
   363d8:	mov    r8d,0x18
   363de:	mov    rsi,r13
   363e1:	mov    rdx,r14
   363e4:	mov    rbx,rax
   363e7:	call   366d0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsdtMXQ4ssD8O_5mibz2>
   363ec:	mov    rax,rbx
   363ef:	mov    rdx,QWORD PTR [r12+0x40]
   363f4:	mov    rsi,QWORD PTR [r12+0x38]
   363f9:	inc    r13
   363fc:	test   r14b,0x1
   36400:	je     3456e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16e>
   36406:	dec    r14
   36409:	lea    rcx,[rdx+rdx*2]
   3640d:	mov    QWORD PTR [rsi+rcx*8],0x0
   36415:	mov    QWORD PTR [rsi+rcx*8+0x8],0x1
   3641e:	mov    QWORD PTR [rsi+rcx*8+0x10],0x0
   36427:	inc    rdx
   3642a:	mov    rcx,QWORD PTR [rsp+0x40]
   3642f:	cmp    rcx,r13
   36432:	mov    r13,rcx
   36435:	je     3457f <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17f>
   3643b:	lea    rcx,[rdx+rdx*2]
   3643f:	lea    rsi,[rsi+rcx*8]
   36443:	add    rsi,0x28
   36447:	neg    r14
   3644a:	xor    ecx,ecx
   3644c:	xorps  xmm0,xmm0
   3644f:	nop
   36450:	mov    QWORD PTR [rsi-0x28],0x0
   36458:	mov    QWORD PTR [rsi-0x20],0x1
   36460:	movups XMMWORD PTR [rsi-0x18],xmm0
   36464:	mov    QWORD PTR [rsi-0x8],0x1
   3646c:	mov    QWORD PTR [rsi],0x0
   36473:	add    rsi,0x30
   36477:	add    rcx,0xfffffffffffffffe
   3647b:	cmp    r14,rcx
   3647e:	jne    36450 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2050>
   36480:	sub    rdx,rcx
   36483:	mov    QWORD PTR [r12+0x40],rdx
   36488:	cmp    r13,rdx
   3648b:	jbe    3458d <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d>
   36491:	jmp    356cf <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12cf>
   36496:	lea    rdx,[rip+0x5d70b]        # 93ba8 <__frame_dummy_init_array_entry+0x8c0>
   3649d:	mov    rsi,rbp
   364a0:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364a5:	lea    rdx,[rip+0x5d9b4]        # 93e60 <__frame_dummy_init_array_entry+0xb78>
   364ac:	mov    rsi,rbx
   364af:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364b4:	lea    rdx,[rip+0x5d9bd]        # 93e78 <__frame_dummy_init_array_entry+0xb90>
   364bb:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364c0:	lea    rax,[rip+0x5d939]        # 93e00 <__frame_dummy_init_array_entry+0xb18>
   364c7:	jmp    364d5 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20d5>
   364c9:	mov    rbp,QWORD PTR [rsp+0x28]
   364ce:	lea    rax,[rip+0x5d943]        # 93e18 <__frame_dummy_init_array_entry+0xb30>
   364d5:	mov    QWORD PTR [rsp+0x40],rax
   364da:	mov    rsi,rbp
   364dd:	mov    rdx,QWORD PTR [rsp+0x40]
   364e2:	mov    r14,QWORD PTR [rsp+0x18]
   364e7:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364ec:	jmp    36597 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2197>
   364f1:	lea    rdx,[rip+0x5d6f8]        # 93bf0 <__frame_dummy_init_array_entry+0x908>
   364f8:	mov    rsi,rbp
   364fb:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36500:	jmp    36597 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2197>
   36505:	lea    rdx,[rip+0x5d90c]        # 93e18 <__frame_dummy_init_array_entry+0xb30>
   3650c:	mov    rdi,rax
   3650f:	mov    rsi,QWORD PTR [rsp+0x20]
   36514:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36519:	lea    rdx,[rip+0x5d8e0]        # 93e00 <__frame_dummy_init_array_entry+0xb18>
   36520:	mov    rdi,rax
   36523:	mov    rsi,QWORD PTR [rsp+0x28]
   36528:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3652d:	lea    rdx,[rip+0x5d6a4]        # 93bd8 <__frame_dummy_init_array_entry+0x8f0>
   36534:	mov    esi,0x20
   36539:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3653e:	lea    rdx,[rip+0x5d933]        # 93e78 <__frame_dummy_init_array_entry+0xb90>
   36545:	mov    rdi,rbp
   36548:	mov    rsi,QWORD PTR [rsp]
   3654c:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36551:	lea    rdx,[rip+0x5d908]        # 93e60 <__frame_dummy_init_array_entry+0xb78>
   36558:	mov    rdi,rbp
   3655b:	mov    rsi,QWORD PTR [rsp+0x28]
   36560:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36565:	lea    rdx,[rip+0x5d8dc]        # 93e48 <__frame_dummy_init_array_entry+0xb60>
   3656c:	mov    rdi,rbx
   3656f:	mov    rsi,QWORD PTR [rsp+0x20]
   36574:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36579:	lea    rdx,[rip+0x5d8b0]        # 93e30 <__frame_dummy_init_array_entry+0xb48>
   36580:	mov    rsi,rbx
   36583:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36588:	lea    rdx,[rip+0x5d8b9]        # 93e48 <__frame_dummy_init_array_entry+0xb60>
   3658f:	mov    rdi,rbx
   36592:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36597:	ud2
   36599:	lea    rdx,[rip+0x5d5f0]        # 93b90 <__frame_dummy_init_array_entry+0x8a8>
   365a0:	mov    rsi,rbp
   365a3:	call   2afab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   365a8:	lea    rdi,[rip+0x5d611]        # 93bc0 <__frame_dummy_init_array_entry+0x8d8>
   365af:	call   2d9a0 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   365b4:	mov    edi,0x8
   365b9:	mov    esi,0x80
   365be:	call   2980b <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   365c3:	mov    rdi,r14
   365c6:	mov    r15,r9
   365c9:	call   367c0 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECsdtMXQ4ssD8O_5mibz2>
   365ce:	mov    r9,r15
   365d1:	xor    ecx,ecx
   365d3:	mov    eax,DWORD PTR [r14+rcx*4+0x4]
   365d8:	cmp    eax,DWORD PTR [r14+rcx*4]
   365dc:	je     365eb <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21eb>
   365de:	inc    rcx
   365e1:	cmp    rbx,rcx
   365e4:	jne    365d3 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21d3>
   365e6:	jmp    344d7 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7>
   365eb:	lea    rdx,[rcx+0x1]
   365ef:	lea    rax,[rcx+0x2]
   365f3:	cmp    rax,r13
   365f6:	jae    3660c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x220c>
   365f8:	mov    esi,r9d
   365fb:	sub    esi,ecx
   365fd:	add    r9,0x1fd
   36604:	test   sil,0x1
   36608:	jne    36611 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2211>
   3660a:	jmp    36635 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2235>
   3660c:	mov    r13,rdx
   3660f:	jmp    3663a <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x223a>
   36611:	mov    eax,DWORD PTR [r14+rcx*4+0x8]
   36616:	mov    rsi,rdx
   36619:	cmp    eax,DWORD PTR [r14+rcx*4]
   3661d:	je     36628 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2228>
   3661f:	mov    DWORD PTR [r14+rcx*4+0x4],eax
   36624:	lea    rsi,[rdx+0x1]
   36628:	add    rdx,0x2
   3662c:	mov    rax,rdx
   3662f:	mov    r13,rsi
   36632:	mov    rdx,rsi
   36635:	cmp    r9,rcx
   36638:	jne    36644 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2244>
   3663a:	mov    QWORD PTR [r12+0x10],r13
   3663f:	jmp    344d7 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7>
   36644:	inc    rbx
   36647:	mov    r13,rdx
   3664a:	jmp    36655 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2255>
   3664c:	add    rax,0x2
   36650:	cmp    rbx,rax
   36653:	je     3663a <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x223a>
   36655:	mov    ecx,DWORD PTR [r14+rax*4]
   36659:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3665e:	jne    3666e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x226e>
   36660:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   36665:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3666a:	je     3664c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   3666c:	jmp    36681 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2281>
   3666e:	mov    DWORD PTR [r14+r13*4],ecx
   36672:	inc    r13
   36675:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   3667a:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3667f:	je     3664c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   36681:	mov    DWORD PTR [r14+r13*4],ecx
   36685:	inc    r13
   36688:	jmp    3664c <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   3668a:	jmp    3668e <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x228e>
   3668c:	jmp    36698 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2298>
   3668e:	mov    rbx,rax
   36691:	mov    r14,QWORD PTR [rsp+0x18]
   36696:	jmp    366b8 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22b8>
   36698:	mov    rbx,rax
   3669b:	cmp    QWORD PTR [rsp+0x168],0x0
   366a4:	je     366c8 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22c8>
   366a6:	mov    rax,QWORD PTR [rsp+0x170]
   366ae:	mov    QWORD PTR [rsp+0x8],rax
   366b3:	jmp    366bd <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22bd>
   366b5:	mov    rbx,rax
   366b8:	test   r14,r14
   366bb:	je     366c8 <_RINvNtCsdtMXQ4ssD8O_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22c8>
   366bd:	mov    rdi,QWORD PTR [rsp+0x8]
   366c2:	call   QWORD PTR [rip+0x5f958]        # 96020 <free@GLIBC_2.2.5>
   366c8:	mov    rdi,rbx
   366cb:	call   922a0 <_Unwind_Resume@plt>
