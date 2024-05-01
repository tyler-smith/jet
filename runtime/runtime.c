#include <stdio.h>
#include <stdlib.h>

int const WORD_SIZE_BYTES = 32;
int const STACK_SIZE_BYTES = 32 * 1024;

typedef struct Context {
    unsigned int stackPointer;
    unsigned int jumpPointer;
    unsigned int returnOffset;
    unsigned int returnLength;

    unsigned char stack[STACK_SIZE_BYTES];
//    unsigned char ram[256 * 1024];
//    unsigned char storage[256 * 1024];
} Context;

// Constructor function for Context
//Context *NewContext() {
//    Context *ctx = (Context *) malloc(sizeof(Context));
//    return ctx;
//}

char stack_push(Context *ctx, const unsigned char *data) {
    if (ctx->stackPointer + WORD_SIZE_BYTES > STACK_SIZE_BYTES) {
        return 0;
    }

    for (int i = 0; i < WORD_SIZE_BYTES; i++) {
        ctx->stack[ctx->stackPointer++] = data[i];
    }
    return 1;
}

char stack_push_i8(Context *ctx, char data) {
    unsigned char dataArr[WORD_SIZE_BYTES];
    dataArr[0] = data;
    return stack_push(ctx, dataArr);
}

char stack_push_i16(Context *ctx, short data) {
    unsigned char dataArr[WORD_SIZE_BYTES];
    dataArr[0] = (data >> 8) & 0xFF;
    dataArr[1] = data & 0xFF;
    return stack_push(ctx, dataArr);
}

char stack_pop(Context *ctx, unsigned char *data) {
    if (ctx->stackPointer < WORD_SIZE_BYTES) {
        return 0;
    }

    for (int i = 0; i < WORD_SIZE_BYTES; i++) {
        data[i] = ctx->stack[--ctx->stackPointer];
    }
    return 1;
}

char stack_pop_i8(Context *ctx, char *data) {
    unsigned char dataArr[WORD_SIZE_BYTES];
    if (!stack_pop(ctx, dataArr)) {
        return 0;
    }
    *data = dataArr[0];
    return 1;
}

char stack_pop_i16(Context *ctx, short *data) {
    unsigned char dataArr[WORD_SIZE_BYTES];
    if (!stack_pop(ctx, dataArr)) {
        return 0;
    }
    *data = (dataArr[0] << 8) | dataArr[1];
    return 1;
}


int x = 0;
void PrintExecContext(Context *ctx) {
    if(x++ > 100) {
        //exit(11);
    }

    printf("StackSize2: %d\n"
           "JumpPtr: %d\n"
           "Return Offset: %d\n"
           "Return Length: %d\n",
           ctx->stackPointer, ctx->jumpPointer, ctx->returnOffset, ctx->returnLength);
    for (int i = 0; i < 3; i++){
        for (int j = 0; j < 256; j++){
            printf("%02X", ctx->stack[(i*256)+j]);
        }
        printf("\n");
    }
}

void jetvmPrintI8(char a) {
    printf("int8: %d\n", a);
}

void jetvmPrintI16(short a) {
    printf("int16: %d\n", a);
}

void jetvmPrintI32(int a) {
    printf("int32: %d\n", a);
}

void jetvmPrintI64(long long a) {
    printf("int64: %lld\n", a);
}
