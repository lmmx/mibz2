
bin/phases-base:     file format elf64-x86-64


Disassembly of section .text:

0000000000030d30 <mibz2::mtf::mtf_avx2>:
   30d30:	push   rbp
   30d31:	mov    rbp,rsp
   30d34:	push   r15
   30d36:	push   r14
   30d38:	push   r13
   30d3a:	push   r12
   30d3c:	push   rbx
   30d3d:	and    rsp,0xffffffffffffffc0
   30d41:	sub    rsp,0x1c0
   30d48:	mov    r15,rcx
   30d4b:	mov    rbx,rdx
   30d4e:	mov    r13,rsi
   30d51:	mov    r14,rdi
   30d54:	mov    rax,QWORD PTR [rsi+0x118]
   30d5b:	mov    QWORD PTR [rdx+0x10],0x0
   30d63:	mov    QWORD PTR [rsp+0x20],rax
   30d68:	cmp    rax,QWORD PTR [rdx]
   30d6b:	ja     3109e <mibz2::mtf::mtf_avx2+0x36e>
   30d71:	xor    r12d,r12d
   30d74:	mov    edx,0x400
   30d79:	mov    rdi,r15
   30d7c:	xor    esi,esi
   30d7e:	call   QWORD PTR [rip+0x3b114]        # 6be98 <memset@GLIBC_2.2.5>
   30d84:	vxorps xmm0,xmm0,xmm0
   30d88:	vmovaps YMMWORD PTR [rsp+0x160],ymm0
   30d91:	vmovaps YMMWORD PTR [rsp+0x140],ymm0
   30d9a:	vmovaps YMMWORD PTR [rsp+0x120],ymm0
   30da3:	vmovaps YMMWORD PTR [rsp+0x100],ymm0
   30dac:	vmovaps YMMWORD PTR [rsp+0xe0],ymm0
   30db5:	vmovaps YMMWORD PTR [rsp+0xc0],ymm0
   30dbe:	vmovaps YMMWORD PTR [rsp+0xa0],ymm0
   30dc7:	vmovaps YMMWORD PTR [rsp+0x80],ymm0
   30dd0:	vmovaps YMMWORD PTR [rsp+0x60],ymm0
   30dd6:	vmovaps YMMWORD PTR [rsp+0x40],ymm0
   30ddc:	vmovups ymm0,YMMWORD PTR [r13+0xf8]
   30de5:	vmovups YMMWORD PTR [rsp+0x140],ymm0
   30dee:	vmovups ymm0,YMMWORD PTR [r13+0xd8]
   30df7:	vmovaps YMMWORD PTR [rsp+0x120],ymm0
   30e00:	vmovups ymm0,YMMWORD PTR [r13+0xb8]
   30e09:	vmovups YMMWORD PTR [rsp+0x100],ymm0
   30e12:	vmovups ymm0,YMMWORD PTR [r13+0x98]
   30e1b:	vmovaps YMMWORD PTR [rsp+0xe0],ymm0
   30e24:	vmovdqu ymm0,YMMWORD PTR [r13+0x18]
   30e2a:	vmovdqu ymm1,YMMWORD PTR [r13+0x38]
   30e30:	vmovdqu ymm2,YMMWORD PTR [r13+0x58]
   30e36:	vmovups ymm3,YMMWORD PTR [r13+0x78]
   30e3c:	vmovups YMMWORD PTR [rsp+0xc0],ymm3
   30e45:	vmovdqa YMMWORD PTR [rsp+0xa0],ymm2
   30e4e:	vmovdqu YMMWORD PTR [rsp+0x80],ymm1
   30e57:	vmovdqa YMMWORD PTR [rsp+0x60],ymm0
   30e5d:	mov    rax,QWORD PTR [r13+0x10]
   30e61:	test   rax,rax
   30e64:	mov    QWORD PTR [rsp+0x28],rbx
   30e69:	je     31063 <mibz2::mtf::mtf_avx2+0x333>
   30e6f:	mov    QWORD PTR [rsp+0x30],r15
   30e74:	lea    r8,[rsp+0x60]
   30e79:	mov    r9,QWORD PTR [rbx+0x8]
   30e7d:	mov    rbx,QWORD PTR [r13+0x8]
   30e81:	lea    r15,[rbx+rax*2]
   30e85:	movzx  r10d,WORD PTR [r13+0x128]
   30e8d:	mov    r13d,0x1
   30e93:	lea    r11,[rsp+0x61]
   30e98:	xor    edi,edi
   30e9a:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffd445e]        # 5300 <GCC_except_table475+0x608>
   30ea2:	mov    QWORD PTR [rsp+0x18],r14
   30ea7:	xor    eax,eax
   30ea9:	nop    DWORD PTR [rax+0x0]
   30eb0:	movzx  r14d,WORD PTR [rbx]
   30eb4:	add    rbx,0x2
   30eb8:	cmp    r14d,0x2
   30ebc:	jae    30ef0 <mibz2::mtf::mtf_avx2+0x1c0>
   30ebe:	cmp    r13d,0x1fffff
   30ec5:	ja     31041 <mibz2::mtf::mtf_avx2+0x311>
   30ecb:	inc    r14d
   30ece:	imul   r14d,r13d
   30ed2:	add    eax,r14d
   30ed5:	add    r13d,r13d
   30ed8:	cmp    rbx,r15
   30edb:	jne    30eb0 <mibz2::mtf::mtf_avx2+0x180>
   30edd:	jmp    3105b <mibz2::mtf::mtf_avx2+0x32b>
   30ee2:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   30ef0:	test   eax,eax
   30ef2:	je     30f49 <mibz2::mtf::mtf_avx2+0x219>
   30ef4:	mov    edx,eax
   30ef6:	lea    r12,[rdi+rdx*1]
   30efa:	cmp    r12,QWORD PTR [rsp+0x20]
   30eff:	ja     3108f <mibz2::mtf::mtf_avx2+0x35f>
   30f05:	movzx  ecx,BYTE PTR [rsp+0x60]
   30f0a:	mov    rsi,QWORD PTR [rsp+0x30]
   30f0f:	add    DWORD PTR [rsi+rcx*4],eax
   30f12:	cmp    eax,0x40
   30f15:	ja     30f57 <mibz2::mtf::mtf_avx2+0x227>
   30f17:	lea    rax,[rdi+0x40]
   30f1b:	cmp    rax,QWORD PTR [rsp+0x20]
   30f20:	ja     30f57 <mibz2::mtf::mtf_avx2+0x227>
   30f22:	vmovd  xmm0,ecx
   30f26:	vpbroadcastb ymm0,xmm0
   30f2b:	vmovdqu YMMWORD PTR [r9+rdi*1],ymm0
   30f31:	vmovdqu YMMWORD PTR [r9+rdi*1+0x20],ymm0
   30f38:	mov    r13d,0x1
   30f3e:	cmp    r14w,r10w
   30f42:	jne    30f9a <mibz2::mtf::mtf_avx2+0x26a>
   30f44:	jmp    3105e <mibz2::mtf::mtf_avx2+0x32e>
   30f49:	mov    r12,rdi
   30f4c:	cmp    r14w,r10w
   30f50:	jne    30f9a <mibz2::mtf::mtf_avx2+0x26a>
   30f52:	jmp    3105e <mibz2::mtf::mtf_avx2+0x32e>
   30f57:	add    rdi,r9
   30f5a:	movzx  esi,cl
   30f5d:	mov    r13,r9
   30f60:	mov    WORD PTR [rsp+0x16],r10w
   30f66:	vzeroupper
   30f69:	call   QWORD PTR [rip+0x3af29]        # 6be98 <memset@GLIBC_2.2.5>
   30f6f:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffd4389]        # 5300 <GCC_except_table475+0x608>
   30f77:	lea    r11,[rsp+0x61]
   30f7c:	movzx  r10d,WORD PTR [rsp+0x16]
   30f82:	mov    r9,r13
   30f85:	lea    r8,[rsp+0x60]
   30f8a:	mov    r13d,0x1
   30f90:	cmp    r14w,r10w
   30f94:	je     3105e <mibz2::mtf::mtf_avx2+0x32e>
   30f9a:	cmp    r12,QWORD PTR [rsp+0x20]
   30f9f:	jae    3108c <mibz2::mtf::mtf_avx2+0x35c>
   30fa5:	mov    eax,r14d
   30fa8:	dec    r14d
   30fab:	movzx  edx,r14w
   30faf:	movzx  r14d,BYTE PTR [rsp+rdx*1+0x60]
   30fb5:	cmp    ax,0x21
   30fb9:	jae    30fe0 <mibz2::mtf::mtf_avx2+0x2b0>
   30fbb:	vmovdqa ymm0,YMMWORD PTR [rsp+0x60]
   30fc1:	inc    dl
   30fc3:	vmovd  xmm1,edx
   30fc7:	vpbroadcastb ymm1,xmm1
   30fcc:	vpcmpgtb ymm1,ymm1,ymm2
   30fd0:	vpblendvb ymm0,ymm0,YMMWORD PTR [rsp+0x5f],ymm1
   30fd8:	vmovdqa YMMWORD PTR [rsp+0x60],ymm0
   30fde:	jmp    31017 <mibz2::mtf::mtf_avx2+0x2e7>
   30fe0:	mov    rdi,r11
   30fe3:	mov    rsi,r8
   30fe6:	mov    QWORD PTR [rsp+0x38],r9
   30feb:	mov    WORD PTR [rsp+0x16],r10w
   30ff1:	vzeroupper
   30ff4:	call   QWORD PTR [rip+0x3af26]        # 6bf20 <memmove@GLIBC_2.2.5>
   30ffa:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffd42fe]        # 5300 <GCC_except_table475+0x608>
   31002:	lea    r11,[rsp+0x61]
   31007:	movzx  r10d,WORD PTR [rsp+0x16]
   3100d:	mov    r9,QWORD PTR [rsp+0x38]
   31012:	lea    r8,[rsp+0x60]
   31017:	movzx  eax,r14b
   3101b:	mov    BYTE PTR [rsp+0x60],al
   3101f:	mov    rcx,QWORD PTR [rsp+0x30]
   31024:	inc    DWORD PTR [rcx+rax*4]
   31027:	mov    BYTE PTR [r9+r12*1],al
   3102b:	inc    r12
   3102e:	mov    rdi,r12
   31031:	cmp    rbx,r15
   31034:	mov    r14,QWORD PTR [rsp+0x18]
   31039:	jne    30ea7 <mibz2::mtf::mtf_avx2+0x177>
   3103f:	jmp    31063 <mibz2::mtf::mtf_avx2+0x333>
   31041:	movabs rax,0x800000000000000c
   3104b:	mov    rcx,QWORD PTR [rsp+0x28]
   31050:	mov    QWORD PTR [rcx+0x10],rdi
   31054:	mov    r14,QWORD PTR [rsp+0x18]
   31059:	jmp    31077 <mibz2::mtf::mtf_avx2+0x347>
   3105b:	mov    r12,rdi
   3105e:	mov    r14,QWORD PTR [rsp+0x18]
   31063:	mov    rax,QWORD PTR [rsp+0x28]
   31068:	mov    QWORD PTR [rax+0x10],r12
   3106c:	mov    QWORD PTR [r14+0x8],r12
   31070:	mov    rax,0xffffffffffffffff
   31077:	mov    QWORD PTR [r14],rax
   3107a:	lea    rsp,[rbp-0x28]
   3107e:	pop    rbx
   3107f:	pop    r12
   31081:	pop    r13
   31083:	pop    r14
   31085:	pop    r15
   31087:	pop    rbp
   31088:	vzeroupper
   3108b:	ret
   3108c:	mov    rdi,r12
   3108f:	movabs rax,0x800000000000000c
   31099:	inc    rax
   3109c:	jmp    3104b <mibz2::mtf::mtf_avx2+0x31b>
   3109e:	mov    ecx,0x1
   310a3:	mov    r8d,0x1
   310a9:	mov    rdi,rbx
   310ac:	xor    esi,esi
   310ae:	mov    rdx,QWORD PTR [rsp+0x20]
   310b3:	call   2b350 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   310b8:	jmp    30d71 <mibz2::mtf::mtf_avx2+0x41>
   310bd:	int3
   310be:	int3
   310bf:	int3

