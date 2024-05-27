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
%jet.types.mem_buf = type [1024 x i8]

%jet.types.mem = type <{
  ;%jet.types.mem_buf *, ; data
  %jet.types.mem_buf, ; data
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
  i32, ; stack.ptr
  i32, ; jump_ptr
  i32, ; return offset
  i32, ; return length
  ptr,; sub call ctx
  [1024 x i256], ; stack

  [1024 x i8], ; memory
  i32, ; mem length
  i32 ; mem capacity
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
  %stack.ptr.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack.ptr = load i32, ptr %stack.ptr.addr
  %stack.top.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 5, i32 %stack.ptr

  ; TODO: Check if we'll break the stack

  ; Store word
  store i256 %1, ptr %stack.top.addr

  ; Increment stack pointer
  %stack.ptr.next = add i32 %stack.ptr, 1
  store i32 %stack.ptr.next, ptr %stack.ptr.addr

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
  %stack.ptr.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack.ptr = load i32, ptr %stack.ptr.addr, align 4
  %stack.ptr.sub_1 = sub i32 %stack.ptr, 1
  %stack.top.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 5, i32 %stack.ptr.sub_1

  ; Load word
  %stack_word = load i256, ptr %stack.top.addr, align 8

  ; Decrement stack pointer
  store i32 %stack.ptr.sub_1, ptr %stack.ptr.addr, align 4

  ret i256 %stack_word
}

define i8 @jet.mem.store.word (%jet.types.exec_ctx* %ctx, i256 %loc, i256 %val) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  store i256 %val, ptr %mem_loc_ptr, align 1

  ret i8 0
}

define i8 @jet.mem.store.byte (%jet.types.exec_ctx* %ctx, i256 %loc, i256 %val) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32
  %val_i8 = trunc i256 %val to i8

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  store i8 %val_i8, ptr %mem_loc_ptr, align 1
  ret i8 0
}

define i256 @jet.mem.load (%jet.types.exec_ctx* %ctx, i256 %loc) #0 {
entry:
  %loc_i32 = trunc i256 %loc to i32

  %mem = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
  %mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %mem, i32 0, i32 %loc_i32

  %val = load i256, ptr %mem_loc_ptr, align 1
  ret i256 %val
}

; define i1 @jet.mem.copy_to_return (%jet.types.exec_ctx* %ctx, i32 %offset, i32 %length) #0 {
; entry:
;   %mem.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
;   %mem.offset.addr = getelementptr inbounds %jet.types.mem_buf, ptr %mem.addr, i32 0, i32 %offset

;   %return_data.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
;   ;%return_data.offset.addr = getelementptr inbounds [0 x i8], ptr %return_data.addr, i32 0, i32 %callee_offset_i32

;   call void @llvm.memcpy.p0.p0.i32(ptr %mem.offset.addr, ptr %return_data.addr, i32 %length, i1 false)

;   ret i1 0
; }

; define i1 @jet.mem.copy_sub_ctx_return (%jet.types.exec_ctx* %caller_ctx, i256 %caller_offset, %jet.types.exec_ctx* %callee_ctx, i256 %callee_offset, i256 %copy_len) #0 {
; entry:
;   %caller_offset_i32 = trunc i256 %caller_offset to i32
;   %callee_offset_i32 = trunc i256 %callee_offset to i32

;   %caller_mem = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 5
;   %caller_mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %caller_mem, i32 0, i32 %caller_offset_i32

;   %callee_mem = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 5
;   %callee_mem_loc_ptr = getelementptr inbounds %jet.types.mem_buf, ptr %callee_mem, i32 0, i32 %callee_offset_i32

;   %copy_len_i32 = trunc i256 %copy_len to i32
;   call void @llvm.memcpy.p0.p0.i32(ptr %caller_mem_loc_ptr, ptr %callee_mem_loc_ptr, i32 %copy_len_i32, i1 false)

;   ret i1 0
; }

