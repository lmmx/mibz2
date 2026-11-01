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
   ...
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
