
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000015aa0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_>:
   15aa0:	push   rbp
   15aa1:	push   r15
   15aa3:	push   r14
   15aa5:	push   r13
   15aa7:	push   r12
   15aa9:	push   rbx
   15aaa:	sub    rsp,0x638
   15ab1:	mov    DWORD PTR [rsp+0x154],esi
   15ab8:	mov    rax,QWORD PTR [rdi+0xb8]
   15abf:	mov    QWORD PTR [rsp+0x178],rax
   15ac7:	mov    eax,DWORD PTR [rdi+0x10]
   15aca:	mov    r12,QWORD PTR [rdi+0x40]
   15ace:	mov    QWORD PTR [rdi+0xb0],0x0
   15ad9:	mov    QWORD PTR [rdi+0x98],0x0
   15ae4:	test   r12,r12
   15ae7:	je     17ab1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2011>
   15aed:	mov    rbx,rdi
   15af0:	shl    eax,0x5
   15af3:	add    eax,0xffffffe0
   15af6:	mov    DWORD PTR [rsp+0x24],eax
   15afa:	xor    edi,edi
   15afc:	lea    rax,[rbx+0xa0]
   15b03:	mov    QWORD PTR [rsp],rax
   15b07:	lea    rax,[rbx+0x88]
   15b0e:	mov    QWORD PTR [rsp+0x8],rax
   15b13:	mov    QWORD PTR [rsp+0x28],r12
   15b18:	jmp    15b49 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xa9>
   15b1a:	nop    WORD PTR [rax+rax*1+0x0]
   15b20:	inc    rdi
   15b23:	mov    rax,QWORD PTR [r14+0x90]
   15b2a:	mov    WORD PTR [rax+rbx*2],bp
   15b2e:	inc    rbx
   15b31:	mov    QWORD PTR [r14+0x98],rbx
   15b38:	mov    r12,QWORD PTR [rsp+0x28]
   15b3d:	cmp    r12,rdi
   15b40:	mov    rbx,r14
   15b43:	je     15c80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e0>
   15b49:	mov    rsi,QWORD PTR [rbx+0x40]
   15b4d:	cmp    rdi,rsi
   15b50:	jae    179a0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f00>
   15b56:	mov    rsi,QWORD PTR [rbx+0x10]
   15b5a:	mov    rax,QWORD PTR [rbx+0x38]
   15b5e:	mov    ebp,DWORD PTR [rax+rdi*4]
   15b61:	mov    r12d,ebp
   15b64:	shr    r12d,0x5
   15b68:	cmp    rsi,r12
   15b6b:	jbe    17991 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ef1>
   15b71:	and    ebp,0x1f
   15b74:	cmp    ebp,0x1e
   15b77:	jae    1797d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1edd>
   15b7d:	mov    rcx,QWORD PTR [rbx+0x8]
   15b81:	mov    rdx,r12
   15b84:	shl    rdx,0x6
   15b88:	lea    rax,[rcx+rdx*1]
   15b8c:	movzx  eax,WORD PTR [rax+rbp*2]
   15b90:	mov    r13d,DWORD PTR [rcx+rdx*1+0x3c]
   15b95:	cmp    eax,0xfffe
   15b9a:	jae    15c20 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x180>
   15ba0:	mov    r14,rbx
   15ba3:	mov    ebx,r13d
   15ba6:	shr    ebx,0x8
   15ba9:	add    ebx,eax
   15bab:	mov    r15,QWORD PTR [r14+0xb0]
   15bb2:	cmp    r15,QWORD PTR [r14+0xa0]
   15bb9:	je     15c5e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1be>
   15bbf:	shl    ebx,0x8
   15bc2:	movzx  eax,r13b
   15bc6:	or     eax,ebx
   15bc8:	mov    rcx,QWORD PTR [r14+0xa8]
   15bcf:	mov    DWORD PTR [rcx+r15*4],eax
   15bd3:	inc    r15
   15bd6:	mov    QWORD PTR [r14+0xb0],r15
   15bdd:	mov    rsi,QWORD PTR [r14+0x10]
   15be1:	cmp    rsi,r12
   15be4:	jbe    1796e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ece>
   15bea:	add    rdx,QWORD PTR [r14+0x8]
   15bee:	movzx  ebp,WORD PTR [rdx+rbp*2]
   15bf2:	mov    rbx,QWORD PTR [r14+0x98]
   15bf9:	cmp    rbx,QWORD PTR [r14+0x88]
   15c00:	jne    15b20 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
   15c06:	mov    r15,rdi
   15c09:	mov    rdi,QWORD PTR [rsp+0x8]
   15c0e:	call   2c840 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVectE8grow_oneCs3AlRknvjqqN_7c30walk>
   15c13:	mov    rdi,r15
   15c16:	jmp    15b20 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
   15c1b:	nop    DWORD PTR [rax+rax*1+0x0]
   15c20:	and    r13d,0xfffff
   15c27:	mov    rsi,QWORD PTR [rbx+0x28]
   15c2b:	cmp    rsi,r13
   15c2e:	jbe    17a26 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f86>
   15c34:	shl    r13d,0x7
   15c38:	add    r13,QWORD PTR [rbx+0x20]
   15c3c:	mov    r13d,DWORD PTR [r13+rbp*4+0x0]
   15c41:	mov    r14,rbx
   15c44:	mov    ebx,r13d
   15c47:	shr    ebx,0x8
   15c4a:	mov    r15,QWORD PTR [r14+0xb0]
   15c51:	cmp    r15,QWORD PTR [r14+0xa0]
   15c58:	jne    15bbf <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
   15c5e:	mov    QWORD PTR [rsp+0x38],rdi
   15c63:	mov    rdi,QWORD PTR [rsp]
   15c67:	mov    QWORD PTR [rsp+0x18],rdx
   15c6c:	call   36730 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCs7ivwnIpGHJL_5mibz2>
   15c71:	mov    rdx,QWORD PTR [rsp+0x18]
   15c76:	mov    rdi,QWORD PTR [rsp+0x38]
   15c7b:	jmp    15bbf <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
   15c80:	xor    edi,edi
   15c82:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   15c90:	mov    rsi,QWORD PTR [rbx+0x40]
   15c94:	cmp    rdi,rsi
   15c97:	jae    179cf <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f2f>
   15c9d:	mov    rsi,QWORD PTR [rbx+0x10]
   15ca1:	mov    rax,QWORD PTR [rbx+0x38]
   15ca5:	mov    eax,DWORD PTR [rax+rdi*4]
   15ca8:	mov    ecx,eax
   15caa:	shr    ecx,0x5
   15cad:	cmp    rsi,rcx
   15cb0:	jbe    179c0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f20>
   15cb6:	and    eax,0x1f
   15cb9:	cmp    eax,0x1e
   15cbc:	jae    179ac <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0c>
   15cc2:	inc    rdi
   15cc5:	shl    rcx,0x6
   15cc9:	add    rcx,QWORD PTR [rbx+0x8]
   15ccd:	mov    WORD PTR [rcx+rax*2],0xfffe
   15cd3:	cmp    r12,rdi
   15cd6:	jne    15c90 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0>
   15cd8:	mov    r14,QWORD PTR [rbx+0x8]
   15cdc:	mov    rax,QWORD PTR [rbx+0x20]
   15ce0:	mov    QWORD PTR [rsp+0x38],rax
   15ce5:	mov    rax,QWORD PTR [rsp+0x178]
   15ced:	add    rax,rax
   15cf0:	mov    rcx,rax
   15cf3:	or     rcx,r12
   15cf6:	shr    rcx,0x20
   15cfa:	je     15d52 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2b2>
   15cfc:	xor    edx,edx
   15cfe:	div    r12
   15d01:	mov    rbp,rax
   15d04:	mov    r13,QWORD PTR [rbx+0x58]
   15d08:	cmp    r13,r12
   15d0b:	cmova  r12,r13
   15d0f:	jae    15d66 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2c6>
   15d11:	sub    r12,r13
   15d14:	mov    rax,QWORD PTR [rbx+0x48]
   15d18:	sub    rax,r13
   15d1b:	mov    rdx,r13
   15d1e:	cmp    r12,rax
   15d21:	ja     15d74 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2d4>
   15d23:	mov    rax,QWORD PTR [rbx+0x50]
   15d27:	inc    r13
   15d2a:	test   r12b,0x1
   15d2e:	jne    15d9f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2ff>
   15d30:	cmp    QWORD PTR [rsp+0x28],r13
   15d35:	jne    15dce <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x32e>
   15d3b:	mov    QWORD PTR [rbx+0x58],rdx
   15d3f:	mov    rsi,QWORD PTR [rsp+0x28]
   15d44:	cmp    rsi,rdx
   15d47:	jbe    15e35 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
   15d4d:	jmp    1793e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
   15d52:	xor    edx,edx
   15d54:	div    r12d
   15d57:	mov    ebp,eax
   15d59:	mov    r13,QWORD PTR [rbx+0x58]
   15d5d:	cmp    r13,r12
   15d60:	cmova  r12,r13
   15d64:	jb     15d11 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x271>
   15d66:	mov    QWORD PTR [rbx+0x58],r12
   15d6a:	mov    rsi,QWORD PTR [rsp+0x28]
   15d6f:	jmp    15e35 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
   15d74:	lea    rdi,[rbx+0x48]
   15d78:	mov    ecx,0x8
   15d7d:	mov    r8d,0x18
   15d83:	mov    rsi,r13
   15d86:	mov    rdx,r12
   15d89:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   15d8e:	mov    rdx,QWORD PTR [rbx+0x58]
   15d92:	mov    rax,QWORD PTR [rbx+0x50]
   15d96:	inc    r13
   15d99:	test   r12b,0x1
   15d9d:	je     15d30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x290>
   15d9f:	dec    r12
   15da2:	lea    rcx,[rdx+rdx*2]
   15da6:	mov    QWORD PTR [rax+rcx*8],0x0
   15dae:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
   15db7:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
   15dc0:	inc    rdx
   15dc3:	cmp    QWORD PTR [rsp+0x28],r13
   15dc8:	je     15d3b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x29b>
   15dce:	lea    rcx,[rdx+rdx*2]
   15dd2:	lea    rax,[rax+rcx*8]
   15dd6:	add    rax,0x28
   15dda:	neg    r12
   15ddd:	xor    ecx,ecx
   15ddf:	xorps  xmm0,xmm0
   15de2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   15df0:	mov    QWORD PTR [rax-0x28],0x0
   15df8:	mov    QWORD PTR [rax-0x20],0x1
   15e00:	movups XMMWORD PTR [rax-0x18],xmm0
   15e04:	mov    QWORD PTR [rax-0x8],0x1
   15e0c:	mov    QWORD PTR [rax],0x0
   15e13:	add    rax,0x30
   15e17:	add    rcx,0xfffffffffffffffe
   15e1b:	cmp    r12,rcx
   15e1e:	jne    15df0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x350>
   15e20:	sub    rdx,rcx
   15e23:	mov    QWORD PTR [rbx+0x58],rdx
   15e27:	mov    rsi,QWORD PTR [rsp+0x28]
   15e2c:	cmp    rsi,rdx
   15e2f:	ja     1793e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
   15e35:	lea    r12,[rbp+0x40]
   15e39:	mov    r13,QWORD PTR [rbx+0x50]
   15e3d:	lea    rax,[rsi*8+0x0]
   15e45:	lea    r15,[rax+rax*2]
   15e49:	nop    DWORD PTR [rax+0x0]
   15e50:	mov    QWORD PTR [r13+0x10],0x0
   15e58:	cmp    r12,QWORD PTR [r13+0x0]
   15e5c:	ja     15e6a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ca>
   15e5e:	add    r13,0x18
   15e62:	add    r15,0xffffffffffffffe8
   15e66:	jne    15e50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
   15e68:	jmp    15e8c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ec>
   15e6a:	mov    ecx,0x1
   15e6f:	mov    r8d,0x1
   15e75:	mov    rdi,r13
   15e78:	xor    esi,esi
   15e7a:	mov    rdx,r12
   15e7d:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   15e82:	add    r13,0x18
   15e86:	add    r15,0xffffffffffffffe8
   15e8a:	jne    15e50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
   15e8c:	mov    QWORD PTR [rbx+0x70],0x0
   15e94:	mov    r12,QWORD PTR [rsp+0x28]
   15e99:	cmp    r12,QWORD PTR [rbx+0x60]
   15e9d:	ja     1794c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1eac>
   15ea3:	xor    r15d,r15d
   15ea6:	mov    QWORD PTR [rsp+0x18],rbx
   15eab:	mov    rbx,QWORD PTR [rbx+0x68]
   15eaf:	lea    rdi,[rbx+r15*8]
   15eb3:	cmp    r12,0x1
   15eb7:	je     15ede <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x43e>
   15eb9:	lea    rdx,[r12*8-0x8]
   15ec1:	mov    esi,0xff
   15ec6:	call   QWORD PTR [rip+0x69494]        # 7f360 <memset@GLIBC_2.2.5>
   15ecc:	lea    rax,[r15+r12*1]
   15ed0:	lea    rdi,[rbx+rax*8]
   15ed4:	add    rdi,0xfffffffffffffff8
   15ed8:	add    r15,r12
   15edb:	dec    r15
   15ede:	mov    QWORD PTR [rdi],0xffffffffffffffff
   15ee5:	inc    r15
   15ee8:	mov    rbx,QWORD PTR [rsp+0x18]
   15eed:	mov    QWORD PTR [rbx+0x70],r15
   15ef1:	xorps  xmm0,xmm0
   15ef4:	movups XMMWORD PTR [rsp+0x240],xmm0
   15efc:	mov    QWORD PTR [rsp+0x250],0x0
   15f08:	movups XMMWORD PTR [rsp+0x260],xmm0
   15f10:	mov    QWORD PTR [rsp+0x270],0x0
   15f1c:	movups XMMWORD PTR [rsp+0x280],xmm0
   15f24:	mov    QWORD PTR [rsp+0x290],0x0
   15f30:	movups XMMWORD PTR [rsp+0x2a0],xmm0
   15f38:	mov    QWORD PTR [rsp+0x2b0],0x0
   15f44:	movups XMMWORD PTR [rsp+0x2c0],xmm0
   15f4c:	mov    QWORD PTR [rsp+0x2d0],0x0
   15f58:	movups XMMWORD PTR [rsp+0x2e0],xmm0
   15f60:	mov    QWORD PTR [rsp+0x2f0],0x0
   15f6c:	movups XMMWORD PTR [rsp+0x300],xmm0
   15f74:	mov    QWORD PTR [rsp+0x310],0x0
   15f80:	movups XMMWORD PTR [rsp+0x320],xmm0
   15f88:	mov    QWORD PTR [rsp+0x330],0x0
   15f94:	movups XMMWORD PTR [rsp+0x340],xmm0
   15f9c:	mov    QWORD PTR [rsp+0x350],0x0
   15fa8:	movups XMMWORD PTR [rsp+0x360],xmm0
   15fb0:	mov    QWORD PTR [rsp+0x370],0x0
   15fbc:	movups XMMWORD PTR [rsp+0x380],xmm0
   15fc4:	mov    QWORD PTR [rsp+0x390],0x0
   15fd0:	movups XMMWORD PTR [rsp+0x3a0],xmm0
   15fd8:	mov    QWORD PTR [rsp+0x3b0],0x0
   15fe4:	movups XMMWORD PTR [rsp+0x3c0],xmm0
   15fec:	mov    QWORD PTR [rsp+0x3d0],0x0
   15ff8:	movups XMMWORD PTR [rsp+0x3e0],xmm0
   16000:	mov    QWORD PTR [rsp+0x3f0],0x0
   1600c:	movups XMMWORD PTR [rsp+0x400],xmm0
   16014:	mov    QWORD PTR [rsp+0x410],0x0
   16020:	movups XMMWORD PTR [rsp+0x420],xmm0
   16028:	mov    QWORD PTR [rsp+0x430],0x0
   16034:	movups XMMWORD PTR [rsp+0x440],xmm0
   1603c:	mov    QWORD PTR [rsp+0x450],0x0
   16048:	movups XMMWORD PTR [rsp+0x460],xmm0
   16050:	mov    QWORD PTR [rsp+0x470],0x0
   1605c:	movups XMMWORD PTR [rsp+0x480],xmm0
   16064:	mov    QWORD PTR [rsp+0x490],0x0
   16070:	movups XMMWORD PTR [rsp+0x4a0],xmm0
   16078:	mov    QWORD PTR [rsp+0x4b0],0x0
   16084:	movups XMMWORD PTR [rsp+0x4c0],xmm0
   1608c:	mov    QWORD PTR [rsp+0x4d0],0x0
   16098:	movups XMMWORD PTR [rsp+0x4e0],xmm0
   160a0:	mov    QWORD PTR [rsp+0x4f0],0x0
   160ac:	movups XMMWORD PTR [rsp+0x500],xmm0
   160b4:	mov    QWORD PTR [rsp+0x510],0x0
   160c0:	movups XMMWORD PTR [rsp+0x520],xmm0
   160c8:	mov    QWORD PTR [rsp+0x530],0x0
   160d4:	movups XMMWORD PTR [rsp+0x540],xmm0
   160dc:	mov    QWORD PTR [rsp+0x550],0x0
   160e8:	movups XMMWORD PTR [rsp+0x560],xmm0
   160f0:	mov    QWORD PTR [rsp+0x570],0x0
   160fc:	movups XMMWORD PTR [rsp+0x580],xmm0
   16104:	mov    QWORD PTR [rsp+0x590],0x0
   16110:	movups XMMWORD PTR [rsp+0x5a0],xmm0
   16118:	mov    QWORD PTR [rsp+0x5b0],0x0
   16124:	movups XMMWORD PTR [rsp+0x5c0],xmm0
   1612c:	mov    QWORD PTR [rsp+0x5d0],0x0
   16138:	movups XMMWORD PTR [rsp+0x5e0],xmm0
   16140:	mov    QWORD PTR [rsp+0x5f0],0x0
   1614c:	movups XMMWORD PTR [rsp+0x600],xmm0
   16154:	mov    QWORD PTR [rsp+0x610],0x0
   16160:	movups XMMWORD PTR [rsp+0x620],xmm0
   16168:	mov    QWORD PTR [rsp+0x630],0x0
   16174:	mov    QWORD PTR [rsp+0x238],0xffffffffffffffff
   16180:	mov    QWORD PTR [rsp+0x258],0xffffffffffffffff
   1618c:	mov    QWORD PTR [rsp+0x278],0xffffffffffffffff
   16198:	mov    QWORD PTR [rsp+0x298],0xffffffffffffffff
   161a4:	mov    QWORD PTR [rsp+0x2b8],0xffffffffffffffff
   161b0:	mov    QWORD PTR [rsp+0x2d8],0xffffffffffffffff
   161bc:	mov    QWORD PTR [rsp+0x2f8],0xffffffffffffffff
   161c8:	mov    QWORD PTR [rsp+0x318],0xffffffffffffffff
   161d4:	mov    QWORD PTR [rsp+0x338],0xffffffffffffffff
   161e0:	mov    QWORD PTR [rsp+0x358],0xffffffffffffffff
   161ec:	mov    QWORD PTR [rsp+0x378],0xffffffffffffffff
   161f8:	mov    QWORD PTR [rsp+0x398],0xffffffffffffffff
   16204:	mov    QWORD PTR [rsp+0x3b8],0xffffffffffffffff
   16210:	mov    QWORD PTR [rsp+0x3d8],0xffffffffffffffff
   1621c:	mov    QWORD PTR [rsp+0x3f8],0xffffffffffffffff
   16228:	mov    QWORD PTR [rsp+0x418],0xffffffffffffffff
   16234:	mov    QWORD PTR [rsp+0x438],0xffffffffffffffff
   16240:	mov    QWORD PTR [rsp+0x458],0xffffffffffffffff
   1624c:	mov    QWORD PTR [rsp+0x478],0xffffffffffffffff
   16258:	mov    QWORD PTR [rsp+0x498],0xffffffffffffffff
   16264:	mov    QWORD PTR [rsp+0x4b8],0xffffffffffffffff
   16270:	mov    QWORD PTR [rsp+0x4d8],0xffffffffffffffff
   1627c:	mov    QWORD PTR [rsp+0x4f8],0xffffffffffffffff
   16288:	mov    QWORD PTR [rsp+0x518],0xffffffffffffffff
   16294:	mov    QWORD PTR [rsp+0x538],0xffffffffffffffff
   162a0:	mov    QWORD PTR [rsp+0x558],0xffffffffffffffff
   162ac:	mov    QWORD PTR [rsp+0x578],0xffffffffffffffff
   162b8:	mov    QWORD PTR [rsp+0x598],0xffffffffffffffff
   162c4:	mov    QWORD PTR [rsp+0x5b8],0xffffffffffffffff
   162d0:	mov    QWORD PTR [rsp+0x5d8],0xffffffffffffffff
   162dc:	mov    QWORD PTR [rsp+0x5f8],0xffffffffffffffff
   162e8:	mov    QWORD PTR [rsp+0x618],0xffffffffffffffff
   162f4:	mov    eax,0x2
   162f9:	xor    edi,edi
   162fb:	xor    r9d,r9d
   162fe:	xchg   ax,ax
   16300:	mov    rsi,QWORD PTR [rbx+0x58]
   16304:	cmp    rdi,rsi
   16307:	jae    179e7 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f47>
   1630d:	mov    rsi,QWORD PTR [rbx+0xb0]
   16314:	cmp    rdi,rsi
   16317:	jae    179db <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f3b>
   1631d:	mov    rcx,QWORD PTR [rbx+0x50]
   16321:	mov    rdx,QWORD PTR [rbx+0xa8]
   16328:	mov    rsi,QWORD PTR [rcx+rax*4]
   1632c:	movzx  r8d,BYTE PTR [rdx+rdi*4]
   16331:	mov    BYTE PTR [rsi],r8b
   16334:	mov    edx,DWORD PTR [rdx+rdi*4]
   16337:	shr    edx,0x8
   1633a:	lea    r8,[rsi+0x1]
   1633e:	add    rsi,QWORD PTR [rcx+rax*4-0x8]
   16343:	mov    QWORD PTR [rsp+r9*1+0x238],rdi
   1634b:	mov    QWORD PTR [rsp+r9*1+0x240],r8
   16353:	mov    QWORD PTR [rsp+r9*1+0x248],rsi
   1635b:	mov    DWORD PTR [rsp+r9*1+0x250],edx
   16363:	mov    DWORD PTR [rsp+r9*1+0x254],0x0
   1636f:	prefetcht0 BYTE PTR [r14+rdx*2]
   16374:	lea    rsi,[rdi+0x1]
   16378:	add    r9,0x20
   1637c:	cmp    rdi,0x1e
   16380:	ja     16392 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x8f2>
   16382:	add    rax,0x6
   16386:	mov    rdi,rsi
   16389:	cmp    rsi,r12
   1638c:	jb     16300 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x860>
   16392:	cmp    rsi,0x20
   16396:	jne    17372 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
   1639c:	mov    QWORD PTR [rsp+0x170],rsi
   163a4:	add    rbp,0x3f
   163a8:	mov    eax,0x20
   163ad:	mov    QWORD PTR [rsp+0x8],rax
   163b2:	mov    QWORD PTR [rsp+0x230],rbp
   163ba:	jmp    1674d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcad>
   163bf:	nop
   163c0:	mov    rbp,QWORD PTR [rsp+0x48]
   163c5:	mov    r12,QWORD PTR [rsp+0x50]
   163ca:	mov    rax,QWORD PTR [rsp+0x58]
   163cf:	mov    QWORD PTR [rsp+0x10],rax
   163d4:	mov    rax,QWORD PTR [rsp+0x60]
   163d9:	mov    QWORD PTR [rsp+0x1b8],rax
   163e1:	mov    rax,QWORD PTR [rsp+0x68]
   163e6:	mov    QWORD PTR [rsp+0x1c0],rax
   163ee:	mov    rax,QWORD PTR [rsp+0x70]
   163f3:	mov    QWORD PTR [rsp+0x1c8],rax
   163fb:	mov    rax,QWORD PTR [rsp+0x78]
   16400:	mov    QWORD PTR [rsp+0x1d0],rax
   16408:	mov    rax,QWORD PTR [rsp+0x80]
   16410:	mov    QWORD PTR [rsp+0x1d8],rax
   16418:	mov    rax,QWORD PTR [rsp+0x88]
   16420:	mov    QWORD PTR [rsp+0x1e0],rax
   16428:	mov    rax,QWORD PTR [rsp+0x90]
   16430:	mov    QWORD PTR [rsp+0x1e8],rax
   16438:	mov    rax,QWORD PTR [rsp+0x98]
   16440:	mov    QWORD PTR [rsp+0x1f0],rax
   16448:	mov    rax,QWORD PTR [rsp+0xa0]
   16450:	mov    QWORD PTR [rsp+0x1f8],rax
   16458:	mov    rax,QWORD PTR [rsp+0xa8]
   16460:	mov    QWORD PTR [rsp+0x200],rax
   16468:	mov    rax,QWORD PTR [rsp+0xb0]
   16470:	mov    QWORD PTR [rsp+0x208],rax
   16478:	mov    rax,QWORD PTR [rsp+0xb8]
   16480:	mov    QWORD PTR [rsp],rax
   16484:	mov    rax,QWORD PTR [rsp+0xc0]
   1648c:	mov    QWORD PTR [rsp+0x190],rax
   16494:	mov    rax,QWORD PTR [rsp+0xc8]
   1649c:	mov    QWORD PTR [rsp+0x198],rax
   164a4:	mov    rax,QWORD PTR [rsp+0xd0]
   164ac:	mov    QWORD PTR [rsp+0x1a0],rax
   164b4:	mov    rax,QWORD PTR [rsp+0xd8]
   164bc:	mov    QWORD PTR [rsp+0x1a8],rax
   164c4:	mov    rax,QWORD PTR [rsp+0xe0]
   164cc:	mov    QWORD PTR [rsp+0x1b0],rax
   164d4:	mov    rax,QWORD PTR [rsp+0xe8]
   164dc:	mov    QWORD PTR [rsp+0x180],rax
   164e4:	mov    rax,QWORD PTR [rsp+0xf0]
   164ec:	mov    QWORD PTR [rsp+0x188],rax
   164f4:	mov    rdi,QWORD PTR [rsp+0xf8]
   164fc:	mov    rsi,QWORD PTR [rsp+0x100]
   16504:	mov    rdx,QWORD PTR [rsp+0x108]
   1650c:	mov    rcx,QWORD PTR [rsp+0x110]
   16514:	mov    rax,QWORD PTR [rsp+0x118]
   1651c:	mov    r13,QWORD PTR [rsp+0x120]
   16524:	mov    r11,QWORD PTR [rsp+0x128]
   1652c:	mov    r10,QWORD PTR [rsp+0x130]
   16534:	mov    r9,QWORD PTR [rsp+0x138]
   1653c:	mov    r8,QWORD PTR [rsp+0x140]
   16544:	add    rbp,r15
   16547:	mov    QWORD PTR [rsp+0x240],rbp
   1654f:	add    r12,r15
   16552:	mov    QWORD PTR [rsp+0x260],r12
   1655a:	mov    r12,QWORD PTR [rsp+0x10]
   1655f:	add    r12,r15
   16562:	mov    QWORD PTR [rsp+0x280],r12
   1656a:	mov    r12,QWORD PTR [rsp+0x1b8]
   16572:	add    r12,r15
   16575:	mov    QWORD PTR [rsp+0x2a0],r12
   1657d:	mov    r12,QWORD PTR [rsp+0x1c0]
   16585:	add    r12,r15
   16588:	mov    QWORD PTR [rsp+0x2c0],r12
   16590:	mov    r12,QWORD PTR [rsp+0x1c8]
   16598:	add    r12,r15
   1659b:	mov    QWORD PTR [rsp+0x2e0],r12
   165a3:	mov    r12,QWORD PTR [rsp+0x1d0]
   165ab:	add    r12,r15
   165ae:	mov    QWORD PTR [rsp+0x300],r12
   165b6:	mov    r12,QWORD PTR [rsp+0x1d8]
   165be:	add    r12,r15
   165c1:	mov    QWORD PTR [rsp+0x320],r12
   165c9:	mov    r12,QWORD PTR [rsp+0x1e0]
   165d1:	add    r12,r15
   165d4:	mov    QWORD PTR [rsp+0x340],r12
   165dc:	mov    r12,QWORD PTR [rsp+0x1e8]
   165e4:	add    r12,r15
   165e7:	mov    QWORD PTR [rsp+0x360],r12
   165ef:	mov    r12,QWORD PTR [rsp+0x1f0]
   165f7:	add    r12,r15
   165fa:	mov    QWORD PTR [rsp+0x380],r12
   16602:	mov    r12,QWORD PTR [rsp+0x1f8]
   1660a:	add    r12,r15
   1660d:	mov    QWORD PTR [rsp+0x3a0],r12
   16615:	mov    r12,QWORD PTR [rsp+0x200]
   1661d:	add    r12,r15
   16620:	mov    QWORD PTR [rsp+0x3c0],r12
   16628:	mov    r12,QWORD PTR [rsp+0x208]
   16630:	add    r12,r15
   16633:	mov    QWORD PTR [rsp+0x3e0],r12
   1663b:	mov    r12,QWORD PTR [rsp]
   1663f:	add    r12,r15
   16642:	mov    QWORD PTR [rsp+0x400],r12
   1664a:	mov    r12,QWORD PTR [rsp+0x190]
   16652:	add    r12,r15
   16655:	mov    QWORD PTR [rsp+0x420],r12
   1665d:	mov    r12,QWORD PTR [rsp+0x198]
   16665:	add    r12,r15
   16668:	mov    QWORD PTR [rsp+0x440],r12
   16670:	mov    r12,QWORD PTR [rsp+0x1a0]
   16678:	add    r12,r15
   1667b:	mov    QWORD PTR [rsp+0x460],r12
   16683:	mov    r12,QWORD PTR [rsp+0x1a8]
   1668b:	add    r12,r15
   1668e:	mov    QWORD PTR [rsp+0x480],r12
   16696:	mov    r12,QWORD PTR [rsp+0x1b0]
   1669e:	add    r12,r15
   166a1:	mov    QWORD PTR [rsp+0x4a0],r12
   166a9:	mov    r12,QWORD PTR [rsp+0x180]
   166b1:	add    r12,r15
   166b4:	mov    QWORD PTR [rsp+0x4c0],r12
   166bc:	mov    r12,QWORD PTR [rsp+0x188]
   166c4:	add    r12,r15
   166c7:	mov    QWORD PTR [rsp+0x4e0],r12
   166cf:	add    rdi,r15
   166d2:	mov    QWORD PTR [rsp+0x500],rdi
   166da:	add    rsi,r15
   166dd:	mov    QWORD PTR [rsp+0x520],rsi
   166e5:	add    rdx,r15
   166e8:	mov    QWORD PTR [rsp+0x540],rdx
   166f0:	add    rcx,r15
   166f3:	mov    QWORD PTR [rsp+0x560],rcx
   166fb:	add    rax,r15
   166fe:	mov    QWORD PTR [rsp+0x580],rax
   16706:	add    r13,r15
   16709:	mov    QWORD PTR [rsp+0x5a0],r13
   16711:	add    r11,r15
   16714:	mov    QWORD PTR [rsp+0x5c0],r11
   1671c:	add    r10,r15
   1671f:	mov    QWORD PTR [rsp+0x5e0],r10
   16727:	add    r9,r15
   1672a:	mov    QWORD PTR [rsp+0x600],r9
   16732:	add    r8,r15
   16735:	mov    QWORD PTR [rsp+0x620],r8
   1673d:	mov    r9,QWORD PTR [rsp+0x148]
   16745:	mov    rbp,QWORD PTR [rsp+0x230]
   1674d:	mov    r15,rbp
   16750:	xor    r12d,r12d
   16753:	jmp    16791 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcf1>
   16755:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   16760:	add    r13,rax
   16763:	mov    QWORD PTR [rsp+r12*1+0x240],r13
   1676b:	add    rdx,rax
   1676e:	mov    QWORD PTR [rsp+r12*1+0x248],rdx
   16776:	sub    rdx,r13
   16779:	add    r12,0x20
   1677d:	cmp    rdx,r15
   16780:	cmovb  r15,rdx
   16784:	cmp    r12,0x400
   1678b:	je     16810 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xd70>
   16791:	mov    r13,QWORD PTR [rsp+r12*1+0x240]
   16799:	mov    rdx,QWORD PTR [rsp+r12*1+0x248]
   167a1:	cmp    r13,rdx
   167a4:	jne    16776 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcd6>
   167a6:	mov    rsi,QWORD PTR [rbx+0x58]
   167aa:	mov    rdi,QWORD PTR [rsp+r12*1+0x238]
   167b2:	cmp    rdi,rsi
   167b5:	jae    17a1a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f7a>
   167bb:	mov    rcx,QWORD PTR [rbx+0x50]
   167bf:	lea    rsi,[rdi+rdi*2]
   167c3:	mov    rdx,QWORD PTR [rcx+rsi*8]
   167c7:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   167cc:	sub    r13,rax
   167cf:	mov    QWORD PTR [rcx+rsi*8+0x10],r13
   167d4:	mov    rdi,rdx
   167d7:	sub    rdi,r13
   167da:	cmp    rdx,rdi
   167dd:	jbe    16760 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
   167df:	lea    rdi,[rcx+rsi*8]
   167e3:	mov    QWORD PTR [rsp],rdi
   167e7:	mov    ecx,0x1
   167ec:	mov    r8d,0x1
   167f2:	mov    rsi,r13
   167f5:	mov    rbp,r9
   167f8:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   167fd:	mov    r9,rbp
   16800:	mov    rax,QWORD PTR [rsp]
   16804:	mov    rdx,QWORD PTR [rax]
   16807:	mov    rax,QWORD PTR [rax+0x8]
   1680b:	jmp    16760 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
   16810:	mov    QWORD PTR [rsp+0x148],r9
   16818:	mov    rax,QWORD PTR [rsp+0x240]
   16820:	mov    QWORD PTR [rsp+0x30],rax
   16825:	mov    rax,QWORD PTR [rsp+0x260]
   1682d:	mov    QWORD PTR [rsp+0x40],rax
   16832:	mov    rcx,QWORD PTR [rsp+0x280]
   1683a:	mov    QWORD PTR [rsp+0x10],rcx
   1683f:	mov    r12,QWORD PTR [rsp+0x2a0]
   16847:	mov    rbp,QWORD PTR [rsp+0x2c0]
   1684f:	mov    r13,QWORD PTR [rsp+0x2e0]
   16857:	mov    r11,QWORD PTR [rsp+0x300]
   1685f:	mov    r10,QWORD PTR [rsp+0x320]
   16867:	mov    r9,QWORD PTR [rsp+0x340]
   1686f:	mov    r8,QWORD PTR [rsp+0x360]
   16877:	mov    rdi,QWORD PTR [rsp+0x380]
   1687f:	mov    rsi,QWORD PTR [rsp+0x3a0]
   16887:	mov    rdx,QWORD PTR [rsp+0x3c0]
   1688f:	mov    rcx,QWORD PTR [rsp+0x3e0]
   16897:	mov    rax,QWORD PTR [rsp+0x400]
   1689f:	mov    QWORD PTR [rsp],rax
   168a3:	mov    rax,QWORD PTR [rsp+0x30]
   168a8:	mov    QWORD PTR [rsp+0x48],rax
   168ad:	mov    rax,QWORD PTR [rsp+0x40]
   168b2:	mov    QWORD PTR [rsp+0x50],rax
   168b7:	mov    rax,QWORD PTR [rsp+0x10]
   168bc:	mov    QWORD PTR [rsp+0x58],rax
   168c1:	mov    QWORD PTR [rsp+0x1b8],r12
   168c9:	mov    QWORD PTR [rsp+0x60],r12
   168ce:	mov    QWORD PTR [rsp+0x1c0],rbp
   168d6:	mov    QWORD PTR [rsp+0x68],rbp
   168db:	mov    QWORD PTR [rsp+0x1c8],r13
   168e3:	mov    QWORD PTR [rsp+0x70],r13
   168e8:	mov    QWORD PTR [rsp+0x1d0],r11
   168f0:	mov    QWORD PTR [rsp+0x78],r11
   168f5:	mov    QWORD PTR [rsp+0x1d8],r10
   168fd:	mov    QWORD PTR [rsp+0x80],r10
   16905:	mov    QWORD PTR [rsp+0x1e0],r9
   1690d:	mov    QWORD PTR [rsp+0x88],r9
   16915:	mov    QWORD PTR [rsp+0x1e8],r8
   1691d:	mov    QWORD PTR [rsp+0x90],r8
   16925:	mov    QWORD PTR [rsp+0x1f0],rdi
   1692d:	mov    QWORD PTR [rsp+0x98],rdi
   16935:	mov    QWORD PTR [rsp+0x1f8],rsi
   1693d:	mov    QWORD PTR [rsp+0xa0],rsi
   16945:	mov    QWORD PTR [rsp+0x200],rdx
   1694d:	mov    QWORD PTR [rsp+0xa8],rdx
   16955:	mov    QWORD PTR [rsp+0x208],rcx
   1695d:	mov    QWORD PTR [rsp+0xb0],rcx
   16965:	mov    rax,QWORD PTR [rsp]
   16969:	mov    QWORD PTR [rsp+0xb8],rax
   16971:	mov    rax,QWORD PTR [rsp+0x420]
   16979:	mov    QWORD PTR [rsp+0x190],rax
   16981:	mov    QWORD PTR [rsp+0xc0],rax
   16989:	mov    rax,QWORD PTR [rsp+0x440]
   16991:	mov    QWORD PTR [rsp+0x198],rax
   16999:	mov    QWORD PTR [rsp+0xc8],rax
   169a1:	mov    rax,QWORD PTR [rsp+0x460]
   169a9:	mov    QWORD PTR [rsp+0x1a0],rax
   169b1:	mov    QWORD PTR [rsp+0xd0],rax
   169b9:	mov    rax,QWORD PTR [rsp+0x480]
   169c1:	mov    QWORD PTR [rsp+0x1a8],rax
   169c9:	mov    QWORD PTR [rsp+0xd8],rax
   169d1:	mov    rax,QWORD PTR [rsp+0x4a0]
   169d9:	mov    QWORD PTR [rsp+0x1b0],rax
   169e1:	mov    QWORD PTR [rsp+0xe0],rax
   169e9:	mov    rbp,QWORD PTR [rsp+0x4c0]
   169f1:	mov    QWORD PTR [rsp+0xe8],rbp
   169f9:	mov    r12,QWORD PTR [rsp+0x4e0]
   16a01:	mov    QWORD PTR [rsp+0xf0],r12
   16a09:	mov    rdi,QWORD PTR [rsp+0x500]
   16a11:	mov    QWORD PTR [rsp+0xf8],rdi
   16a19:	mov    rsi,QWORD PTR [rsp+0x520]
   16a21:	mov    QWORD PTR [rsp+0x100],rsi
   16a29:	mov    rdx,QWORD PTR [rsp+0x540]
   16a31:	mov    QWORD PTR [rsp+0x108],rdx
   16a39:	mov    rcx,QWORD PTR [rsp+0x560]
   16a41:	mov    QWORD PTR [rsp+0x110],rcx
   16a49:	mov    rax,QWORD PTR [rsp+0x580]
   16a51:	mov    QWORD PTR [rsp+0x118],rax
   16a59:	mov    r13,QWORD PTR [rsp+0x5a0]
   16a61:	mov    QWORD PTR [rsp+0x120],r13
   16a69:	mov    r11,QWORD PTR [rsp+0x5c0]
   16a71:	mov    QWORD PTR [rsp+0x128],r11
   16a79:	mov    r10,QWORD PTR [rsp+0x5e0]
   16a81:	mov    QWORD PTR [rsp+0x130],r10
   16a89:	mov    r9,QWORD PTR [rsp+0x600]
   16a91:	mov    QWORD PTR [rsp+0x138],r9
   16a99:	mov    r8,QWORD PTR [rsp+0x620]
   16aa1:	mov    QWORD PTR [rsp+0x140],r8
   16aa9:	test   r15,r15
   16aac:	je     16d30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1290>
   16ab2:	xor    eax,eax
   16ab4:	mov    ebp,DWORD PTR [rsp+0x24]
   16ab8:	mov    r13,QWORD PTR [rsp+0x38]
   16abd:	lea    r12,[rsp+0x250]
   16ac5:	jmp    16add <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x103d>
   16ac7:	nop    WORD PTR [rax+rax*1+0x0]
   16ad0:	mov    rax,QWORD PTR [rsp]
   16ad4:	cmp    rax,r15
   16ad7:	je     163c0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x920>
   16add:	lea    rcx,[rax+0x1]
   16ae1:	mov    QWORD PTR [rsp],rcx
   16ae5:	mov    r8,rax
   16ae8:	not    r8
   16aeb:	xor    r9d,r9d
   16aee:	mov    r10,r9
   16af1:	shl    r10,0x5
   16af5:	add    r10,r12
   16af8:	mov    rdx,r9
   16afb:	jmp    16b2b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x108b>
   16afd:	nop    DWORD PTR [rax]
   16b00:	mov    ecx,esi
   16b02:	shr    ecx,0x8
   16b05:	add    ecx,edi
   16b07:	mov    DWORD PTR [r10],ecx
   16b0a:	mov    ecx,ecx
   16b0c:	prefetcht0 BYTE PTR [r14+rcx*2]
   16b11:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
   16b16:	mov    BYTE PTR [rcx+rax*1],sil
   16b1a:	lea    rcx,[r9-0x1]
   16b1e:	add    r10,0x20
   16b22:	mov    rdx,r9
   16b25:	cmp    rcx,0x1f
   16b29:	jae    16ad0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   16b2b:	lea    r9,[rdx+0x1]
   16b2f:	mov    r11d,DWORD PTR [r10]
   16b32:	movzx  edi,WORD PTR [r14+r11*2]
   16b37:	mov    ecx,r11d
   16b3a:	and    ecx,0xffffffe0
   16b3d:	mov    esi,DWORD PTR [r14+rcx*2+0x3c]
   16b42:	cmp    edi,0xfffe
   16b48:	jb     16b00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1060>
   16b4a:	je     16b70 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x10d0>
   16b4c:	cmp    r11d,ebp
   16b4f:	jne    16c30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1190>
   16b55:	mov    ecx,DWORD PTR [r10+0x4]
   16b59:	mov    esi,DWORD PTR [r13+rcx*4+0x0]
   16b5e:	mov    ecx,esi
   16b60:	shr    ecx,0x8
   16b63:	jmp    16b07 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1067>
   16b65:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   16b70:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
   16b75:	add    rcx,rax
   16b78:	mov    QWORD PTR [r10-0x10],rcx
   16b7c:	mov    rsi,QWORD PTR [rbx+0x58]
   16b80:	mov    rdi,QWORD PTR [r10-0x18]
   16b84:	cmp    rdi,rsi
   16b87:	jae    17a41 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fa1>
   16b8d:	mov    rsi,QWORD PTR [rsp+0x18]
   16b92:	mov    rbx,QWORD PTR [rsi+0x50]
   16b96:	mov    rsi,QWORD PTR [rsp+0x18]
   16b9b:	mov    r13,QWORD PTR [rsi+0x68]
   16b9f:	lea    r12,[rdi+rdi*2]
   16ba3:	sub    rcx,QWORD PTR [rbx+r12*8+0x8]
   16ba8:	mov    rsi,QWORD PTR [rsp+0x18]
   16bad:	mov    rsi,QWORD PTR [rsi+0x70]
   16bb1:	mov    QWORD PTR [rbx+r12*8+0x10],rcx
   16bb6:	mov    rbx,QWORD PTR [rsp+0x18]
   16bbb:	mov    rbp,QWORD PTR [rbx+0x40]
   16bbf:	test   rbp,rbp
   16bc2:	mov    QWORD PTR [rsp+0x10],r13
   16bc7:	je     16c61 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11c1>
   16bcd:	mov    rcx,QWORD PTR [rbx+0x38]
   16bd1:	xor    r13d,r13d
   16bd4:	cmp    rbp,0x1
   16bd8:	je     16bfd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x115d>
   16bda:	nop    WORD PTR [rax+rax*1+0x0]
   16be0:	mov    rbx,r13
   16be3:	mov    r12,rbp
   16be6:	shr    r12,1
   16be9:	add    r13,r12
   16bec:	cmp    DWORD PTR [rcx+r13*4],r11d
   16bf0:	cmova  r13,rbx
   16bf4:	sub    rbp,r12
   16bf7:	cmp    rbp,0x1
   16bfb:	ja     16be0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1140>
   16bfd:	cmp    DWORD PTR [rcx+r13*4],r11d
   16c01:	mov    rcx,0xffffffffffffffff
   16c08:	cmovne r13,rcx
   16c0c:	mov    rbx,QWORD PTR [rsp+0x18]
   16c11:	cmp    rdi,rsi
   16c14:	mov    rcx,QWORD PTR [rsp+0x28]
   16c19:	mov    rbp,QWORD PTR [rsp+0x8]
   16c1e:	jb     16c7b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11db>
   16c20:	jmp    17a35 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
   16c25:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   16c30:	shl    esi,0x5
   16c33:	and    esi,0x1ffffe0
   16c39:	and    r11d,0x1f
   16c3d:	or     r11d,esi
   16c40:	mov    DWORD PTR [r10+0x4],r11d
   16c44:	prefetcht0 BYTE PTR [r13+r11*4+0x0]
   16c4a:	mov    DWORD PTR [r10],ebp
   16c4d:	dec    QWORD PTR [rsp+rdx*8+0x48]
   16c52:	cmp    rdx,0x1f
   16c56:	jb     16aee <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
   16c5c:	jmp    16ad0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   16c61:	mov    r13,0xffffffffffffffff
   16c68:	cmp    rdi,rsi
   16c6b:	mov    rcx,QWORD PTR [rsp+0x28]
   16c70:	mov    rbp,QWORD PTR [rsp+0x8]
   16c75:	jae    17a35 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
   16c7b:	mov    rsi,QWORD PTR [rsp+0x10]
   16c80:	mov    QWORD PTR [rsi+rdi*8],r13
   16c84:	cmp    rbp,rcx
   16c87:	je     16d4f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x12af>
   16c8d:	mov    rsi,QWORD PTR [rbx+0x58]
   16c91:	cmp    rbp,rsi
   16c94:	mov    r13,QWORD PTR [rsp+0x38]
   16c99:	jae    17a5c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fbc>
   16c9f:	mov    rsi,QWORD PTR [rbx+0xb0]
   16ca6:	cmp    rbp,rsi
   16ca9:	jae    17a4d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fad>
   16caf:	mov    rcx,QWORD PTR [rbx+0x50]
   16cb3:	mov    rsi,QWORD PTR [rbx+0xa8]
   16cba:	lea    rdi,[rbp*2+0x0]
   16cc2:	add    rdi,rbp
   16cc5:	mov    r11,QWORD PTR [rcx+rdi*8+0x8]
   16cca:	movzx  ebx,BYTE PTR [rsi+rbp*4]
   16cce:	mov    BYTE PTR [r11],bl
   16cd1:	mov    esi,DWORD PTR [rsi+rbp*4]
   16cd4:	shr    esi,0x8
   16cd7:	lea    rbx,[r11+0x1]
   16cdb:	lea    r12,[r11+r8*1]
   16cdf:	inc    r12
   16ce2:	add    r11,QWORD PTR [rcx+rdi*8]
   16ce6:	mov    QWORD PTR [r10-0x18],rbp
   16cea:	mov    QWORD PTR [r10-0x10],rbx
   16cee:	mov    rbx,QWORD PTR [rsp+0x18]
   16cf3:	mov    QWORD PTR [r10-0x8],r11
   16cf7:	mov    DWORD PTR [r10],esi
   16cfa:	mov    DWORD PTR [r10+0x4],0x0
   16d02:	prefetcht0 BYTE PTR [r14+rsi*2]
   16d07:	inc    rbp
   16d0a:	mov    QWORD PTR [rsp+0x8],rbp
   16d0f:	mov    QWORD PTR [rsp+rdx*8+0x48],r12
   16d14:	mov    ebp,DWORD PTR [rsp+0x24]
   16d18:	lea    r12,[rsp+0x250]
   16d20:	cmp    rdx,0x1f
   16d24:	jb     16aee <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
   16d2a:	jmp    16ad0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
   16d2f:	nop
   16d30:	mov    QWORD PTR [rsp+0x188],r12
   16d38:	mov    r12,QWORD PTR [rsp+0x40]
   16d3d:	mov    QWORD PTR [rsp+0x180],rbp
   16d45:	mov    rbp,QWORD PTR [rsp+0x30]
   16d4a:	jmp    16544 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xaa4>
   16d4f:	test   rdx,rdx
   16d52:	je     1712a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x168a>
   16d58:	mov    rcx,QWORD PTR [rsp+0x48]
   16d5d:	lea    rdi,[rax+0x1]
   16d61:	add    rcx,rdi
   16d64:	mov    QWORD PTR [rsp+0x240],rcx
   16d6c:	cmp    rdx,0x2
   16d70:	mov    r9,QWORD PTR [rsp+0x148]
   16d78:	mov    rsi,QWORD PTR [rsp+0x170]
   16d80:	jb     1714a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16aa>
   16d86:	mov    rcx,QWORD PTR [rsp+0x50]
   16d8b:	add    rcx,rdi
   16d8e:	mov    QWORD PTR [rsp+0x260],rcx
   16d96:	cmp    rdx,0x2
   16d9a:	jbe    1715a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ba>
   16da0:	mov    rcx,QWORD PTR [rsp+0x58]
   16da5:	add    rcx,rdi
   16da8:	mov    QWORD PTR [rsp+0x280],rcx
   16db0:	cmp    rdx,0x3
   16db4:	jbe    1716a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ca>
   16dba:	mov    rcx,QWORD PTR [rsp+0x60]
   16dbf:	add    rcx,rdi
   16dc2:	mov    QWORD PTR [rsp+0x2a0],rcx
   16dca:	cmp    rdx,0x4
   16dce:	jbe    1717a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16da>
   16dd4:	mov    rcx,QWORD PTR [rsp+0x68]
   16dd9:	add    rcx,rdi
   16ddc:	mov    QWORD PTR [rsp+0x2c0],rcx
   16de4:	cmp    rdx,0x5
   16de8:	jbe    1718a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ea>
   16dee:	mov    rcx,QWORD PTR [rsp+0x70]
   16df3:	add    rcx,rdi
   16df6:	mov    QWORD PTR [rsp+0x2e0],rcx
   16dfe:	cmp    rdx,0x6
   16e02:	jbe    1719a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16fa>
   16e08:	mov    rcx,QWORD PTR [rsp+0x78]
   16e0d:	add    rcx,rdi
   16e10:	mov    QWORD PTR [rsp+0x300],rcx
   16e18:	cmp    rdx,0x7
   16e1c:	jbe    171ad <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x170d>
   16e22:	mov    rcx,QWORD PTR [rsp+0x80]
   16e2a:	add    rcx,rdi
   16e2d:	mov    QWORD PTR [rsp+0x320],rcx
   16e35:	cmp    rdx,0x8
   16e39:	jbe    171c0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1720>
   16e3f:	mov    rcx,QWORD PTR [rsp+0x88]
   16e47:	add    rcx,rdi
   16e4a:	mov    QWORD PTR [rsp+0x340],rcx
   16e52:	cmp    rdx,0x9
   16e56:	jbe    171d3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1733>
   16e5c:	mov    rcx,QWORD PTR [rsp+0x90]
   16e64:	add    rcx,rdi
   16e67:	mov    QWORD PTR [rsp+0x360],rcx
   16e6f:	cmp    rdx,0xa
   16e73:	jbe    171e6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1746>
   16e79:	mov    rcx,QWORD PTR [rsp+0x98]
   16e81:	add    rcx,rdi
   16e84:	mov    QWORD PTR [rsp+0x380],rcx
   16e8c:	cmp    rdx,0xb
   16e90:	jbe    171f9 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1759>
   16e96:	mov    rcx,QWORD PTR [rsp+0xa0]
   16e9e:	add    rcx,rdi
   16ea1:	mov    QWORD PTR [rsp+0x3a0],rcx
   16ea9:	cmp    rdx,0xc
   16ead:	jbe    1720c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x176c>
   16eb3:	mov    rcx,QWORD PTR [rsp+0xa8]
   16ebb:	add    rcx,rdi
   16ebe:	mov    QWORD PTR [rsp+0x3c0],rcx
   16ec6:	cmp    rdx,0xd
   16eca:	jbe    1721f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x177f>
   16ed0:	mov    rcx,QWORD PTR [rsp+0xb0]
   16ed8:	lea    rdi,[rax+0x1]
   16edc:	add    rcx,rdi
   16edf:	mov    QWORD PTR [rsp+0x3e0],rcx
   16ee7:	cmp    rdx,0xe
   16eeb:	jbe    17232 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1792>
   16ef1:	mov    rcx,QWORD PTR [rsp+0xb8]
   16ef9:	lea    rdi,[rax+0x1]
   16efd:	add    rcx,rdi
   16f00:	mov    QWORD PTR [rsp+0x400],rcx
   16f08:	cmp    rdx,0xf
   16f0c:	jbe    17245 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17a5>
   16f12:	mov    rcx,QWORD PTR [rsp+0xc0]
   16f1a:	lea    rdi,[rax+0x1]
   16f1e:	add    rcx,rdi
   16f21:	mov    QWORD PTR [rsp+0x420],rcx
   16f29:	cmp    rdx,0x10
   16f2d:	jbe    17258 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17b8>
   16f33:	mov    rcx,QWORD PTR [rsp+0xc8]
   16f3b:	lea    rdi,[rax+0x1]
   16f3f:	add    rcx,rdi
   16f42:	mov    QWORD PTR [rsp+0x440],rcx
   16f4a:	cmp    rdx,0x11
   16f4e:	jbe    1726b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17cb>
   16f54:	mov    rcx,QWORD PTR [rsp+0xd0]
   16f5c:	lea    rdi,[rax+0x1]
   16f60:	add    rcx,rdi
   16f63:	mov    QWORD PTR [rsp+0x460],rcx
   16f6b:	cmp    rdx,0x12
   16f6f:	jbe    1727e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17de>
   16f75:	mov    rcx,QWORD PTR [rsp+0xd8]
   16f7d:	lea    rdi,[rax+0x1]
   16f81:	add    rcx,rdi
   16f84:	mov    QWORD PTR [rsp+0x480],rcx
   16f8c:	cmp    rdx,0x13
   16f90:	jbe    17291 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17f1>
   16f96:	mov    rcx,QWORD PTR [rsp+0xe0]
   16f9e:	lea    rdi,[rax+0x1]
   16fa2:	add    rcx,rdi
   16fa5:	mov    QWORD PTR [rsp+0x4a0],rcx
   16fad:	cmp    rdx,0x14
   16fb1:	jbe    172a4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1804>
   16fb7:	mov    rcx,QWORD PTR [rsp+0xe8]
   16fbf:	lea    rdi,[rax+0x1]
   16fc3:	add    rcx,rdi
   16fc6:	mov    QWORD PTR [rsp+0x4c0],rcx
   16fce:	cmp    rdx,0x15
   16fd2:	jbe    172b7 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1817>
   16fd8:	mov    rcx,QWORD PTR [rsp+0xf0]
   16fe0:	lea    rdi,[rax+0x1]
   16fe4:	add    rcx,rdi
   16fe7:	mov    QWORD PTR [rsp+0x4e0],rcx
   16fef:	cmp    rdx,0x16
   16ff3:	jbe    172ca <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x182a>
   16ff9:	mov    rcx,QWORD PTR [rsp+0xf8]
   17001:	lea    rdi,[rax+0x1]
   17005:	add    rcx,rdi
   17008:	mov    QWORD PTR [rsp+0x500],rcx
   17010:	cmp    rdx,0x17
   17014:	jbe    172dd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x183d>
   1701a:	mov    rcx,QWORD PTR [rsp+0x100]
   17022:	lea    rdi,[rax+0x1]
   17026:	add    rcx,rdi
   17029:	mov    QWORD PTR [rsp+0x520],rcx
   17031:	cmp    rdx,0x18
   17035:	jbe    172f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1850>
   1703b:	mov    rcx,QWORD PTR [rsp+0x108]
   17043:	lea    rdi,[rax+0x1]
   17047:	add    rcx,rdi
   1704a:	mov    QWORD PTR [rsp+0x540],rcx
   17052:	cmp    rdx,0x19
   17056:	jbe    17303 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1863>
   1705c:	mov    rcx,QWORD PTR [rsp+0x110]
   17064:	lea    rdi,[rax+0x1]
   17068:	add    rcx,rdi
   1706b:	mov    QWORD PTR [rsp+0x560],rcx
   17073:	cmp    rdx,0x1a
   17077:	jbe    17316 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1876>
   1707d:	mov    rcx,QWORD PTR [rsp+0x118]
   17085:	lea    rdi,[rax+0x1]
   17089:	add    rcx,rdi
   1708c:	mov    QWORD PTR [rsp+0x580],rcx
   17094:	cmp    rdx,0x1b
   17098:	jbe    17329 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1889>
   1709e:	mov    rcx,QWORD PTR [rsp+0x120]
   170a6:	lea    rdi,[rax+0x1]
   170aa:	add    rcx,rdi
   170ad:	mov    QWORD PTR [rsp+0x5a0],rcx
   170b5:	cmp    rdx,0x1c
   170b9:	jbe    1733c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x189c>
   170bf:	mov    rcx,QWORD PTR [rsp+0x128]
   170c7:	lea    rdi,[rax+0x1]
   170cb:	add    rcx,rdi
   170ce:	mov    QWORD PTR [rsp+0x5c0],rcx
   170d6:	cmp    rdx,0x1d
   170da:	jbe    1734f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18af>
   170e0:	mov    rcx,QWORD PTR [rsp+0x130]
   170e8:	lea    rdi,[rax+0x1]
   170ec:	add    rcx,rdi
   170ef:	mov    QWORD PTR [rsp+0x5e0],rcx
   170f7:	cmp    rdx,0x1e
   170fb:	jbe    17362 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
   17101:	mov    rax,QWORD PTR [rsp+0x138]
   17109:	mov    rcx,QWORD PTR [rsp]
   1710d:	add    rax,rcx
   17110:	mov    QWORD PTR [rsp+0x600],rax
   17118:	mov    rax,rcx
   1711b:	cmp    rdx,0x20
   1711f:	jae    17362 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
   17125:	jmp    17372 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
   1712a:	mov    rcx,QWORD PTR [rsp+0x50]
   1712f:	add    rcx,rax
   17132:	mov    QWORD PTR [rsp+0x260],rcx
   1713a:	mov    r9,QWORD PTR [rsp+0x148]
   17142:	mov    rsi,QWORD PTR [rsp+0x170]
   1714a:	mov    rcx,QWORD PTR [rsp+0x58]
   1714f:	add    rcx,rax
   17152:	mov    QWORD PTR [rsp+0x280],rcx
   1715a:	mov    rcx,QWORD PTR [rsp+0x60]
   1715f:	add    rcx,rax
   17162:	mov    QWORD PTR [rsp+0x2a0],rcx
   1716a:	mov    rcx,QWORD PTR [rsp+0x68]
   1716f:	add    rcx,rax
   17172:	mov    QWORD PTR [rsp+0x2c0],rcx
   1717a:	mov    rcx,QWORD PTR [rsp+0x70]
   1717f:	add    rcx,rax
   17182:	mov    QWORD PTR [rsp+0x2e0],rcx
   1718a:	mov    rcx,QWORD PTR [rsp+0x78]
   1718f:	add    rcx,rax
   17192:	mov    QWORD PTR [rsp+0x300],rcx
   1719a:	mov    rcx,QWORD PTR [rsp+0x80]
   171a2:	add    rcx,rax
   171a5:	mov    QWORD PTR [rsp+0x320],rcx
   171ad:	mov    rcx,QWORD PTR [rsp+0x88]
   171b5:	add    rcx,rax
   171b8:	mov    QWORD PTR [rsp+0x340],rcx
   171c0:	mov    rcx,QWORD PTR [rsp+0x90]
   171c8:	add    rcx,rax
   171cb:	mov    QWORD PTR [rsp+0x360],rcx
   171d3:	mov    rcx,QWORD PTR [rsp+0x98]
   171db:	add    rcx,rax
   171de:	mov    QWORD PTR [rsp+0x380],rcx
   171e6:	mov    rcx,QWORD PTR [rsp+0xa0]
   171ee:	add    rcx,rax
   171f1:	mov    QWORD PTR [rsp+0x3a0],rcx
   171f9:	mov    rcx,QWORD PTR [rsp+0xa8]
   17201:	add    rcx,rax
   17204:	mov    QWORD PTR [rsp+0x3c0],rcx
   1720c:	mov    rcx,QWORD PTR [rsp+0xb0]
   17214:	add    rcx,rax
   17217:	mov    QWORD PTR [rsp+0x3e0],rcx
   1721f:	mov    rcx,QWORD PTR [rsp+0xb8]
   17227:	add    rcx,rax
   1722a:	mov    QWORD PTR [rsp+0x400],rcx
   17232:	mov    rcx,QWORD PTR [rsp+0xc0]
   1723a:	add    rcx,rax
   1723d:	mov    QWORD PTR [rsp+0x420],rcx
   17245:	mov    rcx,QWORD PTR [rsp+0xc8]
   1724d:	add    rcx,rax
   17250:	mov    QWORD PTR [rsp+0x440],rcx
   17258:	mov    rcx,QWORD PTR [rsp+0xd0]
   17260:	add    rcx,rax
   17263:	mov    QWORD PTR [rsp+0x460],rcx
   1726b:	mov    rcx,QWORD PTR [rsp+0xd8]
   17273:	add    rcx,rax
   17276:	mov    QWORD PTR [rsp+0x480],rcx
   1727e:	mov    rcx,QWORD PTR [rsp+0xe0]
   17286:	add    rcx,rax
   17289:	mov    QWORD PTR [rsp+0x4a0],rcx
   17291:	mov    rcx,QWORD PTR [rsp+0xe8]
   17299:	add    rcx,rax
   1729c:	mov    QWORD PTR [rsp+0x4c0],rcx
   172a4:	mov    rcx,QWORD PTR [rsp+0xf0]
   172ac:	add    rcx,rax
   172af:	mov    QWORD PTR [rsp+0x4e0],rcx
   172b7:	mov    rcx,QWORD PTR [rsp+0xf8]
   172bf:	add    rcx,rax
   172c2:	mov    QWORD PTR [rsp+0x500],rcx
   172ca:	mov    rcx,QWORD PTR [rsp+0x100]
   172d2:	add    rcx,rax
   172d5:	mov    QWORD PTR [rsp+0x520],rcx
   172dd:	mov    rcx,QWORD PTR [rsp+0x108]
   172e5:	add    rcx,rax
   172e8:	mov    QWORD PTR [rsp+0x540],rcx
   172f0:	mov    rcx,QWORD PTR [rsp+0x110]
   172f8:	add    rcx,rax
   172fb:	mov    QWORD PTR [rsp+0x560],rcx
   17303:	mov    rcx,QWORD PTR [rsp+0x118]
   1730b:	add    rcx,rax
   1730e:	mov    QWORD PTR [rsp+0x580],rcx
   17316:	mov    rcx,QWORD PTR [rsp+0x120]
   1731e:	add    rcx,rax
   17321:	mov    QWORD PTR [rsp+0x5a0],rcx
   17329:	mov    rcx,QWORD PTR [rsp+0x128]
   17331:	add    rcx,rax
   17334:	mov    QWORD PTR [rsp+0x5c0],rcx
   1733c:	mov    rcx,QWORD PTR [rsp+0x130]
   17344:	add    rcx,rax
   17347:	mov    QWORD PTR [rsp+0x5e0],rcx
   1734f:	mov    rcx,QWORD PTR [rsp+0x138]
   17357:	add    rcx,rax
   1735a:	mov    QWORD PTR [rsp+0x600],rcx
   17362:	add    rax,QWORD PTR [rsp+0x140]
   1736a:	mov    QWORD PTR [rsp+0x620],rax
   17372:	shl    rsi,0x5
   17376:	lea    rax,[rsp+rsi*1]
   1737a:	add    rax,0x238
   17380:	mov    QWORD PTR [rsp],rax
   17384:	mov    rax,QWORD PTR [rbx+0x68]
   17388:	mov    rsi,QWORD PTR [rbx+0x70]
   1738c:	lea    r12,[rsp+0x238]
   17394:	jmp    173ad <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x190d>
   17396:	cs nop WORD PTR [rax+rax*1+0x0]
   173a0:	mov    r12,r13
   173a3:	add    r9,0xffffffffffffffe0
   173a7:	je     174eb <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a4b>
   173ad:	lea    r13,[r12+0x20]
   173b2:	mov    r15,QWORD PTR [r12]
   173b6:	cmp    r15,0xffffffffffffffff
   173ba:	je     173a0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
   173bc:	cmp    r15,rsi
   173bf:	jae    17a91 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ff1>
   173c5:	cmp    QWORD PTR [rax+r15*8],0xffffffffffffffff
   173ca:	jne    173a0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
   173cc:	mov    edi,0x80
   173d1:	call   QWORD PTR [rip+0x67f91]        # 7f368 <malloc@GLIBC_2.2.5>
   173d7:	test   rax,rax
   173da:	je     17af5 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2055>
   173e0:	mov    rcx,QWORD PTR [r12+0x18]
   173e5:	mov    QWORD PTR [rax+0x18],rcx
   173e9:	movups xmm0,XMMWORD PTR [r12+0x8]
   173ef:	movups XMMWORD PTR [rax+0x8],xmm0
   173f3:	mov    QWORD PTR [rax],r15
   173f6:	mov    QWORD PTR [rsp+0x158],0x4
   17402:	mov    QWORD PTR [rsp+0x160],rax
   1740a:	mov    QWORD PTR [rsp+0x168],0x1
   17416:	cmp    r13,QWORD PTR [rsp]
   1741a:	je     174fd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a5d>
   17420:	mov    r12d,0x1
   17426:	lea    r15,[rsp+0x158]
   1742e:	mov    rcx,QWORD PTR [rbx+0x68]
   17432:	mov    rsi,QWORD PTR [rbx+0x70]
   17436:	mov    rdx,r13
   17439:	jmp    1744d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19ad>
   1743b:	nop    DWORD PTR [rax+rax*1+0x0]
   17440:	mov    rdx,r13
   17443:	cmp    r13,QWORD PTR [rsp]
   17447:	je     17503 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
   1744d:	lea    r13,[rdx+0x20]
   17451:	mov    rbp,QWORD PTR [rdx]
   17454:	cmp    rbp,0xffffffffffffffff
   17458:	je     17440 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
   1745a:	cmp    rbp,rsi
   1745d:	jae    17aa0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2000>
   17463:	cmp    QWORD PTR [rcx+rbp*8],0xffffffffffffffff
   17468:	jne    17440 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
   1746a:	mov    rcx,QWORD PTR [rdx+0x18]
   1746e:	mov    QWORD PTR [rsp+0x220],rcx
   17476:	movups xmm0,XMMWORD PTR [rdx+0x8]
   1747a:	movaps XMMWORD PTR [rsp+0x210],xmm0
   17482:	cmp    r12,QWORD PTR [rsp+0x158]
   1748a:	jne    174af <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a0f>
   1748c:	mov    edx,0x1
   17491:	mov    ecx,0x8
   17496:	mov    r8d,0x20
   1749c:	mov    rdi,r15
   1749f:	mov    rsi,r12
   174a2:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   174a7:	mov    rax,QWORD PTR [rsp+0x160]
   174af:	mov    rcx,r12
   174b2:	shl    rcx,0x5
   174b6:	mov    QWORD PTR [rax+rcx*1],rbp
   174ba:	movaps xmm0,XMMWORD PTR [rsp+0x210]
   174c2:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
   174c7:	mov    rdx,QWORD PTR [rsp+0x220]
   174cf:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
   174d4:	inc    r12
   174d7:	mov    QWORD PTR [rsp+0x168],r12
   174df:	cmp    r13,QWORD PTR [rsp]
   174e3:	jne    1742e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x198e>
   174e9:	jmp    17503 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
   174eb:	mov    eax,0x8
   174f0:	mov    QWORD PTR [rsp+0x8],rax
   174f5:	xor    r14d,r14d
   174f8:	jmp    17811 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d71>
   174fd:	mov    r12d,0x1
   17503:	mov    rax,QWORD PTR [rsp+0x158]
   1750b:	mov    QWORD PTR [rsp+0x10],rax
   17510:	mov    rax,QWORD PTR [rsp+0x160]
   17518:	mov    QWORD PTR [rsp+0x8],rax
   1751d:	mov    r15,r12
   17520:	shl    r15,0x5
   17524:	xor    r13d,r13d
   17527:	lea    rax,[rip+0x657a2]        # 7ccd0 <__frame_dummy_init_array_entry+0x290>
   1752e:	mov    QWORD PTR [rsp+0x30],rax
   17533:	jmp    1755e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1abe>
   17535:	data16 cs nop WORD PTR [rax+rax*1+0x0]
   17540:	add    rbp,rax
   17543:	mov    rcx,QWORD PTR [rsp+0x8]
   17548:	mov    QWORD PTR [rcx+r13*1+0x8],rbp
   1754d:	add    rdx,rax
   17550:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
   17555:	add    r13,0x20
   17559:	cmp    r15,r13
   1755c:	je     175d4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b34>
   1755e:	mov    rax,QWORD PTR [rsp+0x8]
   17563:	mov    rbp,QWORD PTR [rax+r13*1+0x8]
   17568:	cmp    rbp,QWORD PTR [rax+r13*1+0x10]
   1756d:	jne    17555 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ab5>
   1756f:	mov    rsi,QWORD PTR [rbx+0x58]
   17573:	mov    rax,QWORD PTR [rsp+0x8]
   17578:	mov    rdi,QWORD PTR [rax+r13*1]
   1757c:	cmp    rdi,rsi
   1757f:	jae    17a80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
   17585:	mov    rcx,QWORD PTR [rbx+0x50]
   17589:	lea    rsi,[rdi+rdi*2]
   1758d:	mov    rdx,QWORD PTR [rcx+rsi*8]
   17591:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   17596:	sub    rbp,rax
   17599:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   1759e:	mov    rdi,rdx
   175a1:	sub    rdi,rbp
   175a4:	cmp    rdx,rdi
   175a7:	jbe    17540 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
   175a9:	lea    rdi,[rcx+rsi*8]
   175ad:	mov    ecx,0x1
   175b2:	mov    r8d,0x1
   175b8:	mov    QWORD PTR [rsp],rdi
   175bc:	mov    rsi,rbp
   175bf:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   175c4:	mov    rax,QWORD PTR [rsp]
   175c8:	mov    rdx,QWORD PTR [rax]
   175cb:	mov    rax,QWORD PTR [rax+0x8]
   175cf:	jmp    17540 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
   175d4:	mov    rax,QWORD PTR [rsp+0x8]
   175d9:	add    rax,0x18
   175dd:	mov    QWORD PTR [rsp],rax
   175e1:	xor    r15d,r15d
   175e4:	jmp    175f3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   175e6:	cs nop WORD PTR [rax+rax*1+0x0]
   175f0:	xor    r15d,r15d
   175f3:	mov    r13,r15
   175f6:	shl    r13,0x5
   175fa:	add    r13,QWORD PTR [rsp]
   175fe:	jmp    1761a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b7a>
   17600:	add    rbp,rax
   17603:	mov    QWORD PTR [r13-0x10],rbp
   17607:	add    rdx,rax
   1760a:	mov    QWORD PTR [r13-0x8],rdx
   1760e:	inc    r15
   17611:	add    r13,0x20
   17615:	cmp    r15,r12
   17618:	jae    175f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b50>
   1761a:	lea    rax,[r13-0x18]
   1761e:	mov    ecx,DWORD PTR [r13+0x0]
   17622:	movzx  esi,WORD PTR [r14+rcx*2]
   17627:	mov    edx,ecx
   17629:	and    edx,0xffffffe0
   1762c:	mov    edx,DWORD PTR [r14+rdx*2+0x3c]
   17631:	cmp    esi,0xfffe
   17637:	jae    17650 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bb0>
   17639:	mov    ecx,edx
   1763b:	shr    ecx,0x8
   1763e:	add    ecx,esi
   17640:	jmp    17671 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bd1>
   17642:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   17650:	je     17700 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1c60>
   17656:	cmp    ecx,DWORD PTR [rsp+0x24]
   1765a:	jne    17790 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cf0>
   17660:	mov    ecx,DWORD PTR [r13+0x4]
   17664:	mov    rdx,QWORD PTR [rsp+0x38]
   17669:	mov    edx,DWORD PTR [rdx+rcx*4]
   1766c:	mov    ecx,edx
   1766e:	shr    ecx,0x8
   17671:	mov    DWORD PTR [r13+0x0],ecx
   17675:	mov    ecx,ecx
   17677:	prefetcht0 BYTE PTR [r14+rcx*2]
   1767c:	mov    rcx,QWORD PTR [r13-0x10]
   17680:	mov    BYTE PTR [rcx],dl
   17682:	mov    rbp,QWORD PTR [r13-0x10]
   17686:	inc    rbp
   17689:	mov    QWORD PTR [r13-0x10],rbp
   1768d:	cmp    rbp,QWORD PTR [r13-0x8]
   17691:	jne    1760e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b6e>
   17697:	mov    rsi,QWORD PTR [rbx+0x58]
   1769b:	mov    rdi,QWORD PTR [rax]
   1769e:	cmp    rdi,rsi
   176a1:	jae    17a80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
   176a7:	mov    rcx,QWORD PTR [rbx+0x50]
   176ab:	lea    rsi,[rdi+rdi*2]
   176af:	mov    rdx,QWORD PTR [rcx+rsi*8]
   176b3:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
   176b8:	sub    rbp,rax
   176bb:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
   176c0:	mov    rdi,rdx
   176c3:	sub    rdi,rbp
   176c6:	cmp    rdx,rdi
   176c9:	jbe    17600 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
   176cf:	lea    rdi,[rcx+rsi*8]
   176d3:	mov    ecx,0x1
   176d8:	mov    r8d,0x1
   176de:	mov    QWORD PTR [rsp+0x40],rdi
   176e3:	mov    rsi,rbp
   176e6:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   176eb:	mov    rax,QWORD PTR [rsp+0x40]
   176f0:	mov    rdx,QWORD PTR [rax]
   176f3:	mov    rax,QWORD PTR [rax+0x8]
   176f7:	jmp    17600 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
   176fc:	nop    DWORD PTR [rax+0x0]
   17700:	mov    rsi,QWORD PTR [rbx+0x58]
   17704:	mov    rdi,QWORD PTR [rax]
   17707:	cmp    rdi,rsi
   1770a:	jae    17a74 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fd4>
   17710:	mov    r8,QWORD PTR [rbx+0x50]
   17714:	mov    rdx,QWORD PTR [rbx+0x68]
   17718:	lea    r9,[rdi+rdi*2]
   1771c:	mov    r10,QWORD PTR [rax+0x8]
   17720:	sub    r10,QWORD PTR [r8+r9*8+0x8]
   17725:	mov    rsi,QWORD PTR [rbx+0x70]
   17729:	mov    QWORD PTR [r8+r9*8+0x10],r10
   1772e:	mov    r10,QWORD PTR [rbx+0x40]
   17732:	test   r10,r10
   17735:	je     177c1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d21>
   1773b:	mov    r9,QWORD PTR [rbx+0x38]
   1773f:	xor    r8d,r8d
   17742:	cmp    r10,0x1
   17746:	je     1776d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ccd>
   17748:	nop    DWORD PTR [rax+rax*1+0x0]
   17750:	mov    r11,r8
   17753:	mov    rbx,r10
   17756:	shr    rbx,1
   17759:	add    r8,rbx
   1775c:	cmp    DWORD PTR [r9+r8*4],ecx
   17760:	cmova  r8,r11
   17764:	sub    r10,rbx
   17767:	cmp    r10,0x1
   1776b:	ja     17750 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cb0>
   1776d:	cmp    DWORD PTR [r9+r8*4],ecx
   17771:	mov    rcx,0xffffffffffffffff
   17778:	cmovne r8,rcx
   1777c:	mov    rbx,QWORD PTR [rsp+0x18]
   17781:	cmp    rdi,rsi
   17784:	jb     177d1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d31>
   17786:	jmp    17a6b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
   1778b:	nop    DWORD PTR [rax+rax*1+0x0]
   17790:	shl    edx,0x5
   17793:	and    edx,0x1ffffe0
   17799:	and    ecx,0x1f
   1779c:	or     ecx,edx
   1779e:	mov    DWORD PTR [r13+0x4],ecx
   177a2:	mov    rax,QWORD PTR [rsp+0x38]
   177a7:	prefetcht0 BYTE PTR [rax+rcx*4]
   177ab:	mov    eax,DWORD PTR [rsp+0x24]
   177af:	mov    DWORD PTR [r13+0x0],eax
   177b3:	inc    r15
   177b6:	cmp    r15,r12
   177b9:	jb     175f3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   177bf:	jmp    177fd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d5d>
   177c1:	mov    r8,0xffffffffffffffff
   177c8:	cmp    rdi,rsi
   177cb:	jae    17a6b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
   177d1:	mov    QWORD PTR [rdx+rdi*8],r8
   177d5:	dec    r12
   177d8:	mov    rcx,r12
   177db:	shl    rcx,0x5
   177df:	mov    rdx,QWORD PTR [rsp+0x8]
   177e4:	movups xmm0,XMMWORD PTR [rdx+rcx*1]
   177e8:	movups xmm1,XMMWORD PTR [rdx+rcx*1+0x10]
   177ed:	movups XMMWORD PTR [rax+0x10],xmm1
   177f1:	movups XMMWORD PTR [rax],xmm0
   177f4:	cmp    r15,r12
   177f7:	jb     175f3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   177fd:	mov    r15d,0x0
   17803:	test   r12,r12
   17806:	jne    175f3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
   1780c:	mov    r14,QWORD PTR [rsp+0x10]
   17811:	xor    eax,eax
   17813:	lea    rcx,[rip+0x6525e]        # 7ca78 <__frame_dummy_init_array_entry+0x38>
   1781a:	lea    r8,[rip+0x6526f]        # 7ca90 <__frame_dummy_init_array_entry+0x50>
   17821:	lea    r9,[rip+0x65280]        # 7caa8 <__frame_dummy_init_array_entry+0x68>
   17828:	lea    rdx,[rip+0x65291]        # 7cac0 <__frame_dummy_init_array_entry+0x80>
   1782f:	mov    r11,QWORD PTR [rsp+0x28]
   17834:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   17840:	mov    rsi,QWORD PTR [rbx+0x40]
   17844:	cmp    rax,rsi
   17847:	jae    17a0a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f6a>
   1784d:	mov    rsi,QWORD PTR [rbx+0x98]
   17854:	cmp    rax,rsi
   17857:	jae    17a02 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f62>
   1785d:	mov    rsi,QWORD PTR [rbx+0x38]
   17861:	mov    edi,DWORD PTR [rsi+rax*4]
   17864:	mov    rsi,QWORD PTR [rbx+0x10]
   17868:	mov    r10d,edi
   1786b:	shr    r10d,0x5
   1786f:	cmp    rsi,r10
   17872:	jbe    179fa <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f5a>
   17878:	and    edi,0x1f
   1787b:	cmp    edi,0x1d
   1787e:	ja     179f3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f53>
   17884:	mov    rsi,QWORD PTR [rbx+0x90]
   1788b:	movzx  esi,WORD PTR [rsi+rax*2]
   1788f:	inc    rax
   17892:	shl    r10,0x6
   17896:	add    r10,QWORD PTR [rbx+0x8]
   1789a:	mov    WORD PTR [r10+rdi*2],si
   1789f:	cmp    r11,rax
   178a2:	jne    17840 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1da0>
   178a4:	mov    rax,QWORD PTR [rbx+0x38]
   178a8:	mov    rcx,QWORD PTR [rbx+0x40]
   178ac:	mov    edi,DWORD PTR [rsp+0x154]
   178b3:	mov    esi,edi
   178b5:	mov    edx,0x88888889
   178ba:	imul   rdx,rsi
   178be:	shr    rdx,0x23
   178c2:	and    edx,0xfffffffe
   178c5:	add    edx,edi
   178c7:	cmp    rcx,0x1
   178cb:	je     178fe <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e5e>
   178cd:	test   rcx,rcx
   178d0:	je     17ac4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2024>
   178d6:	xor    esi,esi
   178d8:	nop    DWORD PTR [rax+rax*1+0x0]
   178e0:	mov    rdi,rsi
   178e3:	mov    r8,rcx
   178e6:	shr    r8,1
   178e9:	add    rsi,r8
   178ec:	cmp    DWORD PTR [rax+rsi*4],edx
   178ef:	cmova  rsi,rdi
   178f3:	sub    rcx,r8
   178f6:	cmp    rcx,0x1
   178fa:	ja     178e0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e40>
   178fc:	jmp    17900 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e60>
   178fe:	xor    esi,esi
   17900:	cmp    DWORD PTR [rax+rsi*4],edx
   17903:	jne    17abd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x201d>
   17909:	mov    QWORD PTR [rbx+0x80],rsi
   17910:	mov    rax,QWORD PTR [rsp+0x178]
   17918:	mov    QWORD PTR [rbx+0x78],rax
   1791c:	test   r14,r14
   1791f:	je     1792c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e8c>
   17921:	mov    rdi,QWORD PTR [rsp+0x8]
   17926:	call   QWORD PTR [rip+0x67a44]        # 7f370 <free@GLIBC_2.2.5>
   1792c:	add    rsp,0x638
   17933:	pop    rbx
   17934:	pop    r12
   17936:	pop    r13
   17938:	pop    r14
   1793a:	pop    r15
   1793c:	pop    rbp
   1793d:	ret
   1793e:	lea    rcx,[rip+0x65193]        # 7cad8 <__frame_dummy_init_array_entry+0x98>
   17945:	xor    edi,edi
   17947:	call   30f40 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
   1794c:	lea    rdi,[rbx+0x60]
   17950:	mov    ecx,0x8
   17955:	mov    r8d,0x8
   1795b:	xor    esi,esi
   1795d:	mov    rdx,r12
   17960:	call   1c880 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
   17965:	mov    r15,QWORD PTR [rbx+0x70]
   17969:	jmp    15ea6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x406>
   1796e:	lea    rdx,[rip+0x651db]        # 7cb50 <__frame_dummy_init_array_entry+0x110>
   17975:	mov    rdi,r12
   17978:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   1797d:	lea    rdx,[rip+0x6576c]        # 7d0f0 <__frame_dummy_init_array_entry+0x6b0>
   17984:	mov    esi,0x1e
   17989:	mov    rdi,rbp
   1798c:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17991:	lea    rdx,[rip+0x65740]        # 7d0d8 <__frame_dummy_init_array_entry+0x698>
   17998:	mov    rdi,r12
   1799b:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179a0:	lea    rdx,[rip+0x65191]        # 7cb38 <__frame_dummy_init_array_entry+0xf8>
   179a7:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179ac:	lea    rdx,[rip+0x6516d]        # 7cb20 <__frame_dummy_init_array_entry+0xe0>
   179b3:	mov    esi,0x1e
   179b8:	mov    rdi,rax
   179bb:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179c0:	lea    rdx,[rip+0x65141]        # 7cb08 <__frame_dummy_init_array_entry+0xc8>
   179c7:	mov    rdi,rcx
   179ca:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179cf:	lea    rdx,[rip+0x6511a]        # 7caf0 <__frame_dummy_init_array_entry+0xb0>
   179d6:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179db:	lea    rdx,[rip+0x652a6]        # 7cc88 <__frame_dummy_init_array_entry+0x248>
   179e2:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179e7:	lea    rdx,[rip+0x65282]        # 7cc70 <__frame_dummy_init_array_entry+0x230>
   179ee:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   179f3:	mov    esi,0x1e
   179f8:	jmp    17a10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   179fa:	mov    rdi,r10
   179fd:	mov    rdx,r9
   17a00:	jmp    17a10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   17a02:	mov    rdi,rax
   17a05:	mov    rdx,r8
   17a08:	jmp    17a10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
   17a0a:	mov    rdi,rax
   17a0d:	mov    rdx,rcx
   17a10:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a15:	jmp    17af3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   17a1a:	lea    rdx,[rip+0x652af]        # 7ccd0 <__frame_dummy_init_array_entry+0x290>
   17a21:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a26:	lea    rdx,[rip+0x656db]        # 7d108 <__frame_dummy_init_array_entry+0x6c8>
   17a2d:	mov    rdi,r13
   17a30:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a35:	lea    rdx,[rip+0x6527c]        # 7ccb8 <__frame_dummy_init_array_entry+0x278>
   17a3c:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a41:	lea    rdx,[rip+0x65258]        # 7cca0 <__frame_dummy_init_array_entry+0x260>
   17a48:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a4d:	lea    rdx,[rip+0x65234]        # 7cc88 <__frame_dummy_init_array_entry+0x248>
   17a54:	mov    rdi,rbp
   17a57:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a5c:	lea    rdx,[rip+0x6520d]        # 7cc70 <__frame_dummy_init_array_entry+0x230>
   17a63:	mov    rdi,rbp
   17a66:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a6b:	lea    rax,[rip+0x65246]        # 7ccb8 <__frame_dummy_init_array_entry+0x278>
   17a72:	jmp    17a7b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fdb>
   17a74:	lea    rax,[rip+0x65225]        # 7cca0 <__frame_dummy_init_array_entry+0x260>
   17a7b:	mov    QWORD PTR [rsp+0x30],rax
   17a80:	mov    rdx,QWORD PTR [rsp+0x30]
   17a85:	mov    r14,QWORD PTR [rsp+0x10]
   17a8a:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17a8f:	jmp    17af3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   17a91:	lea    rdx,[rip+0x65250]        # 7cce8 <__frame_dummy_init_array_entry+0x2a8>
   17a98:	mov    rdi,r15
   17a9b:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17aa0:	lea    rdx,[rip+0x65241]        # 7cce8 <__frame_dummy_init_array_entry+0x2a8>
   17aa7:	mov    rdi,rbp
   17aaa:	call   30423 <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
   17aaf:	jmp    17af3 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
   17ab1:	lea    rdi,[rip+0x64f90]        # 7ca48 <__frame_dummy_init_array_entry+0x8>
   17ab8:	call   324a0 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
   17abd:	adc    rsi,0x0
   17ac1:	mov    rcx,rsi
   17ac4:	mov    QWORD PTR [rsp+0x210],rcx
   17acc:	lea    rdi,[rip+0xffffffffffff27fa]        # a2cd <GCC_except_table566+0x415d>
   17ad3:	lea    rcx,[rip+0x655de]        # 7d0b8 <__frame_dummy_init_array_entry+0x678>
   17ada:	lea    r8,[rip+0x64f7f]        # 7ca60 <__frame_dummy_init_array_entry+0x20>
   17ae1:	lea    rdx,[rsp+0x210]
   17ae9:	mov    esi,0x19
   17aee:	call   322c0 <_RNvNtCscI6d9CVNmLh_4core6result13unwrap_failed>
   17af3:	ud2
   17af5:	mov    edi,0x8
   17afa:	mov    esi,0x80
   17aff:	call   2e99b <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
   17b04:	jmp    17b08 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2068>
   17b06:	jmp    17b12 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2072>
   17b08:	mov    rbx,rax
   17b0b:	mov    r14,QWORD PTR [rsp+0x10]
   17b10:	jmp    17b32 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2092>
   17b12:	mov    rbx,rax
   17b15:	cmp    QWORD PTR [rsp+0x158],0x0
   17b1e:	je     17b42 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
   17b20:	mov    rax,QWORD PTR [rsp+0x160]
   17b28:	mov    QWORD PTR [rsp+0x8],rax
   17b2d:	jmp    17b37 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2097>
   17b2f:	mov    rbx,rax
   17b32:	test   r14,r14
   17b35:	je     17b42 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
   17b37:	mov    rdi,QWORD PTR [rsp+0x8]
   17b3c:	call   QWORD PTR [rip+0x6782e]        # 7f370 <free@GLIBC_2.2.5>
   17b42:	mov    rdi,rbx
   17b45:	call   7b9f0 <_Unwind_Resume@plt>
   17b4a:	int3
   17b4b:	int3
   17b4c:	int3
   17b4d:	int3
   17b4e:	int3
   17b4f:	int3
