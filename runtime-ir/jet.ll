; ModuleID = 'JetVM Runtime'
source_filename = "jet.ll"

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

;
; Globals
;
@jet.jit_engine = external global ptr

;
; Types
;
%jet.types.mem_buf = type [32448 x i8]

%jet.types.mem = type <{
  %jet.types.mem_buf *, ; data
  i32, ; length
  i32 ; capacity
}>

%jet.types.block_info = type <{
  i256, ; balance
  i256, ; gas price
  i160, ; address
  i32,  ; code size
  i160  ; origin
}>

%jet.types.call_info = type <{}>

%jet.types.exec_ctx = type <{
  i32, ; stack_ptr
  i32, ; jump_ptr
  i32, ; return offset
  i32, ; return length
  [1024 x i256], ; stack
  %jet.types.mem ; memory
}>

%jet.types.contract_fn = type i8(%jet.types.exec_ctx*)

;
; Runtime functions
;
attributes #0 = { alwaysinline nounwind }

declare i8 @jet.contracts.lookup(ptr, ptr, i8*)

; Pushes a word onto the stack and incs the stack ptr.
; Returns true if the operation was successful, false if the stack is full.
define i1 @jet.stack.push.word (%jet.types.exec_ctx*, i256) #0 {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_offset_ptr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 4, i32 %stack_ptr

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
define i1 @jet.stack.push.bytes (%jet.types.exec_ctx*, [32 x i8]) #0 {
entry:
  ; Cast byte array to word and send to @jet.stack.push.word
  %stack_bytes_ptr = alloca [32 x i8]
  store [32 x i8] %1, [32 x i8]* %stack_bytes_ptr
  %stack_word = load i256, ptr %stack_bytes_ptr, align 8
  %result = call i1 @jet.stack.push.word (%jet.types.exec_ctx* %0, i256 %stack_word)
  ret i1 %result
}

define i256 @jet.stack.pop (%jet.types.exec_ctx* %0) #0 {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_ptr_sub_1 = sub i32 %stack_ptr, 1
  %stack_offset_ptr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 4, i32 %stack_ptr_sub_1

  ; Load word
  %stack_word = load i256, ptr %stack_offset_ptr, align 8

  ; Decrement stack pointer
  store i32 %stack_ptr_sub_1, ptr %stack_ptr_gep, align 4

  ret i256 %stack_word
}

define i8 @jet.mem.store.word (%jet.types.exec_ctx* %ctx, i256 %loc, i256 %val) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  store i256 %val, ptr %mem_loc_ptr, align 1

  ret i8 0
}

define i8 @jet.mem.store.byte (%jet.types.exec_ctx* %ctx, i256 %loc, i256 %val) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32
  %val_i8 = trunc i256 %val to i8

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  store i8 %val_i8, ptr %mem_loc_ptr, align 1
  ret i8 0
}

define i256 @jet.mem.load (%jet.types.exec_ctx* %ctx, i256 %loc) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 5
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  %val = load i256, ptr %mem_loc_ptr, align 1
  ret i256 %val
}

define i1 @jet.mem.copy (%jet.types.exec_ctx* %caller_ctx, i256 %caller_offset, %jet.types.exec_ctx* %callee_ctx, i256 %callee_offset, i256 %copy_len) #0 {
entry:
  %caller_offset_i32 = trunc i256 %caller_offset to i32
  %callee_offset_i32 = trunc i256 %callee_offset to i32

  %caller_mem = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 5
  %caller_mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %caller_mem, i32 0, i32 %caller_offset_i32

  %callee_mem = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 5
  %callee_mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %callee_mem, i32 0, i32 %callee_offset_i32

  %copy_len_i32 = trunc i256 %copy_len to i32
  call void @llvm.memcpy.p0.p0.i32(ptr %caller_mem_loc_ptr, ptr %callee_mem_loc_ptr, i32 %copy_len_i32, i1 false)

  ret i1 0
}

define %jet.types.exec_ctx* @jet.contracts.new_sub_ctx (%jet.types.exec_ctx* %caller_ctx, i256 %gas, i256 %value, i256 %in_off, i256 %in_len, i256 %out_off, i256 %out_len) #0 {
  %ctx = alloca %jet.types.exec_ctx
  ; TODO: Add call data to ctx and set it here
  ret %jet.types.exec_ctx* %ctx
}

define i8 @jet.contracts.call(%jet.types.exec_ctx* %caller_ctx, %jet.types.exec_ctx* %callee_ctx, i256 %addr) #0 {
    ; Convert the 256bit address to an array of 20 bytes (160 bits)
    %addr_i160 = trunc i256 %addr to i160
    %addr_i160_ptr = alloca i160, align 8
    store i160 %addr_i160, i160* %addr_i160_ptr
    %addr_bytes = bitcast i160* %addr_i160_ptr to i8*

    ; Lookup the function
    %fn_ptr_addr = alloca i64*, align 8
    %lookup_result = call i8 @jet.contracts.lookup(ptr @jet.jit_engine, ptr %fn_ptr_addr, i8* %addr_bytes)
    %success = icmp eq i8 %lookup_result, 0
    br i1 %success, label %invoke_fn, label %error_handle

invoke_fn:
    ; Load the function
    %fn_ptr = load i8*, i8** %fn_ptr_addr, align 8
    %typed_fn_ptr = bitcast i8* %fn_ptr to %jet.types.contract_fn*

    ; Call the function
    %result  = call i8 %typed_fn_ptr(%jet.types.exec_ctx* %callee_ctx)
    br i1 %success, label %continue, label %error_handle

    ; TODO: Copy return data from callee_ctx to caller_ctx

error_handle:
    ret i8 1

continue:
    ret i8 42
}
