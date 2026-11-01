# The packed-token walks' ordinary steps, disassembled (BMI2; reports 138–139)

objdump -d of `target/release/examples/coop` at this commit, one binary: `room_h_bmi2` (b, the baseline), `room_hk_bmi2` (k) and `room_hj_bmi2` (j). Each listing is the FOR step and loop, then the test up to the sentinel branch, in address order. Counts in the journal, §8a.

## b (h compiled for BMI2): 37 instructions

```text
   53a80:	movzbl 0x8(%rsi),%edx
   53a84:	mov    %edi,%r11d
   53a87:	shr    $0x14,%r11d
   53a8b:	mov    %r11b,0x8(%rsi)
   53a8f:	and    $0xfffff,%edi
   53a95:	add    %edi,%r9d
   53a98:	mov    %r9d,(%rsi)
   53a9b:	and    $0xffffffe0,%r9d
   53a9f:	prefetcht0 (%r14,%r9,2)
   53aa4:	mov    0x140(%rsp,%rcx,8),%rcx
   53aac:	mov    %dl,(%rcx,%rax,1)
   53aaf:	lea    -0x1(%r8),%rdx
   53ab3:	add    $0x28,%rsi
   53ab7:	mov    %r8,%rcx
   53aba:	cmp    $0x1f,%rdx
   53abe:	jae    53a40 <<coop::compact::Compact>::room_h_bmi2+0x14d0>
   53ac0:	lea    0x1(%rcx),%r8
   53ac4:	mov    (%rsi),%r12d
   53ac7:	mov    %r12d,%edi
   53aca:	shr    $0x5,%edi
   53acd:	shl    $0x6,%rdi
   53ad1:	lea    (%r14,%rdi,1),%r9
   53ad5:	mov    0x38(%r14,%rdi,1),%rdi
   53ada:	mov    %r12d,%r11d
   53add:	and    $0x1f,%r11d
   53ae1:	mov    %r11,%r13
   53ae4:	add    %r11,%r13
   53ae7:	mov    %r11d,%edx
   53aea:	shl    $0x4,%edx
   53aed:	sub    %r13,%rdx
   53af0:	mov    %edx,%r13d
   53af3:	shr    $0x3,%r13d
   53af7:	and    $0x6,%dl
   53afa:	shrx   %edx,0x0(%r13,%r9,1),%r9d
   53b01:	and    $0x3fff,%r9d
   53b08:	cmp    $0x3fff,%r9d
   53b0f:	jne    53a80 <<coop::compact::Compact>::room_h_bmi2+0x1510>
```

## k (packed token, masked addressing): 31 instructions

```text
   5a4e0:	add    0x38(%r9),%r12d
   5a4e4:	mov    %r12d,(%r8)
   5a4e7:	shr    $0x14,%r14d
   5a4eb:	and    $0xfffe0,%r12d
   5a4f2:	prefetcht0 (%r15,%r12,2)
   5a4f7:	mov    0x140(%rsp,%rcx,8),%rcx
   5a4ff:	mov    %r14b,(%rcx,%rax,1)
   5a503:	lea    -0x1(%rsi),%rdx
   5a507:	add    $0x20,%r8
   5a50b:	mov    %rsi,%rcx
   5a50e:	cmp    $0x1f,%rdx
   5a512:	jae    5a4a0 <<coop::compact::Compact>::room_hk_bmi2+0x1450>
   5a514:	lea    0x1(%rcx),%rsi
   5a518:	mov    (%r8),%r14d
   5a51b:	mov    %r14d,%edi
   5a51e:	and    $0xfffe0,%edi
   5a524:	lea    (%r15,%rdi,2),%r9
   5a528:	mov    %r14d,%edx
   5a52b:	and    $0x1f,%edx
   5a52e:	mov    %rdx,%r12
   5a531:	add    %rdx,%r12
   5a534:	mov    %edx,%edi
   5a536:	shl    $0x4,%edi
   5a539:	sub    %r12,%rdi
   5a53c:	mov    %edi,%r12d
   5a53f:	shr    $0x3,%r12d
   5a543:	and    $0x6,%dil
   5a547:	shrx   %edi,(%r12,%r9,1),%r12d
   5a54d:	and    $0x3fff,%r12d
   5a554:	cmp    $0x3fff,%r12d
   5a55b:	jne    5a4e0 <<coop::compact::Compact>::room_hk_bmi2+0x1490>
```

## j (k with imul for 14 x slot): 27 instructions

```text
   58100:	add    0x38(%r9),%r12d
   58104:	mov    %r12d,(%r8)
   58107:	shr    $0x14,%edi
   5810a:	and    $0xfffe0,%r12d
   58111:	prefetcht0 (%r15,%r12,2)
   58116:	mov    0x140(%rsp,%rcx,8),%rcx
   5811e:	mov    %dil,(%rcx,%rax,1)
   58122:	lea    -0x1(%rsi),%rdx
   58126:	add    $0x20,%r8
   5812a:	mov    %rsi,%rcx
   5812d:	cmp    $0x1f,%rdx
   58131:	jae    58370 <<coop::compact::Compact>::room_hj_bmi2+0x1560>
   58137:	lea    0x1(%rcx),%rsi
   5813b:	mov    (%r8),%edi
   5813e:	mov    %edi,%edx
   58140:	and    $0xfffe0,%edx
   58146:	lea    (%r15,%rdx,2),%r9
   5814a:	mov    %edi,%edx
   5814c:	and    $0x1f,%edx
   5814f:	imul   $0xe,%edx,%r12d
   58153:	mov    %r12d,%r14d
   58156:	shr    $0x3,%r14d
   5815a:	and    $0x7,%r12b
   5815e:	shrx   %r12d,(%r14,%r9,1),%r12d
   58164:	and    $0x3fff,%r12d
   5816b:	cmp    $0x3fff,%r12d
   58172:	jne    58100 <<coop::compact::Compact>::room_hj_bmi2+0x12f0>
```
