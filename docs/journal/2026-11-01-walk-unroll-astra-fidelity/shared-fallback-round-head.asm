   169f1:	mov    r10,QWORD PTR [rsp+0x588]
   169f9:	mov    QWORD PTR [rsp+0x130],r10
   16a01:	mov    r9,QWORD PTR [rsp+0x5a8]
   16a09:	mov    QWORD PTR [rsp+0x138],r9
   16a11:	mov    r8,QWORD PTR [rsp+0x5c8]
   16a19:	mov    QWORD PTR [rsp+0x140],r8
   16a21:	mov    rdi,QWORD PTR [rsp+0x5e8]
   16a29:	mov    QWORD PTR [rsp+0x148],rdi
   16a31:	test   r14,r14
   16a34:	je     16a50 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xa50>
   16a36:	xor    r11d,r11d
   16a39:	mov    rbx,QWORD PTR [rsp]
   16a3d:	mov    rbp,QWORD PTR [rsp+0x10]
   16a42:	lea    r12,[rsp+0x218]
   16a4a:	jmp    16ab9 <mibz2::bwt::room_segments_unrolled::<32, 1>+0xab9>
   16a4c:	nop    DWORD PTR [rax+0x0]
   16a50:	mov    QWORD PTR [rsp+0x190],rbp
   16a58:	mov    rbp,QWORD PTR [rsp+0x10]
   16a5d:	mov    QWORD PTR [rsp+0x1a0],r15
   16a65:	mov    r15,QWORD PTR [rsp+0x608]
   16a6d:	mov    QWORD PTR [rsp+0x198],r12
   16a75:	jmp    1751f <mibz2::bwt::room_segments_unrolled::<32, 1>+0x151f>
   16a7a:	mov    ecx,eax
   16a7c:	shr    ecx,0x8
   16a7f:	mov    DWORD PTR [rsp+0x5f8],ecx
   16a86:	mov    rbx,QWORD PTR [rsp]
   16a8a:	prefetcht0 BYTE PTR [rbx+rcx*4]
   16a8e:	mov    rcx,QWORD PTR [rsp+0x148]
   16a96:	mov    BYTE PTR [rcx+rdx*1],al
   16a99:	mov    rbp,QWORD PTR [rsp+0x10]
   16a9e:	lea    r12,[rsp+0x218]
   16aa6:	cs nop WORD PTR [rax+rax*1+0x0]
   16ab0:	cmp    r11,r14
   16ab3:	je     173a0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x13a0>
   16ab9:	mov    rsi,r11
   16abc:	inc    r11
   16abf:	mov    eax,DWORD PTR [rsp+0x218]
   16ac6:	mov    eax,DWORD PTR [rbx+rax*4]
   16ac9:	test   eax,eax
   16acb:	js     17050 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1050>
   16ad1:	mov    ecx,eax
   16ad3:	shr    ecx,0x8
   16ad6:	mov    DWORD PTR [rsp+0x218],ecx
   16add:	prefetcht0 BYTE PTR [rbx+rcx*4]
   16ae1:	mov    rcx,QWORD PTR [rsp+0x50]
   16ae6:	mov    BYTE PTR [rcx+rsi*1],al
   16ae9:	mov    eax,DWORD PTR [rsp+0x238]
   16af0:	mov    eax,DWORD PTR [rbx+rax*4]
   16af3:	test   eax,eax
   16af5:	js     17060 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1060>
   16afb:	mov    ecx,eax
   16afd:	shr    ecx,0x8
   16b00:	mov    DWORD PTR [rsp+0x238],ecx
   16b07:	prefetcht0 BYTE PTR [rbx+rcx*4]
   16b0b:	mov    rcx,QWORD PTR [rsp+0x58]
   16b10:	mov    BYTE PTR [rcx+rsi*1],al
   16b13:	mov    eax,DWORD PTR [rsp+0x258]
   16b1a:	mov    eax,DWORD PTR [rbx+rax*4]
   16b1d:	test   eax,eax
   16b1f:	js     1706b <mibz2::bwt::room_segments_unrolled::<32, 1>+0x106b>
   16b25:	mov    ecx,eax
   16b27:	shr    ecx,0x8
   16b2a:	mov    DWORD PTR [rsp+0x258],ecx
   16b31:	prefetcht0 BYTE PTR [rbx+rcx*4]
   16b35:	mov    rcx,QWORD PTR [rsp+0x60]
   16b3a:	mov    BYTE PTR [rcx+rsi*1],al
   16f3c:	mov    BYTE PTR [rcx+rsi*1],al
   16f3f:	mov    eax,DWORD PTR [rsp+0x558]
   16f46:	mov    eax,DWORD PTR [rbx+rax*4]
   16f49:	test   eax,eax
   16f4b:	js     17161 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1161>
   16f51:	mov    ecx,eax
   16f53:	shr    ecx,0x8
   16f56:	mov    DWORD PTR [rsp+0x558],ecx
   16f5d:	prefetcht0 BYTE PTR [rbx+rcx*4]
   16f61:	mov    rcx,QWORD PTR [rsp+0x120]
   16f69:	mov    rdx,rsi
   16f6c:	mov    BYTE PTR [rcx+rsi*1],al
   16f6f:	mov    eax,DWORD PTR [rsp+0x578]
   16f76:	mov    eax,DWORD PTR [rbx+rax*4]
   16f79:	test   eax,eax
   16f7b:	js     17176 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1176>
   16f81:	mov    ecx,eax
   16f83:	shr    ecx,0x8
   16f86:	mov    DWORD PTR [rsp+0x578],ecx
   16f8d:	mov    rsi,QWORD PTR [rsp]
   16f91:	prefetcht0 BYTE PTR [rsi+rcx*4]
   16f95:	mov    rcx,QWORD PTR [rsp+0x128]
   16f9d:	mov    BYTE PTR [rcx+rdx*1],al
   16fa0:	mov    eax,DWORD PTR [rsp+0x598]
   16fa7:	mov    eax,DWORD PTR [rsi+rax*4]
   16faa:	test   eax,eax
   16fac:	js     1717e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x117e>
   16fb2:	mov    ecx,eax
   16fb4:	shr    ecx,0x8
   16fb7:	mov    DWORD PTR [rsp+0x598],ecx
   16fbe:	mov    rsi,QWORD PTR [rsp]
   16fc2:	prefetcht0 BYTE PTR [rsi+rcx*4]
   16fc6:	mov    rcx,QWORD PTR [rsp+0x130]
   16fce:	mov    BYTE PTR [rcx+rdx*1],al
   16fd1:	mov    eax,DWORD PTR [rsp+0x5b8]
   16fd8:	mov    eax,DWORD PTR [rsi+rax*4]
   16fdb:	test   eax,eax
   16fdd:	js     17186 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1186>
   16fe3:	mov    ecx,eax
   16fe5:	shr    ecx,0x8
   16fe8:	mov    DWORD PTR [rsp+0x5b8],ecx
   16fef:	mov    rsi,QWORD PTR [rsp]
   16ff3:	prefetcht0 BYTE PTR [rsi+rcx*4]
   16ff7:	mov    rcx,QWORD PTR [rsp+0x138]
   16fff:	mov    BYTE PTR [rcx+rdx*1],al
   17002:	mov    eax,DWORD PTR [rsp+0x5d8]
   17009:	mov    eax,DWORD PTR [rsi+rax*4]
   1700c:	test   eax,eax
   1700e:	js     1718e <mibz2::bwt::room_segments_unrolled::<32, 1>+0x118e>
   17014:	mov    ecx,eax
   17016:	shr    ecx,0x8
   17019:	mov    DWORD PTR [rsp+0x5d8],ecx
   17020:	mov    rsi,QWORD PTR [rsp]
   17024:	prefetcht0 BYTE PTR [rsi+rcx*4]
   17028:	mov    rcx,QWORD PTR [rsp+0x140]
   17030:	mov    BYTE PTR [rcx+rdx*1],al
   17033:	mov    eax,DWORD PTR [rsp+0x5f8]
   1703a:	mov    eax,DWORD PTR [rsi+rax*4]
   1703d:	test   eax,eax
   1703f:	jns    16a7a <mibz2::bwt::room_segments_unrolled::<32, 1>+0xa7a>
   17045:	mov    r10d,0x1f
   1704b:	jmp    17194 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1194>
   17050:	xor    r10d,r10d
   17053:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   17058:	nop    DWORD PTR [rax+rax*1+0x0]
   17060:	mov    r10d,0x1
   17066:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   1706b:	mov    r10d,0x2
   17071:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   17076:	mov    r10d,0x3
   1707c:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   17081:	mov    r10d,0x4
   17087:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   1708c:	mov    r10d,0x5
   17092:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   17097:	mov    r10d,0x6
   1709d:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170a2:	mov    r10d,0x7
   170a8:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170ad:	mov    r10d,0x8
   170b3:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170b8:	mov    r10d,0x9
   170be:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170c3:	mov    r10d,0xa
   170c9:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170ce:	mov    r10d,0xb
   170d4:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170d9:	mov    r10d,0xc
   170df:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
   170e4:	mov    r10d,0xd
   170ea:	jmp    171b0 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x11b0>
