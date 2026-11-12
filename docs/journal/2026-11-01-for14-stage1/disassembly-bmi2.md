# The b walk's ordinary step, disassembled (BMI2; report 135)

objdump -d of `room_h_bmi2` in `target/release/examples/coop` at this commit: 53050-5309f (the test), then 53010-5304e (the FOR step and loop). 37 instructions; the `shrx` at 5308a replaces the baseline's `mov` + `shr %cl`.

```text
   53010:	movzbl 0x8(%rsi),%edx
   53014:	mov    %edi,%r11d
   53017:	shr    $0x14,%r11d
   5301b:	mov    %r11b,0x8(%rsi)
   5301f:	and    $0xfffff,%edi
   53025:	add    %edi,%r9d
   53028:	mov    %r9d,(%rsi)
   5302b:	and    $0xffffffe0,%r9d
   5302f:	prefetcht0 (%r14,%r9,2)
   53034:	mov    0x140(%rsp,%rcx,8),%rcx
   5303c:	mov    %dl,(%rcx,%rax,1)
   5303f:	lea    -0x1(%r8),%rdx
   53043:	add    $0x28,%rsi
   53047:	mov    %r8,%rcx
   5304a:	cmp    $0x1f,%rdx
   5304e:	jae    52fd0 <<coop::compact::Compact>::room_h_bmi2+0x14d0>
   53050:	lea    0x1(%rcx),%r8
   53054:	mov    (%rsi),%r12d
   53057:	mov    %r12d,%edi
   5305a:	shr    $0x5,%edi
   5305d:	shl    $0x6,%rdi
   53061:	lea    (%r14,%rdi,1),%r9
   53065:	mov    0x38(%r14,%rdi,1),%rdi
   5306a:	mov    %r12d,%r11d
   5306d:	and    $0x1f,%r11d
   53071:	mov    %r11,%r13
   53074:	add    %r11,%r13
   53077:	mov    %r11d,%edx
   5307a:	shl    $0x4,%edx
   5307d:	sub    %r13,%rdx
   53080:	mov    %edx,%r13d
   53083:	shr    $0x3,%r13d
   53087:	and    $0x6,%dl
   5308a:	shrx   %edx,0x0(%r13,%r9,1),%r9d
   53091:	and    $0x3fff,%r9d
   53098:	cmp    $0x3fff,%r9d
   5309f:	jne    53010 <<coop::compact::Compact>::room_h_bmi2+0x1510>
```
