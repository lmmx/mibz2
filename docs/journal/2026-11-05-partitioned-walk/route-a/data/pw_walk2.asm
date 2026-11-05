
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/target/release/pwsim:     file format elf64-x86-64


Disassembly of section .text:

0000000000017ec0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2>:
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:334
   17ec0:	push   rbp
   17ec1:	push   r15
   17ec3:	push   r14
   17ec5:	push   r13
   17ec7:	push   r12
   17ec9:	push   rbx
   17eca:	sub    rsp,0x448
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   17ed1:	mov    r12,QWORD PTR [rsi+0xb8]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:339
   17ed8:	cmp    r12,0x41
   17edc:	jae    1824e <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x38e>
   17ee2:	mov    r13,rsi
   17ee5:	mov    r14,rdi
   17ee8:	mov    eax,DWORD PTR [rsi+0x150]
   17eee:	mov    DWORD PTR [rsp+0xc],eax
   17ef2:	mov    r15d,DWORD PTR [rsi+0x154]
   17ef9:	mov    rbp,QWORD PTR [rsi+0x88]
   17f00:	lea    rdi,[rsp+0x248]
   17f08:	mov    rbx,QWORD PTR [rip+0x467d1]        # 5e6e0 <memset@GLIBC_2.2.5>
   17f0f:	mov    edx,0x200
   17f14:	xor    esi,esi
   17f16:	call   rbx
   17f18:	lea    rdi,[rsp+0x48]
   17f1d:	mov    edx,0x200
   17f22:	xor    esi,esi
   17f24:	call   rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   17f26:	test   r12,r12
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   17f29:	je     17fcc <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x10c>
spec_next<usize>():
   17f2f:	mov    rcx,QWORD PTR [r13+0xb0]
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   17f36:	cmp    r12,0x4
   17f3a:	ja     17f40 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x80>
   17f3c:	xor    eax,eax
   17f3e:	jmp    17fa0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0xe0>
   17f40:	mov    eax,r12d
   17f43:	and    eax,0x3
   17f46:	mov    edx,0x4
   17f4b:	cmovne rdx,rax
   17f4f:	mov    rax,r12
   17f52:	sub    rax,rdx
   17f55:	lea    rdx,[rcx+0x50]
   17f59:	xor    esi,esi
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
   17f5b:	nop    DWORD PTR [rax+rax*1+0x0]
   17f60:	movsd  xmm0,QWORD PTR [rdx-0x30]
   17f65:	movsd  xmm1,QWORD PTR [rdx-0x48]
   17f6a:	movlhps xmm1,xmm0
   17f6d:	movsd  xmm0,QWORD PTR [rdx]
   17f71:	movsd  xmm2,QWORD PTR [rdx-0x18]
   17f76:	movlhps xmm2,xmm0
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:344
   17f79:	movups XMMWORD PTR [rsp+rsi*8+0x248],xmm1
   17f81:	movups XMMWORD PTR [rsp+rsi*8+0x258],xmm2
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:345
   17f89:	movups XMMWORD PTR [rsp+rsi*8+0x48],xmm1
   17f8e:	movups XMMWORD PTR [rsp+rsi*8+0x58],xmm2
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   17f93:	add    rsi,0x4
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   17f97:	add    rdx,0x60
   17f9b:	cmp    rax,rsi
   17f9e:	jne    17f60 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0xa0>
   17fa0:	lea    rdx,[rax+rax*2]
   17fa4:	lea    rcx,[rcx+rdx*8]
   17fa8:	add    rcx,0x8
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
   17fac:	nop    DWORD PTR [rax+0x0]
non_null<alloc::alloc::Global, pwsim::Ent>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   17fb0:	mov    rdx,QWORD PTR [rcx]
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:344
   17fb3:	mov    QWORD PTR [rsp+rax*8+0x248],rdx
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:345
   17fbb:	mov    QWORD PTR [rsp+rax*8+0x48],rdx
