
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/tbcg/tiled/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

00000000000347e0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_>:
   347e0:	push   rbp
   347e1:	push   r15
   347e3:	push   r14
   347e5:	push   r13
   347e7:	push   r12
   347e9:	push   rbx
   347ea:	sub    rsp,0x698
   347f1:	mov    r10,rsi
   347f4:	mov    rax,rdi
   347f7:	mov    edi,edx
   347f9:	cmp    rsi,rdi
   347fc:	jbe    3621d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a3d>
   34802:	mov    QWORD PTR [rsp+0x8],rax
   34807:	mov    esi,DWORD PTR [rax+rdi*4]
   3480a:	mov    QWORD PTR [rcx+0x10],0x0
   34812:	movabs rbx,0x2545f4914f6cdd1d
   3481c:	cmp    QWORD PTR [rcx],0x1fe
   34823:	jbe    35f98 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17b8>
   34829:	xor    ebp,ebp
   3482b:	shr    esi,0x8
   3482e:	xor    rbx,r10
   34831:	mov    QWORD PTR [rsp+0x18],rcx
   34836:	mov    r14,QWORD PTR [rcx+0x8]
   3483a:	mov    ecx,0x1ff
   3483f:	mov    r12,rbp
   34842:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34850:	mov    r15,r12
   34853:	mov    rax,rbx
   34856:	shl    rax,0xd
   3485a:	xor    rax,rbx
   3485d:	mov    rdx,rax
   34860:	shr    rdx,0x7
   34864:	xor    rdx,rax
   34867:	mov    rbx,rdx
   3486a:	shl    rbx,0x11
   3486e:	xor    rbx,rdx
   34871:	mov    rax,rbx
   34874:	xor    edx,edx
   34876:	div    r10
   34879:	mov    DWORD PTR [r14+r12*4],edx
   3487d:	inc    r12
   34880:	dec    rcx
   34883:	jne    34850 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x70>
   34885:	mov    rbx,QWORD PTR [rsp+0x18]
   3488a:	mov    QWORD PTR [rbx+0x10],r12
   3488e:	cmp    r12,QWORD PTR [rbx]
   34891:	mov    DWORD PTR [rsp+0x3c],esi
   34895:	je     35c2f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x144f>
   3489b:	mov    DWORD PTR [r14+r12*4],esi
   3489f:	lea    rax,[r15+0x2]
   348a3:	mov    r13,rax
   348a6:	mov    QWORD PTR [rbx+0x10],rax
   348aa:	cmp    r15,0xfffffffffffffffd
   348ae:	jbe    35fcd <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17ed>
   348b4:	mov    r12,QWORD PTR [rsp+0x18]
   348b9:	mov    rdx,QWORD PTR [rsp+0x8]
   348be:	test   r13,r13
   348c1:	je     362d9 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1af9>
   348c7:	lea    rax,[r13*4+0x0]
   348cf:	xor    ecx,ecx
   348d1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   348e0:	mov    edi,DWORD PTR [r14+rcx*1]
   348e4:	cmp    r10,rdi
   348e7:	jbe    36116 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1936>
   348ed:	or     BYTE PTR [rdx+rdi*4+0x3],0x80
   348f2:	add    rcx,0x4
   348f6:	cmp    rax,rcx
   348f9:	jne    348e0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x100>
   348fb:	lea    rax,[r10+r10*1]
   348ff:	mov    rcx,rax
   34902:	or     rcx,r13
   34905:	shr    rcx,0x20
   34909:	je     34915 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x135>
   3490b:	xor    edx,edx
   3490d:	div    r13
   34910:	mov    r15,rax
   34913:	jmp    3491d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13d>
   34915:	xor    edx,edx
   34917:	div    r13d
   3491a:	mov    r15d,eax
   3491d:	mov    r14,QWORD PTR [r12+0x40]
   34922:	cmp    r14,r13
   34925:	mov    rbx,r13
   34928:	cmova  rbx,r14
   3492c:	jae    35947 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1167>
   34932:	sub    rbx,r14
   34935:	mov    rax,QWORD PTR [r12+0x30]
   3493a:	sub    rax,r14
   3493d:	mov    rdx,r14
   34940:	cmp    rbx,rax
   34943:	ja     36054 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1874>
   34949:	mov    rax,QWORD PTR [r12+0x38]
   3494e:	inc    r14
   34951:	test   bl,0x1
   34954:	jne    3608b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18ab>
   3495a:	cmp    r13,r14
   3495d:	jne    360b8 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18d8>
   34963:	mov    QWORD PTR [r12+0x40],rdx
   34968:	cmp    r13,rdx
   3496b:	ja     35958 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1178>
   34971:	mov    QWORD PTR [rsp+0x50],r15
   34976:	lea    rbx,[r15+0x40]
   3497a:	mov    r14,QWORD PTR [r12+0x38]
   3497f:	lea    rax,[r13*8+0x0]
   34987:	lea    r15,[rax+rax*2]
   3498b:	nop    DWORD PTR [rax+rax*1+0x0]
   34990:	mov    QWORD PTR [r14+0x10],0x0
   34998:	cmp    rbx,QWORD PTR [r14]
   3499b:	ja     349a9 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1c9>
   3499d:	add    r14,0x18
   349a1:	add    r15,0xffffffffffffffe8
   349a5:	jne    34990 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   349a7:	jmp    349d1 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1f1>
   349a9:	mov    ecx,0x1
   349ae:	mov    r8d,0x1
   349b4:	mov    rdi,r14
   349b7:	xor    esi,esi
   349b9:	mov    rdx,rbx
   349bc:	mov    rbp,r10
   349bf:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   349c4:	mov    r10,rbp
   349c7:	add    r14,0x18
   349cb:	add    r15,0xffffffffffffffe8
   349cf:	jne    34990 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0>
   349d1:	mov    QWORD PTR [r12+0x58],0x0
   349da:	cmp    r13,QWORD PTR [r12+0x48]
   349df:	ja     3602a <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x184a>
   349e5:	xor    ebx,ebx
   349e7:	mov    r15,QWORD PTR [rsp+0x50]
   349ec:	mov    r14,QWORD PTR [r12+0x50]
   349f1:	lea    rdi,[r14+rbx*8]
   349f5:	cmp    r13,0x1
   349f9:	je     34a26 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x246>
   349fb:	lea    rdx,[r13*8-0x8]
   34a03:	mov    esi,0xff
   34a08:	mov    rbp,r10
   34a0b:	call   QWORD PTR [rip+0x618a7]        # 962b8 <memset@GLIBC_2.2.5>
   34a11:	mov    r10,rbp
   34a14:	lea    rax,[rbx+r13*1]
   34a18:	lea    rdi,[r14+rax*8]
   34a1c:	add    rdi,0xfffffffffffffff8
   34a20:	add    rbx,r13
   34a23:	dec    rbx
   34a26:	mov    QWORD PTR [rdi],0xffffffffffffffff
   34a2d:	inc    rbx
   34a30:	mov    QWORD PTR [r12+0x58],rbx
   34a35:	xorps  xmm0,xmm0
   34a38:	movups XMMWORD PTR [rsp+0x2a0],xmm0
   34a40:	mov    DWORD PTR [rsp+0x2b0],0x0
   34a4b:	movups XMMWORD PTR [rsp+0x2c0],xmm0
   34a53:	mov    DWORD PTR [rsp+0x2d0],0x0
   34a5e:	movups XMMWORD PTR [rsp+0x2e0],xmm0
   34a66:	mov    DWORD PTR [rsp+0x2f0],0x0
   34a71:	movups XMMWORD PTR [rsp+0x300],xmm0
   34a79:	mov    DWORD PTR [rsp+0x310],0x0
   34a84:	movups XMMWORD PTR [rsp+0x320],xmm0
   34a8c:	mov    DWORD PTR [rsp+0x330],0x0
   34a97:	movups XMMWORD PTR [rsp+0x340],xmm0
   34a9f:	mov    DWORD PTR [rsp+0x350],0x0
   34aaa:	movups XMMWORD PTR [rsp+0x360],xmm0
   34ab2:	mov    DWORD PTR [rsp+0x370],0x0
   34abd:	movups XMMWORD PTR [rsp+0x380],xmm0
   34ac5:	mov    DWORD PTR [rsp+0x390],0x0
   34ad0:	movups XMMWORD PTR [rsp+0x3a0],xmm0
   34ad8:	mov    DWORD PTR [rsp+0x3b0],0x0
   34ae3:	movups XMMWORD PTR [rsp+0x3c0],xmm0
   34aeb:	mov    DWORD PTR [rsp+0x3d0],0x0
   34af6:	movups XMMWORD PTR [rsp+0x3e0],xmm0
   34afe:	mov    DWORD PTR [rsp+0x3f0],0x0
   34b09:	movups XMMWORD PTR [rsp+0x400],xmm0
   34b11:	mov    DWORD PTR [rsp+0x410],0x0
   34b1c:	movups XMMWORD PTR [rsp+0x420],xmm0
   34b24:	mov    DWORD PTR [rsp+0x430],0x0
   34b2f:	movups XMMWORD PTR [rsp+0x440],xmm0
   34b37:	mov    DWORD PTR [rsp+0x450],0x0
   34b42:	movups XMMWORD PTR [rsp+0x460],xmm0
   34b4a:	mov    DWORD PTR [rsp+0x470],0x0
   34b55:	movups XMMWORD PTR [rsp+0x480],xmm0
   34b5d:	mov    DWORD PTR [rsp+0x490],0x0
   34b68:	movups XMMWORD PTR [rsp+0x4a0],xmm0
   34b70:	mov    DWORD PTR [rsp+0x4b0],0x0
   34b7b:	movups XMMWORD PTR [rsp+0x4c0],xmm0
   34b83:	mov    DWORD PTR [rsp+0x4d0],0x0
   34b8e:	movups XMMWORD PTR [rsp+0x4e0],xmm0
   34b96:	mov    DWORD PTR [rsp+0x4f0],0x0
   34ba1:	movups XMMWORD PTR [rsp+0x500],xmm0
   34ba9:	mov    DWORD PTR [rsp+0x510],0x0
   34bb4:	movups XMMWORD PTR [rsp+0x520],xmm0
   34bbc:	mov    DWORD PTR [rsp+0x530],0x0
   34bc7:	movups XMMWORD PTR [rsp+0x540],xmm0
   34bcf:	mov    DWORD PTR [rsp+0x550],0x0
   34bda:	movups XMMWORD PTR [rsp+0x560],xmm0
   34be2:	mov    DWORD PTR [rsp+0x570],0x0
   34bed:	movups XMMWORD PTR [rsp+0x580],xmm0
   34bf5:	mov    DWORD PTR [rsp+0x590],0x0
   34c00:	movups XMMWORD PTR [rsp+0x5a0],xmm0
   34c08:	mov    DWORD PTR [rsp+0x5b0],0x0
   34c13:	movups XMMWORD PTR [rsp+0x5c0],xmm0
   34c1b:	mov    DWORD PTR [rsp+0x5d0],0x0
   34c26:	movups XMMWORD PTR [rsp+0x5e0],xmm0
   34c2e:	mov    DWORD PTR [rsp+0x5f0],0x0
   34c39:	movups XMMWORD PTR [rsp+0x600],xmm0
   34c41:	mov    DWORD PTR [rsp+0x610],0x0
   34c4c:	movups XMMWORD PTR [rsp+0x620],xmm0
   34c54:	mov    DWORD PTR [rsp+0x630],0x0
   34c5f:	movups XMMWORD PTR [rsp+0x640],xmm0
   34c67:	mov    DWORD PTR [rsp+0x650],0x0
   34c72:	movups XMMWORD PTR [rsp+0x660],xmm0
   34c7a:	mov    DWORD PTR [rsp+0x670],0x0
   34c85:	movups XMMWORD PTR [rsp+0x680],xmm0
   34c8d:	mov    DWORD PTR [rsp+0x690],0x0
   34c98:	mov    QWORD PTR [rsp+0x298],0xffffffffffffffff
   34ca4:	mov    QWORD PTR [rsp+0x2b8],0xffffffffffffffff
   34cb0:	mov    QWORD PTR [rsp+0x2d8],0xffffffffffffffff
   34cbc:	mov    QWORD PTR [rsp+0x2f8],0xffffffffffffffff
   34cc8:	mov    QWORD PTR [rsp+0x318],0xffffffffffffffff
   34cd4:	mov    QWORD PTR [rsp+0x338],0xffffffffffffffff
   34ce0:	mov    QWORD PTR [rsp+0x358],0xffffffffffffffff
   34cec:	mov    QWORD PTR [rsp+0x378],0xffffffffffffffff
   34cf8:	mov    QWORD PTR [rsp+0x398],0xffffffffffffffff
   34d04:	mov    QWORD PTR [rsp+0x3b8],0xffffffffffffffff
   34d10:	mov    QWORD PTR [rsp+0x3d8],0xffffffffffffffff
   34d1c:	mov    QWORD PTR [rsp+0x3f8],0xffffffffffffffff
   34d28:	mov    QWORD PTR [rsp+0x418],0xffffffffffffffff
   34d34:	mov    QWORD PTR [rsp+0x438],0xffffffffffffffff
   34d40:	mov    QWORD PTR [rsp+0x458],0xffffffffffffffff
   34d4c:	mov    QWORD PTR [rsp+0x478],0xffffffffffffffff
   34d58:	mov    QWORD PTR [rsp+0x498],0xffffffffffffffff
   34d64:	mov    QWORD PTR [rsp+0x4b8],0xffffffffffffffff
   34d70:	mov    QWORD PTR [rsp+0x4d8],0xffffffffffffffff
   34d7c:	mov    QWORD PTR [rsp+0x4f8],0xffffffffffffffff
   34d88:	mov    QWORD PTR [rsp+0x518],0xffffffffffffffff
   34d94:	mov    QWORD PTR [rsp+0x538],0xffffffffffffffff
   34da0:	mov    QWORD PTR [rsp+0x558],0xffffffffffffffff
   34dac:	mov    QWORD PTR [rsp+0x578],0xffffffffffffffff
   34db8:	mov    QWORD PTR [rsp+0x598],0xffffffffffffffff
   34dc4:	mov    QWORD PTR [rsp+0x5b8],0xffffffffffffffff
   34dd0:	mov    QWORD PTR [rsp+0x5d8],0xffffffffffffffff
   34ddc:	mov    QWORD PTR [rsp+0x5f8],0xffffffffffffffff
   34de8:	mov    QWORD PTR [rsp+0x618],0xffffffffffffffff
   34df4:	mov    QWORD PTR [rsp+0x638],0xffffffffffffffff
   34e00:	mov    QWORD PTR [rsp+0x658],0xffffffffffffffff
   34e0c:	mov    QWORD PTR [rsp+0x678],0xffffffffffffffff
   34e18:	mov    rax,QWORD PTR [r12+0x38]
   34e1d:	mov    rdi,QWORD PTR [r12+0x40]
   34e22:	mov    rcx,QWORD PTR [r12+0x8]
   34e27:	mov    QWORD PTR [rsp+0x20],rcx
   34e2c:	mov    rcx,QWORD PTR [r12+0x10]
   34e31:	mov    QWORD PTR [rsp+0x10],rcx
   34e36:	add    rax,0x8
   34e3a:	xor    r14d,r14d
   34e3d:	xor    ecx,ecx
   34e3f:	mov    r9,QWORD PTR [rsp+0x8]
   34e44:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   34e50:	cmp    rdi,rcx
   34e53:	je     36125 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1945>
   34e59:	cmp    QWORD PTR [rsp+0x10],rcx
   34e5e:	je     36134 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1954>
   34e64:	mov    rdx,QWORD PTR [rax]
   34e67:	mov    rsi,QWORD PTR [rsp+0x20]
   34e6c:	mov    esi,DWORD PTR [rsi+rcx*4]
   34e6f:	mov    esi,DWORD PTR [r9+rsi*4]
   34e73:	mov    BYTE PTR [rdx],sil
   34e76:	shr    esi,0x8
   34e79:	and    esi,0x7fffff
   34e7f:	lea    r8,[rdx+0x1]
   34e83:	add    rdx,QWORD PTR [rax-0x8]
   34e87:	mov    QWORD PTR [rsp+r14*1+0x298],rcx
   34e8f:	mov    QWORD PTR [rsp+r14*1+0x2a0],r8
   34e97:	mov    QWORD PTR [rsp+r14*1+0x2a8],rdx
   34e9f:	mov    DWORD PTR [rsp+r14*1+0x2b0],esi
   34ea7:	prefetcht0 BYTE PTR [r9+rsi*4]
   34eac:	lea    rdx,[rcx+0x1]
   34eb0:	add    r14,0x20
   34eb4:	cmp    rcx,0x1e
   34eb8:	ja     34ec6 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x6e6>
   34eba:	add    rax,0x18
   34ebe:	mov    rcx,rdx
   34ec1:	cmp    rdx,r13
   34ec4:	jb     34e50 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x670>
   34ec6:	cmp    rdx,0x20
   34eca:	mov    QWORD PTR [rsp+0x78],r10
   34ecf:	jne    35969 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1189>
   34ed5:	mov    QWORD PTR [rsp+0x180],rdx
   34edd:	mov    QWORD PTR [rsp+0x70],r13
   34ee2:	xorps  xmm0,xmm0
   34ee5:	movups XMMWORD PTR [rsp+0x288],xmm0
   34eed:	movups XMMWORD PTR [rsp+0x278],xmm0
   34ef5:	movups XMMWORD PTR [rsp+0x268],xmm0
   34efd:	movups XMMWORD PTR [rsp+0x258],xmm0
   34f05:	movups XMMWORD PTR [rsp+0x248],xmm0
   34f0d:	movups XMMWORD PTR [rsp+0x238],xmm0
   34f15:	movups XMMWORD PTR [rsp+0x228],xmm0
   34f1d:	movups XMMWORD PTR [rsp+0x218],xmm0
   34f25:	mov    eax,DWORD PTR [rsp+0x2b0]
   34f2c:	mov    ecx,DWORD PTR [rsp+0x2d0]
   34f33:	mov    DWORD PTR [rsp+0x198],eax
   34f3a:	mov    DWORD PTR [rsp+0x19c],ecx
   34f41:	mov    eax,DWORD PTR [rsp+0x2f0]
   34f48:	mov    DWORD PTR [rsp+0x1a0],eax
   34f4f:	mov    eax,DWORD PTR [rsp+0x310]
   34f56:	mov    DWORD PTR [rsp+0x1a4],eax
   34f5d:	mov    eax,DWORD PTR [rsp+0x330]
   34f64:	mov    DWORD PTR [rsp+0x1a8],eax
   34f6b:	mov    eax,DWORD PTR [rsp+0x350]
   34f72:	mov    ecx,DWORD PTR [rsp+0x370]
   34f79:	mov    DWORD PTR [rsp+0x1ac],eax
   34f80:	mov    DWORD PTR [rsp+0x1b0],ecx
   34f87:	mov    eax,DWORD PTR [rsp+0x390]
   34f8e:	mov    DWORD PTR [rsp+0x1b4],eax
   34f95:	mov    eax,DWORD PTR [rsp+0x3b0]
   34f9c:	mov    DWORD PTR [rsp+0x1b8],eax
   34fa3:	mov    eax,DWORD PTR [rsp+0x3d0]
   34faa:	mov    DWORD PTR [rsp+0x1bc],eax
   34fb1:	mov    eax,DWORD PTR [rsp+0x3f0]
   34fb8:	mov    DWORD PTR [rsp+0x1c0],eax
   34fbf:	mov    eax,DWORD PTR [rsp+0x410]
   34fc6:	mov    DWORD PTR [rsp+0x1c4],eax
   34fcd:	mov    eax,DWORD PTR [rsp+0x430]
   34fd4:	mov    DWORD PTR [rsp+0x1c8],eax
   34fdb:	mov    eax,DWORD PTR [rsp+0x450]
   34fe2:	mov    DWORD PTR [rsp+0x1cc],eax
   34fe9:	mov    eax,DWORD PTR [rsp+0x470]
   34ff0:	mov    DWORD PTR [rsp+0x1d0],eax
   34ff7:	mov    eax,DWORD PTR [rsp+0x490]
   34ffe:	mov    DWORD PTR [rsp+0x1d4],eax
   35005:	mov    eax,DWORD PTR [rsp+0x4b0]
   3500c:	mov    DWORD PTR [rsp+0x1d8],eax
   35013:	mov    eax,DWORD PTR [rsp+0x4d0]
   3501a:	mov    DWORD PTR [rsp+0x1dc],eax
   35021:	mov    eax,DWORD PTR [rsp+0x4f0]
   35028:	mov    DWORD PTR [rsp+0x1e0],eax
   3502f:	mov    eax,DWORD PTR [rsp+0x510]
   35036:	mov    DWORD PTR [rsp+0x1e4],eax
   3503d:	mov    eax,DWORD PTR [rsp+0x530]
   35044:	mov    DWORD PTR [rsp+0x1e8],eax
   3504b:	mov    eax,DWORD PTR [rsp+0x550]
   35052:	mov    DWORD PTR [rsp+0x1ec],eax
   35059:	mov    eax,DWORD PTR [rsp+0x570]
   35060:	mov    DWORD PTR [rsp+0x1f0],eax
   35067:	mov    eax,DWORD PTR [rsp+0x590]
   3506e:	mov    DWORD PTR [rsp+0x1f4],eax
   35075:	mov    eax,DWORD PTR [rsp+0x5b0]
   3507c:	mov    DWORD PTR [rsp+0x1f8],eax
   35083:	mov    eax,DWORD PTR [rsp+0x5d0]
   3508a:	mov    DWORD PTR [rsp+0x1fc],eax
   35091:	mov    eax,DWORD PTR [rsp+0x5f0]
   35098:	mov    DWORD PTR [rsp+0x200],eax
   3509f:	mov    eax,DWORD PTR [rsp+0x610]
   350a6:	mov    DWORD PTR [rsp+0x204],eax
   350ad:	mov    eax,DWORD PTR [rsp+0x630]
   350b4:	mov    DWORD PTR [rsp+0x208],eax
   350bb:	mov    eax,DWORD PTR [rsp+0x650]
   350c2:	mov    DWORD PTR [rsp+0x20c],eax
   350c9:	mov    eax,DWORD PTR [rsp+0x670]
   350d0:	mov    DWORD PTR [rsp+0x210],eax
   350d7:	mov    eax,DWORD PTR [rsp+0x690]
   350de:	mov    DWORD PTR [rsp+0x214],eax
   350e5:	add    r15,0x3f
   350e9:	mov    r8,QWORD PTR [r12+0x38]
   350ee:	mov    rsi,QWORD PTR [r12+0x40]
   350f3:	mov    r13d,0x20
   350f9:	mov    QWORD PTR [rsp+0x68],0x0
   35102:	mov    r12,r15
   35105:	mov    QWORD PTR [rsp+0x60],rsi
   3510a:	mov    QWORD PTR [rsp+0x58],r8
   3510f:	mov    QWORD PTR [rsp+0x50],r15
   35114:	xor    r15d,r15d
   35117:	jmp    35151 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x971>
   35119:	nop    DWORD PTR [rax+0x0]
   35120:	add    rbx,rax
   35123:	mov    QWORD PTR [rsp+r15*1+0x2a0],rbx
   3512b:	add    rdx,rax
   3512e:	mov    QWORD PTR [rsp+r15*1+0x2a8],rdx
   35136:	sub    rdx,rbx
   35139:	add    r15,0x20
   3513d:	cmp    rdx,r12
   35140:	cmovb  r12,rdx
   35144:	cmp    r15,0x400
   3514b:	je     351d0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x9f0>
   35151:	mov    rbx,QWORD PTR [rsp+r15*1+0x2a0]
   35159:	mov    rdx,QWORD PTR [rsp+r15*1+0x2a8]
   35161:	cmp    rbx,rdx
   35164:	jne    35136 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x956>
   35166:	mov    rdi,QWORD PTR [rsp+r15*1+0x298]
   3516e:	cmp    rdi,rsi
   35171:	jae    36200 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a20>
   35177:	lea    rcx,[rdi+rdi*2]
   3517b:	mov    rdx,QWORD PTR [r8+rcx*8]
   3517f:	mov    rax,QWORD PTR [r8+rcx*8+0x8]
   35184:	sub    rbx,rax
   35187:	mov    QWORD PTR [r8+rcx*8+0x10],rbx
   3518c:	mov    rdi,rdx
   3518f:	sub    rdi,rbx
   35192:	cmp    rdx,rdi
   35195:	jbe    35120 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   35197:	lea    rbp,[r8+rcx*8]
   3519b:	mov    ecx,0x1
   351a0:	mov    r8d,0x1
   351a6:	mov    rdi,rbp
   351a9:	mov    rsi,rbx
   351ac:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   351b1:	mov    r8,QWORD PTR [rsp+0x58]
   351b6:	mov    rsi,QWORD PTR [rsp+0x60]
   351bb:	mov    rdx,QWORD PTR [rbp+0x0]
   351bf:	mov    rax,QWORD PTR [rbp+0x8]
   351c3:	jmp    35120 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x940>
   351c8:	nop    DWORD PTR [rax+rax*1+0x0]
   351d0:	mov    rax,QWORD PTR [rsp+0x2a0]
   351d8:	mov    QWORD PTR [rsp+0x80],rax
   351e0:	mov    rax,QWORD PTR [rsp+0x2c0]
   351e8:	mov    QWORD PTR [rsp+0x88],rax
   351f0:	mov    rax,QWORD PTR [rsp+0x2e0]
   351f8:	mov    QWORD PTR [rsp+0x90],rax
   35200:	mov    rax,QWORD PTR [rsp+0x300]
   35208:	mov    QWORD PTR [rsp+0x98],rax
   35210:	mov    rax,QWORD PTR [rsp+0x320]
   35218:	mov    QWORD PTR [rsp+0xa0],rax
   35220:	mov    rax,QWORD PTR [rsp+0x340]
   35228:	mov    QWORD PTR [rsp+0xa8],rax
   35230:	mov    rax,QWORD PTR [rsp+0x360]
   35238:	mov    QWORD PTR [rsp+0xb0],rax
   35240:	mov    rax,QWORD PTR [rsp+0x380]
   35248:	mov    QWORD PTR [rsp+0xb8],rax
   35250:	mov    rax,QWORD PTR [rsp+0x3a0]
   35258:	mov    QWORD PTR [rsp+0xc0],rax
   35260:	mov    rax,QWORD PTR [rsp+0x3c0]
   35268:	mov    QWORD PTR [rsp+0xc8],rax
   35270:	mov    rax,QWORD PTR [rsp+0x3e0]
   35278:	mov    QWORD PTR [rsp+0xd0],rax
   35280:	mov    rax,QWORD PTR [rsp+0x400]
   35288:	mov    QWORD PTR [rsp+0xd8],rax
   35290:	mov    rax,QWORD PTR [rsp+0x420]
   35298:	mov    QWORD PTR [rsp+0xe0],rax
   352a0:	mov    rax,QWORD PTR [rsp+0x440]
   352a8:	mov    QWORD PTR [rsp+0xe8],rax
   352b0:	mov    rax,QWORD PTR [rsp+0x460]
   352b8:	mov    QWORD PTR [rsp+0xf0],rax
   352c0:	mov    rax,QWORD PTR [rsp+0x480]
   352c8:	mov    QWORD PTR [rsp+0xf8],rax
   352d0:	mov    rax,QWORD PTR [rsp+0x4a0]
   352d8:	mov    QWORD PTR [rsp+0x100],rax
   352e0:	mov    rax,QWORD PTR [rsp+0x4c0]
   352e8:	mov    QWORD PTR [rsp+0x108],rax
   352f0:	mov    rax,QWORD PTR [rsp+0x4e0]
   352f8:	mov    QWORD PTR [rsp+0x110],rax
   35300:	mov    rax,QWORD PTR [rsp+0x500]
   35308:	mov    QWORD PTR [rsp+0x118],rax
   35310:	mov    rax,QWORD PTR [rsp+0x520]
   35318:	mov    QWORD PTR [rsp+0x120],rax
   35320:	mov    rax,QWORD PTR [rsp+0x540]
   35328:	mov    QWORD PTR [rsp+0x128],rax
   35330:	mov    rax,QWORD PTR [rsp+0x560]
   35338:	mov    QWORD PTR [rsp+0x130],rax
   35340:	mov    rax,QWORD PTR [rsp+0x580]
   35348:	mov    QWORD PTR [rsp+0x138],rax
   35350:	mov    rax,QWORD PTR [rsp+0x5a0]
   35358:	mov    QWORD PTR [rsp+0x140],rax
   35360:	mov    rax,QWORD PTR [rsp+0x5c0]
   35368:	mov    QWORD PTR [rsp+0x148],rax
   35370:	mov    rax,QWORD PTR [rsp+0x5e0]
   35378:	mov    QWORD PTR [rsp+0x150],rax
   35380:	mov    rax,QWORD PTR [rsp+0x600]
   35388:	mov    QWORD PTR [rsp+0x158],rax
   35390:	mov    rax,QWORD PTR [rsp+0x620]
   35398:	mov    QWORD PTR [rsp+0x160],rax
   353a0:	mov    rax,QWORD PTR [rsp+0x640]
   353a8:	mov    QWORD PTR [rsp+0x168],rax
   353b0:	mov    rax,QWORD PTR [rsp+0x660]
   353b8:	mov    QWORD PTR [rsp+0x170],rax
   353c0:	mov    rax,QWORD PTR [rsp+0x680]
   353c8:	mov    QWORD PTR [rsp+0x178],rax
   353d0:	test   r12,r12
   353d3:	je     356e0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf00>
   353d9:	mov    rax,QWORD PTR [rsp+0x18]
   353de:	mov    rcx,QWORD PTR [rax+0x38]
   353e2:	mov    QWORD PTR [rsp+0x48],rcx
   353e7:	mov    rcx,QWORD PTR [rax+0x40]
   353eb:	mov    QWORD PTR [rsp],rcx
   353ef:	mov    rcx,QWORD PTR [rax+0x50]
   353f3:	mov    QWORD PTR [rsp+0x28],rcx
   353f8:	mov    rcx,QWORD PTR [rax+0x58]
   353fc:	mov    QWORD PTR [rsp+0x30],rcx
   35401:	mov    rcx,QWORD PTR [rax+0x8]
   35405:	mov    rax,QWORD PTR [rax+0x10]
   35409:	xor    edx,edx
   3540b:	mov    QWORD PTR [rsp+0x20],rcx
   35410:	mov    QWORD PTR [rsp+0x10],rax
   35415:	mov    rax,QWORD PTR [rsp+0x68]
   3541a:	shl    rax,0x7
   3541e:	mov    QWORD PTR [rsp+0x188],rax
   35426:	lea    r9,[rsp+rax*1]
   3542a:	add    r9,0x198
   35431:	mov    QWORD PTR [rsp+0x40],r9
   35436:	jmp    35451 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc71>
   35438:	nop    DWORD PTR [rax+rax*1+0x0]
   35440:	mov    rdx,QWORD PTR [rsp+0x190]
   35448:	cmp    rdx,r12
   3544b:	jae    356c7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xee7>
   35451:	mov    rbx,rdx
   35454:	cmp    QWORD PTR [rsp+0x68],0x0
   3545a:	je     35490 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcb0>
   3545c:	mov    rdi,QWORD PTR [rsp+0x8]
   35461:	lea    rsi,[rsp+0x198]
   35469:	lea    rdx,[rsp+0x80]
   35471:	mov    rcx,rbx
   35474:	call   34340 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt11spec_offsetKj20_Kh1_Kj1_EB4_>
   35479:	lea    rdx,[rbx+0x1]
   3547d:	test   eax,eax
   3547f:	mov    r9,QWORD PTR [rsp+0x40]
   35484:	js     354be <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcde>
   35486:	jmp    356ac <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xecc>
   3548b:	nop    DWORD PTR [rax+rax*1+0x0]
   35490:	mov    rdi,QWORD PTR [rsp+0x8]
   35495:	lea    rsi,[rsp+0x198]
   3549d:	lea    rdx,[rsp+0x80]
   354a5:	mov    rcx,rbx
   354a8:	call   33ea0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt11spec_offsetKj20_Kh1_Kj0_EB4_>
   354ad:	lea    rdx,[rbx+0x1]
   354b1:	test   eax,eax
   354b3:	mov    r9,QWORD PTR [rsp+0x40]
   354b8:	jns    356ac <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xecc>
   354be:	xor    eax,eax
   354c0:	xor    ecx,ecx
   354c2:	jmp    354dd <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcfd>
   354c4:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   354d0:	bts    rcx,rax
   354d4:	inc    rax
   354d7:	cmp    rax,0x20
   354db:	je     3550f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd2f>
   354dd:	mov    esi,DWORD PTR [r9+rax*4]
   354e1:	mov    r8,QWORD PTR [rsp+0x8]
   354e6:	mov    esi,DWORD PTR [r8+rsi*4]
   354ea:	test   esi,esi
   354ec:	js     354d0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcf0>
   354ee:	mov    rdi,QWORD PTR [rsp+rax*8+0x80]
   354f6:	mov    BYTE PTR [rdi+rbx*1],sil
   354fa:	shr    esi,0x8
   354fd:	mov    DWORD PTR [r9+rax*4],esi
   35501:	prefetcht0 BYTE PTR [r8+rsi*4]
   35506:	inc    rax
   35509:	cmp    rax,0x20
   3550d:	jne    354dd <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xcfd>
   3550f:	mov    QWORD PTR [rsp+0x190],rdx
   35517:	test   rcx,rcx
   3551a:	je     35440 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc60>
   35520:	mov    rsi,rbx
   35523:	not    rsi
   35526:	cs nop WORD PTR [rax+rax*1+0x0]
   35530:	tzcnt  rdi,rcx
   35535:	cmp    rdi,0x1f
   35539:	ja     361b4 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19d4>
   3553f:	mov    r8d,edi
   35542:	shl    r8d,0x5
   35546:	mov    r9d,DWORD PTR [r9+rdi*4]
   3554a:	mov    DWORD PTR [rsp+r8*1+0x2b0],r9d
   35552:	mov    r10,QWORD PTR [rsp+rdi*8+0x80]
   3555a:	add    r10,rbx
   3555d:	mov    QWORD PTR [rsp+r8*1+0x2a0],r10
   35565:	mov    rax,QWORD PTR [rsp+r8*1+0x298]
   3556d:	cmp    rax,QWORD PTR [rsp]
   35571:	mov    rdx,QWORD PTR [rsp+0x20]
   35576:	jae    361a1 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19c1>
   3557c:	lea    r11,[rax+rax*2]
   35580:	mov    r15,QWORD PTR [rsp+0x48]
   35585:	sub    r10,QWORD PTR [r15+r11*8+0x8]
   3558a:	mov    QWORD PTR [r15+r11*8+0x10],r10
   3558f:	mov    r11,QWORD PTR [rsp+0x10]
   35594:	test   r11,r11
   35597:	je     355f0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe10>
   35599:	xor    r10d,r10d
   3559c:	cmp    r11,0x1
   355a0:	je     355cd <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xded>
   355a2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   355b0:	mov    r15,r10
   355b3:	mov    rbp,r11
   355b6:	shr    rbp,1
   355b9:	add    r10,rbp
   355bc:	cmp    DWORD PTR [rdx+r10*4],r9d
   355c0:	cmova  r10,r15
   355c4:	sub    r11,rbp
   355c7:	cmp    r11,0x1
   355cb:	ja     355b0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xdd0>
   355cd:	cmp    DWORD PTR [rdx+r10*4],r9d
   355d1:	mov    r9,0xffffffffffffffff
   355d8:	cmovne r10,r9
   355dc:	cmp    rax,QWORD PTR [rsp+0x30]
   355e1:	jb     35602 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe22>
   355e3:	jmp    3618d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19ad>
   355e8:	nop    DWORD PTR [rax+rax*1+0x0]
   355f0:	mov    r10,0xffffffffffffffff
   355f7:	cmp    rax,QWORD PTR [rsp+0x30]
   355fc:	jae    3618d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19ad>
   35602:	lea    r9,[rcx-0x1]
   35606:	and    rcx,r9
   35609:	mov    r9,QWORD PTR [rsp+0x28]
   3560e:	mov    QWORD PTR [r9+rax*8],r10
   35612:	cmp    r13,QWORD PTR [rsp+0x70]
   35617:	je     35982 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11a2>
   3561d:	cmp    r13,QWORD PTR [rsp]
   35621:	jae    361c5 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19e5>
   35627:	cmp    r13,QWORD PTR [rsp+0x10]
   3562c:	jae    361d8 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x19f8>
   35632:	lea    rax,[rsp+r8*1]
   35636:	add    rax,0x298
   3563c:	lea    r8,[r13+r13*2+0x0]
   35641:	mov    rbp,QWORD PTR [rsp+0x48]
   35646:	mov    r9,QWORD PTR [rbp+r8*8+0x8]
   3564b:	mov    r10d,DWORD PTR [rdx+r13*4]
   3564f:	mov    rdx,QWORD PTR [rsp+0x8]
   35654:	mov    r10d,DWORD PTR [rdx+r10*4]
   35658:	mov    BYTE PTR [r9],r10b
   3565b:	shr    r10d,0x8
   3565f:	and    r10d,0x7fffff
   35666:	lea    r11,[r9+0x1]
   3566a:	lea    r15,[r9+rsi*1]
   3566e:	inc    r15
   35671:	add    r9,QWORD PTR [rbp+r8*8+0x0]
   35676:	mov    QWORD PTR [rax],r13
   35679:	mov    QWORD PTR [rax+0x8],r11
   3567d:	mov    QWORD PTR [rax+0x10],r9
   35681:	mov    DWORD PTR [rax+0x18],r10d
   35685:	mov    r9,QWORD PTR [rsp+0x40]
   3568a:	mov    DWORD PTR [r9+rdi*4],r10d
   3568e:	prefetcht0 BYTE PTR [rdx+r10*4]
   35693:	inc    r13
   35696:	mov    QWORD PTR [rsp+rdi*8+0x80],r15
   3569e:	test   rcx,rcx
   356a1:	jne    35530 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xd50>
   356a7:	jmp    35440 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc60>
   356ac:	xor    QWORD PTR [rsp+0x68],0x1
   356b2:	cmp    rdx,r12
   356b5:	mov    rsi,QWORD PTR [rsp+0x60]
   356ba:	mov    r8,QWORD PTR [rsp+0x58]
   356bf:	jb     35415 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xc35>
   356c5:	jmp    356e0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xf00>
   356c7:	mov    rsi,QWORD PTR [rsp+0x60]
   356cc:	mov    r8,QWORD PTR [rsp+0x58]
   356d1:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   356e0:	mov    rax,QWORD PTR [rsp+0x80]
   356e8:	add    rax,r12
   356eb:	mov    QWORD PTR [rsp+0x2a0],rax
   356f3:	mov    rax,QWORD PTR [rsp+0x88]
   356fb:	add    rax,r12
   356fe:	mov    QWORD PTR [rsp+0x2c0],rax
   35706:	mov    rax,QWORD PTR [rsp+0x90]
   3570e:	add    rax,r12
   35711:	mov    QWORD PTR [rsp+0x2e0],rax
   35719:	mov    rax,QWORD PTR [rsp+0x98]
   35721:	add    rax,r12
   35724:	mov    QWORD PTR [rsp+0x300],rax
   3572c:	mov    rax,QWORD PTR [rsp+0xa0]
   35734:	add    rax,r12
   35737:	mov    QWORD PTR [rsp+0x320],rax
   3573f:	mov    rax,QWORD PTR [rsp+0xa8]
   35747:	add    rax,r12
   3574a:	mov    QWORD PTR [rsp+0x340],rax
   35752:	mov    rax,QWORD PTR [rsp+0xb0]
   3575a:	add    rax,r12
   3575d:	mov    QWORD PTR [rsp+0x360],rax
   35765:	mov    rax,QWORD PTR [rsp+0xb8]
   3576d:	add    rax,r12
   35770:	mov    QWORD PTR [rsp+0x380],rax
   35778:	mov    rax,QWORD PTR [rsp+0xc0]
   35780:	add    rax,r12
   35783:	mov    QWORD PTR [rsp+0x3a0],rax
   3578b:	mov    rax,QWORD PTR [rsp+0xc8]
   35793:	add    rax,r12
   35796:	mov    QWORD PTR [rsp+0x3c0],rax
   3579e:	mov    rax,QWORD PTR [rsp+0xd0]
   357a6:	add    rax,r12
   357a9:	mov    QWORD PTR [rsp+0x3e0],rax
   357b1:	mov    rax,QWORD PTR [rsp+0xd8]
   357b9:	add    rax,r12
   357bc:	mov    QWORD PTR [rsp+0x400],rax
   357c4:	mov    rax,QWORD PTR [rsp+0xe0]
   357cc:	add    rax,r12
   357cf:	mov    QWORD PTR [rsp+0x420],rax
   357d7:	mov    rax,QWORD PTR [rsp+0xe8]
   357df:	add    rax,r12
   357e2:	mov    QWORD PTR [rsp+0x440],rax
   357ea:	mov    rax,QWORD PTR [rsp+0xf0]
   357f2:	add    rax,r12
   357f5:	mov    QWORD PTR [rsp+0x460],rax
   357fd:	mov    rax,QWORD PTR [rsp+0xf8]
   35805:	add    rax,r12
   35808:	mov    QWORD PTR [rsp+0x480],rax
   35810:	mov    rax,QWORD PTR [rsp+0x100]
   35818:	add    rax,r12
   3581b:	mov    QWORD PTR [rsp+0x4a0],rax
   35823:	mov    rax,QWORD PTR [rsp+0x108]
   3582b:	add    rax,r12
   3582e:	mov    QWORD PTR [rsp+0x4c0],rax
   35836:	mov    rax,QWORD PTR [rsp+0x110]
   3583e:	add    rax,r12
   35841:	mov    QWORD PTR [rsp+0x4e0],rax
   35849:	mov    rax,QWORD PTR [rsp+0x118]
   35851:	add    rax,r12
   35854:	mov    QWORD PTR [rsp+0x500],rax
   3585c:	mov    rax,QWORD PTR [rsp+0x120]
   35864:	add    rax,r12
   35867:	mov    QWORD PTR [rsp+0x520],rax
   3586f:	mov    rax,QWORD PTR [rsp+0x128]
   35877:	add    rax,r12
   3587a:	mov    QWORD PTR [rsp+0x540],rax
   35882:	mov    rax,QWORD PTR [rsp+0x130]
   3588a:	add    rax,r12
   3588d:	mov    QWORD PTR [rsp+0x560],rax
   35895:	mov    rax,QWORD PTR [rsp+0x138]
   3589d:	add    rax,r12
   358a0:	mov    QWORD PTR [rsp+0x580],rax
   358a8:	mov    rax,QWORD PTR [rsp+0x140]
   358b0:	add    rax,r12
   358b3:	mov    QWORD PTR [rsp+0x5a0],rax
   358bb:	mov    rax,QWORD PTR [rsp+0x148]
   358c3:	add    rax,r12
   358c6:	mov    QWORD PTR [rsp+0x5c0],rax
   358ce:	mov    rax,QWORD PTR [rsp+0x150]
   358d6:	add    rax,r12
   358d9:	mov    QWORD PTR [rsp+0x5e0],rax
   358e1:	mov    rax,QWORD PTR [rsp+0x158]
   358e9:	add    rax,r12
   358ec:	mov    QWORD PTR [rsp+0x600],rax
   358f4:	mov    rax,QWORD PTR [rsp+0x160]
   358fc:	add    rax,r12
   358ff:	mov    QWORD PTR [rsp+0x620],rax
   35907:	mov    rax,QWORD PTR [rsp+0x168]
   3590f:	add    rax,r12
   35912:	mov    QWORD PTR [rsp+0x640],rax
   3591a:	mov    rax,QWORD PTR [rsp+0x170]
   35922:	add    rax,r12
   35925:	mov    QWORD PTR [rsp+0x660],rax
   3592d:	add    r12,QWORD PTR [rsp+0x178]
   35935:	mov    QWORD PTR [rsp+0x680],r12
   3593d:	mov    r12,QWORD PTR [rsp+0x50]
   35942:	jmp    35114 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x934>
   35947:	mov    rdx,rbx
   3594a:	mov    QWORD PTR [r12+0x40],rdx
   3594f:	cmp    r13,rdx
   35952:	jbe    34971 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191>
   35958:	lea    rcx,[rip+0x5e4e9]        # 93e48 <__frame_dummy_init_array_entry+0x920>
   3595f:	xor    edi,edi
   35961:	mov    rsi,r13
   35964:	call   2b2e0 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   35969:	mov    rax,QWORD PTR [r12+0x50]
   3596e:	mov    QWORD PTR [rsp+0x28],rax
   35973:	mov    rax,QWORD PTR [r12+0x58]
   35978:	mov    QWORD PTR [rsp+0x30],rax
   3597d:	jmp    35a40 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1260>
   35982:	not    rcx
   35985:	mov    rax,QWORD PTR [rsp+0x188]
   3598d:	add    rax,rsp
   35990:	add    rax,0x19c
   35996:	lea    rdx,[rdi-0x1]
   3599a:	xor    esi,esi
   3599c:	xor    r8d,r8d
   3599f:	mov    r10,QWORD PTR [rsp+0x78]
   359a4:	jmp    359c1 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11e1>
   359a6:	cs nop WORD PTR [rax+rax*1+0x0]
   359b0:	add    r8,0x40
   359b4:	add    rsi,0x2
   359b8:	cmp    r8,0x400
   359bf:	je     35a38 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1258>
   359c1:	mov    r9d,DWORD PTR [rax+rsi*4-0x4]
   359c6:	mov    DWORD PTR [rsp+r8*1+0x2b0],r9d
   359ce:	cmp    rdi,rsi
   359d1:	jne    359f0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1210>
   359d3:	mov    r9d,DWORD PTR [rax+rsi*4]
   359d7:	mov    DWORD PTR [rsp+r8*1+0x2d0],r9d
   359df:	cmp    rdx,rsi
   359e2:	je     359b0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11d0>
   359e4:	jmp    35a18 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1238>
   359e6:	cs nop WORD PTR [rax+rax*1+0x0]
   359f0:	bt     rcx,rsi
   359f4:	mov    r9,QWORD PTR [rsp+rsi*8+0x80]
   359fc:	adc    r9,rbx
   359ff:	mov    QWORD PTR [rsp+r8*1+0x2a0],r9
   35a07:	mov    r9d,DWORD PTR [rax+rsi*4]
   35a0b:	mov    DWORD PTR [rsp+r8*1+0x2d0],r9d
   35a13:	cmp    rdx,rsi
   35a16:	je     359b0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11d0>
   35a18:	lea    r9,[rsi+0x1]
   35a1c:	bt     rcx,r9
   35a20:	mov    r9,QWORD PTR [rsp+rsi*8+0x88]
   35a28:	adc    r9,rbx
   35a2b:	mov    QWORD PTR [rsp+r8*1+0x2c0],r9
   35a33:	jmp    359b0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x11d0>
   35a38:	mov    rdx,QWORD PTR [rsp+0x180]
   35a40:	shl    rdx,0x5
   35a44:	lea    r15,[rsp+rdx*1]
   35a48:	add    r15,0x298
   35a4f:	lea    r12,[rsp+0x298]
   35a57:	mov    rcx,QWORD PTR [rsp+0x10]
   35a5c:	mov    rax,QWORD PTR [rsp+0x28]
   35a61:	jmp    35a7d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x129d>
   35a63:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35a70:	mov    r12,r13
   35a73:	add    r14,0xffffffffffffffe0
   35a77:	je     35c09 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1429>
   35a7d:	lea    r13,[r12+0x20]
   35a82:	mov    rbx,QWORD PTR [r12]
   35a86:	cmp    rbx,0xffffffffffffffff
   35a8a:	je     35a70 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1290>
   35a8c:	cmp    rbx,QWORD PTR [rsp+0x30]
   35a91:	jae    361ec <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a0c>
   35a97:	cmp    QWORD PTR [rax+rbx*8],0xffffffffffffffff
   35a9c:	jne    35a70 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1290>
   35a9e:	mov    edi,0x80
   35aa3:	call   QWORD PTR [rip+0x607af]        # 96258 <malloc@GLIBC_2.2.5>
   35aa9:	test   rax,rax
   35aac:	je     3622c <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a4c>
   35ab2:	mov    rcx,QWORD PTR [r12+0x18]
   35ab7:	mov    QWORD PTR [rax+0x18],rcx
   35abb:	movups xmm0,XMMWORD PTR [r12+0x8]
   35ac1:	movups XMMWORD PTR [rax+0x8],xmm0
   35ac5:	mov    QWORD PTR [rax],rbx
   35ac8:	mov    QWORD PTR [rsp+0x198],0x4
   35ad4:	mov    QWORD PTR [rsp+0x1a0],rax
   35adc:	mov    QWORD PTR [rsp+0x1a8],0x1
   35ae8:	mov    r12d,0x1
   35aee:	cmp    r13,r15
   35af1:	je     35bc7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13e7>
   35af7:	lea    rbx,[rsp+0x198]
   35aff:	mov    rdx,QWORD PTR [rsp+0x18]
   35b04:	mov    rcx,QWORD PTR [rdx+0x50]
   35b08:	mov    rsi,QWORD PTR [rdx+0x58]
   35b0c:	mov    rdx,r13
   35b0f:	jmp    35b2c <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x134c>
   35b11:	data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35b20:	mov    rdx,r13
   35b23:	cmp    r13,r15
   35b26:	je     35bc7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13e7>
   35b2c:	lea    r13,[rdx+0x20]
   35b30:	mov    r14,QWORD PTR [rdx]
   35b33:	cmp    r14,0xffffffffffffffff
   35b37:	je     35b20 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1340>
   35b39:	cmp    r14,rsi
   35b3c:	jae    3620c <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a2c>
   35b42:	cmp    QWORD PTR [rcx+r14*8],0xffffffffffffffff
   35b47:	jne    35b20 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1340>
   35b49:	mov    rcx,QWORD PTR [rdx+0x18]
   35b4d:	mov    QWORD PTR [rsp+0x90],rcx
   35b55:	movups xmm0,XMMWORD PTR [rdx+0x8]
   35b59:	movaps XMMWORD PTR [rsp+0x80],xmm0
   35b61:	cmp    r12,QWORD PTR [rsp+0x198]
   35b69:	jne    35b8e <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x13ae>
   35b6b:	mov    edx,0x1
   35b70:	mov    ecx,0x8
   35b75:	mov    r8d,0x20
   35b7b:	mov    rdi,rbx
   35b7e:	mov    rsi,r12
   35b81:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   35b86:	mov    rax,QWORD PTR [rsp+0x1a0]
   35b8e:	mov    rcx,r12
   35b91:	shl    rcx,0x5
   35b95:	mov    QWORD PTR [rax+rcx*1],r14
   35b99:	movaps xmm0,XMMWORD PTR [rsp+0x80]
   35ba1:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   35ba6:	mov    rdx,QWORD PTR [rsp+0x90]
   35bae:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   35bb3:	inc    r12
   35bb6:	mov    QWORD PTR [rsp+0x1a8],r12
   35bbe:	cmp    r13,r15
   35bc1:	jne    35aff <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x131f>
   35bc7:	mov    rax,QWORD PTR [rsp+0x198]
   35bcf:	mov    QWORD PTR [rsp+0x28],rax
   35bd4:	mov    rax,QWORD PTR [rsp+0x1a0]
   35bdc:	mov    QWORD PTR [rsp],rax
   35be0:	mov    r15,r12
   35be3:	shl    r15,0x5
   35be7:	mov    rax,QWORD PTR [rsp+0x18]
   35bec:	mov    r14,QWORD PTR [rax+0x38]
   35bf0:	mov    r13,QWORD PTR [rax+0x40]
   35bf4:	xor    ebp,ebp
   35bf6:	lea    rax,[rip+0x5e473]        # 94070 <__frame_dummy_init_array_entry+0xb48>
   35bfd:	mov    QWORD PTR [rsp+0x40],rax
   35c02:	mov    rdx,QWORD PTR [rsp+0x8]
   35c07:	jmp    35c82 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x14a2>
   35c09:	mov    eax,0x8
   35c0e:	mov    QWORD PTR [rsp],rax
   35c12:	xor    r14d,r14d
   35c15:	mov    rax,0xffffffffffffffff
   35c1c:	test   rcx,rcx
   35c1f:	mov    rsi,QWORD PTR [rsp+0x18]
   35c24:	jne    35ed0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x16f0>
   35c2a:	jmp    35f6f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x178f>
   35c2f:	mov    rdi,rbx
   35c32:	mov    r14,r10
   35c35:	call   363f0 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsg4oLtdKOaQv_5mibz2>
   35c3a:	mov    r10,r14
   35c3d:	mov    esi,DWORD PTR [rsp+0x3c]
   35c41:	mov    r14,QWORD PTR [rbx+0x8]
   35c45:	jmp    3489b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xbb>
   35c4a:	mov    rax,QWORD PTR [rsp+0x10]
   35c4f:	mov    rdx,QWORD PTR [rax]
   35c52:	mov    rax,QWORD PTR [rax+0x8]
   35c56:	cs nop WORD PTR [rax+rax*1+0x0]
   35c60:	add    rbx,rax
   35c63:	mov    rcx,QWORD PTR [rsp]
   35c67:	mov    QWORD PTR [rcx+rbp*1+0x8],rbx
   35c6c:	add    rdx,rax
   35c6f:	mov    QWORD PTR [rcx+rbp*1+0x10],rdx
   35c74:	mov    rdx,QWORD PTR [rsp+0x8]
   35c79:	add    rbp,0x20
   35c7d:	cmp    r15,rbp
   35c80:	je     35ce4 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1504>
   35c82:	mov    rax,QWORD PTR [rsp]
   35c86:	mov    rbx,QWORD PTR [rax+rbp*1+0x8]
   35c8b:	cmp    rbx,QWORD PTR [rax+rbp*1+0x10]
   35c90:	jne    35c79 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1499>
   35c92:	mov    rax,QWORD PTR [rsp]
   35c96:	mov    rdi,QWORD PTR [rax+rbp*1]
   35c9a:	cmp    rdi,r13
   35c9d:	jae    36162 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1982>
   35ca3:	lea    rcx,[rdi+rdi*2]
   35ca7:	mov    rdx,QWORD PTR [r14+rcx*8]
   35cab:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   35cb0:	sub    rbx,rax
   35cb3:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   35cb8:	mov    rsi,rdx
   35cbb:	sub    rsi,rbx
   35cbe:	cmp    rdx,rsi
   35cc1:	jbe    35c60 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1480>
   35cc3:	lea    rdi,[r14+rcx*8]
   35cc7:	mov    ecx,0x1
   35ccc:	mov    r8d,0x1
   35cd2:	mov    QWORD PTR [rsp+0x10],rdi
   35cd7:	mov    rsi,rbx
   35cda:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   35cdf:	jmp    35c4a <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x146a>
   35ce4:	mov    rax,QWORD PTR [rsp+0x18]
   35ce9:	mov    rcx,QWORD PTR [rax+0x8]
   35ced:	mov    QWORD PTR [rsp+0x20],rcx
   35cf2:	mov    rcx,QWORD PTR [rax+0x10]
   35cf6:	mov    QWORD PTR [rsp+0x10],rcx
   35cfb:	mov    rcx,QWORD PTR [rax+0x50]
   35cff:	mov    QWORD PTR [rsp+0x30],rcx
   35d04:	mov    rax,QWORD PTR [rax+0x58]
   35d08:	mov    QWORD PTR [rsp+0x48],rax
   35d0d:	xor    r15d,r15d
   35d10:	jmp    35d23 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1543>
   35d12:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35d20:	xor    r15d,r15d
   35d23:	mov    rbp,r15
   35d26:	shl    rbp,0x5
   35d2a:	add    rbp,QWORD PTR [rsp]
   35d2e:	jmp    35d5f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x157f>
   35d30:	mov    rax,QWORD PTR [rsp+0x70]
   35d35:	mov    rdx,QWORD PTR [rax]
   35d38:	mov    rax,QWORD PTR [rax+0x8]
   35d3c:	nop    DWORD PTR [rax+0x0]
   35d40:	add    rbx,rax
   35d43:	mov    QWORD PTR [rbp+0x8],rbx
   35d47:	add    rdx,rax
   35d4a:	mov    QWORD PTR [rbp+0x10],rdx
   35d4e:	mov    rdx,QWORD PTR [rsp+0x8]
   35d53:	inc    r15
   35d56:	add    rbp,0x20
   35d5a:	cmp    r15,r12
   35d5d:	jae    35d20 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1540>
   35d5f:	mov    eax,DWORD PTR [rbp+0x18]
   35d62:	mov    ecx,DWORD PTR [rdx+rax*4]
   35d65:	test   ecx,ecx
   35d67:	js     35de0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1600>
   35d69:	mov    eax,ecx
   35d6b:	shr    eax,0x8
   35d6e:	mov    DWORD PTR [rbp+0x18],eax
   35d71:	prefetcht0 BYTE PTR [rdx+rax*4]
   35d75:	mov    rax,QWORD PTR [rbp+0x8]
   35d79:	mov    BYTE PTR [rax],cl
   35d7b:	mov    rbx,QWORD PTR [rbp+0x8]
   35d7f:	inc    rbx
   35d82:	mov    QWORD PTR [rbp+0x8],rbx
   35d86:	cmp    rbx,QWORD PTR [rbp+0x10]
   35d8a:	jne    35d53 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1573>
   35d8c:	mov    rdi,QWORD PTR [rbp+0x0]
   35d90:	cmp    rdi,r13
   35d93:	jae    36162 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1982>
   35d99:	lea    rcx,[rdi+rdi*2]
   35d9d:	mov    rdx,QWORD PTR [r14+rcx*8]
   35da1:	mov    rax,QWORD PTR [r14+rcx*8+0x8]
   35da6:	sub    rbx,rax
   35da9:	mov    QWORD PTR [r14+rcx*8+0x10],rbx
   35dae:	mov    rsi,rdx
   35db1:	sub    rsi,rbx
   35db4:	cmp    rdx,rsi
   35db7:	jbe    35d40 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1560>
   35db9:	lea    rdi,[r14+rcx*8]
   35dbd:	mov    ecx,0x1
   35dc2:	mov    r8d,0x1
   35dc8:	mov    QWORD PTR [rsp+0x70],rdi
   35dcd:	mov    rsi,rbx
   35dd0:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   35dd5:	jmp    35d30 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1550>
   35dda:	nop    WORD PTR [rax+rax*1+0x0]
   35de0:	mov    rdi,QWORD PTR [rbp+0x0]
   35de4:	cmp    rdi,r13
   35de7:	jae    36148 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1968>
   35ded:	lea    rcx,[rdi+rdi*2]
   35df1:	mov    rdx,QWORD PTR [rbp+0x8]
   35df5:	sub    rdx,QWORD PTR [r14+rcx*8+0x8]
   35dfa:	mov    QWORD PTR [r14+rcx*8+0x10],rdx
   35dff:	mov    rdx,QWORD PTR [rsp+0x10]
   35e04:	test   rdx,rdx
   35e07:	je     35e18 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1638>
   35e09:	xor    ecx,ecx
   35e0b:	cmp    rdx,0x1
   35e0f:	jne    35e21 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1641>
   35e11:	mov    r9,QWORD PTR [rsp+0x20]
   35e16:	jmp    35e4d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x166d>
   35e18:	mov    rcx,0xffffffffffffffff
   35e1f:	jmp    35e5c <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x167c>
   35e21:	mov    r9,QWORD PTR [rsp+0x20]
   35e26:	cs nop WORD PTR [rax+rax*1+0x0]
   35e30:	mov    rsi,rcx
   35e33:	mov    r8,rdx
   35e36:	shr    r8,1
   35e39:	add    rcx,r8
   35e3c:	cmp    DWORD PTR [r9+rcx*4],eax
   35e40:	cmova  rcx,rsi
   35e44:	sub    rdx,r8
   35e47:	cmp    rdx,0x1
   35e4b:	ja     35e30 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1650>
   35e4d:	cmp    DWORD PTR [r9+rcx*4],eax
   35e51:	mov    rax,0xffffffffffffffff
   35e58:	cmovne rcx,rax
   35e5c:	cmp    rdi,QWORD PTR [rsp+0x48]
   35e61:	mov    rdx,QWORD PTR [rsp+0x8]
   35e66:	jae    36151 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1971>
   35e6c:	mov    rax,QWORD PTR [rsp+0x30]
   35e71:	mov    QWORD PTR [rax+rdi*8],rcx
   35e75:	dec    r12
   35e78:	mov    rax,r12
   35e7b:	shl    rax,0x5
   35e7f:	mov    rcx,QWORD PTR [rsp]
   35e83:	movups xmm0,XMMWORD PTR [rcx+rax*1]
   35e87:	movups xmm1,XMMWORD PTR [rcx+rax*1+0x10]
   35e8c:	movups XMMWORD PTR [rbp+0x10],xmm1
   35e90:	movups XMMWORD PTR [rbp+0x0],xmm0
   35e94:	cmp    r15,r12
   35e97:	jb     35d23 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1543>
   35e9d:	mov    r15d,0x0
   35ea3:	test   r12,r12
   35ea6:	jne    35d23 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1543>
   35eac:	mov    r10,QWORD PTR [rsp+0x78]
   35eb1:	mov    rcx,QWORD PTR [rsp+0x10]
   35eb6:	mov    r14,QWORD PTR [rsp+0x28]
   35ebb:	mov    rax,0xffffffffffffffff
   35ec2:	test   rcx,rcx
   35ec5:	mov    rsi,QWORD PTR [rsp+0x18]
   35eca:	je     35f6f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x178f>
   35ed0:	mov    r9,rcx
   35ed3:	lea    rcx,[rcx*4+0x0]
   35edb:	xor    edx,edx
   35edd:	mov    r8,QWORD PTR [rsp+0x8]
   35ee2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35ef0:	mov    rdi,QWORD PTR [rsp+0x20]
   35ef5:	mov    edi,DWORD PTR [rdi+rdx*1]
   35ef8:	cmp    r10,rdi
   35efb:	jbe    36179 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1999>
   35f01:	and    BYTE PTR [r8+rdi*4+0x3],0x7f
   35f07:	add    rdx,0x4
   35f0b:	cmp    rcx,rdx
   35f0e:	jne    35ef0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1710>
   35f10:	test   r9,r9
   35f13:	je     35f6f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x178f>
   35f15:	mov    rdx,r9
   35f18:	xor    ecx,ecx
   35f1a:	cmp    r9,0x1
   35f1e:	jne    35f2b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x174b>
   35f20:	mov    edi,DWORD PTR [rsp+0x3c]
   35f24:	mov    r8,QWORD PTR [rsp+0x20]
   35f29:	jmp    35f60 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1780>
   35f2b:	mov    edi,DWORD PTR [rsp+0x3c]
   35f2f:	mov    r8,QWORD PTR [rsp+0x20]
   35f34:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   35f40:	mov    rax,rcx
   35f43:	mov    r9,rdx
   35f46:	shr    rdx,1
   35f49:	add    rcx,rdx
   35f4c:	cmp    DWORD PTR [r8+rcx*4],edi
   35f50:	cmova  rcx,rax
   35f54:	sub    r9,rdx
   35f57:	mov    rdx,r9
   35f5a:	cmp    r9,0x1
   35f5e:	ja     35f40 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1760>
   35f60:	cmp    DWORD PTR [r8+rcx*4],edi
   35f64:	mov    rax,0xffffffffffffffff
   35f6b:	cmove  rax,rcx
   35f6f:	mov    QWORD PTR [rsi+0x68],rax
   35f73:	mov    QWORD PTR [rsi+0x60],r10
   35f77:	test   r14,r14
   35f7a:	je     35f86 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17a6>
   35f7c:	mov    rdi,QWORD PTR [rsp]
   35f80:	call   QWORD PTR [rip+0x602da]        # 96260 <free@GLIBC_2.2.5>
   35f86:	add    rsp,0x698
   35f8d:	pop    rbx
   35f8e:	pop    r12
   35f90:	pop    r13
   35f92:	pop    r14
   35f94:	pop    r15
   35f96:	pop    rbp
   35f97:	ret
   35f98:	mov    edx,0x1ff
   35f9d:	mov    r14,rcx
   35fa0:	mov    ecx,0x4
   35fa5:	mov    r8d,0x4
   35fab:	mov    r15d,esi
   35fae:	mov    rdi,r14
   35fb1:	xor    esi,esi
   35fb3:	mov    r12,r10
   35fb6:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   35fbb:	mov    r10,r12
   35fbe:	mov    esi,r15d
   35fc1:	mov    rcx,r14
   35fc4:	mov    rbp,QWORD PTR [r14+0x10]
   35fc8:	jmp    3482b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x4b>
   35fcd:	mov    rsi,r13
   35fd0:	cmp    r13,0x15
   35fd4:	jae    3623b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a5b>
   35fda:	lea    rax,[r14+rsi*4]
   35fde:	lea    rcx,[r14+0x4]
   35fe2:	mov    edx,0x4
   35fe7:	jmp    35fff <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x181f>
   35fe9:	mov    rdi,r14
   35fec:	mov    DWORD PTR [rdi],esi
   35fee:	add    rcx,0x4
   35ff2:	add    rdx,0x4
   35ff6:	cmp    rcx,rax
   35ff9:	je     36249 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a69>
   35fff:	mov    r8d,DWORD PTR [rcx-0x4]
   36003:	mov    esi,DWORD PTR [rcx]
   36005:	cmp    esi,r8d
   36008:	jae    35fee <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x180e>
   3600a:	mov    rdi,rdx
   3600d:	mov    DWORD PTR [r14+rdi*1],r8d
   36011:	cmp    rdi,0x4
   36015:	je     35fe9 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1809>
   36017:	mov    r8d,DWORD PTR [r14+rdi*1-0x8]
   3601c:	add    rdi,0xfffffffffffffffc
   36020:	cmp    esi,r8d
   36023:	jb     3600d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x182d>
   36025:	add    rdi,r14
   36028:	jmp    35fec <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x180c>
   3602a:	lea    rdi,[r12+0x48]
   3602f:	mov    ecx,0x8
   36034:	mov    r8d,0x8
   3603a:	xor    esi,esi
   3603c:	mov    rdx,r13
   3603f:	mov    rbx,r10
   36042:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   36047:	mov    r10,rbx
   3604a:	mov    rbx,QWORD PTR [r12+0x58]
   3604f:	jmp    349e7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x207>
   36054:	lea    rdi,[r12+0x30]
   36059:	mov    ecx,0x8
   3605e:	mov    r8d,0x18
   36064:	mov    rsi,r14
   36067:	mov    rdx,rbx
   3606a:	mov    rbp,r10
   3606d:	call   36370 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsg4oLtdKOaQv_5mibz2>
   36072:	mov    r10,rbp
   36075:	mov    rdx,QWORD PTR [r12+0x40]
   3607a:	mov    rax,QWORD PTR [r12+0x38]
   3607f:	inc    r14
   36082:	test   bl,0x1
   36085:	je     3495a <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x17a>
   3608b:	dec    rbx
   3608e:	lea    rcx,[rdx+rdx*2]
   36092:	mov    QWORD PTR [rax+rcx*8],0x0
   3609a:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   360a3:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   360ac:	inc    rdx
   360af:	cmp    r13,r14
   360b2:	je     34963 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x183>
   360b8:	lea    rcx,[rdx+rdx*2]
   360bc:	lea    rax,[rax+rcx*8]
   360c0:	add    rax,0x28
   360c4:	neg    rbx
   360c7:	xor    ecx,ecx
   360c9:	xorps  xmm0,xmm0
   360cc:	nop    DWORD PTR [rax+0x0]
   360d0:	mov    QWORD PTR [rax-0x28],0x0
   360d8:	mov    QWORD PTR [rax-0x20],0x1
   360e0:	movups XMMWORD PTR [rax-0x18],xmm0
   360e4:	mov    QWORD PTR [rax-0x8],0x1
   360ec:	mov    QWORD PTR [rax],0x0
   360f3:	add    rax,0x30
   360f7:	add    rcx,0xfffffffffffffffe
   360fb:	cmp    rbx,rcx
   360fe:	jne    360d0 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x18f0>
   36100:	sub    rdx,rcx
   36103:	mov    QWORD PTR [r12+0x40],rdx
   36108:	cmp    r13,rdx
   3610b:	jbe    34971 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x191>
   36111:	jmp    35958 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1178>
   36116:	lea    rdx,[rip+0x5dccb]        # 93de8 <__frame_dummy_init_array_entry+0x8c0>
   3611d:	mov    rsi,r10
   36120:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36125:	lea    rdx,[rip+0x5df74]        # 940a0 <__frame_dummy_init_array_entry+0xb78>
   3612c:	mov    rsi,rdi
   3612f:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36134:	lea    rdx,[rip+0x5df7d]        # 940b8 <__frame_dummy_init_array_entry+0xb90>
   3613b:	mov    rdi,QWORD PTR [rsp+0x10]
   36140:	mov    rsi,rdi
   36143:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36148:	lea    rax,[rip+0x5def1]        # 94040 <__frame_dummy_init_array_entry+0xb18>
   3614f:	jmp    3615d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x197d>
   36151:	mov    r13,QWORD PTR [rsp+0x48]
   36156:	lea    rax,[rip+0x5defb]        # 94058 <__frame_dummy_init_array_entry+0xb30>
   3615d:	mov    QWORD PTR [rsp+0x40],rax
   36162:	mov    rsi,r13
   36165:	mov    rdx,QWORD PTR [rsp+0x40]
   3616a:	mov    r14,QWORD PTR [rsp+0x28]
   3616f:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36174:	jmp    3621b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a3b>
   36179:	lea    rdx,[rip+0x5dcb0]        # 93e30 <__frame_dummy_init_array_entry+0x908>
   36180:	mov    rsi,r10
   36183:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36188:	jmp    3621b <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a3b>
   3618d:	lea    rdx,[rip+0x5dec4]        # 94058 <__frame_dummy_init_array_entry+0xb30>
   36194:	mov    rdi,rax
   36197:	mov    rsi,QWORD PTR [rsp+0x30]
   3619c:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   361a1:	lea    rdx,[rip+0x5de98]        # 94040 <__frame_dummy_init_array_entry+0xb18>
   361a8:	mov    rdi,rax
   361ab:	mov    rsi,QWORD PTR [rsp]
   361af:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   361b4:	lea    rdx,[rip+0x5dc5d]        # 93e18 <__frame_dummy_init_array_entry+0x8f0>
   361bb:	mov    esi,0x20
   361c0:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   361c5:	lea    rdx,[rip+0x5ded4]        # 940a0 <__frame_dummy_init_array_entry+0xb78>
   361cc:	mov    rdi,r13
   361cf:	mov    rsi,QWORD PTR [rsp]
   361d3:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   361d8:	lea    rdx,[rip+0x5ded9]        # 940b8 <__frame_dummy_init_array_entry+0xb90>
   361df:	mov    rdi,r13
   361e2:	mov    rsi,QWORD PTR [rsp+0x10]
   361e7:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   361ec:	lea    rdx,[rip+0x5de95]        # 94088 <__frame_dummy_init_array_entry+0xb60>
   361f3:	mov    rdi,rbx
   361f6:	mov    rsi,QWORD PTR [rsp+0x30]
   361fb:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   36200:	lea    rdx,[rip+0x5de69]        # 94070 <__frame_dummy_init_array_entry+0xb48>
   36207:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3620c:	lea    rdx,[rip+0x5de75]        # 94088 <__frame_dummy_init_array_entry+0xb60>
   36213:	mov    rdi,r14
   36216:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3621b:	ud2
   3621d:	lea    rdx,[rip+0x5dbac]        # 93dd0 <__frame_dummy_init_array_entry+0x8a8>
   36224:	mov    rsi,r10
   36227:	call   2a5ab <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   3622c:	mov    edi,0x8
   36231:	mov    esi,0x80
   36236:	call   28e01 <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   3623b:	mov    rdi,r14
   3623e:	mov    rbx,r10
   36241:	call   36460 <_RINvNtNtNtCscI6d9CVNmLh_4core5slice4sort8unstable7ipnsortmNvYmNtNtB8_3cmp10PartialOrd2ltECsg4oLtdKOaQv_5mibz2>
   36246:	mov    r10,rbx
   36249:	xor    ecx,ecx
   3624b:	mov    rdx,QWORD PTR [rsp+0x8]
   36250:	mov    eax,DWORD PTR [r14+rcx*4+0x4]
   36255:	cmp    eax,DWORD PTR [r14+rcx*4]
   36259:	je     36273 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a93>
   3625b:	inc    rcx
   3625e:	cmp    r12,rcx
   36261:	jne    36250 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1a70>
   36263:	mov    r12,QWORD PTR [rsp+0x18]
   36268:	test   r13,r13
   3626b:	jne    348c7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe7>
   36271:	jmp    362d9 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1af9>
   36273:	lea    rdx,[rcx+0x1]
   36277:	lea    rax,[rcx+0x2]
   3627b:	cmp    rax,r13
   3627e:	jae    36293 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ab3>
   36280:	mov    esi,ebp
   36282:	sub    esi,ecx
   36284:	add    rbp,0x1fd
   3628b:	test   sil,0x1
   3628f:	jne    36298 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ab8>
   36291:	jmp    362bc <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1adc>
   36293:	mov    r13,rdx
   36296:	jmp    362c1 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ae1>
   36298:	mov    eax,DWORD PTR [r14+rcx*4+0x8]
   3629d:	mov    rsi,rdx
   362a0:	cmp    eax,DWORD PTR [r14+rcx*4]
   362a4:	je     362af <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1acf>
   362a6:	mov    DWORD PTR [r14+rcx*4+0x4],eax
   362ab:	lea    rsi,[rdx+0x1]
   362af:	add    rdx,0x2
   362b3:	mov    rax,rdx
   362b6:	mov    r13,rsi
   362b9:	mov    rdx,rsi
   362bc:	cmp    rbp,rcx
   362bf:	jne    362e5 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b05>
   362c1:	mov    r12,QWORD PTR [rsp+0x18]
   362c6:	mov    QWORD PTR [r12+0x10],r13
   362cb:	mov    rdx,QWORD PTR [rsp+0x8]
   362d0:	test   r13,r13
   362d3:	jne    348c7 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0xe7>
   362d9:	lea    rdi,[rip+0x5db20]        # 93e00 <__frame_dummy_init_array_entry+0x8d8>
   362e0:	call   2cfa0 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   362e5:	inc    r12
   362e8:	mov    r13,rdx
   362eb:	jmp    362f6 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b16>
   362ed:	add    rax,0x2
   362f1:	cmp    r12,rax
   362f4:	je     362c1 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1ae1>
   362f6:	mov    ecx,DWORD PTR [r14+rax*4]
   362fa:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   362ff:	jne    3630f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b2f>
   36301:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   36306:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   3630b:	je     362ed <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0d>
   3630d:	jmp    36322 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b42>
   3630f:	mov    DWORD PTR [r14+r13*4],ecx
   36313:	inc    r13
   36316:	mov    ecx,DWORD PTR [r14+rax*4+0x4]
   3631b:	cmp    ecx,DWORD PTR [r14+r13*4-0x4]
   36320:	je     362ed <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0d>
   36322:	mov    DWORD PTR [r14+r13*4],ecx
   36326:	inc    r13
   36329:	jmp    362ed <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b0d>
   3632b:	jmp    3632f <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b4f>
   3632d:	jmp    36339 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b59>
   3632f:	mov    rbx,rax
   36332:	mov    r14,QWORD PTR [rsp+0x28]
   36337:	jmp    36358 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b78>
   36339:	mov    rbx,rax
   3633c:	cmp    QWORD PTR [rsp+0x198],0x0
   36345:	je     36367 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b87>
   36347:	mov    rax,QWORD PTR [rsp+0x1a0]
   3634f:	mov    QWORD PTR [rsp],rax
   36353:	jmp    3635d <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b7d>
   36355:	mov    rbx,rax
   36358:	test   r14,r14
   3635b:	je     36367 <_RINvNtCsg4oLtdKOaQv_5mibz23bwt13room_segmentsKj20_Kh1_EB4_+0x1b87>
   3635d:	mov    rdi,QWORD PTR [rsp]
   36361:	call   QWORD PTR [rip+0x5fef9]        # 96260 <free@GLIBC_2.2.5>
   36367:	mov    rdi,rbx
   3636a:	call   924e0 <_Unwind_Resume@plt>
