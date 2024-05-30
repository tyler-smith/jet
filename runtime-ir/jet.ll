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
; Declarations of Rust-defined functions
;

declare i8 @jet.contracts.lookup(ptr, ptr, i8*)
declare %jet.types.exec_ctx* @jet.contracts.new_sub_ctx ()
declare i8 @jet.contracts.call_return_data_copy(ptr, ptr, i32, i32, i32)
declare i8 @jet.ops.keccak256(ptr)

;
; Types
;
%jet.types.mem_buf = type [1024 x i8]

%jet.types.mem = type <{
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
  ptr, ; sub call ctx

  [1024 x i256], ; stack

  [1024 x i8], ; mem buffer
  i32,         ; mem length
  i32          ; mem capacity
}>

%jet.types.contract_fn = type i8(%jet.types.exec_ctx*)

;
; Runtime functions
;
attributes #0 = { alwaysinline nounwind }


; Pushes a word onto the stack and incs the stack ptr.
; Returns true if the operation was successful, false if the stack is full.
define i1 @jet.stack.push.word (%jet.types.exec_ctx*, i256) #0 {
entry:
  ; Load stack pointer
  %stack.ptr.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack.ptr = load i32, ptr %stack.ptr.addr
  %stack.top.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 5, i32 %stack.ptr

  ; Increment stack pointer
  %stack.ptr.next = add i32 %stack.ptr, 1

  ; Check for stack overflow
  %overflow = icmp ugt i32 %stack.ptr.next, 1024
  br i1 %overflow, label %stack_overflow, label %store

 store:
  ; Store pointer and word and return success
  store i32 %stack.ptr.next, ptr %stack.ptr.addr
  store i256 %1, ptr %stack.top.addr
  ret i1 true

stack_overflow:
  ret i1 false
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

define { i1, i256 } @jet.stack.pop (%jet.types.exec_ctx* %0) #0 {
entry:
  ; Load stack pointer
  %stack.ptr.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 0
  %stack.ptr = load i32, ptr %stack.ptr.addr, align 4
  %stack.ptr.next = sub i32 %stack.ptr, 1

  ; Check for stack underflow
  %underflow = icmp eq i32 %stack.ptr.next, 0
  br i1 %underflow, label %stack_underflow, label %load

load: 
  ; Load word and return success
  %stack.top.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %0, i32 0, i32 5, i32 %stack.ptr.next
  %stack_word = load i256, ptr %stack.top.addr, align 8
  store i32 %stack.ptr.next, ptr %stack.ptr.addr, align 4

  %ret.ptr = alloca { i1, i256 }
  %ret.error.ptr = getelementptr inbounds { i1, i256 }, ptr %ret.ptr, i32 0, i32 0
  %ret.word.ptr = getelementptr inbounds { i1, i256 }, ptr %ret.ptr, i32 0, i32 1
  store i1 true, ptr %ret.error.ptr
  store i256 %stack_word, ptr %ret.word.ptr 
  %ret = load { i1, i256 }, ptr %ret.ptr

  ret { i1, i256 } %ret

stack_underflow:
  ret { i1, i256 } { i1 false, i256 0 }
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

; @jet.contracts.call implements an internal call to another contract.
; Parameters:
;   %caller_ctx - The context of the calling contract
;   %callee_ctx - The context of the called contract
;   %addr - The address of the contract to call
;   %ret.dest - The offset in the caller's memory to copy to
;   %ret.len - The length of the data to copy
; Returns:
;   0 - Success
;   1 - Lookup failed
;   2 - Invocation failed
;   3 - Return data out of callee's return data bounds
;   4 - Return data out of caller's memory bounds
define i8 @jet.contracts.call(%jet.types.exec_ctx* %caller_ctx, %jet.types.exec_ctx* %callee_ctx, i160 %addr, i32 %ret.dest, i32 %ret.len) #0 {
entry:
    ; Convert the 160bit address to an array of 20 bytes
    %addr_i160_ptr = alloca i160, align 8
    store i160 %addr, i160* %addr_i160_ptr
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
    br i1 %success, label %set_return_info, label %return
set_return_info:
    %caller.sub_ctx.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %caller_ctx, i32 0, i32 4
    store %jet.types.exec_ctx* %callee_ctx, %jet.types.exec_ctx** %caller.sub_ctx.addr

    %callee.return.len.addr = getelementptr inbounds %jet.types.exec_ctx, ptr %callee_ctx, i32 0, i32 3
    %callee.return.len = load i32, i32* %callee.return.len.addr
    %callee.return.empty = icmp eq i32 %callee.return.len, 0
    br i1 %callee.return.empty, label %return, label %copy_return_data
copy_return_data:
    %copy.ret = call i8 @jet.contracts.call_return_data_copy(ptr %caller_ctx, ptr %callee_ctx, i32 %ret.dest, i32 0, i32 %ret.len)
    br label %return
return:
    %r = phi i8 [%copy.ret, %copy_return_data], [0, %set_return_info], [2, %invoke_fn], [1, %entry]
    ret i8 %r
}