declare %jet.types.exec_ctx* @jet.contracts.new_sub_ctx ()
;declare %jet.types.exec_ctx* @jet.contracts.new_sub_ctx (%jet.types.exec_ctx* %caller_ctx, i256 %gas, i256 %value, i256 %in_off, i256 %in_len, i256 %out_off, i256 %out_len)
; define %jet.types.exec_ctx* @jet.contracts.new_sub_ctx (%jet.types.exec_ctx* %caller_ctx, i256 %gas, i256 %value, i256 %in_off, i256 %in_len, i256 %out_off, i256 %out_len) #0 {
;   ; %mem = alloca %jet.types.mem
;   ;%ctx = alloca %jet.types.exec_ctx
;   %ctx = malloc %jet.types.exec_ctx

;   ; %ctx.mem.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %ctx, i32 0, i32 6
;   ; store %jet.types.mem* %mem, %jet.types.mem* %ctx.mem.addr


;   ; TODO: Add call data to ctx and set it here
;   ret %jet.types.exec_ctx* %ctx
; }

; ;define i8 @jet.contracts.call(%jet.types.exec_ctx* %caller_ctx, i256 %addr) #0 {
; define i8 @jet.contracts.call_old(%jet.types.exec_ctx* %caller_ctx, %jet.types.exec_ctx* %callee_ctx, i256 %addr) #0 {
; ;define i8 @jet.contracts.call(%jet.types.exec_ctx* %caller_ctx, i256 %addr) #0 {
;     ;%callee_ctx = call %jet.types.exec_ctx* @jet.contracts.new_sub_ctx()

;     ; Convert the 256bit address to an array of 20 bytes (160 bits)
;     %addr_i160 = trunc i256 %addr to i160
;     %addr_i160_ptr = alloca i160, align 8
;     store i160 %addr_i160, i160* %addr_i160_ptr
;     %addr_bytes = bitcast i160* %addr_i160_ptr to i8*

;     ; Lookup the function
;     %fn_ptr_addr = alloca i64*, align 8
;     %lookup_result = call i8 @jet.contracts.lookup(ptr @jet.jit_engine, ptr %fn_ptr_addr, i8* %addr_bytes)
;     %success = icmp eq i8 %lookup_result, 0
;     br i1 %success, label %invoke_fn, label %error_lookup

; invoke_fn:
;     ; Load the function
;     %fn_ptr = load i8*, i8** %fn_ptr_addr, align 8
;     %typed_fn_ptr = bitcast i8* %fn_ptr to %jet.types.contract_fn*

;     ; Call the function
;     %result = call i8 %typed_fn_ptr(%jet.types.exec_ctx* %callee_ctx)
;     br i1 %success, label %set_sub_ctx, label %error_invoke

; set_sub_ctx:
;     %caller.sub_ctx.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 4
;     store %jet.types.exec_ctx* %callee_ctx, %jet.types.exec_ctx** %caller.sub_ctx.addr

;     %callee.return.len.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 3
;     %callee.return.len = load i32, i32* %callee.return.len.addr
;     %callee.return.empty = icmp eq i32 %callee.return.len, 0
;     br i1 %callee.return.empty, label %return, label %copy_return_data


; copy_return_data:
;     %callee.return.off.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 2
;     %callee.return.off = load i32, i32* %callee.return.off.addr

;     %ret.copy.result = call i1 @jet.mem.copy_to_return(%jet.types.exec_ctx* %callee_ctx, i32 %ret.off, i32 %ret.len)
;     br i1 %ret.copy.result, label %return, label %error_return_copy

; error_lookup:
;     ret i8 1
; error_invoke:
;     ret i8 2
; return:
;     ret i8 0
; }

; define i8 @jet.contracts.call_bk(%jet.types.exec_ctx* %caller_ctx, %jet.types.exec_ctx* %callee_ctx, i256 %addr) #0 {
; entry:
;     ; Convert the 256bit address to an array of 20 bytes (160 bits)
;     %addr_i160 = trunc i256 %addr to i160
;     %addr_i160_ptr = alloca i160, align 8
;     store i160 %addr_i160, i160* %addr_i160_ptr
;     %addr_bytes = bitcast i160* %addr_i160_ptr to i8*

