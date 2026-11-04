
c30cg/walk/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

00000000000343e0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   343e0:	push   rbp
   343e1:	push   r15
   343e3:	push   r14
   343e5:	push   r13
   343e7:	push   r12
   343e9:	push   rbx
   343ea:	sub    rsp,0x678
   343f1:	mov    rbp,rsi
   343f4:	mov    rax,rdi
   343f7:	mov    edi,edx
   343f9:	cmp    rsi,rdi
   343fc:	jbe    36579 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2199>
   34402:	mov    QWORD PTR [rsp+0x38],rax
   34407:	mov    esi,DWORD PTR [rax+rdi*4]
   3440a:	mov    QWORD PTR [rcx+0x10],0x0
   34412:	movabs r12,0x2545f4914f6cdd1d
   3441c:	mov    QWORD PTR [rsp+0x30],rcx
   34421:	cmp    QWORD PTR [rcx],0x1fe
   34428:	jbe    362e6 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f06>
   3442e:	xor    r9d,r9d
   34431:	shr    esi,0x8
   34434:	xor    r12,rbp
   34437:	mov    rax,QWORD PTR [rsp+0x30]
   3443c:	mov    r14,QWORD PTR [rax+0x8]
   34440:	mov    ecx,0x1ff
   34445:	mov    rbx,r9
   34448:	nop    DWORD PTR [rax+rax*1+0x0]
   34450:	mov    r15,rbx
   34453:	mov    rax,r12
   34456:	shl    rax,0xd
   3445a:	xor    rax,r12
   3445d:	mov    rdx,rax
   34460:	shr    rdx,0x7
   34464:	xor    rdx,rax
   34467:	mov    r12,rdx
   3446a:	shl    r12,0x11
   3446e:	xor    r12,rdx
   34471:	mov    rax,r12
   34474:	xor    edx,edx
   34476:	div    rbp
   34479:	mov    DWORD PTR [r14+rbx*4],edx
   3447d:	inc    rbx
   34480:	dec    rcx
   34483:	jne    34450 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x70>
   34485:	mov    r12,QWORD PTR [rsp+0x30]
   3448a:	mov    QWORD PTR [r12+0x10],rbx
   3448f:	cmp    rbx,QWORD PTR [r12]
   34493:	mov    DWORD PTR [rsp+0x4c],esi
   34497:	je     35fa7 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bc7>
   3449d:	mov    DWORD PTR [r14+rbx*4],esi
   344a1:	lea    rax,[r15+0x2]
   344a5:	mov    r13,rax
   344a8:	mov    QWORD PTR [r12+0x10],rax
   344ad:	cmp    r15,0xfffffffffffffffd
   344b1:	jbe    36314 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f34>
   344b7:	test   r13,r13
   344ba:	mov    rdx,QWORD PTR [rsp+0x38]
   344bf:	je     36588 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21a8>
   344c5:	lea    rax,[r13*4+0x0]
   344cd:	xor    ecx,ecx
   344cf:	nop
   344d0:	mov    edi,DWORD PTR [r14+rcx*1]
   344d4:	cmp    rbp,rdi
   344d7:	jbe    36476 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2096>
   344dd:	or     BYTE PTR [rdx+rdi*4+0x3],0x80
   344e2:	add    rcx,0x4
   344e6:	cmp    rax,rcx
   344e9:	jne    344d0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf0>
   344eb:	lea    rax,[rbp*2+0x0]
   344f3:	mov    rcx,rax
   344f6:	or     rcx,r13
   344f9:	shr    rcx,0x20
   344fd:	je     34506 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x126>
   344ff:	xor    edx,edx
   34501:	div    r13
   34504:	jmp    3450b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12b>
   34506:	xor    edx,edx
   34508:	div    r13d
   3450b:	mov    r14,r13
   3450e:	mov    r13,QWORD PTR [r12+0x40]
   34513:	cmp    r13,r14
   34516:	mov    QWORD PTR [rsp+0x40],r14
   3451b:	cmova  r14,r13
   3451f:	jae    35699 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12b9>
   34525:	sub    r14,r13
   34528:	mov    rcx,QWORD PTR [r12+0x30]
   3452d:	sub    rcx,r13
   34530:	mov    rdx,r13
   34533:	cmp    r14,rcx
   34536:	ja     363ae <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1fce>
   3453c:	mov    rsi,QWORD PTR [r12+0x38]
   34541:	inc    r13
   34544:	test   r14b,0x1
   34548:	jne    363e6 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2006>
   3454e:	mov    rcx,QWORD PTR [rsp+0x40]
   34553:	cmp    rcx,r13
   34556:	mov    r13,rcx
   34559:	jne    3641b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x203b>
   3455f:	mov    QWORD PTR [r12+0x40],rdx
   34564:	cmp    r13,rdx
   34567:	ja     356af <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12cf>
   3456d:	lea    rbx,[rax+0x40]
   34571:	mov    r14,QWORD PTR [r12+0x38]
   34576:	lea    rcx,[r13*8+0x0]
   3457e:	lea    r15,[rcx+rcx*2]
   34582:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34590:	mov    QWORD PTR [r14+0x10],0x0
   34598:	cmp    rbx,QWORD PTR [r14]
   3459b:	ja     345a9 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c9>
   3459d:	add    r14,0x18
   345a1:	add    r15,0xffffffffffffffe8
   345a5:	jne    34590 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   345a7:	jmp    345db <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1fb>
   345a9:	mov    ecx,0x1
   345ae:	mov    r8d,0x1
   345b4:	mov    rdi,r14
   345b7:	xor    esi,esi
   345b9:	mov    rdx,rbx
   345bc:	mov    QWORD PTR [rsp+0x160],rax
   345c4:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   345c9:	mov    rax,QWORD PTR [rsp+0x160]
   345d1:	add    r14,0x18
   345d5:	add    r15,0xffffffffffffffe8
   345d9:	jne    34590 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   345db:	mov    QWORD PTR [r12+0x58],0x0
   345e4:	cmp    r13,QWORD PTR [r12+0x48]
   345e9:	ja     36371 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f91>
   345ef:	xor    ebx,ebx
   345f1:	mov    r14,QWORD PTR [r12+0x50]
   345f6:	lea    rdi,[r14+rbx*8]
   345fa:	cmp    r13,0x1
   345fe:	je     3462b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x24b>
   34600:	lea    rdx,[r13*8-0x8]
   34608:	mov    esi,0xff
   3460d:	mov    r15,rax
   34610:	call   QWORD PTR [rip+0x61a32]        # 96048 <memset@GLIBC_2.2.5>
   34616:	mov    rax,r15
   34619:	lea    rcx,[rbx+r13*1]
   3461d:	lea    rdi,[r14+rcx*8]
   34621:	add    rdi,0xfffffffffffffff8
   34625:	add    rbx,r13
   34628:	dec    rbx
   3462b:	mov    QWORD PTR [rdi],0xffffffffffffffff
   34632:	inc    rbx
   34635:	mov    QWORD PTR [r12+0x58],rbx
   3463a:	xorps  xmm0,xmm0
   3463d:	movups XMMWORD PTR [rsp+0x268],xmm0
   34645:	mov    DWORD PTR [rsp+0x278],0x0
   34650:	movups XMMWORD PTR [rsp+0x288],xmm0
   34658:	mov    DWORD PTR [rsp+0x298],0x0
   34663:	movups XMMWORD PTR [rsp+0x2a8],xmm0
   3466b:	mov    DWORD PTR [rsp+0x2b8],0x0
   34676:	movups XMMWORD PTR [rsp+0x2c8],xmm0
   3467e:	mov    DWORD PTR [rsp+0x2d8],0x0
   34689:	movups XMMWORD PTR [rsp+0x2e8],xmm0
   34691:	mov    DWORD PTR [rsp+0x2f8],0x0
   3469c:	movups XMMWORD PTR [rsp+0x308],xmm0
   346a4:	mov    DWORD PTR [rsp+0x318],0x0
   346af:	movups XMMWORD PTR [rsp+0x328],xmm0
   346b7:	mov    DWORD PTR [rsp+0x338],0x0
   346c2:	movups XMMWORD PTR [rsp+0x348],xmm0
   346ca:	mov    DWORD PTR [rsp+0x358],0x0
   346d5:	movups XMMWORD PTR [rsp+0x368],xmm0
   346dd:	mov    DWORD PTR [rsp+0x378],0x0
   346e8:	movups XMMWORD PTR [rsp+0x388],xmm0
   346f0:	mov    DWORD PTR [rsp+0x398],0x0
   346fb:	movups XMMWORD PTR [rsp+0x3a8],xmm0
   34703:	mov    DWORD PTR [rsp+0x3b8],0x0
   3470e:	movups XMMWORD PTR [rsp+0x3c8],xmm0
   34716:	mov    DWORD PTR [rsp+0x3d8],0x0
   34721:	movups XMMWORD PTR [rsp+0x3e8],xmm0
   34729:	mov    DWORD PTR [rsp+0x3f8],0x0
   34734:	movups XMMWORD PTR [rsp+0x408],xmm0
   3473c:	mov    DWORD PTR [rsp+0x418],0x0
   34747:	movups XMMWORD PTR [rsp+0x428],xmm0
   3474f:	mov    DWORD PTR [rsp+0x438],0x0
   3475a:	movups XMMWORD PTR [rsp+0x448],xmm0
   34762:	mov    DWORD PTR [rsp+0x458],0x0
   3476d:	movups XMMWORD PTR [rsp+0x468],xmm0
   34775:	mov    DWORD PTR [rsp+0x478],0x0
   34780:	movups XMMWORD PTR [rsp+0x488],xmm0
   34788:	mov    DWORD PTR [rsp+0x498],0x0
   34793:	movups XMMWORD PTR [rsp+0x4a8],xmm0
   3479b:	mov    DWORD PTR [rsp+0x4b8],0x0
   347a6:	movups XMMWORD PTR [rsp+0x4c8],xmm0
   347ae:	mov    DWORD PTR [rsp+0x4d8],0x0
   347b9:	movups XMMWORD PTR [rsp+0x4e8],xmm0
   347c1:	mov    DWORD PTR [rsp+0x4f8],0x0
   347cc:	movups XMMWORD PTR [rsp+0x508],xmm0
   347d4:	mov    DWORD PTR [rsp+0x518],0x0
   347df:	movups XMMWORD PTR [rsp+0x528],xmm0
   347e7:	mov    DWORD PTR [rsp+0x538],0x0
   347f2:	movups XMMWORD PTR [rsp+0x548],xmm0
   347fa:	mov    DWORD PTR [rsp+0x558],0x0
   34805:	movups XMMWORD PTR [rsp+0x568],xmm0
   3480d:	mov    DWORD PTR [rsp+0x578],0x0
   34818:	movups XMMWORD PTR [rsp+0x588],xmm0
   34820:	mov    DWORD PTR [rsp+0x598],0x0
   3482b:	movups XMMWORD PTR [rsp+0x5a8],xmm0
   34833:	mov    DWORD PTR [rsp+0x5b8],0x0
   3483e:	movups XMMWORD PTR [rsp+0x5c8],xmm0
   34846:	mov    DWORD PTR [rsp+0x5d8],0x0
   34851:	movups XMMWORD PTR [rsp+0x5e8],xmm0
   34859:	mov    DWORD PTR [rsp+0x5f8],0x0
   34864:	movups XMMWORD PTR [rsp+0x608],xmm0
   3486c:	mov    DWORD PTR [rsp+0x618],0x0
   34877:	movups XMMWORD PTR [rsp+0x628],xmm0
   3487f:	mov    DWORD PTR [rsp+0x638],0x0
   3488a:	movups XMMWORD PTR [rsp+0x648],xmm0
   34892:	mov    DWORD PTR [rsp+0x658],0x0
   3489d:	mov    QWORD PTR [rsp+0x260],0xffffffffffffffff
   348a9:	mov    QWORD PTR [rsp+0x280],0xffffffffffffffff
   348b5:	mov    QWORD PTR [rsp+0x2a0],0xffffffffffffffff
   348c1:	mov    QWORD PTR [rsp+0x2c0],0xffffffffffffffff
   348cd:	mov    QWORD PTR [rsp+0x2e0],0xffffffffffffffff
   348d9:	mov    QWORD PTR [rsp+0x300],0xffffffffffffffff
   348e5:	mov    QWORD PTR [rsp+0x320],0xffffffffffffffff
   348f1:	mov    QWORD PTR [rsp+0x340],0xffffffffffffffff
   348fd:	mov    QWORD PTR [rsp+0x360],0xffffffffffffffff
   34909:	mov    QWORD PTR [rsp+0x380],0xffffffffffffffff
   34915:	mov    QWORD PTR [rsp+0x3a0],0xffffffffffffffff
   34921:	mov    QWORD PTR [rsp+0x3c0],0xffffffffffffffff
   3492d:	mov    QWORD PTR [rsp+0x3e0],0xffffffffffffffff
   34939:	mov    QWORD PTR [rsp+0x400],0xffffffffffffffff
   34945:	mov    QWORD PTR [rsp+0x420],0xffffffffffffffff
   34951:	mov    QWORD PTR [rsp+0x440],0xffffffffffffffff
   3495d:	mov    QWORD PTR [rsp+0x460],0xffffffffffffffff
   34969:	mov    QWORD PTR [rsp+0x480],0xffffffffffffffff
   34975:	mov    QWORD PTR [rsp+0x4a0],0xffffffffffffffff
   34981:	mov    QWORD PTR [rsp+0x4c0],0xffffffffffffffff
   3498d:	mov    QWORD PTR [rsp+0x4e0],0xffffffffffffffff
   34999:	mov    QWORD PTR [rsp+0x500],0xffffffffffffffff
   349a5:	mov    QWORD PTR [rsp+0x520],0xffffffffffffffff
   349b1:	mov    QWORD PTR [rsp+0x540],0xffffffffffffffff
   349bd:	mov    QWORD PTR [rsp+0x560],0xffffffffffffffff
   349c9:	mov    QWORD PTR [rsp+0x580],0xffffffffffffffff
   349d5:	mov    QWORD PTR [rsp+0x5a0],0xffffffffffffffff
   349e1:	mov    QWORD PTR [rsp+0x5c0],0xffffffffffffffff
   349ed:	mov    QWORD PTR [rsp+0x5e0],0xffffffffffffffff
   349f9:	mov    QWORD PTR [rsp+0x600],0xffffffffffffffff
   34a05:	mov    QWORD PTR [rsp+0x620],0xffffffffffffffff
   34a11:	mov    QWORD PTR [rsp+0x640],0xffffffffffffffff
   34a1d:	mov    rcx,QWORD PTR [r12+0x38]
   34a22:	mov    rbx,QWORD PTR [r12+0x40]
   34a27:	mov    r9,QWORD PTR [r12+0x8]
   34a2c:	mov    rsi,QWORD PTR [r12+0x10]
   34a31:	mov    QWORD PTR [rsp+0x8],rcx
   34a36:	lea    r11,[rcx+0x8]
   34a3a:	xor    edi,edi
   34a3c:	xor    r12d,r12d
   34a3f:	mov    r8,QWORD PTR [rsp+0x38]
   34a44:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34a50:	cmp    rbx,rdi
   34a53:	je     36485 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20a5>
   34a59:	cmp    rsi,rdi
   34a5c:	je     36494 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20b4>
   34a62:	mov    rcx,QWORD PTR [r11]
   34a65:	mov    edx,DWORD PTR [r9+rdi*4]
   34a69:	mov    edx,DWORD PTR [r8+rdx*4]
   34a6d:	mov    BYTE PTR [rcx],dl
   34a6f:	shr    edx,0x8
   34a72:	and    edx,0x7fffff
   34a78:	lea    r10,[rcx+0x1]
   34a7c:	add    rcx,QWORD PTR [r11-0x8]
   34a80:	mov    QWORD PTR [rsp+r12*1+0x260],rdi
   34a88:	mov    QWORD PTR [rsp+r12*1+0x268],r10
   34a90:	mov    QWORD PTR [rsp+r12*1+0x270],rcx
   34a98:	mov    DWORD PTR [rsp+r12*1+0x278],edx
   34aa0:	prefetcht0 BYTE PTR [r8+rdx*4]
   34aa5:	lea    rcx,[rdi+0x1]
   34aa9:	add    r12,0x20
   34aad:	cmp    rdi,0x1e
   34ab1:	ja     34abf <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6df>
   34ab3:	add    r11,0x18
   34ab7:	mov    rdi,rcx
   34aba:	cmp    rcx,r13
   34abd:	jb     34a50 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x670>
   34abf:	cmp    rcx,0x20
   34ac3:	jne    356c0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12e0>
   34ac9:	mov    QWORD PTR [rsp+0x1d0],rcx
   34ad1:	mov    QWORD PTR [rsp+0x58],rbp
   34ad6:	add    rax,0x3f
   34ada:	mov    ebp,0x20
   34adf:	mov    QWORD PTR [rsp+0x160],rax
   34ae7:	xor    r15d,r15d
   34aea:	mov    r14,rax
   34aed:	jmp    34b21 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x741>
   34aef:	nop
   34af0:	add    r13,rdi
   34af3:	mov    QWORD PTR [rsp+r15*1+0x268],r13
   34afb:	add    rdx,rdi
   34afe:	mov    QWORD PTR [rsp+r15*1+0x270],rdx
   34b06:	sub    rdx,r13
   34b09:	add    r15,0x20
   34b0d:	cmp    rdx,r14
   34b10:	cmovb  r14,rdx
   34b14:	cmp    r15,0x400
   34b1b:	je     34bb0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x7d0>
   34b21:	mov    r13,QWORD PTR [rsp+r15*1+0x268]
   34b29:	mov    rdx,QWORD PTR [rsp+r15*1+0x270]
   34b31:	cmp    r13,rdx
   34b34:	jne    34b06 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x726>
   34b36:	mov    rdi,QWORD PTR [rsp+r15*1+0x260]
   34b3e:	cmp    rdi,rbx
   34b41:	jae    36559 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2179>
   34b47:	lea    rcx,[rdi+rdi*2]
   34b4b:	mov    rsi,QWORD PTR [rsp+0x8]
   34b50:	mov    rdx,QWORD PTR [rsi+rcx*8]
   34b54:	mov    rdi,QWORD PTR [rsi+rcx*8+0x8]
   34b59:	sub    r13,rdi
   34b5c:	mov    QWORD PTR [rsi+rcx*8+0x10],r13
   34b61:	mov    rsi,rdx
   34b64:	sub    rsi,r13
   34b67:	cmp    rdx,rsi
   34b6a:	jbe    34af0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x710>
   34b6c:	mov    rsi,QWORD PTR [rsp+0x8]
   34b71:	lea    rdi,[rsi+rcx*8]
   34b75:	mov    QWORD PTR [rsp],rdi
   34b79:	mov    ecx,0x1
   34b7e:	mov    r8d,0x1
   34b84:	mov    rsi,r13
   34b87:	mov    QWORD PTR [rsp+0x28],rbx
   34b8c:	mov    rbx,rax
   34b8f:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   34b94:	mov    rax,rbx
   34b97:	mov    rbx,QWORD PTR [rsp+0x28]
   34b9c:	mov    rcx,QWORD PTR [rsp]
   34ba0:	mov    rdx,QWORD PTR [rcx]
   34ba3:	mov    rdi,QWORD PTR [rcx+0x8]
   34ba7:	jmp    34af0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x710>
   34bac:	nop    DWORD PTR [rax+0x0]
   34bb0:	mov    rax,QWORD PTR [rsp+0x268]
   34bb8:	mov    rcx,QWORD PTR [rsp+0x288]
   34bc0:	mov    QWORD PTR [rsp+0x20],rcx
   34bc5:	mov    rcx,QWORD PTR [rsp+0x2a8]
   34bcd:	mov    QWORD PTR [rsp],rcx
   34bd1:	mov    rcx,QWORD PTR [rsp+0x2c8]
   34bd9:	mov    QWORD PTR [rsp+0x18],rcx
   34bde:	mov    rdx,QWORD PTR [rsp+0x2e8]
   34be6:	mov    rsi,QWORD PTR [rsp+0x308]
   34bee:	mov    rdi,QWORD PTR [rsp+0x328]
   34bf6:	mov    r8,QWORD PTR [rsp+0x348]
   34bfe:	mov    r9,QWORD PTR [rsp+0x368]
   34c06:	mov    r10,QWORD PTR [rsp+0x388]
   34c0e:	mov    r11,QWORD PTR [rsp+0x3a8]
   34c16:	mov    rbx,QWORD PTR [rsp+0x3c8]
   34c1e:	mov    r15,QWORD PTR [rsp+0x3e8]
   34c26:	mov    r13,QWORD PTR [rsp+0x408]
   34c2e:	mov    rcx,QWORD PTR [rsp+0x428]
   34c36:	mov    QWORD PTR [rsp+0x10],rcx
   34c3b:	mov    QWORD PTR [rsp+0x50],rax
   34c40:	mov    QWORD PTR [rsp+0x60],rax
   34c45:	mov    rax,QWORD PTR [rsp+0x20]
   34c4a:	mov    QWORD PTR [rsp+0x68],rax
   34c4f:	mov    rax,QWORD PTR [rsp]
   34c53:	mov    QWORD PTR [rsp+0x70],rax
   34c58:	mov    rax,QWORD PTR [rsp+0x18]
   34c5d:	mov    QWORD PTR [rsp+0x78],rax
   34c62:	mov    QWORD PTR [rsp+0x258],rdx
   34c6a:	mov    QWORD PTR [rsp+0x80],rdx
   34c72:	mov    QWORD PTR [rsp+0x250],rsi
   34c7a:	mov    QWORD PTR [rsp+0x88],rsi
   34c82:	mov    QWORD PTR [rsp+0x248],rdi
   34c8a:	mov    QWORD PTR [rsp+0x90],rdi
   34c92:	mov    QWORD PTR [rsp+0x240],r8
   34c9a:	mov    QWORD PTR [rsp+0x98],r8
   34ca2:	mov    QWORD PTR [rsp+0x238],r9
   34caa:	mov    QWORD PTR [rsp+0xa0],r9
   34cb2:	mov    QWORD PTR [rsp+0x230],r10
   34cba:	mov    QWORD PTR [rsp+0xa8],r10
   34cc2:	mov    QWORD PTR [rsp+0x228],r11
   34cca:	mov    QWORD PTR [rsp+0xb0],r11
   34cd2:	mov    QWORD PTR [rsp+0x220],rbx
   34cda:	mov    QWORD PTR [rsp+0xb8],rbx
   34ce2:	mov    QWORD PTR [rsp+0x218],r15
   34cea:	mov    QWORD PTR [rsp+0xc0],r15
   34cf2:	mov    QWORD PTR [rsp+0x1d8],r13
   34cfa:	mov    QWORD PTR [rsp+0xc8],r13
   34d02:	mov    rax,QWORD PTR [rsp+0x10]
   34d07:	mov    QWORD PTR [rsp+0xd0],rax
   34d0f:	mov    rax,QWORD PTR [rsp+0x448]
   34d17:	mov    QWORD PTR [rsp+0x1e0],rax
   34d1f:	mov    QWORD PTR [rsp+0xd8],rax
   34d27:	mov    rax,QWORD PTR [rsp+0x468]
   34d2f:	mov    QWORD PTR [rsp+0x1e8],rax
   34d37:	mov    QWORD PTR [rsp+0xe0],rax
   34d3f:	mov    rax,QWORD PTR [rsp+0x488]
   34d47:	mov    QWORD PTR [rsp+0x1f0],rax
   34d4f:	mov    QWORD PTR [rsp+0xe8],rax
   34d57:	mov    rax,QWORD PTR [rsp+0x4a8]
   34d5f:	mov    QWORD PTR [rsp+0x1f8],rax
   34d67:	mov    QWORD PTR [rsp+0xf0],rax
   34d6f:	mov    rax,QWORD PTR [rsp+0x4c8]
   34d77:	mov    QWORD PTR [rsp+0x200],rax
   34d7f:	mov    QWORD PTR [rsp+0xf8],rax
   34d87:	mov    rax,QWORD PTR [rsp+0x4e8]
   34d8f:	mov    QWORD PTR [rsp+0x208],rax
   34d97:	mov    QWORD PTR [rsp+0x100],rax
   34d9f:	mov    rax,QWORD PTR [rsp+0x508]
   34da7:	mov    QWORD PTR [rsp+0x210],rax
   34daf:	mov    QWORD PTR [rsp+0x108],rax
   34db7:	mov    rcx,QWORD PTR [rsp+0x528]
   34dbf:	mov    QWORD PTR [rsp+0x110],rcx
   34dc7:	mov    rdx,QWORD PTR [rsp+0x548]
   34dcf:	mov    QWORD PTR [rsp+0x118],rdx
   34dd7:	mov    r10,QWORD PTR [rsp+0x568]
   34ddf:	mov    QWORD PTR [rsp+0x120],r10
   34de7:	mov    r13,QWORD PTR [rsp+0x588]
   34def:	mov    QWORD PTR [rsp+0x128],r13
   34df7:	mov    rax,QWORD PTR [rsp+0x5a8]
   34dff:	mov    QWORD PTR [rsp+0x130],rax
   34e07:	mov    r11,QWORD PTR [rsp+0x5c8]
   34e0f:	mov    QWORD PTR [rsp+0x138],r11
   34e17:	mov    r8,QWORD PTR [rsp+0x5e8]
   34e1f:	mov    QWORD PTR [rsp+0x140],r8
   34e27:	mov    r9,QWORD PTR [rsp+0x608]
   34e2f:	mov    QWORD PTR [rsp+0x148],r9
   34e37:	mov    rsi,QWORD PTR [rsp+0x628]
   34e3f:	mov    QWORD PTR [rsp+0x150],rsi
   34e47:	mov    rdi,QWORD PTR [rsp+0x648]
   34e4f:	mov    QWORD PTR [rsp+0x158],rdi
   34e57:	mov    rbx,QWORD PTR [rsp+0x30]
   34e5c:	mov    r15,QWORD PTR [rbx+0x40]
   34e60:	mov    QWORD PTR [rsp+0x28],r15
   34e65:	test   r14,r14
   34e68:	je     35250 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe70>
   34e6e:	mov    rax,QWORD PTR [rbx+0x50]
   34e72:	mov    QWORD PTR [rsp+0x18],rax
   34e77:	mov    rax,QWORD PTR [rbx+0x58]
   34e7b:	mov    QWORD PTR [rsp+0x20],rax
   34e80:	mov    r10,QWORD PTR [rbx+0x8]
   34e84:	mov    rsi,QWORD PTR [rbx+0x10]
   34e88:	xor    r8d,r8d
   34e8b:	mov    rbx,QWORD PTR [rsp+0x38]
   34e90:	mov    QWORD PTR [rsp+0x10],r10
   34e95:	mov    QWORD PTR [rsp],rsi
   34e99:	jmp    34eac <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xacc>
   34e9b:	nop    DWORD PTR [rax+rax*1+0x0]
   34ea0:	inc    r8
   34ea3:	cmp    r8,r14
   34ea6:	je     35502 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1122>
   34eac:	lea    rax,[rsp+0x358]
   34eb4:	xor    ecx,ecx
   34eb6:	xor    r9d,r9d
   34eb9:	jmp    34ee9 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb09>
   34ebb:	nop    DWORD PTR [rax+rax*1+0x0]
   34ec0:	mov    rdx,QWORD PTR [rsp+rcx*8+0x98]
   34ec8:	mov    BYTE PTR [rdx+r8*1],dil
   34ecc:	shr    edi,0x8
   34ecf:	mov    DWORD PTR [rax],edi
   34ed1:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34ed5:	add    rcx,0x8
   34ed9:	add    rax,0x100
   34edf:	cmp    rcx,0x19
   34ee3:	jae    350c0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xce0>
   34ee9:	mov    edx,DWORD PTR [rax-0xe0]
   34eef:	mov    edi,DWORD PTR [rbx+rdx*4]
   34ef2:	test   edi,edi
   34ef4:	js     35000 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc20>
   34efa:	mov    rdx,QWORD PTR [rsp+rcx*8+0x60]
   34eff:	mov    BYTE PTR [rdx+r8*1],dil
   34f03:	shr    edi,0x8
   34f06:	mov    DWORD PTR [rax-0xe0],edi
   34f0c:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f10:	mov    edx,DWORD PTR [rax-0xc0]
   34f16:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f19:	test   edi,edi
   34f1b:	js     35015 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc35>
   34f21:	mov    rdx,QWORD PTR [rsp+rcx*8+0x68]
   34f26:	mov    BYTE PTR [rdx+r8*1],dil
   34f2a:	shr    edi,0x8
   34f2d:	mov    DWORD PTR [rax-0xc0],edi
   34f33:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f37:	mov    edx,DWORD PTR [rax-0xa0]
   34f3d:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f40:	test   edi,edi
   34f42:	js     35031 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc51>
   34f48:	mov    rdx,QWORD PTR [rsp+rcx*8+0x70]
   34f4d:	mov    BYTE PTR [rdx+r8*1],dil
   34f51:	shr    edi,0x8
   34f54:	mov    DWORD PTR [rax-0xa0],edi
   34f5a:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f5e:	mov    edx,DWORD PTR [rax-0x80]
   34f61:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f64:	test   edi,edi
   34f66:	js     3504a <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc6a>
   34f6c:	mov    rdx,QWORD PTR [rsp+rcx*8+0x78]
   34f71:	mov    BYTE PTR [rdx+r8*1],dil
   34f75:	shr    edi,0x8
   34f78:	mov    DWORD PTR [rax-0x80],edi
   34f7b:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34f7f:	mov    edx,DWORD PTR [rax-0x60]
   34f82:	mov    edi,DWORD PTR [rbx+rdx*4]
   34f85:	test   edi,edi
   34f87:	js     35063 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc83>
   34f8d:	mov    rdx,QWORD PTR [rsp+rcx*8+0x80]
   34f95:	mov    BYTE PTR [rdx+r8*1],dil
   34f99:	shr    edi,0x8
   34f9c:	mov    DWORD PTR [rax-0x60],edi
   34f9f:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34fa3:	mov    edx,DWORD PTR [rax-0x40]
   34fa6:	mov    edi,DWORD PTR [rbx+rdx*4]
   34fa9:	test   edi,edi
   34fab:	js     3507c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc9c>
   34fb1:	mov    rdx,QWORD PTR [rsp+rcx*8+0x88]
   34fb9:	mov    BYTE PTR [rdx+r8*1],dil
   34fbd:	shr    edi,0x8
   34fc0:	mov    DWORD PTR [rax-0x40],edi
   34fc3:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34fc7:	mov    edx,DWORD PTR [rax-0x20]
   34fca:	mov    edi,DWORD PTR [rbx+rdx*4]
   34fcd:	test   edi,edi
   34fcf:	js     35095 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcb5>
   34fd5:	mov    rdx,QWORD PTR [rsp+rcx*8+0x90]
   34fdd:	mov    BYTE PTR [rdx+r8*1],dil
   34fe1:	shr    edi,0x8
   34fe4:	mov    DWORD PTR [rax-0x20],edi
   34fe7:	prefetcht0 BYTE PTR [rbx+rdi*4]
   34feb:	mov    edx,DWORD PTR [rax]
   34fed:	mov    edi,DWORD PTR [rbx+rdx*4]
   34ff0:	test   edi,edi
   34ff2:	jns    34ec0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xae0>
   34ff8:	jmp    350ad <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xccd>
   34ffd:	nop    DWORD PTR [rax]
   35000:	bts    r9,rcx
   35004:	mov    edx,DWORD PTR [rax-0xc0]
   3500a:	mov    edi,DWORD PTR [rbx+rdx*4]
   3500d:	test   edi,edi
   3500f:	jns    34f21 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb41>
   35015:	mov    edx,0x2
   3501a:	shl    rdx,cl
   3501d:	or     r9,rdx
   35020:	mov    edx,DWORD PTR [rax-0xa0]
   35026:	mov    edi,DWORD PTR [rbx+rdx*4]
   35029:	test   edi,edi
   3502b:	jns    34f48 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb68>
   35031:	mov    edx,0x4
   35036:	shl    rdx,cl
   35039:	or     r9,rdx
   3503c:	mov    edx,DWORD PTR [rax-0x80]
   3503f:	mov    edi,DWORD PTR [rbx+rdx*4]
   35042:	test   edi,edi
   35044:	jns    34f6c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xb8c>
   3504a:	mov    edx,0x8
   3504f:	shl    rdx,cl
   35052:	or     r9,rdx
   35055:	mov    edx,DWORD PTR [rax-0x60]
   35058:	mov    edi,DWORD PTR [rbx+rdx*4]
   3505b:	test   edi,edi
   3505d:	jns    34f8d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbad>
   35063:	mov    edx,0x10
   35068:	shl    rdx,cl
   3506b:	or     r9,rdx
   3506e:	mov    edx,DWORD PTR [rax-0x40]
   35071:	mov    edi,DWORD PTR [rbx+rdx*4]
   35074:	test   edi,edi
   35076:	jns    34fb1 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbd1>
   3507c:	mov    edx,0x20
   35081:	shl    rdx,cl
   35084:	or     r9,rdx
   35087:	mov    edx,DWORD PTR [rax-0x20]
   3508a:	mov    edi,DWORD PTR [rbx+rdx*4]
   3508d:	test   edi,edi
   3508f:	jns    34fd5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbf5>
   35095:	mov    edx,0x40
   3509a:	shl    rdx,cl
   3509d:	or     r9,rdx
   350a0:	mov    edx,DWORD PTR [rax]
   350a2:	mov    edi,DWORD PTR [rbx+rdx*4]
   350a5:	test   edi,edi
   350a7:	jns    34ec0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xae0>
   350ad:	mov    edx,0x80
   350b2:	shl    rdx,cl
   350b5:	or     r9,rdx
   350b8:	jmp    34ed5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xaf5>
   350bd:	nop    DWORD PTR [rax]
   350c0:	test   r9,r9
   350c3:	je     34ea0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   350c9:	mov    rsi,r8
   350cc:	not    rsi
   350cf:	nop
   350d0:	tzcnt  rdi,r9
   350d5:	cmp    rdi,0x1f
   350d9:	ja     3650d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x212d>
   350df:	mov    r10d,edi
   350e2:	shl    r10d,0x5
   350e6:	mov    r11,QWORD PTR [rsp+rdi*8+0x60]
   350eb:	add    r11,r8
   350ee:	mov    QWORD PTR [rsp+r10*1+0x268],r11
   350f6:	mov    rax,QWORD PTR [rsp+r10*1+0x260]
   350fe:	cmp    rax,QWORD PTR [rsp+0x28]
   35103:	jae    364f9 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2119>
   35109:	add    r10,rsp
   3510c:	add    r10,0x260
   35113:	lea    r15,[rax+rax*2]
   35117:	mov    rcx,QWORD PTR [rsp+0x8]
   3511c:	sub    r11,QWORD PTR [rcx+r15*8+0x8]
   35121:	mov    QWORD PTR [rcx+r15*8+0x10],r11
   35126:	mov    r15,QWORD PTR [rsp]
   3512a:	test   r15,r15
   3512d:	je     35150 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd70>
   3512f:	mov    r13,r15
   35132:	mov    r15d,DWORD PTR [r10+0x18]
   35136:	xor    r11d,r11d
   35139:	cmp    r13,0x1
   3513d:	jne    35160 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd80>
   3513f:	mov    rcx,QWORD PTR [rsp+0x10]
   35144:	jmp    3518d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdad>
   35146:	cs nop WORD PTR [rax+rax*1+0x0]
   35150:	mov    r11,0xffffffffffffffff
   35157:	mov    rcx,QWORD PTR [rsp+0x10]
   3515c:	jmp    351a5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdc5>
   3515e:	xchg   ax,ax
   35160:	mov    rcx,QWORD PTR [rsp+0x10]
   35165:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   35170:	mov    rdx,r11
   35173:	mov    rbx,r13
   35176:	shr    rbx,1
   35179:	add    r11,rbx
   3517c:	cmp    DWORD PTR [rcx+r11*4],r15d
   35180:	cmova  r11,rdx
   35184:	sub    r13,rbx
   35187:	cmp    r13,0x1
   3518b:	ja     35170 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd90>
   3518d:	cmp    DWORD PTR [rcx+r11*4],r15d
   35191:	mov    rdx,0xffffffffffffffff
   35198:	cmovne r11,rdx
   3519c:	mov    rbx,QWORD PTR [rsp+0x38]
   351a1:	mov    r15,QWORD PTR [rsp]
   351a5:	cmp    rax,QWORD PTR [rsp+0x20]
   351aa:	jae    364e5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2105>
   351b0:	lea    rdx,[r9-0x1]
   351b4:	and    r9,rdx
   351b7:	mov    rdx,QWORD PTR [rsp+0x18]
   351bc:	mov    QWORD PTR [rdx+rax*8],r11
   351c0:	cmp    rbp,QWORD PTR [rsp+0x40]
   351c5:	je     356e5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1305>
   351cb:	cmp    rbp,QWORD PTR [rsp+0x28]
   351d0:	jae    36531 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2151>
   351d6:	cmp    rbp,r15
   351d9:	jae    3651e <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x213e>
   351df:	lea    rax,[rbp*2+0x0]
   351e7:	add    rax,rbp
   351ea:	mov    r13,QWORD PTR [rsp+0x8]
   351ef:	mov    rdx,QWORD PTR [r13+rax*8+0x8]
   351f4:	mov    r11d,DWORD PTR [rcx+rbp*4]
   351f8:	mov    r11d,DWORD PTR [rbx+r11*4]
   351fc:	mov    BYTE PTR [rdx],r11b
   351ff:	shr    r11d,0x8
   35203:	and    r11d,0x7fffff
   3520a:	lea    rbx,[rdx+0x1]
   3520e:	lea    r15,[rdx+rsi*1]
   35212:	inc    r15
   35215:	add    rdx,QWORD PTR [r13+rax*8+0x0]
   3521a:	mov    QWORD PTR [r10],rbp
   3521d:	mov    QWORD PTR [r10+0x8],rbx
   35221:	mov    rbx,QWORD PTR [rsp+0x38]
   35226:	mov    QWORD PTR [r10+0x10],rdx
   3522a:	mov    DWORD PTR [r10+0x18],r11d
   3522e:	prefetcht0 BYTE PTR [rbx+r11*4]
   35233:	inc    rbp
   35236:	mov    QWORD PTR [rsp+rdi*8+0x60],r15
   3523b:	test   r9,r9
   3523e:	jne    350d0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcf0>
   35244:	jmp    34ea0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xac0>
   35249:	nop    DWORD PTR [rax+0x0]
   35250:	mov    QWORD PTR [rsp+0x180],r10
   35258:	mov    QWORD PTR [rsp+0x188],r13
   35260:	mov    QWORD PTR [rsp+0x190],rax
   35268:	mov    QWORD PTR [rsp+0x198],rcx
   35270:	mov    QWORD PTR [rsp+0x1a0],rdx
   35278:	mov    QWORD PTR [rsp+0x1a8],rsi
   35280:	mov    QWORD PTR [rsp+0x1b0],rdi
   35288:	mov    QWORD PTR [rsp+0x1b8],r8
   35290:	mov    QWORD PTR [rsp+0x1c0],r9
   35298:	mov    QWORD PTR [rsp+0x1c8],r11
   352a0:	mov    rcx,QWORD PTR [rsp+0x218]
   352a8:	mov    rdx,QWORD PTR [rsp+0x220]
   352b0:	mov    rsi,QWORD PTR [rsp+0x228]
   352b8:	mov    rdi,QWORD PTR [rsp+0x230]
   352c0:	mov    r8,QWORD PTR [rsp+0x238]
   352c8:	mov    r9,QWORD PTR [rsp+0x240]
   352d0:	mov    r11,QWORD PTR [rsp+0x248]
   352d8:	mov    r13,QWORD PTR [rsp+0x250]
   352e0:	mov    r10,QWORD PTR [rsp+0x258]
   352e8:	mov    r15,QWORD PTR [rsp+0x18]
   352ed:	mov    rbx,QWORD PTR [rsp+0x20]
   352f2:	mov    rax,QWORD PTR [rsp+0x50]
   352f7:	add    rax,r14
   352fa:	mov    QWORD PTR [rsp+0x268],rax
   35302:	add    rbx,r14
   35305:	mov    QWORD PTR [rsp+0x288],rbx
   3530d:	mov    rbx,QWORD PTR [rsp]
   35311:	add    rbx,r14
   35314:	mov    QWORD PTR [rsp+0x2a8],rbx
   3531c:	add    r15,r14
   3531f:	mov    QWORD PTR [rsp+0x2c8],r15
   35327:	add    r10,r14
   3532a:	mov    QWORD PTR [rsp+0x2e8],r10
   35332:	add    r13,r14
   35335:	mov    QWORD PTR [rsp+0x308],r13
   3533d:	add    r11,r14
   35340:	mov    QWORD PTR [rsp+0x328],r11
   35348:	add    r9,r14
   3534b:	mov    QWORD PTR [rsp+0x348],r9
   35353:	add    r8,r14
   35356:	mov    QWORD PTR [rsp+0x368],r8
   3535e:	add    rdi,r14
   35361:	mov    QWORD PTR [rsp+0x388],rdi
   35369:	add    rsi,r14
   3536c:	mov    QWORD PTR [rsp+0x3a8],rsi
   35374:	add    rdx,r14
   35377:	mov    QWORD PTR [rsp+0x3c8],rdx
   3537f:	add    rcx,r14
   35382:	mov    QWORD PTR [rsp+0x3e8],rcx
   3538a:	mov    rax,QWORD PTR [rsp+0x1d8]
   35392:	add    rax,r14
   35395:	mov    QWORD PTR [rsp+0x408],rax
   3539d:	mov    rax,QWORD PTR [rsp+0x10]
   353a2:	add    rax,r14
   353a5:	mov    QWORD PTR [rsp+0x428],rax
   353ad:	mov    rax,QWORD PTR [rsp+0x1e0]
   353b5:	add    rax,r14
   353b8:	mov    QWORD PTR [rsp+0x448],rax
   353c0:	mov    rax,QWORD PTR [rsp+0x1e8]
   353c8:	add    rax,r14
   353cb:	mov    QWORD PTR [rsp+0x468],rax
   353d3:	mov    rax,QWORD PTR [rsp+0x1f0]
   353db:	add    rax,r14
   353de:	mov    QWORD PTR [rsp+0x488],rax
   353e6:	mov    rax,QWORD PTR [rsp+0x1f8]
   353ee:	add    rax,r14
   353f1:	mov    QWORD PTR [rsp+0x4a8],rax
   353f9:	mov    rax,QWORD PTR [rsp+0x200]
   35401:	add    rax,r14
   35404:	mov    QWORD PTR [rsp+0x4c8],rax
   3540c:	mov    rax,QWORD PTR [rsp+0x208]
   35414:	add    rax,r14
   35417:	mov    QWORD PTR [rsp+0x4e8],rax
   3541f:	mov    rax,QWORD PTR [rsp+0x210]
   35427:	add    rax,r14
   3542a:	mov    QWORD PTR [rsp+0x508],rax
   35432:	mov    rax,QWORD PTR [rsp+0x198]
   3543a:	add    rax,r14
   3543d:	mov    QWORD PTR [rsp+0x528],rax
   35445:	mov    rax,QWORD PTR [rsp+0x1a0]
   3544d:	add    rax,r14
   35450:	mov    QWORD PTR [rsp+0x548],rax
   35458:	mov    rax,QWORD PTR [rsp+0x180]
   35460:	add    rax,r14
   35463:	mov    QWORD PTR [rsp+0x568],rax
   3546b:	mov    rax,QWORD PTR [rsp+0x188]
   35473:	add    rax,r14
   35476:	mov    QWORD PTR [rsp+0x588],rax
   3547e:	mov    rax,QWORD PTR [rsp+0x190]
   35486:	add    rax,r14
   35489:	mov    QWORD PTR [rsp+0x5a8],rax
   35491:	mov    rax,QWORD PTR [rsp+0x1c8]
   35499:	add    rax,r14
   3549c:	mov    QWORD PTR [rsp+0x5c8],rax
   354a4:	mov    rax,QWORD PTR [rsp+0x1b8]
   354ac:	add    rax,r14
   354af:	mov    QWORD PTR [rsp+0x5e8],rax
   354b7:	mov    rax,QWORD PTR [rsp+0x1c0]
   354bf:	add    rax,r14
   354c2:	mov    QWORD PTR [rsp+0x608],rax
   354ca:	mov    rax,QWORD PTR [rsp+0x1a8]
   354d2:	add    rax,r14
   354d5:	mov    QWORD PTR [rsp+0x628],rax
   354dd:	mov    rax,QWORD PTR [rsp+0x1b0]
   354e5:	add    rax,r14
   354e8:	mov    QWORD PTR [rsp+0x648],rax
   354f0:	mov    rax,QWORD PTR [rsp+0x160]
   354f8:	mov    rbx,QWORD PTR [rsp+0x28]
   354fd:	jmp    34ae7 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x707>
   35502:	mov    rax,QWORD PTR [rsp+0x60]
   35507:	mov    QWORD PTR [rsp+0x50],rax
   3550c:	mov    rbx,QWORD PTR [rsp+0x68]
   35511:	mov    rax,QWORD PTR [rsp+0x70]
   35516:	mov    QWORD PTR [rsp],rax
   3551a:	mov    r15,QWORD PTR [rsp+0x78]
   3551f:	mov    r10,QWORD PTR [rsp+0x80]
   35527:	mov    r13,QWORD PTR [rsp+0x88]
   3552f:	mov    r11,QWORD PTR [rsp+0x90]
   35537:	mov    r9,QWORD PTR [rsp+0x98]
   3553f:	mov    r8,QWORD PTR [rsp+0xa0]
   35547:	mov    rdi,QWORD PTR [rsp+0xa8]
   3554f:	mov    rsi,QWORD PTR [rsp+0xb0]
   35557:	mov    rdx,QWORD PTR [rsp+0xb8]
   3555f:	mov    rcx,QWORD PTR [rsp+0xc0]
   35567:	mov    rax,QWORD PTR [rsp+0xc8]
   3556f:	mov    QWORD PTR [rsp+0x1d8],rax
   35577:	mov    rax,QWORD PTR [rsp+0xd0]
   3557f:	mov    QWORD PTR [rsp+0x10],rax
   35584:	mov    rax,QWORD PTR [rsp+0xd8]
   3558c:	mov    QWORD PTR [rsp+0x1e0],rax
   35594:	mov    rax,QWORD PTR [rsp+0xe0]
   3559c:	mov    QWORD PTR [rsp+0x1e8],rax
   355a4:	mov    rax,QWORD PTR [rsp+0xe8]
   355ac:	mov    QWORD PTR [rsp+0x1f0],rax
   355b4:	mov    rax,QWORD PTR [rsp+0xf0]
   355bc:	mov    QWORD PTR [rsp+0x1f8],rax
   355c4:	mov    rax,QWORD PTR [rsp+0xf8]
   355cc:	mov    QWORD PTR [rsp+0x200],rax
   355d4:	mov    rax,QWORD PTR [rsp+0x100]
   355dc:	mov    QWORD PTR [rsp+0x208],rax
   355e4:	mov    rax,QWORD PTR [rsp+0x108]
   355ec:	mov    QWORD PTR [rsp+0x210],rax
   355f4:	mov    rax,QWORD PTR [rsp+0x110]
   355fc:	mov    QWORD PTR [rsp+0x198],rax
   35604:	mov    rax,QWORD PTR [rsp+0x118]
   3560c:	mov    QWORD PTR [rsp+0x1a0],rax
   35614:	mov    rax,QWORD PTR [rsp+0x120]
   3561c:	mov    QWORD PTR [rsp+0x180],rax
   35624:	mov    rax,QWORD PTR [rsp+0x128]
   3562c:	mov    QWORD PTR [rsp+0x188],rax
   35634:	mov    rax,QWORD PTR [rsp+0x130]
   3563c:	mov    QWORD PTR [rsp+0x190],rax
   35644:	mov    rax,QWORD PTR [rsp+0x138]
   3564c:	mov    QWORD PTR [rsp+0x1c8],rax
   35654:	mov    rax,QWORD PTR [rsp+0x140]
   3565c:	mov    QWORD PTR [rsp+0x1b8],rax
   35664:	mov    rax,QWORD PTR [rsp+0x148]
   3566c:	mov    QWORD PTR [rsp+0x1c0],rax
   35674:	mov    rax,QWORD PTR [rsp+0x150]
   3567c:	mov    QWORD PTR [rsp+0x1a8],rax
   35684:	mov    rax,QWORD PTR [rsp+0x158]
   3568c:	mov    QWORD PTR [rsp+0x1b0],rax
   35694:	jmp    352f2 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf12>
   35699:	mov    rdx,r14
   3569c:	mov    r13,QWORD PTR [rsp+0x40]
   356a1:	mov    QWORD PTR [r12+0x40],rdx
   356a6:	cmp    r13,rdx
   356a9:	jbe    3456d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d>
   356af:	lea    rcx,[rip+0x5e522]        # 93bd8 <__frame_dummy_init_array_entry+0x920>
   356b6:	xor    edi,edi
   356b8:	mov    rsi,r13
   356bb:	call   2bcc0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   356c0:	mov    QWORD PTR [rsp],rsi
   356c4:	mov    QWORD PTR [rsp+0x10],r9
   356c9:	mov    rax,QWORD PTR [rsp+0x30]
   356ce:	mov    rdx,QWORD PTR [rax+0x50]
   356d2:	mov    QWORD PTR [rsp+0x18],rdx
   356d7:	mov    rax,QWORD PTR [rax+0x58]
   356db:	mov    QWORD PTR [rsp+0x20],rax
   356e0:	jmp    35dd3 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19f3>
   356e5:	not    r9
   356e8:	test   rdi,rdi
   356eb:	je     35ae8 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1708>
   356f1:	mov    rax,QWORD PTR [rsp+0x60]
   356f6:	add    rax,r8
   356f9:	inc    rax
   356fc:	mov    QWORD PTR [rsp+0x268],rax
   35704:	cmp    edi,0x1
   35707:	mov    rbp,QWORD PTR [rsp+0x58]
   3570c:	mov    rcx,QWORD PTR [rsp+0x1d0]
   35714:	je     35b0a <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x172a>
   3571a:	bt     r9d,0x1
   3571f:	mov    rax,QWORD PTR [rsp+0x68]
   35724:	adc    rax,r8
   35727:	mov    QWORD PTR [rsp+0x288],rax
   3572f:	cmp    edi,0x2
   35732:	je     35b1f <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x173f>
   35738:	bt     r9d,0x2
   3573d:	mov    rax,QWORD PTR [rsp+0x70]
   35742:	adc    rax,r8
   35745:	mov    QWORD PTR [rsp+0x2a8],rax
   3574d:	cmp    edi,0x3
   35750:	je     35b34 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1754>
   35756:	bt     r9d,0x3
   3575b:	mov    rax,QWORD PTR [rsp+0x78]
   35760:	adc    rax,r8
   35763:	mov    QWORD PTR [rsp+0x2c8],rax
   3576b:	cmp    edi,0x4
   3576e:	je     35b4c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x176c>
   35774:	bt     r9d,0x4
   35779:	mov    rax,QWORD PTR [rsp+0x80]
   35781:	adc    rax,r8
   35784:	mov    QWORD PTR [rsp+0x2e8],rax
   3578c:	cmp    edi,0x5
   3578f:	je     35b64 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1784>
   35795:	bt     r9d,0x5
   3579a:	mov    rax,QWORD PTR [rsp+0x88]
   357a2:	adc    rax,r8
   357a5:	mov    QWORD PTR [rsp+0x308],rax
   357ad:	cmp    edi,0x6
   357b0:	je     35b7c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x179c>
   357b6:	bt     r9d,0x6
   357bb:	mov    rax,QWORD PTR [rsp+0x90]
   357c3:	adc    rax,r8
   357c6:	mov    QWORD PTR [rsp+0x328],rax
   357ce:	cmp    edi,0x7
   357d1:	je     35b94 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17b4>
   357d7:	bt     r9d,0x7
   357dc:	mov    rax,QWORD PTR [rsp+0x98]
   357e4:	adc    rax,r8
   357e7:	mov    QWORD PTR [rsp+0x348],rax
   357ef:	cmp    edi,0x8
   357f2:	je     35bac <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17cc>
   357f8:	bt     r9d,0x8
   357fd:	mov    rax,QWORD PTR [rsp+0xa0]
   35805:	adc    rax,r8
   35808:	mov    QWORD PTR [rsp+0x368],rax
   35810:	cmp    edi,0x9
   35813:	je     35bc4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17e4>
   35819:	bt     r9d,0x9
   3581e:	mov    rax,QWORD PTR [rsp+0xa8]
   35826:	adc    rax,r8
   35829:	mov    QWORD PTR [rsp+0x388],rax
   35831:	cmp    edi,0xa
   35834:	je     35bdc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17fc>
   3583a:	bt     r9d,0xa
   3583f:	mov    rax,QWORD PTR [rsp+0xb0]
   35847:	adc    rax,r8
   3584a:	mov    QWORD PTR [rsp+0x3a8],rax
   35852:	cmp    edi,0xb
   35855:	je     35bf4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1814>
   3585b:	bt     r9d,0xb
   35860:	mov    rax,QWORD PTR [rsp+0xb8]
   35868:	adc    rax,r8
   3586b:	mov    QWORD PTR [rsp+0x3c8],rax
   35873:	cmp    edi,0xc
   35876:	je     35c0c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x182c>
   3587c:	bt     r9d,0xc
   35881:	mov    rax,QWORD PTR [rsp+0xc0]
   35889:	adc    rax,r8
   3588c:	mov    QWORD PTR [rsp+0x3e8],rax
   35894:	cmp    edi,0xd
   35897:	je     35c24 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1844>
   3589d:	bt     r9d,0xd
   358a2:	mov    rax,QWORD PTR [rsp+0xc8]
   358aa:	adc    rax,r8
   358ad:	mov    QWORD PTR [rsp+0x408],rax
   358b5:	cmp    edi,0xe
   358b8:	je     35c3c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x185c>
   358be:	bt     r9d,0xe
   358c3:	mov    rax,QWORD PTR [rsp+0xd0]
   358cb:	adc    rax,r8
   358ce:	mov    QWORD PTR [rsp+0x428],rax
   358d6:	cmp    edi,0xf
   358d9:	je     35c54 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1874>
   358df:	bt     r9d,0xf
   358e4:	mov    rax,QWORD PTR [rsp+0xd8]
   358ec:	adc    rax,r8
   358ef:	mov    QWORD PTR [rsp+0x448],rax
   358f7:	cmp    edi,0x10
   358fa:	je     35c6c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x188c>
   35900:	bt     r9d,0x10
   35905:	mov    rax,QWORD PTR [rsp+0xe0]
   3590d:	adc    rax,r8
   35910:	mov    QWORD PTR [rsp+0x468],rax
   35918:	cmp    edi,0x11
   3591b:	je     35c84 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18a4>
   35921:	bt     r9d,0x11
   35926:	mov    rax,QWORD PTR [rsp+0xe8]
   3592e:	adc    rax,r8
   35931:	mov    QWORD PTR [rsp+0x488],rax
   35939:	cmp    edi,0x12
   3593c:	je     35c9c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18bc>
   35942:	bt     r9d,0x12
   35947:	mov    rax,QWORD PTR [rsp+0xf0]
   3594f:	adc    rax,r8
   35952:	mov    QWORD PTR [rsp+0x4a8],rax
   3595a:	cmp    edi,0x13
   3595d:	je     35cb4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d4>
   35963:	bt     r9d,0x13
   35968:	mov    rax,QWORD PTR [rsp+0xf8]
   35970:	adc    rax,r8
   35973:	mov    QWORD PTR [rsp+0x4c8],rax
   3597b:	cmp    edi,0x14
   3597e:	je     35ccc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ec>
   35984:	bt     r9d,0x14
   35989:	mov    rax,QWORD PTR [rsp+0x100]
   35991:	adc    rax,r8
   35994:	mov    QWORD PTR [rsp+0x4e8],rax
   3599c:	cmp    edi,0x15
   3599f:	je     35ce4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1904>
   359a5:	bt     r9d,0x15
   359aa:	mov    rax,QWORD PTR [rsp+0x108]
   359b2:	adc    rax,r8
   359b5:	mov    QWORD PTR [rsp+0x508],rax
   359bd:	cmp    edi,0x16
   359c0:	je     35cfc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191c>
   359c6:	bt     r9d,0x16
   359cb:	mov    rax,QWORD PTR [rsp+0x110]
   359d3:	adc    rax,r8
   359d6:	mov    QWORD PTR [rsp+0x528],rax
   359de:	cmp    edi,0x17
   359e1:	je     35d14 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1934>
   359e7:	bt     r9d,0x17
   359ec:	mov    rax,QWORD PTR [rsp+0x118]
   359f4:	adc    rax,r8
   359f7:	mov    QWORD PTR [rsp+0x548],rax
   359ff:	cmp    edi,0x18
   35a02:	je     35d2c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x194c>
   35a08:	bt     r9d,0x18
   35a0d:	mov    rax,QWORD PTR [rsp+0x120]
   35a15:	adc    rax,r8
   35a18:	mov    QWORD PTR [rsp+0x568],rax
   35a20:	cmp    edi,0x19
   35a23:	je     35d44 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1964>
   35a29:	bt     r9d,0x19
   35a2e:	mov    rax,QWORD PTR [rsp+0x128]
   35a36:	adc    rax,r8
   35a39:	mov    QWORD PTR [rsp+0x588],rax
   35a41:	cmp    edi,0x1a
   35a44:	je     35d5c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x197c>
   35a4a:	bt     r9d,0x1a
   35a4f:	mov    rax,QWORD PTR [rsp+0x130]
   35a57:	adc    rax,r8
   35a5a:	mov    QWORD PTR [rsp+0x5a8],rax
   35a62:	cmp    edi,0x1b
   35a65:	je     35d74 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1994>
   35a6b:	bt     r9d,0x1b
   35a70:	mov    rax,QWORD PTR [rsp+0x138]
   35a78:	adc    rax,r8
   35a7b:	mov    QWORD PTR [rsp+0x5c8],rax
   35a83:	cmp    edi,0x1c
   35a86:	je     35d8c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19ac>
   35a8c:	bt     r9d,0x1c
   35a91:	mov    rax,QWORD PTR [rsp+0x140]
   35a99:	adc    rax,r8
   35a9c:	mov    QWORD PTR [rsp+0x5e8],rax
   35aa4:	cmp    edi,0x1d
   35aa7:	je     35da4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19c4>
   35aad:	bt     r9d,0x1d
   35ab2:	mov    rax,QWORD PTR [rsp+0x148]
   35aba:	adc    rax,r8
   35abd:	mov    QWORD PTR [rsp+0x608],rax
   35ac5:	cmp    edi,0x1e
   35ac8:	je     35dbc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19dc>
   35ace:	bt     r9d,0x1e
   35ad3:	adc    r8,QWORD PTR [rsp+0x150]
   35adb:	mov    QWORD PTR [rsp+0x628],r8
   35ae3:	jmp    35dd3 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19f3>
   35ae8:	bt     r9d,0x1
   35aed:	mov    rax,QWORD PTR [rsp+0x68]
   35af2:	adc    rax,r8
   35af5:	mov    QWORD PTR [rsp+0x288],rax
   35afd:	mov    rbp,QWORD PTR [rsp+0x58]
   35b02:	mov    rcx,QWORD PTR [rsp+0x1d0]
   35b0a:	bt     r9d,0x2
   35b0f:	mov    rax,QWORD PTR [rsp+0x70]
   35b14:	adc    rax,r8
   35b17:	mov    QWORD PTR [rsp+0x2a8],rax
   35b1f:	bt     r9d,0x3
   35b24:	mov    rax,QWORD PTR [rsp+0x78]
   35b29:	adc    rax,r8
   35b2c:	mov    QWORD PTR [rsp+0x2c8],rax
   35b34:	bt     r9d,0x4
   35b39:	mov    rax,QWORD PTR [rsp+0x80]
   35b41:	adc    rax,r8
   35b44:	mov    QWORD PTR [rsp+0x2e8],rax
   35b4c:	bt     r9d,0x5
   35b51:	mov    rax,QWORD PTR [rsp+0x88]
   35b59:	adc    rax,r8
   35b5c:	mov    QWORD PTR [rsp+0x308],rax
   35b64:	bt     r9d,0x6
   35b69:	mov    rax,QWORD PTR [rsp+0x90]
   35b71:	adc    rax,r8
   35b74:	mov    QWORD PTR [rsp+0x328],rax
   35b7c:	bt     r9d,0x7
   35b81:	mov    rax,QWORD PTR [rsp+0x98]
   35b89:	adc    rax,r8
   35b8c:	mov    QWORD PTR [rsp+0x348],rax
   35b94:	bt     r9d,0x8
   35b99:	mov    rax,QWORD PTR [rsp+0xa0]
   35ba1:	adc    rax,r8
   35ba4:	mov    QWORD PTR [rsp+0x368],rax
   35bac:	bt     r9d,0x9
   35bb1:	mov    rax,QWORD PTR [rsp+0xa8]
   35bb9:	adc    rax,r8
   35bbc:	mov    QWORD PTR [rsp+0x388],rax
   35bc4:	bt     r9d,0xa
   35bc9:	mov    rax,QWORD PTR [rsp+0xb0]
   35bd1:	adc    rax,r8
   35bd4:	mov    QWORD PTR [rsp+0x3a8],rax
   35bdc:	bt     r9d,0xb
   35be1:	mov    rax,QWORD PTR [rsp+0xb8]
   35be9:	adc    rax,r8
   35bec:	mov    QWORD PTR [rsp+0x3c8],rax
   35bf4:	bt     r9d,0xc
   35bf9:	mov    rax,QWORD PTR [rsp+0xc0]
   35c01:	adc    rax,r8
   35c04:	mov    QWORD PTR [rsp+0x3e8],rax
   35c0c:	bt     r9d,0xd
   35c11:	mov    rax,QWORD PTR [rsp+0xc8]
   35c19:	adc    rax,r8
   35c1c:	mov    QWORD PTR [rsp+0x408],rax
   35c24:	bt     r9d,0xe
   35c29:	mov    rax,QWORD PTR [rsp+0xd0]
   35c31:	adc    rax,r8
   35c34:	mov    QWORD PTR [rsp+0x428],rax
   35c3c:	bt     r9d,0xf
   35c41:	mov    rax,QWORD PTR [rsp+0xd8]
   35c49:	adc    rax,r8
   35c4c:	mov    QWORD PTR [rsp+0x448],rax
   35c54:	bt     r9d,0x10
   35c59:	mov    rax,QWORD PTR [rsp+0xe0]
   35c61:	adc    rax,r8
   35c64:	mov    QWORD PTR [rsp+0x468],rax
   35c6c:	bt     r9d,0x11
   35c71:	mov    rax,QWORD PTR [rsp+0xe8]
   35c79:	adc    rax,r8
   35c7c:	mov    QWORD PTR [rsp+0x488],rax
   35c84:	bt     r9d,0x12
   35c89:	mov    rax,QWORD PTR [rsp+0xf0]
   35c91:	adc    rax,r8
   35c94:	mov    QWORD PTR [rsp+0x4a8],rax
   35c9c:	bt     r9d,0x13
   35ca1:	mov    rax,QWORD PTR [rsp+0xf8]
   35ca9:	adc    rax,r8
   35cac:	mov    QWORD PTR [rsp+0x4c8],rax
   35cb4:	bt     r9d,0x14
   35cb9:	mov    rax,QWORD PTR [rsp+0x100]
   35cc1:	adc    rax,r8
   35cc4:	mov    QWORD PTR [rsp+0x4e8],rax
   35ccc:	bt     r9d,0x15
   35cd1:	mov    rax,QWORD PTR [rsp+0x108]
   35cd9:	adc    rax,r8
   35cdc:	mov    QWORD PTR [rsp+0x508],rax
   35ce4:	bt     r9d,0x16
   35ce9:	mov    rax,QWORD PTR [rsp+0x110]
   35cf1:	adc    rax,r8
   35cf4:	mov    QWORD PTR [rsp+0x528],rax
   35cfc:	bt     r9d,0x17
   35d01:	mov    rax,QWORD PTR [rsp+0x118]
   35d09:	adc    rax,r8
   35d0c:	mov    QWORD PTR [rsp+0x548],rax
   35d14:	bt     r9d,0x18
   35d19:	mov    rax,QWORD PTR [rsp+0x120]
   35d21:	adc    rax,r8
   35d24:	mov    QWORD PTR [rsp+0x568],rax
   35d2c:	bt     r9d,0x19
   35d31:	mov    rax,QWORD PTR [rsp+0x128]
   35d39:	adc    rax,r8
   35d3c:	mov    QWORD PTR [rsp+0x588],rax
   35d44:	bt     r9d,0x1a
   35d49:	mov    rax,QWORD PTR [rsp+0x130]
   35d51:	adc    rax,r8
   35d54:	mov    QWORD PTR [rsp+0x5a8],rax
   35d5c:	bt     r9d,0x1b
   35d61:	mov    rax,QWORD PTR [rsp+0x138]
   35d69:	adc    rax,r8
   35d6c:	mov    QWORD PTR [rsp+0x5c8],rax
   35d74:	bt     r9d,0x1c
   35d79:	mov    rax,QWORD PTR [rsp+0x140]
   35d81:	adc    rax,r8
   35d84:	mov    QWORD PTR [rsp+0x5e8],rax
   35d8c:	bt     r9d,0x1d
   35d91:	mov    rax,QWORD PTR [rsp+0x148]
   35d99:	adc    rax,r8
   35d9c:	mov    QWORD PTR [rsp+0x608],rax
   35da4:	bt     r9d,0x1e
   35da9:	mov    rax,QWORD PTR [rsp+0x150]
   35db1:	adc    rax,r8
   35db4:	mov    QWORD PTR [rsp+0x628],rax
   35dbc:	shr    r9d,0x1f
   35dc0:	add    r8,QWORD PTR [rsp+0x158]
   35dc8:	add    r8,r9
   35dcb:	mov    QWORD PTR [rsp+0x648],r8
   35dd3:	shl    rcx,0x5
   35dd7:	lea    r15,[rsp+rcx*1]
   35ddb:	add    r15,0x260
   35de2:	lea    r14,[rsp+0x260]
   35dea:	mov    rax,QWORD PTR [rsp+0x18]
   35def:	jmp    35e0d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a2d>
   35df1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35e00:	mov    r14,r13
   35e03:	add    r12,0xffffffffffffffe0
   35e07:	je     35f8b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bab>
   35e0d:	lea    r13,[r14+0x20]
   35e11:	mov    rbx,QWORD PTR [r14]
   35e14:	cmp    rbx,0xffffffffffffffff
   35e18:	je     35e00 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a20>
   35e1a:	cmp    rbx,QWORD PTR [rsp+0x20]
   35e1f:	jae    36545 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2165>
   35e25:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   35e2a:	jne    35e00 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a20>
   35e2c:	mov    edi,0x80
   35e31:	call   QWORD PTR [rip+0x601b1]        # 95fe8 <malloc@GLIBC_2.2.5>
   35e37:	test   rax,rax
   35e3a:	je     36594 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21b4>
   35e40:	mov    rcx,QWORD PTR [r14+0x18]
   35e44:	mov    QWORD PTR [rax+0x18],rcx
   35e48:	movups xmm0,XMMWORD PTR [r14+0x8]
   35e4d:	movups XMMWORD PTR [rax+0x8],xmm0
   35e51:	mov    QWORD PTR [rax],rbx
   35e54:	mov    QWORD PTR [rsp+0x168],0x4
   35e60:	mov    QWORD PTR [rsp+0x170],rax
   35e68:	mov    QWORD PTR [rsp+0x178],0x1
   35e74:	mov    r12d,0x1
   35e7a:	cmp    r13,r15
   35e7d:	je     35f47 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b67>
   35e83:	lea    r14,[rsp+0x168]
   35e8b:	mov    rdx,QWORD PTR [rsp+0x30]
   35e90:	mov    rcx,QWORD PTR [rdx+0x50]
   35e94:	mov    rsi,QWORD PTR [rdx+0x58]
   35e98:	mov    rdx,r13
   35e9b:	jmp    35eac <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1acc>
   35e9d:	nop    DWORD PTR [rax]
   35ea0:	mov    rdx,r13
   35ea3:	cmp    r13,r15
   35ea6:	je     35f47 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b67>
   35eac:	lea    r13,[rdx+0x20]
   35eb0:	mov    rbx,QWORD PTR [rdx]
   35eb3:	cmp    rbx,0xffffffffffffffff
   35eb7:	je     35ea0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ac0>
   35eb9:	cmp    rbx,rsi
   35ebc:	jae    36568 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2188>
   35ec2:	cmp    QWORD PTR [rcx+rbx*8],0xffffffffffffffff
   35ec7:	jne    35ea0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ac0>
   35ec9:	mov    rcx,QWORD PTR [rdx+0x18]
   35ecd:	mov    QWORD PTR [rsp+0x670],rcx
   35ed5:	movups xmm0,XMMWORD PTR [rdx+0x8]
   35ed9:	movaps XMMWORD PTR [rsp+0x660],xmm0
   35ee1:	cmp    r12,QWORD PTR [rsp+0x168]
   35ee9:	jne    35f0e <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b2e>
   35eeb:	mov    edx,0x1
   35ef0:	mov    ecx,0x8
   35ef5:	mov    r8d,0x20
   35efb:	mov    rdi,r14
   35efe:	mov    rsi,r12
   35f01:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   35f06:	mov    rax,QWORD PTR [rsp+0x170]
   35f0e:	mov    rcx,r12
   35f11:	shl    rcx,0x5
   35f15:	mov    QWORD PTR [rax+rcx*1],rbx
   35f19:	movaps xmm0,XMMWORD PTR [rsp+0x660]
   35f21:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   35f26:	mov    rdx,QWORD PTR [rsp+0x670]
   35f2e:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   35f33:	inc    r12
   35f36:	mov    QWORD PTR [rsp+0x178],r12
   35f3e:	cmp    r13,r15
   35f41:	jne    35e8b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1aab>
   35f47:	mov    QWORD PTR [rsp+0x58],rbp
   35f4c:	mov    rax,QWORD PTR [rsp+0x168]
   35f54:	mov    QWORD PTR [rsp+0x18],rax
   35f59:	mov    rax,QWORD PTR [rsp+0x170]
   35f61:	mov    QWORD PTR [rsp+0x8],rax
   35f66:	mov    r15,r12
   35f69:	shl    r15,0x5
   35f6d:	mov    rax,QWORD PTR [rsp+0x30]
   35f72:	mov    r14,QWORD PTR [rax+0x38]
   35f76:	mov    rbp,QWORD PTR [rax+0x40]
   35f7a:	xor    r13d,r13d
   35f7d:	lea    rax,[rip+0x5de7c]        # 93e00 <__frame_dummy_init_array_entry+0xb48>
   35f84:	mov    QWORD PTR [rsp+0x40],rax
   35f89:	jmp    35fee <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c0e>
   35f8b:	mov    eax,0x8
   35f90:	mov    QWORD PTR [rsp+0x8],rax
   35f95:	xor    r14d,r14d
   35f98:	mov    r9,QWORD PTR [rsp+0x38]
   35f9d:	mov    r8,QWORD PTR [rsp+0x10]
   35fa2:	jmp    36229 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e49>
   35fa7:	mov    rdi,r12
   35faa:	mov    r14,r9
   35fad:	call   36730 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsifwKkFJZ2U4_5mibz2>
   35fb2:	mov    r9,r14
   35fb5:	mov    esi,DWORD PTR [rsp+0x4c]
   35fb9:	mov    r14,QWORD PTR [r12+0x8]
   35fbe:	jmp    3449d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbd>
   35fc3:	mov    rax,QWORD PTR [rsp]
   35fc7:	mov    rdx,QWORD PTR [rax]
   35fca:	mov    rax,QWORD PTR [rax+0x8]
   35fce:	xchg   ax,ax
   35fd0:	add    rbx,rax
   35fd3:	mov    rcx,QWORD PTR [rsp+0x8]
   35fd8:	mov    QWORD PTR [rcx+r13*1+0x8],rbx
   35fdd:	add    rdx,rax
   35fe0:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   35fe5:	add    r13,0x20
   35fe9:	cmp    r15,r13
   35fec:	je     36051 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c71>
   35fee:	mov    rax,QWORD PTR [rsp+0x8]
   35ff3:	mov    rbx,QWORD PTR [rax+r13*1+0x8]
   35ff8:	cmp    rbx,QWORD PTR [rax+r13*1+0x10]
   35ffd:	jne    35fe5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c05>
   35fff:	mov    rax,QWORD PTR [rsp+0x8]
   36004:	mov    rdi,QWORD PTR [rax+r13*1]
   36008:	cmp    rdi,rbp
   3600b:	jae    364ba <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20da>
   36011:	lea    rcx,[rdi+rdi*2]
   36015:	mov    rdx,QWORD PTR [r14+rcx*8]
   36019:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   3601e:	sub    rbx,rax
   36021:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   36026:	mov    rsi,rdx
   36029:	sub    rsi,rbx
   3602c:	cmp    rdx,rsi
   3602f:	jbe    35fd0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1bf0>
   36031:	lea    rdi,[r14+rcx*8]
   36035:	mov    ecx,0x1
   3603a:	mov    r8d,0x1
   36040:	mov    QWORD PTR [rsp],rdi
   36044:	mov    rsi,rbx
   36047:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   3604c:	jmp    35fc3 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1be3>
   36051:	mov    rax,QWORD PTR [rsp+0x30]
   36056:	mov    rcx,QWORD PTR [rax+0x50]
   3605a:	mov    QWORD PTR [rsp+0x20],rcx
   3605f:	mov    rcx,QWORD PTR [rax+0x58]
   36063:	mov    QWORD PTR [rsp+0x28],rcx
   36068:	mov    rcx,QWORD PTR [rax+0x8]
   3606c:	mov    QWORD PTR [rsp+0x10],rcx
   36071:	mov    rax,QWORD PTR [rax+0x10]
   36075:	mov    QWORD PTR [rsp],rax
   36079:	xor    r15d,r15d
   3607c:	mov    r9,QWORD PTR [rsp+0x38]
   36081:	jmp    36093 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   36083:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   36090:	xor    r15d,r15d
   36093:	mov    r13,r15
   36096:	shl    r13,0x5
   3609a:	add    r13,QWORD PTR [rsp+0x8]
   3609f:	jmp    360cf <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cef>
   360a1:	mov    rax,QWORD PTR [rsp+0x50]
   360a6:	mov    rdx,QWORD PTR [rax]
   360a9:	mov    rax,QWORD PTR [rax+0x8]
   360ad:	nop    DWORD PTR [rax]
   360b0:	add    rbx,rax
   360b3:	mov    QWORD PTR [r13+0x8],rbx
   360b7:	add    rdx,rax
   360ba:	mov    QWORD PTR [r13+0x10],rdx
   360be:	inc    r15
   360c1:	add    r13,0x20
   360c5:	cmp    r15,r12
   360c8:	mov    r9,QWORD PTR [rsp+0x38]
   360cd:	jae    36090 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb0>
   360cf:	mov    eax,DWORD PTR [r13+0x18]
   360d3:	mov    ecx,DWORD PTR [r9+rax*4]
   360d7:	test   ecx,ecx
   360d9:	js     36150 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1d70>
   360db:	mov    eax,ecx
   360dd:	shr    eax,0x8
   360e0:	mov    DWORD PTR [r13+0x18],eax
   360e4:	prefetcht0 BYTE PTR [r9+rax*4]
   360e9:	mov    rax,QWORD PTR [r13+0x8]
   360ed:	mov    BYTE PTR [rax],cl
   360ef:	mov    rbx,QWORD PTR [r13+0x8]
   360f3:	inc    rbx
   360f6:	mov    QWORD PTR [r13+0x8],rbx
   360fa:	cmp    rbx,QWORD PTR [r13+0x10]
   360fe:	jne    360be <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cde>
   36100:	mov    rdi,QWORD PTR [r13+0x0]
   36104:	cmp    rdi,rbp
   36107:	jae    364ba <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20da>
   3610d:	lea    rcx,[rdi+rdi*2]
   36111:	mov    rdx,QWORD PTR [r14+rcx*8]
   36115:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   3611a:	sub    rbx,rax
   3611d:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   36122:	mov    rsi,rdx
   36125:	sub    rsi,rbx
   36128:	cmp    rdx,rsi
   3612b:	jbe    360b0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cd0>
   3612d:	lea    rdi,[r14+rcx*8]
   36131:	mov    ecx,0x1
   36136:	mov    r8d,0x1
   3613c:	mov    QWORD PTR [rsp+0x50],rdi
   36141:	mov    rsi,rbx
   36144:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   36149:	jmp    360a1 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cc1>
   3614e:	xchg   ax,ax
   36150:	mov    rdi,QWORD PTR [r13+0x0]
   36154:	cmp    rdi,rbp
   36157:	jae    364a0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20c0>
   3615d:	lea    rcx,[rdi+rdi*2]
   36161:	mov    rdx,QWORD PTR [r13+0x8]
   36165:	sub    rdx,QWORD PTR [r14+rcx*8+0x8]
   3616a:	mov    QWORD PTR [r14+rcx*8+0x10],rdx
   3616f:	mov    rdx,QWORD PTR [rsp]
   36173:	test   rdx,rdx
   36176:	je     36187 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1da7>
   36178:	xor    ecx,ecx
   3617a:	cmp    rdx,0x1
   3617e:	jne    36190 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1db0>
   36180:	mov    r10,QWORD PTR [rsp+0x10]
   36185:	jmp    361bd <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ddd>
   36187:	mov    rcx,0xffffffffffffffff
   3618e:	jmp    361cc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dec>
   36190:	mov    r10,QWORD PTR [rsp+0x10]
   36195:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   361a0:	mov    rsi,rcx
   361a3:	mov    r8,rdx
   361a6:	shr    r8,1
   361a9:	add    rcx,r8
   361ac:	cmp    DWORD PTR [r10+rcx*4],eax
   361b0:	cmova  rcx,rsi
   361b4:	sub    rdx,r8
   361b7:	cmp    rdx,0x1
   361bb:	ja     361a0 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1dc0>
   361bd:	cmp    DWORD PTR [r10+rcx*4],eax
   361c1:	mov    rax,0xffffffffffffffff
   361c8:	cmovne rcx,rax
   361cc:	cmp    rdi,QWORD PTR [rsp+0x28]
   361d1:	jae    364a9 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20c9>
   361d7:	mov    rax,QWORD PTR [rsp+0x20]
   361dc:	mov    QWORD PTR [rax+rdi*8],rcx
   361e0:	dec    r12
   361e3:	mov    rax,r12
   361e6:	shl    rax,0x5
   361ea:	mov    rcx,QWORD PTR [rsp+0x8]
   361ef:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   361f3:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   361f8:	movups XMMWORD PTR [r13+0x10],xmm1
   361fd:	movups XMMWORD PTR [r13+0x0],xmm0
   36202:	cmp    r15,r12
   36205:	jb     36093 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   3620b:	mov    r15d,0x0
   36211:	test   r12,r12
   36214:	jne    36093 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1cb3>
   3621a:	mov    rbp,QWORD PTR [rsp+0x58]
   3621f:	mov    r8,QWORD PTR [rsp+0x10]
   36224:	mov    r14,QWORD PTR [rsp+0x18]
   36229:	mov    rax,0xffffffffffffffff
   36230:	mov    r10,QWORD PTR [rsp]
   36234:	test   r10,r10
   36237:	mov    rsi,QWORD PTR [rsp+0x30]
   3623c:	je     362bc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1edc>
   3623e:	lea    rcx,[r10*4+0x0]
   36246:	xor    edx,edx
   36248:	nop    DWORD PTR [rax+rax*1+0x0]
   36250:	mov    edi,DWORD PTR [r8+rdx*1]
   36254:	cmp    rbp,rdi
   36257:	jbe    364d1 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20f1>
   3625d:	and    BYTE PTR [r9+rdi*4+0x3],0x7f
   36263:	add    rdx,0x4
   36267:	cmp    rcx,rdx
   3626a:	jne    36250 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e70>
   3626c:	test   r10,r10
   3626f:	je     362bc <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1edc>
   36271:	xor    ecx,ecx
   36273:	cmp    r10,0x1
   36277:	jne    3627f <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1e9f>
   36279:	mov    edi,DWORD PTR [rsp+0x4c]
   3627d:	jmp    362ad <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ecd>
   3627f:	mov    edi,DWORD PTR [rsp+0x4c]
   36283:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   36290:	mov    rax,rcx
   36293:	mov    rdx,r10
   36296:	shr    rdx,1
   36299:	add    rcx,rdx
   3629c:	cmp    DWORD PTR [r8+rcx*4],edi
   362a0:	cmova  rcx,rax
   362a4:	sub    r10,rdx
   362a7:	cmp    r10,0x1
   362ab:	ja     36290 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1eb0>
   362ad:	cmp    DWORD PTR [r8+rcx*4],edi
   362b1:	mov    rax,0xffffffffffffffff
   362b8:	cmove  rax,rcx
   362bc:	mov    QWORD PTR [rsi+0x68],rax
   362c0:	mov    QWORD PTR [rsi+0x60],rbp
   362c4:	test   r14,r14
   362c7:	je     362d4 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ef4>
   362c9:	mov    rdi,QWORD PTR [rsp+0x8]
   362ce:	call   QWORD PTR [rip+0x5fd1c]        # 95ff0 <free@GLIBC_2.2.5>
   362d4:	add    rsp,0x678
   362db:	pop    rbx
   362dc:	pop    r12
   362de:	pop    r13
   362e0:	pop    r14
   362e2:	pop    r15
   362e4:	pop    rbp
   362e5:	ret
   362e6:	mov    edx,0x1ff
   362eb:	mov    ecx,0x4
   362f0:	mov    r8d,0x4
   362f6:	mov    rbx,QWORD PTR [rsp+0x30]
   362fb:	mov    r14d,esi
   362fe:	mov    rdi,rbx
   36301:	xor    esi,esi
   36303:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   36308:	mov    esi,r14d
   3630b:	mov    r9,QWORD PTR [rbx+0x10]
   3630f:	jmp    34431 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x51>
   36314:	mov    rsi,r13
   36317:	cmp    r13,0x15
   3631b:	jae    365a3 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21c3>
   36321:	lea    rax,[r14+rsi*4]
   36325:	lea    rcx,[r14+0x4]
   36329:	mov    edx,0x4
   3632e:	jmp    36346 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f66>
   36330:	mov    rdi,r14
   36333:	mov    DWORD PTR [rdi],esi
   36335:	add    rcx,0x4
   36339:	add    rdx,0x4
   3633d:	cmp    rcx,rax
   36340:	je     365b1 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21d1>
   36346:	mov    r8d,DWORD PTR [rcx-0x4]
   3634a:	mov    esi,DWORD PTR [rcx]
   3634c:	cmp    esi,r8d
   3634f:	jae    36335 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f55>
   36351:	mov    rdi,rdx
   36354:	mov    DWORD PTR [r14+rdi*1],r8d
   36358:	cmp    rdi,0x4
   3635c:	je     36330 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f50>
   3635e:	mov    r8d,DWORD PTR [r14+rdi*1-0x8]
   36363:	add    rdi,0xfffffffffffffffc
   36367:	cmp    esi,r8d
   3636a:	jb     36354 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f74>
   3636c:	add    rdi,r14
   3636f:	jmp    36333 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f53>
   36371:	lea    rdi,[r12+0x48]
   36376:	mov    ecx,0x8
   3637b:	mov    r8d,0x8
   36381:	xor    esi,esi
   36383:	mov    rdx,r13
   36386:	mov    rbx,rax
   36389:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   3638e:	mov    rax,rbx
   36391:	mov    rbx,QWORD PTR [r12+0x58]
   36396:	mov    r14,QWORD PTR [r12+0x50]
   3639b:	lea    rdi,[r14+rbx*8]
   3639f:	cmp    r13,0x1
   363a3:	jne    34600 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x220>
   363a9:	jmp    3462b <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x24b>
   363ae:	lea    rdi,[r12+0x30]
   363b3:	mov    ecx,0x8
   363b8:	mov    r8d,0x18
   363be:	mov    rsi,r13
   363c1:	mov    rdx,r14
   363c4:	mov    rbx,rax
   363c7:	call   366b0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifwKkFJZ2U4_5mibz2>
   363cc:	mov    rax,rbx
   363cf:	mov    rdx,QWORD PTR [r12+0x40]
   363d4:	mov    rsi,QWORD PTR [r12+0x38]
   363d9:	inc    r13
   363dc:	test   r14b,0x1
   363e0:	je     3454e <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16e>
   363e6:	dec    r14
   363e9:	lea    rcx,[rdx+rdx*2]
   363ed:	mov    QWORD PTR [rsi+rcx*8],0x0
   363f5:	mov    QWORD PTR [rsi+rcx*8+0x8],0x1
   363fe:	mov    QWORD PTR [rsi+rcx*8+0x10],0x0
   36407:	inc    rdx
   3640a:	mov    rcx,QWORD PTR [rsp+0x40]
   3640f:	cmp    rcx,r13
   36412:	mov    r13,rcx
   36415:	je     3455f <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17f>
   3641b:	lea    rcx,[rdx+rdx*2]
   3641f:	lea    rsi,[rsi+rcx*8]
   36423:	add    rsi,0x28
   36427:	neg    r14
   3642a:	xor    ecx,ecx
   3642c:	xorps  xmm0,xmm0
   3642f:	nop
   36430:	mov    QWORD PTR [rsi-0x28],0x0
   36438:	mov    QWORD PTR [rsi-0x20],0x1
   36440:	movups XMMWORD PTR [rsi-0x18],xmm0
   36444:	mov    QWORD PTR [rsi-0x8],0x1
   3644c:	mov    QWORD PTR [rsi],0x0
   36453:	add    rsi,0x30
   36457:	add    rcx,0xfffffffffffffffe
   3645b:	cmp    r14,rcx
   3645e:	jne    36430 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2050>
   36460:	sub    rdx,rcx
   36463:	mov    QWORD PTR [r12+0x40],rdx
   36468:	cmp    r13,rdx
   3646b:	jbe    3456d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d>
   36471:	jmp    356af <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x12cf>
   36476:	lea    rdx,[rip+0x5d6fb]        # 93b78 <__frame_dummy_init_array_entry+0x8c0>
   3647d:	mov    rsi,rbp
   36480:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36485:	lea    rdx,[rip+0x5d9a4]        # 93e30 <__frame_dummy_init_array_entry+0xb78>
   3648c:	mov    rsi,rbx
   3648f:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36494:	lea    rdx,[rip+0x5d9ad]        # 93e48 <__frame_dummy_init_array_entry+0xb90>
   3649b:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364a0:	lea    rax,[rip+0x5d929]        # 93dd0 <__frame_dummy_init_array_entry+0xb18>
   364a7:	jmp    364b5 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x20d5>
   364a9:	mov    rbp,QWORD PTR [rsp+0x28]
   364ae:	lea    rax,[rip+0x5d933]        # 93de8 <__frame_dummy_init_array_entry+0xb30>
   364b5:	mov    QWORD PTR [rsp+0x40],rax
   364ba:	mov    rsi,rbp
   364bd:	mov    rdx,QWORD PTR [rsp+0x40]
   364c2:	mov    r14,QWORD PTR [rsp+0x18]
   364c7:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364cc:	jmp    36577 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2197>
   364d1:	lea    rdx,[rip+0x5d6e8]        # 93bc0 <__frame_dummy_init_array_entry+0x908>
   364d8:	mov    rsi,rbp
   364db:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364e0:	jmp    36577 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2197>
   364e5:	lea    rdx,[rip+0x5d8fc]        # 93de8 <__frame_dummy_init_array_entry+0xb30>
   364ec:	mov    rdi,rax
   364ef:	mov    rsi,QWORD PTR [rsp+0x20]
   364f4:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   364f9:	lea    rdx,[rip+0x5d8d0]        # 93dd0 <__frame_dummy_init_array_entry+0xb18>
   36500:	mov    rdi,rax
   36503:	mov    rsi,QWORD PTR [rsp+0x28]
   36508:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3650d:	lea    rdx,[rip+0x5d694]        # 93ba8 <__frame_dummy_init_array_entry+0x8f0>
   36514:	mov    esi,0x20
   36519:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3651e:	lea    rdx,[rip+0x5d923]        # 93e48 <__frame_dummy_init_array_entry+0xb90>
   36525:	mov    rdi,rbp
   36528:	mov    rsi,QWORD PTR [rsp]
   3652c:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36531:	lea    rdx,[rip+0x5d8f8]        # 93e30 <__frame_dummy_init_array_entry+0xb78>
   36538:	mov    rdi,rbp
   3653b:	mov    rsi,QWORD PTR [rsp+0x28]
   36540:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36545:	lea    rdx,[rip+0x5d8cc]        # 93e18 <__frame_dummy_init_array_entry+0xb60>
   3654c:	mov    rdi,rbx
   3654f:	mov    rsi,QWORD PTR [rsp+0x20]
   36554:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36559:	lea    rdx,[rip+0x5d8a0]        # 93e00 <__frame_dummy_init_array_entry+0xb48>
   36560:	mov    rsi,rbx
   36563:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36568:	lea    rdx,[rip+0x5d8a9]        # 93e18 <__frame_dummy_init_array_entry+0xb60>
   3656f:	mov    rdi,rbx
   36572:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36577:	ud2
   36579:	lea    rdx,[rip+0x5d5e0]        # 93b60 <__frame_dummy_init_array_entry+0x8a8>
   36580:	mov    rsi,rbp
   36583:	call   2af8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36588:	lea    rdi,[rip+0x5d601]        # 93b90 <__frame_dummy_init_array_entry+0x8d8>
   3658f:	call   2d980 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   36594:	mov    edi,0x8
   36599:	mov    esi,0x80
   3659e:	call   297eb <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   365a3:	mov    rdi,r14
   365a6:	mov    r15,r9
   365a9:	call   367a0 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECsifwKkFJZ2U4_5mibz2>
   365ae:	mov    r9,r15
   365b1:	xor    ecx,ecx
   365b3:	mov    eax,DWORD PTR [r14+rcx*4+0x4]
   365b8:	cmp    eax,DWORD PTR [r14+rcx*4]
   365bc:	je     365cb <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21eb>
   365be:	inc    rcx
   365c1:	cmp    rbx,rcx
   365c4:	jne    365b3 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x21d3>
   365c6:	jmp    344b7 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7>
   365cb:	lea    rdx,[rcx+0x1]
   365cf:	lea    rax,[rcx+0x2]
   365d3:	cmp    rax,r13
   365d6:	jae    365ec <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x220c>
   365d8:	mov    esi,r9d
   365db:	sub    esi,ecx
   365dd:	add    r9,0x1fd
   365e4:	test   sil,0x1
   365e8:	jne    365f1 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2211>
   365ea:	jmp    36615 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2235>
   365ec:	mov    r13,rdx
   365ef:	jmp    3661a <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x223a>
   365f1:	mov    eax,DWORD PTR [r14+rcx*4+0x8]
   365f6:	mov    rsi,rdx
   365f9:	cmp    eax,DWORD PTR [r14+rcx*4]
   365fd:	je     36608 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2228>
   365ff:	mov    DWORD PTR [r14+rcx*4+0x4],eax
   36604:	lea    rsi,[rdx+0x1]
   36608:	add    rdx,0x2
   3660c:	mov    rax,rdx
   3660f:	mov    r13,rsi
   36612:	mov    rdx,rsi
   36615:	cmp    r9,rcx
   36618:	jne    36624 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2244>
   3661a:	mov    QWORD PTR [r12+0x10],r13
   3661f:	jmp    344b7 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd7>
   36624:	inc    rbx
   36627:	mov    r13,rdx
   3662a:	jmp    36635 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2255>
   3662c:	add    rax,0x2
   36630:	cmp    rbx,rax
   36633:	je     3661a <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x223a>
   36635:	mov    ecx,DWORD PTR [r14+rax*4]
   36639:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3663e:	jne    3664e <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x226e>
   36640:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   36645:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3664a:	je     3662c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   3664c:	jmp    36661 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2281>
   3664e:	mov    DWORD PTR [r14+r13*4],ecx
   36652:	inc    r13
   36655:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   3665a:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3665f:	je     3662c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   36661:	mov    DWORD PTR [r14+r13*4],ecx
   36665:	inc    r13
   36668:	jmp    3662c <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x224c>
   3666a:	jmp    3666e <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x228e>
   3666c:	jmp    36678 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x2298>
   3666e:	mov    rbx,rax
   36671:	mov    r14,QWORD PTR [rsp+0x18]
   36676:	jmp    36698 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22b8>
   36678:	mov    rbx,rax
   3667b:	cmp    QWORD PTR [rsp+0x168],0x0
   36684:	je     366a8 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22c8>
   36686:	mov    rax,QWORD PTR [rsp+0x170]
   3668e:	mov    QWORD PTR [rsp+0x8],rax
   36693:	jmp    3669d <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22bd>
   36695:	mov    rbx,rax
   36698:	test   r14,r14
   3669b:	je     366a8 <_RINvNtCsifwKkFJZ2U4_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x22c8>
   3669d:	mov    rdi,QWORD PTR [rsp+0x8]
   366a2:	call   QWORD PTR [rip+0x5f948]        # 95ff0 <free@GLIBC_2.2.5>
   366a8:	mov    rdi,rbx
   366ab:	call   92270 <_Unwind_Resume@plt>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