unchecked_add():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   17fc0:	inc    rax
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   17fc3:	add    rcx,0x18
   17fc7:	cmp    r12,rax
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   17fca:	jne    17fb0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0xf0>
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   17fcc:	mov    QWORD PTR [r13+0xe8],0x0
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   17fd7:	mov    rbx,QWORD PTR [r13+0xe0]
non_null<alloc::alloc::Global, pwsim::Ovf>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   17fde:	mov    rax,QWORD PTR [r13+0x70]
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:367
   17fe2:	test   rbp,rbp
   17fe5:	sete   cl
   17fe8:	test   rax,rax
   17feb:	sete   dl
   17fee:	or     dl,cl
   17ff0:	mov    r11,rbx
   17ff3:	jne    1822e <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x36e>
   17ff9:	mov    rcx,QWORD PTR [r13+0x98]
   18000:	mov    QWORD PTR [rsp+0x18],rcx
   18005:	mov    rcx,QWORD PTR [r13+0xf8]
   1800c:	mov    QWORD PTR [rsp+0x40],rcx
   18011:	mov    rcx,QWORD PTR [r13+0x110]
   18018:	mov    QWORD PTR [rsp+0x38],rcx
   1801d:	mov    rcx,QWORD PTR [r13+0x128]
   18024:	mov    QWORD PTR [rsp+0x30],rcx
   18029:	and    r15d,0x1f
   1802d:	cmp    rbp,rax
   18030:	cmovb  rax,rbp
   18034:	xor    r10d,r10d
   18037:	nop    WORD PTR [rax+rax*1+0x0]
   18040:	mov    rcx,QWORD PTR [rsp+0x18]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:358
   18045:	mov    esi,DWORD PTR [rcx+r10*4]
{closure#0}():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:360
   18049:	mov    r8d,esi
   1804c:	shr    r8d,0x8
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:369
   18050:	mov    edi,r8d
   18053:	mov    ecx,r15d
   18056:	shr    edi,cl
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:371
   18058:	cmp    edi,0x40
   1805b:	jae    18277 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x3b7>
{closure#0}():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:362
   18061:	shl    rsi,0x38
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:361
   18065:	mov    ecx,r10d
   18068:	or     ecx,0x10000000
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:371
   1806e:	mov    r9,QWORD PTR [rsp+rdi*8+0x48]
   18073:	mov    DWORD PTR [r9],r8d
   18076:	mov    DWORD PTR [r9+0x4],ecx
   1807a:	mov    QWORD PTR [r9+0x8],rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   1807e:	add    r9,0x10
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:372
   18082:	mov    QWORD PTR [rsp+rdi*8+0x48],r9
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:374
   18087:	inc    r10
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:367
   1808a:	cmp    rax,r10
   1808d:	jne    18040 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x180>
   1808f:	mov    rdx,rbp
   18092:	mov    QWORD PTR [rsp+0x28],r13
   18097:	and    DWORD PTR [rsp+0xc],0x1f
   1809c:	xor    r13d,r13d
   1809f:	mov    QWORD PTR [rsp+0x10],r10
   180a4:	mov    QWORD PTR [rsp+0x20],rbx
   180a9:	mov    r11,rbx
   180ac:	jmp    180c0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x200>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:438
   180ae:	mov    rax,QWORD PTR [rsp+r13*8+0x240]
   180b6:	mov    QWORD PTR [rsp+r13*8+0x40],rax
   180bb:	nop    DWORD PTR [rax+rax*1+0x0]
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   180c0:	cmp    r13,r12
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   180c3:	jae    18212 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x352>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:378
   180c9:	mov    rbx,r13
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   180cc:	inc    r13
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:379
   180cf:	mov    rax,QWORD PTR [rsp+rbx*8+0x248]
   180d7:	mov    rbp,QWORD PTR [rsp+rbx*8+0x48]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:380
   180dc:	cmp    rax,rbp
   180df:	je     180c0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x200>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:384
   180e1:	jae    180b6 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x1f6>
   180e3:	data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:385
   180f0:	mov    r9d,DWORD PTR [rax]
   180f3:	mov    esi,DWORD PTR [rax+0x4]
   180f6:	mov    r8,QWORD PTR [rax+0x8]
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   180fa:	add    rax,0x10
add<pwsim::Ent>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   180fe:	jmp    18131 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x271>
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:419
   18100:	and    esi,0xfffffff
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:418
   18106:	mov    DWORD PTR [r11],esi
   18109:	mov    DWORD PTR [r11+0x4],0x0
   18111:	mov    QWORD PTR [r11+0x8],r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   18115:	add    r11,0x10
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:427
   18119:	mov    r9d,ecx
   1811c:	shr    r9d,0x8
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:428
   18120:	mov    edi,r9d
   18123:	mov    ecx,r15d
   18126:	shr    edi,cl
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:429
   18128:	cmp    rbx,rdi
   1812b:	jne    181d0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x310>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:389
   18131:	mov    ecx,r9d
   18134:	movsxd rcx,DWORD PTR [r14+rcx*4]
   18138:	test   rcx,rcx
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:390
   1813b:	js     1815a <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x29a>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:414
   1813d:	shrd   r8,rcx,0x8
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:416
   18142:	mov    edi,esi
   18144:	and    edi,0xf0000000
   1814a:	cmp    edi,0x70000000
   18150:	je     18100 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x240>
   18152:	add    esi,0x10000000
   18158:	jmp    18119 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x259>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:391
   1815a:	mov    edi,esi
   1815c:	and    edi,0xfffffff
   18162:	mov    rcx,QWORD PTR [rsp+0x40]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:393
   18167:	mov    QWORD PTR [rcx+rdi*8],r8
   1816b:	mov    ecx,DWORD PTR [rsp+0xc]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:395
   1816f:	shr    r9d,cl
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:394
   18172:	shr    esi,0x1c
   18175:	mov    rcx,QWORD PTR [rsp+0x38]
   1817a:	mov    BYTE PTR [rcx+rdi*1],sil
   1817e:	mov    rcx,QWORD PTR [rsp+0x30]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:395
   18183:	mov    DWORD PTR [rcx+rdi*4],r9d
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:397
   18187:	cmp    r10,rdx
   1818a:	jae    181ff <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x33f>
   1818c:	mov    rcx,QWORD PTR [rsp+0x18]
   18191:	mov    r8d,DWORD PTR [rcx+r10*4]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:360
   18195:	mov    r9d,r8d
{closure#0}():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:360
   18198:	shr    r9d,0x8
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:361
   1819c:	mov    esi,r10d
   1819f:	or     esi,0x10000000
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:362
   181a5:	shl    r8,0x38
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:399
   181a9:	inc    r10
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:400
   181ac:	mov    edi,r9d
   181af:	mov    ecx,r15d
   181b2:	shr    edi,cl
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:401
   181b4:	cmp    rbx,rdi
   181b7:	je     18131 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x271>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:405
   181bd:	cmp    edi,0x3f
   181c0:	jbe    181d9 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x319>
   181c2:	jmp    18288 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x3c8>
   181c7:	nop    WORD PTR [rax+rax*1+0x0]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:431
   181d0:	cmp    edi,0x3f
   181d3:	ja     18266 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x3a6>
   181d9:	mov    rcx,QWORD PTR [rsp+rdi*8+0x48]
   181de:	mov    DWORD PTR [rcx],r9d
   181e1:	mov    DWORD PTR [rcx+0x4],esi
   181e4:	mov    QWORD PTR [rcx+0x8],r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   181e8:	add    rcx,0x10
   181ec:	mov    QWORD PTR [rsp+rdi*8+0x48],rcx
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:384
   181f1:	cmp    rax,rbp
   181f4:	jb     180f0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x230>
   181fa:	jmp    180ae <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x1ee>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:409
   181ff:	dec    QWORD PTR [rsp+0x10]
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:384
   18204:	cmp    rax,rbp
   18207:	jb     180f0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x230>
   1820d:	jmp    180ae <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x1ee>
   18212:	mov    r13d,0x0
   18218:	cmp    QWORD PTR [rsp+0x10],0x0
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:377
   1821e:	jne    180c0 <_RNvCsl7I5l8z6U99_5pwsim8pw_walk2+0x200>
   18224:	mov    r13,QWORD PTR [rsp+0x28]
   18229:	mov    rbx,QWORD PTR [rsp+0x20]
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/const_ptr.rs:619
   1822e:	sub    r11,rbx
offset_from<pwsim::Ovf>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/const_ptr.rs:619
   18231:	sar    r11,0x4
_RNvCsl7I5l8z6U99_5pwsim8pw_walk2():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:2221
   18235:	mov    QWORD PTR [r13+0xe8],r11
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:443
   1823c:	add    rsp,0x448
   18243:	pop    rbx
   18244:	pop    r12
   18246:	pop    r13
   18248:	pop    r14
   1824a:	pop    r15
   1824c:	pop    rbp
   1824d:	ret
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:339
   1824e:	lea    rdi,[rip+0xfffffffffffefffa]        # 824f <GCC_except_table454+0x379b>
   18255:	lea    rdx,[rip+0x445ec]        # 5c848 <__frame_dummy_init_array_entry+0x568>
   1825c:	mov    esi,0x1b
   18261:	call   1bce0 <_RNvNtCscI6d9CVNmLh_4core9panicking5panic>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:431
   18266:	lea    rdx,[rip+0x445f3]        # 5c860 <__frame_dummy_init_array_entry+0x580>
   1826d:	mov    esi,0x40
   18272:	call   1ae8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:371
   18277:	lea    rdx,[rip+0x44612]        # 5c890 <__frame_dummy_init_array_entry+0x5b0>
   1827e:	mov    esi,0x40
   18283:	call   1ae8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/pwalk/routea/sim/src/main.rs:405
   18288:	lea    rdx,[rip+0x445e9]        # 5c878 <__frame_dummy_init_array_entry+0x598>
   1828f:	mov    esi,0x40
   18294:	call   1ae8b <_RNvNtCscI6d9CVNmLh_4core9panicking18panic_bounds_check>

Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .plt:
