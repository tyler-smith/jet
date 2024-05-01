cmake_minimum_required(VERSION 3.28)

set(JET_VERSION 0.0.1)
set(JET_DESCRIPTION "An EVM JIT environment")

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_COMPILER /usr/local/opt/llvm/bin/clang)

project(jet VERSION ${JET_VERSION} DESCRIPTION ${JET_DESCRIPTION} LANGUAGES C)

# Color output and build database for clang-tidy
add_compile_options (-fdiagnostics-color=always)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Set output directory for build artifacts
set(LIBRARY_OUTPUT_PATH ./dist)
set(ARCHIVE_OUTPUT_PATH ./dist)
set(EXECUTABLE_OUTPUT_PATH ./dist/bin)

# Build runtime libs and llvm ir
add_library(jetvm-runtime SHARED runtime/runtime.c)

add_custom_command(TARGET jetvm-runtime
    POST_BUILD
    COMMAND clang -S -emit-llvm -o dist/libjetvm-runtime.ll ${PROJECT_SOURCE_DIR}/runtime/runtime.c
)