;     ; Lookup the function
;     %fn_ptr_addr = alloca i64*, align 8
;     %lookup_result = call i8 @jet.contracts.lookup(ptr @jet.jit_engine, ptr %fn_ptr_addr, i8* %addr_bytes)
;     %success = icmp eq i8 %lookup_result, 0
;     br i1 %success, label %invoke_fn, label %return
; invoke_fn:
;     ; Load the function
;     %fn_ptr = load i8*, i8** %fn_ptr_addr, align 8
;     %typed_fn_ptr = bitcast i8* %fn_ptr to %jet.types.contract_fn*

;     ; Call the function
;     %result = call i8 %typed_fn_ptr(%jet.types.exec_ctx* %callee_ctx)
;     br i1 %success, label %set_sub_ctx, label %return
; set_sub_ctx:
;     %caller.sub_ctx.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 4
;     store %jet.types.exec_ctx* %callee_ctx, %jet.types.exec_ctx** %caller.sub_ctx.addr

;     %callee.return.len.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 3
;     %callee.return.len = load i32, i32* %callee.return.len.addr
;     %callee.return.empty = icmp eq i32 %callee.return.len, 0
;     br return
; return:
;     %r = phi i8 [0, %set_sub_ctx], [2, %invoke_fn], [3, %entry]
;     ret i8 %r
; }


declare i8 @jet.helpers.copy_return_data(ptr, ptr, i32, i32, i32)

define i8 @jet.contracts.call(%jet.types.exec_ctx* %caller_ctx, %jet.types.exec_ctx* %callee_ctx, i256 %addr, i32 %ret.off, i32 %ret.len) #0 {
entry:
    ; Convert the 256bit address to an array of 20 bytes (160 bits)
    %addr_i160 = trunc i256 %addr to i160
    %addr_i160_ptr = alloca i160, align 8
    store i160 %addr_i160, i160* %addr_i160_ptr
    %addr_bytes = bitcast i160* %addr_i160_ptr to i8*

    ; Lookup the function
    %fn_ptr_addr = alloca i64*, align 8
    %lookup_result = call i8 @jet.contracts.lookup(ptr @jet.jit_engine, ptr %fn_ptr_addr, i8* %addr_bytes)
    %success = icmp eq i8 %lookup_result, 0
    br i1 %success, label %invoke_fn, label %return
invoke_fn:
    ; Load the function
    %fn_ptr = load i8*, i8** %fn_ptr_addr, align 8
    %typed_fn_ptr = bitcast i8* %fn_ptr to %jet.types.contract_fn*

    ; Call the function
    %result = call i8 %typed_fn_ptr(%jet.types.exec_ctx* %callee_ctx)
    br i1 %success, label %set_sub_ctx, label %return
set_sub_ctx:
    %caller.sub_ctx.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 4
    store %jet.types.exec_ctx* %callee_ctx, %jet.types.exec_ctx** %caller.sub_ctx.addr

    %callee.return.len.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 3
    %callee.return.len = load i32, i32* %callee.return.len.addr
    %callee.return.empty = icmp eq i32 %callee.return.len, 0
    br i1 %callee.return.empty, label %return, label %check_return_data_bounds
check_return_data_bounds:
    ; TODO: Check if the return data is within bounds
    br label %copy_return_data
copy_return_data:
    ; %copy.ret = call i8 @jet.helpers.copy_return_data(ptr %caller_ctx, ptr %callee_ctx, i32 %ret.off, i32 %ret.len, i32 %ret.size)
    br label %return
return:
    ; %r = phi i8 [0, %set_sub_ctx], [2, %invoke_fn], [3, %entry]
    ; ret i8 %r
    ret i8 0
}