00000000000310c0 <<mibz2::Kernels>::detect>:
   310c0:	push   rbx
   310c1:	mov    rax,QWORD PTR [rip+0x3c9d8]        # 6daa0 <std_detect::detect::cache::CACHE>
   310c8:	test   rax,rax
   310cb:	je     310d6 <<mibz2::Kernels>::detect+0x16>
   310cd:	test   ax,ax
   310d0:	js     310e0 <<mibz2::Kernels>::detect+0x20>
   310d2:	xor    eax,eax
   310d4:	jmp    310f3 <<mibz2::Kernels>::detect+0x33>
   310d6:	call   68290 <std_detect::detect::cache::detect_and_initialize>
   310db:	test   ax,ax
   310de:	jns    310d2 <<mibz2::Kernels>::detect+0x12>
   310e0:	mov    rax,QWORD PTR [rip+0x3c9b9]        # 6daa0 <std_detect::detect::cache::CACHE>
   310e7:	test   rax,rax
   310ea:	je     31131 <<mibz2::Kernels>::detect+0x71>
   310ec:	shr    rax,0x39
   310f0:	and    eax,0x1
   310f3:	mov    rcx,QWORD PTR [rip+0x3c9a6]        # 6daa0 <std_detect::detect::cache::CACHE>
   310fa:	test   rcx,rcx
   310fd:	je     3111c <<mibz2::Kernels>::detect+0x5c>
   310ff:	test   cl,0x2
   31102:	je     31118 <<mibz2::Kernels>::detect+0x58>
   31104:	mov    rcx,QWORD PTR [rip+0x3c995]        # 6daa0 <std_detect::detect::cache::CACHE>
   3110b:	test   rcx,rcx
   3110e:	je     31138 <<mibz2::Kernels>::detect+0x78>
   31110:	test   ecx,0x400
   31116:	jne    3114e <<mibz2::Kernels>::detect+0x8e>
   31118:	xor    edx,edx
   3111a:	pop    rbx
   3111b:	ret
   3111c:	mov    rbx,rax
   3111f:	call   68290 <std_detect::detect::cache::detect_and_initialize>
   31124:	mov    rcx,rax
   31127:	mov    rax,rbx
   3112a:	test   cl,0x2
   3112d:	je     31118 <<mibz2::Kernels>::detect+0x58>
   3112f:	jmp    31104 <<mibz2::Kernels>::detect+0x44>
   31131:	call   68290 <std_detect::detect::cache::detect_and_initialize>
   31136:	jmp    310ec <<mibz2::Kernels>::detect+0x2c>
   31138:	mov    rbx,rax
   3113b:	call   68290 <std_detect::detect::cache::detect_and_initialize>
   31140:	mov    rcx,rax
   31143:	mov    rax,rbx
   31146:	test   ecx,0x400
   3114c:	je     31118 <<mibz2::Kernels>::detect+0x58>
   3114e:	mov    rdx,QWORD PTR [rip+0x3c94b]        # 6daa0 <std_detect::detect::cache::CACHE>
   31155:	test   rdx,rdx
   31158:	je     31162 <<mibz2::Kernels>::detect+0xa2>
   3115a:	shr    edx,0x9
   3115d:	and    edx,0x1
   31160:	pop    rbx
   31161:	ret
   31162:	mov    rbx,rax
   31165:	call   68290 <std_detect::detect::cache::detect_and_initialize>
   3116a:	mov    rdx,rax
   3116d:	mov    rax,rbx
   31170:	shr    edx,0x9
   31173:	and    edx,0x1
   31176:	pop    rbx
   31177:	ret
   31178:	int3
   31179:	int3
   3117a:	int3
   3117b:	int3
   3117c:	int3
   3117d:	int3
   3117e:	int3
   3117f:	int3

