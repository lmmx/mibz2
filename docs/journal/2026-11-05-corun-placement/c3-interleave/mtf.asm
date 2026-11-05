
scratch/so2cg/sonly2/target/release/c30cg:     file format elf64-x86-64


Disassembly of section .text:

0000000000050170 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2>:
   50170:	push   rbp
   50171:	mov    rbp,rsp
   50174:	push   r15
   50176:	push   r14
   50178:	push   r13
   5017a:	push   r12
   5017c:	push   rbx
   5017d:	and    rsp,0xffffffffffffffc0
   50181:	sub    rsp,0x1c0
   50188:	mov    r15,rcx
   5018b:	mov    rbx,rdx
   5018e:	mov    r13,rsi
   50191:	mov    r14,rdi
   50194:	mov    rax,QWORD PTR [rsi+0x118]
   5019b:	mov    QWORD PTR [rdx+0x10],0x0
   501a3:	mov    QWORD PTR [rsp+0x20],rax
   501a8:	cmp    rax,QWORD PTR [rdx]
   501ab:	ja     504de <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x36e>
   501b1:	xor    r12d,r12d
   501b4:	mov    edx,0x400
   501b9:	mov    rdi,r15
   501bc:	xor    esi,esi
   501be:	call   QWORD PTR [rip+0x455fc]        # 957c0 <memset@GLIBC_2.2.5>
   501c4:	vxorps xmm0,xmm0,xmm0
   501c8:	vmovaps YMMWORD PTR [rsp+0x160],ymm0
   501d1:	vmovaps YMMWORD PTR [rsp+0x140],ymm0
   501da:	vmovaps YMMWORD PTR [rsp+0x120],ymm0
   501e3:	vmovaps YMMWORD PTR [rsp+0x100],ymm0
   501ec:	vmovaps YMMWORD PTR [rsp+0xe0],ymm0
   501f5:	vmovaps YMMWORD PTR [rsp+0xc0],ymm0
   501fe:	vmovaps YMMWORD PTR [rsp+0xa0],ymm0
   50207:	vmovaps YMMWORD PTR [rsp+0x80],ymm0
   50210:	vmovaps YMMWORD PTR [rsp+0x60],ymm0
   50216:	vmovaps YMMWORD PTR [rsp+0x40],ymm0
   5021c:	vmovups ymm0,YMMWORD PTR [r13+0xf8]
   50225:	vmovups YMMWORD PTR [rsp+0x140],ymm0
   5022e:	vmovups ymm0,YMMWORD PTR [r13+0xd8]
   50237:	vmovaps YMMWORD PTR [rsp+0x120],ymm0
   50240:	vmovups ymm0,YMMWORD PTR [r13+0xb8]
   50249:	vmovups YMMWORD PTR [rsp+0x100],ymm0
   50252:	vmovups ymm0,YMMWORD PTR [r13+0x98]
   5025b:	vmovaps YMMWORD PTR [rsp+0xe0],ymm0
   50264:	vmovdqu ymm0,YMMWORD PTR [r13+0x18]
   5026a:	vmovdqu ymm1,YMMWORD PTR [r13+0x38]
   50270:	vmovdqu ymm2,YMMWORD PTR [r13+0x58]
   50276:	vmovups ymm3,YMMWORD PTR [r13+0x78]
   5027c:	vmovups YMMWORD PTR [rsp+0xc0],ymm3
   50285:	vmovdqa YMMWORD PTR [rsp+0xa0],ymm2
   5028e:	vmovdqu YMMWORD PTR [rsp+0x80],ymm1
   50297:	vmovdqa YMMWORD PTR [rsp+0x60],ymm0
   5029d:	mov    rax,QWORD PTR [r13+0x10]
   502a1:	test   rax,rax
   502a4:	mov    QWORD PTR [rsp+0x28],rbx
   502a9:	je     504a3 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x333>
   502af:	mov    QWORD PTR [rsp+0x30],r15
   502b4:	lea    r8,[rsp+0x60]
   502b9:	mov    r9,QWORD PTR [rbx+0x8]
   502bd:	mov    rbx,QWORD PTR [r13+0x8]
   502c1:	lea    r15,[rbx+rax*2]
   502c5:	movzx  r10d,WORD PTR [r13+0x128]
   502cd:	mov    r13d,0x1
   502d3:	lea    r11,[rsp+0x61]
   502d8:	xor    edi,edi
   502da:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffb811e]        # 8400 <GCC_except_table701+0x920>
   502e2:	mov    QWORD PTR [rsp+0x18],r14
   502e7:	xor    eax,eax
   502e9:	nop    DWORD PTR [rax+0x0]
   502f0:	movzx  r14d,WORD PTR [rbx]
   502f4:	add    rbx,0x2
   502f8:	cmp    r14d,0x2
   502fc:	jae    50330 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x1c0>
   502fe:	cmp    r13d,0x1fffff
   50305:	ja     50481 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x311>
   5030b:	inc    r14d
   5030e:	imul   r14d,r13d
   50312:	add    eax,r14d
   50315:	add    r13d,r13d
   50318:	cmp    rbx,r15
   5031b:	jne    502f0 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x180>
   5031d:	jmp    5049b <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x32b>
   50322:	data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
   50330:	test   eax,eax
   50332:	je     50389 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x219>
   50334:	mov    edx,eax
   50336:	lea    r12,[rdi+rdx*1]
   5033a:	cmp    r12,QWORD PTR [rsp+0x20]
   5033f:	ja     504cf <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x35f>
   50345:	movzx  ecx,BYTE PTR [rsp+0x60]
   5034a:	mov    rsi,QWORD PTR [rsp+0x30]
   5034f:	add    DWORD PTR [rsi+rcx*4],eax
   50352:	cmp    eax,0x40
   50355:	ja     50397 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x227>
   50357:	lea    rax,[rdi+0x40]
   5035b:	cmp    rax,QWORD PTR [rsp+0x20]
   50360:	ja     50397 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x227>
   50362:	vmovd  xmm0,ecx
   50366:	vpbroadcastb ymm0,xmm0
   5036b:	vmovdqu YMMWORD PTR [r9+rdi*1],ymm0
   50371:	vmovdqu YMMWORD PTR [r9+rdi*1+0x20],ymm0
   50378:	mov    r13d,0x1
   5037e:	cmp    r14w,r10w
   50382:	jne    503da <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x26a>
   50384:	jmp    5049e <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x32e>
   50389:	mov    r12,rdi
   5038c:	cmp    r14w,r10w
   50390:	jne    503da <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x26a>
   50392:	jmp    5049e <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x32e>
   50397:	add    rdi,r9
   5039a:	movzx  esi,cl
   5039d:	mov    r13,r9
   503a0:	mov    WORD PTR [rsp+0x16],r10w
   503a6:	vzeroupper
   503a9:	call   QWORD PTR [rip+0x45411]        # 957c0 <memset@GLIBC_2.2.5>
   503af:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffb8049]        # 8400 <GCC_except_table701+0x920>
   503b7:	lea    r11,[rsp+0x61]
   503bc:	movzx  r10d,WORD PTR [rsp+0x16]
   503c2:	mov    r9,r13
   503c5:	lea    r8,[rsp+0x60]
   503ca:	mov    r13d,0x1
   503d0:	cmp    r14w,r10w
   503d4:	je     5049e <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x32e>
   503da:	cmp    r12,QWORD PTR [rsp+0x20]
   503df:	jae    504cc <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x35c>
   503e5:	mov    eax,r14d
   503e8:	dec    r14d
   503eb:	movzx  edx,r14w
   503ef:	movzx  r14d,BYTE PTR [rsp+rdx*1+0x60]
   503f5:	cmp    ax,0x21
   503f9:	jae    50420 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x2b0>
   503fb:	vmovdqa ymm0,YMMWORD PTR [rsp+0x60]
   50401:	inc    dl
   50403:	vmovd  xmm1,edx
   50407:	vpbroadcastb ymm1,xmm1
   5040c:	vpcmpgtb ymm1,ymm1,ymm2
   50410:	vpblendvb ymm0,ymm0,YMMWORD PTR [rsp+0x5f],ymm1
   50418:	vmovdqa YMMWORD PTR [rsp+0x60],ymm0
   5041e:	jmp    50457 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x2e7>
   50420:	mov    rdi,r11
   50423:	mov    rsi,r8
   50426:	mov    QWORD PTR [rsp+0x38],r9
   5042b:	mov    WORD PTR [rsp+0x16],r10w
   50431:	vzeroupper
   50434:	call   QWORD PTR [rip+0x45346]        # 95780 <memmove@GLIBC_2.2.5>
   5043a:	vmovdqa ymm2,YMMWORD PTR [rip+0xfffffffffffb7fbe]        # 8400 <GCC_except_table701+0x920>
   50442:	lea    r11,[rsp+0x61]
   50447:	movzx  r10d,WORD PTR [rsp+0x16]
   5044d:	mov    r9,QWORD PTR [rsp+0x38]
   50452:	lea    r8,[rsp+0x60]
   50457:	movzx  eax,r14b
   5045b:	mov    BYTE PTR [rsp+0x60],al
   5045f:	mov    rcx,QWORD PTR [rsp+0x30]
   50464:	inc    DWORD PTR [rcx+rax*4]
   50467:	mov    BYTE PTR [r9+r12*1],al
   5046b:	inc    r12
   5046e:	mov    rdi,r12
   50471:	cmp    rbx,r15
   50474:	mov    r14,QWORD PTR [rsp+0x18]
   50479:	jne    502e7 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x177>
   5047f:	jmp    504a3 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x333>
   50481:	movabs rax,0x800000000000000c
   5048b:	mov    rcx,QWORD PTR [rsp+0x28]
   50490:	mov    QWORD PTR [rcx+0x10],rdi
   50494:	mov    r14,QWORD PTR [rsp+0x18]
   50499:	jmp    504b7 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x347>
   5049b:	mov    r12,rdi
   5049e:	mov    r14,QWORD PTR [rsp+0x18]
   504a3:	mov    rax,QWORD PTR [rsp+0x28]
   504a8:	mov    QWORD PTR [rax+0x10],r12
   504ac:	mov    QWORD PTR [r14+0x8],r12
   504b0:	mov    rax,0xffffffffffffffff
   504b7:	mov    QWORD PTR [r14],rax
   504ba:	lea    rsp,[rbp-0x28]
   504be:	pop    rbx
   504bf:	pop    r12
   504c1:	pop    r13
   504c3:	pop    r14
   504c5:	pop    r15
   504c7:	pop    rbp
   504c8:	vzeroupper
   504cb:	ret
   504cc:	mov    rdi,r12
   504cf:	movabs rax,0x800000000000000c
   504d9:	inc    rax
   504dc:	jmp    5048b <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x31b>
   504de:	mov    ecx,0x1
   504e3:	mov    r8d,0x1
   504e9:	mov    rdi,rbx
   504ec:	xor    esi,esi
   504ee:	mov    rdx,QWORD PTR [rsp+0x20]
   504f3:	call   35da0 <_RINvNvMs2_NtCs40k4W9msRzi_5alloc7raw_vecINtB8_11RawVecInnerpE7reserve21do_reserve_and_handleNtNtBa_5alloc6GlobalECsifqXOHMcNaa_5mibz2>
   504f8:	jmp    501b1 <_RNvNtCsifqXOHMcNaa_5mibz23mtf8mtf_avx2+0x41>
   504fd:	int3
   504fe:	int3
   504ff:	int3

