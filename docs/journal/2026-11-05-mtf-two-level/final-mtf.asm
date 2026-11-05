0000000000031620 <mibz2::mtf::mtf_avx2>:
   31620:	push   rbp
   31621:	mov    rbp,rsp
   31624:	push   r15
   31626:	push   r14
   31628:	push   r13
   3162a:	push   r12
   3162c:	push   rbx
   3162d:	and    rsp,0xffffffffffffffc0
   31631:	sub    rsp,0x1c0
   31638:	mov    rbx,rcx
   3163b:	mov    r14,rdx
   3163e:	mov    r13,rsi
   31641:	mov    QWORD PTR [rsp+0x18],rdi
   31646:	mov    rax,QWORD PTR [rsi+0x118]
   3164d:	mov    QWORD PTR [rdx+0x10],0x0
   31655:	mov    QWORD PTR [rsp+0x10],rax
   3165a:	cmp    rax,QWORD PTR [rdx]
   3165d:	ja     31ac9 <mibz2::mtf::mtf_avx2+0x4a9>
   31663:	xor    r12d,r12d
   31666:	mov    edx,0x400
   3166b:	mov    rdi,rbx
   3166e:	xor    esi,esi
   31670:	call   QWORD PTR [rip+0x3b252]        # 6c8c8 <memset@GLIBC_2.2.5>
   31676:	vmovdqu xmm10,XMMWORD PTR [r13+0x18]
   3167c:	vmovdqu xmm9,XMMWORD PTR [r13+0x28]
   31682:	vmovups ymm0,YMMWORD PTR [r13+0xf8]
   3168b:	vmovaps YMMWORD PTR [rsp+0x100],ymm0
   31694:	vmovups ymm0,YMMWORD PTR [r13+0xd8]
   3169d:	vmovaps YMMWORD PTR [rsp+0xe0],ymm0
   316a6:	vmovups ymm0,YMMWORD PTR [r13+0xb8]
   316af:	vmovaps YMMWORD PTR [rsp+0xc0],ymm0
   316b8:	vmovdqu ymm0,YMMWORD PTR [r13+0x38]
   316be:	vmovdqu ymm1,YMMWORD PTR [r13+0x58]
   316c4:	vmovdqu ymm2,YMMWORD PTR [r13+0x78]
   316ca:	vmovdqu ymm3,YMMWORD PTR [r13+0x98]
   316d3:	vmovdqa YMMWORD PTR [rsp+0xa0],ymm3
   316dc:	vmovdqa YMMWORD PTR [rsp+0x80],ymm2
   316e5:	vmovdqa YMMWORD PTR [rsp+0x60],ymm1
   316eb:	vmovdqa YMMWORD PTR [rsp+0x40],ymm0
   316f1:	mov    rax,QWORD PTR [r13+0x10]
   316f5:	test   rax,rax
   316f8:	je     31a8e <mibz2::mtf::mtf_avx2+0x46e>
   316fe:	mov    QWORD PTR [rsp+0x20],rbx
   31703:	mov    r9,QWORD PTR [r14+0x8]
   31707:	mov    rbx,QWORD PTR [r13+0x8]
   3170b:	lea    r15,[rbx+rax*2]
   3170f:	movzx  eax,WORD PTR [r13+0x128]
   31717:	mov    WORD PTR [rsp+0xe],ax
   3171c:	mov    eax,0x1
   31721:	xor    edi,edi
   31723:	lea    r11,[rip+0xfffffffffffdd856]        # ef80 <mibz2::mtf::STEPS>
   3172a:	vmovdqa ymm11,YMMWORD PTR [rip+0xfffffffffffd3cae]        # 53e0 <GCC_except_table475+0x6e8>
   31732:	vmovdqa ymm13,YMMWORD PTR [rip+0xfffffffffffd3c46]        # 5380 <GCC_except_table475+0x688>
   3173a:	vmovdqa ymm14,YMMWORD PTR [rip+0xfffffffffffd3bfe]        # 5340 <GCC_except_table475+0x648>
   31742:	vmovdqa ymm15,YMMWORD PTR [rip+0xfffffffffffd3c76]        # 53c0 <GCC_except_table475+0x6c8>
   3174a:	vmovdqa ymm12,YMMWORD PTR [rip+0xfffffffffffd3bae]        # 5300 <GCC_except_table475+0x608>
   31752:	mov    r10,QWORD PTR [rip+0x3b16f]        # 6c8c8 <memset@GLIBC_2.2.5>
   31759:	xor    ecx,ecx
   3175b:	nop    DWORD PTR [rax+rax*1+0x0]
   31760:	movzx  r13d,WORD PTR [rbx]
   31764:	add    rbx,0x2
   31768:	cmp    r13d,0x2
   3176c:	jae    31790 <mibz2::mtf::mtf_avx2+0x170>
   3176e:	cmp    eax,0x1fffff
   31773:	ja     31a76 <mibz2::mtf::mtf_avx2+0x456>
   31779:	inc    r13d
   3177c:	imul   r13d,eax
   31780:	add    ecx,r13d
   31783:	add    eax,eax
   31785:	cmp    rbx,r15
   31788:	jne    31760 <mibz2::mtf::mtf_avx2+0x140>
   3178a:	jmp    31a8b <mibz2::mtf::mtf_avx2+0x46b>
   3178f:	nop
   31790:	test   ecx,ecx
   31792:	je     317ee <mibz2::mtf::mtf_avx2+0x1ce>
   31794:	mov    edx,ecx
   31796:	lea    r12,[rdi+rdx*1]
   3179a:	cmp    r12,QWORD PTR [rsp+0x10]
   3179f:	ja     31aba <mibz2::mtf::mtf_avx2+0x49a>
   317a5:	vmovd  esi,xmm10
   317a9:	movzx  eax,sil
   317ad:	mov    r8,QWORD PTR [rsp+0x20]
   317b2:	add    DWORD PTR [r8+rax*4],ecx
   317b6:	cmp    ecx,0x40
   317b9:	ja     31802 <mibz2::mtf::mtf_avx2+0x1e2>
   317bb:	lea    rax,[rdi+0x40]
   317bf:	cmp    rax,QWORD PTR [rsp+0x10]
   317c4:	ja     31802 <mibz2::mtf::mtf_avx2+0x1e2>
   317c6:	vpbroadcastb ymm0,xmm10
   317cb:	vmovdqu YMMWORD PTR [r9+rdi*1],ymm0
   317d1:	vmovdqu YMMWORD PTR [r9+rdi*1+0x20],ymm0
   317d8:	mov    eax,0x1
   317dd:	cmp    r13w,WORD PTR [rsp+0xe]
   317e3:	jne    3187d <mibz2::mtf::mtf_avx2+0x25d>
   317e9:	jmp    31a8e <mibz2::mtf::mtf_avx2+0x46e>
   317ee:	mov    r12,rdi
   317f1:	cmp    r13w,WORD PTR [rsp+0xe]
   317f7:	jne    3187d <mibz2::mtf::mtf_avx2+0x25d>
   317fd:	jmp    31a8e <mibz2::mtf::mtf_avx2+0x46e>
   31802:	add    rdi,r9
   31805:	vmovdqa YMMWORD PTR [rsp+0x160],ymm9
   3180e:	vmovdqa XMMWORD PTR [rsp+0x30],xmm10
   31814:	mov    QWORD PTR [rsp+0x28],r9
   31819:	vzeroupper
   3181c:	call   QWORD PTR [rip+0x3b0a6]        # 6c8c8 <memset@GLIBC_2.2.5>
   31822:	mov    r10,QWORD PTR [rip+0x3b09f]        # 6c8c8 <memset@GLIBC_2.2.5>
   31829:	vmovdqa ymm12,YMMWORD PTR [rip+0xfffffffffffd3acf]        # 5300 <GCC_except_table475+0x608>
   31831:	vmovdqa ymm15,YMMWORD PTR [rip+0xfffffffffffd3b87]        # 53c0 <GCC_except_table475+0x6c8>
   31839:	vmovdqa ymm14,YMMWORD PTR [rip+0xfffffffffffd3aff]        # 5340 <GCC_except_table475+0x648>
   31841:	vmovdqa ymm13,YMMWORD PTR [rip+0xfffffffffffd3b37]        # 5380 <GCC_except_table475+0x688>
   31849:	vmovdqa ymm11,YMMWORD PTR [rip+0xfffffffffffd3b8f]        # 53e0 <GCC_except_table475+0x6e8>
   31851:	lea    r11,[rip+0xfffffffffffdd728]        # ef80 <mibz2::mtf::STEPS>
   31858:	mov    r9,QWORD PTR [rsp+0x28]
   3185d:	vmovdqa xmm10,XMMWORD PTR [rsp+0x30]
   31863:	vmovdqa ymm9,YMMWORD PTR [rsp+0x160]
   3186c:	mov    eax,0x1
   31871:	cmp    r13w,WORD PTR [rsp+0xe]
   31877:	je     31a8e <mibz2::mtf::mtf_avx2+0x46e>
   3187d:	cmp    r12,QWORD PTR [rsp+0x10]
   31882:	jae    31ab7 <mibz2::mtf::mtf_avx2+0x497>
   31888:	lea    ecx,[r13-0x1]
   3188c:	movzx  edx,cx
   3188f:	cmp    r13w,0x21
   31894:	jae    318c2 <mibz2::mtf::mtf_avx2+0x2a2>
   31896:	shl    edx,0x6
   31899:	vpshufb xmm0,xmm10,XMMWORD PTR [rdx+r11*1]
   3189f:	vmovdqa xmm1,XMMWORD PTR [rdx+r11*1+0x20]
   318a6:	vpshufb xmm2,xmm9,XMMWORD PTR [rdx+r11*1+0x10]
   318ad:	vpalignr xmm3,xmm9,xmm10,0xf
   318b3:	vpor   xmm10,xmm2,xmm0
   318b7:	vpblendvb xmm9,xmm9,xmm3,xmm1
   318bd:	jmp    31a52 <mibz2::mtf::mtf_avx2+0x432>
   318c2:	movzx  ecx,BYTE PTR [rsp+rdx*1+0x20]
   318c7:	add    rdx,0xffffffffffffffe0
   318cb:	vinserti128 ymm0,ymm9,xmm9,0x1
   318d1:	cmp    r13w,0x40
   318d6:	ja     31905 <mibz2::mtf::mtf_avx2+0x2e5>
   318d8:	vmovdqa ymm1,YMMWORD PTR [rsp+0x40]
   318de:	inc    dl
   318e0:	vmovd  xmm2,edx
   318e4:	vpbroadcastb ymm2,xmm2
   318e9:	vpcmpgtb ymm2,ymm2,ymm11
   318ee:	vperm2i128 ymm0,ymm0,ymm1,0x21
   318f4:	vpalignr ymm0,ymm1,ymm0,0xf
   318fa:	vpblendvb ymm0,ymm1,ymm0,ymm2
   31900:	jmp    31a38 <mibz2::mtf::mtf_avx2+0x418>
   31905:	vmovd  xmm1,edx
   31909:	vpbroadcastb ymm1,xmm1
   3190e:	vmovdqa ymm3,YMMWORD PTR [rsp+0x40]
   31914:	vmovdqa ymm4,YMMWORD PTR [rsp+0x60]
   3191a:	vmovdqa ymm5,YMMWORD PTR [rsp+0x80]
   31923:	vmovdqa ymm6,YMMWORD PTR [rsp+0xa0]
   3192c:	vmovdqa ymm2,YMMWORD PTR [rsp+0xc0]
   31935:	vpmaxub ymm7,ymm11,ymm1
   31939:	vpcmpeqb ymm7,ymm1,ymm7
   3193d:	vperm2i128 ymm0,ymm0,ymm3,0x21
   31943:	vpalignr ymm0,ymm3,ymm0,0xf
   31949:	vpblendvb ymm0,ymm3,ymm0,ymm7
   3194f:	vpminub ymm7,ymm1,YMMWORD PTR [rip+0xfffffffffffd39c9]        # 5320 <GCC_except_table475+0x628>
   31957:	vpcmpeqb ymm7,ymm1,ymm7
   3195b:	vperm2i128 ymm3,ymm3,ymm4,0x21
   31961:	vpalignr ymm3,ymm4,ymm3,0xf
   31967:	vpblendvb ymm3,ymm3,ymm4,ymm7
   3196d:	vpminub ymm7,ymm13,ymm1
   31971:	vpcmpeqb ymm7,ymm1,ymm7
   31975:	vperm2i128 ymm4,ymm4,ymm5,0x21
   3197b:	vpalignr ymm4,ymm5,ymm4,0xf
   31981:	vpblendvb ymm4,ymm4,ymm5,ymm7
   31987:	vpminub ymm7,ymm14,ymm1
   3198b:	vpcmpeqb ymm7,ymm1,ymm7
   3198f:	vperm2i128 ymm5,ymm5,ymm6,0x21
   31995:	vpalignr ymm5,ymm6,ymm5,0xf
   3199b:	vpblendvb ymm5,ymm5,ymm6,ymm7
   319a1:	vpminub ymm7,ymm15,ymm1
   319a5:	vpcmpeqb ymm7,ymm1,ymm7
   319a9:	vperm2i128 ymm6,ymm6,ymm2,0x21
   319af:	vpalignr ymm6,ymm2,ymm6,0xf
   319b5:	vpblendvb ymm6,ymm6,ymm2,ymm7
   319bb:	vmovdqa ymm7,YMMWORD PTR [rsp+0xe0]
   319c4:	vpminub ymm8,ymm12,ymm1
   319c8:	vpcmpeqb ymm8,ymm8,ymm1
   319cc:	vperm2i128 ymm2,ymm2,ymm7,0x21
   319d2:	vpalignr ymm2,ymm7,ymm2,0xf
   319d8:	vpblendvb ymm2,ymm2,ymm7,ymm8
   319de:	vpminub ymm8,ymm1,YMMWORD PTR [rip+0xfffffffffffd39ba]        # 53a0 <GCC_except_table475+0x6a8>
   319e6:	vpcmpeqb ymm1,ymm8,ymm1
   319ea:	vmovdqa ymm8,YMMWORD PTR [rsp+0x100]
   319f3:	vperm2i128 ymm7,ymm7,ymm8,0x21
   319f9:	vpalignr ymm7,ymm8,ymm7,0xf
   319ff:	vpblendvb ymm1,ymm7,ymm8,ymm1
   31a05:	vmovdqa YMMWORD PTR [rsp+0x60],ymm3
   31a0b:	vmovdqa YMMWORD PTR [rsp+0x80],ymm4
   31a14:	vmovdqa YMMWORD PTR [rsp+0xa0],ymm5
   31a1d:	vmovdqa YMMWORD PTR [rsp+0xc0],ymm6
   31a26:	vmovdqa YMMWORD PTR [rsp+0xe0],ymm2
   31a2f:	vmovdqa YMMWORD PTR [rsp+0x100],ymm1
   31a38:	vpalignr xmm9,xmm9,xmm10,0xf
   31a3e:	vmovdqa YMMWORD PTR [rsp+0x40],ymm0
   31a44:	vpslldq xmm0,xmm10,0x1
   31a4a:	vmovd  xmm1,ecx
   31a4e:	vpor   xmm10,xmm1,xmm0
   31a52:	vmovd  ecx,xmm10
   31a56:	movzx  ecx,cl
   31a59:	mov    rdx,QWORD PTR [rsp+0x20]
   31a5e:	inc    DWORD PTR [rdx+rcx*4]
   31a61:	mov    BYTE PTR [r9+r12*1],cl
   31a65:	inc    r12
   31a68:	mov    rdi,r12
   31a6b:	cmp    rbx,r15
   31a6e:	jne    31759 <mibz2::mtf::mtf_avx2+0x139>
   31a74:	jmp    31a8e <mibz2::mtf::mtf_avx2+0x46e>
   31a76:	movabs rax,0x800000000000000c
   31a80:	mov    QWORD PTR [r14+0x10],rdi
   31a84:	mov    rcx,QWORD PTR [rsp+0x18]
   31a89:	jmp    31aa2 <mibz2::mtf::mtf_avx2+0x482>
   31a8b:	mov    r12,rdi
   31a8e:	mov    QWORD PTR [r14+0x10],r12
   31a92:	mov    rcx,QWORD PTR [rsp+0x18]
   31a97:	mov    QWORD PTR [rcx+0x8],r12
   31a9b:	mov    rax,0xffffffffffffffff
   31aa2:	mov    QWORD PTR [rcx],rax
   31aa5:	lea    rsp,[rbp-0x28]
   31aa9:	pop    rbx
   31aaa:	pop    r12
   31aac:	pop    r13
   31aae:	pop    r14
   31ab0:	pop    r15
   31ab2:	pop    rbp
   31ab3:	vzeroupper
   31ab6:	ret
   31ab7:	mov    rdi,r12
   31aba:	movabs rax,0x800000000000000c
   31ac4:	inc    rax
   31ac7:	jmp    31a80 <mibz2::mtf::mtf_avx2+0x460>
   31ac9:	mov    ecx,0x1
   31ace:	mov    r8d,0x1
   31ad4:	mov    rdi,r14
   31ad7:	xor    esi,esi
   31ad9:	mov    rdx,QWORD PTR [rsp+0x10]
   31ade:	call   2bc40 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   31ae3:	jmp    31663 <mibz2::mtf::mtf_avx2+0x43>
   31ae8:	int3
   31ae9:	int3
   31aea:	int3
   31aeb:	int3
   31aec:	int3
   31aed:	int3
   31aee:	int3
   31aef:	int3