0000000000031180 <mibz2::bwt::walk_segments_room>:
   31180:	push   rbp
   31181:	push   r15
   31183:	push   r14
   31185:	push   r13
   31187:	push   r12
   31189:	push   rbx
   3118a:	push   rax
   3118b:	mov    rbx,rdx
   3118e:	mov    ebp,esi
   31190:	mov    r15,QWORD PTR [rdi+0x8]
   31194:	mov    r14,QWORD PTR [rdi+0x10]
   31198:	cmp    r14,0x10000
   3119f:	jb     311ca <mibz2::bwt::walk_segments_room+0x4a>
   311a1:	mov    rdi,r15
   311a4:	mov    rsi,r14
   311a7:	mov    edx,ebp
   311a9:	mov    r14,rcx
   311ac:	call   299f0 <mibz2::bwt::room_segments::<32, 1>>
   311b1:	mov    rdi,r14
   311b4:	mov    rsi,rbx
   311b7:	add    rsp,0x8
   311bb:	pop    rbx
   311bc:	pop    r12
   311be:	pop    r13
   311c0:	pop    r14
   311c2:	pop    r15
   311c4:	pop    rbp
   311c5:	jmp    2d0b0 <<mibz2::bwt::SegScratch>::assemble>
   311ca:	mov    QWORD PTR [rbx+0x10],0x0
   311d2:	cmp    r14,QWORD PTR [rbx]
   311d5:	ja     3122a <mibz2::bwt::walk_segments_room+0xaa>
   311d7:	mov    edi,ebp
   311d9:	cmp    r14,rdi
   311dc:	jbe    31249 <mibz2::bwt::walk_segments_room+0xc9>
   311de:	mov    r12d,DWORD PTR [r15+rdi*4]
   311e2:	mov    r13,QWORD PTR [rbx+0x10]
   311e6:	jmp    31204 <mibz2::bwt::walk_segments_room+0x84>
   311e8:	nop    DWORD PTR [rax+rax*1+0x0]
   311f0:	mov    rax,QWORD PTR [rbx+0x8]
   311f4:	mov    BYTE PTR [rax+r13*1],r12b
   311f8:	inc    r13
   311fb:	mov    QWORD PTR [rbx+0x10],r13
   311ff:	dec    r14
   31202:	je     3121b <mibz2::bwt::walk_segments_room+0x9b>
   31204:	shr    r12d,0x8
   31208:	mov    r12d,DWORD PTR [r15+r12*4]
   3120c:	cmp    r13,QWORD PTR [rbx]
   3120f:	jne    311f0 <mibz2::bwt::walk_segments_room+0x70>
   31211:	mov    rdi,rbx
   31214:	call   1f3c0 <<alloc::raw_vec::RawVec<u8>>::grow_one>
   31219:	jmp    311f0 <mibz2::bwt::walk_segments_room+0x70>
   3121b:	add    rsp,0x8
   3121f:	pop    rbx
   31220:	pop    r12
   31222:	pop    r13
   31224:	pop    r14
   31226:	pop    r15
   31228:	pop    rbp
   31229:	ret
   3122a:	mov    ecx,0x1
   3122f:	mov    r8d,0x1
   31235:	mov    rdi,rbx
   31238:	xor    esi,esi
   3123a:	mov    rdx,r14
   3123d:	call   2b350 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   31242:	mov    edi,ebp
   31244:	cmp    r14,rdi
   31247:	ja     311de <mibz2::bwt::walk_segments_room+0x5e>
   31249:	lea    rdx,[rip+0x39338]        # 6a588 <__frame_dummy_init_array_entry+0xc08>
   31250:	mov    rsi,r14
   31253:	call   2040b <core::panicking::panic_bounds_check>
   31258:	int3
   31259:	int3
   3125a:	int3
   3125b:	int3
   3125c:	int3
   3125d:	int3
   3125e:	int3
   3125f:	int3

