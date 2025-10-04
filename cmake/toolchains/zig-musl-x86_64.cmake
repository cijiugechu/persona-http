set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR x86_64)

if(NOT DEFINED ENV{GITHUB_WORKSPACE})
  set(WORKSPACE "${CMAKE_CURRENT_LIST_DIR}/../..")
else()
  set(WORKSPACE "$ENV{GITHUB_WORKSPACE}")
endif()

set(CMAKE_C_COMPILER "${WORKSPACE}/ci/zig-cc-wrapper.sh")
set(CMAKE_C_COMPILER_ARG1 "x86_64-linux-musl")

set(CMAKE_CXX_COMPILER "${WORKSPACE}/ci/zig-cxx-wrapper.sh")
set(CMAKE_CXX_COMPILER_ARG1 "x86_64-linux-musl")

set(CMAKE_ASM_COMPILER "${WORKSPACE}/ci/zig-cc-wrapper.sh")
set(CMAKE_ASM_COMPILER_ARG1 "x86_64-linux-musl")

set(CMAKE_AR "${WORKSPACE}/ci/zig-ar-wrapper.sh" CACHE FILEPATH "Archiver")
set(CMAKE_RANLIB "${WORKSPACE}/ci/zig-ranlib-wrapper.sh" CACHE FILEPATH "Ranlib")

# Disable assembly for BoringSSL when cross-compiling with Zig
set(OPENSSL_NO_ASM ON CACHE BOOL "Disable assembly" FORCE)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
