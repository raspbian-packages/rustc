	.text
	.file	"20020904-1.c"
	.section	.text.fun,"ax",@progbits
	.hidden	fun                     # -- Begin function fun
	.globl	fun
	.type	fun,@function
fun:                                    # @fun
	.param  	i32
	.result 	i32
# BB#0:                                 # %entry
	i32.const	$push0=, 255
	i32.div_u	$push1=, $pop0, $0
                                        # fallthrough-return: $pop1
	.endfunc
.Lfunc_end0:
	.size	fun, .Lfunc_end0-fun
                                        # -- End function
	.section	.text.main,"ax",@progbits
	.hidden	main                    # -- Begin function main
	.globl	main
	.type	main,@function
main:                                   # @main
	.result 	i32
# BB#0:                                 # %if.end
	i32.const	$push0=, 0
                                        # fallthrough-return: $pop0
	.endfunc
.Lfunc_end1:
	.size	main, .Lfunc_end1-main
                                        # -- End function

	.ident	"clang version 6.0.0 (https://llvm.googlesource.com/clang.git a1774cccdccfa673c057f93ccf23bc2d8cb04932) (https://llvm.googlesource.com/llvm.git fc50e1c6121255333bc42d6faf2b524c074eae25)"