0000000000050500 <_RINvNtNtCsgczF5crJ4sT_3std3sys9backtrace28___rust_begin_short_backtraceNCNCNCINvNtNtB6_6thread9lifecycle15spawn_uncheckedNCINvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB23_13StreamDecoder12with_inverseINtNtB25_3bwt11CooperativeKj2_EE0uEs_000uEB25_>:
   50500:	push   rax
   50501:	call   8ef70 <_RNvMs_NtNtCsgczF5crJ4sT_3std6thread9spawnhookNtB4_15ChildSpawnHooks3run>
   50506:	pop    rax
   50507:	ret
   50508:	int3
   50509:	int3
   5050a:	int3
   5050b:	int3
   5050c:	int3
   5050d:	int3
   5050e:	int3
   5050f:	int3

0000000000050510 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close>:
   50510:	push   rbp
   50511:	push   r14
   50513:	push   rbx
   50514:	mov    r14,rdi
   50517:	lea    rbx,[rdi+0x8]
   5051b:	mov    ecx,0x1
   50520:	xor    eax,eax
   50522:	lock cmpxchg DWORD PTR [rdi+0x8],ecx
   50527:	jne    50583 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x73>
   50529:	mov    rax,QWORD PTR [rip+0x46e98]        # 973c8 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count18GLOBAL_PANIC_COUNT>
   50530:	shl    rax,1
   50533:	test   rax,rax
   50536:	jne    5059a <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x8a>
   50538:	xor    ebp,ebp
   5053a:	movzx  eax,BYTE PTR [r14+0xc]
   5053f:	mov    rdi,QWORD PTR [r14+0x10]
   50543:	mov    rsi,QWORD PTR [r14+0x18]
   50547:	mov    QWORD PTR [r14+0x10],0xffffffffffffffff
   5054f:	cmp    rdi,0xffffffffffffffff
   50553:	je     5055a <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x4a>
   50555:	call   50650 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2>
   5055a:	test   bpl,bpl
   5055d:	jne    50575 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x65>
   5055f:	movabs rax,0x7fffffffffffffff
   50569:	mov    rcx,QWORD PTR [rip+0x46e58]        # 973c8 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count18GLOBAL_PANIC_COUNT>
   50570:	test   rcx,rax
   50573:	jne    505c5 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0xb5>
   50575:	xor    eax,eax
   50577:	xchg   DWORD PTR [rbx],eax
   50579:	cmp    eax,0x2
   5057c:	je     505a7 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x97>
   5057e:	pop    rbx
   5057f:	pop    r14
   50581:	pop    rbp
   50582:	ret
   50583:	mov    rdi,rbx
   50586:	call   61f80 <_RNvMNtNtNtNtCsgczF5crJ4sT_3std3sys4sync5mutex5futexNtB2_5Mutex14lock_contended>
   5058b:	mov    rax,QWORD PTR [rip+0x46e36]        # 973c8 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count18GLOBAL_PANIC_COUNT>
   50592:	shl    rax,1
   50595:	test   rax,rax
   50598:	je     50538 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x28>
   5059a:	call   61d10 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count17is_zero_slow_path>
   5059f:	mov    ebp,eax
   505a1:	xor    bpl,0x1
   505a5:	jmp    5053a <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x2a>
   505a7:	mov    edi,0xca
   505ac:	mov    rsi,rbx
   505af:	mov    edx,0x81
   505b4:	mov    ecx,0x1
   505b9:	xor    eax,eax
   505bb:	pop    rbx
   505bc:	pop    r14
   505be:	pop    rbp
   505bf:	jmp    QWORD PTR [rip+0x451ab]        # 95770 <syscall@GLIBC_2.2.5>
   505c5:	call   61d10 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count17is_zero_slow_path>
   505ca:	test   al,al
   505cc:	jne    50575 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x65>
   505ce:	mov    BYTE PTR [r14+0xc],0x1
   505d3:	jmp    50575 <_RNvMs_NtCsifqXOHMcNaa_5mibz28parallelNtB4_13StreamDecoder5close+0x65>
   505d5:	mov    r14,rax
   505d8:	movzx  esi,bpl
   505dc:	mov    rdi,rbx
   505df:	call   505f0 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2>
   505e4:	mov    rdi,r14
   505e7:	call   919d0 <_Unwind_Resume@plt>
   505ec:	int3
   505ed:	int3
   505ee:	int3
   505ef:	int3

