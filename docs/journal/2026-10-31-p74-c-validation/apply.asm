000000000001d880 <p74_experiment::apply>:
   1d880:	push   rbx
   1d881:	sub    rsp,0x40
   1d885:	mov    r10d,r8d
   1d888:	mov    eax,ecx
   1d88a:	mov    r8d,edx
   1d88d:	mov    rdx,rsi
   1d890:	lea    rcx,[rsi+0x30]
   1d894:	test   r8d,r8d
   1d897:	je     1d8cd <p74_experiment::apply+0x4d>
   1d899:	mov    QWORD PTR [rsp],rdi
   1d89d:	mov    rsi,rsp
   1d8a0:	mov    rsi,QWORD PTR [rsp]
   1d8a4:	lea    r8,[rdx+0x18]
   1d8a8:	mov    BYTE PTR [rsp],r10b
   1d8ac:	mov    BYTE PTR [rsp+0x1],r9b
   1d8b1:	mov    rdi,rsp
   1d8b4:	movzx  edi,BYTE PTR [rsp]
   1d8b8:	mov    BYTE PTR [rsp],al
   1d8bb:	mov    rax,rsp
   1d8be:	movzx  eax,BYTE PTR [rsp]
   1d8c2:	cmp    eax,0x2
   1d8c5:	je     1d901 <p74_experiment::apply+0x81>
   1d8c7:	test   eax,eax
   1d8c9:	je     1d934 <p74_experiment::apply+0xb4>
   1d8cb:	jmp    1d91a <p74_experiment::apply+0x9a>
   1d8cd:	mov    QWORD PTR [rsp],rdi
   1d8d1:	mov    rsi,rsp
   1d8d4:	mov    rsi,QWORD PTR [rsp]
   1d8d8:	mov    BYTE PTR [rsp],r10b
   1d8dc:	mov    BYTE PTR [rsp+0x1],r9b
   1d8e1:	mov    rdi,rsp
   1d8e4:	movzx  edi,BYTE PTR [rsp]
   1d8e8:	mov    BYTE PTR [rsp],al
   1d8eb:	mov    rax,rsp
   1d8ee:	movzx  eax,BYTE PTR [rsp]
   1d8f2:	cmp    eax,0x2
   1d8f5:	je     1d951 <p74_experiment::apply+0xd1>
   1d8f7:	test   eax,eax
   1d8f9:	je     1d984 <p74_experiment::apply+0x104>
   1d8ff:	jmp    1d96a <p74_experiment::apply+0xea>
   1d901:	mov    r9,QWORD PTR [rsi+0x10]
   1d905:	cmp    r9,0x4000
   1d90c:	jb     1d934 <p74_experiment::apply+0xb4>
   1d90e:	add    r9,r9
   1d911:	cmp    r9,QWORD PTR [rsi+0x118]
   1d918:	jae    1d934 <p74_experiment::apply+0xb4>
   1d91a:	test   dil,dil
   1d91d:	je     1d945 <p74_experiment::apply+0xc5>
   1d91f:	xor    r9d,r9d
   1d922:	cmp    al,0x1
   1d924:	sete   r9b
   1d928:	lea    rdi,[rsp+0x20]
   1d92d:	call   349c0 <mibz2::mtf::p74_c::combined>
   1d932:	jmp    1d99f <p74_experiment::apply+0x11f>
   1d934:	test   dil,dil
   1d937:	je     1d945 <p74_experiment::apply+0xc5>
   1d939:	lea    rdi,[rsp+0x20]
   1d93e:	call   2beb0 <mibz2::mtf::mtf_checkpoints_avx2>
   1d943:	jmp    1d99f <p74_experiment::apply+0x11f>
   1d945:	lea    rdi,[rsp+0x20]
   1d94a:	call   2bd30 <mibz2::mtf::mtf_checkpoints_scalar>
   1d94f:	jmp    1d99f <p74_experiment::apply+0x11f>
   1d951:	mov    r8,QWORD PTR [rsi+0x10]
   1d955:	cmp    r8,0x4000
   1d95c:	jb     1d984 <p74_experiment::apply+0x104>
   1d95e:	add    r8,r8
   1d961:	cmp    r8,QWORD PTR [rsi+0x118]
   1d968:	jae    1d984 <p74_experiment::apply+0x104>
   1d96a:	test   dil,dil
   1d96d:	je     1d995 <p74_experiment::apply+0x115>
   1d96f:	xor    r8d,r8d
   1d972:	cmp    al,0x1
   1d974:	sete   r8b
   1d978:	lea    rdi,[rsp+0x20]
   1d97d:	call   34340 <mibz2::mtf::p74_c::combined>
   1d982:	jmp    1d99f <p74_experiment::apply+0x11f>
   1d984:	test   dil,dil
   1d987:	je     1d995 <p74_experiment::apply+0x115>
   1d989:	lea    rdi,[rsp+0x20]
   1d98e:	call   34010 <mibz2::mtf::mtf_avx2>
   1d993:	jmp    1d99f <p74_experiment::apply+0x11f>
   1d995:	lea    rdi,[rsp+0x20]
   1d99a:	call   31100 <mibz2::mtf::mtf_scalar>
   1d99f:	mov    rcx,QWORD PTR [rsp+0x20]
   1d9a4:	mov    rax,QWORD PTR [rsp+0x28]
   1d9a9:	movabs rdx,0x8000000000000018
   1d9b3:	cmp    rcx,rdx
   1d9b6:	jne    1d9c9 <p74_experiment::apply+0x149>
   1d9b8:	mov    QWORD PTR [rsp],rax
   1d9bc:	mov    rax,rsp
   1d9bf:	mov    rax,QWORD PTR [rsp]
   1d9c3:	add    rsp,0x40
   1d9c7:	pop    rbx
   1d9c8:	ret
   1d9c9:	mov    QWORD PTR [rsp],rcx
   1d9cd:	mov    QWORD PTR [rsp+0x8],rax
   1d9d2:	movups xmm0,XMMWORD PTR [rsp+0x30]
   1d9d7:	movups XMMWORD PTR [rsp+0x10],xmm0
   1d9dc:	lea    rdi,[rip+0xffffffffffff04d9]        # debc <mibz2::mtf::p74::BLEND+0x81c>
   1d9e3:	lea    rcx,[rip+0x647d6]        # 821c0 <__frame_dummy_init_array_entry+0x20>
   1d9ea:	lea    r8,[rip+0x64c2f]        # 82620 <__frame_dummy_init_array_entry+0x480>
   1d9f1:	mov    rdx,rsp
   1d9f4:	mov    esi,0x2b
   1d9f9:	call   289c0 <core::result::unwrap_failed>
   1d9fe:	ud2
   1da00:	mov    rbx,rax
   1da03:	mov    rdi,rsp
   1da06:	call   19530 <core::ptr::drop_in_place<mibz2::ErrorKind>>
   1da0b:	mov    rdi,rbx
   1da0e:	call   81160 <_Unwind_Resume@plt>
   1da13:	int3
   1da14:	int3
   1da15:	int3
   1da16:	int3
   1da17:	int3
   1da18:	int3
   1da19:	int3
   1da1a:	int3
   1da1b:	int3
   1da1c:	int3
   1da1d:	int3
   1da1e:	int3
   1da1f:	int3

