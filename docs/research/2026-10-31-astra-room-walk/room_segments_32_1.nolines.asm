
<target>/release/examples/phases:     file format elf64-x86-64


Disassembly of section .text:

0000000000029a50 <mibz2::bwt::room_segments::<32, 1>>:
   29a50:	push   rbp
   29a51:	push   r15
   29a53:	push   r14
   29a55:	push   r13
   29a57:	push   r12
   29a59:	push   rbx
   29a5a:	sub    rsp,0x648
   29a61:	mov    r15,rdi
   29a64:	mov    edi,edx
   29a66:	cmp    rsi,rdi
   29a69:	jbe    2b84e <mibz2::bwt::room_segments::<32, 1>+0x1dfe>
   29a6f:	mov    edi,DWORD PTR [r15+rdi*4]
   29a73:	mov    QWORD PTR [rcx+0x10],0x0
   29a7b:	movabs r12,0x2545f4914f6cdd1d
   29a85:	cmp    QWORD PTR [rcx],0x1fe
   29a8c:	jbe    2b5c6 <mibz2::bwt::room_segments::<32, 1>+0x1b76>
   29a92:	xor    r13d,r13d
   29a95:	shr    edi,0x8
   29a98:	xor    r12,rsi
   29a9b:	mov    QWORD PTR [rsp+0x30],rcx
   29aa0:	mov    r14,QWORD PTR [rcx+0x8]
   29aa4:	mov    ecx,0x1ff
   29aa9:	mov    rbx,r13
   29aac:	nop    DWORD PTR [rax+0x0]
   29ab0:	mov    rbp,rbx
   29ab3:	mov    rax,r12
   29ab6:	shl    rax,0xd
   29aba:	xor    rax,r12
   29abd:	mov    rdx,rax
   29ac0:	shr    rdx,0x7
   29ac4:	xor    rdx,rax
   29ac7:	mov    r12,rdx
   29aca:	shl    r12,0x11
   29ace:	xor    r12,rdx
   29ad1:	mov    rax,r12
   29ad4:	xor    edx,edx
   29ad6:	div    rsi
   29ad9:	mov    DWORD PTR [r14+rbx*4],edx
   29add:	inc    rbx
   29ae0:	dec    rcx
   29ae3:	jne    29ab0 <mibz2::bwt::room_segments::<32, 1>+0x60>
   29ae5:	mov    QWORD PTR [rsp+0x170],rsi
   29aed:	mov    r12,QWORD PTR [rsp+0x30]
   29af2:	mov    QWORD PTR [r12+0x10],rbx
   29af7:	cmp    rbx,QWORD PTR [r12]
   29afb:	mov    DWORD PTR [rsp+0x4c],edi
   29aff:	je     2b272 <mibz2::bwt::room_segments::<32, 1>+0x1822>
   29b05:	mov    DWORD PTR [r14+rbx*4],edi
   29b09:	lea    r9,[rbp+0x2]
   29b0d:	mov    QWORD PTR [r12+0x10],r9
   29b12:	cmp    rbp,0xfffffffffffffffd
   29b16:	jbe    2b5f9 <mibz2::bwt::room_segments::<32, 1>+0x1ba9>
   29b1c:	mov    r12,QWORD PTR [rsp+0x170]
   29b24:	mov    rbp,QWORD PTR [rsp+0x30]
   29b29:	test   r9,r9
   29b2c:	je     2b85a <mibz2::bwt::room_segments::<32, 1>+0x1e0a>
   29b32:	lea    rax,[r9*4+0x0]
   29b3a:	xor    ecx,ecx
   29b3c:	nop    DWORD PTR [rax+0x0]
   29b40:	mov    edi,DWORD PTR [r14+rcx*1]
   29b44:	cmp    r12,rdi
   29b47:	jbe    2b765 <mibz2::bwt::room_segments::<32, 1>+0x1d15>
   29b4d:	or     BYTE PTR [r15+rdi*4+0x3],0x80
   29b53:	add    rcx,0x4
   29b57:	cmp    rax,rcx
   29b5a:	jne    29b40 <mibz2::bwt::room_segments::<32, 1>+0xf0>
   29b5c:	lea    rax,[r12+r12*1]
   29b60:	mov    rcx,rax
   29b63:	or     rcx,r9
   29b66:	shr    rcx,0x20
   29b6a:	je     29b73 <mibz2::bwt::room_segments::<32, 1>+0x123>
   29b6c:	xor    edx,edx
   29b6e:	div    r9
   29b71:	jmp    29b78 <mibz2::bwt::room_segments::<32, 1>+0x128>
   29b73:	xor    edx,edx
   29b75:	div    r9d
   29b78:	mov    rbx,QWORD PTR [rbp+0x40]
   29b7c:	cmp    rbx,r9
   29b7f:	mov    r14,r9
   29b82:	cmova  r14,rbx
   29b86:	mov    QWORD PTR [rsp+0x58],r9
   29b8b:	jae    2aa94 <mibz2::bwt::room_segments::<32, 1>+0x1044>
   29b91:	sub    r14,rbx
   29b94:	mov    rcx,QWORD PTR [rbp+0x30]
   29b98:	sub    rcx,rbx
   29b9b:	mov    rdx,rbx
   29b9e:	cmp    r14,rcx
   29ba1:	ja     2b69f <mibz2::bwt::room_segments::<32, 1>+0x1c4f>
   29ba7:	mov    rsi,QWORD PTR [rbp+0x38]
   29bab:	inc    rbx
   29bae:	test   r14b,0x1
   29bb2:	jne    2b6d9 <mibz2::bwt::room_segments::<32, 1>+0x1c89>
   29bb8:	cmp    r9,rbx
   29bbb:	jne    2b706 <mibz2::bwt::room_segments::<32, 1>+0x1cb6>
   29bc1:	mov    QWORD PTR [rbp+0x40],rdx
   29bc5:	cmp    r9,rdx
   29bc8:	ja     2aaa4 <mibz2::bwt::room_segments::<32, 1>+0x1054>
   29bce:	lea    rbx,[rax+0x40]
   29bd2:	mov    r14,QWORD PTR [rbp+0x38]
   29bd6:	lea    rcx,[r9*8+0x0]
   29bde:	lea    r12,[rcx+rcx*2]
   29be2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   29bf0:	mov    QWORD PTR [r14+0x10],0x0
   29bf8:	cmp    rbx,QWORD PTR [r14]
   29bfb:	ja     29c09 <mibz2::bwt::room_segments::<32, 1>+0x1b9>
   29bfd:	add    r14,0x18
   29c01:	add    r12,0xffffffffffffffe8
   29c05:	jne    29bf0 <mibz2::bwt::room_segments::<32, 1>+0x1a0>
   29c07:	jmp    29c36 <mibz2::bwt::room_segments::<32, 1>+0x1e6>
   29c09:	mov    ecx,0x1
   29c0e:	mov    r8d,0x1
   29c14:	mov    rdi,r14
   29c17:	xor    esi,esi
   29c19:	mov    rdx,rbx
   29c1c:	mov    r13,rax
   29c1f:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   29c24:	mov    rax,r13
   29c27:	mov    r9,QWORD PTR [rsp+0x58]
   29c2c:	add    r14,0x18
   29c30:	add    r12,0xffffffffffffffe8
   29c34:	jne    29bf0 <mibz2::bwt::room_segments::<32, 1>+0x1a0>
   29c36:	mov    QWORD PTR [rbp+0x58],0x0
   29c3e:	cmp    r9,QWORD PTR [rbp+0x48]
   29c42:	ja     2b660 <mibz2::bwt::room_segments::<32, 1>+0x1c10>
   29c48:	xor    ebx,ebx
   29c4a:	mov    r14,QWORD PTR [rbp+0x50]
   29c4e:	lea    rdi,[r14+rbx*8]
   29c52:	cmp    r9,0x1
   29c56:	je     29c88 <mibz2::bwt::room_segments::<32, 1>+0x238>
   29c58:	lea    rdx,[r9*8-0x8]
   29c60:	mov    esi,0xff
   29c65:	mov    r12,rax
   29c68:	call   QWORD PTR [rip+0x427da]        # 6c448 <memset@GLIBC_2.2.5>
   29c6e:	mov    rax,r12
   29c71:	mov    r9,QWORD PTR [rsp+0x58]
   29c76:	lea    rcx,[rbx+r9*1]
   29c7a:	lea    rdi,[r14+rcx*8]
   29c7e:	add    rdi,0xfffffffffffffff8
   29c82:	add    rbx,r9
   29c85:	dec    rbx
   29c88:	mov    QWORD PTR [rdi],0xffffffffffffffff
   29c8f:	inc    rbx
   29c92:	mov    QWORD PTR [rbp+0x58],rbx
   29c96:	xorps  xmm0,xmm0
   29c99:	movups XMMWORD PTR [rsp+0x238],xmm0
   29ca1:	mov    DWORD PTR [rsp+0x248],0x0
   29cac:	movups XMMWORD PTR [rsp+0x258],xmm0
   29cb4:	mov    DWORD PTR [rsp+0x268],0x0
   29cbf:	movups XMMWORD PTR [rsp+0x278],xmm0
   29cc7:	mov    DWORD PTR [rsp+0x288],0x0
   29cd2:	movups XMMWORD PTR [rsp+0x298],xmm0
   29cda:	mov    DWORD PTR [rsp+0x2a8],0x0
   29ce5:	movups XMMWORD PTR [rsp+0x2b8],xmm0
   29ced:	mov    DWORD PTR [rsp+0x2c8],0x0
   29cf8:	movups XMMWORD PTR [rsp+0x2d8],xmm0
   29d00:	mov    DWORD PTR [rsp+0x2e8],0x0
   29d0b:	movups XMMWORD PTR [rsp+0x2f8],xmm0
   29d13:	mov    DWORD PTR [rsp+0x308],0x0
   29d1e:	movups XMMWORD PTR [rsp+0x318],xmm0
   29d26:	mov    DWORD PTR [rsp+0x328],0x0
   29d31:	movups XMMWORD PTR [rsp+0x338],xmm0
   29d39:	mov    DWORD PTR [rsp+0x348],0x0
   29d44:	movups XMMWORD PTR [rsp+0x358],xmm0
   29d4c:	mov    DWORD PTR [rsp+0x368],0x0
   29d57:	movups XMMWORD PTR [rsp+0x378],xmm0
   29d5f:	mov    DWORD PTR [rsp+0x388],0x0
   29d6a:	movups XMMWORD PTR [rsp+0x398],xmm0
   29d72:	mov    DWORD PTR [rsp+0x3a8],0x0
   29d7d:	movups XMMWORD PTR [rsp+0x3b8],xmm0
   29d85:	mov    DWORD PTR [rsp+0x3c8],0x0
   29d90:	movups XMMWORD PTR [rsp+0x3d8],xmm0
   29d98:	mov    DWORD PTR [rsp+0x3e8],0x0
   29da3:	movups XMMWORD PTR [rsp+0x3f8],xmm0
   29dab:	mov    DWORD PTR [rsp+0x408],0x0
   29db6:	movups XMMWORD PTR [rsp+0x418],xmm0
   29dbe:	mov    DWORD PTR [rsp+0x428],0x0
   29dc9:	movups XMMWORD PTR [rsp+0x438],xmm0
   29dd1:	mov    DWORD PTR [rsp+0x448],0x0
   29ddc:	movups XMMWORD PTR [rsp+0x458],xmm0
   29de4:	mov    DWORD PTR [rsp+0x468],0x0
   29def:	movups XMMWORD PTR [rsp+0x478],xmm0
   29df7:	mov    DWORD PTR [rsp+0x488],0x0
   29e02:	movups XMMWORD PTR [rsp+0x498],xmm0
   29e0a:	mov    DWORD PTR [rsp+0x4a8],0x0
   29e15:	movups XMMWORD PTR [rsp+0x4b8],xmm0
   29e1d:	mov    DWORD PTR [rsp+0x4c8],0x0
   29e28:	movups XMMWORD PTR [rsp+0x4d8],xmm0
   29e30:	mov    DWORD PTR [rsp+0x4e8],0x0
   29e3b:	movups XMMWORD PTR [rsp+0x4f8],xmm0
   29e43:	mov    DWORD PTR [rsp+0x508],0x0
   29e4e:	movups XMMWORD PTR [rsp+0x518],xmm0
   29e56:	mov    DWORD PTR [rsp+0x528],0x0
   29e61:	movups XMMWORD PTR [rsp+0x538],xmm0
   29e69:	mov    DWORD PTR [rsp+0x548],0x0
   29e74:	movups XMMWORD PTR [rsp+0x558],xmm0
   29e7c:	mov    DWORD PTR [rsp+0x568],0x0
   29e87:	movups XMMWORD PTR [rsp+0x578],xmm0
   29e8f:	mov    DWORD PTR [rsp+0x588],0x0
   29e9a:	movups XMMWORD PTR [rsp+0x598],xmm0
   29ea2:	mov    DWORD PTR [rsp+0x5a8],0x0
   29ead:	movups XMMWORD PTR [rsp+0x5b8],xmm0
   29eb5:	mov    DWORD PTR [rsp+0x5c8],0x0
   29ec0:	movups XMMWORD PTR [rsp+0x5d8],xmm0
   29ec8:	mov    DWORD PTR [rsp+0x5e8],0x0
   29ed3:	movups XMMWORD PTR [rsp+0x5f8],xmm0
   29edb:	mov    DWORD PTR [rsp+0x608],0x0
   29ee6:	movups XMMWORD PTR [rsp+0x618],xmm0
   29eee:	mov    DWORD PTR [rsp+0x628],0x0
   29ef9:	mov    QWORD PTR [rsp+0x230],0xffffffffffffffff
   29f05:	mov    QWORD PTR [rsp+0x250],0xffffffffffffffff
   29f11:	mov    QWORD PTR [rsp+0x270],0xffffffffffffffff
   29f1d:	mov    QWORD PTR [rsp+0x290],0xffffffffffffffff
   29f29:	mov    QWORD PTR [rsp+0x2b0],0xffffffffffffffff
   29f35:	mov    QWORD PTR [rsp+0x2d0],0xffffffffffffffff
   29f41:	mov    QWORD PTR [rsp+0x2f0],0xffffffffffffffff
   29f4d:	mov    QWORD PTR [rsp+0x310],0xffffffffffffffff
   29f59:	mov    QWORD PTR [rsp+0x330],0xffffffffffffffff
   29f65:	mov    QWORD PTR [rsp+0x350],0xffffffffffffffff
   29f71:	mov    QWORD PTR [rsp+0x370],0xffffffffffffffff
   29f7d:	mov    QWORD PTR [rsp+0x390],0xffffffffffffffff
   29f89:	mov    QWORD PTR [rsp+0x3b0],0xffffffffffffffff
   29f95:	mov    QWORD PTR [rsp+0x3d0],0xffffffffffffffff
   29fa1:	mov    QWORD PTR [rsp+0x3f0],0xffffffffffffffff
   29fad:	mov    QWORD PTR [rsp+0x410],0xffffffffffffffff
   29fb9:	mov    QWORD PTR [rsp+0x430],0xffffffffffffffff
   29fc5:	mov    QWORD PTR [rsp+0x450],0xffffffffffffffff
   29fd1:	mov    QWORD PTR [rsp+0x470],0xffffffffffffffff
   29fdd:	mov    QWORD PTR [rsp+0x490],0xffffffffffffffff
   29fe9:	mov    QWORD PTR [rsp+0x4b0],0xffffffffffffffff
   29ff5:	mov    QWORD PTR [rsp+0x4d0],0xffffffffffffffff
   2a001:	mov    QWORD PTR [rsp+0x4f0],0xffffffffffffffff
   2a00d:	mov    QWORD PTR [rsp+0x510],0xffffffffffffffff
   2a019:	mov    QWORD PTR [rsp+0x530],0xffffffffffffffff
   2a025:	mov    QWORD PTR [rsp+0x550],0xffffffffffffffff
   2a031:	mov    QWORD PTR [rsp+0x570],0xffffffffffffffff
   2a03d:	mov    QWORD PTR [rsp+0x590],0xffffffffffffffff
   2a049:	mov    QWORD PTR [rsp+0x5b0],0xffffffffffffffff
   2a055:	mov    QWORD PTR [rsp+0x5d0],0xffffffffffffffff
   2a061:	mov    QWORD PTR [rsp+0x5f0],0xffffffffffffffff
   2a06d:	mov    QWORD PTR [rsp+0x610],0xffffffffffffffff
   2a079:	mov    rcx,QWORD PTR [rbp+0x38]
   2a07d:	mov    rsi,QWORD PTR [rbp+0x40]
   2a081:	mov    r8,QWORD PTR [rbp+0x8]
   2a085:	mov    r14,QWORD PTR [rbp+0x10]
   2a089:	mov    QWORD PTR [rsp+0x28],rcx
   2a08e:	lea    r10,[rcx+0x8]
   2a092:	xor    edi,edi
   2a094:	xor    r13d,r13d
   2a097:	nop    WORD PTR [rax+rax*1+0x0]
   2a0a0:	cmp    rsi,rdi
   2a0a3:	je     2b774 <mibz2::bwt::room_segments::<32, 1>+0x1d24>
   2a0a9:	cmp    r14,rdi
   2a0ac:	je     2b780 <mibz2::bwt::room_segments::<32, 1>+0x1d30>
   2a0b2:	mov    rcx,QWORD PTR [r10]
   2a0b5:	mov    edx,DWORD PTR [r8+rdi*4]
   2a0b9:	mov    edx,DWORD PTR [r15+rdx*4]
   2a0bd:	mov    BYTE PTR [rcx],dl
   2a0bf:	shr    edx,0x8
   2a0c2:	and    edx,0x7fffff
   2a0c8:	lea    r11,[rcx+0x1]
   2a0cc:	add    rcx,QWORD PTR [r10-0x8]
   2a0d0:	mov    QWORD PTR [rsp+r13*1+0x230],rdi
   2a0d8:	mov    QWORD PTR [rsp+r13*1+0x238],r11
   2a0e0:	mov    QWORD PTR [rsp+r13*1+0x240],rcx
   2a0e8:	mov    DWORD PTR [rsp+r13*1+0x248],edx
   2a0f0:	prefetcht0 BYTE PTR [r15+rdx*4]
   2a0f5:	lea    rdx,[rdi+0x1]
   2a0f9:	add    r13,0x20
   2a0fd:	cmp    rdi,0x1e
   2a101:	ja     2a10f <mibz2::bwt::room_segments::<32, 1>+0x6bf>
   2a103:	add    r10,0x18
   2a107:	mov    rdi,rdx
   2a10a:	cmp    rdx,r9
   2a10d:	jb     2a0a0 <mibz2::bwt::room_segments::<32, 1>+0x650>
   2a10f:	cmp    rdx,0x20
   2a113:	mov    QWORD PTR [rsp+0x20],r8
   2a118:	mov    QWORD PTR [rsp+0x68],r14
   2a11d:	jne    2aab5 <mibz2::bwt::room_segments::<32, 1>+0x1065>
   2a123:	mov    QWORD PTR [rsp+0x220],rdx
   2a12b:	add    rax,0x3f
   2a12f:	mov    ecx,0x20
   2a134:	mov    QWORD PTR [rsp+0x60],rcx
   2a139:	mov    QWORD PTR [rsp+0x8],rsi
   2a13e:	mov    QWORD PTR [rsp+0x228],rax
   2a146:	xor    r12d,r12d
   2a149:	mov    rbp,rax
   2a14c:	jmp    2a181 <mibz2::bwt::room_segments::<32, 1>+0x731>
   2a14e:	xchg   ax,ax
   2a150:	add    rbx,rdi
   2a153:	mov    QWORD PTR [rsp+r12*1+0x238],rbx
   2a15b:	add    rdx,rdi
   2a15e:	mov    QWORD PTR [rsp+r12*1+0x240],rdx
   2a166:	sub    rdx,rbx
   2a169:	add    r12,0x20
   2a16d:	cmp    rdx,rbp
   2a170:	cmovb  rbp,rdx
   2a174:	cmp    r12,0x400
   2a17b:	je     2a210 <mibz2::bwt::room_segments::<32, 1>+0x7c0>
   2a181:	mov    rbx,QWORD PTR [rsp+r12*1+0x238]
   2a189:	mov    rdx,QWORD PTR [rsp+r12*1+0x240]
   2a191:	cmp    rbx,rdx
   2a194:	jne    2a166 <mibz2::bwt::room_segments::<32, 1>+0x716>
   2a196:	mov    rdi,QWORD PTR [rsp+r12*1+0x230]
   2a19e:	cmp    rdi,rsi
   2a1a1:	jae    2b7d5 <mibz2::bwt::room_segments::<32, 1>+0x1d85>
   2a1a7:	lea    rcx,[rdi+rdi*2]
   2a1ab:	mov    r8,QWORD PTR [rsp+0x28]
   2a1b0:	mov    rdx,QWORD PTR [r8+rcx*8]
   2a1b4:	mov    rdi,QWORD PTR [r8+rcx*8+0x8]
   2a1b9:	sub    rbx,rdi
   2a1bc:	mov    QWORD PTR [r8+rcx*8+0x10],rbx
   2a1c1:	mov    r8,rdx
   2a1c4:	sub    r8,rbx
   2a1c7:	cmp    rdx,r8
   2a1ca:	jbe    2a150 <mibz2::bwt::room_segments::<32, 1>+0x700>
   2a1cc:	mov    rsi,QWORD PTR [rsp+0x28]
   2a1d1:	lea    rdi,[rsi+rcx*8]
   2a1d5:	mov    QWORD PTR [rsp+0x40],rdi
   2a1da:	mov    ecx,0x1
   2a1df:	mov    r8d,0x1
   2a1e5:	mov    rsi,rbx
   2a1e8:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2a1ed:	mov    rsi,QWORD PTR [rsp+0x8]
   2a1f2:	mov    rcx,QWORD PTR [rsp+0x40]
   2a1f7:	mov    rdx,QWORD PTR [rcx]
   2a1fa:	mov    rdi,QWORD PTR [rcx+0x8]
   2a1fe:	jmp    2a150 <mibz2::bwt::room_segments::<32, 1>+0x700>
   2a203:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a210:	mov    rax,QWORD PTR [rsp+0x238]
   2a218:	mov    rcx,QWORD PTR [rsp+0x258]
   2a220:	mov    rdx,QWORD PTR [rsp+0x278]
   2a228:	mov    QWORD PTR [rsp+0x50],rdx
   2a22d:	mov    rdx,QWORD PTR [rsp+0x298]
   2a235:	mov    QWORD PTR [rsp+0x10],rdx
   2a23a:	mov    r12,QWORD PTR [rsp+0x2b8]
   2a242:	mov    rbx,QWORD PTR [rsp+0x2d8]
   2a24a:	mov    r11,QWORD PTR [rsp+0x2f8]
   2a252:	mov    r10,QWORD PTR [rsp+0x318]
   2a25a:	mov    r9,QWORD PTR [rsp+0x338]
   2a262:	mov    r8,QWORD PTR [rsp+0x358]
   2a26a:	mov    rdi,QWORD PTR [rsp+0x378]
   2a272:	mov    rsi,QWORD PTR [rsp+0x398]
   2a27a:	mov    rdx,QWORD PTR [rsp+0x3b8]
   2a282:	mov    QWORD PTR [rsp+0x38],rdx
   2a287:	mov    rdx,QWORD PTR [rsp+0x3d8]
   2a28f:	mov    QWORD PTR [rsp+0x18],rdx
   2a294:	mov    rdx,QWORD PTR [rsp+0x3f8]
   2a29c:	mov    QWORD PTR [rsp+0x40],rdx
   2a2a1:	mov    QWORD PTR [rsp+0x1d0],rax
   2a2a9:	mov    QWORD PTR [rsp+0x70],rax
   2a2ae:	mov    QWORD PTR [rsp+0x1d8],rcx
   2a2b6:	mov    QWORD PTR [rsp+0x78],rcx
   2a2bb:	mov    rax,QWORD PTR [rsp+0x50]
   2a2c0:	mov    QWORD PTR [rsp+0x80],rax
   2a2c8:	mov    rax,QWORD PTR [rsp+0x10]
   2a2cd:	mov    QWORD PTR [rsp+0x88],rax
   2a2d5:	mov    QWORD PTR [rsp+0x1e0],r12
   2a2dd:	mov    QWORD PTR [rsp+0x90],r12
   2a2e5:	mov    QWORD PTR [rsp+0x1e8],rbx
   2a2ed:	mov    QWORD PTR [rsp+0x98],rbx
   2a2f5:	mov    QWORD PTR [rsp+0x1f0],r11
   2a2fd:	mov    QWORD PTR [rsp+0xa0],r11
   2a305:	mov    QWORD PTR [rsp+0x1f8],r10
   2a30d:	mov    QWORD PTR [rsp+0xa8],r10
   2a315:	mov    QWORD PTR [rsp+0x200],r9
   2a31d:	mov    QWORD PTR [rsp+0xb0],r9
   2a325:	mov    QWORD PTR [rsp+0x208],r8
   2a32d:	mov    QWORD PTR [rsp+0xb8],r8
   2a335:	mov    QWORD PTR [rsp+0x210],rdi
   2a33d:	mov    QWORD PTR [rsp+0xc0],rdi
   2a345:	mov    QWORD PTR [rsp+0x218],rsi
   2a34d:	mov    QWORD PTR [rsp+0xc8],rsi
   2a355:	mov    rax,QWORD PTR [rsp+0x38]
   2a35a:	mov    QWORD PTR [rsp+0xd0],rax
   2a362:	mov    rax,QWORD PTR [rsp+0x18]
   2a367:	mov    QWORD PTR [rsp+0xd8],rax
   2a36f:	mov    rax,QWORD PTR [rsp+0x40]
   2a374:	mov    QWORD PTR [rsp+0xe0],rax
   2a37c:	mov    rax,QWORD PTR [rsp+0x418]
   2a384:	mov    QWORD PTR [rsp+0x1a0],rax
   2a38c:	mov    QWORD PTR [rsp+0xe8],rax
   2a394:	mov    rax,QWORD PTR [rsp+0x438]
   2a39c:	mov    QWORD PTR [rsp+0x1a8],rax
   2a3a4:	mov    QWORD PTR [rsp+0xf0],rax
   2a3ac:	mov    rax,QWORD PTR [rsp+0x458]
   2a3b4:	mov    QWORD PTR [rsp+0x1b0],rax
   2a3bc:	mov    QWORD PTR [rsp+0xf8],rax
   2a3c4:	mov    rax,QWORD PTR [rsp+0x478]
   2a3cc:	mov    QWORD PTR [rsp+0x1b8],rax
   2a3d4:	mov    QWORD PTR [rsp+0x100],rax
   2a3dc:	mov    rax,QWORD PTR [rsp+0x498]
   2a3e4:	mov    QWORD PTR [rsp+0x1c0],rax
   2a3ec:	mov    QWORD PTR [rsp+0x108],rax
   2a3f4:	mov    rax,QWORD PTR [rsp+0x4b8]
   2a3fc:	mov    QWORD PTR [rsp+0x1c8],rax
   2a404:	mov    QWORD PTR [rsp+0x110],rax
   2a40c:	mov    r12,QWORD PTR [rsp+0x4d8]
   2a414:	mov    QWORD PTR [rsp+0x118],r12
   2a41c:	mov    rdi,QWORD PTR [rsp+0x4f8]
   2a424:	mov    QWORD PTR [rsp+0x120],rdi
   2a42c:	mov    rsi,QWORD PTR [rsp+0x518]
   2a434:	mov    QWORD PTR [rsp+0x128],rsi
   2a43c:	mov    rdx,QWORD PTR [rsp+0x538]
   2a444:	mov    QWORD PTR [rsp+0x130],rdx
   2a44c:	mov    rcx,QWORD PTR [rsp+0x558]
   2a454:	mov    QWORD PTR [rsp+0x138],rcx
   2a45c:	mov    rax,QWORD PTR [rsp+0x578]
   2a464:	mov    QWORD PTR [rsp+0x140],rax
   2a46c:	mov    rbx,QWORD PTR [rsp+0x598]
   2a474:	mov    QWORD PTR [rsp+0x148],rbx
   2a47c:	mov    r11,QWORD PTR [rsp+0x5b8]
   2a484:	mov    QWORD PTR [rsp+0x150],r11
   2a48c:	mov    r10,QWORD PTR [rsp+0x5d8]
   2a494:	mov    QWORD PTR [rsp+0x158],r10
   2a49c:	mov    r9,QWORD PTR [rsp+0x5f8]
   2a4a4:	mov    QWORD PTR [rsp+0x160],r9
   2a4ac:	mov    r8,QWORD PTR [rsp+0x618]
   2a4b4:	mov    QWORD PTR [rsp+0x168],r8
   2a4bc:	test   rbp,rbp
   2a4bf:	je     2a6c0 <mibz2::bwt::room_segments::<32, 1>+0xc70>
   2a4c5:	mov    rax,QWORD PTR [rsp+0x30]
   2a4ca:	mov    rcx,QWORD PTR [rax+0x50]
   2a4ce:	mov    QWORD PTR [rsp+0x50],rcx
   2a4d3:	mov    rax,QWORD PTR [rax+0x58]
   2a4d7:	mov    QWORD PTR [rsp+0x10],rax
   2a4dc:	xor    r9d,r9d
   2a4df:	jmp    2a4f9 <mibz2::bwt::room_segments::<32, 1>+0xaa9>
   2a4e1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a4f0:	cmp    r9,rbp
   2a4f3:	je     2a6e0 <mibz2::bwt::room_segments::<32, 1>+0xc90>
   2a4f9:	mov    rcx,r9
   2a4fc:	inc    r9
   2a4ff:	mov    rax,rcx
   2a502:	not    rax
   2a505:	mov    QWORD PTR [rsp+0x40],rax
   2a50a:	xor    r10d,r10d
   2a50d:	mov    r11,r10
   2a510:	shl    r11,0x5
   2a514:	lea    rax,[rsp+0x248]
   2a51c:	add    r11,rax
   2a51f:	mov    r8,r10
   2a522:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2a530:	lea    r10,[r8+0x1]
   2a534:	mov    ebx,DWORD PTR [r11]
   2a537:	mov    eax,DWORD PTR [r15+rbx*4]
   2a53b:	test   eax,eax
   2a53d:	js     2a570 <mibz2::bwt::room_segments::<32, 1>+0xb20>
   2a53f:	mov    edi,eax
   2a541:	shr    edi,0x8
   2a544:	mov    DWORD PTR [r11],edi
   2a547:	prefetcht0 BYTE PTR [r15+rdi*4]
   2a54c:	mov    rdi,QWORD PTR [rsp+r8*8+0x70]
   2a551:	mov    BYTE PTR [rdi+rcx*1],al
   2a554:	lea    rax,[r10-0x1]
   2a558:	add    r11,0x20
   2a55c:	mov    r8,r10
   2a55f:	cmp    rax,0x1f
   2a563:	jb     2a530 <mibz2::bwt::room_segments::<32, 1>+0xae0>
   2a565:	jmp    2a4f0 <mibz2::bwt::room_segments::<32, 1>+0xaa0>
   2a567:	nop    WORD PTR [rax+rax*1+0x0]
   2a570:	mov    rax,QWORD PTR [rsp+r8*8+0x70]
   2a575:	add    rax,rcx
   2a578:	mov    QWORD PTR [r11-0x10],rax
   2a57c:	mov    rdi,QWORD PTR [r11-0x18]
   2a580:	cmp    rdi,QWORD PTR [rsp+0x8]
   2a585:	jae    2b7e1 <mibz2::bwt::room_segments::<32, 1>+0x1d91>
   2a58b:	lea    r12,[rdi+rdi*2]
   2a58f:	mov    rdx,QWORD PTR [rsp+0x28]
   2a594:	sub    rax,QWORD PTR [rdx+r12*8+0x8]
   2a599:	mov    QWORD PTR [rdx+r12*8+0x10],rax
   2a59e:	test   r14,r14
   2a5a1:	je     2a5b3 <mibz2::bwt::room_segments::<32, 1>+0xb63>
   2a5a3:	cmp    r14,0x1
   2a5a7:	jne    2a5bc <mibz2::bwt::room_segments::<32, 1>+0xb6c>
   2a5a9:	xor    r12d,r12d
   2a5ac:	mov    r14,QWORD PTR [rsp+0x20]
   2a5b1:	jmp    2a5ed <mibz2::bwt::room_segments::<32, 1>+0xb9d>
   2a5b3:	mov    r12,0xffffffffffffffff
   2a5ba:	jmp    2a5fc <mibz2::bwt::room_segments::<32, 1>+0xbac>
   2a5bc:	mov    rax,r14
   2a5bf:	xor    r12d,r12d
   2a5c2:	mov    r14,QWORD PTR [rsp+0x20]
   2a5c7:	nop    WORD PTR [rax+rax*1+0x0]
   2a5d0:	mov    rdx,r12
   2a5d3:	mov    rsi,rax
   2a5d6:	shr    rsi,1
   2a5d9:	add    r12,rsi
   2a5dc:	cmp    DWORD PTR [r14+r12*4],ebx
   2a5e0:	cmova  r12,rdx
   2a5e4:	sub    rax,rsi
   2a5e7:	cmp    rax,0x1
   2a5eb:	ja     2a5d0 <mibz2::bwt::room_segments::<32, 1>+0xb80>
   2a5ed:	cmp    DWORD PTR [r14+r12*4],ebx
   2a5f1:	mov    rax,0xffffffffffffffff
   2a5f8:	cmovne r12,rax
   2a5fc:	cmp    rdi,QWORD PTR [rsp+0x10]
   2a601:	mov    rbx,QWORD PTR [rsp+0x60]
   2a606:	jae    2b7f2 <mibz2::bwt::room_segments::<32, 1>+0x1da2>
   2a60c:	mov    rax,QWORD PTR [rsp+0x50]
   2a611:	mov    QWORD PTR [rax+rdi*8],r12
   2a615:	cmp    rbx,QWORD PTR [rsp+0x58]
   2a61a:	mov    rax,QWORD PTR [rsp+0x68]
   2a61f:	je     2aacc <mibz2::bwt::room_segments::<32, 1>+0x107c>
   2a625:	cmp    rbx,QWORD PTR [rsp+0x8]
   2a62a:	jae    2b815 <mibz2::bwt::room_segments::<32, 1>+0x1dc5>
   2a630:	mov    r14,rax
   2a633:	cmp    rbx,rax
   2a636:	jae    2b803 <mibz2::bwt::room_segments::<32, 1>+0x1db3>
   2a63c:	lea    rax,[rbx+rbx*2]
   2a640:	mov    QWORD PTR [rsp+0x18],rax
   2a645:	mov    r12,QWORD PTR [rsp+0x28]
   2a64a:	mov    rdx,QWORD PTR [r12+rax*8+0x8]
   2a64f:	mov    rsi,QWORD PTR [rsp+0x20]
   2a654:	mov    esi,DWORD PTR [rsi+rbx*4]
   2a657:	mov    esi,DWORD PTR [r15+rsi*4]
   2a65b:	mov    BYTE PTR [rdx],sil
   2a65e:	shr    esi,0x8
   2a661:	and    esi,0x7fffff
   2a667:	lea    rdi,[rdx+0x1]
   2a66b:	mov    rax,QWORD PTR [rsp+0x40]
   2a670:	add    rax,rdx
   2a673:	inc    rax
   2a676:	mov    QWORD PTR [rsp+0x38],rax
   2a67b:	mov    rax,QWORD PTR [rsp+0x18]
   2a680:	add    rdx,QWORD PTR [r12+rax*8]
   2a684:	mov    QWORD PTR [r11-0x18],rbx
   2a688:	mov    QWORD PTR [r11-0x10],rdi
   2a68c:	mov    QWORD PTR [r11-0x8],rdx
   2a690:	mov    DWORD PTR [r11],esi
   2a693:	prefetcht0 BYTE PTR [r15+rsi*4]
   2a698:	inc    rbx
   2a69b:	mov    QWORD PTR [rsp+0x60],rbx
   2a6a0:	mov    rax,QWORD PTR [rsp+0x38]
   2a6a5:	mov    QWORD PTR [rsp+r8*8+0x70],rax
   2a6aa:	cmp    r8,0x1f
   2a6ae:	jb     2a50d <mibz2::bwt::room_segments::<32, 1>+0xabd>
   2a6b4:	jmp    2a4f0 <mibz2::bwt::room_segments::<32, 1>+0xaa0>
   2a6b9:	nop    DWORD PTR [rax+0x0]
   2a6c0:	mov    QWORD PTR [rsp+0x190],r12
   2a6c8:	mov    r12,QWORD PTR [rsp+0x50]
   2a6cd:	mov    QWORD PTR [rsp+0x198],rax
   2a6d5:	jmp    2a87e <mibz2::bwt::room_segments::<32, 1>+0xe2e>
   2a6da:	nop    WORD PTR [rax+rax*1+0x0]
   2a6e0:	mov    rax,QWORD PTR [rsp+0x70]
   2a6e5:	mov    QWORD PTR [rsp+0x1d0],rax
   2a6ed:	mov    rax,QWORD PTR [rsp+0x78]
   2a6f2:	mov    QWORD PTR [rsp+0x1d8],rax
   2a6fa:	mov    r12,QWORD PTR [rsp+0x80]
   2a702:	mov    rax,QWORD PTR [rsp+0x88]
   2a70a:	mov    QWORD PTR [rsp+0x10],rax
   2a70f:	mov    rax,QWORD PTR [rsp+0x90]
   2a717:	mov    QWORD PTR [rsp+0x1e0],rax
   2a71f:	mov    rax,QWORD PTR [rsp+0x98]
   2a727:	mov    QWORD PTR [rsp+0x1e8],rax
   2a72f:	mov    rax,QWORD PTR [rsp+0xa0]
   2a737:	mov    QWORD PTR [rsp+0x1f0],rax
   2a73f:	mov    rax,QWORD PTR [rsp+0xa8]
   2a747:	mov    QWORD PTR [rsp+0x1f8],rax
   2a74f:	mov    rax,QWORD PTR [rsp+0xb0]
   2a757:	mov    QWORD PTR [rsp+0x200],rax
   2a75f:	mov    rax,QWORD PTR [rsp+0xb8]
   2a767:	mov    QWORD PTR [rsp+0x208],rax
   2a76f:	mov    rax,QWORD PTR [rsp+0xc0]
   2a777:	mov    QWORD PTR [rsp+0x210],rax
   2a77f:	mov    rax,QWORD PTR [rsp+0xc8]
   2a787:	mov    QWORD PTR [rsp+0x218],rax
   2a78f:	mov    rax,QWORD PTR [rsp+0xd0]
   2a797:	mov    QWORD PTR [rsp+0x38],rax
   2a79c:	mov    rax,QWORD PTR [rsp+0xd8]
   2a7a4:	mov    QWORD PTR [rsp+0x18],rax
   2a7a9:	mov    rax,QWORD PTR [rsp+0xe0]
   2a7b1:	mov    QWORD PTR [rsp+0x40],rax
   2a7b6:	mov    rax,QWORD PTR [rsp+0xe8]
   2a7be:	mov    QWORD PTR [rsp+0x1a0],rax
   2a7c6:	mov    rax,QWORD PTR [rsp+0xf0]
   2a7ce:	mov    QWORD PTR [rsp+0x1a8],rax
   2a7d6:	mov    rax,QWORD PTR [rsp+0xf8]
   2a7de:	mov    QWORD PTR [rsp+0x1b0],rax
   2a7e6:	mov    rax,QWORD PTR [rsp+0x100]
   2a7ee:	mov    QWORD PTR [rsp+0x1b8],rax
   2a7f6:	mov    rax,QWORD PTR [rsp+0x108]
   2a7fe:	mov    QWORD PTR [rsp+0x1c0],rax
   2a806:	mov    rax,QWORD PTR [rsp+0x110]
   2a80e:	mov    QWORD PTR [rsp+0x1c8],rax
   2a816:	mov    rax,QWORD PTR [rsp+0x118]
   2a81e:	mov    QWORD PTR [rsp+0x190],rax
   2a826:	mov    rdi,QWORD PTR [rsp+0x120]
   2a82e:	mov    rsi,QWORD PTR [rsp+0x128]
   2a836:	mov    rdx,QWORD PTR [rsp+0x130]
   2a83e:	mov    rcx,QWORD PTR [rsp+0x138]
   2a846:	mov    rax,QWORD PTR [rsp+0x140]
   2a84e:	mov    QWORD PTR [rsp+0x198],rax
   2a856:	mov    rbx,QWORD PTR [rsp+0x148]
   2a85e:	mov    r11,QWORD PTR [rsp+0x150]
   2a866:	mov    r10,QWORD PTR [rsp+0x158]
   2a86e:	mov    r9,QWORD PTR [rsp+0x160]
   2a876:	mov    r8,QWORD PTR [rsp+0x168]
   2a87e:	mov    rax,QWORD PTR [rsp+0x1d0]
   2a886:	add    rax,rbp
   2a889:	mov    QWORD PTR [rsp+0x238],rax
   2a891:	mov    rax,QWORD PTR [rsp+0x1d8]
   2a899:	add    rax,rbp
   2a89c:	mov    QWORD PTR [rsp+0x258],rax
   2a8a4:	add    r12,rbp
   2a8a7:	mov    QWORD PTR [rsp+0x278],r12
   2a8af:	mov    r12,QWORD PTR [rsp+0x10]
   2a8b4:	add    r12,rbp
   2a8b7:	mov    QWORD PTR [rsp+0x298],r12
   2a8bf:	mov    r12,QWORD PTR [rsp+0x1e0]
   2a8c7:	add    r12,rbp
   2a8ca:	mov    QWORD PTR [rsp+0x2b8],r12
   2a8d2:	mov    r12,QWORD PTR [rsp+0x1e8]
   2a8da:	add    r12,rbp
   2a8dd:	mov    QWORD PTR [rsp+0x2d8],r12
   2a8e5:	mov    r12,QWORD PTR [rsp+0x1f0]
   2a8ed:	add    r12,rbp
   2a8f0:	mov    QWORD PTR [rsp+0x2f8],r12
   2a8f8:	mov    r12,QWORD PTR [rsp+0x1f8]
   2a900:	add    r12,rbp
   2a903:	mov    QWORD PTR [rsp+0x318],r12
   2a90b:	mov    r12,QWORD PTR [rsp+0x200]
   2a913:	add    r12,rbp
   2a916:	mov    QWORD PTR [rsp+0x338],r12
   2a91e:	mov    r12,QWORD PTR [rsp+0x208]
   2a926:	add    r12,rbp
   2a929:	mov    QWORD PTR [rsp+0x358],r12
   2a931:	mov    r12,QWORD PTR [rsp+0x210]
   2a939:	add    r12,rbp
   2a93c:	mov    QWORD PTR [rsp+0x378],r12
   2a944:	mov    r12,QWORD PTR [rsp+0x218]
   2a94c:	add    r12,rbp
   2a94f:	mov    QWORD PTR [rsp+0x398],r12
   2a957:	mov    r12,QWORD PTR [rsp+0x38]
   2a95c:	add    r12,rbp
   2a95f:	mov    QWORD PTR [rsp+0x3b8],r12
   2a967:	mov    r12,QWORD PTR [rsp+0x18]
   2a96c:	add    r12,rbp
   2a96f:	mov    QWORD PTR [rsp+0x3d8],r12
   2a977:	mov    r12,QWORD PTR [rsp+0x40]
   2a97c:	add    r12,rbp
   2a97f:	mov    QWORD PTR [rsp+0x3f8],r12
   2a987:	mov    rax,QWORD PTR [rsp+0x1a0]
   2a98f:	add    rax,rbp
   2a992:	mov    QWORD PTR [rsp+0x418],rax
   2a99a:	mov    rax,QWORD PTR [rsp+0x1a8]
   2a9a2:	add    rax,rbp
   2a9a5:	mov    QWORD PTR [rsp+0x438],rax
   2a9ad:	mov    rax,QWORD PTR [rsp+0x1b0]
   2a9b5:	add    rax,rbp
   2a9b8:	mov    QWORD PTR [rsp+0x458],rax
   2a9c0:	mov    rax,QWORD PTR [rsp+0x1b8]
   2a9c8:	add    rax,rbp
   2a9cb:	mov    QWORD PTR [rsp+0x478],rax
   2a9d3:	mov    rax,QWORD PTR [rsp+0x1c0]
   2a9db:	add    rax,rbp
   2a9de:	mov    QWORD PTR [rsp+0x498],rax
   2a9e6:	mov    rax,QWORD PTR [rsp+0x1c8]
   2a9ee:	add    rax,rbp
   2a9f1:	mov    QWORD PTR [rsp+0x4b8],rax
   2a9f9:	mov    rax,QWORD PTR [rsp+0x190]
   2aa01:	add    rax,rbp
   2aa04:	mov    QWORD PTR [rsp+0x4d8],rax
   2aa0c:	add    rdi,rbp
   2aa0f:	mov    QWORD PTR [rsp+0x4f8],rdi
   2aa17:	add    rsi,rbp
   2aa1a:	mov    QWORD PTR [rsp+0x518],rsi
   2aa22:	add    rdx,rbp
   2aa25:	mov    QWORD PTR [rsp+0x538],rdx
   2aa2d:	add    rcx,rbp
   2aa30:	mov    QWORD PTR [rsp+0x558],rcx
   2aa38:	mov    rax,QWORD PTR [rsp+0x198]
   2aa40:	add    rax,rbp
   2aa43:	mov    QWORD PTR [rsp+0x578],rax
   2aa4b:	add    rbx,rbp
   2aa4e:	mov    QWORD PTR [rsp+0x598],rbx
   2aa56:	add    r11,rbp
   2aa59:	mov    QWORD PTR [rsp+0x5b8],r11
   2aa61:	add    r10,rbp
   2aa64:	mov    QWORD PTR [rsp+0x5d8],r10
   2aa6c:	add    r9,rbp
   2aa6f:	mov    QWORD PTR [rsp+0x5f8],r9
   2aa77:	add    r8,rbp
   2aa7a:	mov    QWORD PTR [rsp+0x618],r8
   2aa82:	mov    rax,QWORD PTR [rsp+0x228]
   2aa8a:	mov    rsi,QWORD PTR [rsp+0x8]
   2aa8f:	jmp    2a146 <mibz2::bwt::room_segments::<32, 1>+0x6f6>
   2aa94:	mov    rdx,r14
   2aa97:	mov    QWORD PTR [rbp+0x40],rdx
   2aa9b:	cmp    r9,rdx
   2aa9e:	jbe    29bce <mibz2::bwt::room_segments::<32, 1>+0x17e>
   2aaa4:	lea    rcx,[rip+0x3fc6d]        # 6a718 <__frame_dummy_init_array_entry+0x7b8>
   2aaab:	xor    edi,edi
   2aaad:	mov    rsi,r9
   2aab0:	call   211a0 <core::slice::index::slice_index_fail>
   2aab5:	mov    rax,QWORD PTR [rbp+0x50]
   2aab9:	mov    QWORD PTR [rsp+0x50],rax
   2aabe:	mov    rax,QWORD PTR [rbp+0x58]
   2aac2:	mov    QWORD PTR [rsp+0x10],rax
   2aac7:	jmp    2b0ac <mibz2::bwt::room_segments::<32, 1>+0x165c>
   2aacc:	test   r8,r8
   2aacf:	mov    rdx,QWORD PTR [rsp+0x220]
   2aad7:	je     2ae65 <mibz2::bwt::room_segments::<32, 1>+0x1415>
   2aadd:	mov    rax,QWORD PTR [rsp+0x70]
   2aae2:	add    rax,r9
   2aae5:	mov    QWORD PTR [rsp+0x238],rax
   2aaed:	cmp    r8,0x2
   2aaf1:	jb     2ae75 <mibz2::bwt::room_segments::<32, 1>+0x1425>
   2aaf7:	mov    rax,QWORD PTR [rsp+0x78]
   2aafc:	add    rax,r9
   2aaff:	mov    QWORD PTR [rsp+0x258],rax
   2ab07:	cmp    r8,0x2
   2ab0b:	jbe    2ae88 <mibz2::bwt::room_segments::<32, 1>+0x1438>
   2ab11:	mov    rax,QWORD PTR [rsp+0x80]
   2ab19:	add    rax,r9
   2ab1c:	mov    QWORD PTR [rsp+0x278],rax
   2ab24:	cmp    r8,0x3
   2ab28:	jbe    2ae9b <mibz2::bwt::room_segments::<32, 1>+0x144b>
   2ab2e:	mov    rax,QWORD PTR [rsp+0x88]
   2ab36:	add    rax,r9
   2ab39:	mov    QWORD PTR [rsp+0x298],rax
   2ab41:	cmp    r8,0x4
   2ab45:	jbe    2aeae <mibz2::bwt::room_segments::<32, 1>+0x145e>
   2ab4b:	mov    rax,QWORD PTR [rsp+0x90]
   2ab53:	add    rax,r9
   2ab56:	mov    QWORD PTR [rsp+0x2b8],rax
   2ab5e:	cmp    r8,0x5
   2ab62:	jbe    2aec1 <mibz2::bwt::room_segments::<32, 1>+0x1471>
   2ab68:	mov    rax,QWORD PTR [rsp+0x98]
   2ab70:	add    rax,r9
   2ab73:	mov    QWORD PTR [rsp+0x2d8],rax
   2ab7b:	cmp    r8,0x6
   2ab7f:	jbe    2aed4 <mibz2::bwt::room_segments::<32, 1>+0x1484>
   2ab85:	mov    rax,QWORD PTR [rsp+0xa0]
   2ab8d:	add    rax,r9
   2ab90:	mov    QWORD PTR [rsp+0x2f8],rax
   2ab98:	cmp    r8,0x7
   2ab9c:	jbe    2aee7 <mibz2::bwt::room_segments::<32, 1>+0x1497>
   2aba2:	mov    rax,QWORD PTR [rsp+0xa8]
   2abaa:	add    rax,r9
   2abad:	mov    QWORD PTR [rsp+0x318],rax
   2abb5:	cmp    r8,0x8
   2abb9:	jbe    2aefa <mibz2::bwt::room_segments::<32, 1>+0x14aa>
   2abbf:	mov    rax,QWORD PTR [rsp+0xb0]
   2abc7:	add    rax,r9
   2abca:	mov    QWORD PTR [rsp+0x338],rax
   2abd2:	cmp    r8,0x9
   2abd6:	jbe    2af0d <mibz2::bwt::room_segments::<32, 1>+0x14bd>
   2abdc:	mov    rax,QWORD PTR [rsp+0xb8]
   2abe4:	add    rax,r9
   2abe7:	mov    QWORD PTR [rsp+0x358],rax
   2abef:	cmp    r8,0xa
   2abf3:	jbe    2af20 <mibz2::bwt::room_segments::<32, 1>+0x14d0>
   2abf9:	mov    rax,QWORD PTR [rsp+0xc0]
   2ac01:	add    rax,r9
   2ac04:	mov    QWORD PTR [rsp+0x378],rax
   2ac0c:	cmp    r8,0xb
   2ac10:	jbe    2af33 <mibz2::bwt::room_segments::<32, 1>+0x14e3>
   2ac16:	mov    rax,QWORD PTR [rsp+0xc8]
   2ac1e:	add    rax,r9
   2ac21:	mov    QWORD PTR [rsp+0x398],rax
   2ac29:	cmp    r8,0xc
   2ac2d:	jbe    2af46 <mibz2::bwt::room_segments::<32, 1>+0x14f6>
   2ac33:	mov    rax,QWORD PTR [rsp+0xd0]
   2ac3b:	add    rax,r9
   2ac3e:	mov    QWORD PTR [rsp+0x3b8],rax
   2ac46:	cmp    r8,0xd
   2ac4a:	jbe    2af59 <mibz2::bwt::room_segments::<32, 1>+0x1509>
   2ac50:	mov    rsi,r9
   2ac53:	mov    rax,QWORD PTR [rsp+0xd8]
   2ac5b:	add    rax,r9
   2ac5e:	mov    QWORD PTR [rsp+0x3d8],rax
   2ac66:	cmp    r8,0xe
   2ac6a:	jbe    2af6c <mibz2::bwt::room_segments::<32, 1>+0x151c>
   2ac70:	mov    rax,QWORD PTR [rsp+0xe0]
   2ac78:	add    rax,rsi
   2ac7b:	mov    QWORD PTR [rsp+0x3f8],rax
   2ac83:	cmp    r8,0xf
   2ac87:	jbe    2af7f <mibz2::bwt::room_segments::<32, 1>+0x152f>
   2ac8d:	mov    rax,QWORD PTR [rsp+0xe8]
   2ac95:	add    rax,rsi
   2ac98:	mov    QWORD PTR [rsp+0x418],rax
   2aca0:	cmp    r8,0x10
   2aca4:	jbe    2af92 <mibz2::bwt::room_segments::<32, 1>+0x1542>
   2acaa:	mov    rax,QWORD PTR [rsp+0xf0]
   2acb2:	add    rax,rsi
   2acb5:	mov    QWORD PTR [rsp+0x438],rax
   2acbd:	cmp    r8,0x11
   2acc1:	jbe    2afa5 <mibz2::bwt::room_segments::<32, 1>+0x1555>
   2acc7:	mov    rax,QWORD PTR [rsp+0xf8]
   2accf:	add    rax,rsi
   2acd2:	mov    QWORD PTR [rsp+0x458],rax
   2acda:	cmp    r8,0x12
   2acde:	jbe    2afb8 <mibz2::bwt::room_segments::<32, 1>+0x1568>
   2ace4:	mov    rax,QWORD PTR [rsp+0x100]
   2acec:	add    rax,rsi
   2acef:	mov    QWORD PTR [rsp+0x478],rax
   2acf7:	cmp    r8,0x13
   2acfb:	jbe    2afcb <mibz2::bwt::room_segments::<32, 1>+0x157b>
   2ad01:	mov    rax,QWORD PTR [rsp+0x108]
   2ad09:	add    rax,rsi
   2ad0c:	mov    QWORD PTR [rsp+0x498],rax
   2ad14:	cmp    r8,0x14
   2ad18:	jbe    2afde <mibz2::bwt::room_segments::<32, 1>+0x158e>
   2ad1e:	mov    rax,QWORD PTR [rsp+0x110]
   2ad26:	add    rax,rsi
   2ad29:	mov    QWORD PTR [rsp+0x4b8],rax
   2ad31:	cmp    r8,0x15
   2ad35:	jbe    2aff1 <mibz2::bwt::room_segments::<32, 1>+0x15a1>
   2ad3b:	mov    rax,QWORD PTR [rsp+0x118]
   2ad43:	add    rax,rsi
   2ad46:	mov    QWORD PTR [rsp+0x4d8],rax
   2ad4e:	cmp    r8,0x16
   2ad52:	jbe    2b004 <mibz2::bwt::room_segments::<32, 1>+0x15b4>
   2ad58:	mov    rax,QWORD PTR [rsp+0x120]
   2ad60:	add    rax,rsi
   2ad63:	mov    QWORD PTR [rsp+0x4f8],rax
   2ad6b:	cmp    r8,0x17
   2ad6f:	jbe    2b017 <mibz2::bwt::room_segments::<32, 1>+0x15c7>
   2ad75:	mov    rax,QWORD PTR [rsp+0x128]
   2ad7d:	add    rax,rsi
   2ad80:	mov    QWORD PTR [rsp+0x518],rax
   2ad88:	cmp    r8,0x18
   2ad8c:	jbe    2b02a <mibz2::bwt::room_segments::<32, 1>+0x15da>
   2ad92:	mov    rax,QWORD PTR [rsp+0x130]
   2ad9a:	add    rax,rsi
   2ad9d:	mov    QWORD PTR [rsp+0x538],rax
   2ada5:	cmp    r8,0x19
   2ada9:	jbe    2b03d <mibz2::bwt::room_segments::<32, 1>+0x15ed>
   2adaf:	mov    rax,QWORD PTR [rsp+0x138]
   2adb7:	add    rax,rsi
   2adba:	mov    QWORD PTR [rsp+0x558],rax
   2adc2:	cmp    r8,0x1a
   2adc6:	jbe    2b050 <mibz2::bwt::room_segments::<32, 1>+0x1600>
   2adcc:	mov    rax,QWORD PTR [rsp+0x140]
   2add4:	add    rax,rsi
   2add7:	mov    QWORD PTR [rsp+0x578],rax
   2addf:	cmp    r8,0x1b
   2ade3:	jbe    2b063 <mibz2::bwt::room_segments::<32, 1>+0x1613>
   2ade9:	mov    rax,QWORD PTR [rsp+0x148]
   2adf1:	add    rax,rsi
   2adf4:	mov    QWORD PTR [rsp+0x598],rax
   2adfc:	cmp    r8,0x1c
   2ae00:	jbe    2b076 <mibz2::bwt::room_segments::<32, 1>+0x1626>
   2ae06:	mov    rax,QWORD PTR [rsp+0x150]
   2ae0e:	add    rax,rsi
   2ae11:	mov    QWORD PTR [rsp+0x5b8],rax
   2ae19:	cmp    r8,0x1d
   2ae1d:	jbe    2b089 <mibz2::bwt::room_segments::<32, 1>+0x1639>
   2ae23:	mov    rax,QWORD PTR [rsp+0x158]
   2ae2b:	add    rax,rsi
   2ae2e:	mov    QWORD PTR [rsp+0x5d8],rax
   2ae36:	cmp    r8,0x1e
   2ae3a:	jbe    2b09c <mibz2::bwt::room_segments::<32, 1>+0x164c>
   2ae40:	mov    rax,QWORD PTR [rsp+0x160]
   2ae48:	mov    rcx,rsi
   2ae4b:	add    rax,rsi
   2ae4e:	mov    QWORD PTR [rsp+0x5f8],rax
   2ae56:	cmp    r8,0x20
   2ae5a:	jae    2b09c <mibz2::bwt::room_segments::<32, 1>+0x164c>
   2ae60:	jmp    2b0ac <mibz2::bwt::room_segments::<32, 1>+0x165c>
   2ae65:	mov    rax,QWORD PTR [rsp+0x78]
   2ae6a:	add    rax,rcx
   2ae6d:	mov    QWORD PTR [rsp+0x258],rax
   2ae75:	mov    rax,QWORD PTR [rsp+0x80]
   2ae7d:	add    rax,rcx
   2ae80:	mov    QWORD PTR [rsp+0x278],rax
   2ae88:	mov    rax,QWORD PTR [rsp+0x88]
   2ae90:	add    rax,rcx
   2ae93:	mov    QWORD PTR [rsp+0x298],rax
   2ae9b:	mov    rax,QWORD PTR [rsp+0x90]
   2aea3:	add    rax,rcx
   2aea6:	mov    QWORD PTR [rsp+0x2b8],rax
   2aeae:	mov    rax,QWORD PTR [rsp+0x98]
   2aeb6:	add    rax,rcx
   2aeb9:	mov    QWORD PTR [rsp+0x2d8],rax
   2aec1:	mov    rax,QWORD PTR [rsp+0xa0]
   2aec9:	add    rax,rcx
   2aecc:	mov    QWORD PTR [rsp+0x2f8],rax
   2aed4:	mov    rax,QWORD PTR [rsp+0xa8]
   2aedc:	add    rax,rcx
   2aedf:	mov    QWORD PTR [rsp+0x318],rax
   2aee7:	mov    rax,QWORD PTR [rsp+0xb0]
   2aeef:	add    rax,rcx
   2aef2:	mov    QWORD PTR [rsp+0x338],rax
   2aefa:	mov    rax,QWORD PTR [rsp+0xb8]
   2af02:	add    rax,rcx
   2af05:	mov    QWORD PTR [rsp+0x358],rax
   2af0d:	mov    rax,QWORD PTR [rsp+0xc0]
   2af15:	add    rax,rcx
   2af18:	mov    QWORD PTR [rsp+0x378],rax
   2af20:	mov    rax,QWORD PTR [rsp+0xc8]
   2af28:	add    rax,rcx
   2af2b:	mov    QWORD PTR [rsp+0x398],rax
   2af33:	mov    rax,QWORD PTR [rsp+0xd0]
   2af3b:	add    rax,rcx
   2af3e:	mov    QWORD PTR [rsp+0x3b8],rax
   2af46:	mov    rax,QWORD PTR [rsp+0xd8]
   2af4e:	add    rax,rcx
   2af51:	mov    QWORD PTR [rsp+0x3d8],rax
   2af59:	mov    rax,QWORD PTR [rsp+0xe0]
   2af61:	add    rax,rcx
   2af64:	mov    QWORD PTR [rsp+0x3f8],rax
   2af6c:	mov    rax,QWORD PTR [rsp+0xe8]
   2af74:	add    rax,rcx
   2af77:	mov    QWORD PTR [rsp+0x418],rax
   2af7f:	mov    rax,QWORD PTR [rsp+0xf0]
   2af87:	add    rax,rcx
   2af8a:	mov    QWORD PTR [rsp+0x438],rax
   2af92:	mov    rax,QWORD PTR [rsp+0xf8]
   2af9a:	add    rax,rcx
   2af9d:	mov    QWORD PTR [rsp+0x458],rax
   2afa5:	mov    rax,QWORD PTR [rsp+0x100]
   2afad:	add    rax,rcx
   2afb0:	mov    QWORD PTR [rsp+0x478],rax
   2afb8:	mov    rax,QWORD PTR [rsp+0x108]
   2afc0:	add    rax,rcx
   2afc3:	mov    QWORD PTR [rsp+0x498],rax
   2afcb:	mov    rax,QWORD PTR [rsp+0x110]
   2afd3:	add    rax,rcx
   2afd6:	mov    QWORD PTR [rsp+0x4b8],rax
   2afde:	mov    rax,QWORD PTR [rsp+0x118]
   2afe6:	add    rax,rcx
   2afe9:	mov    QWORD PTR [rsp+0x4d8],rax
   2aff1:	mov    rax,QWORD PTR [rsp+0x120]
   2aff9:	add    rax,rcx
   2affc:	mov    QWORD PTR [rsp+0x4f8],rax
   2b004:	mov    rax,QWORD PTR [rsp+0x128]
   2b00c:	add    rax,rcx
   2b00f:	mov    QWORD PTR [rsp+0x518],rax
   2b017:	mov    rax,QWORD PTR [rsp+0x130]
   2b01f:	add    rax,rcx
   2b022:	mov    QWORD PTR [rsp+0x538],rax
   2b02a:	mov    rax,QWORD PTR [rsp+0x138]
   2b032:	add    rax,rcx
   2b035:	mov    QWORD PTR [rsp+0x558],rax
   2b03d:	mov    rax,QWORD PTR [rsp+0x140]
   2b045:	add    rax,rcx
   2b048:	mov    QWORD PTR [rsp+0x578],rax
   2b050:	mov    rax,QWORD PTR [rsp+0x148]
   2b058:	add    rax,rcx
   2b05b:	mov    QWORD PTR [rsp+0x598],rax
   2b063:	mov    rax,QWORD PTR [rsp+0x150]
   2b06b:	add    rax,rcx
   2b06e:	mov    QWORD PTR [rsp+0x5b8],rax
   2b076:	mov    rax,QWORD PTR [rsp+0x158]
   2b07e:	add    rax,rcx
   2b081:	mov    QWORD PTR [rsp+0x5d8],rax
   2b089:	mov    rax,QWORD PTR [rsp+0x160]
   2b091:	add    rax,rcx
   2b094:	mov    QWORD PTR [rsp+0x5f8],rax
   2b09c:	add    rcx,QWORD PTR [rsp+0x168]
   2b0a4:	mov    QWORD PTR [rsp+0x618],rcx
   2b0ac:	shl    rdx,0x5
   2b0b0:	lea    r12,[rsp+rdx*1]
   2b0b4:	add    r12,0x230
   2b0bb:	lea    r14,[rsp+0x230]
   2b0c3:	mov    rax,QWORD PTR [rsp+0x50]
   2b0c8:	jmp    2b0dd <mibz2::bwt::room_segments::<32, 1>+0x168d>
   2b0ca:	nop    WORD PTR [rax+rax*1+0x0]
   2b0d0:	mov    r14,rbp
   2b0d3:	add    r13,0xffffffffffffffe0
   2b0d7:	je     2b25b <mibz2::bwt::room_segments::<32, 1>+0x180b>
   2b0dd:	lea    rbp,[r14+0x20]
   2b0e1:	mov    rbx,QWORD PTR [r14]
   2b0e4:	cmp    rbx,0xffffffffffffffff
   2b0e8:	je     2b0d0 <mibz2::bwt::room_segments::<32, 1>+0x1680>
   2b0ea:	cmp    rbx,QWORD PTR [rsp+0x10]
   2b0ef:	jae    2b829 <mibz2::bwt::room_segments::<32, 1>+0x1dd9>
   2b0f5:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   2b0fa:	jne    2b0d0 <mibz2::bwt::room_segments::<32, 1>+0x1680>
   2b0fc:	mov    edi,0x80
   2b101:	call   QWORD PTR [rip+0x41309]        # 6c410 <malloc@GLIBC_2.2.5>
   2b107:	test   rax,rax
   2b10a:	je     2b866 <mibz2::bwt::room_segments::<32, 1>+0x1e16>
   2b110:	mov    rcx,QWORD PTR [r14+0x18]
   2b114:	mov    QWORD PTR [rax+0x18],rcx
   2b118:	movups xmm0,XMMWORD PTR [r14+0x8]
   2b11d:	movups XMMWORD PTR [rax+0x8],xmm0
   2b121:	mov    QWORD PTR [rax],rbx
   2b124:	mov    QWORD PTR [rsp+0x178],0x4
   2b130:	mov    QWORD PTR [rsp+0x180],rax
   2b138:	mov    QWORD PTR [rsp+0x188],0x1
   2b144:	mov    r13d,0x1
   2b14a:	cmp    rbp,r12
   2b14d:	je     2b217 <mibz2::bwt::room_segments::<32, 1>+0x17c7>
   2b153:	lea    r14,[rsp+0x178]
   2b15b:	mov    rdx,QWORD PTR [rsp+0x30]
   2b160:	mov    rcx,QWORD PTR [rdx+0x50]
   2b164:	mov    rsi,QWORD PTR [rdx+0x58]
   2b168:	mov    rdx,rbp
   2b16b:	jmp    2b17c <mibz2::bwt::room_segments::<32, 1>+0x172c>
   2b16d:	nop    DWORD PTR [rax]
   2b170:	mov    rdx,rbp
   2b173:	cmp    rbp,r12
   2b176:	je     2b217 <mibz2::bwt::room_segments::<32, 1>+0x17c7>
   2b17c:	lea    rbp,[rdx+0x20]
   2b180:	mov    rbx,QWORD PTR [rdx]
   2b183:	cmp    rbx,0xffffffffffffffff
   2b187:	je     2b170 <mibz2::bwt::room_segments::<32, 1>+0x1720>
   2b189:	cmp    rbx,rsi
   2b18c:	jae    2b83d <mibz2::bwt::room_segments::<32, 1>+0x1ded>
   2b192:	cmp    QWORD PTR [rcx+rbx*8],0xffffffffffffffff
   2b197:	jne    2b170 <mibz2::bwt::room_segments::<32, 1>+0x1720>
   2b199:	mov    rcx,QWORD PTR [rdx+0x18]
   2b19d:	mov    QWORD PTR [rsp+0x640],rcx
   2b1a5:	movups xmm0,XMMWORD PTR [rdx+0x8]
   2b1a9:	movaps XMMWORD PTR [rsp+0x630],xmm0
   2b1b1:	cmp    r13,QWORD PTR [rsp+0x178]
   2b1b9:	jne    2b1de <mibz2::bwt::room_segments::<32, 1>+0x178e>
   2b1bb:	mov    edx,0x1
   2b1c0:	mov    ecx,0x8
   2b1c5:	mov    r8d,0x20
   2b1cb:	mov    rdi,r14
   2b1ce:	mov    rsi,r13
   2b1d1:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b1d6:	mov    rax,QWORD PTR [rsp+0x180]
   2b1de:	mov    rcx,r13
   2b1e1:	shl    rcx,0x5
   2b1e5:	mov    QWORD PTR [rax+rcx*1],rbx
   2b1e9:	movaps xmm0,XMMWORD PTR [rsp+0x630]
   2b1f1:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   2b1f6:	mov    rdx,QWORD PTR [rsp+0x640]
   2b1fe:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   2b203:	inc    r13
   2b206:	mov    QWORD PTR [rsp+0x188],r13
   2b20e:	cmp    rbp,r12
   2b211:	jne    2b15b <mibz2::bwt::room_segments::<32, 1>+0x170b>
   2b217:	mov    rax,QWORD PTR [rsp+0x178]
   2b21f:	mov    QWORD PTR [rsp+0x18],rax
   2b224:	mov    rax,QWORD PTR [rsp+0x180]
   2b22c:	mov    QWORD PTR [rsp+0x8],rax
   2b231:	mov    rbp,r13
   2b234:	shl    rbp,0x5
   2b238:	mov    rax,QWORD PTR [rsp+0x30]
   2b23d:	mov    r14,QWORD PTR [rax+0x38]
   2b241:	mov    rax,QWORD PTR [rax+0x40]
   2b245:	mov    QWORD PTR [rsp+0x28],rax
   2b24a:	xor    r12d,r12d
   2b24d:	lea    rax,[rip+0x3f5b4]        # 6a808 <__frame_dummy_init_array_entry+0x8a8>
   2b254:	mov    QWORD PTR [rsp+0x38],rax
   2b259:	jmp    2b2be <mibz2::bwt::room_segments::<32, 1>+0x186e>
   2b25b:	mov    eax,0x8
   2b260:	mov    QWORD PTR [rsp+0x8],rax
   2b265:	xor    r14d,r14d
   2b268:	mov    r9,QWORD PTR [rsp+0x20]
   2b26d:	jmp    2b504 <mibz2::bwt::room_segments::<32, 1>+0x1ab4>
   2b272:	mov    rdi,r12
   2b275:	call   2ba10 <<alloc::raw_vec::RawVec<u32>>::grow_one>
   2b27a:	mov    edi,DWORD PTR [rsp+0x4c]
   2b27e:	mov    r14,QWORD PTR [r12+0x8]
   2b283:	jmp    29b05 <mibz2::bwt::room_segments::<32, 1>+0xb5>
   2b288:	mov    rax,QWORD PTR [rsp+0x20]
   2b28d:	mov    rdx,QWORD PTR [rax]
   2b290:	mov    rax,QWORD PTR [rax+0x8]
   2b294:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2b2a0:	add    rbx,rax
   2b2a3:	mov    rcx,QWORD PTR [rsp+0x8]
   2b2a8:	mov    QWORD PTR [rcx+r12*1+0x8],rbx
   2b2ad:	add    rdx,rax
   2b2b0:	mov    QWORD PTR [rcx+r12*1+0x10],rdx
   2b2b5:	add    r12,0x20
   2b2b9:	cmp    rbp,r12
   2b2bc:	je     2b324 <mibz2::bwt::room_segments::<32, 1>+0x18d4>
   2b2be:	mov    rax,QWORD PTR [rsp+0x8]
   2b2c3:	mov    rbx,QWORD PTR [rax+r12*1+0x8]
   2b2c8:	cmp    rbx,QWORD PTR [rax+r12*1+0x10]
   2b2cd:	jne    2b2b5 <mibz2::bwt::room_segments::<32, 1>+0x1865>
   2b2cf:	mov    rax,QWORD PTR [rsp+0x8]
   2b2d4:	mov    rdi,QWORD PTR [rax+r12*1]
   2b2d8:	cmp    rdi,QWORD PTR [rsp+0x28]
   2b2dd:	jae    2b7ae <mibz2::bwt::room_segments::<32, 1>+0x1d5e>
   2b2e3:	lea    rcx,[rdi+rdi*2]
   2b2e7:	mov    rdx,QWORD PTR [r14+rcx*8]
   2b2eb:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   2b2f0:	sub    rbx,rax
   2b2f3:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   2b2f8:	mov    rsi,rdx
   2b2fb:	sub    rsi,rbx
   2b2fe:	cmp    rdx,rsi
   2b301:	jbe    2b2a0 <mibz2::bwt::room_segments::<32, 1>+0x1850>
   2b303:	lea    rdi,[r14+rcx*8]
   2b307:	mov    ecx,0x1
   2b30c:	mov    r8d,0x1
   2b312:	mov    QWORD PTR [rsp+0x20],rdi
   2b317:	mov    rsi,rbx
   2b31a:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b31f:	jmp    2b288 <mibz2::bwt::room_segments::<32, 1>+0x1838>
   2b324:	mov    rax,QWORD PTR [rsp+0x30]
   2b329:	mov    rcx,QWORD PTR [rax+0x50]
   2b32d:	mov    QWORD PTR [rsp+0x40],rcx
   2b332:	mov    rcx,QWORD PTR [rax+0x58]
   2b336:	mov    QWORD PTR [rsp+0x60],rcx
   2b33b:	mov    rcx,QWORD PTR [rax+0x8]
   2b33f:	mov    QWORD PTR [rsp+0x20],rcx
   2b344:	mov    rax,QWORD PTR [rax+0x10]
   2b348:	mov    QWORD PTR [rsp+0x68],rax
   2b34d:	xor    ebp,ebp
   2b34f:	jmp    2b362 <mibz2::bwt::room_segments::<32, 1>+0x1912>
   2b351:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2b360:	xor    ebp,ebp
   2b362:	mov    r12,rbp
   2b365:	shl    r12,0x5
   2b369:	add    r12,QWORD PTR [rsp+0x8]
   2b36e:	jmp    2b39c <mibz2::bwt::room_segments::<32, 1>+0x194c>
   2b370:	mov    rax,QWORD PTR [rsp+0x10]
   2b375:	mov    rdx,QWORD PTR [rax]
   2b378:	mov    rax,QWORD PTR [rax+0x8]
   2b37c:	nop    DWORD PTR [rax+0x0]
   2b380:	add    rbx,rax
   2b383:	mov    QWORD PTR [r12+0x8],rbx
   2b388:	add    rdx,rax
   2b38b:	mov    QWORD PTR [r12+0x10],rdx
   2b390:	inc    rbp
   2b393:	add    r12,0x20
   2b397:	cmp    rbp,r13
   2b39a:	jae    2b360 <mibz2::bwt::room_segments::<32, 1>+0x1910>
   2b39c:	mov    eax,DWORD PTR [r12+0x18]
   2b3a1:	mov    ecx,DWORD PTR [r15+rax*4]
   2b3a5:	test   ecx,ecx
   2b3a7:	js     2b430 <mibz2::bwt::room_segments::<32, 1>+0x19e0>
   2b3ad:	mov    eax,ecx
   2b3af:	shr    eax,0x8
   2b3b2:	mov    DWORD PTR [r12+0x18],eax
   2b3b7:	prefetcht0 BYTE PTR [r15+rax*4]
   2b3bc:	mov    rax,QWORD PTR [r12+0x8]
   2b3c1:	mov    BYTE PTR [rax],cl
   2b3c3:	mov    rbx,QWORD PTR [r12+0x8]
   2b3c8:	inc    rbx
   2b3cb:	mov    QWORD PTR [r12+0x8],rbx
   2b3d0:	cmp    rbx,QWORD PTR [r12+0x10]
   2b3d5:	jne    2b390 <mibz2::bwt::room_segments::<32, 1>+0x1940>
   2b3d7:	mov    rdi,QWORD PTR [r12]
   2b3db:	cmp    rdi,QWORD PTR [rsp+0x28]
   2b3e0:	jae    2b7ae <mibz2::bwt::room_segments::<32, 1>+0x1d5e>
   2b3e6:	lea    rcx,[rdi+rdi*2]
   2b3ea:	mov    rdx,QWORD PTR [r14+rcx*8]
   2b3ee:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   2b3f3:	sub    rbx,rax
   2b3f6:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   2b3fb:	mov    rsi,rdx
   2b3fe:	sub    rsi,rbx
   2b401:	cmp    rdx,rsi
   2b404:	jbe    2b380 <mibz2::bwt::room_segments::<32, 1>+0x1930>
   2b40a:	lea    rdi,[r14+rcx*8]
   2b40e:	mov    ecx,0x1
   2b413:	mov    r8d,0x1
   2b419:	mov    QWORD PTR [rsp+0x10],rdi
   2b41e:	mov    rsi,rbx
   2b421:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b426:	jmp    2b370 <mibz2::bwt::room_segments::<32, 1>+0x1920>
   2b42b:	nop    DWORD PTR [rax+rax*1+0x0]
   2b430:	mov    rdi,QWORD PTR [r12]
   2b434:	cmp    rdi,QWORD PTR [rsp+0x28]
   2b439:	jae    2b78f <mibz2::bwt::room_segments::<32, 1>+0x1d3f>
   2b43f:	lea    rcx,[rdi+rdi*2]
   2b443:	mov    rdx,QWORD PTR [r12+0x8]
   2b448:	sub    rdx,QWORD PTR [r14+rcx*8+0x8]
   2b44d:	mov    QWORD PTR [r14+rcx*8+0x10],rdx
   2b452:	mov    rdx,QWORD PTR [rsp+0x68]
   2b457:	test   rdx,rdx
   2b45a:	je     2b46b <mibz2::bwt::room_segments::<32, 1>+0x1a1b>
   2b45c:	xor    ecx,ecx
   2b45e:	cmp    rdx,0x1
   2b462:	jne    2b474 <mibz2::bwt::room_segments::<32, 1>+0x1a24>
   2b464:	mov    r9,QWORD PTR [rsp+0x20]
   2b469:	jmp    2b49d <mibz2::bwt::room_segments::<32, 1>+0x1a4d>
   2b46b:	mov    rcx,0xffffffffffffffff
   2b472:	jmp    2b4ac <mibz2::bwt::room_segments::<32, 1>+0x1a5c>
   2b474:	mov    r9,QWORD PTR [rsp+0x20]
   2b479:	nop    DWORD PTR [rax+0x0]
   2b480:	mov    rsi,rcx
   2b483:	mov    r8,rdx
   2b486:	shr    r8,1
   2b489:	add    rcx,r8
   2b48c:	cmp    DWORD PTR [r9+rcx*4],eax
   2b490:	cmova  rcx,rsi
   2b494:	sub    rdx,r8
   2b497:	cmp    rdx,0x1
   2b49b:	ja     2b480 <mibz2::bwt::room_segments::<32, 1>+0x1a30>
   2b49d:	cmp    DWORD PTR [r9+rcx*4],eax
   2b4a1:	mov    rax,0xffffffffffffffff
   2b4a8:	cmovne rcx,rax
   2b4ac:	cmp    rdi,QWORD PTR [rsp+0x60]
   2b4b1:	jae    2b798 <mibz2::bwt::room_segments::<32, 1>+0x1d48>
   2b4b7:	mov    rax,QWORD PTR [rsp+0x40]
   2b4bc:	mov    QWORD PTR [rax+rdi*8],rcx
   2b4c0:	dec    r13
   2b4c3:	mov    rax,r13
   2b4c6:	shl    rax,0x5
   2b4ca:	mov    rcx,QWORD PTR [rsp+0x8]
   2b4cf:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   2b4d3:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   2b4d8:	movups XMMWORD PTR [r12+0x10],xmm1
   2b4de:	movups XMMWORD PTR [r12],xmm0
   2b4e3:	cmp    rbp,r13
   2b4e6:	jb     2b362 <mibz2::bwt::room_segments::<32, 1>+0x1912>
   2b4ec:	mov    ebp,0x0
   2b4f1:	test   r13,r13
   2b4f4:	jne    2b362 <mibz2::bwt::room_segments::<32, 1>+0x1912>
   2b4fa:	mov    r9,QWORD PTR [rsp+0x20]
   2b4ff:	mov    r14,QWORD PTR [rsp+0x18]
   2b504:	mov    rax,0xffffffffffffffff
   2b50b:	mov    r10,QWORD PTR [rsp+0x68]
   2b510:	test   r10,r10
   2b513:	mov    rsi,QWORD PTR [rsp+0x170]
   2b51b:	mov    r8,QWORD PTR [rsp+0x30]
   2b520:	je     2b59c <mibz2::bwt::room_segments::<32, 1>+0x1b4c>
   2b522:	lea    rcx,[r10*4+0x0]
   2b52a:	xor    edx,edx
   2b52c:	nop    DWORD PTR [rax+0x0]
   2b530:	mov    edi,DWORD PTR [r9+rdx*1]
   2b534:	cmp    rsi,rdi
   2b537:	jbe    2b7c7 <mibz2::bwt::room_segments::<32, 1>+0x1d77>
   2b53d:	and    BYTE PTR [r15+rdi*4+0x3],0x7f
   2b543:	add    rdx,0x4
   2b547:	cmp    rcx,rdx
   2b54a:	jne    2b530 <mibz2::bwt::room_segments::<32, 1>+0x1ae0>
   2b54c:	test   r10,r10
   2b54f:	je     2b59c <mibz2::bwt::room_segments::<32, 1>+0x1b4c>
   2b551:	xor    ecx,ecx
   2b553:	cmp    r10,0x1
   2b557:	jne    2b55f <mibz2::bwt::room_segments::<32, 1>+0x1b0f>
   2b559:	mov    edi,DWORD PTR [rsp+0x4c]
   2b55d:	jmp    2b58d <mibz2::bwt::room_segments::<32, 1>+0x1b3d>
   2b55f:	mov    edi,DWORD PTR [rsp+0x4c]
   2b563:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   2b570:	mov    rax,rcx
   2b573:	mov    rdx,r10
   2b576:	shr    rdx,1
   2b579:	add    rcx,rdx
   2b57c:	cmp    DWORD PTR [r9+rcx*4],edi
   2b580:	cmova  rcx,rax
   2b584:	sub    r10,rdx
   2b587:	cmp    r10,0x1
   2b58b:	ja     2b570 <mibz2::bwt::room_segments::<32, 1>+0x1b20>
   2b58d:	cmp    DWORD PTR [r9+rcx*4],edi
   2b591:	mov    rax,0xffffffffffffffff
   2b598:	cmove  rax,rcx
   2b59c:	mov    QWORD PTR [r8+0x68],rax
   2b5a0:	mov    QWORD PTR [r8+0x60],rsi
   2b5a4:	test   r14,r14
   2b5a7:	je     2b5b4 <mibz2::bwt::room_segments::<32, 1>+0x1b64>
   2b5a9:	mov    rdi,QWORD PTR [rsp+0x8]
   2b5ae:	call   QWORD PTR [rip+0x40e54]        # 6c408 <free@GLIBC_2.2.5>
   2b5b4:	add    rsp,0x648
   2b5bb:	pop    rbx
   2b5bc:	pop    r12
   2b5be:	pop    r13
   2b5c0:	pop    r14
   2b5c2:	pop    r15
   2b5c4:	pop    rbp
   2b5c5:	ret
   2b5c6:	mov    edx,0x1ff
   2b5cb:	mov    r14,rcx
   2b5ce:	mov    ecx,0x4
   2b5d3:	mov    r8d,0x4
   2b5d9:	mov    ebp,edi
   2b5db:	mov    rdi,r14
   2b5de:	mov    rbx,rsi
   2b5e1:	xor    esi,esi
   2b5e3:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b5e8:	mov    edi,ebp
   2b5ea:	mov    rcx,r14
   2b5ed:	mov    rsi,rbx
   2b5f0:	mov    r13,QWORD PTR [r14+0x10]
   2b5f4:	jmp    29a95 <mibz2::bwt::room_segments::<32, 1>+0x45>
   2b5f9:	cmp    r9,0x15
   2b5fd:	mov    r12,QWORD PTR [rsp+0x170]
   2b605:	jae    2b875 <mibz2::bwt::room_segments::<32, 1>+0x1e25>
   2b60b:	lea    rax,[r14+r9*4]
   2b60f:	lea    rcx,[r14+0x4]
   2b613:	mov    edx,0x4
   2b618:	mov    rbp,QWORD PTR [rsp+0x30]
   2b61d:	jmp    2b635 <mibz2::bwt::room_segments::<32, 1>+0x1be5>
   2b61f:	mov    rdi,r14
   2b622:	mov    DWORD PTR [rdi],esi
   2b624:	add    rcx,0x4
   2b628:	add    rdx,0x4
   2b62c:	cmp    rcx,rax
   2b62f:	je     2b88b <mibz2::bwt::room_segments::<32, 1>+0x1e3b>
   2b635:	mov    r8d,DWORD PTR [rcx-0x4]
   2b639:	mov    esi,DWORD PTR [rcx]
   2b63b:	cmp    esi,r8d
   2b63e:	jae    2b624 <mibz2::bwt::room_segments::<32, 1>+0x1bd4>
   2b640:	mov    rdi,rdx
   2b643:	mov    DWORD PTR [r14+rdi*1],r8d
   2b647:	cmp    rdi,0x4
   2b64b:	je     2b61f <mibz2::bwt::room_segments::<32, 1>+0x1bcf>
   2b64d:	mov    r8d,DWORD PTR [r14+rdi*1-0x8]
   2b652:	add    rdi,0xfffffffffffffffc
   2b656:	cmp    esi,r8d
   2b659:	jb     2b643 <mibz2::bwt::room_segments::<32, 1>+0x1bf3>
   2b65b:	add    rdi,r14
   2b65e:	jmp    2b622 <mibz2::bwt::room_segments::<32, 1>+0x1bd2>
   2b660:	lea    rdi,[rbp+0x48]
   2b664:	mov    ecx,0x8
   2b669:	mov    r8d,0x8
   2b66f:	xor    esi,esi
   2b671:	mov    rdx,r9
   2b674:	mov    rbx,rax
   2b677:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b67c:	mov    rax,rbx
   2b67f:	mov    r9,QWORD PTR [rsp+0x58]
   2b684:	mov    rbx,QWORD PTR [rbp+0x58]
   2b688:	mov    r14,QWORD PTR [rbp+0x50]
   2b68c:	lea    rdi,[r14+rbx*8]
   2b690:	cmp    r9,0x1
   2b694:	jne    29c58 <mibz2::bwt::room_segments::<32, 1>+0x208>
   2b69a:	jmp    29c88 <mibz2::bwt::room_segments::<32, 1>+0x238>
   2b69f:	lea    rdi,[rbp+0x30]
   2b6a3:	mov    ecx,0x8
   2b6a8:	mov    r8d,0x18
   2b6ae:	mov    rsi,rbx
   2b6b1:	mov    rdx,r14
   2b6b4:	mov    r12,rax
   2b6b7:	call   2b990 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2b6bc:	mov    rax,r12
   2b6bf:	mov    r9,QWORD PTR [rsp+0x58]
   2b6c4:	mov    rdx,QWORD PTR [rbp+0x40]
   2b6c8:	mov    rsi,QWORD PTR [rbp+0x38]
   2b6cc:	inc    rbx
   2b6cf:	test   r14b,0x1
   2b6d3:	je     29bb8 <mibz2::bwt::room_segments::<32, 1>+0x168>
   2b6d9:	dec    r14
   2b6dc:	lea    rcx,[rdx+rdx*2]
   2b6e0:	mov    QWORD PTR [rsi+rcx*8],0x0
   2b6e8:	mov    QWORD PTR [rsi+rcx*8+0x8],0x1
   2b6f1:	mov    QWORD PTR [rsi+rcx*8+0x10],0x0
   2b6fa:	inc    rdx
   2b6fd:	cmp    r9,rbx
   2b700:	je     29bc1 <mibz2::bwt::room_segments::<32, 1>+0x171>
   2b706:	lea    rcx,[rdx+rdx*2]
   2b70a:	lea    rsi,[rsi+rcx*8]
   2b70e:	add    rsi,0x28
   2b712:	neg    r14
   2b715:	xor    ecx,ecx
   2b717:	xorps  xmm0,xmm0
   2b71a:	nop    WORD PTR [rax+rax*1+0x0]
   2b720:	mov    QWORD PTR [rsi-0x28],0x0
   2b728:	mov    QWORD PTR [rsi-0x20],0x1
   2b730:	movups XMMWORD PTR [rsi-0x18],xmm0
   2b734:	mov    QWORD PTR [rsi-0x8],0x1
   2b73c:	mov    QWORD PTR [rsi],0x0
   2b743:	add    rsi,0x30
   2b747:	add    rcx,0xfffffffffffffffe
   2b74b:	cmp    r14,rcx
   2b74e:	jne    2b720 <mibz2::bwt::room_segments::<32, 1>+0x1cd0>
   2b750:	sub    rdx,rcx
   2b753:	mov    QWORD PTR [rbp+0x40],rdx
   2b757:	cmp    r9,rdx
   2b75a:	jbe    29bce <mibz2::bwt::room_segments::<32, 1>+0x17e>
   2b760:	jmp    2aaa4 <mibz2::bwt::room_segments::<32, 1>+0x1054>
   2b765:	lea    rdx,[rip+0x3ef64]        # 6a6d0 <__frame_dummy_init_array_entry+0x770>
   2b76c:	mov    rsi,r12
   2b76f:	call   2046b <core::panicking::panic_bounds_check>
   2b774:	lea    rdx,[rip+0x3f0bd]        # 6a838 <__frame_dummy_init_array_entry+0x8d8>
   2b77b:	call   2046b <core::panicking::panic_bounds_check>
   2b780:	lea    rdx,[rip+0x3f0c9]        # 6a850 <__frame_dummy_init_array_entry+0x8f0>
   2b787:	mov    rsi,r14
   2b78a:	call   2046b <core::panicking::panic_bounds_check>
   2b78f:	lea    rax,[rip+0x3f042]        # 6a7d8 <__frame_dummy_init_array_entry+0x878>
   2b796:	jmp    2b7a9 <mibz2::bwt::room_segments::<32, 1>+0x1d59>
   2b798:	mov    rax,QWORD PTR [rsp+0x60]
   2b79d:	mov    QWORD PTR [rsp+0x28],rax
   2b7a2:	lea    rax,[rip+0x3f047]        # 6a7f0 <__frame_dummy_init_array_entry+0x890>
   2b7a9:	mov    QWORD PTR [rsp+0x38],rax
   2b7ae:	mov    rsi,QWORD PTR [rsp+0x28]
   2b7b3:	mov    rdx,QWORD PTR [rsp+0x38]
   2b7b8:	mov    r14,QWORD PTR [rsp+0x18]
   2b7bd:	call   2046b <core::panicking::panic_bounds_check>
   2b7c2:	jmp    2b84c <mibz2::bwt::room_segments::<32, 1>+0x1dfc>
   2b7c7:	lea    rdx,[rip+0x3ef32]        # 6a700 <__frame_dummy_init_array_entry+0x7a0>
   2b7ce:	call   2046b <core::panicking::panic_bounds_check>
   2b7d3:	jmp    2b84c <mibz2::bwt::room_segments::<32, 1>+0x1dfc>
   2b7d5:	lea    rdx,[rip+0x3f02c]        # 6a808 <__frame_dummy_init_array_entry+0x8a8>
   2b7dc:	call   2046b <core::panicking::panic_bounds_check>
   2b7e1:	lea    rdx,[rip+0x3eff0]        # 6a7d8 <__frame_dummy_init_array_entry+0x878>
   2b7e8:	mov    rsi,QWORD PTR [rsp+0x8]
   2b7ed:	call   2046b <core::panicking::panic_bounds_check>
   2b7f2:	lea    rdx,[rip+0x3eff7]        # 6a7f0 <__frame_dummy_init_array_entry+0x890>
   2b7f9:	mov    rsi,QWORD PTR [rsp+0x10]
   2b7fe:	call   2046b <core::panicking::panic_bounds_check>
   2b803:	lea    rdx,[rip+0x3f046]        # 6a850 <__frame_dummy_init_array_entry+0x8f0>
   2b80a:	mov    rdi,rbx
   2b80d:	mov    rsi,r14
   2b810:	call   2046b <core::panicking::panic_bounds_check>
   2b815:	lea    rdx,[rip+0x3f01c]        # 6a838 <__frame_dummy_init_array_entry+0x8d8>
   2b81c:	mov    rdi,rbx
   2b81f:	mov    rsi,QWORD PTR [rsp+0x8]
   2b824:	call   2046b <core::panicking::panic_bounds_check>
   2b829:	lea    rdx,[rip+0x3eff0]        # 6a820 <__frame_dummy_init_array_entry+0x8c0>
   2b830:	mov    rdi,rbx
   2b833:	mov    rsi,QWORD PTR [rsp+0x10]
   2b838:	call   2046b <core::panicking::panic_bounds_check>
   2b83d:	lea    rdx,[rip+0x3efdc]        # 6a820 <__frame_dummy_init_array_entry+0x8c0>
   2b844:	mov    rdi,rbx
   2b847:	call   2046b <core::panicking::panic_bounds_check>
   2b84c:	ud2
   2b84e:	lea    rdx,[rip+0x3ee63]        # 6a6b8 <__frame_dummy_init_array_entry+0x758>
   2b855:	call   2046b <core::panicking::panic_bounds_check>
   2b85a:	lea    rdi,[rip+0x3ee87]        # 6a6e8 <__frame_dummy_init_array_entry+0x788>
   2b861:	call   22fe0 <core::panicking::panic_const::panic_const_div_by_zero>
   2b866:	mov    edi,0x8
   2b86b:	mov    esi,0x80
   2b870:	call   1f0b4 <alloc::raw_vec::handle_error>
   2b875:	mov    rdi,r14
   2b878:	mov    rsi,r9
   2b87b:	mov    rbp,r9
   2b87e:	call   2ba80 <core::slice::sort::unstable::ipnsort::<u32, <u32 as core::cmp::PartialOrd>::lt>>
   2b883:	mov    r9,rbp
   2b886:	mov    rbp,QWORD PTR [rsp+0x30]
   2b88b:	xor    ecx,ecx
   2b88d:	mov    eax,DWORD PTR [r14+rcx*4+0x4]
   2b892:	cmp    eax,DWORD PTR [r14+rcx*4]
   2b896:	je     2b8a5 <mibz2::bwt::room_segments::<32, 1>+0x1e55>
   2b898:	inc    rcx
   2b89b:	cmp    rbx,rcx
   2b89e:	jne    2b88d <mibz2::bwt::room_segments::<32, 1>+0x1e3d>
   2b8a0:	jmp    29b29 <mibz2::bwt::room_segments::<32, 1>+0xd9>
   2b8a5:	lea    rdx,[rcx+0x1]
   2b8a9:	lea    rax,[rcx+0x2]
   2b8ad:	cmp    rax,r9
   2b8b0:	jae    2b8c6 <mibz2::bwt::room_segments::<32, 1>+0x1e76>
   2b8b2:	mov    esi,r13d
   2b8b5:	sub    esi,ecx
   2b8b7:	add    r13,0x1fd
   2b8be:	test   sil,0x1
   2b8c2:	jne    2b8cb <mibz2::bwt::room_segments::<32, 1>+0x1e7b>
   2b8c4:	jmp    2b8ec <mibz2::bwt::room_segments::<32, 1>+0x1e9c>
   2b8c6:	mov    r9,rdx
   2b8c9:	jmp    2b8f1 <mibz2::bwt::room_segments::<32, 1>+0x1ea1>
   2b8cb:	mov    eax,DWORD PTR [r14+rcx*4+0x8]
   2b8d0:	mov    r9,rdx
   2b8d3:	cmp    eax,DWORD PTR [r14+rcx*4]
   2b8d7:	je     2b8e2 <mibz2::bwt::room_segments::<32, 1>+0x1e92>
   2b8d9:	mov    DWORD PTR [r14+rcx*4+0x4],eax
   2b8de:	lea    r9,[rdx+0x1]
   2b8e2:	add    rdx,0x2
   2b8e6:	mov    rax,rdx
   2b8e9:	mov    rdx,r9
   2b8ec:	cmp    r13,rcx
   2b8ef:	jne    2b8fa <mibz2::bwt::room_segments::<32, 1>+0x1eaa>
   2b8f1:	mov    QWORD PTR [rbp+0x10],r9
   2b8f5:	jmp    29b29 <mibz2::bwt::room_segments::<32, 1>+0xd9>
   2b8fa:	inc    rbx
   2b8fd:	mov    r9,rdx
   2b900:	jmp    2b90b <mibz2::bwt::room_segments::<32, 1>+0x1ebb>
   2b902:	add    rax,0x2
   2b906:	cmp    rbx,rax
   2b909:	je     2b8f1 <mibz2::bwt::room_segments::<32, 1>+0x1ea1>
   2b90b:	mov    ecx,DWORD PTR [r14+rax*4]
   2b90f:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   2b914:	jne    2b924 <mibz2::bwt::room_segments::<32, 1>+0x1ed4>
   2b916:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   2b91b:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   2b920:	je     2b902 <mibz2::bwt::room_segments::<32, 1>+0x1eb2>
   2b922:	jmp    2b937 <mibz2::bwt::room_segments::<32, 1>+0x1ee7>
   2b924:	mov    DWORD PTR [r14+r9*4],ecx
   2b928:	inc    r9
   2b92b:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   2b930:	cmp    ecx,DWORD PTR [r14+r9*4-0x4]
   2b935:	je     2b902 <mibz2::bwt::room_segments::<32, 1>+0x1eb2>
   2b937:	mov    DWORD PTR [r14+r9*4],ecx
   2b93b:	inc    r9
   2b93e:	jmp    2b902 <mibz2::bwt::room_segments::<32, 1>+0x1eb2>
   2b940:	jmp    2b944 <mibz2::bwt::room_segments::<32, 1>+0x1ef4>
   2b942:	jmp    2b94e <mibz2::bwt::room_segments::<32, 1>+0x1efe>
   2b944:	mov    rbx,rax
   2b947:	mov    r14,QWORD PTR [rsp+0x18]
   2b94c:	jmp    2b96e <mibz2::bwt::room_segments::<32, 1>+0x1f1e>
   2b94e:	mov    rbx,rax
   2b951:	cmp    QWORD PTR [rsp+0x178],0x0
   2b95a:	je     2b97e <mibz2::bwt::room_segments::<32, 1>+0x1f2e>
   2b95c:	mov    rax,QWORD PTR [rsp+0x180]
   2b964:	mov    QWORD PTR [rsp+0x8],rax
   2b969:	jmp    2b973 <mibz2::bwt::room_segments::<32, 1>+0x1f23>
   2b96b:	mov    rbx,rax
   2b96e:	test   r14,r14
   2b971:	je     2b97e <mibz2::bwt::room_segments::<32, 1>+0x1f2e>
   2b973:	mov    rdi,QWORD PTR [rsp+0x8]
   2b978:	call   QWORD PTR [rip+0x40a8a]        # 6c408 <free@GLIBC_2.2.5>
   2b97e:	mov    rdi,rbx
   2b981:	call   68f40 <_Unwind_Resume@plt>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
