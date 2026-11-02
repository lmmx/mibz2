
/home/louis/dev/mibz2/target/c30-traversal-pin/target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000000cbd0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_>:
    cbd0:	push   rbp
    cbd1:	push   r15
    cbd3:	push   r14
    cbd5:	push   r13
    cbd7:	push   r12
    cbd9:	push   rbx
    cbda:	sub    rsp,0x638
    cbe1:	mov    DWORD PTR [rsp+0x154],esi
    cbe8:	mov    rax,QWORD PTR [rdi+0xb8]
    cbef:	mov    QWORD PTR [rsp+0x178],rax
    cbf7:	mov    eax,DWORD PTR [rdi+0x10]
    cbfa:	mov    r12,QWORD PTR [rdi+0x40]
    cbfe:	mov    QWORD PTR [rdi+0xb0],0x0
    cc09:	mov    QWORD PTR [rdi+0x98],0x0
    cc14:	test   r12,r12
    cc17:	je     ebe1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2011>
    cc1d:	mov    rbx,rdi
    cc20:	shl    eax,0x5
    cc23:	add    eax,0xffffffe0
    cc26:	mov    DWORD PTR [rsp+0x24],eax
    cc2a:	xor    edi,edi
    cc2c:	lea    rax,[rbx+0xa0]
    cc33:	mov    QWORD PTR [rsp],rax
    cc37:	lea    rax,[rbx+0x88]
    cc3e:	mov    QWORD PTR [rsp+0x8],rax
    cc43:	mov    QWORD PTR [rsp+0x28],r12
    cc48:	jmp    cc79 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xa9>
    cc4a:	nop    WORD PTR [rax+rax*1+0x0]
    cc50:	inc    rdi
    cc53:	mov    rax,QWORD PTR [r14+0x90]
    cc5a:	mov    WORD PTR [rax+rbx*2],bp
    cc5e:	inc    rbx
    cc61:	mov    QWORD PTR [r14+0x98],rbx
    cc68:	mov    r12,QWORD PTR [rsp+0x28]
    cc6d:	cmp    r12,rdi
    cc70:	mov    rbx,r14
    cc73:	je     cdb0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e0>
    cc79:	mov    rsi,QWORD PTR [rbx+0x40]
    cc7d:	cmp    rdi,rsi
    cc80:	jae    ead0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f00>
    cc86:	mov    rsi,QWORD PTR [rbx+0x10]
    cc8a:	mov    rax,QWORD PTR [rbx+0x38]
    cc8e:	mov    ebp,DWORD PTR [rax+rdi*4]
    cc91:	mov    r12d,ebp
    cc94:	shr    r12d,0x5
    cc98:	cmp    rsi,r12
    cc9b:	jbe    eac1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ef1>
    cca1:	and    ebp,0x1f
    cca4:	cmp    ebp,0x1e
    cca7:	jae    eaad <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1edd>
    ccad:	mov    rcx,QWORD PTR [rbx+0x8]
    ccb1:	mov    rdx,r12
    ccb4:	shl    rdx,0x6
    ccb8:	lea    rax,[rcx+rdx*1]
    ccbc:	movzx  eax,WORD PTR [rax+rbp*2]
    ccc0:	mov    r13d,DWORD PTR [rcx+rdx*1+0x3c]
    ccc5:	cmp    eax,0xfffe
    ccca:	jae    cd50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x180>
    ccd0:	mov    r14,rbx
    ccd3:	mov    ebx,r13d
    ccd6:	shr    ebx,0x8
    ccd9:	add    ebx,eax
    ccdb:	mov    r15,QWORD PTR [r14+0xb0]
    cce2:	cmp    r15,QWORD PTR [r14+0xa0]
    cce9:	je     cd8e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1be>
    ccef:	shl    ebx,0x8
    ccf2:	movzx  eax,r13b
    ccf6:	or     eax,ebx
    ccf8:	mov    rcx,QWORD PTR [r14+0xa8]
    ccff:	mov    DWORD PTR [rcx+r15*4],eax
    cd03:	inc    r15
    cd06:	mov    QWORD PTR [r14+0xb0],r15
    cd0d:	mov    rsi,QWORD PTR [r14+0x10]
    cd11:	cmp    rsi,r12
    cd14:	jbe    ea9e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ece>
    cd1a:	add    rdx,QWORD PTR [r14+0x8]
    cd1e:	movzx  ebp,WORD PTR [rdx+rbp*2]
    cd22:	mov    rbx,QWORD PTR [r14+0x98]
    cd29:	cmp    rbx,QWORD PTR [r14+0x88]
    cd30:	jne    cc50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
    cd36:	mov    r15,rdi
    cd39:	mov    rdi,QWORD PTR [rsp+0x8]
    cd3e:	call   6500 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVectE8grow_oneCs3AlRknvjqqN_7c30walk>
    cd43:	mov    rdi,r15
    cd46:	jmp    cc50 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x80>
    cd4b:	nop    DWORD PTR [rax+rax*1+0x0]
    cd50:	and    r13d,0xfffff
    cd57:	mov    rsi,QWORD PTR [rbx+0x28]
    cd5b:	cmp    rsi,r13
    cd5e:	jbe    eb56 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f86>
    cd64:	shl    r13d,0x7
    cd68:	add    r13,QWORD PTR [rbx+0x20]
    cd6c:	mov    r13d,DWORD PTR [r13+rbp*4+0x0]
    cd71:	mov    r14,rbx
    cd74:	mov    ebx,r13d
    cd77:	shr    ebx,0x8
    cd7a:	mov    r15,QWORD PTR [r14+0xb0]
    cd81:	cmp    r15,QWORD PTR [r14+0xa0]
    cd88:	jne    ccef <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
    cd8e:	mov    QWORD PTR [rsp+0x38],rdi
    cd93:	mov    rdi,QWORD PTR [rsp]
    cd97:	mov    QWORD PTR [rsp+0x18],rdx
    cd9c:	call   6f00 <_RNvMs3_NtCs40k4W9msRzi_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCs7ivwnIpGHJL_5mibz2>
    cda1:	mov    rdx,QWORD PTR [rsp+0x18]
    cda6:	mov    rdi,QWORD PTR [rsp+0x38]
    cdab:	jmp    ccef <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11f>
    cdb0:	xor    edi,edi
    cdb2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
    cdc0:	mov    rsi,QWORD PTR [rbx+0x40]
    cdc4:	cmp    rdi,rsi
    cdc7:	jae    eaff <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f2f>
    cdcd:	mov    rsi,QWORD PTR [rbx+0x10]
    cdd1:	mov    rax,QWORD PTR [rbx+0x38]
    cdd5:	mov    eax,DWORD PTR [rax+rdi*4]
    cdd8:	mov    ecx,eax
    cdda:	shr    ecx,0x5
    cddd:	cmp    rsi,rcx
    cde0:	jbe    eaf0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f20>
    cde6:	and    eax,0x1f
    cde9:	cmp    eax,0x1e
    cdec:	jae    eadc <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0c>
    cdf2:	inc    rdi
    cdf5:	shl    rcx,0x6
    cdf9:	add    rcx,QWORD PTR [rbx+0x8]
    cdfd:	mov    WORD PTR [rcx+rax*2],0xfffe
    ce03:	cmp    r12,rdi
    ce06:	jne    cdc0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f0>
    ce08:	mov    r14,QWORD PTR [rbx+0x8]
    ce0c:	mov    rax,QWORD PTR [rbx+0x20]
    ce10:	mov    QWORD PTR [rsp+0x38],rax
    ce15:	mov    rax,QWORD PTR [rsp+0x178]
    ce1d:	add    rax,rax
    ce20:	mov    rcx,rax
    ce23:	or     rcx,r12
    ce26:	shr    rcx,0x20
    ce2a:	je     ce82 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2b2>
    ce2c:	xor    edx,edx
    ce2e:	div    r12
    ce31:	mov    rbp,rax
    ce34:	mov    r13,QWORD PTR [rbx+0x58]
    ce38:	cmp    r13,r12
    ce3b:	cmova  r12,r13
    ce3f:	jae    ce96 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2c6>
    ce41:	sub    r12,r13
    ce44:	mov    rax,QWORD PTR [rbx+0x48]
    ce48:	sub    rax,r13
    ce4b:	mov    rdx,r13
    ce4e:	cmp    r12,rax
    ce51:	ja     cea4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2d4>
    ce53:	mov    rax,QWORD PTR [rbx+0x50]
    ce57:	inc    r13
    ce5a:	test   r12b,0x1
    ce5e:	jne    cecf <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2ff>
    ce60:	cmp    QWORD PTR [rsp+0x28],r13
    ce65:	jne    cefe <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x32e>
    ce6b:	mov    QWORD PTR [rbx+0x58],rdx
    ce6f:	mov    rsi,QWORD PTR [rsp+0x28]
    ce74:	cmp    rsi,rdx
    ce77:	jbe    cf65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
    ce7d:	jmp    ea6e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
    ce82:	xor    edx,edx
    ce84:	div    r12d
    ce87:	mov    ebp,eax
    ce89:	mov    r13,QWORD PTR [rbx+0x58]
    ce8d:	cmp    r13,r12
    ce90:	cmova  r12,r13
    ce94:	jb     ce41 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x271>
    ce96:	mov    QWORD PTR [rbx+0x58],r12
    ce9a:	mov    rsi,QWORD PTR [rsp+0x28]
    ce9f:	jmp    cf65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x395>
    cea4:	lea    rdi,[rbx+0x48]
    cea8:	mov    ecx,0x8
    cead:	mov    r8d,0x18
    ceb3:	mov    rsi,r13
    ceb6:	mov    rdx,r12
    ceb9:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    cebe:	mov    rdx,QWORD PTR [rbx+0x58]
    cec2:	mov    rax,QWORD PTR [rbx+0x50]
    cec6:	inc    r13
    cec9:	test   r12b,0x1
    cecd:	je     ce60 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x290>
    cecf:	dec    r12
    ced2:	lea    rcx,[rdx+rdx*2]
    ced6:	mov    QWORD PTR [rax+rcx*8],0x0
    cede:	mov    QWORD PTR [rax+rcx*8+0x8],0x1
    cee7:	mov    QWORD PTR [rax+rcx*8+0x10],0x0
    cef0:	inc    rdx
    cef3:	cmp    QWORD PTR [rsp+0x28],r13
    cef8:	je     ce6b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x29b>
    cefe:	lea    rcx,[rdx+rdx*2]
    cf02:	lea    rax,[rax+rcx*8]
    cf06:	add    rax,0x28
    cf0a:	neg    r12
    cf0d:	xor    ecx,ecx
    cf0f:	xorps  xmm0,xmm0
    cf12:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
    cf20:	mov    QWORD PTR [rax-0x28],0x0
    cf28:	mov    QWORD PTR [rax-0x20],0x1
    cf30:	movups XMMWORD PTR [rax-0x18],xmm0
    cf34:	mov    QWORD PTR [rax-0x8],0x1
    cf3c:	mov    QWORD PTR [rax],0x0
    cf43:	add    rax,0x30
    cf47:	add    rcx,0xfffffffffffffffe
    cf4b:	cmp    r12,rcx
    cf4e:	jne    cf20 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x350>
    cf50:	sub    rdx,rcx
    cf53:	mov    QWORD PTR [rbx+0x58],rdx
    cf57:	mov    rsi,QWORD PTR [rsp+0x28]
    cf5c:	cmp    rsi,rdx
    cf5f:	ja     ea6e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e9e>
    cf65:	lea    r12,[rbp+0x40]
    cf69:	mov    r13,QWORD PTR [rbx+0x50]
    cf6d:	lea    rax,[rsi*8+0x0]
    cf75:	lea    r15,[rax+rax*2]
    cf79:	nop    DWORD PTR [rax+0x0]
    cf80:	mov    QWORD PTR [r13+0x10],0x0
    cf88:	cmp    r12,QWORD PTR [r13+0x0]
    cf8c:	ja     cf9a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ca>
    cf8e:	add    r13,0x18
    cf92:	add    r15,0xffffffffffffffe8
    cf96:	jne    cf80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
    cf98:	jmp    cfbc <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3ec>
    cf9a:	mov    ecx,0x1
    cf9f:	mov    r8d,0x1
    cfa5:	mov    rdi,r13
    cfa8:	xor    esi,esi
    cfaa:	mov    rdx,r12
    cfad:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    cfb2:	add    r13,0x18
    cfb6:	add    r15,0xffffffffffffffe8
    cfba:	jne    cf80 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x3b0>
    cfbc:	mov    QWORD PTR [rbx+0x70],0x0
    cfc4:	mov    r12,QWORD PTR [rsp+0x28]
    cfc9:	cmp    r12,QWORD PTR [rbx+0x60]
    cfcd:	ja     ea7c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1eac>
    cfd3:	xor    r15d,r15d
    cfd6:	mov    QWORD PTR [rsp+0x18],rbx
    cfdb:	mov    rbx,QWORD PTR [rbx+0x68]
    cfdf:	lea    rdi,[rbx+r15*8]
    cfe3:	cmp    r12,0x1
    cfe7:	je     d00e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x43e>
    cfe9:	lea    rdx,[r12*8-0x8]
    cff1:	mov    esi,0xff
    cff6:	call   QWORD PTR [rip+0x84e4c]        # 91e48 <memset@GLIBC_2.2.5>
    cffc:	lea    rax,[r15+r12*1]
    d000:	lea    rdi,[rbx+rax*8]
    d004:	add    rdi,0xfffffffffffffff8
    d008:	add    r15,r12
    d00b:	dec    r15
    d00e:	mov    QWORD PTR [rdi],0xffffffffffffffff
    d015:	inc    r15
    d018:	mov    rbx,QWORD PTR [rsp+0x18]
    d01d:	mov    QWORD PTR [rbx+0x70],r15
    d021:	xorps  xmm0,xmm0
    d024:	movups XMMWORD PTR [rsp+0x240],xmm0
    d02c:	mov    QWORD PTR [rsp+0x250],0x0
    d038:	movups XMMWORD PTR [rsp+0x260],xmm0
    d040:	mov    QWORD PTR [rsp+0x270],0x0
    d04c:	movups XMMWORD PTR [rsp+0x280],xmm0
    d054:	mov    QWORD PTR [rsp+0x290],0x0
    d060:	movups XMMWORD PTR [rsp+0x2a0],xmm0
    d068:	mov    QWORD PTR [rsp+0x2b0],0x0
    d074:	movups XMMWORD PTR [rsp+0x2c0],xmm0
    d07c:	mov    QWORD PTR [rsp+0x2d0],0x0
    d088:	movups XMMWORD PTR [rsp+0x2e0],xmm0
    d090:	mov    QWORD PTR [rsp+0x2f0],0x0
    d09c:	movups XMMWORD PTR [rsp+0x300],xmm0
    d0a4:	mov    QWORD PTR [rsp+0x310],0x0
    d0b0:	movups XMMWORD PTR [rsp+0x320],xmm0
    d0b8:	mov    QWORD PTR [rsp+0x330],0x0
    d0c4:	movups XMMWORD PTR [rsp+0x340],xmm0
    d0cc:	mov    QWORD PTR [rsp+0x350],0x0
    d0d8:	movups XMMWORD PTR [rsp+0x360],xmm0
    d0e0:	mov    QWORD PTR [rsp+0x370],0x0
    d0ec:	movups XMMWORD PTR [rsp+0x380],xmm0
    d0f4:	mov    QWORD PTR [rsp+0x390],0x0
    d100:	movups XMMWORD PTR [rsp+0x3a0],xmm0
    d108:	mov    QWORD PTR [rsp+0x3b0],0x0
    d114:	movups XMMWORD PTR [rsp+0x3c0],xmm0
    d11c:	mov    QWORD PTR [rsp+0x3d0],0x0
    d128:	movups XMMWORD PTR [rsp+0x3e0],xmm0
    d130:	mov    QWORD PTR [rsp+0x3f0],0x0
    d13c:	movups XMMWORD PTR [rsp+0x400],xmm0
    d144:	mov    QWORD PTR [rsp+0x410],0x0
    d150:	movups XMMWORD PTR [rsp+0x420],xmm0
    d158:	mov    QWORD PTR [rsp+0x430],0x0
    d164:	movups XMMWORD PTR [rsp+0x440],xmm0
    d16c:	mov    QWORD PTR [rsp+0x450],0x0
    d178:	movups XMMWORD PTR [rsp+0x460],xmm0
    d180:	mov    QWORD PTR [rsp+0x470],0x0
    d18c:	movups XMMWORD PTR [rsp+0x480],xmm0
    d194:	mov    QWORD PTR [rsp+0x490],0x0
    d1a0:	movups XMMWORD PTR [rsp+0x4a0],xmm0
    d1a8:	mov    QWORD PTR [rsp+0x4b0],0x0
    d1b4:	movups XMMWORD PTR [rsp+0x4c0],xmm0
    d1bc:	mov    QWORD PTR [rsp+0x4d0],0x0
    d1c8:	movups XMMWORD PTR [rsp+0x4e0],xmm0
    d1d0:	mov    QWORD PTR [rsp+0x4f0],0x0
    d1dc:	movups XMMWORD PTR [rsp+0x500],xmm0
    d1e4:	mov    QWORD PTR [rsp+0x510],0x0
    d1f0:	movups XMMWORD PTR [rsp+0x520],xmm0
    d1f8:	mov    QWORD PTR [rsp+0x530],0x0
    d204:	movups XMMWORD PTR [rsp+0x540],xmm0
    d20c:	mov    QWORD PTR [rsp+0x550],0x0
    d218:	movups XMMWORD PTR [rsp+0x560],xmm0
    d220:	mov    QWORD PTR [rsp+0x570],0x0
    d22c:	movups XMMWORD PTR [rsp+0x580],xmm0
    d234:	mov    QWORD PTR [rsp+0x590],0x0
    d240:	movups XMMWORD PTR [rsp+0x5a0],xmm0
    d248:	mov    QWORD PTR [rsp+0x5b0],0x0
    d254:	movups XMMWORD PTR [rsp+0x5c0],xmm0
    d25c:	mov    QWORD PTR [rsp+0x5d0],0x0
    d268:	movups XMMWORD PTR [rsp+0x5e0],xmm0
    d270:	mov    QWORD PTR [rsp+0x5f0],0x0
    d27c:	movups XMMWORD PTR [rsp+0x600],xmm0
    d284:	mov    QWORD PTR [rsp+0x610],0x0
    d290:	movups XMMWORD PTR [rsp+0x620],xmm0
    d298:	mov    QWORD PTR [rsp+0x630],0x0
    d2a4:	mov    QWORD PTR [rsp+0x238],0xffffffffffffffff
    d2b0:	mov    QWORD PTR [rsp+0x258],0xffffffffffffffff
    d2bc:	mov    QWORD PTR [rsp+0x278],0xffffffffffffffff
    d2c8:	mov    QWORD PTR [rsp+0x298],0xffffffffffffffff
    d2d4:	mov    QWORD PTR [rsp+0x2b8],0xffffffffffffffff
    d2e0:	mov    QWORD PTR [rsp+0x2d8],0xffffffffffffffff
    d2ec:	mov    QWORD PTR [rsp+0x2f8],0xffffffffffffffff
    d2f8:	mov    QWORD PTR [rsp+0x318],0xffffffffffffffff
    d304:	mov    QWORD PTR [rsp+0x338],0xffffffffffffffff
    d310:	mov    QWORD PTR [rsp+0x358],0xffffffffffffffff
    d31c:	mov    QWORD PTR [rsp+0x378],0xffffffffffffffff
    d328:	mov    QWORD PTR [rsp+0x398],0xffffffffffffffff
    d334:	mov    QWORD PTR [rsp+0x3b8],0xffffffffffffffff
    d340:	mov    QWORD PTR [rsp+0x3d8],0xffffffffffffffff
    d34c:	mov    QWORD PTR [rsp+0x3f8],0xffffffffffffffff
    d358:	mov    QWORD PTR [rsp+0x418],0xffffffffffffffff
    d364:	mov    QWORD PTR [rsp+0x438],0xffffffffffffffff
    d370:	mov    QWORD PTR [rsp+0x458],0xffffffffffffffff
    d37c:	mov    QWORD PTR [rsp+0x478],0xffffffffffffffff
    d388:	mov    QWORD PTR [rsp+0x498],0xffffffffffffffff
    d394:	mov    QWORD PTR [rsp+0x4b8],0xffffffffffffffff
    d3a0:	mov    QWORD PTR [rsp+0x4d8],0xffffffffffffffff
    d3ac:	mov    QWORD PTR [rsp+0x4f8],0xffffffffffffffff
    d3b8:	mov    QWORD PTR [rsp+0x518],0xffffffffffffffff
    d3c4:	mov    QWORD PTR [rsp+0x538],0xffffffffffffffff
    d3d0:	mov    QWORD PTR [rsp+0x558],0xffffffffffffffff
    d3dc:	mov    QWORD PTR [rsp+0x578],0xffffffffffffffff
    d3e8:	mov    QWORD PTR [rsp+0x598],0xffffffffffffffff
    d3f4:	mov    QWORD PTR [rsp+0x5b8],0xffffffffffffffff
    d400:	mov    QWORD PTR [rsp+0x5d8],0xffffffffffffffff
    d40c:	mov    QWORD PTR [rsp+0x5f8],0xffffffffffffffff
    d418:	mov    QWORD PTR [rsp+0x618],0xffffffffffffffff
    d424:	mov    eax,0x2
    d429:	xor    edi,edi
    d42b:	xor    r9d,r9d
    d42e:	xchg   ax,ax
    d430:	mov    rsi,QWORD PTR [rbx+0x58]
    d434:	cmp    rdi,rsi
    d437:	jae    eb17 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f47>
    d43d:	mov    rsi,QWORD PTR [rbx+0xb0]
    d444:	cmp    rdi,rsi
    d447:	jae    eb0b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f3b>
    d44d:	mov    rcx,QWORD PTR [rbx+0x50]
    d451:	mov    rdx,QWORD PTR [rbx+0xa8]
    d458:	mov    rsi,QWORD PTR [rcx+rax*4]
    d45c:	movzx  r8d,BYTE PTR [rdx+rdi*4]
    d461:	mov    BYTE PTR [rsi],r8b
    d464:	mov    edx,DWORD PTR [rdx+rdi*4]
    d467:	shr    edx,0x8
    d46a:	lea    r8,[rsi+0x1]
    d46e:	add    rsi,QWORD PTR [rcx+rax*4-0x8]
    d473:	mov    QWORD PTR [rsp+r9*1+0x238],rdi
    d47b:	mov    QWORD PTR [rsp+r9*1+0x240],r8
    d483:	mov    QWORD PTR [rsp+r9*1+0x248],rsi
    d48b:	mov    DWORD PTR [rsp+r9*1+0x250],edx
    d493:	mov    DWORD PTR [rsp+r9*1+0x254],0x0
    d49f:	prefetcht0 BYTE PTR [r14+rdx*2]
    d4a4:	lea    rsi,[rdi+0x1]
    d4a8:	add    r9,0x20
    d4ac:	cmp    rdi,0x1e
    d4b0:	ja     d4c2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x8f2>
    d4b2:	add    rax,0x6
    d4b6:	mov    rdi,rsi
    d4b9:	cmp    rsi,r12
    d4bc:	jb     d430 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x860>
    d4c2:	cmp    rsi,0x20
    d4c6:	jne    e4a2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
    d4cc:	mov    QWORD PTR [rsp+0x170],rsi
    d4d4:	add    rbp,0x3f
    d4d8:	mov    eax,0x20
    d4dd:	mov    QWORD PTR [rsp+0x8],rax
    d4e2:	mov    QWORD PTR [rsp+0x230],rbp
    d4ea:	jmp    d87d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcad>
    d4ef:	nop
    d4f0:	mov    rbp,QWORD PTR [rsp+0x48]
    d4f5:	mov    r12,QWORD PTR [rsp+0x50]
    d4fa:	mov    rax,QWORD PTR [rsp+0x58]
    d4ff:	mov    QWORD PTR [rsp+0x10],rax
    d504:	mov    rax,QWORD PTR [rsp+0x60]
    d509:	mov    QWORD PTR [rsp+0x1b8],rax
    d511:	mov    rax,QWORD PTR [rsp+0x68]
    d516:	mov    QWORD PTR [rsp+0x1c0],rax
    d51e:	mov    rax,QWORD PTR [rsp+0x70]
    d523:	mov    QWORD PTR [rsp+0x1c8],rax
    d52b:	mov    rax,QWORD PTR [rsp+0x78]
    d530:	mov    QWORD PTR [rsp+0x1d0],rax
    d538:	mov    rax,QWORD PTR [rsp+0x80]
    d540:	mov    QWORD PTR [rsp+0x1d8],rax
    d548:	mov    rax,QWORD PTR [rsp+0x88]
    d550:	mov    QWORD PTR [rsp+0x1e0],rax
    d558:	mov    rax,QWORD PTR [rsp+0x90]
    d560:	mov    QWORD PTR [rsp+0x1e8],rax
    d568:	mov    rax,QWORD PTR [rsp+0x98]
    d570:	mov    QWORD PTR [rsp+0x1f0],rax
    d578:	mov    rax,QWORD PTR [rsp+0xa0]
    d580:	mov    QWORD PTR [rsp+0x1f8],rax
    d588:	mov    rax,QWORD PTR [rsp+0xa8]
    d590:	mov    QWORD PTR [rsp+0x200],rax
    d598:	mov    rax,QWORD PTR [rsp+0xb0]
    d5a0:	mov    QWORD PTR [rsp+0x208],rax
    d5a8:	mov    rax,QWORD PTR [rsp+0xb8]
    d5b0:	mov    QWORD PTR [rsp],rax
    d5b4:	mov    rax,QWORD PTR [rsp+0xc0]
    d5bc:	mov    QWORD PTR [rsp+0x190],rax
    d5c4:	mov    rax,QWORD PTR [rsp+0xc8]
    d5cc:	mov    QWORD PTR [rsp+0x198],rax
    d5d4:	mov    rax,QWORD PTR [rsp+0xd0]
    d5dc:	mov    QWORD PTR [rsp+0x1a0],rax
    d5e4:	mov    rax,QWORD PTR [rsp+0xd8]
    d5ec:	mov    QWORD PTR [rsp+0x1a8],rax
    d5f4:	mov    rax,QWORD PTR [rsp+0xe0]
    d5fc:	mov    QWORD PTR [rsp+0x1b0],rax
    d604:	mov    rax,QWORD PTR [rsp+0xe8]
    d60c:	mov    QWORD PTR [rsp+0x180],rax
    d614:	mov    rax,QWORD PTR [rsp+0xf0]
    d61c:	mov    QWORD PTR [rsp+0x188],rax
    d624:	mov    rdi,QWORD PTR [rsp+0xf8]
    d62c:	mov    rsi,QWORD PTR [rsp+0x100]
    d634:	mov    rdx,QWORD PTR [rsp+0x108]
    d63c:	mov    rcx,QWORD PTR [rsp+0x110]
    d644:	mov    rax,QWORD PTR [rsp+0x118]
    d64c:	mov    r13,QWORD PTR [rsp+0x120]
    d654:	mov    r11,QWORD PTR [rsp+0x128]
    d65c:	mov    r10,QWORD PTR [rsp+0x130]
    d664:	mov    r9,QWORD PTR [rsp+0x138]
    d66c:	mov    r8,QWORD PTR [rsp+0x140]
    d674:	add    rbp,r15
    d677:	mov    QWORD PTR [rsp+0x240],rbp
    d67f:	add    r12,r15
    d682:	mov    QWORD PTR [rsp+0x260],r12
    d68a:	mov    r12,QWORD PTR [rsp+0x10]
    d68f:	add    r12,r15
    d692:	mov    QWORD PTR [rsp+0x280],r12
    d69a:	mov    r12,QWORD PTR [rsp+0x1b8]
    d6a2:	add    r12,r15
    d6a5:	mov    QWORD PTR [rsp+0x2a0],r12
    d6ad:	mov    r12,QWORD PTR [rsp+0x1c0]
    d6b5:	add    r12,r15
    d6b8:	mov    QWORD PTR [rsp+0x2c0],r12
    d6c0:	mov    r12,QWORD PTR [rsp+0x1c8]
    d6c8:	add    r12,r15
    d6cb:	mov    QWORD PTR [rsp+0x2e0],r12
    d6d3:	mov    r12,QWORD PTR [rsp+0x1d0]
    d6db:	add    r12,r15
    d6de:	mov    QWORD PTR [rsp+0x300],r12
    d6e6:	mov    r12,QWORD PTR [rsp+0x1d8]
    d6ee:	add    r12,r15
    d6f1:	mov    QWORD PTR [rsp+0x320],r12
    d6f9:	mov    r12,QWORD PTR [rsp+0x1e0]
    d701:	add    r12,r15
    d704:	mov    QWORD PTR [rsp+0x340],r12
    d70c:	mov    r12,QWORD PTR [rsp+0x1e8]
    d714:	add    r12,r15
    d717:	mov    QWORD PTR [rsp+0x360],r12
    d71f:	mov    r12,QWORD PTR [rsp+0x1f0]
    d727:	add    r12,r15
    d72a:	mov    QWORD PTR [rsp+0x380],r12
    d732:	mov    r12,QWORD PTR [rsp+0x1f8]
    d73a:	add    r12,r15
    d73d:	mov    QWORD PTR [rsp+0x3a0],r12
    d745:	mov    r12,QWORD PTR [rsp+0x200]
    d74d:	add    r12,r15
    d750:	mov    QWORD PTR [rsp+0x3c0],r12
    d758:	mov    r12,QWORD PTR [rsp+0x208]
    d760:	add    r12,r15
    d763:	mov    QWORD PTR [rsp+0x3e0],r12
    d76b:	mov    r12,QWORD PTR [rsp]
    d76f:	add    r12,r15
    d772:	mov    QWORD PTR [rsp+0x400],r12
    d77a:	mov    r12,QWORD PTR [rsp+0x190]
    d782:	add    r12,r15
    d785:	mov    QWORD PTR [rsp+0x420],r12
    d78d:	mov    r12,QWORD PTR [rsp+0x198]
    d795:	add    r12,r15
    d798:	mov    QWORD PTR [rsp+0x440],r12
    d7a0:	mov    r12,QWORD PTR [rsp+0x1a0]
    d7a8:	add    r12,r15
    d7ab:	mov    QWORD PTR [rsp+0x460],r12
    d7b3:	mov    r12,QWORD PTR [rsp+0x1a8]
    d7bb:	add    r12,r15
    d7be:	mov    QWORD PTR [rsp+0x480],r12
    d7c6:	mov    r12,QWORD PTR [rsp+0x1b0]
    d7ce:	add    r12,r15
    d7d1:	mov    QWORD PTR [rsp+0x4a0],r12
    d7d9:	mov    r12,QWORD PTR [rsp+0x180]
    d7e1:	add    r12,r15
    d7e4:	mov    QWORD PTR [rsp+0x4c0],r12
    d7ec:	mov    r12,QWORD PTR [rsp+0x188]
    d7f4:	add    r12,r15
    d7f7:	mov    QWORD PTR [rsp+0x4e0],r12
    d7ff:	add    rdi,r15
    d802:	mov    QWORD PTR [rsp+0x500],rdi
    d80a:	add    rsi,r15
    d80d:	mov    QWORD PTR [rsp+0x520],rsi
    d815:	add    rdx,r15
    d818:	mov    QWORD PTR [rsp+0x540],rdx
    d820:	add    rcx,r15
    d823:	mov    QWORD PTR [rsp+0x560],rcx
    d82b:	add    rax,r15
    d82e:	mov    QWORD PTR [rsp+0x580],rax
    d836:	add    r13,r15
    d839:	mov    QWORD PTR [rsp+0x5a0],r13
    d841:	add    r11,r15
    d844:	mov    QWORD PTR [rsp+0x5c0],r11
    d84c:	add    r10,r15
    d84f:	mov    QWORD PTR [rsp+0x5e0],r10
    d857:	add    r9,r15
    d85a:	mov    QWORD PTR [rsp+0x600],r9
    d862:	add    r8,r15
    d865:	mov    QWORD PTR [rsp+0x620],r8
    d86d:	mov    r9,QWORD PTR [rsp+0x148]
    d875:	mov    rbp,QWORD PTR [rsp+0x230]
    d87d:	mov    r15,rbp
    d880:	xor    r12d,r12d
    d883:	jmp    d8c1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcf1>
    d885:	data16 cs nop WORD PTR [rax+rax*1+0x0]
    d890:	add    r13,rax
    d893:	mov    QWORD PTR [rsp+r12*1+0x240],r13
    d89b:	add    rdx,rax
    d89e:	mov    QWORD PTR [rsp+r12*1+0x248],rdx
    d8a6:	sub    rdx,r13
    d8a9:	add    r12,0x20
    d8ad:	cmp    rdx,r15
    d8b0:	cmovb  r15,rdx
    d8b4:	cmp    r12,0x400
    d8bb:	je     d940 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xd70>
    d8c1:	mov    r13,QWORD PTR [rsp+r12*1+0x240]
    d8c9:	mov    rdx,QWORD PTR [rsp+r12*1+0x248]
    d8d1:	cmp    r13,rdx
    d8d4:	jne    d8a6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcd6>
    d8d6:	mov    rsi,QWORD PTR [rbx+0x58]
    d8da:	mov    rdi,QWORD PTR [rsp+r12*1+0x238]
    d8e2:	cmp    rdi,rsi
    d8e5:	jae    eb4a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f7a>
    d8eb:	mov    rcx,QWORD PTR [rbx+0x50]
    d8ef:	lea    rsi,[rdi+rdi*2]
    d8f3:	mov    rdx,QWORD PTR [rcx+rsi*8]
    d8f7:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
    d8fc:	sub    r13,rax
    d8ff:	mov    QWORD PTR [rcx+rsi*8+0x10],r13
    d904:	mov    rdi,rdx
    d907:	sub    rdi,r13
    d90a:	cmp    rdx,rdi
    d90d:	jbe    d890 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
    d90f:	lea    rdi,[rcx+rsi*8]
    d913:	mov    QWORD PTR [rsp],rdi
    d917:	mov    ecx,0x1
    d91c:	mov    r8d,0x1
    d922:	mov    rsi,r13
    d925:	mov    rbp,r9
    d928:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    d92d:	mov    r9,rbp
    d930:	mov    rax,QWORD PTR [rsp]
    d934:	mov    rdx,QWORD PTR [rax]
    d937:	mov    rax,QWORD PTR [rax+0x8]
    d93b:	jmp    d890 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xcc0>
    d940:	mov    QWORD PTR [rsp+0x148],r9
    d948:	mov    rax,QWORD PTR [rsp+0x240]
    d950:	mov    QWORD PTR [rsp+0x30],rax
    d955:	mov    rax,QWORD PTR [rsp+0x260]
    d95d:	mov    QWORD PTR [rsp+0x40],rax
    d962:	mov    rcx,QWORD PTR [rsp+0x280]
    d96a:	mov    QWORD PTR [rsp+0x10],rcx
    d96f:	mov    r12,QWORD PTR [rsp+0x2a0]
    d977:	mov    rbp,QWORD PTR [rsp+0x2c0]
    d97f:	mov    r13,QWORD PTR [rsp+0x2e0]
    d987:	mov    r11,QWORD PTR [rsp+0x300]
    d98f:	mov    r10,QWORD PTR [rsp+0x320]
    d997:	mov    r9,QWORD PTR [rsp+0x340]
    d99f:	mov    r8,QWORD PTR [rsp+0x360]
    d9a7:	mov    rdi,QWORD PTR [rsp+0x380]
    d9af:	mov    rsi,QWORD PTR [rsp+0x3a0]
    d9b7:	mov    rdx,QWORD PTR [rsp+0x3c0]
    d9bf:	mov    rcx,QWORD PTR [rsp+0x3e0]
    d9c7:	mov    rax,QWORD PTR [rsp+0x400]
    d9cf:	mov    QWORD PTR [rsp],rax
    d9d3:	mov    rax,QWORD PTR [rsp+0x30]
    d9d8:	mov    QWORD PTR [rsp+0x48],rax
    d9dd:	mov    rax,QWORD PTR [rsp+0x40]
    d9e2:	mov    QWORD PTR [rsp+0x50],rax
    d9e7:	mov    rax,QWORD PTR [rsp+0x10]
    d9ec:	mov    QWORD PTR [rsp+0x58],rax
    d9f1:	mov    QWORD PTR [rsp+0x1b8],r12
    d9f9:	mov    QWORD PTR [rsp+0x60],r12
    d9fe:	mov    QWORD PTR [rsp+0x1c0],rbp
    da06:	mov    QWORD PTR [rsp+0x68],rbp
    da0b:	mov    QWORD PTR [rsp+0x1c8],r13
    da13:	mov    QWORD PTR [rsp+0x70],r13
    da18:	mov    QWORD PTR [rsp+0x1d0],r11
    da20:	mov    QWORD PTR [rsp+0x78],r11
    da25:	mov    QWORD PTR [rsp+0x1d8],r10
    da2d:	mov    QWORD PTR [rsp+0x80],r10
    da35:	mov    QWORD PTR [rsp+0x1e0],r9
    da3d:	mov    QWORD PTR [rsp+0x88],r9
    da45:	mov    QWORD PTR [rsp+0x1e8],r8
    da4d:	mov    QWORD PTR [rsp+0x90],r8
    da55:	mov    QWORD PTR [rsp+0x1f0],rdi
    da5d:	mov    QWORD PTR [rsp+0x98],rdi
    da65:	mov    QWORD PTR [rsp+0x1f8],rsi
    da6d:	mov    QWORD PTR [rsp+0xa0],rsi
    da75:	mov    QWORD PTR [rsp+0x200],rdx
    da7d:	mov    QWORD PTR [rsp+0xa8],rdx
    da85:	mov    QWORD PTR [rsp+0x208],rcx
    da8d:	mov    QWORD PTR [rsp+0xb0],rcx
    da95:	mov    rax,QWORD PTR [rsp]
    da99:	mov    QWORD PTR [rsp+0xb8],rax
    daa1:	mov    rax,QWORD PTR [rsp+0x420]
    daa9:	mov    QWORD PTR [rsp+0x190],rax
    dab1:	mov    QWORD PTR [rsp+0xc0],rax
    dab9:	mov    rax,QWORD PTR [rsp+0x440]
    dac1:	mov    QWORD PTR [rsp+0x198],rax
    dac9:	mov    QWORD PTR [rsp+0xc8],rax
    dad1:	mov    rax,QWORD PTR [rsp+0x460]
    dad9:	mov    QWORD PTR [rsp+0x1a0],rax
    dae1:	mov    QWORD PTR [rsp+0xd0],rax
    dae9:	mov    rax,QWORD PTR [rsp+0x480]
    daf1:	mov    QWORD PTR [rsp+0x1a8],rax
    daf9:	mov    QWORD PTR [rsp+0xd8],rax
    db01:	mov    rax,QWORD PTR [rsp+0x4a0]
    db09:	mov    QWORD PTR [rsp+0x1b0],rax
    db11:	mov    QWORD PTR [rsp+0xe0],rax
    db19:	mov    rbp,QWORD PTR [rsp+0x4c0]
    db21:	mov    QWORD PTR [rsp+0xe8],rbp
    db29:	mov    r12,QWORD PTR [rsp+0x4e0]
    db31:	mov    QWORD PTR [rsp+0xf0],r12
    db39:	mov    rdi,QWORD PTR [rsp+0x500]
    db41:	mov    QWORD PTR [rsp+0xf8],rdi
    db49:	mov    rsi,QWORD PTR [rsp+0x520]
    db51:	mov    QWORD PTR [rsp+0x100],rsi
    db59:	mov    rdx,QWORD PTR [rsp+0x540]
    db61:	mov    QWORD PTR [rsp+0x108],rdx
    db69:	mov    rcx,QWORD PTR [rsp+0x560]
    db71:	mov    QWORD PTR [rsp+0x110],rcx
    db79:	mov    rax,QWORD PTR [rsp+0x580]
    db81:	mov    QWORD PTR [rsp+0x118],rax
    db89:	mov    r13,QWORD PTR [rsp+0x5a0]
    db91:	mov    QWORD PTR [rsp+0x120],r13
    db99:	mov    r11,QWORD PTR [rsp+0x5c0]
    dba1:	mov    QWORD PTR [rsp+0x128],r11
    dba9:	mov    r10,QWORD PTR [rsp+0x5e0]
    dbb1:	mov    QWORD PTR [rsp+0x130],r10
    dbb9:	mov    r9,QWORD PTR [rsp+0x600]
    dbc1:	mov    QWORD PTR [rsp+0x138],r9
    dbc9:	mov    r8,QWORD PTR [rsp+0x620]
    dbd1:	mov    QWORD PTR [rsp+0x140],r8
    dbd9:	test   r15,r15
    dbdc:	je     de60 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1290>
    dbe2:	xor    eax,eax
    dbe4:	mov    ebp,DWORD PTR [rsp+0x24]
    dbe8:	mov    r13,QWORD PTR [rsp+0x38]
    dbed:	lea    r12,[rsp+0x250]
    dbf5:	jmp    dc0d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x103d>
    dbf7:	nop    WORD PTR [rax+rax*1+0x0]
    dc00:	mov    rax,QWORD PTR [rsp]
    dc04:	cmp    rax,r15
    dc07:	je     d4f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x920>
    dc0d:	lea    rcx,[rax+0x1]
    dc11:	mov    QWORD PTR [rsp],rcx
    dc15:	mov    r8,rax
    dc18:	not    r8
    dc1b:	xor    r9d,r9d
    dc1e:	mov    r10,r9
    dc21:	shl    r10,0x5
    dc25:	add    r10,r12
    dc28:	mov    rdx,r9
    dc2b:	jmp    dc5b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x108b>
    dc2d:	nop    DWORD PTR [rax]
    dc30:	mov    ecx,esi
    dc32:	shr    ecx,0x8
    dc35:	add    ecx,edi
    dc37:	mov    DWORD PTR [r10],ecx
    dc3a:	mov    ecx,ecx
    dc3c:	prefetcht0 BYTE PTR [r14+rcx*2]
    dc41:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
    dc46:	mov    BYTE PTR [rcx+rax*1],sil
    dc4a:	lea    rcx,[r9-0x1]
    dc4e:	add    r10,0x20
    dc52:	mov    rdx,r9
    dc55:	cmp    rcx,0x1f
    dc59:	jae    dc00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
    dc5b:	lea    r9,[rdx+0x1]
    dc5f:	mov    r11d,DWORD PTR [r10]
    dc62:	movzx  edi,WORD PTR [r14+r11*2]
    dc67:	mov    ecx,r11d
    dc6a:	and    ecx,0xffffffe0
    dc6d:	mov    esi,DWORD PTR [r14+rcx*2+0x3c]
    dc72:	cmp    edi,0xfffe
    dc78:	jb     dc30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1060>
    dc7a:	je     dca0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x10d0>
    dc7c:	cmp    r11d,ebp
    dc7f:	jne    dd60 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1190>
    dc85:	mov    ecx,DWORD PTR [r10+0x4]
    dc89:	mov    esi,DWORD PTR [r13+rcx*4+0x0]
    dc8e:	mov    ecx,esi
    dc90:	shr    ecx,0x8
    dc93:	jmp    dc37 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1067>
    dc95:	data16 cs nop WORD PTR [rax+rax*1+0x0]
    dca0:	mov    rcx,QWORD PTR [rsp+rdx*8+0x48]
    dca5:	add    rcx,rax
    dca8:	mov    QWORD PTR [r10-0x10],rcx
    dcac:	mov    rsi,QWORD PTR [rbx+0x58]
    dcb0:	mov    rdi,QWORD PTR [r10-0x18]
    dcb4:	cmp    rdi,rsi
    dcb7:	jae    eb71 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fa1>
    dcbd:	mov    rsi,QWORD PTR [rsp+0x18]
    dcc2:	mov    rbx,QWORD PTR [rsi+0x50]
    dcc6:	mov    rsi,QWORD PTR [rsp+0x18]
    dccb:	mov    r13,QWORD PTR [rsi+0x68]
    dccf:	lea    r12,[rdi+rdi*2]
    dcd3:	sub    rcx,QWORD PTR [rbx+r12*8+0x8]
    dcd8:	mov    rsi,QWORD PTR [rsp+0x18]
    dcdd:	mov    rsi,QWORD PTR [rsi+0x70]
    dce1:	mov    QWORD PTR [rbx+r12*8+0x10],rcx
    dce6:	mov    rbx,QWORD PTR [rsp+0x18]
    dceb:	mov    rbp,QWORD PTR [rbx+0x40]
    dcef:	test   rbp,rbp
    dcf2:	mov    QWORD PTR [rsp+0x10],r13
    dcf7:	je     dd91 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11c1>
    dcfd:	mov    rcx,QWORD PTR [rbx+0x38]
    dd01:	xor    r13d,r13d
    dd04:	cmp    rbp,0x1
    dd08:	je     dd2d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x115d>
    dd0a:	nop    WORD PTR [rax+rax*1+0x0]
    dd10:	mov    rbx,r13
    dd13:	mov    r12,rbp
    dd16:	shr    r12,1
    dd19:	add    r13,r12
    dd1c:	cmp    DWORD PTR [rcx+r13*4],r11d
    dd20:	cmova  r13,rbx
    dd24:	sub    rbp,r12
    dd27:	cmp    rbp,0x1
    dd2b:	ja     dd10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1140>
    dd2d:	cmp    DWORD PTR [rcx+r13*4],r11d
    dd31:	mov    rcx,0xffffffffffffffff
    dd38:	cmovne r13,rcx
    dd3c:	mov    rbx,QWORD PTR [rsp+0x18]
    dd41:	cmp    rdi,rsi
    dd44:	mov    rcx,QWORD PTR [rsp+0x28]
    dd49:	mov    rbp,QWORD PTR [rsp+0x8]
    dd4e:	jb     ddab <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x11db>
    dd50:	jmp    eb65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
    dd55:	data16 cs nop WORD PTR [rax+rax*1+0x0]
    dd60:	shl    esi,0x5
    dd63:	and    esi,0x1ffffe0
    dd69:	and    r11d,0x1f
    dd6d:	or     r11d,esi
    dd70:	mov    DWORD PTR [r10+0x4],r11d
    dd74:	prefetcht0 BYTE PTR [r13+r11*4+0x0]
    dd7a:	mov    DWORD PTR [r10],ebp
    dd7d:	dec    QWORD PTR [rsp+rdx*8+0x48]
    dd82:	cmp    rdx,0x1f
    dd86:	jb     dc1e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
    dd8c:	jmp    dc00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
    dd91:	mov    r13,0xffffffffffffffff
    dd98:	cmp    rdi,rsi
    dd9b:	mov    rcx,QWORD PTR [rsp+0x28]
    dda0:	mov    rbp,QWORD PTR [rsp+0x8]
    dda5:	jae    eb65 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f95>
    ddab:	mov    rsi,QWORD PTR [rsp+0x10]
    ddb0:	mov    QWORD PTR [rsi+rdi*8],r13
    ddb4:	cmp    rbp,rcx
    ddb7:	je     de7f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x12af>
    ddbd:	mov    rsi,QWORD PTR [rbx+0x58]
    ddc1:	cmp    rbp,rsi
    ddc4:	mov    r13,QWORD PTR [rsp+0x38]
    ddc9:	jae    eb8c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fbc>
    ddcf:	mov    rsi,QWORD PTR [rbx+0xb0]
    ddd6:	cmp    rbp,rsi
    ddd9:	jae    eb7d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fad>
    dddf:	mov    rcx,QWORD PTR [rbx+0x50]
    dde3:	mov    rsi,QWORD PTR [rbx+0xa8]
    ddea:	lea    rdi,[rbp*2+0x0]
    ddf2:	add    rdi,rbp
    ddf5:	mov    r11,QWORD PTR [rcx+rdi*8+0x8]
    ddfa:	movzx  ebx,BYTE PTR [rsi+rbp*4]
    ddfe:	mov    BYTE PTR [r11],bl
    de01:	mov    esi,DWORD PTR [rsi+rbp*4]
    de04:	shr    esi,0x8
    de07:	lea    rbx,[r11+0x1]
    de0b:	lea    r12,[r11+r8*1]
    de0f:	inc    r12
    de12:	add    r11,QWORD PTR [rcx+rdi*8]
    de16:	mov    QWORD PTR [r10-0x18],rbp
    de1a:	mov    QWORD PTR [r10-0x10],rbx
    de1e:	mov    rbx,QWORD PTR [rsp+0x18]
    de23:	mov    QWORD PTR [r10-0x8],r11
    de27:	mov    DWORD PTR [r10],esi
    de2a:	mov    DWORD PTR [r10+0x4],0x0
    de32:	prefetcht0 BYTE PTR [r14+rsi*2]
    de37:	inc    rbp
    de3a:	mov    QWORD PTR [rsp+0x8],rbp
    de3f:	mov    QWORD PTR [rsp+rdx*8+0x48],r12
    de44:	mov    ebp,DWORD PTR [rsp+0x24]
    de48:	lea    r12,[rsp+0x250]
    de50:	cmp    rdx,0x1f
    de54:	jb     dc1e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x104e>
    de5a:	jmp    dc00 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1030>
    de5f:	nop
    de60:	mov    QWORD PTR [rsp+0x188],r12
    de68:	mov    r12,QWORD PTR [rsp+0x40]
    de6d:	mov    QWORD PTR [rsp+0x180],rbp
    de75:	mov    rbp,QWORD PTR [rsp+0x30]
    de7a:	jmp    d674 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0xaa4>
    de7f:	test   rdx,rdx
    de82:	je     e25a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x168a>
    de88:	mov    rcx,QWORD PTR [rsp+0x48]
    de8d:	lea    rdi,[rax+0x1]
    de91:	add    rcx,rdi
    de94:	mov    QWORD PTR [rsp+0x240],rcx
    de9c:	cmp    rdx,0x2
    dea0:	mov    r9,QWORD PTR [rsp+0x148]
    dea8:	mov    rsi,QWORD PTR [rsp+0x170]
    deb0:	jb     e27a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16aa>
    deb6:	mov    rcx,QWORD PTR [rsp+0x50]
    debb:	add    rcx,rdi
    debe:	mov    QWORD PTR [rsp+0x260],rcx
    dec6:	cmp    rdx,0x2
    deca:	jbe    e28a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ba>
    ded0:	mov    rcx,QWORD PTR [rsp+0x58]
    ded5:	add    rcx,rdi
    ded8:	mov    QWORD PTR [rsp+0x280],rcx
    dee0:	cmp    rdx,0x3
    dee4:	jbe    e29a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ca>
    deea:	mov    rcx,QWORD PTR [rsp+0x60]
    deef:	add    rcx,rdi
    def2:	mov    QWORD PTR [rsp+0x2a0],rcx
    defa:	cmp    rdx,0x4
    defe:	jbe    e2aa <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16da>
    df04:	mov    rcx,QWORD PTR [rsp+0x68]
    df09:	add    rcx,rdi
    df0c:	mov    QWORD PTR [rsp+0x2c0],rcx
    df14:	cmp    rdx,0x5
    df18:	jbe    e2ba <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16ea>
    df1e:	mov    rcx,QWORD PTR [rsp+0x70]
    df23:	add    rcx,rdi
    df26:	mov    QWORD PTR [rsp+0x2e0],rcx
    df2e:	cmp    rdx,0x6
    df32:	jbe    e2ca <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x16fa>
    df38:	mov    rcx,QWORD PTR [rsp+0x78]
    df3d:	add    rcx,rdi
    df40:	mov    QWORD PTR [rsp+0x300],rcx
    df48:	cmp    rdx,0x7
    df4c:	jbe    e2dd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x170d>
    df52:	mov    rcx,QWORD PTR [rsp+0x80]
    df5a:	add    rcx,rdi
    df5d:	mov    QWORD PTR [rsp+0x320],rcx
    df65:	cmp    rdx,0x8
    df69:	jbe    e2f0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1720>
    df6f:	mov    rcx,QWORD PTR [rsp+0x88]
    df77:	add    rcx,rdi
    df7a:	mov    QWORD PTR [rsp+0x340],rcx
    df82:	cmp    rdx,0x9
    df86:	jbe    e303 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1733>
    df8c:	mov    rcx,QWORD PTR [rsp+0x90]
    df94:	add    rcx,rdi
    df97:	mov    QWORD PTR [rsp+0x360],rcx
    df9f:	cmp    rdx,0xa
    dfa3:	jbe    e316 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1746>
    dfa9:	mov    rcx,QWORD PTR [rsp+0x98]
    dfb1:	add    rcx,rdi
    dfb4:	mov    QWORD PTR [rsp+0x380],rcx
    dfbc:	cmp    rdx,0xb
    dfc0:	jbe    e329 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1759>
    dfc6:	mov    rcx,QWORD PTR [rsp+0xa0]
    dfce:	add    rcx,rdi
    dfd1:	mov    QWORD PTR [rsp+0x3a0],rcx
    dfd9:	cmp    rdx,0xc
    dfdd:	jbe    e33c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x176c>
    dfe3:	mov    rcx,QWORD PTR [rsp+0xa8]
    dfeb:	add    rcx,rdi
    dfee:	mov    QWORD PTR [rsp+0x3c0],rcx
    dff6:	cmp    rdx,0xd
    dffa:	jbe    e34f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x177f>
    e000:	mov    rcx,QWORD PTR [rsp+0xb0]
    e008:	lea    rdi,[rax+0x1]
    e00c:	add    rcx,rdi
    e00f:	mov    QWORD PTR [rsp+0x3e0],rcx
    e017:	cmp    rdx,0xe
    e01b:	jbe    e362 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1792>
    e021:	mov    rcx,QWORD PTR [rsp+0xb8]
    e029:	lea    rdi,[rax+0x1]
    e02d:	add    rcx,rdi
    e030:	mov    QWORD PTR [rsp+0x400],rcx
    e038:	cmp    rdx,0xf
    e03c:	jbe    e375 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17a5>
    e042:	mov    rcx,QWORD PTR [rsp+0xc0]
    e04a:	lea    rdi,[rax+0x1]
    e04e:	add    rcx,rdi
    e051:	mov    QWORD PTR [rsp+0x420],rcx
    e059:	cmp    rdx,0x10
    e05d:	jbe    e388 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17b8>
    e063:	mov    rcx,QWORD PTR [rsp+0xc8]
    e06b:	lea    rdi,[rax+0x1]
    e06f:	add    rcx,rdi
    e072:	mov    QWORD PTR [rsp+0x440],rcx
    e07a:	cmp    rdx,0x11
    e07e:	jbe    e39b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17cb>
    e084:	mov    rcx,QWORD PTR [rsp+0xd0]
    e08c:	lea    rdi,[rax+0x1]
    e090:	add    rcx,rdi
    e093:	mov    QWORD PTR [rsp+0x460],rcx
    e09b:	cmp    rdx,0x12
    e09f:	jbe    e3ae <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17de>
    e0a5:	mov    rcx,QWORD PTR [rsp+0xd8]
    e0ad:	lea    rdi,[rax+0x1]
    e0b1:	add    rcx,rdi
    e0b4:	mov    QWORD PTR [rsp+0x480],rcx
    e0bc:	cmp    rdx,0x13
    e0c0:	jbe    e3c1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x17f1>
    e0c6:	mov    rcx,QWORD PTR [rsp+0xe0]
    e0ce:	lea    rdi,[rax+0x1]
    e0d2:	add    rcx,rdi
    e0d5:	mov    QWORD PTR [rsp+0x4a0],rcx
    e0dd:	cmp    rdx,0x14
    e0e1:	jbe    e3d4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1804>
    e0e7:	mov    rcx,QWORD PTR [rsp+0xe8]
    e0ef:	lea    rdi,[rax+0x1]
    e0f3:	add    rcx,rdi
    e0f6:	mov    QWORD PTR [rsp+0x4c0],rcx
    e0fe:	cmp    rdx,0x15
    e102:	jbe    e3e7 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1817>
    e108:	mov    rcx,QWORD PTR [rsp+0xf0]
    e110:	lea    rdi,[rax+0x1]
    e114:	add    rcx,rdi
    e117:	mov    QWORD PTR [rsp+0x4e0],rcx
    e11f:	cmp    rdx,0x16
    e123:	jbe    e3fa <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x182a>
    e129:	mov    rcx,QWORD PTR [rsp+0xf8]
    e131:	lea    rdi,[rax+0x1]
    e135:	add    rcx,rdi
    e138:	mov    QWORD PTR [rsp+0x500],rcx
    e140:	cmp    rdx,0x17
    e144:	jbe    e40d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x183d>
    e14a:	mov    rcx,QWORD PTR [rsp+0x100]
    e152:	lea    rdi,[rax+0x1]
    e156:	add    rcx,rdi
    e159:	mov    QWORD PTR [rsp+0x520],rcx
    e161:	cmp    rdx,0x18
    e165:	jbe    e420 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1850>
    e16b:	mov    rcx,QWORD PTR [rsp+0x108]
    e173:	lea    rdi,[rax+0x1]
    e177:	add    rcx,rdi
    e17a:	mov    QWORD PTR [rsp+0x540],rcx
    e182:	cmp    rdx,0x19
    e186:	jbe    e433 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1863>
    e18c:	mov    rcx,QWORD PTR [rsp+0x110]
    e194:	lea    rdi,[rax+0x1]
    e198:	add    rcx,rdi
    e19b:	mov    QWORD PTR [rsp+0x560],rcx
    e1a3:	cmp    rdx,0x1a
    e1a7:	jbe    e446 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1876>
    e1ad:	mov    rcx,QWORD PTR [rsp+0x118]
    e1b5:	lea    rdi,[rax+0x1]
    e1b9:	add    rcx,rdi
    e1bc:	mov    QWORD PTR [rsp+0x580],rcx
    e1c4:	cmp    rdx,0x1b
    e1c8:	jbe    e459 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1889>
    e1ce:	mov    rcx,QWORD PTR [rsp+0x120]
    e1d6:	lea    rdi,[rax+0x1]
    e1da:	add    rcx,rdi
    e1dd:	mov    QWORD PTR [rsp+0x5a0],rcx
    e1e5:	cmp    rdx,0x1c
    e1e9:	jbe    e46c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x189c>
    e1ef:	mov    rcx,QWORD PTR [rsp+0x128]
    e1f7:	lea    rdi,[rax+0x1]
    e1fb:	add    rcx,rdi
    e1fe:	mov    QWORD PTR [rsp+0x5c0],rcx
    e206:	cmp    rdx,0x1d
    e20a:	jbe    e47f <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18af>
    e210:	mov    rcx,QWORD PTR [rsp+0x130]
    e218:	lea    rdi,[rax+0x1]
    e21c:	add    rcx,rdi
    e21f:	mov    QWORD PTR [rsp+0x5e0],rcx
    e227:	cmp    rdx,0x1e
    e22b:	jbe    e492 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
    e231:	mov    rax,QWORD PTR [rsp+0x138]
    e239:	mov    rcx,QWORD PTR [rsp]
    e23d:	add    rax,rcx
    e240:	mov    QWORD PTR [rsp+0x600],rax
    e248:	mov    rax,rcx
    e24b:	cmp    rdx,0x20
    e24f:	jae    e492 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18c2>
    e255:	jmp    e4a2 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x18d2>
    e25a:	mov    rcx,QWORD PTR [rsp+0x50]
    e25f:	add    rcx,rax
    e262:	mov    QWORD PTR [rsp+0x260],rcx
    e26a:	mov    r9,QWORD PTR [rsp+0x148]
    e272:	mov    rsi,QWORD PTR [rsp+0x170]
    e27a:	mov    rcx,QWORD PTR [rsp+0x58]
    e27f:	add    rcx,rax
    e282:	mov    QWORD PTR [rsp+0x280],rcx
    e28a:	mov    rcx,QWORD PTR [rsp+0x60]
    e28f:	add    rcx,rax
    e292:	mov    QWORD PTR [rsp+0x2a0],rcx
    e29a:	mov    rcx,QWORD PTR [rsp+0x68]
    e29f:	add    rcx,rax
    e2a2:	mov    QWORD PTR [rsp+0x2c0],rcx
    e2aa:	mov    rcx,QWORD PTR [rsp+0x70]
    e2af:	add    rcx,rax
    e2b2:	mov    QWORD PTR [rsp+0x2e0],rcx
    e2ba:	mov    rcx,QWORD PTR [rsp+0x78]
    e2bf:	add    rcx,rax
    e2c2:	mov    QWORD PTR [rsp+0x300],rcx
    e2ca:	mov    rcx,QWORD PTR [rsp+0x80]
    e2d2:	add    rcx,rax
    e2d5:	mov    QWORD PTR [rsp+0x320],rcx
    e2dd:	mov    rcx,QWORD PTR [rsp+0x88]
    e2e5:	add    rcx,rax
    e2e8:	mov    QWORD PTR [rsp+0x340],rcx
    e2f0:	mov    rcx,QWORD PTR [rsp+0x90]
    e2f8:	add    rcx,rax
    e2fb:	mov    QWORD PTR [rsp+0x360],rcx
    e303:	mov    rcx,QWORD PTR [rsp+0x98]
    e30b:	add    rcx,rax
    e30e:	mov    QWORD PTR [rsp+0x380],rcx
    e316:	mov    rcx,QWORD PTR [rsp+0xa0]
    e31e:	add    rcx,rax
    e321:	mov    QWORD PTR [rsp+0x3a0],rcx
    e329:	mov    rcx,QWORD PTR [rsp+0xa8]
    e331:	add    rcx,rax
    e334:	mov    QWORD PTR [rsp+0x3c0],rcx
    e33c:	mov    rcx,QWORD PTR [rsp+0xb0]
    e344:	add    rcx,rax
    e347:	mov    QWORD PTR [rsp+0x3e0],rcx
    e34f:	mov    rcx,QWORD PTR [rsp+0xb8]
    e357:	add    rcx,rax
    e35a:	mov    QWORD PTR [rsp+0x400],rcx
    e362:	mov    rcx,QWORD PTR [rsp+0xc0]
    e36a:	add    rcx,rax
    e36d:	mov    QWORD PTR [rsp+0x420],rcx
    e375:	mov    rcx,QWORD PTR [rsp+0xc8]
    e37d:	add    rcx,rax
    e380:	mov    QWORD PTR [rsp+0x440],rcx
    e388:	mov    rcx,QWORD PTR [rsp+0xd0]
    e390:	add    rcx,rax
    e393:	mov    QWORD PTR [rsp+0x460],rcx
    e39b:	mov    rcx,QWORD PTR [rsp+0xd8]
    e3a3:	add    rcx,rax
    e3a6:	mov    QWORD PTR [rsp+0x480],rcx
    e3ae:	mov    rcx,QWORD PTR [rsp+0xe0]
    e3b6:	add    rcx,rax
    e3b9:	mov    QWORD PTR [rsp+0x4a0],rcx
    e3c1:	mov    rcx,QWORD PTR [rsp+0xe8]
    e3c9:	add    rcx,rax
    e3cc:	mov    QWORD PTR [rsp+0x4c0],rcx
    e3d4:	mov    rcx,QWORD PTR [rsp+0xf0]
    e3dc:	add    rcx,rax
    e3df:	mov    QWORD PTR [rsp+0x4e0],rcx
    e3e7:	mov    rcx,QWORD PTR [rsp+0xf8]
    e3ef:	add    rcx,rax
    e3f2:	mov    QWORD PTR [rsp+0x500],rcx
    e3fa:	mov    rcx,QWORD PTR [rsp+0x100]
    e402:	add    rcx,rax
    e405:	mov    QWORD PTR [rsp+0x520],rcx
    e40d:	mov    rcx,QWORD PTR [rsp+0x108]
    e415:	add    rcx,rax
    e418:	mov    QWORD PTR [rsp+0x540],rcx
    e420:	mov    rcx,QWORD PTR [rsp+0x110]
    e428:	add    rcx,rax
    e42b:	mov    QWORD PTR [rsp+0x560],rcx
    e433:	mov    rcx,QWORD PTR [rsp+0x118]
    e43b:	add    rcx,rax
    e43e:	mov    QWORD PTR [rsp+0x580],rcx
    e446:	mov    rcx,QWORD PTR [rsp+0x120]
    e44e:	add    rcx,rax
    e451:	mov    QWORD PTR [rsp+0x5a0],rcx
    e459:	mov    rcx,QWORD PTR [rsp+0x128]
    e461:	add    rcx,rax
    e464:	mov    QWORD PTR [rsp+0x5c0],rcx
    e46c:	mov    rcx,QWORD PTR [rsp+0x130]
    e474:	add    rcx,rax
    e477:	mov    QWORD PTR [rsp+0x5e0],rcx
    e47f:	mov    rcx,QWORD PTR [rsp+0x138]
    e487:	add    rcx,rax
    e48a:	mov    QWORD PTR [rsp+0x600],rcx
    e492:	add    rax,QWORD PTR [rsp+0x140]
    e49a:	mov    QWORD PTR [rsp+0x620],rax
    e4a2:	shl    rsi,0x5
    e4a6:	lea    rax,[rsp+rsi*1]
    e4aa:	add    rax,0x238
    e4b0:	mov    QWORD PTR [rsp],rax
    e4b4:	mov    rax,QWORD PTR [rbx+0x68]
    e4b8:	mov    rsi,QWORD PTR [rbx+0x70]
    e4bc:	lea    r12,[rsp+0x238]
    e4c4:	jmp    e4dd <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x190d>
    e4c6:	cs nop WORD PTR [rax+rax*1+0x0]
    e4d0:	mov    r12,r13
    e4d3:	add    r9,0xffffffffffffffe0
    e4d7:	je     e61b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a4b>
    e4dd:	lea    r13,[r12+0x20]
    e4e2:	mov    r15,QWORD PTR [r12]
    e4e6:	cmp    r15,0xffffffffffffffff
    e4ea:	je     e4d0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
    e4ec:	cmp    r15,rsi
    e4ef:	jae    ebc1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ff1>
    e4f5:	cmp    QWORD PTR [rax+r15*8],0xffffffffffffffff
    e4fa:	jne    e4d0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1900>
    e4fc:	mov    edi,0x80
    e501:	call   QWORD PTR [rip+0x839f9]        # 91f00 <malloc@GLIBC_2.2.5>
    e507:	test   rax,rax
    e50a:	je     ec25 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2055>
    e510:	mov    rcx,QWORD PTR [r12+0x18]
    e515:	mov    QWORD PTR [rax+0x18],rcx
    e519:	movups xmm0,XMMWORD PTR [r12+0x8]
    e51f:	movups XMMWORD PTR [rax+0x8],xmm0
    e523:	mov    QWORD PTR [rax],r15
    e526:	mov    QWORD PTR [rsp+0x158],0x4
    e532:	mov    QWORD PTR [rsp+0x160],rax
    e53a:	mov    QWORD PTR [rsp+0x168],0x1
    e546:	cmp    r13,QWORD PTR [rsp]
    e54a:	je     e62d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a5d>
    e550:	mov    r12d,0x1
    e556:	lea    r15,[rsp+0x158]
    e55e:	mov    rcx,QWORD PTR [rbx+0x68]
    e562:	mov    rsi,QWORD PTR [rbx+0x70]
    e566:	mov    rdx,r13
    e569:	jmp    e57d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19ad>
    e56b:	nop    DWORD PTR [rax+rax*1+0x0]
    e570:	mov    rdx,r13
    e573:	cmp    r13,QWORD PTR [rsp]
    e577:	je     e633 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
    e57d:	lea    r13,[rdx+0x20]
    e581:	mov    rbp,QWORD PTR [rdx]
    e584:	cmp    rbp,0xffffffffffffffff
    e588:	je     e570 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
    e58a:	cmp    rbp,rsi
    e58d:	jae    ebd0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2000>
    e593:	cmp    QWORD PTR [rcx+rbp*8],0xffffffffffffffff
    e598:	jne    e570 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x19a0>
    e59a:	mov    rcx,QWORD PTR [rdx+0x18]
    e59e:	mov    QWORD PTR [rsp+0x220],rcx
    e5a6:	movups xmm0,XMMWORD PTR [rdx+0x8]
    e5aa:	movaps XMMWORD PTR [rsp+0x210],xmm0
    e5b2:	cmp    r12,QWORD PTR [rsp+0x158]
    e5ba:	jne    e5df <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a0f>
    e5bc:	mov    edx,0x1
    e5c1:	mov    ecx,0x8
    e5c6:	mov    r8d,0x20
    e5cc:	mov    rdi,r15
    e5cf:	mov    rsi,r12
    e5d2:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    e5d7:	mov    rax,QWORD PTR [rsp+0x160]
    e5df:	mov    rcx,r12
    e5e2:	shl    rcx,0x5
    e5e6:	mov    QWORD PTR [rax+rcx*1],rbp
    e5ea:	movaps xmm0,XMMWORD PTR [rsp+0x210]
    e5f2:	movups XMMWORD PTR [rax+rcx*1+0x8],xmm0
    e5f7:	mov    rdx,QWORD PTR [rsp+0x220]
    e5ff:	mov    QWORD PTR [rax+rcx*1+0x18],rdx
    e604:	inc    r12
    e607:	mov    QWORD PTR [rsp+0x168],r12
    e60f:	cmp    r13,QWORD PTR [rsp]
    e613:	jne    e55e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x198e>
    e619:	jmp    e633 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1a63>
    e61b:	mov    eax,0x8
    e620:	mov    QWORD PTR [rsp+0x8],rax
    e625:	xor    r14d,r14d
    e628:	jmp    e941 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d71>
    e62d:	mov    r12d,0x1
    e633:	mov    rax,QWORD PTR [rsp+0x158]
    e63b:	mov    QWORD PTR [rsp+0x10],rax
    e640:	mov    rax,QWORD PTR [rsp+0x160]
    e648:	mov    QWORD PTR [rsp+0x8],rax
    e64d:	mov    r15,r12
    e650:	shl    r15,0x5
    e654:	xor    r13d,r13d
    e657:	lea    rax,[rip+0x80a72]        # 8f0d0 <__do_global_dtors_aux_fini_array_entry+0x348>
    e65e:	mov    QWORD PTR [rsp+0x30],rax
    e663:	jmp    e68e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1abe>
    e665:	data16 cs nop WORD PTR [rax+rax*1+0x0]
    e670:	add    rbp,rax
    e673:	mov    rcx,QWORD PTR [rsp+0x8]
    e678:	mov    QWORD PTR [rcx+r13*1+0x8],rbp
    e67d:	add    rdx,rax
    e680:	mov    QWORD PTR [rcx+r13*1+0x10],rdx
    e685:	add    r13,0x20
    e689:	cmp    r15,r13
    e68c:	je     e704 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b34>
    e68e:	mov    rax,QWORD PTR [rsp+0x8]
    e693:	mov    rbp,QWORD PTR [rax+r13*1+0x8]
    e698:	cmp    rbp,QWORD PTR [rax+r13*1+0x10]
    e69d:	jne    e685 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ab5>
    e69f:	mov    rsi,QWORD PTR [rbx+0x58]
    e6a3:	mov    rax,QWORD PTR [rsp+0x8]
    e6a8:	mov    rdi,QWORD PTR [rax+r13*1]
    e6ac:	cmp    rdi,rsi
    e6af:	jae    ebb0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
    e6b5:	mov    rcx,QWORD PTR [rbx+0x50]
    e6b9:	lea    rsi,[rdi+rdi*2]
    e6bd:	mov    rdx,QWORD PTR [rcx+rsi*8]
    e6c1:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
    e6c6:	sub    rbp,rax
    e6c9:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
    e6ce:	mov    rdi,rdx
    e6d1:	sub    rdi,rbp
    e6d4:	cmp    rdx,rdi
    e6d7:	jbe    e670 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
    e6d9:	lea    rdi,[rcx+rsi*8]
    e6dd:	mov    ecx,0x1
    e6e2:	mov    r8d,0x1
    e6e8:	mov    QWORD PTR [rsp],rdi
    e6ec:	mov    rsi,rbp
    e6ef:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    e6f4:	mov    rax,QWORD PTR [rsp]
    e6f8:	mov    rdx,QWORD PTR [rax]
    e6fb:	mov    rax,QWORD PTR [rax+0x8]
    e6ff:	jmp    e670 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1aa0>
    e704:	mov    rax,QWORD PTR [rsp+0x8]
    e709:	add    rax,0x18
    e70d:	mov    QWORD PTR [rsp],rax
    e711:	xor    r15d,r15d
    e714:	jmp    e723 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
    e716:	cs nop WORD PTR [rax+rax*1+0x0]
    e720:	xor    r15d,r15d
    e723:	mov    r13,r15
    e726:	shl    r13,0x5
    e72a:	add    r13,QWORD PTR [rsp]
    e72e:	jmp    e74a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b7a>
    e730:	add    rbp,rax
    e733:	mov    QWORD PTR [r13-0x10],rbp
    e737:	add    rdx,rax
    e73a:	mov    QWORD PTR [r13-0x8],rdx
    e73e:	inc    r15
    e741:	add    r13,0x20
    e745:	cmp    r15,r12
    e748:	jae    e720 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b50>
    e74a:	lea    rax,[r13-0x18]
    e74e:	mov    ecx,DWORD PTR [r13+0x0]
    e752:	movzx  esi,WORD PTR [r14+rcx*2]
    e757:	mov    edx,ecx
    e759:	and    edx,0xffffffe0
    e75c:	mov    edx,DWORD PTR [r14+rdx*2+0x3c]
    e761:	cmp    esi,0xfffe
    e767:	jae    e780 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bb0>
    e769:	mov    ecx,edx
    e76b:	shr    ecx,0x8
    e76e:	add    ecx,esi
    e770:	jmp    e7a1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1bd1>
    e772:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
    e780:	je     e830 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1c60>
    e786:	cmp    ecx,DWORD PTR [rsp+0x24]
    e78a:	jne    e8c0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cf0>
    e790:	mov    ecx,DWORD PTR [r13+0x4]
    e794:	mov    rdx,QWORD PTR [rsp+0x38]
    e799:	mov    edx,DWORD PTR [rdx+rcx*4]
    e79c:	mov    ecx,edx
    e79e:	shr    ecx,0x8
    e7a1:	mov    DWORD PTR [r13+0x0],ecx
    e7a5:	mov    ecx,ecx
    e7a7:	prefetcht0 BYTE PTR [r14+rcx*2]
    e7ac:	mov    rcx,QWORD PTR [r13-0x10]
    e7b0:	mov    BYTE PTR [rcx],dl
    e7b2:	mov    rbp,QWORD PTR [r13-0x10]
    e7b6:	inc    rbp
    e7b9:	mov    QWORD PTR [r13-0x10],rbp
    e7bd:	cmp    rbp,QWORD PTR [r13-0x8]
    e7c1:	jne    e73e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b6e>
    e7c7:	mov    rsi,QWORD PTR [rbx+0x58]
    e7cb:	mov    rdi,QWORD PTR [rax]
    e7ce:	cmp    rdi,rsi
    e7d1:	jae    ebb0 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fe0>
    e7d7:	mov    rcx,QWORD PTR [rbx+0x50]
    e7db:	lea    rsi,[rdi+rdi*2]
    e7df:	mov    rdx,QWORD PTR [rcx+rsi*8]
    e7e3:	mov    rax,QWORD PTR [rcx+rsi*8+0x8]
    e7e8:	sub    rbp,rax
    e7eb:	mov    QWORD PTR [rcx+rsi*8+0x10],rbp
    e7f0:	mov    rdi,rdx
    e7f3:	sub    rdi,rbp
    e7f6:	cmp    rdx,rdi
    e7f9:	jbe    e730 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
    e7ff:	lea    rdi,[rcx+rsi*8]
    e803:	mov    ecx,0x1
    e808:	mov    r8d,0x1
    e80e:	mov    QWORD PTR [rsp+0x40],rdi
    e813:	mov    rsi,rbp
    e816:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    e81b:	mov    rax,QWORD PTR [rsp+0x40]
    e820:	mov    rdx,QWORD PTR [rax]
    e823:	mov    rax,QWORD PTR [rax+0x8]
    e827:	jmp    e730 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b60>
    e82c:	nop    DWORD PTR [rax+0x0]
    e830:	mov    rsi,QWORD PTR [rbx+0x58]
    e834:	mov    rdi,QWORD PTR [rax]
    e837:	cmp    rdi,rsi
    e83a:	jae    eba4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fd4>
    e840:	mov    r8,QWORD PTR [rbx+0x50]
    e844:	mov    rdx,QWORD PTR [rbx+0x68]
    e848:	lea    r9,[rdi+rdi*2]
    e84c:	mov    r10,QWORD PTR [rax+0x8]
    e850:	sub    r10,QWORD PTR [r8+r9*8+0x8]
    e855:	mov    rsi,QWORD PTR [rbx+0x70]
    e859:	mov    QWORD PTR [r8+r9*8+0x10],r10
    e85e:	mov    r10,QWORD PTR [rbx+0x40]
    e862:	test   r10,r10
    e865:	je     e8f1 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d21>
    e86b:	mov    r9,QWORD PTR [rbx+0x38]
    e86f:	xor    r8d,r8d
    e872:	cmp    r10,0x1
    e876:	je     e89d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1ccd>
    e878:	nop    DWORD PTR [rax+rax*1+0x0]
    e880:	mov    r11,r8
    e883:	mov    rbx,r10
    e886:	shr    rbx,1
    e889:	add    r8,rbx
    e88c:	cmp    DWORD PTR [r9+r8*4],ecx
    e890:	cmova  r8,r11
    e894:	sub    r10,rbx
    e897:	cmp    r10,0x1
    e89b:	ja     e880 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1cb0>
    e89d:	cmp    DWORD PTR [r9+r8*4],ecx
    e8a1:	mov    rcx,0xffffffffffffffff
    e8a8:	cmovne r8,rcx
    e8ac:	mov    rbx,QWORD PTR [rsp+0x18]
    e8b1:	cmp    rdi,rsi
    e8b4:	jb     e901 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d31>
    e8b6:	jmp    eb9b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
    e8bb:	nop    DWORD PTR [rax+rax*1+0x0]
    e8c0:	shl    edx,0x5
    e8c3:	and    edx,0x1ffffe0
    e8c9:	and    ecx,0x1f
    e8cc:	or     ecx,edx
    e8ce:	mov    DWORD PTR [r13+0x4],ecx
    e8d2:	mov    rax,QWORD PTR [rsp+0x38]
    e8d7:	prefetcht0 BYTE PTR [rax+rcx*4]
    e8db:	mov    eax,DWORD PTR [rsp+0x24]
    e8df:	mov    DWORD PTR [r13+0x0],eax
    e8e3:	inc    r15
    e8e6:	cmp    r15,r12
    e8e9:	jb     e723 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
    e8ef:	jmp    e92d <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1d5d>
    e8f1:	mov    r8,0xffffffffffffffff
    e8f8:	cmp    rdi,rsi
    e8fb:	jae    eb9b <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fcb>
    e901:	mov    QWORD PTR [rdx+rdi*8],r8
    e905:	dec    r12
    e908:	mov    rcx,r12
    e90b:	shl    rcx,0x5
    e90f:	mov    rdx,QWORD PTR [rsp+0x8]
    e914:	movups xmm0,XMMWORD PTR [rdx+rcx*1]
    e918:	movups xmm1,XMMWORD PTR [rdx+rcx*1+0x10]
    e91d:	movups XMMWORD PTR [rax+0x10],xmm1
    e921:	movups XMMWORD PTR [rax],xmm0
    e924:	cmp    r15,r12
    e927:	jb     e723 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
    e92d:	mov    r15d,0x0
    e933:	test   r12,r12
    e936:	jne    e723 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1b53>
    e93c:	mov    r14,QWORD PTR [rsp+0x10]
    e941:	xor    eax,eax
    e943:	lea    rcx,[rip+0x80476]        # 8edc0 <__do_global_dtors_aux_fini_array_entry+0x38>
    e94a:	lea    r8,[rip+0x80487]        # 8edd8 <__do_global_dtors_aux_fini_array_entry+0x50>
    e951:	lea    r9,[rip+0x80498]        # 8edf0 <__do_global_dtors_aux_fini_array_entry+0x68>
    e958:	lea    rdx,[rip+0x804a9]        # 8ee08 <__do_global_dtors_aux_fini_array_entry+0x80>
    e95f:	mov    r11,QWORD PTR [rsp+0x28]
    e964:	data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
    e970:	mov    rsi,QWORD PTR [rbx+0x40]
    e974:	cmp    rax,rsi
    e977:	jae    eb3a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f6a>
    e97d:	mov    rsi,QWORD PTR [rbx+0x98]
    e984:	cmp    rax,rsi
    e987:	jae    eb32 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f62>
    e98d:	mov    rsi,QWORD PTR [rbx+0x38]
    e991:	mov    edi,DWORD PTR [rsi+rax*4]
    e994:	mov    rsi,QWORD PTR [rbx+0x10]
    e998:	mov    r10d,edi
    e99b:	shr    r10d,0x5
    e99f:	cmp    rsi,r10
    e9a2:	jbe    eb2a <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f5a>
    e9a8:	and    edi,0x1f
    e9ab:	cmp    edi,0x1d
    e9ae:	ja     eb23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f53>
    e9b4:	mov    rsi,QWORD PTR [rbx+0x90]
    e9bb:	movzx  esi,WORD PTR [rsi+rax*2]
    e9bf:	inc    rax
    e9c2:	shl    r10,0x6
    e9c6:	add    r10,QWORD PTR [rbx+0x8]
    e9ca:	mov    WORD PTR [r10+rdi*2],si
    e9cf:	cmp    r11,rax
    e9d2:	jne    e970 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1da0>
    e9d4:	mov    rax,QWORD PTR [rbx+0x38]
    e9d8:	mov    rcx,QWORD PTR [rbx+0x40]
    e9dc:	mov    edi,DWORD PTR [rsp+0x154]
    e9e3:	mov    esi,edi
    e9e5:	mov    edx,0x88888889
    e9ea:	imul   rdx,rsi
    e9ee:	shr    rdx,0x23
    e9f2:	and    edx,0xfffffffe
    e9f5:	add    edx,edi
    e9f7:	cmp    rcx,0x1
    e9fb:	je     ea2e <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e5e>
    e9fd:	test   rcx,rcx
    ea00:	je     ebf4 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2024>
    ea06:	xor    esi,esi
    ea08:	nop    DWORD PTR [rax+rax*1+0x0]
    ea10:	mov    rdi,rsi
    ea13:	mov    r8,rcx
    ea16:	shr    r8,1
    ea19:	add    rsi,r8
    ea1c:	cmp    DWORD PTR [rax+rsi*4],edx
    ea1f:	cmova  rsi,rdi
    ea23:	sub    rcx,r8
    ea26:	cmp    rcx,0x1
    ea2a:	ja     ea10 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e40>
    ea2c:	jmp    ea30 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e60>
    ea2e:	xor    esi,esi
    ea30:	cmp    DWORD PTR [rax+rsi*4],edx
    ea33:	jne    ebed <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x201d>
    ea39:	mov    QWORD PTR [rbx+0x80],rsi
    ea40:	mov    rax,QWORD PTR [rsp+0x178]
    ea48:	mov    QWORD PTR [rbx+0x78],rax
    ea4c:	test   r14,r14
    ea4f:	je     ea5c <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1e8c>
    ea51:	mov    rdi,QWORD PTR [rsp+0x8]
    ea56:	call   QWORD PTR [rip+0x8332c]        # 91d88 <free@GLIBC_2.2.5>
    ea5c:	add    rsp,0x638
    ea63:	pop    rbx
    ea64:	pop    r12
    ea66:	pop    r13
    ea68:	pop    r14
    ea6a:	pop    r15
    ea6c:	pop    rbp
    ea6d:	ret
    ea6e:	lea    rcx,[rip+0x803ab]        # 8ee20 <__do_global_dtors_aux_fini_array_entry+0x98>
    ea75:	xor    edi,edi
    ea77:	call   6a70 <_RNvNtNtCscI6d9CVNmLh_4core5slice5index16slice_index_fail>
    ea7c:	lea    rdi,[rbx+0x60]
    ea80:	mov    ecx,0x8
    ea85:	mov    r8d,0x8
    ea8b:	xor    esi,esi
    ea8d:	mov    rdx,r12
    ea90:	call   5100 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECs3AlRknvjqqN_7c30walk>
    ea95:	mov    r15,QWORD PTR [rbx+0x70]
    ea99:	jmp    cfd6 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x406>
    ea9e:	lea    rdx,[rip+0x803f3]        # 8ee98 <__do_global_dtors_aux_fini_array_entry+0x110>
    eaa5:	mov    rdi,r12
    eaa8:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eaad:	lea    rdx,[rip+0x80eec]        # 8f9a0 <__do_global_dtors_aux_fini_array_entry+0xc18>
    eab4:	mov    esi,0x1e
    eab9:	mov    rdi,rbp
    eabc:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eac1:	lea    rdx,[rip+0x80ec0]        # 8f988 <__do_global_dtors_aux_fini_array_entry+0xc00>
    eac8:	mov    rdi,r12
    eacb:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    ead0:	lea    rdx,[rip+0x803a9]        # 8ee80 <__do_global_dtors_aux_fini_array_entry+0xf8>
    ead7:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eadc:	lea    rdx,[rip+0x80385]        # 8ee68 <__do_global_dtors_aux_fini_array_entry+0xe0>
    eae3:	mov    esi,0x1e
    eae8:	mov    rdi,rax
    eaeb:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eaf0:	lea    rdx,[rip+0x80359]        # 8ee50 <__do_global_dtors_aux_fini_array_entry+0xc8>
    eaf7:	mov    rdi,rcx
    eafa:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eaff:	lea    rdx,[rip+0x80332]        # 8ee38 <__do_global_dtors_aux_fini_array_entry+0xb0>
    eb06:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb0b:	lea    rdx,[rip+0x80576]        # 8f088 <__do_global_dtors_aux_fini_array_entry+0x300>
    eb12:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb17:	lea    rdx,[rip+0x80552]        # 8f070 <__do_global_dtors_aux_fini_array_entry+0x2e8>
    eb1e:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb23:	mov    esi,0x1e
    eb28:	jmp    eb40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
    eb2a:	mov    rdi,r10
    eb2d:	mov    rdx,r9
    eb30:	jmp    eb40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
    eb32:	mov    rdi,rax
    eb35:	mov    rdx,r8
    eb38:	jmp    eb40 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1f70>
    eb3a:	mov    rdi,rax
    eb3d:	mov    rdx,rcx
    eb40:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb45:	jmp    ec23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
    eb4a:	lea    rdx,[rip+0x8057f]        # 8f0d0 <__do_global_dtors_aux_fini_array_entry+0x348>
    eb51:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb56:	lea    rdx,[rip+0x80e5b]        # 8f9b8 <__do_global_dtors_aux_fini_array_entry+0xc30>
    eb5d:	mov    rdi,r13
    eb60:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb65:	lea    rdx,[rip+0x8054c]        # 8f0b8 <__do_global_dtors_aux_fini_array_entry+0x330>
    eb6c:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb71:	lea    rdx,[rip+0x80528]        # 8f0a0 <__do_global_dtors_aux_fini_array_entry+0x318>
    eb78:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb7d:	lea    rdx,[rip+0x80504]        # 8f088 <__do_global_dtors_aux_fini_array_entry+0x300>
    eb84:	mov    rdi,rbp
    eb87:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb8c:	lea    rdx,[rip+0x804dd]        # 8f070 <__do_global_dtors_aux_fini_array_entry+0x2e8>
    eb93:	mov    rdi,rbp
    eb96:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    eb9b:	lea    rax,[rip+0x80516]        # 8f0b8 <__do_global_dtors_aux_fini_array_entry+0x330>
    eba2:	jmp    ebab <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x1fdb>
    eba4:	lea    rax,[rip+0x804f5]        # 8f0a0 <__do_global_dtors_aux_fini_array_entry+0x318>
    ebab:	mov    QWORD PTR [rsp+0x30],rax
    ebb0:	mov    rdx,QWORD PTR [rsp+0x30]
    ebb5:	mov    r14,QWORD PTR [rsp+0x10]
    ebba:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    ebbf:	jmp    ec23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
    ebc1:	lea    rdx,[rip+0x80520]        # 8f0e8 <__do_global_dtors_aux_fini_array_entry+0x360>
    ebc8:	mov    rdi,r15
    ebcb:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    ebd0:	lea    rdx,[rip+0x80511]        # 8f0e8 <__do_global_dtors_aux_fini_array_entry+0x360>
    ebd7:	mov    rdi,rbp
    ebda:	call   69fb <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
    ebdf:	jmp    ec23 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2053>
    ebe1:	lea    rdi,[rip+0x801a8]        # 8ed90 <__do_global_dtors_aux_fini_array_entry+0x8>
    ebe8:	call   6d00 <_RNvNtNtCscI6d9CVNmLh_4core9panicking11panic_const23panic_const_div_by_zero>
    ebed:	adc    rsi,0x0
    ebf1:	mov    rcx,rsi
    ebf4:	mov    QWORD PTR [rsp+0x210],rcx
    ebfc:	lea    rdi,[rip+0x70b04]        # 7f707 <_fini+0x570b>
    ec03:	lea    rcx,[rip+0x80d1e]        # 8f928 <__do_global_dtors_aux_fini_array_entry+0xba0>
    ec0a:	lea    r8,[rip+0x80197]        # 8eda8 <__do_global_dtors_aux_fini_array_entry+0x20>
    ec11:	lea    rdx,[rsp+0x210]
    ec19:	mov    esi,0x19
    ec1e:	call   6c00 <_RNvNtCscI6d9CVNmLh_4core6result13unwrap_failed>
    ec23:	ud2
    ec25:	mov    edi,0x8
    ec2a:	mov    esi,0x80
    ec2f:	call   6715 <_RNvNtCs40k4W9msRzi_5alloc7raw_vec12handle_error>
    ec34:	jmp    ec38 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2068>
    ec36:	jmp    ec42 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2072>
    ec38:	mov    rbx,rax
    ec3b:	mov    r14,QWORD PTR [rsp+0x10]
    ec40:	jmp    ec62 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2092>
    ec42:	mov    rbx,rax
    ec45:	cmp    QWORD PTR [rsp+0x158],0x0
    ec4e:	je     ec72 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
    ec50:	mov    rax,QWORD PTR [rsp+0x160]
    ec58:	mov    QWORD PTR [rsp+0x8],rax
    ec5d:	jmp    ec67 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x2097>
    ec5f:	mov    rbx,rax
    ec62:	test   r14,r14
    ec65:	je     ec72 <_RINvMs0_Cs3AlRknvjqqN_7c30walkNtB6_3C304roomKb0_EB6_+0x20a2>
    ec67:	mov    rdi,QWORD PTR [rsp+0x8]
    ec6c:	call   QWORD PTR [rip+0x83116]        # 91d88 <free@GLIBC_2.2.5>
    ec72:	mov    rdi,rbx
    ec75:	call   5040 <_Unwind_Resume@plt>
    ec7a:	nop    WORD PTR [rax+rax*1+0x0]