0000000000031260 <mibz2::bwt::walk_segments_flagged>:
   31260:	push   rbp
   31261:	push   r15
   31263:	push   r14
   31265:	push   r13
   31267:	push   r12
   31269:	push   rbx
   3126a:	sub    rsp,0x4a8
   31271:	mov    rbp,QWORD PTR [rdi+0x10]
   31275:	mov    r14d,esi
   31278:	cmp    rbp,0x10000
   3127f:	mov    QWORD PTR [rsp+0x28],rdx
   31284:	jae    312e4 <mibz2::bwt::walk_segments_flagged+0x84>
   31286:	mov    QWORD PTR [rdx+0x10],0x0
   3128e:	cmp    rbp,QWORD PTR [rdx]
   31291:	ja     3213a <mibz2::bwt::walk_segments_flagged+0xeda>
   31297:	cmp    rbp,r14
   3129a:	jbe    32166 <mibz2::bwt::walk_segments_flagged+0xf06>
   312a0:	mov    rbx,QWORD PTR [rdi+0x8]
   312a4:	mov    r14d,DWORD PTR [rbx+r14*4]
   312a8:	mov    r15,QWORD PTR [rdx+0x10]
   312ac:	jmp    312c8 <mibz2::bwt::walk_segments_flagged+0x68>
   312ae:	xchg   ax,ax
   312b0:	mov    rax,QWORD PTR [rdx+0x8]
   312b4:	mov    BYTE PTR [rax+r15*1],r14b
   312b8:	inc    r15
   312bb:	mov    QWORD PTR [rdx+0x10],r15
   312bf:	dec    rbp
   312c2:	je     32116 <mibz2::bwt::walk_segments_flagged+0xeb6>
   312c8:	shr    r14d,0x8
   312cc:	mov    r14d,DWORD PTR [rbx+r14*4]
   312d0:	cmp    r15,QWORD PTR [rdx]
   312d3:	jne    312b0 <mibz2::bwt::walk_segments_flagged+0x50>
   312d5:	mov    rdi,rdx
   312d8:	call   1f3c0 <<alloc::raw_vec::RawVec<u8>>::grow_one>
   312dd:	mov    rdx,QWORD PTR [rsp+0x28]
   312e2:	jmp    312b0 <mibz2::bwt::walk_segments_flagged+0x50>
   312e4:	mov    r13,QWORD PTR [rdi+0x8]
   312e8:	mov    QWORD PTR [rdx+0x10],0x0
   312f0:	cmp    rbp,r14
   312f3:	jbe    32440 <mibz2::bwt::walk_segments_flagged+0x11e0>
   312f9:	mov    r15d,DWORD PTR [r13+r14*4+0x0]
   312fe:	mov    QWORD PTR [rcx+0x10],0x0
   31306:	movabs r12,0x2545f4914f6cdd1d
   31310:	cmp    QWORD PTR [rcx],0x1fe
   31317:	jbe    32178 <mibz2::bwt::walk_segments_flagged+0xf18>
   3131d:	xor    r14d,r14d
   31320:	shr    r15d,0x8
