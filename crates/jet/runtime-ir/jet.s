	.section	__TEXT,__text,regular,pure_instructions
	.build_version macos, 14, 0
	.globl	_jet.stack.push.i256            ## -- Begin function jet.stack.push.i256
	.p2align	4, 0x90
_jet.stack.push.i256:                   ## @jet.stack.push.i256
	.cfi_startproc
## %bb.0:                               ## %entry
	movslq	(%rdi), %rax
	leal	1(%rax), %r9d
	shlq	$5, %rax
	movq	%r8, 48(%rdi,%rax)
	movq	%rcx, 40(%rdi,%rax)
	movq	%rdx, 32(%rdi,%rax)
	movq	%rsi, 24(%rdi,%rax)
	movl	%r9d, (%rdi)
	movb	$1, %al
	retq
	.cfi_endproc
                                        ## -- End function
.subsections_via_symbols
