; ModuleID = 'JetVM Runtime'
source_filename = "jetvm.ll"

; 'e' - Little-endian
; 'm:o' - MACH-O mangling
; 'p270:32:32' - Addr space with 32-bit pointers, 32-bit alignment (TODO: Do we need these?)
; 'p271:32:32' - Addr space with 32-bit pointers, 32-bit alignment (TODO: Do we need these?)
; 'p272:64:64' - Addr space with 64-bit pointers, 64-bit alignment (TODO: Do we need these?)
; 'i64:64' - 64-bit integers have 64-bit alignment (Other widths are natural aligned by default)
; 'n8:16:32:64' - Native integer widths
; 'S128' - Stack alignment of 128 bits (TODO: is this ideal?)
;
; Original before modification:
; target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
;
; Changes:
;   - Removed 'f80:128'
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx14.0.0"



%jetvm.types.mem_buf = type [32448 x i8]

%jetvm.types.mem = type <{
  %jetvm.types.mem_buf *, ; data
  i32, ; length
  i32 ; capacity
}>

%jetvm.types.block_info = type <{
  i160, ; address
  i256, ; balance
  i160, ; origin
  i32,  ; code size
  i256 ; gas price
}>

%jetvm.types.call_info = type <{}>

%jetvm.types.exec_ctx = type <{
  i32, ; stack_ptr
  i32, ; jump_ptr
  i32, ; return offset
  i32, ; return length
  [1024 x i256], ; stack
  %jetvm.types.mem ; memory
}>

%jetvm.types.contract_fn = type i8(%jetvm.types.exec_ctx*)

@jetvm.jit_engine = external global ptr

; Pushes a word onto the stack and incs the stack ptr.
; Returns true if the operation was successful, false if the stack is full.
define i1 @jetvm.stack.push_word (%jetvm.types.exec_ctx*, i256) {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %jetvm.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_offset_ptr = getelementptr inbounds %jetvm.types.exec_ctx, ptr %0, i32 0, i32 4, i32 %stack_ptr

  ; TODO: Check if we'll break the stack

  ; Store word
  store i256 %1, ptr %stack_offset_ptr, align 8

  ; Increment stack pointer
  %stack_ptr_next = add i32 %stack_ptr, 1
  store i32 %stack_ptr_next, ptr %stack_ptr_gep, align 4

  ret i1 true
}

; Pushes an array of 32 bytes onto the stack as a word, and incs the stack ptr.
; Returns true if the operation was successful, false if the stack is full.
define i1 @jetvm.stack.push_bytes (%jetvm.types.exec_ctx*, [32 x i8]) {
entry:
  ; Cast byte array to word and send to @jetvm.stack.push_word
  %stack_bytes_ptr = alloca [32 x i8]
  store [32 x i8] %1, [32 x i8]* %stack_bytes_ptr
  %stack_word = load i256, ptr %stack_bytes_ptr, align 8
  %result = call i1 @jetvm.stack.push_word (%jetvm.types.exec_ctx* %0, i256 %stack_word)
  ret i1 %result
}

define i256 @jetvm.stack.pop (%jetvm.types.exec_ctx* %0) {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %jetvm.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_ptr_sub_1 = sub i32 %stack_ptr, 1
  %stack_offset_ptr = getelementptr inbounds %jetvm.types.exec_ctx, ptr %0, i32 0, i32 4, i32 %stack_ptr_sub_1

  ; Load word
  %stack_word = load i256, ptr %stack_offset_ptr, align 8

  ; Decrement stack pointer
  store i32 %stack_ptr_sub_1, ptr %stack_ptr_gep, align 4

  ; ret i256 02
  ret i256 %stack_word
}

define i8 @jetvm.mem.store_word (%jetvm.types.exec_ctx* %ctx, i256 %loc, i256 %val) {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jetvm.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_buf_ptr = getelementptr inbounds %jetvm.types.mem, ptr %mem, i32 0, i32 0
  %mem_loc_ptr = getelementptr inbounds %jetvm.types.mem_buf, ptr %mem_buf_ptr, i32 0, i32 %loc_i32

  store i256 %val, ptr %mem_loc_ptr, align 1

  ret i8 0
}

