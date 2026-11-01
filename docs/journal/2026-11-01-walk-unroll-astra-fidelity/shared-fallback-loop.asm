   171bf:	mov    rax,QWORD PTR [rbp+0x50]
   171c3:	mov    QWORD PTR [rsp+0x48],rax
   171c8:	mov    rax,QWORD PTR [rbp+0x58]
   171cc:	mov    QWORD PTR [rsp+0x18],rax
   171d1:	mov    r15,QWORD PTR [rbp+0x8]
   171d5:	mov    rax,QWORD PTR [rbp+0x10]
   171d9:	mov    QWORD PTR [rsp+0x30],rax
   171de:	mov    r13,r10
   171e1:	shl    r13,0x5
   171e5:	add    r13,r12
   171e8:	mov    r9,r10
   171eb:	nop    DWORD PTR [rax+rax*1+0x0]
   171f0:	lea    r10,[r9+0x1]
   171f4:	mov    r8d,DWORD PTR [r13+0x0]
   171f8:	mov    ecx,DWORD PTR [rbx+r8*4]
   171fc:	test   ecx,ecx
   171fe:	js     17230 <mibz2::bwt::room_segments_unrolled::<32, 1>+0x1230>
   17200:	mov    edi,ecx
   17202:	shr    edi,0x8
   17205:	mov    DWORD PTR [r13+0x0],edi
   17209:	prefetcht0 BYTE PTR [rbx+rdi*4]
   1720d:	mov    rdi,QWORD PTR [rsp+r9*8+0x50]
   17212:	mov    BYTE PTR [rdi+rsi*1],cl
   17215:	lea    rcx,[r10-0x1]
   17219:	add    r13,0x20
   1721d:	mov    r9,r10
