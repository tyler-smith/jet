; ModuleID = 'JetVM Runtime'
source_filename = "jet/runtime/runtime.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx14.0.0"


%exec_ctx_t = type <{ 
  i32, ; stack_ptr
  i32, ; jump_ptr
  i32, ; return offset
  i32, ; return length
  [1024 x i256]
}>

%block_info_t = type <{
  i160, ; address
  i256, ; balance
  i160, ; origin
  i160, ; caller
  i256, ; call_value
  i8*,  ; call_data
  i32,  ; call_data_size
  i32,  ; code size
  i256 ; gas price
}>
%call_info_t = type <{}>

@exec_ctx_stack_idx = constant i32 4
; @exec_ctx_stack_idx = global i32 4

declare void @_keccak256(i8*, i8*) nounwind


; define [32 x i8]* @_call_keccak256(i8* %input){
; entry:
;   %result = alloca [32 x i8]

;   call void @_keccak256(i8* %input, i8* %result)

;   ret [32 x i8]* %result
; }


; define i256 @_call_keccak256(i8* %input){
; entry:
;   %result = alloca [32 x i8]

;   call void @_keccak256(i8* %input, i8* %result)
;   %result_word = load i256, ptr %result, align 8

;   ret i256 %result_word
; }


define i256 @_call_keccak256([32 x i8]* %input){
entry:
  %result = alloca [32 x i8]

  call void @_keccak256(i8* %input, i8* %result)
  %result_word = load i256, ptr %result, align 8

  ret i256 %result_word
}



; stack_push_word pushes a word onto the stack and increments the stack pointer.
; Returns true if the operation was successful, false if the stack is full.
define i1 @stack_push_word (%exec_ctx_t*, i256) {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %exec_ctx_t, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_offset_ptr = getelementptr inbounds %exec_ctx_t, ptr %0, i32 0, i32 4, i32 %stack_ptr

  ; TODO: Check if we'll break the stack
  
  ; Store word
  store i256 %1, ptr %stack_offset_ptr, align 8

  ; Increment stack pointer
  %stack_ptr_next = add i32 %stack_ptr, 1
  store i32 %stack_ptr_next, ptr %stack_ptr_gep, align 4

  ret i1 true
}

; stack_push_bytes pushes an array of 32 bytes onto the stack as a word, and
; increments the stack pointer.
; Returns true if the operation was successful, false if the stack is full.
define i1 @stack_push_bytes (%exec_ctx_t*, [32 x i8]) {
entry:
  ; Cast byte array to word
  ; %stack_word = bitcast [32 x i8]* %1 to i256

  %stack_bytes_ptr = alloca [32 x i8]
  store [32 x i8] %1, [32 x i8]* %stack_bytes_ptr
  
  %stack_word = load i256, ptr %stack_bytes_ptr, align 8

  ; Call stack_push_word
  %result = call i1 @stack_push_word (%exec_ctx_t* %0, i256 %stack_word)

  ret i1 %result
}



define i256 @stack_pop_word (%exec_ctx_t* %0) {
entry:
  ; Load stack pointer
  %stack_ptr_gep = getelementptr inbounds %exec_ctx_t, ptr %0, i32 0, i32 0
  %stack_ptr = load i32, ptr %stack_ptr_gep, align 4
  %stack_ptr_sub_1 = sub i32 %stack_ptr, 1
  %stack_offset_ptr = getelementptr inbounds %exec_ctx_t, ptr %0, i32 0, i32 4, i32 %stack_ptr_sub_1

  ; Load word
  %stack_word = load i256, ptr %stack_offset_ptr, align 8

  ; Decrement stack pointer
  store i32 %stack_ptr_sub_1, ptr %stack_ptr_gep, align 4

  ; ret i256 02
  ret i256 %stack_word
}


; !llvm.module.flags = !{!0, !1, !2, !3, !4}
; !llvm.ident = !{!5}

; !0 = !{i32 2, !"SDK Version", [2 x i32] [i32 14, i32 4]}
; !1 = !{i32 1, !"wchar_size", i32 4}
; !2 = !{i32 8, !"PIC Level", i32 2}
; !3 = !{i32 7, !"uwtable", i32 2}
; !4 = !{i32 7, !"frame-pointer", i32 2}
; !5 = !{!"Apple clang version 15.0.0 (clang-1500.3.9.4)"}