00000000000505f0 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2>:
   505f0:	push   rbx
   505f1:	test   sil,0x1
   505f5:	jne    50606 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2+0x16>
   505f7:	mov    rax,QWORD PTR [rip+0x46dca]        # 973c8 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count18GLOBAL_PANIC_COUNT>
   505fe:	shl    rax,1
   50601:	test   rax,rax
   50604:	jne    5062c <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2+0x3c>
   50606:	xor    eax,eax
   50608:	xchg   DWORD PTR [rdi],eax
   5060a:	cmp    eax,0x2
   5060d:	je     50611 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2+0x21>
   5060f:	pop    rbx
   50610:	ret
   50611:	mov    rsi,rdi
   50614:	mov    edi,0xca
   50619:	mov    edx,0x81
   5061e:	mov    ecx,0x1
   50623:	xor    eax,eax
   50625:	pop    rbx
   50626:	jmp    QWORD PTR [rip+0x45144]        # 95770 <syscall@GLIBC_2.2.5>
   5062c:	mov    rbx,rdi
   5062f:	call   61d10 <_RNvNtNtCsgczF5crJ4sT_3std9panicking11panic_count17is_zero_slow_path>
   50634:	mov    rdi,rbx
   50637:	test   al,al
   50639:	jne    50606 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2+0x16>
   5063b:	mov    BYTE PTR [rdi+0x4],0x1
   5063f:	jmp    50606 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex10MutexGuardINtNtB4_6option6OptionINtNtBI_4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEEEECsifqXOHMcNaa_5mibz2+0x16>
   50641:	int3
   50642:	int3
   50643:	int3
   50644:	int3
   50645:	int3
   50646:	int3
   50647:	int3
   50648:	int3
   50649:	int3
   5064a:	int3
   5064b:	int3
   5064c:	int3
   5064d:	int3
   5064e:	int3
   5064f:	int3

