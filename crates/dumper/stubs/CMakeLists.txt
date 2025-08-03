cmake_minimum_required(VERSION 3.20)

project(stubs LANGUAGES C)

set(CMAKE_C_STANDARD_VERSION 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

#set(LIBCXX_CXX_ABI libcxxabi)

add_library(stubs STATIC
    src/common.h    
    src/hack.c
    src/stub.c
    src/stub_asl.c
    src/stub_audio.c
    src/stub_dispatch.c
    src/stub_fs.c
    src/stub_io.c
    src/stub_kernel.c
    src/stub_kernel_kqueue.c
    src/stub_math.c
    src/stub_misc.c
    src/stub_nvcg.c
    src/stub_objc.c
    src/stub_process.c
    src/stub_pthread.c
    src/stub_resolv.c
    src/stub_rune.c
    src/stub_socket.c
    src/stub_str.c
    src/stub_sys.c
    src/stub_video.c
    src/stub_xpc.c
)
target_compile_options(stubs PRIVATE 
    -Wno-nullability-completeness 
    -Wno-c23-extensions
    -Wno-unused-parameter
    -Wno-unused-function
    #-fvisibility=default
)
target_include_directories(stubs PRIVATE src/)