define i8 @jetvm.mem.store_byte (%jetvm.types.exec_ctx* %ctx, i256 %loc, i256 %val) {
entry:
  %loc_i32 = trunc i256 %loc to i32
  %val_i8 = trunc i256 %val to i8

  %mem = getelementptr inbounds %jetvm.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_buf_ptr = getelementptr inbounds %jetvm.types.mem, ptr %mem, i32 0, i32 0
  %mem_loc_ptr = getelementptr inbounds %jetvm.types.mem_buf, ptr %mem_buf_ptr, i32 0, i32 %loc_i32

  store i8 %val_i8, ptr %mem_loc_ptr, align 1
  ret i8 0
}

define i256 @jetvm.mem.load_word (%jetvm.types.exec_ctx* %ctx, i256 %loc) {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jetvm.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_buf_ptr = getelementptr inbounds %jetvm.types.mem, ptr %mem, i32 0, i32 0
  %mem_loc_ptr = getelementptr inbounds %jetvm.types.mem_buf, ptr %mem_buf_ptr, i32 0, i32 %loc_i32

  %val = load i256, ptr %mem_loc_ptr, align 1
  ret i256 %val
}

define i1 @jetvm.mem.copy (%jetvm.types.exec_ctx* %caller_ctx, i256 %caller_offset, %jetvm.types.exec_ctx* %callee_ctx, i256 %callee_offset, i256 %copy_len) {
entry:
  %caller_offset_i32 = trunc i256 %caller_offset to i32
  %callee_offset_i32 = trunc i256 %callee_offset to i32

  %caller_mem = getelementptr inbounds %jetvm.types.exec_ctx, ptr %caller_ctx, i32 0, i32 5
  %caller_mem_buf_ptr = getelementptr inbounds %jetvm.types.mem, ptr %caller_mem, i32 0, i32 0
  %caller_mem_loc_ptr = getelementptr inbounds %jetvm.types.mem_buf, ptr %caller_mem_buf_ptr, i32 0, i32 %caller_offset_i32

  %callee_mem = getelementptr inbounds %jetvm.types.exec_ctx, ptr %callee_ctx, i32 0, i32 5
  %callee_mem_buf_ptr = getelementptr inbounds %jetvm.types.mem, ptr %callee_mem, i32 0, i32 0
  %callee_mem_loc_ptr = getelementptr inbounds %jetvm.types.mem_buf, ptr %callee_mem_buf_ptr, i32 0, i32 %callee_offset_i32

  %copy_len_i32 = trunc i256 %copy_len to i32
  call void @llvm.memcpy.p0.p0.i32(ptr %caller_mem_loc_ptr, ptr %callee_mem_loc_ptr, i32 %copy_len_i32, i1 false)

  ret i1 0
}

define %jetvm.types.exec_ctx* @jetvm.contracts.new_ctx (%jetvm.types.exec_ctx* %caller_ctx, i256 %gas, i256 %value, i256 %in_off, i256 %in_len, i256 %out_off, i256 %out_len) {
  %ctx = alloca %jetvm.types.exec_ctx
  ; TODO: Add call data to ctx and set it here
  ret %jetvm.types.exec_ctx* %ctx
}

declare i8 @jetvm.contracts.lookup(ptr, ptr, i32) nounwind

define i8 @jetvm.contracts.call(i32 %function_name, %jetvm.types.exec_ctx* %caller_ctx) {
    %fn_ptr_addr = alloca i64*, align 8
    %lookup_result = call i8 @jetvm.contracts.lookup(ptr @jetvm.jit_engine, ptr %fn_ptr_addr, i32 5678)
    %success = icmp eq i8 %lookup_result, 0
    br i1 %success, label %invoke_fn, label %error_handle

invoke_fn:
    ; Load the function
    %fn_ptr = load i8*, i8** %fn_ptr_addr, align 8
    %typed_fn_ptr = bitcast i8* %fn_ptr to %jetvm.types.contract_fn*

    ; Call the function
    %result  = call i8 %typed_fn_ptr(%jetvm.types.exec_ctx* %caller_ctx)
    br i1 %success, label %continue, label %error_handle

error_handle:
    ; Handle error
    ret i8 1

continue:
    ret i8 42
}

; !llvm.module.flags = !{!0, !1, !2, !3, !4}
; !llvm.ident = !{!5}

; !0 = !{i32 2, !"SDK Version", [2 x i32] [i32 14, i32 4]}
; !1 = !{i32 1, !"wchar_size", i32 4}
; !2 = !{i32 8, !"PIC Level", i32 2}
; !3 = !{i32 7, !"uwtable", i32 2}
; !4 = !{i32 7, !"frame-pointer", i32 2}
; !5 = !{!"Apple clang version 15.0.0 (clang-1500.3.9.4)"}