0000000000050650 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2>:
   50650:	push   r14
   50652:	push   rbx
   50653:	push   rax
   50654:	mov    rbx,rsi
   50657:	test   rdi,rdi
   5065a:	je     506a8 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x58>
   5065c:	cmp    rdi,0x1
   50660:	jne    506ee <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x9e>
   50666:	lock dec QWORD PTR [rbx+0x180]
   5066e:	jne    506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   50670:	lock bts QWORD PTR [rbx+0x80],0x0
   5067a:	jb     50688 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x38>
   5067c:	lea    rdi,[rbx+0x100]
   50683:	call   3df20 <_RNvMs0_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5wakerNtB5_9SyncWaker10disconnect.304>
   50688:	mov    al,0x1
   5068a:	xchg   BYTE PTR [rbx+0x190],al
   50690:	test   al,al
   50692:	je     506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   50694:	mov    QWORD PTR [rsp],rbx
   50698:	mov    rdi,rsp
   5069b:	call   4ace0 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtCs40k4W9msRzi_5alloc5boxed3BoxINtNtNtNtCsgczF5crJ4sT_3std4sync4mpmc7counter7CounterINtNtB1f_4list7ChannelINtNtBG_3vec3VechEEEEECsifqXOHMcNaa_5mibz2>
   506a0:	add    rsp,0x8
   506a4:	pop    rbx
   506a5:	pop    r14
   506a7:	ret
   506a8:	lock dec QWORD PTR [rbx+0x200]
   506b0:	jne    506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   506b2:	mov    rax,QWORD PTR [rbx+0x80]
   506b9:	mov    rcx,QWORD PTR [rbx+0x190]
   506c0:	mov    rdx,rax
   506c3:	or     rdx,rcx
   506c6:	lock cmpxchg QWORD PTR [rbx+0x80],rdx
   506cf:	jne    506c0 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x70>
   506d1:	test   QWORD PTR [rbx+0x190],rax
   506d8:	je     50713 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0xc3>
   506da:	mov    al,0x1
   506dc:	xchg   BYTE PTR [rbx+0x210],al
   506e2:	test   al,al
   506e4:	jne    5072b <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0xdb>
   506e6:	add    rsp,0x8
   506ea:	pop    rbx
   506eb:	pop    r14
   506ed:	ret
   506ee:	lock dec QWORD PTR [rbx+0x70]
   506f3:	jne    506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   506f5:	mov    rdi,rbx
   506f8:	call   4adc0 <_RNvMs1_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc4zeroINtB5_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE10disconnectCsifqXOHMcNaa_5mibz2>
   506fd:	mov    al,0x1
   506ff:	xchg   BYTE PTR [rbx+0x80],al
   50705:	test   al,al
   50707:	je     506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   50709:	mov    rdi,rbx
   5070c:	call   3e350 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync6poison5mutex5MutexNtNtNtBI_4mpmc4zero5InnerEECsifqXOHMcNaa_5mibz2>
   50711:	jmp    50733 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0xe3>
   50713:	lea    rdi,[rbx+0x140]
   5071a:	call   3df20 <_RNvMs0_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5wakerNtB5_9SyncWaker10disconnect.304>
   5071f:	mov    al,0x1
   50721:	xchg   BYTE PTR [rbx+0x210],al
   50727:	test   al,al
   50729:	je     506e6 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0x96>
   5072b:	mov    rdi,rbx
   5072e:	call   4ac30 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtNtCsgczF5crJ4sT_3std4sync4mpmc7counter7CounterINtNtBG_5array7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEEEECsifqXOHMcNaa_5mibz2>
   50733:	mov    rdi,rbx
   50736:	add    rsp,0x8
   5073a:	pop    rbx
   5073b:	pop    r14
   5073d:	jmp    QWORD PTR [rip+0x45025]        # 95768 <free@GLIBC_2.2.5>
   50743:	jmp    50745 <_RINvNtCscI6d9CVNmLh_4core3ptr9drop_glueINtNtNtCsgczF5crJ4sT_3std4sync4mpsc10SyncSenderINtNtCs40k4W9msRzi_5alloc3vec3VechEEECsifqXOHMcNaa_5mibz2+0xf5>
   50745:	mov    r14,rax
   50748:	mov    rdi,rbx
   5074b:	call   QWORD PTR [rip+0x45017]        # 95768 <free@GLIBC_2.2.5>
   50751:	mov    rdi,r14
   50754:	call   919d0 <_Unwind_Resume@plt>
   50759:	int3
   5075a:	int3
   5075b:	int3
   5075c:	int3
   5075d:	int3
   5075e:	int3
   5075f:	int3

0000000000050760 <_RNCNvMs_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5arrayINtB6_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE4send0CsifqXOHMcNaa_5mibz2>:
   50760:	push   rbp
   50761:	push   r15
   50763:	push   r14
   50765:	push   r13
   50767:	push   r12
   50769:	push   rbx
   5076a:	sub    rsp,0x68
   5076e:	mov    r15,rsi
   50771:	mov    rbx,rdi
   50774:	mov    rsi,QWORD PTR [rdi]
   50777:	mov    r14,QWORD PTR [rdi+0x8]
   5077b:	lea    rdi,[r14+0x100]
   50782:	mov    QWORD PTR [rsp+0x30],rdi
   50787:	mov    QWORD PTR [rsp+0x38],rsi
   5078c:	mov    rdx,r15
   5078f:	call   439b0 <_RNvMs0_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5wakerNtB5_9SyncWaker8register>
   50794:	mov    rax,QWORD PTR [r14+0x80]
   5079b:	mov    rcx,QWORD PTR [r14]
   5079e:	add    rcx,QWORD PTR [r14+0x188]
   507a5:	mov    rdx,QWORD PTR [r14+0x190]
   507ac:	not    rdx
   507af:	and    rdx,rax
   507b2:	cmp    rcx,rdx
   507b5:	jne    507c7 <_RNCNvMs_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5arrayINtB6_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE4send0CsifqXOHMcNaa_5mibz2+0x67>
   507b7:	mov    rax,QWORD PTR [r14+0x80]
   507be:	test   QWORD PTR [r14+0x190],rax
   507c5:	je     507d4 <_RNCNvMs_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5arrayINtB6_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE4send0CsifqXOHMcNaa_5mibz2+0x74>
   507c7:	mov    ecx,0x1
   507cc:	xor    eax,eax
   507ce:	lock cmpxchg QWORD PTR [r15+0x18],rcx
   507d4:	mov    rax,QWORD PTR [rbx+0x10]
   507d8:	mov    ecx,DWORD PTR [rax+0x8]
   507db:	cmp    ecx,0xffffffff
   507de:	je     50954 <_RNCNvMs_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5arrayINtB6_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE4send0CsifqXOHMcNaa_5mibz2+0x1f4>
   507e4:	mov    DWORD PTR [rsp+0x2c],ecx
   507e8:	mov    rax,QWORD PTR [rax]
   507eb:	mov    QWORD PTR [rsp+0x40],rax
   507f0:	mov    rbp,QWORD PTR [rip+0x44f79]        # 95770 <syscall@GLIBC_2.2.5>
   507f7:	mov    r12,QWORD PTR [rip+0x44fba]        # 957b8 <__errno_location@GLIBC_2.2.5>
   507fe:	jmp    50804 <_RNCNvMs_NtNtNtCsgczF5crJ4sT_3std4sync4mpmc5arrayINtB6_7ChannelINtNtCs40k4W9msRzi_5alloc3vec3VechEE4send0CsifqXOHMcNaa_5mibz2+0xa4>
